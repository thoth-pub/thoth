//! The protected, coverage-aware Metrics dashboard read (`MET-WP4-02`,
//! completed by `MET-WP4-03B` and `MET-WP4-03C-B2`).
//!
//! This module owns the `metricDashboard` contract: its input, its response
//! and the one read that answers it. It consumes the `MET-WP4-01` work-day
//! projection and strict-frontier watermark, the `MET-WP4-03A` derived
//! monthly serving projections, the `MET-WP4-03C-A` yearly section
//! projections and the `MET-WP4-03C-B1` coverage runs derived from the
//! `MET-WP1-05` coverage evidence; it owns none of them and writes none of
//! them.
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
//! Before the first query freezes that snapshot, the read takes one
//! table-level `ACCESS SHARE` lock on exactly the six derived projection
//! tables the exceptional monthly rebuild truncates, and holds it until the
//! transaction ends (`MET-WP11-B-CORR-H1`). PostgreSQL's `TRUNCATE` is not
//! MVCC-safe: a snapshot frozen before a concurrent rebuild's truncate would
//! see the tables it had not yet read as empty, and complete coverage would
//! turn that emptiness into a successful, false zero. The lock orders the
//! read and the rebuild instead, exactly as the rollup verifier's does; see
//! [`PROJECTION_LOCK_SQL`].
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
//! One to 100 publishers are selected, each of which must exist and be
//! entitled; one unentitled publisher refuses the whole request. The other
//! selector dimensions narrow the works of those publishers through current
//! Thoth metadata before any Metrics value is read, and only the publishers
//! represented by the resolved works take part in coverage, source accounts,
//! lag, values and identifier quality. Identifier quality, which has no work
//! identity, stays scoped to those publishers' historical imports and is never
//! narrowed to the resolved works.
//!
//! # Whole months, whole years and edge days
//!
//! Complete calendar months wholly inside the served range are served from
//! the `MET-WP4-03A` monthly projections, which already hold each day's
//! resolved representation summed into its month. Clipped leading and
//! trailing days are served from `metric_rollup_work_day` with the same
//! day-level resolution. Totals, countries and institutions add the two; a
//! `MONTH` timeline uses the monthly value for a complete month and the days
//! for a clipped one, a `YEAR` timeline regroups exactly those values by
//! calendar year, and a `DAY` timeline reads every day. For a `MONTH` or
//! `YEAR` timeline the country and institution sections read the complete
//! calendar years inside the range from the yearly projections in place of
//! their months; totals and the timeline never read a yearly projection.
//!
//! # All time
//!
//! `startDate` may be omitted. The read then finds, in the same snapshot,
//! the earliest day of any evidence of the resolved scope before `endDate`
//! and serves from exactly that day. That day is never moved to a supported
//! date: evidence dated before [`METRIC_DASHBOARD_EARLIEST_DATE`] refuses the
//! request instead. The response echoes the served `startDate`, `endDate`
//! and grain.
//!
//! # Coverage as intervals
//!
//! Coverage runs are read clipped to the served range and evaluated as
//! intervals. No statement and no structure of this module expands the days
//! of a range, so the cost of coverage does not grow with its length.
//!
//! # Bounded concurrency
//!
//! At most [`METRIC_DASHBOARD_MAX_CONCURRENT_READS`] reads run at once in one
//! API process. A further read waits at most [`METRIC_DASHBOARD_SLOT_WAIT`]
//! for a slot and otherwise fails as `METRIC_READ_BUSY`, and an admitted read
//! must start its last statement within [`METRIC_DASHBOARD_READ_DEADLINE`].
//!
//! # Deliberately absent
//!
//! Works response sections, pagination, `dois`, `includeDescendants`,
//! `metricWidget`, entity-level Metrics fields, native `MONTH` and
//! `REPORTING_PERIOD` serving, a response cache and any call to OPERAS.

use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Condvar, Mutex, PoisonError};
use std::time::{Duration as StdDuration, Instant};

use chrono::{Datelike, Duration, NaiveDate, Utc};
use diesel::pg::{Pg, PgConnection};
use diesel::r2d2::{ConnectionManager, PooledConnection};
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

/// The longest half-open `[startDate, endDate)` range a `DAY` timeline may
/// cover, in days, and the longest explicit range served whatever its dates.
pub const METRIC_DASHBOARD_MAX_RANGE_DAYS: i64 = 366;
/// The earliest day a range longer than [`METRIC_DASHBOARD_MAX_RANGE_DAYS`]
/// may start on, and the earliest evidence an all-time request may serve.
pub const METRIC_DASHBOARD_EARLIEST_DATE: NaiveDate = match NaiveDate::from_ymd_opt(2000, 1, 1) {
    Some(day) => day,
    None => panic!("2000-01-01 is a calendar date"),
};
/// How many days after the API process's UTC date `endDate` may fall when
/// `startDate` is omitted or the range is longer than
/// [`METRIC_DASHBOARD_MAX_RANGE_DAYS`].
pub const METRIC_DASHBOARD_MAX_FUTURE_DAYS: i64 = 366;
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
pub const METRIC_DASHBOARD_MAX_PUBLISHERS: usize = 100;
/// The most unique explicit work IDs one selector may carry.
pub const METRIC_DASHBOARD_MAX_WORK_IDS: usize = 500;
/// The most unique values every other bounded selector list may carry.
pub const METRIC_DASHBOARD_MAX_SELECTOR_VALUES: usize = 50;
/// The most works one selector may resolve to.
pub const METRIC_DASHBOARD_MAX_RESOLVED_WORKS: usize = 25_000;
/// The most institution rows one response may return.
pub const METRIC_DASHBOARD_MAX_INSTITUTIONS: usize = 100_000;
/// From this many resolved works the step-7 monthly total, monthly ambiguity
/// and yearly section branches read their projection in one filtered pass
/// instead of one index probe per work (#980 Amendment 2 section 4). It is a
/// physical read strategy only, never a product limit.
pub const METRIC_DASHBOARD_SCAN_STRATEGY_MIN_WORKS: usize = 1_200;
/// From this many resolved works the step-7 monthly country and institution
/// branches read their projections in one filtered pass too.
pub const METRIC_DASHBOARD_SECTION_SCAN_MIN_WORKS: usize = 1_600;
const _: () =
    assert!(METRIC_DASHBOARD_SCAN_STRATEGY_MIN_WORKS < METRIC_DASHBOARD_SECTION_SCAN_MIN_WORKS);
/// The most `metricDashboard` reads one API process runs at once.
pub const METRIC_DASHBOARD_MAX_CONCURRENT_READS: usize = 3;
/// The longest a read waits in total for one of those slots.
pub const METRIC_DASHBOARD_SLOT_WAIT: StdDuration = StdDuration::from_secs(1);
/// How long after its slot is granted a read may still start a statement.
pub const METRIC_DASHBOARD_READ_DEADLINE: StdDuration = StdDuration::from_secs(20);

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
        graphql(
            description = "Let Thoth choose the bucket size once the served platforms, measures and startDate are known: DAY when the served range is at most 366 days and its timeline fits, otherwise MONTH when its timeline fits, otherwise YEAR"
        )
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
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "One bucket per calendar year, clipped to the served range. Each bucket is the exact sum of the daily values it contains"
        )
    )]
    Year,
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
            description = "Thoth IDs of one to 100 publishers whose works to describe. Every selected publisher must exist and be entitled to the Metrics dashboard; otherwise the whole request is refused, never served for a subset"
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
        graphql(
            description = "First calendar day of the range, inclusive. Omitted or null means all time: the range then starts on the day of the earliest Metrics evidence for the selected works, platforms and measures before endDate, which must not be before 2000-01-01, or on the first day of the month containing the day before endDate when there is none. An omitted startDate cannot be combined with a DAY timeline"
        )
    )]
    pub start_date: Option<NaiveDate>,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "Calendar day after the range, exclusive. It must be later than startDate. A DAY timeline may cover at most 366 days; MONTH and YEAR timelines have no maximum range. When startDate is omitted or the range is longer than 366 days, startDate must be on or after 2000-01-01, endDate must be later than 2000-01-01 and endDate may be at most 366 days after the current UTC date"
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
            description = "Known per-institution totals over the range, ordered by platform ID, measure ID and institution ID, at most 100000. Empty when includeInstitutions is false. An institution without a row is not a zero; coverage says how far absence can be relied on"
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
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "The first day served. When startDate was omitted, the day of the earliest Metrics evidence for this scope, or the first day of the month containing the day before endDate when there is none"
        )
    )]
    pub start_date: NaiveDate,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "The day after the last day served, exclusive: the requested endDate"
        )
    )]
    pub end_date: NaiveDate,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "The timeline grain actually served: DAY, MONTH or YEAR, never AUTO"
        )
    )]
    pub timeline_grain: MetricTimelineGrain,
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

/// Coverage of one served platform and measure over the served range.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLObject),
    graphql(description = "Coverage of one platform and measure over the served range")
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
            description = "The last day of the unbroken run from the served startDate in which every day is completely covered and untouched by outstanding rollup work, or null when the first day is not"
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
    /// An unexpected database or pool failure, internally inconsistent
    /// committed canonical or derived state, or a read past its whole-read
    /// deadline, deliberately without detail.
    Unavailable,
    /// `METRIC_READ_BUSY`: every concurrent-read slot of this API process
    /// stayed taken for the whole slot wait. Retryable.
    Busy,
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
            MetricReadError::Busy => "METRIC_READ_BUSY",
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
            MetricReadError::Busy => "The Metrics service is busy. Try again shortly.",
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
    "selector.publisherIds must contain between one and 100 publisher IDs.";
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
    "The selector resolves to more than 25000 works. Narrow the selection.";
const TOO_MANY_INSTITUTIONS: &str = "The request would return more than 100000 institution rows. Narrow the request, or omit institutions.";
const DATE_ORDER: &str = "startDate must be earlier than endDate.";
const RANGE_TOO_LONG: &str = "A DAY timeline may cover at most 366 days.";
const DAY_NEEDS_START: &str =
    "A DAY timeline needs a startDate. Omit timelineGrain or use AUTO for an all-time range.";
const START_BEFORE_FLOOR: &str =
    "startDate must be on or after 2000-01-01 when the range is longer than 366 days.";
const END_TOO_LATE: &str = "endDate may be at most 366 days after today (UTC) when startDate is omitted or the range is longer than 366 days.";
const END_NOT_AFTER_FLOOR: &str =
    "endDate must be later than 2000-01-01 when startDate is omitted.";
const EVIDENCE_BEFORE_FLOOR: &str = "Metrics in this scope are dated before 2000-01-01, which this dashboard cannot serve. Give a startDate on or after 2000-01-01.";
const DUPLICATE_MEASURE: &str = "measures must not contain duplicate IDs.";
const DUPLICATE_PLATFORM: &str = "platforms must not contain duplicate IDs.";
const TOO_MANY_MEASURES: &str = "At most 10 measures may be served in one request.";
const TOO_MANY_PLATFORMS: &str = "At most 10 platforms may be served in one request.";
const UNKNOWN_MEASURE: &str = "measures contains an unknown metric measure ID.";
const UNKNOWN_PLATFORM: &str = "platforms contains an unknown metric platform ID.";
const NON_ADDITIVE_MEASURE: &str = "A served metric measure is not additive across both time and works, so this dashboard cannot sum it.";
const TOO_MANY_COMBINATIONS: &str =
    "At most 25 platform and measure combinations may be served in one request.";
const TOO_MANY_CELLS: &str = "The request would return more than 5000 timeline cells. Narrow the range, use a coarser timelineGrain, or serve fewer combinations.";
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

/// The first day of the year after `day`'s year.
fn next_year(day: NaiveDate) -> NaiveDate {
    NaiveDate::from_ymd_opt(day.year() + 1, 1, 1).unwrap_or(NaiveDate::MAX)
}

/// A request that passed every check that needs no database.
///
/// Nothing here fixes the served grain or its buckets: AUTO, the buckets and
/// the month split depend on the served pairs and, when `startDate` was
/// omitted, on where the scope's history starts, and are fixed by [`serve`]
/// inside the read.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ValidatedRequest {
    publisher_ids: Vec<Uuid>,
    selector: Selector,
    /// `None` when `startDate` was omitted: all history before `end`.
    start: Option<NaiveDate>,
    end: NaiveDate,
    /// `None` when the dimension was omitted or empty.
    measures: Option<BTreeSet<Uuid>>,
    platforms: Option<BTreeSet<Uuid>>,
    /// The requested grain; AUTO is resolved by [`serve`].
    grain: MetricTimelineGrain,
    include_countries: bool,
    include_institutions: bool,
}

/// The half-open timeline buckets of `[start, end)` at `grain`; AUTO is
/// treated as `DAY`.
fn buckets_of(
    start: NaiveDate,
    end: NaiveDate,
    grain: MetricTimelineGrain,
) -> Vec<(NaiveDate, NaiveDate)> {
    let mut buckets = Vec::with_capacity(bucket_count(start, end, grain));
    let mut bucket_start = start;
    while bucket_start < end {
        let next = match grain {
            MetricTimelineGrain::Auto | MetricTimelineGrain::Day => {
                bucket_start.succ_opt().unwrap_or(end)
            }
            MetricTimelineGrain::Month => next_month(bucket_start),
            MetricTimelineGrain::Year => next_year(bucket_start),
        };
        let bucket_end = next.min(end);
        buckets.push((bucket_start, bucket_end));
        bucket_start = bucket_end;
    }
    buckets
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

/// Check everything about a request that needs no database (#980 Amendment 3
/// section 4B): its shape, the selector bounds, the date order, the date
/// domain against `today`, the `DAY` range and a `DAY` timeline without a
/// start. `today` is the API process's UTC date, read once per request.
fn validate(
    input: &MetricDashboardInput,
    today: NaiveDate,
) -> Result<ValidatedRequest, MetricReadError> {
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
        return Err(MetricReadError::QueryLimitExceeded(PUBLISHER_CARDINALITY));
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
    let grain = input.timeline_grain.unwrap_or_default();
    let end = input.end_date;
    // The technical date domain applies only to an omitted start or a range
    // longer than a DAY timeline may be: any explicit range of at most 366
    // days is served whatever its dates.
    let latest_end = today
        .checked_add_signed(Duration::days(METRIC_DASHBOARD_MAX_FUTURE_DAYS))
        .unwrap_or(NaiveDate::MAX);
    match input.start_date {
        Some(start) => {
            if start >= end {
                return Err(MetricReadError::QueryInvalid(DATE_ORDER));
            }
            if (end - start).num_days() > METRIC_DASHBOARD_MAX_RANGE_DAYS {
                if grain == MetricTimelineGrain::Day {
                    return Err(MetricReadError::QueryLimitExceeded(RANGE_TOO_LONG));
                }
                if start < METRIC_DASHBOARD_EARLIEST_DATE {
                    return Err(MetricReadError::QueryLimitExceeded(START_BEFORE_FLOOR));
                }
                if end > latest_end {
                    return Err(MetricReadError::QueryLimitExceeded(END_TOO_LATE));
                }
            }
        }
        None => {
            if grain == MetricTimelineGrain::Day {
                return Err(MetricReadError::QueryInvalid(DAY_NEEDS_START));
            }
            if end <= METRIC_DASHBOARD_EARLIEST_DATE {
                return Err(MetricReadError::QueryLimitExceeded(END_NOT_AFTER_FLOOR));
            }
            if end > latest_end {
                return Err(MetricReadError::QueryLimitExceeded(END_TOO_LATE));
            }
        }
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
    Ok(ValidatedRequest {
        publisher_ids,
        selector,
        start: input.start_date,
        end,
        measures,
        platforms,
        grain,
        include_countries: input.include_countries.unwrap_or(true),
        include_institutions: input.include_institutions.unwrap_or(true),
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
#[derive(Debug, Clone, Copy)]
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

/// One row of a section statement (the step-7 complete-month statement or
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

/// One eligible coverage run of a represented publisher, already clipped to
/// the served range by [`ASSERTIONS_SQL`].
#[derive(diesel::QueryableByName)]
struct RunRow {
    #[diesel(sql_type = SqlUuid)]
    publisher_id: Uuid,
    #[diesel(sql_type = SqlUuid)]
    platform_id: Uuid,
    #[diesel(sql_type = SqlUuid)]
    measure_id: Uuid,
    #[diesel(sql_type = Date)]
    run_start: NaiveDate,
    #[diesel(sql_type = Date)]
    run_end: NaiveDate,
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
// resolved work set, the served platforms and measures, the eligible source
// accounts and the date bounds are passed as arrays and bounds, never
// iterated, so the statement count does not depend on how many publishers,
// works, days, months, years, countries or institutions are involved. No
// statement expands the days of a range: there is no day series, recursive
// day expansion or array of generated days anywhere in this module.
// ---------------------------------------------------------------------------

/// Step 0: plan every later statement of the transaction for its actual
/// arguments. A generic prepared plan for the large `work_id = ANY(...)`
/// scopes was measured 1.7 times slower by `MET-WP4-03-BENCH-01`; `LOCAL`
/// confines the setting to this read-only transaction. Custom plans also
/// fold the boolean branch switches and the `NULL` lower date bound of an
/// all-time request away before execution.
pub(crate) const CUSTOM_PLANS_SQL: &str = "SET LOCAL plan_cache_mode = force_custom_plan";

/// Step 0, continued: bound every later statement of this transaction — the
/// lock acquisition below included — to thirty seconds each. This is a limit
/// per statement; the whole read is separately bounded by the caller's
/// [`ReadDeadline`]. `LOCAL` confines it to this transaction. A statement
/// that reaches it fails the whole read as [`MetricReadError::Unavailable`];
/// no partial response is assembled.
pub(crate) const STATEMENT_TIMEOUT_SQL: &str = "SET LOCAL statement_timeout = '30s'";

/// Step 0, concluded: the one table-level lock the read takes, before its
/// first query and therefore before its `REPEATABLE READ` snapshot is
/// frozen, on exactly the four monthly and the two yearly projection tables
/// the exceptional `rebuildMetricRollupMonths` truncates, in the order the
/// rollup verifier locks them, and holds until the transaction ends.
///
/// `ACCESS SHARE` is the weakest table lock, granted to every ordinary
/// reader, conflicting only with `ACCESS EXCLUSIVE`, which that rebuild's
/// `TRUNCATE` takes, and not with the `ROW EXCLUSIVE` locks of normal
/// incremental completion, so routine maintenance is never held up by a
/// read. With the lock held from before the snapshot until commit, the
/// rebuild's truncate either committed before the lock was granted — and the
/// snapshot then sees the rebuilt state — or waits until this read ends,
/// bounded by the rebuild's own lock timeout. A read arriving while a
/// rebuild holds the tables waits here, before its snapshot, bounded by
/// [`STATEMENT_TIMEOUT_SQL`]. It is not a row lock, takes no `FOR UPDATE` or
/// `FOR SHARE`, and the transaction remains read-only.
pub(crate) const PROJECTION_LOCK_SQL: &str = "LOCK TABLE \
         public.metric_rollup_work_month, \
         public.metric_rollup_work_country_month, \
         public.metric_rollup_work_institution_month, \
         public.metric_rollup_work_month_ambiguity, \
         public.metric_rollup_work_country_year, \
         public.metric_rollup_work_institution_year \
     IN ACCESS SHARE MODE";

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

/// Step 4 (R1): the platform/measure pairs represented for the resolved works
/// and the discovery range, with each measure's additivity.
///
/// A pair is represented by any of: a work-day row on a clipped edge day
/// (`[$2, $3)` when `$10`, `[$4, $5)` when `$11`; a branch whose edge is empty
/// is switched off rather than read); a complete month `[$3, $4)` in the
/// monthly value projection, or a total-ambiguous complete month in the sparse
/// ambiguity state (together exactly the months with any work-day row); an
/// eligible `MET-WP4-03C-B1` coverage run overlapping `[$2, $5)` — an enabled
/// `DRIVER` account of a represented publisher (`$8`) whose run is of that
/// account's platform and of the publisher the account expects; the same
/// eligible account's coverage from a terminal import that recorded no
/// completion time, which no run holds (the narrow raw corner probe, written
/// with the enum predicate of `metric_import_terminal_without_completion_idx`
/// so it is served by that index); or identifier-unresolved quarantine
/// admitted under a represented publisher's import (`MET-WP7-PREREQ-04`,
/// scoped as [`IDENTIFIER_QUALITY_SQL`]). An explicit other dimension (`$6`,
/// `$7`) restricts every branch.
///
/// The monthly branches name only identity-key columns, so they are read
/// through `metric_rollup_work_month_identity_key` and
/// `metric_rollup_work_month_ambiguity_identity_key` with `work_id = ANY($1)`
/// leading, and no monthly value is read. When `startDate` is omitted, `$2`
/// and `$3` are `NULL`: every lower date bound is then switched off by its
/// `IS NULL` guard — never replaced by a sentinel date — and the leading edge
/// is empty, so the pairs are those of all history before `$5`.
pub(crate) const REPRESENTED_SQL: &str = "SELECT represented.platform_id, represented.measure_id, \
                    m.additive_across_time, m.additive_across_works \
                 FROM ( \
                     SELECT r.platform_id, r.measure_id \
                     FROM public.metric_rollup_work_day r \
                     WHERE r.work_id = ANY($1) \
                       AND (($10 AND r.day >= $2 AND r.day < $3) \
                            OR ($11 AND r.day >= $4 AND r.day < $5)) \
                       AND (cardinality($6::uuid[]) = 0 OR r.platform_id = ANY($6)) \
                       AND (cardinality($7::uuid[]) = 0 OR r.measure_id = ANY($7)) \
                     UNION \
                     SELECT mm.platform_id, mm.measure_id \
                     FROM public.metric_rollup_work_month mm \
                     WHERE mm.work_id = ANY($1) \
                       AND ($3::date IS NULL OR mm.month_start >= $3) \
                       AND mm.month_start < $4 \
                       AND (cardinality($6::uuid[]) = 0 OR mm.platform_id = ANY($6)) \
                       AND (cardinality($7::uuid[]) = 0 OR mm.measure_id = ANY($7)) \
                     UNION \
                     SELECT a.platform_id, a.measure_id \
                     FROM public.metric_rollup_work_month_ambiguity a \
                     WHERE a.work_id = ANY($1) \
                       AND ($3::date IS NULL OR a.month_start >= $3) \
                       AND a.month_start < $4 \
                       AND a.total_ambiguous \
                       AND (cardinality($6::uuid[]) = 0 OR a.platform_id = ANY($6)) \
                       AND (cardinality($7::uuid[]) = 0 OR a.measure_id = ANY($7)) \
                     UNION \
                     SELECT run.platform_id, run.measure_id \
                     FROM public.metric_source_account sa \
                     JOIN public.metric_source s ON s.source_id = sa.source_id \
                     JOIN public.metric_coverage_run run \
                       ON run.source_account_id = sa.source_account_id \
                      AND run.platform_id = sa.platform_id \
                      AND run.publisher_id = sa.expected_publisher_id \
                     WHERE sa.expected_publisher_id = ANY($8) \
                       AND sa.enabled \
                       AND s.enabled \
                       AND s.acquisition_type::text = $9 \
                       AND run.run_start < $5 \
                       AND ($2::date IS NULL OR run.run_end > $2) \
                       AND (cardinality($6::uuid[]) = 0 OR run.platform_id = ANY($6)) \
                       AND (cardinality($7::uuid[]) = 0 OR run.measure_id = ANY($7)) \
                     UNION \
                     SELECT c.platform_id, c.measure_id \
                     FROM public.metric_source_account sa \
                     JOIN public.metric_source s ON s.source_id = sa.source_id \
                     JOIN public.metric_import mi \
                       ON mi.source_account_id = sa.source_account_id \
                      AND mi.publisher_id = sa.expected_publisher_id \
                     JOIN public.metric_coverage c \
                       ON c.import_id = mi.import_id \
                      AND c.source_account_id = sa.source_account_id \
                      AND c.platform_id = sa.platform_id \
                     WHERE sa.expected_publisher_id = ANY($8) \
                       AND sa.enabled \
                       AND s.enabled \
                       AND s.acquisition_type::text = $9 \
                       AND mi.status IN ('COMPLETED'::public.metric_import_status, \
                                         'COMPLETED_WITH_ERRORS'::public.metric_import_status) \
                       AND mi.completed_at IS NULL \
                       AND c.period_start < $5 \
                       AND ($2::date IS NULL OR c.period_end > $2) \
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
                       AND ($2::date IS NULL OR q.period_end > $2) \
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
/// not settle, and, for an all-time request, where its history starts.
///
/// The `LAG` branch, evaluated only when `$1` says the frontier `W` (`$2`)
/// has not caught up, lists the platform, measure and day of every unapplied
/// work-day delta above `W` for a resolved work, served platform and served
/// measure before `$5`, and on or after `$4` unless `$4` is `NULL`. Every
/// position above `W` is unapplied under the strict frontier, so no status
/// filter is needed.
///
/// The grain branch finds canonical records of a native `MONTH` or
/// `REPORTING_PERIOD` grain for a resolved work, served platform and served
/// measure whose half-open period overlaps the range — any such record
/// before `$5` when `$4` is `NULL` — through `metric_record_native_grain_idx`
/// (its `reporting_grain <> 'DAY'` predicate is restated so the partial index
/// applies), and classifies each by following `current_revision_id` to its
/// revision:
///
/// - the pointer names a `CURRENT` revision: an active unsupported
///   observation (`ACTIVE_NATIVE_GRAIN`);
/// - the pointer names a `RETRACTED` revision and the record has no
///   `CURRENT` revision: withdrawn, and not returned at all;
/// - anything else — no pointer, a pointer that resolves to no revision of
///   the record, a pointer to a `SUPERSEDED` revision, or a pointer to a
///   `RETRACTED` revision beside a `CURRENT` one — is committed canonical
///   state that contradicts itself (`INCONSISTENT_NATIVE_GRAIN`).
///
/// The `START` branch runs only when `$8` says `startDate` was omitted, and
/// returns one row holding the earliest day before `$5` of any evidence a
/// later step reads with a date predicate, for the resolved works, the
/// represented publishers and the served pairs, or `NULL` when there is none:
///
/// - a served pair's `metric_rollup_work_day` row. Without a platform or
///   measure filter (`$9` false) every work-day row of a resolved work is of
///   a served pair, and one ordered probe per work on `(work_id, day)` stops
///   at its first row. With a filter, the earliest month `M` holding a
///   served-pair entry of the resolved works in either monthly identity key
///   is found from those keys alone, and then each work's earliest
///   served-pair row in `[M, min(M + 1 month, $5))` through `(work_id, day)`;
///   every work-day row lies in a month that has such an entry, so no row
///   outside that one month per work is read;
/// - an eligible coverage run of the accounts step 5 returned, passed as the
///   parallel arrays `$10`, `$11`, `$12`, through the run primary key;
/// - the same accounts' coverage from a terminal import without a completion
///   time, through `metric_import_terminal_without_completion_idx`;
/// - identifier-unresolved quarantine of the represented publishers (`$13`)
///   in the [`IDENTIFIER_QUALITY_SQL`] scope;
/// - the earliest `LAG` day above.
///
/// With `$8` false the branch is folded away by the custom plan, so an
/// explicit request reads nothing for it.
pub(crate) const CANONICAL_SQL: &str = "WITH lag AS ( \
                 SELECT DISTINCT r.platform_id, r.measure_id, r.period_start AS day \
                 FROM public.metric_rollup_delta d \
                 JOIN public.metric_record r ON r.record_id = d.record_id \
                 WHERE $1 \
                   AND d.work_day_sequence > $2 \
                   AND r.work_id = ANY($3) \
                   AND ($4::date IS NULL OR r.period_start >= $4) \
                   AND r.period_start < $5 \
                   AND r.platform_id = ANY($6) \
                   AND r.measure_id = ANY($7) \
             ) \
             SELECT 'LAG' AS kind, lag.platform_id, lag.measure_id, lag.day \
             FROM lag \
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
               AND ($4::date IS NULL OR r.period_end > $4) \
               AND r.reporting_grain <> 'DAY' \
               AND r.reporting_grain IN ('MONTH', 'REPORTING_PERIOD') \
               AND NOT (COALESCE(v.status = 'RETRACTED', FALSE) \
                        AND NOT EXISTS (SELECT 1 FROM public.metric_record_revision o \
                                        WHERE o.record_id = r.record_id \
                                          AND o.status = 'CURRENT')) \
             UNION ALL \
             SELECT 'START', NULL::uuid, NULL::uuid, LEAST( \
                 (SELECT MIN(lag.day) FROM lag), \
                 (SELECT MIN(first_row.day) \
                  FROM unnest($3::uuid[]) AS w(work_id) \
                  CROSS JOIN LATERAL ( \
                      SELECT r.day FROM public.metric_rollup_work_day r \
                      WHERE r.work_id = w.work_id \
                        AND r.day < $5 \
                      ORDER BY r.day \
                      LIMIT 1 \
                  ) first_row \
                  WHERE NOT $9), \
                 (SELECT MIN(first_row.day) \
                  FROM ( \
                      SELECT LEAST( \
                          (SELECT MIN(mm.month_start) \
                           FROM public.metric_rollup_work_month mm \
                           WHERE mm.work_id = ANY($3) \
                             AND mm.platform_id = ANY($6) \
                             AND mm.measure_id = ANY($7) \
                             AND mm.month_start < $5), \
                          (SELECT MIN(a.month_start) \
                           FROM public.metric_rollup_work_month_ambiguity a \
                           WHERE a.work_id = ANY($3) \
                             AND a.platform_id = ANY($6) \
                             AND a.measure_id = ANY($7) \
                             AND a.month_start < $5) \
                      ) AS month_start \
                  ) earliest_month \
                  CROSS JOIN unnest($3::uuid[]) AS w(work_id) \
                  CROSS JOIN LATERAL ( \
                      SELECT r.day FROM public.metric_rollup_work_day r \
                      WHERE r.work_id = w.work_id \
                        AND r.day >= earliest_month.month_start \
                        AND r.day < LEAST((earliest_month.month_start \
                                           + interval '1 month')::date, $5) \
                        AND r.platform_id = ANY($6) \
                        AND r.measure_id = ANY($7) \
                      ORDER BY r.day \
                      LIMIT 1 \
                  ) first_row \
                  WHERE $9), \
                 (SELECT MIN(run.run_start) \
                  FROM unnest($10::uuid[], $11::uuid[], $12::uuid[]) \
                       AS sa(source_account_id, platform_id, publisher_id) \
                  JOIN public.metric_coverage_run run \
                    ON run.source_account_id = sa.source_account_id \
                   AND run.platform_id = sa.platform_id \
                   AND run.publisher_id = sa.publisher_id \
                  WHERE run.measure_id = ANY($7) \
                    AND run.run_start < $5), \
                 (SELECT MIN(c.period_start) \
                  FROM unnest($10::uuid[], $11::uuid[], $12::uuid[]) \
                       AS sa(source_account_id, platform_id, publisher_id) \
                  JOIN public.metric_import mi \
                    ON mi.source_account_id = sa.source_account_id \
                   AND mi.publisher_id = sa.publisher_id \
                  JOIN public.metric_coverage c \
                    ON c.import_id = mi.import_id \
                   AND c.source_account_id = sa.source_account_id \
                   AND c.platform_id = sa.platform_id \
                  WHERE mi.status IN ('COMPLETED'::public.metric_import_status, \
                                      'COMPLETED_WITH_ERRORS'::public.metric_import_status) \
                    AND mi.completed_at IS NULL \
                    AND c.measure_id = ANY($7) \
                    AND c.period_start < $5), \
                 (SELECT MIN(q.period_start) \
                  FROM public.metric_identifier_quarantine q \
                  JOIN public.metric_record_provenance p \
                    ON p.record_provenance_id = q.record_provenance_id \
                  JOIN public.metric_import mi ON mi.import_id = p.import_id \
                  LEFT JOIN public.metric_identifier_quarantine_reconciliation rc \
                    ON rc.identifier_quarantine_id = q.identifier_quarantine_id \
                  WHERE mi.publisher_id = ANY($13) \
                    AND q.period_start < $5 \
                    AND q.platform_id = ANY($6) \
                    AND q.measure_id = ANY($7) \
                    AND rc.resolved_at IS NULL) \
             ) \
             WHERE $8";

// Step 7: the complete months, from the `MET-WP4-03A` monthly projections,
// and for a `MONTH` or `YEAR` timeline the complete calendar years of the
// country and institution sections from the `MET-WP4-03C-A` yearly
// projections (#980 Amendment 1 section 3, Amendment 2 sections 3 and 4).
//
// Every monthly and yearly row already holds its work's resolved daily
// contributions, so the rows are only summed, never re-resolved or
// re-ranked: a `NULL` and a publication-specific row are both added. The
// branches return, for the resolved works (`$1`) and served platforms and
// measures (`$2`, `$3`):
//
// - `TOTAL`: each platform/measure/month total over the whole interior
//   `[$4, $5)`, with the OR of its country and institution dependencies.
//   Totals and the timeline never read a yearly projection;
// - `COUNTRY`, only when `$6`, and `INSTITUTION`, only when `$7` (stopping
//   after `$8` grouped rows): each platform/measure/country or institution
//   total, summed over one disjoint composition of section rows — the whole
//   interior from the monthly section projections, or, when the interior
//   holds complete calendar years `[$9, $10)`, those years from the yearly
//   section projections unioned before grouping with the remaining monthly
//   rows `[$4, $9)` and `[$10, $5)`;
// - `AMBIGUITY`: one summary row saying whether any month of the whole
//   interior is total-ambiguous, or country- or institution-ambiguous in a
//   section that is requested. Monthly ambiguity stays authoritative over a
//   year read from a yearly projection.
//
// The family holds eight texts, chosen by [`month_statement`] from the
// composition and the resolved-work count ([`scan_strategy`]). A `_SCAN_`
// text differs from its probe twin only in the scope predicate of the
// monthly total, monthly ambiguity and yearly branches, and a `_FULL_SCAN_`
// text also in that of the monthly section branches: the probe scope
// `<alias>.work_id = ANY($1)` is read through the work-leading identity
// index, while the scan scope `COALESCE(<alias>.work_id, <alias>.work_id) =
// ANY($1)` — the same predicate on the `NOT NULL` column — is not indexable,
// so the projection is read in one filtered pass instead of one probe per
// work. Folding every scan scope back to the probe scope yields the probe
// text byte for byte.

/// The `TOTAL` branch over the whole interior `[$4, $5)`.
macro_rules! total_branch_sql {
    ($scope:literal) => {
        concat!(
            "SELECT 'TOTAL' AS section, m.platform_id, m.measure_id, \
                    m.month_start, NULL::text AS country_code, NULL::uuid AS institution_id, \
                    NULL::text AS institution_name, NULL::text AS ror, \
                    SUM(m.value)::text AS total, \
                    bool_or(m.requires_country_coverage) AS requires_country, \
                    bool_or(m.requires_institution_coverage) AS requires_institution, \
                    FALSE AS total_ambiguous, FALSE AS country_ambiguous, \
                    FALSE AS institution_ambiguous \
             FROM public.metric_rollup_work_month m \
             WHERE ",
            $scope,
            " AND m.platform_id = ANY($2) \
               AND m.measure_id = ANY($3) \
               AND m.month_start >= $4 \
               AND m.month_start < $5 \
             GROUP BY m.platform_id, m.measure_id, m.month_start"
        )
    };
}

/// The `AMBIGUITY` summary over the whole interior `[$4, $5)`.
macro_rules! ambiguity_branch_sql {
    ($scope:literal) => {
        concat!(
            "SELECT 'AMBIGUITY', NULL::uuid, NULL::uuid, NULL::date, NULL::text, NULL::uuid, \
                    NULL::text, NULL::text, NULL::text, FALSE, FALSE, \
                    COALESCE(bool_or(a.total_ambiguous), FALSE), \
                    COALESCE(bool_or($6 AND a.country_ambiguous), FALSE), \
                    COALESCE(bool_or($7 AND a.institution_ambiguous), FALSE) \
             FROM public.metric_rollup_work_month_ambiguity a \
             WHERE ",
            $scope,
            " AND a.platform_id = ANY($2) \
               AND a.measure_id = ANY($3) \
               AND a.month_start >= $4 \
               AND a.month_start < $5"
        )
    };
}

/// The monthly country rows of the months satisfying `$range`.
macro_rules! country_month_rows_sql {
    ($scope:literal, $($range:tt)*) => {
        concat!(
            "SELECT c.platform_id, c.measure_id, c.country_code, c.value, \
                    c.requires_institution_coverage \
             FROM public.metric_rollup_work_country_month c \
             WHERE $6 \
               AND ",
            $scope,
            " AND c.platform_id = ANY($2) \
               AND c.measure_id = ANY($3) \
               AND ",
            $($range)*
        )
    };
}

/// The yearly country rows of the complete years `[$9, $10)`.
macro_rules! country_year_rows_sql {
    ($scope:literal) => {
        concat!(
            "SELECT y.platform_id, y.measure_id, y.country_code, y.value, \
                    y.requires_institution_coverage \
             FROM public.metric_rollup_work_country_year y \
             WHERE $6 \
               AND ",
            $scope,
            " AND y.platform_id = ANY($2) \
               AND y.measure_id = ANY($3) \
               AND y.year_start >= $9 \
               AND y.year_start < $10"
        )
    };
}

/// The monthly institution rows of the months satisfying `$range`.
macro_rules! institution_month_rows_sql {
    ($scope:literal, $($range:tt)*) => {
        concat!(
            "SELECT i.platform_id, i.measure_id, i.institution_id, i.value, \
                    i.requires_country_coverage \
             FROM public.metric_rollup_work_institution_month i \
             WHERE $7 \
               AND ",
            $scope,
            " AND i.platform_id = ANY($2) \
               AND i.measure_id = ANY($3) \
               AND ",
            $($range)*
        )
    };
}

/// The yearly institution rows of the complete years `[$9, $10)`.
macro_rules! institution_year_rows_sql {
    ($scope:literal) => {
        concat!(
            "SELECT y.platform_id, y.measure_id, y.institution_id, y.value, \
                    y.requires_country_coverage \
             FROM public.metric_rollup_work_institution_year y \
             WHERE $7 \
               AND ",
            $scope,
            " AND y.platform_id = ANY($2) \
               AND y.measure_id = ANY($3) \
               AND y.year_start >= $9 \
               AND y.year_start < $10"
        )
    };
}

/// The `COUNTRY` branch over the given section rows.
macro_rules! country_branch_sql {
    ($($rows:tt)*) => {
        concat!(
            "SELECT 'COUNTRY', u.platform_id, u.measure_id, NULL::date, u.country_code::text, \
                    NULL::uuid, NULL::text, NULL::text, SUM(u.value)::text, FALSE, \
                    bool_or(u.requires_institution_coverage), FALSE, FALSE, FALSE \
             FROM ( ",
            $($rows)*,
            " ) u \
             GROUP BY u.platform_id, u.measure_id, u.country_code"
        )
    };
}

/// The `INSTITUTION` branch over the given section rows.
macro_rules! institution_branch_sql {
    ($($rows:tt)*) => {
        concat!(
            "SELECT 'INSTITUTION', v.platform_id, v.measure_id, NULL::date, NULL::text, \
                    v.institution_id, n.institution_name, n.ror, v.total, \
                    v.requires_country, FALSE, FALSE, FALSE, FALSE \
             FROM ( \
                 SELECT u.platform_id, u.measure_id, u.institution_id, \
                        SUM(u.value)::text AS total, \
                        bool_or(u.requires_country_coverage) AS requires_country \
                 FROM ( ",
            $($rows)*,
            " ) u \
                 GROUP BY u.platform_id, u.measure_id, u.institution_id \
                 LIMIT $8 \
             ) v \
             JOIN public.institution n ON n.institution_id = v.institution_id"
        )
    };
}

/// The whole interior `[$4, $5)`, for the monthly texts.
macro_rules! whole_interior {
    ($alias:literal) => {
        concat!(
            "(",
            $alias,
            ".month_start >= $4 AND ",
            $alias,
            ".month_start < $5)"
        )
    };
}

/// The clipped months `[$4, $9)` and `[$10, $5)`, for the composed texts.
macro_rules! clipped_months {
    ($alias:literal) => {
        concat!(
            "((",
            $alias,
            ".month_start >= $4 AND ",
            $alias,
            ".month_start < $9) OR (",
            $alias,
            ".month_start >= $10 AND ",
            $alias,
            ".month_start < $5))"
        )
    };
}

/// Step 7, ordinary family, fewer than
/// [`METRIC_DASHBOARD_SCAN_STRATEGY_MIN_WORKS`] resolved works: a `DAY`
/// timeline, or a `MONTH` or `YEAR` timeline whose interior holds no complete
/// calendar year. Every branch probes.
pub(crate) const MONTHS_SQL: &str = concat!(
    total_branch_sql!("m.work_id = ANY($1)"),
    " UNION ALL ",
    country_branch_sql!(country_month_rows_sql!(
        "c.work_id = ANY($1)",
        whole_interior!("c")
    )),
    " UNION ALL ",
    institution_branch_sql!(institution_month_rows_sql!(
        "i.work_id = ANY($1)",
        whole_interior!("i")
    )),
    " UNION ALL ",
    ambiguity_branch_sql!("a.work_id = ANY($1)")
);

/// [`MONTHS_SQL`] from [`METRIC_DASHBOARD_SCAN_STRATEGY_MIN_WORKS`] resolved
/// works: the monthly total and ambiguity branches scan.
pub(crate) const MONTHS_SCAN_SQL: &str = concat!(
    total_branch_sql!("COALESCE(m.work_id, m.work_id) = ANY($1)"),
    " UNION ALL ",
    country_branch_sql!(country_month_rows_sql!(
        "c.work_id = ANY($1)",
        whole_interior!("c")
    )),
    " UNION ALL ",
    institution_branch_sql!(institution_month_rows_sql!(
        "i.work_id = ANY($1)",
        whole_interior!("i")
    )),
    " UNION ALL ",
    ambiguity_branch_sql!("COALESCE(a.work_id, a.work_id) = ANY($1)")
);

/// [`MONTHS_SQL`] from [`METRIC_DASHBOARD_SECTION_SCAN_MIN_WORKS`] resolved
/// works: every branch scans.
pub(crate) const MONTHS_FULL_SCAN_SQL: &str = concat!(
    total_branch_sql!("COALESCE(m.work_id, m.work_id) = ANY($1)"),
    " UNION ALL ",
    country_branch_sql!(country_month_rows_sql!(
        "COALESCE(c.work_id, c.work_id) = ANY($1)",
        whole_interior!("c")
    )),
    " UNION ALL ",
    institution_branch_sql!(institution_month_rows_sql!(
        "COALESCE(i.work_id, i.work_id) = ANY($1)",
        whole_interior!("i")
    )),
    " UNION ALL ",
    ambiguity_branch_sql!("COALESCE(a.work_id, a.work_id) = ANY($1)")
);

/// Step 7, mixed-year composition, fewer than
/// [`METRIC_DASHBOARD_SCAN_STRATEGY_MIN_WORKS`] resolved works: a `MONTH` or
/// `YEAR` timeline whose interior holds complete calendar years `[$9, $10)`
/// and further complete months. Every branch probes.
pub(crate) const MONTHS_YEAR_SQL: &str = concat!(
    total_branch_sql!("m.work_id = ANY($1)"),
    " UNION ALL ",
    country_branch_sql!(
        country_month_rows_sql!("c.work_id = ANY($1)", clipped_months!("c")),
        " UNION ALL ",
        country_year_rows_sql!("y.work_id = ANY($1)")
    ),
    " UNION ALL ",
    institution_branch_sql!(
        institution_month_rows_sql!("i.work_id = ANY($1)", clipped_months!("i")),
        " UNION ALL ",
        institution_year_rows_sql!("y.work_id = ANY($1)")
    ),
    " UNION ALL ",
    ambiguity_branch_sql!("a.work_id = ANY($1)")
);

/// [`MONTHS_YEAR_SQL`] from [`METRIC_DASHBOARD_SCAN_STRATEGY_MIN_WORKS`]
/// resolved works: the monthly total, monthly ambiguity and yearly branches
/// scan.
pub(crate) const MONTHS_YEAR_SCAN_SQL: &str = concat!(
    total_branch_sql!("COALESCE(m.work_id, m.work_id) = ANY($1)"),
    " UNION ALL ",
    country_branch_sql!(
        country_month_rows_sql!("c.work_id = ANY($1)", clipped_months!("c")),
        " UNION ALL ",
        country_year_rows_sql!("COALESCE(y.work_id, y.work_id) = ANY($1)")
    ),
    " UNION ALL ",
    institution_branch_sql!(
        institution_month_rows_sql!("i.work_id = ANY($1)", clipped_months!("i")),
        " UNION ALL ",
        institution_year_rows_sql!("COALESCE(y.work_id, y.work_id) = ANY($1)")
    ),
    " UNION ALL ",
    ambiguity_branch_sql!("COALESCE(a.work_id, a.work_id) = ANY($1)")
);

/// [`MONTHS_YEAR_SQL`] from [`METRIC_DASHBOARD_SECTION_SCAN_MIN_WORKS`]
/// resolved works: every branch scans.
pub(crate) const MONTHS_YEAR_FULL_SCAN_SQL: &str = concat!(
    total_branch_sql!("COALESCE(m.work_id, m.work_id) = ANY($1)"),
    " UNION ALL ",
    country_branch_sql!(
        country_month_rows_sql!(
            "COALESCE(c.work_id, c.work_id) = ANY($1)",
            clipped_months!("c")
        ),
        " UNION ALL ",
        country_year_rows_sql!("COALESCE(y.work_id, y.work_id) = ANY($1)")
    ),
    " UNION ALL ",
    institution_branch_sql!(
        institution_month_rows_sql!(
            "COALESCE(i.work_id, i.work_id) = ANY($1)",
            clipped_months!("i")
        ),
        " UNION ALL ",
        institution_year_rows_sql!("COALESCE(y.work_id, y.work_id) = ANY($1)")
    ),
    " UNION ALL ",
    ambiguity_branch_sql!("COALESCE(a.work_id, a.work_id) = ANY($1)")
);

/// Step 7, year-only composition, fewer than
/// [`METRIC_DASHBOARD_SCAN_STRATEGY_MIN_WORKS`] resolved works: a `MONTH` or
/// `YEAR` timeline whose interior is exactly the complete calendar years
/// `[$9, $10)`, so no monthly section branch exists. Every branch probes.
pub(crate) const MONTHS_YEAR_ONLY_SQL: &str = concat!(
    total_branch_sql!("m.work_id = ANY($1)"),
    " UNION ALL ",
    country_branch_sql!(country_year_rows_sql!("y.work_id = ANY($1)")),
    " UNION ALL ",
    institution_branch_sql!(institution_year_rows_sql!("y.work_id = ANY($1)")),
    " UNION ALL ",
    ambiguity_branch_sql!("a.work_id = ANY($1)")
);

/// [`MONTHS_YEAR_ONLY_SQL`] from
/// [`METRIC_DASHBOARD_SCAN_STRATEGY_MIN_WORKS`] resolved works: with no
/// monthly section branch both scan tiers coincide, and every branch scans.
pub(crate) const MONTHS_YEAR_ONLY_SCAN_SQL: &str = concat!(
    total_branch_sql!("COALESCE(m.work_id, m.work_id) = ANY($1)"),
    " UNION ALL ",
    country_branch_sql!(country_year_rows_sql!(
        "COALESCE(y.work_id, y.work_id) = ANY($1)"
    )),
    " UNION ALL ",
    institution_branch_sql!(institution_year_rows_sql!(
        "COALESCE(y.work_id, y.work_id) = ANY($1)"
    )),
    " UNION ALL ",
    ambiguity_branch_sql!("COALESCE(a.work_id, a.work_id) = ANY($1)")
);

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

/// Step 10 (`MET-WP4-03C-B1` H1, #980 Amendment 1 section 5, Amendment 2
/// section 5, Amendment 3 section 5): the coverage runs of the eligible
/// source accounts, clipped to the served range `[$4, $5)`.
///
/// The accounts are exactly those step 5 returned, passed as the parallel
/// arrays `$1` (account), `$2` (its platform) and `$3` (its expected
/// publisher), so the request-time eligibility of #952 Amendment 5B section 5
/// is applied once, there: an enabled `DRIVER` account of an enabled source,
/// of a represented publisher and a served platform. A run participates only
/// when it is of that account, of that account's platform and of the
/// publisher the account expects, and of a served measure (`$6`). Its
/// historical stream identity is compared with that configuration, never
/// replaced by it.
///
/// Each run is the coalesced terminal winner of one stream over a half-open
/// interval, built by `MET-WP4-03C-B1` from `COMPLETED` and
/// `COMPLETED_WITH_ERRORS` imports with a recorded completion time, so the
/// statement returns at most one row per eligible run, read through the run
/// primary key, and never one row per day: no day of the range is expanded
/// here or anywhere else in this module. Runs carry no winner identity, so
/// two runs of one stream that cover the same served day are invalid derived
/// state, which the read refuses in [`assemble`].
pub(crate) const ASSERTIONS_SQL: &str = "SELECT sa.publisher_id, run.platform_id, run.measure_id, \
                    GREATEST(run.run_start, $4) AS run_start, \
                    LEAST(run.run_end, $5) AS run_end, \
                    run.coverage_status, run.import_status::text AS import_status, \
                    run.country_coverage, run.institution_coverage \
             FROM unnest($1::uuid[], $2::uuid[], $3::uuid[]) \
                  AS sa(source_account_id, platform_id, publisher_id) \
             JOIN public.metric_coverage_run run \
               ON run.source_account_id = sa.source_account_id \
              AND run.platform_id = sa.platform_id \
              AND run.publisher_id = sa.publisher_id \
             WHERE run.measure_id = ANY($6) \
               AND run.run_start < $5 \
               AND run.run_end > $4";

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
// Concurrency guard and whole-read deadline
// ---------------------------------------------------------------------------

/// A counting guard on concurrent dashboard reads (#980 Amendment 3
/// section 6), built on `std::sync` alone.
///
/// A read takes a slot after [`validate`] and before its pooled connection,
/// waiting at most a fixed time in total for one, and gives it back when its
/// [`ReadPermit`] drops. Waiters are not served in order. A poisoned lock is
/// recovered, never propagated: the count it protects is updated in single
/// steps that cannot be left half done.
pub(crate) struct ReadGuard {
    capacity: usize,
    held: Mutex<usize>,
    freed: Condvar,
}

impl ReadGuard {
    pub(crate) const fn new(capacity: usize) -> Self {
        ReadGuard {
            capacity,
            held: Mutex::new(0),
            freed: Condvar::new(),
        }
    }

    /// Take a slot, waiting at most `wait` in total on the monotonic clock,
    /// re-checked after every wake-up. Fails as [`MetricReadError::Busy`]
    /// when no slot frees in time.
    fn acquire(&self, wait: StdDuration) -> Result<ReadPermit<'_>, MetricReadError> {
        let started = Instant::now();
        let mut held = self.held.lock().unwrap_or_else(PoisonError::into_inner);
        loop {
            if *held < self.capacity {
                *held += 1;
                return Ok(ReadPermit { guard: self });
            }
            let remaining = wait.saturating_sub(started.elapsed());
            if remaining.is_zero() {
                log::warn!(
                    "metricDashboard read refused: every concurrent-read slot stayed taken for the whole slot wait"
                );
                return Err(MetricReadError::Busy);
            }
            held = self
                .freed
                .wait_timeout(held, remaining)
                .unwrap_or_else(PoisonError::into_inner)
                .0;
        }
    }

    /// How many slots are taken.
    #[cfg(test)]
    pub(crate) fn held(&self) -> usize {
        *self.held.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// One taken slot of a [`ReadGuard`], given back when it drops: on success,
/// on every error, on rollback and on panic unwind alike.
pub(crate) struct ReadPermit<'a> {
    guard: &'a ReadGuard,
}

impl Drop for ReadPermit<'_> {
    fn drop(&mut self) {
        let mut held = self
            .guard
            .held
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        *held = held.saturating_sub(1);
        drop(held);
        self.guard.freed.notify_all();
    }
}

/// The guard every production `metricDashboard` read of this process takes.
static METRIC_DASHBOARD_READS: ReadGuard = ReadGuard::new(METRIC_DASHBOARD_MAX_CONCURRENT_READS);

/// The whole-read deadline (#980 Amendment 3 section 6): the instant after
/// which the read starts no further statement, and whether it has refused
/// one.
///
/// The caller creates it when the read's slot is granted and passes it to
/// the engine, which checks it before each statement from `BEGIN` to
/// [`IDENTIFIER_QUALITY_SQL`]; `COMMIT` is not checked, so a read whose
/// statements all started in time is served. A statement already started
/// runs on under [`STATEMENT_TIMEOUT_SQL`].
pub(crate) struct ReadDeadline {
    at: Instant,
    expired: Cell<bool>,
}

impl ReadDeadline {
    pub(crate) fn new(at: Instant) -> Self {
        ReadDeadline {
            at,
            expired: Cell::new(false),
        }
    }

    /// How long remains before the deadline.
    fn remaining(&self) -> StdDuration {
        self.at.saturating_duration_since(Instant::now())
    }

    /// Whether the engine refused a statement because the deadline passed.
    #[cfg(test)]
    pub(crate) fn expired(&self) -> bool {
        self.expired.get()
    }

    /// Allow the next statement, or refuse it once the deadline has passed:
    /// the read then rolls back and fails as [`MetricReadError::Unavailable`].
    fn check(&self) -> Result<(), MetricReadError> {
        if Instant::now() < self.at {
            return Ok(());
        }
        if !self.expired.replace(true) {
            log::warn!(
                "metricDashboard read stopped: the whole-read deadline passed before a statement could start"
            );
        }
        Err(MetricReadError::Unavailable)
    }
}

/// What a read runs under: its per-statement timeout statement, its
/// concurrency guard, how long it waits for a slot, its whole-read deadline
/// and the UTC date its date domain is checked against.
pub(crate) struct ReadControls<'a> {
    pub(crate) statement_timeout_sql: &'static str,
    pub(crate) guard: &'a ReadGuard,
    pub(crate) slot_wait: StdDuration,
    pub(crate) read_deadline: StdDuration,
    pub(crate) today: NaiveDate,
}

// ---------------------------------------------------------------------------
// The read
// ---------------------------------------------------------------------------

/// Answer one dashboard request.
///
/// The caller has already passed `METRICS_READ_SERVICE`. Checks that need no
/// database run first, against the process's UTC date; the read then takes
/// one of this process's [`METRIC_DASHBOARD_MAX_CONCURRENT_READS`] slots,
/// waiting at most [`METRIC_DASHBOARD_SLOT_WAIT`], and must start its last
/// statement within [`METRIC_DASHBOARD_READ_DEADLINE`] of the grant.
/// Everything else runs in one `READ ONLY, REPEATABLE READ` transaction on
/// one connection, in this order:
///
/// 0. plan every statement for its actual arguments, bound each later
///    statement to thirty seconds, and lock the six derived projection
///    tables `ACCESS SHARE` before the first query freezes the snapshot;
/// 1. resolve the selected publishers and require `METRICS_DASHBOARD` for
///    each through the ADR-0001 package model, and, in the same statement,
/// 2. read the `MET-WP4-01` frontier `W`, `next_sequence` and `watermark_at`,
///    together with the transaction timestamp;
/// 3. check every explicit selector ID and resolve the works, and with them
///    the represented publishers;
/// 4. resolve and bound the platform/measure scope, and require every served
///    measure to be additive across time and works; with an explicit start,
///    fix the served grain and bound the timeline;
/// 5. resolve the eligible managed source account per represented publisher
///    and platform;
/// 6. refuse active native-grain canonical records, find unapplied work-day
///    deltas above `W`, and, when `startDate` was omitted, find where the
///    scope's evidence starts, then fix the served grain and bound the
///    timeline;
/// 7. read the complete months from the monthly projections, and for a
///    `MONTH` or `YEAR` timeline the complete calendar years of the sections
///    from the yearly projections;
/// 8. sum the work-day projection per platform, measure and day over the day
///    ranges, resolving by base cell only the groups whose dimensional
///    representation the group sums alone cannot settle;
/// 9. read the edge days' country and institution values;
/// 10. read the eligible coverage runs clipped to the served range;
/// 11. mark unresolved identifier evidence of the represented publishers
///     under immutable import-publisher scope, without consulting current
///     DOI resolution, work metadata or source enablement.
///
/// Every statement is set-based over the whole request, so the number of
/// statements does not depend on how many publishers, works, days, months,
/// years, countries, institutions or rows are involved: at most seventeen,
/// including transaction control, run for any request, and eighteen counting
/// the pool's own connection check on checkout.
pub(crate) fn metric_dashboard(
    db: &PgPool,
    input: &MetricDashboardInput,
) -> Result<MetricDashboard, MetricReadError> {
    metric_dashboard_with_statement_timeout(db, input, STATEMENT_TIMEOUT_SQL)
}

/// [`metric_dashboard`] with an explicit per-statement timeout statement.
///
/// The production entry point passes [`STATEMENT_TIMEOUT_SQL`]. Tests pass a
/// far shorter timeout to prove the lock-contention failure path without
/// waiting thirty seconds; the statement is always one of this module's own
/// constants, never caller text, and it changes nothing else about the read.
fn metric_dashboard_with_statement_timeout(
    db: &PgPool,
    input: &MetricDashboardInput,
    statement_timeout_sql: &'static str,
) -> Result<MetricDashboard, MetricReadError> {
    metric_dashboard_with_controls(
        db,
        input,
        &ReadControls {
            statement_timeout_sql,
            guard: &METRIC_DASHBOARD_READS,
            slot_wait: METRIC_DASHBOARD_SLOT_WAIT,
            read_deadline: METRIC_DASHBOARD_READ_DEADLINE,
            today: Utc::now().date_naive(),
        },
    )
}

/// [`metric_dashboard`] under explicit [`ReadControls`]: the internal entry
/// point, through which tests use a private guard, short durations and a
/// fixed date.
///
/// A request refused by [`validate`] takes no slot. The slot is taken before
/// the pooled connection, so a waiting request holds no connection and the
/// slot is given back only after the connection is. The checkout waits at
/// most the lesser of the pool's own connection timeout and the time left
/// before the deadline, and one the deadline cuts short fails as a deadline
/// expiry ([`checkout`]).
pub(crate) fn metric_dashboard_with_controls(
    db: &PgPool,
    input: &MetricDashboardInput,
    controls: &ReadControls<'_>,
) -> Result<MetricDashboard, MetricReadError> {
    let request = validate(input, controls.today)?;
    let _permit = controls.guard.acquire(controls.slot_wait)?;
    let deadline = ReadDeadline::new(Instant::now() + controls.read_deadline);
    let mut connection = checkout(db, &deadline)?;
    deadline.check()?;
    connection
        .build_transaction()
        .read_only()
        .repeatable_read()
        .run(|connection| {
            evaluate(
                connection,
                &request,
                controls.statement_timeout_sql,
                &deadline,
            )
        })
}

/// Check out the read's one pooled connection, waiting at most the lesser of
/// the pool's own connection timeout and the time left before `deadline`.
///
/// A checkout that fails is [`MetricReadError::Unavailable`] either way.
/// When the deadline has passed by then, because it was the shorter bound,
/// the failure goes through [`ReadDeadline::check`]: it is logged once as a
/// deadline expiry and recorded on the deadline. A pool timeout that fires
/// while the deadline still has time left is the pool's own failure.
fn checkout(
    db: &PgPool,
    deadline: &ReadDeadline,
) -> Result<PooledConnection<ConnectionManager<PgConnection>>, MetricReadError> {
    db.get_timeout(db.connection_timeout().min(deadline.remaining()))
        .map_err(|error| match deadline.check() {
            Err(expired) => expired,
            Ok(()) => MetricReadError::from(error),
        })
}

/// The works a request resolved to, and the publishers they represent.
struct WorkScope {
    works: Vec<Uuid>,
    publishers: Vec<Uuid>,
}

fn evaluate(
    connection: &mut PgConnection,
    request: &ValidatedRequest,
    statement_timeout_sql: &'static str,
    deadline: &ReadDeadline,
) -> Result<MetricDashboard, MetricReadError> {
    // 0. Custom plans, the per-statement timeout and the six-table read lock,
    // for this transaction only and in that order. None of the three takes a
    // snapshot, so the `REPEATABLE READ` snapshot is frozen by the publishers
    // query below, after the lock is already held.
    deadline.check()?;
    diesel::sql_query(CUSTOM_PLANS_SQL).execute(connection)?;
    deadline.check()?;
    diesel::sql_query(statement_timeout_sql).execute(connection)?;
    deadline.check()?;
    diesel::sql_query(PROJECTION_LOCK_SQL).execute(connection)?;

    // 1. Publishers and entitlement, with (2) the rollup frontier read in
    // the same snapshot as everything else.
    deadline.check()?;
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
    let scope = resolve_works(connection, request, deadline)?;

    // 4. Platforms and measures.
    let (platforms, measures) = resolve_scope(connection, request, &scope, deadline)?;
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
    // An explicit start fixes the served range now, so its grain and its
    // timeline bound are settled before any Metrics value is read.
    let explicit_window = match request.start {
        Some(start) => Some(serve(request, start, combinations.len())?),
        None => None,
    };
    let platform_ids: Vec<Uuid> = platforms.iter().copied().collect();
    let measure_ids: Vec<Uuid> = measures.iter().copied().collect();

    let mut facts: BTreeMap<(Uuid, Uuid), PairFacts> = combinations
        .iter()
        .map(|combination| (*combination, PairFacts::default()))
        .collect();
    let mut sections = Sections::default();
    let mut runs: Vec<CoverageRun> = Vec::new();
    let window = if combinations.is_empty() {
        // Nothing is served, so nothing can start the history: an omitted
        // start serves the month before endDate.
        match explicit_window {
            Some(window) => window,
            None => serve(request, fallback_start(request.end), 0)?,
        }
    } else {
        // 5. Eligible managed source accounts of the represented publishers.
        let mut accounts: Vec<AccountRow> = Vec::new();
        if !scope.publishers.is_empty() {
            deadline.check()?;
            accounts = diesel::sql_query(ACCOUNTS_SQL)
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
        }
        let account_ids: Vec<Uuid> = accounts.iter().map(|row| row.source_account_id).collect();
        let account_platforms: Vec<Uuid> = accounts.iter().map(|row| row.platform_id).collect();
        let account_publishers: Vec<Uuid> = accounts.iter().map(|row| row.publisher_id).collect();

        // 6. Unsupported native grains, outstanding rollup work and, when
        // startDate was omitted, the start of the scope's history.
        let mut lag: Vec<(Uuid, Uuid, NaiveDate)> = Vec::new();
        let mut earliest: Option<NaiveDate> = None;
        if !scope.works.is_empty() {
            let lagging = frontier.applied_through_sequence < frontier.next_sequence - 1;
            deadline.check()?;
            let canonical: Vec<CanonicalRow> = diesel::sql_query(CANONICAL_SQL)
                .bind::<Bool, _>(lagging)
                .bind::<SqlBigInt, _>(frontier.applied_through_sequence)
                .bind::<Array<SqlUuid>, _>(&scope.works)
                .bind::<Nullable<Date>, _>(request.start)
                .bind::<Date, _>(request.end)
                .bind::<Array<SqlUuid>, _>(&platform_ids)
                .bind::<Array<SqlUuid>, _>(&measure_ids)
                .bind::<Bool, _>(request.start.is_none())
                .bind::<Bool, _>(request.platforms.is_some() || request.measures.is_some())
                .bind::<Array<SqlUuid>, _>(&account_ids)
                .bind::<Array<SqlUuid>, _>(&account_platforms)
                .bind::<Array<SqlUuid>, _>(&account_publishers)
                .bind::<Array<SqlUuid>, _>(&scope.publishers)
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
            let mut starts = 0;
            for row in canonical {
                match (row.kind.as_str(), row.platform_id, row.measure_id, row.day) {
                    ("LAG", Some(platform_id), Some(measure_id), Some(day)) => {
                        lag.push((platform_id, measure_id, day))
                    }
                    ("START", None, None, day) if request.start.is_none() => {
                        starts += 1;
                        earliest = day;
                    }
                    _ => return Err(MetricReadError::Unavailable),
                }
            }
            if request.start.is_none() && starts != 1 {
                return Err(MetricReadError::Unavailable);
            }
        }
        let window = match explicit_window {
            Some(window) => window,
            None => serve(
                request,
                resolve_start(earliest, request.end)?,
                combinations.len(),
            )?,
        };
        fold_lag(&mut facts, lag, &window);

        if !scope.works.is_empty() {
            // 7. Complete months, and complete calendar years of the
            // sections.
            if window.split.has_interior() {
                let rows = read_months(
                    connection,
                    request,
                    &window,
                    &scope,
                    &platform_ids,
                    &measure_ids,
                    deadline,
                )?;
                apply_sections(rows, &window, &mut facts, &mut sections)?;
            }

            // 8. Day values.
            if window.has_day_reads() {
                read_days(
                    connection,
                    &window,
                    &scope,
                    &platform_ids,
                    &measure_ids,
                    &mut facts,
                    deadline,
                )?;
            }

            // 9. Edge days' countries and institutions.
            if window.has_edges() && (request.include_countries || request.include_institutions) {
                let (start, lead_end, trail_start, end) = window.edges();
                deadline.check()?;
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
                apply_sections(rows, &window, &mut facts, &mut sections)?;
            }
        }

        // 10. Current coverage runs of every represented publisher.
        if !accounts.is_empty() {
            deadline.check()?;
            let rows: Vec<RunRow> = diesel::sql_query(ASSERTIONS_SQL)
                .bind::<Array<SqlUuid>, _>(&account_ids)
                .bind::<Array<SqlUuid>, _>(&account_platforms)
                .bind::<Array<SqlUuid>, _>(&account_publishers)
                .bind::<Date, _>(window.start)
                .bind::<Date, _>(window.end)
                .bind::<Array<SqlUuid>, _>(&measure_ids)
                .load(connection)?;
            for row in rows {
                if row.run_start < window.start
                    || row.run_end > window.end
                    || row.run_start >= row.run_end
                {
                    return Err(MetricReadError::Unavailable);
                }
                let status = match (row.coverage_status, row.import_status.as_str()) {
                    // A terminal import that recorded row errors cannot
                    // establish complete coverage.
                    (MetricCoverageStatus::Complete, "COMPLETED_WITH_ERRORS") => {
                        MetricCoverageStatus::Partial
                    }
                    (declared, _) => declared,
                };
                runs.push(CoverageRun {
                    publisher_id: row.publisher_id,
                    platform_id: row.platform_id,
                    measure_id: row.measure_id,
                    start: row.run_start,
                    end: row.run_end,
                    coverage: DayCoverage {
                        status,
                        country: row.country_coverage,
                        institution: row.institution_coverage,
                    },
                });
            }
        }

        // 11. Identifier quality of the represented publishers. Historical
        // admitted evidence remains load-bearing even if its source is now
        // disabled; a selected publisher without a resolved work contributes
        // none.
        if !scope.publishers.is_empty() {
            deadline.check()?;
            let unresolved: Vec<IdentifierQualityRow> = diesel::sql_query(IDENTIFIER_QUALITY_SQL)
                .bind::<Array<SqlUuid>, _>(&scope.publishers)
                .bind::<Date, _>(window.start)
                .bind::<Date, _>(window.end)
                .bind::<Array<SqlUuid>, _>(&platform_ids)
                .bind::<Array<SqlUuid>, _>(&measure_ids)
                .load(connection)?;
            for row in unresolved {
                let from = row.period_start.max(window.start);
                let to = row.period_end.min(window.end);
                if let Some(pair) = facts.get_mut(&(row.platform_id, row.measure_id)) {
                    if from < to {
                        pair.quarantine.push((from, to));
                    }
                }
            }
        }
        window
    };

    assemble(
        &Evaluation {
            window: &window,
            combinations: &combinations,
            publishers: &scope.publishers,
            include_countries: request.include_countries,
            include_institutions: request.include_institutions,
            facts: &facts,
            runs: &runs,
        },
        frontier,
        sections,
    )
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
    deadline: &ReadDeadline,
) -> Result<WorkScope, MetricReadError> {
    let selector = &request.selector;
    deadline.check()?;
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
/// every identity represented, for the resolved works and range — all
/// history before endDate when startDate was omitted — in the projections,
/// in an eligible coverage run or terminal coverage of a represented
/// publisher, or in unresolved quarantine under a represented publisher's
/// immutable import scope, restricted by the other dimension when that one
/// is explicit. With no resolved work there is no represented publisher, so
/// nothing, quarantine included, is represented. Every served measure,
/// however selected, must be additive across both time and works.
fn resolve_scope(
    connection: &mut PgConnection,
    request: &ValidatedRequest,
    scope: &WorkScope,
    deadline: &ReadDeadline,
) -> Result<(BTreeSet<Uuid>, BTreeSet<Uuid>), MetricReadError> {
    let platform_filter: Vec<Uuid> = request.platforms.iter().flatten().copied().collect();
    let measure_filter: Vec<Uuid> = request.measures.iter().flatten().copied().collect();
    let mut additivity: BTreeMap<Uuid, bool> = BTreeMap::new();
    if request.platforms.is_some() || request.measures.is_some() {
        deadline.check()?;
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
                // The discovery range's edges. With an omitted start there is
                // no lower bound at all: no leading edge, every month before
                // endDate's month, and endDate's month as the trailing edge.
                let (start, lead_end, trail_start) = match request.start {
                    Some(start) => {
                        let split = Split::of(start, request.end);
                        (Some(start), Some(split.interior_start), split.interior_end)
                    }
                    None => (None, None, first_of_month(request.end)),
                };
                let lead = start.zip(lead_end).is_some_and(|(start, end)| start < end);
                let trail = trail_start < request.end;
                deadline.check()?;
                diesel::sql_query(REPRESENTED_SQL)
                    .bind::<Array<SqlUuid>, _>(&scope.works)
                    .bind::<Nullable<Date>, _>(start)
                    .bind::<Nullable<Date>, _>(lead_end)
                    .bind::<Date, _>(trail_start)
                    .bind::<Date, _>(request.end)
                    .bind::<Array<SqlUuid>, _>(&platform_filter)
                    .bind::<Array<SqlUuid>, _>(&measure_filter)
                    .bind::<Array<SqlUuid>, _>(&scope.publishers)
                    .bind::<Text, _>(DRIVER_ACQUISITION)
                    .bind::<Bool, _>(lead)
                    .bind::<Bool, _>(trail)
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

/// Read the step-7 complete-month statement the window and the resolved-work
/// count select ([`month_statement`]). A text with complete calendar years
/// also binds them as `$9` and `$10`.
fn read_months(
    connection: &mut PgConnection,
    request: &ValidatedRequest,
    window: &Window,
    scope: &WorkScope,
    platform_ids: &[Uuid],
    measure_ids: &[Uuid],
    deadline: &ReadDeadline,
) -> Result<Vec<SectionRow>, MetricReadError> {
    let (text, years) = month_statement(window, scan_strategy(scope.works.len()));
    let statement = match text {
        MonthText::Months => diesel::sql_query(MONTHS_SQL),
        MonthText::MonthsScan => diesel::sql_query(MONTHS_SCAN_SQL),
        MonthText::MonthsFullScan => diesel::sql_query(MONTHS_FULL_SCAN_SQL),
        MonthText::MonthsYear => diesel::sql_query(MONTHS_YEAR_SQL),
        MonthText::MonthsYearScan => diesel::sql_query(MONTHS_YEAR_SCAN_SQL),
        MonthText::MonthsYearFullScan => diesel::sql_query(MONTHS_YEAR_FULL_SCAN_SQL),
        MonthText::MonthsYearOnly => diesel::sql_query(MONTHS_YEAR_ONLY_SQL),
        MonthText::MonthsYearOnlyScan => diesel::sql_query(MONTHS_YEAR_ONLY_SCAN_SQL),
    }
    .into_boxed::<Pg>()
    .bind::<Array<SqlUuid>, _>(scope.works.as_slice())
    .bind::<Array<SqlUuid>, _>(platform_ids)
    .bind::<Array<SqlUuid>, _>(measure_ids)
    .bind::<Date, _>(window.split.interior_start)
    .bind::<Date, _>(window.split.interior_end)
    .bind::<Bool, _>(request.include_countries)
    .bind::<Bool, _>(request.include_institutions)
    .bind::<SqlBigInt, _>(institution_read_limit());
    let statement = match years {
        Some((first_year, years_end)) => statement
            .bind::<Date, _>(first_year)
            .bind::<Date, _>(years_end),
        None => statement,
    };
    deadline.check()?;
    Ok(statement.load(connection)?)
}

/// Read and resolve the day values of the window's day ranges.
fn read_days(
    connection: &mut PgConnection,
    window: &Window,
    scope: &WorkScope,
    platform_ids: &[Uuid],
    measure_ids: &[Uuid],
    facts: &mut BTreeMap<(Uuid, Uuid), PairFacts>,
    deadline: &ReadDeadline,
) -> Result<(), MetricReadError> {
    let (start, lead_end, trail_start, end) = window.day_reads();
    deadline.check()?;
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
        deadline.check()?;
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
    let interior = window.split.interior_start..window.split.interior_end;
    for (platform_id, measure_id, day, total, country, institution) in resolved {
        let total = BigInt::parse_canonical(&total)
            .ok_or(MetricReadError::QueryLimitExceeded(AGGREGATE_OUT_OF_RANGE))?;
        let Some(pair) = facts.get_mut(&(platform_id, measure_id)) else {
            continue;
        };
        if window.start <= day && day < window.end {
            pair.days.insert(day, total);
        }
        // A day inside a complete month contributes its dependency through
        // that month's projected row; only edge days add their own here.
        if !interior.contains(&day) {
            pair.depends_on_country |= country;
            pair.depends_on_institution |= institution;
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

/// Fold one section statement's rows into the per-pair facts and the
/// sections.
fn apply_sections(
    rows: Vec<SectionRow>,
    window: &Window,
    facts: &mut BTreeMap<(Uuid, Uuid), PairFacts>,
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
    let interior = window.split.interior_start..window.split.interior_end;
    for row in rows {
        match (row.section.as_str(), row.platform_id, row.measure_id) {
            ("AMBIGUITY", _, _) => {}
            ("TOTAL", Some(platform_id), Some(measure_id)) => {
                // Every monthly total is of a complete month of the interior.
                let month = row
                    .month_start
                    .filter(|month| month.day() == 1 && interior.contains(month))
                    .ok_or(MetricReadError::Unavailable)?;
                let value = parse(row.total)?;
                if let Some(pair) = facts.get_mut(&(platform_id, measure_id)) {
                    pair.months.insert(month, value);
                    pair.depends_on_country |= row.requires_country;
                    pair.depends_on_institution |= row.requires_institution;
                }
            }
            ("COUNTRY", Some(platform_id), Some(measure_id)) => {
                let code = row.country_code.ok_or(MetricReadError::Unavailable)?;
                let value = parse(row.total)?;
                let key = (platform_id, measure_id, code);
                let current = sections.countries.get(&key).copied();
                sections.countries.insert(key, add(current, value)?);
                if let Some(pair) = facts.get_mut(&(platform_id, measure_id)) {
                    pair.country_depends_on_institution |= row.requires_institution;
                }
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
                if let Some(pair) = facts.get_mut(&(platform_id, measure_id)) {
                    pair.institution_depends_on_country |= row.requires_country;
                }
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
// The served range
// ---------------------------------------------------------------------------

/// The served range, its timeline and its month split, fixed once the start
/// is known (#980 Amendment 3 section 4B).
#[derive(Debug, Clone, PartialEq, Eq)]
struct Window {
    start: NaiveDate,
    end: NaiveDate,
    /// The served grain, never [`MetricTimelineGrain::Auto`].
    grain: MetricTimelineGrain,
    buckets: Vec<(NaiveDate, NaiveDate)>,
    split: Split,
}

impl Window {
    fn new(start: NaiveDate, end: NaiveDate, grain: MetricTimelineGrain) -> Self {
        Window {
            start,
            end,
            grain,
            buckets: buckets_of(start, end, grain),
            split: Split::of(start, end),
        }
    }

    /// Whether the timeline is served from complete months and clipped edge
    /// days: `MONTH` and `YEAR` are; `DAY` reads every day.
    fn monthly_timeline(&self) -> bool {
        matches!(
            self.grain,
            MetricTimelineGrain::Month | MetricTimelineGrain::Year
        )
    }

    /// The day ranges whose values are read from the work-day projection,
    /// as `(start, lead_end, trail_start, end)`: the clipped edges for a
    /// monthly timeline, every day for a daily one.
    fn day_reads(&self) -> (NaiveDate, NaiveDate, NaiveDate, NaiveDate) {
        if self.monthly_timeline() {
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
        !self.monthly_timeline() || self.has_edges()
    }

    /// The complete months of the interior wholly inside `[from, to)`, as a
    /// half-open span of month starts, or an empty span at `to`. Inside it a
    /// value comes from the monthly projection; every other day of
    /// `[from, to)` comes from the work-day projection.
    fn monthly_span(&self, from: NaiveDate, to: NaiveDate) -> (NaiveDate, NaiveDate) {
        let first = self.split.interior_start.max(first_full_month(from));
        let last = self.split.interior_end.min(first_of_month(to));
        if first < last {
            (first, last)
        } else {
            (to, to)
        }
    }
}

/// The first day of `day`'s month.
fn first_of_month(day: NaiveDate) -> NaiveDate {
    day.with_day(1).unwrap_or(day)
}

/// The first day of the first month that starts on or after `day`.
fn first_full_month(day: NaiveDate) -> NaiveDate {
    if day.day() == 1 {
        day
    } else {
        next_month(day)
    }
}

/// How many timeline buckets `[start, end)` has at `grain`, without building
/// them.
fn bucket_count(start: NaiveDate, end: NaiveDate, grain: MetricTimelineGrain) -> usize {
    let Some(last) = end.pred_opt().filter(|last| start <= *last) else {
        return 0;
    };
    let count = match grain {
        MetricTimelineGrain::Auto | MetricTimelineGrain::Day => (end - start).num_days(),
        MetricTimelineGrain::Month => {
            i64::from(last.year() - start.year()) * 12 + i64::from(last.month())
                - i64::from(start.month())
                + 1
        }
        MetricTimelineGrain::Year => i64::from(last.year() - start.year()) + 1,
    };
    usize::try_from(count).unwrap_or(usize::MAX)
}

/// Fix the served range `[start, request.end)` (#980 Amendment 3 sections 2
/// and 4B): choose the grain — AUTO is `DAY` when the range is at most 366
/// days and its cells fit, otherwise `MONTH` when its cells fit, otherwise
/// `YEAR` — and refuse a timeline of more than
/// [`METRIC_DASHBOARD_MAX_TIMELINE_CELLS`] cells. An explicit grain never
/// switches.
fn serve(
    request: &ValidatedRequest,
    start: NaiveDate,
    combinations: usize,
) -> Result<Window, MetricReadError> {
    let end = request.end;
    let cells =
        |grain: MetricTimelineGrain| bucket_count(start, end, grain).saturating_mul(combinations);
    let grain = match request.grain {
        MetricTimelineGrain::Auto => {
            if (end - start).num_days() <= METRIC_DASHBOARD_MAX_RANGE_DAYS
                && cells(MetricTimelineGrain::Day) <= METRIC_DASHBOARD_MAX_TIMELINE_CELLS
            {
                MetricTimelineGrain::Day
            } else if cells(MetricTimelineGrain::Month) <= METRIC_DASHBOARD_MAX_TIMELINE_CELLS {
                MetricTimelineGrain::Month
            } else {
                MetricTimelineGrain::Year
            }
        }
        grain => grain,
    };
    if cells(grain) > METRIC_DASHBOARD_MAX_TIMELINE_CELLS {
        return Err(MetricReadError::QueryLimitExceeded(TOO_MANY_CELLS));
    }
    Ok(Window::new(start, end, grain))
}

/// The start served when startDate was omitted and the scope holds no
/// evidence before endDate: the first day of the month holding the day
/// before endDate.
fn fallback_start(end: NaiveDate) -> NaiveDate {
    first_of_month(end.pred_opt().unwrap_or(end))
}

/// The start served when startDate was omitted (#980 Amendment 3 section 4):
/// the exact day of the earliest evidence, never clamped, or the fallback
/// month when there is none. Evidence dated before
/// [`METRIC_DASHBOARD_EARLIEST_DATE`] refuses the request.
fn resolve_start(
    earliest: Option<NaiveDate>,
    end: NaiveDate,
) -> Result<NaiveDate, MetricReadError> {
    match earliest {
        Some(day) if day < METRIC_DASHBOARD_EARLIEST_DATE => {
            Err(MetricReadError::QueryLimitExceeded(EVIDENCE_BEFORE_FLOOR))
        }
        Some(day) => Ok(day),
        None => Ok(fallback_start(end)),
    }
}

/// The step-7 physical read strategy (#980 Amendment 2 section 4.2), fixed
/// by the resolved-work count alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScanStrategy {
    /// Every branch probes the work-leading identity indexes.
    Probe,
    /// The monthly total, monthly ambiguity and yearly branches scan.
    ScanProjections,
    /// Every branch scans.
    ScanAll,
}

fn scan_strategy(resolved_works: usize) -> ScanStrategy {
    if resolved_works >= METRIC_DASHBOARD_SECTION_SCAN_MIN_WORKS {
        ScanStrategy::ScanAll
    } else if resolved_works >= METRIC_DASHBOARD_SCAN_STRATEGY_MIN_WORKS {
        ScanStrategy::ScanProjections
    } else {
        ScanStrategy::Probe
    }
}

/// One text of the step-7 family (#980 Amendment 2 section 4.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MonthText {
    Months,
    MonthsScan,
    MonthsFullScan,
    MonthsYear,
    MonthsYearScan,
    MonthsYearFullScan,
    MonthsYearOnly,
    MonthsYearOnlyScan,
}

/// The complete calendar years `[first_year, years_end)` wholly inside the
/// window's interior, as their first days, if there is at least one.
fn complete_years(split: &Split) -> Option<(NaiveDate, NaiveDate)> {
    if !split.has_interior() {
        return None;
    }
    let start = split.interior_start;
    let first_year = if start.month() == 1 {
        start
    } else {
        NaiveDate::from_ymd_opt(start.year() + 1, 1, 1)?
    };
    let years_end = NaiveDate::from_ymd_opt(split.interior_end.year(), 1, 1)?;
    (first_year < years_end).then_some((first_year, years_end))
}

/// The step-7 text for a window and strategy, with the complete years it
/// binds. A `MONTH` or `YEAR` timeline reads the complete calendar years of
/// its interior from the yearly section projections; a `DAY` timeline, and
/// an interior without a complete year, keep the ordinary family.
fn month_statement(
    window: &Window,
    strategy: ScanStrategy,
) -> (MonthText, Option<(NaiveDate, NaiveDate)>) {
    let years = if window.monthly_timeline() {
        complete_years(&window.split)
    } else {
        None
    };
    let text = match (years, strategy) {
        (None, ScanStrategy::Probe) => MonthText::Months,
        (None, ScanStrategy::ScanProjections) => MonthText::MonthsScan,
        (None, ScanStrategy::ScanAll) => MonthText::MonthsFullScan,
        (Some((first_year, years_end)), strategy)
            if first_year == window.split.interior_start
                && years_end == window.split.interior_end =>
        {
            match strategy {
                ScanStrategy::Probe => MonthText::MonthsYearOnly,
                ScanStrategy::ScanProjections | ScanStrategy::ScanAll => {
                    MonthText::MonthsYearOnlyScan
                }
            }
        }
        (Some(_), ScanStrategy::Probe) => MonthText::MonthsYear,
        (Some(_), ScanStrategy::ScanProjections) => MonthText::MonthsYearScan,
        (Some(_), ScanStrategy::ScanAll) => MonthText::MonthsYearFullScan,
    };
    (text, years)
}

// ---------------------------------------------------------------------------
// Assembly: the interval engine
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

/// The values and evidence read for one served platform and measure, held
/// as sparse rows and intervals: nothing here is sized by the days of the
/// range.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct PairFacts {
    /// Complete-month totals from the monthly projection, by month start.
    months: BTreeMap<NaiveDate, BigInt>,
    /// Day totals from the work-day projection, by day.
    days: BTreeMap<NaiveDate, BigInt>,
    /// Ordered, disjoint runs of days touched by outstanding rollup work.
    lag: Vec<(NaiveDate, NaiveDate)>,
    /// Unresolved identifier evidence, clipped to the served range.
    quarantine: Vec<(NaiveDate, NaiveDate)>,
    /// Whether any total or timeline value was taken from rows broken down
    /// by country, so complete totals also need the country dimension.
    depends_on_country: bool,
    /// The same for the institution dimension.
    depends_on_institution: bool,
    /// Whether any country value was taken from rows also broken down by
    /// institution.
    country_depends_on_institution: bool,
    /// Whether any institution value was taken from rows also broken down by
    /// country.
    institution_depends_on_country: bool,
}

impl PairFacts {
    /// The value of `[from, to)` inside the window: each complete month of
    /// the interior wholly inside it from the monthly projection, and every
    /// other day of it from the work-day projection.
    ///
    /// Projected rows win: their exact sum is the value. With no rows, zero
    /// is returned only when zero is justified over every day of the range;
    /// otherwise the value is unknown.
    fn value(
        &self,
        window: &Window,
        coverage: &PairCoverage,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Option<BigInt>, MetricReadError> {
        let (months_from, months_to) = window.monthly_span(from, to);
        let months = self
            .months
            .range(months_from..months_to)
            .map(|(_, value)| value);
        let days = self
            .days
            .range(from..months_from)
            .chain(self.days.range(months_to..to))
            .map(|(_, value)| value);
        combine(months.chain(days), || coverage.justified(from, to))
    }
}

/// The exact sum of some contributions, or zero when there are none and zero
/// is `justified`, or unknown.
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

/// One clipped coverage run of a represented publisher.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CoverageRun {
    publisher_id: Uuid,
    platform_id: Uuid,
    measure_id: Uuid,
    start: NaiveDate,
    end: NaiveDate,
    coverage: DayCoverage,
}

/// A maximal interval of constant combined coverage; `None` is `UNKNOWN`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Segment {
    start: NaiveDate,
    end: NaiveDate,
    coverage: Option<DayCoverage>,
}

/// The open runs at one point of the sweep, counted so that a closing run
/// undoes exactly what its opening did.
#[derive(Debug, Default)]
struct Tally {
    covering: usize,
    unknown: usize,
    partial: usize,
    without_country: usize,
    without_institution: usize,
}

impl Tally {
    fn apply(&mut self, opens: bool, coverage: DayCoverage) {
        let step = |count: &mut usize, applies: bool| {
            if applies {
                *count = if opens {
                    *count + 1
                } else {
                    count.saturating_sub(1)
                };
            }
        };
        step(&mut self.covering, true);
        step(
            &mut self.unknown,
            coverage.status == MetricCoverageStatus::Unknown,
        );
        step(
            &mut self.partial,
            coverage.status == MetricCoverageStatus::Partial,
        );
        step(&mut self.without_country, !coverage.country);
        step(&mut self.without_institution, !coverage.institution);
    }

    /// The combined coverage across `publishers` represented publishers:
    /// `UNKNOWN` unless every one of them asserts it, then the worst status,
    /// and each dimension only if every assertion includes it. With no
    /// represented publisher there is nothing to assert.
    fn combined(&self, publishers: usize) -> Option<DayCoverage> {
        (publishers > 0 && self.covering == publishers).then_some(DayCoverage {
            status: if self.unknown > 0 {
                MetricCoverageStatus::Unknown
            } else if self.partial > 0 {
                MetricCoverageStatus::Partial
            } else {
                MetricCoverageStatus::Complete
            },
            country: self.without_country == 0,
            institution: self.without_institution == 0,
        })
    }
}

/// Group the runs by served platform and measure, keeping only those of the
/// represented publishers, and refuse same-stream overlap (#980 Amendment 2
/// section 5): two runs of one publisher, platform and measure — one stream,
/// since at most one eligible account serves a publisher and platform — that
/// cover the same served day are invalid derived state with no winner
/// between them, so the read fails as [`MetricReadError::Unavailable`]
/// rather than choose one.
fn coverage_streams(
    runs: &[CoverageRun],
    publishers: &[Uuid],
) -> Result<BTreeMap<(Uuid, Uuid), Vec<CoverageRun>>, MetricReadError> {
    let mut ordered: Vec<CoverageRun> = runs.to_vec();
    ordered.sort_by_key(|run| (run.platform_id, run.measure_id, run.publisher_id, run.start));
    // Sorted by start, a stream's runs are disjoint exactly when each one
    // ends no later than the next one starts.
    if ordered.windows(2).any(|pair| {
        (
            pair[0].platform_id,
            pair[0].measure_id,
            pair[0].publisher_id,
        ) == (
            pair[1].platform_id,
            pair[1].measure_id,
            pair[1].publisher_id,
        ) && pair[1].start < pair[0].end
    }) {
        return Err(MetricReadError::Unavailable);
    }
    let represented: BTreeSet<&Uuid> = publishers.iter().collect();
    let mut streams: BTreeMap<(Uuid, Uuid), Vec<CoverageRun>> = BTreeMap::new();
    for run in ordered {
        if represented.contains(&run.publisher_id) {
            streams
                .entry((run.platform_id, run.measure_id))
                .or_default()
                .push(run);
        }
    }
    record_size("runs", streams.values().map(Vec::len).sum());
    Ok(streams)
}

/// Tile `[start, end)` with maximal segments of constant combined coverage
/// from one pair's runs, which are already clipped to it, by one sweep over
/// their boundaries.
fn segments(
    start: NaiveDate,
    end: NaiveDate,
    runs: &[CoverageRun],
    publishers: usize,
) -> Vec<Segment> {
    // A run closing on a day is applied before one opening on it, so the
    // counts never go below zero.
    let mut events: Vec<(NaiveDate, bool, DayCoverage)> = runs
        .iter()
        .flat_map(|run| {
            [
                (run.start, true, run.coverage),
                (run.end, false, run.coverage),
            ]
        })
        .collect();
    events.sort_by_key(|(day, opens, _)| (*day, *opens));
    record_size("events", events.len());
    let mut tally = Tally::default();
    let mut segments: Vec<Segment> = Vec::new();
    let mut next_event = 0;
    let mut cursor = start;
    while cursor < end {
        while let Some((day, opens, coverage)) = events.get(next_event) {
            if *day > cursor {
                break;
            }
            tally.apply(*opens, *coverage);
            next_event += 1;
        }
        let segment_end = events
            .get(next_event)
            .map_or(end, |(day, _, _)| (*day).min(end));
        let coverage = tally.combined(publishers);
        match segments.last_mut() {
            Some(last) if last.coverage == coverage => last.end = segment_end,
            _ => segments.push(Segment {
                start: cursor,
                end: segment_end,
                coverage,
            }),
        }
        cursor = segment_end;
    }
    record_size("segments", segments.len());
    segments
}

/// Sort intervals and merge the overlapping and adjacent ones.
fn merge_intervals(mut intervals: Vec<(NaiveDate, NaiveDate)>) -> Vec<(NaiveDate, NaiveDate)> {
    intervals.sort_unstable();
    let mut merged: Vec<(NaiveDate, NaiveDate)> = Vec::with_capacity(intervals.len());
    for (start, end) in intervals {
        match merged.last_mut() {
            Some(last) if start <= last.1 => last.1 = last.1.max(end),
            _ => merged.push((start, end)),
        }
    }
    merged
}

/// Clip intervals to `[start, end)`, dropping the ones left empty.
fn clipped(
    intervals: &[(NaiveDate, NaiveDate)],
    start: NaiveDate,
    end: NaiveDate,
) -> Vec<(NaiveDate, NaiveDate)> {
    intervals
        .iter()
        .map(|(from, to)| ((*from).max(start), (*to).min(end)))
        .filter(|(from, to)| from < to)
        .collect()
}

/// The coverage of one served platform and measure over the window, as
/// intervals (#980 Amendment 3 section 5).
#[derive(Debug, Clone, PartialEq, Eq)]
struct PairCoverage {
    /// Ordered, disjoint intervals on which zero is not justified: days not
    /// effectively `COMPLETE` for the totals and timeline, days touched by
    /// outstanding rollup work and days overlapped by unresolved identifier
    /// evidence.
    unjustified: Vec<(NaiveDate, NaiveDate)>,
    /// The first day not `COMPLETE` across every returned section, or
    /// touched by outstanding rollup work.
    first_break: Option<NaiveDate>,
    unknown: bool,
    partial: bool,
    lag: bool,
    identifier_incomplete: bool,
    country_coverage: bool,
    institution_coverage: bool,
}

impl PairCoverage {
    /// Derive the pair's coverage per section (#946 Amendment 1).
    ///
    /// The totals and timeline need only the dimensions their own served
    /// representation depends on, range-wide per pair, exactly as the
    /// `MET-WP4-02` rule: that alone decides their zero or null. The country
    /// section, when returned, always needs country coverage and also needs
    /// institution coverage if a country value came from rows broken down by
    /// institution; the institution section mirrors it. The shared status —
    /// items, top-level status, `dataThrough` and warnings — is the worst of
    /// the returned sections, so an omitted section cannot downgrade it
    /// through its own dimension, and a returned one never changes a total
    /// or timeline value.
    fn of(evaluation: &Evaluation<'_>, facts: &PairFacts, runs: &[CoverageRun]) -> PairCoverage {
        let window = evaluation.window;
        let segments = segments(window.start, window.end, runs, evaluation.publishers.len());
        let lag = merge_intervals(clipped(&facts.lag, window.start, window.end));
        let quarantine = clipped(&facts.quarantine, window.start, window.end);
        record_size("lag", lag.len());
        record_size("quarantine", quarantine.len());
        let mut unjustified: Vec<(NaiveDate, NaiveDate)> =
            Vec::with_capacity(segments.len() + lag.len() + quarantine.len());
        let mut first_break = lag.first().map(|(start, _)| *start);
        let mut coverage = PairCoverage {
            unjustified: Vec::new(),
            first_break: None,
            unknown: false,
            partial: false,
            lag: !lag.is_empty(),
            identifier_incomplete: !quarantine.is_empty(),
            country_coverage: true,
            institution_coverage: true,
        };
        for segment in &segments {
            let total = effective(
                segment.coverage,
                facts.depends_on_country,
                facts.depends_on_institution,
            );
            let mut shared = total;
            if evaluation.include_countries {
                shared = worst(
                    shared,
                    effective(segment.coverage, true, facts.country_depends_on_institution),
                );
            }
            if evaluation.include_institutions {
                shared = worst(
                    shared,
                    effective(segment.coverage, facts.institution_depends_on_country, true),
                );
            }
            if total != MetricCoverageStatus::Complete {
                unjustified.push((segment.start, segment.end));
            }
            if shared != MetricCoverageStatus::Complete {
                first_break = Some(first_break.map_or(segment.start, |day| day.min(segment.start)));
            }
            coverage.unknown |= shared == MetricCoverageStatus::Unknown;
            coverage.partial |= shared == MetricCoverageStatus::Partial;
            coverage.country_coverage &= segment.coverage.is_some_and(|day| day.country);
            coverage.institution_coverage &= segment.coverage.is_some_and(|day| day.institution);
        }
        unjustified.extend(lag);
        unjustified.extend(quarantine);
        coverage.unjustified = merge_intervals(unjustified);
        record_size("unjustified", coverage.unjustified.len());
        coverage.first_break = first_break;
        coverage
    }

    /// Whether zero is justified for `[from, to)`: every day is effectively
    /// `COMPLETE` for the totals and timeline, untouched by outstanding
    /// rollup work and free of unresolved identifier evidence. Complete
    /// months and edge days share this one rule.
    fn justified(&self, from: NaiveDate, to: NaiveDate) -> bool {
        let next = self.unjustified.partition_point(|(_, end)| *end <= from);
        self.unjustified
            .get(next)
            .is_none_or(|(start, _)| *start >= to)
    }
}

/// Everything the interval engine evaluates for one read.
struct Evaluation<'a> {
    window: &'a Window,
    combinations: &'a [(Uuid, Uuid)],
    /// The represented publishers, whose coverage combines per day.
    publishers: &'a [Uuid],
    include_countries: bool,
    include_institutions: bool,
    facts: &'a BTreeMap<(Uuid, Uuid), PairFacts>,
    runs: &'a [CoverageRun],
}

/// Record the element count of one internal structure, so the tests can
/// show that none grows with the days of the range.
#[cfg(test)]
fn record_size(structure: &'static str, len: usize) {
    ENGINE_SIZES.with(|sizes| {
        let mut sizes = sizes.borrow_mut();
        let largest = sizes.entry(structure).or_default();
        *largest = (*largest).max(len);
    });
}

#[cfg(not(test))]
fn record_size(_structure: &'static str, _len: usize) {}

#[cfg(test)]
thread_local! {
    /// The largest element count each internal structure reached on this
    /// thread, by structure.
    pub(crate) static ENGINE_SIZES: std::cell::RefCell<BTreeMap<&'static str, usize>> =
        const { std::cell::RefCell::new(BTreeMap::new()) };
}

fn assemble(
    evaluation: &Evaluation<'_>,
    frontier: FrontierRow,
    sections: Sections,
) -> Result<MetricDashboard, MetricReadError> {
    let window = evaluation.window;
    let combinations = evaluation.combinations;
    let streams = coverage_streams(evaluation.runs, evaluation.publishers)?;
    record_size("buckets", window.buckets.len());
    let mut totals = Vec::with_capacity(combinations.len());
    let mut timeline = Vec::with_capacity(combinations.len() * window.buckets.len());
    let mut items = Vec::with_capacity(combinations.len());
    let mut any_unknown = combinations.is_empty();
    let mut any_partial = false;
    let mut any_identifier_incomplete = false;
    let mut any_lag = false;
    let no_facts = PairFacts::default();

    for (platform_id, measure_id) in combinations {
        let key = (*platform_id, *measure_id);
        let facts = evaluation.facts.get(&key).unwrap_or(&no_facts);
        record_size("months", facts.months.len());
        record_size("days", facts.days.len());
        let runs = streams.get(&key).map_or(&[][..], Vec::as_slice);
        let coverage = PairCoverage::of(evaluation, facts, runs);

        totals.push(MetricTotal {
            platform_id: *platform_id,
            measure_id: *measure_id,
            value: facts.value(window, &coverage, window.start, window.end)?,
        });
        for (bucket_start, bucket_end) in &window.buckets {
            timeline.push(MetricTimeBucket {
                platform_id: *platform_id,
                measure_id: *measure_id,
                start_date: *bucket_start,
                end_date: *bucket_end,
                value: facts.value(window, &coverage, *bucket_start, *bucket_end)?,
            });
        }

        any_unknown |= coverage.unknown;
        any_partial |= coverage.partial;
        any_identifier_incomplete |= coverage.identifier_incomplete;
        any_lag |= coverage.lag;

        let status = if coverage.unknown {
            MetricCoverageStatus::Unknown
        } else if coverage.partial {
            MetricCoverageStatus::Partial
        } else {
            MetricCoverageStatus::Complete
        };
        let data_through = match coverage.first_break {
            Some(day) if day <= window.start => None,
            Some(day) => day.pred_opt(),
            None => window.end.pred_opt(),
        };
        items.push(MetricCoverageItem {
            platform_id: *platform_id,
            measure_id: *measure_id,
            status,
            data_through,
            country_coverage: coverage.country_coverage,
            institution_coverage: coverage.institution_coverage,
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
        start_date: window.start,
        end_date: window.end,
        timeline_grain: window.grain,
    })
}

/// Fold the `LAG` branch's days into per-pair runs of consecutive days inside
/// the served range (#980 Amendment 3 section 5).
fn fold_lag(
    facts: &mut BTreeMap<(Uuid, Uuid), PairFacts>,
    mut days: Vec<(Uuid, Uuid, NaiveDate)>,
    window: &Window,
) {
    days.sort_unstable();
    days.dedup();
    for (platform_id, measure_id, day) in days {
        if day < window.start || day >= window.end {
            continue;
        }
        let Some(pair) = facts.get_mut(&(platform_id, measure_id)) else {
            continue;
        };
        let next = day.succ_opt().unwrap_or(window.end);
        match pair.lag.last_mut() {
            Some(last) if last.1 == day => last.1 = next,
            _ => pair.lag.push((day, next)),
        }
    }
}

#[cfg(test)]
pub(crate) mod tests;
