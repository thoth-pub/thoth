# Task Specification Template

This template is canonical shared doctrine maintained in `thoth-pub/thoth`. It
may be copied into, or referenced by, any repository that adopts these AI-led
delivery controls. Copy this file to the relevant programme task directory and
replace every bracketed field.

A task may not enter implementation with unresolved required fields.

When completing this template for a task in a repository other than
`thoth-pub/thoth`, distinguish two separate authority sources and do not
conflate them:

1. the **target repository's own** root and nested `AGENTS.md` files and
   local controls, which govern that repository and are authoritative for its
   own conventions, stack and prohibited assumptions;
2. this **canonical shared doctrine**, maintained in `thoth-pub/thoth` under
   `docs/engineering/ai-delivery/` and `docs/engineering/repository-map/`,
   consulted when the task requires the shared cross-repository control model
   (action authorization, cross-repository impact analysis, lifecycle
   evidence). Do not assume a `docs/engineering/...` directory exists in the
   target repository itself; it may not.

Write durable state only. Do not record transient pull-request lifecycle status
such as `PENDING MERGE` or `AWAITING CTO MERGE AUTHORIZATION`: GitHub is the
live authority for those, and recording them here guarantees the file is stale
after merge. See `../AGENTS.md` section 1.1 and `operating-model.md` section
5.1 — both canonical shared-doctrine references in `thoth-pub/thoth`, per the
distinction above, not necessarily paths that exist in the target repository.

---

# [TASK-ID] - [Task title]

Status: DRAFT | APPROVED
Programme: [programme]
Stage: [programme stage/phase, or `None`]
Owning GitHub issue: [owner/repository#NNN]
Repository: [owner/repository]
Workflow: STANDARD | PROGRAMME_INTEGRATION
Base branch: [the target repository's verified repository-local base branch —
  for `thoth-pub/thoth` this is normally `develop`; for any other repository,
  verify it live and record it here rather than assuming `develop` — or an
  approved `feature/<programme>` integration branch]
Exact authorized base commit: [full 40-character SHA verified immediately before implementation]
PR target: [the same verified repository-local base branch, or the approved `feature/<programme>` integration branch]
Programme integration branch: [branch or `None`]
Risk: LOW | MEDIUM | HIGH | CRITICAL
Owner: [role/person]
Approved by: [CTO/approver]
Dependencies: [task IDs, PRs, releases or `None`]
Target branch name: `feature/[programme-or-area]/[task-id-or-short-name]`

Authority condition: this record is repository-authoritative when this exact
content is reachable from the repository's authoritative integration branch.
Live review, authorization and merge evidence is the GitHub pull-request record.

If the verified current head of the base branch differs from the exact
authorized base commit above, implementation must not silently rebase the
authorization: return `HOLD - AUTHORIZED BASE MOVED` with the authorized SHA,
the current SHA, the intervening commits and whether they appear relevant.

## 1. Objective

[One paragraph describing the outcome, not the implementation method.]

## 2. Background and authority

Authoritative sources:

- [design/ADR/link]
- [related issue/link]
- [relevant repository path]

Current behaviour:

[Brief evidence-based description.]

## 3. Explicit scope

The task must:

1. [...]
2. [...]
3. [...]

## 4. Non-goals

The task must not:

1. [...]
2. [...]
3. [...]

## 5. Cross-repository impact

Required for every substantive task. A task is not single-repository merely
because the initiating request or issue originated in one repository.

Affected contracts (mark each `AFFECTED` or `NOT AFFECTED`):

- database/domain model:
- GraphQL/API schema and behaviour:
- generated clients/types:
- authorization semantics:
- export formats:
- configuration/environment contracts:
- event/job payloads:
- dissemination/platform behaviour:
- UI assumptions:
- CMS/site contracts:
- package/library interfaces:
- deployment/compatibility windows:

If every contract above is `NOT AFFECTED`, state that conclusion explicitly
and skip the remaining fields in this section; a tiny, clearly single-surface
task does not need every repository mechanically listed.

If any contract is `AFFECTED`:

- owning repository for the changed contract: [owner/repository]
- known consumers, from `docs/engineering/repository-map/contracts.md` and
  verified live evidence (never inferred from a repository's name):
  - [owner/repository] - [REQUIRES CHANGE, tracked as TASK-ID/issue, or
    REMAINS COMPATIBLE - reason]
  - [...]
- compatibility requirements (what must keep working, and for how long,
  across the affected repositories):
- dependency, merge and deployment order across affected repositories:
- downstream repository-local tasks created or referenced: [task
  IDs/issues, or `None required` with reason]

Do not let a downstream repository guess an unmerged upstream contract. Each
affected repository gets its own bounded task, branch and pull request; do not
give one implementing agent unrestricted write access to more than one
repository under this task.

## 6. Invariants

The implementation must preserve:

1. [...]
2. [...]
3. [...]

## 7. Required behaviour

### 7.1 Success behaviour

[...]

### 7.2 Failure behaviour

[...]

### 7.3 Authorization

[...]

### 7.4 Concurrency and idempotency

[State requirements or `Not applicable`.]

### 7.5 Compatibility

[API, database, client, deployment and backwards-compatibility requirements.]

## 8. Data and migration requirements

Migration required: YES | NO

If yes:

- schema changes:
- populated database behaviour:
- locking/downtime assessment:
- data backfill:
- idempotency:
- rollback or forward-repair strategy:
- empty database test:
- populated database test:

## 9. Observability and operations

Required logs:

[...]

Required metrics/alerts:

[...]

Operational runbook changes:

[...]

## 10. Acceptance criteria

Use verifiable statements.

- [ ] [...]
- [ ] [...]
- [ ] [...]

## 11. Required tests

### Unit

- [...]

### Integration/database

- [...]

### Authorization/security

- [...]

### Regression

- [...]

### Manual verification

- [...]

### Performance

[Required target or `Not applicable`.]

## 12. Rollout

- initial state after merge:
- feature flag/configuration:
- staging/preview validation:
- pilot:
- activation approval:
- observation period:

## 13. Rollback

- code rollback:
- data rollback or forward repair:
- feature disable/kill switch:
- external side-effect handling:

## 14. Authorized write paths, new files and prohibited paths

Existing files this task may modify:

- [path]
- [...]

New files this task may create:

- [path]
- [...]

Prohibited paths: file deletion, move or rename is prohibited unless
explicitly listed here as authorized. Any other path is denied by default.

If implementation discovers that another path is required, it must stop and
return `HOLD` with the proposed path and reason rather than writing to it.

## 15. Action authorization matrix

Mark each action `YES` or `NO` for this task. Authorization is granted
action-by-action and is not transitive: see root `AGENTS.md` section 6 and
`operating-model.md` section 2.4. Unlisted mutations are denied.

| Action | Authorized |
|---|---|
| repository/GitHub read inspection | |
| source/worktree edits within the write budget | |
| create the authorized new files | |
| delete/move/rename files | NO |
| branch creation from the exact authorized base | |
| commit | |
| push | |
| open/update draft PR | |
| issue/comment mutation | |
| manual CI dispatch/rerun | NO |
| provider/runtime read | |
| provider/runtime write | NO |
| migration execution | NO |
| release/tag/publication | NO |
| merge | NO |
| deployment | NO |
| production activation | NO |

## 16. Automatic side effects

State any automatic effect that an authorized action is expected to trigger —
for example CI workflows that run on PR open/update, and whether any of them
are capable of a write to an external system (such as a container registry).
State the expected/observed behaviour for documentation-only or otherwise
low-risk diffs, and state explicitly that no additional manual dispatch of
that workflow is authorized.

[...]

## 17. HOLD/STOP conditions

Return `HOLD` for a temporary dependency/evidence/authorization/environment
blocker, and `BLOCKED` where the approved task cannot proceed safely as
specified. Do not reinterpret authorization to work around either.

The implementing agent must stop and report `HOLD` or `BLOCKED` if:

- [...]
- [...]
- the authorized base has moved and has not been freshly reconciled;
- approved architecture would need to change;
- required production information or secrets are unavailable;
- another repository needs source modification to complete this task;
- another file path is required outside the write budget;
- a file deletion/move/rename appears necessary;
- repository ownership or a consumer relationship would need to be guessed;
- scope cannot be completed without unrelated changes.

## 18. Expected implementation report

The agent must use:

`docs/engineering/ai-delivery/implementation-report-template.md`

For a coding-agent handoff, prepare the bounded prompt using
`docs/engineering/ai-delivery/implementation-handoff-template.md`, carrying
forward this specification's write budget, action-authorization matrix and
cross-repository impact fields.

## 19. Recommended execution

Implementation model: [model]
Reasoning level: [level]
Independent reviewer: [model/family]
Review reasoning level: [level]

## 20. Branch and integration plan

- branch source:
- pull-request target:
- expected merge order:
- parent programme branch refresh requirement:
- branch deletion after merge: YES
- final programme PR required: YES | NO
- final release path: [the target repository's verified repository-local
  release path, or `None` if that repository has no separate release branch —
  for `thoth-pub/thoth` this is `develop -> master`; do not assume that path
  for any other repository]

## 21. Approval

Approved for implementation by:
Date:
Notes:

Record only the durable implementation authorization here. Independent review
decisions, CTO merge authorization and the merge itself are terminal GitHub
evidence under `ADR-0005` and must not be copied back into this file.
