# P0-01-CLOSEOUT - Reconcile Publisher Services foundation status

Status: APPROVED
Programme: Publisher Services and Distribution Configuration
Repository: thoth-pub/thoth
Workflow: STANDARD
Base branch: develop
Approved base commit: `5b406e4ef9b5c192cc38eb8a97a41bbd0fc3bc06`
PR target: develop
Programme integration branch: None
Risk: LOW
Owner: CTO
Approved by: Javi, CTO
Approval date: 2026-07-24
Independent reviewer: independent reviewer, separate non-implementing context, high reasoning
Dependencies:

* PR #764 merged into `develop`;
* GitHub issue #765 exists and remains open;
* Publisher Services private design, Drive revision `3`;
* access to PR #764, its final head, CI evidence and merged diff;
* an independent reviewer who did not implement PR #764 or this closeout task.

Target branch name: `feature/publisher-services/p0-01-closeout`

## 1. Objective

Reconcile the Publisher Services programme-control record with the actual state after PR #764 merged, remove stale pre-merge statements, and capture sufficient independent review evidence to close P0-01 without changing architecture, application behaviour, database state, dissemination, migrations, deployment or production configuration.

## 2. Background and authority

Authoritative sources, in precedence order:

1. `thoth-pub/thoth` branch `develop`;
2. merged PR #764 and its final-head CI;
3. the approved Publisher Services private design, Drive revision `3`;
4. `docs/engineering/decisions/`;
5. `docs/publisher-services/`;
6. GitHub issue #765;
7. repository agent instructions and delivery controls.

Current verified state:

* PR #764 is merged as commit `5b406e4ef9b5c192cc38eb8a97a41bbd0fc3bc06`.
* `develop` points exactly to that merge commit at specification time.
* The P0-01 artefacts are present on `develop`.
* The README, task tracker and issue still describe P0-01 as unmerged or under review.
* ADR-0001 and ADR-0002 remain `PROPOSED`.
* ADR-01 and the final platform inventory remain unapproved.
* No submitted independent GitHub review is recorded for PR #764.

## 3. Explicit scope

The task must:

1. Commit this approved task specification at:
   `docs/engineering/ai-delivery/tasks/P0-01-CLOSEOUT.md`.
2. Update `docs/publisher-services/README.md` so that:

   * the control foundation is recorded as merged;
   * P0-01 is no longer listed as unmerged;
   * implementation remains blocked by the actual remaining ADR, inventory and branch-readiness gates;
   * no wording suggests that production implementation is authorized.
3. Update `docs/publisher-services/task-status.md` so that:

   * P0-01 is recorded as `MERGED`;
   * PR #764 and merge commit `5b406e4ef9b5c192cc38eb8a97a41bbd0fc3bc06` are recorded;
   * acceptance distinguishes merged deliverables from independent closeout evidence;
   * the stale instruction to approve and merge PR #764 is removed;
   * the next actions reflect the remaining ADR approvals and ADR-01 specification.
4. Update the Stage 0 section of `docs/publisher-services/rollout-plan.md` to distinguish:

   * achieved evidence: PR #764 merged and the master issue exists;
   * outstanding evidence: independent closeout review and remaining ADR/branch-readiness decisions.
5. Review, but do not alter without a concrete inconsistency:

   * `docs/publisher-services/decisions.md`;
   * `docs/publisher-services/platform-inventory.md`;
   * `docs/publisher-services/acceptance-matrix.md`;
   * `docs/engineering/decisions/ADR-0001-publisher-package-capability-model.md`;
   * `docs/engineering/decisions/ADR-0002-platform-domain-boundaries.md`.
6. Update `CHANGELOG.md` under `Unreleased` with a bounded documentation/control entry referencing the closeout PR once its number is known.
7. Produce an implementation report using:
   `docs/engineering/ai-delivery/implementation-report-template.md`.
8. In the implementation report, provide the exact proposed post-merge edit to issue #765:

   * mark the P0-01 deliverables as merged;
   * link PR #764 and the closeout PR;
   * mark P0-01 closed only after the independent review and closeout PR merge;
   * leave ADR-0001, ADR-0002, ADR-01 and branch-readiness gates unchecked;
   * leave every production task unchecked.
9. Open a draft PR targeting `develop`.
10. Obtain an independent review that explicitly reviews:

    * the complete merged PR #764 diff;
    * PR #764 final-head CI;
    * fidelity to Publisher Services design revision `3`;
    * the closeout diff;
    * the absence of runtime effects;
    * the accuracy of the remaining blockers.
11. After independent `APPROVED` and CTO merge, perform the separately authorized issue #765 synchronization exactly as reviewed.

## 4. Non-goals

The task must not:

1. approve or modify ADR-0001, ADR-0002 or Publisher Services ADR-01;
2. finalize or implement `DistributionPlatform`;
3. implement package, capability, platform-assignment, audit or job models;
4. add or modify PostgreSQL migrations;
5. modify Rust, GraphQL, export-server or generated-client behaviour;
6. change `thoth-app`, `thoth-dissemination` or `cc-license`;
7. modify workflows, branch protections, repository settings or deployment configuration;
8. create a long-lived `feature/publisher-services` branch;
9. modify `master`;
10. run, dispatch or activate production behaviour;
11. claim that PR #764 received independent approval without review evidence;
12. close any implementation task other than P0-01.

## 5. Invariants

The implementation must preserve:

1. Repository code and runtime behaviour remain byte-for-byte unchanged outside approved documentation and changelog files.
2. Publisher package configuration remains independent from distribution-platform assignments.
3. `DistributionPlatform` and `MetricPlatform` remain separate domains.
4. ADR-0001 and ADR-0002 remain `PROPOSED`.
5. The platform inventory remains a baseline and must not be represented as the final enum.
6. No production implementation becomes `READY` solely because P0-01 is closed.
7. Publisher Services continues to use one task branch and one PR targeting the verified development branch.
8. Missing independent-review evidence is reported, not inferred.
9. The implementing agent does not approve or merge its own work.
10. Issue #765 and repository status are synchronized only with factual, reviewed evidence.

## 6. Required behaviour

### 6.1 Success behaviour

After the repository PR merges and issue synchronization is completed:

* P0-01 is accurately recorded as merged, independently reviewed and closed.
* The programme README no longer contains a false unmerged-foundation blocker.
* Remaining architecture and implementation blockers are explicit.
* Issue #765 and the repository tracker agree.
* ADR-01 is identified as the next Publisher Services architecture task, but remains blocked until its applicable dependencies and approved written specification are satisfied.
* No runtime or production behaviour changes.

### 6.2 Failure behaviour

If the independent reviewer cannot verify PR #764 or finds material P0/P1 problems:

* return `CHANGES REQUIRED` or `BLOCKED`;
* do not mark P0-01 `CLOSED`;
* do not check the combined independent-approval gate in issue #765;
* record the missing evidence or required remediation precisely;
* do not proceed to ADR-01 implementation.

### 6.3 Authorization

No application authorization path changes.

GitHub write actions are limited to:

* pushing the task branch;
* opening or updating the draft PR;
* post-merge issue #765 synchronization only after explicit authorization.

The implementing agent must not merge the PR.

### 6.4 Concurrency and idempotency

No runtime concurrency applies.

Documentation and issue updates must be repeatable without duplicating changelog entries, issue sections or task rows.

### 6.5 Compatibility

No database, API, GraphQL, client, workflow, deployment or production compatibility effect is permitted.

## 7. Data and migration requirements

Migration required: NO

* Schema changes: none.
* Existing-data changes: none.
* Locking or downtime: none.
* Backfill: none.
* Generated schema: unchanged.
* Rollback: documentation revert only; any issue-body reversal follows the guarded rollback in section 12.

## 8. Observability and operations

Required logs: none.

Required metrics or alerts: none.

Operational runbook changes: none.

The implementation report must record:

* the exact base and head commits;
* changed files;
* PR #764 merge evidence;
* final-head CI evidence;
* issue #765 pre-change state;
* the proposed post-merge issue update;
* the independent-review decision;
* confirmation of no runtime effect.

## 9. Acceptance criteria

* [ ] Branch creation started from `develop` at `5b406e4ef9b5c192cc38eb8a97a41bbd0fc3bc06`.
* [ ] Only the approved task specification, programme-control documents, changelog and implementation report changed.
* [ ] The README no longer says P0-01 or PR #764 is unmerged.
* [ ] The tracker records P0-01 as `MERGED` and records the exact merge commit.
* [ ] The stale “approve and merge PR #764” next action is removed.
* [ ] The rollout plan identifies PR #764 merge evidence as achieved.
* [ ] ADR-0001 and ADR-0002 remain `PROPOSED`.
* [ ] The platform inventory remains explicitly unapproved.
* [ ] No backend, migration, application, dissemination, workflow or production file changed.
* [ ] `git diff --check` passes.
* [ ] All relative links and GitHub references resolve.
* [ ] Required final-head CI is green.
* [ ] A separate reviewer verifies both merged PR #764 and the closeout diff.
* [ ] The reviewer returns `APPROVED`.
* [ ] The CTO merges the closeout PR.
* [ ] Issue #765 is then synchronized exactly as reviewed.
* [ ] No other programme task is marked complete or implementation-ready.

## 10. Required checks

### Repository and branch evidence

```bash
git fetch origin --prune

test "$(git rev-parse origin/develop)" = \
  "5b406e4ef9b5c192cc38eb8a97a41bbd0fc3bc06"
```

If this assertion fails, apply the stop conditions below.

Verify PR #764:

```bash
gh pr view 764 \
  --repo thoth-pub/thoth \
  --json state,mergedAt,mergeCommit,baseRefName,headRefName,url

gh pr checks 764 --repo thoth-pub/thoth
```

Verify issue #765 before proposing any edit:

```bash
gh issue view 765 \
  --repo thoth-pub/thoth \
  --json state,title,body,updatedAt,url
```

### Documentation checks

```bash
git diff --check \
  5b406e4ef9b5c192cc38eb8a97a41bbd0fc3bc06...HEAD

git diff --name-only \
  5b406e4ef9b5c192cc38eb8a97a41bbd0fc3bc06...HEAD
```

The changed-file list must contain no Rust, SQL, migration, workflow, package, generated-contract, deployment or runtime files.

Check stale programme wording:

```bash
grep -RniE \
  'P0-01 is still part of the unmerged|approve and merge PR #764|P0-01.*IN REVIEW|PENDING REVIEW' \
  docs/publisher-services \
  || true
```

Expected result after remediation: no stale status occurrence, except where deliberately quoted as historical evidence in the implementation report.

Verify ADR status:

```bash
grep -n '^Status: PROPOSED$' \
  docs/engineering/decisions/ADR-0001-publisher-package-capability-model.md \
  docs/engineering/decisions/ADR-0002-platform-domain-boundaries.md
```

Verify the inventory remains unapproved:

```bash
grep -n 'FINAL ENUM NOT APPROVED' \
  docs/publisher-services/platform-inventory.md
```

### Tests

Unit tests: not applicable.

Integration/database tests: not applicable.

Authorization/security tests: not applicable.

Regression evidence:

* required GitHub CI at the closeout PR head;
* manual inspection of the complete diff;
* link and status consistency checks.

Performance: not applicable.

## 11. Rollout

Initial state after merge:

* repository programme status reflects the merged foundation;
* P0-01 may be closed after issue synchronization;
* the programme remains blocked for production implementation.

Feature flag or configuration: none.

Staging or preview: none.

Pilot: none.

Activation approval:

* CTO approval is required to merge;
* separate CTO authorization is required for the post-merge issue edit.

Observation period: none beyond confirming the repository and issue display the same state.

## 12. Rollback

Code rollback:

* revert the closeout documentation PR.

Data rollback:

* not applicable.

Issue rollback (guarded; no unconditional full-body overwrite):

* re-fetch the complete live issue #765 body and its current `updatedAt`;
* compare against the exact state expected by the rollback plan;
* if either the body or `updatedAt` differs, stop;
* generate a minimal reversal that preserves all later unrelated edits;
* obtain fresh independent review and explicit CTO authorization;
* apply only the reviewed minimal reversal;
* never overwrite the complete issue body with an old snapshot blindly.

External side effects: none beyond GitHub documentation metadata.

## 13. Stop conditions

The implementing agent must stop and return `BLOCKED` if:

1. `origin/develop` no longer equals the approved base commit;
2. any Publisher Services control document has changed since this specification and creates a material conflict;
3. PR #764 is not merged into `develop`;
4. the private design cannot be verified as Drive revision `3`;
5. completing the task would require approving an ADR;
6. completing the task would require code, migration, workflow or production changes;
7. no independent reviewer can inspect both PR #764 and the closeout diff;
8. the reviewer cannot substantiate an `APPROVED` decision;
9. issue #765 has materially changed and the proposed synchronization would overwrite newer information;
10. unrelated changes are required.

## 14. Expected implementation report

Use:

`docs/engineering/ai-delivery/implementation-report-template.md`

The report must not issue the approval decision.

## 15. Recommended execution

Implementation model: Codex
Implementation reasoning: High
Independent reviewer: separate non-implementing context
Review reasoning: High

The independent reviewer must inspect actual diffs and evidence, not only the implementation report.

## 16. Branch and integration plan

* Branch source: `develop` at `5b406e4ef9b5c192cc38eb8a97a41bbd0fc3bc06`
* Task branch: `feature/publisher-services/p0-01-closeout`
* Pull-request target: `develop`
* Programme integration branch: None
* Expected merge order: this PR before ADR-01 implementation work
* Branch deletion after merge: YES
* Final programme PR required: NO
* Release path: `develop -> master`; no release is required for this documentation-only closeout

## 17. Approval

Approved for implementation by: Javi, CTO

Date: 2026-07-24

Notes:

* Approved without scope changes.
* Documentation and programme-control changes only.
* This approval does not approve any ADR, implementation, migration, workflow, deployment, release or production activation.
* This approval does not constitute retrospective independent approval of PR #764.
* An independent reviewer in a separate non-implementing context must inspect
  PR #764 and the closeout PR and return `APPROVED` before P0-01 may be marked
  `CLOSED`.
* The implementing agent may create the task branch, commit, push and open a draft PR, but may not approve or merge it.

## Approved Scope Amendment 1 - Shared foundation status consistency

Approved by: Javi, CTO

Approval date: 2026-07-24

Independent-review decision being addressed: `CHANGES REQUIRED`

Risk: LOW, unchanged

The first independent review of the complete PR #764 foundation and PR #767
closeout found four P1 issues:

1. repository-wide engineering and Metrics control documents still described
   PR #764 as unmerged or awaiting merge;
2. the foundation implementation report incorrectly implied that an independent
   review occurred before PR #764 merged;
3. the proposed issue #765 replacement removed existing authority links instead
   of changing only the P0-01 status;
4. the closeout implementation report did not contain a sufficiently exact
   commit and CI evidence record.

This amendment authorizes factual reconciliation in these additional files:

* `docs/engineering/README.md`;
* `docs/engineering/repository-map/control-gaps.md`;
* `docs/engineering/ai-delivery/implementation-reports/CTRL-FOUNDATION-01-implementation-report.md`;
* `docs/metrics/README.md`;
* `docs/metrics/task-status.md`.

The complete amended allowlist is:

* `CHANGELOG.md`;
* `docs/engineering/README.md`;
* `docs/engineering/repository-map/control-gaps.md`;
* `docs/engineering/ai-delivery/implementation-reports/CTRL-FOUNDATION-01-implementation-report.md`;
* `docs/engineering/ai-delivery/implementation-reports/P0-01-CLOSEOUT-implementation-report.md`;
* `docs/engineering/ai-delivery/tasks/P0-01-CLOSEOUT.md`;
* `docs/metrics/README.md`;
* `docs/metrics/task-status.md`;
* `docs/publisher-services/README.md`;
* `docs/publisher-services/rollout-plan.md`;
* `docs/publisher-services/task-status.md`.

This is a narrow cross-programme control-consistency exception because every
newly authorized statement concerns the factual merge and review state of the
shared foundation PR #764. It does not combine Publisher Services and Metrics
implementation scopes, approve Metrics architecture or work packages, close
`MET-CTRL-01`, make any Publisher Services or Metrics implementation task
`READY`, or authorize future cross-programme changes.

No runtime, migration, API, authorization, deployment, release or production
scope is added. Publisher Services and Metrics implementation scopes remain
distinct. P0-01 remains `MERGED`, not `CLOSED`, until remediation receives fresh
independent approval, PR #767 is merged by the CTO, and issue #765 is
synchronized under separate authorization.

## Approved Scope Amendment 2 - Remaining foundation status and synchronization guard

Approved by: Javi, CTO

Approval date: 2026-07-24

Review Cycle 2 decision: `CHANGES REQUIRED`

Reviewed head: `d55ef26a0cc29d28d9c7d69ecbce60eb0082146e`

Reviewer context: fresh Codex context, not the assigned Claude reviewer

Risk: LOW, unchanged

Review Cycle 2 found two remaining P1 issues:

1. the active agent-instruction rollout plan still described the already-merged
   PR #764 engineering-control foundation as awaiting independent review and
   merge, and the Metrics task tracker used obsolete PR-to-target coordinates
   for ADR-0001 and ADR-0002;
2. the exact proposed issue #765 body did not itself contain the synchronization
   guard requiring the complete live body and `updatedAt` to be re-fetched and
   compared before any write.

This amendment authorizes:

* editing
  `docs/engineering/agent-instructions/rollout-plan.md` only to correct obsolete
  active statements about the PR #764 foundation merge and remaining PR #767
  closeout;
* correcting the ADR-0001 and ADR-0002 provenance coordinates in
  `docs/metrics/task-status.md` while keeping both ADRs `PROPOSED`, their
  dependency as a CTO decision, `MET-CTRL-01` as `CHANGES REQUIRED`, and every
  Metrics work package `BLOCKED`;
* placing the concurrency and synchronization guard inside the exact proposed
  issue #765 body in the closeout implementation report; and
* updating the closeout implementation report with Review Cycle 2, both P1
  findings, this scope amendment, their exact remediation and the retained final
  independent-review requirement.

The complete cumulative allowlist for PR #767 is:

* `CHANGELOG.md`;
* `docs/engineering/README.md`;
* `docs/engineering/agent-instructions/rollout-plan.md`;
* `docs/engineering/ai-delivery/implementation-reports/CTRL-FOUNDATION-01-implementation-report.md`;
* `docs/engineering/ai-delivery/implementation-reports/P0-01-CLOSEOUT-implementation-report.md`;
* `docs/engineering/ai-delivery/tasks/P0-01-CLOSEOUT.md`;
* `docs/engineering/repository-map/control-gaps.md`;
* `docs/metrics/README.md`;
* `docs/metrics/task-status.md`;
* `docs/publisher-services/README.md`;
* `docs/publisher-services/rollout-plan.md`;
* `docs/publisher-services/task-status.md`.

No runtime, Rust, SQL, migration, GraphQL, generated-contract, workflow,
repository-configuration, deployment, release, production-activation, Metrics
architecture or implementation scope is added. This amendment does not approve
ADR-0001 or ADR-0002, change the Metrics private design, advance
`MET-CTRL-01`, advance any Metrics or Publisher Services implementation task,
authorize issue #765 modification or close P0-01.

Review Cycle 2 did not approve PR #767 and does not replace the required final
independent review in a separate non-implementing context with high reasoning.

## Approved Scope Amendment 3 - Reviewer Independence Criteria

Approved by: Javi, CTO

Approval date: 2026-07-24

Procedural review decision: `BLOCKED`

Reviewed head: `00988232e40f0357d002ede998fbc31d149ed27f`

Reviewing model: Codex

Risk: LOW, unchanged

The procedural review at `00988232e40f0357d002ede998fbc31d149ed27f`
confirmed that every substantive review criterion passed:

* all six P1 findings from Review Cycles 1 and 2 were resolved;
* the repository, cumulative diff and final remediation delta matched the
  approved scope;
* the Publisher Services private design was verified at Drive revision `3`;
* issue #765 preservation and synchronization controls passed;
* required final-head CI was green;
* no runtime, migration, API, authorization, workflow, deployment, release or
  production effect existed.

The sole blocker was procedural: the specification named Claude while the
available independent reviewer was Codex. Reviewer independence is the control
objective for this LOW-risk documentation/control task; a particular human,
product or model-family name is no longer mandatory.

The final reviewer may be a human or AI model and must:

* not have implemented PR #764 or PR #767;
* not have authored the remediation commits being reviewed;
* work from a fresh context without relying on the implementing context's
  private reasoning;
* use high reasoning;
* have direct access to the repository, complete diffs, CI, issue #765 and
  Publisher Services design revision `3`;
* review the actual evidence rather than accepting the implementation summary;
* return exactly `APPROVED`, `CHANGES REQUIRED` or `BLOCKED`;
* not modify, approve and merge the same work in one role.

A different model family is preferred where practical but is not required. A
fresh Codex context is eligible only if it did not implement or remediate PR
#767 and conducts the complete review independently.

This amendment replaces the previous named-Claude requirement everywhere it was
active. Historical references to the earlier Claude assignment and decisions
remain historical evidence only. Neither previous Codex review becomes final
approval retrospectively. A new exact-head independent review remains required
after this amendment is committed.

The context implementing this amendment is ineligible to review or approve its
own amendment. This amendment changes no risk classification, runtime scope,
file scope beyond the two amendment records, rollout, issue-write
authorization, merge authority, deployment authority, release authority or
production authorization.
