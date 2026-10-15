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
//! `MET-WP1-01` seeded no platform row and approved no source mapping, and
//! `MET-WP1-12` changes neither: it adds the protected SUPERUSER-only
//! administration surface through which a platform row may later be created and
//! maintained, and seeds nothing itself.
//!
//! `MET-WP1-12` therefore exposes [`MetricPlatformOwnershipClass`] as a
//! `juniper::GraphQLEnum`. That is an additive SDL exposure of the existing
//! closed database enum: no value is added, removed or renamed.
//!
//! Administration uses the stable `code`, never the database-generated
//! `platform_id`. Codes are matched by **exact** PostgreSQL `TEXT` equality at
//! every entry point: create stores what the caller supplied, and lookup and
//! update selectors compare literally. Nothing here trims, case-folds,
//! normalizes Unicode or whitespace, aliases or otherwise transforms a code.
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
    derive(diesel_derive_enum::DbEnum, juniper::GraphQLEnum),
    graphql(description = "Who operates and controls a metric platform"),
    ExistingTypePath = "crate::schema::sql_types::MetricPlatformOwnershipClass"
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum MetricPlatformOwnershipClass {
    /// Thoth itself operates the platform and its measurement.
    #[cfg_attr(
        feature = "backend",
        db_rename = "THOTH_MANAGED",
        graphql(description = "Thoth itself operates the platform and its measurement")
    )]
    ThothManaged,
    /// A publisher controls the platform and reports its activity.
    #[cfg_attr(
        feature = "backend",
        db_rename = "PUBLISHER_CONTROLLED",
        graphql(description = "A publisher controls the platform and reports its activity")
    )]
    PublisherControlled,
    /// A third party operates the platform.
    #[cfg_attr(
        feature = "backend",
        db_rename = "EXTERNAL",
        graphql(description = "A third party operates the platform")
    )]
    External,
}

/// One persisted metric-platform registry row.
///
/// `code` is the stable identifier: display-name changes never change codes,
/// and later administration should prefer disablement (`enabled = false`) over
/// destructive registry deletion. The database rejects blank `code` and
/// `display_name` values and duplicate codes.
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
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

/// Values for a new metric-platform registry row (`MET-WP1-12`).
///
/// `platform_id`, `created_at` and `updated_at` are database-owned and are not
/// accepted from callers. `code` is stored exactly as supplied.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(description = "Values for a metric platform to be created. Superuser only")
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewMetricPlatform {
    pub code: String,
    pub display_name: String,
    pub ownership_class: MetricPlatformOwnershipClass,
    pub enabled: bool,
    pub public_description: Option<String>,
}

/// A complete replacement of a metric platform's mutable fields
/// (`MET-WP1-12`).
///
/// This is a **replacement, not a sparse patch**: every mutable field is
/// carried on every call. For the nullable `public_description`, omission and
/// explicit GraphQL `null` both mean *store SQL NULL*; retaining the existing
/// value requires sending that value.
///
/// `code` is the stable selector and is **immutable**: it identifies the row
/// and is never written by an update. `ownership_class` is likewise absent,
/// because it participates in later collection and authorization semantics; a
/// mistaken ownership classification has no in-band repair path in
/// `MET-WP1-12` and requires separately reviewed correction work.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(
        description = "Complete replacement of a metric platform's mutable values, selected by its stable code. Superuser only. This is a replacement, not a partial patch: an omitted or null publicDescription stores SQL NULL. The code and ownershipClass cannot be changed"
    )
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchMetricPlatform {
    pub code: String,
    pub display_name: String,
    pub enabled: bool,
    pub public_description: Option<String>,
}

#[cfg(feature = "backend")]
pub mod crud;
#[cfg(all(test, feature = "backend"))]
pub(crate) mod tests;
