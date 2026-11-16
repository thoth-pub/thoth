//! The public, read-only `workFilterOptions` read (`MET-WP10-FILTER-API-01`).
//!
//! An anonymous caller names a set of publishers and receives, in one
//! response, every filter value their works carry: the publishers
//! themselves, the series, language codes, work types, funding institutions
//! and contributor-affiliation institutions. Every list is complete,
//! deduplicated and sorted in Rust by byte order, or the request fails; no
//! list is ever truncated or partial. No role is required and the caller's
//! identity is never consulted.
//!
//! The work set is the one `WORKS_SQL` in `model::metric_dashboard` resolves
//! for the same publishers with every other selector empty, and every list
//! other than `publishers` holds exactly the values of that dimension that
//! `WORKS_SQL` would resolve at least one work for (the parity invariant).
//!
//! # Order of operations
//!
//! 1. One `workFilterOptions` field per HTTP request, counted on the request
//!    context: every later resolution is refused and reads nothing.
//! 2. Input-only validation, with no slot and no database access.
//! 3. A process-wide slot, taken without waiting.
//! 4. One pool connection with a one-second checkout timeout.
//! 5. One `READ ONLY` transaction at the default `READ COMMITTED` level, with
//!    exactly two settings: custom plans and a ten-second statement timeout.
//! 6. One statement, [`WORK_FILTER_OPTIONS_SQL`], returning every list and
//!    the unknown publisher IDs from one snapshot.
//! 7. Unknown publisher IDs, then the per-list bound, then sorting.
//!
//! The connection and the slot are released on every exit path, panic
//! unwinding included. Nothing in this module returns SQL, database, pool or
//! host text to the caller.

use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use diesel::pg::PgConnection;
use diesel::result::Error as DieselError;
use diesel::sql_types::{Array, BigInt as SqlBigInt, Nullable, Text, Uuid as SqlUuid};
use diesel::RunQueryDsl;
use juniper::{graphql_value, FieldError, IntoFieldError};
use uuid::Uuid;

use crate::db::PgPool;
use crate::model::language::LanguageCode;
use crate::model::series::SeriesType;
use crate::model::work::WorkType;
use crate::model::Ror;

// ---------------------------------------------------------------------------
// Bounds
// ---------------------------------------------------------------------------

/// The most publisher IDs one read accepts. Deliberately independent of the
/// `metricDashboard` publisher bound.
pub const WORK_FILTER_OPTIONS_MAX_PUBLISHERS: usize = 100;
/// The most entries any one list may hold; a list over it fails the request.
pub const WORK_FILTER_OPTIONS_MAX_ENTRIES: usize = 20_000;
/// The most `workFilterOptions` reads one API process runs at once.
pub const WORK_FILTER_OPTIONS_MAX_CONCURRENT_READS: usize = 2;
/// How long a read waits for a pool connection.
pub const WORK_FILTER_OPTIONS_CHECKOUT_TIMEOUT: Duration = Duration::from_secs(1);

/// The first of the two transaction settings: plan the statement for its
/// actual arguments. Diesel caches `sql_query` statements per connection,
/// and a generic plan for `= ANY($1)` can be slow.
pub(crate) const WORK_FILTER_OPTIONS_CUSTOM_PLANS_SQL: &str =
    "SET LOCAL plan_cache_mode = force_custom_plan";
/// The second of the two transaction settings: the statement timeout.
pub(crate) const WORK_FILTER_OPTIONS_STATEMENT_TIMEOUT_SQL: &str =
    "SET LOCAL statement_timeout = '10s'";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(juniper::GraphQLObject, Debug, Clone, PartialEq, Eq)]
#[graphql(
    description = "The filter values of a set of publishers' works: every list is complete, deduplicated and sorted"
)]
pub struct WorkFilterOptions {
    #[graphql(
        description = "Every requested publisher, including one with no works, ordered by name, then ID"
    )]
    pub publishers: Vec<WorkFilterPublisher>,
    #[graphql(
        description = "Every series that issues one of the works, or that issues the parent work of one of the works that is a book chapter (the parent may belong to another publisher), ordered by name, then ID"
    )]
    pub series: Vec<WorkFilterSeries>,
    #[graphql(
        description = "Every language code on a language record of one of the works, ordered by code"
    )]
    pub language_codes: Vec<LanguageCode>,
    #[graphql(description = "Every work type of the works, ordered by its database label")]
    pub work_types: Vec<WorkType>,
    #[graphql(
        description = "Every institution that funds one of the works, ordered by name, then ID"
    )]
    pub funding_institutions: Vec<WorkFilterInstitution>,
    #[graphql(
        description = "Every institution of an affiliation of a contribution to one of the works, ordered by name, then ID"
    )]
    pub affiliation_institutions: Vec<WorkFilterInstitution>,
}

#[derive(juniper::GraphQLObject, Debug, Clone, PartialEq, Eq)]
#[graphql(description = "A publisher offered as a work filter value")]
pub struct WorkFilterPublisher {
    #[graphql(description = "Thoth ID of the publisher")]
    pub publisher_id: Uuid,
    #[graphql(description = "Name of the publisher")]
    pub publisher_name: String,
}

#[derive(juniper::GraphQLObject, Debug, Clone, PartialEq, Eq)]
#[graphql(description = "A series offered as a work filter value")]
pub struct WorkFilterSeries {
    #[graphql(description = "Thoth ID of the series")]
    pub series_id: Uuid,
    #[graphql(description = "Name of the series")]
    pub series_name: String,
    #[graphql(description = "Type of the series")]
    pub series_type: SeriesType,
    #[graphql(description = "Print ISSN of the series")]
    pub issn_print: Option<String>,
    #[graphql(description = "Electronic ISSN of the series")]
    pub issn_digital: Option<String>,
}

#[derive(juniper::GraphQLObject, Debug, Clone, PartialEq, Eq)]
#[graphql(description = "An institution offered as a work filter value")]
pub struct WorkFilterInstitution {
    #[graphql(description = "Thoth ID of the institution")]
    pub institution_id: Uuid,
    #[graphql(description = "Name of the institution")]
    pub institution_name: String,
    #[graphql(description = "ROR identifier of the institution")]
    pub ror: Option<Ror>,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

const ONE_PER_REQUEST_MESSAGE: &str =
    "Only one workFilterOptions field may be requested at a time.";
pub(crate) const EMPTY_MESSAGE: &str = "publisherIds must contain at least one publisher.";
pub(crate) const DUPLICATE_MESSAGE: &str = "publisherIds must not contain duplicate values.";
pub(crate) const TOO_MANY_MESSAGE: &str = "publisherIds may contain at most 100 publishers.";
const BUSY_MESSAGE: &str = "workFilterOptions is busy; try again shortly.";
const TIMEOUT_MESSAGE: &str = "workFilterOptions timed out; try again shortly.";
const UNAVAILABLE_MESSAGE: &str = "workFilterOptions could not be completed; try again shortly.";

/// The exact server message of a statement cancelled by `statement_timeout`.
/// Diesel 2.3 exposes no SQLSTATE; the text depends on the server's
/// `lc_messages`, and any other text is `UNAVAILABLE`.
const STATEMENT_TIMEOUT_TEXT: &str = "canceling statement due to statement timeout";

/// A `workFilterOptions` refusal or failure. Every message is fixed text,
/// apart from the requested IDs and the list name and bound it names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkFilterOptionsError {
    /// `WORK_FILTER_OPTIONS_ONE_PER_REQUEST`.
    OnePerRequest,
    /// `WORK_FILTER_OPTIONS_INVALID`, with its fixed message.
    Invalid(&'static str),
    /// `WORK_FILTER_OPTIONS_UNKNOWN_PUBLISHER`, with the unknown IDs sorted.
    UnknownPublisher(Vec<Uuid>),
    /// `WORK_FILTER_OPTIONS_LIMIT_EXCEEDED`: the first list, in field order,
    /// over the bound in force.
    LimitExceeded {
        /// The GraphQL name of the list.
        field: &'static str,
        /// The bound in force.
        bound: usize,
    },
    /// `WORK_FILTER_OPTIONS_BUSY`: no free slot, or no connection within the
    /// checkout timeout. Retryable.
    Busy,
    /// `WORK_FILTER_OPTIONS_TIMEOUT`: the statement timeout fired. Retryable.
    Timeout,
    /// `WORK_FILTER_OPTIONS_UNAVAILABLE`: any other database or pool
    /// failure. Retryable.
    Unavailable,
}

impl WorkFilterOptionsError {
    /// The stable `extensions.type` classification.
    pub fn code(&self) -> &'static str {
        match self {
            WorkFilterOptionsError::OnePerRequest => "WORK_FILTER_OPTIONS_ONE_PER_REQUEST",
            WorkFilterOptionsError::Invalid(_) => "WORK_FILTER_OPTIONS_INVALID",
            WorkFilterOptionsError::UnknownPublisher(_) => "WORK_FILTER_OPTIONS_UNKNOWN_PUBLISHER",
            WorkFilterOptionsError::LimitExceeded { .. } => "WORK_FILTER_OPTIONS_LIMIT_EXCEEDED",
            WorkFilterOptionsError::Busy => "WORK_FILTER_OPTIONS_BUSY",
            WorkFilterOptionsError::Timeout => "WORK_FILTER_OPTIONS_TIMEOUT",
            WorkFilterOptionsError::Unavailable => "WORK_FILTER_OPTIONS_UNAVAILABLE",
        }
    }

    /// The client-facing message.
    pub fn message(&self) -> String {
        match self {
            WorkFilterOptionsError::OnePerRequest => ONE_PER_REQUEST_MESSAGE.to_string(),
            WorkFilterOptionsError::Invalid(message) => (*message).to_string(),
            WorkFilterOptionsError::UnknownPublisher(ids) => format!(
                "publisherIds contains unknown publisher(s): {}.",
                ids.iter()
                    .map(Uuid::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            WorkFilterOptionsError::LimitExceeded { field, bound } => {
                format!("workFilterOptions.{field} exceeds {bound} entries.")
            }
            WorkFilterOptionsError::Busy => BUSY_MESSAGE.to_string(),
            WorkFilterOptionsError::Timeout => TIMEOUT_MESSAGE.to_string(),
            WorkFilterOptionsError::Unavailable => UNAVAILABLE_MESSAGE.to_string(),
        }
    }
}

impl IntoFieldError for WorkFilterOptionsError {
    fn into_field_error(self) -> FieldError {
        let code = self.code();
        FieldError::new(self.message(), graphql_value!({ "type": code }))
    }
}

/// Why a retryable read failed, for operators only. Never returned to the
/// caller, and no variant carries input or database text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Cause {
    /// Every slot of this process is taken.
    NoSlot,
    /// No pool connection within the checkout timeout, or a pool failure.
    Checkout,
    /// The statement timeout fired.
    Timeout,
    /// A row could not be interpreted, for example an enum label Diesel
    /// cannot map.
    Deserialize,
    /// Any other database failure.
    Database,
}

impl Cause {
    fn name(self) -> &'static str {
        match self {
            Cause::NoSlot => "no-slot",
            Cause::Checkout => "checkout",
            Cause::Timeout => "timeout",
            Cause::Deserialize => "deserialize",
            Cause::Database => "database",
        }
    }
}

/// A failure inside a read, with the operator cause of a retryable one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Failure {
    pub(crate) error: WorkFilterOptionsError,
    pub(crate) cause: Option<Cause>,
}

impl Failure {
    fn retryable(error: WorkFilterOptionsError, cause: Cause) -> Self {
        Failure {
            error,
            cause: Some(cause),
        }
    }

    fn input(error: WorkFilterOptionsError) -> Self {
        Failure { error, cause: None }
    }
}

impl From<DieselError> for Failure {
    fn from(error: DieselError) -> Self {
        match error {
            DieselError::DatabaseError(_, ref info) if info.message() == STATEMENT_TIMEOUT_TEXT => {
                Failure::retryable(WorkFilterOptionsError::Timeout, Cause::Timeout)
            }
            DieselError::DeserializationError(_) => {
                Failure::retryable(WorkFilterOptionsError::Unavailable, Cause::Deserialize)
            }
            _ => Failure::retryable(WorkFilterOptionsError::Unavailable, Cause::Database),
        }
    }
}

/// The one log line of a retryable failure: its level and text. A no-slot
/// `BUSY` is `debug` (anonymous load can drive it), a checkout `BUSY` and a
/// `TIMEOUT` are `warn`, and `UNAVAILABLE` is `error`. Other failures log
/// nothing.
pub(crate) fn log_line(failure: &Failure) -> Option<(log::Level, String)> {
    let cause = failure.cause?;
    let level = match cause {
        Cause::NoSlot => log::Level::Debug,
        Cause::Checkout | Cause::Timeout => log::Level::Warn,
        Cause::Deserialize | Cause::Database => log::Level::Error,
    };
    Some((
        level,
        format!(
            "workFilterOptions failed: type={} cause={}",
            failure.error.code(),
            cause.name()
        ),
    ))
}

fn emit(failure: &Failure) {
    if let Some((level, line)) = log_line(failure) {
        log::log!(level, "{line}");
    }
}

// ---------------------------------------------------------------------------
// Slots
// ---------------------------------------------------------------------------

/// `workFilterOptions` reads running in this process. One process-wide
/// counter, separate from every other read's concurrency control.
pub(crate) static WORK_FILTER_OPTIONS_SLOTS: AtomicUsize = AtomicUsize::new(0);

/// One taken slot of a counter, released when dropped, panic unwinding
/// included.
#[derive(Debug)]
pub(crate) struct SlotGuard<'a>(&'a AtomicUsize);

impl Drop for SlotGuard<'_> {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

/// Take a slot of `slots` without waiting, or `None` when all
/// [`WORK_FILTER_OPTIONS_MAX_CONCURRENT_READS`] are taken.
pub(crate) fn try_take_slot(slots: &AtomicUsize) -> Option<SlotGuard<'_>> {
    let mut taken = slots.load(Ordering::SeqCst);
    while taken < WORK_FILTER_OPTIONS_MAX_CONCURRENT_READS {
        match slots.compare_exchange_weak(taken, taken + 1, Ordering::SeqCst, Ordering::SeqCst) {
            Ok(_) => return Some(SlotGuard(slots)),
            Err(current) => taken = current,
        }
    }
    None
}

// ---------------------------------------------------------------------------
// The statement
// ---------------------------------------------------------------------------

/// Every list and the unknown publisher IDs, from one snapshot.
///
/// `$1` is the publisher IDs and `$2` one more than the per-list bound in
/// force ([`WORK_FILTER_OPTIONS_MAX_ENTRIES`] `+ 1` in production). `w` is the
/// work set of `WORKS_SQL` for `$1` with every other selector empty: every
/// work, of any type and status, whose imprint belongs to one of the
/// publishers. Each branch is tagged with its list's GraphQL name; every list
/// branch stops after `$2` rows, so an oversized list is detected without
/// being read in full (the scan itself is bounded by the statement timeout).
/// Every typed placeholder is cast explicitly so the `UNION ALL` column types
/// agree, and `work_type` and `language_code` are returned both as their
/// enums and as their `::text` labels, which order them in Rust.
///
/// The `series` branch matches `WORKS_SQL`'s series selector: a series that
/// issues a work of `w`, or that issues the current `is-child-of` parent (of
/// any publisher) of a book chapter of `w`.
pub(crate) const WORK_FILTER_OPTIONS_SQL: &str = "WITH w AS ( \
         SELECT w.work_id, w.work_type \
         FROM public.work w \
         JOIN public.imprint i ON i.imprint_id = w.imprint_id \
         WHERE i.publisher_id = ANY($1) \
     ) \
     (SELECT 'unknown'::text AS kind, x.id AS id, NULL::text AS name, \
             NULL::public.series_type AS series_type, NULL::text AS issn_print, \
             NULL::text AS issn_digital, NULL::text AS ror, \
             NULL::public.work_type AS work_type, \
             NULL::public.language_code AS language_code, NULL::text AS label \
      FROM unnest($1::uuid[]) AS x(id) \
      WHERE NOT EXISTS (SELECT 1 FROM public.publisher p WHERE p.publisher_id = x.id)) \
     UNION ALL \
     (SELECT 'publishers'::text, p.publisher_id, p.publisher_name, \
             NULL::public.series_type, NULL::text, NULL::text, NULL::text, \
             NULL::public.work_type, NULL::public.language_code, NULL::text \
      FROM public.publisher p \
      WHERE p.publisher_id = ANY($1) \
      LIMIT $2) \
     UNION ALL \
     (SELECT 'series'::text, s.series_id, s.series_name, \
             s.series_type, s.issn_print, s.issn_digital, NULL::text, \
             NULL::public.work_type, NULL::public.language_code, NULL::text \
      FROM public.series s \
      WHERE s.series_id IN ( \
          SELECT iss.series_id \
          FROM w JOIN public.issue iss ON iss.work_id = w.work_id \
          UNION ALL \
          SELECT iss.series_id \
          FROM w \
          JOIN public.work_relation wr ON wr.relator_work_id = w.work_id \
          JOIN public.issue iss ON iss.work_id = wr.related_work_id \
          WHERE w.work_type = 'book-chapter' AND wr.relation_type = 'is-child-of') \
      LIMIT $2) \
     UNION ALL \
     (SELECT 'languageCodes'::text, NULL::uuid, NULL::text, \
             NULL::public.series_type, NULL::text, NULL::text, NULL::text, \
             NULL::public.work_type, c.language_code, c.language_code::text \
      FROM (SELECT DISTINCT l.language_code \
            FROM w JOIN public.language l ON l.work_id = w.work_id) c \
      LIMIT $2) \
     UNION ALL \
     (SELECT 'workTypes'::text, NULL::uuid, NULL::text, \
             NULL::public.series_type, NULL::text, NULL::text, NULL::text, \
             t.work_type, NULL::public.language_code, t.work_type::text \
      FROM (SELECT DISTINCT w.work_type FROM w) t \
      LIMIT $2) \
     UNION ALL \
     (SELECT 'fundingInstitutions'::text, n.institution_id, n.institution_name, \
             NULL::public.series_type, NULL::text, NULL::text, n.ror, \
             NULL::public.work_type, NULL::public.language_code, NULL::text \
      FROM public.institution n \
      WHERE EXISTS (SELECT 1 FROM public.funding f JOIN w ON w.work_id = f.work_id \
                    WHERE f.institution_id = n.institution_id) \
      LIMIT $2) \
     UNION ALL \
     (SELECT 'affiliationInstitutions'::text, n.institution_id, n.institution_name, \
             NULL::public.series_type, NULL::text, NULL::text, n.ror, \
             NULL::public.work_type, NULL::public.language_code, NULL::text \
      FROM public.institution n \
      WHERE EXISTS (SELECT 1 FROM public.affiliation a \
                    JOIN public.contribution c ON c.contribution_id = a.contribution_id \
                    JOIN w ON w.work_id = c.work_id \
                    WHERE a.institution_id = n.institution_id) \
      LIMIT $2)";

/// One tagged row of [`WORK_FILTER_OPTIONS_SQL`].
#[derive(diesel::QueryableByName, Debug)]
pub(crate) struct OptionRow {
    #[diesel(sql_type = Text)]
    pub(crate) kind: String,
    #[diesel(sql_type = Nullable<SqlUuid>)]
    pub(crate) id: Option<Uuid>,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) name: Option<String>,
    #[diesel(sql_type = Nullable<crate::schema::sql_types::SeriesType>)]
    pub(crate) series_type: Option<SeriesType>,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) issn_print: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) issn_digital: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) ror: Option<Ror>,
    #[diesel(sql_type = Nullable<crate::schema::sql_types::WorkType>)]
    pub(crate) work_type: Option<WorkType>,
    #[diesel(sql_type = Nullable<crate::schema::sql_types::LanguageCode>)]
    pub(crate) language_code: Option<LanguageCode>,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) label: Option<String>,
}

// ---------------------------------------------------------------------------
// The read
// ---------------------------------------------------------------------------

/// What a read runs under. Production uses
/// [`WorkFilterReadControls::production`]; tests lower the bound and the
/// timeouts, inject their own slot counter, and may run one extra statement
/// inside the transaction after the two settings and before the main
/// statement.
pub(crate) struct WorkFilterReadControls<'a> {
    /// The most entries one list may hold.
    pub(crate) max_entries: usize,
    /// The statement-timeout setting, run second in the transaction.
    pub(crate) statement_timeout_sql: &'a str,
    /// How long the read waits for a pool connection.
    pub(crate) checkout_timeout: Duration,
    /// The slot counter the read takes a slot of.
    pub(crate) slots: &'a AtomicUsize,
    /// A test-only statement, run after the settings and before the main
    /// statement.
    #[cfg(test)]
    pub(crate) test_statement: Option<&'a str>,
}

impl WorkFilterReadControls<'static> {
    /// The production controls.
    pub(crate) fn production() -> Self {
        WorkFilterReadControls {
            max_entries: WORK_FILTER_OPTIONS_MAX_ENTRIES,
            statement_timeout_sql: WORK_FILTER_OPTIONS_STATEMENT_TIMEOUT_SQL,
            checkout_timeout: WORK_FILTER_OPTIONS_CHECKOUT_TIMEOUT,
            slots: &WORK_FILTER_OPTIONS_SLOTS,
            #[cfg(test)]
            test_statement: None,
        }
    }
}

/// Answer one `workFilterOptions` field.
///
/// `claims` is the request context's claim counter: only the field that
/// claims it first may read; every later one is refused before validation
/// and reads nothing. The caller's identity is never consulted.
pub(crate) fn work_filter_options(
    pool: &PgPool,
    claims: &AtomicUsize,
    publisher_ids: &[Uuid],
) -> Result<WorkFilterOptions, WorkFilterOptionsError> {
    if claims.fetch_add(1, Ordering::SeqCst) > 0 {
        return Err(WorkFilterOptionsError::OnePerRequest);
    }
    work_filter_options_with(pool, publisher_ids, &WorkFilterReadControls::production())
}

/// The read under explicit controls: validate, take a slot, check out a
/// connection, read, then check unknown IDs and bounds and sort. A retryable
/// failure logs one line.
pub(crate) fn work_filter_options_with(
    pool: &PgPool,
    publisher_ids: &[Uuid],
    controls: &WorkFilterReadControls<'_>,
) -> Result<WorkFilterOptions, WorkFilterOptionsError> {
    read(pool, publisher_ids, controls).map_err(|failure| {
        emit(&failure);
        failure.error
    })
}

/// Validate the input before any slot or database access, failing on the
/// first rule broken: empty, then a duplicate, then more than
/// [`WORK_FILTER_OPTIONS_MAX_PUBLISHERS`].
pub(crate) fn validate(publisher_ids: &[Uuid]) -> Result<(), WorkFilterOptionsError> {
    if publisher_ids.is_empty() {
        return Err(WorkFilterOptionsError::Invalid(EMPTY_MESSAGE));
    }
    let mut seen = HashSet::with_capacity(publisher_ids.len());
    if !publisher_ids.iter().all(|id| seen.insert(*id)) {
        return Err(WorkFilterOptionsError::Invalid(DUPLICATE_MESSAGE));
    }
    if publisher_ids.len() > WORK_FILTER_OPTIONS_MAX_PUBLISHERS {
        return Err(WorkFilterOptionsError::Invalid(TOO_MANY_MESSAGE));
    }
    Ok(())
}

fn read(
    pool: &PgPool,
    publisher_ids: &[Uuid],
    controls: &WorkFilterReadControls<'_>,
) -> Result<WorkFilterOptions, Failure> {
    validate(publisher_ids).map_err(Failure::input)?;
    let _slot = try_take_slot(controls.slots)
        .ok_or_else(|| Failure::retryable(WorkFilterOptionsError::Busy, Cause::NoSlot))?;
    let mut connection = pool
        .get_timeout(controls.checkout_timeout)
        .map_err(|_| Failure::retryable(WorkFilterOptionsError::Busy, Cause::Checkout))?;
    let rows = connection
        .build_transaction()
        .read_only()
        .run(|connection| statement(connection, publisher_ids, controls))?;
    assemble(rows, controls.max_entries)
}

/// The read under explicit controls, without logging, returning the
/// [`Failure`] with its operator cause, so tests can check the cause each
/// real failure path attaches.
#[cfg(test)]
pub(crate) fn read_with_cause(
    pool: &PgPool,
    publisher_ids: &[Uuid],
    controls: &WorkFilterReadControls<'_>,
) -> Result<WorkFilterOptions, Failure> {
    read(pool, publisher_ids, controls)
}

/// The two settings, the test-only statement if any, and the statement,
/// inside the transaction.
fn statement(
    connection: &mut PgConnection,
    publisher_ids: &[Uuid],
    controls: &WorkFilterReadControls<'_>,
) -> Result<Vec<OptionRow>, Failure> {
    diesel::sql_query(WORK_FILTER_OPTIONS_CUSTOM_PLANS_SQL).execute(connection)?;
    diesel::sql_query(controls.statement_timeout_sql).execute(connection)?;
    #[cfg(test)]
    if let Some(sql) = controls.test_statement {
        diesel::sql_query(sql).execute(connection)?;
    }
    let limit = i64::try_from(controls.max_entries)
        .unwrap_or(i64::MAX - 1)
        .saturating_add(1);
    Ok(diesel::sql_query(WORK_FILTER_OPTIONS_SQL)
        .bind::<Array<SqlUuid>, _>(publisher_ids)
        .bind::<SqlBigInt, _>(limit)
        .load(connection)?)
}

/// The lists of a read before their bounds are checked and they are sorted.
#[derive(Debug, Default)]
struct Lists {
    unknown: Vec<Uuid>,
    publishers: Vec<WorkFilterPublisher>,
    series: Vec<WorkFilterSeries>,
    language_codes: Vec<(String, LanguageCode)>,
    work_types: Vec<(String, WorkType)>,
    funding_institutions: Vec<WorkFilterInstitution>,
    affiliation_institutions: Vec<WorkFilterInstitution>,
}

/// A row this module cannot interpret: `UNAVAILABLE`, never a dropped row.
fn unexpected() -> Failure {
    Failure::retryable(WorkFilterOptionsError::Unavailable, Cause::Deserialize)
}

fn institution(row: OptionRow) -> Result<WorkFilterInstitution, Failure> {
    Ok(WorkFilterInstitution {
        institution_id: row.id.ok_or_else(unexpected)?,
        institution_name: row.name.ok_or_else(unexpected)?,
        ror: row.ror,
    })
}

fn collect(rows: Vec<OptionRow>) -> Result<Lists, Failure> {
    let mut lists = Lists::default();
    for row in rows {
        match row.kind.as_str() {
            "unknown" => lists.unknown.push(row.id.ok_or_else(unexpected)?),
            "publishers" => lists.publishers.push(WorkFilterPublisher {
                publisher_id: row.id.ok_or_else(unexpected)?,
                publisher_name: row.name.ok_or_else(unexpected)?,
            }),
            "series" => lists.series.push(WorkFilterSeries {
                series_id: row.id.ok_or_else(unexpected)?,
                series_name: row.name.ok_or_else(unexpected)?,
                series_type: row.series_type.ok_or_else(unexpected)?,
                issn_print: row.issn_print,
                issn_digital: row.issn_digital,
            }),
            "languageCodes" => lists.language_codes.push((
                row.label.ok_or_else(unexpected)?,
                row.language_code.ok_or_else(unexpected)?,
            )),
            "workTypes" => lists.work_types.push((
                row.label.ok_or_else(unexpected)?,
                row.work_type.ok_or_else(unexpected)?,
            )),
            "fundingInstitutions" => lists.funding_institutions.push(institution(row)?),
            "affiliationInstitutions" => lists.affiliation_institutions.push(institution(row)?),
            _ => return Err(unexpected()),
        }
    }
    Ok(lists)
}

/// Unknown publisher IDs first, then the first list over `bound` in field
/// order, then every list sorted. Nothing is truncated.
fn assemble(rows: Vec<OptionRow>, bound: usize) -> Result<WorkFilterOptions, Failure> {
    let mut lists = collect(rows)?;
    if !lists.unknown.is_empty() {
        lists.unknown.sort_unstable();
        return Err(Failure::input(WorkFilterOptionsError::UnknownPublisher(
            lists.unknown,
        )));
    }
    for (field, size) in [
        ("publishers", lists.publishers.len()),
        ("series", lists.series.len()),
        ("languageCodes", lists.language_codes.len()),
        ("workTypes", lists.work_types.len()),
        ("fundingInstitutions", lists.funding_institutions.len()),
        (
            "affiliationInstitutions",
            lists.affiliation_institutions.len(),
        ),
    ] {
        if size > bound {
            return Err(Failure::input(WorkFilterOptionsError::LimitExceeded {
                field,
                bound,
            }));
        }
    }
    sort_by_name_then_id(&mut lists.publishers, |p| {
        (p.publisher_name.as_str(), p.publisher_id)
    });
    sort_by_name_then_id(&mut lists.series, |s| (s.series_name.as_str(), s.series_id));
    sort_by_name_then_id(&mut lists.funding_institutions, |n| {
        (n.institution_name.as_str(), n.institution_id)
    });
    sort_by_name_then_id(&mut lists.affiliation_institutions, |n| {
        (n.institution_name.as_str(), n.institution_id)
    });
    Ok(WorkFilterOptions {
        publishers: lists.publishers,
        series: lists.series,
        language_codes: sort_by_label(lists.language_codes),
        work_types: sort_by_label(lists.work_types),
        funding_institutions: lists.funding_institutions,
        affiliation_institutions: lists.affiliation_institutions,
    })
}

/// Sort by name in byte order, then by ID. `Uuid`'s order is its canonical
/// lower-case text order.
pub(crate) fn sort_by_name_then_id<T>(items: &mut [T], key: impl Fn(&T) -> (&str, Uuid)) {
    items.sort_by(|a, b| {
        let (a_name, a_id) = key(a);
        let (b_name, b_id) = key(b);
        a_name
            .as_bytes()
            .cmp(b_name.as_bytes())
            .then_with(|| a_id.cmp(&b_id))
    });
}

/// Order enum values by their database label text in byte order, never by
/// declaration order.
pub(crate) fn sort_by_label<T>(mut labelled: Vec<(String, T)>) -> Vec<T> {
    labelled.sort_by(|(a, _), (b, _)| a.as_bytes().cmp(b.as_bytes()));
    labelled.into_iter().map(|(_, value)| value).collect()
}

#[cfg(test)]
pub(crate) mod tests;
