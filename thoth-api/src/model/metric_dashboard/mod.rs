//! The minimum protected, coverage-aware Metrics read surface (`MET-WP4-02`).
//!
//! This module owns the MOM-1 `metricDashboard` contract: its input, its
//! response and the one read that answers it. It consumes the `MET-WP4-01`
//! work-day projection and strict-frontier watermark and the `MET-WP1-05`
//! coverage evidence; it owns neither and writes neither.
//!
//! # One snapshot
//!
//! [`metric_dashboard`] evaluates everything on one PostgreSQL connection in
//! one `READ ONLY, REPEATABLE READ` transaction. Publisher entitlement, the
//! rollup frontier, registry scope, projected totals, current coverage and
//! outstanding rollup work are therefore all facts about the same database
//! state, and `asOf` is that transaction's own timestamp. Nothing here
//! advances the frontier, claims or completes a delta, repairs a projection
//! row or records coverage.
//!
//! # Zero is a claim
//!
//! A value of `"0"` asserts that nothing happened. It is returned for an
//! empty cell only when every day of that cell is effectively `COMPLETE`,
 //! no unapplied rollup work touches it and no unresolved identifier evidence
//! overlaps it; otherwise an empty cell is `null`. A cell that does hold
//! projected rows always returns their exact sum, and the coverage items and
//! warnings say whether that sum can be relied on.
//!
//! # Publisher cardinality
//!
//! The public selector is the plural `publisherIds`, and the SQL below is
//! written over a publisher *set*. MOM-1 accepts exactly one publisher at
//! runtime: that is a milestone restriction on this slice, not a statement
//! that a dashboard can only ever represent one publisher. Serving an
//! authorized group of publishers needs its own entitlement and aggregation
//! decisions and is deliberately not implemented here.
//!
//! # Deliberately absent
//!
//! Countries, institutions and works response sections, pagination, the
//! deferred selector dimensions, `metricWidget`, entity-level Metrics fields
//! and any call to OPERAS.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{Datelike, Duration, NaiveDate};
use diesel::pg::PgConnection;
use diesel::sql_types::{
    Array, BigInt as SqlBigInt, Bool, Date, Text, Timestamptz, Uuid as SqlUuid,
};
use diesel::RunQueryDsl;
use juniper::{graphql_value, FieldError, IntoFieldError};
use thoth_errors::ThothError;
use uuid::Uuid;

use crate::db::PgPool;
use crate::graphql::scalars::BigInt;
use crate::model::metric_coverage::MetricCoverageStatus;
use crate::model::publisher::{PublisherCapability, ThothPackage};
use crate::model::Timestamp;

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

/// Which publishers' works a dashboard describes.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(
        description = "Which publishers' works a Metrics dashboard describes. Works are attributed to publishers through current Thoth metadata"
    )
)]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MetricSelectorInput {
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "Thoth IDs of the publishers to describe. This service currently requires exactly one publisher per request; requests with none or several are rejected, never truncated or combined"
        )
    )]
    pub publisher_ids: Option<Vec<Uuid>>,
}

/// One MOM-1 dashboard request.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(description = "A bounded, coverage-aware Metrics dashboard request")
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricDashboardInput {
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Which publishers to describe")
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
            description = "Thoth IDs of the measures to serve, from at most 10 unique IDs. Omitted or empty means every measure represented for the selected publishers and range. Every served measure must be additive across time and works"
        )
    )]
    pub measures: Option<Vec<Uuid>>,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "Thoth IDs of the platforms to serve, from at most 10 unique IDs. Omitted or empty means every platform represented for the selected publishers and range"
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
}

// ---------------------------------------------------------------------------
// Response
// ---------------------------------------------------------------------------

/// One MOM-1 dashboard response.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLObject),
    graphql(
        description = "Coverage-aware Metrics totals and timeline for one bounded request, read from one consistent database snapshot"
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
        graphql(description = "How completely the served values are covered")
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
            description = "COMPLETE only when every day is effectively COMPLETE; UNKNOWN when any day is UNKNOWN; otherwise PARTIAL"
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
    /// Authentication, the `METRICS_READ_SERVICE` role or the selected
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
    /// An unexpected database or pool failure, deliberately without detail.
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
                "A served work, platform, measure and day is recorded in more than one incompatible dimensional breakdown without an undimensioned total, so its value cannot be resolved."
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
    "This dashboard currently requires exactly one publisher in selector.publisherIds.";
const UNKNOWN_PUBLISHER: &str = "selector.publisherIds contains an unknown publisher ID.";
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

/// A request that passed every check that needs no database.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ValidatedRequest {
    publisher_ids: Vec<Uuid>,
    start: NaiveDate,
    end: NaiveDate,
    /// `None` when the dimension was omitted or empty.
    measures: Option<BTreeSet<Uuid>>,
    platforms: Option<BTreeSet<Uuid>>,
    buckets: Vec<(NaiveDate, NaiveDate)>,
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
                let (year, month) = if bucket_start.month() == 12 {
                    (bucket_start.year() + 1, 1)
                } else {
                    (bucket_start.year(), bucket_start.month() + 1)
                };
                let next_month = NaiveDate::from_ymd_opt(year, month, 1).unwrap_or(end);
                let bucket_end = next_month.min(end);
                buckets.push((bucket_start, bucket_end));
                bucket_start = bucket_end;
            }
            buckets
        }
    }
}

/// A filter as a set, rejecting duplicates and enforcing the per-dimension
/// bound. Omitted and empty both mean "resolve from represented state".
fn filter_set(
    values: &Option<Vec<Uuid>>,
    duplicate: &'static str,
    max: usize,
    too_many: &'static str,
) -> Result<Option<BTreeSet<Uuid>>, MetricReadError> {
    let Some(values) = values.as_ref().filter(|values| !values.is_empty()) else {
        return Ok(None);
    };
    let set: BTreeSet<Uuid> = values.iter().copied().collect();
    if set.len() != values.len() {
        return Err(MetricReadError::QueryInvalid(duplicate));
    }
    if set.len() > max {
        return Err(MetricReadError::QueryLimitExceeded(too_many));
    }
    Ok(Some(set))
}

fn validate(input: &MetricDashboardInput) -> Result<ValidatedRequest, MetricReadError> {
    let publisher_ids = input.selector.publisher_ids.clone().unwrap_or_default();
    // MOM-1 runtime cardinality. Zero and several are both refused outright:
    // nothing here picks the first entry, drops the rest or combines them.
    if publisher_ids.len() != 1 {
        return Err(MetricReadError::QueryInvalid(PUBLISHER_CARDINALITY));
    }
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
        start: input.start_date,
        end: input.end_date,
        measures,
        platforms,
        buckets: buckets_of(input.start_date, input.end_date, grain),
    })
}

// ---------------------------------------------------------------------------
// Row shapes
// ---------------------------------------------------------------------------

#[derive(diesel::QueryableByName)]
struct PublisherRow {
    #[diesel(sql_type = SqlUuid)]
    publisher_id: Uuid,
    #[diesel(sql_type = crate::schema::sql_types::ThothPackage)]
    subscription_package: ThothPackage,
}

#[derive(diesel::QueryableByName)]
struct FrontierRow {
    #[diesel(sql_type = Timestamptz)]
    as_of: Timestamp,
    #[diesel(sql_type = SqlBigInt)]
    applied_through_sequence: i64,
    #[diesel(sql_type = SqlBigInt)]
    next_sequence: i64,
    #[diesel(sql_type = Timestamptz)]
    watermark_at: Timestamp,
}

#[derive(diesel::QueryableByName)]
struct IdRow {
    #[diesel(sql_type = SqlUuid)]
    id: Uuid,
}

#[derive(diesel::QueryableByName)]
struct MeasureFlagsRow {
    #[diesel(sql_type = Bool)]
    additive_across_time: bool,
    #[diesel(sql_type = Bool)]
    additive_across_works: bool,
}

#[derive(diesel::QueryableByName)]
struct PairRow {
    #[diesel(sql_type = SqlUuid)]
    platform_id: Uuid,
    #[diesel(sql_type = SqlUuid)]
    measure_id: Uuid,
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
struct LagRow {
    #[diesel(sql_type = SqlUuid)]
    platform_id: Uuid,
    #[diesel(sql_type = SqlUuid)]
    measure_id: Uuid,
    #[diesel(sql_type = Date)]
    day: NaiveDate,
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
// the read executes.
// ---------------------------------------------------------------------------

/// Step 1: the selected publishers and their packages.
pub(crate) const PUBLISHERS_SQL: &str = "SELECT publisher_id, subscription_package \
         FROM public.publisher \
         WHERE publisher_id = ANY($1)";

/// Step 2: the transaction timestamp and the `MET-WP4-01` frontier.
pub(crate) const FRONTIER_SQL: &str = "SELECT transaction_timestamp() AS as_of, \
                applied_through_sequence, next_sequence, watermark_at \
         FROM public.metric_rollup_work_day_state \
         WHERE state_id = 1";

/// Step 4: eligible managed source accounts per publisher and platform.
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

/// Step 5a: projected values per platform, measure and day, attributed to
/// publishers through current `work -> imprint` ownership, with the facts the
/// MOM-1 dimensional representation rule (Specification Amendment 6) needs.
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
             JOIN public.work w ON w.work_id = r.work_id \
             JOIN public.imprint i ON i.imprint_id = w.imprint_id \
             WHERE i.publisher_id = ANY($1) \
               AND r.day >= $2 \
               AND r.day < $3 \
               AND r.platform_id = ANY($4) \
               AND r.measure_id = ANY($5) \
             GROUP BY r.platform_id, r.measure_id, r.day";

/// Step 5b: the base-cell resolution of the candidate groups step 5a could not
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
                 JOIN public.work w ON w.work_id = r.work_id \
                 JOIN public.imprint i ON i.imprint_id = w.imprint_id \
                 WHERE i.publisher_id = ANY($1) \
                   AND r.platform_id = ANY($2) \
                   AND r.measure_id = ANY($3) \
                   AND r.day = ANY($4) \
                 GROUP BY r.platform_id, r.measure_id, r.day, r.work_id \
             ) cell \
             GROUP BY cell.platform_id, cell.measure_id, cell.day";

/// Step 6: the current terminal coverage assertion per platform, measure and
/// day.
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
/// The ownership check is wrapped in a `CASE` that is `TRUE` only when both
/// equalities hold, so it rejects exactly what they reject, NULL included.
/// As plain equalities the planner multiplies in their selectivity, expects
/// about one owned coverage row and stops materializing the day series,
/// re-evaluating it once per coverage row (about five times slower over a
/// year); the `CASE` keeps that estimate out of the plan.
pub(crate) const ASSERTIONS_SQL: &str = "SELECT DISTINCT ON (c.platform_id, c.measure_id, d.day) \
                        c.platform_id, c.measure_id, d.day, c.coverage_status, \
                        mi.status::text AS import_status, \
                        c.country_coverage, c.institution_coverage \
                 FROM (SELECT generate_series($2::date, $3::date - 1, interval '1 day')::date \
                           AS day) d \
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
                 ORDER BY c.platform_id, c.measure_id, d.day, \
                          mi.completed_at DESC, mi.import_id DESC, \
                          c.coverage_status DESC, c.country_coverage ASC, \
                          c.institution_coverage ASC, c.coverage_id DESC";

/// Step 8: identifier-unresolved quarantine evidence intersecting the served
/// publisher, platform, measure and date scope. Historical import publisher
/// scope remains authoritative and source enablement is deliberately ignored.
/// Absence of reconciliation state and every nonterminal state both have
/// `resolved_at IS NULL`; terminal resolution removes the warning.
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

/// Step 7: unapplied work-day deltas above `W` that intersect the request.
///
/// Every position above `W` is unapplied under the strict frontier, so no
/// status filter is needed.
pub(crate) const LAG_SQL: &str =
    "SELECT DISTINCT r.platform_id, r.measure_id, r.period_start AS day \
                 FROM public.metric_rollup_delta d \
                 JOIN public.metric_record r ON r.record_id = d.record_id \
                 JOIN public.work w ON w.work_id = r.work_id \
                 JOIN public.imprint i ON i.imprint_id = w.imprint_id \
                 WHERE d.work_day_sequence > $1 \
                   AND i.publisher_id = ANY($2) \
                   AND r.period_start >= $3 \
                   AND r.period_start < $4 \
                   AND r.platform_id = ANY($5) \
                   AND r.measure_id = ANY($6)";

/// Step 3: which explicitly selected platforms exist.
pub(crate) const KNOWN_PLATFORMS_SQL: &str =
    "SELECT platform_id AS id FROM public.metric_platform WHERE platform_id = ANY($1)";

/// Step 3: which explicitly selected measures exist.
pub(crate) const KNOWN_MEASURES_SQL: &str =
    "SELECT measure_id AS id FROM public.metric_measure WHERE measure_id = ANY($1)";

/// Step 3: the platform/measure pairs represented for the selected
/// publishers and range, in the projection or in terminal coverage from an
/// eligible managed source account whose import that account and its expected
/// publisher own.
pub(crate) const REPRESENTED_SQL: &str =
    "SELECT DISTINCT represented.platform_id, represented.measure_id \
                 FROM ( \
                     SELECT r.platform_id, r.measure_id \
                     FROM public.metric_rollup_work_day r \
                     JOIN public.work w ON w.work_id = r.work_id \
                     JOIN public.imprint i ON i.imprint_id = w.imprint_id \
                     WHERE i.publisher_id = ANY($1) \
                       AND r.day >= $2 \
                       AND r.day < $3 \
                       AND (cardinality($4::uuid[]) = 0 OR r.platform_id = ANY($4)) \
                       AND (cardinality($5::uuid[]) = 0 OR r.measure_id = ANY($5)) \
                     UNION ALL \
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
                     WHERE sa.expected_publisher_id = ANY($1) \
                       AND sa.enabled \
                       AND s.enabled \
                       AND s.acquisition_type::text = $6 \
                       AND mi.status::text IN ('COMPLETED', 'COMPLETED_WITH_ERRORS') \
                       AND c.period_start < $3 \
                       AND c.period_end > $2 \
                       AND (cardinality($4::uuid[]) = 0 OR c.platform_id = ANY($4)) \
                       AND (cardinality($5::uuid[]) = 0 OR c.measure_id = ANY($5)) \
                     UNION ALL \
                     SELECT q.platform_id, q.measure_id \
                     FROM public.metric_identifier_quarantine q \
                     JOIN public.metric_record_provenance p \
                       ON p.record_provenance_id = q.record_provenance_id \
                     JOIN public.metric_import mi ON mi.import_id = p.import_id \
                     LEFT JOIN public.metric_identifier_quarantine_reconciliation r \
                       ON r.identifier_quarantine_id = q.identifier_quarantine_id \
                     WHERE mi.publisher_id = ANY($1) \
                       AND q.period_start < $3 \
                       AND q.period_end > $2 \
                       AND (cardinality($4::uuid[]) = 0 OR q.platform_id = ANY($4)) \
                       AND (cardinality($5::uuid[]) = 0 OR q.measure_id = ANY($5)) \
                       AND r.resolved_at IS NULL \
                 ) represented";

/// Step 3: the additivity of every served measure.
pub(crate) const MEASURE_FLAGS_SQL: &str = "SELECT additive_across_time, additive_across_works \
             FROM public.metric_measure \
             WHERE measure_id = ANY($1)";

// ---------------------------------------------------------------------------
// The read
// ---------------------------------------------------------------------------

/// Answer one dashboard request.
///
/// The caller has already passed `METRICS_READ_SERVICE`. Checks that need no
/// database run first; everything else runs in one `READ ONLY, REPEATABLE
/// READ` transaction on one connection, in this order:
///
/// 1. resolve the selected publishers and require `METRICS_DASHBOARD` for
///    each through the ADR-0001 package model;
/// 2. read the `MET-WP4-01` frontier `W`, `next_sequence` and `watermark_at`,
///    together with the transaction timestamp;
/// 3. resolve and bound the platform/measure scope from canonical projection,
///    terminal coverage or unresolved identifier evidence, and require every
///    served measure to be additive across time and works;
/// 4. resolve the eligible managed source account per publisher/platform;
/// 5. sum the work-day projection per platform, measure and day through
///    current work ownership, resolving by base cell only the groups whose
///    dimensional representation the group sums alone cannot settle;
/// 6. select the current terminal coverage assertion per platform, measure
///    and day;
/// 7. find unapplied work-day deltas above `W` that intersect the request;
/// 8. mark unresolved identifier evidence under immutable import-publisher
///    scope without consulting current DOI resolution or source enablement.
///
/// Every statement is set-based over the whole request, so the number of
/// statements does not depend on how many works, days or rows are involved:
/// the base-cell resolution, lag and identifier-quality statements each run at
/// most once.
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

fn evaluate(
    connection: &mut PgConnection,
    request: &ValidatedRequest,
) -> Result<MetricDashboard, MetricReadError> {
    // 1. Publishers and entitlement.
    let publishers: Vec<PublisherRow> = diesel::sql_query(PUBLISHERS_SQL)
        .bind::<Array<SqlUuid>, _>(&request.publisher_ids)
        .load(connection)?;
    let found: BTreeSet<Uuid> = publishers.iter().map(|row| row.publisher_id).collect();
    if request
        .publisher_ids
        .iter()
        .any(|publisher_id| !found.contains(publisher_id))
    {
        return Err(MetricReadError::QueryInvalid(UNKNOWN_PUBLISHER));
    }
    // The capability comes from the publisher's package alone. Holding
    // METRICS_READ_SERVICE was checked before this function and contributes
    // nothing here.
    if publishers.iter().any(|row| {
        !row.subscription_package
            .has_capability(PublisherCapability::MetricsDashboard)
    }) {
        return Err(MetricReadError::Unauthorised);
    }

    // 2. The rollup frontier, read in the same snapshot as everything else.
    let frontier: FrontierRow = diesel::sql_query(FRONTIER_SQL)
        .load::<FrontierRow>(connection)?
        .into_iter()
        .next()
        .ok_or(MetricReadError::Unavailable)?;

    // 3. Scope.
    let (platforms, measures) = resolve_scope(connection, request)?;
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
    let mut grid = Grid::new(&combinations, days.len());
    if !combinations.is_empty() {
        // 4. Eligible managed source accounts.
        let accounts: Vec<AccountRow> = diesel::sql_query(ACCOUNTS_SQL)
            .bind::<Array<SqlUuid>, _>(&request.publisher_ids)
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
        let account_ids: Vec<Uuid> = accounts.iter().map(|row| row.source_account_id).collect();

        // 5. Projected values.
        let sums: Vec<DaySumRow> = diesel::sql_query(DAY_SUMS_SQL)
            .bind::<Array<SqlUuid>, _>(&request.publisher_ids)
            .bind::<Date, _>(request.start)
            .bind::<Date, _>(request.end)
            .bind::<Array<SqlUuid>, _>(&platform_ids)
            .bind::<Array<SqlUuid>, _>(&measure_ids)
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
                .bind::<Array<SqlUuid>, _>(&request.publisher_ids)
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
        for (platform_id, measure_id, day, total, country, institution) in resolved {
            let total = BigInt::parse_canonical(&total)
                .ok_or(MetricReadError::QueryLimitExceeded(AGGREGATE_OUT_OF_RANGE))?;
            grid.set_sum(platform_id, measure_id, request.start, day, total);
            grid.set_dependence(platform_id, measure_id, country, institution);
        }

        // 6. Current terminal coverage.
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

        // 7. Outstanding rollup work, read only when W has not caught up.
        if frontier.applied_through_sequence < frontier.next_sequence - 1 {
            let lagging: Vec<LagRow> = diesel::sql_query(LAG_SQL)
                .bind::<SqlBigInt, _>(frontier.applied_through_sequence)
                .bind::<Array<SqlUuid>, _>(&request.publisher_ids)
                .bind::<Date, _>(request.start)
                .bind::<Date, _>(request.end)
                .bind::<Array<SqlUuid>, _>(&platform_ids)
                .bind::<Array<SqlUuid>, _>(&measure_ids)
                .load(connection)?;
            for row in lagging {
                grid.set_lag(row.platform_id, row.measure_id, request.start, row.day);
            }
        }

        // 8. Identifier quality. Historical admitted evidence remains
        // load-bearing even if its source is now disabled.
        let unresolved: Vec<IdentifierQualityRow> = diesel::sql_query(IDENTIFIER_QUALITY_SQL)
            .bind::<Array<SqlUuid>, _>(&request.publisher_ids)
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

    grid.apply_dimensional_coverage();
    assemble(request, &combinations, &grid, &days, frontier)
}

/// Resolve the served platforms and measures.
///
/// Explicit IDs must all exist. An omitted or empty dimension resolves to
/// every identity represented for the selected publishers and range in the
/// work-day projection, terminal coverage from an eligible managed source
/// account, or unresolved quarantine under immutable import-publisher scope,
/// restricted by the other dimension when that one is explicit. Every served
/// measure, however selected, must be additive across both time and works.
fn resolve_scope(
    connection: &mut PgConnection,
    request: &ValidatedRequest,
) -> Result<(BTreeSet<Uuid>, BTreeSet<Uuid>), MetricReadError> {
    if let Some(platforms) = &request.platforms {
        let ids: Vec<Uuid> = platforms.iter().copied().collect();
        let known: Vec<IdRow> = diesel::sql_query(KNOWN_PLATFORMS_SQL)
            .bind::<Array<SqlUuid>, _>(&ids)
            .load(connection)?;
        let known: BTreeSet<Uuid> = known.into_iter().map(|row| row.id).collect();
        if known != *platforms {
            return Err(MetricReadError::QueryInvalid(UNKNOWN_PLATFORM));
        }
    }
    if let Some(measures) = &request.measures {
        let ids: Vec<Uuid> = measures.iter().copied().collect();
        let known: Vec<IdRow> = diesel::sql_query(KNOWN_MEASURES_SQL)
            .bind::<Array<SqlUuid>, _>(&ids)
            .load(connection)?;
        let known: BTreeSet<Uuid> = known.into_iter().map(|row| row.id).collect();
        if known != *measures {
            return Err(MetricReadError::QueryInvalid(UNKNOWN_MEASURE));
        }
    }

    let (platforms, measures) = match (&request.platforms, &request.measures) {
        (Some(platforms), Some(measures)) => (platforms.clone(), measures.clone()),
        (explicit_platforms, explicit_measures) => {
            let platform_filter: Vec<Uuid> = explicit_platforms.iter().flatten().copied().collect();
            let measure_filter: Vec<Uuid> = explicit_measures.iter().flatten().copied().collect();
            let represented: Vec<PairRow> = diesel::sql_query(REPRESENTED_SQL)
                .bind::<Array<SqlUuid>, _>(&request.publisher_ids)
                .bind::<Date, _>(request.start)
                .bind::<Date, _>(request.end)
                .bind::<Array<SqlUuid>, _>(&platform_filter)
                .bind::<Array<SqlUuid>, _>(&measure_filter)
                .bind::<Text, _>(DRIVER_ACQUISITION)
                .load(connection)?;
            let platforms = explicit_platforms
                .clone()
                .unwrap_or_else(|| represented.iter().map(|row| row.platform_id).collect());
            let measures = explicit_measures
                .clone()
                .unwrap_or_else(|| represented.iter().map(|row| row.measure_id).collect());
            (platforms, measures)
        }
    };

    if !measures.is_empty() {
        let ids: Vec<Uuid> = measures.iter().copied().collect();
        let flags: Vec<MeasureFlagsRow> = diesel::sql_query(MEASURE_FLAGS_SQL)
            .bind::<Array<SqlUuid>, _>(&ids)
            .load(connection)?;
        if flags.len() != ids.len() {
            return Err(MetricReadError::Unavailable);
        }
        if flags
            .iter()
            .any(|row| !(row.additive_across_time && row.additive_across_works))
        {
            return Err(MetricReadError::QueryInvalid(NON_ADDITIVE_MEASURE));
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

// ---------------------------------------------------------------------------
// Assembly
// ---------------------------------------------------------------------------

/// The effective current coverage of one day.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DayCoverage {
    status: MetricCoverageStatus,
    country: bool,
    institution: bool,
}

/// Per-day facts for one platform/measure combination.
#[derive(Debug, Clone, Default)]
struct Cells {
    sums: Vec<Option<BigInt>>,
    /// `None` means no current assertion, which is `UNKNOWN`.
    coverage: Vec<Option<DayCoverage>>,
    lag: Vec<bool>,
    /// True for days overlapped by identifier-unresolved quarantine evidence.
    identifier_incomplete: Vec<bool>,
    /// Whether any served value of this combination was taken from rows
    /// broken down by country, so complete coverage also needs the country
    /// dimension.
    depends_on_country: bool,
    /// The same for the institution dimension.
    depends_on_institution: bool,
}

impl Cells {
    fn is_complete(&self, index: usize) -> bool {
        matches!(
            self.coverage[index],
            Some(DayCoverage {
                status: MetricCoverageStatus::Complete,
                ..
            })
        )
    }

    /// The value of the half-open day-index range `from..to`.
    ///
    /// Projected rows win: their exact sum is the value. With no rows, zero
    /// is returned only when every day is `COMPLETE`, untouched by
    /// outstanding rollup work and free of unresolved identifier evidence;
    /// otherwise the value is unknown.
    fn value(&self, from: usize, to: usize) -> Result<Option<BigInt>, MetricReadError> {
        let mut total: Option<BigInt> = None;
        for sum in self.sums[from..to].iter().flatten() {
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
        let justified = (from..to).all(|index| {
            self.is_complete(index) && !self.lag[index] && !self.identifier_incomplete[index]
        });
        Ok(justified.then_some(BigInt::ZERO))
    }
}

/// Every combination's per-day facts.
struct Grid {
    cells: BTreeMap<(Uuid, Uuid), Cells>,
}

impl Grid {
    fn new(combinations: &[(Uuid, Uuid)], days: usize) -> Self {
        let cells = combinations
            .iter()
            .map(|combination| {
                (
                    *combination,
                    Cells {
                        sums: vec![None; days],
                        coverage: vec![None; days],
                        lag: vec![false; days],
                        identifier_incomplete: vec![false; days],
                        depends_on_country: false,
                        depends_on_institution: false,
                    },
                )
            })
            .collect();
        Grid { cells }
    }

    fn cell(
        &mut self,
        platform_id: Uuid,
        measure_id: Uuid,
        start: NaiveDate,
        day: NaiveDate,
    ) -> Option<(&mut Cells, usize)> {
        let index = usize::try_from((day - start).num_days()).ok()?;
        let cells = self.cells.get_mut(&(platform_id, measure_id))?;
        (index < cells.sums.len()).then_some((cells, index))
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

    /// Downgrade coverage the served representation cannot rely on
    /// (Specification Amendment 6 section 3).
    ///
    /// When a combination's values come from country rows, a day asserted
    /// `COMPLETE` without country coverage is only `PARTIAL` for it, and the
    /// same holds for institution rows. The downgrade applies to every day of
    /// the combination, including days without a value, so an empty day that
    /// the representation cannot vouch for is `null` rather than `"0"`.
    /// Publication rows have no coverage flag, so they rely on the ordinary
    /// status. Every later decision — values, status, `dataThrough` and
    /// warnings — reads the downgraded coverage.
    fn apply_dimensional_coverage(&mut self) {
        for cells in self.cells.values_mut() {
            let (country, institution) = (cells.depends_on_country, cells.depends_on_institution);
            for day in cells.coverage.iter_mut().flatten() {
                if day.status == MetricCoverageStatus::Complete
                    && ((country && !day.country) || (institution && !day.institution))
                {
                    day.status = MetricCoverageStatus::Partial;
                }
            }
        }
    }

    fn set_assertion(
        &mut self,
        platform_id: Uuid,
        measure_id: Uuid,
        start: NaiveDate,
        day: NaiveDate,
        coverage: DayCoverage,
    ) {
        if let Some((cells, index)) = self.cell(platform_id, measure_id, start, day) {
            cells.coverage[index] = Some(coverage);
        }
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
}

fn assemble(
    request: &ValidatedRequest,
    combinations: &[(Uuid, Uuid)],
    grid: &Grid,
    days: &[NaiveDate],
    frontier: FrontierRow,
) -> Result<MetricDashboard, MetricReadError> {
    let day_count = days.len();
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
            value: cells.value(0, day_count)?,
        });
        for (bucket_start, bucket_end) in &request.buckets {
            let from = (*bucket_start - request.start).num_days() as usize;
            let to = (*bucket_end - request.start).num_days() as usize;
            timeline.push(MetricTimeBucket {
                platform_id: *platform_id,
                measure_id: *measure_id,
                start_date: *bucket_start,
                end_date: *bucket_end,
                value: cells.value(from, to)?,
            });
        }

        let day_unknown = cells
            .coverage
            .iter()
            .any(|day| !matches!(day, Some(coverage) if coverage.status != MetricCoverageStatus::Unknown));
        let day_partial = cells.coverage.iter().any(
            |day| matches!(day, Some(coverage) if coverage.status == MetricCoverageStatus::Partial),
        );
        any_unknown |= day_unknown;
        any_partial |= day_partial;
        any_identifier_incomplete |= cells.identifier_incomplete.iter().any(|pending| *pending);
        any_lag |= cells.lag.iter().any(|lag| *lag);

        let status = if day_unknown {
            MetricCoverageStatus::Unknown
        } else if day_partial {
            MetricCoverageStatus::Partial
        } else {
            MetricCoverageStatus::Complete
        };
        let data_through =
            match (0..day_count).find(|index| !cells.is_complete(*index) || cells.lag[*index]) {
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

    Ok(MetricDashboard {
        totals,
        timeline,
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
