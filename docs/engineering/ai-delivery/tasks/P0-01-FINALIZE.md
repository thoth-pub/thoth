# P0-01-FINALIZE - Finalize Publisher Services P0-01 repository closeout

Status: APPROVED
Programme: Publisher Services and Distribution Configuration
Repository: thoth-pub/thoth
Workflow: STANDARD
Base branch: develop
Verified base commit: `bac598e32abbd0d7e69ff467c82945ee00df02ba`
PR target: develop
Programme integration branch: None
Risk: LOW
Owner: CTO
Approved by: Javi, CTO
Approval date: 2026-07-27
Independent reviewer: fresh non-implementing context, high reasoning
Target branch name: `feature/publisher-services/p0-01-finalize`

Dependencies:

* Closeout PR [#767](https://github.com/thoth-pub/thoth/pull/767) merged into
  `develop` as `bac598e32abbd0d7e69ff467c82945ee00df02ba` on
  `2026-07-27T09:29:57Z`;
* PR #767 received an independent `APPROVED` review in a fresh non-implementing
  context before merge;
* Foundation PR [#764](https://github.com/thoth-pub/thoth/pull/764) merged as
  `5b406e4ef9b5c192cc38eb8a97a41bbd0fc3bc06`;
* GitHub issue [#765](https://github.com/thoth-pub/thoth/issues/765) exists and
  remains open at reviewed baseline `updatedAt: 2026-07-24T17:17:09Z`;
* Publisher Services private design, Drive revision `3`,
  `Approved for phased implementation`;
* an independent reviewer who did not implement PR #764, PR #767 or this task.

## 1. Objective

Reconcile every active repository control with the actual independent approval
and merge of PR #767 so that the merged repository is the authoritative P0-01
closure record, record the exact final-head, review, CI and merge evidence
directly in the tracked report, address the three post-merge Codex findings, and
generate a new exact proposed issue #765 synchronization body for later separate
review and authorization - all without changing architecture, application
behaviour, database state, dissemination, migrations, workflows, deployment or
production configuration, and without writing to issue #765.

## 2. Background and authority

Authoritative sources, in precedence order:

1. `thoth-pub/thoth` branch `develop`;
2. merged PR #767 and its exact-head CI;
3. merged PR #764 and its final-head CI;
4. the approved Publisher Services private design, Drive revision `3`;
5. `docs/engineering/decisions/`;
6. `docs/publisher-services/`;
7. GitHub issue #765;
8. repository agent instructions and delivery controls.

Current verified state at specification time:

* PR #767 is merged as commit `bac598e32abbd0d7e69ff467c82945ee00df02ba`.
* `origin/develop` points exactly to that merge commit.
* PR #767's independently reviewed content head was
  `d72137893ddea512c0d05c81d310eb59d045cd2b`.
* PR #767 received an independent `APPROVED` decision before merge.
* Several active control documents still describe P0-01 as `MERGED` with PR #767
  remediation, fresh independent approval, closeout merge and issue
  synchronization pending.
* ADR-0001 and ADR-0002 remain `PROPOSED`.
* The platform inventory remains `VERIFIED BASELINE; FINAL ENUM NOT APPROVED`.
* Issue #765 remains open and unchanged at `updatedAt: 2026-07-24T17:17:09Z`.

### Repository-first closeout sequence

Git and GitHub issue updates cannot be atomic. The CTO approves this
repository-first sequence:

1. Prepare and independently review this P0-01 finalization PR.
2. Merge the finalization PR only after independent `APPROVED` and explicit CTO
   authorization.
3. The merged repository becomes the authoritative P0-01 closure record.
4. Re-fetch the complete live issue #765 body and `updatedAt`.
5. Apply a separately reviewed and separately authorized issue synchronization.
6. Keep all ADR, inventory, branch-readiness and implementation gates blocked.

The short interval in which the repository says P0-01 is closed while issue #765
remains conservatively unchecked is acceptable. The opposite sequence is not
permitted: issue #765 must not declare P0-01 closed while the repository still
records it as incomplete. Issue #765 synchronization is an external mirror of
the repository closeout. It must not make another task `READY`.

## 3. Post-merge findings addressed

A Codex review posted after PR #767 merged identified three findings, addressed
by this task:

1. **P1 - Repository and issue closure would disagree.** The proposed issue body
   marks P0-01 `CLOSED` while the merged repository still records P0-01 as
   `MERGED` with review, merge and synchronization pending. This task corrects
   the repository first, so the two authoritative sources converge.
2. **P2 - Final content head and CI are not explicit in the tracked report.**
   This task records reviewed content head `d7213789...`, merge commit
   `bac598e3...`, the independent approval, the exact workflow run IDs and all
   seven job conclusions directly in the tracked report.
3. **P2 - Rollback could overwrite later issue edits.** This task replaces every
   unsafe "restore the exact captured issue body" instruction with a guarded
   rollback that re-fetches the live body and `updatedAt`, compares, stops on any
   mismatch, and applies only a reviewed minimal reversal under explicit CTO
   authorization.

## 4. Explicit scope

The task must:

1. Commit this approved task specification first.
2. Reconcile all active repository controls with the actual approval and merge of
   PR #767 and make the repository the authoritative P0-01 closure record, by
   updating only the active stale statements in:
   * `docs/publisher-services/task-status.md` (P0-01 -> `CLOSED`);
   * `docs/publisher-services/README.md`;
   * `docs/publisher-services/rollout-plan.md`;
   * `docs/engineering/README.md`;
   * `docs/engineering/agent-instructions/rollout-plan.md`;
   * `docs/engineering/repository-map/control-gaps.md`;
   * `docs/metrics/README.md` (shared-foundation provenance only);
   * `docs/metrics/task-status.md` (shared-foundation provenance only).
3. Record the concrete final-head, review, CI and merge evidence directly in
   `docs/engineering/ai-delivery/implementation-reports/P0-01-CLOSEOUT-implementation-report.md`.
4. Replace unsafe rollback wording with guarded rollback wording in every
   authorized active file that contains it
   (`docs/engineering/ai-delivery/tasks/P0-01-CLOSEOUT.md` and the closeout
   implementation report).
5. Create
   `docs/engineering/ai-delivery/implementation-reports/P0-01-FINALIZE-implementation-report.md`.
6. Generate a new exact proposed issue #765 synchronization body inside the
   finalization report, for later separate review and authorization.
7. Add a bounded `Changed` CHANGELOG entry referencing this finalization PR.

## 5. Non-goals

* no issue #765 write;
* no issue closure;
* no ADR approval;
* no inventory approval;
* no branch-readiness decision;
* no implementation task readiness;
* no Rust, SQL, migration, GraphQL, workflow or configuration change;
* no deployment or release;
* no production operation;
* no merge authorization.

## 6. Invariants

* ADR-0001 remains `PROPOSED`;
* ADR-0002 remains `PROPOSED`;
* Publisher Services ADR-01 remains unapproved;
* platform inventory remains `FINAL ENUM NOT APPROVED`;
* repository branch-readiness remains outstanding;
* `MET-CTRL-01` remains `CHANGES REQUIRED`;
* all Publisher Services implementation tasks remain blocked;
* all Metrics implementation tasks remain blocked;
* Publisher Services and Metrics remain separate programmes;
* no external issue mutation occurs in this task;
* the implementing agent does not approve or merge its own work.

## 7. Approved file allowlist

```text
CHANGELOG.md
docs/engineering/README.md
docs/engineering/agent-instructions/rollout-plan.md
docs/engineering/ai-delivery/tasks/P0-01-CLOSEOUT.md
docs/engineering/ai-delivery/tasks/P0-01-FINALIZE.md
docs/engineering/ai-delivery/implementation-reports/P0-01-CLOSEOUT-implementation-report.md
docs/engineering/ai-delivery/implementation-reports/P0-01-FINALIZE-implementation-report.md
docs/engineering/repository-map/control-gaps.md
docs/metrics/README.md
docs/metrics/task-status.md
docs/publisher-services/README.md
docs/publisher-services/rollout-plan.md
docs/publisher-services/task-status.md
```

The actual changed-file set may be a subset of this allowlist. A file is edited
only when it contains an active stale statement or is required evidence. If
another active file must change to make repository state consistent, stop and
report `SCOPE AMENDMENT REQUIRED` with the exact file, stale statement and
required correction.

## 8. Acceptance criteria

* [ ] The task specification was committed before any other change.
* [ ] The changed-file set is a subset of the approved allowlist.
* [ ] `docs/publisher-services/task-status.md` records P0-01 as `CLOSED`, with
  no completed review/merge action listed as a blocker.
* [ ] No active control document says PR #767 review or merge remains pending.
* [ ] The closeout report records reviewed content head `d7213789...`, merge
  commit `bac598e3...`, merged-at time, independent `APPROVED`, and all four
  workflow run IDs with all seven successful jobs.
* [ ] All unsafe rollback wording is replaced with guarded rollback wording.
* [ ] The finalization report embeds the exact proposed issue #765 body with the
  forward synchronization guard and rollback guard.
* [ ] ADR-0001 and ADR-0002 remain `PROPOSED`.
* [ ] The platform inventory remains `FINAL ENUM NOT APPROVED`.
* [ ] `MET-CTRL-01` remains `CHANGES REQUIRED` and no Metrics work package
  advances.
* [ ] No Publisher Services or Metrics implementation task becomes `READY`.
* [ ] `git diff --check` passes and no runtime file changed.
* [ ] Required final-head CI is green.
* [ ] Issue #765 remains unchanged at `updatedAt: 2026-07-24T17:17:09Z`.
* [ ] An independent reviewer returns `APPROVED` before merge.

## 9. Verification

```bash
git diff --check
git diff --name-only origin/develop...HEAD

git diff --name-only origin/develop...HEAD \
  | grep -Ev '^(CHANGELOG\.md|docs/)' \
  && exit 1 || true

grep -n '^Status: PROPOSED$' \
  docs/engineering/decisions/ADR-0001-publisher-package-capability-model.md \
  docs/engineering/decisions/ADR-0002-platform-domain-boundaries.md

grep -n 'FINAL ENUM NOT APPROVED' \
  docs/publisher-services/platform-inventory.md

grep -n 'MET-CTRL-01.*CHANGES REQUIRED' \
  docs/metrics/task-status.md

grep -n 'P0-01.*CLOSED' \
  docs/publisher-services/task-status.md
```

Required CI jobs at the final head:

```text
build
format_check
lint
test
build_and_push_staging_docker_image
check-changelog
run_migrations
```

## 10. Rollout

Initial state after merge:

* the repository is the authoritative P0-01 closure record;
* issue #765 synchronization becomes a separately authorized external mirror;
* the programme remains blocked for production implementation.

Feature flag or configuration: none. Staging or preview: none. Pilot: none.

Activation approval:

* CTO approval is required to merge this finalization PR;
* separate CTO authorization is required for the post-merge issue #765
  synchronization.

## 11. Rollback

Code rollback: revert this documentation PR; no runtime effect.

Issue rollback (guarded; no unconditional full-body overwrite):

1. Re-fetch the complete live issue #765 body.
2. Re-fetch its current `updatedAt`.
3. Compare against the exact state expected by the rollback plan.
4. If either differs, stop.
5. Generate a minimal reversal preserving all later unrelated edits.
6. Obtain fresh independent review.
7. Obtain explicit CTO authorization.
8. Apply only the reviewed minimal reversal.
9. Never overwrite the complete issue body with an old snapshot blindly.

## 12. Independent-review requirement

The implementing context is ineligible to approve this work. The final reviewer
must be independent of this implementation, use a fresh context and high
reasoning, inspect the exact final head, all changed files, the complete PR #767
history and merge, the post-merge findings, the issue body and timestamp and
every required CI job, verify no P0/P1 remains, and return exactly `APPROVED`,
`CHANGES REQUIRED` or `BLOCKED`. A different model family is preferred but not
mandatory. The reviewer must not modify, approve and merge the same work.

## 13. Approval

Approved for implementation by: Javi, CTO. Date: 2026-07-27.

Notes:

* Documentation and programme-control changes only.
* This approval does not approve any ADR, the final inventory, branch readiness,
  any implementation task, migration, workflow, deployment, release or
  production activation.
* This approval does not authorize any write to issue #765; that is a separately
  reviewed and separately authorized step after this PR merges.
* The implementing agent may create the task branch, commit, push and open a
  draft PR, but may not approve or merge it.
