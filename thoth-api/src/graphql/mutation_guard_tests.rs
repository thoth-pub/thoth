//! A2-independent regression coverage for the central mutation request guard.
//!
//! These tests deliberately use a minimal in-memory schema and context. They
//! prove the mutation-execution concern without depending on the retired
//! GraphqlBatchStore, prefetch machinery, response scopes, or database fixture.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use juniper::{graphql_object, DefaultScalarValue, EmptySubscription, RootNode};
use serde_json::{json, Value as JsonValue};

use super::mutation_guard::{self, GuardOutcome, MutationGuardMode};
use super::GraphQLRequest;

#[derive(Clone, Default)]
struct GuardContext {
    writes: Arc<AtomicUsize>,
}

impl juniper::Context for GuardContext {}

impl GuardContext {
    fn reset(&self) {
        self.writes.store(0, Ordering::SeqCst);
    }

    fn writes(&self) -> usize {
        self.writes.load(Ordering::SeqCst)
    }
}

struct GuardQuery;
struct GuardMutation;
struct GuardPayload { value: i32 }
struct GuardNested;

#[graphql_object(Context = GuardContext, Scalar = DefaultScalarValue, name = "GuardQuery")]
impl GuardQuery {
    fn value() -> i32 { 7 }
    fn nested() -> GuardNested { GuardNested }
}

#[graphql_object(Context = GuardContext, Scalar = DefaultScalarValue, name = "GuardNested")]
impl GuardNested { fn value() -> i32 { 11 } }

#[graphql_object(Context = GuardContext, Scalar = DefaultScalarValue, name = "GuardMutation")]
impl GuardMutation {
    fn counted_write(context: &GuardContext, value: i32) -> GuardPayload {
        context.writes.fetch_add(1, Ordering::SeqCst);
        GuardPayload { value }
    }
    fn other_write(context: &GuardContext, value: i32) -> GuardPayload {
        context.writes.fetch_add(1, Ordering::SeqCst);
        GuardPayload { value }
    }
}

#[graphql_object(Context = GuardContext, Scalar = DefaultScalarValue, name = "GuardPayload")]
impl GuardPayload {
    fn value(&self) -> i32 { self.value }
    fn nested(&self) -> GuardNested { GuardNested }
}

type GuardSchema = RootNode<'static, GuardQuery, GuardMutation, EmptySubscription<GuardContext>>;
fn schema() -> GuardSchema { GuardSchema::new(GuardQuery, GuardMutation, EmptySubscription::new()) }

fn request(query: &str, operation_name: Option<&str>, variables: Option<JsonValue>) -> GraphQLRequest {
    let mut body = json!({ "query": query });
    if let Some(name) = operation_name { body["operationName"] = json!(name); }
    if let Some(vars) = variables { body["variables"] = vars; }
    serde_json::from_value(body).expect("build GraphQL request")
}

fn guard(mode: MutationGuardMode, schema: &GuardSchema, request: &GraphQLRequest) -> mutation_guard::GuardDecision {
    mutation_guard::evaluate(mode, request, schema)
}

async fn execute_json(schema: &GuardSchema, context: &GuardContext, request: &GraphQLRequest) -> JsonValue {
    serde_json::to_value(request.execute(schema, context).await).expect("serialize response")
}

fn duplicate_mutation() -> &'static str {
    "mutation { x: countedWrite(value: 1) { value } x: countedWrite(value: 1) { value } }"
}

mod guard_tests {
    use super::*;

    #[test]
    fn off_never_rejects_or_emits() {
        let schema = schema(); let req = request(duplicate_mutation(), None, None);
        let decision = guard(MutationGuardMode::Off, &schema, &req);
        assert_eq!(decision.outcome, GuardOutcome::Proceed); assert!(decision.event.is_none());
    }

    #[test]
    fn observe_records_but_never_rejects() {
        let schema = schema(); let req = request(duplicate_mutation(), None, None);
        let decision = guard(MutationGuardMode::Observe, &schema, &req);
        assert_eq!(decision.outcome, GuardOutcome::Proceed);
        let event = decision.event.expect("observe event");
        assert_eq!(event.mode, "OBSERVE"); assert_eq!(event.collisions, vec!["x".to_string()]);
    }

    #[test]
    fn enforce_rejects_direct_duplicate_top_level_response_key() {
        let schema = schema(); let req = request(duplicate_mutation(), None, None);
        let decision = guard(MutationGuardMode::Enforce, &schema, &req);
        assert_eq!(decision.outcome, GuardOutcome::Reject { collisions: vec!["x".to_string()] });
        assert_eq!(decision.event.expect("enforce event").mode, "ENFORCE");
    }

    #[test]
    fn enforce_rejects_named_fragment_duplicate() {
        let schema = schema();
        let req = request("mutation { ...Top } fragment Top on GuardMutation { x: countedWrite(value: 1) { value } x: countedWrite(value: 1) { value } }", None, None);
        assert!(matches!(guard(MutationGuardMode::Enforce, &schema, &req).outcome, GuardOutcome::Reject { .. }));
    }

    #[test]
    fn enforce_rejects_inline_fragment_duplicate() {
        let schema = schema();
        let req = request("mutation { ... on GuardMutation { x: countedWrite(value: 1) { value } x: countedWrite(value: 1) { value } } }", None, None);
        assert!(matches!(guard(MutationGuardMode::Enforce, &schema, &req).outcome, GuardOutcome::Reject { .. }));
    }

    #[test]
    fn nested_duplicate_response_key_is_not_a_top_level_collision() {
        let schema = schema();
        let req = request("mutation { x: countedWrite(value: 1) { y: nested { value } y: nested { value } } }", None, None);
        let decision = guard(MutationGuardMode::Enforce, &schema, &req);
        assert_eq!(decision.outcome, GuardOutcome::Proceed); assert!(decision.event.is_none());
    }

    #[test]
    fn distinct_top_level_aliases_are_allowed() {
        let schema = schema();
        let req = request("mutation { a: countedWrite(value: 1) { value } b: countedWrite(value: 1) { value } }", None, None);
        assert_eq!(guard(MutationGuardMode::Enforce, &schema, &req).outcome, GuardOutcome::Proceed);
    }

    #[test]
    fn operation_selection_only_evaluates_selected_operation() {
        let schema = schema();
        let query = "mutation Clean { a: countedWrite(value: 1) { value } } mutation Dirty { x: countedWrite(value: 1) { value } x: countedWrite(value: 1) { value } }";
        let clean = request(query, Some("Clean"), None); let dirty = request(query, Some("Dirty"), None);
        assert_eq!(guard(MutationGuardMode::Enforce, &schema, &clean).outcome, GuardOutcome::Proceed);
        assert!(matches!(guard(MutationGuardMode::Enforce, &schema, &dirty).outcome, GuardOutcome::Reject { .. }));
    }

    #[test]
    fn rejection_positions_cover_every_colliding_occurrence() {
        let schema = schema(); let req = request(duplicate_mutation(), None, None);
        let decision = guard(MutationGuardMode::Enforce, &schema, &req);
        let GuardOutcome::Reject { collisions } = decision.outcome else { panic!("expected rejection"); };
        let positions = mutation_guard::collision_positions(&req, &schema, &collisions);
        assert_eq!(positions.len(), 2);
        let response = mutation_guard::rejection_response::<DefaultScalarValue>(&collisions, positions);
        assert!(!response.is_ok());
        let body = serde_json::to_value(response).expect("serialize rejection");
        assert_eq!(body["errors"][0]["locations"].as_array().expect("locations").len(), 2);
    }

    #[test]
    fn event_contains_shape_metadata_but_no_document_variables_or_arguments() {
        let schema = schema();
        let req = request("mutation Named($v: Int! = 987654) { x: countedWrite(value: $v) { value } x: countedWrite(value: $v) { value } }", Some("Named"), None);
        let event = guard(MutationGuardMode::Observe, &schema, &req).event.expect("event");
        assert_eq!(event.operation_name.as_deref(), Some("Named")); assert_eq!(event.collisions, vec!["x".to_string()]);
        let rendered = format!("{event:?}");
        for forbidden in ["987654", "countedWrite", "$v", "value:"] { assert!(!rendered.contains(forbidden)); }
    }

    #[test]
    fn rejection_message_exposes_no_loader_store_or_scope_internals() {
        let schema = schema(); let req = request(duplicate_mutation(), None, None);
        let decision = guard(MutationGuardMode::Enforce, &schema, &req);
        let GuardOutcome::Reject { collisions } = decision.outcome else { panic!("expected rejection"); };
        let positions = mutation_guard::collision_positions(&req, &schema, &collisions);
        let response = mutation_guard::rejection_response::<DefaultScalarValue>(&collisions, positions);
        let body = serde_json::to_value(response).expect("serialize rejection");
        let message = body["errors"][0]["message"].as_str().expect("message");
        for forbidden in ["loader", "store", "scope", "batch", "cache"] { assert!(!message.to_ascii_lowercase().contains(forbidden)); }
    }
}

mod query_path {
    use super::*;
    const VALID_QUERY: &str = "{ value nested { value } }";
    const INVALID_QUERY: &str = "{ noSuchField }";

    #[test]
    fn a_valid_query_is_never_restricted_and_emits_no_event_in_any_mode() {
        let schema = schema(); let req = request(VALID_QUERY, None, None);
        for mode in [MutationGuardMode::Off, MutationGuardMode::Observe, MutationGuardMode::Enforce] {
            let decision = guard(mode, &schema, &req); assert_eq!(decision.outcome, GuardOutcome::Proceed); assert!(decision.event.is_none());
        }
    }

    #[tokio::test]
    async fn a_valid_query_response_is_byte_identical_across_every_mode() {
        let schema = schema(); let context = GuardContext::default(); let req = request(VALID_QUERY, None, None);
        let baseline = execute_json(&schema, &context, &req).await;
        for mode in [MutationGuardMode::Off, MutationGuardMode::Observe, MutationGuardMode::Enforce] {
            assert_eq!(guard(mode, &schema, &req).outcome, GuardOutcome::Proceed);
            assert_eq!(execute_json(&schema, &context, &req).await, baseline);
        }
    }

    #[tokio::test]
    async fn an_invalid_query_keeps_juniper_canonical_error_and_produces_no_guard_event() {
        let schema = schema(); let context = GuardContext::default(); let req = request(INVALID_QUERY, None, None);
        let baseline = execute_json(&schema, &context, &req).await; assert!(baseline["errors"].is_array());
        for mode in [MutationGuardMode::Off, MutationGuardMode::Observe, MutationGuardMode::Enforce] {
            let decision = guard(mode, &schema, &req); assert_eq!(decision.outcome, GuardOutcome::Proceed); assert!(decision.event.is_none());
            assert_eq!(execute_json(&schema, &context, &req).await, baseline);
        }
    }
}

mod baseline_matrix {
    use super::*;
    fn cases() -> Vec<(&'static str, &'static str, Option<&'static str>, Option<JsonValue>)> {
        vec![
            ("unknown mutation field", "mutation { x: noSuchMutation(value: 1) { value } x: noSuchMutation(value: 1) { value } }", None, None),
            ("unknown directive", "mutation { x: countedWrite(value: 1) @nonsense { value } x: countedWrite(value: 1) { value } }", None, None),
            ("missing required variable", "mutation Q($v: Int!) { x: countedWrite(value: $v) { value } x: countedWrite(value: $v) { value } }", Some("Q"), None),
            ("invalid variable type", "mutation Q($v: Int!) { x: countedWrite(value: $v) { value } x: countedWrite(value: $v) { value } }", Some("Q"), Some(json!({"v": "not-an-int"}))),
            ("multiple operations without operation name", "mutation A { x: countedWrite(value: 1) { value } x: countedWrite(value: 1) { value } } mutation B { x: countedWrite(value: 2) { value } x: countedWrite(value: 2) { value } }", None, None),
            ("unknown operation name", "mutation A { x: countedWrite(value: 1) { value } x: countedWrite(value: 1) { value } }", Some("Missing"), None),
            ("parse failure", "mutation { x: countedWrite(value 1 { value } x: countedWrite(", None, None),
        ]
    }

    #[tokio::test]
    async fn baseline_invalid_requests_keep_canonical_juniper_response_and_no_guard_event() {
        let schema = schema(); let context = GuardContext::default();
        for (label, query, operation_name, variables) in cases() {
            let req = request(query, operation_name, variables); let baseline = execute_json(&schema, &context, &req).await;
            assert!(baseline["errors"].is_array(), "[{label}] fixture must be invalid");
            for mode in [MutationGuardMode::Off, MutationGuardMode::Observe, MutationGuardMode::Enforce] {
                let decision = guard(mode, &schema, &req); assert_eq!(decision.outcome, GuardOutcome::Proceed); assert!(decision.event.is_none());
                assert_eq!(execute_json(&schema, &context, &req).await, baseline);
            }
        }
    }
}

mod directives {
    use super::*;
    struct Case { label: &'static str, query: &'static str, operation_name: Option<&'static str>, variables: Option<JsonValue> }
    fn cases() -> Vec<Case> {
        vec![
            Case { label: "plain duplicate", query: "mutation { x: countedWrite(value: 1) { value } x: countedWrite(value: 1) { value } }", operation_name: None, variables: None },
            Case { label: "literal skip excludes one", query: "mutation { x: countedWrite(value: 1) @skip(if: true) { value } x: countedWrite(value: 1) { value } }", operation_name: None, variables: None },
            Case { label: "literal include excludes one", query: "mutation { x: countedWrite(value: 1) @include(if: false) { value } x: countedWrite(value: 1) { value } }", operation_name: None, variables: None },
            Case { label: "variable false keeps both", query: "mutation Q($skip: Boolean!) { x: countedWrite(value: 1) @skip(if: $skip) { value } x: countedWrite(value: 1) { value } }", operation_name: Some("Q"), variables: Some(json!({"skip": false})) },
            Case { label: "variable true excludes one", query: "mutation Q($skip: Boolean!) { x: countedWrite(value: 1) @skip(if: $skip) { value } x: countedWrite(value: 1) { value } }", operation_name: Some("Q"), variables: Some(json!({"skip": true})) },
            Case { label: "defaulted false keeps both", query: "mutation Q($skip: Boolean = false) { x: countedWrite(value: 1) @skip(if: $skip) { value } x: countedWrite(value: 1) { value } }", operation_name: Some("Q"), variables: None },
            Case { label: "defaulted true excludes one", query: "mutation Q($skip: Boolean = true) { x: countedWrite(value: 1) @skip(if: $skip) { value } x: countedWrite(value: 1) { value } }", operation_name: Some("Q"), variables: None },
            Case { label: "directive on named fragment spread", query: "mutation Q($include: Boolean!) { x: countedWrite(value: 1) { value } ...Dup @include(if: $include) } fragment Dup on GuardMutation { x: countedWrite(value: 1) { value } }", operation_name: Some("Q"), variables: Some(json!({"include": true})) },
            Case { label: "directive excludes named fragment spread", query: "mutation Q($include: Boolean!) { x: countedWrite(value: 1) { value } ...Dup @include(if: $include) } fragment Dup on GuardMutation { x: countedWrite(value: 1) { value } }", operation_name: Some("Q"), variables: Some(json!({"include": false})) },
        ]
    }

    #[tokio::test]
    async fn guard_verdict_matches_pinned_junipers_actual_observed_execution_count() {
        let schema = schema(); let context = GuardContext::default();
        for case in cases() {
            context.reset(); let req = request(case.query, case.operation_name, case.variables);
            let body = execute_json(&schema, &context, &req).await; assert!(body.get("data").is_some());
            let actual_writes = context.writes(); let decision = guard(MutationGuardMode::Enforce, &schema, &req);
            assert_eq!(matches!(decision.outcome, GuardOutcome::Reject { .. }), actual_writes > 1, "{}", case.label);
        }
    }
}

mod duplicate_mutation_regression {
    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn repeated_top_level_mutation_response_key_still_executes_once_per_occurrence() {
        let schema = schema(); let context = GuardContext::default(); let req = request(duplicate_mutation(), None, None);
        let body = execute_json(&schema, &context, &req).await;
        assert!(body["errors"].is_null() || body.get("errors").is_none());
        assert!(body["data"]["x"].is_object()); assert_eq!(context.writes(), 2);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn repeated_top_level_mutation_response_key_via_fragment_still_executes_twice() {
        let schema = schema(); let context = GuardContext::default();
        let req = request("mutation { ...Top } fragment Top on GuardMutation { x: countedWrite(value: 1) { value } x: countedWrite(value: 1) { value } }", None, None);
        let body = execute_json(&schema, &context, &req).await;
        assert!(body["data"]["x"].is_object()); assert_eq!(context.writes(), 2);
    }
}
