# CTRL-FOUNDATION-01 Implementation Report

Report status: COMPLETE FOR INDEPENDENT REVIEW
Repository: `thoth-pub/thoth`
Workflow: STANDARD
Base branch: `develop`
Base commit: `652a499dfdfbaa7594537e0865c41ec617f52dc2`
PR target: `develop`
Programme integration branch: None
Task branch: `feature/ai-delivery-operating-model`
Implementation and remediation head: `104ffaa16b2436cea2b4e2779c241b016916f083`
Pull request: [#764](https://github.com/thoth-pub/thoth/pull/764)
Expected branch deletion after merge: YES
Final programme PR required: NO
Implementing model: GPT-5.6 Thinking control conversation with manual repository application
Reasoning level: high

The final review head is the report commit that contains this file. Git commits
cannot embed their own hash. The independent reviewer must record the exact PR
head and verify that the delta after `104ffaa16b2436cea2b4e2779c241b016916f083` contains only this
report and the associated evidence-status updates.

## 1. Scope confirmation

Approved specification:

`docs/engineering/ai-delivery/tasks/CTRL-FOUNDATION-01.md`

Implemented objective:

Establish the repository-backed AI-led engineering operating model, repository
orientation, local agent instructions, proposed cross-programme decisions,
Publisher Services controls, Thoth Metrics controls, private design-reference
metadata, and review/release gates.

Out-of-scope changes made: NONE

## 2. Commits through the remediation head

- `2bc1c9e3436c87b9969b093b2af72ff2bd7a9d2a` - docs: add AI-led engineering delivery controls
- `ce9d5a1fc22884b6acdcc7ec43d2b5edb38def2a` - Update changelog
- `aba9333e1d52ab62c69c784ffb4856656a3a3de9` - Fix typo
- `1a0cffa5b26668437acce67818031acb1330cb2b` - docs: add repository and environment map
- `7365d67a322c7256b0bd040017442c763b87af5a` - docs: add repository-local agent instructions
- `2ddf5113a72fcb3c200045f444b938bc5f6942f5` - docs: propose shared package and platform decisions
- `deb041e90c50eb19457a5fc4c0d7f0c28bcb2351` - docs: add Publisher Services programme controls
- `bd1046e7183c926c78bfffbd912e12e0902b63a4` - docs: add Thoth Metrics programme controls
- `4b21cd2b580854affcdc3be9c7b53daf448c35e1` - docs: consolidate engineering control foundation
- `fb38b2f5d85247d7d05ddf799dfe5191df2471e4` - docs: remove trailing whitespace from control documents
- `ea687e88330aea3a9e44c69448c20b2d34422b3a` - docs: address foundation review blockers
- `104ffaa16b2436cea2b4e2779c241b016916f083` - docs: correct foundation review evidence

## 3. Actual files changed through the remediation head

```text
.github/workflows/AGENTS.md
AGENTS.md
CHANGELOG.md
docs/engineering/AGENTS.md
docs/engineering/README.md
docs/engineering/agent-instructions/README.md
docs/engineering/agent-instructions/rollout-plan.md
docs/engineering/ai-delivery/README.md
docs/engineering/ai-delivery/branching-and-release-workflow.md
docs/engineering/ai-delivery/decision-record-template.md
docs/engineering/ai-delivery/implementation-report-template.md
docs/engineering/ai-delivery/independent-review-template.md
docs/engineering/ai-delivery/model-selection.md
docs/engineering/ai-delivery/operating-model.md
docs/engineering/ai-delivery/release-gates.md
docs/engineering/ai-delivery/reviews/CTRL-FOUNDATION-01-review-brief.md
docs/engineering/ai-delivery/risk-classification.md
docs/engineering/ai-delivery/task-specification-template.md
docs/engineering/ai-delivery/tasks/CTRL-FOUNDATION-01.md
docs/engineering/decisions/ADR-0001-publisher-package-capability-model.md
docs/engineering/decisions/ADR-0002-platform-domain-boundaries.md
docs/engineering/decisions/README.md
docs/engineering/decisions/decision-register.md
docs/engineering/decisions/package-capability-matrix.md
docs/engineering/design-references.md
docs/engineering/repository-map/README.md
docs/engineering/repository-map/branch-topology.md
docs/engineering/repository-map/control-gaps.md
docs/engineering/repository-map/environments.md
docs/engineering/repository-map/repositories/cc-license.md
docs/engineering/repository-map/repositories/metrics-dashboard.md
docs/engineering/repository-map/repositories/metrics-widget.md
docs/engineering/repository-map/repositories/thoth-app.md
docs/engineering/repository-map/repositories/thoth-dissemination.md
docs/engineering/repository-map/repositories/thoth-sphinx.md
docs/engineering/repository-map/repositories/thoth.md
docs/metrics/README.md
docs/metrics/acceptance-matrix.md
docs/metrics/contract-register.md
docs/metrics/decisions.md
docs/metrics/master-issue.md
docs/metrics/migration-inventory.md
docs/metrics/rollout-plan.md
docs/metrics/source-inventory.md
docs/metrics/task-status.md
docs/publisher-services/README.md
docs/publisher-services/acceptance-matrix.md
docs/publisher-services/decisions.md
docs/publisher-services/master-issue.md
docs/publisher-services/platform-inventory.md
docs/publisher-services/rollout-plan.md
docs/publisher-services/task-status.md
thoth-api-server/AGENTS.md
thoth-api/AGENTS.md
thoth-client/AGENTS.md
thoth-errors/AGENTS.md
thoth-export-server/AGENTS.md
```

Diff summary:

```text
.github/workflows/AGENTS.md                        | 108 +++++++
 AGENTS.md                                          | 333 ++++++++++++++++++++
 CHANGELOG.md                                       |   2 +
 docs/engineering/AGENTS.md                         | 106 +++++++
 docs/engineering/README.md                         |  94 ++++++
 docs/engineering/agent-instructions/README.md      |  61 ++++
 .../engineering/agent-instructions/rollout-plan.md |  67 ++++
 docs/engineering/ai-delivery/README.md             |  61 ++++
 .../ai-delivery/branching-and-release-workflow.md  | 250 +++++++++++++++
 .../ai-delivery/decision-record-template.md        |  96 ++++++
 .../ai-delivery/implementation-report-template.md  | 191 ++++++++++++
 .../ai-delivery/independent-review-template.md     | 180 +++++++++++
 docs/engineering/ai-delivery/model-selection.md    |  83 +++++
 docs/engineering/ai-delivery/operating-model.md    | 248 +++++++++++++++
 docs/engineering/ai-delivery/release-gates.md      | 149 +++++++++
 .../reviews/CTRL-FOUNDATION-01-review-brief.md     | 126 ++++++++
 .../engineering/ai-delivery/risk-classification.md | 112 +++++++
 .../ai-delivery/task-specification-template.md     | 202 ++++++++++++
 .../ai-delivery/tasks/CTRL-FOUNDATION-01.md        | 326 ++++++++++++++++++++
 .../ADR-0001-publisher-package-capability-model.md | 339 +++++++++++++++++++++
 .../ADR-0002-platform-domain-boundaries.md         | 298 ++++++++++++++++++
 docs/engineering/decisions/README.md               |  64 ++++
 docs/engineering/decisions/decision-register.md    |  37 +++
 .../decisions/package-capability-matrix.md         | 157 ++++++++++
 docs/engineering/design-references.md              |  41 +++
 docs/engineering/repository-map/README.md          |  31 ++
 docs/engineering/repository-map/branch-topology.md | 135 ++++++++
 docs/engineering/repository-map/control-gaps.md    |  66 ++++
 docs/engineering/repository-map/environments.md    |  97 ++++++
 .../repository-map/repositories/cc-license.md      |  52 ++++
 .../repositories/metrics-dashboard.md              |  75 +++++
 .../repository-map/repositories/metrics-widget.md  |  64 ++++
 .../repository-map/repositories/thoth-app.md       | 108 +++++++
 .../repositories/thoth-dissemination.md            |  97 ++++++
 .../repository-map/repositories/thoth-sphinx.md    |  92 ++++++
 .../repository-map/repositories/thoth.md           | 152 +++++++++
 docs/metrics/README.md                             | 131 ++++++++
 docs/metrics/acceptance-matrix.md                  |  33 ++
 docs/metrics/contract-register.md                  |  62 ++++
 docs/metrics/decisions.md                          |  97 ++++++
 docs/metrics/master-issue.md                       |  23 ++
 docs/metrics/migration-inventory.md                |  50 +++
 docs/metrics/rollout-plan.md                       |  57 ++++
 docs/metrics/source-inventory.md                   |  79 +++++
 docs/metrics/task-status.md                        |  59 ++++
 docs/publisher-services/README.md                  | 119 ++++++++
 docs/publisher-services/acceptance-matrix.md       | 104 +++++++
 docs/publisher-services/decisions.md               | 148 +++++++++
 docs/publisher-services/master-issue.md            |  21 ++
 docs/publisher-services/platform-inventory.md      | 136 +++++++++
 docs/publisher-services/rollout-plan.md            | 310 +++++++++++++++++++
 docs/publisher-services/task-status.md             |  57 ++++
 thoth-api-server/AGENTS.md                         |  72 +++++
 thoth-api/AGENTS.md                                | 187 ++++++++++++
 thoth-client/AGENTS.md                             |  60 ++++
 thoth-errors/AGENTS.md                             |  39 +++
 thoth-export-server/AGENTS.md                      |  77 +++++
 57 files changed, 6621 insertions(+)
```

All changed files are documentation, `AGENTS.md` instruction files, or the
changelog. No Rust, SQL, migration, package manifest, workflow YAML, deployment
configuration, generated contract, runtime configuration, or secret changed.

## 4. Material effects

- `AGENTS.md` and scoped instruction files define agent permissions and stop
  conditions; they do not change runtime behaviour.
- `docs/engineering/ai-delivery/**` defines task, review, branching, risk and
  release controls.
- `docs/engineering/repository-map/**` records verified repository state and
  explicit gaps.
- `docs/engineering/design-references.md` records private Google Drive file and
  revision metadata without publishing either design body.
- `docs/publisher-services/**` records the direct one-task/one-branch/one-PR
  workflow and programme gates.
- `docs/metrics/**` records repository-local Metrics integration branches after
  readiness and the Metrics programme gates.
- `docs/engineering/decisions/**` proposes ADR-0001 and ADR-0002 without
  approving them.
- `CHANGELOG.md` records PR #764 under Unreleased.

Behavioural effect: documentation and engineering-control evidence only.

## 5. Implementation decisions

1. Keep the private programme specifications in access-controlled Google Drive.
2. Record exact Drive file IDs, revision IDs and modification timestamps in the
   repository.
3. Use direct task branches for Publisher Services.
4. Use repository-local integration branches for Metrics only after readiness.
5. Keep ADR-0001 and ADR-0002 `PROPOSED` pending separate approval.
6. Describe `thoth-sphinx` as placeholder-only: `main` and `develop` contain a
   README, but no workspace, implementation, CI, protections or runtime.
7. Store completed implementation reports under `implementation-reports/`,
   avoiding the repository's broad ignored `reports/` pattern.

Deviation from the approved specification: NONE after review remediation.

## 6. Database, API, authorization and compatibility effects

Migration added: NO
Database or existing-data effect: none
Locking/downtime: none
GraphQL/API changes: none
Generated schema/client changes: none
Authorization paths changed: none
Roles/scopes changed: none
Secrets or personal data added: none
Backwards compatibility: unaffected

## 7. Exact checks and results

### Diff whitespace

Command:

```text
git diff --check 652a499dfdfbaa7594537e0865c41ec617f52dc2...104ffaa16b2436cea2b4e2779c241b016916f083
```

Result: no output; PASS.

### Stale task and placeholder references

Command:

```text
grep -RniE '(^|[^[:alnum:]_-])CTRL-(01|02)([^[:digit:]]|$)|NOT YET CREATE[D]' docs || true
```

Result: no output; PASS.

### Stale control references in repository-map consumers

Command:

```text
grep -RniE 'CG-(05|06|09)'   docs/engineering/repository-map/environments.md   docs/engineering/repository-map/repositories   || true
```

Result: no output; PASS.

The canonical definitions of `CG-05`, `CG-06`, and `CG-09` in
`docs/engineering/repository-map/control-gaps.md` are expected and are not
stale references.

### Canonical Sphinx spelling

Command:

```text
grep -Rni 'sph[y]nx' AGENTS.md .github thoth-* docs || true
```

Result: no output; PASS.

### Sphinx repository-state wording

Command:

```text
grep -RniE 'thoth-sphinx.*empty|Sphinx.*empty|repository empty|empty repository' docs/engineering docs/metrics || true
```

Result: no output; PASS.

### Repository scope

Command:

```text
git diff --name-only 652a499dfdfbaa7594537e0865c41ec617f52dc2...104ffaa16b2436cea2b4e2779c241b016916f083
```

Result: only the files recorded in section 3; PASS.

### Unit and integration tests

No executable behaviour or migration was changed. Existing build, tests, lint,
formatting and migrations were exercised by current-head GitHub Actions.

## 8. CI at the implementation/remediation head

All required workflows completed successfully for `104ffaa16b2436cea2b4e2779c241b016916f083`:

- `build-test-and-check`: `completed/success`, run `30106043628`, https://github.com/thoth-pub/thoth/actions/runs/30106043628
- `check-changelog`: `completed/success`, run `30106043479`, https://github.com/thoth-pub/thoth/actions/runs/30106043479
- `publish-to-dockerhub`: `completed/success`, run `30106043646`, https://github.com/thoth-pub/thoth/actions/runs/30106043646
- `run-migrations`: `completed/success`, run `30106043610`, https://github.com/thoth-pub/thoth/actions/runs/30106043610

The independent reviewer must also verify all required CI on the final
report-containing review head.

## 9. Manual verification

- Both private Google Docs were identified by exact file ID and Drive revision.
- No private design body or export is committed.
- Issues #765 and #766 exist.
- ADR-0001 and ADR-0002 remain `PROPOSED`.
- `thoth-sphinx` `main` and `develop` contain only a placeholder README; no
  workflow, protection or runtime evidence exists.
- Repository-relative control links were checked during the independent review.

## 10. Rollout and rollback

Initial state after merge: the documentation becomes authoritative on
`develop`; programme implementation remains blocked by each recorded gate.

Activation required: no runtime activation.
Feature flag/configuration: none.
Migration sequence: none.
Rollback: revert PR #764.
Monitoring: none for this documentation-only task.

## 11. Known limitations and deferred work

- ADR-0001 and ADR-0002 remain proposed.
- Sphinx normalization and bootstrap are separate tasks.
- Related repositories still require complete `AGENTS.md` rollout.
- Client CI, Thoth Diesel generation and runtime operations remain incomplete or
  unverified as recorded in the control-gap register.
- Metrics source fixtures, COUNTER mappings, service-role codes and complete
  OPERAS inbound discovery remain programme dependencies.
- The private Metrics design retains an obsolete spelling; committed repository
  documents use the canonical `thoth-sphinx` / `Sphinx` terminology.

## 12. Unresolved review gates

- final-head CI after committing this report;
- independent complete-diff review at the exact final head;
- final independent decision `APPROVED`;
- explicit CTO merge approval.

## 13. Agent self-assessment

Suggested review focus:

- report presence and exact evidence;
- final-head CI and head stability;
- private design access and revision metadata;
- Publisher Services versus Metrics branch workflow distinction;
- placeholder-only Sphinx state;
- absence of runtime, migration, workflow or deployment effects.

The implementing conversation does not approve this task.
