//! The protected, coverage-aware Metrics dashboard read (`MET-WP4-02`,
//! completed by `MET-WP4-03B`).
//!
//! This module owns the `metricDashboard` contract: its input, its response
//! and the one read that answers it. It consumes the `MET-WP4-01` work-day
//! projection and strict-frontier watermark, the `MET-WP4-03A` derived
//! monthly serving projections and the `MET-WP1-05` coverage evidence; it
//! owns none of them and writes none of them.
//!
//! # One snapshot
//!
//! [`metric_dashboard`] evaluates everything on one PostgreSQL connection in
//! one `READ ONLY, REPEATABLE READ` transaction. Publisher entitlement, the
//! rollup frontier, the metadata selector, registry scope, projected values,
//! current coverage, canonical source grains and outstanding rollup work are
//! therefore all facts about the same database state, and `asOf` is that
//! transaction's own timestamp. Nothing here advances the frontier, claims or
//! completes a delta, repairs a projection row or records coverage.
//!
//! # Zero is a claim
//!
//! A value of `"0"` asserts that nothing happened. It is returned for an
//! empty total or timeline cell only when every day of that cell is
//! effectively `COMPLETE` for the totals and timeline, no unapplied rollup
//! work touches it and no unresolved identifier evidence overlaps it;
//! otherwise an empty cell is `null`. A cell that does hold projected rows
//! always returns their exact sum, and the coverage items and warnings say
//! whether that sum can be relied on. Country and institution results never
//! invent a zero: they list only known values.
//!
//! # Publishers and works
//!
//! One to three publishers are selected, each of which must exist and be
//! entitled; one unentitled publisher refuses the whole request. The other
//! selector dimensions narrow the works of those publishers through current
//! Thoth metadata before any Metrics value is read, and only the publishers
//! represented by the resolved works take part in coverage, source accounts,
//! lag, values and identifier quality. Identifier quality, which has no work
//! identity, stays scoped to those publishers' historical imports and is never
//! narrowed to the resolved works.
//!
//! # Whole months and edge days
//!
//! Complete calendar months wholly inside the range are served from the
//! `MET-WP4-03A` monthly projections, which already hold each day's resolved
//! representation summed into its month. Clipped leading and trailing days
//! are served from `metric_rollup_work_day` with the same day-level
//! resolution. Totals, countries and institutions add the two; a `MONTH`
//! timeline uses the monthly value for a complete month and the days for a
//! clipped one; a `DAY` timeline reads every day.
//!
//! # Deliberately absent
//!
//! Works response sections, pagination, `dois`, `includeDescendants`,
//! `metricWidget`, entity-level Metrics fields, native `MONTH` and
//! `REPORTING_PERIOD` serving and any call to OPERAS.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{Datelike, Duration, NaiveDate};
use diesel::pg::PgConnection;
use diesel::sql_types::{
    Array, BigInt as SqlBigInt, Bool, Date, Nullable, Text, Timestamptz, Uuid as SqlUuid,
};
use diesel::RunQueryDsl;
use juniper::{graphql_value, FieldError, IntoFieldError};
use thoth_errors::ThothError;
use uuid::Uuid;

use crate::db::PgPool;
use crate::graphql::scalars::BigInt;
use crate::model::language::LanguageCode;
use crate::model::metric_coverage::MetricCoverageStatus;
use crate::model::publisher::{PublisherCapability, ThothPackage};
use crate::model::work::WorkType;
use crate::model::{Ror, Timestamp};

/// The longest accepted half-open `[startDate, endDate)` range, in days.
pub const METRIC_DASHBOARD_MAX_RANGE_DAYS: i64 = 366;
/// The most unique measures one dashboard request may select or resolve.
pub const METRIC_DASHBOARD_MAX_MEASURES: usize = 10;
/// The most unique platforms one dashboard request may select or resolve.
pub const METRIC_DASHBOARD_MAX_PLATFORMS: usize = 10;
/// The most platform/measure combinations one request may serve.
pub const METRIC_DASHBOARD_MAX_COMBINATIONS: usize = 25;
/// The most timeline cells (combinations x buckets) one request may return.
pub const METRIC_DASHBOARD_MAX_TIMELINE_CELLS: usize = 5000;
/// The most rows `metricMeasures` or `metricPlatforms` may return.
pub const METRIC_REGISTRY_READ_MAX: usize = 500;
/// The most unique publishers one dashboard request may select.
pub const METRIC_DASHBOARD_MAX_PUBLISHERS: usize = 3;
/// The most unique explicit work IDs one selector may carry.
pub const METRIC_DASHBOARD_MAX_WORK_IDS: usize = 500;
/// The most unique values every other bounded selector list may carry.
pub const METRIC_DASHBOARD_MAX_SELECTOR_VALUES: usize = 50;
/// The most works one selector may resolve to.
pub const METRIC_DASHBOARD_MAX_RESOLVED_WORKS: usize = 2000;
/// The most institution rows one response may return.
pub const METRIC_DASHBOARD_MAX_INSTITUTIONS: usize = 2000;

/// The one managed acquisition type MOM-1 treats as operational coverage.
const DRIVER_ACQUISITION: &str = "DRIVER";

// ---------------------------------------------------------------------------
// Input
// ---------------------------------------------------------------------------

/// The timeline bucket size.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "The size of each timeline bucket in a Metrics dashboard")
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MetricTimelineGrain {
    #[default]
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Let Thoth choose the bucket size. This is currently DAY")
    )]
    Auto,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "One bucket per calendar day of the requested range")
    )]
    Day,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "One bucket per calendar month, clipped to the requested range. Each bucket is the exact sum of the daily values it contains"
        )
    )]
    Month,
}

/// Which works a dashboard describes.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(
        description = "Which works a Metrics dashboard describes, resolved through current Thoth metadata before any Metrics value is read. Values within one list are alternatives (OR); different lists must all match (AND). An omitted or empty optional list does not restrict the works. Duplicate values and unknown IDs are rejected, never discarded; a known ID that matches none of the other selected works is valid and may leave no works"
    )
)]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MetricSelectorInput {
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "Thoth IDs of one to three publishers whose works to describe. Every selected publisher must exist and be entitled to the Metrics dashboard; otherwise the whole request is refused, never served for a subset"
        )
    )]
    pub publisher_ids: Option<Vec<Uuid>>,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Thoth IDs of at most 50 imprints the works must belong to")
    )]
    pub imprint_ids: Option<Vec<Uuid>>,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "Thoth IDs of at most 50 series. A work matches when it is issued in a selected series, or when it is a book chapter whose parent work is"
        )
    )]
    pub series_ids: Option<Vec<Uuid>>,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Thoth IDs of at most 500 works")
    )]
    pub work_ids: Option<Vec<Uuid>>,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "At most 50 work types the works must have")
    )]
    pub work_types: Option<Vec<WorkType>>,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "At most 50 languages. A work matches when any of its language records carries a selected language"
        )
    )]
    pub languages: Option<Vec<LanguageCode>>,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "Thoth IDs of at most 50 institutions. A work matches when it records funding from a selected institution"
        )
    )]
    pub funding_institution_ids: Option<Vec<Uuid>>,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "Thoth IDs of at most 50 institutions. A work matches when one of its contributions is affiliated with a selected institution"
        )
    )]
    pub affiliation_institution_ids: Option<Vec<Uuid>>,
}

/// One dashboard request.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(description = "A bounded, coverage-aware Metrics dashboard request")
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricDashboardInput {
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Which publishers and works to describe")
    )]
    pub selector: MetricSelectorInput,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "First calendar day of the range, inclusive")
    )]
    pub start_date: NaiveDate,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "Calendar day after the range, exclusive. It must be later than startDate and at most 366 days after it"
        )
    )]
    pub end_date: NaiveDate,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "Thoth IDs of the measures to serve, from at most 10 unique IDs. Omitted or empty means every measure represented for the selected works and range. Every served measure must be additive across time and works"
        )
    )]
    pub measures: Option<Vec<Uuid>>,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "Thoth IDs of the platforms to serve, from at most 10 unique IDs. Omitted or empty means every platform represented for the selected works and range"
        )
    )]
    pub platforms: Option<Vec<Uuid>>,
    #[cfg_attr(
        feature = "backend",
        graphql(
            default = MetricTimelineGrain::Auto,
            description = "The size of each timeline bucket"
        )
    )]
    pub timeline_grain: Option<MetricTimelineGrain>,
    #[cfg_attr(
        feature = "backend",
        graphql(
            default = true,
            description = "Whether to return per-country totals. When false no country rows are returned, and country coverage and country breakdown ambiguity affect the response only where the totals and timeline themselves depend on them"
        )
    )]
    pub include_countries: Option<bool>,
    #[cfg_attr(
        feature = "backend",
        graphql(
            default = true,
            description = "Whether to return per-institution totals. When false no institution rows are returned, and institution coverage and institution breakdown ambiguity affect the response only where the totals and timeline themselves depend on them"
        )
    )]
    pub include_institutions: Option<bool>,
}

// ---------------------------------------------------------------------------
// Response
// ---------------------------------------------------------------------------

/// One dashboard response.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLObject),
    graphql(
        description = "Coverage-aware Metrics totals, timeline, countries and institutions for one bounded request, read from one consistent database snapshot"
    )
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricDashboard {
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "One total per served platform and measure, never combining different platforms or measures"
        )
    )]
    pub totals: Vec<MetricTotal>,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Every bucket of every served platform and measure")
    )]
    pub timeline: Vec<MetricTimeBucket>,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "Known per-country totals over the range, ordered by platform ID, measure ID and country code. Empty when includeCountries is false. A country without a row is not a zero; coverage says how far absence can be relied on"
        )
    )]
    pub countries: Vec<MetricCountryTotal>,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "Known per-institution totals over the range, ordered by platform ID, measure ID and institution ID, at most 2000. Empty when includeInstitutions is false. An institution without a row is not a zero; coverage says how far absence can be relied on"
        )
    )]
    pub institutions: Vec<MetricInstitutionTotal>,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "How completely the served values of every returned section are covered"
        )
    )]
    pub coverage: MetricDashboardCoverage,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "The timestamp of the database transaction the response was read in"
        )
    )]
    pub as_of: Timestamp,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "The earliest per-item dataThrough date, or null unless every served item has one"
        )
    )]
    pub data_through: Option<NaiveDate>,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "When the applied work-day frontier the response was read against was established"
        )
    )]
    pub rollup_watermark: Timestamp,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "At most one warning per code, in the order UNKNOWN_COVERAGE, PARTIAL_COVERAGE, UNRESOLVED_IDENTIFIERS, ROLLUP_LAG"
        )
    )]
    pub warnings: Vec<MetricWarning>,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "True exactly when at least one warning applies")
    )]
    pub is_partial: bool,
}

/// The total of one served platform and measure over the whole range.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLObject),
    graphql(description = "The total of one platform and measure over the requested range")
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricTotal {
    pub platform_id: Uuid,
    pub measure_id: Uuid,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "The exact sum of projected values. \"0\" only when the whole range is completely covered with no outstanding rollup work or unresolved identifier evidence; null when no value is projected and zero cannot be justified"
        )
    )]
    pub value: Option<BigInt>,
}

/// One timeline bucket of one served platform and measure.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLObject),
    graphql(description = "One timeline bucket of one platform and measure")
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricTimeBucket {
    pub platform_id: Uuid,
    pub measure_id: Uuid,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "First day of the bucket, inclusive")
    )]
    pub start_date: NaiveDate,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Day after the bucket, exclusive")
    )]
    pub end_date: NaiveDate,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "The exact sum of projected values in the bucket. \"0\" only when the whole bucket is completely covered with no outstanding rollup work or unresolved identifier evidence; null when no value is projected and zero cannot be justified"
        )
    )]
    pub value: Option<BigInt>,
}

/// The known total of one country for one served platform and measure.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLObject),
    graphql(
        description = "The known total of one country for one platform and measure over the requested range"
    )
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricCountryTotal {
    pub platform_id: Uuid,
    pub measure_id: Uuid,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "The uppercase ISO 3166-1 alpha-2 country code the value was reported for"
        )
    )]
    pub country_code: String,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "The exact sum of the projected values reported for the country")
    )]
    pub value: BigInt,
}

/// The known total of one institution for one served platform and measure.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLObject),
    graphql(
        description = "The known total of one institution for one platform and measure over the requested range"
    )
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricInstitutionTotal {
    pub platform_id: Uuid,
    pub measure_id: Uuid,
    pub institution_id: Uuid,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "The institution's current Thoth name")
    )]
    pub institution_name: String,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "The institution's current ROR identifier, if Thoth records one")
    )]
    pub ror: Option<Ror>,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "The exact sum of the projected values reported for the institution"
        )
    )]
    pub value: BigInt,
}

/// Coverage across every served platform and measure.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLObject),
    graphql(
        name = "MetricCoverage",
        description = "Coverage across every served platform and measure"
    )
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricDashboardCoverage {
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "COMPLETE only when every item is COMPLETE; UNKNOWN when any item is UNKNOWN or nothing is served; otherwise PARTIAL"
        )
    )]
    pub status: MetricCoverageStatus,
    pub items: Vec<MetricCoverageItem>,
}

/// Coverage of one served platform and measure over the requested range.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLObject),
    graphql(description = "Coverage of one platform and measure over the requested range")
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricCoverageItem {
    pub platform_id: Uuid,
    pub measure_id: Uuid,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "COMPLETE only when every day is effectively COMPLETE for every returned section; UNKNOWN when any day is UNKNOWN; otherwise PARTIAL"
        )
    )]
    pub status: MetricCoverageStatus,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "The last day of the unbroken run from startDate in which every day is completely covered and untouched by outstanding rollup work, or null when the first day is not"
        )
    )]
    pub data_through: Option<NaiveDate>,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "True only when every day of the range has a current coverage assertion that includes the country dimension"
        )
    )]
    pub country_coverage: bool,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "True only when every day of the range has a current coverage assertion that includes the institution dimension"
        )
    )]
    pub institution_coverage: bool,
}

/// Why a response is partial.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLEnum),
    graphql(description = "Why a Metrics dashboard response is partial")
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MetricWarningCode {
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Some served platform, measure and day has partial coverage")
    )]
    PartialCoverage,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Some served platform, measure and day has unknown coverage")
    )]
    UnknownCoverage,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "Some source evidence in the served scope still has unresolved work identifiers"
        )
    )]
    UnresolvedIdentifiers,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "Canonical changes affecting the served values are not yet projected"
        )
    )]
    RollupLag,
}

/// One warning.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLObject),
    graphql(description = "One reason a Metrics dashboard response is partial")
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricWarning {
    pub code: MetricWarningCode,
    #[cfg_attr(feature = "backend", graphql(description = "Fixed explanatory text"))]
    pub message: String,
}

const UNKNOWN_COVERAGE_MESSAGE: &str = "Coverage is unknown for at least one served platform, measure and day, so a missing value there is not a zero.";
const PARTIAL_COVERAGE_MESSAGE: &str = "Coverage is partial for at least one served platform, measure and day, so values there may be incomplete.";
const UNRESOLVED_IDENTIFIERS_MESSAGE: &str = "Some source evidence in this request still has unresolved work identifiers, so values may be incomplete and an otherwise empty cell is not a zero.";
const ROLLUP_LAG_MESSAGE: &str = "Canonical changes affecting this request are not yet reflected in the served values, so they may change.";

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// A bounded, sanitized failure of a protected Metrics read.
///
/// Every message is fixed text. None carries SQL, a database diagnostic, a
/// token, a principal or any source configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetricReadError {
    /// Authentication, the `METRICS_READ_SERVICE` role or a selected
    /// publisher's `METRICS_DASHBOARD` capability is missing: `NO_ACCESS`.
    Unauthorised,
    /// `METRIC_QUERY_INVALID`.
    QueryInvalid(&'static str),
    /// `METRIC_QUERY_LIMIT_EXCEEDED`.
    QueryLimitExceeded(&'static str),
    /// `METRIC_REGISTRY_LIMIT_EXCEEDED`.
    RegistryLimitExceeded,
    /// `MOM1_SOURCE_SCOPE_AMBIGUOUS`.
    SourceScopeAmbiguous,
    /// `MOM1_DIMENSION_SCOPE_AMBIGUOUS`.
    DimensionScopeAmbiguous,
    /// `METRIC_QUERY_UNSUPPORTED_SOURCE_GRAIN`: an active canonical record
    /// of a native `MONTH` or `REPORTING_PERIOD` grain intersects the
    /// request, and this read serves only `DAY` records.
    UnsupportedSourceGrain,
    /// An unexpected database or pool failure, or internally inconsistent
    /// committed canonical state, deliberately without detail.
    Unavailable,
}

impl MetricReadError {
    /// The stable `extensions.type` classification.
    pub fn code(&self) -> &'static str {
        match self {
            MetricReadError::Unauthorised => "NO_ACCESS",
            MetricReadError::QueryInvalid(_) => "METRIC_QUERY_INVALID",
            MetricReadError::QueryLimitExceeded(_) => "METRIC_QUERY_LIMIT_EXCEEDED",
            MetricReadError::RegistryLimitExceeded => "METRIC_REGISTRY_LIMIT_EXCEEDED",
            MetricReadError::SourceScopeAmbiguous => "MOM1_SOURCE_SCOPE_AMBIGUOUS",
            MetricReadError::DimensionScopeAmbiguous => "MOM1_DIMENSION_SCOPE_AMBIGUOUS",
            MetricReadError::UnsupportedSourceGrain => "METRIC_QUERY_UNSUPPORTED_SOURCE_GRAIN",
            MetricReadError::Unavailable => "INTERNAL_ERROR",
        }
    }

    /// The fixed client-facing message.
    pub fn message(&self) -> &'static str {
        match self {
            // Identical to the repository's existing NO_ACCESS rendering.
            MetricReadError::Unauthorised => "Unauthorized",
            MetricReadError::QueryInvalid(message) | MetricReadError::QueryLimitExceeded(message) => {
                message
            }
            MetricReadError::RegistryLimitExceeded => {
                "The Metrics registry holds more than 500 entries, which exceeds what this operation may return."
            }
            MetricReadError::SourceScopeAmbiguous => {
                "More than one enabled managed source account serves a selected publisher and platform, so coverage cannot be resolved."
            }
            MetricReadError::DimensionScopeAmbiguous => {
                "A served work, platform, measure and day is recorded in more than one incompatible dimensional breakdown, so its value cannot be resolved."
            }
            MetricReadError::UnsupportedSourceGrain => {
                "Metrics reported for a whole month or reporting period intersect this request, and this dashboard cannot serve them yet. Narrow the works, platforms, measures or dates."
            }
            MetricReadError::Unavailable => "The Metrics read could not be completed.",
        }
    }
}

impl From<ThothError> for MetricReadError {
    fn from(error: ThothError) -> Self {
        match error {
            ThothError::Unauthorised => MetricReadError::Unauthorised,
            _ => MetricReadError::Unavailable,
        }
    }
}

impl From<diesel::result::Error> for MetricReadError {
    fn from(_: diesel::result::Error) -> Self {
        MetricReadError::Unavailable
    }
}

impl From<diesel::r2d2::PoolError> for MetricReadError {
    fn from(_: diesel::r2d2::PoolError) -> Self {
        MetricReadError::Unavailable
    }
}

impl IntoFieldError for MetricReadError {
    fn into_field_error(self) -> FieldError {
        if self == MetricReadError::Unauthorised {
            return ThothError::Unauthorised.into_field_error();
        }
        let code = self.code();
        FieldError::new(self.message(), graphql_value!({ "type": code }))
    }
}

const PUBLISHER_CARDINALITY: &str =
    "selector.publisherIds must contain between one and three publisher IDs.";
const DUPLICATE_PUBLISHER: &str = "selector.publisherIds must not contain duplicate IDs.";
const UNKNOWN_PUBLISHER: &str = "selector.publisherIds contains an unknown publisher ID.";
const DUPLICATE_IMPRINT: &str = "selector.imprintIds must not contain duplicate IDs.";
const DUPLICATE_SERIES: &str = "selector.seriesIds must not contain duplicate IDs.";
const DUPLICATE_WORK: &str = "selector.workIds must not contain duplicate IDs.";
const DUPLICATE_WORK_TYPE: &str = "selector.workTypes must not contain duplicate values.";
const DUPLICATE_LANGUAGE: &str = "selector.languages must not contain duplicate values.";
const DUPLICATE_FUNDING: &str = "selector.fundingInstitutionIds must not contain duplicate IDs.";
const DUPLICATE_AFFILIATION: &str =
    "selector.affiliationInstitutionIds must not contain duplicate IDs.";
const TOO_MANY_IMPRINTS: &str = "selector.imprintIds may contain at most 50 IDs.";
const TOO_MANY_SERIES: &str = "selector.seriesIds may contain at most 50 IDs.";
const TOO_MANY_WORKS: &str = "selector.workIds may contain at most 500 IDs.";
const TOO_MANY_WORK_TYPES: &str = "selector.workTypes may contain at most 50 values.";
const TOO_MANY_LANGUAGES: &str = "selector.languages may contain at most 50 values.";
const TOO_MANY_FUNDING: &str = "selector.fundingInstitutionIds may contain at most 50 IDs.";
const TOO_MANY_AFFILIATION: &str = "selector.affiliationInstitutionIds may contain at most 50 IDs.";
const UNKNOWN_IMPRINT: &str = "selector.imprintIds contains an unknown imprint ID.";
const UNKNOWN_SERIES: &str = "selector.seriesIds contains an unknown series ID.";
const UNKNOWN_WORK: &str = "selector.workIds contains an unknown work ID.";
const UNKNOWN_FUNDING: &str = "selector.fundingInstitutionIds contains an unknown institution ID.";
const UNKNOWN_AFFILIATION: &str =
    "selector.affiliationInstitutionIds contains an unknown institution ID.";
const TOO_MANY_RESOLVED_WORKS: &str =
    "The selector resolves to more than 2000 works. Narrow the selection.";
const TOO_MANY_INSTITUTIONS: &str = "The request would return more than 2000 institution rows. Narrow the request, or omit institutions.";
const DATE_ORDER: &str = "startDate must be earlier than endDate.";
const RANGE_TOO_LONG: &str = "The requested date range exceeds 366 days.";
const DUPLICATE_MEASURE: &str = "measures must not contain duplicate IDs.";
const DUPLICATE_PLATFORM: &str = "platforms must not contain duplicate IDs.";
const TOO_MANY_MEASURES: &str = "At most 10 measures may be served in one request.";
const TOO_MANY_PLATFORMS: &str = "At most 10 platforms may be served in one request.";
const UNKNOWN_MEASURE: &str = "measures contains an unknown metric measure ID.";
const UNKNOWN_PLATFORM: &str = "platforms contains an unknown metric platform ID.";
const NON_ADDITIVE_MEASURE: &str = "A served metric measure is not additive across both time and works, so this dashboard cannot sum it.";
const TOO_MANY_COMBINATIONS: &str =
    "At most 25 platform and measure combinations may be served in one request.";
const TOO_MANY_CELLS: &str = "The request would return more than 5000 timeline cells. Narrow the range, use MONTH, or serve fewer combinations.";
const AGGREGATE_OUT_OF_RANGE: &str =
    "An aggregate value exceeds the supported numeric range. Narrow the request.";

// ---------------------------------------------------------------------------
// Request validation (no database access)
// ---------------------------------------------------------------------------

/// The explicit metadata selector lists, each empty when unrestricted.
///
/// Work types and languages are held as their database enum labels, so the
/// statements can bind them as `text[]` and cast them in SQL: binding the
/// enum array types directly would make Diesel look their type OIDs up with
/// extra statements on a connection's first use.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct Selector {
    imprints: Vec<Uuid>,
    series: Vec<Uuid>,
    works: Vec<Uuid>,
    work_types: Vec<String>,
    languages: Vec<String>,
    funding_institutions: Vec<Uuid>,
    affiliation_institutions: Vec<Uuid>,
}

/// How a request's range is split between the monthly projections and the
/// work-day projection.
///
/// `[interior_start, interior_end)` is the run of complete calendar months
/// wholly inside the range, served from `MET-WP4-03A`; the leading days
/// `[start, interior_start)` and trailing days `[interior_end, end)` are the
/// clipped edges, served from `metric_rollup_work_day`. A range holding no
/// complete month has `interior_start == interior_end == end`, so its edges
/// are the whole range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Split {
    interior_start: NaiveDate,
    interior_end: NaiveDate,
}

impl Split {
    fn of(start: NaiveDate, end: NaiveDate) -> Self {
        let first_full = if start.day() == 1 {
            start
        } else {
            next_month(start)
        };
        let last_end = end.with_day(1).unwrap_or(end);
        if first_full < last_end {
            Split {
                interior_start: first_full,
                interior_end: last_end,
            }
        } else {
            Split {
                interior_start: end,
                interior_end: end,
            }
        }
    }

    fn has_interior(&self) -> bool {
        self.interior_start < self.interior_end
    }

    /// The first day of every complete month, in order.
    fn months(&self) -> Vec<NaiveDate> {
        let mut months = Vec::new();
        let mut month = self.interior_start;
        while month < self.interior_end {
            months.push(month);
            month = next_month(month);
        }
        months
    }
}

/// The first day of the month after `day`'s month.
fn next_month(day: NaiveDate) -> NaiveDate {
    let (year, month) = if day.month() == 12 {
        (day.year() + 1, 1)
    } else {
        (day.year(), day.month() + 1)
    };
    NaiveDate::from_ymd_opt(year, month, 1).unwrap_or(NaiveDate::MAX)
}

/// A request that passed every check that needs no database.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ValidatedRequest {
    publisher_ids: Vec<Uuid>,
    selector: Selector,
    start: NaiveDate,
    end: NaiveDate,
    /// `None` when the dimension was omitted or empty.
    measures: Option<BTreeSet<Uuid>>,
    platforms: Option<BTreeSet<Uuid>>,
    buckets: Vec<(NaiveDate, NaiveDate)>,
    /// Whether the timeline is monthly; otherwise it is daily.
    monthly_timeline: bool,
    include_countries: bool,
    include_institutions: bool,
    split: Split,
}

impl ValidatedRequest {
    /// The day ranges whose values are read from the work-day projection,
    /// as `(start, lead_end, trail_start, end)`: the clipped edges for a
    /// monthly timeline, every day for a daily one.
    fn day_reads(&self) -> (NaiveDate, NaiveDate, NaiveDate, NaiveDate) {
        if self.monthly_timeline {
            self.edges()
        } else {
            (self.start, self.end, self.end, self.end)
        }
    }

    /// The clipped edges, as `(start, lead_end, trail_start, end)`.
    fn edges(&self) -> (NaiveDate, NaiveDate, NaiveDate, NaiveDate) {
        (
            self.start,
            self.split.interior_start,
            self.split.interior_end,
            self.end,
        )
    }

    fn has_edges(&self) -> bool {
        self.start < self.split.interior_start || self.split.interior_end < self.end
    }

    fn has_day_reads(&self) -> bool {
        !self.monthly_timeline || self.has_edges()
    }
}

/// Every day of `[start, end)`, in order.
fn days_of(start: NaiveDate, end: NaiveDate) -> Vec<NaiveDate> {
    start.iter_days().take_while(|day| *day < end).collect()
}

/// The half-open timeline buckets of `[start, end)` at `grain`.
fn buckets_of(
    start: NaiveDate,
    end: NaiveDate,
    grain: MetricTimelineGrain,
) -> Vec<(NaiveDate, NaiveDate)> {
    match grain {
        MetricTimelineGrain::Auto | MetricTimelineGrain::Day => days_of(start, end)
            .into_iter()
            .map(|day| (day, day + Duration::days(1)))
            .collect(),
        MetricTimelineGrain::Month => {
            let mut buckets = Vec::new();
            let mut bucket_start = start;
            while bucket_start < end {
                let bucket_end = next_month(bucket_start).min(end);
                buckets.push((bucket_start, bucket_end));
                bucket_start = bucket_end;
            }
            buckets
        }
    }
}

/// A list as a set, rejecting duplicates and enforcing the list's bound.
/// Omitted and empty both mean unrestricted.
fn filter_set<T: Ord + Clone>(
    values: &Option<Vec<T>>,
    duplicate: &'static str,
    max: usize,
    too_many: &'static str,
) -> Result<Option<BTreeSet<T>>, MetricReadError> {
    let Some(values) = values.as_ref().filter(|values| !values.is_empty()) else {
        return Ok(None);
    };
    let set: BTreeSet<T> = values.iter().cloned().collect();
    if set.len() != values.len() {
        return Err(MetricReadError::QueryInvalid(duplicate));
    }
    if set.len() > max {
        return Err(MetricReadError::QueryLimitExceeded(too_many));
    }
    Ok(Some(set))
}

/// [`filter_set`] for a bounded selector list, as a plain list.
fn selector_list<T: Ord + Clone>(
    values: &Option<Vec<T>>,
    duplicate: &'static str,
    max: usize,
    too_many: &'static str,
) -> Result<Vec<T>, MetricReadError> {
    Ok(filter_set(values, duplicate, max, too_many)?
        .map(|set| set.into_iter().collect())
        .unwrap_or_default())
}

/// The `work_type` database label of a work type.
fn work_type_label(work_type: WorkType) -> &'static str {
    match work_type {
        WorkType::BookChapter => "book-chapter",
        WorkType::Monograph => "monograph",
        WorkType::EditedBook => "edited-book",
        WorkType::Textbook => "textbook",
        WorkType::JournalIssue => "journal-issue",
        WorkType::BookSet => "book-set",
    }
}

/// The `language_code` database label of a language: its three-letter code
/// in lower case. A value PostgreSQL does not accept fails the cast in SQL
/// rather than matching anything.
fn language_label(language: LanguageCode) -> String {
    language.to_string().to_lowercase()
}

fn validate(input: &MetricDashboardInput) -> Result<ValidatedRequest, MetricReadError> {
    let publisher_ids = input.selector.publisher_ids.clone().unwrap_or_default();
    // Nothing here picks the first entry, drops the rest or keeps an
    // entitled subset: a publisher list is served whole or refused.
    if publisher_ids.is_empty() {
        return Err(MetricReadError::QueryInvalid(PUBLISHER_CARDINALITY));
    }
    if publisher_ids.iter().collect::<BTreeSet<_>>().len() != publisher_ids.len() {
        return Err(MetricReadError::QueryInvalid(DUPLICATE_PUBLISHER));
    }
    if publisher_ids.len() > METRIC_DASHBOARD_MAX_PUBLISHERS {
        return Err(MetricReadError::QueryInvalid(PUBLISHER_CARDINALITY));
    }
    let selector = &input.selector;
    let selector = Selector {
        imprints: selector_list(
            &selector.imprint_ids,
            DUPLICATE_IMPRINT,
            METRIC_DASHBOARD_MAX_SELECTOR_VALUES,
            TOO_MANY_IMPRINTS,
        )?,
        series: selector_list(
            &selector.series_ids,
            DUPLICATE_SERIES,
            METRIC_DASHBOARD_MAX_SELECTOR_VALUES,
            TOO_MANY_SERIES,
        )?,
        works: selector_list(
            &selector.work_ids,
            DUPLICATE_WORK,
            METRIC_DASHBOARD_MAX_WORK_IDS,
            TOO_MANY_WORKS,
        )?,
        work_types: selector_list(
            &selector.work_types.as_ref().map(|types| {
                types
                    .iter()
                    .map(|work_type| work_type_label(*work_type))
                    .collect()
            }),
            DUPLICATE_WORK_TYPE,
            METRIC_DASHBOARD_MAX_SELECTOR_VALUES,
            TOO_MANY_WORK_TYPES,
        )?
        .into_iter()
        .map(str::to_string)
        .collect(),
        languages: selector_list(
            &selector
                .languages
                .as_ref()
                .map(|languages| languages.iter().map(|code| language_label(*code)).collect()),
            DUPLICATE_LANGUAGE,
            METRIC_DASHBOARD_MAX_SELECTOR_VALUES,
            TOO_MANY_LANGUAGES,
        )?,
        funding_institutions: selector_list(
            &selector.funding_institution_ids,
            DUPLICATE_FUNDING,
            METRIC_DASHBOARD_MAX_SELECTOR_VALUES,
            TOO_MANY_FUNDING,
        )?,
        affiliation_institutions: selector_list(
            &selector.affiliation_institution_ids,
            DUPLICATE_AFFILIATION,
            METRIC_DASHBOARD_MAX_SELECTOR_VALUES,
            TOO_MANY_AFFILIATION,
        )?,
    };
    if input.start_date >= input.end_date {
        return Err(MetricReadError::QueryInvalid(DATE_ORDER));
    }
    if (input.end_date - input.start_date).num_days() > METRIC_DASHBOARD_MAX_RANGE_DAYS {
        return Err(MetricReadError::QueryLimitExceeded(RANGE_TOO_LONG));
    }
    let measures = filter_set(
        &input.measures,
        DUPLICATE_MEASURE,
        METRIC_DASHBOARD_MAX_MEASURES,
        TOO_MANY_MEASURES,
    )?;
    let platforms = filter_set(
        &input.platforms,
        DUPLICATE_PLATFORM,
        METRIC_DASHBOARD_MAX_PLATFORMS,
        TOO_MANY_PLATFORMS,
    )?;
    let grain = input.timeline_grain.unwrap_or_default();
    Ok(ValidatedRequest {
        publisher_ids,
        selector,
        start: input.start_date,
        end: input.end_date,
        measures,
        platforms,
        buckets: buckets_of(input.start_date, input.end_date, grain),
        monthly_timeline: grain == MetricTimelineGrain::Month,
        include_countries: input.include_countries.unwrap_or(true),
        include_institutions: input.include_institutions.unwrap_or(true),
        split: Split::of(input.start_date, input.end_date),
    })
}

// ---------------------------------------------------------------------------
// Row shapes
// ---------------------------------------------------------------------------

/// The frontier, repeated on every row, beside one selected publisher that
/// exists, or `NULL`s when none does.
#[derive(diesel::QueryableByName)]
struct PublisherRow {
    #[diesel(sql_type = Timestamptz)]
    as_of: Timestamp,
    #[diesel(sql_type = SqlBigInt)]
    applied_through_sequence: i64,
    #[diesel(sql_type = SqlBigInt)]
    next_sequence: i64,
    #[diesel(sql_type = Timestamptz)]
    watermark_at: Timestamp,
    #[diesel(sql_type = Nullable<SqlUuid>)]
    publisher_id: Option<Uuid>,
    #[diesel(sql_type = Nullable<crate::schema::sql_types::ThothPackage>)]
    subscription_package: Option<ThothPackage>,
}

/// The transaction timestamp and the `MET-WP4-01` frontier.
struct FrontierRow {
    as_of: Timestamp,
    applied_through_sequence: i64,
    next_sequence: i64,
    watermark_at: Timestamp,
}

#[derive(diesel::QueryableByName)]
struct WorkRow {
    /// The selector dimension of an unknown explicit ID, or `NULL` for a
    /// resolved work.
    #[diesel(sql_type = Nullable<Text>)]
    unknown: Option<String>,
    #[diesel(sql_type = Nullable<SqlUuid>)]
    work_id: Option<Uuid>,
    #[diesel(sql_type = Nullable<SqlUuid>)]
    publisher_id: Option<Uuid>,
}

#[derive(diesel::QueryableByName)]
struct KnownRow {
    #[diesel(sql_type = Text)]
    kind: String,
    #[diesel(sql_type = SqlUuid)]
    id: Uuid,
    #[diesel(sql_type = Nullable<Bool>)]
    additive_across_time: Option<bool>,
    #[diesel(sql_type = Nullable<Bool>)]
    additive_across_works: Option<bool>,
}

#[derive(diesel::QueryableByName)]
struct RepresentedRow {
    #[diesel(sql_type = SqlUuid)]
    platform_id: Uuid,
    #[diesel(sql_type = SqlUuid)]
    measure_id: Uuid,
    #[diesel(sql_type = Bool)]
    additive_across_time: bool,
    #[diesel(sql_type = Bool)]
    additive_across_works: bool,
}

#[derive(diesel::QueryableByName)]
struct AccountRow {
    #[diesel(sql_type = SqlUuid)]
    publisher_id: Uuid,
    #[diesel(sql_type = SqlUuid)]
    platform_id: Uuid,
    #[diesel(sql_type = SqlUuid)]
    source_account_id: Uuid,
}

#[derive(diesel::QueryableByName)]
struct CanonicalRow {
    #[diesel(sql_type = Text)]
    kind: String,
    #[diesel(sql_type = Nullable<SqlUuid>)]
    platform_id: Option<Uuid>,
    #[diesel(sql_type = Nullable<SqlUuid>)]
    measure_id: Option<Uuid>,
    #[diesel(sql_type = Nullable<Date>)]
    day: Option<NaiveDate>,
}

/// One row of a section statement ([`MONTHS_SQL`] or
/// [`DAY_SECTIONS_SQL`]): a total, country or institution value, or the
/// statement's one ambiguity summary.
#[derive(diesel::QueryableByName)]
struct SectionRow {
    #[diesel(sql_type = Text)]
    section: String,
    #[diesel(sql_type = Nullable<SqlUuid>)]
    platform_id: Option<Uuid>,
    #[diesel(sql_type = Nullable<SqlUuid>)]
    measure_id: Option<Uuid>,
    #[diesel(sql_type = Nullable<Date>)]
    month_start: Option<NaiveDate>,
    #[diesel(sql_type = Nullable<Text>)]
    country_code: Option<String>,
    #[diesel(sql_type = Nullable<SqlUuid>)]
    institution_id: Option<Uuid>,
    #[diesel(sql_type = Nullable<Text>)]
    institution_name: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    ror: Option<Ror>,
    /// `SUM(bigint)` is `numeric`; it is read as text and parsed exactly so
    /// no aggregate is ever narrowed to `i64` or rounded through a float.
    #[diesel(sql_type = Nullable<Text>)]
    total: Option<String>,
    #[diesel(sql_type = Bool)]
    requires_country: bool,
    #[diesel(sql_type = Bool)]
    requires_institution: bool,
    #[diesel(sql_type = Bool)]
    total_ambiguous: bool,
    #[diesel(sql_type = Bool)]
    country_ambiguous: bool,
    #[diesel(sql_type = Bool)]
    institution_ambiguous: bool,
}

#[derive(diesel::QueryableByName)]
struct DaySumRow {
    #[diesel(sql_type = SqlUuid)]
    platform_id: Uuid,
    #[diesel(sql_type = SqlUuid)]
    measure_id: Uuid,
    #[diesel(sql_type = Date)]
    day: NaiveDate,
    /// `SUM(bigint)` is `numeric`; it is read as text and parsed exactly so
    /// no aggregate is ever narrowed to `i64` or rounded through a float.
    #[diesel(sql_type = Text)]
    total: String,
    #[diesel(sql_type = Bool)]
    has_aggregate: bool,
    #[diesel(sql_type = Bool)]
    has_dimensioned: bool,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    min_mask: i32,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    max_mask: i32,
    #[diesel(sql_type = Bool)]
    uses_country: bool,
    #[diesel(sql_type = Bool)]
    uses_institution: bool,
}

impl DaySumRow {
    /// Whether this group needs base-cell resolution.
    fn is_candidate(&self) -> bool {
        (self.has_aggregate && self.has_dimensioned) || self.min_mask != self.max_mask
    }
}

#[derive(diesel::QueryableByName)]
struct DimensionCellRow {
    #[diesel(sql_type = SqlUuid)]
    platform_id: Uuid,
    #[diesel(sql_type = SqlUuid)]
    measure_id: Uuid,
    #[diesel(sql_type = Date)]
    day: NaiveDate,
    #[diesel(sql_type = Text)]
    total: String,
    #[diesel(sql_type = Bool)]
    ambiguous: bool,
    #[diesel(sql_type = Bool)]
    depends_on_country: bool,
    #[diesel(sql_type = Bool)]
    depends_on_institution: bool,
}

#[derive(diesel::QueryableByName)]
struct AssertionRow {
    #[diesel(sql_type = SqlUuid)]
    publisher_id: Uuid,
    #[diesel(sql_type = SqlUuid)]
    platform_id: Uuid,
    #[diesel(sql_type = SqlUuid)]
    measure_id: Uuid,
    #[diesel(sql_type = Date)]
    day: NaiveDate,
    #[diesel(sql_type = crate::schema::sql_types::MetricCoverageStatus)]
    coverage_status: MetricCoverageStatus,
    #[diesel(sql_type = Text)]
    import_status: String,
    #[diesel(sql_type = Bool)]
    country_coverage: bool,
    #[diesel(sql_type = Bool)]
    institution_coverage: bool,
}

#[derive(diesel::QueryableByName)]
struct IdentifierQualityRow {
    #[diesel(sql_type = SqlUuid)]
    platform_id: Uuid,
    #[diesel(sql_type = SqlUuid)]
    measure_id: Uuid,
    #[diesel(sql_type = Date)]
    period_start: NaiveDate,
    #[diesel(sql_type = Date)]
    period_end: NaiveDate,
}

// ---------------------------------------------------------------------------
// Statements
//
// Each statement is named so the query-plan evidence explains exactly the text
// the read executes. Every one is set-based over the whole request: the
// resolved work set, the served platforms and measures and the day ranges are
// passed as arrays and bounds, never iterated, so the statement count does not
// depend on how many publishers, works, days, months, countries or
// institutions are involved.
// ---------------------------------------------------------------------------

/// Step 0: plan every later statement of the transaction for its actual
/// arguments. A generic prepared plan for the large `work_id = ANY(...)`
/// scopes was measured 1.7 times slower by `MET-WP4-03-BENCH-01`; `LOCAL`
/// confines the setting to this read-only transaction.
pub(crate) const CUSTOM_PLANS_SQL: &str = "SET LOCAL plan_cache_mode = force_custom_plan";

/// Steps 1 and 2: the transaction timestamp and the `MET-WP4-01` frontier,
/// with each selected publisher that exists and its package. The singleton
/// frontier row is always returned, once per existing publisher or once with
/// `NULL` publisher columns when none exists.
pub(crate) const PUBLISHERS_SQL: &str = "SELECT transaction_timestamp() AS as_of, \
                s.applied_through_sequence, s.next_sequence, s.watermark_at, \
                p.publisher_id, p.subscription_package \
         FROM public.metric_rollup_work_day_state s \
         LEFT JOIN public.publisher p ON p.publisher_id = ANY($1) \
         WHERE s.state_id = 1";

/// Step 3: the metadata selector, resolved to works through current Thoth
/// metadata.
///
/// The first branch names the selector dimension of every explicit ID that
/// does not exist at all. The second resolves the works of the selected
/// publishers (through current `work -> imprint` ownership) that satisfy
/// every supplied list — OR within a list, AND between lists — and returns
/// each with its publisher. It stops after `$9` rows, one more than the
/// resolved-work bound, so an oversized selection is detected without being
/// read in full; nothing is served from a stopped read.
///
/// A series matches a work issued in it, or a book chapter whose current
/// `is-child-of` parent is issued in it. A language matches any language
/// record of the work. A funding institution matches any funding of the
/// work, and an affiliation institution any affiliation of any of its
/// contributions. Work types and languages arrive as `text[]` labels and are
/// cast to their enums here.
pub(crate) const WORKS_SQL: &str = "WITH unknown AS ( \
                 SELECT 'imprintIds' AS dimension FROM unnest($2::uuid[]) AS x(id) \
                  WHERE NOT EXISTS (SELECT 1 FROM public.imprint i WHERE i.imprint_id = x.id) \
                 UNION ALL \
                 SELECT 'seriesIds' FROM unnest($3::uuid[]) AS x(id) \
                  WHERE NOT EXISTS (SELECT 1 FROM public.series s WHERE s.series_id = x.id) \
                 UNION ALL \
                 SELECT 'workIds' FROM unnest($4::uuid[]) AS x(id) \
                  WHERE NOT EXISTS (SELECT 1 FROM public.work w WHERE w.work_id = x.id) \
                 UNION ALL \
                 SELECT 'fundingInstitutionIds' FROM unnest($7::uuid[]) AS x(id) \
                  WHERE NOT EXISTS (SELECT 1 FROM public.institution n \
                                    WHERE n.institution_id = x.id) \
                 UNION ALL \
                 SELECT 'affiliationInstitutionIds' FROM unnest($8::uuid[]) AS x(id) \
                  WHERE NOT EXISTS (SELECT 1 FROM public.institution n \
                                    WHERE n.institution_id = x.id) \
             ), \
             resolved AS ( \
                 SELECT w.work_id, i.publisher_id \
                 FROM public.work w \
                 JOIN public.imprint i ON i.imprint_id = w.imprint_id \
                 WHERE i.publisher_id = ANY($1) \
                   AND (cardinality($2::uuid[]) = 0 OR w.imprint_id = ANY($2)) \
                   AND (cardinality($4::uuid[]) = 0 OR w.work_id = ANY($4)) \
                   AND (cardinality($5::text[]) = 0 \
                        OR w.work_type = ANY($5::text[]::public.work_type[])) \
                   AND (cardinality($3::uuid[]) = 0 \
                        OR EXISTS (SELECT 1 FROM public.issue s \
                                   WHERE s.work_id = w.work_id AND s.series_id = ANY($3)) \
                        OR (w.work_type = 'book-chapter' \
                            AND EXISTS (SELECT 1 FROM public.work_relation wr \
                                        JOIN public.issue s ON s.work_id = wr.related_work_id \
                                        WHERE wr.relator_work_id = w.work_id \
                                          AND wr.relation_type = 'is-child-of' \
                                          AND s.series_id = ANY($3)))) \
                   AND (cardinality($6::text[]) = 0 \
                        OR EXISTS (SELECT 1 FROM public.language l \
                                   WHERE l.work_id = w.work_id \
                                     AND l.language_code \
                                         = ANY($6::text[]::public.language_code[]))) \
                   AND (cardinality($7::uuid[]) = 0 \
                        OR EXISTS (SELECT 1 FROM public.funding f \
                                   WHERE f.work_id = w.work_id \
                                     AND f.institution_id = ANY($7))) \
                   AND (cardinality($8::uuid[]) = 0 \
                        OR EXISTS (SELECT 1 FROM public.contribution c \
                                   JOIN public.affiliation a \
                                     ON a.contribution_id = c.contribution_id \
                                   WHERE c.work_id = w.work_id \
                                     AND a.institution_id = ANY($8))) \
                 LIMIT $9 \
             ) \
             SELECT u.dimension AS unknown, NULL::uuid AS work_id, NULL::uuid AS publisher_id \
             FROM unknown u \
             UNION ALL \
             SELECT NULL::text, r.work_id, r.publisher_id FROM resolved r";

/// Step 4: which explicitly selected platforms and measures exist, with the
/// additivity of every explicit measure.
pub(crate) const KNOWN_SQL: &str = "SELECT 'PLATFORM' AS kind, p.platform_id AS id, \
                    NULL::boolean AS additive_across_time, \
                    NULL::boolean AS additive_across_works \
             FROM public.metric_platform p \
             WHERE p.platform_id = ANY($1) \
             UNION ALL \
             SELECT 'MEASURE', m.measure_id, m.additive_across_time, m.additive_across_works \
             FROM public.metric_measure m \
             WHERE m.measure_id = ANY($2)";

/// Step 4: the platform/measure pairs represented for the resolved works and
/// range, with each measure's additivity: on an edge day in the work-day
/// projection, in a complete month in the monthly projection or as a
/// total-ambiguous month (together exactly the months with any work-day
/// row), in terminal coverage from an eligible managed source account of a
/// represented publisher whose import that account and publisher own, or in
/// identifier-unresolved quarantine admitted under a represented publisher's
/// import (`MET-WP7-PREREQ-04`, scoped as [`IDENTIFIER_QUALITY_SQL`]).
pub(crate) const REPRESENTED_SQL: &str = "SELECT represented.platform_id, represented.measure_id, \
                    m.additive_across_time, m.additive_across_works \
                 FROM ( \
                     SELECT r.platform_id, r.measure_id \
                     FROM public.metric_rollup_work_day r \
                     WHERE r.work_id = ANY($1) \
                       AND ((r.day >= $2 AND r.day < $3) OR (r.day >= $4 AND r.day < $5)) \
                       AND (cardinality($6::uuid[]) = 0 OR r.platform_id = ANY($6)) \
                       AND (cardinality($7::uuid[]) = 0 OR r.measure_id = ANY($7)) \
                     UNION \
                     SELECT mm.platform_id, mm.measure_id \
                     FROM public.metric_rollup_work_month mm \
                     WHERE mm.work_id = ANY($1) \
                       AND mm.month_start >= $3 \
                       AND mm.month_start < $4 \
                       AND (cardinality($6::uuid[]) = 0 OR mm.platform_id = ANY($6)) \
                       AND (cardinality($7::uuid[]) = 0 OR mm.measure_id = ANY($7)) \
                     UNION \
                     SELECT a.platform_id, a.measure_id \
                     FROM public.metric_rollup_work_month_ambiguity a \
                     WHERE a.work_id = ANY($1) \
                       AND a.month_start >= $3 \
                       AND a.month_start < $4 \
                       AND a.total_ambiguous \
                       AND (cardinality($6::uuid[]) = 0 OR a.platform_id = ANY($6)) \
                       AND (cardinality($7::uuid[]) = 0 OR a.measure_id = ANY($7)) \
                     UNION \
                     SELECT c.platform_id, c.measure_id \
                     FROM public.metric_coverage c \
                     JOIN public.metric_source_account sa \
                       ON sa.source_account_id = c.source_account_id \
                      AND sa.platform_id = c.platform_id \
                     JOIN public.metric_import mi \
                       ON mi.import_id = c.import_id \
                      AND mi.source_account_id = c.source_account_id \
                      AND mi.publisher_id = sa.expected_publisher_id \
                     JOIN public.metric_source s ON s.source_id = sa.source_id \
                     WHERE sa.expected_publisher_id = ANY($8) \
                       AND sa.enabled \
                       AND s.enabled \
                       AND s.acquisition_type::text = $9 \
                       AND mi.status::text IN ('COMPLETED', 'COMPLETED_WITH_ERRORS') \
                       AND c.period_start < $5 \
                       AND c.period_end > $2 \
                       AND (cardinality($6::uuid[]) = 0 OR c.platform_id = ANY($6)) \
                       AND (cardinality($7::uuid[]) = 0 OR c.measure_id = ANY($7)) \
                     UNION \
                     SELECT q.platform_id, q.measure_id \
                     FROM public.metric_identifier_quarantine q \
                     JOIN public.metric_record_provenance p \
                       ON p.record_provenance_id = q.record_provenance_id \
                     JOIN public.metric_import mi ON mi.import_id = p.import_id \
                     LEFT JOIN public.metric_identifier_quarantine_reconciliation r \
                       ON r.identifier_quarantine_id = q.identifier_quarantine_id \
                     WHERE mi.publisher_id = ANY($8) \
                       AND q.period_start < $5 \
                       AND q.period_end > $2 \
                       AND (cardinality($6::uuid[]) = 0 OR q.platform_id = ANY($6)) \
                       AND (cardinality($7::uuid[]) = 0 OR q.measure_id = ANY($7)) \
                       AND r.resolved_at IS NULL \
                 ) represented \
                 JOIN public.metric_measure m ON m.measure_id = represented.measure_id";

/// Step 5: eligible managed source accounts per represented publisher and
/// served platform.
pub(crate) const ACCOUNTS_SQL: &str =
    "SELECT sa.expected_publisher_id AS publisher_id, sa.platform_id, \
                    sa.source_account_id \
             FROM public.metric_source_account sa \
             JOIN public.metric_source s ON s.source_id = sa.source_id \
             WHERE sa.expected_publisher_id = ANY($1) \
               AND sa.platform_id = ANY($2) \
               AND sa.enabled \
               AND s.enabled \
               AND s.acquisition_type::text = $3";

/// Step 6: canonical state intersecting the request that the projections do
/// not settle.
///
/// The `LAG` branch, evaluated only when `$1` says the frontier `W` (`$2`)
/// has not caught up, lists the platform, measure and day of every unapplied
/// work-day delta above `W` for a resolved work in the range. Every position
/// above `W` is unapplied under the strict frontier, so no status filter is
/// needed.
///
/// The grain branch finds canonical records of a native `MONTH` or
/// `REPORTING_PERIOD` grain for a resolved work, served platform and served
/// measure whose half-open period overlaps the range, and classifies each by
/// following `current_revision_id` to its revision:
///
/// - the pointer names a `CURRENT` revision: an active unsupported
///   observation (`ACTIVE_NATIVE_GRAIN`);
/// - the pointer names a `RETRACTED` revision and the record has no
///   `CURRENT` revision: withdrawn, and not returned at all;
/// - anything else — no pointer, a pointer that resolves to no revision of
///   the record, a pointer to a `SUPERSEDED` revision, or a pointer to a
///   `RETRACTED` revision beside a `CURRENT` one — is committed canonical
///   state that contradicts itself (`INCONSISTENT_NATIVE_GRAIN`).
pub(crate) const CANONICAL_SQL: &str =
    "SELECT 'LAG' AS kind, lag.platform_id, lag.measure_id, lag.day \
             FROM ( \
                 SELECT DISTINCT r.platform_id, r.measure_id, r.period_start AS day \
                 FROM public.metric_rollup_delta d \
                 JOIN public.metric_record r ON r.record_id = d.record_id \
                 WHERE $1 \
                   AND d.work_day_sequence > $2 \
                   AND r.work_id = ANY($3) \
                   AND r.period_start >= $4 \
                   AND r.period_start < $5 \
                   AND r.platform_id = ANY($6) \
                   AND r.measure_id = ANY($7) \
             ) lag \
             UNION ALL \
             SELECT DISTINCT \
                    CASE WHEN v.status = 'CURRENT' THEN 'ACTIVE_NATIVE_GRAIN' \
                         ELSE 'INCONSISTENT_NATIVE_GRAIN' END, \
                    NULL::uuid, NULL::uuid, NULL::date \
             FROM public.metric_record r \
             LEFT JOIN public.metric_record_revision v \
               ON v.record_id = r.record_id \
              AND v.record_revision_id = r.current_revision_id \
             WHERE r.work_id = ANY($3) \
               AND r.platform_id = ANY($6) \
               AND r.measure_id = ANY($7) \
               AND r.period_start < $5 \
               AND r.period_end > $4 \
               AND r.reporting_grain IN ('MONTH', 'REPORTING_PERIOD') \
               AND NOT (COALESCE(v.status = 'RETRACTED', FALSE) \
                        AND NOT EXISTS (SELECT 1 FROM public.metric_record_revision o \
                                        WHERE o.record_id = r.record_id \
                                          AND o.status = 'CURRENT'))";

/// Step 7: the complete months, from the `MET-WP4-03A` monthly projections.
///
/// Every monthly row already holds its work's resolved daily contributions,
/// so the rows are only summed, never re-resolved or re-ranked: a `NULL` and
/// a publication-specific row of one month are both added. The branches
/// return, for the resolved works, served platforms and measures and months
/// in `[$4, $5)`:
///
/// - `TOTAL`: each platform/measure/month total with the OR of its
///   country and institution coverage dependencies;
/// - `COUNTRY`, only when `$6`: each platform/measure/country total over the
///   months with the OR of its institution dependency;
/// - `INSTITUTION`, only when `$7`: each platform/measure/institution total
///   over the months with the OR of its country dependency and the
///   institution's current name and ROR, stopping after `$8` rows;
/// - `AMBIGUITY`: one summary row saying whether any of those months is
///   total-ambiguous, or country- or institution-ambiguous in a section that
///   is requested.
pub(crate) const MONTHS_SQL: &str = "SELECT 'TOTAL' AS section, m.platform_id, m.measure_id, \
                    m.month_start, NULL::text AS country_code, NULL::uuid AS institution_id, \
                    NULL::text AS institution_name, NULL::text AS ror, \
                    SUM(m.value)::text AS total, \
                    bool_or(m.requires_country_coverage) AS requires_country, \
                    bool_or(m.requires_institution_coverage) AS requires_institution, \
                    FALSE AS total_ambiguous, FALSE AS country_ambiguous, \
                    FALSE AS institution_ambiguous \
             FROM public.metric_rollup_work_month m \
             WHERE m.work_id = ANY($1) \
               AND m.platform_id = ANY($2) \
               AND m.measure_id = ANY($3) \
               AND m.month_start >= $4 \
               AND m.month_start < $5 \
             GROUP BY m.platform_id, m.measure_id, m.month_start \
             UNION ALL \
             SELECT 'COUNTRY', c.platform_id, c.measure_id, NULL::date, c.country_code::text, \
                    NULL::uuid, NULL::text, NULL::text, SUM(c.value)::text, FALSE, \
                    bool_or(c.requires_institution_coverage), FALSE, FALSE, FALSE \
             FROM public.metric_rollup_work_country_month c \
             WHERE $6 \
               AND c.work_id = ANY($1) \
               AND c.platform_id = ANY($2) \
               AND c.measure_id = ANY($3) \
               AND c.month_start >= $4 \
               AND c.month_start < $5 \
             GROUP BY c.platform_id, c.measure_id, c.country_code \
             UNION ALL \
             SELECT 'INSTITUTION', v.platform_id, v.measure_id, NULL::date, NULL::text, \
                    v.institution_id, n.institution_name, n.ror, v.total, \
                    v.requires_country, FALSE, FALSE, FALSE, FALSE \
             FROM ( \
                 SELECT i.platform_id, i.measure_id, i.institution_id, \
                        SUM(i.value)::text AS total, \
                        bool_or(i.requires_country_coverage) AS requires_country \
                 FROM public.metric_rollup_work_institution_month i \
                 WHERE $7 \
                   AND i.work_id = ANY($1) \
                   AND i.platform_id = ANY($2) \
                   AND i.measure_id = ANY($3) \
                   AND i.month_start >= $4 \
                   AND i.month_start < $5 \
                 GROUP BY i.platform_id, i.measure_id, i.institution_id \
                 LIMIT $8 \
             ) v \
             JOIN public.institution n ON n.institution_id = v.institution_id \
             UNION ALL \
             SELECT 'AMBIGUITY', NULL::uuid, NULL::uuid, NULL::date, NULL::text, NULL::uuid, \
                    NULL::text, NULL::text, NULL::text, FALSE, FALSE, \
                    COALESCE(bool_or(a.total_ambiguous), FALSE), \
                    COALESCE(bool_or($6 AND a.country_ambiguous), FALSE), \
                    COALESCE(bool_or($7 AND a.institution_ambiguous), FALSE) \
             FROM public.metric_rollup_work_month_ambiguity a \
             WHERE a.work_id = ANY($1) \
               AND a.platform_id = ANY($2) \
               AND a.measure_id = ANY($3) \
               AND a.month_start >= $4 \
               AND a.month_start < $5";

/// Step 8a: projected values per platform, measure and day for the day
/// ranges `[$2, $3)` and `[$4, $5)` of the resolved works, with the facts
/// the MOM-1 dimensional representation rule (Specification Amendment 6)
/// needs.
///
/// A row's presence mask records which optional dimensions it carries:
/// publication = 4, country = 2, institution = 1, so an undimensioned row is
/// `0`. Most groups are settled here without looking at individual works:
///
/// - only undimensioned rows: each is its base cell's aggregate, and `total`
///   is their sum;
/// - only dimensioned rows sharing one mask: no base cell can hold two masks,
///   and `total` is their sum.
///
/// A group mixing undimensioned and dimensioned rows, or carrying different
/// masks, is a candidate: its value depends on how rows fall into base cells,
/// and [`DIMENSION_CELLS_SQL`] resolves exactly those groups.
pub(crate) const DAY_SUMS_SQL: &str = "SELECT r.platform_id, r.measure_id, r.day, \
                    SUM(r.value)::text AS total, \
                    bool_or(((r.publication_id IS NOT NULL)::int * 4 \
                        + (r.country_code IS NOT NULL)::int * 2 \
                        + (r.institution_id IS NOT NULL)::int) = 0) AS has_aggregate, \
                    bool_or(((r.publication_id IS NOT NULL)::int * 4 \
                        + (r.country_code IS NOT NULL)::int * 2 \
                        + (r.institution_id IS NOT NULL)::int) <> 0) AS has_dimensioned, \
                    MIN(((r.publication_id IS NOT NULL)::int * 4 \
                        + (r.country_code IS NOT NULL)::int * 2 \
                        + (r.institution_id IS NOT NULL)::int)) AS min_mask, \
                    MAX(((r.publication_id IS NOT NULL)::int * 4 \
                        + (r.country_code IS NOT NULL)::int * 2 \
                        + (r.institution_id IS NOT NULL)::int)) AS max_mask, \
                    bool_or(r.country_code IS NOT NULL) AS uses_country, \
                    bool_or(r.institution_id IS NOT NULL) AS uses_institution \
             FROM public.metric_rollup_work_day r \
             WHERE r.work_id = ANY($1) \
               AND ((r.day >= $2 AND r.day < $3) OR (r.day >= $4 AND r.day < $5)) \
               AND r.platform_id = ANY($6) \
               AND r.measure_id = ANY($7) \
             GROUP BY r.platform_id, r.measure_id, r.day";

/// Step 8b: the base-cell resolution of the candidate groups step 8a could not
/// settle, read only when there is at least one.
///
/// The inner query groups each candidate group's rows by base cell
/// `(work_id, platform_id, measure_id, day)`. Within one base cell an
/// undimensioned row (`min_mask = 0`) is the aggregate and no dimensioned row
/// of that cell is added to it; otherwise every row must share one mask
/// (`min_mask = max_mask`) and they are summed; otherwise the cell is
/// ambiguous. The outer query sums base cells per group and reports whether
/// any was ambiguous or took its value from country or institution rows. The
/// candidate keys are passed as three parallel arrays, so this is one
/// set-based statement whatever the number of candidates. The same arrays are
/// restated as `= ANY` predicates; they are implied by the join, but they let
/// the planner estimate how many projection rows the candidates select.
pub(crate) const DIMENSION_CELLS_SQL: &str = "SELECT cell.platform_id, cell.measure_id, cell.day, \
                    SUM(CASE WHEN cell.min_mask = 0 THEN cell.aggregate_total \
                             ELSE cell.total END)::text AS total, \
                    bool_or(cell.min_mask <> 0 AND cell.min_mask <> cell.max_mask) AS ambiguous, \
                    bool_or(cell.min_mask <> 0 AND cell.min_mask & 2 <> 0) AS depends_on_country, \
                    bool_or(cell.min_mask <> 0 AND cell.min_mask & 1 <> 0) \
                        AS depends_on_institution \
             FROM ( \
                 SELECT r.platform_id, r.measure_id, r.day, \
                        MIN(((r.publication_id IS NOT NULL)::int * 4 \
                        + (r.country_code IS NOT NULL)::int * 2 \
                        + (r.institution_id IS NOT NULL)::int)) AS min_mask, \
                        MAX(((r.publication_id IS NOT NULL)::int * 4 \
                        + (r.country_code IS NOT NULL)::int * 2 \
                        + (r.institution_id IS NOT NULL)::int)) AS max_mask, \
                        SUM(r.value) FILTER (WHERE ((r.publication_id IS NOT NULL)::int * 4 \
                        + (r.country_code IS NOT NULL)::int * 2 \
                        + (r.institution_id IS NOT NULL)::int) = 0) AS aggregate_total, \
                        SUM(r.value) AS total \
                 FROM public.metric_rollup_work_day r \
                 JOIN unnest($2::uuid[], $3::uuid[], $4::date[]) \
                      AS candidate(platform_id, measure_id, day) \
                   ON candidate.platform_id = r.platform_id \
                  AND candidate.measure_id = r.measure_id \
                  AND candidate.day = r.day \
                 WHERE r.work_id = ANY($1) \
                   AND r.platform_id = ANY($2) \
                   AND r.measure_id = ANY($3) \
                   AND r.day = ANY($4) \
                 GROUP BY r.platform_id, r.measure_id, r.day, r.work_id \
             ) cell \
             GROUP BY cell.platform_id, cell.measure_id, cell.day";

/// Step 9: the clipped edge days' country and institution values, resolved
/// per base cell exactly as `MET-WP4-03A` resolves each day before summing
/// it into a month.
///
/// Only rows carrying a country or an institution are read: the target masks
/// are country `{2, 3, 6, 7}` and institution `{1, 3, 5, 7}`, and rows with
/// neither (masks 0 and 4) can never change which of them is selected. A
/// window `bit_or(1 << mask) OVER (PARTITION BY base cell)` gives every row
/// the bitmap of its cell's masks, and the least target-bearing mask under
/// set inclusion is selected: for country `2`, else `NULL` when both `3` and
/// `6` (incomparable minima) are present, else `3`, `6`, `7`; for institution
/// the mirror over `1`, `3`, `5`, `7`. The rows carrying the selected mask are
/// summed per platform, measure and country or institution, with the OR of
/// their other-dimension dependency. `COUNTRY` rows are returned only when
/// `$8`, `INSTITUTION` rows only when `$9` (stopping after `$10`), and one
/// `AMBIGUITY` summary says whether a requested section met incomparable
/// minima.
pub(crate) const DAY_SECTIONS_SQL: &str = "WITH day_rows AS ( \
                 SELECT x.*, \
                        bit_or(1 << x.mask) OVER (PARTITION BY x.work_id, x.platform_id, \
                                                               x.measure_id, x.day) \
                            AS represented \
                 FROM ( \
                     SELECT r.work_id, r.platform_id, r.measure_id, r.day, r.country_code, \
                            r.institution_id, r.value, \
                            (r.publication_id IS NOT NULL)::int * 4 \
                                + (r.country_code IS NOT NULL)::int * 2 \
                                + (r.institution_id IS NOT NULL)::int AS mask \
                     FROM public.metric_rollup_work_day r \
                     WHERE r.work_id = ANY($1) \
                       AND ((r.day >= $2 AND r.day < $3) OR (r.day >= $4 AND r.day < $5)) \
                       AND r.platform_id = ANY($6) \
                       AND r.measure_id = ANY($7) \
                       AND (r.country_code IS NOT NULL OR r.institution_id IS NOT NULL) \
                 ) x \
             ), \
             resolved AS ( \
                 SELECT d.*, \
                        CASE WHEN d.represented & 4 <> 0 THEN 2 \
                             WHEN d.represented & 8 <> 0 AND d.represented & 64 <> 0 THEN NULL \
                             WHEN d.represented & 8 <> 0 THEN 3 \
                             WHEN d.represented & 64 <> 0 THEN 6 \
                             WHEN d.represented & 128 <> 0 THEN 7 \
                        END AS country_mask, \
                        (d.represented & 4 = 0 AND d.represented & 8 <> 0 \
                         AND d.represented & 64 <> 0) AS country_ambiguous, \
                        CASE WHEN d.represented & 2 <> 0 THEN 1 \
                             WHEN d.represented & 8 <> 0 AND d.represented & 32 <> 0 THEN NULL \
                             WHEN d.represented & 8 <> 0 THEN 3 \
                             WHEN d.represented & 32 <> 0 THEN 5 \
                             WHEN d.represented & 128 <> 0 THEN 7 \
                        END AS institution_mask, \
                        (d.represented & 2 = 0 AND d.represented & 8 <> 0 \
                         AND d.represented & 32 <> 0) AS institution_ambiguous \
                 FROM day_rows d \
             ) \
             SELECT 'COUNTRY' AS section, r.platform_id, r.measure_id, NULL::date AS month_start, \
                    r.country_code::text AS country_code, NULL::uuid AS institution_id, \
                    NULL::text AS institution_name, NULL::text AS ror, \
                    SUM(r.value)::text AS total, FALSE AS requires_country, \
                    bool_or(r.mask & 1 <> 0) AS requires_institution, \
                    FALSE AS total_ambiguous, FALSE AS country_ambiguous, \
                    FALSE AS institution_ambiguous \
             FROM resolved r \
             WHERE $8 AND r.mask = r.country_mask \
             GROUP BY r.platform_id, r.measure_id, r.country_code \
             UNION ALL \
             SELECT 'INSTITUTION', v.platform_id, v.measure_id, NULL::date, NULL::text, \
                    v.institution_id, n.institution_name, n.ror, v.total, \
                    v.requires_country, FALSE, FALSE, FALSE, FALSE \
             FROM ( \
                 SELECT r.platform_id, r.measure_id, r.institution_id, \
                        SUM(r.value)::text AS total, \
                        bool_or(r.mask & 2 <> 0) AS requires_country \
                 FROM resolved r \
                 WHERE $9 AND r.mask = r.institution_mask \
                 GROUP BY r.platform_id, r.measure_id, r.institution_id \
                 LIMIT $10 \
             ) v \
             JOIN public.institution n ON n.institution_id = v.institution_id \
             UNION ALL \
             SELECT 'AMBIGUITY', NULL::uuid, NULL::uuid, NULL::date, NULL::text, NULL::uuid, \
                    NULL::text, NULL::text, NULL::text, FALSE, FALSE, FALSE, \
                    COALESCE(bool_or($8 AND r.country_ambiguous), FALSE), \
                    COALESCE(bool_or($9 AND r.institution_ambiguous), FALSE) \
             FROM resolved r";

/// Step 10: the current terminal coverage assertion per represented
/// publisher, platform, measure and day.
///
/// Only `COMPLETED` and `COMPLETED_WITH_ERRORS` imports with a recorded
/// completion time count, and only when the import belongs to the coverage
/// row's account and that account's expected publisher: the schema lets a
/// coverage row reference any import, so a row whose import is owned elsewhere
/// is not evidence. The latest completion wins, then the greatest import id
/// under PostgreSQL UUID ordering; the remaining keys only make a
/// self-contradictory import deterministic, preferring its most conservative
/// assertion.
///
/// The day series is an explicitly `MATERIALIZED` CTE: inlined, PostgreSQL
/// re-evaluates it once per coverage row, which `MET-WP4-03-BENCH-01`
/// measured about four times slower. The ownership check is wrapped in a
/// `CASE` that is `TRUE` only when both equalities hold, so it rejects
/// exactly what they reject, NULL included; as plain equalities the planner
/// multiplies in their selectivity and misjudges the coverage join.
pub(crate) const ASSERTIONS_SQL: &str = "WITH d AS MATERIALIZED ( \
                     SELECT generate_series($2::date, $3::date - 1, interval '1 day')::date \
                         AS day \
                 ) \
                 SELECT DISTINCT ON (sa.expected_publisher_id, c.platform_id, c.measure_id, \
                                     d.day) \
                        sa.expected_publisher_id AS publisher_id, \
                        c.platform_id, c.measure_id, d.day, c.coverage_status, \
                        mi.status::text AS import_status, \
                        c.country_coverage, c.institution_coverage \
                 FROM d \
                 JOIN public.metric_coverage c \
                   ON c.period_start <= d.day AND c.period_end > d.day \
                 JOIN public.metric_source_account sa \
                   ON sa.source_account_id = c.source_account_id \
                  AND sa.platform_id = c.platform_id \
                 JOIN public.metric_import mi \
                   ON mi.import_id = c.import_id \
                  AND CASE \
                          WHEN mi.source_account_id = c.source_account_id \
                           AND mi.publisher_id = sa.expected_publisher_id \
                          THEN TRUE \
                          ELSE FALSE \
                      END \
                 WHERE c.source_account_id = ANY($1) \
                   AND c.platform_id = ANY($4) \
                   AND c.measure_id = ANY($5) \
                   AND mi.status::text IN ('COMPLETED', 'COMPLETED_WITH_ERRORS') \
                   AND mi.completed_at IS NOT NULL \
                 ORDER BY sa.expected_publisher_id, c.platform_id, c.measure_id, d.day, \
                          mi.completed_at DESC, mi.import_id DESC, \
                          c.coverage_status DESC, c.country_coverage ASC, \
                          c.institution_coverage ASC, c.coverage_id DESC";

/// Step 11 (`MET-WP7-PREREQ-04`): identifier-unresolved quarantine evidence
/// intersecting the served publisher, platform, measure and date scope.
/// Historical import publisher scope remains authoritative and source
/// enablement is deliberately ignored. Absence of reconciliation state and
/// every nonterminal state both have `resolved_at IS NULL`; terminal
/// resolution removes the warning.
///
/// `$1` is the represented publisher set (#946 Amendment 2): the publishers
/// owning at least one resolved work. The evidence has no work identity, so
/// it is never matched to the resolved works or reinterpreted through current
/// metadata.
pub(crate) const IDENTIFIER_QUALITY_SQL: &str =
    "SELECT DISTINCT q.platform_id, q.measure_id, q.period_start, q.period_end \
             FROM public.metric_identifier_quarantine q \
             JOIN public.metric_record_provenance p \
               ON p.record_provenance_id = q.record_provenance_id \
             JOIN public.metric_import mi ON mi.import_id = p.import_id \
             LEFT JOIN public.metric_identifier_quarantine_reconciliation r \
               ON r.identifier_quarantine_id = q.identifier_quarantine_id \
             WHERE mi.publisher_id = ANY($1) \
               AND q.period_start < $3 \
               AND q.period_end > $2 \
               AND q.platform_id = ANY($4) \
               AND q.measure_id = ANY($5) \
               AND r.resolved_at IS NULL";

// ---------------------------------------------------------------------------
// The read
// ---------------------------------------------------------------------------

/// Answer one dashboard request.
///
/// The caller has already passed `METRICS_READ_SERVICE`. Checks that need no
/// database run first; everything else runs in one `READ ONLY, REPEATABLE
/// READ` transaction on one connection, in this order:
///
/// 0. plan every statement for its actual arguments;
/// 1. resolve the selected publishers and require `METRICS_DASHBOARD` for
///    each through the ADR-0001 package model, and, in the same statement,
/// 2. read the `MET-WP4-01` frontier `W`, `next_sequence` and `watermark_at`,
///    together with the transaction timestamp;
/// 3. check every explicit selector ID and resolve the works, and with them
///    the represented publishers;
/// 4. resolve and bound the platform/measure scope, and require every served
///    measure to be additive across time and works;
/// 5. resolve the eligible managed source account per represented publisher
///    and platform;
/// 6. refuse active native-grain canonical records, and find unapplied
///    work-day deltas above `W`, for the resolved works;
/// 7. read the complete months from the monthly projections;
/// 8. sum the work-day projection per platform, measure and day over the day
///    ranges, resolving by base cell only the groups whose dimensional
///    representation the group sums alone cannot settle;
/// 9. read the edge days' country and institution values;
/// 10. select the current terminal coverage assertion per represented
///     publisher, platform, measure and day;
/// 11. mark unresolved identifier evidence of the represented publishers
///     under immutable import-publisher scope, without consulting current
///     DOI resolution, work metadata or source enablement.
///
/// Every statement is set-based over the whole request, so the number of
/// statements does not depend on how many publishers, works, days, months,
/// countries, institutions or rows are involved: at most fifteen, including
/// transaction control, run for any request, and sixteen counting the pool's
/// own connection check on checkout.
pub(crate) fn metric_dashboard(
    db: &PgPool,
    input: &MetricDashboardInput,
) -> Result<MetricDashboard, MetricReadError> {
    let request = validate(input)?;
    let mut connection = db.get()?;
    connection
        .build_transaction()
        .read_only()
        .repeatable_read()
        .run(|connection| evaluate(connection, &request))
}

/// The works a request resolved to, and the publishers they represent.
struct WorkScope {
    works: Vec<Uuid>,
    publishers: Vec<Uuid>,
}

fn evaluate(
    connection: &mut PgConnection,
    request: &ValidatedRequest,
) -> Result<MetricDashboard, MetricReadError> {
    // 0. Custom plans for this transaction only.
    diesel::sql_query(CUSTOM_PLANS_SQL).execute(connection)?;

    // 1. Publishers and entitlement, with (2) the rollup frontier read in
    // the same snapshot as everything else.
    let rows: Vec<PublisherRow> = diesel::sql_query(PUBLISHERS_SQL)
        .bind::<Array<SqlUuid>, _>(&request.publisher_ids)
        .load(connection)?;
    let frontier = rows
        .first()
        .map(|row| FrontierRow {
            as_of: row.as_of,
            applied_through_sequence: row.applied_through_sequence,
            next_sequence: row.next_sequence,
            watermark_at: row.watermark_at,
        })
        .ok_or(MetricReadError::Unavailable)?;
    let publishers: Vec<(Uuid, ThothPackage)> = rows
        .into_iter()
        .filter_map(|row| row.publisher_id.zip(row.subscription_package))
        .collect();
    let found: BTreeSet<Uuid> = publishers
        .iter()
        .map(|(publisher_id, _)| *publisher_id)
        .collect();
    if request
        .publisher_ids
        .iter()
        .any(|publisher_id| !found.contains(publisher_id))
    {
        return Err(MetricReadError::QueryInvalid(UNKNOWN_PUBLISHER));
    }
    // The capability comes from each publisher's package alone. Holding
    // METRICS_READ_SERVICE was checked before this function and contributes
    // nothing here, and one unentitled publisher refuses the whole request.
    if publishers
        .iter()
        .any(|(_, package)| !package.has_capability(PublisherCapability::MetricsDashboard))
    {
        return Err(MetricReadError::Unauthorised);
    }

    // 3. The metadata selector.
    let scope = resolve_works(connection, request)?;

    // 4. Platforms and measures.
    let (platforms, measures) = resolve_scope(connection, request, &scope)?;
    let combinations: Vec<(Uuid, Uuid)> = platforms
        .iter()
        .flat_map(|platform_id| {
            measures
                .iter()
                .map(move |measure_id| (*platform_id, *measure_id))
        })
        .collect();
    if combinations.len() > METRIC_DASHBOARD_MAX_COMBINATIONS {
        return Err(MetricReadError::QueryLimitExceeded(TOO_MANY_COMBINATIONS));
    }
    if combinations.len() * request.buckets.len() > METRIC_DASHBOARD_MAX_TIMELINE_CELLS {
        return Err(MetricReadError::QueryLimitExceeded(TOO_MANY_CELLS));
    }
    let platform_ids: Vec<Uuid> = platforms.iter().copied().collect();
    let measure_ids: Vec<Uuid> = measures.iter().copied().collect();

    let days = days_of(request.start, request.end);
    let months = request.split.months();
    let mut grid = Grid::new(&combinations, days.len(), months.len());
    let mut sections = Sections::default();
    if !combinations.is_empty() {
        // 5. Eligible managed source accounts of the represented publishers.
        let mut account_ids: Vec<Uuid> = Vec::new();
        if !scope.publishers.is_empty() {
            let accounts: Vec<AccountRow> = diesel::sql_query(ACCOUNTS_SQL)
                .bind::<Array<SqlUuid>, _>(&scope.publishers)
                .bind::<Array<SqlUuid>, _>(&platform_ids)
                .bind::<Text, _>(DRIVER_ACQUISITION)
                .load(connection)?;
            let mut per_scope: BTreeMap<(Uuid, Uuid), usize> = BTreeMap::new();
            for account in &accounts {
                *per_scope
                    .entry((account.publisher_id, account.platform_id))
                    .or_default() += 1;
            }
            if per_scope.values().any(|count| *count > 1) {
                return Err(MetricReadError::SourceScopeAmbiguous);
            }
            account_ids = accounts.iter().map(|row| row.source_account_id).collect();
        }

        if !scope.works.is_empty() {
            // 6. Unsupported native grains and outstanding rollup work.
            let lagging = frontier.applied_through_sequence < frontier.next_sequence - 1;
            let canonical: Vec<CanonicalRow> = diesel::sql_query(CANONICAL_SQL)
                .bind::<Bool, _>(lagging)
                .bind::<SqlBigInt, _>(frontier.applied_through_sequence)
                .bind::<Array<SqlUuid>, _>(&scope.works)
                .bind::<Date, _>(request.start)
                .bind::<Date, _>(request.end)
                .bind::<Array<SqlUuid>, _>(&platform_ids)
                .bind::<Array<SqlUuid>, _>(&measure_ids)
                .load(connection)?;
            // Contradictory canonical state is never classified as either
            // served or withdrawn.
            if canonical
                .iter()
                .any(|row| row.kind == "INCONSISTENT_NATIVE_GRAIN")
            {
                return Err(MetricReadError::Unavailable);
            }
            if canonical
                .iter()
                .any(|row| row.kind == "ACTIVE_NATIVE_GRAIN")
            {
                return Err(MetricReadError::UnsupportedSourceGrain);
            }
            for row in canonical {
                match (row.kind.as_str(), row.platform_id, row.measure_id, row.day) {
                    ("LAG", Some(platform_id), Some(measure_id), Some(day)) => {
                        grid.set_lag(platform_id, measure_id, request.start, day)
                    }
                    _ => return Err(MetricReadError::Unavailable),
                }
            }

            // 7. Complete months.
            if request.split.has_interior() {
                let rows: Vec<SectionRow> = diesel::sql_query(MONTHS_SQL)
                    .bind::<Array<SqlUuid>, _>(&scope.works)
                    .bind::<Array<SqlUuid>, _>(&platform_ids)
                    .bind::<Array<SqlUuid>, _>(&measure_ids)
                    .bind::<Date, _>(request.split.interior_start)
                    .bind::<Date, _>(request.split.interior_end)
                    .bind::<Bool, _>(request.include_countries)
                    .bind::<Bool, _>(request.include_institutions)
                    .bind::<SqlBigInt, _>(institution_read_limit())
                    .load(connection)?;
                apply_sections(rows, &months, &mut grid, &mut sections)?;
            }

            // 8. Day values.
            if request.has_day_reads() {
                read_days(
                    connection,
                    request,
                    &scope,
                    &platform_ids,
                    &measure_ids,
                    &mut grid,
                )?;
            }

            // 9. Edge days' countries and institutions.
            if request.has_edges() && (request.include_countries || request.include_institutions) {
                let (start, lead_end, trail_start, end) = request.edges();
                let rows: Vec<SectionRow> = diesel::sql_query(DAY_SECTIONS_SQL)
                    .bind::<Array<SqlUuid>, _>(&scope.works)
                    .bind::<Date, _>(start)
                    .bind::<Date, _>(lead_end)
                    .bind::<Date, _>(trail_start)
                    .bind::<Date, _>(end)
                    .bind::<Array<SqlUuid>, _>(&platform_ids)
                    .bind::<Array<SqlUuid>, _>(&measure_ids)
                    .bind::<Bool, _>(request.include_countries)
                    .bind::<Bool, _>(request.include_institutions)
                    .bind::<SqlBigInt, _>(institution_read_limit())
                    .load(connection)?;
                apply_sections(rows, &months, &mut grid, &mut sections)?;
            }
        }

        // 10. Current terminal coverage of every represented publisher.
        if !account_ids.is_empty() {
            let assertions: Vec<AssertionRow> = diesel::sql_query(ASSERTIONS_SQL)
                .bind::<Array<SqlUuid>, _>(&account_ids)
                .bind::<Date, _>(request.start)
                .bind::<Date, _>(request.end)
                .bind::<Array<SqlUuid>, _>(&platform_ids)
                .bind::<Array<SqlUuid>, _>(&measure_ids)
                .load(connection)?;
            for row in assertions {
                let status = match (row.coverage_status, row.import_status.as_str()) {
                    // A terminal import that recorded row errors cannot
                    // establish complete coverage.
                    (MetricCoverageStatus::Complete, "COMPLETED_WITH_ERRORS") => {
                        MetricCoverageStatus::Partial
                    }
                    (declared, _) => declared,
                };
                grid.set_assertion(
                    row.publisher_id,
                    row.platform_id,
                    row.measure_id,
                    request.start,
                    row.day,
                    DayCoverage {
                        status,
                        country: row.country_coverage,
                        institution: row.institution_coverage,
                    },
                );
            }
        }

        // 11. Identifier quality of the represented publishers. Historical
        // admitted evidence remains load-bearing even if its source is now
        // disabled; a selected publisher without a resolved work contributes
        // none.
        if !scope.publishers.is_empty() {
            let unresolved: Vec<IdentifierQualityRow> = diesel::sql_query(IDENTIFIER_QUALITY_SQL)
                .bind::<Array<SqlUuid>, _>(&scope.publishers)
                .bind::<Date, _>(request.start)
                .bind::<Date, _>(request.end)
                .bind::<Array<SqlUuid>, _>(&platform_ids)
                .bind::<Array<SqlUuid>, _>(&measure_ids)
                .load(connection)?;
            for row in unresolved {
                grid.set_identifier_incomplete_range(
                    row.platform_id,
                    row.measure_id,
                    request.start,
                    request.end,
                    row.period_start,
                    row.period_end,
                );
            }
        }
    }

    grid.combine_assertions(&scope.publishers);
    grid.resolve_coverage(request.include_countries, request.include_institutions);
    assemble(request, &combinations, &grid, &days, frontier, sections)
}

/// The row bound each institution branch stops after: one more than the
/// response may hold, so an oversized result is detected rather than cut.
fn institution_read_limit() -> i64 {
    METRIC_DASHBOARD_MAX_INSTITUTIONS as i64 + 1
}

/// Check the explicit selector IDs and resolve the works.
fn resolve_works(
    connection: &mut PgConnection,
    request: &ValidatedRequest,
) -> Result<WorkScope, MetricReadError> {
    let selector = &request.selector;
    let rows: Vec<WorkRow> = diesel::sql_query(WORKS_SQL)
        .bind::<Array<SqlUuid>, _>(&request.publisher_ids)
        .bind::<Array<SqlUuid>, _>(&selector.imprints)
        .bind::<Array<SqlUuid>, _>(&selector.series)
        .bind::<Array<SqlUuid>, _>(&selector.works)
        .bind::<Array<Text>, _>(&selector.work_types)
        .bind::<Array<Text>, _>(&selector.languages)
        .bind::<Array<SqlUuid>, _>(&selector.funding_institutions)
        .bind::<Array<SqlUuid>, _>(&selector.affiliation_institutions)
        .bind::<SqlBigInt, _>(METRIC_DASHBOARD_MAX_RESOLVED_WORKS as i64 + 1)
        .load(connection)?;
    let unknown: BTreeSet<&str> = rows
        .iter()
        .filter_map(|row| row.unknown.as_deref())
        .collect();
    for (dimension, message) in [
        ("imprintIds", UNKNOWN_IMPRINT),
        ("seriesIds", UNKNOWN_SERIES),
        ("workIds", UNKNOWN_WORK),
        ("fundingInstitutionIds", UNKNOWN_FUNDING),
        ("affiliationInstitutionIds", UNKNOWN_AFFILIATION),
    ] {
        if unknown.contains(dimension) {
            return Err(MetricReadError::QueryInvalid(message));
        }
    }
    if !unknown.is_empty() {
        return Err(MetricReadError::Unavailable);
    }
    let mut works = BTreeSet::new();
    let mut publishers = BTreeSet::new();
    for row in rows {
        let (Some(work_id), Some(publisher_id)) = (row.work_id, row.publisher_id) else {
            return Err(MetricReadError::Unavailable);
        };
        works.insert(work_id);
        publishers.insert(publisher_id);
    }
    if works.len() > METRIC_DASHBOARD_MAX_RESOLVED_WORKS {
        return Err(MetricReadError::QueryLimitExceeded(TOO_MANY_RESOLVED_WORKS));
    }
    Ok(WorkScope {
        works: works.into_iter().collect(),
        publishers: publishers.into_iter().collect(),
    })
}

/// Resolve the served platforms and measures.
///
/// Explicit IDs must all exist. An omitted or empty dimension resolves to
/// every identity represented, for the resolved works and range, in the
/// projections, in terminal coverage from an eligible managed source account
/// of a represented publisher, or in unresolved quarantine under a represented
/// publisher's immutable import scope, restricted by the other dimension when
/// that one is explicit. With no resolved work there is no represented
/// publisher, so nothing, quarantine included, is represented.
/// Every served measure, however selected, must be additive across both time
/// and works.
fn resolve_scope(
    connection: &mut PgConnection,
    request: &ValidatedRequest,
    scope: &WorkScope,
) -> Result<(BTreeSet<Uuid>, BTreeSet<Uuid>), MetricReadError> {
    let platform_filter: Vec<Uuid> = request.platforms.iter().flatten().copied().collect();
    let measure_filter: Vec<Uuid> = request.measures.iter().flatten().copied().collect();
    let mut additivity: BTreeMap<Uuid, bool> = BTreeMap::new();
    if request.platforms.is_some() || request.measures.is_some() {
        let known: Vec<KnownRow> = diesel::sql_query(KNOWN_SQL)
            .bind::<Array<SqlUuid>, _>(&platform_filter)
            .bind::<Array<SqlUuid>, _>(&measure_filter)
            .load(connection)?;
        let known_platforms: BTreeSet<Uuid> = known
            .iter()
            .filter(|row| row.kind == "PLATFORM")
            .map(|row| row.id)
            .collect();
        if let Some(platforms) = &request.platforms {
            if known_platforms != *platforms {
                return Err(MetricReadError::QueryInvalid(UNKNOWN_PLATFORM));
            }
        }
        for row in known.iter().filter(|row| row.kind == "MEASURE") {
            let (Some(across_time), Some(across_works)) =
                (row.additive_across_time, row.additive_across_works)
            else {
                return Err(MetricReadError::Unavailable);
            };
            additivity.insert(row.id, across_time && across_works);
        }
        if let Some(measures) = &request.measures {
            if additivity.keys().copied().collect::<BTreeSet<Uuid>>() != *measures {
                return Err(MetricReadError::QueryInvalid(UNKNOWN_MEASURE));
            }
        }
    }

    let (platforms, measures) = match (&request.platforms, &request.measures) {
        (Some(platforms), Some(measures)) => (platforms.clone(), measures.clone()),
        (explicit_platforms, explicit_measures) => {
            let represented: Vec<RepresentedRow> = if scope.works.is_empty() {
                Vec::new()
            } else {
                let (start, lead_end, trail_start, end) = request.edges();
                diesel::sql_query(REPRESENTED_SQL)
                    .bind::<Array<SqlUuid>, _>(&scope.works)
                    .bind::<Date, _>(start)
                    .bind::<Date, _>(lead_end)
                    .bind::<Date, _>(trail_start)
                    .bind::<Date, _>(end)
                    .bind::<Array<SqlUuid>, _>(&platform_filter)
                    .bind::<Array<SqlUuid>, _>(&measure_filter)
                    .bind::<Array<SqlUuid>, _>(&scope.publishers)
                    .bind::<Text, _>(DRIVER_ACQUISITION)
                    .load(connection)?
            };
            for row in &represented {
                additivity.insert(
                    row.measure_id,
                    row.additive_across_time && row.additive_across_works,
                );
            }
            let platforms = explicit_platforms
                .clone()
                .unwrap_or_else(|| represented.iter().map(|row| row.platform_id).collect());
            let measures = explicit_measures
                .clone()
                .unwrap_or_else(|| represented.iter().map(|row| row.measure_id).collect());
            (platforms, measures)
        }
    };

    for measure_id in &measures {
        match additivity.get(measure_id) {
            Some(true) => {}
            Some(false) => return Err(MetricReadError::QueryInvalid(NON_ADDITIVE_MEASURE)),
            None => return Err(MetricReadError::Unavailable),
        }
    }
    if platforms.len() > METRIC_DASHBOARD_MAX_PLATFORMS {
        return Err(MetricReadError::QueryLimitExceeded(TOO_MANY_PLATFORMS));
    }
    if measures.len() > METRIC_DASHBOARD_MAX_MEASURES {
        return Err(MetricReadError::QueryLimitExceeded(TOO_MANY_MEASURES));
    }
    Ok((platforms, measures))
}

/// Read and resolve the day values of the request's day ranges.
fn read_days(
    connection: &mut PgConnection,
    request: &ValidatedRequest,
    scope: &WorkScope,
    platform_ids: &[Uuid],
    measure_ids: &[Uuid],
    grid: &mut Grid,
) -> Result<(), MetricReadError> {
    let (start, lead_end, trail_start, end) = request.day_reads();
    let sums: Vec<DaySumRow> = diesel::sql_query(DAY_SUMS_SQL)
        .bind::<Array<SqlUuid>, _>(&scope.works)
        .bind::<Date, _>(start)
        .bind::<Date, _>(lead_end)
        .bind::<Date, _>(trail_start)
        .bind::<Date, _>(end)
        .bind::<Array<SqlUuid>, _>(platform_ids)
        .bind::<Array<SqlUuid>, _>(measure_ids)
        .load(connection)?;
    let mut resolved: Vec<(Uuid, Uuid, NaiveDate, String, bool, bool)> = Vec::new();
    let (mut candidate_platforms, mut candidate_measures, mut candidate_days) =
        (Vec::new(), Vec::new(), Vec::new());
    for row in sums {
        if row.is_candidate() {
            candidate_platforms.push(row.platform_id);
            candidate_measures.push(row.measure_id);
            candidate_days.push(row.day);
        } else {
            // Only undimensioned rows, or dimensioned rows sharing one
            // mask: the group total is the representation's total.
            let dimensioned = !row.has_aggregate;
            resolved.push((
                row.platform_id,
                row.measure_id,
                row.day,
                row.total,
                dimensioned && row.uses_country,
                dimensioned && row.uses_institution,
            ));
        }
    }
    if !candidate_days.is_empty() {
        let cells: Vec<DimensionCellRow> = diesel::sql_query(DIMENSION_CELLS_SQL)
            .bind::<Array<SqlUuid>, _>(&scope.works)
            .bind::<Array<SqlUuid>, _>(&candidate_platforms)
            .bind::<Array<SqlUuid>, _>(&candidate_measures)
            .bind::<Array<Date>, _>(&candidate_days)
            .load(connection)?;
        if cells.iter().any(|cell| cell.ambiguous) {
            return Err(MetricReadError::DimensionScopeAmbiguous);
        }
        resolved.extend(cells.into_iter().map(|cell| {
            (
                cell.platform_id,
                cell.measure_id,
                cell.day,
                cell.total,
                cell.depends_on_country,
                cell.depends_on_institution,
            )
        }));
    }
    let interior = request.split.interior_start..request.split.interior_end;
    for (platform_id, measure_id, day, total, country, institution) in resolved {
        let total = BigInt::parse_canonical(&total)
            .ok_or(MetricReadError::QueryLimitExceeded(AGGREGATE_OUT_OF_RANGE))?;
        grid.set_sum(platform_id, measure_id, request.start, day, total);
        // A day inside a complete month contributes its dependency through
        // that month's projected row; only edge days add their own here.
        if !interior.contains(&day) {
            grid.set_dependence(platform_id, measure_id, country, institution);
        }
    }
    Ok(())
}

/// Country and institution values, keyed for deterministic output.
#[derive(Default)]
struct Sections {
    countries: BTreeMap<(Uuid, Uuid, String), BigInt>,
    institutions: BTreeMap<(Uuid, Uuid, Uuid), InstitutionValue>,
}

/// An institution's current name and ROR, and its value.
type InstitutionValue = (String, Option<Ror>, BigInt);

/// Fold one section statement's rows into the grid and the sections.
fn apply_sections(
    rows: Vec<SectionRow>,
    months: &[NaiveDate],
    grid: &mut Grid,
    sections: &mut Sections,
) -> Result<(), MetricReadError> {
    let parse = |total: Option<String>| -> Result<BigInt, MetricReadError> {
        let total = total.ok_or(MetricReadError::Unavailable)?;
        BigInt::parse_canonical(&total)
            .ok_or(MetricReadError::QueryLimitExceeded(AGGREGATE_OUT_OF_RANGE))
    };
    let add = |current: Option<BigInt>, value: BigInt| -> Result<BigInt, MetricReadError> {
        current
            .unwrap_or(BigInt::ZERO)
            .checked_add(value)
            .ok_or(MetricReadError::QueryLimitExceeded(AGGREGATE_OUT_OF_RANGE))
    };
    // Ambiguity is checked before any value is used, so an ambiguous
    // request fails whatever else it holds.
    if rows.iter().any(|row| {
        row.section == "AMBIGUITY"
            && (row.total_ambiguous || row.country_ambiguous || row.institution_ambiguous)
    }) {
        return Err(MetricReadError::DimensionScopeAmbiguous);
    }
    for row in rows {
        match (row.section.as_str(), row.platform_id, row.measure_id) {
            ("AMBIGUITY", _, _) => {}
            ("TOTAL", Some(platform_id), Some(measure_id)) => {
                let month = row
                    .month_start
                    .and_then(|month| months.iter().position(|start| *start == month))
                    .ok_or(MetricReadError::Unavailable)?;
                let value = parse(row.total)?;
                grid.set_month_sum(platform_id, measure_id, month, value);
                grid.set_dependence(
                    platform_id,
                    measure_id,
                    row.requires_country,
                    row.requires_institution,
                );
            }
            ("COUNTRY", Some(platform_id), Some(measure_id)) => {
                let code = row.country_code.ok_or(MetricReadError::Unavailable)?;
                let value = parse(row.total)?;
                let key = (platform_id, measure_id, code);
                let current = sections.countries.get(&key).copied();
                sections.countries.insert(key, add(current, value)?);
                grid.set_section_dependence(
                    platform_id,
                    measure_id,
                    row.requires_institution,
                    false,
                );
            }
            ("INSTITUTION", Some(platform_id), Some(measure_id)) => {
                let institution_id = row.institution_id.ok_or(MetricReadError::Unavailable)?;
                let name = row.institution_name.ok_or(MetricReadError::Unavailable)?;
                let value = parse(row.total)?;
                let key = (platform_id, measure_id, institution_id);
                let current = sections.institutions.get(&key).map(|entry| entry.2);
                sections
                    .institutions
                    .insert(key, (name, row.ror, add(current, value)?));
                grid.set_section_dependence(platform_id, measure_id, false, row.requires_country);
            }
            _ => return Err(MetricReadError::Unavailable),
        }
    }
    // Each statement stops one row beyond the bound, so the union of two
    // statements exceeds it exactly when the response would.
    if sections.institutions.len() > METRIC_DASHBOARD_MAX_INSTITUTIONS {
        return Err(MetricReadError::QueryLimitExceeded(TOO_MANY_INSTITUTIONS));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Assembly
// ---------------------------------------------------------------------------

/// The current coverage of one day.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DayCoverage {
    status: MetricCoverageStatus,
    country: bool,
    institution: bool,
}

/// How far a status is from `COMPLETE`: `UNKNOWN` is worse than `PARTIAL`,
/// which is worse than `COMPLETE`.
fn severity(status: MetricCoverageStatus) -> u8 {
    match status {
        MetricCoverageStatus::Complete => 0,
        MetricCoverageStatus::Partial => 1,
        MetricCoverageStatus::Unknown => 2,
    }
}

fn worst(left: MetricCoverageStatus, right: MetricCoverageStatus) -> MetricCoverageStatus {
    if severity(right) > severity(left) {
        right
    } else {
        left
    }
}

/// The effective status of one day for a section that needs the given
/// dimensional coverage (Specification Amendment 6 section 3 and #946
/// Amendment 1): a day without an assertion is `UNKNOWN`, and a day asserted
/// `COMPLETE` without a dimension the section relies on is only `PARTIAL`.
fn effective(day: Option<DayCoverage>, country: bool, institution: bool) -> MetricCoverageStatus {
    match day {
        None => MetricCoverageStatus::Unknown,
        Some(coverage)
            if coverage.status == MetricCoverageStatus::Complete
                && ((country && !coverage.country) || (institution && !coverage.institution)) =>
        {
            MetricCoverageStatus::Partial
        }
        Some(coverage) => coverage.status,
    }
}

/// Per-day facts for one platform/measure combination.
#[derive(Debug, Clone, Default)]
struct Cells {
    /// Day values read from the work-day projection.
    sums: Vec<Option<BigInt>>,
    /// Complete-month values read from the monthly projection.
    months: Vec<Option<BigInt>>,
    /// The current assertion combined across the represented publishers;
    /// `None` means `UNKNOWN`.
    coverage: Vec<Option<DayCoverage>>,
    lag: Vec<bool>,
    /// True for days overlapped by identifier-unresolved quarantine evidence.
    identifier_incomplete: Vec<bool>,
    /// Whether any total or timeline value of this combination was taken
    /// from rows broken down by country, so complete totals also need the
    /// country dimension.
    depends_on_country: bool,
    /// The same for the institution dimension.
    depends_on_institution: bool,
    /// Whether any country value was taken from rows also broken down by
    /// institution.
    country_depends_on_institution: bool,
    /// Whether any institution value was taken from rows also broken down by
    /// country.
    institution_depends_on_country: bool,
    /// The effective status each day has for the totals and timeline.
    effective: Vec<MetricCoverageStatus>,
    /// The worst effective status each day has across every returned
    /// section.
    shared: Vec<MetricCoverageStatus>,
}

impl Cells {
    fn is_complete(&self, index: usize) -> bool {
        self.effective.get(index) == Some(&MetricCoverageStatus::Complete)
    }

    /// Whether zero is justified for the day-index range `from..to`: every
    /// day is effectively `COMPLETE` for the totals and timeline, untouched
    /// by outstanding rollup work and free of unresolved identifier evidence.
    /// Complete months and edge days share this one day mask.
    fn justified(&self, from: usize, to: usize) -> bool {
        (from..to).all(|index| {
            self.is_complete(index) && !self.lag[index] && !self.identifier_incomplete[index]
        })
    }

    /// The exact sum of some contributions, or zero when there are none and
    /// zero is `justified`, or unknown.
    fn combine<'a>(
        contributions: impl Iterator<Item = &'a BigInt>,
        justified: impl FnOnce() -> bool,
    ) -> Result<Option<BigInt>, MetricReadError> {
        let mut total: Option<BigInt> = None;
        for sum in contributions {
            total = Some(
                total
                    .unwrap_or(BigInt::ZERO)
                    .checked_add(*sum)
                    .ok_or(MetricReadError::QueryLimitExceeded(AGGREGATE_OUT_OF_RANGE))?,
            );
        }
        if total.is_some() {
            return Ok(total);
        }
        Ok(justified().then_some(BigInt::ZERO))
    }

    /// The value of the half-open day-index range `from..to`, from day
    /// values.
    ///
    /// Projected rows win: their exact sum is the value. With no rows, zero
    /// is returned only when every day is `COMPLETE`, untouched by
    /// outstanding rollup work and free of unresolved identifier evidence;
    /// otherwise the value is unknown.
    fn value(&self, from: usize, to: usize) -> Result<Option<BigInt>, MetricReadError> {
        Self::combine(self.sums[from..to].iter().flatten(), || {
            self.justified(from, to)
        })
    }

    /// The value of complete month `month`, whose days are `from..to`, from
    /// the monthly projection.
    fn month_value(
        &self,
        month: usize,
        from: usize,
        to: usize,
    ) -> Result<Option<BigInt>, MetricReadError> {
        Self::combine(self.months[month].iter(), || self.justified(from, to))
    }

    /// The total of the whole range: every complete month from the monthly
    /// projection plus the edge days `0..lead_end` and `trail_start..` from
    /// day values.
    fn total(
        &self,
        lead_end: usize,
        trail_start: usize,
    ) -> Result<Option<BigInt>, MetricReadError> {
        let day_count = self.sums.len();
        let edges = self.sums[..lead_end]
            .iter()
            .chain(self.sums[trail_start..].iter())
            .flatten();
        Self::combine(self.months.iter().flatten().chain(edges), || {
            self.justified(0, day_count)
        })
    }
}

/// Every combination's per-day facts.
struct Grid {
    cells: BTreeMap<(Uuid, Uuid), Cells>,
    /// Assertions per `(publisher, platform, measure)`, before combining.
    assertions: BTreeMap<(Uuid, Uuid, Uuid), Vec<Option<DayCoverage>>>,
    days: usize,
}

impl Grid {
    fn new(combinations: &[(Uuid, Uuid)], days: usize, months: usize) -> Self {
        let cells = combinations
            .iter()
            .map(|combination| {
                (
                    *combination,
                    Cells {
                        sums: vec![None; days],
                        months: vec![None; months],
                        coverage: vec![None; days],
                        lag: vec![false; days],
                        identifier_incomplete: vec![false; days],
                        ..Cells::default()
                    },
                )
            })
            .collect();
        Grid {
            cells,
            assertions: BTreeMap::new(),
            days,
        }
    }

    fn day_index(&self, start: NaiveDate, day: NaiveDate) -> Option<usize> {
        let index = usize::try_from((day - start).num_days()).ok()?;
        (index < self.days).then_some(index)
    }

    fn cell(
        &mut self,
        platform_id: Uuid,
        measure_id: Uuid,
        start: NaiveDate,
        day: NaiveDate,
    ) -> Option<(&mut Cells, usize)> {
        let index = self.day_index(start, day)?;
        let cells = self.cells.get_mut(&(platform_id, measure_id))?;
        Some((cells, index))
    }

    fn set_sum(
        &mut self,
        platform_id: Uuid,
        measure_id: Uuid,
        start: NaiveDate,
        day: NaiveDate,
        total: BigInt,
    ) {
        if let Some((cells, index)) = self.cell(platform_id, measure_id, start, day) {
            cells.sums[index] = Some(total);
        }
    }

    fn set_month_sum(&mut self, platform_id: Uuid, measure_id: Uuid, month: usize, total: BigInt) {
        if let Some(cells) = self.cells.get_mut(&(platform_id, measure_id)) {
            if let Some(slot) = cells.months.get_mut(month) {
                *slot = Some(total);
            }
        }
    }

    fn set_dependence(
        &mut self,
        platform_id: Uuid,
        measure_id: Uuid,
        country: bool,
        institution: bool,
    ) {
        if let Some(cells) = self.cells.get_mut(&(platform_id, measure_id)) {
            cells.depends_on_country |= country;
            cells.depends_on_institution |= institution;
        }
    }

    fn set_section_dependence(
        &mut self,
        platform_id: Uuid,
        measure_id: Uuid,
        country_on_institution: bool,
        institution_on_country: bool,
    ) {
        if let Some(cells) = self.cells.get_mut(&(platform_id, measure_id)) {
            cells.country_depends_on_institution |= country_on_institution;
            cells.institution_depends_on_country |= institution_on_country;
        }
    }

    fn set_assertion(
        &mut self,
        publisher_id: Uuid,
        platform_id: Uuid,
        measure_id: Uuid,
        start: NaiveDate,
        day: NaiveDate,
        coverage: DayCoverage,
    ) {
        let Some(index) = self.day_index(start, day) else {
            return;
        };
        let days = self.days;
        self.assertions
            .entry((publisher_id, platform_id, measure_id))
            .or_insert_with(|| vec![None; days])[index] = Some(coverage);
    }

    fn set_lag(&mut self, platform_id: Uuid, measure_id: Uuid, start: NaiveDate, day: NaiveDate) {
        if let Some((cells, index)) = self.cell(platform_id, measure_id, start, day) {
            cells.lag[index] = true;
        }
    }

    fn set_identifier_incomplete_range(
        &mut self,
        platform_id: Uuid,
        measure_id: Uuid,
        request_start: NaiveDate,
        request_end: NaiveDate,
        period_start: NaiveDate,
        period_end: NaiveDate,
    ) {
        let start = period_start.max(request_start);
        let end = period_end.min(request_end);
        for day in start.iter_days().take_while(|day| *day < end) {
            if let Some((cells, index)) = self.cell(platform_id, measure_id, request_start, day) {
                cells.identifier_incomplete[index] = true;
            }
        }
    }

    /// Combine each day's assertions across the represented publishers: any
    /// publisher without one makes the day `UNKNOWN`; otherwise the worst
    /// status wins and each dimension flag holds only if it holds for every
    /// publisher. With no represented publisher there is nothing to assert,
    /// so every day is `UNKNOWN`.
    fn combine_assertions(&mut self, publishers: &[Uuid]) {
        for ((platform_id, measure_id), cells) in self.cells.iter_mut() {
            for index in 0..cells.coverage.len() {
                let mut combined: Option<DayCoverage> = None;
                for (position, publisher_id) in publishers.iter().enumerate() {
                    let day = self
                        .assertions
                        .get(&(*publisher_id, *platform_id, *measure_id))
                        .and_then(|days| days[index]);
                    let Some(day) = day else {
                        combined = None;
                        break;
                    };
                    combined = Some(match combined {
                        Some(sofar) if position > 0 => DayCoverage {
                            status: worst(sofar.status, day.status),
                            country: sofar.country && day.country,
                            institution: sofar.institution && day.institution,
                        },
                        _ => day,
                    });
                }
                cells.coverage[index] = combined;
            }
        }
    }

    /// Derive each day's effective coverage per section (#946 Amendment 1).
    ///
    /// The totals and timeline need only the dimensions their own served
    /// representation depends on, range-wide per combination, exactly as the
    /// `MET-WP4-02` rule: that alone decides their zero or null. The country
    /// section, when returned, always needs country coverage and also needs
    /// institution coverage if a country value came from rows broken down by
    /// institution; the institution section mirrors it. The shared status —
    /// items, top-level status, `dataThrough` and warnings — is the worst of
    /// the returned sections, so an omitted section cannot downgrade it
    /// through its own dimension, and a returned one never changes a total or
    /// timeline value.
    fn resolve_coverage(&mut self, include_countries: bool, include_institutions: bool) {
        for cells in self.cells.values_mut() {
            let mut totals = Vec::with_capacity(cells.coverage.len());
            let mut shared = Vec::with_capacity(cells.coverage.len());
            for day in &cells.coverage {
                let total = effective(*day, cells.depends_on_country, cells.depends_on_institution);
                let mut worst_day = total;
                if include_countries {
                    worst_day = worst(
                        worst_day,
                        effective(*day, true, cells.country_depends_on_institution),
                    );
                }
                if include_institutions {
                    worst_day = worst(
                        worst_day,
                        effective(*day, cells.institution_depends_on_country, true),
                    );
                }
                totals.push(total);
                shared.push(worst_day);
            }
            cells.effective = totals;
            cells.shared = shared;
        }
    }
}

fn assemble(
    request: &ValidatedRequest,
    combinations: &[(Uuid, Uuid)],
    grid: &Grid,
    days: &[NaiveDate],
    frontier: FrontierRow,
    sections: Sections,
) -> Result<MetricDashboard, MetricReadError> {
    let day_count = days.len();
    let index_of = |day: NaiveDate| (day - request.start).num_days() as usize;
    let (lead_end, trail_start) = (
        index_of(request.split.interior_start),
        index_of(request.split.interior_end),
    );
    let months = request.split.months();
    let mut totals = Vec::with_capacity(combinations.len());
    let mut timeline = Vec::with_capacity(combinations.len() * request.buckets.len());
    let mut items = Vec::with_capacity(combinations.len());
    let mut any_unknown = combinations.is_empty();
    let mut any_partial = false;
    let mut any_identifier_incomplete = false;
    let mut any_lag = false;

    for (platform_id, measure_id) in combinations {
        let cells = &grid.cells[&(*platform_id, *measure_id)];

        totals.push(MetricTotal {
            platform_id: *platform_id,
            measure_id: *measure_id,
            value: cells.total(lead_end, trail_start)?,
        });
        for (bucket_start, bucket_end) in &request.buckets {
            let from = index_of(*bucket_start);
            let to = index_of(*bucket_end);
            // A monthly bucket that is a whole complete month is the monthly
            // projection's value; a clipped bucket and every daily bucket
            // come from day values.
            let month = months
                .iter()
                .position(|month| month == bucket_start && next_month(*month) == *bucket_end)
                .filter(|_| request.monthly_timeline);
            let value = match month {
                Some(month) => cells.month_value(month, from, to)?,
                None => cells.value(from, to)?,
            };
            timeline.push(MetricTimeBucket {
                platform_id: *platform_id,
                measure_id: *measure_id,
                start_date: *bucket_start,
                end_date: *bucket_end,
                value,
            });
        }

        let day_unknown = cells.shared.contains(&MetricCoverageStatus::Unknown);
        let day_partial = cells.shared.contains(&MetricCoverageStatus::Partial);
        any_unknown |= day_unknown;
        any_partial |= day_partial;
        any_identifier_incomplete |= cells.identifier_incomplete.contains(&true);
        any_lag |= cells.lag.iter().any(|lag| *lag);

        let status = if day_unknown {
            MetricCoverageStatus::Unknown
        } else if day_partial {
            MetricCoverageStatus::Partial
        } else {
            MetricCoverageStatus::Complete
        };
        let data_through = match (0..day_count).find(|index| {
            cells.shared[*index] != MetricCoverageStatus::Complete || cells.lag[*index]
        }) {
            Some(0) => None,
            Some(first_failing) => Some(days[first_failing - 1]),
            None => days.last().copied(),
        };
        let every_day = |dimension: fn(&DayCoverage) -> bool| {
            cells
                .coverage
                .iter()
                .all(|day| day.as_ref().is_some_and(dimension))
        };
        items.push(MetricCoverageItem {
            platform_id: *platform_id,
            measure_id: *measure_id,
            status,
            data_through,
            country_coverage: every_day(|day| day.country),
            institution_coverage: every_day(|day| day.institution),
        });
    }

    let coverage_status = if any_unknown {
        MetricCoverageStatus::Unknown
    } else if items
        .iter()
        .any(|item| item.status == MetricCoverageStatus::Partial)
    {
        MetricCoverageStatus::Partial
    } else {
        MetricCoverageStatus::Complete
    };
    let data_through = if items.is_empty() {
        None
    } else {
        items
            .iter()
            .map(|item| item.data_through)
            .collect::<Option<Vec<NaiveDate>>>()
            .and_then(|dates| dates.into_iter().min())
    };

    let mut warnings = Vec::new();
    if any_unknown {
        warnings.push(MetricWarning {
            code: MetricWarningCode::UnknownCoverage,
            message: UNKNOWN_COVERAGE_MESSAGE.to_string(),
        });
    }
    if any_partial {
        warnings.push(MetricWarning {
            code: MetricWarningCode::PartialCoverage,
            message: PARTIAL_COVERAGE_MESSAGE.to_string(),
        });
    }
    if any_identifier_incomplete {
        warnings.push(MetricWarning {
            code: MetricWarningCode::UnresolvedIdentifiers,
            message: UNRESOLVED_IDENTIFIERS_MESSAGE.to_string(),
        });
    }
    if any_lag {
        warnings.push(MetricWarning {
            code: MetricWarningCode::RollupLag,
            message: ROLLUP_LAG_MESSAGE.to_string(),
        });
    }

    let countries = sections
        .countries
        .into_iter()
        .map(
            |((platform_id, measure_id, country_code), value)| MetricCountryTotal {
                platform_id,
                measure_id,
                country_code,
                value,
            },
        )
        .collect();
    let institutions = sections
        .institutions
        .into_iter()
        .map(
            |((platform_id, measure_id, institution_id), (institution_name, ror, value))| {
                MetricInstitutionTotal {
                    platform_id,
                    measure_id,
                    institution_id,
                    institution_name,
                    ror,
                    value,
                }
            },
        )
        .collect();

    Ok(MetricDashboard {
        totals,
        timeline,
        countries,
        institutions,
        coverage: MetricDashboardCoverage {
            status: coverage_status,
            items,
        },
        as_of: frontier.as_of,
        data_through,
        rollup_watermark: frontier.watermark_at,
        is_partial: !warnings.is_empty(),
        warnings,
    })
}

#[cfg(test)]
pub(crate) mod tests;
