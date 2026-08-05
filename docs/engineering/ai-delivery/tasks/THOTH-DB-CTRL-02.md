# THOTH-DB-CTRL-02 - Adopt the repository-authoritative schema contract

Status: IMPLEMENTED (delivered through PR #778; authoritative on merge into
`develop`, which remains subject to independent exact-head review and explicit
CTO merge authorization)
Programme: Shared Repository Controls
Repository: `thoth-pub/thoth`
Workflow: STANDARD
Base branch: `develop`
Exact base commit: `4c53709befc91acb481beac54a1d314926b61d76`
PR target: `develop`
Programme integration branch: None
Risk: HIGH
Owner: CTO
Approved by: Javi, CTO
CTO authorization date: 2026-08-05 (Architecture A selected; combined
architecture-and-cleanup draft PR explicitly authorized)
Dependencies: [ADR-0003](../../decisions/ADR-0003-repository-authoritative-schema-contract.md)
(recorded in this same PR); supersedes [THOTH-DB-CTRL-01](THOTH-DB-CTRL-01.md)
Target branch name: `feature/repository-controls/thoth-db-ctrl-02`

## 0. Deliberate bounded exception to specification-then-implementation

This replacement task documents **and** implements its decision in one bounded
draft PR. The usual sequence separates an approved specification from a later
implementation task. The CTO explicitly authorized combining the architecture
decision (ADR-0003), the repository-control correction, the stale Diesel CLI
configuration removal, and the programme/dependency reconciliation here because
they form one indivisible replacement of the abandoned PR #777 approach.

This exception applies only to this replacement task. It does not waive:

- independent review;
- exact-head evidence;
- CI;
- explicit CTO merge authorization;
- production controls.

## 1. Objective

Adopt Architecture A as the repository's schema-authority model:
`thoth-api/src/schema.rs` is the repository-authoritative, manually maintained
Rust/Diesel compile-time schema contract; ordered migrations under
`thoth-api/migrations/` and the embedded `cargo run migrate` runner remain the
supported database-evolution workflow; and the Diesel CLI and root `diesel.toml`
are retired from the supported build, test, migration, and schema-generation
workflow. Record the decision as ADR-0003, retire the stale root `diesel.toml`,
align repository instructions and the migration CI with the manual atomic
workflow, and reconcile programme and task records — without changing any
migration, `schema.rs`, model, or runtime file.

### 1.1 Risk rationale

HIGH because the control governs how every future populated-database change in
multiple programmes keeps `schema.rs` in step with migrations, and because it
removes a configuration file and rewrites shared repository instructions. It
changes no production schema or runtime behaviour, but the control it
establishes must be independently reviewed and explicitly approved before merge.

## 2. Background and authority

Authoritative sources, in precedence order:

1. merged migrations under `thoth-api/migrations/`, the migrated PostgreSQL
   schema, `thoth-api/src/schema.rs`, and the Rust models that consume it;
2. [ADR-0003](../../decisions/ADR-0003-repository-authoritative-schema-contract.md)
   (recorded in this PR);
3. `AGENTS.md` and `thoth-api/AGENTS.md`;
4. [CG-12 and CG-13](../../repository-map/control-gaps.md);
5. the [`thoth` repository map](../../repository-map/repositories/thoth.md);
6. [BE-01](BE-01.md), the first dependent schema-bearing task;
7. the Publisher Services and Thoth Metrics trackers.

Background: CG-12 asked how `schema.rs` should track migrations.
`THOTH-DB-CTRL-01` answered with a bespoke synchronizer (catalog introspection,
raw `diesel print-schema`, a convention file, a custom structural synchronizer).
Its implementation PR [#777](https://github.com/thoth-pub/thoth/pull/777) was
closed unmerged; no code from it became authoritative. Disposable-database
investigation established that raw Diesel output cannot reproduce the checked-in
contract's aliases, supplemental type, timestamp semantics, column order, or
formatting, and that root `diesel.toml` neither parsed nor targeted the
canonical path. The CTO selected Architecture A on 2026-08-05.

## 3. Explicit scope

The task must:

1. add [ADR-0003](../../decisions/ADR-0003-repository-authoritative-schema-contract.md)
   recording the repository-authoritative `schema.rs` decision, and register it
   in the decision register;
2. mark [THOTH-DB-CTRL-01](THOTH-DB-CTRL-01.md) `SUPERSEDED` with a prominent
   notice, preserving its historical body;
3. delete the stale root `diesel.toml`;
4. update `AGENTS.md` and `thoth-api/AGENTS.md` to describe the manual atomic
   workflow and remove any implication that a structural synchronizer is
   required;
5. add to `.github/workflows/run_migrations.yml` the smallest useful missing
   verification: reapply the full migration chain after the existing full-chain
   revert, so CI proves apply -> revert -> reapply on a disposable database;
6. update [BE-01](BE-01.md) so it no longer depends on the rejected generator
   and edits `schema.rs` directly in its own future bounded PR;
7. reconcile control and programme records: `control-gaps.md`,
   `repositories/thoth.md`, `docs/publisher-services/task-status.md`,
   `docs/metrics/task-status.md`, and `CHANGELOG.md`;
8. produce the implementation report with exact evidence.

## 4. Non-goals

The task must not:

1. modify any file under `thoth-api/migrations/`;
2. modify `thoth-api/src/schema.rs`;
3. modify Rust models, runtime, GraphQL, or public API code;
4. change Cargo dependency declarations or remove the Diesel Rust crates;
5. add a replacement Diesel CLI configuration, convention file, expected-change
   manifest, `check-diesel-schema`/`generate-diesel-schema` target, pinned
   `diesel_cli` installation, or catalog/raw/canonical reconciliation logic;
6. copy, repair, reopen, or build on PR #777 or its branch;
7. implement any part of BE-01 or create the BE-01 branch;
8. execute a production, staging, or shared-database migration;
9. merge, deploy, release, or activate anything;
10. broaden into unrelated architecture, runtime, or model changes.

## 5. Invariants

The implementation must preserve:

1. `thoth-api/src/schema.rs` byte-identical to the base;
2. every file under `thoth-api/migrations/` byte-identical to the base;
3. Rust models, runtime code, GraphQL, and public API unchanged;
4. Cargo dependency declarations unchanged; the Diesel library and embedded
   `diesel_migrations` runner remain in use;
5. the disposable PostgreSQL service and existing safety boundaries in
   `run_migrations.yml`;
6. PR #777 closed and unmerged, with none of its commits merged or
   cherry-picked;
7. production migration and release remain governed by CG-13 and separate
   authorization.

## 6. Required behaviour

### 6.1 Success behaviour

After merge, ADR-0003 is repository-authoritative; root `diesel.toml` is gone;
repository instructions describe the manual atomic workflow; the migration CI
applies, reverts, and reapplies the full chain on a disposable database; CG-12 is
resolved by this merged Architecture A control; and BE-01 is `READY` for
separately authorized implementation that edits `schema.rs` directly.

### 6.2 Failure behaviour

Stop without pushing or opening a PR if the exact base cannot be established, PR
#777 has merged or its head is an ancestor of `develop`, repository evidence
shows the Diesel CLI is actively required by a supported workflow, deleting
`diesel.toml` breaks a supported command, or the change would expand into
runtime, model, migration, API, or production behaviour.

### 6.3 Authorization

No authorization surface changes. No runtime code changes.

### 6.4 Concurrency and idempotency

Not applicable. No runtime, job, or data path changes.

### 6.5 Compatibility

Documentation-and-CI change only. No API, database, client, or deployment
contract changes. Deleting `diesel.toml` does not affect any supported build,
test, migration, release, or runtime command.

## 7. Data and migration requirements

Migration required: NO.

- schema changes: none; `thoth-api/src/schema.rs` and `thoth-api/migrations/`
  are byte-identical to the base;
- populated database behaviour: unchanged;
- locking/downtime: none;
- data backfill: none;
- idempotency: not applicable;
- rollback or forward-repair strategy: revert the merge commit (see §12);
- empty database test: the migration CI applies, reverts, and reapplies the
  existing full chain on a disposable PostgreSQL 17 service;
- populated database test: not applicable; no migration is introduced.

## 8. Observability and operations

Required logs: none. Required metrics/alerts: none. Operational runbook changes:
none. Production migration, deployment, rollback, restore, and approver mapping
remain governed by CG-13 and separate release authorization.

## 9. Acceptance criteria

- [ ] One bounded branch created from exact `develop`
  `4c53709befc91acb481beac54a1d314926b61d76`.
- [ ] PR #777 remains closed and unmerged; no PR #777 commit is merged or
  cherry-picked.
- [ ] ADR-0003 selects repository-maintained `schema.rs` and is registered.
- [ ] THOTH-DB-CTRL-01 is preserved but marked `SUPERSEDED`.
- [ ] This replacement specification exists.
- [ ] Root `diesel.toml` is removed and no replacement Diesel CLI configuration,
  synchronizer, convention file, or expected-change manifest is added.
- [ ] `AGENTS.md` and `thoth-api/AGENTS.md` describe the manual atomic workflow.
- [ ] Migration CI applies, reverts, and reapplies the chain on a disposable
  database.
- [ ] `thoth-api/src/schema.rs` and existing migrations are byte-identical to the
  base; Rust models and runtime code are unchanged.
- [ ] BE-01 no longer depends on the rejected generator and may edit `schema.rs`
  directly in its future bounded task.
- [ ] CG-12, CG-13, Publisher Services, and Metrics records are mutually
  consistent: CG-12 resolves on merge into `develop` and BE-01 is recorded
  `READY` on the same merge, with implementation separately authorization-gated;
  the committed records state this resulting authoritative status directly
  rather than preserving transient open-PR wording.
- [ ] The diff contains no BE-01 implementation and no BE-01 branch was created.
- [ ] No production, staging, shared-database, release, deployment, activation,
  or secret access occurred.
- [ ] Local checks pass and the implementation report contains exact evidence.
- [ ] Full CI passes at the exact final head.
- [ ] A separate agent reviews the complete base-to-head diff; the implementing
  agent does not approve its own work.
- [ ] The PR is not merged without explicit CTO authorization bound to the exact
  reviewed head.

## 10. Required tests

### Unit

- Not applicable; no runtime code changes.

### Integration/database

- The `run_migrations` workflow applies (`cargo run migrate`), reverts
  (`cargo run migrate --revert`), and reapplies (`cargo run migrate`) the full
  chain on the disposable PostgreSQL 17 service.

### Authorization/security

- Not applicable; no authorization surface changes.

### Regression

- Workspace formatting, `thoth-api` backend build/test, and (where the
  environment permits) full workspace checks confirm no unintended code impact
  from documentation and CI edits.

### Manual verification

- `test ! -e diesel.toml` confirms deletion;
- `git diff --exit-code <base>...HEAD -- thoth-api/src/schema.rs` and
  `-- thoth-api/migrations` confirm byte identity;
- `git grep` for Diesel CLI tokens shows only clearly identified superseded
  history, with no active instruction requiring the Diesel CLI.

### Performance

Not applicable.

## 11. Rollout

- initial state after merge: repository instructions and CI change only; no
  runtime feature activates; no migration is introduced; no production service
  changes; future schema-bearing tasks follow Architecture A; after a separate
  authorization, BE-01 may create its own branch;
- feature flag/configuration: none;
- staging/preview validation: not applicable; exact-head GitHub Actions provide
  the disposable-database migration evidence;
- pilot: not applicable;
- activation approval: not applicable to this control;
- observation period: not applicable.

## 12. Rollback

- code rollback: revert the complete merge commit if the architecture decision or
  cleanup must be withdrawn;
- data rollback or forward repair: not applicable; no data or schema changes;
- feature disable/kill switch: not applicable;
- external side-effect handling: none; restoring `diesel.toml` alone is not a
  valid rollback because it would recreate the misleading unsupported
  configuration. Any future move to generated-schema authority requires a
  separate ADR and implementation task.

## 13. Stop conditions

The implementing agent must stop and report `BLOCKED` if:

- the working tree contains unrelated changes that cannot be separated safely;
- the exact base cannot be established;
- PR #777 has merged or its head is an ancestor of `develop`;
- the proposed ADR conflicts with an existing active decision;
- repository evidence shows the Diesel CLI is actively required by a supported
  workflow;
- deleting `diesel.toml` breaks a supported command not covered by this decision;
- the required changes expand into runtime, model, migration, API, or production
  behaviour;
- approved architecture would need to change;
- required production information or secrets are unavailable;
- scope cannot be completed without unrelated changes.

## 14. Expected implementation report

The agent must use:

`docs/engineering/ai-delivery/implementation-report-template.md`

and record it at
`docs/engineering/ai-delivery/implementation-reports/THOTH-DB-CTRL-02-implementation-report.md`.

## 15. Recommended execution

Implementation model: Claude Opus 4.8 (maximum/high reasoning)
Reasoning level: MAXIMUM/HIGH
Independent reviewer: a separate cross-model reviewer that did not implement the
task
Review reasoning level: HIGH or MAXIMUM

The implementing agent may provide a self-assessment but cannot approve the
implementation.

## 16. Branch and integration plan

- branch source: exact `develop` `4c53709befc91acb481beac54a1d314926b61d76`;
- pull-request target: `develop`;
- expected merge order: independent exact-head review, explicit CTO merge
  authorization, then merge into `develop`;
- parent programme branch refresh requirement: not applicable;
- branch deletion after merge: YES;
- final programme PR required: NO;
- final release path: `develop -> master`.

## 17. Approval

Approved for implementation by: Javi, CTO

Date: 2026-08-05

Notes:

- The CTO selected Architecture A and authorized recording the decision and its
  directly related cleanup in one draft PR.
- This authorization does not authorize merge, deployment, release, production
  migration, secrets access, creation of the BE-01 branch, or implementation of
  BE-01.
- The PR remains unmerged pending fresh independent review and explicit CTO
  merge authorization bound to the exact reviewed head.
