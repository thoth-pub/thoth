//! Metric platform/measure mapping registry (`MET-WP1-01`).
//!
//! This module owns the persisted `metric_platform_measure` registry model:
//! which measures a platform supports, at which reporting grains, with which
//! dimensions, and whether Thoth collects the measure directly from the
//! platform.
//!
//! `supported_grains` is a PostgreSQL array of the closed
//! `metric_reporting_grain` enum. The database rejects an empty array, a NULL
//! element and a duplicate grain, so every persisted mapping names at least
//! one distinct supported grain.
//!
//! `metric_platform_measure` deliberately has **no `created_at` or
//! `updated_at` columns**: the approved Metrics design (§6.3) omits them, and
//! future protected registry-administration/audit work must separately decide
//! whether mutation history or timestamps are required before exposing a write
//! path.
//!
//! `MET-WP1-01` seeds no mapping row: source mappings are explicitly
//! unapproved, and no platform/dimension/grain combination may be invented by
//! seed data. Registry foreign keys are non-cascading: deleting a platform or
//! measure with a mapping fails instead of silently deleting the mapping, and
//! later administration should prefer disablement over destructive registry
//! deletion.

use serde::{Deserialize, Serialize};
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

/// The reporting grain at which a platform reports a measure.
///
/// The inventory is closed: an unrecognised database, serde or string value
/// must fail rather than resolve to a nearest grain.
#[cfg_attr(
    feature = "backend",
    derive(diesel_derive_enum::DbEnum),
    ExistingTypePath = "crate::schema::sql_types::MetricReportingGrain"
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum MetricReportingGrain {
    /// One calendar day.
    #[cfg_attr(feature = "backend", db_rename = "DAY")]
    Day,
    /// One calendar month.
    #[cfg_attr(feature = "backend", db_rename = "MONTH")]
    Month,
    /// The source's own reporting period.
    #[cfg_attr(feature = "backend", db_rename = "REPORTING_PERIOD")]
    ReportingPeriod,
}

/// One persisted platform/measure mapping registry row.
///
/// Each `(platform_id, measure_id)` pair is unique, and `supported_grains` is
/// a non-empty array of distinct reporting grains whose order is preserved as
/// persisted. This type deliberately carries no timestamps because the table
/// has none.
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricPlatformMeasure {
    pub platform_measure_id: Uuid,
    pub platform_id: Uuid,
    pub measure_id: Uuid,
    pub supported_grains: Vec<MetricReportingGrain>,
    pub supports_country: bool,
    pub supports_institution: bool,
    pub supports_publication: bool,
    pub direct_collection: bool,
    pub enabled: bool,
}

#[cfg(all(test, feature = "backend"))]
mod tests;
