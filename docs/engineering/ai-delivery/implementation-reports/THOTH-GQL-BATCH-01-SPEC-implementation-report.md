# THOTH-GQL-BATCH-01-SPEC Implementation Report

Documentation-only architecture and specification-authoring task. It writes
`ADR-0006` as a `PROPOSED` shared architecture decision and the bounded
`THOTH-GQL-BATCH-01` runtime implementation specification. It implements no
runtime code and authorizes nothing.

This report covers the original authoring **and** the bounded remediation of an
independent architecture review that returned `CHANGES REQUIRED`. Section 16
records the review findings and exactly what changed in response. The live review
record is the GitHub pull-request history; per `ADR-0005` no review or approval
identifier is copied into this file.

## 1. Repository state

Repository: `thoth-pub/thoth`
Workflow: STANDARD
Base branch: `develop`
Base commit: `5a8c27b1b7c11a4f6bd26d459556468099f8c1f4` (freshly verified by
`git ls-remote origin refs/heads/develop` immediately before branching, and
before any edit)
PR target: `develop`
Programme integration branch: None
Task branch: `feature/shared-architecture/graphql-batching-spec`
Head commit: the current head of that branch; the exact reviewed head is the
GitHub pull-request record
Pull request: draft PR to `develop`; the live record is GitHub
Expected branch deletion after merge: YES
Final programme PR required: NO
Implementing model: Claude Opus 5
Reasoning level: HIGH

## 2. Scope confirmation

Approved specification: this is the architecture and specification-authoring task
itself. The governing controls are root `AGENTS.md`, `docs/engineering/AGENTS.md`,
`thoth-api/AGENTS.md`, `operating-model.md` (Gate 0 and Gate 1),
`decisions/README.md`, `task-specification-template.md`,
`risk-classification.md`, `release-gates.md` and `ADR-0005`.

Implemented objective: record the CTO's Option A direction as a `PROPOSED`
shared architecture decision, determine the exact request-scoped mechanism that
is technically viable against the pinned Juniper/Diesel stack, and specify the
bounded runtime task that would implement it.

Out-of-scope changes made: NONE.

Specifically not done, by instruction:

- PR [#788](https://github.com/thoth-pub/thoth/pull/788) and the
  `feature/publisher-services/be-02-spec` branch are untouched;
- [issue #765](https://github.com/thoth-pub/thoth/issues/765) is untouched;
- `ADR-0001` through `ADR-0005` are unchanged;
- the Publisher Services platform inventory is unchanged;
- `docs/publisher-services/task-status.md` is deliberately **not** edited,
  because PR #788 currently modifies that tracker and concurrent edits would
  create an avoidable conflict;
- no runtime source file, migration, schema, Cargo manifest, lock file or
  workflow is changed;
- the future implementation branch `feature/shared-architecture/graphql-batching`
  is **not** created.

## 3. Commits

- see the GitHub pull-request record for the exact commit list and head.

## 4. Files changed

- `docs/engineering/decisions/ADR-0006-request-scoped-graphql-batching.md`
  - reason: record the shared architecture decision implementing the CTO's
    Option A direction, at `PROPOSED` status.
  - behavioural effect: none. A `PROPOSED` ADR authorizes nothing; per
    `decisions/README.md` an ADR is authoritative only when `APPROVED`.

- `docs/engineering/decisions/decision-register.md`
  - reason: register `ADR-0006`, its status, affected programmes, open approval
    blocker and dependency order.
  - behavioural effect: none. Register content only. No existing row changed.

- `docs/engineering/ai-delivery/tasks/THOTH-GQL-BATCH-01.md`
  - reason: the bounded runtime implementation specification required by Gate 1
    before any implementation could be scoped.
  - behavioural effect: none. `DRAFT`, unapproved and unauthorized.

- `docs/engineering/ai-delivery/README.md`
  - reason: the README indexes task records; the new task record is added to
    that index per live repository convention.
  - behavioural effect: none.

- `docs/engineering/ai-delivery/implementation-reports/THOTH-GQL-BATCH-01-SPEC-implementation-report.md`
  - reason: this report, required by root `AGENTS.md` section 14.
  - behavioural effect: none.

- `CHANGELOG.md`
  - reason: required by root `AGENTS.md` section 13 for every PR.
  - behavioural effect: none.

## 5. Implementation decisions

Decisions made within the authoring scope:

1. **Option A was proven implementable before being written as binding
   architecture, and the conventional loader shape was rejected on evidence.**
   The prompt explicitly forbade assuming that `loader.load(parent_id)` on
   `Context` would batch. Three findings in the pinned sources establish that it
   would not:
   - `juniper_codegen` 0.16.0 wraps a non-`async` resolver body in
     `futures::future::ready(..)` (`graphql_object/mod.rs:719-721`), so the body
     runs at future-construction time. Sibling child resolvers cannot accumulate
     keys and defer;
   - the sync `resolve_field` generated for an `async` field is a `panic!`
     (`graphql_object/mod.rs:629-636`), and the entire GraphQL test suite runs
     through `juniper::execute_sync`, so converting resolvers to `async fn`
     would panic in tests;
   - neither `resolve_into_list_async` (`juniper` 0.16.2,
     `src/types/containers.rs:585`) nor
     `resolve_selection_set_into_async_recursive` (`src/types/async_await.rs:196`)
     exposes an "all siblings pending" dispatch signal.

2. **The selected mechanism is look-ahead-driven set-based prefetch into
   request-scoped state (`ADR-0006` variant A2).** It is fully synchronous, so
   it behaves identically under `execute_sync` and async `execute`; it requires
   no new dependency and no execution-model change. Viability was confirmed
   against `Executor::look_ahead` (`src/executor/mod.rs:694`),
   `LookAheadChildren::has_child`/`select` (`src/executor/look_ahead.rs:426,439`)
   and the codegen's executor-argument recognition
   (`juniper_codegen` `common/field/arg.rs:358,388,395`).

3. **A correctness fallback is mandatory, not optional.** `look_ahead()` in the
   pinned version does not evaluate `@skip`/`@include` — the implementation
   carries an explicit `// TODO: support excludes` (`src/executor/mod.rs:709`).
   The ADR therefore requires the store to distinguish "loaded, empty" from "not
   loaded", and requires the child resolver to fall back to its direct query on a
   miss. Correctness is independent of look-ahead accuracy; batching is the
   optimisation layered on top.

4. **Read-after-write coherence is settled structurally rather than by editing
   mutations.** A GraphQL operation is a query or a mutation, never both, so the
   only staleness risk is a prefetch site reachable from a mutation payload.
   `ADR-0006` section 4.12 confines prefetch sites to query-root list fields,
   requires a test proving coherence, and provides an invalidation entry point
   for any future site that is reachable from a mutation payload. The residual
   risk — that the rule is enforced by review and test rather than by the type
   system — is recorded explicitly rather than glossed.

5. **The proof consumer is a test-only GraphQL schema, not a production
   resolver.** This satisfies the instruction to prefer implementing the
   foundation without changing any existing resolver, and keeps the public
   schema unchanged. `BE-02`'s `DistributionPlatform` cannot be the proof target
   because it does not exist and `BE-02` cannot start until the foundation is
   complete.

6. **A factual correction was recorded in `ADR-0006` section 1.5 rather than by
   editing `BE-02`.** `BE-02` section 9.2.1 describes the repository as using
   Juniper 0.16 sync execution (`execute_sync`). That is true of the test suite
   but not of production, which uses async `execute`
   (`thoth-api-server/src/lib.rs:98`). The distinction matters, because it is
   what eliminates the deferred-dispatch loader. PR #788 was not modified; the
   correction is recorded in the shared ADR, and the `BE-02` amendment is
   separate future work on that existing pull request.

7. **Risk is classified `HIGH` for the runtime task**, on the grounds that it
   adds shared state to the `Context` used by every resolver and substitutes a
   loaded result for a field's direct database read. The report records
   explicitly that neither "no migration" nor "no field adopts it at merge"
   reduces the classification: both limit blast radius at merge, not the
   correctness surface of the mechanism.

8. **`docs/publisher-services/task-status.md` was deliberately left unedited**
   to avoid a concurrent edit with PR #788.

Deviation from the specification: NONE. There was no prior specification for
this authoring task; the controls listed in section 2 governed it.

## 6. Database and migration effects

Migration added: NO

No migration, no schema change, no data effect. `thoth-api/src/schema.rs`,
`thoth-api/migrations/`, `Cargo.toml` and `Cargo.lock` are byte-identical to the
base.

`ADR-0006` records "no database migration" as an expectation for the future
runtime task and makes a required migration an explicit stop condition, because
it would materially change that task's risk and scope.

## 7. API and compatibility effects

GraphQL/API changes: none. No runtime file changed.
Generated schema/client updates: none.
Backwards compatibility: unaffected.
Deprecations: none.
Cross-repository dependencies: none.

`ADR-0006` invariant 8 and the `THOTH-GQL-BATCH-01` acceptance criteria require
the *future* implementation to introduce no public schema change either, and
require the generated SDL to be byte-identical to its base.

## 8. Authorization and security

Authorization paths changed: none. `thoth-api/src/policy.rs` is unchanged.
Roles/scopes involved: none in this task.
Negative authorization tests: not applicable to a documentation-only change.
Secret or personal-data handling: none.

The architecture's security position is stated in `ADR-0006` section 9: prefetch
key sets may contain only keys taken from parents the request has already
resolved and the parent resolver has already authorized, so batching cannot
introduce a key the caller could not already have fetched individually. The
generic-loader hazard — assuming every key in a request is authorized — is named
and prohibited.

Security limitation recorded: `ADR-0006` section 4.13 requires that a loader
over data protected at the *child* level, rather than inherited from the parent,
must carry authorization context or run after the child-level check. No such
loader exists in the foundation.

## 9. Tests and checks

Documentation-only change. Per root `AGENTS.md` section 8 and
`docs/engineering/AGENTS.md` section 6, the required check is `git diff --check`
plus link, heading, terminology and changelog verification.

### Whitespace and conflict markers

Command:

```text
git diff --check
```

Result: recorded on the pull request; no output expected, which is the pass
condition.

### Documentation verification performed

- relative links from `docs/engineering/decisions/` and
  `docs/engineering/ai-delivery/tasks/` resolve to existing files;
- ADR numbering: `ADR-0006` is the next unused number. Verified that no
  `ADR-0006` file, no `THOTH-GQL-BATCH-01` reference and no corresponding branch
  existed anywhere in the repository or on the remote before this task;
- decision-register consistency: one new row, correct status, no existing row
  altered;
- repository and branch names use canonical forms;
- no transient status prose (`PENDING MERGE`, `AWAITING REVIEW`) is written into
  any committed file, per `docs/engineering/AGENTS.md` section 1.1 and
  `ADR-0005`;
- no approval-state-only content is recorded, per `ADR-0005` invariant 12;
- `git diff --stat` confirms that no runtime file, migration, schema file,
  Cargo manifest, lock file or workflow changed;
- PR #788's branch and issue #765 are untouched.

### Rust workspace checks

Not run, and not required: no Rust file changed. Recording them as run would be
false evidence.

## 10. Manual verification

Environment: local checkout of `thoth-pub/thoth` at base
`5a8c27b1b7c11a4f6bd26d459556468099f8c1f4`.

Steps: read the pinned `juniper` 0.16.2, `juniper_codegen` 0.16.0 and `diesel`
2.3.10 sources resolved by the workspace `Cargo.lock`, and the live GraphQL
`Context`, its construction site, the execution calls, representative child
resolvers, the policy layer and the database test helpers.

Observed result: the findings recorded in `ADR-0006` sections 1.2, 1.5, 3.1 and
8.1, each cited to an exact file and line.

Evidence link: the file and line references in `ADR-0006` and in section 5 of
this report are directly checkable against the pinned sources and the base
commit.

## 11. CI

CI status: PENDING at the time of writing; the live status is the GitHub
pull-request record.

Checks: the repository classifies documentation-only changes and skips the heavy
Rust, migration and Docker jobs (PR
[#771](https://github.com/thoth-pub/thoth/pull/771)). Jobs skipped by that
classification must be reported as **skipped**, not as passed.
`check_changelog.yml` is expected to run and is satisfied by the `CHANGELOG.md`
entry.

Failures or warnings: to be recorded from the live CI record.

## 12. Rollout and rollback

Initial state after merge: `ADR-0006` is present at `PROPOSED` and authorizes
nothing; `THOTH-GQL-BATCH-01` is present at `DRAFT` and authorizes nothing. No
runtime behaviour changes.

Activation required: not applicable. There is no runtime effect to activate.

Feature flag/configuration: none.

Migration sequence: none.

Rollback/disable procedure: revert the merge commit. No data or runtime effect
exists to unwind.

Monitoring required: none.

## 13. Known limitations and deferred work

- `ADR-0006` is `PROPOSED`. It is not authoritative and no implementation may
  rely on it until it is `APPROVED` and the approved content is reachable from
  `develop`.
- `THOTH-GQL-BATCH-01` is `DRAFT`, unapproved and unauthorized. The
  implementation branch has not been created.
- The `BE-02` amendment described in `ADR-0006` section 12 is **not** performed
  here. PR #788 still carries its open N+1 architecture gate, and amending it is
  separate future work on that existing pull request.
- The evidence-led N+1 inventory of existing child resolvers (`ADR-0006`
  section 10) is deferred to separate follow-up work and is deliberately not
  created here.
- `docs/publisher-services/task-status.md` is not reconciled in this PR, to
  avoid a concurrent edit with PR #788. Reconciling the Publisher Services
  tracker with the `ADR-0006` dependency is follow-up work once PR #788 settles.
- The feasibility findings are bound to the currently pinned `juniper` 0.16.2
  and `juniper_codegen` 0.16.0. A Juniper upgrade could change the codegen
  behaviour that variant A1 was rejected on, and would warrant re-examining
  `ADR-0006` section 3.1.

## 14. Unresolved issues

- The choice between `ADR-0006`'s Option A variants is settled in the ADR, but
  the ADR itself remains unapproved. The open decision is the CTO's approval of
  this exact content.

## 15. Agent self-assessment

The agent may identify risks but may not approve the task.

Suggested review focus:

1. **The feasibility argument in `ADR-0006` section 3.1.** It is the load-bearing
   claim. Verify the three cited findings against the pinned sources —
   `graphql_object/mod.rs:719-721` and `:629-636` in `juniper_codegen` 0.16.0,
   and the `FuturesOrdered` drivers in `juniper` 0.16.2 — and confirm that they
   do rule out a deferred-dispatch loader rather than merely complicating it.
2. **The dual execution paths.** Production uses async `execute`; the test suite
   uses `execute_sync`. Confirm the selected mechanism genuinely works under
   both, and that this corrects rather than contradicts `BE-02` section 9.2.1.
3. **Read-after-write coherence (`ADR-0006` section 4.12).** The rule is
   structural — prefetch sites unreachable from mutation payloads — and is
   enforced by review and test, not by the compiler. Assess whether that is
   sufficient for a HIGH-risk shared mechanism, or whether a stronger
   enforcement should be required before approval.
4. **The "loaded, empty" versus "not loaded" distinction (section 4.7).**
   Collapsing it would silently convert a missing prefetch into a wrong empty
   result. Confirm the ADR makes the distinction binding.
5. **Query-count evidence (section 8.1).** Confirm that
   `diesel::connection::set_default_instrumentation` being a global default
   consumed at connection establishment is workable under the existing
   `OnceLock` pool and exclusive test lock, and that the fallback wording does
   not permit narrative evidence.
6. **Risk classification.** `HIGH` is proposed. Confirm the rationale, and in
   particular the claim that no-migration and no-adoption-at-merge do not reduce
   it.
7. **Boundary compliance.** Confirm no runtime file, dependency, migration,
   workflow, `BE-02` file, PR #788 content or issue #765 content changed, and
   that no approval-state or transient-status prose was written.

Items 4 and 5 were among the review's actual findings and are now remediated;
see section 16.

---

## 16. Independent review remediation

### 16.1 Review outcome

An independent architecture/specification review of the reviewed head returned:

```text
CHANGES REQUIRED
```

with three P1 findings and one P2 finding. The core architecture direction —
Option A, variant A2, look-ahead-driven set-based prefetch into request-scoped
state — was **accepted** and is unchanged. Options B, C and D were not
reconsidered, because remediation did not prove A2 impossible.

Per `ADR-0005` and `docs/engineering/AGENTS.md` section 1.1, the review
identifier, its decision record and the merge history are terminal GitHub
evidence and are deliberately **not** copied into this file.

Because the exact head changes with this remediation, the previous exact-head
review does not carry forward. A fresh independent exact-head review is
mandatory.

### 16.2 Finding 1 (P1) — cache identity ignored field arguments

**Problem.** The store was keyed by `(loader identity, parent key)`. That is safe
only for an argument-free field. Thoth's existing child fields already take
result-changing arguments — `Publisher.imprints` takes `limit`, `offset`,
`filter`, `order`; `Publisher.contacts` takes `limit`, `offset`, `order`,
`contactTypes` — so `a: contacts(limit: 1)` and `b: contacts(limit: 100)` would
have shared one bucket and one of them would have been wrong.

**Remediation.** Store identity is now
`(loader identity, normalized load shape, parent key)`, settled in `ADR-0006`
sections 4.4-4.4.6:

- the shape is a typed, loader-specific value; free-form serialized GraphQL
  argument strings are prohibited as the canonical key;
- it includes every argument or semantic input that can change the result;
- defaults normalize: an omitted argument and an explicitly supplied schema
  default produce the same shape;
- semantically different shapes never share a bucket;
- an argument-free field uses a unit shape;
- dispatch is once per unique `(loader identity, load shape)` over all relevant
  keys — not one global dispatch across variants, and not one per parent.

**New evidence recorded.** `LookAheadSelection::arguments()`
(`juniper` 0.16.2, `src/executor/look_ahead.rs:577-590`) reads only the arguments
literally present in the query AST and does **not** apply schema defaults, while
the child resolver receives the default-applied value. Naive shape construction
would therefore give `contacts(limit: 100)` and `contacts` different shapes.
`ADR-0006` section 4.4.3 makes explicit default normalization binding and
requires a single loader-owned shape constructor shared by the prefetch site and
the child lookup, so the two cannot drift.
`LookAheadArgument::value()` (`:364-370`) resolves GraphQL variables, so
variable-supplied arguments work.

**`BE-02`.** `Publisher.distributionPlatforms` takes no field arguments in its
approved contract, so its shape is `Unit` and it batches by
`(PublisherDistributionPlatforms, Unit, publisher_id)`. Recorded explicitly in
`ADR-0006` section 4.4.5. No argument was added to that field and the approved
`BE-02` API contract is unchanged.

**Proof without adopting a production field.** `ADR-0006` section 4.4.6 and the
task's section 3.1 require the `#[cfg(test)]` fixture to define its own
argument-bearing field, including one argument with a schema default, so
multi-shape behaviour is proven against real Juniper argument handling without
migrating any production resolver.

### 16.3 Finding 2 (P1) — batch-failure state was internally inconsistent

**Problem.** The reviewed draft required both that a failed prefetch leave the
affected keys **absent** from the store, and that no fallback run after a failed
prefetch. Under A2 those cannot both hold: absence is exactly the signal that
triggers the fallback. The draft also left it ambiguous whether the parent list
resolver should return the prefetch error, which would have attributed the
failure to a field that succeeded.

**Remediation.** `ADR-0006` section 4.7 now defines three states, and section 4.9
settles error ownership:

```text
NotLoaded          -> child executes the ordinary direct fallback query
Loaded(Vec<V>)     -> child returns the rows, no query; includes Loaded([])
LoadFailed(error)  -> child returns the derived FieldError, no query, no retry
```

Sequence (section 4.9.1): the parent list resolver obtains its parents
successfully; the prefetch is attempted; on failure a `LoadFailed` outcome is
recorded for the covered keys, the parent list resolver still returns its parents
normally, each covered child resolver emits the error, and no retry query is
issued. Retrying per parent after a batch failure would reintroduce the `1 + N`
access the decision exists to prevent, on a path where the database is already
unhealthy.

**Storage granularity** (section 4.9.2): once per `(loader, shape)` dispatch,
with the attempted key set. Justified because one statement failed, so one
failure occurred, and it failed the whole dispatch; per-key storage would
duplicate one error `n` times and imply failures could differ per key within one
statement. Recorded alongside: `ThothError` is **not** `Clone`
(`thoth-errors/src/lib.rs:11`), so the implementation must retain a shareable
representation.

**Equivalence contract** (section 4.9.3): identical error text is explicitly
insufficient. An adopting task must verify `errors[].path`, null propagation,
`extensions.type` — Thoth maps `ThothError` to a `FieldError` carrying a `type`
discriminant (`thoth-errors/src/lib.rs:183-207`) — no empty-list substitution,
and no additional fallback SQL. Any intentional difference must be documented;
hiding one is prohibited.

**One difference is documented rather than hidden** (section 4.9.4): a prefetch
failure fails every covered key for that shape, including keys whose individual
query might have succeeded. This is accepted as fail-closed, given that per-key
retry is prohibited.

### 16.4 Finding 3 (P1) — N+1 compliance requires path coverage

**Problem.** The always-correct fallback makes a loader-backed field correct on an
uncovered path; it does not make that path N+1 compliant. The reviewed draft did
not separate the two properties, so "the field is loader-backed" could be read as
compliance evidence.

**Remediation.** `ADR-0006` section 4.18 makes the distinction binding:

```text
Correctness      -> the field returns the correct result on every path,
                    because a direct per-parent fallback always exists.
N+1 compliance   -> every material list/fan-out path capable of producing
                    N child queries is covered by set-based prefetch, and
                    that coverage is measured.
```

Section 4.18.2 adds a binding adoption coverage rule: inventory all fan-out paths
at the exact implementation base, classify each as covered / inherently
single-parent / explicitly excluded, add a prefetch site for every path required
for compliance, measure every covered path or path class, and retain the fallback
for correctness — but never count the fallback's existence as compliance
evidence. A task that cannot achieve coverage within scope must escalate.

**New evidence recorded** (section 4.18.1). `Publisher` is reachable under a
fan-out by more than its own root list query, verified at the base commit:
`QueryRoot.publishers` (`query.rs:521`); `QueryRoot.imprints` (`query.rs:593`)
`-> Imprint.publisher` (`model.rs:1366`); and `QueryRoot.contacts`
(`query.rs:1868`) or `Publisher.contacts` `-> Contact.publisher`
(`model.rs:3120`). So `imprints(limit: 100) { publisher { distributionPlatforms
{ platform } } }` fans out over publishers without touching the `publishers`
root query at all.

**`BE-02` consequence** (section 4.18.3): the `BE-02` adoption task must perform
the exact-base path inventory itself. The four routes listed are an explicit
**minimum investigation set, not a complete answer**, and `BE-02` must search its
own base. The inventory belongs to `BE-02`, not to `THOTH-GQL-BATCH-01`, which
adopts no production field.

**What the foundation must instead prove:** that the mechanism supports several
prefetch sites for one `(loader, shape)` in a single request without duplicate
loading. The task's section 3.1 now requires two prefetch sites in the test
fixture.

**Legacy scope unchanged** (section 4.18.4): coverage obligations attach to a
field when it adopts a loader. Existing child resolvers adopt nothing, so section
10's evidence-led legacy policy is unaffected.

### 16.5 Finding 4 (P2) — SQL instrumentation versus the `OnceLock` pool

**Problem.** `diesel::connection::set_default_instrumentation` applies only to
connections established after installation. The repository's ordinary test pool
is a process-wide `OnceLock<Arc<PgPool>>`
(`thoth-api/src/model/tests.rs:36,63-70`) that may already hold established
connections, and the exclusive test file lock serialises tests without recreating
connections. The reviewed draft's wording could have trapped the implementing
agent into measuring through that pool.

**Remediation.** `ADR-0006` section 8.1.1 and the task's performance section now
require a **dedicated measurement pool constructed after the hook is installed**:
acquire the exclusive lock, reset the disposable database, install the hook,
construct a new pool, run the measured operation through it, count `StartQuery`
events, and isolate the count from setup and migration statements. Any
equivalent actual-SQL observer — for example PostgreSQL statement-log capture —
remains acceptable. Application-level loader counters remain insufficient,
because they cannot see a per-parent statement issued by a fallback path.

The evidence format is now explicit:
`parent count | prefetch child-query count | direct baseline child-query count`,
for at least two values of `n`, with the prefetched count bounded while the
baseline grows.

### 16.6 Read-after-write coherence

Retained unchanged in substance: prefetch sites must initially be unreachable
from `MutationRoot` payload paths, with the required coherence test.

Updated for the corrected state model: the invalidation entry point is now
explicitly **whole-store** — clearing `Loaded` state, `LoadFailed` state, every
load shape and every loader. A narrower `(loader)` / `(loader, shape)` /
`(loader, shape, key)` primitive is deliberately **not** provided, on the grounds
that no evidence yet shows one is needed and a narrower invalidation is the
easier of the two to get subtly wrong.

No mutation resolver is touched. The task's stop conditions now state explicitly
that if the corrected load-shape or `LoadFailed` model turned out to require
mutation-resolver changes, the implementing agent must report `BLOCKED` rather
than widen scope.

### 16.7 Accepted architecture preserved

All previously accepted properties are retained: A2 selected; parent-list
look-ahead drives set-based prefetch; no external DataLoader dependency; no
execution-model migration; both the production async path and the sync test path
must work; request-local state only; no global or static cache; raw canonical
rows rather than GraphQL objects; set-based SQL only; duplicate keys
de-duplicated; the direct fallback mandatory on a genuine `NotLoaded` miss;
`Loaded([])` distinct from `NotLoaded`; database errors fail closed; no
authorization broadening; no migration or schema change; the foundation ships
with no production consumer; no existing production child resolver migrated by
`THOTH-GQL-BATCH-01`; `BE-02` the first required production consumer; legacy N+1
remediation separate, evidence-led work.

### 16.8 Files changed by the remediation

- `docs/engineering/decisions/ADR-0006-request-scoped-graphql-batching.md` —
  sections 4.4-4.4.6 (load shape), 4.5/4.5.1 (loader signature, set-based
  per-parent pagination), 4.6 (per-shape duplicate/alias rules), 4.7 (three-state
  store), 4.8, 4.9-4.9.4 (error ownership), 4.11, 4.12 (whole-store
  invalidation), 4.15/4.15.1 (selection enumeration), 4.16, new 4.18-4.18.4
  (adoption coverage), invariants 5, 6 and new 13-16, section 6 table, 8.1/8.1.1
  and 8.2 (measurement), 11 (negatives and risks), 12 (`BE-02` obligations), 13
  (validation).
- `docs/engineering/ai-delivery/tasks/THOTH-GQL-BATCH-01.md` — sections 2.1
  (new live findings), 3 (scope items 2-15), 3.1 (fixture requirements), 5
  (invariants 5, 6, 13-16), 6.1/6.2/6.4, 9 (acceptance criteria), 10 (tests),
  13 (stop conditions), 14 (report contents), 15 (reasoning emphasis), new 18.1.
- `docs/engineering/ai-delivery/implementation-reports/THOTH-GQL-BATCH-01-SPEC-implementation-report.md` —
  this section and the header note.
- `docs/engineering/decisions/decision-register.md` — the correctness versus
  N+1-compliance distinction and the `BE-02` inventory obligation.
- `CHANGELOG.md` — the remediation entry.

`docs/engineering/ai-delivery/README.md` was reviewed and needs no change: its
one-line task index remains accurate.

### 16.9 Residual risks after remediation

- **Shape-normalization correctness** is now the highest-consequence residual
  risk, and it is new. A shape that omits a result-changing argument lets two
  semantically different requests share one bucket. Unlike every other failure
  mode in this design, the correctness fallback does **not** cover it: it returns
  confidently wrong data rather than a miss. Mitigations: typed loader-owned
  shapes rather than serialized strings, one constructor shared by prefetch and
  lookup, and required non-collision and default-normalization tests. Recorded in
  `ADR-0006` section 11 and reflected in the task's recommended reasoning level.
- **Look-ahead directive reporting.** `look_ahead()` ignores `@skip`/`@include`
  (pinned `// TODO: support excludes`, `src/executor/mod.rs:709`), so a prefetch
  may be issued for an excluded field, or skipped for some fragment shapes.
  Over-reporting costs one query; under-reporting falls back. Neither is a
  correctness defect.
- **Review-enforced mutation reachability.** The rule confining prefetch sites to
  resolvers unreachable from mutation payloads is enforced by review and by the
  coherence test, not by the type system. Unchanged by this remediation and
  still accepted.
- **Explicit opt-in and coverage.** Adoption remains per-field and per-path, so a
  field can be correct without being compliant. Section 4.18 converts this from
  an unstated gap into a binding, measurable obligation on adopting tasks — but
  the obligation is still discharged by those tasks, not by the foundation.
- **Selection-enumeration correctness.** A prefetch site using `select()` /
  `has_child()` would silently miss aliases and defeat batching without breaking
  correctness. Mitigated by the binding requirement to filter on
  `field_original_name()` and by query-count measurement, which fails a field
  that never batches.

### 16.10 Boundaries confirmed after remediation

- no runtime source, schema, migration, `Cargo` manifest, lock file or workflow
  changed;
- PR #788, its branch and the `BE-02` specification unmodified;
- issue #765 unmodified;
- `ADR-0006` remains `PROPOSED`; `THOTH-GQL-BATCH-01` remains `DRAFT`;
- the implementation branch `feature/shared-architecture/graphql-batching` was
  not created;
- no approval-state or transient-status prose was written into any committed
  file.
