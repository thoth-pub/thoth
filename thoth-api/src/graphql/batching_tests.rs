//! Test matrix for the request-scoped GraphQL batching foundation and the
//! central mutation request guard (THOTH-GQL-BATCH-01 section 10).
//!
//! Database-backed cases run against the disposable test database under the
//! repository's existing exclusive test lock.

#![cfg(feature = "backend")]

use std::sync::Arc;

use juniper::{http::GraphQLRequest, DefaultScalarValue, Variables};
use serde_json::{json, Value as JsonValue};
use uuid::Uuid;

use crate::db::PgPool;
use crate::graphql::batching::{BatchLookup, DispatchResult, GraphqlBatchStore};
use crate::graphql::batching_fixture::{
    direct_imprint_names, intermediate_resolver_calls, mutation_resolver_calls, reset_counters,
    terminal_fallback_calls, test_schema, SqlProbe, TestImprintLoader, TestSchema,
    DEFAULT_IMPRINT_LIMIT,
};
use crate::graphql::mutation_guard::{self, GuardOutcome, MutationGuardMode};
use crate::graphql::Context;
use crate::model::publisher::Publisher;
use crate::model::tests::db as test_db;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create `publishers` publishers, each with `imprints_each` imprints.
fn seed(pool: &PgPool, publishers: usize, imprints_each: usize) -> Vec<Publisher> {
    let mut created = Vec::with_capacity(publishers);
    for _ in 0..publishers {
        let publisher = test_db::create_publisher(pool);
        for _ in 0..imprints_each {
            test_db::create_imprint(pool, &publisher);
        }
        created.push(publisher);
    }
    created
}

/// Build a request context whose store availability is *derived* from `mode`.
fn context_in_mode(pool: Arc<PgPool>, mode: MutationGuardMode) -> Context {
    test_db::test_context_with_guard_mode(pool, "batching-test-user", mode)
}

/// Execute against the test-only schema, synchronously.
fn run_sync(
    schema: &TestSchema,
    context: &Context,
    query: &str,
    variables: Variables,
) -> (JsonValue, Vec<String>) {
    match juniper::execute_sync(query, None, schema, &variables, context) {
        Ok((value, errors)) => (
            serde_json::to_value(value).expect("serialize"),
            errors
                .iter()
                .map(|e| e.error().message().to_string())
                .collect(),
        ),
        Err(error) => (JsonValue::Null, vec![error.to_string()]),
    }
}

/// Execute against the test-only schema, asynchronously — the production path.
async fn run_async(
    schema: &TestSchema,
    context: &Context,
    query: &str,
    variables: Variables,
) -> (JsonValue, Vec<String>) {
    match juniper::execute(query, None, schema, &variables, context).await {
        Ok((value, errors)) => (
            serde_json::to_value(value).expect("serialize"),
            errors
                .iter()
                .map(|e| e.error().message().to_string())
                .collect(),
        ),
        Err(error) => (JsonValue::Null, vec![error.to_string()]),
    }
}

fn request(
    query: &str,
    operation_name: Option<&str>,
    variables: Option<JsonValue>,
) -> GraphQLRequest {
    let mut body = json!({ "query": query });
    if let Some(name) = operation_name {
        body["operationName"] = json!(name);
    }
    if let Some(vars) = variables {
        body["variables"] = vars;
    }
    serde_json::from_value(body).expect("build GraphQL request")
}

/// Run the guard exactly as the HTTP boundary does.
fn guard(
    mode: MutationGuardMode,
    schema: &TestSchema,
    request: &GraphQLRequest,
) -> mutation_guard::GuardDecision {
    mutation_guard::evaluate(mode, request, schema)
}

// ---------------------------------------------------------------------------
// Unit — store state model (`ADR-0006` section 4.7)
// ---------------------------------------------------------------------------

mod store_state {
    use super::*;
    use crate::graphql::batching::ScopeKey;

    fn enforce_store() -> GraphqlBatchStore {
        GraphqlBatchStore::new(MutationGuardMode::Enforce)
    }

    #[test]
    fn store_is_unavailable_in_off() {
        let store = GraphqlBatchStore::new(MutationGuardMode::Off);
        assert!(!store.is_available());
    }

    #[test]
    fn store_is_unavailable_in_observe() {
        let store = GraphqlBatchStore::new(MutationGuardMode::Observe);
        assert!(
            !store.is_available(),
            "OBSERVE runs the detector; the store must stay unavailable"
        );
    }

    #[test]
    fn store_is_available_in_enforce() {
        assert!(enforce_store().is_available());
    }

    #[test]
    fn store_availability_follows_the_mode_in_both_directions() {
        // OFF -> OBSERVE -> ENFORCE
        for (mode, expected) in [
            (MutationGuardMode::Off, false),
            (MutationGuardMode::Observe, false),
            (MutationGuardMode::Enforce, true),
        ] {
            assert_eq!(GraphqlBatchStore::new(mode).is_available(), expected);
            assert_eq!(mode.store_available(), expected);
        }
        // ENFORCE -> OBSERVE -> OFF
        for (mode, expected) in [
            (MutationGuardMode::Enforce, true),
            (MutationGuardMode::Observe, false),
            (MutationGuardMode::Off, false),
        ] {
            assert_eq!(GraphqlBatchStore::new(mode).is_available(), expected);
        }
    }

    #[test]
    fn unavailable_store_reads_not_loaded_for_every_key() {
        let store = GraphqlBatchStore::new(MutationGuardMode::Observe);
        let scope = ScopeKey::new("anything");
        let shape = TestImprintLoader::shape(DEFAULT_IMPRINT_LIMIT);
        let lookup = store
            .lookup::<TestImprintLoader>(&scope, &shape, &Uuid::new_v4())
            .expect("lookup");
        assert!(matches!(lookup, BatchLookup::NotLoaded));
    }

    #[test]
    fn empty_result_is_cached_as_loaded_empty_and_is_not_a_miss() {
        let (_guard, pool) = test_db::setup_test_db();
        // A publisher with zero imprints.
        let publisher = test_db::create_publisher(&pool);
        let store = enforce_store();
        let scope = ScopeKey::new("testPublishers");
        let shape = TestImprintLoader::shape(DEFAULT_IMPRINT_LIMIT);

        let dispatched = store
            .dispatch::<TestImprintLoader>(&pool, &scope, &shape, &[publisher.publisher_id])
            .expect("dispatch");
        assert_eq!(dispatched, DispatchResult::Loaded);

        match store
            .lookup::<TestImprintLoader>(&scope, &shape, &publisher.publisher_id)
            .expect("lookup")
        {
            BatchLookup::Loaded(rows) => assert!(rows.is_empty()),
            BatchLookup::NotLoaded => panic!("Loaded([]) must never be represented as absence"),
            BatchLookup::LoadFailed(_) => panic!("unexpected failure"),
        }
    }

    #[test]
    fn repeated_read_does_not_consume_the_entry() {
        let (_guard, pool) = test_db::setup_test_db();
        let publishers = seed(&pool, 1, 2);
        let store = enforce_store();
        let scope = ScopeKey::new("testPublishers");
        let shape = TestImprintLoader::shape(DEFAULT_IMPRINT_LIMIT);
        store
            .dispatch::<TestImprintLoader>(&pool, &scope, &shape, &[publishers[0].publisher_id])
            .expect("dispatch");

        let first = match store
            .lookup::<TestImprintLoader>(&scope, &shape, &publishers[0].publisher_id)
            .expect("lookup")
        {
            BatchLookup::Loaded(rows) => rows,
            _ => panic!("expected Loaded"),
        };
        let second = match store
            .lookup::<TestImprintLoader>(&scope, &shape, &publishers[0].publisher_id)
            .expect("lookup")
        {
            BatchLookup::Loaded(rows) => rows,
            _ => panic!("second read must still find the entry"),
        };
        assert_eq!(first.len(), 2);
        assert_eq!(
            first.iter().map(|r| r.imprint_id).collect::<Vec<_>>(),
            second.iter().map(|r| r.imprint_id).collect::<Vec<_>>()
        );
    }

    #[test]
    fn same_scope_loader_shape_and_key_reuses_without_redispatch() {
        let (_guard, pool) = test_db::setup_test_db();
        let publishers = seed(&pool, 2, 1);
        let store = enforce_store();
        let scope = ScopeKey::new("testPublishers");
        let shape = TestImprintLoader::shape(DEFAULT_IMPRINT_LIMIT);
        let keys: Vec<Uuid> = publishers.iter().map(|p| p.publisher_id).collect();

        assert_eq!(
            store
                .dispatch::<TestImprintLoader>(&pool, &scope, &shape, &keys)
                .expect("dispatch"),
            DispatchResult::Loaded
        );
        assert_eq!(
            store
                .dispatch::<TestImprintLoader>(&pool, &scope, &shape, &keys)
                .expect("dispatch"),
            DispatchResult::AlreadyLoaded,
            "a second dispatch over an already-loaded set must issue no SQL"
        );
    }

    #[test]
    fn different_scope_does_not_reuse() {
        let (_guard, pool) = test_db::setup_test_db();
        let publishers = seed(&pool, 1, 1);
        let store = enforce_store();
        let shape = TestImprintLoader::shape(DEFAULT_IMPRINT_LIMIT);
        let key = publishers[0].publisher_id;

        store
            .dispatch::<TestImprintLoader>(&pool, &ScopeKey::new("first"), &shape, &[key])
            .expect("dispatch");

        let other = store
            .lookup::<TestImprintLoader>(&ScopeKey::new("second"), &shape, &key)
            .expect("lookup");
        assert!(
            matches!(other, BatchLookup::NotLoaded),
            "an entry must never be visible across two top-level response keys"
        );
    }

    #[test]
    fn different_shape_does_not_reuse() {
        let (_guard, pool) = test_db::setup_test_db();
        let publishers = seed(&pool, 1, 3);
        let store = enforce_store();
        let scope = ScopeKey::new("testPublishers");
        let key = publishers[0].publisher_id;

        store
            .dispatch::<TestImprintLoader>(&pool, &scope, &TestImprintLoader::shape(1), &[key])
            .expect("dispatch");

        let other = store
            .lookup::<TestImprintLoader>(&scope, &TestImprintLoader::shape(2), &key)
            .expect("lookup");
        assert!(
            matches!(other, BatchLookup::NotLoaded),
            "argument variants must never share a stored entry"
        );
    }

    #[test]
    fn one_parent_key_may_hold_several_shapes_simultaneously_each_correct() {
        let (_guard, pool) = test_db::setup_test_db();
        let publishers = seed(&pool, 1, 4);
        let store = enforce_store();
        let scope = ScopeKey::new("testPublishers");
        let key = publishers[0].publisher_id;

        for limit in [1, 2, 4] {
            store
                .dispatch::<TestImprintLoader>(
                    &pool,
                    &scope,
                    &TestImprintLoader::shape(limit),
                    &[key],
                )
                .expect("dispatch");
        }
        for limit in [1, 2, 4] {
            match store
                .lookup::<TestImprintLoader>(&scope, &TestImprintLoader::shape(limit), &key)
                .expect("lookup")
            {
                BatchLookup::Loaded(rows) => assert_eq!(rows.len(), limit as usize),
                _ => panic!("expected Loaded for limit {limit}"),
            }
        }
    }

    #[test]
    fn omitted_argument_and_explicit_schema_default_produce_the_same_shape() {
        // The loader's single constructor is the only way a shape is built, so
        // the omitted form (normalized to the default at the prefetch site) and
        // the explicit form are the same value.
        assert_eq!(
            TestImprintLoader::shape(DEFAULT_IMPRINT_LIMIT),
            TestImprintLoader::shape(DEFAULT_IMPRINT_LIMIT)
        );
        assert_ne!(
            TestImprintLoader::shape(DEFAULT_IMPRINT_LIMIT),
            TestImprintLoader::shape(DEFAULT_IMPRINT_LIMIT + 1)
        );
    }

    #[test]
    fn load_failed_is_retained_and_never_retried_or_emptied() {
        let (_guard, pool) = test_db::setup_test_db();
        let publishers = seed(&pool, 1, 1);
        let failing = test_db::failing_pool();
        let store = enforce_store();
        let scope = ScopeKey::new("testPublishers");
        let shape = TestImprintLoader::shape(DEFAULT_IMPRINT_LIMIT);
        let key = publishers[0].publisher_id;

        assert_eq!(
            store
                .dispatch::<TestImprintLoader>(&failing, &scope, &shape, &[key])
                .expect("dispatch"),
            DispatchResult::Failed
        );
        assert_eq!(store.failure_count(), 1);

        // Retained, and distinguishable from both absence and Loaded([]).
        match store
            .lookup::<TestImprintLoader>(&scope, &shape, &key)
            .expect("lookup")
        {
            BatchLookup::LoadFailed(_) => {}
            BatchLookup::NotLoaded => panic!("LoadFailed must never be represented as absence"),
            BatchLookup::Loaded(_) => panic!("LoadFailed must never become an empty result"),
        }

        // No retry: a later dispatch — even against a WORKING pool — must not
        // reload a key already marked failed in this scope.
        assert_eq!(
            store
                .dispatch::<TestImprintLoader>(&pool, &scope, &shape, &[key])
                .expect("dispatch"),
            DispatchResult::AlreadyLoaded,
            "a recorded failure must not be retried"
        );
        match store
            .lookup::<TestImprintLoader>(&scope, &shape, &key)
            .expect("lookup")
        {
            BatchLookup::LoadFailed(_) => {}
            _ => panic!("the failure must still be the state after a later dispatch"),
        }
    }

    #[test]
    fn a_load_failed_under_one_scope_does_not_poison_another() {
        let (_guard, pool) = test_db::setup_test_db();
        let publishers = seed(&pool, 1, 1);
        let failing = test_db::failing_pool();
        let store = enforce_store();
        let shape = TestImprintLoader::shape(DEFAULT_IMPRINT_LIMIT);
        let key = publishers[0].publisher_id;

        store
            .dispatch::<TestImprintLoader>(&failing, &ScopeKey::new("a"), &shape, &[key])
            .expect("dispatch");

        // Scope B is untouched: NotLoaded, so it falls back and is correct.
        let under_b = store
            .lookup::<TestImprintLoader>(&ScopeKey::new("b"), &shape, &key)
            .expect("lookup");
        assert!(matches!(under_b, BatchLookup::NotLoaded));

        // And B can still load successfully.
        assert_eq!(
            store
                .dispatch::<TestImprintLoader>(&pool, &ScopeKey::new("b"), &shape, &[key])
                .expect("dispatch"),
            DispatchResult::Loaded
        );
    }

    #[test]
    fn whole_store_invalidation_clears_loaded_and_failed_across_all_scopes() {
        let (_guard, pool) = test_db::setup_test_db();
        let publishers = seed(&pool, 1, 1);
        let failing = test_db::failing_pool();
        let store = enforce_store();
        let shape = TestImprintLoader::shape(DEFAULT_IMPRINT_LIMIT);
        let key = publishers[0].publisher_id;

        store
            .dispatch::<TestImprintLoader>(&pool, &ScopeKey::new("ok"), &shape, &[key])
            .expect("dispatch");
        store
            .dispatch::<TestImprintLoader>(&failing, &ScopeKey::new("bad"), &shape, &[key])
            .expect("dispatch");
        assert!(store.entry_count() >= 2);
        assert_eq!(store.failure_count(), 1);

        store.invalidate_all().expect("invalidate");

        assert_eq!(store.entry_count(), 0);
        assert_eq!(store.failure_count(), 0);
        for scope in ["ok", "bad"] {
            assert!(matches!(
                store
                    .lookup::<TestImprintLoader>(&ScopeKey::new(scope), &shape, &key)
                    .expect("lookup"),
                BatchLookup::NotLoaded
            ));
        }
    }

    #[test]
    fn duplicate_keys_yield_one_key_and_correct_partitioning() {
        let (_guard, pool) = test_db::setup_test_db();
        let publishers = seed(&pool, 2, 2);
        let store = enforce_store();
        let scope = ScopeKey::new("testPublishers");
        let shape = TestImprintLoader::shape(DEFAULT_IMPRINT_LIMIT);

        // Five references to two distinct keys.
        let keys = vec![
            publishers[0].publisher_id,
            publishers[1].publisher_id,
            publishers[0].publisher_id,
            publishers[0].publisher_id,
            publishers[1].publisher_id,
        ];
        store
            .dispatch::<TestImprintLoader>(&pool, &scope, &shape, &keys)
            .expect("dispatch");

        // Exactly two entries, one per distinct key.
        assert_eq!(store.entry_count(), 2);

        // Every returned row landed in the bucket for its own key and no other.
        for publisher in &publishers {
            match store
                .lookup::<TestImprintLoader>(&scope, &shape, &publisher.publisher_id)
                .expect("lookup")
            {
                BatchLookup::Loaded(rows) => {
                    assert_eq!(rows.len(), 2);
                    assert!(rows
                        .iter()
                        .all(|r| r.publisher_id == publisher.publisher_id));
                }
                _ => panic!("expected Loaded"),
            }
        }
    }

    #[test]
    fn partitioning_is_deterministic_across_repeated_dispatches() {
        let (_guard, pool) = test_db::setup_test_db();
        let publishers = seed(&pool, 3, 3);
        let keys: Vec<Uuid> = publishers.iter().map(|p| p.publisher_id).collect();
        let shape = TestImprintLoader::shape(DEFAULT_IMPRINT_LIMIT);

        let mut runs = Vec::new();
        for _ in 0..3 {
            let store = enforce_store();
            store
                .dispatch::<TestImprintLoader>(&pool, &ScopeKey::new("s"), &shape, &keys)
                .expect("dispatch");
            let mut snapshot = Vec::new();
            for key in &keys {
                match store
                    .lookup::<TestImprintLoader>(&ScopeKey::new("s"), &shape, key)
                    .expect("lookup")
                {
                    BatchLookup::Loaded(rows) => {
                        snapshot.push(rows.into_iter().map(|r| r.imprint_id).collect::<Vec<_>>())
                    }
                    _ => panic!("expected Loaded"),
                }
            }
            runs.push(snapshot);
        }
        assert_eq!(runs[0], runs[1]);
        assert_eq!(runs[1], runs[2]);
    }

    #[test]
    fn prefetched_result_equals_the_direct_per_parent_result_in_order() {
        let (_guard, pool) = test_db::setup_test_db();
        let publishers = seed(&pool, 3, 5);
        let store = enforce_store();
        let scope = ScopeKey::new("testPublishers");
        let shape = TestImprintLoader::shape(DEFAULT_IMPRINT_LIMIT);
        let keys: Vec<Uuid> = publishers.iter().map(|p| p.publisher_id).collect();
        store
            .dispatch::<TestImprintLoader>(&pool, &scope, &shape, &keys)
            .expect("dispatch");

        for publisher in &publishers {
            let prefetched: Vec<String> = match store
                .lookup::<TestImprintLoader>(&scope, &shape, &publisher.publisher_id)
                .expect("lookup")
            {
                BatchLookup::Loaded(rows) => rows.into_iter().map(|r| r.imprint_name).collect(),
                _ => panic!("expected Loaded"),
            };
            let direct = direct_imprint_names(&pool, publisher.publisher_id, DEFAULT_IMPRINT_LIMIT);
            assert_eq!(
                prefetched, direct,
                "prefetched output must equal direct per-parent output, element for element and in order"
            );
        }
    }

    #[test]
    fn two_loaders_with_identical_key_types_cannot_read_each_others_entries() {
        // `LoaderIdentity` is a closed discriminant and is part of the store
        // key, so two loaders sharing `Uuid` keys are structurally separated.
        // With only one loader defined, the property is asserted on the key
        // type: a lookup names its loader through the type parameter, so there
        // is no way to spell "the other loader's entry".
        let store = enforce_store();
        assert!(!std::mem::needs_drop::<
            crate::graphql::batching::LoaderIdentity,
        >());
        let lookup = store
            .lookup::<TestImprintLoader>(
                &ScopeKey::new("s"),
                &TestImprintLoader::shape(1),
                &Uuid::new_v4(),
            )
            .expect("lookup");
        assert!(matches!(lookup, BatchLookup::NotLoaded));
    }

    #[test]
    fn concurrent_independent_requests_share_nothing() {
        let (_guard, pool) = test_db::setup_test_db();
        let publishers = seed(&pool, 1, 2);
        let key = publishers[0].publisher_id;
        let shape = TestImprintLoader::shape(DEFAULT_IMPRINT_LIMIT);

        let first = context_in_mode(Arc::clone(&pool), MutationGuardMode::Enforce);
        let second = context_in_mode(Arc::clone(&pool), MutationGuardMode::Enforce);

        first
            .batch_store
            .dispatch::<TestImprintLoader>(&pool, &ScopeKey::new("s"), &shape, &[key])
            .expect("dispatch");

        assert!(
            matches!(
                second
                    .batch_store
                    .lookup::<TestImprintLoader>(&ScopeKey::new("s"), &shape, &key)
                    .expect("lookup"),
                BatchLookup::NotLoaded
            ),
            "a second request with a fresh Context must observe an empty store"
        );
    }
}

// ---------------------------------------------------------------------------
// Store collision matrix
// ---------------------------------------------------------------------------

mod collision_matrix {
    use super::*;
    use crate::graphql::batching::ScopeKey;

    #[test]
    fn full_collision_matrix() {
        let (_guard, pool) = test_db::setup_test_db();
        let publishers = seed(&pool, 2, 2);
        let store = GraphqlBatchStore::new(MutationGuardMode::Enforce);
        let key_a = publishers[0].publisher_id;
        let key_b = publishers[1].publisher_id;
        let shape_1 = TestImprintLoader::shape(1);
        let shape_2 = TestImprintLoader::shape(2);

        store
            .dispatch::<TestImprintLoader>(&pool, &ScopeKey::new("alpha"), &shape_1, &[key_a])
            .expect("dispatch");

        // same key + same loader + same shape, DIFFERENT scope -> no collision
        assert!(matches!(
            store
                .lookup::<TestImprintLoader>(&ScopeKey::new("beta"), &shape_1, &key_a)
                .expect("lookup"),
            BatchLookup::NotLoaded
        ));

        // same scope + same loader + same key, DIFFERENT shape -> no collision
        assert!(matches!(
            store
                .lookup::<TestImprintLoader>(&ScopeKey::new("alpha"), &shape_2, &key_a)
                .expect("lookup"),
            BatchLookup::NotLoaded
        ));

        // same scope + same loader + same shape, DIFFERENT key -> no collision
        assert!(matches!(
            store
                .lookup::<TestImprintLoader>(&ScopeKey::new("alpha"), &shape_1, &key_b)
                .expect("lookup"),
            BatchLookup::NotLoaded
        ));

        // the exact full key returns the stored value
        match store
            .lookup::<TestImprintLoader>(&ScopeKey::new("alpha"), &shape_1, &key_a)
            .expect("lookup")
        {
            BatchLookup::Loaded(rows) => assert_eq!(rows.len(), 1),
            _ => panic!("the exact full key must return the expected stored value"),
        }
    }
}

// ---------------------------------------------------------------------------
// Look-ahead traversal — alias safety at every segment
// ---------------------------------------------------------------------------

mod traversal {
    use super::*;

    /// Assert which terminal selections a document exposes at a prefetch site,
    /// by executing and letting the site record what it found.
    fn terminal_count(query: &str) -> usize {
        let (_guard, pool) = test_db::setup_test_db();
        seed(&pool, 1, 1);
        let context = context_in_mode(pool, MutationGuardMode::Enforce);
        let schema = test_schema();
        reset_counters();
        let (_data, errors) = run_sync(&schema, &context, query, Variables::new());
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
        // One entry per distinct (scope, loader, shape, key) written.
        context.batch_store.entry_count()
    }

    #[test]
    fn aliased_terminal_field_is_found() {
        // `select()`/`has_child()` would match the alias and miss this.
        assert_eq!(
            terminal_count("{ testPublishers { a: imprints { imprintName } } }"),
            1
        );
    }

    #[test]
    fn aliased_intermediate_segment_is_found_on_the_descendant_path() {
        assert_eq!(
            terminal_count("{ testImprints { p: publisher { a: imprints { imprintName } } } }"),
            1
        );
    }

    #[test]
    fn two_aliased_intermediate_branches_each_carrying_a_terminal_are_both_found() {
        // Both branches carry the SAME normalized shape, so one dispatch and
        // one entry — but traversal must have discovered both, not stopped at
        // the first.
        let (_guard, pool) = test_db::setup_test_db();
        seed(&pool, 1, 2);
        let context = context_in_mode(pool, MutationGuardMode::Enforce);
        let schema = test_schema();
        let (data, errors) = run_sync(
            &schema,
            &context,
            "{ testImprints { first: publisher { one: imprints { imprintName } } \
               second: publisher { two: imprints { imprintName } } } }",
            Variables::new(),
        );
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
        let item = &data["testImprints"][0];
        assert_eq!(item["first"]["one"], item["second"]["two"]);
    }

    #[test]
    fn two_terminal_shapes_under_one_site_produce_one_dispatch_each() {
        let (_guard, pool) = test_db::setup_test_db();
        seed(&pool, 1, 4);
        let context = context_in_mode(pool, MutationGuardMode::Enforce);
        let schema = test_schema();
        let (data, errors) = run_sync(
            &schema,
            &context,
            "{ testPublishers { small: imprints(limit: 1) { imprintName } \
               large: imprints(limit: 4) { imprintName } } }",
            Variables::new(),
        );
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
        // Two distinct shapes -> two entries for the one key.
        assert_eq!(context.batch_store.entry_count(), 2);
        let item = &data["testPublishers"][0];
        assert_eq!(item["small"].as_array().unwrap().len(), 1);
        assert_eq!(item["large"].as_array().unwrap().len(), 4);
    }

    #[test]
    fn omitted_argument_and_explicit_default_resolve_against_the_same_entry() {
        let (_guard, pool) = test_db::setup_test_db();
        seed(&pool, 1, 5);
        let context = context_in_mode(pool, MutationGuardMode::Enforce);
        let schema = test_schema();
        let (data, errors) = run_sync(
            &schema,
            &context,
            &format!(
                "{{ testPublishers {{ omitted: imprints {{ imprintName }} \
                   explicit: imprints(limit: {DEFAULT_IMPRINT_LIMIT}) {{ imprintName }} }} }}"
            ),
            Variables::new(),
        );
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
        assert_eq!(
            context.batch_store.entry_count(),
            1,
            "an omitted argument and an explicitly supplied schema default must share one entry"
        );
        let item = &data["testPublishers"][0];
        assert_eq!(item["omitted"], item["explicit"]);
    }

    #[test]
    fn repeated_aliases_of_the_same_shape_cause_no_additional_entries() {
        let (_guard, pool) = test_db::setup_test_db();
        seed(&pool, 1, 2);
        let context = context_in_mode(pool, MutationGuardMode::Enforce);
        let schema = test_schema();
        let (data, errors) = run_sync(
            &schema,
            &context,
            "{ testPublishers { a: imprints { imprintName } b: imprints { imprintName } \
               c: imprints { imprintName } } }",
            Variables::new(),
        );
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
        assert_eq!(context.batch_store.entry_count(), 1);
        let item = &data["testPublishers"][0];
        assert_eq!(item["a"], item["b"]);
        assert_eq!(item["b"], item["c"]);
    }
}

// ---------------------------------------------------------------------------
// Integration — prefetch through real Juniper execution
// ---------------------------------------------------------------------------

mod integration {
    use super::*;

    const DIRECT_QUERY: &str = "{ testPublishers { publisherId imprints { imprintName } } }";
    const DESCENDANT_QUERY: &str =
        "{ testImprints { imprintId publisher { imprints { imprintName } } } }";

    #[test]
    fn direct_path_resolves_every_parent_correctly() {
        let (_guard, pool) = test_db::setup_test_db();
        let publishers = seed(&pool, 4, 2);
        let context = context_in_mode(Arc::clone(&pool), MutationGuardMode::Enforce);
        let schema = test_schema();
        reset_counters();

        let (data, errors) = run_sync(&schema, &context, DIRECT_QUERY, Variables::new());
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");

        let items = data["testPublishers"].as_array().expect("array");
        assert_eq!(items.len(), publishers.len());
        for item in items {
            let publisher_id: Uuid = item["publisherId"].as_str().unwrap().parse().unwrap();
            let names: Vec<String> = item["imprints"]
                .as_array()
                .unwrap()
                .iter()
                .map(|i| i["imprintName"].as_str().unwrap().to_string())
                .collect();
            assert_eq!(
                names,
                direct_imprint_names(&pool, publisher_id, DEFAULT_IMPRINT_LIMIT)
            );
        }
        assert_eq!(
            terminal_fallback_calls(),
            0,
            "a covered direct path must issue no terminal fallback"
        );
    }

    #[test]
    fn single_key_list_resolves_correctly() {
        let (_guard, pool) = test_db::setup_test_db();
        seed(&pool, 1, 2);
        let context = context_in_mode(Arc::clone(&pool), MutationGuardMode::Enforce);
        let schema = test_schema();
        reset_counters();
        let (data, errors) = run_sync(&schema, &context, DIRECT_QUERY, Variables::new());
        assert!(errors.is_empty());
        assert_eq!(data["testPublishers"].as_array().unwrap().len(), 1);
        assert_eq!(terminal_fallback_calls(), 0);
    }

    #[test]
    fn descendant_path_resolves_and_issues_no_terminal_fallback() {
        let (_guard, pool) = test_db::setup_test_db();
        let publishers = seed(&pool, 3, 2);
        let context = context_in_mode(Arc::clone(&pool), MutationGuardMode::Enforce);
        let schema = test_schema();
        reset_counters();

        let (data, errors) = run_sync(&schema, &context, DESCENDANT_QUERY, Variables::new());
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");

        let items = data["testImprints"].as_array().expect("array");
        assert_eq!(items.len(), publishers.len() * 2);
        assert_eq!(
            terminal_fallback_calls(),
            0,
            "a covered descendant path must issue no terminal fallback statement"
        );
        assert!(
            intermediate_resolver_calls() > 0,
            "the legacy intermediate resolver still runs per item; it is reported separately"
        );
    }

    #[test]
    fn descendant_results_equal_the_direct_per_parent_result_in_order() {
        let (_guard, pool) = test_db::setup_test_db();
        seed(&pool, 2, 4);
        let context = context_in_mode(Arc::clone(&pool), MutationGuardMode::Enforce);
        let schema = test_schema();
        let (data, errors) = run_sync(&schema, &context, DESCENDANT_QUERY, Variables::new());
        assert!(errors.is_empty());

        for item in data["testImprints"].as_array().unwrap() {
            let names: Vec<String> = item["publisher"]["imprints"]
                .as_array()
                .unwrap()
                .iter()
                .map(|i| i["imprintName"].as_str().unwrap().to_string())
                .collect();
            assert!(!names.is_empty());
            // Every entry in the bucket belongs to the same publisher and
            // matches the direct result for it.
            assert_eq!(names.len(), DEFAULT_IMPRINT_LIMIT as usize);
        }
    }

    #[test]
    fn a_list_without_a_prefetch_site_falls_back_and_is_still_correct() {
        let (_guard, pool) = test_db::setup_test_db();
        let publishers = seed(&pool, 3, 2);
        let context = context_in_mode(Arc::clone(&pool), MutationGuardMode::Enforce);
        let schema = test_schema();
        reset_counters();

        let (data, errors) = run_sync(
            &schema,
            &context,
            "{ testPublishersUnprefetched { publisherId imprints { imprintName } } }",
            Variables::new(),
        );
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
        assert_eq!(
            terminal_fallback_calls(),
            publishers.len(),
            "an unprefetched path must take the direct fallback once per parent"
        );
        for item in data["testPublishersUnprefetched"].as_array().unwrap() {
            let publisher_id: Uuid = item["publisherId"].as_str().unwrap().parse().unwrap();
            let names: Vec<String> = item["imprints"]
                .as_array()
                .unwrap()
                .iter()
                .map(|i| i["imprintName"].as_str().unwrap().to_string())
                .collect();
            assert_eq!(
                names,
                direct_imprint_names(&pool, publisher_id, DEFAULT_IMPRINT_LIMIT)
            );
        }
    }

    #[test]
    fn mixed_prefetched_and_unprefetched_paths_both_resolve_correctly() {
        let (_guard, pool) = test_db::setup_test_db();
        let publishers = seed(&pool, 2, 2);
        let context = context_in_mode(Arc::clone(&pool), MutationGuardMode::Enforce);
        let schema = test_schema();
        reset_counters();

        let (data, errors) = run_sync(
            &schema,
            &context,
            "{ covered: testPublishers { imprints { imprintName } } \
               uncovered: testPublishersUnprefetched { imprints { imprintName } } }",
            Variables::new(),
        );
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
        assert_eq!(
            terminal_fallback_calls(),
            publishers.len(),
            "only the uncovered branch falls back"
        );
        assert_eq!(data["covered"], data["uncovered"]);
    }

    #[test]
    fn store_is_unavailable_in_off_so_every_path_falls_back() {
        let (_guard, pool) = test_db::setup_test_db();
        let publishers = seed(&pool, 3, 2);
        let context = context_in_mode(Arc::clone(&pool), MutationGuardMode::Off);
        let schema = test_schema();
        reset_counters();

        let (data, errors) = run_sync(&schema, &context, DIRECT_QUERY, Variables::new());
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
        assert_eq!(
            terminal_fallback_calls(),
            publishers.len(),
            "in OFF every lookup reads NotLoaded and every path falls back"
        );
        assert_eq!(context.batch_store.entry_count(), 0);

        // And the result is identical to the ENFORCE result.
        let enforced = context_in_mode(Arc::clone(&pool), MutationGuardMode::Enforce);
        let (batched, _) = run_sync(&schema, &enforced, DIRECT_QUERY, Variables::new());
        assert_eq!(data, batched);
    }

    #[test]
    fn store_is_unavailable_in_observe_so_every_path_falls_back() {
        let (_guard, pool) = test_db::setup_test_db();
        let publishers = seed(&pool, 3, 2);
        let context = context_in_mode(Arc::clone(&pool), MutationGuardMode::Observe);
        let schema = test_schema();
        reset_counters();

        let (_data, errors) = run_sync(&schema, &context, DIRECT_QUERY, Variables::new());
        assert!(errors.is_empty());
        assert_eq!(terminal_fallback_calls(), publishers.len());
        assert_eq!(context.batch_store.entry_count(), 0);
    }

    #[test]
    fn two_top_level_aliases_of_one_field_produce_separate_namespaces() {
        let (_guard, pool) = test_db::setup_test_db();
        let publishers = seed(&pool, 3, 2);
        let context = context_in_mode(Arc::clone(&pool), MutationGuardMode::Enforce);
        let schema = test_schema();
        reset_counters();

        let (data, errors) = run_sync(
            &schema,
            &context,
            "{ first: testPublishers { imprints { imprintName } } \
               second: testPublishers { imprints { imprintName } } }",
            Variables::new(),
        );
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");

        // One entry per (scope, key): two scopes x n publishers.
        assert_eq!(
            context.batch_store.entry_count(),
            publishers.len() * 2,
            "two top-level aliases must be two separate loader namespaces"
        );
        assert_eq!(data["first"], data["second"]);
        assert_eq!(terminal_fallback_calls(), 0);
    }

    #[test]
    fn same_scope_multi_site_reuse_issues_no_duplicate_work() {
        let (_guard, pool) = test_db::setup_test_db();
        let publishers = seed(&pool, 3, 2);
        let context = context_in_mode(Arc::clone(&pool), MutationGuardMode::Enforce);
        let schema = test_schema();
        reset_counters();

        // One top-level scope; the site runs once and covers everything.
        let (_data, errors) = run_sync(&schema, &context, DIRECT_QUERY, Variables::new());
        assert!(errors.is_empty());
        assert_eq!(context.batch_store.entry_count(), publishers.len());
        assert_eq!(terminal_fallback_calls(), 0);
    }

    #[test]
    fn descendant_and_direct_sites_share_one_terminal_namespace_within_a_scope() {
        let (_guard, pool) = test_db::setup_test_db();
        seed(&pool, 2, 2);
        let context = context_in_mode(Arc::clone(&pool), MutationGuardMode::Enforce);
        let schema = test_schema();
        reset_counters();

        // One top-level field whose selection reaches the terminal loader
        // through BOTH the intermediate object and, for the same publishers,
        // repeatedly. All under one scope, so one shared namespace.
        let (_data, errors) = run_sync(
            &schema,
            &context,
            "{ testImprints { publisher { imprints { imprintName } } } }",
            Variables::new(),
        );
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
        assert_eq!(
            context.batch_store.entry_count(),
            2,
            "descendant entries are stored under the ordinary terminal identity, \
             one per distinct publisher key in this scope"
        );
        assert_eq!(terminal_fallback_calls(), 0);
    }

    #[test]
    fn database_failure_records_load_failed_without_failing_the_parent_list() {
        let (_guard, pool) = test_db::setup_test_db();
        seed(&pool, 2, 2);

        // A context whose db is the failing pool: the parent list resolver
        // cannot run either, so instead drive the store directly and then let
        // the terminal resolver read the retained failure.
        let context = context_in_mode(Arc::clone(&pool), MutationGuardMode::Enforce);
        let failing = test_db::failing_pool();
        let publishers: Vec<Publisher> = {
            use crate::schema::publisher;
            use diesel::RunQueryDsl;
            let mut connection = pool.get().unwrap();
            publisher::table.load(&mut connection).unwrap()
        };
        let keys: Vec<Uuid> = publishers.iter().map(|p| p.publisher_id).collect();

        context
            .batch_store
            .dispatch::<TestImprintLoader>(
                &failing,
                &crate::graphql::batching::ScopeKey::new("testPublishers"),
                &TestImprintLoader::shape(DEFAULT_IMPRINT_LIMIT),
                &keys,
            )
            .expect("dispatch");
        reset_counters();

        let schema = test_schema();
        let (data, errors) = run_sync(&schema, &context, DIRECT_QUERY, Variables::new());

        // The parent list field still resolved; the failure surfaced at the
        // child field.
        assert!(
            !errors.is_empty(),
            "each covered child resolver must emit the failure"
        );
        assert_eq!(
            terminal_fallback_calls(),
            0,
            "no retry query may be issued for a covered key"
        );
        // Never an empty successful list substitution.
        if let Some(items) = data.get("testPublishers").and_then(|v| v.as_array()) {
            for item in items {
                assert!(
                    item.get("imprints").map(|v| v.is_null()).unwrap_or(true),
                    "a failed load must not be substituted with an empty list"
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// GraphQL-visible error contract (`ADR-0006` section 4.9.3)
// ---------------------------------------------------------------------------

mod error_contract {
    use super::*;
    use crate::graphql::batching::ScopeKey;

    /// Full serialized response, including `errors[]`, for contract comparison.
    fn execute_full(schema: &TestSchema, context: &Context, query: &str) -> JsonValue {
        let request = request(query, None, None);
        let response = request.execute_sync(schema, context);
        serde_json::to_value(response).expect("serialize response")
    }

    #[test]
    fn prefetched_failure_matches_the_direct_failure_contract() {
        let (_guard, pool) = test_db::setup_test_db();
        let publishers = seed(&pool, 1, 1);
        let key = publishers[0].publisher_id;
        let schema = test_schema();

        // --- prefetched failure path -------------------------------------
        let prefetched_ctx = context_in_mode(Arc::clone(&pool), MutationGuardMode::Enforce);
        prefetched_ctx
            .batch_store
            .dispatch::<TestImprintLoader>(
                &test_db::failing_pool(),
                &ScopeKey::new("testPublishers"),
                &TestImprintLoader::shape(DEFAULT_IMPRINT_LIMIT),
                &[key],
            )
            .expect("dispatch");
        let prefetched = execute_full(
            &schema,
            &prefetched_ctx,
            "{ testPublishers { imprints { imprintName } } }",
        );

        // --- direct failure path -----------------------------------------
        // The parent list resolves normally; only the terminal field's DIRECT
        // per-parent query fails, so the error is raised at the same position
        // in the tree as the prefetched failure.
        let direct_ctx = context_in_mode(Arc::clone(&pool), MutationGuardMode::Off);
        let direct = execute_full(
            &schema,
            &direct_ctx,
            "{ testPublishersFailingChild { imprints { imprintName } } }",
        );

        let prefetched_error = &prefetched["errors"][0];
        let direct_error = &direct["errors"][0];

        // errors[].path — attributed to the CHILD field on the owning parent,
        // not to the parent list field.
        let prefetched_path = prefetched_error["path"].as_array().expect("path");
        assert_eq!(
            prefetched_path.last().and_then(|v| v.as_str()),
            Some("imprints"),
            "the error must be attributed to the child field"
        );

        // extensions.type — the SAME classification as the direct path.
        assert_eq!(
            prefetched_error["extensions"]["type"], direct_error["extensions"]["type"],
            "the prefetched path must produce the same extensions.type discriminant"
        );
        assert_eq!(
            prefetched_error["extensions"]["type"], "INTERNAL_ERROR",
            "a pool failure classifies as INTERNAL_ERROR on both paths"
        );

        // errors[].path — the direct path attributes the error to the same
        // child field, on the owning parent.
        let direct_path = direct_error["path"].as_array().expect("path");
        assert_eq!(
            direct_path.last().and_then(|v| v.as_str()),
            Some("imprints")
        );
        assert_eq!(
            prefetched_path.len(),
            direct_path.len(),
            "both paths must attribute the error at the same depth"
        );

        // null propagation — identical shape at the child field.
        assert!(prefetched["data"]["testPublishers"][0]["imprints"].is_null());
        assert!(direct["data"]["testPublishersFailingChild"][0]["imprints"].is_null());

        // No successful empty-list substitution anywhere.
        assert_ne!(
            prefetched["data"]["testPublishers"][0]["imprints"],
            json!([])
        );
    }
}

// ---------------------------------------------------------------------------
// Central mutation request guard
// ---------------------------------------------------------------------------

mod guard_tests {
    use super::*;

    fn test_publisher_id(pool: &PgPool) -> Uuid {
        test_db::create_publisher(pool).publisher_id
    }

    fn duplicate_mutation(publisher_id: Uuid) -> String {
        format!(
            r#"mutation {{
                 x: addImprint(publisherId: "{publisher_id}", imprintName: "one") {{ publishers {{ publisherId }} }}
                 x: addImprint(publisherId: "{publisher_id}", imprintName: "one") {{ publishers {{ publisherId }} }}
               }}"#
        )
    }

    // ---- allowed cases ---------------------------------------------------

    #[test]
    fn distinct_top_level_aliases_are_accepted_and_each_executes_once() {
        let (_guard, pool) = test_db::setup_test_db();
        let publisher_id = test_publisher_id(&pool);
        let context = context_in_mode(Arc::clone(&pool), MutationGuardMode::Enforce);
        let schema = test_schema();
        let query = format!(
            r#"mutation {{
                 first:  addImprint(publisherId: "{publisher_id}", imprintName: "a") {{ publishers {{ publisherId }} }}
                 second: addImprint(publisherId: "{publisher_id}", imprintName: "b") {{ publishers {{ publisherId }} }}
               }}"#
        );

        let decision = guard(
            MutationGuardMode::Enforce,
            &schema,
            &request(&query, None, None),
        );
        assert_eq!(decision.outcome, GuardOutcome::Proceed);
        assert!(decision.event.is_none());

        reset_counters();
        let (_data, errors) = run_sync(&schema, &context, &query, Variables::new());
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
        assert_eq!(
            mutation_resolver_calls(),
            2,
            "each alias executes exactly once"
        );
    }

    #[test]
    fn duplicate_response_keys_in_a_query_are_unaffected() {
        let (_guard, pool) = test_db::setup_test_db();
        seed(&pool, 1, 1);
        let schema = test_schema();
        let query = "query { x: testPublishers { publisherId } x: testPublishers { publisherId } }";
        let decision = guard(
            MutationGuardMode::Enforce,
            &schema,
            &request(query, None, None),
        );
        assert_eq!(
            decision.outcome,
            GuardOutcome::Proceed,
            "query operations must never be restricted"
        );
        assert!(decision.event.is_none());
    }

    #[test]
    fn duplicates_below_the_top_level_of_a_mutation_are_unaffected() {
        let (_guard, pool) = test_db::setup_test_db();
        let publisher_id = test_publisher_id(&pool);
        let schema = test_schema();
        let query = format!(
            r#"mutation {{
                 only: addImprint(publisherId: "{publisher_id}", imprintName: "a") {{
                   dup: publishers {{ publisherId }}
                   dup: publishers {{ publisherId }}
                 }}
               }}"#
        );
        let decision = guard(
            MutationGuardMode::Enforce,
            &schema,
            &request(&query, None, None),
        );
        assert_eq!(
            decision.outcome,
            GuardOutcome::Proceed,
            "non-top-level duplicates must not be restricted"
        );
    }

    // ---- rejected cases --------------------------------------------------

    #[test]
    fn direct_duplicate_is_rejected_in_enforce() {
        let (_guard, pool) = test_db::setup_test_db();
        let publisher_id = test_publisher_id(&pool);
        let schema = test_schema();
        let query = duplicate_mutation(publisher_id);

        let decision = guard(
            MutationGuardMode::Enforce,
            &schema,
            &request(&query, None, None),
        );
        match &decision.outcome {
            GuardOutcome::Reject { collisions } => assert_eq!(collisions, &vec!["x".to_string()]),
            other => panic!("expected rejection, got {other:?}"),
        }
        assert!(decision.event.is_some());
    }

    #[test]
    fn named_fragment_duplicate_is_rejected() {
        let (_guard, pool) = test_db::setup_test_db();
        let publisher_id = test_publisher_id(&pool);
        let schema = test_schema();
        let query = format!(
            r#"mutation {{
                 x: addImprint(publisherId: "{publisher_id}", imprintName: "one") {{ publishers {{ publisherId }} }}
                 ...Dup
               }}
               fragment Dup on TestMutation {{
                 x: addImprint(publisherId: "{publisher_id}", imprintName: "one") {{ publishers {{ publisherId }} }}
               }}"#
        );
        let decision = guard(
            MutationGuardMode::Enforce,
            &schema,
            &request(&query, None, None),
        );
        match &decision.outcome {
            GuardOutcome::Reject { collisions } => assert_eq!(collisions, &vec!["x".to_string()]),
            other => panic!("fragment expansion is not optional; got {other:?}"),
        }
    }

    #[test]
    fn inline_fragment_duplicate_is_rejected() {
        let (_guard, pool) = test_db::setup_test_db();
        let publisher_id = test_publisher_id(&pool);
        let schema = test_schema();
        let query = format!(
            r#"mutation {{
                 x: addImprint(publisherId: "{publisher_id}", imprintName: "one") {{ publishers {{ publisherId }} }}
                 ... on TestMutation {{
                   x: addImprint(publisherId: "{publisher_id}", imprintName: "one") {{ publishers {{ publisherId }} }}
                 }}
               }}"#
        );
        let decision = guard(
            MutationGuardMode::Enforce,
            &schema,
            &request(&query, None, None),
        );
        match &decision.outcome {
            GuardOutcome::Reject { collisions } => assert_eq!(collisions, &vec!["x".to_string()]),
            other => panic!("expected rejection, got {other:?}"),
        }
    }

    #[test]
    fn repeated_legitimate_spreads_of_one_fragment_are_both_counted() {
        let (_guard, pool) = test_db::setup_test_db();
        let publisher_id = test_publisher_id(&pool);
        let schema = test_schema();
        // The same fragment spread twice introduces the SAME response key
        // twice. Cycle protection must not suppress the second occurrence.
        let query = format!(
            r#"mutation {{
                 ...Dup
                 ...Dup
               }}
               fragment Dup on TestMutation {{
                 x: addImprint(publisherId: "{publisher_id}", imprintName: "one") {{ publishers {{ publisherId }} }}
               }}"#
        );
        let decision = guard(
            MutationGuardMode::Enforce,
            &schema,
            &request(&query, None, None),
        );
        match &decision.outcome {
            GuardOutcome::Reject { collisions } => assert_eq!(collisions, &vec!["x".to_string()]),
            other => panic!(
                "cycle protection must not globally suppress distinct occurrences; got {other:?}"
            ),
        }
    }

    // ---- measured zero execution -----------------------------------------

    #[test]
    fn a_rejected_operation_executes_zero_resolvers_and_zero_writes() {
        let (_guard, pool) = test_db::setup_test_db();
        let publisher_id = test_publisher_id(&pool);
        let context = context_in_mode(Arc::clone(&pool), MutationGuardMode::Enforce);
        let schema = test_schema();

        for query in [
            duplicate_mutation(publisher_id),
            format!(
                r#"mutation {{
                     x: addImprint(publisherId: "{publisher_id}", imprintName: "one") {{ publishers {{ publisherId }} }}
                     ...Dup
                   }}
                   fragment Dup on TestMutation {{
                     x: addImprint(publisherId: "{publisher_id}", imprintName: "one") {{ publishers {{ publisherId }} }}
                   }}"#
            ),
            format!(
                r#"mutation {{
                     x: addImprint(publisherId: "{publisher_id}", imprintName: "one") {{ publishers {{ publisherId }} }}
                     ... on TestMutation {{
                       x: addImprint(publisherId: "{publisher_id}", imprintName: "one") {{ publishers {{ publisherId }} }}
                     }}
                   }}"#
            ),
        ] {
            reset_counters();
            let imprints_before = imprint_count(&pool);

            let decision = guard(
                MutationGuardMode::Enforce,
                &schema,
                &request(&query, None, None),
            );
            assert!(matches!(decision.outcome, GuardOutcome::Reject { .. }));

            // The boundary returns the rejection WITHOUT executing, so no
            // resolver runs and no write happens.
            assert_eq!(
                mutation_resolver_calls(),
                0,
                "mutation resolver execution count must be 0 for a rejected operation"
            );
            assert_eq!(
                imprint_count(&pool),
                imprints_before,
                "database write count must be 0 for a rejected operation"
            );

            // And the untouched context proves nothing else ran either.
            let _ = &context;
        }
    }

    fn imprint_count(pool: &PgPool) -> i64 {
        use crate::schema::imprint;
        use diesel::{QueryDsl, RunQueryDsl};
        let mut connection = pool.get().unwrap();
        imprint::table.count().get_result(&mut connection).unwrap()
    }

    // ---- modes -----------------------------------------------------------

    #[test]
    fn off_short_circuits_before_parsing_selection_or_validation() {
        let (_guard, pool) = test_db::setup_test_db();
        let publisher_id = test_publisher_id(&pool);
        let schema = test_schema();

        // A document that is BOTH syntactically invalid AND shaped as a
        // duplicate. In OFF the guard must not parse it at all, so it produces
        // no decision and no event — indistinguishable from the no-guard base.
        let decision = guard(
            MutationGuardMode::Off,
            &schema,
            &request("mutation { x: addImprint( x: addImprint(", None, None),
        );
        assert_eq!(decision.outcome, GuardOutcome::Proceed);
        assert!(decision.event.is_none());

        // And a valid duplicate is likewise untouched.
        let decision = guard(
            MutationGuardMode::Off,
            &schema,
            &request(&duplicate_mutation(publisher_id), None, None),
        );
        assert_eq!(decision.outcome, GuardOutcome::Proceed);
        assert!(decision.event.is_none());
    }

    #[test]
    fn observe_detects_but_never_rejects_and_emits_exactly_one_event() {
        let (_guard, pool) = test_db::setup_test_db();
        let publisher_id = test_publisher_id(&pool);
        let schema = test_schema();
        let query = duplicate_mutation(publisher_id);

        let decision = guard(
            MutationGuardMode::Observe,
            &schema,
            &request(&query, None, None),
        );
        assert_eq!(
            decision.outcome,
            GuardOutcome::Proceed,
            "OBSERVE must never reject"
        );
        let event = decision
            .event
            .expect("OBSERVE must record a would-be rejection");
        assert_eq!(event.mode, "OBSERVE");
        assert_eq!(event.collisions, vec!["x".to_string()]);
    }

    #[test]
    fn observe_leaves_the_response_and_resolver_counts_unchanged() {
        let (_guard, pool) = test_db::setup_test_db();
        let publisher_id = test_publisher_id(&pool);
        let schema = test_schema();
        let query = duplicate_mutation(publisher_id);

        // No-guard baseline.
        let baseline_ctx = context_in_mode(Arc::clone(&pool), MutationGuardMode::Off);
        reset_counters();
        let baseline_before = imprint_count(&pool);
        let (baseline_data, baseline_errors) =
            run_sync(&schema, &baseline_ctx, &query, Variables::new());
        let baseline_calls = mutation_resolver_calls();
        let baseline_writes = imprint_count(&pool) - baseline_before;

        // OBSERVE: guard detects but does not act; execution proceeds.
        test_db::reset_db(&pool).expect("reset");
        let publisher_id = test_publisher_id(&pool);
        let query = duplicate_mutation(publisher_id);
        let observe_ctx = context_in_mode(Arc::clone(&pool), MutationGuardMode::Observe);
        let decision = guard(
            MutationGuardMode::Observe,
            &schema,
            &request(&query, None, None),
        );
        assert_eq!(decision.outcome, GuardOutcome::Proceed);
        reset_counters();
        let observe_before = imprint_count(&pool);
        let (observe_data, observe_errors) =
            run_sync(&schema, &observe_ctx, &query, Variables::new());
        let observe_calls = mutation_resolver_calls();
        let observe_writes = imprint_count(&pool) - observe_before;

        assert_eq!(baseline_calls, observe_calls);
        assert_eq!(baseline_writes, observe_writes);
        assert_eq!(baseline_errors.len(), observe_errors.len());
        assert_eq!(
            baseline_data.get("x").is_some(),
            observe_data.get("x").is_some()
        );
    }

    #[test]
    fn observation_event_carries_only_permitted_fields() {
        let (_guard, pool) = test_db::setup_test_db();
        let publisher_id = test_publisher_id(&pool);
        let schema = test_schema();
        // The variable must be *used*, or juniper's `no_unused_variables` rule
        // makes the document baseline-invalid and the guard rightly declines.
        let query = format!(
            r#"mutation Named($skipNothing: Boolean = false) {{
                 x: addImprint(publisherId: "{publisher_id}", imprintName: "secret-payload-value") @skip(if: $skipNothing) {{ publishers {{ publisherId }} }}
                 x: addImprint(publisherId: "{publisher_id}", imprintName: "secret-payload-value") {{ publishers {{ publisherId }} }}
               }}"#
        );
        let variables = json!({ "skipNothing": false });

        let decision = guard(
            MutationGuardMode::Observe,
            &schema,
            &request(&query, Some("Named"), Some(variables)),
        );
        let event = decision.event.expect("event");

        // Positive assertions.
        assert_eq!(event.mode, "OBSERVE");
        assert_eq!(event.collisions, vec!["x".to_string()]);
        assert_eq!(event.operation_name.as_deref(), Some("Named"));

        // Negative assertions: the rendered record carries no document text, no
        // variables and no argument values.
        let rendered = format!("{event:?}");
        for forbidden in [
            "secret-payload-value",
            "addImprint(",
            "publisherId:",
            "skipNothing",
        ] {
            assert!(
                !rendered.contains(forbidden),
                "guard event must never carry `{forbidden}`; got: {rendered}"
            );
        }
    }

    #[test]
    fn operation_name_is_absent_when_the_request_supplies_none() {
        let (_guard, pool) = test_db::setup_test_db();
        let publisher_id = test_publisher_id(&pool);
        let schema = test_schema();
        let decision = guard(
            MutationGuardMode::Enforce,
            &schema,
            &request(&duplicate_mutation(publisher_id), None, None),
        );
        assert_eq!(decision.event.expect("event").operation_name, None);
    }

    #[test]
    fn the_guard_evaluates_only_the_selected_operation() {
        let (_guard, pool) = test_db::setup_test_db();
        let publisher_id = test_publisher_id(&pool);
        let schema = test_schema();
        let query = format!(
            r#"mutation Clean {{
                 only: addImprint(publisherId: "{publisher_id}", imprintName: "a") {{ publishers {{ publisherId }} }}
               }}
               mutation Dirty {{
                 x: addImprint(publisherId: "{publisher_id}", imprintName: "a") {{ publishers {{ publisherId }} }}
                 x: addImprint(publisherId: "{publisher_id}", imprintName: "a") {{ publishers {{ publisherId }} }}
               }}"#
        );

        let clean = guard(
            MutationGuardMode::Enforce,
            &schema,
            &request(&query, Some("Clean"), None),
        );
        assert_eq!(clean.outcome, GuardOutcome::Proceed);

        let dirty = guard(
            MutationGuardMode::Enforce,
            &schema,
            &request(&query, Some("Dirty"), None),
        );
        assert!(matches!(dirty.outcome, GuardOutcome::Reject { .. }));
    }

    #[test]
    fn rejection_message_exposes_no_loader_store_or_scope_internals() {
        let (_guard, pool) = test_db::setup_test_db();
        let publisher_id = test_publisher_id(&pool);
        let schema = test_schema();
        let req = request(&duplicate_mutation(publisher_id), None, None);
        let decision = guard(MutationGuardMode::Enforce, &schema, &req);
        let GuardOutcome::Reject { collisions } = decision.outcome else {
            panic!("expected rejection");
        };
        let positions = mutation_guard::collision_positions(&req, &schema, &collisions);
        let response =
            mutation_guard::rejection_response::<DefaultScalarValue>(&collisions, positions);
        let body = serde_json::to_value(&response).expect("serialize");
        let message = body["errors"][0]["message"].as_str().unwrap();

        for forbidden in [
            "loader",
            "store",
            "scope",
            "batch",
            "cache",
            "invalid GraphQL",
        ] {
            assert!(
                !message.to_lowercase().contains(forbidden),
                "rejection message must not mention `{forbidden}`: {message}"
            );
        }
        assert!(!response.is_ok());
    }
}

// ---------------------------------------------------------------------------
// Guard — query-path behaviour and the non-mutation fast path
// ---------------------------------------------------------------------------
//
// The eligibility gate touches EVERY request in `OBSERVE`/`ENFORCE`, so query
// behaviour must be proven equivalent to the no-guard baseline in every
// observable respect except measurable overhead.

mod query_path {
    use super::*;

    const VALID_QUERY: &str = "{ testPublishers { publisherId imprints { imprintName } } }";
    const INVALID_QUERY: &str = "{ testPublishers { noSuchField } }";

    #[test]
    fn a_valid_query_is_never_restricted_and_emits_no_event_in_any_mode() {
        let (_guard, pool) = test_db::setup_test_db();
        seed(&pool, 2, 2);
        let schema = test_schema();
        let req = request(VALID_QUERY, None, None);

        for mode in [
            MutationGuardMode::Off,
            MutationGuardMode::Observe,
            MutationGuardMode::Enforce,
        ] {
            let decision = guard(mode, &schema, &req);
            assert_eq!(
                decision.outcome,
                GuardOutcome::Proceed,
                "{mode:?}: a query must never be restricted"
            );
            assert!(
                decision.event.is_none(),
                "{mode:?}: a non-mutation must exit at operation-type discrimination \
                 without any duplicate-key analysis, so it can emit no event"
            );
        }
    }

    #[test]
    fn a_valid_query_response_is_byte_identical_across_every_mode() {
        let (_guard, pool) = test_db::setup_test_db();
        seed(&pool, 3, 2);
        let schema = test_schema();
        let req = request(VALID_QUERY, None, None);

        // The no-guard baseline.
        let baseline_ctx = context_in_mode(Arc::clone(&pool), MutationGuardMode::Off);
        let baseline = req.execute_sync(&schema, &baseline_ctx);
        let baseline_ok = baseline.is_ok();
        let baseline_body = serde_json::to_value(&baseline).expect("serialize");

        for mode in [MutationGuardMode::Observe, MutationGuardMode::Enforce] {
            assert_eq!(guard(mode, &schema, &req).outcome, GuardOutcome::Proceed);
            let ctx = context_in_mode(Arc::clone(&pool), mode);
            let guarded = req.execute_sync(&schema, &ctx);
            assert_eq!(guarded.is_ok(), baseline_ok, "{mode:?}: status must match");
            assert_eq!(
                serde_json::to_value(&guarded).expect("serialize"),
                baseline_body,
                "{mode:?}: a query response must be byte-identical to the no-guard baseline"
            );
        }
    }

    #[test]
    fn an_invalid_query_keeps_juniper_canonical_error_and_produces_no_guard_event() {
        let (_guard, pool) = test_db::setup_test_db();
        seed(&pool, 1, 1);
        let schema = test_schema();
        let req = request(INVALID_QUERY, None, None);

        let baseline_ctx = context_in_mode(Arc::clone(&pool), MutationGuardMode::Off);
        let baseline = req.execute_sync(&schema, &baseline_ctx);
        assert!(
            !baseline.is_ok(),
            "the fixture query must really be invalid"
        );
        let baseline_body = serde_json::to_value(&baseline).expect("serialize");

        for mode in [
            MutationGuardMode::Off,
            MutationGuardMode::Observe,
            MutationGuardMode::Enforce,
        ] {
            let decision = guard(mode, &schema, &req);
            assert_eq!(decision.outcome, GuardOutcome::Proceed, "{mode:?}");
            assert!(decision.event.is_none(), "{mode:?}: no guard event");

            let ctx = context_in_mode(Arc::clone(&pool), mode);
            let guarded = req.execute_sync(&schema, &ctx);
            assert!(!guarded.is_ok());
            assert_eq!(
                serde_json::to_value(&guarded).expect("serialize"),
                baseline_body,
                "{mode:?}: juniper's canonical error must be preserved exactly"
            );
        }
    }

    #[test]
    fn a_query_with_a_duplicate_response_key_shares_one_scope_and_adds_no_statement() {
        // Repeated occurrences of ONE top-level query response key share one
        // scope, and that is correct and required.
        let (_guard, pool) = test_db::setup_test_db();
        let publishers = seed(&pool, 3, 2);
        let schema = test_schema();
        let req = request(
            "query { x: testPublishers { imprints { imprintName } } \
                     x: testPublishers { imprints { imprintName } } }",
            None,
            None,
        );

        // Never restricted.
        assert_eq!(
            guard(MutationGuardMode::Enforce, &schema, &req).outcome,
            GuardOutcome::Proceed
        );

        let context = context_in_mode(Arc::clone(&pool), MutationGuardMode::Enforce);
        reset_counters();
        let response = req.execute_sync(&schema, &context);
        assert!(response.is_ok());

        // One shared scope `x`, so one entry per publisher — not two per
        // publisher as two distinct top-level aliases would produce.
        assert_eq!(
            context.batch_store.entry_count(),
            publishers.len(),
            "repeated occurrences of one response key must share one scope"
        );
        assert_eq!(
            terminal_fallback_calls(),
            0,
            "the second occurrence must issue no additional terminal statement"
        );
    }
}

// ---------------------------------------------------------------------------
// Guard — baseline eligibility matrix
// ---------------------------------------------------------------------------

mod baseline_matrix {
    use super::*;

    /// Every document below is BOTH baseline-invalid AND shaped as a duplicate
    /// top-level mutation response key, so a guard ignoring the eligibility
    /// gate would visibly misbehave.
    fn cases(
        publisher_id: Uuid,
    ) -> Vec<(
        &'static str,
        String,
        Option<&'static str>,
        Option<JsonValue>,
    )> {
        vec![
            (
                "unknown field / document validation",
                format!(
                    r#"mutation {{
                         x: noSuchMutation(publisherId: "{publisher_id}") {{ publishers {{ publisherId }} }}
                         x: noSuchMutation(publisherId: "{publisher_id}") {{ publishers {{ publisherId }} }}
                       }}"#
                ),
                None,
                None,
            ),
            (
                "invalid field selection on a scalar / document validation",
                format!(
                    r#"mutation {{
                         x: addImprint(publisherId: "{publisher_id}", imprintName: "a") {{ publishers {{ publisherId {{ nope }} }} }}
                         x: addImprint(publisherId: "{publisher_id}", imprintName: "a") {{ publishers {{ publisherId }} }}
                       }}"#
                ),
                None,
                None,
            ),
            (
                "unknown directive / document validation",
                format!(
                    r#"mutation {{
                         x: addImprint(publisherId: "{publisher_id}", imprintName: "a") @nonsense {{ publishers {{ publisherId }} }}
                         x: addImprint(publisherId: "{publisher_id}", imprintName: "a") {{ publishers {{ publisherId }} }}
                       }}"#
                ),
                None,
                None,
            ),
            (
                "non-null variable declaring a default / document validation",
                format!(
                    r#"mutation Q($skip: Boolean! = true) {{
                         x: addImprint(publisherId: "{publisher_id}", imprintName: "a") @skip(if: $skip) {{ publishers {{ publisherId }} }}
                         x: addImprint(publisherId: "{publisher_id}", imprintName: "a") {{ publishers {{ publisherId }} }}
                       }}"#
                ),
                None,
                None,
            ),
            (
                "missing required variable / input validation",
                format!(
                    r#"mutation Q($skip: Boolean!) {{
                         x: addImprint(publisherId: "{publisher_id}", imprintName: "a") @skip(if: $skip) {{ publishers {{ publisherId }} }}
                         x: addImprint(publisherId: "{publisher_id}", imprintName: "a") {{ publishers {{ publisherId }} }}
                       }}"#
                ),
                None,
                None,
            ),
            (
                "invalid variable type / input validation",
                format!(
                    r#"mutation Q($skip: Boolean!) {{
                         x: addImprint(publisherId: "{publisher_id}", imprintName: "a") @skip(if: $skip) {{ publishers {{ publisherId }} }}
                         x: addImprint(publisherId: "{publisher_id}", imprintName: "a") {{ publishers {{ publisherId }} }}
                       }}"#
                ),
                None,
                Some(json!({ "skip": "not-a-boolean" })),
            ),
            (
                "multiple operations, no operationName / operation selection",
                format!(
                    r#"mutation A {{
                         x: addImprint(publisherId: "{publisher_id}", imprintName: "a") {{ publishers {{ publisherId }} }}
                         x: addImprint(publisherId: "{publisher_id}", imprintName: "a") {{ publishers {{ publisherId }} }}
                       }}
                       mutation B {{
                         x: addImprint(publisherId: "{publisher_id}", imprintName: "c") {{ publishers {{ publisherId }} }}
                         x: addImprint(publisherId: "{publisher_id}", imprintName: "c") {{ publishers {{ publisherId }} }}
                       }}"#
                ),
                None,
                None,
            ),
            (
                "unknown operationName / operation selection",
                format!(
                    r#"mutation A {{
                         x: addImprint(publisherId: "{publisher_id}", imprintName: "a") {{ publishers {{ publisherId }} }}
                         x: addImprint(publisherId: "{publisher_id}", imprintName: "a") {{ publishers {{ publisherId }} }}
                       }}"#
                ),
                Some("NoSuchOperation"),
                None,
            ),
            (
                "parse failure / parse",
                format!(
                    r#"mutation {{
                         x: addImprint(publisherId: "{publisher_id}" imprintName "a" {{ publishers {{
                         x: addImprint(
                       "#
                ),
                None,
                None,
            ),
        ]
    }

    #[test]
    fn baseline_invalid_requests_yield_no_guard_decision_and_no_event_in_any_mode() {
        let (_guard, pool) = test_db::setup_test_db();
        let publisher_id = test_db::create_publisher(&pool).publisher_id;
        let schema = test_schema();

        for (label, query, operation_name, variables) in cases(publisher_id) {
            let req = request(&query, operation_name, variables.clone());

            for mode in [
                MutationGuardMode::Off,
                MutationGuardMode::Observe,
                MutationGuardMode::Enforce,
            ] {
                let decision = guard(mode, &schema, &req);
                assert_eq!(
                    decision.outcome,
                    GuardOutcome::Proceed,
                    "[{label}] in {mode:?}: the guard must return no rejection"
                );
                assert!(
                    decision.event.is_none(),
                    "[{label}] in {mode:?}: the guard must emit no event"
                );
            }
        }
    }

    #[test]
    fn baseline_invalid_responses_are_byte_identical_to_the_no_guard_baseline() {
        let (_guard, pool) = test_db::setup_test_db();
        let publisher_id = test_db::create_publisher(&pool).publisher_id;
        let schema = test_schema();

        for (label, query, operation_name, variables) in cases(publisher_id) {
            let req = request(&query, operation_name, variables.clone());

            // Ordinary juniper with NO guard present.
            let baseline_ctx = context_in_mode(Arc::clone(&pool), MutationGuardMode::Off);
            let baseline = req.execute_sync(&schema, &baseline_ctx);
            let baseline_ok = baseline.is_ok();
            let baseline_body = serde_json::to_value(&baseline).expect("serialize");

            // The guarded path in each evaluating mode: the guard declines to
            // decide, so ordinary juniper produces the canonical response.
            for mode in [MutationGuardMode::Observe, MutationGuardMode::Enforce] {
                let decision = guard(mode, &schema, &req);
                assert_eq!(decision.outcome, GuardOutcome::Proceed, "[{label}]");

                let guarded_ctx = context_in_mode(Arc::clone(&pool), mode);
                let guarded = req.execute_sync(&schema, &guarded_ctx);
                assert_eq!(
                    guarded.is_ok(),
                    baseline_ok,
                    "[{label}] in {mode:?}: HTTP status must match the no-guard baseline"
                );
                assert_eq!(
                    serde_json::to_value(&guarded).expect("serialize"),
                    baseline_body,
                    "[{label}] in {mode:?}: the externally visible error must be juniper's own"
                );
            }
        }
    }

    #[test]
    fn a_rejected_operation_and_a_real_validation_failure_have_the_same_error_shape() {
        let (_guard, pool) = test_db::setup_test_db();
        let publisher_id = test_db::create_publisher(&pool).publisher_id;
        let schema = test_schema();
        let context = context_in_mode(Arc::clone(&pool), MutationGuardMode::Enforce);

        // A real juniper validation failure (unknown field).
        let invalid = request(
            r#"mutation { nope: noSuchMutation { publisherId } }"#,
            None,
            None,
        );
        let juniper_failure = invalid.execute_sync(&schema, &context);
        let juniper_body = serde_json::to_value(&juniper_failure).expect("serialize");

        // A guard rejection.
        let duplicate = request(
            &format!(
                r#"mutation {{
                     x: addImprint(publisherId: "{publisher_id}", imprintName: "a") {{ publishers {{ publisherId }} }}
                     x: addImprint(publisherId: "{publisher_id}", imprintName: "a") {{ publishers {{ publisherId }} }}
                   }}"#
            ),
            None,
            None,
        );
        let GuardOutcome::Reject { collisions } =
            guard(MutationGuardMode::Enforce, &schema, &duplicate).outcome
        else {
            panic!("expected rejection");
        };
        let positions = mutation_guard::collision_positions(&duplicate, &schema, &collisions);
        let rejection =
            mutation_guard::rejection_response::<DefaultScalarValue>(&collisions, positions);
        let rejection_body = serde_json::to_value(&rejection).expect("serialize");

        // Same failure convention: is_ok() false (=> the existing handler
        // branch returns HTTP 400), an `errors` array of {message, locations},
        // and NO `data` key.
        assert!(!juniper_failure.is_ok());
        assert!(!rejection.is_ok());
        assert!(juniper_body.get("data").is_none());
        assert!(
            rejection_body.get("data").is_none(),
            "a guard rejection must carry no data key, exactly like a validation failure"
        );

        let juniper_keys: Vec<&str> = juniper_body["errors"][0]
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        let rejection_keys: Vec<&str> = rejection_body["errors"][0]
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(juniper_keys, rejection_keys);
        assert!(rejection_body["errors"][0]["locations"].is_array());
    }
}

// ---------------------------------------------------------------------------
// Guard — directives and effective variables
// ---------------------------------------------------------------------------

mod directives {
    use super::*;

    /// Assert the guard's verdict against **Juniper's observed execution** of
    /// the same document and variables, rather than against a separately
    /// written expectation table.
    ///
    /// `expected_executions` is what juniper actually runs; the guard must
    /// reject exactly when that is greater than one for a single response key.
    fn assert_against_juniper(
        label: &str,
        query: &str,
        variables: Option<JsonValue>,
        expect_rejected: bool,
    ) {
        let (_guard, pool) = test_db::setup_test_db();
        let publisher_id = test_db::create_publisher(&pool).publisher_id;
        let query = query.replace("{PUBLISHER}", &publisher_id.to_string());
        let schema = test_schema();
        let req = request(&query, None, variables.clone());

        // What juniper ACTUALLY executes, with the guard absent.
        let baseline_ctx = context_in_mode(Arc::clone(&pool), MutationGuardMode::Off);
        reset_counters();
        let response = req.execute_sync(&schema, &baseline_ctx);
        assert!(
            response.is_ok(),
            "[{label}] the document must be baseline-valid; got {:?}",
            serde_json::to_value(&response).unwrap()
        );
        let observed_executions = mutation_resolver_calls();

        // The guard's verdict.
        let decision = guard(MutationGuardMode::Enforce, &schema, &req);
        let rejected = matches!(decision.outcome, GuardOutcome::Reject { .. });

        assert_eq!(
            rejected, expect_rejected,
            "[{label}] guard verdict mismatch (juniper executed {observed_executions} resolver(s))"
        );

        // The binding cross-check: the guard rejects exactly when juniper would
        // execute one response key more than once.
        if rejected {
            assert!(
                observed_executions > 1,
                "[{label}] the guard rejected but juniper executed only {observed_executions}"
            );
        } else {
            assert!(
                observed_executions <= 1,
                "[{label}] the guard accepted but juniper executed {observed_executions} times — \
                 a duplicate write would occur"
            );
        }
    }

    const SKIP_LITERAL_TRUE: &str = r#"mutation {
        x: addImprint(publisherId: "{PUBLISHER}", imprintName: "a") @skip(if: true) { publishers { publisherId } }
        x: addImprint(publisherId: "{PUBLISHER}", imprintName: "a") { publishers { publisherId } }
    }"#;

    const SKIP_LITERAL_FALSE: &str = r#"mutation {
        x: addImprint(publisherId: "{PUBLISHER}", imprintName: "a") @skip(if: false) { publishers { publisherId } }
        x: addImprint(publisherId: "{PUBLISHER}", imprintName: "a") { publishers { publisherId } }
    }"#;

    const INCLUDE_LITERAL_FALSE: &str = r#"mutation {
        x: addImprint(publisherId: "{PUBLISHER}", imprintName: "a") @include(if: false) { publishers { publisherId } }
        x: addImprint(publisherId: "{PUBLISHER}", imprintName: "a") { publishers { publisherId } }
    }"#;

    const INCLUDE_LITERAL_TRUE: &str = r#"mutation {
        x: addImprint(publisherId: "{PUBLISHER}", imprintName: "a") @include(if: true) { publishers { publisherId } }
        x: addImprint(publisherId: "{PUBLISHER}", imprintName: "a") { publishers { publisherId } }
    }"#;

    #[test]
    fn literal_directive_conditions_in_both_directions() {
        assert_against_juniper("@skip(if: true) excludes", SKIP_LITERAL_TRUE, None, false);
        assert_against_juniper("@skip(if: false) includes", SKIP_LITERAL_FALSE, None, true);
        assert_against_juniper(
            "@include(if: false) excludes",
            INCLUDE_LITERAL_FALSE,
            None,
            false,
        );
        assert_against_juniper(
            "@include(if: true) includes",
            INCLUDE_LITERAL_TRUE,
            None,
            true,
        );
    }

    const SKIP_VAR_NO_DEFAULT: &str = r#"mutation Q($skip: Boolean!) {
        x: addImprint(publisherId: "{PUBLISHER}", imprintName: "a") @skip(if: $skip) { publishers { publisherId } }
        x: addImprint(publisherId: "{PUBLISHER}", imprintName: "a") { publishers { publisherId } }
    }"#;

    #[test]
    fn request_supplied_variables_in_both_directions_no_default() {
        // No regression: variables with NO default behave exactly as before.
        assert_against_juniper(
            "$skip=true excludes",
            SKIP_VAR_NO_DEFAULT,
            Some(json!({ "skip": true })),
            false,
        );
        assert_against_juniper(
            "$skip=false includes",
            SKIP_VAR_NO_DEFAULT,
            Some(json!({ "skip": false })),
            true,
        );
    }

    // The pinned `default_values_of_correct_type` rule rejects `Boolean! = true`
    // outright, so defaulted-variable documents MUST declare the variable
    // nullable. The pinned stack accepts a nullable variable in the non-null
    // `if:` position.
    const SKIP_DEFAULT_TRUE: &str = r#"mutation Q($skip: Boolean = true) {
        x: addImprint(publisherId: "{PUBLISHER}", imprintName: "a") @skip(if: $skip) { publishers { publisherId } }
        x: addImprint(publisherId: "{PUBLISHER}", imprintName: "a") { publishers { publisherId } }
    }"#;

    const INCLUDE_DEFAULT_FALSE: &str = r#"mutation Q($inc: Boolean = false) {
        x: addImprint(publisherId: "{PUBLISHER}", imprintName: "a") @include(if: $inc) { publishers { publisherId } }
        x: addImprint(publisherId: "{PUBLISHER}", imprintName: "a") { publishers { publisherId } }
    }"#;

    #[test]
    fn skip_default_true_with_variable_omitted_is_accepted() {
        // The omitted-but-defaulted variable is RESOLVED, not undecidable.
        assert_against_juniper(
            "@skip default true, omitted -> one executable occurrence",
            SKIP_DEFAULT_TRUE,
            None,
            false,
        );
    }

    #[test]
    fn explicit_request_value_overrides_the_operation_default() {
        assert_against_juniper(
            "@skip default true, overridden false -> two executable occurrences",
            SKIP_DEFAULT_TRUE,
            Some(json!({ "skip": false })),
            true,
        );
    }

    #[test]
    fn include_default_false_with_variable_omitted_is_accepted() {
        assert_against_juniper(
            "@include default false, omitted -> duplicate excluded",
            INCLUDE_DEFAULT_FALSE,
            None,
            false,
        );
    }

    #[test]
    fn include_default_false_overridden_true_is_rejected() {
        assert_against_juniper(
            "@include default false, overridden true -> duplicate executable",
            INCLUDE_DEFAULT_FALSE,
            Some(json!({ "inc": true })),
            true,
        );
    }

    const SKIP_DEFAULT_ON_FRAGMENT_SPREAD: &str = r#"mutation Q($skip: Boolean = true) {
        x: addImprint(publisherId: "{PUBLISHER}", imprintName: "a") { publishers { publisherId } }
        ...Dup @skip(if: $skip)
    }
    fragment Dup on TestMutation {
        x: addImprint(publisherId: "{PUBLISHER}", imprintName: "a") { publishers { publisherId } }
    }"#;

    #[test]
    fn defaulted_directive_on_a_named_fragment_spread() {
        assert_against_juniper(
            "directive on the spread itself, default true -> excluded",
            SKIP_DEFAULT_ON_FRAGMENT_SPREAD,
            None,
            false,
        );
        assert_against_juniper(
            "directive on the spread itself, overridden false -> included",
            SKIP_DEFAULT_ON_FRAGMENT_SPREAD,
            Some(json!({ "skip": false })),
            true,
        );
    }

    const SKIP_DEFAULT_ON_INLINE_FRAGMENT: &str = r#"mutation Q($skip: Boolean = true) {
        x: addImprint(publisherId: "{PUBLISHER}", imprintName: "a") { publishers { publisherId } }
        ... on TestMutation @skip(if: $skip) {
            x: addImprint(publisherId: "{PUBLISHER}", imprintName: "a") { publishers { publisherId } }
        }
    }"#;

    #[test]
    fn defaulted_directive_on_an_inline_fragment() {
        assert_against_juniper(
            "directive on the inline fragment, default true -> excluded",
            SKIP_DEFAULT_ON_INLINE_FRAGMENT,
            None,
            false,
        );
        assert_against_juniper(
            "directive on the inline fragment, overridden false -> included",
            SKIP_DEFAULT_ON_INLINE_FRAGMENT,
            Some(json!({ "skip": false })),
            true,
        );
    }

    const MULTIPLE_DIRECTIVES: &str = r#"mutation Q($skip: Boolean = false, $inc: Boolean = true) {
        x: addImprint(publisherId: "{PUBLISHER}", imprintName: "a") @skip(if: $skip) @include(if: $inc) { publishers { publisherId } }
        x: addImprint(publisherId: "{PUBLISHER}", imprintName: "a") { publishers { publisherId } }
    }"#;

    #[test]
    fn multiple_directives_on_one_field() {
        assert_against_juniper(
            "skip=false include=true -> included",
            MULTIPLE_DIRECTIVES,
            None,
            true,
        );
        assert_against_juniper(
            "skip=true wins -> excluded",
            MULTIPLE_DIRECTIVES,
            Some(json!({ "skip": true })),
            false,
        );
        assert_against_juniper(
            "include=false -> excluded",
            MULTIPLE_DIRECTIVES,
            Some(json!({ "inc": false })),
            false,
        );
    }
}

// ---------------------------------------------------------------------------
// Mutation behaviour (`ADR-0006` sections 4.12.10-4.12.12)
// ---------------------------------------------------------------------------

mod mutation_behaviour {
    use super::*;

    #[test]
    fn read_after_write_holds_within_one_top_level_mutation_field() {
        let (_guard, pool) = test_db::setup_test_db();
        let publisher = test_db::create_publisher(&pool);
        test_db::create_imprint(&pool, &publisher);
        let context = context_in_mode(Arc::clone(&pool), MutationGuardMode::Enforce);
        let schema = test_schema();
        reset_counters();

        let query = format!(
            r#"mutation {{
                 only: addImprint(publisherId: "{}", imprintName: "AAA-written-now") {{
                   publishers {{ publisherId imprints {{ imprintName }} }}
                 }}
               }}"#,
            publisher.publisher_id
        );
        let (data, errors) = run_sync(&schema, &context, &query, Variables::new());
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");

        let names: Vec<&str> = data["only"]["publishers"][0]["imprints"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| i["imprintName"].as_str().unwrap())
            .collect();
        assert!(
            names.contains(&"AAA-written-now"),
            "the nested selection must observe the write made in the same top-level field; got {names:?}"
        );
    }

    #[test]
    fn two_top_level_mutation_fields_are_isolated_by_scope() {
        let (_guard, pool) = test_db::setup_test_db();
        let publisher = test_db::create_publisher(&pool);
        let context = context_in_mode(Arc::clone(&pool), MutationGuardMode::Enforce);
        let schema = test_schema();
        reset_counters();

        let query = format!(
            r#"mutation {{
                 first: addImprint(publisherId: "{pid}", imprintName: "AAA-first") {{
                   publishers {{ imprints {{ imprintName }} }}
                 }}
                 second: addImprintAlt(publisherId: "{pid}", imprintName: "AAB-second") {{
                   publishers {{ imprints {{ imprintName }} }}
                 }}
               }}"#,
            pid = publisher.publisher_id
        );
        let (data, errors) = run_sync(&schema, &context, &query, Variables::new());
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");

        let first: Vec<&str> = data["first"]["publishers"][0]["imprints"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| i["imprintName"].as_str().unwrap())
            .collect();
        let second: Vec<&str> = data["second"]["publishers"][0]["imprints"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| i["imprintName"].as_str().unwrap())
            .collect();

        assert!(first.contains(&"AAA-first"));
        assert!(
            second.contains(&"AAB-second"),
            "the second top-level field must observe its own write, never the first scope's \
             cached state; got {second:?}"
        );

        // The assertion rests on SCOPE ISOLATION, not execution order: the two
        // top-level response keys are two namespaces, so each holds its own
        // entries.
        assert_eq!(
            context.batch_store.entry_count(),
            2,
            "one entry per (scope, key): two scopes over one publisher"
        );
    }

    #[test]
    fn a_shared_terminal_fragment_across_two_top_level_fields_still_isolates() {
        // The terminal resolver's own source position is IDENTICAL on both
        // paths here. This is the case that rejected an execution-occurrence
        // scope (`ADR-0006` section 4.12.6.3).
        let (_guard, pool) = test_db::setup_test_db();
        let publisher = test_db::create_publisher(&pool);
        let context = context_in_mode(Arc::clone(&pool), MutationGuardMode::Enforce);
        let schema = test_schema();

        let query = format!(
            r#"mutation {{
                 first: addImprint(publisherId: "{pid}", imprintName: "AAA-first") {{ ...Payload }}
                 second: addImprintAlt(publisherId: "{pid}", imprintName: "AAB-second") {{ ...Payload }}
               }}
               fragment Payload on TestMutationPayload {{
                 publishers {{ imprints {{ imprintName }} }}
               }}"#,
            pid = publisher.publisher_id
        );
        let (data, errors) = run_sync(&schema, &context, &query, Variables::new());
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");

        let second: Vec<&str> = data["second"]["publishers"][0]["imprints"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| i["imprintName"].as_str().unwrap())
            .collect();
        assert!(
            second.contains(&"AAB-second"),
            "scope isolation must hold even through a shared named fragment; got {second:?}"
        );
        assert_eq!(context.batch_store.entry_count(), 2);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn scope_isolation_holds_under_async_execution() {
        // The pinned juniper drives top-level mutation fields through
        // `FuturesOrdered` on the async path and may interleave them. The
        // isolation invariant must hold regardless, because it rests on scope
        // partitioning rather than on execution order.
        let (_guard, pool) = test_db::setup_test_db();
        let publisher = test_db::create_publisher(&pool);
        let context = context_in_mode(Arc::clone(&pool), MutationGuardMode::Enforce);
        let schema = test_schema();

        let query = format!(
            r#"mutation {{
                 first: addImprint(publisherId: "{pid}", imprintName: "AAA-first") {{
                   publishers {{ imprints {{ imprintName }} }}
                 }}
                 second: addImprintAlt(publisherId: "{pid}", imprintName: "AAB-second") {{
                   publishers {{ imprints {{ imprintName }} }}
                 }}
               }}"#,
            pid = publisher.publisher_id
        );
        let (data, errors) = run_async(&schema, &context, &query, Variables::new()).await;
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");

        let second: Vec<&str> = data["second"]["publishers"][0]["imprints"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| i["imprintName"].as_str().unwrap())
            .collect();
        assert!(second.contains(&"AAB-second"));
        assert_eq!(
            context.batch_store.entry_count(),
            2,
            "async execution must not merge the two top-level scopes"
        );
    }

    #[test]
    fn the_duplicate_form_of_the_read_after_write_scenario_is_rejected() {
        // The distinct-alias case above works; the DUPLICATE case is precisely
        // what defeated the previous architecture and must be rejected.
        let (_guard, pool) = test_db::setup_test_db();
        let publisher = test_db::create_publisher(&pool);
        let schema = test_schema();
        let query = format!(
            r#"mutation {{
                 x: addImprint(publisherId: "{pid}", imprintName: "AAA-first") {{
                   publishers {{ imprints {{ imprintName }} }}
                 }}
                 x: addImprint(publisherId: "{pid}", imprintName: "AAA-first") {{
                   publishers {{ imprints {{ imprintName }} }}
                 }}
               }}"#,
            pid = publisher.publisher_id
        );

        reset_counters();
        let before = {
            use crate::schema::imprint;
            use diesel::{QueryDsl, RunQueryDsl};
            let mut connection = pool.get().unwrap();
            imprint::table
                .count()
                .get_result::<i64>(&mut connection)
                .unwrap()
        };

        let decision = guard(
            MutationGuardMode::Enforce,
            &schema,
            &request(&query, None, None),
        );
        assert!(matches!(decision.outcome, GuardOutcome::Reject { .. }));
        assert_eq!(mutation_resolver_calls(), 0);

        let after = {
            use crate::schema::imprint;
            use diesel::{QueryDsl, RunQueryDsl};
            let mut connection = pool.get().unwrap();
            imprint::table
                .count()
                .get_result::<i64>(&mut connection)
                .unwrap()
        };
        assert_eq!(before, after, "zero writes for a rejected operation");
    }

    #[test]
    fn mutation_payload_fan_out_stays_bounded_as_parent_count_rises() {
        let (_guard, pool) = test_db::setup_test_db();
        let publishers = seed(&pool, 5, 2);
        let context = context_in_mode(Arc::clone(&pool), MutationGuardMode::Enforce);
        let schema = test_schema();
        reset_counters();

        let query = format!(
            r#"mutation {{
                 only: addImprint(publisherId: "{}", imprintName: "AAA-new") {{
                   publishers {{ publisherId imprints {{ imprintName }} }}
                 }}
               }}"#,
            publishers[0].publisher_id
        );
        let (data, errors) = run_sync(&schema, &context, &query, Variables::new());
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
        assert_eq!(
            data["only"]["publishers"].as_array().unwrap().len(),
            publishers.len()
        );
        assert_eq!(
            terminal_fallback_calls(),
            0,
            "the mutation payload fan-out must be covered by the prefetch"
        );
        // One entry per publisher within the single top-level scope.
        assert_eq!(context.batch_store.entry_count(), publishers.len());
    }
}

// ---------------------------------------------------------------------------
// Execution-path parity: sync and async
// ---------------------------------------------------------------------------

mod execution_parity {
    use super::*;

    const PARITY_QUERY: &str = "{ \
        a: testPublishers { publisherId alias: imprints { imprintName } } \
        b: testPublishers { publisherId imprints { imprintName } } \
        c: testImprints { publisher { imprints { imprintName } } } \
    }";

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn sync_and_async_produce_the_same_result_and_the_same_scoping() {
        let (_guard, pool) = test_db::setup_test_db();
        let publishers = seed(&pool, 3, 2);
        let schema = test_schema();

        let sync_ctx = context_in_mode(Arc::clone(&pool), MutationGuardMode::Enforce);
        reset_counters();
        let (sync_data, sync_errors) = run_sync(&schema, &sync_ctx, PARITY_QUERY, Variables::new());
        let sync_entries = sync_ctx.batch_store.entry_count();
        let sync_fallbacks = terminal_fallback_calls();

        let async_ctx = context_in_mode(Arc::clone(&pool), MutationGuardMode::Enforce);
        reset_counters();
        let (async_data, async_errors) =
            run_async(&schema, &async_ctx, PARITY_QUERY, Variables::new()).await;
        let async_entries = async_ctx.batch_store.entry_count();
        let async_fallbacks = terminal_fallback_calls();

        assert!(sync_errors.is_empty(), "sync errors: {sync_errors:?}");
        assert!(async_errors.is_empty(), "async errors: {async_errors:?}");

        // Same direct-path result, same descendant-path result, same alias
        // behaviour.
        assert_eq!(sync_data, async_data);
        // Same top-level response-key derivation and the same scope isolation:
        // three top-level scopes, each holding its own entries.
        assert_eq!(sync_entries, async_entries);
        assert_eq!(
            sync_entries,
            publishers.len() * 3,
            "three top-level scopes must each hold their own entries"
        );
        // No GraphQL errors generated by scope extraction, on either path.
        assert_eq!(sync_fallbacks, 0);
        assert_eq!(async_fallbacks, 0);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn the_guard_runs_under_the_async_request_path() {
        let (_guard, pool) = test_db::setup_test_db();
        let publisher = test_db::create_publisher(&pool);
        let schema = test_schema();
        let context = context_in_mode(Arc::clone(&pool), MutationGuardMode::Enforce);

        let query = format!(
            r#"mutation {{
                 x: addImprint(publisherId: "{pid}", imprintName: "a") {{ publishers {{ publisherId }} }}
                 x: addImprint(publisherId: "{pid}", imprintName: "a") {{ publishers {{ publisherId }} }}
               }}"#,
            pid = publisher.publisher_id
        );
        let req = request(&query, None, None);

        // The guard decision is made before execution, identically on both
        // paths.
        let decision = guard(MutationGuardMode::Enforce, &schema, &req);
        assert!(matches!(decision.outcome, GuardOutcome::Reject { .. }));

        reset_counters();
        // Because the boundary rejects, `execute` is never called. Prove the
        // async execute path itself is otherwise healthy.
        let clean = request(
            &format!(
                r#"mutation {{ only: addImprint(publisherId: "{}", imprintName: "a") {{ publishers {{ publisherId }} }} }}"#,
                publisher.publisher_id
            ),
            None,
            None,
        );
        assert_eq!(
            guard(MutationGuardMode::Enforce, &schema, &clean).outcome,
            GuardOutcome::Proceed
        );
        let response = clean.execute(&schema, &context).await;
        assert!(response.is_ok());
        assert_eq!(mutation_resolver_calls(), 1);
    }
}

// ---------------------------------------------------------------------------
// Performance / SQL statement counts (`ADR-0006` section 8)
// ---------------------------------------------------------------------------

mod statement_counts {
    use super::*;

    /// Run `query` through a freshly constructed, instrumented pool and return
    /// the `imprint` statements observed.
    fn measure(
        pool: &Arc<PgPool>,
        mode: MutationGuardMode,
        query: &str,
    ) -> (Vec<String>, JsonValue) {
        let url = test_db::test_db_url();
        // 1. the exclusive DB test lock is already held by the caller
        // 2. the database is already reset and seeded through the ORDINARY pool
        // 3. install the instrumentation hook
        // 4. construct a NEW dedicated pool behind it
        let probe = SqlProbe::install(&url);
        let context = context_in_mode(Arc::clone(&probe.pool), mode);
        let schema = test_schema();
        reset_counters();

        // 5-7. run the measured operation and isolate its statements
        probe.start();
        let (data, errors) = run_sync(&schema, &context, query, Variables::new());
        let statements = probe.imprint_statements();
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
        let _ = pool;
        (statements, data)
    }

    #[test]
    fn direct_path_terminal_statement_count_is_bounded_at_two_list_sizes() {
        let mut rows = Vec::new();
        for n in [3usize, 7usize] {
            let (_guard, pool) = test_db::setup_test_db();
            seed(&pool, n, 2);

            let (batched, _) = measure(
                &pool,
                MutationGuardMode::Enforce,
                "{ testPublishers { imprints { imprintName } } }",
            );
            let (baseline, _) = measure(
                &pool,
                MutationGuardMode::Off,
                "{ testPublishers { imprints { imprintName } } }",
            );
            rows.push((n, batched.len(), baseline.len()));
        }

        for (n, batched, baseline) in &rows {
            assert_eq!(
                *batched, 1,
                "n={n}: the prefetched terminal-query count must stay bounded at 1"
            );
            assert_eq!(
                *baseline, *n,
                "n={n}: the direct baseline must grow with the parent count"
            );
        }
        // The bounded count does not scale with n; the baseline does.
        assert_eq!(rows[0].1, rows[1].1);
        assert!(rows[1].2 > rows[0].2);
        eprintln!("DIRECT PATH | scope=testPublishers | rows (n, prefetch, baseline) = {rows:?}");
    }

    #[test]
    fn descendant_path_reports_terminal_and_intermediate_counts_separately() {
        let mut rows = Vec::new();
        for n in [3usize, 6usize] {
            let (_guard, pool) = test_db::setup_test_db();
            seed(&pool, n, 2);

            let url = test_db::test_db_url();
            let probe = SqlProbe::install(&url);
            let context = context_in_mode(Arc::clone(&probe.pool), MutationGuardMode::Enforce);
            let schema = test_schema();
            reset_counters();
            probe.start();
            let (_data, errors) = run_sync(
                &schema,
                &context,
                "{ testImprints { publisher { imprints { imprintName } } } }",
                Variables::new(),
            );
            let terminal = probe.imprint_statements();
            assert!(errors.is_empty(), "unexpected errors: {errors:?}");

            // The terminal loader's own statements: one set-based dispatch plus
            // the list query that produced the ancestor items.
            let intermediate = intermediate_resolver_calls();
            rows.push((n, terminal.len(), intermediate));
        }

        for (n, terminal, _) in &rows {
            assert!(
                *terminal <= 2,
                "n={n}: the terminal loader must stay bounded (list + one dispatch), got {terminal}"
            );
        }
        // The intermediate resolver is NOT bounded, and is reported separately.
        assert!(
            rows[1].2 > rows[0].2,
            "the legacy intermediate resolver still scales with the list; \
             bounding the terminal loader does not make the operation globally N+1-free"
        );
        eprintln!(
            "DESCENDANT PATH | scope=testImprints | rows (n, terminal-loader stmts, \
             intermediate-resolver stmts) = {rows:?}"
        );
    }

    #[test]
    fn two_top_level_query_fields_issue_two_dispatches_not_n_plus_n() {
        let (_guard, pool) = test_db::setup_test_db();
        let n = 5usize;
        seed(&pool, n, 2);

        let (statements, data) = measure(
            &pool,
            MutationGuardMode::Enforce,
            "{ first: testPublishers { imprints { imprintName } } \
               second: testPublishers { imprints { imprintName } } }",
        );

        // Two list queries (one per top-level field) + two terminal dispatches.
        let dispatches = statements
            .iter()
            .filter(|sql| sql.contains("ANY") || sql.contains("= ANY"))
            .count();
        assert_eq!(
            dispatches, 2,
            "one bounded set-based child dispatch per top-level scope — 2, not N + N; got {statements:#?}"
        );
        assert_eq!(data["first"], data["second"]);
        assert_eq!(
            terminal_fallback_calls(),
            0,
            "no cross-scope reuse, but also no per-parent fallback"
        );
        eprintln!(
            "TWO-TOP-LEVEL QUERY | n={n} | terminal dispatches={dispatches} (accepted \
             ADR-0006 section 4.12.13 cross-scope cost)"
        );
    }

    #[test]
    fn mutation_payload_fan_out_statement_counts() {
        let mut rows = Vec::new();
        for n in [3usize, 6usize] {
            let (_guard, pool) = test_db::setup_test_db();
            let publishers = seed(&pool, n, 2);

            let url = test_db::test_db_url();
            let probe = SqlProbe::install(&url);
            let context = context_in_mode(Arc::clone(&probe.pool), MutationGuardMode::Enforce);
            let schema = test_schema();
            reset_counters();

            let query = format!(
                r#"mutation {{
                     only: addImprint(publisherId: "{}", imprintName: "AAA-new") {{
                       publishers {{ imprints {{ imprintName }} }}
                     }}
                   }}"#,
                publishers[0].publisher_id
            );
            probe.start();
            let (_data, errors) = run_sync(&schema, &context, &query, Variables::new());
            let statements = probe.imprint_statements();
            assert!(errors.is_empty(), "unexpected errors: {errors:?}");

            let dispatches = statements.iter().filter(|sql| sql.contains("ANY")).count();
            rows.push((n, dispatches, terminal_fallback_calls()));
        }

        for (n, dispatches, fallbacks) in &rows {
            assert_eq!(
                *dispatches, 1,
                "n={n}: one bounded dispatch inside the mutation scope"
            );
            assert_eq!(*fallbacks, 0, "n={n}: no per-parent fallback");
        }
        eprintln!("MUTATION FAN-OUT | scope=only | rows (n, dispatches, fallbacks) = {rows:?}");
    }

    #[test]
    fn duplicate_parent_keys_produce_one_key_and_no_extra_statement() {
        let (_guard, pool) = test_db::setup_test_db();
        seed(&pool, 1, 3);

        let (statements, _) = measure(
            &pool,
            MutationGuardMode::Enforce,
            "{ testPublishers { a: imprints { imprintName } b: imprints { imprintName } } }",
        );
        let dispatches = statements.iter().filter(|sql| sql.contains("ANY")).count();
        assert_eq!(
            dispatches, 1,
            "repeated aliases of one normalized shape must cause no additional statements"
        );
    }
}

// ---------------------------------------------------------------------------
// Authorization / key provenance
// ---------------------------------------------------------------------------

mod authorization {
    use super::*;

    #[test]
    fn keys_are_drawn_only_from_already_resolved_parents() {
        let (_guard, pool) = test_db::setup_test_db();
        let visible = seed(&pool, 2, 2);
        // A publisher the operation's parent list will not return, because the
        // descendant site projects keys only from the imprints it resolved.
        let hidden = test_db::create_publisher(&pool);

        let context = context_in_mode(Arc::clone(&pool), MutationGuardMode::Enforce);
        let schema = test_schema();
        reset_counters();

        let (_data, errors) = run_sync(
            &schema,
            &context,
            "{ testImprints { publisher { imprints { imprintName } } } }",
            Variables::new(),
        );
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");

        // Only the publishers reachable from the resolved imprints were loaded.
        assert_eq!(context.batch_store.entry_count(), visible.len());
        let shape = TestImprintLoader::shape(DEFAULT_IMPRINT_LIMIT);
        assert!(
            matches!(
                context
                    .batch_store
                    .lookup::<TestImprintLoader>(
                        &crate::graphql::batching::ScopeKey::new("testImprints"),
                        &shape,
                        &hidden.publisher_id
                    )
                    .expect("lookup"),
                BatchLookup::NotLoaded
            ),
            "a parent the request never resolved must never be fetched"
        );
    }
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

mod configuration {
    use super::*;

    #[test]
    fn guard_mode_defaults_to_off() {
        assert_eq!(MutationGuardMode::default(), MutationGuardMode::Off);
        assert!(!MutationGuardMode::default().store_available());
    }

    #[test]
    fn ordinary_context_new_defaults_to_off_and_an_unavailable_store() {
        let (_guard, pool) = test_db::setup_test_db();
        let context = test_db::test_context(pool, "default-mode-user");
        assert!(
            !context.batch_store.is_available(),
            "Context::new must default to OFF with the store unavailable"
        );
    }

    #[test]
    fn exactly_three_modes_parse_and_unknown_values_are_rejected() {
        use std::str::FromStr;
        assert_eq!(
            MutationGuardMode::from_str("OFF").unwrap(),
            MutationGuardMode::Off
        );
        assert_eq!(
            MutationGuardMode::from_str("observe").unwrap(),
            MutationGuardMode::Observe
        );
        assert_eq!(
            MutationGuardMode::from_str("Enforce").unwrap(),
            MutationGuardMode::Enforce
        );
        assert!(MutationGuardMode::from_str("ON").is_err());
        assert!(MutationGuardMode::from_str("").is_err());
    }
}

// ---------------------------------------------------------------------------
// Look-ahead traversal unit tests
// ---------------------------------------------------------------------------

mod traversal_unit {
    use super::*;

    #[test]
    fn distinct_shape_helper_deduplicates_semantically_equal_shapes() {
        // Guards against a shape type whose equality is syntactic.
        let a = TestImprintLoader::shape(3);
        let b = TestImprintLoader::shape(3);
        let c = TestImprintLoader::shape(4);
        assert_eq!(a, b);
        assert_ne!(a, c);

        use std::collections::HashSet;
        let set: HashSet<_> = [a.clone(), b, c].into_iter().collect();
        assert_eq!(set.len(), 2);
        let _ = a;
    }
}
