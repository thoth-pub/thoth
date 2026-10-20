//! Metric measure registry (`MET-WP1-01`).
//!
//! This module owns the persisted `metric_measure` registry model: the closed
//! vocabulary of what is being counted. Measures are database registry rows
//! identified by a stable `code`; semantically different measures remain
//! distinct, and no generic `usage` or combined-measure shortcut exists.
//!
//! The `MET-WP1-01` migration seeds exactly two measures:
//!
//! - `title_sessions` — usage counted under the approved CloudFront
//!   title-session methodology (`cloudfront-title-session/2`). The registry
//!   `methodology_version` is the measure's declared current baseline
//!   methodology; it does not replace per-batch/per-observation methodology
//!   provenance, which later ingestion slices must continue to record.
//! - `net_units` — signed net sales units, where negative values represent
//!   refunds or returns as reported by the source.
//!
//! Both seeds are additive across time and across works because the approved
//! design requires additive daily/monthly work-level aggregation:
//! `title_sessions` counts one session once per DOI and country under the
//! fixed methodology, and `net_units` sums signed units over non-overlapping
//! periods and work sets. No implication is made that unlike measures may be
//! combined.
//!
//! `MET-WP1-12` adds the protected SUPERUSER-only administration surface for
//! this registry and therefore exposes [`MetricMeasureCategory`] and
//! [`MetricMeasureUnit`] as `juniper::GraphQLEnum`s. That is an additive SDL
//! exposure of the existing closed database enums: no value is added, removed
//! or renamed, and the two migration-owned seeds are neither rewritten nor
//! re-seeded.
//!
//! Administration uses the stable `code`, never the database-generated
//! `measure_id`, so the seeded `title_sessions` and `net_units` measures are
//! addressable without out-of-band UUID discovery. Codes are matched by
//! **exact** PostgreSQL `TEXT` equality: nothing here trims, case-folds,
//! normalizes Unicode or whitespace, aliases or otherwise transforms a code.

use serde::{Deserialize, Serialize};
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

use crate::model::Timestamp;

/// The semantic family of a measure.
///
/// The inventory is closed: an unrecognised database, serde or string value
/// must fail rather than resolve to a nearest category.
#[cfg_attr(
    feature = "backend",
    derive(diesel_derive_enum::DbEnum, juniper::GraphQLEnum),
    graphql(description = "The semantic family of a measure"),
    ExistingTypePath = "crate::schema::sql_types::MetricMeasureCategory"
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum MetricMeasureCategory {
    /// Observed usage activity; usage measures reject negative values.
    #[cfg_attr(
        feature = "backend",
        db_rename = "USAGE",
        graphql(description = "Observed usage activity")
    )]
    Usage,
    /// Sales activity; sales measures may permit signed integer units.
    #[cfg_attr(
        feature = "backend",
        db_rename = "SALES",
        graphql(description = "Sales activity, which may permit signed integer units")
    )]
    Sales,
}

/// The unit in which a measure's values are expressed.
///
/// The initial approved inventory holds only `COUNT`.
#[cfg_attr(
    feature = "backend",
    derive(diesel_derive_enum::DbEnum, juniper::GraphQLEnum),
    graphql(description = "The unit in which a measure's values are expressed"),
    ExistingTypePath = "crate::schema::sql_types::MetricMeasureUnit"
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum MetricMeasureUnit {
    /// A dimensionless integer count.
    #[cfg_attr(
        feature = "backend",
        db_rename = "COUNT",
        graphql(description = "A dimensionless integer count")
    )]
    Count,
}

/// One persisted metric-measure registry row.
///
/// `code` is the stable identifier. `allow_negative` records whether later
/// ingestion may accept signed values for this measure; the additivity flags
/// record whether values may be summed across time and across works. The
/// database rejects blank `code`, `display_name` and `definition` values and
/// duplicate codes.
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MetricMeasure {
    pub measure_id: Uuid,
    pub code: String,
    pub display_name: String,
    pub category: MetricMeasureCategory,
    pub unit: MetricMeasureUnit,
    pub allow_negative: bool,
    pub public_visibility: bool,
    pub additive_across_time: bool,
    pub additive_across_works: bool,
    pub definition: String,
    pub methodology_version: Option<String>,
    pub enabled: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// Values for a new metric-measure registry row (`MET-WP1-12`).
///
/// `measure_id`, `created_at` and `updated_at` are database-owned and are not
/// accepted from callers. `code` is stored exactly as supplied.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(description = "Values for a metric measure to be created. Superuser only")
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewMetricMeasure {
    pub code: String,
    pub display_name: String,
    pub category: MetricMeasureCategory,
    pub unit: MetricMeasureUnit,
    pub allow_negative: bool,
    pub public_visibility: bool,
    pub additive_across_time: bool,
    pub additive_across_works: bool,
    pub definition: String,
    pub methodology_version: Option<String>,
    pub enabled: bool,
}

/// A complete replacement of a metric measure's mutable fields
/// (`MET-WP1-12`).
///
/// This is a **replacement, not a sparse patch**: every mutable field is
/// carried on every call. For the nullable `methodology_version`, omission and
/// explicit GraphQL `null` both mean *store SQL NULL*; retaining the existing
/// value requires sending that value.
///
/// `code` is the stable selector and is **immutable**. The canonical numeric
/// and aggregation semantics — `category`, `unit`, `allow_negative`,
/// `additive_across_time` and `additive_across_works` — are deliberately
/// absent, because changing them underneath accepted records would change the
/// meaning or validity of canonical history. Correcting one requires
/// separately reviewed repair work or a new stable measure code.
///
/// `methodology_version` remains mutable: it is the registry's declared
/// current baseline methodology and does not replace the per-observation and
/// per-import methodology provenance owned elsewhere in the Metrics model.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(
        description = "Complete replacement of a metric measure's mutable values, selected by its stable code. Superuser only. This is a replacement, not a partial patch: an omitted or null methodologyVersion stores SQL NULL. The code, category, unit, allowNegative, additiveAcrossTime and additiveAcrossWorks values cannot be changed"
    )
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchMetricMeasure {
    pub code: String,
    pub display_name: String,
    pub public_visibility: bool,
    pub definition: String,
    pub methodology_version: Option<String>,
    pub enabled: bool,
}

#[cfg(feature = "backend")]
pub mod crud;
#[cfg(all(test, feature = "backend"))]
pub(crate) mod tests;
