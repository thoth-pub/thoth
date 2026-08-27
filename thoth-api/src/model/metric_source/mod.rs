//! Metric source registry (`MET-WP1-02`).
//!
//! This module owns the persisted `metric_source` model: the acquisition
//! route through which metrics arrive in Thoth. Sources are **database rows
//! identified by a stable `code`**, not a Rust enum of source names: the
//! source inventory must be extensible without a Rust enum migration for
//! every route, and no concrete source is approved or seeded by this slice.
//!
//! Per the approved Metrics design, Thoth is the sole canonical owner of
//! durable Metrics state; Sphinx remains stateless orchestration and receives
//! no direct database authority. `MET-WP1-02` is an inactive additive
//! foundation: it seeds no source row, approves no source/platform mapping,
//! implements no driver or driver registry (`driver_key` is plain nullable
//! text with no uniqueness or `DRIVER`-specific constraint), and exposes no
//! GraphQL or administration surface; the enum below is therefore
//! deliberately **not** a `juniper::GraphQLEnum`.

use serde::{Deserialize, Serialize};
use strum::Display;
use strum::EnumString;
use uuid::Uuid;

/// How metric data arrives from a source.
///
/// The inventory is closed. There is deliberately no `OTHER`, `UNKNOWN` or
/// `Default` variant: an unrecognised database, serde or string value must
/// fail rather than silently resolve to a nearest acquisition route.
#[cfg_attr(
    feature = "backend",
    derive(diesel_derive_enum::DbEnum),
    ExistingTypePath = "crate::schema::sql_types::MetricSourceAcquisitionType"
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum MetricSourceAcquisitionType {
    /// A Thoth-side driver collects the data from the source.
    #[cfg_attr(feature = "backend", db_rename = "DRIVER")]
    Driver,
    /// A publisher uploads the data.
    #[cfg_attr(feature = "backend", db_rename = "PUBLISHER_UPLOAD")]
    PublisherUpload,
    /// The data arrives through OPERAS synchronization.
    #[cfg_attr(feature = "backend", db_rename = "OPERAS")]
    Operas,
    /// An administrator imports the data.
    #[cfg_attr(feature = "backend", db_rename = "ADMIN_IMPORT")]
    AdminImport,
}

/// One persisted metric-source row.
///
/// `code` is the stable identifier: the database rejects blank codes and
/// duplicate codes. The optional lookback/finalization day defaults reject
/// negative values at the database boundary (`NULL` means "unset"); this
/// slice selects no actual source-specific value. The approved design
/// deliberately omits `created_at`/`updated_at` on this table.
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricSource {
    pub source_id: Uuid,
    pub code: String,
    pub acquisition_type: MetricSourceAcquisitionType,
    pub driver_key: Option<String>,
    pub enabled: bool,
    pub default_lookback_days: Option<i32>,
    pub default_finalization_delay_days: Option<i32>,
}

#[cfg(all(test, feature = "backend"))]
pub(crate) mod tests;
