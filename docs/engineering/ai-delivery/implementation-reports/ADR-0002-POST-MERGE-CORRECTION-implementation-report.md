# ADR-0002-POST-MERGE-CORRECTION Implementation Report

## 1. Repository state

Repository: `thoth-pub/thoth`
Task: `ADR-0002-POST-MERGE-CORRECTION`
Base branch: `develop`
Base commit: `e124221f8444bd738228f1b609c536639be8789e`
Task branch: `feature/engineering/adr-0002-post-merge-correction`
Pull request: [#770](https://github.com/thoth-pub/thoth/pull/770) (draft)
Risk: MEDIUM
Reviewed evidence head: `8158603b30e87074326b5729bcf661678a4dccd5`
Prior independently approved head: `ca8e90645957c50c25fecd8b220772837bb522d3`
Implementing agent/model: Codex / GPT-5
Designated independent reviewer/model: ChatGPT / GPT-5.6 Thinking
Independent review context: fresh and non-implementing
Implementation reasoning: High
Independent review reasoning: High

## 2. Objective

Resolve the three P1 review findings posted on PR #769 after that PR had already
merged, without altering ADR-0002, writing issues #765/#766, or changing runtime
behaviour.

## 3. Commits

- `e4392d5b1eae661fa5ab7d11a8670bf748d462f2` - approved corrective task
  specification, committed first;
- `cc381a13d41ec97a3a902d61315878563e99cd03` - replace the flawed ADR-0002
  approval evidence report with a corrected consolidated report containing both
  complete proposed issue bodies and no trailing whitespace;
- `3c48daf183a1065d1aae8af258df5fd4aaf9bf24` - reconcile the active agent
  rollout plan with the engineering README;
- `dd292df8276c0b8d024dd772a96672925b9b8268` - create this corrective
  implementation report;
- `8b1647e9f77ec478c0209a82086f638d20ffe16a` - record the no-changelog scope
  amendment after the initial changelog check failed;
- `8158603b30e87074326b5729bcf661678a4dccd5` - record the changelog-check
  amendment and reviewed evidence state.

The independent-review correction commit is identified separately by exact SHA
in the final implementation handoff after it is created.

The evidence-restoration correction commit is also identified separately by exact
SHA in the final implementation handoff after it is created.

## 4. Files changed

The cumulative branch changes are limited to:

```text
CHANGELOG.md
docs/engineering/agent-instructions/rollout-plan.md
docs/engineering/ai-delivery/tasks/ADR-0002-POST-MERGE-CORRECTION.md
docs/engineering/ai-delivery/implementation-reports/ADR-0002-APPROVE-implementation-report.md
docs/engineering/ai-delivery/implementation-reports/ADR-0002-POST-MERGE-CORRECTION-implementation-report.md
```

No file outside the approved documentation/control allowlist changed.

The bounded post-ready correction commit changes only:

```text
CHANGELOG.md
docs/engineering/ai-delivery/tasks/ADR-0002-POST-MERGE-CORRECTION.md
docs/engineering/ai-delivery/implementation-reports/ADR-0002-POST-MERGE-CORRECTION-implementation-report.md
```

The bounded evidence-restoration correction changes only:

```text
docs/engineering/ai-delivery/tasks/ADR-0002-POST-MERGE-CORRECTION.md
docs/engineering/ai-delivery/implementation-reports/ADR-0002-APPROVE-implementation-report.md
docs/engineering/ai-delivery/implementation-reports/ADR-0002-POST-MERGE-CORRECTION-implementation-report.md
```

## 5. Findings addressed

### P1: incomplete proposed issue bodies

Resolved. The ADR-0002 approval report now embeds the complete proposed body for
both issues, rather than abbreviated diffs.

Recorded proposed-body hashes:

```text
#765: da12243b2a1898fd3fd574aada1dede3296ff13f38943e4fbb78a3dcb5ae1a35
#766: f4e8aa7e855b2b3c44b4cf38c60475861079698cc7f5cd95a6ac319b892cb772
```

The embedded #765 proposal updates only the synchronization-guard baseline and
ADR-0002 checkbox. The embedded #766 proposal adds the guarded synchronization
section and updates only the ADR-0002 checkbox apart from that guard.

Neither issue was written.

### P1: contradictory rollout tracker

Resolved. `docs/engineering/agent-instructions/rollout-plan.md` now agrees with
`docs/engineering/README.md` that the `thoth` foundation closeout is complete,
issue #765 was synchronized on 2026-07-27 and remains open, and no foundation
closeout action remains in `thoth`.

### P1: trailing whitespace and inaccurate check claim

Resolved. The flawed embedded diff was removed. The replacement report was
constructed without trailing whitespace. Final review must verify the exact branch
with `git diff --check` before approval.

## 6. Post-ready automated review

Post-ready automated review completed:
`2026-07-28T12:06:55Z`

Reviewed head:
`ca8e90645957c50c25fecd8b220772837bb522d3`

Decision:
`CHANGES REQUIRED`

The review raised these three threads:

1. `PRRT_kwDODkn0bc6UZAcG` - mandatory task-specification fields. Valid and
   corrected by adding explicit Dependencies, Required tests, Migration effect
   and Stop conditions sections.
2. `PRRT_kwDODkn0bc6UZAcH` - required changelog entry. Valid and corrected; the
   earlier no-changelog amendment is superseded.
3. `PRRT_kwDODkn0bc6UZAcI` - claimed merged state. The finding was based on an
   incorrect premise: PR #770 was open and unmerged when the comment was posted.

PR #770 remains open, draft and unmerged.

No merge commit, merged timestamp, merged actor or post-merge PR #769 thread
state exists yet. Those facts must be recorded only after the merge occurs.

The implementation report is a pre-merge evidence record. Final merge and
post-merge closeout evidence will be recorded in GitHub comments after the
corresponding operations complete.

### Subsequent post-ready evidence-restoration review

Post-ready automated review timestamp:
`2026-07-28T13:33:57Z` / `2026-07-28T13:33:58Z`

Reviewed head:
`cb82ce799ca3afecf8f243faa7ebb9d10c5d049b`

Decision:
`CHANGES REQUIRED`

The review raised two valid P1 findings:

1. `PRRT_kwDODkn0bc6Ualwk` - restore mandatory ADR-0002 approval
   implementation evidence, including the 14-file per-path assessment, exact
   commands and truthful results, authorization assessment, rollout, rollback,
   known limitations and deferred work.
2. `PRRT_kwDODkn0bc6Ualwt` - record concrete implementing and independent
   reviewer agent/model identities in the task and implementation report.

The prior independent `APPROVED` decision at
`cb82ce799ca3afecf8f243faa7ebb9d10c5d049b` is superseded by these subsequent
P1 findings. The next exact head requires fresh CI and a fresh independent review.

PR #770 was converted back to draft before this evidence-restoration edit and
remains open, draft and unmerged.

## 7. Superseded scope amendment: no changelog

This internal engineering-control correction has no user-visible product effect.
The initial `check-changelog` run correctly failed because `CHANGELOG.md` was not
changed. The task specification was amended to use the repository-supported
`no changelog` label instead. Post-ready review confirmed that the root
`AGENTS.md` requires every PR to update `CHANGELOG.md`, so that amendment is
superseded before merge.

The corrected scope adds exactly one PR #770 entry under `## [Unreleased]` /
`### Changed`, removes the `no changelog` label before final CI, and does not
alter the runtime or migration scope.

## 8. Invariants

```text
ADR-0001: PROPOSED
ADR-0002: APPROVED
Publisher Services ADR-01: unapproved
Platform inventory: FINAL ENUM NOT APPROVED
MET-CTRL-01: CHANGES REQUIRED
All Publisher Services implementation tasks: BLOCKED
All Metrics work packages: BLOCKED
Issues #765 and #766: unchanged
No runtime effect
```

## 9. Runtime, migration and authorization effects

None. No Rust, SQL, migration, GraphQL, generated code, workflow, deployment,
repository-protection or authorization file changed. No deployment, release or
production activation occurred.

## 10. Issue-write controls

The complete proposed bodies are review evidence only. They do not authorize issue
writes. Each future write still requires an immediate live-body and `updatedAt`
re-fetch, exact baseline match, stop on mismatch, fresh review where needed and
separate explicit CTO authorization.

### Live read-only baselines

```text
Issue #765
state: OPEN
updatedAt: 2026-07-27T15:50:33Z
baseline body sha256:
96c31089a3046eadf51a0fc39b12d0275ce26f4d752c64282f5dcb933f78ca15

proposed body sha256:
da12243b2a1898fd3fd574aada1dede3296ff13f38943e4fbb78a3dcb5ae1a35

Issue #766
state: OPEN
updatedAt: 2026-07-24T17:17:11Z
baseline body sha256:
6b1bb092f3f0b436c01faaabbf4fb5df331268f4d687463b3c715fb4ea9d6dbc

proposed body sha256:
f4e8aa7e855b2b3c44b4cf38c60475861079698cc7f5cd95a6ac319b892cb772
```

Neither issue was written.

## 11. Tests and checks

### Reviewed-head whitespace check

Command:

```bash
git diff --check \
  e124221f8444bd738228f1b609c536639be8789e...8158603b30e87074326b5729bcf661678a4dccd5
```

Result:

```text
exit 0
no output
```

### Corrected working-tree whitespace check

Command:

```bash
git diff --check e124221f8444bd738228f1b609c536639be8789e
```

Result:

```text
exit 0
no output
```

The final handoff records the same check against the new committed and pushed
exact head.

### Cumulative changed-file boundary

Command:

```bash
git diff --name-only \
  e124221f8444bd738228f1b609c536639be8789e...HEAD
```

Result at reviewed evidence head `8158603b30e87074326b5729bcf661678a4dccd5`:

```text
docs/engineering/agent-instructions/rollout-plan.md
docs/engineering/ai-delivery/implementation-reports/ADR-0002-APPROVE-implementation-report.md
docs/engineering/ai-delivery/implementation-reports/ADR-0002-POST-MERGE-CORRECTION-implementation-report.md
docs/engineering/ai-delivery/tasks/ADR-0002-POST-MERGE-CORRECTION.md
```

### Rollout-plan cumulative diff

Command used against the corrected working tree:

```bash
git diff --unified=0 \
  e124221f8444bd738228f1b609c536639be8789e -- \
  docs/engineering/agent-instructions/rollout-plan.md
```

Result:

```text
exit 0
exactly two one-line replacement hunks:
- the thoth current-state row
- rollout-sequence item 1
```

All unrelated content is byte-for-byte restored from the base.

### Live issue baseline hashes

Commands:

```bash
gh api repos/thoth-pub/thoth/issues/765 \
  --jq '{state: .state, updatedAt: .updated_at}'
gh api repos/thoth-pub/thoth/issues/765 |
  jq -r .body |
  shasum -a 256

gh api repos/thoth-pub/thoth/issues/766 \
  --jq '{state: .state, updatedAt: .updated_at}'
gh api repos/thoth-pub/thoth/issues/766 |
  jq -r .body |
  shasum -a 256
```

`jq -r` emits the decoded issue body followed by a final newline, matching the
reviewed baseline-hash convention.

Results:

```text
#765: state open; updatedAt 2026-07-27T15:50:33Z
96c31089a3046eadf51a0fc39b12d0275ce26f4d752c64282f5dcb933f78ca15

#766: state open; updatedAt 2026-07-24T17:17:11Z
6b1bb092f3f0b436c01faaabbf4fb5df331268f4d687463b3c715fb4ea9d6dbc
```

### Embedded proposed-body hashes

Commands:

```bash
git show 8158603b30e87074326b5729bcf661678a4dccd5:docs/engineering/ai-delivery/implementation-reports/ADR-0002-APPROVE-implementation-report.md |
  awk '/^## 8\. Exact proposed body/{section=1}
       section && /^```markdown$/{capture=1;next}
       capture && /^```$/{exit}
       capture{print}' |
  shasum -a 256

git show 8158603b30e87074326b5729bcf661678a4dccd5:docs/engineering/ai-delivery/implementation-reports/ADR-0002-APPROVE-implementation-report.md |
  awk '/^## 9\. Exact proposed body/{section=1}
       section && /^```markdown$/{capture=1;next}
       capture && /^```$/{exit}
       capture{print}' |
  shasum -a 256
```

The `awk` `print` action preserves the embedded lines and supplies the final
newline included in each reviewed proposed-body hash.

Results:

```text
#765: da12243b2a1898fd3fd574aada1dede3296ff13f38943e4fbb78a3dcb5ae1a35
#766: f4e8aa7e855b2b3c44b4cf38c60475861079698cc7f5cd95a6ac319b892cb772
```

The restored current report preserves the same complete bodies. Working-tree
verification commands:

```bash
awk '/^## 15\. Exact proposed body/{section=1}
     section && /^```markdown$/{capture=1;next}
     capture && /^```$/{exit}
     capture{print}' \
  docs/engineering/ai-delivery/implementation-reports/ADR-0002-APPROVE-implementation-report.md |
  shasum -a 256

awk '/^## 16\. Exact proposed body/{section=1}
     section && /^```markdown$/{capture=1;next}
     capture && /^```$/{exit}
     capture{print}' \
  docs/engineering/ai-delivery/implementation-reports/ADR-0002-APPROVE-implementation-report.md |
  shasum -a 256
```

Results:

```text
#765: da12243b2a1898fd3fd574aada1dede3296ff13f38943e4fbb78a3dcb5ae1a35
#766: f4e8aa7e855b2b3c44b4cf38c60475861079698cc7f5cd95a6ac319b892cb772
```

## 12. Concrete CI for reviewed evidence head

All required workflows and jobs succeeded at
`8158603b30e87074326b5729bcf661678a4dccd5`:

```text
30348170518 - build-test-and-check: success
  build: success
  test: success
  lint: success
  format_check: success

30348170455 - run-migrations: success
  run_migrations: success

30348170375 - check-changelog: success
  check-changelog: success

30348170534 - publish-to-dockerhub: success
  build_and_push_staging_docker_image: success
```

The `build`, `test`, `lint`, `format_check` and `run_migrations` jobs used their
existing lightweight `Run echo "No build required"` paths. The Docker workflow
performed a real registry login and `Build and push` operation successfully.

## 13. Final-head evidence model

The report records complete concrete CI evidence for reviewed evidence head
`8158603b30e87074326b5729bcf661678a4dccd5`.

This correction creates a new final PR head. Its exact SHA and fresh workflow
run/job IDs are live GitHub evidence produced only after this commit is pushed.
They must be recorded in the final implementation handoff and top-level PR
comment, and independently verified before approval.

The previous CI is not reused for the new head. All required jobs must succeed at
the new exact head before fresh independent review.

## 14. Residual blockers and independent review

All required jobs must succeed at the exact final PR head. The designated
independent reviewer, ChatGPT / GPT-5.6 Thinking operating in a fresh
non-implementing context at High reasoning, must inspect the full cumulative diff
and return exactly `APPROVED`, `CHANGES REQUIRED` or `BLOCKED` before merge. If
the actual final reviewer/model differs, the review must record the actual
concrete identity and stop until the task record is explicitly updated.

The implementing context must not approve or merge PR #770.

PR #770 remains draft. The three PR #769 review threads remain unresolved until a
separately authorized post-merge action. Issues #765 and #766 remain open and
unchanged.

## 15. Rollout and rollback

Rollout is merge-only documentation reconciliation. No activation, deployment,
migration or production write follows.

Rollback is a normal revert of PR #770. No issue rollback is required because
neither issue is changed by this task.

## 16. Post-merge action

After PR #770 is independently approved and merged, reply to each of the three
unresolved PR #769 review threads with the corrective merge commit and resolve the
threads. Do not edit either programme issue as part of that action.
