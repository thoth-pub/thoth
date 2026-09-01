//! Metrics coverage foundation (`MET-WP1-05`).
//!
//! This module owns the persisted `metric_coverage` model: one durable
//! record of what a source account's import reported it covers for one
//! platform/measure over one half-open period, independent of and prior to
//! any canonical `metric_record` computation.
//!
//! `MET-WP1-05` is an inactive additive foundation. It seeds no coverage row
//! and implements **no runtime behaviour**: there is no coverage
//! calculation, finalization, zero-versus-unknown behaviour or normalized
//! ingestion/`ingestMetricBatch` transaction. Those belong to later bounded
//! WP2/WP4 work. The module exposes no GraphQL or administration surface, so
//! the enum below is deliberately **not** a `juniper::GraphQLEnum`.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

/// Whether a reported coverage period is complete, partial or unknown.
///
/// The inventory is closed and matches the approved design exactly. There is
/// deliberately no `OTHER` or `Default` variant: an unrecognised database,
/// serde or string value must fail rather than silently resolve to a nearest
/// state.
#[cfg_attr(
    feature = "backend",
    derive(diesel_derive_enum::DbEnum),
    ExistingTypePath = "crate::schema::sql_types::MetricCoverageStatus"
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum MetricCoverageStatus {
    /// The reported period is fully covered.
    #[cfg_attr(feature = "backend", db_rename = "COMPLETE")]
    Complete,
    /// The reported period is only partially covered.
    #[cfg_attr(feature = "backend", db_rename = "PARTIAL")]
    Partial,
    /// Whether the reported period is fully covered is not known.
    #[cfg_attr(feature = "backend", db_rename = "UNKNOWN")]
    Unknown,
}

/// One persisted coverage row.
///
/// The database enforces half-open period ordering
/// (`period_end > period_start`) and non-null `coverage_status`,
/// `country_coverage` and `institution_coverage`. Every foreign key — to
/// `metric_source_account`, `metric_import`, `metric_platform` and
/// `metric_measure` — is direct and non-cascading, so deleting a referenced
/// source account, import, platform or measure fails instead of silently
/// deleting coverage history. There is deliberately no coverage uniqueness
/// constraint beyond the primary key and no coverage-specific secondary
/// index at this foundation stage.
///
/// `country_coverage` and `institution_coverage` record whether the reported
/// coverage includes the country and institution dimensions respectively;
/// both are plain non-null booleans, not the closed `MetricCoverageStatus`
/// enum, because they describe dimension presence rather than completeness.
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricCoverage {
    pub coverage_id: Uuid,
    pub source_account_id: Uuid,
    pub import_id: Uuid,
    pub platform_id: Uuid,
    pub measure_id: Uuid,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub coverage_status: MetricCoverageStatus,
    pub country_coverage: bool,
    pub institution_coverage: bool,
    pub notes: Option<String>,
}

#[cfg(all(test, feature = "backend"))]
pub(crate) mod tests;
