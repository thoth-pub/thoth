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
    assert_eq!(METRIC_ROLLUP_RUN_STEP_MAX_WORK_DAY_ROWS, 50_000);
    assert_eq!(METRIC_ROLLUP_RUN_STEP_MAX_MONTH_KEYS, 1_000);
    assert_eq!(METRIC_ROLLUP_RUN_STEP_MAX_REPLACING_READ, 4_000_000);
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
