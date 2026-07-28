# CI-DOCS-01 Implementation Report

## 1. Repository state

Repository: `thoth-pub/thoth`

Workflow: STANDARD

Base branch: `develop`

Base commit: `0af9fbae940464a8f94aa1d9a857bad7a55ac54c`

PR target: `develop`

Programme integration branch: None

Task branch: `feature/engineering/ci-docs-only-gating`

Implementation head before the evidence commit: `b6668d2183c2931d62ae56fd968407804252ac5d`

Final exact head: recorded externally in draft PR [#771](https://github.com/thoth-pub/thoth/pull/771) and the implementation handoff after the evidence commit is pushed

Pull request: [#771](https://github.com/thoth-pub/thoth/pull/771) (draft)

Expected branch deletion after merge: YES

Final programme PR required: NO

Implementing model: Codex / GPT-5

Reasoning level: High

Independent reviewer/model: ChatGPT / GPT-5.6 Thinking

Independent review reasoning: High

### Final-head evidence model

A commit cannot contain its own Git SHA or the IDs and conclusions of CI runs that are created only after that commit is pushed. This report therefore records the complete local evidence and the pre-evidence implementation head. The immutable final evidence commit, exact final head, run IDs and conclusions are recorded in PR #771 and the implementation handoff after fresh CI completes on that head. No later repository commit may be treated as covered by earlier CI.

## 2. Scope confirmation

Approved specification: [`docs/engineering/ai-delivery/tasks/CI-DOCS-01.md`](../tasks/CI-DOCS-01.md)

Implemented objective: classify complete pull-request and push change ranges so documentation-only changes safely skip heavy Rust, migration and Docker jobs while preserving six protected contexts and one additional mandatory Docker workflow context.

Out-of-scope changes made: NONE

## 3. Commits

- `4f08af91c92b41efae180abaea5acf274acd6213` - `docs: specify CI docs-only gating`
- `6b4ae12f132412de5c394bbdb57451bd96f0c6f9` - `docs: approve CI actionlint baseline`
- `b6668d2183c2931d62ae56fd968407804252ac5d` - `ci: gate heavy jobs for docs-only changes`
- final evidence commit - `docs: record CI docs-only gating evidence`; exact SHA recorded externally after creation

The specification-first commit was not amended, rebased or rewritten.

## 4. Files changed

- `.github/scripts/classify_ci_changes.py`
  - reason: repository-owned complete-range classifier and deterministic self-test interface;
  - behavioural effect: emits `docs_only`, `run_build`, `run_migrations` and `run_docker` outputs; fails closed to heavy execution.
- `.github/workflows/build_test_and_check.yml`
  - reason: replace top-level positive path filters with classification and job-level gating;
  - behavioural effect: preserves `build`, `test`, `lint` and `format_check` job identities while safely skipping them for documentation-only changes.
- `.github/workflows/build_test_and_check_no_action.yml`
  - reason: deleted after the main workflow became responsible for all protected contexts;
  - behavioural effect: removes the paired no-action workflow without removing any protected job identity.
- `.github/workflows/run_migrations.yml`
  - reason: replace top-level positive path filters with classification and job-level gating;
  - behavioural effect: preserves `run_migrations` while safely skipping its build/run/revert steps for documentation-only changes.
- `.github/workflows/run_migrations_no_action.yml`
  - reason: deleted after the main workflow became responsible for the protected migration context;
  - behavioural effect: removes the paired no-action workflow without removing the protected job identity.
- `.github/workflows/docker_build_and_push_to_dockerhub.yml`
  - reason: add classification and job-level Docker gating;
  - behavioural effect: documentation-only changes skip the complete heavy Docker job before checkout, metadata, QEMU, Buildx, login, build or push; non-documentation changes retain the existing job.
- `CHANGELOG.md`
  - reason: required PR entry under `## [Unreleased]` / `### Changed`;
  - behavioural effect: none.
- `docs/engineering/ai-delivery/tasks/CI-DOCS-01.md`
  - reason: commit the approved task, protected-context amendment and actionlint baseline amendment;
  - behavioural effect: none.
- `docs/engineering/ai-delivery/implementation-reports/CI-DOCS-01-implementation-report.md`
  - reason: record implementation and verification evidence;
  - behavioural effect: none.

Deleted no-action workflows:

```text
.github/workflows/build_test_and_check_no_action.yml
.github/workflows/run_migrations_no_action.yml
```

Unchanged control surfaces:

```text
.github/workflows/check_changelog.yml
.github/workflows/docker_build_and_push_to_dockerhub_release.yml
branch protection and rulesets
Docker action versions
```

## 5. Implementation decisions

1. Pull requests use `base...head`, which asks Git for the complete merge-base-to-head PR change set and therefore retains earlier source commits when the latest commit changes only documentation.
2. Pushes use the complete `before..head` range.
3. `workflow_dispatch` returns all-heavy outputs without inferring from an empty range.
4. All-zero, missing, malformed, unsupported, empty or unresolvable inputs fail closed to all-heavy outputs with a diagnostic.
5. `git diff --name-only --no-renames -z` preserves deleted control paths and treats renames conservatively.
6. Classifier checkouts use full history and do not persist checkout credentials.
7. Heavy jobs retain their existing IDs and names and are gated at job level through `needs: classify`.
8. The Docker action sequence and versions are unchanged.

Deviation from the approved specification: NONE

## 6. Classifier rules

Documentation-only is true only for a non-empty complete set where every path is either under `docs/` or exactly `CHANGELOG.md`.

Build/test/lint/format run for:

```text
*.rs
*.js
*.json
*.html
Cargo.lock
*Cargo.toml
diesel.toml
.github/workflows/build_test_and_check.yml
.github/workflows/build_test_and_check_no_action.yml
.github/scripts/classify_ci_changes.py
```

Migrations run for:

```text
*up.sql
*down.sql
*db.rs
src/bin/**
.github/workflows/run_migrations.yml
.github/workflows/run_migrations_no_action.yml
.github/scripts/classify_ci_changes.py
```

Docker runs whenever the complete change set is not documentation-only.

## 7. Context inventory

Protected required contexts: 6

```text
build
test
lint
format_check
run_migrations
check-changelog
```

Additional mandatory observed workflow contexts: 1

```text
build_and_push_staging_docker_image
```

Docker is not described or relied upon as a branch-protected required status check.

Workflow and job invariants:

```text
build-test-and-check: build, test, lint, format_check
run-migrations: run_migrations
publish-to-dockerhub: build_and_push_staging_docker_image
check-changelog: check-changelog
```

## 8. Database, runtime and compatibility effects

Migration added: NO

Database/schema/data effect: none

Locking/downtime: none

Runtime/application effect: none

GraphQL/API changes: none

Generated schema/client updates: none

Dockerfile or image-composition changes: none

Release/deployment/production effect: none

Backwards compatibility: existing heavy workflow/job identities and heavy step bodies are preserved.

Cross-repository dependencies: none

## 9. Authorization and security

Authorization paths changed: none

Roles/scopes involved: none

Negative authorization tests: not applicable

Secrets or personal-data handling: no secrets or personal data were accessed or printed.

Permissions: no workflow permission configuration changed.

Credential assessment:

- classifier checkout steps set `persist-credentials: false`;
- a job skipped by its job-level condition receives no runner and does not receive heavy-job credentials;
- the existing Docker login remains inside the gated heavy Docker job;
- no secret or credential reference was added or changed.

Security limitations: the existing Docker action versions are intentionally unchanged and include the obsolete `docker/metadata-action@v4` baseline described below.

## 10. Tests and checks

### Classifier self-tests

Command:

```text
PYTHONDONTWRITEBYTECODE=1 python3 .github/scripts/classify_ci_changes.py --self-test
```

Result: exit 0; 14 cases passed.

Required output matrix:

| Complete changed-file set or event | `docs_only` | `run_build` | `run_migrations` | `run_docker` |
| --- | --- | --- | --- | --- |
| `docs/engineering/example.md`, `docs/publisher-services/README.md` | true | false | false | false |
| `CHANGELOG.md` | true | false | false | false |
| `docs/example.md`, `thoth-api/src/lib.rs` | false | true | false | true |
| `thoth-api/migrations/example/up.sql` | false | false | true | true |
| `Dockerfile` | false | false | false | true |
| `.github/workflows/build_test_and_check.yml` | false | true | false | true |
| `.github/scripts/classify_ci_changes.py` | false | true | true | true |
| `README.md` | false | false | false | true |
| full PR diff: first commit Rust, second commit docs | false | true | false | true |
| `workflow_dispatch` | false | true | true | true |
| empty range, fail-closed result | false | true | true | true |
| all-zero invalid range, fail-closed result | false | true | true | true |

Additional deterministic cases:

| Complete changed-file set | `docs_only` | `run_build` | `run_migrations` | `run_docker` |
| --- | --- | --- | --- | --- |
| deleted `.github/workflows/build_test_and_check_no_action.yml` | false | true | false | true |
| deleted `.github/workflows/run_migrations_no_action.yml` | false | false | true | true |

The full-PR-diff test created a temporary Git repository with a base commit, a Rust commit and a later documentation commit. `base...head` returned both `docs/example.md` and `thoth-api/src/lib.rs`; classification remained heavy.

### Python compile check

Command:

```text
PYTHONPYCACHEPREFIX=/tmp/ci-docs-01-validation.V2kM9B/pycache python3 -m py_compile .github/scripts/classify_ci_changes.py
```

Result: exit 0; no output.

### Whitespace

Command:

```text
git diff --check 0af9fbae940464a8f94aa1d9a857bad7a55ac54c...HEAD
```

Result before evidence commit: exit 0; no output.

The same command must be rerun after the evidence commit.

### Rust, database and integration checks

No Rust, database, migration, API or runtime file changed. Local workspace Rust/database checks were not required by the approved CI-DOCS-01 specification. Because the implementation changes `.github/**`, the final exact-head PR must run all relevant Rust, migration and Docker jobs.

## 11. Actionlint baseline and deferred maintenance

Exact base: `0af9fbae940464a8f94aa1d9a857bad7a55ac54c`

Version command:

```text
/tmp/ci-docs-01-validation.V2kM9B/actionlint -version
```

Version result:

```text
1.7.12
installed by downloading from release page
built with go1.26.1 compiler for darwin/arm64
```

The exact base was exported with:

```text
git archive 0af9fbae940464a8f94aa1d9a857bad7a55ac54c -o /tmp/ci-docs-01-validation.V2kM9B/base.tar
tar -xf /tmp/ci-docs-01-validation.V2kM9B/base.tar -C /tmp/ci-docs-01-validation.V2kM9B/base
git init -q /tmp/ci-docs-01-validation.V2kM9B/base
```

The archive contents are immutable exact-base files; `git init` added only the repository metadata that actionlint requires to locate `.github/workflows`.

Comparison harness:

```text
python3 /tmp/ci-docs-01-validation.V2kM9B/compare_actionlint.py
```

The uncommitted temporary harness ran the identical repository-wide binary command in the exact-base and implementation directories, captured stdout/stderr and exit status, normalized only the filesystem prefix plus line and column, and compared path, complete message, category and count exactly.

Exact-base repository-wide command:

```text
(cd /tmp/ci-docs-01-validation.V2kM9B/base && /tmp/ci-docs-01-validation.V2kM9B/actionlint)
```

Exit status: `1`

Complete base findings:

```text
.github/workflows/docker_build_and_push_to_dockerhub.yml:18:15: the runner of "docker/metadata-action@v4" action is too old to run on GitHub Actions. update the action's version to fix this issue [action]
   |
18 |         uses: docker/metadata-action@v4
   |               ^~~~~~~~~~~~~~~~~~~~~~~~~
.github/workflows/docker_build_and_push_to_dockerhub_release.yml:18:15: the runner of "docker/metadata-action@v4" action is too old to run on GitHub Actions. update the action's version to fix this issue [action]
   |
18 |         uses: docker/metadata-action@v4
   |               ^~~~~~~~~~~~~~~~~~~~~~~~~
```

Implementation repository-wide command:

```text
(cd /Users/ja573/thoth && /tmp/ci-docs-01-validation.V2kM9B/actionlint)
```

Exit status: `1`

Complete implementation findings:

```text
.github/workflows/docker_build_and_push_to_dockerhub.yml:32:15: the runner of "docker/metadata-action@v4" action is too old to run on GitHub Actions. update the action's version to fix this issue [action]
   |
32 |         uses: docker/metadata-action@v4
   |               ^~~~~~~~~~~~~~~~~~~~~~~~~
.github/workflows/docker_build_and_push_to_dockerhub_release.yml:18:15: the runner of "docker/metadata-action@v4" action is too old to run on GitHub Actions. update the action's version to fix this issue [action]
   |
18 |         uses: docker/metadata-action@v4
   |               ^~~~~~~~~~~~~~~~~~~~~~~~~
```

Changed-workflow commands and results:

```text
/tmp/ci-docs-01-validation.V2kM9B/actionlint .github/workflows/build_test_and_check.yml
exit 0; no findings

/tmp/ci-docs-01-validation.V2kM9B/actionlint .github/workflows/run_migrations.yml
exit 0; no findings

/tmp/ci-docs-01-validation.V2kM9B/actionlint .github/workflows/docker_build_and_push_to_dockerhub.yml
exit 1; exactly the approved docker/metadata-action@v4 finding shown above
```

Comparison result:

```text
PASS: normalized findings are exactly baseline-equivalent
```

Required report statement:

```text
actionlint v1.7.12: accepted baseline-equivalent result;
exit 1 with exactly two approved pre-existing findings and no new findings
```

No suppression, ignore, exclusion or actionlint configuration was added.

Both Docker workflows retain `docker/metadata-action@v4`. The release workflow is byte-for-byte unchanged. The approved QEMU, Buildx, login and build-push action references are also unchanged.

Known limitation and deferred recommendation: `docker/metadata-action@v4` remains obsolete in both Docker workflows. A separate bounded action-upgrade task should update and validate both workflows; CI-DOCS-01 does not create or implement that task.

## 12. Manual workflow inspection

Trigger events and branches:

- build/test: pull requests, pushes to `develop`/`master`, and manual dispatch;
- migrations: pull requests, pushes to `develop`/`master`, and manual dispatch;
- PR Docker: pull requests and manual dispatch;
- check-changelog: pull requests, unchanged.

Dependencies and conditions:

- each heavy workflow has one `classify` job;
- heavy jobs declare `needs: classify`;
- job-level `if` expressions require the relevant output to equal `'true'`;
- a missing or invalid event/range emits all-heavy outputs, so uncertainty never selects skipping;
- if output emission itself fails, the classifier job fails and blocks dependent work.

Permissions, secrets and failure propagation:

- no workflow permissions, environments, concurrency or secret references changed;
- classifier checkout credentials are not persisted;
- Docker login and `GITHUB_TOKEN` use remain only inside the heavy Docker job;
- manual dispatch returns all-heavy outputs;
- classifier failure cannot silently produce a documentation-only result.

Context preservation:

- all six branch-protected job IDs/names are retained;
- the additional Docker job ID/name is retained;
- skipped heavy jobs remain in the workflow graph and yield terminal skipped contexts;
- `check-changelog` remains active and unchanged.

## 13. CI

Initial implementation head: `b6668d2183c2931d62ae56fd968407804252ac5d`

Initial classifier observations on draft PR #771:

- build/test classifier: passed;
- migration classifier: passed;
- Docker classifier: passed;
- heavy jobs were scheduled because the complete PR diff contains `.github/**`;
- `check-changelog` failed before the required PR-numbered changelog entry existed.

The initial run is not final-head acceptance evidence.

Final CI status: PENDING until the evidence commit is pushed.

Final acceptance requires all seven contexts on the final exact head:

```text
build
test
lint
format_check
run_migrations
check-changelog
build_and_push_staging_docker_image
```

Their exact run IDs and conclusions must be recorded externally in PR #771 and the implementation handoff. A failure on the final head is a stop condition.

## 14. Rollout and rollback

Initial state after merge: documentation-only PRs are classified in each relevant workflow and heavy jobs are skipped at job level. No production application behaviour changes.

Activation required: merge only, after fresh independent approval and explicit CTO authorization.

Feature flag/configuration: none.

Migration sequence: none.

Observation:

1. verify the merge commit is the live `develop` tip;
2. use the first controlled documentation-only PR to verify all six protected contexts, normal changelog execution, skipped Rust/migration/Docker heavy steps and no Docker credentials or build/push;
3. use the first mixed documentation/source PR to verify relevant heavy jobs and Docker run;
4. observe the next three PRs and record anomalies.

Rollback: normal revert of the CI-DOCS-01 merge.

Immediate rollback criteria:

- a protected context is absent or pending;
- a non-documentation change incorrectly skips relevant heavy work;
- documentation-only work executes heavy Rust, migration or Docker steps;
- Docker login/build/push executes for documentation-only work;
- mixed or non-documentation work incorrectly skips Docker;
- manual dispatch skips heavy work;
- only the latest commit is classified;
- uncertainty results in skipped work.

No database, data or production-state rollback is required.

## 15. Known limitations and deferred work

- Post-merge safely skipped behaviour has not yet been observed; it requires the controlled rollout PR.
- GitHub Actions skipped-job merge-safety must be confirmed in the controlled documentation-only observation PR.
- Obsolete Docker action maintenance is deferred to a separate bounded task.
- Final exact-head CI evidence is necessarily external to the commit it verifies.

## 16. Unresolved issues

- Final exact-head CI is pending.
- Fresh independent review is pending.
- Explicit CTO merge authorization is pending.

## 17. Agent self-assessment

Suggested independent-review focus:

- GitHub Actions skipped-job context semantics for the six protected checks;
- complete PR diff use of `base...head`, including deleted control paths;
- fail-closed behaviour for malformed, empty, all-zero and unavailable ranges;
- Docker job-level gating before checkout, metadata and credentials;
- preservation of all job identities, action versions and release workflow bytes;
- exact actionlint baseline equivalence;
- final exact-head execution of all seven contexts because this PR changes `.github/**`.

The implementing agent does not approve this task or authorize merge.
