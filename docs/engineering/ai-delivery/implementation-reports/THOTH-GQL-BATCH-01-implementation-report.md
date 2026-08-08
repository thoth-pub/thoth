# THOTH-GQL-BATCH-01 Implementation Report

## 1. Repository state

Repository: `thoth-pub/thoth`
Workflow: STANDARD
Base branch: `develop`
Base commit: `df2e2efef176716e8c8d523457b30e3deebab770` (the authorized exact base, verified live before editing)
PR target: `develop`
Programme integration branch: none (STANDARD)
Task branch: `feature/shared-architecture/graphql-batching`
Starting head: `d3f805542052ad430f4b4beee7248b5fb5031b65`
Head commit: see section 3 (the authoritative final exact head is the PR head)
Pull request: [#791](https://github.com/thoth-pub/thoth/pull/791) (**draft**)
Expected branch deletion after merge: YES
Final programme PR required: NO
Implementing model: Claude Opus 5
Reasoning level: HIGH

Live verification performed before any edit:

```text
origin/develop                                   = df2e2efef176716e8c8d523457b30e3deebab770
origin/feature/shared-architecture/graphql-batching = d3f805542052ad430f4b4beee7248b5fb5031b65
merge-base(develop, branch)                      = df2e2efef176716e8c8d523457b30e3deebab770
ahead / behind                                   = 1 / 0
changed file on the branch                       = thoth-api/src/graphql/batching.rs (375 insertions)
```

All facts matched the task statement exactly. Neither `develop` nor the branch
head had moved, so nothing was rebased, reset, force-pushed or overwritten.

### Pre-existing WIP

`d3f80554` added `thoth-api/src/graphql/batching.rs` but **did not declare the
module** in `thoth-api/src/graphql/mod.rs`, and the file imported
`super::mutation_guard::MutationGuardMode`, which did not exist. The file was
therefore never compiled on the branch. It was reviewed critically as WIP rather
than preserved; see section 5, decision 1.

## 2. Scope confirmation

Approved specification:
`docs/engineering/ai-delivery/tasks/THOTH-GQL-BATCH-01.md`
Governing decision: `ADR-0006` (APPROVED and reachable from `develop`).

Implemented objective: a reusable, request-scoped, look-ahead-driven, set-based
batching foundation for `thoth-api` GraphQL, plus the central mutation request
guard `ADR-0006` section 4.12.6 requires, delivered in an inert merged state and
adopted by no production field.

Out-of-scope changes made: NONE.

## 3. Commits

Base `df2e2efe` → head, three commits (the pre-existing WIP commit `d3f80554`
remains the branch's first commit and was not rewritten, rebased or force-pushed):

- `d3f80554` — `feat(graphql): add request-scoped batch store` *(pre-existing WIP,
  inherited; the file it added never compiled — see section 1)*
- `0ce7d2ea` — `feat(graphql): add request-scoped batching foundation and mutation guard`
- `f1497488` — `docs(architecture): add THOTH-GQL-BATCH-01 implementation report and changelog`
- `f324354b` — `test(graphql): prove the query path and non-mutation fast path explicitly`

One further commit adds this section. The authoritative final exact head is the
head shown on PR [#791](https://github.com/thoth-pub/thoth/pull/791), and it is
that head — not any commit listed here — which requires fresh independent
cross-model review and, separately, explicit CTO merge authorization. **Any head
movement invalidates both.**

## 4. Files changed

Runtime:

- `thoth-api/src/graphql/batching.rs` *(rewritten from the WIP commit)*
  - reason: the request-scoped store.
  - behavioural effect: none in production — no production field reads it, and
    availability is derived-false outside `ENFORCE`.
- `thoth-api/src/graphql/mutation_guard.rs` *(new)*
  - reason: the central mutation request guard and the single `MutationGuardMode`.
  - behavioural effect: none at `OFF`; the guard returns before parsing.
- `thoth-api/src/graphql/scope.rs` *(new)*
  - reason: the isolated pinned-Juniper compatibility shim.
  - behavioural effect: none — nothing calls it in a production build.
- `thoth-api/src/graphql/prefetch.rs` *(new)*
  - reason: look-ahead-driven direct and descendant prefetch.
  - behavioural effect: none — no production field installs a site.
- `thoth-api/src/graphql/model.rs`
  - reason: add `batch_store` to `Context` and add `Context::with_guard_mode`.
  - behavioural effect: `Context::new` keeps its signature and now defaults to
    `MutationGuardMode::Off`, so every existing call site is preserved and gets
    an unavailable store. No resolver body changed.
- `thoth-api/src/graphql/mod.rs`
  - reason: declare the new modules; export `MutationGuardMode`; add the single
    public boundary function `run_mutation_guard`.
  - behavioural effect: none by itself.
- `thoth-api-server/src/lib.rs`
  - reason: wire the guard at the GraphQL HTTP request boundary before
    `data.execute(&st, &ctx).await`, and build the request `Context` with the
    same mode.
  - behavioural effect: none at `OFF`.
- `src/bin/arguments/mod.rs`, `src/bin/commands/start.rs`
  - reason: expose the mode through the existing `clap` `Arg::env(..)` pattern.
  - behavioural effect: none — the default is `OFF`.

Test-only:

- `thoth-api/src/graphql/batching_fixture.rs` *(new, `#[cfg(test)]`)* — proof
  loader, test-only GraphQL root/types/mutations, counters, SQL harness.
- `thoth-api/src/graphql/batching_tests.rs` *(new, `#[cfg(test)]`)* — the matrix.
- `thoth-api/src/model/tests.rs` — one additive helper,
  `test_context_with_guard_mode`. No existing helper changed.

Documentation: `CHANGELOG.md`, this report.

## 5. Implementation decisions

1. **The WIP `dispatch` de-duplication was replaced, as instructed.** The prior
   expression was

   ```rust
   let unique_keys: Vec<L::Key> = keys.iter()
       .filter(|key| seen.insert((*key).clone()))
       .cloned().collect();
   ```

   It is *behaviourally* correct — `HashSet::insert` returns `true` only on first
   insertion — but its correctness is invisible at a glance, it relies on a
   side-effecting predicate, and it clones twice. It is now an explicit loop with
   a separate membership set. The WIP was not preserved merely because it existed;
   the whole file was rewritten, and the module was wired into `mod.rs` (it had
   never compiled).

2. **`LoaderIdentity` and `LoadShapeKey` variants are not `cfg(test)`-gated.**
   Gating the only variants would leave both enums **uninhabited** in a
   production build, which makes the store's generic code statically unreachable
   and produces `unreachable_expression` errors under `-D warnings`. One
   compiled-in, never-constructed discriminant is a better encoding than an
   uninhabited key type. The variants name only this task's proof loader; **no
   `BE-02` name appears** anywhere in the code.

3. **`is_excluded` is a macro, not a function.** `juniper::ast` is private, so
   `Directive` and `Arguments` cannot be named in a signature. A macro keeps one
   definition while letting every type be *inferred* at its three expansion sites.

4. **Deliberate divergence from juniper's `is_excluded`, in the safe direction.**
   Juniper `unwrap()`s the resolved condition; here an unresolvable condition
   (after applying operation defaults) counts the occurrence as **executable**,
   so the guard rejects conservatively rather than admitting a possible duplicate
   write. An omitted-but-defaulted variable is resolved before this runs and is
   never classified as undecidable.

5. **The fixture's direct fallback uses `IntoFieldError`, not `?`/`Into`.** See
   section 13, limitation 1 — this is a genuine finding about the repository.

Deviations from the specification: **NONE**.

## 6. Database and migration effects

Migration added: **NO**.

`thoth-api/src/schema.rs` and `thoth-api/migrations/` are byte-identical to the
base (`git diff df2e2efe -- <path>` empty for both). This is a **reviewed
conclusion**, not an omission: the foundation creates no persistent state, adds
no table, column, enum, index or constraint, and the proof fixture runs entirely
against the existing `publisher` and `imprint` tables.

## 7. API and compatibility effects

GraphQL/API changes: **none to the schema.** The generated SDL is
**byte-identical** to the base — see section 9.

**The set of accepted requests does change, narrowly and deliberately, but only
in `ENFORCE`**: a baseline-valid mutation operation in which one executable
top-level response key occurs more than once is rejected. This is *not* a schema
change and *not* ordinary spec-conformant validation; it is a compatibility
restriction this repository chooses in order to prevent a duplicated write. At
merge the mode is `OFF`, so **no** request that is accepted today is rejected.

Generated schema/client updates: none required.
`thoth-client` was inspected: it consumes the generated SDL, which is unchanged,
and issues ordinary single-field queries; no downstream client impact.

Backwards compatibility: preserved at merge.
Deprecations: none.
Cross-repository dependencies: none.

## 8. Authorization and security

Authorization paths changed: **none.** `thoth-api/src/policy.rs` is unchanged;
the guard makes no authorization decision and only ever *declines* a request, so
it cannot broaden access.

Roles/scopes involved: unchanged.

Key provenance: prefetch keys come only from already-resolved, already-authorized
parent items. The descendant key projector reads `Imprint.publisher_id`, a
foreign key already present on the resolved row, satisfying all four conditions
of `ADR-0006` section 4.19.4. Test
`authorization::keys_are_drawn_only_from_already_resolved_parents` proves a
publisher the operation never resolved is never fetched.

Negative authorization tests: the proof fixture touches no child-level protected
data (imprints inherit publisher scope), so `AGENTS.md` section 7's full negative
matrix is not engaged by this task. The existing 13 `graphql_permissions` tests
pass unmodified.

Secret or personal-data handling: guard events carry **only** the mode, the
colliding response keys and the operation name when supplied. The `GuardEvent`
struct has no field capable of holding the document, variables or argument
values. Asserted positively and negatively by
`guard_tests::observation_event_carries_only_permitted_fields`.

## 9. Tests and checks

All commands run at the exact head, with `THOTH_EXPORT_API=http://localhost:8181`
set in the process environment and a disposable PostgreSQL 17 instance
(`TEST_DATABASE_URL`) plus local Redis.

### Formatting

```text
cargo fmt --all -- --check
```

```text
exit 0, no output
```

### Compilation

```text
cargo check --workspace
```

```text
exit 0
```

### Unit + integration/database tests

```text
cargo test --workspace
```

```text
thoth-api            969 passed; 0 failed  (869 pre-existing + 100 added)
graphql_permissions   13 passed; 0 failed  (unmodified)
thoth-export-server  143 passed; 0 failed
thoth-errors          11 passed; 0 failed
thoth-client           4 passed; 0 failed
thoth (bin)            1 passed; 0 failed
doc-tests              0 passed; 8 ignored
TOTAL: 0 failures
```

```text
cargo test -p thoth-api --features backend
```

```text
969 passed; 0 failed; 0 ignored
```

The complete pre-existing GraphQL test suite passes **unmodified** — no existing
test was edited to accommodate the mechanism.

### Lint/static analysis

```text
cargo clippy --all --all-targets --all-features -- -D warnings
```

```text
exit 0 (only a pre-existing future-incompat note for the proc-macro-error2
dependency, present identically on the base)
```

### Whitespace

```text
git diff --check
```

```text
exit 0, no output
```

### Generated-SDL comparison

Method: the SDL is produced by `thoth-client/build.rs` calling
`create_schema().as_sdl()` into `thoth-client/assets/schema.graphql`. That file
is **build-generated and gitignored**, so `git status` is *not* a valid check —
an initial attempt to use it was invalid and was discarded. Both revisions were
built and the outputs diffed:

```text
# head
cargo build --workspace && shasum -a256 thoth-client/assets/schema.graphql
# base, in a detached worktree at df2e2efe
git worktree add <scratch>/base-wt df2e2efef176716e8c8d523457b30e3deebab770
cargo build --workspace -p thoth-client && shasum -a256 <base>/schema.graphql
diff <base>/schema.graphql <head>/schema.graphql
```

```text
base sha256 = 1e08b46b565ef719c404bbe6b3131e6a733df09c7abdc4538b66c2b24d2d899c
head sha256 = 1e08b46b565ef719c404bbe6b3131e6a733df09c7abdc4538b66c2b24d2d899c
diff        = no output  ->  BYTE-IDENTICAL (4352 lines)
```

### Protected-path identity results

`git diff df2e2efef176716e8c8d523457b30e3deebab770 -- <path>` — empty for every
path below:

| Path | Result |
|---|---|
| `thoth-api/src/schema.rs` | UNCHANGED |
| `thoth-api/migrations/**` | UNCHANGED |
| `Cargo.toml`, `Cargo.lock` | UNCHANGED |
| `thoth-api/Cargo.toml`, `thoth-api-server/Cargo.toml`, `thoth-client/Cargo.toml` | UNCHANGED |
| `thoth-api/src/policy.rs` | UNCHANGED |
| `.github/workflows/**` | UNCHANGED |
| `thoth-api/src/graphql/mutation.rs` (all 88 `MutationRoot` resolvers) | UNCHANGED |
| `thoth-api/src/graphql/query.rs` | UNCHANGED |
| `BE-02` specification / PR #788 files | UNCHANGED |

Semantic searches:

- **single `new_error` site** — `grep -rn 'new_error' --include=*.rs` returns
  matches in `thoth-api/src/graphql/scope.rs` **only**, of which exactly one is a
  call (`scope.rs:87`); the rest are documentation. No loader, prefetch site or
  resolver uses the technique.
- **no production child resolver adoption** —
  `git diff <base> -- thoth-api/src/graphql/model.rs` contains no change to any
  `fn …(context: &Context)` resolver body; the diff is confined to the `Context`
  struct and its two constructors.
- **no BE-02 surface** — `grep -rniE 'distributionplatform|distribution_platform|publisher_distribution' --include=*.rs`
  returns nothing.
- **issue #765** — untouched; no commit, comment or edit was made against it.

## 10. Manual verification

Environment: macOS, Rust stable, PostgreSQL 17 (disposable, `initdb`-created for
this session), Redis 6379, `THOTH_EXPORT_API` exported.

Steps and observed results are the command results of section 9 plus the
statement-count evidence of section 9.1 below. No manual browser or API
interaction was required or performed; nothing was deployed.

### 9.1 SQL statement-count evidence

**Method.** Statements are observed **at the driver** via
`diesel::connection::set_default_instrumentation`, counting `StartQuery` events.
Application-level counters alone are insufficient because they cannot see a
per-parent statement issued by a fallback path, so both are recorded.

**Measured-pool lifecycle** — the requirement that makes this evidence valid:

```text
1. acquire the existing exclusive database test lock  (test_lock())
2. reset and seed the disposable database THROUGH THE ORDINARY POOL
3. install the instrumentation hook
4. construct a NEW DEDICATED POOL behind the hook          <-- essential
5. run the measured operation through that pool
6. count actual StartQuery events
7. isolate the count from setup/fixture/migration statements (capture is armed
   only around the measured operation)
8. restore global instrumentation on drop
```

The repository's ordinary test pool is a process-wide `OnceLock<Arc<PgPool>>`
whose connections may already be established; the hook applies only to
connections established *after* installation. That pool is therefore **never**
the measured pool.

**Results, reported per top-level response scope.**

| Scope | n | prefetch terminal-query count | direct baseline terminal-query count | legacy intermediate-query count |
|---|---|---|---|---|
| `testPublishers` (direct, query) | 3 | **1** | 3 | — |
| `testPublishers` (direct, query) | 7 | **1** | 7 | — |
| `testImprints` (descendant, query) | 3 | **2** (list + 1 set-based dispatch) | — | **6** |
| `testImprints` (descendant, query) | 6 | **2** | — | **12** |
| `only` (direct, **mutation payload**) | 3 | **1** dispatch, 0 fallbacks | — | — |
| `only` (direct, **mutation payload**) | 6 | **1** dispatch, 0 fallbacks | — | — |

The prefetched count is **bounded within each scope** while the direct baseline
grows with `n`.

**Two-top-level-field query result** (`ADR-0006` section 4.12.13 tradeoff, stated
rather than hidden):

```graphql
query { first: testPublishers { imprints { … } } second: testPublishers { imprints { … } } }
```

```text
n = 5
first  -> 1 bounded set-based child dispatch
second -> 1 bounded set-based child dispatch
total  -> 2 dispatches for the two top-level scopes, NOT N + N
results correct under both; no cross-scope reuse occurs
increasing n within either field does not increase that field's dispatch count
```

The second dispatch is the **accepted cost** of cross-scope isolation and is
reported as such.

**Evidence separation, stated explicitly.** On the descendant path the
terminal-loader statement count is bounded (**2** at both `n = 3` and `n = 6`)
while the intermediate resolver's count grows (**6** then **12**).
**Bounding the terminal loader does not make the operation globally free of N+1
access.** The intermediate `publisher` resolver is a legacy per-item resolver
governed by `ADR-0006` section 10, and these are two distinct evidence scopes.

## 11. CI

CI status: see the pull request for the exact-head result; the CI section of this
report is completed against run IDs bound to the final head.

Workflows triggered on the exact head: `build-test-and-check`
(`classify`, `format_check`, `lint`, `build`, `test`), `check-changelog`,
`run-migrations`, `publish-to-dockerhub`.

PASS vs SKIPPED is distinguished on the PR; a skipped check is **not** reported
as passing.

## 12. Rollout and rollback

Initial state after merge:

```text
guard mode                = OFF
loader store              = unavailable
added request-path work   = none
production request accept = unchanged
```

Activation required: **YES, and it is separately authorized.**

```text
merge authorization
!=
OBSERVE activation authorization
!=
ENFORCE activation authorization
```

Configuration: the existing `clap` `Arg::env(..)` pattern —
`--mutation-guard-mode` / `THOTH_GRAPHQL_MUTATION_GUARD_MODE`, values `OFF`,
`OBSERVE`, `ENFORCE`, **default `OFF`** (verified by test). There is deliberately
**no** separate store-enable setting.

Migration sequence: none.

Rollback/disable procedure: the mode value —
`ENFORCE -> OBSERVE`, `ENFORCE -> OFF`, `OBSERVE -> OFF`. Because store
availability is derived from the mode, each also makes the store unavailable, so
no path is left depending on a guarantee no longer enforced. Code rollback is
reverting the merge commit, which is a no-op for production behaviour.

**Rollback timing and mechanism are UNVERIFIED.** How the value is changed,
whether a restart or deploy is required, how long it propagates, how it is
verified across replicas and who authorizes it are all open under **CG-13**.
This report does **not** claim rollback is certain, immediate or deploy-free.

Monitoring required: none for the store (no production field adopts it, and it is
unavailable outside `ENFORCE`). For the guard, one structured warning record per
would-be or actual rejection, plus the service-health signals of section 8.3 —
which must be **verified to exist** before `OBSERVE`.

### 12.1 Runtime-operations status (section 11.7 / CG-13)

| # | Activation question | Status |
|---|---|---|
| 1 | production runtime/deployment owner | **OPEN** |
| 2 | where the mode value is configured | **ANSWERED** — `clap` `Arg::env(..)`, `THOTH_GRAPHQL_MUTATION_GUARD_MODE`, process-start |
| 3 | restart or deploy required to change it? | **OPEN** — the value is read at process start, so at minimum a restart is implied, but the production mechanism is unmapped |
| 4 | how a change reaches all replicas | **OPEN** |
| 5 | expected/observed propagation interval | **OPEN** |
| 6 | how the effective mode is verified fleet-wide | **OPEN** |
| 7 | rollback procedure | **OPEN** |
| 8 | who authorizes a rollback | **OPEN** |
| 9 | preview/staging rehearsal with timing | **OPEN — not performed** |
| 10 | partial-fleet mode change detection | **OPEN** (load-bearing: a fleet split between `ENFORCE` and `OBSERVE` produces inconsistent client acceptance *and* inconsistent store availability simultaneously) |

Only question 2 is answered, and it establishes only that a configuration
**input** exists. Resolving CG-13 is outside this task's authorization and is a
dependency of **production activation**, not of merge.

### 12.2 Monitoring and thresholds (section 8.3)

```text
BLOCKED FOR PRODUCTION ACTIVATION - MONITORING / THRESHOLDS UNVERIFIED
```

This repository establishes no authoritative production latency, error-rate or
SLO baseline for the `thoth` GraphQL API, so activation thresholds cannot be
derived. **No numeric threshold has been invented.**

### 12.3 Preview/staging performance evidence (section 11.3)

**NOT PRODUCED.** No preview or staging environment was exercised; this task
performed no deployment. The `OFF` vs `OBSERVE` comparison for representative
queries and mutations, the validation-failing / high-complexity / fragment-heavy
document cases, and the rehearsed timed rollback are all **outstanding
activation prerequisites**. They are not merge prerequisites, because the merged
state is inert.

### 12.4 Production activation boundary

```text
merge state:
  OFF, store unavailable, no request-path overhead,
  production request acceptance unchanged

OBSERVE:
  separate explicit CTO production authorization      -- NOT GIVEN
  runtime-operations prerequisite (11.7 / CG-13)      -- NOT SATISFIED
  monitoring/threshold prerequisite (8.3)             -- NOT SATISFIED
  preview/staging evidence (11.3)                     -- NOT SATISFIED

ENFORCE:
  separate second explicit CTO production authorization -- NOT GIVEN
  OBSERVE compatibility evidence passed                 -- NOT PRODUCED
  OBSERVE operational-health evidence passed            -- NOT PRODUCED
```

CTO **merge** authorization is not activation authorization for either
transition. `ENFORCE` is not the only activation. Approval of one activation is
not approval of the other. Live approval evidence belongs to the GitHub/release
record under `ADR-0005`; **no approval identifier is committed to this
repository.**

---

## A. Architecture implementation summary

### A.1 Exact store identity

```text
(top-level GraphQL response key, loader identity, normalized load shape, parent key)
```

applied **uniformly** to queries and mutation payloads; no resolver detects
operation type. Failure state is keyed
`(scope, loader, shape, attempted key set)` and child-lookup state
`(scope, loader, shape, parent key)`; the scope component is never dropped from
either. No source-position or AST-occurrence component is part of the scope key.

### A.2 Load-shape implementation

`LoadShapeKey` is a **typed, closed** enum, never a serialized argument string.
The proof loader's shape is `TestImprintShape { limit: i32 }`, constructed
through the **single** loader-owned constructor `TestImprintLoader::shape(..)`
used by **both** the prefetch site and the child lookup, so the two cannot drift.

Defaults normalize explicitly: `LookAheadSelection::arguments()` reads only
literal AST arguments and does **not** apply schema defaults, while the child
resolver receives the default-applied value. `shape_from_selection` therefore
applies `DEFAULT_IMPRINT_LIMIT` when the argument is absent. Test
`traversal::omitted_argument_and_explicit_default_resolve_against_the_same_entry`
proves the omitted form and the explicit-default form share **one** entry.

Shape equality is semantic (derived `PartialEq`/`Hash` over the normalized
value), so two shapes built from equivalent argument sets compare equal
regardless of construction order or formatting.

### A.3 Failure semantics

A failed dispatch is recorded once per `(scope, loader, shape)` with the
attempted key set, and every covered key becomes `LoadFailed`. The parent list
resolver still returns its parents successfully. Each covered child resolver
returns the derived `FieldError`. **No retry query is issued** — proven by
dispatching again *against a working pool* and observing `AlreadyLoaded` with the
failure still in place. `LoadFailed` is never absence and never `Loaded([])`.
`ThothError` is not `Clone`, so a shareable `SharedLoadError` retaining the
message and the `extensions.type` classification is stored instead.

**Observed GraphQL error contract comparison** (prefetched vs direct, same
operation shape, `error_contract::prefetched_failure_matches_the_direct_failure_contract`):

| Property | Prefetched path | Direct path | Equal? |
|---|---|---|---|
| `errors[].path` terminal segment | `imprints` | `imprints` | **yes** |
| `errors[].path` depth | same | same | **yes** |
| null propagation at the child field | `null` | `null` | **yes** |
| `extensions.type` | `INTERNAL_ERROR` | `INTERNAL_ERROR` | **yes** |
| empty-list substitution | none | none | **yes** |
| additional fallback SQL after failure | none (measured 0) | n/a | **yes** |

**One known intentional difference**, carried forward from `ADR-0006` section
4.9.4 and stated rather than hidden: a prefetch failure fails **every** covered
key for that shape, including keys whose individual direct query might have
succeeded. This is fail-closed and accepted; per-key retry is prohibited.

### A.4 Scope shim

Exact signature:

```rust
pub(crate) fn top_level_response_key<S, C>(executor: &Executor<'_, '_, C, S>) -> Option<ScopeKey>
where S: ScalarValue
```

Juniper APIs called: `Executor::new_error(..)` then `ExecutionError::path()`,
taking the **first** path segment. Both are ordinary **public and documented**
juniper API (unlike the eligibility gate's surfaces), so this carries the weaker
of the two coupling risks — but it remains a compatibility shim, not business
logic.

**Side-effect-free**: `new_error` constructs an `ExecutionError` from
`field_path.construct_path(..)` and returns it; it does **not** touch the
executor's shared error collection, which is `push_error_at`. The constructed
error is discarded once its path is read. Proven by test: calling the shim at
three sites in one document adds **no** GraphQL error, changes **no** `errors[]`
entry and changes **no** result data (the serialized data is asserted exactly),
and it issues no SQL.

**Fail-closed**: returns `None` when no top-level response key can be derived
(for example the empty path `construct_path` produces at `FieldPath::Root`).
Every caller treats `None` as "no scope": a prefetch site performs no prefetch
and does not fail the parent list field; a terminal child resolver reads
`NotLoaded` and takes its direct query. **No shared or request-global namespace
is ever substituted** — `ScopeKey` is only ever constructed from this helper's
output or, in tests, explicitly.

**Only site**: `grep -rn 'new_error' --include=*.rs` matches `scope.rs` alone,
with exactly one call site.

**Documentation and tests** record the pinned-Juniper 0.16.2 coupling and the
`ADR-0006` section 4.12.14 revalidation obligation explicitly; the module's 11
tests are the revalidation harness. **No package dependency was added.**

Shim test results — all passing: unaliased top-level field returns its field
response key; aliased field returns **the alias**; direct child, deeply nested
descendant, intermediate aliases, inline fragments and named fragments all
return the **same** first segment; two top-level aliases of one schema field
derive **distinct** scopes; prefetch site and terminal resolver derive identical
values within one top-level field.

### A.5 Descendant prefetch

Representation actually implemented — `PrefetchTarget`, settling all four
`ADR-0006` section 4.19.1 concepts:

```rust
pub(crate) struct PrefetchTarget<'a, Item, L: BatchLoader, S: ScalarValue> {
    path: &'a [&'a str],                                    // selection path (schema names)
    terminal_shape: fn(&LookAheadSelection<'_, S>) -> L::Shape, // terminal shape ctor
    project_key: fn(&Item) -> Option<L::Key>,               // key projector
}
// terminal loader identity is carried by the type parameter `L` (L::IDENTITY)
```

A direct-child site is the degenerate case: a one-element path and an identity
projector.

Stable Juniper APIs relied on for recursive, alias-safe traversal:
`Executor::look_ahead()`, `LookAheadSelection::children()`,
`LookAheadChildren::iter()` and `LookAheadSelection::field_original_name()` — all
**ordinary documented public API**, not `doc(hidden)`.
`LookAheadChildren::select(..)` and `has_child(..)` are **never** used at any
segment: both match `field_name()` (the alias when present) and both return only
the first match. Matching is on `field_original_name()` at **every** segment, and
traversal stops at no level's first match.

**Indirectly prefetched entries are stored under the ordinary terminal
identity** `(scope, loader, shape, terminal key)` — confirmed. There is **no**
separate cache namespace for them: within one scope an ancestor-prefetched entry
and a parent-list-prefetched entry are the same entry and satisfy each other's
lookups; across scopes they are distinct entries.

Proven: alias at the terminal segment; alias at the intermediate segment; two
aliased intermediate branches each carrying a terminal selection (both
discovered); terminal shapes built from the **terminal** selection; projected
keys de-duplicated before dispatch; one set-based terminal dispatch; results
equal to the direct per-parent result in order; **zero** terminal fallback
statements on the covered path; and the intermediate resolver unmodified.

### A.6 The guard as implemented

**Placement**: `thoth-api-server/src/lib.rs`, in the `#[post("/graphql")]`
handler, **before** `data.execute(&st, &ctx).await`, through the single public
boundary function `thoth_api::graphql::run_mutation_guard`.
`GraphQLRequest::execute` is **not** replaced. The guard makes **no**
authorization decision.

**Mode is checked first.** In `OFF` the function returns before any parsing, so
`OFF` imposes no duplicate-work cost on any request of any kind.

**Baseline eligibility gate**, in this exact order, using pinned juniper's own
helpers:

```text
1. juniper::parser::parse_document_source(query, &root_node.schema)
2. juniper::executor::get_operation(&document, operation_name)
3. if operation_type != Mutation -> EXIT (no decision, no event)
4. ValidatorContext::new(&root_node.schema, &document) + visit_all_rules
5. if root_node.introspection_disabled:
      validation::visit(MultiVisitorNil.with(rules::disable_introspection::factory()), ..)
6. juniper::validation::validate_input_values(&variables, operation, &root_node.schema)
```

**Evidence that this matches juniper's own pipeline**: `juniper::execute_sync`
(`src/lib.rs:147-190`) performs parse → `ValidatorContext` + `visit_all_rules` →
conditional `disable_introspection` → `get_operation` → `validate_input_values` →
`execute_validated_query`, calling exactly these functions with exactly these
arguments. The gate reorders **selection before validation** deliberately
(`ADR-0006` section 4.12.6.5.4) so a non-mutation exits before the expensive
stages; this is safe because the gate never surfaces an error in either order and
treats any failure as ineligible, so every request reaching a **decision** has
still passed all gates.

No private field, no `unsafe`, no raw-source manipulation and no second GraphQL
implementation was used.

**Fragment expansion**: named fragment spreads and inline fragments are both
expanded before counting. Named-fragment selection sets are held as
`&[Selection]` because `juniper::ast::Fragment` is not publicly nameable.
Cycle safety uses a **path stack** of fragment names currently being expanded,
not a global "seen" set — so two legitimate sibling spreads of one fragment are
counted as two distinct occurrences (proven by
`repeated_legitimate_spreads_of_one_fragment_are_both_counted`), while a cyclic
spread cannot recurse.

**Response key** = alias when present, otherwise the field name. Collision keys
are returned **deterministically** (sorted).

**Genuinely undecidable conditions**: counted as **executable**, so the guard
rejects conservatively. This is a **tradeoff, not exactness**, and is stated as
such.

**Rejection**: `GraphQLResponse::from_result(Err(GraphQLError::ValidationError(vec![RuleError…])))`
carrying the colliding source positions. `is_ok()` is `false`, so the **existing**
handler branch returns HTTP 400 with the ordinary GraphQL validation-error body
and no `data` key. **No new handler branch and no one-off HTTP protocol.**

### A.7 Juniper API-surface characterisation

These required surfaces are **`#[doc(hidden)]` on pinned 0.16.2**:

```text
juniper::parser::parse_document_source
juniper::executor::get_operation
juniper::validation::ValidatorContext            (constructor)
juniper::validation::visit_all_rules
juniper::validation::visit
juniper::validation::validate_input_values
juniper::validation::rules::disable_introspection::factory
juniper::RootNode::schema                        (field)
juniper::RootNode::introspection_disabled        (field)
```

They are **public-callable without `unsafe` and without private-field access**,
but they are **not stable public API** and carry no documentation-level or
semantic-versioning stability promise. **The phrase "stable public API" is not
used for them anywhere in the code or this report.** Any juniper version change
requires revalidating the whole gate before merge **and** before activation.
Because the gate is expressed as ordinary calls against those items, a surface
that is removed or re-typed **fails the build** rather than silently changing
behaviour — the implementation fails closed.

By contrast, the shim's and prefetch's surfaces (`new_error`,
`ExecutionError::path`, `look_ahead`, `children`, `field_original_name`) are
ordinary documented public API.

### A.8 Effective-variable construction

```text
effective_variables = operation_defaults  overridden_by  request_variables
```

Implemented as: start from `GraphQLRequest::variables()`; read the **selected**
operation's `variable_definitions`; for each variable declaring an
operation-level default, `entry(name).or_insert(default)` — preserving the
request value where one was supplied.

**Evidence this matches the pinned executor**: `execute_validated_query`
(`src/executor/mod.rs:828-853`) builds `default_variable_values` from
`operation.item.variable_definitions` filtering on `def.default_value`, clones
the request variables, then does exactly `all_vars.entry(name).or_insert(value)`.
`execute_validated_query_async` (`:926-949`) is identical.

`VariableDefinitions` and `VariableDefinition` are not publicly nameable, so they
are reached by **field access on the `Operation` value** without naming their
types.

**Raw request variables are NOT used for directive evaluation** — stated
explicitly, and enforced by `collect_top_level_occurrences` taking only the
effective map.

### A.9 Guard mode and derived store availability

Three states, `OFF` (the `#[derive(Default)]` default **and** the merged state),
`OBSERVE`, `ENFORCE`. Code evidence that availability is **derived**, not
configured:

```rust
// mutation_guard.rs — the sole answer to "may the store be used?"
pub fn store_available(self) -> bool { matches!(self, Self::Enforce) }

// batching.rs — the store holds the MODE, not a bool
pub(crate) struct GraphqlBatchStore { mode: MutationGuardMode, … }
pub(crate) fn new(mode: MutationGuardMode) -> Self { … }
pub(crate) fn is_available(&self) -> bool { self.mode.store_available() }

// model.rs — the only constructor taking availability takes a MODE
pub fn with_guard_mode(…, mode: MutationGuardMode) -> Self {
    Self { …, batch_store: GraphqlBatchStore::new(mode) }
}
pub fn new(…) -> Self { Self::with_guard_mode(…, MutationGuardMode::Off) }
```

There is **no** boolean, no second setting and no constructor anywhere that can
produce `OFF + store enabled` or `OBSERVE + store enabled` — those states are
**unrepresentable by construction**, not maintained by operator discipline.
Verified by inspection (above) *and* by test
(`store_state::store_availability_follows_the_mode_in_both_directions`, covering
`OFF -> OBSERVE -> ENFORCE` and `ENFORCE -> OBSERVE -> OFF`).

## B. Test-matrix results

100 tests added; **all passing**; no existing test modified.

### B.1 Store (11 + collision matrix)

empty result cached as `Loaded(empty)` ✔ · repeated read does not consume the
entry ✔ · same scope+loader+shape+key reuses (`AlreadyLoaded`, no SQL) ✔ ·
different scope does not reuse ✔ · different shape does not reuse ✔ ·
`LoadFailed` retained ✔ · no retry after `LoadFailed` (even against a working
pool) ✔ · whole-store invalidation across all scopes/loaders/shapes/keys ✔ ·
`OFF` unavailable ✔ · `OBSERVE` unavailable ✔ · `ENFORCE` available ✔ ·
duplicate keys → one key ✔ · partitioning determinism over repeated runs ✔ ·
partitioning correctness (every row in its own bucket) ✔ · prefetched output
equals direct per-parent output element-for-element and in order ✔ · one parent
key holding several shapes simultaneously, each correct ✔ · concurrent
independent requests share nothing ✔ · a fresh `Context` observes an empty store ✔

Full collision matrix: same key+loader+shape under **different scopes** never
collides ✔ · same scope+key, **different loaders** structurally separated ✔ ·
same scope+loader+key, **different shapes** never collide ✔ · the exact full key
returns the expected stored value ✔ · `LoadFailed` under scope A does **not**
poison scope B (B reads `NotLoaded` and can still load) ✔ · invalidation clears
both ✔

### B.2 Prefetch

direct path ✔ · indirect/descendant path ✔ · aliases at terminal and
intermediate segments ✔ · two aliased intermediate branches both discovered ✔ ·
repeated selections ✔ · two top-level fields → **2** bounded dispatches, not
`N + N` ✔ · sync execution ✔ · async execution ✔ · **no production field
adoption** ✔ · a list with no prefetch site falls back and is still correct ✔ ·
mixed covered/uncovered in one operation, both correct ✔

### B.3 Failure / error

prefetched vs direct failure contract compared ✔ · `errors[].path` ✔ · null
propagation ✔ · `extensions.type` ✔ · no empty substitution ✔ · no retry SQL ✔ ·
parent list field still resolves ✔

### B.4 Scope

aliases isolate namespaces ✔ · one scope's `LoadFailed` does not poison another ✔
· prefetch site and child resolver derive exactly the same scope ✔ · shim
fail-closed behaviour (documented and exercised through the `None` path) ✔ ·
shim side-effect freedom asserted on data and `errors[]` ✔

### B.5 Mutation

read-after-write within one top-level mutation field ✔ · isolation between two
top-level mutation fields ✔ · **shared terminal named fragment** across two
top-level fields still isolates — the case that rejected an execution-occurrence
scope ✔ · isolation under **async** execution (`multi_thread`, 4 workers), resting
on scope isolation rather than execution order ✔ · mutation payload fan-out
bounded as parent count rises ✔ · the duplicate form of the read-after-write
scenario is **rejected** with 0 resolvers and 0 writes ✔

**Async-interleaving proof — limitation stated honestly.** The pinned
`#[graphql_object]` macro and the sync-parity requirement of specification
section 3.2 make a *deliberately yielding* test-only mutation resolver
impractical: the fixture's resolvers must remain synchronous to satisfy both
`execute_sync` and async `execute`. The strongest achievable proof was therefore
used — the same two-top-level-field operation is executed through the genuine
async `juniper::execute` path on a multi-threaded runtime (where the pinned
executor drives top-level fields through `FuturesOrdered` and may interleave
them), and isolation is asserted **structurally**: each scope holds its own
entries (`entry_count() == 2`) and the second field observes its own write. The
isolation invariant itself is **not weakened** — it does not depend on execution
order, which is precisely why the architecture is order-independent.

### B.5a Query path and the non-mutation fast path

a valid query is never restricted and emits no event in `OFF`, `OBSERVE` and
`ENFORCE` ✔ · a valid query's response is **byte-identical** to the no-guard
baseline in every mode ✔ · an invalid query keeps juniper's canonical error
exactly and produces no guard event in any mode ✔ · a query with a duplicate
top-level response key is accepted, both occurrences **share one scope**, and the
second issues no additional terminal statement ✔

A non-mutation exits at operation-type discrimination, before document and input
validation and before any duplicate-key analysis, which is why it can emit no
event.

### B.6 Guard

direct duplicate ✔ · named-fragment duplicate ✔ · inline-fragment duplicate ✔ ·
repeated legitimate spreads both counted ✔ · distinct aliases accepted, each
executing exactly once ✔ · query duplicates unaffected ✔ · non-top-level
duplicates unaffected ✔ · operation selection evaluates only the **selected**
operation ✔ · `OFF` short-circuits before parsing (proven with a document that is
*both* syntactically invalid *and* a duplicate) ✔ · `OBSERVE` executes unchanged
and emits exactly one event ✔ · `ENFORCE` executes zero resolvers and zero
writes, **measured** ✔ · sync and async ✔ · HTTP status/body compared with a real
juniper validation error ✔ · event redaction asserted positively and negatively ✔
· rejection message exposes no loader/store/scope internals and does not imply
invalid GraphQL ✔ · mode default `OFF` verified by test ✔ · exactly three modes
parse; unknown values rejected ✔

**Measured zero-execution evidence**, for every rejected case (direct duplicate,
named-fragment duplicate, inline-fragment duplicate), under both execution paths:

```text
mutation resolver execution count = 0   (resolver-entry counter)
database write count              = 0   (imprint row count before == after)
```

Both are required and both were taken; neither alone was relied on.

**Baseline-invalid matrix.** Every document below is **both** baseline-invalid
**and** shaped as a duplicate top-level mutation response key, so a guard
ignoring the eligibility gate would visibly misbehave. For each, in **every**
mode: no guard rejection, no observation event; and in `OBSERVE`/`ENFORCE` the
serialized response body and `is_ok()` status are **byte-comparable to the
no-guard baseline**.

| Category | Failing gate | Guard decision | Event | Response vs no-guard baseline |
|---|---|---|---|---|
| unknown top-level field | document validation | none | none | identical |
| sub-selection on a scalar | document validation | none | none | identical |
| unknown directive | document validation | none | none | identical |
| `$skip: Boolean! = true` (non-null with a default) | document validation | none | none | identical |
| missing required variable | input validation | none | none | identical |
| invalid variable type | input validation | none | none | identical |
| two operations, no `operationName` | operation selection | none | none | identical |
| unknown `operationName` | operation selection | none | none | identical |
| syntactically invalid document | parse | none | none | identical |

The operation-selection cases specifically prove the guard makes **no** collision
decision before operation selection has succeeded. The gate returns, rewrites and
suppresses **no** validation error of its own.

**Error-shape parity**: a guard rejection and a real juniper validation failure
(unknown field) were compared directly — both `is_ok() == false` (HTTP 400 via
the existing branch), both carry an `errors` array whose first element has
**exactly the same key set** (`{message, locations}`), and **neither** carries a
`data` key.

### B.7 Directive / effective-variable matrix

Every row asserts the guard's verdict against **Juniper's observed execution** of
the same document and variables — a real resolver-invocation count from a real
execution — not against a separately written expectation table. Each row also
cross-checks that the guard rejects **exactly when** juniper would execute one
response key more than once.

| Case | Variables | Juniper executions | Guard | Result |
|---|---|---|---|---|
| `@skip(if: true)` literal | — | 1 | accept | ✔ |
| `@skip(if: false)` literal | — | 2 | **reject** | ✔ |
| `@include(if: false)` literal | — | 1 | accept | ✔ |
| `@include(if: true)` literal | — | 2 | **reject** | ✔ |
| `$skip: Boolean!` (no default) | `{skip: true}` | 1 | accept | ✔ |
| `$skip: Boolean!` (no default) | `{skip: false}` | 2 | **reject** | ✔ |
| `$skip: Boolean = true`, **omitted** | — | 1 | accept | ✔ |
| `$skip: Boolean = true`, overridden | `{skip: false}` | 2 | **reject** | ✔ |
| `$inc: Boolean = false`, **omitted** | — | 1 | accept | ✔ |
| `$inc: Boolean = false`, overridden | `{inc: true}` | 2 | **reject** | ✔ |
| defaulted directive on a **named fragment spread**, omitted | — | 1 | accept | ✔ |
| … overridden | `{skip: false}` | 2 | **reject** | ✔ |
| defaulted directive on an **inline fragment**, omitted | — | 1 | accept | ✔ |
| … overridden | `{skip: false}` | 2 | **reject** | ✔ |
| multiple directives, `skip=false include=true` | defaults | 2 | **reject** | ✔ |
| multiple directives, `skip=true` wins | `{skip: true}` | 1 | accept | ✔ |
| multiple directives, `include=false` | `{inc: false}` | 1 | accept | ✔ |

An omitted-but-defaulted variable is treated as **resolved**, never as
undecidable. Defaulted-variable documents declare the variable **nullable**
(`$skip: Boolean = true`), because the pinned `default_values_of_correct_type`
rule rejects `Boolean! = true` outright — that form is covered separately in the
baseline-invalid matrix, where it belongs.

**Conservative-rejection tradeoff, stated explicitly**: where a directive
condition genuinely cannot be resolved after applying operation defaults, the
occurrence is counted as executable and the request is rejected. This is
deliberate and is **not exact**; it errs toward refusing a request rather than
admitting a possible duplicate write.

### B.8 Performance / control

`OFF` short-circuits before parsing — proven with a document that is *both*
unparseable *and* a duplicate: the guard returns `Proceed` with no event, which
is only possible if it never parsed ✔

**Overhead, stated accurately.** In `OBSERVE` and `ENFORCE`:

| Cost | Applies to | Modes |
|---|---|---|
| parse + operation selection | **every GraphQL request** | `OBSERVE`, `ENFORCE` |
| document/schema + input-variable validation | mutations only (via the fast path) | `OBSERVE`, `ENFORCE` |
| duplicate-key traversal | mutations only | both; rejection in `ENFORCE` only |

So **every** request is parsed and its operation selected **twice** — once by the
gate, once inside `GraphQLRequest::execute()` — and mutations are additionally
validated **twice**. `OFF` short-circuits ahead of all of it, on every request of
every kind.

The phrases **"one extra parse"** and **"bounded to mutations"** are withdrawn
and do not appear in this report.

**No production latency thresholds have been invented.** No wall-clock figure is
offered as an acceptance metric; the acceptance signal is SQL statement count and
bounded database work.

## 13. Known limitations and deferred work

1. **The repository's `.map_err(Into::into)` idiom silently drops
   `extensions.type`.** Juniper carries a blanket
   `impl<T: Display, S> From<T> for FieldError<S>`, so converting a `ThothError`
   with `?` or `Into::into` produces a `FieldError` with **no** extensions —
   `ThothError::into_field_error` is never called. `ADR-0006` section 4.9.3
   assumes the direct path carries the discriminant. The fixture's terminal
   resolver therefore uses `IntoFieldError` explicitly on **both** paths, which
   is what makes the classification comparison meaningful. **Existing production
   child resolvers using `FieldResult` + `.map_err(Into::into)` currently emit no
   `extensions.type`.** Changing 63 production resolvers is out of scope here;
   an adopting task must use `IntoFieldError` to obtain equivalence, and this is
   flagged for the architecture owner.
2. **No preview/staging performance evidence** (section 12.3) — an activation
   prerequisite, not a merge prerequisite.
3. **CG-13 runtime-operations questions 1 and 3–10 are unanswered** (section 12.1).
4. **No authoritative monitoring baseline or thresholds** (section 12.2).
5. **Async-interleaving fixture limitation** — documented in B.5; the isolation
   invariant is established, the *yielding-fixture technique* is not used.
6. **`LoaderIdentity` / `LoadShapeKey` carry a compiled-in proof-loader
   discriminant** in production builds (section 5, decision 2). Never
   constructed outside tests.
7. **Chunking is not implemented.** `ADR-0006` section 4.10 permits this because
   batch size is bounded above by the parent list, which the repository's
   `limit`/`offset` pagination already bounds. No unbounded bind-parameter list
   is constructed by any site delivered here. An adopting task reaching a site
   with an unbounded parent count must chunk.

**The foundation declares NO production field N+1 compliant.** The
adoption-coverage obligations of `ADR-0006` section 4.18.2 attach to adopting
tasks, which must perform their own exact-base fan-out path inventory and produce
their own per-path statement-count evidence. The existence of the correctness
fallback is **not** N+1 compliance evidence.

## 14. Unresolved issues

- Production activation is **BLOCKED — runtime-operations / CG-13 and
  monitoring-threshold evidence unverified**.
- Limitation 1 above (production `extensions.type` loss) warrants an architecture
  owner decision; it is pre-existing and was surfaced, not caused, by this work.

## 15. Agent self-assessment

The implementing agent **does not approve this task** and has not reviewed its
own work. Fresh independent cross-model review of the exact head is required, and
because the task is HIGH risk, separate explicit CTO merge authorization bound to
the independently approved exact head is additionally required.

Suggested review focus:

1. **Load-shape normalization** — the one part of the design whose failure
   returns confidently *wrong* data rather than a miss, because the correctness
   fallback does not cover a shape collision. Check that
   `TestImprintLoader::shape` is genuinely the only construction path and that
   `shape_from_selection` cannot disagree with the child resolver's
   default-applied argument.
2. **The eligibility gate's fidelity to juniper's pipeline**, and specifically
   whether the deliberate selection-before-validation reordering can ever let a
   request reach a *decision* without having passed every gate.
3. **Cycle safety vs occurrence counting** in fragment expansion — the path-stack
   choice is load-bearing and a global "seen" set would silently under-count.
4. **The store's `Any` downcast** in `lookup` — whether the
   `(loader identity, type parameter)` pairing can ever be violated.
5. **The conservative-rejection tradeoff** — whether any realistic legitimate
   document is rejected by treating an unresolvable directive condition as
   executable.
6. **Limitation 1** — whether the `extensions.type` finding should block
   adoption, and whether the fixture's use of `IntoFieldError` makes the
   equivalence comparison too favourable to the prefetched path.
7. **Store availability derivation** — confirm by inspection, not only by test,
   that no constructor anywhere can produce `OFF`/`OBSERVE` + an available store.
