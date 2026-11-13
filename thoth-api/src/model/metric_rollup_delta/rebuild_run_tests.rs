//! `MET-WP4-03A-OPS-03`: resumable monthly rebuild and paged verification.
//!
//! The tests of the specification's section 16, numbered as there, and the
//! ignored evidence harness of its section 17. The step and page budgets are
//! injected through [`advance_metric_rollup_month_rebuild_within`], as the
//! OPS-02 `_within` entry point does; production code reads only
//! [`RunBudgets::PRODUCTION`].

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Barrier, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use chrono::{Datelike, NaiveDate};
use diesel::connection::InstrumentationEvent;
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, CustomizeConnection, Pool};
use diesel::result::{DatabaseErrorInformation, DatabaseErrorKind, Error as DieselError};
use diesel::sql_types::{Array, Date, Nullable, Text, Uuid as SqlUuid};
use diesel::{sql_query, Connection, QueryableByName, RunQueryDsl};
use diesel_migrations::MigrationHarness;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::crud::{
    affected_year_keys, recompute_month_projections, recompute_year_projections, MonthKey, YearKey,
    MONTH_DELETE_STATEMENTS, REBUILD_MONTH_KEYS_SQL, YEAR_COUNTRY_DELETE_SQL,
    YEAR_INSTITUTION_DELETE_SQL,
};
use super::rebuild_run::{
    abandon_metric_rollup_month_rebuild, advance_metric_rollup_month_rebuild_within, classify,
    hooks, metric_rollup_month_rebuild_run, parse_expected_step_count, range_bounds, reason,
    server_message, size_range_for_test, start_metric_rollup_month_rebuild,
    MetricRollupMonthRebuildError, RunBudgets, RunRow, Site, SizeBudgets, SizedRange, Stage, Unit,
    ISOLATION_SQL, LOCK_STATE_SQL, LOCK_WORKS_SQL, METRIC_ROLLUP_RUN_GRANT_AGGREGATE_SQL,
    ONE_OPEN_RUN_INDEX, PAGE_FRONTIER_SQL, PAGE_LOCK_SQL, PER_YEAR_SQL, RANGE_DELETE_STATEMENTS,
    RANGE_MONTH_KEYS_SQL, RUN_LOCK_SQL, RUN_READ_SQL, SIZING_SQL,
};
use super::tests::{
    apply_everything, assert_exact_verification, backends_waiting_on_a_lock, captured,
    commit_cell_delta, exec, exec_count, insert_extra_institution, insert_extra_publication,
    logging_pool, month_rows, month_state, oracle_month_state, pausing_pool, projection, rebuild,
    relation_locks, state, table_digest, verification, wait_for_lock_evidence, wide_pool,
    year_state, CellDimensions, HeldTransaction, DERIVED_TABLES,
};
use super::verification::{
    verify_range, RangeVerification, RANGE_VERIFICATION_SQL, VERIFICATION_SQL,
};
use super::{
    AbandonMetricRollupMonthRebuildInput, AdvanceMetricRollupMonthRebuildInput,
    MetricRollupMonthProjectionVerification, MetricRollupMonthRebuildAdvance,
    MetricRollupMonthRebuildMode, MetricRollupMonthRebuildRun, MetricRollupMonthRebuildStatus,
    StartMetricRollupMonthRebuildInput, METRIC_ROLLUP_REBUILD_MAX_MONTH_KEYS,
    METRIC_ROLLUP_RUN_CHECKOUT_TIMEOUT, METRIC_ROLLUP_RUN_PAGE_BUDGET,
    METRIC_ROLLUP_RUN_PAGE_MAX_MONTH_KEYS, METRIC_ROLLUP_RUN_PAGE_MAX_WORK_DAY_ROWS,
    METRIC_ROLLUP_RUN_SIZING_BATCH_WORKS, METRIC_ROLLUP_RUN_STEP_CHUNK_KEYS,
    METRIC_ROLLUP_RUN_STEP_HOLD_BUDGET, METRIC_ROLLUP_RUN_STEP_LOCK_BUDGET,
    METRIC_ROLLUP_RUN_STEP_MAX_MONTH_KEYS, METRIC_ROLLUP_RUN_STEP_MAX_REPLACING_READ,
    METRIC_ROLLUP_RUN_STEP_MAX_WORK_DAY_ROWS, METRIC_ROLLUP_RUN_STEP_PRESIZE_BUDGET,
};
use crate::db::{PgPool, MIGRATIONS};
use crate::model::metric_platform::tests::{scalar_i64, setup_registry_db};
use crate::model::metric_record::tests::RecordFixture;
use crate::model::metric_record_revision::tests::fixture_record;
use crate::model::tests::db::{test_db_url, TestDbGuard};

use MetricRollupMonthRebuildMode::{Rebuild, Verify};
use MetricRollupMonthRebuildStatus::{
    Abandoned, Building, Mismatched, Reconciled, Verified, Verifying,
};

type Error = MetricRollupMonthRebuildError;
type Run = MetricRollupMonthRebuildRun;

/// The sha256 of the rendered `VERIFICATION_SQL` at the specification's
/// base (`develop` at `0740f6c0`), and its length in bytes.
const VERIFICATION_SQL_SHA256: &str =
    "c9348971bb1ccb175fe700ef53d882d62482334f4b5ed380c69170c9c061e853";
const VERIFICATION_SQL_BYTES: usize = 13_301;

/// The migration this stage adds.
const MIGRATION_VERSION: &str = "20261112";

const RUN_TABLE: &str = "metric_rollup_month_rebuild_run";

const STARTED_BY: &str = "metrics-rebuild-test-service";

/// The step's run-row update as Diesel renders it.
const RUN_UPDATE_PREFIX: &str = "UPDATE \"metric_rollup_month_rebuild_run\"";

// ===========================================================================
// Shared helpers
// ===========================================================================

/// A fresh registry database with every hook of this thread disarmed.
fn setup() -> (TestDbGuard, Arc<PgPool>) {
    hooks::clear();
    setup_registry_db()
}

/// Production budgets with smaller key budgets, so a small fixture takes
/// several steps and pages.
fn keys(month_keys: i64) -> RunBudgets {
    RunBudgets {
        step_max_month_keys: month_keys,
        page_max_month_keys: month_keys,
        ..RunBudgets::PRODUCTION
    }
}

/// Short time budgets, so failure and bound tests run quickly.
fn short(month_keys: i64) -> RunBudgets {
    RunBudgets {
        step_presize: Duration::from_millis(1_000),
        step_lock: Duration::from_millis(600),
        step_hold: Duration::from_millis(1_200),
        page: Duration::from_millis(1_500),
        ..keys(month_keys)
    }
}

fn start(pool: &PgPool, mode: MetricRollupMonthRebuildMode) -> Run {
    start_metric_rollup_month_rebuild(
        pool,
        &StartMetricRollupMonthRebuildInput { mode },
        STARTED_BY,
    )
    .expect("start")
}

fn advance_input(run_id: Uuid, expected: i64) -> AdvanceMetricRollupMonthRebuildInput {
    AdvanceMetricRollupMonthRebuildInput {
        run_id,
        expected_step_count: expected.to_string(),
    }
}

fn try_advance(
    pool: &PgPool,
    run: &Run,
    budgets: &RunBudgets,
) -> Result<MetricRollupMonthRebuildAdvance, Error> {
    advance_metric_rollup_month_rebuild_within(
        pool,
        &advance_input(run.run_id, run.step_count),
        budgets,
    )
}

/// One committed step or page.
fn advance(pool: &PgPool, run: &Run, budgets: &RunBudgets) -> Run {
    let advanced = try_advance(pool, run, budgets).unwrap_or_else(|error| {
        panic!(
            "advance from step {}: {} ({})",
            run.step_count,
            error.code(),
            error.reason()
        )
    });
    assert!(
        advanced.advanced,
        "the advance from step {} committed nothing",
        run.step_count
    );
    advanced.run
}

/// Advance until the run is terminal.
fn drive(pool: &PgPool, mut run: Run, budgets: &RunBudgets) -> Run {
    for _ in 0..100_000 {
        if !run.status.is_open() {
            return run;
        }
        run = advance(pool, &run, budgets);
    }
    panic!("the run did not end");
}

/// Advance until the build phase has ended.
fn drive_build(pool: &PgPool, mut run: Run, budgets: &RunBudgets) -> Run {
    while run.status == Building {
        run = advance(pool, &run, budgets);
    }
    run
}

fn abandon(pool: &PgPool, run_id: Uuid) -> Result<Run, Error> {
    abandon_metric_rollup_month_rebuild(pool, &AbandonMetricRollupMonthRebuildInput { run_id })
}

/// The stored run row, cursor and principal included.
fn run_row(pool: &PgPool, run_id: Uuid) -> RunRow {
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query("SELECT * FROM metric_rollup_month_rebuild_run WHERE run_id = $1")
        .bind::<SqlUuid, _>(run_id)
        .get_result::<RunRow>(&mut connection)
        .expect("the run row")
}

fn cursor(pool: &PgPool, run_id: Uuid) -> Option<Unit> {
    run_row(pool, run_id)
        .cursor()
        .expect("a well-formed cursor")
}

fn text_scalar(pool: &PgPool, query: &str) -> Option<String> {
    #[derive(QueryableByName)]
    struct Value {
        #[diesel(sql_type = Nullable<Text>)]
        value: Option<String>,
    }
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(format!("SELECT ({query})::text AS value"))
        .get_result::<Value>(&mut connection)
        .expect("a scalar")
        .value
}

/// `(ctid, xmin)` of every row of the six projection tables: any write to a
/// row, even of identical values, changes them.
fn fingerprints(pool: &PgPool) -> Vec<(String, Option<String>)> {
    DERIVED_TABLES
        .iter()
        .map(|table| {
            (
                table.to_string(),
                text_scalar(
                    pool,
                    &format!(
                        "SELECT string_agg(ctid::text || '/' || xmin::text, ',' ORDER BY ctid) \
                         FROM public.{table}"
                    ),
                ),
            )
        })
        .collect()
}

/// Order-independent digests of every row of the six projection tables,
/// surrogate ids included.
fn digests(pool: &PgPool) -> Vec<Option<String>> {
    DERIVED_TABLES
        .iter()
        .map(|table| table_digest(pool, table))
        .collect()
}

fn truncate_projections(pool: &PgPool) {
    exec(
        pool,
        &format!("TRUNCATE TABLE {}", DERIVED_TABLES.join(", ")),
    );
}

/// The range verifier over `(lo, hi]`.
fn verify(pool: &PgPool, lo: Option<Unit>, hi: Option<Unit>) -> RangeVerification {
    let mut connection = pool.get().expect("Failed to get DB connection");
    verify_range(&mut connection, &range_bounds(lo, hi)).expect("range verification")
}

fn family_counts(family: &MetricRollupMonthProjectionVerification) -> [i64; 5] {
    [
        family.expected_rows,
        family.actual_rows,
        family.missing_rows,
        family.extra_rows,
        family.mismatched_rows,
    ]
}

fn range_families(verification: &RangeVerification) -> [[i64; 5]; 6] {
    let families = verification.families();
    std::array::from_fn(|index| family_counts(families[index]))
}

/// The OPS-02 whole-table verification's per-family counts.
fn whole_families(pool: &PgPool) -> [[i64; 5]; 6] {
    let whole = verification(pool);
    [
        family_counts(&whole.total),
        family_counts(&whole.country),
        family_counts(&whole.institution),
        family_counts(&whole.ambiguity),
        family_counts(&whole.country_year),
        family_counts(&whole.institution_year),
    ]
}

fn run_families(run: &Run) -> [[i64; 5]; 6] {
    let counts = &run.verify.counts;
    [
        family_counts(&counts.total),
        family_counts(&counts.country),
        family_counts(&counts.institution),
        family_counts(&counts.ambiguity),
        family_counts(&counts.country_year),
        family_counts(&counts.institution_year),
    ]
}

fn day(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).expect("a valid date")
}

fn jan1(year: i32) -> NaiveDate {
    day(year, 1, 1)
}

fn unit(work_id: Uuid, year: i32) -> Unit {
    Unit {
        work_id,
        year_start: jan1(year),
    }
}

// ---------------------------------------------------------------------------
// Fixtures written directly to the work-day projection
// ---------------------------------------------------------------------------

/// A work id whose first byte orders it: `controlled_work(1)` sorts before
/// `controlled_work(2)`, and both are version-4 shaped.
fn controlled_work(n: u8) -> Uuid {
    Uuid::from_u128((u128::from(n) << 120) | 0x0000_0000_0000_4000_8000_0000_0000_0000)
}

/// The registry, institutions and imprint the direct fixtures use.
struct Corpus {
    fixture: RecordFixture,
    institutions: [Uuid; 2],
    platforms: Vec<Uuid>,
}

fn corpus(pool: &PgPool, platforms: usize) -> Corpus {
    let (fixture, _record_id) = fixture_record(pool, "identity-base");
    let institutions = [fixture.institution_id, insert_extra_institution(pool)];
    let mut ids = vec![fixture.platform_id];
    for n in 1..platforms {
        let platform_id = Uuid::new_v4();
        exec(
            pool,
            &format!(
                "INSERT INTO metric_platform (platform_id, code, display_name, ownership_class, \
                 enabled) VALUES ('{platform_id}', 'ops03-test-{n}', 'OPS-03 test {n}', \
                 'EXTERNAL', true)"
            ),
        );
        ids.push(platform_id);
    }
    Corpus {
        fixture,
        institutions,
        platforms: ids,
    }
}

/// A work under the fixture imprint with one publication.
fn add_work(pool: &PgPool, corpus: &Corpus, work_id: Uuid) {
    exec(
        pool,
        &format!(
            "INSERT INTO work (work_id, work_type, work_status, imprint_id, edition) \
             SELECT '{work_id}', 'monograph', 'forthcoming', imprint_id, 1 \
             FROM work WHERE work_id = '{}'",
            corpus.fixture.work_id
        ),
    );
    insert_extra_publication(pool, work_id, "PDF");
}

/// Raise the frontier to the greatest work-day watermark.
fn settle(pool: &PgPool) -> i64 {
    exec(
        pool,
        "UPDATE metric_rollup_work_day_state SET \
             applied_through_sequence = GREATEST(applied_through_sequence, \
                 (SELECT COALESCE(MAX(watermark), 0) FROM metric_rollup_work_day)), \
             next_sequence = GREATEST(next_sequence, \
                 (SELECT COALESCE(MAX(watermark), 0) + 1 FROM metric_rollup_work_day))",
    );
    state(pool).applied_through_sequence
}

/// Insert work-day rows for every cell `(work_id, platform_id, day)` the SQL
/// `cells` returns: for each cell, a deterministic subset of the eight
/// dimension masks (publication, country, institution), so every monthly
/// and yearly family is populated, including NULL-publication rows and
/// ambiguous months. Watermarks continue above the current frontier, which
/// is then raised to cover them. Returns the frontier.
fn seed_cells(pool: &PgPool, corpus: &Corpus, cells: &str) -> i64 {
    let base = state(pool).applied_through_sequence.max(scalar_i64(
        pool,
        "(SELECT COALESCE(MAX(watermark), 0) FROM metric_rollup_work_day)",
    ));
    let [first, second] = corpus.institutions;
    exec(
        pool,
        &format!(
            "WITH cells AS ({cells}), \
                  hashed AS ( \
                      SELECT c.work_id, c.platform_id, c.day, \
                             hashtext(c.work_id::text || c.platform_id::text || c.day::text)::bigint \
                                 & 2147483647 AS h \
                      FROM cells c), \
                  chosen AS ( \
                      SELECT x.work_id, x.platform_id, x.day, x.h, m.mask \
                      FROM hashed x CROSS JOIN generate_series(0, 7) AS m(mask) \
                      WHERE ((x.h >> m.mask) & 1) = 1 OR m.mask = x.h % 8) \
             INSERT INTO metric_rollup_work_day (work_id, publication_id, platform_id, \
                 measure_id, day, country_code, institution_id, value, watermark) \
             SELECT ch.work_id, \
                    CASE WHEN ch.mask & 4 <> 0 THEN (SELECT p.publication_id FROM publication p \
                         WHERE p.work_id = ch.work_id ORDER BY p.publication_id LIMIT 1) END, \
                    ch.platform_id, '{measure}', ch.day, \
                    CASE WHEN ch.mask & 2 <> 0 \
                         THEN (ARRAY['GB','US','DE'])[1 + ((ch.h + ch.mask) % 3)::int] END, \
                    CASE WHEN ch.mask & 1 <> 0 \
                         THEN (ARRAY['{first}'::uuid, '{second}'::uuid]) \
                              [1 + ((ch.h / 7 + ch.mask) % 2)::int] END, \
                    1 + (ch.h + ch.mask) % 97, \
                    {base} + row_number() OVER (ORDER BY ch.day, ch.work_id, ch.platform_id, \
                                                         ch.mask) \
             FROM chosen ch",
            measure = corpus.fixture.measure_id,
        ),
    );
    settle(pool)
}

/// The cells of `units`, each holding `days` of `months` on every platform.
fn cells(corpus: &Corpus, units: &[(Uuid, i32)], months: &[u32], days: &[u32]) -> String {
    let mut values = Vec::new();
    for (work_id, year) in units {
        for month in months {
            for day_of_month in days {
                for platform_id in &corpus.platforms {
                    values.push(format!(
                        "('{work_id}'::uuid, '{platform_id}'::uuid, \
                         DATE '{year:04}-{month:02}-{day_of_month:02}')"
                    ));
                }
            }
        }
    }
    format!(
        "SELECT * FROM (VALUES {}) AS v(work_id, platform_id, day)",
        values.join(", ")
    )
}

/// The layered fixture most tests share: five works in order, over three
/// years with gaps (work 0x40 has no 2024 rows), three months a year, two
/// days a month, two series. Returns the corpus, the works and the frontier.
fn layered(pool: &PgPool) -> (Corpus, Vec<Uuid>, i64) {
    let corpus = corpus(pool, 2);
    let works: Vec<Uuid> = [0x20, 0x40, 0x60, 0x80, 0xA0]
        .into_iter()
        .map(controlled_work)
        .collect();
    for work_id in &works {
        add_work(pool, &corpus, *work_id);
    }
    let units = [
        (works[0], 2023),
        (works[0], 2024),
        (works[0], 2025),
        (works[1], 2023),
        (works[1], 2025),
        (works[2], 2024),
        (works[3], 2023),
        (works[3], 2024),
        (works[4], 2025),
    ];
    let frontier = seed_cells(
        pool,
        &corpus,
        &cells(&corpus, &units, &[1, 6, 12], &[1, 15]),
    );
    (corpus, works, frontier)
}

/// The layered fixture with its projections built by the OPS-02 rebuild,
/// and the logical state it built.
fn built_layered(pool: &PgPool) -> (Corpus, Vec<Uuid>, i64) {
    let (corpus, works, frontier) = layered(pool);
    assert!(rebuild(pool, frontier).expect("the OPS-02 rebuild").rebuilt);
    assert!(verification(pool).matches);
    (corpus, works, frontier)
}

/// Commit one canonical `DAY` delta and apply it through the real claim and
/// completion: the completion path, not a direct write.
fn complete_one(pool: &PgPool, corpus: &Corpus, work_id: Uuid, on: NaiveDate, value: i64) -> i64 {
    commit_cell_delta(
        pool,
        &corpus.fixture,
        work_id,
        on,
        CellDimensions {
            publication_id: None,
            country_code: Some("GB"),
            institution_id: None,
        },
        value,
    );
    apply_everything(pool)
}

// ---------------------------------------------------------------------------
// Statement logs
// ---------------------------------------------------------------------------

/// Every statement a pooled connection starts or finishes, timed.
#[derive(Debug)]
struct TimedLog(Arc<Mutex<Vec<(Instant, bool, String)>>>);

impl CustomizeConnection<PgConnection, diesel::r2d2::Error> for TimedLog {
    fn on_acquire(&self, connection: &mut PgConnection) -> Result<(), diesel::r2d2::Error> {
        let log = Arc::clone(&self.0);
        connection.set_instrumentation(move |event: InstrumentationEvent<'_>| match event {
            InstrumentationEvent::StartQuery { query, .. } => log
                .lock()
                .expect("timed log")
                .push((Instant::now(), false, query.to_string())),
            InstrumentationEvent::FinishQuery { query, .. } => log
                .lock()
                .expect("timed log")
                .push((Instant::now(), true, query.to_string())),
            _ => {}
        });
        Ok(())
    }
}

type Timed = Arc<Mutex<Vec<(Instant, bool, String)>>>;

/// A one-connection pool whose statements are captured with their start and
/// finish times.
fn timed_pool() -> (Arc<PgPool>, Timed) {
    let log = Arc::new(Mutex::new(Vec::new()));
    let pool = Pool::builder()
        .max_size(1)
        .connection_customizer(Box::new(TimedLog(Arc::clone(&log))))
        .build(ConnectionManager::<PgConnection>::new(test_db_url()))
        .expect("Failed to create the timed pool");
    (Arc::new(pool), log)
}

/// When the last statement starting with `prefix` finished.
fn finished(log: &Timed, prefix: &str) -> Option<Instant> {
    log.lock()
        .expect("timed log")
        .iter()
        .filter(|(_, done, text)| *done && text.starts_with(prefix))
        .map(|(at, _, _)| *at)
        .next_back()
}

fn is_transaction_control(statement: &str) -> bool {
    let upper = statement.trim_start().to_uppercase();
    ["BEGIN", "COMMIT", "ROLLBACK", "SAVEPOINT", "RELEASE"]
        .iter()
        .any(|keyword| upper.starts_with(keyword))
}

fn is_set_local(statement: &str) -> bool {
    statement.starts_with("SET LOCAL ")
}

/// The statement text without Diesel's bind suffix.
fn text_of(statement: &str) -> &str {
    statement
        .split_once(" -- binds:")
        .map_or(statement, |(text, _)| text)
}

/// The value of a `SET LOCAL statement_timeout = <ms>` line.
fn timeout_value(statement: &str) -> Option<u128> {
    text_of(statement)
        .strip_prefix("SET LOCAL statement_timeout = ")
        .and_then(|value| value.trim().parse().ok())
}

// ===========================================================================
// 2. Migration
// ===========================================================================

/// Every schema object in `public`, one line each.
fn schema_snapshot(pool: &PgPool) -> Vec<String> {
    #[derive(QueryableByName)]
    struct Line {
        #[diesel(sql_type = Text)]
        line: String,
    }
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(
        "SELECT line FROM ( \
             SELECT 'rel ' || c.relname::text || ' ' || c.relkind::text AS line \
             FROM pg_class c WHERE c.relnamespace = 'public'::regnamespace \
             UNION ALL \
             SELECT 'con ' || conrelid::regclass::text || ' ' || conname::text || ' ' \
                    || pg_get_constraintdef(oid) \
             FROM pg_constraint WHERE connamespace = 'public'::regnamespace \
             UNION ALL \
             SELECT 'idx ' || indexdef FROM pg_indexes WHERE schemaname = 'public' \
             UNION ALL \
             SELECT 'trg ' || tgrelid::regclass::text || ' ' || tgname::text FROM pg_trigger \
             WHERE NOT tgisinternal \
             UNION ALL \
             SELECT 'fn ' || p.oid::regprocedure::text || ' ' || md5(p.prosrc) FROM pg_proc p \
             WHERE p.pronamespace = 'public'::regnamespace \
             UNION ALL \
             SELECT 'col ' || table_name::text || '.' || column_name::text || ' ' \
                    || data_type::text || ' ' || is_nullable::text || ' ' \
                    || COALESCE(column_default::text, '') \
             FROM information_schema.columns WHERE table_schema = 'public' \
             UNION ALL \
             SELECT 'typ ' || t.typname::text FROM pg_type t \
             WHERE t.typnamespace = 'public'::regnamespace \
         ) s ORDER BY line",
    )
    .load::<Line>(&mut connection)
    .expect("the schema snapshot")
    .into_iter()
    .map(|line| line.line)
    .collect()
}

type RunValues = BTreeMap<&'static str, String>;

/// A run row of `mode` and `status` with every required column.
fn run_values(mode: &str, status: &str) -> RunValues {
    let mut values = RunValues::new();
    values.insert("mode", format!("'{mode}'"));
    values.insert("status", format!("'{status}'"));
    values.insert("started_by", "'migration-test'".to_string());
    values.insert("started_at", "now()".to_string());
    values.insert("updated_at", "now()".to_string());
    values.insert("start_applied_through_sequence", "5".to_string());
    if !matches!(status, "BUILDING" | "VERIFYING") {
        values.insert("completed_at", "now()".to_string());
    }
    values
}

fn with(mut values: RunValues, overrides: &[(&'static str, &str)]) -> RunValues {
    for (column, value) in overrides {
        values.insert(column, (*value).to_string());
    }
    values
}

fn without(mut values: RunValues, columns: &[&str]) -> RunValues {
    for column in columns {
        values.remove(*column);
    }
    values
}

/// Insert one run row; on failure, the violated constraint's name.
fn insert_run(pool: &PgPool, values: &RunValues) -> Result<(), String> {
    let columns: Vec<&str> = values.keys().copied().collect();
    let literals: Vec<&str> = values.values().map(String::as_str).collect();
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(format!(
        "INSERT INTO metric_rollup_month_rebuild_run ({}) VALUES ({})",
        columns.join(", "),
        literals.join(", ")
    ))
    .execute(&mut connection)
    .map(|_| ())
    .map_err(|error| match error {
        DieselError::DatabaseError(_, info) => {
            info.constraint_name().unwrap_or(info.message()).to_string()
        }
        other => other.to_string(),
    })
}

fn clear_runs(pool: &PgPool) {
    exec(pool, "DELETE FROM metric_rollup_month_rebuild_run");
}

/// The rows the section 5 checks must accept, open rows one at a time.
fn accepted_rows() -> Vec<(&'static str, RunValues)> {
    let cursor: &[(&str, &str)] = &[
        ("cursor_work_id", "'00000000-0000-4000-8000-000000000001'"),
        ("cursor_year_start", "DATE '2024-01-01'"),
    ];
    vec![
        ("building, from empty", run_values("REBUILD", "BUILDING")),
        (
            "building, mid-build",
            with(
                with(run_values("REBUILD", "BUILDING"), cursor),
                &[
                    ("step_count", "2"),
                    ("build_steps", "2"),
                    ("build_replaced_steps", "1"),
                    ("build_units", "3"),
                    ("build_first_frontier", "5"),
                    ("build_last_frontier", "7"),
                ],
            ),
        ),
        ("verifying, VERIFY", run_values("VERIFY", "VERIFYING")),
        (
            "verifying, REBUILD",
            with(
                run_values("REBUILD", "VERIFYING"),
                &[
                    ("build_completed_at", "now()"),
                    ("step_count", "1"),
                    ("build_steps", "1"),
                    ("build_first_frontier", "5"),
                    ("build_last_frontier", "5"),
                ],
            ),
        ),
        (
            "reconciled, VERIFY",
            with(
                run_values("VERIFY", "RECONCILED"),
                &[
                    ("step_count", "1"),
                    ("verify_pages", "1"),
                    ("verify_first_frontier", "5"),
                    ("verify_last_frontier", "5"),
                ],
            ),
        ),
        (
            "verified, VERIFY",
            with(
                run_values("VERIFY", "VERIFIED"),
                &[
                    ("step_count", "2"),
                    ("verify_pages", "2"),
                    ("verify_first_frontier", "5"),
                    ("verify_last_frontier", "6"),
                ],
            ),
        ),
        (
            "mismatched, VERIFY",
            with(
                run_values("VERIFY", "MISMATCHED"),
                &[
                    ("step_count", "1"),
                    ("verify_pages", "1"),
                    ("verify_first_frontier", "5"),
                    ("verify_last_frontier", "5"),
                    ("verify_country_missing_rows", "1"),
                ],
            ),
        ),
        (
            "abandoned VERIFY run with pages",
            with(
                with(run_values("VERIFY", "ABANDONED"), cursor),
                &[
                    ("step_count", "1"),
                    ("verify_pages", "1"),
                    ("verify_units", "2"),
                    ("verify_total_expected_rows", "3"),
                    ("verify_total_actual_rows", "3"),
                    ("verify_first_frontier", "5"),
                    ("verify_last_frontier", "5"),
                ],
            ),
        ),
        (
            "abandoned REBUILD run during its build",
            with(
                with(run_values("REBUILD", "ABANDONED"), cursor),
                &[
                    ("step_count", "1"),
                    ("build_steps", "1"),
                    ("build_first_frontier", "5"),
                    ("build_last_frontier", "5"),
                ],
            ),
        ),
        (
            "abandoned REBUILD run during its verification",
            with(
                with(run_values("REBUILD", "ABANDONED"), cursor),
                &[
                    ("build_completed_at", "now()"),
                    ("step_count", "2"),
                    ("build_steps", "1"),
                    ("build_first_frontier", "5"),
                    ("build_last_frontier", "5"),
                    ("verify_pages", "1"),
                    ("verify_first_frontier", "5"),
                    ("verify_last_frontier", "6"),
                ],
            ),
        ),
        (
            "reconciled, REBUILD",
            with(
                run_values("REBUILD", "RECONCILED"),
                &[
                    ("build_completed_at", "now()"),
                    ("step_count", "3"),
                    ("build_steps", "2"),
                    ("build_first_frontier", "5"),
                    ("build_last_frontier", "6"),
                    ("verify_pages", "1"),
                    ("verify_first_frontier", "6"),
                    ("verify_last_frontier", "6"),
                ],
            ),
        ),
    ]
}

/// One row per check of section 5, each violating exactly that check.
fn violating_rows() -> Vec<(&'static str, RunValues)> {
    let reconciled = with(
        run_values("VERIFY", "RECONCILED"),
        &[
            ("step_count", "1"),
            ("verify_pages", "1"),
            ("verify_first_frontier", "5"),
            ("verify_last_frontier", "5"),
        ],
    );
    let verified = with(
        run_values("VERIFY", "VERIFIED"),
        &[
            ("step_count", "2"),
            ("verify_pages", "2"),
            ("verify_first_frontier", "5"),
            ("verify_last_frontier", "6"),
        ],
    );
    let mismatched = with(
        run_values("VERIFY", "MISMATCHED"),
        &[
            ("step_count", "1"),
            ("verify_pages", "1"),
            ("verify_first_frontier", "5"),
            ("verify_last_frontier", "5"),
        ],
    );
    let verifying_rebuild = with(
        run_values("REBUILD", "VERIFYING"),
        &[
            ("build_completed_at", "now()"),
            ("step_count", "1"),
            ("build_steps", "1"),
            ("build_first_frontier", "5"),
            ("build_last_frontier", "5"),
        ],
    );
    vec![
        (
            "mode",
            with(run_values("VERIFY", "VERIFYING"), &[("mode", "'OTHER'")]),
        ),
        ("status", run_values("VERIFY", "PAUSED")),
        (
            "cursor",
            with(
                run_values("VERIFY", "VERIFYING"),
                &[("cursor_work_id", "'00000000-0000-4000-8000-000000000001'")],
            ),
        ),
        (
            "cursor",
            with(
                run_values("VERIFY", "VERIFYING"),
                &[
                    ("cursor_work_id", "'00000000-0000-4000-8000-000000000001'"),
                    ("cursor_year_start", "DATE '2024-02-01'"),
                ],
            ),
        ),
        (
            "terminal_cursor",
            with(
                reconciled.clone(),
                &[
                    ("cursor_work_id", "'00000000-0000-4000-8000-000000000001'"),
                    ("cursor_year_start", "DATE '2024-01-01'"),
                ],
            ),
        ),
        (
            "abandoned_build",
            with(
                run_values("REBUILD", "ABANDONED"),
                &[
                    ("step_count", "2"),
                    ("build_steps", "1"),
                    ("build_first_frontier", "5"),
                    ("build_last_frontier", "5"),
                    ("verify_pages", "1"),
                ],
            ),
        ),
        (
            "started_by",
            with(
                run_values("REBUILD", "BUILDING"),
                &[("started_by", "'   '")],
            ),
        ),
        (
            "completed_at",
            with(
                run_values("REBUILD", "BUILDING"),
                &[("completed_at", "now()")],
            ),
        ),
        (
            "completed_at",
            without(reconciled.clone(), &["completed_at"]),
        ),
        ("building", run_values("VERIFY", "BUILDING")),
        (
            "building",
            with(
                run_values("REBUILD", "BUILDING"),
                &[("step_count", "1"), ("verify_pages", "1")],
            ),
        ),
        ("rebuild_built", run_values("REBUILD", "VERIFYING")),
        (
            "verify_mode",
            with(
                run_values("VERIFY", "VERIFYING"),
                &[("step_count", "1"), ("build_steps", "1")],
            ),
        ),
        (
            "frontier_pairs",
            with(
                run_values("REBUILD", "BUILDING"),
                &[
                    ("step_count", "1"),
                    ("build_steps", "1"),
                    ("build_first_frontier", "5"),
                ],
            ),
        ),
        (
            "frontier_order",
            with(
                run_values("REBUILD", "BUILDING"),
                &[
                    ("step_count", "1"),
                    ("build_steps", "1"),
                    ("build_first_frontier", "3"),
                    ("build_last_frontier", "3"),
                ],
            ),
        ),
        (
            "terminal_verify",
            with(
                reconciled.clone(),
                &[("step_count", "0"), ("verify_pages", "0")],
            ),
        ),
        (
            "terminal_verify",
            without(
                reconciled.clone(),
                &["verify_first_frontier", "verify_last_frontier"],
            ),
        ),
        (
            "terminal_verify",
            with(
                verified.clone(),
                &[("step_count", "0"), ("verify_pages", "0")],
            ),
        ),
        (
            "terminal_verify",
            with(
                without(
                    mismatched.clone(),
                    &["verify_first_frontier", "verify_last_frontier"],
                ),
                &[("verify_total_missing_rows", "1")],
            ),
        ),
        (
            "reconciled",
            with(reconciled.clone(), &[("verify_last_frontier", "6")]),
        ),
        (
            "reconciled",
            with(reconciled, &[("verify_institution_extra_rows", "1")]),
        ),
        ("verified", with(verified, &[("verify_last_frontier", "5")])),
        ("mismatched", mismatched),
        (
            "build_completed",
            with(
                verifying_rebuild,
                &[("step_count", "0"), ("build_steps", "0")],
            ),
        ),
        (
            "counters",
            with(
                run_values("VERIFY", "VERIFYING"),
                &[("verify_total_expected_rows", "-1")],
            ),
        ),
        (
            "step_count",
            with(run_values("REBUILD", "BUILDING"), &[("step_count", "3")]),
        ),
    ]
}

#[test]
fn the_run_table_accepts_every_legal_row_and_each_check_rejects_its_violation() {
    let (_guard, pool) = setup();
    for (label, values) in accepted_rows() {
        assert_eq!(insert_run(&pool, &values), Ok(()), "{label}");
        clear_runs(&pool);
    }
    let mut checked = BTreeSet::new();
    for (name, values) in violating_rows() {
        let constraint = format!("metric_rollup_month_rebuild_run_{name}_check");
        assert_eq!(
            insert_run(&pool, &values),
            Err(constraint.clone()),
            "{name}: {values:?}"
        );
        checked.insert(constraint);
    }
    // Every named check of the table is exercised, and nothing else exists.
    let declared: BTreeSet<String> = text_scalar(
        &pool,
        "SELECT string_agg(conname, ',' ORDER BY conname) FROM pg_constraint \
         WHERE conrelid = 'public.metric_rollup_month_rebuild_run'::regclass AND contype = 'c'",
    )
    .expect("the checks")
    .split(',')
    .map(str::to_string)
    .collect();
    assert_eq!(declared.len(), 19);
    assert_eq!(checked, declared);
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM metric_rollup_month_rebuild_run)"
        ),
        0
    );
}

#[test]
fn at_most_one_run_is_open_and_terminal_runs_accumulate() {
    let (_guard, pool) = setup();
    let open = run_values("REBUILD", "BUILDING");
    assert_eq!(insert_run(&pool, &open), Ok(()));
    for second in [
        run_values("REBUILD", "BUILDING"),
        run_values("VERIFY", "VERIFYING"),
    ] {
        assert_eq!(
            insert_run(&pool, &second),
            Err(ONE_OPEN_RUN_INDEX.to_string())
        );
    }
    for (label, values) in accepted_rows() {
        let status = values["status"].clone();
        if status == "'BUILDING'" || status == "'VERIFYING'" {
            continue;
        }
        assert_eq!(insert_run(&pool, &values), Ok(()), "{label}");
    }
    let index = text_scalar(
        &pool,
        "SELECT indexdef FROM pg_indexes WHERE indexname = 'metric_rollup_month_rebuild_run_one_open_idx'",
    )
    .expect("the index");
    assert!(index.starts_with("CREATE UNIQUE INDEX"), "{index}");
    assert!(index.contains("((true))"), "{index}");
    assert!(
        index.contains("'BUILDING'") && index.contains("'VERIFYING'"),
        "{index}"
    );
    // The table has its primary key, the partial unique index and nothing
    // else: no other index, no trigger.
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_indexes WHERE tablename = 'metric_rollup_month_rebuild_run')"
        ),
        2
    );
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_trigger \
              WHERE tgrelid = 'public.metric_rollup_month_rebuild_run'::regclass)"
        ),
        0
    );
}

#[test]
fn the_migration_applies_reverts_and_reapplies_changing_no_other_schema_object() {
    let (_guard, pool) = setup();
    let applied = schema_snapshot(&pool);
    assert!(applied
        .iter()
        .any(|line| line == "rel metric_rollup_month_rebuild_run r"));
    // The run table is new and starts empty, whatever the rest of the
    // database holds.
    let (_corpus, _works, _frontier) = built_layered(&pool);
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM metric_rollup_month_rebuild_run)"
        ),
        0
    );
    let run = start(&pool, Verify);
    assert_eq!(run.status, Verifying);

    let mut connection =
        PgConnection::establish(&test_db_url()).expect("Failed to connect to the test database");
    let reverted = connection
        .revert_last_migration(MIGRATIONS)
        .expect("the migration reverts with rows in the run table");
    assert_eq!(reverted.to_string(), MIGRATION_VERSION);
    let reverted_schema = schema_snapshot(&pool);
    let expected: Vec<String> = applied
        .iter()
        .filter(|line| !line.contains(RUN_TABLE))
        .cloned()
        .collect();
    assert_eq!(
        reverted_schema, expected,
        "the revert removes the run table and its index and nothing else"
    );
    // The projections the run worked on are untouched by the revert.
    assert!(verification(&pool).matches);

    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("the migration reapplies");
    assert_eq!(schema_snapshot(&pool), applied);
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM metric_rollup_month_rebuild_run)"
        ),
        0
    );
    let run = start(&pool, Verify);
    let run = drive(&pool, run, &keys(1_000));
    assert_eq!(run.status, Reconciled);
}

// ===========================================================================
// 15 (classifier). Each classifier message has its own test
// ===========================================================================

struct Info {
    message: &'static str,
    constraint: Option<&'static str>,
}

impl DatabaseErrorInformation for Info {
    fn message(&self) -> &str {
        self.message
    }
    fn details(&self) -> Option<&str> {
        None
    }
    fn hint(&self) -> Option<&str> {
        None
    }
    fn table_name(&self) -> Option<&str> {
        None
    }
    fn column_name(&self) -> Option<&str> {
        None
    }
    fn constraint_name(&self) -> Option<&str> {
        self.constraint
    }
    fn statement_position(&self) -> Option<i32> {
        None
    }
}

fn database_error(
    kind: DatabaseErrorKind,
    message: &'static str,
    constraint: Option<&'static str>,
) -> DieselError {
    DieselError::DatabaseError(
        kind,
        Box::new(Info {
            message,
            constraint,
        }),
    )
}

#[test]
fn a_serialization_failure_is_retryable() {
    let error = database_error(
        DatabaseErrorKind::SerializationFailure,
        "could not serialize access due to concurrent update",
        None,
    );
    assert_eq!(
        classify(&error),
        Error::Retryable(reason::SERIALIZATION_FAILURE)
    );
    assert_eq!(classify(&error).code(), "METRIC_REBUILD_RETRYABLE");
}

#[test]
fn a_closed_connection_is_retryable() {
    let error = database_error(
        DatabaseErrorKind::ClosedConnection,
        "server closed the connection unexpectedly",
        None,
    );
    assert_eq!(
        classify(&error),
        Error::Retryable(reason::CONNECTION_CLOSED)
    );
}

#[test]
fn the_one_open_run_violation_is_run_open_and_any_other_unique_violation_is_internal() {
    let open = database_error(
        DatabaseErrorKind::UniqueViolation,
        "duplicate key value violates unique constraint",
        Some(ONE_OPEN_RUN_INDEX),
    );
    assert_eq!(classify(&open), Error::RunOpen);
    assert_eq!(classify(&open).code(), "METRIC_REBUILD_RUN_OPEN");
    let other = database_error(
        DatabaseErrorKind::UniqueViolation,
        "duplicate key value violates unique constraint",
        Some("metric_rollup_month_rebuild_run_pkey"),
    );
    assert_eq!(classify(&other), Error::Internal(reason::DATABASE));
}

#[test]
fn the_lock_timeout_message_is_retryable() {
    let error = database_error(
        DatabaseErrorKind::Unknown,
        server_message::LOCK_TIMEOUT,
        None,
    );
    assert_eq!(classify(&error), Error::Retryable(reason::LOCK_TIMEOUT));
}

#[test]
fn the_statement_timeout_message_is_retryable() {
    let error = database_error(
        DatabaseErrorKind::Unknown,
        server_message::STATEMENT_TIMEOUT,
        None,
    );
    assert_eq!(
        classify(&error),
        Error::Retryable(reason::STATEMENT_TIMEOUT)
    );
}

#[test]
fn the_deadlock_message_is_retryable() {
    let error = database_error(DatabaseErrorKind::Unknown, server_message::DEADLOCK, None);
    assert_eq!(classify(&error), Error::Retryable(reason::DEADLOCK));
}

#[test]
fn the_idle_in_transaction_timeout_message_is_retryable() {
    let error = database_error(
        DatabaseErrorKind::Unknown,
        server_message::IDLE_TIMEOUT,
        None,
    );
    assert_eq!(classify(&error), Error::Retryable(reason::IDLE_TIMEOUT));
}

#[test]
fn the_bigint_overflow_message_is_blocked() {
    let error = database_error(
        DatabaseErrorKind::Unknown,
        server_message::BIGINT_OUT_OF_RANGE,
        None,
    );
    assert_eq!(classify(&error), Error::Blocked(reason::BIGINT_OVERFLOW));
    assert_eq!(classify(&error).code(), "METRIC_REBUILD_BLOCKED");
}

#[test]
fn any_other_message_or_error_is_internal_and_never_retryable() {
    for message in [
        "division by zero",
        "canceling statement due to user request",
        "Canceling statement due to lock timeout",
        "deadlock detected ",
        "integer out of range",
    ] {
        let error = database_error(DatabaseErrorKind::Unknown, message, None);
        assert_eq!(
            classify(&error),
            Error::Internal(reason::DATABASE),
            "{message}"
        );
        assert_eq!(classify(&error).code(), "INTERNAL_ERROR");
    }
    assert_eq!(
        classify(&DieselError::NotFound),
        Error::Internal(reason::DATABASE)
    );
    assert_eq!(
        classify(&DieselError::RollbackTransaction),
        Error::Internal(reason::DATABASE)
    );
}

#[test]
fn the_server_produces_exactly_the_messages_the_classifier_matches() {
    let (_guard, pool) = setup();
    // English server messages, which the exact matching assumes.
    let messages = text_scalar(&pool, "SELECT current_setting('lc_messages')").unwrap_or_default();
    assert!(
        messages.starts_with('C')
            || messages.starts_with("en")
            || messages.is_empty()
            || messages == "POSIX",
        "lc_messages = {messages}"
    );
    let mut connection = pool.get().expect("Failed to get DB connection");

    let timeout = connection.transaction::<(), DieselError, _>(|c| {
        sql_query("SET LOCAL statement_timeout = 50").execute(c)?;
        sql_query("SELECT pg_sleep(1)").execute(c)?;
        Ok(())
    });
    assert_eq!(
        classify(&timeout.expect_err("a statement timeout")),
        Error::Retryable(reason::STATEMENT_TIMEOUT)
    );

    let overflow =
        sql_query("SELECT (9223372036854775807::numeric + 1)::bigint").execute(&mut connection);
    assert_eq!(
        classify(&overflow.expect_err("an overflow")),
        Error::Blocked(reason::BIGINT_OVERFLOW)
    );

    let held = HeldTransaction::start(
        "SELECT * FROM metric_rollup_work_day_state WHERE state_id = 1 FOR UPDATE".to_string(),
    );
    let lock = connection.transaction::<(), DieselError, _>(|c| {
        sql_query("SET LOCAL lock_timeout = 50").execute(c)?;
        sql_query("SELECT * FROM metric_rollup_work_day_state WHERE state_id = 1 FOR UPDATE")
            .execute(c)?;
        Ok(())
    });
    held.commit();
    assert_eq!(
        classify(&lock.expect_err("a lock timeout")),
        Error::Retryable(reason::LOCK_TIMEOUT)
    );
}

// ===========================================================================
// 3. State machine
// ===========================================================================

#[test]
fn a_rebuild_run_walks_building_verifying_reconciled_and_a_verify_run_walks_verifying() {
    let (_guard, pool) = setup();
    let (_corpus, works, frontier) = layered(&pool);
    let budgets = keys(12);

    let run = start(&pool, Rebuild);
    assert_eq!(run.mode, Rebuild);
    assert_eq!(run.status, Building);
    assert_eq!(run.step_count, 0);
    assert_eq!(run.position_permille, 0);
    assert_eq!(run.start_applied_through_sequence, frontier);
    assert!(run.build.as_ref().is_some_and(|build| build.steps == 0));
    assert!(run.completed_at.is_none() && run.build_completed_at.is_none());
    assert_eq!(cursor(&pool, run.run_id), None);
    assert_eq!(run_row(&pool, run.run_id).started_by, STARTED_BY);

    let mut run = advance(&pool, &run, &budgets);
    assert_eq!(run.status, Building, "several steps are needed");
    assert_eq!(run.step_count, 1);
    let first = cursor(&pool, run.run_id).expect("a cursor after a non-final step");
    assert!(works.contains(&first.work_id));
    assert!(run.position_permille > 0 && run.position_permille < 1000);
    let mut positions = vec![run.position_permille];
    while run.status == Building {
        run = advance(&pool, &run, &budgets);
        positions.push(run.position_permille);
    }
    assert_eq!(run.status, Verifying);
    assert!(run.build_completed_at.is_some());
    assert_eq!(
        cursor(&pool, run.run_id),
        None,
        "the verify phase starts at minus infinity"
    );
    let build = run.build.clone().expect("a REBUILD run reports its build");
    assert_eq!(build.steps, run.step_count);
    assert_eq!(build.first_applied_through_sequence, Some(frontier));
    assert_eq!(build.last_applied_through_sequence, Some(frontier));
    let run = drive(&pool, run, &budgets);
    assert_eq!(run.status, Reconciled);
    assert_eq!(run.position_permille, 1000);
    assert!(run.completed_at.is_some());
    assert!(run.verify.has_single_frontier());
    assert_eq!(run.verify.first_applied_through_sequence, Some(frontier));
    assert_eq!(run.step_count, build.steps + run.verify.pages);
    assert!(run.verify.pages >= 2);

    let verify = start(&pool, Verify);
    assert_eq!(verify.status, Verifying);
    assert!(verify.build.is_none());
    let verify = drive(&pool, verify, &budgets);
    assert_eq!(verify.status, Reconciled);
    assert_eq!(verify.step_count, verify.verify.pages);
    assert_eq!(run_families(&verify), run_families(&run));
}

#[test]
fn a_start_while_a_run_is_open_is_refused_and_concurrent_first_arrivals_make_one_run() {
    let (_guard, pool) = setup();
    let open = start(&pool, Verify);
    for mode in [Rebuild, Verify] {
        assert_eq!(
            start_metric_rollup_month_rebuild(
                &pool,
                &StartMetricRollupMonthRebuildInput { mode },
                STARTED_BY
            ),
            Err(Error::RunOpen)
        );
    }
    abandon(&pool, open.run_id).expect("abandon");

    for _ in 0..5 {
        let pool = wide_pool();
        let barrier = Arc::new(Barrier::new(2));
        let callers: Vec<_> = [Rebuild, Verify]
            .into_iter()
            .map(|mode| {
                let pool = Arc::clone(&pool);
                let barrier = Arc::clone(&barrier);
                thread::spawn(move || {
                    barrier.wait();
                    start_metric_rollup_month_rebuild(
                        &pool,
                        &StartMetricRollupMonthRebuildInput { mode },
                        STARTED_BY,
                    )
                })
            })
            .collect();
        let results: Vec<Result<Run, Error>> = callers
            .into_iter()
            .map(|caller| caller.join().expect("join"))
            .collect();
        let started: Vec<&Run> = results.iter().filter_map(|r| r.as_ref().ok()).collect();
        assert_eq!(started.len(), 1, "{results:?}");
        assert!(results.contains(&Err(Error::RunOpen)), "{results:?}");
        assert_eq!(
            scalar_i64(
                &pool,
                "(SELECT COUNT(*) FROM metric_rollup_month_rebuild_run \
                  WHERE status IN ('BUILDING', 'VERIFYING'))"
            ),
            1
        );
        abandon(&pool, started[0].run_id).expect("abandon");
    }
}

#[test]
fn terminal_stale_and_unknown_advances_write_nothing() {
    let (_guard, pool) = setup();
    let (_corpus, _works, _frontier) = built_layered(&pool);
    let budgets = keys(12);
    let run = start(&pool, Rebuild);
    let stepped = advance(&pool, &run, &budgets);

    // A lost response retried with the old count: nothing is written.
    let before = run_row(&pool, run.run_id);
    let stale = try_advance(&pool, &run, &budgets).expect("a stale advance");
    assert!(!stale.advanced);
    assert_eq!(stale.run, stepped);
    assert_eq!(run_row(&pool, run.run_id), before);
    let ahead = advance_metric_rollup_month_rebuild_within(
        &pool,
        &advance_input(run.run_id, stepped.step_count + 1),
        &budgets,
    )
    .expect("a count from the future is stale too");
    assert!(!ahead.advanced);
    assert_eq!(run_row(&pool, run.run_id), before);

    // A terminal run.
    let done = drive(&pool, stepped, &budgets);
    let before = run_row(&pool, run.run_id);
    let fingerprint = fingerprints(&pool);
    let terminal = try_advance(&pool, &done, &budgets).expect("a terminal advance");
    assert!(!terminal.advanced);
    assert_eq!(terminal.run, done);
    assert_eq!(run_row(&pool, run.run_id), before);
    assert_eq!(fingerprints(&pool), fingerprint);

    // An abandoned run.
    let other = start(&pool, Verify);
    let abandoned = abandon(&pool, other.run_id).expect("abandon");
    let before = run_row(&pool, other.run_id);
    let advanced = try_advance(&pool, &abandoned, &budgets).expect("an abandoned advance");
    assert!(!advanced.advanced);
    assert_eq!(advanced.run.status, Abandoned);
    assert_eq!(run_row(&pool, other.run_id), before);

    // An unknown run and a malformed count are invalid requests.
    assert_eq!(
        advance_metric_rollup_month_rebuild_within(
            &pool,
            &advance_input(Uuid::new_v4(), 0),
            &budgets
        ),
        Err(Error::QueryInvalid)
    );
    assert_eq!(
        advance_metric_rollup_month_rebuild_within(
            &pool,
            &AdvanceMetricRollupMonthRebuildInput {
                run_id: run.run_id,
                expected_step_count: "-1".to_string(),
            },
            &budgets
        ),
        Err(Error::QueryInvalid)
    );
    assert_eq!(abandon(&pool, Uuid::new_v4()), Err(Error::QueryInvalid));
    assert_eq!(
        metric_rollup_month_rebuild_run(&pool, Some(Uuid::new_v4())),
        Ok(None)
    );
}

#[test]
fn abandon_works_in_every_state_and_keeps_cursor_counters_and_frontiers() {
    let (_guard, pool) = setup();
    let (_corpus, _works, frontier) = built_layered(&pool);
    let budgets = keys(12);

    // BUILDING with a NULL cursor.
    let run = start(&pool, Rebuild);
    let abandoned = abandon(&pool, run.run_id).expect("abandon");
    assert_eq!(abandoned.status, Abandoned);
    assert!(abandoned.completed_at.is_some());
    assert_eq!(abandoned.position_permille, 0);

    // BUILDING mid-build.
    let run = advance(&pool, &start(&pool, Rebuild), &budgets);
    let before = run_row(&pool, run.run_id);
    let abandoned = abandon(&pool, run.run_id).expect("abandon");
    let after = run_row(&pool, run.run_id);
    assert_eq!(after.cursor_work_id, before.cursor_work_id);
    assert_eq!(after.cursor_year_start, before.cursor_year_start);
    assert_eq!(abandoned.build, run.build);
    assert_eq!(abandoned.step_count, run.step_count);
    assert_eq!(abandoned.position_permille, run.position_permille);
    assert!(after.build_completed_at.is_none());

    // VERIFYING with pages: a VERIFY run keeps its cursor, counts and
    // frontiers.
    let run = start(&pool, Verify);
    let run = advance(&pool, &advance(&pool, &run, &budgets), &budgets);
    assert_eq!(run.status, Verifying);
    let before = run_row(&pool, run.run_id);
    let abandoned = abandon(&pool, run.run_id).expect("abandon");
    let after = run_row(&pool, run.run_id);
    assert!(after.cursor_work_id.is_some());
    assert_eq!(after.cursor_work_id, before.cursor_work_id);
    assert_eq!(after.cursor_year_start, before.cursor_year_start);
    assert_eq!(abandoned.verify, run.verify);
    assert_eq!(abandoned.verify.pages, 2);
    assert_eq!(
        abandoned.verify.first_applied_through_sequence,
        Some(frontier)
    );
    assert_eq!(
        abandoned.verify.last_applied_through_sequence,
        Some(frontier)
    );

    // VERIFYING after a REBUILD build, with a NULL cursor.
    let run = drive_build(&pool, start(&pool, Rebuild), &budgets);
    assert_eq!(run.status, Verifying);
    let abandoned = abandon(&pool, run.run_id).expect("abandon");
    assert_eq!(abandoned.build, run.build);
    assert!(abandoned.build_completed_at.is_some());

    // A terminal run is returned unchanged.
    let done = drive(&pool, start(&pool, Verify), &budgets);
    let before = run_row(&pool, done.run_id);
    assert_eq!(abandon(&pool, done.run_id), Ok(done.clone()));
    assert_eq!(run_row(&pool, done.run_id), before);
    // So is an abandoned one.
    let again = abandon(&pool, abandoned.run_id).expect("abandon again");
    assert_eq!(again, abandoned);
}

#[test]
fn the_status_read_prefers_the_open_run_and_otherwise_the_latest_started() {
    let (_guard, pool) = setup();
    let (_corpus, _works, _frontier) = built_layered(&pool);
    let budgets = keys(1_000);
    assert_eq!(metric_rollup_month_rebuild_run(&pool, None), Ok(None));

    let first = drive(&pool, start(&pool, Verify), &budgets);
    assert_eq!(first.status, Reconciled);
    assert_eq!(
        metric_rollup_month_rebuild_run(&pool, None),
        Ok(Some(first.clone()))
    );

    // A RECONCILED run stops being the latest once any later run starts.
    let second = start(&pool, Rebuild);
    assert_eq!(
        metric_rollup_month_rebuild_run(&pool, None),
        Ok(Some(second.clone()))
    );
    let second = abandon(&pool, second.run_id).expect("abandon");
    assert_eq!(
        metric_rollup_month_rebuild_run(&pool, None),
        Ok(Some(second.clone()))
    );

    // With terminal and abandoned runs present, an open run wins even over
    // a terminal run whose started_at is later.
    let third = start(&pool, Verify);
    exec(
        &pool,
        &format!(
            "UPDATE metric_rollup_month_rebuild_run SET started_at = now() + interval '1 day' \
             WHERE run_id = '{}'",
            first.run_id
        ),
    );
    assert_eq!(
        metric_rollup_month_rebuild_run(&pool, None),
        Ok(Some(third.clone()))
    );
    let third = drive(&pool, third, &budgets);
    // Without an open run, the greatest started_at is the latest.
    assert_eq!(
        metric_rollup_month_rebuild_run(&pool, None)
            .expect("status")
            .expect("a run")
            .run_id,
        first.run_id
    );
    for run in [&first, &second, &third] {
        assert_eq!(
            metric_rollup_month_rebuild_run(&pool, Some(run.run_id))
                .expect("status")
                .expect("a run")
                .run_id,
            run.run_id
        );
    }
}

/// Abandon against an in-flight step or page: either it waits behind the
/// run-row lock and wins once the step or page has committed, or it times
/// out retryable. The step or page is never half-applied.
#[test]
fn abandon_against_an_in_flight_step_or_page_waits_and_wins_or_is_retryable() {
    let (_guard, pool) = setup();
    let (_corpus, _works, frontier) = layered(&pool);
    let budgets = keys(12);
    let other = wide_pool();

    // A replacing step paused at its pre-verify, holding the run row.
    let run = start(&pool, Rebuild);
    let (paused, pause, _log) = pausing_pool(RANGE_VERIFICATION_SQL);
    let stepping = {
        let run = run.clone();
        thread::spawn(move || try_advance(&paused, &run, &budgets))
    };
    pause.wait_reached();
    let abandoning = {
        let other = Arc::clone(&other);
        thread::spawn(move || abandon(&other, run.run_id))
    };
    wait_for_lock_evidence("abandon waits on the run row", || {
        backends_waiting_on_a_lock(&pool) >= 1
    });
    pause.release();
    let stepped = stepping.join().expect("join").expect("the step commits");
    assert!(stepped.advanced);
    let abandoned = abandoning.join().expect("join").expect("abandon wins");
    assert_eq!(abandoned.status, Abandoned);
    assert_eq!(abandoned.step_count, 1);
    assert_eq!(abandoned.build, stepped.run.build);
    let row = run_row(&pool, run.run_id);
    assert!(
        row.cursor_work_id.is_some(),
        "abandoned at the cursor the step left"
    );
    let left = row.cursor().expect("cursor");
    assert!(
        verify(&pool, None, left).exact_at(frontier),
        "the step applied whole"
    );

    // A step that holds the run row longer than abandon's lock timeout:
    // abandon is retryable and the step still commits whole.
    let run = start(&pool, Rebuild);
    let slow = RunBudgets {
        step_hold: Duration::from_secs(20),
        ..budgets
    };
    let stepping = {
        let pool = Arc::clone(&other);
        let run = run.clone();
        thread::spawn(move || {
            hooks::inject(
                Site::PreVerify,
                hooks::Injection::Statement("SELECT pg_sleep(6.5)".to_string()),
            );
            let result = try_advance(&pool, &run, &slow);
            hooks::clear();
            result
        })
    };
    wait_for_lock_evidence("the slow step holds the run row", || {
        text_scalar(
            &pool,
            "SELECT count(*) FROM pg_stat_activity WHERE query = 'SELECT pg_sleep(6.5)' \
             AND state = 'active'",
        )
        .as_deref()
            == Some("1")
    });
    let started = Instant::now();
    assert_eq!(
        abandon(&pool, run.run_id),
        Err(Error::Retryable(reason::LOCK_TIMEOUT))
    );
    assert!(started.elapsed() < Duration::from_secs(6));
    let stepped = stepping
        .join()
        .expect("join")
        .expect("the slow step commits");
    assert!(stepped.advanced);
    let abandoned = abandon(&pool, run.run_id).expect("abandon after the step");
    assert_eq!(abandoned.step_count, 1);

    // A page paused after locking the run row.
    let run = drive_build(&pool, start(&pool, Rebuild), &budgets);
    let (paused, pause, _log) = pausing_pool(PAGE_FRONTIER_SQL);
    let paging = {
        let run = run.clone();
        thread::spawn(move || try_advance(&paused, &run, &budgets))
    };
    pause.wait_reached();
    let abandoning = {
        let other = Arc::clone(&other);
        thread::spawn(move || abandon(&other, run.run_id))
    };
    wait_for_lock_evidence("abandon waits on the page's run row", || {
        backends_waiting_on_a_lock(&pool) >= 1
    });
    pause.release();
    let paged = paging.join().expect("join").expect("the page commits");
    assert!(paged.advanced);
    let abandoned = abandoning.join().expect("join").expect("abandon wins");
    assert_eq!(abandoned.status, Abandoned);
    assert_eq!(abandoned.verify, paged.run.verify);
    assert_eq!(abandoned.verify.pages, 1);

    // A page that holds the run row past abandon's lock timeout.
    let run = drive_build(&pool, start(&pool, Rebuild), &budgets);
    let slow = RunBudgets {
        page: Duration::from_secs(20),
        ..budgets
    };
    let paging = {
        let pool = Arc::clone(&other);
        let run = run.clone();
        thread::spawn(move || {
            hooks::inject(
                Site::PageVerify,
                hooks::Injection::Statement("SELECT pg_sleep(6.5)".to_string()),
            );
            let result = try_advance(&pool, &run, &slow);
            hooks::clear();
            result
        })
    };
    wait_for_lock_evidence("the slow page holds the run row", || {
        text_scalar(
            &pool,
            "SELECT count(*) FROM pg_stat_activity WHERE query = 'SELECT pg_sleep(6.5)' \
             AND state = 'active'",
        )
        .as_deref()
            == Some("1")
    });
    assert_eq!(
        abandon(&pool, run.run_id),
        Err(Error::Retryable(reason::LOCK_TIMEOUT))
    );
    let paged = paging.join().expect("join").expect("the slow page commits");
    assert!(paged.advanced);
    assert_eq!(paged.run.verify.pages, 1);
    abandon(&pool, run.run_id).expect("abandon after the page");
}

// ===========================================================================
// 4. Concurrent advance
// ===========================================================================

#[test]
fn two_concurrent_build_advances_with_one_count_make_exactly_one_step() {
    let (_guard, pool) = setup();
    let (_corpus, _works, _frontier) = layered(&pool);
    let budgets = keys(12);
    let run = start(&pool, Rebuild);

    // The first step pauses after it holds the works and the state row.
    let (paused, pause, _log) = pausing_pool(RUN_LOCK_SQL);
    let first = {
        let run = run.clone();
        thread::spawn(move || try_advance(&paused, &run, &budgets))
    };
    pause.wait_reached();
    let other = wide_pool();
    let second = {
        let run = run.clone();
        thread::spawn(move || try_advance(&other, &run, &budgets))
    };
    wait_for_lock_evidence("the second step waits on the state row", || {
        backends_waiting_on_a_lock(&pool) >= 1
    });
    pause.release();
    let first = first.join().expect("join").expect("the first step");
    let second = second.join().expect("join").expect("the second call");
    assert!(first.advanced);
    assert!(!second.advanced, "the second call found the count moved");
    assert_eq!(second.run, first.run);
    assert_eq!(run_row(&pool, run.run_id).step_count, 1);
    assert_eq!(first.run.build.expect("build").steps, 1);
}

#[test]
fn two_concurrent_pages_with_one_count_make_one_page_and_one_serialization_refusal() {
    let (_guard, pool) = setup();
    let (_corpus, _works, _frontier) = built_layered(&pool);
    let budgets = keys(12);
    let run = start(&pool, Verify);

    let (paused, pause, _log) = pausing_pool(PAGE_FRONTIER_SQL);
    let first = {
        let run = run.clone();
        thread::spawn(move || try_advance(&paused, &run, &budgets))
    };
    pause.wait_reached();
    let other = wide_pool();
    let second = {
        let run = run.clone();
        thread::spawn(move || try_advance(&other, &run, &budgets))
    };
    wait_for_lock_evidence("the second page waits on the run row", || {
        backends_waiting_on_a_lock(&pool) >= 1
    });
    pause.release();
    let first = first.join().expect("join").expect("the first page");
    assert!(first.advanced);
    assert_eq!(
        second.join().expect("join"),
        Err(Error::Retryable(reason::SERIALIZATION_FAILURE))
    );
    let row = run_row(&pool, run.run_id);
    assert_eq!(row.step_count, 1);
    assert_eq!(row.verify_pages, 1);
}

// ===========================================================================
// 5. Equivalence, and 6. a healthy REBUILD
// ===========================================================================

fn publication_of(pool: &PgPool, work_id: Uuid) -> Uuid {
    text_scalar(
        pool,
        &format!(
            "SELECT publication_id FROM publication WHERE work_id = '{work_id}' \
             ORDER BY publication_id LIMIT 1"
        ),
    )
    .expect("a publication")
    .parse()
    .expect("a uuid")
}

#[test]
fn a_build_from_empty_reaches_reconciled_and_equals_the_ops02_rebuild() {
    let (_guard, pool) = setup();
    let (corpus, works, _frontier) = layered(&pool);
    // Canonical rows through the completion path too: a new unit, an
    // existing unit, every dimension shape.
    let shapes = [
        (works[1], day(2024, 3, 2), None, Some("GB"), None),
        (
            works[1],
            day(2024, 3, 2),
            None,
            None,
            Some(corpus.institutions[1]),
        ),
        (
            works[2],
            day(2024, 6, 2),
            Some(publication_of(&pool, works[2])),
            Some("US"),
            None,
        ),
        (
            works[2],
            day(2024, 6, 2),
            None,
            Some("US"),
            Some(corpus.institutions[0]),
        ),
        (works[4], day(2025, 12, 31), None, None, None),
    ];
    for (work_id, on, publication_id, country_code, institution_id) in shapes {
        commit_cell_delta(
            &pool,
            &corpus.fixture,
            work_id,
            on,
            CellDimensions {
                publication_id,
                country_code,
                institution_id,
            },
            7,
        );
    }
    let frontier = apply_everything(&pool);
    assert!(
        rebuild(&pool, frontier)
            .expect("the OPS-02 rebuild")
            .rebuilt
    );
    let months = month_state(&pool);
    let years = year_state(&pool);
    assert_eq!(months, oracle_month_state(&projection(&pool)));

    truncate_projections(&pool);
    let run = drive(&pool, start(&pool, Rebuild), &keys(12));
    assert_eq!(run.status, Reconciled);
    let build = run.build.clone().expect("build");
    assert!(build.steps >= 3);
    assert_eq!(
        build.replaced_steps, build.steps,
        "every step found missing rows"
    );
    assert_eq!(build.found_extra_rows, 0);
    assert_eq!(build.found_mismatched_rows, 0);
    assert_eq!(month_state(&pool), months);
    assert_eq!(year_state(&pool), years);
    assert_exact_verification(&pool, &verification(&pool));
    assert_eq!(run_families(&run), whole_families(&pool));
}

#[test]
fn a_healthy_rebuild_writes_no_projection_row_and_keeps_every_surrogate() {
    let (_guard, pool) = setup();
    let (_corpus, _works, frontier) = built_layered(&pool);
    let prints = fingerprints(&pool);
    let rows = digests(&pool);
    let budgets = keys(12);
    let mut run = start(&pool, Rebuild);
    while run.status == Building {
        run = advance(&pool, &run, &budgets);
        let build = run.build.clone().expect("build");
        assert_eq!(build.replaced_steps, 0, "every step was exact");
        assert_eq!(
            (
                build.found_missing_rows,
                build.found_extra_rows,
                build.found_mismatched_rows
            ),
            (0, 0, 0)
        );
        assert_eq!(fingerprints(&pool), prints, "no projection row was written");
    }
    let run = drive(&pool, run, &budgets);
    assert_eq!(run.status, Reconciled);
    assert_eq!(run.verify.first_applied_through_sequence, Some(frontier));
    assert_eq!(fingerprints(&pool), prints);
    assert_eq!(digests(&pool), rows, "values and surrogate ids unchanged");
}

// ===========================================================================
// 7. Corruption in each family
// ===========================================================================

/// One corruption: its label, the family it shows in (total, country,
/// institution, ambiguity, country_year, institution_year), its expected
/// `[missing, extra, mismatched]`, and the statement that makes it.
struct Corruption {
    label: &'static str,
    family: usize,
    counts: [i64; 3],
    statement: String,
}

fn corruptions(pool: &PgPool, works: &[Uuid], frontier: i64) -> Vec<Corruption> {
    let second_publication = insert_extra_publication(pool, works[0], "Paperback");
    let third_institution = insert_extra_institution(pool);
    let first = |table: &str, id: &str, filter: &str| {
        format!(
            "{id} = (SELECT {id} FROM public.{table} WHERE {filter} \
             ORDER BY work_id, {id} LIMIT 1)"
        )
    };
    vec![
        Corruption {
            label: "a missing row",
            family: 0,
            counts: [1, 0, 0],
            statement: format!(
                "DELETE FROM metric_rollup_work_month WHERE {}",
                first("metric_rollup_work_month", "rollup_work_month_id", "true")
            ),
        },
        Corruption {
            label: "an extra row",
            family: 1,
            counts: [0, 1, 0],
            statement: "INSERT INTO metric_rollup_work_country_month (work_id, publication_id, \
                            platform_id, measure_id, month_start, country_code, value, \
                            requires_institution_coverage, watermark) \
                        SELECT work_id, publication_id, platform_id, measure_id, month_start, \
                               'FR', 5, false, watermark \
                        FROM metric_rollup_work_country_month \
                        ORDER BY work_id, rollup_work_country_month_id LIMIT 1"
                .to_string(),
        },
        Corruption {
            label: "a value",
            family: 2,
            counts: [0, 0, 1],
            statement: format!(
                "UPDATE metric_rollup_work_institution_month SET value = value + 1 WHERE {}",
                first(
                    "metric_rollup_work_institution_month",
                    "rollup_work_institution_month_id",
                    "true"
                )
            ),
        },
        Corruption {
            label: "publication identity",
            family: 0,
            counts: [1, 1, 0],
            statement: format!(
                "UPDATE metric_rollup_work_month SET publication_id = '{second_publication}' \
                 WHERE {}",
                first(
                    "metric_rollup_work_month",
                    "rollup_work_month_id",
                    &format!("work_id = '{}'", works[0])
                )
            ),
        },
        Corruption {
            label: "country identity",
            family: 4,
            counts: [1, 1, 0],
            statement: format!(
                "UPDATE metric_rollup_work_country_year SET country_code = 'FR' WHERE {}",
                first(
                    "metric_rollup_work_country_year",
                    "rollup_work_country_year_id",
                    "true"
                )
            ),
        },
        Corruption {
            label: "institution identity",
            family: 5,
            counts: [1, 1, 0],
            statement: format!(
                "UPDATE metric_rollup_work_institution_year \
                 SET institution_id = '{third_institution}' WHERE {}",
                first(
                    "metric_rollup_work_institution_year",
                    "rollup_work_institution_year_id",
                    "true"
                )
            ),
        },
        Corruption {
            label: "a dependency flag",
            family: 0,
            counts: [0, 0, 1],
            statement: format!(
                "UPDATE metric_rollup_work_month \
                 SET requires_country_coverage = NOT requires_country_coverage WHERE {}",
                first("metric_rollup_work_month", "rollup_work_month_id", "true")
            ),
        },
        Corruption {
            label: "an ambiguity flag",
            family: 3,
            counts: [0, 0, 1],
            statement: format!(
                "UPDATE metric_rollup_work_month_ambiguity SET institution_ambiguous = true \
                 WHERE {}",
                first(
                    "metric_rollup_work_month_ambiguity",
                    "rollup_work_month_ambiguity_id",
                    "NOT institution_ambiguous"
                )
            ),
        },
        Corruption {
            label: "a watermark",
            family: 1,
            counts: [0, 0, 1],
            statement: format!(
                "UPDATE metric_rollup_work_country_month SET watermark = watermark - 1 WHERE {}",
                first(
                    "metric_rollup_work_country_month",
                    "rollup_work_country_month_id",
                    "watermark > 1"
                )
            ),
        },
        Corruption {
            label: "a derived row watermarked above the frontier",
            family: 0,
            counts: [0, 0, 1],
            statement: format!(
                "UPDATE metric_rollup_work_month SET watermark = {} WHERE {}",
                frontier + 10,
                first("metric_rollup_work_month", "rollup_work_month_id", "true")
            ),
        },
    ]
}

#[test]
fn verify_reports_each_corruption_in_its_family_and_rebuild_repairs_it() {
    let (_guard, pool) = setup();
    let (_corpus, works, frontier) = built_layered(&pool);
    let months = month_state(&pool);
    let years = year_state(&pool);
    let budgets = keys(12);
    for corruption in corruptions(&pool, &works, frontier) {
        let label = corruption.label;
        assert_eq!(exec_count(&pool, &corruption.statement), 1, "{label}");
        let prints = fingerprints(&pool);

        let verified = drive(&pool, start(&pool, Verify), &budgets);
        assert_eq!(verified.status, Mismatched, "{label}");
        let families = run_families(&verified);
        for (index, family) in families.iter().enumerate() {
            let expected = if index == corruption.family {
                corruption.counts
            } else {
                [0, 0, 0]
            };
            assert_eq!(
                [family[2], family[3], family[4]],
                expected,
                "{label}: family {index}"
            );
        }
        assert_eq!(families, whole_families(&pool), "{label}");
        assert_eq!(
            fingerprints(&pool),
            prints,
            "{label}: VERIFY writes no projection row"
        );

        let rebuilt = drive(&pool, start(&pool, Rebuild), &budgets);
        assert_eq!(rebuilt.status, Reconciled, "{label}");
        let build = rebuilt.build.expect("build");
        assert_eq!(
            build.replaced_steps, 1,
            "{label}: exactly one range was inexact"
        );
        assert_eq!(
            [
                build.found_missing_rows,
                build.found_extra_rows,
                build.found_mismatched_rows
            ],
            corruption.counts,
            "{label}"
        );
        assert_eq!(month_state(&pool), months, "{label}");
        assert_eq!(year_state(&pool), years, "{label}");
        assert!(verification(&pool).matches, "{label}");
    }
}

#[test]
fn a_work_day_row_above_the_frontier_blocks_both_modes_until_abandoned() {
    let (_guard, pool) = setup();
    let (_corpus, works, frontier) = built_layered(&pool);
    let budgets = keys(12);
    exec(
        &pool,
        &format!(
            "UPDATE metric_rollup_work_day SET watermark = {} WHERE rollup_work_day_id = \
             (SELECT rollup_work_day_id FROM metric_rollup_work_day WHERE work_id = '{}' \
              ORDER BY day DESC, rollup_work_day_id LIMIT 1)",
            frontier + 5,
            works[2]
        ),
    );
    for mode in [Rebuild, Verify] {
        let prints = fingerprints(&pool);
        let mut run = start(&pool, mode);
        let refusal = loop {
            match try_advance(&pool, &run, &budgets) {
                Ok(advanced) => {
                    assert!(advanced.advanced);
                    run = advanced.run;
                    assert!(run.status.is_open(), "{mode:?}: the run cannot end");
                }
                Err(error) => break error,
            }
        };
        assert_eq!(
            refusal,
            Error::Blocked(reason::WORK_DAY_ABOVE_FRONTIER),
            "{mode:?}"
        );
        let before = run_row(&pool, run.run_id);
        assert_eq!(
            try_advance(&pool, &run, &budgets),
            Err(refusal),
            "{mode:?}: the refusal repeats"
        );
        assert_eq!(
            run_row(&pool, run.run_id),
            before,
            "{mode:?}: nothing was written"
        );
        assert_eq!(fingerprints(&pool), prints, "{mode:?}");
        assert!(run_row(&pool, run.run_id).cursor_work_id.is_some() || run.step_count == 0);
        let abandoned = abandon(&pool, run.run_id).expect("abandon");
        assert_eq!(abandoned.status, Abandoned);
        assert_eq!(abandoned.step_count, before.step_count);
    }
}

// ===========================================================================
// 8. Orphans, and 9. partition additivity
// ===========================================================================

/// Orphan projection rows at four positions on the layered fixture: before
/// the first unit, between two works, inside a work between two of its
/// years, and after the last unit. Returns their units, in order.
fn insert_orphans(pool: &PgPool, corpus: &Corpus, works: &[Uuid]) -> Vec<Unit> {
    insert_orphans_at(pool, corpus, controlled_work(0x50), (works[1], 2024))
}

/// [`insert_orphans`] with the between-works work and the unit inside a
/// work chosen by the caller; neither may hold work-day rows.
fn insert_orphans_at(
    pool: &PgPool,
    corpus: &Corpus,
    between: Uuid,
    inside: (Uuid, i32),
) -> Vec<Unit> {
    let before = controlled_work(0x10);
    let after = controlled_work(0xF0);
    let (inside_work, inside_year) = inside;
    for work_id in [before, between, after] {
        if text_scalar(
            pool,
            &format!("SELECT work_id FROM work WHERE work_id = '{work_id}'"),
        )
        .is_none()
        {
            add_work(pool, corpus, work_id);
        }
    }
    let platform = corpus.platforms[0];
    let measure = corpus.fixture.measure_id;
    let institution = corpus.institutions[0];
    exec(
        pool,
        &format!(
            "INSERT INTO metric_rollup_work_month (work_id, publication_id, platform_id, \
                 measure_id, month_start, value, requires_country_coverage, \
                 requires_institution_coverage, watermark) \
             VALUES ('{before}', NULL, '{platform}', '{measure}', DATE '2024-03-01', 3, false, \
                     false, 1)"
        ),
    );
    exec(
        pool,
        &format!(
            "INSERT INTO metric_rollup_work_country_month (work_id, publication_id, platform_id, \
                 measure_id, month_start, country_code, value, requires_institution_coverage, \
                 watermark) \
             VALUES ('{between}', NULL, '{platform}', '{measure}', DATE '2024-03-01', 'GB', 3, \
                     false, 1)"
        ),
    );
    exec(
        pool,
        &format!(
            "INSERT INTO metric_rollup_work_institution_year (work_id, publication_id, \
                 platform_id, measure_id, year_start, institution_id, value, \
                 requires_country_coverage, watermark) \
             VALUES ('{inside_work}', NULL, '{platform}', '{measure}', \
                     DATE '{inside_year:04}-01-01', '{institution}', 3, false, 1)"
        ),
    );
    exec(
        pool,
        &format!(
            "INSERT INTO metric_rollup_work_month_ambiguity (work_id, platform_id, measure_id, \
                 month_start, total_ambiguous, country_ambiguous, institution_ambiguous, \
                 watermark) \
             VALUES ('{after}', '{platform}', '{measure}', DATE '2026-03-01', true, false, \
                     false, 1)"
        ),
    );
    vec![
        unit(before, 2024),
        unit(between, 2024),
        unit(inside_work, inside_year),
        unit(after, 2026),
    ]
}

#[test]
fn every_orphan_is_deleted_by_exactly_one_step_and_counted_by_exactly_one_page() {
    let (_guard, pool) = setup();
    let (corpus, works, _frontier) = built_layered(&pool);
    let months = month_state(&pool);
    let years = year_state(&pool);
    let budgets = keys(12);

    insert_orphans(&pool, &corpus, &works);
    let verified = drive(&pool, start(&pool, Verify), &budgets);
    assert_eq!(verified.status, Mismatched);
    let families = run_families(&verified);
    let extras: Vec<i64> = families.iter().map(|family| family[3]).collect();
    assert_eq!(
        extras,
        vec![1, 1, 0, 1, 0, 1],
        "one orphan in each of four families"
    );
    assert!(families
        .iter()
        .all(|family| family[2] == 0 && family[4] == 0));

    let mut run = start(&pool, Rebuild);
    let mut steps_with_extras = 0;
    while run.status == Building {
        let before = run.build.clone().expect("build").found_extra_rows;
        run = advance(&pool, &run, &budgets);
        let found = run.build.clone().expect("build").found_extra_rows - before;
        if found > 0 {
            steps_with_extras += 1;
        }
    }
    let build = run.build.clone().expect("build");
    assert_eq!(build.found_extra_rows, 4, "each orphan counted once");
    assert_eq!(build.replaced_steps, steps_with_extras);
    assert!(steps_with_extras >= 3, "the orphans sit in different steps");
    let run = drive(&pool, run, &budgets);
    assert_eq!(run.status, Reconciled);
    assert_eq!(month_state(&pool), months);
    assert_eq!(year_state(&pool), years);
}

#[test]
fn per_range_counts_add_up_to_the_whole_table_verification_for_every_cut() {
    let (_guard, pool) = setup();
    let (corpus, works, frontier) = built_layered(&pool);
    let orphans = insert_orphans(&pool, &corpus, &works);
    // Corruptions at known units: a value in work 0x20's 2024, a missing
    // ambiguity row, and the orphans.
    exec(
        &pool,
        &format!(
            "UPDATE metric_rollup_work_month SET value = value + 1 WHERE rollup_work_month_id = \
             (SELECT rollup_work_month_id FROM metric_rollup_work_month WHERE work_id = '{}' \
              AND month_start = DATE '2024-06-01' ORDER BY rollup_work_month_id LIMIT 1)",
            works[0]
        ),
    );
    // The last ambiguity row of a work that has work-day rows (not an
    // orphan).
    let last_ambiguity = "SELECT a.rollup_work_month_ambiguity_id, a.work_id, \
                                 date_part('year', a.month_start)::int AS year \
                          FROM metric_rollup_work_month_ambiguity a \
                          WHERE EXISTS (SELECT 1 FROM metric_rollup_work_day d \
                                        WHERE d.work_id = a.work_id) \
                          ORDER BY a.work_id DESC, a.month_start DESC LIMIT 1";
    let ambiguity = unit(
        text_scalar(&pool, &format!("SELECT work_id FROM ({last_ambiguity}) x"))
            .expect("an ambiguity row")
            .parse()
            .expect("uuid"),
        text_scalar(&pool, &format!("SELECT year FROM ({last_ambiguity}) x"))
            .expect("its year")
            .parse()
            .expect("year"),
    );
    exec(
        &pool,
        &format!(
            "DELETE FROM metric_rollup_work_month_ambiguity WHERE rollup_work_month_ambiguity_id \
             = (SELECT rollup_work_month_ambiguity_id FROM ({last_ambiguity}) x)"
        ),
    );
    let null_publication_work: Uuid = text_scalar(
        &pool,
        "SELECT m.work_id FROM metric_rollup_work_month m WHERE m.publication_id IS NULL \
         AND EXISTS (SELECT 1 FROM metric_rollup_work_day d WHERE d.work_id = m.work_id) \
         ORDER BY m.work_id DESC, m.month_start LIMIT 1",
    )
    .expect("a NULL-publication row")
    .parse()
    .expect("uuid");
    let mut corrupted: Vec<Unit> = orphans.clone();
    corrupted.push(unit(works[0], 2024));
    corrupted.push(ambiguity);

    let whole = whole_families(&pool);
    assert!(!verification(&pool).matches);
    let all_units: Vec<Unit> = {
        let mut units = BTreeSet::new();
        for row in projection(&pool) {
            units.insert(unit(row.work_id, row.day.year()));
        }
        units.into_iter().collect()
    };
    let cuts: Vec<(&str, Vec<Unit>)> = vec![
        (
            "inside a work",
            vec![unit(works[0], 2023), unit(works[0], 2024)],
        ),
        (
            "between works",
            vec![
                unit(works[0], 2025),
                unit(works[2], 2024),
                unit(works[3], 2024),
            ],
        ),
        ("at and next to the orphans", {
            let mut cut = orphans.clone();
            cut.push(unit(works[1], 2023));
            cut.sort();
            cut
        }),
        ("next to NULL-publication and ambiguity rows", {
            let mut cut = vec![
                unit(null_publication_work, 2022),
                unit(null_publication_work, 2023),
                ambiguity,
                unit(ambiguity.work_id, ambiguity.year_start.year() - 1),
            ];
            cut.sort();
            cut.dedup();
            cut
        }),
        (
            "before the first unit and after the last",
            vec![
                unit(controlled_work(0x01), 2000),
                unit(controlled_work(0xFE), 2100),
            ],
        ),
        ("every unit", all_units),
        ("no cut", Vec::new()),
    ];
    for (label, cut) in cuts {
        let mut bounds: Vec<Option<Unit>> = vec![None];
        bounds.extend(cut.iter().copied().map(Some));
        bounds.push(None);
        let mut sum = [[0_i64; 5]; 6];
        for pair in bounds.windows(2) {
            let (lo, hi) = (pair[0], pair[1]);
            let range = verify(&pool, lo, hi);
            assert!(!range.work_day_above(frontier));
            let counts = range_families(&range);
            for (total, family) in sum.iter_mut().zip(counts.iter()) {
                for (accumulated, value) in total.iter_mut().zip(family.iter()) {
                    *accumulated += value;
                }
            }
            let reported = counts
                .iter()
                .map(|family| family[2] + family[3] + family[4])
                .sum::<i64>();
            let inside = corrupted.iter().any(|corrupt| {
                lo.is_none_or(|lo| *corrupt > lo) && hi.is_none_or(|hi| *corrupt <= hi)
            });
            assert_eq!(reported > 0, inside, "{label}: ({lo:?}, {hi:?}]");
        }
        assert_eq!(sum, whole, "{label}");
    }
}

// ===========================================================================
// 10. Interleaving, and 11. concurrent canonical ingestion
// ===========================================================================

#[test]
fn completions_between_steps_keep_the_built_prefix_exact_and_the_run_ends_exact() {
    let (_guard, pool) = setup();
    let (corpus, works, _frontier) = layered(&pool);
    let budgets = keys(12);
    let prefix_exact = |pool: &PgPool, run: &Run, label: &str| {
        let frontier = state(pool).applied_through_sequence;
        let cursor = cursor(pool, run.run_id);
        assert!(
            verify(pool, None, cursor).exact_at(frontier),
            "{label}: I1 at {cursor:?}"
        );
    };

    let run = advance(&pool, &start(&pool, Rebuild), &budgets);
    let built = cursor(&pool, run.run_id).expect("a cursor");
    assert_eq!(built.work_id, works[0]);
    prefix_exact(&pool, &run, "after the first step");

    // One batch whose month keys straddle the cursor: a built unit and an
    // unbuilt one.
    for (work_id, on) in [(works[0], day(2023, 1, 2)), (works[3], day(2024, 6, 2))] {
        commit_cell_delta(
            &pool,
            &corpus.fixture,
            work_id,
            on,
            CellDimensions {
                publication_id: None,
                country_code: Some("DE"),
                institution_id: None,
            },
            11,
        );
    }
    apply_everything(&pool);
    prefix_exact(&pool, &run, "after a straddling batch");

    let run = advance(&pool, &run, &budgets);
    prefix_exact(&pool, &run, "after the second step");
    // A new month key inside the built prefix, then one beyond the cursor.
    complete_one(&pool, &corpus, works[0], day(2023, 3, 9), 5);
    prefix_exact(&pool, &run, "after a new key in the built prefix");
    complete_one(&pool, &corpus, works[4], day(2025, 3, 9), 5);
    prefix_exact(&pool, &run, "after a new key beyond the cursor");

    let mut run = advance(&pool, &run, &budgets);
    while run.status == Building {
        complete_one(&pool, &corpus, works[2], day(2024, 12, 20), 1);
        prefix_exact(&pool, &run, "between later steps");
        run = advance(&pool, &run, &budgets);
    }
    let run = drive(&pool, run, &budgets);
    assert_eq!(
        run.status, Reconciled,
        "no completion ran during the verify phase"
    );
    assert_exact_verification(&pool, &verification(&pool));
}

/// The canonical `DAY` ingestion statements for one new record: record,
/// current revision and its `PENDING` delta, whose insert allocates the
/// next work-day sequence.
fn ingestion_statements(corpus: &Corpus, work_id: Uuid, on: NaiveDate, value: i64) -> Vec<String> {
    let record_id = Uuid::new_v4();
    let revision_id = Uuid::new_v4();
    let fixture = &corpus.fixture;
    vec![
        format!(
            "INSERT INTO metric_record (record_id, identity_hash, work_id, publication_id, \
                 platform_id, measure_id, period_start, period_end, reporting_grain, country_code, \
                 institution_id, winning_source_account_id) \
             VALUES ('{record_id}', 'identity-{}', '{work_id}', NULL, '{}', '{}', DATE '{on}', \
                     DATE '{on}' + 1, 'DAY', 'GB', NULL, '{}')",
            record_id.simple(),
            fixture.platform_id,
            fixture.measure_id,
            fixture.source_account_id
        ),
        format!(
            "INSERT INTO metric_record_revision (record_revision_id, record_id, revision_number, \
                 import_id, value, content_hash, status, supersedes_revision_id) \
             VALUES ('{revision_id}', '{record_id}', 1, '{}', {value}, 'content-{record_id}-1', \
                     'CURRENT', NULL)",
            fixture.import_id
        ),
        format!(
            "UPDATE metric_record SET current_revision_id = '{revision_id}' \
             WHERE record_id = '{record_id}'"
        ),
        format!(
            "INSERT INTO metric_rollup_delta (record_id, revision_id, delta_value, status) \
             VALUES ('{record_id}', '{revision_id}', {value}, 'PENDING')"
        ),
    ]
}

/// An ingestion transaction that first locks its work `FOR UPDATE`, then
/// ingests, and commits when released: a held ingestion.
struct HeldIngestion {
    release: std::sync::mpsc::Sender<()>,
    handle: thread::JoinHandle<Result<(), String>>,
}

impl HeldIngestion {
    fn start(statements: Vec<String>, work_id: Uuid, wait_ready: bool) -> Self {
        let (release, wait) = std::sync::mpsc::channel::<()>();
        let (ready_tx, ready_rx) = std::sync::mpsc::channel::<()>();
        let handle = thread::spawn(move || {
            let mut connection =
                PgConnection::establish(&test_db_url()).map_err(|error| error.to_string())?;
            connection
                .transaction::<_, DieselError, _>(|c| {
                    sql_query(format!(
                        "SELECT work_id FROM work WHERE work_id = '{work_id}' FOR UPDATE"
                    ))
                    .execute(c)?;
                    for statement in &statements {
                        sql_query(statement).execute(c)?;
                    }
                    let _ = ready_tx.send(());
                    let _ = wait.recv();
                    Ok(())
                })
                .map_err(|error| error.to_string())
        });
        if wait_ready {
            ready_rx
                .recv_timeout(Duration::from_secs(30))
                .expect("the held ingestion started");
        }
        HeldIngestion { release, handle }
    }

    fn commit(self) {
        let _ = self.release.send(());
        self.handle
            .join()
            .expect("join")
            .expect("canonical ingestion never fails");
    }
}

#[test]
fn concurrent_canonical_ingestion_never_fails_and_steps_are_at_worst_retryable() {
    let (_guard, pool) = setup();
    let (corpus, works, _frontier) = layered(&pool);
    let corpus = Arc::new(corpus);
    let budgets = short(12);

    // Ingestion holding a work FOR UPDATE before the step locks it: the
    // step is retryable and nothing is written.
    let run = start(&pool, Rebuild);
    let held = HeldIngestion::start(
        ingestion_statements(&corpus, works[0], day(2023, 1, 3), 2),
        works[0],
        true,
    );
    let prints = fingerprints(&pool);
    let refused = try_advance(&pool, &run, &budgets).expect_err("the step waits on the work");
    assert!(
        matches!(
            refused,
            Error::Retryable(reason::STATEMENT_TIMEOUT | reason::LOCK_TIMEOUT)
        ),
        "{refused:?}"
    );
    assert_eq!(fingerprints(&pool), prints);
    assert_eq!(run_row(&pool, run.run_id).step_count, 0);
    held.commit();

    // Ingestion that locks a work FOR UPDATE after the step locked it waits
    // for the step's commit and then succeeds.
    let (paused, pause, _log) = pausing_pool(RUN_LOCK_SQL);
    let stepping = {
        let run = run.clone();
        thread::spawn(move || try_advance(&paused, &run, &budgets))
    };
    pause.wait_reached();
    let waiting = HeldIngestion::start(
        ingestion_statements(&corpus, works[0], day(2023, 1, 4), 3),
        works[0],
        false,
    );
    wait_for_lock_evidence("the ingestion waits on the step's work lock", || {
        backends_waiting_on_a_lock(&pool) >= 1
    });
    pause.release();
    let stepped = stepping.join().expect("join").expect("the step commits");
    assert!(stepped.advanced);
    waiting.commit();

    // Continuous ingestion over overlapping works while the build runs.
    let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let ingesting = {
        let corpus = Arc::clone(&corpus);
        let stop = Arc::clone(&stop);
        let works = works.clone();
        thread::spawn(move || {
            let pool = wide_pool();
            let mut n = 0_u32;
            while !stop.load(std::sync::atomic::Ordering::SeqCst) {
                let work_id = works[(n as usize) % works.len()];
                commit_cell_delta(
                    &pool,
                    &corpus.fixture,
                    work_id,
                    day(2024, 1 + n % 12, 5),
                    CellDimensions {
                        publication_id: None,
                        country_code: Some("US"),
                        institution_id: None,
                    },
                    1,
                );
                n += 1;
            }
            n
        })
    };
    let mut run = stepped.run;
    let mut retryable = 0;
    while run.status == Building {
        match try_advance(&pool, &run, &budgets) {
            Ok(advanced) => {
                assert!(advanced.advanced);
                run = advanced.run;
            }
            Err(Error::Retryable(why)) => {
                assert_ne!(why, reason::DEADLOCK, "no deadlock");
                retryable += 1;
                assert!(retryable < 50, "steps keep failing: {why}");
            }
            Err(other) => panic!("a step failed: {other:?}"),
        }
    }
    stop.store(true, std::sync::atomic::Ordering::SeqCst);
    let ingested = ingesting.join().expect("ingestion never fails");
    assert!(ingested > 0);
    apply_everything(&pool);
    let run = drive(&pool, run, &budgets);
    assert_eq!(run.status, Reconciled);
    assert_exact_verification(&pool, &verification(&pool));
}

// ===========================================================================
// 12. Pages under change
// ===========================================================================

#[test]
fn a_completion_during_a_non_final_page_gives_no_false_mismatch_and_ends_verified() {
    let (_guard, pool) = setup();
    let (corpus, works, frontier) = built_layered(&pool);
    let budgets = keys(12);
    commit_cell_delta(
        &pool,
        &corpus.fixture,
        works[0],
        day(2023, 1, 2),
        CellDimensions {
            publication_id: None,
            country_code: Some("GB"),
            institution_id: None,
        },
        4,
    );
    let run = start(&pool, Verify);
    let (paused, pause, _log) = pausing_pool(PAGE_FRONTIER_SQL);
    let paging = {
        let run = run.clone();
        thread::spawn(move || try_advance(&paused, &run, &budgets))
    };
    pause.wait_reached();
    // The page's snapshot is open; the completion commits now.
    let applied = apply_everything(&pool);
    assert!(applied > frontier);
    pause.release();
    let paged = paging.join().expect("join").expect("the page");
    assert!(paged.advanced);
    assert_eq!(paged.run.status, Verifying, "a non-final page");
    assert_eq!(
        paged.run.verify.last_applied_through_sequence,
        Some(frontier)
    );
    let run = drive(&pool, paged.run, &budgets);
    assert_eq!(
        run.status, Verified,
        "never RECONCILED across a frontier change"
    );
    assert_eq!(run.verify.first_applied_through_sequence, Some(frontier));
    assert_eq!(run.verify.last_applied_through_sequence, Some(applied));
    assert!(run_families(&run)
        .iter()
        .all(|family| family[2] == 0 && family[3] == 0 && family[4] == 0));
}

#[test]
fn a_completion_during_the_final_page_may_end_reconciled_at_the_frontier_it_saw() {
    let (_guard, pool) = setup();
    let (corpus, works, frontier) = built_layered(&pool);
    let budgets = keys(1_000);
    commit_cell_delta(
        &pool,
        &corpus.fixture,
        works[4],
        day(2025, 12, 2),
        CellDimensions {
            publication_id: None,
            country_code: None,
            institution_id: None,
        },
        4,
    );
    let run = start(&pool, Verify);
    let (paused, pause, _log) = pausing_pool(PAGE_FRONTIER_SQL);
    let paging = {
        let run = run.clone();
        thread::spawn(move || try_advance(&paused, &run, &budgets))
    };
    pause.wait_reached();
    let applied = apply_everything(&pool);
    pause.release();
    let paged = paging.join().expect("join").expect("the only page");
    assert_eq!(paged.run.status, Reconciled);
    assert_eq!(paged.run.verify.pages, 1);
    assert_eq!(
        paged.run.verify.first_applied_through_sequence,
        Some(frontier)
    );
    assert_eq!(
        paged.run.verify.last_applied_through_sequence,
        Some(frontier)
    );
    assert!(applied > frontier);
}

#[test]
fn completions_between_pages_never_yield_reconciled() {
    let (_guard, pool) = setup();
    let (corpus, works, frontier) = built_layered(&pool);
    let budgets = keys(12);
    for (month, mode) in [(2, Verify), (3, Rebuild)] {
        let mut run = drive_build(&pool, start(&pool, mode), &budgets);
        let mut completions: u32 = 0;
        while run.status.is_open() {
            run = advance(&pool, &run, &budgets);
            if run.status.is_open() && completions < 3 {
                let work_id = works[completions as usize];
                complete_one(
                    &pool,
                    &corpus,
                    work_id,
                    day(2025, month, 1 + completions),
                    2,
                );
                completions += 1;
            }
        }
        assert!(completions >= 1, "{mode:?}: the run had several pages");
        assert_eq!(run.status, Verified, "{mode:?}");
        assert!(run.verify.first_applied_through_sequence >= Some(frontier));
        assert!(
            run.verify.last_applied_through_sequence > run.verify.first_applied_through_sequence
        );
        assert!(verification(&pool).matches);
    }
}

// ===========================================================================
// 13. OPS-02 interplay, and 14. readers
// ===========================================================================

#[test]
fn the_full_rebuild_between_two_steps_leaves_the_run_able_to_finish_exact() {
    let (_guard, pool) = setup();
    let (_corpus, _works, frontier) = layered(&pool);
    let budgets = keys(12);
    let run = advance(&pool, &start(&pool, Rebuild), &budgets);
    assert!(
        rebuild(&pool, frontier)
            .expect("the OPS-02 rebuild")
            .rebuilt
    );
    let prints = fingerprints(&pool);
    let run = drive_build(&pool, run, &budgets);
    assert_eq!(
        run.build.clone().expect("build").replaced_steps,
        1,
        "only the first step"
    );
    assert_eq!(
        fingerprints(&pool),
        prints,
        "after the full rebuild every step is exact"
    );
    let run = drive(&pool, run, &budgets);
    assert_eq!(run.status, Reconciled);
    assert_exact_verification(&pool, &verification(&pool));
}

fn corrupt_one_value(pool: &PgPool) {
    assert_eq!(
        exec_count(
            pool,
            "UPDATE metric_rollup_work_month SET value = value + 1 WHERE rollup_work_month_id = \
             (SELECT rollup_work_month_id FROM metric_rollup_work_month \
              ORDER BY work_id, rollup_work_month_id LIMIT 1)"
        ),
        1
    );
}

#[test]
fn the_full_rebuild_waits_for_a_page_holding_access_share() {
    let (_guard, pool) = setup();
    let (_corpus, _works, frontier) = built_layered(&pool);
    let budgets = keys(12);
    corrupt_one_value(&pool);
    let run = start(&pool, Verify);
    let (paused, pause, _log) = pausing_pool(PAGE_FRONTIER_SQL);
    let paging = {
        let run = run.clone();
        thread::spawn(move || try_advance(&paused, &run, &budgets))
    };
    pause.wait_reached();
    let rebuilding = {
        let pool = wide_pool();
        thread::spawn(move || rebuild(&pool, frontier).map(|result| result.rebuilt))
    };
    wait_for_lock_evidence("the full rebuild waits behind the page", || {
        relation_locks(&pool, "metric_rollup_work_month", "AccessExclusiveLock")
            .iter()
            .any(|(_, granted)| !granted)
    });
    pause.release();
    let paged = paging.join().expect("join").expect("the page commits");
    assert!(paged.advanced);
    assert_eq!(rebuilding.join().expect("join"), Ok(true));
    let run = drive(&pool, paged.run, &budgets);
    // The first page saw the corruption or not, depending on its range; the
    // later pages see the rebuilt state. Either way the counts are those of
    // states that existed, and the rebuilt state is exact.
    assert!(
        matches!(run.status, Mismatched | Reconciled),
        "{:?}",
        run.status
    );
    assert!(verification(&pool).matches);
}

#[test]
fn a_page_behind_the_full_rebuilds_truncate_is_retryable_or_sees_the_rebuilt_state() {
    let (_guard, pool) = setup();
    let (_corpus, _works, frontier) = built_layered(&pool);
    let budgets = keys(1_000);
    corrupt_one_value(&pool);

    // The page times out at its LOCK TABLE behind a held TRUNCATE.
    let run = start(&pool, Verify);
    let (paused, pause, _log) = pausing_pool(REBUILD_MONTH_KEYS_SQL);
    let rebuilding = thread::spawn(move || rebuild(&paused, frontier).map(|r| r.rebuilt));
    pause.wait_reached();
    let started = Instant::now();
    assert_eq!(
        try_advance(&pool, &run, &budgets),
        Err(Error::Retryable(reason::LOCK_TIMEOUT))
    );
    assert!(started.elapsed() < Duration::from_secs(7));
    assert_eq!(run_row(&pool, run.run_id).step_count, 0);
    pause.release();
    assert_eq!(rebuilding.join().expect("join"), Ok(true));

    // Corrupt again; this time the TRUNCATE commits while the page waits at
    // its LOCK TABLE: the page then sees the rebuilt state, because no
    // snapshot was taken before the lock.
    corrupt_one_value(&pool);
    let (paused, pause, _log) = pausing_pool(REBUILD_MONTH_KEYS_SQL);
    let rebuilding = thread::spawn(move || rebuild(&paused, frontier).map(|r| r.rebuilt));
    pause.wait_reached();
    let (logged, log) = logging_pool();
    let paging = {
        let run = run.clone();
        thread::spawn(move || try_advance(&logged, &run, &budgets))
    };
    wait_for_lock_evidence("the page waits at its LOCK TABLE", || {
        relation_locks(&pool, "metric_rollup_work_month", "AccessShareLock")
            .iter()
            .any(|(_, granted)| !granted)
    });
    // Inside the page's transaction: only SET LOCAL and the waiting LOCK.
    let statements = captured(&log);
    let begin = statements
        .iter()
        .rposition(|s| s.to_uppercase().starts_with("BEGIN"))
        .expect("the page's BEGIN");
    let issued = &statements[begin + 1..];
    assert!(
        issued.iter().all(|s| is_set_local(s) || s == PAGE_LOCK_SQL),
        "{issued:?}"
    );
    assert_eq!(issued.last().map(String::as_str), Some(PAGE_LOCK_SQL));
    pause.release();
    assert_eq!(rebuilding.join().expect("join"), Ok(true));
    let paged = paging.join().expect("join").expect("the page");
    assert_eq!(
        paged.run.status, Reconciled,
        "the page saw the rebuilt state"
    );
}

#[test]
fn a_dashboard_reader_holding_access_share_does_not_delay_a_step() {
    let (_guard, pool) = setup();
    let (_corpus, _works, _frontier) = built_layered(&pool);
    truncate_projections(&pool);
    let budgets = short(12);
    let reader = HeldTransaction::start(format!(
        "SELECT {} AS rows",
        DERIVED_TABLES
            .iter()
            .map(|table| format!("(SELECT count(*) FROM public.{table})"))
            .collect::<Vec<_>>()
            .join(" + ")
    ));
    for table in DERIVED_TABLES {
        assert!(
            relation_locks(&pool, table, "AccessShareLock")
                .iter()
                .any(|(_, granted)| *granted),
            "{table}"
        );
    }
    let run = start(&pool, Rebuild);
    let started = Instant::now();
    let run = advance(&pool, &run, &budgets);
    assert_eq!(run.build.clone().expect("build").replaced_steps, 1);
    assert!(
        started.elapsed() < Duration::from_millis(1_500),
        "the replacing step committed without waiting: {:?}",
        started.elapsed()
    );
    reader.commit();
}

// ===========================================================================
// 15. Failure paths
// ===========================================================================

/// Every statement site of a replacing step over a range of two month
/// chunks and one year chunk, in execution order.
fn replacing_step_sites() -> Vec<Site> {
    let mut sites = vec![
        Site::Isolation(Stage::Presize),
        Site::RunRead,
        Site::Sizing(Stage::Presize),
        Site::LockWorks,
        Site::LockState,
        Site::RunLock(Stage::Hold),
        Site::Sizing(Stage::Hold),
        Site::PreVerify,
    ];
    sites.extend((0..6).map(Site::RangeDelete));
    sites.push(Site::MonthKeys);
    for chunk in 0..2 {
        sites.push(Site::MonthWatermark(chunk));
        sites.extend((0..4).map(|index| Site::MonthInsert(chunk, index)));
    }
    sites.extend((0..2).map(|index| Site::YearInsert(0, index)));
    sites.push(Site::PostVerify);
    sites.push(Site::RunUpdate(Stage::Hold));
    sites
}

#[test]
fn an_injected_error_before_any_statement_of_a_step_rolls_the_whole_step_back() {
    let (_guard, pool) = setup();
    let (_corpus, _works, _frontier) = layered(&pool);
    let budgets = RunBudgets::PRODUCTION;
    let run = start(&pool, Rebuild);
    let row = run_row(&pool, run.run_id);
    let prints = fingerprints(&pool);
    let sized = size_range_for_test(&pool, None, budgets.step_size()).expect("sizing");
    assert!(
        sized.month_keys > 50,
        "two month chunks: {}",
        sized.month_keys
    );
    for site in replacing_step_sites() {
        hooks::clear();
        hooks::inject(site, hooks::Injection::Statement("SELECT 1/0".to_string()));
        let refused = try_advance(&pool, &run, &budgets).expect_err("the injected failure");
        hooks::clear();
        assert_eq!(refused, Error::Internal(reason::DATABASE), "{site:?}");
        assert_eq!(refused.code(), "INTERNAL_ERROR");
        assert_eq!(
            run_row(&pool, run.run_id),
            row,
            "{site:?}: the cursor is unchanged"
        );
        assert_eq!(fingerprints(&pool), prints, "{site:?}: nothing was written");
    }
    let stepped = advance(&pool, &run, &budgets);
    assert_eq!(stepped.status, Verifying, "one step covers the fixture");
    assert!(verification(&pool).matches);
}

#[test]
fn lock_waits_behind_a_held_state_row_or_work_row_are_retryable_and_write_nothing() {
    let (_guard, pool) = setup();
    let (_corpus, works, _frontier) = layered(&pool);
    let budgets = short(1_000);
    let run = start(&pool, Rebuild);
    let row = run_row(&pool, run.run_id);
    for statement in [
        "SELECT * FROM metric_rollup_work_day_state WHERE state_id = 1 FOR UPDATE".to_string(),
        format!(
            "SELECT work_id FROM work WHERE work_id = '{}' FOR UPDATE",
            works[2]
        ),
    ] {
        let held = HeldTransaction::start(statement.clone());
        let started = Instant::now();
        let refused = try_advance(&pool, &run, &budgets).expect_err("a lock wait");
        held.commit();
        assert!(
            matches!(
                refused,
                Error::Retryable(reason::LOCK_TIMEOUT | reason::STATEMENT_TIMEOUT)
            ),
            "{statement}: {refused:?}"
        );
        assert!(
            started.elapsed()
                < budgets.step_presize + budgets.step_lock + Duration::from_millis(500)
        );
        assert_eq!(run_row(&pool, run.run_id), row, "{statement}");
        assert!(month_rows(&pool).is_empty());
    }
}

#[test]
fn a_post_verify_mismatch_or_a_delete_count_mismatch_is_blocked_and_rolled_back() {
    let (_guard, pool) = setup();
    let (_corpus, _works, frontier) = built_layered(&pool);
    let budgets = RunBudgets::PRODUCTION;
    corrupt_one_value(&pool);
    let run = start(&pool, Rebuild);
    let row = run_row(&pool, run.run_id);
    let rows = digests(&pool);

    let altering = super::tests::InjectedTrigger::install(
        &pool,
        "BEFORE INSERT",
        "metric_rollup_work_month",
        "ROW",
        None,
        "NEW.value := NEW.value + 1; RETURN NEW;",
    );
    assert_eq!(
        try_advance(&pool, &run, &budgets),
        Err(Error::Blocked(reason::POST_VERIFY_INEXACT))
    );
    drop(altering);
    assert_eq!(run_row(&pool, run.run_id), row);
    assert_eq!(digests(&pool), rows);

    let skipping = super::tests::InjectedTrigger::install(
        &pool,
        "BEFORE DELETE",
        "metric_rollup_work_country_month",
        "ROW",
        None,
        "RETURN NULL;",
    );
    assert_eq!(
        try_advance(&pool, &run, &budgets),
        Err(Error::Blocked(reason::DELETE_COUNT_MISMATCH))
    );
    drop(skipping);
    assert_eq!(run_row(&pool, run.run_id), row);
    assert_eq!(digests(&pool), rows);

    let run = drive(&pool, run, &budgets);
    assert_eq!(run.status, Reconciled);
    assert_eq!(run.verify.first_applied_through_sequence, Some(frontier));
}

#[test]
fn an_overflow_in_a_pre_verify_a_page_or_a_re_executed_insert_is_blocked() {
    let (_guard, pool) = setup();
    let (_corpus, works, _frontier) = built_layered(&pool);
    let budgets = RunBudgets::PRODUCTION;

    // A re-executed insert that overflows (injected: the pre-verify would
    // otherwise see the same overflow first).
    corrupt_one_value(&pool);
    let run = start(&pool, Rebuild);
    let row = run_row(&pool, run.run_id);
    let rows = digests(&pool);
    let overflowing = super::tests::InjectedTrigger::install(
        &pool,
        "BEFORE INSERT",
        "metric_rollup_work_country_year",
        "ROW",
        None,
        "RAISE EXCEPTION 'bigint out of range' USING ERRCODE = '22003'; RETURN NEW;",
    );
    assert_eq!(
        try_advance(&pool, &run, &budgets),
        Err(Error::Blocked(reason::BIGINT_OVERFLOW))
    );
    drop(overflowing);
    assert_eq!(run_row(&pool, run.run_id), row);
    assert_eq!(digests(&pool), rows);
    abandon(&pool, run.run_id).expect("abandon");

    // Two work-day values of one month key whose sum overflows: the
    // pre-verify of a step and a page both refuse.
    exec(
        &pool,
        &format!(
            "UPDATE metric_rollup_work_day SET value = 9223372036854775807 \
             WHERE rollup_work_day_id IN (SELECT rollup_work_day_id FROM metric_rollup_work_day \
                 WHERE work_id = '{}' AND country_code IS NULL AND institution_id IS NULL \
                 AND publication_id IS NULL AND day = DATE '2024-06-01' \
                 UNION ALL \
                 SELECT rollup_work_day_id FROM metric_rollup_work_day \
                 WHERE work_id = '{}' AND country_code IS NULL AND institution_id IS NULL \
                 AND publication_id IS NULL AND day = DATE '2024-06-15')",
            works[2], works[2]
        ),
    );
    for mode in [Rebuild, Verify] {
        let mut run = start(&pool, mode);
        let refusal = loop {
            match try_advance(&pool, &run, &budgets) {
                Ok(advanced) => run = advanced.run,
                Err(error) => break error,
            }
        };
        assert_eq!(refusal, Error::Blocked(reason::BIGINT_OVERFLOW), "{mode:?}");
        assert!(run.status.is_open());
        abandon(&pool, run.run_id).expect("abandon");
    }
}

#[test]
fn a_session_ended_by_the_idle_timeout_is_retryable_and_its_connection_is_discarded() {
    let (_guard, pool) = setup();
    let (_corpus, _works, _frontier) = layered(&pool);
    let single = Arc::new(
        Pool::builder()
            .max_size(1)
            .build(ConnectionManager::<PgConnection>::new(test_db_url()))
            .expect("a one-connection pool"),
    );
    let backend_pid = |pool: &PgPool| -> i32 {
        #[derive(QueryableByName)]
        struct Pid {
            #[diesel(sql_type = diesel::sql_types::Integer)]
            pid: i32,
        }
        let mut connection = pool.get().expect("a connection");
        sql_query("SELECT pg_backend_pid() AS pid")
            .get_result::<Pid>(&mut connection)
            .expect("pid")
            .pid
    };
    let before = backend_pid(&single);
    let run = start(&pool, Rebuild);
    let row = run_row(&pool, run.run_id);
    let budgets = RunBudgets {
        step_hold: Duration::from_secs(20),
        ..RunBudgets::PRODUCTION
    };
    hooks::inject(
        Site::PreVerify,
        hooks::Injection::ClientPause(Duration::from_millis(2_600)),
    );
    let refused = try_advance(&single, &run, &budgets).expect_err("the session ended");
    hooks::clear();
    assert!(
        matches!(
            refused,
            Error::Retryable(reason::IDLE_TIMEOUT | reason::CONNECTION_CLOSED)
        ),
        "{refused:?}"
    );
    assert_eq!(run_row(&pool, run.run_id), row);
    assert!(month_rows(&pool).is_empty());
    let after = backend_pid(&single);
    assert_ne!(after, before, "the pool discarded the ended session");
    assert_eq!(
        scalar_i64(
            &pool,
            &format!("(SELECT COUNT(*) FROM pg_stat_activity WHERE pid = {before})")
        ),
        0
    );
    // The pool's fresh session serves the retried advance.
    let retried = advance(&single, &run, &budgets);
    assert_eq!(retried.step_count, run.step_count + 1);
}

#[test]
fn an_exhausted_pool_is_retryable_within_the_checkout_wait_and_issues_no_statement() {
    let (_guard, pool) = setup();
    let run = start(&pool, Verify);
    let (logged, log) = logging_pool();
    let held = logged.get().expect("the pool's only connection");
    let issued = log.lock().expect("statement log").len();
    let budgets = RunBudgets::PRODUCTION;
    type Operation<'a> = Box<dyn Fn() -> Option<Error> + 'a>;
    let operations: Vec<(&str, Operation<'_>)> = vec![
        (
            "start",
            Box::new(|| {
                start_metric_rollup_month_rebuild(
                    &logged,
                    &StartMetricRollupMonthRebuildInput { mode: Verify },
                    STARTED_BY,
                )
                .err()
            }),
        ),
        (
            "advance",
            Box::new(|| try_advance(&logged, &run, &budgets).err()),
        ),
        ("abandon", Box::new(|| abandon(&logged, run.run_id).err())),
        (
            "status",
            Box::new(|| metric_rollup_month_rebuild_run(&logged, None).err()),
        ),
    ];
    for (label, operation) in operations {
        let started = Instant::now();
        assert_eq!(
            operation(),
            Some(Error::Retryable(reason::POOL_CHECKOUT)),
            "{label}"
        );
        let waited = started.elapsed();
        assert!(
            waited >= METRIC_ROLLUP_RUN_CHECKOUT_TIMEOUT - Duration::from_millis(100)
                && waited < METRIC_ROLLUP_RUN_CHECKOUT_TIMEOUT + Duration::from_millis(1_500),
            "{label}: {waited:?}"
        );
        assert_eq!(log.lock().expect("statement log").len(), issued, "{label}");
    }
    drop(held);
}

#[test]
fn each_operation_takes_exactly_one_connection_with_a_bounded_wait() {
    // The source guard, as `metric_dashboard`'s.
    let source = include_str!("rebuild_run.rs");
    assert_eq!(source.matches(".get_timeout(").count(), 1);
    assert_eq!(source.matches(".get()").count(), 0);
    assert_eq!(source.matches("db.get(").count(), 0);
    assert_eq!(source.matches("fn checkout(").count(), 1);
    for operation in [
        "pub fn start_metric_rollup_month_rebuild(",
        "pub fn abandon_metric_rollup_month_rebuild(",
        "pub fn metric_rollup_month_rebuild_run(",
        "pub(crate) fn advance_metric_rollup_month_rebuild_within(",
    ] {
        let body = source
            .split_once(operation)
            .unwrap_or_else(|| panic!("{operation}"))
            .1
            .split_once("\n}\n")
            .expect("the body")
            .0;
        assert_eq!(body.matches("checkout(db)").count(), 1, "{operation}");
    }

    // One advance: one checkout validation, then the dispatch read and the
    // step on that same connection.
    let (_guard, pool) = setup();
    let (_corpus, _works, _frontier) = layered(&pool);
    let run = start(&pool, Rebuild);
    let (logged, log) = logging_pool();
    advance(&logged, &run, &RunBudgets::PRODUCTION);
    let statements = captured(&log);
    assert_eq!(statements.first().map(String::as_str), Some("SELECT 1"));
    assert_eq!(statements.iter().filter(|s| *s == "SELECT 1").count(), 1);
    let dispatch = statements
        .iter()
        .position(|s| s == RUN_READ_SQL)
        .expect("the dispatch read");
    let works = statements
        .iter()
        .position(|s| s.starts_with(LOCK_WORKS_SQL))
        .expect("the step");
    assert!(dispatch < works);
    assert_eq!(
        statements
            .iter()
            .filter(|s| s.to_uppercase().starts_with("BEGIN"))
            .count(),
        2,
        "the dispatch read's transaction and the step's"
    );
}

#[test]
fn the_dispatch_and_status_reads_behind_an_exclusive_run_table_lock_are_retryable_within_a_second()
{
    let (_guard, pool) = setup();
    let run = start(&pool, Verify);
    let held = HeldTransaction::start(
        "LOCK TABLE metric_rollup_month_rebuild_run IN ACCESS EXCLUSIVE MODE".to_string(),
    );
    for label in ["dispatch", "status"] {
        let started = Instant::now();
        let refused = if label == "dispatch" {
            try_advance(&pool, &run, &RunBudgets::PRODUCTION).expect_err("a lock wait")
        } else {
            metric_rollup_month_rebuild_run(&pool, Some(run.run_id)).expect_err("a lock wait")
        };
        assert!(
            matches!(
                refused,
                Error::Retryable(reason::LOCK_TIMEOUT | reason::STATEMENT_TIMEOUT)
            ),
            "{label}: {refused:?}"
        );
        assert!(started.elapsed() < Duration::from_millis(1_800), "{label}");
    }
    held.commit();
    assert_eq!(run_row(&pool, run.run_id).step_count, 0);
}

// ===========================================================================
// 16. Hold bound
// ===========================================================================

/// What a hold-bound case injects.
enum Delay {
    Slow(Vec<(Site, f64)>),
    Held(String),
}

/// Run one step under `delay` on the timed pool and return the measured
/// state-row and work-row holds (each up to the call's return, so rollback
/// is included).
fn measure_step(
    timed: &(Arc<PgPool>, Timed),
    run: &Run,
    budgets: &RunBudgets,
    delay: &Delay,
) -> (
    Result<MetricRollupMonthRebuildAdvance, Error>,
    Duration,
    Duration,
) {
    let (pool, log) = timed;
    log.lock().expect("timed log").clear();
    hooks::clear();
    let held = match delay {
        Delay::Slow(sites) => {
            for (site, seconds) in sites {
                hooks::inject(
                    *site,
                    hooks::Injection::Statement(format!("SELECT pg_sleep({seconds})")),
                );
            }
            None
        }
        Delay::Held(statement) => Some(HeldTransaction::start(statement.clone())),
    };
    let result = try_advance(pool, run, budgets);
    let end = Instant::now();
    hooks::clear();
    if let Some(held) = held {
        held.commit();
    }
    let hold = |prefix: &str| finished(log, prefix).map_or(Duration::ZERO, |at| end - at);
    (result, hold(LOCK_STATE_SQL), hold(LOCK_WORKS_SQL))
}

#[test]
fn the_state_row_and_work_row_holds_never_exceed_their_budgets() {
    let (_guard, pool) = setup();
    let (corpus, works, _frontier) = built_layered(&pool);
    let budgets = short(1_000);
    let margin = Duration::from_millis(400);
    corrupt_one_value(&pool);
    let run = start(&pool, Rebuild);
    let row = run_row(&pool, run.run_id);
    let timed = timed_pool();
    let publication = publication_of(&pool, works[0]);

    let mut cases: Vec<(String, Delay)> = vec![
        (
            "two slow statements in one chunk".to_string(),
            Delay::Slow(vec![
                (Site::MonthInsert(0, 1), 0.7),
                (Site::MonthInsert(0, 2), 0.7),
            ]),
        ),
        (
            "a held state row".to_string(),
            Delay::Held(
                "SELECT * FROM metric_rollup_work_day_state WHERE state_id = 1 FOR UPDATE"
                    .to_string(),
            ),
        ),
        (
            "a held work row".to_string(),
            Delay::Held(format!(
                "SELECT work_id FROM work WHERE work_id = '{}' FOR UPDATE",
                works[3]
            )),
        ),
        (
            "a held run row".to_string(),
            Delay::Held(format!(
                "SELECT * FROM metric_rollup_month_rebuild_run WHERE run_id = '{}' FOR UPDATE",
                run.run_id
            )),
        ),
        (
            "a held projection row".to_string(),
            Delay::Held(
                "UPDATE metric_rollup_work_country_month SET value = value \
                 WHERE rollup_work_country_month_id = (SELECT rollup_work_country_month_id \
                 FROM metric_rollup_work_country_month ORDER BY work_id DESC LIMIT 1)"
                    .to_string(),
            ),
        ),
        (
            "a held foreign-key parent publication".to_string(),
            Delay::Held(format!(
                "SELECT publication_id FROM publication WHERE publication_id = '{publication}' \
                 FOR UPDATE"
            )),
        ),
        (
            "a held foreign-key parent institution".to_string(),
            Delay::Held(format!(
                "SELECT institution_id FROM institution WHERE institution_id = '{}' FOR UPDATE",
                corpus.institutions[1]
            )),
        ),
    ];
    for site in replacing_step_sites() {
        cases.push((
            format!("a slow statement at {site:?}"),
            Delay::Slow(vec![(site, 30.0)]),
        ));
    }
    for (label, delay) in &cases {
        let (result, state_hold, work_hold) = measure_step(&timed, &run, &budgets, delay);
        assert!(
            matches!(
                result,
                Err(Error::Retryable(
                    reason::STATEMENT_TIMEOUT | reason::LOCK_TIMEOUT | reason::BUDGET
                ))
            ),
            "{label}: {result:?}"
        );
        assert!(
            state_hold <= budgets.step_hold + margin,
            "{label}: state-row hold {state_hold:?}"
        );
        assert!(
            work_hold <= budgets.step_lock + budgets.step_hold + margin,
            "{label}: work-row hold {work_hold:?}"
        );
        assert_eq!(run_row(&pool, run.run_id), row, "{label}");
    }
}

#[test]
fn a_page_never_exceeds_its_budget() {
    let (_guard, pool) = setup();
    let (_corpus, _works, _frontier) = built_layered(&pool);
    let budgets = short(1_000);
    let margin = Duration::from_millis(400);
    let run = start(&pool, Verify);
    let row = run_row(&pool, run.run_id);
    let timed = timed_pool();
    let mut cases: Vec<(String, Delay)> = vec![
        (
            "two slow statements".to_string(),
            Delay::Slow(vec![(Site::PageFrontier, 0.8), (Site::PageVerify, 0.8)]),
        ),
        (
            "a held run row".to_string(),
            Delay::Held(format!(
                "SELECT * FROM metric_rollup_month_rebuild_run WHERE run_id = '{}' FOR UPDATE",
                run.run_id
            )),
        ),
        (
            "an exclusive lock on the work-day table".to_string(),
            Delay::Held("LOCK TABLE metric_rollup_work_day IN ACCESS EXCLUSIVE MODE".to_string()),
        ),
    ];
    for site in [
        Site::PageLock,
        Site::Isolation(Stage::Page),
        Site::RunLock(Stage::Page),
        Site::PageFrontier,
        Site::Sizing(Stage::Page),
        Site::PageVerify,
        Site::RunUpdate(Stage::Page),
    ] {
        cases.push((
            format!("a slow statement at {site:?}"),
            Delay::Slow(vec![(site, 30.0)]),
        ));
    }
    for (label, delay) in &cases {
        timed.1.lock().expect("timed log").clear();
        let (result, _, _) = measure_step(&timed, &run, &budgets, delay);
        let end = Instant::now();
        let begun = finished(
            &timed.1,
            "BEGIN TRANSACTION ISOLATION LEVEL REPEATABLE READ",
        )
        .expect("the page began");
        assert!(
            matches!(
                result,
                Err(Error::Retryable(
                    reason::STATEMENT_TIMEOUT | reason::LOCK_TIMEOUT | reason::BUDGET
                ))
            ),
            "{label}: {result:?}"
        );
        assert!(
            end - begun <= budgets.page + margin,
            "{label}: page time {:?}",
            end - begun
        );
        assert_eq!(run_row(&pool, run.run_id), row, "{label}");
    }
}

// ===========================================================================
// 17. Range verifier, and 18. pages write only the run row
// ===========================================================================

#[test]
fn the_rendered_whole_table_verification_statement_is_byte_identical() {
    assert_eq!(VERIFICATION_SQL.len(), VERIFICATION_SQL_BYTES);
    assert_eq!(
        hex::encode(Sha256::digest(VERIFICATION_SQL.as_bytes())),
        VERIFICATION_SQL_SHA256
    );
}

#[test]
fn the_expected_step_count_is_a_bounded_decimal_string() {
    assert_eq!(parse_expected_step_count("0"), Ok(0));
    assert_eq!(
        parse_expected_step_count("9223372036854775807"),
        Ok(i64::MAX)
    );
    for invalid in [
        "",
        "-1",
        "+1",
        " 1",
        "1 ",
        "1e3",
        "0x10",
        "9223372036854775808",
        "99999999999999999999",
    ] {
        assert_eq!(
            parse_expected_step_count(invalid),
            Err(Error::QueryInvalid),
            "{invalid:?}"
        );
    }
}

#[test]
fn the_range_verifier_over_everything_equals_the_whole_table_verification() {
    let (_guard, pool) = setup();
    let (corpus, works, frontier) = built_layered(&pool);
    for corruption in corruptions(&pool, &works, frontier).into_iter().take(9) {
        exec_count(&pool, &corruption.statement);
    }
    insert_orphans(&pool, &corpus, &works);
    let whole = verification(&pool);
    assert!(!whole.matches);
    let range = verify(&pool, None, None);
    assert_eq!(range_families(&range), whole_families(&pool));
    assert_eq!(range.work_day_rows, whole.work_day_row_count);
    assert_eq!(range.month_keys, whole.represented_month_key_count);
    assert!(!range.work_day_above(frontier));
    assert!(!range.exact_at(frontier));
    rebuild(&pool, frontier).expect("rebuild");
    let range = verify(&pool, None, None);
    assert_eq!(range_families(&range), whole_families(&pool));
    assert!(range.exact_at(frontier));
}

#[test]
fn the_range_verifier_shares_no_sql_with_the_writer_and_only_reads() {
    let upper = RANGE_VERIFICATION_SQL.to_uppercase();
    assert!(upper.starts_with("WITH") || upper.starts_with("SELECT"));
    for forbidden in [
        "INSERT ",
        "UPDATE ",
        "DELETE ",
        "TRUNCATE",
        "FOR UPDATE",
        "FOR SHARE",
        "FOR KEY SHARE",
        "UNNEST(",
    ] {
        assert!(!upper.contains(forbidden), "{forbidden}");
    }
    let writers: Vec<&str> = super::crud::MONTH_MAINTENANCE_STATEMENTS
        .iter()
        .chain(super::crud::YEAR_MAINTENANCE_STATEMENTS.iter())
        .chain(super::crud::REBUILD_YEAR_STATEMENTS.iter())
        .chain(RANGE_DELETE_STATEMENTS.iter())
        .copied()
        .chain([RANGE_MONTH_KEYS_SQL, SIZING_SQL, PER_YEAR_SQL])
        .collect();
    for writer in &writers {
        assert!(!RANGE_VERIFICATION_SQL.contains(writer));
        assert!(!writer.contains(RANGE_VERIFICATION_SQL));
        for fragment in [
            "expected_total AS",
            "expected_country_year AS",
            "expected_institution_year AS",
            "range_stats AS",
        ] {
            assert!(!writer.contains(fragment), "{fragment}");
        }
    }
    // The expected yearly families come from the work-day rows, never from
    // the monthly tables the yearly writer reads; each monthly section table
    // is read once, as the actual side of its comparison.
    assert!(RANGE_VERIFICATION_SQL.contains("expected_country_year AS"));
    assert!(RANGE_VERIFICATION_SQL.contains("date_trunc('year', r.day)::date AS year_start"));
    for monthly in [
        "metric_rollup_work_country_month",
        "metric_rollup_work_institution_month",
    ] {
        assert_eq!(
            RANGE_VERIFICATION_SQL.matches(monthly).count(),
            1,
            "{monthly}"
        );
    }
}

#[test]
fn a_page_writes_only_the_run_row() {
    let (_guard, pool) = setup();
    let (_corpus, _works, _frontier) = built_layered(&pool);
    corrupt_one_value(&pool);
    let budgets = keys(12);
    let mut run = start(&pool, Verify);
    let work_day = || {
        text_scalar(
            &pool,
            "SELECT string_agg(ctid::text || '/' || xmin::text, ',' ORDER BY ctid) \
             FROM metric_rollup_work_day",
        )
    };
    let state_row = || {
        text_scalar(
            &pool,
            "SELECT ctid::text || '/' || xmin::text FROM metric_rollup_work_day_state",
        )
    };
    while run.status.is_open() {
        let prints = fingerprints(&pool);
        let days = work_day();
        let frontier_row = state_row();
        let before = run_row(&pool, run.run_id);
        run = advance(&pool, &run, &budgets);
        assert_eq!(fingerprints(&pool), prints);
        assert_eq!(work_day(), days);
        assert_eq!(state_row(), frontier_row);
        assert_ne!(run_row(&pool, run.run_id), before, "the run row moved");
    }
    assert_eq!(run.status, Mismatched);
}

// ===========================================================================
// 19. Lock order and modes, and 25. the page statement sequence
// ===========================================================================

/// The relation lock modes `pid` holds in `public`.
fn modes_held(pool: &PgPool, pid: i32) -> BTreeSet<String> {
    text_scalar(
        pool,
        &format!(
            "SELECT string_agg(DISTINCT l.mode, ',') FROM pg_locks l \
             JOIN pg_class c ON c.oid = l.relation \
             WHERE l.pid = {pid} AND l.locktype = 'relation' \
               AND c.relnamespace = 'public'::regnamespace"
        ),
    )
    .unwrap_or_default()
    .split(',')
    .filter(|mode| !mode.is_empty())
    .map(str::to_string)
    .collect()
}

/// The backend holding a row lock on the run table (a step or a page).
fn run_row_holder(pool: &PgPool) -> i32 {
    text_scalar(
        pool,
        "SELECT pid FROM pg_locks WHERE locktype = 'relation' AND granted \
         AND relation = 'public.metric_rollup_month_rebuild_run'::regclass \
         AND mode = 'RowShareLock' LIMIT 1",
    )
    .expect("a holder")
    .parse()
    .expect("a pid")
}

#[test]
fn steps_and_pages_take_no_exclusive_lock_and_lock_works_then_state_then_run() {
    let (_guard, pool) = setup();
    let (_corpus, _works, _frontier) = layered(&pool);
    let budgets = keys(12);
    let run = start(&pool, Rebuild);

    // A step paused at its pre-verify holds every lock it will take before
    // writing; afterwards its log shows the order.
    let (paused, pause, log) = pausing_pool(RANGE_VERIFICATION_SQL);
    let stepping = {
        let run = run.clone();
        thread::spawn(move || try_advance(&paused, &run, &budgets))
    };
    pause.wait_reached();
    let step_pid = run_row_holder(&pool);
    let modes = modes_held(&pool, step_pid);
    assert!(
        modes
            .iter()
            .all(|mode| mode == "AccessShareLock" || mode == "RowShareLock"),
        "{modes:?}"
    );
    pause.release();
    let run = stepping.join().expect("join").expect("the step").run;
    let statements = captured(&log);
    let position = |prefix: &str| {
        statements
            .iter()
            .position(|s| s.starts_with(prefix))
            .unwrap_or_else(|| panic!("{prefix}"))
    };
    assert!(position(LOCK_WORKS_SQL) < position(LOCK_STATE_SQL));
    assert!(position(LOCK_STATE_SQL) < position(RUN_LOCK_SQL));
    for statement in &statements {
        let upper = statement.to_uppercase();
        assert!(!upper.contains("ACCESS EXCLUSIVE"), "{statement}");
        assert!(!upper.starts_with("TRUNCATE"), "{statement}");
        assert!(!upper.starts_with("LOCK TABLE"), "{statement}");
    }

    // A replacing step's own writes take ROW EXCLUSIVE and nothing
    // stronger.
    let run = drive_build(&pool, run, &budgets);
    let (paused, pause, log) = pausing_pool(PAGE_FRONTIER_SQL);
    let paging = {
        let run = run.clone();
        thread::spawn(move || try_advance(&paused, &run, &budgets))
    };
    pause.wait_reached();
    let page_pid = run_row_holder(&pool);
    let modes = modes_held(&pool, page_pid);
    assert!(
        modes
            .iter()
            .all(|mode| mode == "AccessShareLock" || mode == "RowShareLock"),
        "{modes:?}"
    );
    pause.release();
    paging.join().expect("join").expect("the page");
    for statement in captured(&log) {
        assert!(
            !statement.to_uppercase().contains("ACCESS EXCLUSIVE"),
            "{statement}"
        );
    }
}

#[test]
fn a_page_issues_only_set_local_then_its_table_lock_then_the_isolation_check() {
    let (_guard, pool) = setup();
    let (_corpus, _works, _frontier) = built_layered(&pool);
    let run = start(&pool, Verify);
    let (logged, log) = logging_pool();
    advance(&logged, &run, &keys(12));
    let statements = captured(&log);
    let begin = statements
        .iter()
        .position(|s| s.starts_with("BEGIN TRANSACTION ISOLATION LEVEL REPEATABLE READ"))
        .expect("the page's transaction");
    let page = &statements[begin + 1..];
    let lock = page
        .iter()
        .position(|s| s == PAGE_LOCK_SQL)
        .expect("the table lock");
    assert!(
        page[..lock].iter().all(|s| is_set_local(s)),
        "{:?}",
        &page[..lock]
    );
    assert!(page[..lock]
        .iter()
        .any(|s| s.starts_with("SET LOCAL statement_timeout")));
    let first_query = page[lock + 1..]
        .iter()
        .find(|s| !is_set_local(s))
        .expect("a first query");
    assert_eq!(first_query, ISOLATION_SQL);
    let locked: Vec<&str> = PAGE_LOCK_SQL
        .trim_start_matches("LOCK TABLE ")
        .trim_end_matches(" IN ACCESS SHARE MODE")
        .split(',')
        .map(str::trim)
        .collect();
    assert_eq!(
        locked,
        vec![
            "public.metric_rollup_work_month",
            "public.metric_rollup_work_country_month",
            "public.metric_rollup_work_institution_month",
            "public.metric_rollup_work_month_ambiguity",
            "public.metric_rollup_work_country_year",
            "public.metric_rollup_work_institution_year",
            "public.metric_rollup_work_day",
            "public.metric_rollup_work_day_state",
        ]
    );
}

// ===========================================================================
// 20. More than 100,000 keys
// ===========================================================================

/// `works` works with `keys_per_work` month keys each, one work-day row per
/// key, on one series: a sparse corpus. Returns the frontier.
fn seed_sparse(pool: &PgPool, corpus: &Corpus, works: i64, keys_per_work: i64) -> i64 {
    exec(
        pool,
        &format!(
            "INSERT INTO work (work_id, work_type, work_status, imprint_id, edition) \
             SELECT gen_random_uuid(), 'monograph', 'forthcoming', imprint_id, 1 \
             FROM work, generate_series(1, {works}) WHERE work_id = '{}'",
            corpus.fixture.work_id
        ),
    );
    let base = state(pool).applied_through_sequence;
    exec(
        pool,
        &format!(
            "INSERT INTO metric_rollup_work_day (work_id, publication_id, platform_id, \
                 measure_id, day, country_code, institution_id, value, watermark) \
             SELECT w.work_id, NULL, '{platform}', '{measure}', \
                    (DATE '2010-01-03' + make_interval(months => k))::date, NULL, NULL, \
                    1 + k % 7, \
                    {base} + row_number() OVER (ORDER BY w.work_id, k) \
             FROM work w CROSS JOIN generate_series(0, {last}) AS k \
             WHERE w.work_id <> '{base_work}'",
            platform = corpus.platforms[0],
            measure = corpus.fixture.measure_id,
            last = keys_per_work - 1,
            base_work = corpus.fixture.work_id,
        ),
    );
    exec(pool, "ANALYZE metric_rollup_work_day");
    settle(pool)
}

#[test]
fn a_sparse_corpus_above_one_hundred_thousand_keys_builds_to_reconciled() {
    let (_guard, pool) = setup();
    let corpus = corpus(&pool, 1);
    let frontier = seed_sparse(&pool, &corpus, 1_001, 100);
    let keys = scalar_i64(
        &pool,
        "(SELECT COUNT(DISTINCT (work_id, platform_id, measure_id, \
                                 date_trunc('month', day))) FROM metric_rollup_work_day)",
    );
    assert!(keys > METRIC_ROLLUP_REBUILD_MAX_MONTH_KEYS, "{keys}");
    assert!(
        rebuild(&pool, frontier).is_err(),
        "the OPS-02 rebuild refuses the corpus"
    );
    assert!(month_rows(&pool).is_empty());

    let run = drive(&pool, start(&pool, Rebuild), &RunBudgets::PRODUCTION);
    assert_eq!(run.status, Reconciled);
    let build = run.build.clone().expect("build");
    assert_eq!(build.month_keys, keys);
    assert_eq!(run.verify.month_keys, keys);
    assert!(build.steps >= keys / METRIC_ROLLUP_RUN_STEP_MAX_MONTH_KEYS);
    assert!(verification(&pool).matches);
}

// ===========================================================================
// 22. Statement fidelity (differential)
// ===========================================================================

#[test]
fn a_replacing_step_issues_the_completions_own_statements_with_one_deadline_each() {
    let (_guard, pool) = setup();
    let corpus = corpus(&pool, 2);
    let works: Vec<Uuid> = (0..10).map(|n| controlled_work(0x20 + n * 8)).collect();
    for work_id in &works {
        add_work(&pool, &corpus, *work_id);
    }
    let units: Vec<(Uuid, i32)> = works
        .iter()
        .flat_map(|work_id| [(*work_id, 2023), (*work_id, 2024), (*work_id, 2025)])
        .collect();
    let frontier = seed_cells(&pool, &corpus, &cells(&corpus, &units, &[2, 9], &[3]));
    assert!(rebuild(&pool, frontier).expect("rebuild").rebuilt);
    // An extra row and an orphan in the range; every family populated.
    exec(
        &pool,
        "INSERT INTO metric_rollup_work_country_month (work_id, publication_id, platform_id, \
             measure_id, month_start, country_code, value, requires_institution_coverage, \
             watermark) \
         SELECT work_id, publication_id, platform_id, measure_id, month_start, 'FR', 5, false, 1 \
         FROM metric_rollup_work_country_month ORDER BY work_id LIMIT 1",
    );
    insert_orphans_at(&pool, &corpus, controlled_work(0x24), (works[1], 2022));
    for table in DERIVED_TABLES {
        assert!(
            scalar_i64(&pool, &format!("(SELECT COUNT(*) FROM {table})")) > 0,
            "{table}"
        );
    }
    let budgets = RunBudgets::PRODUCTION;
    let sized = size_range_for_test(&pool, None, budgets.step_size()).expect("sizing");
    assert_eq!(sized.hi, None, "one step covers the fixture");
    assert!(sized.month_keys > 2 * METRIC_ROLLUP_RUN_STEP_CHUNK_KEYS as i64);

    let run = start(&pool, Rebuild);
    let (logged, log) = logging_pool();
    let stepped = advance(&logged, &run, &budgets);
    assert_eq!(stepped.build.clone().expect("build").replaced_steps, 1);
    let raw: Vec<String> = log.lock().expect("statement log").clone();

    // (a) One SET LOCAL statement_timeout immediately before every other
    // statement from the first pre-sizing statement to the run update,
    // never above its phase's remaining budget and never increasing within
    // a phase.
    let first = raw
        .iter()
        .position(|s| s.starts_with(ISOLATION_SQL))
        .expect("the first pre-sizing statement");
    let last = raw
        .iter()
        .position(|s| s.starts_with(RUN_UPDATE_PREFIX))
        .expect("the run update");
    let mut phase_values: BTreeMap<&str, Vec<u128>> = BTreeMap::new();
    let mut phase = "presize";
    let mut index = first;
    while index <= last {
        let statement = &raw[index];
        assert!(!is_transaction_control(statement), "{statement}");
        assert!(
            !is_set_local(statement),
            "an unexpected SET LOCAL inside the step: {statement}"
        );
        if statement.starts_with(LOCK_WORKS_SQL) {
            phase = "lock";
        } else if statement.starts_with(RUN_LOCK_SQL) {
            phase = "hold";
        }
        let previous = &raw[index - 1];
        let value = timeout_value(previous)
            .unwrap_or_else(|| panic!("no deadline before {statement}: {previous}"));
        if index > first {
            assert!(
                timeout_value(&raw[index - 2]).is_none(),
                "exactly one deadline before {statement}"
            );
        }
        phase_values.entry(phase).or_default().push(value);
        // Skip the statement and the next statement's deadline.
        index += 2;
    }
    for (phase, budget) in [
        ("presize", budgets.step_presize),
        ("lock", budgets.step_lock),
        ("hold", budgets.step_hold),
    ] {
        let values = &phase_values[phase];
        assert!(values
            .iter()
            .all(|value| *value >= 1 && *value <= budget.as_millis()));
        assert!(
            values.windows(2).all(|pair| pair[1] <= pair[0]),
            "{phase}: {values:?}"
        );
    }

    // (b) Between the range's month-key listing and the post-verify, the
    // completion's own statements, texts, order and rendered binds, for the
    // same chunked keys, minus its six keyed deletes.
    let keys_at = raw
        .iter()
        .position(|s| s.starts_with(RANGE_MONTH_KEYS_SQL))
        .expect("the month-key listing");
    let post = raw
        .iter()
        .rposition(|s| s.starts_with(RANGE_VERIFICATION_SQL))
        .expect("the post-verify");
    let segment: Vec<String> = raw[keys_at + 1..post]
        .iter()
        .filter(|s| !is_set_local(s))
        .cloned()
        .collect();
    let month_keys: Vec<MonthKey> = {
        let mut keys = BTreeSet::new();
        for row in projection(&pool) {
            keys.insert((
                row.work_id,
                row.platform_id,
                row.measure_id,
                NaiveDate::from_ymd_opt(row.day.year(), row.day.month(), 1).expect("month"),
            ));
        }
        keys.into_iter().collect()
    };
    let year_keys: Vec<YearKey> = affected_year_keys(&month_keys.iter().copied().collect())
        .into_iter()
        .collect();
    assert!(
        year_keys.len() > METRIC_ROLLUP_RUN_STEP_CHUNK_KEYS,
        "two year chunks"
    );
    let (capture, captured_log) = logging_pool();
    {
        let mut connection = capture.get().expect("a connection");
        let _ = connection.transaction::<(), DieselError, _>(|c| {
            for chunk in month_keys.chunks(METRIC_ROLLUP_RUN_STEP_CHUNK_KEYS) {
                recompute_month_projections(c, &chunk.iter().copied().collect(), frontier)
                    .expect("month recompute");
            }
            for chunk in year_keys.chunks(METRIC_ROLLUP_RUN_STEP_CHUNK_KEYS) {
                recompute_year_projections(c, &chunk.iter().copied().collect())
                    .expect("year recompute");
            }
            Err(DieselError::RollbackTransaction)
        });
    }
    let keyed_deletes: Vec<&str> = MONTH_DELETE_STATEMENTS
        .iter()
        .copied()
        .chain([YEAR_COUNTRY_DELETE_SQL, YEAR_INSTITUTION_DELETE_SQL])
        .collect();
    let expected: Vec<String> = captured_log
        .lock()
        .expect("statement log")
        .iter()
        .filter(|s| {
            // The pool's checkout validation is not part of the recompute.
            !is_transaction_control(s)
                && text_of(s) != "SELECT 1"
                && !keyed_deletes.iter().any(|delete| s.starts_with(delete))
        })
        .cloned()
        .collect();
    assert_eq!(segment, expected, "the same statements, order and binds");

    // (c) In a separate transaction, after the range delete, the keyed
    // deletes before each chunk's inserts affect zero rows; before the
    // range delete they would not.
    let mut connection = pool.get().expect("a connection");
    let _ = connection.transaction::<(), DieselError, _>(|c| {
        let chunk = &month_keys[..METRIC_ROLLUP_RUN_STEP_CHUNK_KEYS];
        let arrays = month_arrays(chunk);
        let before: usize = MONTH_DELETE_STATEMENTS
            .iter()
            .map(|statement| keyed(c, statement, &arrays))
            .sum();
        assert!(before > 0, "the zero-row check below is not trivial");
        Err(DieselError::RollbackTransaction)
    });
    let _ = connection.transaction::<(), DieselError, _>(|c| {
        let bounds = range_bounds(None, None);
        for statement in RANGE_DELETE_STATEMENTS {
            sql_query(statement)
                .bind::<SqlUuid, _>(bounds.lo_work_id)
                .bind::<Nullable<Date>, _>(bounds.lo_end)
                .bind::<SqlUuid, _>(bounds.hi_work_id)
                .bind::<Nullable<Date>, _>(bounds.hi_end)
                .execute(c)
                .expect("range delete");
        }
        for chunk in month_keys.chunks(METRIC_ROLLUP_RUN_STEP_CHUNK_KEYS) {
            let arrays = month_arrays(chunk);
            for statement in MONTH_DELETE_STATEMENTS {
                assert_eq!(keyed(c, statement, &arrays), 0, "{statement}");
            }
            for statement in super::crud::MONTH_INSERT_STATEMENTS {
                keyed(c, statement, &arrays);
            }
        }
        for chunk in year_keys.chunks(METRIC_ROLLUP_RUN_STEP_CHUNK_KEYS) {
            let arrays = month_arrays(chunk);
            for statement in [YEAR_COUNTRY_DELETE_SQL, YEAR_INSTITUTION_DELETE_SQL] {
                assert_eq!(keyed(c, statement, &arrays), 0, "{statement}");
            }
            for statement in [
                super::crud::YEAR_COUNTRY_INSERT_SQL,
                super::crud::YEAR_INSTITUTION_INSERT_SQL,
            ] {
                keyed(c, statement, &arrays);
            }
        }
        Err(DieselError::RollbackTransaction)
    });
}

type KeyArrays = (Vec<Uuid>, Vec<Uuid>, Vec<Uuid>, Vec<NaiveDate>);

fn month_arrays(keys: &[(Uuid, Uuid, Uuid, NaiveDate)]) -> KeyArrays {
    (
        keys.iter().map(|key| key.0).collect(),
        keys.iter().map(|key| key.1).collect(),
        keys.iter().map(|key| key.2).collect(),
        keys.iter().map(|key| key.3).collect(),
    )
}

/// Execute one keyed statement bound to four key arrays.
fn keyed(connection: &mut PgConnection, statement: &str, arrays: &KeyArrays) -> usize {
    sql_query(statement)
        .bind::<Array<SqlUuid>, _>(&arrays.0)
        .bind::<Array<SqlUuid>, _>(&arrays.1)
        .bind::<Array<SqlUuid>, _>(&arrays.2)
        .bind::<Array<Date>, _>(&arrays.3)
        .execute(connection)
        .unwrap_or_else(|error| panic!("{statement}: {error}"))
}

// ===========================================================================
// 23. Sizing, and 24. re-size
// ===========================================================================

/// Direct per-unit counts from the work-day rows: rows, month keys and the
/// replacing-read figure, the year term using the work's whole history.
fn direct_units(pool: &PgPool) -> BTreeMap<Unit, (i64, i64, i64)> {
    let days = projection(pool);
    let mut whole: BTreeMap<Uuid, i64> = BTreeMap::new();
    let mut month_rows: BTreeMap<(Uuid, NaiveDate), i64> = BTreeMap::new();
    let mut month_series: BTreeMap<(Uuid, NaiveDate), BTreeSet<(Uuid, Uuid)>> = BTreeMap::new();
    let mut year_series: BTreeMap<Unit, BTreeSet<(Uuid, Uuid)>> = BTreeMap::new();
    for row in &days {
        let month = NaiveDate::from_ymd_opt(row.day.year(), row.day.month(), 1).expect("month");
        *whole.entry(row.work_id).or_default() += 1;
        *month_rows.entry((row.work_id, month)).or_default() += 1;
        month_series
            .entry((row.work_id, month))
            .or_default()
            .insert((row.platform_id, row.measure_id));
        year_series
            .entry(unit(row.work_id, row.day.year()))
            .or_default()
            .insert((row.platform_id, row.measure_id));
    }
    let mut units: BTreeMap<Unit, (i64, i64, i64)> = BTreeMap::new();
    for ((work_id, month), rows) in &month_rows {
        let keys = month_series[&(*work_id, *month)].len() as i64;
        let entry = units.entry(unit(*work_id, month.year())).or_default();
        entry.0 += rows;
        entry.1 += keys;
        entry.2 += keys * rows;
    }
    for (unit, counts) in units.iter_mut() {
        counts.2 += year_series[unit].len() as i64 * whole[&unit.work_id];
    }
    units
}

/// The direct totals of the units in `(lo, hi]`.
fn direct_range(
    units: &BTreeMap<Unit, (i64, i64, i64)>,
    lo: Option<Unit>,
    hi: Option<Unit>,
) -> (i64, i64, i64, i64) {
    units
        .iter()
        .filter(|(unit, _)| lo.is_none_or(|lo| **unit > lo) && hi.is_none_or(|hi| **unit <= hi))
        .fold((0, 0, 0, 0), |acc, (_, counts)| {
            (
                acc.0 + 1,
                acc.1 + counts.0,
                acc.2 + counts.1,
                acc.3 + counts.2,
            )
        })
}

fn assert_direct(sized: &SizedRange, units: &BTreeMap<Unit, (i64, i64, i64)>, label: &str) {
    let (count, rows, keys, figure) = direct_range(units, sized.lo, sized.hi);
    assert_eq!(
        (
            sized.units,
            sized.work_day_rows,
            sized.month_keys,
            sized.replacing_read
        ),
        (count, rows, keys, figure),
        "{label}"
    );
}

fn budgets_of(work_day_rows: i64, month_keys: i64, replacing_read: i64) -> SizeBudgets {
    SizeBudgets {
        work_day_rows,
        month_keys,
        replacing_read,
    }
}

/// A heavy work over five years, then two light works.
fn sizing_fixture(pool: &PgPool) -> (Corpus, [Uuid; 3]) {
    let corpus = corpus(pool, 3);
    let works = [
        controlled_work(0x30),
        controlled_work(0x50),
        controlled_work(0x70),
    ];
    for work_id in works {
        add_work(pool, &corpus, work_id);
    }
    let heavy: Vec<(Uuid, i32)> = (2021..=2025).map(|year| (works[0], year)).collect();
    seed_cells(
        pool,
        &corpus,
        &cells(&corpus, &heavy, &(1..=12).collect::<Vec<_>>(), &[1, 10, 20]),
    );
    seed_cells(
        pool,
        &corpus,
        &cells(&corpus, &[(works[1], 2024)], &[1, 6], &[1]),
    );
    seed_cells(
        pool,
        &corpus,
        &cells(
            &corpus,
            &[(works[2], 2023), (works[2], 2025)],
            &[3],
            &[1, 2],
        ),
    );
    (corpus, works)
}

#[test]
fn sizing_cuts_at_each_budget_and_its_figure_equals_a_direct_count() {
    let (_guard, pool) = setup();
    let (_corpus, works) = sizing_fixture(&pool);
    let units = direct_units(&pool);
    let unlimited = budgets_of(i64::MAX / 4, i64::MAX / 4, i64::MAX / 4);

    // Everything, and from a cursor inside the heavy work whose earlier
    // years hold rows.
    let all = size_range_for_test(&pool, None, unlimited).expect("sizing");
    assert_eq!(all.hi, None);
    assert_eq!(all.statements, 1);
    assert_direct(&all, &units, "everything");
    let inside = size_range_for_test(&pool, Some(unit(works[0], 2022)), unlimited).expect("sizing");
    assert_eq!(inside.first_unit, Some(unit(works[0], 2023)));
    assert_eq!(inside.hi, None);
    assert_direct(&inside, &units, "a cursor inside a work");

    // A work alone over a budget: the per-year statement takes its years
    // while they fit, again from inside the work.
    let heavy_unit = units[&unit(works[0], 2023)];
    let split = size_range_for_test(
        &pool,
        Some(unit(works[0], 2021)),
        budgets_of(i64::MAX / 4, heavy_unit.1 * 2 + 1, i64::MAX / 4),
    )
    .expect("sizing");
    assert_eq!(split.statements, 2, "the sizing and the per-year statement");
    assert_eq!(split.units, 2);
    assert_eq!(split.hi, Some(unit(works[0], 2023)));
    assert_direct(&split, &units, "a work alone over the key budget");

    // A unit alone over a budget is still one unit.
    let alone = size_range_for_test(&pool, None, budgets_of(10, i64::MAX / 4, i64::MAX / 4))
        .expect("sizing");
    assert_eq!(alone.units, 1);
    assert_eq!(alone.hi, Some(unit(works[0], 2021)));
    assert_direct(&alone, &units, "a unit alone over the row budget");

    // Units that each fit the replacing-read budget but together exceed it
    // are cut at it.
    let light_b = units[&unit(works[1], 2024)].2;
    let light_c: i64 = units
        .iter()
        .filter(|(unit, _)| unit.work_id == works[2])
        .map(|(_, counts)| counts.2)
        .sum();
    let cut = size_range_for_test(
        &pool,
        Some(unit(works[0], 2025)),
        budgets_of(i64::MAX / 4, i64::MAX / 4, light_b + light_c - 1),
    )
    .expect("sizing");
    assert_eq!(cut.hi, Some(unit(works[1], 2024)));
    assert_eq!(cut.replacing_read, light_b);
    assert_direct(&cut, &units, "cut at the replacing-read budget");
    // Exactly filled: the range ends at the filling work.
    let filled = size_range_for_test(
        &pool,
        Some(unit(works[0], 2025)),
        budgets_of(i64::MAX / 4, i64::MAX / 4, light_b),
    )
    .expect("sizing");
    assert_eq!(filled.hi, Some(unit(works[1], 2024)));
}

#[test]
fn the_run_grant_aggregate_returns_exactly_the_units_sizing_finds_over_a_step_budget() {
    let (_guard, pool) = setup();
    let (_corpus, works) = sizing_fixture(&pool);
    let units = direct_units(&pool);
    #[derive(QueryableByName, Debug, PartialEq, Eq)]
    struct AggregateRow {
        #[diesel(sql_type = Nullable<SqlUuid>)]
        work_id: Option<Uuid>,
        #[diesel(sql_type = Nullable<Date>)]
        year_start: Option<NaiveDate>,
        #[diesel(sql_type = Nullable<diesel::sql_types::BigInt>)]
        work_day_rows: Option<i64>,
        #[diesel(sql_type = Nullable<diesel::sql_types::BigInt>)]
        month_keys: Option<i64>,
        #[diesel(sql_type = Nullable<diesel::sql_types::BigInt>)]
        replacing_read_figure: Option<i64>,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        max_month_keys_per_work_month: i64,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        max_whole_work_rows: i64,
    }
    let aggregate = |budgets: SizeBudgets| -> Vec<AggregateRow> {
        let mut connection = pool.get().expect("a connection");
        sql_query(METRIC_ROLLUP_RUN_GRANT_AGGREGATE_SQL)
            .bind::<diesel::sql_types::BigInt, _>(budgets.work_day_rows)
            .bind::<diesel::sql_types::BigInt, _>(budgets.month_keys)
            .bind::<diesel::sql_types::BigInt, _>(budgets.replacing_read)
            .load(&mut connection)
            .expect("the aggregate")
    };
    let max_keys = {
        let days = projection(&pool);
        let mut series: BTreeMap<(Uuid, i32, u32), BTreeSet<(Uuid, Uuid)>> = BTreeMap::new();
        for row in &days {
            series
                .entry((row.work_id, row.day.year(), row.day.month()))
                .or_default()
                .insert((row.platform_id, row.measure_id));
        }
        series.values().map(BTreeSet::len).max().unwrap_or(0) as i64
    };
    let max_whole = works
        .iter()
        .map(|work_id| {
            units
                .iter()
                .filter(|(u, _)| u.work_id == *work_id)
                .map(|(_, c)| c.0)
                .sum::<i64>()
        })
        .max()
        .unwrap_or(0);
    let heavy = units[&unit(works[0], 2023)];
    for (label, budgets) in [
        ("rows", budgets_of(heavy.0 - 1, i64::MAX / 4, i64::MAX / 4)),
        ("keys", budgets_of(i64::MAX / 4, heavy.1 - 1, i64::MAX / 4)),
        (
            "figure",
            budgets_of(
                i64::MAX / 4,
                i64::MAX / 4,
                units[&unit(works[1], 2024)].2 - 1,
            ),
        ),
        ("none", budgets_of(i64::MAX / 4, i64::MAX / 4, i64::MAX / 4)),
    ] {
        let rows = aggregate(budgets);
        let over: Vec<(Unit, (i64, i64, i64))> = units
            .iter()
            .filter(|(_, c)| {
                c.0 > budgets.work_day_rows
                    || c.1 > budgets.month_keys
                    || c.2 > budgets.replacing_read
            })
            .map(|(u, c)| (*u, *c))
            .collect();
        if over.is_empty() {
            assert_eq!(rows.len(), 1, "{label}");
            assert!(
                rows[0].work_id.is_none() && rows[0].year_start.is_none(),
                "{label}"
            );
        } else {
            let found: Vec<(Unit, (i64, i64, i64))> = rows
                .iter()
                .map(|row| {
                    (
                        unit(
                            row.work_id.expect("unit"),
                            row.year_start.expect("unit").year(),
                        ),
                        (
                            row.work_day_rows.expect("rows"),
                            row.month_keys.expect("keys"),
                            row.replacing_read_figure.expect("figure"),
                        ),
                    )
                })
                .collect();
            assert_eq!(found, over, "{label}");
            // Sizing, from just before each unit, finds the same counts:
            // the per-year statement for a work over the budget, the
            // sizing statement for a single-unit work.
            for (over_unit, counts) in &over {
                let lo = unit(over_unit.work_id, over_unit.year_start.year() - 1);
                let sized = size_range_for_test(&pool, Some(lo), budgets).expect("sizing");
                assert_eq!(sized.first_unit, Some(*over_unit), "{label}");
                assert_eq!(
                    (sized.work_day_rows, sized.month_keys, sized.replacing_read),
                    *counts,
                    "{label}: {over_unit:?}"
                );
                assert_eq!(sized.units, 1, "{label}: a unit over a budget is alone");
            }
        }
        for row in &rows {
            assert_eq!(row.max_month_keys_per_work_month, max_keys, "{label}");
            assert_eq!(row.max_whole_work_rows, max_whole, "{label}");
        }
    }
}

#[test]
fn the_register_quotes_the_run_grant_aggregate_and_the_step_budgets_exactly() {
    let register = include_str!("../../../../docs/metrics/contract-register.md");
    let section = register
        .split_once("### 3.8")
        .expect("register section 3.8")
        .1
        .split_once("\n### ")
        .map_or_else(
            || register.split_once("### 3.8").expect("3.8").1,
            |(body, _)| body,
        );
    let quoted = section
        .split_once("```sql\n")
        .expect("the quoted aggregate")
        .1
        .split_once("\n```")
        .expect("the end of the quotation")
        .0;
    assert_eq!(quoted, METRIC_ROLLUP_RUN_GRANT_AGGREGATE_SQL);
    for (bind, value) in [
        ("$1", METRIC_ROLLUP_RUN_STEP_MAX_WORK_DAY_ROWS),
        ("$2", METRIC_ROLLUP_RUN_STEP_MAX_MONTH_KEYS),
        ("$3", METRIC_ROLLUP_RUN_STEP_MAX_REPLACING_READ),
    ] {
        let quotation = format!("`{bind}` = {value}");
        assert!(section.contains(&quotation), "{quotation}");
    }
    assert_eq!(METRIC_ROLLUP_RUN_STEP_MAX_WORK_DAY_ROWS, 8_000);
    assert_eq!(METRIC_ROLLUP_RUN_STEP_MAX_MONTH_KEYS, 1_000);
    assert_eq!(METRIC_ROLLUP_RUN_STEP_MAX_REPLACING_READ, 600_000);
    assert_eq!(METRIC_ROLLUP_RUN_PAGE_MAX_MONTH_KEYS, 10_000);
    assert_eq!(METRIC_ROLLUP_RUN_PAGE_MAX_WORK_DAY_ROWS, 150_000);
    for (name, value) in [
        ("STEP_MAX_WORK_DAY_ROWS", "8,000"),
        ("STEP_MAX_REPLACING_READ", "600,000"),
        ("PAGE_MAX_WORK_DAY_ROWS", "150,000"),
        ("PAGE_MAX_MONTH_KEYS", "10,000"),
        ("STEP_MAX_MONTH_KEYS", "1,000"),
    ] {
        let quoted = section
            .lines()
            .find(|line| line.split_whitespace().next() == Some(name))
            .unwrap_or_else(|| panic!("register 3.8 quotes {name}"));
        assert_eq!(
            quoted.split_whitespace().nth(1),
            Some(value),
            "register 3.8 quotes {name} as {value}"
        );
    }
    assert_eq!(
        METRIC_ROLLUP_RUN_STEP_PRESIZE_BUDGET,
        Duration::from_secs(5)
    );
    assert_eq!(METRIC_ROLLUP_RUN_STEP_LOCK_BUDGET, Duration::from_secs(2));
    assert_eq!(METRIC_ROLLUP_RUN_STEP_HOLD_BUDGET, Duration::from_secs(5));
    assert_eq!(METRIC_ROLLUP_RUN_PAGE_BUDGET, Duration::from_secs(15));
    assert_eq!(METRIC_ROLLUP_RUN_SIZING_BATCH_WORKS, 256);
}

#[test]
fn sizing_continues_after_a_full_batch_of_256_works_and_stops_on_a_short_one() {
    let (_guard, pool) = setup();
    let corpus = corpus(&pool, 1);
    seed_sparse(&pool, &corpus, 257, 1);
    let unlimited = budgets_of(i64::MAX / 4, i64::MAX / 4, i64::MAX / 4);
    let all = size_range_for_test(&pool, None, unlimited).expect("sizing");
    assert_eq!(
        (all.works.len(), all.units, all.statements, all.hi),
        (257, 257, 2, None)
    );
    // Filled exactly at the 256th work: one statement, ending there.
    let filled = size_range_for_test(&pool, None, budgets_of(i64::MAX / 4, 256, i64::MAX / 4))
        .expect("sizing");
    assert_eq!((filled.works.len(), filled.statements), (256, 1));
    assert_eq!(filled.hi, filled.last_unit);
    // Exactly 256 works left: the full batch is followed by an empty one.
    let after_first = Some(Unit {
        work_id: all.works[0],
        year_start: jan1(2010),
    });
    let rest = size_range_for_test(&pool, after_first, unlimited).expect("sizing");
    assert_eq!((rest.works.len(), rest.statements, rest.hi), (256, 2, None));
}

#[test]
fn a_slow_sizing_statement_is_retryable_within_the_presize_budget_before_any_work_lock() {
    let (_guard, pool) = setup();
    let (_corpus, _works, _frontier) = layered(&pool);
    let budgets = short(1_000);
    let run = start(&pool, Rebuild);
    let (logged, log) = logging_pool();
    hooks::inject(
        Site::Sizing(Stage::Presize),
        hooks::Injection::Statement("SELECT pg_sleep(30)".to_string()),
    );
    let started = Instant::now();
    let refused = try_advance(&logged, &run, &budgets).expect_err("the presize budget");
    hooks::clear();
    assert_eq!(refused, Error::Retryable(reason::STATEMENT_TIMEOUT));
    assert!(started.elapsed() < budgets.step_presize + Duration::from_millis(500));
    assert!(!captured(&log).iter().any(|s| s.starts_with(LOCK_WORKS_SQL)));
    assert_eq!(run_row(&pool, run.run_id).step_count, 0);
}

#[test]
fn a_new_work_between_the_presize_and_the_resize_cuts_the_range_or_empties_it() {
    let (_guard, pool) = setup();
    let (corpus, works, _frontier) = layered(&pool);
    let budgets = keys(30);
    let run = start(&pool, Rebuild);
    let candidate = size_range_for_test(&pool, None, budgets.step_size()).expect("sizing");
    assert_eq!(candidate.works, vec![works[0], works[1]]);

    // Inside the candidate range: the step is cut before the new work.
    let inside = controlled_work(0x30);
    let (paused, pause, _log) = pausing_pool(LOCK_WORKS_SQL);
    let stepping = {
        let run = run.clone();
        thread::spawn(move || try_advance(&paused, &run, &budgets))
    };
    pause.wait_reached();
    add_work(&pool, &corpus, inside);
    complete_one(&pool, &corpus, inside, day(2024, 5, 5), 3);
    pause.release();
    let stepped = stepping.join().expect("join").expect("the step").run;
    assert_eq!(cursor(&pool, run.run_id), Some(unit(works[0], 2025)));
    assert_eq!(stepped.build.clone().expect("build").units, 3);

    // Before the candidate range, which now starts at the first case's work
    // (0x30): the re-size is empty, and retryable.
    let before = controlled_work(0x28);
    let (paused, pause, _log) = pausing_pool(LOCK_WORKS_SQL);
    let stepping = {
        let run = stepped.clone();
        thread::spawn(move || try_advance(&paused, &run, &budgets))
    };
    pause.wait_reached();
    add_work(&pool, &corpus, before);
    complete_one(&pool, &corpus, before, day(2024, 5, 5), 3);
    pause.release();
    assert_eq!(
        stepping.join().expect("join"),
        Err(Error::Retryable(reason::EMPTY_RESIZE))
    );
    assert_eq!(cursor(&pool, run.run_id), Some(unit(works[0], 2025)));
    let run = drive(&pool, stepped, &budgets);
    assert_eq!(run.status, Reconciled);
    assert!(verification(&pool).matches);
}

// ===========================================================================
// Performance evidence (section 17 of the specification)
// ===========================================================================

/// The section 17 evidence, as ignored tests run one at a time, for example
/// `cargo test -p thoth-api --lib rebuild_run_tests::evidence::f1 --
/// --ignored --nocapture`. Durations depend on the host, so every committed
/// step and page is printed raw (`RAW` lines, and `GROUPS` lines for
/// replacing steps) and summarised (`SUMMARY` lines), each section 17
/// acceptance line is printed with what was measured (`ACCEPT` lines), and
/// only correctness outcomes are asserted. Every pass runs under the section
/// 10 budgets; a range they refuse is measured once under the labelled
/// diagnostic budgets (`DIAGNOSTIC` lines), still counts as a refusal of its
/// class, and its pass moves on. Scale knobs are environment variables named
/// in each fixture; every default is the specified scale.
mod evidence {
    use std::sync::atomic::{AtomicBool, AtomicI64, AtomicUsize, Ordering};
    use std::sync::RwLock;

    use diesel::sql_types::{BigInt, Integer};

    use super::*;
    use crate::model::metric_rollup_delta::crud::{
        claim_metric_rollup_deltas, complete_metric_rollup_deltas, MONTH_INSERT_STATEMENTS,
        MONTH_SOURCE_WATERMARK_SQL, YEAR_COUNTRY_INSERT_SQL, YEAR_INSTITUTION_INSERT_SQL,
    };
    use crate::model::metric_rollup_delta::tests::{percentiles, seed_production_shaped_days};
    use crate::model::metric_rollup_delta::verification::verify_metric_rollup_months;
    use crate::model::metric_rollup_delta::METRIC_ROLLUP_CLAIM_MAX_BATCH;

    /// How many bytes of each statement the evidence log keeps: enough to
    /// tell every statement of a step or page apart.
    const HEAD_BYTES: usize = 400;

    /// The institutions the census-shaped fixtures draw from.
    const INSTITUTIONS: i64 = 1_000;

    /// The application name of the run's connection.
    const STEP_APPLICATION: &str = "ops03-step";

    /// How often the F6 sampler reads `pg_blocking_pids()`.
    const SAMPLE_INTERVAL: Duration = Duration::from_millis(100);

    /// The budgets a range refused under the section 10 budgets is measured
    /// with, once (section 17): the time budgets raised, the lock budget and
    /// every size budget as in production, so the range is the same.
    const DIAGNOSTIC: RunBudgets = RunBudgets {
        step_presize: Duration::from_secs(60),
        step_hold: Duration::from_secs(120),
        page: Duration::from_secs(120),
        ..RunBudgets::PRODUCTION
    };

    /// The last year of every fixture's history.
    const LAST_YEAR: i32 = 2025;

    /// The F4 line's bound on hold / fill, and F5's (section 17).
    const F4_LINE: Duration = Duration::from_secs(4);

    /// A scale knob: the environment variable `name`, or its default.
    fn knob(name: &str, default: i64) -> i64 {
        std::env::var(name)
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(default)
    }

    fn ms(duration: Duration) -> String {
        format!("{:.1}", duration.as_secs_f64() * 1_000.0)
    }

    fn ms_of(micros: u128) -> String {
        format!("{:.1}", micros as f64 / 1_000.0)
    }

    fn millis(duration: Duration) -> f64 {
        duration.as_secs_f64() * 1_000.0
    }

    fn secs(duration: Duration) -> String {
        format!("{:.1}", duration.as_secs_f64())
    }

    fn quoted(ids: &[Uuid]) -> String {
        ids.iter()
            .map(|id| format!("'{id}'"))
            .collect::<Vec<_>>()
            .join(", ")
    }

    #[derive(QueryableByName)]
    struct Count {
        #[diesel(sql_type = BigInt)]
        n: i64,
    }

    /// The `n` column of a statement's single row.
    fn count_of(pool: &PgPool, statement: &str) -> i64 {
        let mut connection = pool.get().expect("Failed to get DB connection");
        sql_query(statement)
            .get_result::<Count>(&mut connection)
            .expect("a count")
            .n
    }

    #[derive(QueryableByName)]
    struct Value {
        #[diesel(sql_type = Nullable<Text>)]
        value: Option<String>,
    }

    /// `pg_jit_available()` in a transaction that sets `jit` on: whether
    /// the build can JIT at all, which the cluster's `jit = off` hides.
    fn jit_available_when_on(pool: &PgPool) -> String {
        let mut connection = pool.get().expect("Failed to get DB connection");
        connection
            .transaction::<_, DieselError, _>(|c| {
                sql_query("SET LOCAL jit = on").execute(c)?;
                sql_query("SELECT pg_jit_available()::text AS value")
                    .get_result::<Value>(c)
                    .map(|row| row.value.unwrap_or_default())
            })
            .unwrap_or_else(|error| format!("unreadable: {error}"))
    }

    /// The evidence host's PostgreSQL build, databases, settings and
    /// round-trip latency (200 `SELECT 1` on one connection), the budgets and
    /// the statement-label map, printed at the start of every run (section
    /// 17). The installation prefix is a path of the evidence host, not a
    /// fact of the build, so it is printed as `<prefix>`.
    fn host_facts(pool: &PgPool) {
        let setting = |name: &str| {
            text_scalar(pool, &format!("current_setting('{name}')")).unwrap_or_default()
        };
        let build = |name: &str| {
            text_scalar(
                pool,
                &format!("SELECT setting FROM pg_config WHERE name = '{name}'"),
            )
            .unwrap_or_default()
        };
        // The installed prefix, and the prefix the build was configured with
        // (an installation can be moved after it is built).
        let bindir = build("BINDIR");
        let configure = build("CONFIGURE");
        let prefixes: Vec<String> = [
            bindir.strip_suffix("/bin").unwrap_or(&bindir).to_string(),
            configure
                .split("'--prefix=")
                .nth(1)
                .and_then(|rest| rest.split('\'').next())
                .unwrap_or_default()
                .to_string(),
        ]
        .into_iter()
        .filter(|prefix| prefix.len() > 1)
        .collect();
        let redact = |text: String| {
            prefixes
                .iter()
                .fold(text, |text, prefix| text.replace(prefix, "<prefix>"))
        };
        let mut connection = pool.get().expect("Failed to get DB connection");
        let mut round_trips = Vec::new();
        for _ in 0..200 {
            let started = Instant::now();
            sql_query("SELECT 1")
                .execute(&mut connection)
                .expect("a round trip");
            round_trips.push(started.elapsed());
        }
        drop(connection);
        let (p50, p95, max) = percentiles(&mut round_trips);
        // The test harness has printed the test's name without a newline.
        println!();
        println!(
            "HOST|PostgreSQL {} ({})|round trip SELECT 1 p50 {} ms, p95 {} ms, max {} ms|\
             cpus {}|shared_buffers {}|work_mem {}|effective_cache_size {}|max_wal_size {}|\
             autovacuum {}|random_page_cost {}|jit {}",
            setting("server_version"),
            setting("server_version_num"),
            ms_of(p50),
            ms_of(p95),
            ms_of(max),
            thread::available_parallelism().map_or(0, |n| n.get()),
            setting("shared_buffers"),
            setting("work_mem"),
            setting("effective_cache_size"),
            setting("max_wal_size"),
            setting("autovacuum"),
            setting("random_page_cost"),
            setting("jit"),
        );
        for name in ["VERSION", "CONFIGURE", "CFLAGS", "LDFLAGS"] {
            println!("HOSTBUILD|pg_config {name}|{}", redact(build(name)));
        }
        println!(
            "HOSTDB|{}",
            text_scalar(
                pool,
                "SELECT string_agg(datname || ': encoding ' || pg_encoding_to_char(encoding) \
                                   || ', collate ' || datcollate || ', ctype ' || datctype \
                                   || ', locale provider ' \
                                   || CASE datlocprovider WHEN 'c' THEN 'libc' \
                                                          WHEN 'i' THEN 'icu' \
                                                          WHEN 'b' THEN 'builtin' \
                                                          ELSE datlocprovider::text END, \
                                   '; ' ORDER BY datname) \
                 FROM pg_database"
            )
            .unwrap_or_default()
        );
        println!(
            "HOSTJIT|jit {}|jit_provider {}|pg_jit_available() {} (false whenever jit is off)|\
             pg_jit_available() with jit set on in a transaction {}",
            setting("jit"),
            setting("jit_provider"),
            text_scalar(pool, "SELECT pg_jit_available()").unwrap_or_default(),
            jit_available_when_on(pool),
        );
        println!(
            "HOSTSETTINGS|settings not at their defaults|{}",
            text_scalar(
                pool,
                "SELECT string_agg(name || '=' || setting || COALESCE(' ' || unit, '') \
                                   || ' (' || source || ')', '; ' ORDER BY name) \
                 FROM pg_settings WHERE source NOT IN ('default', 'override', 'client')"
            )
            .unwrap_or_default()
        );
        println!(
            "BUDGETS|every pass runs under the section 10 budgets|a range they refuse is \
             measured once under the DIAGNOSTIC budgets (pre-size {} s, step hold {} s, page {} \
             s; lock and size budgets as in production), counts as a refusal of its class, and \
             its pass moves on|F6 never uses the diagnostic budgets",
            DIAGNOSTIC.step_presize.as_secs(),
            DIAGNOSTIC.step_hold.as_secs(),
            DIAGNOSTIC.page.as_secs(),
        );
        print_label_map();
    }

    // -----------------------------------------------------------------------
    // The run's connection and its statement log
    // -----------------------------------------------------------------------

    /// One statement event: when, whether it finished, and its head.
    type Event = (Instant, bool, String);

    type Events = Arc<Mutex<Vec<Event>>>;

    /// The first `HEAD_BYTES` of a statement's text and binds, without
    /// formatting the rest.
    struct Head(String);

    impl std::fmt::Write for Head {
        fn write_str(&mut self, text: &str) -> std::fmt::Result {
            let room = HEAD_BYTES.saturating_sub(self.0.len());
            if text.len() <= room {
                self.0.push_str(text);
                return Ok(());
            }
            let mut cut = room;
            while !text.is_char_boundary(cut) {
                cut -= 1;
            }
            self.0.push_str(&text[..cut]);
            Err(std::fmt::Error)
        }
    }

    /// Tags each connection with an application name and, with `events`,
    /// captures its statements with their start and finish times.
    #[derive(Debug)]
    struct Tagged {
        application_name: &'static str,
        events: Option<Events>,
    }

    impl CustomizeConnection<PgConnection, diesel::r2d2::Error> for Tagged {
        fn on_acquire(&self, connection: &mut PgConnection) -> Result<(), diesel::r2d2::Error> {
            sql_query(format!(
                "SET application_name = '{}'",
                self.application_name
            ))
            .execute(connection)
            .map_err(diesel::r2d2::Error::QueryError)?;
            if let Some(events) = &self.events {
                let events = Arc::clone(events);
                connection.set_instrumentation(move |event: InstrumentationEvent<'_>| {
                    let at = Instant::now();
                    let (done, query) = match event {
                        InstrumentationEvent::StartQuery { query, .. } => (false, query),
                        InstrumentationEvent::FinishQuery { query, .. } => (true, query),
                        _ => return,
                    };
                    let mut head = Head(String::new());
                    let _ = std::fmt::write(&mut head, format_args!("{query}"));
                    events
                        .lock()
                        .expect("evidence log")
                        .push((at, done, head.0));
                });
            }
            Ok(())
        }
    }

    fn tagged_pool(
        application_name: &'static str,
        size: u32,
        events: Option<Events>,
    ) -> Arc<PgPool> {
        let pool = Pool::builder()
            .max_size(size)
            .connection_customizer(Box::new(Tagged {
                application_name,
                events,
            }))
            .build(ConnectionManager::<PgConnection>::new(test_db_url()))
            .expect("Failed to create an evidence pool");
        Arc::new(pool)
    }

    /// The run's connection: one pooled connection tagged
    /// `STEP_APPLICATION`, whose statements are captured.
    struct Evidence {
        pool: Arc<PgPool>,
        events: Events,
    }

    impl Evidence {
        fn new() -> Self {
            let events: Events = Arc::new(Mutex::new(Vec::new()));
            let pool = tagged_pool(STEP_APPLICATION, 1, Some(Arc::clone(&events)));
            Evidence { pool, events }
        }

        fn take(&self) -> Vec<Event> {
            std::mem::take(&mut *self.events.lock().expect("evidence log"))
        }
    }

    // -----------------------------------------------------------------------
    // Statement labels and the corner line's groups (section 10)
    // -----------------------------------------------------------------------

    /// The five statements of a month chunk, in the order a chunk issues
    /// them. Their texts share a long head, so they are told apart by
    /// position.
    const MONTH_CHUNK_LABELS: [&str; 5] = [
        "month watermark",
        "month total insert",
        "month country insert",
        "month institution insert",
        "month ambiguity insert",
    ];

    /// The head the month chunk statements share, up to `k.month_start`:
    /// the year statements' head differs there (`k.year_start`).
    const MONTH_CHUNK_HEAD: usize = 87;

    /// Where a statement ran: in a step before its hold, in a step's hold
    /// (from the moment the state-row lock is granted to the end of the
    /// transaction), or in a verify page, which has no hold and no group.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Phase {
        Outside,
        Held,
        Page,
    }

    const OUTSIDE: &str = "outside the hold (no group)";

    /// The group of every step label in the corner line (section 10, as
    /// R14.1 maps it). Labels outside the hold belong to no group; inside
    /// it, walk, inserts and verify take the labels named here and the rest
    /// is the remainder of the hold.
    const LABEL_GROUPS: [(&str, &str); 32] = [
        ("checkout check", OUTSIDE),
        ("transaction control (outside the hold)", OUTSIDE),
        ("set local (outside the hold)", OUTSIDE),
        ("run read", OUTSIDE),
        ("isolation assertion", OUTSIDE),
        ("pre-sizing", OUTSIDE),
        ("pre-sizing per-year", OUTSIDE),
        ("lock works", OUTSIDE),
        ("lock state", OUTSIDE),
        ("other (outside the hold)", OUTSIDE),
        ("run lock", "rest"),
        ("set local (in the hold)", "rest"),
        ("month keys", "rest"),
        ("run update", "rest"),
        ("transaction control (in the hold)", "rest"),
        ("other (in the hold)", "rest"),
        ("month watermark", "walk"),
        ("month total insert", "walk"),
        ("month ambiguity insert", "walk"),
        ("month country insert", "inserts"),
        ("month institution insert", "inserts"),
        ("year country insert", "inserts"),
        ("year institution insert", "inserts"),
        ("range verifier", "verify"),
        ("range delete 1", "verify"),
        ("range delete 2", "verify"),
        ("range delete 3", "verify"),
        ("range delete 4", "verify"),
        ("range delete 5", "verify"),
        ("range delete 6", "verify"),
        ("re-sizing", "verify"),
        ("re-sizing per-year", "verify"),
    ];

    /// The label map, printed in every run (section 10: the report lists
    /// the mapping).
    fn print_label_map() {
        for (label, group) in LABEL_GROUPS {
            println!("LABELMAP|{label}|{group}");
        }
        println!("LABELMAP|<any label> (page)|a verify page's statement: no group");
        println!(
            "LABELMAP|groups|the hold starts when lock state finishes and ends when the step's \
             COMMIT finishes|walk = 5/3 x (month watermark + month total insert + month \
             ambiguity insert)|inserts = month country insert + month institution insert - 2/3 x \
             (the walk's three statements) + year country insert + year institution insert|\
             verify = range verifier (pre and post) + range deletes + re-sizing with its \
             per-year sizing|rest = hold - walk - inserts - verify (run lock, set local in the \
             hold, month keys, run update, COMMIT and the time between statements)"
        );
    }

    fn is_statement(text: &str, statement: &str) -> bool {
        text.starts_with(&statement[..statement.len().min(HEAD_BYTES / 2)])
    }

    /// A label for one logged statement; `month_position` counts the month
    /// chunk statements in a row.
    fn statement_label(text: &str, month_position: &mut usize, phase: Phase) -> String {
        let place = match phase {
            Phase::Outside => "outside the hold",
            Phase::Held => "in the hold",
            Phase::Page => "page",
        };
        let label = step_label(text, month_position, phase, place);
        if phase == Phase::Page && !label.contains("(page)") {
            format!("{label} (page)")
        } else {
            label
        }
    }

    fn step_label(text: &str, month_position: &mut usize, phase: Phase, place: &str) -> String {
        if is_set_local(text) {
            return format!("set local ({place})");
        }
        if text.starts_with(&MONTH_SOURCE_WATERMARK_SQL[..MONTH_CHUNK_HEAD]) {
            let label = MONTH_CHUNK_LABELS[*month_position % MONTH_CHUNK_LABELS.len()];
            *month_position += 1;
            return label.to_string();
        }
        *month_position = 0;
        if text.starts_with(&YEAR_COUNTRY_INSERT_SQL[..MONTH_CHUNK_HEAD - 1]) {
            return if text.contains("INSERT INTO public.metric_rollup_work_country_year") {
                "year country insert".to_string()
            } else {
                "year institution insert".to_string()
            };
        }
        if let Some(index) = RANGE_DELETE_STATEMENTS
            .iter()
            .position(|statement| is_statement(text, statement))
        {
            return format!("range delete {}", index + 1);
        }
        let sizing = match phase {
            Phase::Outside => "pre-sizing",
            Phase::Held => "re-sizing",
            Phase::Page => "sizing",
        };
        if is_statement(text, SIZING_SQL) {
            return sizing.to_string();
        }
        if is_statement(text, PER_YEAR_SQL) {
            return format!("{sizing} per-year");
        }
        if text_of(text) == "SELECT 1" {
            return "checkout check".to_string();
        }
        if text.starts_with(RUN_UPDATE_PREFIX) {
            return "run update".to_string();
        }
        // In this order: the run lock's text extends the run read's, and the
        // state lock's extends the page's frontier read.
        let named = [
            ("lock works", LOCK_WORKS_SQL),
            ("lock state", LOCK_STATE_SQL),
            ("frontier read", PAGE_FRONTIER_SQL),
            ("table lock", PAGE_LOCK_SQL),
            ("range verifier", RANGE_VERIFICATION_SQL),
            ("month keys", RANGE_MONTH_KEYS_SQL),
            ("run lock", RUN_LOCK_SQL),
            ("run read", RUN_READ_SQL),
            ("isolation assertion", ISOLATION_SQL),
        ];
        if let Some((label, _)) = named
            .iter()
            .find(|(_, statement)| is_statement(text, statement))
        {
            return (*label).to_string();
        }
        if is_transaction_control(text) {
            return format!("transaction control ({place})");
        }
        let head: String = text.chars().take(40).collect();
        format!("other ({place}): {head}")
    }

    /// The corner line's group of a label (section 10).
    fn group_of(label: &str) -> &'static str {
        if label.ends_with("(page)") {
            return "page (no group)";
        }
        if label.starts_with("other (in the hold)") {
            return "rest";
        }
        if label.starts_with("other (outside the hold)") {
            return OUTSIDE;
        }
        LABEL_GROUPS
            .iter()
            .find(|(name, _)| *name == label)
            .map_or("unmapped", |(_, group)| group)
    }

    /// One finished statement of a call's log.
    struct Logged {
        label: String,
        took: Duration,
        phase: Phase,
    }

    /// The finished statements of a call's log, labelled. A call runs the
    /// dispatch read in a transaction of its own before the step's or the
    /// page's; the last transaction is a page's when it takes the page's
    /// table lock.
    fn logged(events: &[Event]) -> Vec<Logged> {
        let last_begin = events
            .iter()
            .rposition(|(_, done, text)| !*done && text.starts_with("BEGIN"));
        let paged = last_begin.is_some_and(|first| {
            events[first..]
                .iter()
                .any(|(_, _, text)| is_statement(text, PAGE_LOCK_SQL))
        });
        let mut phase = Phase::Outside;
        let mut month_position = 0;
        let mut started: Option<Instant> = None;
        let mut statements = Vec::new();
        for (index, (at, done, text)) in events.iter().enumerate() {
            if !*done {
                if paged && Some(index) == last_begin {
                    phase = Phase::Page;
                }
                started = Some(*at);
                continue;
            }
            let took = started
                .take()
                .map_or(Duration::ZERO, |from| at.saturating_duration_since(from));
            let label = statement_label(text, &mut month_position, phase);
            statements.push(Logged { label, took, phase });
            if phase == Phase::Outside && text_of(text) == LOCK_STATE_SQL {
                phase = Phase::Held;
            } else if phase == Phase::Held
                && (text.starts_with("COMMIT") || text.starts_with("ROLLBACK"))
            {
                phase = Phase::Outside;
            }
        }
        statements
    }

    /// Where a call's time went: per statement label, its count, total and
    /// longest duration, the largest total first.
    fn statement_profile(events: &[Event]) -> String {
        let mut by_label: BTreeMap<String, (usize, Duration, Duration)> = BTreeMap::new();
        for statement in logged(events) {
            let entry =
                by_label
                    .entry(statement.label)
                    .or_insert((0, Duration::ZERO, Duration::ZERO));
            entry.0 += 1;
            entry.1 += statement.took;
            entry.2 = entry.2.max(statement.took);
        }
        let mut rows: Vec<(String, (usize, Duration, Duration))> = by_label.into_iter().collect();
        rows.sort_by_key(|row| std::cmp::Reverse(row.1 .1));
        rows.iter()
            .map(|(label, (count, total, max))| {
                format!(
                    "{label} x{count} total {} ms max {} ms",
                    ms(*total),
                    ms(*max)
                )
            })
            .collect::<Vec<_>>()
            .join("; ")
    }

    /// What the statement log says about one advance call.
    #[derive(Clone, Copy, Debug, Default)]
    struct Timing {
        /// From `BEGIN` to the end of the transaction: a page's time.
        transaction: Duration,
        /// From the state-row lock to the end of the transaction: a step's
        /// hold.
        state_hold: Duration,
        /// From the work-row locks to the end of the transaction.
        work_hold: Duration,
        /// The pre-sizing with its per-year sizing (a page's sizing, for a
        /// page).
        presizing: Duration,
        /// The re-size under the hold, with its per-year sizing.
        resizing: Duration,
        /// In the hold: the three keyed month statements that write no
        /// section rows (watermark, total insert, ambiguity insert).
        light: Duration,
        /// In the hold: the month country and institution inserts.
        sections: Duration,
        /// In the hold: the year country and institution inserts.
        years: Duration,
        /// In the hold: the verify group (range verifier, range deletes and
        /// the re-size).
        verify: Duration,
        /// Every statement the call sent, the pool's checkout check and the
        /// dispatch read's own transaction included.
        statements: usize,
        /// The statements of the call's last transaction (the step's or the
        /// page's), `BEGIN` and `COMMIT` included.
        in_transaction: usize,
        set_locals: usize,
    }

    impl Timing {
        /// The walk group: the three light statements charged at 5/3, so it
        /// covers all five keyed month statements (section 10).
        fn walk_ms(&self) -> f64 {
            millis(self.light) * 5.0 / 3.0
        }

        /// The inserts group: the two month section inserts less the walk
        /// charged to them, and the two year inserts.
        fn inserts_ms(&self) -> f64 {
            millis(self.sections) + millis(self.years) - millis(self.light) * 2.0 / 3.0
        }

        fn verify_ms(&self) -> f64 {
            millis(self.verify)
        }

        /// The rest: the remainder of the hold, so no time is counted twice
        /// or lost.
        fn rest_ms(&self) -> f64 {
            millis(self.state_hold)
                - millis(self.light)
                - millis(self.sections)
                - millis(self.years)
                - millis(self.verify)
        }
    }

    /// The call's timing from its statement log. A call runs the dispatch
    /// read in a transaction of its own before the step's or the page's, so
    /// every transaction measure is taken over the last transaction.
    fn timing(events: &[Event], end: Instant) -> Timing {
        let mut timing = Timing::default();
        let mut begun: Option<Instant> = None;
        let mut ended: Option<Instant> = None;
        let mut state_locked: Option<Instant> = None;
        let mut works_locked: Option<Instant> = None;
        let mut inside = false;
        let mut current = 0;
        for (at, done, text) in events {
            if !*done {
                timing.statements += 1;
                if text.starts_with("BEGIN") {
                    begun = Some(*at);
                    ended = None;
                    inside = true;
                    current = 0;
                }
                if inside {
                    current += 1;
                }
                if is_set_local(text) {
                    timing.set_locals += 1;
                }
                continue;
            }
            if text_of(text) == LOCK_STATE_SQL {
                state_locked = Some(*at);
            } else if is_statement(text, LOCK_WORKS_SQL) {
                works_locked = Some(*at);
            } else if inside && (text.starts_with("COMMIT") || text.starts_with("ROLLBACK")) {
                ended = Some(*at);
                inside = false;
                timing.in_transaction = current;
            }
        }
        if inside {
            timing.in_transaction = current;
        }
        for statement in logged(events) {
            match statement.label.as_str() {
                "pre-sizing"
                | "pre-sizing per-year"
                | "sizing (page)"
                | "sizing per-year (page)" => timing.presizing += statement.took,
                "re-sizing" | "re-sizing per-year" => timing.resizing += statement.took,
                _ => {}
            }
            if statement.phase != Phase::Held {
                continue;
            }
            match group_of(&statement.label) {
                "walk" => timing.light += statement.took,
                "inserts" if statement.label.starts_with("month") => {
                    timing.sections += statement.took;
                }
                "inserts" => timing.years += statement.took,
                "verify" => timing.verify += statement.took,
                _ => {}
            }
        }
        let end = ended.unwrap_or(end);
        let since = |from: Option<Instant>| {
            from.map_or(Duration::ZERO, |from| end.saturating_duration_since(from))
        };
        timing.transaction = since(begun);
        timing.state_hold = since(state_locked);
        timing.work_hold = since(works_locked);
        timing
    }

    // -----------------------------------------------------------------------
    // Direct counts over a range of units
    // -----------------------------------------------------------------------

    /// The SQL predicate of the work-day rows of the units in `(lo, hi]`,
    /// with the bounding `work_id` range spelled out for the planner.
    fn range_sql(lo: Option<Unit>, hi: Option<Unit>) -> String {
        let end = |unit: Unit| format!("DATE '{:04}-01-01'", unit.year_start.year() + 1);
        let mut parts = vec!["TRUE".to_string()];
        if let Some(lo) = lo {
            parts.push(format!("work_id >= '{}'", lo.work_id));
            parts.push(format!(
                "(work_id > '{work}' OR (work_id = '{work}' AND day >= {end}))",
                work = lo.work_id,
                end = end(lo)
            ));
        }
        if let Some(hi) = hi {
            parts.push(format!("work_id <= '{}'", hi.work_id));
            parts.push(format!(
                "(work_id < '{work}' OR (work_id = '{work}' AND day < {end}))",
                work = hi.work_id,
                end = end(hi)
            ));
        }
        parts.join(" AND ")
    }

    /// Direct counts over the units of a range: its rows and month keys,
    /// both terms of the replacing-read figure, the whole-history rows of
    /// its works (summed, and the largest), the most month keys of one
    /// work-month, its units, year keys and work-months, the walk's two
    /// drivers summed over its works (section 10, R14.1), and its works.
    #[derive(Clone, Debug, Default, QueryableByName)]
    struct Terms {
        #[diesel(sql_type = BigInt)]
        rows: i64,
        #[diesel(sql_type = BigInt)]
        month_keys: i64,
        #[diesel(sql_type = BigInt)]
        month_term: i64,
        #[diesel(sql_type = BigInt)]
        year_term: i64,
        #[diesel(sql_type = BigInt)]
        whole_rows: i64,
        #[diesel(sql_type = BigInt)]
        max_whole: i64,
        #[diesel(sql_type = BigInt)]
        max_month_keys: i64,
        #[diesel(sql_type = BigInt)]
        units: i64,
        #[diesel(sql_type = BigInt)]
        year_keys: i64,
        #[diesel(sql_type = BigInt)]
        work_months: i64,
        /// The sum over the range's works of the work's month keys in the
        /// range times its whole-history rows.
        #[diesel(sql_type = BigInt)]
        walk_keys: i64,
        /// The sum over the range's works of the work's work-months in the
        /// range times its whole-history rows.
        #[diesel(sql_type = BigInt)]
        walk_months: i64,
        #[diesel(sql_type = Array<SqlUuid>)]
        work_ids: Vec<Uuid>,
    }

    impl Terms {
        fn figure(&self) -> i64 {
            self.month_term + self.year_term
        }

        fn row_fill(&self) -> f64 {
            self.rows as f64 / METRIC_ROLLUP_RUN_STEP_MAX_WORK_DAY_ROWS as f64
        }

        fn figure_fill(&self) -> f64 {
            self.figure() as f64 / METRIC_ROLLUP_RUN_STEP_MAX_REPLACING_READ as f64
        }

        /// F4's fill: the smaller of the row fill and the figure fill
        /// (section 10, R12).
        fn fill(&self) -> f64 {
            self.row_fill().min(self.figure_fill())
        }
    }

    fn terms(pool: &PgPool, lo: Option<Unit>, hi: Option<Unit>) -> Terms {
        let range = range_sql(lo, hi);
        let mut connection = pool.get().expect("Failed to get DB connection");
        sql_query(format!(
            "WITH d AS ( \
                 SELECT work_id, day, platform_id, measure_id FROM metric_rollup_work_day \
                 WHERE {range}), \
             wm AS ( \
                 SELECT work_id, date_trunc('month', day) AS m, count(*) AS rows, \
                        count(DISTINCT (platform_id, measure_id)) AS keys \
                 FROM d GROUP BY 1, 2), \
             uy AS ( \
                 SELECT work_id, date_trunc('year', day) AS y, \
                        count(DISTINCT (platform_id, measure_id)) AS series \
                 FROM d GROUP BY 1, 2), \
             whole AS ( \
                 SELECT r.work_id, count(*) AS rows FROM metric_rollup_work_day r \
                 WHERE r.work_id IN (SELECT work_id FROM uy) GROUP BY r.work_id), \
             pw AS ( \
                 SELECT work_id, sum(keys) AS keys, count(*) AS months FROM wm \
                 GROUP BY work_id) \
             SELECT COALESCE((SELECT sum(rows) FROM wm), 0)::bigint AS rows, \
                    COALESCE((SELECT sum(keys) FROM wm), 0)::bigint AS month_keys, \
                    COALESCE((SELECT sum(keys * rows) FROM wm), 0)::bigint AS month_term, \
                    COALESCE((SELECT sum(uy.series * whole.rows) \
                              FROM uy JOIN whole USING (work_id)), 0)::bigint AS year_term, \
                    COALESCE((SELECT sum(rows) FROM whole), 0)::bigint AS whole_rows, \
                    COALESCE((SELECT max(rows) FROM whole), 0)::bigint AS max_whole, \
                    COALESCE((SELECT max(keys) FROM wm), 0)::bigint AS max_month_keys, \
                    (SELECT count(*) FROM uy)::bigint AS units, \
                    COALESCE((SELECT sum(series) FROM uy), 0)::bigint AS year_keys, \
                    (SELECT count(*) FROM wm)::bigint AS work_months, \
                    COALESCE((SELECT sum(pw.keys * whole.rows) \
                              FROM pw JOIN whole USING (work_id)), 0)::bigint AS walk_keys, \
                    COALESCE((SELECT sum(pw.months * whole.rows) \
                              FROM pw JOIN whole USING (work_id)), 0)::bigint AS walk_months, \
                    COALESCE((SELECT array_agg(work_id ORDER BY work_id) FROM whole), \
                             '{{}}'::uuid[]) AS work_ids"
        ))
        .get_result(&mut connection)
        .expect("the range's terms")
    }

    fn works_in(pool: &PgPool, lo: Option<Unit>, hi: Option<Unit>) -> Vec<Uuid> {
        #[derive(QueryableByName)]
        struct Work {
            #[diesel(sql_type = SqlUuid)]
            work_id: Uuid,
        }
        let mut connection = pool.get().expect("Failed to get DB connection");
        sql_query(format!(
            "SELECT DISTINCT work_id FROM metric_rollup_work_day WHERE {}",
            range_sql(lo, hi)
        ))
        .load::<Work>(&mut connection)
        .expect("the range's works")
        .into_iter()
        .map(|work| work.work_id)
        .collect()
    }

    fn month_key_count(pool: &PgPool) -> i64 {
        scalar_i64(
            pool,
            "(SELECT COUNT(*) FROM (SELECT DISTINCT work_id, platform_id, measure_id, \
                                       date_trunc('month', day) \
                                   FROM metric_rollup_work_day) k)",
        )
    }

    // -----------------------------------------------------------------------
    // Driving a run and measuring each call
    // -----------------------------------------------------------------------

    /// One committed step or page.
    #[derive(Clone, Debug)]
    struct Sample {
        pass: String,
        page: bool,
        /// The run's step count after the call.
        index: i64,
        /// Which embedded fixture's works the range covers (`base` for none).
        class: &'static str,
        replaced: bool,
        /// Committed under the diagnostic budgets after a refusal under the
        /// section 10 budgets: labelled, and counted as a refusal.
        diagnostic: bool,
        call: Duration,
        timing: Timing,
        units: i64,
        month_keys: i64,
        rows: i64,
        terms: Terms,
        lo: Option<Unit>,
        hi: Option<Unit>,
        /// Where the call's time went, by statement.
        profile: String,
    }

    impl Sample {
        /// A step's state-row hold, or a page's time.
        fn hold(&self) -> Duration {
            if self.page {
                self.timing.transaction
            } else {
                self.timing.state_hold
            }
        }

        fn kind(&self) -> &'static str {
            if self.page {
                "page"
            } else if self.replaced {
                "replacing step"
            } else {
                "non-replacing step"
            }
        }
    }

    /// The embedded fixtures' works by class, in priority order: a range
    /// holding works of two classes takes the first.
    type Classes = Vec<(&'static str, BTreeSet<Uuid>)>;

    /// How one pass drives its run.
    struct Drive {
        pass: String,
        /// Measure a range refused under the section 10 budgets once under
        /// the diagnostic budgets, then move on (every pass but F6's).
        diagnose: bool,
        /// Pause after every call while `BUILDING` by the section 15 rule.
        paced: bool,
        /// Stop when the build phase ends.
        build_only: bool,
        /// Stop once the cursor has reached this unit (mix (c)'s and F5's
        /// passes, whose runs are then abandoned).
        stop_at: Option<Unit>,
        /// Count each step's terms directly.
        terms: bool,
        classes: Classes,
        /// Consecutive retryable refusals before the pass stops.
        max_streak: usize,
    }

    impl Drive {
        fn new(pass: &str, classes: &Classes) -> Self {
            Drive {
                pass: pass.to_string(),
                diagnose: true,
                paced: false,
                build_only: false,
                stop_at: None,
                terms: true,
                classes: classes.clone(),
                max_streak: 20,
            }
        }
    }

    /// One refusal, attributed to the class of its range: the class of the
    /// diagnostic measurement of that range when it committed, otherwise the
    /// class of the first unit after the cursor.
    #[derive(Clone, Debug)]
    struct Refusal {
        class: &'static str,
        /// `replacing step`, `non-replacing step`, `page`, or `step` for a
        /// step range that never committed.
        kind: &'static str,
        reason: String,
        /// Refused for a spent time budget: the budget check before a
        /// statement, or a statement cut by its deadline.
        budget: bool,
        /// Refused under the diagnostic budgets.
        diagnostic: bool,
    }

    /// One pass's outcome.
    struct Driven {
        pass: String,
        run: Run,
        samples: Vec<Sample>,
        /// Every refusal by code and reason.
        refusals: BTreeMap<String, usize>,
        /// Every refusal, attributed to its class.
        refused: Vec<Refusal>,
        longest_streak: usize,
        /// The build phase's time: its calls, and any pauses.
        build: Duration,
        /// The verify phase's time: its calls, and any back-offs.
        verify: Duration,
        /// Why the pass stopped before the run ended, if it did.
        stuck: Option<String>,
    }

    impl Driven {
        fn steps(&self, replaced: bool) -> Vec<&Sample> {
            self.samples
                .iter()
                .filter(|sample| !sample.page && sample.replaced == replaced)
                .collect()
        }

        fn pages(&self) -> Vec<&Sample> {
            self.samples.iter().filter(|sample| sample.page).collect()
        }

        /// Budget refusals of `classes` and `kinds` (each: every one when
        /// empty). A step range that never committed counts for both step
        /// kinds.
        fn budget_refusals(&self, classes: &[&str], kinds: &[&str]) -> usize {
            self.refused
                .iter()
                .filter(|refusal| refusal.budget)
                .filter(|refusal| classes.is_empty() || classes.contains(&refusal.class))
                .filter(|refusal| {
                    kinds.is_empty()
                        || kinds.contains(&refusal.kind)
                        || (refusal.kind == "step"
                            && kinds.iter().any(|kind| kind.ends_with("step")))
                })
                .count()
        }

        fn by_class(&self) -> BTreeMap<String, usize> {
            let mut counts = BTreeMap::new();
            for refusal in &self.refused {
                *counts
                    .entry(format!(
                        "{} {} {}{}",
                        refusal.class,
                        refusal.kind,
                        refusal.reason,
                        if refusal.diagnostic {
                            " (under the diagnostic budgets)"
                        } else {
                            ""
                        }
                    ))
                    .or_insert(0) += 1;
            }
            counts
        }
    }

    /// The section 15 pause after an advance call made while `BUILDING`.
    fn pacing_pause(call: Duration) -> Duration {
        Duration::from_secs(1).max(call * 2)
    }

    fn class_of(classes: &Classes, works: &[Uuid]) -> &'static str {
        classes
            .iter()
            .find(|(_, set)| works.iter().any(|work_id| set.contains(work_id)))
            .map_or("base", |(name, _)| name)
    }

    /// One committed call's sample, from the run row before and after it.
    struct Call<'a> {
        before: &'a RunRow,
        after: &'a RunRow,
        building: bool,
        call: Duration,
        events: &'a [Event],
        end: Instant,
        diagnostic: bool,
    }

    fn sample(pool: &PgPool, drive: &Drive, call: &Call<'_>) -> Sample {
        let (before, after) = (call.before, call.after);
        let lo = before.cursor().expect("the cursor before");
        let hi = if after.status == before.status {
            after.cursor().expect("the cursor after")
        } else {
            None
        };
        let (units, month_keys, rows, replaced) = if call.building {
            (
                after.build_units - before.build_units,
                after.build_month_keys - before.build_month_keys,
                after.build_work_day_rows - before.build_work_day_rows,
                after.build_replaced_steps > before.build_replaced_steps,
            )
        } else {
            (
                after.verify_units - before.verify_units,
                after.verify_month_keys - before.verify_month_keys,
                after.verify_work_day_rows - before.verify_work_day_rows,
                false,
            )
        };
        let terms = if call.building && drive.terms {
            terms(pool, lo, hi)
        } else {
            Terms::default()
        };
        let class = if drive.classes.is_empty() {
            "base"
        } else if call.building && drive.terms {
            class_of(&drive.classes, &terms.work_ids)
        } else {
            class_of(&drive.classes, &works_in(pool, lo, hi))
        };
        Sample {
            pass: drive.pass.clone(),
            page: !call.building,
            index: after.step_count,
            class,
            replaced,
            diagnostic: call.diagnostic,
            call: call.call,
            timing: timing(call.events, call.end),
            units,
            month_keys,
            rows,
            terms,
            lo,
            hi,
            profile: statement_profile(call.events),
        }
    }

    /// The class of the first unit after `cursor`: where a refused range
    /// starts.
    fn class_after(pool: &PgPool, classes: &Classes, cursor: Option<Unit>) -> &'static str {
        if classes.is_empty() {
            return "base";
        }
        let range = range_sql(cursor, None);
        let next = text_scalar(
            pool,
            &format!(
                "SELECT work_id::text FROM metric_rollup_work_day WHERE {range} \
                 ORDER BY work_id, day LIMIT 1"
            ),
        );
        match next.and_then(|work_id| Uuid::parse_str(&work_id).ok()) {
            Some(work_id) => class_of(classes, &[work_id]),
            None => "none",
        }
    }

    fn is_budget_refusal(error: Error) -> bool {
        matches!(error.reason(), reason::BUDGET | reason::STATEMENT_TIMEOUT)
    }

    /// One advance call through the evidence connection.
    struct Attempt {
        result: Result<MetricRollupMonthRebuildAdvance, Error>,
        events: Vec<Event>,
        call: Duration,
        end: Instant,
    }

    fn attempt(evidence: &Evidence, run: &Run, budgets: &RunBudgets) -> Attempt {
        evidence.take();
        let started = Instant::now();
        let result = try_advance(&evidence.pool, run, budgets);
        let end = Instant::now();
        Attempt {
            result,
            events: evidence.take(),
            call: end - started,
            end,
        }
    }

    /// A refused call's `REFUSED` line.
    struct Refused<'a> {
        building: bool,
        step_count: i64,
        refusal: &'a str,
        class: &'a str,
        streak: usize,
        budgets: &'a str,
        events: &'a [Event],
        call: Duration,
        end: Instant,
    }

    fn print_refusal(drive: &Drive, refused: &Refused<'_>) {
        let timing = timing(refused.events, refused.end);
        println!(
            "REFUSED|{}|{}|step count {}|{}|under {}|streak {}|range starts in {}|call_ms={}|\
             hold_ms={}|work_hold_ms={}|statements={}|profile {}",
            drive.pass,
            if refused.building { "step" } else { "page" },
            refused.step_count,
            refused.refusal,
            refused.budgets,
            refused.streak,
            refused.class,
            ms(refused.call),
            ms(if refused.building {
                timing.state_hold
            } else {
                timing.transaction
            }),
            ms(timing.work_hold),
            timing.statements,
            statement_profile(refused.events),
        );
    }

    /// Drive `run` through the evidence connection, one call at a time,
    /// measuring every committed step and page, under the section 10
    /// budgets. A range refused for a spent time budget is measured once
    /// under the diagnostic budgets when the pass diagnoses (section 17): it
    /// counts as a refusal of its class, and the pass moves on. Any other
    /// retryable refusal is retried with the same step count after the
    /// section 15 back-off; a non-retryable refusal, or `max_streak`
    /// refusals in a row, stops the pass. Every refusal is printed
    /// (`REFUSED` lines) with where its time went.
    fn drive_measured(pool: &PgPool, evidence: &Evidence, mut run: Run, drive: &Drive) -> Driven {
        let mut driven = Driven {
            pass: drive.pass.clone(),
            run: run.clone(),
            samples: Vec::new(),
            refusals: BTreeMap::new(),
            refused: Vec::new(),
            longest_streak: 0,
            build: Duration::ZERO,
            verify: Duration::ZERO,
            stuck: None,
        };
        let mut streak = 0;
        while run.status.is_open() && !(drive.build_only && run.status != Building) {
            let building = run.status == Building;
            let before = run_row(pool, run.run_id);
            if let Some(stop) = drive.stop_at {
                let reached = before
                    .cursor()
                    .expect("a well-formed cursor")
                    .is_some_and(|cursor| cursor >= stop);
                if building && reached {
                    break;
                }
            }
            let Attempt {
                result,
                events,
                call,
                end,
            } = attempt(evidence, &run, &RunBudgets::PRODUCTION);
            let mut pause = if drive.paced && building {
                pacing_pause(call)
            } else {
                Duration::ZERO
            };
            let mut spent = call;
            let mut stop = false;
            match result {
                Ok(advance) => {
                    if advance.advanced {
                        streak = 0;
                        let after = run_row(pool, run.run_id);
                        let measured = Call {
                            before: &before,
                            after: &after,
                            building,
                            call,
                            events: &events,
                            end,
                            diagnostic: false,
                        };
                        driven.samples.push(sample(pool, drive, &measured));
                    }
                    run = advance.run;
                }
                Err(error) => {
                    let refusal = format!("{} {}", error.code(), error.reason());
                    *driven.refusals.entry(refusal.clone()).or_insert(0) += 1;
                    streak += 1;
                    driven.longest_streak = driven.longest_streak.max(streak);
                    let cursor = before.cursor().ok().flatten();
                    let mut attributed = Refusal {
                        class: class_after(pool, &drive.classes, cursor),
                        kind: if building { "step" } else { "page" },
                        reason: refusal.clone(),
                        budget: is_budget_refusal(error),
                        diagnostic: false,
                    };
                    print_refusal(
                        drive,
                        &Refused {
                            building,
                            step_count: run.step_count,
                            refusal: &refusal,
                            class: attributed.class,
                            streak,
                            budgets: "the section 10 budgets",
                            events: &events,
                            call,
                            end,
                        },
                    );
                    let mut retry = Some(error);
                    if attributed.budget && drive.diagnose {
                        let second = attempt(evidence, &run, &DIAGNOSTIC);
                        spent += second.call;
                        match second.result {
                            Ok(advance) => {
                                if advance.advanced {
                                    let after = run_row(pool, run.run_id);
                                    let measured = sample(
                                        pool,
                                        drive,
                                        &Call {
                                            before: &before,
                                            after: &after,
                                            building,
                                            call: second.call,
                                            events: &second.events,
                                            end: second.end,
                                            diagnostic: true,
                                        },
                                    );
                                    attributed.class = measured.class;
                                    attributed.kind = measured.kind();
                                    println!(
                                        "DIAGNOSTIC|{}|{}|{}|{}|hold_ms={}|call_ms={}|measured \
                                         once under the diagnostic budgets after a refusal \
                                         under the section 10 budgets; it counts as a refusal \
                                         of its class",
                                        drive.pass,
                                        measured.index,
                                        measured.class,
                                        measured.kind(),
                                        ms(measured.hold()),
                                        ms(measured.call),
                                    );
                                    driven.samples.push(measured);
                                    streak = 0;
                                }
                                run = advance.run;
                                retry = None;
                            }
                            Err(second_error) => {
                                let second_refusal =
                                    format!("{} {}", second_error.code(), second_error.reason());
                                *driven
                                    .refusals
                                    .entry(format!("{second_refusal} (diagnostic budgets)"))
                                    .or_insert(0) += 1;
                                print_refusal(
                                    drive,
                                    &Refused {
                                        building,
                                        step_count: run.step_count,
                                        refusal: &second_refusal,
                                        class: attributed.class,
                                        streak,
                                        budgets: "the DIAGNOSTIC budgets",
                                        events: &second.events,
                                        call: second.call,
                                        end: second.end,
                                    },
                                );
                                driven.refused.push(Refusal {
                                    reason: second_refusal,
                                    budget: is_budget_refusal(second_error),
                                    diagnostic: true,
                                    ..attributed.clone()
                                });
                                retry = Some(second_error);
                            }
                        }
                    }
                    driven.refused.push(attributed);
                    if let Some(error) = retry {
                        if !matches!(error, Error::Retryable(_)) || streak >= drive.max_streak {
                            driven.stuck = Some(format!(
                                "{} {} after {streak} consecutive refusals at step count {}",
                                error.code(),
                                error.reason(),
                                run.step_count
                            ));
                            stop = true;
                        } else {
                            pause = pause.max(if building {
                                pacing_pause(call)
                            } else {
                                Duration::from_secs(1)
                            });
                        }
                    }
                }
            }
            if stop {
                pause = Duration::ZERO;
            }
            if building {
                driven.build += spent + pause;
            } else {
                driven.verify += spent + pause;
            }
            if stop {
                break;
            }
            if !pause.is_zero() {
                thread::sleep(pause);
            }
        }
        driven.run = run;
        driven
    }

    // -----------------------------------------------------------------------
    // Reports
    // -----------------------------------------------------------------------

    fn unit_text(unit: Option<Unit>) -> String {
        unit.map_or("-".to_string(), |unit| {
            format!("{}/{}", unit.work_id, unit.year_start.year())
        })
    }

    /// One replacing step's hold split into the corner line's groups, with
    /// the groups' drivers (section 10), in milliseconds and rows.
    #[derive(Clone, Debug)]
    struct Groups {
        source: String,
        hold: f64,
        walk: f64,
        inserts: f64,
        verify: f64,
        rest: f64,
        statements: f64,
        rows: f64,
        month_term: f64,
        year_term: f64,
        whole_rows: f64,
        walk_keys: f64,
        walk_months: f64,
    }

    impl Groups {
        fn of(sample: &Sample) -> Self {
            let timing = &sample.timing;
            let terms = &sample.terms;
            Groups {
                source: format!("{}|{}|{}", sample.pass, sample.index, sample.class),
                hold: millis(sample.hold()),
                walk: timing.walk_ms(),
                inserts: timing.inserts_ms(),
                verify: timing.verify_ms(),
                rest: timing.rest_ms(),
                statements: timing.in_transaction as f64,
                rows: terms.rows as f64,
                month_term: terms.month_term as f64,
                year_term: terms.year_term as f64,
                whole_rows: terms.whole_rows as f64,
                walk_keys: terms.walk_keys as f64,
                walk_months: terms.walk_months as f64,
            }
        }

        /// A `GROUPS` line as `print_raw` prints it.
        fn parse(line: &str) -> Option<Self> {
            let fields: Vec<&str> = line.trim_end().split('|').collect();
            if fields.first() != Some(&"GROUPS") || fields.len() < 5 {
                return None;
            }
            let values: BTreeMap<&str, f64> = fields[4..]
                .iter()
                .filter_map(|field| field.split_once('='))
                .filter_map(|(key, value)| value.parse().ok().map(|value| (key, value)))
                .collect();
            let get = |key: &str| values.get(key).copied();
            Some(Groups {
                source: fields[1..4].join("|"),
                hold: get("hold")?,
                walk: get("walk")?,
                inserts: get("inserts")?,
                verify: get("verify")?,
                rest: get("rest")?,
                statements: get("statements")?,
                rows: get("rows")?,
                month_term: get("month_term")?,
                year_term: get("year_term")?,
                whole_rows: get("whole_rows")?,
                walk_keys: get("walk_keys")?,
                walk_months: get("walk_months")?,
            })
        }
    }

    fn print_raw(driven: &Driven) {
        for sample in &driven.samples {
            let timing = &sample.timing;
            let terms = &sample.terms;
            println!(
                "RAW|{}|{}|{}|{}|{}|diagnostic={}|call_ms={}|hold_ms={}|work_hold_ms={}|\
                 presizing_ms={}|resizing_ms={}|statements={}|in_transaction={}|set_local={}|\
                 units={}|month_keys={}|rows={}|month_term={}|year_term={}|figure={}|\
                 row_fill={:.3}|figure_fill={:.3}|fill={:.3}|whole_rows={}|max_whole={}|\
                 max_keys_per_work_month={}|work_months={}|year_keys={}|walk_keys={}|\
                 walk_months={}|works={}|lo={}|hi={}",
                sample.pass,
                if sample.page { "page" } else { "step" },
                sample.index,
                sample.class,
                if sample.page {
                    "-"
                } else if sample.replaced {
                    "replacing"
                } else {
                    "non-replacing"
                },
                sample.diagnostic,
                ms(sample.call),
                ms(sample.hold()),
                ms(timing.work_hold),
                ms(timing.presizing),
                ms(timing.resizing),
                timing.statements,
                timing.in_transaction,
                timing.set_locals,
                sample.units,
                sample.month_keys,
                sample.rows,
                terms.month_term,
                terms.year_term,
                terms.figure(),
                terms.row_fill(),
                terms.figure_fill(),
                terms.fill(),
                terms.whole_rows,
                terms.max_whole,
                terms.max_month_keys,
                terms.work_months,
                terms.year_keys,
                terms.walk_keys,
                terms.walk_months,
                terms.work_ids.len(),
                unit_text(sample.lo),
                unit_text(sample.hi),
            );
            if sample.replaced {
                println!(
                    "GROUPS|{}|{}|{}|diagnostic={}|hold={:.3}|light={:.3}|sections={:.3}|\
                     years={:.3}|verify={:.3}|walk={:.3}|inserts={:.3}|rest={:.3}|\
                     presizing={:.3}|resizing={:.3}|statements={}|rows={}|month_keys={}|\
                     work_months={}|month_term={}|year_term={}|whole_rows={}|walk_keys={}|\
                     walk_months={}|works={}",
                    sample.pass,
                    sample.index,
                    sample.class,
                    sample.diagnostic,
                    millis(sample.hold()),
                    millis(timing.light),
                    millis(timing.sections),
                    millis(timing.years),
                    millis(timing.verify),
                    timing.walk_ms(),
                    timing.inserts_ms(),
                    timing.rest_ms(),
                    millis(timing.presizing),
                    millis(timing.resizing),
                    timing.in_transaction,
                    terms.rows,
                    terms.month_keys,
                    terms.work_months,
                    terms.month_term,
                    terms.year_term,
                    terms.whole_rows,
                    terms.walk_keys,
                    terms.walk_months,
                    terms.work_ids.len(),
                );
                println!(
                    "PROFILE|{}|step|{}|{}|{}",
                    sample.pass, sample.index, sample.class, sample.profile
                );
            }
        }
    }

    /// p50, p95 and max of `pick` over `samples`, in microseconds.
    fn spread(
        samples: &[&Sample],
        pick: impl Fn(&Sample) -> Duration,
    ) -> Option<(u128, u128, u128)> {
        if samples.is_empty() {
            return None;
        }
        let mut values: Vec<Duration> = samples.iter().map(|sample| pick(sample)).collect();
        Some(percentiles(&mut values))
    }

    fn spread_text(spread: Option<(u128, u128, u128)>) -> String {
        spread.map_or("none".to_string(), |(p50, p95, max)| {
            format!(
                "p50 {} ms, p95 {} ms, max {} ms",
                ms_of(p50),
                ms_of(p95),
                ms_of(max)
            )
        })
    }

    fn count_spread(values: impl Iterator<Item = usize>) -> String {
        let mut values: Vec<usize> = values.collect();
        if values.is_empty() {
            return "none".to_string();
        }
        values.sort_unstable();
        format!(
            "min {} median {} max {}",
            values[0],
            values[values.len() / 2],
            values[values.len() - 1]
        )
    }

    fn diagnostic_count(samples: &[&Sample]) -> usize {
        samples.iter().filter(|sample| sample.diagnostic).count()
    }

    fn summarise(driven: &Driven, keys: i64) {
        let classes: BTreeSet<&str> = driven.samples.iter().map(|sample| sample.class).collect();
        for class in classes {
            for (kind, replaced) in [("replacing", true), ("non-replacing", false)] {
                let group: Vec<&Sample> = driven
                    .steps(replaced)
                    .into_iter()
                    .filter(|sample| sample.class == class)
                    .collect();
                if group.is_empty() {
                    continue;
                }
                println!(
                    "SUMMARY|{}|{class}|{kind} steps|n={}|diagnostic {}|hold {}|work-row hold {}|\
                     call {}|pre-sizing {}|re-sizing {}|statements per step {}|in the \
                     transaction {}",
                    driven.pass,
                    group.len(),
                    diagnostic_count(&group),
                    spread_text(spread(&group, Sample::hold)),
                    spread_text(spread(&group, |sample| sample.timing.work_hold)),
                    spread_text(spread(&group, |sample| sample.call)),
                    spread_text(spread(&group, |sample| sample.timing.presizing)),
                    spread_text(spread(&group, |sample| sample.timing.resizing)),
                    count_spread(group.iter().map(|sample| sample.timing.statements)),
                    count_spread(group.iter().map(|sample| sample.timing.in_transaction)),
                );
            }
            let pages: Vec<&Sample> = driven
                .pages()
                .into_iter()
                .filter(|sample| sample.class == class)
                .collect();
            if !pages.is_empty() {
                println!(
                    "SUMMARY|{}|{class}|pages|n={}|diagnostic {}|page time {}|call {}|sizing {}|\
                     statements per page {}",
                    driven.pass,
                    pages.len(),
                    diagnostic_count(&pages),
                    spread_text(spread(&pages, Sample::hold)),
                    spread_text(spread(&pages, |sample| sample.call)),
                    spread_text(spread(&pages, |sample| sample.timing.presizing)),
                    count_spread(pages.iter().map(|sample| sample.timing.statements)),
                );
            }
        }
        let total = driven.build + driven.verify;
        let rate = keys as f64 / total.as_secs_f64().max(0.001);
        println!(
            "SUMMARY|{}|run|status {:?}|steps {}|pages {}|build {} s|verify {} s|total {} s|\
             month keys {keys}|keys per second {rate:.0}|projected at 460,000 keys {:.0} s|\
             projected at 1,000,000 keys {:.0} s|refusals {:?}|budget refusals {}|refusals by \
             class {:?}|diagnostic measurements {}|longest run of refusals {}|stuck {:?}",
            driven.pass,
            driven.run.status,
            driven.samples.iter().filter(|sample| !sample.page).count(),
            driven.pages().len(),
            secs(driven.build),
            secs(driven.verify),
            secs(total),
            460_000.0 / rate,
            1_000_000.0 / rate,
            driven.refusals,
            driven.budget_refusals(&[], &[]),
            driven.by_class(),
            diagnostic_count(&driven.samples.iter().collect::<Vec<_>>()),
            driven.longest_streak,
            driven.stuck,
        );
    }

    /// Print a pass and require it to have ended where `expected`.
    fn finish_pass(driven: &Driven, keys: i64, expected: MetricRollupMonthRebuildStatus) {
        print_raw(driven);
        summarise(driven, keys);
        assert!(
            driven.stuck.is_none(),
            "{}: {:?}",
            driven.pass,
            driven.stuck
        );
        assert_eq!(driven.run.status, expected, "{}", driven.pass);
    }

    /// `verifyMetricRollupMonths` after a RECONCILED pass on F2. At F2's size
    /// the whole-table verification can exceed its 30 s statement timeout
    /// (section 3.7's bound, past which the supported path is section 3.8),
    /// so a timeout is reported rather than failed; an answer must be exact.
    fn whole_table(pool: &PgPool, label: &str) {
        let started = Instant::now();
        match verify_metric_rollup_months(pool) {
            Ok(whole) => {
                println!(
                    "WHOLE|{label}|verifyMetricRollupMonths answered in {} s|matches {}",
                    secs(started.elapsed()),
                    whole.matches
                );
                assert!(whole.matches, "{label}");
            }
            Err(error) if format!("{error:?}").contains("statement timeout") => println!(
                "WHOLE|{label}|verifyMetricRollupMonths exceeded its 30 s statement timeout \
                 after {} s",
                secs(started.elapsed())
            ),
            Err(error) => panic!("{label}: verification: {error:?}"),
        }
    }

    /// One section 17 acceptance line on the p95 of `samples`' holds, with
    /// the budget refusals attributed to it; returns whether it was met.
    fn accept(line: &str, samples: &[&Sample], bound: Duration, budget_refusals: usize) -> bool {
        let measured = spread(samples, Sample::hold);
        let met =
            measured.is_some_and(|(_, p95, _)| p95 <= bound.as_micros() && budget_refusals == 0);
        println!(
            "ACCEPT|{line}|n={}|{}|budget refusals {budget_refusals}|diagnostic measurements \
             among them {}|bound p95 <= {} ms|{}",
            samples.len(),
            spread_text(measured),
            diagnostic_count(samples),
            bound.as_millis(),
            match measured {
                None => "NOT MEASURED",
                Some(_) if met => "MET",
                Some(_) => "MISSED",
            }
        );
        met
    }

    fn of_class<'a>(samples: Vec<&'a Sample>, classes: &[&str]) -> Vec<&'a Sample> {
        samples
            .into_iter()
            .filter(|sample| classes.contains(&sample.class))
            .collect()
    }

    /// The walk's measured rate over `steps`: the walk group's time per
    /// (month key x whole-work row) summed over the steps' works, over the
    /// five keyed month statements and per statement.
    fn walk_rate(steps: &[&Sample]) -> String {
        let walk: f64 = steps.iter().map(|sample| sample.timing.walk_ms()).sum();
        let entries: f64 = steps
            .iter()
            .map(|sample| sample.terms.walk_keys as f64)
            .sum();
        if entries <= 0.0 {
            return "walk rate not measured".to_string();
        }
        let rate = walk * 1e6 / entries;
        format!(
            "walk {rate:.1} ns per (month key x whole-work row) over the five keyed month \
             statements, {:.1} ns per statement",
            rate / 5.0
        )
    }

    // -----------------------------------------------------------------------
    // Census-shaped fixtures
    // -----------------------------------------------------------------------

    /// The registry size for mix (b)'s `k` series per work-month: room for
    /// k and more, so mix (b)'s series are shared with many F2 works.
    fn registry_for(k: i64) -> usize {
        usize::try_from((k + 8).max(48)).expect("a registry size")
    }

    /// A corpus with `registry` platforms (one measure, so `registry`
    /// series) and `INSTITUTIONS` institutions.
    fn census_corpus(pool: &PgPool, registry: usize) -> Corpus {
        let corpus = corpus(pool, registry);
        exec(
            pool,
            &format!(
                "INSERT INTO institution (institution_id, institution_name) \
                 SELECT md5('ops03-institution-' || g)::uuid, 'OPS-03 evidence institution ' || g \
                 FROM generate_series(1, {INSTITUTIONS}) g"
            ),
        );
        corpus
    }

    /// A work id below every hashed fixture work's: `000000rr-...`, so the
    /// work sorts first in run order, a smaller `rank` first (section 17:
    /// mix (c)'s works, and F5's growth works, each later growth before
    /// every earlier one).
    fn low_work(rank: u8) -> Uuid {
        Uuid::from_u128((u128::from(rank) << 96) | 0x0000_0000_0000_4000_8000_0000_0000_0000)
    }

    /// One group of works of one shape, written straight to the work-day
    /// projection. Work `n` is `md5('<tag>-<n>')`, so a group scatters across
    /// the corpus's work order, unless the group is one work with a low id
    /// (`rank`); its series are consecutive in the registry, so every group
    /// shares them.
    #[derive(Clone, Debug)]
    struct Shape {
        tag: String,
        works: i64,
        /// A low work id (`low_work`) for a group of one work.
        rank: Option<u8>,
        /// Series per work.
        series: i64,
        first_month: (i32, u32),
        months: i64,
        /// Active days per key-month, at most 28 (census shapes).
        days: i64,
        /// Country rows per active day (census shapes).
        countries: i64,
        /// One more country row on every n-th active day (0: never).
        extra_country_every: i64,
        /// Distinct countries per key-month the country rows rotate through,
        /// at most the smallest country set of a work.
        window: i64,
        /// The smallest and largest country set of a work.
        work_countries: (i64, i64),
        /// Key-days with one institution row, per thousand.
        institution_permille: i64,
        /// Key-months, per thousand, whose first active day has no
        /// undimensioned row and an institution row beside its country rows,
        /// so the month is ambiguous, as the section 16 ambiguity tests
        /// build it (census shapes; R11).
        ambiguity_permille: i64,
        /// Every row carries a country and an institution, each distinct
        /// within its key-month, and no row is undimensioned (F4's density,
        /// used by F4, mix (c) and F5).
        both: bool,
        /// With `both`: the rows per key-month, over at most 20 active days.
        rows_per_key_month: i64,
    }

    impl Shape {
        /// The SQL of work `n`'s id.
        fn work_sql(&self) -> String {
            match self.rank {
                Some(rank) => format!("'{}'::uuid", low_work(rank)),
                None => format!("md5('{}-' || n)::uuid", self.tag),
            }
        }

        /// Rows of one work, for batching.
        fn rows_per_work(&self) -> i64 {
            if self.both {
                self.series * self.months * self.rows_per_key_month
            } else {
                self.series * self.months * self.days * (self.countries + 2)
            }
        }
    }

    /// An F4-density shape: `series` series at `rows` rows per key-month
    /// over `years` years ending in `LAST_YEAR`.
    fn dense(tag: &str, works: i64, rank: Option<u8>, series: i64, years: i64, rows: i64) -> Shape {
        Shape {
            tag: tag.to_string(),
            works,
            rank,
            series,
            first_month: (LAST_YEAR + 1 - i32::try_from(years).expect("years"), 1),
            months: 12 * years,
            days: 0,
            countries: 0,
            extra_country_every: 0,
            window: 1,
            work_countries: (1, 1),
            institution_permille: 0,
            ambiguity_permille: 0,
            both: true,
            rows_per_key_month: rows,
        }
    }

    /// Seed works `from..=to` of `shape`; returns the work-day rows written.
    fn seed_shape(pool: &PgPool, corpus: &Corpus, shape: &Shape, from: i64, to: i64) -> usize {
        let work = shape.work_sql();
        exec(
            pool,
            &format!(
                "INSERT INTO work (work_id, work_type, work_status, imprint_id, edition) \
                 SELECT {work}, 'monograph', 'forthcoming', w.imprint_id, 1 \
                 FROM work w CROSS JOIN generate_series({from}, {to}) n \
                 WHERE w.work_id = '{}' \
                 ON CONFLICT (work_id) DO NOTHING",
                corpus.fixture.work_id
            ),
        );
        let base = scalar_i64(
            pool,
            "(SELECT COALESCE(MAX(watermark), 0) FROM metric_rollup_work_day)",
        )
        .max(state(pool).applied_through_sequence);
        let registry = corpus.platforms.len();
        let (year, month) = shape.first_month;
        let (series, months) = (shape.series, shape.months);
        let (days, kinds, filter, code, institution) = if shape.both {
            let rows = shape.rows_per_key_month;
            let days = rows.clamp(1, 20);
            let per_day = (rows + days - 1) / days;
            (
                days,
                (0..per_day)
                    .map(|c| format!("(1, {c})"))
                    .collect::<Vec<_>>()
                    .join(", "),
                format!("k * {per_day} + x.c < {rows}"),
                format!("(n * 13 + s * 7 + m * 3 + k * {per_day} + x.c) % 676"),
                format!("(n * 17 + s * 11 + m * 5 + k * {per_day} + x.c) % {INSTITUTIONS}"),
            )
        } else {
            let countries = shape.countries;
            let (low, high) = shape.work_countries;
            let extra = if shape.extra_country_every > 0 {
                format!(
                    "CASE WHEN k % {} = 0 THEN 1 ELSE 0 END",
                    shape.extra_country_every
                )
            } else {
                "0".to_string()
            };
            let ambiguous = format!(
                "(k = 0 AND (n * 7919 + s * 6271 + m * 3037) % 1000 < {})",
                shape.ambiguity_permille
            );
            let mut kinds = vec!["(0, 0)".to_string(), "(2, 0)".to_string()];
            kinds.extend((0..=countries).map(|c| format!("(1, {c})")));
            let set = format!("({low} + (n * 37) % {})", high - low + 1);
            (
                shape.days,
                kinds.join(", "),
                format!(
                    "(x.kind <> 0 OR NOT {ambiguous}) \
                     AND (x.kind <> 1 OR x.c < {countries} + {extra}) \
                     AND (x.kind <> 2 \
                          OR (n * 919 + s * 729 + m * 709 + k * 863) % 1000 < {} \
                          OR {ambiguous})",
                    shape.institution_permille
                ),
                format!(
                    "(n * 13 + ((m * 5 + s * 11) % {set} + (k * {} + x.c) % {}) % {set}) % 676",
                    countries + 1,
                    shape.window
                ),
                format!("(n * 17 + s * 11 + m * 5 + k) % {INSTITUTIONS}"),
            )
        };
        exec_count(
            pool,
            &format!(
                "INSERT INTO metric_rollup_work_day (work_id, publication_id, platform_id, \
                     measure_id, day, country_code, institution_id, value, watermark) \
                 SELECT g.work_id, NULL, g.platform_id, '{measure}', g.day, \
                        CASE WHEN g.kind = 1 \
                             THEN chr((65 + g.code / 26)::int) || chr((65 + g.code % 26)::int) END, \
                        CASE WHEN g.kind = 2 OR {both} \
                             THEN md5('ops03-institution-' || (1 + g.institution))::uuid END, \
                        1 + g.seed % 97, \
                        {base} + row_number() OVER () \
                 FROM ( \
                     SELECT {work} AS work_id, \
                            (ARRAY[{platforms}]::uuid[])[(1 + (n + s) % {registry})::int] \
                                AS platform_id, \
                            (DATE '{year:04}-{month:02}-01' + make_interval(months => m::int))::date \
                                + ((k * 28) / {days})::int AS day, \
                            x.kind, {code} AS code, {institution} AS institution, \
                            n * 7 + s * 5 + m * 3 + k + x.c AS seed \
                     FROM generate_series({from}::bigint, {to}) n \
                     CROSS JOIN generate_series(0::bigint, {series} - 1) s \
                     CROSS JOIN generate_series(0::bigint, {months} - 1) m \
                     CROSS JOIN generate_series(0::bigint, {days} - 1) k \
                     CROSS JOIN (VALUES {kinds}) AS x(kind, c) \
                     WHERE {filter}) g",
                measure = corpus.fixture.measure_id,
                both = shape.both,
                platforms = quoted(&corpus.platforms),
            ),
        )
    }

    /// Seed every work of `shape`, about a million rows per statement;
    /// returns the work-day rows written.
    fn seed_all(pool: &PgPool, corpus: &Corpus, shape: &Shape) -> usize {
        let batch = (1_000_000 / shape.rows_per_work().max(1)).max(1);
        let mut rows = 0;
        let mut from = 1;
        while from <= shape.works {
            let to = (from + batch - 1).min(shape.works);
            rows += seed_shape(pool, corpus, shape, from, to);
            from = to + 1;
        }
        rows
    }

    fn shape_works(pool: &PgPool, shape: &Shape) -> Vec<Uuid> {
        #[derive(QueryableByName)]
        struct Work {
            #[diesel(sql_type = SqlUuid)]
            work_id: Uuid,
        }
        let mut connection = pool.get().expect("Failed to get DB connection");
        sql_query(format!(
            "SELECT DISTINCT {} AS work_id FROM generate_series(1, {}) n ORDER BY 1",
            shape.work_sql(),
            shape.works
        ))
        .load::<Work>(&mut connection)
        .expect("the shape's works")
        .into_iter()
        .map(|work| work.work_id)
        .collect()
    }

    /// The shape facts of a seeded group: rows, month keys, rows per key,
    /// the share of key-days with an institution row and the country set
    /// sizes of its works.
    fn shape_facts(pool: &PgPool, shape: &Shape) -> (i64, i64) {
        #[derive(QueryableByName)]
        struct Facts {
            #[diesel(sql_type = BigInt)]
            rows: i64,
            #[diesel(sql_type = BigInt)]
            keys: i64,
            #[diesel(sql_type = BigInt)]
            key_days: i64,
            #[diesel(sql_type = BigInt)]
            institution_rows: i64,
            #[diesel(sql_type = BigInt)]
            least_countries: i64,
            #[diesel(sql_type = BigInt)]
            most_countries: i64,
            #[diesel(sql_type = BigInt)]
            most_keys_per_work_month: i64,
        }
        let mut connection = pool.get().expect("Failed to get DB connection");
        let facts: Facts = sql_query(format!(
            "WITH w AS (SELECT DISTINCT {} AS work_id FROM generate_series(1, {}) n), \
                  d AS (SELECT r.* FROM metric_rollup_work_day r JOIN w USING (work_id)), \
                  c AS (SELECT work_id, count(DISTINCT country_code) AS n FROM d GROUP BY work_id), \
                  m AS (SELECT count(DISTINCT (platform_id, measure_id)) AS n FROM d \
                        GROUP BY work_id, date_trunc('month', day)) \
             SELECT count(*) AS rows, \
                    count(DISTINCT (work_id, platform_id, measure_id, date_trunc('month', day))) \
                        AS keys, \
                    count(DISTINCT (work_id, platform_id, measure_id, day)) AS key_days, \
                    count(*) FILTER (WHERE institution_id IS NOT NULL) AS institution_rows, \
                    COALESCE((SELECT min(n) FROM c), 0) AS least_countries, \
                    COALESCE((SELECT max(n) FROM c), 0) AS most_countries, \
                    COALESCE((SELECT max(n) FROM m), 0) AS most_keys_per_work_month \
             FROM d",
            shape.work_sql(),
            shape.works
        ))
        .get_result(&mut connection)
        .expect("the shape facts");
        println!(
            "FIXTURE|{}|works {}|rows {}|month keys {}|month keys per work {:.1}|rows per key {:.1}|\
             key-days with an institution row {:.1}% (institution rows over key-days)|\
             countries per work {} to {}|most month keys per work-month {}",
            shape.tag,
            shape.works,
            facts.rows,
            facts.keys,
            facts.keys as f64 / shape.works as f64,
            facts.rows as f64 / facts.keys.max(1) as f64,
            100.0 * facts.institution_rows as f64 / facts.key_days.max(1) as f64,
            facts.least_countries,
            facts.most_countries,
            facts.most_keys_per_work_month,
        );
        (facts.rows, facts.keys)
    }

    /// Add one to the value of one country-month row in every unit of
    /// `works`; returns the rows damaged.
    fn damage_units(pool: &PgPool, works: &[Uuid]) -> usize {
        exec_count(
            pool,
            &format!(
                "UPDATE metric_rollup_work_country_month SET value = value + 1 \
                 WHERE rollup_work_country_month_id IN ( \
                     SELECT DISTINCT ON (work_id, date_trunc('year', month_start)) \
                            rollup_work_country_month_id \
                     FROM metric_rollup_work_country_month WHERE work_id IN ({}) \
                     ORDER BY work_id, date_trunc('year', month_start), \
                              rollup_work_country_month_id)",
                quoted(works)
            ),
        )
    }

    /// Remove `works`' work-day rows and every projection row of theirs;
    /// returns the rows removed.
    fn delete_work_rows(pool: &PgPool, works: &[Uuid]) -> usize {
        let list = quoted(works);
        let mut deleted = exec_count(
            pool,
            &format!("DELETE FROM metric_rollup_work_day WHERE work_id IN ({list})"),
        );
        for table in DERIVED_TABLES {
            deleted += exec_count(
                pool,
                &format!("DELETE FROM public.{table} WHERE work_id IN ({list})"),
            );
        }
        deleted
    }

    /// `ANALYZE` the work-day projection and the six projection tables.
    fn analyse(pool: &PgPool) {
        exec(
            pool,
            &format!(
                "ANALYZE metric_rollup_work_day, {}",
                DERIVED_TABLES.join(", ")
            ),
        );
    }

    /// The on-disk size of the rollup tables, with their indexes, and of the
    /// database.
    fn print_sizes(pool: &PgPool, label: &str) {
        println!(
            "SIZE|{label}|{}|database {}",
            text_scalar(
                pool,
                "SELECT string_agg(relname || ' ' || pg_size_pretty(pg_total_relation_size(oid)), \
                                   '; ' ORDER BY relname) \
                 FROM pg_class WHERE relname LIKE 'metric_rollup_work%' AND relkind = 'r'"
            )
            .unwrap_or_default(),
            text_scalar(
                pool,
                "SELECT pg_size_pretty(pg_database_size(current_database()))"
            )
            .unwrap_or_default()
        );
    }

    /// The vacuum counters and dead tuples of the rollup tables.
    fn vacuum_facts(pool: &PgPool) -> String {
        text_scalar(
            pool,
            "SELECT string_agg(relname || ' autovacuum ' || autovacuum_count || ' vacuum ' \
                               || vacuum_count || ' dead ' || n_dead_tup, '; ' ORDER BY relname) \
             FROM pg_stat_user_tables WHERE relname LIKE 'metric_rollup_work%'",
        )
        .unwrap_or_default()
    }

    /// `total` corruptions scattered over the six families at hashed
    /// positions: in each family half the picked rows move their value by one
    /// and half are deleted; the ambiguity family has no value, so its picked
    /// rows are deleted. Returns the rows corrupted per family.
    fn scatter_corruptions(pool: &PgPool, total: i64) -> Vec<(&'static str, i64)> {
        let families = [
            ("metric_rollup_work_month", "rollup_work_month_id", true),
            (
                "metric_rollup_work_country_month",
                "rollup_work_country_month_id",
                true,
            ),
            (
                "metric_rollup_work_institution_month",
                "rollup_work_institution_month_id",
                true,
            ),
            (
                "metric_rollup_work_month_ambiguity",
                "rollup_work_month_ambiguity_id",
                false,
            ),
            (
                "metric_rollup_work_country_year",
                "rollup_work_country_year_id",
                true,
            ),
            (
                "metric_rollup_work_institution_year",
                "rollup_work_institution_year_id",
                true,
            ),
        ];
        families
            .iter()
            .enumerate()
            .map(|(index, (table, key, valued))| {
                let count = total / 6 + i64::from((index as i64) < total % 6);
                let picked = format!(
                    "WITH picked AS ( \
                         SELECT {key} AS id, row_number() OVER (ORDER BY md5({key}::text)) AS n \
                         FROM {table} ORDER BY md5({key}::text) LIMIT {count})"
                );
                let statement = if *valued {
                    format!(
                        "{picked}, \
                         bumped AS (UPDATE {table} SET value = value + 1 \
                                    WHERE {key} IN (SELECT id FROM picked WHERE n % 2 = 1) \
                                    RETURNING 1), \
                         deleted AS (DELETE FROM {table} \
                                     WHERE {key} IN (SELECT id FROM picked WHERE n % 2 = 0) \
                                     RETURNING 1) \
                         SELECT (SELECT count(*) FROM bumped) + (SELECT count(*) FROM deleted) \
                             AS n"
                    )
                } else {
                    format!(
                        "{picked}, \
                         deleted AS (DELETE FROM {table} WHERE {key} IN (SELECT id FROM picked) \
                                     RETURNING 1) \
                         SELECT count(*) AS n FROM deleted"
                    )
                };
                (*table, count_of(pool, &statement))
            })
            .collect()
    }

    // -----------------------------------------------------------------------
    // Query plans
    // -----------------------------------------------------------------------

    #[derive(QueryableByName)]
    struct PlanLine {
        #[diesel(sql_type = Text, column_name = "QUERY PLAN")]
        line: String,
    }

    #[derive(QueryableByName)]
    struct KeyRow {
        #[diesel(sql_type = SqlUuid)]
        work_id: Uuid,
        #[diesel(sql_type = SqlUuid)]
        platform_id: Uuid,
        #[diesel(sql_type = SqlUuid)]
        measure_id: Uuid,
        #[diesel(sql_type = Date)]
        month_start: NaiveDate,
    }

    /// The rows of the work-day projection and of each projection table,
    /// read in the transaction of the statement about to be explained, just
    /// before it runs.
    fn table_rows(c: &mut PgConnection) -> Result<BTreeMap<String, i64>, DieselError> {
        let mut rows = BTreeMap::new();
        for table in std::iter::once("metric_rollup_work_day").chain(DERIVED_TABLES) {
            let count: Count =
                sql_query(format!("SELECT count(*) AS n FROM public.{table}")).get_result(c)?;
            rows.insert(table.to_string(), count.n);
        }
        Ok(rows)
    }

    /// Print one plan with the rows of each table when it ran; returns its
    /// sequential scans of the work-day projection or of a projection table
    /// that held rows (a scan of a table holding none is reported and not
    /// counted, section 17), and whether it reads the work-day projection
    /// through the series index. Its JIT use, from the plan's JIT lines, is
    /// printed too.
    fn print_plan(
        label: &str,
        statement: &str,
        plan: &[PlanLine],
        rows: &BTreeMap<String, i64>,
    ) -> (usize, bool) {
        let mut counted = 0;
        let mut empty = Vec::new();
        let (mut in_jit, mut functions, mut total) = (false, None, None);
        for line in plan {
            println!("PLAN|{label}|{statement}|{}", line.line);
            if let Some(rest) = line.line.split("Seq Scan on ").nth(1) {
                let table = rest.split_whitespace().next().unwrap_or_default();
                match rows.get(table) {
                    Some(n) if *n > 0 => counted += 1,
                    Some(_) => empty.push(table.to_string()),
                    None => {}
                }
            }
            let text = line.line.trim();
            if text == "JIT:" {
                in_jit = true;
            } else if in_jit {
                if let Some(value) = text.strip_prefix("Functions:") {
                    functions = Some(value.trim().to_string());
                } else if text.starts_with("Timing:") {
                    total = text
                        .split("Total ")
                        .nth(1)
                        .and_then(|rest| rest.split_whitespace().next())
                        .map(str::to_string);
                }
            }
        }
        let indexes: BTreeSet<String> = plan
            .iter()
            .filter_map(|line| line.line.split(" using ").nth(1))
            .filter_map(|rest| rest.split_whitespace().next())
            .map(str::to_string)
            .collect();
        let by_series = indexes.contains("metric_rollup_work_day_measure_id_platform_id_day_idx");
        println!(
            "PLANTABLES|{label}|{statement}|{}",
            rows.iter()
                .map(|(table, n)| format!("{table} {n} rows"))
                .collect::<Vec<_>>()
                .join("; ")
        );
        println!(
            "PLANSUMMARY|{label}|{statement}|sequential scans of a table holding rows \
             {counted}|sequential scans of a table holding none (reported, not counted) \
             {empty:?}|work-day by the series index {by_series}|indexes {indexes:?}"
        );
        println!(
            "PLANJIT|{label}|{statement}|{}",
            match (functions, total) {
                (Some(functions), Some(total)) => {
                    format!("JIT functions {functions}|JIT total {total} ms")
                }
                (Some(functions), None) => format!("JIT functions {functions}|JIT total unread"),
                _ => "no JIT lines: the plan used no JIT".to_string(),
            }
        );
        (counted, by_series)
    }

    macro_rules! bound {
        ($query:expr, $bounds:expr) => {
            $query
                .bind::<SqlUuid, _>($bounds.lo_work_id)
                .bind::<Nullable<Date>, _>($bounds.lo_end)
                .bind::<SqlUuid, _>($bounds.hi_work_id)
                .bind::<Nullable<Date>, _>($bounds.hi_end)
        };
    }

    macro_rules! keyed_by {
        ($query:expr, $keys:expr) => {
            $query
                .bind::<Array<SqlUuid>, _>($keys.iter().map(|key| key.0).collect::<Vec<Uuid>>())
                .bind::<Array<SqlUuid>, _>($keys.iter().map(|key| key.1).collect::<Vec<Uuid>>())
                .bind::<Array<SqlUuid>, _>($keys.iter().map(|key| key.2).collect::<Vec<Uuid>>())
                .bind::<Array<Date>, _>($keys.iter().map(|key| key.3).collect::<Vec<NaiveDate>>())
        };
    }

    /// EXPLAIN (ANALYZE, BUFFERS) of the statements a replacing step issues
    /// over `(lo, hi]`, as executed: with their bind parameters, under
    /// `plan_cache_mode = force_custom_plan`, in the step's order (sizing,
    /// the month keys, the six range deletes, the first chunk's watermark
    /// and month inserts, the first year chunk's inserts, the range
    /// verifier), in one transaction that is rolled back, each after a count
    /// of every table's rows. Returns the sequential scans of tables holding
    /// rows and the statements that read the work-day projection through the
    /// series index although led by work.
    fn explain_step(
        pool: &PgPool,
        label: &str,
        lo: Option<Unit>,
        hi: Option<Unit>,
    ) -> (usize, usize) {
        let bounds = range_bounds(lo, hi);
        let explain = |statement: &str| format!("EXPLAIN (ANALYZE, BUFFERS) {statement}");
        let mut counted = 0;
        let mut by_series = 0;
        let mut tally = |name: &str, plan: &[PlanLine], rows: &BTreeMap<String, i64>, led: bool| {
            let (scans, series) = print_plan(label, name, plan, rows);
            counted += scans;
            if led && series {
                by_series += 1;
            }
        };
        let mut connection = pool.get().expect("Failed to get DB connection");
        let outcome = connection.transaction::<(), DieselError, _>(|c| {
            sql_query("SET LOCAL plan_cache_mode = force_custom_plan").execute(c)?;
            let rows = table_rows(c)?;
            let plan: Vec<PlanLine> = sql_query(explain(SIZING_SQL))
                .bind::<SqlUuid, _>(bounds.lo_work_id)
                .bind::<Nullable<Date>, _>(bounds.lo_end)
                .bind::<BigInt, _>(METRIC_ROLLUP_RUN_SIZING_BATCH_WORKS)
                .bind::<BigInt, _>(METRIC_ROLLUP_RUN_STEP_MAX_WORK_DAY_ROWS)
                .bind::<BigInt, _>(METRIC_ROLLUP_RUN_STEP_MAX_MONTH_KEYS)
                .bind::<BigInt, _>(METRIC_ROLLUP_RUN_STEP_MAX_REPLACING_READ)
                .load(c)?;
            tally("sizing", &plan, &rows, true);
            let rows = table_rows(c)?;
            let plan: Vec<PlanLine> =
                bound!(sql_query(explain(RANGE_MONTH_KEYS_SQL)), bounds).load(c)?;
            tally("RANGE_MONTH_KEYS_SQL", &plan, &rows, true);
            let keys: Vec<KeyRow> = bound!(sql_query(RANGE_MONTH_KEYS_SQL), bounds).load(c)?;
            for (index, statement) in RANGE_DELETE_STATEMENTS.iter().enumerate() {
                let rows = table_rows(c)?;
                let plan: Vec<PlanLine> = bound!(sql_query(explain(statement)), bounds).load(c)?;
                tally(&format!("range delete {}", index + 1), &plan, &rows, true);
            }
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
            let chunk: Vec<MonthKey> = keys
                .iter()
                .copied()
                .take(METRIC_ROLLUP_RUN_STEP_CHUNK_KEYS)
                .collect();
            let rows = table_rows(c)?;
            let plan: Vec<PlanLine> =
                keyed_by!(sql_query(explain(MONTH_SOURCE_WATERMARK_SQL)), chunk).load(c)?;
            tally("month watermark (reused)", &plan, &rows, false);
            for (index, statement) in MONTH_INSERT_STATEMENTS.iter().enumerate() {
                let rows = table_rows(c)?;
                let plan: Vec<PlanLine> =
                    keyed_by!(sql_query(explain(statement)), chunk).load(c)?;
                tally(
                    &format!("month insert {} (reused)", index + 1),
                    &plan,
                    &rows,
                    false,
                );
            }
            let years: Vec<YearKey> = affected_year_keys(&keys)
                .into_iter()
                .take(METRIC_ROLLUP_RUN_STEP_CHUNK_KEYS)
                .collect();
            for (index, statement) in [YEAR_COUNTRY_INSERT_SQL, YEAR_INSTITUTION_INSERT_SQL]
                .iter()
                .enumerate()
            {
                let rows = table_rows(c)?;
                let plan: Vec<PlanLine> =
                    keyed_by!(sql_query(explain(statement)), years).load(c)?;
                tally(
                    &format!("year insert {} (reused)", index + 1),
                    &plan,
                    &rows,
                    false,
                );
            }
            let rows = table_rows(c)?;
            let plan: Vec<PlanLine> =
                bound!(sql_query(explain(RANGE_VERIFICATION_SQL)), bounds).load(c)?;
            tally("range verifier", &plan, &rows, true);
            Err(DieselError::RollbackTransaction)
        });
        assert!(
            matches!(outcome, Err(DieselError::RollbackTransaction)),
            "{label}: {outcome:?}"
        );
        (counted, by_series)
    }

    // -----------------------------------------------------------------------
    // The statement-group fits and the corner line (section 10)
    // -----------------------------------------------------------------------

    /// Solve `a x = b` by Gaussian elimination with partial pivoting.
    #[allow(clippy::needless_range_loop)]
    fn solve(mut a: Vec<Vec<f64>>, mut b: Vec<f64>) -> Option<Vec<f64>> {
        let n = b.len();
        for col in 0..n {
            let pivot = (col..n).max_by(|&i, &j| a[i][col].abs().total_cmp(&a[j][col].abs()))?;
            if a[pivot][col].abs() < 1e-12 {
                return None;
            }
            a.swap(col, pivot);
            b.swap(col, pivot);
            for row in col + 1..n {
                let factor = a[row][col] / a[col][col];
                for k in col..n {
                    a[row][k] -= factor * a[col][k];
                }
                b[row] -= factor * b[col];
            }
        }
        let mut x = vec![0.0; n];
        for row in (0..n).rev() {
            let mut sum = b[row];
            for k in row + 1..n {
                sum -= a[row][k] * x[k];
            }
            x[row] = sum / a[row][row];
        }
        Some(x)
    }

    fn predict(row: &[f64], coefficients: &[f64]) -> f64 {
        row.iter().zip(coefficients).map(|(x, c)| x * c).sum()
    }

    /// Non-negative least squares, exactly: the least-squares fit on every
    /// subset of the columns, keeping the one with the smallest residual
    /// whose coefficients are all non-negative.
    #[allow(clippy::needless_range_loop)]
    fn nnls(x: &[Vec<f64>], y: &[f64]) -> Vec<f64> {
        let width = x.first().map_or(0, Vec::len);
        let mut scale = vec![1.0f64; width];
        for j in 0..width {
            let max = x.iter().map(|row| row[j].abs()).fold(0.0, f64::max);
            if max > 0.0 {
                scale[j] = max;
            }
        }
        let rss = |coefficients: &[f64]| -> f64 {
            x.iter()
                .zip(y)
                .map(|(row, target)| (target - predict(row, coefficients)).powi(2))
                .sum()
        };
        let mut best = (vec![0.0; width], rss(&vec![0.0; width]));
        for mask in 1u32..(1 << width) {
            let columns: Vec<usize> = (0..width).filter(|j| mask & (1 << j) != 0).collect();
            let n = columns.len();
            let mut ata = vec![vec![0.0; n]; n];
            let mut aty = vec![0.0; n];
            for (row, target) in x.iter().zip(y) {
                for a in 0..n {
                    let xa = row[columns[a]] / scale[columns[a]];
                    aty[a] += xa * target;
                    for b in 0..n {
                        ata[a][b] += xa * row[columns[b]] / scale[columns[b]];
                    }
                }
            }
            let Some(solution) = solve(ata, aty) else {
                continue;
            };
            if solution.iter().any(|value| *value < 0.0) {
                continue;
            }
            let mut coefficients = vec![0.0; width];
            for a in 0..n {
                coefficients[columns[a]] = solution[a] / scale[columns[a]];
            }
            let residual = rss(&coefficients);
            if residual < best.1 {
                best = (coefficients, residual);
            }
        }
        best.0
    }

    /// Fit `target` on `drivers` over `rows` by NNLS; print the
    /// coefficients (milliseconds per unit of each driver) and the residual
    /// spread, and, with `residuals`, every residual. Returns the
    /// coefficients.
    fn fit(
        label: &str,
        names: &[&str],
        rows: &[Groups],
        drivers: impl Fn(&Groups) -> Vec<f64>,
        target: impl Fn(&Groups) -> f64,
        residuals: bool,
    ) -> Vec<f64> {
        let x: Vec<Vec<f64>> = rows.iter().map(&drivers).collect();
        let y: Vec<f64> = rows.iter().map(&target).collect();
        let c = nnls(&x, &y);
        println!(
            "FIT|{label}|n={}|{}",
            rows.len(),
            names
                .iter()
                .zip(&c)
                .map(|(name, value)| format!("{name} {value:.4e} ms"))
                .collect::<Vec<_>>()
                .join("|")
        );
        let mut absolute = Vec::new();
        for (row, (drivers, actual)) in rows.iter().zip(x.iter().zip(&y)) {
            let predicted = predict(drivers, &c);
            absolute.push((actual - predicted).abs());
            if residuals {
                println!(
                    "RESIDUAL|{label}|{}|actual {actual:.1} ms|predicted {predicted:.1} ms|\
                     residual {:.1} ms",
                    row.source,
                    actual - predicted
                );
            }
        }
        absolute.sort_by(f64::total_cmp);
        if let Some(largest) = absolute.last() {
            println!(
                "FIT|{label}|mean target {:.1} ms|absolute residual median {:.1} ms|p95 {:.1} \
                 ms|max {largest:.1} ms",
                y.iter().sum::<f64>() / y.len() as f64,
                absolute[absolute.len() / 2],
                absolute[(absolute.len() * 95 / 100).min(absolute.len() - 1)],
            );
        }
        c
    }

    /// The four group fits of the corner line (section 10).
    struct GroupFits {
        walk: Vec<f64>,
        inserts: Vec<f64>,
        verify: Vec<f64>,
        rest: Vec<f64>,
    }

    /// One corner's predicted hold from the group fits, at its drivers.
    #[allow(clippy::too_many_arguments)]
    fn corner(
        fits: &GroupFits,
        label: &str,
        rows: f64,
        whole: f64,
        series: f64,
        year_term: f64,
        walk_keys: f64,
        walk_months: f64,
        statements: f64,
    ) -> bool {
        let walk = predict(&[walk_keys, walk_months], &fits.walk);
        let inserts = predict(&[rows, year_term], &fits.inserts);
        let verify = predict(&[rows, whole], &fits.verify);
        let rest = predict(&[1.0, statements, whole], &fits.rest);
        let hold = walk + inserts + verify + rest;
        let met = hold <= 4_500.0;
        println!(
            "CORNER|{label}|rows {rows:.0}|whole-work rows {whole:.0}|series per work-month \
             {series:.2}|year term {year_term:.0}|walk drivers {walk_keys:.0} and \
             {walk_months:.0}|statements {statements:.0}|walk {walk:.1} ms|inserts {inserts:.1} \
             ms|verify {verify:.1} ms|rest {rest:.1} ms|predicted hold in repair {hold:.1} ms|\
             bound 4500 ms|{}",
            if met { "MET" } else { "MISSED" }
        );
        met
    }

    // -----------------------------------------------------------------------
    // F1, F7: the OPS-02 fixture and the sparse corpus
    // -----------------------------------------------------------------------

    #[test]
    #[ignore = "section 17 performance evidence; run explicitly"]
    fn f1_ops02_fixture() {
        let (_guard, pool) = setup();
        host_facts(&pool);
        let (fixture, _record_id) = fixture_record(&pool, "identity-base");
        let works = knob("THOTH_EVIDENCE_F1_WORKS", 2_000);
        let started = Instant::now();
        let rows = seed_production_shaped_days(&pool, &fixture, works);
        exec(&pool, "ANALYZE");
        let keys = month_key_count(&pool);
        println!(
            "FIXTURE|F1|{rows} work-day rows|{keys} month keys|{works} works|seeded in {} s",
            secs(started.elapsed())
        );
        let evidence = Evidence::new();
        let none = Classes::new();
        let passes = [
            ("F1 REBUILD from empty", Rebuild),
            ("F1 healthy REBUILD", Rebuild),
            ("F1 VERIFY", Verify),
        ]
        .map(|(pass, mode)| {
            let driven = drive_measured(
                &pool,
                &evidence,
                start(&pool, mode),
                &Drive::new(pass, &none),
            );
            finish_pass(&driven, keys, Reconciled);
            exec(&pool, "ANALYZE");
            driven
        });
        assert_eq!(
            passes[1].run.build.as_ref().expect("build").replaced_steps,
            0
        );
        let whole = verification(&pool);
        println!(
            "ACCEPT|F1 verifyMetricRollupMonths is exact after RECONCILED|{}",
            if whole.matches { "MET" } else { "MISSED" }
        );
        assert!(whole.matches);
        let refusals = |kinds: &[&str]| -> usize {
            passes
                .iter()
                .map(|driven| driven.budget_refusals(&[], kinds))
                .sum()
        };
        let non_replacing: Vec<&Sample> = passes.iter().flat_map(|p| p.steps(false)).collect();
        let pages: Vec<&Sample> = passes.iter().flat_map(Driven::pages).collect();
        accept(
            "F1 non-replacing step hold p95 <= 2 s",
            &non_replacing,
            Duration::from_secs(2),
            refusals(&["non-replacing step"]),
        );
        accept(
            "F1 replacing step hold p95 <= 3 s (from-empty build)",
            &passes[0].steps(true),
            Duration::from_secs(3),
            passes[0].budget_refusals(&[], &["replacing step"]),
        );
        accept(
            "F1 verify page time p95 <= 5 s",
            &pages,
            Duration::from_secs(5),
            refusals(&["page"]),
        );
    }

    /// F7. Its from-empty replacing steps' `GROUPS` lines also feed the
    /// rest group's fit (section 10), which the F2 test reads from this
    /// test's log (`THOTH_EVIDENCE_F7_LOG`).
    #[test]
    #[ignore = "section 17 performance evidence; run explicitly"]
    fn f7_sparse_corpus() {
        let (_guard, pool) = setup();
        host_facts(&pool);
        let corpus = corpus(&pool, 1);
        let works = knob("THOTH_EVIDENCE_F7_WORKS", 100_000);
        let started = Instant::now();
        seed_sparse(&pool, &corpus, works, 1);
        exec(&pool, "ANALYZE");
        let keys = month_key_count(&pool);
        println!(
            "FIXTURE|F7|{works} works|{keys} month keys, one per unit|seeded in {} s",
            secs(started.elapsed())
        );
        let evidence = Evidence::new();
        let none = Classes::new();
        let passes = [
            ("F7 REBUILD from empty", Rebuild),
            ("F7 healthy REBUILD", Rebuild),
        ]
        .map(|(pass, mode)| {
            let driven = drive_measured(
                &pool,
                &evidence,
                start(&pool, mode),
                &Drive::new(pass, &none),
            );
            finish_pass(&driven, keys, Reconciled);
            exec(&pool, "ANALYZE");
            driven
        });
        assert!(verification(&pool).matches);
        let pages: Vec<&Sample> = passes.iter().flat_map(Driven::pages).collect();
        accept(
            "F7 non-replacing step hold p95 <= 2 s",
            &passes[1].steps(false),
            Duration::from_secs(2),
            passes[1].budget_refusals(&[], &["non-replacing step"]),
        );
        accept(
            "F7 replacing step hold p95 <= 3 s (from-empty build)",
            &passes[0].steps(true),
            Duration::from_secs(3),
            passes[0].budget_refusals(&[], &["replacing step"]),
        );
        accept(
            "F7 verify page time p95 <= 5 s",
            &pages,
            Duration::from_secs(5),
            passes
                .iter()
                .map(|driven| driven.budget_refusals(&[], &["page"]))
                .sum(),
        );
    }

    // -----------------------------------------------------------------------
    // F4's classes, mix (c) and F5's growths
    // -----------------------------------------------------------------------

    /// The per-work facts F4 reports: rows, years, the most month keys of
    /// one work-month (k), rows per key-month, distinct countries over the
    /// whole history and per key-month against its rows (F4's density), and
    /// the section rows in both monthly section tables.
    fn f4_work_facts(pool: &PgPool, label: &str, works: &[Uuid]) {
        for work_id in works {
            let facts = text_scalar(
                pool,
                &format!(
                    "WITH d AS (SELECT * FROM metric_rollup_work_day WHERE work_id = '{work_id}'), \
                          km AS (SELECT count(*) AS r, count(DISTINCT country_code) AS c, \
                                        count(DISTINCT institution_id) AS i \
                                 FROM d GROUP BY platform_id, measure_id, date_trunc('month', day)) \
                     SELECT 'rows ' || (SELECT count(*) FROM d) \
                     || '|years ' || (SELECT count(DISTINCT date_trunc('year', day)) FROM d) \
                     || '|k (most month keys per work-month) ' \
                     || (SELECT max(n) FROM (SELECT count(DISTINCT (platform_id, measure_id)) n \
                                             FROM d GROUP BY date_trunc('month', day)) x) \
                     || '|rows per key-month ' \
                     || (SELECT round(avg(r), 1) FROM km) \
                     || '|distinct countries per key-month over its rows ' \
                     || (SELECT round(avg(c::numeric / r), 3) FROM km) \
                     || '|distinct institutions per key-month over its rows ' \
                     || (SELECT round(avg(i::numeric / r), 3) FROM km) \
                     || '|distinct countries over the whole history ' \
                     || (SELECT count(DISTINCT country_code) FROM d) \
                     || '|country section rows ' \
                     || (SELECT count(*) FROM metric_rollup_work_country_month \
                         WHERE work_id = '{work_id}') \
                     || '|institution section rows ' \
                     || (SELECT count(*) FROM metric_rollup_work_institution_month \
                         WHERE work_id = '{work_id}')"
                ),
            )
            .unwrap_or_default();
            println!("F4WORK|{label}|{work_id}|{facts}");
        }
    }

    /// One F4 line: the largest hold over fill of `steps` (fill: the smaller
    /// of row fill and figure fill), with the class's budget refusals in the
    /// pass; returns whether it was met.
    fn mix_line(line: &str, steps: &[&Sample], budget_refusals: usize) -> bool {
        let worst = steps
            .iter()
            .map(|sample| millis(sample.hold()) / sample.terms.fill().max(1e-9))
            .fold(0.0, f64::max);
        let range = |pick: &dyn Fn(&Terms) -> f64| {
            let values: Vec<f64> = steps.iter().map(|sample| pick(&sample.terms)).collect();
            format!(
                "{:.3} to {:.3}",
                values.iter().copied().fold(f64::INFINITY, f64::min),
                values.iter().copied().fold(0.0, f64::max)
            )
        };
        let met = !steps.is_empty() && worst <= millis(F4_LINE) && budget_refusals == 0;
        println!(
            "ACCEPT|{line}|n={}|row fill {}|figure fill {}|fill {}|max hold / fill {worst:.1} \
             ms|k {}|largest whole-work total {}|{}|budget refusals {budget_refusals}|diagnostic \
             measurements {}|bound 4000 ms|{}",
            steps.len(),
            range(&Terms::row_fill),
            range(&Terms::figure_fill),
            range(&Terms::fill),
            steps
                .iter()
                .map(|sample| sample.terms.max_month_keys)
                .max()
                .unwrap_or(0),
            steps
                .iter()
                .map(|sample| sample.terms.max_whole)
                .max()
                .unwrap_or(0),
            walk_rate(steps),
            diagnostic_count(steps),
            if steps.is_empty() {
                "NOT MEASURED"
            } else if met {
                "MET"
            } else {
                "MISSED"
            }
        );
        met
    }

    /// A report-only pass over late works that sort first in run order
    /// (mix (c), F5): a build or a repair until the cursor reaches
    /// `stop_at`, after which the run is abandoned.
    fn late_pass(
        pool: &PgPool,
        evidence: &Evidence,
        pass: &str,
        classes: &Classes,
        stop_at: Unit,
    ) -> Driven {
        let mut drive = Drive::new(pass, classes);
        drive.build_only = true;
        drive.stop_at = Some(stop_at);
        drive.max_streak = 3;
        let driven = drive_measured(pool, evidence, start(pool, Rebuild), &drive);
        let _ = abandon(pool, driven.run.run_id);
        print_raw(&driven);
        summarise(&driven, 0);
        driven
    }

    /// Mix (c), report only (section 17): a work of one series and a work of
    /// two at F4's density, light years before heavy years, so a step takes
    /// several light years with the figure full. Seeded after every gated
    /// pass with low work ids, analysed, built, damaged and repaired; its
    /// works are removed afterwards, before F5's are seeded.
    fn mix_c(pool: &PgPool, corpus: &Corpus, evidence: &Evidence) {
        let light_years = knob("THOTH_EVIDENCE_F4C_LIGHT_YEARS", 10);
        let heavy_years = knob("THOTH_EVIDENCE_F4C_HEAVY_YEARS", 12);
        let works = [
            (
                "F4c1",
                0x10_u8,
                1,
                knob("THOTH_EVIDENCE_F4C1_LIGHT", 67),
                knob("THOTH_EVIDENCE_F4C1_HEAVY", 625),
            ),
            (
                "F4c2",
                0x11_u8,
                2,
                knob("THOTH_EVIDENCE_F4C2_LIGHT", 33),
                knob("THOTH_EVIDENCE_F4C2_HEAVY", 312),
            ),
        ];
        let mut classes = Classes::new();
        for (class, rank, series, light, heavy) in works {
            let mut early = dense(class, 1, Some(rank), series, light_years, light);
            early.first_month = (
                LAST_YEAR + 1 - i32::try_from(light_years + heavy_years).expect("years"),
                1,
            );
            let late = dense(class, 1, Some(rank), series, heavy_years, heavy);
            let rows =
                seed_shape(pool, corpus, &early, 1, 1) + seed_shape(pool, corpus, &late, 1, 1);
            println!(
                "FIXTURE|mix (c) {class}|{series} series|{light_years} light years at {light} \
                 rows per key-month before {heavy_years} heavy years at {heavy}|{rows} work-day \
                 rows|work {}",
                low_work(rank)
            );
            classes.push((class, [low_work(rank)].into_iter().collect()));
        }
        settle(pool);
        analyse(pool);
        let mix_works: Vec<Uuid> = classes
            .iter()
            .flat_map(|(_, set)| set.iter().copied())
            .collect();
        let last = unit(*mix_works.iter().max().expect("mix (c)'s works"), LAST_YEAR);
        let built = late_pass(
            pool,
            evidence,
            "mix (c) build (report only)",
            &classes,
            last,
        );
        analyse(pool);
        for (class, set) in &classes {
            f4_work_facts(pool, class, &set.iter().copied().collect::<Vec<_>>());
        }
        let damaged = damage_units(pool, &mix_works);
        println!(
            "VACUUM|before mix (c)'s repair|damaged rows {damaged}|{}",
            vacuum_facts(pool)
        );
        let repaired = late_pass(
            pool,
            evidence,
            "mix (c) repair (report only)",
            &classes,
            last,
        );
        for (class, _) in &classes {
            for driven in [&built, &repaired] {
                let steps: Vec<&Sample> = driven
                    .steps(true)
                    .into_iter()
                    .filter(|sample| sample.class == *class)
                    .collect();
                let worst = steps
                    .iter()
                    .map(|sample| sample.hold())
                    .max()
                    .unwrap_or_default();
                println!(
                    "MIXC|{class}|{}|report only|replacing steps {}|hold {}|max hold {} ms against \
                     STEP_HOLD_BUDGET {} ms|refusals under the section 10 budgets {}|diagnostic \
                     measurements {}|largest whole-work total {}|light-year steps: units {}|fill \
                     {}|{}",
                    driven.pass,
                    steps.len(),
                    spread_text(spread(&steps, Sample::hold)),
                    ms(worst),
                    METRIC_ROLLUP_RUN_STEP_HOLD_BUDGET.as_millis(),
                    driven.budget_refusals(&[class], &[]),
                    diagnostic_count(&steps),
                    steps.iter().map(|sample| sample.terms.max_whole).max().unwrap_or(0),
                    count_spread(steps.iter().map(|sample| usize::try_from(sample.terms.units).unwrap_or(0))),
                    steps
                        .iter()
                        .map(|sample| format!("{:.3}", sample.terms.fill()))
                        .collect::<Vec<_>>()
                        .join(" "),
                    walk_rate(&steps),
                );
            }
        }
        println!(
            "MIXC|removed mix (c)'s works before F5|{} rows",
            delete_work_rows(pool, &mix_works)
        );
    }

    /// One F5 growth (section 17): a work with a low id, `series` series per
    /// work-month and `years` of history, every year holding the same rows.
    struct Growth {
        number: usize,
        class: &'static str,
        label: &'static str,
        series: i64,
        years: i64,
        rank: u8,
        /// The coarse levels, in rows per key-month; the first inside every
        /// step budget.
        coarse: Vec<i64>,
    }

    /// One F5 level's outcome.
    struct Level {
        rows_per_key_month: i64,
        /// Every replacing step of the measured repair pass committed under
        /// the section 10 budgets within 4 s.
        within: bool,
        /// The direct counts of the work's last unit.
        unit: Terms,
        whole: i64,
        steps: Vec<Sample>,
    }

    /// Grow one level: replace the growth work's rows by every year at
    /// `rows` rows per key-month, analyse, build its range, analyse, damage
    /// one row per unit and measure the repair; each pass's run is abandoned
    /// once the growth work's range is replaced.
    fn growth_level(
        pool: &PgPool,
        corpus: &Corpus,
        evidence: &Evidence,
        growth: &Growth,
        rows: i64,
    ) -> Level {
        let work_id = low_work(growth.rank);
        delete_work_rows(pool, &[work_id]);
        let shape = dense(
            growth.class,
            1,
            Some(growth.rank),
            growth.series,
            growth.years,
            rows,
        );
        let seeded = seed_shape(pool, corpus, &shape, 1, 1);
        settle(pool);
        exec(pool, "ANALYZE metric_rollup_work_day");
        let classes: Classes = vec![(growth.class, [work_id].into_iter().collect())];
        let last = unit(work_id, LAST_YEAR);
        let built = late_pass(
            pool,
            evidence,
            &format!(
                "F5 growth {} at {rows} rows per key-month: build",
                growth.number
            ),
            &classes,
            last,
        );
        analyse(pool);
        let damaged = damage_units(pool, &[work_id]);
        let repaired = late_pass(
            pool,
            evidence,
            &format!(
                "F5 growth {} at {rows} rows per key-month: repair",
                growth.number
            ),
            &classes,
            last,
        );
        let steps: Vec<Sample> = repaired
            .samples
            .iter()
            .filter(|sample| !sample.page && sample.replaced && sample.class == growth.class)
            .cloned()
            .collect();
        let worst = steps.iter().map(Sample::hold).max().unwrap_or_default();
        let within = repaired.refused.is_empty()
            && repaired.stuck.is_none()
            && !steps.is_empty()
            && steps.iter().all(|sample| !sample.diagnostic)
            && worst <= F4_LINE;
        let unit_terms = terms(pool, Some(unit(work_id, LAST_YEAR - 1)), Some(last));
        let steps_ref: Vec<&Sample> = steps.iter().collect();
        println!(
            "GROWTH|{}|{rows} rows per key-month|series {}|years {}|unit rows {}|unit month keys \
             {}|unit figure {}|whole-work rows {seeded}|damaged units {damaged}|build refusals \
             {}|repair steps {}|repair hold {}|max {} ms|pre-sizing {}|re-sizing {}|refusals {:?}|\
             stuck {:?}|within 4 s {within}",
            growth.number,
            growth.series,
            growth.years,
            unit_terms.rows,
            unit_terms.month_keys,
            unit_terms.figure(),
            built.refused.len(),
            steps.len(),
            spread_text(spread(&steps_ref, Sample::hold)),
            ms(worst),
            spread_text(spread(&steps_ref, |sample| sample.timing.presizing)),
            spread_text(spread(&steps_ref, |sample| sample.timing.resizing)),
            repaired.by_class(),
            repaired.stuck,
        );
        Level {
            rows_per_key_month: rows,
            within,
            unit: unit_terms,
            whole: i64::try_from(seeded).expect("rows"),
            steps,
        }
    }

    /// Run one growth: the coarse levels until one crosses, then halving
    /// down to a gap of four rows per key-month, then one row at a time up
    /// to the crossing (section 17). Returns the largest level within 4 s.
    fn run_growth(
        pool: &PgPool,
        corpus: &Corpus,
        evidence: &Evidence,
        growth: &Growth,
    ) -> Option<Level> {
        // A dry-run knob only: the specified run measures every level.
        let limit = usize::try_from(knob("THOTH_EVIDENCE_F5_MAX_LEVELS", 0)).unwrap_or(0);
        let room = |measured: usize| limit == 0 || measured < limit;
        let mut measured = 0;
        let mut best: Option<Level> = None;
        let mut crossing = None;
        for rows in &growth.coarse {
            if !room(measured) {
                break;
            }
            measured += 1;
            let outcome = growth_level(pool, corpus, evidence, growth, *rows);
            if outcome.within {
                best = Some(outcome);
            } else {
                crossing = Some(*rows);
                break;
            }
        }
        if let (Some(within), Some(mut above)) = (
            best.as_ref().map(|level| level.rows_per_key_month),
            crossing,
        ) {
            let mut below = within;
            while above - below > 4 && room(measured) {
                measured += 1;
                let middle = (below + above) / 2;
                let outcome = growth_level(pool, corpus, evidence, growth, middle);
                if outcome.within {
                    below = middle;
                    best = Some(outcome);
                } else {
                    above = middle;
                }
            }
            let mut rows = below + 1;
            while rows < above && room(measured) {
                measured += 1;
                let outcome = growth_level(pool, corpus, evidence, growth, rows);
                if !outcome.within {
                    break;
                }
                best = Some(outcome);
                rows += 1;
            }
        }
        match &best {
            Some(level) => {
                let steps: Vec<&Sample> = level.steps.iter().collect();
                println!(
                    "F5|growth {}: {}|largest unit committing within 4 s|{} rows per key-month|\
                     rows {}|month keys {}|replacing-read figure {}|whole-work rows {}|years {}|\
                     its repair steps' holds {}|sizing of its work: pre-sizing {}, re-sizing {}|\
                     first crossing level measured {:?}",
                    growth.number,
                    growth.label,
                    level.rows_per_key_month,
                    level.unit.rows,
                    level.unit.month_keys,
                    level.unit.figure(),
                    level.whole,
                    growth.years,
                    spread_text(spread(&steps, Sample::hold)),
                    spread_text(spread(&steps, |sample| sample.timing.presizing)),
                    spread_text(spread(&steps, |sample| sample.timing.resizing)),
                    crossing,
                );
            }
            None => println!(
                "F5|growth {}: {}|no level committed within 4 s",
                growth.number, growth.label
            ),
        }
        best
    }

    /// F5's three growths, all at F4's density (section 17), after every
    /// gated pass and mix (c): each growth work has a low id, a later growth
    /// before every earlier one in run order.
    fn f5(pool: &PgPool, corpus: &Corpus, evidence: &Evidence, k: i64) -> Vec<Option<Level>> {
        let growths = [
            Growth {
                number: 1,
                class: "F5g1",
                label: "at least mix (b)'s k series, one year of history",
                series: k,
                years: 1,
                rank: 0x03,
                coarse: vec![2, 8, 16, 24, 32, 40, 48, 64, 80],
            },
            Growth {
                number: 2,
                class: "F5g2",
                label: "at least mix (b)'s k series, eight years of history",
                series: k,
                years: 8,
                rank: 0x02,
                coarse: vec![3, 6, 9, 12, 15, 18, 24, 30],
            },
            Growth {
                number: 3,
                class: "F5g3",
                label: "F3's four series, eight years of history",
                series: 4,
                years: 8,
                rank: 0x01,
                coarse: vec![
                    20, 60, 100, 140, 180, 220, 260, 300, 340, 400, 480, 560, 640,
                ],
            },
        ];
        growths
            .iter()
            .map(|growth| run_growth(pool, corpus, evidence, growth))
            .collect()
    }

    /// The rest group's rows from F7's own log (section 10: the rest is
    /// fitted on the from-empty replacing steps of F2, F3, F4 and F7).
    fn f7_groups() -> Vec<Groups> {
        let Ok(path) = std::env::var("THOTH_EVIDENCE_F7_LOG") else {
            println!("F7LOG|THOTH_EVIDENCE_F7_LOG is not set: the rest fit has no F7 steps");
            return Vec::new();
        };
        let text = std::fs::read_to_string(&path).unwrap_or_default();
        let rows: Vec<Groups> = text
            .lines()
            .filter(|line| line.starts_with("GROUPS|F7 REBUILD from empty|"))
            .filter_map(Groups::parse)
            .collect();
        println!(
            "F7LOG|{path}|{} from-empty replacing steps read",
            rows.len()
        );
        rows
    }

    // -----------------------------------------------------------------------
    // F2 with F3 and F4 embedded, F6 on it, then mix (c) and F5
    // -----------------------------------------------------------------------

    #[test]
    #[ignore = "section 17 performance evidence; run explicitly"]
    fn f2_census_corpus_with_f3_f4_and_f6() {
        let (_guard, pool) = setup();
        host_facts(&pool);
        let k = knob("THOTH_EVIDENCE_F4_K", 39);
        let corpus = Arc::new(census_corpus(&pool, registry_for(k)));
        let window = knob("THOTH_EVIDENCE_F2_WINDOW", 10);
        let books = Shape {
            tag: "ops03-f2-book".to_string(),
            works: knob("THOTH_EVIDENCE_F2_BOOKS", 2_000),
            rank: None,
            series: 4,
            first_month: (2021, 12),
            months: 49,
            days: 10,
            countries: 2,
            extra_country_every: 0,
            window,
            work_countries: (40, 160),
            institution_permille: 150,
            ambiguity_permille: knob("THOTH_EVIDENCE_F2_AMBIGUITY_PERMILLE", 20),
            both: false,
            rows_per_key_month: 0,
        };
        let chapters = Shape {
            tag: "ops03-f2-chapter".to_string(),
            works: knob("THOTH_EVIDENCE_F2_CHAPTERS", 2_600),
            series: 2,
            first_month: (2024, 4),
            months: 21,
            ..books.clone()
        };
        let f3 = Shape {
            tag: "ops03-f3".to_string(),
            works: knob("THOTH_EVIDENCE_F3_WORKS", 4),
            series: 4,
            first_month: (2018, 1),
            months: 96,
            days: 28,
            countries: 4,
            extra_country_every: 7,
            window: 160,
            work_countries: (160, 160),
            ambiguity_permille: 0,
            ..books.clone()
        };
        let f4a = dense(
            "ops03-f4a",
            knob("THOTH_EVIDENCE_F4A_WORKS", 2),
            None,
            knob("THOTH_EVIDENCE_F4A_SERIES", 8),
            knob("THOTH_EVIDENCE_F4A_YEARS", 8),
            knob("THOTH_EVIDENCE_F4A_ROWS", 80),
        );
        let f4a2 = dense(
            "ops03-f4a2",
            knob("THOTH_EVIDENCE_F4A2_WORKS", 2),
            None,
            knob("THOTH_EVIDENCE_F4A2_SERIES", 5),
            knob("THOTH_EVIDENCE_F4A2_YEARS", 14),
            knob("THOTH_EVIDENCE_F4A2_ROWS", 128),
        );
        let f4b = dense(
            "ops03-f4b",
            knob("THOTH_EVIDENCE_F4B_WORKS", 8),
            None,
            k,
            1,
            knob("THOTH_EVIDENCE_F4B_ROWS", 16),
        );
        let shapes = [&books, &chapters, &f3, &f4a, &f4a2, &f4b];
        let started = Instant::now();
        for shape in shapes {
            let rows = seed_all(&pool, &corpus, shape);
            println!(
                "FIXTURE|{}|seeded {rows} work-day rows over {} works|{} s",
                shape.tag,
                shape.works,
                secs(started.elapsed())
            );
        }
        settle(&pool);
        exec(&pool, "ANALYZE");
        let mut keys = month_key_count(&pool);
        let rows = scalar_i64(&pool, "(SELECT COUNT(*) FROM metric_rollup_work_day)");
        println!(
            "FIXTURE|F2|{rows} work-day rows|{keys} month keys|registry {} series|k {k}|\
             seeded and analysed in {} s",
            corpus.platforms.len(),
            secs(started.elapsed())
        );
        let mut base = (0, 0);
        for shape in shapes {
            let (shape_rows, shape_keys) = shape_facts(&pool, shape);
            if shape.tag.starts_with("ops03-f2-") {
                base = (base.0 + shape_rows, base.1 + shape_keys);
            }
        }
        let density = base.0 as f64 / base.1.max(1) as f64;
        let largest_r =
            (METRIC_ROLLUP_RUN_STEP_MAX_REPLACING_READ as f64 / (108.0 * density)).sqrt();
        println!(
            "FLOOR|F2 rows per key {density:.2}|floor figure {:.0} R^2 for R registry pairs|\
             STEP_MAX_REPLACING_READ admits R <= {}|at R = 13: {:.0}|at R = 14: {:.0}",
            108.0 * density,
            largest_r.floor(),
            108.0 * density * 169.0,
            108.0 * density * 196.0,
        );
        print_sizes(&pool, "after seeding");

        let f3_works = shape_works(&pool, &f3);
        let f4a_works = shape_works(&pool, &f4a);
        let f4a2_works = shape_works(&pool, &f4a2);
        let f4b_works = shape_works(&pool, &f4b);
        let classes: Classes = vec![
            ("F4a2", f4a2_works.iter().copied().collect()),
            ("F4a", f4a_works.iter().copied().collect()),
            ("F4b", f4b_works.iter().copied().collect()),
            ("F3", f3_works.iter().copied().collect()),
        ];
        let evidence = Evidence::new();

        // REBUILD from empty, F3 and F4 embedded; then ANALYZE.
        let from_empty = drive_measured(
            &pool,
            &evidence,
            start(&pool, Rebuild),
            &Drive::new("F2 REBUILD from empty", &classes),
        );
        finish_pass(&from_empty, keys, Reconciled);
        exec(&pool, "ANALYZE");
        print_sizes(&pool, "after the from-empty build");
        println!(
            "AMBIGUITY|ambiguity rows after the from-empty build {}|base month keys {}",
            scalar_i64(
                &pool,
                "(SELECT COUNT(*) FROM metric_rollup_work_month_ambiguity)"
            ),
            base.1
        );
        println!("VACUUM|after the from-empty build|{}", vacuum_facts(&pool));
        f4_work_facts(&pool, "F4a", &f4a_works);
        f4_work_facts(&pool, "F4a2", &f4a2_works);
        f4_work_facts(&pool, "F4b", &f4b_works);

        // Query plans on F2, F3 and F4 ranges, before any late work.
        let mut counted = 0;
        let mut by_series = 0;
        for (class, label) in [
            ("base", "F2"),
            ("F3", "F3"),
            ("F4a", "F4a"),
            ("F4a2", "F4a2"),
            ("F4b", "F4b"),
        ] {
            let mut candidates: Vec<&Sample> = from_empty
                .steps(true)
                .into_iter()
                .filter(|sample| sample.class == class)
                .collect();
            candidates.sort_by_key(|sample| sample.hold());
            if let Some(sample) = candidates.get(candidates.len() / 2) {
                let (scans, series) = explain_step(&pool, label, sample.lo, sample.hi);
                counted += scans;
                by_series += series;
            }
        }
        println!(
            "ACCEPT|EXPLAIN on F2, F3 and F4: no sequential scan of metric_rollup_work_day or of \
             a projection table holding rows|{counted} found|{}",
            if counted == 0 { "MET" } else { "MISSED" }
        );
        println!(
            "ACCEPT|EXPLAIN: sizing, RANGE_MONTH_KEYS_SQL, the range deletes and the range \
             verifier are led by work_id|{by_series} read work-day rows through the series \
             index|{}",
            if by_series == 0 { "MET" } else { "MISSED" }
        );

        // F4 repair passes over fully built tables: the first, which the
        // fits use, and its repeat without an intervening vacuum (report
        // only).
        let f4_works: Vec<Uuid> = f4a_works
            .iter()
            .chain(&f4a2_works)
            .chain(&f4b_works)
            .copied()
            .collect();
        let mut repairs = Vec::new();
        for pass in [
            "F4 repair pass",
            "F4 repair pass repeated without vacuum (report only)",
        ] {
            println!("VACUUM|before {pass}|{}", vacuum_facts(&pool));
            let damaged = damage_units(&pool, &f4_works);
            let driven = drive_measured(
                &pool,
                &evidence,
                start(&pool, Rebuild),
                &Drive::new(pass, &classes),
            );
            finish_pass(&driven, keys, Reconciled);
            println!(
                "VACUUM|after {pass}|damaged rows {damaged}|{}",
                vacuum_facts(&pool)
            );
            repairs.push(driven);
        }

        // 200 scattered corruptions, VERIFY, then REBUILD.
        let corrupted = scatter_corruptions(&pool, 200);
        println!(
            "CORRUPTED|{corrupted:?}|applied {} of 200",
            corrupted.iter().map(|(_, n)| n).sum::<i64>()
        );
        let verified = drive_measured(
            &pool,
            &evidence,
            start(&pool, Verify),
            &Drive::new("F2 VERIFY after 200 corruptions", &classes),
        );
        finish_pass(&verified, keys, Mismatched);
        println!("MISMATCHED|{:?}", run_families(&verified.run));
        let repaired = drive_measured(
            &pool,
            &evidence,
            start(&pool, Rebuild),
            &Drive::new("F2 repair REBUILD", &classes),
        );
        finish_pass(&repaired, keys, Reconciled);
        whole_table(&pool, "F2 after the repair REBUILD");
        println!(
            "F2|200 corruptions -> VERIFY {:?} -> REBUILD {:?} with {} replacing steps",
            verified.run.status,
            repaired.run.status,
            repaired.steps(true).len()
        );

        // Acceptance on the gated passes.
        let all = [&from_empty, &repairs[0], &repairs[1], &verified, &repaired];
        let refusals = |classes: &[&str], kinds: &[&str]| -> usize {
            all.iter()
                .map(|driven| driven.budget_refusals(classes, kinds))
                .sum()
        };
        let non_replacing: Vec<&Sample> =
            all.iter().flat_map(|driven| driven.steps(false)).collect();
        let pages: Vec<&Sample> = all.iter().flat_map(|driven| driven.pages()).collect();
        for (class, label) in [("base", "F2"), ("F3", "F3")] {
            accept(
                &format!("{label} non-replacing step hold p95 <= 2 s"),
                &of_class(non_replacing.clone(), &[class]),
                Duration::from_secs(2),
                refusals(&[class], &["non-replacing step"]),
            );
            accept(
                &format!("{label} replacing step hold p95 <= 3 s (from-empty build)"),
                &of_class(from_empty.steps(true), &[class]),
                Duration::from_secs(3),
                from_empty.budget_refusals(&[class], &["replacing step"]),
            );
        }
        accept(
            "F2 replacing step hold p95 <= 3 s (its repair run; F2 and F3 ranges)",
            &of_class(repaired.steps(true), &["base", "F3"]),
            Duration::from_secs(3),
            repaired.budget_refusals(&["base", "F3"], &["replacing step"]),
        );
        // F4 ranges are judged by the F4 line; their repair-run holds are
        // reported beside it.
        accept(
            "F2 repair run, every replacing step including F4 ranges (report only)",
            &repaired.steps(true),
            Duration::from_secs(3),
            repaired.budget_refusals(&[], &["replacing step"]),
        );
        let mut page_met = BTreeMap::new();
        for (class, label) in [
            ("base", "F2"),
            ("F3", "F3"),
            ("F4a", "F4a"),
            ("F4a2", "F4a2 (mix (a)'s second length: a miss caps T)"),
            ("F4b", "F4b"),
        ] {
            let met = accept(
                &format!("{label} verify page time p95 <= 5 s"),
                &of_class(pages.clone(), &[class]),
                Duration::from_secs(5),
                refusals(&[class], &["page"]),
            );
            page_met.insert(class, met);
        }
        let mut line_met = BTreeMap::new();
        for (class, mix) in [
            ("F4a", "mix (a)"),
            ("F4a2", "mix (a)'s second length (a miss caps T)"),
            ("F4b", "mix (b)"),
        ] {
            let step_kinds = ["replacing step", "non-replacing step"];
            let empty = mix_line(
                &format!("F4 {mix} from-empty steps: max of hold / fill <= 4 s"),
                &of_class(from_empty.steps(true), &[class]),
                from_empty.budget_refusals(&[class], &step_kinds),
            );
            let repair = mix_line(
                &format!("F4 {mix} repair steps: max of hold / fill <= 4 s"),
                &of_class(repairs[0].steps(true), &[class]),
                repairs[0].budget_refusals(&[class], &step_kinds),
            );
            mix_line(
                &format!("F4 {mix} repair steps repeated without vacuum (report only)"),
                &of_class(repairs[1].steps(true), &[class]),
                repairs[1].budget_refusals(&[class], &step_kinds),
            );
            line_met.insert(class, empty && repair);
        }
        let second_length_met = line_met["F4a2"] && page_met["F4a2"];
        println!(
            "SECONDLENGTH|F4 line {}|page line {}|{}",
            line_met["F4a2"],
            page_met["F4a2"],
            if second_length_met {
                "met: T may come from F4 and F5"
            } else {
                "MISSED: T comes from the F4 classes that met both lines, and F6 runs without \
                 the second length's works (section 10)"
            }
        );

        // F6 on the whole of F2 with its F3 and F4 works, before any late
        // work; without the second length's works after its miss.
        if knob("THOTH_EVIDENCE_F6", 1) != 0 {
            if !second_length_met {
                println!(
                    "F6|removed the second length's works before F6|{} rows",
                    delete_work_rows(&pool, &f4a2_works)
                );
                keys = month_key_count(&pool);
            }
            let book_works = shape_works(
                &pool,
                &Shape {
                    works: books.works.min(80),
                    ..books.clone()
                },
            );
            f6(&pool, &corpus, &evidence, &book_works, keys);
        }

        // The late works, report only: mix (c), then F5.
        if knob("THOTH_EVIDENCE_F4C", 1) != 0 {
            mix_c(&pool, &corpus, &evidence);
        }
        let growths = if knob("THOTH_EVIDENCE_F5", 1) != 0 {
            f5(&pool, &corpus, &evidence, k)
        } else {
            Vec::new()
        };

        // The whole-work threshold T (section 10).
        let t_classes: Vec<&str> = ["F4a", "F4a2", "F4b"]
            .into_iter()
            .filter(|class| *class != "F4a2" || second_length_met)
            .collect();
        let f4_threshold = [&from_empty, &repairs[0]]
            .iter()
            .flat_map(|driven| driven.steps(true))
            .filter(|sample| t_classes.contains(&sample.class))
            .filter(|sample| !sample.diagnostic && sample.hold() <= F4_LINE)
            .map(|sample| sample.terms.max_whole)
            .max()
            .unwrap_or(0);
        let f5_threshold = growths
            .iter()
            .flatten()
            .map(|level| level.whole)
            .max()
            .unwrap_or(0);
        let threshold = if second_length_met {
            f4_threshold.max(f5_threshold)
        } else {
            f4_threshold
        };
        let f3_total = f3_works
            .iter()
            .map(|work_id| {
                scalar_i64(
                    &pool,
                    &format!(
                        "(SELECT COUNT(*) FROM metric_rollup_work_day WHERE work_id = '{work_id}')"
                    ),
                )
            })
            .max()
            .unwrap_or(0);
        println!(
            "THRESHOLD|F4 classes {t_classes:?}: largest whole-work total of a replacing step \
             within 4 s {f4_threshold}|F5's units within 4 s: largest whole-work total \
             {f5_threshold}{}|whole-work threshold T {threshold}|F3's whole-work total \
             {f3_total}|{}",
            if second_length_met {
                ""
            } else {
                " (not used: the second length missed)"
            },
            if threshold >= f3_total {
                "T >= F3's total: MET"
            } else {
                "T < F3's total: HOLD"
            }
        );

        // The statement-group fits and both corners (section 10).
        let f4_repair: Vec<Groups> = repairs[0]
            .steps(true)
            .into_iter()
            .filter(|sample| matches!(sample.class, "F4a" | "F4a2" | "F4b"))
            .map(Groups::of)
            .collect();
        let mut rest_rows: Vec<Groups> =
            from_empty.steps(true).into_iter().map(Groups::of).collect();
        rest_rows.extend(f7_groups());
        let fits = GroupFits {
            walk: fit(
                "walk group (F4 first repair pass)",
                &[
                    "per (month key x whole-work row)",
                    "per (work-month x whole-work row)",
                ],
                &f4_repair,
                |row| vec![row.walk_keys, row.walk_months],
                |row| row.walk,
                false,
            ),
            inserts: fit(
                "inserts group (F4 first repair pass)",
                &["per work-day row", "per year-term row"],
                &f4_repair,
                |row| vec![row.rows, row.year_term],
                |row| row.inserts,
                false,
            ),
            verify: fit(
                "verify group (F4 first repair pass)",
                &["per work-day row", "per whole-work row"],
                &f4_repair,
                |row| vec![row.rows, row.whole_rows],
                |row| row.verify,
                false,
            ),
            rest: fit(
                "rest group (from-empty replacing steps of F2, F3, F4 and F7)",
                &["intercept", "per statement", "per whole-work row"],
                &rest_rows,
                |row| vec![1.0, row.statements, row.whole_rows],
                |row| row.rest,
                false,
            ),
        };
        let measured_statements = rest_rows
            .iter()
            .chain(&f4_repair)
            .map(|row| row.statements)
            .chain(
                repaired
                    .steps(true)
                    .iter()
                    .map(|sample| sample.timing.in_transaction as f64),
            )
            .fold(0.0, f64::max);
        // A step at STEP_MAX_MONTH_KEYS with one pre-sizing and one re-sizing
        // statement: 6 unbudgeted statements and 2 per budgeted one.
        let chunks = |keys: i64| (keys + 49) / 50;
        let at_max_keys = (6 + 2
            * (15
                + 1
                + 1
                + 5 * chunks(METRIC_ROLLUP_RUN_STEP_MAX_MONTH_KEYS)
                + 2 * chunks(METRIC_ROLLUP_RUN_STEP_MAX_MONTH_KEYS)))
            as f64;
        let statements = measured_statements.max(at_max_keys);
        println!(
            "STATEMENTS|largest measured in a replacing step {measured_statements:.0}|a step at \
             STEP_MAX_MONTH_KEYS {at_max_keys:.0}|the corners use {statements:.0}"
        );
        let rows_budget = METRIC_ROLLUP_RUN_STEP_MAX_WORK_DAY_ROWS as f64;
        let figure = METRIC_ROLLUP_RUN_STEP_MAX_REPLACING_READ as f64;
        let t = threshold as f64;
        let series = figure / (rows_budget + t);
        let corner_a = corner(
            &fits,
            &format!("mix (a) corner at T = {threshold}"),
            rows_budget,
            t,
            series,
            series * t,
            12.0 * series * t,
            12.0 * t,
            statements,
        );
        let k_corner = figure / (2.0 * rows_budget);
        let corner_b = corner(
            &fits,
            "mix (b) corner (one year, the figure full at equal terms)",
            rows_budget,
            rows_budget,
            k_corner,
            k_corner * rows_budget,
            12.0 * k_corner * rows_budget,
            12.0 * rows_budget,
            statements,
        );
        println!(
            "ACCEPT|F4 statement-group prediction at both corners <= 4.5 s|{}",
            if corner_a && corner_b {
                "MET"
            } else {
                "MISSED"
            }
        );

        // The six-term pooled fit over every replacing step of F2 to F4,
        // information only (section 10).
        let pooled: Vec<Groups> = [&from_empty, &repairs[0], &repaired]
            .iter()
            .flat_map(|driven| driven.steps(true))
            .map(Groups::of)
            .collect();
        fit(
            "six-term pooled fit, F2 to F4 (information only)",
            &[
                "intercept",
                "per statement",
                "per work-day row",
                "per month-term row",
                "per year-term row",
                "per whole-work row",
            ],
            &pooled,
            |row| {
                vec![
                    1.0,
                    row.statements,
                    row.rows,
                    row.month_term,
                    row.year_term,
                    row.whole_rows,
                ]
            },
            |row| row.hold,
            true,
        );
    }

    // -----------------------------------------------------------------------
    // F6: concurrency
    // -----------------------------------------------------------------------

    /// The F6 workload's tallies.
    #[derive(Debug, Default)]
    struct Tally {
        ingested: usize,
        batches: usize,
        ingestion_failures: Vec<String>,
        claims: usize,
        completed: usize,
        completion_failures: Vec<String>,
    }

    /// One workload statement that waited on a lock, as the sampler saw it.
    #[derive(Clone, Debug)]
    struct Wait {
        application: String,
        work_row: bool,
        first: Instant,
        last: Instant,
        /// When the step's backend blocked it directly.
        step_direct: Option<(Instant, Instant)>,
        /// When the step's backend was anywhere in its blocking chain.
        step_chain: Option<(Instant, Instant)>,
    }

    #[derive(QueryableByName)]
    struct Activity {
        #[diesel(sql_type = Integer)]
        pid: i32,
        #[diesel(sql_type = Text)]
        application_name: String,
        #[diesel(sql_type = Nullable<Text>)]
        wait_event_type: Option<String>,
        #[diesel(sql_type = Nullable<Text>)]
        query: Option<String>,
        #[diesel(sql_type = Nullable<Text>)]
        query_start: Option<String>,
        #[diesel(sql_type = Array<Integer>)]
        blockers: Vec<i32>,
    }

    /// Whether `target` is in `pid`'s blocking chain.
    fn reaches(blockers: &BTreeMap<i32, Vec<i32>>, pid: i32, target: i32) -> bool {
        let mut seen = BTreeSet::new();
        let mut queue = vec![pid];
        while let Some(next) = queue.pop() {
            for blocker in blockers.get(&next).into_iter().flatten() {
                if *blocker == target {
                    return true;
                }
                if seen.insert(*blocker) {
                    queue.push(*blocker);
                }
            }
        }
        false
    }

    /// Sample `pg_blocking_pids()` of every workload backend until `stop`.
    fn sampler(stop: Arc<AtomicBool>) -> thread::JoinHandle<Vec<Wait>> {
        thread::spawn(move || {
            let mut connection =
                PgConnection::establish(&test_db_url()).expect("the sampler connection");
            let mut waits: BTreeMap<(i32, String), Wait> = BTreeMap::new();
            while !stop.load(Ordering::SeqCst) {
                let now = Instant::now();
                let rows: Vec<Activity> = sql_query(
                    "SELECT pid, application_name, wait_event_type, left(query, 300) AS query, \
                            query_start::text AS query_start, pg_blocking_pids(pid) AS blockers \
                     FROM pg_stat_activity WHERE application_name LIKE 'ops03-%'",
                )
                .load(&mut connection)
                .unwrap_or_default();
                let step = rows
                    .iter()
                    .find(|row| row.application_name == STEP_APPLICATION)
                    .map(|row| row.pid);
                let blockers: BTreeMap<i32, Vec<i32>> = rows
                    .iter()
                    .map(|row| (row.pid, row.blockers.clone()))
                    .collect();
                for row in &rows {
                    if row.application_name == STEP_APPLICATION
                        || row.wait_event_type.as_deref() != Some("Lock")
                    {
                        continue;
                    }
                    let query = row.query.clone().unwrap_or_default();
                    let wait = waits
                        .entry((row.pid, row.query_start.clone().unwrap_or_default()))
                        .or_insert_with(|| Wait {
                            application: row.application_name.clone(),
                            work_row: query.contains("FROM work") && query.contains("FOR UPDATE"),
                            first: now,
                            last: now,
                            step_direct: None,
                            step_chain: None,
                        });
                    wait.last = now;
                    if step.is_some_and(|step| row.blockers.contains(&step)) {
                        wait.step_direct.get_or_insert((now, now)).1 = now;
                    }
                    if step.is_some_and(|step| reaches(&blockers, row.pid, step)) {
                        wait.step_chain.get_or_insert((now, now)).1 = now;
                    }
                }
                thread::sleep(SAMPLE_INTERVAL.saturating_sub(now.elapsed()));
            }
            waits.into_values().collect()
        })
    }

    fn failure_text(error: &DieselError) -> String {
        match error {
            DieselError::DatabaseError(_, info) => {
                format!("{} ({})", info.message(), info.details().unwrap_or(""))
            }
            other => other.to_string(),
        }
    }

    /// The shared state of one F6 workload.
    struct Workload {
        corpus: Arc<Corpus>,
        sets: [Vec<Uuid>; 2],
        /// The set ingestion writes to: `current % 2`.
        current: AtomicUsize,
        /// The deltas the completion stream may still claim: the previous
        /// set's, all below any delta of the current set.
        allowed: AtomicI64,
        /// Held for reading by an ingestion batch, for writing by a switch.
        gate: RwLock<()>,
        stop: AtomicBool,
        tally: Mutex<Tally>,
    }

    /// One ingestion stream: canonical `DAY` batches of one or two works of
    /// the current set, the works locked `FOR UPDATE` first, under the
    /// ingestion's 30 s statement timeout.
    fn ingestion_stream(workload: &Workload, stream: usize) {
        let mut connection =
            PgConnection::establish(&test_db_url()).expect("an ingestion connection");
        sql_query(format!("SET application_name = 'ops03-ingest-{stream}'"))
            .execute(&mut connection)
            .expect("the ingestion's application name");
        let mut n = stream * 7_919;
        while !workload.stop.load(Ordering::SeqCst) {
            let gate = workload.gate.read().expect("the gate");
            let set = &workload.sets[workload.current.load(Ordering::SeqCst) % 2];
            let mut works = vec![set[n % set.len()], set[(n / 3 + 1) % set.len()]];
            works.sort();
            works.dedup();
            let on = day(2025, 12, 1 + u32::try_from(n % 28).expect("a day"));
            let value = 1 + i64::try_from(n % 5).expect("a value");
            let result = connection.transaction::<_, DieselError, _>(|c| {
                sql_query("SET LOCAL statement_timeout = 30000").execute(c)?;
                sql_query(format!(
                    "SELECT work_id FROM work WHERE work_id IN ({}) ORDER BY work_id FOR UPDATE",
                    quoted(&works)
                ))
                .execute(c)?;
                for work_id in &works {
                    for statement in ingestion_statements(&workload.corpus, *work_id, on, value) {
                        sql_query(statement).execute(c)?;
                    }
                }
                Ok(())
            });
            drop(gate);
            let mut tally = workload.tally.lock().expect("the tally");
            tally.batches += 1;
            match result {
                Ok(()) => tally.ingested += works.len(),
                Err(error) => tally.ingestion_failures.push(failure_text(&error)),
            }
            drop(tally);
            n += 1;
            thread::sleep(Duration::from_millis(25));
        }
    }

    /// The completion stream: claims and completions of the previous set's
    /// deltas only. A failed completion is counted and retried with its
    /// token, so its claim never blocks the frontier.
    fn completion_stream(workload: &Workload) {
        let pool = tagged_pool("ops03-completion", 1, None);
        let claimant = "metrics-evidence-completion";
        while !workload.stop.load(Ordering::SeqCst) {
            let limit = workload
                .allowed
                .load(Ordering::SeqCst)
                .min(i64::from(METRIC_ROLLUP_CLAIM_MAX_BATCH));
            if limit <= 0 {
                thread::sleep(Duration::from_millis(50));
                continue;
            }
            let claims = match claim_metric_rollup_deltas(
                &pool,
                claimant,
                i32::try_from(limit).expect("a claim limit"),
            ) {
                Ok(claims) => claims,
                Err(error) => {
                    workload
                        .tally
                        .lock()
                        .expect("the tally")
                        .completion_failures
                        .push(format!("claim: {error}"));
                    thread::sleep(Duration::from_millis(200));
                    continue;
                }
            };
            let Some(first) = claims.first() else {
                thread::sleep(Duration::from_millis(50));
                continue;
            };
            let claimed = i64::try_from(claims.len()).expect("a claim count");
            workload.allowed.fetch_sub(claimed, Ordering::SeqCst);
            for _ in 0..30 {
                match complete_metric_rollup_deltas(&pool, claimant, first.claim_token) {
                    Ok(_) => {
                        let mut tally = workload.tally.lock().expect("the tally");
                        tally.claims += 1;
                        tally.completed += claims.len();
                        break;
                    }
                    Err(error) => {
                        workload
                            .tally
                            .lock()
                            .expect("the tally")
                            .completion_failures
                            .push(format!("completion: {error}"));
                        thread::sleep(Duration::from_secs(1));
                    }
                }
            }
        }
    }

    /// Switch the sets every `phase`: wait until the completions have drained
    /// the previous set, close the gate on new batches, count the current
    /// set's deltas for the completion stream, and switch.
    fn coordinator(workload: &Workload, pool: &PgPool, phase: Duration) {
        while !workload.stop.load(Ordering::SeqCst) {
            let switch_at = Instant::now() + phase;
            while Instant::now() < switch_at && !workload.stop.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_millis(100));
            }
            let drained_by = Instant::now() + Duration::from_secs(120);
            while workload.allowed.load(Ordering::SeqCst) > 0
                && Instant::now() < drained_by
                && !workload.stop.load(Ordering::SeqCst)
            {
                thread::sleep(Duration::from_millis(100));
            }
            if workload.stop.load(Ordering::SeqCst) {
                break;
            }
            let _switch = workload.gate.write().expect("the gate");
            let pending = scalar_i64(
                pool,
                "(SELECT COUNT(*) FROM metric_rollup_delta WHERE status <> 'APPLIED')",
            );
            workload.allowed.store(pending, Ordering::SeqCst);
            workload.current.fetch_add(1, Ordering::SeqCst);
        }
    }

    /// Run the F6 workload while `body` runs; returns body's result, the
    /// tallies and the sampled waits.
    fn with_workload<T>(
        pool: &PgPool,
        corpus: &Arc<Corpus>,
        sets: [Vec<Uuid>; 2],
        body: impl FnOnce() -> T,
    ) -> (T, Tally, Vec<Wait>) {
        let workload = Workload {
            corpus: Arc::clone(corpus),
            sets,
            current: AtomicUsize::new(0),
            allowed: AtomicI64::new(0),
            gate: RwLock::new(()),
            stop: AtomicBool::new(false),
            tally: Mutex::new(Tally::default()),
        };
        let sampler_stop = Arc::new(AtomicBool::new(false));
        let sampling = sampler(Arc::clone(&sampler_stop));
        let phase = Duration::from_secs(
            u64::try_from(knob("THOTH_EVIDENCE_F6_PHASE_SECS", 20)).expect("a phase"),
        );
        let result = thread::scope(|scope| {
            for stream in 0..2 {
                let workload = &workload;
                scope.spawn(move || ingestion_stream(workload, stream));
            }
            scope.spawn(|| completion_stream(&workload));
            scope.spawn(|| coordinator(&workload, pool, phase));
            let result = body();
            workload.stop.store(true, Ordering::SeqCst);
            result
        });
        sampler_stop.store(true, Ordering::SeqCst);
        let waits = sampling.join().expect("the sampler");
        let tally = workload.tally.into_inner().expect("the tally");
        (result, tally, waits)
    }

    fn wait_spread(label: &str, waits: Vec<Duration>, bound: Option<Duration>) {
        let mut waits = waits;
        if waits.is_empty() {
            println!("WAITS|{label}|n=0");
            return;
        }
        let (p50, p95, max) = percentiles(&mut waits);
        let verdict = bound.map_or(String::new(), |bound| {
            format!(
                "|bound max <= {} s|{}",
                bound.as_secs(),
                if max <= bound.as_micros() {
                    "MET"
                } else {
                    "MISSED"
                }
            )
        });
        println!(
            "WAITS|{label}|n={}|p50 {} ms|p95 {} ms|max {} ms{verdict}",
            waits.len(),
            ms_of(p50),
            ms_of(p95),
            ms_of(max)
        );
    }

    fn report_workload(label: &str, tally: &Tally, waits: &[Wait], attributed: bool) {
        let span = |(first, last): (Instant, Instant)| last - first + SAMPLE_INTERVAL;
        let ingestion = |wait: &&Wait| wait.application.starts_with("ops03-ingest");
        println!(
            "WORKLOAD|{label}|ingestion batches {}|works ingested {}|ingestion failures {}|\
             claims {}|deltas completed {}|completion failures {}|failures {:?} {:?}",
            tally.batches,
            tally.ingested,
            tally.ingestion_failures.len(),
            tally.claims,
            tally.completed,
            tally.completion_failures.len(),
            tally.ingestion_failures,
            tally.completion_failures,
        );
        wait_spread(
            &format!("{label}|state-row waits, all causes (DAY inserts, claims, completions)"),
            waits
                .iter()
                .filter(|wait| !wait.work_row)
                .map(|wait| span((wait.first, wait.last)))
                .collect(),
            None,
        );
        wait_spread(
            &format!("{label}|ingestion work-row waits, all causes"),
            waits
                .iter()
                .filter(ingestion)
                .filter(|wait| wait.work_row)
                .map(|wait| span((wait.first, wait.last)))
                .collect(),
            None,
        );
        wait_spread(
            &format!("{label}|ingestion statement waits, all causes"),
            waits
                .iter()
                .filter(ingestion)
                .map(|wait| span((wait.first, wait.last)))
                .collect(),
            None,
        );
        if attributed {
            wait_spread(
                &format!("{label}|direct state-row waits attributable to steps"),
                waits
                    .iter()
                    .filter(|wait| !wait.work_row)
                    .filter_map(|wait| wait.step_direct.map(span))
                    .collect(),
                Some(Duration::from_secs(6)),
            );
            wait_spread(
                &format!("{label}|direct ingestion work-row waits attributable to steps"),
                waits
                    .iter()
                    .filter(ingestion)
                    .filter(|wait| wait.work_row)
                    .filter_map(|wait| wait.step_direct.map(span))
                    .collect(),
                Some(Duration::from_secs(8)),
            );
            wait_spread(
                &format!(
                    "{label}|ingestion statement waits with a step in the blocking chain \
                     (whole wait)"
                ),
                waits
                    .iter()
                    .filter(ingestion)
                    .filter(|wait| wait.step_chain.is_some())
                    .map(|wait| span((wait.first, wait.last)))
                    .collect(),
                Some(Duration::from_secs(13)),
            );
            wait_spread(
                &format!("{label}|ingestion statement waits, time with a step in the chain"),
                waits
                    .iter()
                    .filter(ingestion)
                    .filter_map(|wait| wait.step_chain.map(span))
                    .collect(),
                None,
            );
        }
    }

    /// F6 on the F2 corpus: the workload alone as a baseline, then during a
    /// paced REBUILD's build phase; the verify phase runs after the
    /// workload has stopped and its deltas are applied. F6's bounds are
    /// stated for the section 10 budgets, so F6 never uses the diagnostic
    /// budgets.
    fn f6(
        pool: &Arc<PgPool>,
        corpus: &Arc<Corpus>,
        evidence: &Evidence,
        books: &[Uuid],
        keys: i64,
    ) {
        let half = books.len() / 2;
        let sets = [books[..half].to_vec(), books[half..].to_vec()];
        let baseline = Duration::from_secs(
            u64::try_from(knob("THOTH_EVIDENCE_F6_BASELINE_SECS", 300)).expect("seconds"),
        );
        let ((), tally, waits) =
            with_workload(pool, corpus, sets.clone(), || thread::sleep(baseline));
        apply_everything(pool);
        report_workload("F6 baseline without a run", &tally, &waits, false);

        if knob("THOTH_EVIDENCE_F6_FROM_EMPTY", 1) != 0 {
            truncate_projections(pool);
        }
        let mut drive = Drive::new("F6 paced REBUILD build phase", &Classes::new());
        drive.diagnose = false;
        drive.paced = true;
        drive.build_only = true;
        drive.terms = false;
        let (built, tally, waits) = with_workload(pool, corpus, sets, || {
            drive_measured(pool, evidence, start(pool, Rebuild), &drive)
        });
        apply_everything(pool);
        print_raw(&built);
        summarise(&built, keys);
        report_workload("F6 during the paced build phase", &tally, &waits, true);
        let step_deadlocks = built
            .refusals
            .iter()
            .filter(|(refusal, _)| refusal.ends_with(reason::DEADLOCK))
            .map(|(_, count)| count)
            .sum::<usize>();
        let workload_deadlocks = tally
            .ingestion_failures
            .iter()
            .chain(&tally.completion_failures)
            .filter(|failure| failure.contains("deadlock"))
            .count();
        println!(
            "ACCEPT|F6 zero ingestion or completion failures|{} and {}|{}",
            tally.ingestion_failures.len(),
            tally.completion_failures.len(),
            if tally.ingestion_failures.is_empty() && tally.completion_failures.is_empty() {
                "MET"
            } else {
                "MISSED"
            }
        );
        println!(
            "ACCEPT|F6 no deadlock naming a step's backend|step refusals for a deadlock \
             {step_deadlocks}|workload deadlocks {workload_deadlocks}|{}",
            if step_deadlocks == 0 && workload_deadlocks == 0 {
                "MET"
            } else {
                "MISSED"
            }
        );
        println!(
            "ACCEPT|F6 the build phase completes|{:?}|refusals by reason {:?}|longest run of \
             refusals {}|build phase {} s|{}",
            built.run.status,
            built.refusals,
            built.longest_streak,
            secs(built.build),
            if built.stuck.is_none() && built.run.status != Building {
                "MET"
            } else {
                "MISSED"
            }
        );
        let mut verify = Drive::new("F6 verify phase after the workload", &Classes::new());
        verify.diagnose = false;
        verify.terms = false;
        let ended = drive_measured(pool, evidence, built.run.clone(), &verify);
        finish_pass(&ended, keys, Reconciled);
        whole_table(pool, "F6 after the verify phase");
    }
}
