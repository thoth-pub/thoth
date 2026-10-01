# CTRL-ADR-AMEND-01 Implementation Report

## 1. Repository state

Owning GitHub issue: `thoth-pub/thoth#964`
Repository: `thoth-pub/thoth`
Workflow: `STANDARD`
Base branch: `develop`
Authorized base commit: `345a7a04131e7c0539f7518c6ea457fa5cb6a462`
Actual base commit: `345a7a04131e7c0539f7518c6ea457fa5cb6a462`
PR target: `develop`
Programme integration branch: `NONE`
Task branch: `feature/engineering-control/adr-preauthority-correction`
Head commit: live GitHub lifecycle evidence under ADR-0005; the final remediation head is the commit containing this report and is not embedded here because doing so would require a self-referential follow-up commit.
Pull request: `#965`; open/draft/merge/review state is live GitHub lifecycle evidence under ADR-0005.
Expected branch deletion after merge: `YES`
Final programme PR required: `NO`
Implementing model: `ChatGPT / GPT-5.6 Sol`
Reasoning level: `maximum practical / high`
Independent implementation reviewer/model: cross-model `REQUIRED`
Independent review reasoning level: `maximum practical / high`
Dependencies for #964 itself: `None`

Fresh remediation preflight immediately before editing confirmed:

- PR #965 was `OPEN / DRAFT / UNMERGED`;
- PR head was exactly `24b6f524cbd0204ffb855920756b7bdabb1a9e72`;
- PR base and live `develop` were exactly `345a7a04131e7c0539f7518c6ea457fa5cb6a462`;
- current report blob was exactly `2da21bc0b3ae42c5e024072dcfb00e774cf99383`;
- the four automatic workflows for the pre-remediation head were complete and successful;
- no source path other than this report is authorized for remediation.

## 2. Scope confirmation

Approved specification:

- original #964 issue-body SHA-256 `4beda48432f64b1d9cf0ad7db32f6059e02b2701e7a9d44bff5a334b5b789f80`;
- Specification Amendment 1 `5939276423`;
- CTO approval `5939297808`;
- independent CRITICAL specification approval `5940159575`;
- original implementation authorization `5940287216`;
- exact-head review/control disposition `5941598377` / PR comment `5941598956`;
- report-only remediation authorization `5941624634`.

Implemented objective:

- retain the already-reviewed CTRL-ADR-AMEND-01 doctrine/changelog implementation unchanged; and
- correct only the implementation-report evidence format so it uses the repository implementation-report template while preserving truthful provenance and ADR-0005 lifecycle boundaries.

Out-of-scope changes made: `NONE`.

Corrected evidence-format deviation: the first implementation report used nine condensed sections and stated `Specification deviations: NONE`. The approved Specification Amendment 1 required the report to use `docs/engineering/ai-delivery/implementation-report-template.md`. This remediation corrects that evidence-format miss. It does not change the implemented doctrine or changelog.

## 3. Commits

- `24b6f524cbd0204ffb855920756b7bdabb1a9e72` - `CTRL-ADR-AMEND-01: correct pre-authority ADR amendment control`
- remediation commit containing this report - exact SHA is live GitHub lifecycle evidence under ADR-0005 after the authorized non-force push; it is not copied into this file by a follow-up metadata-only commit.

## 4. Files changed

Authorized write paths for the original implementation:

- `CHANGELOG.md`
- `docs/engineering/decisions/README.md`

Authorized new-file path for the original implementation:

- `docs/engineering/ai-delivery/implementation-reports/CTRL-ADR-AMEND-01-implementation-report.md`

Authorized remediation path:

- `docs/engineering/ai-delivery/implementation-reports/CTRL-ADR-AMEND-01-implementation-report.md`

Actual remediation file changed:

- `docs/engineering/ai-delivery/implementation-reports/CTRL-ADR-AMEND-01-implementation-report.md`
  - reason: align implementation evidence with the mandatory repository template after control accepted exact-head review finding L-1;
  - behavioural effect: documentation/evidence format only; no doctrine/runtime behaviour change;
  - within authorized remediation write budget: `YES`.

Actual new files created by remediation: `NONE`.

Files deleted, moved or renamed: `NONE`.

### 4.1 Write-budget compliance

`PASS`

The remediation candidate changes exactly one authorized path.

### 4.2 Authorized actions actually used

- repository inspection: `YES`
- source edit: `YES - report path only`
- new file creation: `NO`
- file deletion/move/rename: `NO`
- branch creation: `NO - existing task branch reused`
- commit: `YES - exactly one additional remediation commit authorized`
- push: `YES - non-force push authorized`
- PR creation/update: `YES - existing draft PR #965 updated through the pushed commit only`
- issue/comment mutation: `YES - authorization and remediation/review handoff evidence only`
- manual CI dispatch/rerun: `NO`
- provider/runtime read: `NO`
- provider/runtime write: `NO`
- migration execution: `NO`
- release/tag/publication: `NO`
- merge: `NO`
- deployment: `NO`
- production activation: `NO`
- other: `NO`

Unauthorized actions performed: `NONE`.

### 4.3 Automatic and manual external effects

Automatic CI/provider effects observed for the original implementation head `24b6f524cbd0204ffb855920756b7bdabb1a9e72`:

- `build-test-and-check` run `36925094595`: success; classify success; lint/build/format_check/test skipped;
- `publish-to-dockerhub` run `36925094604`: success; classify success; `build_and_push_staging_docker_image` skipped;
- `check-changelog` run `36925094597`: success; changelog check success;
- `run-migrations` run `36925094601`: success; classify success; `run_migrations` skipped.

Automatic CI for the remediation head is live GitHub lifecycle evidence under ADR-0005 and is recorded in the remediation handoff after the push. It is not copied into this report by a CI-state-only source commit.

Manually initiated external actions: `NONE`.

External writes/publication: `NONE`; the Docker build/push job was skipped on the original implementation head. No migration executed.

## 5. Implementation decisions

Decisions made within the approved design:

1. Keep `APPROVED` as durable exact decision-owner approval while repository authority additionally requires required exact-head review and reachability from `develop`.
2. Preserve ADR-0013 as the only narrower exact-version programme-local pre-`develop` reliance mechanism.
3. Require CTO classification of post-approval changes as `FACTUAL CLARIFICATION` or `MATERIAL ARCHITECTURAL CORRECTION`, with reviewer challenge and fail-closed material treatment when uncertain/unclassified.
4. Key material pre-authority eligibility to ADR-number history, so any `APPROVED` instance ever reachable from `develop` permanently closes the exception for that ADR number.
5. Define an ADR version as the exact Git blob and stale earlier exact-version approval/review/reliance/merge authorization on material decision drift.
6. Use exact-final-blob CTO approval before commit for material corrections, including blob SHA, path, owning issue, carried approval date and inspection basis; prohibit back-dating.
7. Allow alignment of ADR-local clauses only when they merely restate repository amendment doctrine and only for ADRs eligible for the exception.
8. Introduce no fifth ADR status and no runtime authority.
9. For this remediation, preserve `README.md` and `CHANGELOG.md` byte-for-byte and modify only the evidence report.

Deviation from the approved specification requiring correction:

- `CORRECTED`: the original implementation report used a condensed nine-section structure rather than the mandatory implementation-report template. This report-only remediation was explicitly authorized in `5941624634` after control disposition `5941598377`.

No other specification deviation is known.

## 6. Database and migration effects

Migration added: `NO`.

- migration files: `NONE`
- schema effect: `NONE`
- existing-data effect: `NONE`
- locking/downtime: `NONE`
- empty database result: `NOT APPLICABLE - documentation/control only`
- populated database result: `NOT APPLICABLE - documentation/control only`
- rollback/forward repair: `NOT APPLICABLE - no database mutation`
- idempotency: `NOT APPLICABLE`

## 7. API and compatibility effects

GraphQL/API changes: `NONE`.
Generated schema/client updates: `NONE`.
Backwards compatibility: `UNCHANGED`.
Deprecations: `NONE`.
Cross-repository dependencies: `NONE` for #964 implementation; #958/PR #960 is a downstream consumer that remains HOLD until this doctrine is merged and #958 is freshly reconciled/authorized.

## 8. Authorization and security

Authorization paths changed: `NONE` in application/runtime authorization. This task changes engineering decision-control doctrine only.
Roles/scopes involved: `NONE`.
Negative authorization tests: `NOT APPLICABLE - documentation/control only`.
Secret or personal-data handling: `NONE`.
Security limitations: `NONE` identified; this remediation grants no runtime/provider/migration/deployment authority.

## 9. Tests and checks

Record of the original implementation validation is preserved below; remediation validation is run against the report-only candidate before the authorized remediation commit.

### Formatting

Command:

```text
git diff --check HEAD
```

Original implementation result:

```text
exit 0; stdout empty; stderr empty
```

Remediation result: recorded from the report-only validation repository before commit; must be exit 0 with empty stdout/stderr or remediation HOLDS.

### Unit tests

Command:

```text
NOT APPLICABLE
```

Result:

```text
Documentation/control-only change; runtime unit-test jobs were skipped by docs-only CI classification.
```

### Integration/database tests

Command:

```text
NOT APPLICABLE
```

Result:

```text
No schema/data/runtime change; migration workflow classified the change as docs-only and skipped migration execution on the original implementation head.
```

### Lint/static analysis

Command:

```text
NOT APPLICABLE TO RUNTIME SOURCE
```

Result:

```text
The repository docs-only classifier skipped lint/build/format/test jobs on the original implementation head. Documentation whitespace validation is covered by git diff --check.
```

### Other required checks

Original implementation command:

```text
git diff --name-only HEAD
```

Original result:

```text
CHANGELOG.md
docs/engineering/ai-delivery/implementation-reports/CTRL-ADR-AMEND-01-implementation-report.md
docs/engineering/decisions/README.md
```

Original implementation command:

```text
git status --short
```

Original result:

```text
M  CHANGELOG.md
A  docs/engineering/ai-delivery/implementation-reports/CTRL-ADR-AMEND-01-implementation-report.md
M  docs/engineering/decisions/README.md
```

Original provenance: literal local Git validation used an exact README plus a bounded changelog insertion-context copy; the final full changelog was constructed from authoritative GitHub base blob `cf662733b61354f08f7d6b26ead3b8079f10d8ba` and verified through committed blob identity. Independent review reproduced the full tree diff and accepted this provenance as truthful and sufficient.

Remediation validation repository note: the bounded local Git repository validates candidate whitespace, changed-path identity and template structure. Exact predecessor branch bytes and the final base-to-head diff are verified separately against live GitHub before and after the authorized branch update; the local repository is not presented as a full clone of the remote repository.

Remediation required checks before commit:

- `git diff --check HEAD`;
- `git diff --name-only HEAD` must list only this report path;
- `git status --short` must show only this report modified;
- template structure check must confirm numbered sections `1` through `15` are present exactly once;
- no trailing whitespace;
- live GitHub preflight must still bind PR #965 to authorized head/base before branch update.

## 10. Manual verification

Environment: bounded local Git remediation-validation repository plus live GitHub repository/PR inspection.

Steps:

1. Reverified PR #965 open/draft/unmerged at authorized head `24b6f524cbd0204ffb855920756b7bdabb1a9e72` and base `345a7a04131e7c0539f7518c6ea457fa5cb6a462`.
2. Reverified live `develop` remained `345a7a04131e7c0539f7518c6ea457fa5cb6a462`.
3. Reverified current report blob was `2da21bc0b3ae42c5e024072dcfb00e774cf99383`.
4. Compared the candidate report structure with `docs/engineering/ai-delivery/implementation-report-template.md`.
5. Verified the remediation candidate changes only the authorized report path.
6. Preserved README and changelog without edit.

Observed result: preflight passed; report-only remediation is sufficient; no doctrine/runtime correction is required.

Evidence: #964 control disposition `5941598377`, PR #965 disposition `5941598956`, remediation authorization `5941624634`, and live GitHub diff/CI evidence. Final remediation head/CI are recorded in GitHub after push.

## 11. CI

CI status for original implementation head `24b6f524cbd0204ffb855920756b7bdabb1a9e72`: `PASSING`.

Checks:

- `build-test-and-check` `36925094595`: `success`; docs-only classifier succeeded; runtime jobs skipped.
- `publish-to-dockerhub` `36925094604`: `success`; Docker build/push job skipped; no publication.
- `check-changelog` `36925094597`: `success`.
- `run-migrations` `36925094601`: `success`; migration job skipped.

Failures or warnings: `NONE` on the original implementation head.

CI status for the remediation head: live GitHub lifecycle evidence after push. Normal automatic PR-triggered CI is authorized; manual dispatch/rerun is not authorized. The remediation handoff records the resulting run IDs, conclusions and whether any publication-capable job was skipped or executed.

## 12. Rollout and rollback

Initial state after merge: repository engineering-control doctrine only.
Activation required: `NO`.
Feature flag/configuration: `NONE`.
Migration sequence: `NONE`.
Rollback/disable procedure: prospective normal reviewed Git revert of the doctrine/report change. Corrections already completed under the repository-authoritative rule remain valid historical evidence; in-flight corrections HOLD and reconcile against then-authoritative doctrine.
Monitoring required: `NONE` for runtime; normal control observation/reconciliation only.

The remediation itself changes evidence format only and has no separate rollout or activation.

## 13. Known limitations and deferred work

- The original implementation report did not use the mandatory fifteen-section template; this remediation corrects that exact evidence-format miss.
- Final remediation head, PR state and remediation CI are transient GitHub lifecycle evidence under ADR-0005 and are intentionally not copied into this committed report through a follow-up metadata-only commit.
- The original literal local Git validation used a bounded changelog insertion-context copy rather than the full 253 KB changelog; independent exact-head review verified the final full-tree diff and accepted that provenance as truthful and sufficient.
- Changelog traceability finding L-2 (no issue/PR link in the CTRL-ADR-AMEND-01 entry) remains a non-blocking limitation and is not changed because the remediation authorization forbids changelog edits.
- #958 remains HOLD until #964 receives fresh exact-head approval, separate CTO merge authorization, and merges to `develop`, followed by fresh #958 reconciliation and bounded authorization.

## 14. Unresolved issues

`NONE` within the authorized report-only remediation scope.

Remaining lifecycle gates are not implementation issues:

- remediation head automatic CI must settle;
- fresh independent CRITICAL cross-model exact-head review is required;
Remaining lifecycle gates are GitHub-owned under ADR-0005:

- merge requires separate CTO merge authorization bound to the exact reviewed head;
- merge occurs only through the separately authorized GitHub merge gate;
- #958 / PR #960 may resume only after #965 merges to `develop` and fresh #958 reconciliation and bounded authorization complete.

## 15. Agent self-assessment

The implementing agent does not approve its own work.

Suggested review focus:

- confirm the remediation diff changes exactly one path: this implementation report;
- confirm all fifteen numbered template sections are present and the prior condensed-format deviation is explicitly corrected;
- confirm README and CHANGELOG blobs are unchanged from the previously reviewed head;
- confirm the report preserves truthful original validation provenance and does not overstate full-local-worktree evidence;
- confirm transient final head/PR/CI facts are correctly delegated to GitHub lifecycle evidence under ADR-0005 rather than omitted or copied through another source commit;
- confirm the explicit action-use matrix matches authorization `5941624634`;
- confirm normal remediation CI is docs-only and no Docker publication, migration execution or other external write occurred;
- confirm no source/runtime/provider/migration/merge action outside the report-only authorization occurred;
- confirm #958 remains HOLD.
