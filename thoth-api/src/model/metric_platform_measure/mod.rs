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
//! `updated_at` columns**: the approved Metrics design (§6.3) omits them. The
//! protected registry-administration/audit decision that shape deferred is
//! now closed by `MET-WP1-12`, which records mutation history in the separate
//! append-only [`metric_registry_history`] table and adds **no** timestamp
//! column here.
//!
//! `MET-WP1-12` exposes [`MetricReportingGrain`] as a `juniper::GraphQLEnum`.
//! That is an additive SDL exposure of the existing closed database enum: no
//! value is added, removed or renamed.
//!
//! Administration addresses a mapping by the stable `(platform_code,
//! measure_code)` pair rather than by any database-generated UUID, and those
//! codes are resolved by **exact** PostgreSQL `TEXT` equality, identically to
//! the platform and measure registries themselves.
//!
//! [`metric_registry_history`]: crate::model::metric_registry_history
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
    derive(diesel_derive_enum::DbEnum, juniper::GraphQLEnum),
    graphql(description = "The reporting grain at which a platform reports a measure"),
    ExistingTypePath = "crate::schema::sql_types::MetricReportingGrain"
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum MetricReportingGrain {
    /// One calendar day.
    #[cfg_attr(
        feature = "backend",
        db_rename = "DAY",
        graphql(description = "One calendar day")
    )]
    Day,
    /// One calendar month.
    #[cfg_attr(
        feature = "backend",
        db_rename = "MONTH",
        graphql(description = "One calendar month")
    )]
    Month,
    /// The source's own reporting period.
    #[cfg_attr(
        feature = "backend",
        db_rename = "REPORTING_PERIOD",
        graphql(description = "The source's own reporting period")
    )]
    ReportingPeriod,
}

/// One persisted platform/measure mapping registry row.
///
/// Each `(platform_id, measure_id)` pair is unique, and `supported_grains` is
/// a non-empty array of distinct reporting grains whose order is preserved as
/// persisted. This type deliberately carries no timestamps because the table
/// has none.
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
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

/// Values for a new platform/measure mapping registry row (`MET-WP1-12`).
///
/// The mapping is created from the stable platform and measure codes, which
/// the coordinator resolves to the persisted UUID foreign keys inside its own
/// transaction. `platform_measure_id` is database-owned.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(
        description = "Values for a metric platform/measure mapping to be created, referencing the platform and measure by their stable codes. Superuser only"
    )
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewMetricPlatformMeasure {
    pub platform_code: String,
    pub measure_code: String,
    pub supported_grains: Vec<MetricReportingGrain>,
    pub supports_country: bool,
    pub supports_institution: bool,
    pub supports_publication: bool,
    pub direct_collection: bool,
    pub enabled: bool,
}

/// A complete replacement of a platform/measure mapping's mutable fields
/// (`MET-WP1-12`).
///
/// This is a **replacement, not a sparse patch**: every mutable field is
/// carried on every call. The mapping has no nullable mutable field, so no
/// omission can store SQL NULL here.
///
/// The `(platform_code, measure_code)` pair is the stable selector and is
/// **immutable**: moving an existing mapping to another platform or measure
/// would silently change the identity of configuration that other durable
/// state may already reference. A different pair is created separately, and
/// the old row disabled where appropriate.
///
/// `direct_collection` is configuration rather than historical measurement
/// data and is mutable under the audit contract. No runtime collection,
/// import, export or reconciliation behaviour exists in WP1, so changing it
/// enqueues, backfills and activates nothing.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(
        description = "Complete replacement of a metric platform/measure mapping's mutable values, selected by the stable platform and measure codes. Superuser only. This is a replacement, not a partial patch. The mapped platform and measure cannot be changed"
    )
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchMetricPlatformMeasure {
    pub platform_code: String,
    pub measure_code: String,
    pub supported_grains: Vec<MetricReportingGrain>,
    pub supports_country: bool,
    pub supports_institution: bool,
    pub supports_publication: bool,
    pub direct_collection: bool,
    pub enabled: bool,
}

#[cfg(feature = "backend")]
pub mod crud;
#[cfg(all(test, feature = "backend"))]
pub(crate) mod tests;
