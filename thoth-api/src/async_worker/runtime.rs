//! The runtime (spec section 6): startup, the pools, the claim loop, the slot
//! threads, the heartbeat, the liveness ticker and shutdown.
//!
//! Threads: `main` runs startup, waits for the stop signal and runs the
//! shutdown; the liveness ticker, the claim loop and the heartbeat each run
//! their body inside `catch_unwind`; `--slots` slot threads each run one
//! handler call at a time and catch handler panics per call; short-lived
//! fallback threads run only during shutdown. No thread here uses an async
//! runtime.
//!
//! In-flight state is a map keyed by the claim token (one attempt for all
//! time), never by job id. Free slots are counted from idle slot threads. The
//! map lock is never held across an engine call; a claim's renewal lock is,
//! by design (spec section 5.1).

use std::collections::HashMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Condvar, Mutex, MutexGuard, OnceLock};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use diesel::connection::SimpleConnection;
use diesel::migration::MigrationSource;
use diesel::pg::{Pg, PgConnection};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, CustomizeConnection, HandleError, Pool};
use diesel::sql_query;
use diesel::sql_types::{Bool, Text};
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

use super::config::{check_aws_sources, WorkerArgs, WorkerConfig, WORKER_PRINCIPAL};
use super::handler::{HandlerRegistry, PRODUCTION_HANDLERS};
use super::health::{unix_now, withhold_reason, write_liveness, LivenessInputs, LivenessLimits};
use crate::db::{PgPool, MIGRATIONS};
use crate::model::async_job::{
    with_engine_transaction, AsyncEffectPhase, AsyncJobKindPolicy, AsyncJobRegistry, Executor,
    ExecutorDomain, RevisionMode, RiskClass, WorkerClaim, PRODUCTION_REGISTRY,
};

#[cfg(test)]
use super::test_support::TestHooks;

mod claims;
mod heartbeat;
mod slots;

pub use claims::{compute_claim_sets, ClaimSets};
#[cfg(test)]
pub(crate) use heartbeat::due_claims;
pub(crate) use slots::Assignment;
#[cfg(test)]
pub(crate) use slots::{db_clock, deadline_guard, plan_resume, settled_at, ResumePlan};

// ---------------------------------------------------------------------------
// Fixed constants (spec section 4). Step 3 relies on them.
// ---------------------------------------------------------------------------

/// The liveness ticker period (the health max age must be at least three of
/// these).
pub const LIVENESS_CADENCE: Duration = Duration::from_secs(5);
/// The session `statement_timeout` of engine and heartbeat connections.
pub const ENGINE_STATEMENT_TIMEOUT: Duration = Duration::from_secs(15);
/// The r2d2 `connection_timeout` of the engine and heartbeat pools.
pub const ENGINE_POOL_CHECKOUT: Duration = Duration::from_secs(5);
/// A heartbeat not cycling (stops claims and liveness), or renewal health
/// stale with claims in flight (stops claims).
pub const HEARTBEAT_STALL: Duration = Duration::from_secs(30);
/// Claim-loop progress older than this stops liveness.
pub const LOOP_STALL: Duration = Duration::from_secs(120);
/// A slot hung longer than this stops liveness.
pub const HUNG_SLOT_LIMIT: Duration = Duration::from_secs(300);
/// The whole post-grace fallback phase.
pub const FALLBACK_PHASE_BOUND: Duration = Duration::from_secs(20);
/// How long shutdown waits for an in-flight claim call.
pub const CLAIM_LOOP_JOIN_BOUND: Duration = Duration::from_secs(25);
/// The effect lease's margin above the effect window, in seconds.
pub const EFFECT_MARGIN_SECONDS: i32 = 10;
/// Added to `poll_interval_min_seconds` in every `resume_at`, in seconds.
pub const RESUME_MARGIN_SECONDS: i32 = 10;

/// The session settings every engine and heartbeat connection gets on
/// acquire.
const SESSION_SETTINGS: &str =
    "SET statement_timeout = '15s'; SET idle_in_transaction_session_timeout = '60s'";
/// At most this many jobs per claim call (the engine's bound).
pub(crate) const CLAIM_BATCH_MAX: usize = 50;
/// The `recover_on` limit when no slot is free.
pub(crate) const RECOVER_LIMIT: i64 = 50;
/// A liveness write failure is logged at most this often.
const WRITE_FAILURE_LOG_INTERVAL: Duration = Duration::from_secs(3600);

/// `E = max(lease_default_seconds, effect_window_seconds + EFFECT_MARGIN)`:
/// the effect lease request (spec section 5.1).
pub(crate) fn effect_lease_request(policy: &AsyncJobKindPolicy) -> i32 {
    policy.lease_default_seconds.max(
        policy
            .effect_window_seconds
            .unwrap_or(0)
            .saturating_add(EFFECT_MARGIN_SECONDS),
    )
}

/// The smallest delay of a `resume_at`: `poll_interval_min_seconds +
/// RESUME_MARGIN`.
pub(crate) fn minimum_resume_delay(policy: &AsyncJobKindPolicy) -> chrono::Duration {
    chrono::Duration::seconds(i64::from(policy.poll_interval_min_seconds))
        + chrono::Duration::seconds(i64::from(RESUME_MARGIN_SECONDS))
}

// ---------------------------------------------------------------------------
// Timings: the fixed constants and the configured intervals, settable to
// small values by a test-only constructor (spec section 8).
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub(crate) struct Timings {
    pub poll: Duration,
    pub grace: Duration,
    /// The `BUSY` retry delays of final calls and effect marks (1, 2, 4 s).
    pub busy_retries: Vec<Duration>,
    /// The one retry of a shutdown fallback (1 s).
    pub fallback_retry: Duration,
    pub liveness_cadence: Duration,
    pub heartbeat_stall: Duration,
    pub loop_stall: Duration,
    pub hung_slot_limit: Duration,
    pub claim_loop_join_bound: Duration,
    pub fallback_phase_bound: Duration,
    /// The heartbeat wakes this often (1 s).
    pub heartbeat_wake: Duration,
    /// The summary line period (60 s).
    pub summary_interval: Duration,
    /// The pending-migration re-check period (30 s).
    pub migration_recheck: Duration,
    /// The first and the largest database-connection back-off (1 s, 30 s).
    pub connect_backoff_start: Duration,
    pub connect_backoff_cap: Duration,
    /// The largest claim back-off (60 s).
    pub busy_backoff_cap: Duration,
    /// The pools' checkout timeout (`ENGINE_POOL_CHECKOUT`).
    pub pool_checkout: Duration,
    /// Test only: renew this often instead of at a third of the default
    /// lease.
    pub renew_interval: Option<Duration>,
    /// Jitter the idle poll by plus or minus 20 percent.
    pub jitter: bool,
}

impl Timings {
    pub(crate) fn production(config: &WorkerConfig) -> Self {
        Timings {
            poll: Duration::from_secs(config.poll_seconds),
            grace: Duration::from_secs(config.shutdown_grace_seconds),
            busy_retries: vec![
                Duration::from_secs(1),
                Duration::from_secs(2),
                Duration::from_secs(4),
            ],
            fallback_retry: Duration::from_secs(1),
            liveness_cadence: LIVENESS_CADENCE,
            heartbeat_stall: HEARTBEAT_STALL,
            loop_stall: LOOP_STALL,
            hung_slot_limit: HUNG_SLOT_LIMIT,
            claim_loop_join_bound: CLAIM_LOOP_JOIN_BOUND,
            fallback_phase_bound: FALLBACK_PHASE_BOUND,
            heartbeat_wake: Duration::from_secs(1),
            summary_interval: Duration::from_secs(60),
            migration_recheck: Duration::from_secs(30),
            connect_backoff_start: Duration::from_secs(1),
            connect_backoff_cap: Duration::from_secs(30),
            busy_backoff_cap: Duration::from_secs(60),
            pool_checkout: ENGINE_POOL_CHECKOUT,
            renew_interval: None,
            jitter: true,
        }
    }

    /// How often a claim of `policy` is renewed: a third of its default
    /// lease, before and after effect start alike.
    pub(crate) fn renew_every(&self, policy: &AsyncJobKindPolicy) -> Duration {
        self.renew_interval.unwrap_or_else(|| {
            Duration::from_millis(
                u64::try_from(policy.lease_default_seconds).unwrap_or(15) * 1000 / 3,
            )
        })
    }
}

// ---------------------------------------------------------------------------
// Claims and their tracked state (spec sections 5.1 and 6.1)
// ---------------------------------------------------------------------------

/// What a claim was taken for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Purpose {
    Execution,
    Reconciliation,
}

impl Purpose {
    pub fn as_str(self) -> &'static str {
        match self {
            Purpose::Execution => "EXECUTION",
            Purpose::Reconciliation => "RECONCILIATION",
        }
    }
}

/// The effect phase the runtime tracks for a claim: the claim's phase, moved
/// only by a successful T4 or T5 of this attempt, or to `PossiblyStarted`
/// when a T4's outcome is unknown (spec section 6.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackedPhase {
    PreWrite,
    EffectStarted,
    EffectConfirmed,
    /// A T4 may have committed; every fallback treats it as after effect
    /// start.
    PossiblyStarted,
}

impl TrackedPhase {
    pub fn from_phase(phase: AsyncEffectPhase) -> Self {
        match phase {
            AsyncEffectPhase::PreWrite => TrackedPhase::PreWrite,
            AsyncEffectPhase::EffectStarted => TrackedPhase::EffectStarted,
            AsyncEffectPhase::EffectConfirmed => TrackedPhase::EffectConfirmed,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            TrackedPhase::PreWrite => "PRE_WRITE",
            TrackedPhase::EffectStarted => "EFFECT_STARTED",
            TrackedPhase::EffectConfirmed => "EFFECT_CONFIRMED",
            TrackedPhase::PossiblyStarted => "POSSIBLY_STARTED",
        }
    }

    /// `EFFECT_STARTED`, `EFFECT_CONFIRMED` or `PossiblyStarted`.
    pub fn is_after_start(self) -> bool {
        self != TrackedPhase::PreWrite
    }
}

/// The mutable part of one claim's state.
#[derive(Debug, Clone)]
pub(crate) struct ClaimStatus {
    pub phase: TrackedPhase,
    /// An engine call answered `ASYNC_JOB_STALE_CLAIM`. Final.
    pub claim_lost: bool,
    /// An engine call answered `ASYNC_JOB_LEASE_EXPIRED`.
    pub lease_expired: bool,
    /// When `claim_lost` or `lease_expired` was first set.
    pub lost_at: Option<Instant>,
    /// The final call is under way.
    pub finishing: bool,
    /// The handler returned (or never ran).
    pub handler_returned: bool,
    /// The claim ended: a final call, the unrun rule or a fallback committed,
    /// or the token was no longer the job's.
    pub ended: bool,
    /// The last successful renewal (or the claim).
    pub last_renewal: Instant,
    /// The lease expiry the last claim or renewal returned.
    pub lease_expires_at: DateTime<Utc>,
    /// The local deadline estimate: the instant before the claim call plus
    /// the kind's `max_attempt_runtime_seconds`, never later than the truth.
    pub deadline_estimate: Instant,
    /// `mark_effect_started` answered `ShutdownInProgress` after a T4 call
    /// (carry OPW4-1): a `Release` refused with `PHASE` is then a shutdown
    /// fallback, not a handler fault.
    pub shutdown_after_effect_mark: bool,
    /// The slot running the handler.
    pub slot: Option<usize>,
}

/// One claim, tracked from the claim call to its end.
pub(crate) struct ClaimState {
    pub claim: WorkerClaim,
    pub job_id: Uuid,
    pub policy: &'static AsyncJobKindPolicy,
    pub purpose: Purpose,
    pub payload_version: i16,
    pub attempt_number: i32,
    /// The renewal lock, holding the claim's lease request: `None` from the
    /// claim, `Some(E)` once set, never set back. Every renewal of the claim
    /// runs under it.
    pub renewal: Mutex<Option<i32>>,
    pub status: Mutex<ClaimStatus>,
}

impl ClaimState {
    pub(crate) fn status(&self) -> MutexGuard<'_, ClaimStatus> {
        lock(&self.status)
    }

    pub(crate) fn renewal_lock(&self) -> MutexGuard<'_, Option<i32>> {
        lock(&self.renewal)
    }

    pub(crate) fn snapshot(&self) -> ClaimStatus {
        self.status().clone()
    }

    pub(crate) fn kind(&self) -> &'static str {
        self.policy.kind
    }
}

/// Lock a mutex, taking it over when a panicking thread poisoned it.
pub(crate) fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

// ---------------------------------------------------------------------------
// Events: the stop signal, the phase deadlines and the idle slots, under one
// mutex and one condition variable, so every wait ends at once on the
// change it waits for.
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
pub(crate) struct EventState {
    pub stop: bool,
    pub stop_at: Option<Instant>,
    pub second_signal: bool,
    pub fallback_end: Option<Instant>,
    pub loop_exited: bool,
    pub exited: bool,
    /// Test only: every thread exits at once with no final write.
    pub hard_stop: bool,
    /// The idle slot threads.
    pub idle_slots: Vec<usize>,
}

#[derive(Default)]
pub(crate) struct Events {
    state: Mutex<EventState>,
    cv: Condvar,
}

impl Events {
    pub(crate) fn read<R>(&self, f: impl FnOnce(&EventState) -> R) -> R {
        f(&lock(&self.state))
    }

    pub(crate) fn update<R>(&self, f: impl FnOnce(&mut EventState) -> R) -> R {
        let result = f(&mut lock(&self.state));
        self.cv.notify_all();
        result
    }

    /// Wait until `done` holds or `deadline` passes; true when `done` holds.
    pub(crate) fn wait_until(
        &self,
        deadline: Option<Instant>,
        done: impl Fn(&EventState) -> bool,
    ) -> bool {
        let mut guard = lock(&self.state);
        loop {
            if done(&guard) {
                return true;
            }
            match deadline {
                None => {
                    guard = self
                        .cv
                        .wait(guard)
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                }
                Some(deadline) => {
                    let now = Instant::now();
                    if now >= deadline {
                        return false;
                    }
                    guard = self
                        .cv
                        .wait_timeout(guard, deadline - now)
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .0;
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Counters (spec section 7.3)
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
pub(crate) struct Counters {
    pub claims: AtomicU64,
    pub successes: AtomicU64,
    pub failures: AtomicU64,
    pub waits: AtomicU64,
    pub releases: AtomicU64,
    pub unrun: AtomicU64,
    pub resolutions: AtomicU64,
    pub lost_claims: AtomicU64,
    pub expired_leases: AtomicU64,
    pub busy: AtomicU64,
    pub action_fallbacks: AtomicU64,
    pub fault_fallbacks: AtomicU64,
    pub shutdown_fallbacks: AtomicU64,
    pub stalled_ticks: AtomicU64,
}

pub(crate) fn bump(counter: &AtomicU64) {
    counter.fetch_add(1, Ordering::Relaxed);
}

fn read(counter: &AtomicU64) -> u64 {
    counter.load(Ordering::Relaxed)
}

// ---------------------------------------------------------------------------
// Engine operations, named for logs and for the test fault hook
// ---------------------------------------------------------------------------

/// Every engine call the runtime makes, by name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Op {
    Claim,
    ClaimReconciliation,
    Recover,
    /// A heartbeat renewal; only the test fault hook names it.
    #[cfg_attr(not(test), allow(dead_code))]
    Renew,
    EffectStart,
    ExtraRenew,
    EffectConfirm,
    ConfirmClaim,
    ReadTargetState,
    Attempts,
    Complete,
    Fail,
    Wait,
    Release,
    Resolve,
    Unrun,
    Fallback,
}

// ---------------------------------------------------------------------------
// Pools (spec section 6.1)
// ---------------------------------------------------------------------------

pub(crate) struct Pools {
    /// Claim loop, context calls, final actions and fallbacks.
    pub engine: PgPool,
    /// The heartbeat only, so renewals never wait behind other callers.
    pub heartbeat: PgPool,
    /// What a handler's `pool()` returns; handlers can exhaust only this.
    pub handler: PgPool,
}

/// The session settings of engine and heartbeat connections.
#[derive(Debug, Clone, Copy)]
struct EngineSession;

impl CustomizeConnection<PgConnection, diesel::r2d2::Error> for EngineSession {
    fn on_acquire(&self, conn: &mut PgConnection) -> Result<(), diesel::r2d2::Error> {
        conn.batch_execute(SESSION_SETTINGS)
            .map_err(diesel::r2d2::Error::QueryError)
    }
}

/// r2d2's default error handler logs the driver's text; this one logs one
/// fixed line.
#[derive(Debug, Clone, Copy)]
struct FixedErrorHandler;

impl HandleError<diesel::r2d2::Error> for FixedErrorHandler {
    fn handle_error(&self, _error: diesel::r2d2::Error) {
        log::warn!("async worker: database pool connection attempt failed (DATABASE_UNAVAILABLE)");
    }
}

fn build_pools(database_url: &str, slots: usize, checkout: Duration) -> Pools {
    let slots = u32::try_from(slots).unwrap_or(16);
    let engine = Pool::builder()
        .max_size(2 * slots + 2)
        .min_idle(Some(1))
        .connection_timeout(checkout)
        .error_handler(Box::new(FixedErrorHandler))
        .connection_customizer(Box::new(EngineSession))
        .build_unchecked(ConnectionManager::<PgConnection>::new(database_url));
    let heartbeat = Pool::builder()
        .max_size(1)
        .min_idle(Some(1))
        .connection_timeout(checkout)
        .error_handler(Box::new(FixedErrorHandler))
        .connection_customizer(Box::new(EngineSession))
        .build_unchecked(ConnectionManager::<PgConnection>::new(database_url));
    let handler = Pool::builder()
        .max_size(slots + 1)
        .min_idle(Some(0))
        .connection_timeout(checkout)
        .error_handler(Box::new(FixedErrorHandler))
        .build_unchecked(ConnectionManager::<PgConnection>::new(database_url));
    Pools {
        engine,
        heartbeat,
        handler,
    }
}

// ---------------------------------------------------------------------------
// Pending migrations, read only (spec section 6.2 step 8)
// ---------------------------------------------------------------------------

#[derive(QueryableByName)]
struct ExistsRow {
    #[diesel(sql_type = Bool)]
    exists: bool,
}

#[derive(QueryableByName)]
struct VersionRow {
    #[diesel(sql_type = Text)]
    version: String,
}

/// Run `f` in a `READ ONLY` transaction: any write or DDL fails.
pub(crate) fn in_read_only<T>(
    conn: &mut PgConnection,
    f: impl FnOnce(&mut PgConnection) -> QueryResult<T>,
) -> QueryResult<T> {
    conn.build_transaction().read_only().run(f)
}

/// The applied migration versions, or `None` when the migrations table does
/// not exist. Read only: the worker never runs migrations and never calls
/// diesel's migration harness, which creates its table.
pub(crate) fn applied_migrations(conn: &mut PgConnection) -> QueryResult<Option<Vec<String>>> {
    in_read_only(conn, |conn| {
        let exists = sql_query(
            "SELECT to_regclass('public.__diesel_schema_migrations') IS NOT NULL AS exists",
        )
        .get_result::<ExistsRow>(conn)?
        .exists;
        if !exists {
            return Ok(None);
        }
        let versions = sql_query("SELECT version FROM public.__diesel_schema_migrations")
            .load::<VersionRow>(conn)?
            .into_iter()
            .map(|row| row.version)
            .collect();
        Ok(Some(versions))
    })
}

/// The versions of the migrations embedded in this binary.
pub(crate) fn embedded_migrations() -> Option<Vec<String>> {
    MigrationSource::<Pg>::migrations(&MIGRATIONS)
        .ok()
        .map(|migrations| {
            migrations
                .iter()
                .map(|migration| migration.name().version().to_string())
                .collect()
        })
}

/// Pending: the table is missing, or an embedded version is not applied. A
/// database ahead of the binary (newer versions applied) is not pending.
pub(crate) fn migrations_pending(embedded: &[String], applied: Option<&[String]>) -> bool {
    match applied {
        None => true,
        Some(applied) => embedded
            .iter()
            .any(|version| !applied.iter().any(|done| done == version)),
    }
}

// ---------------------------------------------------------------------------
// Shared runtime state
// ---------------------------------------------------------------------------

pub(crate) struct Shared {
    pub config: WorkerConfig,
    pub registry: &'static AsyncJobRegistry,
    pub handlers: &'static HandlerRegistry,
    pub executor: Executor,
    pub timings: Timings,
    base: Instant,
    pub events: Events,
    loop_progress: AtomicU64,
    heartbeat_progress: AtomicU64,
    renewal_health: AtomicU64,
    last_claim_success: AtomicU64,
    pub in_flight: Mutex<HashMap<Uuid, Arc<ClaimState>>>,
    pub counters: Counters,
    panicked: AtomicBool,
    fatal: AtomicBool,
    unended: AtomicBool,
    sets: OnceLock<ClaimSets>,
    pools: OnceLock<Pools>,
    slot_senders: Mutex<Vec<mpsc::Sender<Assignment>>>,
    threads: Mutex<Vec<JoinHandle<()>>>,
    #[cfg(test)]
    pub hooks: Arc<TestHooks>,
}

/// The pool profile (spec section 5.2): fixed in code except the name and
/// the capabilities.
pub(crate) fn worker_executor(config: &WorkerConfig) -> Executor {
    Executor {
        name: config.worker_name.clone(),
        principal: WORKER_PRINCIPAL.to_string(),
        domain: ExecutorDomain::ThothWorker,
        risk_class: RiskClass::Trusted,
        capabilities: config.capabilities.clone(),
        revision_modes: vec![RevisionMode::CurrentState, RevisionMode::RevisionBound],
    }
}

impl Shared {
    fn new(
        config: WorkerConfig,
        registry: &'static AsyncJobRegistry,
        handlers: &'static HandlerRegistry,
        timings: Timings,
        #[cfg(test)] hooks: Arc<TestHooks>,
    ) -> Self {
        let executor = worker_executor(&config);
        Shared {
            config,
            registry,
            handlers,
            executor,
            timings,
            base: Instant::now(),
            events: Events::default(),
            loop_progress: AtomicU64::new(0),
            heartbeat_progress: AtomicU64::new(0),
            renewal_health: AtomicU64::new(0),
            last_claim_success: AtomicU64::new(0),
            in_flight: Mutex::new(HashMap::new()),
            counters: Counters::default(),
            panicked: AtomicBool::new(false),
            fatal: AtomicBool::new(false),
            unended: AtomicBool::new(false),
            sets: OnceLock::new(),
            pools: OnceLock::new(),
            slot_senders: Mutex::new(Vec::new()),
            threads: Mutex::new(Vec::new()),
            #[cfg(test)]
            hooks,
        }
    }

    // --- progress instants (monotonic, 0 = never) ----------------------------

    fn stamp(&self) -> u64 {
        u64::try_from(self.base.elapsed().as_nanos())
            .unwrap_or(u64::MAX - 1)
            .saturating_add(1)
    }

    fn age_of(&self, stamp: u64) -> Option<Duration> {
        if stamp == 0 {
            return None;
        }
        let now = self.stamp();
        Some(Duration::from_nanos(now.saturating_sub(stamp)))
    }

    pub(crate) fn record_loop(&self) {
        self.loop_progress.store(self.stamp(), Ordering::Relaxed);
    }

    pub(crate) fn record_heartbeat(&self) {
        self.heartbeat_progress
            .store(self.stamp(), Ordering::Relaxed);
    }

    pub(crate) fn record_renewal_health(&self) {
        self.renewal_health.store(self.stamp(), Ordering::Relaxed);
    }

    pub(crate) fn record_claim_success(&self) {
        self.last_claim_success
            .store(self.stamp(), Ordering::Relaxed);
    }

    pub(crate) fn heartbeat_age(&self) -> Option<Duration> {
        self.age_of(self.heartbeat_progress.load(Ordering::Relaxed))
    }

    pub(crate) fn renewal_health_age(&self) -> Option<Duration> {
        self.age_of(self.renewal_health.load(Ordering::Relaxed))
    }

    fn loop_age(&self) -> Duration {
        self.age_of(self.loop_progress.load(Ordering::Relaxed))
            .unwrap_or(Duration::ZERO)
    }

    // --- stop, exit and fatal flags -----------------------------------------

    pub(crate) fn stopping(&self) -> bool {
        self.events.read(|state| state.stop)
    }

    pub(crate) fn hard_stopped(&self) -> bool {
        self.events.read(|state| state.hard_stop)
    }

    pub(crate) fn exited(&self) -> bool {
        self.events.read(|state| state.exited || state.hard_stop)
    }

    /// A stop signal (`SIGTERM` or `SIGINT`): the first stops claims and
    /// starts the grace phase, a second ends the grace phase at once.
    pub(crate) fn signal(&self) {
        let first = self.events.update(|state| {
            if state.stop {
                state.second_signal = true;
                false
            } else {
                state.stop = true;
                state.stop_at = Some(Instant::now());
                true
            }
        });
        if first {
            log::info!("async worker: stop signal received; no new claims");
        } else {
            log::info!("async worker: second stop signal received; the grace phase ends now");
        }
    }

    /// Stop from inside the runtime (a thread panic or a fatal engine
    /// answer). Never counts as a second signal.
    pub(crate) fn stop_internal(&self) {
        self.events.update(|state| {
            if !state.stop {
                state.stop = true;
                state.stop_at = Some(Instant::now());
            }
        });
    }

    /// `ASYNC_JOB_KIND_NOT_AUTHORIZED`: a bug. Stop, and exit 1.
    pub(crate) fn fatal(&self, what: &str) {
        log::error!("async worker: {what} answered ASYNC_JOB_KIND_NOT_AUTHORIZED; stopping");
        self.fatal.store(true, Ordering::SeqCst);
        self.stop_internal();
    }

    pub(crate) fn mark_panicked(&self, thread: &str) {
        log::error!("async worker: runtime thread {thread} panicked; stopping");
        self.panicked.store(true, Ordering::SeqCst);
        self.stop_internal();
    }

    pub(crate) fn has_panicked(&self) -> bool {
        self.panicked.load(Ordering::SeqCst)
    }

    /// A claim did not end during shutdown: exit 1.
    pub(crate) fn note_unended(&self) {
        if self.stopping() {
            self.unended.store(true, Ordering::SeqCst);
        }
    }

    /// Sleep up to `delay`; true when the stop signal ended the sleep (or
    /// was already set).
    pub(crate) fn sleep_unless_stopped(&self, delay: Duration) -> bool {
        self.events
            .wait_until(Some(Instant::now() + delay), |state| {
                state.stop || state.hard_stop
            })
    }

    /// Sleep up to `delay` during a final call's `BUSY` retries; false when
    /// the fallback phase has ended (stop retrying).
    pub(crate) fn sleep_retry(&self, delay: Duration) -> bool {
        let fallback_end = self.events.read(|state| state.fallback_end);
        let wake = Instant::now() + delay;
        let deadline = fallback_end.map_or(wake, |end| end.min(wake));
        self.events
            .wait_until(Some(deadline), |state| state.exited || state.hard_stop);
        !self.retry_window_closed()
    }

    /// The fallback phase has ended (or the runtime exited).
    pub(crate) fn retry_window_closed(&self) -> bool {
        self.events.read(|state| {
            state.exited
                || state.hard_stop
                || state.fallback_end.is_some_and(|end| Instant::now() >= end)
        })
    }

    // --- pools, sets and engine calls ---------------------------------------

    pub(crate) fn pools(&self) -> &Pools {
        self.pools
            .get()
            .expect("the pools are built before any engine call")
    }

    pub(crate) fn sets(&self) -> &ClaimSets {
        self.sets
            .get()
            .expect("the claim sets are computed at startup")
    }

    /// One engine call in its own `with_engine_transaction` on the engine
    /// pool.
    pub(crate) fn engine_call<T>(
        &self,
        op: Op,
        f: impl FnOnce(&mut PgConnection) -> ThothResult<T>,
    ) -> ThothResult<T> {
        let pool = &self.pools().engine;
        #[cfg(test)]
        if let Some(fault) = self.hooks.take_fault(op) {
            return fault.apply(|| with_engine_transaction(pool, f));
        }
        #[cfg(not(test))]
        let _ = op;
        with_engine_transaction(pool, f)
    }

    /// An engine call retried after each `BUSY` with the configured delays,
    /// stopping early at the end of the shutdown fallback phase. Returns the
    /// last answer and whether a `BUSY` preceded it.
    pub(crate) fn engine_call_with_retries<T>(
        &self,
        op: Op,
        mut f: impl FnMut(&mut PgConnection) -> ThothResult<T>,
    ) -> (ThothResult<T>, bool) {
        let mut after_busy = false;
        let mut delays = self.timings.busy_retries.iter();
        loop {
            let result = self.engine_call(op, &mut f);
            match result {
                Err(ThothError::AsyncJobBusy) => {
                    bump(&self.counters.busy);
                    let Some(delay) = delays.next() else {
                        return (Err(ThothError::AsyncJobBusy), after_busy);
                    };
                    after_busy = true;
                    if !self.sleep_retry(*delay) {
                        return (Err(ThothError::AsyncJobBusy), after_busy);
                    }
                }
                other => return (other, after_busy),
            }
        }
    }

    // --- in-flight claims ----------------------------------------------------

    pub(crate) fn in_flight(&self) -> MutexGuard<'_, HashMap<Uuid, Arc<ClaimState>>> {
        lock(&self.in_flight)
    }

    pub(crate) fn claims_snapshot(&self) -> Vec<Arc<ClaimState>> {
        self.in_flight().values().cloned().collect()
    }

    pub(crate) fn remove_claim(&self, state: &ClaimState) {
        self.in_flight().remove(&state.claim.claim_token);
        self.events.update(|_| ());
    }

    /// Set `claim_lost` (once) and count it.
    pub(crate) fn mark_lost(&self, state: &ClaimState) {
        let mut status = state.status();
        if !status.claim_lost {
            status.claim_lost = true;
            status.lost_at.get_or_insert_with(Instant::now);
            bump(&self.counters.lost_claims);
        }
    }

    /// Set `lease_expired` (once) and count it.
    pub(crate) fn mark_expired(&self, state: &ClaimState) {
        let mut status = state.status();
        if !status.lease_expired {
            status.lease_expired = true;
            status.lost_at.get_or_insert_with(Instant::now);
            bump(&self.counters.expired_leases);
        }
    }

    /// Store a successful renewal: its instant and the returned expiry.
    pub(crate) fn store_renewal(&self, state: &ClaimState, at: Instant, expiry: DateTime<Utc>) {
        let mut status = state.status();
        if at > status.last_renewal {
            status.last_renewal = at;
        }
        // Renewals commit in renewal-lock order but are stored after the
        // lock drops, so an older answer can arrive last. The request only
        // grows and the deadline is fixed, so the latest expiry is the
        // largest.
        if expiry > status.lease_expires_at {
            status.lease_expires_at = expiry;
        }
    }

    pub(crate) fn idle_slot_count(&self) -> usize {
        self.events.read(|state| state.idle_slots.len())
    }

    /// The hung slots and how long each has been hung (spec section 6.7): a
    /// handler that has not returned more than the grace period after its
    /// claim was lost or its lease expired.
    fn hung_durations(&self) -> Vec<Duration> {
        let now = Instant::now();
        self.claims_snapshot()
            .iter()
            .filter_map(|state| {
                let status = state.status();
                if status.handler_returned || status.slot.is_none() {
                    return None;
                }
                let hung_from = status.lost_at? + self.timings.grace;
                (now > hung_from).then(|| now - hung_from)
            })
            .collect()
    }

    // --- threads --------------------------------------------------------------

    /// Spawn a runtime thread whose body runs inside `catch_unwind`: a panic
    /// logs one fixed line, requests shutdown and makes the exit code 1.
    pub(crate) fn spawn(
        self: &Arc<Self>,
        name: &'static str,
        body: impl FnOnce(&Arc<Self>) + Send + 'static,
    ) {
        let rt = Arc::clone(self);
        let handle = thread::Builder::new()
            .name(format!("async-worker-{name}"))
            .spawn(move || {
                if catch_unwind(AssertUnwindSafe(|| body(&rt))).is_err() {
                    rt.mark_panicked(name);
                }
            })
            .expect("the worker can spawn a thread");
        lock(&self.threads).push(handle);
    }

    // --- the exit code ----------------------------------------------------

    fn exit_code(&self) -> i32 {
        let unended_claims = self
            .claims_snapshot()
            .iter()
            .any(|state| !state.status().ended);
        if self.panicked.load(Ordering::SeqCst)
            || self.fatal.load(Ordering::SeqCst)
            || self.unended.load(Ordering::SeqCst)
            || unended_claims
        {
            1
        } else {
            0
        }
    }

    fn finish(&self, code: i32) -> i32 {
        self.events.update(|state| state.exited = true);
        // A thread panic or a fatal answer after the code was decided still
        // makes it 1.
        if self.panicked.load(Ordering::SeqCst) || self.fatal.load(Ordering::SeqCst) {
            1
        } else {
            code
        }
    }

    // --- operational reporting ---------------------------------------------

    fn summary_line(&self) -> String {
        let c = &self.counters;
        let in_flight = self.in_flight().len();
        let last_claim = match self.age_of(self.last_claim_success.load(Ordering::Relaxed)) {
            Some(age) => format!("{} s ago", age.as_secs()),
            None => "never".to_string(),
        };
        format!(
            "async worker: summary: in-flight {in_flight}, idle slots {}, hung slots {}; since \
             start: claims {}, successes {}, failures {}, waits {}, releases {}, unrun {}, \
             reconciliation resolutions {}, lost claims {}, expired leases {}, busy answers {}, \
             action fallbacks {}, fault fallbacks {}, shutdown fallbacks {}, claim ticks stopped by \
             heartbeat or renewal health {}; last successful claim call {last_claim}",
            self.idle_slot_count(),
            self.hung_durations().len(),
            read(&c.claims),
            read(&c.successes),
            read(&c.failures),
            read(&c.waits),
            read(&c.releases),
            read(&c.unrun),
            read(&c.resolutions),
            read(&c.lost_claims),
            read(&c.expired_leases),
            read(&c.busy),
            read(&c.action_fallbacks),
            read(&c.fault_fallbacks),
            read(&c.shutdown_fallbacks),
            read(&c.stalled_ticks),
        )
    }

    fn liveness_inputs(&self) -> LivenessInputs {
        // Each read takes the in-flight lock on its own: a guard held as a
        // temporary of the struct literal would deadlock the second read.
        let in_flight = self.in_flight().len();
        let hung = self.hung_durations();
        LivenessInputs {
            loop_age: self.loop_age(),
            heartbeat_age: self.heartbeat_age(),
            in_flight,
            hung,
            slots: self.config.slots,
            panicked: self.has_panicked(),
        }
    }

    fn liveness_limits(&self) -> LivenessLimits {
        LivenessLimits {
            loop_stall: self.timings.loop_stall,
            heartbeat_stall: self.timings.heartbeat_stall,
            hung_slot_limit: self.timings.hung_slot_limit,
        }
    }
}

// ---------------------------------------------------------------------------
// The liveness ticker (spec section 7.2) and the summary line (7.3)
// ---------------------------------------------------------------------------

fn ticker_loop(rt: &Arc<Shared>) {
    let mut last_write_failure_log: Option<Instant> = None;
    let mut last_withhold: Option<&'static str> = None;
    let mut next_summary = Instant::now() + rt.timings.summary_interval;
    loop {
        if rt.exited() {
            return;
        }
        let reason = withhold_reason(&rt.liveness_inputs(), &rt.liveness_limits());
        #[cfg(test)]
        rt.hooks.record_liveness(reason);
        match reason {
            Some(reason) => {
                if last_withhold != Some(reason) {
                    log::warn!("async worker: liveness withheld ({reason})");
                }
                last_withhold = Some(reason);
            }
            None => {
                last_withhold = None;
                if write_liveness(&rt.config.liveness_file, unix_now()).is_err()
                    && last_write_failure_log
                        .is_none_or(|at| at.elapsed() >= WRITE_FAILURE_LOG_INTERVAL)
                {
                    log::warn!("async worker: the liveness file could not be written (LIVENESS_WRITE_FAILED)");
                    last_write_failure_log = Some(Instant::now());
                }
            }
        }
        if Instant::now() >= next_summary {
            log::info!("{}", rt.summary_line());
            next_summary = Instant::now() + rt.timings.summary_interval;
        }
        rt.events.wait_until(
            Some(Instant::now() + rt.timings.liveness_cadence),
            |state| state.exited || state.hard_stop,
        );
    }
}

// ---------------------------------------------------------------------------
// The handle, the process and the runtime
// ---------------------------------------------------------------------------

/// What the binary's signal thread holds.
#[derive(Clone)]
pub struct WorkerHandle {
    shared: Arc<Shared>,
}

impl WorkerHandle {
    /// A `SIGTERM` or `SIGINT`: the first stops claims and starts the grace
    /// phase; a second ends the grace phase.
    pub fn request_shutdown(&self) {
        self.shared.signal();
    }

    /// The binary could not set up its signal handling: stop, and exit 1.
    pub fn signal_setup_failed(&self) {
        log::error!(
            "async worker: stop signals cannot be received (SIGNAL_SETUP_FAILED); stopping"
        );
        self.shared.fatal.store(true, Ordering::SeqCst);
        self.shared.stop_internal();
    }

    #[cfg(test)]
    pub(crate) fn shared(&self) -> &Arc<Shared> {
        &self.shared
    }
}

/// A validated worker, ready to run.
pub struct WorkerProcess {
    runtime: Runtime,
}

impl WorkerProcess {
    pub fn handle(&self) -> WorkerHandle {
        self.runtime.handle()
    }

    /// Run until the stop signal and the shutdown end; returns the exit
    /// code. Only the binary's entry point calls `std::process::exit` with
    /// it.
    pub fn run(self) -> i32 {
        self.runtime.run()
    }
}

/// Startup steps 1 to 3 (spec section 6.2): validate the configuration,
/// refuse static or file AWS credential sources, initialise the logger and
/// install the worker panic hook. A problem is printed to standard error
/// (the logger does not exist yet) and answers exit code 1.
pub fn launch(args: WorkerArgs) -> Result<WorkerProcess, i32> {
    let config = match WorkerConfig::from_args(&args) {
        Ok(config) => config,
        Err(problems) => {
            for problem in problems {
                eprintln!("async worker: refusing to start: {problem}");
            }
            return Err(1);
        }
    };
    if let Err(problems) = check_aws_sources(|name| std::env::var_os(name)) {
        for problem in problems {
            eprintln!("async worker: refusing to start: {problem}");
        }
        return Err(1);
    }
    init_logging();
    let timings = Timings::production(&config);
    Ok(WorkerProcess {
        runtime: Runtime::new(config, &PRODUCTION_REGISTRY, &PRODUCTION_HANDLERS, timings),
    })
}

/// The process-global logger (`env_logger`, default `info`) and the worker
/// panic hook. Only the binary's entry point installs them.
fn init_logging() {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .try_init();
    std::panic::set_hook(Box::new(|info| {
        log::error!("{}", panic_line(info.location()));
    }));
}

/// The panic hook's line: the location only, never the panic payload.
pub fn panic_line(location: Option<&std::panic::Location<'_>>) -> String {
    match location {
        Some(location) => format!(
            "async worker: panic at {}:{}",
            location.file(),
            location.line()
        ),
        None => "async worker: panic at an unknown location".to_string(),
    }
}

pub(crate) struct Runtime {
    shared: Arc<Shared>,
}

impl Runtime {
    pub(crate) fn new(
        config: WorkerConfig,
        registry: &'static AsyncJobRegistry,
        handlers: &'static HandlerRegistry,
        timings: Timings,
    ) -> Self {
        Runtime {
            shared: Arc::new(Shared::new(
                config,
                registry,
                handlers,
                timings,
                #[cfg(test)]
                Arc::new(TestHooks::default()),
            )),
        }
    }

    /// The test-only constructor: every timing settable, and the hooks.
    #[cfg(test)]
    pub(crate) fn for_test(
        config: WorkerConfig,
        registry: &'static AsyncJobRegistry,
        handlers: &'static HandlerRegistry,
        timings: Timings,
        hooks: Arc<TestHooks>,
    ) -> Self {
        Runtime {
            shared: Arc::new(Shared::new(config, registry, handlers, timings, hooks)),
        }
    }

    pub(crate) fn handle(&self) -> WorkerHandle {
        WorkerHandle {
            shared: Arc::clone(&self.shared),
        }
    }

    /// Steps 3 (the ticker) to 9, then the shutdown. Returns the exit code.
    pub(crate) fn run(self) -> i32 {
        let rt = self.shared;
        rt.record_loop();
        rt.spawn("liveness", ticker_loop);
        match catch_unwind(AssertUnwindSafe(|| run_inner(&rt))) {
            Ok(code) => rt.finish(code),
            Err(_) => {
                rt.mark_panicked("main");
                rt.finish(1)
            }
        }
    }
}

fn run_inner(rt: &Arc<Shared>) -> i32 {
    // Step 4: registry and handler consistency.
    let mut problems = Vec::new();
    if let Err(found) = rt.registry.validate() {
        problems.extend(found);
    }
    if let Err(found) = rt.handlers.validate(rt.registry) {
        problems.extend(found);
    }
    if !problems.is_empty() {
        for problem in problems {
            log::error!("async worker: refusing to start: {problem}");
        }
        return 1;
    }

    // Step 5: the claim sets and the effective configuration.
    let (sets, notes) =
        compute_claim_sets(&rt.config.kinds, rt.registry, rt.handlers, &rt.executor);
    for note in &notes {
        log::error!("async worker: {note}");
    }
    log::info!(
        "async worker: starting: name {}, slots {}, poll {} s, grace {} s, liveness file {}, \
         capabilities [{}], execution set [{}], reconciliation set [{}]",
        rt.config.worker_name,
        rt.config.slots,
        rt.config.poll_seconds,
        rt.config.shutdown_grace_seconds,
        rt.config.liveness_file.display(),
        rt.config.capabilities.join(", "),
        sets.describe(Purpose::Execution),
        sets.describe(Purpose::Reconciliation),
    );
    let idle = sets.is_empty();
    let _ = rt.sets.set(sets);

    // Step 6: idle. No pool, no connection.
    if idle {
        log::info!("async worker: no kind to claim; idle (no database connection)");
        rt.spawn("claim-loop", claims::idle_loop);
        rt.events
            .wait_until(None, |state| state.stop || state.hard_stop);
        rt.events.wait_until(
            Some(Instant::now() + rt.timings.claim_loop_join_bound),
            |state| state.loop_exited || state.hard_stop,
        );
        return rt.exit_code();
    }

    // Step 7: the pools, retried until a connection succeeds.
    let _ = rt.pools.set(build_pools(
        rt.config.database_url(),
        rt.config.slots,
        rt.timings.pool_checkout,
    ));
    #[cfg(test)]
    rt.hooks.note_pools_built();
    if !wait_for_database(rt) {
        return rt.exit_code();
    }

    // Step 8: pending migrations, read only.
    if !wait_for_migrations(rt) {
        return rt.exit_code();
    }

    // Step 9: the heartbeat, the slots and the claim loop. The heartbeat's
    // progress and renewal health start now, so the first claim tick does
    // not read a heartbeat that has not been scheduled yet as stalled.
    rt.record_heartbeat();
    rt.record_renewal_health();
    rt.spawn("heartbeat", heartbeat::heartbeat_loop);
    slots::start_slots(rt);
    rt.spawn("claim-loop", claims::claim_loop);
    log::info!("async worker: running");

    rt.events
        .wait_until(None, |state| state.stop || state.hard_stop);
    if rt.hard_stopped() {
        return 1;
    }
    shutdown(rt)
}

/// Step 7: until a connection succeeds, back off (capped), logging each
/// failure by a fixed code and honouring the stop signal. False on stop.
fn wait_for_database(rt: &Arc<Shared>) -> bool {
    let mut delay = rt.timings.connect_backoff_start;
    loop {
        rt.record_loop();
        if rt.stopping() || rt.hard_stopped() {
            return false;
        }
        let connected = rt.pools().engine.get().is_ok();
        rt.record_loop();
        if connected {
            return true;
        }
        log::warn!(
            "async worker: database unavailable (DATABASE_UNAVAILABLE); retrying in {} ms",
            delay.as_millis()
        );
        if rt.sleep_unless_stopped(delay) {
            return false;
        }
        delay = (delay * 2).min(rt.timings.connect_backoff_cap);
    }
}

/// Step 8: claim nothing while a migration is pending; re-check
/// periodically. False on stop.
fn wait_for_migrations(rt: &Arc<Shared>) -> bool {
    let mut logged = false;
    loop {
        rt.record_loop();
        if rt.stopping() || rt.hard_stopped() {
            return false;
        }
        match pending_migrations(rt) {
            Ok(false) => return true,
            Ok(true) => {
                if !logged {
                    log::warn!(
                        "async worker: database migrations are pending (MIGRATIONS_PENDING); \
                         claiming nothing until they are applied"
                    );
                    logged = true;
                }
            }
            Err(code) => log::warn!("async worker: the migration check failed ({code})"),
        }
        rt.record_loop();
        if rt.sleep_unless_stopped(rt.timings.migration_recheck) {
            return false;
        }
    }
}

fn pending_migrations(rt: &Shared) -> Result<bool, &'static str> {
    #[cfg(test)]
    if rt.hooks.force_pending_migrations() {
        return Ok(true);
    }
    let embedded = embedded_migrations().ok_or("MIGRATION_SOURCE_INVALID")?;
    let mut conn = rt
        .pools()
        .engine
        .get()
        .map_err(|_| "DATABASE_UNAVAILABLE")?;
    let applied = applied_migrations(&mut conn).map_err(|_| "MIGRATION_CHECK_FAILED")?;
    Ok(migrations_pending(&embedded, applied.as_deref()))
}

/// Graceful shutdown (spec section 6.8 steps 2 to 7). Returns the exit code.
fn shutdown(rt: &Arc<Shared>) -> i32 {
    let stop_at = rt
        .events
        .read(|state| state.stop_at)
        .unwrap_or_else(Instant::now);
    let join_deadline = stop_at + rt.timings.claim_loop_join_bound;
    let grace_end = stop_at + rt.timings.grace;

    // Step 2: the claim loop makes no new claim call; wait for one in
    // flight, bounded.
    let loop_done = rt.events.wait_until(Some(join_deadline), |state| {
        state.loop_exited || state.hard_stop
    });
    if !loop_done {
        log::warn!(
            "async worker: a claim call was still in flight at the join bound; its jobs are left \
             to lease expiry"
        );
        rt.unended.store(true, Ordering::SeqCst);
    }
    if rt.hard_stopped() {
        return 1;
    }

    // Step 5: the grace phase ends at its deadline, on a second signal, or
    // as soon as no claim is left.
    loop {
        if rt.in_flight().is_empty() {
            break;
        }
        let ended = rt.events.wait_until(
            Some((Instant::now() + Duration::from_millis(50)).min(grace_end)),
            |state| state.second_signal || state.hard_stop,
        );
        if ended || Instant::now() >= grace_end {
            break;
        }
    }
    if rt.hard_stopped() {
        return 1;
    }

    // Step 6: the fallback phase, bounded as a whole.
    let fallback_end = Instant::now() + rt.timings.fallback_phase_bound;
    rt.events
        .update(|state| state.fallback_end = Some(fallback_end));
    let pending: Vec<Arc<ClaimState>> = rt
        .claims_snapshot()
        .into_iter()
        .filter(|state| {
            let status = state.status();
            !status.handler_returned && !status.ended
        })
        .collect();
    let mut fallbacks = Vec::new();
    for state in pending {
        let rt = Arc::clone(rt);
        fallbacks.push(
            thread::Builder::new()
                .name("async-worker-fallback".to_string())
                .spawn(move || {
                    if catch_unwind(AssertUnwindSafe(|| slots::shutdown_fallback(&rt, &state)))
                        .is_err()
                    {
                        rt.mark_panicked("fallback");
                    }
                })
                .expect("the worker can spawn a thread"),
        );
    }
    // Wait for every fallback and every final call in its retries, bounded.
    loop {
        let done = fallbacks.iter().all(JoinHandle::is_finished)
            && rt.claims_snapshot().iter().all(|state| {
                let status = state.status();
                status.ended || !status.handler_returned
            });
        if done || Instant::now() >= fallback_end {
            break;
        }
        rt.events.wait_until(
            Some((Instant::now() + Duration::from_millis(20)).min(fallback_end)),
            |state| state.hard_stop,
        );
    }
    #[cfg(test)]
    lock(&rt.threads).extend(fallbacks);

    // Step 7: exit at once, without joining handler threads.
    let code = rt.exit_code();
    let left = rt
        .claims_snapshot()
        .iter()
        .filter(|state| !state.status().ended)
        .count();
    if left > 0 {
        log::warn!("async worker: {left} claim(s) did not end; they are recovered at lease expiry");
    }
    log::info!("async worker: stopped with exit code {code}");
    code
}

// ---------------------------------------------------------------------------
// Test access
// ---------------------------------------------------------------------------

#[cfg(test)]
impl Shared {
    /// Join every runtime thread (tests only), bounded.
    pub(crate) fn join_threads(&self, bound: Duration) {
        self.release_threads();
        let deadline = Instant::now() + bound;
        loop {
            let handles: Vec<JoinHandle<()>> = lock(&self.threads).drain(..).collect();
            if handles.is_empty() {
                return;
            }
            for handle in handles {
                while !handle.is_finished() {
                    assert!(
                        Instant::now() < deadline,
                        "a runtime thread did not finish within the bound"
                    );
                    thread::sleep(Duration::from_millis(5));
                }
                let _ = handle.join();
            }
        }
    }

    pub(crate) fn hard_stop(&self) {
        self.events.update(|state| state.hard_stop = true);
    }

    /// Every runtime thread's exit condition: the exited flag, and no slot
    /// sender left. Joins nothing.
    pub(crate) fn release_threads(&self) {
        self.events.update(|state| state.exited = true);
        lock(&self.slot_senders).clear();
    }

    pub(crate) fn summary_for_test(&self) -> String {
        self.summary_line()
    }

    pub(crate) fn pools_built(&self) -> bool {
        self.pools.get().is_some()
    }

    /// Move every in-flight claim's local deadline estimate to `now +
    /// remaining`.
    pub(crate) fn set_remaining(&self, remaining: Duration) {
        let at = Instant::now() + remaining;
        for state in self.claims_snapshot() {
            state.status().deadline_estimate = at;
        }
    }
}
