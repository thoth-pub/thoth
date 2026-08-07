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
Reviewed commit:
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
