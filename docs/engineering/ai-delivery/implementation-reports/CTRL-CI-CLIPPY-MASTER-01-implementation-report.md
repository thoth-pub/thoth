# CTRL-CI-CLIPPY-MASTER-01 Implementation Report

## 1. Repository state

Owning GitHub issue: [861](https://github.com/thoth-pub/thoth/issues/861)
Parent coordination issue: [850](https://github.com/thoth-pub/thoth/issues/850)
Blocked consumer: [851](https://github.com/thoth-pub/thoth/issues/851) / draft PR [860](https://github.com/thoth-pub/thoth/pull/860)
Repository: `thoth-pub/thoth`
Workflow: STANDARD (task-specific direct-to-`master` hotfix prerequisite, approved in #861)
Base branch: `master`
Authorized base commit: `40e9c06d4ab76217c3ef277dd539d3b5580e2bb8`
Actual base commit: `40e9c06d4ab76217c3ef277dd539d3b5580e2bb8` (authorized base tree `8797558959fcf1f418eb33ca080b469e97783cbd`, verified identical)
PR target: `master`
Programme integration branch: N/A
Task branch: `hotfix/ctrl-ci-clippy-master-01`
Head commit: recorded in the #861 implementation-evidence comment; this report is committed as part of that head
Pull request: **NOT CREATED - NOT AUTHORIZED AT THIS GATE**
Expected branch deletion after merge: YES
Final programme PR required: NO
Implementing model: `claude-opus-5`
Reasoning level: MEDIUM

### 1.1 Lifecycle records

- Independent specification review: comment `5454442905` - APPROVED
- CTO specification approval: comment `5454487997` - APPROVED
- Bounded implementation authorization: comment `5454533280` - AUTHORIZED

All three were read in full before any mutation and were current and materially
unchanged at implementation time.

### 1.2 Preflight verification

| Premise | Required | Observed |
|---|---|---|
| `origin/master` | `40e9c06d4ab76217c3ef277dd539d3b5580e2bb8` | identical |
| base tree | `8797558959fcf1f418eb33ca080b469e97783cbd` | identical |
| `hotfix/ctrl-ci-clippy-master-01` | absent | absent locally and remotely |
| #861 | OPEN, approvals current | OPEN, unchanged |
| PR #860 | OPEN, DRAFT, head `307f0435817d5df485ff3b51d61bc0b4585b57b8`, base `master` | identical |
| `thoth-api/src/model/tests.rs` at base | blob `45353520bf4d0ed199c1757b692f43bdbff8f389`, exactly three target assertions | identical; `grep -c 'format!("{}"'` = 3 |
| worktree | clean | clean (`git status --porcelain -uall` empty) |

No HOLD condition was triggered.

## 2. Scope confirmation

Approved specification: the complete body of #861, independently reviewed in
`5454442905` and CTO-approved in `5454487997`.

Implemented objective: propagate the already-approved and merged #844 Clippy
1.98 lint-baseline repair onto the release line, so that `master` restores a
green `-D warnings` Clippy baseline and #860 can subsequently be reconciled and
retested against a repaired `master`.

Out-of-scope changes made: NONE.

Explicitly not done: no lint suppression, no `allow` attribute, no Clippy
configuration, no Rust/toolchain pinning, no workflow redesign, no dependency or
lockfile change, no production-domain refactor, no adjacent cleanup, no
GraphQL/API or migration change, no ORCID batch (#851/#860) source change, no
Metrics change, and no wholesale cherry-pick of #844 commit
`f5b12a682a4cc4fc536563e73984d3b0fc3f628f`.

## 3. Commits

One bounded implementation commit:

- `fix(ci): propagate Clippy 1.98 baseline repair to master`

The exact SHA is reported in the #861 implementation-evidence comment and in the
final handoff. This report file is part of that same commit, so its own SHA
cannot be self-referentially embedded without moving the head it would record
(`operating-model.md` Gate 4).

## 4. Files changed

Authorized write paths (from the #861 implementation authorization):

- `thoth-api/src/model/tests.rs`
- `CHANGELOG.md`

Authorized new-file paths:

- `docs/engineering/ai-delivery/implementation-reports/CTRL-CI-CLIPPY-MASTER-01-implementation-report.md`

Actual files changed:

- `thoth-api/src/model/tests.rs`
  - reason: three `assert_eq!` expressions wrapped `Display` output in
    `format!("{}", ...)` purely to invoke `Display`. Clippy 1.98 on the
    GitHub-hosted runner rejects these under `clippy::useless_format`, failing
    the lint job under its `-D warnings` policy. Rust's blanket `ToString`
    implementation makes `.to_string()` exactly equivalent.
  - behavioural effect: NONE. Test-only. The three test names, their input
    values and their expected values are unchanged, and no production type,
    `Display` implementation, `with_domain` behaviour or parsing rule is
    touched.
  - within authorized write budget: YES
- `CHANGELOG.md`
  - reason: repository doctrine (root `AGENTS.md` section 13) requires an
    `## [Unreleased]` entry. A `### Fixed` heading was added because the
    `Unreleased` section was empty at the authorized base.
  - behavioural effect: NONE. Documentation only.
  - within authorized write budget: YES

Actual new files created:

- `docs/engineering/ai-delivery/implementation-reports/CTRL-CI-CLIPPY-MASTER-01-implementation-report.md` - within authorized new-file list: YES

Files deleted, moved or renamed: NONE.

No `.env`, symlink, copied environment/credential file, or any other unlisted
filesystem artifact was created at any point. `git status --porcelain -uall`
reports no untracked files.

### 4.1 Write-budget compliance

**PASS.** Exactly the two authorized modified paths and the one authorized new
path. No other path was written.

## 4.2 Authorized actions actually used

- repository inspection: USED
- source edit: USED (within the two authorized paths)
- new file creation: USED (the one authorized report path)
- file deletion/move/rename: NOT USED (not authorized)
- branch creation: USED (`hotfix/ctrl-ci-clippy-master-01` from the exact authorized base)
- commit: USED (one bounded commit)
- push: USED (task branch only)
- PR creation/update: **NOT USED - NOT AUTHORIZED AT THIS GATE**
- issue/comment mutation: USED, limited to one implementation-evidence comment on #861. No comment or mutation on #850, #851 or #860.
- manual CI dispatch/rerun: NOT USED (not authorized)
- provider/runtime read: NOT USED (not authorized)
- provider/runtime write: NOT USED (not authorized)
- migration execution: NOT USED (not authorized)
- release/tag/publication: NOT USED (not authorized)
- merge: NOT USED (not authorized)
- deployment: NOT USED (not authorized)
- production activation: NOT USED (not authorized)
- other: local validation only, against local disposable PostgreSQL and Redis. No production or provider endpoint was contacted and no production secret was accessed.

Unauthorized actions performed: **NONE.**

## 4.3 Automatic and manual external effects

Automatic CI/provider effects observed: **NONE expected and none observed at
this gate.** Repository workflows run build/test and migration CI on push only
for `master` and `develop`; `check-changelog` and the staging-container
workflow are `pull_request`-triggered; release-image publication is
release-triggered. Pushing `hotfix/ctrl-ci-clippy-master-01` therefore triggers
no workflow.

Manually initiated external actions: NONE.

External writes/publication (releases, tags, packages, registries, third-party
services): **NONE.** In particular no `ghcr.io/thoth-pub/thoth:staging-pr-*`
image was published, because no pull request exists.

## 5. Implementation decisions

1. Applied only the three-line already-reviewed correction from #844 rather than
   cherry-picking commit `f5b12a682a4cc4fc536563e73984d3b0fc3f628f` wholesale,
   as required by #861. The resulting `thoth-api/src/model/tests.rs` blob is
   `5b4697b3`, byte-identical to the post-correction blob produced by #844 on
   `develop`, and the added/removed lines were mechanically diffed against
   #844's own diff and confirmed identical.
2. Added a `### Fixed` heading under `## [Unreleased]` because that section was
   empty at the authorized base. No unrelated `develop` changelog content was
   copied onto `master`.
3. Did not touch any adjacent code, including other assertions in the same file,
   in line with the "no adjacent cleanup" instruction.
4. Did not create any environment file to unblock the three workspace-scoped
   validation commands; they are reported BLOCKED below rather than worked
   around or claimed as passing.

Deviations from the specification requiring authorization: NONE.

## 6. Database and migration effects

Migration added: **NO.**

No migration, no schema change, no `thoth-api/src/schema.rs` change, no
data effect and no locking or downtime implication. `migrations/` is untouched.

## 7. API and compatibility effects

GraphQL/API changes: **NONE.** No resolver, type, input, scalar, enum or field
is added, removed or altered.
Generated schema/client updates: NONE required. The change is confined to a test
module and cannot alter the generated SDL.
Backwards compatibility: unaffected.
Deprecations: NONE.
Cross-repository dependencies: NONE. No consumer of the Thoth GraphQL API,
export API or internal client is affected.

## 8. Authorization and security

Authorization paths changed: NONE.
Roles/scopes involved: NONE.
Negative authorization tests: not applicable; no authorization surface is
touched. `thoth-api/src/policy.rs` is untouched.
Secret or personal-data handling: NONE. No secret, credential or personal data
is read, written or logged, and no environment or credential file was created.
Security limitations: none introduced.

## 9. Tests and checks

Exact commands and outcomes. Local toolchain: `rustc 1.97.0 (2d8144b78
2026-07-07)`, `cargo 1.97.0`, `clippy 0.1.97 (2d8144b788 2026-07-07)`.

### Formatting

Command:

```text
cargo fmt --all -- --check
```

Result:

```text
exit 0, no output
```

### Unit tests

Command:

```text
cargo test -p thoth-api --features backend
```

Result:

```text
test model::tests::test_doi_with_domain ... ok
test model::tests::test_orcid_with_domain ... ok
test model::tests::test_ror_with_domain ... ok
test result: ok. 1178 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 102.65s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.61s
test result: ok. 0 passed; 0 failed; 8 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

The three corrected tests are individually named above and pass.

### Integration/database tests

Command:

```text
cargo test --workspace
```

Result:

```text
BLOCKED - NOT VERIFIED

error: failed to run custom build command for `thoth-export-server v1.7.0`
Caused by:
  process didn't exit successfully: .../build/thoth-export-server-.../build-script-build (exit status: 101)
  --- stdout
  cargo:rerun-if-changed=../.env
  --- stderr
  thread 'main' panicked at thoth-export-server/build.rs:22:60:
  called `Result::unwrap()` on an `Err` value: Os { code: 2, kind: NotFound, message: "No such file or directory" }
```

### Lint/static analysis

Command:

```text
cargo clippy --all --all-targets --all-features -- -D warnings
```

Result:

```text
BLOCKED - NOT VERIFIED

Same thoth-export-server build-script failure as above.
```

Command:

```text
cargo check --workspace
```

Result:

```text
BLOCKED - NOT VERIFIED

Same thoth-export-server build-script failure as above.
```

Supplementary, **not** a substitute for the required workspace command:

```text
cargo clippy -p thoth-api --all-targets --all-features -- -D warnings
```

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 31s
(no warnings, no errors)
```

### Other required checks

```text
git diff --check      -> exit 0, no output
git diff --stat       -> CHANGELOG.md | 2 ++
                         thoth-api/src/model/tests.rs | 6 +++---
                         2 files changed, 5 insertions(+), 3 deletions(-)
git status --short    -> M CHANGELOG.md
                         M thoth-api/src/model/tests.rs
git status --porcelain -uall (untracked) -> none
```

### 9.1 Validation limitations - read before relying on the local evidence

Two independent limitations apply, and neither is worked around:

1. **Three required commands are BLOCKED by a pre-existing local environment
   assumption.** `thoth-export-server/build.rs` calls
   `fs::read_to_string("../.env").unwrap()` after `dotenv()` succeeds by finding
   a `.env` higher up the filesystem, so any workspace-scoped Cargo command
   panics in that build script when the repository working directory has no
   `.env` of its own. This was reproduced against the **unmodified authorized
   base**, before any edit in this task, so it is environmental and provably not
   caused by this change. Per the #861 authorization and the implementation
   handoff, no `.env`, symlink or copied environment file was created to force
   these commands to run. They are recorded as BLOCKED / NOT VERIFIED, never as
   passing.
2. **The local toolchain cannot reproduce the CI lint failure this task
   repairs.** Local Clippy is `0.1.97`; the `clippy::useless_format` findings
   that fail #860's CI arise on the GitHub runner's Clippy 1.98. A local Clippy
   run over the *uncorrected* tree passes, so local Clippy success is weak
   evidence that the CI lint is now green. The strong evidence is instead that
   the corrected `thoth-api/src/model/tests.rs` is byte-identical (blob
   `5b4697b3`) to the file that #844 produced, whose final reviewed head
   `5444081aba7adafb17822cb17fe67e6326828a79` had green automatic CI including
   the Clippy job.

Both limitations must be closed by PR CI on a clean GitHub runner, which is a
later gate contingent on separate PR/GHCR authorization.

## 10. Manual verification

Environment: local macOS worktree; local disposable PostgreSQL 17 and Redis for
the `thoth-api` backend tests. No production or provider endpoint contacted.

Steps:

1. Verified the authorized base commit, tree and the `thoth-api/src/model/tests.rs`
   blob hash `45353520bf4d0ed199c1757b692f43bdbff8f389` at that base.
2. Verified exactly three `format!("{}", ...)` expressions existed in the file.
3. Applied the three replacements.
4. Verified zero `format!("{}"` occurrences remain in the file.
5. Diffed the added/removed lines against #844's own diff for the same file.

Observed result: the two diffs are identical, and the resulting blob is
`5b4697b3`, matching #844's post-correction blob.

Evidence link: #844 commit `f5b12a682a4cc4fc536563e73984d3b0fc3f628f`; the #861
implementation-evidence comment carries the full command output.

## 11. CI

CI status: **NOT AVAILABLE.**

No pull request exists, because PR creation is not authorized at this gate, so
no PR CI has run against this head. Pushing the task branch triggers no
workflow: build/test and migration CI run on push only for `master` and
`develop`, `check-changelog` and the staging-container workflow are
`pull_request`-triggered, and release-image publication is release-triggered.

Checks: none available.
Failures or warnings: none observable at this gate.

PR CI on a clean GitHub runner - which is the only environment that reproduces
the Clippy 1.98 behaviour this task repairs - remains a later gate after
separate PR and GHCR authorization.

## 12. Rollout and rollback

Initial state after merge: the release-line Clippy baseline is restored. No
runtime behaviour changes, because the change is confined to a test module that
is not compiled into any shipped binary.

Activation required: NONE.
Feature flag/configuration: NONE.
Migration sequence: NONE.

Rollback/disable procedure: revert the single bounded commit. There is no data,
schema, API or runtime state to unwind, and no consumer depends on the change.

Monitoring required: NONE.

Integration ordering, per #861:

1. independent exact-head review of this branch;
2. separate PR creation authorization, explicitly covering the automatic
   `ghcr.io/thoth-pub/thoth:staging-pr-<PR>` publication;
3. PR CI green on a clean runner;
4. separate merge authorization and merge to `master`;
5. separately authorized reconciliation of the repaired `master` into #860;
6. fresh exact-head review and CI for the resulting #860 head.

Steps 5 and 6 are outside this task. #860 was not mutated, rebased or merged.

## 13. Known limitations and deferred work

- Three of the eight required validation commands are BLOCKED by the
  pre-existing `thoth-export-server/build.rs` `.env` assumption described in
  section 9.1. This is an environment limitation reproduced at the unmodified
  authorized base, not a defect introduced here, and it was not worked around.
- Local Clippy 0.1.97 cannot reproduce the Clippy 1.98 `useless_format`
  findings, so local lint success does not by itself prove the CI lint job is
  repaired. Blob-level equivalence with the CI-green #844 result is the stronger
  evidence; PR CI remains the authoritative confirmation.
- The `thoth-export-server/build.rs` fragility itself - `dotenv()` succeeding
  from an ancestor directory while `fs::read_to_string("../.env")` then panics -
  is a genuine separate defect that makes workspace validation impossible in a
  worktree without its own `.env`. It is out of scope here and would need its
  own specification, task and authorization.

## 14. Unresolved issues

- Whether the control plane treats the three BLOCKED validation commands as
  satisfying #861's validation requirement at this gate, or requires them to be
  closed by PR CI before source approval. The implementation handoff's section
  11 directs that blocked commands be reported as BLOCKED and the work
  continued, while its section 13 also lists blocked required validation as a
  HOLD condition. The more specific instruction was followed and the tension is
  recorded here rather than resolved unilaterally.

## 15. Agent self-assessment

The implementing agent may identify risks but may not approve the task. This
task is **not self-approved**.

Suggested review focus:

- confirm the three-line diff is byte-identical to #844's reviewed correction
  and that the resulting file blob is `5b4697b3`;
- confirm the write budget held: exactly `thoth-api/src/model/tests.rs`,
  `CHANGELOG.md` and this report file, with no deletions, moves or renames and
  no untracked artifacts;
- confirm no `.env` or other unlisted filesystem artifact was created at any
  point;
- decide whether the three BLOCKED workspace validation commands are acceptable
  at this gate or must be closed by PR CI first (section 14);
- confirm the changelog entry claims test/CI compatibility only and asserts no
  production behaviour change;
- confirm #860 remains untouched and no PR, GHCR publication, merge, deployment
  or provider action occurred.
