# Implementation Report Template

The implementing agent completes this report after pushing the task branch and opening or updating the draft PR.

Do not write `passed` without the exact command and result.

# [TASK-ID] Implementation Report

## 1. Repository state

Repository:
Workflow: STANDARD | PROGRAMME_INTEGRATION
Base branch:
Base commit:
PR target:
Programme integration branch:
Task branch:
Head commit:
Pull request:
Expected branch deletion after merge: YES
Final programme PR required: YES | NO
Implementing model:
Reasoning level:

## 2. Scope confirmation

Approved specification:
Implemented objective:

Out-of-scope changes made: NONE | [explain and link to approval]

## 3. Commits

- `[sha]` - [message]
- [...]

## 4. Files changed

For each material file:

- `path`
  - reason:
  - behavioural effect:

## 5. Implementation decisions

List decisions made within the approved design:

1. [...]
2. [...]

List any deviation from the specification:

- NONE; or
- [deviation, reason, approval status]

## 6. Database and migration effects

Migration added: YES | NO

If yes:

- migration files:
- schema effect:
- existing-data effect:
- locking/downtime:
- empty database result:
- populated database result:
- rollback/forward repair:
- idempotency:

## 7. API and compatibility effects

GraphQL/API changes:
Generated schema/client updates:
Backwards compatibility:
Deprecations:
Cross-repository dependencies:

## 8. Authorization and security

Authorization paths changed:
Roles/scopes involved:
Negative authorization tests:
Secret or personal-data handling:
Security limitations:

## 9. Tests and checks

Record exact commands and outcomes.

### Formatting

Command:

```text
[command]
```

Result:

```text
[exact concise result]
```

### Unit tests

Command:

```text
[command]
```

Result:

```text
[exact concise result]
```

### Integration/database tests

Command:

```text
[command]
```

Result:

```text
[exact concise result]
```

### Lint/static analysis

Command:

```text
[command]
```

Result:

```text
[exact concise result]
```

### Other required checks

[...]

## 10. Manual verification

Environment:
Steps:
Observed result:
Evidence link/screenshot/log reference:

## 11. CI

CI status: PASSING | FAILING | PENDING | NOT AVAILABLE
Checks:
Failures or warnings:

## 12. Rollout and rollback

Initial state after merge:
Activation required:
Feature flag/configuration:
Migration sequence:
Rollback/disable procedure:
Monitoring required:

## 13. Known limitations and deferred work

- [...]
- [...]

## 14. Unresolved issues

- NONE; or
- [...]

## 15. Agent self-assessment

The agent may identify risks but may not approve the task.

Suggested review focus:

- [...]
- [...]
