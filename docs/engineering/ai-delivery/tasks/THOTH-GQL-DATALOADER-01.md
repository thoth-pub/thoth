# THOTH-GQL-DATALOADER-01 - Request-scoped GraphQL DataLoader foundation and A2 retirement

Status: DRAFT
Implementation: NOT AUTHORIZED
Programme: Shared Thoth GraphQL / Backend Architecture
Dependent programmes: Publisher Services and Distribution Configuration (first
expected consumer, `BE-02`, under its own later task); Thoth Metrics and any
other programme resolving child fields through `thoth-api` GraphQL (later
consumers only)
Repository: `thoth-pub/thoth`
Workflow: STANDARD
Base branch: `develop`
Base commit at specification time: `687ee0a40360fb28ef9aab1aa41fc69e35ed93ea`
(the merge of PR [#800](https://github.com/thoth-pub/thoth/pull/800); the
implementing agent must live-preflight and reconcile a fresh exact base before
any code change — see section 3.1)
PR target: `develop`
Programme integration branch: None
Risk: HIGH
Owner: Shared backend architecture
Approved by: not yet approved
Dependencies, all required before implementation may begin:
[`ADR-0007`](../../decisions/ADR-0007-conventional-request-scoped-graphql-dataloader.md)
`APPROVED` and repository-authoritative (satisfied 2026-08-11 through PR
[#800](https://github.com/thoth-pub/thoth/pull/800)); this specification
approved; a freshly verified exact `develop` base; explicit CTO implementation
authorization
Target branch name: `feature/shared-architecture/graphql-dataloader-foundation`
(**must not exist** until implementation is authorized)

Authority condition: this record is repository-authoritative when this exact
content is reachable from the repository's authoritative integration branch.
Live review, authorization and merge evidence is the GitHub pull-request record.

This specification does not authorize implementation. It defines what
implementation must do once separately authorized. `THOTH-GQL-DATALOADER-01`
implementation remains `NOT AUTHORIZED` until this specification is approved by
the CTO after fresh independent exact-head review.

Recommended implementation model: a high-reasoning implementing agent for the
bounded implementation, and a separate high-reasoning independent reviewer of
the exact PR head. The implementing agent must not approve or merge its own
work (section 16).

---

## 1. Objective

Establish the `ADR-0007` B0 foundation: conventional request-scoped,
**non-cached** DataLoaders on the current pinned Juniper 0.16.x execution
stack, with async Juniper execution as the supported test/resolver model, safe
reusable infrastructure for set-based synchronous Diesel batching behind an
approved blocking boundary, and a safe shareable batch-error representation.
In the same bounded task, retire the superseded `ADR-0006` A2
batching/store/scope infrastructure, which has no production-field consumer,
and decouple batching availability from mutation-guard mode.

The task must:

1. establish the `ADR-0007` conventional request-scoped non-cached DataLoader
   foundation;
2. make async Juniper execution the supported test/resolver model required by
   that foundation;
3. create safe reusable infrastructure for set-based synchronous Diesel
   batching behind the approved blocking boundary (section 6.7);
4. remove the `ADR-0006` A2 batching/store/scope infrastructure that has no
   production-field consumer (section 7);
5. decouple batching availability from mutation-guard mode (section 8);
6. preserve the independent mutation execution/guard concern and its
   regression evidence (section 8);
7. adopt **no** production GraphQL child field (section 9).

The foundation must be ready for later `BE-02` adoption without implementing
`BE-02`. The production GraphQL SDL must remain byte-identical (section 10.13).

## 2. Background and authority

Authoritative sources:

- [`ADR-0007`](../../decisions/ADR-0007-conventional-request-scoped-graphql-dataloader.md) -
  `APPROVED` 2026-08-11, repository-authoritative since the merge of PR
  [#800](https://github.com/thoth-pub/thoth/pull/800)
  (`687ee0a40360fb28ef9aab1aa41fc69e35ed93ea`). This task implements `ADR-0007`.
  It may refine implementation detail only where `ADR-0007` explicitly leaves a
  choice open; it must not change, weaken, reinterpret or bypass an `ADR-0007`
  invariant. If a necessary implementation choice would materially change the
  approved architecture, the implementing agent must stop, return `BLOCKED`,
  and surface it as a new architecture decision;
- [`ADR-0006`](../../decisions/ADR-0006-request-scoped-graphql-batching.md) -
  `SUPERSEDED` for batching architecture; preserved unchanged as the historical
  record. Its section 4.12.6 pinned-Juniper
  duplicate-top-level-mutation-execution finding remains a live, separately
  controlled concern under `ADR-0007` section 4.13;
- the independently reviewed B0 feasibility evidence recorded by `ADR-0007`
  section 10 (spike archive
  `THOTH-GQL-DATALOADER-SPIKE-02-EVIDENCE.tar.gz`, SHA-256
  `935d4b913abf6299debae45c32f582c85b691b2a1708a2b9ff9611ff7ac8a769`);
- `AGENTS.md` (repository root) and `thoth-api/AGENTS.md`, including the
  standing section 6 N+1 control;
- [`decision-register.md`](../../decisions/decision-register.md) rows for
  `ADR-0006` and `ADR-0007`.

Dependency posture:

- the current Juniper line is pinned: `juniper` requirement `0.16.1` in
  `thoth-api/Cargo.toml`, resolved as `0.16.2` with `juniper_codegen` `0.16.0`
  by `Cargo.lock`. No Juniper upgrade is part of this task;
- Publisher Services `BE-02` is a **later consumer only**. It is not part of
  this task and requires its own freshly reconciled approved specification
  after this foundation merges;
- Thoth Metrics is a **later consumer only**. It is not part of this task;
- PR [#799](https://github.com/thoth-pub/thoth/pull/799) / `THOTH-GQL-OPS-03`
  is **NOT a dependency** of this task and remains entirely outside its scope.
  It stays frozen, draft and unmerged, untouched by this task.

Current behaviour at the base commit (verified against source, not against
historical task narratives):

- production GraphQL execution is already asynchronous:
  `thoth-api-server/src/lib.rs` (`graphql` handler) calls
  `data.execute(&st, &ctx).await` after the central mutation guard check
  `run_mutation_guard(mode, &data, &st)`;
- the GraphQL `Context` (`thoth-api/src/graphql/model.rs`, `pub struct
  Context`) carries `db: Arc<PgPool>`, `user`, `s3_client`,
  `cloudfront_client`, and the A2 field
  `pub(crate) batch_store: GraphqlBatchStore`. `Context::new(..)` delegates to
  `Context::with_guard_mode(.., MutationGuardMode::Off)`, which constructs
  `GraphqlBatchStore::new(mode)` — store availability is derived from guard
  mode. Production constructs the context per request in
  `thoth-api-server/src/lib.rs` via `Context::with_guard_mode(..)`;
- the A2 foundation merged by `THOTH-GQL-BATCH-01` has **no production-field
  consumer**: in a non-test build, nothing reads `batch_store`, guard mode is
  `OFF` in production, and the store is unavailable. That is the specified
  merged state;
- the general GraphQL unit-test suite executes synchronously: the central
  helper `execute_graphql` at `thoth-api/src/graphql/tests.rs:44-57` calls
  `juniper::execute_sync`, with six additional direct `execute_sync` call
  sites in the same file (`tests.rs:2843`, `:2872`, `:2912`, `:3119`, `:3796`,
  `:3827`). The database integration harness is already async:
  `thoth-api/tests/support/mod.rs` (`execute_graphql`) builds a
  `GraphQLRequest` and calls `request.execute(&schema, &ctx).await`;
- `ThothError` implements `juniper::IntoFieldError`
  (`thoth-errors/src/lib.rs:183-207`) emitting `extensions.type`
  discriminators (`INVALID_SUBJECT_CODE`, `NO_ACCESS`, `INTERNAL_ERROR`), but
  most production resolvers convert errors with generic `.map_err(Into::into)`
  through Juniper's `Display` conversion, which emits **no** `extensions`
  object. `ThothError` is not `Clone`; it offers `to_json`/`from_json` serde
  round-trips, which the B0 spike used as disposable evidence scaffolding and
  which are prohibited as production error plumbing (`ADR-0007` section 4.9);
- representative synchronous Diesel child-field resolvers issue one statement
  per parent, e.g. `Publisher.imprints` (`model.rs:1266`) and
  `Publisher.contacts` (`model.rs:1304`) call `Imprint::all(..)` /
  `Contact::all(..)` with `Some(self.publisher_id)`; `Imprint.publisher`
  (`model.rs:1412`) and `Contact.publisher` (`model.rs:3166`) call per-parent
  lookups. These stay unchanged in this task (no production adoption);
- the connection pool is synchronous r2d2 Diesel:
  `thoth-api/src/db.rs` defines
  `pub type PgPool = Pool<ConnectionManager<PgConnection>>`; resolvers and
  models acquire connections synchronously;
- the production SDL is generated by `create_schema().as_sdl()`
  (`thoth-client/build.rs` writes `assets/schema.graphql` at build time;
  `thoth-api-server/src/lib.rs` serves it at `/schema.graphql`; the SDL guard
  test `generated_schema_exposes_no_package_or_capability_surface` lives at
  `thoth-api/src/graphql/tests.rs:3752`);
- `tokio` `1.52.3` is currently a **dev-dependency** of `thoth-api`
  (`thoth-api/Cargo.toml` `[dev-dependencies]`, features
  `macros`, `rt`, `rt-multi-thread`) and a normal dependency of the root
  `thoth` binary crate. There is no `dataloader` dependency anywhere in the
  workspace.

The complete A2 and mutation-guard source inventory is in sections 7 and 8.

## 3. Explicit scope

### 3.1 Implementation preflight

Before any code change the implementing agent must:

1. fetch and record the exact live `origin/develop` SHA;
2. if it differs from `687ee0a40360fb28ef9aab1aa41fc69e35ed93ea`, inspect every
   intervening commit and determine whether any change affects `ADR-0007`,
   `ADR-0006`, the GraphQL `Context`, the batching/store/scope infrastructure,
   the mutation guard, Juniper execution helpers, GraphQL test harnesses,
   Cargo dependencies, Diesel/pool abstractions, error handling, the generated
   SDL, or repository delivery requirements. Unrelated movement is recorded as
   reconciliation and the new exact base is used; material movement is a stop
   condition (section 14);
3. confirm `ADR-0007` is still `APPROVED` and repository-authoritative;
4. confirm the resolved versions of `juniper` (0.16.2), `juniper_codegen`
   (0.16.0), `diesel` (2.3.10) and `tokio` (1.52.3) are unchanged, or run the
   `ADR-0007` section 4.14 revalidation if they are not;
5. record the exact resolved `dataloader` version after adding the dependency.

### 3.2 Dependency changes

The task must make exactly these dependency changes and no others:

1. add `dataloader` `0.18.x`, initially resolved as `0.18.0`, to
   `thoth-api/Cargo.toml` with **default features disabled** and the crate's
   Tokio runtime support feature enabled (at specification time the expected
   feature name is `runtime-tokio`; the implementing agent must verify the
   exact feature name against the resolved crate and record it). The
   production implementation uses `dataloader::non_cached::Loader` only;
2. make `tokio` a **direct normal dependency** of `thoth-api`, with only the
   features required by production loader infrastructure — at minimum the
   blocking boundary (`tokio::task::spawn_blocking` requires the `rt`
   feature). Test-only Tokio features (`macros`, `rt-multi-thread`) remain
   dev-only where possible;
3. retain `Cargo.lock` as the resolved dependency authority and commit the
   lockfile change.

Prohibited: any Juniper or `juniper_codegen` version change; any
`async-graphql` dependency; any Diesel version change; any other new
dependency not strictly required by the above.

### 3.3 Request-local loader ownership

Add a request-local loader bundle — an explicit struct (suggested name
`RequestLoaders` or equivalent; naming is an open implementation detail) —
owned by the real GraphQL `Context` in `thoth-api/src/graphql/model.rs`.

The actual construction paths the implementation will modify are:

- `Context::new(..)` / `Context::with_guard_mode(..)` in
  `thoth-api/src/graphql/model.rs:79-120` (see section 8 for the guard-mode
  decoupling that applies here);
- the production per-request construction in `thoth-api-server/src/lib.rs`
  (`graphql` POST handler);
- the test constructions in `thoth-api/src/model/tests.rs` (`test_context`,
  `test_context_with_guard_mode`, `test_context_with_user`,
  `test_context_anonymous`) and `thoth-api/tests/support/mod.rs`
  (`execute_graphql`).

Binding requirements:

1. every loader is created for exactly one GraphQL request and owned directly
   or indirectly by that request's `Context`;
2. prohibited: `static` loaders; application-global loaders; cross-request
   loader sharing; completed-result caches that outlive the request; Actix
   application-data ownership of a request loader; reuse of a `Context` or its
   loaders by a later HTTP request;
3. request-local ownership is a correctness and authorization invariant, not a
   performance preference (`ADR-0007` section 4.3).

The foundation ships **test-only** loader consumers (section 3.7); the
production bundle may therefore be empty or carry only the shared construction
machinery in the merged state, provided the real `Context` lifecycle
demonstrably owns and drops it per request.

### 3.4 Loader identity and typed keys

Each loader represents one reviewed logical query/field family
(`ADR-0007` section 4.4):

- typed keys must contain every input that changes the returned result shape;
- simple parent-child loaders may key only by parent ID;
- argument-bearing loaders require composite typed keys (parent ID plus a
  typed, normalized representation of all result-shaping arguments) or an
  equivalently explicit per-shape loader design;
- the retired A2 central store tuple
  `(response scope, loader identity, normalized load shape, parent key)` must
  **not** be reproduced; there is no cross-field central result store.

### 3.5 Loader-first rule

The following `ADR-0007` section 4.5 rule is binding and is quoted verbatim:

> A loader-backed resolver MUST register its DataLoader key before performing
> unrelated awaited work. Any resolver that cannot obey this loader-first rule
> MUST provide field-specific query-count evidence demonstrating that its
> actual execution shape remains bounded and set-based before that field is
> approved for DataLoader adoption.

The foundation must include test fixtures capable of proving this rule for
future consumers (section 10.4). Neither this specification nor the
implementation may claim that the DataLoader library globally guarantees one
or sublinear dispatch count under arbitrary resolver scheduling; the
loader-first rule exists precisely because it does not.

### 3.6 Explicit DataLoader configuration

Production loader construction must configure explicitly, never relying on
crate defaults:

- maximum batch size: `200`;
- yield count / dispatch wait: `10`.

These are the values exercised by the independently reviewed B0 evidence. Any
future change to either value is architecture-sensitive and requires focused
batching/query-count evidence before merge (`ADR-0007` section 4.6).

### 3.7 Load API, batch totality and test-only consumers

- Thoth database DataLoaders must use `try_load`. `Loader::load()` is
  **prohibited** for production Thoth database loaders because a missing map
  result can panic; the architecture must fail closed through ordinary GraphQL
  error handling (`ADR-0007` section 4.8);
- every batch function must be **total** over its requested keys:
  - a valid relationship with no rows returns a successful empty value for
    that key;
  - a valid populated relationship returns the correct value;
  - a batch-wide backend error returns an error value for **every** requested
    key;
  - a disappearing/missing requested map key fails closed through `try_load`;
  - no fabricated empty success for a backend failure, ever;
- the foundation proves itself through **test-only representative loader
  consumers** (a test-only schema/fixture in the spirit of the current
  `batching_fixture.rs`, rebuilt for DataLoader): a simple parent-keyed loader
  over real Diesel data, and fixtures sufficient for the section 10 evidence
  matrix. No production resolver adopts a loader in this task.

### 3.8 Async GraphQL execution/test migration

Production already executes async; this task introduces **no new production
execution model**. The test migration scope is:

1. migrate the general GraphQL unit-test execution path in
   `thoth-api/src/graphql/tests.rs` from `juniper::execute_sync` to Juniper
   async execution. Prefer one bounded central bridge/helper — the existing
   central helper `execute_graphql` (`tests.rs:44`) is the natural seam — so
   the ~7 `execute_sync` call sites migrate without broad caller rewrites.
   The direct call sites at `tests.rs:2843`, `:2872`, `:2912`, `:3119`,
   `:3796` and `:3827` either route through the bridge or migrate
   individually; schema-validation-only call sites (which never execute
   resolvers) may use whichever form is simplest, but the supported general
   path is async;
2. if the bridge must be callable from synchronous `#[test]` functions, it
   must implement safe runtime-boundary behaviour: construct or reuse a
   runtime explicitly, and **fail explicitly** (not deadlock, not silently
   misbehave) if invoked from inside an already-running Tokio runtime —
   e.g. by detecting `tokio::runtime::Handle::try_current()` and panicking
   with a clear message, or by being compiled such that misuse cannot occur.
   Section 10.1 requires a test for this;
3. a synchronous compatibility requirement must **NOT** remain for new
   loader-backed fields;
4. the three test-code populations must be treated differently:
   - **general GraphQL tests** (`tests.rs`): migrate to async execution via
     the bridge;
   - **A2-specific tests** (`batching_tests.rs` modules that prove the
     superseded store/prefetch/scope architecture): removed with that
     architecture (section 7);
   - **mutation/guard regression tests**: preserved and rehosted onto
     independent fixtures (section 8); they must not be deleted merely
     because they currently live beside A2 fixtures;
5. the async integration harness in `thoth-api/tests/support/mod.rs` is
   already correct and needs no execution-model change.

## 4. Non-goals

The task must not:

1. implement `BE-02` or any part of it;
2. implement `Publisher.distributionPlatforms`;
3. implement any Thoth Metrics production adoption or consumer conversion;
4. adopt the loader in **any** production GraphQL field;
5. upgrade Juniper or `juniper_codegen`;
6. migrate to `async-graphql`;
7. create any database migration, data migration or schema change;
8. change the public GraphQL schema/SDL in any way;
9. deploy anything, dispatch any workflow, or touch production configuration;
10. access production secrets, production databases, protected deployment
    sources, or fleet/orchestrator queries;
11. activate `OBSERVE` or `ENFORCE`, or change the production guard mode from
    `OFF`;
12. claim the duplicate top-level mutation execution defect is fixed;
13. remediate, merge, close or otherwise touch PR
    [#799](https://github.com/thoth-pub/thoth/pull/799);
14. resume `THOTH-GQL-OPS-03` or implement `THOTH-GQL-OPS-04`;
15. redesign the mutation-guard architecture or decide its eventual
    keep/simplify/replace/retire disposition (`ADR-0007` section 7.4);
16. perform unrelated error-contract normalization (in particular, no
    repository-wide move from `.map_err(Into::into)` to
    `ThothError::into_field_error`);
17. perform unrelated authorization changes;
18. perform unrelated refactors.

## 5. Invariants

The implementation must prove all `ADR-0007` binding invariants
(`ADR-0007` section 5). At minimum:

1. every production loader belongs to exactly one GraphQL request;
2. no loader or loader-result cache crosses request boundaries;
3. production child loaders use `non_cached::Loader` and `try_load` only;
4. batch functions are total over requested keys;
5. the loader-first scheduling rule (section 3.5) holds and is testable;
6. batching configuration is explicit: max batch size `200`, yield count `10`;
7. no per-parent SQL loop is hidden inside a batch function;
8. no Diesel connection is held across an `.await`;
9. a batch-wide backend failure fails closed — never successful empty data,
   never per-key fallback/retry SQL;
10. existing fields' actual current GraphQL-visible error conventions are
    preserved (section 6.8);
11. result membership, ordering, pagination and authorization remain
    equivalent to each field's approved direct contract, where applicable;
12. completed loader results are not reused after return; read-after-write
    freshness requires no invalidation;
13. loader availability is independent of mutation guard mode;
14. the duplicate-mutation-execution defect is not declared solved;
15. the foundation leaves production SDL byte-identical;
16. the first future production adopter still requires field-specific
    query-count and failure evidence under its own task;
17. the implementation PR is not production activation authorization.

## 6. Required behaviour

### 6.1 Loader construction

The request-local loader bundle is constructed with the `Context`, empty of
completed state, configured explicitly per section 3.6, and dropped with the
`Context`. Construction must be reachable from the real production `Context`
path (`thoth-api-server/src/lib.rs` handler), not only from tests.

### 6.2 Batching

For an immediately ready sibling set of N keys under one fixed shape, the
intended bound is chunked set-based execution with statement count
proportional to `ceil(N / 200)`. This bound is asserted only for loader-first
resolver shapes, never for arbitrary pre-loader async scheduling.

### 6.3 Non-caching and freshness

`non_cached::Loader` semantics are required behaviour: sequential
`load(key) -> complete -> load(key)` re-enters the source; concurrent pending
requests for the same key may coalesce (that is batching, not caching); within
one request, `load -> write -> load` observes the changed value with no manual
invalidation, no store clear, no guard mode and no scope machinery.

### 6.4 Request isolation

Two concurrent request `Context`s with the same logical key share no loader
state, no pending batch and no completed results.

### 6.5 Failure behaviour

A batch-wide backend failure surfaces at the owning child-field path with the
field's correct null propagation; every requested key in the failed batch
receives an error; no fallback or retry SQL runs; no panic occurs; no key
receives a fabricated successful empty result. A deliberately missing
requested key in a returned batch map fails closed through `try_load` as a
GraphQL field error, without panic.

### 6.6 Authorization

DataLoader is not an authorization layer and must not weaken one. The
foundation must not change any authorization behaviour. Where representative
test fixtures exercise protected paths, positive and negative authorization
tests are required (`thoth-api/AGENTS.md` section 7). Future protected
production loaders require field-specific authorization evidence under their
own adopting task.

### 6.7 Blocking Diesel boundary

Diesel remains synchronous. The initial approved blocking boundary is
`tokio::task::spawn_blocking`. At the base commit no existing repository
abstraction provides an equivalent bounded blocking bridge (the workspace has
no `spawn_blocking` wrapper and no async pool); if implementation-time
inspection reveals one that is demonstrably equivalent and more appropriate,
the implementation may use it and must record the equivalence argument.

Binding requirements for every batch function:

1. clone `Arc<PgPool>` (the shareable pool handle from `thoth-api/src/db.rs`)
   into the blocking closure;
2. clone immutable key/input data into the closure;
3. acquire the pooled Diesel connection **inside** the blocking closure;
4. execute the complete synchronous Diesel work inside the closure;
5. drop the connection before the closure returns;
6. never hold a Diesel connection across an `.await`;
7. normally, one dispatch chunk plus one load shape maps to **one** set-based
   SQL statement (`WHERE key = ANY($1)` / `.eq_any(keys)` shape);
8. no per-key SQL loop hidden inside the batch function.

If a loader genuinely needs more than one fixed statement per chunk, the
reason and the fixed statement bound must be explicit in code and proven by a
test; N statements for N parents are non-compliant.

### 6.8 Error representation

The foundation must preserve `ADR-0007`'s corrected error contract
(section 4.9):

1. do **not** assume all current GraphQL errors carry `extensions.type`; the
   generic `.map_err(Into::into)` path emits no `extensions` object, and
   `ThothError::into_field_error` is not the universal current production
   path;
2. for existing converted resolver fixtures, preserve each fixture's actual
   current conversion convention;
3. provide a safe, non-panicking, shareable production batch-error
   representation — an error projection/snapshot type (or `Arc`-shared
   equivalent) that preserves the GraphQL-visible fields required below;
4. the spike's serialization/deserialization `clone_thoth_error` technique is
   **explicitly prohibited** as production plumbing: the production solution
   must not use JSON/serde round-tripping (`ThothError::to_json` /
   `from_json` or equivalent) merely to clone or share errors;
5. failure tests must compare GraphQL-visible behaviour between the direct
   and loader paths: data/null propagation; field path; location where stable
   and applicable; message; and extensions convention (section 10.10);
6. no per-parent fallback/retry SQL after a batch-wide failure.

Any repository-wide error-convention standardization is a separate externally
observable decision and is out of scope (non-goal 16).

## 7. A2 retirement scope

The following current source surfaces are removed or simplified. For each, the
reason it is no longer required under `ADR-0007` is stated. Do not delete code
merely because its name sounds A2-related; the guard boundary in section 8
lists what must survive.

### 7.1 Removed: `thoth-api/src/graphql/batching.rs` (entire module)

Contains `LoaderIdentity`, `LoadShapeKey`, `StoredParentKey`, `ScopeKey`,
`StoreKey`, `DispatchFailureKey`, `SharedErrorType`, `SharedLoadError`,
`StoredEntry`, `BatchLookup`, `DispatchResult`, the `BatchLoader` trait, and
`GraphqlBatchStore` (with `new`, `is_available`, `lookup`, `dispatch`,
`record_failure`, `invalidate_all`, `failure_count`, `entry_count`).

Why: `ADR-0007` section 6 supersedes the central request-scoped stored-result
store, the store-level load-shape/result state machinery and the
`(scope, loader, shape, key)` identity outright. Typed per-loader DataLoader
keys replace the central tuple; non-cached loaders have no stored results to
partition, no availability mode, and no failure-record store. The
`SharedLoadError` concept is superseded by the new production error projection
of section 6.8 (which must be designed for DataLoader batch sharing, not
carried over as-is).

### 7.2 Removed: `thoth-api/src/graphql/prefetch.rs` (entire module)

Contains `PrefetchTarget`, `prefetch(..)` and `collect_terminal_selections`.

Why: A2's look-ahead-driven prefetch-site model is superseded
(`ADR-0007` section 6: "A2 look-ahead-driven prefetch as the shared N+1
mechanism" and "all-material-path look-ahead prefetch-site coverage as the
generic adoption model"). DataLoader batching arises from deferred dispatch of
sibling `try_load` calls; no look-ahead and no prefetch sites exist.

### 7.3 Removed: `thoth-api/src/graphql/scope.rs` (entire module)

Contains the pinned-Juniper response-scope compatibility shim
`top_level_response_key(..)` and its inline probe tests.

Why: top-level-response-key partitioning existed only to make stored results
coherent across mutation payloads. Non-cached loaders retain no completed
results, so there is nothing to partition; `ADR-0007` section 6 retires the
response-scope shim explicitly. Its removal also removes a pinned-framework
hidden-API surface (`Executor::new_error` / `ExecutionError::path`
scaffolding).

### 7.4 Simplified: `Context` in `thoth-api/src/graphql/model.rs`

Remove the `pub(crate) batch_store: GraphqlBatchStore` field and the
guard-mode-derived store construction. Section 8 governs what happens to the
`with_guard_mode` constructor signature.

Why: the store is retired (7.1); the replacement request-local loader bundle
(section 3.3) is owned here instead, with availability independent of guard
mode (`ADR-0007` invariant 13).

### 7.5 Removed or split: A2-specific tests in `thoth-api/src/graphql/batching_tests.rs`

The modules whose purpose is wholly to prove the superseded architecture are
removed with it: `store_state`, `collision_matrix`, `traversal`,
`traversal_unit`, `integration`, `error_contract`, `statement_counts`,
`configuration`, `execution_parity`, and the store/scope-specific parts of
`mutation_behaviour` (its read-after-write and scope-isolation tests prove
store coherence machinery that no longer exists; the *property*
read-after-write freshness is re-proven for DataLoader by section 10.7).

The `query_path` module is **mixed-purpose** and must be split, not removed
wholesale:

- **preserve/rehost** — its A2-independent mutation-guard/query-path
  regressions, which prove guard behaviour on the query operation path and do
  not depend on the store, scope or prefetch machinery. At minimum the
  implementation must preserve tests equivalent to:
  1. a valid query operation is never restricted by the mutation guard and
     emits no guard event in any mode
     (`a_valid_query_is_never_restricted_and_emits_no_event_in_any_mode`);
  2. a valid query response remains equivalent to the no-guard baseline
     across guard modes
     (`a_valid_query_response_is_byte_identical_across_every_mode`);
  3. a baseline-invalid query preserves Juniper's canonical response/error
     behaviour and emits no guard event
     (`an_invalid_query_keeps_juniper_canonical_error_and_produces_no_guard_event`).

  These are mutation-guard/query-path regressions, not A2 batching tests.
  They are rehosted onto A2-independent fixtures with the other preserved
  guard tests (section 8.3);
- **retire** — its A2 store/scope-specific coverage: the test asserting
  shared `batch_store` entry counts, a shared top-level response scope and no
  additional A2 statement due to stored-result reuse
  (`a_query_with_a_duplicate_response_key_shares_one_scope_and_adds_no_statement`)
  proves retired store/scope machinery and is removed with it. Obsolete A2
  assertions must not be preserved merely because they currently share a Rust
  module with guard tests.

Why: `ADR-0007` section 4.1 permits removing tests whose purpose is
specifically to exercise retired A2/store/scope behaviour. The properties
worth keeping (set-based statement counts, failure semantics, isolation,
freshness) are re-established for the DataLoader foundation by the section 10
evidence matrix, not silently dropped — and guard regressions that merely
co-reside with A2 fixtures are rehosted, never dropped.

The modules that are **not** A2-specific — `guard_tests`, `baseline_matrix`,
`directives` — are mutation-guard regression coverage and are preserved by
rehosting, together with the preserved A2-independent `query_path`
regressions above (section 8.3).

### 7.6 Retired/rehosted: `thoth-api/src/graphql/batching_fixture.rs`

The A2 proof fixture (`TestImprintLoader`, `TestImprintDescendingLoader`,
`TestImprintShape`, the `BatchLoader` impls, the test-only
query/mutation roots wired to the store, and the store-specific counters) is
removed with the architecture it proves.

Two pieces of test machinery are **concepts to rehost**, not delete:

1. the Diesel `SqlProbe` statement-capture harness
   (`set_default_instrumentation` + a dedicated measured pool constructed
   after hook installation) — this is the repository's existing trustworthy
   connection-instrumentation mechanism and is exactly what section 10.9
   requires for real query-count evidence. Rehost it into the new DataLoader
   test fixture;
2. the mutation-execution counter pattern (`MUTATION_RESOLVER_CALLS` /
   `mutation_resolver_calls()`) and a minimal test-only mutation schema —
   required by the rehosted duplicate-mutation regression tests
   (section 8.3), independent of any store.

### 7.7 Simplified: `thoth-api/src/graphql/mod.rs`

Remove the `mod batching; mod batching_fixture; mod batching_tests; mod
prefetch; mod scope;` declarations with their modules. The guard surface
(`mod mutation_guard`, `pub use mutation_guard::MutationGuardMode`,
`run_mutation_guard`) is preserved (section 8.1).

### 7.8 Simplified: `test_context_with_guard_mode` in `thoth-api/src/model/tests.rs`

This helper exists so batching tests can exercise store availability derived
from guard mode. With the coupling removed it is either deleted or reduced to
whatever the rehosted guard tests actually need (section 8.3). Its ADR-0006
"invariant 30" comment goes with it.

## 8. Mutation guard boundary

The mutation concern remains a separate, live GraphQL execution concern
(`ADR-0007` sections 4.13 and 7.4). This task may remove only the
batching-specific coupling to the guard.

### 8.1 Preserved guard surfaces

The following current surfaces must survive with activation state unchanged:

- the independent mutation-guard mechanism in
  `thoth-api/src/graphql/mutation_guard.rs`: the `MutationGuardMode`
  enum (`Off`, `Observe`, `Enforce`) with default `OFF` and its
  parsing/`FromStr` behaviour; guard evaluation (`evaluate`); the baseline
  eligibility gate; duplicate-response-key detection; `GuardDecision`;
  `GuardOutcome`; event construction/emission (`emit_event`);
  `collision_positions`; and `rejection_response`. `mutation_guard.rs` is
  **not** preserved in full: its single A2-only coupling API,
  `MutationGuardMode::store_available()`, and the coupling documentation
  around it are removed under section 8.2. `MutationGuardMode` remains
  because the mutation guard remains; `store_available()` does not remain
  because the A2 store does not;
- `run_mutation_guard(..)` in `thoth-api/src/graphql/mod.rs` and its call at
  the request boundary in `thoth-api-server/src/lib.rs` (`graphql` handler),
  evaluated before `GraphQLRequest::execute`;
- the guard-mode wiring: the `MutationGuardMode` parameter through
  `start_server(..)`, the Actix `Data<MutationGuardMode>` registration, and
  the CLI/environment mode resolution and its tests in `src/bin/thoth.rs`;
- production guard mode remains `OFF`; no `OFF -> OBSERVE` or
  `OBSERVE -> ENFORCE` transition occurs or is implied.

### 8.2 Removed guard coupling

- the implementation **must remove** `MutationGuardMode::store_available()`
  (`thoth-api/src/graphql/mutation_guard.rs:142`). Its only purpose is to
  derive A2 store availability from guard mode — the superseded `ADR-0006`
  invariant 30 coupling that `ADR-0007` explicitly retires. With the A2
  store removed (section 7.1), the method answers a question that no longer
  exists;
- the implementation must also remove all documentation/comments whose only
  purpose is to describe that coupling: the `MutationGuardMode` enum and
  method doc comments presenting guard mode as the sole switch controlling
  loader/store availability and `ENFORCE` as the prerequisite for
  batching/store availability (`mutation_guard.rs:110-144`), and the
  equivalent `ADR-0006` invariant 30 coupling comments at
  `thoth-api/src/graphql/model.rs:105` and
  `thoth-api/src/model/tests.rs:154` (section 7.8). Assertions on
  `store_available()` inside otherwise-preserved test surfaces — the CLI
  mode-resolution tests in `src/bin/thoth.rs:337-339` — are removed with the
  method while the mode-resolution tests themselves are preserved;
- this is **not** a guard redesign. It is the deletion of superseded
  batching-specific coupling required by `ADR-0007`. The guard mechanism,
  its evaluation semantics and its activation state are untouched:
  production guard mode remains `OFF`, no guard activation occurs, and the
  duplicate-mutation concern remains unresolved and separately controlled;
- `Context` no longer derives any batching availability from guard mode;
  `GraphqlBatchStore::new(mode)` disappears with the store. The
  `Context::with_guard_mode` constructor loses its reason to exist: the
  expected simplification is to restore the four-argument `Context::new(..)`
  as the single constructor and delete `with_guard_mode`, updating its call
  sites (`thoth-api-server/src/lib.rs`, `thoth-api/src/model/tests.rs`). The
  guard keeps receiving its mode at the request boundary
  (`run_mutation_guard`), which never depended on `Context`;
- DataLoader availability is independent of guard mode: loaders are
  unconditionally available on their approved resolver paths, regardless of
  `OFF` / `OBSERVE` / `ENFORCE` (`ADR-0007` invariant 13).

### 8.3 Rehosted regression evidence

The guard and duplicate-mutation regression tests currently live inside
`batching_tests.rs` and depend on A2 fixtures (the fixture mutation schema
`TestMutationRoot::add_imprint` and the `mutation_resolver_calls()` counters).
They must be rehosted onto independent fixtures — a dedicated test module
(e.g. `mutation_guard_tests.rs` or inline `#[cfg(test)]` tests) with a minimal
counter-instrumented test mutation schema that has no store, no loader and no
A2 dependency:

- `guard_tests` (direct/named-fragment/inline-fragment duplicate rejection in
  `ENFORCE`; `OFF`/`OBSERVE` non-rejection; query operations unaffected;
  non-top-level duplicates unaffected; operation selection; rejection
  response/positions; event emission);
- `baseline_matrix` (baseline-invalid requests yield no guard decision, no
  event, byte-identical responses to the no-guard baseline);
- `directives` (the `assert_against_juniper` cross-check comparing the guard's
  verdict against Juniper's **actual observed** mutation resolver execution
  count, across `@skip`/`@include` literal, variable, defaulted and overridden
  forms);
- the preserved A2-independent `query_path` regressions (section 7.5): a valid
  query operation is never restricted by the mutation guard and emits no guard
  event in any mode; a valid query response remains equivalent to the no-guard
  baseline across guard modes; a baseline-invalid query preserves Juniper's
  canonical response/error behaviour and emits no guard event;
- independent duplicate-mutation-execution evidence: tests proving that
  pinned Juniper 0.16.2 still executes a compatible repeated top-level
  mutation response key once per occurrence under **async** execution while
  merging results under one response key. This coverage must survive A2
  removal and must not be weakened.

This task must not claim the duplicate-execution defect is fixed, must not
redesign the guard, and must not decide its eventual disposition. The final
state leaves the mutation guard as a separately controlled mechanism with
activation state unchanged.

## 9. Production-field boundary

This task adopts **no** production GraphQL child field:

- no `Publisher.distributionPlatforms`;
- no Thoth Metrics consumer conversion;
- no change to any existing production resolver's data access.

Test-only representative loader consumers are allowed and expected
(section 3.7). The foundation must preserve byte-identical production SDL
(section 10.13).

## 10. Required tests and acceptance evidence

The implementation is acceptable only with all of the following, as real
recorded evidence. Missing evidence is missing work.

### 10.1 Async execution

- a genuine async child resolver executes correctly on pinned Juniper 0.16.2;
- the general GraphQL test helper executes async Juniper correctly, and the
  migrated `tests.rs` suite passes through it;
- the central bridge/harness behaves correctly from each of its supported
  calling contexts;
- invalid nested-runtime use (calling the sync-callable bridge from inside a
  running Tokio runtime), if that misuse is constructible, fails explicitly
  with a clear error rather than deadlocking or behaving ambiguously.

### 10.2 Batch boundaries

For independent sibling `try_load` calls under one shape with explicit
`200`/`10` configuration, test at least N = 1, 100, 200, 201, 500, asserting
the immediately-ready chunk shapes:

| N | Expected dispatch chunks |
|---:|---|
| 1 | `[1]` |
| 100 | `[100]` |
| 200 | `[200]` |
| 201 | `[200, 1]` |
| 500 | `[200, 200, 100]` |

Run representative batching tests on **both** the current-thread and the
multi-thread Tokio runtime. Observed immediate-sibling shapes must not be
asserted (in test names, comments or docs) as a universal
arbitrary-scheduling guarantee.

### 10.3 Configuration

Tests prove the explicit max-batch-size `200` and yield-count `10`
configuration is what production construction sets — not crate defaults.

### 10.4 Scheduling

Explicit scheduling fixtures, at minimum:

- immediate loader call (loader-first shape);
- a benign `yield_now()` before the load;
- a delayed cohort demonstrating that dispatch fragmentation **can** occur
  (documenting why loader-first is binding);
- a loader-behind-loader / upstream-loader scenario.

The suite must be usable to enforce/review the loader-first adoption rule for
future consumers.

### 10.5 Request isolation

At least two independent request `Context`s/loaders with the same logical key,
proving no state, pending batch or completed result crosses requests.

### 10.6 Non-caching

Sequential `load(key) -> complete -> load(key)` invokes the source again.
Concurrent pending loads of one key may coalesce; after completion, a later
load refetches.

### 10.7 Read-write-read freshness

Within one request: `load -> write/change source -> load` observes the changed
value without any invalidation call.

### 10.8 Missing-key failure

A deliberately defective batch function that omits a requested key fails
closed through `try_load` as a GraphQL error: no panic, no fabricated result.

### 10.9 Real-Diesel query-count evidence

At least one representative real-Diesel loader fixture, with actual statement
count demonstrated by connection instrumentation (the rehosted `SqlProbe`
mechanism of section 7.6, or an equivalently trustworthy external observation
point). Counters that merely increment inside the code under test are not
acceptable as sole proof of SQL statement count.

Evidence must cross a configured batch boundary: for an N > 200 (e.g. N=250
where the fixture shape allows), demonstrate chunked set-based behaviour —
two dispatch chunks and exactly two set-based SQL statements
(`WHERE ... = ANY($1)` shape), not N statements.

### 10.10 Backend failure

One representative loader failure fixture proving, against the corresponding
direct path:

- the GraphQL child field becomes null/error according to current semantics;
- the field error path is correct;
- message and extensions convention match the corresponding direct path
  (respecting whichever conversion convention that fixture actually uses —
  generic `Display` or `into_field_error`);
- location matches where stable/applicable;
- no panic; no successful empty substitution; no per-key fallback SQL; no
  retry SQL.

### 10.11 Error representation

Tests prove the production shareable error type/projection preserves the
required GraphQL-visible semantics, and the implementation contains no serde
round-trip error cloning (review-verifiable; a lint/grep-level check for
`to_json`/`from_json` in the loader error path is sufficient evidence).

### 10.12 Diesel ownership

Evidence/tests sufficient to show: the pool handle may cross the async
boundary; the acquired connection does not; connection acquisition occurs
inside `spawn_blocking`; the synchronous query completes before the closure
returns. (Compile-time structure plus a focused test/review argument is
acceptable; the property must be explicit, not incidental.)

### 10.13 SDL

Generate the production SDL (`create_schema().as_sdl()`) at the reconciled
implementation base and at the implementation head, and prove byte-identical
output. Record byte length and SHA-256 of both. (For reference, `ADR-0007`
section 10.8 recorded 160,799 bytes at its verification base; the
implementation must measure its own base rather than assuming this value.)

### 10.14 Mutation regression

The rehosted duplicate-mutation and guard regression suites of section 8.3
pass, independent of any A2 fixture, and demonstrably preserve the
pinned-Juniper duplicate top-level mutation response-key execution behaviour
under async execution. No test, comment or report claims this task fixes it.

### 10.15 Repository validation gate

All of, with real recorded output:

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test -p thoth-api --features backend
cargo test --workspace
```

plus every focused DataLoader/batching/error/mutation test above. If the
repository-required commands have changed by implementation time, follow the
current repository-required equivalents and record the reconciliation.

## 11. Data and migration requirements

Database migration: **NONE**
Data migration: **NONE**
GraphQL schema migration: **NONE**

This is a code-internal architecture migration only. The required code
migration sequence is additive-first:

1. add the dependency/runtime foundation (section 3.2);
2. add the request-local loader bundle to `Context` (section 3.3);
3. add the safe error projection (section 6.8);
4. add the async execution/test infrastructure (section 3.8);
5. prove B0 using test-only consumers (section 10);
6. rehost the independent mutation/guard tests (section 8.3);
7. remove the superseded A2 store/prefetch/scope infrastructure (section 7);
8. prove byte-identical production SDL and complete the full validation gate.

No production child field adopts the loader at any step.

## 12. Observability and operations

### 12.1 Required production logs

**NONE.** No new production DataLoader/batching logs are required by this
foundation because:

- no production GraphQL field adopts the loader;
- there is no production/client cutover;
- the task must introduce no production diagnostic dependency.

Test instrumentation used to prove batching/query counts (the rehosted
`SqlProbe` mechanism of section 7.6, the section 10.9 statement-count
evidence) is test evidence, not production observability. The implementation
must not add permanent production logs merely to satisfy the specification
template.

### 12.2 Required metrics/alerts

**NONE.** No new production metric or alert is required because no production
resolver consumes the DataLoader foundation in this task. Future production
adoption tasks (`BE-02`, Thoth Metrics, or any later consumer) must define
their own monitoring requirements under their own approved specifications.

### 12.3 Operational runbook changes

**NONE.** No operational runbook change is required because this task:

- deploys nothing;
- activates nothing;
- changes no guard mode;
- creates no new operator action;
- changes no request-acceptance policy;
- changes no public API.

This task does not close or modify any mutation-guard operational gate; the
guard's operational disposition remains a separate concern
(`ADR-0007` section 7.4).

### 12.4 Operational effects

- Deployment: **NONE**
- Production activation: **NONE**
- Runtime configuration change: **NONE**
- Guard mode change: **NONE**
- GraphQL request-acceptance change: **NONE**
- Database migration: **NONE**
- Data migration: **NONE**
- GraphQL schema migration: **NONE**

## 13. Rollout and rollback

### 13.1 Rollout

Foundation rollout has no public/client cutover. The implementation PR itself
must not: deploy; activate a feature; change GraphQL SDL; change request
acceptance; change database schema; change guard mode. No feature flag is
required merely for an internal foundation with no production consumer.

Future production consumers follow their own approved programme rollout
requirements. `BE-02` still needs its own freshly reconciled approved
task/specification after this foundation is merged; nothing in this task
pre-authorizes it.

### 13.2 Rollback

Before any production consumer depends on the foundation, rollback is one
bounded revert of the `THOTH-GQL-DATALOADER-01` implementation PR. Rollback
must not reactivate or imply `OBSERVE`/`ENFORCE`, and must not imply that
`ADR-0006` becomes architecturally authoritative again — reversing the
architecture itself requires another ADR.

This specification deliberately prescribes no rollback model for later
production fields; each adopting task defines its own, and rollback to an N+1
implementation is not an available option under the standing control
(`ADR-0007` section 13).

## 14. Stop conditions

The implementing agent must stop and return `BLOCKED` if:

- `ADR-0007` is no longer `APPROVED`/repository-authoritative;
- base movement materially invalidates this specification (section 3.1);
- the resolved versions of Juniper, `dataloader`, Tokio or Diesel differ in an
  architecture-relevant way and the `ADR-0007` section 4.14 revalidation has
  not been run;
- the A2 infrastructure has already materially changed and the section 7
  removal plan no longer matches source;
- removing A2 would require deleting independent mutation
  protection/evidence (section 8.3);
- production SDL cannot remain byte-identical;
- batching cannot meet the required set-based/query-count evidence
  (section 10.9);
- the implementation would require a production field to prove the foundation;
- a safe non-panicking shared error representation cannot preserve the
  required GraphQL-visible semantics (section 6.8);
- production or protected infrastructure access would be required;
- any necessary implementation choice would change approved `ADR-0007`
  architecture;
- approved architecture would need to change;
- required production information or secrets are unavailable;
- scope cannot be completed without unrelated changes.

## 15. Expected implementation report

The agent must use
`docs/engineering/ai-delivery/implementation-report-template.md`, including
exact base and head commits, the resolved `dataloader`/`tokio` versions, the
complete evidence of section 10 with real command output, and the SDL byte
length/SHA-256 comparison.

## 16. Recommended execution

Implementation model: high-reasoning agent
Reasoning level: high
Independent reviewer: separate high-reasoning agent/model, reviewing the exact
PR head
Review reasoning level: high

The implementing agent must not approve its own work. Because risk is HIGH,
merge requires fresh independent exact-head review **and** separate explicit
CTO merge authorization; neither is granted by this specification.

## 17. Branch and integration plan

- branch source: fresh, reconciled `develop`;
- one bounded implementation branch:
  `feature/shared-architecture/graphql-dataloader-foundation` (do **not**
  create it before implementation authorization);
- one bounded implementation PR targeting `develop`; suggested title:
  `THOTH-GQL-DATALOADER-01: add request-scoped GraphQL DataLoader foundation`;
- the PR must remain bounded to this foundation, carry the full section 10
  acceptance evidence, receive fresh independent exact-head review and
  separate explicit CTO merge authorization, and must not be merged by the
  implementing agent;
- expected merge order: after this specification is approved; before any
  `BE-02` adoption task;
- branch deletion after merge: YES
- final programme PR required: NO
- final release path: `develop -> master`

## 18. Approval

Approved for implementation by:
Date:
Notes:

Record only the durable implementation authorization here. Independent review
decisions, CTO merge authorization and the merge itself are terminal GitHub
evidence under `ADR-0005` and must not be copied back into this file.
