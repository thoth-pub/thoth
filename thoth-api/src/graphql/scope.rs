//! The pinned-Juniper compatibility shim for top-level response-key scoping
//! (`ADR-0006` section 4.12.8).
//!
//! # What this exists for
//!
//! The request-scoped batch store is partitioned by the **top-level GraphQL
//! response key** so that no loader entry — successful or failed — crosses two
//! top-level fields. A nested resolver must therefore be able to answer "which
//! top-level response key am I executing under?".
//!
//! # Why it is a shim
//!
//! Pinned juniper 0.16.2 exposes no dedicated public path accessor.
//! `Executor::field_path` is a private field, and `FieldPath::construct_path` /
//! `FieldPath::location` are private methods, so although `FieldPath` itself is
//! reachable its contents are not. The accepted mechanism is:
//!
//! ```text
//! Executor::new_error(..) -> ExecutionError::path() -> first response-key segment
//! ```
//!
//! Both `Executor::new_error` (`src/executor/mod.rs:679`) and
//! `ExecutionError::path` (`src/executor/mod.rs:797`) are public and
//! documented — unlike the `#[doc(hidden)]` surfaces the mutation guard's
//! eligibility gate depends on — so this carries the weaker of the two coupling
//! risks. It is nonetheless a **compatibility shim, not business logic**.
//!
//! # Pinned-Juniper coupling and the revalidation obligation
//!
//! This module is coupled to pinned juniper **0.16.2** behaviour, specifically
//! that `new_error` materializes the current execution path and that the first
//! segment of that path is the top-level response key. Per `ADR-0006` section
//! 4.12.14 this mechanism **must be revalidated on any juniper upgrade**,
//! before merge and before activation. The tests in this module are the
//! revalidation harness: if an upgrade changes the path representation they
//! fail rather than silently returning a wrong scope.
//!
//! # Why it is side-effect-free
//!
//! `new_error` constructs an `ExecutionError` from `field_path.construct_path(..)`
//! and returns it. It does **not** touch the executor's shared error
//! collection — that is `push_error_at`, which acquires `self.errors.write()`
//! and pushes. Calling `new_error` therefore adds no GraphQL error, changes no
//! `errors[]` entry, changes no result data, and performs no database access.
//! The constructed error is discarded once its path has been read.
//!
//! # This is the only permitted site
//!
//! `new_error(..)` path extraction must appear in **exactly one** module.
//! Loaders, prefetch sites and resolvers all obtain their scope from
//! [`top_level_response_key`] and must never use the technique directly.

use juniper::{Executor, FieldError, ScalarValue};

use super::batching::ScopeKey;

/// Derive the top-level GraphQL response key for the executor's current
/// position.
///
/// Returns [`None`] when no top-level response key can be derived — for example
/// an empty path, which is what `construct_path` produces at `FieldPath::Root`.
///
/// # Fail closed
///
/// Callers **must** treat [`None`] as "no scope", per `ADR-0006` section
/// 4.12.9:
///
/// - a **prefetch site** that cannot derive its scope performs no prefetch, and
///   does not fail the parent list field;
/// - a **terminal child resolver** that cannot derive its scope treats its
///   lookup as `NotLoaded` and takes the ordinary direct-query fallback.
///
/// Substituting a shared or request-global namespace is **prohibited**: that
/// would let entries cross top-level scopes, which is the one thing this
/// partition exists to prevent. Degrading to the correctness fallback is the
/// safe direction; degrading to a shared namespace is not.
///
/// The returned key is the GraphQL **response key** — therefore the alias when
/// one is present — and is never normalized to the schema field name
/// (`ADR-0006` invariant 21).
pub(crate) fn top_level_response_key<S, C>(executor: &Executor<'_, '_, C, S>) -> Option<ScopeKey>
where
    S: ScalarValue,
{
    // `new_error` is called solely to materialize the current execution path.
    // The error value is constructed, read, and dropped; it is never pushed.
    let materialized = executor.new_error(FieldError::<S>::from(SCOPE_PROBE));
    materialized
        .path()
        .first()
        .filter(|segment| !segment.is_empty())
        .map(ScopeKey::new)
}

/// The placeholder message carried by the discarded probe error.
///
/// It never reaches a client: the `ExecutionError` built around it is dropped
/// as soon as its path has been read. It exists only because `new_error`
/// requires a `FieldError` argument.
const SCOPE_PROBE: &str = "scope probe";

#[cfg(test)]
mod tests {
    //! Revalidation harness for the pinned-Juniper coupling of `ADR-0006`
    //! section 4.12.14.
    //!
    //! These tests assert the two properties the shim depends on:
    //!
    //! 1. `Executor::new_error(..) -> ExecutionError::path()` yields the
    //!    execution path, whose **first** segment is the top-level response key
    //!    (the alias when present);
    //! 2. calling it is side-effect-free — no GraphQL error is added, no
    //!    `errors[]` entry changes, no result data changes.
    //!
    //! If a juniper upgrade breaks either, these fail rather than letting a
    //! wrong scope reach the store.

    use std::sync::{Arc, Mutex};

    use juniper::{graphql_object, EmptyMutation, EmptySubscription, RootNode, Variables};

    use super::*;

    /// One probe observation: the site that ran, and the scope it derived.
    type Observation = (&'static str, Option<String>);

    /// Records the scope every probed resolver observed, in execution order.
    #[derive(Clone, Default)]
    struct Recorder(Arc<Mutex<Vec<Observation>>>);

    impl Recorder {
        fn record(&self, site: &'static str, scope: Option<ScopeKey>) {
            self.0
                .lock()
                .expect("recorder lock")
                .push((site, scope.map(|key| key.as_str().to_string())));
        }

        fn entries(&self) -> Vec<Observation> {
            self.0.lock().expect("recorder lock").clone()
        }
    }

    struct ProbeContext {
        recorder: Recorder,
    }

    impl juniper::Context for ProbeContext {}

    struct Leaf;

    #[graphql_object(Context = ProbeContext, Scalar = juniper::DefaultScalarValue)]
    impl Leaf {
        /// A deeply nested descendant: must observe the same first path
        /// segment as its ancestors.
        fn deep(context: &ProbeContext, executor: &Executor<'_, '_, ProbeContext>) -> String {
            let scope = top_level_response_key(executor);
            context.recorder.record("leaf.deep", scope.clone());
            scope.map(|k| k.as_str().to_string()).unwrap_or_default()
        }
    }

    struct Branch;

    #[graphql_object(Context = ProbeContext, Scalar = juniper::DefaultScalarValue)]
    impl Branch {
        /// A direct child of the top-level field.
        fn child(context: &ProbeContext, executor: &Executor<'_, '_, ProbeContext>) -> String {
            let scope = top_level_response_key(executor);
            context.recorder.record("branch.child", scope.clone());
            scope.map(|k| k.as_str().to_string()).unwrap_or_default()
        }

        fn leaf() -> Leaf {
            Leaf
        }
    }

    struct ProbeQuery;

    #[graphql_object(Context = ProbeContext, Scalar = juniper::DefaultScalarValue)]
    impl ProbeQuery {
        /// The top-level field itself.
        fn top(context: &ProbeContext, executor: &Executor<'_, '_, ProbeContext>) -> Branch {
            let scope = top_level_response_key(executor);
            context.recorder.record("query.top", scope);
            Branch
        }

        fn other(context: &ProbeContext, executor: &Executor<'_, '_, ProbeContext>) -> Branch {
            let scope = top_level_response_key(executor);
            context.recorder.record("query.other", scope);
            Branch
        }
    }

    type ProbeSchema =
        RootNode<'static, ProbeQuery, EmptyMutation<ProbeContext>, EmptySubscription<ProbeContext>>;

    fn probe_schema() -> ProbeSchema {
        ProbeSchema::new(ProbeQuery, EmptyMutation::new(), EmptySubscription::new())
    }

    /// Execute `query` and return (recorded scopes, GraphQL errors, data).
    /// The full result of one probe execution.
    type ProbeRun = (
        Vec<Observation>,
        Vec<juniper::ExecutionError<juniper::DefaultScalarValue>>,
        juniper::Value,
    );

    fn run(query: &str) -> ProbeRun {
        let recorder = Recorder::default();
        let context = ProbeContext {
            recorder: recorder.clone(),
        };
        let schema = probe_schema();
        let (data, errors) =
            juniper::execute_sync(query, None, &schema, &Variables::new(), &context)
                .expect("probe execution failed");
        (recorder.entries(), errors, data)
    }

    fn scope_of(entries: &[Observation], site: &str) -> Option<String> {
        entries
            .iter()
            .find(|(name, _)| *name == site)
            .and_then(|(_, scope)| scope.clone())
    }

    #[test]
    fn unaliased_top_level_field_returns_its_field_response_key() {
        let (entries, errors, _) = run("{ top { child } }");
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
        assert_eq!(scope_of(&entries, "query.top").as_deref(), Some("top"));
    }

    #[test]
    fn aliased_top_level_field_returns_the_alias_not_the_field_name() {
        let (entries, errors, _) = run("{ renamed: top { child } }");
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
        assert_eq!(
            scope_of(&entries, "query.top").as_deref(),
            Some("renamed"),
            "the scope key must be the response key (the alias), never the schema field name"
        );
    }

    #[test]
    fn direct_child_returns_the_same_first_segment_as_its_parent_site() {
        let (entries, errors, _) = run("{ renamed: top { child } }");
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
        assert_eq!(scope_of(&entries, "query.top").as_deref(), Some("renamed"));
        assert_eq!(
            scope_of(&entries, "branch.child").as_deref(),
            Some("renamed"),
            "a direct child must derive the same scope as its top-level site"
        );
    }

    #[test]
    fn deeply_nested_descendant_returns_the_same_first_segment() {
        let (entries, errors, _) = run("{ renamed: top { leaf { deep } } }");
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
        assert_eq!(
            scope_of(&entries, "leaf.deep").as_deref(),
            Some("renamed"),
            "a deeply nested descendant must still derive the top-level response key"
        );
    }

    #[test]
    fn aliases_at_intermediate_segments_do_not_change_the_top_level_scope() {
        let (entries, errors, _) = run("{ renamed: top { l: leaf { d: deep } } }");
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
        assert_eq!(scope_of(&entries, "leaf.deep").as_deref(), Some("renamed"));
    }

    #[test]
    fn inline_fragments_preserve_the_same_first_scope() {
        let (entries, errors, _) = run("{ renamed: top { ... on Branch { leaf { deep } } } }");
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
        assert_eq!(scope_of(&entries, "leaf.deep").as_deref(), Some("renamed"));
    }

    #[test]
    fn named_fragments_preserve_the_same_first_scope() {
        let (entries, errors, _) = run(
            "{ renamed: top { ...BranchFields } } fragment BranchFields on Branch { leaf { deep } }",
        );
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
        assert_eq!(scope_of(&entries, "leaf.deep").as_deref(), Some("renamed"));
    }

    #[test]
    fn two_top_level_aliases_of_one_field_derive_distinct_scopes() {
        let (entries, errors, _) = run("{ a: top { child } b: top { child } }");
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
        let scopes: Vec<_> = entries
            .iter()
            .filter(|(site, _)| *site == "query.top")
            .map(|(_, scope)| scope.clone())
            .collect();
        assert_eq!(
            scopes,
            vec![Some("a".to_string()), Some("b".to_string())],
            "two top-level aliases of one schema field must be separate namespaces"
        );
    }

    #[test]
    fn site_and_terminal_resolver_derive_identical_scope_within_one_top_level_field() {
        let (entries, errors, _) = run("{ renamed: top { child leaf { deep } } }");
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
        let site = scope_of(&entries, "query.top");
        let direct = scope_of(&entries, "branch.child");
        let descendant = scope_of(&entries, "leaf.deep");
        assert_eq!(site, direct);
        assert_eq!(site, descendant);
        assert_eq!(site.as_deref(), Some("renamed"));
    }

    #[test]
    fn calling_the_shim_adds_no_graphql_error_and_changes_no_result_data() {
        // The shim is called at three sites in this document.
        let (_, errors, data) = run("{ renamed: top { child leaf { deep } } }");
        assert!(
            errors.is_empty(),
            "the scope helper must add no GraphQL error, got: {errors:?}"
        );

        // And the data is exactly what the resolvers returned — the discarded
        // probe error never becomes a null or an `errors[]` entry.
        let serialized = serde_json::to_value(&data).expect("serialize probe data");
        assert_eq!(
            serialized,
            serde_json::json!({
                "renamed": {
                    "child": "renamed",
                    "leaf": { "deep": "renamed" }
                }
            })
        );
    }

    #[test]
    fn shim_is_side_effect_free_across_repeated_calls() {
        // Calling it many times in one execution must still add nothing.
        let (entries, errors, _) =
            run("{ a: top { child leaf { deep } } b: top { child leaf { deep } } }");
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
        assert_eq!(
            entries.len(),
            6,
            "expected six probe calls across the two top-level scopes"
        );
    }
}
