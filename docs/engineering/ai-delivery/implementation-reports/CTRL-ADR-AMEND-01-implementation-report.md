# CTRL-ADR-AMEND-01 Implementation Report

## 1. Repository state

- Owning issue: `thoth-pub/thoth#964`
- Repository: `thoth-pub/thoth`
- Workflow: `STANDARD`
- Base / actual base: `develop @ 345a7a04131e7c0539f7518c6ea457fa5cb6a462`
- PR target: `develop`
- Task branch: `feature/engineering-control/adr-preauthority-correction`
- Expected branch deletion after merge: `YES`
- Implementing model: `ChatGPT / GPT-5.6 Sol`
- Reasoning: `maximum practical / high`
- Independent implementation review: `cross-model REQUIRED / maximum practical / high`
- Dependencies for #964 itself: `None`
- Head commit / PR: live GitHub lifecycle evidence; not embedded because this report is part of the single authorized implementation commit.

## 2. Approved specification and scope

Bound to:

- original #964 issue-body SHA-256 `4beda48432f64b1d9cf0ad7db32f6059e02b2701e7a9d44bff5a334b5b789f80`;
- Specification Amendment 1 `5939276423`;
- CTO amendment approval `5939297808`;
- independent CRITICAL specification approval `5940159575`;
- fresh implementation authorization `5940287216`.

Objective implemented: correct the ADR authority/amendment process so `APPROVED` decision state is distinct from repository authority, and define the fail-closed pre-authority material-correction mechanism.

Out-of-scope changes: `NONE`.

## 3. Files and write budget

Authorized existing files:

- `docs/engineering/decisions/README.md`
- `CHANGELOG.md`

Authorized new file:

- `docs/engineering/ai-delivery/implementation-reports/CTRL-ADR-AMEND-01-implementation-report.md`

Actual candidate: exactly those three paths. Files deleted/moved/renamed: `NONE`.

Write-budget compliance: `PASS`.

## 4. Actions and external effects

Used: repository/GitHub inspection, bounded documentation edits, authorized report creation, local validation, one implementation commit, non-force branch update, draft PR creation, and issue/comment evidence updates.

Not used / prohibited: file deletion/move/rename, fourth source path, ADR source edits, decision-register/operating-model/branching/workflow edits, manual CI dispatch/rerun, provider/runtime access or writes, migration execution, merge, release/publication, deployment, production activation, #958 source mutation.

The task branch pre-existed at the exact base and was reused; no branch creation occurred under this implementation authorization.

Normal automatic PR-triggered CI may run. Transient head/PR/CI state is GitHub lifecycle evidence under ADR-0005 and is not copied into a follow-up source commit. No manual external action is authorized.

## 5. Implemented decision-process behaviour

1. `APPROVED` records exact decision-owner approval; repository authority additionally requires required independent exact-head review and reachability from `develop`.
2. Normal reliance requires repository authority, except the explicit exact-version ADR-0013 programme-local mechanism.
3. Post-approval changes are CTO-classified as `FACTUAL CLARIFICATION` or `MATERIAL ARCHITECTURAL CORRECTION`; reviewer challenge blocks merge and uncertainty is material.
4. A factual clarification identifies the exact prior approved blob and leaves the selected architecture decision unchanged; source-head review and any exact-version pin reconciliation still apply.
5. The material pre-authority exception is keyed to ADR number/history. Any `APPROVED` instance ever reachable from `develop` makes that ADR number ineligible thereafter, regardless of later revert/removal/supersession or historical review-evidence completeness.
6. An ADR version is the exact Git blob. Material decision drift stales earlier blob/head-bound approval, review, programme-local reliance and merge authorization.
7. Material correction uses exact-final-blob CTO approval before commit. The approval evidence identifies blob SHA, path, owning issue, approval date carried in the blob and inspection basis; no back-dating is allowed.
8. ADR-local clauses that merely restate repository amendment doctrine may be aligned only for an ADR eligible for the pre-authority exception; substantive ADR-specific architecture remains binding.
9. No fifth ADR status is introduced.

Specification deviations: `NONE`.

## 6. Runtime, data, API and security effects

- Migration/schema/data effect: `NONE`.
- GraphQL/API/generated-contract effect: `NONE`.
- Application authorization/roles/scopes effect: `NONE`.
- Cross-repository implementation dependency: `NONE`.
- Secrets/personal data: `NONE`.
- Runtime/provider/deployment/production effect: `NONE`.

## 7. Validation

Literal Git checks were run in a bounded three-path validation repository. The README base/candidate bytes are exact. The changelog validation copy contains the exact insertion anchor and exact new entry rather than the full 253 KB file; separately, the full authoritative changelog candidate is constructed from GitHub `develop` blob `cf662733b61354f08f7d6b26ead3b8079f10d8ba`, and the full README candidate from base blob `fe4983447db96bd929a935a79dd014ea51a5aa66`. Final full-file Git blobs are verified before branch update.

Command:

```text
git diff --check HEAD
```

Result:

```text
exit 0; stdout empty; stderr empty
```

Command:

```text
git diff --name-only HEAD
```

Result:

```text
CHANGELOG.md
docs/engineering/ai-delivery/implementation-reports/CTRL-ADR-AMEND-01-implementation-report.md
docs/engineering/decisions/README.md
```

Command:

```text
git status --short
```

Result:

```text
M  CHANGELOG.md
A  docs/engineering/ai-delivery/implementation-reports/CTRL-ADR-AMEND-01-implementation-report.md
M  docs/engineering/decisions/README.md
```

README checks confirm the four-status vocabulary only, the factual/material classification rule, fail-closed ADR-number historical test, ADR-0013 programme-branch exclusion, and exact-blob command/evidence requirements.

Runtime/unit/database/migration/provider tests: `NOT APPLICABLE - documentation/control only`.
Production rehearsal/backup/pilot/kill switch/change window: `NOT APPLICABLE - no runtime/data/deployment effect`.

## 8. Rollout and rollback

After merge: repository engineering-control doctrine only; no activation, flag, migration or deployment.

Rollback is prospective through normal reviewed Git controls. Corrections already completed under the repository-authoritative rule remain valid historical evidence; in-flight corrections HOLD and reconcile against then-authoritative doctrine. Historical approval/review evidence is never rewritten.

## 9. Limitations and remaining gates

- The literal local Git validation uses a bounded changelog validation copy; final full-file blob construction is separately bound to the authoritative GitHub base blob and verified before commit.
- Exact implementation head, draft PR and CI outcomes remain live GitHub evidence rather than self-referential source metadata.
- #958 remains HOLD until #964 is independently reviewed at its exact implementation head, separately CTO merge-authorized, merged to `develop`, and #958 is freshly reconciled/authorized.

Unresolved implementation issues: `NONE`.

The implementing agent does not approve its own work.
