//! `MET-WP4-02` evidence for the coverage-aware dashboard read.
//!
//! These tests drive [`metric_dashboard`] directly against a disposable
//! database whose projection is produced by the real `MET-WP4-01` claim and
//! completion path: exact totals and timelines, zero versus unknown, current
//! coverage selection, source-scope eligibility, rollup freshness and
//! `dataThrough`, the one-snapshot property, every request bound, additivity,
//! publisher entitlement and set-based statement counts.
//!
//! Authorization, the GraphQL error classifications and the SDL are proven at
//! the API boundary in `crate::graphql::metric_dashboard_tests`, which reuses
//! the fixture below.
//!
//! One `#[ignore]`d test builds a production-shaped 600-work publisher and
//! prints `EXPLAIN (ANALYZE, BUFFERS)` plans and resolver timings for the
//! approved maximum request shapes. Run it explicitly with
//! `cargo test -p thoth-api --features backend -- --ignored metric_dashboard`.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration as StdDuration, Instant};

use chrono::NaiveDate;
use diesel::connection::{InstrumentationEvent, SimpleConnection};
use diesel::r2d2::{ConnectionManager, CustomizeConnection};
use diesel::{sql_query, Connection, PgConnection, RunQueryDsl};
use uuid::Uuid;

use super::*;
use crate::db::PgPool;
use crate::model::metric_platform::tests::{insert_platform_row, scalar_i64, setup_registry_db};
use crate::model::metric_rollup_delta::crud::{
    claim_metric_rollup_deltas, complete_metric_rollup_deltas,
};
use crate::model::metric_rollup_delta::tests::{deltas, projection, state};
use crate::model::tests::db::{failing_pool, test_db_url, TestDbGuard};

// ==========================================================================
// Fixture
// ==========================================================================

/// The claimant the fixture applies rollup batches as.
const APPLIER: &str = "metric-dashboard-test-applier";

pub(crate) fn d(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).expect("a valid fixture date")
}

/// 2026-03-01, the first day of the default window.
pub(crate) fn d1() -> NaiveDate {
    d(2026, 3, 1)
}

pub(crate) fn day_n(n: i64) -> NaiveDate {
    d1() + Duration::days(n - 1)
}

/// Run one or more statements that must succeed.
pub(crate) fn exec(pool: &PgPool, statements: &str) {
    let mut connection = pool.get().expect("Failed to get DB connection");
    connection
        .batch_execute(statements)
        .unwrap_or_else(|error| panic!("fixture statement failed: {error}\n{statements}"));
}

/// A publisher entitled or not entitled through its package, with works.
pub(crate) struct Fixture {
    pub(crate) pool: Arc<PgPool>,
    pub(crate) publisher_id: Uuid,
    pub(crate) works: Vec<Uuid>,
    pub(crate) other_publisher_id: Uuid,
    pub(crate) other_work: Uuid,
    pub(crate) platform_id: Uuid,
    pub(crate) other_platform_id: Uuid,
    /// The seeded additive `title_sessions` measure.
    pub(crate) sessions: Uuid,
    /// The seeded additive, signed `net_units` measure.
    pub(crate) units: Uuid,
    /// The one eligible DRIVER account for `publisher_id` on `platform_id`.
    pub(crate) account_id: Uuid,
    /// A completed import that canonical revisions are attributed to.
    pub(crate) canonical_import: Uuid,
}

pub(crate) fn publisher_sql(publisher_id: Uuid, package: &str) -> String {
    format!(
        "INSERT INTO publisher (publisher_id, publisher_name, subscription_package) \
         VALUES ('{publisher_id}', 'Publisher {publisher_id}', '{package}');"
    )
}

/// Insert one imprint with `count` works for a publisher.
pub(crate) fn works_sql(publisher_id: Uuid, work_ids: &[Uuid]) -> String {
    let imprint_id = Uuid::new_v4();
    let mut sql = format!(
        "INSERT INTO imprint (imprint_id, publisher_id, imprint_name) \
         VALUES ('{imprint_id}', '{publisher_id}', 'Imprint {imprint_id}');"
    );
    for work_id in work_ids {
        sql.push_str(&format!(
            "INSERT INTO work (work_id, work_type, work_status, imprint_id, edition) \
             VALUES ('{work_id}', 'monograph', 'forthcoming', '{imprint_id}', 1);"
        ));
    }
    sql
}

/// One source row. A `DRIVER` source carries a driver key, as the
/// `MET-WP1-13` `metric_source_driver_key_check` requires; any other
/// acquisition type carries none.
pub(crate) fn source_sql(source_id: Uuid, acquisition: &str, enabled: bool) -> String {
    let driver_key = if acquisition == "DRIVER" {
        "'dashboard_test_driver'"
    } else {
        "NULL"
    };
    format!(
        "INSERT INTO metric_source (source_id, code, acquisition_type, driver_key, enabled) \
         VALUES ('{source_id}', 'source-{source_id}', '{acquisition}', {driver_key}, {enabled});"
    )
}

pub(crate) fn account_sql(
    account_id: Uuid,
    source_id: Uuid,
    platform_id: Uuid,
    publisher_id: Option<Uuid>,
    enabled: bool,
) -> String {
    let publisher = publisher_id.map_or("NULL".to_string(), |id| format!("'{id}'"));
    format!(
        "INSERT INTO metric_source_account \
             (source_account_id, code, source_id, platform_id, external_key, \
              expected_publisher_id, enabled) \
         VALUES ('{account_id}', 'account-{account_id}', '{source_id}', '{platform_id}', \
                 'key-{account_id}', {publisher}, {enabled});"
    )
}

/// One import on an account, scoped to the account's expected publisher as
/// managed ingestion requires; `completed_at` is an SQL literal or `NULL`.
pub(crate) fn import_sql(
    import_id: Uuid,
    account_id: Uuid,
    status: &str,
    completed_at: &str,
) -> String {
    scoped_import_sql(
        import_id,
        account_id,
        &format!(
            "(SELECT expected_publisher_id FROM metric_source_account \
              WHERE source_account_id = '{account_id}')"
        ),
        status,
        completed_at,
    )
}

/// [`import_sql`] with an explicit publisher scope, an SQL expression or
/// `NULL`, which may contradict the account's expected publisher.
pub(crate) fn scoped_import_sql(
    import_id: Uuid,
    account_id: Uuid,
    publisher: &str,
    status: &str,
    completed_at: &str,
) -> String {
    format!(
        "INSERT INTO metric_import \
             (import_id, source_account_id, publisher_id, format_code, format_version, \
              status, normalizer_version, created_by, completed_at) \
         VALUES ('{import_id}', '{account_id}', {publisher}, 'thoth_csv', '1', '{status}', \
                 'normalizer/1', 'test', {completed_at});"
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn coverage_sql(
    account_id: Uuid,
    import_id: Uuid,
    platform_id: Uuid,
    measure_id: Uuid,
    start: NaiveDate,
    end: NaiveDate,
    status: &str,
    country: bool,
    institution: bool,
) -> String {
    format!(
        "INSERT INTO metric_coverage \
             (source_account_id, import_id, platform_id, measure_id, period_start, \
              period_end, coverage_status, country_coverage, institution_coverage) \
         VALUES ('{account_id}', '{import_id}', '{platform_id}', '{measure_id}', \
                 '{start}', '{end}', '{status}', {country}, {institution});"
    )
}

/// One canonical one-day `DAY` record with a current revision and its
/// `PENDING` work-day delta, exactly as canonical ingestion commits them.
pub(crate) fn record_sql(
    fx: &Fixture,
    record_id: Uuid,
    work_id: Uuid,
    platform_id: Uuid,
    measure_id: Uuid,
    day: NaiveDate,
    value: i64,
) -> String {
    record_dims_sql(
        fx,
        record_id,
        work_id,
        platform_id,
        measure_id,
        day,
        Dims::default(),
        value,
    )
}

/// The optional dimensions of one canonical work-day record.
#[derive(Clone, Copy, Default)]
pub(crate) struct Dims {
    pub(crate) publication: Option<Uuid>,
    pub(crate) country: Option<&'static str>,
    pub(crate) institution: Option<Uuid>,
}

fn sql_opt<T: std::fmt::Display>(value: Option<T>) -> String {
    value.map_or("NULL".to_string(), |value| format!("'{value}'"))
}

/// [`record_sql`] with optional publication, country and institution
/// dimensions.
#[allow(clippy::too_many_arguments)]
pub(crate) fn record_dims_sql(
    fx: &Fixture,
    record_id: Uuid,
    work_id: Uuid,
    platform_id: Uuid,
    measure_id: Uuid,
    day: NaiveDate,
    dims: Dims,
    value: i64,
) -> String {
    let revision_id = Uuid::new_v4();
    format!(
        "INSERT INTO metric_record \
             (record_id, identity_hash, work_id, publication_id, platform_id, measure_id, \
              period_start, period_end, reporting_grain, country_code, institution_id, \
              winning_source_account_id) \
         VALUES ('{record_id}', 'identity-{record_id}', '{work_id}', {publication}, \
                 '{platform_id}', '{measure_id}', '{day}', DATE '{day}' + 1, 'DAY', \
                 {country}, {institution}, '{account}'); \
         INSERT INTO metric_record_revision \
             (record_revision_id, record_id, revision_number, import_id, value, \
              content_hash, status) \
         VALUES ('{revision_id}', '{record_id}', 1, '{import}', {value}, \
                 'content-{revision_id}', 'CURRENT'); \
         UPDATE metric_record SET current_revision_id = '{revision_id}' \
          WHERE record_id = '{record_id}'; \
         INSERT INTO metric_rollup_delta (record_id, revision_id, delta_value, status) \
         VALUES ('{record_id}', '{revision_id}', {value}, 'PENDING');",
        publication = sql_opt(dims.publication),
        country = sql_opt(dims.country),
        institution = sql_opt(dims.institution),
        account = fx.account_id,
        import = fx.canonical_import,
    )
}

/// Commit one dimensioned canonical work-day value and its pending delta.
#[allow(clippy::too_many_arguments)]
pub(crate) fn commit_dims(
    fx: &Fixture,
    work_id: Uuid,
    platform_id: Uuid,
    measure_id: Uuid,
    day: NaiveDate,
    dims: Dims,
    value: i64,
) {
    exec(
        &fx.pool,
        &record_dims_sql(
            fx,
            Uuid::new_v4(),
            work_id,
            platform_id,
            measure_id,
            day,
            dims,
            value,
        ),
    );
}

/// Insert one publication of a work and return its id.
pub(crate) fn insert_publication(pool: &PgPool, work_id: Uuid, publication_type: &str) -> Uuid {
    let publication_id = Uuid::new_v4();
    exec(
        pool,
        &format!(
            "INSERT INTO publication (publication_id, publication_type, work_id) \
             VALUES ('{publication_id}', '{publication_type}', '{work_id}');"
        ),
    );
    publication_id
}

/// Insert one institution and return its id.
pub(crate) fn insert_institution(pool: &PgPool) -> Uuid {
    let institution_id = Uuid::new_v4();
    exec(
        pool,
        &format!(
            "INSERT INTO institution (institution_id, institution_name) \
             VALUES ('{institution_id}', 'Institution {institution_id}');"
        ),
    );
    institution_id
}

/// Commit one canonical work-day value and its pending delta.
pub(crate) fn commit(
    fx: &Fixture,
    work_id: Uuid,
    platform_id: Uuid,
    measure_id: Uuid,
    day: NaiveDate,
    value: i64,
) -> Uuid {
    let record_id = Uuid::new_v4();
    exec(
        &fx.pool,
        &record_sql(fx, record_id, work_id, platform_id, measure_id, day, value),
    );
    record_id
}

/// Commit a further revision of a record with its signed difference delta.
pub(crate) fn revise(
    fx: &Fixture,
    record_id: Uuid,
    revision_number: i32,
    new_value: i64,
    old_value: i64,
) {
    let revision_id = Uuid::new_v4();
    exec(
        &fx.pool,
        &format!(
            "UPDATE metric_record_revision SET status = 'SUPERSEDED' \
              WHERE record_id = '{record_id}' AND status = 'CURRENT'; \
             INSERT INTO metric_record_revision \
                 (record_revision_id, record_id, revision_number, import_id, value, \
                  content_hash, status) \
             VALUES ('{revision_id}', '{record_id}', {revision_number}, '{import}', \
                     {new_value}, 'content-{revision_id}', 'CURRENT'); \
             UPDATE metric_record SET current_revision_id = '{revision_id}' \
              WHERE record_id = '{record_id}'; \
             INSERT INTO metric_rollup_delta (record_id, revision_id, delta_value, status) \
             VALUES ('{record_id}', '{revision_id}', {delta}, 'PENDING');",
            import = fx.canonical_import,
            delta = new_value - old_value,
        ),
    );
}

/// Apply every pending work-day delta through the real rollup operations.
pub(crate) fn apply_all(pool: &PgPool) {
    loop {
        let claims = claim_metric_rollup_deltas(pool, APPLIER, 50).expect("claim rollup deltas");
        let Some(first) = claims.first() else {
            return;
        };
        complete_metric_rollup_deltas(pool, APPLIER, first.claim_token)
            .expect("complete rollup deltas");
    }
}

/// Insert one terminal import on the fixture account covering `measure` on
/// the fixture platform over `[start, end)`.
pub(crate) fn cover(
    fx: &Fixture,
    measure_id: Uuid,
    start: NaiveDate,
    end: NaiveDate,
    status: &str,
    import_status: &str,
    completed_at: &str,
) -> Uuid {
    let import_id = Uuid::new_v4();
    exec(
        &fx.pool,
        &format!(
            "{}{}",
            import_sql(import_id, fx.account_id, import_status, completed_at),
            coverage_sql(
                fx.account_id,
                import_id,
                fx.platform_id,
                measure_id,
                start,
                end,
                status,
                true,
                true
            )
        ),
    );
    import_id
}

/// Insert one historical unresolved-DOI quarantine observation. The import's
/// publisher is supplied explicitly so tests can prove identifier quality uses
/// immutable admitted import scope rather than current source-account or Work
/// metadata.
#[allow(clippy::too_many_arguments)]
pub(crate) fn quarantine(
    fx: &Fixture,
    publisher_id: Uuid,
    platform_id: Uuid,
    measure_id: Uuid,
    start: NaiveDate,
    end: NaiveDate,
    grain: &str,
) -> Uuid {
    let import_id = Uuid::new_v4();
    let provenance_id = Uuid::new_v4();
    let quarantine_id = Uuid::new_v4();
    let publisher = format!("'{publisher_id}'");
    exec(
        &fx.pool,
        &format!(
            "{} \
             INSERT INTO metric_record_provenance \
                 (record_provenance_id, import_id, classification, details) \
             VALUES ('{provenance_id}', '{import_id}', 'REJECTED', \
                 '{{\"schema\":\"thoth-metric-provenance-details/1\",\
                    \"reason_code\":\"UNKNOWN_DOI\",\"reporting_grain\":\"{grain}\"}}'::jsonb); \
             INSERT INTO metric_identifier_quarantine \
                 (identifier_quarantine_id, record_provenance_id, source_account_id, \
                  platform_id, measure_id, schema_version, work_doi, period_start, period_end, \
                  reporting_grain, country_code, value, methodology_version) \
             VALUES ('{quarantine_id}', '{provenance_id}', '{}', '{platform_id}', \
                     '{measure_id}', 'thoth-normalized-metrics/1', \
                     'https://doi.org/10.12345/dashboard-{quarantine_id}', \
                     '{start}', '{end}', '{grain}', NULL, 1, 'dashboard-test/1');",
            scoped_import_sql(
                import_id,
                fx.account_id,
                &publisher,
                "COMPLETED_WITH_ERRORS",
                "'2026-03-10T00:00:00Z'",
            ),
            fx.account_id,
        ),
    );
    quarantine_id
}

/// Replace the reconciliation row for one quarantine observation with an exact
/// state. Terminal states require a canonical record whose current revision is
/// used only to satisfy #935's durable terminal-state foreign-key shape.
pub(crate) fn set_quarantine_reconciliation_state(
    fx: &Fixture,
    quarantine_id: Uuid,
    state: &str,
    record_id: Option<Uuid>,
) {
    exec(
        &fx.pool,
        &format!(
            "DELETE FROM metric_identifier_quarantine_reconciliation \
              WHERE identifier_quarantine_id = '{quarantine_id}';"
        ),
    );
    if state.starts_with("RESOLVED_") {
        let record_id = record_id.expect("terminal reconciliation needs a canonical record");
        exec(
            &fx.pool,
            &format!(
                "INSERT INTO metric_identifier_quarantine_reconciliation \
                     (identifier_quarantine_id, state, attempt_count, last_attempted_by, \
                      first_attempt_at, last_attempt_at, next_attempt_at, resolved_at, \
                      record_id, record_revision_id) \
                 VALUES ('{quarantine_id}', '{state}', 1, 'dashboard-quality-test', \
                         transaction_timestamp(), transaction_timestamp(), NULL, \
                         transaction_timestamp(), '{record_id}', \
                         (SELECT current_revision_id FROM metric_record \
                           WHERE record_id = '{record_id}'));"
            ),
        );
    } else {
        assert!(
            record_id.is_none(),
            "nonterminal reconciliation must not identify a canonical record"
        );
        exec(
            &fx.pool,
            &format!(
                "INSERT INTO metric_identifier_quarantine_reconciliation \
                     (identifier_quarantine_id, state, attempt_count, last_attempted_by, \
                      first_attempt_at, last_attempt_at, next_attempt_at, resolved_at, \
                      record_id, record_revision_id) \
                 VALUES ('{quarantine_id}', '{state}', 1, 'dashboard-quality-test', \
                         transaction_timestamp(), transaction_timestamp(), \
                         transaction_timestamp() + interval '1 hour', NULL, NULL, NULL);"
            ),
        );
    }
}

/// A pristine registry plus two entitled publishers, two platforms and one
/// eligible managed source account.
pub(crate) fn setup() -> (TestDbGuard, Fixture) {
    let (guard, pool) = setup_registry_db();
    let publisher_id = Uuid::new_v4();
    let other_publisher_id = Uuid::new_v4();
    let works = vec![Uuid::new_v4(), Uuid::new_v4()];
    let other_work = Uuid::new_v4();
    let platform_id = Uuid::new_v4();
    let other_platform_id = Uuid::new_v4();
    insert_platform_row(&pool, platform_id, "dashboard_platform_a");
    insert_platform_row(&pool, other_platform_id, "dashboard_platform_b");
    let source_id = Uuid::new_v4();
    let account_id = Uuid::new_v4();
    let canonical_import = Uuid::new_v4();
    exec(
        &pool,
        &format!(
            "{}{}{}{}{}{}{}",
            publisher_sql(publisher_id, "SPHINX"),
            publisher_sql(other_publisher_id, "SPHINX"),
            works_sql(publisher_id, &works),
            works_sql(other_publisher_id, &[other_work]),
            source_sql(source_id, "DRIVER", true),
            account_sql(account_id, source_id, platform_id, Some(publisher_id), true),
            import_sql(canonical_import, account_id, "PROCESSING", "NULL"),
        ),
    );
    let measure = |code: &str| {
        let mut connection = pool.get().expect("Failed to get DB connection");
        #[derive(diesel::QueryableByName)]
        struct Row {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            measure_id: Uuid,
        }
        sql_query(format!(
            "SELECT measure_id FROM metric_measure WHERE code = '{code}'"
        ))
        .get_result::<Row>(&mut connection)
        .expect("a seeded measure")
        .measure_id
    };
    let fixture = Fixture {
        sessions: measure("title_sessions"),
        units: measure("net_units"),
        pool,
        publisher_id,
        works,
        other_publisher_id,
        other_work,
        platform_id,
        other_platform_id,
        account_id,
        canonical_import,
    };
    (guard, fixture)
}

/// Insert one additional measure and return its id.
pub(crate) fn insert_measure(
    pool: &PgPool,
    code: &str,
    across_time: bool,
    across_works: bool,
) -> Uuid {
    let measure_id = Uuid::new_v4();
    exec(
        pool,
        &format!(
            "INSERT INTO metric_measure \
                 (measure_id, code, display_name, category, unit, allow_negative, \
                  additive_across_time, additive_across_works, definition, enabled) \
             VALUES ('{measure_id}', '{code}', 'Measure {code}', 'USAGE', 'COUNT', FALSE, \
                     {across_time}, {across_works}, 'What {code} counts.', TRUE);"
        ),
    );
    measure_id
}

/// Insert one projection row directly, for shapes the canonical path cannot
/// cheaply reach (values at the `BIGINT` limits, very large fixtures).
pub(crate) fn projection_sql(
    work_id: Uuid,
    platform_id: Uuid,
    measure_id: Uuid,
    day: NaiveDate,
    country: Option<&str>,
    value: i64,
) -> String {
    let country = country.map_or("NULL".to_string(), |code| format!("'{code}'"));
    format!(
        "INSERT INTO metric_rollup_work_day \
             (work_id, platform_id, measure_id, day, country_code, value, watermark) \
         VALUES ('{work_id}', '{platform_id}', '{measure_id}', '{day}', {country}, {value}, 1);"
    )
}

pub(crate) fn request(
    publisher_id: Uuid,
    start: NaiveDate,
    end: NaiveDate,
    platforms: &[Uuid],
    measures: &[Uuid],
    grain: Option<MetricTimelineGrain>,
) -> MetricDashboardInput {
    MetricDashboardInput {
        selector: MetricSelectorInput {
            publisher_ids: Some(vec![publisher_id]),
        },
        start_date: start,
        end_date: end,
        measures: Some(measures.to_vec()),
        platforms: Some(platforms.to_vec()),
        timeline_grain: grain,
    }
}

/// The default window `[2026-03-01, 2026-03-05)` for one platform/measure.
fn window(fx: &Fixture, measures: &[Uuid]) -> MetricDashboardInput {
    request(
        fx.publisher_id,
        d1(),
        day_n(5),
        &[fx.platform_id],
        measures,
        None,
    )
}

pub(crate) fn text(value: &Option<BigInt>) -> Option<String> {
    value.map(|value| value.to_string())
}

fn bucket_values(dashboard: &MetricDashboard, measure_id: Uuid) -> Vec<Option<String>> {
    dashboard
        .timeline
        .iter()
        .filter(|bucket| bucket.measure_id == measure_id)
        .map(|bucket| text(&bucket.value))
        .collect()
}

fn total(dashboard: &MetricDashboard, platform_id: Uuid, measure_id: Uuid) -> Option<String> {
    let found: Vec<_> = dashboard
        .totals
        .iter()
        .filter(|total| total.platform_id == platform_id && total.measure_id == measure_id)
        .collect();
    assert_eq!(found.len(), 1, "exactly one total per combination");
    text(&found[0].value)
}

fn item(dashboard: &MetricDashboard, measure_id: Uuid) -> &MetricCoverageItem {
    dashboard
        .coverage
        .items
        .iter()
        .find(|item| item.measure_id == measure_id)
        .expect("a coverage item for the measure")
}

fn codes(dashboard: &MetricDashboard) -> Vec<MetricWarningCode> {
    dashboard
        .warnings
        .iter()
        .map(|warning| warning.code)
        .collect()
}

fn read(fx: &Fixture, input: &MetricDashboardInput) -> MetricDashboard {
    metric_dashboard(&fx.pool, input).unwrap_or_else(|error| panic!("dashboard failed: {error:?}"))
}

fn some(value: &str) -> Option<String> {
    Some(value.to_string())
}

// ==========================================================================
// Totals and timeline
// ==========================================================================

#[test]
fn totals_and_timeline_are_exact_per_platform_and_measure_within_the_publisher() {
    let (_guard, fx) = setup();
    let [first, second] = [fx.works[0], fx.works[1]];
    commit(&fx, first, fx.platform_id, fx.sessions, day_n(1), 10);
    commit(&fx, second, fx.platform_id, fx.sessions, day_n(1), 5);
    commit(&fx, first, fx.platform_id, fx.sessions, day_n(3), 7);
    commit(&fx, first, fx.platform_id, fx.units, day_n(1), 100);
    // Another platform and another publisher: neither may leak into the
    // selected totals.
    commit(
        &fx,
        first,
        fx.other_platform_id,
        fx.sessions,
        day_n(1),
        1_000,
    );
    commit(
        &fx,
        fx.other_work,
        fx.platform_id,
        fx.sessions,
        day_n(1),
        99_999,
    );
    apply_all(&fx.pool);
    for measure in [fx.sessions, fx.units] {
        cover(
            &fx,
            measure,
            d1(),
            day_n(5),
            "COMPLETE",
            "COMPLETED",
            "'2026-03-10T00:00:00Z'",
        );
    }

    let dashboard = read(&fx, &window(&fx, &[fx.sessions, fx.units]));

    // Totals stay separate per measure, and the order is deterministic.
    assert_eq!(total(&dashboard, fx.platform_id, fx.sessions), some("22"));
    assert_eq!(total(&dashboard, fx.platform_id, fx.units), some("100"));
    let order: Vec<(Uuid, Uuid)> = dashboard
        .totals
        .iter()
        .map(|total| (total.platform_id, total.measure_id))
        .collect();
    let mut sorted = order.clone();
    sorted.sort();
    assert_eq!(order, sorted);
    assert_eq!(
        order.len(),
        2,
        "no cross-measure or cross-platform total exists"
    );

    // Complete days without rows are real zeros.
    assert_eq!(
        bucket_values(&dashboard, fx.sessions),
        vec![some("15"), some("0"), some("7"), some("0")]
    );
    assert_eq!(
        bucket_values(&dashboard, fx.units),
        vec![some("100"), some("0"), some("0"), some("0")]
    );
    let first_bucket = &dashboard.timeline[0];
    assert_eq!(
        (first_bucket.start_date, first_bucket.end_date),
        (d1(), day_n(2))
    );
    assert_eq!(dashboard.coverage.status, MetricCoverageStatus::Complete);
    assert_eq!(dashboard.data_through, Some(day_n(4)));
    assert!(dashboard.warnings.is_empty());
    assert!(!dashboard.is_partial);

    // Filtering to the other platform serves only that platform, and with no
    // eligible account there its coverage is unknown: the value it has is
    // exact, and the days without one are not zero.
    let other = read(
        &fx,
        &request(
            fx.publisher_id,
            d1(),
            day_n(5),
            &[fx.other_platform_id],
            &[fx.sessions],
            None,
        ),
    );
    assert_eq!(
        total(&other, fx.other_platform_id, fx.sessions),
        some("1000")
    );
    assert_eq!(
        bucket_values(&other, fx.sessions),
        vec![some("1000"), None, None, None]
    );
    assert_eq!(other.coverage.status, MetricCoverageStatus::Unknown);
    assert_eq!(codes(&other), vec![MetricWarningCode::UnknownCoverage]);

    // The other publisher sees only its own work.
    let theirs = read(
        &fx,
        &request(
            fx.other_publisher_id,
            d1(),
            day_n(5),
            &[fx.platform_id],
            &[fx.sessions],
            None,
        ),
    );
    assert_eq!(total(&theirs, fx.platform_id, fx.sessions), some("99999"));
}

#[test]
fn attribution_follows_current_work_ownership() {
    let (_guard, fx) = setup();
    commit(&fx, fx.works[0], fx.platform_id, fx.sessions, day_n(1), 12);
    apply_all(&fx.pool);
    let input = window(&fx, &[fx.sessions]);
    assert_eq!(
        total(&read(&fx, &input), fx.platform_id, fx.sessions),
        some("12")
    );

    // Move the work to the other publisher's imprint. The canonical record and
    // its projected history are untouched, and the history follows the work.
    exec(
        &fx.pool,
        &format!(
            "UPDATE work SET imprint_id = \
                 (SELECT imprint_id FROM work WHERE work_id = '{other}') \
             WHERE work_id = '{moved}';",
            other = fx.other_work,
            moved = fx.works[0],
        ),
    );
    assert_eq!(total(&read(&fx, &input), fx.platform_id, fx.sessions), None);
    let mut theirs = input.clone();
    theirs.selector.publisher_ids = Some(vec![fx.other_publisher_id]);
    assert_eq!(
        total(&read(&fx, &theirs), fx.platform_id, fx.sessions),
        some("12")
    );
}

#[test]
fn month_buckets_are_exact_sums_of_daily_rows_clipped_to_the_range() {
    let (_guard, fx) = setup();
    for (day, value) in [
        (d(2026, 1, 31), 3),
        (d(2026, 2, 15), 4),
        (d(2026, 2, 28), 5),
        (d(2026, 3, 1), 6),
    ] {
        commit(&fx, fx.works[0], fx.platform_id, fx.sessions, day, value);
    }
    apply_all(&fx.pool);
    let (start, end) = (d(2026, 1, 30), d(2026, 3, 2));
    cover(
        &fx,
        fx.sessions,
        start,
        end,
        "COMPLETE",
        "COMPLETED",
        "'2026-03-10T00:00:00Z'",
    );

    let month = read(
        &fx,
        &request(
            fx.publisher_id,
            start,
            end,
            &[fx.platform_id],
            &[fx.sessions],
            Some(MetricTimelineGrain::Month),
        ),
    );
    let ranges: Vec<(NaiveDate, NaiveDate)> = month
        .timeline
        .iter()
        .map(|bucket| (bucket.start_date, bucket.end_date))
        .collect();
    assert_eq!(
        ranges,
        vec![
            (d(2026, 1, 30), d(2026, 2, 1)),
            (d(2026, 2, 1), d(2026, 3, 1)),
            (d(2026, 3, 1), d(2026, 3, 2)),
        ]
    );
    assert_eq!(
        bucket_values(&month, fx.sessions),
        vec![some("3"), some("9"), some("6")]
    );
    assert_eq!(total(&month, fx.platform_id, fx.sessions), some("18"));

    // AUTO is DAY for MOM-1, bucket for bucket.
    let auto = read(
        &fx,
        &request(
            fx.publisher_id,
            start,
            end,
            &[fx.platform_id],
            &[fx.sessions],
            Some(MetricTimelineGrain::Auto),
        ),
    );
    let day = read(
        &fx,
        &request(
            fx.publisher_id,
            start,
            end,
            &[fx.platform_id],
            &[fx.sessions],
            Some(MetricTimelineGrain::Day),
        ),
    );
    assert_eq!(auto.timeline, day.timeline);
    // 30 and 31 January, the whole of February and 1 March.
    assert_eq!(day.timeline.len(), 2 + 28 + 1);
    let omitted = read(
        &fx,
        &request(
            fx.publisher_id,
            start,
            end,
            &[fx.platform_id],
            &[fx.sessions],
            None,
        ),
    );
    assert_eq!(omitted.timeline, day.timeline);
}

#[test]
fn a_revision_is_reflected_exactly_once_after_rollup_application() {
    let (_guard, fx) = setup();
    cover(
        &fx,
        fx.sessions,
        d1(),
        day_n(5),
        "COMPLETE",
        "COMPLETED",
        "'2026-03-10T00:00:00Z'",
    );
    let record = commit(&fx, fx.works[0], fx.platform_id, fx.sessions, day_n(2), 10);
    apply_all(&fx.pool);
    let input = window(&fx, &[fx.sessions]);
    assert_eq!(
        total(&read(&fx, &input), fx.platform_id, fx.sessions),
        some("10")
    );

    // The source corrects the figure upwards. Until the +15 delta is applied
    // the served value is the applied 10, flagged as lagging.
    revise(&fx, record, 2, 25, 10);
    let lagging = read(&fx, &input);
    assert_eq!(total(&lagging, fx.platform_id, fx.sessions), some("10"));
    assert_eq!(codes(&lagging), vec![MetricWarningCode::RollupLag]);
    assert!(lagging.is_partial);
    assert_eq!(item(&lagging, fx.sessions).data_through, Some(day_n(1)));

    apply_all(&fx.pool);
    let applied = read(&fx, &input);
    assert_eq!(total(&applied, fx.platform_id, fx.sessions), some("25"));
    assert_eq!(
        bucket_values(&applied, fx.sessions),
        vec![some("0"), some("25"), some("0"), some("0")]
    );
    assert!(applied.warnings.is_empty());
    // Reading again, and applying again, changes nothing.
    apply_all(&fx.pool);
    let again = read(&fx, &input);
    assert_eq!(again.totals, applied.totals);
    assert_eq!(again.timeline, applied.timeline);
}

#[test]
fn signed_values_and_totals_beyond_every_narrower_integer_are_exact() {
    let (_guard, fx) = setup();
    cover(
        &fx,
        fx.units,
        d1(),
        day_n(5),
        "COMPLETE",
        "COMPLETED",
        "'2026-03-10T00:00:00Z'",
    );
    commit(&fx, fx.works[0], fx.platform_id, fx.units, day_n(1), -7);
    commit(&fx, fx.works[1], fx.platform_id, fx.units, day_n(1), 3);
    apply_all(&fx.pool);
    // Values at the BIGINT limits, written to the projection directly: two
    // maxima sum beyond i64, and two minima below it.
    exec(
        &fx.pool,
        &format!(
            "{}{}{}{}",
            projection_sql(
                fx.works[0],
                fx.platform_id,
                fx.units,
                day_n(2),
                None,
                i64::MAX
            ),
            projection_sql(
                fx.works[1],
                fx.platform_id,
                fx.units,
                day_n(2),
                None,
                i64::MAX
            ),
            projection_sql(
                fx.works[0],
                fx.platform_id,
                fx.units,
                day_n(3),
                None,
                i64::MIN
            ),
            projection_sql(
                fx.works[1],
                fx.platform_id,
                fx.units,
                day_n(3),
                None,
                i64::MIN
            ),
        ),
    );

    let dashboard = read(&fx, &window(&fx, &[fx.units]));
    let max_pair = i128::from(i64::MAX) * 2;
    let min_pair = i128::from(i64::MIN) * 2;
    assert_eq!(
        bucket_values(&dashboard, fx.units),
        vec![
            some("-4"),
            Some(max_pair.to_string()),
            Some(min_pair.to_string()),
            some("0"),
        ]
    );
    assert_eq!(max_pair.to_string(), "18446744073709551614");
    assert_eq!(min_pair.to_string(), "-18446744073709551616");
    assert_eq!(
        total(&dashboard, fx.platform_id, fx.units),
        Some((-4 + max_pair + min_pair).to_string())
    );
}

#[test]
fn an_unrepresentable_aggregate_fails_rather_than_wrapping() {
    let cells = Cells {
        sums: vec![Some(BigInt::new(i128::MAX)), Some(BigInt::new(1))],
        coverage: vec![None, None],
        lag: vec![false, false],
        identifier_incomplete: vec![false, false],
        depends_on_country: false,
        depends_on_institution: false,
    };
    assert_eq!(
        cells.value(0, 2),
        Err(MetricReadError::QueryLimitExceeded(AGGREGATE_OUT_OF_RANGE))
    );
    assert_eq!(cells.value(0, 1), Ok(Some(BigInt::new(i128::MAX))));
    assert_eq!(
        BigInt::parse_canonical("170141183460469231731687303715884105728"),
        None
    );
}

// ==========================================================================
// Identifier quality (MET-WP7-PREREQ-04)
// ==========================================================================

#[test]
fn unresolved_identifier_evidence_suppresses_only_unjustified_zeroes() {
    let (_guard, fx) = setup();
    cover(
        &fx,
        fx.sessions,
        d1(),
        day_n(5),
        "COMPLETE",
        "COMPLETED",
        "'2026-03-10T00:00:00Z'",
    );
    quarantine(
        &fx,
        fx.publisher_id,
        fx.platform_id,
        fx.sessions,
        day_n(2),
        day_n(3),
        "DAY",
    );

    let dashboard = read(&fx, &window(&fx, &[fx.sessions]));
    assert_eq!(
        bucket_values(&dashboard, fx.sessions),
        vec![some("0"), None, some("0"), some("0")],
        "an unresolved empty day is unknown rather than a fabricated zero"
    );
    assert_eq!(
        total(&dashboard, fx.platform_id, fx.sessions),
        None,
        "the whole-range empty total is unknown when any day is identifier-incomplete"
    );
    assert_eq!(dashboard.coverage.status, MetricCoverageStatus::Complete);
    assert_eq!(item(&dashboard, fx.sessions).status, MetricCoverageStatus::Complete);
    assert_eq!(dashboard.data_through, Some(day_n(4)));
    assert_eq!(
        codes(&dashboard),
        vec![MetricWarningCode::UnresolvedIdentifiers]
    );
    assert_eq!(
        dashboard.warnings[0].message,
        UNRESOLVED_IDENTIFIERS_MESSAGE
    );
    assert!(dashboard.is_partial);
}

#[test]
fn every_nonterminal_state_warns_and_every_terminal_state_restores_zero_eligibility() {
    let (_guard, fx) = setup();
    cover(
        &fx,
        fx.sessions,
        d1(),
        day_n(5),
        "COMPLETE",
        "COMPLETED",
        "'2026-03-10T00:00:00Z'",
    );
    let quarantine_id = quarantine(
        &fx,
        fx.publisher_id,
        fx.platform_id,
        fx.sessions,
        day_n(2),
        day_n(3),
        "DAY",
    );
    let input = window(&fx, &[fx.sessions]);

    let never_attempted = read(&fx, &input);
    assert_eq!(
        codes(&never_attempted),
        vec![MetricWarningCode::UnresolvedIdentifiers]
    );
    assert_eq!(
        bucket_values(&never_attempted, fx.sessions),
        vec![some("0"), None, some("0"), some("0")]
    );

    for state in [
        "PENDING_UNKNOWN_DOI",
        "BLOCKED_AMBIGUOUS_DOI",
        "BLOCKED_PUBLISHER_SCOPE_MISMATCH",
        "BLOCKED_SOURCE_CONFLICT",
        "BLOCKED_OVERLAPPING_PERIOD",
        "BLOCKED_SAME_IMPORT_ORDER",
        "BLOCKED_IMPORT_ORDER_AMBIGUOUS",
        "BLOCKED_DELTA_OVERFLOW",
        "BLOCKED_INCONSISTENT_EVIDENCE",
    ] {
        set_quarantine_reconciliation_state(&fx, quarantine_id, state, None);
        let dashboard = read(&fx, &input);
        assert_eq!(
            codes(&dashboard),
            vec![MetricWarningCode::UnresolvedIdentifiers],
            "{state}"
        );
        assert_eq!(bucket_values(&dashboard, fx.sessions)[1], None, "{state}");
    }

    let support = commit(
        &fx,
        fx.works[0],
        fx.platform_id,
        fx.units,
        day_n(10),
        1,
    );
    for state in [
        "RESOLVED_WINNER",
        "RESOLVED_DUPLICATE",
        "RESOLVED_REVISION",
        "RESOLVED_SUPERSEDED",
    ] {
        set_quarantine_reconciliation_state(&fx, quarantine_id, state, Some(support));
        let dashboard = read(&fx, &input);
        assert!(dashboard.warnings.is_empty(), "{state}");
        assert!(!dashboard.is_partial, "{state}");
        assert_eq!(
            bucket_values(&dashboard, fx.sessions),
            vec![some("0"), some("0"), some("0"), some("0")],
            "{state}"
        );
        assert_eq!(
            total(&dashboard, fx.platform_id, fx.sessions),
            some("0"),
            "{state}"
        );
    }
}

#[test]
fn reporting_period_identifier_evidence_marks_every_overlapped_day() {
    let (_guard, fx) = setup();
    cover(
        &fx,
        fx.sessions,
        d1(),
        day_n(5),
        "COMPLETE",
        "COMPLETED",
        "'2026-03-10T00:00:00Z'",
    );
    quarantine(
        &fx,
        fx.publisher_id,
        fx.platform_id,
        fx.sessions,
        day_n(2),
        day_n(4),
        "REPORTING_PERIOD",
    );

    let dashboard = read(&fx, &window(&fx, &[fx.sessions]));
    assert_eq!(
        bucket_values(&dashboard, fx.sessions),
        vec![some("0"), None, None, some("0")]
    );
    assert_eq!(
        codes(&dashboard),
        vec![MetricWarningCode::UnresolvedIdentifiers]
    );
}

#[test]
fn projected_values_remain_exact_and_warning_order_is_deterministic() {
    let (_guard, fx) = setup();
    cover(
        &fx,
        fx.sessions,
        d1(),
        day_n(5),
        "PARTIAL",
        "COMPLETED",
        "'2026-03-10T00:00:00Z'",
    );
    let record = commit(
        &fx,
        fx.works[0],
        fx.platform_id,
        fx.sessions,
        day_n(2),
        9,
    );
    apply_all(&fx.pool);
    revise(&fx, record, 2, 11, 9);

    let input = window(&fx, &[fx.sessions, fx.units]);
    let before = read(&fx, &input);
    assert_eq!(
        codes(&before),
        vec![
            MetricWarningCode::UnknownCoverage,
            MetricWarningCode::PartialCoverage,
            MetricWarningCode::RollupLag,
        ]
    );
    assert_eq!(total(&before, fx.platform_id, fx.sessions), some("9"));

    quarantine(
        &fx,
        fx.publisher_id,
        fx.platform_id,
        fx.sessions,
        day_n(2),
        day_n(3),
        "DAY",
    );
    let after = read(&fx, &input);
    assert_eq!(
        total(&after, fx.platform_id, fx.sessions),
        some("9"),
        "projected canonical sums are never replaced by null"
    );
    assert_eq!(
        codes(&after),
        vec![
            MetricWarningCode::UnknownCoverage,
            MetricWarningCode::PartialCoverage,
            MetricWarningCode::UnresolvedIdentifiers,
            MetricWarningCode::RollupLag,
        ]
    );
    assert_eq!(after.coverage, before.coverage);
    assert_eq!(after.data_through, before.data_through);
    assert_eq!(after.rollup_watermark, before.rollup_watermark);
}

#[test]
fn unresolved_scope_uses_immutable_import_publisher_and_ignores_source_disablement() {
    let (_guard, fx) = setup();
    cover(
        &fx,
        fx.sessions,
        d1(),
        day_n(5),
        "COMPLETE",
        "COMPLETED",
        "'2026-03-10T00:00:00Z'",
    );

    // Other publisher, other platform and non-overlapping evidence are all
    // irrelevant to the selected explicit cell.
    quarantine(
        &fx,
        fx.other_publisher_id,
        fx.platform_id,
        fx.sessions,
        day_n(2),
        day_n(3),
        "DAY",
    );
    quarantine(
        &fx,
        fx.publisher_id,
        fx.other_platform_id,
        fx.sessions,
        day_n(2),
        day_n(3),
        "DAY",
    );
    quarantine(
        &fx,
        fx.publisher_id,
        fx.platform_id,
        fx.sessions,
        day_n(10),
        day_n(11),
        "DAY",
    );
    let input = window(&fx, &[fx.sessions]);
    assert!(
        read(&fx, &input).warnings.is_empty(),
        "out-of-scope unresolved evidence must not leak into the request"
    );

    let relevant = quarantine(
        &fx,
        fx.publisher_id,
        fx.platform_id,
        fx.sessions,
        day_n(2),
        day_n(3),
        "DAY",
    );
    assert_eq!(
        codes(&read(&fx, &input)),
        vec![MetricWarningCode::UnresolvedIdentifiers]
    );

    // Historical admitted import scope stays authoritative even after the
    // source account changes publisher and the source/account are disabled.
    exec(
        &fx.pool,
        &format!(
            "UPDATE metric_source_account \
                SET expected_publisher_id = '{}', enabled = FALSE \
              WHERE source_account_id = '{}'; \
             UPDATE metric_source SET enabled = FALSE \
              WHERE source_id = (SELECT source_id FROM metric_source_account \
                                  WHERE source_account_id = '{}');",
            fx.other_publisher_id, fx.account_id, fx.account_id
        ),
    );
    let after_disable = read(&fx, &input);
    assert!(codes(&after_disable).contains(&MetricWarningCode::UnresolvedIdentifiers));
    assert_eq!(
        total(&after_disable, fx.platform_id, fx.sessions),
        None,
        "coverage may become unknown, but immutable unresolved evidence still blocks zero"
    );

    // Moving the current account scope to the other publisher does not move
    // the historical quarantine import with it.
    let other_input = request(
        fx.other_publisher_id,
        d1(),
        day_n(5),
        &[fx.platform_id],
        &[fx.sessions],
        None,
    );
    assert!(
        !codes(&read(&fx, &other_input)).contains(&MetricWarningCode::UnresolvedIdentifiers),
        "identifier quality is attributed by immutable import publisher"
    );

    // Keep the variable load-bearing for the fixture and prove the unresolved
    // row itself was not rewritten by any read.
    assert_eq!(
        scalar_i64(
            &fx.pool,
            &format!(
                "(SELECT COUNT(*) FROM metric_identifier_quarantine \
                  WHERE identifier_quarantine_id = '{relevant}')"
            )
        ),
        1
    );
}

#[test]
fn quarantine_only_pairs_expand_omitted_scope_conservatively_and_obey_bounds() {
    {
        let (_guard, fx) = setup();
        quarantine(
            &fx,
            fx.publisher_id,
            fx.platform_id,
            fx.units,
            day_n(2),
            day_n(3),
            "DAY",
        );
        let mut input = request(
            fx.publisher_id,
            d1(),
            day_n(5),
            &[fx.platform_id],
            &[],
            None,
        );
        input.measures = None;
        let dashboard = read(&fx, &input);
        assert_eq!(dashboard.totals.len(), 1);
        assert_eq!(dashboard.totals[0].measure_id, fx.units);
        assert_eq!(text(&dashboard.totals[0].value), None);
        assert_eq!(
            codes(&dashboard),
            vec![
                MetricWarningCode::UnknownCoverage,
                MetricWarningCode::UnresolvedIdentifiers,
            ]
        );
    }

    {
        let (_guard, fx) = setup();
        for index in 0..=METRIC_DASHBOARD_MAX_MEASURES {
            let measure = insert_measure(
                &fx.pool,
                &format!("quarantine_only_{index}"),
                true,
                true,
            );
            quarantine(
                &fx,
                fx.publisher_id,
                fx.platform_id,
                measure,
                day_n(2),
                day_n(3),
                "DAY",
            );
        }
        let mut input = request(
            fx.publisher_id,
            d1(),
            day_n(5),
            &[fx.platform_id],
            &[],
            None,
        );
        input.measures = None;
        assert_eq!(
            metric_dashboard(&fx.pool, &input),
            Err(MetricReadError::QueryLimitExceeded(TOO_MANY_MEASURES)),
            "quarantine-derived scope must fail closed rather than truncate"
        );
    }
}

// ==========================================================================
// Dimensional representation (Specification Amendment 6)
// ==========================================================================

/// Serve exactly one day of the fixture sessions measure.
fn one_day(fx: &Fixture, day: NaiveDate) -> MetricDashboardInput {
    request(
        fx.publisher_id,
        day,
        day + Duration::days(1),
        &[fx.platform_id],
        &[fx.sessions],
        None,
    )
}

#[test]
fn each_base_cell_is_served_from_exactly_one_dimensional_representation() {
    let (_guard, fx) = setup();
    let work = fx.works[0];
    let pdf = insert_publication(&fx.pool, work, "PDF");
    let epub = insert_publication(&fx.pool, work, "Epub");
    let (harvard, oxford) = (insert_institution(&fx.pool), insert_institution(&fx.pool));
    let none = Dims::default();
    let country = |code| Dims {
        country: Some(code),
        ..Dims::default()
    };
    let institution = |id| Dims {
        institution: Some(id),
        ..Dims::default()
    };
    let publication = |id| Dims {
        publication: Some(id),
        ..Dims::default()
    };
    let at = |day: i64, dims: Dims, value: i64| {
        commit_dims(
            &fx,
            work,
            fx.platform_id,
            fx.sessions,
            day_n(day),
            dims,
            value,
        )
    };

    // Day 1: an undimensioned aggregate alongside a country breakdown.
    at(1, none, 10);
    at(1, country("GB"), 6);
    at(1, country("US"), 4);
    // Day 2: an aggregate alongside an institution breakdown.
    at(2, none, 7);
    at(2, institution(harvard), 7);
    // Day 3: an aggregate alongside a publication breakdown.
    at(3, none, 5);
    at(3, publication(pdf), 5);
    // Day 4: country rows only.
    at(4, country("GB"), 3);
    at(4, country("US"), 4);
    // Day 5: institution rows only.
    at(5, institution(harvard), 2);
    at(5, institution(oxford), 5);
    // Day 6: publication rows only.
    at(6, publication(pdf), 1);
    at(6, publication(epub), 2);
    // Day 7: one consistent combined mask, publication and country.
    for (publication_id, code, value) in [(pdf, "GB", 1), (pdf, "US", 2), (epub, "GB", 3)] {
        at(
            7,
            Dims {
                publication: Some(publication_id),
                country: Some(code),
                institution: None,
            },
            value,
        );
    }
    // Day 8: different works may use different representations; each base
    // cell is resolved on its own.
    commit_dims(
        &fx,
        fx.works[0],
        fx.platform_id,
        fx.sessions,
        day_n(8),
        country("GB"),
        11,
    );
    commit_dims(
        &fx,
        fx.works[1],
        fx.platform_id,
        fx.sessions,
        day_n(8),
        institution(oxford),
        13,
    );
    apply_all(&fx.pool);
    cover(
        &fx,
        fx.sessions,
        d1(),
        day_n(9),
        "COMPLETE",
        "COMPLETED",
        "'2026-03-10T00:00:00Z'",
    );

    for (day, expected) in [
        (1, "10"),
        (2, "7"),
        (3, "5"),
        (4, "7"),
        (5, "7"),
        (6, "3"),
        (7, "6"),
        (8, "24"),
    ] {
        let dashboard = read(&fx, &one_day(&fx, day_n(day)));
        assert_eq!(
            total(&dashboard, fx.platform_id, fx.sessions),
            some(expected),
            "day {day}"
        );
    }
    let week = read(
        &fx,
        &request(
            fx.publisher_id,
            d1(),
            day_n(9),
            &[fx.platform_id],
            &[fx.sessions],
            None,
        ),
    );
    assert_eq!(
        bucket_values(&week, fx.sessions),
        ["10", "7", "5", "7", "7", "3", "6", "24"]
            .map(some)
            .to_vec(),
        "an aggregate is never added to its own breakdown"
    );
    assert_eq!(total(&week, fx.platform_id, fx.sessions), some("69"));
    assert_eq!(week.coverage.status, MetricCoverageStatus::Complete);
}

#[test]
fn mixed_dimensional_representations_without_an_aggregate_fail_closed() {
    let (_guard, fx) = setup();
    let work = fx.works[0];
    let institution_id = insert_institution(&fx.pool);
    let pdf = insert_publication(&fx.pool, work, "PDF");
    cover(
        &fx,
        fx.sessions,
        d1(),
        day_n(5),
        "COMPLETE",
        "COMPLETED",
        "'2026-03-10T00:00:00Z'",
    );
    let mixed: [(i64, [Dims; 2]); 3] = [
        // Country rows and institution rows.
        (
            1,
            [
                Dims {
                    country: Some("GB"),
                    ..Dims::default()
                },
                Dims {
                    institution: Some(institution_id),
                    ..Dims::default()
                },
            ],
        ),
        // Country rows and country-with-institution rows.
        (
            2,
            [
                Dims {
                    country: Some("GB"),
                    ..Dims::default()
                },
                Dims {
                    country: Some("US"),
                    institution: Some(institution_id),
                    ..Dims::default()
                },
            ],
        ),
        // Publication rows and publication-with-country rows.
        (
            3,
            [
                Dims {
                    publication: Some(pdf),
                    ..Dims::default()
                },
                Dims {
                    publication: Some(pdf),
                    country: Some("GB"),
                    ..Dims::default()
                },
            ],
        ),
    ];
    for (day, dims) in mixed {
        for representation in dims {
            commit_dims(
                &fx,
                work,
                fx.platform_id,
                fx.sessions,
                day_n(day),
                representation,
                5,
            );
        }
    }
    apply_all(&fx.pool);

    for day in 1..=3 {
        assert_eq!(
            metric_dashboard(&fx.pool, &one_day(&fx, day_n(day))),
            Err(MetricReadError::DimensionScopeAmbiguous),
            "day {day} is ambiguous and must not be guessed, chosen or summed"
        );
    }
    // One ambiguous base cell fails the whole request that includes it.
    let range = request(
        fx.publisher_id,
        d1(),
        day_n(5),
        &[fx.platform_id],
        &[fx.sessions],
        None,
    );
    assert_eq!(
        metric_dashboard(&fx.pool, &range),
        Err(MetricReadError::DimensionScopeAmbiguous)
    );
    assert_eq!(
        MetricReadError::DimensionScopeAmbiguous.code(),
        "MOM1_DIMENSION_SCOPE_AMBIGUOUS"
    );
    // A day outside the ambiguity is still served.
    assert_eq!(
        total(
            &read(&fx, &one_day(&fx, day_n(4))),
            fx.platform_id,
            fx.sessions
        ),
        some("0")
    );

    // An undimensioned aggregate resolves the ambiguity for its base cell.
    for day in 1..=3 {
        commit_dims(
            &fx,
            work,
            fx.platform_id,
            fx.sessions,
            day_n(day),
            Dims::default(),
            8,
        );
    }
    apply_all(&fx.pool);
    let resolved = read(&fx, &range);
    assert_eq!(
        bucket_values(&resolved, fx.sessions),
        vec![some("8"), some("8"), some("8"), some("0")]
    );
}

/// One terminal import covering days `from..to` of the fixture sessions
/// measure with the given dimension flags.
fn cover_dimensions(fx: &Fixture, from: i64, to: i64, country: bool, institution: bool) {
    let import_id = Uuid::new_v4();
    exec(
        &fx.pool,
        &format!(
            "{}{}",
            import_sql(
                import_id,
                fx.account_id,
                "COMPLETED",
                "'2026-03-10T00:00:00Z'"
            ),
            coverage_sql(
                fx.account_id,
                import_id,
                fx.platform_id,
                fx.sessions,
                day_n(from),
                day_n(to),
                "COMPLETE",
                country,
                institution
            )
        ),
    );
}

#[test]
fn a_country_representation_needs_country_coverage_to_be_complete_or_zero() {
    let (_guard, fx) = setup();
    // Days 1-2 fully covered including country; days 3-4 COMPLETE without
    // the country dimension.
    cover_dimensions(&fx, 1, 3, true, true);
    cover_dimensions(&fx, 3, 5, false, true);
    let input = window(&fx, &[fx.sessions]);

    // Undimensioned values do not depend on country, so the ordinary status
    // stands and empty days are real zeros.
    commit_dims(
        &fx,
        fx.works[0],
        fx.platform_id,
        fx.sessions,
        day_n(1),
        Dims::default(),
        9,
    );
    apply_all(&fx.pool);
    let aggregate = read(&fx, &input);
    assert_eq!(aggregate.coverage.status, MetricCoverageStatus::Complete);
    assert_eq!(
        bucket_values(&aggregate, fx.sessions),
        vec![some("9"), some("0"), some("0"), some("0")]
    );

    // Once the served values come from country rows, the days without
    // country coverage are only PARTIAL, and their empty cells are unknown.
    commit_dims(
        &fx,
        fx.works[1],
        fx.platform_id,
        fx.sessions,
        day_n(2),
        Dims {
            country: Some("GB"),
            ..Dims::default()
        },
        4,
    );
    apply_all(&fx.pool);
    let by_country = read(&fx, &input);
    assert_eq!(
        item(&by_country, fx.sessions).status,
        MetricCoverageStatus::Partial
    );
    assert_eq!(by_country.coverage.status, MetricCoverageStatus::Partial);
    assert_eq!(
        bucket_values(&by_country, fx.sessions),
        vec![some("9"), some("4"), None, None],
        "a dimensionally incomplete day must not manufacture a zero"
    );
    assert_eq!(codes(&by_country), vec![MetricWarningCode::PartialCoverage]);
    assert!(by_country.is_partial);
    assert_eq!(by_country.data_through, Some(day_n(2)));
    // Known values stay served.
    assert_eq!(total(&by_country, fx.platform_id, fx.sessions), some("13"));
}

#[test]
fn an_institution_representation_needs_institution_coverage_to_be_complete_or_zero() {
    let (_guard, fx) = setup();
    cover_dimensions(&fx, 1, 3, true, true);
    cover_dimensions(&fx, 3, 5, true, false);
    let input = window(&fx, &[fx.sessions]);
    let institution_id = insert_institution(&fx.pool);
    commit_dims(
        &fx,
        fx.works[0],
        fx.platform_id,
        fx.sessions,
        day_n(3),
        Dims {
            institution: Some(institution_id),
            ..Dims::default()
        },
        6,
    );
    apply_all(&fx.pool);

    let dashboard = read(&fx, &input);
    assert_eq!(
        item(&dashboard, fx.sessions).status,
        MetricCoverageStatus::Partial
    );
    assert_eq!(
        bucket_values(&dashboard, fx.sessions),
        vec![some("0"), some("0"), some("6"), None]
    );
    assert_eq!(codes(&dashboard), vec![MetricWarningCode::PartialCoverage]);
    assert_eq!(dashboard.data_through, Some(day_n(2)));

    // A publication representation has no coverage flag of its own, so it
    // relies on the ordinary status even where country and institution
    // coverage are absent.
    let publication = insert_publication(&fx.pool, fx.works[1], "PDF");
    let mut other = window(&fx, &[fx.units]);
    other.start_date = day_n(3);
    let import_id = Uuid::new_v4();
    exec(
        &fx.pool,
        &format!(
            "{}{}",
            import_sql(
                import_id,
                fx.account_id,
                "COMPLETED",
                "'2026-03-10T00:00:00Z'"
            ),
            coverage_sql(
                fx.account_id,
                import_id,
                fx.platform_id,
                fx.units,
                day_n(3),
                day_n(5),
                "COMPLETE",
                false,
                false
            )
        ),
    );
    commit_dims(
        &fx,
        fx.works[1],
        fx.platform_id,
        fx.units,
        day_n(3),
        Dims {
            publication: Some(publication),
            ..Dims::default()
        },
        2,
    );
    apply_all(&fx.pool);
    let by_publication = read(&fx, &other);
    assert_eq!(
        by_publication.coverage.status,
        MetricCoverageStatus::Complete
    );
    assert_eq!(
        bucket_values(&by_publication, fx.units),
        vec![some("2"), some("0")]
    );
}

// ==========================================================================
// Zero versus unknown, and coverage resolution
// ==========================================================================

#[test]
fn only_complete_coverage_turns_an_absent_value_into_zero() {
    let (_guard, fx) = setup();
    cover(
        &fx,
        fx.sessions,
        d1(),
        day_n(3),
        "COMPLETE",
        "COMPLETED",
        "'2026-03-10T00:00:00Z'",
    );
    cover(
        &fx,
        fx.sessions,
        day_n(3),
        day_n(4),
        "PARTIAL",
        "COMPLETED",
        "'2026-03-10T00:00:00Z'",
    );
    // Day 4 has no assertion at all.
    let input = window(&fx, &[fx.sessions]);

    let empty = read(&fx, &input);
    assert_eq!(
        bucket_values(&empty, fx.sessions),
        vec![some("0"), some("0"), None, None],
        "COMPLETE justifies zero; PARTIAL and UNKNOWN do not"
    );
    assert_eq!(total(&empty, fx.platform_id, fx.sessions), None);
    let coverage = item(&empty, fx.sessions);
    assert_eq!(coverage.status, MetricCoverageStatus::Unknown);
    assert_eq!(coverage.data_through, Some(day_n(2)));
    assert_eq!(
        codes(&empty),
        vec![
            MetricWarningCode::UnknownCoverage,
            MetricWarningCode::PartialCoverage
        ]
    );

    // A projected value inside incomplete coverage is still served exactly.
    commit(&fx, fx.works[0], fx.platform_id, fx.sessions, day_n(3), 4);
    apply_all(&fx.pool);
    let valued = read(&fx, &input);
    assert_eq!(
        bucket_values(&valued, fx.sessions),
        vec![some("0"), some("0"), some("4"), None]
    );
    assert_eq!(total(&valued, fx.platform_id, fx.sessions), some("4"));
    assert_eq!(
        item(&valued, fx.sessions).status,
        MetricCoverageStatus::Unknown
    );
}

/// The coverage status of exactly one day of the fixture measure.
fn day_status(fx: &Fixture, measure_id: Uuid, day: NaiveDate) -> MetricCoverageStatus {
    let dashboard = read(
        fx,
        &request(
            fx.publisher_id,
            day,
            day + Duration::days(1),
            &[fx.platform_id],
            &[measure_id],
            None,
        ),
    );
    item(&dashboard, measure_id).status
}

fn assert_on(
    fx: &Fixture,
    import_id: &str,
    status: &str,
    completed_at: &str,
    day: NaiveDate,
    declared: &str,
) {
    let import_id: Uuid = import_id.parse().expect("a fixture uuid");
    exec(
        &fx.pool,
        &format!(
            "{}{}",
            import_sql(import_id, fx.account_id, status, completed_at),
            coverage_sql(
                fx.account_id,
                import_id,
                fx.platform_id,
                fx.sessions,
                day,
                day + Duration::days(1),
                declared,
                true,
                true
            )
        ),
    );
}

#[test]
fn the_current_assertion_is_the_latest_terminal_import_then_the_greatest_import_id() {
    let (_guard, fx) = setup();
    let t10 = "'2026-03-10T00:00:00Z'";
    let t11 = "'2026-03-11T00:00:00Z'";
    let t12 = "'2026-03-12T00:00:00Z'";

    // Day 1: a later, more pessimistic assertion supersedes.
    assert_on(
        &fx,
        "10000000-0000-0000-0000-000000000001",
        "COMPLETED",
        t10,
        day_n(1),
        "COMPLETE",
    );
    assert_on(
        &fx,
        "10000000-0000-0000-0000-000000000002",
        "COMPLETED",
        t11,
        day_n(1),
        "PARTIAL",
    );
    // Day 2: a later, more optimistic assertion supersedes too.
    assert_on(
        &fx,
        "20000000-0000-0000-0000-000000000001",
        "COMPLETED",
        t10,
        day_n(2),
        "PARTIAL",
    );
    assert_on(
        &fx,
        "20000000-0000-0000-0000-000000000002",
        "COMPLETED",
        t11,
        day_n(2),
        "COMPLETE",
    );
    // Days 3 and 4: equal completion times, so the greatest import id wins,
    // whichever of the two it is.
    assert_on(
        &fx,
        "30000000-0000-0000-0000-000000000001",
        "COMPLETED",
        t12,
        day_n(3),
        "COMPLETE",
    );
    assert_on(
        &fx,
        "3fffffff-ffff-ffff-ffff-ffffffffffff",
        "COMPLETED",
        t12,
        day_n(3),
        "PARTIAL",
    );
    assert_on(
        &fx,
        "40000000-0000-0000-0000-000000000001",
        "COMPLETED",
        t12,
        day_n(4),
        "PARTIAL",
    );
    assert_on(
        &fx,
        "4fffffff-ffff-ffff-ffff-ffffffffffff",
        "COMPLETED",
        t12,
        day_n(4),
        "COMPLETE",
    );

    assert_eq!(
        day_status(&fx, fx.sessions, day_n(1)),
        MetricCoverageStatus::Partial
    );
    assert_eq!(
        day_status(&fx, fx.sessions, day_n(2)),
        MetricCoverageStatus::Complete
    );
    assert_eq!(
        day_status(&fx, fx.sessions, day_n(3)),
        MetricCoverageStatus::Partial
    );
    assert_eq!(
        day_status(&fx, fx.sessions, day_n(4)),
        MetricCoverageStatus::Complete
    );

    // Non-terminal imports, a failed import and a terminal import without a
    // completion time never become current, however recent or optimistic.
    let later = "'2026-03-20T00:00:00Z'";
    for (index, status) in ["UPLOADED", "QUEUED", "PROCESSING", "FAILED"]
        .iter()
        .enumerate()
    {
        assert_on(
            &fx,
            &format!("5000000{index}-0000-0000-0000-000000000001"),
            status,
            later,
            day_n(1),
            "COMPLETE",
        );
    }
    assert_on(
        &fx,
        "60000000-0000-0000-0000-000000000001",
        "COMPLETED",
        "NULL",
        day_n(2),
        "UNKNOWN",
    );
    assert_eq!(
        day_status(&fx, fx.sessions, day_n(1)),
        MetricCoverageStatus::Partial
    );
    assert_eq!(
        day_status(&fx, fx.sessions, day_n(2)),
        MetricCoverageStatus::Complete
    );

    // Reading is deterministic.
    let input = window(&fx, &[fx.sessions]);
    assert_eq!(read(&fx, &input).coverage, read(&fx, &input).coverage);
}

#[test]
fn completed_with_errors_cannot_establish_complete_coverage() {
    let (_guard, fx) = setup();
    let at = "'2026-03-10T00:00:00Z'";
    assert_on(
        &fx,
        "70000000-0000-0000-0000-000000000001",
        "COMPLETED_WITH_ERRORS",
        at,
        day_n(1),
        "COMPLETE",
    );
    assert_on(
        &fx,
        "70000000-0000-0000-0000-000000000002",
        "COMPLETED_WITH_ERRORS",
        at,
        day_n(2),
        "PARTIAL",
    );
    assert_on(
        &fx,
        "70000000-0000-0000-0000-000000000003",
        "COMPLETED_WITH_ERRORS",
        at,
        day_n(3),
        "UNKNOWN",
    );
    assert_on(
        &fx,
        "70000000-0000-0000-0000-000000000004",
        "COMPLETED",
        at,
        day_n(4),
        "COMPLETE",
    );

    assert_eq!(
        day_status(&fx, fx.sessions, day_n(1)),
        MetricCoverageStatus::Partial
    );
    assert_eq!(
        day_status(&fx, fx.sessions, day_n(2)),
        MetricCoverageStatus::Partial
    );
    assert_eq!(
        day_status(&fx, fx.sessions, day_n(3)),
        MetricCoverageStatus::Unknown
    );
    assert_eq!(
        day_status(&fx, fx.sessions, day_n(4)),
        MetricCoverageStatus::Complete
    );
    let dashboard = read(&fx, &window(&fx, &[fx.sessions]));
    assert_eq!(
        bucket_values(&dashboard, fx.sessions),
        vec![None, None, None, some("0")],
        "a downgraded day must not produce a zero"
    );
}

/// Coverage whose referenced import is not owned by its eligible account and
/// publisher, in the forms the schema's independent foreign keys permit.
#[derive(Clone, Copy)]
enum Malformed {
    /// The coverage names the eligible account, but its import belongs to
    /// another valid account of the same publisher.
    AccountMismatch,
    /// The import belongs to the eligible account but is scoped to another
    /// publisher.
    PublisherMismatch,
    /// The import belongs to the eligible account but has no publisher scope.
    PublisherMissing,
}

/// Insert one COMPLETED import and one coverage row for `(platform, measure)`
/// over `[start, end)` naming `account_id`, whose import is owned as
/// `malformed` says, or by `account_id` and its publisher when `None`.
#[allow(clippy::too_many_arguments)]
fn cover_owned(
    fx: &Fixture,
    account_id: Uuid,
    platform_id: Uuid,
    measure_id: Uuid,
    start: NaiveDate,
    end: NaiveDate,
    declared: &str,
    completed_at: &str,
    malformed: Option<Malformed>,
) {
    let import_id = Uuid::new_v4();
    let import = match malformed {
        None => import_sql(import_id, account_id, "COMPLETED", completed_at),
        Some(Malformed::AccountMismatch) => {
            let source_id = Uuid::new_v4();
            let owner = Uuid::new_v4();
            let third_platform = Uuid::new_v4();
            insert_platform_row(&fx.pool, third_platform, &format!("owner_{owner}"));
            format!(
                "{}{}{}",
                source_sql(source_id, "DRIVER", true),
                account_sql(
                    owner,
                    source_id,
                    third_platform,
                    Some(fx.publisher_id),
                    true
                ),
                import_sql(import_id, owner, "COMPLETED", completed_at)
            )
        }
        Some(Malformed::PublisherMismatch) => scoped_import_sql(
            import_id,
            account_id,
            &format!("'{}'", fx.other_publisher_id),
            "COMPLETED",
            completed_at,
        ),
        Some(Malformed::PublisherMissing) => {
            scoped_import_sql(import_id, account_id, "NULL", "COMPLETED", completed_at)
        }
    };
    exec(
        &fx.pool,
        &format!(
            "{import}{}",
            coverage_sql(
                account_id,
                import_id,
                platform_id,
                measure_id,
                start,
                end,
                declared,
                true,
                true
            )
        ),
    );
}

#[test]
fn coverage_counts_only_when_its_import_is_owned_by_the_account_and_publisher() {
    let (_guard, fx) = setup();
    let early = "'2026-03-10T00:00:00Z'";
    let later = "'2026-03-20T00:00:00Z'";
    let on_day = |n: i64, declared: &str, completed_at: &str, malformed: Option<Malformed>| {
        cover_owned(
            &fx,
            fx.account_id,
            fx.platform_id,
            fx.sessions,
            day_n(n),
            day_n(n + 1),
            declared,
            completed_at,
            malformed,
        )
    };

    // Days 1-3: only malformed COMPLETE evidence, which establishes nothing.
    on_day(1, "COMPLETE", early, Some(Malformed::AccountMismatch));
    on_day(2, "COMPLETE", early, Some(Malformed::PublisherMismatch));
    on_day(3, "COMPLETE", early, Some(Malformed::PublisherMissing));
    // Day 4: valid owned COMPLETE evidence.
    on_day(4, "COMPLETE", early, None);
    // Days 5-8: valid evidence that later malformed evidence cannot supersede,
    // whether it would be more pessimistic or more optimistic.
    on_day(5, "COMPLETE", early, None);
    on_day(5, "PARTIAL", later, Some(Malformed::AccountMismatch));
    on_day(6, "PARTIAL", early, None);
    on_day(6, "COMPLETE", later, Some(Malformed::AccountMismatch));
    on_day(7, "PARTIAL", early, None);
    on_day(7, "COMPLETE", later, Some(Malformed::PublisherMismatch));
    on_day(8, "COMPLETE", early, None);
    on_day(8, "PARTIAL", later, Some(Malformed::PublisherMismatch));

    let expected = [
        MetricCoverageStatus::Unknown,
        MetricCoverageStatus::Unknown,
        MetricCoverageStatus::Unknown,
        MetricCoverageStatus::Complete,
        MetricCoverageStatus::Complete,
        MetricCoverageStatus::Partial,
        MetricCoverageStatus::Partial,
        MetricCoverageStatus::Complete,
    ];
    for (index, status) in expected.iter().enumerate() {
        let day = day_n(index as i64 + 1);
        assert_eq!(
            day_status(&fx, fx.sessions, day),
            *status,
            "coverage status of {day}"
        );
    }

    // With nothing projected, only owned COMPLETE evidence justifies zero.
    let input = request(
        fx.publisher_id,
        d1(),
        day_n(9),
        &[fx.platform_id],
        &[fx.sessions],
        None,
    );
    let dashboard = read(&fx, &input);
    assert_eq!(
        bucket_values(&dashboard, fx.sessions),
        vec![
            None,
            None,
            None,
            some("0"),
            some("0"),
            None,
            None,
            some("0")
        ],
        "malformed coverage must never manufacture or withdraw a zero"
    );
    assert_eq!(total(&dashboard, fx.platform_id, fx.sessions), None);
    let coverage = item(&dashboard, fx.sessions);
    assert_eq!(coverage.status, MetricCoverageStatus::Unknown);
    assert_eq!(coverage.data_through, None);

    // Valid owned evidence over the whole window is COMPLETE with a real zero.
    let valid = request(
        fx.publisher_id,
        day_n(4),
        day_n(6),
        &[fx.platform_id],
        &[fx.sessions],
        None,
    );
    let complete = read(&fx, &valid);
    assert_eq!(complete.coverage.status, MetricCoverageStatus::Complete);
    assert_eq!(total(&complete, fx.platform_id, fx.sessions), some("0"));
    assert_eq!(item(&complete, fx.sessions).data_through, Some(day_n(5)));
    assert!(!complete.is_partial);
}

#[test]
fn more_than_one_eligible_driver_account_is_ambiguous_and_ineligible_ones_are_ignored() {
    let (_guard, fx) = setup();
    let input = window(&fx, &[fx.sessions]);
    let add_account = |source_enabled: bool,
                       acquisition: &str,
                       platform: Uuid,
                       publisher: Option<Uuid>,
                       enabled: bool| {
        let source_id = Uuid::new_v4();
        let account_id = Uuid::new_v4();
        exec(
            &fx.pool,
            &format!(
                "{}{}",
                source_sql(source_id, acquisition, source_enabled),
                account_sql(account_id, source_id, platform, publisher, enabled)
            ),
        );
        account_id
    };

    // None of these is a second eligible account for the selected
    // publisher and platform.
    add_account(true, "DRIVER", fx.platform_id, Some(fx.publisher_id), false);
    add_account(false, "DRIVER", fx.platform_id, Some(fx.publisher_id), true);
    add_account(
        true,
        "PUBLISHER_UPLOAD",
        fx.platform_id,
        Some(fx.publisher_id),
        true,
    );
    add_account(true, "OPERAS", fx.platform_id, Some(fx.publisher_id), true);
    add_account(
        true,
        "DRIVER",
        fx.platform_id,
        Some(fx.other_publisher_id),
        true,
    );
    add_account(true, "DRIVER", fx.platform_id, None, true);
    add_account(
        true,
        "DRIVER",
        fx.other_platform_id,
        Some(fx.publisher_id),
        true,
    );
    assert!(metric_dashboard(&fx.pool, &input).is_ok());

    // Coverage from an ineligible account never counts.
    let ineligible = add_account(
        true,
        "PUBLISHER_UPLOAD",
        fx.platform_id,
        Some(fx.publisher_id),
        true,
    );
    let import_id = Uuid::new_v4();
    exec(
        &fx.pool,
        &format!(
            "{}{}",
            import_sql(import_id, ineligible, "COMPLETED", "'2026-03-10T00:00:00Z'"),
            coverage_sql(
                ineligible,
                import_id,
                fx.platform_id,
                fx.sessions,
                d1(),
                day_n(5),
                "COMPLETE",
                true,
                true
            )
        ),
    );
    assert_eq!(
        read(&fx, &input).coverage.status,
        MetricCoverageStatus::Unknown
    );

    add_account(true, "DRIVER", fx.platform_id, Some(fx.publisher_id), true);
    assert_eq!(
        metric_dashboard(&fx.pool, &input),
        Err(MetricReadError::SourceScopeAmbiguous)
    );
}

#[test]
fn without_an_eligible_account_values_are_retained_but_coverage_is_unknown() {
    let (_guard, fx) = setup();
    commit(&fx, fx.works[0], fx.platform_id, fx.sessions, day_n(1), 9);
    apply_all(&fx.pool);
    cover(
        &fx,
        fx.sessions,
        d1(),
        day_n(5),
        "COMPLETE",
        "COMPLETED",
        "'2026-03-10T00:00:00Z'",
    );
    let input = window(&fx, &[fx.sessions]);
    assert_eq!(
        read(&fx, &input).coverage.status,
        MetricCoverageStatus::Complete
    );

    // Disabling the account keeps its data but withdraws its completeness
    // claim.
    exec(
        &fx.pool,
        &format!(
            "UPDATE metric_source_account SET enabled = FALSE \
             WHERE source_account_id = '{}';",
            fx.account_id
        ),
    );
    let dashboard = read(&fx, &input);
    assert_eq!(total(&dashboard, fx.platform_id, fx.sessions), some("9"));
    assert_eq!(
        bucket_values(&dashboard, fx.sessions),
        vec![some("9"), None, None, None]
    );
    assert_eq!(dashboard.coverage.status, MetricCoverageStatus::Unknown);
    assert_eq!(dashboard.data_through, None);
    assert_eq!(codes(&dashboard), vec![MetricWarningCode::UnknownCoverage]);
}

#[test]
fn dimension_coverage_requires_every_day_to_include_the_dimension() {
    let (_guard, fx) = setup();
    let at = "'2026-03-10T00:00:00Z'";
    let import_id = Uuid::new_v4();
    exec(
        &fx.pool,
        &format!(
            "{}{}{}",
            import_sql(import_id, fx.account_id, "COMPLETED", at),
            coverage_sql(
                fx.account_id,
                import_id,
                fx.platform_id,
                fx.sessions,
                d1(),
                day_n(2),
                "COMPLETE",
                true,
                true
            ),
            coverage_sql(
                fx.account_id,
                import_id,
                fx.platform_id,
                fx.sessions,
                day_n(2),
                day_n(3),
                "COMPLETE",
                true,
                false
            ),
        ),
    );
    let two_days = request(
        fx.publisher_id,
        d1(),
        day_n(3),
        &[fx.platform_id],
        &[fx.sessions],
        None,
    );
    let coverage = item(&read(&fx, &two_days), fx.sessions).clone();
    assert!(coverage.country_coverage);
    assert!(!coverage.institution_coverage);

    // A day without any assertion includes no dimension.
    let three_days = request(
        fx.publisher_id,
        d1(),
        day_n(4),
        &[fx.platform_id],
        &[fx.sessions],
        None,
    );
    let coverage = item(&read(&fx, &three_days), fx.sessions).clone();
    assert!(!coverage.country_coverage);
    assert!(!coverage.institution_coverage);
}

#[test]
fn nothing_to_serve_is_unknown_rather_than_complete() {
    let (_guard, fx) = setup();
    let mut input = window(&fx, &[]);
    input.platforms = None;
    let dashboard = read(&fx, &input);
    assert!(dashboard.totals.is_empty());
    assert!(dashboard.timeline.is_empty());
    assert!(dashboard.coverage.items.is_empty());
    assert_eq!(dashboard.coverage.status, MetricCoverageStatus::Unknown);
    assert_eq!(dashboard.data_through, None);
    assert_eq!(codes(&dashboard), vec![MetricWarningCode::UnknownCoverage]);
    assert!(dashboard.is_partial);
}

// ==========================================================================
// Freshness
// ==========================================================================

#[test]
fn freshness_maps_the_frontier_exactly_and_only_intersecting_backlog_lags() {
    let (_guard, fx) = setup();
    cover(
        &fx,
        fx.sessions,
        d1(),
        day_n(5),
        "COMPLETE",
        "COMPLETED",
        "'2026-03-10T00:00:00Z'",
    );
    commit(&fx, fx.works[0], fx.platform_id, fx.sessions, day_n(1), 5);
    apply_all(&fx.pool);
    let input = window(&fx, &[fx.sessions]);

    let before = state(&fx.pool);
    let fresh = read(&fx, &input);
    assert_eq!(fresh.rollup_watermark, before.watermark_at);
    assert!(fresh.as_of >= before.watermark_at);
    assert!(fresh.warnings.is_empty());
    assert_eq!(fresh.data_through, Some(day_n(4)));

    // Backlog that does not intersect the request: another publisher, another
    // platform, another measure, the day before startDate and endDate itself.
    commit(&fx, fx.other_work, fx.platform_id, fx.sessions, day_n(2), 1);
    commit(
        &fx,
        fx.works[0],
        fx.other_platform_id,
        fx.sessions,
        day_n(2),
        1,
    );
    commit(&fx, fx.works[0], fx.platform_id, fx.units, day_n(2), 1);
    commit(&fx, fx.works[0], fx.platform_id, fx.sessions, day_n(0), 1);
    commit(&fx, fx.works[0], fx.platform_id, fx.sessions, day_n(5), 1);
    let unrelated = read(&fx, &input);
    assert!(state(&fx.pool).applied_through_sequence < state(&fx.pool).next_sequence - 1);
    assert!(
        unrelated.warnings.is_empty(),
        "unrelated backlog is not lag"
    );
    assert!(!unrelated.is_partial);
    assert_eq!(
        unrelated.data_through,
        Some(day_n(4)),
        "pre-startDate and non-intersecting backlog must not reduce dataThrough"
    );

    // Backlog on day 3 of the selected scope.
    commit(&fx, fx.works[1], fx.platform_id, fx.sessions, day_n(3), 8);
    let lagging = read(&fx, &input);
    assert_eq!(codes(&lagging), vec![MetricWarningCode::RollupLag]);
    assert!(lagging.is_partial);
    assert_eq!(
        item(&lagging, fx.sessions).status,
        MetricCoverageStatus::Complete
    );
    assert_eq!(item(&lagging, fx.sessions).data_through, Some(day_n(2)));
    assert_eq!(lagging.data_through, Some(day_n(2)));
    assert_eq!(
        bucket_values(&lagging, fx.sessions),
        vec![some("5"), some("0"), None, some("0")],
        "a lagging empty day is not a zero, and other complete days still are"
    );
    assert_eq!(total(&lagging, fx.platform_id, fx.sessions), some("5"));

    // Lag on the first day leaves no justified prefix.
    commit(&fx, fx.works[1], fx.platform_id, fx.sessions, day_n(1), 8);
    assert_eq!(read(&fx, &input).data_through, None);
}

#[test]
fn reading_never_advances_repairs_or_mutates_rollup_or_coverage_state() {
    let (_guard, fx) = setup();
    cover(
        &fx,
        fx.sessions,
        d1(),
        day_n(5),
        "COMPLETE",
        "COMPLETED",
        "'2026-03-10T00:00:00Z'",
    );
    commit(&fx, fx.works[0], fx.platform_id, fx.sessions, day_n(1), 5);
    apply_all(&fx.pool);
    commit(&fx, fx.works[0], fx.platform_id, fx.sessions, day_n(2), 6);

    let state_before = state(&fx.pool);
    let deltas_before = deltas(&fx.pool);
    let projection_before = projection(&fx.pool);
    let coverage_before = scalar_i64(&fx.pool, "(SELECT COUNT(*) FROM metric_coverage)");
    for _ in 0..3 {
        let dashboard = read(&fx, &window(&fx, &[fx.sessions]));
        assert_eq!(codes(&dashboard), vec![MetricWarningCode::RollupLag]);
    }
    assert_eq!(state(&fx.pool), state_before);
    assert_eq!(deltas(&fx.pool), deltas_before);
    assert_eq!(projection(&fx.pool), projection_before);
    assert_eq!(
        scalar_i64(&fx.pool, "(SELECT COUNT(*) FROM metric_coverage)"),
        coverage_before
    );
}

#[test]
fn the_whole_response_is_read_from_one_repeatable_read_snapshot() {
    let (_guard, fx) = setup();
    commit(&fx, fx.works[0], fx.platform_id, fx.sessions, day_n(1), 5);
    apply_all(&fx.pool);
    let input = window(&fx, &[fx.sessions]);

    // Hold an exclusive lock on the coverage table, so the dashboard's
    // coverage statement — which runs after it has already read the frontier
    // and the projection — has to wait.
    let mut writer =
        PgConnection::establish(&test_db_url()).expect("Failed to open the writer connection");
    writer
        .batch_execute("BEGIN; LOCK TABLE metric_coverage IN ACCESS EXCLUSIVE MODE;")
        .expect("lock the coverage table");

    let pool = Arc::clone(&fx.pool);
    let reader_input = input.clone();
    let reader = std::thread::spawn(move || metric_dashboard(&pool, &reader_input));

    let deadline = Instant::now() + StdDuration::from_secs(30);
    loop {
        let waiting = scalar_i64(
            &fx.pool,
            "(SELECT COUNT(*) FROM pg_stat_activity \
              WHERE datname = current_database() \
                AND wait_event_type = 'Lock' \
                AND query LIKE '%JOIN public.metric_coverage c%')",
        );
        if waiting == 1 {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "the dashboard never reached the coverage read"
        );
        std::thread::sleep(StdDuration::from_millis(20));
    }

    // While the dashboard waits, commit complete coverage and a query-relevant
    // pending delta. A read that re-snapshotted per statement would see both.
    let import_id = Uuid::new_v4();
    writer
        .batch_execute(&format!(
            "{}{}{}",
            import_sql(
                import_id,
                fx.account_id,
                "COMPLETED",
                "'2026-03-10T00:00:00Z'"
            ),
            coverage_sql(
                fx.account_id,
                import_id,
                fx.platform_id,
                fx.sessions,
                d1(),
                day_n(5),
                "COMPLETE",
                true,
                true
            ),
            record_sql(
                &fx,
                Uuid::new_v4(),
                fx.works[1],
                fx.platform_id,
                fx.sessions,
                day_n(2),
                3
            ),
        ))
        .expect("write while the dashboard waits");
    #[derive(diesel::QueryableByName)]
    struct Clock {
        #[diesel(sql_type = Timestamptz)]
        at: Timestamp,
    }
    let committed_after = sql_query("SELECT clock_timestamp() AS at")
        .get_result::<Clock>(&mut writer)
        .expect("read the writer clock")
        .at;
    writer.batch_execute("COMMIT;").expect("commit the writer");

    let snapshot = reader
        .join()
        .expect("the dashboard thread")
        .expect("the dashboard read");
    assert_eq!(
        snapshot.coverage.status,
        MetricCoverageStatus::Unknown,
        "coverage committed after the snapshot must not be visible"
    );
    assert!(
        !codes(&snapshot).contains(&MetricWarningCode::RollupLag),
        "a delta committed after the snapshot must not be visible"
    );
    assert!(
        snapshot.as_of < committed_after,
        "asOf is the transaction timestamp, not a clock read after the query"
    );

    // A fresh read sees both, so their absence above was the snapshot.
    let fresh = read(&fx, &input);
    assert_eq!(fresh.coverage.status, MetricCoverageStatus::Complete);
    assert_eq!(codes(&fresh), vec![MetricWarningCode::RollupLag]);
    assert!(fresh.as_of > committed_after);
}

#[test]
fn the_read_path_is_one_read_only_repeatable_read_transaction_of_selects() {
    let source = include_str!("mod.rs");
    let entry = source
        .split_once("pub(crate) fn metric_dashboard(")
        .expect("the entry point")
        .1
        .split_once("\n}\n")
        .expect("the entry point body")
        .0;
    let chain: String = entry.split_whitespace().collect();
    assert!(
        chain.contains(".build_transaction().read_only().repeatable_read().run(|connection|evaluate(connection,&request))"),
        "every statement must run inside one read-only repeatable-read transaction: {chain}"
    );
    assert_eq!(
        source.matches("db.get()").count(),
        1,
        "exactly one connection"
    );

    for (name, statement) in [
        ("PUBLISHERS_SQL", PUBLISHERS_SQL),
        ("FRONTIER_SQL", FRONTIER_SQL),
        ("KNOWN_PLATFORMS_SQL", KNOWN_PLATFORMS_SQL),
        ("KNOWN_MEASURES_SQL", KNOWN_MEASURES_SQL),
        ("REPRESENTED_SQL", REPRESENTED_SQL),
        ("MEASURE_FLAGS_SQL", MEASURE_FLAGS_SQL),
        ("ACCOUNTS_SQL", ACCOUNTS_SQL),
        ("DAY_SUMS_SQL", DAY_SUMS_SQL),
        ("DIMENSION_CELLS_SQL", DIMENSION_CELLS_SQL),
        ("ASSERTIONS_SQL", ASSERTIONS_SQL),
        ("LAG_SQL", LAG_SQL),
        ("IDENTIFIER_QUALITY_SQL", IDENTIFIER_QUALITY_SQL),
    ] {
        let upper = statement.to_uppercase();
        assert!(
            upper.trim_start().starts_with("SELECT"),
            "{name} must be a SELECT"
        );
        for forbidden in [
            "INSERT",
            "UPDATE",
            "DELETE",
            "FOR SHARE",
            "LOCK",
            ";",
            "NEXTVAL",
        ] {
            assert!(
                !upper.contains(forbidden),
                "{name} must not contain `{forbidden}`"
            );
        }
    }
    assert_eq!(
        source.matches("diesel::sql_query(").count(),
        12,
        "every statement the read executes is one of the named constants"
    );
}

// ==========================================================================
// Bounds and validation
// ==========================================================================

#[test]
fn request_shape_is_validated_before_any_database_access() {
    // Every one of these fails on an unreachable pool with its own
    // classification, so none of them touched the database.
    let unreachable = failing_pool();
    let publisher = Uuid::new_v4();
    let base = request(
        publisher,
        d1(),
        day_n(5),
        &[Uuid::new_v4()],
        &[Uuid::new_v4()],
        None,
    );
    let ids = |count: usize| (0..count).map(|_| Uuid::new_v4()).collect::<Vec<_>>();

    let mut cases: Vec<(&str, MetricDashboardInput, MetricReadError)> = Vec::new();
    for (label, publishers) in [
        ("no selector publishers", None),
        ("empty selector publishers", Some(vec![])),
        ("two publishers", Some(vec![publisher, Uuid::new_v4()])),
        ("the same publisher twice", Some(vec![publisher, publisher])),
    ] {
        let mut input = base.clone();
        input.selector.publisher_ids = publishers;
        cases.push((
            label,
            input,
            MetricReadError::QueryInvalid(PUBLISHER_CARDINALITY),
        ));
    }
    let mut input = base.clone();
    input.end_date = input.start_date;
    cases.push((
        "empty range",
        input,
        MetricReadError::QueryInvalid(DATE_ORDER),
    ));
    let mut input = base.clone();
    input.end_date = input.start_date - Duration::days(1);
    cases.push((
        "reversed range",
        input,
        MetricReadError::QueryInvalid(DATE_ORDER),
    ));
    let mut input = base.clone();
    input.end_date = input.start_date + Duration::days(367);
    cases.push((
        "367 days",
        input,
        MetricReadError::QueryLimitExceeded(RANGE_TOO_LONG),
    ));
    let duplicate = Uuid::new_v4();
    let mut input = base.clone();
    input.measures = Some(vec![duplicate, duplicate]);
    cases.push((
        "duplicate measure",
        input,
        MetricReadError::QueryInvalid(DUPLICATE_MEASURE),
    ));
    let mut input = base.clone();
    input.platforms = Some(vec![duplicate, duplicate]);
    cases.push((
        "duplicate platform",
        input,
        MetricReadError::QueryInvalid(DUPLICATE_PLATFORM),
    ));
    let mut input = base.clone();
    input.measures = Some(ids(11));
    cases.push((
        "11 measures",
        input,
        MetricReadError::QueryLimitExceeded(TOO_MANY_MEASURES),
    ));
    let mut input = base.clone();
    input.platforms = Some(ids(11));
    cases.push((
        "11 platforms",
        input,
        MetricReadError::QueryLimitExceeded(TOO_MANY_PLATFORMS),
    ));

    for (label, input, expected) in cases {
        assert_eq!(
            metric_dashboard(&unreachable, &input),
            Err(expected),
            "{label}"
        );
    }

    // The accepted edges reach the database, which here is unreachable.
    let mut input = base.clone();
    input.end_date = input.start_date + Duration::days(366);
    input.measures = Some(ids(10));
    input.platforms = Some(ids(10));
    assert_eq!(
        metric_dashboard(&unreachable, &input),
        Err(MetricReadError::Unavailable)
    );
}

#[test]
fn publisher_existence_and_registry_ids_are_validated_inside_the_snapshot() {
    let (_guard, fx) = setup();
    let mut unknown_publisher = window(&fx, &[fx.sessions]);
    unknown_publisher.selector.publisher_ids = Some(vec![Uuid::new_v4()]);
    assert_eq!(
        metric_dashboard(&fx.pool, &unknown_publisher),
        Err(MetricReadError::QueryInvalid(UNKNOWN_PUBLISHER))
    );
    assert_eq!(
        metric_dashboard(&fx.pool, &window(&fx, &[fx.sessions, Uuid::new_v4()])),
        Err(MetricReadError::QueryInvalid(UNKNOWN_MEASURE))
    );
    let mut unknown_platform = window(&fx, &[fx.sessions]);
    unknown_platform.platforms = Some(vec![fx.platform_id, Uuid::new_v4()]);
    assert_eq!(
        metric_dashboard(&fx.pool, &unknown_platform),
        Err(MetricReadError::QueryInvalid(UNKNOWN_PLATFORM))
    );

    // A disabled registry identity is still addressable for retained data.
    commit(&fx, fx.works[0], fx.platform_id, fx.sessions, day_n(1), 4);
    apply_all(&fx.pool);
    exec(
        &fx.pool,
        &format!(
            "UPDATE metric_platform SET enabled = FALSE WHERE platform_id = '{}'; \
             UPDATE metric_measure SET enabled = FALSE WHERE measure_id = '{}';",
            fx.platform_id, fx.sessions
        ),
    );
    assert_eq!(
        total(
            &read(&fx, &window(&fx, &[fx.sessions])),
            fx.platform_id,
            fx.sessions
        ),
        some("4")
    );
}

#[test]
fn combination_timeline_and_resolved_dimension_limits_are_enforced_exactly() {
    let (_guard, fx) = setup();
    let platforms: Vec<Uuid> = (0..11)
        .map(|index| {
            let platform_id = Uuid::new_v4();
            insert_platform_row(&fx.pool, platform_id, &format!("bound_platform_{index:02}"));
            platform_id
        })
        .collect();
    let measures: Vec<Uuid> = (0..6)
        .map(|index| insert_measure(&fx.pool, &format!("bound_measure_{index}"), true, true))
        .collect();
    let at = |days: i64, p: &[Uuid], m: &[Uuid], grain| {
        request(
            fx.publisher_id,
            d1(),
            d1() + Duration::days(days),
            p,
            m,
            grain,
        )
    };

    // 25 combinations x 200 daily buckets = 5000 cells: accepted.
    let edge = read(&fx, &at(200, &platforms[..5], &measures[..5], None));
    assert_eq!(edge.totals.len(), 25);
    assert_eq!(edge.timeline.len(), 5000);
    // One more day is 5025 cells.
    assert_eq!(
        metric_dashboard(&fx.pool, &at(201, &platforms[..5], &measures[..5], None)),
        Err(MetricReadError::QueryLimitExceeded(TOO_MANY_CELLS))
    );
    // The same 25 combinations over 366 days fit as MONTH buckets.
    let months = read(
        &fx,
        &at(
            366,
            &platforms[..5],
            &measures[..5],
            Some(MetricTimelineGrain::Month),
        ),
    );
    assert_eq!(months.timeline.len(), 25 * 13);
    // 26 or more combinations are refused even for one day.
    assert_eq!(
        metric_dashboard(&fx.pool, &at(1, &platforms[..6], &measures[..5], None)),
        Err(MetricReadError::QueryLimitExceeded(TOO_MANY_COMBINATIONS))
    );
    // Ten explicit platforms with one measure is within every bound.
    assert_eq!(
        read(&fx, &at(1, &platforms[..10], &measures[..1], None))
            .totals
            .len(),
        10
    );
    // 366 days is the longest range.
    assert_eq!(
        read(&fx, &at(366, &platforms[..1], &measures[..1], None))
            .timeline
            .len(),
        366
    );

    // An omitted dimension that resolves to more than ten identities is
    // refused rather than truncated.
    let mut rows = String::new();
    for platform_id in &platforms {
        rows.push_str(&projection_sql(
            fx.works[0],
            *platform_id,
            measures[0],
            d1(),
            None,
            1,
        ));
    }
    exec(&fx.pool, &rows);
    let mut implicit = at(1, &[], &measures[..1], None);
    implicit.platforms = Some(vec![]);
    assert_eq!(
        metric_dashboard(&fx.pool, &implicit),
        Err(MetricReadError::QueryLimitExceeded(TOO_MANY_PLATFORMS))
    );
    implicit.platforms = None;
    assert_eq!(
        metric_dashboard(&fx.pool, &implicit),
        Err(MetricReadError::QueryLimitExceeded(TOO_MANY_PLATFORMS))
    );
}

#[test]
fn omitted_filters_resolve_to_the_represented_scope() {
    let (_guard, fx) = setup();
    // Represented in the projection for this publisher and range.
    commit(&fx, fx.works[0], fx.platform_id, fx.sessions, day_n(1), 2);
    commit(
        &fx,
        fx.works[0],
        fx.other_platform_id,
        fx.sessions,
        day_n(2),
        3,
    );
    // Not represented: another publisher, and outside the range.
    commit(&fx, fx.other_work, fx.platform_id, fx.units, day_n(1), 50);
    commit(&fx, fx.works[0], fx.platform_id, fx.units, day_n(9), 60);
    apply_all(&fx.pool);
    // Represented only through terminal coverage from the eligible account.
    let extra = insert_measure(&fx.pool, "covered_only", true, true);
    cover(
        &fx,
        extra,
        d1(),
        day_n(5),
        "COMPLETE",
        "COMPLETED",
        "'2026-03-10T00:00:00Z'",
    );

    let mut input = window(&fx, &[]);
    input.platforms = None;
    input.measures = None;
    let dashboard = read(&fx, &input);
    let mut served: Vec<(Uuid, Uuid)> = dashboard
        .totals
        .iter()
        .map(|total| (total.platform_id, total.measure_id))
        .collect();
    served.sort();
    let mut expected: Vec<(Uuid, Uuid)> = [fx.platform_id, fx.other_platform_id]
        .iter()
        .flat_map(|platform| [fx.sessions, extra].map(|measure| (*platform, measure)))
        .collect();
    expected.sort();
    assert_eq!(
        served, expected,
        "units is not represented and must not be served"
    );
    assert_eq!(total(&dashboard, fx.platform_id, fx.sessions), some("2"));
    assert_eq!(total(&dashboard, fx.platform_id, extra), some("0"));

    // Omitting one dimension resolves it under the explicit other one.
    let mut measures_only = window(&fx, &[extra]);
    measures_only.platforms = Some(vec![]);
    let resolved = read(&fx, &measures_only);
    assert_eq!(
        resolved
            .totals
            .iter()
            .map(|total| total.platform_id)
            .collect::<Vec<_>>(),
        vec![fx.platform_id]
    );
}

#[test]
fn coverage_not_owned_by_its_account_and_publisher_is_not_represented() {
    let (_guard, fx) = setup();
    commit(&fx, fx.works[0], fx.platform_id, fx.sessions, day_n(1), 2);
    apply_all(&fx.pool);
    let early = "'2026-03-10T00:00:00Z'";
    // A measure, and a platform with its own eligible account, each covered
    // only by malformed evidence in every form.
    let measure = insert_measure(&fx.pool, "malformed_only", true, true);
    let platform = Uuid::new_v4();
    insert_platform_row(&fx.pool, platform, "dashboard_platform_c");
    let source_id = Uuid::new_v4();
    let account = Uuid::new_v4();
    exec(
        &fx.pool,
        &format!(
            "{}{}",
            source_sql(source_id, "DRIVER", true),
            account_sql(account, source_id, platform, Some(fx.publisher_id), true)
        ),
    );
    for malformed in [
        Malformed::AccountMismatch,
        Malformed::PublisherMismatch,
        Malformed::PublisherMissing,
    ] {
        cover_owned(
            &fx,
            fx.account_id,
            fx.platform_id,
            measure,
            d1(),
            day_n(5),
            "COMPLETE",
            early,
            Some(malformed),
        );
        cover_owned(
            &fx,
            account,
            platform,
            fx.sessions,
            d1(),
            day_n(5),
            "COMPLETE",
            early,
            Some(malformed),
        );
    }

    let served = |omitted: bool| {
        let mut input = window(&fx, &[]);
        if omitted {
            input.platforms = None;
            input.measures = None;
        } else {
            input.platforms = Some(vec![]);
        }
        let mut served: Vec<(Uuid, Uuid)> = read(&fx, &input)
            .totals
            .iter()
            .map(|total| (total.platform_id, total.measure_id))
            .collect();
        served.sort();
        served
    };
    for omitted in [true, false] {
        assert_eq!(
            served(omitted),
            vec![(fx.platform_id, fx.sessions)],
            "malformed coverage alone must not enter the resolved scope (omitted: {omitted})"
        );
    }

    // Owned evidence for the same identities does enter it.
    cover_owned(
        &fx,
        fx.account_id,
        fx.platform_id,
        measure,
        d1(),
        day_n(5),
        "COMPLETE",
        early,
        None,
    );
    cover_owned(
        &fx,
        account,
        platform,
        fx.sessions,
        d1(),
        day_n(5),
        "COMPLETE",
        early,
        None,
    );
    let mut expected: Vec<(Uuid, Uuid)> = [fx.platform_id, platform]
        .iter()
        .flat_map(|platform| [fx.sessions, measure].map(|measure| (*platform, measure)))
        .collect();
    expected.sort();
    for omitted in [true, false] {
        assert_eq!(served(omitted), expected, "omitted: {omitted}");
    }
}

// ==========================================================================
// Additivity
// ==========================================================================

#[test]
fn only_measures_additive_across_time_and_works_are_served() {
    let (_guard, fx) = setup();
    let not_over_time = insert_measure(&fx.pool, "not_over_time", false, true);
    let not_over_works = insert_measure(&fx.pool, "not_over_works", true, false);
    let neither = insert_measure(&fx.pool, "neither", false, false);

    assert!(metric_dashboard(&fx.pool, &window(&fx, &[fx.sessions, fx.units])).is_ok());
    for measure in [not_over_time, not_over_works, neither] {
        assert_eq!(
            metric_dashboard(&fx.pool, &window(&fx, &[fx.sessions, measure])),
            Err(MetricReadError::QueryInvalid(NON_ADDITIVE_MEASURE)),
            "an explicitly selected non-additive measure is refused, not omitted"
        );
    }

    // Implicitly resolved: a represented non-additive measure fails the whole
    // request rather than being silently left out.
    let mut implicit = window(&fx, &[]);
    implicit.measures = None;
    commit(&fx, fx.works[0], fx.platform_id, fx.sessions, day_n(1), 1);
    apply_all(&fx.pool);
    assert!(metric_dashboard(&fx.pool, &implicit).is_ok());
    exec(
        &fx.pool,
        &projection_sql(
            fx.works[0],
            fx.platform_id,
            not_over_works,
            day_n(2),
            None,
            5,
        ),
    );
    assert_eq!(
        metric_dashboard(&fx.pool, &implicit),
        Err(MetricReadError::QueryInvalid(NON_ADDITIVE_MEASURE))
    );
    // Outside the range it is not represented, so the request is served.
    let mut later = implicit.clone();
    later.start_date = day_n(3);
    assert!(metric_dashboard(&fx.pool, &later).is_ok());
    // Represented only through coverage counts as well.
    cover(
        &fx,
        not_over_time,
        day_n(3),
        day_n(4),
        "COMPLETE",
        "COMPLETED",
        "'2026-03-10T00:00:00Z'",
    );
    assert_eq!(
        metric_dashboard(&fx.pool, &later),
        Err(MetricReadError::QueryInvalid(NON_ADDITIVE_MEASURE))
    );
}

// ==========================================================================
// Entitlement
// ==========================================================================

#[test]
fn the_selected_publisher_must_hold_the_dashboard_capability_through_its_package() {
    let (_guard, fx) = setup();
    commit(&fx, fx.works[0], fx.platform_id, fx.sessions, day_n(1), 4);
    apply_all(&fx.pool);
    let input = window(&fx, &[fx.sessions]);
    for (package, entitled) in [
        ("OASIS", false),
        ("OBELISK", false),
        ("SPHINX", true),
        ("PYRAMID", true),
    ] {
        exec(
            &fx.pool,
            &format!(
                "UPDATE publisher SET subscription_package = '{package}' \
                 WHERE publisher_id = '{}';",
                fx.publisher_id
            ),
        );
        let result = metric_dashboard(&fx.pool, &input);
        if entitled {
            assert_eq!(
                total(
                    &result.expect("an entitled publisher is served"),
                    fx.platform_id,
                    fx.sessions
                ),
                some("4")
            );
        } else {
            assert_eq!(result, Err(MetricReadError::Unauthorised), "{package}");
        }
    }
    // The capability is decided by the package model, not by a name here.
    assert!(ThothPackage::Sphinx.has_capability(PublisherCapability::MetricsDashboard));
    assert!(!ThothPackage::Obelisk.has_capability(PublisherCapability::MetricsDashboard));
}

// ==========================================================================
// Set-based access
// ==========================================================================

#[derive(Debug)]
struct CountStatements(Arc<AtomicUsize>);

impl CustomizeConnection<PgConnection, diesel::r2d2::Error> for CountStatements {
    fn on_acquire(&self, connection: &mut PgConnection) -> Result<(), diesel::r2d2::Error> {
        let counter = Arc::clone(&self.0);
        connection.set_instrumentation(move |event: InstrumentationEvent<'_>| {
            if let InstrumentationEvent::StartQuery { .. } = event {
                counter.fetch_add(1, Ordering::SeqCst);
            }
        });
        Ok(())
    }
}

fn counted_statements(input: &MetricDashboardInput) -> usize {
    let counter = Arc::new(AtomicUsize::new(0));
    let pool = diesel::r2d2::Pool::builder()
        .max_size(1)
        .connection_customizer(Box::new(CountStatements(Arc::clone(&counter))))
        .build(ConnectionManager::<PgConnection>::new(test_db_url()))
        .expect("a counting pool");
    // Warm the connection so establishment is not counted.
    drop(pool.get().expect("a counting connection"));
    counter.store(0, Ordering::SeqCst);
    metric_dashboard(&pool, input).expect("the dashboard read");
    counter.load(Ordering::SeqCst)
}

#[test]
fn the_statement_count_does_not_grow_with_works_days_or_rows() {
    let (_guard, fx) = setup();
    cover(
        &fx,
        fx.sessions,
        d1(),
        day_n(5),
        "COMPLETE",
        "COMPLETED",
        "'2026-03-10T00:00:00Z'",
    );
    commit(&fx, fx.works[0], fx.platform_id, fx.sessions, day_n(1), 1);
    commit(&fx, fx.works[0], fx.platform_id, fx.sessions, day_n(9), 1);
    apply_all(&fx.pool);
    let mut input = window(&fx, &[fx.sessions]);
    input.platforms = None;
    let small = counted_statements(&input);

    // Forty more works with values on every day, backlog included.
    let many: Vec<Uuid> = (0..40).map(|_| Uuid::new_v4()).collect();
    exec(&fx.pool, &works_sql(fx.publisher_id, &many));
    for work_id in &many {
        for day in 1..=4 {
            commit(&fx, *work_id, fx.platform_id, fx.sessions, day_n(day), 2);
        }
    }
    // Dimensional rows too: every work gains a country breakdown of its
    // aggregate on day 1, and half gain an institution breakdown on day 3.
    let institution_id = insert_institution(&fx.pool);
    for work_id in &many {
        for code in ["GB", "US"] {
            commit_dims(
                &fx,
                *work_id,
                fx.platform_id,
                fx.sessions,
                day_n(1),
                Dims {
                    country: Some(code),
                    ..Dims::default()
                },
                1,
            );
        }
    }
    for work_id in &many[..20] {
        commit_dims(
            &fx,
            *work_id,
            fx.platform_id,
            fx.sessions,
            day_n(3),
            Dims {
                institution: Some(institution_id),
                ..Dims::default()
            },
            1,
        );
    }
    apply_all(&fx.pool);
    commit(&fx, many[0], fx.platform_id, fx.sessions, day_n(2), 1);
    let large = counted_statements(&input);
    let mut wide = input.clone();
    wide.end_date = d1() + Duration::days(366);
    let wide = counted_statements(&wide);

    // Transaction control, publisher, frontier, measure existence, represented
    // scope, additivity, accounts, projection, coverage, identifier quality and
    // lag: a fixed set.
    assert!(small <= 15, "{small} statements");
    assert_eq!(
        large,
        small + 2,
        "only the lag statement and the one base-cell resolution statement are added once \
         backlog and mixed dimensional representations exist"
    );
    assert_eq!(wide, large);

    // Many more dimensional rows and many more mixed groups add no statement.
    for work_id in &many {
        for day in [2, 4] {
            for code in ["DE", "FR", "IT"] {
                commit_dims(
                    &fx,
                    *work_id,
                    fx.platform_id,
                    fx.sessions,
                    day_n(day),
                    Dims {
                        country: Some(code),
                        ..Dims::default()
                    },
                    1,
                );
            }
        }
    }
    commit(&fx, many[1], fx.platform_id, fx.sessions, day_n(3), 1);
    assert_eq!(counted_statements(&input), large);
}

// ==========================================================================
// Query-plan and latency evidence (run explicitly)
// ==========================================================================

/// The first day of the one-year plan fixture.
fn plan_start() -> NaiveDate {
    d(2025, 3, 1)
}

fn uuid_array(ids: &[Uuid]) -> String {
    let quoted: Vec<String> = ids.iter().map(|id| format!("'{id}'")).collect();
    format!("ARRAY[{}]::uuid[]", quoted.join(","))
}

/// The grid a plan fixture serves.
struct PlanGrid {
    platforms: Vec<Uuid>,
    measures: Vec<Uuid>,
    imprint: Uuid,
}

/// A production-shaped fixture for the selected publisher: 600 works with a
/// year of daily sessions in three countries each, weekly signed units, a
/// sparse 5x5 platform/measure grid, one eligible account per platform with
/// a daily terminal import covering every measure plus monthly reprocessing,
/// two 600-work noise publishers with daily sessions, and an unapplied
/// backlog of 2,000 selected and 3,000 unrelated work-day deltas.
fn plan_fixture(fx: &Fixture) -> PlanGrid {
    let start = plan_start();
    let third_publisher = Uuid::new_v4();
    let (imprint_a, imprint_b, imprint_c) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    let mut platforms = vec![fx.platform_id];
    for index in 0..4 {
        let platform_id = Uuid::new_v4();
        insert_platform_row(&fx.pool, platform_id, &format!("plan_platform_{index}"));
        platforms.push(platform_id);
    }
    let mut measures = vec![fx.sessions, fx.units];
    for index in 0..3 {
        measures.push(insert_measure(
            &fx.pool,
            &format!("plan_measure_{index}"),
            true,
            true,
        ));
    }
    let driver = Uuid::new_v4();
    let mut accounts = source_sql(driver, "DRIVER", true);
    for platform_id in &platforms[1..] {
        accounts.push_str(&account_sql(
            Uuid::new_v4(),
            driver,
            *platform_id,
            Some(fx.publisher_id),
            true,
        ));
    }
    for publisher_id in [fx.other_publisher_id, third_publisher] {
        accounts.push_str(&account_sql(
            Uuid::new_v4(),
            driver,
            fx.platform_id,
            Some(publisher_id),
            true,
        ));
    }

    let statements = [
        publisher_sql(third_publisher, "SPHINX"),
        accounts,
        format!(
            "INSERT INTO imprint (imprint_id, publisher_id, imprint_name) VALUES \
             ('{imprint_a}', '{a}', 'Plan A'), ('{imprint_b}', '{b}', 'Plan B'), \
             ('{imprint_c}', '{third_publisher}', 'Plan C');",
            a = fx.publisher_id,
            b = fx.other_publisher_id,
        ),
        format!(
            "INSERT INTO work (work_id, work_type, work_status, imprint_id, edition) \
             SELECT gen_random_uuid(), 'monograph', 'forthcoming', imprint.id, 1 \
             FROM unnest({imprints}) AS imprint(id) CROSS JOIN generate_series(1, 600);",
            imprints = uuid_array(&[imprint_a, imprint_b, imprint_c]),
        ),
        format!(
            "INSERT INTO metric_rollup_work_day \
                 (work_id, platform_id, measure_id, day, country_code, value, watermark) \
             SELECT w.work_id, '{platform}', '{sessions}', g.day::date, c.code, \
                    1 + (random() * 20)::bigint, 1 \
             FROM work w \
             CROSS JOIN generate_series(DATE '{start}', DATE '{start}' + 365, interval '1 day') AS g(day) \
             CROSS JOIN unnest(ARRAY['GB', 'US', 'DE']) AS c(code) \
             WHERE w.imprint_id = '{imprint_a}';",
            platform = fx.platform_id,
            sessions = fx.sessions,
        ),
        format!(
            "INSERT INTO metric_rollup_work_day \
                 (work_id, platform_id, measure_id, day, value, watermark) \
             SELECT w.work_id, '{platform}', '{units}', g.day::date, \
                    (random() * 10)::bigint - 3, 1 \
             FROM work w \
             CROSS JOIN generate_series(DATE '{start}', DATE '{start}' + 365, interval '7 day') AS g(day) \
             WHERE w.imprint_id = '{imprint_a}';",
            platform = fx.platform_id,
            units = fx.units,
        ),
        format!(
            "INSERT INTO metric_rollup_work_day \
                 (work_id, platform_id, measure_id, day, value, watermark) \
             SELECT w.work_id, p.id, m.id, g.day::date, 1, 1 \
             FROM work w \
             CROSS JOIN unnest({platforms}) AS p(id) \
             CROSS JOIN unnest({measures}) AS m(id) \
             CROSS JOIN generate_series(DATE '{start}', DATE '{start}' + 365, interval '30 day') AS g(day) \
             WHERE w.imprint_id = '{imprint_a}' \
               AND NOT (p.id = '{platform}' AND m.id IN ('{sessions}', '{units}'));",
            platforms = uuid_array(&platforms),
            measures = uuid_array(&measures),
            platform = fx.platform_id,
            sessions = fx.sessions,
            units = fx.units,
        ),
        format!(
            "INSERT INTO metric_rollup_work_day \
                 (work_id, platform_id, measure_id, day, country_code, value, watermark) \
             SELECT w.work_id, '{platform}', '{sessions}', g.day::date, 'FR', 1, 1 \
             FROM work w \
             CROSS JOIN generate_series(DATE '{start}', DATE '{start}' + 365, interval '1 day') AS g(day) \
             WHERE w.imprint_id IN ('{imprint_b}', '{imprint_c}');",
            platform = fx.platform_id,
            sessions = fx.sessions,
        ),
        format!(
            "INSERT INTO metric_import \
                 (import_id, source_account_id, publisher_id, format_code, format_version, \
                  status, normalizer_version, created_by, period_start, period_end, completed_at) \
             SELECT gen_random_uuid(), sa.source_account_id, sa.expected_publisher_id, 'plan', \
                    '1', 'COMPLETED', 'plan/1', 'plan', g.day::date, g.day::date + 1, \
                    g.day + interval '26 hours' \
             FROM metric_source_account sa \
             CROSS JOIN generate_series(DATE '{start}', DATE '{start}' + 365, interval '1 day') AS g(day) \
             WHERE sa.expected_publisher_id IS NOT NULL; \
             INSERT INTO metric_import \
                 (import_id, source_account_id, publisher_id, format_code, format_version, \
                  status, normalizer_version, created_by, period_start, period_end, completed_at) \
             SELECT gen_random_uuid(), '{account}', '{publisher}', 'plan', '1', \
                    'COMPLETED_WITH_ERRORS', 'plan/1', 'plan', g.day::date, g.day::date + 30, \
                    g.day + interval '40 days' \
             FROM generate_series(DATE '{start}', DATE '{start}' + 365, interval '30 day') AS g(day); \
             INSERT INTO metric_coverage \
                 (source_account_id, import_id, platform_id, measure_id, period_start, \
                  period_end, coverage_status, country_coverage, institution_coverage) \
             SELECT mi.source_account_id, mi.import_id, sa.platform_id, m.id, \
                    mi.period_start, mi.period_end, 'COMPLETE', TRUE, FALSE \
             FROM metric_import mi \
             JOIN metric_source_account sa ON sa.source_account_id = mi.source_account_id \
             CROSS JOIN unnest({measures}) AS m(id) \
             WHERE mi.format_code = 'plan';",
            account = fx.account_id,
            publisher = fx.publisher_id,
            measures = uuid_array(&measures),
        ),
        format!(
            "INSERT INTO metric_record \
                 (record_id, identity_hash, work_id, platform_id, measure_id, period_start, \
                  period_end, reporting_grain, country_code, winning_source_account_id) \
             SELECT gen_random_uuid(), 'plan-backlog-' || w.imprint_id || '-' || w.rn || '-' || g.n, \
                    w.work_id, '{platform}', '{sessions}', DATE '{start}' + ((w.rn + g.n) % 366)::int, \
                    DATE '{start}' + ((w.rn + g.n) % 366)::int + 1, 'DAY', 'NL', '{account}' \
             FROM (SELECT work_id, imprint_id, \
                          row_number() OVER (PARTITION BY imprint_id ORDER BY work_id) AS rn \
                   FROM work WHERE imprint_id IN ('{imprint_a}', '{imprint_b}')) w \
             CROSS JOIN generate_series(1, 5) AS g(n) \
             WHERE (w.imprint_id = '{imprint_a}' AND w.rn <= 400) \
                OR (w.imprint_id = '{imprint_b}' AND w.rn <= 600); \
             INSERT INTO metric_record_revision \
                 (record_revision_id, record_id, revision_number, import_id, value, \
                  content_hash, status) \
             SELECT gen_random_uuid(), r.record_id, 1, '{import}', 1, \
                    'plan-' || r.record_id, 'CURRENT' \
             FROM metric_record r WHERE r.identity_hash LIKE 'plan-backlog-%'; \
             INSERT INTO metric_rollup_delta (record_id, revision_id, delta_value, status) \
             SELECT rv.record_id, rv.record_revision_id, 1, 'PENDING' \
             FROM metric_record_revision rv WHERE rv.content_hash LIKE 'plan-%' \
             ORDER BY rv.record_id;",
            platform = fx.platform_id,
            sessions = fx.sessions,
            account = fx.account_id,
            import = fx.canonical_import,
        ),
        "ANALYZE;".to_string(),
    ];
    for statement in statements {
        exec(&fx.pool, &statement);
    }
    PlanGrid {
        platforms,
        measures,
        imprint: imprint_a,
    }
}

#[derive(diesel::QueryableByName)]
struct PlanLine {
    #[diesel(sql_type = Text, column_name = "QUERY PLAN")]
    line: String,
}

/// `EXPLAIN (ANALYZE, BUFFERS)` of one named resolver statement, executed as
/// a prepared statement with the given typed arguments inside a read-only
/// repeatable-read transaction, as the resolver runs it.
fn explain(name: &str, statement: &str, parameter_types: &str, arguments: &str) {
    explain_with(name, "", statement, parameter_types, arguments);
}

/// [`explain`] with transaction-local planner settings, used diagnostically to
/// show the plan PostgreSQL would use when a sequential scan is not chosen.
fn explain_with(
    name: &str,
    settings: &str,
    statement: &str,
    parameter_types: &str,
    arguments: &str,
) {
    let mut connection =
        PgConnection::establish(&test_db_url()).expect("Failed to open the plan connection");
    connection
        .batch_execute("BEGIN ISOLATION LEVEL REPEATABLE READ READ ONLY;")
        .expect("begin the plan transaction");
    if !settings.is_empty() {
        connection
            .batch_execute(settings)
            .expect("apply planner settings");
    }
    let prepare = if parameter_types.is_empty() {
        format!("PREPARE plan_statement AS {statement}")
    } else {
        format!("PREPARE plan_statement ({parameter_types}) AS {statement}")
    };
    connection
        .batch_execute(&prepare)
        .expect("prepare the statement");
    let execute = if arguments.is_empty() {
        "EXPLAIN (ANALYZE, BUFFERS) EXECUTE plan_statement".to_string()
    } else {
        format!("EXPLAIN (ANALYZE, BUFFERS) EXECUTE plan_statement ({arguments})")
    };
    let plan: Vec<PlanLine> = sql_query(execute)
        .load(&mut connection)
        .expect("explain the statement");
    println!("\n---- {name} ----");
    for line in plan {
        println!("{}", line.line);
    }
    connection
        .batch_execute("ROLLBACK;")
        .expect("end the plan transaction");
}

/// The host load average, recorded beside latency samples.
fn load_average() -> String {
    std::process::Command::new("uptime")
        .output()
        .map(|output| format!("uptime: {}", String::from_utf8_lossy(&output.stdout).trim()))
        .unwrap_or_else(|_| "uptime: unavailable".to_string())
}

/// Resolver wall-clock latency over repeated requests, after warm-up.
fn latency(fx: &Fixture, label: &str, input: &MetricDashboardInput) {
    for _ in 0..3 {
        metric_dashboard(&fx.pool, input).expect("warm-up read");
    }
    let mut samples: Vec<StdDuration> = (0..40)
        .map(|_| {
            let started = Instant::now();
            let dashboard = metric_dashboard(&fx.pool, input).expect("measured read");
            let elapsed = started.elapsed();
            assert!(!dashboard.totals.is_empty());
            elapsed
        })
        .collect();
    samples.sort();
    let percentile = |p: f64| samples[((samples.len() as f64 * p).ceil() as usize).max(1) - 1];
    let dashboard = metric_dashboard(&fx.pool, input).expect("shape read");
    println!(
        "{label}: n=40 p50={:?} p95={:?} max={:?} | totals={} timeline_cells={} coverage={:?} warnings={:?}",
        percentile(0.50),
        percentile(0.95),
        samples[samples.len() - 1],
        dashboard.totals.len(),
        dashboard.timeline.len(),
        dashboard.coverage.status,
        codes(&dashboard),
    );
}

#[test]
#[ignore = "query-plan and latency evidence; run explicitly with --ignored"]
fn metric_dashboard_query_plan_and_latency_evidence() {
    let (_guard, fx) = setup();
    let built = Instant::now();
    let grid = plan_fixture(&fx);
    println!("plan fixture built in {:?}", built.elapsed());
    for (label, query) in [
        (
            "selected publisher works",
            format!(
                "(SELECT COUNT(*) FROM work w JOIN imprint i ON i.imprint_id = w.imprint_id \
                  WHERE i.publisher_id = '{}')",
                fx.publisher_id
            ),
        ),
        ("all works", "(SELECT COUNT(*) FROM work)".to_string()),
        (
            "metric_rollup_work_day rows",
            "(SELECT COUNT(*) FROM metric_rollup_work_day)".to_string(),
        ),
        (
            "metric_coverage rows",
            "(SELECT COUNT(*) FROM metric_coverage)".to_string(),
        ),
        (
            "metric_import rows",
            "(SELECT COUNT(*) FROM metric_import)".to_string(),
        ),
        (
            "unapplied work-day deltas",
            "(SELECT COUNT(*) FROM metric_rollup_delta WHERE status <> 'APPLIED')".to_string(),
        ),
    ] {
        println!("{label}: {}", scalar_i64(&fx.pool, &query));
    }

    let start = plan_start();
    let publishers = uuid_array(&[fx.publisher_id]);
    let one_platform = uuid_array(&[fx.platform_id]);
    let one_measure = uuid_array(&[fx.sessions]);
    let all_platforms = uuid_array(&grid.platforms);
    let all_measures = uuid_array(&grid.measures);
    let year_end = start + Duration::days(366);
    let grid_end = start + Duration::days(200);
    let account_ids: Vec<Uuid> = {
        #[derive(diesel::QueryableByName)]
        struct Row {
            #[diesel(sql_type = SqlUuid)]
            source_account_id: Uuid,
        }
        let mut connection = fx.pool.get().expect("Failed to get DB connection");
        sql_query(format!(
            "SELECT source_account_id FROM metric_source_account \
             WHERE expected_publisher_id = '{}'",
            fx.publisher_id
        ))
        .load::<Row>(&mut connection)
        .expect("the selected publisher's accounts")
        .into_iter()
        .map(|row| row.source_account_id)
        .collect()
    };
    let accounts = uuid_array(&account_ids);

    explain("PUBLISHERS_SQL", PUBLISHERS_SQL, "uuid[]", &publishers);
    explain("FRONTIER_SQL", FRONTIER_SQL, "", "");
    explain(
        "ACCOUNTS_SQL (5 platforms)",
        ACCOUNTS_SQL,
        "uuid[], uuid[], text",
        &format!("{publishers}, {all_platforms}, 'DRIVER'"),
    );
    explain(
        "DAY_SUMS_SQL (366 days, 1 platform x 1 measure)",
        DAY_SUMS_SQL,
        "uuid[], date, date, uuid[], uuid[]",
        &format!("{publishers}, '{start}', '{year_end}', {one_platform}, {one_measure}"),
    );
    explain(
        "DAY_SUMS_SQL (200 days, 5 platforms x 5 measures)",
        DAY_SUMS_SQL,
        "uuid[], date, date, uuid[], uuid[]",
        &format!("{publishers}, '{start}', '{grid_end}', {all_platforms}, {all_measures}"),
    );
    explain(
        "ASSERTIONS_SQL (366 days, 1 platform x 1 measure)",
        ASSERTIONS_SQL,
        "uuid[], date, date, uuid[], uuid[]",
        &format!(
            "{}, '{start}', '{year_end}', {one_platform}, {one_measure}",
            uuid_array(&[fx.account_id])
        ),
    );
    explain(
        "ASSERTIONS_SQL (200 days, 5 platforms x 5 measures)",
        ASSERTIONS_SQL,
        "uuid[], date, date, uuid[], uuid[]",
        &format!("{accounts}, '{start}', '{grid_end}', {all_platforms}, {all_measures}"),
    );
    explain(
        "LAG_SQL (366 days, 1 platform x 1 measure, W = 0)",
        LAG_SQL,
        "bigint, uuid[], date, date, uuid[], uuid[]",
        &format!("0, {publishers}, '{start}', '{year_end}', {one_platform}, {one_measure}"),
    );
    explain(
        "REPRESENTED_SQL (366 days, both filters omitted)",
        REPRESENTED_SQL,
        "uuid[], date, date, uuid[], uuid[], text",
        &format!(
            "{publishers}, '{start}', '{year_end}', ARRAY[]::uuid[], ARRAY[]::uuid[], 'DRIVER'"
        ),
    );
    explain(
        "IDENTIFIER_QUALITY_SQL (366 days, 5 platforms x 5 measures)",
        IDENTIFIER_QUALITY_SQL,
        "uuid[], date, date, uuid[], uuid[]",
        &format!(
            "{publishers}, '{start}', '{year_end}', {all_platforms}, {all_measures}"
        ),
    );

    // Diagnostic only: the same statements with sequential scans disabled,
    // showing that the existing MET-WP4-01 projection indexes and the
    // existing attribution indexes can serve them when the selected
    // publisher is a small fraction of the projection. In this fixture the
    // selected publisher holds about half of all projection rows, so the
    // planner's own choice above is a parallel sequential scan.
    let no_seqscan = "SET LOCAL enable_seqscan = off;";
    explain_with(
        "DAY_SUMS_SQL (366 days, 1 platform x 1 measure) [enable_seqscan = off]",
        no_seqscan,
        DAY_SUMS_SQL,
        "uuid[], date, date, uuid[], uuid[]",
        &format!("{publishers}, '{start}', '{year_end}', {one_platform}, {one_measure}"),
    );
    explain_with(
        "DAY_SUMS_SQL (200 days, 5 platforms x 5 measures) [enable_seqscan = off]",
        no_seqscan,
        DAY_SUMS_SQL,
        "uuid[], date, date, uuid[], uuid[]",
        &format!("{publishers}, '{start}', '{grid_end}', {all_platforms}, {all_measures}"),
    );
    explain_with(
        "LAG_SQL (366 days, 1 platform x 1 measure, W = 0) [enable_seqscan = off]",
        no_seqscan,
        LAG_SQL,
        "bigint, uuid[], date, date, uuid[], uuid[]",
        &format!("0, {publishers}, '{start}', '{year_end}', {one_platform}, {one_measure}"),
    );

    let run_shapes = |phase: &str| {
        println!("\n== {phase} ==");
        println!("{}", load_average());
        explain(
            &format!("DAY_SUMS_SQL (366 days, 5 platforms x 5 measures) [{phase}]"),
            DAY_SUMS_SQL,
            "uuid[], date, date, uuid[], uuid[]",
            &format!("{publishers}, '{start}', '{year_end}', {all_platforms}, {all_measures}"),
        );
        latency(
            &fx,
            "366 days, 1 platform x 1 measure, DAY (366 cells)",
            &request(
                fx.publisher_id,
                start,
                year_end,
                &[fx.platform_id],
                &[fx.sessions],
                Some(MetricTimelineGrain::Day),
            ),
        );
        latency(
            &fx,
            "200 days, 5 platforms x 5 measures, DAY (5000 cells)",
            &request(
                fx.publisher_id,
                start,
                grid_end,
                &grid.platforms,
                &grid.measures,
                Some(MetricTimelineGrain::Day),
            ),
        );
        latency(
            &fx,
            "366 days, 5 platforms x 5 measures, MONTH (325 cells)",
            &request(
                fx.publisher_id,
                start,
                year_end,
                &grid.platforms,
                &grid.measures,
                Some(MetricTimelineGrain::Month),
            ),
        );
        let mut omitted = request(
            fx.publisher_id,
            start,
            year_end,
            &[],
            &[],
            Some(MetricTimelineGrain::Month),
        );
        omitted.platforms = None;
        omitted.measures = None;
        latency(
            &fx,
            "366 days, filters omitted (resolves 5 x 5), MONTH",
            &omitted,
        );
        println!("{}", load_average());
    };

    // Representative: every base cell has exactly one representation, as a
    // managed source reports either country rows or a total, not both.
    run_shapes("representative: one representation per base cell");

    // Stress: half the selected publisher's works additionally carry an
    // undimensioned aggregate for every day alongside their country rows, so
    // every day group needs the aggregate-precedence path.
    exec(
        &fx.pool,
        &format!(
            "INSERT INTO metric_rollup_work_day \
                 (work_id, platform_id, measure_id, day, value, watermark) \
             SELECT w.work_id, '{platform}', '{sessions}', g.day::date, 30, 1 \
             FROM (SELECT work_id, row_number() OVER (ORDER BY work_id) AS rn \
                   FROM work WHERE imprint_id = '{imprint}') w \
             CROSS JOIN generate_series(DATE '{start}', DATE '{start}' + 365, interval '1 day') AS g(day) \
             WHERE w.rn <= 300; \
             ANALYZE metric_rollup_work_day;",
            platform = fx.platform_id,
            sessions = fx.sessions,
            imprint = grid.imprint,
        ),
    );
    println!(
        "metric_rollup_work_day rows after stress rows: {}",
        scalar_i64(&fx.pool, "(SELECT COUNT(*) FROM metric_rollup_work_day)")
    );
    let candidate_days: Vec<String> = (0..366)
        .map(|offset| format!("'{}'", start + Duration::days(offset)))
        .collect();
    explain(
        "DIMENSION_CELLS_SQL (366 candidate days, 1 platform x 1 measure) [stress]",
        DIMENSION_CELLS_SQL,
        "uuid[], uuid[], uuid[], date[]",
        &format!(
            "{publishers}, {platforms}, {measures}, ARRAY[{days}]::date[]",
            platforms = uuid_array(&vec![fx.platform_id; 366]),
            measures = uuid_array(&vec![fx.sessions; 366]),
            days = candidate_days.join(","),
        ),
    );
    run_shapes("stress: aggregate plus country breakdown in half of all base cells");
}
