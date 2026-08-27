//! Metric source checkpoints (`MET-WP1-02`).
//!
//! This module owns the persisted `metric_source_checkpoint` model: durable
//! per-partition checkpoint, progress and lease **storage** for one
//! [`metric_source_account`](crate::model::metric_source_account). Checkpoint
//! identity is `(source_account_id, partition_key)` and is enforced by the
//! database. PostgreSQL is the sole durable owner of this state: Sphinx and
//! other orchestration must never keep canonical checkpoints in local files,
//! S3 or CI state.
//!
//! This slice establishes the durable columns only. The operation-level
//! concurrency protocol — claim tokens, lease acquisition/release,
//! `FOR UPDATE SKIP LOCKED`, stale-lease recovery, retries — is deliberately
//! **not** implemented or modelled here; it belongs to the later bounded
//! internal claim/checkpoint API task, together with its own tests.
//!
//! `cursor` stays generic nullable JSON because its content is
//! source-specific; no source-specific schema is imposed. `updated_at` uses
//! the repository-standard `diesel_manage_updated_at` trigger; the approved
//! design specifies no `created_at` column.

use chrono::NaiveDate;
use uuid::Uuid;

use crate::model::Timestamp;

/// One persisted metric-source-checkpoint row.
///
/// All progress, lease and error fields are nullable as designed: a fresh
/// checkpoint records only its identity. The database rejects blank
/// `partition_key` values and duplicate `(source_account_id, partition_key)`
/// pairs.
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricSourceCheckpoint {
    pub source_checkpoint_id: Uuid,
    pub source_account_id: Uuid,
    pub partition_key: String,
    pub cursor: Option<serde_json::Value>,
    pub last_discovered_at: Option<Timestamp>,
    pub last_completed_at: Option<Timestamp>,
    pub last_successful_period_end: Option<NaiveDate>,
    pub lease_owner: Option<String>,
    pub lease_expires_at: Option<Timestamp>,
    pub last_error: Option<String>,
    pub updated_at: Timestamp,
}

#[cfg(all(test, feature = "backend"))]
pub(crate) mod tests;
