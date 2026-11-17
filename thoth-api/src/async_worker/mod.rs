//! The trusted in-process `thoth-worker` runtime (THOTH-ASYNC-01-WORKER-01,
//! ADR-0012, ADR-0015): `thoth start worker` claims `ThothWorker` jobs of the
//! kinds it lists and runs their handlers; `thoth worker health` reads the
//! liveness file.
//!
//! **It ships inert.** [`PRODUCTION_HANDLERS`] and the engine's
//! `PRODUCTION_REGISTRY` are empty, and an empty `--kinds`
//! (`ASYNC_WORKER_KINDS`) is idle: no pool and no database connection are
//! built, and the worker only keeps its liveness file fresh.
//!
//! # The handler contract
//!
//! A handler implements [`JobHandler`] for one kind registered with
//! `executor_domain` or `reconciler_domain` `ThothWorker` and risk class
//! `Trusted`. It receives a [`JobView`] (every claimed field except the claim
//! token) and a context. The context holds the token: every token-fenced
//! engine call goes through it, each in its own engine transaction on the
//! engine pool.
//!
//! **Token confinement is by API shape, not a security boundary.** The
//! handler pool ([`ExecutionContext::pool`]) connects with the primary
//! credential. Rule for every handler, checked in review: handler code never
//! calls an engine `_on` function and never queries an `async_*` table.
//!
//! A handler returns exactly one final action. `FailRequest.error_code` must
//! match the engine's code format, and `error_detail` must not contain
//! payload data, credentials or environment values.
//!
//! # The effect rules (binding on every handler)
//!
//! 1. **Fences.** A provider write (an external or domain effect the
//!    reconciler must account for) starts only immediately after a fence
//!    call returned `Ok`: [`ExecutionContext::mark_effect_started`] for the
//!    first write of an attempt claimed at `PRE_WRITE`;
//!    [`ExecutionContext::confirm_claim`] for the first write of a resumed
//!    attempt (claimed at `EFFECT_STARTED` or `EFFECT_CONFIRMED`) and for
//!    every later write in any attempt, including writes after T5. After a
//!    release, a recovery, a lost or expired lease, or the stop signal,
//!    every fence refuses, so no write starts.
//! 2. **The window.** The kind's effect window covers the time from a
//!    write's fence to the write being settled (its outcome observable by
//!    the reconciler). A handler whose write can take longer splits it, with
//!    a fence before each part.
//! 3. On [`HandlerError::EffectWindowInsufficient`] the handler starts no
//!    write: before effect start it returns `Release`, and the next claim
//!    starts a fresh attempt with a full deadline; after effect start it
//!    returns `Wait` with a safe checkpoint, or `Fail` `Indeterminate`.
//!    From `mark_effect_started` the answer normally means T4 did not
//!    commit, and `Release` is right. In one case T4 had committed (an
//!    earlier try answered busy, and the renewal after it left less than
//!    the window before the deadline): the engine then refuses the
//!    `Release`, and the runtime fails the attempt `Indeterminate` with code
//!    `HANDLER_ACTION_INVALID`, which the engine resolves to reconciliation
//!    by the durable phase. The job's outcome is right; only that code
//!    overstates the handler's part.
//! 4. On [`HandlerError::Busy`] (or any refusal) from a fence, the handler
//!    must not start the write. For T4 the runtime has already set the
//!    tracked phase to "possibly started" if the outcome is unknown, and the
//!    handler returns `Fail` `Indeterminate`, which the engine resolves by
//!    the durable phase.
//!
//! The effect lease request is `E = max(lease_default_seconds,
//! effect_window_seconds + EFFECT_MARGIN)`. `mark_effect_started` runs
//! `renew_on(Some(E))` and T4 in one transaction under the claim's renewal
//! lock, so from T4 on the lease covers the effect start plus the window.
//!
//! # `Wait` and `resume_at`
//!
//! [`WaitAction::resume_after`] is relative. The runtime reads
//! `clock_timestamp()` in the same engine transaction as `wait_on` and sends
//! `resume_at = clock + max(resume_after, poll_interval_min_seconds +
//! RESUME_MARGIN)`, capped at the job's waiting deadline. A deadline already
//! passed takes the engine's T10d; one closer than the minimum delay ends the
//! attempt with `fail_on` `Indeterminate` `WAIT_DEADLINE_TOO_CLOSE`.
//!
//! The waiting deadline is set by the job's first wait to the kind's
//! `waiting_horizon_seconds` from then, and the engine never clears it.
//! **On a first wait there is no deadline to cap at, so a `resume_after`
//! beyond the kind's waiting horizon is a handler fault**: the engine
//! refuses the `resume_at` and the runtime applies the action fallback
//! (carry FBW4-2).
//!
//! A reconciler's `Waiting` ([`ReconciliationAction::waiting`]) gets the same
//! database-clock treatment (carry FBW4-1): a relative delay, `resume_at`
//! from `clock_timestamp()` in the same transaction, capped at the waiting
//! deadline. When the deadline has passed or is closer than the minimum
//! delay, the runtime resolves `StillRequired` instead.
//!
//! # The reconciler rule
//!
//! A reconciler never concludes no effect (`Failed` or `Cancelled` on
//! not-applied evidence) before [`ReconciliationContext::effect_settled_at`];
//! until then it returns `StillRequired` or `Waiting`. A shutdown or panic
//! fallback makes a job `RECONCILIATION_REQUIRED` at once, so a reconciler in
//! another task can claim it while the original effect may still be landing.
//! The in-process reconciler cannot accept a regression.
//!
//! # Operator notes
//!
//! - **Keep a kind listed until its jobs have drained**, unless the point is
//!   to stop it: removing a kind from `--kinds` stops its claims, its
//!   recovery and its reconciliation by this task, and a job of that kind
//!   killed past T4 stays `RUNNING` with an expired lease until a task that
//!   lists the kind runs again.
//! - Worker AWS authority comes from the ECS task role only. The worker
//!   refuses to start when `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`,
//!   `AWS_SESSION_TOKEN`, `AWS_PROFILE`, `AWS_SHARED_CREDENTIALS_FILE` or
//!   `AWS_CONFIG_FILE` is set and non-empty. **The binary loads `.env`
//!   first, so a developer `.env` holding AWS keys for the API stops a local
//!   worker.** The check cannot see the default profile files; worker AWS
//!   clients must use the ECS container credentials provider only.
//! - Tasks of one service share `--worker-name`, and so share their
//!   `async_executor_poll` rows.
//! - The worker never runs migrations: it waits while any is pending.
//! - The task's `stopTimeout` must be at least `max(grace,
//!   CLAIM_LOOP_JOIN_BOUND) + FALLBACK_PHASE_BOUND + 10` seconds (55 with
//!   the defaults).
//! - `DATABASE_URL` should carry libpq `connect_timeout` and keepalive or
//!   `tcp_user_timeout` parameters: a half-open connection is bounded only
//!   by TCP.
//!
//! # Fixed constants
//!
//! ```text
//! LIVENESS_CADENCE            5 s     ticker period (health max age >= 3 x this)
//! ENGINE_STATEMENT_TIMEOUT    15 s    statement_timeout of engine and heartbeat connections
//! ENGINE_POOL_CHECKOUT        5 s     checkout timeout of the engine and heartbeat pools
//! HEARTBEAT_STALL             30 s    heartbeat not cycling, or renewal health stale
//! LOOP_STALL                  120 s   claim-loop progress older than this stops liveness
//! HUNG_SLOT_LIMIT             300 s   a slot hung longer than this stops liveness
//! FALLBACK_PHASE_BOUND        20 s    the whole post-grace fallback phase
//! CLAIM_LOOP_JOIN_BOUND       25 s    how long shutdown waits for a claim call
//! EFFECT_MARGIN               10 s    the effect lease's margin above the window
//! RESUME_MARGIN               10 s    added to poll_interval_min_seconds in every resume_at
//! ```
//!
//! # Logging
//!
//! Worker log lines start `async worker:` and never contain a payload,
//! checkpoint, evidence, error detail, provider reference, claim token,
//! target key, database URL, environment variable value, driver text or
//! panic message. The worker never reads the whole environment.

pub mod config;
pub mod errors;
pub mod handler;
pub mod health;
pub mod runtime;

#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;

pub use config::{WorkerArgs, WorkerConfig};
pub use handler::{
    ExecutionAction, ExecutionContext, HandlerError, HandlerRegistry, JobHandler, JobView,
    ReconciliationAction, ReconciliationContext, WaitAction, PRODUCTION_HANDLERS,
};
pub use health::health_command;
pub use runtime::{
    launch, panic_line, WorkerHandle, WorkerProcess, CLAIM_LOOP_JOIN_BOUND, EFFECT_MARGIN_SECONDS,
    ENGINE_POOL_CHECKOUT, ENGINE_STATEMENT_TIMEOUT, FALLBACK_PHASE_BOUND, HEARTBEAT_STALL,
    HUNG_SLOT_LIMIT, LIVENESS_CADENCE, LOOP_STALL, RESUME_MARGIN_SECONDS,
};
