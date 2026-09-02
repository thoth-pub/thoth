//! Durable Metrics rollup deltas (`MET-WP1-07`).
//!
//! This module owns the persisted `metric_rollup_delta` model: the durable
//! accounting bridge between one canonical metric-record revision and the
//! rebuildable work-level rollup projections. The approved Metrics design
//! commits a canonical revision transactionally together with its delta —
//! applying a new record adds its value, a revision contributes `new - old`,
//! and a retraction subtracts the old value — so a delta is canonical
//! accounting evidence rather than a derived cache.
//!
//! `MET-WP1-07` stores delta state only. It implements **no** delta generation
//! from ingestion, no first-arrival/revision/retraction transaction, no
//! claiming, lease, retry, backoff or stale-claim recovery, no
//! `FOR UPDATE SKIP LOCKED` application loop, no delta application, no rebuild
//! generation and no active-watermark behaviour: those belong to the later
//! bounded WP4 rollup work. The four rebuildable work-level rollup projection
//! tables remain approved future architecture and are deliberately not created
//! by this slice. The module exposes no GraphQL, authorization or
//! administration surface.

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
}

#[cfg(all(test, feature = "backend"))]
pub(crate) mod tests;
