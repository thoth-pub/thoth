//! GraphQL acceptance evidence for MET-WP7-PREREQ-03.

#![cfg(all(test, feature = "backend"))]

use std::collections::HashMap;
use std::sync::Arc;

use diesel::RunQueryDsl;
use serde_json::{json, Value as JsonValue};
use uuid::Uuid;
use zitadel::actix::introspection::IntrospectedUser;

use super::sdl_support::sdl_block;
use super::{create_schema, Context, GraphQLRequest, Schema};
use crate::db::PgPool;
use crate::model::metric_ingestion_lifecycle::tests::setup;
use crate::model::tests::db as test_db;
use crate::policy::Role;

const INGEST_USER: &str = "metrics-reconciler";

fn request(query: &str) -> GraphQLRequest {
    serde_json::from_value(json!({ "query": query })).expect("GraphQL request")
}

async fn run(schema: &Schema, context: &Context, query: &str) -> JsonValue {
    serde_json::to_value(request(query).execute(schema, context).await).expect("GraphQL response")
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
    assert_eq!(errors.len(), 1, "{response}");
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
    assert_eq!(kind, "NO_ACCESS", "{response}");
    assert_eq!(message, "Unauthorized");
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

fn mutation(limit: i32) -> String {
    format!(
        "mutation {{ reconcileMetricIdentifierQuarantine(limit: {limit})          {{ attempted resolved pending blocked }} }}"
    )
}

fn matrix(org: &str) -> Vec<(&'static str, Option<IntrospectedUser>, bool)> {
    vec![
        ("anonymous", None, false),
        (
            "authenticated without roles",
            Some(user_with("none", &[])),
            false,
        ),
        (
            "PUBLISHER_USER",
            Some(user_with("publisher-user", &[(Role::PublisherUser, org)])),
            false,
        ),
        (
            "PUBLISHER_ADMIN",
            Some(user_with("publisher-admin", &[(Role::PublisherAdmin, org)])),
            false,
        ),
        (
            "WORK_LIFECYCLE",
            Some(user_with("work-lifecycle", &[(Role::WorkLifecycle, org)])),
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
            "METRICS_READ_SERVICE",
            Some(user_with("reader", &[(Role::MetricsReadService, org)])),
            false,
        ),
        (
            "METRICS_INGEST_SERVICE",
            Some(user_with(INGEST_USER, &[(Role::MetricsIngestService, org)])),
            true,
        ),
        (
            "METRICS_INGEST_SERVICE plus unrelated role",
            Some(user_with(
                INGEST_USER,
                &[
                    (Role::MetricsIngestService, org),
                    (Role::PublisherUser, "other-org"),
                ],
            )),
            true,
        ),
    ]
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn the_exact_authorization_matrix_is_fail_closed() {
    let (_guard, fixture) = setup();
    let schema = create_schema();

    for (label, user, allowed) in matrix("org-1") {
        let context = context_for(&fixture.pool, user);
        let response = run(&schema, &context, &mutation(10)).await;
        if allowed {
            assert_eq!(
                data(&response, "reconcileMetricIdentifierQuarantine"),
                &json!({
                    "attempted": 0,
                    "resolved": 0,
                    "pending": 0,
                    "blocked": 0
                }),
                "{label}: {response}"
            );
        } else {
            assert_unauthorized(&response);
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn limit_bounds_are_rejected_before_any_reconciliation_state_is_written() {
    let (_guard, fixture) = setup();
    let schema = create_schema();
    let context = context_for(
        &fixture.pool,
        Some(user_with(
            INGEST_USER,
            &[(Role::MetricsIngestService, "org-1")],
        )),
    );

    for limit in [0, -1, 51, i32::MAX] {
        let response = run(&schema, &context, &mutation(limit)).await;
        let (message, kind) = only_error(&response);
        assert_eq!(
            message,
            "A metric identifier quarantine reconciliation limit must be between 1 and 50 inclusive."
        );
        assert_eq!(kind, "INTERNAL_ERROR");
    }

    let mut connection = fixture.pool.get().expect("database connection");
    let count: i64 = diesel::select(diesel::dsl::sql::<diesel::sql_types::BigInt>(
        "(SELECT COUNT(*) FROM metric_identifier_quarantine_reconciliation)",
    ))
    .get_result(&mut connection)
    .expect("state count");
    assert_eq!(count, 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn unexpected_database_failure_is_fixed_and_redacted_at_graphql() {
    let (_guard, fixture) = setup();
    let schema = create_schema();
    let context = context_for(
        &fixture.pool,
        Some(user_with(
            INGEST_USER,
            &[(Role::MetricsIngestService, "org-1")],
        )),
    );

    // Seed one otherwise-valid unresolved-DOI quarantine row so the mutation
    // reaches the reconciliation-state write. A reversible CHECK constraint
    // then forces that write to fail for this exact actor. Unlike dropping the
    // table, this leaves the migrated schema intact for TestDbGuard rollback
    // and for unrelated tests sharing the CI database.
    let import_id = Uuid::new_v4();
    let batch_id = Uuid::new_v4();
    let provenance_id = Uuid::new_v4();
    let quarantine_id = Uuid::new_v4();
    let mut connection = fixture.pool.get().expect("database connection");

    diesel::sql_query(format!(
        "INSERT INTO metric_import              (import_id, source_account_id, publisher_id, format_code, format_version, status,               received_count, invalid_count, normalizer_version, created_by, created_at, completed_at)          VALUES ('{import_id}', '{}', '{}', 'cloudfront-legacy-s3', '1',                  'COMPLETED_WITH_ERRORS', 1, 1, 'sphinx-cloudfront/1', 'fixture',                  TIMESTAMPTZ '2026-09-22 08:00:00+00', TIMESTAMPTZ '2026-09-22 08:00:00+00')",
        fixture.account_a, fixture.publisher_id
    ))
    .execute(&mut connection)
    .expect("seed import");
    diesel::sql_query(format!(
        "INSERT INTO metric_import_batch (import_batch_id, import_id, batch_key, request_hash)          VALUES ('{batch_id}', '{import_id}', '{quarantine_id}', 'hash-{quarantine_id}')"
    ))
    .execute(&mut connection)
    .expect("seed batch");
    diesel::sql_query(format!(
        "INSERT INTO metric_record_provenance              (record_provenance_id, import_id, classification, details, import_batch_id, batch_row_index)          VALUES ('{provenance_id}', '{import_id}', 'REJECTED',                  '{{\"schema\":\"thoth-metric-provenance-details/1\",                    \"reason_code\":\"UNKNOWN_DOI\",\"reporting_grain\":\"DAY\"}}'::jsonb,                  '{batch_id}', 0)"
    ))
    .execute(&mut connection)
    .expect("seed provenance");
    diesel::sql_query(format!(
        "INSERT INTO metric_identifier_quarantine              (identifier_quarantine_id, record_provenance_id, source_account_id, platform_id,               measure_id, schema_version, work_doi, period_start, period_end, reporting_grain,               country_code, value, methodology_version, created_at)          VALUES ('{quarantine_id}', '{provenance_id}', '{}', '{}',                  (SELECT measure_id FROM metric_measure WHERE code = 'title_sessions'),                  'thoth-normalized-metrics/1', 'https://doi.org/10.12345/graphql-redaction',                  DATE '2026-03-01', DATE '2026-03-02', 'DAY', NULL, 1,                  'cloudfront-title-session/2', TIMESTAMPTZ '2026-09-22 08:00:00+00')",
        fixture.account_a, fixture.platform_id
    ))
    .execute(&mut connection)
    .expect("seed quarantine");
    diesel::sql_query(
        "ALTER TABLE metric_identifier_quarantine_reconciliation          ADD CONSTRAINT test_reconciliation_actor CHECK (last_attempted_by <> 'metrics-reconciler')",
    )
    .execute(&mut connection)
    .expect("install reversible fault");

    drop(connection);
    let response = run(&schema, &context, &mutation(1)).await;

    // Restore the disposable schema before making assertions so even an
    // assertion failure cannot poison later migration rollbacks.
    let mut connection = fixture.pool.get().expect("database connection");
    diesel::sql_query(
        "ALTER TABLE metric_identifier_quarantine_reconciliation          DROP CONSTRAINT test_reconciliation_actor",
    )
    .execute(&mut connection)
    .expect("remove reversible fault");

    let (message, kind) = only_error(&response);
    assert_eq!(
        message,
        "Internal error: Metric identifier quarantine reconciliation failed safely."
    );
    assert_eq!(kind, "INTERNAL_ERROR");
    for leaked in [
        "metric_identifier",
        "SELECT",
        "postgres",
        "constraint",
        "test_reconciliation_actor",
        "sql",
    ] {
        assert!(
            !message
                .to_ascii_lowercase()
                .contains(&leaked.to_ascii_lowercase()),
            "leaked {leaked}: {message}"
        );
    }
}

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

#[test]
fn sdl_exposes_only_the_bounded_mutation_and_aggregate_result() {
    let sdl = create_schema().as_sdl();
    let mutation_root = sdl_block(&sdl, "type MutationRoot {");
    let signature = "reconcileMetricIdentifierQuarantine(limit:Int!):MetricIdentifierQuarantineReconciliationBatch!";
    assert_eq!(
        signatures(mutation_root).matches(signature).count(),
        1,
        "MutationRoot: {mutation_root}"
    );

    let result = sdl_block(&sdl, "type MetricIdentifierQuarantineReconciliationBatch {");
    assert_eq!(
        signatures(result),
        "attempted:Int!resolved:Int!pending:Int!blocked:Int!"
    );

    let query_root = sdl_block(&sdl, "type QueryRoot {");
    assert!(!query_root.contains("IdentifierQuarantineReconciliation"));
    for forbidden in [
        "identifierQuarantineId",
        "recordId",
        "recordRevisionId",
        "nextAttemptAt",
        "lastAttemptAt",
        "attemptCount",
        "state:",
    ] {
        assert!(
            !result.contains(forbidden),
            "aggregate result leaked {forbidden}: {result}"
        );
    }
}
