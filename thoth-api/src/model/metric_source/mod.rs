//! Metric source registry (`MET-WP1-02`, administered by `MET-WP1-13`).
//!
//! This module owns the persisted `metric_source` model: the acquisition
//! route through which metrics arrive in Thoth. Sources are **database rows
//! identified by a stable `code`**, not a Rust enum of source names: the
//! source inventory must be extensible without a Rust enum migration for
//! every route, and no concrete source is approved or seeded by either slice.
//!
//! Per the approved Metrics design, Thoth is the sole canonical owner of
//! durable Metrics state; Sphinx remains stateless orchestration and receives
//! no direct database authority. `MET-WP1-02` was an inactive additive
//! foundation. `MET-WP1-13` adds the protected SUPERUSER-only administration
//! surface through which a source row may later be created and maintained,
//! and seeds nothing itself: it approves no real driver key, implements no
//! driver, and administers no checkpoint.
//!
//! `MET-WP1-13` exposes [`MetricSourceAcquisitionType`] as a
//! `juniper::GraphQLEnum`. That is an additive SDL exposure of the existing
//! closed database enum: no value is added, removed or renamed.
//!
//! Administration uses the stable `code`, never the database-generated
//! `source_id`. Codes are matched by **exact** PostgreSQL `TEXT` equality at
//! every entry point: create stores what the caller supplied, and lookup and
//! update selectors compare literally. Nothing here trims, case-folds,
//! normalizes Unicode or whitespace, aliases or otherwise transforms a code.
//!
//! The driver-key invariant is enforced at both boundaries: PostgreSQL's
//! `metric_source_driver_key_check` and the coordinator's own pre-check agree
//! that `DRIVER` requires a nonblank `driver_key` and every other acquisition
//! type requires `NULL`. A valid driver key is stored exactly as supplied.

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
    derive(diesel_derive_enum::DbEnum, juniper::GraphQLEnum),
    graphql(description = "How metric data arrives from a source"),
    ExistingTypePath = "crate::schema::sql_types::MetricSourceAcquisitionType"
)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, EnumString, Display)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum MetricSourceAcquisitionType {
    /// A Thoth-side driver collects the data from the source.
    #[cfg_attr(
        feature = "backend",
        db_rename = "DRIVER",
        graphql(description = "A Thoth-side driver collects the data from the source")
    )]
    Driver,
    /// A publisher uploads the data.
    #[cfg_attr(
        feature = "backend",
        db_rename = "PUBLISHER_UPLOAD",
        graphql(description = "A publisher uploads the data")
    )]
    PublisherUpload,
    /// The data arrives through OPERAS synchronization.
    #[cfg_attr(
        feature = "backend",
        db_rename = "OPERAS",
        graphql(description = "The data arrives through OPERAS synchronization")
    )]
    Operas,
    /// An administrator imports the data.
    #[cfg_attr(
        feature = "backend",
        db_rename = "ADMIN_IMPORT",
        graphql(description = "An administrator imports the data")
    )]
    AdminImport,
}

/// One persisted metric-source row.
///
/// `code` is the stable identifier: the database rejects blank codes and
/// duplicate codes. The optional lookback/finalization day defaults reject
/// negative values at the database boundary (`NULL` means "unset"). The
/// approved design deliberately omits `created_at`/`updated_at` on this table.
///
/// The row serializes to camel case so that an audit `before_state` /
/// `after_state` records exactly the persisted canonical state.
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MetricSource {
    pub source_id: Uuid,
    pub code: String,
    pub acquisition_type: MetricSourceAcquisitionType,
    pub driver_key: Option<String>,
    pub enabled: bool,
    pub default_lookback_days: Option<i32>,
    pub default_finalization_delay_days: Option<i32>,
}

/// Values for a new metric-source row (`MET-WP1-13`).
///
/// `source_id` is database-owned and is not accepted from callers. `code`,
/// `driver_key` and the day defaults are stored exactly as supplied.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(
        description = "Values for a metric source to be created. Superuser only. A DRIVER source requires a non-blank driverKey and every other acquisition type requires driverKey to be absent or null"
    )
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewMetricSource {
    pub code: String,
    pub acquisition_type: MetricSourceAcquisitionType,
    pub driver_key: Option<String>,
    pub enabled: bool,
    pub default_lookback_days: Option<i32>,
    pub default_finalization_delay_days: Option<i32>,
}

/// A complete replacement of a metric source's mutable fields (`MET-WP1-13`).
///
/// This is a **replacement, not a sparse patch**: every mutable field is
/// carried on every call. For the two nullable day defaults, omission and
/// explicit GraphQL `null` both mean *store SQL NULL*; retaining the existing
/// value requires sending that value.
///
/// `code` is the stable selector and is **immutable**. `acquisition_type` and
/// `driver_key` are likewise absent: they are immutable after creation and a
/// correction to either is separately reviewed repair work or a new source
/// identity, never an ordinary patch.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(
        description = "Complete replacement of a metric source's mutable values, selected by its stable code. Superuser only. This is a replacement, not a partial patch: an omitted or null day default stores SQL NULL. The code, acquisitionType and driverKey cannot be changed"
    )
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchMetricSource {
    pub code: String,
    pub enabled: bool,
    pub default_lookback_days: Option<i32>,
    pub default_finalization_delay_days: Option<i32>,
}

/// Every character the driver-key invariant treats as whitespace.
///
/// This is the Unicode `White_Space` property written out as explicit code
/// points, and it is the one definition both boundaries share:
/// [`MetricSourceAcquisitionType::accepts_driver_key`] consults exactly this
/// set, and PostgreSQL's `metric_source_driver_key_check` spells the same 25
/// code points as `\uXXXX` escapes in a negated regular-expression bracket.
/// Neither boundary consults a character-class name such as `[:space:]` or
/// `char::is_whitespace`, so the decision cannot vary with the database's
/// `LC_CTYPE`, collation or platform, nor with a toolchain's Unicode tables.
pub const DRIVER_KEY_WHITESPACE: [char; 25] = [
    '\u{0009}', '\u{000A}', '\u{000B}', '\u{000C}', '\u{000D}', '\u{0020}', '\u{0085}', '\u{00A0}',
    '\u{1680}', '\u{2000}', '\u{2001}', '\u{2002}', '\u{2003}', '\u{2004}', '\u{2005}', '\u{2006}',
    '\u{2007}', '\u{2008}', '\u{2009}', '\u{200A}', '\u{2028}', '\u{2029}', '\u{202F}', '\u{205F}',
    '\u{3000}',
];

impl MetricSourceAcquisitionType {
    /// Whether the driver-key invariant holds for this acquisition type.
    ///
    /// `DRIVER` requires a key containing at least one character outside
    /// [`DRIVER_KEY_WHITESPACE`]; every other type requires no key at all. This
    /// is the exact application-boundary twin of PostgreSQL's
    /// `metric_source_driver_key_check`, and it neither trims nor rewrites the
    /// key it inspects.
    pub fn accepts_driver_key(self, driver_key: Option<&str>) -> bool {
        match (self, driver_key) {
            (Self::Driver, Some(key)) => key.chars().any(|c| !DRIVER_KEY_WHITESPACE.contains(&c)),
            (Self::Driver, None) => false,
            (_, None) => true,
            (_, Some(_)) => false,
        }
    }
}

#[cfg(feature = "backend")]
pub mod crud;
#[cfg(all(test, feature = "backend"))]
pub(crate) mod tests;
