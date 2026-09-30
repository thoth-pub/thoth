# THOTH-ASYNC-01-CTRL-01 Implementation Report

## 1. Repository state

Owning GitHub issue: #961 `THOTH-ASYNC-01-CTRL-01`
Parent programme: #957 `THOTH-ASYNC-01`
Repository: `thoth-pub/thoth`
Workflow: STANDARD
Risk: CRITICAL
Base branch: `develop`
Authorized base commit: `923545d5c9028bc04c40e38efeb7de674efed3fd`
Actual base commit: `923545d5c9028bc04c40e38efeb7de674efed3fd`
PR target: `develop`
Programme integration branch: not created by this task
Task branch: `feature/engineering-control/adr-0013-programme-integration-authority`
Head commit: the exact report-containing candidate SHA is GitHub lifecycle
evidence recorded on #961 at implementation handoff; this report deliberately
does not attempt to embed its own containing Git commit SHA.
Pull request: NOT CREATED - explicitly outside the authorized action set
Expected branch deletion after merge: YES
Final programme PR required: NO for this control task; adopting programmes retain
their own final integration PR
Implementing model: ChatGPT / GPT-5.6 Sol

## 2. Scope confirmation

Approved specification: #961, authorized by the CTO on 2026-09-30.

Implemented objective:

- propose ADR-0013 establishing a tightly fenced programme-integration reliance
  mechanism;
- preserve repository authority on `develop`;
- update shared branching and operating doctrine conditionally on ADR-0013
  becoming repository-authoritative;
- update the decision register and changelog;
- do not modify ADR-0012.

Out-of-scope changes made: NONE.

## 3. Candidate commit

The candidate is intentionally produced as one bounded six-path commit from the
authorized base.

The exact candidate SHA is recorded in the #961 implementation-completion /
review-handoff comment because a Git commit cannot contain its own SHA without a
self-referential commit cycle.

## 4. Files changed

Authorized write paths:

- `docs/engineering/decisions/decision-register.md`
- `docs/engineering/ai-delivery/branching-and-release-workflow.md`
- `docs/engineering/ai-delivery/operating-model.md`
- `CHANGELOG.md`

Authorized new-file paths:

- `docs/engineering/decisions/ADR-0013-programme-integration-decision-reliance.md`
- `docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-CTRL-01-implementation-report.md`

Actual material changes:

- `docs/engineering/decisions/ADR-0013-programme-integration-decision-reliance.md`
  - reason: records the proposed shared control decision;
  - effect: governance only, unavailable until separately approved/reviewed and
    repository-authoritative;
  - within authorized write budget: YES.
- `docs/engineering/decisions/decision-register.md`
  - reason: registers ADR-0013 as PROPOSED and records the conditional exception;
  - effect: default develop-based reliance rule remains active while ADR-0013 is
    proposed;
  - within authorized write budget: YES.
- `docs/engineering/ai-delivery/branching-and-release-workflow.md`
  - reason: defines branch-side containment of programme-integration reliance;
  - effect: conditional doctrine only; no branch created;
  - within authorized write budget: YES.
- `docs/engineering/ai-delivery/operating-model.md`
  - reason: defines Gate 1 evidence required for tasks relying on non-repository-
    authoritative decisions under ADR-0013;
  - effect: fail-closed control only;
  - within authorized write budget: YES.
- `CHANGELOG.md`
  - reason: records the proposed control decision under Unreleased;
  - effect: documentation only;
  - within authorized write budget: YES.
- this implementation report
  - reason: review handoff and write/action evidence;
  - effect: documentation only;
  - within authorized write budget: YES.

Files deleted, moved or renamed: NONE.

### 4.1 Write-budget compliance

PASS - candidate contains exactly six authorized paths.

### 4.2 Authorized actions actually used

- repository inspection: YES
- source edit: YES
- new file creation: YES - exactly two authorized new files
- file deletion/move/rename: NO
- branch creation: YES
- commit: YES
- push: YES
- PR creation/update: NO
- issue/comment mutation: YES - #961 creation, one #957 programme comment and
  the authorized #961 completion/review handoff
- manual CI dispatch/rerun: NO
- provider/runtime read: NO
- provider/runtime write: NO
- migration execution: NO
- release/tag/publication: NO
- merge: NO
- deployment: NO
- production activation: NO

Unauthorized actions performed: NONE.

### 4.3 Automatic and manual external effects

Automatic CI/provider effects observed: no PR was opened, so no PR-triggered CI
was intentionally started by this task.

Manually initiated external actions: NONE.

External writes/publication: NONE beyond the authorized Git branch push and
GitHub issue/comment ledger mutations.

## 5. Implementation decisions

1. ADR-0012 is not modified. ADR-0013 defines a general shared-control mechanism
   instead of embedding a THOTH-ASYNC-specific exception in the async
   architecture ADR.
2. Repository authority remains tied to the normal development-branch authority
   condition.
3. Programme-integration reliance is explicitly narrower, opt-in,
   exact-version-bound and non-retroactive.
4. The exception is unavailable while ADR-0013 is PROPOSED.
5. An ADR being APPROVED or merely present on a programme branch is
   insufficient; the exact approved/reviewed version must be merged into the
   designated integration branch and the programme must explicitly opt in.
6. Existing programmes remain on their current authority model unless they
   separately adopt ADR-0013 after it becomes repository-authoritative.
7. Cross-repository consumers require their own task and pinned preview or
   exact dependency evidence; programme-local reliance never crosses repository
   boundaries implicitly.
8. No production or provider authority follows from programme-local reliance.

## 6. Cross-repository impact

Affected shared contract: engineering delivery/control doctrine only.

Owning repository: `thoth-pub/thoth` for this shared control record.

Known consumers:

- `thoth-pub/thoth`: first intended future adopter through THOTH-ASYNC-01;
- `thoth-pub/thoth-dissemination`, `thoth-pub/thoth-app`,
  `thoth-pub/infrastructure`: no change required by this task; future
  programme dependencies require repository-local tasks and pinned dependency
  evidence;
- other repositories adopting shared engineering controls: remain compatible
  because the mechanism is explicit opt-in and non-retroactive.

No runtime, database, GraphQL/API, generated-client, event-payload,
configuration or deployment contract changes.

## 7. Migration, authorization and operational effects

Database migration effect: NONE.

Data effect: NONE.

Application authorization effect: NONE.

GitHub/provider/IAM/runtime effect: branch creation and commit/push only; no
settings, ruleset, CI, provider or IAM mutation.

Deployment/release effect: NONE.

Production activation effect: NONE.

## 8. Validation

Required documentation validation:

- candidate path set equals the six authorized paths;
- no ADR-0012 path changed;
- no trailing whitespace or conflict markers;
- every candidate text file ends with a newline;
- ADR-0013 status is PROPOSED;
- decision register states the exception is unavailable while ADR-0013 is
  PROPOSED;
- branching and operating doctrine both preserve repository authority;
- changelog entry is under `## [Unreleased]` / `### Added`;
- `git diff --check` against the authorized base must pass after push.

Exact validation commands/results are recorded in the #961 completion/review
handoff together with the final candidate SHA.

## 9. Rollout

No control becomes active merely because the candidate exists.

Required next gates:

1. independent CRITICAL exact-head review;
2. explicit CTO decision approval;
3. controlled approval-state reconciliation as required;
4. fresh exact-head review of any reconciliation commit;
5. separately authorized PR/CI/merge lifecycle;
6. only after ADR-0013 becomes repository-authoritative may any programme opt in.

## 10. Rollback

Before merge: abandon or replace the task branch under explicit control.

After repository-authoritative adoption: material rollback requires a
superseding ADR. A programme may separately stop using programme-integration
reliance by placing its programme state on HOLD and reconciling dependent work.

No branch history, task evidence or review history is rewritten as rollback.

## 11. Known limitations and deferred work

- ADR-0013 is only PROPOSED in this candidate.
- THOTH-ASYNC-01 is not opted in by this task.
- `feature/worker` is not created.
- PR #960 is not retargeted.
- ADR-0012 remains unchanged and non-repository-authoritative.
- Shared async implementation remains HOLD.
- No downstream repository adopts the mechanism.

## 12. Self-assessment

Candidate is ready for independent CRITICAL exact-head review if the final
six-path diff and `git diff --check` verification pass.

The implementing agent does not approve its own work.
