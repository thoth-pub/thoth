# ADR-0006 - Request-scoped GraphQL batching and set-based child loading

Status: PROPOSED
Date: 2026-08-07
Decision owner: CTO
Programmes affected: Shared Thoth GraphQL / backend architecture (owning
programme); Publisher Services and Distribution Configuration (first required
consumer); Thoth Metrics and any other programme resolving child fields through
`thoth-api` GraphQL
Repositories affected: `thoth-pub/thoth`
Supersedes: None
Superseded by: None

Direction: request-scoped batching / set-based loading selected in principle by
the CTO. Final repository decision requires independent review and explicit CTO
approval of this ADR.

Authority condition: this record is repository-authoritative when this exact
content is reachable from the repository's authoritative integration branch
(`develop`) **and** its status is `APPROVED`. Committing a `PROPOSED` ADR does
not approve it, and no implementation task may rely on it until it is approved
(`decisions/README.md`, "Authority"). Live independent-review,
merge-authorization, CI and merge evidence for the pull request carrying this
content is the GitHub pull-request record.

Verification base: every repository finding below was verified against `develop`
at `5a8c27b1b7c11a4f6bd26d459556468099f8c1f4`, and against the pinned dependency
sources resolved by the workspace `Cargo.lock` (`juniper` 0.16.2,
`juniper_codegen` 0.16.0, `diesel` 2.3.10). An implementing agent must refresh
these findings against its own exact base.

---

## 1. Context

### 1.1 The standing control

`thoth-api/AGENTS.md` section 6 requires that new GraphQL lists and reports:

```text
- avoid N+1 access;
- use set-based SQL or batched loaders;
```

The control is current, is not scoped to any one programme, and is not in
question here.

### 1.2 The gap

The repository provides no mechanism with which a new field can satisfy that
control. Verified at the base commit:

- `crate::graphql::model::Context` (`thoth-api/src/graphql/model.rs:54`) holds
  exactly four fields — `db: Arc<PgPool>`, `user: Option<IntrospectedUser>`,
  `s3_client: Arc<S3Client>`, `cloudfront_client: Arc<CloudFrontClient>`. There
  is no request-scoped accumulator, store or cache of any kind;
- there is no DataLoader, no `dataloader` dependency, no `batch_load` and no
  equivalent facility anywhere in the workspace;
- Juniper's look-ahead API (`Executor::look_ahead`, `LookAheadSelection`) is
  available in the pinned version but is not used anywhere in the repository;
- the established repository pattern for a child field on an object type is a
  per-parent database call. `Publisher.imprints` calls `Imprint::all(...)` with
  `Some(self.publisher_id)`; `Publisher.contacts` calls `Contact::all(...)` the
  same way; other child fields call `X::all(...)` or `X::from_id(...)` once per
  parent. `thoth-api/src/graphql/model.rs` contains 63 resolver methods taking
  `context: &Context`.

A new field therefore cannot satisfy section 6 by following an existing
repository pattern, because no compliant pattern exists.

### 1.3 How the gap surfaced

`BE-02` requires `Publisher.distributionPlatforms`, a child field on
`Publisher`. `Publisher` is returned by the **pre-existing** `publishers` root
query (`thoth-api/src/graphql/query.rs:521`), so the exposure is not confined to
`BE-02`'s own new root fields and cannot be removed by changing them:

```graphql
publishers(limit: 100) { publisherId distributionPlatforms { platform } }
```

`BE-02` sections 9.2.1 and 19.1 escalated this as a blocking architecture gate
rather than resolving it inside a programme task. That escalation was correct:
the decision is a shared GraphQL concern.

### 1.4 What is *not* claimed

This ADR does **not** claim that the existing child resolvers are proven
problematic. No measurement of them exists. Their per-parent access is an
observed structural fact; whether any particular one is a material production
cost is unmeasured, and section 10 keeps that question in evidence-led follow-up
work rather than settling it here.

### 1.5 Execution model actually in use

This is the decisive technical finding and it corrects a premise carried in
`BE-02` section 9.2.1, which described the repository as using Juniper 0.16
**sync** execution. The repository uses **both** execution paths:

| Site | Call | Path |
|---|---|---|
| production API (`thoth-api-server/src/lib.rs:98`) | `data.execute(&st, &ctx).await` | `juniper::http::GraphQLRequest::execute` — **async** |
| the entire GraphQL test suite (`thoth-api/src/graphql/tests.rs:52`) | `juniper::execute_sync(...)` | **sync** |

Any mechanism adopted here must therefore work identically under both. That
single constraint eliminates the conventional deferred-dispatch loader, as
section 3.1 shows.

---

## 2. Decision drivers

1. compliance with the `thoth-api/AGENTS.md` section 6 N+1 control, without
   waiving or narrowing it;
2. predictable database query growth as a parent list grows;
3. no cross-request state leakage under any circumstances;
4. no weakening of authorization or publisher scoping;
5. no public GraphQL contract change introduced by the batching foundation
   itself;
6. reusability by future nested fields across programmes;
7. compatibility with the pinned Juniper 0.16 / Diesel 2.3 architecture, under
   **both** the async production path and the sync test path;
8. bounded introduction — a foundation, not a wholesale refactor of historical
   resolvers;
9. measurable query-count evidence rather than narrative assertion;
10. safe error semantics — a database failure must fail closed, never degrade to
    an empty or unfiltered result;
11. safe mutation and read-after-write coherence semantics.

---

## 3. Options considered

### Option A - Request-scoped batching / set-based child loading

Chosen direction. The concrete mechanism within Option A is settled in section
4; the variants investigated are in section 3.1.

### Option B - External DataLoader dependency

Adopt a third-party loader crate.

Rejected for this decision. It adds a workspace dependency to solve a problem
that section 3.1 shows the pinned execution model cannot express through a
deferred-dispatch loader anyway, so the dependency would not by itself deliver
batching. Rejected without prejudice: a later ADR may supersede this one if the
execution model changes.

### Option C - Bounded N+1 exception

Grant `Publisher.distributionPlatforms` a documented exception to
`thoth-api/AGENTS.md` section 6, justified by the hard 17-row-per-publisher
bound.

Rejected. It deliberately waives a standing engineering control, and it leaves
the next nested field facing the same decision with a precedent for waiving it
again.

### Option D - Remove or defer the nested field

Rejected as a general solution. It is not an architecture for nested fields, it
is the absence of one. Specifically for `BE-02`, `Publisher.distributionPlatforms`
is required by the approved design and by `ADR-0002` section 4.4, so removing it
would contradict approved architecture.

### 3.1 Option A implementation variants investigated

The CTO's direction names an objective — request-scoped batching / set-based
loading — not a mechanism. Four mechanisms were investigated against the pinned
stack. Only one survives.

#### A1 - Request-scoped deferred-dispatch loader (conventional DataLoader shape)

Sibling child resolvers register a key and receive a future; the loader
dispatches one set-based query once all siblings have registered.

**Rejected: not implementable under the pinned execution model.** Three
independent findings, each sufficient on its own:

1. **Sync resolver bodies are evaluated eagerly, before any future is polled.**
   `juniper_codegen` 0.16.0 generates, for a non-`async` field
   (`graphql_object/mod.rs:719-721`):

   ```rust
   if !field.is_async {
       res = quote! { ::juniper::futures::future::ready(#res) };
   }
   ```

   The resolver body is the argument to `future::ready(..)`, so it runs to
   completion at future-construction time. Sibling child resolvers cannot
   accumulate keys and then defer: by the time a second sibling's future exists,
   the first has already returned its value.

2. **Converting a field to `async fn` breaks the sync execution path.** The same
   file (`graphql_object/mod.rs:629-636`) generates, for the sync
   `resolve_field` of an `async` field:

   ```rust
   ::core::panic!(
       "Tried to resolve async field `{}` on type `{}` with a sync resolver", ...
   );
   ```

   Every GraphQL test in the repository executes through
   `juniper::execute_sync`. An `async fn` resolver would panic in the test suite
   and in any other sync execution site.

3. **No dispatch trigger exists.** Even with async resolvers throughout,
   `resolve_into_list_async` (`juniper` 0.16.2,
   `src/types/containers.rs:585-586`) drives siblings with `FuturesOrdered`, and
   `resolve_selection_set_into_async_recursive` (`src/types/async_await.rs:196`)
   does the same for a selection set. Neither exposes a "all siblings are now
   pending" signal. A loader would have to infer the batch boundary from
   executor-internal polling behaviour, which is not part of Juniper's public
   contract.

   Making A1 work would require converting the resolver surface to `async fn`,
   converting the test harness off `execute_sync`, resolving blocking Diesel
   calls inside an async executor, and depending on unspecified polling
   behaviour — a GraphQL execution-architecture rewrite. That is outside this
   decision and is not required, because A2 works.

#### A2 - Look-ahead-driven set-based prefetch into request-scoped state

**Selected.** A resolver that returns a list of parents inspects the requested
selection set through `Executor::look_ahead()`, and when a registered
loader-backed child field is selected, issues **one** set-based query for all
parent keys and writes the partitioned result into request-scoped state on
`Context`. The child resolver reads its parent's entry from that state.

Verified viable:

- `Executor::look_ahead()` exists in the pinned version
  (`juniper` 0.16.2, `src/executor/mod.rs:694`) and returns a
  `LookAheadSelection` for the current field, with
  `LookAheadChildren::has_child(name)` / `select(name)`
  (`src/executor/look_ahead.rs:426,439`);
- a `#[juniper::graphql_object]` method may take an executor argument.
  `juniper_codegen` 0.16.0 recognises it by type or by the parameter name
  `executor` / `_executor` (`common/field/arg.rs:388,395`) and passes
  `&executor` (`common/field/arg.rs:358`);
- the mechanism is entirely synchronous. It behaves identically under
  `execute_sync` and under async `execute`, because it does not depend on future
  polling at all;
- it requires no new dependency, no `async fn` resolver, and no change to
  Juniper's execution model.

Known limitation, accepted and mitigated by the mandatory fallback in section
4.7: `look_ahead()` does not evaluate `@skip` / `@include` directives — the
pinned implementation carries an explicit `// TODO: support excludes`
(`src/executor/mod.rs:709`). Look-ahead may therefore over-report a field that a
directive excludes, costing one unnecessary prefetch query, and in some
fragment-spread shapes may under-report, in which case the child resolver falls
back to its direct per-parent query and remains **correct**. Over-reporting is a
bounded cost; under-reporting is a missed optimisation. Neither is a correctness
defect.

#### A3 - Parent wrapper type carrying preloaded children

The list resolver returns a wrapper struct holding each parent plus its
preloaded children.

Rejected. It changes the GraphQL object implementation target for the parent
type, so it cannot serve a field reached through an existing parent (for example
`Imprint.publisher`) without changing that path too, and it propagates a wrapper
type through the model layer. Invasive and less general than A2 for no gain.

#### A4 - Unconditional parent-list preloading

Same as A2 without look-ahead: the list resolver always preloads.

Rejected. It executes a child query for every list request whether or not the
child field was selected, making the common case worse. Look-ahead is what makes
A2 pay only when the field is actually requested.

---

## 4. Decision

Adopt **Option A, realised as variant A2**: look-ahead-driven set-based prefetch
into request-scoped state on the GraphQL `Context`.

The following subsections are binding on the implementation. Where a point is
deliberately left to implementation, the invariant it must satisfy is stated
instead.

### 4.1 Where request-scoped state lives

On `crate::graphql::model::Context`, as an additional field. `Context` is
constructed once per GraphQL HTTP request in `thoth-api-server/src/lib.rs:96`
and once per test, so it is already the correct request-scoped lifetime carrier.

The store requires interior mutability, because resolvers receive `&Context`.
The async execution path requires `QueryT::Context: Sync`
(`juniper` 0.16.2, `src/http/mod.rs:119`), so the chosen primitive must keep
`Context: Sync`; a `std::sync::RwLock` or `Mutex` over the store satisfies this.

### 4.2 Lifetime

One GraphQL request only. The store is created empty when `Context` is
constructed and dropped when `Context` is dropped.

**A `Context` value must never be reused across requests, stored in a `static`,
placed behind a process-lifetime `Arc`, or registered as actix `app_data`.** The
existing per-request construction already satisfies this and must not change.

### 4.3 No global cache

There is no process-global, cross-request or cross-user cache, and no singleton.
This is invariant 9 in section 5 and is not subject to implementation
discretion.

### 4.4 Batching key representation

The store is keyed by the pair `(loader identity, parent key)`:

- **loader identity** distinguishes one loader-backed field from another and
  must be a closed, compile-time-checked discriminant (for example a
  crate-internal enum), not a free-form string;
- **parent key** is the parent's primary key as its canonical Rust type
  (`Uuid` for every parent type currently in the model). Keys are never
  stringified into a shared namespace.

Mixing two loaders' results is made impossible by construction, not by
convention.

### 4.5 Set-based loader contract

A loader implementation provides:

```text
load(db, keys: &[K]) -> ThothResult<Vec<(K, V)>>
```

executed as **one** database statement using set-based SQL — Diesel
`.eq_any(keys)`, equivalently `WHERE key = ANY($1)` / `IN (...)`. Producing the
result by iterating keys and querying per key is prohibited and defeats the
entire decision.

The loader returns **raw canonical model rows**, not GraphQL objects. Rationale:
GraphQL object construction belongs in the resolver, the loader stays reusable
by non-GraphQL callers, and the loader cannot accidentally acquire
presentation-layer behaviour.

Result partitioning from the flat returned rows back to per-parent buckets is
performed by the shared mechanism, not by each loader.

### 4.6 Duplicate keys, aliases and repeated parents

- the key set is de-duplicated before the query is issued, so `n` parent
  references to the same key produce **one** key in the statement;
- a parent key appearing more than once in the parent list is loaded once;
- repeated aliases for the same field on the same parent
  (`a: distributionPlatforms b: distributionPlatforms`) both read the same stored
  entry and cause **no** additional database access. The store is read
  non-destructively;
- a parent that appears in two different prefetched lists within one request is
  loaded once; a second prefetch for an already-present key does not re-query
  it.

### 4.7 Missing keys, and the mandatory fallback

Two distinct cases, which must not be conflated:

1. **Key present in the store with an empty bucket** — the parent genuinely has
   no child rows. The child resolver returns the empty result. It must **not**
   fall back to a database query.
2. **Key absent from the store** — no prefetch ran for this parent (it was
   reached by a path with no prefetch site, or look-ahead under-reported). The
   child resolver falls back to its ordinary direct per-parent query.

The fallback is what makes correctness independent of look-ahead accuracy. A
loader-backed field is always correct; it is *batched* where a prefetch site
covered it.

The store must therefore distinguish "loaded, empty" from "not loaded". A
representation that cannot express that distinction is non-compliant.

### 4.8 Deterministic result partitioning and ordering

- partitioning is a pure function of the returned rows and the input keys: the
  same rows and keys always yield the same buckets;
- within each key, ordering obeys the owning field's declared contract. The
  set-based query carries the field's `ORDER BY`, extended with the partition key
  so that ordering is total and stable across the whole result set;
- the prefetched result for a parent must be indistinguishable from what the
  field's direct per-parent query would have returned. This equivalence is an
  acceptance criterion, not an assumption.

### 4.9 Database-error semantics

Fail closed. If the set-based query fails, the error propagates as a GraphQL
field error exactly as the direct per-parent query's error would.

Explicitly prohibited: swallowing the error and storing an empty bucket,
returning an empty list, silently falling back to per-parent queries after a
prefetch failure, or storing a poisoned entry that a later reader treats as
"loaded, empty". A failed prefetch must leave the store in a state where the
affected keys are **absent**, not empty.

### 4.10 Bounded batch size

Set-based loading is bounded above by the parent list size, which is already
bounded by the repository's `limit`/`offset` pagination. No additional chunking
is mandated.

Invariant instead: the implementation must not construct an unbounded bind-parameter
list. If a prefetch site can be reached with an unbounded parent count, the
implementation must chunk into bounded statements — a fixed, documented number of
statements is compliant; a per-parent statement is not.

### 4.11 Caching within one request

Yes, within the single request, and only through the mechanism above: a key
loaded once is not loaded again in that request.

### 4.12 Mutation and read-after-write coherence

A GraphQL operation is a query **or** a mutation; it cannot be both. Within a
query operation no write occurs, so no stale read is possible. The risk is
confined to a mutation operation whose payload selection reaches a prefetch site.

Binding rule:

> **Prefetch sites may be installed only on resolvers that are unreachable from
> `MutationRoot` payload selections.** In the initial foundation and in `BE-02`
> the prefetch sites are query-root list fields only (`QueryRoot::publishers`,
> `QueryRoot::publishersByDistributionPlatform`). `MutationRoot` exposes no
> publisher-list field, so no prefetch site is reachable from a mutation
> payload, and no prefetched value can be read after a write in the same
> operation.

Two required supports:

1. `Context` exposes an explicit invalidation entry point that empties the
   store. It is unused by the initial foundation and exists so that a future
   prefetch site reachable from a mutation payload has a correct mechanism
   available rather than inventing one;
2. a test proves the rule holds: a mutation operation that writes child data and
   then selects the affected field in the same operation returns the written
   value, not a prefetched one.

Residual risk, recorded and accepted: the rule is enforced by review and by that
test, not by the type system. Any future task installing a prefetch site
reachable from a mutation payload **must** invalidate on write, and this is
invariant 10 in section 5.

### 4.13 Authorization interaction

Binding rule, and the security core of this decision:

> **A prefetch key set may contain only keys taken from parent objects that the
> current request has already resolved and that the parent resolver has already
> authorized.** A loader never derives keys from user input, never widens a key
> set, and never loads a key that the request has not already been permitted to
> see as a parent.

Consequences:

- batching cannot broaden publisher scope, because every key came from a parent
  the existing scoping already returned;
- the loader is not a generic "load anything by id" facility, and must not be
  used as one;
- if a loader is ever applied to data protected at the *child* level rather than
  inherited from the parent, the loader contract must carry the authorization
  context needed to filter, or must run only after the child-level check. A
  generic loader must never assume all keys in a request are authorized.

The batching foundation introduces no new authorization decision and changes no
policy in `thoth-api/src/policy.rs`.

### 4.14 Transaction and connection model

Unchanged from current practice. The prefetch query acquires a connection from
`Context.db` (`Arc<PgPool>`, r2d2) for the duration of the statement and
releases it, exactly as `X::all(...)` does today. No new transaction boundary,
no long-held connection, no connection stored in the request-scoped state.

Because a prefetch replaces `n` statements with one, it strictly reduces
connection checkouts relative to the per-parent path.

### 4.15 How a resolver opts in

Explicitly and visibly, in two coordinated places:

1. the **parent list resolver** takes the executor argument, consults
   look-ahead for the child field name, and calls the shared prefetch helper
   with the parent keys;
2. the **child resolver** asks the store for its parent's entry and falls back to
   its direct query when the key is absent (section 4.7).

There is no implicit or automatic adoption. A field that does not opt in behaves
exactly as it does today.

### 4.16 Preventing accidental direct per-parent access on loader-backed fields

For a loader-backed field, the direct per-parent query is the mandatory
correctness fallback (section 4.7), so it cannot be forbidden outright. It is
constrained instead:

- the fallback is the **only** permitted direct access on a loader-backed field,
  and only on a store miss;
- detection is by measurement, not by inspection: the query-count evidence of
  section 8 fails if a covered list path issues per-parent statements;
- the acceptance criteria of any task adopting a loader must include that
  measurement for that field.

### 4.17 Public schema effect

None. The batching foundation introduces no GraphQL type, field, argument, enum
value or description change. `thoth-client/build.rs` generated output is
unchanged. This is invariant 8 in section 5 and is directly verifiable by
diffing the generated SDL.

---

## 5. Invariants created by this decision

1. Request-scoped state never crosses GraphQL requests.
2. One parent's data can never be returned for another parent's key.
3. Batching can never broaden authorization or publisher scope; keys come only
   from already-resolved, already-authorized parents.
4. Database errors fail closed and never become an empty or unfiltered result.
5. Loader output is deterministic for a given input key set.
6. Duplicate keys cause no duplicate backend fetches within one request.
7. Result ordering within each key obeys the owning field's declared contract
   and is identical to the direct per-parent result.
8. The batching foundation introduces no public GraphQL schema change.
9. There is no global, static or cross-request cache and no singleton.
10. No stale read-after-write result is served within one operation: prefetch
    sites are unreachable from mutation payloads, and any future site that is
    reachable must invalidate on write.
11. A loader-backed field is correct whether or not a prefetch ran; batching is
    an optimisation layered on an always-correct fallback.
12. Existing fields that do not opt in are behaviourally unchanged.

---

## 6. Implementation impact

Identified from live repository evidence at the verification base. A file is
listed as *expected* only where inspection supports it.

| Area | Expected effect |
|---|---|
| `thoth-api/src/graphql/model.rs` | `Context` gains the request-scoped store field and its accessor/invalidation methods; `Context::new` initialises it empty |
| a new focused module under `thoth-api/src/graphql/` | the store, the loader-identity discriminant, the loader contract, key de-duplication, partitioning and the look-ahead prefetch helper. Justified as a new module because none of the existing modules in `thoth-api/src/graphql/` is a plausible home and `model.rs` is already 107 KB |
| `thoth-api/src/model/**` | for an adopting field only: a set-based query function using `.eq_any(...)`. The foundation itself adds none |
| `thoth-api-server/src/lib.rs` | none expected. `Context::new` keeps its signature if the store is initialised internally; if the signature changes, this call site changes with it |
| `thoth-api/src/graphql/tests.rs` (or a new sibling test module) | the mechanism tests and the query-count evidence harness |
| `thoth-api/src/model/tests.rs` | possible additions to the existing `db` test helpers |
| `thoth-api/src/policy.rs` | none. No authorization decision changes |
| `thoth-api/src/schema.rs`, `thoth-api/migrations/` | none |
| `Cargo.toml` / `Cargo.lock` | none. No new dependency |

---

## 7. Migration, rollout and rollback

### 7.1 Migration

**No database migration.** No table, column, enum, index, constraint or trigger
is added or altered, and `thoth-api/src/schema.rs` is unchanged.

Stop condition: if implementation discovers that a migration *is* required, that
materially changes the task's risk and scope. The implementing agent must stop
and escalate rather than adding one.

### 7.2 Rollout

The foundation merges with **no broad behavioural activation**:

1. the mechanism merges first, exercised by its own tests, adopted by no
   production field;
2. `BE-02` becomes the first required consumer, in its own separately authorized
   task;
3. the existing per-parent child resolvers are **unchanged**;
4. later adoption is by bounded, evidence-led tasks (section 10).

No feature flag is required, because merging the foundation changes no existing
field's behaviour. That is a stronger guarantee than a flag.

### 7.3 Rollback

- **code rollback:** revert the merge commit. Nothing depends on it at that
  point;
- **after adoption:** revert the adopting field to its direct per-parent query.
  Because the fallback path is the field's ordinary query and is retained
  (section 4.7), the adopting field's *result* is unchanged by rollback — only
  its query count is;
- **data rollback:** none. The decision creates no persistent state.

---

## 8. Observability and evidence

Narrative inspection is **not** acceptable evidence. The acceptance signal is
**SQL statement count**, not wall-clock time.

### 8.1 Required measurement

Statements must be observed at the database driver or database, not inferred
from application-level counters alone. The pinned Diesel 2.3.10 exposes
`diesel::connection::set_default_instrumentation`
(`src/connection/instrumentation.rs:280`) with an `InstrumentationEvent::StartQuery`
variant (`:112`), which is sufficient to count statements issued through the
pool.

Constraints the implementation must respect:

- the hook is a **global** default consumed by newly established connections, so
  the counting test must install it before the pool it measures establishes
  connections, and must serialise against other database tests. The existing
  exclusive file lock in `thoth-api/src/model/tests.rs` (`test_lock()`) already
  serialises database tests;
- a loader-level counter alone is insufficient — it cannot detect a per-parent
  query issued by a fallback path.

If instrumentation proves unworkable, an equivalent method that observes actual
SQL (for example PostgreSQL statement logging capture) is acceptable. Dropping to
narrative or application-counter-only evidence is not.

### 8.2 Required evidence

For a loader-backed field under a parent list of size `n`:

1. the child-query count does **not** scale linearly with `n`;
2. proved for at least **two distinct values of `n`** (for example `n = 3` and
   `n = 25`), showing the child-query count constant while `n` changes;
3. duplicate parent references and repeated aliases add **no** child queries;
4. an equivalent per-parent baseline is recorded alongside, so the comparison is
   evidenced rather than asserted;
5. the prefetched result equals the direct per-parent result, element for
   element, in order.

### 8.3 Operational observability

The foundation adds no production log, metric or alert. It changes no
operational runbook. Query-count observation is a test-time concern.

---

## 9. Security and authorization analysis

The mechanism introduces no new authorization decision. Restating the proof:

1. keys originate **only** from parent objects the request has already resolved.
   Those parents passed the existing scoping in their own resolver — for example
   `Publisher::all(..., publishers, ...)` already applies the caller's publisher
   constraint;
2. therefore the key set is a subset of the parents the caller was already
   permitted to see. Batching cannot introduce a key the caller could not have
   fetched individually;
3. the set-based query filters on exactly that key set, so no row outside it can
   be returned;
4. partitioning is by key, so a row can only ever be attached to the parent whose
   key it carries (invariant 2);
5. the store is request-scoped and never shared, so one caller's results are
   unreachable to another (invariants 1 and 9).

The residual case is a loader over data protected at the child level rather than
inherited from the parent. Section 4.13 forbids a generic loader from assuming
such keys are authorized: the loader contract must carry the authorization
context, or the check must precede the load. No such loader exists in the
foundation.

---

## 10. Legacy resolver policy

Binding:

```text
Existing historical child resolvers are not automatically migrated by
THOTH-GQL-BATCH-01.
```

Instead:

1. an evidence-led N+1 inventory and audit is created as **separate follow-up
   work**, which measures actual query counts on representative operations
   rather than assuming that per-parent structure implies a material cost;
2. measurement precedes change in every case;
3. prioritisation follows measured, high-volume or high-cost paths;
4. remediation proceeds through bounded follow-up tasks, each with its own
   specification, review and query-count evidence;
5. legacy remediation is **not** a prerequisite for `BE-02`, unless evidence
   shows a specific legacy path is directly required by `BE-02`.

Rationale: migrating every per-parent child resolver on structural suspicion
alone would be a large, unmeasured change to shared read paths, which is exactly the risk profile this
decision exists to control.

---

## 11. Consequences

### Positive

- new nested fields can satisfy `thoth-api/AGENTS.md` section 6 by following a
  repository pattern instead of escalating an architecture decision;
- database work for a covered list becomes bounded rather than proportional to
  parent count;
- no new dependency and no execution-model change;
- the always-correct fallback means adoption cannot break a field's result;
- the same mechanism serves the sync and async execution paths.

### Negative

- adoption is explicit and touches two places per field (list resolver and child
  resolver); it is not automatic;
- look-ahead does not honour `@skip`/`@include`, so a prefetch can occasionally
  be issued for a field a directive excludes;
- a parent reached by a path with no prefetch site is not batched, so coverage is
  per-path rather than universal;
- `Context` gains interior mutability, which must be kept `Sync`.

### Risks

- **Coherence risk** — a future prefetch site reachable from a mutation payload
  could serve a stale read. Mitigation: section 4.12's rule, invariant 10, the
  provided invalidation entry point, and the required coherence test.
- **Silent non-adoption risk** — a field could appear loader-backed while always
  taking the fallback path. Mitigation: query-count measurement is an acceptance
  criterion, so a field that never batches fails its own tests.
- **Scope-creep risk** — the foundation could grow into a general refactor.
  Mitigation: section 10 and the non-goals of `THOTH-GQL-BATCH-01`.
- **Cross-request leakage risk** — a future change could hoist `Context` into
  shared state. Mitigation: invariants 1 and 9, and a test that proves two
  concurrent requests do not share store contents.

---

## 12. Implementation and dependency relationship

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

Binding consequences:

- `BE-02` is **not** unblocked by this ADR being drafted;
- `BE-02` is **not** unblocked by this ADR being approved;
- `BE-02` runtime implementation requires the foundation to be present on
  `develop`, unless the CTO later explicitly changes that dependency;
- `BE-02` is **not** a dependency of `THOTH-GQL-BATCH-01`. The dependency runs
  the other way.

This ADR does not modify `BE-02`, PR
[#788](https://github.com/thoth-pub/thoth/pull/788) or its branch. The `BE-02`
amendment described above is separate future work on that existing pull request.

---

## 13. Validation

Evidence that this decision is correctly implemented:

- request-scoped state is provably unique per GraphQL request, including under
  concurrent independent requests;
- a covered list of size `n` issues a child-query count that does not scale with
  `n`, measured at two distinct values of `n`;
- duplicate keys and repeated aliases add no database access;
- absent keys fall back correctly and mixed present/absent key sets resolve
  correctly;
- a database failure propagates as a field error and never as an empty result;
- prefetched and direct per-parent results are identical, in order;
- read-after-write coherence holds within one mutation operation;
- the generated GraphQL SDL is byte-identical to the base;
- `thoth-api/src/schema.rs`, `thoth-api/migrations/` and the workspace
  dependency declarations are unchanged;
- non-adopting fields are behaviourally unchanged and existing GraphQL tests pass
  unmodified.

---

## 14. Approval

Approval required from: CTO
Approved by: not yet approved
Approval date: not yet approved

Direction recorded: the CTO selected request-scoped batching / set-based loading
in principle, with `BE-02`'s `Publisher.distributionPlatforms` as the first
required consumer, and directed that the N+1 control is not waived, the approved
`BE-02` field is not removed, and shared GraphQL infrastructure is not invented
inside `BE-02`.

That direction authorizes architecture and task-specification authoring only. It
is not approval of this ADR's content. The final repository decision requires
independent review and explicit CTO approval of this exact content through its
GitHub pull-request record.

This decision does not authorize runtime implementation, modification of
`BE-02` or PR #788, migration of existing legacy resolvers, merge, deployment,
release or any production action.
