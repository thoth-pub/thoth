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
//! `MET-WP1-01` is an inactive additive registry foundation with no GraphQL or
//! administration surface; the enums below are therefore deliberately **not**
//! `juniper::GraphQLEnum`s.

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
    derive(diesel_derive_enum::DbEnum),
    ExistingTypePath = "crate::schema::sql_types::MetricMeasureCategory"
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum MetricMeasureCategory {
    /// Observed usage activity; usage measures reject negative values.
    #[cfg_attr(feature = "backend", db_rename = "USAGE")]
    Usage,
    /// Sales activity; sales measures may permit signed integer units.
    #[cfg_attr(feature = "backend", db_rename = "SALES")]
    Sales,
}

/// The unit in which a measure's values are expressed.
///
/// The initial approved inventory holds only `COUNT`.
#[cfg_attr(
    feature = "backend",
    derive(diesel_derive_enum::DbEnum),
    ExistingTypePath = "crate::schema::sql_types::MetricMeasureUnit"
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum MetricMeasureUnit {
    /// A dimensionless integer count.
    #[cfg_attr(feature = "backend", db_rename = "COUNT")]
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
#[derive(Debug, Clone, PartialEq, Eq)]
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

#[cfg(all(test, feature = "backend"))]
mod tests;
