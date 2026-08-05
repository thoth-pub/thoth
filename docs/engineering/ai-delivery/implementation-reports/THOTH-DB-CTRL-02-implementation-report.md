# THOTH-DB-CTRL-02 Implementation Report

## 1. Repository state

Repository: `thoth-pub/thoth`
Workflow: STANDARD
Base branch: `develop`
Base commit: `4c53709befc91acb481beac54a1d314926b61d76`
PR target: `develop`
Programme integration branch: None
Task branch: `feature/repository-controls/thoth-db-ctrl-02`
Head commit: recorded at push time (see the immutable PR evidence comment)
Pull request: draft PR #778 (opened after push)
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
- `<this report commit>` - docs: report schema control replacement

(Exact final head SHA is recorded in the PR evidence comment after push.)

## 4. Files changed

Base-to-head `git diff --name-status`:

```text
M	.github/workflows/run_migrations.yml
M	AGENTS.md
M	CHANGELOG.md
D	diesel.toml
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
  - reason: reconcile CG-12, the repository map, BE-01 readiness, and the
    programme trackers to Architecture A; record the changelog entries.

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
git rev-parse HEAD                -> a6e14a64... (pre-report head; final head in PR comment)
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
Observed result: changes match the approved scope; 14 files changed, none under
`thoth-api/src`, `thoth-api/migrations`, `src`, or any Cargo manifest.
Evidence: §9 command results.

## 11. CI

CI status: PENDING (populated after the draft PR opens and workflows run).
Checks: `build_test_and_check`, `run-migrations` (apply/revert/reapply on
`postgres:17`), changelog, and classifier gating.
Failures or warnings: to be recorded in the immutable PR evidence comment at the
exact final head.

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
Architecture A: APPROVED AND (draft) IMPLEMENTED - unmerged
Old THOTH-DB-CTRL-01: SUPERSEDED
PR #777: CLOSED UNMERGED
CG-12: RESOLUTION PENDING PR #778 MERGE (unresolved while the PR is open)
CG-13: OPEN
BE-01: BLOCKED while PR #778 is open; READY on merge
BE-01 implementation branch: ABSENT
BE-01 implementation: NOT STARTED
Production migration/release authorization: NONE
```

Suggested review focus:

- byte identity of `thoth-api/src/schema.rs` and `thoth-api/migrations/`;
- absence of any replacement Diesel CLI configuration, synchronizer, convention
  file, or expected-change manifest;
- temporal accuracy of the CG-12 / BE-01 wording (unresolved/blocked while open;
  resolved/ready on merge);
- that no PR #777 commit is merged or cherry-picked;
- the migration reapply CI step and its disposable-database boundaries.

Independent review: PENDING.
CTO merge authorization: PENDING.
