# MET-WP11-B-CLIPPY-01 Implementation Report

## 1. Repository state

Owning GitHub issue: [#985](https://github.com/thoth-pub/thoth/issues/985)
Parent programme-integration task: [#981](https://github.com/thoth-pub/thoth/issues/981)
Outer integration PR: [#984](https://github.com/thoth-pub/thoth/pull/984) (`feature/metrics` -> `develop`)
Repository: `thoth-pub/thoth`
Programme: Thoth Metrics, WP11-B (final inactive Thoth API integration)
Workflow: PROGRAMME_INTEGRATION — the single one-time post-A2 CI-correction slice
governed by `thoth-pub/thoth#981 / thoth-pub/thoth#985`
Risk: HIGH (dependency code-generation change within the CRITICAL B integration)
Base branch: `feature/metrics`
Authorized base commit: `d5451f8516fa18126f6f4969fb3409775408e242` (tree `c15d07c3d48bb84fab6033307369c51de2258506`)
Actual base commit: `d5451f8516fa18126f6f4969fb3409775408e242` (tree `c15d07c3d48bb84fab6033307369c51de2258506`)
Included `develop` commit: `b44303c214498baf76c9d9a0a7cab374377f9cbe` (tree `050e1e2954b518d62988e31d74a7d9c280b8822c`), unchanged; no newer `develop` commit is imported
PR target: `feature/metrics`
Programme integration branch: `feature/metrics`
Task branch: `feature/metrics--wp11-b-clippy`
Head commit: the single commit on the task branch that adds this report. A
commit cannot contain its own SHA, so the exact head is the one recorded on the
draft pull request; the independent exact-head source review binds to that SHA.
Pull request: the draft pull request from `feature/metrics--wp11-b-clippy` into
`feature/metrics` for #985. Its number, live head, CI and review state are
GitHub records.
Expected branch deletion after merge: YES — only through the renewed exact-list
A2 retirement under its own authorization; `delete_branch_on_merge` was `false`
at preflight, so a merge does not delete the ref.
Final programme PR required: YES — the existing outer PR #984, separately gated.
Implementing model: Claude Opus 5.5 (`claude-opus-5-5`)
Reasoning level: set by the operator session; not independently observable by
the implementing agent.

### 1.1 Approval and authorization records

| Gate | Record | Outcome |
|---|---|---|
| Parent specification, Amendments 1-6 | #981 CTO approval `6126801903` | `APPROVED` |
| Original A2 completion receipt | #981 `6198611715` | A2 COMPLETE, 37/37 slice refs retired |
| Amendment 7A / 7B / 7C | #981 `6273625367` / `6288115189` / `6288991064` | approved cumulatively below |
| Final independent cumulative specification review | #981 `6296015048` | `APPROVED - SPECIFICATION ONLY` |
| CTO cumulative specification approval | #981 `6296026365` | `APPROVED - SPECIFICATION ONLY` |
| Implementation, branch, commit, push and draft-PR authorization | CTO implementation handoff for `MET-WP11-B-CLIPPY-01`, issued in the engineering-control conversation (no GitHub record was authorized for it) | bounded actions listed in section 4.2 |

The approved #985 body was re-read immediately before any mutation: updated
`2026-10-20T10:39:08Z`, UTF-8 SHA-256
`e27f048a5ed25d5b5fdeeb3e2823cc38b3ceddb5329e21ee6c4bf082034edd1f`, matching
the hash bound by `6296015048` and `6296026365`. The body's own header still
carries its historical pre-approval DRAFT/HOLD wording; the later independent
review and CTO approval on #981 govern, as both records state. The earlier
unapproved Amendment 7 (`6272787777`) and its authored-Rust-edit premise were
not implemented.

### 1.2 Preflight, performed before any mutation

```bash
git fetch origin --prune
git rev-parse origin/feature/metrics origin/feature/metrics^{tree} origin/develop origin/develop^{tree}
git merge-base --is-ancestor <D> <M>; git rev-list --left-right --count <M>...<D>
git ls-remote origin 'refs/heads/*'
gh pr view 984 --repo thoth-pub/thoth --json state,isDraft,mergedAt,headRefOid,baseRefOid
gh pr list --repo thoth-pub/thoth --state all --head feature/metrics--wp11-b-clippy
gh pr list --repo thoth-pub/thoth --state open --base feature/metrics
gh api repos/thoth-pub/thoth --jq .delete_branch_on_merge
gh api repos/thoth-pub/thoth/issues/985
gh api repos/thoth-pub/thoth/issues/comments/<id>   # each record in section 1.1
```

Results:

1. `feature/metrics` = M `d5451f85…` / tree `c15d07c3…`; `develop` = D
   `b44303c2…` / tree `050e1e29…`; D is an ancestor of M; M is 171 commits
   ahead and 0 behind D. **PASS**
2. #984 `OPEN`, draft, unmerged, head M, base D. **PASS**
3. No open PR targets `feature/metrics`; no PR in any state has head
   `feature/metrics--wp11-b-clippy`. **PASS**
4. Namespace preflight (root `AGENTS.md` section 5.1): the exact ref
   `feature/metrics--wp11-b-clippy` was absent; no descendant ref exists beneath
   it; no flat `refs/heads/feature` ref exists; no `feature/metrics/` descendant
   exists; `metrics` and `wp11-b-clippy` are single path segments without `--`.
   No live `feature/metrics--*` ref existed. **PASS**
5. All 37 refs retired by the original A2 receipt remain absent from the live
   ref list (empty intersection). **PASS**
6. Approval chain and #985 body hash as in section 1.1. **PASS**
7. The four post-A2 doctrine paragraphs at M named develop-refresh as the only
   exception (the transient discrepancy governed by approved Amendment 7C §2).
   CI workflows and `.github/scripts/classify_ci_changes.py` at M match the
   behaviour the specification relies on (section 4.4). **PASS**
8. `delete_branch_on_merge=false` (preflight evidence only, not merge
   authority). **PASS**

The branch was then created from exact M in a dedicated worktree outside the
main checkout; no other branch or worktree was touched.

### 1.3 Root cause and fix evidence

The natural lint failure at M (run `38806270979`, lint job `116667915795`,
runner image `ubuntu-24.04` version `20261012.334.1`) contains 347 rendered and
346 unique `clippy::redundant_field_names` positions, single lint class, in
exactly the 36 Rust paths and per-file counts inventoried in #985. Each points
at a field declaration of a `#[derive(diesel::QueryableByName)]` struct.

The cached registry archives were verified against the crates.io sparse index
(`https://index.crates.io/di/es/diesel_derives`): `diesel_derives-2.3.9.crate`
SHA-256 `d1817b7f…bddd1cf5` and `diesel_derives-2.3.10.crate` SHA-256
`ecbd51fb…5e72de03a`, both equal to the index `cksum`, neither yanked, with
identical published dependency declarations. `diesel 2.3.10` declares
`diesel_derives = "~2.3.0"`.

The source difference between the two releases that affects generated code is
confined to `src/queryable_by_name.rs`:

```diff
-                diesel::deserialize::Result::Ok(Self {
-                    #(
-                        #field_names: #fields,
-                    )*
-                })
+                diesel::deserialize::Result::Ok(Self {
+                    #(#field_constructor,)*
+                })
```

where `field_constructor` emits the shorthand `#f_n` for named fields and keeps
`#field_name: #f_n` for tuple fields. For a named field, `get_ident` returns the
field's own identifier, so `Self { id }` and the former `Self { id: id }` bind the
same local to the same field. The other source changes are a `format!`
argument spelling in a compile-time `table!` diagnostic message
(`&column_names[0]` -> `column_names[0]`, identical `Display` output), test
snapshots, and `build.rs` code compiled only under `cfg(diesel_docsrs)`.

## 2. Scope confirmation

Approved specification: #981 original plus Amendments 1-6, Amendments 7A, 7B
and 7C, and the exact #985 body bound by CTO approval `6296026365`.

Implemented objective: restore the unsuppressed natural Clippy command by
pinning `diesel_derives` 2.3.10 in `Cargo.lock` (exactly two lines), record the
one-time CI correction in the four Thoth-specific post-A2 programme-flow
paragraphs, add one `CHANGELOG.md` `### Fixed` entry, and add this report.

Out-of-scope changes made: NONE.

## 3. Commits

One commit on top of M, carrying the complete bounded change:

- `MET-WP11-B-CLIPPY-01: restore unsuppressed Clippy with diesel_derives 2.3.10 (#985)`

Its SHA is the draft pull request head (see section 1).

## 4. Files changed

Authorized write paths (from the task specification):

- `Cargo.lock` (the `diesel_derives` stanza `version` and `checksum` lines only)
- `docs/engineering/ai-delivery/branching-and-release-workflow.md`
- `docs/engineering/repository-map/branch-topology.md`
- `docs/metrics/README.md`
- `docs/metrics/task-status.md`
- `CHANGELOG.md`

Authorized new-file paths:

- `docs/engineering/ai-delivery/implementation-reports/MET-WP11-B-CLIPPY-01-implementation-report.md`

Actual files changed:

- `Cargo.lock` (+2/-2)
  - reason: pin `diesel_derives` 2.3.10 with its crates.io checksum.
  - behavioural effect: `QueryableByName` expansions use shorthand named-field
    initializers; no change to the expanded types, fields, SQL types or
    deserialization order.
  - within authorized write budget: YES
- `docs/engineering/ai-delivery/branching-and-release-workflow.md` (+1/-1),
  `docs/engineering/repository-map/branch-topology.md` (+11/-1),
  `docs/metrics/README.md` (+11/-1), `docs/metrics/task-status.md` (+12/-2)
  - reason: each file's existing Thoth-specific post-A2 programme-flow sentence
    now keeps the prohibition on normal post-A2 `feature/metrics--<slice>`
    branches, names the separately authorized develop-refresh recovery and the
    single one-time CI-correction slice `feature/metrics--wp11-b-clippy` under
    `thoth-pub/thoth#981 / thoth-pub/thoth#985` as the only exceptions (neither
    a standing authorization; the CI correction is not a develop refresh and
    imports no `develop` commit), requires the exact-list A2 inventory,
    independent review, retirement authorization, guarded retirement and
    completion receipt to be repeated against the resulting `feature/metrics`
    head after that slice merges and before B proceeds, and preserves B's own
    independent CRITICAL review and explicit CTO merge authorization. No
    comment ID, PR status or completion claim is committed; the wording holds
    before and after B. No other paragraph or table changed.
  - behavioural effect: documentation only.
  - within authorized write budget: YES
- `CHANGELOG.md` (+1/-0)
  - reason: one entry at the top of the existing `[Unreleased]` / `### Fixed`
    heading referencing issue #985; no heading added.
  - behavioural effect: none.
  - within authorized write budget: YES

Actual new files created:

- `docs/engineering/ai-delivery/implementation-reports/MET-WP11-B-CLIPPY-01-implementation-report.md` - within authorized new-file list: YES

Files deleted, moved or renamed: NONE

### 4.1 Write-budget compliance

PASS. `git diff --name-status <M> HEAD` lists exactly the six modified paths
above and the one added report; `git diff --name-only <M> HEAD -- '*.rs'` is
empty; `Cargo.toml` files, `thoth-api/migrations`, `thoth-api/src/schema.rs`
and `.github` are unchanged.

The exact `Cargo.lock` hunk:

```diff
@@ -1664,9 +1664,9 @@ dependencies = [

 [[package]]
 name = "diesel_derives"
-version = "2.3.9"
+version = "2.3.10"
 source = "registry+https://github.com/rust-lang/crates.io-index"
-checksum = "d1817b7f4279b947fc4cafddec12b0e5f8727141706561ce3ac94a60bddd1cf5"
+checksum = "ecbd51fb6c020672543641167efa4e6417ff7ad76849ed556ace3595e72de03a"
 dependencies = [
  "diesel_table_macro_syntax",
  "dsl_auto_type",
```

### 4.2 Authorized actions actually used

- repository inspection: YES
- source edit: YES, within the write budget
- new file creation: YES, this report only
- file deletion/move/rename: NO
- branch creation: YES, `feature/metrics--wp11-b-clippy` from exact M only
- commit: YES, one commit
- push: YES, the task branch only
- PR creation/update: YES, one draft PR targeting `feature/metrics`; not marked
  ready for review
- issue/comment mutation: NO
- manual CI dispatch/rerun: NO
- provider/runtime read: NO (crates.io index and runner-image manifest are
  public package/documentation reads)
- provider/runtime write: NO
- migration execution: NO non-disposable migration; the test suite applied the
  embedded migrations to the disposable local `thoth_test` database only
- release/tag/publication: NO manual publication; see section 4.3
- merge: NO
- deployment: NO
- production activation: NO
- other: a diagnostic `cargo update -p diesel_derives --precise 2.3.10` was run
  in the worktree and fully discarded (section 5)

Unauthorized actions performed: NONE

### 4.3 Automatic and manual external effects

Automatic CI/provider effects: opening the draft PR triggers the bare
`pull_request` workflows `build-test-and-check`, `check-changelog`,
`run-migrations` and `publish-to-dockerhub`. For this path set the classifier
(run locally from M's script with `--paths`) returns `docs_only=false`,
`run_build=true`, `run_migrations=false`, `run_docker=true`, so the build, test,
lint and format jobs run, the migration job is skipped, and the staging job
builds and pushes `ghcr.io/thoth-pub/thoth:staging-pr-<child PR number>` with
`THOTH_EXPORT_API=https://export.test.thoth.pub`. That GHCR push is a staging
image only, not a release, migration or deployment. Run IDs, conclusions and the
observed image digest are recorded on the pull request's checks and in the
implementation handoff, not here.

Manually initiated external actions: NONE

External writes/publication: the automatic staging image above only.

## 5. Implementation decisions

1. The two lockfile lines were edited directly. The diagnostic
   `cargo update -p diesel_derives --precise 2.3.10` produced the derive change
   plus the prohibited collateral `prost-build` edge `heck 0.5.0` -> `heck 0.4.1`;
   that output was discarded with `git checkout -- Cargo.lock` before the direct
   edit.
2. The locked graph was proven by `cargo metadata --locked` and by every build,
   lint and test command running with `--locked` without rewriting
   `Cargo.lock` (SHA-256 unchanged before and after each step).
3. The SDL baseline and candidate were generated in the same worktree, target
   directory, toolchain and environment: first at pristine M, then after the
   lockfile edit, deleting the generated asset before each build so a stale file
   could not be mistaken for a regenerated one.
4. The four doctrine paragraphs use the same sentence, wrapped to each file's
   existing style, so the exception cannot be read differently in different
   places.
5. One commit and one push, so the PR runs one natural CI round and publishes
   one staging image; this report records CI as pending rather than chasing it
   with a report-only commit.

Deviations from the specification requiring authorization: NONE.

## 6. Database and migration effects

Migration added: NO. No SQL, migration directory, `thoth-api/src/schema.rs` or
data change. The 20 Metrics v1.9.0 migration pairs and all earlier migrations
are byte-identical to M.

## 7. API and compatibility effects

GraphQL/API changes: NONE. The workspace-generated
`thoth-client/assets/schema.graphql` is byte-identical at M and at the candidate
(section 9).
Generated schema/client updates: NONE committed; the generated SDL is ignored
by `thoth-client/.gitignore` and stays untracked.
Backwards compatibility: unchanged; the change is a compile-time spelling of the
same initializer.
Deprecations: NONE.
Cross-repository dependencies: NONE. The SDL hash is unchanged, so pinned
Sphinx and dashboard operations and the v1.8 baseline are unaffected; no
downstream repository is modified.

`develop` and `master` keep `diesel_derives` 2.3.9 and remain potentially
exposed to the same Clippy diagnostic wherever affected derives compile; this
fix reaches `develop` only through the separately authorized #984 true merge.

## 8. Authorization and security

Authorization paths changed: NONE (`thoth-api/src/policy.rs` and every policy,
role and service-role check are byte-identical to M).
Roles/scopes involved: none.
Negative authorization tests: the existing protected-surface and service-role
negative tests run unchanged in the full suite (section 9).
Secret or personal-data handling: none. No credential, `.env`, provider or
production system was read. Local services were disposable and bound to
localhost.
Security limitations: none introduced.

## 9. Tests and checks

Environment: macOS `aarch64-apple-darwin`; rustc `1.99.0 (b940084d7 2026-09-28)`,
cargo `1.99.0 (5f94df478 2026-08-27)`, clippy `0.1.99 (b940084d7e 2026-09-28)`,
rustfmt `1.10.0-stable (b940084d7e 2026-09-28)`, selected with
`RUSTUP_TOOLCHAIN=1.99.0`; disposable PostgreSQL `17.10` (Homebrew), database
`thoth_test` with `server_encoding` `UTF8`, `LC_COLLATE`/`LC_CTYPE` `C`
(`char_length('é') = 1`), TCP on localhost; disposable Redis `8.10.0` on
localhost; libpq client `18.4`. CI-equivalent environment variables:
`THOTH_EXPORT_API=https://export.thoth.pub`, `TEST_DATABASE_URL` and
`TEST_REDIS_URL` pointing at the disposable services; no `.env` file exists in
the worktree or any parent directory.

### Formatting

Command:

```text
git diff --check
cargo fmt --all -- --check
```

Result:

```text
git diff --check                 exit 0, no output
cargo fmt --all -- --check       exit 0, no output
```

### Unit tests

Command:

```text
cargo test --workspace --locked
```

Result:

```text
exit 0, 671 s wall clock, Cargo.lock SHA-256 unchanged before and after

thoth (lib)                          0 passed; 0 failed; 0 ignored
thoth (bin src/bin/thoth.rs)        31 passed; 0 failed; 0 ignored
thoth-api (lib)                   2123 passed; 0 failed; 3 ignored
thoth-api tests/graphql_permissions 13 passed; 0 failed; 0 ignored
thoth-api-server (lib)               3 passed; 0 failed; 0 ignored
thoth-client (lib)                   4 passed; 0 failed; 0 ignored
thoth-errors (lib)                  13 passed; 0 failed; 0 ignored
thoth-export-server (lib)          144 passed; 0 failed; 0 ignored
doc-tests (all six crates)           8 passed; 0 failed; 8 ignored

total                             2339 passed; 0 failed; 11 ignored
```

### Integration/database tests

Included in the workspace run above against the disposable UTF-8 PostgreSQL 17
database and disposable Redis; no test was skipped beyond the repository's
existing `#[ignore]` tests.

### Lint/static analysis

Command:

```text
cargo clippy --all --all-targets --all-features --locked -- -D warnings
```

Result:

```text
candidate: exit 0; zero rustc or Clippy diagnostics; all six workspace crates
           linted; the only warning is Cargo's future-incompatibility
           advisory for the dependency proc-macro-error2 v2.0.1 (not a lint;
           identical at M)
```

No `allow` attribute, lint configuration or suppression exists in the diff.

Same-toolchain negative control: the identical command with M's `Cargo.lock`
temporarily restored in the same worktree exits 101 with 347 rendered / 346
unique `redundant field names in struct initialization` errors, and the set of
346 file/line/column positions is identical to the natural CI lint log at M.
The candidate `Cargo.lock` was then restored (SHA-256 re-verified).

### Other required checks

Dependency resolution:

```text
cargo metadata --locked --format-version 1        exit 0, Cargo.lock unchanged
cargo check --workspace --locked                   exit 0 at M and at the candidate, Cargo.lock unchanged
compiled derive                                    diesel_derives v2.3.9 at M, v2.3.10 at the candidate
resolved packages, M vs candidate                  564 / 564
packages only at M                                 diesel_derives 2.3.9
packages only in candidate                         diesel_derives 2.3.10
other packages with changed edges or features      diesel 2.3.10 (its diesel_derives edge only)
diesel_derives features and dependencies           identical in both versions
prost-build heck edge                              heck 0.5.0 in both (no downgrade)
Cargo.lock SHA-256 at M                            7b28f9b4e04fdfdc3c7e693eb946bb5ac027c85f783316975160d1953456f4e6
Cargo.lock SHA-256 candidate                       1a48d28b6424e7941de5140ace3a0f7700c1ff318a6ba34e4895ae75a250a79b
```

GraphQL SDL reproduction (`thoth-client/assets/schema.graphql`, written by
`thoth-client/build.rs` during a workspace-level build):

```text
pinned approved SDL SHA-256        2681b54ac18296f886451d08f5396c0a9ef8351762f63f7b3f80f319ae505496
exact M, cargo check --workspace   2681b54ac18296f886451d08f5396c0a9ef8351762f63f7b3f80f319ae505496  (239574 bytes)
candidate, cargo check --workspace 2681b54ac18296f886451d08f5396c0a9ef8351762f63f7b3f80f319ae505496  (239574 bytes)
candidate, cargo clippy build      2681b54ac18296f886451d08f5396c0a9ef8351762f63f7b3f80f319ae505496
candidate, cargo test build        2681b54ac18296f886451d08f5396c0a9ef8351762f63f7b3f80f319ae505496
cmp M candidate                    byte-identical
git status                         asset ignored by thoth-client/.gitignore; untracked; not committed
```

The asset was deleted before every build, so each hash is of a freshly
regenerated file.

Rust source identity: all 36 inventoried paths from #985 have the same Git blob
in the index, in the working tree and at M:

```text
2799ddcdf15e182b8885be3d7298aa053e7a21ed  thoth-api/src/graphql/distribution_job_tests.rs
d3b21f9e8202f40b0ba5bf418cbbe3b73e4930b5  thoth-api/src/model/distribution_job/crud.rs
42fc83ff135ddb59d9c25e1389fa22d9eec7baa9  thoth-api/src/model/distribution_job/mod.rs
f1ad4cb9d5e75df67e6bcc94b9ce01453be7d116  thoth-api/src/model/distribution_job/tests.rs
21c32cff634d416ad658458c834c69a6186aa687  thoth-api/src/model/metric_coverage/tests.rs
3ccc7cf59cc5df63bcef9900cfc5951b4325b248  thoth-api/src/model/metric_coverage_run/crud.rs
057af5dfa83ee61ea164efb4c7c940c7e122e4cb  thoth-api/src/model/metric_coverage_run/tests.rs
0001e8f20f797512116031a7cd1148851f197c11  thoth-api/src/model/metric_coverage_run/verification.rs
0a4e4db476b61aef2549fff1d7b99d0b59aa54c6  thoth-api/src/model/metric_dashboard/mod.rs
d297f5db3cf5697655cd3640aaf7edb01afd75d0  thoth-api/src/model/metric_dashboard/tests.rs
189ddaf47b7d0f9f87e755f5d617a5f728ca2cc0  thoth-api/src/model/metric_identifier_quarantine/tests.rs
bf047015446135235a07873bdff0028120b98802  thoth-api/src/model/metric_identifier_quarantine_reconciliation/mod.rs
835c8f3f9ada146a565a4ea85ad7714dfcdf6e40  thoth-api/src/model/metric_identifier_quarantine_reconciliation/tests.rs
9df235b56f71e4c1b1d9f7da1dc4da0538e1afa3  thoth-api/src/model/metric_import/tests.rs
b2201460f0b1eae92eb922b1108cb93fd8ea28c8  thoth-api/src/model/metric_import_error/tests.rs
9c44d13158bd178b1b2b7811d0b7c6d182116d69  thoth-api/src/model/metric_ingestion/tests.rs
46280d7f03557f226a8b3b0ea9e3d5b88d290276  thoth-api/src/model/metric_ingestion_lifecycle/mod.rs
e89d62b8e5ad2b2454a630d67a64f463ff72c2ab  thoth-api/src/model/metric_measure/tests.rs
cde9327132193225ca6915c3ad5235541d352367  thoth-api/src/model/metric_operas_export/tests.rs
e694b732bfe0311d6302d644f3271d6b74240333  thoth-api/src/model/metric_operas_import/tests.rs
b6d799e149e5f474e6598c0ba89d99ed5bd5f735  thoth-api/src/model/metric_operas_mapping/tests.rs
f63f9a1b8e8a20afc10bde01b67752d0d35ce3af  thoth-api/src/model/metric_platform/tests.rs
972bcd028718e58338582d9ac972ce7831d5b595  thoth-api/src/model/metric_publisher_platform_approval/tests.rs
dd4b0bb50430669cd99f6f248af08f2f962d4419  thoth-api/src/model/metric_reconciliation_issue/tests.rs
0c4e94f7975905e59ed87554a6cc9a3001b4e019  thoth-api/src/model/metric_reconciliation_run/tests.rs
696d5c05f14c61bb7a23c7138c98f61cfe192723  thoth-api/src/model/metric_record/tests.rs
bd4822d792e6a8d7f87e073e4826e19a9f1b258e  thoth-api/src/model/metric_record_revision/tests.rs
ffe30293f1ead14fc92bf86e9bcfb16609d439ad  thoth-api/src/model/metric_rollup_delta/crud.rs
f72f231aa5dec39a2f50b172dcddeb266eb86494  thoth-api/src/model/metric_rollup_delta/tests.rs
bb4f32b37a5dd62fde9b034370166d34a8724d28  thoth-api/src/model/metric_rollup_delta/verification.rs
3d8844917f36d108dec9877247d3ecf1c49e2c52  thoth-api/src/model/metric_source/tests.rs
d7446598b05b2e346bf02fb8b0489bef28e5c6f6  thoth-api/src/model/metric_source_registry_history/tests.rs
a5e7cc5e969991ea11ea93844661326dd8f2183a  thoth-api/src/model/publisher_distribution_platform/tests.rs
b37aef86ad9b082d8b56102077626e36d56aab29  thoth-api/src/model/publisher_service_configuration/migration_backfill.rs
5201ddb3b593fad3baa6ddbf362dfe272a1aa21d  thoth-api/src/model/publisher_service_configuration/tests.rs
6976f21b1149dcfd3e2cc0a49b577a2f8f75f086  thoth-api/src/model/work_relation/tests.rs

36/36 MATCH (Git blob at M == index == working tree)
```

CI classifier (M's `.github/scripts/classify_ci_changes.py --paths <the seven changed paths>`):

```text
{"docs_only": "false", "run_build": "true", "run_docker": "true", "run_migrations": "false"}
```

## 10. Manual verification

Environment: the disposable local environment in section 9.
Steps: diffed the `diesel_derives` 2.3.9 and 2.3.10 crate sources (section 1.3);
compared the resolved dependency graphs and SDL outputs of M and the candidate;
reviewed the complete candidate diff against M before committing.
Observed result: as recorded in sections 1.3, 4.1 and 9.
Evidence link/screenshot/log reference: local logs retained by the implementing
agent; summarized in the implementation handoff.

## 11. CI

CI status: PENDING when this report was committed. The natural `pull_request`
runs for the exact PR head, the observed runner image and its published Rust
toolchain manifest, and the GHCR staging digest are GitHub records and are
reported in the implementation handoff; the natural lint job runs the unlocked
`cargo clippy --all --all-targets --all-features -- -D warnings` on the
runner's preinstalled stable toolchain, which the workflow does not pin.

## 12. Rollout and rollback

Initial state after merge: only the `feature/metrics` programme integration
branch carries the pin; no deployment, data change or Metrics activation.
Activation required: NO.
Feature flag/configuration: unchanged.
Migration sequence: none.
Rollback/disable procedure: before the child merge, leave `feature/metrics`
unchanged and abandon the branch under separate ref/PR authority; after the
merge, only independently reviewed forward correction under new authority, with
no reset, rebase or force-push of the shared integration branch.
Monitoring required: none beyond the natural CI of this PR and of #984.

## 13. Known limitations and deferred work

- Local validation ran on macOS `aarch64` with the reference toolchain; natural
  CI runs on GitHub-hosted `ubuntu-latest` with an unpinned stable toolchain.
  Local results do not replace natural CI; a changed runner toolchain or a new
  lint class is a HOLD, not a waiver.
- `develop` and `master` keep `diesel_derives` 2.3.9 (section 7); any
  `master`-side correction is a separate STANDARD task.
- Cargo's future-incompatibility advisory for the dependency `proc-macro-error2`
  is not a lint and is out of scope.

## 14. Unresolved issues

- NONE within this task's scope. The remaining gates are separately authorized:
  independent cross-model HIGH exact-head source review; the child PR's
  draft-to-ready transition; the child TRUE MERGE COMMIT into `feature/metrics`
  (including the resulting #984 synchronization, CI and `staging-pr-984`
  replacement); renewed exact-list A2 bound to the post-merge `feature/metrics`
  head; #984 rebinding, fresh cumulative CRITICAL review and its own merge
  authorization; and every later release, deployment and activation gate.

## 15. Agent self-assessment

The agent may identify risks but may not approve the task.

Suggested review focus:

- that the `Cargo.lock` diff is exactly the two approved lines and the locked
  graph differs from M only in `diesel_derives`;
- that the four doctrine sentences are durable, cite only
  `thoth-pub/thoth#981 / thoth-pub/thoth#985`, keep the prohibition on normal
  post-A2 slices and the develop-refresh distinction, and require renewed A2
  before B;
- the natural CI lint job at the exact head, its runner image and toolchain;
- the two-sided SDL hash evidence and the 36-path identity inventory.
