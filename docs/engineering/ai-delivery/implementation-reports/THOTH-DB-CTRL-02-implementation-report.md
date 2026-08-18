# THOTH-DB-CTRL-02 Implementation Report

## 1. Repository state

Repository: `thoth-pub/thoth`
Workflow: STANDARD
Base branch: `develop`
Base commit: `4c53709befc91acb481beac54a1d314926b61d76`
PR target: `develop`
Programme integration branch: None
Task branch: `feature/repository-controls/thoth-db-ctrl-02`
Head commit: established after push through GitHub PR metadata and the new
immutable PR evidence comment that supersedes the earlier one (this file cannot
embed the SHA of its own containing commit)
Pull request: draft PR #778 (remains draft and unmerged; see §16)
Expected branch deletion after merge: YES
Final programme PR required: NO
Implementing model: Claude Opus 4.8
Reasoning level: HIGH/MAXIMUM

## 2. Scope confirmation

Approved specification:
[THOTH-DB-CTRL-02](../tasks/THOTH-DB-CTRL-02.md) and
[ADR-0003](../../decisions/ADR-0003-repository-authoritative-schema-contract.md),
delivered together in this bounded PR under the CTO's explicit 2026-08-05
authorization to combine the architecture decision and its directly related
cleanup (see the deliberate bounded exception in §0 of the task specification).

Implemented objective: adopt Architecture A — `thoth-api/src/schema.rs` is the
repository-authoritative, manually maintained Diesel schema contract; the Diesel
CLI and root `diesel.toml` are retired from the supported workflow; migrations,
`schema.rs`, models, and database-backed tests change atomically in future
bounded tasks. Record the decision (ADR-0003), retire the stale root
`diesel.toml`, align repository instructions and the migration CI, and reconcile
programme/dependency records — without changing any migration, `schema.rs`,
model, or runtime file.

Out-of-scope changes made: NONE.

## 3. Commits

- `c07cd43e` - docs: select repository-authoritative schema contract
- `f370a465` - chore: retire unused Diesel CLI configuration
- `a6e14a64` - docs: reconcile schema controls and BE-01 readiness
- `0d311dbe` - docs: report schema control replacement (reviewed pre-remediation
  head; independent review at this head returned `CHANGES REQUIRED`)
- remediation commit(s) on this branch - docs(control): correct
  THOTH-DB-CTRL-02 merge state and evidence (this remediation)

The authorized base is `4c53709befc91acb481beac54a1d314926b61d76`. The exact
final head is established after push, through GitHub PR metadata and a new
immutable PR evidence comment that supersedes the earlier one; the complete
ordered commit list and the final head SHA are recorded there. This file cannot
embed the SHA of its own containing commit, so no self-referential head SHA is
asserted here.

## 4. Files changed

Base-to-head `git diff --name-status`, 15 files:

```text
M	.github/workflows/run_migrations.yml
M	AGENTS.md
M	CHANGELOG.md
D	diesel.toml
A	docs/engineering/ai-delivery/implementation-reports/THOTH-DB-CTRL-02-implementation-report.md
M	docs/engineering/ai-delivery/tasks/BE-01.md
M	docs/engineering/ai-delivery/tasks/THOTH-DB-CTRL-01.md
A	docs/engineering/ai-delivery/tasks/THOTH-DB-CTRL-02.md
A	docs/engineering/decisions/ADR-0003-repository-authoritative-schema-contract.md
M	docs/engineering/decisions/decision-register.md
M	docs/engineering/repository-map/control-gaps.md
M	docs/engineering/repository-map/repositories/thoth.md
M	docs/metrics/task-status.md
M	docs/publisher-services/task-status.md
M	thoth-api/AGENTS.md
```

This report (`THOTH-DB-CTRL-02-implementation-report.md`) is itself one of the 15
changed files. The earlier draft of this report and PR body listed 14 files by
omitting the report itself; that count is corrected here and in the PR body.

Material files:

- `diesel.toml` (deleted)
  - reason: stale, unused root Diesel CLI configuration; it never parsed (missing
    commas), did not target the canonical `thoth-api/src/schema.rs`, and was not
    used by any supported build, test, migration, release, or runtime command.
  - behavioural effect: none on any supported command; the Diesel Rust crates and
    the embedded `diesel_migrations` runner are unaffected.
- `docs/engineering/decisions/ADR-0003-repository-authoritative-schema-contract.md` (added)
  - reason: record Architecture A.
  - behavioural effect: documentation; authoritative on merge.
- `docs/engineering/decisions/decision-register.md`
  - reason: register ADR-0003 and note the pending-merge state.
- `docs/engineering/ai-delivery/tasks/THOTH-DB-CTRL-02.md` (added)
  - reason: replacement task specification.
- `docs/engineering/ai-delivery/tasks/THOTH-DB-CTRL-01.md`
  - reason: mark `SUPERSEDED` with a prominent notice; historical body preserved.
- `AGENTS.md`, `thoth-api/AGENTS.md`
  - reason: describe the manual atomic schema workflow; remove any implication
    that a structural synchronizer is required.
  - behavioural effect: instruction change only.
- `.github/workflows/run_migrations.yml`
  - reason: add a reapply step so CI proves apply -> revert -> reapply of the full
    migration chain on the disposable PostgreSQL 17 service.
  - behavioural effect: one extra CI step; no application migration changed.
- `docs/engineering/ai-delivery/tasks/BE-01.md`
  - reason: remove the dependency on the rejected generator; BE-01 now edits
    `thoth-api/src/schema.rs` directly in its own bounded PR and must not use the
    Diesel CLI to generate the canonical contract.
- `docs/engineering/repository-map/control-gaps.md`,
  `docs/engineering/repository-map/repositories/thoth.md`,
  `docs/publisher-services/task-status.md`, `docs/metrics/task-status.md`,
  `CHANGELOG.md`
  - reason: reconcile CG-12 (`RESOLVED` by Architecture A), the repository map,
    BE-01 readiness (`READY`, separately authorization-gated), and the programme
    trackers to Architecture A; record the changelog entries. These committed
    records state the resulting authoritative status directly and become
    authoritative on merge into `develop`; CG-13 remains `OPEN`.
- `docs/engineering/ai-delivery/implementation-reports/THOTH-DB-CTRL-02-implementation-report.md`
  (added)
  - reason: this report. Records the base, truthfully nameable commits, exact
    protected-file evidence, the 15-file set, the independent-review outcome, and
    the remediation (see §16). The exact final head and exact-head CI are
    recorded in the superseding immutable PR evidence comment.

## 5. Implementation decisions

1. ADR-0003 was assigned as the next repository-wide ADR number and registered in
   `decision-register.md`.
2. The task ID `THOTH-DB-CTRL-02`, branch
   `feature/repository-controls/thoth-db-ctrl-02`, and ADR number ADR-0003 were
   all confirmed unused before creation; no substitution was needed.
3. `.github/scripts/classify_ci_changes.py` was left unchanged. It contains a
   `path == "diesel.toml"` build-classification entry, but that is a
   fail-safe path classifier (it maps a changed file named `diesel.toml` to the
   build category), not a Diesel CLI requirement. It does not break with the file
   removed, and touching CI classification logic and its historical CI-DOCS-01
   records would exceed the approved bounded scope.
4. Historical implementation reports of prior merged PRs (for example
   `THOTH-DB-CTRL-01-SPEC-implementation-report.md`,
   `BE-01-SPEC-implementation-report.md`, `CI-DOCS-01*`) were left byte-identical
   as immutable historical evidence.

Deviations from the specification: NONE.

## 6. Database and migration effects

Migration added: NO.

- migration files: none;
- schema effect: none; `thoth-api/src/schema.rs` and all files under
  `thoth-api/migrations/` are byte-identical to the base
  (`git diff --exit-code 4c53709b...HEAD -- thoth-api/src/schema.rs` and
  `-- thoth-api/migrations` both return 0);
- existing-data effect: none;
- locking/downtime: none;
- empty database result: not run locally (see §9); the migration workflow now
  applies, reverts, and reapplies the full chain on the CI disposable database;
- populated database result: not applicable; no migration is introduced;
- rollback/forward repair: revert the merge commit (see §12);
- idempotency: not applicable.

## 7. API and compatibility effects

GraphQL/API changes: none.
Generated schema/client updates: none.
Backwards compatibility: fully preserved; documentation-and-CI change only.
Deprecations: none (the Diesel Rust crates remain in use; only the unused
`diesel.toml` CLI configuration is retired).
Cross-repository dependencies: none.

## 8. Authorization and security

Authorization paths changed: none.
Roles/scopes involved: none.
Negative authorization tests: not applicable.
Secret or personal-data handling: none. No secrets, production, staging, or
shared-database access occurred.
Security limitations: none introduced.

## 9. Tests and checks

### Formatting

Command:

```text
cargo fmt --all -- --check
```

Result:

```text
exit 0 (no formatting differences)
```

### Backend check

Command:

```text
cargo check -p thoth-api --features backend
```

Result:

```text
Finished `dev` profile ... exit 0
```

### Backend tests (compilation)

Command:

```text
cargo test -p thoth-api --features backend --no-run
```

Result:

```text
Finished `test` profile ... exit 0 (all test targets compiled)
```

### Backend tests (execution)

Command:

```text
cargo test -p thoth-api --features backend
```

Result:

```text
test result: FAILED. 436 passed; 409 failed; 0 ignored
```

All 409 failures are the identical environment limitation: the test harness
(`thoth-api/src/model/tests.rs:68`, "Failed to run migrations for test DB")
cannot reach a database — `connection to server at "localhost" ... port 5432
failed: Connection refused`. No local PostgreSQL is available (see below). The
436 passing tests are the non-database unit tests. No Rust code was changed by
this task, so these failures reflect the absence of a local database, not a code
regression. Full database-backed execution is deferred to the exact-head GitHub
Actions jobs.

### Migration apply/revert/reapply (disposable database)

Local execution: UNAVAILABLE. The Docker daemon is not running, and only
PostgreSQL 18 client tooling is present locally (no PostgreSQL 17 server, which
is what CI and production use). Per the task's validation guidance, no claim is
substituted. The exact-head `run-migrations` workflow provides this evidence on
a disposable `postgres:17` service and now runs:

```text
cargo run migrate          # apply full chain
cargo run migrate --revert # revert full chain (revert_all_migrations)
cargo run migrate          # reapply full chain (added by this task)
```

### Workflow lint

`actionlint` is not installed in this environment; the `run_migrations.yml`
change was reviewed by hand (a single added `Reapply migrations` step mirroring
the existing `Run migrations` step). Not silently installed.

### Git validation

```text
git status --short                -> clean
git rev-parse HEAD                -> reviewed head 0d311dbe...; the remediation adds further commit(s), and the exact final head is recorded in the superseding immutable PR evidence comment
git rev-parse origin/develop      -> 4c53709befc91acb481beac54a1d314926b61d76
git merge-base --is-ancestor 551565d0... origin/develop -> exit 1 (PR #777 head NOT an ancestor)
git diff --check                  -> clean
test ! -e diesel.toml             -> OK (absent)
git diff --exit-code <base>...HEAD -- thoth-api/src/schema.rs   -> 0 (identical)
git diff --exit-code <base>...HEAD -- thoth-api/migrations      -> 0 (identical)
```

Diesel-token searches (`git grep`) were run; every remaining match is either a
new prohibitive instruction, clearly-marked superseded history
(`THOTH-DB-CTRL-01*`), or a harmless fail-safe CI path-classification reference
(`classify_ci_changes.py`, `CI-DOCS-01*`). No active instruction requires the
Diesel CLI.

## 10. Manual verification

Environment: local working tree on branch
`feature/repository-controls/thoth-db-ctrl-02`.
Steps: inspected the rendered ADR, replacement spec, supersession notice, AGENTS
edits, CI reapply step, and tracker reconciliations; confirmed byte identity of
protected files and deletion of `diesel.toml`.
Observed result: changes match the approved scope; 15 files changed (including
this report), none under `thoth-api/src`, `thoth-api/migrations`, `src`, or any
Cargo manifest.
Evidence: §9 command results.

## 11. CI

CI status: exact-head CI is re-run at the new remediation head after push; runs
from the pre-remediation head `0d311dbe` are not treated as evidence for the new
head. Checks: `build_test_and_check`, `run-migrations` (apply/revert/reapply on
`postgres:17`), changelog, and classifier gating.
Failures or warnings: the exact workflow names, run IDs, conclusions, head SHA,
and migration apply/revert/reapply evidence at the new head are recorded in the
superseding immutable PR evidence comment.

## 12. Rollout and rollback

Initial state after merge: repository instructions and CI change only; no runtime
feature activates; no migration is introduced; no production service changes;
future schema-bearing tasks follow Architecture A; after a separate
authorization, BE-01 may create its own branch.
Activation required: none.
Feature flag/configuration: none.
Migration sequence: none introduced by this task.
Rollback/disable procedure: revert the complete merge commit. Restoring
`diesel.toml` alone is not a valid rollback (it would recreate the misleading
unsupported configuration). Any future move to generated-schema authority
requires a separate ADR and implementation task.
Monitoring required: none.

## 13. Known limitations and deferred work

- Local database-backed test execution and the disposable-database
  apply/revert/reapply migration run were unavailable (no local PostgreSQL 17; no
  Docker daemon). These are covered by the exact-head CI jobs.
- `actionlint` was unavailable; the workflow edit was reviewed by hand.
- CG-13 (Thoth runtime operations unmapped) remains open; production migration,
  deployment, rollback, restore verification, and approver mapping are outside
  this task.

## 14. Unresolved issues

- NONE beyond the environment limitations in §13.

## 15. Agent self-assessment

The implementing agent does not approve its own work. This task requires a fresh,
independent, exact-head review of the complete base-to-head diff and explicit CTO
merge authorization bound to the exact reviewed head before it may merge. The PR
is left in draft.

State summary:

```text
Architecture A: APPROVED AND IMPLEMENTED in PR #778 - authoritative on merge into develop
Old THOTH-DB-CTRL-01: SUPERSEDED
PR #777: CLOSED UNMERGED
CG-12: RESOLVED by Architecture A (authoritative on merge)
CG-13: OPEN
BE-01: READY on merge, separately authorization-gated for implementation
BE-01 implementation branch: ABSENT
BE-01 implementation: NOT STARTED
Production migration/release authorization: NONE
```

The committed records state this resulting authoritative status directly; no
transient open/draft/unmerged PR wording is left to survive into `develop`. The
merge itself remains gated by fresh independent exact-head review and explicit
CTO merge authorization (see §16).

Suggested review focus:

- byte identity of `thoth-api/src/schema.rs` and `thoth-api/migrations/`;
- absence of any replacement Diesel CLI configuration, synchronizer, convention
  file, or expected-change manifest;
- consistency of CG-12 (`RESOLVED`) and BE-01 (`READY`, separately
  authorization-gated) across the committed records, with no residual
  open/draft/unmerged PR state surviving into `develop`;
- that no PR #777 commit is merged or cherry-picked;
- the migration reapply CI step and its disposable-database boundaries.

Independent review: PENDING.
CTO merge authorization: PENDING.

## 16. Remediation (independent-review response)

Independent review of head `0d311dbe1ea2c1305e8799a7d2cfec22eaed7d3e`:
`CHANGES REQUIRED`.

The review found three classes of defect. This remediation corrected them on the
same branch and PR, without changing the approved architecture (ADR-0003,
Architecture A) or expanding implementation scope, and without touching any file
under `thoth-api/migrations/`, `thoth-api/src/schema.rs`, Rust source, models,
Cargo manifests, or runtime/GraphQL/API code:

1. **Stale post-merge state (Finding 1).** Committed trackers and repository maps
   encoded PR #778's transient open/draft/unmerged state. They now state the
   resulting authoritative status directly — CG-12 `RESOLVED` by Architecture A,
   BE-01 `READY` (separately authorization-gated), ADR-0003 accepted and
   implemented, THOTH-DB-CTRL-02 implemented — becoming authoritative on merge
   into `develop`. CG-13 remains `OPEN`. Corrected in `control-gaps.md`,
   `repositories/thoth.md`, `decision-register.md`, `docs/metrics/task-status.md`,
   `docs/publisher-services/task-status.md`, `THOTH-DB-CTRL-02.md`, and this
   report. The CG-12 heading anchor was updated in `repositories/thoth.md` to
   match its new heading.
2. **BE-01 readiness contradiction (Finding 2).** One consistent model is now
   used everywhere: merging PR #778 resolves CG-12 and records BE-01 `READY`;
   `READY` does not authorize implementation; creating the BE-01 branch and any
   implementation edit require separate explicit authorization; the branch
   remains absent. All language requiring a second repository-control update
   merely to move BE-01 from `BLOCKED` to `READY` was removed from `BE-01.md`,
   `docs/publisher-services/task-status.md`, and `docs/metrics/task-status.md`.
   No approved higher-authority document requires such a separate update, so no
   `BLOCKED` conflict arises.
3. **Evidence and changed-file accounting (Finding 3).** The changed-file set is
   15 (verified from Git), correcting the earlier 14 that omitted this report.
   The `<this report commit>` placeholder was removed; only truthfully nameable
   commits are listed, and no self-referential head SHA is embedded. Stale head
   and CI statements were corrected, and the exact final head and exact-head CI
   are recorded in the superseding immutable PR evidence comment.

The executable migration-CI change (`run_migrations.yml` apply/revert/reapply)
was accepted in principle by the review and was not redesigned; repository
evidence revealed no defect in it.

Fresh independent review of the new head: PENDING.
CTO merge authorization: PENDING.
Production authorization: absent (none requested or granted).

A new immutable PR evidence comment records the exact final head, ordered commit
list, 15-file set, protected-file identity checks, exact-head CI, and migration
apply/revert/reapply evidence. It supersedes the earlier immutable evidence
comment, which is left unedited.
