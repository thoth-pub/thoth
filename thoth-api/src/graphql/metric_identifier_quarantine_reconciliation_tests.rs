//! GraphQL boundary acceptance evidence for `MET-WP7-PREREQ-03`.
//!
//! The tests use the production schema/resolver and a disposable migrated
//! database. Authorization is proven before operation-specific database access.

#![cfg(all(test, feature = "backend"))]

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::{json, Value as JsonValue};
use zitadel::actix::introspection::IntrospectedUser;

use super::sdl_support::sdl_block;
use super::{create_schema, Context, GraphQLRequest, Schema};
use crate::db::PgPool;
use crate::model::metric_ingestion_lifecycle::tests::setup;
use crate::model::tests::db as test_db;
use crate::policy::Role;

fn request(query: &str) -> GraphQLRequest {
    serde_json::from_value(json!({ "query": query })).expect("request")
}

async fn run(schema: &Schema, context: &Context, query: &str) -> JsonValue {
    serde_json::to_value(request(query).execute(schema, context).await).expect("response")
}

fn only_error(response: &JsonValue) -> (String, String) {
    let errors = response["errors"].as_array().expect("errors");
    assert_eq!(errors.len(), 1, "{response}");
    (
        errors[0]["message"].as_str().expect("message").to_string(),
        errors[0]["extensions"]["type"]
            .as_str()
            .expect("extensions.type")
            .to_string(),
    )
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

fn signature(block: &str) -> String {
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
fn sdl_is_exactly_one_bounded_aggregate_only_mutation() {
    let sdl = create_schema().as_sdl();
    let mutation_root = sdl_block(&sdl, "type MutationRoot {");
    assert_eq!(
        mutation_root.matches("reconcileMetricIdentifierQuarantine(").count(),
        1
    );
    assert!(
        signature(mutation_root).contains(
            "reconcileMetricIdentifierQuarantine(limit:Int!):MetricIdentifierQuarantineReconciliationBatch!"
        ),
        "{mutation_root}"
    );

    let batch = signature(sdl_block(
        &sdl,
        "type MetricIdentifierQuarantineReconciliationBatch {",
    ));
    assert!(batch.contains("attempted:Int!"));
    assert!(batch.contains("resolved:Int!"));
    assert!(batch.contains("pending:Int!"));
    assert!(batch.contains("blocked:Int!"));
    for forbidden in [
        "identifierQuarantineId",
        "recordId",
        "recordRevisionId",
        "state",
        "nextAttemptAt",
        "lastAttemptAt",
        "attemptCount",
        "lastAttemptedBy",
    ] {
        assert!(!batch.contains(forbidden), "{forbidden} leaked into {batch}");
    }
}

#[tokio::test]
async fn only_metrics_ingest_service_can_reach_reconciliation() {
    let (_guard, fixture) = setup();
    let schema = create_schema();
    let denied = vec![
        ("anonymous", None),
        ("authenticated-no-role", Some(user_with("none", &[]))),
        (
            "superuser",
            Some(user_with("super", &[(Role::Superuser, "org")])),
        ),
        (
            "publisher-admin",
            Some(user_with(
                "publisher-admin",
                &[(Role::PublisherAdmin, "org")],
            )),
        ),
        (
            "publisher-user",
            Some(user_with("publisher-user", &[(Role::PublisherUser, "org")])),
        ),
        (
            "work-lifecycle",
            Some(user_with(
                "work-lifecycle",
                &[(Role::WorkLifecycle, "org")],
            )),
        ),
        (
            "cdn-write",
            Some(user_with("cdn-write", &[(Role::CdnWrite, "org")])),
        ),
        (
            "dissemination-worker",
            Some(user_with(
                "worker",
                &[(Role::DisseminationWorker, "org")],
            )),
        ),
        (
            "metrics-read",
            Some(user_with(
                "metrics-read",
                &[(Role::MetricsReadService, "org")],
            )),
        ),
    ];

    for (label, user) in denied {
        let response = run(&schema, &context_for(&fixture.pool, user), &mutation(1)).await;
        let (message, kind) = only_error(&response);
        assert_eq!((message.as_str(), kind.as_str()), ("Unauthorized", "NO_ACCESS"), "{label}");
    }

    for user in [
        user_with(
            "ingest",
            &[(Role::MetricsIngestService, "org")],
        ),
        user_with(
            "ingest-plus-unrelated",
            &[
                (Role::MetricsIngestService, "org"),
                (Role::Superuser, "org"),
                (Role::MetricsReadService, "org"),
            ],
        ),
    ] {
        let response = run(&schema, &context_for(&fixture.pool, Some(user)), &mutation(1)).await;
        assert!(
            response.get("errors").is_none()
                || response["errors"].as_array().is_some_and(Vec::is_empty),
            "{response}"
        );
        assert_eq!(
            response["data"]["reconcileMetricIdentifierQuarantine"],
            json!({"attempted": 0, "resolved": 0, "pending": 0, "blocked": 0})
        );
    }
}

#[tokio::test]
async fn limit_is_rejected_outside_one_through_fifty_without_truncation() {
    for limit in [0, -1, 51, i32::MAX] {
        let (_guard, fixture) = setup();
        let schema = create_schema();
        let context = context_for(
            &fixture.pool,
            Some(user_with(
                "ingest",
                &[(Role::MetricsIngestService, "org")],
            )),
        );
        let response = run(&schema, &context, &mutation(limit)).await;
        let (message, _kind) = only_error(&response);
        assert_eq!(
            message,
            "A metric identifier quarantine reconciliation limit must be between 1 and 50 inclusive."
        );
        assert!(
            response["data"]["reconcileMetricIdentifierQuarantine"].is_null(),
            "{response}"
        );
    }
}

#[tokio::test]
async fn authorization_precedes_database_access_and_internal_failures_are_redacted() {
    let (_guard, fixture) = setup();
    fixture.sql("DROP TABLE metric_identifier_quarantine_reconciliation");
    let schema = create_schema();

    let denied = run(
        &schema,
        &context_for(
            &fixture.pool,
            Some(user_with("reader", &[(Role::MetricsReadService, "org")])),
        ),
        &mutation(1),
    )
    .await;
    assert_eq!(
        only_error(&denied),
        ("Unauthorized".to_string(), "NO_ACCESS".to_string())
    );

    let authorized = run(
        &schema,
        &context_for(
            &fixture.pool,
            Some(user_with(
                "ingest",
                &[(Role::MetricsIngestService, "org")],
            )),
        ),
        &mutation(1),
    )
    .await;
    let (message, kind) = only_error(&authorized);
    assert_eq!(kind, "INTERNAL_ERROR");
    assert_eq!(
        message,
        "Metric identifier quarantine reconciliation failed safely."
    );
    assert!(message.len() < 100);
    for leaked in [
        "SELECT",
        "INSERT",
        "UPDATE",
        "postgres",
        "relation",
        "metric_identifier_quarantine_reconciliation",
        "password",
        "token",
    ] {
        assert!(!message.contains(leaked), "{leaked} leaked in {message}");
    }
}
