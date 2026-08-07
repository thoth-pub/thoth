# Task Specification Template

Copy this file to the relevant programme task directory and replace every bracketed field.

A task may not enter implementation with unresolved required fields.

Write durable state only. Do not record transient pull-request lifecycle status
such as `PENDING MERGE` or `AWAITING CTO MERGE AUTHORIZATION`: GitHub is the
live authority for those, and recording them here guarantees the file is stale
after merge. See `../AGENTS.md` section 1.1 and `operating-model.md` section 5.1.

---

# [TASK-ID] - [Task title]

Status: DRAFT | APPROVED
Programme: [programme]
Repository: [owner/repository]
Workflow: STANDARD | PROGRAMME_INTEGRATION
Base branch: [normally `develop`; otherwise approved `feature/<programme>`]
PR target: [`develop` or approved `feature/<programme>`]
Programme integration branch: [branch or `None`]
Risk: LOW | MEDIUM | HIGH | CRITICAL
Owner: [role/person]
Approved by: [CTO/approver]
Dependencies: [task IDs, PRs, releases or `None`]
Target branch name: `feature/[programme-or-area]/[task-id-or-short-name]`

Authority condition: this record is repository-authoritative when this exact
content is reachable from the repository's authoritative integration branch.
Live review, authorization and merge evidence is the GitHub pull-request record.

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

## 5. Invariants

The implementation must preserve:

1. [...]
2. [...]
3. [...]

## 6. Required behaviour

### 6.1 Success behaviour

[...]

### 6.2 Failure behaviour

[...]

### 6.3 Authorization

[...]

### 6.4 Concurrency and idempotency

[State requirements or `Not applicable`.]

### 6.5 Compatibility

[API, database, client, deployment and backwards-compatibility requirements.]

## 7. Data and migration requirements

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

## 8. Observability and operations

Required logs:

[...]

Required metrics/alerts:

[...]

Operational runbook changes:

[...]

## 9. Acceptance criteria

Use verifiable statements.

- [ ] [...]
- [ ] [...]
- [ ] [...]

## 10. Required tests

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

## 11. Rollout

- initial state after merge:
- feature flag/configuration:
- staging/preview validation:
- pilot:
- activation approval:
- observation period:

## 12. Rollback

- code rollback:
- data rollback or forward repair:
- feature disable/kill switch:
- external side-effect handling:

## 13. Stop conditions

The implementing agent must stop and report `BLOCKED` if:

- [...]
- [...]
- approved architecture would need to change;
- required production information or secrets are unavailable;
- scope cannot be completed without unrelated changes.

## 14. Expected implementation report

The agent must use:

`docs/engineering/ai-delivery/implementation-report-template.md`

## 15. Recommended execution

Implementation model: [model]
Reasoning level: [level]
Independent reviewer: [model/family]
Review reasoning level: [level]

## 16. Branch and integration plan

- branch source:
- pull-request target:
- expected merge order:
- parent programme branch refresh requirement:
- branch deletion after merge: YES
- final programme PR required: YES | NO
- final release path: `develop -> master`

## 17. Approval

Approved for implementation by:
Date:
Notes:

Record only the durable implementation authorization here. Independent review
decisions, CTO merge authorization and the merge itself are terminal GitHub
evidence under `ADR-0005` and must not be copied back into this file.
