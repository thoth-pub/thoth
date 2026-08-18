# CI-DOCS-01 - Skip Heavy CI for Documentation-Only Changes

Status: APPROVED
Programme: Cross-programme Engineering Control
Repository: `thoth-pub/thoth`
Workflow: STANDARD
Base branch: `develop`
Base commit: `0af9fbae940464a8f94aa1d9a857bad7a55ac54c`
PR target: `develop`
Programme integration branch: None
Risk: HIGH
Owner: Javi, CTO
Approved by: Javi, CTO
Initial approval date: 2026-07-28
Protected-context amendment approval date: 2026-07-28
Dependencies: PR [#770](https://github.com/thoth-pub/thoth/pull/770), merged as `0af9fbae940464a8f94aa1d9a857bad7a55ac54c`; live protected-context inventory; verified paired real/no-action workflow design
Target branch: `feature/engineering/ci-docs-only-gating`
Implementing agent/model: Codex / GPT-5
Implementation reasoning: High
Independent reviewer/model: ChatGPT / GPT-5.6 Thinking
Independent review reasoning: High

## 1. Objective

Prevent documentation-only pull requests from running expensive Rust, migration and Docker work while preserving all six branch-protected required contexts and the additional mandatory Docker workflow context.

A documentation-only change is a non-empty complete changed-file set in which every path is either:

```text
docs/**
CHANGELOG.md
```

Root `README.md`, `.github/**`, mixed changes and empty or indeterminate changed-file sets are not documentation-only.

## 2. Background and authority

Authoritative sources:

1. merged repository workflows and branch protection at the exact base;
2. the CTO-approved CI-DOCS-01 specification supplied on 2026-07-28;
3. the CTO-approved protected-context amendment supplied on 2026-07-28;
4. [`AGENTS.md`](../../../../AGENTS.md) and [`.github/workflows/AGENTS.md`](../../../../.github/workflows/AGENTS.md);
5. the [AI-led Engineering Operating Model](../operating-model.md), [Branching and Release Workflow](../branching-and-release-workflow.md), [Risk Classification](../risk-classification.md), [Release Gates](../release-gates.md) and [Thoth repository map](../../repository-map/repositories/thoth.md).

Observed state at task start:

- `origin/develop` is exactly `0af9fbae940464a8f94aa1d9a857bad7a55ac54c`;
- the worktree is clean and the task branch did not previously exist locally or remotely;
- PR #770 merged as the approved base;
- the main build and migration workflows use positive path filters;
- their paired no-action workflows use complementary `paths-ignore` filters;
- the Docker workflow runs checkout, metadata, QEMU, Buildx, registry login, build and push on every pull request;
- live `develop` branch protection requires exactly six contexts and no ruleset adds another;
- PR #770 produced all seven workflow/job contexts successfully.

## 3. Explicit scope

Permitted workflow changes:

```text
.github/workflows/build_test_and_check.yml
.github/workflows/build_test_and_check_no_action.yml
.github/workflows/run_migrations.yml
.github/workflows/run_migrations_no_action.yml
.github/workflows/docker_build_and_push_to_dockerhub.yml
```

Permitted additional files:

```text
.github/scripts/classify_ci_changes.py
CHANGELOG.md
docs/engineering/ai-delivery/tasks/CI-DOCS-01.md
docs/engineering/ai-delivery/implementation-reports/CI-DOCS-01-implementation-report.md
```

The task must:

1. add one Python-standard-library classifier helper with deterministic self-tests;
2. classify the complete pull-request or pushed range and fail closed on uncertainty;
3. replace top-level path filtering with lightweight classifier jobs and job-level conditions;
4. preserve existing heavy job IDs and names;
5. make manual dispatches run all heavy jobs;
6. delete the two no-action workflows only after the main workflows provide the preserved contexts;
7. keep `check_changelog.yml` active and unchanged;
8. update the changelog and provide a complete implementation report.

A separate committed test file is not in scope.

## 4. Dependencies

- PR #770 merged as `0af9fbae940464a8f94aa1d9a857bad7a55ac54c`.
- `develop` pointed exactly to that commit at task start.
- Live branch protection and rulesets were inventoried.
- The paired real/no-action workflow design was verified.
- The existing Docker workflow and job identity were verified separately.
- Recent PR evidence confirmed all seven workflow/job contexts can be produced.
- No repository-wide branch normalization is required.
- No production environment or secret is required for local validation.

## 5. Non-goals

The task must not:

1. alter Rust application code, migrations or database state;
2. alter Dockerfile contents or image composition;
3. alter release workflows, deployment or production behaviour;
4. change branch protection, repository rulesets, secrets, permissions or protected environments;
5. change security policy;
6. change PR #770 or its evidence;
7. edit issues #765 or #766;
8. optimize unrelated CI;
9. introduce a third-party changed-files action;
10. broaden the approved file allowlist.

## 6. Invariants

Protected required contexts:

```text
build
test
lint
format_check
run_migrations
check-changelog
```

Additional mandatory workflow validation context:

```text
build_and_push_staging_docker_image
```

The implementation must preserve:

```text
Workflow: build-test-and-check
Jobs: build, test, lint, format_check

Workflow: run-migrations
Job: run_migrations

Workflow: publish-to-dockerhub
Job: build_and_push_staging_docker_image

Workflow/check: check-changelog
```

All six protected contexts and the additional Docker context must remain present for every applicable pull request. The implementation must not rely on Docker being branch-protected.

Manual dispatches must default to heavy execution. Classifier failure must block or fail closed and must never silently classify an uncertain change as documentation-only. Skipped jobs must not receive credentials.

Failure handling is explicitly divided into three cases:

```text
Script-level classification error:
emit all-heavy outputs and allow the classifier job to succeed.

Classifier-job or prerequisite failure:
dependent heavy jobs execute conservatively rather than being skipped.

Workflow cancellation:
do not force heavy execution.
```

Classifier-job or prerequisite failure includes checkout failure, inability to start Python, an unexpected exception, or inability to write `GITHUB_OUTPUT`. Every gated heavy-job condition must therefore inspect both the classifier job result and the relevant output, while guarding cancellation.

## 7. Required behaviour

### 7.1 Classification

`docs_only=true` only when the complete changed-file set is non-empty and every path is under `docs/**` or is exactly `CHANGELOG.md`. When true, all heavy outputs are false.

Build/test/lint/format must preserve the current relevant paths:

```text
*.rs anywhere
*.js anywhere
*.json anywhere
*.html anywhere
Cargo.lock
Cargo.toml anywhere
diesel.toml
```

They must also run for changes to the build workflow, deleted build no-action workflow, or classifier.

Migrations must preserve:

```text
*up.sql anywhere
*down.sql anywhere
*db.rs anywhere
src/bin/**
```

They must also run for changes to the migration workflow, deleted migration no-action workflow, or classifier.

Docker must run whenever any changed path is outside `docs/**` and `CHANGELOG.md`.

### 7.2 Events

- Pull requests use the full base-SHA-to-head-SHA diff, never only the latest commit.
- Pushes to `develop` or `master` use the complete `before`-to-`github.sha` range.
- An all-zero or unavailable push `before` SHA fails closed to heavy execution.
- `workflow_dispatch` sets `docs_only=false` and all heavy outputs to true.
- Unknown events and expected script-level `ClassificationError` cases must emit all-heavy outputs with a clear diagnostic and allow the classifier job to succeed.
- If the classifier job or one of its prerequisites fails before valid outputs are available, each dependent heavy job must execute conservatively.
- Workflow cancellation must not force expensive dependent jobs to start.

### 7.3 Workflow behaviour

Each relevant workflow remains triggered and gains a lightweight classifier job. Heavy jobs retain their IDs and names, depend on classification, and use job-level conditions.

For documentation-only pull requests:

- the six protected contexts reach merge-safe terminal results;
- build/test/lint/format heavy steps do not execute;
- migration build/run/revert steps do not execute;
- `check-changelog` executes normally;
- the Docker workflow is triggered but its heavy job is safely skipped;
- checkout, metadata, QEMU, Buildx, registry login, image build and image push do not execute in the Docker job.

For non-documentation or mixed changes, all relevant protected heavy jobs and Docker run normally.

### 7.4 Authorization, concurrency and compatibility

Authorization paths changed: none.

Concurrency and idempotency: the classifier is read-only and deterministic. GitHub Actions is not introduced as durable state.

Compatibility: no API, database, generated contract, application runtime, Docker image composition, release or production behaviour changes.

## 8. Data and migration requirements

Migration required: NO

- schema changes: none;
- existing-data effect: none;
- locking/downtime: none;
- backfill: none;
- empty/populated database tests: not applicable;
- rollback: revert the CI-DOCS-01 merge.

## 9. Observability and operations

The implementation report must keep two separate inventories:

```text
Protected required contexts: 6
Additional mandatory observed workflow contexts: 1
```

The first controlled documentation-only PR after merge must verify safe skipped results and absence of heavy steps. The first mixed documentation/source PR must verify relevant heavy jobs and Docker run. Observe the next three PRs and record anomalies without changing branch protection.

No claim may be made that post-merge skipped-job behaviour has been observed before that controlled observation occurs.

## 10. Acceptance criteria

- [ ] the complete changed-file set is classified deterministically;
- [ ] empty, invalid and uncertain inputs fail closed;
- [ ] classifier-job and prerequisite failures run dependent heavy jobs conservatively;
- [ ] workflow cancellation does not force dependent heavy jobs to start;
- [ ] full PR history remains represented when the latest commit is documentation-only;
- [ ] manual dispatch runs all heavy work;
- [ ] all six protected contexts retain their identities;
- [ ] Docker retains its workflow/job identity as an additional validation context;
- [ ] documentation-only changes skip all heavy Rust, migration and Docker work;
- [ ] non-documentation and mixed changes run all relevant heavy work;
- [ ] no skipped Docker job receives credentials;
- [ ] implementation-workflow changes run all relevant heavy jobs;
- [ ] the two no-action workflows are deleted;
- [ ] `check_changelog.yml` remains unchanged;
- [ ] deterministic classifier tests pass;
- [ ] `actionlint` v1.7.12 produces an accepted baseline-equivalent result with exactly the two approved pre-existing findings and no new findings;
- [ ] `git diff --check` passes;
- [ ] all seven implementation-PR contexts succeed at the final exact head;
- [ ] changelog and implementation report are complete;
- [ ] no out-of-scope file changes occur.

## 11. Required tests

Classifier tests must cover:

1. documentation only;
2. changelog only;
3. mixed docs and Rust;
4. migration only;
5. `Dockerfile`;
6. build workflow change;
7. classifier change;
8. root `README.md`;
9. multiple commits/full PR diff where the first changes Rust and the second only docs;
10. manual dispatch;
11. empty or invalid range.

Workflow validation:

```bash
actionlint
git diff --check
```

The actionlint command must use version 1.7.12 and the approved baseline-equivalence procedure in section 19. Also inspect triggers, branches, workflow/job names, `needs`, job-level conditions, permissions, secrets/credentials, manual dispatch, failure propagation and context preservation. Do not dispatch production or write-capable workflows.

## 12. Rollout

After fresh independent approval and explicit CTO merge authorization:

1. merge to `develop`;
2. verify the merge commit is the live `develop` tip;
3. use one controlled documentation-only PR to validate all protected and Docker skip behaviour;
4. use one mixed documentation/source PR to validate heavy execution;
5. observe the next three PRs and record anomalies.

The merge remains separately gated. No branch-protection change is part of rollout.

## 13. Rollback

Rollback is a normal revert of the CI-DOCS-01 merge. No database, data or production-state rollback is required.

Immediate rollback criteria:

- any protected context is absent or remains pending;
- a protected job is incorrectly skipped for a non-documentation change;
- a documentation-only PR executes a heavy Rust, migration or Docker step;
- Docker login/build/push runs for a documentation-only PR;
- a mixed or non-documentation PR incorrectly skips Docker;
- manual dispatch skips heavy work;
- only the latest commit is classified;
- uncertainty results in skipped work.

## 14. Stop conditions

Stop and return `BLOCKED` if:

- `develop` differs from the exact approved base before branching;
- any of the six protected contexts differs;
- preserving them requires a branch-protection change;
- the Docker workflow or job identity differs materially from the approved premise;
- workflow behaviour cannot be preserved safely;
- implementation needs a third-party changed-files action;
- any file outside the approved allowlist is required;
- testing requires production secrets or a production dispatch;
- classifier tests do not cover the full PR diff;
- `actionlint` v1.7.12 does not reproduce the approved exact-base baseline or the implementation adds, removes or materially changes a normalized finding;
- the implementation PR lacks any of the seven contexts;
- any final exact-head CI job fails;
- a cross-programme architecture decision is discovered.

## 15. Expected implementation report

Use:

`docs/engineering/ai-delivery/implementation-reports/CI-DOCS-01-implementation-report.md`

The report must include the exact repository state, commit sequence, changed files, deleted workflows, classifier rules, workflow changes, the separate protected/additional context inventories, no-effect assessments, exact local test evidence, exact-head CI evidence, rollout, rollback, observation and limitations.

## 16. Recommended execution

Implementation model: Codex / GPT-5
Reasoning level: High
Independent reviewer/model: ChatGPT / GPT-5.6 Thinking
Independent review reasoning level: High

The implementing agent may self-assess risks but may not approve or merge the task.

## 17. Branch and integration plan

- branch source: `develop` at `0af9fbae940464a8f94aa1d9a857bad7a55ac54c`;
- task branch: `feature/engineering/ci-docs-only-gating`;
- pull-request target: `develop`;
- expected merge order: this bounded PR only;
- programme branch refresh: not applicable;
- branch deletion after merge: YES;
- final programme PR required: NO;
- final release path: `develop -> master`.

## 18. Approval

Approved for implementation by: Javi, CTO

Initial approval date: 2026-07-28

Protected-context amendment approved by: Javi, CTO

Amendment date: 2026-07-28

The amendment establishes six protected required contexts and one additional mandatory Docker workflow-validation context. It authorizes implementation, commit, push, draft PR creation and exact-head CI evidence gathering, but not merge, deployment, release, branch-protection changes, issue edits or production activation.

## 19. Approved actionlint baseline amendment

Approved by: Javi, CTO

Approval date: 2026-07-28

Required actionlint version: `1.7.12`

At exact base `0af9fbae940464a8f94aa1d9a857bad7a55ac54c`, repository-wide actionlint validation reports exactly two pre-existing obsolete-action findings:

```text
.github/workflows/docker_build_and_push_to_dockerhub.yml
docker/metadata-action@v4
obsolete-action diagnostic

.github/workflows/docker_build_and_push_to_dockerhub_release.yml
docker/metadata-action@v4
obsolete-action diagnostic
```

The release workflow remains outside CI-DOCS-01 scope. No action upgrade is authorized. In particular, this task must not change:

```text
docker/metadata-action@v4
docker/setup-qemu-action@v3
docker/setup-buildx-action@v3
docker/login-action@v3
docker/build-push-action@v5
```

The repository-wide actionlint requirement is amended for CI-DOCS-01 only. Exit status `1` is acceptable only when the complete diagnostic set contains exactly the two approved findings and no other finding.

### Normalized comparison procedure

1. Run repository-wide `actionlint` v1.7.12 in an immutable temporary worktree at the exact base.
2. Capture the exact command, version, exit status and complete stdout/stderr.
3. Run the identical repository-wide command in the implementation worktree and capture the same evidence.
4. Normalize only line number, column number and any absolute local filesystem prefix.
5. Do not normalize workflow path, action reference, diagnostic category, diagnostic message or diagnostic count.
6. Require the implementation findings to equal the approved two-entry baseline exactly.
7. Run actionlint explicitly against every surviving changed workflow. The build/test and migration workflows must have no finding. The PR Docker workflow may have only its one approved obsolete `docker/metadata-action@v4` finding.
8. Verify deleted no-action workflows through the cumulative diff and classifier tests.
9. Verify the release workflow is byte-for-byte unchanged and all listed action versions are unchanged.

No actionlint suppression comment, global exclusion or configuration may be added.

### Revised actionlint acceptance

The actionlint gate is satisfied only when:

- the exact-base repository-wide run reproduces exactly the two approved findings;
- the implementation repository-wide run produces exactly the same two normalized findings;
- the changed build/test workflow has no finding;
- the changed migration workflow has no finding;
- the changed PR Docker workflow has only its approved baseline finding;
- no diagnostic is added, removed, broadened, suppressed or materially changed;
- the release workflow is byte-for-byte unchanged;
- all listed action versions remain unchanged.

The implementation report must state:

```text
actionlint v1.7.12: accepted baseline-equivalent result;
exit 1 with exactly two approved pre-existing findings and no new findings
```

It must not state that actionlint passed cleanly.

### Revised actionlint stop conditions

Stop and return `BLOCKED` if:

- the exact-base run does not reproduce exactly the two approved findings;
- the implementation adds any finding;
- either approved finding disappears because an action reference or release workflow changed;
- either diagnostic changes materially;
- a third workflow produces a finding;
- the release workflow or an action version changes;
- a suppression or exclusion is introduced;
- validation uses a version other than 1.7.12 without another approved amendment;
- any changed workflow has a non-baseline finding;
- the normalized comparison cannot be made deterministically.

The implementation report must include a section titled `Actionlint baseline and deferred maintenance`, record all base and implementation commands and findings, and recommend a separate bounded task to upgrade both Docker workflows. That separate maintenance task is not part of CI-DOCS-01.

## 20. Approved classifier least-privilege amendment

Automated post-ready review at the previously approved head
`7c2ad5e63cbbcde2789174db727e654a80556c7a` raised a P1 finding because
`persist-credentials: false` prevents checkout credentials from being stored in
Git configuration but does not reduce the permissions of the `GITHUB_TOKEN`
available while the classifier job executes.

Approved by: Javi, CTO

Approval date: 2026-07-28

The original non-goal prohibiting workflow-permission changes is amended only
to authorize the following job-level declaration on the `classify` job in
exactly these workflows:

```text
.github/workflows/build_test_and_check.yml
.github/workflows/run_migrations.yml
.github/workflows/docker_build_and_push_to_dockerhub.yml
```

```yaml
permissions:
  contents: read
```

Each classifier job must have only repository-content read access. No
workflow-level permission declaration, heavy-job permission, release-workflow
permission or write permission may be added or changed. Docker package and
registry access, secrets, credentials, protected environments, action versions,
workflow triggers, job identities, classifier outputs, heavy-job conditions,
release workflows, branch protection and rulesets remain outside this
amendment.

The corrective commit invalidates the previous independent approval and CTO
merge authorization. The PR must be returned to draft before correction and
must remain draft afterward. The new exact head requires successful exact-head
CI, fresh independent review and fresh CTO merge authorization before it can be
marked ready or merged.
