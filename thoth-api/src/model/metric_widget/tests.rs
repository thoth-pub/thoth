//! `MET-WP4-WIDGET-01` evidence for the public widget read.
//!
//! These tests drive the read directly, through [`resolve`] with a test
//! profile, against the dashboard's disposable-database fixture: the order of
//! operations and the captured statement sequence (every assertion naming an
//! engine statement, a captured sequence or the statement ceiling lives in
//! this file), the slots, the deadline and the projection lock, the public
//! scope, the sections and their child works, the pair bound, pair mode and
//! its equivalence to single-pair dashboard reads, the snapshot, the error
//! mapping and the log decision. Several selections in one GraphQL document
//! are proven here too, through the real schema, because they assert
//! captured statements.
//!
//! The SDL contract and the API-boundary behaviour are proven in
//! `crate::graphql::metric_widget_tests`.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use diesel::connection::SimpleConnection;
use diesel::{Connection, PgConnection};
use serde_json::{json, Value as JsonValue};
use uuid::Uuid;

use super::*;
use crate::graphql::{create_schema, Context, GraphQLRequest};
use crate::model::metric_dashboard::tests::{
    account_sql, apply_all, capturing_pool, commit, commit_dims, cover, coverage_sql, d, day_n,
    exec, exec_raw, import_sql, insert_institution, insert_measure, insert_publication,
    publisher_sql, quarantine, request, setup, source_sql, statement_names, statement_text, text,
    works_sql, Dims, Fixture,
};
use crate::model::metric_dashboard::{
    metric_dashboard, MetricDashboardInput, MetricWarningCode, ACCOUNTS_SQL, ASSERTIONS_SQL,
    CANONICAL_SQL, DAY_SECTIONS_SQL, DAY_SUMS_SQL, DIMENSION_CELLS_SQL, IDENTIFIER_QUALITY_SQL,
    KNOWN_SQL, METRIC_DASHBOARD_EARLIEST_DATE, METRIC_DASHBOARD_MAX_FUTURE_DAYS,
    MONTHS_FULL_SCAN_SQL, MONTHS_SCAN_SQL, MONTHS_SQL, MONTHS_YEAR_FULL_SCAN_SQL,
    MONTHS_YEAR_ONLY_SCAN_SQL, MONTHS_YEAR_ONLY_SQL, MONTHS_YEAR_SCAN_SQL, MONTHS_YEAR_SQL,
    PUBLISHERS_SQL, REPRESENTED_SQL, STATEMENT_TIMEOUT_SQL, WORKS_SQL,
};
use crate::model::tests::db::{test_context_anonymous, test_db_url, TestDbGuard};

// ==========================================================================
// Fixture
// ==========================================================================

/// The embedded book: the fixture's first work.
const BOOK_DOI: &str = "https://doi.org/10.5555/widget-book";
/// The same DOI as a consumer sends it.
const BOOK_BARE: &str = "10.5555/widget-book";

/// The read's month: March 2026, the dashboard fixture's default window.
fn march() -> (NaiveDate, NaiveDate) {
    (d(2026, 3, 1), d(2026, 4, 1))
}

/// A whole calendar year, January 2025 to January 2026: the widget and the
/// dashboard both read it with `MONTHS_YEAR_ONLY_SQL`.
fn year_2025() -> (NaiveDate, NaiveDate) {
    (d(2025, 1, 1), d(2026, 1, 1))
}

/// A fixed "today" well after the fixture's data, so no test depends on the
/// wall clock.
fn today() -> NaiveDate {
    d(2026, 11, 10)
}

/// The dashboard fixture, with the first work given [`BOOK_DOI`].
fn widget_setup() -> (TestDbGuard, Fixture) {
    let (guard, fx) = setup();
    set_doi(&fx.pool, fx.works[0], BOOK_DOI);
    (guard, fx)
}

fn set_doi(pool: &PgPool, work_id: Uuid, doi: &str) {
    exec(
        pool,
        &format!("UPDATE work SET doi = '{doi}' WHERE work_id = '{work_id}';"),
    );
}

fn imprint_of(pool: &PgPool, work_id: Uuid) -> Uuid {
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = SqlUuid)]
        imprint_id: Uuid,
    }
    let mut connection = pool.get().expect("a connection");
    diesel::sql_query(format!(
        "SELECT imprint_id FROM work WHERE work_id = '{work_id}'"
    ))
    .get_result::<Row>(&mut connection)
    .expect("the work's imprint")
    .imprint_id
}

/// One child work under `parent`, in `imprint`, with both relation rows.
fn child_sql(
    parent: Uuid,
    imprint: Uuid,
    child: Uuid,
    work_type: &str,
    doi: Option<&str>,
    ordinal: usize,
) -> String {
    let edition = if work_type == "book-chapter" {
        "NULL"
    } else {
        "1"
    };
    let doi = doi.map_or("NULL".to_string(), |doi| format!("'{doi}'"));
    format!(
        "INSERT INTO work (work_id, work_type, work_status, imprint_id, edition, doi) \
         VALUES ('{child}', '{work_type}', 'forthcoming', '{imprint}', {edition}, {doi}); \
         INSERT INTO work_relation (relator_work_id, related_work_id, relation_type, \
                                    relation_ordinal) \
         VALUES ('{parent}', '{child}', 'has-child', {ordinal}), \
                ('{child}', '{parent}', 'is-child-of', 1);"
    )
}

/// Add one chapter under `parent` and return its id.
fn add_chapter(
    pool: &PgPool,
    parent: Uuid,
    imprint: Uuid,
    doi: Option<&str>,
    ordinal: usize,
) -> Uuid {
    let child = Uuid::new_v4();
    exec(
        pool,
        &child_sql(parent, imprint, child, "book-chapter", doi, ordinal),
    );
    child
}

/// Add `count` chapters (no DOI) under `parent` in one batch.
fn add_chapters(pool: &PgPool, parent: Uuid, imprint: Uuid, count: usize) -> Vec<Uuid> {
    let children: Vec<Uuid> = (0..count).map(|_| Uuid::new_v4()).collect();
    let sql: String = children
        .iter()
        .enumerate()
        .map(|(index, child)| child_sql(parent, imprint, *child, "book-chapter", None, index + 1))
        .collect();
    exec(pool, &sql);
    children
}

/// One active native-grain canonical record (no rollup delta: the projection
/// admits only `DAY` records).
fn native_record(fx: &Fixture, work_id: Uuid, platform_id: Uuid, measure_id: Uuid) {
    let (record_id, revision_id) = (Uuid::new_v4(), Uuid::new_v4());
    let (start, end) = march();
    exec(
        &fx.pool,
        &format!(
            "INSERT INTO metric_record \
                 (record_id, identity_hash, work_id, platform_id, measure_id, period_start, \
                  period_end, reporting_grain, winning_source_account_id) \
             VALUES ('{record_id}', 'native-{record_id}', '{work_id}', '{platform_id}', \
                     '{measure_id}', '{start}', '{end}', 'MONTH', '{account}'); \
             INSERT INTO metric_record_revision \
                 (record_revision_id, record_id, revision_number, import_id, value, \
                  content_hash, status) \
             VALUES ('{revision_id}', '{record_id}', 1, '{import}', 50, \
                     'content-{revision_id}', 'CURRENT'); \
             UPDATE metric_record SET current_revision_id = '{revision_id}' \
              WHERE record_id = '{record_id}';",
            account = fx.account_id,
            import = fx.canonical_import,
        ),
    );
}

/// Complete coverage of `measure` on the fixture platform for March.
fn cover_march(fx: &Fixture, measure_id: Uuid) {
    let (start, end) = march();
    cover(
        fx,
        measure_id,
        start,
        end,
        "COMPLETE",
        "COMPLETED",
        "'2026-04-10T00:00:00Z'",
    );
}

fn input_for(doi: &str) -> MetricWidgetInput {
    let (start, end) = march();
    MetricWidgetInput {
        doi: Doi(doi.to_string()),
        chapter_dois: None,
        start_date: Some(start),
        end_date: end,
        timeline_grain: Some(MetricWidgetGrain::Month),
    }
}

fn input_over(doi: &str, (start, end): (NaiveDate, NaiveDate)) -> MetricWidgetInput {
    MetricWidgetInput {
        start_date: Some(start),
        end_date: end,
        ..input_for(doi)
    }
}

fn with_chapters(doi: &str, chapters: &[&str]) -> MetricWidgetInput {
    MetricWidgetInput {
        chapter_dois: Some(chapters.iter().map(|doi| Doi(doi.to_string())).collect()),
        ..input_for(doi)
    }
}

fn no_hook(_: WidgetStep) {}

/// The production profile with a fixed date and an optional hook.
fn profile(hook: &dyn Fn(WidgetStep)) -> Profile<'_> {
    Profile {
        statement_timeout_sql: METRIC_WIDGET_STATEMENT_TIMEOUT_SQL,
        deadline: METRIC_WIDGET_DEADLINE,
        checkout_timeout: METRIC_WIDGET_CHECKOUT_TIMEOUT,
        today: today(),
        hook,
    }
}

/// One resolution with a fresh claim counter and a recording log sink.
fn resolve_with(
    pool: &PgPool,
    switch: MetricWidgetSwitch,
    input: &MetricWidgetInput,
    profile: &Profile<'_>,
) -> (Result<MetricWidget, MetricWidgetError>, Vec<WidgetLogLine>) {
    let claims = AtomicUsize::new(0);
    let mut lines = Vec::new();
    let result = resolve(pool, switch, &claims, input, profile, &mut |emitted| {
        lines.extend_from_slice(emitted)
    });
    (result, lines)
}

/// One resolution with the switch on and the test profile.
fn read(pool: &PgPool, input: &MetricWidgetInput) -> Result<MetricWidget, MetricWidgetError> {
    resolve_with(pool, MetricWidgetSwitch::On, input, &profile(&no_hook)).0
}

/// One resolution on a capturing pool: the result, its log lines and every
/// statement it sent.
fn captured(
    input: &MetricWidgetInput,
    profile: &Profile<'_>,
) -> (
    Result<MetricWidget, MetricWidgetError>,
    Vec<WidgetLogLine>,
    Vec<&'static str>,
) {
    let (pool, log) = capturing_pool();
    let (result, lines) = resolve_with(&pool, MetricWidgetSwitch::On, input, profile);
    let statements = log.lock().expect("the statement log").clone();
    (result, lines, widget_statement_names(&statements))
}

/// Every engine statement, by name: one text per slot, and the eight texts
/// of the step-7 family.
const ENGINE_STATEMENTS: [(&str, &str); 19] = [
    ("PUBLISHERS_SQL", PUBLISHERS_SQL),
    ("WORKS_SQL", WORKS_SQL),
    ("KNOWN_SQL", KNOWN_SQL),
    ("REPRESENTED_SQL", REPRESENTED_SQL),
    ("ACCOUNTS_SQL", ACCOUNTS_SQL),
    ("CANONICAL_SQL", CANONICAL_SQL),
    ("MONTHS_SQL", MONTHS_SQL),
    ("MONTHS_SCAN_SQL", MONTHS_SCAN_SQL),
    ("MONTHS_FULL_SCAN_SQL", MONTHS_FULL_SCAN_SQL),
    ("MONTHS_YEAR_SQL", MONTHS_YEAR_SQL),
    ("MONTHS_YEAR_SCAN_SQL", MONTHS_YEAR_SCAN_SQL),
    ("MONTHS_YEAR_FULL_SCAN_SQL", MONTHS_YEAR_FULL_SCAN_SQL),
    ("MONTHS_YEAR_ONLY_SQL", MONTHS_YEAR_ONLY_SQL),
    ("MONTHS_YEAR_ONLY_SCAN_SQL", MONTHS_YEAR_ONLY_SCAN_SQL),
    ("DAY_SUMS_SQL", DAY_SUMS_SQL),
    ("DIMENSION_CELLS_SQL", DIMENSION_CELLS_SQL),
    ("DAY_SECTIONS_SQL", DAY_SECTIONS_SQL),
    ("ASSERTIONS_SQL", ASSERTIONS_SQL),
    ("IDENTIFIER_QUALITY_SQL", IDENTIFIER_QUALITY_SQL),
];

/// The widget's own statement naming: transaction control, the prelude, the
/// widget-owned statements and the engine's, each matched on its exact text.
/// A statement timeout other than the widget's is the test-only one.
fn widget_statement_names(statements: &[String]) -> Vec<&'static str> {
    statements
        .iter()
        .map(|statement| {
            let sql = statement_text(statement);
            if statement.starts_with("SELECT 1") {
                return "CHECKOUT_CHECK";
            }
            for (prefix, name) in [
                ("BEGIN", "BEGIN"),
                ("COMMIT", "COMMIT"),
                ("ROLLBACK", "ROLLBACK"),
            ] {
                if statement.starts_with(prefix) {
                    return name;
                }
            }
            let owned = [
                ("CUSTOM_PLANS_SQL", CUSTOM_PLANS_SQL),
                (
                    "METRIC_WIDGET_STATEMENT_TIMEOUT_SQL",
                    METRIC_WIDGET_STATEMENT_TIMEOUT_SQL,
                ),
                ("PROJECTION_LOCK_SQL", PROJECTION_LOCK_SQL),
                ("METRIC_WIDGET_RESOLUTION_SQL", METRIC_WIDGET_RESOLUTION_SQL),
                ("METRIC_WIDGET_PAIRS_SQL", METRIC_WIDGET_PAIRS_SQL),
                (
                    "METRIC_WIDGET_HISTORY_PAIRS_SQL",
                    METRIC_WIDGET_HISTORY_PAIRS_SQL,
                ),
                ("METRIC_WIDGET_LABELS_SQL", METRIC_WIDGET_LABELS_SQL),
            ];
            if let Some((name, _)) = owned
                .iter()
                .chain(ENGINE_STATEMENTS.iter())
                .find(|(_, text)| sql == *text)
            {
                return name;
            }
            if sql.starts_with("SET LOCAL statement_timeout = '") {
                return "TEST_STATEMENT_TIMEOUT_SQL";
            }
            panic!("an unnamed statement was executed: {statement}")
        })
        .collect()
}

/// The six statements one pair-mode engine run executes.
const ENGINE_RUN: [&str; 6] = [
    "PUBLISHERS_SQL",
    "ACCOUNTS_SQL",
    "CANONICAL_SQL",
    "MONTHS_SQL",
    "ASSERTIONS_SQL",
    "IDENTIFIER_QUALITY_SQL",
];

/// One pair-mode engine run whose step-7 text is `months`.
fn engine_run_with(months: &'static str) -> [&'static str; 6] {
    let mut run = ENGINE_RUN;
    run[3] = months;
    run
}

fn pairs_of(section: &MetricWidgetSection) -> Vec<(Uuid, Uuid)> {
    section
        .coverage
        .iter()
        .map(|item| (item.platform_id, item.measure_id))
        .collect()
}

fn total(section: &MetricWidgetSection, platform_id: Uuid, measure_id: Uuid) -> Option<String> {
    section
        .totals
        .iter()
        .find(|total| total.platform_id == platform_id && total.measure_id == measure_id)
        .and_then(|total| text(&total.value))
}

fn sorted<T: Ord + Clone>(values: &[T]) -> Vec<T> {
    let mut values = values.to_vec();
    values.sort();
    values
}

/// A book with two chapters and two served pairs in each section, so both
/// sections run the engine: the longest path.
fn longest_path(fx: &Fixture) -> (Uuid, Uuid) {
    let imprint = imprint_of(&fx.pool, fx.works[0]);
    let first = add_chapter(
        &fx.pool,
        fx.works[0],
        imprint,
        Some("https://doi.org/10.5555/widget-chapter-1"),
        1,
    );
    let second = add_chapter(
        &fx.pool,
        fx.works[0],
        imprint,
        Some("https://doi.org/10.5555/widget-chapter-2"),
        2,
    );
    cover_march(fx, fx.sessions);
    cover_march(fx, fx.units);
    for work in [fx.works[0], first, second] {
        commit(fx, work, fx.platform_id, fx.sessions, day_n(3), 4);
        commit(fx, work, fx.platform_id, fx.units, day_n(5), 2);
    }
    apply_all(&fx.pool);
    (first, second)
}

// ==========================================================================
// Statement sequence, ceiling and prelude
// ==========================================================================

#[test]
fn the_longest_path_runs_exactly_the_bounded_statement_sequence() {
    let (_guard, fx) = widget_setup();
    longest_path(&fx);
    let (result, _, names) = captured(&input_for(BOOK_BARE), &profile(&no_hook));
    let widget = result.expect("the longest path is served");
    assert_eq!(pairs_of(&widget.work).len(), 2);
    assert_eq!(
        pairs_of(widget.chapters.as_ref().expect("chapters are served")).len(),
        2
    );

    let mut expected = vec![
        "CHECKOUT_CHECK",
        "BEGIN",
        "CUSTOM_PLANS_SQL",
        "METRIC_WIDGET_STATEMENT_TIMEOUT_SQL",
        "PROJECTION_LOCK_SQL",
        "METRIC_WIDGET_RESOLUTION_SQL",
        "METRIC_WIDGET_PAIRS_SQL",
        "METRIC_WIDGET_PAIRS_SQL",
        "METRIC_WIDGET_LABELS_SQL",
    ];
    expected.extend(ENGINE_RUN);
    expected.extend(ENGINE_RUN);
    expected.push("COMMIT");
    assert_eq!(names, expected);
    // The ceiling: 19 named statements, 22 captured.
    assert_eq!(names.len(), 22);
    let named = names
        .iter()
        .filter(|name| !["CHECKOUT_CHECK", "BEGIN", "COMMIT"].contains(name))
        .count();
    assert_eq!(named, 19, "the statement ceiling is 19 named statements");
    // One connection and one transaction: one checkout, one BEGIN, one COMMIT.
    for control in ["CHECKOUT_CHECK", "BEGIN", "COMMIT"] {
        assert_eq!(names.iter().filter(|name| **name == control).count(), 1);
    }
    // The engine never runs its discovery or day statements in pair mode.
    for absent in [
        "WORKS_SQL",
        "KNOWN_SQL",
        "REPRESENTED_SQL",
        "DAY_SUMS_SQL",
        "DIMENSION_CELLS_SQL",
        "DAY_SECTIONS_SQL",
    ] {
        assert!(!names.contains(&absent), "{absent} must not run");
    }
}

#[test]
fn the_january_to_january_longest_path_reads_years_only_in_both_engine_runs() {
    let (_guard, fx) = widget_setup();
    longest_path(&fx);
    let input = input_over(BOOK_BARE, (d(2026, 1, 1), d(2027, 1, 1)));
    let (result, _, names) = captured(&input, &profile(&no_hook));
    let widget = result.expect("the longest path is served");
    assert_eq!(pairs_of(&widget.work).len(), 2);
    assert_eq!(
        pairs_of(widget.chapters.as_ref().expect("chapters are served")).len(),
        2
    );

    let mut expected = vec![
        "CHECKOUT_CHECK",
        "BEGIN",
        "CUSTOM_PLANS_SQL",
        "METRIC_WIDGET_STATEMENT_TIMEOUT_SQL",
        "PROJECTION_LOCK_SQL",
        "METRIC_WIDGET_RESOLUTION_SQL",
        "METRIC_WIDGET_PAIRS_SQL",
        "METRIC_WIDGET_PAIRS_SQL",
        "METRIC_WIDGET_LABELS_SQL",
    ];
    expected.extend(engine_run_with("MONTHS_YEAR_ONLY_SQL"));
    expected.extend(engine_run_with("MONTHS_YEAR_ONLY_SQL"));
    expected.push("COMMIT");
    assert_eq!(names, expected);
    // The same ceiling: 19 named statements, 22 captured.
    assert_eq!(names.len(), 22);
    let named = names
        .iter()
        .filter(|name| !["CHECKOUT_CHECK", "BEGIN", "COMMIT"].contains(name))
        .count();
    assert_eq!(named, 19, "the statement ceiling is 19 named statements");
}

#[test]
fn the_prelude_is_exact_and_precedes_the_first_query() {
    let (_guard, _fx) = widget_setup();
    let (result, _, names) = captured(&input_for(BOOK_BARE), &profile(&no_hook));
    result.expect("an entitled book with no data is served");
    assert_eq!(
        names[..6],
        [
            "CHECKOUT_CHECK",
            "BEGIN",
            "CUSTOM_PLANS_SQL",
            "METRIC_WIDGET_STATEMENT_TIMEOUT_SQL",
            "PROJECTION_LOCK_SQL",
            "METRIC_WIDGET_RESOLUTION_SQL",
        ]
    );
    // The widget's own timeout, not the dashboard's.
    assert_eq!(
        METRIC_WIDGET_STATEMENT_TIMEOUT_SQL,
        "SET LOCAL statement_timeout = '5s'"
    );
    assert_ne!(METRIC_WIDGET_STATEMENT_TIMEOUT_SQL, STATEMENT_TIMEOUT_SQL);
}

#[test]
fn the_profile_values_are_the_specified_bounds() {
    assert_eq!(METRIC_WIDGET_MAX_RANGE_MONTHS, 12);
    assert_eq!(METRIC_WIDGET_MAX_CHAPTER_DOIS, 500);
    assert_eq!(METRIC_WIDGET_MAX_CHILD_WORKS, 500);
    assert_eq!(METRIC_WIDGET_MAX_PAIRS, 40);
    assert_eq!(METRIC_WIDGET_SLOTS, 2);
    assert_eq!(METRIC_WIDGET_CHECKOUT_TIMEOUT, Duration::from_secs(1));
    assert_eq!(METRIC_WIDGET_DEADLINE, Duration::from_secs(10));
    assert_eq!(METRIC_WIDGET_END_DATE_CEILING_DAYS, 366);
    let production = Profile::production();
    assert_eq!(
        production.statement_timeout_sql,
        METRIC_WIDGET_STATEMENT_TIMEOUT_SQL
    );
    assert_eq!(production.deadline, METRIC_WIDGET_DEADLINE);
    assert_eq!(production.checkout_timeout, METRIC_WIDGET_CHECKOUT_TIMEOUT);
}

#[test]
fn the_date_floor_and_ceiling_are_the_dashboards_technical_domain() {
    assert_eq!(date_floor(), METRIC_DASHBOARD_EARLIEST_DATE);
    assert_eq!(
        METRIC_WIDGET_END_DATE_CEILING_DAYS,
        METRIC_DASHBOARD_MAX_FUTURE_DAYS
    );
}

#[test]
fn an_unknown_doi_and_an_unentitled_publisher_are_one_refusal_after_one_statement() {
    let (_guard, fx) = widget_setup();
    let obelisk = Uuid::new_v4();
    let obelisk_work = Uuid::new_v4();
    exec(
        &fx.pool,
        &format!(
            "{}{}",
            publisher_sql(obelisk, "OBELISK"),
            works_sql(obelisk, &[obelisk_work])
        ),
    );
    set_doi(
        &fx.pool,
        obelisk_work,
        "https://doi.org/10.5555/widget-obelisk",
    );
    let expected = [
        "CHECKOUT_CHECK",
        "BEGIN",
        "CUSTOM_PLANS_SQL",
        "METRIC_WIDGET_STATEMENT_TIMEOUT_SQL",
        "PROJECTION_LOCK_SQL",
        "METRIC_WIDGET_RESOLUTION_SQL",
        "ROLLBACK",
    ];
    let mut reasons = Vec::new();
    for doi in ["10.5555/widget-unknown", "10.5555/widget-obelisk"] {
        let (result, lines, names) = captured(&input_for(doi), &profile(&no_hook));
        assert_eq!(result, Err(MetricWidgetError::Unavailable), "{doi}");
        assert_eq!(names, expected, "{doi}");
        let reason = lines
            .iter()
            .find(|line| line.kind == WidgetLineKind::Reason)
            .expect("a reason line");
        assert_eq!(reason.step, WidgetStep::Entitlement);
        reasons.push(reason.reason);
    }
    assert_eq!(
        reasons,
        [
            Some(WidgetReason::UnknownWork),
            Some(WidgetReason::NotEntitled)
        ]
    );
}

#[test]
fn the_switch_off_and_input_failures_run_no_statement_and_take_no_slot() {
    // The capture log stays empty and every hook panics, so neither the
    // switch-off nor an input failure reaches a slot, a checkout or a
    // statement.
    let (pool, log) = capturing_pool();
    let (result, lines) = resolve_with(
        &pool,
        MetricWidgetSwitch::Off,
        &input_for(BOOK_BARE),
        &profile(&|step| panic!("step {step:?} reached with the switch off")),
    );
    assert_eq!(result, Err(MetricWidgetError::Unavailable));
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].level, log::Level::Debug);

    let mut bad = input_for(BOOK_BARE);
    bad.end_date = d(2027, 4, 1);
    let (result, _) = resolve_with(
        &pool,
        MetricWidgetSwitch::On,
        &bad,
        &profile(&|step| panic!("step {step:?} reached after an input failure")),
    );
    assert_eq!(
        result,
        Err(MetricWidgetError::QueryLimitExceeded(RANGE_MESSAGE))
    );
    assert!(log.lock().expect("the statement log").is_empty());
}

// ==========================================================================
// Several selections in one request (section 4.1 step 3)
// ==========================================================================

const SELECTION: &str = "metricWidget(input: {doi: \"10.5555/widget-book\", \
     startDate: \"2026-03-01\", endDate: \"2026-04-01\"}) \
     { excludedChildCount work { totals { value } } }";

async fn execute(context: &Context, query: &str) -> JsonValue {
    let request: GraphQLRequest =
        serde_json::from_value(json!({ "query": query })).expect("a GraphQL request");
    serde_json::to_value(request.execute(&create_schema(), context).await)
        .expect("a serialized response")
}

/// Execute one document with a fresh context, the switch on, on a capturing
/// pool: the response and the statements it sent.
async fn execute_captured(query: &str) -> (JsonValue, Vec<&'static str>) {
    let (pool, log) = capturing_pool();
    let context = test_context_anonymous(Arc::new(pool)).with_metric_widget(MetricWidgetSwitch::On);
    let response = execute(&context, query).await;
    let statements = log.lock().expect("the statement log").clone();
    (response, widget_statement_names(&statements))
}

fn error_types(response: &JsonValue) -> Vec<String> {
    response["errors"]
        .as_array()
        .map(|errors| {
            errors
                .iter()
                .map(|error| {
                    error["extensions"]["type"]
                        .as_str()
                        .unwrap_or("")
                        .to_string()
                })
                .collect()
        })
        .unwrap_or_default()
}

fn transactions(names: &[&str]) -> usize {
    names.iter().filter(|name| **name == "BEGIN").count()
}

fn served_fixture(fx: &Fixture) {
    cover_march(fx, fx.sessions);
    commit(fx, fx.works[0], fx.platform_id, fx.sessions, day_n(2), 3);
    apply_all(&fx.pool);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_second_selection_under_an_alias_or_the_same_key_is_refused() {
    let (_guard, fx) = widget_setup();
    served_fixture(&fx);
    for query in [
        format!("{{ a: {SELECTION} b: {SELECTION} }}"),
        format!("{{ {SELECTION} {SELECTION} }}"),
    ] {
        let (response, names) = execute_captured(&query).await;
        assert_eq!(
            error_types(&response),
            ["METRIC_QUERY_LIMIT_EXCEEDED"],
            "{query}: {response}"
        );
        assert_eq!(
            response["errors"][0]["message"],
            json!(ONE_SELECTION_MESSAGE)
        );
        assert_eq!(response["data"], JsonValue::Null, "{response}");
        assert_eq!(transactions(&names), 1, "{query}: {names:?}");
        assert_eq!(
            names
                .iter()
                .filter(|name| **name == "METRIC_WIDGET_RESOLUTION_SQL")
                .count(),
            1
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn repeated_fragments_serve_the_first_read_and_refuse_the_rest() {
    let (_guard, fx) = widget_setup();
    served_fixture(&fx);
    for query in [
        format!("{{ ...F ...F ...F }} fragment F on QueryRoot {{ {SELECTION} }}"),
        format!(
            "{{ ... on QueryRoot {{ {SELECTION} }} ... on QueryRoot {{ {SELECTION} }} \
             ... on QueryRoot {{ {SELECTION} }} }}"
        ),
    ] {
        let (response, names) = execute_captured(&query).await;
        assert_eq!(
            error_types(&response),
            ["METRIC_QUERY_LIMIT_EXCEEDED", "METRIC_QUERY_LIMIT_EXCEEDED"],
            "{query}: {response}"
        );
        assert_eq!(
            response["data"]["metricWidget"]["work"]["totals"][0]["value"],
            json!("3"),
            "{response}"
        );
        assert_eq!(transactions(&names), 1, "{query}: {names:?}");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_single_refused_selection_in_a_named_fragment_leaves_data_without_it() {
    let (_guard, _fx) = widget_setup();
    let query = "{ ...F } fragment F on QueryRoot { metricWidget(input: \
                 {doi: \"10.5555/widget-unknown\", startDate: \"2026-03-01\", \
                 endDate: \"2026-04-01\"}) { excludedChildCount } }";
    let (response, names) = execute_captured(query).await;
    assert_eq!(
        error_types(&response),
        ["METRIC_WIDGET_UNAVAILABLE"],
        "{response}"
    );
    assert_eq!(response["data"], json!({}), "{response}");
    assert_eq!(transactions(&names), 1);
}

// ==========================================================================
// Log decision (section 4.8)
// ==========================================================================

fn facts(error: Option<MetricWidgetError>, step: WidgetStep) -> WidgetFacts {
    WidgetFacts {
        switch: MetricWidgetSwitch::On,
        previous_claims: Some(0),
        error,
        reason: None,
        step,
        elapsed_ms: 12,
        doi: Some(Doi::from_str(BOOK_BARE).expect("a DOI")),
        work: Some(SectionSize {
            pairs: 2,
            cross_product: 4,
        }),
        chapters: None,
        grain: Some(MetricWidgetGrain::Month),
        history: Some(false),
        buckets: Some(1),
        engine_section: None,
        last_statement: None,
    }
}

fn levels(lines: &[WidgetLogLine]) -> Vec<(log::Level, WidgetLineKind)> {
    lines.iter().map(|line| (line.level, line.kind)).collect()
}

#[test]
fn a_served_resolution_is_one_info_outcome_line_with_the_doi_and_sizes() {
    let lines = log_decision(&facts(None, WidgetStep::Commit));
    assert_eq!(
        levels(&lines),
        [(log::Level::Info, WidgetLineKind::Outcome)]
    );
    let line = &lines[0];
    assert_eq!(line.error, None);
    assert_eq!(line.elapsed_ms, 12);
    assert!(line.doi.is_some());
    assert_eq!(
        line.work,
        Some(SectionSize {
            pairs: 2,
            cross_product: 4
        })
    );
    let message = line.message();
    assert!(message.contains("outcome=OK"), "{message}");
    assert!(message.contains("work_pairs/cross=2/4"), "{message}");
    assert!(
        message.contains("doi=https://doi.org/10.5555/widget-book"),
        "{message}"
    );
}

#[test]
fn the_first_second_selection_refusal_is_info_and_later_ones_debug() {
    let mut refused = facts(
        Some(MetricWidgetError::QueryLimitExceeded(ONE_SELECTION_MESSAGE)),
        WidgetStep::Claim,
    );
    refused.doi = None;
    refused.work = None;
    for (previous, level) in [
        (1, log::Level::Info),
        (2, log::Level::Debug),
        (7, log::Level::Debug),
    ] {
        refused.previous_claims = Some(previous);
        let lines = log_decision(&refused);
        assert_eq!(
            levels(&lines),
            [(level, WidgetLineKind::Outcome)],
            "{previous}"
        );
    }
}

#[test]
fn nothing_above_debug_is_written_with_the_switch_off() {
    let mut off = facts(Some(MetricWidgetError::Unavailable), WidgetStep::Switch);
    off.switch = MetricWidgetSwitch::Off;
    off.previous_claims = None;
    off.reason = Some(WidgetReason::SwitchOff);
    let lines = log_decision(&off);
    assert_eq!(
        levels(&lines),
        [(log::Level::Debug, WidgetLineKind::Reason)]
    );
    assert_eq!(lines[0].reason, Some(WidgetReason::SwitchOff));
}

#[test]
fn unavailable_and_busy_reasons_are_debug_with_the_doi_only_after_step_8() {
    for error in [MetricWidgetError::Unavailable, MetricWidgetError::Busy] {
        for (step, outcome_doi, reason_doi) in [
            (WidgetStep::Slot, false, false),
            (WidgetStep::Resolution, false, false),
            (WidgetStep::Entitlement, false, true),
            (WidgetStep::Engine, true, true),
        ] {
            let mut refused = facts(Some(error), step);
            refused.reason = Some(WidgetReason::NoSlot);
            let lines = log_decision(&refused);
            assert_eq!(
                levels(&lines),
                [
                    (log::Level::Info, WidgetLineKind::Outcome),
                    (log::Level::Debug, WidgetLineKind::Reason)
                ],
                "{error:?} {step:?}"
            );
            assert_eq!(lines[0].doi.is_some(), outcome_doi, "{error:?} {step:?}");
            assert_eq!(lines[1].doi.is_some(), reason_doi, "{error:?} {step:?}");
            assert_eq!(lines[1].reason, Some(WidgetReason::NoSlot));
            assert_eq!(lines[0].reason, None, "the outcome line carries no reason");
        }
    }
}

#[test]
fn a_checkout_failure_is_also_a_warn_line() {
    let mut refused = facts(Some(MetricWidgetError::Busy), WidgetStep::Checkout);
    refused.reason = Some(WidgetReason::CheckoutFailed);
    assert_eq!(
        levels(&log_decision(&refused)),
        [
            (log::Level::Info, WidgetLineKind::Outcome),
            (log::Level::Debug, WidgetLineKind::Reason),
            (log::Level::Warn, WidgetLineKind::Pool),
        ]
    );
}

#[test]
fn an_internal_error_is_an_error_line_with_section_step_statement_and_time() {
    let mut failed = facts(Some(MetricWidgetError::Internal), WidgetStep::Engine);
    failed.reason = Some(WidgetReason::EngineUnavailable);
    failed.engine_section = Some(WidgetSection::Chapters);
    failed.last_statement = Some("ASSERTIONS_SQL");
    let lines = log_decision(&failed);
    assert_eq!(
        levels(&lines),
        [
            (log::Level::Info, WidgetLineKind::Outcome),
            (log::Level::Error, WidgetLineKind::Internal),
        ]
    );
    let error = &lines[1];
    assert_eq!(error.reason, Some(WidgetReason::EngineUnavailable));
    assert_eq!(error.engine_section, Some(WidgetSection::Chapters));
    assert_eq!(error.step, WidgetStep::Engine);
    assert_eq!(error.last_statement, Some("ASSERTIONS_SQL"));
    assert_eq!(error.elapsed_ms, 12);
    assert!(error.doi.is_some());
    let message = error.message();
    for part in [
        "outcome=INTERNAL_ERROR",
        "step=13",
        "elapsed_ms=12",
        "reason=EngineUnavailable",
        "section=chapters",
        "last_statement=ASSERTIONS_SQL",
    ] {
        assert!(message.contains(part), "{message}");
    }
    // An internal error before step 9 carries no DOI.
    let mut early = facts(Some(MetricWidgetError::Internal), WidgetStep::Prelude);
    early.reason = Some(WidgetReason::Database);
    assert!(log_decision(&early).iter().all(|line| line.doi.is_none()));
}

#[test]
fn an_input_failure_is_one_info_outcome_line_without_the_doi() {
    let lines = log_decision(&facts(
        Some(MetricWidgetError::QueryInvalid(DATES_MESSAGE)),
        WidgetStep::Input,
    ));
    assert_eq!(
        levels(&lines),
        [(log::Level::Info, WidgetLineKind::Outcome)]
    );
    assert!(lines[0].doi.is_none());
    assert!(lines[0].message().contains("outcome=METRIC_QUERY_INVALID"));
}

#[test]
fn the_resolver_decides_its_log_lines_once_per_resolution() {
    // No database is reached: the counter refuses at step 3, or the input is
    // invalid at step 4.
    let (pool, log) = capturing_pool();
    let mut bad = input_for(BOOK_BARE);
    bad.start_date = Some(d(2026, 3, 2));
    for (previous, level, kind, step) in [
        (
            0,
            log::Level::Info,
            WidgetLineKind::Outcome,
            WidgetStep::Input,
        ),
        (
            1,
            log::Level::Info,
            WidgetLineKind::Outcome,
            WidgetStep::Claim,
        ),
        (
            2,
            log::Level::Debug,
            WidgetLineKind::Outcome,
            WidgetStep::Claim,
        ),
    ] {
        let claims = AtomicUsize::new(previous);
        let mut calls = Vec::new();
        let result = resolve(
            &pool,
            MetricWidgetSwitch::On,
            &claims,
            &bad,
            &profile(&no_hook),
            &mut |lines| calls.push(lines.to_vec()),
        );
        assert!(result.is_err());
        assert_eq!(calls.len(), 1, "one decision per resolution");
        assert_eq!(levels(&calls[0]), [(level, kind)], "previous {previous}");
        assert_eq!(calls[0][0].step, step);
        assert_eq!(claims.load(Ordering::SeqCst), previous + 1);
    }
    // With the switch off the counter is not claimed.
    let claims = AtomicUsize::new(0);
    let mut calls = 0;
    let _ = resolve(
        &pool,
        MetricWidgetSwitch::Off,
        &claims,
        &bad,
        &profile(&no_hook),
        &mut |_| calls += 1,
    );
    assert_eq!(calls, 1);
    assert_eq!(claims.load(Ordering::SeqCst), 0);
    assert!(log.lock().expect("the statement log").is_empty());
}

// ==========================================================================
// Error mapping (section 4.8)
// ==========================================================================

/// Every message the engine can produce: its message constants and its
/// variant messages.
fn engine_messages() -> Vec<&'static str> {
    use crate::model::metric_dashboard::*;
    let mut messages = vec![
        PUBLISHER_CARDINALITY,
        DUPLICATE_PUBLISHER,
        UNKNOWN_PUBLISHER,
        DUPLICATE_IMPRINT,
        DUPLICATE_SERIES,
        DUPLICATE_WORK,
        DUPLICATE_WORK_TYPE,
        DUPLICATE_LANGUAGE,
        DUPLICATE_FUNDING,
        DUPLICATE_AFFILIATION,
        TOO_MANY_IMPRINTS,
        TOO_MANY_SERIES,
        TOO_MANY_WORKS,
        TOO_MANY_WORK_TYPES,
        TOO_MANY_LANGUAGES,
        TOO_MANY_FUNDING,
        TOO_MANY_AFFILIATION,
        UNKNOWN_IMPRINT,
        UNKNOWN_SERIES,
        UNKNOWN_WORK,
        UNKNOWN_FUNDING,
        UNKNOWN_AFFILIATION,
        TOO_MANY_RESOLVED_WORKS,
        TOO_MANY_INSTITUTIONS,
        DATE_ORDER,
        RANGE_TOO_LONG,
        DAY_NEEDS_START,
        START_BEFORE_FLOOR,
        END_TOO_LATE,
        END_NOT_AFTER_FLOOR,
        EVIDENCE_BEFORE_FLOOR,
        DUPLICATE_MEASURE,
        DUPLICATE_PLATFORM,
        TOO_MANY_MEASURES,
        TOO_MANY_PLATFORMS,
        UNKNOWN_MEASURE,
        UNKNOWN_PLATFORM,
        NON_ADDITIVE_MEASURE,
        TOO_MANY_COMBINATIONS,
        TOO_MANY_CELLS,
        AGGREGATE_OUT_OF_RANGE,
    ];
    messages.extend(engine_variants().iter().map(MetricReadError::message));
    messages
}

/// Today's `MetricReadError` variants, listed as values (no exhaustive
/// `match`, so a later variant needs no change here).
fn engine_variants() -> Vec<MetricReadError> {
    use crate::model::metric_dashboard::{TOO_MANY_CELLS, UNKNOWN_WORK};
    vec![
        MetricReadError::Unauthorised,
        MetricReadError::QueryInvalid(UNKNOWN_WORK),
        MetricReadError::QueryLimitExceeded(AGGREGATE_OUT_OF_RANGE),
        MetricReadError::QueryLimitExceeded(TOO_MANY_CELLS),
        MetricReadError::RegistryLimitExceeded,
        MetricReadError::DimensionScopeAmbiguous,
        MetricReadError::UnsupportedSourceGrain,
        MetricReadError::Unavailable,
        MetricReadError::Busy,
    ]
}

/// Every widget error value.
fn widget_errors() -> Vec<MetricWidgetError> {
    let mut errors = vec![
        MetricWidgetError::Unavailable,
        MetricWidgetError::Busy,
        MetricWidgetError::Internal,
    ];
    for message in [
        INVALID_DOI_MESSAGE,
        DATES_MESSAGE,
        DUPLICATE_CHAPTER_DOI_MESSAGE,
        NOT_A_CHILD_MESSAGE,
        START_REQUIRED_MESSAGE,
    ] {
        errors.push(MetricWidgetError::QueryInvalid(message));
    }
    for message in [
        ONE_SELECTION_MESSAGE,
        DATE_DOMAIN_MESSAGE,
        RANGE_MESSAGE,
        TOO_MANY_CHAPTER_DOIS_MESSAGE,
        TOO_MANY_CHILDREN_MESSAGE,
        TOO_MANY_PAIRS_MESSAGE,
        AGGREGATE_MESSAGE,
        OMITTED_START_DOMAIN_MESSAGE,
        HISTORY_FLOOR_MESSAGE,
    ] {
        errors.push(MetricWidgetError::QueryLimitExceeded(message));
    }
    errors
}

#[test]
fn every_engine_error_maps_to_its_widget_classification() {
    let expected = [
        (
            MetricWidgetError::Unavailable,
            Some(WidgetReason::EngineUnauthorised),
        ),
        (
            MetricWidgetError::Internal,
            Some(WidgetReason::EngineRejected),
        ),
        (
            MetricWidgetError::QueryLimitExceeded(AGGREGATE_MESSAGE),
            None,
        ),
        (
            MetricWidgetError::Internal,
            Some(WidgetReason::EngineRejected),
        ),
        (
            MetricWidgetError::Internal,
            Some(WidgetReason::EngineRejected),
        ),
        (
            MetricWidgetError::Unavailable,
            Some(WidgetReason::DimensionScopeAmbiguous),
        ),
        (
            MetricWidgetError::Unavailable,
            Some(WidgetReason::UnsupportedSourceGrain),
        ),
        (
            MetricWidgetError::Internal,
            Some(WidgetReason::EngineUnavailable),
        ),
        // The dashboard guard's busy refusal, which a widget read never
        // takes, reaches the wildcard: INTERNAL_ERROR, never retryable.
        (
            MetricWidgetError::Internal,
            Some(WidgetReason::EngineRejected),
        ),
    ];
    let variants = engine_variants();
    assert_eq!(variants.len(), expected.len());
    for (variant, (error, reason)) in variants.iter().zip(expected) {
        let failure = engine_failure(variant, false);
        assert_eq!(failure.error, error, "{variant:?}");
        assert_eq!(failure.reason, reason, "{variant:?}");
        // The deadline flag decides BUSY alone, whatever the variant.
        assert_eq!(
            engine_failure(variant, true),
            Failure::new(MetricWidgetError::Busy, WidgetReason::DeadlinePassed),
            "{variant:?}"
        );
    }
}

#[test]
fn every_widget_error_has_its_fixed_type_and_message() {
    let table = [
        (
            MetricWidgetError::Unavailable,
            "METRIC_WIDGET_UNAVAILABLE",
            "Metrics are not available for this work.",
        ),
        (
            MetricWidgetError::Busy,
            "METRIC_WIDGET_BUSY",
            "Metrics are busy. Try again shortly.",
        ),
        (
            MetricWidgetError::QueryLimitExceeded(ONE_SELECTION_MESSAGE),
            "METRIC_QUERY_LIMIT_EXCEEDED",
            "Only one metricWidget field may be requested at a time.",
        ),
        (
            MetricWidgetError::QueryInvalid(INVALID_DOI_MESSAGE),
            "METRIC_QUERY_INVALID",
            "doi and chapterDois must be valid DOIs.",
        ),
        (
            MetricWidgetError::QueryInvalid(DATES_MESSAGE),
            "METRIC_QUERY_INVALID",
            "startDate and endDate must be first days of months, with startDate before endDate.",
        ),
        (
            MetricWidgetError::QueryLimitExceeded(DATE_DOMAIN_MESSAGE),
            "METRIC_QUERY_LIMIT_EXCEEDED",
            "startDate must be on or after 2000-01-01, and endDate at most 366 days after today (UTC).",
        ),
        (
            MetricWidgetError::QueryLimitExceeded(RANGE_MESSAGE),
            "METRIC_QUERY_LIMIT_EXCEEDED",
            "A MONTH timeline may cover at most 12 months.",
        ),
        (
            MetricWidgetError::QueryInvalid(START_REQUIRED_MESSAGE),
            "METRIC_QUERY_INVALID",
            "startDate is required for a MONTH timeline.",
        ),
        (
            MetricWidgetError::QueryLimitExceeded(OMITTED_START_DOMAIN_MESSAGE),
            "METRIC_QUERY_LIMIT_EXCEEDED",
            "When startDate is omitted, endDate must be after 2000-01-01 and at most 366 days after today (UTC).",
        ),
        (
            MetricWidgetError::QueryLimitExceeded(HISTORY_FLOOR_MESSAGE),
            "METRIC_QUERY_LIMIT_EXCEEDED",
            "Metrics for this work or its child works start before 2000-01-01. Give a startDate on or after 2000-01-01.",
        ),
        (
            MetricWidgetError::QueryLimitExceeded(TOO_MANY_CHAPTER_DOIS_MESSAGE),
            "METRIC_QUERY_LIMIT_EXCEEDED",
            "chapterDois may hold at most 500 DOIs.",
        ),
        (
            MetricWidgetError::QueryInvalid(DUPLICATE_CHAPTER_DOI_MESSAGE),
            "METRIC_QUERY_INVALID",
            "chapterDois must not repeat a DOI.",
        ),
        (
            MetricWidgetError::QueryInvalid(NOT_A_CHILD_MESSAGE),
            "METRIC_QUERY_INVALID",
            "chapterDois must list child works of doi.",
        ),
        (
            MetricWidgetError::QueryLimitExceeded(TOO_MANY_CHILDREN_MESSAGE),
            "METRIC_QUERY_LIMIT_EXCEEDED",
            "This work has more child works than the widget supports.",
        ),
        (
            MetricWidgetError::QueryLimitExceeded(TOO_MANY_PAIRS_MESSAGE),
            "METRIC_QUERY_LIMIT_EXCEEDED",
            "This work has more platform and measure combinations than the widget supports.",
        ),
        (
            MetricWidgetError::QueryLimitExceeded(AGGREGATE_MESSAGE),
            "METRIC_QUERY_LIMIT_EXCEEDED",
            "Metrics for this work exceed the supported range.",
        ),
        (
            MetricWidgetError::Internal,
            "INTERNAL_ERROR",
            "Metrics could not be read.",
        ),
    ];
    for (error, code, message) in table {
        assert_eq!(error.code(), code);
        assert_eq!(error.message(), message);
        let field_error = error.into_field_error();
        assert_eq!(field_error.message(), message);
        assert_eq!(
            field_error.extensions(),
            &graphql_value!({ "type": code }),
            "{error:?}"
        );
    }
}

#[test]
fn no_widget_error_carries_an_engine_message_sql_or_a_package_value() {
    let engine = engine_messages();
    for error in widget_errors() {
        let message = error.message();
        for engine_message in &engine {
            assert_ne!(message, *engine_message, "{error:?}");
            assert!(!message.contains(engine_message), "{error:?}");
        }
        for forbidden in [
            "SELECT",
            "LOCK",
            "SET ",
            "SPHINX",
            "PYRAMID",
            "OBELISK",
            "OASIS",
            "METRICS_WIDGET",
            "THOTH_",
            "package",
            "capability",
        ] {
            assert!(!message.contains(forbidden), "{error:?}: {forbidden}");
        }
    }
}

// ==========================================================================
// Input validation (section 4.1 step 4)
// ==========================================================================

fn validated(input: &MetricWidgetInput) -> Result<ValidatedInput, MetricWidgetError> {
    validate(input, today()).map_err(|failure| failure.error)
}

#[test]
fn dates_must_be_whole_months_inside_the_domain_and_at_most_twelve() {
    let with_dates = |start: NaiveDate, end: NaiveDate| MetricWidgetInput {
        start_date: Some(start),
        end_date: end,
        ..input_for(BOOK_BARE)
    };
    let invalid = Err(MetricWidgetError::QueryInvalid(DATES_MESSAGE));
    assert_eq!(
        validated(&with_dates(d(2026, 3, 2), d(2026, 4, 1))),
        invalid
    );
    assert_eq!(
        validated(&with_dates(d(2026, 3, 1), d(2026, 4, 2))),
        invalid
    );
    assert_eq!(
        validated(&with_dates(d(2026, 3, 1), d(2026, 3, 1))),
        invalid
    );
    assert_eq!(
        validated(&with_dates(d(2026, 4, 1), d(2026, 3, 1))),
        invalid
    );

    let domain = Err(MetricWidgetError::QueryLimitExceeded(DATE_DOMAIN_MESSAGE));
    assert_eq!(
        validated(&with_dates(d(1999, 12, 1), d(2000, 2, 1))),
        domain
    );
    assert!(validated(&with_dates(d(2000, 1, 1), d(2000, 2, 1))).is_ok());
    // 366 days after 2026-11-10 is 2027-11-11.
    assert!(validated(&with_dates(d(2027, 1, 1), d(2027, 11, 1))).is_ok());
    assert_eq!(
        validated(&with_dates(d(2027, 1, 1), d(2027, 12, 1))),
        domain
    );

    assert!(validated(&with_dates(d(2025, 3, 1), d(2026, 3, 1))).is_ok());
    assert_eq!(
        validated(&with_dates(d(2025, 3, 1), d(2026, 4, 1))),
        Err(MetricWidgetError::QueryLimitExceeded(RANGE_MESSAGE))
    );
}

#[test]
fn chapter_dois_are_bounded_parsed_and_unique_in_canonical_form() {
    let many: Vec<String> = (0..501).map(|n| format!("10.5555/c-{n}")).collect();
    let refs: Vec<&str> = many.iter().map(String::as_str).collect();
    assert_eq!(
        validated(&with_chapters(BOOK_BARE, &refs)),
        Err(MetricWidgetError::QueryLimitExceeded(
            TOO_MANY_CHAPTER_DOIS_MESSAGE
        ))
    );
    assert!(validated(&with_chapters(BOOK_BARE, &refs[..500])).is_ok());
    assert_eq!(
        validated(&with_chapters(BOOK_BARE, &["10.5555/ok", "not a doi"])),
        Err(MetricWidgetError::QueryInvalid(INVALID_DOI_MESSAGE))
    );
    assert_eq!(
        validated(&with_chapters(
            BOOK_BARE,
            &["10.5555/Chapter", "https://doi.org/10.5555/CHAPTER"]
        )),
        Err(MetricWidgetError::QueryInvalid(
            DUPLICATE_CHAPTER_DOI_MESSAGE
        ))
    );
}

#[test]
fn the_first_input_failure_in_step_4_order_is_returned() {
    // A bad DOI wins over bad dates, bad dates over the domain, the domain
    // over the range, the range over the entry count, the count over entry
    // parsing, and parsing over duplicates.
    let mut input = with_chapters("nonsense", &["also nonsense"]);
    input.start_date = Some(d(1990, 1, 2));
    assert_eq!(
        validated(&input),
        Err(MetricWidgetError::QueryInvalid(INVALID_DOI_MESSAGE))
    );
    input.doi = Doi(BOOK_BARE.to_string());
    assert_eq!(
        validated(&input),
        Err(MetricWidgetError::QueryInvalid(DATES_MESSAGE))
    );
    input.start_date = Some(d(1990, 1, 1));
    input.end_date = d(1990, 2, 1);
    assert_eq!(
        validated(&input),
        Err(MetricWidgetError::QueryLimitExceeded(DATE_DOMAIN_MESSAGE))
    );
    input.start_date = Some(d(2020, 1, 1));
    input.end_date = d(2022, 1, 1);
    assert_eq!(
        validated(&input),
        Err(MetricWidgetError::QueryLimitExceeded(RANGE_MESSAGE))
    );
    input.end_date = d(2020, 2, 1);
    input.chapter_dois = Some((0..501).map(|_| Doi("also nonsense".to_string())).collect());
    assert_eq!(
        validated(&input),
        Err(MetricWidgetError::QueryLimitExceeded(
            TOO_MANY_CHAPTER_DOIS_MESSAGE
        ))
    );
    input.chapter_dois = Some(vec![
        Doi("10.5555/a".to_string()),
        Doi("10.5555/a".to_string()),
        Doi("also nonsense".to_string()),
    ]);
    assert_eq!(
        validated(&input),
        Err(MetricWidgetError::QueryInvalid(INVALID_DOI_MESSAGE))
    );
}

#[test]
fn the_three_doi_spellings_parse_to_one_lookup_key() {
    let keys: Vec<String> = [
        "10.1234/abc",
        "https://doi.org/10.1234/abc",
        "HTTP://DX.DOI.ORG/10.1234/ABC",
    ]
    .iter()
    .map(|doi| {
        validated(&input_for(doi))
            .expect("a valid DOI")
            .doi
            .to_lowercase_string()
    })
    .collect();
    assert_eq!(keys, ["https://doi.org/10.1234/abc"; 3]);
}

// ==========================================================================
// Slots, checkout, deadline and the projection lock
// ==========================================================================

#[test]
fn with_both_slots_held_a_third_read_is_busy_without_a_checkout() {
    let (_guard, fx) = widget_setup();
    let first = try_take_slot().expect("a first slot");
    let second = try_take_slot().expect("a second slot");
    assert!(try_take_slot().is_none());
    let (result, lines, names) = captured(&input_for(BOOK_BARE), &profile(&no_hook));
    assert_eq!(result, Err(MetricWidgetError::Busy));
    assert!(names.is_empty(), "no checkout and no statement: {names:?}");
    assert!(lines
        .iter()
        .any(|line| line.reason == Some(WidgetReason::NoSlot)));
    drop((first, second));
    assert_eq!(SLOTS_IN_USE.load(Ordering::SeqCst), 0);
    read(&fx.pool, &input_for(BOOK_BARE)).expect("a slot is free again");
}

#[test]
fn slots_and_the_connection_are_released_on_success_error_and_panic() {
    let (_guard, fx) = widget_setup();
    let (pool, _log) = capturing_pool();
    // Success.
    read(&pool, &input_for(BOOK_BARE)).expect("served");
    assert_eq!(SLOTS_IN_USE.load(Ordering::SeqCst), 0);
    // An error after checkout.
    assert_eq!(
        read(&pool, &input_for("10.5555/widget-unknown")),
        Err(MetricWidgetError::Unavailable)
    );
    assert_eq!(SLOTS_IN_USE.load(Ordering::SeqCst), 0);
    // A panic inside the transaction.
    let panicked = catch_unwind(AssertUnwindSafe(|| {
        resolve_with(
            &pool,
            MetricWidgetSwitch::On,
            &input_for(BOOK_BARE),
            &profile(&|step| {
                if step == WidgetStep::Children {
                    panic!("injected");
                }
            }),
        )
    }));
    assert!(panicked.is_err());
    assert_eq!(SLOTS_IN_USE.load(Ordering::SeqCst), 0);
    // The one-connection pool serves the next read: the connection came back.
    read(&pool, &input_for(BOOK_BARE)).expect("the connection was released");
    drop(fx);
}

#[test]
fn a_checkout_timeout_is_busy() {
    let (_guard, _fx) = widget_setup();
    let (pool, _log) = capturing_pool();
    let held = pool.get().expect("the only connection");
    let mut quick = profile(&no_hook);
    quick.checkout_timeout = Duration::from_millis(100);
    let (result, lines) =
        resolve_with(&pool, MetricWidgetSwitch::On, &input_for(BOOK_BARE), &quick);
    assert_eq!(result, Err(MetricWidgetError::Busy));
    assert!(lines
        .iter()
        .any(|line| line.level == log::Level::Warn && line.kind == WidgetLineKind::Pool));
    drop(held);
}

#[test]
fn a_passed_deadline_at_a_widget_step_is_busy_and_runs_no_further_statement() {
    let (_guard, fx) = widget_setup();
    longest_path(&fx);
    let mut late = profile(&|step| {
        if step == WidgetStep::Pairs {
            std::thread::sleep(Duration::from_millis(1500));
        }
    });
    late.deadline = Duration::from_secs(1);
    let (result, lines, names) = captured(&input_for(BOOK_BARE), &late);
    assert_eq!(result, Err(MetricWidgetError::Busy));
    assert_eq!(
        names,
        [
            "CHECKOUT_CHECK",
            "BEGIN",
            "CUSTOM_PLANS_SQL",
            "METRIC_WIDGET_STATEMENT_TIMEOUT_SQL",
            "PROJECTION_LOCK_SQL",
            "METRIC_WIDGET_RESOLUTION_SQL",
            "ROLLBACK",
        ]
    );
    assert!(lines
        .iter()
        .any(|line| line.reason == Some(WidgetReason::DeadlinePassed)));
}

#[test]
fn a_passed_deadline_inside_the_engine_is_busy_through_the_deadline_flag() {
    let (_guard, fx) = widget_setup();
    longest_path(&fx);
    let mut late = profile(&|step| {
        if step == WidgetStep::Engine {
            std::thread::sleep(Duration::from_millis(1500));
        }
    });
    late.deadline = Duration::from_secs(1);
    let (result, lines, names) = captured(&input_for(BOOK_BARE), &late);
    assert_eq!(result, Err(MetricWidgetError::Busy));
    // The engine admitted nothing: no engine statement ran.
    assert_eq!(names.last(), Some(&"ROLLBACK"));
    assert!(names.contains(&"METRIC_WIDGET_LABELS_SQL"));
    for (name, _) in ENGINE_STATEMENTS {
        assert!(!names.contains(&name), "{name} ran after the deadline");
    }
    let reason = lines
        .iter()
        .find(|line| line.kind == WidgetLineKind::Reason)
        .expect("a reason line");
    assert_eq!(reason.reason, Some(WidgetReason::DeadlinePassed));
    assert_eq!(reason.step, WidgetStep::Engine);
}

/// A second connection holding `lock` in its own open transaction.
fn hold_lock(lock: &str) -> PgConnection {
    let mut connection = PgConnection::establish(&test_db_url()).expect("a lock holder");
    connection
        .batch_execute(&format!("BEGIN; {lock};"))
        .expect("the lock is taken");
    connection
}

#[test]
fn an_engine_statement_started_before_the_deadline_and_failing_after_it_is_internal() {
    let (_guard, fx) = widget_setup();
    longest_path(&fx);
    // ACCOUNTS_SQL, the engine's second statement, waits on this lock until
    // the test-only statement timeout fires after the deadline has passed.
    let mut holder = hold_lock("LOCK TABLE public.metric_source_account IN ACCESS EXCLUSIVE MODE");
    let mut slow = profile(&no_hook);
    slow.statement_timeout_sql = "SET LOCAL statement_timeout = '3s'";
    slow.deadline = Duration::from_secs(2);
    let started = Instant::now();
    let (result, lines, names) = captured(&input_for(BOOK_BARE), &slow);
    holder.batch_execute("ROLLBACK;").expect("release the lock");
    assert!(started.elapsed() >= Duration::from_secs(2));
    assert_eq!(result, Err(MetricWidgetError::Internal));
    assert_eq!(names[3], "TEST_STATEMENT_TIMEOUT_SQL");
    assert!(names.ends_with(&["PUBLISHERS_SQL", "ACCOUNTS_SQL", "ROLLBACK"]));
    let error = lines
        .iter()
        .find(|line| line.level == log::Level::Error)
        .expect("an error line");
    assert_eq!(error.reason, Some(WidgetReason::EngineUnavailable));
    assert_eq!(error.engine_section, Some(WidgetSection::Work));
    assert_eq!(error.step, WidgetStep::Engine);
    assert_eq!(error.last_statement, Some("ACCOUNTS_SQL"));
}

#[test]
fn a_widget_step_refusal_sets_no_engine_flag_and_an_engine_refusal_sets_it() {
    let (_guard, fx) = widget_setup();
    longest_path(&fx);
    let request = validate(&input_for(BOOK_BARE), today()).expect("valid");
    // Late at the first section's pairs statement (widget-owned), then at
    // its engine run, whose first check refuses.
    for (late_step, engine_refused) in [(WidgetStep::Pairs, false), (WidgetStep::Engine, true)] {
        let hook = move |step: WidgetStep| {
            if step == late_step {
                std::thread::sleep(Duration::from_millis(1500));
            }
        };
        let late = profile(&hook);
        let deadline = Deadline::new(Instant::now() + Duration::from_secs(1));
        let mut facts = fresh_facts();
        let mut connection = fx.pool.get().expect("a connection");
        let failure = connection
            .build_transaction()
            .read_only()
            .repeatable_read()
            .run(|connection| transact(connection, &request, &late, &deadline, &mut facts))
            .map(|_| ())
            .expect_err("busy");
        assert_eq!(
            failure,
            Failure::new(MetricWidgetError::Busy, WidgetReason::DeadlinePassed),
            "{late_step:?}"
        );
        assert_eq!(facts.step, late_step);
        // The engine's flag, and so B2's warn line, only for a refusal
        // inside the engine.
        assert_eq!(deadline.engine.expired(), engine_refused, "{late_step:?}");
        // A refused check records nothing, and no engine check had passed.
        assert_eq!(deadline.engine.last_statement(), None, "{late_step:?}");
        assert_eq!(
            deadline.last_statement(),
            Some(if engine_refused {
                "METRIC_WIDGET_LABELS_SQL"
            } else {
                "METRIC_WIDGET_RESOLUTION_SQL"
            }),
            "{late_step:?}"
        );
    }
}

/// The facts a resolution starts from, as [`resolve`] builds them.
fn fresh_facts() -> WidgetFacts {
    WidgetFacts {
        switch: MetricWidgetSwitch::On,
        previous_claims: Some(0),
        error: None,
        reason: None,
        step: WidgetStep::Switch,
        elapsed_ms: 0,
        doi: None,
        work: None,
        chapters: None,
        grain: None,
        history: None,
        buckets: None,
        engine_section: None,
        last_statement: None,
    }
}

#[test]
fn a_deadline_passing_between_engine_statements_is_busy_before_the_next_one() {
    let (_guard, fx) = widget_setup();
    longest_path(&fx);
    // ACCOUNTS_SQL, the engine's second statement, is admitted before the
    // deadline and waits on this lock, which is released after the deadline
    // but well inside the production statement timeout (5 s). The statement
    // completes; the engine's next admission is refused. The capturing pool
    // connects before the lock is taken, so establishing it cannot wait on
    // the lock.
    let (pool, log) = capturing_pool();
    let mut holder = hold_lock("LOCK TABLE public.metric_source_account IN ACCESS EXCLUSIVE MODE");
    let release = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(2));
        holder.batch_execute("ROLLBACK;").expect("release the lock");
    });
    let mut late = profile(&no_hook);
    late.deadline = Duration::from_secs(1);
    // The read itself, not `resolve`, so the facts it records are visible:
    // the debug reason line carries no last statement (section 4.8).
    let mut facts = fresh_facts();
    let started = Instant::now();
    let result = super::read(&pool, &input_for(BOOK_BARE), &late, &mut facts);
    let elapsed = started.elapsed();
    release.join().expect("the lock holder");
    let names = widget_statement_names(&log.lock().expect("the statement log"));

    let failure = result.map(|_| ()).expect_err("busy");
    assert_eq!(
        failure,
        Failure::new(MetricWidgetError::Busy, WidgetReason::DeadlinePassed)
    );
    assert_eq!(failure.error.code(), "METRIC_WIDGET_BUSY");
    assert!(elapsed >= Duration::from_secs(1), "{elapsed:?}");
    assert!(elapsed < Duration::from_secs(5), "{elapsed:?}");
    assert_eq!(names[3], "METRIC_WIDGET_STATEMENT_TIMEOUT_SQL");
    assert!(
        names.ends_with(&["PUBLISHERS_SQL", "ACCOUNTS_SQL", "ROLLBACK"]),
        "{names:?}"
    );
    assert!(!names.contains(&"CANONICAL_SQL"), "{names:?}");
    assert_eq!(
        names
            .iter()
            .filter(|name| **name == "PUBLISHERS_SQL")
            .count(),
        1
    );
    assert_eq!(facts.step, WidgetStep::Engine);
    assert_eq!(facts.engine_section, Some(WidgetSection::Work));
    assert_eq!(facts.last_statement, Some("ACCOUNTS_SQL"));
}

#[test]
fn a_rebuild_holding_the_projections_makes_the_read_busy_never_zero() {
    let (_guard, fx) = widget_setup();
    longest_path(&fx);
    let mut holder =
        hold_lock("LOCK TABLE public.metric_rollup_work_month IN ACCESS EXCLUSIVE MODE");
    let mut short = profile(&no_hook);
    short.statement_timeout_sql = "SET LOCAL statement_timeout = '300ms'";
    let (result, lines, names) = captured(&input_for(BOOK_BARE), &short);
    holder.batch_execute("ROLLBACK;").expect("release the lock");
    assert_eq!(result, Err(MetricWidgetError::Busy));
    assert!(lines
        .iter()
        .any(|line| line.reason == Some(WidgetReason::LockFailed)));
    assert!(!names.contains(&"METRIC_WIDGET_RESOLUTION_SQL"));
    // Without the holder the same read is served, with its values.
    let widget = read(&fx.pool, &input_for(BOOK_BARE)).expect("served");
    assert_eq!(
        total(&widget.work, fx.platform_id, fx.sessions),
        Some("4".to_string())
    );
}

// ==========================================================================
// Public scope (section 4.2)
// ==========================================================================

#[test]
fn only_public_enabled_additive_measures_on_enabled_platforms_are_served() {
    let (_guard, fx) = widget_setup();
    let book = fx.works[0];
    cover_march(&fx, fx.units);
    // A public SALES measure, with a negative value.
    commit(&fx, book, fx.platform_id, fx.units, day_n(2), -5);
    commit(&fx, book, fx.platform_id, fx.units, day_n(3), 2);
    // A non-public USAGE measure and a non-public SALES measure.
    let hidden = insert_measure(&fx.pool, "widget_hidden_usage", true, true);
    let hidden_sales = insert_measure(&fx.pool, "widget_hidden_sales", true, true);
    exec(
        &fx.pool,
        &format!(
            "UPDATE metric_measure SET public_visibility = FALSE \
              WHERE measure_id IN ('{hidden}', '{hidden_sales}'); \
             UPDATE metric_measure SET category = 'SALES' WHERE measure_id = '{hidden_sales}';"
        ),
    );
    // Non-additive measures.
    let not_across_time = insert_measure(&fx.pool, "widget_not_across_time", false, true);
    let not_across_works = insert_measure(&fx.pool, "widget_not_across_works", true, false);
    // A measure disabled after its data was committed.
    let disabled = insert_measure(&fx.pool, "widget_disabled", true, true);
    for measure in [
        hidden,
        hidden_sales,
        not_across_time,
        not_across_works,
        disabled,
    ] {
        commit(&fx, book, fx.platform_id, measure, day_n(4), 9);
    }
    // A platform disabled after its data was committed.
    commit(&fx, book, fx.other_platform_id, fx.sessions, day_n(4), 9);
    apply_all(&fx.pool);
    exec(
        &fx.pool,
        &format!(
            "UPDATE metric_measure SET enabled = FALSE WHERE measure_id = '{disabled}'; \
             UPDATE metric_platform SET enabled = FALSE \
              WHERE platform_id = '{}';",
            fx.other_platform_id
        ),
    );

    let widget = read(&fx.pool, &input_for(BOOK_BARE)).expect("ineligible data never fails");
    assert_eq!(pairs_of(&widget.work), [(fx.platform_id, fx.units)]);
    assert_eq!(
        total(&widget.work, fx.platform_id, fx.units),
        Some("-3".to_string())
    );
    assert_eq!(
        widget
            .measures
            .iter()
            .map(|measure| measure.code.as_str())
            .collect::<Vec<_>>(),
        ["net_units"]
    );
    assert_eq!(widget.platforms.len(), 1);
    assert_eq!(widget.platforms[0].platform_id, fx.platform_id);
}

#[test]
fn no_eligible_measure_gives_empty_sections_and_no_engine_statement() {
    let (_guard, fx) = widget_setup();
    let hidden = insert_measure(&fx.pool, "widget_hidden_only", true, true);
    exec(
        &fx.pool,
        &format!(
            "UPDATE metric_measure SET public_visibility = FALSE WHERE measure_id = '{hidden}';"
        ),
    );
    commit(&fx, fx.works[0], fx.platform_id, hidden, day_n(2), 5);
    apply_all(&fx.pool);
    let (result, _, names) = captured(&input_for(BOOK_BARE), &profile(&no_hook));
    let widget = result.expect("served, empty");
    assert_eq!(widget.work, empty_section());
    assert!(widget.platforms.is_empty() && widget.measures.is_empty());
    for (name, _) in ENGINE_STATEMENTS {
        assert!(!names.contains(&name), "{name} ran with nothing to serve");
    }
}

// ==========================================================================
// Empty sections (section 4.6)
// ==========================================================================

#[test]
fn empty_sections_are_partial_with_nothing_served_and_a_frontier() {
    let (_guard, fx) = widget_setup();
    let imprint = imprint_of(&fx.pool, fx.works[0]);
    // A book with no rows in range and no children.
    let widget = read(&fx.pool, &input_for(BOOK_BARE)).expect("served");
    assert_eq!(widget.work, empty_section());
    assert!(widget.work.is_partial);
    assert_eq!(widget.chapters, None);

    // Both sections empty: the frontier still comes from the labels.
    add_chapter(&fx.pool, fx.works[0], imprint, None, 1);
    let (result, _, names) = captured(&input_for(BOOK_BARE), &profile(&no_hook));
    let widget = result.expect("served");
    assert_eq!(widget.work, empty_section());
    assert_eq!(widget.chapters, Some(empty_section()));
    assert!(names.contains(&"METRIC_WIDGET_LABELS_SQL"));
    for (name, _) in ENGINE_STATEMENTS {
        assert!(!names.contains(&name));
    }
    let watermark = state_watermark(&fx.pool);
    assert_eq!(widget.rollup_watermark, watermark);

    // Rows for the book, none for its chapters.
    served_fixture(&fx);
    let widget = read(&fx.pool, &input_for(BOOK_BARE)).expect("served");
    assert_eq!(pairs_of(&widget.work), [(fx.platform_id, fx.sessions)]);
    assert_eq!(widget.chapters, Some(empty_section()));
}

fn state_watermark(pool: &PgPool) -> Timestamp {
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = Timestamptz)]
        watermark_at: Timestamp,
    }
    let mut connection = pool.get().expect("a connection");
    diesel::sql_query("SELECT watermark_at FROM metric_rollup_work_day_state WHERE state_id = 1")
        .get_result::<Row>(&mut connection)
        .expect("the frontier")
        .watermark_at
}

// ==========================================================================
// Work and child works (section 4.3)
// ==========================================================================

#[test]
fn a_chapter_is_served_alone_and_refuses_chapter_dois() {
    let (_guard, fx) = widget_setup();
    let imprint = imprint_of(&fx.pool, fx.works[0]);
    let chapter = add_chapter(
        &fx.pool,
        fx.works[0],
        imprint,
        Some("https://doi.org/10.5555/widget-chapter"),
        1,
    );
    // A chapter's own has-child rows are ignored.
    add_chapter(&fx.pool, chapter, imprint, None, 1);
    cover_march(&fx, fx.sessions);
    commit(&fx, chapter, fx.platform_id, fx.sessions, day_n(2), 6);
    apply_all(&fx.pool);

    let widget = read(&fx.pool, &input_for("10.5555/widget-chapter")).expect("served");
    assert_eq!(widget.chapters, None);
    assert_eq!(widget.excluded_child_count, 0);
    assert_eq!(
        total(&widget.work, fx.platform_id, fx.sessions),
        Some("6".to_string())
    );
    assert_eq!(
        read(
            &fx.pool,
            &with_chapters("10.5555/widget-chapter", &["10.5555/widget-chapter"])
        ),
        Err(MetricWidgetError::QueryInvalid(NOT_A_CHILD_MESSAGE))
    );
    assert!(read(&fx.pool, &with_chapters("10.5555/widget-chapter", &[])).is_ok());
}

fn without_as_of(mut widget: MetricWidget) -> MetricWidget {
    widget.as_of = widget.rollup_watermark;
    widget
}

#[test]
fn child_selection_exclusion_and_counting_follow_section_4_3() {
    let (_guard, fx) = widget_setup();
    let book = fx.works[0];
    let imprint = imprint_of(&fx.pool, book);
    let foreign_imprint = imprint_of(&fx.pool, fx.other_work);
    let own_a = add_chapter(
        &fx.pool,
        book,
        imprint,
        Some("https://doi.org/10.5555/own-a"),
        1,
    );
    let own_b = add_chapter(
        &fx.pool,
        book,
        imprint,
        Some("https://doi.org/10.5555/own-b"),
        2,
    );
    let own_no_doi = add_chapter(&fx.pool, book, imprint, None, 3);
    add_chapter(
        &fx.pool,
        book,
        foreign_imprint,
        Some("https://doi.org/10.5555/foreign"),
        4,
    );
    add_chapter(&fx.pool, book, foreign_imprint, None, 5);
    cover_march(&fx, fx.sessions);
    for (work, value) in [(own_a, 1), (own_b, 10), (own_no_doi, 100)] {
        commit(&fx, work, fx.platform_id, fx.sessions, day_n(2), value);
    }
    apply_all(&fx.pool);
    let chapters_total = |widget: &MetricWidget| {
        widget
            .chapters
            .as_ref()
            .and_then(|section| total(section, fx.platform_id, fx.sessions))
    };

    // Omitted and [] select every child: both foreign children are counted.
    let omitted = read(&fx.pool, &input_for(BOOK_BARE)).expect("served");
    let empty = read(&fx.pool, &with_chapters(BOOK_BARE, &[])).expect("served");
    assert_eq!(without_as_of(omitted.clone()), without_as_of(empty));
    assert_eq!(omitted.excluded_child_count, 2);
    assert_eq!(chapters_total(&omitted), Some("111".to_string()));

    // An explicit list of every DOI-bearing child drops the DOI-less ones,
    // so the count and the values differ from omitted.
    let explicit = read(
        &fx.pool,
        &with_chapters(
            BOOK_BARE,
            &[
                "10.5555/own-a",
                "10.5555/OWN-B",
                "https://doi.org/10.5555/foreign",
            ],
        ),
    )
    .expect("served");
    assert_eq!(explicit.excluded_child_count, 1);
    assert_eq!(chapters_total(&explicit), Some("11".to_string()));

    // An unselected foreign child is not counted.
    let one = read(&fx.pool, &with_chapters(BOOK_BARE, &["10.5555/own-a"])).expect("served");
    assert_eq!(one.excluded_child_count, 0);
    assert_eq!(chapters_total(&one), Some("1".to_string()));

    // Every selected child excluded: chapters is null.
    let foreign_only =
        read(&fx.pool, &with_chapters(BOOK_BARE, &["10.5555/foreign"])).expect("served");
    assert_eq!(foreign_only.excluded_child_count, 1);
    assert_eq!(foreign_only.chapters, None);

    // A DOI that is not a child, even an existing work's.
    assert_eq!(
        read(
            &fx.pool,
            &with_chapters(BOOK_BARE, &["10.5555/widget-book"])
        ),
        Err(MetricWidgetError::QueryInvalid(NOT_A_CHILD_MESSAGE))
    );
    // A case-insensitive duplicate.
    assert_eq!(
        read(
            &fx.pool,
            &with_chapters(BOOK_BARE, &["10.5555/own-a", "10.5555/OWN-A"])
        ),
        Err(MetricWidgetError::QueryInvalid(
            DUPLICATE_CHAPTER_DOI_MESSAGE
        ))
    );
}

#[test]
fn five_hundred_child_works_are_served_and_five_hundred_and_one_refused_whole() {
    let (_guard, fx) = widget_setup();
    let imprint = imprint_of(&fx.pool, fx.works[0]);
    add_chapters(&fx.pool, fx.works[0], imprint, 500);
    let widget = read(&fx.pool, &input_for(BOOK_BARE)).expect("500 children are served");
    assert_eq!(widget.chapters, Some(empty_section()));

    let _extra = add_chapter(
        &fx.pool,
        fx.works[0],
        imprint,
        Some("https://doi.org/10.5555/child-501"),
        501,
    );
    let refused = Err(MetricWidgetError::QueryLimitExceeded(
        TOO_MANY_CHILDREN_MESSAGE,
    ));
    assert_eq!(read(&fx.pool, &input_for(BOOK_BARE)), refused);
    // Whatever chapterDois selects, a non-child entry included: the bound is
    // checked before the entries (step 10 order).
    assert_eq!(
        read(&fx.pool, &with_chapters(BOOK_BARE, &["10.5555/child-501"])),
        refused
    );
    assert_eq!(
        read(
            &fx.pool,
            &with_chapters(BOOK_BARE, &["10.5555/not-a-child"])
        ),
        refused
    );
}

#[test]
fn a_repeated_child_row_is_internal_never_deduplicated() {
    // No database: step 10 runs over the resolution rows alone.
    let request = validate(&input_for(BOOK_BARE), today()).expect("valid");
    let (work_id, publisher_id) = (Uuid::new_v4(), Uuid::new_v4());
    let row = |child_id: Uuid| ResolutionRow {
        work_id,
        work_type: "monograph".to_string(),
        publisher_id,
        subscription_package: ThothPackage::Sphinx,
        child_id: Some(child_id),
        child_doi: None,
        child_publisher_id: Some(publisher_id),
    };
    let served = |rows: &[ResolutionRow]| {
        select_children(&request, rows, false, publisher_id).map(|children| children.served)
    };
    let (first, second) = (Uuid::new_v4(), Uuid::new_v4());
    assert_eq!(
        served(&[row(first), row(second)]),
        Ok(sorted(&[first, second]))
    );
    // The bound never rests on deduplication: a child returned twice is a
    // row the resolution cannot produce, so the read fails.
    let failure = served(&[row(first), row(first)]).expect_err("a repeated child");
    assert_eq!(
        failure,
        Failure::new(MetricWidgetError::Internal, WidgetReason::UnexpectedRow)
    );
    assert_eq!(failure.error.code(), "INTERNAL_ERROR");
}

#[test]
fn chapter_dois_on_a_chapter_is_refused_before_any_other_child_check() {
    let (_guard, fx) = widget_setup();
    let imprint = imprint_of(&fx.pool, fx.works[0]);
    let chapter = add_chapter(
        &fx.pool,
        fx.works[0],
        imprint,
        Some("https://doi.org/10.5555/many-chapter"),
        1,
    );
    // The chapter has 501 has-child rows of its own; they are not counted.
    add_chapters(&fx.pool, chapter, imprint, 501);
    assert_eq!(
        read(
            &fx.pool,
            &with_chapters("10.5555/many-chapter", &["10.5555/whatever"])
        ),
        Err(MetricWidgetError::QueryInvalid(NOT_A_CHILD_MESSAGE))
    );
    assert!(read(&fx.pool, &input_for("10.5555/many-chapter")).is_ok());
}

// ==========================================================================
// Served pairs and bounds (section 4.4)
// ==========================================================================

#[test]
fn a_pair_with_only_coverage_is_not_served() {
    let (_guard, fx) = widget_setup();
    cover_march(&fx, fx.sessions);
    cover_march(&fx, fx.units);
    commit(&fx, fx.works[0], fx.platform_id, fx.units, day_n(2), 1);
    apply_all(&fx.pool);
    let widget = read(&fx.pool, &input_for(BOOK_BARE)).expect("served");
    assert_eq!(pairs_of(&widget.work), [(fx.platform_id, fx.units)]);
}

#[test]
fn forty_pairs_are_served_and_forty_one_refused_never_truncated() {
    let (_guard, fx) = widget_setup();
    // 40 pairs on the fixture platform: the two seeded measures and 38 more.
    let mut measures = vec![fx.sessions, fx.units];
    for n in 0..38 {
        measures.push(insert_measure(
            &fx.pool,
            &format!("widget_bound_{n}"),
            true,
            true,
        ));
    }
    for measure in &measures {
        commit(&fx, fx.works[0], fx.platform_id, *measure, day_n(2), 1);
    }
    apply_all(&fx.pool);
    let widget = read(&fx.pool, &input_for(BOOK_BARE)).expect("40 pairs are served");
    assert_eq!(widget.work.totals.len(), 40);
    assert_eq!(widget.measures.len(), 40);

    commit(
        &fx,
        fx.works[0],
        fx.other_platform_id,
        fx.sessions,
        day_n(2),
        1,
    );
    apply_all(&fx.pool);
    assert_eq!(
        read(&fx.pool, &input_for(BOOK_BARE)),
        Err(MetricWidgetError::QueryLimitExceeded(
            TOO_MANY_PAIRS_MESSAGE
        ))
    );
}

#[test]
fn twelve_months_are_served_and_output_is_ordered() {
    let (_guard, fx) = widget_setup();
    let imprint = imprint_of(&fx.pool, fx.works[0]);
    let chapter = add_chapter(&fx.pool, fx.works[0], imprint, None, 1);
    for (platform, measure) in [
        (fx.platform_id, fx.sessions),
        (fx.platform_id, fx.units),
        (fx.other_platform_id, fx.sessions),
        (fx.other_platform_id, fx.units),
    ] {
        for (day, country) in [(day_n(2), "GB"), (day_n(3), "US"), (d(2026, 5, 4), "DE")] {
            for work in [fx.works[0], chapter] {
                commit(&fx, work, platform, measure, day, 2);
                commit_dims(
                    &fx,
                    work,
                    platform,
                    measure,
                    day,
                    Dims {
                        country: Some(country),
                        ..Dims::default()
                    },
                    2,
                );
            }
        }
    }
    apply_all(&fx.pool);
    let mut input = input_for(BOOK_BARE);
    input.start_date = Some(d(2026, 1, 1));
    input.end_date = d(2027, 1, 1);
    let widget = read(&fx.pool, &input).expect("12 months are served");
    let platform_ids: Vec<Uuid> = widget.platforms.iter().map(|p| p.platform_id).collect();
    assert_eq!(platform_ids, sorted(&platform_ids));
    let measure_ids: Vec<Uuid> = widget.measures.iter().map(|m| m.measure_id).collect();
    assert_eq!(measure_ids, sorted(&measure_ids));
    for section in [&widget.work, widget.chapters.as_ref().expect("chapters")] {
        let totals: Vec<(Uuid, Uuid)> = section
            .totals
            .iter()
            .map(|total| (total.platform_id, total.measure_id))
            .collect();
        assert_eq!(totals.len(), 4);
        assert_eq!(totals, sorted(&totals));
        assert_eq!(pairs_of(section), totals);
        let timeline: Vec<(Uuid, Uuid, NaiveDate)> = section
            .timeline
            .iter()
            .map(|bucket| (bucket.platform_id, bucket.measure_id, bucket.start_date))
            .collect();
        assert_eq!(timeline.len(), 4 * 12);
        assert_eq!(timeline, sorted(&timeline));
        let countries: Vec<(Uuid, Uuid, String)> = section
            .countries
            .iter()
            .map(|row| (row.platform_id, row.measure_id, row.country_code.clone()))
            .collect();
        assert_eq!(countries.len(), 4 * 3);
        assert_eq!(countries, sorted(&countries));
    }
    input.end_date = d(2027, 2, 1);
    assert_eq!(
        read(&fx.pool, &input),
        Err(MetricWidgetError::QueryLimitExceeded(RANGE_MESSAGE))
    );
}

// ==========================================================================
// Capability parameter
// ==========================================================================

#[test]
fn the_pair_entry_checks_the_capability_it_is_given() {
    let (_guard, fx) = widget_setup();
    let obelisk = Uuid::new_v4();
    let work = Uuid::new_v4();
    exec(
        &fx.pool,
        &format!(
            "{}{}",
            publisher_sql(obelisk, "OBELISK"),
            works_sql(obelisk, &[work])
        ),
    );
    let (start, end) = march();
    let pairs = [(fx.platform_id, fx.sessions)];
    let run = |publisher: Uuid, works: &[Uuid], capability: PublisherCapability| {
        let mut connection = fx.pool.get().expect("a connection");
        connection
            .build_transaction()
            .read_only()
            .repeatable_read()
            .run(|connection| {
                Ok::<_, diesel::result::Error>(metric_pair_read(
                    connection,
                    &MetricPairRead::new(publisher, works, &pairs, start, end)
                        .with_timeline_grain(MetricTimelineGrain::Month)
                        .with_capability(capability),
                    &ReadDeadline::new(Instant::now() + METRIC_WIDGET_DEADLINE),
                ))
            })
            .expect("a transaction")
    };
    // No package separates METRICS_WIDGET from METRICS_DASHBOARD, so an
    // OBELISK publisher shows the parameter is what is checked.
    assert!(run(obelisk, &[work], PublisherCapability::MetricsCollect).is_ok());
    assert_eq!(
        run(obelisk, &[work], PublisherCapability::MetricsWidget).map(|_| ()),
        Err(MetricReadError::Unauthorised)
    );
    assert!(run(
        fx.publisher_id,
        &[fx.works[0]],
        PublisherCapability::MetricsWidget
    )
    .is_ok());
}

#[test]
fn the_widget_checks_metrics_widget_at_step_9_and_passes_it_to_the_engine() {
    let source = include_str!("mod.rs");
    let source = source
        .split("#[cfg(test)]\nmod tests;")
        .next()
        .expect("the module source");
    assert_eq!(
        source.matches("PublisherCapability::MetricsWidget").count(),
        2
    );
    assert!(source.contains(".has_capability(PublisherCapability::MetricsWidget)"));
    assert!(source.contains(".with_capability(PublisherCapability::MetricsWidget)"));
    assert!(!source.contains("PublisherCapability::MetricsDashboard"));
    // The widget never reaches the engine's dashboard entry points or its
    // statement timeout, and has no exhaustive match on an engine enum.
    for absent in [
        "metric_dashboard(",
        "metric_dashboard_with_statement_timeout",
        " STATEMENT_TIMEOUT_SQL",
        "(STATEMENT_TIMEOUT_SQL",
        "MetricReadError::RegistryLimitExceeded",
        "MetricReadError::QueryInvalid",
        "MetricTimelineGrain::Day",
        "MetricTimelineGrain::Auto",
        "Work::from_doi",
        "db.get()",
    ] {
        assert!(!source.contains(absent), "{absent}");
    }
}

#[test]
fn the_widget_names_neither_the_dashboard_entry_nor_its_guard_nor_its_deadline() {
    let source = include_str!("mod.rs");
    let source = source
        .split("#[cfg(test)]\nmod tests;")
        .next()
        .expect("the module source");
    for absent in [
        "metric_dashboard(",
        "metric_dashboard_with_controls",
        "ReadGuard",
        "ReadControls",
        "METRIC_DASHBOARD_READS",
        "METRIC_DASHBOARD_MAX_CONCURRENT_READS",
        "METRIC_DASHBOARD_SLOT_WAIT",
        "METRIC_DASHBOARD_READ_DEADLINE",
    ] {
        assert!(!source.contains(absent), "{absent}");
    }
    // Its one engine deadline value is created with the widget's own instant.
    assert_eq!(source.matches("ReadDeadline::new(").count(), 1);
    assert!(source.contains("Deadline::new(Instant::now() + profile.deadline)"));
}

// ==========================================================================
// Pair mode (section 4.5)
// ==========================================================================

/// Served pairs (platform, sessions) and (other platform, units).
fn diagonal(fx: &Fixture) {
    commit(fx, fx.works[0], fx.platform_id, fx.sessions, day_n(2), 3);
    commit(fx, fx.works[0], fx.other_platform_id, fx.units, day_n(2), 5);
    apply_all(&fx.pool);
}

#[test]
fn a_native_record_on_a_served_cross_pair_makes_the_widget_unavailable() {
    let (_guard, fx) = widget_setup();
    diagonal(&fx);
    read(&fx.pool, &input_for(BOOK_BARE)).expect("served before the native record");
    native_record(&fx, fx.works[0], fx.platform_id, fx.units);
    let (result, lines) = resolve_with(
        &fx.pool,
        MetricWidgetSwitch::On,
        &input_for(BOOK_BARE),
        &profile(&no_hook),
    );
    assert_eq!(result, Err(MetricWidgetError::Unavailable));
    assert!(lines
        .iter()
        .any(|line| line.reason == Some(WidgetReason::UnsupportedSourceGrain)));
}

#[test]
fn a_native_only_pair_off_the_served_platforms_is_absent() {
    let (_guard, fx) = widget_setup();
    commit(&fx, fx.works[0], fx.platform_id, fx.sessions, day_n(2), 3);
    apply_all(&fx.pool);
    native_record(&fx, fx.works[0], fx.other_platform_id, fx.sessions);
    let widget = read(&fx.pool, &input_for(BOOK_BARE)).expect("served");
    assert_eq!(pairs_of(&widget.work), [(fx.platform_id, fx.sessions)]);
}

#[test]
fn a_total_ambiguous_pair_is_served_counted_and_makes_the_widget_unavailable() {
    let (_guard, fx) = widget_setup();
    let institution = insert_institution(&fx.pool);
    for dims in [
        Dims {
            country: Some("GB"),
            ..Dims::default()
        },
        Dims {
            institution: Some(institution),
            ..Dims::default()
        },
    ] {
        commit_dims(
            &fx,
            fx.works[0],
            fx.platform_id,
            fx.sessions,
            day_n(2),
            dims,
            1,
        );
    }
    apply_all(&fx.pool);
    // Discovery serves it (and so counts it toward the bound).
    let request = validate(&input_for(BOOK_BARE), today()).expect("valid");
    let mut connection = fx.pool.get().expect("a connection");
    let deadline = Deadline::new(Instant::now() + METRIC_WIDGET_DEADLINE);
    let pairs =
        served_pairs(&mut connection, &[fx.works[0]], &request, &deadline).expect("discovery");
    assert_eq!(pairs, [(fx.platform_id, fx.sessions)]);
    drop(connection);
    let (result, lines) = resolve_with(
        &fx.pool,
        MetricWidgetSwitch::On,
        &input_for(BOOK_BARE),
        &profile(&no_hook),
    );
    assert_eq!(result, Err(MetricWidgetError::Unavailable));
    assert!(lines
        .iter()
        .any(|line| line.reason == Some(WidgetReason::DimensionScopeAmbiguous)));
}

#[test]
fn a_country_row_on_an_unserved_cross_pair_is_not_served() {
    let (_guard, fx) = widget_setup();
    diagonal(&fx);
    let (start, _) = march();
    exec(
        &fx.pool,
        &format!(
            "INSERT INTO metric_rollup_work_country_month \
                 (rollup_work_country_month_id, work_id, publication_id, platform_id, \
                  measure_id, month_start, country_code, value, \
                  requires_institution_coverage, watermark) \
             VALUES ('{}', '{}', NULL, '{}', '{}', '{start}', 'FR', 7, FALSE, 1);",
            Uuid::new_v4(),
            fx.works[0],
            fx.platform_id,
            fx.units,
        ),
    );
    let widget = read(&fx.pool, &input_for(BOOK_BARE)).expect("served");
    assert!(widget
        .work
        .countries
        .iter()
        .all(|row| (row.platform_id, row.measure_id) != (fx.platform_id, fx.units)));
    assert_eq!(
        sorted(&pairs_of(&widget.work)),
        sorted(&[
            (fx.platform_id, fx.sessions),
            (fx.other_platform_id, fx.units)
        ])
    );
}

/// Two coverage runs of one stream of the fixture account, on (`platform`,
/// `measure`), overlapping from 10 to 15 March: invalid derived state, written
/// last and directly so nothing re-derives it.
fn overlapping_march_runs(fx: &Fixture, platform_id: Uuid, measure_id: Uuid) {
    exec_raw(
        &fx.pool,
        &format!(
            "INSERT INTO metric_coverage_run \
                 (source_account_id, platform_id, publisher_id, measure_id, run_start, run_end, \
                  coverage_status, import_status, country_coverage, institution_coverage) \
             VALUES ('{account}', '{platform_id}', '{publisher}', '{measure_id}', '2026-03-01', \
                     '2026-03-15', 'COMPLETE', 'COMPLETED', TRUE, TRUE), \
                    ('{account}', '{platform_id}', '{publisher}', '{measure_id}', '2026-03-10', \
                     '2026-04-01', 'COMPLETE', 'COMPLETED', TRUE, TRUE);",
            account = fx.account_id,
            publisher = fx.publisher_id,
        ),
    );
}

#[test]
fn overlapping_runs_on_an_unserved_cross_pair_do_not_refuse_the_section() {
    let (_guard, fx) = widget_setup();
    diagonal(&fx);
    overlapping_march_runs(&fx, fx.platform_id, fx.units);
    // The state is live: the cross pair's own single-pair read refuses it.
    let cross = single_pair_input(&fx, &[fx.works[0]], fx.platform_id, fx.units, march());
    assert_eq!(
        metric_dashboard(&fx.pool, &cross).map(|_| ()),
        Err(MetricReadError::Unavailable)
    );

    let widget = read(&fx.pool, &input_for(BOOK_BARE)).expect("served");
    assert_eq!(
        pairs_of(&widget.work),
        sorted(&[
            (fx.platform_id, fx.sessions),
            (fx.other_platform_id, fx.units)
        ])
    );
    assert_each_served_pair_equals_its_dashboard_read(&fx, &widget.work, &[fx.works[0]]);
}

#[test]
fn overlapping_runs_on_a_served_pair_are_internal_as_the_dashboard_refuses_them() {
    let (_guard, fx) = widget_setup();
    diagonal(&fx);
    overlapping_march_runs(&fx, fx.platform_id, fx.sessions);
    // B2's rule, kept: the single-pair read of that pair refuses it, while
    // the other served pair's own read is served.
    let refused = single_pair_input(&fx, &[fx.works[0]], fx.platform_id, fx.sessions, march());
    assert_eq!(
        metric_dashboard(&fx.pool, &refused).map(|_| ()),
        Err(MetricReadError::Unavailable)
    );
    single_pair_read(&fx, &[fx.works[0]], fx.other_platform_id, fx.units);

    let (result, lines) = resolve_with(
        &fx.pool,
        MetricWidgetSwitch::On,
        &input_for(BOOK_BARE),
        &profile(&no_hook),
    );
    let error = result.map(|_| ()).expect_err("refused");
    assert_eq!(error, MetricWidgetError::Internal);
    assert_eq!(error.code(), "INTERNAL_ERROR");
    // No engine message reaches the caller.
    assert_eq!(error.message(), INTERNAL_MESSAGE);
    assert!(!engine_messages().contains(&error.message()));
    let line = lines
        .iter()
        .find(|line| line.level == log::Level::Error)
        .expect("an error line");
    assert_eq!(line.reason, Some(WidgetReason::EngineUnavailable));
    assert_eq!(line.engine_section, Some(WidgetSection::Work));
    assert_eq!(line.step, WidgetStep::Engine);
    // B2 refuses the overlap once every statement of the run has run.
    assert_eq!(line.last_statement, Some("IDENTIFIER_QUALITY_SQL"));
}

// ==========================================================================
// Equivalence oracle and snapshot
// ==========================================================================

#[test]
fn each_served_pair_equals_its_single_pair_dashboard_read() {
    let (_guard, fx) = widget_setup();
    let imprint = imprint_of(&fx.pool, fx.works[0]);
    let first = add_chapter(&fx.pool, fx.works[0], imprint, None, 1);
    let second = add_chapter(&fx.pool, fx.works[0], imprint, None, 2);
    cover_march(&fx, fx.sessions);
    let gb = Dims {
        country: Some("GB"),
        ..Dims::default()
    };
    // The book: sessions with a country breakdown, and units partly covered.
    commit(&fx, fx.works[0], fx.platform_id, fx.sessions, day_n(2), 4);
    commit_dims(
        &fx,
        fx.works[0],
        fx.platform_id,
        fx.sessions,
        day_n(2),
        gb,
        4,
    );
    commit(&fx, fx.works[0], fx.platform_id, fx.units, day_n(9), -2);
    cover(
        &fx,
        fx.units,
        d(2026, 3, 1),
        d(2026, 3, 15),
        "COMPLETE",
        "COMPLETED",
        "'2026-04-10T00:00:00Z'",
    );
    // The chapters: sessions on both platforms, one unapplied delta.
    commit(&fx, first, fx.platform_id, fx.sessions, day_n(3), 7);
    commit(&fx, second, fx.other_platform_id, fx.sessions, day_n(4), 11);
    apply_all(&fx.pool);
    commit(&fx, second, fx.platform_id, fx.sessions, day_n(5), 13);

    let (result, _, names) = captured(&input_for(BOOK_BARE), &profile(&no_hook));
    let widget = result.expect("served");
    // One connection, one transaction: the snapshot.
    assert_eq!(transactions(&names), 1);
    assert_eq!(widget.rollup_watermark, state_watermark(&fx.pool));

    let sections = [
        (&widget.work, vec![fx.works[0]]),
        (
            widget.chapters.as_ref().expect("chapters"),
            vec![first, second],
        ),
    ];
    for (section, works) in sections {
        assert_each_served_pair_equals_its_dashboard_read(&fx, section, &works);
    }
}

#[test]
fn two_eligible_accounts_on_a_served_platform_are_served_and_each_pair_equals_its_dashboard_read() {
    let (_guard, fx) = widget_setup();
    let imprint = imprint_of(&fx.pool, fx.works[0]);
    let chapter = add_chapter(&fx.pool, fx.works[0], imprint, None, 1);
    // A second eligible account of the work's publisher on the served
    // platform, beside the fixture's, asserting only the first half of
    // March for sessions and all of it for units.
    let (source_id, second) = (Uuid::new_v4(), Uuid::new_v4());
    exec(
        &fx.pool,
        &format!(
            "{}{}",
            source_sql(source_id, "DRIVER", true),
            account_sql(
                second,
                source_id,
                fx.platform_id,
                Some(fx.publisher_id),
                true
            )
        ),
    );
    cover_march(&fx, fx.sessions);
    cover_march(&fx, fx.units);
    for (measure, end) in [(fx.sessions, d(2026, 3, 16)), (fx.units, d(2026, 4, 1))] {
        let import_id = Uuid::new_v4();
        exec(
            &fx.pool,
            &format!(
                "{}{}",
                import_sql(import_id, second, "COMPLETED", "'2026-04-10T00:00:00Z'"),
                coverage_sql(
                    second,
                    import_id,
                    fx.platform_id,
                    measure,
                    d(2026, 3, 1),
                    end,
                    "COMPLETE",
                    true,
                    true
                )
            ),
        );
    }
    commit(&fx, fx.works[0], fx.platform_id, fx.sessions, day_n(2), 4);
    commit(&fx, fx.works[0], fx.platform_id, fx.units, day_n(3), 6);
    commit(&fx, chapter, fx.platform_id, fx.sessions, day_n(20), 7);
    apply_all(&fx.pool);

    let widget = read(&fx.pool, &input_for(BOOK_BARE)).expect("served, not refused");
    assert_eq!(
        pairs_of(&widget.work),
        sorted(&[(fx.platform_id, fx.sessions), (fx.platform_id, fx.units)])
    );
    // The accounts' coverage is merged: sessions are UNKNOWN after the
    // second account's last asserted day, units COMPLETE.
    let status = |measure: Uuid| {
        widget
            .work
            .coverage
            .iter()
            .find(|item| (item.platform_id, item.measure_id) == (fx.platform_id, measure))
            .expect("a served pair")
            .status
    };
    assert_eq!(status(fx.sessions), MetricCoverageStatus::Unknown);
    assert_eq!(status(fx.units), MetricCoverageStatus::Complete);
    let sections = [
        (&widget.work, vec![fx.works[0]]),
        (widget.chapters.as_ref().expect("chapters"), vec![chapter]),
    ];
    for (section, works) in sections {
        assert_each_served_pair_equals_its_dashboard_read(&fx, section, &works);
    }
}

/// The single-pair `metricDashboard` read the oracle compares a served pair
/// with, over March.
fn single_pair_read(
    fx: &Fixture,
    works: &[Uuid],
    platform: Uuid,
    measure: Uuid,
) -> MetricDashboard {
    single_pair_read_over(fx, works, platform, measure, march())
}

/// The single-pair `metricDashboard` read over `range`.
fn single_pair_read_over(
    fx: &Fixture,
    works: &[Uuid],
    platform: Uuid,
    measure: Uuid,
    range: (NaiveDate, NaiveDate),
) -> MetricDashboard {
    single_pair_read_at(
        fx,
        works,
        platform,
        measure,
        range,
        MetricTimelineGrain::Month,
    )
}

/// The single-pair `metricDashboard` read over `range` at `grain`.
fn single_pair_read_at(
    fx: &Fixture,
    works: &[Uuid],
    platform: Uuid,
    measure: Uuid,
    range: (NaiveDate, NaiveDate),
    grain: MetricTimelineGrain,
) -> MetricDashboard {
    let input = single_pair_input_at(fx, works, platform, measure, range, grain);
    metric_dashboard(&fx.pool, &input).expect("the oracle read")
}

/// The input of a single-pair read: the publisher, the section's works, one
/// platform and one measure, monthly, countries on and institutions off.
fn single_pair_input(
    fx: &Fixture,
    works: &[Uuid],
    platform: Uuid,
    measure: Uuid,
    range: (NaiveDate, NaiveDate),
) -> MetricDashboardInput {
    single_pair_input_at(
        fx,
        works,
        platform,
        measure,
        range,
        MetricTimelineGrain::Month,
    )
}

/// [`single_pair_input`] at `grain`.
fn single_pair_input_at(
    fx: &Fixture,
    works: &[Uuid],
    platform: Uuid,
    measure: Uuid,
    (start, end): (NaiveDate, NaiveDate),
    grain: MetricTimelineGrain,
) -> MetricDashboardInput {
    let mut input = request(
        fx.publisher_id,
        start,
        end,
        &[platform],
        &[measure],
        Some(grain),
    );
    input.selector.work_ids = Some(works.to_vec());
    input.include_countries = Some(true);
    input.include_institutions = Some(false);
    input
}

/// The equivalence oracle over one section: each served pair's totals,
/// timeline, countries and coverage item equal its single-pair read, and the
/// section's `dataThrough` and `isPartial` are the register s4.1 aggregation
/// over exactly the served pairs. Over March.
fn assert_each_served_pair_equals_its_dashboard_read(
    fx: &Fixture,
    section: &MetricWidgetSection,
    works: &[Uuid],
) {
    assert_each_served_pair_equals_its_dashboard_read_over(fx, section, works, march());
}

/// The equivalence oracle over one section, read over `range`.
fn assert_each_served_pair_equals_its_dashboard_read_over(
    fx: &Fixture,
    section: &MetricWidgetSection,
    works: &[Uuid],
    range: (NaiveDate, NaiveDate),
) {
    assert_each_served_pair_equals_its_dashboard_read_at(
        fx,
        section,
        works,
        range,
        MetricTimelineGrain::Month,
    );
}

/// The equivalence oracle over one section, read over `range` at `grain`.
fn assert_each_served_pair_equals_its_dashboard_read_at(
    fx: &Fixture,
    section: &MetricWidgetSection,
    works: &[Uuid],
    range: (NaiveDate, NaiveDate),
    grain: MetricTimelineGrain,
) {
    let mut partial = false;
    let mut through = Vec::new();
    assert!(!section.coverage.is_empty());
    for item in &section.coverage {
        let (platform, measure) = (item.platform_id, item.measure_id);
        let dashboard = single_pair_read_at(fx, works, platform, measure, range, grain);
        let label = format!("{works:?} {platform} {measure} {grain:?}");
        assert_eq!(
            section
                .totals
                .iter()
                .filter(|t| (t.platform_id, t.measure_id) == (platform, measure))
                .cloned()
                .collect::<Vec<_>>(),
            dashboard.totals,
            "{label}"
        );
        assert_eq!(
            section
                .timeline
                .iter()
                .filter(|b| (b.platform_id, b.measure_id) == (platform, measure))
                .cloned()
                .collect::<Vec<_>>(),
            dashboard.timeline,
            "{label}"
        );
        assert_eq!(
            section
                .countries
                .iter()
                .filter(|c| (c.platform_id, c.measure_id) == (platform, measure))
                .cloned()
                .collect::<Vec<_>>(),
            dashboard.countries,
            "{label}"
        );
        let oracle = &dashboard.coverage.items[0];
        assert_eq!(
            (item.status, item.data_through, item.country_coverage),
            (oracle.status, oracle.data_through, oracle.country_coverage),
            "{label}"
        );
        partial |= dashboard.is_partial;
        through.push(oracle.data_through);
    }
    // The register s4.1 aggregation over exactly the served pairs.
    let expected_through = through
        .into_iter()
        .collect::<Option<Vec<NaiveDate>>>()
        .and_then(|dates| dates.into_iter().min());
    assert_eq!(section.data_through, expected_through);
    assert_eq!(section.is_partial, partial);
}

#[test]
fn a_served_pair_beside_an_unserved_cross_pair_equals_its_single_pair_read() {
    let (_guard, fx) = widget_setup();
    let (start, end) = march();
    // The served pairs, (platform, sessions) with a country breakdown and
    // (other platform, units), each completely covered with countries; the
    // other platform gets its own eligible account for that.
    let gb = Dims {
        country: Some("GB"),
        ..Dims::default()
    };
    commit_dims(
        &fx,
        fx.works[0],
        fx.platform_id,
        fx.sessions,
        day_n(2),
        gb,
        3,
    );
    diagonal(&fx);
    cover_march(&fx, fx.sessions);
    let (source, account, import) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    exec(
        &fx.pool,
        &format!(
            "{}{}{}{}",
            source_sql(source, "DRIVER", true),
            account_sql(
                account,
                source,
                fx.other_platform_id,
                Some(fx.publisher_id),
                true
            ),
            import_sql(import, account, "COMPLETED", "'2026-04-10T00:00:00Z'"),
            coverage_sql(
                account,
                import,
                fx.other_platform_id,
                fx.units,
                start,
                end,
                "COMPLETE",
                true,
                true
            ),
        ),
    );
    // The unserved cross pair (platform, units): an unapplied delta (rollup
    // lag), a partial coverage assertion and unresolved identifier evidence.
    commit(&fx, fx.works[0], fx.platform_id, fx.units, day_n(9), 6);
    cover(
        &fx,
        fx.units,
        start,
        end,
        "PARTIAL",
        "COMPLETED",
        "'2026-04-10T00:00:00Z'",
    );
    quarantine(
        &fx,
        fx.publisher_id,
        fx.platform_id,
        fx.units,
        day_n(9),
        day_n(10),
        "DAY",
    );
    // The cross pair's own read carries all three, so they are live.
    let cross = single_pair_read(&fx, &[fx.works[0]], fx.platform_id, fx.units);
    assert!(cross.is_partial);
    for code in [
        MetricWarningCode::PartialCoverage,
        MetricWarningCode::UnresolvedIdentifiers,
        MetricWarningCode::RollupLag,
    ] {
        assert!(
            cross.warnings.iter().any(|warning| warning.code == code),
            "{code:?}"
        );
    }

    let widget = read(&fx.pool, &input_for(BOOK_BARE)).expect("served");
    assert_eq!(widget.chapters, None);
    let section = &widget.work;
    let served = sorted(&[
        (fx.platform_id, fx.sessions),
        (fx.other_platform_id, fx.units),
    ]);
    // Exactly the two diagonal pairs, in every list of the section.
    assert_eq!(pairs_of(section), served);
    let pair_of_total = |t: &MetricTotal| (t.platform_id, t.measure_id);
    assert_eq!(
        section.totals.iter().map(pair_of_total).collect::<Vec<_>>(),
        served
    );
    let mut timeline: Vec<(Uuid, Uuid)> = section
        .timeline
        .iter()
        .map(|b| (b.platform_id, b.measure_id))
        .collect();
    timeline.dedup();
    assert_eq!(timeline, served);
    assert!(!section.countries.is_empty());
    assert!(section
        .countries
        .iter()
        .all(|c| served.contains(&(c.platform_id, c.measure_id))));
    // Each served pair equals its single-pair read; dataThrough and
    // isPartial aggregate over those two pairs only, so the cross pair's lag,
    // assertion and identifier evidence neither make the section partial nor
    // move its dataThrough.
    assert_each_served_pair_equals_its_dashboard_read(&fx, section, &[fx.works[0]]);
    assert!(!section.is_partial);
    assert_eq!(section.data_through, Some(d(2026, 3, 31)));
}

#[test]
fn each_served_pair_equals_its_single_pair_read_from_january_to_january() {
    let (_guard, fx) = widget_setup();
    let range = year_2025();
    let (gb, fr) = (
        Dims {
            country: Some("GB"),
            ..Dims::default()
        },
        Dims {
            country: Some("FR"),
            ..Dims::default()
        },
    );
    // The served pairs: (platform, sessions) with countries in two months,
    // covered for the year, and (other platform, units), uncovered.
    commit(
        &fx,
        fx.works[0],
        fx.platform_id,
        fx.sessions,
        d(2025, 3, 2),
        4,
    );
    commit_dims(
        &fx,
        fx.works[0],
        fx.platform_id,
        fx.sessions,
        d(2025, 3, 2),
        gb,
        4,
    );
    commit_dims(
        &fx,
        fx.works[0],
        fx.platform_id,
        fx.sessions,
        d(2025, 7, 9),
        fr,
        6,
    );
    commit(
        &fx,
        fx.works[0],
        fx.other_platform_id,
        fx.units,
        d(2025, 5, 1),
        5,
    );
    cover(
        &fx,
        fx.sessions,
        range.0,
        range.1,
        "COMPLETE",
        "COMPLETED",
        "'2026-01-10T00:00:00Z'",
    );
    apply_all(&fx.pool);
    // A yearly country row on the unserved cross pair (platform, units),
    // which the pair-mode filter drops.
    exec(
        &fx.pool,
        &format!(
            "INSERT INTO metric_rollup_work_country_year \
                 (rollup_work_country_year_id, work_id, publication_id, platform_id, \
                  measure_id, year_start, country_code, value, \
                  requires_institution_coverage, watermark) \
             VALUES ('{}', '{}', NULL, '{}', '{}', '2025-01-01', 'DE', 7, FALSE, 1);",
            Uuid::new_v4(),
            fx.works[0],
            fx.platform_id,
            fx.units,
        ),
    );

    let (result, _, names) = captured(&input_over(BOOK_BARE, range), &profile(&no_hook));
    let widget = result.expect("served");
    assert_eq!(widget.chapters, None);
    assert_eq!(
        names
            .iter()
            .filter(|name| **name == "MONTHS_YEAR_ONLY_SQL")
            .count(),
        1
    );
    let section = &widget.work;
    let served = sorted(&[
        (fx.platform_id, fx.sessions),
        (fx.other_platform_id, fx.units),
    ]);
    assert_eq!(pairs_of(section), served);
    assert!(!section.countries.is_empty());
    assert!(section
        .countries
        .iter()
        .all(|c| served.contains(&(c.platform_id, c.measure_id))));
    // The dashboard reads the same year with the same text.
    let (pool, log) = capturing_pool();
    metric_dashboard(
        &pool,
        &single_pair_input(&fx, &[fx.works[0]], fx.platform_id, fx.sessions, range),
    )
    .expect("the oracle read");
    assert!(
        statement_names(&log.lock().expect("the statement log")).contains(&"MONTHS_YEAR_ONLY_SQL")
    );
    assert_each_served_pair_equals_its_dashboard_read_over(&fx, section, &[fx.works[0]], range);
}

#[test]
fn a_missing_frontier_or_label_is_internal() {
    let ids = BTreeSet::from([Uuid::new_v4()]);
    let none = BTreeSet::new();
    assert_eq!(
        interpret_labels(Vec::new(), &none, &none).map(|_| ()),
        Err(Failure::new(
            MetricWidgetError::Internal,
            WidgetReason::FrontierMissing
        ))
    );
    let frontier = LabelRow {
        kind: "FRONTIER".to_string(),
        id: None,
        code: None,
        display_name: None,
        description: None,
        unit: None,
        as_of: Some(Timestamp(Utc::now())),
        watermark_at: Some(Timestamp(Utc::now())),
    };
    assert_eq!(
        interpret_labels(vec![frontier], &ids, &none).map(|_| ()),
        Err(Failure::new(
            MetricWidgetError::Internal,
            WidgetReason::LabelMissing
        ))
    );
}

#[test]
fn an_engine_frontier_other_than_the_labels_frontier_is_internal() {
    // A whole read cannot produce the mismatch: the labels statement and the
    // engine read `transaction_timestamp()` and the frontier row in one
    // repeatable-read snapshot. So step 13 runs here on its own, inside such
    // a transaction, given the labels' frontier and then a different one.
    let (_guard, fx) = widget_setup();
    commit(&fx, fx.works[0], fx.platform_id, fx.sessions, day_n(2), 3);
    apply_all(&fx.pool);
    let request = validate(&input_for(BOOK_BARE), today()).expect("valid");
    let (works, pairs) = ([fx.works[0]], [(fx.platform_id, fx.sessions)]);
    let platforms = BTreeSet::from([fx.platform_id]);
    let measures = BTreeSet::from([fx.sessions]);
    let mut connection = fx.pool.get().expect("a connection");
    connection
        .build_transaction()
        .read_only()
        .repeatable_read()
        .run(|connection| {
            diesel::sql_query(CUSTOM_PLANS_SQL).execute(connection)?;
            diesel::sql_query(METRIC_WIDGET_STATEMENT_TIMEOUT_SQL).execute(connection)?;
            diesel::sql_query(PROJECTION_LOCK_SQL).execute(connection)?;
            let labels: Vec<LabelRow> = diesel::sql_query(METRIC_WIDGET_LABELS_SQL)
                .bind::<Array<SqlUuid>, _>(platforms.iter().copied().collect::<Vec<Uuid>>())
                .bind::<Array<SqlUuid>, _>(measures.iter().copied().collect::<Vec<Uuid>>())
                .load(connection)?;
            let ((as_of, watermark), _, _) =
                interpret_labels(labels, &platforms, &measures).expect("the labels");
            let deadline = Deadline::new(Instant::now() + METRIC_WIDGET_DEADLINE);
            let mut engine = |frontier: (Timestamp, Timestamp)| {
                let mut facts = fresh_facts();
                let section = engine_section(
                    connection,
                    WidgetSection::Work,
                    fx.publisher_id,
                    &works,
                    &pairs,
                    &request,
                    frontier,
                    &deadline,
                    &mut facts,
                );
                (section, facts)
            };
            // The labels' own frontier: served.
            let (section, _) = engine((as_of, watermark));
            assert_eq!(
                section.map(|section| pairs_of(&section)),
                Ok(pairs.to_vec())
            );
            // Any other `asOf` or watermark: INTERNAL_ERROR.
            let later = |at: Timestamp| Timestamp(at.0 + chrono::TimeDelta::seconds(1));
            for frontier in [(later(as_of), watermark), (as_of, later(watermark))] {
                let (section, facts) = engine(frontier);
                let failure = section.map(|_| ()).expect_err("a frontier mismatch");
                assert_eq!(
                    failure,
                    Failure::new(MetricWidgetError::Internal, WidgetReason::FrontierMismatch)
                );
                assert_eq!(failure.error.code(), "INTERNAL_ERROR");
                assert_eq!(facts.engine_section, Some(WidgetSection::Work));
            }
            Ok::<(), diesel::result::Error>(())
        })
        .expect("the engine step's transaction");
}

#[test]
fn the_switch_parses_exactly_off_and_on() {
    assert_eq!("OFF".parse(), Ok(MetricWidgetSwitch::Off));
    assert_eq!("ON".parse(), Ok(MetricWidgetSwitch::On));
    for invalid in ["", "off", "on", "1", "ON "] {
        assert!(invalid.parse::<MetricWidgetSwitch>().is_err(), "{invalid}");
    }
    assert_eq!(MetricWidgetSwitch::default(), MetricWidgetSwitch::Off);
    assert_eq!(MetricWidgetSwitch::On.to_string(), "ON");
    assert_eq!(MetricWidgetSwitch::Off.to_string(), "OFF");
}

// ==========================================================================
// MET-WP4-WIDGET-ALLTIME: yearly timelines and all-history reads
// ==========================================================================

/// A `YEAR` read of all served history before `end` (`startDate` omitted).
fn year_input(doi: &str, end: NaiveDate) -> MetricWidgetInput {
    MetricWidgetInput {
        start_date: None,
        end_date: end,
        timeline_grain: Some(MetricWidgetGrain::Year),
        ..input_for(doi)
    }
}

/// A `YEAR` read over `[start, end)`.
fn year_input_from(doi: &str, start: NaiveDate, end: NaiveDate) -> MetricWidgetInput {
    MetricWidgetInput {
        start_date: Some(start),
        ..year_input(doi, end)
    }
}

/// [`year_input`] selecting `chapters`.
fn year_input_with_chapters(doi: &str, end: NaiveDate, chapters: &[&str]) -> MetricWidgetInput {
    MetricWidgetInput {
        chapter_dois: Some(chapters.iter().map(|doi| Doi(doi.to_string())).collect()),
        ..year_input(doi, end)
    }
}

/// Two chapters of the book, with DOIs.
fn two_chapters(fx: &Fixture) -> (Uuid, Uuid) {
    let imprint = imprint_of(&fx.pool, fx.works[0]);
    let first = add_chapter(
        &fx.pool,
        fx.works[0],
        imprint,
        Some("https://doi.org/10.5555/widget-chapter-1"),
        1,
    );
    let second = add_chapter(
        &fx.pool,
        fx.works[0],
        imprint,
        Some("https://doi.org/10.5555/widget-chapter-2"),
        2,
    );
    (first, second)
}

/// One active native `MONTH` canonical record over `[start, end)`.
fn native_record_over(
    fx: &Fixture,
    work_id: Uuid,
    platform_id: Uuid,
    measure_id: Uuid,
    (start, end): (NaiveDate, NaiveDate),
) {
    let (record_id, revision_id) = (Uuid::new_v4(), Uuid::new_v4());
    exec(
        &fx.pool,
        &format!(
            "INSERT INTO metric_record \
                 (record_id, identity_hash, work_id, platform_id, measure_id, period_start, \
                  period_end, reporting_grain, winning_source_account_id) \
             VALUES ('{record_id}', 'native-{record_id}', '{work_id}', '{platform_id}', \
                     '{measure_id}', '{start}', '{end}', 'MONTH', '{account}'); \
             INSERT INTO metric_record_revision \
                 (record_revision_id, record_id, revision_number, import_id, value, \
                  content_hash, status) \
             VALUES ('{revision_id}', '{record_id}', 1, '{import}', 50, \
                     'content-{revision_id}', 'CURRENT'); \
             UPDATE metric_record SET current_revision_id = '{revision_id}' \
              WHERE record_id = '{record_id}';",
            account = fx.account_id,
            import = fx.canonical_import,
        ),
    );
}

/// Complete coverage, with countries, of `measure` on the fixture platform
/// over `[start, end)`.
fn cover_over(fx: &Fixture, measure_id: Uuid, start: NaiveDate, end: NaiveDate) {
    cover(
        fx,
        measure_id,
        start,
        end,
        "COMPLETE",
        "COMPLETED",
        "'2026-04-10T00:00:00Z'",
    );
}

/// A work-day value with a country breakdown.
fn commit_country(
    fx: &Fixture,
    work: Uuid,
    platform: Uuid,
    measure: Uuid,
    day: NaiveDate,
    country: &'static str,
    value: i64,
) {
    commit(fx, work, platform, measure, day, value);
    commit_dims(
        fx,
        work,
        platform,
        measure,
        day,
        Dims {
            country: Some(country),
            ..Dims::default()
        },
        value,
    );
}

/// An all-history read, which must be served, and the same request with
/// `startDate` set to its served start, read against the same database state:
/// the two are equal except for `asOf` (section 4.3).
fn history_and_explicit(pool: &PgPool, input: &MetricWidgetInput) -> MetricWidget {
    assert_eq!(input.start_date, None);
    let history = read(pool, input).expect("the all-history read is served");
    let explicit = MetricWidgetInput {
        start_date: Some(history.start_date),
        ..input.clone()
    };
    let served = read(pool, &explicit).expect("the explicit-start read is served");
    assert_eq!(without_as_of(history.clone()), without_as_of(served));
    history
}

/// The `(startDate, endDate)` of each timeline bucket of one pair.
fn buckets_of(
    section: &MetricWidgetSection,
    platform_id: Uuid,
    measure_id: Uuid,
) -> Vec<(NaiveDate, NaiveDate)> {
    section
        .timeline
        .iter()
        .filter(|bucket| (bucket.platform_id, bucket.measure_id) == (platform_id, measure_id))
        .map(|bucket| (bucket.start_date, bucket.end_date))
        .collect()
}

/// The values of each timeline bucket of one pair.
fn values_of(
    section: &MetricWidgetSection,
    platform_id: Uuid,
    measure_id: Uuid,
) -> Vec<Option<String>> {
    section
        .timeline
        .iter()
        .filter(|bucket| (bucket.platform_id, bucket.measure_id) == (platform_id, measure_id))
        .map(|bucket| text(&bucket.value))
        .collect()
}

/// The year buckets of `[start, end)`: the first from `start` to the next
/// 1 January, the last from the last 1 January before `end` to `end`.
fn year_buckets(start: NaiveDate, end: NaiveDate) -> Vec<(NaiveDate, NaiveDate)> {
    let mut buckets = Vec::new();
    let mut from = start;
    while from < end {
        let to = d(from.year() + 1, 1, 1).min(end);
        buckets.push((from, to));
        from = to;
    }
    buckets
}

/// Every pair of every section of `widget` has exactly the year buckets of
/// `[start, end)`.
fn assert_year_buckets(widget: &MetricWidget, start: NaiveDate, end: NaiveDate) {
    let expected = year_buckets(start, end);
    for section in [Some(&widget.work), widget.chapters.as_ref()]
        .into_iter()
        .flatten()
    {
        for (platform_id, measure_id) in pairs_of(section) {
            assert_eq!(
                buckets_of(section, platform_id, measure_id),
                expected,
                "{platform_id} {measure_id}"
            );
        }
    }
}

/// The outcome line of one resolution.
fn outcome_line(lines: &[WidgetLogLine]) -> &WidgetLogLine {
    lines
        .iter()
        .find(|line| line.kind == WidgetLineKind::Outcome)
        .expect("an outcome line")
}

/// The captured sequence of a read serving both sections: `pairs` twice and
/// both engine runs with the step-7 text `months`.
fn two_section_sequence(pairs: &'static str, months: &'static str) -> Vec<&'static str> {
    let mut expected = vec![
        "CHECKOUT_CHECK",
        "BEGIN",
        "CUSTOM_PLANS_SQL",
        "METRIC_WIDGET_STATEMENT_TIMEOUT_SQL",
        "PROJECTION_LOCK_SQL",
        "METRIC_WIDGET_RESOLUTION_SQL",
        pairs,
        pairs,
        "METRIC_WIDGET_LABELS_SQL",
    ];
    expected.extend(engine_run_with(months));
    expected.extend(engine_run_with(months));
    expected.push("COMMIT");
    expected
}

/// The book and both chapters each serve two pairs on each of `days`.
fn two_section_book(fx: &Fixture, days: &[NaiveDate]) {
    let (first, second) = two_chapters(fx);
    for day in days {
        for work in [fx.works[0], first, second] {
            commit(fx, work, fx.platform_id, fx.sessions, *day, 4);
            commit(fx, work, fx.platform_id, fx.units, *day, 2);
        }
    }
    apply_all(&fx.pool);
}

/// The 22 captured statements, of which 19 named, of the longest path.
fn assert_the_ceiling(names: &[&str]) {
    assert_eq!(names.len(), 22, "{names:?}");
    let named = names
        .iter()
        .filter(|name| !["CHECKOUT_CHECK", "BEGIN", "COMMIT"].contains(name))
        .count();
    assert_eq!(named, 19, "the statement ceiling is 19 named statements");
    for absent in [
        "WORKS_SQL",
        "KNOWN_SQL",
        "REPRESENTED_SQL",
        "MONTHS_SCAN_SQL",
        "MONTHS_FULL_SCAN_SQL",
        "MONTHS_YEAR_SCAN_SQL",
        "MONTHS_YEAR_FULL_SCAN_SQL",
        "MONTHS_YEAR_ONLY_SCAN_SQL",
        "DAY_SUMS_SQL",
        "DIMENSION_CELLS_SQL",
        "DAY_SECTIONS_SQL",
    ] {
        assert!(!names.contains(&absent), "{absent} must not run");
    }
}

// -- Validation (section 4.1 step 4) ---------------------------------------

#[test]
fn an_omitted_or_null_grain_is_month_and_a_month_read_needs_a_start() {
    let explicit = input_for(BOOK_BARE);
    let null = MetricWidgetInput {
        timeline_grain: None,
        ..explicit.clone()
    };
    let validated_null = validated(&null).expect("valid");
    assert_eq!(validated_null.grain, MetricWidgetGrain::Month);
    assert!(!validated_null.history);
    assert_eq!(validated_null, validated(&explicit).expect("valid"));
    let required = Err(MetricWidgetError::QueryInvalid(START_REQUIRED_MESSAGE));
    for grain in [None, Some(MetricWidgetGrain::Month)] {
        let no_start = MetricWidgetInput {
            start_date: None,
            timeline_grain: grain,
            ..input_for(BOOK_BARE)
        };
        assert_eq!(validated(&no_start), required, "{grain:?}");
    }
    // Every R11 MONTH case keeps its result with a null grain too.
    for (start, end) in [
        (d(2026, 3, 2), d(2026, 4, 1)),
        (d(2026, 4, 1), d(2026, 3, 1)),
        (d(1999, 12, 1), d(2000, 2, 1)),
        (d(2027, 1, 1), d(2027, 12, 1)),
        (d(2025, 3, 1), d(2026, 3, 1)),
        (d(2025, 3, 1), d(2026, 4, 1)),
    ] {
        let month = input_over(BOOK_BARE, (start, end));
        let null = MetricWidgetInput {
            timeline_grain: None,
            ..month.clone()
        };
        assert_eq!(validated(&null), validated(&month), "{start} {end}");
    }
}

#[test]
fn a_year_read_with_a_start_keeps_r11_dates_and_domain_without_the_month_cap() {
    let year = |start: NaiveDate, end: NaiveDate| year_input_from(BOOK_BARE, start, end);
    let invalid = Err(MetricWidgetError::QueryInvalid(DATES_MESSAGE));
    let domain = Err(MetricWidgetError::QueryLimitExceeded(DATE_DOMAIN_MESSAGE));
    assert_eq!(validated(&year(d(2026, 3, 2), d(2026, 4, 1))), invalid);
    assert_eq!(validated(&year(d(2026, 3, 1), d(2026, 4, 2))), invalid);
    assert_eq!(validated(&year(d(2026, 3, 1), d(2026, 3, 1))), invalid);
    assert_eq!(validated(&year(d(2026, 4, 1), d(2025, 3, 1))), invalid);
    assert_eq!(validated(&year(d(1999, 12, 1), d(2026, 4, 1))), domain);
    // 366 days after 2026-11-10 is 2027-11-11.
    assert_eq!(validated(&year(d(2000, 1, 1), d(2027, 12, 1))), domain);
    // More than 12 months: from the floor to the first day of next month.
    let longest = validated(&year(d(2000, 1, 1), d(2026, 12, 1))).expect("no month cap");
    assert_eq!(
        (longest.start, longest.end, longest.grain, longest.history),
        (
            d(2000, 1, 1),
            d(2026, 12, 1),
            MetricWidgetGrain::Year,
            false
        )
    );
    assert!(validated(&year(d(2000, 1, 1), d(2027, 11, 1))).is_ok());
}

#[test]
fn a_year_read_without_a_start_checks_its_end_date_alone() {
    let omitted = |end: NaiveDate| validated(&year_input(BOOK_BARE, end));
    assert_eq!(
        omitted(d(2026, 3, 2)),
        Err(MetricWidgetError::QueryInvalid(DATES_MESSAGE))
    );
    let domain = Err(MetricWidgetError::QueryLimitExceeded(
        OMITTED_START_DOMAIN_MESSAGE,
    ));
    for end in [d(1999, 12, 1), d(2000, 1, 1), d(2027, 12, 1)] {
        assert_eq!(omitted(end), domain, "{end}");
    }
    let earliest = omitted(d(2000, 2, 1)).expect("accepted");
    assert!(earliest.history);
    assert_eq!(earliest.grain, MetricWidgetGrain::Year);
    // Until step 11a the start is the empty response's: the month before.
    assert_eq!(earliest.start, d(2000, 1, 1));
    assert_eq!(
        omitted(d(2026, 1, 1)).expect("accepted").start,
        d(2025, 12, 1)
    );
    assert!(omitted(d(2027, 11, 1)).is_ok());
}

#[test]
fn the_first_failure_in_section_4_1_order_is_returned_for_any_doi() {
    let many = || Some((0..501).map(|_| Doi("nonsense".to_string())).collect());
    for doi in [BOOK_BARE, "10.5555/widget-unknown"] {
        // The DOI comes first, then a MONTH read's missing start.
        let mut month = MetricWidgetInput {
            start_date: None,
            end_date: d(2026, 3, 2),
            chapter_dois: many(),
            ..input_for("nonsense")
        };
        assert_eq!(
            validated(&month),
            Err(MetricWidgetError::QueryInvalid(INVALID_DOI_MESSAGE))
        );
        month.doi = Doi(doi.to_string());
        assert_eq!(
            validated(&month),
            Err(MetricWidgetError::QueryInvalid(START_REQUIRED_MESSAGE)),
            "{doi}"
        );
        // Without a start, the end date's day, then its domain, then
        // chapterDois.
        let mut year = MetricWidgetInput {
            chapter_dois: many(),
            ..year_input(doi, d(2026, 3, 2))
        };
        assert_eq!(
            validated(&year),
            Err(MetricWidgetError::QueryInvalid(DATES_MESSAGE)),
            "{doi}"
        );
        year.end_date = d(2000, 1, 1);
        assert_eq!(
            validated(&year),
            Err(MetricWidgetError::QueryLimitExceeded(
                OMITTED_START_DOMAIN_MESSAGE
            )),
            "{doi}"
        );
        year.end_date = d(2026, 4, 1);
        assert_eq!(
            validated(&year),
            Err(MetricWidgetError::QueryLimitExceeded(
                TOO_MANY_CHAPTER_DOIS_MESSAGE
            )),
            "{doi}"
        );
        // With a start, a YEAR read has no range check before chapterDois.
        year.start_date = Some(d(2000, 1, 1));
        assert_eq!(
            validated(&year),
            Err(MetricWidgetError::QueryLimitExceeded(
                TOO_MANY_CHAPTER_DOIS_MESSAGE
            )),
            "{doi}"
        );
    }
}

// -- Logging (section 4.6) -------------------------------------------------

#[test]
fn the_outcome_line_carries_grain_history_and_buckets_once_step_4_passed() {
    let mut served = facts(None, WidgetStep::Commit);
    served.grain = Some(MetricWidgetGrain::Year);
    served.history = Some(true);
    served.buckets = Some(27);
    let message = log_decision(&served)[0].message();
    assert!(
        message.contains(
            " work_pairs/cross=2/4 chapters_pairs/cross=- grain=YEAR history=true buckets=27 "
        ),
        "{message}"
    );
    // Before step 11a, and on the floor refusal, the count prints as `-`.
    let mut refused = facts(
        Some(MetricWidgetError::QueryLimitExceeded(HISTORY_FLOOR_MESSAGE)),
        WidgetStep::Pairs,
    );
    refused.grain = Some(MetricWidgetGrain::Year);
    refused.history = Some(true);
    refused.buckets = None;
    let message = log_decision(&refused)[0].message();
    assert!(
        message.contains("chapters_pairs/cross=- grain=YEAR history=true buckets=-"),
        "{message}"
    );
    // A MONTH read with its start.
    let message = log_decision(&facts(None, WidgetStep::Commit))[0].message();
    assert!(
        message.contains("grain=MONTH history=false buckets=1"),
        "{message}"
    );
    // Lines of reads refused at steps 2 to 4 are unchanged, and only the
    // outcome line carries the fields.
    assert_eq!(
        log_decision(&facts(
            Some(MetricWidgetError::QueryInvalid(DATES_MESSAGE)),
            WidgetStep::Input,
        ))[0]
            .message(),
        "metricWidget outcome outcome=METRIC_QUERY_INVALID step=4 elapsed_ms=12 \
         work_pairs/cross=2/4 chapters_pairs/cross=-"
    );
    let mut claimed = facts(
        Some(MetricWidgetError::QueryLimitExceeded(ONE_SELECTION_MESSAGE)),
        WidgetStep::Claim,
    );
    claimed.previous_claims = Some(1);
    assert_eq!(
        log_decision(&claimed)[0].message(),
        "metricWidget outcome outcome=METRIC_QUERY_LIMIT_EXCEEDED step=3 elapsed_ms=12 \
         work_pairs/cross=- chapters_pairs/cross=-"
    );
    let mut failed = facts(Some(MetricWidgetError::Internal), WidgetStep::Engine);
    failed.reason = Some(WidgetReason::EngineUnavailable);
    let lines = log_decision(&failed);
    assert!(lines[0].message().contains("grain=MONTH"));
    assert!(
        !lines[1].message().contains("grain="),
        "{}",
        lines[1].message()
    );
    let mut off = facts(Some(MetricWidgetError::Unavailable), WidgetStep::Switch);
    off.switch = MetricWidgetSwitch::Off;
    assert!(!log_decision(&off)[0].message().contains("grain="));
}

// -- The served start (section 4.3) ----------------------------------------

#[test]
fn the_earliest_value_of_either_section_sets_one_start_for_both_timelines() {
    let (_guard, fx) = widget_setup();
    let (first, _) = two_chapters(&fx);
    let end = d(2026, 4, 1);
    // The book from June 2024; a chapter from August 2023.
    commit(
        &fx,
        fx.works[0],
        fx.platform_id,
        fx.sessions,
        d(2024, 6, 3),
        4,
    );
    commit(
        &fx,
        fx.works[0],
        fx.platform_id,
        fx.sessions,
        d(2026, 3, 2),
        1,
    );
    commit(&fx, first, fx.platform_id, fx.units, d(2023, 8, 9), 2);
    apply_all(&fx.pool);
    let widget = history_and_explicit(&fx.pool, &year_input(BOOK_BARE, end));
    assert_eq!((widget.start_date, widget.end_date), (d(2023, 8, 1), end));
    assert_eq!(pairs_of(&widget.work), [(fx.platform_id, fx.sessions)]);
    assert_eq!(
        pairs_of(widget.chapters.as_ref().expect("chapters")),
        [(fx.platform_id, fx.units)]
    );
    // Both timelines start on that day, with a clipped first bucket and a
    // clipped last one.
    assert_year_buckets(&widget, d(2023, 8, 1), end);
    assert_eq!(
        buckets_of(&widget.work, fx.platform_id, fx.sessions)[0],
        (d(2023, 8, 1), d(2024, 1, 1))
    );
    assert_eq!(
        buckets_of(&widget.work, fx.platform_id, fx.sessions)[3],
        (d(2026, 1, 1), end)
    );

    // An earlier value of the work moves the start for both sections.
    commit(
        &fx,
        fx.works[0],
        fx.platform_id,
        fx.sessions,
        d(2022, 2, 7),
        1,
    );
    apply_all(&fx.pool);
    let widget = history_and_explicit(&fx.pool, &year_input(BOOK_BARE, end));
    assert_eq!(widget.start_date, d(2022, 2, 1));
    assert_year_buckets(&widget, d(2022, 2, 1), end);
    assert_eq!(
        total(&widget.work, fx.platform_id, fx.sessions),
        Some("6".to_string())
    );
}

#[test]
fn ineligible_rows_do_not_move_the_served_start() {
    let (_guard, fx) = widget_setup();
    let book = fx.works[0];
    commit(&fx, book, fx.platform_id, fx.sessions, d(2025, 5, 4), 3);
    let hidden = insert_measure(&fx.pool, "widget_history_hidden", true, true);
    exec(
        &fx.pool,
        &format!(
            "UPDATE metric_measure SET public_visibility = FALSE WHERE measure_id = '{hidden}';"
        ),
    );
    let not_across_time = insert_measure(&fx.pool, "widget_history_not_time", false, true);
    let not_across_works = insert_measure(&fx.pool, "widget_history_not_works", true, false);
    let disabled = insert_measure(&fx.pool, "widget_history_disabled", true, true);
    for measure in [hidden, not_across_time, not_across_works, disabled] {
        commit(&fx, book, fx.platform_id, measure, d(2021, 2, 4), 9);
    }
    commit(
        &fx,
        book,
        fx.other_platform_id,
        fx.sessions,
        d(2021, 2, 4),
        9,
    );
    apply_all(&fx.pool);
    exec(
        &fx.pool,
        &format!(
            "UPDATE metric_measure SET enabled = FALSE WHERE measure_id = '{disabled}'; \
             UPDATE metric_platform SET enabled = FALSE WHERE platform_id = '{}';",
            fx.other_platform_id
        ),
    );
    let widget = history_and_explicit(&fx.pool, &year_input(BOOK_BARE, d(2026, 4, 1)));
    assert_eq!(widget.start_date, d(2025, 5, 1));
    assert_eq!(pairs_of(&widget.work), [(fx.platform_id, fx.sessions)]);
}

#[test]
fn coverage_lag_identifiers_native_records_and_later_rows_do_not_move_the_start() {
    let (_guard, fx) = widget_setup();
    let book = fx.works[0];
    let end = d(2026, 4, 1);
    commit(&fx, book, fx.platform_id, fx.sessions, d(2025, 5, 4), 3);
    apply_all(&fx.pool);
    // Earlier coverage, unresolved identifier evidence and a native record
    // that ends before the served start, on the served pair.
    cover_over(&fx, fx.sessions, d(2020, 1, 1), end);
    quarantine(
        &fx,
        fx.publisher_id,
        fx.platform_id,
        fx.sessions,
        d(2021, 1, 4),
        d(2021, 1, 5),
        "DAY",
    );
    native_record_over(
        &fx,
        book,
        fx.platform_id,
        fx.sessions,
        (d(2019, 1, 1), d(2019, 2, 1)),
    );
    // Values on or after endDate, on the served pair and on a pair served
    // nowhere else.
    commit(&fx, book, fx.platform_id, fx.sessions, end, 8);
    commit(&fx, book, fx.platform_id, fx.units, d(2026, 6, 2), 8);
    apply_all(&fx.pool);
    // An earlier value that rollup has not applied yet.
    commit(&fx, book, fx.platform_id, fx.sessions, d(2022, 3, 3), 2);

    let widget = history_and_explicit(&fx.pool, &year_input(BOOK_BARE, end));
    assert_eq!(widget.start_date, d(2025, 5, 1));
    assert_eq!(pairs_of(&widget.work), [(fx.platform_id, fx.sessions)]);
    assert_eq!(
        total(&widget.work, fx.platform_id, fx.sessions),
        Some("3".to_string())
    );

    // Once applied, the earlier value moves the start, and is served.
    apply_all(&fx.pool);
    let widget = history_and_explicit(&fx.pool, &year_input(BOOK_BARE, end));
    assert_eq!(widget.start_date, d(2022, 3, 1));
    assert_eq!(
        total(&widget.work, fx.platform_id, fx.sessions),
        Some("5".to_string())
    );
}

#[test]
fn unselected_and_excluded_children_do_not_move_the_start() {
    let (_guard, fx) = widget_setup();
    let (first, second) = two_chapters(&fx);
    let foreign_imprint = imprint_of(&fx.pool, fx.other_work);
    let foreign = add_chapter(
        &fx.pool,
        fx.works[0],
        foreign_imprint,
        Some("https://doi.org/10.5555/widget-foreign"),
        3,
    );
    let end = d(2026, 4, 1);
    commit(
        &fx,
        fx.works[0],
        fx.platform_id,
        fx.sessions,
        d(2025, 9, 2),
        1,
    );
    commit(&fx, first, fx.platform_id, fx.sessions, d(2025, 6, 2), 2);
    commit(&fx, second, fx.platform_id, fx.sessions, d(2023, 6, 2), 3);
    commit(&fx, foreign, fx.platform_id, fx.sessions, d(2021, 6, 2), 4);
    apply_all(&fx.pool);
    // Every child selected: the foreign one is excluded and moves nothing.
    let all = history_and_explicit(&fx.pool, &year_input(BOOK_BARE, end));
    assert_eq!(all.start_date, d(2023, 6, 1));
    assert_eq!(all.excluded_child_count, 1);
    // Only the first chapter and the foreign one: the second is unselected.
    let some = history_and_explicit(
        &fx.pool,
        &year_input_with_chapters(
            BOOK_BARE,
            end,
            &["10.5555/widget-chapter-1", "10.5555/widget-foreign"],
        ),
    );
    assert_eq!(some.start_date, d(2025, 6, 1));
    assert_eq!(some.excluded_child_count, 1);
    assert_year_buckets(&some, d(2025, 6, 1), end);
}

#[test]
fn with_no_served_pair_the_start_is_the_month_before_the_end_and_no_engine_statement_runs() {
    let (_guard, fx) = widget_setup();
    let imprint = imprint_of(&fx.pool, fx.works[0]);
    add_chapter(&fx.pool, fx.works[0], imprint, None, 1);
    for (end, start) in [
        (d(2026, 4, 1), d(2026, 3, 1)),
        (d(2026, 1, 1), d(2025, 12, 1)),
    ] {
        let (result, lines, names) = captured(&year_input(BOOK_BARE, end), &profile(&no_hook));
        let widget = result.expect("served, empty");
        assert_eq!((widget.start_date, widget.end_date), (start, end));
        assert_eq!(widget.work, empty_section());
        assert_eq!(widget.chapters, Some(empty_section()));
        assert_eq!(
            names,
            [
                "CHECKOUT_CHECK",
                "BEGIN",
                "CUSTOM_PLANS_SQL",
                "METRIC_WIDGET_STATEMENT_TIMEOUT_SQL",
                "PROJECTION_LOCK_SQL",
                "METRIC_WIDGET_RESOLUTION_SQL",
                "METRIC_WIDGET_HISTORY_PAIRS_SQL",
                "METRIC_WIDGET_HISTORY_PAIRS_SQL",
                "METRIC_WIDGET_LABELS_SQL",
                "COMMIT",
            ]
        );
        let message = outcome_line(&lines).message();
        assert!(
            message.contains("grain=YEAR history=true buckets=1"),
            "{message}"
        );
    }
}

#[test]
fn a_served_month_before_2000_is_refused_after_the_pair_statements() {
    let (_guard, fx) = widget_setup();
    let end = d(2026, 4, 1);
    commit(
        &fx,
        fx.works[0],
        fx.platform_id,
        fx.sessions,
        d(1999, 12, 6),
        2,
    );
    commit(
        &fx,
        fx.works[0],
        fx.platform_id,
        fx.sessions,
        d(2026, 3, 2),
        1,
    );
    apply_all(&fx.pool);
    let (result, lines, names) = captured(&year_input(BOOK_BARE, end), &profile(&no_hook));
    let error = result.map(|_| ()).expect_err("refused");
    assert_eq!(
        error,
        MetricWidgetError::QueryLimitExceeded(HISTORY_FLOOR_MESSAGE)
    );
    assert_eq!(error.code(), "METRIC_QUERY_LIMIT_EXCEEDED");
    assert_eq!(
        names,
        [
            "CHECKOUT_CHECK",
            "BEGIN",
            "CUSTOM_PLANS_SQL",
            "METRIC_WIDGET_STATEMENT_TIMEOUT_SQL",
            "PROJECTION_LOCK_SQL",
            "METRIC_WIDGET_RESOLUTION_SQL",
            "METRIC_WIDGET_HISTORY_PAIRS_SQL",
            "ROLLBACK",
        ]
    );
    let outcome = outcome_line(&lines);
    assert_eq!(outcome.step, WidgetStep::Pairs);
    assert!(outcome.doi.is_some());
    let message = outcome.message();
    assert!(
        message.contains("grain=YEAR history=true buckets=-"),
        "{message}"
    );
    // A given startDate on or after 2000-01-01 reads nothing before it.
    let widget = read(&fx.pool, &year_input_from(BOOK_BARE, d(2000, 1, 1), end))
        .expect("an explicit start is served");
    assert_eq!(widget.start_date, d(2000, 1, 1));
    assert_eq!(
        total(&widget.work, fx.platform_id, fx.sessions),
        Some("1".to_string())
    );
}

// -- The uniform refusal with the new inputs (R11 section 4.8) -----------

#[test]
fn an_unknown_doi_and_an_unentitled_publisher_refuse_an_all_history_read_after_resolution() {
    let (_guard, fx) = widget_setup();
    let obelisk = Uuid::new_v4();
    let obelisk_work = Uuid::new_v4();
    exec(
        &fx.pool,
        &format!(
            "{}{}",
            publisher_sql(obelisk, "OBELISK"),
            works_sql(obelisk, &[obelisk_work])
        ),
    );
    set_doi(
        &fx.pool,
        obelisk_work,
        "https://doi.org/10.5555/widget-obelisk",
    );
    // Both works hold history, so a read that went further would run the
    // history statement.
    for work in [fx.works[0], obelisk_work] {
        commit(&fx, work, fx.platform_id, fx.sessions, d(2025, 3, 2), 1);
    }
    apply_all(&fx.pool);
    let expected = [
        "CHECKOUT_CHECK",
        "BEGIN",
        "CUSTOM_PLANS_SQL",
        "METRIC_WIDGET_STATEMENT_TIMEOUT_SQL",
        "PROJECTION_LOCK_SQL",
        "METRIC_WIDGET_RESOLUTION_SQL",
        "ROLLBACK",
    ];
    let mut refusals = Vec::new();
    let mut reasons = Vec::new();
    for doi in ["10.5555/widget-unknown", "10.5555/widget-obelisk"] {
        let (result, lines, names) = captured(&year_input(doi, d(2026, 4, 1)), &profile(&no_hook));
        let error = result.map(|_| ()).expect_err("refused");
        assert_eq!(error, MetricWidgetError::Unavailable, "{doi}");
        assert_eq!(names, expected, "{doi}");
        assert!(
            !names.contains(&"METRIC_WIDGET_HISTORY_PAIRS_SQL"),
            "{doi}: {names:?}"
        );
        refusals.push((error.code(), error.message()));
        reasons.push(
            lines
                .iter()
                .find(|line| line.kind == WidgetLineKind::Reason)
                .expect("a reason line")
                .reason,
        );
    }
    assert_eq!(refusals[0], refusals[1]);
    assert_eq!(
        refusals[0],
        (
            MetricWidgetError::Unavailable.code(),
            MetricWidgetError::Unavailable.message()
        )
    );
    assert_eq!(
        reasons,
        [
            Some(WidgetReason::UnknownWork),
            Some(WidgetReason::NotEntitled)
        ]
    );
    // The entitled book's all-history read runs the history statement.
    let (result, _, names) = captured(&year_input(BOOK_BARE, d(2026, 4, 1)), &profile(&no_hook));
    assert_eq!(result.expect("served").start_date, d(2025, 3, 1));
    assert!(names.contains(&"METRIC_WIDGET_HISTORY_PAIRS_SQL"));
}

// -- Statement sequences (section 4.7) -------------------------------------

#[test]
fn an_all_history_read_of_whole_years_reads_years_only_in_both_engine_runs() {
    let (_guard, fx) = widget_setup();
    two_section_book(&fx, &[d(2025, 1, 6), d(2025, 7, 9)]);
    let input = year_input(BOOK_BARE, d(2026, 1, 1));
    let (result, lines, names) = captured(&input, &profile(&no_hook));
    let widget = result.expect("served");
    assert_eq!(widget.start_date, d(2025, 1, 1));
    assert_eq!(
        names,
        two_section_sequence("METRIC_WIDGET_HISTORY_PAIRS_SQL", "MONTHS_YEAR_ONLY_SQL")
    );
    assert_the_ceiling(&names);
    let message = outcome_line(&lines).message();
    assert!(
        message.contains(
            "work_pairs/cross=2/2 chapters_pairs/cross=2/2 grain=YEAR history=true buckets=1"
        ),
        "{message}"
    );
    history_and_explicit(&fx.pool, &input);
}

#[test]
fn an_all_history_read_with_a_whole_year_and_clipped_ends_reads_months_and_years() {
    let (_guard, fx) = widget_setup();
    two_section_book(&fx, &[d(2024, 3, 2), d(2025, 6, 2), d(2026, 3, 2)]);
    let input = year_input(BOOK_BARE, d(2026, 4, 1));
    let (result, lines, names) = captured(&input, &profile(&no_hook));
    let widget = result.expect("served");
    assert_eq!(widget.start_date, d(2024, 3, 1));
    assert_eq!(
        names,
        two_section_sequence("METRIC_WIDGET_HISTORY_PAIRS_SQL", "MONTHS_YEAR_SQL")
    );
    assert_the_ceiling(&names);
    assert!(outcome_line(&lines)
        .message()
        .contains("grain=YEAR history=true buckets=3"));
    history_and_explicit(&fx.pool, &input);
}

#[test]
fn an_all_history_read_crossing_one_january_reads_months_only() {
    let (_guard, fx) = widget_setup();
    two_section_book(&fx, &[d(2025, 11, 4), d(2026, 2, 3)]);
    let input = year_input(BOOK_BARE, d(2026, 4, 1));
    let (result, lines, names) = captured(&input, &profile(&no_hook));
    let widget = result.expect("served");
    assert_eq!(widget.start_date, d(2025, 11, 1));
    assert_eq!(
        names,
        two_section_sequence("METRIC_WIDGET_HISTORY_PAIRS_SQL", "MONTHS_SQL")
    );
    assert_the_ceiling(&names);
    assert!(outcome_line(&lines)
        .message()
        .contains("grain=YEAR history=true buckets=2"));
    history_and_explicit(&fx.pool, &input);
}

#[test]
fn a_year_read_with_a_start_runs_the_month_bounded_pair_statement() {
    let (_guard, fx) = widget_setup();
    two_section_book(&fx, &[d(2024, 3, 2), d(2025, 6, 2), d(2026, 3, 2)]);
    let input = year_input_from(BOOK_BARE, d(2024, 1, 1), d(2026, 4, 1));
    let (result, lines, names) = captured(&input, &profile(&no_hook));
    let widget = result.expect("served");
    assert_eq!(widget.start_date, d(2024, 1, 1));
    assert_year_buckets(&widget, d(2024, 1, 1), d(2026, 4, 1));
    assert_eq!(
        names,
        two_section_sequence("METRIC_WIDGET_PAIRS_SQL", "MONTHS_YEAR_SQL")
    );
    assert_the_ceiling(&names);
    assert!(outcome_line(&lines)
        .message()
        .contains("grain=YEAR history=false buckets=3"));
}

#[test]
fn a_five_hundred_child_book_read_without_a_start_keeps_its_sequence() {
    let (_guard, fx) = widget_setup();
    let imprint = imprint_of(&fx.pool, fx.works[0]);
    let children = add_chapters(&fx.pool, fx.works[0], imprint, 500);
    for work in [fx.works[0], children[0], children[499]] {
        commit(&fx, work, fx.platform_id, fx.sessions, d(2025, 11, 4), 1);
        commit(&fx, work, fx.platform_id, fx.units, d(2026, 2, 3), 1);
    }
    apply_all(&fx.pool);
    let (result, _, names) = captured(&year_input(BOOK_BARE, d(2026, 4, 1)), &profile(&no_hook));
    let widget = result.expect("500 children are served");
    assert_eq!(
        total(
            widget.chapters.as_ref().expect("chapters"),
            fx.platform_id,
            fx.sessions
        ),
        Some("2".to_string())
    );
    assert_eq!(
        names,
        two_section_sequence("METRIC_WIDGET_HISTORY_PAIRS_SQL", "MONTHS_SQL")
    );
    assert_the_ceiling(&names);
}

#[test]
fn an_all_history_read_of_two_whole_years_reads_years_only_in_both_runs() {
    let (_guard, fx) = widget_setup();
    // Values from January 2024, so the window holds two whole calendar
    // years.
    let (first, second) = two_chapters(&fx);
    for day in [d(2024, 1, 9), d(2025, 6, 2)] {
        for work in [fx.works[0], first, second] {
            commit(&fx, work, fx.platform_id, fx.sessions, day, 4);
            commit(&fx, work, fx.platform_id, fx.units, day, 2);
        }
    }
    apply_all(&fx.pool);
    let (start, end) = (d(2024, 1, 1), d(2026, 1, 1));
    let input = year_input(BOOK_BARE, end);
    let (result, lines, names) = captured(&input, &profile(&no_hook));
    let widget = result.expect("served");
    assert_eq!(widget.start_date, start);
    assert_eq!(
        names,
        two_section_sequence("METRIC_WIDGET_HISTORY_PAIRS_SQL", "MONTHS_YEAR_ONLY_SQL")
    );
    assert_the_ceiling(&names);
    assert!(outcome_line(&lines)
        .message()
        .contains("grain=YEAR history=true buckets=2"));
    assert_eq!(
        year_buckets(start, end),
        [(start, d(2025, 1, 1)), (d(2025, 1, 1), end)]
    );
    assert_year_buckets(&widget, start, end);
    assert_eq!(
        values_of(&widget.work, fx.platform_id, fx.sessions),
        [Some("4".to_string()), Some("4".to_string())]
    );
    assert_eq!(
        values_of(
            widget.chapters.as_ref().expect("chapters"),
            fx.platform_id,
            fx.units
        ),
        [Some("4".to_string()), Some("4".to_string())]
    );
    // The explicit-start run at the served start reads years only too, and
    // gives the same response.
    let (result, lines, names) =
        captured(&year_input_from(BOOK_BARE, start, end), &profile(&no_hook));
    let explicit = result.expect("served");
    assert_eq!(
        names,
        two_section_sequence("METRIC_WIDGET_PAIRS_SQL", "MONTHS_YEAR_ONLY_SQL")
    );
    assert_the_ceiling(&names);
    assert!(outcome_line(&lines)
        .message()
        .contains("grain=YEAR history=false buckets=2"));
    assert_eq!(without_as_of(widget.clone()), without_as_of(explicit));
    history_and_explicit(&fx.pool, &input);
    assert_year_oracle(&fx, &widget, &[first, second]);
}

// -- Equivalence oracle (section 8) ----------------------------------------

/// The oracle over every section of `widget`, at `YEAR` over its served
/// range.
fn assert_year_oracle(fx: &Fixture, widget: &MetricWidget, chapters: &[Uuid]) {
    let range = (widget.start_date, widget.end_date);
    assert_each_served_pair_equals_its_dashboard_read_at(
        fx,
        &widget.work,
        &[fx.works[0]],
        range,
        MetricTimelineGrain::Year,
    );
    if let Some(section) = &widget.chapters {
        assert_each_served_pair_equals_its_dashboard_read_at(
            fx,
            section,
            chapters,
            range,
            MetricTimelineGrain::Year,
        );
    }
}

#[test]
fn each_served_pair_of_a_mixed_all_history_window_equals_its_single_pair_read() {
    let (_guard, fx) = widget_setup();
    let (first, second) = two_chapters(&fx);
    let (book, platform, other) = (fx.works[0], fx.platform_id, fx.other_platform_id);
    let end = d(2026, 4, 1);
    // The chapters start before the book.
    commit_country(&fx, first, platform, fx.sessions, d(2024, 3, 5), "GB", 2);
    commit(&fx, second, platform, fx.sessions, d(2025, 2, 2), 3);
    // The book's sessions, with countries in a clipped first year (monthly
    // projection), a whole year (yearly projection) and the clipped last year.
    commit_country(&fx, book, platform, fx.sessions, d(2024, 5, 3), "GB", 4);
    commit_country(&fx, book, platform, fx.sessions, d(2025, 7, 9), "FR", 6);
    commit_country(&fx, book, platform, fx.sessions, d(2026, 2, 11), "DE", 1);
    // A pair with "0" years: completely covered, values only in 2026.
    cover_over(&fx, fx.units, d(2024, 3, 1), end);
    commit(&fx, book, platform, fx.units, d(2026, 3, 4), 7);
    // A pair with null years: uncovered, values only in 2026.
    commit(&fx, book, other, fx.units, d(2026, 3, 4), 5);
    apply_all(&fx.pool);

    let input = year_input(BOOK_BARE, end);
    let (result, _, names) = captured(&input, &profile(&no_hook));
    let widget = result.expect("served");
    assert_eq!(
        names
            .iter()
            .filter(|name| **name == "MONTHS_YEAR_SQL")
            .count(),
        2
    );
    assert_eq!(widget.start_date, d(2024, 3, 1));
    assert_year_buckets(&widget, d(2024, 3, 1), end);
    assert_eq!(
        values_of(&widget.work, platform, fx.units),
        [
            Some("0".to_string()),
            Some("0".to_string()),
            Some("7".to_string())
        ]
    );
    assert_eq!(
        values_of(&widget.work, other, fx.units),
        [None, None, Some("5".to_string())]
    );
    assert_eq!(
        values_of(&widget.work, platform, fx.sessions),
        [
            Some("4".to_string()),
            Some("6".to_string()),
            Some("1".to_string())
        ]
    );
    let countries: Vec<&str> = widget
        .work
        .countries
        .iter()
        .map(|row| row.country_code.as_str())
        .collect();
    assert_eq!(countries, ["DE", "FR", "GB"]);
    assert_year_oracle(&fx, &widget, &[first, second]);
    // The same with startDate set to the served start, and with an earlier
    // one.
    history_and_explicit(&fx.pool, &input);
    let earlier = read(&fx.pool, &year_input_from(BOOK_BARE, d(2023, 1, 1), end))
        .expect("an earlier start is served");
    assert_year_oracle(&fx, &earlier, &[first, second]);
}

#[test]
fn each_served_pair_of_an_all_history_whole_year_equals_its_single_pair_read() {
    let (_guard, fx) = widget_setup();
    let end = d(2026, 1, 1);
    commit_country(
        &fx,
        fx.works[0],
        fx.platform_id,
        fx.sessions,
        d(2025, 1, 6),
        "GB",
        3,
    );
    commit_country(
        &fx,
        fx.works[0],
        fx.platform_id,
        fx.sessions,
        d(2025, 7, 9),
        "FR",
        2,
    );
    commit(
        &fx,
        fx.works[0],
        fx.other_platform_id,
        fx.units,
        d(2025, 5, 1),
        5,
    );
    cover_over(&fx, fx.sessions, d(2025, 1, 1), end);
    apply_all(&fx.pool);
    let input = year_input(BOOK_BARE, end);
    let (result, _, names) = captured(&input, &profile(&no_hook));
    let widget = result.expect("served");
    assert!(names.contains(&"MONTHS_YEAR_ONLY_SQL"), "{names:?}");
    assert_eq!(widget.start_date, d(2025, 1, 1));
    assert_year_buckets(&widget, d(2025, 1, 1), end);
    assert_eq!(widget.work.countries.len(), 2);
    assert_year_oracle(&fx, &widget, &[]);
    history_and_explicit(&fx.pool, &input);
}

#[test]
fn each_served_pair_of_an_all_history_read_across_one_january_equals_its_single_pair_read() {
    let (_guard, fx) = widget_setup();
    let end = d(2026, 4, 1);
    commit_country(
        &fx,
        fx.works[0],
        fx.platform_id,
        fx.sessions,
        d(2025, 11, 4),
        "GB",
        3,
    );
    commit_country(
        &fx,
        fx.works[0],
        fx.platform_id,
        fx.sessions,
        d(2026, 2, 3),
        "FR",
        2,
    );
    cover_over(&fx, fx.sessions, d(2025, 11, 1), end);
    apply_all(&fx.pool);
    let input = year_input(BOOK_BARE, end);
    let (result, _, names) = captured(&input, &profile(&no_hook));
    let widget = result.expect("served");
    assert!(names.contains(&"MONTHS_SQL"), "{names:?}");
    assert_eq!(widget.start_date, d(2025, 11, 1));
    assert_eq!(
        buckets_of(&widget.work, fx.platform_id, fx.sessions),
        [(d(2025, 11, 1), d(2026, 1, 1)), (d(2026, 1, 1), end)]
    );
    assert_eq!(
        values_of(&widget.work, fx.platform_id, fx.sessions),
        [Some("3".to_string()), Some("2".to_string())]
    );
    assert_eq!(widget.work.countries.len(), 2);
    assert_eq!(
        widget.work.coverage[0].status,
        MetricCoverageStatus::Complete
    );
    assert_year_oracle(&fx, &widget, &[]);
    history_and_explicit(&fx.pool, &input);
}

#[test]
fn an_all_history_read_reaching_past_today_carries_no_coverage_signal() {
    let (_guard, fx) = widget_setup();
    // The profile's today is 2026-11-10; coverage asserts no later day.
    let end = d(2026, 12, 1);
    commit(
        &fx,
        fx.works[0],
        fx.platform_id,
        fx.sessions,
        d(2026, 3, 2),
        3,
    );
    cover_over(&fx, fx.sessions, d(2026, 3, 1), d(2026, 11, 1));
    apply_all(&fx.pool);
    let widget = history_and_explicit(&fx.pool, &year_input(BOOK_BARE, end));
    assert_eq!(widget.start_date, d(2026, 3, 1));
    let coverage = &widget.work.coverage[0];
    assert_eq!(coverage.status, MetricCoverageStatus::Unknown);
    assert!(!coverage.country_coverage);
    assert!(widget.work.is_partial);
    assert_year_oracle(&fx, &widget, &[]);
}

// -- Native grain and ambiguity (section 4.3) ------------------------------

#[test]
fn a_native_record_ending_on_the_served_start_does_not_refuse_the_read() {
    let (_guard, fx) = widget_setup();
    let end = d(2026, 4, 1);
    commit(
        &fx,
        fx.works[0],
        fx.platform_id,
        fx.sessions,
        d(2025, 11, 3),
        4,
    );
    commit(
        &fx,
        fx.works[0],
        fx.platform_id,
        fx.sessions,
        d(2026, 2, 3),
        2,
    );
    apply_all(&fx.pool);
    native_record_over(
        &fx,
        fx.works[0],
        fx.platform_id,
        fx.sessions,
        (d(2025, 10, 1), d(2025, 11, 1)),
    );
    let widget = history_and_explicit(&fx.pool, &year_input(BOOK_BARE, end));
    assert_eq!(widget.start_date, d(2025, 11, 1));
    assert_eq!(
        total(&widget.work, fx.platform_id, fx.sessions),
        Some("6".to_string())
    );
    // The same record with an explicit startDate before it refuses, for
    // its native grain.
    let (result, lines) = resolve_with(
        &fx.pool,
        MetricWidgetSwitch::On,
        &year_input_from(BOOK_BARE, d(2025, 1, 1), end),
        &profile(&no_hook),
    );
    assert_eq!(result, Err(MetricWidgetError::Unavailable));
    assert!(lines
        .iter()
        .any(|line| line.reason == Some(WidgetReason::UnsupportedSourceGrain)));
}

#[test]
fn a_native_record_in_the_first_served_month_on_a_cross_pair_refuses_the_read() {
    let (_guard, fx) = widget_setup();
    for day in [d(2025, 3, 4), d(2026, 3, 4)] {
        commit(&fx, fx.works[0], fx.platform_id, fx.sessions, day, 3);
        commit(&fx, fx.works[0], fx.other_platform_id, fx.units, day, 5);
    }
    apply_all(&fx.pool);
    native_record_over(
        &fx,
        fx.works[0],
        fx.platform_id,
        fx.units,
        (d(2025, 3, 1), d(2025, 4, 1)),
    );
    let (result, lines) = resolve_with(
        &fx.pool,
        MetricWidgetSwitch::On,
        &year_input(BOOK_BARE, d(2026, 4, 1)),
        &profile(&no_hook),
    );
    assert_eq!(result, Err(MetricWidgetError::Unavailable));
    assert!(lines
        .iter()
        .any(|line| line.reason == Some(WidgetReason::UnsupportedSourceGrain)));
    // A MONTH read of the following year is served.
    let widget = read(
        &fx.pool,
        &input_over(BOOK_BARE, (d(2026, 1, 1), d(2027, 1, 1))),
    )
    .expect("served");
    assert_eq!(pairs_of(&widget.work).len(), 2);
}

#[test]
fn native_history_on_one_served_platform_overlapping_another_platforms_values_refuses() {
    let (_guard, fx) = widget_setup();
    // Day-grain values from July on one platform; native records to
    // September on another served platform, for a served measure.
    commit(
        &fx,
        fx.works[0],
        fx.platform_id,
        fx.sessions,
        d(2025, 7, 2),
        3,
    );
    commit(
        &fx,
        fx.works[0],
        fx.other_platform_id,
        fx.units,
        d(2025, 10, 2),
        5,
    );
    apply_all(&fx.pool);
    let input = year_input(BOOK_BARE, d(2026, 4, 1));
    // The control: without the native records the same fixture is served
    // from July.
    let widget = history_and_explicit(&fx.pool, &input);
    assert_eq!(widget.start_date, d(2025, 7, 1));
    assert_eq!(
        sorted(&pairs_of(&widget.work)),
        sorted(&[
            (fx.platform_id, fx.sessions),
            (fx.other_platform_id, fx.units)
        ])
    );
    for month in [8, 9] {
        native_record_over(
            &fx,
            fx.works[0],
            fx.other_platform_id,
            fx.sessions,
            (d(2025, month, 1), d(2025, month + 1, 1)),
        );
    }
    let (result, lines) =
        resolve_with(&fx.pool, MetricWidgetSwitch::On, &input, &profile(&no_hook));
    assert_eq!(result, Err(MetricWidgetError::Unavailable));
    assert!(lines
        .iter()
        .any(|line| line.reason == Some(WidgetReason::UnsupportedSourceGrain)));
}

#[test]
fn native_history_of_the_work_overlapping_the_chapters_start_refuses_the_whole_read() {
    let (_guard, fx) = widget_setup();
    let (first, second) = two_chapters(&fx);
    let end = d(2026, 4, 1);
    commit(&fx, first, fx.platform_id, fx.sessions, d(2024, 2, 5), 1);
    commit(&fx, second, fx.platform_id, fx.sessions, d(2025, 6, 5), 2);
    commit(
        &fx,
        fx.works[0],
        fx.platform_id,
        fx.sessions,
        d(2025, 6, 5),
        3,
    );
    apply_all(&fx.pool);
    native_record_over(
        &fx,
        fx.works[0],
        fx.platform_id,
        fx.sessions,
        (d(2025, 3, 1), d(2025, 4, 1)),
    );
    // The first chapter starts the read in February 2024, before the
    // work's native record ends: the work section, so the read, is refused.
    let (result, lines) = resolve_with(
        &fx.pool,
        MetricWidgetSwitch::On,
        &year_input(BOOK_BARE, end),
        &profile(&no_hook),
    );
    assert_eq!(result, Err(MetricWidgetError::Unavailable));
    assert!(lines
        .iter()
        .any(|line| line.reason == Some(WidgetReason::UnsupportedSourceGrain)));
    // Naming only the chapter whose values start after the record ends.
    let widget = history_and_explicit(
        &fx.pool,
        &year_input_with_chapters(BOOK_BARE, end, &["10.5555/widget-chapter-2"]),
    );
    assert_eq!(widget.start_date, d(2025, 6, 1));
    assert_eq!(
        total(
            widget.chapters.as_ref().expect("chapters"),
            fx.platform_id,
            fx.sessions
        ),
        Some("2".to_string())
    );
}

#[test]
fn an_ambiguous_month_years_before_the_end_refuses_the_all_history_read() {
    let (_guard, fx) = widget_setup();
    let institution = insert_institution(&fx.pool);
    for dims in [
        Dims {
            country: Some("GB"),
            ..Dims::default()
        },
        Dims {
            institution: Some(institution),
            ..Dims::default()
        },
    ] {
        commit_dims(
            &fx,
            fx.works[0],
            fx.platform_id,
            fx.sessions,
            d(2023, 5, 2),
            dims,
            1,
        );
    }
    commit(
        &fx,
        fx.works[0],
        fx.platform_id,
        fx.sessions,
        d(2026, 3, 2),
        4,
    );
    apply_all(&fx.pool);
    let (result, lines) = resolve_with(
        &fx.pool,
        MetricWidgetSwitch::On,
        &year_input(BOOK_BARE, d(2026, 4, 1)),
        &profile(&no_hook),
    );
    assert_eq!(result, Err(MetricWidgetError::Unavailable));
    assert!(lines
        .iter()
        .any(|line| line.reason == Some(WidgetReason::DimensionScopeAmbiguous)));
    let widget = read(
        &fx.pool,
        &input_over(BOOK_BARE, (d(2026, 1, 1), d(2027, 1, 1))),
    )
    .expect("a MONTH read of a later year is served");
    assert_eq!(
        total(&widget.work, fx.platform_id, fx.sessions),
        Some("4".to_string())
    );
}

#[test]
fn a_country_ambiguous_month_years_before_the_end_refuses_the_all_history_read() {
    #[derive(diesel::QueryableByName)]
    struct Ambiguity {
        #[diesel(sql_type = Date)]
        month_start: NaiveDate,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        total_ambiguous: bool,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        country_ambiguous: bool,
    }
    let (_guard, fx) = widget_setup();
    let pdf = insert_publication(&fx.pool, fx.works[0], "PDF");
    let institution = insert_institution(&fx.pool);
    // An earlier value, so the ambiguous month lies inside a whole year of
    // the served range.
    commit(
        &fx,
        fx.works[0],
        fx.platform_id,
        fx.sessions,
        d(2022, 11, 3),
        1,
    );
    // April 2023: an aggregate row and two incomparable country-bearing
    // breakdowns, so country-ambiguous with no total ambiguity.
    for (dims, value) in [
        (Dims::default(), 10),
        (
            Dims {
                country: Some("GB"),
                institution: Some(institution),
                publication: None,
            },
            4,
        ),
        (
            Dims {
                publication: Some(pdf),
                country: Some("US"),
                institution: None,
            },
            6,
        ),
    ] {
        commit_dims(
            &fx,
            fx.works[0],
            fx.platform_id,
            fx.sessions,
            d(2023, 4, 10),
            dims,
            value,
        );
    }
    commit(
        &fx,
        fx.works[0],
        fx.platform_id,
        fx.sessions,
        d(2026, 3, 2),
        4,
    );
    apply_all(&fx.pool);
    let ambiguity: Vec<Ambiguity> = {
        let mut connection = fx.pool.get().expect("a connection");
        diesel::sql_query(format!(
            "SELECT month_start, total_ambiguous, country_ambiguous \
               FROM metric_rollup_work_month_ambiguity WHERE work_id = '{}'",
            fx.works[0]
        ))
        .load(&mut connection)
        .expect("the ambiguity rows")
    };
    assert_eq!(ambiguity.len(), 1);
    assert_eq!(ambiguity[0].month_start, d(2023, 4, 1));
    assert!(ambiguity[0].country_ambiguous);
    assert!(!ambiguity[0].total_ambiguous);
    let (result, lines) = resolve_with(
        &fx.pool,
        MetricWidgetSwitch::On,
        &year_input(BOOK_BARE, d(2026, 4, 1)),
        &profile(&no_hook),
    );
    assert_eq!(result, Err(MetricWidgetError::Unavailable));
    assert!(lines
        .iter()
        .any(|line| line.reason == Some(WidgetReason::DimensionScopeAmbiguous)));
    let widget = read(
        &fx.pool,
        &input_over(BOOK_BARE, (d(2026, 1, 1), d(2027, 1, 1))),
    )
    .expect("a MONTH read of a later year is served");
    assert_eq!(
        total(&widget.work, fx.platform_id, fx.sessions),
        Some("4".to_string())
    );
}

// -- Bounds (section 4.5) --------------------------------------------------

#[test]
fn forty_pairs_over_all_history_are_served_and_forty_one_refused_before_the_start() {
    let (_guard, fx) = widget_setup();
    let mut measures = vec![fx.sessions, fx.units];
    for n in 0..38 {
        measures.push(insert_measure(
            &fx.pool,
            &format!("widget_history_bound_{n}"),
            true,
            true,
        ));
    }
    for (n, measure) in measures.iter().enumerate() {
        let year = 2000 + i32::try_from(n % 27).expect("a year offset");
        commit(&fx, fx.works[0], fx.platform_id, *measure, d(year, 1, 3), 1);
    }
    apply_all(&fx.pool);
    let widget = history_and_explicit(&fx.pool, &year_input(BOOK_BARE, d(2026, 4, 1)));
    assert_eq!(widget.start_date, d(2000, 1, 1));
    assert_eq!(widget.work.totals.len(), 40);
    assert_eq!(widget.work.timeline.len(), 40 * 27);
    // A YEAR read from 2000-01-01 to the latest endDate: 28 years, 1,120
    // cells.
    let widest = read(
        &fx.pool,
        &year_input_from(BOOK_BARE, d(2000, 1, 1), d(2027, 11, 1)),
    )
    .expect("the widest YEAR read is served");
    assert_eq!(widest.work.timeline.len(), 40 * 28);

    // A 41st pair, whose value predates 2000: refused at the pair bound,
    // before the served start (so not at the floor) and before labels.
    commit(
        &fx,
        fx.works[0],
        fx.other_platform_id,
        fx.sessions,
        d(1999, 6, 1),
        1,
    );
    apply_all(&fx.pool);
    let (result, lines, names) =
        captured(&year_input(BOOK_BARE, d(2026, 4, 1)), &profile(&no_hook));
    assert_eq!(
        result,
        Err(MetricWidgetError::QueryLimitExceeded(
            TOO_MANY_PAIRS_MESSAGE
        ))
    );
    assert!(
        names.ends_with(&["METRIC_WIDGET_HISTORY_PAIRS_SQL", "ROLLBACK"]),
        "{names:?}"
    );
    assert!(!names.contains(&"METRIC_WIDGET_LABELS_SQL"));
    let message = outcome_line(&lines).message();
    assert!(
        message.contains("work_pairs/cross=41/") && message.contains("buckets=-"),
        "{message}"
    );
}
