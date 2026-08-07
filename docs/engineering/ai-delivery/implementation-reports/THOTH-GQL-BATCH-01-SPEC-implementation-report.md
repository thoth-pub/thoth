# THOTH-GQL-BATCH-01-SPEC Implementation Report

Documentation-only architecture and specification-authoring task. It writes
`ADR-0006` as a `PROPOSED` shared architecture decision and the bounded
`THOTH-GQL-BATCH-01` runtime implementation specification. It implements no
runtime code and authorizes nothing.

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
