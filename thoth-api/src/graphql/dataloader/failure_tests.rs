use std::sync::Arc;

use serde_json::{json, Value as JsonValue};

use super::fixture::{empty_source, fixture_context, schema, FixtureLoaders, TestSchema};
use super::FieldErrorConvention;
use crate::db::PgPool;
use crate::graphql::GraphQLRequest;
use crate::model::tests::db as test_db;

fn request(query: &str) -> GraphQLRequest {
    serde_json::from_value(json!({ "query": query })).expect("build GraphQL request")
}

async fn run_response(
    schema: &TestSchema,
    context: &crate::graphql::Context,
    query: &str,
) -> JsonValue {
    serde_json::to_value(request(query).execute(schema, context).await)
        .expect("serialize GraphQL response")
}

async fn failure_response(
    pool: Arc<PgPool>,
    direct: bool,
    convention: FieldErrorConvention,
    field: &str,
) -> (JsonValue, usize) {
    let failing = Arc::new(test_db::failing_pool());
    let mut loaders = FixtureLoaders::in_memory(empty_source(), "");
    if direct {
        loaders = loaders.with_direct_db(failing);
    } else {
        loaders = loaders.with_db(failing, convention);
    }
    let stats = Arc::clone(&loaders.db_stats);
    let context = fixture_context(pool, loaders);
    let response = run_response(
        &schema(),
        &context,
        &format!("{{ dbParents {{ publisherId {field} }} }}"),
    )
    .await;
    (response, stats.dispatch_count())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn conventional_backend_failure_matches_direct_graphql_semantics_without_retry_or_fallback() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    test_db::create_imprint(&pool, &publisher);

    let (direct, direct_dispatches) = failure_response(
        Arc::clone(&pool),
        true,
        FieldErrorConvention::Conventional,
        "imprints",
    )
    .await;
    let (loaded, dispatches) =
        failure_response(pool, false, FieldErrorConvention::Conventional, "imprints").await;

    assert_eq!(direct_dispatches, 0);
    assert_eq!(
        dispatches, 1,
        "batch failure must not retry or fall back per key"
    );
    assert_eq!(loaded, direct);
    // `imprints` is a non-null list, so the field error null-propagates the
    // whole `data` object on both paths identically.
    assert!(loaded["data"].is_null());
    assert!(loaded["errors"].as_array().is_some_and(|e| !e.is_empty()));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn explicit_thoth_backend_failure_preserves_extensions_without_serde_clone() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    test_db::create_imprint(&pool, &publisher);

    let (direct, _) = failure_response(
        Arc::clone(&pool),
        true,
        FieldErrorConvention::ExplicitThoth,
        "imprintsExplicit",
    )
    .await;
    let (loaded, dispatches) = failure_response(
        pool,
        false,
        FieldErrorConvention::ExplicitThoth,
        "imprintsExplicit",
    )
    .await;

    assert_eq!(
        dispatches, 1,
        "batch failure must not retry or fall back per key"
    );
    assert_eq!(loaded, direct);
    assert_eq!(loaded["errors"][0]["extensions"]["type"], "INTERNAL_ERROR");
}
