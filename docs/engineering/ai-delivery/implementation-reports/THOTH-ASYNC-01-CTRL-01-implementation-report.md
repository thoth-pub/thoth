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
Head commit: live exact-head evidence is GitHub-owned and is intentionally not
embedded in this committed report.
Pull request: live PR identity, review, authorization, CI and merge state is
GitHub-owned and is intentionally not duplicated here.
Expected branch deletion after merge: YES
Final programme PR required: NO for this control task; adopting programmes retain
their own final integration PR
Implementing model: ChatGPT / GPT-5.6 Sol

Durable decision state produced by this task:

```text
ADR-0013 status: APPROVED
Approved by: Javi, CTO
Approval date: 2026-09-30
```

`APPROVED` does not by itself make ADR-0013 repository-authoritative.
Repository authority remains governed by ADR-0013's authority condition,
including independent exact-head review of the approval-state content and
reachability from `develop`. Live satisfaction of that condition is GitHub-owned
lifecycle evidence rather than committed status prose.

## 2. Scope confirmation

Approved specification: #961, authorized by the CTO on 2026-09-30.

Implemented objective:

- author ADR-0013 establishing a tightly fenced programme-integration reliance
  mechanism;
- preserve repository authority on `develop`;
- update shared branching and operating doctrine conditionally on ADR-0013
  becoming repository-authoritative;
- update the decision register and changelog;
- reconcile the ADR's durable CTO-owned decision state to `APPROVED`;
- do not modify ADR-0012.

Out-of-scope changes made: NONE.

## 3. Candidate history

The initial authoring candidate was produced within the bounded six-path task
footprint from the authorized base. Subsequent durable decision-state
reconciliation and implementation-report remediation remain within that same
cumulative six-path footprint.

Exact commit SHAs, live PR state, independent-review decisions, merge
authorization and merge evidence are GitHub lifecycle records. They are not
copied into this report so that the committed report remains truthful before
and after review or merge.

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
  - reason: records the shared control decision and its durable CTO-owned
    approval state;
  - effect: governance only; programme-integration reliance remains subject to
    ADR-0013's repository-authority and explicit programme-opt-in conditions;
  - within authorized write budget: YES.
- `docs/engineering/decisions/decision-register.md`
  - reason: registers ADR-0013 and the controlled programme-integration reliance
    exception;
  - effect: the default `develop`-based reliance rule remains in force unless
    ADR-0013's eligibility conditions are fully satisfied;
  - within authorized write budget: YES.
- `docs/engineering/ai-delivery/branching-and-release-workflow.md`
  - reason: defines branch-side containment of programme-integration reliance;
  - effect: conditional doctrine only; no programme branch is created by this
    task;
  - within authorized write budget: YES.
- `docs/engineering/ai-delivery/operating-model.md`
  - reason: defines Gate 1 evidence required for tasks relying on a
    non-repository-authoritative decision under ADR-0013;
  - effect: fail-closed control only;
  - within authorized write budget: YES.
- `CHANGELOG.md`
  - reason: records the approved shared control decision under Unreleased;
  - effect: documentation only;
  - within authorized write budget: YES.
- this implementation report
  - reason: durable implementation, scope and effects evidence;
  - effect: documentation only;
  - within authorized write budget: YES.

Files deleted, moved or renamed: NONE.

### 4.1 Write-budget compliance

PASS - the cumulative task candidate remains limited to the six authorized
paths. Evidence: the unrestricted `git diff --name-only HEAD` command and result
against the authorized base, recorded in section 8.1.

### 4.2 Authorized actions actually used

This section records durable action categories used by the task. Exact timing,
PR state and authorization identifiers are GitHub-owned lifecycle evidence.

- repository inspection: YES
- source edit: YES
- new file creation: YES - exactly two authorized new files
- file deletion/move/rename: NO
- branch creation: YES
- commit: YES
- push: YES
- PR creation/update: YES - PR lifecycle only; no source authority is inferred
  from PR state
- issue/comment mutation: YES - task/programme ledger evidence
- manual CI dispatch/rerun: NO
- provider/runtime read: NO
- provider/runtime write: NO
- migration execution: NO
- release/tag/publication: NO
- merge: live GitHub lifecycle evidence; not duplicated here
- deployment: NO
- production activation: NO

Unauthorized actions performed: NONE.

### 4.3 Automatic and manual external effects

Normal PR-triggered CI is an expected GitHub lifecycle side effect. Exact
workflow runs and results are GitHub-owned evidence and are not duplicated here.

The task authorizes no migration execution, container publication,
provider/IAM/runtime mutation, deployment, release or production activation.

Manually initiated external actions: NONE.

External writes/publication beyond the authorized Git/GitHub control-plane
actions: NONE.

## 5. Implementation decisions

1. ADR-0012 is not modified. ADR-0013 defines a general shared-control mechanism
   instead of embedding a THOTH-ASYNC-specific exception in the async
   architecture ADR.
2. Repository authority remains tied to the normal development-branch authority
   condition.
3. Programme-integration reliance is explicitly narrower, opt-in,
   exact-version-bound and non-retroactive.
4. Programme-integration reliance is unavailable until ADR-0013 itself is
   `APPROVED`, independently reviewed and repository-authoritative, and the
   programme has separately opted in.
5. An ADR being `APPROVED` or merely present on a programme branch is
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
  `thoth-pub/infrastructure`: no source or runtime change is required by this
  task; any future programme dependency requires a repository-local task and
  pinned dependency evidence;
- other repositories adopting shared engineering controls remain compatible
  because the mechanism is explicit opt-in and non-retroactive.

No runtime, database, GraphQL/API, generated-client, event-payload,
configuration or deployment contract changes.

## 7. Migration, authorization and operational effects

Database migration effect: NONE.

Data effect: NONE.

Application authorization effect: NONE.

GitHub/provider/IAM/runtime effect: task branch, Git commit/push, GitHub
issue/comment ledger and PR lifecycle only; no repository settings, ruleset,
manual CI, provider or IAM mutation.

Deployment/release effect: NONE.

Production activation effect: NONE.

## 8. Validation

Durable documentation validation for this task requires:

- cumulative candidate path set equals the six authorized paths;
- no ADR-0012 path changes;
- no trailing whitespace or conflict markers;
- every candidate text file ends with a newline;
- ADR-0013 records durable `APPROVED` decision state with approver and approval
  date;
- the decision register preserves the default `develop` authority boundary and
  conditions the exception on ADR-0013 being approved, independently reviewed
  and repository-authoritative;
- branching and operating doctrine preserve repository authority and explicit
  programme opt-in;
- the changelog records the approved control decision under
  `## [Unreleased]` / `### Added`;
- `git diff --check` passes for the complete cumulative six-path candidate
  against the authorized base;
- the final report correction, compared with the preceding source head, changes
  exactly this implementation report;
- internal Markdown links and repository-relative paths resolve, repository
  names and terminology are correct, and no stale duplicate doctrine
  contradicts the decision.

### 8.1 Exact local validation results

Validation workspace: a Git worktree of this repository, detached at the
authorized base commit `923545d5c9028bc04c40e38efeb7de674efed3fd`, whose working
tree and index were replaced by the complete final six-path candidate:

- the five other task files at their exact candidate bytes, checked out from the
  preceding source head `bf6b65bd66a351918c0dc6f0ee12f2a88080f880`;
- the exact final bytes of this implementation report.

All six candidate paths were staged, so comparisons with `HEAD` include the two
new files. Before the workspace was relied on, the complete Git trees of the
authorized base (`dd040332bf199e6440da9731fbfc4431af3bdcec`) and of the
preceding source head (`1922c62200f2c453d75aa69e1dd61a757a51757a`) were
confirmed identical to the trees GitHub reports for those two commits. The
source commit containing this report records exactly the validated candidate
tree.

No command below uses a restricting pathspec.

Whole-candidate whitespace and conflict-marker check against the authorized
base:

```text
git diff --check HEAD
```

Result:

```text
exit: 0
stdout: <empty>
stderr: <empty>
```

Whole-candidate path set against the authorized base:

```text
git diff --name-only HEAD
```

Result:

```text
exit: 0
stdout:
CHANGELOG.md
docs/engineering/ai-delivery/branching-and-release-workflow.md
docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-CTRL-01-implementation-report.md
docs/engineering/ai-delivery/operating-model.md
docs/engineering/decisions/ADR-0013-programme-integration-decision-reliance.md
docs/engineering/decisions/decision-register.md
stderr: <empty>
```

Report-correction boundary against the preceding source head:

```text
git diff --name-only bf6b65bd66a351918c0dc6f0ee12f2a88080f880
```

Result:

```text
exit: 0
stdout:
docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-CTRL-01-implementation-report.md
stderr: <empty>
```

Documentation checks on the lines the complete candidate adds relative to the
authorized base:

- Internal links and paths: both relative Markdown links (in `CHANGELOG.md` and
  the decision register, each to ADR-0013) resolve from their containing files;
  the only other links are the GitHub issue links for #957 and #961; all six
  backticked repository-relative paths resolve in the candidate tree.
  Unresolved: none.
- Repository names and terminology: the repository names used are
  `thoth-pub/thoth`, `thoth-pub/thoth-dissemination`, `thoth-pub/thoth-app`
  and `thoth-pub/infrastructure`; the first three have repository-map entries
  and `thoth-pub/infrastructure` matches its existing usage in the repository
  documentation. The ADR identifiers used resolve to decision records in the
  candidate, except ADR-0012, which is referenced by identifier only and is not
  a path in the authorized base or the candidate.
- Stale duplicate doctrine: the general decision-reliance rule is stated in the
  decision register (default rule plus the ADR-0013 exception), in
  `branching-and-release-workflow.md` section 3.1 and in `operating-model.md`
  section 3.3; all three condition the exception on ADR-0013 being `APPROVED`,
  independently reviewed and repository-authoritative and on explicit programme
  opt-in. The reliance clauses in ADR-0006 to ADR-0011 each restrict reliance on
  their own decision only, and the status vocabulary in
  `docs/engineering/decisions/README.md` is unchanged. No conflicting duplicate
  was found.
- Changelog: `CHANGELOG.md` gains exactly one `THOTH-ASYNC-01-CTRL-01` entry,
  under the existing `## [Unreleased]` / `### Added` heading, and the
  Unreleased section contains no duplicate headings.

Path-set, whitespace and documentation validation depend only on the authorized
base and the candidate bytes, so they are recorded here. Only the resulting
source-head SHA and the final-head CI results inherently become available after
the report-containing commit is created and pushed; both are GitHub-owned
lifecycle evidence (section 8.2).

### 8.2 Test applicability and CI

Unit tests: NOT APPLICABLE - documentation/control-only task; no executable
product code changed.

Integration/database tests: NOT APPLICABLE - no schema, migration, SQL or
database behaviour changed.

Runtime tests: NOT APPLICABLE - no runtime, provider, deployment or production
behaviour changed.

Manual CI dispatch/rerun: NO.

CI status: **NOT AVAILABLE at report-authoring/commit time**.

Reason: final PR-head CI cannot exist until this report-containing commit is
created and pushed. Exact final-head CI status and check results are therefore
GitHub-owned post-push evidence. This statement records the report-authoring
boundary and remains true regardless of the later live CI outcome.

## 9. Rollout

The durable control sequence is:

1. ADR-0013 carries CTO-owned `APPROVED` decision state.
2. The exact approval-state content receives independent exact-head review.
3. The exact approved/reviewed content becomes reachable from `develop` before
   ADR-0013 is repository-authoritative.
4. A programme that wants programme-integration reliance separately opts in
   through its programme control record.
5. The exact approved/reviewed relied-upon ADR version is merged into the
   designated programme integration branch.
6. Each dependent task is separately specified, authorized and reviewed.
7. Final programme integration into `develop` retains its own integrated review
   and merge gates.

The live position within this sequence is GitHub-owned lifecycle evidence and
is intentionally not duplicated here.

## 10. Rollback

Before repository authority is established through `develop`, the task branch
or PR may be abandoned or replaced under explicit control.

After repository-authoritative adoption, material rollback of ADR-0013 requires
a superseding ADR. A programme may separately stop using programme-integration
reliance by placing its programme state on HOLD and reconciling dependent work.

No branch history, task evidence or review history is rewritten as rollback.

## 11. Known limitations and deferred work

- This task does not itself opt THOTH-ASYNC-01 into ADR-0013.
- This task does not create `feature/worker`.
- This task does not retarget PR #960.
- This task does not change ADR-0012's decision content or authority condition.
- This task does not authorize shared async implementation.
- This task does not cause any downstream repository to adopt the mechanism.
- Migration execution, provider/IAM/runtime action, deployment, release and
  production activation remain separate gates.

## 12. Self-assessment

Any source head containing this report requires the independent review mandated
by the governing controls before merge. Live review and merge-readiness state is
GitHub-owned and is intentionally not duplicated here.

The implementing agent does not approve or merge its own work.
