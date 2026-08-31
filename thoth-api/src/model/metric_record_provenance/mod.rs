//! Canonical metric-record provenance (`MET-WP1-04`).
//!
//! This module owns the persisted `metric_record_provenance` model: durable
//! evidence describing how one normalized source row relates to canonical
//! state. Per `thoth-api/AGENTS.md`, no import row or state transition may
//! disappear silently, so provenance exists for **every** normalized row —
//! including rows that were rejected or that conflicted with canonical state
//! and therefore produced no record.
//!
//! `MET-WP1-04` stores the classification but implements **no** algorithm
//! that assigns it. First-arrival arbitration, duplicate/revision/conflict
//! resolution and rejected-row handling all belong to the later bounded WP2
//! ingestion work. The module exposes no GraphQL or administration surface, so
//! the enum below is deliberately **not** a `juniper::GraphQLEnum`.

use serde::{Deserialize, Serialize};
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

use crate::model::Timestamp;

/// How one normalized source row related to canonical state.
///
/// The inventory is closed and matches the approved design exactly. There is
/// deliberately no `OTHER`, `UNKNOWN` or `Default` variant: an unrecognised
/// database, serde or string value must fail rather than silently resolve to
/// a nearest classification.
#[cfg_attr(
    feature = "backend",
    derive(diesel_derive_enum::DbEnum),
    ExistingTypePath = "crate::schema::sql_types::MetricRecordProvenanceClassification"
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum MetricRecordProvenanceClassification {
    /// The row won first arrival and established the canonical record value.
    #[cfg_attr(feature = "backend", db_rename = "WINNER")]
    Winner,
    /// The row repeated content already held for the canonical record.
    #[cfg_attr(feature = "backend", db_rename = "DUPLICATE")]
    Duplicate,
    /// The row supplied an authorized correction to the canonical record.
    #[cfg_attr(feature = "backend", db_rename = "REVISION")]
    Revision,
    /// The row disagreed with canonical state without authority to revise it.
    #[cfg_attr(feature = "backend", db_rename = "CONFLICT")]
    Conflict,
    /// The row was refused, so it produced no canonical record.
    #[cfg_attr(feature = "backend", db_rename = "REJECTED")]
    Rejected,
}

/// One persisted canonical metric-record provenance row.
///
/// `record_id` is deliberately optional: rejected and conflicting rows need
/// durable evidence without a canonical record link, so provenance must be
/// recordable without inventing a record for it. Both foreign keys — the
/// optional record and the required MET-WP1-03 import — are non-cascading.
///
/// The database rejects blank `identity_hash` and `content_hash` values but
/// imposes no algorithm, encoding or length rule. `source_record_id` and
/// `source_row_number` stay optional and carry no origin convention: whether
/// rows are counted from zero, from one, or after a header belongs to the
/// later per-format normalizer contract. `details` is generic non-null JSONB
/// defaulting to an empty object; no source-specific schema is imposed on it.
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricRecordProvenance {
    pub record_provenance_id: Uuid,
    pub record_id: Option<Uuid>,
    pub import_id: Uuid,
    pub source_record_id: Option<String>,
    pub source_row_number: Option<i64>,
    pub identity_hash: String,
    pub content_hash: String,
    pub classification: MetricRecordProvenanceClassification,
    pub details: serde_json::Value,
    pub received_at: Timestamp,
}

#[cfg(all(test, feature = "backend"))]
pub(crate) mod tests;
