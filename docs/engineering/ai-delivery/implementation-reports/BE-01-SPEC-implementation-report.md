# BE-01-SPEC Implementation Report

## 1. Repository state

Repository: `thoth-pub/thoth`

Programme: Publisher Services and Distribution Configuration

Task ID: `BE-01-SPEC`

Risk: MEDIUM

Workflow: STANDARD

Base branch: `develop`

Base commit: `4bd95587809637e1b3a03b8d5cdfed877779aecc`

PR target: `develop`

Programme integration branch: None

Task branch: `feature/publisher-services/be-01-spec`

Pre-report head:
`f7c06b7e8c946e29d754e36a44da9419264a0e6c`

Final report commit and exact final head: recorded after this report is
committed in the immutable top-level PR evidence comment, as required by the
approved finalization mechanism.

Pull request: [#774](https://github.com/thoth-pub/thoth/pull/774)

Expected branch deletion after merge: YES

Final programme PR required: NO

Implementing model: Codex / GPT-5

Reasoning level: Medium

Independent reviewer/model: OpenAI ChatGPT / GPT-5.6 Thinking

Review reasoning level: High

## 2. Scope confirmation

Approved specification: the CTO-authorized BE-01-SPEC execution specification
dated 2026-07-29.

Implemented objective: create and register an approved, repository-backed,
bounded implementation specification for BE-01 Publisher package model without
implementing the model.

Out-of-scope changes made: NONE.

The final cumulative PR changes exactly:

```text
CHANGELOG.md
docs/engineering/ai-delivery/implementation-reports/BE-01-SPEC-implementation-report.md
docs/engineering/ai-delivery/tasks/BE-01.md
docs/publisher-services/README.md
docs/publisher-services/task-status.md
```

No other path is authorized or changed.

## 3. Commits

1. `b325eaa1d46a672d69961597cf1c8d8740cf7996` -
   `docs: specify BE-01 publisher package model`
2. `f7c06b7e8c946e29d754e36a44da9419264a0e6c` -
   `docs: register BE-01 implementation readiness`
3. The report commit containing this file is recorded in the immutable
   top-level PR evidence comment after it exists.

No evidence-only fourth commit is permitted.

## 4. Files changed

- `docs/engineering/ai-delivery/tasks/BE-01.md`
  - reason: establish the complete approved BE-01 implementation contract;
  - behavioural effect: none; documentation and future implementation control
    only.
- `CHANGELOG.md`
  - reason: add the required Unreleased Changed entry for PR #774;
  - behavioural effect: none.
- `docs/publisher-services/README.md`
  - reason: record partial programme readiness and the BE-01/BE-03 boundary;
  - behavioural effect: none.
- `docs/publisher-services/task-status.md`
  - reason: make BE-01 `READY` when this specification PR merges while
    preserving all other gates;
  - behavioural effect: none.
- `docs/engineering/ai-delivery/implementation-reports/BE-01-SPEC-implementation-report.md`
  - reason: record scope, evidence, effects, and handoff for this specification
    task;
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

BE-01 may derive GraphQL enum representations but must not add a public field,
query, filter, report, protected configuration surface, or mutation. If an
unreferenced enum does not enter Juniper's generated SDL, BE-01 records that
fact and does not add an artificial public reference.

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
`thoth-api/src/schema.rs`. The specification makes CG-12 discovery the first
implementation precondition and prohibits guessing, hand-copying generated
output, or changing tooling in BE-01 without separate approval.

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
11. A non-OASIS value or later dependency changes the safe rollback from a
    tested down migration to coordinated data-preserving repair.

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

### CG-12

Before the first migration or schema edit, the BE-01 implementing agent must
prove the exact migration and Diesel schema-generation or verification
procedure, working directory, root `diesel.toml` relationship, baseline diff,
and absence of unrelated generated changes.

Failure returns:

```text
BLOCKED - CG-12 SCHEMA GENERATION CONTROL
```

BE-01 cannot silently repair `diesel.toml`, `Makefile`, or schema tooling.

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

The approved future BE-01 specification requires empty and representative
populated disposable-database apply/revert/reapply evidence, PostgreSQL-version
locking and rewrite analysis, and the conditional data-preserving rollback.

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
Status: READY
Verified base / PR target:
latest verified develop after BE-01-SPEC merge / develop
Blocking dependencies:
approved BE-01 specification merged;
CG-12 schema-generation discovery must pass before schema changes
Acceptance:
APPROVED SPECIFICATION - IMPLEMENTATION NOT STARTED
```

The row links both this specification and PR #774. BE-02 and all later
Publisher Services, licensing, migration, app, dissemination, OAI, and
operational tasks remain blocked.

README:

```text
CONTROL FOUNDATION CLOSED
BE-01 SPECIFIED AND READY AFTER PR #774 MERGES
ALL OTHER IMPLEMENTATION REMAINS GATED
```

It records that BE-01 adds only the inactive package/capability foundation,
BE-03 retains protected reads and mutation, CG-12 precedes migration edits,
ADR-01 and final platform inventory remain unresolved, and this PR has no
runtime effect.

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

### 14.5 Final cumulative validation

The exact final three-commit/five-path commands and outputs cannot be recorded
inside the report commit that they validate. They are run immediately after
this report commit, then recorded in the immutable top-level PR evidence
comment without creating a fourth commit.

The final checks cover:

- whitespace;
- exactly three ordered commits;
- exactly five cumulative paths;
- `1 / 3 / 1` commit path scopes;
- required specification and control terms;
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
GraphQL boundary without changing an active implementation path or requiring
an additional specification-task file.

Evidence: PR #774 and its final immutable evidence comment.

## 16. CI

CI status at report authorship: PENDING by design because the report commit is
not yet pushed.

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

Initial state after merge: the approved BE-01 implementation specification and
partial-readiness controls become repository-authoritative. No application or
database behaviour changes.

Activation required: BE-01 implementation requires this PR to be independently
approved and merged, a fresh `develop` base verification, and successful CG-12
schema-generation discovery.

Feature flag/configuration: none.

Migration sequence: none in this PR.

Rollback/disable procedure: revert PR #774 through a separately reviewed
documentation correction if the approved control state must be withdrawn. No
runtime disable or data repair is required.

Monitoring required: repository PR/CI and review evidence only.

## 18. Known limitations and deferred work

- The report cannot contain its own commit SHA or post-push exact-head CI;
  those are finalized in the immutable PR evidence comment.
- CG-12 schema-generation discovery remains the first BE-01 implementation
  precondition.
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
- tracker accuracy without unlocking later work;
- exact three-commit/five-path evidence and exact-head skipped-job behaviour.
