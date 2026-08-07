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
- there is no SQL statement-count test facility. Diesel 2.3.10 provides
  `set_default_instrumentation`, which is unused here.

The implementing agent must refresh all of the above against its own exact base
before editing.

---

## 3. Explicit scope

The task must:

1. add the request-scoped store to `crate::graphql::model::Context` per
   `ADR-0006` sections 4.1-4.4, initialised empty at construction, keeping
   `Context: Sync` so the async execution path continues to compile;
2. add a focused module under `thoth-api/src/graphql/` containing the store, the
   closed loader-identity discriminant, the set-based loader contract, key
   de-duplication, deterministic result partitioning, and the look-ahead-driven
   prefetch helper;
3. implement the loader contract as a **single** set-based statement per
   dispatch, using Diesel `.eq_any(...)` (`WHERE key = ANY(...)`), returning raw
   canonical model rows rather than GraphQL objects (`ADR-0006` section 4.5);
4. implement the store-read path used by a child resolver, distinguishing
   "loaded, empty" from "not loaded", with the mandatory direct-query fallback on
   a miss (`ADR-0006` section 4.7);
5. implement duplicate-key handling, repeated-alias handling and the
   non-destructive read (`ADR-0006` section 4.6);
6. implement the fail-closed error behaviour, leaving affected keys **absent**
   from the store on failure (`ADR-0006` section 4.9);
7. add the explicit invalidation entry point required by `ADR-0006` section
   4.12, unused by this task;
8. add the SQL statement-count test facility required by `ADR-0006` section 8.1,
   observing statements at the driver, serialised against other database tests
   through the existing exclusive test lock;
9. prove the mechanism end to end through Juniper execution using a
   **test-only** GraphQL root and object type defined under `#[cfg(test)]`,
   against an existing table, exercising real look-ahead, real set-based SQL and
   real partitioning;
10. update `thoth-api-server/src/lib.rs` only if `Context::new`'s signature
    changes; prefer initialising the store internally so it does not;
11. add the tests of section 10 and the query-count evidence of section 9.

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
5. migrate, refactor or "improve" any of the 63 existing child resolvers;
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
5. loader output is deterministic for a given input key set;
6. duplicate keys cause no duplicate backend fetches within one request;
7. per-key ordering matches the owning field's contract and the direct
   per-parent result exactly;
8. no public GraphQL schema change;
9. no global, static or cross-request cache, and no singleton;
10. no stale read-after-write result within one operation;
11. a loader-backed field is correct whether or not a prefetch ran;
12. non-adopting fields are behaviourally unchanged.

---

## 6. Required behaviour

### 6.1 Success behaviour

- a parent list resolver that opts in consults look-ahead and, when the
  loader-backed child field is selected, issues **one** set-based statement for
  the de-duplicated parent keys and stores the partitioned result;
- a child resolver on a stored parent key returns its bucket without any
  database access;
- a child resolver on a parent key absent from the store performs its ordinary
  direct query and returns the same result it returns today;
- results are identical, element for element and in order, to the direct
  per-parent path.

### 6.2 Failure behaviour

- a failed set-based statement propagates as a GraphQL field error, exactly as
  the direct per-parent query's error would;
- the affected keys are left **absent** from the store, never present-and-empty;
- no empty list, unfiltered result, silent per-parent retry, or swallowed error
  is permitted on any path.

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
- a repeated prefetch for an already-present key does not re-query it;
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
- [ ] The set-based loader issues exactly one statement per dispatch, using
      `.eq_any(...)`, and never iterates keys issuing per-key statements.
- [ ] For a covered parent list of size `n`, the measured child-statement count
      does **not** scale linearly with `n`.
- [ ] That measurement is recorded for at least **two distinct values of `n`**,
      with the per-parent baseline recorded alongside.
- [ ] Duplicate parent keys produce one key in the statement and no additional
      statements.
- [ ] Repeated aliases for the same field on the same parent cause no additional
      statements.
- [ ] A key absent from the store falls back to the direct query and returns the
      correct result.
- [ ] A key present with an empty bucket returns empty **without** a database
      query.
- [ ] A mixed present/absent key set resolves every parent correctly.
- [ ] A database failure propagates as a field error, leaves affected keys
      absent from the store, and never yields an empty or unfiltered result.
- [ ] Prefetched results equal direct per-parent results, element for element
      and in order.
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
- [ ] No existing child resolver is modified.
- [ ] Existing GraphQL tests pass unmodified.
- [ ] `BE-02` remains unimplemented: no `DistributionPlatform`, no
      `publisher_distribution_platform`, no `BE-02` GraphQL field.
- [ ] PR #788 and issue #765 are unmodified.

---

## 10. Required tests

### Unit

- key de-duplication: `n` references to one key yield one key;
- partitioning determinism: identical rows and keys yield identical buckets,
  repeatedly;
- partitioning correctness: every returned row lands in the bucket for its own
  key and no other;
- "loaded, empty" is representationally distinct from "not loaded";
- non-destructive read: two reads of the same key return the same bucket;
- loader-identity separation: two loaders using the same parent key type cannot
  read each other's entries;
- the invalidation entry point empties the store.

### Integration/database

Against the disposable test database, through Juniper execution:

- **single key** — a one-parent list resolves correctly;
- **many keys** — an `n`-parent list resolves every parent correctly;
- **duplicate keys** — a parent appearing more than once resolves correctly with
  no extra statement;
- **absent key** — a parent reached without a prefetch site falls back and
  returns the correct result;
- **mixed present/absent** — one operation containing both shapes resolves both
  correctly;
- **alias/repeated reference** — two aliases of the same field on the same
  parent both return the correct result with no extra statement;
- **database error** — using the existing `failing_pool()` helper
  (`thoth-api/src/model/tests.rs:73`) or equivalent, the prefetch failure
  surfaces as a field error, the affected keys are absent from the store, and no
  empty-list result is produced;
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
- counts recorded for at least two distinct `n`, with the per-parent baseline
  alongside;
- the counting test must install instrumentation before the pool it measures
  establishes connections, and must hold the existing exclusive database test
  lock.

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
  without editing mutation resolvers;
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
- the exact method used to observe SQL statements;
- the generated-SDL comparison result;
- confirmation that `schema.rs`, `migrations/`, `Cargo.toml`, `Cargo.lock` and
  `policy.rs` are unchanged;
- confirmation that no existing child resolver was modified;
- the reviewed conclusion that no migration was required.

---

## 15. Recommended execution

Implementation model: strongest available Codex coding model
Reasoning level: HIGH, using maximum practical reasoning for the store lifetime,
partitioning, error and coherence semantics
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
