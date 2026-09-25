//! `MET-WP4-02` and `MET-WP4-03B` evidence for the coverage-aware dashboard
//! read.
//!
//! These tests drive [`metric_dashboard`] directly against a disposable
//! database whose projection is produced by the real `MET-WP4-01` claim and
//! completion path: exact totals and timelines, zero versus unknown, current
//! coverage selection, source-scope eligibility, rollup freshness and
//! `dataThrough`, the one-snapshot property, every request bound, additivity,
//! publisher entitlement and set-based statement counts; and, for
//! `MET-WP4-03B`, the metadata selector, several publishers, the country and
//! institution sections and their coverage, complete-month and edge-day
//! serving against an independent day-by-day oracle, native source grains and
//! resolved-work lag.
//!
//! Authorization, the GraphQL error classifications and the SDL are proven at
//! the API boundary in `crate::graphql::metric_dashboard_tests`, which reuses
//! the fixture below.
//!
//! One `#[ignore]`d test builds three production-shaped 600-work publishers
//! with metadata for every selector dimension, derives the `MET-WP4-03A`
//! monthly projections through the reviewed rebuild, and prints
//! `EXPLAIN (ANALYZE, BUFFERS)` plans, statement counts and resolver p50/p95
//! for the `MET-WP4-03B` acceptance workloads. Run it explicitly with
//! `cargo test -p thoth-api --features backend -- --ignored --nocapture
//! metric_dashboard_query_plan_and_latency_evidence`.

use std::sync::{Arc, Mutex};
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
    claim_metric_rollup_deltas, complete_metric_rollup_deltas, rebuild_month_projections,
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
            ..MetricSelectorInput::default()
        },
        start_date: start,
        end_date: end,
        measures: Some(measures.to_vec()),
        platforms: Some(platforms.to_vec()),
        timeline_grain: grain,
        include_countries: None,
        include_institutions: None,
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

/// The same request with neither dimensional section returned: the
/// `MET-WP4-02` totals-and-timeline response.
fn totals_only(input: &MetricDashboardInput) -> MetricDashboardInput {
    let mut input = input.clone();
    input.include_countries = Some(false);
    input.include_institutions = Some(false);
    input
}

/// `(measure, country, value)` of every country row, in response order.
fn countries(dashboard: &MetricDashboard) -> Vec<(Uuid, String, String)> {
    dashboard
        .countries
        .iter()
        .map(|row| {
            (
                row.measure_id,
                row.country_code.clone(),
                row.value.to_string(),
            )
        })
        .collect()
}

/// `(measure, institution, value)` of every institution row, in response
/// order.
fn institutions(dashboard: &MetricDashboard) -> Vec<(Uuid, Uuid, String)> {
    dashboard
        .institutions
        .iter()
        .map(|row| (row.measure_id, row.institution_id, row.value.to_string()))
        .collect()
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
        ..Cells::default()
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
    assert_eq!(
        item(&dashboard, fx.sessions).status,
        MetricCoverageStatus::Complete
    );
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

    let support = commit(&fx, fx.works[0], fx.platform_id, fx.units, day_n(10), 1);
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
    let record = commit(&fx, fx.works[0], fx.platform_id, fx.sessions, day_n(2), 9);
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
    // First prove that unresolved evidence outside the selected immutable
    // publisher/platform/measure/date scope does not leak into this request.
    {
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
            fx.units,
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
    }

    // Then use a clean fixture to prove historical import-publisher scope does
    // not move when the current source-account publisher or enablement changes.
    {
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

        let relevant = quarantine(
            &fx,
            fx.publisher_id,
            fx.platform_id,
            fx.sessions,
            day_n(2),
            day_n(3),
            "DAY",
        );
        let input = window(&fx, &[fx.sessions]);
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

        // Moving the current account scope to the other publisher does not
        // move the historical quarantine import with it.
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

        // Keep the variable load-bearing for the fixture and prove the
        // unresolved row itself was not rewritten by any read.
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
            let measure = insert_measure(&fx.pool, &format!("quarantine_only_{index}"), true, true);
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

/// A request for one publisher over `[start, end)` with both the platform and
/// the measure filter omitted, so the served scope is resolved entirely from
/// represented state.
fn omitted_scope(publisher_id: Uuid, start: NaiveDate, end: NaiveDate) -> MetricDashboardInput {
    let mut input = request(publisher_id, start, end, &[], &[], None);
    input.platforms = None;
    input.measures = None;
    input
}

/// Assert that a response serves exactly `platforms` x `measures`: the served
/// platform and measure sets, the totals, the coverage items and the timeline
/// buckets all name exactly those combinations and nothing else.
fn assert_serves(dashboard: &MetricDashboard, platforms: &[Uuid], measures: &[Uuid]) {
    let expected: BTreeSet<(Uuid, Uuid)> = platforms
        .iter()
        .flat_map(|platform_id| {
            measures
                .iter()
                .map(move |measure_id| (*platform_id, *measure_id))
        })
        .collect();

    let totals: Vec<(Uuid, Uuid)> = dashboard
        .totals
        .iter()
        .map(|total| (total.platform_id, total.measure_id))
        .collect();
    assert_eq!(
        totals
            .iter()
            .map(|(platform_id, _)| *platform_id)
            .collect::<BTreeSet<_>>(),
        platforms.iter().copied().collect::<BTreeSet<_>>(),
        "served platforms"
    );
    assert_eq!(
        totals
            .iter()
            .map(|(_, measure_id)| *measure_id)
            .collect::<BTreeSet<_>>(),
        measures.iter().copied().collect::<BTreeSet<_>>(),
        "served measures"
    );
    assert_eq!(totals.len(), expected.len(), "one total per combination");
    assert_eq!(
        totals.into_iter().collect::<BTreeSet<_>>(),
        expected,
        "served totals"
    );

    let items: Vec<(Uuid, Uuid)> = dashboard
        .coverage
        .items
        .iter()
        .map(|item| (item.platform_id, item.measure_id))
        .collect();
    assert_eq!(
        items.len(),
        expected.len(),
        "one coverage item per combination"
    );
    assert_eq!(
        items.into_iter().collect::<BTreeSet<_>>(),
        expected,
        "coverage items"
    );

    assert_eq!(
        dashboard
            .timeline
            .iter()
            .map(|bucket| (bucket.platform_id, bucket.measure_id))
            .collect::<BTreeSet<_>>(),
        expected,
        "timeline combinations"
    );
}

#[test]
fn unrelated_quarantine_does_not_expand_scope_when_both_dimensions_are_omitted() {
    let (_guard, fx) = setup();
    // Every unrelated row has a platform and a measure of its own, so a row
    // that leaked into represented scope would add both a served platform and
    // a served measure.
    let other_publisher_platform = Uuid::new_v4();
    let ends_at_start_platform = Uuid::new_v4();
    let starts_at_end_platform = Uuid::new_v4();
    let resolved_platform = Uuid::new_v4();
    insert_platform_row(&fx.pool, other_publisher_platform, "scope_other_publisher");
    insert_platform_row(&fx.pool, ends_at_start_platform, "scope_ends_at_start");
    insert_platform_row(&fx.pool, starts_at_end_platform, "scope_starts_at_end");
    insert_platform_row(&fx.pool, resolved_platform, "scope_resolved");
    let other_publisher_measure = insert_measure(&fx.pool, "scope_other_publisher", true, true);
    let ends_at_start_measure = insert_measure(&fx.pool, "scope_ends_at_start", true, true);
    let starts_at_end_measure = insert_measure(&fx.pool, "scope_starts_at_end", true, true);
    let resolved_measure = insert_measure(&fx.pool, "scope_resolved", true, true);

    // The one in-scope pair: unresolved, admitted under the selected
    // publisher's import, overlapping [2026-03-01, 2026-03-05), and neither
    // projected nor covered.
    quarantine(
        &fx,
        fx.publisher_id,
        fx.platform_id,
        fx.sessions,
        day_n(2),
        day_n(3),
        "DAY",
    );
    // 1. Unresolved and overlapping, but admitted under the other publisher's
    //    import.
    quarantine(
        &fx,
        fx.other_publisher_id,
        other_publisher_platform,
        other_publisher_measure,
        day_n(2),
        day_n(3),
        "DAY",
    );
    // 2. The selected publisher, ending exactly at the request start.
    quarantine(
        &fx,
        fx.publisher_id,
        ends_at_start_platform,
        ends_at_start_measure,
        d1() - Duration::days(1),
        d1(),
        "DAY",
    );
    // 3. The selected publisher, starting exactly at the request end.
    quarantine(
        &fx,
        fx.publisher_id,
        starts_at_end_platform,
        starts_at_end_measure,
        day_n(5),
        day_n(6),
        "DAY",
    );
    // 4. The selected publisher and overlapping, but terminally resolved. The
    //    supporting canonical record only satisfies #935's terminal-state
    //    shape: it lies outside every window below and is never applied, so it
    //    represents nothing.
    let resolved = quarantine(
        &fx,
        fx.publisher_id,
        resolved_platform,
        resolved_measure,
        day_n(2),
        day_n(3),
        "DAY",
    );
    let support = commit(&fx, fx.works[0], fx.platform_id, fx.units, day_n(10), 1);
    set_quarantine_reconciliation_state(&fx, resolved, "RESOLVED_WINNER", Some(support));

    let window = omitted_scope(fx.publisher_id, d1(), day_n(5));
    let dashboard = read(&fx, &window);
    assert_serves(&dashboard, &[fx.platform_id], &[fx.sessions]);
    // Quarantine-only, so nothing justifies a zero anywhere.
    assert_eq!(total(&dashboard, fx.platform_id, fx.sessions), None);
    assert_eq!(
        bucket_values(&dashboard, fx.sessions),
        vec![None, None, None, None]
    );
    assert_eq!(
        item(&dashboard, fx.sessions).status,
        MetricCoverageStatus::Unknown
    );
    assert_eq!(
        codes(&dashboard),
        vec![
            MetricWarningCode::UnknownCoverage,
            MetricWarningCode::UnresolvedIdentifiers,
        ]
    );
    assert!(dashboard.is_partial);

    // Controls: every excluded row is valid, discoverable evidence that only
    // the predicate under test keeps out.
    //
    // The other publisher's own omitted-filter request discovers its row and
    // nothing of the selected publisher's.
    let other = read(&fx, &omitted_scope(fx.other_publisher_id, d1(), day_n(5)));
    assert_serves(
        &other,
        &[other_publisher_platform],
        &[other_publisher_measure],
    );
    assert!(codes(&other).contains(&MetricWarningCode::UnresolvedIdentifiers));

    // One more day on each side takes in both boundary rows, but still neither
    // the other publisher's row nor the resolved one.
    let widened = read(
        &fx,
        &omitted_scope(fx.publisher_id, d1() - Duration::days(1), day_n(6)),
    );
    assert_serves(
        &widened,
        &[
            fx.platform_id,
            ends_at_start_platform,
            starts_at_end_platform,
        ],
        &[fx.sessions, ends_at_start_measure, starts_at_end_measure],
    );

    // A nonterminal state for the same row makes it represented again.
    set_quarantine_reconciliation_state(&fx, resolved, "PENDING_UNKNOWN_DOI", None);
    let reopened = read(&fx, &window);
    assert_serves(
        &reopened,
        &[fx.platform_id, resolved_platform],
        &[fx.sessions, resolved_measure],
    );
}

#[test]
fn quarantine_outside_an_explicit_dimension_does_not_expand_the_omitted_one() {
    let (_guard, fx) = setup();
    let other_platform_measure = insert_measure(&fx.pool, "scope_other_platform", true, true);
    // In scope on the fixture platform and sessions measure.
    quarantine(
        &fx,
        fx.publisher_id,
        fx.platform_id,
        fx.sessions,
        day_n(2),
        day_n(3),
        "DAY",
    );
    // The same publisher and window, but on the other platform with a
    // measure that nothing else represents.
    quarantine(
        &fx,
        fx.publisher_id,
        fx.other_platform_id,
        other_platform_measure,
        day_n(2),
        day_n(3),
        "DAY",
    );

    // Control: with both dimensions omitted the second row is represented, so
    // it is valid in-window evidence of the selected publisher.
    let both_omitted = read(&fx, &omitted_scope(fx.publisher_id, d1(), day_n(5)));
    assert_serves(
        &both_omitted,
        &[fx.platform_id, fx.other_platform_id],
        &[fx.sessions, other_platform_measure],
    );

    // Explicit platform, omitted measures: the other platform's row cannot
    // add its measure.
    let mut platform_explicit = omitted_scope(fx.publisher_id, d1(), day_n(5));
    platform_explicit.platforms = Some(vec![fx.platform_id]);
    let dashboard = read(&fx, &platform_explicit);
    assert_serves(&dashboard, &[fx.platform_id], &[fx.sessions]);
    assert_eq!(total(&dashboard, fx.platform_id, fx.sessions), None);
    assert_eq!(
        codes(&dashboard),
        vec![
            MetricWarningCode::UnknownCoverage,
            MetricWarningCode::UnresolvedIdentifiers,
        ]
    );

    // Omitted platforms, explicit measure: the other measure's row cannot add
    // its platform.
    let mut measure_explicit = omitted_scope(fx.publisher_id, d1(), day_n(5));
    measure_explicit.measures = Some(vec![fx.sessions]);
    let dashboard = read(&fx, &measure_explicit);
    assert_serves(&dashboard, &[fx.platform_id], &[fx.sessions]);
    assert_eq!(total(&dashboard, fx.platform_id, fx.sessions), None);
    assert_eq!(
        codes(&dashboard),
        vec![
            MetricWarningCode::UnknownCoverage,
            MetricWarningCode::UnresolvedIdentifiers,
        ]
    );
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
    // The MET-WP4-02 totals-and-timeline response: neither dimensional
    // section is returned, so only the totals' own dependencies count.
    let aggregate = read(&fx, &totals_only(&input));
    assert_eq!(aggregate.coverage.status, MetricCoverageStatus::Complete);
    assert_eq!(
        bucket_values(&aggregate, fx.sessions),
        vec![some("9"), some("0"), some("0"), some("0")]
    );
    // #946 Amendment 1 section 2.6 item 1: requesting the country section,
    // which intrinsically needs country coverage, downgrades the shared
    // completeness for days 3-4, while the totals, which do not depend on
    // country, keep exactly the same zeros.
    let with_countries = read(&fx, &input);
    assert_eq!(
        bucket_values(&with_countries, fx.sessions),
        bucket_values(&aggregate, fx.sessions)
    );
    assert_eq!(with_countries.totals, aggregate.totals);
    assert_eq!(
        item(&with_countries, fx.sessions).status,
        MetricCoverageStatus::Partial
    );
    assert_eq!(
        codes(&with_countries),
        vec![MetricWarningCode::PartialCoverage]
    );
    assert_eq!(with_countries.data_through, Some(day_n(2)));
    // Item 2: omitting only the country section removes that downgrade;
    // the institution section is fully covered here.
    let mut no_countries = input.clone();
    no_countries.include_countries = Some(false);
    let no_countries = read(&fx, &no_countries);
    assert_eq!(no_countries.coverage.status, MetricCoverageStatus::Complete);
    assert!(no_countries.countries.is_empty());

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
    // Item 2, second half: once the totals themselves depend on country,
    // omitting the country section does not remove that dependency.
    let by_country = read(&fx, &totals_only(&input));
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
    // With the sections returned the totals and timeline are unchanged, and
    // the one known country value is listed although coverage is PARTIAL.
    let returned = read(&fx, &input);
    assert_eq!(returned.totals, by_country.totals);
    assert_eq!(returned.timeline, by_country.timeline);
    assert_eq!(
        countries(&returned),
        vec![(fx.sessions, "GB".to_string(), "4".to_string())]
    );
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

    let dashboard = read(&fx, &totals_only(&input));
    assert_eq!(
        item(&dashboard, fx.sessions).status,
        MetricCoverageStatus::Partial
    );
    assert_eq!(
        bucket_values(&dashboard, fx.sessions),
        vec![some("0"), some("0"), some("6"), None]
    );
    assert_eq!(read(&fx, &input).timeline, dashboard.timeline);
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
    let by_publication = read(&fx, &totals_only(&other));
    assert_eq!(
        by_publication.coverage.status,
        MetricCoverageStatus::Complete
    );
    assert_eq!(
        bucket_values(&by_publication, fx.units),
        vec![some("2"), some("0")]
    );
    // #946 Amendment 1 section 2.6 item 3: the returned sections need their
    // own dimensions, which these days do not assert, so the shared status
    // is downgraded while the publication-based values are unchanged.
    let returned = read(&fx, &other);
    assert_eq!(returned.coverage.status, MetricCoverageStatus::Partial);
    assert_eq!(returned.timeline, by_publication.timeline);
    let mut institutions_only = other.clone();
    institutions_only.include_countries = Some(false);
    assert_eq!(
        read(&fx, &institutions_only).coverage.status,
        MetricCoverageStatus::Partial,
        "the institution section alone still needs institution coverage"
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

    // The one non-query statement only sets a planner option for this
    // transaction.
    assert_eq!(
        CUSTOM_PLANS_SQL,
        "SET LOCAL plan_cache_mode = force_custom_plan"
    );
    for (name, statement) in READ_STATEMENTS {
        let upper = statement.to_uppercase();
        assert!(
            upper.trim_start().starts_with("SELECT") || upper.trim_start().starts_with("WITH"),
            "{name} must be a query"
        );
        for forbidden in [
            "INSERT",
            "UPDATE",
            "DELETE",
            "FOR SHARE",
            "LOCK",
            ";",
            "NEXTVAL",
            "SETVAL",
            "SET ",
        ] {
            assert!(
                !upper.contains(forbidden),
                "{name} must not contain `{forbidden}`"
            );
        }
    }
    assert_eq!(
        source.matches("diesel::sql_query(").count(),
        READ_STATEMENTS.len() + 1,
        "every statement the read executes is one of the named constants"
    );
}

/// Every query the read may execute, by name.
const READ_STATEMENTS: [(&str, &str); 12] = [
    ("PUBLISHERS_SQL", PUBLISHERS_SQL),
    ("WORKS_SQL", WORKS_SQL),
    ("KNOWN_SQL", KNOWN_SQL),
    ("REPRESENTED_SQL", REPRESENTED_SQL),
    ("ACCOUNTS_SQL", ACCOUNTS_SQL),
    ("CANONICAL_SQL", CANONICAL_SQL),
    ("MONTHS_SQL", MONTHS_SQL),
    ("DAY_SUMS_SQL", DAY_SUMS_SQL),
    ("DIMENSION_CELLS_SQL", DIMENSION_CELLS_SQL),
    ("DAY_SECTIONS_SQL", DAY_SECTIONS_SQL),
    ("ASSERTIONS_SQL", ASSERTIONS_SQL),
    ("IDENTIFIER_QUALITY_SQL", IDENTIFIER_QUALITY_SQL),
];

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
        ("four publishers", Some(ids(4))),
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
    input.selector.publisher_ids = Some(vec![publisher, Uuid::new_v4(), publisher]);
    cases.push((
        "the same publisher twice",
        input,
        MetricReadError::QueryInvalid(DUPLICATE_PUBLISHER),
    ));
    // Every bounded selector list: a duplicate is invalid, and one value
    // beyond its bound exceeds a limit, before any database access.
    let uuid_lists: [(&str, SetIds, usize, &str, &str); 5] = [
        (
            "imprintIds",
            |selector, ids| selector.imprint_ids = Some(ids),
            50,
            DUPLICATE_IMPRINT,
            TOO_MANY_IMPRINTS,
        ),
        (
            "seriesIds",
            |selector, ids| selector.series_ids = Some(ids),
            50,
            DUPLICATE_SERIES,
            TOO_MANY_SERIES,
        ),
        (
            "workIds",
            |selector, ids| selector.work_ids = Some(ids),
            500,
            DUPLICATE_WORK,
            TOO_MANY_WORKS,
        ),
        (
            "fundingInstitutionIds",
            |selector, ids| selector.funding_institution_ids = Some(ids),
            50,
            DUPLICATE_FUNDING,
            TOO_MANY_FUNDING,
        ),
        (
            "affiliationInstitutionIds",
            |selector, ids| selector.affiliation_institution_ids = Some(ids),
            50,
            DUPLICATE_AFFILIATION,
            TOO_MANY_AFFILIATION,
        ),
    ];
    for (label, set, max, duplicate_message, limit_message) in uuid_lists {
        let mut input = base.clone();
        let repeated = Uuid::new_v4();
        set(&mut input.selector, vec![repeated, repeated]);
        cases.push((
            label,
            input,
            MetricReadError::QueryInvalid(duplicate_message),
        ));
        let mut input = base.clone();
        set(&mut input.selector, ids(max + 1));
        cases.push((
            label,
            input,
            MetricReadError::QueryLimitExceeded(limit_message),
        ));
    }
    let mut input = base.clone();
    input.selector.work_types = Some(vec![WorkType::Monograph, WorkType::Monograph]);
    cases.push((
        "workTypes",
        input,
        MetricReadError::QueryInvalid(DUPLICATE_WORK_TYPE),
    ));
    let mut input = base.clone();
    input.selector.languages = Some(vec![LanguageCode::Eng, LanguageCode::Eng]);
    cases.push((
        "languages",
        input,
        MetricReadError::QueryInvalid(DUPLICATE_LANGUAGE),
    ));
    let mut input = base.clone();
    input.selector.languages = Some(distinct_languages(51));
    cases.push((
        "languages",
        input,
        MetricReadError::QueryLimitExceeded(TOO_MANY_LANGUAGES),
    ));
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

    // The accepted edges reach the database, which here is unreachable:
    // three publishers and every selector list at its bound.
    let mut input = base.clone();
    input.end_date = input.start_date + Duration::days(366);
    input.measures = Some(ids(10));
    input.platforms = Some(ids(10));
    input.selector = MetricSelectorInput {
        publisher_ids: Some(ids(3)),
        imprint_ids: Some(ids(50)),
        series_ids: Some(ids(50)),
        work_ids: Some(ids(500)),
        work_types: Some(vec![
            WorkType::BookChapter,
            WorkType::Monograph,
            WorkType::EditedBook,
            WorkType::Textbook,
            WorkType::JournalIssue,
            WorkType::BookSet,
        ]),
        languages: Some(distinct_languages(50)),
        funding_institution_ids: Some(ids(50)),
        affiliation_institution_ids: Some(ids(50)),
    };
    assert_eq!(
        metric_dashboard(&unreachable, &input),
        Err(MetricReadError::Unavailable)
    );
    // Work types have only six values, so the 50-value bound can never be
    // reached without a duplicate: 51 values are always refused.
    let mut input = base.clone();
    input.selector.work_types = Some(vec![WorkType::Monograph; 51]);
    assert_eq!(
        metric_dashboard(&unreachable, &input),
        Err(MetricReadError::QueryInvalid(DUPLICATE_WORK_TYPE))
    );
    // An empty optional list is unrestricted, never an error.
    let mut input = base.clone();
    input.selector.imprint_ids = Some(vec![]);
    input.selector.languages = Some(vec![]);
    assert_eq!(
        metric_dashboard(&unreachable, &input),
        Err(MetricReadError::Unavailable)
    );
}

/// Sets one selector list to the given IDs.
type SetIds = fn(&mut MetricSelectorInput, Vec<Uuid>);
/// Sets one selector list to a single ID.
type SetId = fn(&mut MetricSelectorInput, Uuid);
/// Edits a selector.
type EditSelector = Box<dyn Fn(&mut MetricSelectorInput)>;

/// `count` distinct languages, parsed from their database labels.
fn distinct_languages(count: usize) -> Vec<LanguageCode> {
    [
        "AAR", "ABK", "ACE", "ACH", "ADA", "ADY", "AFA", "AFH", "AFR", "AIN", "AKA", "AKK", "ALE",
        "ALG", "ALT", "AMH", "ANG", "ANP", "APA", "ARA", "ARC", "ARG", "ARN", "ARP", "ART", "ARW",
        "ASM", "AST", "ATH", "AUS", "AVA", "AVE", "AWA", "AYM", "AZE", "BAD", "BAI", "BAK", "BAL",
        "BAM", "BAN", "BAS", "BAT", "BEJ", "BEL", "BEM", "BEN", "BER", "BHO", "BIK", "BIN", "BIS",
        "BLA", "BNT",
    ]
    .iter()
    .take(count)
    .map(|code| code.parse().expect("a language code"))
    .collect()
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
struct CaptureStatements(Arc<Mutex<Vec<String>>>);

impl CustomizeConnection<PgConnection, diesel::r2d2::Error> for CaptureStatements {
    fn on_acquire(&self, connection: &mut PgConnection) -> Result<(), diesel::r2d2::Error> {
        let log = Arc::clone(&self.0);
        connection.set_instrumentation(move |event: InstrumentationEvent<'_>| {
            if let InstrumentationEvent::StartQuery { query, .. } = event {
                log.lock()
                    .expect("the statement log")
                    .push(query.to_string());
            }
        });
        Ok(())
    }
}

/// Every statement one read sends to PostgreSQL, transaction control
/// included, in order, on a fresh connection whose establishment is not
/// counted.
fn captured_statements(
    input: &MetricDashboardInput,
) -> (Result<MetricDashboard, MetricReadError>, Vec<String>) {
    let log = Arc::new(Mutex::new(Vec::new()));
    let pool = diesel::r2d2::Pool::builder()
        .max_size(1)
        .connection_customizer(Box::new(CaptureStatements(Arc::clone(&log))))
        .build(ConnectionManager::<PgConnection>::new(test_db_url()))
        .expect("a capturing pool");
    drop(pool.get().expect("a capturing connection"));
    log.lock().expect("the statement log").clear();
    let result = metric_dashboard(&pool, input);
    let statements = log.lock().expect("the statement log").clone();
    (result, statements)
}

fn counted_statements(input: &MetricDashboardInput) -> usize {
    let (result, statements) = captured_statements(input);
    result.expect("the dashboard read");
    statements.len()
}

/// The name of each captured statement: a named constant, or the
/// transaction control Diesel issues.
fn statement_names(statements: &[String]) -> Vec<&'static str> {
    statements
        .iter()
        .map(|statement| {
            // The pool's own connection check on checkout, which the
            // production pool also runs: counted, although no read issues it.
            if statement.starts_with("SELECT 1 ") {
                return "CHECKOUT_CHECK";
            }
            if statement.starts_with("BEGIN") {
                return "BEGIN";
            }
            if statement.starts_with("COMMIT") {
                return "COMMIT";
            }
            if statement.starts_with("ROLLBACK") {
                return "ROLLBACK";
            }
            if statement.starts_with(CUSTOM_PLANS_SQL) {
                return "CUSTOM_PLANS_SQL";
            }
            READ_STATEMENTS
                .iter()
                .find(|(_, sql)| statement.starts_with(sql))
                .map(|(name, _)| *name)
                .unwrap_or_else(|| panic!("an unnamed statement was executed: {statement}"))
        })
        .collect()
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

    // The pool's checkout check, transaction control, custom plans,
    // publishers and frontier, works, registry, represented scope, accounts,
    // canonical state, day values, edge sections, coverage and identifier
    // quality: a fixed set.
    assert_eq!(small, 14, "{small} statements");
    assert_eq!(
        large,
        small + 1,
        "only the one base-cell resolution statement is added once mixed \
         dimensional representations exist; backlog is read by the canonical \
         statement that always runs"
    );
    // A year adds only the one monthly statement for its complete months.
    assert_eq!(wide, large + 1);
    assert!(wide <= 16);

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
// MET-WP4-03B: metadata selector
// ==========================================================================

/// The one UUID a query returns.
fn uuid_query(pool: &PgPool, query: &str) -> Uuid {
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = SqlUuid)]
        id: Uuid,
    }
    let mut connection = pool.get().expect("Failed to get DB connection");
    sql_query(query)
        .get_result::<Row>(&mut connection)
        .unwrap_or_else(|error| panic!("fixture query failed: {error}\n{query}"))
        .id
}

fn imprint_of(fx: &Fixture, work_id: Uuid) -> Uuid {
    uuid_query(
        &fx.pool,
        &format!("SELECT imprint_id AS id FROM work WHERE work_id = '{work_id}'"),
    )
}

/// A new imprint of a publisher.
fn add_imprint(fx: &Fixture, publisher_id: Uuid) -> Uuid {
    let imprint_id = Uuid::new_v4();
    exec(
        &fx.pool,
        &format!(
            "INSERT INTO imprint (imprint_id, publisher_id, imprint_name) \
             VALUES ('{imprint_id}', '{publisher_id}', 'Imprint {imprint_id}');"
        ),
    );
    imprint_id
}

/// A new work of the given database work type in an imprint.
fn add_work(fx: &Fixture, imprint_id: Uuid, work_type: &str) -> Uuid {
    let work_id = Uuid::new_v4();
    let edition = if work_type == "book-chapter" {
        "NULL"
    } else {
        "1"
    };
    exec(
        &fx.pool,
        &format!(
            "INSERT INTO work (work_id, work_type, work_status, imprint_id, edition) \
             VALUES ('{work_id}', '{work_type}', 'forthcoming', '{imprint_id}', {edition});"
        ),
    );
    work_id
}

fn add_series(fx: &Fixture, imprint_id: Uuid) -> Uuid {
    let series_id = Uuid::new_v4();
    exec(
        &fx.pool,
        &format!(
            "INSERT INTO series (series_id, series_type, series_name, imprint_id) \
             VALUES ('{series_id}', 'book-series', 'Series {series_id}', '{imprint_id}');"
        ),
    );
    series_id
}

fn add_issue(fx: &Fixture, series_id: Uuid, work_id: Uuid, ordinal: i32) {
    exec(
        &fx.pool,
        &format!(
            "INSERT INTO issue (series_id, work_id, issue_ordinal) \
             VALUES ('{series_id}', '{work_id}', {ordinal});"
        ),
    );
}

/// Relate a chapter to its parent work, stored in both directions as Thoth
/// stores work relations.
fn add_child_of(fx: &Fixture, chapter: Uuid, parent: Uuid, ordinal: i32) {
    exec(
        &fx.pool,
        &format!(
            "INSERT INTO work_relation (relator_work_id, related_work_id, relation_type, \
                                        relation_ordinal) \
             VALUES ('{chapter}', '{parent}', 'is-child-of', 1), \
                    ('{parent}', '{chapter}', 'has-child', {ordinal});"
        ),
    );
}

fn add_language(fx: &Fixture, work_id: Uuid, code: &str, relation: &str) {
    exec(
        &fx.pool,
        &format!(
            "INSERT INTO language (work_id, language_code, language_relation) \
             VALUES ('{work_id}', '{code}', '{relation}');"
        ),
    );
}

fn add_funding(fx: &Fixture, work_id: Uuid, institution_id: Uuid) {
    exec(
        &fx.pool,
        &format!(
            "INSERT INTO funding (work_id, institution_id) \
             VALUES ('{work_id}', '{institution_id}');"
        ),
    );
}

/// A contribution to a work affiliated with an institution.
fn add_affiliation(fx: &Fixture, work_id: Uuid, institution_id: Uuid) {
    let contributor_id = Uuid::new_v4();
    let contribution_id = Uuid::new_v4();
    exec(
        &fx.pool,
        &format!(
            "INSERT INTO contributor (contributor_id, last_name, full_name) \
             VALUES ('{contributor_id}', 'Author', 'An Author'); \
             INSERT INTO contribution (contribution_id, work_id, contributor_id, \
                                       contribution_type, main_contribution, last_name, \
                                       full_name, contribution_ordinal) \
             VALUES ('{contribution_id}', '{work_id}', '{contributor_id}', 'author', TRUE, \
                     'Author', 'An Author', 1); \
             INSERT INTO affiliation (contribution_id, institution_id, affiliation_ordinal) \
             VALUES ('{contribution_id}', '{institution_id}', 1);"
        ),
    );
}

/// The default four-day window of the fixture sessions measure for the given
/// publishers, with a selector edit.
fn selected(
    fx: &Fixture,
    publishers: &[Uuid],
    edit: impl FnOnce(&mut MetricSelectorInput),
) -> MetricDashboardInput {
    let mut input = window(fx, &[fx.sessions]);
    input.selector.publisher_ids = Some(publishers.to_vec());
    edit(&mut input.selector);
    input
}

fn sessions_total(fx: &Fixture, input: &MetricDashboardInput) -> Option<String> {
    total(&read(fx, input), fx.platform_id, fx.sessions)
}

#[test]
fn selector_dimensions_are_or_within_and_and_between_on_current_metadata() {
    let (_guard, fx) = setup();
    let first_imprint = imprint_of(&fx, fx.works[0]);
    let second_imprint = add_imprint(&fx, fx.publisher_id);
    let monograph = add_work(&fx, second_imprint, "monograph");
    let edited = add_work(&fx, second_imprint, "edited-book");
    let chapter = add_work(&fx, second_imprint, "book-chapter");
    add_child_of(&fx, chapter, monograph, 1);
    // An edited book related to the monograph is not a chapter, so it never
    // inherits the monograph's series.
    add_child_of(&fx, edited, monograph, 2);
    for (work_id, value) in [
        (fx.works[0], 1),
        (fx.works[1], 2),
        (monograph, 4),
        (edited, 8),
        (chapter, 16),
    ] {
        commit(&fx, work_id, fx.platform_id, fx.sessions, day_n(1), value);
    }
    // The other publisher's work never reaches the selected publisher.
    commit(
        &fx,
        fx.other_work,
        fx.platform_id,
        fx.sessions,
        day_n(1),
        1000,
    );
    apply_all(&fx.pool);
    let (series, other_series) = (
        add_series(&fx, second_imprint),
        add_series(&fx, first_imprint),
    );
    add_issue(&fx, series, monograph, 1);
    add_issue(&fx, other_series, fx.works[0], 1);
    add_language(&fx, fx.works[0], "eng", "original");
    add_language(&fx, fx.works[1], "fre", "translated-from");
    add_language(&fx, fx.works[1], "eng", "translated-into");
    let (funder, affiliated) = (insert_institution(&fx.pool), insert_institution(&fx.pool));
    add_funding(&fx, monograph, funder);
    add_funding(&fx, edited, funder);
    add_affiliation(&fx, fx.works[0], affiliated);
    add_affiliation(&fx, edited, affiliated);
    let publisher = [fx.publisher_id];
    let at = |edit: &dyn Fn(&mut MetricSelectorInput)| {
        sessions_total(&fx, &selected(&fx, &publisher, |selector| edit(selector)))
    };

    assert_eq!(at(&|_| {}), some("31"), "every work of the publisher");
    assert_eq!(
        at(&|s| s.imprint_ids = Some(vec![second_imprint])),
        some("28")
    );
    assert_eq!(
        at(&|s| s.imprint_ids = Some(vec![first_imprint, second_imprint])),
        some("31"),
        "OR within a list"
    );
    assert_eq!(
        at(&|s| s.work_ids = Some(vec![fx.works[0], edited])),
        some("9")
    );
    assert_eq!(
        at(&|s| s.work_types = Some(vec![WorkType::EditedBook])),
        some("8")
    );
    assert_eq!(
        at(&|s| s.work_types = Some(vec![WorkType::Monograph, WorkType::BookChapter])),
        some("23")
    );
    assert_eq!(
        at(&|s| {
            s.imprint_ids = Some(vec![second_imprint]);
            s.work_types = Some(vec![WorkType::Monograph]);
        }),
        some("4"),
        "AND between lists"
    );
    // Direct issue membership, and a chapter through its parent.
    assert_eq!(at(&|s| s.series_ids = Some(vec![series])), some("20"));
    assert_eq!(at(&|s| s.series_ids = Some(vec![other_series])), some("1"));
    assert_eq!(
        at(&|s| s.series_ids = Some(vec![series, other_series])),
        some("21")
    );
    // Any language record matches, whatever its relation.
    assert_eq!(
        at(&|s| s.languages = Some(vec![LanguageCode::Fre])),
        some("2")
    );
    assert_eq!(
        at(&|s| s.languages = Some(vec![LanguageCode::Eng])),
        some("3")
    );
    assert_eq!(
        at(&|s| s.languages = Some(vec![LanguageCode::Eng, LanguageCode::Ger])),
        some("3")
    );
    assert_eq!(
        at(&|s| s.funding_institution_ids = Some(vec![funder])),
        some("12")
    );
    assert_eq!(
        at(&|s| s.affiliation_institution_ids = Some(vec![affiliated])),
        some("9")
    );
    assert_eq!(
        at(&|s| {
            s.funding_institution_ids = Some(vec![funder]);
            s.affiliation_institution_ids = Some(vec![affiliated]);
        }),
        some("8")
    );

    // Current metadata decides: moving the edited book to the other
    // publisher removes it from every selection of this one.
    exec(
        &fx.pool,
        &format!(
            "UPDATE work SET imprint_id = '{}' WHERE work_id = '{edited}';",
            imprint_of(&fx, fx.other_work)
        ),
    );
    assert_eq!(
        at(&|s| s.funding_institution_ids = Some(vec![funder])),
        some("4")
    );
    assert_eq!(at(&|_| {}), some("23"));
}

#[test]
fn unknown_selector_ids_are_invalid_but_known_out_of_scope_ids_resolve_to_no_works() {
    let (_guard, fx) = setup();
    commit(&fx, fx.works[0], fx.platform_id, fx.sessions, day_n(1), 5);
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
    let publisher = [fx.publisher_id];
    let unknown = Uuid::new_v4();
    let edits: [(SetId, &str); 5] = [
        (|s, id| s.imprint_ids = Some(vec![id]), UNKNOWN_IMPRINT),
        (|s, id| s.series_ids = Some(vec![id]), UNKNOWN_SERIES),
        (|s, id| s.work_ids = Some(vec![id]), UNKNOWN_WORK),
        (
            |s, id| s.funding_institution_ids = Some(vec![id]),
            UNKNOWN_FUNDING,
        ),
        (
            |s, id| s.affiliation_institution_ids = Some(vec![id]),
            UNKNOWN_AFFILIATION,
        ),
    ];
    for (edit, message) in edits {
        assert_eq!(
            metric_dashboard(&fx.pool, &selected(&fx, &publisher, |s| edit(s, unknown))),
            Err(MetricReadError::QueryInvalid(message))
        );
        // One unknown ID among known ones is still refused, never dropped.
        assert_eq!(
            metric_dashboard(
                &fx.pool,
                &selected(&fx, &publisher, |s| {
                    edit(s, unknown);
                    for ids in [
                        &mut s.imprint_ids,
                        &mut s.series_ids,
                        &mut s.work_ids,
                        &mut s.funding_institution_ids,
                        &mut s.affiliation_institution_ids,
                    ]
                    .into_iter()
                    .flatten()
                    {
                        ids.push(Uuid::new_v4());
                    }
                })
            ),
            Err(MetricReadError::QueryInvalid(message))
        );
    }

    // IDs that exist but belong elsewhere, or match nothing selected, are
    // valid and leave no works.
    let other_imprint = imprint_of(&fx, fx.other_work);
    let other_series = add_series(&fx, other_imprint);
    let unrelated = insert_institution(&fx.pool);
    let other_work = fx.other_work;
    let edits: [(&str, EditSelector); 5] = [
        (
            "another publisher's imprint",
            Box::new(move |s| s.imprint_ids = Some(vec![other_imprint])),
        ),
        (
            "another publisher's series",
            Box::new(move |s| s.series_ids = Some(vec![other_series])),
        ),
        (
            "another publisher's work",
            Box::new(move |s| s.work_ids = Some(vec![other_work])),
        ),
        (
            "a funder of nothing",
            Box::new(move |s| s.funding_institution_ids = Some(vec![unrelated])),
        ),
        (
            "an affiliation of nothing",
            Box::new(move |s| s.affiliation_institution_ids = Some(vec![unrelated])),
        ),
    ];
    for (label, edit) in edits {
        let dashboard = read(&fx, &selected(&fx, &publisher, |s| edit(s)));
        assert_eq!(
            total(&dashboard, fx.platform_id, fx.sessions),
            None,
            "{label}: no works is not a zero"
        );
        assert_eq!(
            dashboard.coverage.status,
            MetricCoverageStatus::Unknown,
            "{label}"
        );
    }
}

#[test]
fn a_selection_without_works_is_the_frozen_unknown_shape() {
    let (_guard, fx) = setup();
    commit(&fx, fx.works[0], fx.platform_id, fx.sessions, day_n(1), 5);
    apply_all(&fx.pool);
    // The publisher's account asserts complete coverage for every day, but a
    // publisher without a selected work does not take part.
    cover(
        &fx,
        fx.sessions,
        d1(),
        day_n(5),
        "COMPLETE",
        "COMPLETED",
        "'2026-03-10T00:00:00Z'",
    );
    let none = |s: &mut MetricSelectorInput| {
        s.work_types = Some(vec![WorkType::JournalIssue]);
    };

    let mut omitted = selected(&fx, &[fx.publisher_id], none);
    omitted.platforms = None;
    omitted.measures = None;
    let empty = read(&fx, &omitted);
    assert!(empty.totals.is_empty());
    assert!(empty.timeline.is_empty());
    assert!(empty.countries.is_empty());
    assert!(empty.institutions.is_empty());
    assert!(empty.coverage.items.is_empty());
    assert_eq!(empty.coverage.status, MetricCoverageStatus::Unknown);
    assert_eq!(empty.data_through, None);
    assert_eq!(codes(&empty), vec![MetricWarningCode::UnknownCoverage]);
    assert!(empty.is_partial);

    // Explicit platforms and measures keep their combinations, but with no
    // represented publisher nothing is asserted: every value is null.
    let explicit = read(&fx, &selected(&fx, &[fx.publisher_id], none));
    assert_eq!(total(&explicit, fx.platform_id, fx.sessions), None);
    assert_eq!(bucket_values(&explicit, fx.sessions), vec![None; 4]);
    assert_eq!(
        item(&explicit, fx.sessions).status,
        MetricCoverageStatus::Unknown
    );
    assert!(!item(&explicit, fx.sessions).country_coverage);
    assert_eq!(explicit.data_through, None);
    assert_eq!(codes(&explicit), vec![MetricWarningCode::UnknownCoverage]);
    assert!(explicit.countries.is_empty() && explicit.institutions.is_empty());
}

#[test]
fn the_resolved_work_scope_is_bounded_at_2000_without_truncation() {
    let (_guard, fx) = setup();
    let imprint = add_imprint(&fx, fx.publisher_id);
    // 2 fixture works plus 1,998 here: exactly 2,000.
    exec(
        &fx.pool,
        &format!(
            "INSERT INTO work (work_id, work_type, work_status, imprint_id, edition) \
             SELECT gen_random_uuid(), 'monograph', 'forthcoming', '{imprint}', 1 \
             FROM generate_series(1, 1998);"
        ),
    );
    let all = selected(&fx, &[fx.publisher_id], |_| {});
    assert!(metric_dashboard(&fx.pool, &all).is_ok());
    // 500 explicit work IDs are accepted.
    let five_hundred: Vec<Uuid> = {
        #[derive(diesel::QueryableByName)]
        struct Row {
            #[diesel(sql_type = SqlUuid)]
            work_id: Uuid,
        }
        let mut connection = fx.pool.get().expect("Failed to get DB connection");
        sql_query(format!(
            "SELECT work_id FROM work WHERE imprint_id = '{imprint}' ORDER BY work_id LIMIT 500"
        ))
        .load::<Row>(&mut connection)
        .expect("work ids")
        .into_iter()
        .map(|row| row.work_id)
        .collect()
    };
    assert_eq!(five_hundred.len(), 500);
    assert!(metric_dashboard(
        &fx.pool,
        &selected(&fx, &[fx.publisher_id], |s| s.work_ids =
            Some(five_hundred.clone()))
    )
    .is_ok());

    add_work(&fx, imprint, "monograph");
    assert_eq!(
        metric_dashboard(&fx.pool, &all),
        Err(MetricReadError::QueryLimitExceeded(TOO_MANY_RESOLVED_WORKS)),
        "2,001 works are refused, never truncated"
    );
    // Narrowing the selection brings it back under the bound.
    let first_imprint = imprint_of(&fx, fx.works[0]);
    assert!(metric_dashboard(
        &fx.pool,
        &selected(&fx, &[fx.publisher_id], |s| s.imprint_ids =
            Some(vec![first_imprint]))
    )
    .is_ok());
}

// ==========================================================================
// MET-WP4-03B: several publishers
// ==========================================================================

/// An eligible DRIVER account for a publisher on a platform.
fn add_account(fx: &Fixture, platform_id: Uuid, publisher_id: Uuid) -> Uuid {
    let source_id = Uuid::new_v4();
    let account_id = Uuid::new_v4();
    exec(
        &fx.pool,
        &format!(
            "{}{}",
            source_sql(source_id, "DRIVER", true),
            account_sql(account_id, source_id, platform_id, Some(publisher_id), true)
        ),
    );
    account_id
}

/// One terminal import on an account asserting `status` for one measure on
/// `[start, end)` with the given dimension flags.
#[allow(clippy::too_many_arguments)]
fn assert_coverage(
    fx: &Fixture,
    account_id: Uuid,
    platform_id: Uuid,
    measure_id: Uuid,
    start: NaiveDate,
    end: NaiveDate,
    status: &str,
    country: bool,
    institution: bool,
) {
    let import_id = Uuid::new_v4();
    exec(
        &fx.pool,
        &format!(
            "{}{}",
            import_sql(import_id, account_id, "COMPLETED", "'2026-03-10T00:00:00Z'"),
            coverage_sql(
                account_id,
                import_id,
                platform_id,
                measure_id,
                start,
                end,
                status,
                country,
                institution
            )
        ),
    );
}

#[test]
fn every_selected_publisher_must_exist_and_be_entitled_and_their_works_combine() {
    let (_guard, fx) = setup();
    let third = Uuid::new_v4();
    let third_work = Uuid::new_v4();
    exec(
        &fx.pool,
        &format!(
            "{}{}",
            publisher_sql(third, "SPHINX"),
            works_sql(third, &[third_work])
        ),
    );
    commit(&fx, fx.works[0], fx.platform_id, fx.sessions, day_n(1), 1);
    commit(
        &fx,
        fx.other_work,
        fx.platform_id,
        fx.sessions,
        day_n(1),
        10,
    );
    commit(&fx, third_work, fx.platform_id, fx.sessions, day_n(1), 100);
    apply_all(&fx.pool);
    let three = [fx.publisher_id, fx.other_publisher_id, third];

    assert_eq!(
        sessions_total(&fx, &selected(&fx, &three[..1], |_| {})),
        some("1")
    );
    assert_eq!(
        sessions_total(&fx, &selected(&fx, &three, |_| {})),
        some("111")
    );
    // The metadata selector applies across every selected publisher.
    let third_imprint = imprint_of(&fx, third_work);
    assert_eq!(
        sessions_total(
            &fx,
            &selected(&fx, &three, |s| s.imprint_ids = Some(vec![third_imprint]))
        ),
        some("100")
    );

    // One unentitled publisher refuses everything, never an entitled subset.
    for package in ["OASIS", "OBELISK"] {
        exec(
            &fx.pool,
            &format!(
                "UPDATE publisher SET subscription_package = '{package}' \
                 WHERE publisher_id = '{third}';"
            ),
        );
        assert_eq!(
            metric_dashboard(&fx.pool, &selected(&fx, &three, |_| {})),
            Err(MetricReadError::Unauthorised),
            "{package}"
        );
        // Even when the selector would exclude its works.
        assert_eq!(
            metric_dashboard(
                &fx.pool,
                &selected(&fx, &three, |s| s.work_ids = Some(vec![fx.works[0]]))
            ),
            Err(MetricReadError::Unauthorised)
        );
    }
    // An unknown publisher among known ones is invalid.
    assert_eq!(
        metric_dashboard(
            &fx.pool,
            &selected(&fx, &[fx.publisher_id, Uuid::new_v4()], |_| {})
        ),
        Err(MetricReadError::QueryInvalid(UNKNOWN_PUBLISHER))
    );
}

#[test]
fn only_represented_publishers_take_part_in_coverage_and_their_assertions_combine() {
    let (_guard, fx) = setup();
    let other_account = add_account(&fx, fx.platform_id, fx.other_publisher_id);
    // The selected publisher: complete, with both dimensions, every day.
    assert_coverage(
        &fx,
        fx.account_id,
        fx.platform_id,
        fx.sessions,
        d1(),
        day_n(5),
        "COMPLETE",
        true,
        true,
    );
    // The other publisher: day 1 complete, day 2 partial, day 3 unasserted,
    // day 4 complete without the country dimension.
    for (day, status, country) in [
        (1, "COMPLETE", true),
        (2, "PARTIAL", true),
        (4, "COMPLETE", false),
    ] {
        assert_coverage(
            &fx,
            other_account,
            fx.platform_id,
            fx.sessions,
            day_n(day),
            day_n(day + 1),
            status,
            country,
            true,
        );
    }
    let both = [fx.publisher_id, fx.other_publisher_id];

    let combined = read(&fx, &totals_only(&selected(&fx, &both, |_| {})));
    assert_eq!(
        bucket_values(&combined, fx.sessions),
        vec![some("0"), None, None, some("0")],
        "UNKNOWN beats PARTIAL beats COMPLETE, per day"
    );
    let combined_item = item(&combined, fx.sessions);
    assert_eq!(combined_item.status, MetricCoverageStatus::Unknown);
    assert_eq!(combined_item.data_through, Some(day_n(1)));
    assert!(
        !combined_item.institution_coverage,
        "day 3 has no assertion"
    );
    // Day 4 alone: both publishers assert COMPLETE, but only one of them
    // includes the country dimension, so the combined flag is false.
    let mut day_four = totals_only(&selected(&fx, &both, |_| {}));
    day_four.start_date = day_n(4);
    let day_four = read(&fx, &day_four);
    let day_four = item(&day_four, fx.sessions);
    assert_eq!(day_four.status, MetricCoverageStatus::Complete);
    assert!(!day_four.country_coverage, "dimension flags combine by AND");
    assert!(day_four.institution_coverage);
    assert_eq!(
        codes(&combined),
        vec![
            MetricWarningCode::UnknownCoverage,
            MetricWarningCode::PartialCoverage
        ]
    );

    // A selected, entitled publisher whose works the selector excludes does
    // not take part at all.
    let only_mine = read(
        &fx,
        &selected(&fx, &both, |s| {
            s.imprint_ids = Some(vec![imprint_of(&fx, fx.works[0])]);
        }),
    );
    assert_eq!(bucket_values(&only_mine, fx.sessions), vec![some("0"); 4]);
    assert_eq!(only_mine.coverage.status, MetricCoverageStatus::Complete);
    assert_eq!(only_mine.data_through, Some(day_n(4)));

    // Nor does its source-account configuration: a second eligible account
    // for the excluded publisher is not ambiguity for this request...
    add_account(&fx, fx.platform_id, fx.other_publisher_id);
    assert!(metric_dashboard(
        &fx.pool,
        &selected(&fx, &both, |s| {
            s.imprint_ids = Some(vec![imprint_of(&fx, fx.works[0])]);
        })
    )
    .is_ok());
    // ...but it is once that publisher is represented.
    assert_eq!(
        metric_dashboard(&fx.pool, &selected(&fx, &both, |_| {})),
        Err(MetricReadError::SourceScopeAmbiguous)
    );
}

// ==========================================================================
// MET-WP4-03B: country and institution sections
// ==========================================================================

/// An institution with a chosen name and optional ROR.
fn named_institution(fx: &Fixture, institution_id: Uuid, name: &str, ror: Option<&str>) {
    exec(
        &fx.pool,
        &format!(
            "INSERT INTO institution (institution_id, institution_name, ror) \
             VALUES ('{institution_id}', '{name}', {});",
            sql_opt(ror)
        ),
    );
}

#[test]
fn countries_and_institutions_are_known_values_in_deterministic_order() {
    let (_guard, fx) = setup();
    let country = |code| Dims {
        country: Some(code),
        ..Dims::default()
    };
    let institution = |id| Dims {
        institution: Some(id),
        ..Dims::default()
    };
    // Ordered by ID, deliberately against the order of their names.
    let (low, high) = {
        let (a, b) = (Uuid::new_v4(), Uuid::new_v4());
        (a.min(b), a.max(b))
    };
    named_institution(&fx, low, "Zeta University", None);
    named_institution(
        &fx,
        high,
        "Alpha Institute",
        Some("https://ror.org/0abcdef12"),
    );
    for (work, measure, day, dims, value) in [
        (fx.works[0], fx.sessions, 1, country("US"), 4),
        (fx.works[0], fx.sessions, 1, country("GB"), 3),
        (fx.works[1], fx.sessions, 1, country("GB"), 5),
        (fx.works[0], fx.units, 1, country("FR"), 7),
        (fx.works[0], fx.sessions, 2, institution(high), 3),
        (fx.works[0], fx.sessions, 2, institution(low), 2),
        (fx.works[1], fx.sessions, 2, institution(low), 5),
        // An undimensioned aggregate is never an "unknown" country or
        // institution row.
        (fx.works[1], fx.units, 3, Dims::default(), 9),
    ] {
        commit_dims(&fx, work, fx.platform_id, measure, day_n(day), dims, value);
    }
    apply_all(&fx.pool);
    let input = window(&fx, &[fx.sessions, fx.units]);
    let dashboard = read(&fx, &input);

    let mut expected_countries = vec![
        (fx.sessions, "GB".to_string(), "8".to_string()),
        (fx.sessions, "US".to_string(), "4".to_string()),
        (fx.units, "FR".to_string(), "7".to_string()),
    ];
    expected_countries.sort_by_key(|(measure, code, _)| (*measure, code.clone()));
    assert_eq!(countries(&dashboard), expected_countries);
    assert!(dashboard
        .countries
        .iter()
        .all(|row| row.platform_id == fx.platform_id));
    assert_eq!(
        institutions(&dashboard),
        vec![
            (fx.sessions, low, "7".to_string()),
            (fx.sessions, high, "3".to_string())
        ],
        "ordered by institution ID, not by name"
    );
    assert_eq!(
        dashboard.institutions[0].institution_name,
        "Zeta University"
    );
    assert_eq!(dashboard.institutions[0].ror, None);
    assert_eq!(
        dashboard.institutions[1]
            .ror
            .as_ref()
            .map(ToString::to_string),
        Some("0abcdef12".to_string())
    );
    // Known values are listed although nothing asserts coverage here.
    assert_eq!(dashboard.coverage.status, MetricCoverageStatus::Unknown);

    // Institution metadata is current metadata.
    exec(
        &fx.pool,
        &format!(
            "UPDATE institution SET institution_name = 'Renamed University' \
             WHERE institution_id = '{low}';"
        ),
    );
    assert_eq!(
        read(&fx, &input).institutions[0].institution_name,
        "Renamed University"
    );

    // Each section can be omitted on its own; totals are unaffected.
    let mut no_countries = input.clone();
    no_countries.include_countries = Some(false);
    let no_countries = read(&fx, &no_countries);
    assert!(no_countries.countries.is_empty());
    assert_eq!(no_countries.institutions.len(), 2);
    let mut no_institutions = input.clone();
    no_institutions.include_institutions = Some(false);
    let no_institutions = read(&fx, &no_institutions);
    assert!(no_institutions.institutions.is_empty());
    assert_eq!(no_institutions.countries.len(), 3);
    assert_eq!(no_institutions.totals, dashboard.totals);
    assert_eq!(no_countries.timeline, dashboard.timeline);
}

/// Insert `count` institutions and one institution-dimensioned projection
/// row for each, for one work, measure and day.
fn institution_rows(fx: &Fixture, work_id: Uuid, day: NaiveDate, count: usize) {
    exec(
        &fx.pool,
        &format!(
            "WITH new AS ( \
                 INSERT INTO institution (institution_name) \
                 SELECT 'Bulk institution ' || n FROM generate_series(1, {count}) n \
                 RETURNING institution_id) \
             INSERT INTO metric_rollup_work_day \
                 (work_id, platform_id, measure_id, day, institution_id, value, watermark) \
             SELECT '{work_id}', '{platform}', '{measure}', '{day}', institution_id, 1, 1 \
             FROM new;",
            platform = fx.platform_id,
            measure = fx.sessions,
        ),
    );
}

/// Advance the frontier past watermark 1 with one real application, so rows
/// written directly with watermark 1 can be rebuilt into the monthly
/// projections by the `MET-WP4-03A` rebuild.
fn advance_frontier(fx: &Fixture) {
    commit(fx, fx.works[1], fx.platform_id, fx.units, d(2020, 1, 1), 1);
    apply_all(&fx.pool);
}

#[test]
fn institution_rows_are_bounded_at_2000_without_truncation() {
    let (_guard, fx) = setup();
    advance_frontier(&fx);
    // Edge days: 2,000 rows are returned in full.
    institution_rows(&fx, fx.works[0], day_n(1), 2000);
    let input = window(&fx, &[fx.sessions]);
    assert_eq!(read(&fx, &input).institutions.len(), 2000);
    institution_rows(&fx, fx.works[1], day_n(2), 1);
    assert_eq!(
        metric_dashboard(&fx.pool, &input),
        Err(MetricReadError::QueryLimitExceeded(TOO_MANY_INSTITUTIONS))
    );
    // Omitting the section serves the rest of the response.
    let mut omitted = input.clone();
    omitted.include_institutions = Some(false);
    assert_eq!(
        total(&read(&fx, &omitted), fx.platform_id, fx.sessions),
        some("2001")
    );

    // A complete month and an edge day together: 1,000 institutions in
    // April from the monthly projection and 1,001 others on 1 May.
    let (april, may) = (d(2026, 4, 1), d(2026, 5, 1));
    institution_rows(&fx, fx.works[0], d(2026, 4, 10), 1000);
    rebuild_month_projections(&fx.pool).expect("rebuild the monthly projections");
    let mut spanning = input.clone();
    spanning.start_date = april;
    spanning.end_date = may;
    assert_eq!(read(&fx, &spanning).institutions.len(), 1000);
    institution_rows(&fx, fx.works[0], may, 1000);
    spanning.end_date = may + Duration::days(1);
    assert_eq!(read(&fx, &spanning).institutions.len(), 2000);
    institution_rows(&fx, fx.works[1], may, 1);
    assert_eq!(
        metric_dashboard(&fx.pool, &spanning),
        Err(MetricReadError::QueryLimitExceeded(TOO_MANY_INSTITUTIONS)),
        "the bound applies to the whole response"
    );
}

/// Commit rows of several masks for one base cell.
fn commit_cell(fx: &Fixture, work: Uuid, day: NaiveDate, rows: &[(Dims, i64)]) {
    for (dims, value) in rows {
        commit_dims(fx, work, fx.platform_id, fx.sessions, day, *dims, *value);
    }
}

#[test]
fn section_ambiguity_fails_only_the_requested_sections_on_months_and_edges() {
    let (_guard, fx) = setup();
    let pdf = insert_publication(&fx.pool, fx.works[0], "PDF");
    let (harvard, oxford) = (insert_institution(&fx.pool), insert_institution(&fx.pool));
    let aggregate = (Dims::default(), 10);
    let country_institution = (
        Dims {
            country: Some("GB"),
            institution: Some(harvard),
            publication: None,
        },
        4,
    );
    let publication_country = (
        Dims {
            publication: Some(pdf),
            country: Some("US"),
            institution: None,
        },
        6,
    );
    let publication_institution = (
        Dims {
            publication: Some(pdf),
            country: None,
            institution: Some(oxford),
        },
        6,
    );
    // Country-ambiguous (masks 3 and 6 are incomparable) with an aggregate.
    let country_ambiguous = [aggregate, country_institution, publication_country];
    // Institution-ambiguous (masks 3 and 5).
    let institution_ambiguous = [aggregate, country_institution, publication_institution];
    // Total-ambiguous: country rows and institution rows, no aggregate.
    let total_ambiguous = [
        (
            Dims {
                country: Some("GB"),
                ..Dims::default()
            },
            1,
        ),
        (
            Dims {
                institution: Some(harvard),
                ..Dims::default()
            },
            1,
        ),
    ];
    let month = |start: NaiveDate| {
        let mut input = window(&fx, &[fx.sessions]);
        input.start_date = start;
        input.end_date = next_month(start);
        input
    };
    let edge = |day: NaiveDate| {
        let mut input = window(&fx, &[fx.sessions]);
        input.start_date = day;
        input.end_date = day + Duration::days(1);
        input
    };
    let with = |input: &MetricDashboardInput, countries: bool, institutions: bool| {
        let mut input = input.clone();
        input.include_countries = Some(countries);
        input.include_institutions = Some(institutions);
        metric_dashboard(&fx.pool, &input).map(|dashboard| text(&dashboard.totals[0].value))
    };

    // April: country ambiguity on a complete month. 1 June: the same on an
    // edge day. May: institution ambiguity on a complete month; 2 June on an
    // edge day. July: total ambiguity on a complete month; 3 June an edge.
    commit_cell(&fx, fx.works[0], d(2026, 4, 10), &country_ambiguous);
    commit_cell(&fx, fx.works[0], d(2026, 6, 1), &country_ambiguous);
    commit_cell(&fx, fx.works[0], d(2026, 5, 10), &institution_ambiguous);
    commit_cell(&fx, fx.works[0], d(2026, 6, 2), &institution_ambiguous);
    commit_cell(&fx, fx.works[0], d(2026, 7, 10), &total_ambiguous);
    commit_cell(&fx, fx.works[0], d(2026, 6, 3), &total_ambiguous);
    apply_all(&fx.pool);

    for (label, input) in [
        ("complete month", month(d(2026, 4, 1))),
        ("edge day", edge(d(2026, 6, 1))),
    ] {
        assert_eq!(
            with(&input, true, true),
            Err(MetricReadError::DimensionScopeAmbiguous),
            "{label}"
        );
        assert_eq!(
            with(&input, true, false),
            Err(MetricReadError::DimensionScopeAmbiguous),
            "{label}"
        );
        assert_eq!(with(&input, false, true), Ok(some("10")), "{label}");
        assert_eq!(with(&input, false, false), Ok(some("10")), "{label}");
    }
    for (label, input) in [
        ("complete month", month(d(2026, 5, 1))),
        ("edge day", edge(d(2026, 6, 2))),
    ] {
        assert_eq!(
            with(&input, true, true),
            Err(MetricReadError::DimensionScopeAmbiguous),
            "{label}"
        );
        assert_eq!(
            with(&input, false, true),
            Err(MetricReadError::DimensionScopeAmbiguous),
            "{label}"
        );
        assert_eq!(with(&input, true, false), Ok(some("10")), "{label}");
        assert_eq!(with(&input, false, false), Ok(some("10")), "{label}");
    }
    for (label, input) in [
        ("complete month", month(d(2026, 7, 1))),
        ("edge day", edge(d(2026, 6, 3))),
    ] {
        for (countries, institutions) in [(true, true), (false, false), (true, false)] {
            assert_eq!(
                with(&input, countries, institutions),
                Err(MetricReadError::DimensionScopeAmbiguous),
                "{label}: total ambiguity fails whatever sections are returned"
            );
        }
    }
    // The monthly timeline and a daily timeline agree about months that are
    // not ambiguous.
    let mut april_daily = month(d(2026, 4, 1));
    april_daily.include_countries = Some(false);
    april_daily.timeline_grain = Some(MetricTimelineGrain::Day);
    assert!(metric_dashboard(&fx.pool, &april_daily).is_ok());
}

#[test]
fn each_section_needs_its_own_dimensions_range_wide() {
    let (_guard, fx) = setup();
    let harvard = insert_institution(&fx.pool);
    let aggregate = (Dims::default(), 10);
    let country_only = (
        Dims {
            country: Some("GB"),
            ..Dims::default()
        },
        6,
    );
    let institution_only = (
        Dims {
            institution: Some(harvard),
            ..Dims::default()
        },
        6,
    );
    let both = (
        Dims {
            country: Some("GB"),
            institution: Some(harvard),
            publication: None,
        },
        4,
    );
    let request_for = |measure: Uuid, countries: bool, institutions: bool| {
        let mut input = window(&fx, &[measure]);
        input.include_countries = Some(countries);
        input.include_institutions = Some(institutions);
        input
    };
    // Sessions: country values come from {country, institution} rows
    // (#946 Amendment 1 section 2.6 item 4); units: institution values
    // come from them (item 5). An aggregate keeps every total independent.
    commit_cell(
        &fx,
        fx.works[0],
        day_n(1),
        &[aggregate, both, institution_only],
    );
    for (dims, value) in [aggregate, both, country_only] {
        commit_dims(
            &fx,
            fx.works[0],
            fx.platform_id,
            fx.units,
            day_n(1),
            dims,
            value,
        );
    }
    apply_all(&fx.pool);
    // Sessions: country asserted every day, institution missing on day 3.
    // Units: institution asserted every day, country missing on day 3.
    for (measure, country_on_3, institution_on_3) in
        [(fx.sessions, true, false), (fx.units, false, true)]
    {
        assert_coverage(
            &fx,
            fx.account_id,
            fx.platform_id,
            measure,
            d1(),
            day_n(5),
            "COMPLETE",
            true,
            true,
        );
        let import_id = Uuid::new_v4();
        exec(
            &fx.pool,
            &format!(
                "{}{}",
                import_sql(
                    import_id,
                    fx.account_id,
                    "COMPLETED",
                    "'2026-03-11T00:00:00Z'"
                ),
                coverage_sql(
                    fx.account_id,
                    import_id,
                    fx.platform_id,
                    measure,
                    day_n(3),
                    day_n(4),
                    "COMPLETE",
                    country_on_3,
                    institution_on_3
                )
            ),
        );
    }
    let status = |input: &MetricDashboardInput| read(&fx, input).coverage.status;

    // Totals alone depend on neither dimension: complete, with real zeros.
    for measure in [fx.sessions, fx.units] {
        let totals = read(&fx, &request_for(measure, false, false));
        assert_eq!(totals.coverage.status, MetricCoverageStatus::Complete);
        assert_eq!(
            bucket_values(&totals, measure),
            vec![some("10"), some("0"), some("0"), some("0")]
        );
    }
    // Item 4: the sessions country section needs institution coverage too,
    // because its country values carry an institution.
    assert_eq!(
        status(&request_for(fx.sessions, true, false)),
        MetricCoverageStatus::Partial
    );
    // Item 5: the units institution section needs country coverage too.
    assert_eq!(
        status(&request_for(fx.units, false, true)),
        MetricCoverageStatus::Partial
    );
    // A country section over plain country rows needs only country
    // coverage, which units lacks on day 3 while sessions has it; the
    // mirror holds for institutions.
    assert_eq!(
        status(&request_for(fx.units, true, false)),
        MetricCoverageStatus::Partial
    );
    assert_eq!(
        status(&request_for(fx.sessions, false, true)),
        MetricCoverageStatus::Partial
    );
    // With every value the same whatever the sections, and known values
    // listed under PARTIAL coverage (item 6).
    let all = read(&fx, &request_for(fx.sessions, true, true));
    assert_eq!(all.coverage.status, MetricCoverageStatus::Partial);
    assert_eq!(
        all.timeline,
        read(&fx, &request_for(fx.sessions, false, false)).timeline
    );
    assert_eq!(
        countries(&all),
        vec![(fx.sessions, "GB".to_string(), "4".to_string())]
    );
    assert_eq!(
        institutions(&all),
        vec![(fx.sessions, harvard, "6".to_string())]
    );
    // dataThrough follows the shared completeness.
    assert_eq!(all.data_through, Some(day_n(2)));
}

// ==========================================================================
// MET-WP4-03B: complete months and edge days
// ==========================================================================

#[test]
fn complete_months_are_served_from_the_monthly_projection_and_edges_from_days() {
    let (_guard, fx) = setup();
    for (day, value) in [
        (d(2026, 1, 31), 3),
        (d(2026, 2, 15), 4),
        (d(2026, 2, 28), 5),
        (d(2026, 3, 1), 6),
    ] {
        commit(&fx, fx.works[0], fx.platform_id, fx.sessions, day, value);
    }
    // A publication-specific and an undimensioned row of the same complete
    // month, on different days, are both part of it.
    let pdf = insert_publication(&fx.pool, fx.works[1], "PDF");
    commit_dims(
        &fx,
        fx.works[1],
        fx.platform_id,
        fx.sessions,
        d(2026, 2, 2),
        Dims {
            publication: Some(pdf),
            ..Dims::default()
        },
        20,
    );
    commit(
        &fx,
        fx.works[1],
        fx.platform_id,
        fx.sessions,
        d(2026, 2, 3),
        30,
    );
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
    let at = |grain| {
        read(
            &fx,
            &request(
                fx.publisher_id,
                start,
                end,
                &[fx.platform_id],
                &[fx.sessions],
                Some(grain),
            ),
        )
    };
    let month = at(MetricTimelineGrain::Month);
    assert_eq!(
        bucket_values(&month, fx.sessions),
        vec![some("3"), some("59"), some("6")]
    );
    assert_eq!(total(&month, fx.platform_id, fx.sessions), some("68"));
    let day = at(MetricTimelineGrain::Day);
    assert_eq!(total(&day, fx.platform_id, fx.sessions), some("68"));

    // Changing only February's monthly row changes the February bucket and
    // every total, and leaves the daily buckets as they are: complete
    // months are read from the monthly projection.
    exec(
        &fx.pool,
        &format!(
            "UPDATE metric_rollup_work_month SET value = value + 100 \
             WHERE month_start = '2026-02-01' AND work_id = '{}';",
            fx.works[0]
        ),
    );
    let month = at(MetricTimelineGrain::Month);
    assert_eq!(
        bucket_values(&month, fx.sessions),
        vec![some("3"), some("159"), some("6")]
    );
    assert_eq!(total(&month, fx.platform_id, fx.sessions), some("168"));
    let tampered_day = at(MetricTimelineGrain::Day);
    assert_eq!(tampered_day.timeline, day.timeline);
    assert_eq!(
        total(&tampered_day, fx.platform_id, fx.sessions),
        some("168")
    );

    // Changing a day row inside February changes only the daily bucket;
    // changing an edge day changes its clipped bucket and the totals.
    exec(
        &fx.pool,
        "UPDATE metric_rollup_work_day SET value = value + 1000 WHERE day = '2026-02-15';",
    );
    let month = at(MetricTimelineGrain::Month);
    assert_eq!(
        bucket_values(&month, fx.sessions),
        vec![some("3"), some("159"), some("6")]
    );
    exec(
        &fx.pool,
        "UPDATE metric_rollup_work_day SET value = value + 10000 WHERE day = '2026-01-31';",
    );
    let month = at(MetricTimelineGrain::Month);
    assert_eq!(
        bucket_values(&month, fx.sessions),
        vec![some("10003"), some("159"), some("6")]
    );
    assert_eq!(total(&month, fx.platform_id, fx.sessions), some("10168"));

    // The reviewed rebuild restores the monthly rows from the day rows, and
    // then both paths agree again.
    rebuild_month_projections(&fx.pool).expect("rebuild the monthly projections");
    let month = at(MetricTimelineGrain::Month);
    let day = at(MetricTimelineGrain::Day);
    assert_eq!(
        bucket_values(&month, fx.sessions),
        vec![some("10003"), some("1059"), some("6")]
    );
    assert_eq!(total(&month, fx.platform_id, fx.sessions), some("11068"));
    let day_sum: i128 = day
        .timeline
        .iter()
        .map(|bucket| bucket.value.expect("a covered day").value())
        .sum();
    assert_eq!(
        day_sum, 11068,
        "the daily buckets add up to the monthly total"
    );
    assert_eq!(month.totals, day.totals);
}

/// A tiny deterministic generator (xorshift64*).
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, bound: u64) -> u64 {
        self.next() % bound
    }
}

/// One generated work-day row.
#[derive(Clone, Copy)]
struct Generated {
    work: Uuid,
    measure: Uuid,
    day: NaiveDate,
    dims: Dims,
    value: i64,
}

fn mask_of(dims: &Dims) -> u8 {
    u8::from(dims.publication.is_some()) * 4
        + u8::from(dims.country.is_some()) * 2
        + u8::from(dims.institution.is_some())
}

/// The unique least mask among `masks` containing `bit`, under set
/// inclusion; `None` when no mask contains it.
fn least_with(masks: &BTreeSet<u8>, bit: u8) -> Option<u8> {
    let candidates: Vec<u8> = masks
        .iter()
        .copied()
        .filter(|mask| mask & bit != 0)
        .collect();
    let minimal: Vec<u8> = candidates
        .iter()
        .copied()
        .filter(|mask| {
            !candidates
                .iter()
                .any(|other| other != mask && other & mask == *other)
        })
        .collect();
    assert!(
        minimal.len() <= 1,
        "the generator never makes a section ambiguous"
    );
    minimal.first().copied()
}

#[test]
fn monthly_and_edge_serving_equals_an_independent_day_by_day_oracle() {
    let (_guard, fx) = setup();
    let third = Uuid::new_v4();
    exec(&fx.pool, &works_sql(fx.publisher_id, &[third]));
    let works = [fx.works[0], fx.works[1], third];
    let publications: Vec<[Uuid; 2]> = works
        .iter()
        .map(|work| {
            [
                insert_publication(&fx.pool, *work, "PDF"),
                insert_publication(&fx.pool, *work, "Epub"),
            ]
        })
        .collect();
    let institution_pool = [
        insert_institution(&fx.pool),
        insert_institution(&fx.pool),
        insert_institution(&fx.pool),
    ];
    // Every mask set whose total and sections resolve unambiguously.
    let patterns: [&[u8]; 17] = [
        &[0],
        &[0, 2],
        &[2],
        &[1],
        &[3],
        &[0, 2, 1],
        &[0, 3],
        &[4],
        &[0, 6],
        &[0, 2, 3],
        &[0, 1, 3],
        &[6],
        &[0, 7],
        &[5],
        &[0, 4],
        &[0, 2, 6],
        &[0, 1, 5],
    ];
    let (start, end) = (d(2026, 1, 20), d(2026, 5, 10));
    let mut rng = Rng(20_260_923);
    let mut rows: Vec<Generated> = Vec::new();
    for (work_index, work) in works.iter().enumerate() {
        for measure in [fx.sessions, fx.units] {
            for day in days_of(start, end) {
                if rng.below(10) >= 4 {
                    continue;
                }
                let pattern = patterns[rng.below(patterns.len() as u64) as usize];
                for mask in pattern {
                    let mut seen: BTreeSet<(Option<Uuid>, Option<&str>, Option<Uuid>)> =
                        BTreeSet::new();
                    let copies = if *mask == 0 { 1 } else { 1 + rng.below(2) };
                    for _ in 0..copies {
                        let dims = Dims {
                            publication: (mask & 4 != 0)
                                .then(|| publications[work_index][rng.below(2) as usize]),
                            country: (mask & 2 != 0)
                                .then(|| ["GB", "US", "DE"][rng.below(3) as usize]),
                            institution: (mask & 1 != 0)
                                .then(|| institution_pool[rng.below(3) as usize]),
                        };
                        if seen.insert((dims.publication, dims.country, dims.institution)) {
                            rows.push(Generated {
                                work: *work,
                                measure,
                                day,
                                dims,
                                value: 1 + rng.below(20) as i64,
                            });
                        }
                    }
                }
            }
        }
    }
    let mut statements = String::new();
    for row in &rows {
        statements.push_str(&record_dims_sql(
            &fx,
            Uuid::new_v4(),
            row.work,
            fx.platform_id,
            row.measure,
            row.day,
            row.dims,
            row.value,
        ));
    }
    exec(&fx.pool, &statements);
    apply_all(&fx.pool);
    for measure in [fx.sessions, fx.units] {
        cover(
            &fx,
            measure,
            start,
            end,
            "COMPLETE",
            "COMPLETED",
            "'2026-03-10T00:00:00Z'",
        );
    }

    // The oracle: resolve every base cell from the generated rows alone.
    let mut cells: BTreeMap<(Uuid, Uuid, NaiveDate), Vec<Generated>> = BTreeMap::new();
    for row in &rows {
        cells
            .entry((row.work, row.measure, row.day))
            .or_default()
            .push(*row);
    }
    let mut per_day: BTreeMap<(Uuid, NaiveDate), i128> = BTreeMap::new();
    let mut per_country: BTreeMap<(Uuid, String), i128> = BTreeMap::new();
    let mut per_institution: BTreeMap<(Uuid, Uuid), i128> = BTreeMap::new();
    for ((_, measure, day), cell) in &cells {
        let masks: BTreeSet<u8> = cell.iter().map(|row| mask_of(&row.dims)).collect();
        let total_mask = if masks.contains(&0) {
            0
        } else {
            assert_eq!(
                masks.len(),
                1,
                "the generator never makes a total ambiguous"
            );
            *masks.iter().next().expect("a mask")
        };
        let sum = |mask: u8| -> i128 {
            cell.iter()
                .filter(|row| mask_of(&row.dims) == mask)
                .map(|row| i128::from(row.value))
                .sum()
        };
        *per_day.entry((*measure, *day)).or_default() += sum(total_mask);
        if let Some(mask) = least_with(&masks, 2) {
            for row in cell.iter().filter(|row| mask_of(&row.dims) == mask) {
                *per_country
                    .entry((*measure, row.dims.country.expect("a country").to_string()))
                    .or_default() += i128::from(row.value);
            }
        }
        if let Some(mask) = least_with(&masks, 1) {
            for row in cell.iter().filter(|row| mask_of(&row.dims) == mask) {
                *per_institution
                    .entry((*measure, row.dims.institution.expect("an institution")))
                    .or_default() += i128::from(row.value);
            }
        }
    }
    let day_value = |measure: Uuid, from: NaiveDate, to: NaiveDate| -> String {
        days_of(from, to)
            .into_iter()
            .map(|day| per_day.get(&(measure, day)).copied().unwrap_or(0))
            .sum::<i128>()
            .to_string()
    };

    let for_grain = |grain| {
        request(
            fx.publisher_id,
            start,
            end,
            &[fx.platform_id],
            &[fx.sessions, fx.units],
            Some(grain),
        )
    };
    let monthly = read(&fx, &for_grain(MetricTimelineGrain::Month));
    let daily = read(&fx, &for_grain(MetricTimelineGrain::Day));
    assert_eq!(monthly.coverage.status, MetricCoverageStatus::Complete);
    for measure in [fx.sessions, fx.units] {
        let expected_total = Some(day_value(measure, start, end));
        assert_eq!(total(&monthly, fx.platform_id, measure), expected_total);
        assert_eq!(total(&daily, fx.platform_id, measure), expected_total);
        let month_buckets: Vec<Option<String>> = buckets_of(start, end, MetricTimelineGrain::Month)
            .into_iter()
            .map(|(from, to)| Some(day_value(measure, from, to)))
            .collect();
        assert_eq!(bucket_values(&monthly, measure), month_buckets);
        let day_buckets: Vec<Option<String>> = days_of(start, end)
            .into_iter()
            .map(|day| Some(day_value(measure, day, day + Duration::days(1))))
            .collect();
        assert_eq!(bucket_values(&daily, measure), day_buckets);
    }
    let mut expected_countries: Vec<(Uuid, String, String)> = per_country
        .iter()
        .map(|((measure, code), value)| (*measure, code.clone(), value.to_string()))
        .collect();
    expected_countries.sort_by_key(|(measure, code, _)| (*measure, code.clone()));
    let mut expected_institutions: Vec<(Uuid, Uuid, String)> = per_institution
        .iter()
        .map(|((measure, institution), value)| (*measure, *institution, value.to_string()))
        .collect();
    expected_institutions.sort_by_key(|(measure, institution, _)| (*measure, *institution));
    assert!(!expected_countries.is_empty() && !expected_institutions.is_empty());
    for dashboard in [&monthly, &daily] {
        assert_eq!(countries(dashboard), expected_countries);
        assert_eq!(institutions(dashboard), expected_institutions);
    }
    println!(
        "oracle: {} generated rows in {} base cells",
        rows.len(),
        cells.len()
    );

    // The reviewed rebuild reproduces the incrementally maintained months,
    // so a rebuilt projection serves exactly the same response.
    rebuild_month_projections(&fx.pool).expect("rebuild the monthly projections");
    let rebuilt = read(&fx, &for_grain(MetricTimelineGrain::Month));
    assert_eq!(rebuilt.totals, monthly.totals);
    assert_eq!(rebuilt.timeline, monthly.timeline);
    assert_eq!(rebuilt.countries, monthly.countries);
    assert_eq!(rebuilt.institutions, monthly.institutions);
}

// ==========================================================================
// MET-WP4-03B: native source grains
// ==========================================================================

/// How a native-grain record's current revision pointer is left.
#[derive(Clone, Copy, Debug)]
enum Native {
    /// The pointer names the record's CURRENT revision.
    Active,
    /// The pointer names a RETRACTED revision that superseded a CURRENT one.
    Retracted,
    /// Contradictory: the pointer names a SUPERSEDED revision.
    PointsAtSuperseded,
    /// Contradictory: a revision exists but no pointer does.
    NoPointer,
    /// Contradictory: the pointer names a RETRACTED revision while another
    /// revision of the record is still CURRENT.
    RetractedBesideCurrent,
}

/// One canonical record of a native grain with the given pointer state.
#[allow(clippy::too_many_arguments)]
fn native_record(
    fx: &Fixture,
    work_id: Uuid,
    platform_id: Uuid,
    measure_id: Uuid,
    grain: &str,
    start: NaiveDate,
    end: NaiveDate,
    state: Native,
) {
    let record_id = Uuid::new_v4();
    let (first, second) = (Uuid::new_v4(), Uuid::new_v4());
    let revision = |id: Uuid, number: i32, status: &str| {
        format!(
            "INSERT INTO metric_record_revision \
                 (record_revision_id, record_id, revision_number, import_id, value, \
                  content_hash, status) \
             VALUES ('{id}', '{record_id}', {number}, '{import}', 50, 'content-{id}', \
                     '{status}');",
            import = fx.canonical_import,
        )
    };
    let (revisions, pointer) = match state {
        Native::Active => (revision(first, 1, "CURRENT"), Some(first)),
        Native::Retracted => (
            format!(
                "{}{}",
                revision(first, 1, "SUPERSEDED"),
                revision(second, 2, "RETRACTED")
            ),
            Some(second),
        ),
        Native::PointsAtSuperseded => (revision(first, 1, "SUPERSEDED"), Some(first)),
        Native::NoPointer => (revision(first, 1, "CURRENT"), None),
        Native::RetractedBesideCurrent => (
            format!(
                "{}{}",
                revision(first, 1, "CURRENT"),
                revision(second, 2, "RETRACTED")
            ),
            Some(second),
        ),
    };
    exec(
        &fx.pool,
        &format!(
            "INSERT INTO metric_record \
                 (record_id, identity_hash, work_id, platform_id, measure_id, period_start, \
                  period_end, reporting_grain, winning_source_account_id) \
             VALUES ('{record_id}', 'native-{record_id}', '{work_id}', '{platform_id}', \
                     '{measure_id}', '{start}', '{end}', '{grain}', '{account}'); \
             {revisions} \
             UPDATE metric_record SET current_revision_id = {pointer} \
              WHERE record_id = '{record_id}';",
            account = fx.account_id,
            pointer = sql_opt(pointer),
        ),
    );
}

#[test]
fn active_native_month_and_reporting_period_records_fail_and_retracted_ones_do_not() {
    let (_guard, fx) = setup();
    let imprint = imprint_of(&fx, fx.works[0]);
    let only = |work_id: Uuid| {
        let mut input = window(&fx, &[fx.sessions]);
        input.selector.work_ids = Some(vec![work_id]);
        input
    };
    // The records overlap the half-open window [1 March, 5 March).
    let (start, end) = (d(2026, 2, 1), d(2026, 3, 2));
    for grain in ["MONTH", "REPORTING_PERIOD"] {
        let active = add_work(&fx, imprint, "monograph");
        native_record(
            &fx,
            active,
            fx.platform_id,
            fx.sessions,
            grain,
            start,
            end,
            Native::Active,
        );
        assert_eq!(
            metric_dashboard(&fx.pool, &only(active)),
            Err(MetricReadError::UnsupportedSourceGrain),
            "{grain}"
        );
        assert_eq!(
            MetricReadError::UnsupportedSourceGrain.code(),
            "METRIC_QUERY_UNSUPPORTED_SOURCE_GRAIN"
        );
        let retracted = add_work(&fx, imprint, "monograph");
        native_record(
            &fx,
            retracted,
            fx.platform_id,
            fx.sessions,
            grain,
            start,
            end,
            Native::Retracted,
        );
        assert!(
            metric_dashboard(&fx.pool, &only(retracted)).is_ok(),
            "a retracted {grain} record does not poison the dashboard"
        );
        // Outside the request in each dimension, an active record is
        // irrelevant: another work, the day before the window, the window's
        // end, another platform and another measure.
        let elsewhere = add_work(&fx, imprint, "monograph");
        native_record(
            &fx,
            elsewhere,
            fx.platform_id,
            fx.sessions,
            grain,
            d(2026, 2, 1),
            d1(),
            Native::Active,
        );
        native_record(
            &fx,
            elsewhere,
            fx.platform_id,
            fx.sessions,
            grain,
            day_n(5),
            day_n(40),
            Native::Active,
        );
        native_record(
            &fx,
            elsewhere,
            fx.other_platform_id,
            fx.sessions,
            grain,
            start,
            end,
            Native::Active,
        );
        native_record(
            &fx,
            elsewhere,
            fx.platform_id,
            fx.units,
            grain,
            start,
            end,
            Native::Active,
        );
        assert!(
            metric_dashboard(&fx.pool, &only(elsewhere)).is_ok(),
            "{grain} outside the date, platform and measure scope"
        );
        assert!(
            metric_dashboard(&fx.pool, &only(retracted)).is_ok(),
            "{grain} on another work"
        );
        // A daily window reaching the active record's platform or measure
        // does fail.
        let mut units = only(elsewhere);
        units.measures = Some(vec![fx.units]);
        assert_eq!(
            metric_dashboard(&fx.pool, &units),
            Err(MetricReadError::UnsupportedSourceGrain)
        );
    }
    // The whole publisher now intersects active records: a MONTH request
    // fails too, whatever the timeline grain.
    let mut whole = window(&fx, &[fx.sessions]);
    whole.timeline_grain = Some(MetricTimelineGrain::Month);
    assert_eq!(
        metric_dashboard(&fx.pool, &whole),
        Err(MetricReadError::UnsupportedSourceGrain)
    );

    // Contradictory committed canonical state fails closed as an internal
    // error rather than being classified either way.
    for state in [
        Native::PointsAtSuperseded,
        Native::NoPointer,
        Native::RetractedBesideCurrent,
    ] {
        let work = add_work(&fx, imprint, "monograph");
        native_record(
            &fx,
            work,
            fx.platform_id,
            fx.sessions,
            "MONTH",
            start,
            end,
            state,
        );
        assert_eq!(
            metric_dashboard(&fx.pool, &only(work)),
            Err(MetricReadError::Unavailable),
            "{state:?}"
        );
    }
    // Daily records are served, never refused as a native grain.
    let daily = add_work(&fx, imprint, "monograph");
    commit(&fx, daily, fx.platform_id, fx.sessions, day_n(2), 3);
    apply_all(&fx.pool);
    assert_eq!(sessions_total(&fx, &only(daily)), some("3"));
}

// ==========================================================================
// MET-WP4-03B Amendment 2: identifier quality under the selector
// ==========================================================================

/// A platform and a measure that nothing but quarantine evidence represents.
fn quarantine_only_pair(fx: &Fixture, code: &str) -> (Uuid, Uuid) {
    let platform_id = Uuid::new_v4();
    insert_platform_row(&fx.pool, platform_id, &format!("{code}_platform"));
    (
        platform_id,
        insert_measure(&fx.pool, &format!("{code}_measure"), true, true),
    )
}

/// Selected publishers `[selected publisher, other publisher, third]`, where
/// the third publisher owns one work in its own imprint.
fn with_third_publisher(fx: &Fixture) -> (Uuid, Uuid) {
    let third = Uuid::new_v4();
    let third_work = Uuid::new_v4();
    exec(
        &fx.pool,
        &format!(
            "{}{}",
            publisher_sql(third, "SPHINX"),
            works_sql(third, &[third_work])
        ),
    );
    (third, third_work)
}

#[test]
fn a_selected_publisher_without_resolved_works_contributes_no_identifier_quality() {
    let (_guard, fx) = setup();
    let (third, _) = with_third_publisher(&fx);
    commit(&fx, fx.works[0], fx.platform_id, fx.sessions, day_n(1), 5);
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
    let (other_platform, other_measure) = quarantine_only_pair(&fx, "unrepresented");
    // Unresolved evidence admitted under the other two publishers' imports:
    // one row on the served pair, one on a pair nothing else represents.
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
        fx.other_publisher_id,
        other_platform,
        other_measure,
        day_n(2),
        day_n(3),
        "DAY",
    );
    quarantine(
        &fx,
        third,
        fx.platform_id,
        fx.sessions,
        day_n(3),
        day_n(4),
        "REPORTING_PERIOD",
    );
    let three = [fx.publisher_id, fx.other_publisher_id, third];
    let mine_only = |s: &mut MetricSelectorInput| {
        s.imprint_ids = Some(vec![imprint_of(&fx, fx.works[0])]);
    };

    // All three are selected and entitled, but only the first owns a
    // resolved work: the others' quarantine neither warns nor blocks zeros.
    let explicit = read(&fx, &selected(&fx, &three, mine_only));
    assert_eq!(
        bucket_values(&explicit, fx.sessions),
        vec![some("5"), some("0"), some("0"), some("0")]
    );
    assert!(explicit.warnings.is_empty(), "{:?}", codes(&explicit));
    assert!(!explicit.is_partial);
    // Nor does it add the pair only it represents to an omitted scope.
    let mut omitted = selected(&fx, &three, mine_only);
    omitted.platforms = None;
    omitted.measures = None;
    let omitted = read(&fx, &omitted);
    assert_serves(&omitted, &[fx.platform_id], &[fx.sessions]);
    assert!(omitted.warnings.is_empty(), "{:?}", codes(&omitted));
    // Two selected publishers, the second unrepresented, behave the same.
    let two = read(
        &fx,
        &selected(&fx, &three[..2], |s| {
            s.work_ids = Some(vec![fx.works[0]]);
        }),
    );
    assert!(two.warnings.is_empty(), "{:?}", codes(&two));

    // Control: once their works are resolved, the same evidence counts.
    let mut all = selected(&fx, &three, |_| {});
    all.platforms = None;
    all.measures = None;
    let all = read(&fx, &all);
    assert_serves(
        &all,
        &[fx.platform_id, other_platform],
        &[fx.sessions, other_measure],
    );
    assert!(codes(&all).contains(&MetricWarningCode::UnresolvedIdentifiers));
}

#[test]
fn represented_quarantine_warns_under_a_narrowed_selection_without_a_work_identity() {
    let (_guard, fx) = setup();
    // The only value belongs to a work the selector excludes.
    commit(&fx, fx.works[0], fx.platform_id, fx.sessions, day_n(1), 5);
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
    let quarantine_id = quarantine(
        &fx,
        fx.publisher_id,
        fx.platform_id,
        fx.sessions,
        day_n(2),
        day_n(3),
        "DAY",
    );
    let series = add_series(&fx, imprint_of(&fx, fx.works[1]));
    add_issue(&fx, series, fx.works[1], 1);
    for edit in [
        Box::new(|s: &mut MetricSelectorInput| s.work_ids = Some(vec![fx.works[1]]))
            as Box<dyn Fn(&mut MetricSelectorInput)>,
        Box::new(|s: &mut MetricSelectorInput| s.series_ids = Some(vec![series])),
    ] {
        let dashboard = read(&fx, &selected(&fx, &[fx.publisher_id], |s| edit(s)));
        // The excluded work's value is gone, the unresolved day is not a
        // zero, and the evidence was never matched to the remaining work.
        assert_eq!(
            bucket_values(&dashboard, fx.sessions),
            vec![some("0"), None, some("0"), some("0")]
        );
        assert_eq!(total(&dashboard, fx.platform_id, fx.sessions), None);
        assert_eq!(
            codes(&dashboard),
            vec![MetricWarningCode::UnresolvedIdentifiers]
        );
        assert_eq!(dashboard.coverage.status, MetricCoverageStatus::Complete);
        assert_eq!(dashboard.data_through, Some(day_n(4)));
    }
    // Reading reinterpreted nothing: the quarantine row still carries its
    // unresolved DOI and has no reconciliation row.
    assert_eq!(
        scalar_i64(
            &fx.pool,
            &format!(
                "(SELECT COUNT(*) FROM metric_identifier_quarantine q \
                  WHERE q.identifier_quarantine_id = '{quarantine_id}' \
                    AND q.work_doi LIKE 'https://doi.org/10.12345/dashboard-%' \
                    AND NOT EXISTS (SELECT 1 FROM metric_identifier_quarantine_reconciliation r \
                                    WHERE r.identifier_quarantine_id = q.identifier_quarantine_id))"
            )
        ),
        1
    );
}

#[test]
fn quarantine_never_repopulates_a_selection_without_works() {
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
    let (quarantine_platform, quarantine_measure) = quarantine_only_pair(&fx, "zero_work");
    quarantine(
        &fx,
        fx.publisher_id,
        fx.platform_id,
        fx.sessions,
        day_n(2),
        day_n(3),
        "DAY",
    );
    quarantine(
        &fx,
        fx.publisher_id,
        quarantine_platform,
        quarantine_measure,
        day_n(2),
        day_n(3),
        "DAY",
    );
    let nothing = |s: &mut MetricSelectorInput| s.work_types = Some(vec![WorkType::JournalIssue]);

    // Omitted dimensions: the frozen empty shape, with no quarantine-derived
    // pair and no identifier warning.
    let mut omitted = selected(&fx, &[fx.publisher_id], nothing);
    omitted.platforms = None;
    omitted.measures = None;
    let empty = read(&fx, &omitted);
    assert!(empty.totals.is_empty() && empty.coverage.items.is_empty());
    assert_eq!(codes(&empty), vec![MetricWarningCode::UnknownCoverage]);
    // One omitted dimension: quarantine does not resolve it either.
    let mut platforms_omitted = selected(&fx, &[fx.publisher_id], nothing);
    platforms_omitted.platforms = None;
    assert!(read(&fx, &platforms_omitted).totals.is_empty());
    // Explicit dimensions: null and UNKNOWN, never a warning from evidence
    // of a publisher that is not represented.
    let explicit = read(&fx, &selected(&fx, &[fx.publisher_id], nothing));
    assert_eq!(bucket_values(&explicit, fx.sessions), vec![None; 4]);
    assert_eq!(codes(&explicit), vec![MetricWarningCode::UnknownCoverage]);

    // Control: the same publisher with its works resolved.
    let control = read(&fx, &omitted_scope(fx.publisher_id, d1(), day_n(5)));
    assert_serves(
        &control,
        &[fx.platform_id, quarantine_platform],
        &[fx.sessions, quarantine_measure],
    );
    assert!(codes(&control).contains(&MetricWarningCode::UnresolvedIdentifiers));
}

#[test]
fn quarantine_only_pairs_of_represented_publishers_resolve_omitted_dimensions_within_filters() {
    let (_guard, fx) = setup();
    let (mine_platform, mine_measure) = quarantine_only_pair(&fx, "represented_mine");
    let (theirs_platform, theirs_measure) = quarantine_only_pair(&fx, "represented_theirs");
    quarantine(
        &fx,
        fx.publisher_id,
        mine_platform,
        mine_measure,
        day_n(2),
        day_n(3),
        "DAY",
    );
    quarantine(
        &fx,
        fx.other_publisher_id,
        theirs_platform,
        theirs_measure,
        day_n(3),
        day_n(4),
        "DAY",
    );
    let both = [fx.publisher_id, fx.other_publisher_id];
    let omitted = |edit: &dyn Fn(&mut MetricDashboardInput)| {
        let mut input = selected(&fx, &both, |_| {});
        input.platforms = None;
        input.measures = None;
        edit(&mut input);
        read(&fx, &input)
    };

    // Both publishers are represented, and each one's quarantine-only pair
    // is discovered.
    let discovered = omitted(&|_| {});
    assert_serves(
        &discovered,
        &[mine_platform, theirs_platform],
        &[mine_measure, theirs_measure],
    );
    assert_eq!(
        codes(&discovered),
        vec![
            MetricWarningCode::UnknownCoverage,
            MetricWarningCode::UnresolvedIdentifiers,
        ]
    );
    assert!(discovered.totals.iter().all(|total| total.value.is_none()));
    // An explicit platform constrains the discovered measures, and an
    // explicit measure the discovered platforms.
    let platform_explicit = omitted(&|input| input.platforms = Some(vec![mine_platform]));
    assert_serves(&platform_explicit, &[mine_platform], &[mine_measure]);
    let measure_explicit = omitted(&|input| input.measures = Some(vec![theirs_measure]));
    assert_serves(&measure_explicit, &[theirs_platform], &[theirs_measure]);
    // A selector leaving the other publisher without works drops its pair.
    let narrowed = omitted(&|input| {
        input.selector.imprint_ids = Some(vec![imprint_of(&fx, fx.works[0])]);
    });
    assert_serves(&narrowed, &[mine_platform], &[mine_measure]);
}

#[test]
fn identifier_quality_masks_complete_months_and_edges_alike_and_keeps_projected_values() {
    let (_guard, fx) = setup();
    // An April value served from the monthly projection.
    commit(
        &fx,
        fx.works[0],
        fx.platform_id,
        fx.sessions,
        d(2026, 4, 10),
        7,
    );
    apply_all(&fx.pool);
    let (start, end) = (d(2026, 3, 20), d(2026, 5, 10));
    cover(
        &fx,
        fx.sessions,
        start,
        end,
        "COMPLETE",
        "COMPLETED",
        "'2026-05-20T00:00:00Z'",
    );
    let at = |grain| {
        read(
            &fx,
            &request(
                fx.publisher_id,
                start,
                end,
                &[fx.platform_id],
                &[fx.sessions],
                Some(grain),
            ),
        )
    };
    let clean = at(MetricTimelineGrain::Month);
    assert_eq!(
        bucket_values(&clean, fx.sessions),
        vec![some("0"), some("7"), some("0")]
    );
    assert!(clean.warnings.is_empty());

    // Unresolved evidence inside the valued complete month, and on an empty
    // trailing edge day.
    quarantine(
        &fx,
        fx.publisher_id,
        fx.platform_id,
        fx.sessions,
        d(2026, 4, 15),
        d(2026, 4, 16),
        "DAY",
    );
    let edge = quarantine(
        &fx,
        fx.publisher_id,
        fx.platform_id,
        fx.sessions,
        d(2026, 5, 5),
        d(2026, 5, 6),
        "DAY",
    );
    let month = at(MetricTimelineGrain::Month);
    assert_eq!(
        bucket_values(&month, fx.sessions),
        vec![some("0"), some("7"), None],
        "the monthly value stays exact; the empty edge bucket is not a zero"
    );
    assert_eq!(
        total(&month, fx.platform_id, fx.sessions),
        some("7"),
        "a projected total is never replaced by null"
    );
    assert_eq!(
        codes(&month),
        vec![MetricWarningCode::UnresolvedIdentifiers]
    );
    assert_eq!(month.coverage, clean.coverage);
    assert_eq!(month.data_through, clean.data_through);
    let day = at(MetricTimelineGrain::Day);
    let april_15 = (d(2026, 4, 15) - start).num_days() as usize;
    assert_eq!(bucket_values(&day, fx.sessions)[april_15], None);
    assert_eq!(bucket_values(&day, fx.sessions)[april_15 + 1], some("0"));

    // Terminal resolution of the edge evidence restores the edge zero; the
    // April evidence still warns.
    let support = commit(&fx, fx.works[1], fx.platform_id, fx.units, d(2026, 9, 1), 1);
    set_quarantine_reconciliation_state(&fx, edge, "RESOLVED_WINNER", Some(support));
    let resolved = at(MetricTimelineGrain::Month);
    assert_eq!(
        bucket_values(&resolved, fx.sessions),
        vec![some("0"), some("7"), some("0")]
    );
    assert_eq!(
        codes(&resolved),
        vec![MetricWarningCode::UnresolvedIdentifiers]
    );

    // An empty complete month overlapped by reporting-period evidence is not
    // a zero, whichever timeline serves it.
    quarantine(
        &fx,
        fx.publisher_id,
        fx.platform_id,
        fx.sessions,
        d(2026, 6, 10),
        d(2026, 7, 1),
        "REPORTING_PERIOD",
    );
    cover(
        &fx,
        fx.sessions,
        d(2026, 6, 1),
        d(2026, 8, 1),
        "COMPLETE",
        "COMPLETED",
        "'2026-08-20T00:00:00Z'",
    );
    let month_of = |start: NaiveDate, grain| {
        read(
            &fx,
            &request(
                fx.publisher_id,
                start,
                next_month(start),
                &[fx.platform_id],
                &[fx.sessions],
                Some(grain),
            ),
        )
    };
    let june = month_of(d(2026, 6, 1), MetricTimelineGrain::Month);
    assert_eq!(bucket_values(&june, fx.sessions), vec![None]);
    assert_eq!(total(&june, fx.platform_id, fx.sessions), None);
    assert_eq!(june.coverage.status, MetricCoverageStatus::Complete);
    let june_days = month_of(d(2026, 6, 1), MetricTimelineGrain::Day);
    let values = bucket_values(&june_days, fx.sessions);
    assert_eq!(values[8], some("0"), "9 June is before the evidence");
    assert_eq!(values[9], None, "10 June is inside it");
    // Control: July is covered and outside the evidence, so it is a zero.
    let july = month_of(d(2026, 7, 1), MetricTimelineGrain::Month);
    assert_eq!(bucket_values(&july, fx.sessions), vec![some("0")]);
    assert!(july.warnings.is_empty());
}

#[test]
fn identifier_quality_is_orthogonal_to_section_coverage_and_section_flags() {
    let (_guard, fx) = setup();
    // Days 1-2 fully covered; days 3-4 COMPLETE without the country
    // dimension. The value does not depend on country.
    cover_dimensions(&fx, 1, 3, true, true);
    cover_dimensions(&fx, 3, 5, false, true);
    commit(&fx, fx.works[0], fx.platform_id, fx.sessions, day_n(1), 9);
    apply_all(&fx.pool);
    let input = window(&fx, &[fx.sessions]);
    let flags = [(true, true), (true, false), (false, true), (false, false)];
    let with_flags = |countries: bool, institutions: bool| {
        let mut input = input.clone();
        input.include_countries = Some(countries);
        input.include_institutions = Some(institutions);
        read(&fx, &input)
    };
    let before: Vec<MetricDashboard> = flags
        .iter()
        .map(|(countries, institutions)| with_flags(*countries, *institutions))
        .collect();

    quarantine(
        &fx,
        fx.publisher_id,
        fx.platform_id,
        fx.sessions,
        day_n(1),
        day_n(2),
        "DAY",
    );
    quarantine(
        &fx,
        fx.publisher_id,
        fx.platform_id,
        fx.sessions,
        day_n(4),
        day_n(5),
        "DAY",
    );
    for ((countries, institutions), before) in flags.iter().zip(&before) {
        let after = with_flags(*countries, *institutions);
        let label = format!("countries={countries} institutions={institutions}");
        // Coverage, items, dimension flags and dataThrough are those the
        // sections alone decide.
        assert_eq!(after.coverage, before.coverage, "{label}");
        assert_eq!(after.data_through, before.data_through, "{label}");
        assert_eq!(after.rollup_watermark, before.rollup_watermark, "{label}");
        assert_eq!(after.countries, before.countries, "{label}");
        assert_eq!(after.institutions, before.institutions, "{label}");
        // No section flag suppresses the warning, in its fixed position.
        let mut expected = codes(before);
        let position = expected
            .iter()
            .position(|code| *code == MetricWarningCode::RollupLag)
            .unwrap_or(expected.len());
        expected.insert(position, MetricWarningCode::UnresolvedIdentifiers);
        assert_eq!(codes(&after), expected, "{label}");
        assert!(after.is_partial, "{label}");
        // The projected value stays; the evidence-overlapped empty day 4 is
        // no longer a zero, and days 2-3 keep the section-independent zeros.
        let values = bucket_values(&after, fx.sessions);
        assert_eq!(values[0], some("9"), "{label}");
        assert_eq!(values[1], some("0"), "{label}");
        assert_eq!(values[3], None, "{label}");
    }
    // Sections decided the shared status before and after.
    assert_eq!(before[0].coverage.status, MetricCoverageStatus::Partial);
    assert_eq!(before[3].coverage.status, MetricCoverageStatus::Complete);
    assert_eq!(before[3].data_through, Some(day_n(4)));
}

// ==========================================================================
// MET-WP4-03B performance correction: assertion statement equivalence
// ==========================================================================

/// The coverage-assertion statement as reviewed at `8bbbc508`, before the
/// performance correction: every requested day joined to every coverage row
/// whose period contains it. Kept only as the oracle the corrected
/// [`ASSERTIONS_SQL`] must reproduce row for row.
const PRE_CORRECTION_ASSERTIONS_SQL: &str = "WITH d AS MATERIALIZED ( \
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

/// One selected assertion row, comparable.
type AssertionKey = (
    Uuid,
    Uuid,
    Uuid,
    NaiveDate,
    MetricCoverageStatus,
    String,
    bool,
    bool,
);

fn assertion_rows(
    fx: &Fixture,
    statement: &str,
    accounts: &[Uuid],
    start: NaiveDate,
    end: NaiveDate,
    platforms: &[Uuid],
    measures: &[Uuid],
) -> Vec<AssertionKey> {
    let mut connection = fx.pool.get().expect("Failed to get DB connection");
    connection
        .build_transaction()
        .read_only()
        .repeatable_read()
        .run(|connection| {
            sql_query(CUSTOM_PLANS_SQL).execute(connection)?;
            sql_query(statement)
                .bind::<Array<SqlUuid>, _>(accounts)
                .bind::<Date, _>(start)
                .bind::<Date, _>(end)
                .bind::<Array<SqlUuid>, _>(platforms)
                .bind::<Array<SqlUuid>, _>(measures)
                .load::<AssertionRow>(connection)
        })
        .expect("the assertion statement")
        .into_iter()
        .map(|row| {
            (
                row.publisher_id,
                row.platform_id,
                row.measure_id,
                row.day,
                row.coverage_status,
                row.import_status,
                row.country_coverage,
                row.institution_coverage,
            )
        })
        .collect()
}

#[test]
fn the_corrected_assertion_statement_selects_exactly_what_the_reviewed_one_selected() {
    let (_guard, fx) = setup();
    // Accounts: the fixture's, one per other publisher and platform, and a
    // second platform for the selected publisher.
    let other_account = add_account(&fx, fx.platform_id, fx.other_publisher_id);
    let second_platform_account = add_account(&fx, fx.other_platform_id, fx.publisher_id);
    let accounts = [fx.account_id, other_account, second_platform_account];
    let publisher_of = |account: Uuid| {
        if account == other_account {
            fx.other_publisher_id
        } else {
            fx.publisher_id
        }
    };
    let platform_of = |account: Uuid| {
        if account == second_platform_account {
            fx.other_platform_id
        } else {
            fx.platform_id
        }
    };
    let base = d(2026, 1, 1);
    let mut rng = Rng(20_260_924);
    let completion = [
        "'2026-01-10T00:00:00Z'",
        "'2026-01-10T00:00:00Z'",
        "'2026-02-01T00:00:00Z'",
        "'2026-03-15T00:00:00Z'",
        "NULL",
    ];
    let statuses = [
        "COMPLETED",
        "COMPLETED",
        "COMPLETED_WITH_ERRORS",
        "PROCESSING",
        "FAILED",
    ];
    // Imports, some sharing a completion time so the import id decides, some
    // non-terminal, failed or without a completion time, and some scoped to
    // another publisher or none.
    let mut imports: Vec<(Uuid, Uuid)> = Vec::new();
    let mut statements = String::new();
    for _ in 0..60 {
        let import_id = Uuid::new_v4();
        let account = accounts[rng.below(3) as usize];
        let publisher = match rng.below(10) {
            0 => "NULL".to_string(),
            1 => format!("'{}'", fx.other_publisher_id),
            _ => format!("'{}'", publisher_of(account)),
        };
        statements.push_str(&scoped_import_sql(
            import_id,
            account,
            &publisher,
            statuses[rng.below(5) as usize],
            completion[rng.below(5) as usize],
        ));
        imports.push((import_id, account));
    }
    // Overlapping one- to forty-day coverage rows. Most name their import's
    // account; some name another account (an import owned elsewhere), and
    // some a platform their account does not serve.
    for _ in 0..400 {
        let (import_id, import_account) = imports[rng.below(imports.len() as u64) as usize];
        let account = if rng.below(8) == 0 {
            accounts[rng.below(3) as usize]
        } else {
            import_account
        };
        let platform = if rng.below(12) == 0 {
            fx.other_platform_id
        } else {
            platform_of(account)
        };
        let start = base + Duration::days(rng.below(90) as i64);
        let length = if rng.below(3) == 0 {
            1
        } else {
            1 + rng.below(40) as i64
        };
        statements.push_str(&coverage_sql(
            account,
            import_id,
            platform,
            [fx.sessions, fx.units][rng.below(2) as usize],
            start,
            start + Duration::days(length),
            ["COMPLETE", "PARTIAL", "UNKNOWN"][rng.below(3) as usize],
            rng.below(2) == 0,
            rng.below(2) == 0,
        ));
    }
    exec(&fx.pool, &statements);

    let platforms = [fx.platform_id, fx.other_platform_id];
    let measures = [fx.sessions, fx.units];
    let mut compared = 0;
    for (label, selected_accounts) in [
        ("every account", accounts.to_vec()),
        (
            "one publisher",
            vec![fx.account_id, second_platform_account],
        ),
        ("one account", vec![other_account]),
    ] {
        for (from, to) in [
            (0, 130),
            (10, 50),
            (37, 38),
            (-20, 5),
            (85, 140),
            (129, 140),
        ] {
            let (start, end) = (base + Duration::days(from), base + Duration::days(to));
            for (served_platforms, served_measures) in [
                (&platforms[..], &measures[..]),
                (&platforms[..1], &measures[1..]),
            ] {
                let expected = assertion_rows(
                    &fx,
                    PRE_CORRECTION_ASSERTIONS_SQL,
                    &selected_accounts,
                    start,
                    end,
                    served_platforms,
                    served_measures,
                );
                let corrected = assertion_rows(
                    &fx,
                    ASSERTIONS_SQL,
                    &selected_accounts,
                    start,
                    end,
                    served_platforms,
                    served_measures,
                );
                assert_eq!(corrected, expected, "{label}, [{start}, {end})");
                compared += expected.len();
            }
        }
    }
    assert!(compared > 1000, "the fixture exercised {compared} rows");
    // The day series is still a materialized relation.
    assert!(ASSERTIONS_SQL.contains("WITH d AS MATERIALIZED"));
    assert!(ASSERTIONS_SQL.contains("JOIN d ON d.day = covered.day::date"));
}

// ==========================================================================
// MET-WP4-03B: lag scope and statement shape
// ==========================================================================

#[test]
fn rollup_lag_is_scoped_to_the_resolved_works() {
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
    // Backlog on a work of the same publisher that the selector excludes.
    commit(&fx, fx.works[1], fx.platform_id, fx.sessions, day_n(2), 8);
    let mine = selected(&fx, &[fx.publisher_id], |s| {
        s.work_ids = Some(vec![fx.works[0]]);
    });
    let unaffected = read(&fx, &mine);
    assert!(
        unaffected.warnings.is_empty(),
        "unrelated backlog is not lag"
    );
    assert_eq!(unaffected.data_through, Some(day_n(4)));
    let everything = read(&fx, &window(&fx, &[fx.sessions]));
    assert_eq!(codes(&everything), vec![MetricWarningCode::RollupLag]);
    assert_eq!(everything.data_through, Some(day_n(1)));
    // Backlog on the selected work itself is lag.
    commit(&fx, fx.works[0], fx.platform_id, fx.sessions, day_n(3), 1);
    assert_eq!(codes(&read(&fx, &mine)), vec![MetricWarningCode::RollupLag]);
}

#[test]
fn the_statement_sequence_is_fixed_bounded_and_planned_for_its_arguments() {
    let (_guard, fx) = setup();
    let third = Uuid::new_v4();
    let third_work = Uuid::new_v4();
    exec(
        &fx.pool,
        &format!(
            "{}{}",
            publisher_sql(third, "SPHINX"),
            works_sql(third, &[third_work])
        ),
    );
    let series = add_series(&fx, imprint_of(&fx, fx.works[0]));
    add_issue(&fx, series, fx.works[0], 1);
    // Complete months, clipped edges, mixed representations that need the
    // base-cell statement, both sections, coverage and backlog.
    let institution_id = insert_institution(&fx.pool);
    for (day, dims, value) in [
        (d(2026, 3, 30), Dims::default(), 4),
        (
            d(2026, 3, 30),
            Dims {
                country: Some("GB"),
                ..Dims::default()
            },
            4,
        ),
        (
            d(2026, 4, 12),
            Dims {
                institution: Some(institution_id),
                ..Dims::default()
            },
            2,
        ),
        (d(2026, 6, 2), Dims::default(), 1),
    ] {
        commit_dims(
            &fx,
            fx.works[0],
            fx.platform_id,
            fx.sessions,
            day,
            dims,
            value,
        );
    }
    commit(
        &fx,
        third_work,
        fx.platform_id,
        fx.sessions,
        d(2026, 4, 2),
        9,
    );
    apply_all(&fx.pool);
    cover(
        &fx,
        fx.sessions,
        d(2026, 3, 20),
        d(2026, 6, 10),
        "COMPLETE",
        "COMPLETED",
        "'2026-06-10T00:00:00Z'",
    );
    commit(
        &fx,
        fx.works[0],
        fx.platform_id,
        fx.sessions,
        d(2026, 5, 5),
        1,
    );
    // Unresolved identifier evidence of a represented publisher.
    quarantine(
        &fx,
        fx.publisher_id,
        fx.platform_id,
        fx.sessions,
        d(2026, 4, 3),
        d(2026, 4, 4),
        "DAY",
    );
    let mut input = request(
        fx.publisher_id,
        d(2026, 3, 20),
        d(2026, 6, 10),
        &[],
        &[fx.sessions],
        Some(MetricTimelineGrain::Month),
    );
    input.platforms = None;
    input.selector.publisher_ids = Some(vec![fx.publisher_id, fx.other_publisher_id, third]);
    input.selector.series_ids = Some(vec![series]);
    let (result, statements) = captured_statements(&input);
    result.expect("the dashboard read");
    let expected = vec![
        "CHECKOUT_CHECK",
        "BEGIN",
        "CUSTOM_PLANS_SQL",
        "PUBLISHERS_SQL",
        "WORKS_SQL",
        "KNOWN_SQL",
        "REPRESENTED_SQL",
        "ACCOUNTS_SQL",
        "CANONICAL_SQL",
        "MONTHS_SQL",
        "DAY_SUMS_SQL",
        "DIMENSION_CELLS_SQL",
        "DAY_SECTIONS_SQL",
        "ASSERTIONS_SQL",
        "IDENTIFIER_QUALITY_SQL",
        "COMMIT",
    ];
    assert_eq!(
        statement_names(&statements),
        expected,
        "every statement of the longest path, each at most once"
    );
    // #946 Amendment 2 section 10: the pool check, transaction control, the
    // planner setting and the identifier-quality statement all count, and
    // the longest path is exactly the permitted sixteen.
    assert_eq!(statements.len(), 16);

    // Many more works, publishers' works, days and rows run the same
    // statements: nothing is issued per work, day, month or institution.
    let many: Vec<Uuid> = (0..60).map(|_| Uuid::new_v4()).collect();
    exec(&fx.pool, &works_sql(fx.publisher_id, &many));
    let mut rows = String::new();
    for (index, work_id) in many.iter().enumerate() {
        add_issue(&fx, series, *work_id, index as i32 + 2);
        for day in [d(2026, 3, 25), d(2026, 4, 20), d(2026, 6, 5)] {
            rows.push_str(&record_dims_sql(
                &fx,
                Uuid::new_v4(),
                *work_id,
                fx.platform_id,
                fx.sessions,
                day,
                Dims {
                    country: Some("US"),
                    ..Dims::default()
                },
                1,
            ));
            rows.push_str(&record_dims_sql(
                &fx,
                Uuid::new_v4(),
                *work_id,
                fx.platform_id,
                fx.sessions,
                day,
                Dims::default(),
                2,
            ));
        }
    }
    exec(&fx.pool, &rows);
    for (index, work_id) in many.iter().take(40).enumerate() {
        let extra = Uuid::new_v4();
        exec(
            &fx.pool,
            &format!(
                "INSERT INTO institution (institution_id, institution_name) \
                 VALUES ('{extra}', 'Institution {index}');"
            ),
        );
        commit_dims(
            &fx,
            *work_id,
            fx.platform_id,
            fx.sessions,
            d(2026, 4, 21),
            Dims {
                institution: Some(extra),
                country: Some("GB"),
                publication: None,
            },
            1,
        );
    }
    apply_all(&fx.pool);
    commit(&fx, many[0], fx.platform_id, fx.sessions, d(2026, 5, 6), 1);
    for offset in 0..30 {
        quarantine(
            &fx,
            fx.publisher_id,
            fx.platform_id,
            fx.sessions,
            d(2026, 3, 20) + Duration::days(offset * 2),
            d(2026, 3, 21) + Duration::days(offset * 2),
            "DAY",
        );
    }
    let (result, larger) = captured_statements(&input);
    let larger_dashboard = result.expect("the larger read");
    assert!(larger_dashboard.institutions.len() > 30);
    assert!(codes(&larger_dashboard).contains(&MetricWarningCode::UnresolvedIdentifiers));
    assert_eq!(statement_names(&larger), expected);

    // Custom plans are set inside the transaction, and only there.
    let mut connection = fx.pool.get().expect("Failed to get DB connection");
    #[derive(diesel::QueryableByName)]
    struct Setting {
        #[diesel(sql_type = Text)]
        plan_cache_mode: String,
    }
    let setting = |connection: &mut PgConnection| {
        sql_query("SHOW plan_cache_mode")
            .get_result::<Setting>(connection)
            .expect("the planner setting")
            .plan_cache_mode
    };
    let inside = connection
        .build_transaction()
        .read_only()
        .repeatable_read()
        .run(|connection| {
            sql_query(CUSTOM_PLANS_SQL).execute(connection)?;
            Ok::<_, diesel::result::Error>(setting(connection))
        })
        .expect("a read-only transaction");
    assert_eq!(inside, "force_custom_plan");
    assert_eq!(setting(&mut connection), "auto");
    // The per-publisher assertion day series is materialized.
    assert!(ASSERTIONS_SQL.contains("WITH d AS MATERIALIZED"));
}

// ==========================================================================
// Query-plan and latency evidence (run explicitly)
// ==========================================================================

fn uuid_array(ids: &[Uuid]) -> String {
    let quoted: Vec<String> = ids.iter().map(|id| format!("'{id}'")).collect();
    format!("ARRAY[{}]::uuid[]", quoted.join(","))
}

/// The identities a plan fixture serves.
struct PlanScope {
    third_publisher: Uuid,
    platforms: Vec<Uuid>,
    measures: Vec<Uuid>,
    imprint: Uuid,
    other_imprint: Uuid,
    series: Vec<Uuid>,
    funders: Vec<Uuid>,
    affiliated: Vec<Uuid>,
    works: Vec<Uuid>,
}

/// The first and last day of generated plan data.
fn plan_days() -> (NaiveDate, NaiveDate) {
    (d(2025, 3, 1), d(2026, 3, 31))
}

/// A production-shaped fixture of three entitled publishers.
///
/// The selected publisher A has 600 monographs and 100 chapters in one
/// imprint, each monograph in one of 20 series, English and every third also
/// French, one of 50 funders and two contributions affiliated with two of
/// 300 institutions. Over thirteen months A has daily sessions in three
/// countries, weekly signed units, a daily country-and-institution
/// breakdown of a third measure on a second platform, and a sparse
/// every-thirtieth-day aggregate across the rest of a 5x5 platform/measure
/// grid. Publishers B and C have 600 works each with daily sessions and
/// weekly units. Every publisher has one eligible account per platform with
/// a daily terminal import covering every measure, A also has monthly
/// reprocessing, and there is an unapplied backlog of 2,000 selected and
/// 3,000 unrelated work-day deltas. The monthly projections are then derived
/// by the reviewed `MET-WP4-03A` rebuild from the work-day rows alone.
fn plan_fixture(fx: &Fixture) -> PlanScope {
    advance_frontier(fx);
    let (first, last) = plan_days();
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
        for platform_id in &platforms {
            accounts.push_str(&account_sql(
                Uuid::new_v4(),
                driver,
                *platform_id,
                Some(publisher_id),
                true,
            ));
        }
    }
    let series: Vec<Uuid> = (0..20).map(|_| Uuid::new_v4()).collect();
    let (contributor, second_contributor) = (Uuid::new_v4(), Uuid::new_v4());
    let series_rows: Vec<String> = series
        .iter()
        .enumerate()
        .map(|(index, id)| format!("('{id}', 'book-series', 'Plan series {index}', '{imprint_a}')"))
        .collect();
    let statements = [
        publisher_sql(third_publisher, "SPHINX"),
        format!(
            "INSERT INTO imprint (imprint_id, publisher_id, imprint_name) VALUES \
             ('{imprint_a}', '{a}', 'Plan A'), ('{imprint_b}', '{b}', 'Plan B'), \
             ('{imprint_c}', '{third_publisher}', 'Plan C');",
            a = fx.publisher_id,
            b = fx.other_publisher_id,
        ),
        accounts,
        format!(
            "INSERT INTO work (work_id, work_type, work_status, imprint_id, edition) \
             SELECT gen_random_uuid(), 'monograph', 'forthcoming', imprint.id, 1 \
             FROM unnest({imprints}) AS imprint(id) CROSS JOIN generate_series(1, 600); \
             CREATE TEMPORARY TABLE plan_work ON COMMIT PRESERVE ROWS AS \
             SELECT work_id, row_number() OVER (ORDER BY work_id) AS rn \
             FROM work WHERE imprint_id = '{imprint_a}';",
            imprints = uuid_array(&[imprint_a, imprint_b, imprint_c]),
        ),
        format!(
            "INSERT INTO series (series_id, series_type, series_name, imprint_id) VALUES {rows}; \
             INSERT INTO issue (series_id, work_id, issue_ordinal) \
             SELECT ({series})[1 + w.rn % 20], w.work_id, 1 + w.rn / 20 FROM plan_work w; \
             INSERT INTO language (work_id, language_code, language_relation) \
             SELECT work_id, 'eng'::language_code, 'original'::language_relation FROM plan_work \
             UNION ALL SELECT work_id, 'fre', 'translated-into' FROM plan_work \
             WHERE rn % 3 = 0; \
             INSERT INTO institution (institution_id, institution_name, ror) \
             SELECT gen_random_uuid(), 'Plan institution ' || n, \
                    CASE WHEN n % 2 = 0 \
                         THEN 'https://ror.org/0' || lpad(n::text, 6, '0') || '12' END \
             FROM generate_series(1, 300) n; \
             CREATE TEMPORARY TABLE plan_institution ON COMMIT PRESERVE ROWS AS \
             SELECT institution_id, row_number() OVER (ORDER BY institution_id) - 1 AS n \
             FROM institution WHERE institution_name LIKE 'Plan institution %'; \
             INSERT INTO funding (work_id, institution_id) \
             SELECT w.work_id, i.institution_id FROM plan_work w \
             JOIN plan_institution i ON i.n = w.rn % 50; \
             INSERT INTO contributor (contributor_id, last_name, full_name) \
             VALUES ('{contributor}', 'Author', 'Plan Author'), \
                    ('{second_contributor}', 'Author', 'Second Author'); \
             INSERT INTO contribution (work_id, contributor_id, contribution_type, \
                                       main_contribution, last_name, full_name, \
                                       contribution_ordinal) \
             SELECT w.work_id, \
                    CASE WHEN o = 1 THEN '{contributor}'::uuid \
                         ELSE '{second_contributor}'::uuid END, \
                    'author', o = 1, 'Author', 'Plan Author', o \
             FROM plan_work w CROSS JOIN generate_series(1, 2) o; \
             INSERT INTO affiliation (contribution_id, institution_id, affiliation_ordinal) \
             SELECT c.contribution_id, i.institution_id, 1 \
             FROM contribution c JOIN plan_work w ON w.work_id = c.work_id \
             JOIN plan_institution i ON i.n = (w.rn * 7 + c.contribution_ordinal * 13) % 300;",
            rows = series_rows.join(", "),
            series = uuid_array(&series),
        ),
        format!(
            "CREATE TEMPORARY TABLE plan_chapter ON COMMIT PRESERVE ROWS AS \
             SELECT gen_random_uuid() AS work_id, w.work_id AS parent_id, w.rn \
             FROM plan_work w WHERE w.rn <= 100; \
             INSERT INTO work (work_id, work_type, work_status, imprint_id) \
             SELECT work_id, 'book-chapter', 'forthcoming', '{imprint_a}' FROM plan_chapter; \
             INSERT INTO work_relation (relator_work_id, related_work_id, relation_type, \
                                        relation_ordinal) \
             SELECT work_id, parent_id, 'is-child-of'::relation_type, 1 FROM plan_chapter \
             UNION ALL SELECT parent_id, work_id, 'has-child', 1 FROM plan_chapter;"
        ),
        format!(
            "INSERT INTO metric_rollup_work_day \
                 (work_id, platform_id, measure_id, day, country_code, value, watermark) \
             SELECT w.work_id, '{platform}', '{sessions}', g.day::date, c.code, \
                    1 + (random() * 20)::bigint, 1 \
             FROM plan_work w \
             CROSS JOIN generate_series(DATE '{first}', DATE '{last}', interval '1 day') AS g(day) \
             CROSS JOIN unnest(ARRAY['GB', 'US', 'DE']) AS c(code); \
             INSERT INTO metric_rollup_work_day \
                 (work_id, platform_id, measure_id, day, value, watermark) \
             SELECT w.work_id, '{platform}', '{units}', g.day::date, \
                    (random() * 10)::bigint - 3, 1 \
             FROM work w \
             CROSS JOIN generate_series(DATE '{first}', DATE '{last}', interval '7 day') AS g(day) \
             WHERE w.imprint_id IN ('{imprint_a}', '{imprint_b}', '{imprint_c}') \
               AND w.work_type = 'monograph'; \
             INSERT INTO metric_rollup_work_day \
                 (work_id, platform_id, measure_id, day, country_code, institution_id, value, \
                  watermark) \
             SELECT w.work_id, '{second_platform}', '{third_measure}', g.day::date, \
                    (ARRAY['GB', 'US', 'DE'])[1 + w.rn % 3], i.institution_id, \
                    1 + (random() * 5)::bigint, 1 \
             FROM plan_work w \
             JOIN plan_institution i ON i.n = w.rn % 300 \
             CROSS JOIN generate_series(DATE '{first}', DATE '{last}', interval '1 day') AS g(day); \
             INSERT INTO metric_rollup_work_day \
                 (work_id, platform_id, measure_id, day, value, watermark) \
             SELECT w.work_id, p.id, m.id, g.day::date, 1, 1 \
             FROM plan_work w \
             CROSS JOIN unnest({platforms}) AS p(id) \
             CROSS JOIN unnest({measures}) AS m(id) \
             CROSS JOIN generate_series(DATE '{first}', DATE '{last}', interval '30 day') AS g(day) \
             WHERE NOT (p.id = '{platform}' AND m.id IN ('{sessions}', '{units}')); \
             INSERT INTO metric_rollup_work_day \
                 (work_id, platform_id, measure_id, day, country_code, value, watermark) \
             SELECT w.work_id, '{platform}', '{sessions}', g.day::date, 'FR', 1, 1 \
             FROM work w \
             CROSS JOIN generate_series(DATE '{first}', DATE '{last}', interval '1 day') AS g(day) \
             WHERE w.imprint_id IN ('{imprint_b}', '{imprint_c}');",
            platform = fx.platform_id,
            sessions = fx.sessions,
            units = fx.units,
            second_platform = platforms[1],
            third_measure = measures[2],
            platforms = uuid_array(&platforms),
            measures = uuid_array(&measures),
        ),
        format!(
            "INSERT INTO metric_import \
                 (import_id, source_account_id, publisher_id, format_code, format_version, \
                  status, normalizer_version, created_by, period_start, period_end, completed_at) \
             SELECT gen_random_uuid(), sa.source_account_id, sa.expected_publisher_id, 'plan', \
                    '1', 'COMPLETED', 'plan/1', 'plan', g.day::date, g.day::date + 1, \
                    g.day + interval '26 hours' \
             FROM metric_source_account sa \
             CROSS JOIN generate_series(DATE '{first}', DATE '{last}', interval '1 day') AS g(day) \
             WHERE sa.expected_publisher_id IS NOT NULL; \
             INSERT INTO metric_import \
                 (import_id, source_account_id, publisher_id, format_code, format_version, \
                  status, normalizer_version, created_by, period_start, period_end, completed_at) \
             SELECT gen_random_uuid(), '{account}', '{publisher}', 'plan', '1', \
                    'COMPLETED_WITH_ERRORS', 'plan/1', 'plan', g.day::date, g.day::date + 30, \
                    g.day + interval '40 days' \
             FROM generate_series(DATE '{first}', DATE '{last}', interval '30 day') AS g(day); \
             INSERT INTO metric_coverage \
                 (source_account_id, import_id, platform_id, measure_id, period_start, \
                  period_end, coverage_status, country_coverage, institution_coverage) \
             SELECT mi.source_account_id, mi.import_id, sa.platform_id, m.id, \
                    mi.period_start, mi.period_end, 'COMPLETE', TRUE, TRUE \
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
                    w.work_id, '{platform}', '{sessions}', DATE '{first}' + ((w.rn + g.n) % 366)::int, \
                    DATE '{first}' + ((w.rn + g.n) % 366)::int + 1, 'DAY', 'NL', '{account}' \
             FROM (SELECT work_id, imprint_id, \
                          row_number() OVER (PARTITION BY imprint_id ORDER BY work_id) AS rn \
                   FROM work WHERE imprint_id IN ('{imprint_a}', '{imprint_b}') \
                     AND work_type = 'monograph') w \
             CROSS JOIN generate_series(1, 5) AS g(n) \
             WHERE (w.imprint_id = '{imprint_a}' AND w.rn <= 400) \
                OR (w.imprint_id = '{imprint_b}' AND w.rn <= 600); \
             INSERT INTO metric_record_revision \
                 (record_revision_id, record_id, revision_number, import_id, value, \
                  content_hash, status) \
             SELECT gen_random_uuid(), r.record_id, 1, '{import}', 1, \
                    'plan-' || r.record_id, 'CURRENT' \
             FROM metric_record r WHERE r.identity_hash LIKE 'plan-backlog-%'; \
             UPDATE metric_record r SET current_revision_id = v.record_revision_id \
             FROM metric_record_revision v \
             WHERE v.record_id = r.record_id AND r.identity_hash LIKE 'plan-backlog-%'; \
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
    // Unresolved identifier evidence (`MET-WP7-PREREQ-04`) for all three
    // publishers across the grid and the year: 180 day observations and 20
    // reporting periods, a quarter of them terminally resolved.
    let support = commit(fx, fx.works[1], fx.platform_id, fx.units, d(2020, 6, 1), 1);
    for index in 0..200_i64 {
        let publisher_id =
            [fx.publisher_id, fx.other_publisher_id, third_publisher][(index % 3) as usize];
        let day = first + Duration::days((index * 7) % 390);
        let (end, grain) = if index % 10 == 9 {
            (day + Duration::days(20), "REPORTING_PERIOD")
        } else {
            (day + Duration::days(1), "DAY")
        };
        let quarantine_id = quarantine(
            fx,
            publisher_id,
            platforms[(index % 5) as usize],
            measures[((index / 5) % 5) as usize],
            day,
            end,
            grain,
        );
        if index % 4 == 3 {
            set_quarantine_reconciliation_state(
                fx,
                quarantine_id,
                "RESOLVED_WINNER",
                Some(support),
            );
        }
    }
    exec(&fx.pool, "ANALYZE;");
    let rebuilt = Instant::now();
    rebuild_month_projections(&fx.pool).expect("rebuild the monthly projections");
    println!(
        "MET-WP4-03A rebuild of the plan fixture: {:?}",
        rebuilt.elapsed()
    );
    exec(&fx.pool, "ANALYZE;");

    let ids = |query: String| -> Vec<Uuid> {
        #[derive(diesel::QueryableByName)]
        struct Row {
            #[diesel(sql_type = SqlUuid)]
            id: Uuid,
        }
        let mut connection = fx.pool.get().expect("Failed to get DB connection");
        sql_query(query)
            .load::<Row>(&mut connection)
            .expect("fixture ids")
            .into_iter()
            .map(|row| row.id)
            .collect()
    };
    PlanScope {
        third_publisher,
        platforms,
        measures,
        imprint: imprint_a,
        other_imprint: imprint_b,
        series,
        funders: ids("SELECT institution_id AS id FROM institution \
             WHERE institution_name LIKE 'Plan institution %' \
             ORDER BY institution_id LIMIT 50"
            .to_string()),
        affiliated: ids(
            "SELECT DISTINCT a.institution_id AS id FROM affiliation a ORDER BY 1 LIMIT 10"
                .to_string(),
        ),
        works: ids(format!(
            "SELECT work_id AS id FROM work WHERE imprint_id = '{imprint_a}' \
             AND work_type = 'monograph' ORDER BY work_id"
        )),
    }
}

#[derive(diesel::QueryableByName)]
struct PlanLine {
    #[diesel(sql_type = Text, column_name = "QUERY PLAN")]
    line: String,
}

/// `EXPLAIN (ANALYZE, BUFFERS)` of one named resolver statement, prepared
/// with typed parameters and executed with the given argument expressions
/// inside a read-only repeatable-read transaction after `settings`, as the
/// resolver runs it after `CUSTOM_PLANS_SQL`.
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
    connection
        .batch_execute(settings)
        .expect("apply planner settings");
    connection
        .batch_execute(&format!(
            "PREPARE plan_statement ({parameter_types}) AS {statement}"
        ))
        .expect("prepare the statement");
    let plan: Vec<PlanLine> = sql_query(format!(
        "EXPLAIN (ANALYZE, BUFFERS) EXECUTE plan_statement ({arguments})"
    ))
    .load(&mut connection)
    .expect("explain the statement");
    println!("\n---- {name} [{settings}] ----");
    for line in plan {
        println!("{}", line.line);
    }
    connection
        .batch_execute("ROLLBACK;")
        .expect("end the plan transaction");
}

fn explain(name: &str, statement: &str, parameter_types: &str, arguments: &str) {
    explain_with(
        name,
        &format!("{CUSTOM_PLANS_SQL};"),
        statement,
        parameter_types,
        arguments,
    );
}

/// The host load average, recorded beside latency samples.
fn load_average() -> String {
    std::process::Command::new("uptime")
        .output()
        .map(|output| format!("uptime: {}", String::from_utf8_lossy(&output.stdout).trim()))
        .unwrap_or_else(|_| "uptime: unavailable".to_string())
}

/// Resolver wall-clock latency over 40 requests after three warm-up reads,
/// with the statement count of one captured read and the response shape.
fn latency(fx: &Fixture, label: &str, input: &MetricDashboardInput) {
    let (outcome, statements) = captured_statements(input);
    for _ in 0..3 {
        let _ = metric_dashboard(&fx.pool, input);
    }
    let mut samples: Vec<StdDuration> = (0..40)
        .map(|_| {
            let started = Instant::now();
            let result = metric_dashboard(&fx.pool, input);
            let elapsed = started.elapsed();
            assert_eq!(result.is_ok(), outcome.is_ok(), "{label}: a stable outcome");
            elapsed
        })
        .collect();
    samples.sort();
    let percentile = |p: f64| samples[((samples.len() as f64 * p).ceil() as usize).max(1) - 1];
    let shape = match &outcome {
        Ok(dashboard) => format!(
            "totals={} timeline_cells={} countries={} institutions={} coverage={:?} warnings={:?}",
            dashboard.totals.len(),
            dashboard.timeline.len(),
            dashboard.countries.len(),
            dashboard.institutions.len(),
            dashboard.coverage.status,
            codes(dashboard),
        ),
        Err(error) => format!("error={}", error.code()),
    };
    println!(
        "{label}: n=40 p50={:?} p95={:?} max={:?} statements={} | {shape}",
        percentile(0.50),
        percentile(0.95),
        samples[samples.len() - 1],
        statements.len(),
    );
    assert!(
        statements.len() <= 16,
        "{label}: {:?}",
        statement_names(&statements)
    );
}

#[test]
#[ignore = "query-plan and latency evidence; run explicitly with --ignored"]
fn metric_dashboard_query_plan_and_latency_evidence() {
    let (_guard, fx) = setup();
    let built = Instant::now();
    let scope = plan_fixture(&fx);
    println!("plan fixture built in {:?}", built.elapsed());
    for (label, query) in [
        (
            "publisher A works",
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
            "metric_rollup_work_month rows",
            "(SELECT COUNT(*) FROM metric_rollup_work_month)".to_string(),
        ),
        (
            "metric_rollup_work_country_month rows",
            "(SELECT COUNT(*) FROM metric_rollup_work_country_month)".to_string(),
        ),
        (
            "metric_rollup_work_institution_month rows",
            "(SELECT COUNT(*) FROM metric_rollup_work_institution_month)".to_string(),
        ),
        (
            "metric_rollup_work_month_ambiguity rows",
            "(SELECT COUNT(*) FROM metric_rollup_work_month_ambiguity)".to_string(),
        ),
        (
            "metric_coverage rows",
            "(SELECT COUNT(*) FROM metric_coverage)".to_string(),
        ),
        (
            "unresolved identifier quarantine rows",
            "(SELECT COUNT(*) FROM metric_identifier_quarantine q \
               LEFT JOIN metric_identifier_quarantine_reconciliation r \
                 ON r.identifier_quarantine_id = q.identifier_quarantine_id \
              WHERE r.resolved_at IS NULL)"
                .to_string(),
        ),
        (
            "unapplied work-day deltas",
            "(SELECT COUNT(*) FROM metric_rollup_delta WHERE status <> 'APPLIED')".to_string(),
        ),
        (
            "indexes on the monthly projections",
            "(SELECT COUNT(*) FROM pg_indexes WHERE tablename LIKE 'metric_rollup_work_%month%')"
                .to_string(),
        ),
    ] {
        println!("{label}: {}", scalar_i64(&fx.pool, &query));
    }

    // 366 days with clipped leading and trailing months: 15 March 2025 to
    // 15 March 2026 inclusive, so eleven complete months are monthly rows.
    let (start, end) = (d(2025, 3, 15), d(2026, 3, 16));
    let day_end = start + Duration::days(200);
    let three = vec![
        fx.publisher_id,
        fx.other_publisher_id,
        scope.third_publisher,
    ];
    let base = |publishers: &[Uuid], grain, to: NaiveDate| {
        let mut input = request(
            fx.publisher_id,
            start,
            to,
            &scope.platforms,
            &scope.measures,
            Some(grain),
        );
        input.selector.publisher_ids = Some(publishers.to_vec());
        input
    };
    let month = |publishers: &[Uuid]| base(publishers, MetricTimelineGrain::Month, end);

    // Plans of every statement with the arguments the resolver binds.
    let id_list = |query: String| -> String {
        #[derive(diesel::QueryableByName)]
        struct Row {
            #[diesel(sql_type = SqlUuid)]
            id: Uuid,
        }
        let mut connection = fx.pool.get().expect("Failed to get DB connection");
        let ids: Vec<Uuid> = sql_query(query)
            .load::<Row>(&mut connection)
            .expect("plan argument ids")
            .into_iter()
            .map(|row| row.id)
            .collect();
        uuid_array(&ids)
    };
    let works_of = |publishers: &[Uuid]| {
        id_list(format!(
            "SELECT w.work_id AS id FROM work w JOIN imprint i \
               ON i.imprint_id = w.imprint_id WHERE i.publisher_id = ANY({}) \
             ORDER BY 1",
            uuid_array(publishers)
        ))
    };
    let accounts_of = |publishers: &[Uuid]| {
        id_list(format!(
            "SELECT source_account_id AS id FROM metric_source_account \
               WHERE expected_publisher_id = ANY({}) ORDER BY 1",
            uuid_array(publishers)
        ))
    };
    let platforms = uuid_array(&scope.platforms);
    let measures = uuid_array(&scope.measures);
    let one = uuid_array(&[fx.publisher_id]);
    let split = Split::of(start, end);
    let (interior_start, interior_end) = (split.interior_start, split.interior_end);
    println!("\n{}", load_average());
    explain(
        "PUBLISHERS_SQL (3 publishers)",
        PUBLISHERS_SQL,
        "uuid[]",
        &uuid_array(&three),
    );
    explain(
        "WORKS_SQL (1 publisher, no other selector)",
        WORKS_SQL,
        "uuid[], uuid[], uuid[], uuid[], text[], text[], uuid[], uuid[], bigint",
        &format!("{one}, '{{}}', '{{}}', '{{}}', '{{}}', '{{}}', '{{}}', '{{}}', 2001"),
    );
    explain(
        "WORKS_SQL (series, language, funding and affiliation)",
        WORKS_SQL,
        "uuid[], uuid[], uuid[], uuid[], text[], text[], uuid[], uuid[], bigint",
        &format!(
            "{one}, '{{}}', {series}, '{{}}', '{{}}', ARRAY['eng'], {funders}, {affiliated}, 2001",
            series = uuid_array(&scope.series[..5]),
            funders = uuid_array(&scope.funders[..5]),
            affiliated = uuid_array(&scope.affiliated),
        ),
    );
    explain(
        "REPRESENTED_SQL (1 publisher, both filters omitted)",
        REPRESENTED_SQL,
        "uuid[], date, date, date, date, uuid[], uuid[], uuid[], text",
        &format!(
            "{works}, '{start}', '{interior_start}', '{interior_end}', '{end}', '{{}}', '{{}}', \
             {one}, 'DRIVER'",
            works = works_of(&[fx.publisher_id])
        ),
    );
    explain(
        "IDENTIFIER_QUALITY_SQL (3 represented publishers, 5x5, 366 days)",
        IDENTIFIER_QUALITY_SQL,
        "uuid[], date, date, uuid[], uuid[]",
        &format!(
            "{}, '{start}', '{end}', {platforms}, {measures}",
            uuid_array(&three)
        ),
    );
    explain(
        "ACCOUNTS_SQL (3 publishers, 5 platforms)",
        ACCOUNTS_SQL,
        "uuid[], uuid[], text",
        &format!("{}, {platforms}, 'DRIVER'", uuid_array(&three)),
    );
    explain(
        "CANONICAL_SQL (3 publishers' works, 5x5, 366 days, lagging)",
        CANONICAL_SQL,
        "boolean, bigint, uuid[], date, date, uuid[], uuid[]",
        &format!(
            "TRUE, 1, {works}, '{start}', '{end}', {platforms}, {measures}",
            works = works_of(&three)
        ),
    );
    for (label, publishers) in [
        ("1 publisher", vec![fx.publisher_id]),
        ("3 publishers", three.clone()),
    ] {
        explain(
            &format!("MONTHS_SQL ({label}, 5x5, 11 complete months, both sections)"),
            MONTHS_SQL,
            "uuid[], uuid[], uuid[], date, date, boolean, boolean, bigint",
            &format!(
                "{works}, {platforms}, {measures}, '{interior_start}', '{interior_end}', TRUE, \
                 TRUE, 2001",
                works = works_of(&publishers)
            ),
        );
        explain(
            &format!("DAY_SUMS_SQL ({label}, 5x5, clipped edges)"),
            DAY_SUMS_SQL,
            "uuid[], date, date, date, date, uuid[], uuid[]",
            &format!(
                "{works}, '{start}', '{interior_start}', '{interior_end}', '{end}', {platforms}, \
                 {measures}",
                works = works_of(&publishers)
            ),
        );
        explain(
            &format!("DAY_SECTIONS_SQL ({label}, 5x5, clipped edges, both sections)"),
            DAY_SECTIONS_SQL,
            "uuid[], date, date, date, date, uuid[], uuid[], boolean, boolean, bigint",
            &format!(
                "{works}, '{start}', '{interior_start}', '{interior_end}', '{end}', {platforms}, \
                 {measures}, TRUE, TRUE, 2001",
                works = works_of(&publishers)
            ),
        );
        explain(
            &format!("ASSERTIONS_SQL ({label}, 5x5, 366 days)"),
            ASSERTIONS_SQL,
            "uuid[], date, date, uuid[], uuid[]",
            &format!(
                "{accounts}, '{start}', '{end}', {platforms}, {measures}",
                accounts = accounts_of(&publishers)
            ),
        );
        // The reviewed pre-correction shape, for comparison in the same run.
        explain(
            &format!("PRE_CORRECTION_ASSERTIONS_SQL ({label}, 5x5, 366 days)"),
            PRE_CORRECTION_ASSERTIONS_SQL,
            "uuid[], date, date, uuid[], uuid[]",
            &format!(
                "{accounts}, '{start}', '{end}', {platforms}, {measures}",
                accounts = accounts_of(&publishers)
            ),
        );
    }
    explain_with(
        "ASSERTIONS_SQL (3 publishers, 5x5, 366 days) [generic]",
        "SET LOCAL plan_cache_mode = force_generic_plan;",
        ASSERTIONS_SQL,
        "uuid[], date, date, uuid[], uuid[]",
        &format!(
            "{accounts}, '{start}', '{end}', {platforms}, {measures}",
            accounts = accounts_of(&three)
        ),
    );
    explain(
        "DAY_SUMS_SQL (1 publisher, 5x5, 200 days, DAY timeline)",
        DAY_SUMS_SQL,
        "uuid[], date, date, date, date, uuid[], uuid[]",
        &format!(
            "{works}, '{start}', '{day_end}', '{day_end}', '{day_end}', {platforms}, {measures}",
            works = works_of(&[fx.publisher_id])
        ),
    );
    let candidate_days: Vec<String> = days_of(start, interior_start)
        .into_iter()
        .map(|day| format!("'{day}'"))
        .collect();
    explain(
        "DIMENSION_CELLS_SQL (1 publisher, leading edge of one platform x measure)",
        DIMENSION_CELLS_SQL,
        "uuid[], uuid[], uuid[], date[]",
        &format!(
            "{works}, {candidate_platforms}, {candidate_measures}, ARRAY[{days}]::date[]",
            works = works_of(&[fx.publisher_id]),
            candidate_platforms = uuid_array(&vec![scope.platforms[1]; candidate_days.len()]),
            candidate_measures = uuid_array(&vec![scope.measures[2]; candidate_days.len()]),
            days = candidate_days.join(","),
        ),
    );
    // Diagnostic: the generic plans the custom-plan setting avoids.
    let generic = "SET LOCAL plan_cache_mode = force_generic_plan;";
    explain_with(
        "MONTHS_SQL (1 publisher, 5x5, 11 complete months, both sections) [generic]",
        generic,
        MONTHS_SQL,
        "uuid[], uuid[], uuid[], date, date, boolean, boolean, bigint",
        &format!(
            "{works}, {platforms}, {measures}, '{interior_start}', '{interior_end}', TRUE, TRUE, \
             2001",
            works = works_of(&[fx.publisher_id])
        ),
    );
    explain_with(
        "DAY_SUMS_SQL (1 publisher, 5x5, 200 days, DAY timeline) [generic]",
        generic,
        DAY_SUMS_SQL,
        "uuid[], date, date, date, date, uuid[], uuid[]",
        &format!(
            "{works}, '{start}', '{day_end}', '{day_end}', '{day_end}', {platforms}, {measures}",
            works = works_of(&[fx.publisher_id])
        ),
    );

    println!("\n== latency ==\n{}", load_average());
    let a = [fx.publisher_id];
    latency(
        &fx,
        "S1 1 publisher, 366 days, 5x5, MONTH, both sections",
        &month(&a),
    );
    latency(
        &fx,
        "S2 3 publishers, 366 days, 5x5, MONTH, both sections",
        &month(&three),
    );
    let mut countries_only = month(&a);
    countries_only.include_institutions = Some(false);
    latency(
        &fx,
        "S3 1 publisher, 366 days, 5x5, MONTH, countries only",
        &countries_only,
    );
    let mut institutions_only = month(&a);
    institutions_only.include_countries = Some(false);
    latency(
        &fx,
        "S4 1 publisher, 366 days, 5x5, MONTH, institutions only",
        &institutions_only,
    );
    latency(
        &fx,
        "S5 1 publisher, 200 days, 5x5, DAY (5000 cells), both sections",
        &base(&a, MetricTimelineGrain::Day, day_end),
    );
    let mut single_day = base(&a, MetricTimelineGrain::Day, end);
    single_day.platforms = Some(vec![fx.platform_id]);
    single_day.measures = Some(vec![fx.sessions]);
    latency(
        &fx,
        "S5b 1 publisher, 366 days, 1x1, DAY (366 cells), both sections",
        &single_day,
    );
    latency(
        &fx,
        "S5c 3 publishers, 200 days, 5x5, DAY (5000 cells), both sections",
        &base(&three, MetricTimelineGrain::Day, day_end),
    );
    let mut five_hundred = month(&a);
    five_hundred.selector.work_ids = Some(scope.works[..500].to_vec());
    latency(
        &fx,
        "S6 500 workIds, 366 days, 5x5, MONTH, both sections",
        &five_hundred,
    );
    for (label, edit) in [
        (
            "S7a seriesIds (5 of 20)",
            Box::new(|s: &mut MetricSelectorInput| s.series_ids = Some(scope.series[..5].to_vec()))
                as Box<dyn Fn(&mut MetricSelectorInput)>,
        ),
        (
            "S7b languages [FRE]",
            Box::new(|s: &mut MetricSelectorInput| s.languages = Some(vec![LanguageCode::Fre])),
        ),
        (
            "S7c fundingInstitutionIds (5 of 50)",
            Box::new(|s: &mut MetricSelectorInput| {
                s.funding_institution_ids = Some(scope.funders[..5].to_vec())
            }),
        ),
        (
            "S7d affiliationInstitutionIds (10)",
            Box::new(|s: &mut MetricSelectorInput| {
                s.affiliation_institution_ids = Some(scope.affiliated.clone())
            }),
        ),
        (
            "S7e series + languages + funding + affiliation + workTypes",
            Box::new(|s: &mut MetricSelectorInput| {
                s.series_ids = Some(scope.series[..10].to_vec());
                s.languages = Some(vec![LanguageCode::Eng]);
                s.funding_institution_ids = Some(scope.funders.clone());
                s.affiliation_institution_ids = Some(scope.affiliated.clone());
                s.work_types = Some(vec![WorkType::Monograph, WorkType::BookChapter]);
            }),
        ),
    ] {
        let mut input = month(&a);
        edit(&mut input.selector);
        latency(
            &fx,
            &format!("{label}, 366 days, 5x5, MONTH, both sections"),
            &input,
        );
    }
    let mut zero = month(&a);
    zero.selector.imprint_ids = Some(vec![scope.other_imprint]);
    latency(
        &fx,
        "S8 zero-work intersection, 366 days, 5x5, MONTH",
        &zero,
    );
    let mut omitted = month(&a);
    omitted.platforms = None;
    omitted.measures = None;
    latency(
        &fx,
        "S9 1 publisher, filters omitted (resolves 5x5), MONTH, both sections",
        &omitted,
    );
    let mut own_imprint = month(&three);
    own_imprint.selector.imprint_ids = Some(vec![scope.imprint]);
    latency(
        &fx,
        "S10 3 publishers selected, 1 represented by imprint, MONTH, both sections",
        &own_imprint,
    );

    // An active native MONTH record intersecting the request fails it.
    let (native_record, native_revision) = (Uuid::new_v4(), Uuid::new_v4());
    exec(
        &fx.pool,
        &format!(
            "INSERT INTO metric_record (record_id, identity_hash, work_id, platform_id, \
                 measure_id, period_start, period_end, reporting_grain, \
                 winning_source_account_id) \
             VALUES ('{native_record}', 'plan-native', '{work}', '{platform}', '{measure}', \
                     '2025-06-01', '2025-07-01', 'MONTH', '{account}'); \
             INSERT INTO metric_record_revision (record_revision_id, record_id, \
                 revision_number, import_id, value, content_hash, status) \
             VALUES ('{native_revision}', '{native_record}', 1, '{import}', 5, 'plan-native', \
                     'CURRENT'); \
             UPDATE metric_record SET current_revision_id = '{native_revision}' \
              WHERE record_id = '{native_record}';",
            work = scope.works[0],
            platform = fx.platform_id,
            measure = fx.sessions,
            account = fx.account_id,
            import = fx.canonical_import,
        ),
    );
    latency(
        &fx,
        "S11 unsupported native MONTH grain, 366 days, 5x5, MONTH",
        &month(&a),
    );
    println!("{}", load_average());
}
