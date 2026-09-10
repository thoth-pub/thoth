//! Bounded ingestion batches (`MET-WP2-01A`).
//!
//! This module owns the persisted `metric_import_batch` model: durable
//! identity for one bounded ingestion batch attempt under an existing
//! [`metric_import`](crate::model::metric_import). It exists so that
//! `MET-WP2-01B` can recognise an already committed batch and return its
//! prior per-row outcomes without repeating canonical, provenance, counter,
//! revision, coverage or rollup-delta writes.
//!
//! The per-row outcomes themselves are **not** stored here. They remain in
//! [`metric_record_provenance`](crate::model::metric_record_provenance),
//! which stays the single authoritative classification/evidence row for every
//! normalized observation; provenance links back to its batch and carries its
//! position within it. Nothing in this table duplicates that classification,
//! and there is deliberately no opaque result JSON, no status, no completion
//! timestamp and no per-classification counter.
//!
//! `MET-WP2-01A` is an inactive additive foundation. It seeds no batch row and
//! implements no ingestion behaviour: batch creation, replay, and the
//! `IDEMPOTENCY_KEY_REUSED` classification of a repeated `batch_key` under a
//! different request hash are all `MET-WP2-01B` responsibilities. The module
//! exposes no GraphQL or administration surface.

use uuid::Uuid;

use crate::model::Timestamp;

/// One persisted bounded-batch row.
///
/// `batch_key` is the caller-supplied idempotency key, unique within its
/// import; the same key under a different import is a different batch.
///
/// `request_hash` is the deterministic hash of the batch request that first
/// created this row, stored as opaque nonblank text. `MET-WP2-01A`
/// deliberately fixes no canonicalization algorithm, encoding or length for
/// it: choosing how a request is canonicalized before hashing, and deciding
/// what a repeated `batch_key` with a differing hash means, belong to
/// `MET-WP2-01B`.
///
/// The database rejects blank `batch_key` and `request_hash` values, rejects a
/// duplicate `(import_id, batch_key)` pair, and refuses to cascade: deleting
/// an import that still has batches is restricted, not silently destructive.
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricImportBatch {
    pub import_batch_id: Uuid,
    pub import_id: Uuid,
    pub batch_key: String,
    pub request_hash: String,
    pub created_at: Timestamp,
}

#[cfg(all(test, feature = "backend"))]
pub(crate) mod tests;
