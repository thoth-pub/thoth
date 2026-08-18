# THOTH-GQL-DATALOADER-01 Implementation Report

## 1. Repository state

Repository: `thoth-pub/thoth`
Workflow: STANDARD
Base branch: `develop`
Base commit: `060052b47490d3d977db3b9d9f188c4c70760a9a` (merge of PR [#801](https://github.com/thoth-pub/thoth/pull/801); reconciled live base — `origin/develop` was identical at implementation-takeover preflight)
PR target: `develop`
Programme integration branch: None
Task branch: `feature/shared-architecture/graphql-dataloader-foundation`
Head commit: established after push through GitHub PR #802 metadata (this file cannot embed the SHA of its own containing commit); the PR head at review time is the authoritative exact head
Pull request: [#802](https://github.com/thoth-pub/thoth/pull/802) (DRAFT)
Expected branch deletion after merge: YES
Final programme PR required: NO
Implementing model: high-reasoning agent (initial implementation), completed and corrected by a second high-reasoning takeover agent auditing the inherited branch against the approved specification
Reasoning level: high

Authorization: CTO implementation authorization recorded on merged PR
[#801](https://github.com/thoth-pub/thoth/pull/801) (2026-08-11). This report
records implementation only. Independent exact-head review and separate
explicit CTO merge authorization remain required and are **not** granted here.

## 2. Scope confirmation

Approved specification: `docs/engineering/ai-delivery/tasks/THOTH-GQL-DATALOADER-01.md` (Status: APPROVED)
Architecture: `docs/engineering/decisions/ADR-0007-conventional-request-scoped-graphql-dataloader.md` (APPROVED, repository-authoritative; supersedes ADR-0006)

Implemented objective: the `ADR-0007` B0 foundation — conventional
request-scoped **non-cached** DataLoaders on pinned Juniper 0.16.2, async
Juniper execution as the supported general test execution model, a safe
reusable set-based synchronous-Diesel batching pattern behind
`tokio::task::spawn_blocking`, a safe shareable batch-error projection,
retirement of the superseded `ADR-0006` A2 batching/store/scope
infrastructure, and decoupling of loader availability from mutation-guard
mode. No production GraphQL field adopts a loader.

Out-of-scope changes made: NONE

## 3. Commits

Inherited implementation commits (first agent), audited rather than assumed
correct:

- `3e6d6c98` - feat: add request-scoped DataLoader foundation
- `162feab3` - test: add DataLoader fixture infrastructure
- `638fa134` - test: cover DataLoader batching and failure semantics
- `ae057584` - refactor: retire A2 module wiring
- `862488d2` - test: rehost mutation guard regressions off A2
- `e03cee09` - build: add conventional DataLoader runtime dependencies
- `d50dd567` - fix: preserve existing optional dependency features
- `84912522` - refactor: decouple request context from guard mode
- `cc718e7e` - refactor: retire A2 look-ahead prefetch
- `fb88ef36` - refactor: retire A2 response-scope shim
- `608bd77d` - refactor: retire A2 test fixture
- `125aa880` - refactor: retire A2-specific batching tests
- `9fe4af2e` - chore: keep temporary A2 compile bridge during Context migration
- `575139fe` - refactor: remove A2 store implementation behind migration alias
- `9e2c0ec1` - refactor: make loader bundle guard-independent during Context migration
- `0e2fb1b5` - test: point fixture at temporary loader alias field
- `dc7f1948` - refactor: remove mutation guard batching availability coupling
- `776dae6c` - test: remove obsolete guard-store availability assertion
- `89d713d0` - test: fix request-isolation fixture bridge and tighten loader evidence
- `c327bf3b` - test: prove DataLoader backend failure equivalence
- `56f50274` - test: add backend failure equivalence suite

Takeover completion commits (this report's agent):

- `b7a24a6c` - refactor: complete Context migration onto request-local loader bundle
- `75de87f1` - test: correct defective loader and guard evidence fixtures
- `12e08998` - test: migrate general GraphQL execution to the async bridge
- `3f770f4d` - build: commit resolved dataloader lockfile
- `fe2ca4e6` - docs: add PR #802 implementation changelog entry
- `4b0f11f3` - docs: add THOTH-GQL-DATALOADER-01 implementation report
- `dee36f1f` - test: make delayed-cohort fragmentation deterministic (the
  inherited 1 ms-sleep scheduling fixture asserted fragmentation **must**
  occur; a loaded CI runner coalesced it into one dispatch at head
  `4b0f11f3` — every other check passed there — so the fixture now gates the
  delayed cohort on the first dispatch having demonstrably happened, making
  fragmentation deterministic; the full local gate was re-run green at this
  commit)
- plus the final documentation commit carrying this report update (the PR
  head at review time is authoritative)

## 4. Files changed

- `thoth-api/Cargo.toml`
  - reason: add `dataloader = { version = "0.18", default-features = false, features = ["runtime-tokio"] }` and direct normal `tokio` (`rt`) for the production `spawn_blocking` boundary; dev-only Tokio features are unchanged (`macros`, `rt`, `rt-multi-thread`).
  - behavioural effect: dependency addition only; no production runtime behaviour change.
- `Cargo.lock`
  - reason: commit the resolved dependency authority — `dataloader 0.18.0` (sole dependency edge: `tokio`), generated by cargo, not hand-authored.
  - behavioural effect: none beyond pinning.
- `thoth-api/src/graphql/dataloader.rs` (new)
  - reason: the ADR-0007 foundation — explicit `LoaderConfig { max_batch_size: 200, yield_count: 10 }`, the `configured_loader` constructor applying it, the request-local `RequestLoaders` bundle, the `FieldErrorConvention` enum and the cloneable non-serde `SharedBatchError` projection.
  - behavioural effect: none in production builds (no production consumer); the bundle is constructed and dropped with every `Context`.
- `thoth-api/src/graphql/dataloader/fixture.rs` (new, test-only)
  - reason: representative loader consumers — in-memory batchers with dispatch statistics, a real-Diesel `DbBatcher` (`spawn_blocking` + `eq_any`), a test GraphQL schema with loader-first/yield/delayed/chained resolver shapes, and the rehosted `SqlProbe` Diesel connection-instrumentation harness.
  - behavioural effect: test-only (`#[cfg(all(test, feature = "backend"))]`).
- `thoth-api/src/graphql/dataloader/tests.rs`, `thoth-api/src/graphql/dataloader/failure_tests.rs` (new, test-only)
  - reason: the section 10 evidence matrix (details in section 9 below).
  - behavioural effect: test-only.
- `thoth-api/src/graphql/model.rs`
  - reason: final Context migration — `Context` directly owns `pub(crate) loaders: RequestLoaders`; `Context::new` is again the single four-argument constructor; `with_guard_mode`, the `batch_store` field, the A2 imports and the ADR-0006 invariant-30 comments are removed.
  - behavioural effect: request context construction no longer takes a guard mode; loader availability is guard-independent.
- `thoth-api/src/graphql/mod.rs`
  - reason: remove `batching`/`batching_fixture`/`batching_tests`/`prefetch`/`scope` module wiring; add `dataloader` and `mutation_guard_tests`; update `run_mutation_guard` docs to the ADR-0007 boundary.
  - behavioural effect: none (module wiring and docs).
- `thoth-api/src/graphql/batching.rs`, `batching_fixture.rs`, `batching_tests.rs`, `prefetch.rs`, `scope.rs` (deleted)
  - reason: A2 retirement (section 7 of the specification); `batching.rs` briefly survived as a handoff-time compatibility alias and is now fully removed.
  - behavioural effect: none — the A2 foundation had no production-field consumer.
- `thoth-api/src/graphql/mutation_guard.rs`
  - reason: remove `MutationGuardMode::store_available()` and the guard-as-store-switch documentation; the guard mechanism, evaluation semantics, default `OFF` and activation state are untouched.
  - behavioural effect: none at runtime (the method's only callers were A2 code/tests).
- `thoth-api/src/graphql/mutation_guard_tests.rs` (new, test-only)
  - reason: rehost the guard/query-path/baseline/directive/duplicate-mutation regression evidence onto a minimal counter-instrumented fixture with no store, loader or A2 dependency.
  - behavioural effect: test-only.
- `thoth-api/src/graphql/tests.rs`
  - reason: migrate the general GraphQL unit-test execution path from `juniper::execute_sync` to async Juniper through the bounded central `block_on_graphql` bridge (all 7 former `execute_sync` sites); the bridge fails explicitly on nested-runtime misuse, with tests for both behaviours.
  - behavioural effect: test-only.
- `thoth-api/src/model/tests.rs`
  - reason: delete the obsolete `test_context_with_guard_mode` helper and its ADR-0006 invariant-30 comment.
  - behavioural effect: test-only.
- `thoth-api-server/src/lib.rs`
  - reason: the `graphql` handler constructs the request `Context` with `Context::new(..)` (guard-independent) after the unchanged request-boundary `run_mutation_guard` check; the superseded store-availability comment is replaced. Handler test comments preserved from base.
  - behavioural effect: none observable — guard evaluation, ordering, statuses and response bodies are unchanged; the context simply no longer carries a guard mode.
- `src/bin/thoth.rs`
  - reason: remove the `store_availability_is_derived_only_from_enforce` test with the removed method; CLI/environment mode-resolution tests are preserved.
  - behavioural effect: test-only.
- `CHANGELOG.md`
  - reason: required PR #802 `Unreleased` entry.
- `docs/engineering/ai-delivery/implementation-reports/THOTH-GQL-DATALOADER-01-implementation-report.md` (this file)

## 5. Implementation decisions

Decisions within the approved design:

1. Bundle name `RequestLoaders`, owned as `Context.loaders` (the specification left naming open).
2. The production bundle carries no field-specific loader (approved merged state); the test fixture loaders are injected through a `#[cfg(all(test, feature = "backend"))]` `fixture` field so the real `Context` lifecycle is exercised without any production surface.
3. `SharedBatchError` snapshots `(message, Option<&'static str> extension type)` per `FieldErrorConvention`, mirroring `ThothError::into_field_error`'s exact mapping for the explicit convention and plain `Display` for the conventional one; shared across keys via `Arc<str>` clone. No serde round trip (grep-level evidence test included).
4. The async test bridge `block_on_graphql` builds one current-thread Tokio runtime per call and `assert!`s `Handle::try_current().is_err()` so nested-runtime misuse fails explicitly with an actionable message (specification section 3.8.2).
5. Foundation items with no production consumer carry `#[cfg_attr(not(test), allow(dead_code))]`, matching the repository convention already used at base for the merged-but-unconsumed A2 state; this reflects the approved "no production consumer" merged state, not lint suppression of a defect.
6. Takeover corrections to inherited defects (audit findings, section 13):
   - fixed the `batch_wide_in_memory_failure...` test, which asserted an impossible response shape — `children` is a non-null list (`[String!]!`), so a per-key error null-propagates the whole `data` object; the corrected test proves totality at the loader level (every key receives an error from exactly one dispatch) and correct propagation at the GraphQL level;
   - fixed the `event_contains_shape_metadata...` guard test, whose fixture was baseline-invalid: pinned Juniper's `validate_input_values` rejects an absent non-null variable even when the definition carries a default, so the guard correctly produced no event; the variable is now supplied;
   - strengthened the missing-key and failure-equivalence assertions that previously indexed into `null` and therefore passed vacuously;
   - restored base explanatory comments in the preserved `thoth-api-server` guard HTTP tests that the inherited branch had stripped without specification mandate.

Deviations from the specification: NONE

## 6. Database and migration effects

Migration added: NO

- Database migration: NONE
- Data migration: NONE
- GraphQL schema migration: NONE
- `thoth-api/src/schema.rs`: untouched

## 7. API and compatibility effects

GraphQL/API changes: NONE — production SDL byte-identical (evidence below).

Production SDL (`create_schema().as_sdl()`, generated by `thoth-client/build.rs` at base and at head):

```text
base  bytes: 160799  sha256: 1e08b46b565ef719c404bbe6b3131e6a733df09c7abdc4538b66c2b24d2d899c
head  bytes: 160799  sha256: 1e08b46b565ef719c404bbe6b3131e6a733df09c7abdc4538b66c2b24d2d899c
byte-identical: YES (cmp exit 0; generated by `cargo check --workspace` via thoth-client/build.rs at base worktree 060052b4 and at the implementation head)
```

Generated schema/client updates: none required (SDL identical).
Backwards compatibility: full — no public contract change.
Deprecations: none.
Cross-repository dependencies: none.

## 8. Authorization and security

Authorization paths changed: NONE. DataLoader is not an authorization layer
and the foundation changes no authorization behaviour (`ADR-0007` section
4.11). The test fixture executes with the existing anonymous test context and
touches no protected query predicate; the existing positive/negative
authorization suites in `graphql::tests` continue to pass unchanged through
the async bridge. Future protected production loaders require field-specific
authorization evidence under their own adopting task.

Roles/scopes involved: none changed.
Negative authorization tests: existing suites preserved (e.g. publisher
package/capability rejection tests, anonymous-caller handler test asserting
`Invalid credentials.` before any write).
Secret or personal-data handling: none.
Security limitations: the pinned-Juniper duplicate top-level mutation
execution finding remains a live, separately controlled concern (`ADR-0007`
section 4.13); this task does **not** fix it and preserves its regression
evidence.

## 9. Tests and checks

### Formatting

Command:

```text
cargo fmt --all -- --check
```

Result:

```text
exit 0; no formatting differences
```

### git diff --check

Command:

```text
git diff --check
```

Result:

```text
exit 0; no whitespace errors
```

### Type check

Command:

```text
cargo check --workspace
```

Result:

```text
exit 0; Finished `dev` profile in 1m 02s. (Pre-existing upstream note: proc-macro-error2 v2.0.1 future-incompatibility warning, present at base, unrelated to this change.)
```

### Lint/static analysis

Command:

```text
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Result:

```text
exit 0 with `-D warnings`; Finished `dev` profile in 1m 09s; no lint findings
```

### Unit/integration tests (thoth-api, backend)

Command:

```text
cargo test -p thoth-api --features backend
```

Result:

```text
exit 0. Unit tests (src/lib.rs): 909 passed; 0 failed; 0 ignored. Integration tests (tests/graphql_permissions.rs): 13 passed; 0 failed. Doc-tests: 0 passed; 8 ignored.
```

### Workspace tests

Command:

```text
cargo test --workspace
```

Result:

```text
exit 0. thoth bin: 14 passed. thoth-api lib: 909 passed. thoth-api graphql_permissions: 13 passed. thoth-api-server: 3 passed. thoth-client: 4 passed (+6 doc-tests). thoth-errors: 11 passed. thoth-export-server: 144 passed (+2 doc-tests). 0 failed everywhere.
```

### Focused DataLoader / bridge / mutation-guard evidence

Command:

```text
cargo test -p thoth-api --features backend --lib -- graphql::dataloader graphql::mutation_guard_tests graphql::tests::block_on_graphql
```

Result:

```text
exit 0. 40 passed; 0 failed (869 filtered out) — 25 DataLoader foundation/evidence tests, 2 backend-failure equivalence tests, 11 rehosted mutation-guard/regression tests (plus module-level async/bridge coverage), 2 async-bridge behaviour tests.
```

Evidence matrix (specification section 10 → test):

- 10.1 async execution: `genuine_async_child_resolver_executes_on_pinned_juniper`; the migrated `graphql::tests` suite through `block_on_graphql`; `block_on_graphql_runs_juniper_async_execution_from_a_sync_test`; nested-runtime misuse fails explicitly — `block_on_graphql_fails_explicitly_inside_a_running_runtime`.
- 10.2 batch boundaries: `batch_boundaries_current_thread` and `batch_boundaries_multi_thread` prove N=1→`[1]`, 100→`[100]`, 200→`[200]`, 201→`[200, 1]`, 500→`[200, 200, 100]` on both Tokio runtime flavours, with every sibling resolver calling `try_load` independently.
- 10.3 configuration: `production_constructor_uses_explicit_200_10_configuration_constants` plus the boundary shapes themselves (a 201st key splitting into `[200, 1]` is only possible with the explicit 200 configuration).
- 10.4 scheduling: `scheduling_immediate_and_benign_yield_coalesce` (1 dispatch), `scheduling_delayed_cohort_can_fragment_dispatch` (fragmentation is expected scheduler behaviour, documenting why loader-first is binding), `scheduling_loader_behind_loader_remains_set_based_for_target` (upstream loader dependency; target stays one dispatch).
- 10.5 request isolation: `two_request_contexts_share_no_loader_state` — two `Context`s, same logical key, marker-distinguished sources, one dispatch each.
- 10.6 non-caching: `completed_result_is_not_cached_and_read_write_read_is_fresh`, `pending_duplicate_keys_may_coalesce_without_becoming_cache` (pending coalescing is batching, not completed-value caching).
- 10.7 read-write-read freshness: `mutation_read_write_read_observes_new_value_without_invalidation` (GraphQL mutation payload re-loads after write; 2 dispatches, no invalidation call).
- 10.8 missing-key: `missing_batch_result_fails_closed_through_try_load_without_panic` — omitted key fails closed through `try_load` as a GraphQL error with non-null propagation; no panic, no fabricated success.
- 10.9 real-Diesel query count: `real_diesel_250_parents_use_two_set_based_imprint_statements` — 250 seeded publishers; DataLoader dispatches `[200, 50]`; **exactly 2** imprint statements captured by the rehosted `SqlProbe` Diesel `Instrumentation` hook (external connection instrumentation, not an implementation-side counter), both containing the set-based `= ANY` shape.
- 10.10 backend failure: `failure_tests::conventional_backend_failure_matches_direct_graphql_semantics_without_retry_or_fallback` and `failure_tests::explicit_thoth_backend_failure_preserves_extensions_without_serde_clone` — full serialized direct-vs-loader GraphQL response equality per convention (data/null propagation, path, message, extensions), exactly one failed batch dispatch, no retry/fallback.
- 10.11 error representation: `shared_batch_error_projection_preserves_current_conventions_without_serde` plus the grep-level `source_contains_no_serde_round_trip_error_clone`.
- 10.12 Diesel ownership: compile-time structure of `DbBatcher::load` — `Arc<PgPool>` and owned keys clone into `spawn_blocking`; the connection is acquired, used and dropped entirely inside the blocking closure; only the closure result is awaited. `direct_try_load_batch_function_is_total_for_childless_parent` proves batch totality for the empty relationship.
- 10.14 mutation regression: `duplicate_mutation_regression::repeated_top_level_mutation_response_key_still_executes_once_per_occurrence` (and the fragment variant) prove pinned Juniper still executes a compatible repeated top-level mutation response key twice under **async** execution — the defect is preserved as evidence, not claimed fixed; `guard_tests`, `query_path`, `baseline_matrix` and `directives` rehost the guard regressions A2-independently.

## 10. Manual verification

Environment: local disposable PostgreSQL 17 (`TEST_DATABASE_URL`) and Redis
per repository test convention; no production system touched.
Steps: full validation gate plus focused suites above; SDL generation at base
and head worktrees.
Observed result: as recorded in sections 7 and 9.
Evidence link: CI runs on PR #802 at the final head (section 11).

## 11. CI

CI status: PASSING at code head `dee36f1f` — build-test-and-check (run
31583105994: classify, build, format_check, lint, test), check-changelog
(run 31583105927), run-migrations (run 31583105950) and the staging image
build (run 31583105932) all green. The final documentation commit carrying
this report update re-triggers the same checks; its exact-head results are
recorded on the PR.
Checks: build-test-and-check (classify, build, format_check, lint, test), check-changelog, run_migrations, staging image build
Failures or warnings: one CI-only failure occurred at intermediate head
`4b0f11f3` (`scheduling_delayed_cohort_can_fragment_dispatch`, a
timing-dependent inherited fixture); root-caused and fixed deterministically
in `dee36f1f`, after which the full check set passed

## 12. Rollout and rollback

Initial state after merge: inert foundation — no production GraphQL field
consumes a loader; no public SDL change; no database change; mutation guard
default/effective mode remains `OFF`; no request-acceptance change; no
deployment; no configuration change; no new production logs, metrics or
alerts (specification section 12).

Activation required: NONE (nothing to activate; `OBSERVE`/`ENFORCE`/`BE-02`/
Metrics adoption remain NOT AUTHORIZED).
Feature flag/configuration: none — not required for an internal foundation
with no production consumer (specification section 13.1).
Migration sequence: none.
Rollback/disable procedure: before any production consumer exists, one
bounded revert of this implementation PR. Rollback must not reactivate or
imply `OBSERVE`/`ENFORCE` and must not make `ADR-0006` architecturally
authoritative again; reversing the architecture requires a new ADR.
Monitoring required: none for this foundation; future adopters define their
own under their own approved specifications.

## 13. Known limitations and deferred work

- No production GraphQL field adopts the DataLoader foundation (by design);
  `BE-02` (`Publisher.distributionPlatforms`) and Thoth Metrics adoption
  require their own freshly reconciled approved specifications with
  field-specific query-count, authorization, error, rollout and rollback
  evidence.
- The pinned-Juniper duplicate top-level mutation execution defect remains
  unresolved and separately controlled (`ADR-0007` sections 4.13/7.4);
  its regression evidence is preserved, and its eventual disposition is a
  separate CTO-controlled decision.
- The mutation guard remains merged with default/effective mode `OFF`;
  `OFF -> OBSERVE` and `OBSERVE -> ENFORCE` remain NOT AUTHORIZED.
- Batching efficiency depends on resolver arrival timing; the loader-first
  rule (`ADR-0007` section 4.5) is a binding adoption/review rule for every
  future consumer, enforced through the scheduling fixtures added here.

Audit findings on the inherited branch that were corrected during takeover
(all fixed in this head; recorded for review transparency):

1. Incomplete Context migration (temporary `GraphqlBatchStore` alias,
   `batch_store` field, `with_guard_mode` constructor) — completed and the
   alias module deleted.
2. General GraphQL test path still `execute_sync` — migrated to the async
   bridge with nested-runtime misuse evidence.
3. `batch_wide_in_memory_failure...` asserted an impossible non-null
   propagation shape — corrected (root-caused, not suppressed).
4. `event_contains_shape_metadata...` used a baseline-invalid fixture
   (absent non-null variable with default) — corrected (root-caused against
   Juniper's `validate_input_values`).
5. Dead `test_context_with_guard_mode` helper — deleted with its ADR-0006
   comment.
6. `Cargo.lock` not committed — committed (cargo-generated).
7. Missing changelog entry — added.
8. rustfmt failures on new files — formatted.
9. Vacuously-true assertions (indexing into JSON `null`) in missing-key and
   failure tests — strengthened.
10. Unmandated comment stripping in preserved `thoth-api-server` guard HTTP
    tests — restored to base.

## 14. Unresolved issues

- NONE

## 15. Agent self-assessment

The implementing agent may identify risks but may not approve the task.

Suggested review focus:

- the final `Context` diff in `thoth-api/src/graphql/model.rs` (single
  constructor, direct `loaders` ownership, no guard parameter);
- the `DbBatcher::load` blocking boundary (connection lifetime entirely
  inside `spawn_blocking`; totality over requested keys; `eq_any` shape);
- the corrected failure-semantics tests: non-null list propagation means a
  per-key error nulls `data` — reviewers should confirm this matches their
  reading of the direct-path contract (the direct-vs-loader equality tests
  in `failure_tests.rs` are the strongest evidence);
- `SharedBatchError::from_thoth` mirroring `ThothError::into_field_error`'s
  mapping (a future change to that mapping must be reflected here; test
  `shared_batch_error_projection_preserves_current_conventions_without_serde`
  pins it);
- absence of any `Loader::load()` call, static/global loader, or serde error
  round trip (grep-verifiable).

Merge authorization: NOT GRANTED
Deployment authorization: NOT GRANTED
