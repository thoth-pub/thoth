//! The heartbeat (spec section 6.6): renews due claims, soonest lease
//! expiry first, on its own one-connection pool.

use std::sync::{Arc, TryLockError};
use std::time::Instant;

use chrono::{DateTime, Utc};
use thoth_errors::ThothError;

use super::{bump, ClaimState, Shared, Timings};
use crate::async_worker::errors::log_code;
use crate::model::async_job::{renew_on, with_engine_transaction_on};

#[cfg(test)]
use super::Op;

/// The claims due for renewal at `now`, soonest lease expiry first: neither
/// lost nor expired nor ended, and last renewed (or claimed) at least
/// [`Timings::renew_every`] ago.
pub(crate) fn due_claims(
    claims: Vec<Arc<ClaimState>>,
    now: Instant,
    timings: &Timings,
) -> Vec<Arc<ClaimState>> {
    let mut due: Vec<(DateTime<Utc>, Arc<ClaimState>)> = claims
        .into_iter()
        .filter_map(|state| {
            let (skip, expiry, last) = {
                let status = state.status();
                (
                    status.claim_lost || status.lease_expired || status.ended,
                    status.lease_expires_at,
                    status.last_renewal,
                )
            };
            if skip || now.saturating_duration_since(last) < timings.renew_every(state.policy) {
                None
            } else {
                Some((expiry, state))
            }
        })
        .collect();
    due.sort_by_key(|(expiry, _)| *expiry);
    due.into_iter().map(|(_, state)| state).collect()
}

pub(crate) fn heartbeat_loop(rt: &Arc<Shared>) {
    loop {
        if rt.exited() {
            return;
        }
        rt.record_heartbeat();
        let mut healthy = true;
        for state in due_claims(rt.claims_snapshot(), Instant::now(), &rt.timings) {
            if rt.exited() {
                return;
            }
            if !renew(rt, &state) {
                healthy = false;
            }
        }
        if healthy {
            rt.record_renewal_health();
        }
        rt.events
            .wait_until(Some(Instant::now() + rt.timings.heartbeat_wake), |events| {
                events.exited || events.hard_stop
            });
    }
}

/// Renew one claim under its renewal lock, skipped (and not due) when the
/// context holds the lock. False when the renewal was attempted and failed.
fn renew(rt: &Shared, state: &ClaimState) -> bool {
    let guard = match state.renewal.try_lock() {
        Ok(guard) => guard,
        Err(TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
        Err(TryLockError::WouldBlock) => return true,
    };
    let request = *guard;
    #[cfg(test)]
    rt.hooks.pause_heartbeat_renewal();
    let before = Instant::now();
    let checkout = rt.pools().heartbeat.get();
    rt.record_heartbeat();
    let result = match checkout {
        Err(_) => Err(ThothError::AsyncJobBusy),
        Ok(mut conn) => {
            let registry = rt.registry;
            let claim = state.claim;
            let call = |conn: &mut diesel::PgConnection| {
                with_engine_transaction_on(conn, |c| renew_on(c, registry, &claim, request))
            };
            #[cfg(test)]
            let result = match rt.hooks.take_fault(Op::Renew) {
                Some(fault) => fault.apply(|| call(&mut conn)),
                None => call(&mut conn),
            };
            #[cfg(not(test))]
            let result = call(&mut conn);
            result
        }
    };
    drop(guard);
    rt.record_heartbeat();
    match result {
        Ok(renewal) => {
            rt.store_renewal(state, before, renewal.lease_expires_at);
            true
        }
        Err(ThothError::AsyncJobStaleClaim) => {
            // A finishing claim's final call may just have committed.
            if !state.status().finishing {
                rt.mark_lost(state);
            }
            false
        }
        Err(ThothError::AsyncJobLeaseExpired) => {
            rt.mark_expired(state);
            false
        }
        Err(ThothError::AsyncJobBusy) => {
            bump(&rt.counters.busy);
            false
        }
        Err(error) => {
            log::warn!(
                "async worker: job {} renewal failed ({}); retrying on the next wake",
                state.job_id,
                log_code(&error)
            );
            false
        }
    }
}
