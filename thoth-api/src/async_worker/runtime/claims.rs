//! The claim sets (spec section 5.4) and the claim loop (spec section 6.3).

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use rand::Rng;

use super::slots::{apply_unrun, Assignment};
use super::{
    bump, lock, ClaimState, ClaimStatus, Op, Purpose, Shared, TrackedPhase, CLAIM_BATCH_MAX,
    RECOVER_LIMIT,
};
use crate::async_worker::errors::{classify, log_code, ErrorClass};
use crate::async_worker::handler::HandlerRegistry;
use crate::model::async_job::{
    claim_on, claim_reconciliation_on, recover_on, AsyncJobRegistry, ClaimRequest, ClaimedJob,
    Executor, ExecutorDomain, RiskClass,
};

// ---------------------------------------------------------------------------
// The claim sets
// ---------------------------------------------------------------------------

/// The kinds this task claims, each with the handler's payload versions,
/// sorted by kind.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClaimSets {
    pub execution: Vec<(String, Vec<i16>)>,
    pub reconciliation: Vec<(String, Vec<i16>)>,
}

impl ClaimSets {
    pub fn is_empty(&self) -> bool {
        self.execution.is_empty() && self.reconciliation.is_empty()
    }

    pub fn for_purpose(&self, purpose: Purpose) -> &[(String, Vec<i16>)] {
        match purpose {
            Purpose::Execution => &self.execution,
            Purpose::Reconciliation => &self.reconciliation,
        }
    }

    /// The kinds of one set, comma separated, for the startup line.
    pub fn describe(&self, purpose: Purpose) -> String {
        self.for_purpose(purpose)
            .iter()
            .map(|(kind, _)| kind.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Every kind of either set, sorted, once: what `recover_on` names.
    pub fn union_kinds(&self) -> Vec<String> {
        let mut kinds: Vec<String> = self
            .execution
            .iter()
            .chain(self.reconciliation.iter())
            .map(|(kind, _)| kind.clone())
            .collect();
        kinds.sort();
        kinds.dedup();
        kinds
    }
}

/// Compute both claim sets from `--kinds`, the registry, the handlers and
/// the pool profile. A listed kind that is unknown, has no handler or fails
/// the profile is left out with one note naming the kind and the reason.
pub fn compute_claim_sets(
    kinds: &[String],
    registry: &AsyncJobRegistry,
    handlers: &HandlerRegistry,
    executor: &Executor,
) -> (ClaimSets, Vec<String>) {
    let mut execution = BTreeMap::new();
    let mut reconciliation = BTreeMap::new();
    let mut notes = Vec::new();
    let mut listed: Vec<&String> = kinds.iter().collect();
    listed.sort();
    listed.dedup();
    for kind in listed {
        let leave_out = |reason: &str| format!("kind {kind} left out of the claim sets: {reason}");
        let Some(policy) = registry.get(kind) else {
            notes.push(leave_out("unknown to this binary's registry"));
            continue;
        };
        let Some(handler) = handlers.get(kind) else {
            notes.push(leave_out("no handler in this binary"));
            continue;
        };
        let executes = policy.executor_domain == executor.domain;
        let reconciles = policy.reconciler_domain == executor.domain;
        if !executes && !reconciles {
            notes.push(leave_out("served by another executor domain"));
            continue;
        }
        if policy.risk_class != executor.risk_class || policy.risk_class != RiskClass::Trusted {
            notes.push(leave_out("its risk class is not Trusted"));
            continue;
        }
        if policy
            .required_capabilities
            .iter()
            .any(|needed| !executor.capabilities.iter().any(|have| have == needed))
        {
            notes.push(leave_out("a required capability is not provided"));
            continue;
        }
        if !executor.revision_modes.contains(&policy.revision_mode) {
            notes.push(leave_out("its revision mode is not served"));
            continue;
        }
        let mut versions: Vec<i16> = handler
            .payload_versions()
            .iter()
            .copied()
            .filter(|version| policy.is_executable_version(*version))
            .collect();
        versions.sort_unstable();
        versions.dedup();
        if versions.is_empty() {
            notes.push(leave_out("no executable payload version"));
            continue;
        }
        if executes && executor.domain == ExecutorDomain::ThothWorker {
            execution.insert(kind.clone(), versions.clone());
        }
        if reconciles && executor.domain == ExecutorDomain::ThothWorker {
            reconciliation.insert(kind.clone(), versions);
        }
    }
    (
        ClaimSets {
            execution: execution.into_iter().collect(),
            reconciliation: reconciliation.into_iter().collect(),
        },
        notes,
    )
}

// ---------------------------------------------------------------------------
// The loops
// ---------------------------------------------------------------------------

/// Idle mode (spec section 6.2 step 6): record progress every poll interval
/// and make no database call.
pub(crate) fn idle_loop(rt: &Arc<Shared>) {
    let _exit = LoopExit(rt);
    loop {
        rt.record_loop();
        if rt.stopping() || rt.hard_stopped() {
            break;
        }
        rt.sleep_unless_stopped(rt.timings.poll);
    }
}

/// Sets `loop_exited` when the loop ends, by return or by panic, so the
/// shutdown never waits the join bound for a loop that is gone.
struct LoopExit<'a>(&'a Shared);

impl Drop for LoopExit<'_> {
    fn drop(&mut self) {
        self.0.events.update(|state| state.loop_exited = true);
    }
}

/// The claim loop (spec section 6.3).
pub(crate) fn claim_loop(rt: &Arc<Shared>) {
    let _exit = LoopExit(rt);
    let mut first = Purpose::Execution;
    let mut backoff: Option<Duration> = None;
    let mut last_recover: Option<Instant> = None;
    let mut stall_logged = false;
    loop {
        // Step 1.
        rt.record_loop();
        #[cfg(test)]
        if rt.hooks.take_claim_loop_panic() {
            panic!("injected claim-loop panic");
        }
        // Step 2.
        if rt.stopping() || rt.hard_stopped() {
            break;
        }
        // Step 3.
        if stalled(rt) {
            if !stall_logged {
                log::warn!(
                    "async worker: claiming nothing while the heartbeat is not cycling or \
                     renewals are failing (HEARTBEAT_STALLED)"
                );
                stall_logged = true;
            }
            bump(&rt.counters.stalled_ticks);
            rt.sleep_unless_stopped(rt.timings.poll);
            continue;
        }
        stall_logged = false;

        // Step 4.
        if rt.idle_slot_count() == 0 {
            if last_recover.is_none_or(|at| at.elapsed() >= rt.timings.poll) {
                recover(rt);
                last_recover = Some(Instant::now());
            }
            rt.record_loop();
            let deadline = Instant::now() + rt.timings.poll;
            rt.events.wait_until(Some(deadline), |state| {
                state.stop || state.hard_stop || !state.idle_slots.is_empty()
            });
            continue;
        }

        // Step 5.
        let second = match first {
            Purpose::Execution => Purpose::Reconciliation,
            Purpose::Reconciliation => Purpose::Execution,
        };
        let order = [first, second];
        first = second;
        let mut full_batch = false;
        let mut failure = None;
        for purpose in order {
            // No claim call starts after the stop signal.
            if rt.stopping() || rt.hard_stopped() {
                break;
            }
            let set = rt.sets().for_purpose(purpose);
            if set.is_empty() {
                continue;
            }
            let free = rt.idle_slot_count();
            if free == 0 {
                continue;
            }
            let asked = free.min(CLAIM_BATCH_MAX);
            match claim(rt, purpose, set.to_vec(), asked) {
                Ok(count) => {
                    backoff = None;
                    if count == asked {
                        full_batch = true;
                    }
                }
                Err(error) => {
                    failure = Some(error);
                    break;
                }
            }
        }
        rt.record_loop();

        if let Some(error) = failure {
            match classify(&error) {
                ErrorClass::NotAuthorized => {
                    rt.fatal("a claim call");
                    break;
                }
                ErrorClass::Busy => {
                    bump(&rt.counters.busy);
                    log::warn!(
                        "async worker: a claim call answered ASYNC_JOB_BUSY; backing off (jobs a \
                         lost commit claimed are recovered at lease expiry)"
                    );
                }
                _ => log::warn!(
                    "async worker: a claim call failed ({}); backing off",
                    log_code(&error)
                ),
            }
            let delay = backoff.map_or(rt.timings.poll, |previous| {
                (previous * 2).min(rt.timings.busy_backoff_cap)
            });
            backoff = Some(delay);
            rt.sleep_unless_stopped(delay);
            rt.record_loop();
            continue;
        }

        // Step 7.
        if full_batch && rt.idle_slot_count() > 0 {
            continue;
        }
        let delay = jittered(rt.timings.poll, rt.timings.jitter);
        rt.sleep_unless_stopped(delay);
        rt.record_loop();
    }
}

/// Step 3: the heartbeat thread is not cycling, or claims are in flight and
/// renewal health is stale.
fn stalled(rt: &Shared) -> bool {
    let limit = rt.timings.heartbeat_stall;
    if rt.heartbeat_age().is_some_and(|age| age > limit) {
        return true;
    }
    !rt.in_flight().is_empty() && rt.renewal_health_age().is_some_and(|age| age > limit)
}

/// The poll interval with plus or minus 20 percent jitter.
pub(crate) fn jittered(poll: Duration, jitter: bool) -> Duration {
    if !jitter {
        return poll;
    }
    let factor: f64 = rand::rng().random_range(0.8..=1.2);
    poll.mul_f64(factor)
}

/// Step 4: recover a crashed peer's jobs of the claim-set kinds while no
/// slot is free.
fn recover(rt: &Shared) {
    let kinds = rt.sets().union_kinds();
    let registry = rt.registry;
    rt.record_loop();
    let result = rt.engine_call(Op::Recover, |conn| {
        recover_on(conn, registry, Some(&kinds), RECOVER_LIMIT)
    });
    rt.record_loop();
    match result {
        Ok(report) if !report.recovered.is_empty() || !report.failed.is_empty() => log::info!(
            "async worker: recovery with no slot free: {} recovered, {} deferred, {} failed",
            report.recovered.len(),
            report.deferred.len(),
            report.failed.len()
        ),
        Ok(_) => {}
        Err(error) => {
            if classify(&error) == ErrorClass::NotAuthorized {
                rt.fatal("a recovery call");
            } else {
                log::warn!(
                    "async worker: a recovery call failed ({})",
                    log_code(&error)
                );
            }
        }
    }
}

/// One claim call for `purpose`; dispatches every claimed job. Returns how
/// many jobs it claimed.
fn claim(
    rt: &Arc<Shared>,
    purpose: Purpose,
    kinds: Vec<(String, Vec<i16>)>,
    asked: usize,
) -> Result<usize, thoth_errors::ThothError> {
    let registry = rt.registry;
    let executor = rt.executor.clone();
    let request = ClaimRequest {
        kinds,
        max_jobs: i32::try_from(asked).unwrap_or(1),
        lease_seconds: None,
    };
    let before = Instant::now();
    rt.record_loop();
    let result = match purpose {
        Purpose::Execution => rt.engine_call(Op::Claim, |conn| {
            claim_on(conn, registry, &executor, request)
        }),
        Purpose::Reconciliation => rt.engine_call(Op::ClaimReconciliation, |conn| {
            claim_reconciliation_on(conn, registry, &executor, request)
        }),
    };
    rt.record_loop();
    let result = result?;
    rt.record_claim_success();
    let count = result.jobs.len();
    for job in result.jobs {
        dispatch(rt, purpose, job, before);
    }
    Ok(count)
}

/// Step 6: track a claimed job and hand it to an idle slot, or apply the
/// unrun rule.
fn dispatch(rt: &Arc<Shared>, purpose: Purpose, job: ClaimedJob, before: Instant) {
    bump(&rt.counters.claims);
    let Some(policy) = rt.registry.get(&job.kind) else {
        // The engine returns only requested kinds; this cannot happen.
        log::error!(
            "async worker: job {} kind {} is not in the registry; left to lease expiry",
            job.async_job_id,
            job.kind
        );
        return;
    };
    let runtime =
        Duration::from_secs(u64::try_from(policy.max_attempt_runtime_seconds).unwrap_or(0));
    let state = Arc::new(ClaimState {
        claim: job.worker_claim(ExecutorDomain::ThothWorker),
        job_id: job.async_job_id,
        policy,
        purpose,
        payload_version: job.payload_version,
        attempt_number: job.attempt_number,
        renewal: Mutex::new(None),
        status: Mutex::new(ClaimStatus {
            phase: TrackedPhase::from_phase(job.effect_phase),
            claim_lost: false,
            lease_expired: false,
            lost_at: None,
            finishing: false,
            handler_returned: false,
            ended: false,
            last_renewal: before,
            lease_expires_at: job.lease_expires_at,
            deadline_estimate: before + runtime,
            shutdown_after_effect_mark: false,
            slot: None,
        }),
    });
    let live_post_effect = {
        let mut in_flight = rt.in_flight();
        let live = in_flight.values().any(|other| {
            if other.job_id != job.async_job_id {
                return false;
            }
            let status = other.status();
            !status.handler_returned && status.phase.is_after_start()
        });
        in_flight.insert(state.claim.claim_token, Arc::clone(&state));
        live
    };
    #[cfg(test)]
    rt.hooks.pause_after_claim_commit();
    if live_post_effect {
        log::warn!(
            "async worker: job {} kind {} purpose {} is still running a post-effect handler in \
             this process; the new claim is not run",
            job.async_job_id,
            job.kind,
            purpose.as_str()
        );
        apply_unrun(rt, &state);
        return;
    }
    if rt.stopping() {
        apply_unrun(rt, &state);
        return;
    }
    let slot = rt.events.update(|events| events.idle_slots.pop());
    let Some(slot) = slot else {
        apply_unrun(rt, &state);
        return;
    };
    state.status().slot = Some(slot);
    let sender = lock(&rt.slot_senders).get(slot).cloned();
    let assignment = Assignment {
        state: Arc::clone(&state),
        job,
    };
    let sent = sender.is_some_and(|sender| sender.send(assignment).is_ok());
    if !sent {
        state.status().slot = None;
        apply_unrun(rt, &state);
    }
}
