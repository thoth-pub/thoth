# BE-01-SPEC Implementation Report

## 1. Repository state

Repository: `thoth-pub/thoth`

Programme: Publisher Services and Distribution Configuration

Task ID: `BE-01-SPEC`

Risk: HIGH

Workflow: STANDARD

Base branch: `develop`

Base commit: `4bd95587809637e1b3a03b8d5cdfed877779aecc`

PR target: `develop`

Programme integration branch: None

Task branch: `feature/publisher-services/be-01-spec`

Historical reviewed head:
`6d4067d4ee196976e9d06074d3bd790cd3b0096a`

Independent review at that head: `CHANGES REQUIRED` with one P1 because BE-01
was prematurely marked `READY` while the shared `THOTH-DB-CTRL-01` Diesel
control remained blocked.

First corrective commit and exact corrected head:
`537656cb2187ab91eec30fbbd64c89e495b3a4e9`, recorded in the prior superseding
immutable top-level PR evidence comment.

Historical post-ready reviewed head:
`537656cb2187ab91eec30fbbd64c89e495b3a4e9`

The post-ready automated review at that head raised P1 thread
`PRRT_kwDODkn0bc6U18En`. It correctly identified ambiguity in the
migration-evidence description: the repository's `--revert` command reverts the
complete migration history, so it cannot independently prove preservation of a
populated pre-BE-01 baseline after an isolated BE-01 rollback.

The exact-head approval and CTO authorization for
`537656cb2187ab91eec30fbbd64c89e495b3a4e9` are historical after the bounded
rollback-evidence correction creates a new head.

Historical head after the rollback-evidence correction:
`85a83d857b56873f95a0d2011ac06ad0d61319ed`

Post-ready review: `PRR_kwDODkn0bc8AAAABHsdgiA`

P1 thread: `PRRT_kwDODkn0bc6U2xH6`

Decision: `CHANGES REQUIRED`.

The post-ready review correctly identified that the specification allowed
destructive removal of the package foundation even though higher-authority
ADR-0001 requires operational rollback to retain the package column and mapping
code. No ADR change was approved or necessary. This correction distinguishes
disposable down-migration testing from approved operational rollback, retains
the schema, stored values, domain types, and canonical mapping foundation
operationally, and requires an ADR change plus a separate authorized migration
task before any foundation removal.

The exact-head independent approval and CTO authorization for
`85a83d857b56873f95a0d2011ac06ad0d61319ed` become historical when this
corrective commit creates a new head.

Historical head after the ADR-0001 rollback alignment:
`9b68bc3d4008e5aba1c243c523e4c5b2d36d308e`

Post-ready review: `PRR_kwDODkn0bc8AAAABHsstmA`

Decision: `CHANGES REQUIRED`.

P1 thread `PRRT_kwDODkn0bc6U3SZ9`: BE-01 must be HIGH risk.

P1 thread `PRRT_kwDODkn0bc6U3SaA`: GraphQL enum representations must be
mandatory.

The review correctly identified that the former MEDIUM classification
conflicted with repository risk policy and that optional GraphQL wording
conflicted with ADR-0001 and the specification's own tests. A populated-table
migration and package entitlements independently trigger HIGH, and the highest
applicable classification applies. No design or ADR amendment was required.
The correction changes control classification and future implementation
requirements only; no code, migration, schema, GraphQL runtime, database, or
external action occurred.

The exact-head independent approval and CTO authorization for
`9b68bc3d4008e5aba1c243c523e4c5b2d36d308e` become historical when this
corrective commit creates a new head.

Pull request: [#774](https://github.com/thoth-pub/thoth/pull/774)

Expected branch deletion after merge: YES

Final programme PR required: NO

Implementing model: Codex / GPT-5

Reasoning level: High

Independent reviewer/model: OpenAI ChatGPT / GPT-5.6 Thinking

Review reasoning level: High

## 2. Scope confirmation

Approved specification: the CTO-authorized BE-01-SPEC execution specification
dated 2026-07-29.

Implemented objective: create and register an approved, repository-backed,
bounded implementation specification for BE-01 Publisher package model without
implementing the model.

Out-of-scope changes made: NONE.

This PR is documentation-only. Nevertheless, its repository-authoritative
specification governs a HIGH-risk implementation task. Recording HIGH
consistently in the specification, tracker, report, and PR body prevents the
future implementation from receiving MEDIUM-risk controls. Migration of the
populated `publisher` table and establishment of package entitlements each
independently trigger HIGH under the repository policy, which requires the
highest applicable classification.

The final cumulative PR changes exactly:

```text
CHANGELOG.md
docs/engineering/ai-delivery/implementation-reports/BE-01-SPEC-implementation-report.md
docs/engineering/ai-delivery/tasks/BE-01.md
docs/publisher-services/README.md
docs/publisher-services/task-status.md
```

No other path is authorized or changed.

The fifth, rollback-evidence correction changes exactly:

```text
docs/engineering/ai-delivery/implementation-reports/BE-01-SPEC-implementation-report.md
docs/engineering/ai-delivery/tasks/BE-01.md
```

It adds no migration command or implementation mechanism and changes no Rust,
SQL, migration, schema, tooling, workflow, or runtime path.

The sixth, ADR-0001 rollback-alignment correction changes exactly:

```text
docs/engineering/ai-delivery/implementation-reports/BE-01-SPEC-implementation-report.md
docs/engineering/ai-delivery/tasks/BE-01.md
```

It changes no ADR and performs no code, migration, schema, runtime, database, or
operational action.

The seventh, risk-and-GraphQL correction changes exactly:

```text
docs/engineering/ai-delivery/implementation-reports/BE-01-SPEC-implementation-report.md
docs/engineering/ai-delivery/tasks/BE-01.md
docs/publisher-services/task-status.md
```

It changes control classification and future implementation requirements only.

## 3. Commits

1. `b325eaa1d46a672d69961597cf1c8d8740cf7996` -
   `docs: specify BE-01 publisher package model`
2. `f7c06b7e8c946e29d754e36a44da9419264a0e6c` -
   `docs: register BE-01 implementation readiness`
3. `6d4067d4ee196976e9d06074d3bd790cd3b0096a` -
   `docs: report BE-01 specification task`
4. `537656cb2187ab91eec30fbbd64c89e495b3a4e9` -
   `docs: block BE-01 on shared Diesel control`
5. `85a83d857b56873f95a0d2011ac06ad0d61319ed` -
   `docs: clarify BE-01 migration rollback evidence`
6. `9b68bc3d4008e5aba1c243c523e4c5b2d36d308e` -
   `docs: align BE-01 rollback with ADR-0001`
7. The bounded correction classifying BE-01 HIGH and requiring GraphQL enum
   representations in the specification, this report, and the tracker is
   recorded in the superseding immutable evidence comment after it exists.

No evidence-only eighth commit is permitted.

## 4. Files changed

- `docs/engineering/ai-delivery/tasks/BE-01.md`
  - reason: establish the complete approved BE-01 implementation contract;
  - behavioural effect: none; documentation and future implementation control
    only.
- `CHANGELOG.md`
  - reason: add the required Unreleased Changed entry for PR #774;
  - behavioural effect: none.
- `docs/publisher-services/README.md`
  - reason: record specification approval, blocked implementation, and the
    BE-01/BE-03 boundary;
  - behavioural effect: none.
- `docs/publisher-services/task-status.md`
  - reason: classify BE-01 HIGH while keeping it `BLOCKED` on
    `THOTH-DB-CTRL-01` after specification merge and preserving all later
    gates;
  - behavioural effect: none.
- `docs/engineering/ai-delivery/implementation-reports/BE-01-SPEC-implementation-report.md`
  - reason: record scope, evidence, effects, handoff, and the corrected
    rollback-evidence distinction for this specification task;
  - behavioural effect: none.

## 5. Authoritative sources inspected

Repository instructions and templates:

- `AGENTS.md`;
- `docs/engineering/AGENTS.md`;
- `thoth-api/AGENTS.md`;
- `docs/engineering/ai-delivery/task-specification-template.md`;
- `docs/engineering/ai-delivery/implementation-report-template.md`;
- `docs/engineering/ai-delivery/operating-model.md`;
- `docs/engineering/ai-delivery/branching-and-release-workflow.md`;
- `docs/engineering/ai-delivery/risk-classification.md`;
- `docs/engineering/ai-delivery/release-gates.md`.

Approved decisions and design:

- `docs/engineering/decisions/ADR-0001-publisher-package-capability-model.md`;
- `docs/engineering/decisions/package-capability-matrix.md`;
- `docs/engineering/decisions/ADR-0002-platform-domain-boundaries.md`;
- private `Publisher Services and Distribution Configuration - Technical
  Design and Implementation Plan`, with Drive metadata and content verified at
  current revision `3`.

Publisher Services and repository controls:

- `docs/publisher-services/README.md`;
- `docs/publisher-services/decisions.md`;
- `docs/publisher-services/task-status.md`;
- `docs/publisher-services/acceptance-matrix.md`;
- `docs/publisher-services/rollout-plan.md`;
- `docs/engineering/repository-map/control-gaps.md`;
- `docs/engineering/repository-map/repositories/thoth.md`;
- live issue #765, inspected read-only.

Implementation surfaces:

- `thoth-api/src/model/publisher/`;
- `thoth-api/src/graphql/`;
- `thoth-api/src/policy.rs`;
- `thoth-api/src/db.rs`;
- `thoth-api/src/schema.rs`;
- `thoth-api/migrations/`;
- `thoth-api/src/model/tests.rs`;
- `thoth-api/src/graphql/tests.rs`;
- `thoth-client/`;
- `diesel.toml`;
- `Makefile`;
- relevant Cargo and CI workflow configuration;
- `CHANGELOG.md`.

No authoritative-source conflict was found.

## 6. Current publisher-model and GraphQL findings

### 6.1 Persisted model and inputs

- `Publisher` is a serde-serializable Diesel `Queryable`.
- `NewPublisher` is a GraphQL input and Diesel `Insertable`.
- `PatchPublisher` is a GraphQL input and Diesel `AsChangeset`.
- Neither input currently contains a package field.
- The persisted model currently contains no package field.
- Ordinary create and update mutations accept those two inputs directly.

The BE-01 specification therefore requires the new persisted field while
preserving both ordinary input shapes so PostgreSQL supplies the `OASIS`
default and ordinary mutations cannot alter a package.

### 6.2 Public GraphQL boundary

`Publisher` fields are explicitly implemented in
`thoth-api/src/graphql/model.rs`. Existing public publisher queries are
anonymous-readable and expose no package or capability field.

`ThothPackage` and `PublisherCapability` must implement or derive the
repository-standard Juniper GraphQL enum representation with the exact stable
ADR-0001 codes. Rust, serde, and GraphQL use the same screaming-snake-case
semantic codes without aliases, lowercase values, `OTHER`, wildcards, or
fallbacks.

BE-01 must not add a public field, query, filter, report, protected
configuration surface, or mutation. If an unreferenced enum does not enter
Juniper's generated SDL, BE-01 records that reachability result and does not add
an artificial public reference. SDL reachability does not make the mandatory
type-level GraphQL representation optional.

### 6.3 Authorization boundary

The ordinary publisher update path uses backend policy and accepts
`PatchPublisher`. Keeping the package absent from that input prevents both
publisher-scoped users and superusers from changing the package through the
ordinary mutation.

BE-01 introduces no new authorization surface. The specification nevertheless
requires schema-validation and non-exposure tests.

### 6.4 Publisher history

Publisher update history records a JSONB value containing the serialized
persisted publisher snapshot. No current publisher-history read path inspected
deserializes every old snapshot into the latest `Publisher` struct.

The specification requires old snapshots to remain readable without a history
backfill, permits new snapshots to include the package where serialization
naturally does so, and blocks implementation if an unrelated history migration
would be required.

### 6.5 Generated schema control

`thoth-api/src/schema.rs` is a generated/derived Diesel schema. Root
`diesel.toml` currently points to `src/schema.rs`, not explicitly to
`thoth-api/src/schema.rs`. Repository controls already assign resolution of
CG-12 to the shared blocked task `THOTH-DB-CTRL-01`. The corrected specification
requires BE-01 to consume and verify that independently approved, merged
procedure and prohibits establishing, redefining, or repairing it inside
BE-01.

## 7. Implementation decisions

Decisions established within the approved design:

1. BE-01 is an additive, initially inactive database and Rust-model foundation.
2. Package codes are exactly `OASIS`, `OBELISK`, `SPHINX`, and `PYRAMID`.
3. Capability codes are exactly the six values approved by ADR-0001.
4. One code-owned exhaustive mapping implements all 24 package/capability
   pairs.
5. PostgreSQL owns the non-null `OASIS` default for existing and new
   publishers.
6. `NewPublisher` and `PatchPublisher` expose no package field.
7. BE-01 exposes no package or capability query, field, report, filter, or
   mutation.
8. Package and platform configuration remain independent.
9. BE-03 owns protected reads and the dedicated superuser mutation.
10. MIG-01 owns the later approved production package mapping and backfill.
11. Approved operational rollback always retains
    `publisher.subscription_package`, `thoth_package`, stored package values,
    required domain types, and the canonical mapping foundation.
12. Specification approval does not make BE-01 implementation-ready.
13. `THOTH-DB-CTRL-01` must be independently approved and merged before BE-01
    moves from `BLOCKED` to `READY`, its implementation branch is created, or
    any implementation edit occurs.
14. The exact BE-01 base is recorded only after the shared control passes, when
    the branch is created from then-current verified `develop`.
15. The repository's existing `cargo run migrate --revert` command intentionally
    tests reversal of the complete migration chain on a disposable database;
    it is not evidence of an isolated BE-01-only revert.
16. Representative populated-database validation proves forward-migration
    preservation without running the full-history revert against its fixtures.
17. No additional migration command, test-only Rust harness, direct `down.sql`
    execution, or implementation mechanism is required by this clarification.
18. Execution of the BE-01 down migration is required only as disposable
    complete-chain reversibility evidence and is not authorized as normal
    post-merge operational rollback.
19. Operational defects use a reviewed forward repair; mapping-code reversal or
    change requires a separately reviewed release.
20. Removing the package foundation requires an approved ADR change and a
    separate authorized migration and data-preservation task.
21. BE-01 is HIGH risk because populated-table migration and package
    entitlements each independently trigger HIGH under repository policy.
22. The future implementation requires high or maximum reasoning, independent
    cross-model review, empty and populated migration evidence, failure-path and
    authorization tests, rollout/rollback controls, and explicit CTO merge
    approval.
23. `ThothPackage` and `PublisherCapability` must implement the
    repository-standard Juniper GraphQL enum representation with the exact
    stable ADR-0001 codes.
24. Generated-SDL omission caused solely by an enum being unreferenced is
    acceptable and recorded; it does not weaken the mandatory type-level
    GraphQL contract or authorize public exposure.

Deviation from the specification: NONE.

## 8. BE-01 and BE-03 boundary

BE-01 owns:

- `thoth_package` storage;
- `publisher.subscription_package NOT NULL DEFAULT OASIS`;
- the persisted Rust package field;
- closed package and capability enums;
- the exhaustive capability mapping;
- migration, history, compatibility, and non-exposure evidence.

BE-03 retains:

- protected package and effective-capability reads;
- publisher-owner and superuser read authorization;
- staff reporting;
- the dedicated superuser package mutation;
- service-configuration audit and concurrency behaviour.

The specification explicitly prohibits adding a public field merely to expose
the enums before BE-03.

## 9. CG-12 and CG-13 treatment

### THOTH-DB-CTRL-01 / CG-12

`THOTH-DB-CTRL-01` is the existing shared repository task for establishing the
Diesel generation procedure. It remains `BLOCKED` and is now an explicit BE-01
dependency.

The shared task must be independently approved and merged before BE-01 becomes
`READY`, before `feature/publisher-services/be-01` is created, or before any
migration, `schema.rs`, model, test, or other implementation edit.

After the shared control passes, the BE-01 implementing agent must consume and
verify the merged repository-authoritative procedure, including its exact
source commit, working directory, commands, root `diesel.toml` relationship,
baseline diff, canonical `thoth-api/src/schema.rs` result, and absence of
unrelated generated changes. It must not independently establish or redefine
the shared procedure.

Failure returns:

```text
BLOCKED - CG-12 SCHEMA GENERATION CONTROL
```

BE-01 cannot silently repair `diesel.toml`, `Makefile`, schema tooling, or the
shared procedure. Any such correction requires a separate shared-control task.

### CG-13

BE-01 implementation and this specification do not authorize production
migration or deployment. A later operational gate must verify runtime and
migration ownership, exact commands, approver, rollback, restore evidence where
required, and explicit CTO authorization.

## 10. Database and migration effects

Migration added: NO.

Schema effect: NONE.

Existing-data effect: NONE.

Locking/downtime: NONE.

Empty database result: not applicable to this documentation-only task.

Populated database result: not applicable to this documentation-only task.

Rollback/forward repair: revert this documentation PR if its control state must
be withdrawn; no data repair is involved.

Idempotency: not applicable; no runtime or database operation occurs.

The approved future BE-01 specification requires a disposable complete-chain
apply/revert/reapply smoke test and a separate representative populated-database
forward-migration preservation test. The former demonstrates that every
committed down migration, including BE-01's, executes as part of the full
reversible chain. The latter does not run the full-history revert against its
fixtures. PostgreSQL-version locking, rewrite analysis, and operational
assessment remain required. Approved operational rollback retains
the package schema, stored values, domain types, canonical mapping foundation,
package history, and canonical Metrics history.

## 11. API and compatibility effects

GraphQL/API changes: NONE.

Generated schema/client updates: NONE.

Backwards compatibility: unchanged.

Deprecations: NONE.

Cross-repository dependencies introduced by this PR: NONE.

The specification requires BE-01 to inspect generated GraphQL SDL, the internal
Rust client, `thoth-app` generated-client impact, dissemination, exports, OAI,
and Metrics, with no unexplained downstream change.

## 12. Authorization and security

Authorization paths changed: NONE.

Roles/scopes involved: NONE.

Negative authorization tests: not applicable to this documentation-only task.

Secret or personal-data handling: no production services, secrets, credentials,
personal source data, or sensitive object URLs were accessed or recorded.

Security limitations: future BE-01 must prove non-exposure and ordinary-input
schema rejection; BE-03 owns protected reads and mutation authorization.

## 13. Tracker, README, and changelog state

Tracker:

```text
BE-01 Publisher package model
Status: BLOCKED
Verified base / PR target:
exact base recorded after THOTH-DB-CTRL-01 passes;
then-current develop / develop
Blocking dependencies:
approved BE-01 specification merged;
THOTH-DB-CTRL-01 independently approved and merged;
separate BLOCKED -> READY control update
Acceptance:
APPROVED SPECIFICATION
IMPLEMENTATION BLOCKED ON THOTH-DB-CTRL-01
IMPLEMENTATION NOT STARTED
```

The row links both this specification and PR #774. BE-02 and all later
Publisher Services, licensing, migration, app, dissemination, OAI, and
operational tasks remain blocked.

README:

```text
CONTROL FOUNDATION CLOSED
BE-01 SPECIFICATION APPROVED AFTER PR #774 MERGES
BE-01 IMPLEMENTATION BLOCKED ON SHARED DIESEL CONTROL
ALL OTHER IMPLEMENTATION REMAINS GATED
```

It records that BE-01 adds only the inactive package/capability foundation,
BE-03 retains protected reads and mutation, `THOTH-DB-CTRL-01` must resolve
CG-12 before BE-01 becomes `READY`, ADR-01 and final platform inventory remain
unresolved, and this PR has no runtime effect.

Changelog:

```text
[774] - Approve the bounded BE-01 publisher package model implementation
specification, defining the non-null OASIS default, exhaustive package
capabilities, migration evidence, protected GraphQL boundary, and inactive
rollout controls
```

## 14. Tests and checks

### 14.1 Preconditions

Commands and authenticated read-only equivalents:

```text
git fetch origin --prune
git status --short --branch
git rev-parse develop origin/develop
gh api repos/thoth-pub/thoth/git/ref/heads/develop
gh api repos/thoth-pub/thoth/pulls/773
git for-each-ref ... feature/publisher-services/be-01...
GitHub pull-request inventory search for BE-01
repository and issue #765 searches for an approved BE-01 specification
```

Result:

```text
PASS
worktree clean
local develop: 4bd95587809637e1b3a03b8d5cdfed877779aecc
origin/develop: 4bd95587809637e1b3a03b8d5cdfed877779aecc
live develop: 4bd95587809637e1b3a03b8d5cdfed877779aecc
PR #773: merged at 4bd95587809637e1b3a03b8d5cdfed877779aecc
specification branch: absent locally and remotely before creation
BE-01 implementation branch/PR: absent
approved BE-01 specification: absent before this task
```

### 14.2 Specification validation

Command:

```text
git diff --check
```

Result:

```text
PASS for the specification-only working diff
```

Additional checks confirmed all 17 required template sections, existing
relative link targets, the named reviewer, and no unresolved template token or
unknown required field.

### 14.3 Pre-report cumulative validation

Command:

```text
git diff --check \
  4bd95587809637e1b3a03b8d5cdfed877779aecc...HEAD
```

Result:

```text
PASS after commits 1 and 2
```

Command:

```text
git diff --name-only \
  4bd95587809637e1b3a03b8d5cdfed877779aecc...HEAD
```

Result:

```text
CHANGELOG.md
docs/engineering/ai-delivery/tasks/BE-01.md
docs/publisher-services/README.md
docs/publisher-services/task-status.md
```

Command:

```text
git log --oneline \
  4bd95587809637e1b3a03b8d5cdfed877779aecc..HEAD
```

Result:

```text
f7c06b7e docs: register BE-01 implementation readiness
b325eaa1 docs: specify BE-01 publisher package model
```

Commit scope checks:

```text
b325eaa1: exactly docs/engineering/ai-delivery/tasks/BE-01.md
f7c06b7e: exactly CHANGELOG.md, docs/publisher-services/README.md,
          docs/publisher-services/task-status.md
```

### 14.4 Documentation classifier before the report commit

Command:

```text
python3 .github/scripts/classify_ci_changes.py --paths \
  CHANGELOG.md \
  docs/engineering/ai-delivery/tasks/BE-01.md \
  docs/publisher-services/README.md \
  docs/publisher-services/task-status.md
```

Result:

```json
{"docs_only":"true","run_build":"false","run_docker":"false","run_migrations":"false"}
```

### 14.5 First corrected cumulative validation

The four-commit/five-path validation was recorded for historical head
`537656cb2187ab91eec30fbbd64c89e495b3a4e9` in its superseding immutable
evidence comment.

Those historical checks covered:

- whitespace;
- exactly four ordered commits;
- exactly five cumulative paths;
- `1 / 3 / 1 / 4` commit path scopes;
- required specification and control terms;
- explicit `THOTH-DB-CTRL-01` dependency and `BLOCKED` state;
- exactly one `Changed` heading within `Unreleased`;
- exactly one PR #774 changelog entry;
- no unresolved specification placeholders;
- the five-path documentation classifier.

### 14.6 Rollback-evidence correction validation

The exact five-commit/five-path commands and outputs were recorded after the
bounded two-file rollback-evidence correction in its superseding immutable
top-level PR evidence comment.

The final checks cover:

- whitespace;
- exactly five ordered commits;
- exactly five cumulative paths;
- `1 / 3 / 1 / 4 / 2` commit path scopes;
- corrective scope exactly the BE-01 specification and this report;
- unchanged migration commands;
- separate complete-chain rollback and populated forward-preservation evidence;
- no new Rust, SQL, migration, schema, tooling, or workflow path;
- exactly one `Changed` heading within `Unreleased`;
- exactly one PR #774 changelog entry;
- no unresolved specification placeholders;
- the five-path documentation classifier.

### 14.7 ADR-0001 rollback-alignment correction validation

The exact six-commit/five-path commands and outputs were recorded after the
bounded two-file ADR-0001 rollback-alignment correction in its superseding
immutable top-level PR evidence comment.

The final checks cover:

- whitespace;
- exactly six ordered commits;
- exactly five cumulative paths;
- `1 / 3 / 1 / 4 / 2 / 2` commit path scopes;
- corrective scope exactly the BE-01 specification and this report;
- consistency with ADR-0001's retained-foundation rollback requirements;
- separate disposable migration-reversibility and operational-rollback
  evidence;
- no ADR, Rust, SQL, migration, schema, tooling, workflow, or runtime change;
- exactly one `Changed` heading within `Unreleased`;
- exactly one PR #774 changelog entry;
- no unresolved specification placeholders;
- the five-path documentation classifier.

### 14.8 HIGH-risk and GraphQL-enum correction validation

The exact final seven-commit/five-path commands and outputs cannot be recorded
inside the correction commit that they validate. They are run immediately
after the bounded three-file correction commit, then recorded in a new
superseding immutable top-level PR evidence comment without creating an eighth
commit.

The final checks cover:

- whitespace;
- exactly seven ordered commits;
- exactly five cumulative paths;
- `1 / 3 / 1 / 4 / 2 / 2 / 3` commit path scopes;
- corrective scope exactly the BE-01 specification, this report, and the BE-01
  tracker row;
- HIGH risk everywhere authoritative for BE-01 implementation;
- populated-table migration and package-entitlement HIGH-risk triggers;
- high or maximum reasoning, independent cross-model review, and explicit CTO
  controls;
- mandatory `ThothPackage` and `PublisherCapability` GraphQL enum
  representations;
- exact stable Rust, serde, and GraphQL codes;
- preserved non-exposure and generated-SDL reachability boundaries;
- no ADR, Rust, SQL, migration, schema, GraphQL runtime, tooling, or workflow
  change;
- exactly one `Changed` heading within `Unreleased`;
- exactly one PR #774 changelog entry;
- no unresolved specification placeholders;
- the five-path documentation classifier.

## 15. Manual verification

Environment: local clean checkout of `thoth-pub/thoth` plus authenticated
read-only GitHub and connected Google Drive metadata/content access.

Steps:

1. verified exact local, remote-tracking, and live base;
2. verified PR #773 merge state;
3. verified branch, specification, and implementation absence;
4. verified private design revision `3`;
5. reconciled repository controls, ADRs, programme documents, and current
   implementation surfaces;
6. inspected each scoped diff and commit boundary;
7. verified issue #765 remained read-only.

Observed result: the approved BE-01 specification fits the existing model and
GraphQL boundary, but independent review correctly identified that shared
Diesel control resolution cannot occur inside BE-01. The corrected records keep
implementation blocked on `THOTH-DB-CTRL-01`.

The post-ready review at
`537656cb2187ab91eec30fbbd64c89e495b3a4e9` then correctly identified an
ambiguity in the migration-evidence wording. No additional command or
implementation mechanism was required. The bounded correction distinguishes
full-chain rollback smoke evidence from populated forward-migration
preservation.

The post-ready review at
`85a83d857b56873f95a0d2011ac06ad0d61319ed` then correctly identified that the
specification's destructive operational rollback conflicted with ADR-0001.
No ADR change was approved or required. The bounded correction retains
disposable down-migration reversibility testing while making operational
rollback retain the package schema, stored values, domain types, mapping
foundation, package history, and canonical Metrics history.

The fresh post-ready review at
`9b68bc3d4008e5aba1c243c523e4c5b2d36d308e` then correctly identified two
remaining authoritative-control conflicts. BE-01 must be `HIGH` risk because
it migrates a populated publisher table and defines package entitlements, and
both package-domain enums must have mandatory standard Juniper GraphQL enum
representations even though they are not made reachable in the public schema
by this task. The bounded correction changes documentation only; it neither
changes the approved architecture nor authorizes implementation.

Evidence: PR #774, the historical head-bound evidence comments, post-ready P1
threads `PRRT_kwDODkn0bc6U18En`, `PRRT_kwDODkn0bc6U2xH6`,
`PRRT_kwDODkn0bc6U3SZ9`, and `PRRT_kwDODkn0bc6U3SaA`, review
`PRR_kwDODkn0bc8AAAABHsstmA`, and the new superseding corrected-head evidence
comment.

## 16. CI

Historical CI at reviewed head
`6d4067d4ee196976e9d06074d3bd790cd3b0096a`: PASSING, but superseded by the
correction.

Historical CI at post-ready reviewed head
`537656cb2187ab91eec30fbbd64c89e495b3a4e9`: PASSING, but its approval and CTO
authorization are superseded by the new correction.

Historical CI at post-ready reviewed head
`85a83d857b56873f95a0d2011ac06ad0d61319ed`: PASSING, but its independent
approval and CTO authorization are historical after the ADR-alignment
correction.

Historical CI at post-ready reviewed head
`9b68bc3d4008e5aba1c243c523e4c5b2d36d308e`: PASSING, but its independent
approval and CTO authorization are historical after the HIGH-risk and
GraphQL-enum correction.

New corrected-head CI status inside this correction commit: not yet available
by design. It is recorded in the new superseding immutable evidence comment
after the commit is pushed.

Required exact-head checks:

- `build-test-and-check`: classifier succeeds; build, test, lint, and format
  jobs skip;
- `run-migrations`: classifier succeeds; migration job skips;
- `publish-to-dockerhub`: classifier succeeds; Docker build/push job skips;
- `check-changelog`: succeeds.

Every skipped heavy job must have an empty or absent step array. Exact workflow
run and job IDs, conclusions, step-array evidence, final head, and unresolved
thread count are recorded in the immutable top-level PR evidence comment.

No workflow is manually dispatched.

## 17. Rollout and rollback

Initial state after merge: the approved BE-01 implementation specification
becomes repository-authoritative, while BE-01 remains `BLOCKED` on
`THOTH-DB-CTRL-01`. No application or database behaviour changes.

Activation required: after this PR is independently approved and merged,
`THOTH-DB-CTRL-01` must be independently approved and merged. A separate
control update may then mark BE-01 `READY`; only after that transition may the
implementation branch be created from and record the then-current verified
`develop`.

Feature flag/configuration: none.

Migration sequence: none in this PR.

Rollback/disable procedure: revert PR #774 through a separately reviewed
documentation correction if the approved control state must be withdrawn. No
runtime disable or data repair is required.

For the future BE-01 implementation, disposable complete-chain reversal remains
required migration-reversibility evidence only. Approved operational rollback
retains the package column, PostgreSQL enum, stored values, required domain
types, canonical mapping foundation, package history, and canonical Metrics
history; it leaves the additive foundation inactive and uses reviewed forward
repair for defects. Foundation removal requires an approved ADR change and a
separately authorized migration and data-preservation task.

Monitoring required: repository PR/CI and review evidence only.

## 18. Known limitations and deferred work

- The report cannot contain its own commit SHA or post-push exact-head CI;
  those are finalized in the immutable PR evidence comment.
- The repository has no dedicated single-migration rollback command. The
  complete-chain disposable rollback smoke test is not evidence of isolated
  BE-01 rollback preservation.
- Representative populated-database validation covers forward-migration
  preservation without applying the full-history revert to its fixtures.
- The tested down migration is not authorized as normal operational rollback;
  operational rollback retains the package foundation and canonical mapping.
- Foundation removal requires an approved ADR change and a separately scoped,
  independently reviewed, CTO-authorized migration and data-preservation task.
- `THOTH-DB-CTRL-01` remains blocked and must independently establish the
  shared Diesel procedure before BE-01 can become `READY`.
- BE-01 may verify but may not independently establish or redefine that merged
  procedure.
- BE-01 implementation is `HIGH` risk. It requires high or maximum
  implementation and review reasoning, an independent cross-model reviewer,
  the complete HIGH-risk evidence set, and explicit CTO approval before the
  implementation PR may merge.
- `ThothPackage` and `PublisherCapability` must expose exact, closed,
  repo-standard Juniper GraphQL enum representations. BE-01 does not add a
  public field solely to make those enum types reachable in generated SDL.
- CG-13 runtime and production migration ownership remains open.
- ADR-01 and final distribution-platform inventory remain unresolved.
- BE-01 implementation has not started.
- BE-03 protected reads and mutation remain deferred.
- No OAI-PMH, Metrics, licensing, dissemination, UI, deployment, release, or
  production work is authorized.

## 19. Unresolved issues

NONE within BE-01-SPEC scope.

The explicitly deferred programme tasks and CG-12/CG-13 gates are controlled
dependencies, not unresolved defects in this documentation PR.

## 20. Agent self-assessment

The implementing agent does not approve this task.

Suggested independent review focus:

- completeness and internal consistency of all BE-01 requirements;
- exact BE-01/BE-03 and package/platform boundaries;
- migration, history, rollback, and CG-12/CG-13 controls;
- separation of complete-chain rollback evidence from populated forward
  preservation;
- consistency with ADR-0001's retained-foundation operational rollback;
- consistency of `HIGH` risk across the specification, report, and tracker;
- completeness of the required HIGH-risk implementation and approval controls;
- exact `ThothPackage` and `PublisherCapability` Rust, serde, and GraphQL enum
  codes without aliases or fallback values;
- preservation of the GraphQL non-exposure and SDL-reachability boundary;
- tracker accuracy without unlocking later work;
- exact seven-commit/five-path evidence, the three-file corrective scope, and
  exact-head skipped-job behaviour.
