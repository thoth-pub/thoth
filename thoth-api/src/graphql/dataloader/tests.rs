use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use futures::future::join_all;
use serde_json::{json, Value as JsonValue};
use uuid::Uuid;

use super::fixture::{
    empty_source, fixture_context, schema, source_from, BatchStats, FixtureLoaders, MemSource,
    SqlProbe, TestSchema,
};
use super::{configured_loader, FieldErrorConvention, LOADER_CONFIG, MAX_BATCH_SIZE, YIELD_COUNT};
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

fn response_data<'a>(response: &'a JsonValue, field: &str) -> &'a JsonValue {
    assert!(
        response.get("errors").is_none()
            || response["errors"].as_array().is_some_and(Vec::is_empty),
        "unexpected GraphQL errors: {response}"
    );
    &response["data"][field]
}

async fn scenario_run(
    pool: &Arc<PgPool>,
    parent_count: usize,
    field: &str,
) -> (usize, Vec<usize>, usize, usize) {
    let source: MemSource = Arc::new(Mutex::new(
        (1..=parent_count as i32)
            .map(|id| (id, vec![format!("c-{id}")]))
            .collect(),
    ));
    let loaders = FixtureLoaders::in_memory(source, "");
    let stats = Arc::clone(&loaders.mem_stats);
    let meta_stats = Arc::clone(&loaders.meta_stats);
    let calls = Arc::clone(&loaders.load_calls);
    let context = fixture_context(Arc::clone(pool), loaders);
    let schema = schema();
    let query = format!("{{ parents(count: {parent_count}) {{ parentId {field} }} }}");
    let response = run_response(&schema, &context, &query).await;
    let parents = response_data(&response, "parents")
        .as_array()
        .expect("parents array");
    assert_eq!(parents.len(), parent_count);
    for parent in parents {
        let id = parent["parentId"].as_i64().expect("parent id");
        assert_eq!(parent[field], json!([format!("c-{id}")]));
    }
    (
        stats.dispatch_count(),
        stats.batch_sizes(),
        calls.load(Ordering::SeqCst),
        meta_stats.dispatch_count(),
    )
}

async fn boundary_suite(pool: &Arc<PgPool>) {
    for (n, expected) in [
        (1usize, vec![1usize]),
        (100, vec![100]),
        (200, vec![200]),
        (201, vec![200, 1]),
        (500, vec![200, 200, 100]),
    ] {
        let (dispatches, sizes, calls, _) = scenario_run(pool, n, "children").await;
        assert_eq!(calls, n, "every sibling resolver must call try_load");
        assert_eq!(dispatches, expected.len(), "N={n}");
        assert_eq!(sizes, expected, "N={n}");
    }
}

#[test]
fn production_constructor_uses_explicit_200_10_configuration_constants() {
    assert_eq!(MAX_BATCH_SIZE, 200);
    assert_eq!(YIELD_COUNT, 10);
    assert_eq!(LOADER_CONFIG.max_batch_size, 200);
    assert_eq!(LOADER_CONFIG.yield_count, 10);

    let _loader = configured_loader(super::fixture::MemBatcher {
        source: empty_source(),
        stats: Arc::new(BatchStats::default()),
        marker: "",
        fail: false,
        omit_all: false,
    });
}

#[tokio::test]
async fn batch_boundaries_current_thread() {
    let (_guard, pool) = test_db::setup_test_db();
    boundary_suite(&pool).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn batch_boundaries_multi_thread() {
    let (_guard, pool) = test_db::setup_test_db();
    boundary_suite(&pool).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn genuine_async_child_resolver_executes_on_pinned_juniper() {
    let (_guard, pool) = test_db::setup_test_db();
    let context = fixture_context(pool, FixtureLoaders::in_memory(empty_source(), ""));
    let response = run_response(
        &schema(),
        &context,
        "{ parents(count: 3) { parentId asyncProbe } }",
    )
    .await;
    let parents = response_data(&response, "parents").as_array().expect("parents");
    assert_eq!(parents.len(), 3);
    assert_eq!(parents[0]["asyncProbe"], "async-1");
    assert_eq!(parents[2]["asyncProbe"], "async-3");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn scheduling_immediate_and_benign_yield_coalesce() {
    let (_guard, pool) = test_db::setup_test_db();
    let (direct_dispatches, direct_sizes, _, _) = scenario_run(&pool, 100, "children").await;
    assert_eq!((direct_dispatches, direct_sizes), (1, vec![100]));
    let (yield_dispatches, yield_sizes, _, _) =
        scenario_run(&pool, 100, "childrenAfterYield").await;
    assert_eq!((yield_dispatches, yield_sizes), (1, vec![100]));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn scheduling_delayed_cohort_can_fragment_dispatch() {
    let (_guard, pool) = test_db::setup_test_db();
    let (dispatches, sizes, calls, _) = scenario_run(&pool, 100, "childrenDelayed").await;
    assert_eq!(calls, 100);
    assert!(dispatches > 1, "delayed cohort must fragment: {sizes:?}");
    assert_eq!(sizes.iter().sum::<usize>(), 100);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn scheduling_loader_behind_loader_remains_set_based_for_target() {
    let (_guard, pool) = test_db::setup_test_db();
    let (dispatches, sizes, calls, meta_dispatches) =
        scenario_run(&pool, 100, "childrenChained").await;
    assert_eq!(calls, 100);
    assert_eq!((dispatches, sizes), (1, vec![100]));
    assert_eq!(meta_dispatches, 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn two_request_contexts_share_no_loader_state() {
    let (_guard, pool) = test_db::setup_test_db();
    let source = source_from(&[(7, &["child"])]);
    let a = FixtureLoaders::in_memory(Arc::clone(&source), "request-a");
    let b = FixtureLoaders::in_memory(source, "request-b");
    let a_stats = Arc::clone(&a.mem_stats);
    let b_stats = Arc::clone(&b.mem_stats);
    let a_ctx = fixture_context(Arc::clone(&pool), a);
    let b_ctx = fixture_context(pool, b);
    let a_loader = &a_ctx
        .batch_store
        .fixture
        .as_ref()
        .expect("a fixture")
        .mem;
    let b_loader = &b_ctx
        .batch_store
        .fixture
        .as_ref()
        .expect("b fixture")
        .mem;
    let (a_value, b_value) = tokio::join!(a_loader.try_load(7), b_loader.try_load(7));
    assert_eq!(
        a_value.expect("a key").expect("a value"),
        vec!["request-a:child"]
    );
    assert_eq!(
        b_value.expect("b key").expect("b value"),
        vec!["request-b:child"]
    );
    assert_eq!(a_stats.dispatch_count(), 1);
    assert_eq!(b_stats.dispatch_count(), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn completed_result_is_not_cached_and_read_write_read_is_fresh() {
    let source = source_from(&[(7, &["old"])]);
    let loaders = FixtureLoaders::in_memory(Arc::clone(&source), "");
    let stats = Arc::clone(&loaders.mem_stats);
    let first = loaders
        .mem
        .try_load(7)
        .await
        .expect("first key")
        .expect("first value");
    assert_eq!(first, vec!["old"]);
    source
        .lock()
        .expect("source lock")
        .insert(7, vec!["new".to_string()]);
    let second = loaders
        .mem
        .try_load(7)
        .await
        .expect("second key")
        .expect("second value");
    assert_eq!(second, vec!["new"]);
    assert_eq!(stats.dispatch_count(), 2, "completed result must refetch");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn pending_duplicate_keys_may_coalesce_without_becoming_cache() {
    let loaders = FixtureLoaders::in_memory(source_from(&[(7, &["v"])]), "");
    let stats = Arc::clone(&loaders.mem_stats);
    let (a, b) = tokio::join!(loaders.mem.try_load(7), loaders.mem.try_load(7));
    assert_eq!(a.expect("a key").expect("a value"), vec!["v"]);
    assert_eq!(b.expect("b key").expect("b value"), vec!["v"]);
    assert_eq!(stats.dispatch_count(), 1);
    assert_eq!(stats.batch_sizes(), vec![1]);
    let _ = loaders
        .mem
        .try_load(7)
        .await
        .expect("later key")
        .expect("later value");
    assert_eq!(stats.dispatch_count(), 2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn mutation_read_write_read_observes_new_value_without_invalidation() {
    let (_guard, pool) = test_db::setup_test_db();
    let loaders = FixtureLoaders::in_memory(source_from(&[(1, &["before"])]), "");
    let stats = Arc::clone(&loaders.mem_stats);
    let context = fixture_context(pool, loaders);
    let response = run_response(
        &schema(),
        &context,
        "mutation { rewrite(id: 1, newChild: \"after\") { id before after } }",
    )
    .await;
    let payload = response_data(&response, "rewrite");
    assert_eq!(payload["before"], json!(["before"]));
    assert_eq!(payload["after"], json!(["after"]));
    assert_eq!(stats.dispatch_count(), 2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn missing_batch_result_fails_closed_through_try_load_without_panic() {
    let (_guard, pool) = test_db::setup_test_db();
    let context = fixture_context(pool, FixtureLoaders::in_memory_omitting(empty_source()));
    let response = run_response(
        &schema(),
        &context,
        "{ parents(count: 1) { parentId children } }",
    )
    .await;
    assert!(response["data"]["parents"][0]["children"].is_null());
    assert!(response["errors"]
        .as_array()
        .is_some_and(|errors| !errors.is_empty()));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn batch_wide_in_memory_failure_is_error_for_every_key_not_empty_success() {
    let (_guard, pool) = test_db::setup_test_db();
    let loaders = FixtureLoaders::in_memory_failing(empty_source());
    let stats = Arc::clone(&loaders.mem_stats);
    let context = fixture_context(pool, loaders);
    let response = run_response(
        &schema(),
        &context,
        "{ parents(count: 3) { parentId children } }",
    )
    .await;
    let parents = response["data"]["parents"].as_array().expect("parents");
    assert!(parents.iter().all(|parent| parent["children"].is_null()));
    assert_eq!(response["errors"].as_array().expect("errors").len(), 3);
    assert_eq!(stats.dispatch_count(), 1);
}

fn seed_publishers(pool: &PgPool, count: usize) -> HashMap<Uuid, Vec<String>> {
    let mut expected = HashMap::new();
    for _ in 0..count {
        let publisher = test_db::create_publisher(pool);
        let imprint = test_db::create_imprint(pool, &publisher);
        expected.insert(publisher.publisher_id, vec![imprint.imprint_name]);
    }
    expected
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn real_diesel_250_parents_use_two_set_based_imprint_statements() {
    let (_guard, ordinary_pool) = test_db::setup_test_db();
    let expected = seed_publishers(&ordinary_pool, 250);
    let probe = SqlProbe::install(&test_db::test_db_url());
    let loaders = FixtureLoaders::in_memory(empty_source(), "")
        .with_db(Arc::clone(&probe.pool), FieldErrorConvention::Conventional);
    let stats = Arc::clone(&loaders.db_stats);
    let context = fixture_context(Arc::clone(&probe.pool), loaders);
    probe.start();
    let response = run_response(&schema(), &context, "{ dbParents { publisherId imprints } }").await;
    let statements = probe.imprint_statements();
    let parents = response_data(&response, "dbParents")
        .as_array()
        .expect("db parents");
    assert_eq!(parents.len(), 250);
    for parent in parents {
        let id: Uuid = parent["publisherId"]
            .as_str()
            .expect("publisher id")
            .parse()
            .expect("uuid");
        let values: Vec<String> = parent["imprints"]
            .as_array()
            .expect("imprints")
            .iter()
            .map(|value| value.as_str().expect("imprint").to_string())
            .collect();
        assert_eq!(&values, expected.get(&id).expect("expected publisher"));
    }
    assert_eq!(stats.dispatch_count(), 2);
    assert_eq!(stats.batch_sizes(), vec![200, 50]);
    assert_eq!(statements.len(), 2, "expected two real imprint statements");
    assert!(statements.iter().all(|statement| statement.contains("= ANY")));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn direct_try_load_batch_function_is_total_for_childless_parent() {
    let (_guard, pool) = test_db::setup_test_db();
    let publisher = test_db::create_publisher(&pool);
    let loaders = FixtureLoaders::in_memory(empty_source(), "")
        .with_db(Arc::clone(&pool), FieldErrorConvention::Conventional);
    let values = loaders
        .db
        .as_ref()
        .expect("db loader")
        .try_load(publisher.publisher_id)
        .await
        .expect("key present")
        .expect("successful empty relationship");
    assert!(values.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn independent_try_load_calls_coalesce_without_manual_key_aggregation() {
    let source: MemSource = Arc::new(Mutex::new(
        (1..=100)
            .map(|id| (id, vec![format!("v-{id}")]))
            .collect(),
    ));
    let loaders = FixtureLoaders::in_memory(source, "");
    let stats = Arc::clone(&loaders.mem_stats);
    let futures = (1..=100).map(|id| loaders.mem.try_load(id));
    let results = join_all(futures).await;
    assert!(results.iter().all(Result::is_ok));
    assert_eq!(stats.dispatch_count(), 1);
    assert_eq!(stats.batch_sizes(), vec![100]);
}

#[test]
fn shared_batch_error_projection_preserves_current_conventions_without_serde() {
    let conventional = super::SharedBatchError::from_thoth(
        thoth_errors::ThothError::Unauthorised,
        FieldErrorConvention::Conventional,
    )
    .to_field_error();
    let explicit = super::SharedBatchError::from_thoth(
        thoth_errors::ThothError::Unauthorised,
        FieldErrorConvention::ExplicitThoth,
    )
    .to_field_error();
    assert_eq!(conventional.message(), "Invalid credentials.");
    assert_eq!(conventional.extensions(), &juniper::Value::Null);
    assert_eq!(explicit.message(), "Unauthorized");
    assert_eq!(
        explicit.extensions(),
        &juniper::graphql_value!({ "type": "NO_ACCESS" })
    );
}

#[test]
fn source_contains_no_serde_round_trip_error_clone() {
    let source = include_str!("../dataloader.rs");
    let forbidden = [
        format!("{}{}", "ThothError::to_", "json"),
        format!("{}{}", "ThothError::from_", "json"),
        format!("{}{}", "clone_thoth_", "error"),
    ];
    for needle in forbidden {
        assert!(!source.contains(&needle));
    }
}
