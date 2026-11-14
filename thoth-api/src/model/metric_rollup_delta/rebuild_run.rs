//! Resumable monthly rebuild and paged verification (`MET-WP4-03A-OPS-03`).
//!
//! A run is one row of `metric_rollup_month_rebuild_run`: a durable cursor
//! over the unit space `(work_id, year_start)`, in ascending `work_id`
//! (PostgreSQL `uuid` order) and then year. The caller drives every step:
//! Thoth runs nothing on its own, and holds no worker, queue, lease,
//! scheduler or timer.
//!
//! * A **build step** (`BUILDING`, `READ COMMITTED`) sizes the next range of
//!   units without any lock, locks the range's works `FOR KEY SHARE` in
//!   ascending order and then the singleton state row, re-sizes under those
//!   locks admitting only locked works, verifies the range independently,
//!   and, only if it is not exact, replaces exactly that range with the
//!   completion's own keyed statements and verifies it again before commit.
//!   It never truncates, never writes a work-day row, delta, state row,
//!   canonical row or reconciliation ledger, and never moves the frontier.
//! * A **verify page** (`VERIFYING`, `REPEATABLE READ`) takes `ACCESS SHARE`
//!   on the six projection tables, the work-day projection and the state
//!   row before its first query, verifies the next range in one snapshot and
//!   writes only the run row.
//!
//! Every statement inside a step or a page runs under a per-statement
//! deadline: `SET LOCAL statement_timeout = <ms>` immediately before it,
//! where `<ms>` is the smaller of the statement ceiling and the time left in
//! the current budget, never caller input, and a check between statements
//! refuses the next one once the budget is spent. Every database error is
//! classified from the raw `diesel::result::Error` before any conversion
//! (section 11 of the specification), and the first classified error of a
//! step survives a failed rollback.

use std::collections::BTreeSet;
use std::time::{Duration, Instant};

use chrono::{Datelike, NaiveDate};
use diesel::dsl::sql;
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::sql_types::{Array, BigInt, Date, Nullable, Text, Timestamptz, Uuid as SqlUuid};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use juniper::{graphql_value, FieldError, IntoFieldError};
use uuid::Uuid;

use super::crud::{
    affected_year_keys, MonthKey, YearKey, MONTH_INSERT_STATEMENTS, MONTH_SOURCE_WATERMARK_SQL,
    YEAR_COUNTRY_INSERT_SQL, YEAR_INSTITUTION_INSERT_SQL,
};
use super::verification::{verify_range, RangeBounds, RangeVerification};
use super::{
    AbandonMetricRollupMonthRebuildInput, AdvanceMetricRollupMonthRebuildInput,
    MetricRollupMonthProjectionVerification, MetricRollupMonthRebuildAdvance,
    MetricRollupMonthRebuildBuildProgress, MetricRollupMonthRebuildFamilies,
    MetricRollupMonthRebuildMode, MetricRollupMonthRebuildRun, MetricRollupMonthRebuildStatus,
    MetricRollupMonthRebuildVerifyProgress, StartMetricRollupMonthRebuildInput,
    METRIC_ROLLUP_RUN_CHECKOUT_TIMEOUT, METRIC_ROLLUP_RUN_PAGE_BUDGET,
    METRIC_ROLLUP_RUN_PAGE_MAX_MONTH_KEYS, METRIC_ROLLUP_RUN_PAGE_MAX_WORK_DAY_ROWS,
    METRIC_ROLLUP_RUN_SIZING_BATCH_WORKS, METRIC_ROLLUP_RUN_STATEMENT_TIMEOUT_CEILING,
    METRIC_ROLLUP_RUN_STEP_CHUNK_KEYS, METRIC_ROLLUP_RUN_STEP_HOLD_BUDGET,
    METRIC_ROLLUP_RUN_STEP_LOCK_BUDGET, METRIC_ROLLUP_RUN_STEP_MAX_MONTH_KEYS,
    METRIC_ROLLUP_RUN_STEP_MAX_REPLACING_READ, METRIC_ROLLUP_RUN_STEP_MAX_WORK_DAY_ROWS,
    METRIC_ROLLUP_RUN_STEP_PRESIZE_BUDGET,
};
use crate::db::PgPool;
use crate::model::Timestamp;
use crate::schema::metric_rollup_month_rebuild_run as run_table;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// The fixed reason ids of refusals, for server logs only.
pub(crate) mod reason {
    pub(crate) const SERIALIZATION_FAILURE: &str = "serialization_failure";
    pub(crate) const CONNECTION_CLOSED: &str = "connection_closed";
    pub(crate) const LOCK_TIMEOUT: &str = "lock_timeout";
    pub(crate) const STATEMENT_TIMEOUT: &str = "statement_timeout";
    pub(crate) const DEADLOCK: &str = "deadlock";
    pub(crate) const IDLE_TIMEOUT: &str = "idle_timeout";
    pub(crate) const BUDGET: &str = "budget";
    pub(crate) const POOL_CHECKOUT: &str = "pool_checkout";
    pub(crate) const EMPTY_RESIZE: &str = "empty_resize";
    pub(crate) const BIGINT_OVERFLOW: &str = "bigint_overflow";
    pub(crate) const WORK_DAY_ABOVE_FRONTIER: &str = "work_day_above_frontier";
    pub(crate) const SOURCE_WATERMARK_ABOVE_FRONTIER: &str = "source_watermark_above_frontier";
    pub(crate) const DELETE_COUNT_MISMATCH: &str = "delete_count_mismatch";
    pub(crate) const POST_VERIFY_INEXACT: &str = "post_verify_inexact";
    pub(crate) const DATABASE: &str = "database";
    pub(crate) const ISOLATION: &str = "isolation";
    pub(crate) const INVARIANT: &str = "invariant";
}

/// PostgreSQL's fixed server messages the classifier matches exactly. They
/// assume English server messages (`lc_messages`), which the tests assert
/// on the CI server and every run grant checks on its environment.
pub(crate) mod server_message {
    pub(crate) const LOCK_TIMEOUT: &str = "canceling statement due to lock timeout";
    pub(crate) const STATEMENT_TIMEOUT: &str = "canceling statement due to statement timeout";
    pub(crate) const DEADLOCK: &str = "deadlock detected";
    pub(crate) const IDLE_TIMEOUT: &str =
        "terminating connection due to idle-in-transaction timeout";
    pub(crate) const BIGINT_OUT_OF_RANGE: &str = "bigint out of range";
}

/// The partial unique index that admits at most one open run.
pub(crate) const ONE_OPEN_RUN_INDEX: &str = "metric_rollup_month_rebuild_run_one_open_idx";

/// The local error boundary of the run operations: fixed messages and a
/// stable `extensions.type`.
///
/// No message carries a frontier, row, SQL, principal, cursor or
/// connection detail. The reason a refusal carries is a fixed id for the
/// server log line only; it never reaches GraphQL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricRollupMonthRebuildError {
    /// A malformed `expectedStepCount`, or an unknown `runId`:
    /// `METRIC_QUERY_INVALID`.
    QueryInvalid,
    /// `start` while a run is open: `METRIC_REBUILD_RUN_OPEN`.
    RunOpen,
    /// Nothing was committed and a later retry may succeed:
    /// `METRIC_REBUILD_RETRYABLE`.
    Retryable(&'static str),
    /// Nothing was committed, and the refusal will repeat until a separately
    /// authorized repair: `METRIC_REBUILD_BLOCKED`.
    Blocked(&'static str),
    /// Anything unclassified: `INTERNAL_ERROR`.
    Internal(&'static str),
}

impl MetricRollupMonthRebuildError {
    /// The stable machine-readable code, returned as `extensions.type`.
    pub fn code(self) -> &'static str {
        match self {
            Self::QueryInvalid => "METRIC_QUERY_INVALID",
            Self::RunOpen => "METRIC_REBUILD_RUN_OPEN",
            Self::Retryable(_) => "METRIC_REBUILD_RETRYABLE",
            Self::Blocked(_) => "METRIC_REBUILD_BLOCKED",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }

    /// The fixed client-facing message. Never derived from input or state.
    pub fn message(self) -> &'static str {
        match self {
            Self::QueryInvalid => {
                "The monthly rebuild run request is invalid: the run does not exist, or \
                 expectedStepCount is not a decimal string holding a non-negative 64-bit \
                 integer. Nothing was changed."
            }
            Self::RunOpen => {
                "A monthly rebuild run is already open. Adopt it, let it finish or abandon it."
            }
            Self::Retryable(_) => {
                "The monthly rebuild run could not advance now. Nothing was committed; back \
                 off and retry with the same expected step count."
            }
            Self::Blocked(_) => {
                "The monthly rebuild run is blocked: the derived or work-day state needs a \
                 separately authorized repair. Nothing was committed."
            }
            Self::Internal(_) => "The monthly rebuild run operation could not be completed.",
        }
    }

    /// The fixed reason id, for server logs and tests.
    pub fn reason(self) -> &'static str {
        match self {
            Self::QueryInvalid => "query_invalid",
            Self::RunOpen => "run_open",
            Self::Retryable(reason) | Self::Blocked(reason) | Self::Internal(reason) => reason,
        }
    }

    /// The refusal class of the log line, or `None` for a request error.
    fn refused(self) -> Option<&'static str> {
        match self {
            Self::Retryable(_) => Some("RETRYABLE"),
            Self::Blocked(_) => Some("BLOCKED"),
            Self::Internal(_) => Some("INTERNAL"),
            Self::QueryInvalid | Self::RunOpen => None,
        }
    }
}

impl std::fmt::Display for MetricRollupMonthRebuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.code())
    }
}

impl std::error::Error for MetricRollupMonthRebuildError {}

/// Every database error reaching the boundary is classified, never passed
/// through: see [`classify`].
impl From<DieselError> for MetricRollupMonthRebuildError {
    fn from(error: DieselError) -> Self {
        classify(&error)
    }
}

impl IntoFieldError for MetricRollupMonthRebuildError {
    fn into_field_error(self) -> FieldError {
        let code = self.code();
        FieldError::new(self.message(), graphql_value!({ "type": code }))
    }
}

/// Classify one raw database error (section 11).
///
/// * a serialization failure (SQLSTATE `40001`) or a closed connection is
///   retryable;
/// * a unique violation on [`ONE_OPEN_RUN_INDEX`] is `RUN_OPEN`;
/// * the server's exact lock-timeout, statement-timeout, deadlock and
///   idle-in-transaction-timeout messages are retryable;
/// * `bigint out of range`, from any statement of a step or page, is
///   blocked;
/// * anything else is internal, and is logged server-side.
///
/// A misclassification therefore fails safe as a HOLD and never causes a
/// write.
pub(crate) fn classify(error: &DieselError) -> MetricRollupMonthRebuildError {
    use MetricRollupMonthRebuildError::{Blocked, Internal, Retryable, RunOpen};
    let DieselError::DatabaseError(kind, info) = error else {
        log::error!("metric_rollup_rebuild unclassified failure: {error}");
        return Internal(reason::DATABASE);
    };
    match kind {
        DatabaseErrorKind::SerializationFailure => return Retryable(reason::SERIALIZATION_FAILURE),
        DatabaseErrorKind::ClosedConnection => return Retryable(reason::CONNECTION_CLOSED),
        DatabaseErrorKind::UniqueViolation
            if info.constraint_name() == Some(ONE_OPEN_RUN_INDEX) =>
        {
            return RunOpen
        }
        _ => {}
    }
    match info.message() {
        server_message::LOCK_TIMEOUT => Retryable(reason::LOCK_TIMEOUT),
        server_message::STATEMENT_TIMEOUT => Retryable(reason::STATEMENT_TIMEOUT),
        server_message::DEADLOCK => Retryable(reason::DEADLOCK),
        server_message::IDLE_TIMEOUT => Retryable(reason::IDLE_TIMEOUT),
        server_message::BIGINT_OUT_OF_RANGE => Blocked(reason::BIGINT_OVERFLOW),
        _ => {
            log::error!("metric_rollup_rebuild unclassified database failure: {error}");
            Internal(reason::DATABASE)
        }
    }
}

fn invariant() -> MetricRollupMonthRebuildError {
    MetricRollupMonthRebuildError::Internal(reason::INVARIANT)
}

fn overflow() -> MetricRollupMonthRebuildError {
    MetricRollupMonthRebuildError::Blocked(reason::BIGINT_OVERFLOW)
}

/// `a + b`, or the blocked overflow refusal.
fn add(a: i64, b: i64) -> Result<i64, MetricRollupMonthRebuildError> {
    a.checked_add(b).ok_or_else(overflow)
}

// ---------------------------------------------------------------------------
// Budgets and the per-statement deadline
// ---------------------------------------------------------------------------

/// The step and page budgets of section 10.
///
/// Production always uses [`RunBudgets::PRODUCTION`]; the `_within`
/// variants that accept another value are crate-private and exist for tests
/// only, following the OPS-02 `_within` pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RunBudgets {
    pub(crate) step_presize: Duration,
    pub(crate) step_lock: Duration,
    pub(crate) step_hold: Duration,
    pub(crate) step_max_month_keys: i64,
    pub(crate) step_max_work_day_rows: i64,
    pub(crate) step_max_replacing_read: i64,
    pub(crate) page: Duration,
    pub(crate) page_max_month_keys: i64,
    pub(crate) page_max_work_day_rows: i64,
}

impl RunBudgets {
    /// The server-fixed budgets.
    pub(crate) const PRODUCTION: Self = Self {
        step_presize: METRIC_ROLLUP_RUN_STEP_PRESIZE_BUDGET,
        step_lock: METRIC_ROLLUP_RUN_STEP_LOCK_BUDGET,
        step_hold: METRIC_ROLLUP_RUN_STEP_HOLD_BUDGET,
        step_max_month_keys: METRIC_ROLLUP_RUN_STEP_MAX_MONTH_KEYS,
        step_max_work_day_rows: METRIC_ROLLUP_RUN_STEP_MAX_WORK_DAY_ROWS,
        step_max_replacing_read: METRIC_ROLLUP_RUN_STEP_MAX_REPLACING_READ,
        page: METRIC_ROLLUP_RUN_PAGE_BUDGET,
        page_max_month_keys: METRIC_ROLLUP_RUN_PAGE_MAX_MONTH_KEYS,
        page_max_work_day_rows: METRIC_ROLLUP_RUN_PAGE_MAX_WORK_DAY_ROWS,
    };

    pub(crate) fn step_size(&self) -> SizeBudgets {
        SizeBudgets {
            work_day_rows: self.step_max_work_day_rows,
            month_keys: self.step_max_month_keys,
            replacing_read: self.step_max_replacing_read,
        }
    }

    /// A page never replaces, so its replacing-read budget is unbounded.
    pub(crate) fn page_size(&self) -> SizeBudgets {
        SizeBudgets {
            work_day_rows: self.page_max_work_day_rows,
            month_keys: self.page_max_month_keys,
            replacing_read: i64::MAX,
        }
    }
}

/// The three size budgets the sizing loop fills.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SizeBudgets {
    pub(crate) work_day_rows: i64,
    pub(crate) month_keys: i64,
    pub(crate) replacing_read: i64,
}

/// One budget clock: the pre-size, lock or hold budget of a step, or the
/// budget of a page.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Clock {
    started: Instant,
    budget: Duration,
}

impl Clock {
    pub(crate) fn start(budget: Duration) -> Self {
        Self {
            started: Instant::now(),
            budget,
        }
    }

    /// The milliseconds the next statement may run: the smaller of the
    /// statement ceiling and the time left in this budget, with a floor of
    /// one, or the budget refusal once the budget is spent.
    fn deadline_ms(&self) -> Result<u128, MetricRollupMonthRebuildError> {
        let remaining = self
            .budget
            .checked_sub(self.started.elapsed())
            .filter(|remaining| !remaining.is_zero())
            .ok_or(MetricRollupMonthRebuildError::Retryable(reason::BUDGET))?;
        Ok(remaining
            .as_millis()
            .min(METRIC_ROLLUP_RUN_STATEMENT_TIMEOUT_CEILING.as_millis())
            .max(1))
    }
}

/// The phase a sizing, run-row or update statement belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Stage {
    /// A build step's pre-sizing, before any row lock.
    Presize,
    /// A build step's state-row hold.
    Hold,
    /// A verify page.
    Page,
}

/// Where a statement of a step or a page runs. It names the statement in
/// tests (failure and delay injection) and nowhere else.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Site {
    Isolation(Stage),
    RunRead,
    Sizing(Stage),
    PerYear(Stage),
    LockWorks,
    LockState,
    RunLock(Stage),
    PreVerify,
    RangeDelete(usize),
    MonthKeys,
    MonthWatermark(usize),
    MonthInsert(usize, usize),
    YearInsert(usize, usize),
    PostVerify,
    RunUpdate(Stage),
    PageLock,
    PageFrontier,
    PageVerify,
}

/// Test-only injection of a statement or a client-side pause before a
/// named statement. Production builds compile no hook.
#[cfg(test)]
pub(crate) mod hooks {
    use std::cell::RefCell;
    use std::time::Duration;

    use super::Site;

    /// What runs immediately before the statement at a site.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub(crate) enum Injection {
        /// A statement executed on the step's connection under its own
        /// deadline, exactly like one of the step's statements.
        Statement(String),
        /// The calling thread sleeps, as a stalled API process would.
        ClientPause(Duration),
    }

    thread_local! {
        static INJECTIONS: RefCell<Vec<(Site, Injection)>> = const { RefCell::new(Vec::new()) };
    }

    /// Arm a one-shot injection at `site` on the calling thread.
    pub(crate) fn inject(site: Site, injection: Injection) {
        INJECTIONS.with(|injections| injections.borrow_mut().push((site, injection)));
    }

    /// Disarm every injection of the calling thread.
    pub(crate) fn clear() {
        INJECTIONS.with(|injections| injections.borrow_mut().clear());
    }

    /// Take the first injection armed at `site`, if any.
    pub(super) fn take(site: Site) -> Option<Injection> {
        INJECTIONS.with(|injections| {
            let mut injections = injections.borrow_mut();
            let position = injections.iter().position(|(armed, _)| *armed == site)?;
            Some(injections.remove(position).1)
        })
    }
}

/// Run one statement of a step or a page under the current budget: refuse
/// it once the budget is spent, otherwise set its deadline with `SET LOCAL
/// statement_timeout` immediately before it, and classify its raw error.
fn run_statement<T>(
    connection: &mut PgConnection,
    clock: &Clock,
    site: Site,
    statement: impl FnOnce(&mut PgConnection) -> Result<T, DieselError>,
) -> Result<T, MetricRollupMonthRebuildError> {
    #[cfg(test)]
    if let Some(injection) = hooks::take(site) {
        match injection {
            hooks::Injection::ClientPause(pause) => std::thread::sleep(pause),
            hooks::Injection::Statement(text) => {
                set_deadline(connection, clock)?;
                diesel::sql_query(text)
                    .execute(connection)
                    .map_err(|error| classify(&error))?;
            }
        }
    }
    #[cfg(not(test))]
    let _ = site;
    set_deadline(connection, clock)?;
    statement(connection).map_err(|error| classify(&error))
}

/// Set the next statement's deadline, or refuse once the budget is spent.
fn set_deadline(
    connection: &mut PgConnection,
    clock: &Clock,
) -> Result<(), MetricRollupMonthRebuildError> {
    let deadline = clock.deadline_ms()?;
    diesel::sql_query(format!("SET LOCAL statement_timeout = {deadline}"))
        .execute(connection)
        .map_err(|error| classify(&error))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Statements
// ---------------------------------------------------------------------------

/// The session settings of a build step, before its first budgeted
/// statement. `plan_cache_mode` plans each statement for its actual bounds
/// and arrays, as register 4.2 does for reads.
pub(crate) const STEP_SESSION_SQL: [&str; 4] = [
    "SET LOCAL lock_timeout = '5s'",
    "SET LOCAL statement_timeout = 30000",
    "SET LOCAL idle_in_transaction_session_timeout = '2s'",
    "SET LOCAL plan_cache_mode = force_custom_plan",
];

/// The session settings of a verify page after its initial statement
/// timeout (the page budget in milliseconds), before `LOCK TABLE`.
pub(crate) const PAGE_SESSION_SQL: [&str; 3] = [
    "SET LOCAL lock_timeout = '5s'",
    "SET LOCAL idle_in_transaction_session_timeout = '2s'",
    "SET LOCAL plan_cache_mode = force_custom_plan",
];

/// The settings of `start` and `abandon`.
const WRITE_SESSION_SQL: [&str; 2] = [
    "SET LOCAL lock_timeout = '5s'",
    "SET LOCAL statement_timeout = 30000",
];

/// The settings of the dispatch read and the status read.
const READ_SESSION_SQL: [&str; 2] = [
    "SET LOCAL lock_timeout = '1s'",
    "SET LOCAL statement_timeout = 1000",
];

/// The isolation level the server reports for this transaction.
pub(crate) const ISOLATION_SQL: &str =
    "SELECT current_setting('transaction_isolation') AS isolation";

/// One run row, without a lock.
pub(crate) const RUN_READ_SQL: &str =
    "SELECT * FROM public.metric_rollup_month_rebuild_run WHERE run_id = $1";

/// One run row, locked.
pub(crate) const RUN_LOCK_SQL: &str =
    "SELECT * FROM public.metric_rollup_month_rebuild_run WHERE run_id = $1 FOR UPDATE";

/// The open run when one exists (the one-open-run index makes it unique),
/// otherwise the run with the greatest `started_at`, ties broken by
/// `run_id`.
pub(crate) const LATEST_RUN_SQL: &str = "SELECT * FROM public.metric_rollup_month_rebuild_run \
     ORDER BY (status IN ('BUILDING', 'VERIFYING')) DESC, started_at DESC, run_id DESC \
     LIMIT 1";

/// Insert a run at the current frontier, read without a lock. `started_at`
/// is set here, by the insert.
pub(crate) const START_SQL: &str = "INSERT INTO public.metric_rollup_month_rebuild_run \
         (mode, status, started_by, started_at, updated_at, start_applied_through_sequence) \
     SELECT $1, $2, $3, statement_timestamp(), statement_timestamp(), s.applied_through_sequence \
     FROM public.metric_rollup_work_day_state s \
     WHERE s.state_id = 1 \
     RETURNING *";

/// Lock the works of a step's candidate range `FOR KEY SHARE`, in ascending
/// `work_id` order (PostgreSQL locks rows after the sort): canonical
/// ingestion's own order, works before the state row.
pub(crate) const LOCK_WORKS_SQL: &str = "SELECT work_id FROM public.work \
     WHERE work_id = ANY($1) ORDER BY work_id FOR KEY SHARE";

/// Lock the singleton state row and read the frontier `W`.
pub(crate) const LOCK_STATE_SQL: &str = "SELECT applied_through_sequence \
     FROM public.metric_rollup_work_day_state WHERE state_id = 1 FOR UPDATE";

/// Read the frontier `W` in a page's snapshot, without a lock.
pub(crate) const PAGE_FRONTIER_SQL: &str = "SELECT applied_through_sequence \
     FROM public.metric_rollup_work_day_state WHERE state_id = 1";

/// A page's one table-level lock, before its first query: the six
/// projection tables in the OPS-02 order, then the work-day projection and
/// the state row's table, all `ACCESS SHARE`. A concurrent OPS-02
/// `TRUNCATE`, or a migration rewriting one of these tables, therefore
/// cannot cross the page's snapshot.
pub(crate) const PAGE_LOCK_SQL: &str = "LOCK TABLE \
         public.metric_rollup_work_month, \
         public.metric_rollup_work_country_month, \
         public.metric_rollup_work_institution_month, \
         public.metric_rollup_work_month_ambiguity, \
         public.metric_rollup_work_country_year, \
         public.metric_rollup_work_institution_year, \
         public.metric_rollup_work_day, \
         public.metric_rollup_work_day_state \
     IN ACCESS SHARE MODE";

/// The writer-side range predicate on `column` of a table keyed by
/// `work_id`: `$1` and `$2` are the lower bound's `work_id` and the 31
/// December of its year (`NULL` for minus infinity, with the nil uuid), `$3`
/// and `$4` the upper bound's (`NULL` for plus infinity, with the greatest
/// uuid). It is the `(work_id, day)` index range of section 4 on the
/// work-day projection, and explicit `work_id` bounds plus the row's own
/// year on a projection table, so orphan rows are covered.
macro_rules! writer_range_predicate_sql {
    ($column:literal) => {
        concat!(
            "WHERE work_id >= $1 AND work_id <= $3 \
               AND (work_id, ",
            $column,
            ") > ($1, COALESCE($2, '-infinity'::date)) \
               AND (work_id, ",
            $column,
            ") <= ($3, COALESCE($4, 'infinity'::date))"
        )
    };
}

/// The six range deletes, one per family, in the OPS-02 order. Each
/// family's deleted count must equal the pre-verify's actual row count.
pub(crate) const RANGE_DELETE_STATEMENTS: [&str; 6] = [
    concat!(
        "DELETE FROM public.metric_rollup_work_month ",
        writer_range_predicate_sql!("month_start")
    ),
    concat!(
        "DELETE FROM public.metric_rollup_work_country_month ",
        writer_range_predicate_sql!("month_start")
    ),
    concat!(
        "DELETE FROM public.metric_rollup_work_institution_month ",
        writer_range_predicate_sql!("month_start")
    ),
    concat!(
        "DELETE FROM public.metric_rollup_work_month_ambiguity ",
        writer_range_predicate_sql!("month_start")
    ),
    concat!(
        "DELETE FROM public.metric_rollup_work_country_year ",
        writer_range_predicate_sql!("year_start")
    ),
    concat!(
        "DELETE FROM public.metric_rollup_work_institution_year ",
        writer_range_predicate_sql!("year_start")
    ),
];

/// Every month key represented in a range's work-day rows, in the order the
/// replacing step recomputes them: the range-bounded form of the OPS-02
/// key listing, reading work-day rows only through the `(work_id, day)`
/// index range.
pub(crate) const RANGE_MONTH_KEYS_SQL: &str = concat!(
    "SELECT DISTINCT work_id, platform_id, measure_id, \
            date_trunc('month', day)::date AS month_start \
     FROM public.metric_rollup_work_day ",
    writer_range_predicate_sql!("day"),
    " ORDER BY 1, 2, 3, 4"
);

/// The per-(series, month) groups of one work's work-day rows after the
/// cursor, with each group's month total across every series of the work
/// (`month_rows`). `$1` and `$2` are the cursor; the predicate is the
/// `(work_id, day)` index range, so a work after the cursor's work is read
/// whole.
macro_rules! series_months_sql {
    ($work:literal) => {
        concat!(
            "SELECT s.platform_id, s.measure_id, s.year_start, s.rows, \
                    SUM(s.rows) OVER (PARTITION BY s.month_start) AS month_rows \
             FROM ( \
                 SELECT r.platform_id, r.measure_id, \
                        date_trunc('month', r.day)::date AS month_start, \
                        date_trunc('year', r.day)::date AS year_start, \
                        count(*) AS rows \
                 FROM public.metric_rollup_work_day r \
                 WHERE r.work_id = ",
            $work,
            " AND (r.work_id, r.day) > ($1, COALESCE($2, '-infinity'::date)) \
                 GROUP BY r.platform_id, r.measure_id, date_trunc('month', r.day), \
                          date_trunc('year', r.day) \
             ) s"
        )
    };
}

/// One work's sizing counts: from the cursor, its units, work-day rows,
/// month keys, the month term of the replacing-read figure and its year
/// keys, in one grouped pass over its index range; and its rows before the
/// cursor, a plain count the planner may answer from the index alone, which
/// completes the whole-work total the year term needs.
macro_rules! work_counts_sql {
    ($work:literal) => {
        concat!(
            "SELECT count(DISTINCT g.year_start) AS units, \
                    COALESCE(SUM(g.rows), 0)::bigint AS unit_rows, \
                    count(*) AS month_keys, \
                    COALESCE(SUM(g.month_rows), 0)::bigint AS month_term, \
                    count(DISTINCT (g.platform_id, g.measure_id, g.year_start)) AS year_keys, \
                    MIN(g.year_start) AS first_year, \
                    MAX(g.year_start) AS last_year, \
                    (SELECT count(*) FROM public.metric_rollup_work_day b \
                     WHERE b.work_id = ",
            $work,
            " AND b.work_id = $1 \
                       AND b.day <= COALESCE($2, '-infinity'::date)) AS before_rows \
             FROM (",
            series_months_sql!($work),
            ") g"
        )
    };
}

/// One sizing statement (section 10): a recursive walk forward from the
/// cursor, work by work. Each iteration finds the next `work_id` with
/// `ORDER BY work_id LIMIT 1` over the `(work_id, day)` index (a skip scan)
/// and counts that work alone. The running totals are carried in the
/// recursive row; the walk stops after the first work that fills a budget
/// (`$4` rows, `$5` month keys, `$6` replacing-read figure, each the budget
/// still left) or after `$3` works, and reads the works it returns and
/// nothing else.
pub(crate) const SIZING_SQL: &str = concat!(
    "WITH RECURSIVE walk AS ( \
         SELECT 1::bigint AS n, f.work_id, c.units, c.unit_rows, c.month_keys, \
                c.month_term + c.year_keys * (c.unit_rows + c.before_rows) AS figure, \
                c.unit_rows + c.before_rows AS whole_rows, c.first_year, c.last_year, \
                c.unit_rows AS cum_rows, c.month_keys AS cum_keys, \
                c.month_term + c.year_keys * (c.unit_rows + c.before_rows) AS cum_figure \
         FROM ( \
             SELECT r.work_id FROM public.metric_rollup_work_day r \
             WHERE (r.work_id, r.day) > ($1, COALESCE($2, '-infinity'::date)) \
             ORDER BY r.work_id, r.day \
             LIMIT 1 \
         ) f \
         CROSS JOIN LATERAL (",
    work_counts_sql!("f.work_id"),
    ") c \
         UNION ALL \
         SELECT w.n + 1, x.work_id, c.units, c.unit_rows, c.month_keys, \
                c.month_term + c.year_keys * (c.unit_rows + c.before_rows), \
                c.unit_rows + c.before_rows, c.first_year, c.last_year, \
                w.cum_rows + c.unit_rows, w.cum_keys + c.month_keys, \
                w.cum_figure + c.month_term + c.year_keys * (c.unit_rows + c.before_rows) \
         FROM walk w \
         CROSS JOIN LATERAL ( \
             SELECT r.work_id FROM public.metric_rollup_work_day r \
             WHERE r.work_id > w.work_id \
             ORDER BY r.work_id \
             LIMIT 1 \
         ) x \
         CROSS JOIN LATERAL (",
    work_counts_sql!("x.work_id"),
    ") c \
         WHERE w.n < $3 AND w.cum_rows < $4 AND w.cum_keys < $5 AND w.cum_figure < $6 \
     ) \
     SELECT n, work_id, units, unit_rows, month_keys, figure, whole_rows, first_year, last_year \
     FROM walk \
     ORDER BY n"
);

/// The per-year counts of one work (`$3`) from the cursor, for a work that
/// alone exceeds a budget. Each year's year term uses the work's
/// whole-history row count (`$4`) from the sizing statement.
pub(crate) const PER_YEAR_SQL: &str = concat!(
    "SELECT g.year_start, \
            SUM(g.rows)::bigint AS unit_rows, \
            count(*) AS month_keys, \
            (SUM(g.month_rows) + count(DISTINCT (g.platform_id, g.measure_id)) * $4)::bigint \
                AS figure \
     FROM (",
    series_months_sql!("$3"),
    ") g \
     GROUP BY g.year_start \
     ORDER BY g.year_start"
);

/// The run-grant aggregate of section 15: a read-only, counts-only full
/// scan of `metric_rollup_work_day`, run once under each `REBUILD` run
/// grant through a read-only database session, never by Thoth.
///
/// Taking each unit alone, it returns the units that exceed a step budget,
/// each with its work-day rows, month keys and replacing-read figure, and
/// two maxima on every row: the largest month-key count per work-month and
/// the largest whole-work row total. With no unit over a budget it returns
/// one row holding the maxima and `NULL` unit columns. The step budgets are
/// bind parameters: `$1` [`METRIC_ROLLUP_RUN_STEP_MAX_WORK_DAY_ROWS`], `$2`
/// [`METRIC_ROLLUP_RUN_STEP_MAX_MONTH_KEYS`] and `$3`
/// [`METRIC_ROLLUP_RUN_STEP_MAX_REPLACING_READ`]. Register section 3.8
/// quotes this text and the bound values, and a test compares both.
pub const METRIC_ROLLUP_RUN_GRANT_AGGREGATE_SQL: &str = r"WITH series_month AS (
    SELECT r.work_id, r.platform_id, r.measure_id,
           date_trunc('month', r.day)::date AS month_start,
           count(*) AS rows
    FROM public.metric_rollup_work_day r
    GROUP BY r.work_id, r.platform_id, r.measure_id, date_trunc('month', r.day)
),
work_month AS (
    SELECT work_id, month_start,
           count(*) AS month_keys,
           SUM(rows) AS rows
    FROM series_month
    GROUP BY work_id, month_start
),
work_year AS (
    SELECT work_id, date_trunc('year', month_start)::date AS year_start,
           SUM(rows) AS rows,
           SUM(month_keys) AS month_keys,
           SUM(month_keys * rows) AS month_term
    FROM work_month
    GROUP BY work_id, date_trunc('year', month_start)
),
year_keys AS (
    SELECT work_id, date_trunc('year', month_start)::date AS year_start,
           count(DISTINCT (platform_id, measure_id)) AS year_keys
    FROM series_month
    GROUP BY work_id, date_trunc('year', month_start)
),
whole_work AS (
    SELECT work_id, SUM(rows) AS rows
    FROM work_year
    GROUP BY work_id
),
sized AS (
    SELECT y.work_id, y.year_start,
           y.rows::bigint AS work_day_rows,
           y.month_keys::bigint AS month_keys,
           (y.month_term + k.year_keys * w.rows)::bigint AS replacing_read_figure
    FROM work_year y
    JOIN year_keys k ON k.work_id = y.work_id AND k.year_start = y.year_start
    JOIN whole_work w ON w.work_id = y.work_id
),
maxima AS (
    SELECT (SELECT COALESCE(MAX(month_keys), 0) FROM work_month)::bigint
               AS max_month_keys_per_work_month,
           (SELECT COALESCE(MAX(rows), 0) FROM whole_work)::bigint
               AS max_whole_work_rows
)
SELECT s.work_id, s.year_start, s.work_day_rows, s.month_keys, s.replacing_read_figure,
       m.max_month_keys_per_work_month, m.max_whole_work_rows
FROM maxima m
LEFT JOIN sized s
  ON s.work_day_rows > $1 OR s.month_keys > $2 OR s.replacing_read_figure > $3
ORDER BY s.work_id, s.year_start";

/// `statement_timestamp()`: one value for every timestamp a statement
/// writes, from the database clock.
fn statement_time() -> diesel::expression::SqlLiteral<Timestamptz> {
    sql::<Timestamptz>("statement_timestamp()")
}

fn statement_time_nullable() -> diesel::expression::SqlLiteral<Nullable<Timestamptz>> {
    sql::<Nullable<Timestamptz>>("statement_timestamp()")
}

// ---------------------------------------------------------------------------
// Rows
// ---------------------------------------------------------------------------

/// One run row, exactly as stored. Never returned to GraphQL: it carries the
/// cursor and the principal.
#[derive(
    Debug, Clone, PartialEq, Eq, diesel::Queryable, diesel::QueryableByName, diesel::Selectable,
)]
#[diesel(table_name = run_table)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct RunRow {
    pub(crate) run_id: Uuid,
    pub(crate) mode: String,
    pub(crate) status: String,
    pub(crate) step_count: i64,
    pub(crate) cursor_work_id: Option<Uuid>,
    pub(crate) cursor_year_start: Option<NaiveDate>,
    pub(crate) started_by: String,
    pub(crate) started_at: Timestamp,
    pub(crate) updated_at: Timestamp,
    pub(crate) build_completed_at: Option<Timestamp>,
    pub(crate) completed_at: Option<Timestamp>,
    pub(crate) start_applied_through_sequence: i64,
    pub(crate) build_steps: i64,
    pub(crate) build_replaced_steps: i64,
    pub(crate) build_units: i64,
    pub(crate) build_month_keys: i64,
    pub(crate) build_work_day_rows: i64,
    pub(crate) build_found_missing_rows: i64,
    pub(crate) build_found_extra_rows: i64,
    pub(crate) build_found_mismatched_rows: i64,
    pub(crate) build_first_frontier: Option<i64>,
    pub(crate) build_last_frontier: Option<i64>,
    pub(crate) verify_pages: i64,
    pub(crate) verify_units: i64,
    pub(crate) verify_month_keys: i64,
    pub(crate) verify_work_day_rows: i64,
    pub(crate) verify_first_frontier: Option<i64>,
    pub(crate) verify_last_frontier: Option<i64>,
    pub(crate) verify_total_expected_rows: i64,
    pub(crate) verify_total_actual_rows: i64,
    pub(crate) verify_total_missing_rows: i64,
    pub(crate) verify_total_extra_rows: i64,
    pub(crate) verify_total_mismatched_rows: i64,
    pub(crate) verify_country_expected_rows: i64,
    pub(crate) verify_country_actual_rows: i64,
    pub(crate) verify_country_missing_rows: i64,
    pub(crate) verify_country_extra_rows: i64,
    pub(crate) verify_country_mismatched_rows: i64,
    pub(crate) verify_institution_expected_rows: i64,
    pub(crate) verify_institution_actual_rows: i64,
    pub(crate) verify_institution_missing_rows: i64,
    pub(crate) verify_institution_extra_rows: i64,
    pub(crate) verify_institution_mismatched_rows: i64,
    pub(crate) verify_ambiguity_expected_rows: i64,
    pub(crate) verify_ambiguity_actual_rows: i64,
    pub(crate) verify_ambiguity_missing_rows: i64,
    pub(crate) verify_ambiguity_extra_rows: i64,
    pub(crate) verify_ambiguity_mismatched_rows: i64,
    pub(crate) verify_country_year_expected_rows: i64,
    pub(crate) verify_country_year_actual_rows: i64,
    pub(crate) verify_country_year_missing_rows: i64,
    pub(crate) verify_country_year_extra_rows: i64,
    pub(crate) verify_country_year_mismatched_rows: i64,
    pub(crate) verify_institution_year_expected_rows: i64,
    pub(crate) verify_institution_year_actual_rows: i64,
    pub(crate) verify_institution_year_missing_rows: i64,
    pub(crate) verify_institution_year_extra_rows: i64,
    pub(crate) verify_institution_year_mismatched_rows: i64,
}

/// One unit of the key space: `(work_id, year_start)`, in unit order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Unit {
    pub(crate) work_id: Uuid,
    pub(crate) year_start: NaiveDate,
}

impl Unit {
    /// The 31 December of the unit's year: the date a range bound binds.
    fn year_end(self) -> NaiveDate {
        NaiveDate::from_ymd_opt(self.year_start.year(), 12, 31).unwrap_or(self.year_start)
    }

    fn log_text(unit: Option<Self>) -> String {
        unit.map_or_else(
            || "none".to_string(),
            |unit| format!("{}/{}", unit.work_id, unit.year_start.year()),
        )
    }
}

/// The range verifier's bounds of the range `(lo, hi]`.
pub(crate) fn range_bounds(lo: Option<Unit>, hi: Option<Unit>) -> RangeBounds {
    RangeBounds {
        lo_work_id: lo.map_or(Uuid::nil(), |unit| unit.work_id),
        lo_end: lo.map(Unit::year_end),
        hi_work_id: hi.map_or(Uuid::max(), |unit| unit.work_id),
        hi_end: hi.map(Unit::year_end),
    }
}

impl RunRow {
    fn mode(&self) -> Result<MetricRollupMonthRebuildMode, MetricRollupMonthRebuildError> {
        MetricRollupMonthRebuildMode::from_stored(&self.mode).ok_or_else(invariant)
    }

    fn status(&self) -> Result<MetricRollupMonthRebuildStatus, MetricRollupMonthRebuildError> {
        MetricRollupMonthRebuildStatus::from_stored(&self.status).ok_or_else(invariant)
    }

    pub(crate) fn cursor(&self) -> Result<Option<Unit>, MetricRollupMonthRebuildError> {
        match (self.cursor_work_id, self.cursor_year_start) {
            (Some(work_id), Some(year_start)) => Ok(Some(Unit {
                work_id,
                year_start,
            })),
            (None, None) => Ok(None),
            _ => Err(invariant()),
        }
    }

    fn verify_counts(&self) -> [[i64; 5]; 6] {
        [
            [
                self.verify_total_expected_rows,
                self.verify_total_actual_rows,
                self.verify_total_missing_rows,
                self.verify_total_extra_rows,
                self.verify_total_mismatched_rows,
            ],
            [
                self.verify_country_expected_rows,
                self.verify_country_actual_rows,
                self.verify_country_missing_rows,
                self.verify_country_extra_rows,
                self.verify_country_mismatched_rows,
            ],
            [
                self.verify_institution_expected_rows,
                self.verify_institution_actual_rows,
                self.verify_institution_missing_rows,
                self.verify_institution_extra_rows,
                self.verify_institution_mismatched_rows,
            ],
            [
                self.verify_ambiguity_expected_rows,
                self.verify_ambiguity_actual_rows,
                self.verify_ambiguity_missing_rows,
                self.verify_ambiguity_extra_rows,
                self.verify_ambiguity_mismatched_rows,
            ],
            [
                self.verify_country_year_expected_rows,
                self.verify_country_year_actual_rows,
                self.verify_country_year_missing_rows,
                self.verify_country_year_extra_rows,
                self.verify_country_year_mismatched_rows,
            ],
            [
                self.verify_institution_year_expected_rows,
                self.verify_institution_year_actual_rows,
                self.verify_institution_year_missing_rows,
                self.verify_institution_year_extra_rows,
                self.verify_institution_year_mismatched_rows,
            ],
        ]
    }

    /// The run as the operations return it: counts, frontiers, timestamps
    /// and the run's own id only.
    pub(crate) fn into_run(
        self,
    ) -> Result<MetricRollupMonthRebuildRun, MetricRollupMonthRebuildError> {
        let mode = self.mode()?;
        let status = self.status()?;
        let family = |counts: [i64; 5]| MetricRollupMonthProjectionVerification {
            expected_rows: counts[0],
            actual_rows: counts[1],
            missing_rows: counts[2],
            extra_rows: counts[3],
            mismatched_rows: counts[4],
        };
        let [total, country, institution, ambiguity, country_year, institution_year] =
            self.verify_counts();
        let build = (mode == MetricRollupMonthRebuildMode::Rebuild).then_some(
            MetricRollupMonthRebuildBuildProgress {
                steps: self.build_steps,
                replaced_steps: self.build_replaced_steps,
                units: self.build_units,
                month_keys: self.build_month_keys,
                work_day_rows: self.build_work_day_rows,
                found_missing_rows: self.build_found_missing_rows,
                found_extra_rows: self.build_found_extra_rows,
                found_mismatched_rows: self.build_found_mismatched_rows,
                first_applied_through_sequence: self.build_first_frontier,
                last_applied_through_sequence: self.build_last_frontier,
            },
        );
        Ok(MetricRollupMonthRebuildRun {
            run_id: self.run_id,
            mode,
            status,
            step_count: self.step_count,
            position_permille: position_permille(status, self.cursor_work_id),
            started_at: self.started_at,
            updated_at: self.updated_at,
            build_completed_at: self.build_completed_at,
            completed_at: self.completed_at,
            start_applied_through_sequence: self.start_applied_through_sequence,
            build,
            verify: MetricRollupMonthRebuildVerifyProgress {
                pages: self.verify_pages,
                units: self.verify_units,
                month_keys: self.verify_month_keys,
                work_day_rows: self.verify_work_day_rows,
                first_applied_through_sequence: self.verify_first_frontier,
                last_applied_through_sequence: self.verify_last_frontier,
                counts: MetricRollupMonthRebuildFamilies {
                    total: family(total),
                    country: family(country),
                    institution: family(institution),
                    ambiguity: family(ambiguity),
                    country_year: family(country_year),
                    institution_year: family(institution_year),
                },
            },
        })
    }
}

/// `floor(first 32 bits of cursor_work_id * 1000 / 2^32)`: 0 when the cursor
/// is `NULL` mid-phase, 1000 at `RECONCILED`, `VERIFIED` and `MISMATCHED`,
/// and the retained cursor's position at `ABANDONED`. An estimate, uniform
/// for version-4 work ids, never a count.
pub(crate) fn position_permille(
    status: MetricRollupMonthRebuildStatus,
    cursor_work_id: Option<Uuid>,
) -> i32 {
    use MetricRollupMonthRebuildStatus::{Mismatched, Reconciled, Verified};
    if matches!(status, Reconciled | Verified | Mismatched) {
        return 1000;
    }
    cursor_work_id.map_or(0, |work_id| {
        let bytes = work_id.as_bytes();
        let lead = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        // At most (2^32 - 1) * 1000 / 2^32 < 1000, so the cast is exact.
        ((u64::from(lead) * 1000) >> 32) as i32
    })
}

/// The changes one build step writes to its run row, besides the
/// timestamps.
#[derive(Debug, Clone, diesel::AsChangeset)]
#[diesel(table_name = run_table, treat_none_as_null = true)]
struct BuildStepChanges {
    status: String,
    step_count: i64,
    cursor_work_id: Option<Uuid>,
    cursor_year_start: Option<NaiveDate>,
    build_steps: i64,
    build_replaced_steps: i64,
    build_units: i64,
    build_month_keys: i64,
    build_work_day_rows: i64,
    build_found_missing_rows: i64,
    build_found_extra_rows: i64,
    build_found_mismatched_rows: i64,
    build_first_frontier: Option<i64>,
    build_last_frontier: Option<i64>,
}

/// The changes one verify page writes to its run row, besides the
/// timestamps.
#[derive(Debug, Clone, diesel::AsChangeset)]
#[diesel(table_name = run_table, treat_none_as_null = true)]
struct VerifyPageChanges {
    status: String,
    step_count: i64,
    cursor_work_id: Option<Uuid>,
    cursor_year_start: Option<NaiveDate>,
    verify_pages: i64,
    verify_units: i64,
    verify_month_keys: i64,
    verify_work_day_rows: i64,
    verify_first_frontier: Option<i64>,
    verify_last_frontier: Option<i64>,
    verify_total_expected_rows: i64,
    verify_total_actual_rows: i64,
    verify_total_missing_rows: i64,
    verify_total_extra_rows: i64,
    verify_total_mismatched_rows: i64,
    verify_country_expected_rows: i64,
    verify_country_actual_rows: i64,
    verify_country_missing_rows: i64,
    verify_country_extra_rows: i64,
    verify_country_mismatched_rows: i64,
    verify_institution_expected_rows: i64,
    verify_institution_actual_rows: i64,
    verify_institution_missing_rows: i64,
    verify_institution_extra_rows: i64,
    verify_institution_mismatched_rows: i64,
    verify_ambiguity_expected_rows: i64,
    verify_ambiguity_actual_rows: i64,
    verify_ambiguity_missing_rows: i64,
    verify_ambiguity_extra_rows: i64,
    verify_ambiguity_mismatched_rows: i64,
    verify_country_year_expected_rows: i64,
    verify_country_year_actual_rows: i64,
    verify_country_year_missing_rows: i64,
    verify_country_year_extra_rows: i64,
    verify_country_year_mismatched_rows: i64,
    verify_institution_year_expected_rows: i64,
    verify_institution_year_actual_rows: i64,
    verify_institution_year_missing_rows: i64,
    verify_institution_year_extra_rows: i64,
    verify_institution_year_mismatched_rows: i64,
}

#[derive(diesel::QueryableByName)]
struct IsolationRow {
    #[diesel(sql_type = Text)]
    isolation: String,
}

#[derive(diesel::QueryableByName)]
struct FrontierRow {
    #[diesel(sql_type = BigInt)]
    applied_through_sequence: i64,
}

#[derive(diesel::QueryableByName)]
struct WorkIdRow {
    #[diesel(sql_type = SqlUuid)]
    #[allow(dead_code)]
    work_id: Uuid,
}

#[derive(diesel::QueryableByName)]
struct SourceWatermarkRow {
    #[diesel(sql_type = Nullable<BigInt>)]
    watermark: Option<i64>,
}

#[derive(diesel::QueryableByName)]
struct MonthKeyRow {
    #[diesel(sql_type = SqlUuid)]
    work_id: Uuid,
    #[diesel(sql_type = SqlUuid)]
    platform_id: Uuid,
    #[diesel(sql_type = SqlUuid)]
    measure_id: Uuid,
    #[diesel(sql_type = Date)]
    month_start: NaiveDate,
}

/// One work of a sizing walk.
#[derive(Debug, Clone, PartialEq, Eq, diesel::QueryableByName)]
pub(crate) struct SizingRow {
    #[diesel(sql_type = BigInt)]
    pub(crate) n: i64,
    #[diesel(sql_type = SqlUuid)]
    pub(crate) work_id: Uuid,
    #[diesel(sql_type = BigInt)]
    pub(crate) units: i64,
    #[diesel(sql_type = BigInt)]
    pub(crate) unit_rows: i64,
    #[diesel(sql_type = BigInt)]
    pub(crate) month_keys: i64,
    #[diesel(sql_type = BigInt)]
    pub(crate) figure: i64,
    #[diesel(sql_type = BigInt)]
    pub(crate) whole_rows: i64,
    #[diesel(sql_type = Date)]
    pub(crate) first_year: NaiveDate,
    #[diesel(sql_type = Date)]
    pub(crate) last_year: NaiveDate,
}

/// One year of a work that alone exceeds a budget.
#[derive(Debug, Clone, PartialEq, Eq, diesel::QueryableByName)]
pub(crate) struct YearSizingRow {
    #[diesel(sql_type = Date)]
    pub(crate) year_start: NaiveDate,
    #[diesel(sql_type = BigInt)]
    pub(crate) unit_rows: i64,
    #[diesel(sql_type = BigInt)]
    pub(crate) month_keys: i64,
    #[diesel(sql_type = BigInt)]
    pub(crate) figure: i64,
}

// ---------------------------------------------------------------------------
// Sizing
// ---------------------------------------------------------------------------

/// A sized range `(lo, hi]` of units, with the counts of its units that have
/// work-day rows. `hi` is `None` for plus infinity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SizedRange {
    pub(crate) lo: Option<Unit>,
    pub(crate) hi: Option<Unit>,
    /// The works with work-day rows in the range, ascending.
    pub(crate) works: Vec<Uuid>,
    pub(crate) units: i64,
    pub(crate) work_day_rows: i64,
    pub(crate) month_keys: i64,
    pub(crate) replacing_read: i64,
    pub(crate) first_unit: Option<Unit>,
    pub(crate) last_unit: Option<Unit>,
    /// Sizing statements issued, the per-year statement included.
    pub(crate) statements: usize,
}

impl SizedRange {
    pub(crate) fn bounds(&self) -> RangeBounds {
        range_bounds(self.lo, self.hi)
    }

    /// The range's three totals plus one more work or year, or the overflow
    /// refusal.
    fn plus(
        &self,
        rows: i64,
        keys: i64,
        figure: i64,
    ) -> Result<(i64, i64, i64), MetricRollupMonthRebuildError> {
        Ok((
            add(self.work_day_rows, rows)?,
            add(self.month_keys, keys)?,
            add(self.replacing_read, figure)?,
        ))
    }

    /// End the range after its last taken unit, or refuse an empty re-size.
    fn cut(mut self) -> Result<Self, MetricRollupMonthRebuildError> {
        if self.units == 0 {
            return Err(MetricRollupMonthRebuildError::Retryable(
                reason::EMPTY_RESIZE,
            ));
        }
        self.hi = self.last_unit;
        Ok(self)
    }
}

fn over(totals: (i64, i64, i64), budgets: SizeBudgets) -> bool {
    totals.0 > budgets.work_day_rows
        || totals.1 > budgets.month_keys
        || totals.2 > budgets.replacing_read
}

fn filled(totals: (i64, i64, i64), budgets: SizeBudgets) -> bool {
    totals.0 >= budgets.work_day_rows
        || totals.1 >= budgets.month_keys
        || totals.2 >= budgets.replacing_read
}

/// The sizing loop of section 10, from `lo`.
///
/// Takes whole works while they fit the budgets; if the first work alone
/// exceeds a budget, one more statement returns its per-year counts and the
/// range takes its years while they fit, always at least one unit. Another
/// sizing statement is issued only if the batch limit stopped the walk and
/// the budgets are not yet filled. When the walk finds no further work, the
/// range ends at plus infinity, so trailing orphan rows are covered.
///
/// With `locked`, only works in that set are admitted: the range ends
/// before the first work outside it, and a range left without any unit is
/// the retryable empty re-size.
pub(crate) fn size_range(
    connection: &mut PgConnection,
    clock: &Clock,
    stage: Stage,
    lo: Option<Unit>,
    budgets: SizeBudgets,
    locked: Option<&BTreeSet<Uuid>>,
) -> Result<SizedRange, MetricRollupMonthRebuildError> {
    let mut range = SizedRange {
        lo,
        hi: None,
        works: Vec::new(),
        units: 0,
        work_day_rows: 0,
        month_keys: 0,
        replacing_read: 0,
        first_unit: None,
        last_unit: None,
        statements: 0,
    };
    let lo_work_id = lo.map_or(Uuid::nil(), |unit| unit.work_id);
    let lo_end = lo.map(Unit::year_end);
    let mut from_work_id = lo_work_id;
    let mut from_end = lo_end;
    loop {
        let remaining = (
            budgets.work_day_rows - range.work_day_rows,
            budgets.month_keys - range.month_keys,
            budgets.replacing_read - range.replacing_read,
        );
        let walk: Vec<SizingRow> = run_statement(connection, clock, Site::Sizing(stage), |c| {
            diesel::sql_query(SIZING_SQL)
                .bind::<SqlUuid, _>(from_work_id)
                .bind::<Nullable<Date>, _>(from_end)
                .bind::<BigInt, _>(METRIC_ROLLUP_RUN_SIZING_BATCH_WORKS)
                .bind::<BigInt, _>(remaining.0)
                .bind::<BigInt, _>(remaining.1)
                .bind::<BigInt, _>(remaining.2)
                .load(c)
        })?;
        range.statements += 1;
        let Some(last_work_id) = walk.last().map(|row| row.work_id) else {
            range.hi = None;
            return Ok(range);
        };
        let batch_full =
            i64::try_from(walk.len()).ok() == Some(METRIC_ROLLUP_RUN_SIZING_BATCH_WORKS);
        for row in &walk {
            if locked.is_some_and(|locked| !locked.contains(&row.work_id)) {
                return range.cut();
            }
            let totals = range.plus(row.unit_rows, row.month_keys, row.figure)?;
            if over(totals, budgets) {
                if range.units == 0 {
                    return split_first_work(
                        connection, clock, stage, range, row, budgets, lo_work_id, lo_end,
                    );
                }
                return range.cut();
            }
            range.works.push(row.work_id);
            range.units = add(range.units, row.units)?;
            (range.work_day_rows, range.month_keys, range.replacing_read) = totals;
            range.first_unit.get_or_insert(Unit {
                work_id: row.work_id,
                year_start: row.first_year,
            });
            range.last_unit = Some(Unit {
                work_id: row.work_id,
                year_start: row.last_year,
            });
            if filled(totals, budgets) {
                range.hi = range.last_unit;
                return Ok(range);
            }
        }
        if !batch_full {
            range.hi = None;
            return Ok(range);
        }
        // Continue after the batch's last work: every unit of a greater
        // work_id, whole.
        let Some(next) = last_work_id.as_u128().checked_add(1) else {
            range.hi = None;
            return Ok(range);
        };
        from_work_id = Uuid::from_u128(next);
        from_end = None;
    }
}

/// Take the years of the range's first work while they fit, always at
/// least one.
#[allow(clippy::too_many_arguments)]
fn split_first_work(
    connection: &mut PgConnection,
    clock: &Clock,
    stage: Stage,
    mut range: SizedRange,
    work: &SizingRow,
    budgets: SizeBudgets,
    lo_work_id: Uuid,
    lo_end: Option<NaiveDate>,
) -> Result<SizedRange, MetricRollupMonthRebuildError> {
    let years: Vec<YearSizingRow> = run_statement(connection, clock, Site::PerYear(stage), |c| {
        diesel::sql_query(PER_YEAR_SQL)
            .bind::<SqlUuid, _>(lo_work_id)
            .bind::<Nullable<Date>, _>(lo_end)
            .bind::<SqlUuid, _>(work.work_id)
            .bind::<BigInt, _>(work.whole_rows)
            .load(c)
    })?;
    range.statements += 1;
    for year in &years {
        let totals = range.plus(year.unit_rows, year.month_keys, year.figure)?;
        if range.units > 0 && over(totals, budgets) {
            break;
        }
        if range.units == 0 {
            range.works.push(work.work_id);
        }
        range.units += 1;
        (range.work_day_rows, range.month_keys, range.replacing_read) = totals;
        let unit = Unit {
            work_id: work.work_id,
            year_start: year.year_start,
        };
        range.first_unit.get_or_insert(unit);
        range.last_unit = Some(unit);
        if filled(totals, budgets) {
            break;
        }
    }
    if range.units == 0 {
        return Err(invariant());
    }
    range.hi = range.last_unit;
    Ok(range)
}

// ---------------------------------------------------------------------------
// Connections and transactions
// ---------------------------------------------------------------------------

/// Check out the operation's one pooled connection, waiting at most
/// [`METRIC_ROLLUP_RUN_CHECKOUT_TIMEOUT`]. An expired wait is an r2d2 error,
/// mapped here, before any conversion, to the retryable refusal.
fn checkout(
    db: &PgPool,
) -> Result<PooledConnection<ConnectionManager<PgConnection>>, MetricRollupMonthRebuildError> {
    db.get_timeout(METRIC_ROLLUP_RUN_CHECKOUT_TIMEOUT)
        .map_err(|error| {
            log::warn!("metric_rollup_rebuild pool checkout failed: {error}");
            MetricRollupMonthRebuildError::Retryable(reason::POOL_CHECKOUT)
        })
}

/// The isolation of one run transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Isolation {
    ReadCommitted,
    ReadCommittedReadOnly,
    RepeatableRead,
}

/// Run `body` in one transaction and keep its first classified error: a
/// rollback that fails afterwards (for example on a connection the server
/// already ended) never replaces it.
fn transact<T>(
    connection: &mut PgConnection,
    isolation: Isolation,
    body: impl FnOnce(&mut PgConnection) -> Result<T, MetricRollupMonthRebuildError>,
) -> Result<T, MetricRollupMonthRebuildError> {
    let mut first: Option<MetricRollupMonthRebuildError> = None;
    let mut keep = |error: MetricRollupMonthRebuildError| *first.get_or_insert(error);
    let result = match isolation {
        Isolation::ReadCommitted => connection
            .build_transaction()
            .read_committed()
            .run(|c| body(c).map_err(&mut keep)),
        Isolation::ReadCommittedReadOnly => connection
            .build_transaction()
            .read_committed()
            .read_only()
            .run(|c| body(c).map_err(&mut keep)),
        Isolation::RepeatableRead => connection
            .build_transaction()
            .repeatable_read()
            .run(|c| body(c).map_err(&mut keep)),
    };
    result.map_err(|error| first.unwrap_or(error))
}

/// Run fixed session settings, classifying any failure.
fn settings(
    connection: &mut PgConnection,
    statements: &[&str],
) -> Result<(), MetricRollupMonthRebuildError> {
    for statement in statements {
        diesel::sql_query(*statement).execute(connection)?;
    }
    Ok(())
}

/// Read one run, or the latest run, in its own short read-only transaction
/// under the 1 s lock and statement timeouts.
fn read_run(
    connection: &mut PgConnection,
    run_id: Option<Uuid>,
) -> Result<Option<RunRow>, MetricRollupMonthRebuildError> {
    transact(connection, Isolation::ReadCommittedReadOnly, |c| {
        settings(c, &READ_SESSION_SQL)?;
        let rows: Vec<RunRow> = match run_id {
            Some(run_id) => diesel::sql_query(RUN_READ_SQL)
                .bind::<SqlUuid, _>(run_id)
                .load(c)?,
            None => diesel::sql_query(LATEST_RUN_SQL).load(c)?,
        };
        Ok(rows.into_iter().next())
    })
}

/// Read one run row inside a step or a page, under the current budget.
fn load_run(
    connection: &mut PgConnection,
    clock: &Clock,
    site: Site,
    statement: &'static str,
    run_id: Uuid,
) -> Result<RunRow, MetricRollupMonthRebuildError> {
    let rows: Vec<RunRow> = run_statement(connection, clock, site, |c| {
        diesel::sql_query(statement)
            .bind::<SqlUuid, _>(run_id)
            .load(c)
    })?;
    rows.into_iter().next().ok_or_else(invariant)
}

/// Fail closed unless the server reports `expected` as this transaction's
/// isolation level.
fn require_isolation(
    connection: &mut PgConnection,
    clock: &Clock,
    site: Site,
    expected: &str,
) -> Result<(), MetricRollupMonthRebuildError> {
    let rows: Vec<IsolationRow> = run_statement(connection, clock, site, |c| {
        diesel::sql_query(ISOLATION_SQL).load(c)
    })?;
    if rows.first().is_some_and(|row| row.isolation == expected) {
        Ok(())
    } else {
        Err(MetricRollupMonthRebuildError::Internal(reason::ISOLATION))
    }
}

// ---------------------------------------------------------------------------
// Operations
// ---------------------------------------------------------------------------

/// Parse the caller's expected step count: exactly a decimal string of ASCII
/// digits that fits a non-negative `i64`, decided before any database
/// access.
pub(crate) fn parse_expected_step_count(
    expected: &str,
) -> Result<i64, MetricRollupMonthRebuildError> {
    let valid = !expected.is_empty()
        && expected.len() <= 19
        && expected.bytes().all(|byte| byte.is_ascii_digit());
    valid
        .then(|| expected.parse::<i64>().ok())
        .flatten()
        .ok_or(MetricRollupMonthRebuildError::QueryInvalid)
}

/// Start a run in `input.mode` at the current frontier, read without a
/// lock: `BUILDING` with a `NULL` cursor for `REBUILD`, `VERIFYING` for
/// `VERIFY`. A second open run violates the partial unique index, which
/// surfaces as `METRIC_REBUILD_RUN_OPEN`, so a concurrent first arrival
/// yields exactly one run.
pub fn start_metric_rollup_month_rebuild(
    db: &PgPool,
    input: &StartMetricRollupMonthRebuildInput,
    started_by: &str,
) -> Result<MetricRollupMonthRebuildRun, MetricRollupMonthRebuildError> {
    let status = match input.mode {
        MetricRollupMonthRebuildMode::Rebuild => MetricRollupMonthRebuildStatus::Building,
        MetricRollupMonthRebuildMode::Verify => MetricRollupMonthRebuildStatus::Verifying,
    };
    let mut connection = checkout(db)?;
    let row = transact(&mut connection, Isolation::ReadCommitted, |c| {
        settings(c, &WRITE_SESSION_SQL)?;
        let rows: Vec<RunRow> = diesel::sql_query(START_SQL)
            .bind::<Text, _>(input.mode.as_str())
            .bind::<Text, _>(status.as_str())
            .bind::<Text, _>(started_by)
            .load(c)?;
        rows.into_iter().next().ok_or_else(invariant)
    })?;
    row.into_run()
}

/// Abandon an open run: `ABANDONED` with `completed_at`, keeping its cursor,
/// counters and frontiers. A terminal run is returned unchanged. Against an
/// in-flight step or page it waits on the run row, behind the 5 s
/// `lock_timeout`, or fails retryable.
pub fn abandon_metric_rollup_month_rebuild(
    db: &PgPool,
    input: &AbandonMetricRollupMonthRebuildInput,
) -> Result<MetricRollupMonthRebuildRun, MetricRollupMonthRebuildError> {
    let mut connection = checkout(db)?;
    let (row, abandoned) = transact(&mut connection, Isolation::ReadCommitted, |c| {
        settings(c, &WRITE_SESSION_SQL)?;
        let rows: Vec<RunRow> = diesel::sql_query(RUN_LOCK_SQL)
            .bind::<SqlUuid, _>(input.run_id)
            .load(c)?;
        let row = rows
            .into_iter()
            .next()
            .ok_or(MetricRollupMonthRebuildError::QueryInvalid)?;
        if !row.status()?.is_open() {
            return Ok((row, false));
        }
        let row: RunRow =
            diesel::update(run_table::table.filter(run_table::run_id.eq(input.run_id)))
                .set((
                    run_table::status.eq(MetricRollupMonthRebuildStatus::Abandoned.as_str()),
                    run_table::updated_at.eq(statement_time()),
                    run_table::completed_at.eq(statement_time_nullable()),
                ))
                .returning(RunRow::as_returning())
                .get_result(c)?;
        Ok((row, true))
    })?;
    if abandoned {
        log_status(&row);
    }
    row.into_run()
}

/// The run `run_id`, or `None`. Without a `run_id`: the open run when one
/// exists, otherwise the run with the greatest `started_at` (ties broken by
/// `run_id`), or `None`. Read-only, under the 1 s lock and statement
/// timeouts.
pub fn metric_rollup_month_rebuild_run(
    db: &PgPool,
    run_id: Option<Uuid>,
) -> Result<Option<MetricRollupMonthRebuildRun>, MetricRollupMonthRebuildError> {
    let mut connection = checkout(db)?;
    read_run(&mut connection, run_id)?
        .map(RunRow::into_run)
        .transpose()
}

/// Advance a run by one build step or one verify page.
pub fn advance_metric_rollup_month_rebuild(
    db: &PgPool,
    input: &AdvanceMetricRollupMonthRebuildInput,
) -> Result<MetricRollupMonthRebuildAdvance, MetricRollupMonthRebuildError> {
    advance_metric_rollup_month_rebuild_within(db, input, &RunBudgets::PRODUCTION)
}

/// [`advance_metric_rollup_month_rebuild`] under explicit budgets, for tests.
///
/// Validates `expectedStepCount` before any database access, checks out one
/// connection, and reads the run's status in its own short transaction to
/// choose the build step or the verify page. An unknown run is
/// `METRIC_QUERY_INVALID`; a terminal run, or one whose `step_count` is not
/// the expected count, returns `advanced: false` with the run unchanged.
/// The chosen transaction re-checks both under its locks.
pub(crate) fn advance_metric_rollup_month_rebuild_within(
    db: &PgPool,
    input: &AdvanceMetricRollupMonthRebuildInput,
    budgets: &RunBudgets,
) -> Result<MetricRollupMonthRebuildAdvance, MetricRollupMonthRebuildError> {
    let expected = parse_expected_step_count(&input.expected_step_count)?;
    let attempted = expected.saturating_add(1);
    let mut connection = checkout(db).inspect_err(|error| {
        log_refusal(input.run_id, "none", attempted, *error, None, 0);
    })?;
    let row = read_run(&mut connection, Some(input.run_id))
        .inspect_err(|error| log_refusal(input.run_id, "none", attempted, *error, None, 0))?
        .ok_or(MetricRollupMonthRebuildError::QueryInvalid)?;
    let status = row.status()?;
    if !status.is_open() || row.step_count != expected {
        return Ok(MetricRollupMonthRebuildAdvance {
            advanced: false,
            run: row.into_run()?,
        });
    }
    let mode = row.mode()?;
    if status == MetricRollupMonthRebuildStatus::Building {
        build_step(&mut connection, input.run_id, mode, expected, budgets)
    } else {
        verify_page(&mut connection, input.run_id, mode, expected, budgets)
    }
}

// ---------------------------------------------------------------------------
// Build step and verify page
// ---------------------------------------------------------------------------

/// What a step or page did, for the response and its log lines.
enum Outcome {
    /// The re-check under the locks found another status or step count:
    /// nothing was written.
    Stale(Box<RunRow>),
    Committed(Box<Committed>),
}

struct Committed {
    run: RunRow,
    frontier: i64,
    replaced: bool,
    units: i64,
    month_keys: i64,
    work_day_rows: i64,
    missing: i64,
    extra: i64,
    mismatched: i64,
    first_unit: Option<Unit>,
    last_unit: Option<Unit>,
}

/// What a step or page knew when it ended, for its refusal line.
#[derive(Default)]
struct Trace {
    hold_started: Option<Instant>,
    range_first: Option<Unit>,
}

impl Trace {
    fn hold_ms(&self) -> u128 {
        self.hold_started
            .map_or(0, |started| started.elapsed().as_millis())
    }
}

fn finish(
    run_id: Uuid,
    mode: MetricRollupMonthRebuildMode,
    phase: &'static str,
    attempted: i64,
    trace: &Trace,
    result: Result<Outcome, MetricRollupMonthRebuildError>,
) -> Result<MetricRollupMonthRebuildAdvance, MetricRollupMonthRebuildError> {
    let hold_ms = trace.hold_ms();
    match result {
        Ok(Outcome::Stale(row)) => Ok(MetricRollupMonthRebuildAdvance {
            advanced: false,
            run: row.into_run()?,
        }),
        Ok(Outcome::Committed(committed)) => {
            let run = committed.run.clone().into_run()?;
            log_commit(mode, phase, &committed, &run, hold_ms);
            if !run.status.is_open() {
                log_status(&committed.run);
            }
            Ok(MetricRollupMonthRebuildAdvance {
                advanced: true,
                run,
            })
        }
        Err(error) => {
            log_refusal(run_id, phase, attempted, error, trace.range_first, hold_ms);
            Err(error)
        }
    }
}

/// One build step (section 8.1), in one `READ COMMITTED` transaction.
fn build_step(
    connection: &mut PgConnection,
    run_id: Uuid,
    mode: MetricRollupMonthRebuildMode,
    expected: i64,
    budgets: &RunBudgets,
) -> Result<MetricRollupMonthRebuildAdvance, MetricRollupMonthRebuildError> {
    let mut trace = Trace::default();
    let result = transact(connection, Isolation::ReadCommitted, |c| {
        build_step_body(c, run_id, expected, budgets, &mut trace)
    });
    finish(
        run_id,
        mode,
        "build",
        expected.saturating_add(1),
        &trace,
        result,
    )
}

fn build_step_body(
    c: &mut PgConnection,
    run_id: Uuid,
    expected: i64,
    budgets: &RunBudgets,
    trace: &mut Trace,
) -> Result<Outcome, MetricRollupMonthRebuildError> {
    use MetricRollupMonthRebuildError::Blocked;
    settings(c, &STEP_SESSION_SQL)?;

    // Pre-size without any row lock.
    let presize = Clock::start(budgets.step_presize);
    require_isolation(
        c,
        &presize,
        Site::Isolation(Stage::Presize),
        "read committed",
    )?;
    let unlocked = load_run(c, &presize, Site::RunRead, RUN_READ_SQL, run_id)?;
    let candidate = size_range(
        c,
        &presize,
        Stage::Presize,
        unlocked.cursor()?,
        budgets.step_size(),
        None,
    )?;
    trace.range_first = candidate.first_unit;

    // Lock the works, then the state row.
    let lock = Clock::start(budgets.step_lock);
    let _: Vec<WorkIdRow> = run_statement(c, &lock, Site::LockWorks, |c| {
        diesel::sql_query(LOCK_WORKS_SQL)
            .bind::<Array<SqlUuid>, _>(&candidate.works)
            .load(c)
    })?;
    let state: Vec<FrontierRow> = run_statement(c, &lock, Site::LockState, |c| {
        diesel::sql_query(LOCK_STATE_SQL).load(c)
    })?;
    let hold = Clock::start(budgets.step_hold);
    trace.hold_started = Some(hold.started);
    let frontier = state
        .first()
        .map(|row| row.applied_through_sequence)
        .ok_or_else(invariant)?;

    // Re-check the run under its lock.
    let run = load_run(c, &hold, Site::RunLock(Stage::Hold), RUN_LOCK_SQL, run_id)?;
    if run.status()? != MetricRollupMonthRebuildStatus::Building || run.step_count != expected {
        return Ok(Outcome::Stale(Box::new(run)));
    }

    // Re-size under the locks, admitting only locked works.
    let locked: BTreeSet<Uuid> = candidate.works.iter().copied().collect();
    let range = size_range(
        c,
        &hold,
        Stage::Hold,
        run.cursor()?,
        budgets.step_size(),
        Some(&locked),
    )?;
    trace.range_first = range.first_unit;
    let bounds = range.bounds();

    // Pre-verify.
    let pre = run_statement(c, &hold, Site::PreVerify, |c| verify_range(c, &bounds))?;
    if pre.work_day_above(frontier) {
        return Err(Blocked(reason::WORK_DAY_ABOVE_FRONTIER));
    }
    let replaced = !pre.exact_at(frontier);
    if replaced {
        replace_range(c, &hold, &bounds, &pre, frontier)?;
        let post = run_statement(c, &hold, Site::PostVerify, |c| verify_range(c, &bounds))?;
        if !post.exact_at(frontier) || post.work_day_above(frontier) {
            return Err(Blocked(reason::POST_VERIFY_INEXACT));
        }
    }

    // Update the run row.
    let reached_end = range.hi.is_none();
    let cursor = if reached_end { None } else { range.hi };
    let changes = BuildStepChanges {
        status: if reached_end {
            MetricRollupMonthRebuildStatus::Verifying
        } else {
            MetricRollupMonthRebuildStatus::Building
        }
        .as_str()
        .to_string(),
        step_count: add(run.step_count, 1)?,
        cursor_work_id: cursor.map(|unit| unit.work_id),
        cursor_year_start: cursor.map(|unit| unit.year_start),
        build_steps: add(run.build_steps, 1)?,
        build_replaced_steps: add(run.build_replaced_steps, i64::from(replaced))?,
        build_units: add(run.build_units, range.units)?,
        build_month_keys: add(run.build_month_keys, pre.month_keys)?,
        build_work_day_rows: add(run.build_work_day_rows, pre.work_day_rows)?,
        build_found_missing_rows: add(run.build_found_missing_rows, pre.missing_rows())?,
        build_found_extra_rows: add(run.build_found_extra_rows, pre.extra_rows())?,
        build_found_mismatched_rows: add(run.build_found_mismatched_rows, pre.mismatched_rows())?,
        build_first_frontier: Some(run.build_first_frontier.unwrap_or(frontier)),
        build_last_frontier: Some(frontier),
    };
    let updated: RunRow = run_statement(c, &hold, Site::RunUpdate(Stage::Hold), |c| {
        let target = run_table::table.filter(run_table::run_id.eq(run_id));
        if reached_end {
            diesel::update(target)
                .set((
                    &changes,
                    run_table::updated_at.eq(statement_time()),
                    run_table::build_completed_at.eq(statement_time_nullable()),
                ))
                .returning(RunRow::as_returning())
                .get_result(c)
        } else {
            diesel::update(target)
                .set((&changes, run_table::updated_at.eq(statement_time())))
                .returning(RunRow::as_returning())
                .get_result(c)
        }
    })?;
    Ok(Outcome::Committed(Box::new(Committed {
        run: updated,
        frontier,
        replaced,
        units: range.units,
        month_keys: pre.month_keys,
        work_day_rows: pre.work_day_rows,
        missing: pre.missing_rows(),
        extra: pre.extra_rows(),
        mismatched: pre.mismatched_rows(),
        first_unit: range.first_unit,
        last_unit: range.last_unit,
    })))
}

/// Bind a range's four bounds to a writer statement.
macro_rules! bind_range {
    ($query:expr, $bounds:expr) => {
        $query
            .bind::<SqlUuid, _>($bounds.lo_work_id)
            .bind::<Nullable<Date>, _>($bounds.lo_end)
            .bind::<SqlUuid, _>($bounds.hi_work_id)
            .bind::<Nullable<Date>, _>($bounds.hi_end)
    };
}

/// Replace exactly one inexact range (section 8.1 step 7): delete every row
/// of the six families in the range, checking each family's count against
/// the pre-verify; then re-execute the completion's own statements, one at
/// a time and in the completion's order, over the range's month keys in
/// chunks and then over their year keys in chunks. The completion's six
/// keyed deletes are skipped: the range delete has already removed every
/// row they would match.
fn replace_range(
    c: &mut PgConnection,
    hold: &Clock,
    bounds: &RangeBounds,
    pre: &RangeVerification,
    frontier: i64,
) -> Result<(), MetricRollupMonthRebuildError> {
    use MetricRollupMonthRebuildError::Blocked;
    for (index, (statement, family)) in RANGE_DELETE_STATEMENTS
        .iter()
        .zip(pre.families())
        .enumerate()
    {
        let deleted = run_statement(c, hold, Site::RangeDelete(index), |c| {
            bind_range!(diesel::sql_query(*statement), bounds).execute(c)
        })?;
        if i64::try_from(deleted).ok() != Some(family.actual_rows) {
            return Err(Blocked(reason::DELETE_COUNT_MISMATCH));
        }
    }
    let keys: Vec<MonthKeyRow> = run_statement(c, hold, Site::MonthKeys, |c| {
        bind_range!(diesel::sql_query(RANGE_MONTH_KEYS_SQL), bounds).load(c)
    })?;
    let keys: BTreeSet<MonthKey> = keys
        .into_iter()
        .map(|key| {
            (
                key.work_id,
                key.platform_id,
                key.measure_id,
                key.month_start,
            )
        })
        .collect();
    let month_keys: Vec<MonthKey> = keys.iter().copied().collect();
    for (chunk_index, chunk) in month_keys
        .chunks(METRIC_ROLLUP_RUN_STEP_CHUNK_KEYS)
        .enumerate()
    {
        let work_ids: Vec<Uuid> = chunk.iter().map(|key| key.0).collect();
        let platform_ids: Vec<Uuid> = chunk.iter().map(|key| key.1).collect();
        let measure_ids: Vec<Uuid> = chunk.iter().map(|key| key.2).collect();
        let month_starts: Vec<NaiveDate> = chunk.iter().map(|key| key.3).collect();
        let source: Vec<SourceWatermarkRow> =
            run_statement(c, hold, Site::MonthWatermark(chunk_index), |c| {
                diesel::sql_query(MONTH_SOURCE_WATERMARK_SQL)
                    .bind::<Array<SqlUuid>, _>(&work_ids)
                    .bind::<Array<SqlUuid>, _>(&platform_ids)
                    .bind::<Array<SqlUuid>, _>(&measure_ids)
                    .bind::<Array<Date>, _>(&month_starts)
                    .load(c)
            })?;
        if source
            .first()
            .and_then(|row| row.watermark)
            .is_some_and(|watermark| watermark > frontier)
        {
            return Err(Blocked(reason::SOURCE_WATERMARK_ABOVE_FRONTIER));
        }
        for (index, statement) in MONTH_INSERT_STATEMENTS.iter().enumerate() {
            run_statement(c, hold, Site::MonthInsert(chunk_index, index), |c| {
                diesel::sql_query(*statement)
                    .bind::<Array<SqlUuid>, _>(&work_ids)
                    .bind::<Array<SqlUuid>, _>(&platform_ids)
                    .bind::<Array<SqlUuid>, _>(&measure_ids)
                    .bind::<Array<Date>, _>(&month_starts)
                    .execute(c)
            })?;
        }
    }
    let year_keys: Vec<YearKey> = affected_year_keys(&keys).into_iter().collect();
    for (chunk_index, chunk) in year_keys
        .chunks(METRIC_ROLLUP_RUN_STEP_CHUNK_KEYS)
        .enumerate()
    {
        let work_ids: Vec<Uuid> = chunk.iter().map(|key| key.0).collect();
        let platform_ids: Vec<Uuid> = chunk.iter().map(|key| key.1).collect();
        let measure_ids: Vec<Uuid> = chunk.iter().map(|key| key.2).collect();
        let year_starts: Vec<NaiveDate> = chunk.iter().map(|key| key.3).collect();
        for (index, statement) in [YEAR_COUNTRY_INSERT_SQL, YEAR_INSTITUTION_INSERT_SQL]
            .iter()
            .enumerate()
        {
            run_statement(c, hold, Site::YearInsert(chunk_index, index), |c| {
                diesel::sql_query(*statement)
                    .bind::<Array<SqlUuid>, _>(&work_ids)
                    .bind::<Array<SqlUuid>, _>(&platform_ids)
                    .bind::<Array<SqlUuid>, _>(&measure_ids)
                    .bind::<Array<Date>, _>(&year_starts)
                    .execute(c)
            })?;
        }
    }
    Ok(())
}

/// One verify page (section 8.2), in one `REPEATABLE READ` transaction.
fn verify_page(
    connection: &mut PgConnection,
    run_id: Uuid,
    mode: MetricRollupMonthRebuildMode,
    expected: i64,
    budgets: &RunBudgets,
) -> Result<MetricRollupMonthRebuildAdvance, MetricRollupMonthRebuildError> {
    let mut trace = Trace::default();
    let result = transact(connection, Isolation::RepeatableRead, |c| {
        verify_page_body(c, run_id, expected, budgets, &mut trace)
    });
    finish(
        run_id,
        mode,
        "verify",
        expected.saturating_add(1),
        &trace,
        result,
    )
}

fn verify_page_body(
    c: &mut PgConnection,
    run_id: Uuid,
    expected: i64,
    budgets: &RunBudgets,
    trace: &mut Trace,
) -> Result<Outcome, MetricRollupMonthRebuildError> {
    use MetricRollupMonthRebuildError::Blocked;
    // Only `SET LOCAL` statements precede the table lock: no snapshot is
    // taken before it.
    diesel::sql_query(format!(
        "SET LOCAL statement_timeout = {}",
        budgets.page.as_millis()
    ))
    .execute(c)?;
    settings(c, &PAGE_SESSION_SQL)?;
    let page = Clock::start(budgets.page);
    trace.hold_started = Some(page.started);
    run_statement(c, &page, Site::PageLock, |c| {
        diesel::sql_query(PAGE_LOCK_SQL).execute(c)
    })?;
    // The first query: it freezes the snapshot.
    require_isolation(c, &page, Site::Isolation(Stage::Page), "repeatable read")?;
    let run = load_run(c, &page, Site::RunLock(Stage::Page), RUN_LOCK_SQL, run_id)?;
    if run.status()? != MetricRollupMonthRebuildStatus::Verifying || run.step_count != expected {
        return Ok(Outcome::Stale(Box::new(run)));
    }
    let state: Vec<FrontierRow> = run_statement(c, &page, Site::PageFrontier, |c| {
        diesel::sql_query(PAGE_FRONTIER_SQL).load(c)
    })?;
    let frontier = state
        .first()
        .map(|row| row.applied_through_sequence)
        .ok_or_else(invariant)?;
    let range = size_range(
        c,
        &page,
        Stage::Page,
        run.cursor()?,
        budgets.page_size(),
        None,
    )?;
    trace.range_first = range.first_unit;
    let bounds = range.bounds();
    let verified = run_statement(c, &page, Site::PageVerify, |c| verify_range(c, &bounds))?;
    if verified.work_day_above(frontier) {
        return Err(Blocked(reason::WORK_DAY_ABOVE_FRONTIER));
    }

    // Accumulate the page into the run's counters.
    let mut counts = run.verify_counts();
    for (accumulated, family) in counts.iter_mut().zip(verified.families()) {
        let page_counts = [
            family.expected_rows,
            family.actual_rows,
            family.missing_rows,
            family.extra_rows,
            family.mismatched_rows,
        ];
        for (total, count) in accumulated.iter_mut().zip(page_counts) {
            *total = add(*total, count)?;
        }
    }
    let first_frontier = run.verify_first_frontier.unwrap_or(frontier);
    let reached_end = range.hi.is_none();
    let status = if !reached_end {
        MetricRollupMonthRebuildStatus::Verifying
    } else if counts
        .iter()
        .any(|family| family[2] != 0 || family[3] != 0 || family[4] != 0)
    {
        MetricRollupMonthRebuildStatus::Mismatched
    } else if first_frontier == frontier {
        MetricRollupMonthRebuildStatus::Reconciled
    } else {
        MetricRollupMonthRebuildStatus::Verified
    };
    let cursor = if reached_end { None } else { range.hi };
    let [total, country, institution, ambiguity, country_year, institution_year] = counts;
    let changes = VerifyPageChanges {
        status: status.as_str().to_string(),
        step_count: add(run.step_count, 1)?,
        cursor_work_id: cursor.map(|unit| unit.work_id),
        cursor_year_start: cursor.map(|unit| unit.year_start),
        verify_pages: add(run.verify_pages, 1)?,
        verify_units: add(run.verify_units, range.units)?,
        verify_month_keys: add(run.verify_month_keys, verified.month_keys)?,
        verify_work_day_rows: add(run.verify_work_day_rows, verified.work_day_rows)?,
        verify_first_frontier: Some(first_frontier),
        verify_last_frontier: Some(frontier),
        verify_total_expected_rows: total[0],
        verify_total_actual_rows: total[1],
        verify_total_missing_rows: total[2],
        verify_total_extra_rows: total[3],
        verify_total_mismatched_rows: total[4],
        verify_country_expected_rows: country[0],
        verify_country_actual_rows: country[1],
        verify_country_missing_rows: country[2],
        verify_country_extra_rows: country[3],
        verify_country_mismatched_rows: country[4],
        verify_institution_expected_rows: institution[0],
        verify_institution_actual_rows: institution[1],
        verify_institution_missing_rows: institution[2],
        verify_institution_extra_rows: institution[3],
        verify_institution_mismatched_rows: institution[4],
        verify_ambiguity_expected_rows: ambiguity[0],
        verify_ambiguity_actual_rows: ambiguity[1],
        verify_ambiguity_missing_rows: ambiguity[2],
        verify_ambiguity_extra_rows: ambiguity[3],
        verify_ambiguity_mismatched_rows: ambiguity[4],
        verify_country_year_expected_rows: country_year[0],
        verify_country_year_actual_rows: country_year[1],
        verify_country_year_missing_rows: country_year[2],
        verify_country_year_extra_rows: country_year[3],
        verify_country_year_mismatched_rows: country_year[4],
        verify_institution_year_expected_rows: institution_year[0],
        verify_institution_year_actual_rows: institution_year[1],
        verify_institution_year_missing_rows: institution_year[2],
        verify_institution_year_extra_rows: institution_year[3],
        verify_institution_year_mismatched_rows: institution_year[4],
    };
    let updated: RunRow = run_statement(c, &page, Site::RunUpdate(Stage::Page), |c| {
        let target = run_table::table.filter(run_table::run_id.eq(run_id));
        if reached_end {
            diesel::update(target)
                .set((
                    &changes,
                    run_table::updated_at.eq(statement_time()),
                    run_table::completed_at.eq(statement_time_nullable()),
                ))
                .returning(RunRow::as_returning())
                .get_result(c)
        } else {
            diesel::update(target)
                .set((&changes, run_table::updated_at.eq(statement_time())))
                .returning(RunRow::as_returning())
                .get_result(c)
        }
    })?;
    Ok(Outcome::Committed(Box::new(Committed {
        run: updated,
        frontier,
        replaced: false,
        units: range.units,
        month_keys: verified.month_keys,
        work_day_rows: verified.work_day_rows,
        missing: verified.missing_rows(),
        extra: verified.extra_rows(),
        mismatched: verified.mismatched_rows(),
        first_unit: range.first_unit,
        last_unit: range.last_unit,
    })))
}

// ---------------------------------------------------------------------------
// Logging (section 12)
// ---------------------------------------------------------------------------

fn frontier_text(frontier: Option<i64>) -> String {
    frontier.map_or_else(|| "none".to_string(), |frontier| frontier.to_string())
}

/// The line of one committed step or page, and its range line when it found
/// missing, extra or mismatched rows. Range identities go to server logs
/// only, never to GraphQL.
fn log_commit(
    mode: MetricRollupMonthRebuildMode,
    phase: &'static str,
    committed: &Committed,
    run: &MetricRollupMonthRebuildRun,
    hold_ms: u128,
) {
    let outcome = match (phase, committed.replaced) {
        ("verify", _) => "verified",
        (_, true) => "replaced",
        (_, false) => "exact",
    };
    log::info!(
        "metric_rollup_rebuild run={} mode={} phase={} step={} outcome={} units={} \
         month_keys={} work_day_rows={} missing={} extra={} mismatched={} frontier={} \
         position_permille={} hold_ms={}",
        run.run_id,
        mode.as_str(),
        phase,
        run.step_count,
        outcome,
        committed.units,
        committed.month_keys,
        committed.work_day_rows,
        committed.missing,
        committed.extra,
        committed.mismatched,
        committed.frontier,
        run.position_permille,
        hold_ms,
    );
    if committed.missing != 0 || committed.extra != 0 || committed.mismatched != 0 {
        log::info!(
            "metric_rollup_rebuild run={} phase={} step={} range_first={} range_last={}",
            run.run_id,
            phase,
            run.step_count,
            Unit::log_text(committed.first_unit),
            Unit::log_text(committed.last_unit),
        );
    }
}

/// The line of one refused step or page.
fn log_refusal(
    run_id: Uuid,
    phase: &'static str,
    step: i64,
    error: MetricRollupMonthRebuildError,
    range_first: Option<Unit>,
    hold_ms: u128,
) {
    let Some(refused) = error.refused() else {
        return;
    };
    log::warn!(
        "metric_rollup_rebuild run={} phase={} step={} refused={} reason={} range_first={} \
         hold_ms={}",
        run_id,
        phase,
        step,
        refused,
        error.reason(),
        Unit::log_text(range_first),
        hold_ms,
    );
}

/// The status line of a run that reached a terminal status or was
/// abandoned.
fn log_status(row: &RunRow) {
    let line = format!(
        "metric_rollup_rebuild run={} status={} steps={} verify_first_frontier={} \
         verify_last_frontier={}",
        row.run_id,
        row.status,
        row.step_count,
        frontier_text(row.verify_first_frontier),
        frontier_text(row.verify_last_frontier),
    );
    if row.status == MetricRollupMonthRebuildStatus::Mismatched.as_str() {
        log::warn!("{line}");
    } else {
        log::info!("{line}");
    }
}

// ---------------------------------------------------------------------------
// Test support
// ---------------------------------------------------------------------------

/// Size one range from `lo` in its own read-only transaction, without any
/// budget clock: the sizing loop exactly as a step or page runs it.
#[cfg(test)]
pub(crate) fn size_range_for_test(
    db: &PgPool,
    lo: Option<Unit>,
    budgets: SizeBudgets,
) -> Result<SizedRange, MetricRollupMonthRebuildError> {
    let mut connection = checkout(db)?;
    transact(&mut connection, Isolation::ReadCommittedReadOnly, |c| {
        let clock = Clock::start(Duration::from_secs(600));
        size_range(c, &clock, Stage::Presize, lo, budgets, None)
    })
}
