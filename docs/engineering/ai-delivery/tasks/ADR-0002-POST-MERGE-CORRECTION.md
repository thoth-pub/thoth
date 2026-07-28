# ADR-0002-POST-MERGE-CORRECTION - Resolve post-merge review findings

Status: APPROVED
Programme: Cross-programme Engineering Control
Affected programmes:

- Publisher Services and Distribution Configuration
- Thoth Metrics

Repository: thoth-pub/thoth
Workflow: STANDARD
Base branch: develop
Verified base commit: `e124221f8444bd738228f1b609c536639be8789e`
PR target: develop
Programme integration branch: None
Risk: MEDIUM
Owner: CTO
Approved by: Javi, CTO
Approval date: 2026-07-28
Implementation reasoning: High
Independent review reasoning: High
Target branch: `feature/engineering/adr-0002-post-merge-correction`

## 1. Objective

Resolve the three P1 findings raised by the automated Codex review after PR #769
had already merged, without changing ADR-0002, changing runtime behaviour, or
writing GitHub issues #765 or #766.

## 2. Authoritative findings

The follow-up must address all three unresolved review threads on PR #769:

1. Embed the complete exact proposed post-merge issue bodies for #765 and #766 in
   the ADR-0002 approval implementation report. Abbreviated diffs and literal
   ellipses are not acceptable.
2. Reconcile `docs/engineering/agent-instructions/rollout-plan.md` with the active
   engineering README so it no longer says issue #765 synchronization remains the
   only foundation-closeout step.
3. Remove trailing whitespace from the ADR-0002 approval implementation report and
   verify `git diff --check` is genuinely clean.

## 3. Explicit scope

The task must:

- commit this specification first;
- update
  `docs/engineering/ai-delivery/implementation-reports/ADR-0002-APPROVE-implementation-report.md`;
- replace both abbreviated issue proposals with the complete exact proposed issue
  bodies based on the reviewed live baselines;
- preserve the issue baseline timestamps and hashes;
- state clearly that the proposed bodies are recorded evidence only and are not
  authorized writes;
- remove all trailing whitespace from the report;
- update `docs/engineering/agent-instructions/rollout-plan.md` so its current-state
  and rollout-sequence entries agree that the repository foundation closeout is
  complete and issue #765 was synchronized on 2026-07-27;
- create a separate implementation report for this correction;
- reply to the three PR #769 review threads after the corrective PR is independently
  approved and merged, linking the corrective merge commit, then resolve them.

## 3.1 Scope amendment: no changelog

This correction changes only internal engineering-control evidence and does not
change user-visible product behaviour. The repository's supported `no changelog`
PR label is therefore used instead of modifying `CHANGELOG.md`. This amendment was
recorded after the changelog check correctly failed on the initial branch head.
The label must cause the required check to rerun and conclude successfully; it is
not a bypass of any other CI or review gate.

## 4. Non-goals

This task must not:

- edit ADR-0002 or any other ADR;
- edit issues #765 or #766;
- execute either proposed issue synchronization;
- approve ADR-0001 or Publisher Services ADR-01;
- change programme readiness or make a task `READY`;
- change Rust, SQL, migrations, GraphQL, workflows, generated code, deployment or
  branch-protection configuration;
- deploy, release or activate production behaviour;
- rewrite historical review comments;
- merge without independent review.

## 5. Approved file allowlist

```text
docs/engineering/agent-instructions/rollout-plan.md
docs/engineering/ai-delivery/tasks/ADR-0002-POST-MERGE-CORRECTION.md
docs/engineering/ai-delivery/implementation-reports/ADR-0002-APPROVE-implementation-report.md
docs/engineering/ai-delivery/implementation-reports/ADR-0002-POST-MERGE-CORRECTION-implementation-report.md
```

Any additional file requires a written scope amendment before editing.

## 6. Invariants

```text
ADR-0001: PROPOSED
ADR-0002: APPROVED
Publisher Services ADR-01: unapproved
Platform inventory: FINAL ENUM NOT APPROVED
MET-CTRL-01: CHANGES REQUIRED
All Publisher Services implementation tasks: BLOCKED
All Metrics work packages: BLOCKED
Issues #765 and #766: unchanged by this task
No runtime effect
```

## 7. Acceptance criteria

- [ ] This task specification is the first commit on the branch.
- [ ] The cumulative changed-file set is a subset of the approved allowlist.
- [ ] The ADR-0002 approval report contains the complete exact proposed body for
      issue #765, preserving every unrelated line and changing only the guard
      baseline and ADR-0002 checkbox.
- [ ] The ADR-0002 approval report contains the complete exact proposed body for
      issue #766, adding the guard and changing only the ADR-0002 checkbox apart
      from the guard.
- [ ] Neither issue is written.
- [ ] The exact proposed-body hashes in the report match the embedded bodies.
- [ ] The agent rollout plan no longer contradicts the engineering README.
- [ ] `git diff --check` passes with no trailing-whitespace errors.
- [ ] The `no changelog` label is present and the changelog check succeeds.
- [ ] No ADR, runtime, migration, API, workflow, generated or deployment file changes.
- [ ] The corrective implementation report records base, branch, PR, commits,
      changed files, findings addressed, tests, CI, no-runtime assessment, issue
      baselines and residual blockers.
- [ ] All required final-head CI jobs succeed.
- [ ] A fresh non-implementing reviewer returns `APPROVED` before merge.
- [ ] After merge, the three PR #769 review threads receive a reply linking the
      corrective merge commit and are resolved.

## 8. Rollout and rollback

Rollout is merge-only documentation reconciliation. No activation follows.

Rollback is a normal revert of the corrective PR. Issues #765 and #766 remain
untouched, so no issue rollback is involved.

## 9. Required handoff

Return `READY FOR INDEPENDENT REVIEW` with the exact base and head, commit list,
changed-file list, exact proposed-body hashes, `git diff --check` result, final-head
CI, issue baselines, and confirmation that no issue or runtime write occurred.
