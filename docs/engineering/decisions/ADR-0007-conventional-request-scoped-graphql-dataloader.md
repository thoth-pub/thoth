# ADR-0007 - Conventional request-scoped GraphQL DataLoader and async resolver execution

Status: APPROVED
Date: 2026-08-11
Approved by: CTO
Approval date: 2026-08-11
Decision owner: CTO
Programmes affected: Shared Thoth GraphQL / Backend Architecture (owning programme); Publisher Services and Distribution Configuration; Thoth Metrics; any programme resolving `thoth-api` GraphQL child fields
Repositories affected: `thoth-pub/thoth`
Supersedes: `ADR-0006`
Superseded by: None

Decision: adopt conventional request-scoped, non-cached DataLoaders on the current pinned Juniper 0.16.x execution stack; make async GraphQL execution the supported execution model for loader-backed resolvers and general GraphQL test execution; retire ADR-0006's look-ahead-prefetch/store architecture and its batching-specific coupling to mutation-guard mode; retain the pinned-Juniper duplicate-top-level-mutation-execution finding as a separate GraphQL execution concern whose eventual protection mechanism is not decided or activated by this ADR.

Authority condition: this ADR is only a proposal while its status is `PROPOSED`. It becomes repository-authoritative only after explicit CTO approval, an `APPROVED` status, independent exact-head review, and merge of the exact approved content into `develop`. No implementation task may rely on this proposal before those conditions are satisfied.

Verification base: `develop` at `2bec75e6698232f7643862120e5437452fcfa252`. The independent B0 feasibility evidence was produced against and subsequently reviewed against this exact commit. At drafting time, live `develop` remained identical to that SHA.

---

## 1. Context

### 1.1 Standing N+1 control

`thoth-api/AGENTS.md` requires new GraphQL lists and reports to avoid N+1 access and use set-based SQL or batched loaders. That control remains unchanged.

Publisher Services `BE-02` first exposed the shared architecture gap through the required `Publisher.distributionPlatforms` child field. Thoth Metrics independently requires reusable entity-centric GraphQL child loading without per-parent database statements. The solution therefore belongs to Shared Thoth GraphQL / Backend Architecture rather than either programme alone.

### 1.2 What ADR-0006 decided

ADR-0006 selected variant A2: look-ahead-driven set-based prefetch into a request-scoped store. The store was partitioned by top-level GraphQL response key and coordinated with a central mutation request guard. The architecture was chosen after rejecting a conventional deferred-dispatch DataLoader under the then-binding requirement that the same resolver surface continue to work under both Juniper's synchronous and asynchronous execution paths.

The merged foundation carries a `GraphqlBatchStore` on `Context`, but deliberately adopts no production GraphQL field. In the merged state, guard mode is `OFF` and the store is unavailable. Production GraphQL execution itself is already asynchronous through `GraphQLRequest::execute(...).await`.

ADR-0006 also established a separate pinned-Juniper defect: compatible repeated top-level mutation selections sharing one response key can pass validation, execute the mutation resolver more than once, and merge the results under one response key. That can duplicate writes independently of any batching mechanism.

### 1.3 Why the decision is being revisited

ADR-0006's A2 architecture introduced substantial machinery to preserve the synchronous test path and to make stored results coherent across mutation payloads:

- look-ahead-driven prefetch sites;
- request-scoped stored results;
- loader identity and normalized load-shape namespaces;
- top-level-response-key execution scopes;
- a pinned-Juniper response-scope compatibility shim;
- mutation-guard/store availability coupling;
- `OFF` / `OBSERVE` / `ENFORCE` production-control lifecycle;
- operational fleet-verification work required before activation.

The operational path exposed further complexity. In particular, the current fleet-verification candidate in PR #799 is `BLOCKED` and remains draft and unmerged. That fact does not itself make ADR-0006 incorrect, but it is evidence of the cost of coupling batching correctness to mutation-guard activation.

The CTO therefore authorized a bounded reconsideration of conventional DataLoader architecture while keeping the current Juniper major/minor line. Two disposable B0 spikes were run and independently reviewed. The second spike supplied the actual patch, source, raw logs and manifests for inspection.

### 1.4 B0 evidence now established

The independently reviewed B0 evidence establishes all of the following on the current pinned stack:

1. Juniper 0.16.2 executes genuine async child resolvers correctly without a Juniper upgrade.
2. The general GraphQL unit-test surface can execute through async Juniper with a bounded central bridge and zero caller rewrites in the measured suite.
3. A conventional `dataloader::non_cached::Loader` can coalesce independent sibling `try_load(parent_key).await` calls without parent-side key collection, `load_many`, look-ahead or prefetch.
4. With `max_batch_size = 200`, immediately ready sibling calls produced exactly one batch through N=200, `[200, 1]` at N=201 and `[200, 200, 100]` at N=500, stable across current-thread and multi-thread Tokio runtimes in the repeated spike runs.
5. A 250-parent real-Diesel fixture produced two DataLoader dispatches `[200, 50]` and exactly two connection-instrumented set-based `WHERE ... = ANY($1)` statements.
6. Synchronous Diesel can be bounded behind `tokio::task::spawn_blocking`, with the pooled connection acquired and dropped entirely inside the blocking closure and no connection living across `.await`.
7. Loader state can be owned by one real GraphQL `Context` and isolated across concurrent requests.
8. `non_cached::Loader` does not retain completed key results. Sequential load -> write -> load re-fetched the source and observed the new value without invalidation.
9. Database failure can fail closed through `try_load`, preserve the child-field GraphQL path/null propagation of the corresponding direct path, avoid fabricated empty success and issue no fallback retry SQL.
10. The production SDL was byte-identical before and with the disposable B0 foundation fixture.
11. Backend and workspace validation passed in the spike evidence, subject to the evidence-note in section 10.8.
12. Pinned Juniper still executes a compatible repeated top-level mutation response key more than once under async execution. B0 does not remove that independent execution defect.

The independent review therefore classified B0 as `FEASIBLE` and recommended drafting this superseding ADR.

---

## 2. Decision drivers

1. Satisfy the repository's standing N+1 control without waiving it.
2. Keep database statement growth set-based and explicitly bounded for approved resolver shapes.
3. Keep all loader state request-local; never share authorization-sensitive state across requests.
4. Preserve GraphQL-visible correctness, authorization and error semantics.
5. Preserve read-after-write freshness without cache invalidation or a batching-specific mutation guard.
6. Stay on the current Juniper 0.16.x line unless evidence proves an upgrade necessary.
7. Minimize special execution machinery and pinned-framework compatibility shims.
8. Keep production adoption additive, reversible and independently measurable.
9. Support Publisher Services and later Metrics through one reusable pattern.
10. Separate the N+1 solution from the independent duplicate-mutation-execution defect.
11. Avoid making crate scheduling heuristics into unproven correctness guarantees.
12. Keep dependency and maintenance risk bounded and explicit.

---

## 3. Options considered

### Option A - Keep ADR-0006 A2 unchanged

Retain look-ahead prefetch, stored results, response-key scopes and guard/store coupling.

Advantages:

- already approved and implemented as a foundation;
- deterministic parent-list prefetch does not depend on async scheduling;
- introduces no third-party DataLoader dependency.

Disadvantages:

- material custom framework machinery;
- stored-result coherence creates a coupling to the mutation guard;
- batching availability is blocked behind operational mode-control evidence even though the production API already executes asynchronously;
- each adoption requires prefetch-site coverage and load-shape/store integration;
- the execution-scope compatibility shim and guard eligibility path are pinned to internal/hidden Juniper behaviour;
- the operational path is demonstrably expensive, with PR #799 currently blocked.

Rejected because B0 now demonstrates a materially simpler architecture on the same Juniper line while preserving the required N+1 and correctness properties for approved resolver shapes.

### Option B - Current-Juniper async execution plus conventional non-cached DataLoader

Use Juniper 0.16.2, make async execution the supported path for loader-backed fields and general GraphQL tests, and use request-scoped `dataloader::non_cached::Loader` instances whose batch functions perform set-based Diesel work.

Chosen.

### Option C - Upgrade Juniper before adopting DataLoader

Not required by the evidence. B0 works on the current pinned Juniper. An upgrade would add dependency and compatibility scope without solving an observed blocker.

Rejected for this decision. Any future Juniper upgrade follows its own dependency-change review and the revalidation requirements in section 4.14.

### Option D - Migrate to async-graphql

Potentially offers different execution and DataLoader primitives but would be a broad GraphQL framework migration affecting schema macros, execution, errors, tests, generated contracts and downstream assumptions.

Rejected as disproportionate to the current problem.

### Option E - Implement bespoke set-based logic independently in each field

Would avoid a third-party loader dependency but duplicate request batching, scheduling, result partitioning and error handling across fields and programmes.

Rejected because it creates repeated local architecture rather than one reviewed shared mechanism.

---

## 4. Decision

### 4.1 Supported GraphQL execution model

Loader-backed resolvers are asynchronous and execute through Juniper's async execution path.

Production already uses `GraphQLRequest::execute(...).await`; this ADR does not introduce a new production execution model.

The general GraphQL unit-test harness must migrate from `execute_sync` to async Juniper through one bounded central test bridge or an equivalently small async harness. Existing general test callers should not be rewritten solely to adopt async execution when a central bridge is sufficient.

Synchronous execution is no longer a compatibility requirement for new loader-backed fields.

Tests whose purpose is specifically to exercise retired ADR-0006 A2/store/scope behaviour may be removed with that architecture. Independent mutation-execution/guard regression tests must be rehosted rather than deleted merely because they currently live beside A2 fixtures.

### 4.2 Loader implementation

Use `dataloader` 0.18.x, initially `0.18.0`, with default features disabled and Tokio runtime support enabled.

The production implementation uses `dataloader::non_cached::Loader` only. Completed key results must not be memoized across completed loads.

The dependency is resolved and pinned by `Cargo.lock`. A later dependency update must run the revalidation in section 4.14 before merge.

`tokio` becomes a direct normal dependency of `thoth-api` for the runtime primitives called by loader batch functions; test-only Tokio features may remain dev-only where possible.

### 4.3 Request ownership

Every production DataLoader instance is created for exactly one GraphQL request and owned directly or indirectly by that request's `Context`.

A loader must never be:

- `static`;
- process-global;
- shared through Actix application data across requests;
- reused by a later HTTP request;
- placed in any cache whose lifetime exceeds the GraphQL request.

Request-local loader ownership is a correctness and authorization invariant, not a performance preference.

### 4.4 Loader identity and typed keys

Each loader represents one reviewed logical field/query family.

Its key type must contain every input that can change the returned result. For a simple child relation this can be only the parent identifier. For an argument-bearing field, the key must include the parent identifier plus a typed, normalized representation of all result-shaping arguments, or the implementation must use an equivalently explicit per-shape loader design.

The central ADR-0006 store-level tuple:

`(response scope, loader identity, normalized load shape, parent key)`

is retired. It is replaced by ordinary typed loader construction and typed DataLoader keys. There is no cross-field central result store.

If one dispatch contains several load shapes, the batch function may partition keys by shape and issue one set-based statement per distinct shape. Statement growth may follow distinct approved shapes and configured chunks; it must not be implemented as a loop issuing one database statement per parent.

### 4.5 Loader-first scheduling invariant

This is binding:

> A loader-backed resolver MUST register its target DataLoader key before performing unrelated awaited work. Any resolver that cannot obey this loader-first rule MUST provide field-specific query-count evidence demonstrating that its actual execution shape remains bounded and set-based before that field is approved for DataLoader adoption.

Permitted before the target loader call only when explicitly justified by the field's evidence:

- async work intrinsic to the target DataLoader;
- an upstream loader dependency whose measured execution does not cause unacceptable fragmentation;
- other work specifically approved by the adopting task's query-count evidence.

Unrelated sleeps, network calls, external API calls or arbitrary asynchronous prerequisites must not be placed before the target `try_load` by default.

This rule exists because DataLoader batching depends on which requests are pending in the same dispatch opportunity. The library does not guarantee sublinear statement growth for arbitrarily separated arrival cohorts.

### 4.6 Batch-size and dispatch configuration

Production code must set the DataLoader batching configuration explicitly; it must not rely silently on crate defaults.

Initial reviewed values:

- maximum batch size: `200` keys;
- yield count / dispatch wait: `10`.

These are the values exercised by the independent B0 evidence. Changing either is an architecture-sensitive performance change and requires focused batching/query-count tests before merge.

For an immediately ready sibling set of N keys under one fixed shape, the intended bound is chunked set-based execution, with statement count proportional to `ceil(N / configured_max_batch_size)` rather than N.

That formula is not asserted for arbitrary pre-loader async scheduling. The loader-first invariant in section 4.5 is what constrains production resolver shapes.

### 4.7 Database boundary

Diesel remains synchronous. A loader batch function must not execute blocking Diesel work directly on an async executor thread.

The approved boundary is a bounded blocking bridge, initially `tokio::task::spawn_blocking`, with these invariants:

1. clone only shareable request dependencies into the blocking closure, such as `Arc<PgPool>` and immutable batch inputs;
2. acquire the pooled Diesel connection inside the blocking closure;
3. perform the complete synchronous statement(s) inside that closure;
4. drop the connection before the closure returns;
5. never hold a Diesel connection across `.await`;
6. one dispatch chunk and one load shape should normally map to one set-based SQL statement;
7. a loader must never hide a per-key database loop inside the blocking closure.

An adopting field that genuinely requires more than one statement per chunk must document the fixed reason and prove the statement bound. N statements for N parents are non-compliant.

### 4.8 Load API and missing-key discipline

Thoth database DataLoaders must use `try_load`, not `load`.

`Loader::load()` is prohibited for Thoth database loaders because a missing result can panic. The architecture must fail closed through ordinary GraphQL error handling rather than process panic.

Every batch function must be total over its requested keys:

- a valid relationship with no rows returns a successful empty value for that key;
- a valid populated relationship returns the corresponding rows/value;
- a batch-wide backend failure returns an error value for every requested key;
- no requested key may simply disappear from the returned map as a way to encode failure.

### 4.9 Error ownership and GraphQL-visible equivalence

ADR-0006 section 4.9.3 contains a factual overstatement: it describes the existing direct-path GraphQL error contract as always carrying an `extensions.type` discriminator from `ThothError::into_field_error`.

At the verified base, ordinary production resolvers frequently use `.map_err(Into::into)`, which follows Juniper's generic `Display` conversion and does not emit that extension object. `ThothError::into_field_error` exists and does emit stable classifications, but that richer conversion is not the universal current production path.

This ADR therefore requires:

1. when converting an existing field to DataLoader, preserve that field's actual current GraphQL-visible error behaviour unless a separate approved change explicitly standardizes it;
2. for a new field, its task specification must state the intended error convention consistently with current repository policy;
3. a DataLoader batch failure must surface at the owning child field path with the correct null propagation;
4. a failed batch must never become a successful empty result;
5. no per-key fallback/retry SQL may run after a batch-wide backend failure merely to recover individual results;
6. shared batch errors must use a non-panicking, shareable production representation.

The disposable spike used a serialization/deserialization round-trip to obtain owned `ThothError` values for one test comparison. That is evidence scaffolding only and is explicitly NOT the production design. Production implementation must use a safe shareable error projection/snapshot or equivalent representation that preserves the approved GraphQL-visible fields without panic or lossy reconstruction.

Any repository-wide move from generic `.map_err(Into::into)` to explicit `ThothError::into_field_error` is a separate externally observable behavioural decision and must not be smuggled into DataLoader adoption.

### 4.10 Result partitioning and ordering

For each requested key, the loader result must be indistinguishable from the field's direct/set-based contract for:

- membership;
- filtering;
- ordering;
- pagination semantics;
- null/empty distinction;
- authorization scope.

When a field has per-parent pagination or ordering, the set-based query must preserve per-parent semantics, for example through partitioned SQL/windowing or another reviewed set-based construction. A loader that can only reproduce the field by issuing one statement per parent is not eligible.

### 4.11 Authorization and tenant/publisher scope

DataLoader is not an authorization layer and must not weaken one.

A protected field's batch function must preserve the same authorization and publisher-scoping rules as the direct path. Because loaders are request-local, a batcher may safely capture immutable request-scoped authorization context where required, but it must never broaden access because several parent keys are being loaded together.

Every protected loader adoption requires the same negative and positive authorization tests required by `thoth-api/AGENTS.md`.

### 4.12 Read-after-write coherence

Non-cached DataLoader results are not retained after a completed load. A later load of the same key in the same request therefore re-enters the batch function and can observe data written between the two loads.

No mutation-guard mode, store clear, cache invalidation, top-level response scope or execution-occurrence identity is a prerequisite for DataLoader freshness.

Pending requests may still be coalesced within one dispatch. That is batching, not completed-result caching.

### 4.13 Duplicate top-level mutation execution is separate

The pinned Juniper 0.16.2 duplicate-execution finding remains valid:

- compatible repeated top-level mutation occurrences may share one response key;
- validation may accept them;
- the resolver can execute once per occurrence;
- the results are merged under one response key;
- therefore one apparent response field can cause duplicate writes.

This is a shared GraphQL execution concern independent of DataLoader.

Consequences of this ADR:

1. the DataLoader foundation MUST NOT depend on mutation guard mode;
2. loaders are available on their approved resolver paths regardless of `OFF` / `OBSERVE` / `ENFORCE` state;
3. ADR-0006's guard/store availability coupling is superseded;
4. the existing guard remains a separate compensating-control candidate, not part of the N+1 architecture;
5. this ADR does not authorize `OFF -> OBSERVE` or `OBSERVE -> ENFORCE`;
6. this ADR does not authorize remediation, merge or activation of PR #799;
7. OPS-03/OPS-04 are not prerequisites for B0 DataLoader adoption;
8. the eventual keep/simplify/replace/retire decision for mutation protection requires a separate CTO-controlled architecture decision before any production activation.

Until that separate decision, current production-control state remains unchanged: guard activation is unauthorized.

### 4.14 Dependency and framework revalidation

Any change to the resolved versions of Juniper, `juniper_codegen`, `dataloader`, Tokio or Diesel that could affect this architecture must re-run focused compatibility evidence before merge.

At minimum revalidate:

- async child resolver execution;
- sibling batching at N=1, 100, 200, 201 and at least one value above two full chunks;
- current-thread and multi-thread runtime behaviour;
- request-local isolation;
- sequential same-key non-caching;
- loader-first scheduling fixture;
- `try_load` missing-key behaviour;
- one-chunk-to-one-set-based-statement evidence for a representative real-Diesel loader;
- database-failure GraphQL path/null/error equivalence;
- production SDL impact;
- duplicate mutation execution behaviour if Juniper changes.

A future Juniper version that fixes duplicate mutation execution does not silently retire the separate mutation concern; that conclusion must be recorded from new evidence.

### 4.15 Public GraphQL contract

The B0 foundation itself introduces no public GraphQL schema change.

The disposable evidence produced byte-identical production SDL before and with the foundation fixture.

Individual programme tasks such as `BE-02` may add approved GraphQL fields under their own designs/specifications. Their schema changes are not authorized by this ADR alone.

---

## 5. Binding invariants

1. Every production DataLoader is owned by exactly one GraphQL request.
2. No loader or loader result cache crosses request boundaries.
3. Production child loaders use `non_cached::Loader` and `try_load` only.
4. Batch functions are total over requested keys.
5. No loader-backed resolver performs unrelated awaited work before registering its target key unless field-specific evidence explicitly approves that shape.
6. Batching configuration is explicit, initially max batch size 200 and yield count 10.
7. Database access is set-based; no per-parent SQL loop is hidden inside a batch function.
8. Blocking Diesel work uses an approved blocking boundary and no connection crosses `.await`.
9. A batch-wide backend failure fails closed, never becomes successful empty data and never triggers per-key fallback SQL.
10. Converted existing fields preserve their current GraphQL-visible error contract unless a separate approved change alters it.
11. Result membership, ordering, pagination and authorization remain equivalent to the field's approved direct contract.
12. Completed loader results are not reused after return; read-after-write freshness requires no invalidation.
13. Loader availability is independent of mutation guard mode.
14. The duplicate-mutation-execution defect is not declared fixed or irrelevant by adopting DataLoader.
15. The B0 foundation does not change production SDL by itself.
16. Every first production adoption carries measured query-count evidence at representative parent counts and failure-path evidence.
17. Implementing agents may not treat this ADR as implementation authorization; each code change requires its own approved bounded task specification.

---

## 6. Supersession of ADR-0006

If this ADR is approved and becomes repository-authoritative, ADR-0006 becomes `SUPERSEDED`.

The following ADR-0006 architecture is superseded:

- A2 look-ahead-driven prefetch as the shared N+1 mechanism;
- central request-scoped stored-result reuse;
- `GraphqlBatchStore` as the shared batching foundation;
- store-level load-shape/result state machinery;
- top-level-response-key partitioning used to make stored results coherent;
- the response-scope compatibility shim used only by that store architecture;
- mutation guard mode as a prerequisite for loader/store availability;
- `OFF` / `OBSERVE` / `ENFORCE` lifecycle as a prerequisite to enable GraphQL batching;
- operational fleet verification as a prerequisite for child-field batching;
- all-material-path look-ahead prefetch-site coverage as the generic adoption model.

The following ADR-0006 findings/controls are retained or corrected rather than discarded:

- the standing N+1/set-based-loading objective is retained;
- request-local state isolation is retained;
- fail-closed database error behaviour is retained;
- query-count evidence is retained and strengthened around explicit DataLoader chunking/scheduling;
- duplicate top-level mutation execution on pinned Juniper is retained as a separate execution concern;
- production activation of the current mutation guard remains unauthorized;
- ADR-0006 section 4.9.3's description of the existing `extensions.type` contract is corrected as described in section 4.9 of this ADR.

This ADR does not retroactively erase the ADR-0006 implementation/review record. Historical commits, tests, reports and PRs remain evidence of the path taken and why it was superseded.

---

## 7. Implementation model and task boundaries

Architecture risk: HIGH. Implementation should use high reasoning depth and independent exact-diff review.

No implementation is authorized by this proposal.

After this ADR is approved and repository-authoritative, the recommended implementation sequence is:

### 7.1 B0 foundation and A2 retirement - one bounded shared-architecture task

Create one approved task specification before code changes. A suggested identifier is `THOTH-GQL-DATALOADER-01`, but the repository task record is authoritative if a different identifier is assigned.

Scope:

- add the pinned `dataloader` dependency and direct Tokio runtime dependency required by `thoth-api`;
- add request-local loader ownership to `Context`;
- add the safe shared loader-error representation required by section 4.9;
- migrate the general GraphQL test execution helper to async Juniper;
- add reusable test helpers proving chunking, request isolation, non-caching and failure semantics;
- remove ADR-0006 A2 store/prefetch/scope machinery that has no production field consumer;
- remove guard-mode coupling from `Context` and batching availability;
- retain the central mutation guard code only as a separate execution concern, with current activation state unchanged;
- rehost mutation-specific regression tests that currently depend on A2 test fixtures;
- keep production SDL byte-identical;
- adopt no production child field in this foundation task.

Non-goals:

- no BE-02 field implementation;
- no Metrics field implementation;
- no Juniper upgrade;
- no async-graphql migration;
- no mutation-guard activation;
- no PR #799 remediation/merge;
- no production deployment or configuration change;
- no database migration.

### 7.2 First production consumer - Publisher Services BE-02

Only after the B0 foundation is independently reviewed, merge-authorized and merged may `BE-02` adopt it, and only under a freshly reconciled, approved BE-02 specification.

`Publisher.distributionPlatforms` is expected to be the simple loader-first shape: register `publisher_id` at or near resolver entry and let the batch function issue set-based SQL for all pending publisher IDs.

BE-02 must carry field-specific query-count evidence, authorization evidence, error behaviour and its own programme rollout/rollback controls.

### 7.3 Later consumers - Thoth Metrics and others

Metrics does not inherit BE-02's query shapes automatically. Each loader family must define its typed key, result-shaping arguments, set-based SQL and query-count evidence.

Loader-first is the default. Any exception requires explicit field-specific evidence.

### 7.4 Separate mutation-execution decision

The current guard, PR #799 and OPS-03/OPS-04 remain outside the B0 implementation dependency graph.

A separate CTO-controlled decision must determine whether duplicate mutation execution should be:

- mitigated by the current guard;
- mitigated by a simpler guard/control;
- addressed through a future Juniper upgrade if verified fixed;
- accepted with a different documented contract;
- or otherwise replaced.

No option is selected by this ADR.

---

## 8. Migration

Database migration: none.

Public GraphQL schema migration for the B0 foundation: none.

Code migration is additive-first and then subtractive inside one bounded foundation task:

1. introduce the DataLoader dependency, request-local loader bundle and async test harness;
2. prove the new foundation with test-only consumers;
3. rehost independent mutation regression tests;
4. remove A2 store/prefetch/scope infrastructure that has no production consumer;
5. verify production SDL remains byte-identical;
6. merge only after independent review and explicit CTO merge authorization.

No production field moves to DataLoader in the foundation task, so there is no live client cutover at this stage.

---

## 9. Rollout and production activation

### 9.1 Foundation rollout

The B0 foundation is initially inactive from a public-contract perspective because it adopts no production field. Merging the foundation must not:

- add or remove GraphQL fields;
- change request acceptance;
- activate mutation guard modes;
- alter database schema;
- deploy anything by itself.

### 9.2 Consumer rollout

Each programme consumer follows its own approved rollout plan.

For the first production loader-backed field, require at minimum:

- exact query-count evidence at representative list sizes, including at least one value above the configured batch size where the parent query allows it;
- field correctness against representative data;
- authorization tests where protected;
- backend-failure behaviour;
- no per-parent fallback after a batch failure;
- staging/preview verification before production exposure where the programme rollout supports it;
- monitoring sufficient to detect error-rate or database-load regression;
- a documented release rollback.

A new additive field does not require a feature flag merely because DataLoader exists, but the programme task must use a flag/comparison mode when its approved rollout design calls for one.

### 9.3 Mutation guard activation

Unchanged: NOT AUTHORIZED by this ADR.

No `OFF -> OBSERVE` or `OBSERVE -> ENFORCE` transition is implied by B0 adoption.

---

## 10. Validation evidence

### 10.1 Evidence identity

Primary disposable evidence package:

`THOTH-GQL-DATALOADER-SPIKE-02-EVIDENCE.tar.gz`

Archive SHA-256 independently verified during architecture review:

`935d4b913abf6299debae45c32f582c85b691b2a1708a2b9ff9611ff7ac8a769`

Original evidence manifest entries were independently verified, including:

- `spike02-tracked.patch` - `93dec8f0bf5f20a2cf7a8bbbdd80c803d5dceed105a8c538c268fe5044071abb`
- `dataloader_spike_tests.rs` - `03e183dd8f9316a6db2548de113418bb5ed833403dbc1143aedf19c432e64a10`
- `focused-tests.log` - `6b81fdf7c911fcf09211c6db8b54f820990f981a7538984f3c330a0ebd929a54`
- `validation.log` - `4766e0d33a182cdfb174f2c93f78af77afa9168a63a62972d49db8996e3907e1`
- `cargo-changes.diff` - `27fb97091d16d2f3e17341cd1cd7d93bb53cc40f6f1e725c97ce60bc80388a50`
- base and B0 SDL - `1e08b46b565ef719c404bbe6b3131e6a733df09c7abdc4538b66c2b24d2d899c`

### 10.2 Batch-boundary evidence

With max batch size 200 and yield count 10, both current-thread and multi-thread runtimes produced:

| Parent/load count | Dispatches | Batch sizes |
|---:|---:|---|
| 1 | 1 | `[1]` |
| 10 | 1 | `[10]` |
| 100 | 1 | `[100]` |
| 199 | 1 | `[199]` |
| 200 | 1 | `[200]` |
| 201 | 2 | `[200, 1]` |
| 500 | 3 | `[200, 200, 100]` |

N=201 and N=500 were each repeated 20 times per runtime flavour with identical dispatch shapes.

### 10.3 Scheduling evidence

At N=100, 30 repeated multi-thread runs per scenario produced:

- immediate loader call: 1 dispatch in every run;
- one `yield_now()` before load: 1 dispatch in every run;
- deterministic three-yield mixed cohorts: 1 dispatch in every run;
- deterministic 1 ms delayed cohort: 2-3 dispatches;
- loader-behind-loader fixture: 1 target dispatch in every run, with one upstream loader dispatch.

The evidence demonstrates useful coalescing for the tested shapes but does not establish a universal sublinear bound under arbitrary arrival timing. Section 4.5 therefore carries the loader-first adoption invariant.

### 10.4 Real-Diesel evidence

250 real publisher keys, one imprint each:

- DataLoader dispatches: 2;
- batch sizes: `[200, 50]`;
- connection-instrumented imprint statements: 2;
- SQL shape: set-based `WHERE publisher_id = ANY($1)`.

### 10.5 Error evidence

The spike compared complete serialized direct-vs-loader GraphQL failure responses for both:

- the repository-conventional generic error conversion, with no `extensions` object;
- explicit `ThothError::into_field_error`, carrying `extensions.type = INTERNAL_ERROR`.

For each comparison, direct and loader paths matched on data/null propagation, error path, location, message and extensions convention. The loader issued no fallback retry after the batch failure.

A missing batch-map key through `try_load` failed closed as a GraphQL error without a process panic.

### 10.6 Request scope and freshness

The spike demonstrated:

- two concurrent request Contexts over the same logical key shared no loader state;
- a completed load did not satisfy a later completed load of the same key;
- load -> write -> load observed old then new data with two underlying fetches and no invalidation.

### 10.7 Mutation-execution evidence

Pinned Juniper 0.16.2 still executed two compatible repeated top-level mutation occurrences sharing one response key as two resolver executions under async execution, while returning one merged response field.

This evidence is why section 4.13 keeps duplicate mutation execution separate from DataLoader coherence.

### 10.8 Validation and evidence caveat

The supplied validation log records successful:

- focused B0 tests: 25 passed, 0 failed;
- `cargo test -p thoth-api --features backend`: 998 library tests passed plus 13 integration tests, with 8 doc tests ignored;
- `cargo check --workspace`;
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`;
- `cargo test --workspace` with all recorded crate suites passing.

The handoff narrative states `cargo fmt --all -- --check` passed, but the independently inspected `validation.log` does not contain the raw fmt invocation/result. This is a minor evidence-record discrepancy from the disposable spike, not a B0 feasibility blocker. The implementation task must run and record the repository-required format gate normally.

Production SDL was byte-identical: 160,799 bytes with the same SHA-256 before and with the spike fixture.

---

## 11. Required implementation acceptance criteria

A future B0 foundation task must not be approved unless it proves at minimum:

1. exact base/head and dependency resolution;
2. no Juniper upgrade unless separately authorized;
3. request-local loader construction in the real `Context` lifecycle;
4. explicit max-batch-size and yield-count configuration;
5. general GraphQL unit tests execute through the async path without broad caller rewrites;
6. loader-backed fixture resolvers call `try_load` independently and obey loader-first;
7. batching at 1/100/200/201/500 or equivalent values that cross configured boundaries;
8. current-thread and multi-thread stability;
9. one representative real-Diesel dispatch chunk -> one set-based statement;
10. no Diesel connection across `.await`;
11. no per-key SQL loop inside a batch function;
12. request isolation;
13. completed-result non-caching;
14. load -> write -> load freshness;
15. missing-key failure without panic;
16. batch-wide DB failure with correct child-field error path/null propagation and no empty substitution or retry SQL;
17. safe non-panicking production error representation, not the spike's serialization round-trip scaffold;
18. authorization equivalence for any protected fixture/adoption;
19. mutation duplicate-execution regression tests preserved independently of A2;
20. byte-identical production SDL for the foundation;
21. `cargo fmt --all -- --check`;
22. `cargo check --workspace`;
23. `cargo clippy --all --all-targets --all-features -- -D warnings`;
24. `cargo test -p thoth-api --features backend`;
25. `cargo test --workspace`;
26. no migration/schema/data effect;
27. no production activation or deployment;
28. independent exact-head review and separate CTO merge authorization.

A first production consumer must add field-specific query-count, correctness, authorization, error, rollout and rollback evidence on top of the foundation acceptance criteria.

---

## 12. Consequences and risks

### Positive consequences

- conventional, recognizable DataLoader pattern;
- materially smaller request-state model;
- no completed-result cache coherence problem;
- no response-key scoping requirement for batching;
- no batching dependency on mutation guard activation;
- direct fit for BE-02 and reusable pattern for Metrics;
- reduced pinned-Juniper hidden-API surface in the batching mechanism;
- additive first adoption and straightforward foundation rollback before production consumers.

### Costs and risks

- introduces the `dataloader` dependency;
- batching efficiency depends on resolver arrival timing, so loader-first is a binding coding/review rule;
- `Loader::load()` is a panic footgun and must be prohibited in Thoth DB loaders;
- synchronous Diesel must consistently use the blocking boundary;
- argument-bearing fields need carefully typed/normalized loader keys and set-based per-shape semantics;
- `dataloader` is a small, relatively slow-moving dependency; if maintenance becomes unacceptable, the bounded contingency is to vendor or replace its small mechanism under a future reviewed change;
- the duplicate-mutation-execution concern remains unresolved as a separate production-safety decision;
- A2-specific tests and current guard tests are intermingled and require careful rehosting rather than wholesale deletion.

---

## 13. Rollback

### Before any production field adopts B0

The foundation can be reverted as one bounded release change because it carries no public SDL, database or production-behaviour requirement of its own.

Rollback restores the previous code foundation but does not reactivate or authorize mutation guard modes.

### After production fields adopt B0

Rollback is programme-specific and must preserve public API compatibility. Do not silently revert a loader-backed field to an N+1 direct query merely to remove DataLoader; that would violate the standing GraphQL control.

An adopting task must instead provide a release rollback that either:

- reverts the whole not-yet-relied-upon additive field/release within its approved compatibility window; or
- restores another proven set-based/batched implementation with the same public contract.

No destructive database rollback is involved in the B0 foundation.

---

## 14. Approval state

Current state: `APPROVED`.

Approved by: CTO
Approval date: 2026-08-11

This ADR records the architecture decision supported by the independently reviewed B0 evidence. It does not authorize implementation, merge, deployment, mutation-guard activation, BE-02 runtime work or Metrics runtime work.

The CTO approved this decision on 2026-08-11. The recording steps it requires are:

1. record the approval date/owner in this ADR;
2. update ADR-0006 to `SUPERSEDED` and point it to ADR-0007;
3. update `decision-register.md` with ADR-0007 and ADR-0006's superseded status;
4. include the required `CHANGELOG.md` entry;
5. obtain independent exact-head review of the complete documentation diff;
6. obtain explicit CTO merge authorization;
7. merge the approved exact content into `develop`;
8. only then specify and authorize the bounded B0 foundation implementation task.

PR #799 remains frozen, draft and unmerged. Its remediation or closure is not authorized by this ADR proposal and is not a prerequisite for B0 batching.
