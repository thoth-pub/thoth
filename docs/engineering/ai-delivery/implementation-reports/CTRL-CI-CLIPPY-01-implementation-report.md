# CTRL-CI-CLIPPY-01 Implementation Report

## 1. Repository state

Owning GitHub issue: [#844](https://github.com/thoth-pub/thoth/issues/844)
Repository: `thoth-pub/thoth`
Programme: Shared Engineering Control
Workflow: STANDARD
Risk: LOW
Base branch: `develop`
Authorized base commit: `a6c8cb2016179db635c4bc86ef366aae190829c2`
Actual base commit: `a6c8cb2016179db635c4bc86ef366aae190829c2`
PR target: `develop`
Programme integration branch: None
Task branch: `feature/engineering-control/ctrl-ci-clippy-01`
Head commit: recorded in section 1.4
Pull request: recorded in section 1.4
Expected branch deletion after merge: YES (separately authorized)
Final programme PR required: NO
Blocked consumer: `MET-WP1-02` / [#841](https://github.com/thoth-pub/thoth/issues/841) / draft PR [#843](https://github.com/thoth-pub/thoth/pull/843)
Implementing model: Claude Opus 5
Reasoning level: MEDIUM

### 1.1 Approval and authorization records

| Gate | Record | Outcome |
|---|---|---|
| Independent specification review | issue comment `5442740800` | `APPROVED` |
| CTO specification approval | issue comment `5442767976` | `APPROVED` |
| Bounded implementation authorization / handoff | issue comment `5442804789` | `IMPLEMENTATION AUTHORIZED WITHIN BOUNDS` |

All three records were read in full immediately before any mutation. #844 was
`OPEN`, its `updatedAt` was `2026-08-27T17:29:43Z` — exactly the timestamp of
authorization comment `5442804789` — and no comment exists after that
authorization. The specification/approval chain was therefore materially
unchanged from the state the authorization describes.

### 1.2 Preflight, performed before any mutation

```bash
git fetch --all --prune
git rev-parse origin/develop
git ls-remote --heads origin 'refs/heads/feature/engineering-control/*'
git ls-remote --heads origin refs/heads/feature/engineering-control
git rev-parse origin/develop:thoth-api/src/model/tests.rs
gh issue view 844 --repo thoth-pub/thoth --json number,state,updatedAt,body
gh api repos/thoth-pub/thoth/issues/844/comments
gh pr list --repo thoth-pub/thoth --state open
git status --short
```

Observed:

```text
origin/develop                                a6c8cb2016179db635c4bc86ef366aae190829c2
origin/develop:thoth-api/src/model/tests.rs   45353520bf4d0ed199c1757b692f43bdbff8f389
feature/engineering-control/ctrl-branch-namespace-01  (present, descendant sibling)
feature/engineering-control                   (absent — no flat parent ref)
feature/engineering-control/ctrl-ci-clippy-01 (absent)
issue #844                                    OPEN, updatedAt 2026-08-27T17:29:43Z
working tree                                  clean
```

Preflight results:

1. Remote fetched. **PASS**
2. `origin/develop` is exactly the authorized base `a6c8cb20…`. No rebinding
   required. **PASS**
3. #844 `OPEN`; review comment `5442740800`, CTO approval `5442767976` and
   authorization `5442804789` present and unchanged; no later comment. **PASS**
4. ADR-0009 / root `AGENTS.md` section 5.1 namespace preflight for the
   descendant ref `feature/engineering-control/ctrl-ci-clippy-01`: the exact ref
   is absent; no flat parent ref `feature/engineering-control` occupies the
   location; the existing sibling `feature/engineering-control/ctrl-branch-namespace-01`
   confirms the descendant namespace is already valid; `ctrl-ci-clippy-01` is a
   single path segment containing no reserved `--` token. **PASS**
5. No conflicting active PR or task exists for this branch/task. The six open
   PRs are #843, #799, #752, #744, #742 and #668; none uses this head branch and
   none targets this task. **PASS**
6. The three intended source expressions are present verbatim in
   `thoth-api/src/model/tests.rs`, and the file's blob SHA on `develop` is
   `45353520bf4d0ed199c1757b692f43bdbff8f389`, exactly as recorded in #844
   section 2. Root-cause evidence unchanged. **PASS**
7. Classifier and workflow reverification: see section 4.3. **PASS**

No HOLD condition was met, so implementation proceeded.

### 1.3 Root-cause reproduction and fix evidence

The repository's local default toolchain is Rust/Clippy **1.97.0**, one minor
version behind the GitHub-hosted runner's **1.98.0** that produced the #844
failure, so the default local toolchain does not reproduce the defect. To avoid
reporting an unreproduced fix, the extended `clippy::useless_format` behaviour
was exercised with the locally installed newer Clippy `0.1.99`
(`rustc 1.99.0-nightly (da80ed070 2026-07-14)`), which is at or beyond the CI
version and carries the same extended lint.

Command, run twice on the same worktree:

```bash
rustup run nightly cargo clippy -p thoth-api --all-targets --all-features
```

Result:

```text
unmodified base file  -> 3 x "useless use of `format!`" (clippy::useless_format),
                         thoth-api/src/model/tests.rs
corrected file        -> 0 x clippy::useless_format
```

This reproduces exactly the three-warning failure #844 attributes to lint job
`98595021926` and demonstrates the corrected file clears it. This was a
read-only validation using an already-installed toolchain; **no** repository
toolchain file, pin, dependency, lockfile or workflow was added or changed, and
the default toolchain remains untouched. The authoritative confirmation under
the exact CI toolchain is the automatic PR `build-test-and-check` run recorded
in section 11.

### 1.4 Head, commits and pull request

Head commit: `PENDING`
Pull request: `PENDING`

## 2. Scope confirmation

Approved specification: the complete #844 issue body, independently reviewed in
comment `5442740800` and approved in comment `5442767976`.

Implemented objective: restore the repository-wide Rust lint baseline on
`develop` by replacing exactly the three pre-existing `format!("{}", …)`
wrappers in `thoth-api/src/model/tests.rs` with the semantically direct
`.to_string()` form, without changing production behaviour and without weakening
lint policy.

Out-of-scope changes made: NONE. No adjacent cleanup or refactoring was
performed, including in the same file and the same test module.

## 3. Commits

- `PENDING` - `CTRL-CI-CLIPPY-01: restore the Rust/Clippy 1.98 lint baseline (#844)`

## 4. Files changed

Authorized write paths (existing files):

- `thoth-api/src/model/tests.rs`
- `CHANGELOG.md`

Authorized new-file paths:

- `docs/engineering/ai-delivery/implementation-reports/CTRL-CI-CLIPPY-01-implementation-report.md`

Actual files changed:

- `thoth-api/src/model/tests.rs`
  - reason: the three `*_with_domain` test assertions used `format!("{}", value)`
    solely to invoke `Display`; Clippy 1.98 rejects this under
    `clippy::useless_format` and the repository lint policy is `-D warnings`.
  - behavioural effect: none. Rust's blanket `ToString` implementation for
    `Display` makes `.to_string()` produce the identical string, so the three
    tests assert exactly the same values against exactly the same inputs. Change
    is test-only and affects no production compilation unit.
  - within authorized write budget: YES

- `CHANGELOG.md`
  - reason: required bounded Unreleased entry for the CI/test compatibility
    correction.
  - behavioural effect: none; documentation only.
  - within authorized write budget: YES

Actual new files created:

- `docs/engineering/ai-delivery/implementation-reports/CTRL-CI-CLIPPY-01-implementation-report.md` - within authorized new-file list: YES

Files deleted, moved or renamed: NONE.

### 4.1 Write-budget compliance

**PASS.**

`git status --short` and `git diff --stat` against the authorized base list
exactly three paths, all within the approved write/new-file budget. No file
outside the budget was created, modified, deleted, moved or renamed.

### 4.2 Exact source substitution

Exactly three expressions were replaced. Each substitution was applied only
after asserting the search string occurred exactly once in the file, and no
`format!("{}", …)` wrapper remains anywhere in
`thoth-api/src/model/tests.rs`.

```diff
 fn test_doi_with_domain() {
     let doi = "https://doi.org/10.12345/Test-Suffix.01";
-    assert_eq!(format!("{}", Doi(doi.to_string()).with_domain()), doi);
+    assert_eq!(Doi(doi.to_string()).with_domain().to_string(), doi);
 }

 fn test_orcid_with_domain() {
     let orcid = "https://orcid.org/0000-0002-1234-5678";
-    assert_eq!(format!("{}", Orcid(orcid.to_string()).with_domain()), orcid);
+    assert_eq!(Orcid(orcid.to_string()).with_domain().to_string(), orcid);
 }

 fn test_ror_with_domain() {
     let ror = "https://ror.org/0abcdef12";
-    assert_eq!(format!("{}", Ror(ror.to_string()).with_domain()), ror);
+    assert_eq!(Ror(ror.to_string()).with_domain().to_string(), ror);
 }
```

Preservation verified line by line against the base:

| Test | Name changed | Input changed | Expected value changed |
|---|---|---|---|
| `test_doi_with_domain` | NO | NO (`https://doi.org/10.12345/Test-Suffix.01`) | NO (`doi`) |
| `test_orcid_with_domain` | NO | NO (`https://orcid.org/0000-0002-1234-5678`) | NO (`orcid`) |
| `test_ror_with_domain` | NO | NO (`https://ror.org/0abcdef12`) | NO (`ror`) |

The complete source diff is 3 changed lines (`3 insertions(+), 3 deletions(-)`)
in one test file. No `Doi`, `Orcid`, `Ror`, `Display`, `with_domain` or parsing
implementation was touched; no production Rust source changed at all.

No `#[allow(...)]`, `#![allow(...)]`, `expect`, `cfg_attr` lint escape or any
other lint suppression was added, and the repository lint policy remains
`cargo clippy --all --all-targets --all-features -- -D warnings`, unmodified.

### 4.3 Authorized actions actually used

- repository inspection: YES
- source edit (within write budget): YES
- new file creation (authorized path): YES
- file deletion/move/rename: NO
- branch creation: YES (`feature/engineering-control/ctrl-ci-clippy-01` from the
  exact authorized base)
- commit: YES
- push: YES (authorized task branch only)
- PR creation/update: YES (DRAFT PR to `develop`)
- issue/comment mutation: NO
- manual CI dispatch/rerun: NO
- provider/runtime read: NO
- provider/runtime write: NO
- migration execution: NO
- release/tag/publication: NO, other than the automatic staging-PR container
  image described below, which the authorization explicitly accepts as a
  non-release external side effect
- merge: NO
- deployment: NO
- production activation: NO
- other: NONE

Unauthorized actions performed: **NONE.**

Explicitly not done, per the authorization: no `feature/metrics` refresh, no
modification of `feature/metrics--wp1-source-state`, no rebase/update/cherry-pick
into or any other mutation of PR #843, no comment on #844, no manual CI rerun,
no branch deletion.

### 4.4 Automatic and manual external effects

Classifier reverification. The changed-file set is
`thoth-api/src/model/tests.rs`, `CHANGELOG.md` and one path under `docs/`.
Under the current `.github/scripts/classify_ci_changes.py`, the set is not
documentation-only (`is_documentation_path` matches `CHANGELOG.md` and `docs/`
but not the `.rs` file); `is_build_path` matches the `.rs` file; and
`is_migration_path` matches none of them (no `src/bin/` path, no `up.sql`,
`down.sql` or `db.rs`, and no migration control path). Expected classification:

```text
docs_only=false
run_build=true
run_migrations=false
run_docker=true
```

This is exactly the approved candidate classification in #844 section 13.

Workflow reverification against `develop` at the authorized base:

- `.github/workflows/build_test_and_check.yml` — runs on `pull_request`;
  classify + build + test + Clippy + rustfmt gated on `run_build=true`;
- `.github/workflows/run_migrations.yml` — runs on `pull_request`; the classify
  job runs, and the `run_migrations` job is gated on `run_migrations == 'true'`,
  so migration execution is skipped;
- `.github/workflows/check_changelog.yml` — runs unconditionally on
  `pull_request`;
- `.github/workflows/docker_build_and_push_to_dockerhub.yml` — runs on
  `pull_request`; gated on `run_docker=true`; `REGISTRY: ghcr.io`, images
  `ghcr.io/thoth-pub/thoth`, tags `type=ref,event=pr,prefix=staging-pr-`,
  `push: true`.

Registry, tag semantics and gating are materially unchanged from #844
section 13, so no `HOLD - AUTOMATIC SIDE-EFFECT REBINDING REQUIRED` applies.

Automatic external write expected and accepted: the staging PR container image
`ghcr.io/thoth-pub/thoth:staging-pr-<PR>` is built and pushed to GHCR by the
`publish-to-dockerhub` workflow. Observed status is recorded in section 11.
This is a staging side effect, not a release, deployment or production
activation.

Manually initiated external actions: **NONE.** No workflow dispatch and no CI
rerun was triggered.

Other external writes/publication (releases, tags, packages, third-party
services): **NONE.**

## 5. Implementation decisions

1. Applied the substitutions programmatically with an exact-match, occurrence-count
   assertion (each search string had to occur exactly once) rather than by
   pattern rewriting, so no unintended expression elsewhere in the file could be
   altered.
2. Placed the changelog entry under a new `### Fixed` heading in `[Unreleased]`,
   matching the repository's existing Keep a Changelog section vocabulary
   (`Added`, `Changed`, `Fixed`, `Removed`, `Security`) and the file's blank-line
   convention before a subsection heading that follows a list. No existing
   changelog entry was edited.
3. Used the already-installed newer Clippy `0.1.99` toolchain read-only to
   reproduce and then clear the lint (section 1.3), because the default local
   toolchain 1.97.0 cannot reproduce the CI 1.98.0 failure. No toolchain pin,
   dependency or workflow was changed to achieve this.

Deviations from the specification requiring authorization: **NONE.**

## 6. Database and migration effects

Migration added: **NO.**

Database/schema effect: NONE. No migration was executed, and none is implied by
this change. The `run-migrations` workflow's classify job will run and the
migration execution job is expected to be skipped (`run_migrations=false`).

## 7. API and compatibility effects

GraphQL/API changes: NONE.
Generated schema/client updates: NONE.
Backwards compatibility: unaffected; the change is confined to test code.
Deprecations: NONE.
Cross-repository dependencies: NONE. #844 section 7 records every listed
contract as NOT AFFECTED, and this implementation introduces no change to any of
them.

## 8. Authorization and security

Authorization paths changed: NONE.
Roles/scopes involved: NONE.
Negative authorization tests: not applicable; no authorization behaviour is
touched.
Secret or personal-data handling: NONE. No secret, credential or personal data
was read, written or logged.
Security limitations: none introduced. No lint suppression was added, so no
static-analysis coverage was reduced.

## 9. Tests and checks

All commands were run fresh, post-change, on the task branch at the
implementation head. Local database-backed tests used the repository's
`.env`-declared `TEST_DATABASE_URL`, `TEST_REDIS_URL` and `THOTH_EXPORT_API`
exported into the process environment.

### Formatting

Command:

```text
cargo fmt --all -- --check
```

Result:

```text
exit status 0; no output (no formatting differences)
```

### Unit tests

Command:

```text
cargo test -p thoth-api --features backend
```

Result:

```text
exit status 0
test result: ok. 1229 passed; 0 failed; 0 ignored
test result: ok. 13 passed; 0 failed; 0 ignored
test result: ok. 0 passed; 0 failed; 8 ignored
```

Targeted confirmation of the three affected tests:

```text
cargo test -p thoth-api --features backend with_domain

test model::tests::test_doi_with_domain ... ok
test model::tests::test_orcid_with_domain ... ok
test model::tests::test_ror_with_domain ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 1226 filtered out
```

### Integration/database tests

Command:

```text
cargo test --workspace
```

Result:

```text
exit status 0
all test binaries ok; 0 failed across the workspace
(largest binaries: 1229 passed, 144 passed, 31 passed, 13 passed, 11 passed,
 6 passed, 4 passed, 3 passed, 2 passed; 8 ignored)
```

### Lint/static analysis

Command:

```text
cargo clippy --all --all-targets --all-features -- -D warnings
```

Result:

```text
exit status 0
Finished `dev` profile [unoptimized + debuginfo] target(s)
(clippy 0.1.97 / rustc 1.97.0 — the repository default local toolchain)
```

Supplementary reproduction and fix confirmation under a Clippy at or beyond the
CI version, per section 1.3:

```text
rustup run nightly cargo clippy -p thoth-api --all-targets --all-features
(clippy 0.1.99 / rustc 1.99.0-nightly)

base file      -> 3 x clippy::useless_format in thoth-api/src/model/tests.rs
corrected file -> 0 x clippy::useless_format
```

### Other required checks

Command:

```text
cargo check --workspace
```

Result:

```text
exit status 0
```

Command:

```text
git diff --check
```

Result:

```text
exit status 0; no output (no whitespace errors)
```

Pre-existing, unrelated and unchanged by this task: the workspace emits
`warning: the following packages contain code that will be rejected by a future
version of Rust: proc-macro-error2 v2.0.1`. This is a cargo future-incompatibility
notice about a third-party dependency, is present identically at the authorized
base, and is not a Clippy or rustc diagnostic against repository code.

### Recorded repository state

```text
git status --short
git diff --stat
git diff a6c8cb2016179db635c4bc86ef366aae190829c2...HEAD
git rev-parse HEAD
```

Observed:

```text
 CHANGELOG.md                 | 3 +++
 thoth-api/src/model/tests.rs | 6 +++---
 2 files changed, 6 insertions(+), 3 deletions(-)
```

plus the new untracked implementation report at its authorized path. The exact
committed diff and head SHA are recorded in section 1.4.

## 10. Manual verification

Environment: local macOS workstation, repository worktree on
`feature/engineering-control/ctrl-ci-clippy-01` at the authorized base, with the
repository's local Postgres 17 and Redis 6379 test services available.

Steps:

1. Confirmed the three target expressions existed verbatim on `develop` and that
   the file blob SHA matched #844 section 2.
2. Applied exactly three single-occurrence substitutions.
3. Confirmed zero `format!("{}", …)` wrappers remain in the file.
4. Reviewed the full `git diff` line by line to confirm test names, inputs and
   expected values were preserved and nothing else in the file changed.
5. Reproduced the 3-warning `clippy::useless_format` failure on the unmodified
   file and confirmed 0 warnings on the corrected file under Clippy 0.1.99.
6. Ran the complete #844 acceptance command set.

Observed result: all checks pass; the three functional tests assert identical
outputs; the lint defect is demonstrably cleared without suppression.

Evidence: command transcripts in section 9 and section 1.3; automatic CI
evidence in section 11.

## 11. CI

CI status: `PENDING`

Checks: recorded in section 1.4 / below once the draft PR's automatically
triggered workflows report.

Expected, per section 4.3: `build-test-and-check` runs; `run-migrations`
classify runs with migration execution **skipped**; `check-changelog` runs;
`publish-to-dockerhub` builds and pushes
`ghcr.io/thoth-pub/thoth:staging-pr-<PR>`.

No CI run was manually dispatched or rerun.

## 12. Rollout and rollback

Initial state after merge: the corrected lint baseline is restored on `develop`.
No behavioural change reaches any runtime.

Activation required: NONE.
Feature flag/configuration: NONE.
Migration sequence: NONE.

Rollback/disable procedure: revert the bounded test-only commit. There is no
data, schema or external-state rollback to perform.

Monitoring required: NONE.

Downstream sequence, explicitly **not** authorized by this task and each
requiring its own authorization:

1. independent exact-head source review of this PR;
2. merge of this PR to `develop`;
3. refresh of `develop` into `feature/metrics`;
4. reconciliation and fresh review of PR #843.

## 13. Known limitations and deferred work

- The repository's default local toolchain is Rust/Clippy 1.97.0, which does not
  reproduce the CI 1.98.0 `clippy::useless_format` failure. The defect was
  reproduced and the fix confirmed with the locally installed Clippy 0.1.99
  (section 1.3), but the authoritative confirmation under the exact CI toolchain
  is the automatic PR `build-test-and-check` run in section 11.
- This task restores the lint baseline on `develop` only. PR #843 remains blocked
  until the separately authorized `feature/metrics` refresh and reconciliation
  are performed; this task deliberately did not touch #843 or either metrics
  branch.
- No guard was added to prevent a future toolchain bump from introducing new
  lint failures; that is outside this task's scope and was not authorized.

## 14. Unresolved issues

NONE.

## 15. Agent self-assessment

The agent may identify risks but may not approve the task. This implementation
is **not** self-approved. The next gate is independent exact-head source review,
and any later source commit invalidates that review.

Suggested review focus:

- confirm the committed diff touches exactly three lines of
  `thoth-api/src/model/tests.rs` and that test names, inputs and expected values
  are byte-identical to the base;
- confirm no lint suppression, lint-policy weakening, dependency, lockfile,
  toolchain or workflow change is present anywhere in the diff;
- confirm the changed-file set is exactly the three authorized paths and that
  the actual base is `a6c8cb2016179db635c4bc86ef366aae190829c2`;
- confirm the automatic CI run for the exact head is green under the real CI
  Clippy 1.98 toolchain, that migration execution was skipped, and that the only
  external write is the staging-PR GHCR image;
- confirm PR #843, `feature/metrics` and `feature/metrics--wp1-source-state` are
  untouched by this task.
