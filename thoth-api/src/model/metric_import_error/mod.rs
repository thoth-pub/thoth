//! Metric import errors (`MET-WP1-03`).
//!
//! This module owns the persisted `metric_import_error` model: durable,
//! stable, machine-readable row-level errors and warnings belonging to one
//! [`metric_import`](crate::model::metric_import). Per the approved Metrics
//! design and `thoth-api/AGENTS.md`, no import row or state transition may
//! disappear silently; this table is where a rejected or flagged row stays
//! explicit and diagnosable.
//!
//! `MET-WP1-03` is an inactive additive foundation. It seeds no error row and
//! defines **no** source-specific error-code registry, row-number origin
//! convention, downloadable error-file format, localization model or UI
//! representation: those belong to the later format/normalizer and publisher
//! upload contracts. The foreign key to the owning import is deliberately
//! non-cascading so durable import evidence cannot be silently erased by
//! deleting its parent. The module exposes no GraphQL or administration
//! surface, so the enum below is deliberately **not** a
//! `juniper::GraphQLEnum`.

use serde::{Deserialize, Serialize};
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

use crate::model::Timestamp;

/// How severe one row-level import finding is.
///
/// The inventory is closed and matches the approved design exactly: an import
/// finding either rejects its row or flags it. There is deliberately no
/// `INFO`, `DEBUG`, `FATAL`, `OTHER` or `Default` variant inferred from
/// generic application errors, logging frameworks or Publisher Services, and
/// an unrecognised database, serde or string value must fail rather than
/// silently resolve to a nearest severity.
#[cfg_attr(
    feature = "backend",
    derive(diesel_derive_enum::DbEnum),
    ExistingTypePath = "crate::schema::sql_types::MetricImportErrorSeverity"
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum MetricImportErrorSeverity {
    /// The row could not be accepted.
    #[cfg_attr(feature = "backend", db_rename = "ERROR")]
    Error,
    /// The row was accepted, but something about it needs attention.
    #[cfg_attr(feature = "backend", db_rename = "WARNING")]
    Warning,
}

/// One persisted metric-import-error row.
///
/// `error_code` is the stable machine-readable classification and `message`
/// the human-readable detail; the database rejects blank values for both.
/// `row_number`, `field_name` and `raw_value` remain nullable exactly as
/// designed, because a finding need not belong to a single numbered row or a
/// single named field. `row_number` deliberately carries no sign, range or
/// origin constraint in this slice: whether rows are counted from zero, from
/// one, or after a header belongs to the later per-format normalizer
/// contract.
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricImportError {
    pub import_error_id: Uuid,
    pub import_id: Uuid,
    pub row_number: Option<i64>,
    pub error_code: String,
    pub severity: MetricImportErrorSeverity,
    pub field_name: Option<String>,
    pub message: String,
    pub raw_value: Option<String>,
    pub created_at: Timestamp,
}

#[cfg(all(test, feature = "backend"))]
pub(crate) mod tests;
