//! `MET-WP4-02` protected Metrics read-service evidence at the API boundary.
//!
//! These tests exercise the real production schema and the real resolvers
//! against a disposable database: the three approved read operations, their
//! exact `METRICS_READ_SERVICE` authorization and complete negative matrix,
//! the fact that denial precedes every Metrics read, the separate
//! `METRICS_DASHBOARD` publisher entitlement, the protected registry lists and
//! their bound, the frozen error classifications, `BigInt` transport, the
//! strictly additive SDL and unchanged unrelated behaviour.
//!
//! Coverage, freshness, snapshot, bounds and additivity semantics are proven
//! with the read itself, in `crate::model::metric_dashboard::tests`.

#![cfg(all(test, feature = "backend"))]

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::{json, Value as JsonValue};
use uuid::Uuid;
use zitadel::actix::introspection::IntrospectedUser;

use super::sdl_support::sdl_block;
use super::{create_schema, Context, GraphQLRequest, Schema};
use crate::db::PgPool;
use crate::model::metric_dashboard::tests::{
    apply_all, commit, commit_dims, cover, day_n, exec, insert_institution, insert_measure,
    projection_sql, setup, Dims, Fixture,
};
use crate::model::metric_platform::tests::{insert_platform_row, setup_registry_db};
use crate::model::tests::db as test_db;
use crate::policy::Role;

// --------------------------------------------------------------------------
// Execution helpers
// --------------------------------------------------------------------------

async fn run(schema: &Schema, context: &Context, query: &str, variables: JsonValue) -> JsonValue {
    let request: GraphQLRequest =
        serde_json::from_value(json!({ "query": query, "variables": variables }))
            .expect("build GraphQL request");
    serde_json::to_value(request.execute(schema, context).await)
        .expect("serialize GraphQL response")
}

fn data<'a>(response: &'a JsonValue, field: &str) -> &'a JsonValue {
    assert!(
        response.get("errors").is_none()
            || response["errors"].as_array().is_some_and(Vec::is_empty),
        "unexpected GraphQL errors: {response}"
    );
    &response["data"][field]
}

fn only_error(response: &JsonValue) -> (String, String) {
    let errors = response["errors"].as_array().expect("errors array");
    assert_eq!(errors.len(), 1, "expected exactly one error: {response}");
    (
        errors[0]["message"].as_str().expect("message").to_string(),
        errors[0]["extensions"]["type"]
            .as_str()
            .expect("extensions.type")
            .to_string(),
    )
}

fn assert_unauthorized(response: &JsonValue) {
    let (message, kind) = only_error(response);
    assert_eq!(
        kind, "NO_ACCESS",
        "expected a fail-closed denial: {response}"
    );
    assert_eq!(message, "Unauthorized");
    assert!(
        response["data"].is_null(),
        "a denial must carry no Metrics data, not an empty or zero result: {response}"
    );
}

fn assert_classified(response: &JsonValue, expected: &str) -> String {
    let (message, kind) = only_error(response);
    assert_eq!(kind, expected, "unexpected classification: {response}");
    assert!(
        response["data"].is_null(),
        "an error must carry no data: {response}"
    );
    let rendered = response.to_string();
    for leaked in [
        "SELECT",
        "FROM ",
        "public.",
        "metric_",
        "postgres",
        "DETAIL",
        "violates",
        "uuid[]",
        "Database error",
    ] {
        assert!(
            !rendered.contains(leaked),
            "the error leaked `{leaked}`: {rendered}"
        );
    }
    message
}

fn user_with(user_id: &str, roles: &[(Role, &str)]) -> IntrospectedUser {
    let mut project_roles: HashMap<String, HashMap<String, String>> = HashMap::new();
    for (role, org_id) in roles {
        project_roles
            .entry(role.as_ref().to_string())
            .or_default()
            .insert((*org_id).to_string(), "role".to_string());
    }
    IntrospectedUser {
        user_id: user_id.to_string(),
        username: None,
        name: None,
        given_name: None,
        family_name: None,
        preferred_username: None,
        email: None,
        email_verified: None,
        locale: None,
        project_roles: Some(project_roles),
        metadata: None,
    }
}

fn context_for(pool: &Arc<PgPool>, user: Option<IntrospectedUser>) -> Context {
    match user {
        Some(user) => test_db::test_context_with_user(Arc::clone(pool), user),
        None => test_db::test_context_anonymous(Arc::clone(pool)),
    }
}

fn reader() -> IntrospectedUser {
    user_with(
        "metrics-read-service-1",
        &[(Role::MetricsReadService, "org-1")],
    )
}

// --------------------------------------------------------------------------
// Operation documents
// --------------------------------------------------------------------------

const DASHBOARD: &str = "query Dashboard($input: MetricDashboardInput!) { \
    metricDashboard(input: $input) { \
      totals { platformId measureId value } \
      timeline { platformId measureId startDate endDate value } \
      coverage { status items { platformId measureId status dataThrough \
                                countryCoverage institutionCoverage } } \
      asOf dataThrough rollupWatermark warnings { code message } isPartial } }";

const MEASURES: &str = "{ metricMeasures { measureId code enabled additiveAcrossTime \
                            additiveAcrossWorks } }";

const PLATFORMS: &str = "{ metricPlatforms { platformId code enabled } }";

fn dashboard_variables(fx: &Fixture, publishers: JsonValue) -> JsonValue {
    json!({
        "input": {
            "selector": { "publisherIds": publishers },
            "startDate": day_n(1).to_string(),
            "endDate": day_n(5).to_string(),
            "platforms": [fx.platform_id],
            "measures": [fx.sessions],
        }
    })
}

/// Every operation as `(field, document, variables)`.
fn operations(fx: &Fixture) -> Vec<(&'static str, &'static str, JsonValue)> {
    vec![
        (
            "metricDashboard",
            DASHBOARD,
            dashboard_variables(fx, json!([fx.publisher_id])),
        ),
        ("metricMeasures", MEASURES, json!({})),
        ("metricPlatforms", PLATFORMS, json!({})),
    ]
}

/// Every caller of the approved matrix, as `(label, user, allowed)`.
fn matrix(org: &str) -> Vec<(&'static str, Option<IntrospectedUser>, bool)> {
    vec![
        ("anonymous", None, false),
        (
            "authenticated, no role",
            Some(user_with("no-roles", &[])),
            false,
        ),
        (
            "PUBLISHER_USER",
            Some(user_with("user", &[(Role::PublisherUser, org)])),
            false,
        ),
        (
            "PUBLISHER_ADMIN",
            Some(user_with("admin", &[(Role::PublisherAdmin, org)])),
            false,
        ),
        (
            "WORK_LIFECYCLE",
            Some(user_with("lifecycle", &[(Role::WorkLifecycle, org)])),
            false,
        ),
        (
            "CDN_WRITE",
            Some(user_with("cdn", &[(Role::CdnWrite, org)])),
            false,
        ),
        (
            "SUPERUSER",
            Some(user_with("root", &[(Role::Superuser, org)])),
            false,
        ),
        (
            "DISSEMINATION_WORKER",
            Some(user_with("worker", &[(Role::DisseminationWorker, org)])),
            false,
        ),
        (
            "METRICS_INGEST_SERVICE",
            Some(user_with("ingest", &[(Role::MetricsIngestService, org)])),
            false,
        ),
        (
            "every other role together, without METRICS_READ_SERVICE",
            Some(user_with(
                "everything-else",
                &[
                    (Role::Superuser, org),
                    (Role::PublisherAdmin, org),
                    (Role::PublisherUser, org),
                    (Role::WorkLifecycle, org),
                    (Role::CdnWrite, org),
                    (Role::DisseminationWorker, org),
                    (Role::MetricsIngestService, org),
                ],
            )),
            false,
        ),
        ("METRICS_READ_SERVICE", Some(reader()), true),
        (
            "METRICS_READ_SERVICE alongside unrelated roles",
            Some(user_with(
                "reader-plus",
                &[
                    (Role::MetricsReadService, org),
                    (Role::Superuser, org),
                    (Role::PublisherUser, "org-elsewhere"),
                    (Role::MetricsIngestService, org),
                ],
            )),
            true,
        ),
    ]
}

// ==========================================================================
// Authorization
// ==========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn every_row_of_the_read_service_matrix_holds_for_every_operation() {
    let (_guard, fx) = setup();
    commit(&fx, fx.works[0], fx.platform_id, fx.sessions, day_n(1), 7);
    apply_all(&fx.pool);
    let schema = create_schema();

    for (label, user, allowed) in matrix("org-1") {
        let context = context_for(&fx.pool, user);
        for (field, document, variables) in operations(&fx) {
            let response = run(&schema, &context, document, variables).await;
            if allowed {
                let served = data(&response, field);
                assert!(
                    served.is_object() || served.as_array().is_some_and(|rows| !rows.is_empty()),
                    "{label} must be served {field}: {response}"
                );
            } else {
                assert_unauthorized(&response);
            }
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn denial_is_decided_before_any_metrics_database_access() {
    // The pool is unreachable. A denied caller that reached any Metrics read
    // would see an internal error instead of the fail-closed denial.
    let unreachable = Arc::new(test_db::failing_pool());
    let schema = create_schema();
    let fx_variables = json!({
        "input": {
            "selector": { "publisherIds": [Uuid::new_v4()] },
            "startDate": "2026-03-01",
            "endDate": "2026-03-05",
        }
    });

    for (label, user, allowed) in matrix("org-1") {
        let context = context_for(&unreachable, user);
        for (field, document, variables) in [
            ("metricDashboard", DASHBOARD, fx_variables.clone()),
            ("metricMeasures", MEASURES, json!({})),
            ("metricPlatforms", PLATFORMS, json!({})),
        ] {
            let response = run(&schema, &context, document, variables).await;
            if allowed {
                // The authorized caller does reach the database, which fails
                // closed with a sanitized error and no data.
                let message = assert_classified(&response, "INTERNAL_ERROR");
                assert_eq!(
                    message, "The Metrics read could not be completed.",
                    "{field}"
                );
            } else {
                assert_unauthorized(&response);
                assert!(response["data"].is_null(), "{label}: {field}");
            }
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn the_read_role_confers_no_other_authority() {
    let (_guard, fx) = setup();
    let schema = create_schema();
    let context = context_for(&fx.pool, Some(reader()));

    for document in [
        "{ metricPlatformByCode(code: \"dashboard_platform_a\") { platformId } }",
        "{ metricMeasureByCode(code: \"title_sessions\") { measureId } }",
        "{ metricSourceByCode(code: \"any\") { sourceId } }",
        "{ metricSourceAccountByCode(code: \"any\") { sourceAccountId } }",
        "mutation { createMetricPlatform(data: { code: \"x\", displayName: \"X\", \
           ownershipClass: EXTERNAL, enabled: true }) { platformId } }",
        "mutation { claimMetricRollupDeltas(limit: 1) { deltaId } }",
    ] {
        assert_unauthorized(&run(&schema, &context, document, json!({})).await);
    }
}

// ==========================================================================
// Publisher entitlement and the MOM-1 selector
// ==========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn the_dashboard_requires_publisher_entitlement_that_no_role_supplies() {
    let (_guard, fx) = setup();
    commit(&fx, fx.works[0], fx.platform_id, fx.sessions, day_n(1), 7);
    apply_all(&fx.pool);
    let schema = create_schema();
    let variables = dashboard_variables(&fx, json!([fx.publisher_id]));

    let served = run(
        &schema,
        &context_for(&fx.pool, Some(reader())),
        DASHBOARD,
        variables.clone(),
    )
    .await;
    assert_eq!(
        data(&served, "metricDashboard")["totals"][0]["value"],
        json!("7")
    );

    exec(
        &fx.pool,
        &format!(
            "UPDATE publisher SET subscription_package = 'OBELISK' WHERE publisher_id = '{}';",
            fx.publisher_id
        ),
    );
    for (label, user) in [
        ("read service", reader()),
        (
            "read service with every other role",
            user_with(
                "reader-plus",
                &[
                    (Role::MetricsReadService, "org-1"),
                    (Role::Superuser, "org-1"),
                    (Role::PublisherAdmin, "org-1"),
                    (Role::MetricsIngestService, "org-1"),
                ],
            ),
        ),
    ] {
        let response = run(
            &schema,
            &context_for(&fx.pool, Some(user)),
            DASHBOARD,
            variables.clone(),
        )
        .await;
        assert_unauthorized(&response);
        assert!(response["data"].is_null(), "{label}");
    }

    // Registry discovery needs no publisher and no entitlement.
    let context = context_for(&fx.pool, Some(reader()));
    assert!(data(
        &run(&schema, &context, MEASURES, json!({})).await,
        "metricMeasures"
    )
    .is_array());
    assert!(data(
        &run(&schema, &context, PLATFORMS, json!({})).await,
        "metricPlatforms"
    )
    .is_array());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn the_plural_selector_accepts_exactly_one_publisher_in_mom_1() {
    let (_guard, fx) = setup();
    let schema = create_schema();
    let context = context_for(&fx.pool, Some(reader()));

    let one = run(
        &schema,
        &context,
        DASHBOARD,
        dashboard_variables(&fx, json!([fx.publisher_id])),
    )
    .await;
    assert!(data(&one, "metricDashboard").is_object());

    // MOM-1 runtime cardinality: none, several and unknown are all refused
    // with a query-validation error, never truncated, combined or emptied.
    for (label, publishers) in [
        ("omitted", JsonValue::Null),
        ("empty", json!([])),
        ("two", json!([fx.publisher_id, fx.other_publisher_id])),
        ("unknown", json!([Uuid::new_v4()])),
    ] {
        let response = run(
            &schema,
            &context,
            DASHBOARD,
            dashboard_variables(&fx, publishers),
        )
        .await;
        let message = assert_classified(&response, "METRIC_QUERY_INVALID");
        assert!(message.contains("publisher"), "{label}: {message}");
    }
}

// ==========================================================================
// Error classifications
// ==========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn every_bounded_failure_carries_its_frozen_classification() {
    let (_guard, fx) = setup();
    let non_additive = insert_measure(&fx.pool, "non_additive", false, true);
    let schema = create_schema();
    let context = context_for(&fx.pool, Some(reader()));
    let with = |changes: JsonValue| {
        let mut variables = dashboard_variables(&fx, json!([fx.publisher_id]));
        for (key, value) in changes.as_object().expect("an object") {
            variables["input"][key] = value.clone();
        }
        variables
    };

    for (label, variables, expected) in [
        (
            "reversed dates",
            with(json!({ "endDate": day_n(1).to_string(), "startDate": day_n(2).to_string() })),
            "METRIC_QUERY_INVALID",
        ),
        (
            "duplicate measures",
            with(json!({ "measures": [fx.sessions, fx.sessions] })),
            "METRIC_QUERY_INVALID",
        ),
        (
            "duplicate platforms",
            with(json!({ "platforms": [fx.platform_id, fx.platform_id] })),
            "METRIC_QUERY_INVALID",
        ),
        (
            "unknown measure",
            with(json!({ "measures": [Uuid::new_v4()] })),
            "METRIC_QUERY_INVALID",
        ),
        (
            "unknown platform",
            with(json!({ "platforms": [Uuid::new_v4()] })),
            "METRIC_QUERY_INVALID",
        ),
        (
            "non-additive measure",
            with(json!({ "measures": [non_additive] })),
            "METRIC_QUERY_INVALID",
        ),
        (
            "367 days",
            with(json!({ "endDate": (day_n(1) + chrono::Duration::days(367)).to_string() })),
            "METRIC_QUERY_LIMIT_EXCEEDED",
        ),
        (
            "11 measures",
            with(json!({ "measures": (0..11).map(|_| Uuid::new_v4()).collect::<Vec<_>>() })),
            "METRIC_QUERY_LIMIT_EXCEEDED",
        ),
    ] {
        let response = run(&schema, &context, DASHBOARD, variables).await;
        let message = assert_classified(&response, expected);
        assert!(!message.is_empty(), "{label}");
    }

    // A second eligible managed account for the same publisher and platform.
    let source_id = Uuid::new_v4();
    let account_id = Uuid::new_v4();
    exec(
        &fx.pool,
        &format!(
            "INSERT INTO metric_source (source_id, code, acquisition_type, driver_key, enabled) \
             VALUES ('{source_id}', 'second-driver', 'DRIVER', 'second_driver', TRUE); \
             INSERT INTO metric_source_account \
                 (source_account_id, code, source_id, platform_id, external_key, \
                  expected_publisher_id, enabled) \
             VALUES ('{account_id}', 'second-account', '{source_id}', '{platform}', 'k2', \
                     '{publisher}', TRUE);",
            platform = fx.platform_id,
            publisher = fx.publisher_id,
        ),
    );
    let response = run(
        &schema,
        &context,
        DASHBOARD,
        dashboard_variables(&fx, json!([fx.publisher_id])),
    )
    .await;
    assert_classified(&response, "MOM1_SOURCE_SCOPE_AMBIGUOUS");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn mixed_dimensional_breakdowns_are_classified_through_the_api() {
    let (_guard, fx) = setup();
    let institution_id = insert_institution(&fx.pool);
    commit_dims(
        &fx,
        fx.works[0],
        fx.platform_id,
        fx.sessions,
        day_n(2),
        Dims {
            country: Some("GB"),
            ..Dims::default()
        },
        3,
    );
    commit_dims(
        &fx,
        fx.works[0],
        fx.platform_id,
        fx.sessions,
        day_n(2),
        Dims {
            institution: Some(institution_id),
            ..Dims::default()
        },
        3,
    );
    apply_all(&fx.pool);
    let schema = create_schema();
    let context = context_for(&fx.pool, Some(reader()));
    let response = run(
        &schema,
        &context,
        DASHBOARD,
        dashboard_variables(&fx, json!([fx.publisher_id])),
    )
    .await;
    let message = assert_classified(&response, "MOM1_DIMENSION_SCOPE_AMBIGUOUS");
    assert!(message.contains("dimensional"), "{message}");
}

// ==========================================================================
// The response through GraphQL
// ==========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn the_response_carries_bigint_strings_real_zeros_and_nulls() {
    let (_guard, fx) = setup();
    cover(
        &fx,
        fx.units,
        day_n(1),
        day_n(3),
        "COMPLETE",
        "COMPLETED",
        "'2026-03-10T00:00:00Z'",
    );
    commit(&fx, fx.works[0], fx.platform_id, fx.units, day_n(1), -9);
    apply_all(&fx.pool);
    exec(
        &fx.pool,
        &format!(
            "{}{}",
            projection_sql(
                fx.works[0],
                fx.platform_id,
                fx.units,
                day_n(3),
                None,
                i64::MAX
            ),
            projection_sql(
                fx.works[1],
                fx.platform_id,
                fx.units,
                day_n(3),
                None,
                i64::MAX
            ),
        ),
    );
    let schema = create_schema();
    let context = context_for(&fx.pool, Some(reader()));
    let mut variables = dashboard_variables(&fx, json!([fx.publisher_id]));
    variables["input"]["measures"] = json!([fx.units]);

    let response = run(&schema, &context, DASHBOARD, variables).await;
    let dashboard = data(&response, "metricDashboard");
    let values: Vec<&JsonValue> = dashboard["timeline"]
        .as_array()
        .expect("timeline")
        .iter()
        .map(|bucket| &bucket["value"])
        .collect();
    assert_eq!(
        values,
        vec![
            &json!("-9"),
            &json!("0"),
            &json!("18446744073709551614"),
            &JsonValue::Null
        ],
        "exact signed strings, a justified zero, and null where zero is not justified"
    );
    assert_eq!(
        dashboard["totals"][0]["value"],
        json!("18446744073709551605")
    );
    assert_eq!(dashboard["timeline"][0]["startDate"], json!("2026-03-01"));
    assert_eq!(dashboard["timeline"][0]["endDate"], json!("2026-03-02"));
    assert_eq!(dashboard["coverage"]["status"], json!("UNKNOWN"));
    assert_eq!(
        dashboard["coverage"]["items"][0]["dataThrough"],
        json!("2026-03-02")
    );
    assert_eq!(dashboard["dataThrough"], json!("2026-03-02"));
    assert_eq!(dashboard["warnings"][0]["code"], json!("UNKNOWN_COVERAGE"));
    assert_eq!(dashboard["isPartial"], json!(true));
    assert!(dashboard["asOf"].is_string());
    assert!(dashboard["rollupWatermark"].is_string());
}

// ==========================================================================
// Registry lists
// ==========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn registry_lists_return_every_identity_in_exact_code_order() {
    let (_guard, pool) = setup_registry_db();
    let schema = create_schema();
    let context = context_for(&pool, Some(reader()));
    for code in ["b_platform", "B_platform", "a_platform"] {
        insert_platform_row(&pool, Uuid::new_v4(), code);
    }
    exec(
        &pool,
        "UPDATE metric_platform SET enabled = FALSE WHERE code = 'b_platform';",
    );
    let non_additive = insert_measure(&pool, "Z_non_additive", false, false);
    exec(
        &pool,
        "UPDATE metric_measure SET enabled = FALSE WHERE code = 'net_units';",
    );

    let platforms = run(&schema, &context, PLATFORMS, json!({})).await;
    let platforms = data(&platforms, "metricPlatforms")
        .as_array()
        .expect("a list")
        .clone();
    let codes: Vec<&str> = platforms
        .iter()
        .map(|row| row["code"].as_str().expect("code"))
        .collect();
    assert_eq!(
        codes,
        vec!["B_platform", "a_platform", "b_platform"],
        "byte-wise code order"
    );
    assert_eq!(
        platforms[2]["enabled"],
        json!(false),
        "disabled identities stay discoverable"
    );

    let measures = run(&schema, &context, MEASURES, json!({})).await;
    let measures = data(&measures, "metricMeasures")
        .as_array()
        .expect("a list")
        .clone();
    let codes: Vec<&str> = measures
        .iter()
        .map(|row| row["code"].as_str().expect("code"))
        .collect();
    assert_eq!(codes, vec!["Z_non_additive", "net_units", "title_sessions"]);
    assert_eq!(measures[0]["measureId"], json!(non_additive));
    assert_eq!(
        measures[0]["additiveAcrossTime"],
        json!(false),
        "discovery is broader than serving"
    );
    assert_eq!(measures[1]["enabled"], json!(false));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn registry_lists_are_bounded_to_500_and_refuse_rather_than_truncate() {
    let (_guard, pool) = setup_registry_db();
    let schema = create_schema();
    let context = context_for(&pool, Some(reader()));

    // 500 platforms, and 498 measures on top of the two seeds.
    exec(
        &pool,
        "INSERT INTO metric_platform (code, display_name, ownership_class, enabled) \
         SELECT format('bulk_%s', lpad(n::text, 3, '0')), 'Bulk', 'EXTERNAL', TRUE \
         FROM generate_series(1, 500) n; \
         INSERT INTO metric_measure (code, display_name, category, unit, allow_negative, \
             additive_across_time, additive_across_works, definition, enabled) \
         SELECT format('bulk_%s', lpad(n::text, 3, '0')), 'Bulk', 'USAGE', 'COUNT', FALSE, \
             TRUE, TRUE, 'Bulk.', TRUE \
         FROM generate_series(1, 498) n;",
    );
    let platforms = run(&schema, &context, PLATFORMS, json!({})).await;
    assert_eq!(
        data(&platforms, "metricPlatforms").as_array().map(Vec::len),
        Some(500)
    );
    let measures = run(&schema, &context, MEASURES, json!({})).await;
    assert_eq!(
        data(&measures, "metricMeasures").as_array().map(Vec::len),
        Some(500)
    );

    exec(
        &pool,
        "INSERT INTO metric_platform (code, display_name, ownership_class, enabled) \
         VALUES ('bulk_501', 'Bulk', 'EXTERNAL', FALSE); \
         INSERT INTO metric_measure (code, display_name, category, unit, allow_negative, \
             additive_across_time, additive_across_works, definition, enabled) \
         VALUES ('bulk_499', 'Bulk', 'USAGE', 'COUNT', FALSE, TRUE, TRUE, 'Bulk.', FALSE);",
    );
    for (field, document) in [("metricPlatforms", PLATFORMS), ("metricMeasures", MEASURES)] {
        let response = run(&schema, &context, document, json!({})).await;
        let message = assert_classified(&response, "METRIC_REGISTRY_LIMIT_EXCEEDED");
        assert!(message.contains("500"), "{field}: {message}");
    }
}

// ==========================================================================
// Generated schema
// ==========================================================================

/// One SDL block with descriptions removed and whitespace collapsed.
fn signatures(block: &str) -> String {
    let mut out = String::new();
    let mut in_string = false;
    let mut escaped = false;
    for character in block.chars() {
        match character {
            _ if escaped => escaped = false,
            '\\' if in_string => escaped = true,
            '"' => in_string = !in_string,
            _ if in_string => {}
            _ if character.is_whitespace() => {}
            _ => out.push(character),
        }
    }
    out
}

fn enum_values(sdl: &str, declaration: &str) -> Vec<String> {
    sdl_block(sdl, declaration)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| {
            line.rsplit(' ')
                .next()
                .expect("enum value token")
                .to_string()
        })
        .collect()
}

#[test]
fn the_sdl_declares_exactly_the_approved_read_contract() {
    let sdl = create_schema().as_sdl();
    let query = signatures(sdl_block(&sdl, "type QueryRoot {"));
    for operation in [
        "metricDashboard(input:MetricDashboardInput!):MetricDashboard!",
        "metricMeasures:[MetricMeasure!]!",
        "metricPlatforms:[MetricPlatform!]!",
    ] {
        assert!(
            query.contains(operation),
            "`{operation}` is missing from QueryRoot"
        );
    }
    let read_fields = sdl_block(&sdl, "type QueryRoot {")
        .lines()
        .map(str::trim_start)
        .filter(|line| {
            [
                "metricDashboard",
                "metricMeasures",
                "metricPlatforms",
                "metricWidget",
                "metricWork",
            ]
            .iter()
            .any(|name| line.starts_with(name))
        })
        .count();
    assert_eq!(read_fields, 3, "exactly the three MOM-1 read operations");

    for (declaration, expected) in [
        (
            "input MetricDashboardInput {",
            "selector:MetricSelectorInput!startDate:Date!endDate:Date!measures:[Uuid!]platforms:[Uuid!]timelineGrain:MetricTimelineGrain=",
        ),
        ("input MetricSelectorInput {", "publisherIds:[Uuid!]"),
        (
            "type MetricDashboard {",
            "totals:[MetricTotal!]!timeline:[MetricTimeBucket!]!coverage:MetricCoverage!asOf:Timestamp!dataThrough:DaterollupWatermark:Timestamp!warnings:[MetricWarning!]!isPartial:Boolean!",
        ),
        ("type MetricTotal {", "platformId:Uuid!measureId:Uuid!value:BigInt"),
        (
            "type MetricTimeBucket {",
            "platformId:Uuid!measureId:Uuid!startDate:Date!endDate:Date!value:BigInt",
        ),
        ("type MetricCoverage {", "status:MetricCoverageStatus!items:[MetricCoverageItem!]!"),
        (
            "type MetricCoverageItem {",
            "platformId:Uuid!measureId:Uuid!status:MetricCoverageStatus!dataThrough:DatecountryCoverage:Boolean!institutionCoverage:Boolean!",
        ),
        ("type MetricWarning {", "code:MetricWarningCode!message:String!"),
    ] {
        assert_eq!(signatures(sdl_block(&sdl, declaration)), expected, "{declaration}");
    }
    // Juniper renders an enum input default quoted, exactly as the existing
    // `markupFormat: MarkupFormat = "JATS_XML"` arguments are rendered.
    assert!(sdl_block(&sdl, "input MetricDashboardInput {")
        .contains("timelineGrain: MetricTimelineGrain = \"AUTO\""));
    assert_eq!(
        enum_values(&sdl, "enum MetricCoverageStatus {"),
        ["COMPLETE", "PARTIAL", "UNKNOWN"]
    );
    assert_eq!(
        enum_values(&sdl, "enum MetricWarningCode {"),
        ["PARTIAL_COVERAGE", "UNKNOWN_COVERAGE", "ROLLUP_LAG"]
    );
    assert_eq!(
        enum_values(&sdl, "enum MetricTimelineGrain {"),
        ["AUTO", "DAY", "MONTH"]
    );

    // Repository scalar spellings, and exactly one new scalar.
    assert_eq!(sdl.matches("scalar BigInt").count(), 1);
    for absent in [
        "scalar UUID",
        "scalar DateTime",
        "scalar Long",
        "scalar BigInteger",
    ] {
        assert!(!sdl.contains(absent), "`{absent}` must not be declared");
    }
    // The plural selector is the public contract; there is no singular one.
    assert!(!signatures(sdl_block(&sdl, "input MetricSelectorInput {")).contains("publisherId:"));
    // Numeric Metrics values are never 32-bit Ints or floats.
    for block in ["type MetricTotal {", "type MetricTimeBucket {"] {
        let body = sdl_block(&sdl, block);
        assert!(
            !body.contains(": Int") && !body.contains(": Float"),
            "{block}"
        );
    }
}

#[test]
fn deferred_dashboard_surface_is_absent() {
    let sdl = create_schema().as_sdl();
    let dashboard = sdl_block(&sdl, "type MetricDashboard {");
    for deferred in ["countries", "institutions", "works", "workPage", "pageInfo"] {
        assert!(
            !dashboard.contains(deferred),
            "`{deferred}` is not part of MOM-1"
        );
    }
    let selector = sdl_block(&sdl, "input MetricSelectorInput {");
    for deferred in [
        "imprintIds",
        "seriesIds",
        "workIds",
        "dois",
        "workTypes",
        "languages",
        "fundingInstitutionIds",
        "affiliationInstitutionIds",
        "includeDescendants",
    ] {
        assert!(
            !selector.contains(deferred),
            "`{deferred}` is deferred from MOM-1"
        );
    }
    let input = sdl_block(&sdl, "input MetricDashboardInput {");
    for deferred in [
        "includeCountries",
        "includeInstitutions",
        "includeWorks",
        "workPage",
    ] {
        assert!(
            !input.contains(deferred),
            "`{deferred}` is deferred from MOM-1"
        );
    }
    for absent in ["metricWidget", "MetricWidget", "type MetricRollupWorkDay "] {
        assert!(!sdl.contains(absent), "`{absent}` must not be exposed");
    }
    // No public type navigates into Metrics data.
    for public_type in [
        "type Work {",
        "type Publisher {",
        "type Imprint {",
        "type Publication {",
        "type Series {",
    ] {
        assert!(
            !sdl_block(&sdl, public_type).contains("Metric"),
            "`{public_type}` must not expose a Metrics field"
        );
    }
}

#[test]
fn every_read_resolver_authorizes_through_the_read_service_guard_first() {
    let query_source = include_str!("query.rs");
    let guarded: String = query_source
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    for read in [
        "metric_dashboard(&context.db, &input)",
        "list_metric_measures(&context.db)",
        "list_metric_platforms(&context.db)",
    ] {
        let expected = format!(
            "context .require_metrics_read_service() .map_err(MetricReadError::from) .and_then(|_| {read})"
        );
        assert_eq!(
            guarded.matches(&expected).count(),
            1,
            "`{read}` must be guarded"
        );
        assert_eq!(
            guarded.matches(read).count(),
            1,
            "`{read}` must have one call site"
        );
    }
    assert_eq!(
        query_source
            .matches("require_metrics_read_service()")
            .count(),
        3
    );
}

// ==========================================================================
// Compatibility
// ==========================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn unrelated_public_queries_are_unchanged() {
    let (_guard, fx) = setup();
    let schema = create_schema();
    // Anonymous callers still read public bibliographic data exactly as before.
    let anonymous = context_for(&fx.pool, None);
    let response = run(
        &schema,
        &anonymous,
        &format!(
            "{{ publisher(publisherId: \"{}\") {{ publisherId publisherName }} }}",
            fx.publisher_id
        ),
        json!({}),
    )
    .await;
    assert_eq!(
        data(&response, "publisher")["publisherId"],
        json!(fx.publisher_id)
    );
    let response = run(&schema, &anonymous, "{ publisherCount }", json!({})).await;
    assert_eq!(data(&response, "publisherCount"), &json!(2));
    // A superuser still administers the registry, and still cannot read the
    // Metrics service surface.
    let superuser =
        test_db::test_context_with_user(Arc::clone(&fx.pool), test_db::test_superuser("root"));
    let response = run(
        &schema,
        &superuser,
        "{ metricMeasureByCode(code: \"net_units\") { code } }",
        json!({}),
    )
    .await;
    assert_eq!(
        data(&response, "metricMeasureByCode")["code"],
        json!("net_units")
    );
    assert_unauthorized(&run(&schema, &superuser, MEASURES, json!({})).await);
}
