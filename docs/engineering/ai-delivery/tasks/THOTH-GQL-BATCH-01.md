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

Give Thoth's GraphQL API a reusable, request-scoped mechanism by which a child
field on a list of parent objects can be resolved with a bounded, set-based
number of database statements instead of one statement per parent, so that new
nested fields can satisfy the `thoth-api/AGENTS.md` section 6 N+1 control by
following a repository pattern rather than escalating an architecture decision.

The task delivers the foundation and proves it. It adopts the foundation in no
production field.

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

### 1.2 Required HIGH-risk controls

Per `risk-classification.md` and `release-gates.md` section 1:

- approved design (`ADR-0006`) and this approved specification;
- implementation at high or maximum reasoning;
- independent cross-model review;
- failure-path and authorization tests;
- rollout and rollback plan;
- explicit CTO merge authorization;
- production activation, if any is ever required, separately authorized.

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

1. add the request-scoped store to `crate::graphql::model::Context` per
   `ADR-0006` sections 4.1-4.4, initialised empty at construction, keeping
   `Context: Sync` so the async execution path continues to compile;
2. add a focused module under `thoth-api/src/graphql/` containing the store, the
   closed loader-identity discriminant, the **typed load-shape contract**, the
   set-based loader contract, key de-duplication, deterministic result
   partitioning, failure recording, and the look-ahead-driven prefetch helper;
3. key the store by `(loader identity, normalized load shape, parent key)` per
   `ADR-0006` sections 4.4-4.4.4, with the load shape typed and loader-specific,
   never a serialized GraphQL argument string, and with a **single loader-owned
   shape constructor** used by both the prefetch site and the child lookup so the
   two cannot drift;
4. implement default normalization so an omitted argument and an explicitly
   supplied schema default produce the same shape, given that look-ahead does not
   apply schema defaults (`ADR-0006` section 4.4.3);
5. dispatch **once per unique `(loader identity, load shape)`** over the
   de-duplicated key set, never once per parent and never one dispatch shared
   across argument variants (`ADR-0006` section 4.4.4);
6. implement the loader contract as a **single** set-based statement per
   dispatch, using Diesel `.eq_any(...)` (`WHERE key = ANY(...)`), returning raw
   canonical model rows rather than GraphQL objects (`ADR-0006` section 4.5);
7. implement the three-state store — `NotLoaded`, `Loaded(Vec<V>)` including
   `Loaded([])`, and `LoadFailed` — with the child-resolver behaviour table of
   `ADR-0006` section 4.7, so that only `NotLoaded` triggers the direct-query
   fallback;
8. implement failure recording per `ADR-0006` section 4.9: the parent list
   resolver still returns its parents successfully, the failure is recorded once
   per `(loader, shape)` dispatch with the attempted key set, each covered child
   resolver returns the derived `FieldError`, and **no retry query is issued**.
   `ThothError` is not `Clone`, so retain a shareable representation;
9. implement duplicate-key handling, alias handling per shape and the
   non-destructive read (`ADR-0006` section 4.6), including that a second
   prefetch covering an already-loaded `(loader, shape, key)` set issues no
   additional SQL;
10. enumerate child selections by iterating `children()` and filtering on
    `field_original_name()`, never via `select()` / `has_child()`
    (`ADR-0006` section 4.15.1);
11. add the explicit **whole-store** invalidation entry point required by
    `ADR-0006` section 4.12 — clearing `Loaded` and `LoadFailed` state across all
    loaders and all shapes — unused by this task;
12. add the SQL statement-count test facility required by `ADR-0006` section
    8.1.1, using a **dedicated pool constructed after the instrumentation hook is
    installed**, under the existing exclusive database test lock, and isolating
    the measured operation's statements from setup and migration statements;
13. prove the mechanism end to end through Juniper execution using a
    **test-only** GraphQL root and object types defined under `#[cfg(test)]`,
    against existing tables, exercising real look-ahead, real set-based SQL and
    real partitioning. The fixture must include an **argument-bearing** test-only
    field so multi-shape behaviour is proven without adopting a production field
    (`ADR-0006` section 4.4.6), and must support **two prefetch sites** covering
    one loader so multi-site coverage is proven (`ADR-0006` section 4.18.3);
14. update `thoth-api-server/src/lib.rs` only if `Context::new`'s signature
    changes; prefer initialising the store internally so it does not;
15. add the tests of section 10 and the query-count evidence of section 9.

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
  `BE-02`.

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
13. activate any production feature, deploy, release, or run a production
    migration;
14. change CI workflows, repository settings or branch protection.

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
6. duplicate keys cause no duplicate backend fetches within one request, per
   `(loader, shape)`;
7. per-key ordering matches the owning field's contract and the direct
   per-parent result exactly;
8. no public GraphQL schema change;
9. no global, static or cross-request cache, and no singleton;
10. no stale read-after-write result within one operation;
11. a loader-backed field is correct whether or not a prefetch ran;
12. non-adopting fields are behaviourally unchanged;
13. store identity is `(loader identity, normalized load shape, parent key)`;
    argument variants never share an entry, and omitted arguments normalize to
    the schema default;
14. `NotLoaded`, `Loaded([])` and `LoadFailed` are distinguishable, and only
    `NotLoaded` triggers the fallback;
15. a failed prefetch does not fail the parent list field, and issues no retry
    SQL;
16. the existence of a correctness fallback is never treated as N+1 compliance
    evidence.

---

## 6. Required behaviour

### 6.1 Success behaviour

- a parent list resolver that opts in enumerates every requested selection of the
  loader-backed child field, derives one normalized load shape per distinct
  variant, and issues **one** set-based statement per shape over the
  de-duplicated parent keys, storing the partitioned result;
- a child resolver whose `(loader, shape, parent key)` entry is `Loaded` returns
  its bucket without any database access, including when the bucket is empty;
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
  `(loader, shape)` dispatch, covering the attempted key set;
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
- a repeated prefetch for an already-present `(loader, shape, key)` entry does
  not re-query it, including when the repeat comes from a *different* prefetch
  site in the same request;
- the mechanism introduces no lock, lease, claim or background job.

### 6.5 Compatibility

- no public GraphQL contract change; the generated SDL must be byte-identical to
  the base;
- no database, migration or `schema.rs` change;
- no `thoth-client` or downstream generated-client impact;
- no dependency change;
- existing GraphQL tests must pass **unmodified**. Editing an existing test to
  accommodate the mechanism is a signal that behaviour changed and requires
  explicit justification in the implementation report.

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
- [ ] The request-scoped store exists on `Context`, is initialised empty per
      construction, and `Context` remains `Sync`.
- [ ] Store contents are unique per GraphQL request; concurrent independent
      requests share nothing, proven by test.
- [ ] No global, static or cross-request cache or singleton exists, verified by
      inspection of the added module.
- [ ] The set-based loader issues exactly one statement per
      `(loader, shape)` dispatch, using `.eq_any(...)`, and never iterates keys
      issuing per-key statements.
- [ ] Store identity is `(loader identity, normalized load shape, parent key)`;
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
- [ ] A second prefetch site covering an already-loaded `(loader, shape, key)`
      set issues no additional SQL.
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
      loaders and all shapes.
- [ ] Read-after-write coherence holds: a mutation operation that writes and
      then selects the affected field in the same operation returns the written
      value.
- [ ] No prefetch site is reachable from a `MutationRoot` payload selection.
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
- **multi-site coverage** — two prefetch sites covering the same
  `(loader, shape)` in one request issue no duplicate SQL, and every parent
  resolves correctly;
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
- **read-after-write coherence** — a mutation operation writing child data and
  then selecting the affected field returns the written value;
- **execution-path parity** — the same operation produces the same result and
  the same statement count under `execute_sync` and under async `execute`.

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
- counts recorded for at least two distinct `n`, reported as:

  ```text
  parent count | prefetch child-query count | direct baseline child-query count
  ```

  with the prefetched count bounded while the direct baseline grows with `n`.

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

- **initial state after merge:** foundation only. No production field adopts the
  mechanism, so no production behaviour changes at merge;
- **feature flag/configuration:** none, and none required. Merging changes no
  existing field's behaviour, which is a stronger guarantee than a flag;
- **staging/preview validation:** not required for merge, since no production
  path is affected. It becomes required for the first adopting task;
- **pilot:** the first adoption is `BE-02`, in its own separately specified,
  separately reviewed and separately authorized task;
- **activation approval:** not applicable to this task; the first adoption
  carries it;
- **observation period:** not applicable to this task;
- **mass adoption:** prohibited. Existing child resolvers are unchanged and are
  migrated, if at all, under `ADR-0006` section 10.

---

## 12. Rollback

- **code rollback:** revert the merge commit. Nothing depends on the mechanism at
  that point;
- **after a later adoption:** revert the adopting field to its direct per-parent
  query. Because that query is retained as the mandatory fallback, the field's
  *result* is unchanged by rollback — only its statement count is;
- **data rollback or forward repair:** none. The task creates no persistent
  state;
- **feature disable/kill switch:** not applicable; there is nothing to disable
  until a field adopts the mechanism;
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
- read-after-write coherence cannot be satisfied under `ADR-0006` section 4.12
  without editing mutation resolvers. **Report `BLOCKED` rather than widening
  scope to touch the mutation resolvers**; the same applies if the corrected
  load-shape or `LoadFailed` model turns out to require mutation-resolver
  changes;
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

- the statement-count table: `n`, prefetched count, per-parent baseline count,
  for at least two values of `n`;
- the exact method used to observe SQL statements, **including how the measured
  pool was constructed relative to the instrumentation hook**;
- the load-shape type for each loader implemented, and how defaults normalize;
- the observed GraphQL error contract comparison (`errors[].path`, null
  propagation, `extensions.type`) between the prefetched and direct failure
  paths, with any intentional difference stated explicitly;
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
- proof that several prefetch sites can cover one `(loader, shape)` in a single
  request without duplicate loading.

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
