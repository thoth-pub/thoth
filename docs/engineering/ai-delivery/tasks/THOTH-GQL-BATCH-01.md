# THOTH-GQL-BATCH-01 - Request-scoped GraphQL batching foundation

Status: DRAFT
Programme: Shared Thoth GraphQL / Backend Architecture
Dependent programmes: Publisher Services and Distribution Configuration (first
required consumer, `BE-02`); Thoth Metrics and any other programme resolving
child fields through `thoth-api` GraphQL
Repository: `thoth-pub/thoth`
Workflow: STANDARD
Base branch: `develop`
PR target: `develop`
Programme integration branch: None
Risk: HIGH
Owner: Shared backend architecture
Approved by: not yet approved
Dependencies: `ADR-0006` (APPROVED and repository-authoritative); this
specification approved; a freshly verified exact `develop` base; explicit CTO
implementation authorization
Target branch name: `feature/shared-architecture/graphql-batching`

Authority condition: this record is repository-authoritative when this exact
content is reachable from the repository's authoritative integration branch.
Live review, authorization and merge evidence is the GitHub pull-request record.

This specification does not authorize implementation. It defines what
implementation must do once separately authorized.

---

## 1. Objective

Give Thoth's GraphQL API a reusable, request-scoped mechanism by which a
loader-backed field reached from a list of parent objects — whether as a direct
child of those parents or as a **descendant** beneath intermediate object fields
— can be resolved with a bounded, set-based number of database statements
instead of one statement per parent, so that new nested fields can satisfy the
`thoth-api/AGENTS.md` section 6 N+1 control by following a repository pattern
rather than escalating an architecture decision.

The task delivers the foundation and proves it. It adopts the foundation in no
production field.

Four boundaries are set by `ADR-0006` and are binding on this task:

- **the central mutation request guard is in scope** (`ADR-0006` section 4.12.6).
  Before any resolver executes, a **mutation** operation in which one executable
  top-level response key occurs more than once is rejected at the request
  boundary. This is a **shared GraphQL execution prerequisite**, not a batching
  helper: it protects every mutation whether or not a loader-backed field is
  selected, because the defect it compensates — pinned Juniper executing one
  response key's compatible occurrences as several resolver invocations — causes
  a duplicated **write**. Query operations are not restricted;
- **top-level-response-key scoping is in scope** (`ADR-0006` section 4.12). The
  store is owned by one GraphQL request but partitioned by the current top-level
  GraphQL response key, giving the store identity
  `(top-level response key, loader identity, normalized load shape, parent key)`.
  The scoping rule is applied uniformly to queries and mutation payloads; no
  resolver detects operation type. This requires the single pinned-Juniper
  compatibility shim of `ADR-0006` section 4.12.8. Its mutation isolation
  guarantee **depends on** the guard above, and the store must be unavailable
  without it (`ADR-0006` section 4.12.6.6);
- **descendant prefetch is in scope** (`ADR-0006` section 4.19). The material
  fan-out paths the ADR identifies reach the loader-backed field through an
  intermediate object (`QueryRoot.imprints -> Imprint.publisher ->
  Publisher.distributionPlatforms`), so a direct-child-only mechanism would not
  satisfy the ADR's own coverage rule and would force `BE-02` to invent a second
  architecture;
- **mutation-payload fan-out is a supported path**, not an exception. The guard
  plus scope isolation make correctness independent of whether the executor
  serializes or interleaves top-level mutation fields, so no prohibition on
  mutation-reachable prefetch sites applies, and **no production mutation
  resolver is modified**.

### 1.1 Risk rationale

`HIGH`, and the classification is deliberate rather than defaulted.

`risk-classification.md` assigns HIGH to work involving "changes to canonical
data semantics" and to changes "capable of broadening processing scope". This
task adds shared request-scoped state to the GraphQL `Context` used by every
query and mutation resolver, and introduces a data-loading path whose result is
substituted for a field's direct database read. Implemented incorrectly it could
attach one parent's rows to another parent, leak state between requests, return
a stale or empty result where a live query would have returned data, or serve
rows the caller's publisher scope excludes.

The escalation rules also apply: the affected production query volume is
unknown, and the mechanism is intended for reuse by callers that do not yet
exist. Uncertainty raises the level; it does not justify lowering the controls.

Two facts do **not** reduce the classification: that no migration is required,
and that no field adopts the mechanism at merge. Both limit the *blast radius at
merge*; neither limits the correctness surface of the mechanism itself.

#### Re-evaluation after the mutation execution-scope remediation

The classification was re-run against `risk-classification.md` from scratch after
the remediation of `ADR-0006` section 4.12.6, rather than carried forward by
default. The result is **`HIGH`** — unchanged as a label, but reached on
materially different and stronger grounds, and it is no longer reached only
through the store's correctness surface.

**What changed.** The specification now carries a central request-boundary guard
that rejects mutation operations with a duplicate executable top-level response
key. That is runtime behaviour on the common GraphQL request path, live from the
merge commit, affecting every mutation request from every API client.

Against the framework, that engages criteria the previous classification did not:

| `risk-classification.md` criterion | Engaged by |
|---|---|
| production feature activation | the guard is active on the production request path at merge, not dark-launched (`ADR-0006` section 7.2) |
| cross-repository API contract change | the set of accepted GraphQL mutation requests changes for all clients, including clients outside this repository |
| changes to canonical data semantics | unchanged from before: a data-loading path is substituted for a field's direct read |
| changes capable of broadening processing scope | unchanged from before |

The escalation rules also still apply: the affected production query volume is
unknown, and no exhaustive inventory of external clients exists in this
repository.

**Why it is not raised to `Critical`.** Checked criterion by criterion rather
than assumed: there is no destructive or irreversible production migration, no
canonical data rewrite at scale, no mass redistribution or external publication,
no security boundary affecting all publishers (the guard only ever **rejects**,
so it cannot broaden access, and authorization is untouched), no secrets or
identity-provider work, no historical metrics recomputation, no source-of-truth
cutover, and no material legal, privacy or contractual consequence. Rollback is
also **not** uncertain, which is the escalation rule that would otherwise bite:
the kill switch of section 11 restores prior request-acceptance behaviour
without a deploy, and reverting the merge commit is clean because nothing has
adopted the store.

**Why it is not lowered.** `Medium` would require the behaviour to be
feature-flagged with limited data effect and no automatic production effect. The
guard has automatic production effect at merge, so `Medium` does not fit.

`HIGH` is therefore re-derived, not retained by inertia. One required control
changes as a result: `risk-classification.md`'s "feature flag, comparison mode or
controlled pilot where possible" is now genuinely engaged and is discharged by
the kill switch of section 11, which was not required by the previous
classification.

The remainder of this section records the store-side rationale, which is
unchanged.

Factors raising the correctness surface:

- the store is shared request-scoped state on the GraphQL `Context` used by every
  query and mutation resolver;
- **every** loader lookup now depends on correct scope extraction. A wrong or
  insufficiently discriminating scope value could let entries cross top-level
  response keys, which is the mechanism preventing a mutation payload from
  serving a stale read;
- scope collision is a response-correctness risk, not merely a performance one;
- mutation read-after-write safety is now architecturally load-bearing, since
  mutation-payload fan-out is a supported path;
- the design acquires a documented compatibility coupling to the pinned Juniper
  API (`ADR-0006` sections 4.12.8, 4.12.14);
- descendant prefetch adds recursive alias-safe traversal and a key projector
  crossing an intermediate field.

Factors bounding it:

- no database migration, no `schema.rs` change, no data semantics change on disk;
- no production consumer of the **store** at initial merge — no field adopts the
  batching mechanism. This no longer means the task has no production effect at
  merge: the guard does (`ADR-0006` section 7.2);
- no new workspace dependency, including for the shim and the guard;
- no public GraphQL **schema** change, and the generated SDL is byte-identical.
  The set of accepted **requests** does change (`ADR-0006` section 4.12.6.7);
- the always-correct direct fallback means a scope-extraction failure degrades to
  unbatched-but-correct, not to wrong data (`ADR-0006` section 4.12.9);
- the guard fails **closed**: it only ever rejects a request, and a rejected
  request executes no resolver and performs no write, so its failure mode is
  refusal rather than incorrect data.

Against the framework, this sits squarely in `HIGH`: it matches "changes to
canonical data semantics" (a data-loading path substituted for a field's direct
read) and "changes capable of broadening processing scope", and the escalation
rules apply because the affected production query volume is unknown and the
mechanism is intended for callers that do not yet exist.

It does **not** meet any `Critical` criterion: there is no destructive or
irreversible production migration, no canonical data rewrite, no mass
redistribution or external publication, no security boundary change affecting all
publishers (authorization is untouched, and scoping is isolation rather than
authorization — `ADR-0006` section 4.13), no secrets or identity-provider work,
no metrics recomputation, no source-of-truth cutover, and no material legal,
privacy or contractual consequence.

`HIGH` is therefore reached on store-side grounds too, with the shim-specific
acceptance criteria of section 9. The required controls of section 1.2 are
**not** unchanged: the kill switch added by the guard's re-derivation above is a
newly engaged control.

### 1.2 Required HIGH-risk controls

Per `risk-classification.md` and `release-gates.md` section 1:

- approved design (`ADR-0006`) and this approved specification;
- implementation at high or maximum reasoning;
- independent cross-model review;
- failure-path and authorization tests;
- rollout and rollback plan;
- **kill switch** for the request-boundary guard (section 11), discharging
  `risk-classification.md`'s "feature flag, comparison mode or controlled pilot
  where possible" for HIGH-risk work. This control is newly engaged, because the
  guard has production effect at merge;
- explicit CTO merge authorization;
- production activation of the **store**, if any is ever required, separately
  authorized. Note that the **guard** activates at merge and is therefore covered
  by the merge authorization itself, not by a later activation decision.

---

## 2. Background and authority

Authoritative sources:

- [`ADR-0006`](../../decisions/ADR-0006-request-scoped-graphql-batching.md) -
  the governing architecture decision; every binding mechanism detail is settled
  there and is not restated in full here;
- [`thoth-api/AGENTS.md`](../../../../thoth-api/AGENTS.md) section 6 - the N+1
  control this task exists to make satisfiable;
- [`ADR-0003`](../../decisions/ADR-0003-repository-authoritative-schema-contract.md) -
  the schema contract, relevant only to confirm this task does not touch it;
- the `BE-02` specification, sections 9.2.1 and 19.1 - the escalation that
  surfaced the gap. That specification is not yet on `develop`; it is carried by
  PR [#788](https://github.com/thoth-pub/thoth/pull/788), and
  `docs/engineering/ai-delivery/tasks/BE-02.md` becomes linkable from here only
  once that pull request merges. `BE-02` is a **dependent** of this task, not a
  dependency of it;
- [PR #788](https://github.com/thoth-pub/thoth/pull/788) - the `BE-02`
  specification pull request. **This task must not modify it.**

### 2.1 Current behaviour, verified at `5a8c27b1b7c11a4f6bd26d459556468099f8c1f4`

- `Context` (`thoth-api/src/graphql/model.rs:54`) holds `db`, `user`,
  `s3_client`, `cloudfront_client` and nothing else;
- `Context` is constructed once per GraphQL HTTP request
  (`thoth-api-server/src/lib.rs:96`) and once per test
  (`thoth-api/src/model/tests.rs:147,192,197`);
- production executes GraphQL asynchronously —
  `data.execute(&st, &ctx).await` (`thoth-api-server/src/lib.rs:98`);
- the GraphQL test suite executes synchronously — `juniper::execute_sync`
  (`thoth-api/src/graphql/tests.rs:52`). **Both paths must keep working;**
- child fields query once per parent. `Publisher.imprints` calls
  `Imprint::all(..., Some(self.publisher_id), ...)`; `Publisher.contacts` calls
  `Contact::all(...)` the same way; `thoth-api/src/graphql/model.rs` contains 63
  resolver methods taking `context: &Context`;
- Juniper look-ahead is available (`juniper` 0.16.2) and unused in the
  repository; there is no DataLoader and no `dataloader` dependency;
- existing child fields already take result-changing arguments:
  `Publisher.imprints` takes `limit`, `offset`, `filter` and `order`;
  `Publisher.contacts` takes `limit`, `offset`, `order` and `contactTypes`.
  A load-shape-free cache key is therefore not a viable shared architecture
  (`ADR-0006` section 4.4.1);
- `LookAheadSelection::arguments()` reads only literal AST arguments and does
  **not** apply schema defaults (`src/executor/look_ahead.rs:577-590`), while the
  child resolver receives the default-applied value. Shape construction must
  normalize defaults explicitly (`ADR-0006` section 4.4.3);
- `LookAheadChildren::select()` / `has_child()` match on `field_name()`, which is
  **the alias when present** (`src/executor/look_ahead.rs:419-441`). Neither finds
  `a: distributionPlatforms`, and both return only the first match. Prefetch sites
  must iterate `children()` filtering on `field_original_name()`
  (`ADR-0006` section 4.15.1);
- `ThothError` derives `Error, Debug, PartialEq, Eq, Serialize, Deserialize` and
  is **not** `Clone` (`thoth-errors/src/lib.rs:11`); it maps to a `FieldError`
  carrying an `extensions.type` discriminant
  (`thoth-errors/src/lib.rs:183-207`);
- `Publisher` is reachable under a fan-out by more than its own root list query —
  at minimum `QueryRoot.publishers` (`query.rs:521`), `QueryRoot.imprints`
  (`query.rs:593`) `-> Imprint.publisher` (`model.rs:1366`), and
  `QueryRoot.contacts` (`query.rs:1868`) or `Publisher.contacts`
  `-> Contact.publisher` (`model.rs:3120`);
- there is no SQL statement-count test facility. Diesel 2.3.10 provides
  `set_default_instrumentation`, which is unused here. The ordinary test pool is
  a process-wide `OnceLock<Arc<PgPool>>` (`thoth-api/src/model/tests.rs:36,63-70`)
  whose connections may already be established, so it must **not** be the pool
  under measurement (`ADR-0006` section 8.1.1).

The implementing agent must refresh all of the above against its own exact base
before editing.

---

## 3. Explicit scope

The task must:

0. implement the **central mutation request guard** of `ADR-0006` section 4.12.6
   as its own module under `thoth-api/src/graphql/`, separate from the store,
   and invoke it from the production handler
   (`thoth-api-server/src/lib.rs`) **before** `data.execute(&st, &ctx).await`.
   This is numbered first because the store's mutation isolation guarantee
   depends on it. Binding properties, all of which use public non-`unsafe`
   juniper API only:

   - it parses the incoming document with
     `juniper::parser::parse_document_source`, selects the operation with
     `juniper::executor::get_operation` honouring `operationName`, and reads
     `Operation::operation_type`;
   - it applies **only** to `OperationType::Mutation`. Query operations are
     returned unchanged and are never restricted (4.12.6.8);
   - it expands named fragment spreads and inline fragments before counting
     top-level occurrences, and is cycle-safe;
   - it evaluates `@skip`/`@include` against the request's coerced variables, so
     a definitely-excluded occurrence is **not** counted as executable. Juniper's
     own `is_excluded` is `pub(super)`, so this must be reimplemented on public
     API and kept behaviourally identical;
   - where a directive condition cannot be resolved for the concrete request, it
     counts the occurrence as executable — rejecting conservatively rather than
     admitting a possible duplicate write (4.12.6.7);
   - it rejects when any executable top-level response key occurs more than once,
     returning `GraphQLResponse::from_result(Err(GraphQLError::ValidationError(..)))`
     carrying a `RuleError` with the colliding source positions, so the existing
     handler branch produces **HTTP 400** with the ordinary GraphQL
     validation-error body and no `data` key. No new handler branch and no
     one-off HTTP protocol;
   - the message must not expose loader, store or scope internals, and must not
     imply the document is invalid GraphQL;
   - it **must not** replace `GraphQLRequest::execute`, and must not make any
     authorization decision;
   - rejection precedes execution entirely: mutation resolver execution count and
     database write count are both **zero** for a rejected operation;
   - it modifies **none** of the 88 resolver methods on `MutationRoot`;
   - it emits exactly one warning-level log record per rejection, carrying the
     operation name if supplied and the colliding response key, and **never** the
     document, the variables or any argument value;
   - `juniper::ast` is a private module, so `Fragment`, `Field`, `Directive` and
     `ast::Arguments` are not publicly nameable. Named-fragment expansion must
     hold `&[Selection]` selection sets, and directive evaluation must be written
     so those types are only inferred, never named in a signature;
1. add the request-scoped store to `crate::graphql::model::Context` per
   `ADR-0006` sections 4.1-4.4, initialised empty at construction, keeping
   `Context: Sync` so the async execution path continues to compile;
2. add a focused module under `thoth-api/src/graphql/` containing the store, the
   closed loader-identity discriminant, the **typed load-shape contract**, the
   set-based loader contract, key de-duplication, deterministic result
   partitioning, failure recording, and the look-ahead-driven prefetch helper;
3. key the store by
   `(top-level response key, loader identity, normalized load shape, parent key)`
   per `ADR-0006` sections 4.4-4.4.4 and 4.12, with the load shape typed and
   loader-specific, never a serialized GraphQL argument string, and with a
   **single loader-owned shape constructor** used by both the prefetch site and
   the child lookup so the two cannot drift;
4. implement **top-level response-key partitioning** per `ADR-0006` section
   4.12, applied uniformly to query and mutation operations. Binding properties:

   - no loader entry, successful or failed, crosses top-level response-key
     scopes;
   - the scope key is the GraphQL **response key**, therefore the alias when one
     is present, and is never normalized to the schema field name (4.12.5);
   - **No** source-position or AST-occurrence component is added to the scope
     key. `ADR-0006` section 4.12.6.3 rejects that option on evidence: a nested
     resolver cannot derive a top-level execution-occurrence identity on the
     pinned stack, because `path()` and `location()` can both be identical
     across two distinct top-level mutation executions;
   - in a **query** operation, repeated occurrences of one top-level response key
     share one scope, and that is correct and required (4.12.6.8);
   - in a **mutation** operation, a duplicate executable top-level response key
     cannot arise, because the section 6.6 guard rejects the request before
     execution. The scope's one-to-one correspondence with a write execution
     **depends on** that guard (4.12.6.4);
   - **no** resolver detects operation type, and the raw GraphQL document is
     never parsed to derive scope (4.12.7). The guard is the sole place where the
     document is parsed and the operation type is read, and it runs at the
     request boundary, never inside a resolver;
   - both the prefetch site and the terminal child resolver derive the scope from
     the same helper, so they necessarily agree;
5. implement the **pinned-Juniper compatibility shim** of `ADR-0006` section
   4.12.8 as one small, isolated, separately-documented module exposing a single
   helper materially equivalent to
   `top_level_response_key(executor) -> Result<ScopeKey>`, which must:

   - use `Executor::new_error(..)` solely to materialize the current execution
     path, then read `ExecutionError::path()` and return its first segment;
   - never call `push_error` / `push_error_at`, never modify the GraphQL
     response, and never issue SQL;
   - **fail closed** when no top-level response key can be derived (4.12.9): the
     prefetch is skipped and lookups read `NotLoaded`, falling back to the direct
     query. Substituting a shared or request-global namespace is prohibited;
   - never parse the raw query string, never inspect private Juniper fields, and
     never use `unsafe`;
   - be the **only** site in the codebase using this technique — `new_error(..)`
     path extraction must not be scattered across loaders, prefetch sites or
     resolvers;
   - carry module documentation and tests that state the pinned-Juniper coupling
     and the revalidation obligation of `ADR-0006` section 4.12.14 explicitly.

   No package dependency may be added to implement the shim;
6. implement default normalization so an omitted argument and an explicitly
   supplied schema default produce the same shape, given that look-ahead does not
   apply schema defaults (`ADR-0006` section 4.4.3);
7. dispatch **once per unique
   `(top-level response key, loader identity, load shape)`** over the
   de-duplicated key set, never once per parent and never one dispatch shared
   across argument variants or across scopes (`ADR-0006` section 4.4.4);
8. implement the loader contract as a **single** set-based statement per
   dispatch, using Diesel `.eq_any(...)` (`WHERE key = ANY(...)`), returning raw
   canonical model rows rather than GraphQL objects (`ADR-0006` section 4.5);
9. implement the three-state store — `NotLoaded`, `Loaded(Vec<V>)` including
   `Loaded([])`, and `LoadFailed` — with the child-resolver behaviour table of
   `ADR-0006` section 4.7, so that only `NotLoaded` triggers the direct-query
   fallback;
10. implement failure recording per `ADR-0006` section 4.9: the parent list
   resolver still returns its parents successfully, the failure is recorded once
   per `(scope, loader, shape)` dispatch with the attempted key set, each covered
   child resolver returns the derived `FieldError`, and **no retry query is
   issued**. Failure-dispatch identity must match the successful-load identity
   exactly, including its scope component, so a `LoadFailed` recorded under one
   scope never poisons another (`ADR-0006` sections 4.9.2, 4.12; invariant 31).
   `ThothError` is not `Clone`, so retain a shareable representation;
11. implement duplicate-key handling, alias handling per shape and the
   non-destructive read (`ADR-0006` section 4.6), including that a second
   prefetch covering an already-loaded `(scope, loader, shape, key)` set issues
   no additional SQL **within that scope**;
12. enumerate child selections by iterating `children()` and filtering on
    `field_original_name()`, never via `select()` / `has_child()`
    (`ADR-0006` section 4.15.1), applying this at **every** segment of a
    descendant selection path (`ADR-0006` section 4.19.3);
13. implement **descendant prefetch** per `ADR-0006` section 4.19. A prefetch
    site must be able to target a loader-backed field reached through one or more
    intermediate object fields, and must settle all four concepts the ADR
    requires — selection path, terminal loader identity, terminal normalized
    load-shape constructor, and a key projector from the resolved list item to
    the terminal loader key. The implementation chooses its own Rust
    representation; the ADR mandates the concepts, not type names.

    Binding properties:

    - traversal is recursive or path-based over `LookAheadSelection::children()`,
      matching `field_original_name()` at every segment;
    - **every** matching terminal selection is collected, across every matching
      intermediate branch — traversal must not stop at the first match at any
      level;
    - the terminal load shape is constructed from each matching **terminal**
      selection, never from an ancestor selection;
    - projected keys are de-duplicated before dispatch;
    - results are stored under the **ordinary** terminal identity
      `(loader, shape, terminal key)`. A separate cache namespace for indirectly
      prefetched entries is prohibited — an entry prefetched from an ancestor and
      one prefetched from the terminal field's own parent list must satisfy each
      other's lookups;
    - the key projector may read **only** data already present on
      already-resolved, already-authorized items, and the four conditions of
      `ADR-0006` section 4.19.4 must hold;
14. add the explicit **whole-store** invalidation entry point required by
    `ADR-0006` section 4.12.5 — clearing `Loaded` and `LoadFailed` state across
    all loaders and all shapes — unused by this task;
15. add the SQL statement-count test facility required by `ADR-0006` section
    8.1.1, using a **dedicated pool constructed after the instrumentation hook is
    installed**, under the existing exclusive database test lock, and isolating
    the measured operation's statements from setup and migration statements;
16. prove the mechanism end to end through Juniper execution using a
    **test-only** GraphQL root and object types defined under `#[cfg(test)]`,
    against existing tables, exercising real look-ahead, real set-based SQL and
    real partitioning. The fixture must include an **argument-bearing** test-only
    field so multi-shape behaviour is proven without adopting a production field
    (`ADR-0006` section 4.4.6), must support **two prefetch sites** covering
    one loader so multi-site coverage is proven (`ADR-0006` section 4.18.3), and
    must include **both a direct and an indirect (descendant) path** per section
    3.1 below;
17. update `thoth-api-server/src/lib.rs` only if `Context::new`'s signature
    changes; prefer initialising the store internally so it does not;
18. add the tests of section 10 and the query-count evidence of section 9.

### 3.1 The proof consumer

`ADR-0006` requires a real consumer to prove the abstraction. `BE-02`'s
`DistributionPlatform` model does not exist and cannot be used, and adopting an
existing production child resolver would begin the legacy migration this task
excludes.

Required approach, in order of preference:

1. **A test-only GraphQL schema.** Define, under `#[cfg(test)]`, a root query
   exposing a list of an existing parent type and a loader-backed child field
   over an existing child table (`imprint` keyed by `publisher_id` is a
   suitable, already-seeded relationship — `create_publisher` and
   `create_imprint` helpers exist in `thoth-api/src/model/tests.rs`). Build a
   test-only `RootNode` over it and execute with `juniper::execute_sync`. This
   exercises the whole mechanism with **no public schema change and no change to
   any existing resolver**, and is the required approach if it is workable.
2. If and only if approach 1 proves technically impossible, the implementing
   agent must stop and report `BLOCKED` with the exact obstacle, rather than
   adopting a production resolver on its own authority.

The proof consumer must not trigger any broader cleanup of the relationship it
uses.

The fixture must additionally provide, all within `#[cfg(test)]`:

- an **argument-bearing** loader-backed field, so load-shape behaviour is proven
  against real Juniper argument handling. The field's arguments must include at
  least one with a **schema default**, so default normalization is exercised.
  This must not be achieved by adding arguments to any production field
  (`ADR-0006` sections 4.4.6 and 4.4.5);
- **two distinct prefetch sites** covering the same loader, so `ADR-0006` section
  4.18.3's requirement — that the mechanism supports multi-site coverage without
  duplicate loading — is proven by the foundation rather than deferred to
  `BE-02`;
- **test-only mutations** returning payload objects whose selections reach a
  loader-backed field, so mutation-payload batching, read-after-write and
  cross-top-level scope isolation can be proven without touching any production
  mutation resolver (`ADR-0006` sections 4.12.10-4.12.11). At least two distinct
  test-only top-level mutation fields are required, so a single operation can
  carry two top-level response keys.

#### 3.1.1 Descendant prefetch must be proven, not only direct prefetch

Proving two **direct** parent-list prefetch sites is no longer sufficient. The
fixture must contain at least both shapes:

```text
direct path:    list -> loader-backed child
indirect path:  list -> intermediate object -> loader-backed descendant child
```

The indirect case must prove all of:

- recursive or path-based look-ahead inspection across the intermediate segment;
- alias-safe matching at **every** level, including intermediate segments;
- key projection from the already-resolved list item;
- de-duplication of the projected terminal keys;
- terminal load-shape construction from the terminal selection;
- **one** set-based terminal dispatch for the unique key set;
- correct terminal child results, equal to the direct per-parent result in order;
- **no** terminal fallback statement on the covered path;
- repeated descendant paths and repeated sites reusing already-loaded terminal
  entries rather than re-dispatching;
- the existing intermediate resolver behaviour being **unchanged** — the fixture
  must not require modifying it.

Use test-only GraphQL types or wrappers where necessary. A production resolver
must **not** be modified merely to prove this. The `imprint` -> `publisher`
relationship is a suitable model for a test-only intermediate, since
`Imprint.publisher_id` is present on the resolved row
(`thoth-api/src/model/imprint/mod.rs:44`), but the fixture must express it
through its own `#[cfg(test)]` types.

The measurement must additionally report the terminal-loader statement count
**separately** from any statements issued by the intermediate resolver, per
`ADR-0006` sections 4.19.5 and 8.2 item 9. A single combined figure is not
acceptable evidence.

**Stop condition.** If descendant prefetch cannot be expressed cleanly using
stable pinned Juniper APIs plus already-resolved data, the implementing agent
must stop and report:

```text
BLOCKED - A2 CANNOT COVER INDIRECT FAN-OUT WITHOUT NEW EXECUTION ARCHITECTURE
```

The unresolved architecture must **not** be pushed into `BE-02`.

### 3.2 Execution-path coverage

The mechanism must be proven under **both** execution paths:

- `juniper::execute_sync`, as used by the existing test suite;
- asynchronous `execute`, as used in production.

An implementation that works only under one is non-compliant. The mechanism is
synchronous by design (`ADR-0006` section 3.1 variant A2), so this should follow
from the design rather than require two implementations; it must nonetheless be
demonstrated, not assumed.

---

## 4. Non-goals

The task must not:

1. implement any part of `BE-02` at runtime;
2. add `DistributionPlatform`, `publisher_distribution_platform`, or any `BE-02`
   GraphQL field, type or enum;
3. modify PR [#788](https://github.com/thoth-pub/thoth/pull/788), its branch, or
   the `BE-02` specification;
4. modify [issue #765](https://github.com/thoth-pub/thoth/issues/765);
5. migrate, refactor or "improve" any existing child resolver;
6. perform performance work unrelated to the batching primitive;
7. migrate GraphQL execution to async-only, convert resolvers to `async fn`, or
   change the test suite off `execute_sync`;
8. add a database migration, table, column, enum, index or constraint, or edit
   `thoth-api/src/schema.rs`;
9. change the public GraphQL schema. No production type, field, argument, enum
   value or description may change, and the generated SDL must be unchanged;
10. add a workspace dependency, including any DataLoader crate;
11. change authorization policy, `thoth-api/src/policy.rs`, or any role or scope
    rule;
12. implement `ADR-0006` option C (an N+1 exception) or option D (removing the
    nested field);
13. deploy, release, or run a production migration. **Narrow exception,
    deliberate and recorded:** the section 6.6 request guard is behaviour that
    takes effect on the common GraphQL request path at merge (section 11). That
    is not a discretionary activation the implementing agent performs — it is a
    property of the merge itself, covered by the CTO merge authorization, and it
    must be reported plainly rather than described as inert. No **store** feature
    is activated for any production field;
14. change CI workflows, repository settings or branch protection;
15. modify any production mutation resolver, or require mutation resolvers to
    call invalidation. Avoiding that retrofit is a principal reason `ADR-0006`
    section 4.12 partitions by scope rather than invalidating on write;
16. upgrade `juniper`, or attempt to fix Juniper's concurrent async execution of
    mutation root fields. The architecture is designed not to depend on that
    behaviour (`ADR-0006` section 4.12.10);
17. detect the GraphQL operation type at a nested resolver, or parse the raw
    GraphQL document to derive execution **scope**. The section 6.6 guard does
    parse the document and does read the operation type, but it does so once, at
    the **request boundary**, before any resolver runs, and never to derive a
    scope value. That is the only permitted document parsing;
18. use `Executor::new_error(..)` path extraction anywhere outside the single
    compatibility-shim module;
19. broaden the guard's compatibility restriction beyond `ADR-0006` section
    4.12.6.7 — in particular, it must not restrict query operations, duplicate
    response keys below the top level of a mutation, or distinct top-level
    aliases;
20. implement `ADR-0006` option F3 (correcting the execution layer so compatible
    repeated top-level fields execute once), fork or patch Juniper, or begin
    maintaining a custom GraphQL executor. That option is rejected as
    architecture expansion in `ADR-0006` section 4.12.10.1 and would require its
    own architecture decision;
21. add an execution-occurrence or source-position component to the scope key.
    `ADR-0006` section 4.12.6.3 rejects that on evidence.

---

## 5. Invariants

The implementation must preserve every invariant in `ADR-0006` section 5. In
summary, and binding here:

1. request-scoped state never crosses GraphQL requests;
2. one parent's data is never returned for another parent's key;
3. batching never broadens authorization or publisher scope — keys come only
   from already-resolved, already-authorized parents;
4. database errors fail closed, never becoming an empty or unfiltered result;
5. loader output is deterministic for a given input key set and load shape;
6. duplicate keys cause no duplicate backend fetches within one top-level
   response scope, per `(scope, loader, shape)`;
7. per-key ordering matches the owning field's contract and the direct
   per-parent result exactly;
8. no public GraphQL schema change;
9. no global, static or cross-request cache, and no singleton;
10. no stale read-after-write result within one operation;
11. a loader-backed field is correct whether or not a prefetch ran;
12. non-adopting fields are behaviourally unchanged;
13. store identity is
    `(top-level response key, loader identity, normalized load shape, parent key)`;
    argument variants never share an entry, omitted arguments normalize to the
    schema default, and entries never cross top-level response-key scopes;
14. `NotLoaded`, `Loaded([])` and `LoadFailed` are distinguishable, and only
    `NotLoaded` triggers the fallback;
15. a failed prefetch does not fail the parent list field, and issues no retry
    SQL;
16. the existence of a correctness fallback is never treated as N+1 compliance
    evidence;
17. a prefetch site may target a descendant loader-backed field, and descendant
    results are stored under the ordinary terminal identity of invariant 13. No
    separate cache namespace exists for indirectly prefetched entries;
18. a descendant key projector reads only data already present on
    already-resolved, already-authorized items, and never bypasses an
    intermediate authorization decision;
19. terminal-loader compliance and legacy intermediate resolver performance are
    reported as separate evidence scopes, and neither is presented as the other;
20. loader state is owned by one request but partitioned by top-level response
    key: storage lifetime and reuse namespace are distinct, the store still never
    crosses requests, and reuse is confined to one top-level response key within
    a request;
21. the scope key is the GraphQL response key — the alias when present — and is
    never normalized to the schema field name. Selection-path matching continues
    to use `field_original_name()`, because it identifies schema fields;
22. scope derivation happens in exactly one isolated compatibility shim, which is
    side-effect-free: it adds no GraphQL error, alters no `errors[]` entry,
    changes no result data and issues no SQL;
23. scope derivation fails closed: a site that cannot derive its scope performs no
    prefetch and its lookups read `NotLoaded`. No shared or request-global
    namespace is ever substituted;
24. the scoping rule is applied uniformly to queries and mutation payloads; no
    resolver detects operation type, and mutation-payload fan-out is a supported
    covered path rather than an exception;
25. correctness does not depend on the executor serializing top-level mutation
    fields;
26. request-wide reuse across top-level response keys is **not** an invariant: the
    same `(loader, shape, key)` sub-tuple reached under two scopes is two entries
    and is loaded once per scope, and the resulting extra dispatches are bounded
    by the operation's top-level structure rather than by parent count. "Two
    prefetch sites covering the same loader in one request issue no duplicate
    SQL" is **false** as stated and must not appear; the rule is same-scope
    reuse;
27. for an accepted **mutation** operation, each executable top-level response key
    occurs exactly once, guaranteed by the section 6.6 request guard rather than
    by GraphQL validation, so a scope corresponds to exactly one mutation
    resolver execution;
28. a **rejected** mutation operation executes no mutation resolver and performs
    no database write;
29. the store is unavailable whenever the guard is not applied; "batching on,
    guard off" is not a representable state.

---

## 6. Required behaviour

### 6.1 Success behaviour

- a prefetch site that opts in derives its top-level response scope, enumerates
  every requested terminal selection of the loader-backed field — whether a
  direct child of the resolved items or a descendant beneath intermediate object
  fields — derives one normalized terminal load shape per distinct variant,
  projects and de-duplicates the terminal keys from the already-resolved items,
  and issues **one** set-based statement per shape, storing the partitioned
  result under that scope;
- a descendant path resolves identically to the direct path from the terminal
  child resolver's point of view: it reads the same
  `(scope, loader, shape, terminal key)` entry and cannot tell which site
  produced it;
- a terminal child resolver derives the **same** scope as the site that
  prefetched for it, so the entry is found;
- the same key reached under a **different** top-level response key is a
  different entry, reads `NotLoaded`, and is dispatched once for that scope;
- a child resolver whose `(scope, loader, shape, parent key)` entry is `Loaded`
  returns its bucket without any database access, including when the bucket is
  empty;
- a child resolver whose entry is `NotLoaded` performs its ordinary direct query
  and returns the same result it returns today;
- two aliases of the same field with the same normalized shape return identical
  results from one dispatch; two aliases with different shapes return each
  shape's correct result from one dispatch each;
- an omitted argument and an explicitly supplied schema default resolve against
  the same entry;
- results are identical, element for element and in order, to the direct
  per-parent path.

### 6.2 Failure behaviour

- a failed set-based statement is recorded as `LoadFailed` once per
  `(scope, loader, shape)` dispatch, covering the attempted key set, and does not
  affect any other scope;
- the parent list resolver still returns its parents successfully; the prefetch
  failure does not become the parent list field's error;
- each covered child resolver returns a `FieldError` derived from the recorded
  failure, with the same `extensions.type` classification the direct path
  produces;
- **no retry query is issued** for any covered key;
- `LoadFailed` is never represented as absence and never as `Loaded([])`;
- no empty list, unfiltered result, silent per-parent retry, or swallowed error
  is permitted on any path;
- the GraphQL-visible equivalence contract of `ADR-0006` section 4.9.3 —
  `errors[].path`, null propagation, `extensions.type`, no empty substitution, no
  extra SQL — must be verified, not asserted. Any intentional difference must be
  documented explicitly rather than hidden.

### 6.3 Authorization

- the mechanism makes no authorization decision and changes no policy;
- prefetch key sets contain only keys taken from parents the request has already
  resolved and the parent resolver has already authorized;
- the loader is not a general "load by id" facility and must not accept keys
  from user input;
- if the proof consumer touches data protected at the child level rather than
  inherited from the parent, the authorization tests of section 10 are mandatory
  rather than conditional.

### 6.4 Concurrency and idempotency

- two concurrent independent GraphQL requests must not observe each other's
  store contents, and this must be proven by test rather than argued from the
  construction site;
- a repeated prefetch for an already-present `(scope, loader, shape, key)` entry
  does not re-query it, including when the repeat comes from a *different*
  prefetch site **under the same execution scope**. Two prefetch sites covering
  the same loader under *different* scopes are two entries, and the second
  dispatch there is correct and required;
- the mechanism introduces no lock, lease, claim or background job.

### 6.5 Compatibility

- no public GraphQL **schema** contract change; the generated SDL must be
  byte-identical to the base;
- **the set of accepted requests does change**, narrowly and deliberately: a
  mutation operation with a duplicate executable top-level response key is now
  rejected (`ADR-0006` section 4.12.6.7). This must be stated plainly in the
  implementation report and must not be described as a schema change, nor as
  ordinary spec-conformant validation;
- no database, migration or `schema.rs` change;
- no `thoth-client` or downstream generated-client impact. The implementing agent
  must confirm this by inspection rather than assumption, since `thoth-client`
  issues GraphQL requests;
- no dependency change;
- existing GraphQL tests must pass **unmodified**. Editing an existing test to
  accommodate the mechanism is a signal that behaviour changed and requires
  explicit justification in the implementation report.

### 6.6 Central mutation request guard

Binding behaviour, over and above the construction rules in section 3 item 0.

**Accepted, and unchanged from the base:**

```graphql
mutation {
  first:  updateA(...) { ... }
  second: updateA(...) { ... }
}
```

Distinct top-level response keys are allowed, and each executes exactly once.

**Rejected — direct duplicate:**

```graphql
mutation {
  x: updateA(...) { a }
  x: updateA(...) { b }
}
```

One response key, two executable occurrences: rejected before any resolver runs.

**Rejected — duplicate through a named fragment spread**, and **rejected —
duplicate through an inline fragment**: an equivalent duplicate introduced
indirectly is rejected identically. Fragment expansion is not optional.

**Directives.** `@skip`, `@include` and variable-driven conditions must be proven
in both directions. A syntactic duplicate that is *definitely excluded* for the
concrete request must **not** be rejected — it is not an executable occurrence.
Where the condition cannot be resolved, rejection is conservative and that
tradeoff is recorded (`ADR-0006` section 4.12.6.7); it must not be presented as
exact.

**Resolver execution count.** For any rejected operation:

```text
mutation resolver execution count = 0
database write count             = 0
```

This is mandatory and must be **measured**. A guard that rejects only after the
first resolver has executed is non-compliant.

**Query operations.** Duplicate compatible response keys remain allowed in
queries and keep ordinary GraphQL/Juniper behaviour. The restriction must not be
broadened to queries, to non-top-level selections, or to distinct aliases.

**Failure behaviour.** HTTP status and response body follow the repository's
existing GraphQL request-validation failure convention: `is_ok()` is `false`, the
existing handler branch returns HTTP 400, and the body is the ordinary
`{"errors":[{"message","locations"}]}` shape with no `data` key. No resolver
executes, no database write occurs, and no partial mutation execution is
possible.

**Prerequisite coupling.** The store must be unavailable whenever the guard is
not applied (`ADR-0006` section 4.12.6.6). Because a nested resolver cannot
detect operation type, this is all-or-nothing: with the guard disabled, every
prefetch site performs no prefetch and every lookup reads `NotLoaded`, so every
path takes its always-correct direct fallback. The implementation must make
"batching on, guard off" **unrepresentable**, not merely discouraged.

---

## 7. Data and migration requirements

Migration required: **NO**

No table, column, enum, index, constraint or trigger is added or altered.
`thoth-api/src/schema.rs` and `thoth-api/migrations/` must be byte-identical to
the base. Per `ADR-0003` section 4.3 and `thoth-api/AGENTS.md` section 4, the
absence of a `schema.rs` change is recorded here as an explicit reviewed
conclusion, not an omission.

Stop condition: if implementation finds a migration is required, stop and report
`BLOCKED` (section 13).

---

## 8. Observability and operations

Required logs: none. The foundation adds no production logging.

Required metrics/alerts: none. Query-count observation is a test-time concern.

Operational runbook changes: none.

Deliberate: adding production instrumentation for a mechanism no production
field yet uses would be unmeasurable noise. Instrumentation for adopted fields is
the adopting task's concern.

---

## 9. Acceptance criteria

- [ ] `ADR-0006` is `APPROVED` and its approved content is reachable from
      `develop` before implementation begins.

**Central mutation request guard (`ADR-0006` section 4.12.6; section 6.6)**

- [ ] A mutation with a duplicate executable top-level response key written
      directly is rejected before execution.
- [ ] The same duplicate introduced through a **named fragment spread** is
      rejected.
- [ ] The same duplicate introduced through an **inline fragment** is rejected.
- [ ] For every rejected case, the **measured** mutation resolver execution count
      is `0` and the **measured** database write count is `0`.
- [ ] Distinct top-level mutation aliases are accepted and each executes exactly
      once.
- [ ] A duplicate that `@skip`/`@include` definitely excludes for the concrete
      request is **accepted** and executes once — proven for literal conditions
      and for variable-driven conditions, in both directions.
- [ ] A duplicate compatible response key in a **query** operation is accepted and
      is unaffected by the guard.
- [ ] Duplicate response keys **below** the top level of a mutation are accepted
      and unaffected.
- [ ] The rejection's HTTP status and serialized body are compared against a real
      juniper validation failure and match it; no new handler branch exists.
- [ ] The rejection message exposes no loader, store or scope internals.
- [ ] The guard makes no authorization decision, and `GraphQLRequest::execute` is
      not replaced.
- [ ] None of the 88 `MutationRoot` resolver methods is modified.
- [ ] Exactly one warning-level log record is emitted per rejection, carrying no
      document, variables or argument values.
- [ ] The guard runs under **both** the async `GraphQLRequest::execute` path and
      the `execute_sync` test path.
- [ ] With the guard disabled by configuration, the store is unavailable: every
      lookup reads `NotLoaded`, every path takes the direct fallback, and results
      are unchanged. "Batching on, guard off" is unrepresentable.

**Store and scoping**

- [ ] The request-scoped store exists on `Context`, is initialised empty per
      construction, and `Context` remains `Sync`.
- [ ] Store contents are unique per GraphQL request; concurrent independent
      requests share nothing, proven by test.
- [ ] No global, static or cross-request cache or singleton exists, verified by
      inspection of the added module.
- [ ] The set-based loader issues exactly one statement per
      `(scope, loader, shape)` dispatch, using `.eq_any(...)`, and never iterates
      keys issuing per-key statements.
- [ ] Store identity is
      `(top-level response key, loader identity, normalized load shape, parent key)`;
      the load shape is a typed loader-specific value, not a serialized argument
      string.
- [ ] One loader-owned shape constructor is used by both the prefetch site and
      the child lookup.
- [ ] An omitted argument and an explicitly supplied schema default produce the
      same shape and resolve against the same entry.
- [ ] Semantically different argument variants never share a stored entry, and
      each alias returns the correct result for its own shape.
- [ ] Distinct shapes produce exactly one dispatch each — not one per parent, and
      not one shared dispatch across variants.
- [ ] Prefetch sites enumerate child selections via `children()` filtered on
      `field_original_name()`, never via `select()` / `has_child()`, so aliases
      are found.
- [ ] For a covered parent list of size `n`, the measured child-statement count
      does **not** scale linearly with `n`.
- [ ] That measurement is recorded for at least **two distinct values of `n`**,
      reported as `parent count | prefetch child-query count | direct baseline
      child-query count`, with the prefetched count bounded while the baseline
      grows.
- [ ] Statement counts are measured through a pool constructed **after** the
      instrumentation hook is installed, never through the existing
      `OnceLock<Arc<PgPool>>` test pool.
- [ ] Duplicate parent keys produce one key in the statement and no additional
      statements.
- [ ] Repeated aliases of the same normalized shape on the same parent cause no
      additional statements.
- [ ] A second prefetch site covering an already-loaded
      `(scope, loader, shape, key)` set issues no additional SQL **within that
      scope**; across distinct scopes a second bounded dispatch is correct and
      expected.
- [ ] The store distinguishes `NotLoaded`, `Loaded` (including `Loaded([])`) and
      `LoadFailed`.
- [ ] `NotLoaded` falls back to the direct query and returns the correct result.
- [ ] `Loaded([])` returns empty **without** a database query and is never
      treated as a miss.
- [ ] `LoadFailed` returns a field error, issues **no** retry query, and is never
      treated as absence or as an empty result.
- [ ] A mixed present/absent key set resolves every parent correctly.
- [ ] A prefetch failure does **not** fail the parent list field; the parent list
      resolves and the error surfaces at the child field.
- [ ] The GraphQL-visible error contract is verified against the direct path —
      `errors[].path`, null propagation and `extensions.type` — with any
      intentional difference documented explicitly.
- [ ] Prefetched results equal direct per-parent results, element for element
      and in order.
- [ ] Whole-store invalidation clears `Loaded` and `LoadFailed` state across all
      scopes, all loaders, all shapes and all keys.
- [ ] Ordinary correctness — including mutation read-after-write — does **not**
      depend on any mutation resolver invoking invalidation, and no production
      mutation resolver is modified.

#### Top-level response-key scoping (`ADR-0006` section 4.12)

- [ ] The store is keyed by
      `(top-level response key, loader identity, normalized load shape, parent key)`.
- [ ] The prefetch site and its terminal resolvers derive **identical** scope
      values on both direct and descendant paths.
- [ ] No loader entry, successful or failed, is visible across two top-level
      response keys.
- [ ] Two top-level aliases of the same schema field
      (`a: publishers { ... } b: publishers { ... }`) produce **separate** loader
      namespaces.
- [ ] Repeated occurrences of one response key share one scope, and no
      source-position or AST-occurrence component is part of the scope key.
- [ ] No resolver detects operation type, and the raw GraphQL document is never
      parsed to derive scope.

##### Compatibility shim — path extraction

- [ ] A top-level field **without** an alias returns its field response key.
- [ ] A top-level **aliased** field returns the alias.
- [ ] A nested **direct child** returns the same first path segment as its parent
      site.
- [ ] A **deeply nested descendant** returns the same first path segment.
- [ ] **Inline fragments** preserve the same first scope.
- [ ] **Named fragments** preserve the same first scope.
- [ ] Direct prefetch site, descendant prefetch site and terminal resolver all
      derive identical scope values within one top-level field.

##### Compatibility shim — side effects

- [ ] Calling the scope helper adds **no** GraphQL error.
- [ ] It changes **no** `errors[]` entry.
- [ ] It changes **no** result data.
- [ ] It has **no** database side effect.

##### Compatibility shim — failure and isolation

- [ ] If the scope cannot be derived the helper **fails closed**: the prefetch is
      skipped, lookups read `NotLoaded`, and the field falls back to its direct
      query. No shared or request-global namespace is substituted, and the parent
      list field does not fail.
- [ ] `Executor::new_error(..)` path extraction appears in **exactly one** module;
      no loader, prefetch site or resolver uses the technique directly.
- [ ] The shim's module documentation and tests state its coupling to the pinned
      Juniper API and the revalidation obligation of `ADR-0006` section 4.12.14.
- [ ] No package dependency was added to implement the shim.

##### Store collision matrix

- [ ] Same parent key + same loader + same shape under **different scopes** never
      collides.
- [ ] Same scope + same parent key + **different loaders** never collides.
- [ ] Same scope + same loader + same parent + **different shapes** never
      collides.
- [ ] The same full key returns the expected stored value.
- [ ] A `LoadFailed` recorded under scope `A` does **not** poison scope `B`; the
      lookup under `B` is `NotLoaded` and falls back.
- [ ] Whole-store invalidation clears both scopes.

#### Mutation behaviour (`ADR-0006` sections 4.12.10-4.12.12)

- [ ] Read-after-write holds **within one top-level mutation field**: a test-only
      mutation that writes child data and then selects the affected loader-backed
      field in the same operation returns the written value, not a prefetched one.
- [ ] Batching inside a mutation payload fan-out works, and stays bounded as the
      nested parent count rises.
- [ ] Two top-level mutation fields in one operation are isolated: entries created
      under the first cannot satisfy the second, and the second's nested selection
      observes the second write rather than cached state from the first.
- [ ] That isolation holds **under async execution where the executor may
      interleave the two top-level futures**, and is proven by scope isolation
      rather than by relying on execution order.
- [ ] No production mutation resolver is modified.

#### Descendant prefetch (`ADR-0006` section 4.19)

- [ ] A **direct** path (`list -> loader-backed child`) batches correctly.
- [ ] An **indirect** path (`list -> intermediate object -> loader-backed
      descendant`) batches correctly.
- [ ] Aliases at **intermediate** path segments are matched by
      `field_original_name()` and do not defeat detection.
- [ ] Aliases at the **terminal** segment are matched the same way.
- [ ] **Every** matching terminal selection is discovered, including two
      intermediate branches each carrying a terminal selection; traversal stops
      at no level's first match.
- [ ] Terminal load shapes normalize correctly, and are constructed from the
      terminal selection rather than from an ancestor selection.
- [ ] The descendant key projector derives keys **only** from data already
      present on the already-resolved parent items.
- [ ] Duplicate projected terminal keys are de-duplicated before dispatch.
- [ ] A second ancestor or list site does not reload an already-loaded terminal
      key within the same valid cache scope; results are stored under the
      ordinary terminal identity, with **no** separate namespace for indirectly
      prefetched entries.
- [ ] Indirect prefetch never bypasses an intermediate authorization decision
      (`ADR-0006` section 4.19.4).
- [ ] The terminal child resolver issues **no** fallback statement on a covered
      descendant path.
- [ ] The terminal statement count stays bounded as the ancestor list grows,
      measured at two distinct list sizes.
- [ ] Statements issued by the **intermediate** resolver are reported as a
      separate figure and are **not** presented as part of terminal-loader
      compliance (`ADR-0006` section 4.19.5).
- [ ] The mechanism is proven under both `juniper::execute_sync` and
      asynchronous `execute`.
- [ ] The generated GraphQL SDL is byte-identical to the base.
- [ ] `thoth-api/src/schema.rs`, `thoth-api/migrations/`, `Cargo.toml` and
      `Cargo.lock` are byte-identical to the base.
- [ ] `thoth-api/src/policy.rs` is unchanged.
- [ ] No existing child resolver is modified, and no argument is added to any
      production field.
- [ ] The foundation declares **no** production field N+1 compliant; the
      adoption-coverage obligations of `ADR-0006` section 4.18.2 attach to
      adopting tasks.
- [ ] Existing GraphQL tests pass unmodified.
- [ ] `BE-02` remains unimplemented: no `DistributionPlatform`, no
      `publisher_distribution_platform`, no `BE-02` GraphQL field.
- [ ] PR #788 and issue #765 are unmodified.

---

## 10. Required tests

### Central mutation request guard

Real GraphQL tests, exercised through both `juniper::execute_sync` and the async
`GraphQLRequest::execute` harness (`thoth-api/tests/support/mod.rs:108`). Test-only
mutations and types; **no production mutation resolver may be modified**.

- **allowed — distinct aliases**

  ```graphql
  mutation {
    first:  updateA(...) { ... }
    second: updateA(...) { ... }
  }
  ```

  accepted; each executes exactly once;
- **rejected — direct duplicate**

  ```graphql
  mutation {
    x: updateA(...) { a }
    x: updateA(...) { b }
  }
  ```

  rejected before resolver execution;
- **rejected — duplicate through a named fragment** — the equivalent duplicate
  introduced by a fragment spread is rejected;
- **rejected — duplicate through an inline fragment** — likewise;
- **directives** — `@skip`, `@include` and variable-driven conditions, proven in
  both directions. A syntactic duplicate that is definitely excluded for the
  concrete request must be **accepted** and execute once, and must not be
  misclassified. An undecidable condition rejects conservatively, and the test
  must assert that recorded tradeoff rather than an exact result;
- **resolver execution count** — for each rejected operation, a resolver-call
  counter on the test-only mutation reads `0` **and** the statement-count harness
  of section 10's performance tests reads `0` database writes. Both are required;
  neither alone is sufficient;
- **query operations** — a duplicate compatible response key in a query is
  accepted, both occurrences share one scope, and the terminal loader issues no
  additional statement for the second;
- **nested duplicates** — duplicate response keys below the top level of a
  mutation are accepted and unaffected;
- **error shape parity** — the rejection's HTTP status and serialized body are
  compared directly against a real juniper validation failure (for example an
  unknown field) and match: `is_ok()` false, HTTP 400, an `errors` array of
  `{message, locations}`, and no `data` key;
- **operation selection** — with several named operations in one document and an
  `operationName` supplied, the guard evaluates the **selected** operation only;
- **guard disabled** — with the kill switch off, the previously rejected
  documents are accepted again **and** the store is unavailable: every lookup
  reads `NotLoaded`, every path falls back, and results are unchanged.

### Unit — state model

- `NotLoaded` is representationally distinct from `Loaded([])`;
- `LoadFailed` is representationally distinct from both;
- a failed dispatch never triggers the fallback;
- a successful empty bucket never triggers the fallback;
- a genuine miss does trigger the fallback;
- non-destructive read: two reads of the same entry return the same bucket.

### Unit — load shapes

- the same normalized shape de-duplicates to one entry and one dispatch;
- different argument values produce different shapes and do not collide;
- an omitted argument and an explicitly supplied schema default normalize to the
  same shape;
- one parent key may hold several shapes simultaneously, each correct;
- two loaders with identical parent key types cannot read each other's entries;
- the shape type's equality is semantic — two shapes built from equivalent
  argument sets compare equal regardless of construction order or formatting.

### Unit — partitioning and store

- key de-duplication: `n` references to one key yield one key;
- partitioning determinism: identical rows and keys yield identical buckets,
  repeatedly;
- partitioning correctness: every returned row lands in the bucket for its own
  key and no other;
- whole-store invalidation clears `Loaded` and `LoadFailed` across all loaders
  and all shapes.

### Integration/database

Against the disposable test database, through Juniper execution:

- **single key** — a one-parent list resolves correctly;
- **many keys** — an `n`-parent list resolves every parent correctly;
- **duplicate keys** — a parent appearing more than once resolves correctly with
  no extra statement;
- **absent key** — a parent reached without a prefetch site falls back and
  returns the correct result;
- **mixed present/absent** — one operation containing both cases resolves both
  correctly;
- **same-shape aliases** — two aliases of the same field with the same normalized
  arguments yield one shape, one dispatch, and identical correct results for both
  aliases;
- **different-shape aliases** — two aliases of the same field with different
  arguments yield distinct shapes, one dispatch each, no cross-contamination, and
  the correct result for each alias;
- **default equivalence** — the omitted-argument and explicit-default forms
  resolve against the same entry and issue one dispatch between them;
- **same-scope multi-site reuse** — two prefetch sites covering the same
  `(scope, loader, shape)` **under the same execution scope** reuse the same
  entries, issue no duplicate SQL, and resolve every parent correctly;
- **cross-scope isolation** — the same two prefetch sites placed under
  *different* top-level response keys are two entries: each scope issues its own
  bounded dispatch, neither reads the other's entries, and every parent still
  resolves correctly. This test must be distinct from the one above, and a
  design that merges them is not compliant;
- **database error** — using the existing `failing_pool()` helper
  (`thoth-api/src/model/tests.rs:73`) or equivalent, the prefetch failure is
  stored as `LoadFailed`, the parent list field still resolves, each covered
  child field emits the failure, no retry SQL is issued, and no empty-list result
  is produced;
- **error contract** — the prefetched failure's `errors[].path`, null propagation
  and `extensions.type` are compared against the direct per-parent failure, with
  any intentional difference recorded;
- **equivalence** — prefetched output equals direct per-parent output, in order;
- **concurrent independent requests** — two `Context` values executing
  concurrently observe disjoint store contents;
- **request isolation** — a second request with a fresh `Context` observes an
  empty store regardless of what the first loaded;
- **read-after-write coherence within one top-level mutation field** — a
  test-only mutation writes data, returns a payload object, a nested list/fan-out
  resolver prefetches after the write, and the loader-backed terminal field
  receives the newly written value. Batching stays bounded as the nested parent
  count rises. Use test-only mutations and types; **no production mutation
  resolver may be modified**;
- **isolation across top-level mutation fields, distinct aliases** — one
  operation contains two distinct top-level response keys, the first creating
  loader state and the second writing different data and reading the same logical
  loader/key. The second nested selection must observe the second write, never
  the first scope's cached value. The assertion must rest on **scope isolation,
  not execution order**;
- **duplicate compatible response keys** — the same read-after-write scenario
  written with one duplicated top-level response key instead of two aliases must
  be **rejected by the guard**, with zero resolver executions and zero writes. A
  design that handles only the distinct-alias case and lets the duplicate through
  is **not compliant**: the duplicate is precisely the case that defeated the
  previous architecture (`ADR-0006` section 4.12.6.1);
- **fragment-created duplicates** — the same, with the duplicate introduced
  through a named fragment spread and through an inline fragment;
- **shared terminal fragment** — two top-level mutation fields with **distinct
  aliases** whose payload selections reach the terminal loader-backed field
  through **one shared named fragment**. Scope isolation must still hold, even
  though the terminal resolver's own source position is identical on both paths.
  This is the case that rejected an execution-occurrence scope
  (`ADR-0006` section 4.12.6.3), and it must be covered explicitly;
- **async interleaving** — the isolation above must hold under async `execute`
  where the executor may interleave the two top-level futures. Where practical,
  use a deliberately yielding async test-only mutation or resolver to force
  interleaving. If the pinned macro and test architecture cannot produce such a
  yielding fixture while preserving the sync-parity requirement of section 3.2,
  document the exact limitation and use the strongest available proof — but do
  **not** weaken the isolation invariant itself;
- **query scope isolation** — see the two-top-level-field query test in section
  9's scoping criteria; entries under one top-level query response key are never
  reused under another;
- **descendant path** — `list -> intermediate object -> loader-backed
  descendant` issues one terminal dispatch over the de-duplicated projected key
  set, returns results equal to the direct per-parent result in order, and issues
  no terminal fallback statement;
- **descendant alias matrix** — aliases at intermediate and terminal segments,
  and two aliased intermediate branches each carrying a terminal selection, are
  all detected, with every terminal selection collected;
- **descendant reuse** — a terminal key already loaded by one site is not
  re-dispatched by a second site, proving the single shared terminal namespace;
- **evidence separation** — the terminal-loader statement count and the
  intermediate resolver's statement count are reported as distinct figures on the
  same measured descendant path;
- **execution-path parity** — the same operation produces the same result and
  the same statement count under `execute_sync` and under async `execute`. The
  pinned Juniper serializes top-level fields on the sync path but drives them
  through `FuturesOrdered` on the async path (`ADR-0006` section 4.12.10), so
  parity must be demonstrated on both rather than inferred from the sync result.
  Under both paths, verify specifically:

  - the same top-level response-key derivation;
  - the same direct-path result;
  - the same descendant-path result;
  - the same alias behaviour;
  - the same scope isolation;
  - no GraphQL errors generated by scope extraction.

### Authorization/security

- keys are drawn only from already-resolved parents: an operation whose parent
  list is scope-restricted must not cause child rows for out-of-scope parents to
  be fetched or returned;
- if the proof consumer touches child-level protected data, the full negative
  matrix required by `thoth-api/AGENTS.md` section 7 applies — no
  authentication, wrong role, wrong publisher scope, correct scope, superuser.

### Regression

- the complete existing `thoth-api` GraphQL test suite passes **unmodified**;
- `cargo test --workspace` passes.

### Manual verification

- confirm the generated GraphQL SDL is byte-identical to the base, recording the
  exact command and diff result;
- confirm by `git diff --stat` that no runtime file outside the intended set,
  and no dependency manifest, changed.

### Performance

Wall-clock time is **not** an acceptance metric.

The acceptance signal is **SQL statement count and bounded database work**, per
`ADR-0006` section 8. Required evidence:

- statement counts observed at the driver via
  `diesel::connection::set_default_instrumentation`, or an equivalent method that
  observes actual SQL if instrumentation proves unworkable;
- application-level counters alone are insufficient, because they cannot detect
  a per-parent statement issued by a fallback path;
- counts recorded for at least two distinct `n`, reported **per top-level
  response scope**, per `ADR-0006` section 8.2:

  ```text
  top-level scope | parent count | prefetch terminal-query count |
  direct baseline terminal-query count | legacy intermediate-query count, if any
  ```

  with the prefetched count bounded within each scope while the direct baseline
  grows with `n`.

**Two-top-level-field query test.** Using the test-only schema, prove the
accepted cross-scope reuse tradeoff explicitly rather than hiding it
(`ADR-0006` section 4.12.13):

```graphql
query {
  first:  parents { childLoaderField }
  second: parents { childLoaderField }
}
```

Expected and required to be reported:

- `first` issues **one** bounded set-based child dispatch;
- `second` issues **one** bounded set-based child dispatch;
- results are correct under both;
- **no** cross-scope reuse occurs;
- the total is **2** dispatches for the two top-level scopes — **not** `N + N`;
- increasing the parent count within either field does **not** increase that
  field's child dispatch count.

The second dispatch is the accepted cost of `ADR-0006` section 4.12.13 and must
appear in the reported evidence.

**Mutation fan-out counts.** Exact statement-count evidence must also be produced
inside a mutation payload fan-out, not only inside a query fan-out, using the
same per-scope reporting unit.

Measurement-pool lifecycle, per `ADR-0006` section 8.1.1. The hook applies only
to connections established after installation, and the repository's ordinary test
pool is a process-wide `OnceLock<Arc<PgPool>>` that may already hold established
connections. Holding the test lock serializes tests; it does not recreate
connections. The measurement must therefore **not** use that singleton pool.

Required sequence:

1. acquire the existing exclusive database test lock (`test_lock()`);
2. reset and prepare the disposable test database;
3. install the instrumentation hook;
4. construct a **new dedicated pool** after hook installation;
5. run the measured operation through that pool;
6. count actual `StartQuery` events;
7. isolate the count from setup, fixture and migration statements.

If Diesel instrumentation proves unsuitable, any mechanism observing actual
PostgreSQL/Diesel SQL — for example PostgreSQL statement-log capture — is
acceptable. Narrative or application-counter-only evidence is not.

### Required commands

```bash
cargo test -p thoth-api --features backend
cargo test --workspace
cargo check --workspace
cargo clippy --all --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

Record exact commands and concise results. `passed` without the command and
result is not acceptable evidence.

---

## 11. Rollout

**Corrected during remediation.** This task may **no longer** claim that it has
no production effect at merge. The store does not, but the section 6.6 guard sits
on the common GraphQL request path and takes effect immediately. The sentence

```text
no production effect because no field adopts batching
```

is **false** for this task as specified and must not appear in the
implementation report, the PR body or the ADR.

- **initial state after merge:**
  - the **store**: present, adopted by no production field. No existing field's
    behaviour or statement count changes;
  - the **guard**: **active on every mutation request**, whether or not a
    loader-backed field is selected. From the merge commit, a mutation with a
    duplicate executable top-level response key is rejected with HTTP 400;
- **feature flag/configuration:** a **kill switch** is required, discharging
  `risk-classification.md`'s "feature flag, comparison mode or controlled pilot
  where possible" control for HIGH-risk work. A single boolean, **defaulting to
  enabled**, following the established `clap` `Arg::env(..)` pattern in
  `src/bin/arguments/mod.rs` and threaded through `start_server(..)` exactly as
  existing settings are. No new configuration mechanism may be invented. When
  disabled, the guard does not run **and the store is unavailable**
  (`ADR-0006` sections 4.12.6.6, 7.2.1) — the two cannot be decoupled, because a
  nested resolver cannot distinguish a mutation from a query. The switch exists
  to bound an unforeseen client compatibility problem, not to stage the rollout,
  and running with it disabled is an incident response rather than a supported
  operating mode;
- **staging/preview validation:** **required for merge**, limited to the guard.
  Confirm on a preview deployment that ordinary single-mutation and
  distinct-alias mutation traffic is unaffected, and that a duplicate top-level
  response key is rejected with HTTP 400 and the ordinary validation-error body.
  Store-side staging validation remains not required for merge and becomes
  required for the first adopting task;
- **pilot:** no pilot is proposed, and the reason is recorded rather than
  omitted: the guard's effect is a discrete accept/reject decision on a document
  shape, fully determined at the request boundary and fully covered by the
  section 10 tests, so a shadow-comparison deployment would add operational
  surface without adding evidence;
- **activation approval:** the **guard** activates at merge and is therefore
  covered by the CTO merge authorization itself. **Store** activation is not
  applicable to this task; the first adoption carries it;
- **observation period:** a short post-merge watch on the guard's rejection log
  (`ADR-0006` section 8.3). A sustained non-zero rejection rate is the signal
  that would justify the kill switch. No new alert or dashboard is created;
- **mass adoption:** prohibited. Existing child resolvers are unchanged and are
  migrated, if at all, under `ADR-0006` section 10.

---

## 12. Rollback

- **code rollback:** revert the merge commit. Nothing depends on the store at
  that point, and reverting the guard with it is correct — nothing is left
  depending on the guarantee it provides;
- **feature disable/kill switch:** disable the switch of section 11. This is the
  **immediate operational rollback and requires no deploy**: mutation requests
  are accepted exactly as before the merge, and the store is simultaneously
  unavailable, so no path is left depending on a guarantee no longer enforced.
  This replaces the previous "not applicable" entry, which is no longer true;
- **after a later adoption:** revert the adopting field to its direct per-parent
  query. Because that query is retained as the mandatory fallback, the field's
  *result* is unchanged by rollback — only its statement count is. The guard must
  **not** be reverted on its own once a field has adopted the store on a mutation
  path; the fail-closed coupling makes the store unavailable in that
  configuration rather than silently unsafe;
- **data rollback or forward repair:** none. The task creates no persistent
  state, and a rejected request performs no write;
- **external side-effect handling:** none. No external system is contacted.

---

## 13. Stop conditions

The implementing agent must stop and report `BLOCKED` if:

- `ADR-0006` is not `APPROVED`, or its approved content is not reachable from
  `develop`;
- explicit CTO implementation authorization, bound to a freshly verified exact
  `develop` head, has not been given;
- the approach of section 3.1 (a test-only GraphQL schema) proves technically
  impossible, so proving the mechanism would require adopting or modifying a
  production resolver;
- the mechanism cannot be made to work under **both** `execute_sync` and async
  `execute` without changing Juniper's execution model;
- the statement-count evidence of section 10 cannot be produced by any method
  that observes actual SQL;
- a database migration or a `thoth-api/src/schema.rs` change turns out to be
  required;
- a public GraphQL schema change turns out to be required;
- a new workspace dependency turns out to be required;
- keeping `Context: Sync` conflicts with the store, breaking the async execution
  path;
- read-after-write coherence within one top-level mutation field cannot be
  satisfied without editing production mutation resolvers. **Report `BLOCKED`
  rather than widening scope to touch the mutation resolvers**; the same applies
  if the load-shape, `LoadFailed` or descendant model turns out to require
  mutation-resolver changes;
- descendant prefetch cannot be expressed cleanly using stable pinned Juniper
  APIs plus already-resolved data. Report exactly:

  ```text
  BLOCKED - A2 CANNOT COVER INDIRECT FAN-OUT WITHOUT NEW EXECUTION ARCHITECTURE
  ```

  and do **not** push the unresolved architecture into `BE-02`;
- the top-level response key cannot be derived from a nested resolver through the
  compatibility shim of `ADR-0006` section 4.12.8 using only stable public
  Juniper API — for example if `Executor::new_error(..)` or
  `ExecutionError::path()` does not yield the execution path, or the first
  segment is not the top-level response key. **Report `BLOCKED`**; do not parse
  the raw GraphQL document, do not inspect private Juniper fields, do not use
  `unsafe`, and do not fall back to a request-global namespace;
- the scope helper cannot be made side-effect-free — for instance if
  materializing the path proves to mutate the executor's error state. Report
  `BLOCKED` rather than accepting a mechanism that alters the GraphQL response;
- scope isolation across two top-level mutation fields cannot be demonstrated
  under async execution. Report `BLOCKED` rather than weakening the isolation
  invariant. (Documenting a limitation in the *yielding-fixture* technique is
  permitted by section 10 and is not itself a stop condition; failing to
  establish the invariant at all is.);
- the scoping rule turns out to require modifying production mutation resolvers,
  detecting operation type at nested resolvers, or a `new_error(..)` call outside
  the single shim module. **Report `BLOCKED`** — each of these is prohibited by
  `ADR-0006` section 4.12;
- the central mutation request guard cannot be implemented at the request
  boundary using only public, non-`unsafe` pinned Juniper API — for example if
  `parse_document_source`, `get_operation`, `Operation::operation_type` or the
  public `Selection` AST proves insufficient to expand fragments and identify
  executable top-level occurrences. **Report `BLOCKED`**; do not modify Juniper,
  do not upgrade it, do not fork it, do not add a dependency, and do not
  retrofit the 88 `MutationRoot` resolvers;
- the guard cannot guarantee **zero** mutation resolver executions and **zero**
  database writes for a rejected operation. A guard that rejects after the first
  resolver has run is non-compliant and must be reported as `BLOCKED` rather than
  shipped;
- the guard cannot avoid rejecting a duplicate that `@skip`/`@include`
  definitely excludes for the concrete request, so ordinary conditional documents
  would be rejected. Report `BLOCKED` rather than broadening the compatibility
  restriction beyond `ADR-0006` section 4.12.6.7;
- the store cannot be made unavailable when the guard is not applied, so
  "batching on, guard off" would be a representable state. Report `BLOCKED`
  rather than shipping batching without its prerequisite;
- correcting the execution layer so that compatible repeated top-level fields
  execute once turns out to be required after all. That is option F3, rejected as
  architecture expansion in `ADR-0006` section 4.12.10.1. Report exactly:

  ```text
  BLOCKED - REQUIRES GRAPHQL EXECUTION-LAYER ARCHITECTURE OUTSIDE THIS TASK
  ```

  and do **not** begin maintaining a custom GraphQL executor or patching Juniper;
- the three-state store cannot be represented such that `NotLoaded`,
  `Loaded([])` and `LoadFailed` are mutually unambiguous;
- a prefetch failure cannot be surfaced at the child field without failing the
  parent list field, so the `ADR-0006` section 4.9.3 equivalence contract cannot
  be met and no acceptable documented difference exists;
- the normalized load shape cannot be constructed identically at the prefetch
  site and at the child lookup, so the two could drift;
- an argument-bearing test-only field cannot be added under `#[cfg(test)]`, so
  multi-shape behaviour could only be proven by adding arguments to a production
  field;
- approved architecture would need to change;
- the scope cannot be completed without unrelated changes;
- repository state differs materially from section 2.1.

The agent must report the stop condition rather than weakening an `ADR-0006`
invariant, adopting a production resolver on its own authority, substituting
narrative for query-count evidence, or shipping an unmeasured per-parent query.

---

## 14. Expected implementation report

The agent must use:

`docs/engineering/ai-delivery/implementation-report-template.md`

The report must additionally contain:

- the statement-count table reported **per top-level response scope** — scope,
  parent count, prefetch terminal-query count, direct baseline terminal-query
  count and legacy intermediate-query count — for at least two values of `n`, for
  **both** the direct and the descendant path, and inside **both** a query and a
  mutation payload fan-out;
- the two-top-level-field query result showing **2** dispatches rather than
  `N + N`, presented as the accepted `ADR-0006` section 4.12.13 tradeoff rather
  than omitted;
- for the descendant path, the **terminal-loader** statement count and the
  **intermediate-resolver** statement count as two separate figures, with an
  explicit statement that bounding the terminal loader does not make the
  operation globally free of N+1 access (`ADR-0006` section 4.19.5);
- the descendant prefetch representation actually implemented — how the selection
  path, terminal loader identity, terminal shape constructor and key projector
  are expressed — and the stable Juniper APIs relied on for recursive,
  alias-safe traversal;
- confirmation that indirectly prefetched entries are stored under the ordinary
  terminal identity, with no separate cache namespace;
- the compatibility shim as implemented: its exact signature, the Juniper APIs
  it calls, the evidence that it is side-effect-free, its fail-closed behaviour,
  and confirmation that it is the **only** site in the codebase using
  `new_error(..)` path extraction;
- confirmation that the shim's module documentation and tests record the pinned
  Juniper coupling and the section 4.12.14 revalidation obligation, and that no
  package dependency was added for it;
- the scope-isolation evidence: identical scope derivation at prefetch site and
  terminal resolver, two top-level aliases of one schema field producing separate
  namespaces, `LoadFailed` under one scope not poisoning another, and the store
  collision matrix results;
- the mutation evidence: read-after-write within one top-level mutation field,
  isolation across two top-level mutation fields, the async-interleaving proof
  (or the documented limitation of the yielding-fixture technique and the
  strongest proof achieved in its place), and confirmation that **no production
  mutation resolver was modified**;
- the exact method used to observe SQL statements, **including how the measured
  pool was constructed relative to the instrumentation hook**;
- the load-shape type for each loader implemented, and how defaults normalize;
- the observed GraphQL error contract comparison (`errors[].path`, null
  propagation, `extensions.type`) between the prefetched and direct failure
  paths, with any intentional difference stated explicitly;
- the **guard** as implemented: its exact placement in the request path, the
  public Juniper APIs it calls, how it expands named and inline fragments, how it
  evaluates `@skip`/`@include` against coerced variables, how it handles an
  undecidable condition, and confirmation that it does not replace
  `GraphQLRequest::execute` and makes no authorization decision;
- the guard's **measured** zero-execution evidence: resolver call counts and
  database write counts for every rejected case — direct duplicate, named
  fragment duplicate and inline fragment duplicate — under both execution paths;
- the guard's directive matrix results, showing which duplicates are accepted and
  which rejected, and stating the conservative-rejection tradeoff explicitly
  rather than implying exactness;
- the observed HTTP status and serialized body of a guard rejection, compared
  side by side with a real juniper validation failure;
- confirmation that queries, non-top-level selections and distinct aliases are
  unaffected, and that none of the 88 `MutationRoot` resolvers was modified;
- the kill switch as implemented, and the evidence that with it disabled the
  store is unavailable and every path falls back — that is, that "batching on,
  guard off" is unrepresentable;
- an explicit, accurate **production activation boundary** statement. The report
  must say that the **store** is adopted by no production field at merge, **and**
  that the **guard** is live on every mutation request from the merge commit. It
  must **not** claim "no production effect because no field adopts batching";
  that claim is false for this task (section 11);
- an explicit statement that the foundation declares **no** production field N+1
  compliant, and that adoption-coverage obligations attach to adopting tasks;
- the generated-SDL comparison result;
- confirmation that `schema.rs`, `migrations/`, `Cargo.toml`, `Cargo.lock` and
  `policy.rs` are unchanged;
- confirmation that no existing child resolver was modified;
- the reviewed conclusion that no migration was required.

---

## 15. Recommended execution

Implementation model: strongest available Codex coding model
Reasoning level: HIGH, using maximum practical reasoning for the store lifetime,
load-shape normalization, partitioning, failure-state and coherence semantics.
Load-shape normalization warrants the highest care: it is the one part of this
design whose failure returns confidently wrong data rather than a miss, because
the correctness fallback does not cover a shape collision.
Independent reviewer: strongest available Claude model, per `model-selection.md`
section 3 (different model family from the implementer)
Review reasoning level: HIGH

Independent cross-model review is mandatory. Explicit CTO merge authorization is
mandatory for this HIGH-risk runtime implementation. Production deployment
remains a separate, separately authorized event.

---

## 16. Branch and integration plan

- branch source: freshly verified `develop`
- pull-request target: `develop`
- task branch: `feature/shared-architecture/graphql-batching`
- expected merge order: after `ADR-0006` is approved and reachable from
  `develop`; before any `BE-02` runtime implementation
- parent programme branch refresh requirement: not applicable (STANDARD
  workflow)
- branch deletion after merge: YES
- final programme PR required: NO
- final release path: `develop -> master`

The implementation branch must not be created before explicit CTO implementation
authorization.

---

## 17. Approval

Approved for implementation by: not yet approved
Date: not yet approved
Notes: this specification requires independent review and explicit CTO approval,
and it depends on `ADR-0006` being approved and repository-authoritative.
Approval of this specification is not implementation authorization; that is a
separate explicit decision bound to a freshly verified exact `develop` head.

Record only the durable implementation authorization here. Independent review
decisions, CTO merge authorization and the merge itself are terminal GitHub
evidence under `ADR-0005` and must not be copied back into this file.

---

## 18. Relationship to `BE-02`

`BE-02` is a **dependent** of this task, not a dependency of it.

```text
ADR-0006 approved + repository-authoritative
        |
        v
THOTH-GQL-BATCH-01 implemented, independently reviewed,
CTO merge-authorized and merged into develop
        |
        v
BE-02 specification amended on its existing PR #788 to replace the open
N+1 architecture gate with the approved mechanism and dependency
        |
        v
fresh independent exact-head review of PR #788
        |
        v
explicit CTO approval of the BE-02 specification
        |
        v
fresh develop verification + separate CTO BE-02 implementation authorization
```

This task must not implement, prepare or anticipate any part of `BE-02`, and
must not modify PR #788 or its branch. `BE-02` runtime implementation remains
blocked and unauthorized throughout.

### 18.1 What `BE-02` inherits, and what it must do itself

`BE-02` inherits from this foundation:

- the mechanism, the three-state store, the load-shape contract and the
  measurement approach;
- the recorded fact that `Publisher.distributionPlatforms` has **no** field
  arguments in its approved contract, so its load shape is `Unit`
  (`ADR-0006` section 4.4.5). This specification adds no argument to that field
  and changes nothing in the approved `BE-02` API contract;
- proof that several prefetch sites can cover one `(scope, loader, shape)` under
  one execution scope without duplicate loading, and that the same sites under
  distinct scopes are correctly isolated;
- the central mutation request guard, which protects `BE-02`'s mutation-payload
  paths and which `BE-02` must not reimplement or weaken.

`BE-02` must do for itself, and this task must **not** attempt on its behalf:

- the exact-base fan-out path inventory for `Publisher.distributionPlatforms`
  required by `ADR-0006` section 4.18.2, searching its own implementation base
  rather than treating the minimum investigation set in `ADR-0006` section 4.18.3
  as exhaustive;
- installing a prefetch site on every material fan-out path, or escalating if
  compliant coverage would need architecture outside its approved scope;
- per-path statement-count evidence.

The foundation therefore leaves `BE-02` a mechanism and an obligation. It does
not leave it a compliance claim.
