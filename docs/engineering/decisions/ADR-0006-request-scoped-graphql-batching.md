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

The store is keyed by the triple:

```text
(loader identity, normalized load shape, parent key)
```

- **loader identity** distinguishes one loader-backed field from another and
  must be a closed, compile-time-checked discriminant (for example a
  crate-internal enum), not a free-form string;
- **normalized load shape** is a typed, loader-specific value capturing every
  argument or semantic input that can change the child result — see 4.4.1;
- **parent key** is the parent's primary key as its canonical Rust type
  (`Uuid` for every parent type currently in the model). Keys are never
  stringified into a shared namespace.

Mixing two loaders' results, or two argument variants' results, is made
impossible by construction, not by convention.

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
one set-based dispatch per unique (loader identity, load shape),
covering all relevant parent keys for that shape
```

Not one global dispatch for all argument variants, and not one dispatch per
parent. A request selecting two argument variants of the same field over the
same parent list issues two dispatches — one per shape — each set-based over the
whole key set.

The number of dispatches is bounded by the number of distinct shapes actually
requested, which is bounded by the query text. It does not grow with parent
count.

#### 4.4.5 `BE-02`'s load shape

`Publisher.distributionPlatforms` takes **no** field arguments in the approved
`BE-02` contract (`BE-02` section 9.2: no `limit`/`offset`, result bounded above
by 17 rows by the composite primary key). Its load shape is therefore trivial:

```text
DistributionPlatformsLoadShape = Unit
```

and the future `BE-02` loader batches by:

```text
(loader = PublisherDistributionPlatforms, shape = Unit, publisher_id)
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

All of the following are per `(loader, shape)`:

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
- a parent that appears in two different prefetched lists within one request is
  loaded once per shape; a second prefetch that would cover an already-present
  `(loader, shape, key)` entry does not re-query it. This is what allows several
  prefetch sites to cover the same loader in one request without duplicate SQL
  (section 4.18).

### 4.7 Store state model

The store holds, for each `(loader identity, load shape, parent key)`, exactly
one of three states. A representation that cannot express all three is
non-compliant.

```text
NotLoaded          // no prefetch attempted for this key under this shape
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
or when look-ahead under-reported (section 3.1, A2). The fallback is what makes
**correctness** independent of look-ahead accuracy. It is *not* evidence of N+1
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
     for that `(loader, shape)`;
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

The failure is recorded **once per `(loader identity, load shape)` dispatch**,
together with the key set that dispatch attempted. A lookup for a key resolves
to `LoadFailed` when that key is in a failed dispatch's attempted set.

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

### 4.11 Caching within one request

Yes, within the single request, and only through the mechanism above: a
`(loader, shape, key)` entry loaded once is not loaded again in that request.
Two shapes of the same field over the same key are two entries and are loaded
once each.

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

1. `Context` exposes an explicit invalidation entry point. For the initial
   foundation it is deliberately the **simplest correct primitive: whole-store
   invalidation**, clearing every entry — successful `Loaded` state, `LoadFailed`
   state, every load shape and every loader. It is unused by the initial
   foundation and exists so that a future prefetch site reachable from a mutation
   payload has a correct mechanism available rather than inventing one.

   A narrower primitive targeting `(loader)`, `(loader, shape)` or
   `(loader, shape, key)` is **not** provided, because no evidence yet shows one
   is needed, and a narrower invalidation is the easier of the two to get subtly
   wrong. Introducing one later requires evidence that whole-store invalidation
   is materially insufficient;
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

1. the **parent list resolver** takes the executor argument, enumerates the
   requested child selections (4.15.1), derives a normalized load shape for each
   distinct variant (4.4.2), and calls the shared prefetch helper once per shape
   with the de-duplicated parent keys;
2. the **child resolver** builds its lookup shape with the *same* loader-owned
   constructor, reads the `(loader, shape, parent key)` state, and acts per the
   table in section 4.7.

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
2. **classify** each path as one of:
   - covered by a set-based prefetch;
   - inherently single-parent and incapable of producing N+1;
   - excluded, with an explicit approved reason;
3. **add a prefetch site** for every list/fan-out path required for compliance;
4. **measure** — provide SQL statement-count evidence for every covered path, or
   for every materially distinct path class;
5. **retain the direct fallback** for correctness on genuinely uncovered,
   single-parent or unanticipated paths — but never count the existence of a
   fallback as N+1 compliance evidence.

A task that cannot achieve compliant coverage within its approved scope must
**escalate**, not declare compliance on the strength of the fallback.

#### 4.18.3 Consequence for `BE-02`

The `BE-02` adoption task must perform this exact-base path inventory for
`Publisher.distributionPlatforms`. At minimum it must investigate:

```text
QueryRoot.publishers                     -> Publisher.distributionPlatforms
QueryRoot.publishersByDistributionPlatform -> Publisher.distributionPlatforms
list paths reaching Imprint.publisher    -> Publisher.distributionPlatforms
list paths reaching Contact.publisher    -> Publisher.distributionPlatforms
any other exact-base list/fan-out route producing Publisher objects
```

That list is a **minimum investigation set, not a complete answer**. `BE-02`
must search its own exact base rather than treating these four as exhaustive,
and must either cover every material fan-out path or explicitly escalate if
compliant coverage would require architecture outside its approved scope.

The inventory belongs to `BE-02` as the adopting task. It is **not** work for
`THOTH-GQL-BATCH-01`, which adopts no production field. What the foundation must
prove is that the mechanism *supports* several prefetch sites for one
`(loader, shape)` in a single request without duplicate loading (4.6), so that
`BE-02` can cover multiple paths when it does the inventory.

#### 4.18.4 This does not widen legacy remediation

Coverage obligations attach to a field **when it adopts a loader**. Existing
child resolvers adopt nothing and are unaffected; section 10 continues to govern
them.

---

## 5. Invariants created by this decision

1. Request-scoped state never crosses GraphQL requests.
2. One parent's data can never be returned for another parent's key.
3. Batching can never broaden authorization or publisher scope; keys come only
   from already-resolved, already-authorized parents.
4. Database errors fail closed and never become an empty or unfiltered result.
5. Loader output is deterministic for a given input key set and load shape.
6. Duplicate keys cause no duplicate backend fetches within one request, per
   `(loader, shape)`.
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
13. Store identity is `(loader identity, normalized load shape, parent key)`.
    Semantically different argument variants of the same field never share a
    stored entry, and an omitted argument normalizes identically to an explicitly
    supplied schema default.
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

---

## 6. Implementation impact

Identified from live repository evidence at the verification base. A file is
listed as *expected* only where inspection supports it.

| Area | Expected effect |
|---|---|
| `thoth-api/src/graphql/model.rs` | `Context` gains the request-scoped store field and its accessor/invalidation methods; `Context::new` initialises it empty |
| a new focused module under `thoth-api/src/graphql/` | the three-state store, the loader-identity discriminant, the load-shape contract, the loader contract, key de-duplication, partitioning, failure recording and the look-ahead prefetch helper. Justified as a new module because none of the existing modules in `thoth-api/src/graphql/` is a plausible home and `model.rs` is already 107 KB |
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

For a loader-backed field under a parent list of size `n`:

1. the child-query count does **not** scale linearly with `n`;
2. proved for at least **two distinct values of `n`** (for example `n = 3` and
   `n = 25`), reported as a table of:

   ```text
   parent count | prefetch child-query count | direct baseline child-query count
   ```

   in which the prefetched count stays bounded while the direct baseline grows
   with `n`;
3. duplicate parent references and repeated aliases of the same normalized shape
   add **no** child queries;
4. distinct load shapes add exactly one dispatch each, not one per parent;
5. a second prefetch site covering an already-loaded `(loader, shape, key)` set
   issues no additional SQL;
6. the prefetched result equals the direct per-parent result, element for
   element, in order;
7. for an adopting task, per section 4.18.2, coverage evidence for every material
   fan-out path or path class — not only for the field's own root list query.

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
  per-path rather than universal, and each adopting task carries a path-inventory
  obligation (section 4.18.2);
- an argument-bearing loader must express per-parent pagination set-based
  (section 4.5.1), which is more demanding than the per-parent query it replaces;
- a failed prefetch fails every covered key for that shape, including keys whose
  individual query might have succeeded (section 4.9.4);
- `Context` gains interior mutability, which must be kept `Sync`.

### Risks

- **Coherence risk** — a future prefetch site reachable from a mutation payload
  could serve a stale read. Mitigation: section 4.12's rule, invariant 10, the
  provided whole-store invalidation entry point, and the required coherence test.
  Residual: the rule is enforced by review and test, not by the type system.
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
  to `THOTH-GQL-BATCH-01`;
- `Publisher.distributionPlatforms`'s load shape is `Unit` (section 4.4.5). This
  ADR adds no argument to that field and changes nothing in the approved `BE-02`
  API contract.

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
- several prefetch sites can cover one `(loader, shape)` in a single request
  without duplicate SQL;
- `NotLoaded`, `Loaded([])` and `LoadFailed` are distinguishable, and only
  `NotLoaded` triggers the fallback;
- a database failure is recorded as `LoadFailed`, surfaces at the child field
  with the same error classification as the direct path, issues no retry SQL, and
  never becomes an empty result;
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
