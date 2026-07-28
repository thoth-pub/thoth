# ADR-0001-APPROVAL - Approve the Publisher Package Capability Model

Status: APPROVED
Programme: Shared Publisher Services / Thoth Metrics foundation
Repository: thoth-pub/thoth
Workflow: STANDARD
Base branch: develop
Verified base commit: `bafd4cbf752f9d6153036fc7f47115220fed3fbd`
PR target: develop
Programme integration branch: None
Risk: MEDIUM
Owner: CTO
Approved by: Javi, CTO
Approval date: 2026-07-28
Target branch name: `feature/engineering/adr-0001-approval`

Dependencies:

- the ADR-0001 proposal and package-capability appendix introduced through the
  merged engineering-control foundation;
- the CTO decisions recorded in this specification;
- merged CI-DOCS-01 pull request
  [#771](https://github.com/thoth-pub/thoth/pull/771) at merge commit
  `bafd4cbf752f9d6153036fc7f47115220fed3fbd`;
- a fresh independent reviewer who did not implement this task;
- explicit CTO authorization before merge.

Recommended execution:

- Implementing agent/model: Codex / GPT-5
- Implementation reasoning: High
- Independent reviewer/model: ChatGPT / GPT-5.6 Thinking
- Independent review reasoning: High

## 1. Objective

Record the CTO's approval of ADR-0001 and its final normative package-capability
matrix, reconcile the bounded Publisher Services, Metrics and engineering-control
records, and perform the first controlled documentation-only CI observation
required by CI-DOCS-01.

This task records an approved architectural decision. It does not implement
package storage, capability enforcement, metrics collection, GraphQL fields,
OAI-PMH gating or any runtime behaviour.

## 2. Background and authority

Authoritative sources, in precedence order:

1. `thoth-pub/thoth` branch `develop` at
   `bafd4cbf752f9d6153036fc7f47115220fed3fbd`;
2. the CTO decisions in Section 3 of this approved specification;
3. the proposed ADR-0001 and package-capability appendix;
4. the merged CI-DOCS-01 specification and implementation at the approved base;
5. the engineering-control, Publisher Services and Metrics repository records;
6. repository agent instructions and delivery controls.

Verified baseline:

- live `origin/develop` and local `develop` both point to the exact approved
  base;
- the working tree is clean before branching;
- no local or remote
  `feature/engineering/adr-0001-approval` branch exists before branching;
- ADR-0001 and its matrix are `PROPOSED`;
- ADR-0002 is `APPROVED`;
- Publisher Services `ADR-01` and the final distribution-platform inventory
  remain unresolved;
- Publisher Services `BE-01` is `BLOCKED`;
- Metrics `MET-CTRL-01` is `CHANGES REQUIRED`;
- every Metrics work package remains `BLOCKED`;
- CI-DOCS-01 is merged, but its controlled documentation-only, mixed-source and
  next-three-PR observations remain outstanding.

## 3. Approved CTO decisions

The following matrix is final and normative:

| Package | OAI_PMH | METRICS_COLLECT | METRICS_IMPORT | METRICS_DASHBOARD | METRICS_WIDGET | METRICS_OPERAS_EXPORT |
|---|---:|---:|---:|---:|---:|---:|
| OASIS | No | No | No | No | No | No |
| OBELISK | Yes | Yes | No | No | No | No |
| SPHINX | Yes | Yes | Yes | Yes | Yes | Yes |
| PYRAMID | Yes | Yes | Yes | Yes | Yes | Yes |

Approved decisions:

1. OASIS has no `METRICS_COLLECT` capability as an independent
   package-entitlement decision. At the time of approval, Thoth has no managed
   OASIS usage-data source because it does not operationally distribute OASIS
   files. That operational context does not create a package-to-platform rule.
2. OBELISK permits background collection when a valid source and operational
   configuration exist.
3. Metrics collected for OBELISK remain private:
   - no publisher import;
   - no dashboard serving;
   - no widget serving;
   - no OPERAS export.
4. OBELISK collection is operationally non-blocking:
   - missing source configuration must not block unrelated operations;
   - source outages must not block unrelated operations;
   - retries or reconciliation must not block distribution or metadata
     workflows;
   - collection failure must not block package changes;
   - collection must not become a prerequisite for unrelated publisher
     services.
5. Non-blocking does not mean unconfigured collection:
   - a source account, credentials and source-specific configuration are still
     required;
   - the system must not fabricate data or treat missing data as zero.
6. SPHINX has all initial metrics capabilities.
7. PYRAMID includes all SPHINX metrics capabilities.
8. Retained canonical history becomes available when a publisher upgrades to a
   package with the relevant serving capability.
9. An upgrade must not automatically create historical OPERAS exports.
10. Historical OPERAS export requires a separately scoped, reviewed and
    explicitly activated backfill.
11. Downgrades retain canonical metrics.
12. Downgrades stop only import, serving, collection or export behaviour whose
    capability is absent from the resulting package.
13. Package changes never modify distribution-platform assignments.
14. ADR-0001 does not disable or remove OASIS distribution assignments, prevent
    superuser platform configuration, define dissemination eligibility, create
    a distribution capability or change distribution-job behaviour.
15. Metrics collection must not infer entitlement from a distribution-platform
    assignment or remote location.

Approved by: Javi, CTO. Approval date: 2026-07-28.

### 3.1 Independent-review correction

Independent review of draft PR [#772](https://github.com/thoth-pub/thoth/pull/772)
at reviewed head `4af692490875c66a9a0c7fb32354f10f136889e6` returned:

```text
Decision: CHANGES REQUIRED
P0: none
P1: one - decouple OASIS metrics entitlement from distribution
P2: one - make package-change behaviour capability-based
```

The correction preserves the normative matrix byte-for-byte. It clarifies:

- OASIS metrics entitlement is independent from distribution configuration and
  dissemination behaviour;
- the absence of a managed OASIS metrics source is current operational context,
  not a permanent distribution invariant created by ADR-0001;
- any permanent rule prohibiting OASIS distribution requires a separately
  approved Publisher Services decision through ADR-01 or another
  cross-programme ADR;
- every package change is evaluated from the resulting package's capabilities:
  - `PYRAMID -> SPHINX` removes no initial capability;
  - `SPHINX` or `PYRAMID -> OBELISK` retains `OAI_PMH` and configured private
    collection while denying import, dashboard, widget and OPERAS export;
  - any package `-> OASIS` denies all six initial capabilities and stops
    Thoth-managed collection;
- every downgrade retains canonical history, leaves distribution-platform
  assignments unchanged and rechecks the relevant capability at the final
  boundary.

The correction may modify exactly:

```text
docs/engineering/ai-delivery/tasks/ADR-0001-APPROVAL.md
docs/engineering/ai-delivery/implementation-reports/ADR-0001-APPROVAL-implementation-report.md
docs/engineering/decisions/ADR-0001-publisher-package-capability-model.md
docs/engineering/decisions/package-capability-matrix.md
docs/publisher-services/decisions.md
docs/metrics/decisions.md
```

It requires one new bounded commit,
`docs: clarify package distribution and downgrade semantics`, fresh exact-head
CI, a new immutable observation comment, fresh independent exact-head review and
explicit CTO authorization before merge. The existing three commits and
historical CI observation comment must not be amended, rewritten or replaced.

## 4. Explicit scope

The task must:

1. Commit this approved task specification before any other change.
2. Push the specification-only commit and open a draft pull request targeting
   `develop`.
3. Use the actual draft pull-request number, without a placeholder, in the ADR,
   decision register, changelog, approval records and implementation report.
4. Change ADR-0001 from `PROPOSED` to `APPROVED`, preserve its code-owned
   exhaustive mapping, Thoth ownership, stable GraphQL capability codes, absence
   of database capability rows, absence of bespoke publisher overrides and
   entitlement/configuration separation, and record the final approved matrix
   and approval metadata.
5. Change the package-capability appendix to `APPROVED`, make its matrix
   identical to the ADR, replace the proposed collection rationale, update
   upgrade/downgrade rules, record the OBELISK non-blocking invariant and valid
   source requirement, complete every CTO checklist item, and record the
   approver and approval date.
6. Reconcile the engineering decision register and only the ADR-0001 portion of
   control gap CG-06.
7. Reconcile Publisher Services records so ADR-0001 is approved, `BE-01` no
   longer lists it as unresolved, and `BE-01` remains blocked by its own missing
   approved bounded specification. Preserve the unresolved ADR-01 and
   distribution-platform inventory gates and do not mark any implementation
   task ready.
8. Reconcile Metrics records so ADR-0001 is approved, the final matrix is
   accurate, OASIS collection is excluded, OBELISK collection is private and
   non-blocking, `MET-CTRL-01` remains `CHANGES REQUIRED`, and WP1 and later work
   remain blocked by their remaining control, Diesel and repository-readiness
   dependencies.
9. Add one concise `Unreleased` changelog entry describing the approval and the
   final OASIS/OBELISK distinction without claiming runtime implementation.
10. Create the required implementation report and record exact repository,
    validation, no-effect, review and CI observation evidence.
11. Observe the final exact-head GitHub Actions jobs and post one top-level PR
    evidence comment with immutable CI evidence.
12. Keep the pull request draft and hand it off for fresh independent review.

## 5. Approved file allowlist

The complete pull request may change exactly these paths:

```text
CHANGELOG.md
docs/engineering/ai-delivery/tasks/ADR-0001-APPROVAL.md
docs/engineering/ai-delivery/implementation-reports/ADR-0001-APPROVAL-implementation-report.md
docs/engineering/decisions/ADR-0001-publisher-package-capability-model.md
docs/engineering/decisions/package-capability-matrix.md
docs/engineering/decisions/decision-register.md
docs/engineering/repository-map/control-gaps.md
docs/publisher-services/decisions.md
docs/publisher-services/task-status.md
docs/metrics/decisions.md
docs/metrics/task-status.md
```

The actual changed-file set may be a subset of this allowlist. If another
repository file requires modification, stop and report the exact stale reference
as follow-up work rather than expanding scope.

## 6. Non-goals

This task must not:

- modify Rust, SQL, GraphQL, JavaScript, generated code or workflow files;
- create package enums, capability mappings, database columns or migrations;
- implement package storage, capability checks or authorization;
- implement metrics collection, imports, serving or OPERAS export;
- alter OAI-PMH behaviour;
- change distribution-platform assignments;
- create or modify source accounts or credentials;
- change branch protection or rulesets;
- dispatch production or write-capable workflows;
- deploy, release or activate production behaviour;
- access production secrets or services;
- edit GitHub issues #765 or #766;
- mark any Publisher Services or Metrics implementation task ready;
- broaden the approved file scope;
- approve its own work;
- merge the pull request.

## 7. Invariants

The final branch must preserve:

1. Package/capability mapping remains code-owned and exhaustive in Thoth.
2. `ThothPackage` and `PublisherCapability` remain closed Rust enums with stable
   GraphQL codes in the approved target architecture.
3. Capability mappings are not persisted as independent database rows.
4. Bespoke per-publisher capability overrides remain excluded.
5. Product entitlement remains separate from source accounts, credentials,
   source-specific configuration, feature flags, schedules and rollout
   approval.
6. Package changes never alter distribution-platform assignments,
   distribution-job eligibility or dissemination behaviour.
7. OASIS has no initial capability. At approval time, Thoth has no managed OASIS
   usage-data source; that operational fact is not a permanent
   package-to-platform rule.
8. OBELISK private collection is permitted only when configured and is
   operationally non-blocking for unrelated work.
9. Missing or unavailable metric data is never fabricated or treated as zero.
10. Canonical metric history is retained after downgrade.
11. Historical OPERAS export is never an automatic upgrade side effect.
12. ADR-01, the distribution-platform inventory, `MET-CTRL-01`, Diesel schema
    generation and repository-readiness gaps retain their existing independent
    status.
13. No documentation claim treats approval as implementation or readiness.
14. No runtime, migration, API, workflow, production, deployment or release
    effect occurs.
15. OASIS distribution assignments and superuser platform configuration remain
    unaffected; any permanent distribution prohibition requires ADR-01 or
    another approved cross-programme ADR.
16. Metrics collection never infers package entitlement from a distribution
    assignment or remote location.
17. Every package change uses the resulting package's capabilities, retains
    canonical history on downgrade and rechecks the relevant capability at the
    final boundary.

## 8. Required documentation behaviour

### 8.1 ADR-0001

The ADR must:

- show `Status: APPROVED`;
- contain the exact matrix in Section 3;
- remove statements suggesting OASIS permits collection;
- state that OASIS is not entitled to Thoth-managed metrics collection and that,
  under current operations, Thoth has no managed OASIS usage-data source;
- state that this operational context does not define or alter distribution
  assignments, distribution-job eligibility or dissemination behaviour;
- define OBELISK background collection as private, configured and non-blocking;
- distinguish downgrade to OBELISK, where configured private collection may
  continue, from downgrade to OASIS, where managed collection stops;
- preserve canonical history on either downgrade;
- record:

```text
Approved by: Javi, CTO
Approval date: 2026-07-28
Approval PR: #<actual PR number>
```

It must not claim implementation has started or completed.

### 8.2 Package-capability appendix

The appendix must:

- show `Status: APPROVED`;
- contain the exact matrix in Section 3;
- replace the “Collection for every package” proposal with the approved OASIS,
  OBELISK and SPHINX/PYRAMID rationale;
- state the OBELISK non-blocking invariant and valid-source/configuration
  requirement;
- update upgrade and downgrade behaviour;
- mark every CTO approval-checklist item complete;
- record Javi, CTO and 2026-07-28.

### 8.3 Control and programme records

The decision register must show ADR-0001 approved and reference the actual
approval PR. CG-06 must record the ADR-0001 approval while preserving every
unrelated control gap.

Publisher Services and Metrics records must accurately distinguish approval from
implementation readiness and retain all remaining blockers specified in Section
7.

## 9. Acceptance criteria

- [ ] The exact-base, clean-tree and absent-branch preconditions passed before
  branch creation.
- [ ] This task specification is the only file in the first commit.
- [ ] The first commit message is `docs: specify ADR-0001 approval`.
- [ ] A draft pull request targets `develop`.
- [ ] Every changed path is in the approved allowlist and under `docs/**` or is
  exactly `CHANGELOG.md`.
- [ ] No runtime or workflow file changed.
- [ ] ADR-0001 and the matrix appendix show `APPROVED`.
- [ ] The ADR, appendix and programme summaries contain the identical normative
  matrix.
- [ ] OASIS never has `METRICS_COLLECT`.
- [ ] OBELISK has only `OAI_PMH` and `METRICS_COLLECT`.
- [ ] SPHINX and PYRAMID have all six capabilities.
- [ ] OBELISK collection is private, requires valid source configuration and
  credentials, and is non-blocking for unrelated work.
- [ ] Upgrade, historical export and both downgrade paths match the approved CTO
  decisions.
- [ ] `PYRAMID -> SPHINX` preserves all six initial capabilities.
- [ ] A resulting OBELISK package permits only `OAI_PMH` and configured private
  collection; a resulting OASIS package denies all six initial capabilities.
- [ ] Every downgrade retains canonical history, leaves distribution assignments
  unchanged and rechecks the relevant capability at its final boundary.
- [ ] No text makes ADR-0001 define OASIS distribution or dissemination
  eligibility, and collection never infers entitlement from an assignment or
  remote location.
- [ ] Every package-capability approval checklist item is checked.
- [ ] Approver, approval date and actual approval PR are consistent.
- [ ] The ADR's code-owned mapping, ownership, API-code, no-row, no-override and
  entitlement/configuration architecture is preserved.
- [ ] ADR-01 and the distribution-platform inventory remain unresolved.
- [ ] `BE-01` remains `BLOCKED` pending its own approved bounded specification.
- [ ] `MET-CTRL-01` remains `CHANGES REQUIRED`.
- [ ] Metrics WP1 and later work packages remain `BLOCKED`.
- [ ] No document claims implementation is ready, started or complete.
- [ ] The changelog records approval without describing runtime functionality as
  implemented.
- [ ] The implementation report records the exact final head, commits, changed
  files, validation, CI and no-effect assessment.
- [ ] Local documentation validation passes.
- [ ] Exact-head CI satisfies Section 11.
- [ ] One top-level immutable PR evidence comment satisfies Section 11.
- [ ] The pull request remains draft with zero unresolved review threads at
  handoff.
- [ ] A fresh independent reviewer returns `APPROVED` before merge.
- [ ] Explicit CTO authorization is obtained after independent approval and
  before merge.

## 10. Local validation

Run:

```bash
git diff --check bafd4cbf752f9d6153036fc7f47115220fed3fbd...HEAD
```

Verify:

- every changed path is under `docs/**` or exactly `CHANGELOG.md`;
- the changed-file set is a subset of Section 5;
- no runtime or workflow file changed;
- every active ADR-0001 status in the approved scope is consistent;
- matrices are identical in the ADR, appendix and programme summaries;
- OASIS has no capability;
- OBELISK has only `OAI_PMH` and `METRICS_COLLECT`;
- SPHINX and PYRAMID have all six capabilities;
- `PYRAMID -> SPHINX` removes no initial capability;
- transition to OBELISK and transition to OASIS use the resulting capability
  set;
- canonical history and distribution assignments remain unchanged on downgrade;
- in-flight work rechecks the relevant capability at its final boundary;
- no text makes package choice define distribution or dissemination eligibility;
- metrics collection does not infer entitlement from a distribution assignment
  or remote location;
- every checklist item is checked;
- approver, approval date and actual PR number are consistent;
- internal links and paths resolve;
- canonical repository and programme terminology is used;
- no active document claims implementation readiness.

Rust, database, migration, GraphQL and authorization tests are not applicable
because the approved task is documentation-only and changes none of those
surfaces.

## 11. Documentation-only CI observation

This pull request is the first controlled documentation-only observation for
merged task CI-DOCS-01.

At the exact final head, require:

```text
all classifier jobs: success
build: skipped
test: skipped
lint: skipped
format_check: skipped
run_migrations: skipped
build_and_push_staging_docker_image: skipped
check-changelog: success
```

Inspect job steps and prove:

- no Rust build step executed;
- no test step executed;
- no lint step executed;
- no migration build, apply or revert step executed;
- no Docker registry login executed;
- no Docker image build or push executed;
- all six protected contexts reached terminal merge-safe conclusions;
- the additional mandatory Docker workflow context reached a terminal
  conclusion;
- no required context remained pending;
- `check-changelog` remained active.

Post one top-level PR evidence comment containing:

- exact base and final head;
- workflow run IDs;
- all classifier conclusions;
- each protected and mandatory job conclusion;
- confirmation that heavy steps did not execute;
- confirmation that `check-changelog` remained active;
- a statement that this satisfies only the controlled documentation-only
  observation;
- a statement that the mixed-source and next-three-PR observations remain
  outstanding.

Do not mark CI-DOCS-01 operationally complete. Exact final-head run IDs belong in
the immutable PR evidence comment, not in a later evidence-only commit.
If remediation creates a new exact head, post a new immutable top-level evidence
comment and leave every earlier observation comment unchanged.

## 12. Implementation report

Create:

`docs/engineering/ai-delivery/implementation-reports/ADR-0001-APPROVAL-implementation-report.md`

The report must record:

- task and actual draft pull request;
- exact base and final head;
- implementing agent/model and independent-review requirement;
- approved decisions and normative matrix;
- ordered commits and actual changed files;
- exact local validation commands and results;
- exact-head CI status and the evidence-comment link;
- CI-DOCS-01 documentation-only observation status;
- absence of runtime, migration, API, authorization, workflow, production,
  deployment and release effects;
- rollout and rollback;
- all remaining Publisher Services, Metrics, Diesel and repository-readiness
  blockers.

The implementing agent may self-assess risks but may not approve the task.

## 13. Commit structure

Required first commit:

```text
docs: specify ADR-0001 approval
```

After the draft pull request exists and its actual number is known, use bounded
commits such as:

```text
docs: approve publisher package capability model
docs: record ADR-0001 approval evidence
```

Do not amend, squash or rewrite the specification-first commit.

Independent-review remediation adds exactly one further bounded commit:

```text
docs: clarify package distribution and downgrade semantics
```

Do not amend, squash, rebase or rewrite any of the existing three commits.

## 14. Rollout

Initial state after merge:

- ADR-0001 is an approved architectural dependency;
- no package, capability, metrics or OAI-PMH runtime behaviour exists merely
  because the ADR is approved;
- Publisher Services and Metrics implementation remains blocked by the
  remaining task-specific and programme-readiness dependencies;
- CI-DOCS-01 has one controlled documentation-only observation, while its
  mixed-source and next-three-PR observations remain outstanding.

Feature flag/configuration: none.
Staging/preview: none.
Pilot: none.
Production activation: none.

Merge requires a fresh independent `APPROVED` review followed by explicit CTO
authorization. The implementing agent must not merge.

## 15. Rollback

Rollback is a normal revert of this documentation-only approval pull request.
The revert must restore the prior `PROPOSED` decision and tracker state while
preserving unrelated later work.

No database, data, API, authorization, source-account, distribution,
deployment, release or production-state rollback is required.

The CI observation comment is immutable historical evidence and must not be
rewritten as though the observation did not occur.

## 16. Stop conditions

Stop and report `BLOCKED` if:

- `develop` differs from
  `bafd4cbf752f9d6153036fc7f47115220fed3fbd` before branching;
- the working tree is not clean before branching;
- the task branch already exists locally or remotely;
- the approved architecture conflicts with merged repository evidence;
- another repository file must change to maintain consistency;
- implementation requires runtime, migration, API, workflow, issue, deployment,
  release, production or secret access;
- the actual pull-request number is unavailable for approval records;
- an exact-head required context is missing, pending or not merge-safe;
- a heavy Rust, migration or Docker step executes for this documentation-only
  pull request;
- the branch ceases to be a documentation-only diff;
- independent review or explicit CTO merge authorization is unavailable.

## 17. Independent review and merge authorization

A fresh non-implementing reviewer must inspect:

- this approved specification;
- exact base and final head;
- ordered commit sequence and specification-first history;
- complete cumulative diff and allowlist compliance;
- all ADR, matrix, control and programme changes;
- matrix identity and every approved invariant;
- local validation evidence;
- exact-head GitHub Actions run and job-step evidence;
- the CI-DOCS-01 top-level evidence comment;
- remaining blockers and the absence of implementation/readiness claims;
- absence of runtime, migration, workflow, issue, deployment, release and
  production changes;
- unresolved review threads.

The reviewer must return exactly one verdict: `APPROVED`, `CHANGES REQUIRED` or
`BLOCKED`. Approval is permitted only when no unresolved P0 or P1 finding
remains.

The implementing agent may not approve or merge this work. Do not merge without
explicit CTO authorization after independent approval.

## 18. Approval

Approved for implementation by: Javi, CTO

Decision date: 2026-07-28

Notes: This approval authorizes only the bounded documentation and CI-observation
task defined above.
