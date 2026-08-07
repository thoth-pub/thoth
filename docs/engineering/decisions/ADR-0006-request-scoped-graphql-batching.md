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

Remediation base: the execution-scope findings in sections 4.12.6 and 4.12.10
were additionally **reproduced** against the pinned `juniper` 0.16.2 sources in
an isolated throwaway probe outside this repository, not derived from reading
alone. No repository code was built, modified or added for that reproduction.

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
| the integration test harness (`thoth-api/tests/support/mod.rs:108-110`) | `request.execute(&schema, &ctx).await` | **async**, via a deserialized `GraphQLRequest` |

Any mechanism adopted here must therefore work identically under both. That
single constraint eliminates the conventional deferred-dispatch loader, as
section 3.1 shows.

The **request boundary** matters as much as the execution path, because section
4.12.6 places a control there. At that boundary the operation is still a
document plus variables, and `juniper::http::GraphQLRequest` exposes `query`,
`operation_name` and `variables` as public fields (`src/http/mod.rs:32,36,44`),
with `variables()` as a public accessor (`:61`). Everything section 4.12.6
requires is therefore reachable **before** `execute`/`execute_sync` is called,
without replacing either call.

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

**Selected.** A resolver that returns a list of items inspects the requested
selection set through `Executor::look_ahead()`, and when a registered
loader-backed field is selected — either as a direct child of those items or as
a **descendant** beneath intermediate object fields (section 4.19) — issues
**one** set-based query for the keys projected from those already-resolved items
and writes the partitioned result into request-scoped state on `Context`, under
the current top-level response scope (section 4.12). The terminal child resolver
derives the same scope and reads its parent's entry from that state.

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
into request-scoped state on the GraphQL `Context`, **partitioned by top-level
GraphQL response key** (section 4.12).

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

This is the store's **storage lifetime**. Its **reuse namespace** is narrower —
one top-level response key within that request — and the two must not be
conflated (section 4.12.3).

### 4.3 No global cache

There is no process-global, cross-request or cross-user cache, and no singleton.
This is invariant 9 in section 5 and is not subject to implementation
discretion.

### 4.4 Batching key representation

The store is keyed by the quadruple:

```text
(top-level response key, loader identity, normalized load shape, parent key)
```

- **top-level response key** is the execution scope of section 4.12: the first
  GraphQL response-key path segment of the resolver currently executing. It is
  the response key, therefore an alias when one is present (4.12.5). No loader
  entry crosses scopes;
- **loader identity** distinguishes one loader-backed field from another and
  must be a closed, compile-time-checked discriminant (for example a
  crate-internal enum), not a free-form string;
- **normalized load shape** is a typed, loader-specific value capturing every
  argument or semantic input that can change the child result — see 4.4.1;
- **parent key** is the parent's primary key as its canonical Rust type
  (`Uuid` for every parent type currently in the model). Keys are never
  stringified into a shared namespace.

Mixing two loaders' results, two argument variants' results, or two top-level
scopes' results is made impossible by construction, not by convention.

Sections 4.4.1 to 4.4.6 concern the load-shape dimension. The scope dimension is
settled in section 4.12; it is listed here because it is part of the store
identity, not because it is a load-shape concern.

#### 4.4.1 Why the load shape is part of the identity

A pair of `(loader, parent key)` is safe only for an argument-free field. It is
not a reusable GraphQL architecture, because Thoth's existing child fields
already take result-changing arguments. `Publisher.imprints` takes `limit`,
`offset`, `filter` and `order`; `Publisher.contacts` takes `limit`, `offset`,
`order` and `contactTypes`. Under a pair key, these would collide:

```graphql
publisher {
  a: contacts(limit: 1)
  b: contacts(limit: 100)
}

publisher {
  a: imprints(order: { field: IMPRINT_NAME, direction: ASC })
  b: imprints(order: { field: IMPRINT_NAME, direction: DESC })
}
```

Both aliases would read one shared bucket and one of them would be wrong. The
load shape removes that class of defect by construction.

#### 4.4.2 Load-shape rules

Binding:

1. the shape is a **typed, loader-specific** Rust value, not a serialized
   GraphQL argument string. Free-form stringified arguments are prohibited as
   the canonical key: they make equivalent-but-differently-written arguments
   collide-or-diverge on formatting rather than on meaning;
2. the shape includes **every** argument or semantic input that can affect the
   result — pagination, ordering, filters and domain-specific filter arguments;
3. the shape must be usable as a map key with value equality — semantically
   equal shapes compare equal, semantically different shapes never do;
4. **defaults normalize.** An omitted argument and an explicitly supplied
   argument equal to the field's schema default must produce the **same** shape.
   Section 4.4.3 explains why this requires explicit work;
5. semantically different shapes never share a stored bucket;
6. a loader whose field takes no result-changing arguments uses a unit/empty
   shape.

#### 4.4.3 Look-ahead does not apply schema defaults — evidence

This is a concrete hazard in the pinned version, not a theoretical one.

`LookAheadSelection::arguments()` (`juniper` 0.16.2,
`src/executor/look_ahead.rs:577-590`) iterates `f.arguments` — the arguments
**literally present in the query AST**. It does not consult the schema, so it
never materialises a declared default. By contrast, the child resolver receives
the argument *after* Juniper has applied the schema default, so:

| Query text | What look-ahead reports | What the child resolver receives |
|---|---|---|
| `contacts(limit: 100)` | `limit = 100` | `Some(100)` |
| `contacts` | *no `limit` argument* | `Some(100)` (schema default) |

A prefetch site that builds its shape naively from look-ahead would therefore
produce a *different* shape for the two forms, and the omitted form would miss
the store on every parent — correct, because of the fallback, but not batched.

Binding requirement: the shape constructor is a single loader-owned function
that applies the field's declared defaults when an argument is absent, so both
forms normalize to one shape. The **same** constructor must be used by the
prefetch site and by the child resolver's lookup, so the two cannot drift.

`LookAheadArgument::value()` (`src/executor/look_ahead.rs:364-370`) resolves
GraphQL variables against the request's variable map, so `contacts(limit: $n)`
yields the concrete value rather than a variable reference. Shape construction
therefore works for variable-supplied arguments too.

#### 4.4.4 Dispatch model

```text
one set-based dispatch per unique
(top-level response key, loader identity, load shape),
covering all relevant parent keys for that scope and shape
```

Not one global dispatch for all argument variants, and not one dispatch per
parent. A request selecting two argument variants of the same field over the
same parent list issues two dispatches — one per shape — each set-based over the
whole key set.

The number of dispatches is bounded by the number of distinct
`(scope, shape)` combinations actually requested, which is bounded by the query
text: the operation's top-level response fields times the argument variants it
selects. It does **not** grow with parent count, so `1 + N` access cannot arise
from this model (section 4.12.13).

#### 4.4.5 `BE-02`'s load shape

`Publisher.distributionPlatforms` takes **no** field arguments in the approved
`BE-02` contract (`BE-02` section 9.2: no `limit`/`offset`, result bounded above
by 17 rows by the composite primary key). Its load shape is therefore trivial:

```text
DistributionPlatformsLoadShape = Unit
```

and the future `BE-02` loader batches by:

```text
(scope = <top-level response key>,
 loader = PublisherDistributionPlatforms,
 shape = Unit,
 publisher_id)
```

with one set-based query for the requested publisher key set.

This ADR does **not** add pagination, filter or ordering arguments to
`Publisher.distributionPlatforms`, and must not be read as authorizing any
change to the approved `BE-02` API contract. The generic mechanism must
nevertheless be architecturally capable of supporting argument-bearing loaders,
and the foundation must prove that capability (section 4.4.6).

#### 4.4.6 Proving shape support without adopting an argument-bearing field

The foundation must demonstrate multi-shape behaviour, but must **not** do so by
migrating an existing production field that takes arguments — that would begin
the legacy migration section 10 excludes. The proof fixture defines its own
argument-bearing test-only field, so shape support is proven against a real
Juniper execution without touching production resolvers.

### 4.5 Set-based loader contract

A loader implementation provides:

```text
load(db, shape: &Shape, keys: &[K]) -> ThothResult<Vec<(K, V)>>
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

#### 4.5.1 Per-parent pagination in a set-based statement

A shape that carries per-parent `limit`/`offset` must still be satisfied by
**one** statement. Applying the limit to the whole result set is wrong — it
truncates across parents rather than within each — and issuing one statement per
parent is prohibited outright.

A loader whose shape includes per-parent pagination must therefore express it
set-based, for example with `ROW_NUMBER() OVER (PARTITION BY key ORDER BY ...)`
or an equivalent lateral construction. If a particular loader cannot express its
pagination set-based, that loader is not eligible for prefetch and its adopting
task must escalate rather than degrade to per-parent statements.

`BE-02`'s field is unaffected: it takes no pagination arguments (section 4.4.5).

### 4.6 Duplicate keys, aliases and repeated parents

All of the following are per `(scope, loader, shape)` — that is, **within one
top-level response scope**:

- the key set is de-duplicated before the query is issued, so `n` parent
  references to the same key produce **one** key in the statement;
- a parent key appearing more than once in the parent list is loaded once;
- repeated aliases of the same field with the **same normalized shape** on the
  same parent (`a: distributionPlatforms b: distributionPlatforms`) both read the
  same stored entry and cause **no** additional database access. The store is
  read non-destructively;
- repeated aliases of the same field with **different** shapes
  (`a: contacts(limit: 1) b: contacts(limit: 100)`) resolve against different
  stored entries, each correct for its own shape, with one dispatch per shape;
- a parent that appears in two different prefetched lists **within one scope** is
  loaded once per shape; a second prefetch that would cover an already-present
  `(scope, loader, shape, key)` entry does not re-query it. This is what allows
  several prefetch sites — direct and descendant alike — to cover the same loader
  within one scope without duplicate SQL (sections 4.18, 4.19).

Across scopes the opposite holds, and deliberately so: the same
`(loader, shape, key)` reached under a different top-level response key is a
different entry, is `NotLoaded` there, and is dispatched once for that scope
(section 4.12.13).

### 4.7 Store state model

The store holds, for each
`(top-level response key, loader identity, load shape, parent key)`, exactly one
of three states. A representation that cannot express all three is
non-compliant.

```text
NotLoaded          // no prefetch attempted for this key under this scope and shape
Loaded(Vec<V>)     // prefetch succeeded; includes Loaded([]) meaning genuinely empty
LoadFailed(error)  // the covering prefetch dispatch failed
```

Child-resolver behaviour is fully determined by the state:

| State | Child resolver must | Must not |
|---|---|---|
| `NotLoaded` | execute its ordinary direct per-parent query | — |
| `Loaded(rows)` | return `rows` | issue any database query |
| `Loaded([])` | return the empty result | issue any database query, or treat it as a miss |
| `LoadFailed(e)` | return the field error derived from `e` | issue any query, retry, or return an empty result |

`NotLoaded` arises when the parent was reached by a path with no prefetch site,
when the key was loaded only under a **different** top-level response scope
(section 4.12.13), when look-ahead under-reported (section 3.1, A2), or when
scope derivation failed closed (section 4.12.9). The fallback is what makes
**correctness** independent of all four. It is *not* evidence of N+1
compliance — see section 4.18, which makes that distinction binding.

The three states must be mutually unambiguous. In particular `Loaded([])` must
never be represented as absence, and `LoadFailed` must never be represented as
either absence or an empty successful result.

### 4.8 Deterministic result partitioning and ordering

- partitioning is a pure function of the returned rows and the input keys: the
  same rows and keys always yield the same buckets, independently of the load
  shape that produced them;
- within each key, ordering obeys the owning field's declared contract. The
  set-based query carries the field's `ORDER BY`, extended with the partition key
  so that ordering is total and stable across the whole result set;
- the prefetched result for a parent must be indistinguishable from what the
  field's direct per-parent query would have returned. This equivalence is an
  acceptance criterion, not an assumption.

### 4.9 Database-error semantics and error ownership

Fail closed. The difficulty is *where* the failure surfaces, and the earlier
draft of this ADR was internally inconsistent about it: it required both that a
failed prefetch leave the affected keys absent from the store, and that no
fallback run after a failed prefetch. Under A2 those cannot both hold, because
absence is exactly the signal that triggers the fallback. Section 4.7's
`LoadFailed` state exists to resolve that contradiction, and this section
settles the surrounding semantics.

#### 4.9.1 Why the parent list resolver must not fail

Under A2 the prefetch runs **inside the parent list resolver**, before any child
field resolves. If that resolver returned the prefetch error, GraphQL would
attribute the failure to the *parent list* field path — a field that succeeded —
rather than to the child field that would have failed on the direct path. That
would change `errors[].path` and null propagation relative to the unbatched
behaviour, for a failure in what is only an optimisation.

Required sequence:

1. the parent list resolver obtains its parent objects **successfully**;
2. the requested prefetch is attempted;
3. if the set-based statement fails:
   - record a `LoadFailed` outcome in request state covering the attempted keys
     for that `(scope, loader, shape)`;
   - do **not** convert the failure into successful empty buckets;
   - do **not** return the prefetch error from the parent list resolver merely
     because the optimisation failed;
   - the parent list resolver returns its parents normally;
4. each covered child resolver observes `LoadFailed`;
5. each such child resolver returns a `FieldError` derived from the recorded
   failure;
6. **no retry query occurs** on any covered key.

Step 6 is deliberate and is the fail-closed choice. Retrying per parent after a
batch failure would reintroduce exactly the `1 + N` access this decision exists
to prevent, on the pathological path where the database is already unhealthy.

#### 4.9.2 Failure storage granularity

The failure is recorded **once per
`(top-level response key, loader identity, load shape)` dispatch**, together
with the key set that dispatch attempted. A lookup for a key resolves to
`LoadFailed` when that key is in a failed dispatch's attempted set **under the
same scope**.

A `LoadFailed` recorded under scope `A` therefore does **not** poison scope `B`:
a lookup for the same key under `B` is `NotLoaded` and takes the ordinary direct
fallback, exactly as if `A` had never run. Failure state is partitioned by scope
for the same reason successful state is.

Justified by the fact being represented: one set-based statement failed, so one
failure occurred, and it failed the entire dispatch. Per-key storage would
duplicate one error across every attempted key while carrying no additional
information, and would invite the incorrect implication that failures could
differ per key within a single statement.

`ThothError` derives `Error, Debug, PartialEq, Eq, Serialize, Deserialize` and is
**not** `Clone` (`thoth-errors/src/lib.rs:11`). Several child resolvers must each
produce an error from one recorded failure, so the implementation must retain a
shareable representation — `Arc<ThothError>` is the obvious minimal choice. What
is binding is the observable outcome in 4.9.3, not the container.

#### 4.9.3 GraphQL-visible error equivalence contract

Identical error text is **not** sufficient evidence of equivalence, and must not
be offered as such. Any task adopting a loader must verify, for a database
failure, the GraphQL-visible behaviour of the prefetched path against the direct
per-parent path, covering at minimum:

- `errors[].path` — the error is attributed to the child field on the owning
  parent, not to the parent list field;
- null propagation — identical, including propagation through non-null list and
  field types;
- error classification — Thoth maps `ThothError` into `FieldError` with an
  `extensions` object carrying a `type` discriminant
  (`thoth-errors/src/lib.rs:183-207`, for example `INTERNAL_ERROR`,
  `NO_ACCESS`). The prefetched path must produce the same `type`;
- no successful empty-list substitution anywhere;
- no additional fallback SQL after the failure.

If exact equivalence proves technically impossible because the failure
originated during prefetch rather than during child resolution, this ADR
requires the adopting task to define the externally observable equivalence it
does achieve and to **document the intentional difference explicitly**. Hiding
such a difference is prohibited.

#### 4.9.4 One known intentional difference

A prefetch failure fails every covered key for that shape, including keys whose
individual direct query might have succeeded had it been attempted — for example
a transient failure affecting only the batch statement. This is accepted: it is
fail-closed, and the alternative (per-key retry) is prohibited by 4.9.1 step 6.

Explicitly prohibited on every path: swallowing the error and storing an empty
bucket, returning an empty list, silently falling back to per-parent queries
after a prefetch failure, or storing an entry that a later reader can mistake
for `Loaded([])`.

### 4.10 Bounded batch size

Set-based loading is bounded above by the parent list size, which is already
bounded by the repository's `limit`/`offset` pagination. No additional chunking
is mandated.

Invariant instead: the implementation must not construct an unbounded bind-parameter
list. If a prefetch site can be reached with an unbounded parent count, the
implementation must chunk into bounded statements — a fixed, documented number of
statements is compliant; a per-parent statement is not.

### 4.11 Reuse inside one request, partitioned by execution scope

Reuse happens within a single request and only through the mechanism above, but
it is **partitioned by top-level response scope** (section 4.12). A
`(scope, loader, shape, key)` entry loaded once is not loaded again under that
scope. Two shapes of the same field over the same key are two entries and are
loaded once each. The same `(loader, shape, key)` reached under two different
top-level response keys is two entries and is loaded once per scope.

Stated as the binding reuse rule, because the looser form is wrong: two prefetch
sites covering the same loader **under the same execution scope** reuse the same
entry and issue no duplicate SQL. Two prefetch sites covering the same loader
under **different** execution scopes are two entries, and a second bounded
dispatch there is correct and required. "Two prefetch sites covering the same
loader in one request issue no duplicate SQL" is **not** an invariant of this
architecture and must not be written anywhere as one.

### 4.12 Execution scope: top-level response-key partitioning

#### 4.12.1 The problem this settles

Thoth's mutations return rich model objects rather than thin acknowledgement
payloads, so a mutation payload can fan out over publishers with no query
operation involved:

| Mutation | Payload type | Evidence |
|---|---|---|
| `updatePublisher` | `Publisher` | `thoth-api/src/graphql/mutation.rs:405-412` |
| `createPublisher` | `Publisher` | `mutation.rs:75` |
| `deletePublisher` | `Publisher` | `mutation.rs:799` |
| `updateContact` | `Contact` | `mutation.rs:720` |

`Publisher` exposes `contacts` (`model.rs:1258`) and `Contact` exposes
`publisher` (`model.rs:3120`), so this is a material fan-out path:

```graphql
mutation {
  updatePublisher(data: { ... }) {
    contacts { publisher { distributionPlatforms { platform } } }
  }
}
```

An earlier draft attempted to confine prefetch sites to resolvers unreachable
from `MutationRoot` payload selections. That rule is **withdrawn**: it could not
coexist with section 4.18's all-material-path coverage rule, and it would have
left mutation payload fan-out permanently outside the
`thoth-api/AGENTS.md` section 6 N+1 control.

An operation-type-discriminating **scoping** design was investigated and rejected
on evidence, because a nested resolver cannot determine the operation type
through stable public Juniper API (section 4.12.7). The scoping rule is therefore
uniform, and no resolver discriminates operation type.

Operation type **is** discriminated in exactly one place: the request-boundary
guard of section 4.12.6, where `Operation::operation_type` is directly available
and no resolver has yet run. That is a different mechanism at a different point
in the request, and it is not in tension with the rule above.

#### 4.12.2 The decision

This decision is **two coordinated controls**, not one. The second is not an
optimisation or a detail of the first: without it the first is unsound on the
pinned stack, for the reasons established in section 4.12.6.

> **Control 1 — central mutation request guard (section 4.12.6).**
> At the GraphQL request boundary, before any resolver executes, Thoth rejects a
> **mutation** operation in which one top-level response key has more than one
> executable field occurrence:
>
> ```text
> for a mutation operation:
>   each executable top-level response key occurs at most once
> ```
>
> Query operations are **not** restricted and keep ordinary GraphQL/Juniper
> behaviour.
>
> **Control 2 — scope-partitioned loader store.**
> Thoth GraphQL loader state is **owned by one GraphQL request** but
> **partitioned by the current top-level GraphQL response key**. Every loader
> lookup and every prefetch is scoped by:
>
> ```text
> (top-level response key, loader identity, normalized load shape, parent key)
> ```
>
> The top-level response key is derived through one isolated pinned-Juniper
> compatibility shim (section 4.12.8) built on
> `Executor::new_error(..)` / `ExecutionError::path()`.
>
> The scoping rule applies **uniformly** to query operations and mutation
> payloads. No loader entry crosses top-level response-key scopes.
>
> Within one scope, direct and descendant prefetch sites share terminal entries
> and perform set-based loading.
>
> **The dependency between them is binding.** Control 2's mutation
> read-after-write isolation guarantee (invariant 10) holds **only because**
> Control 1 makes a top-level mutation response key correspond to exactly one
> mutation resolver execution. If Control 1 is absent, disabled or bypassed, the
> loader store must **fail closed** and be unavailable (section 4.12.6.6).
> Batching must never operate without its prerequisite.
>
> Correctness does **not** depend on Juniper serializing top-level mutation
> fields.

#### 4.12.3 Storage lifetime is not reuse namespace

These are two different things and conflating them would misdescribe the
architecture:

```text
storage lifetime:      one GraphQL request
reuse/execution scope: one unique executable top-level response key
                       within that request
```

The store still lives on the request-scoped `Context` (section 4.1), is still
created empty per request and dropped with the request (section 4.2), and there
is still no global, static or cross-request cache (section 4.3). Scope
partitioning changes none of that. It partitions entries *inside* the request.
This is **not** a cross-request or independent cache, and must not be described
as one.

The architecture may still be called **request-scoped**, because the container
lives on exactly one request. It must **not** be described as giving request-wide
reuse, because it does not: reuse is confined to one execution scope. The word
"uniquely" in "one unique executable top-level response key" is doing real work
— for mutations it is guaranteed by the section 4.12.6 guard, not by GraphQL.

#### 4.12.4 What a scope is

A scope is identified by the **first GraphQL response-key path segment** of the
resolver currently executing:

```text
top_level_response_key(executor) -> ScopeKey
```

For:

```graphql
query {
  pubs: publishers {
    distributionPlatforms { platform }
  }
}
```

the scope is `"pubs"`, for the `publishers` resolver and for every resolver
nested beneath it, including `Publisher.distributionPlatforms`.

For:

```graphql
mutation {
  first:  updatePublisher(...) { contacts { publisher { distributionPlatforms { platform } } } }
  second: updatePublisher(...) { contacts { publisher { distributionPlatforms { platform } } } }
}
```

there are two distinct scopes, `"first"` and `"second"`. Every resolver beneath
`first` shares `"first"`; every resolver beneath `second` shares `"second"`; and
no entry created under one is visible under the other.

#### 4.12.5 The scope key is a response key, therefore an alias when present

This is intentional and must not be normalized away. For:

```graphql
query {
  a: publishers { ... }
  b: publishers { ... }
}
```

there are two scopes, `"a"` and `"b"`, even though both are the schema field
`publishers`. Collapsing them to `publishers` would let one top-level field's
entries satisfy another's — exactly what the partition exists to prevent in the
mutation case. Alias-keyed scope is the conservative direction.

**Two different rules, for two different purposes, and they must not be
confused:**

| Purpose | Rule | Why |
|---|---|---|
| selection-path matching (4.15.1, 4.19.3) | `field_original_name()` | it identifies **schema fields**, so an alias must not be mistaken for a field name |
| execution-scope identity (this section) | response key / alias | it identifies the **top-level response namespace**, which is exactly what an alias creates |

Both are correct. A reviewer encountering `field_original_name()` in traversal
code and a response key in scope code is looking at two deliberate, different
decisions.

#### 4.12.6 Repeated response keys: the pinned-Juniper duplicate-execution defect, and the central mutation guard

##### 4.12.6.1 The defect

An earlier revision of this ADR asserted that two occurrences of one response
key **share one scope**, and justified it by claiming that validation guarantees
they are "the same schema field with the same arguments, contributing to the
same response field". That justification does not survive contact with the
pinned sources, and the claim it supported is **withdrawn**. It rested on a
false premise:

```text
WITHDRAWN:  one top-level response key == one top-level mutation execution
```

That equality is **false** on `juniper` 0.16.2. The correct finding:

1. `OverlappingFieldsCanBeMerged`
   (`src/validation/rules/overlapping_fields_can_be_merged.rs`, registered at
   `rules/mod.rs:78`) rejects only *incompatible* repeats — different field
   names (`:398-411`), differing arguments (`:415-424`) or conflicting types
   (`:428-440`). Repeated fields that share one response key and are otherwise
   compatible **pass validation**. `find_conflict` does not inspect directives
   at all, so directive differences never make two occurrences conflict;
2. the pinned executor does **not** then execute that compatible group as one
   resolver invocation. It has no field-collection step;
3. the **sync** executor iterates every `Selection::Field` occurrence in
   `resolve_selection_set_into` (`src/types/base.rs:436-500`) and calls
   `instance.resolve_field(..)` once **per occurrence**;
4. the **async** executor does the same in `resolve_selection_set_into_async`
   (`src/types/async_await.rs:209-283`), pushing one future **per occurrence**
   into a `FuturesOrdered`;
5. the per-occurrence results are afterwards reconciled under the shared
   response key by `merge_key_into` (`src/types/base.rs:627-651`), which
   deep-merges objects and lists. The earlier revision cited
   `Object::add_field` "replacing" the value; that is the wrong function — the
   executor calls `merge_key_into`, and `add_field` is only its first-occurrence
   branch. The executor therefore **does** merge repeated selections' *results*,
   while **not** merging their *execution*;
6. therefore one valid top-level response key can correspond to **several actual
   mutation resolver executions**;
7. all of those executions derive the **same** scope under Control 2;
8. loader state created after the first write is then visible after the second
   write;
9. that breaks invariant 10, the central read-after-write isolation invariant.

##### 4.12.6.2 Reproduction against the pinned sources

Reproduced directly against `juniper` 0.16.2 with a probe schema whose mutation
resolver increments a counter. For:

```graphql
mutation {
  x: updateA(id: 1) { id child }
  x: updateA(id: 1) { id child }
}
```

validation passes with **no** errors, the response contains a single merged `x`
object, and the mutation resolver executes **twice** — identically under
`execute_sync` and under async `execute`. The same holds when the duplicate is
introduced through a named fragment spread or an inline fragment. Under async
execution the second source occurrence's resolver was observed completing
*first*, confirming these are genuinely concurrent executions and not a serial
repeat.

##### 4.12.6.3 Why an execution-occurrence scope cannot fix this (F1 rejected)

The natural repair is a richer scope — `(top-level response key, occurrence
identity)` — so that two compatible top-level selections sharing one response
key get different scopes. This was investigated against the pinned API and
**rejected on evidence**, because a nested resolver cannot derive any such
identity:

- `Executor::field_path` is a **private** field. `FieldPath` is a public enum
  (`src/executor/mod.rs:61-64`) and its `Field` variant does carry the ancestor
  `SourcePosition`, but `FieldPath::construct_path` and `FieldPath::location`
  are **private methods** and no public accessor returns the chain.
  `OwnedExecutor` exposes no more (`src/executor/owned_executor.rs:145`);
- `Executor::location()` (`:655`) returns only the **currently executing
  field's** own position, never an ancestor's;
- `ExecutionError::path()` (`:797`) returns **response-key names only**
  (`Vec<String>`), with no positional component;
- `Executor::look_ahead()` (`:694`) locates the current field in the parent
  selection set with `find_map` on the response name, so it resolves to the
  **first** matching occurrence and cannot distinguish duplicates either.

The two publicly derivable signals therefore each fail on their own, and — this
is decisive — they **also fail in combination**. Reproduced on the pinned
sources:

```graphql
mutation {
  x: updateA(id: 1) { ...P }
  x: updateA(id: 1) { ...P }
}
fragment P on Payload { id child }
```

Two distinct mutation resolver executions occur, and the terminal `child`
resolver beneath each observes an **identical** `path()` of `["x", "child"]`
**and** an **identical** `location()`, because both occurrences reach the
terminal field through the same fragment text. Every identity signal available
to a nested resolver is the same under two different writes.

Independently, F1 also fails a structural requirement of section 4.19: the
prefetch site and the terminal descendant must derive the **same** value, yet
their `location()` values are by construction different source positions, and
the descendant cannot recover its ancestor's.

**F1 is rejected.** The pinned stack provides no top-level execution-occurrence
identity to a nested resolver.

##### 4.12.6.4 The decision: a central pre-execution mutation guard (F2)

Since the executor cannot be asked *which* execution this is, the request
boundary is made to guarantee there is only one:

> **Binding rule.** Before executing a **mutation** operation, Thoth rejects the
> request if any executable top-level response key occurs more than once.

This restores, by construction, the equality the architecture needs:

```text
for an accepted mutation operation:
  one top-level response key == one top-level mutation execution
```

and with it the one-to-one relationship between a loader scope and an actual
write execution.

**This is a deliberate server safety restriction, not ordinary validation.** It
rejects some documents the GraphQL specification considers merge-compatible. It
is recorded here as a compensating control for pinned Juniper's repeated
mutation execution behaviour, and section 4.12.6.7 records its compatibility
impact. It must not be described anywhere as spec-conformant validation.

**It is a shared GraphQL execution concern, not a batching helper.** Duplicate
top-level mutation response keys cause a mutation resolver — and therefore a
database write — to run twice for one requested response field, whether or not
any loader-backed field is selected. The guard must therefore protect **every**
mutation request unconditionally, and must not be hidden behind, or conditioned
on, the batching module. `THOTH-GQL-BATCH-01` delivers it as a **shared GraphQL
execution prerequisite** that the loader store then depends on.

##### 4.12.6.5 Feasibility against the pinned public API

Verified reachable, and reproduced end-to-end, using **only** public,
non-`unsafe` API on `juniper` 0.16.2:

| Need | Public API | Location |
|---|---|---|
| read the request without replacing it | `GraphQLRequest::{query, operation_name, variables}`, `variables()` | `src/http/mod.rs:32,36,44,61` |
| parse the document | `juniper::parser::parse_document_source` | `pub mod parser`, `src/lib.rs:35`; `src/parser/document.rs:22` |
| reach the schema metadata | `RootNode::schema` (public field) | `src/schema/model.rs:42` |
| select the operation by `operationName` | `juniper::executor::get_operation` | `pub mod executor`, `src/lib.rs:33`; `src/executor/mod.rs:1001` |
| discriminate operation type | `Operation::operation_type`, `OperationType` | `src/ast.rs:127`; re-exported `src/lib.rs:69` |
| walk top-level selections | `Selection`, `Definition` and their public fields | `src/ast.rs:104,144`; re-exported `src/lib.rs:68-70` |
| resolve directive conditions | `InputValue::into_const` | `src/ast.rs:311` |
| emit a request-validation failure | `RuleError::new`, `GraphQLError::ValidationError` | `src/validation/context.rs:33`; `src/lib.rs:101` |
| return it in the normal response shape | `GraphQLResponse::from_result` | `src/http/mod.rs:176` |

Two pinned-API constraints an implementation must expect, both confirmed by
compilation against the pinned crate:

- `juniper::ast` is a **private module** (`src/lib.rs:32`). `Definition`,
  `Document`, `Operation`, `OperationType` and `Selection` are re-exported at
  the crate root, but `Fragment`, `Field`, `Directive` and `ast::Arguments` are
  **not publicly nameable**. Named-fragment expansion must therefore hold
  `&[Selection]` selection sets rather than `Fragment` values, and directive
  evaluation must be written so those types are only ever *inferred* — for
  example a macro or a closure at the call site — never named in a signature;
- `types::base::is_excluded`, which the executor uses to apply `@skip` and
  `@include`, is `pub(super)` (`src/types/base.rs:596`). The guard must
  reimplement that condition evaluation on public API and keep it behaviourally
  identical.

**The guard does not replace `GraphQLRequest::execute`.** It runs before it, on
the same `GraphQLRequest` the handler already owns, and the accepted-request
execution path is untouched. Explicit parse/validate/execute orchestration —
`parse_document_source` → `ValidatorContext`/`visit_all_rules` →
`execute_validated_query{,_async}` — is also fully public and was confirmed
available, but it is **not** adopted: it would reimplement juniper's request
pipeline and couple Thoth to far more of it than the guard needs. The accepted
cost is instead **one additional document parse per mutation request**, which
must be recorded as such and not presented as free.

##### 4.12.6.6 Fail-closed dependency

The loader store's mutation isolation guarantee is *derived from* the guard.
Binding consequences:

- the guard is **active by default**;
- a nested resolver **cannot** detect operation type (section 4.12.7), so
  "disable batching for mutations only" is not implementable. Therefore if the
  guard is disabled by configuration, the **entire loader store** must be
  unavailable: every prefetch site performs no prefetch and every lookup reads
  `NotLoaded`, so every affected field takes its always-correct direct fallback
  (section 4.7). This is the same fail-closed direction as section 4.12.9;
- the store must **not** be reachable in a build or configuration in which the
  guard is not applied. A silent combination of "batching on, guard off" is
  prohibited, and the implementation must make it unrepresentable rather than
  merely discouraged.

##### 4.12.6.7 Compatibility impact, and the error returned

**What changes.** The public GraphQL **schema** is unchanged — no type, field,
argument or directive is added, removed or altered, and the generated SDL is
byte-identical (section 4.17 is unaffected). What changes is the set of
**accepted requests**: a mutation document with a duplicate executable top-level
response key was previously accepted and executed, and is now rejected.

**Scope of the restriction.** Deliberately as narrow as the defect:

| Case | Behaviour |
|---|---|
| distinct top-level aliases (`first:`/`second:`) | **allowed**, unchanged |
| duplicate executable top-level mutation response key, written directly | rejected |
| the same duplicate introduced through a named fragment spread | rejected |
| the same duplicate introduced through an inline fragment | rejected |
| a syntactic duplicate that `@skip`/`@include` **definitely excludes** for the concrete request, including through variables | **allowed** — it is not an executable occurrence, and must not be misclassified |
| a duplicate whose directive condition cannot be resolved for the concrete request | rejected, conservatively |
| duplicate response keys **anywhere below** the top level of a mutation | **allowed**, unchanged — the defect is about repeated *write* execution |
| duplicate response keys in a **query** operation | **allowed**, unchanged (section 4.12.6.8) |

The restriction is not broadened beyond this. A duplicate top-level mutation
response key means "perform this write twice and merge both results into one
response field", which no client can act on coherently; rejecting it is the
conservative reading, and a client that wants two writes expresses that with two
aliases, which remains fully supported.

**The error.** The guard reuses the repository's existing GraphQL
request-validation failure convention rather than inventing an HTTP protocol:

- it is returned as `GraphQLError::ValidationError` carrying a `RuleError`
  built with `RuleError::new(message, locations)`, wrapped by
  `GraphQLResponse::from_result(Err(..))`;
- consequently `GraphQLResponse::is_ok()` is `false`, and the existing handler
  at `thoth-api-server/src/lib.rs:103-106` returns **HTTP 400** — the same
  status any other GraphQL validation failure already produces. No handler
  branch is added for it;
- the serialized body is the ordinary GraphQL validation-error shape, with an
  `errors` array of `{message, locations}` and **no** `data` key — verified
  byte-comparable to the shape produced by an existing rule such as an unknown
  field;
- the message states that duplicate executable top-level mutation response keys
  are not supported by this server and directs the caller to distinct aliases.
  It must not expose loader, store, scope or other internal implementation
  detail, and must not imply the document is invalid **GraphQL**;
- `locations` carries the source position of each colliding occurrence.

**Zero-execution guarantee.** Rejection happens before `execute`/`execute_sync`
is called at all, so for a rejected operation:

```text
mutation resolver execution count = 0
database write count             = 0
```

A guard that rejected only after the first resolver executed would be
non-compliant. This is a binding acceptance criterion, and section 8.2 requires
it to be measured rather than reasoned about.

##### 4.12.6.8 Query operations keep ordinary behaviour

Duplicate compatible response keys remain **allowed** in query operations, and
two occurrences of one top-level query response key **share one scope**. The
compatibility restriction is not broadened to queries, for two reasons:

- the defect being compensated is *repeated write execution*. A query operation
  performs no write, so no loader entry created under one occurrence can be
  stale with respect to another occurrence in the same request. Sharing is
  therefore safe, and it is what keeps a duplicate query occurrence from
  re-dispatching;
- fragmenting duplicate query occurrences into separate scopes would create one
  dispatch per occurrence for no correctness benefit, working against section
  4.12.13's requirement that terminal statement counts stay bounded
  independently of parent count.

Operation type is available at the request boundary, so the guard discriminates
there — and only there.

##### 4.12.7 Why no operation-type detection is required *at a resolver*

Control 2's scoping rule is uniform, so no **resolver** needs to know whether it
is executing inside a query or a mutation. This is a deliberate design choice
and also the only implementable one on the pinned stack:

- a field resolver's executor is always a **sub**-executor whose `current_type`
  is that field's own type, never the root operation type
  (`src/executor/mod.rs:568-596`);
- `FieldPath::Root` carries only a `SourcePosition` (`:61-64`);
- `SchemaType::query_type()` / `mutation_type()` (`src/schema/model.rs:371,387`)
  describe the schema's shape, not the operation in flight;
- `GraphQLRequest` exposes only `operation_name()` (`src/http/mod.rs:55`), a
  caller-chosen label rather than the operation type.

Implementations **must not** attempt to detect query versus mutation at nested
resolvers, and must not parse the raw GraphQL document to derive scope.

This restriction is about **resolvers**. It is not in tension with section
4.12.6, which discriminates operation type at the **request boundary** — the one
place where `Operation::operation_type` is directly available — before any
resolver runs. The two rules are consistent: operation type is decided once, up
front, and never inferred downstream.

#### 4.12.8 The pinned-Juniper compatibility shim

Control 2 requires deriving the current top-level response key from a nested
resolver.
The pinned Juniper exposes no dedicated public path accessor: `Executor::field_path`
is a private field, and `FieldPath::construct_path` / `FieldPath::location` are
private methods, so although `FieldPath` is reachable
(`pub mod executor`, `src/lib.rs:33`; `pub enum FieldPath`,
`src/executor/mod.rs:61`) its contents are not.

The accepted mechanism is:

```text
Executor::new_error(..) -> ExecutionError::path() -> first response-key segment
```

Both are public and documented (`src/executor/mod.rs:679`; `:797`). This is a
**compatibility shim, not business logic**, and is accepted as a pinned-Juniper
coupling under the controls below.

**Binding contract.** One isolated helper, materially equivalent to:

```text
top_level_response_key(executor) -> Result<ScopeKey>
```

The implementation chooses the exact Rust signature. The helper must:

1. call `Executor::new_error(..)` **only** to materialize the current execution
   path;
2. never call `push_error` or `push_error_at`;
3. never modify the GraphQL response;
4. return the **first** path response-key segment;
5. **fail closed** if no top-level response key can be derived (4.12.9);
6. never parse the raw GraphQL query string;
7. never inspect private Juniper fields;
8. never use `unsafe`;
9. be the **only** location in the codebase permitted to use this technique.

`new_error(..)` calls must not be scattered across loaders, prefetch sites or
resolvers. Every caller obtains its scope from this one helper.

**Why it is side-effect-free.** `new_error` constructs an `ExecutionError` from
`field_path.construct_path(..)` and returns it (`src/executor/mod.rs:679-689`).
It does **not** touch the executor's shared error collection — that is
`push_error_at`, which acquires `self.errors.write()` and pushes (`:665-677`).
Calling `new_error` therefore adds no GraphQL error, changes no `errors[]`
entry, changes no result data, and performs no database access. The constructed
error is discarded once its path has been read.

#### 4.12.9 Fail-closed scope derivation

If a top-level response key cannot be derived — for example an empty path,
which is what `construct_path` produces at `FieldPath::Root` — the helper
returns an error and the calling site **fails closed**:

- a **prefetch site** that cannot derive its scope performs **no prefetch**. It
  does not fall back to a request-global namespace, and it does not fail the
  parent list field; every affected terminal lookup is then simply `NotLoaded`
  and takes the ordinary direct-query fallback (section 4.7), so the operation
  remains **correct** while not being batched on that path;
- a **terminal child resolver** that cannot derive its scope treats its lookup
  as `NotLoaded` and takes the direct-query fallback.

Silently substituting a shared or global namespace is **prohibited**: it would
allow entries to cross top-level scopes, which is the one thing this partition
exists to prevent. Degrading to the correctness fallback is the safe direction;
degrading to a shared namespace is not.

#### 4.12.10 Why correctness does not depend on mutation serialization

The GraphQL specification requires mutation root fields to execute serially. The
pinned Juniper honours that on the sync path only:

| Path | Driver | Behaviour |
|---|---|---|
| sync (`src/executor/mod.rs:883`) | `resolve_selection_set_into`, a plain `for` loop (`src/types/base.rs:436-500`) | serial |
| async (`src/executor/mod.rs:985`) | `resolve_selection_set_into_async`, `FuturesOrdered` (`src/types/async_await.rs:196,209-283,393`) | concurrent — no `OperationType`-aware serialization |

Reproduced: under async execution, two top-level mutation fields' resolvers were
observed completing in the opposite order to their source order.

This is a **pre-existing** deviation from the GraphQL specification in the pinned
dependency. It is recorded here as a finding, not repaired here. Repairing it
would mean reimplementing mutation root execution (option F3, section
4.12.10.1), which this decision does not adopt.

The architecture is **not affected by this discrepancy**, and that is one reason
it is the right shape. For:

```graphql
mutation {
  first:  updateSomething(...)     { ...nested loader-backed selections... }
  second: updateSomethingElse(...) { ...nested loader-backed selections... }
}
```

entries loaded beneath `first` live under scope `"first"` and are structurally
unreachable from scope `"second"`. A write performed by `second` therefore
cannot be followed by a stale read of `first`'s loader state, **even if the
executor interleaves the two top-level futures**.

Correctness rests on scope isolation plus the section 4.12.6 guard, not on
execution order. Precisely:

- the **guard** makes each executable top-level mutation response key correspond
  to exactly one mutation resolver execution, so a scope never spans two writes;
- **scope isolation** makes entries created under one such execution
  unreachable from any other, so ordering and interleaving between distinct
  top-level fields cannot produce a stale read.

An architecture that depended on serialization would be relying on executor
polling behaviour, which section 3.1 rejected when it eliminated variant A1;
this one does not.

##### 4.12.10.1 Option F3 — correcting the execution layer — considered and rejected

The third possible repair is to make the execution layer perform proper GraphQL
field collection, so compatible repeated top-level fields execute once. Assessed
against the pinned sources and **rejected as architecture expansion**:

- juniper's field-collection point, `resolve_selection_set_into{,_async}`, is
  `pub(crate)` inside the **private** `mod types` (`src/lib.rs:37`). It cannot
  be replaced or wrapped from outside the crate;
- the only external interception point is to hand-write
  `GraphQLValue`/`GraphQLValueAsync` for the mutation root — overriding
  `resolve`/`resolve_async` to collect fields itself and delegating
  `resolve_field{,_async}` and `meta` to the codegen-generated implementation.
  That is possible in principle, but it puts Thoth in the business of
  maintaining GraphQL field-collection semantics — fragments, inline fragments,
  `@skip`/`@include`, sub-selection merging, error paths and null propagation —
  against a dependency whose own implementation of them is private;
- correctness would also require fixing serial mutation execution at the same
  time, since concurrent execution of collected mutation fields is itself a spec
  deviation. That changes behaviour for **every** existing multi-field mutation
  request, well beyond the defect being repaired;
- it must hold for both the sync and async paths, doubling the surface;
- it couples Thoth to pinned juniper execution internals far more tightly than
  the section 4.12.8 shim does, and makes any juniper upgrade a re-derivation
  rather than a revalidation.

This is materially larger than `THOTH-GQL-BATCH-01` and is effectively
maintaining a partial custom GraphQL executor. It is recorded as rejected, not
as a batching detail. Should it ever be revisited, it requires its own
architecture decision and specification; it is **not** authorized by this ADR.

#### 4.12.11 Read-after-write within one top-level mutation field

A top-level mutation resolver's write completes before its payload selection set
can resolve: the resolver returns `FieldResult<Publisher>`, and the value must
exist before its sub-selection is resolved. A nested prefetch therefore runs
after the write and observes current database state. This is structural on both
execution paths, not polling-dependent.

#### 4.12.12 Whole-store invalidation

`Context` retains the explicit invalidation entry point, and it remains the
**simplest correct primitive: whole-store invalidation**, clearing every scope,
every loader, every shape, every key, every `Loaded` state and every
`LoadFailed` state.

Its role changes under scope partitioning. Ordinary correctness — including
mutation read-after-write — comes from **scope isolation plus the section 4.12.6
guard**, so it does **not** depend on
mutation resolvers invoking invalidation, and no retrofit of the 88 resolver
methods on `MutationRoot` (`thoth-api/src/graphql/mutation.rs:58-61`) is
required. Avoiding that retrofit is a principal reason this shape was chosen.
The primitive remains available as a conservative API for future exceptional or
non-standard write scenarios.

A narrower primitive targeting `(scope)`, `(loader)`, `(loader, shape)` or
`(scope, loader, shape, key)` is **not** provided, because no evidence yet shows
one is needed and a narrower invalidation is the easier of the two to get subtly
wrong. Introducing one later requires evidence that whole-store invalidation is
materially insufficient.

#### 4.12.13 Accepted tradeoff: cross-top-level query reuse is given up

This is a real cost and must not be presented as zero-cost.

Before scope partitioning, a `(loader, shape, key)` entry loaded anywhere in a
request satisfied every later lookup in that request. Under it, the entry
satisfies only lookups under the same top-level response key. Consequently:

- the same `(loader, shape, parent key)` reached beneath two top-level query
  response keys is loaded **once per top-level response key**;
- an operation may therefore issue a bounded number of additional set-based
  dispatches;
- the bound is determined by the operation's top-level response fields — that
  is, by the query structure — and is **independent of parent list size**;
- so it does **not** recreate `1 + N` behaviour. Two top-level scopes over a
  100-parent list issue **2** set-based statements, not 200.

Request-wide reuse across top-level fields is **no longer an invariant** and must
not be stated as one anywhere in this repository's architecture documents.

#### 4.12.14 Compatibility upgrade policy

The scope shim is coupled to the pinned Juniper API surface. Binding rule:

> Any future change to the `juniper` version that affects `Executor`,
> `ExecutionError`, field-path construction, alias/response-key handling, or
> `new_error()` requires **revalidation of the scope compatibility shim before
> deployment**.

This is a revalidation obligation, not a prohibition on upgrading Juniper. It is
discharged through the repository's ordinary dependency-change review under
`docs/engineering/AGENTS.md` and the release gates — no new process is created —
and the shim's own tests (section 8.2) are the evidence. The shim's module
documentation and tests must state the coupling explicitly so an upgrading agent
encounters it.

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

Top-level response-key scoping (section 4.12) does not change any of this.
Scoping is **isolation, not authorization**: it partitions a request's own
already-authorized results so they cannot be reused across top-level response
keys. It must never be relied on as a permission check, and it neither widens
nor narrows what a caller may see.

### 4.14 Transaction and connection model

Unchanged from current practice. The prefetch query acquires a connection from
`Context.db` (`Arc<PgPool>`, r2d2) for the duration of the statement and
releases it, exactly as `X::all(...)` does today. No new transaction boundary,
no long-held connection, no connection stored in the request-scoped state.

Because a prefetch replaces `n` statements with one, it strictly reduces
connection checkouts relative to the per-parent path.

### 4.15 How a resolver opts in

Explicitly and visibly, in two coordinated places:

1. the **prefetch site** — a resolver that has just resolved a list of items —
   takes the executor argument, derives its **top-level response scope** through
   the shim of section 4.12.8, traverses the requested selection set to find
   every terminal loader-backed selection it covers (4.15.1 for a direct child,
   4.19.3 for a descendant), derives a normalized terminal load shape for each
   distinct variant (4.4.2), projects the terminal loader keys from the resolved
   items (identity for a direct child, the key projector of 4.19.1 for a
   descendant), and calls the shared prefetch helper once per shape with the
   de-duplicated key set;
2. the **terminal child resolver** takes the executor argument, derives its
   **top-level response scope** through the *same* shim, builds its lookup shape
   with the *same* loader-owned constructor, reads the
   `(scope, loader, shape, parent key)` state, and acts per the table in section
   4.7. It is unchanged by whether the entry was prefetched from its own parent
   list or from an ancestor (4.19.2).

Both sides derive the scope from the same helper, so a prefetch site and the
terminal resolvers beneath it necessarily agree: they share a first path
segment by construction. That agreement is an acceptance criterion, not an
assumption (section 8.2).

Both sides also fail closed identically when scope derivation fails (section
4.12.9): the prefetch is skipped, the lookup reads `NotLoaded`, and the field
falls back to its direct query.

There is no implicit or automatic adoption. A field that does not opt in behaves
exactly as it does today.

#### 4.15.1 Enumerating child selections correctly — evidence

The prefetch site must **not** use `LookAheadChildren::select(name)` or
`has_child(name)` to find the child field.

Both match on `LookAheadSelection::field_name()`, which returns the **alias when
one is present** and only otherwise the field's real name
(`juniper` 0.16.2, `src/executor/look_ahead.rs:419-441`). Consequently
`select("distributionPlatforms")` does **not** match `a: distributionPlatforms`,
and both helpers return only the *first* match, so a second alias is invisible
to them.

Binding requirement: the prefetch site iterates `children().iter()` and filters
on `field_original_name()` (`src/executor/look_ahead.rs:528`), collecting **every**
matching selection. It then derives one normalized shape per selection and
dispatches once per distinct shape.

Getting this wrong is not a correctness defect — the fallback covers it — but it
silently defeats batching for exactly the aliased and multi-shape queries the
mechanism must handle, so it is stated here rather than left to the
implementation.

For a descendant site this requirement applies at **every** path segment, not
only at the terminal field. Section 4.19.3 is binding there.

### 4.16 Preventing accidental direct per-parent access on loader-backed fields

For a loader-backed field, the direct per-parent query is the mandatory
correctness fallback (section 4.7), so it cannot be forbidden outright. It is
constrained instead:

- the fallback is the **only** permitted direct access on a loader-backed field,
  and only on a genuine `NotLoaded` miss. `Loaded([])` and `LoadFailed` must
  never reach it (section 4.7);
- detection is by measurement, not by inspection: the query-count evidence of
  section 8 fails if a covered list path issues per-parent statements;
- the acceptance criteria of any task adopting a loader must include that
  measurement for that field, on every material fan-out path identified by the
  section 4.18.2 inventory — not only on the field's own root list query.

### 4.17 Public schema effect

None. The batching foundation introduces no GraphQL type, field, argument, enum
value or description change. `thoth-client/build.rs` generated output is
unchanged. This is invariant 8 in section 5 and is directly verifiable by
diffing the generated SDL.

### 4.18 Adoption coverage: correctness is not N+1 compliance

A2 is opt-in at two places (4.15). The mandatory fallback makes a loader-backed
field **correct** on a path where no prefetch ran. It does **not** make that path
**N+1 compliant**. These are different properties and this ADR treats them
separately:

```text
Correctness:
the field returns the correct result on every path, because a direct
per-parent fallback always exists.

N+1 compliance:
every material list/fan-out path capable of producing N child queries is
covered by a set-based prefetch, and that coverage is measured.
```

"The field is loader-backed" is therefore **not** evidence that the field
satisfies `thoth-api/AGENTS.md` section 6. This distinction is binding.

#### 4.18.1 Why this matters immediately

A parent type is generally reachable by more routes than its own root list
query. Verified at the base commit, `Publisher` objects are produced by at least:

| Route | Evidence |
|---|---|
| `QueryRoot.publishers` (list) | `thoth-api/src/graphql/query.rs:521` |
| `QueryRoot.imprints` (list) `-> Imprint.publisher` | `query.rs:593`; `model.rs:1366` |
| `Publisher.contacts` / `QueryRoot.contacts` (list) `-> Contact.publisher` | `model.rs:3120` |

So once `BE-02` exists, this operation fans out over publishers without touching
the `publishers` root query at all:

```graphql
imprints(limit: 100) {
  publisher {
    distributionPlatforms { platform }
  }
}
```

With a prefetch site only on `QueryRoot.publishers`, every `Publisher` here is
`NotLoaded`, every child resolver falls back, and the operation issues one
`distributionPlatforms` query per imprint. The field is correct and **not** N+1
compliant.

#### 4.18.2 Binding adoption coverage rule

Before any field may be declared N+1 compliant under this ADR, its adopting task
must:

1. **inventory** all GraphQL paths **at its exact implementation base** that can
   expose the parent type under a list or fan-out of multiple parents. The
   inventory is produced by searching the base, not by copying a list from this
   ADR;
2. for each path, **identify the nearest suitable already-resolved list or
   fan-out site** from which the terminal loader key can be safely projected —
   which may be an ancestor rather than the terminal field's own parent (4.19);
3. **record the terminal selection path** from that site to the loader-backed
   field;
4. **record the terminal loader** the path resolves to;
5. **record normalized load-shape construction** for that terminal field;
6. **record the key projector** used at that site, and show it satisfies the four
   conditions of 4.19.4;
7. **record any intermediate authorization boundary** the path crosses, and
   either establish equivalent authorization before the prefetch or escalate;
8. **install the prefetch site**, or **explicitly escalate**;
9. **verify that the prefetch site and the terminal lookup derive the same
   top-level response scope** (4.12, 4.15). A site whose scope differs from its
   terminal resolvers' scope would store entries nothing reads, leaving the path
   correct but never batched;
10. **measure** — provide SQL statement-count evidence for the **terminal
    loader** on every covered path, or on every materially distinct path class,
    reported per top-level scope (section 8.2);
11. **do not claim remediation of legacy intermediate resolvers.** Statements
    issued by pre-existing intermediate resolvers on the measured path are
    reported separately and never counted as terminal-loader compliance (4.19.5);
12. **retain the direct fallback** for correctness on genuinely uncovered,
    single-parent or unanticipated paths — but never count the existence of a
    fallback as N+1 compliance evidence.

**Query and mutation paths use the same algorithm.** Operation kind is no longer
an architecture branch: section 4.12 applies one uniform scoping rule to both, so
a mutation-payload fan-out path is inventoried, covered and measured exactly like
a query path. A task may still *report* operation kind where it makes the
evidence clearer, but it must not treat mutation paths as exempt, blocked or
excluded.

A task that cannot achieve compliant coverage within its approved scope must
**escalate**, not declare compliance on the strength of the fallback.

#### 4.18.3 Consequence for `BE-02`

The `BE-02` adoption task must perform this exact-base path inventory for
`Publisher.distributionPlatforms`. At minimum it must investigate:

```text
query paths:

QueryRoot.publishers                       -> Publisher.distributionPlatforms
QueryRoot.publishersByDistributionPlatform -> Publisher.distributionPlatforms
QueryRoot.imprints -> Imprint.publisher    -> Publisher.distributionPlatforms
QueryRoot.contacts / Publisher.contacts
                   -> Contact.publisher    -> Publisher.distributionPlatforms

mutation payload paths, for example:

updatePublisher -> Publisher.contacts
                -> Contact.publisher       -> Publisher.distributionPlatforms

any other exact-base list/fan-out route producing Publisher objects, under
either operation kind
```

The `-> X.publisher ->` entries are **descendant** paths under section 4.19: the
site belongs at the list resolver that produced the `Imprint`s or `Contact`s,
projecting `publisher_id` from the resolved rows, not at `Imprint.publisher` or
`Contact.publisher`.

The mutation-payload entries are **ordinary covered paths**, handled by the same
algorithm and the same uniform scoping rule as the query paths (section 4.12).
They are not blocked, not exempt and not excluded, and `BE-02` must inventory and
cover them like any other material fan-out path.

That list is a **minimum investigation set, not a complete answer**. `BE-02`
must search its own exact base rather than treating these as exhaustive, and
must either cover every material fan-out path or explicitly escalate if
compliant coverage would require architecture outside its approved scope.

`BE-02` must also keep the two evidence scopes of 4.19.5 separate: bounding
`distributionPlatforms` statements on the imprint and contact paths does not
remediate the per-parent `Publisher::from_id` calls in `Imprint.publisher` and
`Contact.publisher`, and `BE-02` must not report that it does.

The inventory belongs to `BE-02` as the adopting task. It is **not** work for
`THOTH-GQL-BATCH-01`, which adopts no production field for batching. What the
foundation must
prove is that the mechanism *supports* descendant prefetch (4.19), several
prefetch sites for one `(scope, loader, shape)` within one scope without
duplicate loading (4.6), and correct scope isolation across top-level response
keys under both operation kinds (4.12), so that `BE-02` can cover query and
mutation paths alike when it does the inventory.

#### 4.18.4 This does not widen legacy remediation

Coverage obligations attach to a field **when it adopts a loader**. Existing
child resolvers adopt nothing and are unaffected; section 10 continues to govern
them.

### 4.19 Descendant prefetch: a prefetch site need not own the terminal field

Section 4.18.1 identifies material fan-out paths in which the loader-backed
field is **not** a direct child of the list item:

```text
QueryRoot.imprints -> Imprint.publisher -> Publisher.distributionPlatforms
QueryRoot.contacts -> Contact.publisher  -> Publisher.distributionPlatforms
Publisher.contacts -> Contact.publisher  -> Publisher.distributionPlatforms
```

A prefetch model that can only target a **direct** child of the list item cannot
cover these, so section 4.18's coverage rule would be unsatisfiable on exactly
the paths it identifies. This section closes that gap in the shared
architecture. `BE-02` must not invent a second batching mechanism for descendant
paths.

#### 4.19.1 The prefetch site contract

A prefetch site targets either a **direct** loader-backed child of the resolved
list item, or a **descendant** loader-backed field reached through one or more
intermediate object fields, whenever the terminal loader key can be safely
projected from data already present on the resolved list items.

A site settles exactly four things. These are conceptual obligations; the ADR
does not mandate Rust type names, which the implementation may choose on the
evidence available to it:

```text
selection path                       the ordered schema field names from the list
                                     item's selection set down to the terminal
                                     loader-backed field
terminal loader identity             which loader the terminal field is backed by
terminal load-shape constructor      the loader-owned constructor of section 4.4.2
key projector                        resolved list item -> terminal loader key
```

A direct-child site is the degenerate case: a selection path of length one, and
an identity key projector.

#### 4.19.2 Worked example

For:

```graphql
imprints(limit: 100) {
  publisher {
    distributionPlatforms { platform }
  }
}
```

the site installed at `QueryRoot.imprints` expresses:

```text
selection path:  publisher -> distributionPlatforms
terminal loader: PublisherDistributionPlatforms
terminal shape:  Unit
key projector:   Imprint -> imprint.publisher_id
```

and the list resolver:

1. resolves the `Imprint` list normally, through its existing authorized query;
2. detects, by look-ahead traversal (4.19.3), that the terminal field is selected
   beneath the intermediate field;
3. projects the publisher keys from those already-resolved imprints —
   `Imprint.publisher_id` is a `Uuid` present on the resolved row
   (`thoth-api/src/model/imprint/mod.rs:44`);
4. de-duplicates the projected keys;
5. dispatches the terminal loader **once** for the unique key set;
6. stores each result under the ordinary terminal identity **within its own
   top-level response scope** —
   `("imprints", PublisherDistributionPlatforms, Unit, publisher_id)`;
7. leaves `Publisher.distributionPlatforms` to derive the same scope from its own
   executor and consume that ordinary entry when it later resolves, per the
   section 4.7 table.

The same holds inside a mutation payload, with no mutation-specific loader and no
mutation-specific code path. For:

```graphql
mutation {
  up: updatePublisher(...) {
    contacts { publisher { distributionPlatforms { platform } } }
  }
}
```

the list/fan-out site writes
`("up", PublisherDistributionPlatforms, Unit, publisher_id)`, and the terminal
`Publisher.distributionPlatforms` resolver derives the same `"up"` scope from its
executor and consumes that entry.

**There is no separate cache namespace for indirectly prefetched entries.** The
store identity of section 4.4 is unchanged. Within one scope, an entry prefetched
from an ancestor and an entry prefetched from the terminal field's own parent
list are the same entry, and either satisfies the other's lookup. A second
namespace would reintroduce duplicate SQL for the same key and would break
section 4.6's multi-site reuse guarantee.

Scope is the *only* new partition, and it is orthogonal to direct-versus-descendant:
both kinds of site write into, and read from, the scope they are executing under.

#### 4.19.3 Selection-path traversal — binding semantics

Section 4.15.1's alias correction applies at **every** segment of the path, not
only at the terminal field.

- matching is on `field_original_name()`
  (`juniper` 0.16.2, `src/executor/look_ahead.rs:528`) at every segment.
  `LookAheadChildren::select(name)` and `has_child(name)` must not be used at any
  segment: both match `field_name()`, which returns the **alias** when one is
  present, and both return only the first match (`:426-441`);
- traversal is recursive over `LookAheadSelection::children()` (`:606`), whose
  `iter()` (`:451`) yields child selections that themselves expose `children()`.
  This composes to arbitrary depth using stable public API;
- **every** matching terminal selection must be collected, across **all**
  matching intermediate branches. Traversal must not stop at the first match at
  any level.

So this must be detected:

```graphql
imprints { p: publisher { a: distributionPlatforms { platform } } }
```

and this must yield **both** terminal selections, not one:

```graphql
imprints {
  first:  publisher { one: distributionPlatforms { platform } }
  second: publisher { two: distributionPlatforms { platform } }
}
```

Load-shape extraction is performed from **each matching terminal field
selection**, never from an ancestor selection — an intermediate field's own
arguments are not part of the terminal loader's shape. Identical normalized
terminal shapes de-duplicate to one dispatch; different terminal shapes remain
separate dispatches, exactly as section 4.4.4 already requires.

For `BE-02` the terminal shape remains `DistributionPlatformsLoadShape = Unit`
(section 4.4.5). This section adds no argument to
`Publisher.distributionPlatforms`.

The `@skip` / `@include` limitation of section 3.1 (A2) is unchanged and applies
per segment: look-ahead may over-report a directive-excluded path, costing one
unnecessary prefetch, or under-report some fragment shapes, in which case the
terminal child resolver falls back per section 4.7 and remains correct.

#### 4.19.4 Key-projection security rule

Indirect prefetch must not weaken section 4.13. A descendant site may project a
terminal loader key from an already-resolved list item **only when all four**
hold:

1. the relationship is **deterministic from data already present on that
   resolved item** — a foreign key on the row, not a value re-derived from user
   input;
2. the projected key is one that **following the GraphQL relationship would
   itself expose**, so prefetching reveals nothing the traversal would not;
3. the intermediate resolver applies **no additional authorization or policy
   decision** that the prefetch would bypass;
4. the terminal loader does not retrieve **child-protected** data without the
   authorization context that data requires (section 4.13).

`resolved authorized Imprint -> imprint.publisher_id` is admissible under this
rule: the imprint was returned by its own authorized resolver, and the foreign
key is on the resolved row.

It must not be generalized into arbitrary ID derivation from user input, nor
into skipping an intermediate authorization check on the grounds that its
foreign key is already known. Where a descendant path crosses an intermediate
resolver carrying a distinct authorization decision, the adopting task must
either establish equivalent authorization **before** the prefetch or escalate.
Authorization logic must not be duplicated inside a generic loader.

#### 4.19.5 Boundary: descendant prefetch does not remediate legacy intermediate N+1

This must not be overstated, and a future report must not be able to overstate
it. For:

```graphql
imprints(limit: 100) {
  publisher { distributionPlatforms { platform } }
}
```

the intermediate `Imprint.publisher` resolver calls
`Publisher::from_id(&context.db, &self.publisher_id)` once per imprint
(`thoth-api/src/graphql/model.rs:1366`); `Contact.publisher` does the same
(`model.rs:3120`). Those are **pre-existing legacy resolvers** governed by
section 10.

Installing the descendant prefetch at `QueryRoot.imprints` prevents the **new
loader-backed terminal field** from adding a further query per imprint. It does
**not** make the operation globally N+1-free.

Two scopes of evidence are therefore distinct and binding:

```text
loader-backed-field compliance      terminal loader statements are bounded on
                                    every material path

legacy intermediate resolver        statements issued by pre-existing resolvers
performance                         on the same path
```

`BE-02` must prove the first for `distributionPlatforms` on every material path.
`BE-02` is **not** required to remediate `Imprint.publisher`, `Contact.publisher`
or any other legacy access unless separate evidence and an explicit scope
decision require it.

No report may claim that a whole GraphQL operation is free of N+1 access unless
every intermediate path in that operation was separately measured and remediated.
A claim of loader-backed-field compliance must state which scope it covers.

---

## 5. Invariants created by this decision

1. Request-scoped state never crosses GraphQL requests.
2. One parent's data can never be returned for another parent's key.
3. Batching can never broaden authorization or publisher scope; keys come only
   from already-resolved, already-authorized parents.
4. Database errors fail closed and never become an empty or unfiltered result.
5. Loader output is deterministic for a given input key set and load shape.
6. Duplicate keys cause no duplicate backend fetches within one top-level
   response scope, per `(scope, loader, shape)`.
7. Result ordering within each key obeys the owning field's declared contract
   and is identical to the direct per-parent result.
8. The batching foundation introduces no public GraphQL schema change.
9. There is no global, static or cross-request cache and no singleton.
10. No stale read-after-write result is served within one operation. **No loader
    result created after one top-level mutation resolver execution can be
    consumed after a distinct top-level mutation resolver execution that may have
    changed the relevant data.** This rests on three facts together, and on all
    three: a top-level mutation resolver's write completes before its payload
    selection resolves, so read-after-write within one top-level mutation field
    is sound (4.12.11); the section 4.12.6 guard makes each executable top-level
    mutation response key correspond to **exactly one** mutation resolver
    execution, so one scope never spans two writes; and no loader entry crosses
    top-level response-key scopes, so a write performed by one top-level field
    can never be followed by a stale read of another's loader state. This holds
    **regardless of whether the executor serializes or interleaves top-level
    fields** (section 4.12.10). Remove the guard and this invariant fails, which
    is why section 4.12.6.6 makes the store unavailable without it.
11. A loader-backed field is correct whether or not a prefetch ran; batching is
    an optimisation layered on an always-correct fallback.
12. Existing fields that do not opt in are behaviourally unchanged.
13. Store identity is
    `(top-level response key, loader identity, normalized load shape, parent key)`.
    Semantically different argument variants of the same field never share a
    stored entry; an omitted argument normalizes identically to an explicitly
    supplied schema default; and entries never cross top-level response-key
    scopes.
14. The store distinguishes three states — `NotLoaded`, `Loaded` (including
    `Loaded([])`) and `LoadFailed`. `LoadFailed` never triggers a fallback query,
    a retry, or an empty successful result; `Loaded([])` never triggers a
    fallback query; only `NotLoaded` does.
15. A failed prefetch does not fail the parent list field. The failure surfaces
    at the child field that would have failed on the direct path, with the same
    error classification.
16. The existence of a correctness fallback is never, by itself, evidence of N+1
    compliance. Compliance requires an exact-base fan-out path inventory,
    coverage of every material path, and statement-count measurement.
17. A prefetch site may target a descendant loader-backed field, not only a
    direct child. Descendant results are stored under the ordinary terminal
    identity of invariant 13; indirect prefetch never creates a second cache
    namespace.
18. A descendant key projector derives keys **only** from data already present
    on already-resolved, already-authorized items, and never bypasses an
    intermediate authorization decision (4.19.4). Invariant 3 is unweakened by
    indirect prefetch.
19. Terminal-loader compliance and legacy intermediate resolver performance are
    separate evidence scopes. Bounding a terminal loader on a path is never
    reported as making that whole operation free of N+1 access (4.19.5).
20. Loader state is **owned by one request but partitioned by top-level response
    key**. Storage lifetime and reuse namespace are distinct (4.12.3): the store
    still lives on the request-scoped `Context` and still never crosses requests
    (invariants 1 and 9), while reuse is confined to one top-level response key
    within that request.
21. The scope key is the GraphQL **response key**, therefore an alias when one is
    present, and is never normalized to the underlying schema field name. This is
    distinct from selection-path matching, which uses `field_original_name()`
    because it identifies schema fields (4.12.5).
22. The scope is derived through **one** isolated pinned-Juniper compatibility
    shim (4.12.8). It is side-effect-free — it adds no GraphQL error, alters no
    `errors[]` entry, changes no result data and performs no database access —
    and no other site in the codebase uses that technique.
23. Scope derivation **fails closed** (4.12.9). A site that cannot derive its
    scope performs no prefetch and its lookups read `NotLoaded`, falling back to
    the direct query. Substituting a shared or request-global namespace is
    prohibited.
24. The **scoping** rule is applied **uniformly** to query operations and
    mutation payloads. No resolver detects operation type, and mutation-payload
    fan-out is a supported, covered architecture path rather than an exception.
    Operation type is discriminated once, at the request boundary, by the
    section 4.12.6 guard alone.
25. Request-wide reuse across top-level response keys is **not** an invariant. The
    same `(loader, shape, key)` reached under two top-level response keys is
    loaded once per scope; the resulting number of extra dispatches is bounded by
    the operation's top-level structure and never grows with parent count
    (4.12.13). "Two prefetch sites covering the same loader in one request issue
    no duplicate SQL" is **false** as stated; the true rule is same-scope reuse
    (4.11).
26. For an accepted **mutation** operation, each executable top-level response
    key occurs exactly once, guaranteed by the central request guard of section
    4.12.6 rather than by GraphQL validation. A top-level response key therefore
    identifies exactly one mutation resolver execution, which is what makes it a
    sound execution-scope identity. This is **not** true of the pinned executor
    on its own (4.12.6.1).
27. A rejected mutation operation executes **no** mutation resolver and performs
    **no** database write. Rejection precedes execution entirely.
28. The guard restricts **only** duplicate executable top-level response keys in
    **mutation** operations. Query operations, non-top-level selections, and
    distinct top-level aliases are unaffected, and a duplicate that
    `@skip`/`@include` definitely excludes for the concrete request is not an
    executable occurrence (4.12.6.7).
29. The guard changes the set of accepted **requests**, not the public GraphQL
    **schema**. The generated SDL is byte-identical to the base (invariant 8 is
    unweakened), and the rejection uses the repository's existing GraphQL
    validation-failure status and response shape rather than a new protocol.
30. Batching never operates without its prerequisite. If the guard is not
    applied, the loader store is unavailable and every lookup takes the direct
    fallback; "batching on, guard off" is not a representable state (4.12.6.6).
31. No loader entry — successful **or failed** — crosses execution scopes. A
    `LoadFailed` recorded under scope A never poisons scope B, and failure
    dispatch identity is exactly the successful-load identity of invariant 13,
    including its scope component.

---

## 6. Implementation impact

Identified from live repository evidence at the verification base. A file is
listed as *expected* only where inspection supports it.

| Area | Expected effect |
|---|---|
| `thoth-api/src/graphql/model.rs` | `Context` gains the request-scoped store field and its accessor/invalidation methods; `Context::new` initialises it empty |
| a new focused module under `thoth-api/src/graphql/` | the three-state store, the scope-partitioned store identity, the loader-identity discriminant, the load-shape contract, the loader contract, key de-duplication, partitioning, failure recording and the look-ahead prefetch helper. Justified as a new module because none of the existing modules in `thoth-api/src/graphql/` is a plausible home and `model.rs` is already 107 KB |
| a small, separate compatibility-shim module | the single `top_level_response_key(executor)` helper of section 4.12.8, its fail-closed behaviour, its documented pinned-Juniper coupling and its own regression tests. Kept separate from the store module so the coupling is visible and greppable, and so a Juniper upgrade has one place to revalidate (4.12.14) |
| a separate request-boundary guard module under `thoth-api/src/graphql/` | the central duplicate-mutation-response-key guard of section 4.12.6: document parse, operation selection, operation-type discrimination, fragment and inline-fragment expansion, `@skip`/`@include` evaluation, duplicate detection and the `RuleError` it returns. Kept separate from the store because it is a **shared GraphQL execution** control that protects every mutation, not a batching helper (4.12.6.4) |
| `thoth-api/src/model/**` | for an adopting field only: a set-based query function using `.eq_any(...)`. The foundation itself adds none |
| `thoth-api-server/src/lib.rs` | **changes.** The `graphql` handler invokes the guard on the incoming `GraphQLRequest` before `data.execute(&st, &ctx).await`, returning the guard's `GraphQLResponse` unchanged through the existing `is_ok()` branch (`:103-106`) so the HTTP status behaviour is not special-cased. `Context::new` keeps its signature if the store is initialised internally; if the signature changes, this call site changes with it |
| `src/bin/arguments/mod.rs`, `src/bin/commands/start.rs`, `thoth-api-server/src/lib.rs` | the kill switch of section 7.2.1, following the repository's established `clap` `Arg::env(..)` pattern (`src/bin/arguments/mod.rs`). No new configuration mechanism is invented |
| `thoth-api/tests/**` | guard tests exercised through the async `GraphQLRequest::execute` harness (`tests/support/mod.rs:108`), alongside the `execute_sync` mechanism tests |
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

**This section was corrected during remediation.** An earlier revision stated
that the foundation merges with no behavioural activation and that no feature
flag was required. That is **no longer accurate**, because the section 4.12.6
guard sits on the common GraphQL request path and takes effect the moment the
foundation merges — before any production field adopts the store. The claim

```text
no production behaviour at foundation merge
```

must **not** be made, here or in the implementation task.

What is true at merge:

| Component | State at foundation merge |
|---|---|
| loader store | present, adopted by **no** production field. No existing field's behaviour changes |
| the two existing per-parent child resolver patterns | unchanged |
| the scope shim | present, exercised only by the store and its tests |
| **the mutation request guard** | **active on every mutation request**, whether or not any loader-backed field is selected |

So the rollout is:

1. the mechanism and the guard merge together, exercised by their own tests. No
   production field adopts the store;
2. from that merge, mutation requests carrying a duplicate executable top-level
   response key are rejected with HTTP 400 (section 4.12.6.7). Every other
   request is unaffected;
3. `BE-02` becomes the first required consumer of the store, in its own
   separately authorized task;
4. later adoption is by bounded, evidence-led tasks (section 10).

The guard is deliberately **active by default rather than dark-launched**,
because it is the prerequisite the store's mutation isolation depends on
(section 4.12.6.6); shipping it inactive would either leave the store unsafe or
defer the same behaviour change to a less visible merge.

#### 7.2.1 Required control: kill switch

`risk-classification.md` requires, for HIGH-risk work, a "feature flag,
comparison mode or controlled pilot where possible". Here that is discharged by
a **kill switch**, not a rollout flag:

- a single boolean configuration value, **defaulting to enabled**, following the
  established `clap` `Arg::env(..)` pattern in `src/bin/arguments/mod.rs` and
  threaded through `start_server(..)` exactly as existing settings are. No new
  configuration mechanism is invented;
- when disabled, the guard does not run **and the loader store is unavailable**
  (section 4.12.6.6). The two cannot be decoupled, because a nested resolver
  cannot tell a mutation from a query. Disabling therefore degrades the server
  to its pre-foundation behaviour on both counts, which is the safe direction;
- it exists to bound the blast radius of an unforeseen client compatibility
  problem in production, not to stage the rollout. It is not a long-lived flag,
  and it must not become a supported operating mode: a decision to run with it
  disabled is an incident response, not a configuration preference.

No comparison mode or pilot is proposed. The guard's effect is a discrete
accept/reject decision on a document shape, fully determined at the request
boundary and fully covered by the section 8.2 tests; a shadow-comparison
deployment would add operational surface without adding evidence those tests do
not already give.

### 7.3 Rollback

- **code rollback:** revert the merge commit. Nothing depends on the store at
  that point. Note that reverting also removes the guard, which is correct:
  nothing has adopted the store, so nothing is left depending on the guard's
  guarantee;
- **immediate operational rollback, without a deploy:** disable the kill switch
  of section 7.2.1. Mutation requests are accepted exactly as before the merge,
  and the store is unavailable, so no path depends on the guarantee that is no
  longer being enforced;
- **after adoption:** revert the adopting field to its direct per-parent query.
  Because the fallback path is the field's ordinary query and is retained
  (section 4.7), the adopting field's *result* is unchanged by rollback — only
  its query count is. The guard must **not** be reverted independently once a
  field has adopted the store under a mutation path; section 4.12.6.6 makes the
  store unavailable in that configuration rather than silently unsafe;
- **data rollback:** none. The decision creates no persistent state, and a
  rejected request performs no write (invariant 27).

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

The hook is a **global** default consumed by connections established *after* it
is installed. It does not retrofit onto connections that already exist.

That matters here, because the repository's ordinary test pool is a process-wide
`OnceLock<Arc<PgPool>>` (`thoth-api/src/model/tests.rs:36,63-70`). By the time a
particular test installs instrumentation, that pool may already hold established
connections, and holding the exclusive database test lock does not recreate them
— it serialises tests, nothing more. **The measurement must therefore not depend
on the existing singleton pool.**

#### 8.1.1 Required measurement-pool lifecycle

Preferred, and the default the implementation should take:

1. acquire the existing exclusive database test lock (`test_lock()`);
2. reset and prepare the disposable test database;
3. install the instrumentation hook;
4. construct a **new dedicated pool** — after hook installation — for the
   measured operation;
5. run the measured GraphQL operation through that pool;
6. count actual `InstrumentationEvent::StartQuery` events;
7. isolate the count from setup, fixture and migration statements, so the
   reported number is the operation's statements alone.

Equivalent alternative: any other mechanism that observes actual
PostgreSQL/Diesel SQL statements — for example PostgreSQL statement-log capture.

Not acceptable in either case: an application-level loader counter on its own.
It cannot see a per-parent statement issued by a fallback path, which is
precisely the failure mode the evidence exists to detect.

### 8.2 Required evidence

Statement counts are reported **per top-level response scope**, because the scope
is now part of the store identity. The reporting unit is:

```text
top-level scope | parent count | prefetch terminal-query count |
direct baseline terminal-query count | legacy intermediate-query count, if any
```

For a loader-backed field under a parent list of size `n`:

1. **within one top-level scope**, the terminal-query count does **not** scale
   linearly with `n`;
2. proved for at least **two distinct values of `n`** (for example `n = 3` and
   `n = 25`), in which the prefetched count stays bounded while the direct
   baseline grows with `n`;
3. duplicate parent references and repeated aliases of the same normalized shape
   add **no** terminal queries;
4. distinct load shapes add exactly one dispatch each, not one per parent;
5. a second prefetch site covering an already-loaded
   `(scope, loader, shape, key)` set issues no additional SQL **within that
   scope**;
6. **across two top-level scopes**, one dispatch per scope is compliant. For two
   top-level response keys over the same parent list, **2** set-based statements
   is compliant; `2N` terminal statements is not. This extra dispatch is the
   accepted tradeoff of section 4.12.13 and must be reported, not hidden;
7. increasing the parent count within either top-level field does not increase
   that field's terminal dispatch count;
8. the prefetched result equals the direct per-parent result, element for
   element, in order;
9. for an adopting task, per section 4.18.2, coverage evidence for every material
   fan-out path or path class — query and mutation alike — not only for the
   field's own root list query;
10. a **descendant** path (list -> intermediate object -> loader-backed field)
    issues one terminal dispatch for the de-duplicated projected key set, and the
    terminal child resolver issues no fallback statement on that path;
11. statements issued by pre-existing **intermediate** resolvers on a measured
    path are reported as a separate figure from the terminal loader's statements
    (4.19.5). A single combined number is not acceptable evidence;
12. the prefetch site and its terminal resolvers are shown to derive the **same**
    scope value, since a mismatch would leave the path correct but silently
    unbatched. Proved as an explicit equality along the whole descendant chain:

    ```text
    root/list site scope == intermediate descendant site scope
                         == terminal child lookup scope
    ```

    for an unaliased root field, an aliased root field, a path reaching the
    terminal field through a named fragment, a path reaching it through an
    inline fragment, a mutation payload, and — for **query** operations only,
    since mutations reject them — a duplicate compatible top-level response key,
    where all occurrences must derive the one shared scope;
13. the compatibility shim adds **no** GraphQL error, alters no `errors[]` entry,
    changes no result data and issues no SQL (4.12.8);
14. for every rejected mutation operation, the **measured** mutation resolver
    execution count is `0` and the **measured** database write count is `0`
    (invariant 27). This must be observed — a resolver-invocation counter on a
    test-only mutation, and the statement-count harness of section 8.1 — not
    argued from the fact that rejection happens early;
15. duplicate executable top-level mutation response keys are rejected whether
    written directly, introduced through a named fragment spread, or introduced
    through an inline fragment;
16. a duplicate that `@skip`/`@include` definitely excludes for the concrete
    request — including through a variable — is **accepted** and executes once,
    proving the guard does not over-reject;
17. distinct top-level mutation aliases are accepted and each executes once;
18. a duplicate compatible response key in a **query** operation is accepted, and
    the two occurrences share one scope and issue no additional terminal SQL;
19. the rejection's HTTP status and serialized response body match those produced
    by an existing juniper validation rule, compared against a real validation
    failure rather than asserted in isolation.

Wall-clock time remains non-authoritative for every one of these.

### 8.3 Operational observability

The loader store adds no production log, metric or alert, and query-count
observation remains a test-time concern.

The **guard** is different, because it is live on the production request path
from merge (section 7.2). It must be observable enough to detect a client
compatibility problem without waiting for a report:

- each rejection emits **one** log record at warning level, carrying the
  operation name if supplied and the colliding response key. It must **not** log
  the full document, variables, or any argument value, because mutation
  arguments carry user and publisher data;
- no new alert or dashboard is required at merge, and none is created here. A
  sustained non-zero rejection rate is the signal that would justify the kill
  switch of section 7.2.1, and the log record is sufficient to see it;
- no operational runbook changes beyond recording the kill switch and what it
  does.

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
  per-path rather than universal, and each adopting task carries a path-inventory
  obligation (section 4.18.2);
- an argument-bearing loader must express per-parent pagination set-based
  (section 4.5.1), which is more demanding than the per-parent query it replaces;
- a failed prefetch fails every covered key for that shape, including keys whose
  individual query might have succeeded (section 4.9.4);
- `Context` gains interior mutability, which must be kept `Sync`;
- **cross-top-level reuse is given up.** The same `(loader, shape, key)` reached
  under two top-level response keys is loaded once per scope, so an operation may
  issue a bounded number of extra set-based dispatches (section 4.12.13). This is
  a genuine cost, not a zero-cost partition, and it is accepted in exchange for
  mutation-payload coverage that does not depend on execution order;
- the architecture acquires an explicit, documented coupling to the pinned
  Juniper API through the scope shim (section 4.12.8), and a Juniper upgrade
  carries a revalidation obligation (section 4.12.14);
- **a GraphQL compatibility restriction is introduced.** Mutation documents with
  a duplicate executable top-level response key were accepted before and are
  rejected after, even though the GraphQL specification considers them
  merge-compatible (section 4.12.6.7). This is a real, if narrow, deviation from
  specification-conformant server behaviour, adopted deliberately because the
  pinned executor's handling of those documents — executing the write twice — is
  itself incorrect;
- **the foundation is no longer inert at merge.** The guard runs on every
  mutation request from the merge commit, so the previously available claim that
  the foundation has no production effect until a field adopts it is withdrawn
  (section 7.2);
- the guard adds **one document parse per mutation request**, on top of the
  parse juniper already performs (section 4.12.6.5);
- the architecture acquires a second, independent pinned-Juniper coupling — the
  guard's use of `parse_document_source`, `get_operation` and the public AST —
  including a reimplementation of `@skip`/`@include` evaluation that must be
  kept behaviourally identical to juniper's private `is_excluded`.

### Risks

- **Client compatibility risk** — the guard rejects mutation documents that were
  previously accepted, on the live request path, for every API client including
  ones outside this repository. If a real client emits duplicate top-level
  mutation response keys, its mutations start failing at merge. Mitigations: the
  restriction is confined to top-level **mutation** response keys and does not
  touch queries, nested selections or distinct aliases (4.12.6.7); a duplicate
  that a directive definitely excludes is not treated as executable, so the
  common conditional-selection idiom is unaffected; the failure is an ordinary
  GraphQL validation error with an actionable message rather than a 500; a
  rejection log makes the condition visible (8.3); and the kill switch of 7.2.1
  restores prior behaviour without a deploy. Residual: this repository cannot
  enumerate every external client, so the evidence that no client relies on the
  shape is inductive — a duplicate top-level mutation response key requests the
  same write twice and merges both results into one response field, which no
  client can act on coherently — rather than exhaustive.
- **Over-rejection risk** — the guard must decide *executability* before
  execution, which means reimplementing `@skip`/`@include` evaluation that
  juniper keeps private (`is_excluded`, `pub(super)`). If that reimplementation
  drifts, a request the executor would have run correctly could be rejected.
  Mitigations: the deliberate directive test matrix of section 8.2 items 15-17,
  including the variable-driven cases in both directions; and the conservative
  rule that an *undecidable* condition rejects rather than silently admits a
  possible duplicate execution — recorded as a compatibility tradeoff in
  4.12.6.7, not hidden.
- **Prerequisite-decoupling risk** — a future change could leave the store
  enabled while the guard is not applied, which would silently reinstate exactly
  the defect this remediation exists to close. Mitigation: section 4.12.6.6
  requires that state to be **unrepresentable** rather than merely discouraged,
  and invariant 30 is binding on any later change.
- **Scope-extraction risk** — the highest-consequence new risk, because the scope
  key is load-bearing for every lookup. If the shim returned a wrong or
  insufficiently discriminating value, entries could cross top-level scopes and a
  mutation payload could serve a stale read. Mitigations: one isolated helper
  rather than scattered calls (4.12.8); the response key is used verbatim without
  normalization (4.12.5); fail-closed behaviour that degrades to the correctness
  fallback rather than to a shared namespace (4.12.9); and the binding
  path-extraction, isolation, side-effect and collision tests of section 8.2.
  Residual: correctness depends on a public-but-off-label use of
  `new_error(..).path()`, which is why the coupling is documented and carries a
  revalidation obligation.
- **Pinned-Juniper compatibility risk** — the shim depends on `Executor`,
  `ExecutionError`, field-path construction and alias/response-key handling
  staying as they are in `juniper` 0.16.2. Mitigation: the single-site
  restriction, the documented coupling in module docs and tests, and the
  revalidation rule of section 4.12.14. Residual: a Juniper upgrade cannot be
  treated as routine for this module.
- **Coherence risk** — substantially reduced by this decision rather than
  merely held. The guard plus scope isolation together make cross-top-level
  staleness structurally impossible, and neither depends on the executor
  serializing top-level mutation fields — which matters, because the pinned
  async path drives them concurrently through `FuturesOrdered` while the sync
  path is serial (4.12.10). Residual: the guarantee now rests on **two** things
  being correct — scope extraction (the risk above) and the guard's duplicate
  detection including its fragment expansion and directive evaluation — and
  async-path behaviour must still be demonstrated rather than inferred from
  sync-path evidence.
- **Pre-existing non-serial mutation execution** — the pinned async executor
  drives top-level mutation fields concurrently, contrary to the GraphQL
  specification (4.12.10). This decision neither introduces nor repairs that;
  scope isolation makes the loader store correct in its presence. Residual: it
  remains a live deviation in Thoth's GraphQL behaviour, independent of
  batching, and repairing it would require its own architecture decision
  (4.12.10.1). It is recorded here so it is not lost.
- **Descendant key-projection risk** — an indirect site projects a terminal key
  across an intermediate field, so a projector could in principle cross an
  authorization boundary the direct traversal would have enforced. Mitigation:
  the four binding conditions of 4.19.4, the requirement that adopting tasks
  record any intermediate authorization boundary (4.18.2 step 6), and the
  prohibition on duplicating authorization logic inside a generic loader.
- **Recursive look-ahead risk** — descendant traversal must match
  `field_original_name()` at every segment and collect every matching terminal
  selection across every matching branch (4.19.3). Getting it wrong silently
  defeats batching on aliased or repeated descendant paths rather than breaking
  correctness. Mitigation: query-count measurement on descendant paths is an
  acceptance criterion.
- **Legacy-visibility risk** — a descendant prefetch bounds the terminal loader
  while a pre-existing intermediate resolver still issues one statement per
  parent, which could be misreported as whole-operation N+1 freedom. Mitigation:
  the two evidence scopes of 4.19.5, and the separate-figure requirement in
  section 8.2 item 11.
- **Shape-normalization risk** — the highest-consequence new risk. A shape that
  omits a result-changing argument would let two semantically different requests
  share one bucket and return a wrong result. Mitigations: the shape is typed and
  loader-owned rather than a serialized string; the same constructor serves the
  prefetch site and the child lookup so they cannot drift; and the required tests
  include different-argument non-collision and default-normalization cases. This
  risk is *not* covered by the fallback, because it produces a confidently wrong
  answer rather than a miss — it is the one failure mode in this design that can
  return incorrect data.
- **Coverage risk** — a field could be declared N+1 compliant on the strength of
  being loader-backed while a material fan-out path still issues per-parent
  queries. Mitigation: section 4.18 makes correctness and compliance distinct,
  requires an exact-base path inventory, and requires per-path measurement.
- **Silent non-adoption risk** — a field could appear loader-backed while always
  taking the fallback path, for example because the prefetch site used
  `select()` and missed an alias (section 4.15.1). Mitigation: query-count
  measurement is an acceptance criterion, so a field that never batches fails its
  own tests.
- **Scope-creep risk** — the foundation could grow into a general refactor.
  Mitigation: section 10 and the non-goals of `THOTH-GQL-BATCH-01`.
- **Cross-request leakage risk** — a future change could hoist `Context` into
  shared state. Mitigation: invariants 1 and 9, and a test that proves two
  concurrent requests do not share store contents.
- **Look-ahead reporting risk** — `look_ahead()` ignores `@skip`/`@include`
  (section 3.1, A2), so a prefetch may be issued for an excluded field
  (over-reporting, one wasted query) or skipped for some fragment shapes
  (under-reporting, a fallback). Neither is a correctness defect; both are
  bounded costs.

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
  the other way;
- the `BE-02` adoption task carries the exact-base fan-out path inventory for
  `Publisher.distributionPlatforms` required by section 4.18.3, and must either
  cover every material path or escalate. That obligation belongs to `BE-02`, not
  to `THOTH-GQL-BATCH-01`. The imprint and contact routes are **descendant**
  paths under section 4.19; the mutation-payload routes are **ordinary covered
  paths** under the uniform scoping rule of section 4.12, no longer blocked on a
  further architecture decision, and `BE-02`'s inventory must cover query and
  mutation paths alike;
- `Publisher.distributionPlatforms`'s load shape is `Unit` (section 4.4.5). This
  ADR adds no argument to that field and changes nothing in the approved `BE-02`
  API contract;
- **added by remediation.** `BE-02`'s mutation-payload coverage now depends on
  the section 4.12.6 guard, not on scoping alone. Two consequences for the
  future `BE-02` amendment, which is **not** made here:
  - `BE-02` must not restate the withdrawn claim that repeated occurrences of one
    response key safely share one execution scope, nor rely on it for its
    mutation-payload paths;
  - `BE-02`'s compatibility assessment must record that mutation requests with a
    duplicate executable top-level response key are rejected server-wide, since
    that is a client-visible behaviour its programme inherits rather than
    introduces.

  Both are notes for the eventual `BE-02` amendment on its own pull request.
  Nothing in `BE-02`, PR #788, its branch or issue #765 is changed by this ADR.

This ADR does not modify `BE-02`, PR
[#788](https://github.com/thoth-pub/thoth/pull/788) or its branch. The `BE-02`
amendment described above is separate future work on that existing pull request.

---

## 13. Validation

Evidence that this decision is correctly implemented:

- request-scoped state is provably unique per GraphQL request, including under
  concurrent independent requests;
- a covered list of size `n` issues a child-query count that does not scale with
  `n`, measured at two distinct values of `n` through a pool established after
  the instrumentation hook (section 8.1.1), with the direct baseline recorded
  alongside;
- duplicate keys and repeated aliases of the same normalized shape add no
  database access;
- different argument variants of one field resolve against distinct shapes with
  no cross-contamination, and an omitted argument normalizes identically to an
  explicitly supplied schema default;
- several prefetch sites can cover one `(scope, loader, shape)` within one
  top-level scope without duplicate SQL;
- the scope shim returns the field response key for an unaliased top-level field
  and the alias for an aliased one; direct children, deeply nested descendants,
  inline fragments and named fragments all yield the same first path segment; and
  a prefetch site and its terminal resolvers derive identical scope values;
- calling the scope shim adds no GraphQL error, alters no `errors[]` entry,
  changes no result data and issues no SQL;
- scope derivation failure is fail-closed: no prefetch runs, lookups read
  `NotLoaded`, and no shared or request-global namespace is substituted;
- two top-level aliases of the same schema field produce separate loader
  namespaces, and the same `(loader, shape, key)` under two scopes never
  collides;
- a `LoadFailed` recorded under one scope does not poison another scope, and
  whole-store invalidation clears both;
- two top-level response keys over the same parent list issue two set-based
  statements, not `2N`, and increasing the parent count within either does not
  increase that field's dispatch count;
- `NotLoaded`, `Loaded([])` and `LoadFailed` are distinguishable, and only
  `NotLoaded` triggers the fallback;
- a database failure is recorded as `LoadFailed`, surfaces at the child field
  with the same error classification as the direct path, issues no retry SQL, and
  never becomes an empty result;
- prefetched and direct per-parent results are identical, in order;
- a descendant path (list -> intermediate object -> loader-backed field) is
  detected through recursive alias-safe look-ahead, projects and de-duplicates
  its terminal keys from the already-resolved list items, issues one terminal
  dispatch, stores under the ordinary terminal identity, and causes no terminal
  fallback statement;
- terminal-loader statements and pre-existing intermediate-resolver statements
  are reported as separate figures on a descendant path;
- read-after-write coherence holds within one top-level mutation field, using
  test-only mutations and types;
- no loader entry crosses top-level mutation fields: a second top-level mutation
  field's nested selection observes its own write, never a value cached under the
  first, and this holds under async execution where the executor may interleave
  the two top-level futures — demonstrated with deliberate async interleaving
  where practical, not only with sequential completion;
- a mutation operation with a duplicate executable top-level response key is
  rejected before execution, written directly, through a named fragment spread,
  and through an inline fragment; and for each, the mutation resolver execution
  count and the database write count are **measured** as zero;
- a duplicate top-level mutation field that `@skip`/`@include` definitely
  excludes for the concrete request — literal and variable-driven, in both
  directions — is accepted and executes exactly once;
- distinct top-level mutation aliases are accepted, each executes once, and
  their loader entries do not cross;
- a duplicate compatible response key in a **query** operation is accepted, the
  two occurrences share one scope, and the terminal loader issues no additional
  statement for the second occurrence;
- the guard's rejection HTTP status and serialized body match an existing
  juniper validation failure, compared directly rather than asserted;
- with the guard disabled by configuration, the loader store is unavailable:
  every lookup reads `NotLoaded`, every path takes the direct fallback, and
  results are unchanged;
- every scoping and guard behaviour above is demonstrated under **both**
  `juniper::execute_sync` and the async `execute` path, not inferred from the
  sync result alone;
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

Further direction recorded: after an independent review returned `BLOCKED` on the
mutation-payload N+1 boundary, the CTO selected **uniform top-level-response-key
scoping** as the architecture direction to encode here, accepting the
pinned-Juniper path-extraction limitation as a compatibility constraint subject to
the isolation, regression-testing and upgrade-revalidation controls of sections
4.12.8 and 4.12.14, and accepting the loss of cross-top-level query reuse
(section 4.12.13). Section 4.12 records that architecture; no alternative remains
open.

**Remediation recorded, and it requires a decision the CTO has not yet made.** A
subsequent independent review of exact head
`7991d26fe64b8a4a1770cb1062a98a64fb07ba20` returned `BLOCKED`, finding that a
top-level response key does not uniquely identify a mutation execution on pinned
Juniper. That finding is confirmed against the pinned sources (section 4.12.6.1)
and reproduced (4.12.6.2). The CTO-selected objective — mutation isolation
through top-level-response-key scoping — is **sound only if** each executable
top-level mutation response key corresponds to exactly one write execution, and
on the pinned stack it does not.

The content added by this remediation is therefore

```text
remediation required to make the CTO-selected mutation-isolation objective sound
```

and must **not** be read as pre-approved CTO content. Specifically, the central
mutation request guard of section 4.12.6:

- was not part of the direction recorded above;
- changes the set of **accepted GraphQL requests**, deliberately rejecting some
  documents the GraphQL specification considers merge-compatible (4.12.6.7);
- is a **shared GraphQL execution control** affecting every mutation, not a
  batching component, and it is live on the production request path from the
  foundation's merge (section 7.2);
- withdraws this ADR's previous rollout claim that the foundation has no
  production behaviour at merge.

**This is flagged, not assumed.** A restriction on accepted GraphQL requests
affecting all mutations and all API clients is a materially broader decision
than the loader architecture it exists to support. It is recorded here rather
than split into a separate ADR because it is inseparable from the mutation
isolation guarantee — without it the loader store cannot be used on any mutation
path (section 4.12.6.6) — but the CTO is asked to approve it **as its own
decision**, on its own merits, and not as a consequence of having previously
selected response-key scoping. If the CTO declines the request-boundary
restriction, section 4.12 does not survive in its present form and the mutation
isolation objective returns to open, since F1 is rejected on evidence
(4.12.6.3) and F3 is rejected as architecture expansion (4.12.10.1).

That direction authorizes architecture and task-specification authoring only. It
is **not** approval of this ADR's resulting exact content, and this ADR is not
`APPROVED` by virtue of the direction having been given. Final ADR approval
evidence is the GitHub pull-request record on an independently reviewed exact
head, per `ADR-0005`.

This decision does not authorize runtime implementation, modification of
`BE-02` or PR #788, migration of existing legacy resolvers, merge, deployment,
release or any production action.
