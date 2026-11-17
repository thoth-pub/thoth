//! Test support for the worker (spec section 8): the `cfg(test)` hooks the
//! runtime consults, the worker test kinds and registries, scripted
//! handlers, re-implementations of the engine's private fixtures, a test
//! runtime driver and the capturing logger.
//!
//! The fixtures duplicate the engine's private test support on purpose:
//! changing the engine module's visibility is outside the write budget.

use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::BigInt;
use serde_json::{json, Value};
use thoth_errors::{ThothError, ThothResult};
use uuid::Uuid;

use super::config::{WorkerArgs, WorkerConfig};
use super::handler::{
    ExecutionAction, ExecutionContext, HandlerRegistry, JobHandler, JobView, ReconciliationAction,
    ReconciliationContext,
};
use super::runtime::{lock, Op, Runtime, Shared, Timings, WorkerHandle};
use crate::model::async_job::{
    create_job_on, get_job, job_attempts, with_engine_transaction_on, AsyncJob, AsyncJobAttempt,
    AsyncJobKindPolicy, AsyncJobRegistry, CancelPolicy, CompleteRequest, CompletionOutcome,
    CreateOutcome, EffectClass, EvidenceContext, EvidenceEntry, EvidenceUse, ExecutorDomain,
    FailRequest, FailureClass, NewAsyncJob, PayloadError, RevisionMode, RiskClass, StaffOutcome,
};
use crate::model::tests::db::test_db_url;

/// The bound of every condition wait in the worker tests.
pub(crate) const BOUND: Duration = Duration::from_secs(30);

// ---------------------------------------------------------------------------
// Hooks the runtime consults (cfg(test) only)
// ---------------------------------------------------------------------------

/// The error a scripted fault answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FaultError {
    Busy,
    Integrity,
}

/// One scripted answer of the call-wrapper fault hook: run the call through
/// `with_engine_transaction` or not, then an optional side effect, then
/// answer `error` (whatever the call's own result) or the call's result.
pub(crate) struct Fault {
    error: Option<FaultError>,
    run: bool,
    then: Option<Box<dyn FnOnce() + Send>>,
}

impl Fault {
    /// `ASYNC_JOB_BUSY` without running the call.
    pub(crate) fn busy() -> Self {
        Fault {
            error: Some(FaultError::Busy),
            run: false,
            then: None,
        }
    }

    /// `ASYNC_JOB_BUSY` after running (and committing) the call.
    pub(crate) fn busy_after_commit() -> Self {
        Fault {
            error: Some(FaultError::Busy),
            run: true,
            then: None,
        }
    }

    /// `ASYNC_JOB_BUSY` without running the call, after `then`.
    pub(crate) fn busy_then(then: impl FnOnce() + Send + 'static) -> Self {
        Fault {
            error: Some(FaultError::Busy),
            run: false,
            then: Some(Box::new(then)),
        }
    }

    /// `ASYNC_JOB_INTEGRITY_VIOLATION` without running the call.
    pub(crate) fn integrity() -> Self {
        Fault {
            error: Some(FaultError::Integrity),
            run: false,
            then: None,
        }
    }

    /// Run the call, then `then`, and answer the call's own result.
    pub(crate) fn pass_then(then: impl FnOnce() + Send + 'static) -> Self {
        Fault {
            error: None,
            run: true,
            then: Some(Box::new(then)),
        }
    }

    pub(crate) fn apply<T>(self, call: impl FnOnce() -> ThothResult<T>) -> ThothResult<T> {
        let result = self.run.then(call);
        if let Some(then) = self.then {
            then();
        }
        match (self.error, result) {
            (Some(FaultError::Busy), _) => Err(ThothError::AsyncJobBusy),
            (Some(FaultError::Integrity), _) => Err(ThothError::AsyncJobIntegrityViolation),
            (None, Some(result)) => result,
            (None, None) => Err(ThothError::AsyncJobBusy),
        }
    }
}

/// A one-shot pause point: armed by a test, reached by a runtime thread,
/// which then waits (bounded) until the test releases it.
#[derive(Default)]
pub(crate) struct Gate {
    state: Mutex<(bool, bool, bool)>,
    cv: Condvar,
}

impl Gate {
    pub(crate) fn arm(&self) {
        *lock(&self.state) = (true, false, false);
    }

    /// Called by the runtime: pauses once when armed.
    pub(crate) fn pass(&self) {
        let mut state = lock(&self.state);
        if !state.0 {
            return;
        }
        state.0 = false;
        state.1 = true;
        self.cv.notify_all();
        let deadline = Instant::now() + BOUND;
        while !state.2 && Instant::now() < deadline {
            state = self
                .cv
                .wait_timeout(state, Duration::from_millis(50))
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .0;
        }
    }

    pub(crate) fn wait_reached(&self) -> bool {
        let mut state = lock(&self.state);
        let deadline = Instant::now() + BOUND;
        while !state.1 && Instant::now() < deadline {
            state = self
                .cv
                .wait_timeout(state, Duration::from_millis(50))
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .0;
        }
        state.1
    }

    pub(crate) fn release(&self) {
        lock(&self.state).2 = true;
        self.cv.notify_all();
    }
}

#[derive(Default)]
pub(crate) struct TestHooks {
    faults: Mutex<HashMap<Op, VecDeque<Fault>>>,
    calls: Mutex<HashMap<Op, usize>>,
    pub claim_gate: Gate,
    pub heartbeat_gate: Gate,
    pending_migrations: AtomicBool,
    pools_built: AtomicUsize,
    liveness: Mutex<Vec<Option<&'static str>>>,
    claim_loop_panic: AtomicBool,
}

impl TestHooks {
    /// Script the next answers of `op`, in order.
    pub(crate) fn push_faults(&self, op: Op, faults: Vec<Fault>) {
        lock(&self.faults).entry(op).or_default().extend(faults);
    }

    pub(crate) fn take_fault(&self, op: Op) -> Option<Fault> {
        *lock(&self.calls).entry(op).or_default() += 1;
        lock(&self.faults)
            .get_mut(&op)
            .and_then(VecDeque::pop_front)
    }

    /// How many calls of `op` consulted the hook.
    pub(crate) fn calls(&self, op: Op) -> usize {
        lock(&self.calls).get(&op).copied().unwrap_or(0)
    }

    pub(crate) fn set_pending_migrations(&self, pending: bool) {
        self.pending_migrations.store(pending, Ordering::SeqCst);
    }

    pub(crate) fn force_pending_migrations(&self) -> bool {
        self.pending_migrations.load(Ordering::SeqCst)
    }

    pub(crate) fn note_pools_built(&self) {
        self.pools_built.fetch_add(1, Ordering::SeqCst);
    }

    pub(crate) fn pools_built(&self) -> usize {
        self.pools_built.load(Ordering::SeqCst)
    }

    pub(crate) fn record_liveness(&self, reason: Option<&'static str>) {
        let mut records = lock(&self.liveness);
        if records.len() < 100_000 {
            records.push(reason);
        }
    }

    pub(crate) fn liveness(&self) -> Vec<Option<&'static str>> {
        lock(&self.liveness).clone()
    }

    pub(crate) fn inject_claim_loop_panic(&self) {
        self.claim_loop_panic.store(true, Ordering::SeqCst);
    }

    pub(crate) fn take_claim_loop_panic(&self) -> bool {
        self.claim_loop_panic.swap(false, Ordering::SeqCst)
    }

    pub(crate) fn pause_after_claim_commit(&self) {
        self.claim_gate.pass();
    }

    pub(crate) fn pause_heartbeat_renewal(&self) {
        self.heartbeat_gate.pass();
    }
}

// ---------------------------------------------------------------------------
// Worker test kinds
// ---------------------------------------------------------------------------

pub(crate) const W_READ: &str = "WTEST_READ";
pub(crate) const W_EFFECT: &str = "WTEST_EFFECT";
pub(crate) const W_REVISION: &str = "WTEST_REVISION";
pub(crate) const W_RESUMABLE: &str = "WTEST_RESUMABLE";
pub(crate) const W_CAPABILITY: &str = "WTEST_CAPABILITY";
pub(crate) const W_LONG: &str = "WTEST_LONG";
pub(crate) const W_SHORT: &str = "WTEST_SHORT";
pub(crate) const W_UNTRUSTED: &str = "WTEST_UNTRUSTED";
pub(crate) const W_DISSEMINATION: &str = "WTEST_DISSEMINATION";
pub(crate) const W_RECONCILER: &str = "WTEST_RECONCILER";
pub(crate) const W_LIMITED: &str = "WTEST_LIMITED";
pub(crate) const W_X: &str = "WTEST_X";
pub(crate) const W_Y: &str = "WTEST_Y";

pub(crate) const NS_EFFECT: &str = "wtest_cs";
pub(crate) const NS_REVISION: &str = "wtest_rb";
pub(crate) const NS_RESUMABLE: &str = "wtest_resume";
pub(crate) const NS_LONG: &str = "wtest_long";
pub(crate) const W_CAPABILITY_NAME: &str = "wtest.capability";

fn validate_payload(_version: i16, payload: &Value) -> Result<(), PayloadError> {
    match payload.get("id").and_then(Value::as_str) {
        Some(id) if !id.is_empty() => Ok(()),
        _ => Err(PayloadError::new("the payload needs an id")),
    }
}

fn effect_identity(_version: i16, payload: &Value) -> Result<String, PayloadError> {
    Ok(payload
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string())
}

fn targets(_version: i16, payload: &Value) -> Result<Vec<String>, PayloadError> {
    Ok(payload
        .get("targets")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default())
}

/// Applied evidence is optional (`{"valid": true}` when present); no-effect
/// evidence is `{"valid": true}`; not-applied is `{"notApplied": true}`.
fn validate_evidence(
    use_: EvidenceUse,
    _context: &EvidenceContext<'_>,
    evidence: &Value,
) -> Result<(), PayloadError> {
    let valid = evidence.get("valid") == Some(&Value::Bool(true));
    match use_ {
        EvidenceUse::Applied if evidence.is_null() || valid => Ok(()),
        EvidenceUse::NoEffect if valid => Ok(()),
        EvidenceUse::NotApplied if evidence.get("notApplied") == Some(&Value::Bool(true)) => Ok(()),
        _ => Err(PayloadError::new("the evidence is not valid")),
    }
}

const ALL_STAFF: &[StaffOutcome] = &[
    StaffOutcome::Succeeded,
    StaffOutcome::Failed,
    StaffOutcome::Cancelled,
    StaffOutcome::Waiting,
];

const REVISION_STAFF: &[StaffOutcome] = &[
    StaffOutcome::Succeeded,
    StaffOutcome::Failed,
    StaffOutcome::Cancelled,
    StaffOutcome::Waiting,
    StaffOutcome::AcceptRegression,
];

/// The effectful `CURRENT_STATE` kind every other worker kind derives from.
pub(crate) const EFFECT: AsyncJobKindPolicy = AsyncJobKindPolicy {
    kind: W_EFFECT,
    owning_domain: "worker_test",
    executor_domain: ExecutorDomain::ThothWorker,
    reconciler_domain: ExecutorDomain::ThothWorker,
    risk_class: RiskClass::Trusted,
    required_capabilities: &[],
    creatable_versions: &[1],
    executable_versions: &[1],
    effect_class: EffectClass::Effectful,
    revision_mode: RevisionMode::CurrentState,
    target_namespaces: &[NS_EFFECT],
    admission_open: true,
    retry_budget: 3,
    max_execution_claims: 4,
    max_reconciliation_attempts: 3,
    backoff_base_seconds: 10,
    backoff_max_seconds: 300,
    poll_interval_min_seconds: 5,
    lease_default_seconds: 60,
    lease_min_seconds: 15,
    lease_max_seconds: 300,
    max_attempt_runtime_seconds: 600,
    effect_window_seconds: Some(30),
    waiting_horizon_seconds: 3600,
    concurrency_limit: 10,
    reconciliation_concurrency_limit: 5,
    max_targets: 4,
    cancellable: CancelPolicy::ALL,
    manual_retry_allowed: true,
    no_effect_after_start_allowed: true,
    confirmed_effect_resumable: false,
    staff_reconciliation_outcomes: ALL_STAFF,
    validate_payload,
    effect_identity,
    targets,
    validate_evidence,
};

pub(crate) const READ: AsyncJobKindPolicy = AsyncJobKindPolicy {
    kind: W_READ,
    effect_class: EffectClass::ReadOnly,
    effect_window_seconds: None,
    target_namespaces: &[],
    max_targets: 0,
    staff_reconciliation_outcomes: &[],
    ..EFFECT
};

pub(crate) const REVISION: AsyncJobKindPolicy = AsyncJobKindPolicy {
    kind: W_REVISION,
    revision_mode: RevisionMode::RevisionBound,
    target_namespaces: &[NS_REVISION],
    max_targets: 1,
    staff_reconciliation_outcomes: REVISION_STAFF,
    ..EFFECT
};

pub(crate) const RESUMABLE: AsyncJobKindPolicy = AsyncJobKindPolicy {
    kind: W_RESUMABLE,
    target_namespaces: &[NS_RESUMABLE],
    confirmed_effect_resumable: true,
    ..EFFECT
};

pub(crate) const CAPABILITY: AsyncJobKindPolicy = AsyncJobKindPolicy {
    kind: W_CAPABILITY,
    required_capabilities: &[W_CAPABILITY_NAME],
    ..EFFECT
};

/// The effect window (90 s) is above the default lease (60 s), within rule
/// 7: `E = max(60, 90 + 10) = 100`.
pub(crate) const LONG: AsyncJobKindPolicy = AsyncJobKindPolicy {
    kind: W_LONG,
    target_namespaces: &[NS_LONG],
    effect_window_seconds: Some(90),
    ..EFFECT
};

/// A 15 s lease and a 20 s runtime: renewals every 5 s reach the deadline.
pub(crate) const SHORT: AsyncJobKindPolicy = AsyncJobKindPolicy {
    kind: W_SHORT,
    lease_default_seconds: 15,
    lease_min_seconds: 15,
    lease_max_seconds: 15,
    max_attempt_runtime_seconds: 20,
    ..READ
};

pub(crate) const UNTRUSTED: AsyncJobKindPolicy = AsyncJobKindPolicy {
    kind: W_UNTRUSTED,
    risk_class: RiskClass::UntrustedContent,
    ..READ
};

pub(crate) const DISSEMINATION: AsyncJobKindPolicy = AsyncJobKindPolicy {
    kind: W_DISSEMINATION,
    executor_domain: ExecutorDomain::Dissemination,
    reconciler_domain: ExecutorDomain::Dissemination,
    ..READ
};

/// Executed by Dissemination, reconciled by the worker.
pub(crate) const RECONCILER: AsyncJobKindPolicy = AsyncJobKindPolicy {
    kind: W_RECONCILER,
    executor_domain: ExecutorDomain::Dissemination,
    ..READ
};

pub(crate) const LIMITED: AsyncJobKindPolicy = AsyncJobKindPolicy {
    kind: W_LIMITED,
    concurrency_limit: 3,
    ..READ
};

pub(crate) const X: AsyncJobKindPolicy = AsyncJobKindPolicy {
    kind: W_X,
    creatable_versions: &[1, 2],
    executable_versions: &[1, 2],
    ..READ
};

pub(crate) const Y: AsyncJobKindPolicy = AsyncJobKindPolicy { kind: W_Y, ..READ };

/// Every worker test kind.
pub(crate) static REGISTRY: AsyncJobRegistry = AsyncJobRegistry::new(&[
    READ,
    EFFECT,
    REVISION,
    RESUMABLE,
    CAPABILITY,
    LONG,
    SHORT,
    UNTRUSTED,
    DISSEMINATION,
    RECONCILER,
    LIMITED,
    X,
    Y,
]);

/// An "older" binary: kind X version 1 only, no kind Y.
pub(crate) static OLDER_REGISTRY: AsyncJobRegistry = AsyncJobRegistry::new(&[AsyncJobKindPolicy {
    creatable_versions: &[1],
    executable_versions: &[1],
    ..X
}]);

// ---------------------------------------------------------------------------
// Scripted handlers
// ---------------------------------------------------------------------------

pub(crate) type ExecuteFn =
    Arc<dyn Fn(&JobView, &mut ExecutionContext<'_>) -> ExecutionAction + Send + Sync>;
pub(crate) type ReconcileFn =
    Arc<dyn Fn(&JobView, &mut ReconciliationContext<'_>) -> ReconciliationAction + Send + Sync>;

pub(crate) struct ScriptedHandler {
    pub kind: &'static str,
    pub versions: &'static [i16],
    pub execute: ExecuteFn,
    pub reconcile: ReconcileFn,
}

impl JobHandler for ScriptedHandler {
    fn kind(&self) -> &'static str {
        self.kind
    }

    fn payload_versions(&self) -> &'static [i16] {
        self.versions
    }

    fn execute(&self, job: &JobView, ctx: &mut ExecutionContext<'_>) -> ExecutionAction {
        (self.execute)(job, ctx)
    }

    fn reconcile(
        &self,
        job: &JobView,
        ctx: &mut ReconciliationContext<'_>,
    ) -> ReconciliationAction {
        (self.reconcile)(job, ctx)
    }
}

/// A leaked handler.
pub(crate) fn handler(
    kind: &'static str,
    versions: &'static [i16],
    execute: ExecuteFn,
    reconcile: ReconcileFn,
) -> &'static dyn JobHandler {
    Box::leak(Box::new(ScriptedHandler {
        kind,
        versions,
        execute,
        reconcile,
    }))
}

/// A leaked registry of `handlers`.
pub(crate) fn handler_registry(handlers: Vec<&'static dyn JobHandler>) -> &'static HandlerRegistry {
    Box::leak(Box::new(HandlerRegistry::new(Box::leak(
        handlers.into_boxed_slice(),
    ))))
}

/// One handler for every `Trusted` `ThothWorker` kind of `registry`, all
/// running `execute` and `reconcile`, each serving the kind's executable
/// versions.
pub(crate) fn handlers_for(
    registry: &'static AsyncJobRegistry,
    execute: ExecuteFn,
    reconcile: ReconcileFn,
) -> &'static HandlerRegistry {
    handler_registry(
        registry
            .policies()
            .iter()
            .filter(|policy| {
                policy.risk_class == RiskClass::Trusted
                    && (policy.executor_domain == ExecutorDomain::ThothWorker
                        || policy.reconciler_domain == ExecutorDomain::ThothWorker)
            })
            .map(|policy| {
                handler(
                    policy.kind,
                    policy.executable_versions,
                    Arc::clone(&execute),
                    Arc::clone(&reconcile),
                )
            })
            .collect(),
    )
}

pub(crate) fn exec_fn(
    f: impl Fn(&JobView, &mut ExecutionContext<'_>) -> ExecutionAction + Send + Sync + 'static,
) -> ExecuteFn {
    Arc::new(f)
}

pub(crate) fn rec_fn(
    f: impl Fn(&JobView, &mut ReconciliationContext<'_>) -> ReconciliationAction + Send + Sync + 'static,
) -> ReconcileFn {
    Arc::new(f)
}

/// A reconciler that always answers `StillRequired`.
pub(crate) fn still_required() -> ReconcileFn {
    rec_fn(|_, _| ReconciliationAction::still_required())
}

/// `Complete` with the right entries for the job's kind: key-only
/// `COMPLETED` for a read-only kind, `EFFECT_APPLIED` with a fingerprint
/// (and the order for a revision-bound job) otherwise.
pub(crate) fn complete(job: &JobView) -> ExecutionAction {
    let policy = REGISTRY.get(&job.kind).expect("a worker test kind");
    ExecutionAction::Complete(complete_request(policy, job))
}

pub(crate) fn complete_request(policy: &AsyncJobKindPolicy, job: &JobView) -> CompleteRequest {
    if policy.is_effectful() {
        CompleteRequest {
            outcome: CompletionOutcome::EffectApplied,
            entries: applied_entries(job),
            evidence: None,
        }
    } else {
        CompleteRequest {
            outcome: CompletionOutcome::Completed,
            entries: job
                .target_keys
                .iter()
                .map(|key| EvidenceEntry::key_only(key.clone()))
                .collect(),
            evidence: None,
        }
    }
}

pub(crate) fn applied_entries(job: &JobView) -> Vec<EvidenceEntry> {
    job.target_keys
        .iter()
        .map(|key| EvidenceEntry {
            target_key: key.clone(),
            order: job.source_order.map(i128::from),
            fingerprint: Some(
                job.source_fingerprint
                    .clone()
                    .unwrap_or_else(|| "fingerprint-1".to_string()),
            ),
            source_ref: job.source_ref.clone(),
        })
        .collect()
}

pub(crate) fn fail(class: FailureClass, code: &str) -> ExecutionAction {
    ExecutionAction::Fail(FailRequest {
        class,
        error_code: code.to_string(),
        error_detail: None,
        no_effect_proven: false,
        evidence: None,
    })
}

// ---------------------------------------------------------------------------
// Synchronisation for scripted handlers
// ---------------------------------------------------------------------------

/// A flag a test or a handler sets once and the other waits for (bounded).
#[derive(Default)]
pub(crate) struct Flag {
    set: Mutex<bool>,
    cv: Condvar,
}

impl Flag {
    pub(crate) fn new() -> Arc<Flag> {
        Arc::new(Flag::default())
    }

    pub(crate) fn set(&self) {
        *lock(&self.set) = true;
        self.cv.notify_all();
    }

    /// Wait up to [`BOUND`]; true when set.
    pub(crate) fn wait(&self) -> bool {
        let mut set = lock(&self.set);
        let deadline = Instant::now() + BOUND;
        while !*set && Instant::now() < deadline {
            set = self
                .cv
                .wait_timeout(set, Duration::from_millis(20))
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .0;
        }
        *set
    }
}

/// Poll `condition` every 20 ms for up to [`BOUND`].
pub(crate) fn wait_for(mut condition: impl FnMut() -> bool) -> bool {
    let deadline = Instant::now() + BOUND;
    loop {
        if condition() {
            return true;
        }
        if Instant::now() >= deadline {
            // The worker's own lines, for the failure report. They carry no
            // secret (W21).
            for record in logger().records().iter().rev().take(40).rev() {
                eprintln!("{record}");
            }
            return false;
        }
        thread::sleep(Duration::from_millis(20));
    }
}

// ---------------------------------------------------------------------------
// The test runtime driver
// ---------------------------------------------------------------------------

/// A fresh absolute liveness path under the temporary directory.
pub(crate) fn liveness_path() -> PathBuf {
    std::env::temp_dir()
        .join("thoth-worker-tests")
        .join(format!("liveness-{}", Uuid::new_v4()))
}

/// The test database URL with an `application_name`, so a test can find
/// the runtime's backends.
pub(crate) fn url_with_application(name: &str) -> String {
    let url = test_db_url();
    let separator = if url.contains('?') { '&' } else { '?' };
    format!("{url}{separator}application_name={name}")
}

/// A validated configuration for `kinds`.
pub(crate) fn config(kinds: &[&str], slots: usize, database_url: &str) -> WorkerConfig {
    let args = WorkerArgs {
        kinds: kinds.join(","),
        slots: slots.to_string(),
        capabilities: W_CAPABILITY_NAME.to_string(),
        liveness_file: liveness_path().to_string_lossy().into_owned(),
        ..WorkerArgs::with_defaults(Some(database_url.to_string()))
    };
    WorkerConfig::from_args(&args).expect("a valid test configuration")
}

/// Every timing small, no jitter, renewal at a third of the default lease.
pub(crate) fn fast_timings(config: &WorkerConfig) -> Timings {
    Timings {
        poll: Duration::from_millis(100),
        grace: Duration::from_secs(3),
        busy_retries: vec![
            Duration::from_millis(50),
            Duration::from_millis(100),
            Duration::from_millis(200),
        ],
        fallback_retry: Duration::from_millis(50),
        liveness_cadence: Duration::from_millis(50),
        claim_loop_join_bound: Duration::from_secs(10),
        fallback_phase_bound: Duration::from_secs(10),
        heartbeat_wake: Duration::from_millis(50),
        migration_recheck: Duration::from_millis(100),
        connect_backoff_start: Duration::from_millis(50),
        connect_backoff_cap: Duration::from_millis(200),
        busy_backoff_cap: Duration::from_millis(500),
        pool_checkout: Duration::from_secs(5),
        jitter: false,
        ..Timings::production(config)
    }
}

/// A runtime running on its own thread.
pub(crate) struct TestWorker {
    pub handle: WorkerHandle,
    pub hooks: Arc<TestHooks>,
    pub config: WorkerConfig,
    run: Option<JoinHandle<i32>>,
    joined: bool,
}

impl TestWorker {
    pub(crate) fn start(
        config: WorkerConfig,
        registry: &'static AsyncJobRegistry,
        handlers: &'static HandlerRegistry,
        timings: Timings,
    ) -> TestWorker {
        let hooks = Arc::new(TestHooks::default());
        TestWorker::start_with(config, registry, handlers, timings, hooks)
    }

    pub(crate) fn start_with(
        config: WorkerConfig,
        registry: &'static AsyncJobRegistry,
        handlers: &'static HandlerRegistry,
        timings: Timings,
        hooks: Arc<TestHooks>,
    ) -> TestWorker {
        let runtime = Runtime::for_test(
            config.clone(),
            registry,
            handlers,
            timings,
            Arc::clone(&hooks),
        );
        let handle = runtime.handle();
        let run = thread::Builder::new()
            .name("async-worker-test-main".to_string())
            .spawn(move || runtime.run())
            .expect("spawn the test runtime");
        TestWorker {
            handle,
            hooks,
            config,
            run: Some(run),
            joined: false,
        }
    }

    pub(crate) fn shared(&self) -> &Arc<Shared> {
        self.handle.shared()
    }

    /// The first stop signal, then the exit code; every runtime thread is
    /// joined.
    pub(crate) fn stop(mut self) -> i32 {
        self.handle.request_shutdown();
        let code = self.wait_code();
        self.join_threads();
        code
    }

    /// The first stop signal, then the exit code, then `after` (to release
    /// handlers the test kept blocked), then every runtime thread joined.
    pub(crate) fn stop_then(mut self, after: impl FnOnce()) -> i32 {
        self.handle.request_shutdown();
        let code = self.wait_code();
        after();
        self.join_threads();
        code
    }

    /// The hard stop (W16): every thread exits with no final write. `after`
    /// runs once the run function has returned.
    pub(crate) fn hard_stop_then(mut self, after: impl FnOnce()) -> i32 {
        self.shared().hard_stop();
        let code = self.wait_code();
        after();
        self.join_threads();
        code
    }

    /// Wait (bounded) for the run function's exit code.
    pub(crate) fn wait_code(&mut self) -> i32 {
        let run = self.run.take().expect("waited once");
        let deadline = Instant::now() + Duration::from_secs(90);
        while !run.is_finished() {
            assert!(
                Instant::now() < deadline,
                "the runtime did not return within the bound"
            );
            thread::sleep(Duration::from_millis(10));
        }
        run.join().expect("the run function returned")
    }

    /// Join every runtime thread, bounded.
    pub(crate) fn join_threads(&mut self) {
        if !self.joined {
            self.joined = true;
            self.shared().join_threads(Duration::from_secs(60));
        }
    }

    pub(crate) fn liveness_file(&self) -> PathBuf {
        self.config.liveness_file.clone()
    }
}

impl Drop for TestWorker {
    fn drop(&mut self) {
        // Always hard-stop, so a failed test leaves no runtime claiming the
        // next test's jobs.
        self.shared().hard_stop();
        if thread::panicking() {
            // Bounded and without asserting: a second panic would abort.
            // Slot threads blocked in a test handler are abandoned; the
            // claim loop, the heartbeat and the ticker exit on the hard stop.
            if let Some(run) = self.run.take() {
                let deadline = Instant::now() + Duration::from_secs(30);
                while !run.is_finished() && Instant::now() < deadline {
                    thread::sleep(Duration::from_millis(10));
                }
                if run.is_finished() {
                    let _ = run.join();
                }
            }
            self.shared().release_threads();
        } else {
            if self.run.is_some() {
                let _ = self.wait_code();
            }
            self.join_threads();
        }
        let _ = std::fs::remove_file(&self.config.liveness_file);
    }
}

// ---------------------------------------------------------------------------
// Database fixtures (re-implementations of the engine's private ones)
// ---------------------------------------------------------------------------

pub(crate) fn connect() -> PgConnection {
    PgConnection::establish(&test_db_url()).expect("connect to the test database")
}

#[derive(QueryableByName)]
struct I64Row {
    #[diesel(sql_type = BigInt)]
    value: i64,
}

pub(crate) fn scalar(conn: &mut PgConnection, sql: &str) -> i64 {
    sql_query(sql)
        .get_result::<I64Row>(conn)
        .unwrap_or_else(|error| panic!("{sql}: {error}"))
        .value
}

pub(crate) fn exec(conn: &mut PgConnection, sql: &str) {
    conn.batch_execute(sql)
        .unwrap_or_else(|error| panic!("{sql}: {error}"));
}

/// Run `sql` with every ordinary trigger disabled.
pub(crate) fn bypass(conn: &mut PgConnection, sql: &str) {
    conn.transaction(|conn| {
        conn.batch_execute("SET LOCAL session_replication_role = replica")?;
        conn.batch_execute(sql)
    })
    .unwrap_or_else(|error| panic!("{sql}: {error}"));
}

/// A version-1 job of `kind` with payload `{"id": id, "targets": targets}`.
pub(crate) fn new_job(kind: &str, id: &str, targets: &[String]) -> NewAsyncJob {
    NewAsyncJob {
        kind: kind.to_string(),
        payload_version: 1,
        payload: json!({ "id": id, "targets": targets }).to_string(),
        priority: 0,
        source_order: None,
        source_fingerprint: None,
        source_ref: None,
        correlation_id: None,
        causation_ref: None,
        created_by: "test:creator".to_string(),
    }
}

pub(crate) fn key(namespace: &str) -> String {
    format!("{namespace}:{}", Uuid::new_v4())
}

pub(crate) fn create(conn: &mut PgConnection, input: NewAsyncJob) -> AsyncJob {
    create_in(conn, &REGISTRY, input)
}

pub(crate) fn create_in(
    conn: &mut PgConnection,
    registry: &AsyncJobRegistry,
    input: NewAsyncJob,
) -> AsyncJob {
    match with_engine_transaction_on(conn, |c| create_job_on(c, registry, input)).expect("create") {
        CreateOutcome::Created(job) => job,
        CreateOutcome::Existing(_) => panic!("expected a new job"),
    }
}

/// A job of `kind`: one target for an effectful kind, none otherwise.
pub(crate) fn create_kind(conn: &mut PgConnection, kind: &str) -> AsyncJob {
    let policy = REGISTRY.get(kind).expect("a worker test kind");
    let targets: Vec<String> = policy
        .target_namespaces
        .first()
        .map(|namespace| vec![key(namespace)])
        .unwrap_or_default();
    create(conn, new_job(kind, &Uuid::new_v4().to_string(), &targets))
}

pub(crate) fn job(conn: &mut PgConnection, id: Uuid) -> AsyncJob {
    get_job(conn, id).expect("job")
}

pub(crate) fn attempts(conn: &mut PgConnection, id: Uuid) -> Vec<AsyncJobAttempt> {
    job_attempts(conn, id).expect("attempts")
}

pub(crate) fn expire_lease(conn: &mut PgConnection, id: Uuid) {
    exec(
        conn,
        &format!(
            "UPDATE public.async_job SET lease_expires_at = clock_timestamp() - interval '1 second' \
              WHERE async_job_id = '{id}'"
        ),
    );
}

pub(crate) fn make_available(conn: &mut PgConnection, id: Uuid) {
    exec(
        conn,
        &format!(
            "UPDATE public.async_job SET available_at = clock_timestamp() - interval '1 second' \
              WHERE async_job_id = '{id}'"
        ),
    );
}

/// Move the write-once waiting deadline to `seconds` from now (negative:
/// into the past).
pub(crate) fn set_waiting_deadline(conn: &mut PgConnection, id: Uuid, seconds: i64) {
    bypass(
        conn,
        &format!(
            "UPDATE public.async_job \
                SET waiting_deadline = clock_timestamp() + interval '{seconds} seconds' \
              WHERE async_job_id = '{id}'"
        ),
    );
}

/// Move the attempt runtime deadline (and a lease past it) to `seconds`
/// from now.
pub(crate) fn set_runtime_deadline(conn: &mut PgConnection, id: Uuid, seconds: i64) {
    bypass(
        conn,
        &format!(
            "UPDATE public.async_job \
                SET attempt_runtime_deadline = clock_timestamp() + interval '{seconds} seconds', \
                    lease_expires_at = LEAST(lease_expires_at, \
                        clock_timestamp() + interval '{seconds} seconds') \
              WHERE async_job_id = '{id}'"
        ),
    );
}

pub(crate) fn db_now(conn: &mut PgConnection) -> DateTime<Utc> {
    super::runtime::db_clock(conn).expect("clock")
}

/// The job's status label.
pub(crate) fn status(conn: &mut PgConnection, id: Uuid) -> String {
    job(conn, id).status.as_str().to_string()
}

/// Wait until the job's status is `expected`.
pub(crate) fn wait_status(conn: &mut PgConnection, id: Uuid, expected: &str) -> bool {
    wait_for(|| status(conn, id) == expected)
}

// ---------------------------------------------------------------------------
// The capturing logger
// ---------------------------------------------------------------------------

pub(crate) struct CapturingLogger {
    records: Mutex<Vec<String>>,
}

impl log::Log for CapturingLogger {
    fn enabled(&self, _metadata: &log::Metadata<'_>) -> bool {
        true
    }

    fn log(&self, record: &log::Record<'_>) {
        let mut records = lock(&self.records);
        if records.len() < 500_000 {
            records.push(format!(
                "{} {} {}",
                record.level(),
                record.target(),
                record.args()
            ));
        }
    }

    fn flush(&self) {}
}

static LOGGER: OnceLock<&'static CapturingLogger> = OnceLock::new();

/// The capturing logger, installed once. Its maximum level is set only
/// here, at installation.
pub(crate) fn logger() -> &'static CapturingLogger {
    LOGGER.get_or_init(|| {
        let logger: &'static CapturingLogger = Box::leak(Box::new(CapturingLogger {
            records: Mutex::new(Vec::new()),
        }));
        if log::set_logger(logger).is_ok() {
            log::set_max_level(log::LevelFilter::Trace);
        }
        logger
    })
}

impl CapturingLogger {
    pub(crate) fn records(&self) -> Vec<String> {
        lock(&self.records).clone()
    }

    /// The records containing `needle`.
    pub(crate) fn containing(&self, needle: &str) -> Vec<String> {
        lock(&self.records)
            .iter()
            .filter(|record| record.contains(needle))
            .cloned()
            .collect()
    }
}
