# MET-CTRL-01-CLOSEOUT-01 Implementation Report

## 1. Repository state

Repository: `thoth-pub/thoth`
Task: `MET-CTRL-01-CLOSEOUT-01`
Task issue: [#834](https://github.com/thoth-pub/thoth/issues/834)
Parent task: `MET-CTRL-01` (issue [#832](https://github.com/thoth-pub/thoth/issues/832))
Programme issue: [#766](https://github.com/thoth-pub/thoth/issues/766)
Workflow: STANDARD documentation/control closeout
Risk: LOW
Base branch: `develop`
PR target: `develop`
Programme integration branch: None
Task branch: `feature/metrics-control/met-ctrl-01-closeout`
Expected branch deletion after merge: YES
Final programme PR required: NO
Implementing model: Claude Opus 5

### 1.1 Exact authorized base

```text
e62a476e87916d0b44a963652c9c5e7ab6afa10e
```

This is the merge commit of PR [#833](https://github.com/thoth-pub/thoth/pull/833)
and the exact `develop` head named in the #834 specification and in the bounded
implementation authorization.

### 1.2 Preflight, performed before branch creation and before any edit

```bash
git fetch origin --prune
git status --short
git rev-parse origin/develop
git branch -a --list '*met-ctrl-01-closeout*'
git branch -a --list '*feature/metrics'
```

Observed:

- `git status --short` produced no output; the worktree was clean.
- `git rev-parse origin/develop` returned
  `e62a476e87916d0b44a963652c9c5e7ab6afa10e`, exactly the authorized base.
- No `met-ctrl-01-closeout` branch existed.
- No `feature/metrics` branch existed.

Both HOLD conditions were therefore clear and the branch was created from the
exact authorized SHA:

```bash
git checkout -b feature/metrics-control/met-ctrl-01-closeout \
  e62a476e87916d0b44a963652c9c5e7ab6afa10e
```

### 1.3 Parent lifecycle re-verification, performed before any edit

The #834 specification requires re-verifying that live parent lifecycle
evidence still matches the corrected reconstruction. Observed:

- PR #833: `state=MERGED`, `baseRefName=develop`,
  `headRefName=feature/metrics-control/met-ctrl-01`,
  `headRefOid=bbe0928b96d8475a38628cdf1fd08455da418d83`,
  `mergeCommit=e62a476e87916d0b44a963652c9c5e7ab6afa10e`,
  `mergedAt=2026-08-25T17:41:47Z`, `isDraft=false`.
- `e62a476e87916d0b44a963652c9c5e7ab6afa10e` has parents
  `250554dd7351c97af46d59b5033abd391d9eec16` (pre-merge `develop`) and
  `bbe0928b96d8475a38628cdf1fd08455da418d83` (implementation head).
- #832 comment `5414236565` exists, is authored by the CTO, and records
  `Decision: APPROVED` at reviewed head
  `bbe0928b96d8475a38628cdf1fd08455da418d83` against reviewed base
  `250554dd7351c97af46d59b5033abd391d9eec16`.

Live evidence matches the reconstruction. No HOLD condition was triggered.

### 1.4 Final implementation head

The final head of this branch is the commit carrying the independent-review
remediation described in section 12. The first reconciliation commit is
`78b9db5d5fe0d887cec72f321038aa041ee48c1c`. The exact final head is terminal
GitHub evidence on the pull request; under `ADR-0005` this report does not
require a further commit to transcribe it, and no commit is created solely to
transcribe its own SHA.

## 2. Scope confirmation

This task changes documentation and control records only. It creates no
branch other than the authorized task branch, no issue, no comment, no
`feature/metrics` branch and no WP1 child specification.

Changed files are a strict subset of the approved eight-path #834 write
budget. Six of the eight approved paths were used; the two unused paths are
none, because both new files were required by the specification and all six
existing files carried active stale state.

## 3. Commits

```text
78b9db5d5fe0d887cec72f321038aa041ee48c1c
  MET-CTRL-01-CLOSEOUT-01: reconcile post-merge Metrics programme state
  - CHANGELOG.md
  - docs/engineering/README.md
  - docs/engineering/repository-map/control-gaps.md
  - docs/metrics/README.md
  - docs/metrics/rollout-plan.md
  - docs/metrics/task-status.md
  - docs/engineering/ai-delivery/tasks/MET-CTRL-01-CLOSEOUT-01.md (new)

<report commit; exact SHA is GitHub PR evidence>
  MET-CTRL-01-CLOSEOUT-01: add implementation report
  - docs/engineering/ai-delivery/implementation-reports/MET-CTRL-01-CLOSEOUT-01-implementation-report.md (new)

<current PR head; exact SHA is GitHub PR evidence>
  MET-CTRL-01-CLOSEOUT-01: correct parent lifecycle evidence wording
  - CHANGELOG.md
  - docs/metrics/README.md
  - docs/metrics/task-status.md
  - docs/engineering/ai-delivery/implementation-reports/MET-CTRL-01-CLOSEOUT-01-implementation-report.md
```

The third commit is the additive remediation required by the independent
exact-head review of `46dd669ed0ac440a9cf33aa2e1028940253fe80f`, recorded in
section 12. No commit on this branch was amended, rebased or force-pushed.

## 4. Files changed

```text
CHANGELOG.md
docs/engineering/README.md
docs/engineering/ai-delivery/implementation-reports/MET-CTRL-01-CLOSEOUT-01-implementation-report.md
docs/engineering/ai-delivery/tasks/MET-CTRL-01-CLOSEOUT-01.md
docs/engineering/repository-map/control-gaps.md
docs/metrics/README.md
docs/metrics/rollout-plan.md
docs/metrics/task-status.md
```

All eight paths are in the approved #834 write budget. No path outside the
budget was created, modified, renamed or deleted.

### 4.1 `docs/metrics/README.md`

- Programme status line changed from
  `PROGRAMME CONTROLS UNDER RECONCILIATION (MET-CTRL-01)` to
  `PROGRAMME CONTROLS RECONCILED - IMPLEMENTATION NOT AUTHORIZED`. The new
  wording is durable: it stays truthful before and after any further review or
  merge event.
- `MET-CTRL-01` added to the completed shared/global controls as
  `MERGED - COMPLETE`, delivered through PR #833 and reachable from `develop`,
  with its dependency recorded satisfied and explicitly no longer a WP1 entry
  gate. The bullet names PR #833 as the parent lifecycle anchor and states that
  exact review and authorization provenance is retained in the owning task and
  closeout evidence, which this active tracker does not restate. It does **not**
  claim that GitHub holds the complete review and authorization record; see
  section 12.
- The former two-part "Remaining Thoth-local gate for entering WP1" block, whose
  item 1 was `MET-CTRL-01` closure, is replaced by exactly two remaining gates:
  separately authorized `feature/metrics` creation from a freshly verified
  `develop` head, and one approved bounded WP1 child issue/specification. Each
  item states that the thing does not exist and that the record does not
  authorize creating it.
- The later-gates block (Sphinx `BR-SPHINX-01`/`SPHINX-BOOT-01`, client
  `BR-DASH-01`/`BR-WIDGET-01`/`BR-APP-01`, source fixtures/COUNTER/OPERAS, and
  WP5 service-role decisions) is unchanged in ownership and scope.
- The closing paragraph still records that no Metrics implementation is
  authorized; only the now-inaccurate self-reference "This reconciliation" was
  changed to "This record".

### 4.2 `docs/metrics/task-status.md`

- `Last updated` provenance refreshed to `MET-CTRL-01-CLOSEOUT-01` / #834 and
  restated in durable programme terms.
- The `MET-CTRL-01` foundation row moves from `ACTIVE` to `MERGED - COMPLETE`.
  Its dependency cell records delivery through PR #833 and reachability from
  `develop`, records the dependency satisfied and no longer gating WP1, names
  PR #833 as the parent lifecycle anchor, and states that exact review and
  authorization provenance is retained in the owning task and closeout evidence
  rather than being restated in this active tracker. The pre-merge clause "Remains the current gate until its PR is
  independently approved at its exact head and merged" is removed. The stale
  point-in-time base SHA in the `Base / target` cell is replaced by
  `develop -> develop`.
- `SPHINX-BOOT-01`: only the `MET-CTRL-01` dependency is annotated
  `(**satisfied**)`. Its status stays `BLOCKED`, its risk stays `MEDIUM`, and
  `BR-SPHINX-01` and the approved bootstrap specification are untouched.
- WP1: `MET-CTRL-01 closure` removed from blocking dependencies, which now read
  `separately authorized feature/metrics creation; approved bounded WP1 child
  specification (the MET-CTRL-01 dependency is satisfied)`. WP1 stays `HIGH` and
  `BLOCKED`.
- WP2-WP11 and `MET-E2E-01` rows are unchanged.
- The paragraph after the work-package table records the `MET-CTRL-01`
  dependency satisfied and WP1 blocked on its two remaining entry gates; the
  later-work-package sentence is unchanged.
- Section 3.1 (WP5 and the shared machine-role convention) is unchanged.
- Section 4 (branch strategy) is unchanged; `feature/metrics` remains a future
  branch in the documented topology and is not created.
- Immediate next action 4 changes from "Close `MET-CTRL-01`" to a durable
  statement that it is `MERGED - COMPLETE` through PR #833 with its dependency
  satisfied. Action 5 changes from "After `MET-CTRL-01` closes" to the two
  remaining WP1 entry gates, explicitly stating that neither the branch nor the
  child specification exists or is authorized. Action 6 (SPHINX-BOOT-01) is
  unchanged.

### 4.3 `docs/metrics/rollout-plan.md`

- Stage 0: `MET-CTRL-01` moves from the remaining WP1-entry gates into the
  completed shared/global controls, recorded as delivered through merged PR #833
  and reachable from `develop`, with its dependency satisfied and no longer a WP1
  entry gate.
- The remaining Stage-0 WP1-entry gates are now exactly `feature/metrics`
  authorization/creation and one approved bounded WP1 child specification.
- The Stage-0 later-gates block and Stages 1-8 are unchanged.

### 4.4 `docs/engineering/README.md`

- The Metrics sentence in the foundation-closeout `Outstanding` block no longer
  says Thoth WP1 entry waits on `MET-CTRL-01` closure. It records the
  `MET-CTRL-01` programme-control dependency satisfied through merged PR #833
  and states that WP1 entry now waits only on separately authorized
  `feature/metrics` creation and one approved bounded WP1 child specification.
- The remaining authorization/specification gates, the Sphinx/client/source
  later-gate clause, the CG-08 cross-reference and the surrounding Publisher
  Services text are preserved.

### 4.5 `docs/engineering/repository-map/control-gaps.md` (CG-08)

- CG-08 remains `OPEN`.
- The first Thoth WP1 entry requirement, `MET-CTRL-01` closure, is now marked
  **satisfied**, recording delivery through PR #833 and reachability from
  `develop`.
- The `feature/metrics` creation and bounded WP1 child specification
  requirements are each marked **outstanding** with the explicit observation
  that no such branch and no such specification exist.
- The closing paragraph now states that the satisfied `MET-CTRL-01` component
  does not close CG-08, and that CG-08 closes only when the whole WP1 entry path
  is complete.
- The sentence preserving CG-03, CG-04, CG-09, CG-10, CG-11 and CG-13 exactly as
  recorded is retained verbatim. Those six gap sections themselves are untouched.
- The later-gates block inside CG-08 is untouched.

### 4.6 `CHANGELOG.md`

- One new bounded `### Added` entry under `## [Unreleased]`, describing the
  material programme-state correction, the two remaining WP1 entry gates, the
  scoped `SPHINX-BOOT-01` dependency change, the CG-08 outcome, the preserved
  later gates, the `ADR-0005` treatment, the preserved historical prior entry,
  the two preserved provenance exceptions, and the explicit statement that #832
  and #766 are untouched and that no `feature/metrics` branch or WP1 child issue
  is created.
- The `ADR-0005` clause records that active trackers use PR #833 as the parent
  lifecycle anchor and do not restate the reviewed source head, merge commit,
  review identifiers or merge-authorization identifiers, and that exact review
  and authorization provenance is retained in the owning `MET-CTRL-01` task and
  closeout evidence rather than asserted to be recorded in full on GitHub. The
  explicit Exception B statement is retained verbatim in the same entry.
- The existing `MET-CTRL-01` entry immediately below is **not modified**. See
  section 7.2.

### 4.7 New files

- `docs/engineering/ai-delivery/tasks/MET-CTRL-01-CLOSEOUT-01.md` - the bounded
  repository task specification, including the parent evidence map, both
  provenance exceptions, the write budget, the per-file required outcome, the
  stale-state sweep definition, non-goals, invariants, acceptance criteria,
  validation, rollout/rollback, stop conditions and the approval boundary.
- This implementation report.

## 5. Write-budget compliance

Approved budget (eight paths) versus actually changed (eight paths):

| Approved path | Changed | Note |
|---|---|---|
| `CHANGELOG.md` | yes | one new entry; prior entry untouched |
| `docs/metrics/README.md` | yes | active stale programme status |
| `docs/metrics/task-status.md` | yes | active stale task/WP state |
| `docs/metrics/rollout-plan.md` | yes | active stale Stage-0 gate |
| `docs/engineering/README.md` | yes | active stale WP1 gate sentence |
| `docs/engineering/repository-map/control-gaps.md` | yes | CG-08 only |
| `docs/engineering/ai-delivery/tasks/MET-CTRL-01-CLOSEOUT-01.md` | yes | new, required |
| `.../MET-CTRL-01-CLOSEOUT-01-implementation-report.md` | yes | new, required |

No file outside the budget was touched. `SCOPE AMENDMENT REQUIRED` was not
triggered: every active stale statement found by the sweep lies inside the
budget.

## 6. Stale-state sweep and classification

### 6.1 Method

The sweep was run over tracked repository content with `git grep`, deliberately
excluding untracked `.claude/worktrees/` scratch copies, which are not
repository content. Patterns searched, per #834:

- `MET-CTRL-01` anywhere in tracked Markdown;
- `MET-CTRL-01` combined with `ACTIVE`, `current`, `awaiting`, `remains open`,
  `closure`, `CHANGES REQUIRED`;
- `833` combined with awaiting/pending merge;
- `under reconciliation`;
- `AWAITING REVIEW` / `awaiting independent review` / `awaiting merge`
  / `merge authorization` phrasing.

### 6.2 `ACTIVE STALE STATE - corrected` (all inside budget)

| Path | Stale statement | Correction |
|---|---|---|
| `docs/metrics/README.md:3` | `Status: PROGRAMME CONTROLS UNDER RECONCILIATION (MET-CTRL-01)` | replaced with a durable reconciled/unauthorized status |
| `docs/metrics/README.md:138-148` | `MET-CTRL-01` "is the current programme-control gate and remains open until its PR is independently approved at its exact head and merged"; "After `MET-CTRL-01` closes..." | `MET-CTRL-01` recorded satisfied among completed controls; exactly two remaining WP1 gates listed |
| `docs/metrics/task-status.md:7-11` | closeout provenance pointed at the parent reconciliation | refreshed to `MET-CTRL-01-CLOSEOUT-01` / #834 |
| `docs/metrics/task-status.md:21` | `MET-CTRL-01 ... ACTIVE ... Remains the current gate until its PR is independently approved at its exact head and merged` | `MERGED - COMPLETE`; dependency satisfied; PR #833 as parent lifecycle anchor |
| `docs/metrics/task-status.md:24` | `SPHINX-BOOT-01` dependencies list bare `MET-CTRL-01` | only that dependency marked `(**satisfied**)` |
| `docs/metrics/task-status.md:35` | WP1 blocked by `MET-CTRL-01 closure` | removed; two remaining gates retained |
| `docs/metrics/task-status.md:50-52` | "WP1 remains blocked only on ... `MET-CTRL-01` closure" | dependency recorded satisfied; two remaining gates |
| `docs/metrics/task-status.md:134-142` | next actions "Close `MET-CTRL-01`" and "After `MET-CTRL-01` closes" | durable satisfied statement plus the two remaining gates |
| `docs/metrics/rollout-plan.md:29` | Stage 0 remaining gate `MET-CTRL-01 closure (independent exact-head approval and merge)` | moved to completed shared/global controls |
| `docs/engineering/README.md:119` | "Thoth WP1 entry now waits only on `MET-CTRL-01` closure, ..." | dependency satisfied; WP1 waits only on the two remaining gates |
| `docs/engineering/repository-map/control-gaps.md:213-215` | CG-08 WP1 entry requires "`MET-CTRL-01` closure ... independently approved at its exact head and merged" | marked **satisfied**; CG-08 stays `OPEN` on the two outstanding items |

### 6.3 `HISTORICAL EVIDENCE - preserved unchanged`

| Path | Why preserved |
|---|---|
| `CHANGELOG.md` prior `MET-CTRL-01` entry | Point-in-time release-note evidence of that task's own gate language. `ADR-0005` section 9 preserves historical records as written; #834 requires it be classified historical, not rewritten. **Explicitly classified HISTORICAL.** |
| `docs/engineering/ai-delivery/tasks/MET-CTRL-01.md` | Point-in-time parent task specification; #834 out of scope. No active contradiction found: it describes the parent task's own pre-merge gates, not current programme state. |
| `docs/engineering/ai-delivery/implementation-reports/MET-CTRL-01-implementation-report.md` | Point-in-time parent implementation evidence, including its record at line 268 that `docs/metrics/README.md` then read `PROGRAMME CONTROLS UNDER RECONCILIATION (MET-CTRL-01)`. That is a true statement about the state at that head. |
| `docs/engineering/ai-delivery/tasks/ADR-0001-APPROVAL.md`, `ADR-0001-POST-MERGE.md`, `ADR-0002-APPROVE.md`, `ADR-0002-POST-MERGE-CORRECTION.md`, `P0-01-CLOSEOUT.md`, `P0-01-FINALIZE.md` and their implementation reports | Each records `MET-CTRL-01` as `CHANGES REQUIRED` **at its own point in time**, as deliberate evidence that those tasks did not opportunistically remediate it. Historical by construction; outside the budget; `ADR-0005` section 9 forbids opportunistic repair. |
| `docs/engineering/ai-delivery/implementation-reports/ADR-0008-RECORD-implementation-report.md` | Same construction: records that `MET-CTRL-01` was deliberately not remediated at that head. |
| `docs/engineering/ai-delivery/tasks/BE-02-CLOSEOUT-01.md`, `BE-03-CLOSEOUT-01.md` and their reports | Publisher Services closeout evidence; their "awaiting review"/"merge authorization" strings describe `BE-02`/`BE-03` state, not Metrics. Unrelated programme. |
| `docs/engineering/ai-delivery/tasks/ADR-01-CLOSEOUT-01.md`, `ADR-01-SPEC-AMEND-01-CLOSEOUT-01.md` | "awaiting merge" strings refer to PR #781, not #833. Unrelated. |
| `docs/engineering/AGENTS.md:37`, `docs/engineering/decisions/ADR-0005-terminal-merge-evidence.md:220` | `AWAITING REVIEW` appears as a prohibited-vocabulary example, not as a status assertion. |

### 6.4 `CURRENT AND CORRECT - preserved unchanged`

| Path | Why |
|---|---|
| `docs/engineering/repository-map/branch-topology.md:206` | "Metrics control task `MET-CTRL-01` is merged" is a durable precondition that is now satisfied, not stale. #834 explicitly excludes this file. |
| `docs/engineering/decisions/ADR-0008-...md:453` | "`MET-CTRL-01` and every other recorded Metrics control debt is untouched by" is a scope statement about `ADR-0008`, true regardless of `MET-CTRL-01`'s status. |
| `docs/metrics/decisions.md`, `docs/metrics/master-issue.md` | Contain no `MET-CTRL-01` status assertion; #834 excludes both. Verified: `git grep -n MET-CTRL-01` returns no hit in either. |
| `docs/metrics/acceptance-matrix.md`, `source-inventory.md`, `contract-register.md`, `migration-inventory.md` | No `MET-CTRL-01` status assertion. |

### 6.5 Remaining hits after correction

Post-edit, every `MET-CTRL-01` hit in the active Metrics/engineering control
surfaces (`docs/metrics/**`, `docs/engineering/README.md`,
`docs/engineering/repository-map/**`) states either that the task is
merged/complete/satisfied or that the satisfied component does not close CG-08.
The targeted pattern sweep for active stale phrasing over those surfaces returns
**no hits**. No remaining active stale hit exists inside or outside the budget.

## 7. ADR-0005 treatment

### 7.1 Why this task is legitimate under ADR-0005

`ADR-0005` section 8 permits a post-merge repository task when "a committed
tracker contains materially incorrect programme state". Six committed active
controls asserted that `MET-CTRL-01` was an unsatisfied programme/WP1 entry
gate. That assertion determines what work is permitted next, so correcting it is
a material programme-state correction, not lifecycle transcription.

The prohibited section-8 forms - "the PR is now merged", "the merge SHA is X",
"review ID was Y", "CTO authorized merge", "the task is complete", "the PR is no
longer draft" - are **not** what this change writes into active trackers.

### 7.2 What was deliberately not written into active trackers

Active trackers record the durable consequence only. They do **not** reproduce:

- the reviewed parent source head `bbe0928b...`;
- the parent merge commit `e62a476e...`;
- the independent review comment identifier `5414236565`;
- any merge-authorization identifier;
- the parent merge timestamp;
- any draft/ready state.

PR #833 is referenced as the parent lifecycle anchor only. Under section 5,
repository documents may reference the pull request without copying every review
identifier or merge timestamp. Active trackers also do **not** assert that the
GitHub record contains the complete review and authorization history: the exact
provenance is retained in the owning `MET-CTRL-01` task and closeout evidence,
including the durable independent review record and the control-plane merge
authorization described in section 8.

The exact SHAs that appear in **this report** - the authorized base, the
preflight observations and the reconciliation commit - are execution evidence
that the repository's own task-reporting controls require, not post-merge
lifecycle transcription into active trackers.

### 7.3 Durable wording

New and rewritten status prose was checked against `ADR-0005` section 6: it must
be truthful before review, after review, before merge and after merge. The
active trackers now assert only facts about `MET-CTRL-01`, which has already
merged, and about the two WP1 gates, which are absent. Nothing on this branch
asserts `PENDING MERGE`, `AWAITING REVIEW` or `AWAITING CTO MERGE AUTHORIZATION`
about this closeout itself.

The task specification's own acceptance criteria for CI, independent review,
merge authorization, #832/#766 synchronization and non-recursion are left
unchecked and explicitly annotated as live gates owned by GitHub and the CTO,
so that reviewing or merging this branch does not itself require another commit
to correct status prose.

### 7.4 Non-recursion confirmation

After this closeout PR merges, **no further repository commit or pull request
will be created solely to record that it merged.** GitHub lifecycle evidence is
terminal. #832 final reconciliation/closure and #766 programme synchronization
are separate post-merge GitHub mutations, not repository commits. A further
bounded repository task would be justified only by a genuinely material,
independently identified repository defect.

This is stated in the changelog entry, in section 10 of the task specification
and here.

## 8. Corrected parent evidence map

Three distinct records carry the `MET-CTRL-01` lifecycle. This closeout
preserves them exactly and does not collapse them:

| Evidence | Record | What it establishes |
|---|---|---|
| Implementation head, CI, ready/merge event, merge commit, merge timestamp | PR [#833](https://github.com/thoth-pub/thoth/pull/833) | terminal GitHub lifecycle evidence |
| Independent exact-head `APPROVED` source review of `bbe0928b96d8475a38628cdf1fd08455da418d83` | #832 comment `5414236565` | the durable independent review record |
| CTO merge authorization bound to `bbe0928b96d8475a38628cdf1fd08455da418d83` | control-plane conversation, before merge execution | authorization existed before the merge |

PR #833 is **not** asserted to contain the independent review record or the
merge-authorization record. No active tracker on this branch makes that claim.

## 9. Historical control-process provenance exceptions

Both parent exceptions are preserved distinctly. Neither is erased, backdated,
reconstructed as a pre-existing GitHub record, or presented as retroactively
cured by this closeout.

### 9.1 Exception A - implementation authorization

Parent implementation mutations - branch creation, documentation edits, commit,
push and draft-PR creation - occurred **before** a separate explicit
implementation authorization was durably recorded.

This exception is recorded in the parent's own historical task and
implementation report, which this closeout leaves unchanged, and is restated in
section 2.2 of the `MET-CTRL-01-CLOSEOUT-01` task specification.

### 9.2 Exception B - merge-authorization durable ledger

The CTO **explicitly authorized** the merge of exact head
`bbe0928b96d8475a38628cdf1fd08455da418d83` in the control-plane conversation
**before** the merge was executed. That authorization was **not durably recorded
on PR #833 or #832 before execution**.

Exception B is a missing durable GitHub record, not a missing authorization.
This closeout therefore does **not**:

- state or imply that the merge was unauthorized;
- create, backdate or reconstruct a pre-merge GitHub authorization record;
- post any comment on #832 or #833 purporting to supply one;
- describe either exception as cured.

### 9.3 Distinctness

A and B are separate defects at separate lifecycle stages: A concerns
implementation authorization provenance, B concerns merge-authorization ledger
provenance. They are recorded separately in the task specification (section 2.2),
in the changelog entry and here.

## 10. Effects

| Dimension | Effect |
|---|---|
| Migration | NONE. No migration added, altered, reordered or executed. |
| Data | NONE. No data read, written, backfilled or deleted. |
| Schema / `thoth-api/src/schema.rs` | NONE. Untouched. |
| Rust / SQL | NONE. No `.rs`, `.sql` or `Cargo.*` file changed. |
| GraphQL / API | NONE. No schema, resolver, type, input or generated contract changed. |
| Auth / security / policy | NONE. No role, permission, policy guard or authorization matrix changed. `ADR-0008` still selects no Metrics role name, permission set, credential model or entitlement model. |
| Workflows / branch protection | NONE. `.github/**` untouched; no CI dispatched or re-run manually. |
| Provider / runtime | NONE. No provider, runtime or production environment was read or written. |
| Deployment / release / production activation | NONE. None performed or authorized. |
| Client repositories (Sphinx, dashboard, widget, app) | NONE. Untouched. |
| Programme authorization | Corrected only: `MET-CTRL-01` satisfied. WP1 remains blocked and unauthorized; no new work is authorized. |

## 11. Validation

### 11.1 Commands and results

```bash
git status --short
```

Before commit: exactly the six modified budget files plus the two new budget
files. After commit: clean.

```bash
git diff --check
```

No output. No whitespace or conflict-marker error.

```bash
git diff --stat e62a476e87916d0b44a963652c9c5e7ab6afa10e...HEAD
git diff --name-only e62a476e87916d0b44a963652c9c5e7ab6afa10e...HEAD
```

Eight files, all inside the approved budget. Exact stat is recorded on the pull
request.

```bash
grep -nE '^### CG-(03|04|09|10|11|13)' docs/engineering/repository-map/control-gaps.md
```

All six unrelated gap sections still present:

```text
### CG-03 - `thoth-sphinx` is bootstrap-only
### CG-04 - Branch topology differs
### CG-09 - Source fixtures/mappings incomplete
### CG-10 - OPERAS inbound completeness unavailable
### CG-11 - CI gaps
### CG-13 - Thoth runtime operations unmapped
```

CG-08 is still present and still `OPEN`.

```bash
git branch -a --list '*feature/metrics'
```

No output. No `feature/metrics` branch exists locally or on the remote as a
result of this task.

### 11.2 Path containment

No changed path matches `*.rs`, `*.sql`, `migrations/**`, `.github/**`,
`thoth-api/src/schema.rs`, any generated contract, or any client repository.
The complete changed set is the eight Markdown paths in section 4.

### 11.3 Stale-state re-run

The active-control pattern sweep of section 6.1 was re-run after editing over
`docs/metrics/**`, `docs/engineering/README.md` and
`docs/engineering/repository-map/**`. It returned no hits.

### 11.4 Changelog

The new entry is present under `## [Unreleased]` / `### Added`. The prior
`MET-CTRL-01` entry immediately below is byte-identical to its state at
`e62a476e87916d0b44a963652c9c5e7ab6afa10e` and is classified **HISTORICAL** in
section 6.3.

### 11.5 Untouched-file verification

`docs/metrics/decisions.md`, `docs/metrics/master-issue.md`,
`docs/engineering/decisions/ADR-0005-terminal-merge-evidence.md`,
`docs/engineering/repository-map/branch-topology.md`,
`docs/engineering/ai-delivery/tasks/MET-CTRL-01.md` and
`docs/engineering/ai-delivery/implementation-reports/MET-CTRL-01-implementation-report.md`
do not appear in `git diff --name-only`.

## 12. Independent review remediation - `CHANGES REQUIRED`

### 12.1 Finding

An independent review of exact head
`46dd669ed0ac440a9cf33aa2e1028940253fe80f` returned `CHANGES REQUIRED`.

The finding is that the source **overstated where parent lifecycle evidence
lives**. Three records asserted, in materially identical terms, that live
review, authorization and merge evidence is the GitHub pull-request and issue
history:

| Record | Wording found |
|---|---|
| `docs/metrics/README.md` | "Live review, authorization and merge evidence is the GitHub pull-request and issue history; this record does not restate it." |
| `docs/metrics/task-status.md` | materially identical wording in the `MET-CTRL-01` dependency cell |
| `CHANGELOG.md` | the new entry left the reviewed-head, merge, review and merge-authorization identifiers "to GitHub as terminal evidence" |

That wording is incorrect for this parent. The CTO merge authorization for
`MET-CTRL-01` existed in the control-plane record **before** merge execution but
was **not durably recorded on GitHub before merge**. Asserting that GitHub holds
the review, authorization and merge evidence therefore implies that GitHub
contains a record which does not exist, and silently contradicts Exception B in
section 9.2.

### 12.2 Remediation

One additive remediation commit, inside the existing eight-path #834 write
budget. No amend, no rebase, no force-push. No GitHub record was mutated.

`docs/metrics/README.md` and `docs/metrics/task-status.md` now read, in the
`MET-CTRL-01` completed-control bullet and dependency cell respectively:

> PR #833 is the parent lifecycle anchor; exact review and authorization
> provenance is retained in the owning task and closeout evidence, and this
> active tracker does not restate it.

`CHANGELOG.md` now records that active trackers reference PR #833 as the parent
lifecycle anchor and do not restate the reviewed source head, merge commit,
review identifiers or merge-authorization identifiers, and that exact review and
authorization provenance is retained in the owning `MET-CTRL-01` task and
closeout evidence **and is not asserted to be recorded in full on GitHub**. The
explicit Exception B statement in the same entry is retained verbatim.

### 12.3 Durable semantics now carried

| Required semantic | Where carried after remediation |
|---|---|
| PR #833 is the parent lifecycle anchor | `docs/metrics/README.md`, `docs/metrics/task-status.md`, `CHANGELOG.md`, section 8 |
| #832 comment `5414236565` is the durable independent exact-head `APPROVED` review | section 8 and the task specification section 2.1 - deliberately not in active trackers |
| CTO merge authorization existed before execution in the control-plane record | section 8, section 9.2, task specification sections 2.1 and 2.2 |
| that merge authorization was **not** durably recorded on GitHub before merge | section 9.2, task specification section 2.2, `CHANGELOG.md` Exception B clause |
| the missing GitHub record remains a historical provenance exception, uncured | section 9, `CHANGELOG.md` |
| active trackers need not restate exact lifecycle identifiers | sections 7.2 and 12.2 |
| nothing implies GitHub contains the missing authorization record | corrected wording in all three records |

The corrected parent evidence map in section 8 and both provenance exceptions in
section 9 are preserved unchanged by this remediation; the remediation removes
an active-tracker assertion that contradicted them, and adds no new one.

### 12.4 Files changed by the remediation commit

```text
CHANGELOG.md
docs/engineering/ai-delivery/implementation-reports/MET-CTRL-01-CLOSEOUT-01-implementation-report.md
docs/metrics/README.md
docs/metrics/task-status.md
```

All four are inside the approved eight-path #834 write budget. The task
specification, `docs/metrics/rollout-plan.md`, `docs/engineering/README.md` and
`docs/engineering/repository-map/control-gaps.md` needed no change: none carried
the overstated wording.

### 12.5 Non-recursion

This remediation records a genuine independently identified defect in repository
content, not the lifecycle of the branch that carries it. The `<head>`
placeholder previously left in section 3 is replaced with durable wording rather
than a self-SHA, and no commit is created solely to transcribe its own SHA.

### 12.6 Remediation validation

```bash
git diff --check
```

No output.

```bash
git diff --name-only e62a476e87916d0b44a963652c9c5e7ab6afa10e...HEAD
```

The same eight budget paths as section 4; the remediation adds no path.

```bash
git status --short
```

Clean after commit.

```bash
grep -rn 'Live review, authorization and merge evidence' docs/metrics/ CHANGELOG.md
grep -rn 'terminal evidence' CHANGELOG.md
```

No hits in `docs/metrics/**` or in the `MET-CTRL-01-CLOSEOUT-01` changelog
entry. Unrelated Publisher-Services and engineering-control documents that
legitimately use the phrase for their own tasks are outside this budget and were
not touched.

```bash
git branch -a --list '*feature/metrics'
```

No output. No `feature/metrics` branch exists.

No `.rs`, `.sql`, `migrations/**`, `.github/**`, `schema.rs`, generated contract
or client-repository path is in the changed set, so the remediation has no
runtime, migration, authorization or provider effect. Issues #832 and #766, PR
#833 and all other GitHub records are untouched.

## 13. CI

Only the normal automatic CI triggered by branch push and draft-PR creation was
allowed to run. No workflow was dispatched, re-run or modified. Live CI state is
the GitHub pull-request record; this report does not transcribe it.

## 14. Rollout and rollback

Rollout is unchanged from the approved specification: bounded task branch, one
draft PR to `develop`, automatic CI, fresh independent exact-head review,
explicit CTO merge authorization durably recorded before merge and bound to the
exact reviewed head, then merge. After merge, #832 and #766 are reconciled
through separate GitHub mutations. `feature/metrics` and the first bounded WP1
child task are separately authorized only after durable closeout.

Rollback is a revert of this documentation/control merge. It restores the prior
active-tracker wording and creates no runtime, data or migration effect.

## 15. Deviations and limitations

1. **No deviation from the approved scope.** All eight approved paths were used;
   no path outside the budget was touched; `SCOPE AMENDMENT REQUIRED` was not
   triggered.
2. **Exception B rests on a control-plane record.** Its evidence is the
   control-plane conversation, not GitHub. That is the exception itself. This
   report records it as stated in #834 and its authorization comment, and does
   not independently verify a record it is definitionally impossible to find on
   GitHub, nor create one.
3. **Sweep boundary.** The sweep covers tracked repository content. Untracked
   `.claude/worktrees/` scratch copies contain older Metrics text but are not
   repository content, are not on this branch, and were deliberately excluded.
4. **Live gates are not repository state.** CI, independent review, merge
   authorization, #832/#766 synchronization and the non-recursion undertaking
   remain unchecked acceptance criteria owned by GitHub and the CTO.
5. **Self-approval prohibited.** The implementing agent has not reviewed or
   approved this work.
6. **One review-driven remediation.** An independent review of
   `46dd669ed0ac440a9cf33aa2e1028940253fe80f` returned `CHANGES REQUIRED` on the
   overstated location of parent lifecycle evidence. It was remediated additively
   inside the existing write budget, as recorded in section 12. History was not
   rewritten and no GitHub record was mutated.

## 16. Remaining gates

Immediately for this task:

1. normal documentation/control PR CI green at the exact reviewed head;
2. fresh independent exact-head review returning `APPROVED`;
3. explicit CTO merge authorization bound to the final reviewed head and
   **durably recorded before merge** - which, given Exception B, matters
   particularly here;
4. merge;
5. separate post-merge GitHub mutations: #832 final reconciliation/closure and
   #766 programme synchronization.

Then, for the Metrics programme:

6. separately authorized creation of repository-local `feature/metrics` from a
   freshly verified `develop` head;
7. one approved bounded repository-local WP1 child specification.

Neither 6 nor 7 exists, and neither is authorized by this task. WP1 remains
`BLOCKED`, unimplemented and unauthorized, and all later Sphinx/WP6, client,
source-specific/COUNTER/OPERAS and WP5 gates remain attached to their owning
later work.

## 17. Proposed GitHub comments - TEXT ONLY, NOT POSTED

Neither comment below has been posted. Both require separate explicit
authorization as post-merge GitHub mutations. They are recorded here as text
only.

### 17.1 Proposed final #832 reconciliation/closure comment

````markdown
## MET-CTRL-01 - final reconciliation and closure

**Task:** `MET-CTRL-01`
**PR:** #833 (merged)
**Closeout task:** `MET-CTRL-01-CLOSEOUT-01` (#834)

### Outcome

`MET-CTRL-01` is **MERGED - COMPLETE**. The programme-control reconciliation was
delivered through PR #833 and is reachable from `develop`.

The bounded post-merge closeout `MET-CTRL-01-CLOSEOUT-01` (#834) has merged. Active
repository controls now record `MET-CTRL-01` as satisfied and no longer assert that
it is an unsatisfied Thoth WP1 entry gate:

- `docs/metrics/README.md` - programme status reconciled; `MET-CTRL-01` recorded
  among completed shared/global controls;
- `docs/metrics/task-status.md` - `MET-CTRL-01` is `MERGED - COMPLETE`; the WP1
  dependency is removed; the `SPHINX-BOOT-01` dependency on `MET-CTRL-01` is marked
  satisfied while `BR-SPHINX-01`, the approved bootstrap specification and every
  other Sphinx/WP6 blocker remain intact;
- `docs/metrics/rollout-plan.md` - `MET-CTRL-01` moved to completed Stage-0
  controls;
- `docs/engineering/README.md` - WP1 no longer waits on `MET-CTRL-01` closure;
- `docs/engineering/repository-map/control-gaps.md` - CG-08 records the satisfied
  `MET-CTRL-01` component and remains **OPEN**; CG-03, CG-04, CG-09, CG-10, CG-11
  and CG-13 are unchanged.

### Evidence map

- PR #833 is terminal GitHub evidence for the implementation head, CI, the
  ready/merge event, the merge commit and the merge timestamp.
- Comment 5414236565 on this issue is the durable independent exact-head `APPROVED`
  source-review record for `bbe0928b96d8475a38628cdf1fd08455da418d83`.
- CTO merge authorization was explicitly granted in the control-plane conversation,
  bound to that exact head, before the merge was executed.

### Historical control-process provenance exceptions - preserved, not cured

Both remain on the record and neither is erased, backdated or retroactively cured:

- **A - implementation authorization.** Implementation mutations occurred before a
  separate explicit implementation authorization was durably recorded.
- **B - merge-authorization ledger.** Merge was explicitly authorized before
  execution, but that authorization was not durably recorded on PR #833 or this
  issue before the merge. This is a missing durable record, **not** a missing
  authorization, and no pre-merge GitHub authorization record has been created,
  backdated or reconstructed.

### Effects

No runtime, migration, data, schema, GraphQL/API, authorization, workflow, provider,
deployment or production effect. No Metrics implementation is authorized.

### Remaining Thoth WP1 entry gates

1. separately authorized creation of repository-local `feature/metrics` from a
   freshly verified `develop` head;
2. one approved bounded repository-local WP1 child specification.

Neither exists. WP1 remains **BLOCKED**, unimplemented and unauthorized.

### Non-recursion

Under `ADR-0005`, GitHub lifecycle evidence is terminal. No further repository task
or pull request will be created solely to record that the closeout merged.

`MET-CTRL-01` is closed.
````

### 17.2 Proposed #766 programme synchronization comment

````markdown
## Thoth Metrics - programme synchronization after MET-CTRL-01 closeout

**Programme:** Thoth Metrics - canonical ingestion, Sphinx orchestration and client
cutover
**Trigger:** `MET-CTRL-01` (#832) closed; `MET-CTRL-01-CLOSEOUT-01` (#834) merged

### Programme decision - unchanged

```text
BLOCKED FOR IMPLEMENTATION
```

No Metrics implementation is authorized.

### What changed

The Metrics programme controls are reconciled. `MET-CTRL-01` is
`MERGED - COMPLETE` and its dependency is **satisfied**; it is no longer a Thoth WP1
entry gate, and active repository controls no longer describe the programme as being
under `MET-CTRL-01` reconciliation.

### Completed shared/global controls

- engineering-control foundation closed (P0-01, merged closeout PR #767);
- `ADR-0001` package capabilities - approved and merged;
- `ADR-0002` platform domain boundaries - approved and merged; `MetricPlatform`
  remains separate from `DistributionPlatform`;
- `ADR-0003` repository-authoritative schema contract - authoritative through merged
  PR #778; CG-12 resolved; `THOTH-DB-CTRL-01` superseded;
- `ADR-0008` machine roles and durable job primitives - authoritative within its
  approved scope, selecting no Metrics role name, permission set, credential model
  or entitlement model;
- `MET-CTRL-01` programme controls - merged through PR #833.

### Remaining Thoth WP1 entry gates - exactly two

1. separately authorized creation of repository-local `feature/metrics` from a
   freshly verified `develop` head;
2. one approved bounded repository-local WP1 child specification.

Neither exists and neither is authorized by the closeout. WP1 remains `HIGH` and
`BLOCKED`.

### Later gates - unchanged and still owned by their later work

- `BR-SPHINX-01` and `SPHINX-BOOT-01` gate WP6 and later Sphinx work.
  `SPHINX-BOOT-01` has only its `MET-CTRL-01` dependency marked satisfied; it
  remains `BLOCKED` on `BR-SPHINX-01` and its approved bootstrap specification.
- Client branch/CI readiness (`BR-DASH-01`, `BR-WIDGET-01`, `BR-APP-01`) gates the
  client-dependent work packages.
- Representative source fixtures, COUNTER mappings and guaranteed OPERAS inbound
  discovery gate the applicable source/driver/inbound work.
- Exact Metrics service-role codes, permissions and credential/provisioning
  arrangements remain unapproved WP5-owned bounded decisions.

### Control gaps

CG-08 remains **OPEN** on the two outstanding WP1 entry items. CG-03, CG-04, CG-09,
CG-10, CG-11 and CG-13 are neither weakened nor closed.

### Effects

Documentation and control records only: no runtime, migration, data, schema,
GraphQL/API, authorization, workflow, provider, deployment or production effect.

### Next authorized step

None automatically. The next step is a separate explicit authorization to create
`feature/metrics` from a freshly verified `develop` head, followed by scoping and
approving one bounded WP1 child specification.
````

## 18. Agent self-assessment

The implementing agent cannot review or approve its own work. This closeout
requires fresh independent exact-head review and separate explicit CTO merge
authorization, durably recorded before merge. The remediation in section 12
changed the branch head, so the earlier independent review no longer applies to
the current content and a fresh independent exact-head review is required.
