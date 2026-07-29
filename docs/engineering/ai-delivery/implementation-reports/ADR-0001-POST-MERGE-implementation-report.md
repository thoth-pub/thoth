# ADR-0001-POST-MERGE Implementation Report

## 1. Repository state

Repository: `thoth-pub/thoth`
Programme: Cross-programme Engineering Control
Task ID: `ADR-0001-POST-MERGE`
Risk: LOW
Workflow: STANDARD
Base branch: `develop`
Exact base commit: `b2c91ff25b95ab0e10a477ff21dbd4702f5db8d4`
PR target: `develop`
Programme integration branch: None
Task branch: `feature/engineering/adr-0001-post-merge`
Pull request: [#773](https://github.com/thoth-pub/thoth/pull/773) (draft)
Specification commit: `3006abba82aedc296f6c761aaa743395988c5ea6`
Implementation content commit:
`f062e96a463c2a80b9476dd30807c47e2c09dd6b`
Implementation content head:
`f062e96a463c2a80b9476dd30807c47e2c09dd6b`
Previous exact PR head:
`8a51790f6a473432f1c6bb30eb0a72d3f18307a0`
Implementing agent/model: Codex / GPT-5
Implementation reasoning: Medium
Independent reviewer/model: OpenAI ChatGPT / GPT-5.6 Thinking
Independent review comment:
https://github.com/thoth-pub/thoth/pull/773#issuecomment-5118851221
Historical reviewed head:
`8a51790f6a473432f1c6bb30eb0a72d3f18307a0`
Expected branch deletion after merge: YES
Final programme PR required: NO

## 2. Scope confirmation

Approved specification:
[`docs/engineering/ai-delivery/tasks/ADR-0001-POST-MERGE.md`](../tasks/ADR-0001-POST-MERGE.md)

Approval: Javi, CTO, 2026-07-29

Approved scope amendment: Javi, CTO, 2026-07-29. The amendment added the active
decision register after the pre-implementation scan found its stale future-tense
ADR-0001 merge requirement. It established the original seven-file cumulative
and five-file implementation scopes while preserving all normative decision
documents.

Approved post-ready correction: Javi, CTO, 2026-07-29. The correction addresses
only the mandatory changelog entry and concrete independent reviewer/model. It
expands the cumulative allowlist to eight paths without changing the five-file
active-control implementation scope.

Implemented objective: reconcile active repository controls with PR #772's
completed merge while preserving every task-specific and programme-readiness
gate.

Out-of-scope changes made: NONE

### 2.1 Exact eight-file cumulative scope

The final pull request contains exactly one changelog, one specification, five
active control documents and this implementation report:

```text
CHANGELOG.md
docs/engineering/ai-delivery/tasks/ADR-0001-POST-MERGE.md
docs/engineering/ai-delivery/implementation-reports/ADR-0001-POST-MERGE-implementation-report.md
docs/engineering/README.md
docs/engineering/repository-map/control-gaps.md
docs/engineering/decisions/decision-register.md
docs/publisher-services/README.md
docs/metrics/README.md
```

### 2.2 Exact five-file implementation scope

The implementation content commit changes exactly:

```text
docs/engineering/README.md
docs/engineering/repository-map/control-gaps.md
docs/engineering/decisions/decision-register.md
docs/publisher-services/README.md
docs/metrics/README.md
```

### 2.3 Exact three-file corrective scope

The fourth corrective commit changes exactly:

```text
CHANGELOG.md
docs/engineering/ai-delivery/tasks/ADR-0001-POST-MERGE.md
docs/engineering/ai-delivery/implementation-reports/ADR-0001-POST-MERGE-implementation-report.md
```

## 3. Commits

The exact ordered history is:

1. `3006abba82aedc296f6c761aaa743395988c5ea6` -
   `docs: specify ADR-0001 post-merge reconciliation`;
2. `f062e96a463c2a80b9476dd30807c47e2c09dd6b` -
   `docs: reconcile ADR-0001 merged state`;
3. `8a51790f6a473432f1c6bb30eb0a72d3f18307a0` -
   `docs: report ADR-0001 post-merge reconciliation`;
4. corrective commit - `docs: address PR 773 review findings`.

The fourth commit's SHA cannot be embedded in the commit it identifies. Its SHA,
the new final PR head, fresh exact-head CI and the new immutable evidence-comment
URL are recorded after push under the approved finalization mechanism. No later
evidence-only commit will be created.

## 4. Active-document corrections

### 4.1 Engineering README

Corrected state:

> ADR-0001 was approved by Javi, CTO, on 2026-07-28. Its independently reviewed
> approval PR #772 merged on 2026-07-29 as
> `b2c91ff25b95ab0e10a477ff21dbd4702f5db8d4`. The shared ADR
> approval-and-merge dependency is satisfied, but no implementation task is
> ready.

The README also states that PR #772 changed engineering-control documentation
only and implemented or activated no runtime behaviour.

### 4.2 Engineering control gaps

The evidence date is 2026-07-29. CG-06 states:

> Both approval records are merged, so CG-06 is fully resolved and no remaining
> dependency requires PR #772 to merge. Dependent work still requires its own
> approved bounded specification and remaining programme controls.

CG-07, CG-08 and every later control gap remain open and unchanged.

### 4.3 Engineering decision register

This additional active reference was added by the CTO-approved scope amendment.
The stale text was:

> Merge the independently approved `ADR-0001` approval PR before:

The corrected Approval sequence states:

> The ADR-0001 approval-and-merge gate is satisfied. The independently reviewed
> approval PR #772 merged into `develop` on 2026-07-29 as
> `b2c91ff25b95ab0e10a477ff21dbd4702f5db8d4`.
>
> The following work no longer waits on ADR-0001 approval or merge, but remains
> subject to its own approved bounded specification, dependencies, review,
> migration, rollout and activation controls.

ADR-0001 remains `APPROVED`. The ADR-0002 row, approval sequence and requirements
are unchanged.

### 4.4 Publisher Services README

Corrected state:

> ADR-0001 publisher package capabilities is `APPROVED AND MERGED`. The
> independently reviewed approval record merged on 2026-07-29 as
> `b2c91ff25b95ab0e10a477ff21dbd4702f5db8d4`. The shared capability
> decision dependency is satisfied.

Package and distribution-platform independence remains explicit. No package
storage, capability enforcement or distribution behaviour was implemented.

### 4.5 Metrics README

Corrected state:

> ADR-0001 publisher package capabilities is `APPROVED AND MERGED`. The
> independently reviewed approval record merged on 2026-07-29 as
> `b2c91ff25b95ab0e10a477ff21dbd4702f5db8d4`. The shared
> package-capability dependency is satisfied.

No metrics collection, entitlement enforcement, serving, import or export
behaviour was activated.

### 4.6 Post-ready findings and corrective evidence

The automated review submitted at `2026-07-29T14:17:27Z` against historical head
`8a51790f6a473432f1c6bb30eb0a72d3f18307a0` raised:

```text
P1 mandatory changelog entry:
  Thread PRRT_kwDODkn0bc6Uycpt
  Inline comment 3675117373

P1 concrete independent reviewer/model:
  Thread PRRT_kwDODkn0bc6Uycp7
  Inline comment 3675117389
```

The correction adds the mandatory PR #773 entry under the existing Unreleased
`Changed` heading and records OpenAI ChatGPT / GPT-5.6 Thinking as the actual
independent reviewer/model. The historical review evidence is:

```markdown
  - [773](https://github.com/thoth-pub/thoth/pull/773) - Reconcile active engineering, Publisher Services, and Metrics control records with ADR-0001's merged state while preserving all implementation and rollout blockers
```

```text
Comment 5118851221
Reviewed head 8a51790f6a473432f1c6bb30eb0a72d3f18307a0
Review decision APPROVED
```

The independent reviewer/model was not concretely recorded before the active
document changes began. This was a process deviation from `AGENTS.md`.

A fresh non-implementing review was nevertheless completed before merge and
before the first ready transition. The actual reviewer/model and review evidence
are now recorded explicitly.

The previous exact-head approval is historical after this corrective commit.
Fresh independent review is required for the new head. The previous CTO
authorization is likewise historical and fresh explicit CTO authorization is
required after the fresh independent approval.

The earlier no-changelog approach was attempted under CTO authorization but is
superseded by the repository-level requirement and this correction. Historical
evidence is preserved:

```text
Original check-changelog failure:
  run 30454621431, attempt 1, job 90584906474
no changelog label added:
  2026-07-29T13:39:04Z
Failed rerun with the original pre-label event payload:
  run 30454621431, attempt 2, job 90594807733
CTO-authorized close:
  2026-07-29T13:53:05Z
CTO-authorized reopen:
  2026-07-29T13:53:25Z
no changelog label removed:
  2026-07-29T14:24:59Z
```

## 5. Preserved blockers and invariants

Verified preserved state:

```text
ADR-0001: APPROVED
ADR-0001 approval-and-merge gate: SATISFIED
ADR-0001 normative content: UNCHANGED
Package-capability matrix: UNCHANGED
Package and distribution-platform configuration: INDEPENDENT
Publisher Services: BLOCKED FOR IMPLEMENTATION
BE-01: BLOCKED pending its own approved bounded specification
Publisher Services ADR-01: UNRESOLVED
Final distribution-platform inventory: UNRESOLVED
Metrics: BLOCKED FOR IMPLEMENTATION
MET-CTRL-01: CHANGES REQUIRED
All Metrics work packages: BLOCKED
OAI package gating: UNAUTHORIZED
CI-DOCS-01: MERGED - OBSERVATION REQUIRED
Issues #765 and #766: UNCHANGED
No task or rollout stage: READY
```

The legitimate CI-DOCS-01 mixed-source and remaining next-three-PR observations
remain outstanding.

## 6. Validation

All results below were recorded at exact implementation content head
`f062e96a463c2a80b9476dd30807c47e2c09dd6b`.

### 6.1 Exhaustive active-document reference review

Command:

```bash
rg -n \
  'ADR-0001|PR #772|pull/772|b2c91ff25b95ab0e10a477ff21dbd4702f5db8d4' \
  docs \
  --glob '*.md' \
  --glob '!docs/engineering/ai-delivery/tasks/**' \
  --glob '!docs/engineering/ai-delivery/implementation-reports/**'
```

Result:

```text
exit 0
45 matching lines reviewed and classified
```

Classification:

- the five corrected active controls record the approved-and-merged state and
  preserve genuine blockers;
- Publisher Services and Metrics decisions, trackers and rollout controls state
  that ADR-0001 is approved or that its shared dependency is removed, without
  any future merge requirement or readiness claim;
- the normative ADR and package matrix identify approval PR #772 without
  asserting a stale pending state;
- the engineering review brief contains only an ADR document-list reference;
- no active match requires PR #772 to merge or describes it as pending or
  unmerged.

### 6.2 General stale-state search

Command:

```bash
rg -n \
  '(ADR-0001|PR #772|pull/772).*(pending merge|pending approval|pending|unmerged|requires? .*merge|must .*merge|after .*merges?)|((pending merge|pending approval|unmerged|requires? .*merge|must .*merge).*(ADR-0001|PR #772|pull/772))' \
  docs \
  --glob '*.md' \
  --glob '!docs/engineering/ai-delivery/tasks/**' \
  --glob '!docs/engineering/ai-delivery/implementation-reports/**'
```

Result:

```text
exit 1
zero matches
```

### 6.3 Exact known-stale phrases

Command:

```bash
rg -n \
  'approval record pending merge|pending merge of that independently reviewed approval record|requires PR #772 to merge|PR #772 to merge after fresh independent review' \
  docs \
  --glob '*.md' \
  --glob '!docs/engineering/ai-delivery/tasks/**' \
  --glob '!docs/engineering/ai-delivery/implementation-reports/**'
```

Result:

```text
exit 1
zero matches
```

### 6.4 Decision-register future-tense search

Command:

```bash
rg -n \
  'Merge the independently approved `ADR-0001` approval PR before|ADR-0001.*(before|pending|unmerged|requires?.*merge)' \
  docs/engineering/decisions/decision-register.md
```

Result:

```text
exit 1
zero stale future-tense ADR-0001 merge requirements
```

### 6.5 `READY` search

Command:

```bash
rg -n '\bREADY\b' docs/publisher-services docs/metrics
```

Result:

```text
docs/publisher-services/README.md:126:- `READY` - written specification and dependencies are approved.
docs/publisher-services/task-status.md:13:No task moves to `READY` without an approved specification, architecture dependencies, verified repository/base/target, branch-readiness completion or CTO exception, named implementation/review models, tests, migration, rollout and rollback.
```

Both matches are a status definition or a prohibition. Neither marks a task or
rollout stage `READY`.

### 6.6 Diff validation

Command:

```bash
git diff --check \
  b2c91ff25b95ab0e10a477ff21dbd4702f5db8d4...HEAD
```

Result:

```text
exit 0
no output
```

At the implementation content head, the cumulative diff contained exactly the
specification and five active documents. The report-finalization commit added
only the seventh original path. The fourth corrective commit adds `CHANGELOG.md`
and changes exactly the specification, report and changelog, producing eight
unique cumulative paths.

Per-commit checks:

```text
3006abba82aedc296f6c761aaa743395988c5ea6:
  docs/engineering/ai-delivery/tasks/ADR-0001-POST-MERGE.md

f062e96a463c2a80b9476dd30807c47e2c09dd6b:
  docs/engineering/README.md
  docs/engineering/decisions/decision-register.md
  docs/engineering/repository-map/control-gaps.md
  docs/metrics/README.md
  docs/publisher-services/README.md
```

### 6.7 Normative decision integrity

Base and implementation-head Git object IDs:

```text
ADR-0001:
857eaee93ec57f265c279100d2f307f7d6a095b8
857eaee93ec57f265c279100d2f307f7d6a095b8

ADR-0002:
231b545fabcbbdacebe2362c76357cad8ce6d7e4
231b545fabcbbdacebe2362c76357cad8ce6d7e4

package-capability-matrix:
65a3e9279cb9ef785e298c8540a52a2535d8479b
65a3e9279cb9ef785e298c8540a52a2535d8479b
```

Each base/head pair is identical. The normative files remain byte-for-byte
unchanged.

The following also have zero diff from the base:

```text
docs/engineering/decisions/README.md
docs/publisher-services/decisions.md
docs/publisher-services/task-status.md
docs/publisher-services/rollout-plan.md
docs/metrics/decisions.md
docs/metrics/task-status.md
```

### 6.8 Documentation classifier

Command:

```bash
python3 .github/scripts/classify_ci_changes.py --paths \
  docs/engineering/README.md \
  docs/engineering/ai-delivery/tasks/ADR-0001-POST-MERGE.md \
  docs/engineering/decisions/decision-register.md \
  docs/engineering/repository-map/control-gaps.md \
  docs/metrics/README.md \
  docs/publisher-services/README.md
```

Result:

```json
{"docs_only": "true", "run_build": "false", "run_docker": "false", "run_migrations": "false"}
```

The classifier must be rerun over the exact final eight-file range after the
corrective commit.

## 7. Database and migration effects

Migration added: NO

Schema effect: NONE

Existing-data effect: NONE

Locking/downtime: NONE

Data rollback or forward repair: NOT APPLICABLE

## 8. API, compatibility and authorization

GraphQL/API changes: NONE

Generated schema/client updates: NONE

Backwards-compatibility effect: NONE

Authorization paths changed: NONE

Roles/scopes involved: NONE

Secret or personal-data handling: NONE

## 9. Runtime and external effects

No Rust, SQL, migration, GraphQL, generated, workflow, deployment, repository
protection or ruleset file changed.

No issue, deployment, release, production, secret, branch-protection or ruleset
write occurred. Issues #765 and #766 were not modified. No production service,
database, credential or secret was accessed. The authorized PR metadata changes
were returning PR #773 to draft and removing exactly the obsolete `no changelog`
label.

## 10. CI

Status before the corrective push: PENDING

Required exact-head result:

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

Every skipped heavy job must have an empty step array. The corrective commit's
own SHA, new final head, workflow run and job IDs, final conclusions and immutable
evidence-comment URL are recorded externally after push under the approved
finalization mechanism; they are not claimed or embedded in this report.

## 11. Rollout and rollback

Rollout: documentation becomes authoritative when PR #773 merges into `develop`.
No runtime activation occurs.

Rollback: revert this documentation-only PR. No database, API, authorization,
deployment, release, production or data rollback is required.

## 12. Known limitations and deferred work

- Publisher Services ADR-01 and its final distribution-platform inventory remain
  unresolved.
- BE-01 remains blocked pending its own approved bounded specification.
- MET-CTRL-01 remains `CHANGES REQUIRED`.
- Sphinx bootstrap, Diesel generation, branch readiness, service-role, fixture,
  COUNTER and OPERAS completeness controls remain outstanding.
- OAI package gating remains unauthorized.
- CI-DOCS-01 remains `MERGED - OBSERVATION REQUIRED`; the legitimate mixed-source
  and remaining next-three-PR observations remain outstanding.

## 13. Approved task-specific reporting deviation

The corrective commit cannot embed its own SHA or exact-head CI created after
push. Under the CTO-approved finalization mechanism, the new final head, fresh
exact-head CI and final evidence URL are recorded externally in one immutable
top-level PR evidence comment and the implementation handoff. No later
evidence-only commit will be created merely to add the corrective commit's own
SHA or CI to this report.

## 14. Independent review requirement

PR #773 must remain draft and unmerged. A fresh non-implementing reviewer must
review the approved specification and amendments, exact eight-file cumulative
diff, exact five-file implementation commit, exact three-file corrective commit,
validation evidence, exact-head CI payloads and unresolved-thread state.

The implementing agent does not approve this task. Explicit CTO authorization is
required after independent approval and before the PR may be marked ready or
merged.

## 15. Agent self-assessment

Suggested review focus:

- verify the five reconciled active-document excerpts against PR #772's exact
  merge state;
- verify the decision-register amendment did not alter ADR-0002;
- verify all Publisher Services, Metrics and OAI-PMH blockers remain effective;
- verify the four-commit, eight-file cumulative and three-file corrective
  boundaries;
- verify exact-head skipped-job payloads contain empty step arrays; and
- verify the immutable evidence is bound to the final exact head.
