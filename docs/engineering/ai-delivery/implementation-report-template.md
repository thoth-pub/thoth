# Implementation Report Template

The implementing agent completes this report after pushing the task branch and opening or updating the draft PR.

Do not write `passed` without the exact command and result.

# [TASK-ID] Implementation Report

## 1. Repository state

Owning GitHub issue:
Repository:
Workflow: STANDARD | PROGRAMME_INTEGRATION
Base branch:
Authorized base commit:
Actual base commit:
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

Authorized write paths (from the task specification):

- [path]
- [...]

Authorized new-file paths:

- [path]
- [...]

Actual files changed, for each material file:

- `path`
  - reason:
  - behavioural effect:
  - within authorized write budget: YES | NO

Actual new files created:

- `path` - within authorized new-file list: YES | NO
- [...]

Files deleted, moved or renamed: NONE | [list and link to the authorization
that permitted it]

### 4.1 Write-budget compliance

PASS | FAIL

[If FAIL, list every file changed outside the authorized write/new-file paths
and treat this as an unauthorized action per section 4.2, not as a normal
deviation.]

## 4.2 Authorized actions actually used

For each action in the task specification's action-authorization matrix,
record whether it was actually used:

- repository inspection:
- source edit:
- new file creation:
- file deletion/move/rename:
- branch creation:
- commit:
- push:
- PR creation/update:
- issue/comment mutation:
- manual CI dispatch/rerun:
- provider/runtime read:
- provider/runtime write:
- migration execution:
- release/tag/publication:
- merge:
- deployment:
- production activation:
- other:

Unauthorized actions performed: NONE | [list explicitly; this is a stop
condition, not a routine finding]

## 4.3 Automatic and manual external effects

Automatic CI/provider effects observed (for example a workflow triggered by
opening the PR, and whether it performed any external write such as a
container-registry push):

[...]

Manually initiated external actions (anything the implementing agent
triggered outside the normal push/PR flow, e.g. a manual workflow dispatch):
NONE | [list and link to the authorization that permitted it]

External writes/publication (releases, tags, packages, registries, third-party
services): NONE | [list]

## 5. Implementation decisions

List decisions made within the approved design:

1. [...]
2. [...]

List any deviation from the specification requiring authorization:

- NONE; or
- [deviation, reason, whether it was authorized and by whom]

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
