//! `MET-WP4-03C-B1` acceptance evidence for the derived coverage runs: the
//! exact migration/schema contract, the exhaustive B1 index set, the exact
//! trigger and function catalogue identity, revert/reapply, the incremental
//! producer (terminalization, provenance, winner ordering, splitting,
//! fragment preservation, maximal coalescing, OLD/NEW publisher and
//! terminal-identity changes, replay, failure atomicity, isolation, and
//! concurrency), the one-account rebuild, the independent verifier, the
//! identity manifests and the H2 planner evidence.
//!
//! Every test runs against the real schema in a disposable database. Raw
//! evidence is written through plain SQL so the trigger is exercised exactly
//! as PostgreSQL fires it; the real lifecycle path is exercised in
//! `metric_ingestion_lifecycle::tests`. Expected runs are stated explicitly
//! per case **and** checked against the independent verifier, so a test
//! never proves the producer by the producer.
//!
//! Test-only triggers, pause points and held sessions live only here, are
//! named uniquely and are removed on drop (the same discipline as the
//! ingestion and lifecycle suites). Nothing here is reachable from runtime
//! code.

use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use chrono::NaiveDate;
use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use diesel::{sql_query, Connection, QueryDsl, RunQueryDsl};
use diesel_migrations::MigrationHarness;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::crud::{
    rebuild_metric_coverage_runs, DELETE_ACCOUNT_RUNS_SQL, LOCK_SOURCE_ACCOUNT_SQL,
    REBUILD_ACCOUNT_RUNS_SQL, TRANSACTION_ISOLATION_SQL,
};
use super::verification::{
    bounded_count, domain_fingerprint, verify_metric_coverage_runs, verify_source_account,
    ACCOUNT_VERIFICATION_SQL, DOMAIN_SQL, TRANSACTION_MODE_SQL,
};
use super::{
    MetricCoverageRun, MetricCoverageRunAccountVerification, MetricCoverageRunMaintenanceError,
    MAINTENANCE_LOCK_TIMEOUT_SQL, MAINTENANCE_STATEMENT_TIMEOUT_SQL,
    METRIC_COVERAGE_RUN_DOMAIN_FINGERPRINT_SCHEMA,
};
use crate::db::{PgPool, MIGRATIONS};
use crate::model::metric_coverage::MetricCoverageStatus;
use crate::model::metric_import::MetricImportStatus;
use crate::model::metric_platform::tests::{scalar_i64, setup_registry_db};
use crate::model::metric_rollup_delta::tests::logging_pool;
use crate::model::tests::db::{test_db_url, TestDbGuard};
use crate::schema::metric_coverage_run;

/// The Diesel migration version of `thoth-api/migrations/20261006_v1.9.0`.
pub(crate) const MET_WP4_03C_B1_MIGRATION_VERSION: &str = "20261006";

const TRIGGER: &str = "metric_import_maintain_coverage_runs";
const FUNCTION: &str = "maintain_metric_coverage_runs_from_import";

use MetricCoverageStatus::{Complete, Partial, Unknown};
use MetricImportStatus::{Completed, CompletedWithErrors};

// ---------------------------------------------------------------------------
// Fixture
// ---------------------------------------------------------------------------

pub(crate) fn d(month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, month, day).expect("a valid 2026 date")
}

fn establish() -> PgConnection {
    PgConnection::establish(&test_db_url()).expect("Failed to connect to the test database")
}

fn exec(connection: &mut PgConnection, sql: &str) {
    sql_query(sql)
        .execute(connection)
        .unwrap_or_else(|error| panic!("{sql}: {error}"));
}

fn text(connection: &mut PgConnection, query: &str) -> String {
    diesel::select(diesel::dsl::sql::<diesel::sql_types::Text>(query))
        .get_result(connection)
        .unwrap_or_else(|error| panic!("{query}: {error}"))
}

/// One source with its platform, a second platform, two publishers, one
/// measure and two enabled accounts, on a pool wide enough for the
/// concurrency tests.
pub(crate) struct Fixture {
    pub(crate) pool: Arc<PgPool>,
    pub(crate) platform: Uuid,
    pub(crate) other_platform: Uuid,
    pub(crate) publisher_a: Uuid,
    pub(crate) publisher_b: Uuid,
    pub(crate) measure: Uuid,
    pub(crate) account: Uuid,
    pub(crate) account_b: Uuid,
}

/// A fresh registry database with the fixture rows.
pub(crate) fn setup() -> (TestDbGuard, Fixture) {
    let (guard, _registry_pool) = setup_registry_db();
    let pool = Arc::new(
        diesel::r2d2::Pool::builder()
            .max_size(8)
            .build(ConnectionManager::<PgConnection>::new(test_db_url()))
            .expect("Failed to create the coverage-run test pool"),
    );
    let fixture = Fixture::insert(pool);
    (guard, fixture)
}

impl Fixture {
    fn insert(pool: Arc<PgPool>) -> Self {
        let source = Uuid::new_v4();
        let platform = Uuid::new_v4();
        let other_platform = Uuid::new_v4();
        let publisher_a = Uuid::new_v4();
        let publisher_b = Uuid::new_v4();
        let account = Uuid::new_v4();
        let account_b = Uuid::new_v4();
        let mut c = pool.get().expect("Failed to get DB connection");
        exec(&mut c, &format!("INSERT INTO metric_source (source_id, code, acquisition_type, driver_key, enabled) VALUES ('{source}', 'b1-src', 'DRIVER', 'cloudfront', TRUE)"));
        exec(&mut c, &format!("INSERT INTO metric_platform (platform_id, code, display_name, ownership_class, enabled) VALUES ('{platform}', 'b1-p1', 'P1', 'THOTH_MANAGED', TRUE), ('{other_platform}', 'b1-p2', 'P2', 'THOTH_MANAGED', TRUE)"));
        exec(&mut c, &format!("INSERT INTO publisher (publisher_id, publisher_name) VALUES ('{publisher_a}', 'Publisher A'), ('{publisher_b}', 'Publisher B')"));
        exec(&mut c, &format!("INSERT INTO metric_source_account (source_account_id, code, source_id, platform_id, external_key, expected_publisher_id, configuration, enabled) VALUES ('{account}', 'b1-acct-a', '{source}', '{platform}', 'key-a', '{publisher_a}', '{{}}'::jsonb, TRUE), ('{account_b}', 'b1-acct-b', '{source}', '{platform}', 'key-b', '{publisher_a}', '{{}}'::jsonb, TRUE)"));
        let measure: Uuid = diesel::select(diesel::dsl::sql::<diesel::sql_types::Uuid>(
            "(SELECT measure_id FROM metric_measure WHERE code = 'title_sessions')",
        ))
        .get_result(&mut c)
        .expect("the seeded title_sessions measure");
        Fixture {
            pool,
            platform,
            other_platform,
            publisher_a,
            publisher_b,
            measure,
            account,
            account_b,
        }
    }

    pub(crate) fn sql(&self, sql: &str) {
        let mut c = self.pool.get().expect("Failed to get DB connection");
        exec(&mut c, sql);
    }

    fn count(&self, query: &str) -> i64 {
        scalar_i64(&self.pool, query)
    }

    /// Insert one `PROCESSING` import of `account` naming `publisher`.
    fn import_of(&self, account: Uuid, publisher: Option<Uuid>) -> Uuid {
        let import_id = Uuid::new_v4();
        let publisher = publisher.map_or("NULL".to_string(), |p| format!("'{p}'"));
        self.sql(&format!("INSERT INTO metric_import (import_id, source_account_id, publisher_id, format_code, format_version, status, normalizer_version, created_by) VALUES ('{import_id}', '{account}', {publisher}, 'f', '1', 'PROCESSING', 'n', 'b1-test')"));
        import_id
    }

    /// Insert one `PROCESSING` import of the fixture account naming
    /// publisher A.
    pub(crate) fn import(&self) -> Uuid {
        self.import_of(self.account, Some(self.publisher_a))
    }

    /// Insert one coverage row of `import` on `platform` for the fixture
    /// measure.
    #[allow(clippy::too_many_arguments)]
    fn coverage_on(
        &self,
        account: Uuid,
        import: Uuid,
        platform: Uuid,
        start: NaiveDate,
        end: NaiveDate,
        status: MetricCoverageStatus,
        country: bool,
        institution: bool,
    ) -> Uuid {
        let coverage_id = Uuid::new_v4();
        self.sql(&format!("INSERT INTO metric_coverage (coverage_id, source_account_id, import_id, platform_id, measure_id, period_start, period_end, coverage_status, country_coverage, institution_coverage) VALUES ('{coverage_id}', '{account}', '{import}', '{platform}', '{}', '{start}', '{end}', '{status}', {country}, {institution})", self.measure));
        coverage_id
    }

    /// Insert one coverage row of `import` on the account platform with no
    /// country or institution dimension.
    pub(crate) fn coverage(
        &self,
        import: Uuid,
        start: NaiveDate,
        end: NaiveDate,
        status: MetricCoverageStatus,
    ) -> Uuid {
        self.coverage_on(
            self.account,
            import,
            self.platform,
            start,
            end,
            status,
            false,
            false,
        )
    }

    /// Terminalize `import` exactly as the lifecycle does, with an explicit
    /// completion time so ordering is deterministic.
    fn terminalize_at(
        &self,
        import: Uuid,
        status: MetricImportStatus,
        completed_at: &str,
    ) -> Result<usize, diesel::result::Error> {
        let mut c = self.pool.get().expect("Failed to get DB connection");
        sql_query(format!("UPDATE metric_import SET status = '{status}', completed_at = '{completed_at}'::timestamptz WHERE import_id = '{import}'"))
            .execute(&mut c)
    }

    /// Terminalize `import` as `COMPLETED` at second `n` of a fixed minute.
    pub(crate) fn complete(&self, import: Uuid, n: u32) {
        self.terminalize_at(import, Completed, &at(n))
            .expect("terminalization must succeed");
    }

    /// Every run, ordered by stream and start.
    pub(crate) fn runs(&self) -> Vec<MetricCoverageRun> {
        let mut c = self.pool.get().expect("Failed to get DB connection");
        metric_coverage_run::table
            .order((
                metric_coverage_run::source_account_id,
                metric_coverage_run::platform_id,
                metric_coverage_run::publisher_id,
                metric_coverage_run::measure_id,
                metric_coverage_run::run_start,
            ))
            .load(&mut c)
            .expect("Failed to load coverage runs")
    }

    fn runs_of(&self, account: Uuid) -> Vec<MetricCoverageRun> {
        self.runs()
            .into_iter()
            .filter(|run| run.source_account_id == account)
            .collect()
    }

    /// The physical identity of every run row (`xmin` per primary key), so a
    /// test can prove a row was neither rewritten nor replaced.
    fn run_versions(&self) -> String {
        let mut c = self.pool.get().expect("Failed to get DB connection");
        text(&mut c, "(SELECT COALESCE(string_agg(concat_ws(':', source_account_id, platform_id, publisher_id, measure_id, run_start, xmin::text), ';' ORDER BY source_account_id, platform_id, publisher_id, measure_id, run_start), '') FROM metric_coverage_run)")
    }

    fn import_status(&self, import: Uuid) -> (String, Option<String>) {
        let mut c = self.pool.get().expect("Failed to get DB connection");
        let status = text(
            &mut c,
            &format!("(SELECT status::text FROM metric_import WHERE import_id = '{import}')"),
        );
        let completed = text(&mut c, &format!("(SELECT COALESCE(completed_at::text, '') FROM metric_import WHERE import_id = '{import}')"));
        (status, Some(completed).filter(|c| !c.is_empty()))
    }

    /// The independent verification of one account.
    pub(crate) fn verify(&self, account: Uuid) -> MetricCoverageRunAccountVerification {
        let mut c = self.pool.get().expect("Failed to get DB connection");
        verify_source_account(&mut c, account).expect("verification must succeed")
    }

    fn assert_exact(&self, account: Uuid, expected_rows: i32) {
        let verification = self.verify(account);
        assert!(
            verification.exact && verification.expected_rows == expected_rows,
            "account {account} must verify exact with {expected_rows} runs: {verification:?}"
        );
    }

    /// A run of the fixture account on its platform for publisher A.
    fn run(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        coverage: MetricCoverageStatus,
        import: MetricImportStatus,
    ) -> MetricCoverageRun {
        self.run_for(
            self.publisher_a,
            self.platform,
            start,
            end,
            coverage,
            import,
            false,
            false,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn run_for(
        &self,
        publisher: Uuid,
        platform: Uuid,
        start: NaiveDate,
        end: NaiveDate,
        coverage: MetricCoverageStatus,
        import: MetricImportStatus,
        country: bool,
        institution: bool,
    ) -> MetricCoverageRun {
        MetricCoverageRun {
            source_account_id: self.account,
            platform_id: platform,
            publisher_id: publisher,
            measure_id: self.measure,
            run_start: start,
            run_end: end,
            coverage_status: coverage,
            import_status: import,
            country_coverage: country,
            institution_coverage: institution,
        }
    }
}

/// `2026-04-01 00:00:nn+00`.
fn at(second: u32) -> String {
    format!("2026-04-01 00:00:{second:02}+00")
}

fn sorted(mut runs: Vec<MetricCoverageRun>) -> Vec<MetricCoverageRun> {
    runs.sort_by(|a, b| {
        (
            a.source_account_id,
            a.platform_id,
            a.publisher_id,
            a.measure_id,
            a.run_start,
        )
            .cmp(&(
                b.source_account_id,
                b.platform_id,
                b.publisher_id,
                b.measure_id,
                b.run_start,
            ))
    });
    runs
}

fn assert_runs(f: &Fixture, account: Uuid, expected: Vec<MetricCoverageRun>) {
    assert_eq!(f.runs_of(account), sorted(expected));
    let verification = f.verify(account);
    assert!(verification.exact, "{verification:?}");
    assert_eq!(
        verification.expected_rows as usize,
        f.runs_of(account).len()
    );
}

// ---------------------------------------------------------------------------
// Test-only triggers, pause points and held sessions
// ---------------------------------------------------------------------------

/// An ephemeral PL/pgSQL trigger installed in the disposable test database
/// and removed on drop.
struct TestTrigger {
    name: String,
    table: String,
}

impl TestTrigger {
    fn install(timing_and_event: &str, table: &str, body: &str) -> Self {
        let name = format!("b1_test_{}", Uuid::new_v4().simple());
        let mut c = establish();
        exec(&mut c, &format!("CREATE FUNCTION {name}() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN {body} END $$"));
        exec(&mut c, &format!("CREATE TRIGGER {name} {timing_and_event} ON {table} FOR EACH ROW EXECUTE FUNCTION {name}()"));
        TestTrigger {
            name,
            table: table.into(),
        }
    }

    /// Fails every insert into the coverage-run table.
    fn failing_run_insert() -> Self {
        Self::install(
            "BEFORE INSERT",
            "metric_coverage_run",
            "RAISE EXCEPTION 'thoth b1 test failure injection'; RETURN NULL;",
        )
    }

    /// Pauses every insert into the coverage-run table until the pause key
    /// is released.
    fn pausing_run_insert() -> Self {
        Self::install(
            "BEFORE INSERT",
            "metric_coverage_run",
            &format!("PERFORM pg_advisory_xact_lock({PAUSE_KEY}); RETURN NEW;"),
        )
    }

    /// Corrupts every inserted run by one day, so a replacement cannot be
    /// exact.
    fn corrupting_run_insert() -> Self {
        Self::install(
            "BEFORE INSERT",
            "metric_coverage_run",
            "NEW.run_end := NEW.run_end + 1; RETURN NEW;",
        )
    }
}

impl Drop for TestTrigger {
    fn drop(&mut self) {
        let mut c = establish();
        exec(
            &mut c,
            &format!("DROP TRIGGER IF EXISTS {} ON {}", self.name, self.table),
        );
        exec(&mut c, &format!("DROP FUNCTION IF EXISTS {}()", self.name));
    }
}

const PAUSE_KEY: i64 = 987_654_321_306;

/// Holds the pause key on a dedicated session so a pausing trigger blocks.
struct PauseHold {
    connection: PgConnection,
}

impl PauseHold {
    fn acquire() -> Self {
        let mut connection = establish();
        exec(
            &mut connection,
            &format!("SELECT pg_advisory_lock({PAUSE_KEY})"),
        );
        PauseHold { connection }
    }

    fn release(mut self) {
        exec(
            &mut self.connection,
            &format!("SELECT pg_advisory_unlock({PAUSE_KEY})"),
        );
    }
}

impl Drop for PauseHold {
    fn drop(&mut self) {
        let _ = sql_query("SELECT pg_advisory_unlock_all()").execute(&mut self.connection);
    }
}

fn backend_pid(connection: &mut PgConnection) -> i64 {
    diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>(
        "pg_backend_pid()",
    ))
    .get_result::<i32>(connection)
    .expect("pg_backend_pid") as i64
}

/// A one-connection pool, so the backend pid an operation will run on is
/// known before it runs.
fn pinned_pool() -> (Arc<PgPool>, i64) {
    let pool = Arc::new(
        diesel::r2d2::Pool::builder()
            .max_size(1)
            .build(ConnectionManager::<PgConnection>::new(test_db_url()))
            .expect("Failed to build a pinned pool"),
    );
    let pid = backend_pid(&mut pool.get().unwrap());
    (pool, pid)
}

/// Whether backend `pid` is waiting on a heavyweight lock, optionally of one
/// `wait_event` kind.
fn is_lock_waiting(pid: i64, wait_event: Option<&str>) -> bool {
    let mut monitor = establish();
    let event = wait_event
        .map(|event| format!(" AND wait_event = '{event}'"))
        .unwrap_or_default();
    diesel::select(diesel::dsl::sql::<diesel::sql_types::BigInt>(&format!(
        "(SELECT COUNT(*) FROM pg_stat_activity WHERE pid = {pid} AND wait_event_type = 'Lock'{event})"
    )))
    .get_result::<i64>(&mut monitor)
    .expect("pg_stat_activity")
        == 1
}

/// Block until backend `pid` is waiting on a heavyweight lock. The deadline
/// is a hang guard only; nothing is decided by elapsed time.
fn wait_until_blocked(pid: i64, wait_event: Option<&str>) {
    let started = Instant::now();
    while !is_lock_waiting(pid, wait_event) {
        assert!(
            started.elapsed() < Duration::from_secs(20),
            "backend {pid} never blocked on a lock"
        );
        thread::sleep(Duration::from_millis(15));
    }
}

/// A transaction on its own connection that runs `statements`, then waits
/// for `commit` before committing. The statements themselves may block.
struct HeldTransaction {
    release: Sender<()>,
    handle: JoinHandle<Result<(), diesel::result::Error>>,
}

impl HeldTransaction {
    fn start(open: &'static str, statements: Vec<String>) -> Self {
        let (release, wait): (Sender<()>, Receiver<()>) = channel();
        let (ready_tx, ready_rx) = channel();
        let handle = thread::spawn(move || {
            let mut connection = establish();
            sql_query(open).execute(&mut connection)?;
            for statement in &statements {
                if let Err(error) = sql_query(statement).execute(&mut connection) {
                    let _ = sql_query("ROLLBACK").execute(&mut connection);
                    return Err(error);
                }
            }
            ready_tx.send(()).unwrap();
            wait.recv().expect("release signal");
            sql_query("COMMIT").execute(&mut connection)?;
            Ok(())
        });
        // Wait until the statements have run (or the thread failed), so the
        // caller observes the held state.
        let _ = ready_rx.recv();
        HeldTransaction { release, handle }
    }

    fn commit(self) -> Result<(), diesel::result::Error> {
        let _ = self.release.send(());
        self.handle.join().unwrap()
    }
}

/// The SQL that terminalizes `import` as `COMPLETED` at `second`.
fn terminalize_sql(import: Uuid, second: u32) -> String {
    format!(
        "UPDATE metric_import SET status = 'COMPLETED', completed_at = '{}'::timestamptz WHERE import_id = '{import}'",
        at(second)
    )
}

// ---------------------------------------------------------------------------
// Catalogue helpers
// ---------------------------------------------------------------------------

fn index_definitions(connection: &mut PgConnection, table: &str) -> Vec<String> {
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Text)]
        indexdef: String,
    }
    sql_query(format!(
        "SELECT indexdef FROM pg_indexes WHERE schemaname = 'public' AND tablename = '{table}' ORDER BY indexname"
    ))
    .load::<Row>(connection)
    .expect("Failed to read index definitions")
    .into_iter()
    .map(|row| row.indexdef)
    .collect()
}

fn function_definition(connection: &mut PgConnection) -> String {
    text(
        connection,
        &format!("(SELECT pg_get_functiondef(p.oid) FROM pg_proc p JOIN pg_namespace n ON n.oid = p.pronamespace WHERE n.nspname = 'public' AND p.proname = '{FUNCTION}')"),
    )
}

fn trigger_definition(connection: &mut PgConnection) -> String {
    text(
        connection,
        &format!("(SELECT pg_get_triggerdef(oid) FROM pg_trigger WHERE tgname = '{TRIGGER}')"),
    )
}

fn public_function_names(connection: &mut PgConnection) -> String {
    text(connection, "(SELECT string_agg(p.proname, ',' ORDER BY p.proname) FROM pg_proc p JOIN pg_namespace n ON n.oid = p.pronamespace WHERE n.nspname = 'public')")
}

fn trigger_names(connection: &mut PgConnection) -> String {
    text(connection, "(SELECT string_agg(tgname || '@' || tgrelid::regclass::text, ',' ORDER BY tgname, tgrelid) FROM pg_trigger WHERE NOT tgisinternal)")
}

/// Revert migrations until the B1 migration itself has been reverted.
pub(crate) fn revert_through_b1_migration(connection: &mut PgConnection) {
    let applied = connection
        .applied_migrations()
        .expect("Failed to read applied migrations")
        .iter()
        .any(|version| version.to_string() == MET_WP4_03C_B1_MIGRATION_VERSION);
    assert!(
        applied,
        "the B1 migration must be applied before reverting through it"
    );
    loop {
        let reverted = connection
            .revert_last_migration(MIGRATIONS)
            .expect("Failed to revert migration");
        if reverted.to_string() == MET_WP4_03C_B1_MIGRATION_VERSION {
            return;
        }
    }
}

const EXPECTED_TRIGGER_DEFINITION: &str = "CREATE TRIGGER metric_import_maintain_coverage_runs AFTER UPDATE OF status, completed_at, publisher_id ON public.metric_import FOR EACH ROW WHEN ((((old.status IS DISTINCT FROM new.status) OR (old.completed_at IS DISTINCT FROM new.completed_at) OR (old.publisher_id IS DISTINCT FROM new.publisher_id)) AND (((old.status = ANY (ARRAY['COMPLETED'::metric_import_status, 'COMPLETED_WITH_ERRORS'::metric_import_status])) AND (old.completed_at IS NOT NULL) AND (old.publisher_id IS NOT NULL)) OR ((new.status = ANY (ARRAY['COMPLETED'::metric_import_status, 'COMPLETED_WITH_ERRORS'::metric_import_status])) AND (new.completed_at IS NOT NULL) AND (new.publisher_id IS NOT NULL))))) EXECUTE FUNCTION maintain_metric_coverage_runs_from_import()";

const EXPECTED_B1_INDEXES: [(&str, &str); 4] = [
    (
        "metric_coverage",
        "CREATE INDEX metric_coverage_import_id_idx ON public.metric_coverage USING btree (import_id)",
    ),
    (
        "metric_coverage",
        "CREATE INDEX metric_coverage_source_account_id_period_end_idx ON public.metric_coverage USING btree (source_account_id, period_end)",
    ),
    (
        "metric_import",
        "CREATE INDEX metric_import_terminal_without_completion_idx ON public.metric_import USING btree (source_account_id) WHERE ((status = ANY (ARRAY['COMPLETED'::metric_import_status, 'COMPLETED_WITH_ERRORS'::metric_import_status])) AND (completed_at IS NULL))",
    ),
    (
        "metric_record",
        "CREATE INDEX metric_record_native_grain_idx ON public.metric_record USING btree (work_id, platform_id, measure_id, period_start) WHERE (reporting_grain <> 'DAY'::metric_reporting_grain)",
    ),
];

// ===========================================================================
// Migration and schema
// ===========================================================================

#[test]
fn metric_coverage_run_has_exactly_the_approved_ten_non_null_columns() {
    let (_guard, pool) = setup_registry_db();
    let mut c = pool.get().unwrap();
    let columns = text(
        &mut c,
        "(SELECT string_agg(column_name || ':' || CASE WHEN data_type = 'USER-DEFINED' THEN udt_name ELSE data_type END || ':' || is_nullable || ':' || COALESCE(column_default, '-'), ',' ORDER BY ordinal_position) FROM information_schema.columns WHERE table_schema = 'public' AND table_name = 'metric_coverage_run')",
    );
    assert_eq!(
        columns,
        "source_account_id:uuid:NO:-,platform_id:uuid:NO:-,publisher_id:uuid:NO:-,\
         measure_id:uuid:NO:-,run_start:date:NO:-,run_end:date:NO:-,\
         coverage_status:metric_coverage_status:NO:-,import_status:metric_import_status:NO:-,\
         country_coverage:boolean:NO:-,institution_coverage:boolean:NO:-",
        "exactly the ten approved durable columns, all NOT NULL, none defaulted, no \
         surrogate id, winner identity, timestamp, generation or watermark"
    );
}

#[test]
fn metric_coverage_run_has_exactly_the_approved_key_check_and_foreign_keys() {
    let (_guard, pool) = setup_registry_db();
    let mut c = pool.get().unwrap();
    let constraints = text(
        &mut c,
        "(SELECT string_agg(conname || ':' || contype::text || ':' || pg_get_constraintdef(oid) || ':' || confdeltype::text || confupdtype::text, ';' ORDER BY contype, conname) FROM pg_constraint WHERE conrelid = 'public.metric_coverage_run'::regclass)",
    );
    assert_eq!(
        constraints,
        "metric_coverage_run_interval_check:c:CHECK ((run_end > run_start)):  ;\
         metric_coverage_run_measure_id_fkey:f:FOREIGN KEY (measure_id) REFERENCES metric_measure(measure_id):aa;\
         metric_coverage_run_platform_id_fkey:f:FOREIGN KEY (platform_id) REFERENCES metric_platform(platform_id):aa;\
         metric_coverage_run_publisher_id_fkey:f:FOREIGN KEY (publisher_id) REFERENCES publisher(publisher_id):aa;\
         metric_coverage_run_source_account_id_fkey:f:FOREIGN KEY (source_account_id) REFERENCES metric_source_account(source_account_id):aa;\
         metric_coverage_run_pkey:p:PRIMARY KEY (source_account_id, platform_id, publisher_id, measure_id, run_start):  ",
        "exactly one check, four plain NO ACTION foreign keys and the composite primary \
         key; no unique, exclusion or extension constraint"
    );
    assert_eq!(
        scalar_i64(
            &pool,
            "(SELECT COUNT(*) FROM pg_extension WHERE extname IN ('btree_gist', 'btree_gin'))"
        ),
        0,
        "no extension is introduced to enforce interval exclusion"
    );
}

#[test]
fn metric_coverage_run_carries_only_its_primary_key_index_and_the_four_b1_indexes_are_exact() {
    let (_guard, pool) = setup_registry_db();
    let mut c = pool.get().unwrap();
    assert_eq!(
        index_definitions(&mut c, "metric_coverage_run"),
        vec!["CREATE UNIQUE INDEX metric_coverage_run_pkey ON public.metric_coverage_run USING btree (source_account_id, platform_id, publisher_id, measure_id, run_start)"],
        "metric_coverage_run carries exactly its primary-key index"
    );
    for (table, definition) in EXPECTED_B1_INDEXES {
        let definitions = index_definitions(&mut c, table);
        assert!(
            definitions.iter().any(|actual| actual == definition),
            "{table} must carry exactly `{definition}`: {definitions:?}"
        );
    }
    // The exhaustive B1 permanent index set: nothing else on the three raw
    // tables beyond what the predecessors proved and these four.
    assert_eq!(
        index_definitions(&mut c, "metric_coverage").len(),
        3,
        "metric_coverage: primary key plus the two B1 indexes"
    );
    assert_eq!(
        index_definitions(&mut c, "metric_import").len(),
        5,
        "metric_import: primary key, two idempotency indexes, status/created_at and the B1 partial index"
    );
    assert_eq!(index_definitions(&mut c, "metric_record").len(), 8);
    // Every B1 index is valid and ready, built transactionally.
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM pg_index i JOIN pg_class c ON c.oid = i.indexrelid WHERE c.relname IN ('metric_coverage_import_id_idx', 'metric_coverage_source_account_id_period_end_idx', 'metric_import_terminal_without_completion_idx', 'metric_record_native_grain_idx', 'metric_coverage_run_pkey') AND i.indisvalid AND i.indisready AND i.indislive)"),
        5
    );
}

#[test]
fn the_h1_trigger_and_function_have_exactly_the_frozen_catalogue_identity() {
    let (_guard, pool) = setup_registry_db();
    let mut c = pool.get().unwrap();

    // Exactly one trigger of that name, on metric_import, enabled, AFTER UPDATE
    // OF exactly the three columns, FOR EACH ROW, calling exactly the function.
    assert_eq!(
        scalar_i64(
            &pool,
            &format!("(SELECT COUNT(*) FROM pg_trigger WHERE tgname = '{TRIGGER}')")
        ),
        1
    );
    let identity = text(
        &mut c,
        &format!("(SELECT tgrelid::regclass::text || '|' || tgenabled::text || '|' || tgtype::text || '|' || (SELECT string_agg(a.attname, ',' ORDER BY k.ordinality) FROM unnest(t.tgattr) WITH ORDINALITY AS k(attnum, ordinality) JOIN pg_attribute a ON a.attrelid = t.tgrelid AND a.attnum = k.attnum) || '|' || tgfoid::regproc::text FROM pg_trigger t WHERE tgname = '{TRIGGER}')"),
    );
    // tgtype 17 = ROW (1) + UPDATE (16); BEFORE (2) and INSERT/DELETE/TRUNCATE
    // bits are clear, so it is an AFTER UPDATE row trigger.
    assert_eq!(
        identity,
        format!("metric_import|O|17|status,completed_at,publisher_id|{FUNCTION}")
    );
    assert_eq!(trigger_definition(&mut c), EXPECTED_TRIGGER_DEFINITION);
    // The only non-internal trigger on metric_import.
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM pg_trigger WHERE tgrelid = 'public.metric_import'::regclass AND NOT tgisinternal)"),
        1
    );
    // Exactly one B1 function: a plpgsql trigger function of that name, and
    // no other coverage-run, maintenance or helper function anywhere.
    let functions = text(
        &mut c,
        "(SELECT string_agg(p.proname || ':' || p.prokind::text || ':' || l.lanname || ':' || p.prorettype::regtype::text, ',' ORDER BY p.proname) FROM pg_proc p JOIN pg_namespace n ON n.oid = p.pronamespace JOIN pg_language l ON l.oid = p.prolang WHERE n.nspname = 'public' AND (p.proname LIKE '%coverage_run%' OR p.proname LIKE 'maintain_%' OR p.proname LIKE '%coverage%'))",
    );
    assert_eq!(functions, format!("{FUNCTION}:f:plpgsql:trigger"));
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM pg_proc p JOIN pg_namespace n ON n.oid = p.pronamespace WHERE n.nspname = 'public' AND p.proname LIKE '%reconciliation%')"),
        0,
        "no B1 SQL object name contains `reconciliation`"
    );
    assert_eq!(
        scalar_i64(&pool, "(SELECT COUNT(*) FROM pg_trigger WHERE NOT tgisinternal AND tgname LIKE '%reconciliation%')"),
        0
    );
    // The function body: isolation check first, account FOR UPDATE, no
    // timeout or work_mem setting, no call to another B1 function.
    let body = function_definition(&mut c);
    for required in [
        "current_setting('transaction_isolation')",
        "'read committed'",
        "FROM public.metric_source_account sa",
        "FOR UPDATE",
        "FROM public.metric_coverage c",
        "JOIN public.metric_import mi",
        "completed_at DESC, e.import_id DESC",
        "coverage_status DESC, e.country_coverage ASC",
        "institution_coverage ASC, e.coverage_id DESC",
        "mi.publisher_id IS NOT NULL",
        "RETURN NULL",
    ] {
        assert!(
            body.contains(required),
            "the function must contain `{required}`:\n{body}"
        );
    }
    for forbidden in [
        "lock_timeout",
        "statement_timeout",
        "work_mem",
        "SET LOCAL",
        "expected_publisher_id",
        "sa.platform_id",
        "metric_rollup",
        "metricDashboard",
        "reconciliation",
    ] {
        assert!(
            !body.contains(forbidden),
            "the function must not contain `{forbidden}`:\n{body}"
        );
    }
    assert!(
        body.find("'read committed'").unwrap() < body.find("FOR UPDATE").unwrap(),
        "the isolation check precedes the account lock"
    );
}

#[test]
fn metric_coverage_run_rows_map_through_diesel_and_the_constraints_fail_closed() {
    let (_guard, f) = setup();
    f.sql(&format!("INSERT INTO metric_coverage_run VALUES ('{}', '{}', '{}', '{}', '2026-03-01', '2026-03-05', 'PARTIAL', 'COMPLETED_WITH_ERRORS', TRUE, FALSE)", f.account, f.platform, f.publisher_b, f.measure));
    assert_eq!(
        f.runs(),
        vec![f.run_for(
            f.publisher_b,
            f.platform,
            d(3, 1),
            d(3, 5),
            Partial,
            CompletedWithErrors,
            true,
            false
        )]
    );
    let mut c = f.pool.get().unwrap();
    for (label, sql) in [
        ("invalid interval", format!("INSERT INTO metric_coverage_run VALUES ('{}', '{}', '{}', '{}', '2026-03-05', '2026-03-05', 'COMPLETE', 'COMPLETED', FALSE, FALSE)", f.account, f.platform, f.publisher_a, f.measure)),
        ("duplicate stream and start", format!("INSERT INTO metric_coverage_run VALUES ('{}', '{}', '{}', '{}', '2026-03-01', '2026-03-09', 'COMPLETE', 'COMPLETED', FALSE, FALSE)", f.account, f.platform, f.publisher_b, f.measure)),
        ("unknown publisher", format!("INSERT INTO metric_coverage_run VALUES ('{}', '{}', '{}', '{}', '2026-05-01', '2026-05-02', 'COMPLETE', 'COMPLETED', FALSE, FALSE)", f.account, f.platform, Uuid::new_v4(), f.measure)),
        ("unknown account", format!("INSERT INTO metric_coverage_run VALUES ('{}', '{}', '{}', '{}', '2026-05-01', '2026-05-02', 'COMPLETE', 'COMPLETED', FALSE, FALSE)", Uuid::new_v4(), f.platform, f.publisher_a, f.measure)),
        ("null visible value", format!("INSERT INTO metric_coverage_run VALUES ('{}', '{}', '{}', '{}', '2026-05-01', '2026-05-02', NULL, 'COMPLETED', FALSE, FALSE)", f.account, f.platform, f.publisher_a, f.measure)),
    ] {
        assert!(sql_query(&sql).execute(&mut c).is_err(), "{label} must be rejected");
    }
    for referenced in [
        "publisher",
        "metric_platform",
        "metric_measure",
        "metric_source_account",
    ] {
        let column = match referenced {
            "publisher" => format!("publisher_id = '{}'", f.publisher_b),
            "metric_platform" => format!("platform_id = '{}'", f.platform),
            "metric_measure" => format!("measure_id = '{}'", f.measure),
            _ => format!("source_account_id = '{}'", f.account),
        };
        assert!(
            sql_query(format!("DELETE FROM {referenced} WHERE {column}"))
                .execute(&mut c)
                .is_err(),
            "deleting a referenced {referenced} must fail while a run names it"
        );
    }
}

#[test]
fn the_migration_backfills_nothing_over_populated_raw_evidence() {
    let (_guard, f) = setup();
    // Raw terminal evidence written while the B1 objects are absent: the
    // migration must create the table empty and the verifier must then
    // report that evidence as missing until a rebuild.
    let mut connection = establish();
    revert_through_b1_migration(&mut connection);
    assert_eq!(
        scalar_i64(
            &f.pool,
            "(SELECT COUNT(*) FROM pg_class WHERE relname = 'metric_coverage_run')"
        ),
        0
    );
    let import = f.import();
    f.coverage(import, d(3, 1), d(3, 11), Complete);
    let second = f.import();
    f.coverage(second, d(3, 5), d(3, 8), Partial);
    f.complete(import, 1);
    f.complete(second, 2);
    let raw_before = {
        let mut c = f.pool.get().unwrap();
        text(&mut c, "(SELECT md5(string_agg(c::text, ';' ORDER BY coverage_id)) || '|' || (SELECT md5(string_agg(i::text, ';' ORDER BY import_id)) FROM metric_import i) FROM metric_coverage c)")
    };

    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("Failed to reapply the B1 migration over populated evidence");

    assert_eq!(
        f.count("(SELECT COUNT(*) FROM metric_coverage_run)"),
        0,
        "no historical backfill"
    );
    let verification = f.verify(f.account);
    assert_eq!(
        (
            verification.expected_rows,
            verification.actual_rows,
            verification.missing_rows,
            verification.exact
        ),
        (3, 0, 3, false),
        "{verification:?}"
    );
    let raw_after = {
        let mut c = f.pool.get().unwrap();
        text(&mut c, "(SELECT md5(string_agg(c::text, ';' ORDER BY coverage_id)) || '|' || (SELECT md5(string_agg(i::text, ';' ORDER BY import_id)) FROM metric_import i) FROM metric_coverage c)")
    };
    assert_eq!(
        raw_before, raw_after,
        "canonical rows are untouched by the migration"
    );

    // A separately invoked rebuild populates the account exactly.
    let result = rebuild_metric_coverage_runs(&f.pool, f.account).expect("rebuild");
    assert!(result.rebuilt);
    assert_runs(
        &f,
        f.account,
        vec![
            f.run(d(3, 1), d(3, 5), Complete, Completed),
            f.run(d(3, 5), d(3, 8), Partial, Completed),
            f.run(d(3, 8), d(3, 11), Complete, Completed),
        ],
    );
}

#[test]
fn reverting_through_the_b1_migration_removes_exactly_its_objects_and_reapplication_restores_them()
{
    let (_guard, f) = setup();
    let import = f.import();
    f.coverage(import, d(3, 1), d(3, 4), Complete);
    f.complete(import, 1);
    assert_eq!(f.runs().len(), 1);

    let mut c = establish();
    let functions_before = public_function_names(&mut c);
    let triggers_before = trigger_names(&mut c);
    let function_before = function_definition(&mut c);
    let trigger_before = trigger_definition(&mut c);
    let indexes_before: Vec<Vec<String>> = ["metric_coverage", "metric_import", "metric_record"]
        .iter()
        .map(|table| index_definitions(&mut c, table))
        .collect();
    let raw_before = text(&mut c, "(SELECT md5(string_agg(c::text, ';' ORDER BY coverage_id)) || '|' || (SELECT md5(string_agg(i::text, ';' ORDER BY import_id)) FROM metric_import i) FROM metric_coverage c)");

    revert_through_b1_migration(&mut c);

    for (object, query) in [
        ("table", "(SELECT COUNT(*) FROM pg_class WHERE relname = 'metric_coverage_run')"),
        ("function", &format!("(SELECT COUNT(*) FROM pg_proc WHERE proname = '{FUNCTION}')")),
        ("trigger", &format!("(SELECT COUNT(*) FROM pg_trigger WHERE tgname = '{TRIGGER}')")),
        ("indexes", "(SELECT COUNT(*) FROM pg_indexes WHERE indexname IN ('metric_coverage_import_id_idx', 'metric_coverage_source_account_id_period_end_idx', 'metric_import_terminal_without_completion_idx', 'metric_record_native_grain_idx'))"),
    ] {
        assert_eq!(scalar_i64(&f.pool, query), 0, "the B1 {object} must be removed");
    }
    assert_eq!(
        public_function_names(&mut c),
        functions_before.replace(&format!("{FUNCTION},"), ""),
        "only the B1 function is removed"
    );
    assert_eq!(
        trigger_names(&mut c),
        triggers_before.replace(&format!("{TRIGGER}@metric_import,"), ""),
        "only the B1 trigger is removed"
    );
    assert_eq!(raw_before, text(&mut c, "(SELECT md5(string_agg(c::text, ';' ORDER BY coverage_id)) || '|' || (SELECT md5(string_agg(i::text, ';' ORDER BY import_id)) FROM metric_import i) FROM metric_coverage c)"), "raw evidence is untouched by the downgrade");
    // Without the trigger, a terminalization maintains nothing (and errors
    // nowhere): the import table is otherwise unchanged.
    let later = f.import();
    f.coverage(later, d(3, 4), d(3, 6), Complete);
    f.complete(later, 2);

    c.run_pending_migrations(MIGRATIONS)
        .expect("Failed to reapply the B1 migration");

    assert_eq!(
        function_definition(&mut c),
        function_before,
        "the function is recreated identically"
    );
    assert_eq!(
        trigger_definition(&mut c),
        trigger_before,
        "the trigger is recreated identically"
    );
    assert_eq!(public_function_names(&mut c), functions_before);
    assert_eq!(trigger_names(&mut c), triggers_before);
    for (table, before) in ["metric_coverage", "metric_import", "metric_record"]
        .iter()
        .zip(indexes_before)
    {
        assert_eq!(
            index_definitions(&mut c, table),
            before,
            "{table} indexes are recreated identically"
        );
    }
    assert_eq!(
        f.runs().len(),
        0,
        "the recreated table is empty: no backfill on reapply"
    );
    // The recreated trigger maintains runs again, over exactly the new
    // import's hull [6, 7): the earlier evidence outside that hull is not
    // backfilled by maintenance, and the verifier reports it missing until a
    // rebuild coalesces all three imports into one exact run.
    let third = f.import();
    f.coverage(third, d(3, 6), d(3, 7), Complete);
    f.complete(third, 3);
    assert_eq!(f.runs(), vec![f.run(d(3, 6), d(3, 7), Complete, Completed)]);
    let v = f.verify(f.account);
    assert_eq!(
        (
            v.expected_rows,
            v.actual_rows,
            v.missing_rows,
            v.extra_rows,
            v.exact
        ),
        (1, 1, 1, 1, false),
        "{v:?}"
    );
    assert!(
        rebuild_metric_coverage_runs(&f.pool, f.account)
            .unwrap()
            .rebuilt
    );
    assert_runs(
        &f,
        f.account,
        vec![f.run(d(3, 1), d(3, 7), Complete, Completed)],
    );
}

// ===========================================================================
// Incremental producer
// ===========================================================================

#[test]
fn terminalizing_a_processing_import_creates_its_runs_with_the_raw_terminal_status() {
    let (_guard, f) = setup();
    let completed = f.import();
    f.coverage(completed, d(3, 1), d(3, 11), Complete);
    let with_errors = f.import();
    f.coverage(with_errors, d(4, 1), d(4, 3), Complete);
    assert!(
        f.runs().is_empty(),
        "a PROCESSING import contributes nothing"
    );

    f.complete(completed, 1);
    f.terminalize_at(with_errors, CompletedWithErrors, &at(2))
        .unwrap();

    assert_runs(
        &f,
        f.account,
        vec![
            f.run(d(3, 1), d(3, 11), Complete, Completed),
            f.run(d(4, 1), d(4, 3), Complete, CompletedWithErrors),
        ],
    );
    // The other account is untouched and exact at zero.
    f.assert_exact(f.account_b, 0);
}

#[test]
fn an_import_without_coverage_rows_changes_no_run() {
    let (_guard, f) = setup();
    let import = f.import();
    f.coverage(import, d(3, 1), d(3, 11), Complete);
    f.complete(import, 1);
    let versions = f.run_versions();

    let empty = f.import();
    f.complete(empty, 2);
    assert_eq!(
        f.run_versions(),
        versions,
        "no run is rewritten or replaced"
    );
    f.assert_exact(f.account, 1);
}

#[test]
fn historical_provenance_is_the_coverage_platform_and_the_import_publisher_not_the_account() {
    let (_guard, f) = setup();
    // The account is configured for platform P1 / publisher A, but the raw
    // evidence names publisher B on platform P2 and publisher A on P1.
    let foreign = f.import_of(f.account, Some(f.publisher_b));
    f.coverage_on(
        f.account,
        foreign,
        f.other_platform,
        d(3, 1),
        d(3, 4),
        Complete,
        true,
        true,
    );
    let own = f.import();
    f.coverage(own, d(3, 1), d(3, 4), Partial);
    f.complete(foreign, 1);
    f.complete(own, 2);

    let expected = vec![
        f.run_for(
            f.publisher_b,
            f.other_platform,
            d(3, 1),
            d(3, 4),
            Complete,
            Completed,
            true,
            true,
        ),
        f.run(d(3, 1), d(3, 4), Partial, Completed),
    ];
    assert_runs(&f, f.account, expected.clone());
    let versions = f.run_versions();

    // Current configuration writers maintain no run: re-pointing the account
    // at platform P2, publisher B, and disabling it, changes nothing.
    f.sql(&format!("UPDATE metric_source_account SET platform_id = '{}', expected_publisher_id = '{}', enabled = FALSE WHERE source_account_id = '{}'", f.other_platform, f.publisher_b, f.account));
    assert_eq!(f.run_versions(), versions);
    assert_runs(&f, f.account, expected.clone());
    f.sql(&format!("UPDATE metric_source_account SET expected_publisher_id = NULL WHERE source_account_id = '{}'", f.account));
    assert_eq!(f.run_versions(), versions);
    assert_runs(&f, f.account, expected);
}

#[test]
fn the_winner_ordering_is_exact_including_every_tie_component() {
    let (_guard, f) = setup();
    // completed_at DESC decides first: the later completion wins regardless
    // of import id.
    let early = f.import();
    let late = f.import();
    f.coverage(early, d(3, 1), d(3, 3), Complete);
    f.coverage(late, d(3, 1), d(3, 3), Partial);
    f.complete(late, 2);
    f.complete(early, 1);
    assert_runs(
        &f,
        f.account,
        vec![f.run(d(3, 1), d(3, 3), Partial, Completed)],
    );

    // Equal completed_at: import_id DESC.
    let low = Uuid::parse_str("40000000-0000-4000-8000-000000000001").unwrap();
    let high = Uuid::parse_str("40000000-0000-4000-8000-000000000002").unwrap();
    for id in [low, high] {
        f.sql(&format!("INSERT INTO metric_import (import_id, source_account_id, publisher_id, format_code, format_version, status, normalizer_version, created_by) VALUES ('{id}', '{}', '{}', 'f', '1', 'PROCESSING', 'n', 'b1-test')", f.account, f.publisher_a));
    }
    f.coverage(low, d(5, 1), d(5, 4), Complete);
    f.coverage(high, d(5, 1), d(5, 4), Partial);
    // Within the higher import, two rows cover 2 May: coverage_status DESC
    // (UNKNOWN beats PARTIAL), then country_coverage ASC and
    // institution_coverage ASC prefer the row without the dimension.
    f.coverage_on(
        f.account,
        high,
        f.platform,
        d(5, 2),
        d(5, 3),
        Unknown,
        true,
        true,
    );
    f.coverage_on(
        f.account,
        high,
        f.platform,
        d(5, 2),
        d(5, 3),
        Unknown,
        false,
        true,
    );
    f.coverage_on(
        f.account,
        high,
        f.platform,
        d(5, 3),
        d(5, 4),
        Unknown,
        true,
        false,
    );
    f.coverage_on(
        f.account,
        high,
        f.platform,
        d(5, 3),
        d(5, 4),
        Unknown,
        true,
        true,
    );
    f.complete(low, 9);
    f.complete(high, 9);
    let mut expected = vec![
        f.run(d(3, 1), d(3, 3), Partial, Completed),
        f.run(d(5, 1), d(5, 2), Partial, Completed),
        f.run_for(
            f.publisher_a,
            f.platform,
            d(5, 2),
            d(5, 3),
            Unknown,
            Completed,
            false,
            true,
        ),
        f.run_for(
            f.publisher_a,
            f.platform,
            d(5, 3),
            d(5, 4),
            Unknown,
            Completed,
            true,
            false,
        ),
    ];
    assert_runs(&f, f.account, expected.clone());

    // Two rows equal in every ordering key but coverage_id have an equal
    // visible tuple, so the coverage_id tie-break is unobservable and they
    // produce exactly one run.
    let twin = f.import();
    f.coverage(twin, d(6, 1), d(6, 3), Complete);
    f.coverage(twin, d(6, 1), d(6, 3), Complete);
    f.complete(twin, 10);
    expected.push(f.run(d(6, 1), d(6, 3), Complete, Completed));
    assert_runs(&f, f.account, expected);
}

#[test]
fn a_newer_assertion_splits_runs_and_an_equal_newest_one_recoalesces_them() {
    let (_guard, f) = setup();
    let base = f.import();
    f.coverage(base, d(3, 1), d(3, 11), Complete);
    f.complete(base, 1);
    // An unaffected run far away, to prove outer rows are never touched.
    let distant = f.import();
    f.coverage(distant, d(9, 1), d(9, 3), Partial);
    f.complete(distant, 1);
    let distant_version = f.run_versions();

    // A later partial assertion over the interior splits the run in three.
    let partial = f.import();
    f.coverage(partial, d(3, 5), d(3, 8), Partial);
    f.complete(partial, 2);
    assert_runs(
        &f,
        f.account,
        vec![
            f.run(d(3, 1), d(3, 5), Complete, Completed),
            f.run(d(3, 5), d(3, 8), Partial, Completed),
            f.run(d(3, 8), d(3, 11), Complete, Completed),
            f.run(d(9, 1), d(9, 3), Partial, Completed),
        ],
    );
    assert!(
        f.run_versions()
            .contains(&distant_version[distant_version.rfind(';').unwrap() + 1..]),
        "the distant run keeps its physical row"
    );

    // An even later complete assertion over the same interior restores one
    // maximal run: the outer fragments and the recomputed interior coalesce.
    let complete = f.import();
    f.coverage(complete, d(3, 5), d(3, 8), Complete);
    f.complete(complete, 3);
    assert_runs(
        &f,
        f.account,
        vec![
            f.run(d(3, 1), d(3, 11), Complete, Completed),
            f.run(d(9, 1), d(9, 3), Partial, Completed),
        ],
    );
}

#[test]
fn equal_outer_fragments_coalesce_across_both_hull_boundaries() {
    let (_guard, f) = setup();
    // Existing runs that end exactly at the new hull's start and begin
    // exactly at its end, both with the tuple the new import asserts.
    let left = f.import();
    f.coverage(left, d(3, 1), d(3, 5), Complete);
    let right = f.import();
    f.coverage(right, d(3, 8), d(3, 12), Complete);
    f.complete(left, 1);
    f.complete(right, 1);
    assert_eq!(f.runs().len(), 2);

    let middle = f.import();
    f.coverage(middle, d(3, 5), d(3, 8), Complete);
    f.complete(middle, 2);
    assert_runs(
        &f,
        f.account,
        vec![f.run(d(3, 1), d(3, 12), Complete, Completed)],
    );

    // The mirror: a hull whose first recomputed run differs from the touching
    // left run but whose last run equals the touching right run.
    let left2 = f.import();
    f.coverage(left2, d(5, 1), d(5, 3), Partial);
    let right2 = f.import();
    f.coverage(right2, d(5, 6), d(5, 9), Complete);
    f.complete(left2, 3);
    f.complete(right2, 3);
    let middle2 = f.import();
    f.coverage(middle2, d(5, 3), d(5, 4), Partial);
    f.coverage(middle2, d(5, 4), d(5, 6), Complete);
    f.complete(middle2, 4);
    assert_runs(
        &f,
        f.account,
        vec![
            f.run(d(3, 1), d(3, 12), Complete, Completed),
            f.run(d(5, 1), d(5, 4), Partial, Completed),
            f.run(d(5, 4), d(5, 9), Complete, Completed),
        ],
    );
}

#[test]
fn a_raw_winner_change_with_an_equal_visible_tuple_coalesces_into_one_run() {
    let (_guard, f) = setup();
    let first = f.import();
    f.coverage(first, d(3, 1), d(3, 11), Complete);
    f.complete(first, 1);
    let newer = f.import();
    f.coverage(newer, d(3, 4), d(3, 7), Complete);
    f.complete(newer, 2);
    // The winner of 4-6 March changed to the newer import, but every visible
    // value is equal, so there is still exactly one run.
    assert_runs(
        &f,
        f.account,
        vec![f.run(d(3, 1), d(3, 11), Complete, Completed)],
    );
    let newest = f.import_of(f.account, Some(f.publisher_a));
    f.coverage(newest, d(2, 20), d(3, 20), Complete);
    f.complete(newest, 3);
    assert_runs(
        &f,
        f.account,
        vec![f.run(d(2, 20), d(3, 20), Complete, Completed)],
    );
}

#[test]
fn a_publisher_change_on_a_terminal_import_reconciles_old_and_new_publisher_effects() {
    let (_guard, f) = setup();
    let base = f.import();
    f.coverage(base, d(3, 1), d(3, 11), Complete);
    let partial = f.import();
    f.coverage(partial, d(3, 5), d(3, 8), Partial);
    let newest = f.import();
    f.coverage(newest, d(3, 5), d(3, 8), Complete);
    f.complete(base, 1);
    f.complete(partial, 2);
    f.complete(newest, 3);
    assert_runs(
        &f,
        f.account,
        vec![f.run(d(3, 1), d(3, 11), Complete, Completed)],
    );

    // Re-homing the newest import to publisher B removes its effect from
    // publisher A's stream (the partial assertion wins 5-7 March again) and
    // creates publisher B's stream.
    f.sql(&format!(
        "UPDATE metric_import SET publisher_id = '{}' WHERE import_id = '{newest}'",
        f.publisher_b
    ));
    assert_runs(
        &f,
        f.account,
        vec![
            f.run(d(3, 1), d(3, 5), Complete, Completed),
            f.run(d(3, 5), d(3, 8), Partial, Completed),
            f.run(d(3, 8), d(3, 11), Complete, Completed),
            f.run_for(
                f.publisher_b,
                f.platform,
                d(3, 5),
                d(3, 8),
                Complete,
                Completed,
                false,
                false,
            ),
        ],
    );
    // And back again.
    f.sql(&format!(
        "UPDATE metric_import SET publisher_id = '{}' WHERE import_id = '{newest}'",
        f.publisher_a
    ));
    assert_runs(
        &f,
        f.account,
        vec![f.run(d(3, 1), d(3, 11), Complete, Completed)],
    );
    // A terminal import whose publisher becomes NULL contributes nothing.
    f.sql(&format!(
        "UPDATE metric_import SET publisher_id = NULL WHERE import_id = '{newest}'"
    ));
    assert_runs(
        &f,
        f.account,
        vec![
            f.run(d(3, 1), d(3, 5), Complete, Completed),
            f.run(d(3, 5), d(3, 8), Partial, Completed),
            f.run(d(3, 8), d(3, 11), Complete, Completed),
        ],
    );
}

#[test]
fn terminal_status_and_completed_at_changes_reconcile_removal_and_reranking() {
    let (_guard, f) = setup();
    let base = f.import();
    f.coverage(base, d(3, 1), d(3, 11), Complete);
    let partial = f.import();
    f.coverage(partial, d(3, 5), d(3, 8), Partial);
    f.complete(base, 1);
    f.complete(partial, 2);
    assert_eq!(f.runs().len(), 3);

    // Terminal -> non-participating removes the partial assertion.
    f.sql(&format!(
        "UPDATE metric_import SET status = 'FAILED' WHERE import_id = '{partial}'"
    ));
    assert_runs(
        &f,
        f.account,
        vec![f.run(d(3, 1), d(3, 11), Complete, Completed)],
    );
    assert_eq!(f.import_status(partial).0, "FAILED");

    // Terminal -> terminal with a cleared completed_at also removes it.
    f.sql(&format!(
        "UPDATE metric_import SET status = 'COMPLETED' WHERE import_id = '{partial}'"
    ));
    assert_eq!(
        f.runs().len(),
        3,
        "restored as terminal, it participates again"
    );
    f.sql(&format!(
        "UPDATE metric_import SET completed_at = NULL WHERE import_id = '{partial}'"
    ));
    assert_runs(
        &f,
        f.account,
        vec![f.run(d(3, 1), d(3, 11), Complete, Completed)],
    );

    // A completed_at re-ranking: the base import completes later than the
    // partial one and wins the whole range.
    f.sql(&format!(
        "UPDATE metric_import SET completed_at = '{}'::timestamptz WHERE import_id = '{partial}'",
        at(2)
    ));
    assert_eq!(f.runs().len(), 3);
    f.sql(&format!(
        "UPDATE metric_import SET completed_at = '{}'::timestamptz WHERE import_id = '{base}'",
        at(5)
    ));
    assert_runs(
        &f,
        f.account,
        vec![f.run(d(3, 1), d(3, 11), Complete, Completed)],
    );
    // COMPLETED -> COMPLETED_WITH_ERRORS changes the visible import status.
    f.sql(&format!(
        "UPDATE metric_import SET status = 'COMPLETED_WITH_ERRORS' WHERE import_id = '{base}'"
    ));
    assert_runs(
        &f,
        f.account,
        vec![f.run(d(3, 1), d(3, 11), Complete, CompletedWithErrors)],
    );
}

#[test]
fn a_terminal_import_with_a_null_publisher_contributes_no_run_and_no_expected_evidence() {
    let (_guard, f) = setup();
    let orphan = f.import_of(f.account, None);
    f.coverage(orphan, d(3, 1), d(3, 11), Complete);
    f.complete(orphan, 1);
    assert!(f.runs().is_empty());
    f.assert_exact(f.account, 0);
    // Giving it a publisher later makes it participate.
    f.sql(&format!(
        "UPDATE metric_import SET publisher_id = '{}' WHERE import_id = '{orphan}'",
        f.publisher_a
    ));
    assert_runs(
        &f,
        f.account,
        vec![f.run(d(3, 1), d(3, 11), Complete, Completed)],
    );
}

#[test]
fn an_ordinary_counter_update_on_a_processing_import_does_not_invoke_maintenance_or_take_the_account_lock(
) {
    let (_guard, f) = setup();
    let terminal = f.import();
    f.coverage(terminal, d(3, 1), d(3, 11), Complete);
    f.complete(terminal, 1);
    let processing = f.import();
    f.coverage(processing, d(3, 3), d(3, 6), Partial);

    // Corrupt the derived state inside the processing import's hull. A
    // maintenance pass over that hull would repair it.
    f.sql("DELETE FROM metric_coverage_run");
    assert!(!f.verify(f.account).exact);

    // The canonical ingestion counter update touches none of the three
    // trigger columns: the corruption survives, so no maintenance ran.
    f.sql(&format!("UPDATE metric_import SET received_count = received_count + 7, accepted_count = accepted_count + 7 WHERE import_id = '{processing}'"));
    assert!(f.runs().is_empty(), "no maintenance pass repaired the hull");
    assert!(!f.verify(f.account).exact);

    // And it waits on no account lock: with the account row held FOR UPDATE
    // elsewhere, the counter update succeeds immediately while a
    // terminalization of the same account is refused by its lock timeout.
    let hold = HeldTransaction::start(
        "BEGIN",
        vec![format!(
            "SELECT 1 FROM metric_source_account WHERE source_account_id = '{}' FOR UPDATE",
            f.account
        )],
    );
    let mut c = f.pool.get().unwrap();
    c.transaction::<_, diesel::result::Error, _>(|c| {
        sql_query("SET LOCAL lock_timeout = '500ms'").execute(c)?;
        sql_query(format!("UPDATE metric_import SET received_count = received_count + 1 WHERE import_id = '{processing}'")).execute(c)
    })
    .expect("the counter update must not wait on the account row");
    let refused = c.transaction::<_, diesel::result::Error, _>(|c| {
        sql_query("SET LOCAL lock_timeout = '500ms'").execute(c)?;
        sql_query(terminalize_sql(processing, 2)).execute(c)
    });
    assert!(
        refused.unwrap_err().to_string().contains("lock timeout"),
        "the terminalization takes the account row lock"
    );
    assert_eq!(
        f.import_status(processing).0,
        "PROCESSING",
        "the refused terminalization rolled back"
    );
    hold.commit().unwrap();

    // Once terminalized, exactly the import's hull [3, 6) is recomputed from
    // all raw evidence; the deleted runs outside that hull are not the
    // incremental producer's to repair, so the account stays inexact until
    // a rebuild, which restores all three runs.
    f.complete(processing, 2);
    assert_eq!(f.runs(), vec![f.run(d(3, 3), d(3, 6), Partial, Completed)]);
    let v = f.verify(f.account);
    assert_eq!(
        (v.expected_rows, v.actual_rows, v.missing_rows, v.exact),
        (3, 1, 2, false),
        "{v:?}"
    );
    assert!(
        rebuild_metric_coverage_runs(&f.pool, f.account)
            .unwrap()
            .rebuilt
    );
    assert_runs(
        &f,
        f.account,
        vec![
            f.run(d(3, 1), d(3, 3), Complete, Completed),
            f.run(d(3, 3), d(3, 6), Partial, Completed),
            f.run(d(3, 6), d(3, 11), Complete, Completed),
        ],
    );
}

#[test]
fn a_committed_terminalization_replay_performs_no_run_write() {
    let (_guard, f) = setup();
    let import = f.import();
    f.coverage(import, d(3, 1), d(3, 11), Complete);
    f.complete(import, 1);
    let versions = f.run_versions();
    // The same terminal values again: the WHEN gate sees no change, so the
    // function never runs and no run row is rewritten.
    f.complete(import, 1);
    f.sql(&format!("UPDATE metric_import SET status = 'COMPLETED', completed_at = completed_at, publisher_id = publisher_id WHERE import_id = '{import}'"));
    assert_eq!(f.run_versions(), versions);
    f.assert_exact(f.account, 1);
}

#[test]
fn a_failure_inside_maintenance_rolls_back_the_terminalization_and_the_runs_together() {
    let (_guard, f) = setup();
    let base = f.import();
    f.coverage(base, d(3, 1), d(3, 11), Complete);
    f.complete(base, 1);
    let versions = f.run_versions();
    let partial = f.import();
    f.coverage(partial, d(3, 5), d(3, 8), Partial);

    let failing = TestTrigger::failing_run_insert();
    let error = f.terminalize_at(partial, Completed, &at(2)).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("thoth b1 test failure injection"),
        "{error}"
    );
    assert_eq!(
        f.import_status(partial),
        ("PROCESSING".to_string(), None),
        "the terminal update rolled back"
    );
    assert_eq!(
        f.run_versions(),
        versions,
        "the displaced run is back, physically unchanged"
    );
    drop(failing);

    // Retry after the rolled-back failure recomputes from raw state.
    f.complete(partial, 2);
    assert_eq!(f.runs().len(), 3);
    f.assert_exact(f.account, 3);
}

#[test]
fn maintenance_fails_closed_under_an_unsupported_isolation_level_and_rolls_back_the_import() {
    let (_guard, f) = setup();
    let base = f.import();
    f.coverage(base, d(3, 1), d(3, 11), Complete);
    f.complete(base, 1);
    let versions = f.run_versions();
    let partial = f.import();
    f.coverage(partial, d(3, 5), d(3, 8), Partial);

    for level in ["REPEATABLE READ", "SERIALIZABLE"] {
        let mut c = establish();
        exec(&mut c, &format!("BEGIN ISOLATION LEVEL {level}"));
        let error = sql_query(terminalize_sql(partial, 2))
            .execute(&mut c)
            .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("requires READ COMMITTED isolation"),
            "{level}: {error}"
        );
        exec(&mut c, "ROLLBACK");
        assert_eq!(
            f.import_status(partial),
            ("PROCESSING".to_string(), None),
            "{level}: the import is still non-terminal"
        );
        assert_eq!(f.run_versions(), versions, "{level}: no run changed");
    }
    // The failure is the trigger's own, inside the UPDATE statement, so the
    // whole transaction is aborted: a COMMIT after it commits nothing.
    let mut c = establish();
    exec(&mut c, "BEGIN ISOLATION LEVEL REPEATABLE READ");
    assert!(sql_query(terminalize_sql(partial, 2))
        .execute(&mut c)
        .is_err());
    assert!(
        sql_query("SELECT 1").execute(&mut c).is_err(),
        "the transaction is aborted"
    );
    exec(&mut c, "ROLLBACK");
    assert_eq!(f.import_status(partial).0, "PROCESSING");

    // READ COMMITTED is the supported level.
    f.complete(partial, 2);
    f.assert_exact(f.account, 3);
}

#[test]
fn concurrent_same_account_terminalizations_serialize_on_the_account_row_without_lost_update() {
    let (_guard, f) = setup();
    let first = f.import();
    f.coverage(first, d(3, 1), d(3, 11), Complete);
    let second = f.import();
    f.coverage(second, d(3, 5), d(3, 15), Partial);

    // Freeze the first terminalization inside its maintenance pass, after it
    // has taken the account lock and while it inserts its runs.
    let hold = PauseHold::acquire();
    let pausing = TestTrigger::pausing_run_insert();
    let (pool_a, pid_a) = pinned_pool();
    let a = thread::spawn(move || {
        let mut c = pool_a.get().unwrap();
        sql_query(terminalize_sql(first, 1)).execute(&mut c)
    });
    wait_until_blocked(pid_a, Some("advisory"));

    // The second terminalization of the same account queues on the account
    // row lock rather than interleaving.
    let (pool_b, pid_b) = pinned_pool();
    let b = thread::spawn(move || {
        let mut c = pool_b.get().unwrap();
        sql_query(terminalize_sql(second, 2)).execute(&mut c)
    });
    wait_until_blocked(pid_b, None);
    assert!(
        is_lock_waiting(pid_b, Some("tuple")) || is_lock_waiting(pid_b, Some("transactionid")),
        "the second terminalization waits on the account row held by the first"
    );
    assert!(!b.is_finished());

    hold.release();
    a.join().unwrap().expect("first terminalization");
    b.join().unwrap().expect("second terminalization");
    drop(pausing);

    assert_runs(
        &f,
        f.account,
        vec![
            f.run(d(3, 1), d(3, 5), Complete, Completed),
            f.run(d(3, 5), d(3, 15), Partial, Completed),
        ],
    );
}

#[test]
fn a_later_committing_transaction_with_the_earlier_transaction_timestamp_stays_exact() {
    let (_guard, f) = setup();
    let early_import = f.import();
    f.coverage(early_import, d(3, 1), d(3, 11), Complete);
    let late_import = f.import();
    f.coverage(late_import, d(3, 5), d(3, 8), Partial);

    // T1 begins first, so its transaction_timestamp() is the earlier one,
    // but it commits last. T2 begins later, terminalizes its import with
    // `completed_at = transaction_timestamp()` exactly as the lifecycle
    // does, and commits first.
    let mut t1 = establish();
    exec(&mut t1, "BEGIN");
    exec(&mut t1, "SELECT transaction_timestamp()");
    thread::sleep(Duration::from_millis(50));
    let mut t2 = establish();
    exec(&mut t2, "BEGIN");
    exec(&mut t2, &format!("UPDATE metric_import SET status = 'COMPLETED', completed_at = transaction_timestamp() WHERE import_id = '{late_import}'"));
    exec(&mut t2, "COMMIT");
    assert_runs(
        &f,
        f.account,
        vec![f.run(d(3, 5), d(3, 8), Partial, Completed)],
    );

    // T1 now terminalizes with its earlier timestamp and commits later.
    // Under READ COMMITTED its maintenance pass, after taking the account
    // lock, sees T2's committed evidence, and the later completion (T2)
    // keeps winning 5-7 March: commit order never stands in for
    // completed_at order.
    exec(&mut t1, &format!("UPDATE metric_import SET status = 'COMPLETED', completed_at = transaction_timestamp() WHERE import_id = '{early_import}'"));
    exec(&mut t1, "COMMIT");

    let mut c = establish();
    let early_at = text(
        &mut c,
        &format!(
            "(SELECT completed_at::text FROM metric_import WHERE import_id = '{early_import}')"
        ),
    );
    let late_at = text(
        &mut c,
        &format!(
            "(SELECT completed_at::text FROM metric_import WHERE import_id = '{late_import}')"
        ),
    );
    assert!(
        early_at < late_at,
        "the later committer carries the earlier completion: {early_at} < {late_at}"
    );
    assert_runs(
        &f,
        f.account,
        vec![
            f.run(d(3, 1), d(3, 5), Complete, Completed),
            f.run(d(3, 5), d(3, 8), Partial, Completed),
            f.run(d(3, 8), d(3, 11), Complete, Completed),
        ],
    );

    // The mirror: the earlier-begun transaction commits first and the
    // later-begun one, with the later timestamp, commits last and wins.
    let third = f.import();
    f.coverage(third, d(3, 9), d(3, 12), Unknown);
    let fourth = f.import();
    f.coverage(fourth, d(3, 10), d(3, 13), Partial);
    let mut t3 = establish();
    exec(&mut t3, "BEGIN");
    exec(&mut t3, "SELECT transaction_timestamp()");
    thread::sleep(Duration::from_millis(50));
    let mut t4 = establish();
    exec(&mut t4, "BEGIN");
    exec(&mut t3, &format!("UPDATE metric_import SET status = 'COMPLETED', completed_at = transaction_timestamp() WHERE import_id = '{third}'"));
    exec(&mut t3, "COMMIT");
    exec(&mut t4, &format!("UPDATE metric_import SET status = 'COMPLETED', completed_at = transaction_timestamp() WHERE import_id = '{fourth}'"));
    exec(&mut t4, "COMMIT");
    assert_runs(
        &f,
        f.account,
        vec![
            f.run(d(3, 1), d(3, 5), Complete, Completed),
            f.run(d(3, 5), d(3, 8), Partial, Completed),
            f.run(d(3, 8), d(3, 9), Complete, Completed),
            f.run(d(3, 9), d(3, 10), Unknown, Completed),
            f.run(d(3, 10), d(3, 13), Partial, Completed),
        ],
    );
}

// ===========================================================================
// Rebuild
// ===========================================================================

#[test]
fn rebuild_locks_the_account_before_pre_verification_and_leaves_an_exact_account_untouched() {
    let (_guard, f) = setup();
    let import = f.import();
    f.coverage(import, d(3, 1), d(3, 11), Complete);
    f.complete(import, 1);
    let versions = f.run_versions();

    let (pool, log) = logging_pool();
    let result = rebuild_metric_coverage_runs(&pool, f.account).expect("rebuild");
    assert!(!result.rebuilt);
    assert_eq!(result.source_account_id, f.account);
    assert!(result.verification.exact);
    assert_eq!(result.verification.expected_rows, 1);
    assert_eq!(
        f.run_versions(),
        versions,
        "an exact account is not written"
    );

    let statements: Vec<String> = log
        .lock()
        .unwrap()
        .iter()
        .map(|s| {
            s.split_once(" -- binds:")
                .map_or(s.as_str(), |(t, _)| t)
                .to_string()
        })
        .collect();
    let position = |needle: &str| {
        statements
            .iter()
            .position(|s| s.starts_with(needle))
            .unwrap_or_else(|| panic!("`{needle}` must be issued: {statements:#?}"))
    };
    let begin = position("BEGIN");
    let lock_timeout = position(MAINTENANCE_LOCK_TIMEOUT_SQL);
    let statement_timeout = position(MAINTENANCE_STATEMENT_TIMEOUT_SQL);
    let isolation = position(TRANSACTION_ISOLATION_SQL);
    let lock = position(LOCK_SOURCE_ACCOUNT_SQL);
    let verify = position("WITH accounts AS");
    assert!(begin < lock_timeout && lock_timeout < statement_timeout && statement_timeout < isolation && isolation < lock && lock < verify,
        "order must be BEGIN, timeouts, isolation check, account FOR UPDATE, verification: {statements:#?}");
    assert_eq!(
        statements
            .iter()
            .filter(|s| s.starts_with("WITH accounts AS"))
            .count(),
        1,
        "one verification only"
    );
    assert!(
        statements
            .iter()
            .all(|s| !s.starts_with(DELETE_ACCOUNT_RUNS_SQL)
                && !s.starts_with("INSERT INTO public.metric_coverage_run")),
        "no write"
    );
    assert!(statements.iter().any(|s| s.starts_with("COMMIT")));
}

#[test]
fn rebuild_rejects_an_unknown_account_with_the_invalid_classification_and_writes_nothing() {
    let (_guard, f) = setup();
    let import = f.import();
    f.coverage(import, d(3, 1), d(3, 11), Complete);
    f.complete(import, 1);
    let versions = f.run_versions();
    let (pool, log) = logging_pool();
    assert_eq!(
        rebuild_metric_coverage_runs(&pool, Uuid::new_v4()),
        Err(MetricCoverageRunMaintenanceError::UnknownSourceAccount)
    );
    assert_eq!(f.run_versions(), versions);
    let statements = log.lock().unwrap();
    assert!(statements
        .iter()
        .any(|s| s.starts_with(LOCK_SOURCE_ACCOUNT_SQL)));
    assert!(
        !statements.iter().any(|s| s.starts_with("WITH accounts AS")),
        "nothing is read after the missing account"
    );
    assert!(statements.iter().any(|s| s.starts_with("ROLLBACK")));
    assert_eq!(
        MetricCoverageRunMaintenanceError::UnknownSourceAccount.code(),
        "METRIC_QUERY_INVALID"
    );
}

#[test]
fn rebuild_repairs_missing_extra_mismatched_and_structural_corruption_and_repeats_idempotently() {
    let (_guard, f) = setup();
    let base = f.import();
    f.coverage(base, d(3, 1), d(3, 11), Complete);
    let partial = f.import();
    f.coverage(partial, d(3, 5), d(3, 8), Partial);
    f.complete(base, 1);
    f.complete(partial, 2);
    let expected = vec![
        f.run(d(3, 1), d(3, 5), Complete, Completed),
        f.run(d(3, 5), d(3, 8), Partial, Completed),
        f.run(d(3, 8), d(3, 11), Complete, Completed),
    ];
    assert_runs(&f, f.account, expected.clone());
    // Another account's exact runs must survive the rebuild untouched.
    let other = f.import_of(f.account_b, Some(f.publisher_a));
    f.coverage_on(
        f.account_b,
        other,
        f.platform,
        d(3, 1),
        d(3, 3),
        Complete,
        false,
        false,
    );
    f.complete(other, 3);
    let other_versions: String = f
        .run_versions()
        .split(';')
        .filter(|v| v.starts_with(&f.account_b.to_string()))
        .collect::<Vec<_>>()
        .join(";");

    // Missing: delete one run.
    f.sql(&format!("DELETE FROM metric_coverage_run WHERE run_start = '2026-03-05' AND source_account_id = '{}'", f.account));
    let v = f.verify(f.account);
    assert_eq!(
        (
            v.expected_rows,
            v.actual_rows,
            v.missing_rows,
            v.extra_rows,
            v.mismatched_rows,
            v.structural_violations,
            v.exact
        ),
        (3, 2, 1, 0, 0, 0, false)
    );
    // Extra: a run no evidence implies.
    f.sql(&format!("INSERT INTO metric_coverage_run VALUES ('{}', '{}', '{}', '{}', '2026-07-01', '2026-07-05', 'COMPLETE', 'COMPLETED', FALSE, FALSE)", f.account, f.platform, f.publisher_b, f.measure));
    let v = f.verify(f.account);
    assert_eq!(
        (
            v.missing_rows,
            v.extra_rows,
            v.mismatched_rows,
            v.structural_violations
        ),
        (1, 1, 0, 0)
    );
    // Mismatched: a wrong run_end and a wrong visible value on matched keys.
    f.sql(&format!("UPDATE metric_coverage_run SET run_end = '2026-03-04' WHERE run_start = '2026-03-01' AND source_account_id = '{}'", f.account));
    f.sql(&format!("UPDATE metric_coverage_run SET import_status = 'COMPLETED_WITH_ERRORS', country_coverage = TRUE WHERE run_start = '2026-03-08' AND source_account_id = '{}'", f.account));
    let v = f.verify(f.account);
    assert_eq!(
        (
            v.expected_rows,
            v.actual_rows,
            v.missing_rows,
            v.extra_rows,
            v.mismatched_rows
        ),
        (3, 3, 1, 1, 2),
        "{v:?}"
    );
    // Structural V3 (overlap) and V4 (non-maximal adjacency), additive.
    f.sql(&format!("INSERT INTO metric_coverage_run VALUES ('{}', '{}', '{}', '{}', '2026-03-03', '2026-03-05', 'COMPLETE', 'COMPLETED', FALSE, FALSE)", f.account, f.platform, f.publisher_a, f.measure));
    f.sql(&format!("INSERT INTO metric_coverage_run VALUES ('{}', '{}', '{}', '{}', '2026-07-05', '2026-07-09', 'COMPLETE', 'COMPLETED', FALSE, FALSE)", f.account, f.platform, f.publisher_b, f.measure));
    let v = f.verify(f.account);
    assert_eq!(
        v.structural_violations, 2,
        "one overlap ([1,4) then [3,5)) and one uncoalesced adjacency: {v:?}"
    );
    assert!(!v.exact);

    let result = rebuild_metric_coverage_runs(&f.pool, f.account).expect("rebuild");
    assert!(result.rebuilt);
    assert!(result.verification.exact);
    assert_eq!(result.verification.expected_rows, 3);
    assert_eq!(result.verification.actual_rows, 3);
    assert_runs(&f, f.account, expected);
    assert!(
        f.run_versions().contains(&other_versions),
        "the other account's rows are physically untouched"
    );
    f.assert_exact(f.account_b, 1);

    // Idempotent: the second rebuild finds the account exact and writes nothing.
    let versions = f.run_versions();
    let again = rebuild_metric_coverage_runs(&f.pool, f.account).expect("rebuild");
    assert!(!again.rebuilt);
    assert_eq!(again.verification, result.verification);
    assert_eq!(f.run_versions(), versions);
    // An account without any evidence rebuilds to nothing and is exact.
    let empty = f.import_of(f.account_b, Some(f.publisher_a));
    f.complete(empty, 4);
    f.sql(&format!(
        "DELETE FROM metric_coverage_run WHERE source_account_id = '{}'",
        f.account_b
    ));
    assert!(
        rebuild_metric_coverage_runs(&f.pool, f.account_b)
            .unwrap()
            .rebuilt
    );
    f.assert_exact(f.account_b, 1);
}

#[test]
fn a_failed_post_rebuild_verification_rolls_the_replacement_back() {
    let (_guard, f) = setup();
    let import = f.import();
    f.coverage(import, d(3, 1), d(3, 11), Complete);
    f.complete(import, 1);
    // Corrupt the account so the rebuild must replace it...
    f.sql(&format!(
        "UPDATE metric_coverage_run SET run_end = '2026-03-09' WHERE source_account_id = '{}'",
        f.account
    ));
    let corrupt = f.runs();
    let versions = f.run_versions();
    // ...and make every replacement row inexact.
    let corrupting = TestTrigger::corrupting_run_insert();
    assert_eq!(
        rebuild_metric_coverage_runs(&f.pool, f.account),
        Err(MetricCoverageRunMaintenanceError::Internal)
    );
    drop(corrupting);
    assert_eq!(
        f.runs(),
        corrupt,
        "the corrupt state is intact: the replacement was rolled back"
    );
    assert_eq!(f.run_versions(), versions);
    assert_eq!(
        MetricCoverageRunMaintenanceError::Internal.code(),
        "INTERNAL_ERROR"
    );

    // Without the injection the same rebuild repairs and commits.
    assert!(
        rebuild_metric_coverage_runs(&f.pool, f.account)
            .unwrap()
            .rebuilt
    );
    assert_runs(
        &f,
        f.account,
        vec![f.run(d(3, 1), d(3, 11), Complete, Completed)],
    );
}

#[test]
fn rebuild_honours_the_frozen_timeouts_and_fails_closed_under_unsupported_isolation() {
    let (_guard, f) = setup();
    let import = f.import();
    f.coverage(import, d(3, 1), d(3, 11), Complete);
    f.complete(import, 1);
    f.sql("DELETE FROM metric_coverage_run");
    let versions = f.run_versions();

    // Unsupported isolation on the connection the rebuild will use.
    let (pool, _pid) = pinned_pool();
    {
        let mut c = pool.get().unwrap();
        exec(
            &mut c,
            "SET SESSION CHARACTERISTICS AS TRANSACTION ISOLATION LEVEL REPEATABLE READ",
        );
    }
    assert_eq!(
        rebuild_metric_coverage_runs(&pool, f.account),
        Err(MetricCoverageRunMaintenanceError::Internal)
    );
    assert_eq!(
        f.run_versions(),
        versions,
        "nothing was written under the refused isolation"
    );
    {
        let mut c = pool.get().unwrap();
        exec(
            &mut c,
            "SET SESSION CHARACTERISTICS AS TRANSACTION ISOLATION LEVEL READ COMMITTED",
        );
    }

    // lock_timeout 5s: with the account row held elsewhere the rebuild fails
    // after about five seconds rather than waiting indefinitely.
    let hold = HeldTransaction::start(
        "BEGIN",
        vec![format!(
            "SELECT 1 FROM metric_source_account WHERE source_account_id = '{}' FOR UPDATE",
            f.account
        )],
    );
    let started = Instant::now();
    assert_eq!(
        rebuild_metric_coverage_runs(&pool, f.account),
        Err(MetricCoverageRunMaintenanceError::Internal)
    );
    let elapsed = started.elapsed();
    assert!(
        elapsed >= Duration::from_secs(4) && elapsed < Duration::from_secs(20),
        "the 5s lock timeout bounds the wait: {elapsed:?}"
    );
    hold.commit().unwrap();
    assert_eq!(f.run_versions(), versions);

    // statement_timeout 30s is set locally, in the frozen form.
    let (logging, log) = logging_pool();
    assert!(
        rebuild_metric_coverage_runs(&logging, f.account)
            .unwrap()
            .rebuilt
    );
    let statements = log.lock().unwrap();
    assert_eq!(
        statements
            .iter()
            .filter(|s| s.starts_with("SET LOCAL lock_timeout = '5s'"))
            .count(),
        1
    );
    assert_eq!(
        statements
            .iter()
            .filter(|s| s.starts_with("SET LOCAL statement_timeout = '30s'"))
            .count(),
        1
    );
    assert!(!statements.iter().any(|s| s.contains("work_mem")));
    f.assert_exact(f.account, 1);
}

#[test]
fn a_concurrent_terminalization_and_rebuild_serialize_at_the_account_row() {
    let (_guard, f) = setup();
    let base = f.import();
    f.coverage(base, d(3, 1), d(3, 11), Complete);
    f.complete(base, 1);
    f.sql("DELETE FROM metric_coverage_run");
    let partial = f.import();
    f.coverage(partial, d(3, 5), d(3, 8), Partial);

    // Freeze the rebuild while it inserts its replacement, holding the
    // account lock.
    let hold = PauseHold::acquire();
    let pausing = TestTrigger::pausing_run_insert();
    let (rebuild_pool, rebuild_pid) = pinned_pool();
    let account = f.account;
    let rebuild = thread::spawn(move || rebuild_metric_coverage_runs(&rebuild_pool, account));
    wait_until_blocked(rebuild_pid, Some("advisory"));

    let (pool, pid) = pinned_pool();
    let terminalization = thread::spawn(move || {
        let mut c = pool.get().unwrap();
        sql_query(terminalize_sql(partial, 2)).execute(&mut c)
    });
    wait_until_blocked(pid, None);
    assert!(
        !terminalization.is_finished(),
        "the terminalization waits behind the rebuild"
    );

    hold.release();
    let result = rebuild.join().unwrap().expect("rebuild");
    assert!(result.rebuilt);
    terminalization.join().unwrap().expect("terminalization");
    drop(pausing);
    assert_runs(
        &f,
        f.account,
        vec![
            f.run(d(3, 1), d(3, 5), Complete, Completed),
            f.run(d(3, 5), d(3, 8), Partial, Completed),
            f.run(d(3, 8), d(3, 11), Complete, Completed),
        ],
    );
}

// ===========================================================================
// Verifier
// ===========================================================================

#[test]
fn structural_violations_v1_to_v4_are_counted_independently_and_additively() {
    let (_guard, f) = setup();
    let import = f.import();
    f.coverage(import, d(3, 1), d(3, 11), Complete);
    f.complete(import, 1);

    // V1 and V2 are prevented by the check and the foreign keys, so they are
    // exercised in a transaction that drops those constraints and rolls
    // back: fail-closed defence remains proven without weakening the schema.
    let mut c = establish();
    c.transaction::<_, diesel::result::Error, _>(|c| {
        sql_query("ALTER TABLE metric_coverage_run DROP CONSTRAINT metric_coverage_run_interval_check").execute(c)?;
        sql_query("ALTER TABLE metric_coverage_run DROP CONSTRAINT metric_coverage_run_publisher_id_fkey").execute(c)?;
        sql_query(format!("INSERT INTO metric_coverage_run VALUES ('{}', '{}', '{}', '{}', '2026-05-05', '2026-05-05', 'COMPLETE', 'COMPLETED', FALSE, FALSE)", f.account, f.platform, f.publisher_a, f.measure)).execute(c)?;
        sql_query(format!("INSERT INTO metric_coverage_run VALUES ('{}', '{}', '{}', '{}', '2026-06-01', '2026-06-02', 'COMPLETE', 'COMPLETED', FALSE, FALSE)", f.account, f.platform, Uuid::new_v4(), f.measure)).execute(c)?;
        let v = verify_source_account(c, f.account).unwrap();
        assert_eq!((v.expected_rows, v.actual_rows, v.extra_rows, v.structural_violations, v.exact), (1, 3, 2, 2, false), "V1 + V2: {v:?}");
        Err::<(), diesel::result::Error>(diesel::result::Error::RollbackTransaction)
    })
    .unwrap_err();
    f.assert_exact(f.account, 1);

    // V3 overlap and V4 non-maximal adjacency, in one stream, additively;
    // a row may contribute to more than one predicate.
    f.sql(&format!(
        "UPDATE metric_coverage_run SET run_end = '2026-03-06' WHERE source_account_id = '{}'",
        f.account
    ));
    f.sql(&format!("INSERT INTO metric_coverage_run VALUES ('{}', '{}', '{}', '{}', '2026-03-04', '2026-03-08', 'COMPLETE', 'COMPLETED', FALSE, FALSE)", f.account, f.platform, f.publisher_a, f.measure));
    f.sql(&format!("INSERT INTO metric_coverage_run VALUES ('{}', '{}', '{}', '{}', '2026-03-08', '2026-03-11', 'COMPLETE', 'COMPLETED', FALSE, FALSE)", f.account, f.platform, f.publisher_a, f.measure));
    let v = f.verify(f.account);
    assert_eq!(
        v.structural_violations, 2,
        "[1,6) overlaps [4,8) (V3); [4,8) and [8,11) are equal and adjacent (V4): {v:?}"
    );
    assert_eq!((v.mismatched_rows, v.extra_rows), (1, 2));
    // Adjacent runs with a differing visible value are not V4.
    f.sql(&format!("UPDATE metric_coverage_run SET coverage_status = 'PARTIAL' WHERE run_start = '2026-03-08' AND source_account_id = '{}'", f.account));
    assert_eq!(f.verify(f.account).structural_violations, 1);
    // Runs of different streams never interact structurally.
    f.sql(&format!("INSERT INTO metric_coverage_run VALUES ('{}', '{}', '{}', '{}', '2026-03-01', '2026-03-11', 'COMPLETE', 'COMPLETED', FALSE, FALSE)", f.account, f.platform, f.publisher_b, f.measure));
    assert_eq!(f.verify(f.account).structural_violations, 1);
    assert!(
        rebuild_metric_coverage_runs(&f.pool, f.account)
            .unwrap()
            .rebuilt
    );
    f.assert_exact(f.account, 1);
}

#[test]
fn current_account_configuration_is_neither_structural_corruption_nor_a_domain_exclusion() {
    let (_guard, f) = setup();
    let import = f.import();
    f.coverage(import, d(3, 1), d(3, 11), Complete);
    f.complete(import, 1);
    let page_before = verify_metric_coverage_runs(&f.pool, None, None).unwrap();
    assert_eq!(page_before.domain_account_count, 2);

    // Disabled, re-pointed at another platform and publisher, then without an
    // expected publisher at all: the account stays in the domain with the
    // same fingerprint and its runs remain structurally exact.
    for change in [
        format!("UPDATE metric_source_account SET enabled = FALSE WHERE source_account_id = '{}'", f.account),
        format!("UPDATE metric_source_account SET platform_id = '{}', expected_publisher_id = '{}' WHERE source_account_id = '{}'", f.other_platform, f.publisher_b, f.account),
        format!("UPDATE metric_source_account SET expected_publisher_id = NULL WHERE source_account_id = '{}'", f.account),
        format!("UPDATE metric_source SET enabled = FALSE WHERE source_id = (SELECT source_id FROM metric_source_account WHERE source_account_id = '{}')", f.account),
    ] {
        f.sql(&change);
        let page = verify_metric_coverage_runs(&f.pool, None, None).unwrap();
        assert_eq!(page.domain_account_count, 2);
        assert_eq!(page.domain_fingerprint, page_before.domain_fingerprint);
        let account = page.accounts.iter().find(|a| a.source_account_id == f.account).unwrap();
        assert!(account.exact && account.structural_violations == 0 && account.expected_rows == 1, "{account:?}");
    }
}

#[test]
fn verification_pages_are_strict_ascending_keyset_pages_with_an_exact_domain_fingerprint() {
    let (_guard, f) = setup();
    // 23 accounts in total: the two fixture accounts plus 21 more.
    let mut c = f.pool.get().unwrap();
    for n in 0..21 {
        exec(&mut c, &format!("INSERT INTO metric_source_account (source_account_id, code, source_id, platform_id, external_key, enabled) VALUES ('{}', 'b1-page-{n}', (SELECT source_id FROM metric_source WHERE code = 'b1-src'), '{}', 'page-{n}', TRUE)", Uuid::new_v4(), f.platform));
    }
    let mut domain: Vec<Uuid> = {
        #[derive(diesel::QueryableByName)]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            source_account_id: Uuid,
        }
        sql_query(DOMAIN_SQL)
            .load::<Row>(&mut c)
            .unwrap()
            .into_iter()
            .map(|r| r.source_account_id)
            .collect()
    };
    assert_eq!(domain.len(), 23);
    let server_order = domain.clone();
    domain.sort();
    assert_eq!(
        domain, server_order,
        "Rust UUID order equals PostgreSQL uuid order"
    );

    // The fingerprint, computed independently here.
    let mut hasher = Sha256::new();
    hasher.update(format!("{METRIC_COVERAGE_RUN_DOMAIN_FINGERPRINT_SCHEMA}\n").as_bytes());
    for id in &domain {
        hasher.update(format!("{}\n", id.hyphenated()).as_bytes());
    }
    let fingerprint = format!("{:x}", hasher.finalize());
    assert_eq!(domain_fingerprint(&domain), fingerprint);
    assert_eq!(fingerprint.len(), 64);
    assert!(fingerprint
        .chars()
        .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));

    // Default limit 10, null boundary: the first ten; then exactly the next
    // ten after the returned boundary; then the final three and no boundary.
    let first = verify_metric_coverage_runs(&f.pool, None, None).unwrap();
    assert_eq!(
        first
            .accounts
            .iter()
            .map(|a| a.source_account_id)
            .collect::<Vec<_>>(),
        domain[0..10]
    );
    assert_eq!(first.next_after_source_account_id, Some(domain[9]));
    assert_eq!(first.domain_account_count, 23);
    assert_eq!(first.domain_fingerprint, fingerprint);
    let second =
        verify_metric_coverage_runs(&f.pool, first.next_after_source_account_id, Some(10)).unwrap();
    assert_eq!(
        second
            .accounts
            .iter()
            .map(|a| a.source_account_id)
            .collect::<Vec<_>>(),
        domain[10..20]
    );
    assert_eq!(second.next_after_source_account_id, Some(domain[19]));
    let third = verify_metric_coverage_runs(&f.pool, second.next_after_source_account_id, Some(10))
        .unwrap();
    assert_eq!(
        third
            .accounts
            .iter()
            .map(|a| a.source_account_id)
            .collect::<Vec<_>>(),
        domain[20..23]
    );
    assert_eq!(third.next_after_source_account_id, None);
    for page in [&first, &second, &third] {
        assert_eq!(
            (page.domain_account_count, page.domain_fingerprint.as_str()),
            (23, fingerprint.as_str())
        );
        assert!(page
            .accounts
            .iter()
            .all(|a| a.exact && a.expected_rows == 0));
    }
    // A boundary that names no account is an exclusive ordering boundary.
    let between = Uuid::from_u128(domain[4].as_u128() + 1);
    assert!(!domain.contains(&between));
    let page = verify_metric_coverage_runs(&f.pool, Some(between), Some(3)).unwrap();
    assert_eq!(
        page.accounts
            .iter()
            .map(|a| a.source_account_id)
            .collect::<Vec<_>>(),
        domain[5..8]
    );
    assert_eq!(page.next_after_source_account_id, Some(domain[7]));
    // Exactly `limit` accounts remaining: the page is full and the domain is
    // exhausted, so no boundary is returned.
    let page = verify_metric_coverage_runs(&f.pool, Some(domain[19]), Some(3)).unwrap();
    assert_eq!(page.accounts.len(), 3);
    assert_eq!(page.next_after_source_account_id, None);
    // Past the end: empty, no boundary, same domain facts.
    let page = verify_metric_coverage_runs(&f.pool, Some(domain[22]), Some(1)).unwrap();
    assert!(page.accounts.is_empty());
    assert_eq!(page.next_after_source_account_id, None);
    assert_eq!(page.domain_fingerprint, fingerprint);
    // Limit 1 and 10 are accepted; 0 and 11 are rejected before any
    // connection is used.
    assert_eq!(
        verify_metric_coverage_runs(&f.pool, None, Some(1))
            .unwrap()
            .accounts
            .len(),
        1
    );
    assert_eq!(
        verify_metric_coverage_runs(&f.pool, None, Some(10))
            .unwrap()
            .accounts
            .len(),
        10
    );
    let (logging, log) = logging_pool();
    for bad in [0, 11, -1, i32::MIN, i32::MAX] {
        assert_eq!(
            verify_metric_coverage_runs(&logging, None, Some(bad)),
            Err(MetricCoverageRunMaintenanceError::VerifyLimitOutOfRange),
            "{bad}"
        );
    }
    assert!(
        log.lock().unwrap().is_empty(),
        "a rejected limit opens no transaction"
    );
    assert_eq!(
        MetricCoverageRunMaintenanceError::VerifyLimitOutOfRange.code(),
        "METRIC_QUERY_LIMIT_EXCEEDED"
    );

    // The page runs read-only at repeatable read with the frozen timeouts and
    // no row lock, and the domain read precedes the page comparison.
    verify_metric_coverage_runs(&logging, None, Some(2)).unwrap();
    let statements: Vec<String> = log
        .lock()
        .unwrap()
        .iter()
        .map(|s| {
            s.split_once(" -- binds:")
                .map_or(s.as_str(), |(t, _)| t)
                .to_string()
        })
        .collect();
    let begin = statements
        .iter()
        .find(|s| s.starts_with("BEGIN"))
        .expect("BEGIN");
    assert!(
        begin.contains("READ ONLY") && begin.contains("REPEATABLE READ"),
        "{begin}"
    );
    let position = |needle: &str| {
        statements
            .iter()
            .position(|s| s.starts_with(needle))
            .unwrap_or_else(|| panic!("{needle}: {statements:#?}"))
    };
    assert!(position(MAINTENANCE_LOCK_TIMEOUT_SQL) < position(MAINTENANCE_STATEMENT_TIMEOUT_SQL));
    assert!(position(MAINTENANCE_STATEMENT_TIMEOUT_SQL) < position(TRANSACTION_MODE_SQL));
    assert!(position(TRANSACTION_MODE_SQL) < position(DOMAIN_SQL));
    assert!(position(DOMAIN_SQL) < position("WITH accounts AS"));
    assert!(statements
        .iter()
        .all(|s| !s.contains("FOR UPDATE") && !s.contains("LOCK TABLE")));
    assert!(statements.iter().any(|s| s.starts_with("COMMIT")));
}

#[test]
fn the_empty_domain_fingerprint_is_the_schema_line_alone_and_counts_fail_closed_beyond_int() {
    let (_guard, _pool) = setup_registry_db();
    let mut c = establish();
    exec(&mut c, "DELETE FROM metric_source_account");
    let pool = Arc::new(
        diesel::r2d2::Pool::builder()
            .max_size(1)
            .build(ConnectionManager::<PgConnection>::new(test_db_url()))
            .unwrap(),
    );
    let page = verify_metric_coverage_runs(&pool, None, None).unwrap();
    assert!(page.accounts.is_empty());
    assert_eq!(page.next_after_source_account_id, None);
    assert_eq!(page.domain_account_count, 0);
    let mut hasher = Sha256::new();
    hasher.update(b"thoth-metric-coverage-run-domain/1\n");
    assert_eq!(page.domain_fingerprint, format!("{:x}", hasher.finalize()));
    assert_eq!(page.domain_fingerprint, domain_fingerprint(&[]));

    assert_eq!(bounded_count(0, "x"), Ok(0));
    assert_eq!(bounded_count(i64::from(i32::MAX), "x"), Ok(i32::MAX));
    assert_eq!(
        bounded_count(i64::from(i32::MAX) + 1, "x"),
        Err(MetricCoverageRunMaintenanceError::Internal)
    );
    assert_eq!(bounded_count(-1, "x"), Ok(-1));
    assert_eq!(
        bounded_count(i64::MAX, "x"),
        Err(MetricCoverageRunMaintenanceError::Internal)
    );
}

#[test]
fn the_verifier_is_structurally_independent_of_the_producer() {
    // The verifier derives expected runs by per-day expansion; the trigger
    // function and the rebuild derive them by a boundary sweep. They share
    // no statement text, and the verifier module never references the
    // producer module.
    assert!(ACCOUNT_VERIFICATION_SQL.contains("generate_series"));
    assert!(!ACCOUNT_VERIFICATION_SQL.contains("LEAD("));
    assert!(!ACCOUNT_VERIFICATION_SQL.contains("boundar"));
    assert!(
        !ACCOUNT_VERIFICATION_SQL.contains("INSERT")
            && !ACCOUNT_VERIFICATION_SQL.contains("DELETE")
    );
    assert!(REBUILD_ACCOUNT_RUNS_SQL.contains("LEAD("));
    assert!(!REBUILD_ACCOUNT_RUNS_SQL.contains("generate_series"));
    let migration = include_str!("../../../migrations/20261006_v1.9.0/up.sql");
    assert!(migration.contains("LEAD(b.boundary)") && !migration.contains("generate_series"));
    let verifier = include_str!("verification.rs");
    let verifier_code: String = verifier
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !verifier_code.contains("crud::") && !verifier_code.contains("super::crud"),
        "the verifier imports nothing from the producer"
    );
    assert!(!verifier_code.contains("REBUILD_ACCOUNT_RUNS_SQL"));
    let producer = include_str!("crud.rs");
    assert!(
        producer.contains("use super::verification::verify_source_account"),
        "the rebuild commits only on the independent verifier"
    );
    // Both sides apply the same approved participation predicate and winner
    // ordering, which is semantics, not shared code.
    let migration_sql: String = migration
        .lines()
        .filter(|line| !line.trim_start().starts_with("--"))
        .collect::<Vec<_>>()
        .join("\n");
    for sql in [
        ACCOUNT_VERIFICATION_SQL,
        REBUILD_ACCOUNT_RUNS_SQL,
        migration_sql.as_str(),
    ] {
        assert!(sql.contains("mi.completed_at IS NOT NULL"));
        assert!(sql.contains("mi.publisher_id IS NOT NULL"));
        assert!(sql.contains("'COMPLETED_WITH_ERRORS'::public.metric_import_status"));
        assert!(sql.contains("e.completed_at DESC, e.import_id DESC"));
        assert!(sql.contains("e.coverage_status DESC, e.country_coverage ASC"));
        assert!(sql.contains("e.institution_coverage ASC, e.coverage_id DESC"));
        assert!(
            !sql.contains("expected_publisher_id"),
            "no current-account provenance"
        );
    }
    // The H1 error boundary is local: no reader-owned error type is used.
    for source in [include_str!("mod.rs"), include_str!("crud.rs"), verifier] {
        let code: String = source
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(!code.contains("MetricReadError") && !code.contains("metric_dashboard"));
    }
}

// ===========================================================================
// Identity manifests
// ===========================================================================

const PRODUCER_MANIFEST: &str = include_str!("producer_manifest.txt");
const VERIFIER_MANIFEST: &str = include_str!("verifier_manifest.txt");

/// Parse one canonical manifest, panicking on any encoding violation.
fn parse_manifest(manifest: &str, header: &str) -> Vec<(String, String)> {
    assert!(manifest.is_ascii(), "ASCII only");
    assert!(
        manifest.ends_with('\n') && !manifest.ends_with("\n\n"),
        "exactly one final LF"
    );
    assert!(!manifest.contains('\r'), "LF line endings");
    let mut lines = manifest.split_terminator('\n');
    assert_eq!(lines.next(), Some(header));
    let entries: Vec<(String, String)> = lines
        .map(|line| {
            assert!(!line.is_empty(), "no blank line");
            assert!(!line.starts_with('#'), "no comment");
            let (role, path) = line
                .split_once('\t')
                .unwrap_or_else(|| panic!("`{line}` is not role<TAB>path"));
            assert!(
                !role.is_empty() && !path.is_empty() && !path.contains('\t') && !role.contains(' '),
                "{line}"
            );
            (role.to_string(), path.to_string())
        })
        .collect();
    let paths: Vec<&[u8]> = entries.iter().map(|(_, path)| path.as_bytes()).collect();
    assert!(
        paths.windows(2).all(|pair| pair[0] < pair[1]),
        "sorted strictly by path in byte order and unique: {paths:?}"
    );
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    for (_, path) in &entries {
        assert!(
            root.join(path).is_file(),
            "`{path}` must exist in the repository"
        );
    }
    entries
}

/// The identity of one manifest: SHA-256 over the identity stream of the
/// manifest hash and every entry's file hash, from the working-tree bytes.
fn identity(manifest: &str, manifest_header: &str, identity_header: &str) -> (String, String) {
    let entries = parse_manifest(manifest, manifest_header);
    let manifest_hex = format!("{:x}", Sha256::digest(manifest.as_bytes()));
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut stream = format!("{identity_header}\nmanifest-sha256\t{manifest_hex}\n");
    for (role, path) in entries {
        let bytes = std::fs::read(root.join(&path)).unwrap();
        stream.push_str(&format!("{role}\t{path}\t{:x}\n", Sha256::digest(&bytes)));
    }
    (
        manifest_hex,
        format!("{:x}", Sha256::digest(stream.as_bytes())),
    )
}

#[test]
fn the_identity_manifests_have_exactly_the_frozen_canonical_contents() {
    assert_eq!(
        PRODUCER_MANIFEST,
        "thoth-h1-producer-manifest/1\n\
         dependency-lock\tCargo.lock\n\
         workspace-manifest\tCargo.toml\n\
         crate-manifest\tthoth-api/Cargo.toml\n\
         migration-down\tthoth-api/migrations/20261006_v1.9.0/down.sql\n\
         migration-up\tthoth-api/migrations/20261006_v1.9.0/up.sql\n\
         maintenance-resolver\tthoth-api/src/graphql/mutation.rs\n\
         coverage-model\tthoth-api/src/model/metric_coverage/mod.rs\n\
         h1-producer\tthoth-api/src/model/metric_coverage_run/crud.rs\n\
         h1-model\tthoth-api/src/model/metric_coverage_run/mod.rs\n\
         h1-verifier\tthoth-api/src/model/metric_coverage_run/verification.rs\n\
         import-model\tthoth-api/src/model/metric_import/mod.rs\n\
         ingestion-country\tthoth-api/src/model/metric_ingestion/country.rs\n\
         ingestion-error\tthoth-api/src/model/metric_ingestion/error.rs\n\
         ingestion-hash\tthoth-api/src/model/metric_ingestion/hash.rs\n\
         raw-evidence-writer\tthoth-api/src/model/metric_ingestion/mod.rs\n\
         terminalization\tthoth-api/src/model/metric_ingestion_lifecycle/mod.rs\n\
         measure-model\tthoth-api/src/model/metric_measure/mod.rs\n\
         platform-model\tthoth-api/src/model/metric_platform/mod.rs\n\
         platform-measure-model\tthoth-api/src/model/metric_platform_measure/mod.rs\n\
         source-model\tthoth-api/src/model/metric_source/mod.rs\n\
         source-account-model\tthoth-api/src/model/metric_source_account/mod.rs\n\
         publisher-model\tthoth-api/src/model/publisher/mod.rs\n\
         authorization\tthoth-api/src/policy.rs\n\
         diesel-schema\tthoth-api/src/schema.rs\n"
    );
    assert_eq!(
        VERIFIER_MANIFEST,
        "thoth-h1-verifier-manifest/1\n\
         dependency-lock\tCargo.lock\n\
         workspace-manifest\tCargo.toml\n\
         crate-manifest\tthoth-api/Cargo.toml\n\
         graphql-output\tthoth-api/src/graphql/model.rs\n\
         maintenance-resolver\tthoth-api/src/graphql/mutation.rs\n\
         coverage-model\tthoth-api/src/model/metric_coverage/mod.rs\n\
         h1-model\tthoth-api/src/model/metric_coverage_run/mod.rs\n\
         h1-verifier\tthoth-api/src/model/metric_coverage_run/verification.rs\n\
         import-model\tthoth-api/src/model/metric_import/mod.rs\n\
         source-account-model\tthoth-api/src/model/metric_source_account/mod.rs\n\
         authorization\tthoth-api/src/policy.rs\n\
         diesel-schema\tthoth-api/src/schema.rs\n"
    );
    let producer = parse_manifest(PRODUCER_MANIFEST, "thoth-h1-producer-manifest/1");
    let verifier = parse_manifest(VERIFIER_MANIFEST, "thoth-h1-verifier-manifest/1");
    assert_eq!(producer.len(), 24);
    assert_eq!(verifier.len(), 12);
    assert_ne!(
        PRODUCER_MANIFEST, VERIFIER_MANIFEST,
        "separate files, separate identities"
    );
    let roles = |entries: &[(String, String)]| {
        entries
            .iter()
            .map(|(r, _)| r.clone())
            .collect::<std::collections::BTreeSet<_>>()
    };
    assert_eq!(
        roles(&producer).len(),
        producer.len(),
        "roles are unique too"
    );
    assert_eq!(roles(&verifier).len(), verifier.len());
}

#[test]
fn producer_and_verifier_identities_are_deterministic_content_identities() {
    let (producer_manifest_sha, producer_identity) = identity(
        PRODUCER_MANIFEST,
        "thoth-h1-producer-manifest/1",
        "thoth-h1-producer-identity/1",
    );
    let (verifier_manifest_sha, verifier_identity) = identity(
        VERIFIER_MANIFEST,
        "thoth-h1-verifier-manifest/1",
        "thoth-h1-verifier-identity/1",
    );
    for hex in [
        &producer_manifest_sha,
        &producer_identity,
        &verifier_manifest_sha,
        &verifier_identity,
    ] {
        assert_eq!(hex.len(), 64);
        assert!(hex
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    }
    assert_ne!(producer_identity, verifier_identity);
    // Deterministic over the same bytes.
    assert_eq!(
        identity(
            PRODUCER_MANIFEST,
            "thoth-h1-producer-manifest/1",
            "thoth-h1-producer-identity/1"
        )
        .1,
        producer_identity
    );
    // Reported for the implementation record; the committed-head values are
    // computed from Git blob bytes and must agree with these when the tree
    // is clean.
    println!("producer manifest SHA-256: {producer_manifest_sha}");
    println!("producerIdentity (working tree): {producer_identity}");
    println!("verifier manifest SHA-256: {verifier_manifest_sha}");
    println!("verifierIdentity (working tree): {verifier_identity}");
}

// ===========================================================================
// H2: the native-grain partial index under the live predicate
// ===========================================================================

#[test]
fn the_planner_uses_the_native_grain_index_under_the_live_predicate_without_changing_results() {
    let (_guard, f) = setup();
    let mut c = f.pool.get().unwrap();
    // A representative canonical population: 200 works with a year of DAY
    // rows each, and MONTH rows for a fifth of them, so the DAY-dominated
    // table makes a full scan unattractive and the partial index selective.
    let imprint = Uuid::new_v4();
    exec(&mut c, &format!("INSERT INTO imprint (imprint_id, publisher_id, imprint_name) VALUES ('{imprint}', '{}', 'B1 imprint')", f.publisher_a));
    exec(&mut c, &format!("INSERT INTO work (work_id, work_type, work_status, imprint_id, edition) SELECT uuid_generate_v4(), 'monograph', 'forthcoming', '{imprint}', 1 FROM generate_series(1, 200)"));
    exec(&mut c, &format!("INSERT INTO metric_record (identity_hash, work_id, platform_id, measure_id, period_start, period_end, reporting_grain, winning_source_account_id) SELECT 'h2-day-' || w.work_id || '-' || d, w.work_id, '{}', '{}', DATE '2025-01-01' + d, DATE '2025-01-02' + d, 'DAY', '{}' FROM work w CROSS JOIN generate_series(0, 364) AS d", f.platform, f.measure, f.account));
    exec(&mut c, &format!("INSERT INTO metric_record (identity_hash, work_id, platform_id, measure_id, period_start, period_end, reporting_grain, winning_source_account_id) SELECT 'h2-month-' || w.work_id || '-' || m, w.work_id, '{}', '{}', (DATE '2025-01-01' + (m || ' months')::interval)::date, (DATE '2025-02-01' + (m || ' months')::interval)::date, (CASE WHEN m % 2 = 0 THEN 'MONTH' ELSE 'REPORTING_PERIOD' END)::metric_reporting_grain, '{}' FROM (SELECT work_id FROM work ORDER BY work_id LIMIT 40) w CROSS JOIN generate_series(0, 11) AS m", f.platform, f.measure, f.account));
    exec(&mut c, "ANALYZE metric_record");
    assert!(scalar_i64(&f.pool, "(SELECT COUNT(*) FROM metric_record)") > 70_000);

    // The live `NATIVE_GRAIN_SQL` predicate shape of the dashboard reader
    // over 50 works, with the array parameters as the reader binds them.
    let works = text(&mut c, "(SELECT string_agg(quote_literal(work_id::text), ',') FROM (SELECT work_id FROM work ORDER BY work_id LIMIT 50) w)");
    let predicate = format!(
        "FROM public.metric_record r \
         LEFT JOIN public.metric_record_revision v \
           ON v.record_id = r.record_id AND v.record_revision_id = r.current_revision_id \
         WHERE r.work_id = ANY(ARRAY[{works}]::uuid[]) \
           AND r.platform_id = ANY(ARRAY['{}']::uuid[]) \
           AND r.measure_id = ANY(ARRAY['{}']::uuid[]) \
           AND r.period_start < DATE '2025-12-31' \
           AND r.period_end > DATE '2025-01-01' \
           AND r.reporting_grain IN ('MONTH', 'REPORTING_PERIOD') \
           AND NOT (COALESCE(v.status = 'RETRACTED', FALSE) \
                    AND NOT EXISTS (SELECT 1 FROM public.metric_record_revision o \
                                    WHERE o.record_id = r.record_id AND o.status = 'CURRENT'))",
        f.platform, f.measure
    );
    let query = format!("SELECT COUNT(*) {predicate}");

    exec(
        &mut c,
        "CREATE TEMP TABLE b1_h2_plan (step_id serial, line text)",
    );
    exec(&mut c, &format!(
        "DO $$ DECLARE plan_row record; BEGIN FOR plan_row IN EXECUTE 'EXPLAIN {}' LOOP INSERT INTO b1_h2_plan (line) VALUES (plan_row.\"QUERY PLAN\"); END LOOP; END $$;",
        query.replace('\'', "''")
    ));
    let plan = text(
        &mut c,
        "(SELECT string_agg(line, chr(10) ORDER BY step_id) FROM b1_h2_plan)",
    );
    assert!(
        plan.contains("metric_record_native_grain_idx"),
        "the live native-grain predicate must use the partial index:\n{plan}"
    );
    assert!(
        !plan.contains("Seq Scan on metric_record r"),
        "no sequential scan of metric_record:\n{plan}"
    );
    println!("H2 plan:\n{plan}");

    // Query correctness is unchanged: the indexed result equals the result
    // with every index access path disabled.
    let indexed = scalar_i64(&f.pool, &format!("({query})"));
    let scanned = c
        .transaction::<i64, diesel::result::Error, _>(|c| {
            sql_query("SET LOCAL enable_indexscan = off").execute(c)?;
            sql_query("SET LOCAL enable_bitmapscan = off").execute(c)?;
            sql_query("SET LOCAL enable_indexonlyscan = off").execute(c)?;
            diesel::select(diesel::dsl::sql::<diesel::sql_types::BigInt>(&format!(
                "({query})"
            )))
            .get_result(c)
        })
        .unwrap();
    assert_eq!(indexed, scanned);
    assert_eq!(indexed, 40 * 12);
}
