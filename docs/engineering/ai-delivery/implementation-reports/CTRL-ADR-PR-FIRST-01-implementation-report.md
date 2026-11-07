# CTRL-ADR-PR-FIRST-01 Implementation Report

This report records the bounded Shared Engineering Control task that adds a
PR-first staged approval route for material pre-authority ADR corrections. It
records durable task evidence only. Live head, CI, review, authorization and
merge state is GitHub-owned lifecycle evidence under `ADR-0005` and is not
copied here.

## 1. Repository state

Owning GitHub issue: [#966](https://github.com/thoth-pub/thoth/issues/966)
Repository: `thoth-pub/thoth`
Workflow: STANDARD
Base branch: `develop`
Authorized base commit: `8ac771c55937f7a4fe4131b3c21c466f0b4d7abc`
Actual base commit: `8ac771c55937f7a4fe4131b3c21c466f0b4d7abc`
PR target: `develop`
Programme integration branch: NOT APPLICABLE (STANDARD workflow)
Task branch: `feature/engineering-control/adr-pr-first-review`
Head commit: GitHub-owned lifecycle evidence; the pull request head for this
task branch is the authoritative head and is not self-pinned in this report.
Pull request: the draft pull request opened from the task branch against
`develop` for issue #966; its number and head are GitHub-owned.
Expected branch deletion after merge: YES
Final programme PR required: NO
Implementing model: Claude (Fable 5.1), bounded implementing agent
Reasoning level: standard

## 2. Scope confirmation

Approved specification: issue #966 body, CTO-approved 2026-10-02.
Implemented objective: extend the repository-authoritative ADR amendment
process in `docs/engineering/decisions/README.md` so a material correction
before repository authority may use either the existing pre-commit
exact-final-blob approval route (Route A, unchanged) or a new PR-first staged
approval route (Route B), with the candidate, content-approval,
approval-state, exact-final-blob, independent-review and merge stages kept
distinct, and with an explicit explanation of why the Route B approval-state
commit is consistent with `ADR-0005`.

Out-of-scope changes made: NONE

## 3. Commits

- one bounded commit on `feature/engineering-control/adr-pr-first-review`
  whose direct parent is `8ac771c55937f7a4fe4131b3c21c466f0b4d7abc` -
  `CTRL-ADR-PR-FIRST-01: add PR-first staged ADR correction approval`

The commit SHA is the pull-request head recorded by GitHub. It is not embedded
here because this report is part of that commit.

## 4. Files changed

Authorized write paths (from the task specification):

- `docs/engineering/decisions/README.md`
- `CHANGELOG.md`

Authorized new-file paths:

- `docs/engineering/ai-delivery/implementation-reports/CTRL-ADR-PR-FIRST-01-implementation-report.md`

Actual files changed, for each material file:

- `docs/engineering/decisions/README.md`
  - reason: add the PR-first staged approval route while retaining the
    existing pre-commit exact-final-blob route.
  - behavioural effect: documentation/control doctrine only. The former
    `#### Exact-final-blob approval` section becomes `#### Approval routes for
    a material correction` (route selection and common gates), `#### Route A -
    pre-commit exact-final-blob approval` (the existing nine-step sequence,
    text unchanged), `#### Route B - PR-first staged approval` (six stages,
    fail-closed conditions and the `ADR-0005` rationale) and `#### Rules
    common to both routes` (the pre-existing closing paragraphs, text
    unchanged). The eligibility test, the stale-control list, the factual
    clarification rules, the `ADR-0013` references, the repository-authority
    condition and the final supersession and no-runtime-authorization
    paragraphs are unchanged.
  - within authorized write budget: YES
- `CHANGELOG.md`
  - reason: required changelog entry.
  - behavioural effect: one `CTRL-ADR-PR-FIRST-01` entry added under
    `[Unreleased] -> Changed`; no new heading; existing entries preserved.
  - within authorized write budget: YES

Actual new files created:

- `docs/engineering/ai-delivery/implementation-reports/CTRL-ADR-PR-FIRST-01-implementation-report.md` - within authorized new-file list: YES

Files deleted, moved or renamed: NONE

### 4.1 Write-budget compliance

PASS

Exactly the three authorized paths differ from the authorized base. No ADR,
runtime, migration, schema, API, authorization, workflow or settings path is
touched.

## 4.2 Authorized actions actually used

- repository inspection: USED (Git history, `develop` doctrine, issue #966)
- source edit: USED (the two authorized existing paths only)
- new file creation: USED (this report only)
- file deletion/move/rename: NOT USED (not authorized)
- branch creation: USED (`feature/engineering-control/adr-pr-first-review`
  from exact `8ac771c55937f7a4fe4131b3c21c466f0b4d7abc`, after confirming the
  ref and its namespace were absent locally and remotely)
- commit: USED (one commit)
- push: USED (one non-force push of the task branch)
- PR creation/update: USED (one draft pull request targeting `develop`)
- issue/comment mutation: NOT USED (not authorized)
- manual CI dispatch/rerun: NOT USED (not authorized)
- provider/runtime read: NOT USED (not authorized)
- provider/runtime write: NOT USED (not authorized)
- migration execution: NOT USED (not authorized)
- release/tag/publication: NOT USED (not authorized)
- merge: NOT USED (not authorized; the implementing agent must not merge its
  own work)
- deployment: NOT USED (not authorized)
- production activation: NOT USED (not authorized)
- other: NONE

Unauthorized actions performed: NONE

## 4.3 Automatic and manual external effects

Automatic CI/provider effects observed (for example a workflow triggered by
opening the PR, and whether it performed any external write such as a
container-registry push):

Opening the pull request triggers the repository's `pull_request` workflows
(`build-test-and-check`, `check-changelog`, `publish-to-dockerhub`,
`run-migrations`) and the configured automatic Codex/app review. The change
set consists solely of `CHANGELOG.md` and `docs/**` paths, which
`.github/scripts/classify_ci_changes.py` classifies as documentation-only, so
the build, test, lint, format, migration and Docker build/push jobs are
expected to be skipped and `classify` and `check-changelog` are expected to
pass. No container-registry push is expected. The actual job outcomes for the
final head are GitHub-owned and are recorded in the pull request.

Manually initiated external actions (anything the implementing agent
triggered outside the normal push/PR flow, e.g. a manual workflow dispatch):
NONE

External writes/publication (releases, tags, packages, registries, third-party
services): NONE

## 5. Implementation decisions

List decisions made within the approved design:

1. Route A keeps the existing nine-step text verbatim under its own heading so
   the pre-commit route is preserved without ambiguity; only the heading and
   the introductory sentence change.
2. Route selection is recorded by the CTO in the correction's task
   authorization before any correction commit, so an implementing agent never
   chooses between routes.
3. The approval date carried by the Route B approval-state blob is defined as
   the date of the durable content-approval record, which already exists
   before the approval-state commit. This avoids approval-date recursion
   between the blob and the later exact-final-blob record, while retaining the
   Route A rule that a mismatched date requires a new blob and fresh
   exact-final-blob approval.
4. The Route B approval-state commit is constrained to a direct child of the
   content-approved head and to an enumerated metadata delta; any other ADR
   byte change returns the correction to the candidate stage.
5. Companion durable records (decision-register row, changelog, implementation
   report) must describe the corrected version as proposed during the
   candidate stage, so no committed file asserts approval before the
   decision owner has approved the content.
6. The `ADR-0005` rationale distinguishes the status/approver/date transition,
   which only the repository can carry, from lifecycle identifiers, which
   remain GitHub-owned and must not be transcribed.

List any deviation from the specification requiring authorization:

- NONE

## 6. Database and migration effects

Migration added: NO

No schema, data, index, constraint or `thoth-api/src/schema.rs` effect.

## 7. API and compatibility effects

GraphQL/API changes: NONE
Generated schema/client updates: NONE
Backwards compatibility: unaffected; documentation/control only
Deprecations: NONE
Cross-repository dependencies: NONE. This is shared engineering-control
doctrine owned by `thoth-pub/thoth`; no downstream repository requires a
source change.

## 8. Authorization and security

Authorization paths changed: NONE
Roles/scopes involved: NONE
Negative authorization tests: NOT APPLICABLE
Secret or personal-data handling: NONE
Security limitations: NONE

## 9. Tests and checks

Record exact commands and outcomes.

### Formatting

Command:

```text
git diff --check
```

Result:

```text
exit 0; no output (no whitespace errors)
```

### Unit tests

Command:

```text
NOT APPLICABLE - documentation/control-only change; no Rust source changed
```

Result:

```text
NOT RUN
```

### Integration/database tests

Command:

```text
NOT APPLICABLE - no migration, schema or database code changed
```

Result:

```text
NOT RUN
```

### Lint/static analysis

Command:

```text
NOT APPLICABLE - no Rust source changed
```

Result:

```text
NOT RUN
```

### Other required checks

Command:

```text
git status --short
```

Result (before commit, after staging the three paths):

```text
M  CHANGELOG.md
A  docs/engineering/ai-delivery/implementation-reports/CTRL-ADR-PR-FIRST-01-implementation-report.md
M  docs/engineering/decisions/README.md
```

Command:

```text
git diff --name-only 8ac771c55937f7a4fe4131b3c21c466f0b4d7abc...HEAD
```

Result (after commit):

```text
CHANGELOG.md
docs/engineering/ai-delivery/implementation-reports/CTRL-ADR-PR-FIRST-01-implementation-report.md
docs/engineering/decisions/README.md
```

Additional checks:

- the `[Unreleased]` section retains exactly one `### Added`, one
  `### Changed` and one `### Fixed` heading;
- the README heading outline is `## Amendments` -> `### Factual
  clarification` -> `### Material correction before repository authority` ->
  `#### Approval routes for a material correction` -> `#### Route A -
  pre-commit exact-final-blob approval` -> `#### Route B - PR-first staged
  approval` -> `#### Rules common to both routes`;
- the Route A nine-step list is byte-identical to the previous
  `#### Exact-final-blob approval` list;
- internal links and ADR identifiers (`ADR-0005`, `ADR-0013`) resolve to
  existing files;
- no ADR file, workflow, migration, runtime or settings path differs from the
  base.

## 10. Manual verification

Environment: local Git worktree of `thoth-pub/thoth` on the task branch,
created from exact `8ac771c55937f7a4fe4131b3c21c466f0b4d7abc` after verifying
`origin/develop` resolved to that commit and that the task ref and its
namespace were absent.
Steps: edit the two authorized existing paths; create this report; run the
validation commands above; inspect the full diff for the invariants listed in
issue #966.
Observed result: three-path footprint; no weakening of exact-content approval,
exact-head independent review, merge authorization, material-correction
eligibility or `ADR-0013` drift controls; Route A preserved; no transient
lifecycle prose that review or merge would falsify.
Evidence link/screenshot/log reference: the pull request diff for issue #966.

## 11. CI

CI status: GitHub-owned lifecycle evidence for the pull-request head; expected
documentation-only classification (`classify` and `check-changelog` pass;
build, test, lint, format, migration and Docker jobs skipped).
Checks: as listed in section 4.3.
Failures or warnings: NONE EXPECTED; actual results are recorded on the pull
request.

## 12. Rollout and rollback

Initial state after merge: shared doctrine permits either Route A (pre-commit
exact-final-blob approval) or Route B (PR-first staged approval) for an
eligible material pre-authority ADR correction. No ADR changes state as a
result of this merge.
Activation required: NONE
Feature flag/configuration: NONE
Migration sequence: NONE
Rollback/disable procedure: before merge, close the pull request and discard
the task branch; after merge, a separately approved Shared Engineering Control
correction. No history rewrite.
Monitoring required: NONE

## 13. Known limitations and deferred work

- This task changes shared doctrine only. Any task that wants to use Route B,
  including `THOTH-ASYNC-01-ADR-01` (#958), still needs its own specification
  or amendment selecting the route, its own authorizations and its own
  reviews; nothing here reauthorizes or alters #958 or PR #960.
- The doctrine becomes repository-authoritative only when this exact content
  is reachable from `develop`; it is not effective from the unmerged branch.

## 14. Unresolved issues

- NONE

## 15. Agent self-assessment

The agent may identify risks but may not approve the task.

The implementing agent does not approve, mark merge-ready or merge its own
work. Fresh independent CRITICAL exact-head review and separate CTO merge
authorization remain required.

Suggested review focus:

- that Route A text is byte-identical to the previous exact-final-blob
  sequence;
- that Route B cannot let corrected unapproved bytes carry current `APPROVED`
  status (candidate stage `PROPOSED`, approval-state commit only after a
  durable content-approval record for the exact parent head);
- that the approval-date rule (date of the content-approval record) avoids
  recursion while still forcing a new blob and fresh exact-final-blob approval
  on mismatch;
- that the `ADR-0005` rationale does not open a path for transcribing
  lifecycle identifiers into repository files;
- that eligibility, stale-control, `ADR-0013` and repository-authority text is
  unchanged.
