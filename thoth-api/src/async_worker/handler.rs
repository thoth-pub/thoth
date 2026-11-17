//! The handler contract (spec section 5.1): [`JobHandler`], [`JobView`], the
//! two contexts, the actions, [`HandlerError`] and the [`HandlerRegistry`]
//! with its seven validation rules (spec section 5.3).
//!
//! The context, not the handler, holds the claim token: every token-fenced
//! engine call a handler can make goes through its context. That is
//! confinement by API shape, not a security boundary (see the module docs of
//! [`crate::async_worker`]).

use std::collections::HashSet;
use std::fmt;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use diesel::PgConnection;
use thoth_errors::ThothError;
use uuid::Uuid;

use super::errors::{error_code, is_phase_refusal, refusal_reason};
use super::runtime::{
    bump, effect_lease_request, ClaimState, Op, Shared, TrackedPhase, EFFECT_MARGIN_SECONDS,
};
use crate::db::PgPool;
use crate::model::async_job::{
    mark_effect_confirmed_on, mark_effect_started_on, read_target_state_on, renew_on,
    AsyncEffectPhase, AsyncJobRegistry, ClaimedJob, CompleteRequest, EvidenceEntry, ExecutorDomain,
    FailRequest, LeaseRenewal, ReconcilerOutcome, RiskClass, TargetState,
};

// ---------------------------------------------------------------------------
// The trait and the job view
// ---------------------------------------------------------------------------

/// One kind's in-process handler.
pub trait JobHandler: Send + Sync + 'static {
    /// The kind served. Registered with `executor_domain` or
    /// `reconciler_domain` `ThothWorker` and risk class `Trusted`.
    fn kind(&self) -> &'static str;
    /// Payload versions executed and reconciled: non-empty, a subset of the
    /// kind's `executable_versions`.
    fn payload_versions(&self) -> &'static [i16];
    /// One execution attempt (kinds whose `executor_domain` is
    /// `ThothWorker`).
    fn execute(&self, job: &JobView, ctx: &mut ExecutionContext<'_>) -> ExecutionAction;
    /// One reconciliation attempt (kinds whose `reconciler_domain` is
    /// `ThothWorker`).
    fn reconcile(&self, job: &JobView, ctx: &mut ReconciliationContext<'_>)
        -> ReconciliationAction;
}

/// Every field of the claimed job except the claim token. The effect phase
/// is the phase **at claim**. `Debug` prints only the id, kind, payload
/// version, attempt number and phase.
#[derive(Clone)]
pub struct JobView {
    pub async_job_id: Uuid,
    pub kind: String,
    pub payload_version: i16,
    /// The stored payload text.
    pub payload: String,
    /// The stored payload, parsed.
    pub payload_value: serde_json::Value,
    pub source_order: Option<i64>,
    pub source_fingerprint: Option<String>,
    pub source_ref: Option<String>,
    pub target_keys: Vec<String>,
    /// The lease expiry the claim returned.
    pub lease_expires_at: DateTime<Utc>,
    /// The attempt runtime deadline the claim returned.
    pub attempt_runtime_deadline: DateTime<Utc>,
    pub attempt_number: i32,
    pub effect_phase: AsyncEffectPhase,
    pub checkpoint: Option<String>,
    pub checkpoint_version: i32,
    pub provider_ref: Option<String>,
}

impl JobView {
    /// The view of a claimed job; `None` when the stored payload does not
    /// parse (impossible after engine validation; the runtime handles it as
    /// a panic).
    pub(crate) fn from_claimed(job: &ClaimedJob) -> Option<JobView> {
        let payload_value = serde_json::from_str(&job.payload).ok()?;
        Some(JobView {
            async_job_id: job.async_job_id,
            kind: job.kind.clone(),
            payload_version: job.payload_version,
            payload: job.payload.clone(),
            payload_value,
            source_order: job.source_order,
            source_fingerprint: job.source_fingerprint.clone(),
            source_ref: job.source_ref.clone(),
            target_keys: job.target_keys.clone(),
            lease_expires_at: job.lease_expires_at,
            attempt_runtime_deadline: job.attempt_runtime_deadline,
            attempt_number: job.attempt_number,
            effect_phase: job.effect_phase,
            checkpoint: job.checkpoint.clone(),
            checkpoint_version: job.checkpoint_version,
            provider_ref: job.provider_ref.clone(),
        })
    }
}

impl fmt::Debug for JobView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JobView")
            .field("async_job_id", &self.async_job_id)
            .field("kind", &self.kind)
            .field("payload_version", &self.payload_version)
            .field("attempt_number", &self.attempt_number)
            .field("effect_phase", &self.effect_phase)
            .finish_non_exhaustive()
    }
}

// ---------------------------------------------------------------------------
// Errors and actions
// ---------------------------------------------------------------------------

/// What a context call answers instead of `Ok`. Closed, without free text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandlerError {
    /// The token is no longer the job's. Final.
    ClaimLost,
    /// The lease has expired (the token may still be current until
    /// recovery).
    LeaseExpired,
    /// The stop signal is set: no fence answers `Ok` any more.
    ShutdownInProgress,
    /// The tracked phase does not allow the call.
    Phase,
    /// The runtime deadline is too close for the kind's effect window.
    EffectWindowInsufficient,
    /// The database stayed busy; the outcome of an effect mark is unknown.
    Busy,
    /// The engine refused the call (an engine code or a refusal reason).
    Refused(&'static str),
    /// A runtime or database fault (an engine code).
    Failed(&'static str),
}

/// A `Wait`: the engine's `WaitRequest` with `resume_at` replaced by a
/// relative delay. The runtime computes `resume_at` from the database clock
/// (at least `poll_interval_min_seconds + RESUME_MARGIN` away, capped at the
/// job's waiting deadline).
#[derive(Clone, PartialEq, Eq)]
pub struct WaitAction {
    /// A JSON object as text: a checkpoint that makes continuation safe.
    pub checkpoint: String,
    pub resume_after: Duration,
    pub provider_ref: Option<String>,
}

impl fmt::Debug for WaitAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WaitAction")
            .field("resume_after", &self.resume_after)
            .finish_non_exhaustive()
    }
}

/// The one final action of an execution attempt.
#[derive(Clone)]
pub enum ExecutionAction {
    /// `complete_on` (T6, T6x).
    Complete(CompleteRequest),
    /// `fail_on` (T7, T8, T9, T9n).
    Fail(FailRequest),
    /// `wait_on` (T10, T10d), with `resume_at` from the database clock.
    Wait(WaitAction),
    /// `release_on` (T13): pre-write only.
    Release,
}

impl ExecutionAction {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            ExecutionAction::Complete(_) => "COMPLETE",
            ExecutionAction::Fail(_) => "FAIL",
            ExecutionAction::Wait(_) => "WAIT",
            ExecutionAction::Release => "RELEASE",
        }
    }
}

impl fmt::Debug for ExecutionAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// The one final action of a reconciliation attempt: the engine's
/// `ResolveReconciliation` with `resume_at` replaced by a relative delay
/// (used only by `Waiting`). The in-process reconciler cannot accept a
/// regression.
#[derive(Clone, PartialEq, Eq)]
pub struct ReconciliationAction {
    pub outcome: ReconcilerOutcome,
    pub entries: Option<Vec<EvidenceEntry>>,
    pub evidence: Option<String>,
    pub checkpoint: Option<String>,
    pub resume_after: Option<Duration>,
}

impl ReconciliationAction {
    /// `SUCCEEDED`: the effect applied, one entry per target.
    pub fn succeeded(entries: Vec<EvidenceEntry>, evidence: Option<String>) -> Self {
        ReconciliationAction {
            outcome: ReconcilerOutcome::Succeeded,
            entries: Some(entries),
            evidence,
            checkpoint: None,
            resume_after: None,
        }
    }

    /// `FAILED` on not-applied evidence (never before `effect_settled_at`).
    pub fn failed(evidence: Option<String>) -> Self {
        ReconciliationAction {
            outcome: ReconcilerOutcome::Failed,
            entries: None,
            evidence,
            checkpoint: None,
            resume_after: None,
        }
    }

    /// `CANCELLED` on not-applied evidence (never before
    /// `effect_settled_at`).
    pub fn cancelled(evidence: Option<String>) -> Self {
        ReconciliationAction {
            outcome: ReconcilerOutcome::Cancelled,
            ..ReconciliationAction::failed(evidence)
        }
    }

    /// `WAITING` with a checkpoint, resumed after `resume_after` (from the
    /// database clock, capped at the waiting deadline).
    pub fn waiting(checkpoint: String, resume_after: Duration) -> Self {
        ReconciliationAction {
            outcome: ReconcilerOutcome::Waiting,
            entries: None,
            evidence: None,
            checkpoint: Some(checkpoint),
            resume_after: Some(resume_after),
        }
    }

    /// Back to unclaimed `RECONCILIATION_REQUIRED`.
    pub fn still_required() -> Self {
        ReconciliationAction {
            outcome: ReconcilerOutcome::StillRequired,
            entries: None,
            evidence: None,
            checkpoint: None,
            resume_after: None,
        }
    }
}

impl fmt::Debug for ReconciliationAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReconciliationAction")
            .field("outcome", &self.outcome)
            .finish_non_exhaustive()
    }
}

// ---------------------------------------------------------------------------
// The contexts
// ---------------------------------------------------------------------------

/// What both contexts share: the runtime and the claim.
struct ClaimHandle<'a> {
    rt: &'a Shared,
    state: &'a ClaimState,
}

impl ClaimHandle<'_> {
    fn registry(&self) -> &'static AsyncJobRegistry {
        self.rt.registry
    }

    fn phase(&self) -> TrackedPhase {
        self.state.status().phase
    }

    fn set_phase(&self, phase: TrackedPhase) {
        self.state.status().phase = phase;
    }

    fn shutdown_requested(&self) -> bool {
        self.rt.stopping()
    }

    fn claim_lost(&self) -> bool {
        self.state.status().claim_lost
    }

    fn lease_expired(&self) -> bool {
        self.state.status().lease_expired
    }

    fn remaining_runtime(&self) -> Duration {
        self.state
            .status()
            .deadline_estimate
            .saturating_duration_since(Instant::now())
    }

    fn window(&self) -> Duration {
        Duration::from_secs(
            u64::try_from(self.state.policy.effect_window_seconds.unwrap_or(0)).unwrap_or(0),
        )
    }

    fn should_yield(&self) -> bool {
        let policy = self.state.policy;
        let horizon = Duration::from_secs(u64::try_from(policy.lease_default_seconds).unwrap_or(0))
            .max(self.window());
        self.shutdown_requested()
            || self.claim_lost()
            || self.lease_expired()
            || self.remaining_runtime() <= horizon
    }

    /// The local refusals of the fences, in order: the stop signal, then
    /// the claim flags.
    fn check_fence_flags(&self) -> Result<(), HandlerError> {
        if self.shutdown_requested() {
            return Err(HandlerError::ShutdownInProgress);
        }
        let status = self.state.status();
        if status.claim_lost {
            return Err(HandlerError::ClaimLost);
        }
        if status.lease_expired {
            return Err(HandlerError::LeaseExpired);
        }
        Ok(())
    }

    /// Map an engine answer, updating the claim flags.
    fn note_error(&self, error: &ThothError) -> HandlerError {
        match error {
            ThothError::AsyncJobStaleClaim => {
                self.rt.mark_lost(self.state);
                HandlerError::ClaimLost
            }
            ThothError::AsyncJobLeaseExpired => {
                self.rt.mark_expired(self.state);
                HandlerError::LeaseExpired
            }
            ThothError::AsyncJobBusy => {
                bump(&self.rt.counters.busy);
                HandlerError::Busy
            }
            ThothError::AsyncJobEffectWindowInsufficient => HandlerError::EffectWindowInsufficient,
            ThothError::AsyncJobTransitionRefused(_) if is_phase_refusal(error) => {
                HandlerError::Phase
            }
            ThothError::AsyncJobTransitionRefused(_) => {
                HandlerError::Refused(refusal_reason(error).unwrap_or("OTHER"))
            }
            ThothError::AsyncJobPayloadInvalid(_) | ThothError::AsyncJobEvidenceInvalid(_) => {
                HandlerError::Refused(error_code(error))
            }
            ThothError::AsyncJobKindNotAuthorized => {
                self.rt.fatal("a context call");
                HandlerError::Failed(error_code(error))
            }
            _ => HandlerError::Failed(error_code(error)),
        }
    }

    fn store(&self, before: Instant, renewal: &LeaseRenewal) {
        self.rt
            .store_renewal(self.state, before, renewal.lease_expires_at);
    }

    /// After a fence call returned: once the stop signal is set no fence
    /// answers `Ok` (spec section 6.5).
    fn stop_after_fence(&self) -> Result<(), HandlerError> {
        if self.shutdown_requested() {
            Err(HandlerError::ShutdownInProgress)
        } else {
            Ok(())
        }
    }

    /// Sleep before a `BUSY` retry; the stop signal ends it at once.
    fn retry_sleep(&self, delay: Duration) {
        self.rt.sleep_unless_stopped(delay);
    }

    fn read_target_state(&self) -> Result<Vec<TargetState>, HandlerError> {
        if self.claim_lost() {
            return Err(HandlerError::ClaimLost);
        }
        let registry = self.registry();
        let claim = self.state.claim;
        self.rt
            .engine_call(Op::ReadTargetState, |conn: &mut PgConnection| {
                read_target_state_on(conn, registry, &claim)
            })
            .map_err(|error| self.note_error(&error))
    }

    /// `mark_effect_started` (spec sections 5.1 and 6.5).
    fn mark_effect_started(&self) -> Result<(), HandlerError> {
        self.check_fence_flags()?;
        if self.phase() != TrackedPhase::PreWrite {
            return Err(HandlerError::Phase);
        }
        let request = effect_lease_request(self.state.policy);
        *self.state.renewal_lock() = Some(request);
        let registry = self.registry();
        let claim = self.state.claim;
        let mut delays = self.rt.timings.busy_retries.iter();
        let mut unknown = false;
        loop {
            if unknown && self.shutdown_requested() {
                self.set_phase(TrackedPhase::PossiblyStarted);
                self.state.status().shutdown_after_effect_mark = true;
                return Err(HandlerError::ShutdownInProgress);
            }
            let before = Instant::now();
            let result = {
                let _renewal = self.state.renewal_lock();
                self.rt
                    .engine_call(Op::EffectStart, |conn: &mut PgConnection| {
                        let renewal = renew_on(conn, registry, &claim, Some(request))?;
                        mark_effect_started_on(conn, registry, &claim)?;
                        Ok(renewal)
                    })
            };
            match result {
                Ok(renewal) => {
                    self.store(before, &renewal);
                    self.set_phase(TrackedPhase::EffectStarted);
                    return self.after_effect_mark();
                }
                Err(ThothError::AsyncJobBusy) => {
                    bump(&self.rt.counters.busy);
                    unknown = true;
                    match delays.next() {
                        Some(delay) => self.retry_sleep(*delay),
                        None => {
                            self.set_phase(TrackedPhase::PossiblyStarted);
                            self.mark_shutdown_after_effect_mark();
                            return Err(HandlerError::Busy);
                        }
                    }
                }
                Err(error) if unknown && is_phase_refusal(&error) => {
                    // An earlier try committed T4 and the token is still
                    // ours: renew alone, so the lease covers the write from
                    // now.
                    self.set_phase(TrackedPhase::EffectStarted);
                    if self.shutdown_requested() {
                        self.state.status().shutdown_after_effect_mark = true;
                        return Err(HandlerError::ShutdownInProgress);
                    }
                    let before = Instant::now();
                    let renewed = {
                        let _renewal = self.state.renewal_lock();
                        self.rt
                            .engine_call(Op::ExtraRenew, |conn: &mut PgConnection| {
                                renew_on(conn, registry, &claim, Some(request))
                            })
                    };
                    return match renewed {
                        Ok(renewal) => {
                            self.store(before, &renewal);
                            self.after_effect_mark()?;
                            // `renew_on` caps the lease at the runtime
                            // deadline: after long retries the renewal can
                            // leave less than the window, as a post-effect
                            // `confirm_claim` checks too.
                            if self.remaining_runtime() < self.window() {
                                return Err(HandlerError::EffectWindowInsufficient);
                            }
                            Ok(())
                        }
                        Err(error) => Err(self.note_error(&error)),
                    };
                }
                Err(ThothError::AsyncJobStaleClaim) if unknown => {
                    self.set_phase(TrackedPhase::PossiblyStarted);
                    self.rt.mark_lost(self.state);
                    return Err(HandlerError::ClaimLost);
                }
                Err(ThothError::AsyncJobLeaseExpired) if unknown => {
                    self.set_phase(TrackedPhase::PossiblyStarted);
                    self.rt.mark_expired(self.state);
                    return Err(HandlerError::LeaseExpired);
                }
                Err(error) => {
                    // After an unknown try, only a proven answer moves the
                    // phase. `EFFECT_WINDOW_INSUFFICIENT` is one (the engine
                    // checks the phase first, so T4 never committed); any
                    // other error leaves the earlier try's outcome unknown.
                    if unknown && !matches!(error, ThothError::AsyncJobEffectWindowInsufficient) {
                        self.set_phase(TrackedPhase::PossiblyStarted);
                        self.mark_shutdown_after_effect_mark();
                    }
                    return Err(self.note_error(&error));
                }
            }
        }
    }

    /// After an effect mark that established `EFFECT_STARTED`: answer
    /// `ShutdownInProgress` if the stop signal is set (carry OPW4-1).
    fn after_effect_mark(&self) -> Result<(), HandlerError> {
        let stopped = self.stop_after_fence();
        if stopped.is_err() {
            self.state.status().shutdown_after_effect_mark = true;
        }
        stopped
    }

    /// Record that the stop signal came after an effect mark call, so a
    /// later `Release` refused with `PHASE` is a shutdown fallback.
    fn mark_shutdown_after_effect_mark(&self) {
        if self.shutdown_requested() {
            self.state.status().shutdown_after_effect_mark = true;
        }
    }

    /// `mark_effect_confirmed` (T5), retried on `BUSY` (spec section 6.5).
    fn mark_effect_confirmed(&self, provider_ref: Option<&str>) -> Result<(), HandlerError> {
        if self.claim_lost() {
            return Err(HandlerError::ClaimLost);
        }
        if self.phase() != TrackedPhase::EffectStarted {
            return Err(HandlerError::Phase);
        }
        let registry = self.registry();
        let claim = self.state.claim;
        let mut delays = self.rt.timings.busy_retries.iter();
        let mut unknown = false;
        loop {
            if unknown && self.shutdown_requested() {
                return Err(HandlerError::ShutdownInProgress);
            }
            let result = self
                .rt
                .engine_call(Op::EffectConfirm, |conn: &mut PgConnection| {
                    mark_effect_confirmed_on(conn, registry, &claim, provider_ref)
                });
            match result {
                Ok(_) => {
                    self.set_phase(TrackedPhase::EffectConfirmed);
                    return Ok(());
                }
                Err(ThothError::AsyncJobBusy) => {
                    bump(&self.rt.counters.busy);
                    unknown = true;
                    match delays.next() {
                        Some(delay) => self.retry_sleep(*delay),
                        None => return Err(HandlerError::Busy),
                    }
                }
                Err(error) if unknown && is_phase_refusal(&error) => {
                    // An earlier try committed T5. T5 admits no write, so no
                    // extra renewal.
                    self.set_phase(TrackedPhase::EffectConfirmed);
                    return Ok(());
                }
                Err(error) => return Err(self.note_error(&error)),
            }
        }
    }

    /// `confirm_claim` (T3) for an execution claim.
    fn confirm_execution_claim(&self) -> Result<(), HandlerError> {
        self.check_fence_flags()?;
        let phase = self.phase();
        if phase == TrackedPhase::PossiblyStarted {
            return Err(HandlerError::Phase);
        }
        let after_start = phase.is_after_start();
        let margin = Duration::from_secs(u64::try_from(EFFECT_MARGIN_SECONDS).unwrap_or(10));
        if after_start && self.remaining_runtime() < self.window() + margin {
            return Err(HandlerError::EffectWindowInsufficient);
        }
        let effect_request = after_start.then(|| effect_lease_request(self.state.policy));
        self.renew_fence(effect_request)?;
        self.stop_after_fence()?;
        if after_start && self.remaining_runtime() < self.window() {
            return Err(HandlerError::EffectWindowInsufficient);
        }
        Ok(())
    }

    /// `confirm_claim` (T3) for a reconciliation claim.
    fn confirm_reconciliation_claim(&self) -> Result<(), HandlerError> {
        self.check_fence_flags()?;
        self.renew_fence(None)?;
        self.stop_after_fence()
    }

    /// One `renew_on` under the renewal lock, with the claim's request (set
    /// to `Some(E)` first when `effect_request` is given; never set back).
    fn renew_fence(&self, effect_request: Option<i32>) -> Result<(), HandlerError> {
        let registry = self.registry();
        let claim = self.state.claim;
        let before = Instant::now();
        let result = {
            let mut request = self.state.renewal_lock();
            if let Some(effect) = effect_request {
                *request = Some(effect);
            }
            let lease_seconds = *request;
            self.rt
                .engine_call(Op::ConfirmClaim, |conn: &mut PgConnection| {
                    renew_on(conn, registry, &claim, lease_seconds)
                })
        };
        match result {
            Ok(renewal) => {
                self.store(before, &renewal);
                Ok(())
            }
            Err(error) => Err(self.note_error(&error)),
        }
    }

    fn pool(&self) -> &PgPool {
        &self.rt.pools().handler
    }
}

/// The context of one execution attempt.
pub struct ExecutionContext<'a> {
    handle: ClaimHandle<'a>,
}

impl<'a> ExecutionContext<'a> {
    pub(crate) fn new(rt: &'a Shared, state: &'a ClaimState) -> Self {
        ExecutionContext {
            handle: ClaimHandle { rt, state },
        }
    }

    pub fn shutdown_requested(&self) -> bool {
        self.handle.shutdown_requested()
    }

    pub fn claim_lost(&self) -> bool {
        self.handle.claim_lost()
    }

    pub fn lease_expired(&self) -> bool {
        self.handle.lease_expired()
    }

    /// True on shutdown, claim loss, lease expiry, or when the local
    /// deadline estimate is within `max(effect window,
    /// lease_default_seconds)`.
    pub fn should_yield(&self) -> bool {
        self.handle.should_yield()
    }

    /// From the local deadline estimate (never later than the truth).
    pub fn remaining_runtime(&self) -> Duration {
        self.handle.remaining_runtime()
    }

    pub fn read_target_state(&mut self) -> Result<Vec<TargetState>, HandlerError> {
        self.handle.read_target_state()
    }

    /// The fence before the first write of an attempt claimed at
    /// `PRE_WRITE`: the effect lease renewal and T4 in one transaction.
    pub fn mark_effect_started(&mut self) -> Result<(), HandlerError> {
        self.handle.mark_effect_started()
    }

    /// T5, with an optional provider reference.
    pub fn mark_effect_confirmed(
        &mut self,
        provider_ref: Option<&str>,
    ) -> Result<(), HandlerError> {
        self.handle.mark_effect_confirmed(provider_ref)
    }

    /// The fence before the first write of a resumed attempt and before
    /// every later write.
    pub fn confirm_claim(&mut self) -> Result<(), HandlerError> {
        self.handle.confirm_execution_claim()
    }

    /// The handler pool, never the engine pool.
    pub fn pool(&self) -> &PgPool {
        self.handle.pool()
    }

    /// The tracked phase (tests only).
    #[cfg(test)]
    pub(crate) fn tracked_phase(&self) -> TrackedPhase {
        self.handle.phase()
    }
}

/// The context of one reconciliation attempt.
pub struct ReconciliationContext<'a> {
    handle: ClaimHandle<'a>,
    settled: Option<DateTime<Utc>>,
}

impl<'a> ReconciliationContext<'a> {
    pub(crate) fn new(
        rt: &'a Shared,
        state: &'a ClaimState,
        settled: Option<DateTime<Utc>>,
    ) -> Self {
        ReconciliationContext {
            handle: ClaimHandle { rt, state },
            settled,
        }
    }

    pub fn shutdown_requested(&self) -> bool {
        self.handle.shutdown_requested()
    }

    pub fn claim_lost(&self) -> bool {
        self.handle.claim_lost()
    }

    pub fn lease_expired(&self) -> bool {
        self.handle.lease_expired()
    }

    pub fn should_yield(&self) -> bool {
        self.handle.should_yield()
    }

    pub fn remaining_runtime(&self) -> Duration {
        self.handle.remaining_runtime()
    }

    pub fn read_target_state(&mut self) -> Result<Vec<TargetState>, HandlerError> {
        self.handle.read_target_state()
    }

    pub fn confirm_claim(&mut self) -> Result<(), HandlerError> {
        self.handle.confirm_reconciliation_claim()
    }

    pub fn pool(&self) -> &PgPool {
        self.handle.pool()
    }

    /// The instant by which every write of the job's execution attempts is
    /// settled: the latest `finished_at` (the database clock for an open
    /// attempt) of the attempts that may have written, plus the effect
    /// window. `None` when no attempt reached effect start. A reconciler
    /// never concludes no effect before it.
    pub fn effect_settled_at(&self) -> Option<DateTime<Utc>> {
        self.settled
    }
}

// ---------------------------------------------------------------------------
// The handler registry (spec section 5.3)
// ---------------------------------------------------------------------------

/// The handlers this binary serves.
pub struct HandlerRegistry {
    handlers: &'static [&'static dyn JobHandler],
}

/// Empty in step 2: the worker ships inert.
pub static PRODUCTION_HANDLERS: HandlerRegistry = HandlerRegistry::new(&[]);

impl HandlerRegistry {
    pub const fn new(handlers: &'static [&'static dyn JobHandler]) -> Self {
        HandlerRegistry { handlers }
    }

    pub fn handlers(&self) -> &'static [&'static dyn JobHandler] {
        self.handlers
    }

    pub fn get(&self, kind: &str) -> Option<&'static dyn JobHandler> {
        self.handlers
            .iter()
            .copied()
            .find(|handler| handler.kind() == kind)
    }

    pub fn is_empty(&self) -> bool {
        self.handlers.is_empty()
    }

    /// Report every problem of spec section 5.3 against `registry`.
    pub fn validate(&self, registry: &AsyncJobRegistry) -> Result<(), Vec<String>> {
        let mut problems = Vec::new();
        let mut seen = HashSet::new();
        for handler in self.handlers {
            let kind = handler.kind();
            // Rule 1.
            if !seen.insert(kind) {
                problems.push(format!("handler {kind}: the kind is served twice"));
            }
            // Rule 2.
            let Some(policy) = registry.get(kind) else {
                problems.push(format!("handler {kind}: the kind is not in the registry"));
                continue;
            };
            // Rule 3.
            if policy.executor_domain != ExecutorDomain::ThothWorker
                && policy.reconciler_domain != ExecutorDomain::ThothWorker
            {
                problems.push(format!(
                    "handler {kind}: neither the executor nor the reconciler domain is ThothWorker"
                ));
            }
            // Rule 4.
            if policy.risk_class != RiskClass::Trusted {
                problems.push(format!("handler {kind}: the risk class is not Trusted"));
            }
            // Rule 5.
            let versions = handler.payload_versions();
            if versions.is_empty()
                || versions
                    .iter()
                    .any(|version| !policy.is_executable_version(*version))
            {
                problems.push(format!(
                    "handler {kind}: the payload versions are empty or not a subset of the \
                     kind's executable versions"
                ));
            }
        }
        for policy in registry.policies() {
            let served = policy.executor_domain == ExecutorDomain::ThothWorker
                || policy.reconciler_domain == ExecutorDomain::ThothWorker;
            // Rule 6.
            if served && policy.risk_class == RiskClass::Trusted && self.get(policy.kind).is_none()
            {
                problems.push(format!(
                    "kind {}: a Trusted ThothWorker kind has no handler",
                    policy.kind
                ));
            }
            // Rule 7.
            if policy.is_effectful()
                && policy.executor_domain == ExecutorDomain::ThothWorker
                && policy
                    .effect_window_seconds
                    .unwrap_or(0)
                    .saturating_add(EFFECT_MARGIN_SECONDS)
                    > policy.lease_max_seconds
            {
                problems.push(format!(
                    "kind {}: effect_window_seconds plus the {EFFECT_MARGIN_SECONDS} s margin \
                     exceeds lease_max_seconds",
                    policy.kind
                ));
            }
        }
        if problems.is_empty() {
            Ok(())
        } else {
            Err(problems)
        }
    }
}
