//! Metric platform registry (`MET-WP1-01`).
//!
//! This module owns the persisted `metric_platform` registry model: the
//! services on which measured activity occurred. Platforms are **database
//! registry rows identified by a stable `code`**, not a Rust enum of platform
//! names: the registry must be extensible without a Rust enum migration for
//! every source ([ADR-0002] section 4.1).
//!
//! Per [ADR-0002], `MetricPlatform` is a domain separate from
//! `DistributionPlatform`. There is deliberately no conversion, alias or
//! name-based mapping between them: a metrics platform describes where
//! activity was observed, not where works are delivered.
//!
//! `MET-WP1-01` is an inactive additive registry foundation. It seeds no
//! platform row, approves no source mapping, and exposes no GraphQL or
//! administration surface; the enum below is therefore deliberately **not** a
//! `juniper::GraphQLEnum`.
//!
//! [ADR-0002]: ../../../docs/engineering/decisions/ADR-0002-platform-domain-boundaries.md

use serde::{Deserialize, Serialize};
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

use crate::model::Timestamp;

/// Who operates and controls a metric platform.
///
/// The inventory is closed. There is deliberately no `OTHER`, `UNKNOWN` or
/// `Default` variant: an unrecognised database, serde or string value must
/// fail rather than silently resolve to a nearest class.
#[cfg_attr(
    feature = "backend",
    derive(diesel_derive_enum::DbEnum),
    ExistingTypePath = "crate::schema::sql_types::MetricPlatformOwnershipClass"
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum MetricPlatformOwnershipClass {
    /// Thoth itself operates the platform and its measurement.
    #[cfg_attr(feature = "backend", db_rename = "THOTH_MANAGED")]
    ThothManaged,
    /// A publisher controls the platform and reports its activity.
    #[cfg_attr(feature = "backend", db_rename = "PUBLISHER_CONTROLLED")]
    PublisherControlled,
    /// A third party operates the platform.
    #[cfg_attr(feature = "backend", db_rename = "EXTERNAL")]
    External,
}

/// One persisted metric-platform registry row.
///
/// `code` is the stable identifier: display-name changes never change codes,
/// and later administration should prefer disablement (`enabled = false`) over
/// destructive registry deletion. The database rejects blank `code` and
/// `display_name` values and duplicate codes.
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricPlatform {
    pub platform_id: Uuid,
    pub code: String,
    pub display_name: String,
    pub ownership_class: MetricPlatformOwnershipClass,
    pub enabled: bool,
    pub public_description: Option<String>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[cfg(all(test, feature = "backend"))]
pub(crate) mod tests;
