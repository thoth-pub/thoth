//! Durable Metrics rollup deltas and the MOM-1 work-day projection
//! (`MET-WP1-07`, `MET-WP4-01`).
//!
//! This module owns the persisted `metric_rollup_delta` model: the durable
//! accounting bridge between one canonical metric-record revision and the
//! rebuildable work-level rollup projections. The approved Metrics design
//! commits a canonical revision transactionally together with its delta —
//! applying a new record adds its value, a revision contributes `new - old`,
//! and a retraction subtracts the old value — so a delta is canonical
//! accounting evidence rather than a derived cache.
//!
//! `MET-WP1-07` established delta storage only. `MET-WP4-01` adds the MOM-1
//! application path on top of it: a gap-free work-day progress ordering, a
//! strict-frontier claim/lease protocol, whole-batch atomic application into
//! the one delivered projection `metric_rollup_work_day`, and a durable
//! contiguous watermark. See [`crud`] for the two protected operations and
//! their exact transaction shapes.
//!
//! What is still deliberately absent: any callable rebuild operation, any
//! retry/backoff or poison-skipping behaviour, any projection other than
//! `metric_rollup_work_day`, and any progress stream for a non-`DAY` grain.
//! A poison frontier blocks and waits for separately authorized repair rather
//! than being stepped over, and the monthly, work-country-month and
//! work-institution-month projections named by the approved design remain
//! future architecture.

use chrono::NaiveDate;
use uuid::Uuid;

use crate::model::Timestamp;

/// One persisted durable rollup delta row.
///
/// The database enforces `UNIQUE(revision_id)`, so at most one durable delta
/// exists per canonical revision and later rollup application cannot double
/// count a duplicated row. A composite foreign key over
/// `(record_id, revision_id)` against the MET-WP1-04
/// `metric_record_revision (record_id, record_revision_id)` unique key
/// guarantees the named revision genuinely belongs to the named record. That
/// foreign key is non-cascading, so deleting a canonical record or revision
/// that still has a delta fails rather than silently erasing accounting
/// evidence.
///
/// `delta_value` is a signed `i64` with deliberately **no** non-negative
/// rule: a revision contributes the signed difference `new - old` and a
/// retraction subtracts the previously applied value, so positive, zero and
/// negative values are all valid.
///
/// `status` is a plain `String`, not an enum. The approved design names the
/// field but defines no closed status vocabulary, transition model, claim
/// ownership, lease or recovery protocol, so this foundation deliberately
/// declares no closed runtime state machine — in the database or in Rust —
/// and offers no claim or apply method. `applied_at` is correspondingly a
/// plain nullable timestamp: nothing in this slice ties it to any particular
/// `status` value.
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricRollupDelta {
    pub delta_id: Uuid,
    pub record_id: Uuid,
    pub revision_id: Uuid,
    pub delta_value: i64,
    pub status: String,
    pub created_at: Timestamp,
    pub applied_at: Option<Timestamp>,
    pub work_day_sequence: Option<i64>,
    pub claim_token: Option<Uuid>,
    pub claimed_by: Option<String>,
    pub claimed_at: Option<Timestamp>,
    pub lease_expires_at: Option<Timestamp>,
}

/// One row of the single MOM-1 rollup projection.
///
/// Derived, rebuildable state. `value` is a signed `i64` because a revision
/// applies `new - old`, which is negative whenever a source corrects a figure
/// downwards, and a zero-valued row is retained rather than deleted so the
/// difference between "counted, and the answer is zero" and "never counted"
/// survives.
///
/// `watermark` is an **ordered work-day progress position**, not a wall-clock
/// time: it is the greatest `work_day_sequence` that has actually changed this
/// row. It never overrides the global serving boundary in
/// [`MetricRollupWorkDayState::applied_through_sequence`], which is the only
/// value a read path may treat as "everything up to here is applied".
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricRollupWorkDay {
    pub rollup_work_day_id: Uuid,
    pub work_id: Uuid,
    pub publication_id: Option<Uuid>,
    pub platform_id: Uuid,
    pub measure_id: Uuid,
    pub day: NaiveDate,
    pub country_code: Option<String>,
    pub institution_id: Option<Uuid>,
    pub value: i64,
    pub watermark: i64,
}

/// The singleton work-day progress and watermark row.
///
/// Exactly one row exists, with `state_id = 1`. It owns the next work-day
/// position to allocate and the contiguous applied-through frontier, and the
/// database enforces `applied_through_sequence < next_sequence` so the
/// frontier can never claim a position that was never allocated.
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricRollupWorkDayState {
    pub state_id: i16,
    pub next_sequence: i64,
    pub applied_through_sequence: i64,
    pub watermark_at: Timestamp,
    pub updated_at: Timestamp,
}

/// The durable `PENDING` status written by canonical ingestion.
pub const METRIC_ROLLUP_DELTA_PENDING: &str = "PENDING";
/// The durable `CLAIMED` status written by a successful claim.
pub const METRIC_ROLLUP_DELTA_CLAIMED: &str = "CLAIMED";
/// The durable terminal `APPLIED` status written by a successful completion.
pub const METRIC_ROLLUP_DELTA_APPLIED: &str = "APPLIED";

/// The smallest accepted `claimMetricRollupDeltas` limit.
pub const METRIC_ROLLUP_CLAIM_MIN_BATCH: i32 = 1;
/// The largest accepted `claimMetricRollupDeltas` limit.
///
/// A limit outside `1..=50` is **rejected**, not clamped. Unlike the
/// `BE-04` worker claim, where clamping keeps a long-running process making
/// bounded progress, a rollup claim's batch size determines exactly how far
/// the single global frontier advances in one transaction; silently granting
/// a different span than the caller asked for would make the caller's own
/// idea of what it holds wrong.
pub const METRIC_ROLLUP_CLAIM_MAX_BATCH: i32 = 50;

/// The server-fixed rollup lease, in seconds.
///
/// Fixed rather than caller-supplied: the lease is the only thing that makes
/// a crashed claimant's hold on the global frontier recoverable, so its
/// duration is a property of the protocol rather than a request parameter.
pub const METRIC_ROLLUP_LEASE_SECONDS: i32 = 900;

/// One delta granted by a successful claim.
///
/// Every row of one claim carries the **same** `claim_token` and the same
/// `lease_expires_at`: the token identifies the whole contiguous batch, and
/// completion addresses the batch by that token alone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricRollupDeltaClaim {
    pub delta_id: Uuid,
    pub work_day_sequence: i64,
    pub claim_token: Uuid,
    pub lease_expires_at: Timestamp,
}

/// The durable global rollup serving boundary.
///
/// `applied_through_sequence = W` means every committed work-day delta
/// position `1..=W` is `APPLIED`. Because allocation is gap-free and
/// application is strict-frontier, no projection row can hold the effect of a
/// work-day delta above `W` after a committed transaction.
///
/// `watermark_at` is the wall-clock time at which the current `W` was
/// atomically established. It is a freshness fact about the frontier, not a
/// claim that every canonical row is projected: a reader detects outstanding
/// rollup lag by comparing `W` with `next_sequence - 1`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricRollupWatermark {
    pub applied_through_sequence: i64,
    pub watermark_at: Timestamp,
}

/// Which claimed rollup batch to apply.
///
/// This carries the batch claim token and **nothing else**. No delta id,
/// arithmetic value, projection dimension or watermark value is accepted from
/// the caller: Thoth derives every one of them from the durable claimed batch
/// and the canonical records it names, so a claimant cannot choose what its
/// completion adds, to which aggregate, or how far the watermark moves.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(description = "Which claimed rollup delta batch to apply")
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompleteMetricRollupDeltasInput {
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "The batch token returned by every row of one successful claim. It is the only accepted input: all values, dimensions and the resulting watermark are derived from durable state"
        )
    )]
    pub claim_token: Uuid,
}

#[cfg(feature = "backend")]
pub mod crud;
#[cfg(all(test, feature = "backend"))]
pub(crate) mod tests;
