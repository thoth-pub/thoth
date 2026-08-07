# AI-led Engineering Operating Model

Status: Proposed until merged and approved
Owner: CTO

## 1. Purpose

This operating model allows AI agents to perform architecture support, implementation, testing and review while preserving human accountability, independent approval and production safety.

## 2. Roles

### 2.1 CTO

The CTO owns:

- programme priority;
- product and business decisions;
- architecture approval;
- risk acceptance;
- exceptions to this process;
- merge approval for high-risk work;
- production activation approval.

The CTO is not expected to reproduce every test or manually inspect every line. The CTO must ensure that sufficient independent evidence exists.

### 2.2 Control conversation or coordinating architect

The control role:

- identifies programme, repository, task ID, base branch, dependencies and risk;
- verifies an approved written specification exists;
- scopes one bounded task;
- recommends implementation model and reasoning level;
- reviews actual diffs, tests, CI and operational effects;
- consolidates independent review;
- returns `APPROVED`, `CHANGES REQUIRED` or `BLOCKED`;
- escalates architecture conflicts to the CTO.

The control role may not treat conversational memory as more authoritative than repository evidence.

### 2.3 Programme conversation

A programme conversation:

- maintains programme-local context;
- decomposes the approved design into bounded tasks;
- prepares task specifications and prompts;
- analyses implementation reports;
- identifies programme dependencies and risks;
- updates the programme status recommendation.

It may not settle a cross-programme architectural conflict independently.

### 2.4 Implementing agent

An implementing agent may:

- inspect repositories;
- create a task branch;
- edit code and documentation;
- add or update tests;
- run local checks;
- commit changes;
- push a branch;
- open or update a draft pull request.

An implementing agent may not:

- merge its own pull request;
- deploy or activate production behaviour;
- access production secrets;
- perform destructive production operations;
- silently change approved architecture;
- approve its own work;
- broaden task scope without an approved specification change.

### 2.5 Independent reviewer

The independent reviewer:

- reviews the approved task specification;
- inspects the actual diff;
- inspects test and CI evidence;
- assesses migrations, authorization, compatibility and operations;
- identifies missing acceptance evidence;
- classifies findings by severity;
- returns `APPROVED`, `CHANGES REQUIRED` or `BLOCKED`.

The reviewer must not be the same agent instance that implemented the task.

## 3. Source of truth

The authority order is:

1. Merged code and migrations.
2. Approved ADRs and technical designs.
3. Approved task specifications.
4. GitHub issues, PRs, review threads and CI.
5. Programme status and rollout documents.
6. Agent reports and conversations.

When authoritative sources conflict, implementation stops until the conflict is resolved and recorded.

### 3.1 Lifecycle evidence

For task lifecycle evidence specifically - what was reviewed, by whom, what was
authorized, and whether it merged - `ADR-0005` refines the order above:

1. committed specifications and ADRs define what is authorized and required;
2. the exact PR diff and head define what is proposed for merge;
3. GitHub review records define independent review decisions;
4. GitHub CTO authorization records define merge authorization;
5. CI records define automated validation evidence;
6. the GitHub merge event and merge commit define whether the task merged.

Do not duplicate a live GitHub lifecycle event into a new repository commit
merely so that a Markdown file repeats it. Repository documents may reference
the PR; they need not copy every review identifier or merge timestamp.

## 4. Mandatory task and branch boundary

Use one bounded task per slice branch and pull request.

Follow `branching-and-release-workflow.md`.

Normal tasks branch from `develop` and target `develop`.

For an approved large programme, create `feature/<programme>` from `develop`; each bounded slice branches from and targets that programme integration branch. The final approved programme pull request targets `develop`.

A bounded task has:

- one principal objective;
- explicit scope;
- explicit non-goals;
- identified dependencies;
- acceptance criteria that can be independently verified;
- a defined merge and release effect.

A programme integration branch is permitted only under the controlled large-programme workflow. It does not replace bounded slice branches, task specifications or independent review.

Prefer additive, backwards-compatible and initially inactive changes that can merge directly into `develop` safely. Use an integration branch only when the programme genuinely requires integrated validation before entering `develop`.

## 5. Task lifecycle

### Gate 0 - Design ready

Implementation may be scoped only when:

- required product decisions are settled;
- architecture is approved or the task is explicitly discovery-only;
- dependencies are known;
- blocking unknowns are resolved or declared stop conditions.

### Gate 1 - Specification approved

The task specification must be committed or attached to an authoritative GitHub issue.

The specification must use `task-specification-template.md` or contain equivalent information.

### Gate 2 - Implementation

The implementing agent:

1. confirms repository, base branch and base commit;
2. creates a fresh task branch from the approved base (`develop` or the programme integration branch);
3. inspects relevant code before editing;
4. implements only the approved scope;
5. adds required tests;
6. runs the specified checks;
7. opens or updates a draft PR;
8. completes the implementation report.

### Gate 3 - Evidence complete

Implementation cannot enter review until:

- the PR is available;
- the actual diff is reviewable;
- required tests have exact results;
- migration effects are documented;
- deviations and limitations are explicit;
- CI evidence is available or its absence is explained.

### Gate 4 - Independent review

The independent reviewer applies `independent-review-template.md`.

Narrative claims do not replace inspection of code, migrations and test evidence.

Review is bound to an exact PR head. If the reviewed head changes for any
repository commit, substantive or not, the previous exact-head review does not
carry forward and fresh review is required.

Do not create a commit solely to copy a review decision or approval identifier
into the repository. That moves the head and invalidates the very review it
records. PR body edits and GitHub comments do not change the Git commit and may
record such metadata without invalidating the reviewed head.

### Gate 5 - Remediation

Accepted findings are returned to the original implementing branch as a targeted remediation task.

The implementing agent must:

- address findings;
- rerun relevant tests;
- update the implementation report;
- avoid unrelated refactoring.

The reviewer verifies the changed diff and may raise new findings.

### Gate 6 - Merge ready

See `release-gates.md`.

### Gate 7 - Production ready

See `release-gates.md`.

### Gate 8 - Closed

A task closes only after:

- required observation is complete;
- unexpected behaviour is reconciled;
- cleanup or follow-up tasks are recorded;
- programme status is updated.

### 5.1 Terminal merge evidence

`ADR-0005` governs how a task's completion is evidenced.

The successful GitHub merge event and the resulting merge commit are terminal
evidence that a task merged. No new commit or PR is required solely to record
review identifiers, approval identifiers, merge-authorization identifiers, the
fact that a PR merged, the merge SHA, the merged timestamp, or a transition from
"pending merge" to "merged" or from "implementation authorized" to
"implementation complete", when GitHub already holds those facts.

**Approval-state-only commits are prohibited** when their sole purpose is copying
existing GitHub review or approval metadata into repository files.

An optional evidence comment may be posted on the already merged PR. It is
evidence only: it needs no branch, commit, PR, further review or further merge
authorization.

A post-merge task and PR is required only for a material repository change - for
example a materially incorrect committed tracker, a substantive ADR
contradiction, a migration or operational correction, a defect found in runtime
verification, authorization or security behaviour differing from the approved
design, a substantive documentation error, or a state that could not reasonably
have been represented before merge.

This changes only how lifecycle evidence is recorded. It does not change what
must be reviewed, who may approve or merge, or any production control. Merge is
not production activation.

### 5.2 Normal lifecycle

Where no additional merge authorization is required by specification:

```text
approved specification
-> explicit implementation authorization
-> implementation
-> CI / validation
-> independent exact-head review
-> guarded merge
-> DONE
```

Where CTO merge authorization is explicitly required, and always for HIGH and
CRITICAL risk:

```text
approved specification
-> explicit implementation authorization
-> implementation
-> CI / validation / migration / security / operational evidence
-> independent exact-head review
-> explicit CTO merge authorization
-> guarded merge
-> DONE
```

Production activation, deployment, migration execution and release may still
require a separate authorization or event. That separate production gate does
not require a new PR merely to document that the implementation PR merged.

## 6. Review decisions

Use exactly one decision:

### APPROVED

All blocking requirements and evidence are satisfied. No unresolved P0 or P1 findings remain.

### CHANGES REQUIRED

The task is viable but has unresolved blocking defects, missing evidence or incomplete acceptance criteria.

### BLOCKED

Implementation cannot safely continue because of an unresolved design decision, dependency, repository conflict, missing environment, unsafe approach or invalid premise.

## 7. Parallel work

Parallel tasks are permitted only when:

- their write surfaces do not conflict; or
- a stable shared contract already exists;
- dependencies are explicit;
- merge order is known;
- each task has an independent branch and PR.

Do not run competing agents on alternative implementations of the same approved task unless the task is explicitly a time-boxed design spike.

## 8. Production safety defaults

Prefer:

- additive schema;
- inactive code paths;
- feature flags;
- comparison or shadow mode;
- bounded pilots;
- explicit service roles;
- idempotent migrations;
- fail-closed behaviour;
- monitoring before activation;
- documented rollback;
- observation before cleanup.

High-risk production activation requires explicit CTO approval.
