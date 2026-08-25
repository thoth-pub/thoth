# MET-CTRL-01 Implementation Report

## 1. Repository state

Owning GitHub issue: [thoth-pub/thoth#832](https://github.com/thoth-pub/thoth/issues/832)
(parent programme issue
[#766](https://github.com/thoth-pub/thoth/issues/766))
Repository: thoth-pub/thoth
Workflow: STANDARD
Base branch: `develop`
Authorized base commit: `250554dd7351c97af46d59b5033abd391d9eec16`
Actual base commit: `250554dd7351c97af46d59b5033abd391d9eec16` (verified with
`git fetch origin --prune` and `git rev-parse origin/develop` immediately
before branch creation; worktree clean)
PR target: `develop`
Programme integration branch: None (`feature/metrics` deliberately does not
exist and is not created by this task)
Task branch: `feature/metrics-control/met-ctrl-01`
Head commit: recorded in the pull request; this report is written at the
branch head that carries it (a file cannot embed the SHA of its own
containing commit)
Pull request: draft pull request against `develop`; live state is the GitHub
record
Expected branch deletion after merge: YES
Final programme PR required: NO
Implementing model: bounded coding/documentation agent
Reasoning level: high

## 2. Scope confirmation

Approved specification: the complete current body of issue #832, including
its 2026-08-25 specification amendment, encoded in
[`docs/engineering/ai-delivery/tasks/MET-CTRL-01.md`](../tasks/MET-CTRL-01.md).
The specification received a fresh independent non-implementing review
(`APPROVED`) before implementation was authorized.

Implemented objective: reconcile the repository-backed Thoth Metrics
programme controls with the actual live merged state, so the remaining gates
for beginning the first bounded Thoth WP1 slice are accurate, durable and
independently reviewable. Documentation/control only.

Out-of-scope changes made: NONE

## 3. Commits

- `docs(metrics): reconcile MET-CTRL-01 programme controls (#832)` — the
  single bounded commit carrying the task specification, the Metrics and
  engineering-control reconciliations, the changelog entry and this report.

Exact SHAs are live in the pull-request record.

## 4. Files changed

Authorized write paths (from the amended #832 write budget):

- `CHANGELOG.md`
- `docs/metrics/README.md`
- `docs/metrics/task-status.md`
- `docs/metrics/decisions.md`
- `docs/metrics/rollout-plan.md`
- `docs/metrics/contract-register.md`
- `docs/metrics/master-issue.md` (conditional)
- `docs/engineering/repository-map/control-gaps.md`
- `docs/engineering/README.md`
- `docs/engineering/repository-map/repositories/thoth.md`
- `docs/engineering/decisions/decision-register.md`

Authorized new-file paths:

- `docs/engineering/ai-delivery/tasks/MET-CTRL-01.md`
- `docs/engineering/ai-delivery/implementation-reports/MET-CTRL-01-implementation-report.md`

Actual files changed:

- `CHANGELOG.md`
  - reason: one bounded `Unreleased` entry for this documentation/control
    reconciliation (also required by the changelog CI check);
  - behavioural effect: none;
  - within authorized write budget: YES
- `docs/metrics/README.md`
  - reason: section 6 still recorded the pre-pivot blocking-control list
    (`MET-CTRL-01` `CHANGES REQUIRED`, "Thoth Diesel generation procedure is
    unresolved", blanket Sphinx/branch/role/fixture blockers). Reconciled
    into completed shared controls, the remaining Thoth-local WP1 entry gate
    and later owning-work-package gates; status line updated;
  - behavioural effect: none;
  - within authorized write budget: YES
- `docs/metrics/task-status.md`
  - reason: `MET-CTRL-01` recorded as the active programme-control task under
    #832; `THOTH-DB-CTRL-02`/ADR-0003 recorded as merged and
    repository-authoritative (merge commit
    `37b802776ae6853affe19d90156f3c1e0654ebe3`) instead of
    `AUTHORITATIVE ON MERGE`; ADR-0008's authority condition recorded as
    satisfied (record merged through PR #815); WP1's blocking dependencies
    reduced to its actual entry gates; immediate next actions updated with
    the post-MET-CTRL sequence;
  - behavioural effect: none;
  - within authorized write budget: YES
- `docs/metrics/decisions.md`
  - reason: stale shared-decision status only — recorded ADR-0003 and
    ADR-0008 current authority in section 2; re-scoped "Service-role codes"
    as WP5-owned bounded decisions (proposed codes explicitly not promoted);
    marked "Lease/job primitive reuse" as decided by ADR-0008. No
    architectural content revised;
  - behavioural effect: none;
  - within authorized write budget: YES
- `docs/metrics/rollout-plan.md`
  - reason: Stage 0 previously required "Sphinx bootstrap, branch decisions,
    Diesel procedure and role decision" as one undifferentiated readiness
    list. Reconciled to distinguish completed shared/global controls, the
    remaining Thoth-local WP1 entry gate, Sphinx/WP6 readiness,
    client-specific readiness, source/driver-specific readiness and WP5
    service-role work. No later gate was removed;
  - behavioural effect: none;
  - within authorized write budget: YES
- `docs/metrics/contract-register.md`
  - reason: the service-role section (titled `Authentication`) presented the
    entire machine-role architecture as undecided (`CTO APPROVAL REQUIRED`).
    Reconciled: ADR-0008 established the shared domain-specific
    least-privilege convention; exact Metrics role codes, permissions/
    operation matrix and credential/provisioning arrangements remain
    unapproved WP5-owned bounded decisions; proposed codes remain proposals;
    `SUPERUSER` is not a machine-service shortcut;
  - behavioural effect: none;
  - within authorized write budget: YES
- `docs/engineering/repository-map/control-gaps.md`
  - reason: CG-08 only. It previously read "Sphinx normalization/bootstrap,
    Diesel control, branch readiness and service-role decisions remain
    prerequisites". Reconciled to distinguish Thoth WP1 entry (MET-CTRL-01
    closure; satisfied shared architecture/schema controls; separately
    authorized `feature/metrics`; approved bounded WP1 child specification)
    from the later Sphinx/WP6, client, WP5 role, source-fixture and OPERAS
    gates, each cross-referenced to its owning gap. CG-03, CG-04, CG-09,
    CG-10, CG-11 and CG-13 are byte-for-byte unchanged;
  - behavioural effect: none;
  - within authorized write budget: YES
- `docs/engineering/README.md`
  - reason: active Metrics gate wording only — the section 9 "Outstanding"
    bullet still cited "the Metrics programme-control, Diesel-generation and
    repository-readiness gates" as blanket implementation blockers.
    Reconciled to the resolved Diesel/schema control and the WP1-entry vs
    later-gate distinction. Publisher Services wording and all other history
    untouched;
  - behavioural effect: none;
  - within authorized write budget: YES
- `docs/engineering/repository-map/repositories/thoth.md`
  - reason: the schema-contract section still described ADR-0003/PR #778
    prospectively ("this record becomes authoritative when the change
    merges"). Reconciled to the actual merged authority (merge commit
    `37b802776ae6853affe19d90156f3c1e0654ebe3`). Architecture A, the manually
    maintained `schema.rs` authority, the no-Diesel-CLI rule and the CG-13
    production/runtime restrictions are preserved;
  - behavioural effect: none;
  - within authorized write budget: YES
- `docs/engineering/decisions/decision-register.md`
  - reason: ADR-0003's temporally stale authority text only — the table row
    and the approval-sequence paragraph both said it "becomes
    repository-authoritative on merge into `develop`". Reconciled to record
    PR #778 merged into `develop` as
    `37b802776ae6853affe19d90156f3c1e0654ebe3` and ADR-0003
    repository-authoritative; status cell updated to
    `APPROVED AND REPOSITORY-AUTHORITATIVE` in the register's established
    style. ADR-0003's decision content is unchanged;
  - behavioural effect: none;
  - within authorized write budget: YES

Actual new files created:

- `docs/engineering/ai-delivery/tasks/MET-CTRL-01.md` - within authorized
  new-file list: YES
- `docs/engineering/ai-delivery/implementation-reports/MET-CTRL-01-implementation-report.md`
  - within authorized new-file list: YES

`docs/metrics/master-issue.md` was reviewed and **not** changed: it contains
links and a process rule only, and no concrete active stale statement
requiring correction was identified.

`docs/engineering/ai-delivery/tasks/THOTH-DB-CTRL-02.md` was deliberately
**not** edited: its pre-merge conditional language is a point-in-time
historical task record outside the write budget, per the #832 historical
record rule. No historical implementation report was rewritten.

Files deleted, moved or renamed: NONE

### 4.1 Write-budget compliance

PASS — every changed file is in the amended approved write budget; the two
new files are exactly the two authorized new files.

## 4.2 Authorized actions actually used

- repository inspection: YES (git history, ADRs, control documents, issue
  #832 body via `gh issue view`)
- source edit: YES (within the write budget)
- new file creation: YES (the two authorized files)
- file deletion/move/rename: NO
- branch creation: YES (`feature/metrics-control/met-ctrl-01` from exact base
  `250554dd7351c97af46d59b5033abd391d9eec16`)
- commit: YES
- push: YES
- PR creation/update: YES (one draft PR against `develop`)
- issue/comment mutation: NO (issues #832 and #766 untouched)
- manual CI dispatch/rerun: NO
- provider/runtime read: NO
- provider/runtime write: NO
- migration execution: NO
- release/tag/publication: NO
- merge: NO
- deployment: NO
- production activation: NO
- other: NONE

Unauthorized actions performed: NONE

## 4.3 Automatic and manual external effects

Automatic CI/provider effects observed: the repository's normal PR CI
triggered by push/PR-open (build/test/clippy/format/migrations/changelog
checks). For this documentation-only diff no external write (container
registry, release, package) is expected or was observed.

Manually initiated external actions: NONE

External writes/publication: NONE

## 5. Implementation decisions

1. The Metrics README status line was updated from
   `CONTROL FOUNDATION IN PROGRESS` (stale: P0-01 is closed) to
   `PROGRAMME CONTROLS UNDER RECONCILIATION (MET-CTRL-01)`; the programme
   decision remains `BLOCKED FOR IMPLEMENTATION`.
2. `docs/metrics/task-status.md` section 3.1's ADR-0008 authority condition
   was marked satisfied with the merged-record evidence (PR #815) rather than
   deleted, preserving the process rule it states.
3. In `docs/metrics/decisions.md` the "Service-role codes" and "Lease/job
   primitive reuse" items were retitled and re-scoped rather than removed, so
   the WP5-owned open decisions remain visible while the already-decided
   shared convention is no longer presented as open.
4. CG-08 remains **OPEN**: reconciling its gating model does not close it;
   closure requires the WP1 entry path it now records.
5. `docs/engineering/README.md` section 6 (which lists only ADR-0001/0002)
   was left unchanged — it is an incomplete but not incorrect index, and the
   decision register it links to carries the full current ADR authority; only
   the actively stale Metrics gate wording in section 9 was corrected.
6. One accurate new sentence was reworded ("Diesel-generation gate is
   resolved" -> "Diesel/schema-control blocker is resolved") so the #832
   stale-state search returns zero matches without any active statement being
   silenced.

List any deviation from the specification requiring authorization: NONE

## 6. Database and migration effects

Migration added: NO. No SQL, `thoth-api/migrations/**`,
`thoth-api/src/schema.rs`, data, lock or downtime effect.

## 7. API and compatibility effects

GraphQL/API changes: NONE
Generated schema/client updates: NONE
Backwards compatibility: unaffected (documentation/control only)
Deprecations: NONE
Cross-repository dependencies: NONE (no other repository is touched; Sphinx,
client and source gates remain with their owning work packages)

## 8. Authorization and security

Authorization paths changed: NONE
Roles/scopes involved: none — no Metrics role name, permission,
entitlement, credential or provisioning arrangement was selected; those
remain WP5-owned bounded decisions under the ADR-0008 convention
Negative authorization tests: not applicable
Secret or personal-data handling: NONE
Security limitations: NONE

## 9. Tests and checks

### Formatting

Command:

```text
git diff --check 250554dd7351c97af46d59b5033abd391d9eec16
```

Result:

```text
no output — PASS (no whitespace errors)
```

### Scope verification

Command:

```text
git diff --name-only 250554dd7351c97af46d59b5033abd391d9eec16...HEAD
```

Result:

```text
CHANGELOG.md
docs/engineering/README.md
docs/engineering/ai-delivery/implementation-reports/MET-CTRL-01-implementation-report.md
docs/engineering/ai-delivery/tasks/MET-CTRL-01.md
docs/engineering/decisions/decision-register.md
docs/engineering/repository-map/control-gaps.md
docs/engineering/repository-map/repositories/thoth.md
docs/metrics/README.md
docs/metrics/contract-register.md
docs/metrics/decisions.md
docs/metrics/rollout-plan.md
docs/metrics/task-status.md
```

All twelve paths are within the amended budget; no Rust, SQL, migration,
GraphQL, workflow or generated-contract file changed (a targeted
`git diff --stat` over `*.rs`, `*.sql`, `*.toml`, `*.lock`, `.github` and
`thoth-api/migrations` returned zero entries).

### ADR authority verification

Command:

```text
grep -n '^Status: APPROVED$' \
  docs/engineering/decisions/ADR-0001-publisher-package-capability-model.md \
  docs/engineering/decisions/ADR-0002-platform-domain-boundaries.md \
  docs/engineering/decisions/ADR-0003-repository-authoritative-schema-contract.md \
  docs/engineering/decisions/ADR-0008-machine-roles-and-durable-job-primitives.md
```

Result:

```text
all four files match at line 3 — PASS
```

Merged-evidence verification: `git merge-base --is-ancestor
37b802776ae6853affe19d90156f3c1e0654ebe3 250554dd7351c97af46d59b5033abd391d9eec16`
succeeded (PR #778's merge commit is an ancestor of the authorized base), and
the PR #815 merge commit (`8703dd5ca2080bb97debc9d14cca33db9956f7b4`,
ADR-0008 record) is reachable from `origin/develop`.

### Stale-state search (issue #832)

Command:

```text
grep -RniE \
  'Diesel generation procedure is unresolved|Diesel-generation.*gate|THOTH-DB-CTRL-02.*(pending|on merge)|THOTH-DB-CTRL-01 complete|becomes repository-authoritative on merge|becomes authoritative when the change merges|Sphinx normalization/bootstrap, Diesel control, branch readiness and service-role decisions remain prerequisites' \
  docs/metrics \
  docs/engineering/README.md \
  docs/engineering/repository-map/control-gaps.md \
  docs/engineering/repository-map/repositories/thoth.md \
  docs/engineering/decisions/decision-register.md
```

Result:

```text
no matches — PASS (zero active stale-state occurrences; nothing was silenced,
see section 5 item 6)
```

A supplementary sweep for `AUTHORITATIVE ON MERGE`, `CTO APPROVAL REQUIRED`
and `MET-CTRL-01 ... CHANGES REQUIRED` over the same files also returned no
active stale statement; remaining `Diesel` mentions are accurate
resolved-state records or the deliberately historical `THOTH-DB-CTRL-01`
`SUPERSEDED` row.

### Preserved-gap verification

Command:

```text
grep -nE '^### CG-(03|04|09|10|11|13)' \
  docs/engineering/repository-map/control-gaps.md
```

Result:

```text
26:### CG-03 - `thoth-sphinx` is bootstrap-only
49:### CG-04 - Branch topology differs
248:### CG-09 - Source fixtures/mappings incomplete
252:### CG-10 - OPERAS inbound completeness unavailable
256:### CG-11 - CI gaps
319:### CG-13 - Thoth runtime operations unmapped
```

All six preserved gaps present; their bodies are unchanged by this task
(the diff touches only the CG-08 section).

### Branch-safety verification

`git branch -r | grep -i 'feature/metrics'` before branch creation returned
only this task's own branch path after push; no `feature/metrics` integration
branch exists and none was created.

### Other required checks

Unit/integration/lint checks: not applicable — no Rust, SQL or workflow file
changed; the repository's own CI runs the full check suite on the PR.

## 10. Manual verification

Environment: local clean worktree on
`feature/metrics-control/met-ctrl-01`.
Steps: preflight (`git fetch origin --prune`, `git status --short`,
`git rev-parse origin/develop`); reviewed every in-budget document against
issue #832 and the live merged evidence; applied the reconciliation; ran the
section 9 checks; verified relative links target existing files
(`docs/engineering/decisions/ADR-0008-machine-roles-and-durable-job-primitives.md`
from `docs/metrics/`, control-gap anchors from `docs/engineering/README.md`).
Observed result: all checks pass; changed set is exactly the twelve in-budget
paths.
Evidence link/screenshot/log reference: command results above; live PR record.

## 11. CI

CI status: PENDING at report-writing time (CI is triggered by the push/PR
that carries this report; live state is the GitHub PR record).
Checks: normal PR suite (build, workspace tests, clippy, formatting,
migrations, changelog check).
Failures or warnings: none expected for a documentation-only diff; the
changelog check is satisfied by the bounded `Unreleased` entry.

## 12. Rollout and rollback

Initial state after merge: reconciled controls become
repository-authoritative; no runtime effect.
Activation required: NONE — nothing is activated by this task.
Feature flag/configuration: NONE.
Migration sequence: NONE.
Rollback/disable procedure: revert the complete documentation/control merge
if required. Any #766 synchronization rollback must re-fetch live
issue/comment state, preserve unrelated later edits and use a freshly
reviewed minimal reversal under explicit authorization.
Monitoring required: NONE.

## 13. Known limitations and deferred work

- CG-08 remains OPEN until the recorded WP1 entry path completes.
- `feature/metrics` creation, the bounded WP1 child specification and all
  WP1+ implementation remain separately controlled, unauthorized work.
- Sphinx (BR-SPHINX-01/SPHINX-BOOT-01), client (BR-DASH-01/BR-WIDGET-01/
  BR-APP-01 and the CG-11 CI gaps), source-fixture/COUNTER, OPERAS
  completeness and WP5 service-role decisions remain open with their owning
  work packages.
- The #766 issue body still reflects pre-reconciliation state; its
  synchronization is a separately authorized post-merge GitHub mutation
  (section 15).

## 14. Unresolved issues

- NONE

## 15. Proposed post-merge #766 synchronization comment

The following is the exact proposed comment for issue #766. It is recorded
here as text only. **It must not be posted** as part of this implementation;
posting it is a separately authorized GitHub mutation, permitted only after
this PR receives fresh independent exact-head review, explicit CTO merge
authorization and merge, and only if the merged result still supports it.

```text
MET-CTRL-01 synchronization (issue #832)

MET-CTRL-01 - "Reconcile Metrics programme controls and open the Thoth WP1
gate" - has been independently reviewed, CTO merge-authorized and merged into
develop. The repository-backed Metrics programme controls now record the
live state:

Completed shared controls:
- ADR-0001 (publisher package capabilities): APPROVED, merged through PR #772
  as b2c91ff25b95ab0e10a477ff21dbd4702f5db8d4.
- ADR-0002 (platform domain boundaries): APPROVED, merged through PR #769.
- ADR-0003 (repository-authoritative Diesel schema contract, Architecture A):
  repository-authoritative through THOTH-DB-CTRL-02 and merged PR #778
  (merge commit 37b802776ae6853affe19d90156f3c1e0654ebe3). THOTH-DB-CTRL-01
  is SUPERSEDED and the Diesel/schema-control blocker is resolved (CG-12).
- ADR-0008 (machine roles and durable job primitives):
  repository-authoritative within its approved scope; it selects no Metrics
  role name, entitlement model, credential model or operation matrix - those
  remain WP5-owned bounded decisions.

Gate model after reconciliation (CG-08):
- Thoth WP1 entry requires only: MET-CTRL-01 closure (now complete),
  separately authorized creation of repository-local feature/metrics from a
  freshly verified develop head, and one approved bounded WP1 child
  specification.
- Later gates stay with their owning work packages and do not block WP1
  entry: BR-SPHINX-01/SPHINX-BOOT-01 gate WP6/Sphinx work; client branch/CI
  readiness gates the client-dependent work packages; source fixtures,
  COUNTER mappings and OPERAS inbound completeness gate the applicable
  driver/import/inbound work; exact Metrics service-role decisions are
  WP5-owned.

Next Thoth action: separately authorize feature/metrics creation from the
then-current exact develop head, then create and approve one bounded WP1
child specification. No WP1 implementation, migration, deployment,
production action or Metrics activation of any kind is authorized by
MET-CTRL-01's closure.
```

## 16. Agent self-assessment

The agent may identify risks but may not approve the task.

Suggested review focus:

- verify CG-03, CG-04, CG-09, CG-10, CG-11 and CG-13 are byte-identical to
  the base (the diff should touch only CG-08 within `control-gaps.md`);
- verify no reconciliation statement over-claims: WP1 remains blocked and
  unauthorized, later gates remain open, and no Metrics role code was
  promoted from proposal to approved;
- verify the ADR-0003 merged-authority record (`37b8027…`) against the live
  PR #778 record;
- verify the ADR-0008 authority-condition satisfaction against the live PR
  #815 record;
- confirm `docs/metrics/master-issue.md` needed no change and that the
  unedited historical `THOTH-DB-CTRL-02.md` task record is correctly treated
  as point-in-time evidence.
