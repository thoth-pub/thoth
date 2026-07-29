# ADR-0001-POST-MERGE - Reconcile ADR-0001 post-merge control state

Status: APPROVED
Programme: Cross-programme Engineering Control
Affected programmes:

- Publisher Services
- Thoth Metrics
- OAI-PMH

Repository: thoth-pub/thoth
Workflow: STANDARD
Base branch: develop
Exact approved base: `b2c91ff25b95ab0e10a477ff21dbd4702f5db8d4`
PR target: develop
Programme integration branch: None
Risk: LOW
Owner: CTO
Approved by: Javi, CTO
Original approval date: 2026-07-29
Scope-amendment approval date: 2026-07-29
Implementing agent/model: Codex / GPT-5
Implementation reasoning: Medium
Independent reviewer/model: A fresh non-implementing reviewer; concrete identity
must be recorded in the independent review
Target branch: `feature/engineering/adr-0001-post-merge`
Dependencies: PR #772 merged at the exact parent, content and merge commits below;
all task-specific and programme-readiness controls remain independently required

## 1. Objective

Reconcile active repository control documents that still describe ADR-0001
approval PR #772 as pending merge or as an unsatisfied merge dependency. Record
that ADR-0001 is approved and merged while preserving every remaining
implementation, migration, review, rollout and activation control.

This task is documentation-only. It does not authorize Publisher Services,
Metrics, OAI-PMH, package enforcement, migration, deployment, release or
production work.

## 2. Background and authority

Javi, CTO, approved the original bounded specification on 2026-07-29 and approved
the decision-register scope amendment on 2026-07-29.

The authoritative completed event is PR
[#772](https://github.com/thoth-pub/thoth/pull/772):

```text
Authorized base parent:
bafd4cbf752f9d6153036fc7f47115220fed3fbd

Merged content parent:
55d424839b6740559c39c9e518c2b37f41caf6f8

Merge commit:
b2c91ff25b95ab0e10a477ff21dbd4702f5db8d4

Merged at:
2026-07-29T12:11:48Z

Merge method:
merge commit

Commits preserved:
7
```

The resulting decision state is:

```text
ADR-0001: APPROVED AND MERGED
```

The merge satisfies only the shared ADR-0001 approval-and-merge dependency. It
does not make any implementation task ready.

## 3. Explicit scope

The task must:

1. commit this approved specification first and alone;
2. open a draft PR targeting `develop` before changing active documents;
3. reconcile exactly five active control documents;
4. correct the active decision register under the CTO-approved scope amendment;
5. validate every active ADR-0001 and PR #772 reference outside historical task
   specifications and implementation reports;
6. create the implementation report in a separate final report commit;
7. keep the PR draft and unmerged throughout implementation and handoff;
8. wait for and inspect exact-head documentation-only CI;
9. post one immutable top-level exact-head evidence comment; and
10. hand the draft PR to a fresh independent reviewer.

### 3.1 Seven-file cumulative allowlist

The complete pull request may change exactly:

```text
docs/engineering/ai-delivery/tasks/ADR-0001-POST-MERGE.md
docs/engineering/ai-delivery/implementation-reports/ADR-0001-POST-MERGE-implementation-report.md
docs/engineering/README.md
docs/engineering/repository-map/control-gaps.md
docs/engineering/decisions/decision-register.md
docs/publisher-services/README.md
docs/metrics/README.md
```

The cumulative scope is exactly one specification, five active control
documents and one implementation report.

### 3.2 Five-file implementation scope

The implementation commit may change exactly:

```text
docs/engineering/README.md
docs/engineering/repository-map/control-gaps.md
docs/engineering/decisions/decision-register.md
docs/publisher-services/README.md
docs/metrics/README.md
```

### 3.3 Decision-register scope amendment

The decision register is an active post-merge control document. Its stale
Approval sequence currently states:

```text
Merge the independently approved `ADR-0001` approval PR before:

- Publisher Services `BE-01`;
- metrics entitlement implementation;
- protected metrics serving;
- OAI package-gating implementation.
```

That future-tense requirement must be replaced with completed-state wording. The
listed dependent work must remain governed by its own specifications,
dependencies, review, migration, rollout and activation controls.

Only `docs/engineering/decisions/decision-register.md` is authorized within the
decisions directory. The following remain prohibited:

```text
docs/engineering/decisions/ADR-0001-publisher-package-capability-model.md
docs/engineering/decisions/ADR-0002-platform-domain-boundaries.md
docs/engineering/decisions/package-capability-matrix.md
docs/engineering/decisions/README.md
```

ADR-0001's normative content and the package-capability matrix must remain
byte-for-byte unchanged.

## 4. Non-goals and prohibited changes

The task must not:

- modify `CHANGELOG.md`; this is an explicit CTO-approved bounded-scope
  exception for this internal documentation-only reconciliation;
- modify historical task specifications, implementation reports, PR comments or
  review evidence;
- modify `docs/engineering/ai-delivery/tasks/ADR-0001-APPROVAL.md`;
- modify the ADR-0001 approval implementation report;
- modify Publisher Services decisions, task status or rollout plan;
- modify Metrics decisions or task status;
- modify workflows, runtime code, migrations or generated files;
- modify issues #765 or #766;
- modify branch protection or repository rulesets;
- change the approved ADR, capability matrix or architectural semantics;
- start BE-01, Publisher Services ADR-01, Metrics WP1 or any implementation;
- mark the PR ready, approve it, merge it, deploy, release or activate production;
  or
- access production services, data, credentials or secrets.

## 5. Invariants

1. ADR-0001 remains `APPROVED`.
2. Its approved package-capability matrix and semantics are unchanged.
3. Package and distribution-platform configuration remain independent.
4. Publisher Services remains `BLOCKED FOR IMPLEMENTATION`.
5. Metrics remains `BLOCKED FOR IMPLEMENTATION`.
6. `BE-01` remains `BLOCKED` pending its own approved bounded specification.
7. Publisher Services ADR-01 and its final platform inventory remain unresolved.
8. `MET-CTRL-01` remains `CHANGES REQUIRED`.
9. All Metrics work packages remain `BLOCKED`.
10. `CI-DOCS-01` remains `MERGED - OBSERVATION REQUIRED`.
11. No implementation, migration, API, authorization, deployment, release or
    production behaviour changes.
12. Issues #765 and #766 are not modified.
13. Historical evidence is not rewritten to appear current.
14. No task or rollout stage is marked `READY`.
15. OAI package gating remains unauthorized.
16. ADR-0002 and its approval sequence remain unchanged.

## 6. Required active-document state

### 6.1 Engineering README

Record ADR-0001 as `APPROVED AND MERGED`, approved by Javi, CTO, on 2026-07-28.
Record that PR #772 merged on 2026-07-29 as
`b2c91ff25b95ab0e10a477ff21dbd4702f5db8d4`. Remove future merge conditions,
state that the shared ADR dependency is satisfied and preserve all
task-specific and programme-readiness blockers.

### 6.2 Engineering control gaps

Set the evidence date to 2026-07-29. Reconcile CG-06 as fully resolved because
both ADR approval records are merged. Record PR #772's merge commit and state
that dependent work still requires its own bounded specification and remaining
programme controls. Do not close CG-07, CG-08 or any later gap.

### 6.3 Engineering decision register

Set `Last updated` to 2026-07-29. Preserve ADR-0001 as `APPROVED` and record that
the independently reviewed PR #772 merged into `develop` on 2026-07-29 as
`b2c91ff25b95ab0e10a477ff21dbd4702f5db8d4`.

Replace the future approval sequence with completed-state wording. State that
the listed work no longer waits on ADR-0001 approval or merge but remains
subject to its own approved bounded specification, dependencies, review,
migration, rollout and activation controls. Preserve the ADR-0002 sequence and
requirements unchanged.

### 6.4 Publisher Services README

Record ADR-0001 as approved and merged, including PR #772's merge date and
commit. State that the shared capability dependency is satisfied without
implementing package storage, capability enforcement or distribution behaviour.
Preserve ADR-01, final distribution-platform inventory, BE-01 and
branch-readiness blockers and retain `BLOCKED FOR IMPLEMENTATION`.

### 6.5 Metrics README

Record ADR-0001 as approved and merged, including PR #772's merge date and
commit. State that the shared package-capability dependency is satisfied without
activating collection, entitlement enforcement, serving, import or export
behaviour. Preserve `MET-CTRL-01`, Sphinx, Diesel, client branch, service-role,
fixture, COUNTER and OPERAS blockers and retain `BLOCKED FOR IMPLEMENTATION`.

## 7. Commit sequence

The branch must contain exactly these three ordered task commits:

1. `docs: specify ADR-0001 post-merge reconciliation`
   - changes only this specification;
2. `docs: reconcile ADR-0001 merged state`
   - changes exactly the five active control documents; and
3. `docs: report ADR-0001 post-merge reconciliation`
   - changes only the implementation report.

The first commit must be pushed and a draft PR opened before the second commit is
created. Do not amend, squash, rebase or force-push.

## 8. Acceptance criteria

- [ ] The branch originated from exact base
      `b2c91ff25b95ab0e10a477ff21dbd4702f5db8d4`.
- [ ] The specification was committed first and alone.
- [ ] The PR is draft, unmerged and targets `develop`.
- [ ] The implementation commit changes exactly five active documents.
- [ ] The cumulative PR changes exactly seven authorized documentation paths.
- [ ] Every active document describes ADR-0001 as approved and merged.
- [ ] No active document says PR #772 is pending, unmerged or still required to
      merge.
- [ ] CG-06 is fully resolved.
- [ ] Publisher Services and Metrics remain blocked.
- [ ] BE-01 remains blocked pending its own specification.
- [ ] `MET-CTRL-01` remains `CHANGES REQUIRED`.
- [ ] OAI package gating remains unauthorized.
- [ ] No task or rollout stage is marked `READY`.
- [ ] ADR-0001, ADR-0002 and the capability matrix are unchanged.
- [ ] Exact-head CI is terminal and documentation-only.
- [ ] Every skipped heavy job has an empty step array.
- [ ] Immutable finalization evidence is posted.
- [ ] The implementation report is complete under the approved finalization
      mechanism.
- [ ] There are zero unresolved review threads.
- [ ] No runtime or operational action occurred.

## 9. Validation commands

Run:

```bash
rg -n \
  'ADR-0001|PR #772|pull/772|b2c91ff25b95ab0e10a477ff21dbd4702f5db8d4' \
  docs \
  --glob '*.md' \
  --glob '!docs/engineering/ai-delivery/tasks/**' \
  --glob '!docs/engineering/ai-delivery/implementation-reports/**'

rg -n \
  '(ADR-0001|PR #772|pull/772).*(pending merge|pending approval|pending|unmerged|requires? .*merge|must .*merge|after .*merges?)|((pending merge|pending approval|unmerged|requires? .*merge|must .*merge).*(ADR-0001|PR #772|pull/772))' \
  docs \
  --glob '*.md' \
  --glob '!docs/engineering/ai-delivery/tasks/**' \
  --glob '!docs/engineering/ai-delivery/implementation-reports/**'

rg -n \
  'approval record pending merge|pending merge of that independently reviewed approval record|requires PR #772 to merge|PR #772 to merge after fresh independent review' \
  docs \
  --glob '*.md' \
  --glob '!docs/engineering/ai-delivery/tasks/**' \
  --glob '!docs/engineering/ai-delivery/implementation-reports/**'

rg -n \
  'Merge the independently approved `ADR-0001` approval PR before|ADR-0001.*(before|pending|unmerged|requires?.*merge)' \
  docs/engineering/decisions/decision-register.md

rg -n '\bREADY\b' docs/publisher-services docs/metrics

git diff --check \
  b2c91ff25b95ab0e10a477ff21dbd4702f5db8d4...HEAD

git diff --name-only \
  b2c91ff25b95ab0e10a477ff21dbd4702f5db8d4...HEAD

git diff \
  b2c91ff25b95ab0e10a477ff21dbd4702f5db8d4...HEAD \
  -- docs/engineering/decisions/decision-register.md
```

Required stale-state result: zero active-document stale-state matches.

Classify every exhaustive-search result. Historical specifications and reports
may describe earlier pending states and must remain unchanged.

Also verify:

- the specification commit changes only the specification;
- the implementation commit changes exactly five active documents;
- the report commit changes only the implementation report;
- the cumulative changed-file set is exactly the seven-file allowlist;
- the normative ADR-0001 document is byte-for-byte unchanged;
- ADR-0002, the capability matrix, Publisher Services decisions/tracker/rollout
  plan and Metrics decisions/tracker are unchanged;
- no workflow, code, migration, generated file, issue, branch-protection or
  ruleset change occurred; and
- the repository documentation classifier returns exactly:

```json
{"docs_only":"true","run_build":"false","run_docker":"false","run_migrations":"false"}
```

## 10. CI expectations

After the report commit is pushed, exact-head CI must reach:

```text
build-test-and-check:
  classify: success
  build: skipped
  test: skipped
  lint: skipped
  format_check: skipped

run-migrations:
  classify: success
  run_migrations: skipped

publish-to-dockerhub:
  classify: success
  build_and_push_staging_docker_image: skipped

check-changelog:
  check-changelog: success
```

Inspect job payloads. Every skipped heavy job must have an empty step array.
No Rust build, test, lint or format step; migration build, apply or revert step;
or Docker checkout, login, build or push step may execute.

This PR is an additional controlled documentation-only observation for
CI-DOCS-01. CI-DOCS-01 remains `MERGED - OBSERVATION REQUIRED`; the legitimate
mixed-source and remaining next-three-PR observations remain outstanding.

## 11. Implementation report

Create:

```text
docs/engineering/ai-delivery/implementation-reports/ADR-0001-POST-MERGE-implementation-report.md
```

The report must record:

- repository, programme, task ID, risk and approved scope amendment;
- exact base, specification commit and implementation content commit/head;
- complete ordered task commit history available before the report commit;
- exact seven-file cumulative and five-file implementation scopes;
- the additional decision-register reference and corrected excerpt;
- stale-reference commands, results and classifications;
- `READY`, diff and documentation-classifier results;
- preserved blockers and invariants;
- no runtime, migration, API, authorization, workflow, issue, deployment,
  release, production or secret effects;
- rollout, rollback and fresh independent-review requirement; and
- CI-DOCS-01 observation status.

### 11.1 Approved report-finalization mechanism

The report commit cannot embed its own SHA or CI generated after it is pushed.
Therefore:

1. the report records the exact base and exact implementation content head;
2. it records all completed repository validation available before the report
   commit;
3. the report-finalization commit SHA, final PR head, exact-head CI and immutable
   evidence-comment URL are recorded after push in a top-level immutable PR
   evidence comment and in the implementation handoff;
4. no later evidence-only commit may be created merely to put the report
   commit's own SHA or CI inside the report; and
5. the report lists this as an approved task-specific reporting deviation.

## 12. Immutable evidence

Post one top-level immutable PR comment bound to the final exact head. Record:

- exact base;
- specification, implementation content and report-finalization commits;
- final exact PR head and complete three-commit sequence;
- exact seven-file cumulative and five-file implementation scopes;
- exhaustive stale-reference and `READY` results;
- diff and documentation-classifier results;
- exact workflow run IDs and job conclusions;
- empty skipped-job step arrays;
- no-effect assessment;
- draft and unmerged state;
- unresolved-thread count;
- confirmation that no later evidence-only commit will be created;
- the additional decision-register correction and CTO scope amendment; and
- CI-DOCS-01's observation status and outstanding observations.

Do not edit or replace any PR #772 evidence comment.

## 13. Data, API, authorization and operational effects

Migration required: NO

GraphQL/API change: NONE

Authorization change: NONE

Generated output: NONE

Deployment or release effect: NONE

Production or secret access: NONE

This task updates documentation state only.

## 14. Independent review

A fresh reviewer in a non-implementing context must inspect the approved
specification and amendment, exact seven-file diff, five-file implementation
commit, validation results, exact-head CI payloads, immutable evidence and
unresolved review threads.

The implementing agent may not approve its own work. Explicit CTO authorization
is required after independent approval and before the PR may be marked ready or
merged.

## 15. Rollout

Documentation becomes authoritative only when this PR merges into `develop`.
No runtime activation occurs.

## 16. Rollback

Revert this documentation-only PR. No database, API, authorization, deployment,
release, production or data rollback is required.

## 17. Stop conditions

Stop and report `BLOCKED` if:

- any base, branch, PR #772 or repository precondition moves;
- another active document contains a materially stale post-merge requirement
  outside the seven-file allowlist;
- any changed path falls outside the allowlist;
- the specification, implementation or report commit has the wrong per-commit
  file scope;
- ADR-0001, ADR-0002 or the capability matrix changes;
- any dependent implementation is described as ready, authorized or activated;
- a stale-state search returns an unexplained active-document match;
- the documentation classifier differs from the required result;
- a required CI context fails or remains missing;
- a heavy job executes or a skipped heavy job has non-empty steps;
- unresolved review threads exist at handoff;
- production credentials or destructive access would be required; or
- any implementation, migration, API, authorization, workflow, issue,
  deployment, release, production, secret, ruleset or branch-protection effect
  is detected.

If another active out-of-scope reference is found before the first commit,
return:

```text
BLOCKED - ADDITIONAL ACTIVE POST-MERGE REFERENCE
```

If CI classification or a required context fails, return:

```text
BLOCKED - CI CLASSIFICATION OR CONTEXT FAILURE
```

Do not reset, amend, rebase, force-push, mark ready, approve, merge, deploy,
release, activate production or delete the task branch before merge.

## 18. Approval

Approved for implementation by: Javi, CTO

Date: 2026-07-29

Notes: The original specification and the CTO-approved decision-register scope
amendment are incorporated here. Fresh independent review and later explicit CTO
authorization remain mandatory before ready or merge.
