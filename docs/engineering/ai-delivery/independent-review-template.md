# Independent Review Template

Review the actual diff and evidence. Do not approve from the implementation report alone.

The review is bound to the exact reviewed commit. If that head changes for any
repository commit, substantive or not, this review does not carry forward and
fresh review is required.

Record the review on the pull request. Under `ADR-0005` the review decision is
terminal GitHub evidence: do not commit it to a repository file, which would
move the head and invalidate the review itself. The reviewer must not be the
agent instance that implemented the task.

# [TASK-ID] Independent Review

Reviewer model:
Reasoning level:
Authorized base branch/commit:
Reviewed commit (exact head):
Pull request:
Specification:

## 1. Decision

APPROVED | CHANGES REQUIRED | BLOCKED

## 2. Scope and specification coverage

- Does the diff implement every in-scope requirement?
- Are any non-goals implemented?
- Are any acceptance criteria unsupported by evidence?
- Does the implementation silently change architecture?

Findings:

[...]

## 2.1 Base, head and write-budget verification

- Verify the base branch/commit the task branch was actually created from
  matches the specification's authorized base; if it does not, this is a
  blocking finding, not a note.
- Verify the reviewed commit is the exact current PR head; if the head has
  moved since this review began, restart the review at the new head rather
  than approving a stale diff.
- Compare the actual diff file-by-file against the specification's authorized
  write paths and authorized new-file list. Every changed or created file must
  appear on one of those lists.
- Verify no file was deleted, moved or renamed unless explicitly authorized.
- Verify no path outside the write budget was touched, including
  configuration, CI workflow, branch-protection or provider/runtime files
  unless explicitly authorized.

Findings:

[...]

## 2.2 Action authorization compliance

Compare the implementation report's "authorized actions actually used" against
the specification's action-authorization matrix.

- Was any action performed that the matrix marks `NO` or leaves unauthorized?
- Was commit/push/PR/issue-mutation/CI-dispatch/provider-write/merge/
  deployment/activation authorization ever assumed from a different,
  unrelated authorization (non-transitivity violation)?
- Are automatic external effects (CI runs, container/package publication)
  disclosed and consistent with what the specification predicted?
- Were any manual external actions taken that were not explicitly authorized?

Findings:

[...]

## 3. Correctness

Assess:

- normal success paths;
- failure paths;
- boundary conditions;
- state transitions;
- error handling;
- pagination and ordering where relevant;
- partial failure;
- restart/retry behaviour.

Findings:

[...]

## 4. Data and migration safety

Assess:

- empty and populated database behaviour;
- constraints;
- defaults and nullability;
- backfill effects;
- locks and downtime;
- idempotency;
- rollback or forward repair;
- accidental side effects.

Findings:

[...]

## 5. Authorization and security

Assess:

- positive and negative authorization;
- tenant/publisher isolation;
- machine-role scope;
- secrets and logs;
- untrusted input;
- personal data;
- fail-open or scope-broadening behaviour.

Findings:

[...]

## 6. Concurrency and idempotency

Assess:

- races;
- duplicate processing;
- leases/claim tokens;
- stale workers;
- retry safety;
- uniqueness enforcement;
- transaction boundaries.

Findings:

[...]

## 7. API and compatibility

Assess:

- backwards compatibility;
- GraphQL/schema changes;
- generated clients;
- cross-repository consumers;
- error-contract changes;
- rollout ordering.

Findings:

[...]

### 7.1 Cross-repository impact and downstream compatibility

- Does the specification's cross-repository impact assessment match the
  actual diff (no undeclared contract change, no declared-but-unchanged
  contract)?
- For every known consumer listed in
  `docs/engineering/repository-map/contracts.md`, is it either assigned a
  tracked downstream task or backed by an explicit, reviewable reason it
  remains compatible?
- Does any downstream repository need to guess an unmerged contract from this
  PR? If so, this is a blocking finding.
- Is the recorded merge/deployment order across affected repositories correct
  and sufficient to prevent a broken intermediate state?
- Does this change affect a generated contract (GraphQL schema, generated
  client/types, exported OpenAPI/export format)? If so, is the generation
  command and resulting diff recorded, and are downstream generated-client
  implications identified?

Findings:

[...]

## 8. Tests and CI

Assess whether tests prove the required behaviour rather than merely execute code.

Missing or weak evidence:

[...]

CI status and relevant checks:

[...]

## 9. Operations

Assess:

- initial post-merge state;
- feature flags;
- monitoring;
- alerting;
- runbooks;
- rollback;
- pilot and observation;
- external side effects.

Findings:

[...]

## 10. Findings

Classify each finding.

### P0 - Critical

[Data loss, security breach, catastrophic side effect, invalid architecture.]

### P1 - Blocking

[Incorrect behaviour, missing acceptance criterion, unsafe migration, authorization defect, race, serious regression.]

### P2 - Non-blocking but required follow-up

[Maintainability, diagnostics, incomplete non-critical tests, operational improvement.]

### P3 - Optional

[Polish or future enhancement.]

For every P0/P1 finding include:

- location;
- evidence;
- impact;
- required correction;
- required test.

## 11. Acceptance criteria matrix

- [ ] Criterion 1 - PASS/FAIL - evidence
- [ ] Criterion 2 - PASS/FAIL - evidence
- [...]

## 12. Final rationale

Explain why the decision is justified.

If `APPROVED`, explicitly state that no unresolved P0 or P1 findings remain.

If `CHANGES REQUIRED`, list the exact blocking corrections.

If `BLOCKED`, identify the missing decision, dependency or unsafe premise.
