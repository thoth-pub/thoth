# P0-01-CLOSEOUT Implementation Report

## 1. Repository state

Repository: `thoth-pub/thoth`
Workflow: STANDARD
Base branch: `develop`
Base commit: `5b406e4ef9b5c192cc38eb8a97a41bbd0fc3bc06`
PR target: `develop`
Programme integration branch: None
Task branch: `feature/publisher-services/p0-01-closeout`
Implementation content commit: `3d51e7a1`
Original independently reviewed PR head:
`4f5c3491815e8d5ce4d1d6b15f316508494e503e`
First independent-review decision: `CHANGES REQUIRED`
Review Cycle 1 reviewer (historical): Claude, separate context, high reasoning
Scope Amendment 1 commit: `425eab61`
Shared-status remediation commit: `08411cfe`
Review Cycle 2 decision: `CHANGES REQUIRED`
Review Cycle 2 reviewer context: fresh Codex context, not the assigned Claude
reviewer
Review Cycle 2 reviewed head:
`d55ef26a0cc29d28d9c7d69ecbce60eb0082146e`
Scope Amendment 2 commit: `2348f130`
Scope Amendment 2 remediation commit: `8c76b3ca`
Procedural Review Cycle 3 decision: `BLOCKED`
Procedural Review Cycle 3 reviewed head:
`00988232e40f0357d002ede998fbc31d149ed27f`
Procedural Review Cycle 3 reviewing model: Codex
Procedural Review Cycle 3 substantive assessment: every substantive criterion
passed; the sole blocker was the then-active named-Claude reviewer requirement
Scope Amendment 3: approved by Javi, CTO on 2026-07-24; replaces the named-model
requirement with capability-based independence criteria
Reviewed content head (PR #767): `d72137893ddea512c0d05c81d310eb59d045cd2b`
Final independent-review decision: `APPROVED` (fresh non-implementing context,
high reasoning)
Closeout merge commit: `bac598e32abbd0d7e69ff467c82945ee00df02ba`
Closeout merged at: `2026-07-27T09:29:57Z`
Post-merge finalization: task P0-01-FINALIZE recorded this concrete final-head,
review, CI and merge evidence in a separate finalization commit and PR (see
[`P0-01-FINALIZE-implementation-report.md`](./P0-01-FINALIZE-implementation-report.md)).
Recording it separately, rather than inside PR #767 itself, avoids a
self-referential loop and does not alter the content reviewed and merged through
PR #767.
Pull request: [#767](https://github.com/thoth-pub/thoth/pull/767)
Expected branch deletion after merge: YES
Final programme PR required: NO
Implementing model: Codex
Reasoning level: High

## 2. Scope confirmation

Approved specification:
[`docs/engineering/ai-delivery/tasks/P0-01-CLOSEOUT.md`](../tasks/P0-01-CLOSEOUT.md)

Implemented objective: reconcile the Publisher Services programme-control
record with the merged P0-01 foundation while keeping independent review,
architecture, inventory, branch-readiness and production gates explicit.

Out-of-scope changes made: NONE

## 3. Commits

- `c9b00e14` - docs: approve Publisher Services P0-01 closeout task
- `3d51e7a1` - docs: reconcile Publisher Services foundation status
- `cd92ca4c` - docs: record P0-01 closeout verification
- `4f5c3491` - docs: link Publisher Services closeout PR
- `425eab61` - docs: approve P0-01 closeout remediation scope
- `08411cfe` - docs: reconcile merged foundation control status
- `d55ef26a` - docs: correct P0-01 closeout review evidence
- `2348f130` - docs: approve final P0-01 remediation scope
- `8c76b3ca` - docs: complete P0-01 closeout remediation
- `00988232` - docs: record final P0-01 remediation evidence

The Scope Amendment 3 commit containing this updated report is recorded
externally in PR #767, together with its exact final-head CI.

## 4. Files changed

- `docs/engineering/ai-delivery/tasks/P0-01-CLOSEOUT.md`
  - reason: commit the CTO-approved task specification before any other edit;
  - behavioural effect: none.
- `docs/publisher-services/README.md`
  - reason: replace the stale unmerged-foundation statement with the factual
    merged state and remaining blockers;
  - behavioural effect: none.
- `docs/publisher-services/task-status.md`
  - reason: record P0-01 deliverables as `MERGED`, identify the exact merge
    commit, and separate merged deliverables from outstanding closeout evidence;
  - behavioural effect: none.
- `docs/publisher-services/rollout-plan.md`
  - reason: separate achieved Stage 0 evidence from outstanding review, ADR,
    inventory and branch-readiness evidence;
  - behavioural effect: none.
- `CHANGELOG.md`
  - reason: record this bounded documentation/control change under
    `Unreleased`;
  - behavioural effect: none.
- `docs/engineering/ai-delivery/implementation-reports/P0-01-CLOSEOUT-implementation-report.md`
  - reason: capture implementation, verification, CI, issue synchronization
    and review evidence;
  - behavioural effect: none.
- `docs/engineering/README.md`
  - reason: distinguish the merged foundation evidence from outstanding
    retrospective remediation and approval;
  - behavioural effect: none.
- `docs/engineering/repository-map/control-gaps.md`
  - reason: record PR #764's factual merge and retrospective review state;
  - behavioural effect: none.
- `docs/engineering/ai-delivery/implementation-reports/CTRL-FOUNDATION-01-implementation-report.md`
  - reason: correct the inaccurate implication that independent review occurred
    before PR #764 merged;
  - behavioural effect: none.
- `docs/metrics/README.md`
  - reason: correct only the shared foundation merge-state blocker;
  - behavioural effect: none.
- `docs/metrics/task-status.md`
  - reason: record `MET-CTRL-01` as `CHANGES REQUIRED` and correct the
    ADR-0001/ADR-0002 provenance coordinates without advancing any Metrics
    implementation task;
  - behavioural effect: none.
- `docs/engineering/agent-instructions/rollout-plan.md`
  - reason: replace active instructions to merge the already-merged PR #764
    foundation with the factual PR #767 closeout action;
  - behavioural effect: none.

Reviewed without modification because no concrete inconsistency was found:

- `docs/publisher-services/decisions.md`
- `docs/publisher-services/platform-inventory.md`
- `docs/publisher-services/acceptance-matrix.md`
- `docs/engineering/decisions/ADR-0001-publisher-package-capability-model.md`
- `docs/engineering/decisions/ADR-0002-platform-domain-boundaries.md`

## 5. Implementation decisions

1. Record P0-01 as `MERGED`, not `CLOSED`, because PR #764 merged but no
   independent approval is recorded and the closeout PR has not merged.
2. Keep all implementation tasks blocked and distinguish their architecture,
   final-inventory and branch-readiness gates from the completed foundation
   merge.
3. Preserve the platform inventory as a verified baseline rather than a final
   enum.
4. Propose, but do not apply, the exact issue #765 synchronization in this
   report.
5. Apply the CTO-approved Scope Amendment 1 only to factual shared foundation
   status and review evidence. Publisher Services and Metrics implementation
   scopes remain distinct.
6. Apply the CTO-approved Scope Amendment 2 only to the remaining active
   foundation-status statements, factual Metrics ADR provenance and the
   synchronization guard inside the exact proposed issue body.
7. Apply the CTO-approved Scope Amendment 3 only to replace the named-model
   reviewer requirement with capability-based independence criteria and record
   the procedural `BLOCKED` review at
   `00988232e40f0357d002ede998fbc31d149ed27f`.

Deviation from the specification: NONE after approved Scope Amendments 1, 2 and
3.

### Scope Amendment 1 and first-review remediation

Scope Amendment 1 was approved by Javi, CTO on 2026-07-24 after the first
independent review returned `CHANGES REQUIRED`. Risk remains LOW.

The four P1 findings and their remediation are:

1. Stale shared merge state: engineering and Metrics controls now record that
   PR #764 merged, while remediation and fresh independent approval remain
   outstanding.
2. Inaccurate prior-review implication: the foundation report now states that
   link checking occurred during implementation verification and independent
   review remained outstanding at merge.
3. Unsafe issue replacement: the proposed issue #765 body now preserves the
   complete live authority section and changes only the permitted P0-01 state.
4. Inexact closeout evidence: this report now records the four original commits,
   original reviewed head, first review decision, reviewer, baseline CI, scope
   amendment and remediation commits.

No runtime, migration, API, authorization, deployment, release or production
scope was added.

### Scope Amendment 2 and Review Cycle 2 remediation

Review Cycle 2 used a fresh Codex context and reviewed head
`d55ef26a0cc29d28d9c7d69ecbce60eb0082146e`. It returned
`CHANGES REQUIRED`; it was not a Claude review and did not approve PR #767.
Javi, CTO approved Scope Amendment 2 on 2026-07-24. Risk remains LOW.

The two P1 findings and their remediation are:

1. Remaining stale foundation instructions: the agent-instruction rollout plan
   now records that PR #764 merged as
   `5b406e4ef9b5c192cc38eb8a97a41bbd0fc3bc06` and directs agents to complete
   PR #767 remediation, fresh independent approval and CTO closeout merge. The
   Metrics tracker now records ADR-0001 and ADR-0002 as proposals present on
   `develop` via merged PR #764, while both remain `PROPOSED`, dependent on a CTO
   decision, and all Metrics work packages remain `BLOCKED`.
2. Missing in-body synchronization guard: the exact proposed issue #765 body
   now requires the complete live body and `updatedAt` to be re-fetched and
   compared with the reviewed `2026-07-24T17:17:09Z` baseline before any write.
   A mismatch requires the replacement to stop, be regenerated as a minimal
   diff and receive fresh independent review.

Review Cycle 2 remains historical and did not approve PR #767. Scope Amendment
3 replaces its then-active named-Claude final-review requirement with
capability-based independence criteria.

### Scope Amendment 3 and Procedural Review Cycle 3

Procedural Review Cycle 3 used a fresh Codex context and reviewed exact head
`00988232e40f0357d002ede998fbc31d149ed27f`. It returned `BLOCKED`.

That review verified:

* all six prior P1 findings were fully resolved;
* all 12 cumulative changed files were authorized;
* both previous `CHANGES REQUIRED` decisions and reviewer contexts were
  represented accurately;
* the private design was current at Drive revision `3` with status
  `Approved for phased implementation`;
* issue #765 still matched baseline `2026-07-24T17:17:09Z`, and the exact
  proposed replacement preserved all ten original authority links, all 23
  checkbox rows and every unrelated control;
* the synchronization guard protected both the complete body and `updatedAt`;
* all required final-head CI passed;
* there was no runtime, database, migration, GraphQL/API, authorization,
  workflow, deployment, release or production effect.

The sole blocker was that the approved task still named Claude while the
available reviewer was Codex. Javi, CTO approved Scope Amendment 3 on
2026-07-24 to make reviewer independence, evidence access and fresh
non-implementing context the requirements. Claude is no longer mandatory.

The replacement reviewer must:

- not have implemented any part of PR #764 or PR #767;
- not have authored any remediation commit under review;
- not rely on private chain-of-thought or hidden implementation context from
  the implementing agent;
- use high reasoning effort;
- directly inspect the complete repository evidence, PR #764 and PR #767, CI,
  issue #765, and design revision 3;
- review the actual evidence rather than an implementing-agent summary;
- return exactly one verdict: `APPROVED`, `CHANGES REQUIRED`, or `BLOCKED`; and
- not modify, approve, or merge the same work.

Neither prior Codex review becomes final approval retrospectively. The context
implementing Scope Amendment 3 is not eligible to approve its own amendment. A
new exact-head independent review remains required after the amendment commit.
The external final-head evidence model remains unchanged: the exact amendment
head and complete final-head CI are recorded in PR #767 after CI completes.

## 6. Database and migration effects

Migration added: NO

- schema effect: none;
- existing-data effect: none;
- locking/downtime: none;
- backfill: none;
- generated schema: unchanged;
- rollback: revert the documentation PR.

## 7. API and compatibility effects

GraphQL/API changes: NONE
Generated schema/client updates: NONE
Backwards compatibility: no effect
Deprecations: NONE
Cross-repository dependencies: NONE changed

## 8. Authorization and security

Authorization paths changed: NONE
Roles/scopes involved: NONE
Negative authorization tests: not applicable
Secret or personal-data handling: no secrets or personal source data accessed
Security limitations: none introduced

## 9. Tests and checks

### Formatting

Command:

```text
git diff --check \
  5b406e4ef9b5c192cc38eb8a97a41bbd0fc3bc06...HEAD
```

Result:

```text
No output; exit status 0 for the complete amended branch diff.
```

### Unit tests

Command:

```text
Not applicable: documentation-only change.
```

Result:

```text
No unit-test surface changed.
```

### Integration/database tests

Command:

```text
Not applicable: no runtime, database, migration or integration change.
```

Result:

```text
No integration/database-test surface changed.
```

### Lint/static analysis

Command:

```text
Not applicable: no Rust, GraphQL, SQL, generated contract or workflow change.
```

Result:

```text
No code/static-analysis surface changed.
```

### Other required checks

Commands:

```text
git diff --name-only \
  5b406e4ef9b5c192cc38eb8a97a41bbd0fc3bc06...HEAD

grep -RniE \
  'independently review and merge|Approve and merge the `thoth` control foundation|PR #764 -> `develop`|foundation is unmerged|merge PR #764 into `develop`|Required:.*merge into `develop`|independent review; merge' \
  docs \
  || true

grep -Rni \
  'checked during the independent review' \
  docs \
  || true

grep -n '^Status: PROPOSED$' \
  docs/engineering/decisions/ADR-0001-publisher-package-capability-model.md \
  docs/engineering/decisions/ADR-0002-platform-domain-boundaries.md

grep -n 'FINAL ENUM NOT APPROVED' \
  docs/publisher-services/platform-inventory.md

git diff --name-only \
  5b406e4ef9b5c192cc38eb8a97a41bbd0fc3bc06...HEAD \
  | grep -Ev '^(CHANGELOG\.md|docs/)' \
  && exit 1 || true
```

Result:

```text
Changed files:
CHANGELOG.md
docs/engineering/README.md
docs/engineering/agent-instructions/rollout-plan.md
docs/engineering/ai-delivery/implementation-reports/CTRL-FOUNDATION-01-implementation-report.md
docs/engineering/ai-delivery/implementation-reports/P0-01-CLOSEOUT-implementation-report.md
docs/engineering/ai-delivery/tasks/P0-01-CLOSEOUT.md
docs/engineering/repository-map/control-gaps.md
docs/metrics/README.md
docs/metrics/task-status.md
docs/publisher-services/README.md
docs/publisher-services/rollout-plan.md
docs/publisher-services/task-status.md

Stale-wording search: one match in this report's quoted verification command;
no active control statement matches.
Inaccurate-review search: one match in this report's quoted verification
command; no active inaccurate historical-review claim matches.
ADR-0001: line 3, Status: PROPOSED.
ADR-0002: line 3, Status: PROPOSED.
Platform inventory: line 3, Status: VERIFIED BASELINE; FINAL ENUM NOT APPROVED.
Reviewed-only file diff: no output.
Runtime-surface diff: no output.
Changed-file allowlist: exact match.
Issue proposal: all ten authority links and all non-P0 gates/tasks preserved;
one PR #767 authority link added; all 23 checkbox rows retained; only the P0-01
gate and task were checked; synchronization guard present inside the exact body
with the reviewed timestamp, complete-body re-fetch, `updatedAt` comparison and
stop/regenerate/re-review instructions.
Relative links:
docs/publisher-services/README.md: ../engineering/design-references.md -> OK
docs/engineering/ai-delivery/implementation-reports/P0-01-CLOSEOUT-implementation-report.md: ../tasks/P0-01-CLOSEOUT.md -> OK
```

## 10. Manual verification

Environment: local clean branch created from the verified `origin/develop`
commit, plus read-only GitHub and Google Drive evidence.

Steps:

1. fetched `origin` and verified `origin/develop`;
2. inspected PR #764 state, exact head, merge commit, reviews and checks;
3. inspected issue #765 state, timestamp and complete body;
4. verified the private design's current Drive revision;
5. inspected the complete closeout diff and changed-file list;
6. checked the statuses of ADR-0001, ADR-0002 and the platform inventory;
7. recorded Scope Amendment 1 before editing its newly authorized files;
8. compared the proposed issue body with the captured live body for a minimal
   semantic diff;
9. re-fetched the complete live issue #765 body and `updatedAt` and confirmed
   that both still matched the reviewed baseline;
10. recorded Scope Amendment 2 before editing its newly authorized
    agent-instruction rollout-plan file;
11. ran the repository-wide stale-state search and manually inspected every
    result.

Observed result:

- `origin/develop` was
  `5b406e4ef9b5c192cc38eb8a97a41bbd0fc3bc06`;
- PR #764 final head was
  `b5b1622e54cb3c6fb372dcf02366c8dc4e38654e`;
- PR #764 merged into `develop` as
  `5b406e4ef9b5c192cc38eb8a97a41bbd0fc3bc06` at
  `2026-07-24T17:27:37Z`;
- PR #764 `reviewDecision` was `REVIEW_REQUIRED` with no submitted reviews;
- PR #764 checks `build`, `build_and_push_staging_docker_image`,
  `check-changelog`, `format_check`, `lint`, `run_migrations` and `test` all
  reported `pass`;
- issue #765 was `OPEN`, last updated `2026-07-24T17:17:09Z`;
- the live issue #765 body and `updatedAt` still matched the reviewed baseline
  when Scope Amendment 2 remediation was performed;
- the private design's current Drive revision was `3`, modified
  `2026-07-23T20:32:36.556Z`;
- ADR-0001 and ADR-0002 remained `PROPOSED`;
- the inventory remained `VERIFIED BASELINE; FINAL ENUM NOT APPROVED`.

Evidence:

- [PR #764](https://github.com/thoth-pub/thoth/pull/764)
- [issue #765](https://github.com/thoth-pub/thoth/issues/765)
- [Publisher Services design revision 3](https://docs.google.com/document/d/1kr2Ft0Y4pxgcXGyFAKs_wfFx4I0jlxEvaceswE5Dus8/edit)

### Issue #765 pre-change snapshot

Captured at issue `updatedAt` value `2026-07-24T17:17:09Z`. The issue was not
edited. Immediately before any future authorized issue edit, re-fetch the live
body and `updatedAt`; if either differs from this snapshot, regenerate and
re-review the proposed synchronization.

```markdown
## Objective

Implement the approved Publisher Services and Distribution Configuration design across Thoth, thoth-app, thoth-dissemination and cc-license with additive schema, explicit authorization, audited migration, comparison-mode cutover, bounded pilots, monitoring and rollback.

## Immutable authority at foundation review head

- [Private approved design](https://docs.google.com/document/d/1kr2Ft0Y4pxgcXGyFAKs_wfFx4I0jlxEvaceswE5Dus8/edit) - Drive revision `3`
- [Private design reference metadata](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/engineering/design-references.md#publisher-services-and-distribution-configuration)
- [Programme README](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/publisher-services/README.md)
- [Task tracker](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/publisher-services/task-status.md)
- [Platform inventory](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/publisher-services/platform-inventory.md)
- [Acceptance matrix](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/publisher-services/acceptance-matrix.md)
- [Rollout plan](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/publisher-services/rollout-plan.md)
- [Foundation specification](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/engineering/ai-delivery/tasks/CTRL-FOUNDATION-01.md)
- [Foundation implementation report](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/engineering/ai-delivery/implementation-reports/CTRL-FOUNDATION-01-implementation-report.md)
- [Foundation PR #764](https://github.com/thoth-pub/thoth/pull/764)

The Publisher Services design requires one fresh task branch and one PR per task. There is no long-lived `feature/publisher-services` integration branch.

## Current gate

- [ ] P0-01 independently approved and merged
- [ ] ADR-0001 approved
- [ ] ADR-0002 approved
- [ ] ADR-01 platform inventory approved
- [ ] repository branch-readiness decisions recorded

No production implementation begins before the applicable gate passes.

## Tasks

### Foundation

- [ ] P0-01 - Project control documents and tracker
- [ ] ADR-01 - Platform inventory and final architecture
- [ ] LIC-01 - Expand cc-license
- [ ] LIC-02 - Enforce supported licences in Thoth

### Backend

- [ ] BE-01 - Publisher package model
- [ ] BE-02 - Distribution platform model
- [ ] BE-03 - Protected service configuration
- [ ] BE-04 - Durable distribution jobs

### Migration and interfaces

- [ ] MIG-01 - Audit and production backfill
- [ ] APP-01 - Publisher service configuration UI
- [ ] APP-02 - Staff subscription report
- [ ] APP-03 - API-backed licence options

### Cutover and downstream services

- [ ] DIS-01 - API publisher discovery and comparison mode
- [ ] DIS-02 - Back-catalogue job worker
- [ ] EXP-01 - OCLC KBART feed index
- [ ] OAI-01 - Package and licence gating

### Stabilization

- [ ] OPS-01 - Monitoring, runbooks and cleanup
- [ ] E2E-01 - Full workflow verification

Do not close a task at PR creation or CI success. Close only after independent approval, merge, required rollout/observation and repository tracker update.
```

### Exact proposed post-merge issue #765 body

This replacement is proposed only for use after the assigned reviewer returns
`APPROVED`, the CTO merges the closeout PR, and the CTO separately authorizes
the issue edit.

> Superseded (2026-07-27): PR #767 has since been independently `APPROVED` and
> merged. The authoritative post-merge issue #765 synchronization proposal is
> now the regenerated body in
> [`P0-01-FINALIZE-implementation-report.md`](./P0-01-FINALIZE-implementation-report.md),
> which reflects the completed repository closeout. The body below is retained
> as historical evidence of what PR #767 proposed at review time.

```markdown
## Objective

Implement the approved Publisher Services and Distribution Configuration design across Thoth, thoth-app, thoth-dissemination and cc-license with additive schema, explicit authorization, audited migration, comparison-mode cutover, bounded pilots, monitoring and rollback.

## Immutable authority at foundation review head

- [Private approved design](https://docs.google.com/document/d/1kr2Ft0Y4pxgcXGyFAKs_wfFx4I0jlxEvaceswE5Dus8/edit) - Drive revision `3`
- [Private design reference metadata](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/engineering/design-references.md#publisher-services-and-distribution-configuration)
- [Programme README](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/publisher-services/README.md)
- [Task tracker](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/publisher-services/task-status.md)
- [Platform inventory](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/publisher-services/platform-inventory.md)
- [Acceptance matrix](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/publisher-services/acceptance-matrix.md)
- [Rollout plan](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/publisher-services/rollout-plan.md)
- [Foundation specification](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/engineering/ai-delivery/tasks/CTRL-FOUNDATION-01.md)
- [Foundation implementation report](https://github.com/thoth-pub/thoth/blob/b5b1622e54cb3c6fb372dcf02366c8dc4e38654e/docs/engineering/ai-delivery/implementation-reports/CTRL-FOUNDATION-01-implementation-report.md)
- [Foundation PR #764](https://github.com/thoth-pub/thoth/pull/764)
- [P0-01 closeout PR #767](https://github.com/thoth-pub/thoth/pull/767)

The Publisher Services design requires one fresh task branch and one PR per task. There is no long-lived `feature/publisher-services` integration branch.

## Synchronization guard

Before applying this replacement, re-fetch the complete live issue body and confirm that its `updatedAt` still matches the reviewed baseline `2026-07-24T17:17:09Z`. If either the live body or `updatedAt` differs, do not apply this replacement. Regenerate the minimal diff from the new live body and obtain fresh independent review before writing.

## Current gate

- [x] P0-01 independently approved and merged
- [ ] ADR-0001 approved
- [ ] ADR-0002 approved
- [ ] ADR-01 platform inventory approved
- [ ] repository branch-readiness decisions recorded

No production implementation begins before the applicable gate passes.

## Tasks

### Foundation

- [x] P0-01 - Project control documents and tracker - CLOSED after independent approval, PR #767 merge and issue synchronization
- [ ] ADR-01 - Platform inventory and final architecture
- [ ] LIC-01 - Expand cc-license
- [ ] LIC-02 - Enforce supported licences in Thoth

### Backend

- [ ] BE-01 - Publisher package model
- [ ] BE-02 - Distribution platform model
- [ ] BE-03 - Protected service configuration
- [ ] BE-04 - Durable distribution jobs

### Migration and interfaces

- [ ] MIG-01 - Audit and production backfill
- [ ] APP-01 - Publisher service configuration UI
- [ ] APP-02 - Staff subscription report
- [ ] APP-03 - API-backed licence options

### Cutover and downstream services

- [ ] DIS-01 - API publisher discovery and comparison mode
- [ ] DIS-02 - Back-catalogue job worker
- [ ] EXP-01 - OCLC KBART feed index
- [ ] OAI-01 - Package and licence gating

### Stabilization

- [ ] OPS-01 - Monitoring, runbooks and cleanup
- [ ] E2E-01 - Full workflow verification

P0-01 closure records control-foundation completion only. It does not approve an ADR or make another task ready.

Do not close a task at PR creation or CI success. Close only after independent approval, merge, required rollout/observation and repository tracker update.
```

Minimal semantic diff from the captured live issue:

- all ten existing authority links and their labels are preserved exactly;
- one authority link to PR #767 is added;
- all 23 checkbox rows are retained;
- only the P0-01 gate and P0-01 task change from unchecked to checked;
- the synchronization guard is added inside the exact proposed issue body;
- the guard requires the complete live issue body to be re-fetched and its
  `updatedAt` compared with the reviewed baseline before any write;
- any body or `updatedAt` mismatch requires the replacement to stop, be
  regenerated as a minimal diff and receive fresh independent review;
- no other gate, task or control rule is removed or changed.

## 11. CI

Baseline closeout head
`4f5c3491815e8d5ce4d1d6b15f316508494e503e` passed all seven required
checks: `build`, `build_and_push_staging_docker_image`, `check-changelog`,
`format_check`, `lint`, `run_migrations` and `test`.

The exact pre-amendment head
`00988232e40f0357d002ede998fbc31d149ed27f` passed all seven required checks.

Final reviewed content head `d72137893ddea512c0d05c81d310eb59d045cd2b`
passed all four required workflow runs and all seven required jobs, each
concluding `success`:

- run `30125538102` (build-test-and-check): `build`, `format_check`, `lint`,
  `test`;
- run `30125538127` (publish-to-dockerhub):
  `build_and_push_staging_docker_image`;
- run `30125538103` (check-changelog);
- run `30125538058` (run-migrations): `run_migrations`.

PR #767 merged into `develop` as
`bac598e32abbd0d7e69ff467c82945ee00df02ba` on `2026-07-27T09:29:57Z` after an
independent `APPROVED` review of that content head. GitHub's PR, review and
Actions records remain the external authorities; this report now records their
immutable identifiers. `d7213789...` is the independently reviewed content head;
`bac598e3...` is the merge commit. A later evidence-record commit does not
change the reviewed content that was merged through PR #767.

PR #764 final-head CI was green, but it is not evidence of independent
approval.

Review Cycle 2 returned `CHANGES REQUIRED` against
`d55ef26a0cc29d28d9c7d69ecbce60eb0082146e`; it did not approve PR #767 and
remains historical rather than final approval.

Procedural Review Cycle 3 returned `BLOCKED` against
`00988232e40f0357d002ede998fbc31d149ed27f`. Scope Amendment 3 resolves its sole
named-reviewer blocker but does not turn that review into approval. A new
independent review of the exact amendment head remains required.

## 12. Rollout and rollback

Initial state after merge: repository programme controls record the merged
foundation; P0-01 may be closed only after the reviewed, separately authorized
issue synchronization; production implementation remains blocked.

Activation required: none
Feature flag/configuration: none
Migration sequence: none
Rollback/disable procedure: revert the documentation PR. If issue #765 is later
synchronized and the closeout is reverted, do not restore an old snapshot
blindly. Follow the guarded issue rollback: re-fetch the complete live issue
body and its current `updatedAt`, compare against the expected state, stop on
any mismatch, generate a minimal reversal that preserves later unrelated edits,
and apply it only after fresh independent review and explicit CTO authorization.
Monitoring required: none

## 13. Known limitations and deferred work

- The historical Review Cycle 1 Claude review returned `CHANGES REQUIRED`.
- Review Cycle 2, performed in a fresh Codex context against
  `d55ef26a0cc29d28d9c7d69ecbce60eb0082146e`, returned
  `CHANGES REQUIRED` and did not approve the PR.
- Procedural Review Cycle 3, performed in a fresh Codex context against
  `00988232e40f0357d002ede998fbc31d149ed27f`, returned `BLOCKED` solely because
  the then-active task named Claude. Scope Amendment 3 removes that named-model
  requirement without granting retrospective approval.
- RESOLVED: the final content head
  `d72137893ddea512c0d05c81d310eb59d045cd2b` passed fresh final-head CI and
  received an independent `APPROVED` review from an eligible non-implementing
  context.
- RESOLVED: the closeout PR received CTO merge authorization and PR #767 merged
  as `bac598e32abbd0d7e69ff467c82945ee00df02ba` on 2026-07-27.
- Issue #765 remains unchanged and requires separate post-merge CTO
  authorization; see the guarded synchronization in
  `P0-01-FINALIZE-implementation-report.md`.
- ADR-0001 and ADR-0002 remain `PROPOSED`.
- Publisher Services ADR-01 and the final platform inventory remain unapproved.
- Applicable repository branch-readiness decisions remain outstanding.
- Every backend, migration, application, dissemination, export, OAI,
  operational and E2E task remains blocked.

## 14. Unresolved issues

- Fresh independent approval of the final content head
  `d72137893ddea512c0d05c81d310eb59d045cd2b`: RESOLVED - `APPROVED` before
  merge.
- Closeout PR merge: RESOLVED - PR #767 merged as
  `bac598e32abbd0d7e69ff467c82945ee00df02ba` on `2026-07-27T09:29:57Z`.
- Authorized post-merge issue synchronization: PENDING - a separately authorized
  external mirror of the completed repository closeout; see
  `P0-01-FINALIZE-implementation-report.md`.

## 15. Agent self-assessment

This report does not approve the task.

Suggested review focus:

- verify the complete merged PR #764 diff and its exact final-head CI;
- compare the foundation and closeout documents with Publisher Services design
  revision `3`;
- confirm that P0-01 is `MERGED` but not `CLOSED`;
- confirm that ADR, final-inventory and branch-readiness blockers remain;
- confirm that the diff has no runtime, migration, API, authorization,
  deployment or production effect;
- verify the exact proposed issue #765 body before any separately authorized
  post-merge synchronization.

## 16. Final review cycle, merge and post-merge findings (recorded by P0-01-FINALIZE)

This section was added by task P0-01-FINALIZE on 2026-07-27 to record concrete
final evidence. It does not rewrite the historical `CHANGES REQUIRED` and
`BLOCKED` review cycles above, which remain accurate historical evidence.

### Final independent review cycle

- Decision: `APPROVED`.
- Reviewed content head: `d72137893ddea512c0d05c81d310eb59d045cd2b`.
- Reviewer: fresh non-implementing context, high reasoning.
- Independence attestation: the final reviewer did not implement PR #764 or
  PR #767, did not author any PR #767 commit, and did not author Scope
  Amendments 1-3. It independently inspected the repository, the complete
  cumulative diff, the Scope Amendment 3 delta, required CI, issue #765 and
  Publisher Services design revision `3`.
- The review concluded that all six prior substantive P1 findings were resolved,
  the cumulative diff contained exactly the authorised documentation/control
  files, all required exact-head CI runs succeeded, issue #765 still matched the
  reviewed baseline `updatedAt: 2026-07-24T17:17:09Z`, the proposed issue body
  preserved all unrelated content and contained the synchronization guard, and
  no runtime, migration, API, authorization, workflow, deployment, release or
  production effect existed. No unresolved P0 or P1 remained at approval time.
- Neither prior Codex review (`CHANGES REQUIRED`, `BLOCKED`) became final
  approval retrospectively; this is a distinct, new review cycle.

### Merge

PR #767 merged into `develop` as
`bac598e32abbd0d7e69ff467c82945ee00df02ba` on `2026-07-27T09:29:57Z`. The merged
repository is the authoritative P0-01 closure record.

### Post-merge Codex findings and resolution

A Codex review posted after PR #767 merged identified three findings, each
addressed by task P0-01-FINALIZE:

1. **P1 - repository and issue closure would disagree.** Applying the proposed
   issue body immediately would mark P0-01 `CLOSED` while the merged repository
   still recorded `MERGED` with review, merge and synchronization pending.
   Resolution: P0-01-FINALIZE corrects the repository first (this report and the
   Publisher Services, engineering and Metrics control documents), then defers
   the issue #765 write to a separately authorized external-mirror step, so the
   two authoritative sources converge repository-first.
2. **P2 - final content head and CI not explicit in the tracked report.**
   Resolution: sections 1 and 11 now record reviewed content head
   `d72137893ddea512c0d05c81d310eb59d045cd2b`, merge commit
   `bac598e32abbd0d7e69ff467c82945ee00df02ba`, the independent `APPROVED`
   decision, the four workflow run IDs and all seven `success` jobs directly.
3. **P2 - rollback could overwrite later issue edits.** Resolution: sections 11
   and 12 and the associated task specification now require a guarded rollback -
   fresh body and `updatedAt` re-fetch, comparison, stop on mismatch, reviewed
   minimal reversal, and explicit CTO authorization - instead of an
   unconditional snapshot restore.
