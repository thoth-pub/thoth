//! Metric imports (`MET-WP1-03`).
//!
//! This module owns the persisted `metric_import` model: one durable,
//! Thoth-owned normalized ingestion job associated with a raw report, API
//! response, source partition or publisher upload. An import records its
//! identity, lifecycle status, immutable raw-evidence references, the
//! deterministic idempotency evidence used to recognise a repeat submission,
//! its row-result summary counters and the normalizer that produced them.
//!
//! Per the approved Metrics design, Thoth is the sole canonical owner of
//! durable Metrics state; Sphinx remains stateless orchestration and receives
//! no direct database authority. Metrics deliberately does **not** reuse the
//! Publisher Services `distribution_job*` tables, Rust types or lifecycle
//! APIs by analogy: import state is Metrics-specific durable state.
//!
//! `MET-WP1-03` is an inactive additive foundation. It seeds no import row and
//! implements **no runtime behaviour**: there is no upload or completion API,
//! no worker claiming, lease, retry or stale-claim recovery, no status
//! transition machine, no counter mutation protocol and no "return the
//! existing import" duplicate handling. The database uniqueness that later
//! idempotent-return behaviour will rely on is established here, but the
//! lookup/return path itself belongs to later bounded WP2/WP3 work. The module
//! exposes no GraphQL or administration surface, so the enum below is
//! deliberately **not** a `juniper::GraphQLEnum`.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

use crate::model::Timestamp;

/// The lifecycle state of one metric import.
///
/// The inventory is closed and matches the approved design exactly. There is
/// deliberately no `OTHER`, `UNKNOWN` or `Default` variant: an unrecognised
/// database, serde or string value must fail rather than silently resolve to
/// a nearest lifecycle state.
///
/// This slice stores the state only. Transition authorization, claiming,
/// retries, stale-worker recovery and any lifecycle state-machine API are
/// deliberately absent.
#[cfg_attr(
    feature = "backend",
    derive(diesel_derive_enum::DbEnum),
    ExistingTypePath = "crate::schema::sql_types::MetricImportStatus"
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum MetricImportStatus {
    /// Raw evidence has been received but no processing is scheduled yet.
    #[cfg_attr(feature = "backend", db_rename = "UPLOADED")]
    Uploaded,
    /// The import is queued for processing.
    #[cfg_attr(feature = "backend", db_rename = "QUEUED")]
    Queued,
    /// The import is being processed.
    #[cfg_attr(feature = "backend", db_rename = "PROCESSING")]
    Processing,
    /// Processing finished with no row-level error.
    #[cfg_attr(feature = "backend", db_rename = "COMPLETED")]
    Completed,
    /// Processing finished, but at least one row was rejected or flagged.
    #[cfg_attr(feature = "backend", db_rename = "COMPLETED_WITH_ERRORS")]
    CompletedWithErrors,
    /// Processing could not complete.
    #[cfg_attr(feature = "backend", db_rename = "FAILED")]
    Failed,
}

/// One persisted metric-import row.
///
/// Import identity is source-account scoped. The two design-fixed idempotency
/// paths are enforced by mutually exclusive partial unique indexes:
/// `(source_account_id, upstream_report_id)` when an upstream report ID is
/// supplied, and otherwise `(source_account_id, raw_sha256, format_version)`
/// when a raw hash is supplied. Both columns remain nullable exactly as
/// designed: nothing here requires an import to carry idempotency evidence at
/// row creation, because later upload/claim APIs own the rule for when
/// sufficient evidence is required before queueing or processing.
///
/// The database rejects blank `format_code`, `format_version`,
/// `normalizer_version` and `created_by` values and negative summary
/// counters. `created_by` is deliberately plain text with no account foreign
/// key, identity-provider binding or actor-namespace rule in this slice.
/// `manifest` is generic non-null JSONB defaulting to `{}`; no
/// source-specific manifest schema is imposed on it here. The import period
/// deliberately carries no ordering constraint: the approved design places
/// that constraint on `metric_record`, so malformed source period evidence
/// must stay representable at the import/error layer.
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricImport {
    pub import_id: Uuid,
    pub source_account_id: Uuid,
    pub publisher_id: Option<Uuid>,
    pub format_code: String,
    pub format_version: String,
    pub raw_object_key: Option<String>,
    pub raw_sha256: Option<String>,
    pub upstream_report_id: Option<String>,
    pub period_start: Option<NaiveDate>,
    pub period_end: Option<NaiveDate>,
    pub status: MetricImportStatus,
    pub received_count: i64,
    pub accepted_count: i64,
    pub duplicate_count: i64,
    pub revision_count: i64,
    pub conflict_count: i64,
    pub invalid_count: i64,
    pub normalizer_version: String,
    pub manifest: serde_json::Value,
    pub created_by: String,
    pub created_at: Timestamp,
    pub completed_at: Option<Timestamp>,
}

#[cfg(all(test, feature = "backend"))]
pub(crate) mod tests;
