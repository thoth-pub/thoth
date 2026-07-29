# ADR-0001-APPROVAL Implementation Report

## 1. Repository state

Repository: `thoth-pub/thoth`
Workflow: STANDARD
Base branch: `develop`
Base commit: `bafd4cbf752f9d6153036fc7f47115220fed3fbd`
PR target: `develop`
Programme integration branch: None
Task branch: `feature/engineering/adr-0001-approval`
Approval content commit: `55fc33fe4ea5251ab941002ed9480f3b78aceaa7`
Previous reviewed head: `4af692490875c66a9a0c7fb32354f10f136889e6`
Independent-review correction head:
`1124262dd28e0b51f33259be1b70e1396e3bdb1c`
Post-ready automated reviewed head:
`1124262dd28e0b51f33259be1b70e1396e3bdb1c`
Second post-ready automated reviewed head:
`56c4f873c27fa83e6358c1f207cd718cb3dde679`
Pull request: [#772](https://github.com/thoth-pub/thoth/pull/772) (draft)
Expected branch deletion after merge: YES
Final programme PR required: NO
Implementing agent/model: Codex / GPT-5
Reasoning level: High
Independent reviewer/model: ChatGPT / GPT-5.6 Thinking in a fresh,
non-implementing context
Risk: MEDIUM

The exact final branch head necessarily includes the commit containing this
report, so the report cannot contain its own commit SHA. The immutable final
head, exact-head workflow run IDs and final conclusions are recorded in the
top-level PR evidence comment and final implementation handoff. No later
repository commit is covered by that evidence.

Independent review of the previous head returned `CHANGES REQUIRED` with no P0,
one P1 finding on distribution/metrics-entitlement coupling and one P2 finding
on downgrade semantics. The bounded correction commit contains this report, so
its exact SHA, the new final head, revised exact-head CI and new immutable
observation-comment URL are likewise recorded externally after push. This
preserves the required one-commit correction without amending history or
creating an endless evidence-only commit chain.

After the PR was marked ready, the automated review submitted against exact
head `1124262dd28e0b51f33259be1b70e1396e3bdb1c` opened one P1 finding:
the active engineering, Metrics and Publisher Services README entry points still
described ADR-0001 as proposed or as an outstanding approval blocker. The PR
returned to draft before this correction. The prior exact-head independent
approval and CTO merge authorization are now historical and inapplicable to the
new correction head.

A second post-ready automated review against exact head
`56c4f873c27fa83e6358c1f207cd718cb3dde679` opened one P1 because the active
Publisher Services rollout plan still listed `ADR-0001 approval` as outstanding
Stage 0 evidence. The PR returned to draft before this correction. The
independent approval and CTO authorization for that reviewed head are now
historical and inapplicable to the new correction head.

## 2. Scope confirmation

Approved specification:
[`docs/engineering/ai-delivery/tasks/ADR-0001-APPROVAL.md`](../tasks/ADR-0001-APPROVAL.md)

Implemented objective: record the CTO approval of ADR-0001 and the final
publisher package-capability matrix, reconcile only the approved engineering,
Publisher Services and Metrics records, and prepare PR #772 as the first
controlled CI-DOCS-01 documentation-only observation.

The CTO-approved post-ready scope amendment additionally reconciles the three
active README entry points with the normative decision and tracker records while
preserving every genuine programme blocker.

The second CTO-approved post-ready amendment additionally reconciles Stage 0 of
the active Publisher Services rollout control. It moves ADR-0001 approval from
outstanding to achieved evidence while preserving it as a requirement and
preserving every genuine rollout blocker. The rollout plan is an active control
document because it governs the current staged implementation, evidence and
activation gates rather than recording historical review evidence.

Out-of-scope changes made: NONE.

No GitHub issue, branch-protection rule, ruleset, workflow, runtime, migration,
API, source account, deployment, release or production state was changed.

## 3. Approved decisions

Approved by: Javi, CTO

Approval date: 2026-07-28

Approval PR: [#772](https://github.com/thoth-pub/thoth/pull/772)

The final normative matrix is:

| Package | OAI_PMH | METRICS_COLLECT | METRICS_IMPORT | METRICS_DASHBOARD | METRICS_WIDGET | METRICS_OPERAS_EXPORT |
|---|---:|---:|---:|---:|---:|---:|
| OASIS | No | No | No | No | No | No |
| OBELISK | Yes | Yes | No | No | No | No |
| SPHINX | Yes | Yes | Yes | Yes | Yes | Yes |
| PYRAMID | Yes | Yes | Yes | Yes | Yes | Yes |

The recorded decision preserves:

- code-owned exhaustive package/capability mapping in Thoth;
- closed package and capability enums with stable GraphQL codes;
- no independent database capability rows;
- no bespoke per-publisher capability overrides;
- separation between product entitlement and operational configuration;
- no coupling between package changes and distribution-platform assignments.

The final operational decisions are:

- OASIS has no `METRICS_COLLECT` capability as an independent entitlement
  decision; under current operations Thoth has no managed OASIS usage-data
  source because it does not operationally distribute OASIS files;
- that operational context does not disable or remove OASIS distribution
  assignments, prevent superuser platform configuration, define dissemination
  eligibility, create a distribution capability or change distribution-job
  behaviour;
- any permanent prohibition on OASIS distribution requires ADR-01 or another
  separately approved cross-programme ADR;
- metrics collection does not infer entitlement from a distribution assignment
  or remote location;
- OBELISK collection is configured, private and non-blocking for unrelated
  operations;
- missing source configuration or source outages do not become zero and do not
  block distribution, metadata, package changes or unrelated publisher
  services;
- SPHINX and PYRAMID have all six capabilities;
- retained canonical history becomes available after upgrade only through the
  relevant serving capability;
- historical OPERAS export requires a separately scoped, reviewed and activated
  backfill;
- every package change uses the resulting package's capabilities;
- every downgrade retains canonical history, leaves distribution assignments
  unchanged and rechecks the relevant capability at the final boundary.

Corrected package-change semantics:

| Package change | Capability effect |
|---|---|
| `PYRAMID -> SPHINX` | No initial capability is removed; collection, import, dashboard, widget, OAI-PMH and eligible OPERAS export remain permitted subject to normal configuration, authorization and rollout requirements |
| `SPHINX` or `PYRAMID -> OBELISK` | OAI-PMH and configured private collection remain permitted; publisher import, dashboard, widget and OPERAS export are denied |
| Any package `-> OASIS` | All six initial capabilities are denied and Thoth-managed collection stops |

## 4. Commits

- `b81c5eecabf2c1ca9761a5b5651f8ea97cecf18b` -
  `docs: specify ADR-0001 approval`
  - the approved task specification was the only file in the first commit.
- `55fc33fe4ea5251ab941002ed9480f3b78aceaa7` -
  `docs: approve publisher package capability model`
  - the ADR, matrix, changelog and bounded control/programme records were
    reconciled after draft PR #772 supplied the actual approval PR number.
- `4af692490875c66a9a0c7fb32354f10f136889e6` -
  `docs: record ADR-0001 approval evidence`
  - the implementation report at the previous reviewed head.
- `1124262dd28e0b51f33259be1b70e1396e3bdb1c` -
  `docs: clarify package distribution and downgrade semantics`
  - one bounded correction across the six approved documentation files; its
    exact SHA is the post-ready automated reviewed head.
- `56c4f873c27fa83e6358c1f207cd718cb3dde679` -
  `docs: reconcile ADR-0001 programme entry points`
  - one bounded post-ready correction across the five amended documentation
    files; its exact SHA is the second post-ready automated reviewed head.
- `docs: reconcile ADR-0001 rollout gate`
  - one bounded second post-ready correction across the three amended
    documentation files; its exact SHA and the resulting final head are recorded
    externally because the commit contains this report.

No commit was amended, squashed or rewritten.

## 5. Files changed

The complete pull request changes exactly the following fifteen approved paths:

- `CHANGELOG.md`
  - records the architectural approval and the final OASIS/OBELISK collection
    distinction without claiming implementation.
- `docs/engineering/ai-delivery/tasks/ADR-0001-APPROVAL.md`
  - records the approved specification, task identity, scope, invariants,
    acceptance criteria, validation, CI observation, review and merge gates.
- `docs/engineering/ai-delivery/implementation-reports/ADR-0001-APPROVAL-implementation-report.md`
  - records implementation and validation evidence.
- `docs/engineering/decisions/ADR-0001-publisher-package-capability-model.md`
  - records `APPROVED`, the final matrix, operational decisions and approval
    metadata while preserving the approved architecture.
- `docs/engineering/decisions/package-capability-matrix.md`
  - records the approved matrix, OASIS/OBELISK rationale, upgrade/downgrade
    semantics and completed CTO checklist.
- `docs/engineering/decisions/decision-register.md`
  - records ADR-0001 approval and PR #772.
- `docs/engineering/repository-map/control-gaps.md`
  - resolves only the shared-ADR CG-06 gate while preserving all unrelated
    control gaps and implementation blockers.
- `docs/engineering/README.md`
  - records both cross-programme ADRs as approved while distinguishing the
    pending PR #772 merge and preserving genuine implementation blockers.
- `docs/publisher-services/decisions.md`
  - summarizes the approved architecture and exact final matrix.
- `docs/publisher-services/task-status.md`
  - removes ADR-0001 as an unresolved dependency while retaining `BE-01` and all
    implementation work as blocked.
- `docs/publisher-services/README.md`
  - records the approved ADR-0001 control pending merge and preserves ADR-01,
    inventory, task-specification and branch-readiness blockers.
- `docs/publisher-services/rollout-plan.md`
  - records ADR-0001 approval as achieved Stage 0 evidence, removes only the
    stale approval blocker and preserves every genuine rollout gate.
- `docs/metrics/decisions.md`
  - summarizes the exact final matrix, OASIS exclusion, private/non-blocking
    OBELISK collection and upgrade/downgrade/export rules.
- `docs/metrics/task-status.md`
  - records ADR-0001 approval while retaining `MET-CTRL-01` as
    `CHANGES REQUIRED` and every work package as `BLOCKED`.
- `docs/metrics/README.md`
  - records the approved ADR-0001 control pending merge and preserves
    `MET-CTRL-01`, Sphinx, Diesel, branch, service-role, fixture and OPERAS
    blockers.

No file outside the approved allowlist changed.

The independent-review correction changes exactly:

```text
docs/engineering/ai-delivery/tasks/ADR-0001-APPROVAL.md
docs/engineering/ai-delivery/implementation-reports/ADR-0001-APPROVAL-implementation-report.md
docs/engineering/decisions/ADR-0001-publisher-package-capability-model.md
docs/engineering/decisions/package-capability-matrix.md
docs/publisher-services/decisions.md
docs/metrics/decisions.md
```

The post-ready active-entry-point correction changes exactly:

```text
docs/engineering/ai-delivery/tasks/ADR-0001-APPROVAL.md
docs/engineering/ai-delivery/implementation-reports/ADR-0001-APPROVAL-implementation-report.md
docs/engineering/README.md
docs/metrics/README.md
docs/publisher-services/README.md
```

The second post-ready rollout-gate correction changes exactly:

```text
docs/engineering/ai-delivery/tasks/ADR-0001-APPROVAL.md
docs/engineering/ai-delivery/implementation-reports/ADR-0001-APPROVAL-implementation-report.md
docs/publisher-services/rollout-plan.md
```

## 6. Implementation decisions and deviations

Implementation decisions within the approved specification:

1. The ADR, matrix appendix and both programme decision summaries use a
   byte-identical Markdown matrix so their values can be compared
   deterministically.
2. Resolved ADR-0001 dependencies were removed from tracker rows, while each row
   retained or gained its remaining explicit bounded-specification and
   readiness dependencies.
3. CG-06 is recorded as resolved for shared ADR approval, with dependent
   implementation still gated on PR merge, fresh independent review, approved
   bounded specifications and remaining programme controls.
4. Exact-head CI evidence is kept in one immutable top-level PR comment, avoiding
   evidence-only commits that would create a new unevidenced head.
5. The P1 correction treats OASIS metrics entitlement as independent from
   distribution configuration and current operational source availability.
6. The P2 correction evaluates every package change through the resulting
   capability set rather than applying generic downgrade denials.
7. The post-ready correction treats the three README files as active entry
   points, not historical evidence, so they must agree with the normative
   decision and tracker records without erasing genuine implementation
   blockers.
8. The second post-ready correction treats the Publisher Services rollout plan
   as an active control: approval moves to achieved evidence, but Stage 0 and
   implementation remain blocked by their genuine remaining gates.

Deviations from the approved specification: NONE.

The stale active references found by both post-ready reviews are reconciled
under their CTO-approved five-file and three-file scope amendments. Historical
task specifications, implementation reports and review records remain intact.

## 7. Database and migration effects

Migration added: NO

Schema effect: NONE

Existing-data effect: NONE

Locking/downtime: NONE

Backfill: NONE

Empty/populated database testing: Not applicable to this documentation-only
task.

Rollback/forward repair: normal documentation PR revert; no database action.

## 8. API, authorization and security effects

GraphQL/API changes: NONE

Generated schema/client updates: NONE

Backwards compatibility effect: NONE

Authorization paths changed: NONE

Roles/scopes changed: NONE

Negative authorization tests: Not applicable; no authorization code or contract
changed.

Secret or personal-data handling: no production secrets, credentials, personal
data or sensitive object URLs were accessed or recorded.

## 9. Local tests and checks

### Documentation diff

Command:

```text
git diff --check bafd4cbf752f9d6153036fc7f47115220fed3fbd...HEAD
```

Result:

```text
Exit 0; no output.
```

The command is rerun after the report commit before push.

### Changed-file scope

Command:

```text
git diff --name-only bafd4cbf752f9d6153036fc7f47115220fed3fbd...HEAD
```

Result:

```text
Exactly fifteen paths: CHANGELOG.md and the fourteen approved docs/** paths listed in Section 5.
No runtime or workflow path.
```

### Deterministic matrix comparison

Command:

```text
for each of the ADR, appendix, Publisher Services summary and Metrics summary:
  extract the table from "| Package | OAI_PMH" through "| PYRAMID"
  hash it with shasum -a 256
```

Result:

```text
All four hashes:
e2225da6575b484e79555fc23045c8ac2685d68ab088e831418cea9252a77cf7
```

The rows are OASIS all `No`; OBELISK only `OAI_PMH` and
`METRICS_COLLECT`; SPHINX and PYRAMID all `Yes`.

### Status, checklist and readiness inspection

Commands:

```text
rg -n '^Status: APPROVED$|^Approved by: Javi, CTO$|^Approval date: 2026-07-28$|Approval PR:.*#772' \
  docs/engineering/decisions/ADR-0001-publisher-package-capability-model.md \
  docs/engineering/decisions/package-capability-matrix.md

rg -n 'MET-CTRL-01.*CHANGES REQUIRED' docs/metrics/task-status.md
rg -n '^\s*- \[ \]' docs/engineering/decisions/package-capability-matrix.md
rg -n '\| READY \|' docs/publisher-services/task-status.md docs/metrics/task-status.md
```

Result:

```text
ADR and appendix status/approver/date/PR: present and consistent.
MET-CTRL-01: CHANGES REQUIRED.
Unchecked CTO checklist items: none.
READY tracker rows: none.
BE-01: BLOCKED by its own approved bounded specification.
WP1-WP11 and MET-E2E-01: BLOCKED.
```

### Documentation-only classifier

Command:

```text
python3 .github/scripts/classify_ci_changes.py --paths <the exact fifteen changed paths>
```

Result:

```json
{"docs_only": "true", "run_build": "false", "run_docker": "false", "run_migrations": "false"}
```

### Independent-review correction

Commands:

```text
git diff --check bafd4cbf752f9d6153036fc7f47115220fed3fbd...HEAD
git diff --name-only 4af692490875c66a9a0c7fb32354f10f136889e6...HEAD
```

Result:

```text
The cumulative diff is whitespace-clean.
The correction changes exactly the six approved documentation files.
The four normative matrix copies remain byte-identical and unchanged.
No runtime, workflow, migration, tracker, register, control-gap, changelog or issue file changes in the correction.
```

Semantic inspection confirms:

- OASIS retains no initial capability without creating a distribution rule;
- OBELISK retains only OAI-PMH and configured private collection;
- SPHINX and PYRAMID retain all six capabilities;
- `PYRAMID -> SPHINX` removes no initial capability;
- downgrade to OBELISK and transition to OASIS are explicitly
  capability-scoped;
- every downgrade retains canonical history and distribution assignments;
- in-flight work rechecks the relevant capability at its final boundary;
- collection does not infer entitlement from an assignment or remote location.

### Post-ready active-entry-point correction

Commands:

```text
git diff --check bafd4cbf752f9d6153036fc7f47115220fed3fbd...HEAD
git diff --name-only 1124262dd28e0b51f33259be1b70e1396e3bdb1c...HEAD
rg -n \
  'ADR-0001.*PROPOSED|ADR-0001 remains proposed|ADR-0001.*blocking|blocked.*ADR-0001' \
  docs \
  --glob '*.md'
```

Result:

```text
The cumulative diff is whitespace-clean.
The post-ready correction changes exactly the five CTO-authorized files.
No active README describes ADR-0001 as proposed or lists ADR-0001 approval as
an outstanding blocker.
Remaining search matches are historical task/report/review evidence,
baseline/change-history statements in the current specification, or the
validation command and result text in the current task/report. No result is
ambiguous.
```

The engineering, Metrics and Publisher Services README entry points consistently
record ADR-0001 as approved by Javi, CTO, on 2026-07-28 through PR #772, pending
merge of the approval record. They preserve Publisher Services ADR-01 and final
inventory, task-specific specifications including BE-01, branch readiness,
`MET-CTRL-01`, Sphinx bootstrap, Diesel generation, service-role, fixture,
COUNTER and OPERAS completeness blockers. No implementation task is marked
ready.

### Second post-ready rollout-gate correction

Commands:

```text
git diff --check bafd4cbf752f9d6153036fc7f47115220fed3fbd...HEAD
git diff --name-only 56c4f873c27fa83e6358c1f207cd718cb3dde679...HEAD
rg -n 'ADR-0001' docs \
  --glob '*.md' \
  --glob '!docs/engineering/ai-delivery/tasks/**' \
  --glob '!docs/engineering/ai-delivery/implementation-reports/**'
rg -n \
  'ADR-0001.*(PROPOSED|proposed|outstanding|awaiting|pending approval|block)|((outstanding|awaiting|pending approval|block).*)ADR-0001' \
  docs \
  --glob '*.md'
```

Result:

```text
The cumulative diff is whitespace-clean.
The second post-ready correction changes exactly the three CTO-authorized files.
The cumulative pull request changes exactly fifteen approved documentation paths.
No runtime, workflow, migration, API, issue or normative decision file changed.
No active document describes ADR-0001 as proposed, outstanding, awaiting
approval or a current blocker, and no active document claims runtime
implementation. No active result is ambiguous.
```

Every result from the exhaustive active-document search is classified:

- normative approved decision records:
  `docs/engineering/decisions/ADR-0001-publisher-package-capability-model.md`,
  `docs/engineering/decisions/package-capability-matrix.md`,
  `docs/engineering/decisions/decision-register.md`,
  `docs/publisher-services/decisions.md` and `docs/metrics/decisions.md`;
- current control and status records that describe approval as satisfied while
  preserving other blockers:
  `docs/engineering/README.md`,
  `docs/engineering/repository-map/control-gaps.md`,
  `docs/publisher-services/README.md`,
  `docs/publisher-services/task-status.md`,
  `docs/publisher-services/rollout-plan.md`,
  `docs/metrics/README.md` and `docs/metrics/task-status.md`;
- reference-only historical review brief:
  `docs/engineering/ai-delivery/reviews/CTRL-FOUNDATION-01-review-brief.md`.

Stage 0 still requires ADR-0001 approval and now records its decision, Javi as
CTO approver, 2026-07-28 decision date and PR #772 under achieved evidence.
Only `ADR-0001 approval` was removed from outstanding evidence. Publisher
Services ADR-01, final inventory approval, applicable repository and branch
readiness, approved bounded task specifications, independent review assignments
and absence of unresolved control contradictions remain outstanding. Stage 0,
Stage 1, `BE-01`, `OAI-01` and the programme remain unready and unactivated.

### Internal path and terminology inspection

Result:

```text
Referenced repository paths exist.
Canonical repository and programme names are used.
No active scoped document claims that approval starts or completes implementation.
No stale active README describes ADR-0001 as PROPOSED or an outstanding
approval blocker.
```

### Rust, database and workflow tests

Not run and not applicable. This task changes no Rust, SQL, migration, GraphQL,
JavaScript, generated or workflow file. GitHub Actions must confirm the merged
CI-DOCS-01 documentation-only gating at the exact final head.

## 10. Manual verification

Environment: local task branch based on the exact approved `develop` commit.

Verified:

1. the specification is the only file in the first commit;
2. PR #772 is draft and targets `develop`;
3. all fifteen cumulative changed paths are allowlisted and
   documentation-only;
4. all four matrix copies are identical;
5. OASIS, OBELISK, SPHINX and PYRAMID match the CTO decision;
6. every checklist item is complete;
7. approver, date and actual PR number are consistent;
8. ADR-01 and the final distribution-platform inventory remain unresolved;
9. `BE-01`, `MET-CTRL-01` and all Metrics work packages retain their required
   non-ready status;
10. issues #765 and #766 were not modified;
11. no runtime, migration, workflow, deployment, release or production action
    occurred.
12. the PR returned to draft before post-ready repository changes;
13. the post-ready correction changes exactly the five amended files;
14. active README entry points agree with the normative decision and tracker
    records while programme implementation remains blocked.
15. the PR returned to draft before the second post-ready correction;
16. the second correction changes exactly the three amended files;
17. every active-document ADR-0001 reference is classified without ambiguity;
18. Stage 0 records ADR-0001 approval as achieved while all genuine Publisher
    Services rollout and implementation blockers remain.

## 11. CI

CI status at report creation: PENDING

Required final exact-head conclusions:

| Workflow/job | Required conclusion |
|---|---|
| all three classifier jobs | `success` |
| `build` | `skipped` |
| `test` | `skipped` |
| `lint` | `skipped` |
| `format_check` | `skipped` |
| `run_migrations` | `skipped` |
| `build_and_push_staging_docker_image` | `skipped` |
| `check-changelog` | `success` |

The exact final head, workflow run IDs, classifier conclusions, protected and
mandatory job conclusions, step-level heavy-work inspection and evidence-comment
URL are recorded externally after CI completes. A failure, missing/pending
context or executed heavy step is a stop condition.

## 12. CI-DOCS-01 observation

Observation at report creation: PENDING exact-final-head CI.

PR #772 is the first controlled documentation-only observation after CI-DOCS-01
merged through PR #771. The final evidence comment must prove:

- no Rust build, test or lint step executed;
- no migration build, apply or revert step executed;
- no Docker login, build or push step executed;
- all six protected contexts and the additional mandatory Docker context reached
  terminal merge-safe conclusions;
- `check-changelog` remained active;
- no required context remained pending.

Successful evidence satisfies only the controlled documentation-only
observation. The mixed-source and next-three-PR observations remain outstanding,
and CI-DOCS-01 must not be marked operationally complete.

### 12.1 Historical reviewed-head evidence

The immutable observation for previous reviewed head
`4af692490875c66a9a0c7fb32354f10f136889e6` remains unchanged:

- build-test-and-check run `30390801269`;
- run-migrations run `30390801250`;
- publish-to-dockerhub run `30390801048`;
- check-changelog run `30390801277`;
- [historical observation comment](https://github.com/thoth-pub/thoth/pull/772#issuecomment-5108602642).

That evidence is historical and does not cover the correction commit.

### 12.2 Corrective exact-head evidence

Status at correction-report creation: PENDING push and exact-head CI.

The correction remains documentation-only. At its new exact head, all three
classifiers must succeed; build, test, lint, format, migrations and Docker must
be skipped with no executed heavy steps; and `check-changelog` must succeed.

After those terminal conclusions, a new immutable top-level PR comment records
the correction commit, new final head, revised workflow run IDs, job/step
conclusions and its own URL. The comment is the authoritative location because
the correction commit cannot contain its own SHA or CI runs. The historical
comment above must not be edited.

### 12.3 Post-ready active-entry-point evidence

Status at post-ready correction-report creation: PENDING push and exact-head CI.

The automated post-ready P1 reviewed head
`1124262dd28e0b51f33259be1b70e1396e3bdb1c`. The correction commit and its new
final head are recorded in the immutable top-level evidence comment and handoff
after push because this report is part of that commit.

At the new exact head, all three classifiers must succeed; build, test, lint,
format, migrations and Docker must be skipped with no executed heavy steps; and
`check-changelog` must succeed. Only after that evidence is terminal may the
implementation reply to and resolve the post-ready P1 thread. The previous
independent approval and CTO authorization remain historical and do not cover
the new head.

### 12.4 Second post-ready rollout-gate evidence

Status at second post-ready correction-report creation: PENDING push and
exact-head CI.

The second automated post-ready P1 reviewed exact head
`56c4f873c27fa83e6358c1f207cd718cb3dde679`. The correction commit, new final
head, exact-head workflow runs, immutable observation-comment URL and review
thread reply are recorded externally after push because this report is part of
the correction commit.

At the new exact head, all three classifiers must succeed; build, test, lint,
format, migrations and Docker must be skipped with empty step arrays; and
`check-changelog` must succeed. Only after that terminal evidence may the
implementation reply to and resolve thread `PRRT_kwDODkn0bc6UtkFD`. The prior
independent approval and CTO authorization remain historical and do not cover
the new head. Fresh exact-head independent review, fresh CTO authorization and a
new automated post-ready review are required before merge.

## 13. Rollout and rollback

Initial state after merge:

- ADR-0001 becomes an approved merged architectural dependency;
- no package, capability, metrics or OAI-PMH runtime behaviour is implemented or
  activated by this documentation;
- Publisher Services and Metrics work remains blocked by task-specific and
  programme-readiness dependencies;
- CI-DOCS-01 has one controlled documentation-only observation only.

Activation required: none.

Feature flag/configuration: none.

Migration sequence: none.

Deployment/release: none.

Merge gate: fresh independent `APPROVED` review followed by explicit CTO
authorization. The implementing agent must not approve or merge.

Rollback: normal revert of PR #772, preserving unrelated later changes. No
database, data, source-account, API, authorization, distribution, deployment,
release or production rollback is required.

## 14. Known limitations and remaining blockers

- Publisher Services `ADR-01` and the final distribution-platform inventory
  remain unresolved.
- `BE-01` remains `BLOCKED` pending its own approved bounded specification.
- `MET-CTRL-01` remains `CHANGES REQUIRED`.
- THOTH-DB-CTRL-01 Diesel generation procedure remains unresolved.
- SPHINX bootstrap and branch readiness remain blocked.
- Dashboard, widget and app branch-readiness gates remain blocked.
- Metrics service-role codes, source fixtures, COUNTER decisions, OPERAS
  completeness and other work-package dependencies remain unresolved.
- Every Metrics work package remains `BLOCKED`.
- The CI-DOCS-01 mixed-source and next-three-PR observations remain outstanding.
- Second post-ready corrective exact-head CI and P1 thread resolution remain
  pending at correction-report creation.
- Fresh independent exact-head review and fresh explicit CTO authorization are
  required for the new head.

## 15. Unresolved issues

- Second post-ready corrective exact-final-head CI evidence and the new
  top-level observation comment are pending at correction-report creation.
- The second post-ready P1 reply and resolution are pending exact-head CI.
- Fresh independent review is pending.
- Explicit CTO merge authorization is pending.

The earlier distribution, downgrade and active-entry-point findings are
addressed. The rollout-gate P1 is addressed in the working correction but
remains open until pushed exact-head validation and CI permit an immutable reply
and resolution. No runtime defect or scope deviation is known.

## 16. Agent self-assessment

The implementing agent does not issue an approval decision.

Suggested independent-review focus:

- exact matrix identity and OASIS/OBELISK semantics;
- preservation of code ownership, stable capability codes, no mapping rows, no
  bespoke overrides and entitlement/configuration separation;
- upgrade, downgrade and historical OPERAS export behaviour;
- residual Publisher Services and Metrics blockers;
- consistency of the three active README entry points with the normative
  decision and tracker records;
- exhaustive active-document ADR-0001 classification and the corrected
  Publisher Services rollout gate;
- specification-first commit order and exact allowlist;
- exact-head CI context and job-step evidence;
- absence of runtime, migration, workflow, issue, deployment, release and
  production effects.
