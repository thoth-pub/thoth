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

---

## 17. Second independent review remediation

### 17.1 Review outcome

```text
Independent decision: CHANGES REQUIRED
Reviewed head: 42bdb7b32109f03b32f7fa54a0cc40ee2a8662a4
```

The core direction — Option A, variant A2, look-ahead-driven set-based prefetch
into request-scoped state — remains **accepted in principle** and is unchanged.
Two newly discovered P1 architecture inconsistencies and one P2 PR-metadata
correction were raised. The four findings remediated in section 16 were not
regressed; section 17.6 records the check.

One of the two P1 findings is resolved in the architecture. The other is
**escalated to the CTO as an open decision**, because the investigation the
review directed showed the candidate architecture is not implementable on the
pinned stack.

Per `ADR-0005` and `docs/engineering/AGENTS.md` section 1.1, the review
identifier and its decision record are terminal GitHub evidence and are
deliberately **not** copied into this file.

The exact head changes with this remediation, so no previous exact-head review
carries forward. A fresh independent exact-head review is mandatory.

### 17.2 Finding 1 (P1) — descendant fan-out was required but unspecified

**Exact conflict.** `ADR-0006` section 4.18.2 required an adopting task to cover
**every** material fan-out path, and section 4.18.1 itself identified paths where
the loader-backed field is not a direct child of the list item. But A2 was
specified only as:

```text
parent list resolver -> directly requested loader-backed child field
                     -> derive keys from the returned parents -> prefetch
```

So the ADR mandated coverage of paths its own mechanism could not express. Left
unremediated, `BE-02` would have had to invent a second batching architecture for
descendant paths inside a programme task — exactly the escalation `ADR-0006`
exists to prevent.

**Live examples, verified at the base.**

```text
QueryRoot.imprints -> Imprint.publisher -> Publisher.distributionPlatforms
QueryRoot.contacts -> Contact.publisher -> Publisher.distributionPlatforms
Publisher.contacts -> Contact.publisher -> Publisher.distributionPlatforms
```

The list resolver already holds enough information to derive the terminal key:
`Imprint.publisher_id` is a `Uuid` on the resolved row
(`thoth-api/src/model/imprint/mod.rs:44`), as is `Contact.publisher_id`
(`thoth-api/src/model/contact/mod.rs:54`).

**Architecture selected.** `ADR-0006` section 4.19 extends A2 so one prefetch
site may target either a direct child or a descendant loader-backed field. A site
settles four concepts — selection path, terminal loader identity, terminal
normalized load-shape constructor, and a key projector from the resolved list
item to the terminal loader key. The ADR mandates the concepts and deliberately
does not mandate Rust type names, leaving the representation to implementation
evidence.

Traversal semantics (section 4.19.3) match `field_original_name()` at **every**
path segment, collect **every** matching terminal selection across every matching
intermediate branch, and extract the load shape from each terminal selection
rather than from an ancestor. Identical normalized terminal shapes de-duplicate;
different shapes remain separate dispatches. `BE-02`'s terminal shape is
unchanged at `Unit`, and no argument is added to any production field.

Feasibility was verified against the pinned sources rather than assumed:
`LookAheadSelection::children()` (`juniper` 0.16.2,
`src/executor/look_ahead.rs:606`) returns `LookAheadChildren`, whose `iter()`
(`:451`) yields child selections that themselves expose `children()`, so
recursive traversal composes to arbitrary depth on stable public API;
`field_original_name()` (`:528`) is public; and `select()` / `has_child()`
(`:426-441`) match `field_name()`, which returns the alias when present and
returns only the first match — confirming the alias hazard applies at every
level, not only at the terminal field.

**No second namespace.** Descendant results are stored under the ordinary
terminal identity `(loader, shape, terminal key)` of section 4.4. An entry
prefetched from an ancestor and one prefetched from the terminal field's own
parent list satisfy each other's lookups. A separate namespace would reintroduce
duplicate SQL for the same key and break section 4.6's multi-site reuse
guarantee.

**Key-projection security rule (section 4.19.4).** A descendant site may project
a terminal key only when all four hold: the relationship is deterministic from
data already on the resolved item; the projected key is one the GraphQL
relationship would itself expose; the intermediate resolver applies no additional
authorization decision the prefetch would bypass; and the loader does not
retrieve child-protected data without the authorization context that data
requires. `resolved authorized Imprint -> imprint.publisher_id` is admissible.
The rule explicitly does **not** generalize to arbitrary ID derivation from user
input or to skipping an intermediate check because its foreign key is known.
Where a path crosses a distinct authorization boundary, the adopting task must
establish equivalent authorization before prefetch or escalate, and authorization
logic must not be duplicated inside a generic loader.

**Distinction from legacy intermediate N+1 (section 4.19.5).** `Imprint.publisher`
calls `Publisher::from_id` once per imprint
(`thoth-api/src/graphql/model.rs:1366`), as does `Contact.publisher`
(`model.rs:3120`). A descendant prefetch at `QueryRoot.imprints` stops the
**terminal** loader-backed field from adding a further query per imprint; it does
not make the operation globally N+1-free. The ADR now defines two separate
evidence scopes — loader-backed-field compliance and legacy intermediate resolver
performance — requires them reported as distinct figures (section 8.2 item 9),
and prohibits any claim that a whole operation is free of N+1 access unless every
intermediate path was separately measured and remediated. `BE-02` must prove the
terminal bound on every material path and is **not** required to remediate
`Imprint.publisher` or `Contact.publisher`.

**Proof fixture changes.** `THOTH-GQL-BATCH-01` section 3.1.1 now requires both a
direct path and an indirect path in the test-only schema, proving recursive
look-ahead, alias-safe matching at every level, key projection, de-duplication,
terminal shape construction, one set-based terminal dispatch, correct results, no
terminal fallback on the covered path, reuse of already-loaded terminal entries
across repeated sites, and unchanged intermediate resolver behaviour. Test-only
types and wrappers must be used; no production resolver may be modified to prove
this. A stop condition returns
`BLOCKED - A2 CANNOT COVER INDIRECT FAN-OUT WITHOUT NEW EXECUTION ARCHITECTURE`
rather than pushing unresolved architecture into `BE-02`.

### 17.3 Finding 2 (P1) — mutation payloads conflicted with the coverage rule

**Exact live conflict.** `ADR-0006` simultaneously required that all material
fan-out paths be covered (section 4.18.2) and that prefetch sites be installed
only on resolvers unreachable from `MutationRoot` payload selections (former
section 4.12). Both cannot hold.

**`updatePublisher -> Publisher` evidence.** Thoth's mutations return rich model
objects, not thin acknowledgements: `updatePublisher -> Publisher`
(`thoth-api/src/graphql/mutation.rs:405-412`), `createPublisher -> Publisher`
(`:75`), `deletePublisher -> Publisher` (`:799`), `updateContact -> Contact`
(`:720`). `Publisher` exposes `contacts` (`model.rs:1258`) and `Contact` exposes
`publisher` (`model.rs:3120`), so a mutation payload forms a real publisher
fan-out with no query operation involved:

```graphql
mutation {
  updatePublisher(data: { ... }) {
    contacts { publisher { distributionPlatforms { platform } } }
  }
}
```

**Juniper execution/path investigation.** The directed candidate was
operation-scoped batching — query scope = whole request, mutation scope = current
top-level mutation field. It was verified against the pinned sources. Two of
three prerequisites fail.

*Path derivation — possible but off-label.* `Executor::field_path` is a private
field; `FieldPath::construct_path` and `FieldPath::location` are private methods;
and although `FieldPath` is reachable (`pub mod executor`, `src/lib.rs:33`;
`pub enum FieldPath`, `src/executor/mod.rs:61`) it exposes no public accessor for
its contents. The only public route to the current path is
`Executor::new_error(FieldError) -> ExecutionError` (`:679`) followed by
`ExecutionError::path()` (`:797`) — an error constructor used as a path accessor.
Section 3.1 already rejected a mechanism resting on non-contractual executor
behaviour, so accepting this would apply a weaker standard to a higher-risk
decision. Path segments are **response keys, i.e. aliases**:
`field_sub_executor` (`:568`, `#[doc(hidden)]`) stores `field_alias`, supplied by
both drivers as `response_name = f.alias.unwrap_or(f.name)`
(`src/types/base.rs:446`; `src/types/async_await.rs:216`). Alias-keyed scope is
conservative rather than unsafe, so this part is workable.

*Operation type — not derivable. This is the blocking finding.* A field
resolver's executor is always a sub-executor whose `current_type` is that field's
own type, never the root operation type (`src/executor/mod.rs:568-596`), and the
public surface carries no operation-type discriminant. `FieldPath::Root` carries
only a `SourcePosition` (`:61-64`). `SchemaType::query_type()` /
`mutation_type()` (`src/schema/model.rs:371,387`) describe the schema's shape,
not the operation in flight. `GraphQLRequest` exposes only `operation_name()`
(`src/http/mod.rs:55`), a caller-chosen label. The two remaining routes are both
excluded by the review's own constraints: parsing the raw GraphQL document is
prohibited, and having each mutation resolver mark its scope is the 88-resolver
retrofit (`mutation.rs:58-61`) the candidate existed to avoid.

*Serial top-level mutation execution — true on sync, false on async.* The pinned
Juniper honours serial mutation root fields only on the sync path
(`src/executor/mod.rs:883` -> `resolve_selection_set_into`, a plain `for` loop,
`src/types/base.rs:430-470`). On the async production path
(`src/executor/mod.rs:985`) it calls the same `resolve_into_value_async` used for
queries, which drives every field through `FuturesOrdered`
(`src/types/async_await.rs:196,262`) with **no `OperationType`-aware
serialization anywhere**. Thoth's mutation resolvers are all synchronous, so
`juniper_codegen`'s `future::ready(..)` wrapper makes each field future complete
on its first poll and they happen to run serially today — but that is inference
from polling behaviour, which section 3.1 rejected. A scope keyed on the
top-level response key would in fact be robust to interleaving, so this is not
itself the blocker; it does mean sync-path evidence would not establish
async-path behaviour, so the "sync and async agree" requirement could not be met
from the sync harness alone.

**Outcome — BLOCKED, escalated.** Because operation type is not derivable through
stable public API, the candidate is not implementable on the pinned stack. The
contradictory rule was **withdrawn** rather than left in place, and `ADR-0006`
section 4.12 now records the conflict, the investigation and a decision set for
the CTO — M1 (explicit query-only compliance boundary, recorded as a scoped
control exception requiring CTO acceptance) and M2 (expand the architecture;
on the evidence, the only workable shape is a store scoped by top-level response
key applied uniformly to queries and mutations, costing cross-top-level-field
reuse in queries and resting on the off-label path accessor).

**Why the standing control is not silently narrowed.** No exclusion such as
"mutation payloads are outside N+1 compliance" was written. Section 4.12.3 states
that the hold on mutation-reachable prefetch sites is temporary and pending the
decision, **not** a finding that such paths are inherently exempt; requires
adopting tasks to record mutation-payload paths as *blocked* rather than as
covered or excluded; and notes that such paths remain **correct** via the section
4.7 fallback while not being N+1 compliant. Invariant 10 was rewritten to state
the open decision rather than assert the withdrawn structural rule. Narrowing a
standing engineering control is a CTO decision, not an authoring one, and neither
M1 nor M2 was selected here.

**What remains settled.** Section 4.12.5 records that query operations perform no
write so cannot serve a stale read; that a top-level mutation resolver's write
completes before its payload selection resolves on both paths, so
read-after-write *within* one top-level mutation field is structurally sound
under either option; that whole-store invalidation remains the provided
primitive; and that the foundation must still prove within-field read-after-write
using test-only mutations and types, with no production mutation resolver
modified.

### 17.4 Finding 3 (P2) — stale PR #789 body

The PR body still described pre-remediation architecture — including that a
failed prefetch leaves affected keys "absent, never empty", the loader/key
batching identity, and read-after-write coherence as structurally confined to
sites unreachable from mutation payloads. All three now contradict the committed
ADR. The body was rewritten after this remediation was pushed, so that it
describes the new exact-head architecture. Updating a PR body is metadata and
does not change the Git head; no new PR was opened.

### 17.5 Files changed by this remediation

- `docs/engineering/decisions/ADR-0006-request-scoped-graphql-batching.md`
- `docs/engineering/ai-delivery/tasks/THOTH-GQL-BATCH-01.md`
- `docs/engineering/ai-delivery/implementation-reports/THOTH-GQL-BATCH-01-SPEC-implementation-report.md`
- `CHANGELOG.md`

`docs/engineering/decisions/decision-register.md` and
`docs/engineering/ai-delivery/README.md` were reviewed; neither required a change,
because `ADR-0006` keeps its number and `PROPOSED` status and
`THOTH-GQL-BATCH-01` keeps its `DRAFT` status, so both index entries remain
accurate.

### 17.6 Previous remediations preserved

Verified unchanged by this remediation:

- **load identity** — `(loader identity, normalized load shape, parent key)`,
  now explicitly shared by direct and descendant prefetch, with no additional
  namespace. Any operation-scope component would arrive only with an M2
  selection, which was not made;
- **load shapes** — typed, loader-owned, every result-changing argument
  represented, defaults normalized, one constructor shared by prefetch site and
  child lookup, no serialized argument strings. Section 4.19.3 extends this by
  requiring the shape to come from the terminal selection, not an ancestor;
- **store states** — `NotLoaded`, `Loaded(Vec<V>)`, `LoadFailed(error)`, with
  only `NotLoaded` falling back;
- **failure ownership** — the parent list does not fail because a prefetch
  failed; the child field surfaces `LoadFailed`; no retry; no empty substitution;
  the `errors[].path`, null-propagation and `extensions.type` contract is tested;
- **SQL measurement** — an actual SQL observer with a dedicated pool created
  after instrumentation, never the existing `OnceLock` pool. Extended, not
  weakened, by the separate-figure requirement for intermediate resolvers;
- **coverage** — the correctness fallback is still never N+1 compliance
  evidence;
- **`BE-02`** — `DistributionPlatformsLoadShape = Unit`, no API argument change.

### 17.7 Residual risks after this remediation

- **unresolved mutation-payload boundary** — the highest-consequence open item.
  Until the CTO selects M1 or M2, mutation-payload fan-out paths carry no
  prefetch site and are correct but not N+1 compliant. Recorded as an open
  decision, not an exclusion;
- **descendant key-projection authorization** — an indirect site projects a key
  across an intermediate field, so a projector could cross a boundary the direct
  traversal would have enforced. Mitigated by the four binding conditions of
  section 4.19.4 and the recorded-boundary obligation in section 4.18.2 step 6;
- **recursive look-ahead and alias matching** — traversal must match
  `field_original_name()` at every segment and collect every terminal selection
  across every branch. Failure silently defeats batching rather than breaking
  correctness; mitigated by query-count measurement on descendant paths;
- **legacy intermediate N+1 visibility** — a bounded terminal loader alongside an
  unbounded intermediate resolver could be misreported as whole-operation N+1
  freedom. Mitigated by the two evidence scopes and the separate-figure
  requirement;
- **load-shape normalization** — unchanged and still the one failure mode
  returning confidently wrong data rather than a miss;
- **look-ahead directive reporting** — `@skip` / `@include` are still not
  evaluated, now applying per path segment;
- **mutation scope identity** — not adopted. If M2 is later selected, the
  off-label `new_error(..).path()` accessor and the loss of cross-top-level-field
  query reuse both become live risks requiring their own review;
- **explicit opt-in and path coverage** — adoption remains two-place and
  per-path, so an uncovered path stays correct without being compliant.

### 17.8 Boundaries confirmed after this remediation

- no runtime source, schema, migration, `Cargo` manifest, lock file or workflow
  changed;
- PR #788, its branch and the `BE-02` specification unmodified;
- issue #765 unmodified;
- `ADR-0006` remains `PROPOSED`; `THOTH-GQL-BATCH-01` remains `DRAFT`;
- the implementation branch `feature/shared-architecture/graphql-batching` was
  not created;
- `BE-02` runtime implementation remains unauthorized, and the `BE-02` path
  inventory was not performed here beyond the live examples used to prove the
  architecture;
- no approval-state or transient-status prose was written into any committed
  file.

---

## 18. Third remediation: CTO selection of top-level-response-key scoping

### 18.1 Position at the start of this remediation

```text
Previous independent decision: BLOCKED
Previous exact reviewed head:  0de5d0ef6d04e87e4204dc88a57d14edb313ac1d
Blocking decision:             M1 vs M2
CTO direction:                 M2 - uniform top-level-response-key scoping
```

The previous exact head was `BLOCKED` because `ADR-0006` could not settle the
mutation-payload N+1 boundary on authoring authority: M1 would have waived part
of a standing engineering control, and M2 would have enlarged the approved
architecture. The CTO subsequently selected **M2**, which this remediation
encodes.

The direction authorizes architecture and specification remediation only. It is
**not** approval of the resulting exact ADR content, and does not authorize
runtime implementation, the implementation branch, `BE-02`, merge or any
production behaviour. Per `ADR-0005` and `docs/engineering/AGENTS.md` section
1.1, review identifiers and merge evidence are terminal GitHub evidence and are
not copied into this file.

### 18.2 How the open decision was resolved

`ADR-0006` section 4.12 was rewritten from an open escalation into a binding
architecture. The withdrawn rule, the "OPEN"/"BLOCKED" framing, the temporary
hold on mutation-reachable prefetch sites and the M1/M2 decision set are all
**gone** from the binding sections — not merely annotated with the outcome. The
document no longer contains a live architecture blocker for mutation-payload
coverage. `ADR-0006` nevertheless remains `PROPOSED`, because exact-head review
and explicit CTO approval are still outstanding.

Consequential reconciliation across the ADR: sections 4.2, 4.4, 4.4.4, 4.4.5,
4.6, 4.7, 4.9.1, 4.9.2, 4.11, 4.13, 4.15, 4.18.2, 4.18.3, 4.19.2, section 5
invariants, section 6, section 8.2, section 11, section 12, section 13 and
section 14 were each updated where the scope dimension changed their meaning,
rather than by mechanical find-and-replace.

### 18.3 Exact new store identity

```text
(top-level response key, loader identity, normalized load shape, parent key)
```

with dispatch-level failure state tied to:

```text
(top-level response key, loader identity, normalized load shape, attempted key set)
```

Consequences now binding: a key loaded in scope `A` is `NotLoaded` in scope `B`;
duplicate keys within one scope still de-duplicate; identical terminal aliases
within one scope still reuse; direct and descendant prefetch within one scope
share the same entry with no second namespace; multiple prefetch sites within one
scope issue no duplicate SQL; and the same loader/key beneath two top-level
response keys dispatches twice, once per scope. A `LoadFailed` under one scope
does not poison another.

The **load-shape dimension was not removed**, and the three-state store, failure
ownership, descendant contract and actual-SQL measurement requirements are all
carried forward unchanged (section 18.9).

### 18.4 Why no operation-type detection is needed

The rule is uniform, so nothing needs to know whether it is executing in a query
or a mutation. That is also the only implementable option on the pinned stack:
a field resolver's executor is always a sub-executor whose `current_type` is the
field's own type (`src/executor/mod.rs:568-596`); `FieldPath::Root` carries only
a `SourcePosition` (`:61-64`); `SchemaType::query_type()`/`mutation_type()`
(`src/schema/model.rs:371,387`) describe schema shape rather than the operation
in flight; and `GraphQLRequest` exposes only a caller-chosen `operation_name`
(`src/http/mod.rs:55`). The ADR now prohibits attempting such detection, and
prohibits parsing the raw GraphQL document to derive scope.

### 18.5 Why mutation serialization is no longer an architecture dependency

Scope isolation is structural. Entries created beneath one top-level response key
are unreachable from another, so a write performed by a second top-level mutation
field cannot be followed by a stale read of the first field's loader state
**regardless of execution order**.

This matters concretely, because the pinned Juniper does not serialize mutation
root fields on the async production path: `execute_validated_query_async`
(`src/executor/mod.rs:985`) routes mutations into the same
`resolve_into_value_async` used for queries, which drives every field through
`FuturesOrdered` (`src/types/async_await.rs:196,262`) with no `OperationType`-aware
serialization, whereas the sync path uses a plain `for` loop
(`src/executor/mod.rs:883`; `src/types/base.rs:430-470`). An architecture relying
on serialization would have been relying on executor polling behaviour, which
section 3.1 rejected when it eliminated variant A1. M2 does not.

Read-after-write *within* one top-level mutation field remains structurally sound
on both paths: the resolver returns `FieldResult<Publisher>`, and the value must
exist before its sub-selection resolves.

### 18.6 Accepted query reuse tradeoff

Recorded explicitly in `ADR-0006` section 4.12.13 and **not** presented as
zero-cost. The same `(loader, shape, key)` reached beneath two top-level query
response keys is loaded once per scope, so an operation may issue a bounded
number of extra set-based dispatches. The bound comes from the operation's
top-level structure, is independent of parent list size, and therefore does not
recreate `1 + N`: two top-level scopes over a 100-parent list issue **2**
set-based statements, not 200. Request-wide reuse across top-level fields is no
longer stated as an invariant anywhere.

The foundation must prove this explicitly rather than hide it: a
two-top-level-field query test must report 2 dispatches, not `N + N`.

### 18.7 Compatibility shim design

The pinned Juniper exposes no dedicated public path accessor —
`Executor::field_path` is a private field and `FieldPath::construct_path` /
`FieldPath::location` are private methods, so `FieldPath` is reachable but its
contents are not. The accepted mechanism is `Executor::new_error(..)`
(`src/executor/mod.rs:679`) followed by `ExecutionError::path()` (`:797`), taking
the first segment.

It is confined to **one** isolated helper, materially
`top_level_response_key(executor) -> Result<ScopeKey>`, which must not call
`push_error`, must not modify the response, must return the first response-key
segment, must fail closed, must not parse the query string, must not inspect
private fields, must not use `unsafe`, and must be the only site in the codebase
using the technique. No package dependency is added for it.

**Side-effect freedom is evidenced, not assumed.** `new_error` builds an
`ExecutionError` from `field_path.construct_path(..)` and returns it
(`:679-689`); it never touches the executor's shared error collection, which is
what `push_error_at` does via `self.errors.write()` (`:665-677`). The constructed
error is discarded after its path is read.

**Fail-closed behaviour is specified exactly** (section 4.12.9): a prefetch site
that cannot derive its scope performs no prefetch and does not fail the parent
list field; a terminal resolver that cannot derive its scope treats the lookup as
`NotLoaded`. Both degrade to the correctness fallback. Substituting a shared or
request-global namespace is prohibited, because that is precisely what would let
entries cross scopes.

**Scope keys are response keys, therefore aliases**, and must not be normalized
to the schema field name — `a: publishers` and `b: publishers` are two scopes.
The ADR makes explicit that this is a different rule from selection-path
matching, which continues to use `field_original_name()` because it identifies
schema fields.

**Field merging was investigated rather than assumed** (section 4.12.6). Juniper's
executor does not merge selections: `resolve_selection_set_into` iterates each
`Selection::Field` and calls `Object::add_field`, which replaces an existing value
for the key (`src/value/object.rs:28-37`). The `OverlappingFieldsCanBeMerged`
validation rule (`src/validation/rules/overlapping_fields_can_be_merged.rs`,
registered at `rules/mod.rs:78`) rejects, before execution, any document where two
selections sharing a response key differ in field name (`:398-411`), arguments
(`:415-424`) or type (`:428-440`). Two occurrences of one response key are
therefore validated-compatible and contribute to the same response field, so
sharing one scope is correct and lets the second occurrence reuse the first's
entries. **No source-position or AST-occurrence component was added**, because the
evidence does not require one and adding one would fragment a single response
field and cause avoidable duplicate SQL.

**Upgrade policy** (section 4.12.14): any Juniper version change affecting
`Executor`, `ExecutionError`, field-path construction, alias/response-key handling
or `new_error()` requires revalidation of the shim before deployment. This is a
revalidation obligation, not a prohibition on upgrading, and it is discharged
through the repository's existing dependency-change review and release gates
rather than through a new process.

### 18.8 Risk reclassification

The classification was re-run against `risk-classification.md` rather than
carried forward. Result: **`HIGH`, unchanged**, with a strengthened rationale
recorded in `THOTH-GQL-BATCH-01` section 1.1.

It matches HIGH on "changes to canonical data semantics" and "changes capable of
broadening processing scope", and the escalation rules apply because production
query volume is unknown and the mechanism targets callers that do not yet exist.
Scope extraction is now load-bearing for every lookup, and scope collision would
be a response-correctness fault rather than a performance one.

It meets **no** `Critical` criterion: no destructive or irreversible production
migration, no canonical data rewrite at scale, no mass redistribution or external
publication, no security boundary affecting all publishers (authorization is
untouched; scoping is isolation, not authorization), no secrets or
identity-provider work, no metrics recomputation, no source-of-truth cutover and
no material legal, privacy or contractual consequence. Bounding factors: no
migration, no production consumer at merge, no new dependency, no public API
change, and a fail-closed path that degrades to unbatched-but-correct.

The classification was neither raised merely because M2 adds partitioning nor
kept merely because it was previously HIGH.

### 18.9 Previously accepted architecture preserved

Verified unchanged by this remediation:

- **A2 mechanism** — look-ahead-driven set-based prefetch into request-scoped
  GraphQL state. No external DataLoader dependency, no execution-model migration;
- **typed load identity** — typed, loader-owned normalized load shapes, every
  result-changing argument represented, defaults normalized, one constructor
  shared by prefetch site and child lookup, no serialized argument strings. The
  load-shape dimension is retained and extended by the scope dimension, not
  replaced;
- **store state machine** — `NotLoaded`, `Loaded(Vec<V>)`, `LoadFailed(error)`,
  with only `NotLoaded` executing the direct fallback;
- **failure semantics** — failed prefetch does not fail the parent list field;
  `LoadFailed` is consumed by the terminal child resolver; no per-parent retry;
  no successful-empty substitution; `errors[].path`, null propagation and
  `extensions.type` tested;
- **descendant prefetch** — the four-concept contract (selection path, terminal
  loader identity, terminal normalized load-shape constructor, key projector),
  recursive `children()` traversal, `field_original_name()` matching at every
  segment, all matching terminal aliases collected, no second namespace for
  indirect prefetch, the key-projection authorization conditions, and terminal
  compliance kept separate from legacy intermediate N+1 performance;
- **SQL evidence** — actual-SQL observation through a pool created after
  instrumentation, or an equivalent real-SQL observer. Application loader
  counters remain unacceptable as primary proof;
- **authorization** — projected keys only from already-resolved parent rows, no
  arbitrary IDs from user input, no bypass of intermediate policy checks,
  child-protected data requiring its own context, no policy change. The ADR now
  states explicitly that scoping is isolation and must never substitute for a
  permission check.

### 18.10 New tests and acceptance evidence required

Added to `THOTH-GQL-BATCH-01` section 9 and section 10: scope-key store identity;
identical scope derivation at prefetch site and terminal resolver; no entry
visible across scopes; two top-level aliases of one schema field producing
separate namespaces; repeated response keys sharing one scope with no
source-position component; shim path extraction across unaliased and aliased
top-level fields, direct children, deep descendants, inline fragments and named
fragments; shim side-effect freedom (no error, no `errors[]` change, no result
change, no SQL); fail-closed derivation with no global-namespace substitution;
single-site restriction and documented pinned coupling; the full store collision
matrix including `LoadFailed` non-poisoning and invalidation across scopes; the
two-top-level-field query test proving 2 dispatches rather than `N + N`;
mutation read-after-write within one top-level field; isolation across two
top-level mutation fields proven by scope isolation rather than execution order,
including under async interleaving with a yielding fixture where practical; and
per-scope SQL-count reporting inside both query and mutation fan-outs, under both
`execute_sync` and async `execute`.

New stop conditions cover inability to derive the scope through the shim,
inability to make the shim side-effect-free, inability to demonstrate
cross-top-level mutation isolation under async execution, and any requirement to
modify production mutation resolvers, detect operation type at nested resolvers,
or use `new_error(..)` outside the shim module.

### 18.11 Boundaries confirmed

- **no runtime work was performed.** No runtime source, schema, migration,
  `Cargo` manifest, lock file or workflow changed;
- PR #788, its branch and the `BE-02` specification unmodified;
- issue #765 unmodified;
- `docs/publisher-services/task-status.md` unmodified;
- `ADR-0006` remains `PROPOSED`; `THOTH-GQL-BATCH-01` remains `DRAFT` and `HIGH`;
- the implementation branch `feature/shared-architecture/graphql-batching` was
  not created;
- no additional ADR was created — M2 was fully specifiable inside `ADR-0006`;
- `BE-02` runtime implementation remains unauthorized, and its exact-base
  inventory — now covering query **and** mutation paths — remains its own future
  responsibility;
- no approval-state or transient-status prose was written into any committed
  file.

---

## 19. Fourth remediation: mutation execution scope

### 19.1 Position at the start of this remediation

```text
Previous independent decision:
BLOCKED

Previous reviewed exact head:
7991d26fe64b8a4a1770cb1062a98a64fb07ba20

Blocking finding:
TOP-LEVEL RESPONSE KEY DOES NOT UNIQUELY IDENTIFY
A MUTATION EXECUTION ON PINNED JUNIPER
```

A second, `P1` specification-consistency finding accompanied it:
`THOTH-GQL-BATCH-01` still carried pre-scoping binding identities such as
`(loader, shape)` and `(loader, shape, key)` in places where the architecture
requires the execution-scope dimension.

Verified live before editing: PR #789 open and draft at exactly
`7991d26fe64b8a4a1770cb1062a98a64fb07ba20` with no intervening commit; `develop`
at `5a8c27b1b7c11a4f6bd26d459556468099f8c1f4`; `ADR-0006` `PROPOSED`;
`THOTH-GQL-BATCH-01` `DRAFT`; PR #788 at `d411d4935a507804f28d8798419d405e32880d02`;
issue #765 last updated 2026-07-27; and no implementation branch in existence.

### 19.2 Pinned-source reproduction

The finding was reproduced against the exact pinned `juniper` 0.16.2 sources in
an isolated throwaway probe **outside** this repository. No repository code was
built, added or modified for it.

Confirmed:

1. **validation** — `OverlappingFieldsCanBeMerged` rejects only *incompatible*
   repeats: different field names, differing arguments or conflicting types
   (`find_conflict`, `overlapping_fields_can_be_merged.rs:378-460`). Compatible
   repeats sharing one response key pass. `find_conflict` never inspects
   directives, so directive differences cannot make two occurrences conflict;
2. **sync execution** — `resolve_selection_set_into` (`types/base.rs:436-500`)
   iterates every `Selection::Field` occurrence and calls `resolve_field` once
   per occurrence;
3. **async execution** — `resolve_selection_set_into_async`
   (`types/async_await.rs:209-283`) pushes one future per occurrence into a
   `FuturesOrdered`;
4. **merging** — per-occurrence results are reconciled under the shared response
   key by `merge_key_into` (`types/base.rs:627-651`), which deep-merges objects
   and lists. The previous ADR text cited `Object::add_field` "replacing" the
   value; that was the wrong function, and it supported the wrong conclusion —
   the executor **does** merge repeated selections' results while **not** merging
   their execution;
5. **observed behaviour** — for a mutation containing one response key twice with
   compatible arguments, validation passed with no errors, the response carried a
   single merged object, and the mutation resolver executed **twice**, identically
   under `execute_sync` and async `execute`, and identically when the duplicate
   arrived through a named fragment spread or an inline fragment;
6. **ordering** — under async execution the *second* source occurrence's resolver
   was observed completing first, confirming the pinned async path drives
   top-level mutation fields concurrently rather than serially. That is a
   pre-existing GraphQL specification deviation in the dependency; it is recorded,
   not repaired here.

The withdrawn claim was therefore

```text
one top-level response key == one top-level mutation execution
```

which is false on the pinned stack.

Also established, for the guard's feasibility: `ExecutionError::path()` exposes
response-key names only; source positions do **not** survive into any public
nested-executor API (`Executor::field_path` private, `FieldPath::construct_path`
and `FieldPath::location` private methods, `OwnedExecutor` no richer); the
parser and executor **do** expose stable public operation and selection
information at the request boundary (`pub mod parser`, `pub mod executor`,
`parse_document_source`, `get_operation`, `Operation::operation_type`, the
re-exported `Selection`/`Definition` AST); Juniper exposes **no** supported
custom validation-rule extension point (`visit_all_rules` is a fixed list),
though `validation::{visit, ValidatorContext, MultiVisitorNil, Visitor}` are
public and a rule-shaped visitor could be run separately; and Thoth **can**
invoke parse/validate/execute as separate public stages
(`execute_validated_query`, `execute_validated_query_async`) rather than only
`GraphQLRequest::execute`. Production wiring is
`thoth-api-server/src/lib.rs:94-106` (async `GraphQLRequest::execute`), and test
wiring is both `juniper::execute_sync` (`thoth-api/src/graphql/tests.rs`) and the
async `GraphQLRequest::execute` harness (`thoth-api/tests/support/mod.rs:108`).

### 19.3 F1 — execution occurrence identity: rejected

Investigated whether a nested resolver can derive a stable identifier for the
actual top-level resolver invocation rather than merely its response key.

APIs investigated: `Executor::location()`, `Executor::new_error(..)` /
`ExecutionError::path()` and `::location()`, `Executor::look_ahead()`,
`FieldPath` and its variants, `OwnedExecutor`, and `Executor::field_sub_executor`.

Findings:

- `field_path` is a private field, and although `FieldPath::Field` carries the
  ancestor `SourcePosition`, `construct_path` and `location` are private methods
  with no public accessor returning the chain. A descendant cannot recover an
  ancestor's position;
- `Executor::location()` returns only the currently executing field's own
  position;
- `ExecutionError::path()` returns response-key names only, with no positional
  component;
- `look_ahead()` finds the current field by response name with `find_map`, so it
  resolves to the **first** matching occurrence and cannot distinguish duplicates
  either.

Decisive counterexample, reproduced: for

```graphql
mutation {
  x: updateA(id: 1) { ...P }
  x: updateA(id: 1) { ...P }
}
fragment P on Payload { id child }
```

two distinct mutation resolver executions occurred, and the terminal `child`
resolver under each observed an **identical** `path()` of `["x", "child"]` **and**
an **identical** `location()`. Every publicly derivable identity signal collapses.
Separately, F1 also fails the section 4.19 requirement that the prefetch site and
the terminal descendant derive the *same* value, since their positions differ by
construction.

**F1 rejected** — not on cost, on impossibility with public API.

### 19.4 F2 — central duplicate-mutation-response-key guard: accepted

Central pre-execution detection is possible, and was reproduced end to end using
only public, non-`unsafe` API: `GraphQLRequest`'s public `query`,
`operation_name` and `variables` fields; `parse_document_source`;
`RootNode::schema`; `get_operation`; `Operation::operation_type`; the public
`Selection`/`Definition` AST; `InputValue::into_const`; `RuleError::new`;
`GraphQLError::ValidationError`; and `GraphQLResponse::from_result`.

Handling confirmed by reproduction:

- **fragments** — named spreads and inline fragments expanded before counting;
- **directives** — `@skip`/`@include` evaluated against coerced variables, so a
  definitely-excluded duplicate is correctly **accepted**; an undecidable
  condition rejects conservatively, recorded as a tradeoff;
- **operation type** — read directly at the boundary, so queries are untouched;
- **operation selection** — `operationName` honoured through `get_operation`.

Two pinned-API constraints were found by compiling against the crate:
`juniper::ast` is private, so `Fragment`, `Field`, `Directive` and
`ast::Arguments` are not publicly nameable — fragment expansion must hold
`&[Selection]` and directive evaluation must keep those types inferred; and
`types::base::is_excluded` is `pub(super)`, so directive evaluation must be
reimplemented on public API and kept behaviourally identical.

Reproduced results, with a resolver-call counter:

| Document | Outcome | `is_ok()` | Mutation resolver calls |
|---|---|---|---|
| distinct aliases | accepted | true | 2 |
| direct duplicate response key | rejected | false | **0** |
| duplicate via named fragment | rejected | false | **0** |
| duplicate via inline fragment | rejected | false | **0** |
| duplicate with literal `@skip(if: true)` | accepted | true | 1 |
| duplicate with `@skip(if: $s)`, `$s = true` | accepted | true | 1 |
| duplicate with `@skip(if: $s)`, `$s = false` | rejected | false | **0** |
| duplicate response key in a **query** | accepted | true | 0 |
| baseline: real validation error (unknown field) | rejected | false | 0 |

The rejection's serialized body was byte-comparable in shape to the baseline
juniper validation failure — an `errors` array of `{message, locations}` with no
`data` key — so the existing handler branch yields **HTTP 400** with no new
branch and no one-off protocol. Resolver count on rejection is **guaranteed
zero** because rejection precedes `execute`/`execute_sync` entirely.

Notably, the guard does **not** need to replace `GraphQLRequest::execute`.
Explicit parse/validate/execute orchestration is available and was confirmed
public, but was **not** adopted: it would reimplement juniper's request pipeline
for no additional guarantee. The accepted cost is one extra document parse per
mutation request, recorded rather than presented as free.

**F2 accepted**, and recorded as a **shared GraphQL execution prerequisite**
rather than a batching helper, because duplicate top-level mutation response keys
duplicate a **write** whether or not any loader-backed field is selected.

### 19.5 F3 — execution-layer correction: rejected as architecture expansion

Feasibility: juniper's field-collection point,
`resolve_selection_set_into{,_async}`, is `pub(crate)` inside the private
`mod types`, so it cannot be replaced or wrapped externally. The only external
interception is hand-writing `GraphQLValue`/`GraphQLValueAsync` for the mutation
root, overriding `resolve`/`resolve_async` and delegating `resolve_field{,_async}`
and `meta`.

Scope and burden: that places Thoth in the business of maintaining GraphQL field
collection — fragments, inline fragments, directives, sub-selection merging,
error paths, null propagation — against a dependency whose own implementation is
private; it must hold for both sync and async; correctness would additionally
require fixing serial mutation execution, changing behaviour for every existing
multi-field mutation request; and it couples Thoth to pinned execution internals
far more tightly than the section 4.12.8 shim, making any juniper upgrade a
re-derivation rather than a revalidation.

**F3 rejected** — materially larger than `THOTH-GQL-BATCH-01`, effectively
maintaining a partial custom GraphQL executor. Recorded as architecture
expansion requiring its own decision, not as a batching detail.

### 19.6 Selected architecture and final identity

**Outcome B.** Response-key scoping survives, but **only** because the request
boundary now guarantees uniqueness for mutation top-level response keys. The
architecture is two coordinated controls:

```text
1. central mutation request guard:
   each executable top-level mutation response key occurs at most once

2. loader store:
   scoped by top-level response key
```

Final store and execution identity, unchanged in shape but now sound:

```text
(top-level response key, loader identity, normalized load shape, parent key)
```

Why it uniquely corresponds to the write execution boundary: for an **accepted**
mutation operation the guard makes each executable top-level response key
correspond to exactly one top-level mutation resolver execution, so a scope never
spans two writes; a top-level mutation resolver's write completes before its
payload selection resolves, so read-after-write within a scope is sound; and no
entry crosses scopes, so a write under one top-level field can never be followed
by a stale read of another's state — regardless of ordering or interleaving. The
one-to-one invariant explicitly **depends on** the guard, which is why the store
must be unavailable without it.

Terminology corrected accordingly:

```text
storage lifetime:      one GraphQL request
reuse/execution scope: one unique executable top-level response key
```

The architecture may still be called request-scoped, because the container lives
on one request; it must not be described as giving request-wide reuse.

### 19.7 GraphQL compatibility implications

The public **schema** is unchanged and the generated SDL stays byte-identical.
The set of **accepted requests** changes: a mutation with a duplicate executable
top-level response key is now rejected. This rejects some documents the GraphQL
specification considers merge-compatible, and is recorded in `ADR-0006` section
4.12.6.4 as a deliberate server safety restriction compensating for pinned
Juniper's repeated mutation execution — explicitly **not** as ordinary
spec-conformant validation.

The restriction is deliberately narrow: queries, non-top-level duplicates,
distinct aliases and directive-excluded duplicates are all unaffected.

### 19.8 P1 specification sweep

Corrected in `THOTH-GQL-BATCH-01`, by targeted search across the whole file
rather than first occurrences:

| Was | Now |
|---|---|
| "repeated occurrences of one response key share one scope" as a general rule | split by operation type: true for queries, impossible for accepted mutations because the guard rejects them |
| failure recorded once per `(loader, shape)` dispatch | once per `(scope, loader, shape)` dispatch, with failure identity required to match load identity exactly |
| second prefetch covering an already-loaded `(loader, shape, key)` | `(scope, loader, shape, key)`, **within that scope** |
| "one statement per `(loader, shape)` dispatch" (acceptance criterion) | `(scope, loader, shape)` |
| "Store identity is `(loader identity, normalized load shape, parent key)`" (acceptance criterion) | `(top-level response key, loader identity, normalized load shape, parent key)` |
| repeated prefetch for an already-present `(loader, shape, key)` entry (concurrency section) | `(scope, loader, shape, key)` under the same execution scope, with cross-scope dispatch stated as correct |
| "two prefetch sites covering the same `(loader, shape)` in one request issue no duplicate SQL" (required test) | split into **same-scope reuse** and **cross-scope isolation** as two distinct required tests |
| "several prefetch sites can cover one `(loader, shape)` in a single request" (`BE-02` inheritance) | one `(scope, loader, shape)` under one execution scope, plus cross-scope isolation |

The internal `M1`/`M2` labels were also removed from `ADR-0006` in favour of
descriptive text, since the architecture is now two controls rather than one
model. No binding current-specification occurrence of an unscoped identity
remains in either document; the sole remaining `(loader, shape, key)` in the task
is the deliberate contrast describing what is **not** shared across scopes.

Historical text in this report describing prior rejected designs retains its
original wording, as required.

### 19.9 Risk reclassification

Re-run against `risk-classification.md` from scratch. Result: **`HIGH`**,
unchanged as a label but re-derived on materially different grounds.

Newly engaged criteria: **production feature activation** — the guard is live on
the production request path at merge, not dark-launched; and **cross-repository
API contract change** — the set of accepted GraphQL mutation requests changes for
all clients, including clients outside this repository. Previously engaged
criteria (canonical data semantics, capable of broadening processing scope) still
apply.

Not raised to `Critical`: checked criterion by criterion, there is no destructive
migration, no canonical data rewrite, no mass redistribution, no security
boundary affecting all publishers (the guard only rejects, so it cannot broaden
access), no secrets or identity-provider work, no metrics recomputation, no
source-of-truth cutover and no material legal or contractual consequence. The
"rollback is uncertain" escalation rule does not bite: the kill switch restores
prior behaviour without a deploy.

Not lowered to `Medium`, which would require no automatic production effect.

One required control changes as a result: `risk-classification.md`'s "feature
flag, comparison mode or controlled pilot where possible" is now genuinely
engaged, and is discharged by the kill switch.

### 19.10 Rollout and rollback implications

The selected fix **does** change the common GraphQL request path before any
loader adoption. Consequently:

- the claim "no production effect because no field adopts batching" is **false**
  for this task and is now explicitly prohibited in `ADR-0006` section 7.2 and
  `THOTH-GQL-BATCH-01` section 11;
- activation state at merge: store inert, **guard active**;
- the guard is deliberately **not** initially inactive. Shipping it dark would
  either leave the store unsafe or defer the same behaviour change to a less
  visible merge. It is instead protected by a kill switch defaulting to enabled,
  built on the repository's established `clap` `Arg::env(..)` configuration
  pattern — no new mechanism is invented;
- disabling the switch also makes the **store** unavailable, because a nested
  resolver cannot distinguish a mutation from a query. Batching therefore never
  operates without its prerequisite;
- monitoring: one warning-level log record per rejection, carrying no document,
  variables or argument values. No new alert or dashboard;
- rollback: disable the switch (no deploy), or revert the merge commit;
- before `BE-02` may depend on this: `ADR-0006` approved and merged;
  `THOTH-GQL-BATCH-01` implemented, independently reviewed and merged with the
  guard demonstrably active and its zero-execution evidence recorded; the `BE-02`
  specification amended to adopt the final mechanism and to record the inherited
  request-acceptance change; that amendment freshly reviewed and approved; and a
  separate `BE-02` implementation authorization.

### 19.11 A second architectural decision is flagged, not assumed

The central mutation request guard was **not** part of the direction the CTO
recorded when selecting top-level-response-key scoping. It changes the set of
accepted GraphQL requests for every mutation and every API client, and it is a
shared GraphQL execution control rather than a batching component.

It is recorded inside `ADR-0006` rather than split into a separate ADR because it
is inseparable from the mutation isolation guarantee — without it the store
cannot be used on any mutation path — but `ADR-0006` section 14 now asks the CTO
to approve it **as its own decision**, on its own merits, and states that if the
request-boundary restriction is declined then section 4.12 does not survive in
its present form and the mutation isolation objective returns to open, F1 being
rejected on evidence and F3 as architecture expansion.

Separately, the pinned async executor's concurrent execution of top-level
mutation fields is recorded as a **pre-existing** specification deviation in the
dependency. It is not repaired here, and repairing it would require its own
architecture decision.

### 19.12 Boundaries confirmed after this remediation

- **no runtime work was performed.** No runtime source, schema, migration,
  `Cargo` manifest, lock file or workflow changed. The F1/F2/F3 reproductions ran
  in a throwaway crate outside the repository and left nothing behind in it;
- **no authorization was granted.** `ADR-0006` remains `PROPOSED`;
  `THOTH-GQL-BATCH-01` remains `DRAFT` and `HIGH` with implementation
  `NOT AUTHORIZED`;
- PR #788, its branch and the `BE-02` specification unmodified;
- issue #765 unmodified;
- the implementation branch `feature/shared-architecture/graphql-batching` was
  not created;
- no additional ADR was created; the guard is specified inside `ADR-0006` and
  explicitly flagged for its own CTO decision;
- authorization remains unchanged by this remediation: keys still come only from
  already-resolved rows, no arbitrary user-input IDs are accepted, intermediate
  authorization is not bypassed, child-protected data still requires child
  authorization context, and scope identity remains isolation metadata rather
  than permission. The request-boundary guard makes **no** authorization
  decision;
- no approval-state or transient-status prose was written into any committed
  file.

---

## 20. Fifth remediation: activation controls, effective variables, identity sweep

### 20.1 Position at the start of this remediation

```text
Previous independent decision:
CHANGES REQUIRED

Previous exact reviewed head:
ef3a895a8acd5f372eb4440c7350cf7f09d5c527
```

Three findings:

```text
1. default-on guard violated safe post-merge / production activation controls
2. directive evaluation omitted operation variable defaults
3. one binding descendant identity remained unscoped
```

Verified live before editing: PR #789 open and draft at exactly
`ef3a895a8acd5f372eb4440c7350cf7f09d5c527` with no intervening commit; `develop`
at `5a8c27b1b7c11a4f6bd26d459556468099f8c1f4`; `ADR-0006` `PROPOSED`;
`THOTH-GQL-BATCH-01` `DRAFT` and `HIGH`; PR #788 at
`d411d4935a507804f28d8798419d405e32880d02`, `updatedAt` 2026-08-07T17:35:39Z;
issue #765 `updatedAt` 2026-07-27T15:50:33Z; no implementation branch; and the
branch diff still documentation-only.

**F1, F2 and F3 were not reopened.** No fresh evidence disturbed the F2
selection, and this remediation is bounded to the three findings above.

### 20.2 Finding 1 — activation controls

The previous revision made the guard active on every mutation request from the
merge commit, behind a kill switch defaulting to enabled, and treated CTO merge
authorization as authorizing that activation. That conflicts with
`release-gates.md`: safe post-merge changes should prefer disabled-by-default
behaviour; a merge that itself changes production behaviour must satisfy the
production-ready gate first; and production activation of HIGH-risk work
requires preview acceptance, controlled activation, monitoring, rollback, an
activation owner, an observation period and explicit CTO approval. A change to
accepted GraphQL mutation requests affecting every API client must not take
effect because an implementation pull request merged.

**Remediation.** The guard now has three modes (`ADR-0006` section 4.12.6.6):

| Mode | Evaluates | Rejects | Event | Loader store | Request acceptance |
|---|---|---|---|---|---|
| `OFF` (default, merged state) | no | no | none | unavailable | unchanged |
| `OBSERVE` | yes, as `ENFORCE` would | **no** | one per would-be rejection | unavailable | unchanged |
| `ENFORCE` | yes | yes | one per rejection | may be available | duplicates rejected |

and the binding rule is now

```text
repository merge  !=  production ENFORCE activation
```

with the merged state being `guard OFF, store unavailable, request acceptance
unchanged`. `OFF -> OBSERVE` and `OBSERVE -> ENFORCE` are separate transitions
with separate evidence, and `ENFORCE` requires explicit CTO production
activation approval distinct from merge authorization.

The fail-closed coupling was strengthened from "guard applied" to enforcement:

```text
loader store available  =>  guard mode == ENFORCE
```

with `guard OFF + store enabled` and `guard OBSERVE + store enabled` required to
be **unrepresentable**, encoded structurally — store availability derived from
the single mode value — rather than left to operator discipline.

**The "a comparison period adds no evidence" rationale is withdrawn.** It
answered the wrong question. The guard's decision function is deterministic and
test-covered; the open question is whether real production traffic contains
documents `ENFORCE` would reject, which no unit test can answer and which
matters precisely because the repository cannot enumerate its external clients.
`OBSERVE` is therefore recorded as the controlled compatibility pilot required
by the HIGH-risk release control, and it is mandatory before `ENFORCE`. A
non-zero would-be-rejection count **blocks** `ENFORCE` until the affected
callers are identified and addressed.

Preview/staging acceptance of the exact implementation candidate is required
before activation; the activation owner is recorded in repository terms (CTO
approves `ENFORCE`; the task's named engineering owner executes the change); and
rollback from `ENFORCE` is `ENFORCE -> OBSERVE` or `ENFORCE -> OFF` by
configuration without a deploy, with code revert as the secondary path.

### 20.3 Finding 2 — effective variables

The previous revision specified directive evaluation as
`InputValue::into_const(request_variables)` — the raw request variables. The
pinned executor does not use those.

**Verified against the pinned sources.** `execute_validated_query`
(`src/executor/mod.rs:828-855`) and `execute_validated_query_async`
(`:926-953`) both build, byte-equivalently:

```text
default_variable_values = { name -> default for each operation variable
                            definition declaring a default }
all_vars = request_variables.clone()
for (name, value) in default_variable_values:
    all_vars.entry(name).or_insert(value)     # request value always wins
final_vars = all_vars
```

`final_vars` is what the `Executor` carries and therefore what `is_excluded(..)`
evaluates `@skip`/`@include` against.

**Remediation.** The guard must now construct

```text
effective_variables =
    operation_defaults
    overridden_by
    request_variables
```

by starting from `GraphQLRequest::variables()`, reading the **selected**
operation's `variable_definitions`, and inserting each declared default only
where the request supplied nothing — mirroring `or_insert` exactly, with no
additional coercion. Raw request variables are prohibited.

**Measured impact.** Reproduced in a throwaway probe outside the repository,
comparing the guard's verdict against Juniper's **actual** mutation resolver
execution count across thirteen documents. With raw request variables the guard
**over-rejected six of thirteen** — every omitted-but-defaulted case, rejecting
requests the executor runs with a single occurrence. With the effective map, the
guard matched actual execution in **all thirteen**, including through named
fragments, fragment-spread directives and inline-fragment directives.

**A pinned-stack constraint found while proving this.** Juniper's
`default_values_of_correct_type` rule
(`src/validation/rules/default_values_of_correct_type.rs:31-46`) rejects a
**non-null** variable declaring a default:

```text
Argument "skip" has type "Boolean!" and is not nullable,
so it can't have a default value
```

This deviates from the current GraphQL specification, which permits
`$skip: Boolean! = true`. Such a document never reaches the guard — juniper's own
validation rejects it first — so defaulted-variable tests must declare the
variable **nullable** (`$skip: Boolean = true`), which the pinned stack accepts
in the non-null `if:` position. Both documents are now specified accordingly; a
test written with `Boolean! = true` would prove nothing.

The specification also now requires behavioural equivalence with the executor
across literal values, variables, operation defaults, request overrides, multiple
directives, and directives on fields, fragment spreads and inline fragments —
with tests asserting against **Juniper's observed execution** rather than an
independently written expectation table, and a stop condition of
`BLOCKED - MUTATION GUARD CANNOT MATCH PINNED JUNIPER EXECUTABLE-SELECTION SEMANTICS`
if equivalence proves impossible.

### 20.4 Required variable-default tests added

| Document | Variables | Executable occurrences | Expected |
|---|---|---|---|
| `@skip(if: $skip)` on the duplicate, `$skip: Boolean = true` | omitted | 1 | **accepted**, not rejected in `ENFORCE` |
| same | `{"skip": false}` | 2 | **rejected** in `ENFORCE`; would-be rejection in `OBSERVE` |
| `@include(if: $inc)` on the duplicate, `$inc: Boolean = false` | omitted | 1 | **accepted** |
| same | `{"inc": true}` | 2 | **rejected** |
| defaulted case through a **named fragment**, including a directive on the spread | omitted / overridden | 1 / 2 | accepted / rejected |
| defaulted case through an **inline fragment** | omitted / overridden | 1 / 2 | accepted / rejected |
| request value precedence over the operation default | supplied | per value | matches juniper's `or_insert` |
| no default declared, supplied `true` / `false` | supplied | 1 / 2 | accepted / rejected — no regression |

An omitted-but-defaulted variable is explicitly **resolved**, never classified as
an undecidable condition.

### 20.5 Finding 3 — identity sweep completed

The previous remediation claimed no binding unscoped store identity remained.
That claim was **false**. `THOTH-GQL-BATCH-01` still required descendant results
to be stored under

```text
(loader, shape, terminal key)
```

Corrected to

```text
(scope, loader, shape, terminal key)
```

with the intended semantics stated explicitly: "ordinary" means *not a special
ancestor-prefetched namespace*, it does **not** mean *unscoped*. The binding rule
is now

```text
same scope + same loader + same shape + same terminal key
  => one shared entry, whichever prefetch site produced it
different scope
  => distinct entries
```

Also corrected in the same sweep: the descendant invariant (task invariant 17)
and the descendant acceptance criterion, both of which referred to the "ordinary
terminal identity" without the scope component; the equivalent `ADR-0006`
validation-list entry; and the failure/child-lookup identities, now stated
explicitly as `(scope, loader, shape, attempted key set)` and
`(scope, loader, shape, parent key)`.

A full re-search of both documents for `(loader, shape)`,
`(loader, shape, key)`, `(loader identity, normalized load shape, parent key)`,
`ordinary terminal identity`, `one request`, `same request` and `request-wide`
leaves only: the deliberate contrasts describing what is *not* shared across
scopes, the narrower-invalidation-primitive discussion, explicit prohibitions of
the withdrawn wording, and historical sections. **No binding current-specification
identity remains unscoped.**

Same-scope reuse versus cross-scope isolation remains as corrected in the
previous remediation, with the unqualified "two prefetch sites in one request
issue no duplicate SQL" still explicitly prohibited.

### 20.6 Observability and runbook reconciled

The task previously stated "Required logs: none / Required metrics/alerts: none /
Operational runbook changes: none" while the guard elsewhere required a record
per rejection. Split by component:

- **loader store** — no production log, metric or alert required before first
  adoption; query-count evidence remains test and preview evidence;
- **mutation guard** — production observability **required**: one structured
  event per would-be rejection in `OBSERVE` and per actual rejection in
  `ENFORCE`, carrying the mode, the colliding response key and the operation name
  only when supplied, and never the document, variables, argument values or any
  publisher or user payload data.

A minimal runbook obligation was added covering how to change mode, what blocks
`ENFORCE`, what triggers rollback, and how to verify store unavailability outside
`ENFORCE`. No dashboard, alerting rule or on-call procedure is created.

### 20.7 Fresh risk classification

Re-run against `risk-classification.md` from scratch. Result: **`HIGH`**.

The material change is that **merge no longer activates the guard**, so
"production feature activation occurs at merge" is withdrawn as grounds and was
removed rather than left standing.

`HIGH` still holds on: production feature activation — at `ENFORCE`, which this
task specifies and delivers the path for; cross-repository API contract change —
`ENFORCE` changes accepted mutation requests for all clients including those
outside this repository; idempotency/deduplication — the guard exists to prevent
a duplicated write; changes to canonical data semantics; and changes capable of
broadening processing scope. Escalation rules apply: production query volume is
unknown and external clients cannot be enumerated.

Not `Critical`: no destructive migration, no canonical data rewrite, no mass
redistribution, no security boundary affecting all publishers (the guard only
rejects, so it cannot broaden access), no secrets work, no metrics recomputation,
no source-of-truth cutover, no material legal or contractual consequence, and
rollback is certain. Not `Medium`: `ENFORCE` changes a cross-client request
contract, beyond flagged behaviour with limited data effect.

The HIGH-risk "feature flag, comparison mode or controlled pilot where possible"
control is now discharged by the **`OBSERVE` mode** rather than by a kill switch.

### 20.8 Boundaries confirmed after this remediation

- **no runtime work was performed.** No runtime source, schema, migration,
  `Cargo` manifest, lock file or workflow changed. The effective-variable
  reproduction ran in a throwaway crate outside the repository and left nothing
  behind in it;
- **no production action was taken.** No mode was activated, no deployment made,
  no configuration changed;
- **no authorization was granted.** `ADR-0006` remains `PROPOSED`;
  `THOTH-GQL-BATCH-01` remains `DRAFT` and `HIGH` with implementation
  `NOT AUTHORIZED`;
- PR #788, its branch and the `BE-02` specification unmodified;
- issue #765 unmodified;
- `docs/publisher-services/task-status.md` unmodified;
- the implementation branch `feature/shared-architecture/graphql-batching` was
  not created;
- F1, F2 and F3 were not reopened, and F2 remains the selected architecture;
- authorization is unchanged: the guard makes no authorization decision, keys
  still come only from already-resolved rows, and scope identity remains
  isolation metadata rather than permission;
- no approval-state or transient-status prose was written into any committed
  file.
