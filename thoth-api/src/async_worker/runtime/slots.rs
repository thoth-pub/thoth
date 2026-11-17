//! The slot threads (spec section 6.4): one claim at a time, the final
//! action, the action, fault and panic fallbacks, the unrun rule (6.3 step
//! 6) and the shutdown fallbacks (6.8 step 6).

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::Timestamptz;
use diesel::PgConnection;
use thoth_errors::{ThothError, ThothResult};

use super::{bump, lock, minimum_resume_delay, ClaimState, Op, Purpose, Shared, TrackedPhase};
use crate::async_worker::errors::{classify, error_code, is_phase_refusal, log_code, ErrorClass};
use crate::async_worker::handler::{
    ExecutionAction, ExecutionContext, JobView, ReconciliationAction, ReconciliationContext,
    WaitAction,
};
use crate::model::async_job::{
    complete_on, fail_on, get_job, job_attempts, map_db_error, release_on,
    resolve_reconciliation_on, wait_on, AsyncAttemptPurpose, AsyncEffectPhase, AsyncJobAttempt,
    AsyncJobKindPolicy, ClaimedJob, CompleteOutcome, FailRequest, FailureClass, ReconcilerOutcome,
    ResolveReconciliation, WaitOutcome, WaitRequest,
};

/// The code of a handler `Wait` whose waiting deadline is too close for any
/// valid `resume_at`; the runtime fails the attempt instead.
const WAIT_DEADLINE_TOO_CLOSE: &str = "WAIT_DEADLINE_TOO_CLOSE";

/// A claimed job handed to a slot thread.
pub(crate) struct Assignment {
    pub state: Arc<ClaimState>,
    pub job: ClaimedJob,
}

// ---------------------------------------------------------------------------
// The database clock and `resume_at` (spec section 5.1)
// ---------------------------------------------------------------------------

#[derive(QueryableByName)]
struct ClockRow {
    #[diesel(sql_type = Timestamptz)]
    now: DateTime<Utc>,
}

/// `SELECT clock_timestamp()`: no table.
pub(crate) fn db_clock(conn: &mut PgConnection) -> ThothResult<DateTime<Utc>> {
    sql_query("SELECT clock_timestamp() AS now")
        .get_result::<ClockRow>(conn)
        .map(|row| row.now)
        .map_err(map_db_error)
}

/// Where a `resume_at` computed at the database clock lands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ResumePlan {
    /// A valid `resume_at`.
    At(DateTime<Utc>),
    /// The waiting deadline has passed: `wait_on` takes T10d.
    DeadlinePassed,
    /// The deadline is ahead but closer than the minimum delay: no valid
    /// `resume_at` exists.
    TooClose,
}

/// `resume_at = clock + max(resume_after, poll_interval_min_seconds +
/// RESUME_MARGIN)`, capped at the waiting deadline.
pub(crate) fn plan_resume(
    clock: DateTime<Utc>,
    deadline: Option<DateTime<Utc>>,
    resume_after: Duration,
    policy: &AsyncJobKindPolicy,
) -> ResumePlan {
    let minimum = minimum_resume_delay(policy);
    let wanted = chrono::Duration::from_std(resume_after).unwrap_or(chrono::Duration::MAX);
    let at = clock
        .checked_add_signed(wanted.max(minimum))
        .unwrap_or(DateTime::<Utc>::MAX_UTC);
    match deadline {
        Some(deadline) if deadline <= clock => ResumePlan::DeadlinePassed,
        Some(deadline) if deadline - clock < minimum => ResumePlan::TooClose,
        Some(deadline) => ResumePlan::At(at.min(deadline)),
        None => ResumePlan::At(at),
    }
}

/// The deadline guard of spec section 6.3 step 6: the waiting deadline is
/// absent or later than the clock plus the minimum delay.
pub(crate) fn deadline_guard(
    clock: DateTime<Utc>,
    deadline: Option<DateTime<Utc>>,
    policy: &AsyncJobKindPolicy,
) -> bool {
    deadline.is_none_or(|deadline| deadline > clock + minimum_resume_delay(policy))
}

/// `effect_settled_at`: the latest `finished_at` (the clock for an open
/// attempt) of the execution attempts that may have written, plus the
/// window; `None` when no attempt reached effect start.
pub(crate) fn settled_at(
    attempts: &[AsyncJobAttempt],
    clock: DateTime<Utc>,
    policy: &AsyncJobKindPolicy,
) -> Option<DateTime<Utc>> {
    let window = chrono::Duration::seconds(i64::from(policy.effect_window_seconds.unwrap_or(0)));
    attempts
        .iter()
        .filter(|attempt| {
            attempt.purpose == AsyncAttemptPurpose::Execution
                && (attempt.effect_started_at.is_some()
                    || attempt.phase_at_start != AsyncEffectPhase::PreWrite)
        })
        .map(|attempt| attempt.finished_at.unwrap_or(clock))
        .max()
        .map(|finished| finished + window)
}

// ---------------------------------------------------------------------------
// Slot threads
// ---------------------------------------------------------------------------

pub(crate) fn start_slots(rt: &Arc<Shared>) {
    let slots = rt.config.slots;
    for slot in 0..slots {
        let (sender, receiver) = mpsc::channel::<Assignment>();
        lock(&rt.slot_senders).push(sender);
        rt.spawn("slot", move |rt| slot_loop(rt, slot, receiver));
    }
    rt.events
        .update(|events| events.idle_slots = (0..slots).rev().collect());
}

fn slot_loop(rt: &Arc<Shared>, slot: usize, receiver: mpsc::Receiver<Assignment>) {
    loop {
        if rt.exited() {
            return;
        }
        match receiver.recv_timeout(Duration::from_millis(50)) {
            Ok(assignment) => run_claim(rt, slot, assignment),
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => return,
        }
    }
}

fn log_outcome(state: &ClaimState, started: Instant, outcome: &str) {
    log::info!(
        "async worker: job {} kind {} v{} attempt {} purpose {} -> {outcome} in {} ms",
        state.job_id,
        state.kind(),
        state.payload_version,
        state.attempt_number,
        state.purpose.as_str(),
        started.elapsed().as_millis()
    );
}

fn mark_returned(state: &ClaimState) {
    let mut status = state.status();
    status.handler_returned = true;
    status.finishing = true;
}

/// Step 7: record whether the claim ended, drop it from the map and make the
/// slot idle.
fn finish_claim(rt: &Shared, state: &ClaimState, slot: usize, ended: bool) {
    {
        let mut status = state.status();
        status.ended = status.ended || ended;
        if !status.ended {
            drop(status);
            rt.note_unended();
        }
    }
    rt.remove_claim(state);
    rt.events.update(|events| events.idle_slots.push(slot));
}

/// Run one claim to its end (spec section 6.4).
fn run_claim(rt: &Arc<Shared>, slot: usize, assignment: Assignment) {
    let Assignment { state, job } = assignment;
    let started = Instant::now();
    let handler = rt.handlers.get(&job.kind);
    let view = JobView::from_claimed(&job);
    let ended = match state.purpose {
        Purpose::Execution => {
            let returned = match (handler, view.as_ref()) {
                (Some(handler), Some(view)) => {
                    let mut ctx = ExecutionContext::new(rt, &state);
                    catch_unwind(AssertUnwindSafe(|| handler.execute(view, &mut ctx))).ok()
                }
                _ => None,
            };
            mark_returned(&state);
            if rt.hard_stopped() {
                return;
            }
            match returned {
                Some(action) => finish_execution(rt, &state, action, started),
                None => panic_fallback(rt, &state, started),
            }
        }
        Purpose::Reconciliation => match effect_settled(rt, &state) {
            None => {
                mark_returned(&state);
                if rt.hard_stopped() {
                    return;
                }
                log::warn!(
                    "async worker: job {} the attempts could not be read; resolving \
                     STILL_REQUIRED without calling the reconciler",
                    state.job_id
                );
                still_required(rt, &state, started, "EFFECT_SETTLED_UNREADABLE")
            }
            Some(settled) => {
                let returned = match (handler, view.as_ref()) {
                    (Some(handler), Some(view)) => {
                        let mut ctx = ReconciliationContext::new(rt, &state, settled);
                        catch_unwind(AssertUnwindSafe(|| handler.reconcile(view, &mut ctx))).ok()
                    }
                    _ => None,
                };
                mark_returned(&state);
                if rt.hard_stopped() {
                    return;
                }
                match returned {
                    Some(action) => finish_reconciliation(rt, &state, action, started),
                    None => panic_fallback(rt, &state, started),
                }
            }
        },
    };
    finish_claim(rt, &state, slot, ended);
}

/// The reconciler's `effect_settled_at`, read before the handler runs;
/// `None` when the attempts cannot be read.
fn effect_settled(rt: &Shared, state: &ClaimState) -> Option<Option<DateTime<Utc>>> {
    let job_id = state.job_id;
    let (result, _) = rt.engine_call_with_retries(Op::Attempts, |conn| {
        let clock = db_clock(conn)?;
        let attempts = job_attempts(conn, job_id)?;
        Ok((clock, attempts))
    });
    match result {
        Ok((clock, attempts)) => Some(settled_at(&attempts, clock, state.policy)),
        Err(error) => {
            log::warn!(
                "async worker: job {} reading the attempts failed ({})",
                state.job_id,
                log_code(&error)
            );
            None
        }
    }
}

// ---------------------------------------------------------------------------
// Final actions
// ---------------------------------------------------------------------------

fn finish_execution(
    rt: &Shared,
    state: &ClaimState,
    action: ExecutionAction,
    started: Instant,
) -> bool {
    let registry = rt.registry;
    let claim = state.claim;
    let name = action.name();
    let (result, after_busy) = match &action {
        ExecutionAction::Complete(request) => rt.engine_call_with_retries(Op::Complete, |conn| {
            complete_on(conn, registry, &claim, request.clone()).map(|outcome| match outcome {
                CompleteOutcome::Succeeded(job) | CompleteOutcome::ReconciliationRequired(job) => {
                    job.status.as_str().to_string()
                }
            })
        }),
        ExecutionAction::Fail(request) => rt.engine_call_with_retries(Op::Fail, |conn| {
            fail_on(conn, registry, &claim, request.clone())
                .map(|job| job.status.as_str().to_string())
        }),
        ExecutionAction::Wait(wait) => {
            rt.engine_call_with_retries(Op::Wait, |conn| wait_in_transaction(conn, rt, state, wait))
        }
        ExecutionAction::Release => rt.engine_call_with_retries(Op::Release, |conn| {
            release_on(conn, registry, &claim).map(|job| job.status.as_str().to_string())
        }),
    };
    match result {
        Ok(outcome) => {
            let counters = &rt.counters;
            bump(match action {
                ExecutionAction::Complete(_) => &counters.successes,
                ExecutionAction::Fail(_) => &counters.failures,
                // A wait whose deadline was too close became a `fail_on`.
                ExecutionAction::Wait(_) if outcome.starts_with(WAIT_DEADLINE_TOO_CLOSE) => {
                    &counters.failures
                }
                ExecutionAction::Wait(_) => &counters.waits,
                ExecutionAction::Release => &counters.releases,
            });
            log_outcome(state, started, &outcome);
            true
        }
        Err(error) => final_error(rt, state, &error, after_busy, name, started),
    }
}

/// A handler `Wait` in one engine transaction: the database clock, the
/// waiting deadline, then `wait_on` (or `fail_on` when the deadline is too
/// close for any valid `resume_at`).
fn wait_in_transaction(
    conn: &mut PgConnection,
    rt: &Shared,
    state: &ClaimState,
    wait: &WaitAction,
) -> ThothResult<String> {
    let clock = db_clock(conn)?;
    let job = get_job(conn, state.job_id)?;
    let minimum = minimum_resume_delay(state.policy);
    let resume_at = match plan_resume(clock, job.waiting_deadline, wait.resume_after, state.policy)
    {
        ResumePlan::At(at) => at,
        // T10d: the engine checks the deadline before `resume_at`.
        ResumePlan::DeadlinePassed => clock + minimum,
        ResumePlan::TooClose => {
            return fail_on(
                conn,
                rt.registry,
                &state.claim,
                fallback_request(FailureClass::Indeterminate, WAIT_DEADLINE_TOO_CLOSE),
            )
            .map(|job| format!("{WAIT_DEADLINE_TOO_CLOSE} -> {}", job.status.as_str()));
        }
    };
    let request = WaitRequest {
        checkpoint: wait.checkpoint.clone(),
        resume_at,
        provider_ref: wait.provider_ref.clone(),
    };
    wait_on(conn, rt.registry, &state.claim, request).map(wait_outcome)
}

fn wait_outcome(outcome: WaitOutcome) -> String {
    match outcome {
        WaitOutcome::Waiting(job) => job.status.as_str().to_string(),
        WaitOutcome::DeadlineExceeded(job) => {
            format!("DEADLINE_EXCEEDED -> {}", job.status.as_str())
        }
    }
}

fn finish_reconciliation(
    rt: &Shared,
    state: &ClaimState,
    action: ReconciliationAction,
    started: Instant,
) -> bool {
    let registry = rt.registry;
    let claim = state.claim;
    let (result, after_busy) = rt.engine_call_with_retries(Op::Resolve, |conn| {
        if action.outcome == ReconcilerOutcome::Waiting {
            return resolve_waiting(conn, rt, state, &action);
        }
        let request = ResolveReconciliation {
            outcome: action.outcome,
            entries: action.entries.clone(),
            evidence: action.evidence.clone(),
            checkpoint: action.checkpoint.clone(),
            resume_at: None,
        };
        resolve_reconciliation_on(conn, registry, &claim, request)
            .map(|job| job.status.as_str().to_string())
    });
    match result {
        Ok(outcome) => {
            bump(&rt.counters.resolutions);
            log_outcome(state, started, &outcome);
            true
        }
        Err(error) => final_error(rt, state, &error, after_busy, "RESOLVE", started),
    }
}

/// A reconciler `Waiting` with the database-clock treatment (carry FBW4-1):
/// `resume_at` from `clock_timestamp()` in the same transaction, capped at
/// the waiting deadline. A deadline already passed or closer than the
/// minimum delay admits no `Waiting`: the runtime resolves `StillRequired`.
fn resolve_waiting(
    conn: &mut PgConnection,
    rt: &Shared,
    state: &ClaimState,
    action: &ReconciliationAction,
) -> ThothResult<String> {
    let clock = db_clock(conn)?;
    let job = get_job(conn, state.job_id)?;
    let resume_after = action.resume_after.unwrap_or(Duration::ZERO);
    let request = match plan_resume(clock, job.waiting_deadline, resume_after, state.policy) {
        ResumePlan::At(resume_at) => ResolveReconciliation {
            outcome: ReconcilerOutcome::Waiting,
            entries: action.entries.clone(),
            evidence: action.evidence.clone(),
            checkpoint: action.checkpoint.clone(),
            resume_at: Some(resume_at),
        },
        ResumePlan::DeadlinePassed | ResumePlan::TooClose => {
            return resolve_reconciliation_on(
                conn,
                rt.registry,
                &state.claim,
                still_required_request(),
            )
            .map(|job| format!("WAITING_DEADLINE -> {}", job.status.as_str()));
        }
    };
    resolve_reconciliation_on(conn, rt.registry, &state.claim, request)
        .map(|job| job.status.as_str().to_string())
}

fn still_required_request() -> ResolveReconciliation {
    ResolveReconciliation {
        outcome: ReconcilerOutcome::StillRequired,
        entries: None,
        evidence: None,
        checkpoint: None,
        resume_at: None,
    }
}

fn fallback_request(class: FailureClass, code: &str) -> FailRequest {
    FailRequest {
        class,
        error_code: code.to_string(),
        error_detail: None,
        no_effect_proven: false,
        evidence: None,
    }
}

/// Steps 4 to 6 for a final call that did not commit.
fn final_error(
    rt: &Shared,
    state: &ClaimState,
    error: &ThothError,
    after_busy: bool,
    action: &'static str,
    started: Instant,
) -> bool {
    match classify(error) {
        ErrorClass::Busy => {
            log::warn!(
                "async worker: job {} every try of the final call answered ASYNC_JOB_BUSY; \
                 lease-expiry recovery owns the job",
                state.job_id
            );
            log_outcome(state, started, error_code(error));
            false
        }
        ErrorClass::StaleClaim => {
            if after_busy {
                log::warn!(
                    "async worker: job {} a retried final call answered ASYNC_JOB_STALE_CLAIM; \
                     an earlier try may have committed and the durable state decides",
                    state.job_id
                );
            }
            log_outcome(state, started, error_code(error));
            true
        }
        ErrorClass::ActionInvalid => action_fallback(rt, state, error, action, started),
        ErrorClass::NotAuthorized => {
            rt.fatal("a final call");
            fault_fallback(rt, state, error, started)
        }
        _ => fault_fallback(rt, state, error, started),
    }
}

/// The action fallback (spec section 6.4 step 5), once, by tracked phase.
fn action_fallback(
    rt: &Shared,
    state: &ClaimState,
    error: &ThothError,
    action: &'static str,
    started: Instant,
) -> bool {
    log::warn!(
        "async worker: job {} the engine refused the handler's {action} ({}); applying the \
         action fallback",
        state.job_id,
        log_code(error)
    );
    let plan = match state.purpose {
        Purpose::Reconciliation => None,
        Purpose::Execution => {
            let status = state.snapshot();
            Some(if action == "RELEASE" && is_phase_refusal(error) {
                if rt.stopping() && status.shutdown_after_effect_mark {
                    // Carry OPW4-1: a T4 committed as the stop signal came.
                    (FailureClass::Indeterminate, "WORKER_SHUTDOWN")
                } else {
                    (FailureClass::Indeterminate, "HANDLER_ACTION_INVALID")
                }
            } else if status.phase == TrackedPhase::PreWrite {
                (FailureClass::Permanent, "HANDLER_ACTION_INVALID")
            } else {
                (FailureClass::Indeterminate, "HANDLER_ACTION_INVALID")
            })
        }
    };
    if matches!(plan, Some((_, "WORKER_SHUTDOWN"))) {
        bump(&rt.counters.shutdown_fallbacks);
    } else {
        bump(&rt.counters.action_fallbacks);
    }
    fallback(rt, state, plan, error_code(error), started)
}

/// The fault fallback: `WORKER_ENGINE_FAULT`, or `StillRequired`.
fn fault_fallback(rt: &Shared, state: &ClaimState, error: &ThothError, started: Instant) -> bool {
    log::warn!(
        "async worker: job {} the final call failed ({}); applying the fault fallback",
        state.job_id,
        log_code(error)
    );
    bump(&rt.counters.fault_fallbacks);
    let plan = match state.purpose {
        Purpose::Execution => Some((FailureClass::Indeterminate, "WORKER_ENGINE_FAULT")),
        Purpose::Reconciliation => None,
    };
    fallback(rt, state, plan, error_code(error), started)
}

/// A handler panic (D8), by tracked phase. No detail text.
fn panic_fallback(rt: &Shared, state: &ClaimState, started: Instant) -> bool {
    log::error!(
        "async worker: job {} kind {} the handler panicked (HANDLER_PANIC)",
        state.job_id,
        state.kind()
    );
    let plan = match state.purpose {
        Purpose::Reconciliation => None,
        Purpose::Execution => Some(if state.status().phase == TrackedPhase::PreWrite {
            (FailureClass::Permanent, "HANDLER_PANIC")
        } else {
            (FailureClass::Indeterminate, "HANDLER_PANIC")
        }),
    };
    bump(&rt.counters.failures);
    fallback(rt, state, plan, "HANDLER_PANIC", started)
}

/// `StillRequired` for a reconciliation the runtime resolves itself.
fn still_required(rt: &Shared, state: &ClaimState, started: Instant, why: &str) -> bool {
    fallback(rt, state, None, why, started)
}

/// One fallback call (`fail_on` with `plan`, or `StillRequired` when `plan`
/// is `None`), retried on `BUSY` like a final call. A refusal is logged with
/// the first code and the claim is left to lease expiry.
fn fallback(
    rt: &Shared,
    state: &ClaimState,
    plan: Option<(FailureClass, &'static str)>,
    first_code: &str,
    started: Instant,
) -> bool {
    let registry = rt.registry;
    let claim = state.claim;
    let (result, _) = rt.engine_call_with_retries(Op::Fallback, |conn| match plan {
        Some((class, code)) => fail_on(conn, registry, &claim, fallback_request(class, code))
            .map(|job| format!("{code} -> {}", job.status.as_str())),
        None => resolve_reconciliation_on(conn, registry, &claim, still_required_request())
            .map(|job| format!("{first_code} -> {}", job.status.as_str())),
    });
    match result {
        Ok(outcome) => {
            log_outcome(state, started, &outcome);
            true
        }
        Err(ThothError::AsyncJobStaleClaim) => {
            log_outcome(state, started, "ASYNC_JOB_STALE_CLAIM");
            true
        }
        Err(error) => {
            log::warn!(
                "async worker: job {} the fallback was refused too ({first_code}, then {}); \
                 lease-expiry recovery owns the job",
                state.job_id,
                log_code(&error)
            );
            log_outcome(state, started, error_code(&error));
            false
        }
    }
}

// ---------------------------------------------------------------------------
// The unrun rule (spec section 6.3 step 6)
// ---------------------------------------------------------------------------

/// What a guarded wait did.
enum Guarded {
    Waited(String),
    /// No stored checkpoint, or the deadline guard failed.
    NotWaited,
}

/// In one transaction: the database clock, the job's checkpoint and waiting
/// deadline, the deadline guard, then `wait_on` with the stored checkpoint
/// and `resume_after` zero.
fn guarded_wait(conn: &mut PgConnection, rt: &Shared, state: &ClaimState) -> ThothResult<Guarded> {
    let clock = db_clock(conn)?;
    let job = get_job(conn, state.job_id)?;
    let Some(checkpoint) = job.checkpoint else {
        return Ok(Guarded::NotWaited);
    };
    if !deadline_guard(clock, job.waiting_deadline, state.policy) {
        return Ok(Guarded::NotWaited);
    }
    let ResumePlan::At(resume_at) =
        plan_resume(clock, job.waiting_deadline, Duration::ZERO, state.policy)
    else {
        return Ok(Guarded::NotWaited);
    };
    let request = WaitRequest {
        checkpoint,
        resume_at,
        provider_ref: None,
    };
    wait_on(conn, rt.registry, &state.claim, request)
        .map(|outcome| Guarded::Waited(wait_outcome(outcome)))
}

/// The outcome of one runtime-initiated call.
enum Step {
    Done(String),
    /// The token is no longer the job's: the claim ended.
    Stale,
    /// Busy on every try, or another error.
    Failed(ThothError),
}

fn step(result: ThothResult<String>) -> Step {
    match result {
        Ok(outcome) => Step::Done(outcome),
        Err(ThothError::AsyncJobStaleClaim) => Step::Stale,
        Err(error) => Step::Failed(error),
    }
}

/// Runs one engine call (on a connection, answering a short outcome text)
/// with the caller's retries.
type WithRetries<'c> =
    dyn Fn(&mut dyn FnMut(&mut PgConnection) -> ThothResult<String>) -> ThothResult<String> + 'c;

/// By tracked phase: `release_on` at `PRE_WRITE` (then `fail_on`
/// `Indeterminate` `WORKER_SHUTDOWN` if refused with `PHASE`), otherwise
/// `fail_on` `Indeterminate` `WORKER_SHUTDOWN`; `StillRequired` for a
/// reconciliation. `call` runs one engine call with the caller's retries.
fn by_phase(rt: &Shared, state: &ClaimState, call: &WithRetries<'_>) -> Step {
    let registry = rt.registry;
    let claim = state.claim;
    let fail_shutdown = |conn: &mut PgConnection| {
        fail_on(
            conn,
            registry,
            &claim,
            fallback_request(FailureClass::Indeterminate, "WORKER_SHUTDOWN"),
        )
        .map(|job| format!("WORKER_SHUTDOWN -> {}", job.status.as_str()))
    };
    match state.purpose {
        Purpose::Reconciliation => step(call(&mut |conn| {
            resolve_reconciliation_on(conn, registry, &claim, still_required_request())
                .map(|job| job.status.as_str().to_string())
        })),
        Purpose::Execution => {
            if state.status().phase == TrackedPhase::PreWrite {
                match call(&mut |conn| {
                    release_on(conn, registry, &claim).map(|job| job.status.as_str().to_string())
                }) {
                    Err(error) if is_phase_refusal(&error) => step(call(&mut { fail_shutdown })),
                    other => step(other),
                }
            } else {
                step(call(&mut { fail_shutdown }))
            }
        }
    }
}

/// The unrun rule: the handler is never called.
pub(crate) fn apply_unrun(rt: &Shared, state: &ClaimState) {
    let started = Instant::now();
    mark_returned(state);
    bump(&rt.counters.unrun);
    // The unrun rule runs on the claim-loop thread: record loop progress
    // before and after each call (spec section 6.3 step 7).
    let with_retries = |f: &mut dyn FnMut(&mut PgConnection) -> ThothResult<String>| {
        rt.record_loop();
        let result = rt.engine_call_with_retries(Op::Unrun, f).0;
        rt.record_loop();
        result
    };
    let mut result = Step::Failed(ThothError::AsyncJobBusy);
    let mut decided = false;
    if state.purpose == Purpose::Execution {
        rt.record_loop();
        let (waited, _) =
            rt.engine_call_with_retries(Op::Unrun, |conn| guarded_wait(conn, rt, state));
        rt.record_loop();
        match waited {
            Ok(Guarded::Waited(outcome)) => {
                result = Step::Done(outcome);
                decided = true;
            }
            Ok(Guarded::NotWaited) => {}
            Err(ThothError::AsyncJobStaleClaim) => {
                result = Step::Stale;
                decided = true;
            }
            Err(ThothError::AsyncJobBusy) => decided = true,
            Err(_) => {}
        }
    }
    if !decided {
        result = by_phase(rt, state, &with_retries);
    }
    let ended = match result {
        Step::Done(outcome) => {
            log_outcome(state, started, &format!("UNRUN {outcome}"));
            true
        }
        Step::Stale => {
            log_outcome(state, started, "UNRUN ASYNC_JOB_STALE_CLAIM");
            true
        }
        Step::Failed(error) => {
            log::warn!(
                "async worker: job {} the unrun rule failed ({}); lease-expiry recovery owns the \
                 job",
                state.job_id,
                log_code(&error)
            );
            false
        }
    };
    state.status().ended = ended;
    if !ended {
        rt.note_unended();
    }
    rt.remove_claim(state);
}

// ---------------------------------------------------------------------------
// Shutdown fallbacks (spec section 6.8 step 6)
// ---------------------------------------------------------------------------

/// One engine call, with one retry after `fallback_retry` on `BUSY` if the
/// fallback phase leaves time for it.
fn fallback_try<T>(
    rt: &Shared,
    mut f: impl FnMut(&mut PgConnection) -> ThothResult<T>,
) -> ThothResult<T> {
    let first = rt.engine_call(Op::Fallback, &mut f);
    if !matches!(first, Err(ThothError::AsyncJobBusy)) {
        return first;
    }
    bump(&rt.counters.busy);
    let end = rt.events.read(|events| events.fallback_end);
    let retry = rt.timings.fallback_retry;
    if end.is_some_and(|end| Instant::now() + retry >= end) {
        return first;
    }
    if !rt.sleep_retry(retry) {
        return first;
    }
    rt.engine_call(Op::Fallback, f)
}

/// The fallback for a claim whose handler has not returned by the end of
/// the grace phase. Never takes the renewal lock.
pub(crate) fn shutdown_fallback(rt: &Shared, state: &ClaimState) {
    {
        let status = state.status();
        if status.ended || status.handler_returned {
            return;
        }
    }
    let started = Instant::now();
    bump(&rt.counters.shutdown_fallbacks);
    let single = |f: &mut dyn FnMut(&mut PgConnection) -> ThothResult<String>| fallback_try(rt, f);
    let phase = state.status().phase;
    let mut result = None;
    if state.purpose == Purpose::Execution
        && phase == TrackedPhase::EffectConfirmed
        && state.policy.confirmed_effect_resumable
    {
        match fallback_try(rt, |conn| guarded_wait(conn, rt, state)) {
            Ok(Guarded::Waited(outcome)) => result = Some(Step::Done(outcome)),
            Ok(Guarded::NotWaited) => {}
            Err(ThothError::AsyncJobStaleClaim) => result = Some(Step::Stale),
            Err(_) => {}
        }
    }
    let result = match result {
        Some(result) => result,
        None => by_phase(rt, state, &single),
    };
    let ended = match result {
        Step::Done(outcome) => {
            log_outcome(state, started, &format!("SHUTDOWN_FALLBACK {outcome}"));
            true
        }
        Step::Stale => {
            log_outcome(state, started, "SHUTDOWN_FALLBACK ASYNC_JOB_STALE_CLAIM");
            true
        }
        Step::Failed(error) => {
            log::warn!(
                "async worker: job {} the shutdown fallback failed ({}); recovered at lease \
                 expiry",
                state.job_id,
                log_code(&error)
            );
            false
        }
    };
    if ended {
        state.status().ended = true;
    } else {
        rt.note_unended();
    }
}
