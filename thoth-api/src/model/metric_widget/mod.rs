//! The public, read-only `metricWidget` read for the embeddable metrics
//! widget (`MET-WP4-WIDGET-01`, under `ADR-0014`).
//!
//! An anonymous caller names one work by DOI (a book, or a chapter) and a
//! whole-month range; for a book it may also narrow the child works. The read
//! serves, per section (the work itself, and its child works), only the
//! platform/measure pairs the registry makes public and the projections hold
//! for that section, with exactly the per-pair values and coverage the
//! `metricDashboard` engine gives a single-pair read, for publishers whose
//! package grants `METRICS_WIDGET`. No role is required and the caller's
//! identity is never consulted.
//!
//! # Order of operations
//!
//! 1. GraphQL coercion (before the resolver).
//! 2. The server switch: `OFF` refuses with no slot, connection or statement.
//! 3. One resolution per HTTP request, counted on the request context.
//! 4. Input-only validation, with no database access.
//! 5. A process-wide widget slot, taken without waiting; the request deadline
//!    starts here.
//! 6. One pool connection with a short checkout timeout, and one
//!    `READ ONLY, REPEATABLE READ` transaction.
//! 7. The prelude: the engine's planner setting, this module's statement
//!    timeout and the engine's projection lock, before any query.
//! 8. One resolution statement: the work, its publisher's package and its
//!    child works.
//! 9. Unknown work or unentitled publisher: the uniform refusal.
//! 10. Child validation.
//! 11. Served-pair discovery, one statement per non-empty section, then the
//!     pair bound.
//! 12. One labels statement: public labels, the transaction timestamp and the
//!     rollup frontier.
//! 13. The engine in pair mode for each section that serves a pair.
//! 14. Commit; the connection and the slot are released on every exit path,
//!     panic unwinding included.
//!
//! The deadline is checked before every statement from step 8 on, the
//! engine's included: one engine [`ReadDeadline`], created at step 5 with
//! this module's own instant, serves both engine runs, and this module admits
//! its own statements against the same instant. Nothing in this module
//! returns SQL, configuration, a package or capability value or an engine
//! message to the caller.

use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use chrono::{Datelike, NaiveDate, Utc};
use diesel::pg::PgConnection;
use diesel::sql_types::{
    Array, BigInt as SqlBigInt, Date, Nullable, Text, Timestamptz, Uuid as SqlUuid,
};
use diesel::RunQueryDsl;
use juniper::{graphql_value, FieldError, IntoFieldError};
use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;
use uuid::Uuid;

use crate::db::PgPool;
use crate::model::metric_coverage::MetricCoverageStatus;
use crate::model::metric_dashboard::{
    metric_pair_read, MetricCountryTotal, MetricDashboard, MetricPairRead, MetricReadError,
    MetricTimeBucket, MetricTimelineGrain, MetricTotal, ReadDeadline, AGGREGATE_OUT_OF_RANGE,
    CUSTOM_PLANS_SQL, PROJECTION_LOCK_SQL,
};
use crate::model::metric_measure::MetricMeasureUnit;
use crate::model::publisher::{PublisherCapability, ThothPackage};
use crate::model::{Doi, Timestamp};

// ---------------------------------------------------------------------------
// Profile
// ---------------------------------------------------------------------------

/// The most whole months one widget read may cover.
pub const METRIC_WIDGET_MAX_RANGE_MONTHS: u32 = 12;
/// The most entries `chapterDois` may hold.
pub const METRIC_WIDGET_MAX_CHAPTER_DOIS: usize = 500;
/// The most child works a work may have and still be served.
pub const METRIC_WIDGET_MAX_CHILD_WORKS: usize = 500;
/// The most platform/measure pairs one section may serve.
pub const METRIC_WIDGET_MAX_PAIRS: usize = 40;
/// The most widget reads one process runs at once.
pub const METRIC_WIDGET_SLOTS: usize = 2;
/// How long a read waits for a pool connection.
pub const METRIC_WIDGET_CHECKOUT_TIMEOUT: Duration = Duration::from_secs(1);
/// How long a read may run, from taking its slot, before no further
/// statement starts.
pub const METRIC_WIDGET_DEADLINE: Duration = Duration::from_secs(10);
/// The per-statement timeout of the widget's transaction, run as the second
/// prelude statement in place of the dashboard's.
pub const METRIC_WIDGET_STATEMENT_TIMEOUT_SQL: &str = "SET LOCAL statement_timeout = '5s'";
/// The most days after the current UTC date `endDate` may be (the technical
/// date domain's ceiling).
pub const METRIC_WIDGET_END_DATE_CEILING_DAYS: i64 = 366;

/// The earliest `startDate` (the technical date domain's floor).
fn date_floor() -> NaiveDate {
    NaiveDate::from_ymd_opt(2000, 1, 1).unwrap_or(NaiveDate::MIN)
}

// ---------------------------------------------------------------------------
// Server switch
// ---------------------------------------------------------------------------

/// Whether the public widget read is served (`THOTH_METRIC_WIDGET`).
///
/// Read once at process start; there is no reload path. `Off`, the default,
/// refuses every read with the uniform unavailable error before any slot,
/// connection or statement. Turning it on in a deployed environment is a
/// separately authorized activation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MetricWidgetSwitch {
    #[default]
    Off,
    On,
}

impl FromStr for MetricWidgetSwitch {
    type Err = String;

    /// Accepts exactly `OFF` and `ON` and fails on anything else, so an
    /// unparseable value stops the process rather than resolving to either.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "OFF" => Ok(MetricWidgetSwitch::Off),
            "ON" => Ok(MetricWidgetSwitch::On),
            other => Err(format!(
                "Invalid metric widget setting: {other}. Expected OFF or ON."
            )),
        }
    }
}

impl std::fmt::Display for MetricWidgetSwitch {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            MetricWidgetSwitch::Off => "OFF",
            MetricWidgetSwitch::On => "ON",
        })
    }
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

/// One widget request.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLInputObject),
    graphql(
        description = "A public widget request: one work by DOI, optionally some of its child works, and a range of whole calendar months"
    )
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricWidgetInput {
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "DOI of the embedded work, a book or a chapter. A bare 10.x/y DOI or a doi.org URL is accepted, case-insensitively"
        )
    )]
    pub doi: Doi,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "DOIs of at most 500 child works of the embedded work to serve, each at most once. Omitted or empty means every child work. Must be omitted or empty for a chapter"
        )
    )]
    pub chapter_dois: Option<Vec<Doi>>,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "First day of the first month of the range, inclusive. It must be the first day of a month, on or after 2000-01-01"
        )
    )]
    pub start_date: NaiveDate,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "First day of the month after the range, exclusive. It must be the first day of a month, later than startDate, at most 12 months after it and at most 366 days after today (UTC)"
        )
    )]
    pub end_date: NaiveDate,
}

/// One widget response.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLObject),
    graphql(
        description = "Public monthly metrics for one work and its child works, read from one consistent database snapshot"
    )
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricWidget {
    #[cfg_attr(feature = "backend", graphql(description = "The embedded work only"))]
    pub work: MetricWidgetSection,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "The selected child works of the embedded work's publisher, together. Null for a chapter, or when no selected child work is served"
        )
    )]
    pub chapters: Option<MetricWidgetSection>,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "How many selected child works are not served because another publisher owns them"
        )
    )]
    pub excluded_child_count: i32,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Every platform served in either section, ordered by platform ID")
    )]
    pub platforms: Vec<MetricWidgetPlatform>,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Every measure served in either section, ordered by measure ID")
    )]
    pub measures: Vec<MetricWidgetMeasure>,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "First day of the range, inclusive")
    )]
    pub start_date: NaiveDate,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "Day after the range, exclusive")
    )]
    pub end_date: NaiveDate,
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
            description = "When the applied work-day frontier the response was read against was established"
        )
    )]
    pub rollup_watermark: Timestamp,
}

/// The values of one section.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLObject),
    graphql(
        description = "Monthly values and coverage of one widget section, for each platform and measure it serves"
    )
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricWidgetSection {
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "One total per served platform and measure, ordered by platform ID and measure ID"
        )
    )]
    pub totals: Vec<MetricTotal>,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "One bucket per calendar month of the range for each served platform and measure, ordered by platform ID, measure ID and month"
        )
    )]
    pub timeline: Vec<MetricTimeBucket>,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "Known per-country totals over the range for the served platforms and measures, ordered by platform ID, measure ID and country code. A country without a row is not a zero"
        )
    )]
    pub countries: Vec<MetricCountryTotal>,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "Coverage of each served platform and measure, ordered by platform ID and measure ID"
        )
    )]
    pub coverage: Vec<MetricWidgetPairCoverage>,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "The earliest per-pair dataThrough date, or null unless every served pair has one"
        )
    )]
    pub data_through: Option<NaiveDate>,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "True when coverage is not complete for some served platform, measure and day, when some values may still change or be incomplete, or when nothing is served"
        )
    )]
    pub is_partial: bool,
}

/// Coverage of one served pair in one section.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLObject),
    graphql(description = "Coverage of one platform and measure in one widget section")
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricWidgetPairCoverage {
    pub platform_id: Uuid,
    pub measure_id: Uuid,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "COMPLETE only when every day of the range is completely covered, including by country; UNKNOWN when any day has no coverage; otherwise PARTIAL"
        )
    )]
    pub status: MetricCoverageStatus,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "The last day of the unbroken run from startDate in which every day is completely covered and its values settled, or null when the first day is not"
        )
    )]
    pub data_through: Option<NaiveDate>,
    #[cfg_attr(
        feature = "backend",
        graphql(
            description = "True only when every day of the range has coverage that includes the country dimension"
        )
    )]
    pub country_coverage: bool,
}

/// A served platform's public labels.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLObject),
    graphql(description = "A platform served by a widget response")
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricWidgetPlatform {
    pub platform_id: Uuid,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "The platform's stable code")
    )]
    pub code: String,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "The platform's display name")
    )]
    pub display_name: String,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "The platform's public description, if it has one")
    )]
    pub public_description: Option<String>,
}

/// A served measure's public labels.
#[cfg_attr(
    feature = "backend",
    derive(juniper::GraphQLObject),
    graphql(description = "A measure served by a widget response")
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricWidgetMeasure {
    pub measure_id: Uuid,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "The measure's stable code")
    )]
    pub code: String,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "The measure's display name")
    )]
    pub display_name: String,
    #[cfg_attr(
        feature = "backend",
        graphql(description = "The unit the measure's values are expressed in")
    )]
    pub unit: MetricMeasureUnit,
    #[cfg_attr(feature = "backend", graphql(description = "What the measure counts"))]
    pub definition: String,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

const UNAVAILABLE_MESSAGE: &str = "Metrics are not available for this work.";
const BUSY_MESSAGE: &str = "Metrics are busy. Try again shortly.";
const ONE_SELECTION_MESSAGE: &str = "Only one metricWidget field may be requested at a time.";
const INVALID_DOI_MESSAGE: &str = "doi and chapterDois must be valid DOIs.";
const DATES_MESSAGE: &str =
    "startDate and endDate must be first days of months, with startDate before endDate.";
const DATE_DOMAIN_MESSAGE: &str =
    "startDate must be on or after 2000-01-01, and endDate at most 366 days after today (UTC).";
const RANGE_MESSAGE: &str = "The date range may cover at most 12 months.";
const TOO_MANY_CHAPTER_DOIS_MESSAGE: &str = "chapterDois may hold at most 500 DOIs.";
const DUPLICATE_CHAPTER_DOI_MESSAGE: &str = "chapterDois must not repeat a DOI.";
const NOT_A_CHILD_MESSAGE: &str = "chapterDois must list child works of doi.";
const TOO_MANY_CHILDREN_MESSAGE: &str = "This work has more child works than the widget supports.";
const TOO_MANY_PAIRS_MESSAGE: &str =
    "This work has more platform and measure combinations than the widget supports.";
const AGGREGATE_MESSAGE: &str = "Metrics for this work exceed the supported range.";
const INTERNAL_MESSAGE: &str = "Metrics could not be read.";

/// A widget refusal or failure. Every message is fixed widget text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricWidgetError {
    /// `METRIC_WIDGET_UNAVAILABLE`.
    Unavailable,
    /// `METRIC_WIDGET_BUSY`.
    Busy,
    /// `METRIC_QUERY_INVALID`, with its fixed widget message.
    QueryInvalid(&'static str),
    /// `METRIC_QUERY_LIMIT_EXCEEDED`, with its fixed widget message.
    QueryLimitExceeded(&'static str),
    /// `INTERNAL_ERROR`.
    Internal,
}

impl MetricWidgetError {
    /// The stable `extensions.type` classification.
    pub fn code(&self) -> &'static str {
        match self {
            MetricWidgetError::Unavailable => "METRIC_WIDGET_UNAVAILABLE",
            MetricWidgetError::Busy => "METRIC_WIDGET_BUSY",
            MetricWidgetError::QueryInvalid(_) => "METRIC_QUERY_INVALID",
            MetricWidgetError::QueryLimitExceeded(_) => "METRIC_QUERY_LIMIT_EXCEEDED",
            MetricWidgetError::Internal => "INTERNAL_ERROR",
        }
    }

    /// The fixed client-facing message.
    pub fn message(&self) -> &'static str {
        match self {
            MetricWidgetError::Unavailable => UNAVAILABLE_MESSAGE,
            MetricWidgetError::Busy => BUSY_MESSAGE,
            MetricWidgetError::QueryInvalid(message)
            | MetricWidgetError::QueryLimitExceeded(message) => message,
            MetricWidgetError::Internal => INTERNAL_MESSAGE,
        }
    }
}

impl IntoFieldError for MetricWidgetError {
    fn into_field_error(self) -> FieldError {
        let code = self.code();
        FieldError::new(self.message(), graphql_value!({ "type": code }))
    }
}

/// Why a read was refused or failed, for operators only. Never returned to
/// the caller, and no variant can carry a package or capability value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WidgetReason {
    /// The server switch is `OFF`.
    SwitchOff,
    /// No work has the DOI.
    UnknownWork,
    /// The work's publisher's package does not grant `METRICS_WIDGET`.
    NotEntitled,
    /// The engine re-checked the capability and refused it.
    EngineUnauthorised,
    /// More than one eligible managed source account on a served platform.
    SourceScopeAmbiguous,
    /// A total or country dimension ambiguity in a section.
    DimensionScopeAmbiguous,
    /// An active native-grain record on a served platform and measure.
    UnsupportedSourceGrain,
    /// Both widget slots of this process are taken.
    NoSlot,
    /// No pool connection within the checkout timeout, or a pool failure.
    CheckoutFailed,
    /// The request deadline passed before a statement.
    DeadlinePassed,
    /// The prelude's projection lock failed, for example under a rebuild.
    LockFailed,
    /// A database failure outside the engine.
    Database,
    /// The rollup frontier row is missing.
    FrontierMissing,
    /// An engine run read a different frontier than the labels statement.
    FrontierMismatch,
    /// A served pair's platform or measure has no label row.
    LabelMissing,
    /// A widget statement returned a row this module cannot interpret.
    UnexpectedRow,
    /// The engine failed without a deadline refusal.
    EngineUnavailable,
    /// The engine rejected the widget's own request (a widget defect).
    EngineRejected,
}

/// A failure inside a read, with the reason operators see.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Failure {
    error: MetricWidgetError,
    reason: Option<WidgetReason>,
}

impl Failure {
    fn new(error: MetricWidgetError, reason: WidgetReason) -> Self {
        Failure {
            error,
            reason: Some(reason),
        }
    }

    fn input(error: MetricWidgetError) -> Self {
        Failure {
            error,
            reason: None,
        }
    }
}

impl From<diesel::result::Error> for Failure {
    fn from(_: diesel::result::Error) -> Self {
        Failure::new(MetricWidgetError::Internal, WidgetReason::Database)
    }
}

/// Map an engine error to a widget failure.
///
/// The engine's deadline flag ([`ReadDeadline::expired`]) decides
/// `METRIC_WIDGET_BUSY` alone, whatever the variant. Otherwise the
/// publisher-state refusals are the uniform unavailable error, an aggregate
/// outside the representable range is a limit error, and every other variant,
/// including any added later (the dashboard guard's `Busy`, which a widget
/// read never takes, among them), is `INTERNAL_ERROR`. No engine message
/// reaches the caller.
fn engine_failure(error: &MetricReadError, deadline_expired: bool) -> Failure {
    if deadline_expired {
        return Failure::new(MetricWidgetError::Busy, WidgetReason::DeadlinePassed);
    }
    match error {
        MetricReadError::Unauthorised => Failure::new(
            MetricWidgetError::Unavailable,
            WidgetReason::EngineUnauthorised,
        ),
        MetricReadError::SourceScopeAmbiguous => Failure::new(
            MetricWidgetError::Unavailable,
            WidgetReason::SourceScopeAmbiguous,
        ),
        MetricReadError::DimensionScopeAmbiguous => Failure::new(
            MetricWidgetError::Unavailable,
            WidgetReason::DimensionScopeAmbiguous,
        ),
        MetricReadError::UnsupportedSourceGrain => Failure::new(
            MetricWidgetError::Unavailable,
            WidgetReason::UnsupportedSourceGrain,
        ),
        MetricReadError::QueryLimitExceeded(..) if error.message() == AGGREGATE_OUT_OF_RANGE => {
            Failure::input(MetricWidgetError::QueryLimitExceeded(AGGREGATE_MESSAGE))
        }
        // Every other variant, today's and any later one, is INTERNAL_ERROR.
        _ => {
            let reason = if matches!(error, MetricReadError::Unavailable) {
                WidgetReason::EngineUnavailable
            } else {
                WidgetReason::EngineRejected
            };
            Failure::new(MetricWidgetError::Internal, reason)
        }
    }
}

// ---------------------------------------------------------------------------
// Steps and logging
// ---------------------------------------------------------------------------

/// The step of section 4.1 an outcome arose at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum WidgetStep {
    Switch = 2,
    Claim = 3,
    Input = 4,
    Slot = 5,
    Checkout = 6,
    Prelude = 7,
    Resolution = 8,
    Entitlement = 9,
    Children = 10,
    Pairs = 11,
    Labels = 12,
    Engine = 13,
    Commit = 14,
}

impl WidgetStep {
    fn number(self) -> u8 {
        self as u8
    }
}

/// The two sections of a response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WidgetSection {
    Work,
    Chapters,
}

impl WidgetSection {
    fn name(self) -> &'static str {
        match self {
            WidgetSection::Work => "work",
            WidgetSection::Chapters => "chapters",
        }
    }
}

/// The served pair count and the platform x measure cross-product size of
/// one section, once known.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct SectionSize {
    pub(crate) pairs: usize,
    pub(crate) cross_product: usize,
}

impl SectionSize {
    fn of(pairs: &[(Uuid, Uuid)]) -> Self {
        let platforms: BTreeSet<Uuid> = pairs.iter().map(|(platform, _)| *platform).collect();
        let measures: BTreeSet<Uuid> = pairs.iter().map(|(_, measure)| *measure).collect();
        SectionSize {
            pairs: pairs.len(),
            cross_product: platforms.len() * measures.len(),
        }
    }
}

/// Everything one resolution learned that its log lines may carry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WidgetFacts {
    pub(crate) switch: MetricWidgetSwitch,
    /// The claim counter's value before this resolution's claim; `None` on
    /// the switch-off path, which claims nothing.
    pub(crate) previous_claims: Option<usize>,
    /// `None` for a served response.
    pub(crate) error: Option<MetricWidgetError>,
    pub(crate) reason: Option<WidgetReason>,
    pub(crate) step: WidgetStep,
    pub(crate) elapsed_ms: u128,
    pub(crate) doi: Option<Doi>,
    pub(crate) work: Option<SectionSize>,
    pub(crate) chapters: Option<SectionSize>,
    /// The section whose engine run failed, if one did.
    pub(crate) engine_section: Option<WidgetSection>,
    /// The last statement admitted against the deadline: the engine's last,
    /// or this module's own when no engine statement was admitted.
    pub(crate) last_statement: Option<&'static str>,
}

/// What a log line reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WidgetLineKind {
    /// The outcome of one resolution, or of a refused second selection.
    Outcome,
    /// Why the read was unavailable or busy.
    Reason,
    /// The pool could not supply a connection.
    Pool,
    /// An internal failure, with its reason.
    Internal,
}

/// One log line, as a value.
///
/// Like the mutation guard's event, it has no field able to hold a package
/// or capability value, a token or SQL: every field is a fixed enumeration,
/// a count, a duration, a parsed DOI (public data) or the name of a statement
/// constant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WidgetLogLine {
    pub(crate) level: log::Level,
    pub(crate) kind: WidgetLineKind,
    pub(crate) error: Option<MetricWidgetError>,
    pub(crate) step: WidgetStep,
    pub(crate) elapsed_ms: u128,
    pub(crate) doi: Option<Doi>,
    pub(crate) work: Option<SectionSize>,
    pub(crate) chapters: Option<SectionSize>,
    pub(crate) reason: Option<WidgetReason>,
    pub(crate) engine_section: Option<WidgetSection>,
    pub(crate) last_statement: Option<&'static str>,
}

impl WidgetLogLine {
    fn base(facts: &WidgetFacts, level: log::Level, kind: WidgetLineKind) -> Self {
        WidgetLogLine {
            level,
            kind,
            error: facts.error,
            step: facts.step,
            elapsed_ms: facts.elapsed_ms,
            doi: None,
            work: None,
            chapters: None,
            reason: None,
            engine_section: None,
            last_statement: None,
        }
    }

    /// The rendered line.
    pub(crate) fn message(&self) -> String {
        let outcome = self.error.map_or("OK", |error| error.code());
        let size = |size: Option<SectionSize>| {
            size.map_or("-".to_string(), |size| {
                format!("{}/{}", size.pairs, size.cross_product)
            })
        };
        let mut line = format!(
            "metricWidget {} outcome={outcome} step={} elapsed_ms={}",
            match self.kind {
                WidgetLineKind::Outcome => "outcome",
                WidgetLineKind::Reason => "reason",
                WidgetLineKind::Pool => "pool",
                WidgetLineKind::Internal => "internal",
            },
            self.step.number(),
            self.elapsed_ms,
        );
        if self.kind == WidgetLineKind::Outcome {
            line.push_str(&format!(
                " work_pairs/cross={} chapters_pairs/cross={}",
                size(self.work),
                size(self.chapters)
            ));
        }
        if let Some(reason) = self.reason {
            line.push_str(&format!(" reason={reason:?}"));
        }
        if let Some(section) = self.engine_section {
            line.push_str(&format!(" section={}", section.name()));
        }
        if let Some(statement) = self.last_statement {
            line.push_str(&format!(" last_statement={statement}"));
        }
        if let Some(doi) = &self.doi {
            line.push_str(&format!(" doi={}", doi.to_lowercase_string()));
        }
        line
    }
}

/// The log lines of one outcome (section 4.8): the level and fields of each,
/// from the outcome, the step it arose at, the switch state and the claim
/// counter's previous value.
///
/// - Switch `OFF`: only a debug reason line.
/// - A refused second selection (step 3): an info outcome line for the first
///   one of a request, debug for every later one.
/// - Otherwise one info outcome line, carrying the DOI only for an outcome
///   after step 9; a debug reason line for `METRIC_WIDGET_UNAVAILABLE` and
///   `METRIC_WIDGET_BUSY`, carrying the DOI after step 8; a warn line for a
///   checkout failure; an error line for `INTERNAL_ERROR` with the internal
///   reason, the section, the step, the last statement and the elapsed time.
pub(crate) fn log_decision(facts: &WidgetFacts) -> Vec<WidgetLogLine> {
    use log::Level;
    let doi_after = |step: WidgetStep| facts.doi.clone().filter(|_| facts.step > step);
    if facts.switch == MetricWidgetSwitch::Off {
        let mut line = WidgetLogLine::base(facts, Level::Debug, WidgetLineKind::Reason);
        line.reason = facts.reason;
        return vec![line];
    }
    if facts.step == WidgetStep::Claim {
        let level = if facts.previous_claims == Some(1) {
            Level::Info
        } else {
            Level::Debug
        };
        return vec![WidgetLogLine::base(facts, level, WidgetLineKind::Outcome)];
    }
    let mut lines = Vec::new();
    let mut outcome = WidgetLogLine::base(facts, Level::Info, WidgetLineKind::Outcome);
    outcome.work = facts.work;
    outcome.chapters = facts.chapters;
    outcome.doi = doi_after(WidgetStep::Entitlement);
    lines.push(outcome);
    match facts.error {
        Some(MetricWidgetError::Unavailable | MetricWidgetError::Busy) => {
            let mut reason = WidgetLogLine::base(facts, Level::Debug, WidgetLineKind::Reason);
            reason.reason = facts.reason;
            reason.doi = doi_after(WidgetStep::Resolution);
            lines.push(reason);
            if facts.reason == Some(WidgetReason::CheckoutFailed) {
                let mut pool = WidgetLogLine::base(facts, Level::Warn, WidgetLineKind::Pool);
                pool.reason = facts.reason;
                lines.push(pool);
            }
        }
        Some(MetricWidgetError::Internal) => {
            let mut internal = WidgetLogLine::base(facts, Level::Error, WidgetLineKind::Internal);
            internal.reason = facts.reason;
            internal.engine_section = facts.engine_section;
            internal.last_statement = facts.last_statement;
            internal.doi = doi_after(WidgetStep::Resolution);
            lines.push(internal);
        }
        _ => {}
    }
    lines
}

/// Write the lines through the process logger.
fn emit_lines(lines: &[WidgetLogLine]) {
    for line in lines {
        log::log!(line.level, "{}", line.message());
    }
}

// ---------------------------------------------------------------------------
// Slots
// ---------------------------------------------------------------------------

/// Widget reads running in this process. One process-wide counter, never
/// carried through app data or the request context, which are built per
/// worker; separate from any dashboard concurrency control.
static SLOTS_IN_USE: AtomicUsize = AtomicUsize::new(0);

/// One taken slot, released when dropped, panic unwinding included.
#[derive(Debug)]
struct SlotGuard;

impl Drop for SlotGuard {
    fn drop(&mut self) {
        SLOTS_IN_USE.fetch_sub(1, Ordering::SeqCst);
    }
}

/// Take a slot without waiting, or `None` when every slot is taken.
fn try_take_slot() -> Option<SlotGuard> {
    let mut taken = SLOTS_IN_USE.load(Ordering::SeqCst);
    while taken < METRIC_WIDGET_SLOTS {
        match SLOTS_IN_USE.compare_exchange_weak(
            taken,
            taken + 1,
            Ordering::SeqCst,
            Ordering::SeqCst,
        ) {
            Ok(_) => return Some(SlotGuard),
            Err(current) => taken = current,
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Statements
// ---------------------------------------------------------------------------

/// Step 8: the work whose DOI, lower-cased, equals `$1` (the parsed,
/// canonical, lower-cased input; `doi_uniq_idx` is unique on `lower(doi)`),
/// with its work type, its current imprint's publisher and that publisher's
/// package, and, unless the work is a chapter, its current `has-child` related
/// works of any type with their lower-cased DOIs and current publishers,
/// stopping after `$2` child rows. No row: no such work. It reads no Metrics
/// table.
pub(crate) const METRIC_WIDGET_RESOLUTION_SQL: &str =
    "SELECT w.work_id, w.work_type::text AS work_type, i.publisher_id, \
                    p.subscription_package, c.child_id, c.child_doi, c.child_publisher_id \
             FROM public.work w \
             JOIN public.imprint i ON i.imprint_id = w.imprint_id \
             JOIN public.publisher p ON p.publisher_id = i.publisher_id \
             LEFT JOIN LATERAL ( \
                 SELECT cw.work_id AS child_id, lower(cw.doi) AS child_doi, \
                        ci.publisher_id AS child_publisher_id \
                 FROM public.work_relation wr \
                 JOIN public.work cw ON cw.work_id = wr.related_work_id \
                 JOIN public.imprint ci ON ci.imprint_id = cw.imprint_id \
                 WHERE wr.relator_work_id = w.work_id \
                   AND wr.relation_type = 'has-child' \
                   AND w.work_type <> 'book-chapter' \
                 LIMIT $2 \
             ) c ON TRUE \
             WHERE lower(w.doi) = $1";

/// Step 11: the served pairs of one section: the distinct eligible
/// platform/measure pairs with a monthly projection row, or a total-ambiguous
/// month, for the section's works `$1` in the months `[$2, $3)`, ordered by
/// platform ID and measure ID, stopping after `$4` rows. A measure is eligible
/// only when it is public, enabled and additive across time and works, and a
/// platform only when it is enabled, so nothing else can be served, bound,
/// checked or read.
pub(crate) const METRIC_WIDGET_PAIRS_SQL: &str = "SELECT served.platform_id, served.measure_id \
             FROM ( \
                 SELECT mm.platform_id, mm.measure_id \
                 FROM public.metric_rollup_work_month mm \
                 WHERE mm.work_id = ANY($1) \
                   AND mm.month_start >= $2 \
                   AND mm.month_start < $3 \
                 UNION \
                 SELECT a.platform_id, a.measure_id \
                 FROM public.metric_rollup_work_month_ambiguity a \
                 WHERE a.work_id = ANY($1) \
                   AND a.month_start >= $2 \
                   AND a.month_start < $3 \
                   AND a.total_ambiguous \
             ) served \
             JOIN public.metric_platform p ON p.platform_id = served.platform_id \
             JOIN public.metric_measure m ON m.measure_id = served.measure_id \
             WHERE p.enabled \
               AND m.enabled \
               AND m.public_visibility \
               AND m.additive_across_time \
               AND m.additive_across_works \
             ORDER BY served.platform_id, served.measure_id \
             LIMIT $4";

/// Step 12: the transaction timestamp and the rollup frontier's
/// `watermark_at`, and the public labels of the served platforms `$1` and
/// measures `$2`.
pub(crate) const METRIC_WIDGET_LABELS_SQL: &str =
    "SELECT 'FRONTIER' AS kind, NULL::uuid AS id, NULL::text AS code, \
                    NULL::text AS display_name, NULL::text AS description, NULL::text AS unit, \
                    transaction_timestamp() AS as_of, s.watermark_at \
             FROM public.metric_rollup_work_day_state s \
             WHERE s.state_id = 1 \
             UNION ALL \
             SELECT 'PLATFORM', p.platform_id, p.code, p.display_name, p.public_description, \
                    NULL::text, NULL::timestamptz, NULL::timestamptz \
             FROM public.metric_platform p \
             WHERE p.platform_id = ANY($1) \
             UNION ALL \
             SELECT 'MEASURE', m.measure_id, m.code, m.display_name, m.definition, \
                    m.unit::text, NULL::timestamptz, NULL::timestamptz \
             FROM public.metric_measure m \
             WHERE m.measure_id = ANY($2)";

#[derive(diesel::QueryableByName)]
struct ResolutionRow {
    #[diesel(sql_type = SqlUuid)]
    work_id: Uuid,
    #[diesel(sql_type = Text)]
    work_type: String,
    #[diesel(sql_type = SqlUuid)]
    publisher_id: Uuid,
    #[diesel(sql_type = crate::schema::sql_types::ThothPackage)]
    subscription_package: ThothPackage,
    #[diesel(sql_type = Nullable<SqlUuid>)]
    child_id: Option<Uuid>,
    #[diesel(sql_type = Nullable<Text>)]
    child_doi: Option<String>,
    #[diesel(sql_type = Nullable<SqlUuid>)]
    child_publisher_id: Option<Uuid>,
}

#[derive(diesel::QueryableByName)]
struct PairRow {
    #[diesel(sql_type = SqlUuid)]
    platform_id: Uuid,
    #[diesel(sql_type = SqlUuid)]
    measure_id: Uuid,
}

#[derive(diesel::QueryableByName)]
struct LabelRow {
    #[diesel(sql_type = Text)]
    kind: String,
    #[diesel(sql_type = Nullable<SqlUuid>)]
    id: Option<Uuid>,
    #[diesel(sql_type = Nullable<Text>)]
    code: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    display_name: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    description: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    unit: Option<String>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    as_of: Option<Timestamp>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    watermark_at: Option<Timestamp>,
}

// ---------------------------------------------------------------------------
// Input validation (no database access)
// ---------------------------------------------------------------------------

/// A request that passed every check that needs no database.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ValidatedInput {
    doi: Doi,
    /// Canonical, lower-cased entries; empty selects every child work.
    chapter_dois: Vec<String>,
    start: NaiveDate,
    end: NaiveDate,
}

fn months_between(start: NaiveDate, end: NaiveDate) -> i64 {
    (i64::from(end.year()) * 12 + i64::from(end.month0()))
        - (i64::from(start.year()) * 12 + i64::from(start.month0()))
}

/// Parse a GraphQL `Doi` value, which the scalar accepts as any string.
fn parse_doi(doi: &Doi) -> Result<Doi, Failure> {
    Doi::from_str(&doi.0)
        .map_err(|_| Failure::input(MetricWidgetError::QueryInvalid(INVALID_DOI_MESSAGE)))
}

/// Step 4, in order; the first failure is returned, whatever work the DOI
/// names.
fn validate(input: &MetricWidgetInput, today: NaiveDate) -> Result<ValidatedInput, Failure> {
    let invalid = |message| Failure::input(MetricWidgetError::QueryInvalid(message));
    let limit = |message| Failure::input(MetricWidgetError::QueryLimitExceeded(message));
    let doi = parse_doi(&input.doi)?;
    let (start, end) = (input.start_date, input.end_date);
    if start.day() != 1 || end.day() != 1 || start >= end {
        return Err(invalid(DATES_MESSAGE));
    }
    if start < date_floor() || (end - today).num_days() > METRIC_WIDGET_END_DATE_CEILING_DAYS {
        return Err(limit(DATE_DOMAIN_MESSAGE));
    }
    if months_between(start, end) > i64::from(METRIC_WIDGET_MAX_RANGE_MONTHS) {
        return Err(limit(RANGE_MESSAGE));
    }
    let entries = input.chapter_dois.as_deref().unwrap_or_default();
    if entries.len() > METRIC_WIDGET_MAX_CHAPTER_DOIS {
        return Err(limit(TOO_MANY_CHAPTER_DOIS_MESSAGE));
    }
    let parsed = entries
        .iter()
        .map(|entry| parse_doi(entry).map(|doi| doi.to_lowercase_string()))
        .collect::<Result<Vec<String>, Failure>>()?;
    if parsed.iter().collect::<BTreeSet<_>>().len() != parsed.len() {
        return Err(invalid(DUPLICATE_CHAPTER_DOI_MESSAGE));
    }
    Ok(ValidatedInput {
        doi,
        chapter_dois: parsed,
        start,
        end,
    })
}

// ---------------------------------------------------------------------------
// The read
// ---------------------------------------------------------------------------

/// The cost profile and test seams of one read. Production uses
/// [`Profile::production`]; tests vary the timeouts, the date and a hook run
/// at the start of each step from step 5 on.
struct Profile<'a> {
    statement_timeout_sql: &'static str,
    deadline: Duration,
    checkout_timeout: Duration,
    today: NaiveDate,
    hook: &'a dyn Fn(WidgetStep),
}

impl Profile<'static> {
    fn production() -> Self {
        Profile {
            statement_timeout_sql: METRIC_WIDGET_STATEMENT_TIMEOUT_SQL,
            deadline: METRIC_WIDGET_DEADLINE,
            checkout_timeout: METRIC_WIDGET_CHECKOUT_TIMEOUT,
            today: Utc::now().date_naive(),
            hook: &|_| {},
        }
    }
}

/// Answer one `metricWidget` selection.
///
/// `claims` is the request context's claim counter: only the selection that
/// claims it first may read. The caller's identity is never consulted.
pub(crate) fn metric_widget(
    db: &PgPool,
    switch: MetricWidgetSwitch,
    claims: &AtomicUsize,
    input: &MetricWidgetInput,
) -> Result<MetricWidget, MetricWidgetError> {
    resolve(
        db,
        switch,
        claims,
        input,
        &Profile::production(),
        &mut emit_lines,
    )
}

/// [`metric_widget`] with an explicit profile and log sink. The log decision
/// runs exactly once per call.
fn resolve(
    db: &PgPool,
    switch: MetricWidgetSwitch,
    claims: &AtomicUsize,
    input: &MetricWidgetInput,
    profile: &Profile<'_>,
    sink: &mut dyn FnMut(&[WidgetLogLine]),
) -> Result<MetricWidget, MetricWidgetError> {
    let started = Instant::now();
    let mut facts = WidgetFacts {
        switch,
        previous_claims: None,
        error: None,
        reason: None,
        step: WidgetStep::Switch,
        elapsed_ms: 0,
        doi: None,
        work: None,
        chapters: None,
        engine_section: None,
        last_statement: None,
    };
    let result = if switch == MetricWidgetSwitch::Off {
        Err(Failure::new(
            MetricWidgetError::Unavailable,
            WidgetReason::SwitchOff,
        ))
    } else {
        let previous = claims.fetch_add(1, Ordering::SeqCst);
        facts.previous_claims = Some(previous);
        if previous > 0 {
            facts.step = WidgetStep::Claim;
            Err(Failure::input(MetricWidgetError::QueryLimitExceeded(
                ONE_SELECTION_MESSAGE,
            )))
        } else {
            read(db, input, profile, &mut facts)
        }
    };
    facts.elapsed_ms = started.elapsed().as_millis();
    if let Err(failure) = &result {
        facts.error = Some(failure.error);
        facts.reason = failure.reason;
    }
    sink(&log_decision(&facts));
    result.map_err(|failure| failure.error)
}

/// Steps 4 to 14 of the first selection of a request.
fn read(
    db: &PgPool,
    input: &MetricWidgetInput,
    profile: &Profile<'_>,
    facts: &mut WidgetFacts,
) -> Result<MetricWidget, Failure> {
    facts.step = WidgetStep::Input;
    let request = validate(input, profile.today)?;
    facts.doi = Some(request.doi.clone());

    // 5. A slot, without waiting; the deadline starts here.
    facts.step = WidgetStep::Slot;
    (profile.hook)(WidgetStep::Slot);
    let _slot = try_take_slot()
        .ok_or_else(|| Failure::new(MetricWidgetError::Busy, WidgetReason::NoSlot))?;
    let deadline = Deadline::new(Instant::now() + profile.deadline);

    // 6. One connection, then one read-only repeatable-read transaction.
    facts.step = WidgetStep::Checkout;
    (profile.hook)(WidgetStep::Checkout);
    let mut connection = db
        .get_timeout(profile.checkout_timeout)
        .map_err(|_| Failure::new(MetricWidgetError::Busy, WidgetReason::CheckoutFailed))?;
    let outcome = connection
        .build_transaction()
        .read_only()
        .repeatable_read()
        .run(|connection| transact(connection, &request, profile, &deadline, facts));
    facts.last_statement = deadline.last_statement();
    let widget = outcome?;
    facts.step = WidgetStep::Commit;
    Ok(widget)
}

/// The served child works of a book, after validation and exclusion.
struct Children {
    served: Vec<Uuid>,
    excluded: usize,
}

/// Step 10.
fn select_children(
    request: &ValidatedInput,
    rows: &[ResolutionRow],
    is_chapter: bool,
    publisher_id: Uuid,
) -> Result<Children, Failure> {
    if is_chapter {
        if !request.chapter_dois.is_empty() {
            return Err(Failure::input(MetricWidgetError::QueryInvalid(
                NOT_A_CHILD_MESSAGE,
            )));
        }
        return Ok(Children {
            served: Vec::new(),
            excluded: 0,
        });
    }
    let mut children: BTreeMap<Uuid, (Option<String>, Uuid)> = BTreeMap::new();
    for row in rows {
        match (row.child_id, row.child_publisher_id) {
            (Some(child_id), Some(child_publisher)) => {
                // `work_relation_relator_related_uniq` allows one row per child.
                if children
                    .insert(child_id, (row.child_doi.clone(), child_publisher))
                    .is_some()
                {
                    return Err(Failure::new(
                        MetricWidgetError::Internal,
                        WidgetReason::UnexpectedRow,
                    ));
                }
            }
            (None, None) => {}
            _ => {
                return Err(Failure::new(
                    MetricWidgetError::Internal,
                    WidgetReason::UnexpectedRow,
                ))
            }
        }
    }
    if children.len() > METRIC_WIDGET_MAX_CHILD_WORKS {
        return Err(Failure::input(MetricWidgetError::QueryLimitExceeded(
            TOO_MANY_CHILDREN_MESSAGE,
        )));
    }
    let child_dois: BTreeSet<&str> = children
        .values()
        .filter_map(|(doi, _)| doi.as_deref())
        .collect();
    if request
        .chapter_dois
        .iter()
        .any(|entry| !child_dois.contains(entry.as_str()))
    {
        return Err(Failure::input(MetricWidgetError::QueryInvalid(
            NOT_A_CHILD_MESSAGE,
        )));
    }
    let wanted: BTreeSet<&str> = request.chapter_dois.iter().map(String::as_str).collect();
    let mut served = Vec::new();
    let mut excluded = 0;
    for (child_id, (doi, child_publisher)) in &children {
        let selected = wanted.is_empty() || doi.as_deref().is_some_and(|doi| wanted.contains(doi));
        if !selected {
            continue;
        }
        if *child_publisher == publisher_id {
            served.push(*child_id);
        } else {
            excluded += 1;
        }
    }
    Ok(Children { served, excluded })
}

/// The request deadline of one read (section 4.10): the engine's
/// [`ReadDeadline`] for both engine runs, and its instant, against which this
/// module admits its own statements.
struct Deadline {
    at: Instant,
    engine: ReadDeadline,
    /// The last widget-owned statement admitted.
    widget_statement: Cell<Option<&'static str>>,
    /// Whether an engine run has started since that statement.
    engine_since: Cell<bool>,
}

impl Deadline {
    /// A deadline at `at`, created when the read takes its slot.
    fn new(at: Instant) -> Self {
        Deadline {
            at,
            engine: ReadDeadline::new(at),
            widget_statement: Cell::new(None),
            engine_since: Cell::new(false),
        }
    }

    /// Admit one widget-owned statement, refusing it at or after the instant
    /// exactly as the engine's own check does. A refusal here sets no engine
    /// flag and writes no engine log line.
    fn admit(&self, name: &'static str) -> Result<(), Failure> {
        if Instant::now() >= self.at {
            return Err(Failure::new(
                MetricWidgetError::Busy,
                WidgetReason::DeadlinePassed,
            ));
        }
        self.widget_statement.set(Some(name));
        self.engine_since.set(false);
        Ok(())
    }

    /// The engine's deadline, for one engine run that starts now.
    fn for_engine_run(&self) -> &ReadDeadline {
        self.engine_since.set(true);
        &self.engine
    }

    /// The last statement started: the engine's last, when an engine run has
    /// started since the last widget-owned statement and the engine has
    /// admitted one, otherwise that widget-owned statement. A second run
    /// refused at its first check still names the first run's last
    /// statement; that outcome is `METRIC_WIDGET_BUSY`, whose lines name no
    /// statement.
    fn last_statement(&self) -> Option<&'static str> {
        self.engine
            .last_statement()
            .filter(|_| self.engine_since.get())
            .or(self.widget_statement.get())
    }
}

/// Step 11 for one section.
fn served_pairs(
    connection: &mut PgConnection,
    works: &[Uuid],
    request: &ValidatedInput,
    deadline: &Deadline,
) -> Result<Vec<(Uuid, Uuid)>, Failure> {
    deadline.admit("METRIC_WIDGET_PAIRS_SQL")?;
    let rows: Vec<PairRow> = diesel::sql_query(METRIC_WIDGET_PAIRS_SQL)
        .bind::<Array<SqlUuid>, _>(works)
        .bind::<Date, _>(request.start)
        .bind::<Date, _>(request.end)
        .bind::<SqlBigInt, _>(METRIC_WIDGET_MAX_PAIRS as i64 + 1)
        .load(connection)?;
    Ok(rows
        .into_iter()
        .map(|row| (row.platform_id, row.measure_id))
        .collect())
}

/// Step 13 for one section with at least one served pair.
#[allow(clippy::too_many_arguments)]
fn engine_section(
    connection: &mut PgConnection,
    section: WidgetSection,
    publisher_id: Uuid,
    works: &[Uuid],
    pairs: &[(Uuid, Uuid)],
    request: &ValidatedInput,
    frontier: (Timestamp, Timestamp),
    deadline: &Deadline,
    facts: &mut WidgetFacts,
) -> Result<MetricWidgetSection, Failure> {
    let read = MetricPairRead::new(publisher_id, works, pairs, request.start, request.end)
        .with_timeline_grain(MetricTimelineGrain::Month)
        .with_countries(true)
        .with_institutions(false)
        .with_capability(PublisherCapability::MetricsWidget);
    let result =
        metric_pair_read(connection, &read, deadline.for_engine_run()).map_err(|error| {
            facts.engine_section = Some(section);
            engine_failure(&error, deadline.engine.expired())
        })?;
    if (result.as_of, result.rollup_watermark) != frontier {
        facts.engine_section = Some(section);
        return Err(Failure::new(
            MetricWidgetError::Internal,
            WidgetReason::FrontierMismatch,
        ));
    }
    Ok(section_of(result))
}

/// A served section from one engine result.
fn section_of(result: MetricDashboard) -> MetricWidgetSection {
    MetricWidgetSection {
        coverage: result
            .coverage
            .items
            .iter()
            .map(|item| MetricWidgetPairCoverage {
                platform_id: item.platform_id,
                measure_id: item.measure_id,
                status: item.status,
                data_through: item.data_through,
                country_coverage: item.country_coverage,
            })
            .collect(),
        totals: result.totals,
        timeline: result.timeline,
        countries: result.countries,
        data_through: result.data_through,
        is_partial: result.is_partial,
    }
}

/// A section that serves no pair (section 4.6): no engine statement runs.
fn empty_section() -> MetricWidgetSection {
    MetricWidgetSection {
        totals: Vec::new(),
        timeline: Vec::new(),
        countries: Vec::new(),
        coverage: Vec::new(),
        data_through: None,
        is_partial: true,
    }
}

/// Steps 7 to 13, inside the transaction.
fn transact(
    connection: &mut PgConnection,
    request: &ValidatedInput,
    profile: &Profile<'_>,
    deadline: &Deadline,
    facts: &mut WidgetFacts,
) -> Result<MetricWidget, Failure> {
    // 7. The prelude, before any query: the engine's planner setting, the
    // widget's statement timeout and the engine's projection lock. A lock
    // that cannot be taken (a rebuild holds the tables) is busy, never a
    // read of truncated tables.
    facts.step = WidgetStep::Prelude;
    (profile.hook)(WidgetStep::Prelude);
    diesel::sql_query(CUSTOM_PLANS_SQL).execute(connection)?;
    diesel::sql_query(profile.statement_timeout_sql).execute(connection)?;
    diesel::sql_query(PROJECTION_LOCK_SQL)
        .execute(connection)
        .map_err(|_| Failure::new(MetricWidgetError::Busy, WidgetReason::LockFailed))?;

    // 8. The work, its publisher's package and its child works.
    facts.step = WidgetStep::Resolution;
    (profile.hook)(WidgetStep::Resolution);
    deadline.admit("METRIC_WIDGET_RESOLUTION_SQL")?;
    let rows: Vec<ResolutionRow> = diesel::sql_query(METRIC_WIDGET_RESOLUTION_SQL)
        .bind::<Text, _>(request.doi.to_lowercase_string())
        .bind::<SqlBigInt, _>(METRIC_WIDGET_MAX_CHILD_WORKS as i64 + 1)
        .load(connection)?;

    // 9. One refusal for an unknown work and for an unentitled publisher.
    facts.step = WidgetStep::Entitlement;
    let Some(first) = rows.first() else {
        return Err(Failure::new(
            MetricWidgetError::Unavailable,
            WidgetReason::UnknownWork,
        ));
    };
    if !first
        .subscription_package
        .has_capability(PublisherCapability::MetricsWidget)
    {
        return Err(Failure::new(
            MetricWidgetError::Unavailable,
            WidgetReason::NotEntitled,
        ));
    }
    let (work_id, publisher_id) = (first.work_id, first.publisher_id);
    let is_chapter = first.work_type == "book-chapter";
    if rows
        .iter()
        .any(|row| row.work_id != work_id || row.publisher_id != publisher_id)
    {
        return Err(Failure::new(
            MetricWidgetError::Internal,
            WidgetReason::UnexpectedRow,
        ));
    }

    // 10. Child works.
    facts.step = WidgetStep::Children;
    (profile.hook)(WidgetStep::Children);
    let children = select_children(request, &rows, is_chapter, publisher_id)?;

    // 11. Served pairs per section, then the pair bound.
    facts.step = WidgetStep::Pairs;
    (profile.hook)(WidgetStep::Pairs);
    let work_works = [work_id];
    let work_pairs = served_pairs(connection, &work_works, request, deadline)?;
    facts.work = Some(SectionSize::of(&work_pairs));
    let chapter_pairs = if children.served.is_empty() {
        Vec::new()
    } else {
        let pairs = served_pairs(connection, &children.served, request, deadline)?;
        facts.chapters = Some(SectionSize::of(&pairs));
        pairs
    };
    if work_pairs.len() > METRIC_WIDGET_MAX_PAIRS || chapter_pairs.len() > METRIC_WIDGET_MAX_PAIRS {
        return Err(Failure::input(MetricWidgetError::QueryLimitExceeded(
            TOO_MANY_PAIRS_MESSAGE,
        )));
    }

    // 12. Labels and the frontier, on every path that reaches here.
    facts.step = WidgetStep::Labels;
    (profile.hook)(WidgetStep::Labels);
    let platform_ids: BTreeSet<Uuid> = work_pairs
        .iter()
        .chain(&chapter_pairs)
        .map(|(platform_id, _)| *platform_id)
        .collect();
    let measure_ids: BTreeSet<Uuid> = work_pairs
        .iter()
        .chain(&chapter_pairs)
        .map(|(_, measure_id)| *measure_id)
        .collect();
    deadline.admit("METRIC_WIDGET_LABELS_SQL")?;
    let labels: Vec<LabelRow> = diesel::sql_query(METRIC_WIDGET_LABELS_SQL)
        .bind::<Array<SqlUuid>, _>(platform_ids.iter().copied().collect::<Vec<Uuid>>())
        .bind::<Array<SqlUuid>, _>(measure_ids.iter().copied().collect::<Vec<Uuid>>())
        .load(connection)?;
    let (frontier, platforms, measures) = interpret_labels(labels, &platform_ids, &measure_ids)?;

    // 13. The engine, per section with a served pair, on this connection and
    // snapshot.
    facts.step = WidgetStep::Engine;
    (profile.hook)(WidgetStep::Engine);
    let work = if work_pairs.is_empty() {
        empty_section()
    } else {
        engine_section(
            connection,
            WidgetSection::Work,
            publisher_id,
            &work_works,
            &work_pairs,
            request,
            frontier,
            deadline,
            facts,
        )?
    };
    let chapters = if chapter_pairs.is_empty() {
        if children.served.is_empty() {
            None
        } else {
            Some(empty_section())
        }
    } else {
        Some(engine_section(
            connection,
            WidgetSection::Chapters,
            publisher_id,
            &children.served,
            &chapter_pairs,
            request,
            frontier,
            deadline,
            facts,
        )?)
    };

    // 14. Commit follows this closure; it is not checked against the
    // deadline.
    facts.step = WidgetStep::Commit;
    (profile.hook)(WidgetStep::Commit);
    Ok(MetricWidget {
        work,
        chapters,
        excluded_child_count: i32::try_from(children.excluded)
            .map_err(|_| Failure::new(MetricWidgetError::Internal, WidgetReason::UnexpectedRow))?,
        platforms,
        measures,
        start_date: request.start,
        end_date: request.end,
        as_of: frontier.0,
        rollup_watermark: frontier.1,
    })
}

/// The frontier `(asOf, watermark)`, and the labels of exactly the served
/// platforms and measures, ordered by ID.
type Labels = (
    (Timestamp, Timestamp),
    Vec<MetricWidgetPlatform>,
    Vec<MetricWidgetMeasure>,
);

fn interpret_labels(
    rows: Vec<LabelRow>,
    platform_ids: &BTreeSet<Uuid>,
    measure_ids: &BTreeSet<Uuid>,
) -> Result<Labels, Failure> {
    let unexpected = || Failure::new(MetricWidgetError::Internal, WidgetReason::UnexpectedRow);
    let mut frontier = None;
    let mut platforms: BTreeMap<Uuid, MetricWidgetPlatform> = BTreeMap::new();
    let mut measures: BTreeMap<Uuid, MetricWidgetMeasure> = BTreeMap::new();
    for row in rows {
        match row.kind.as_str() {
            "FRONTIER" => {
                let (Some(as_of), Some(watermark)) = (row.as_of, row.watermark_at) else {
                    return Err(unexpected());
                };
                frontier = Some((as_of, watermark));
            }
            "PLATFORM" => {
                let (Some(platform_id), Some(code), Some(display_name)) =
                    (row.id, row.code, row.display_name)
                else {
                    return Err(unexpected());
                };
                platforms.insert(
                    platform_id,
                    MetricWidgetPlatform {
                        platform_id,
                        code,
                        display_name,
                        public_description: row.description,
                    },
                );
            }
            "MEASURE" => {
                let (
                    Some(measure_id),
                    Some(code),
                    Some(display_name),
                    Some(definition),
                    Some(unit),
                ) = (
                    row.id,
                    row.code,
                    row.display_name,
                    row.description,
                    row.unit,
                )
                else {
                    return Err(unexpected());
                };
                let unit = MetricMeasureUnit::from_str(&unit).map_err(|_| unexpected())?;
                measures.insert(
                    measure_id,
                    MetricWidgetMeasure {
                        measure_id,
                        code,
                        display_name,
                        unit,
                        definition,
                    },
                );
            }
            _ => return Err(unexpected()),
        }
    }
    let frontier = frontier.ok_or(Failure::new(
        MetricWidgetError::Internal,
        WidgetReason::FrontierMissing,
    ))?;
    if platforms.keys().ne(platform_ids.iter()) || measures.keys().ne(measure_ids.iter()) {
        return Err(Failure::new(
            MetricWidgetError::Internal,
            WidgetReason::LabelMissing,
        ));
    }
    Ok((
        frontier,
        platforms.into_values().collect(),
        measures.into_values().collect(),
    ))
}

#[cfg(test)]
mod tests;
