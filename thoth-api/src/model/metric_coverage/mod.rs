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
//! WP2/WP4 work.
//!
//! `MET-WP4-02` exposes [`MetricCoverageStatus`] as a `juniper::GraphQLEnum`,
//! because the protected Metrics dashboard reports coverage in exactly this
//! vocabulary. That is an additive SDL exposure of the existing closed
//! database enum: no value is added, removed or renamed, and the persisted
//! coverage row itself is still exposed nowhere.

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
    derive(diesel_derive_enum::DbEnum, juniper::GraphQLEnum),
    graphql(description = "Whether coverage of a Metrics period is complete, partial or unknown"),
    ExistingTypePath = "crate::schema::sql_types::MetricCoverageStatus"
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum MetricCoverageStatus {
    /// The reported period is fully covered.
    #[cfg_attr(
        feature = "backend",
        db_rename = "COMPLETE",
        graphql(description = "The period is fully covered")
    )]
    Complete,
    /// The reported period is only partially covered.
    #[cfg_attr(
        feature = "backend",
        db_rename = "PARTIAL",
        graphql(
            description = "The period is only partially covered, so an absent value is not a zero"
        )
    )]
    Partial,
    /// Whether the reported period is fully covered is not known.
    #[cfg_attr(
        feature = "backend",
        db_rename = "UNKNOWN",
        graphql(
            description = "Whether the period is covered is not known, so an absent value is not a zero"
        )
    )]
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
