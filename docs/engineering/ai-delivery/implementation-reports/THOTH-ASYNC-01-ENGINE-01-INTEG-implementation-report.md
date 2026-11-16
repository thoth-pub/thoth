# THOTH-ASYNC-01-ENGINE-01-INTEG Implementation Report

G6 of `THOTH-ASYNC-01-ENGINE-01`: the integration of `feature/worker`, which holds the inert shared job engine of E1 (`THOTH-ASYNC-01-ENGINE-01A`) and E2 (`THOTH-ASYNC-01-ENGINE-01B`), into `develop`. The task branch merges `develop` into `feature/worker`, renames the engine migration's version suffix, and re-runs the spec 19.0 migration evidence on the integrated chain. Risk CRITICAL. One HOLD was raised and then released by an amendment (section 0). Section 16 records the corrections to the E1 and E2 reports.

## 0. HOLD 6922872012 and amendment GRANT 6924016558

The first run resolved the two expected conflicts (section 5). It then stopped without committing anything, because the merged tree did not compile:

```text
error[E0004]: non-exhaustive patterns: `policy::Role::RorSyncService` not covered
  --> thoth-api/src/model/async_job/executor.rs:53:11
```

- E2's `executor_profile` matches `Role` exhaustively on purpose. Its doc comment says: "The match names every variant, so a new role must be placed here explicitly."
- `develop` gained `Role::RorSyncService` (`thoth-api/src/policy.rs:89`) with ROR-SYNC-01C (#1028), after `feature/worker`'s base.
- The fix changes engine code, which handoff sections 5 and 9 make a HOLD. The HOLD was reported (6922872012).

The CTO chose "Allow the fix" on 2026-11-16 at 17:33Z (GRANT 6924016558 on #1026, handoff section 12). The amendment added two edits to the write budget, and nothing else:

1. `thoth-api/src/model/async_job/executor.rs`: `| Role::RorSyncService` in the `None` arm of `executor_profile`. It is inside the merge commit, so that the merge commit compiles. `git show --remerge-diff c7bc361a -- thoth-api/src/model/async_job/executor.rs` shows that arm only (+2 -1).
2. `thoth-api/src/graphql/async_job_tests.rs`: `ROR_SYNC_SERVICE` in the authorization test role lists. This is test-only and has its own commit, `3be5f83b` (section 5, decision 4).

Effect: `ROR_SYNC_SERVICE` maps to no executor profile, like every role except `DISSEMINATION_WORKER`. Every executor operation refuses it, and operator commands and reads stay superuser-only. No other role's behaviour changes.

The FBF-4 guard tightening (E2 review carry item 3) stays carried to step 4, as the amendment says, even though this task changes `async_job_tests.rs`.

## 1. Repository state

Owning GitHub issue: thoth-pub/thoth#1026 (programme thoth-pub/thoth#957)
Repository: thoth-pub/thoth
Workflow: PROGRAMME_INTEGRATION
Base branch: `feature/worker`
Authorized base commit: `59e967e327695e29225e09ac94bd92fa09cfc0d5` (E2's merge commit, #1037)
Actual base commit: `59e967e327695e29225e09ac94bd92fa09cfc0d5`
Merged in: `develop` at `bd69aaafbca3aaa4d382f8cfbbd00c3c7f749290` (#1027), its tip at the start. The merge is history-preserving (`--no-ff`); nothing was rebased.
PR target: `develop`
Programme integration branch: `feature/worker`
Task branch: `feature/worker--integrate-01`, created with `git switch -c feature/worker--integrate-01 origin/feature/worker`. That command set `origin/feature/worker` as its upstream, and `git branch --unset-upstream` removed it, so a bare `git push` cannot reach `feature/worker`.
Head commit: the code head is `60aff21965364a731f3a90d4415a1f707dd53642`. This report and the two pointer lines are in the commit that follows it.
Pull request: the orchestrator opens the draft PR into `develop` after its push; GitHub holds its record.
Expected branch deletion after merge: YES
Final programme PR required: NO. This branch is the integration of `feature/worker` into `develop` for this stage (G6).
Implementing model: Claude Opus 5.5 (the run that raised the HOLD and the run that continued after its release; several context compactions, no work lost)
Reasoning level: max

Preflight (handoff section 4):

1. `git fetch origin feature/worker develop`: `origin/feature/worker` was `59e967e3` and `origin/develop` was `bd69aaaf`. Rechecked at 18:33Z, before the report commit: the same fetch, with `--no-tags`, returned the same two commits, and the command of item 2 again printed nothing.
2. `git ls-remote origin 'refs/heads/feature/worker--integrate-01'` printed nothing.
3. Identity: `user.name` is `Javier Arias`, `user.email` is `javier@jarias.org`, `commit.gpgsign` is `false`.
4. Read the root `AGENTS.md` and `docs/engineering/AGENTS.md`.
5. Migration order on `develop` at `bd69aaaf`: 31 directories, newest `20261113_v1.10.0` (ROR-SYNC C). None is dated `20261115` or later. ROR-SYNC B's reserved `20261114` is not on `develop`.
6. The specification's SHA-256 is `75e38f4b83697cdb860e4bb01bffc64dc98297127778afa952e47da0fe021baf`, as the handoff requires. ADR-0012 is blob `892cbcad50c6dbf9bd943590a3bd7027b78e0341` and ADR-0015 is blob `55c38c2e0b4856562e42a19e7eff4d865be0fc39`, the same on `develop` and at the head.

## 2. Scope confirmation

Approved specification: `ENGINE-01-spec-R5.5.md` (SHA-256 `75e38f4b...`), sections 3, 7, 19.0, 21 and 22, through the G6 handoff (GRANT 6922457134) and its section 12 amendment (GRANT 6924016558).

Implemented objective:

- `develop` at `bd69aaaf` merged into `feature/worker` at `59e967e3`, with the two conflicts resolved (section 5) and the amendment's arm (section 0);
- the amendment's test role lists;
- the engine migration renamed from `20261115_v1.10.0` to `20261115_v1.11.0` (section 6);
- the spec 19.0 evidence re-run on the integrated chain, with the previous image now v1.10.0 (section 6.1);
- the full suite and every migration revert guard on the integrated chain (section 9);
- the ADR re-binding (section 9.2) and the corrections to the E1 and E2 reports (section 16).

Carries handled:

- `E2-carry-from-E1.md` item 6 (OPE-1, G6): the revert guards and spec 19.0 item 2 on the integrated chain (sections 6.1 and 9.1), the `schema.rs` and `CHANGELOG.md` conflicts (section 5), and the rename (section 6).
- `E2-review-carry.md` item 5 (OPF-2, OPF-4, FBF-6): section 16.
- `E2-review-carry.md` item 7 (FBC-3), the four Release N duties: section 9.2.
- `E2-review-carry.md` item 8 (FBC-6), the migration name and the previous-image evidence: section 6, section 6.1 item 6 and section 16.4.

Out-of-scope changes made: NONE. The two engine edits are the amendment's (section 0).

## 3. Commits

- `c7bc361a6ea396bc842f81c8ff4d7d38b3a2a251` - THOTH-ASYNC-01-ENGINE-01-INTEG: Merge develop into the branch (parents `59e967e3` and `bd69aaaf`)
- `3be5f83b95c4cc18f70e67c9424247119dabe200` - THOTH-ASYNC-01-ENGINE-01-INTEG: Add ROR_SYNC_SERVICE to the async job role tests
- `60aff21965364a731f3a90d4415a1f707dd53642` - THOTH-ASYNC-01-ENGINE-01-INTEG: Rename the engine migration to 20261115_v1.11.0
- the commit that follows: THOTH-ASYNC-01-ENGINE-01-INTEG: Add the integration report and the correction pointers

Every commit is authored and committed by `Javier Arias <javier@jarias.org>`, unsigned, with no trailer. `git log --first-parent --format='%h %an <%ae> | %cn <%ce>' origin/feature/worker..HEAD` prints that pair for each of the four. The 37 `develop` commits that the merge brings in keep their own authors.

The merge commit compiles on its own: `cargo check --workspace --all-targets --all-features` passed on the merged tree with the arm before `3be5f83b` was made. The evidence of sections 6 and 9 was taken at the code head `60aff219`.

## 4. Files changed

Authorized write paths (handoff sections 5 and 12):

- `CHANGELOG.md` and `thoth-api/src/schema.rs`: the merge's conflict resolutions;
- `thoth-api/migrations/20261115_v1.10.0/` to `thoth-api/migrations/20261115_v1.11.0/`, and every path that names the directory;
- test-only fixes to existing migration revert guards, only if one fails because the engine migration is the newest;
- `thoth-api/src/model/async_job/executor.rs`: one arm (amendment);
- `thoth-api/src/graphql/async_job_tests.rs`: the role lists (amendment);
- the E1 and E2 reports: one pointer line each.

Authorized new-file paths:

- `docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ENGINE-01-INTEG-implementation-report.md`

Actual files changed by this task: the merge's own resolutions (`git show --remerge-diff c7bc361a`) and the three commits after it.

- `CHANGELOG.md`
  - reason: the merge conflict (section 5, decision 1)
  - behavioural effect: none
  - within authorized write budget: YES
- `thoth-api/src/schema.rs`
  - reason: the merge conflict (section 5, decision 2)
  - behavioural effect: none; it declares both sides' SQL types
  - within authorized write budget: YES
- `thoth-api/src/model/async_job/executor.rs`
  - reason: amendment (a), so the merge compiles (section 0)
  - behavioural effect: `executor_profile(Role::RorSyncService)` is `None`
  - within authorized write budget: YES (GRANT 6924016558)
- `thoth-api/src/graphql/async_job_tests.rs`
  - reason: amendment (b) (section 5, decision 4)
  - behavioural effect: none; test-only
  - within authorized write budget: YES (GRANT 6924016558)
- `thoth-api/migrations/20261115_v1.10.0/up.sql` and `down.sql`, renamed to `thoth-api/migrations/20261115_v1.11.0/`
  - reason: the rename (section 6)
  - behavioural effect: none; the Diesel version stays `20261115`
  - within authorized write budget: YES
- `docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ENGINE-01A-implementation-report.md` and `...-ENGINE-01B-implementation-report.md`
  - reason: the pointer line under each title (section 16)
  - behavioural effect: none
  - within authorized write budget: YES

Actual new files created:

- `docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ENGINE-01-INTEG-implementation-report.md` - within authorized new-file list: YES

Everything else that the merge commit changes relative to its first parent is `develop`'s own content: 37 commits since the merge base `da3e15a3` (#1022), merged without conflict. Relative to `develop`, the code head adds E1's and E2's content: 29 files, 28434 lines added and 17 removed.

Files deleted, moved or renamed: the migration directory rename only (handoff section 5 item 2). No repository file was deleted. Build output:

- test executables under `target/debug/deps` were deleted to keep free disk above 3 GB (handoff section 10's exception);
- after the v1.10.0 build, `cargo clean -p` removed the build output of the six workspace crates, which that build had made stale (reading R7). Third-party build output was kept.

### 4.1 Write-budget compliance

PASS.

- `git show --remerge-diff --stat c7bc361a` lists `CHANGELOG.md` (9 lines: +3 -6), `thoth-api/src/model/async_job/executor.rs` (+2 -1) and `thoth-api/src/schema.rs` (+4 -5), and nothing else.
- `3be5f83b` changes `thoth-api/src/graphql/async_job_tests.rs` only (+10 -1).
- `60aff219` renames the two migration files and changes no line.
- The report commit adds this file and one pointer line (with its blank line) to each of the E1 and E2 reports.
- Relative to `develop`, the branch changes no `Cargo.toml`, `Cargo.lock`, `.github/**` or `docs/engineering/decisions/**` path: `git diff --stat origin/develop HEAD -- Cargo.toml Cargo.lock '*/Cargo.toml' .github docs/engineering/decisions` is empty.
- No new dependency.
- BE-04: section 5, reading R4, and section 9.

### 4.2 Authorized actions actually used

- repository inspection: YES
- source edit: YES (the two conflict resolutions and the two amendment edits)
- new file creation: YES (this report)
- file deletion/move/rename: YES, the migration directory rename only
- branch creation: YES, local `feature/worker--integrate-01`
- commit: YES, four commits
- push: NO
- PR creation/update: NO
- issue/comment mutation: NO
- manual CI dispatch/rerun: NO
- provider/runtime read: NO (local PostgreSQL 17.10 and Redis only)
- provider/runtime write: NO
- migration execution: YES, local only: the disposable `g6_*` databases (section 6.1) and the suite's `thoth_test`
- release/tag/publication: NO. The handoff's `git fetch origin tag v1.10.0` also fetched 162 other tags through git's automatic tag following, so it created 163 local tag refs in a clone that had none. All 163 were deleted locally with `git tag -d` after the v1.10.0 worktree was removed, and none was ever pushed. (E1 avoided this with `git fetch --no-tags`.)
- merge: YES, the local merge of `develop` into the task branch (handoff section 5 item 1); no PR was merged
- deployment: NO
- production activation: NO
- other:
  - a scratch worktree of tag `v1.10.0` at `/home/claude/thoth-v110`, created and then removed with `git worktree remove` (section 6.1 item 6);
  - a local identity stand-in on `127.0.0.1` with a throwaway RSA key, generated for that run, never printed and deleted afterwards (section 8);
  - a temporary Rust 1.100.0 toolchain for one clippy run, installed and then removed (section 9);
  - `cargo clean -p` of the six workspace crates after the v1.10.0 build, and a rebuild at the head (reading R7).

Unauthorized actions performed: NONE.

### 4.3 Automatic and manual external effects

Automatic CI/provider effects observed: none. Nothing was pushed in this task; CI starts with the orchestrator's push and draft PR.

Manually initiated external actions: NONE.

External writes/publication: NONE. External reads only: `git fetch` and `git ls-remote` against `origin`, and the toolchain download by `rustup`.

## 5. Implementation decisions

1. **`CHANGELOG.md`.** The conflict came from `develop`'s release of v1.10.0, which moved the old `## [Unreleased]` lines into the v1.10.0 section, while `feature/worker` had added E1's and E2's lines to the old list. The resolution is `develop`'s file byte for byte, plus one engine line under `## [Unreleased]`:

   ```text
   ## [Unreleased]
   ### Added
     - [1026](https://github.com/thoth-pub/thoth/issues/1026) - Add the shared asynchronous job engine (its tables and core, and a protected GraphQL API with executor operations for `DISSEMINATION_WORKER` and superuser commands and reads), inert until a job kind is registered, and leave async job request bodies out of the access log

   ### Changed
     - [1026](https://github.com/thoth-pub/thoth/issues/1026) - Record ADR-0015, ...
   ```

   - The `### Changed` line is `develop`'s ADR-0015 line, unchanged.
   - The engine line replaces E1's and E2's two lines (spec 22.4: "G6 may merge the two lines into one").
   - The third line on the `feature/worker` side of the conflict, the #1019 `metricWidget` line, is `develop`'s own older text. `develop` now carries it, reworded, in its v1.10.0 section, so keeping `develop`'s text drops nothing.
   - `git diff origin/develop HEAD -- CHANGELOG.md` is three added lines (the heading, the engine line and a blank line). Nothing goes under a released version.
2. **`thoth-api/src/schema.rs`.** Only the `sql_types` block conflicted: both sides had appended SQL types after `MetricSourceRegistryHistoryAction`. The resolution keeps `develop`'s `RorStatus` first, then the engine's nine types in E1's order:

   ```rust
       #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
       #[diesel(postgres_type(name = "ror_status"))]
       pub struct RorStatus;

       #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
       #[diesel(postgres_type(name = "async_job_status"))]
       pub struct AsyncJobStatus;
       // ... AsyncEffectPhase, AsyncJobEffectClass, AsyncJobRevisionMode,
       // AsyncJobLineage, AsyncAttemptPurpose, AsyncAttemptResult,
       // AsyncControlCommandKind, AsyncControlCommandOutcome, as on feature/worker
   ```

   - `git diff origin/develop HEAD -- thoth-api/src/schema.rs`: 243 lines added, none removed. That is E1's `schema.rs` change.
   - `git diff origin/feature/worker HEAD -- thoth-api/src/schema.rs`: 73 lines added, none removed. That is `develop`'s change.
   - The result is the exact union: no entry is dropped or renamed. Every `table!`, `joinable!` and `allow_tables_to_appear_in_same_query!` entry of both sides is present, including `develop`'s `metric_rollup_month_rebuild_run`.
3. **`executor_profile`** (amendment (a)). The `None` arm, as merged:

   ```rust
           Role::Superuser
           | Role::PublisherAdmin
           | Role::PublisherUser
           | Role::WorkLifecycle
           | Role::CdnWrite
           | Role::MetricsIngestService
           | Role::MetricsReadService
           | Role::RorSyncService => None,
   ```

4. **The test role lists** (amendment (b), `3be5f83b`). `ROR_SYNC_SERVICE` joins every list in `thoth-api/src/graphql/async_job_tests.rs` that enumerates roles:
   - `outsiders()`: a new caller `Caller::new("ROR_SYNC_SERVICE", "machine:ror-sync", &[Role::RorSyncService])`, and the doc comment names it;
   - `refused_executors()`: the "`SUPERUSER` with every role but `DISSEMINATION_WORKER`" caller;
   - `refused_operators()`: the "`DISSEMINATION_WORKER` with every role but `SUPERUSER`" caller;
   - the `roles` array of `the_test_registries_validate_and_only_dissemination_worker_maps_to_an_executor`, which now checks that `ROR_SYNC_SERVICE` maps to no executor.

   `has_unscoped_role` (`thoth-api/src/policy.rs:111-116`) checks only that the role key is present, so each test user really holds the role. No test asserts the length of these lists.
5. **The rename.** `git mv thoth-api/migrations/20261115_v1.10.0 thoth-api/migrations/20261115_v1.11.0` (section 6).
6. **No revert-guard fix.** Every migration revert guard passes unchanged on the integrated chain (section 9.1). The three guards that the handoff names already revert every newer migration first: OPS-03's `revert_migrations_newer_than_this_stage` (`rebuild_run_tests.rs:1118`, since #1032), ROR-SYNC C's `revert_newer_than_c` (`institution/tests.rs:3231`) and E1's `revert_migrations_newer_than_b1` (`metric_coverage_run/tests.rs:618`).

Readings:

- R1. **`PUBLISHER_ROLES` unchanged.** The amendment names `outsiders`, `refused_executors`, `refused_operators` "and any other list that enumerates roles". `PUBLISHER_ROLES` (`async_job_tests.rs:263`) lists the four publisher-scoped roles only, and the Metrics machine roles are not in it either. `ROR_SYNC_SERVICE` is an unscoped machine role (`policy.rs:80-89`), so it stays out of that list and joins the lists of decision 4.
- R2. **`### Added` under Unreleased.** `develop`'s Unreleased section had only `### Changed`. The engine line is an addition, so it gets an `### Added` heading, placed before `### Changed` as in every released section.
- R3. **SQL type order.** `develop`'s `RorStatus` keeps its place, and the engine's nine types follow it as one block in E1's order.
- R4. **The BE-04 diff.** Handoff section 6 item 7 expects `git diff origin/develop -- <every BE-04 path>` to be empty. It shows one file, `thoth-api/src/model/publisher_service_configuration/tests.rs`: 4 added lines, the four `async_job*` table names that E1 added to the `%job%` table-inventory assertion of `the_migrated_database_matches_the_schema_contract`. Spec 7.11 and 22.4 authorize exactly that assertion, and both E1 legs reviewed it. Under the handoff's rule ("the specification wins and you report the difference"), this is reported as a difference, not a BE-04 change. The same diff against `origin/feature/worker` is empty, so G6 changes no BE-04 path (section 9, other required checks).
- R5. **Test 10 and `create_job_on`.** Ascending idempotency-key order is a caller duty (section 9.2). The doc comment of `create_job_on` does not restate it. That is not a gap in Release N, which has no caller, but step 4's first creation path should state it (section 13).
- R6. **Evidence method under the disk floor.** Once every test executable of `cargo test --workspace --no-run` existed, free disk was 2.4 GB, below the handoff's 3 GB floor.
  - The suite therefore ran as three commands, each with `--workspace`: `--lib --bins`, `--doc`, and `--test graphql_permissions --test institution_ror_state`. Together they select the targets of CI's `cargo test --workspace`. The two integration-test executables were deleted before any run (handoff section 10's exception), and every other test executable was deleted after its run.
  - The first two commands ran the very executables that `cargo test --workspace --no-run` had built (the same file names), and compiled only non-test artifacts.
  - The third command unified dependency features differently from the full build. Before linking the two tests, it rebuilt `diesel`, `diesel_migrations`, `serde_with`, `openidconnect`, `zitadel`, the AWS SDK crates, `thoth-errors` and `thoth-api` from the same sources. The two tests passed in that build. In the full build they had compiled and linked without error but had not run. CI's `cargo test --workspace` runs them in the full build.
  - For the same reason, the 19.0 harness ran as `cargo test --workspace --lib -- <harness> --exact --ignored --nocapture`. That reuses the suite's `thoth-api` test executable and compiled nothing. E1 ran it as `cargo test -p thoth-api --features backend --lib`, which can unify features differently and rebuild. Both commands run the same test function.
- R7. **The shared target directory after the v1.10.0 build.** Cargo treats a workspace crate as up to date when no source file that its last build read is newer than that build. The handoff's method builds v1.10.0 into the head's target directory, under the artifact names that the head's builds use, and the head's source files are older than that build.
  - So a `cargo build` at the head right after it recompiled only `thoth-client`, `thoth-export-server` and `thoth`, and linked v1.10.0's `thoth-api`, `thoth-errors` and `thoth-api-server`. The binary embedded no engine migration, and `thoth-client`'s build script wrote `develop`'s SDL into the head's generated, git-ignored `schema.graphql`.
  - No evidence came from that build. Every head result in sections 6, 7 and 9 was taken before the v1.10.0 build, and item 6 used the v1.10.0 binary on purpose.
  - `cargo clean -p` on the six workspace crates (10.7 GiB of their build output) and a new `cargo build` at the head restored the head's artifacts. The rebuild compiled all six crates from the head's sources (exit 0). The binary embeds `20261115_v1.11.0` once and `20261115_v1.10.0` never, and the regenerated SDL is byte-identical to the head capture of section 7.
  - E1 used the same method for v1.9.0 (E1 report, section 6.1 item 6). Whoever repeats it should clean the workspace crates afterwards.

Deviations from the specification requiring authorization: the two engine edits of section 0, authorized by the CTO through GRANT 6924016558 (2026-11-16, 17:33Z). There is no other deviation.

## 6. Database and migration effects

Migration added: NO. E1's engine migration is renamed, not changed.

- migration files: `thoth-api/migrations/20261115_v1.11.0/up.sql` (blob `a78af3d6bee40ff914ffb9355c847380c0b317b4`) and `down.sql` (blob `4ba06ec14e081826d7cd38f43d21f8f5906bcb62`), byte-identical to E1's `20261115_v1.10.0` files. They carry no version text.
- version: Diesel's version is the directory name's leading `20261115`, so the rename changes no applied version. The directory has never been applied to a shared database. Its suffix is now the next unreleased version, v1.11.0 (spec 7), because v1.10.0 shipped without the engine (tag `v1.10.0` is `32af2a9df8068e485c6a9b1a5ef96fb3dac48668`).
- paths: no `.rs` file, `include_str!` path, test, Makefile or workflow names the directory. `git grep 20261115` finds only E1's report (13 historical lines, section 16.4). `embed_migrations!` reads the directory at compile time; the binary and the `thoth-api` test executable built at the head each embed `20261115_v1.11.0` once and `20261115_v1.10.0` never.
- order: 32 migrations on the integrated chain. `develop`'s newest, `20261113_v1.10.0`, is followed by `20261115_v1.11.0`, the newest.
- schema effect, existing-data effect, locking and downtime: unchanged from E1 (spec 7 and 7.11). The migration creates new objects only and touches no existing table.
- empty database result, populated database result, rollback and idempotency: section 6.1.

### 6.1 Spec 19.0 evidence on the integrated chain

Environment: PostgreSQL 17.10 on port 5433 and Redis on 6379, both local; code head `60aff219`. Items 1 to 3 and the `develop` reference each started from a new, empty `g6_*` database; items 4 and 5 continue item 1's, and item 6 continues item 2's. All four databases were dropped afterwards. Schema dumps are `pg_dump --schema-only --restrict-key=g6` and data dumps `pg_dump --data-only --restrict-key=g6`. `cargo run migrate` runs the head's `target/debug/thoth`. The harness is E1's ignored test `model::async_job::tests::evidence_migration_harness`, which applies migrations one at a time with `run_next_migration` (`ASYNC_EVIDENCE_STEP=apply-through:<version>`) or reverts them with `revert_last_migration` (`revert-through:<version>`).

Items 1 to 6 and the `develop` reference of item 2 PASS. "Engine objects" below means 9 `async_` tables, 9 `async_` enum types, 12 `async_` functions and 16 triggers on `async_` tables, as in E1's report.

**Item 1, fresh empty database, CI's `run-migrations` sequence** (`g6_fresh`; handoff item 1):

```text
cargo run migrate              exit 0   32 applied, newest 20261115, engine objects present
cargo run migrate --revert     exit 0   0 applied; public holds only __diesel_schema_migrations
cargo run migrate              exit 0   32 applied, newest 20261115, engine objects present
```

- The 32 versions, in order: `20250000 20260417 20260429 20260504 20260805 20260811 20260812 20260813 20260814 20260826 20260827 20260828 20260831 20260901 20260902 20260903 20260904 20260905 20260906 20260907 20260908 20260909 20260912 20260913 20260921 20260922 20260923 20260928 20261006 20261112 20261113 20261115`. The chain includes `20260814_v1.7.0` and `develop`'s `20261112` and `20261113`, and the engine migration is last.
- After the revert, `public` held no table but `__diesel_schema_migrations`, no enum, domain or composite type, and no function.
- The schema dump after the second migrate is byte-identical to the dump after the first (sha256 `5d74bed14e0ea91139ec7d68f4fe08b4ed4e9561f700af26a7cb252760b89ee8`, 10230 lines).

**Item 4, whole-chain revert and migrate** (`g6_fresh`): `cargo run migrate --revert` reverted all 32 migrations and `cargo run migrate` reapplied them, both with exit 0 (the second and third lines of item 1).

**Item 5, a second migrate is a no-op** (`g6_fresh`): one more `cargo run migrate` exited 0. All 32 `__diesel_schema_migrations` rows (version and `run_on`) and the schema dump are unchanged.

**Item 2, production-like database** (`g6_prod`; handoff item 2, "reverting the engine migration returns a schema dump identical to the develop tip's"):

1. The harness `apply-through:20261113` applied `develop`'s 31 migrations one at a time, `20250000` to `20261113`, so v1.7.0 (`20260814`, BE-04) is applied.
2. Seeded Publisher Services rows: 1 publisher, 1 enabled `INTERNET_ARCHIVE` assignment, and 1 PENDING `PUBLISHER_BACK_CATALOGUE` distribution job with its target.
3. Dump A: schema 8776 lines, sha256 `895588a9e5aca781359d10a28861fed42f81f6cab84b6feca98c87f8abf4095f`; data dump 817 lines.
4. `cargo run migrate` applied the engine migration only: exit 0, 32 applied, newest `20261115`, engine objects present. Against dump A, the applied dump only adds lines: 1454 added, none removed or changed (the same count as E1's).
5. The harness `revert-through:20261115` reverted with `revert_last_migration` until the engine migration was reverted. It reverted `20261115` only. Afterwards 31 were applied, newest `20261113`, and no `async_` table, type, function or trigger was left.
6. Dump B is byte-identical to dump A, and data dump B is byte-identical to data dump A.
7. `cargo run migrate` reapplied the engine migration: exit 0. The schema dump is byte-identical to the first apply's and to item 1's (`5d74bed1...`).

**The `develop` reference for item 2, without the harness** (`g6_devtip`): a new, empty database migrated by the released v1.10.0 binary alone (item 6) has 31 migrations applied, newest `20261113`. Its schema dump is byte-identical to dump A and to dump B (sha256 `895588a9...`, 8776 lines). v1.10.0's migrations are the develop tip's (item 6), so dump B is the develop tip's schema whether the chain is applied one migration at a time or all at once.

**Item 3, pre-v1.7 path** (`g6_pre17`; handoff item 3):

- The harness `apply-through:20260813` applied, one at a time: `20250000 20260417 20260429 20260504 20260805 20260811 20260812 20260813`.
- `cargo run migrate` then applied the 24 pending migrations, exit 0. In `run_on` order: `20260814` (BE-04's v1.7.0) first, then `20260826` to `20261113`, and `20261115` last. The `run_on` values strictly increase in version order.
- The final schema dump is byte-identical to item 1's (`5d74bed1...`), so all three paths end in the same schema.

Each harness step ran as one passing ignored test (`1 passed; 0 failed`) in the `thoth-api` binary; the other lib binaries ran no test.

**Item 6, previous-image compatibility** (handoff item 4: the v1.10.0 binary against `g6_prod` after step 7 of item 2, with the integrated chain applied and the seeded rows present):

- Release: tag `v1.10.0`, a lightweight tag on `32af2a9df8068e485c6a9b1a5ef96fb3dac48668` ("Merge pull request #1035: Release v1.10.0"), as the handoff states. `git diff v1.10.0 bd69aaaf` touches only `CHANGELOG.md` and three decision documents, so v1.10.0's code and migrations are the develop tip's.
- Build, as the handoff directs:
  - `git fetch origin tag v1.10.0` (section 4.2 records the tag refs it created and their deletion);
  - `git worktree add --detach /home/claude/thoth-v110 v1.10.0`;
  - `cargo build` there with `CARGO_TARGET_DIR=/home/claude/thoth-e1/target`: exit 0 in 126 s. `Cargo.lock` and every `Cargo.toml` at the tag are identical to the head's.
  - The binary reports `thoth 1.10.0`, as the head's does too, since the workspace version is unchanged. Unlike the head's binary, it holds no string that contains `20261115`.
- How it ran, where `<g6_prod>` is the database URL:

  ```text
  DATABASE_URL=<g6_prod> thoth migrate
  PRIVATE_KEY=<throwaway key> DATABASE_URL=<g6_prod> REDIS_URL=redis://localhost:6379 \
    GRAPHQL_API_HOST=127.0.0.1 GRAPHQL_API_PORT=18001 THOTH_GRAPHQL_API=http://127.0.0.1:18001 \
    ZITADEL_URL=http://127.0.0.1:18090 AWS_REGION=eu-west-1 RUST_LOG=info \
    thoth start graphql-api
  ```

  The two AWS key variables held placeholder values; no request used AWS.

- v1.10.0 `migrate`: exit 0, no output. The 32 `__diesel_schema_migrations` rows (version and `run_on`), the schema dump (`5d74bed1...`) and the data dump are unchanged. Nothing was pending: v1.10.0's 31 migrations are all applied, and the newest row, `20261115`, is unknown to it.
- v1.10.0 `start graphql-api`, with the identity stand-in of section 8 on 127.0.0.1:18090, served three requests, each with HTTP 200:
  - BE-04's staff report, with the stand-in's superuser token:

    ```json
    {"data":{"publisherServiceConfigurations":[{"configuration":{"publisher":{"publisherId":"96000000-0000-4000-8000-000000000001","publisherName":"G6 Evidence Press"},"subscriptionPackage":"OBELISK","enabledDistributionPlatforms":[{"platform":"INTERNET_ARCHIVE","enabledAt":"2026-11-16T00:00:00Z"}]},"latestBackCatalogueJob":{"distributionJobId":"96000000-0000-4000-8000-0000000000b1","kind":"PUBLISHER_BACK_CATALOGUE","status":"PENDING","attemptCount":0,"availableAt":"2026-11-16T00:00:00Z","targets":[{"platform":"INTERNET_ARCHIVE"}],"attempts":[]}}],"publisherServiceConfigurationCount":1}}
    ```

    The request was `publisherServiceConfigurations(jobStatuses: [PENDING])` and `publisherServiceConfigurationCount(jobStatuses: [PENDING])`, with `latestBackCatalogueJob`, its targets and its attempts.
  - The same request without a token: `Unauthorized` (`NO_ACCESS`), since the report is superuser only.
  - BE-02's public `publishersByDistributionPlatform(platform: INTERNET_ARCHIVE)` and its count: the seeded publisher, count 1.
- The stand-in received the discovery request, the key-set request and one introspection carrying a JWT client assertion, signed with the throwaway key. The server log has no `ERROR`, `WARN` or panic line. After the server stopped, `g6_prod` still had 32 migration rows, newest `20261115`.
- Cleanup: `git worktree remove` and `git worktree prune` removed the worktree, and `git worktree list` no longer shows it. The copied binary and the key files were deleted, and so were the tag refs (section 4.2). Nothing from the worktree was committed or pushed.

## 7. API and compatibility effects

GraphQL/API changes: relative to `develop`, E2's additive API (5 queries, 14 mutations and 40 declarations; E2 report section 7). This task's commits change no GraphQL code. The `executor_profile` arm decides authorization; it adds no GraphQL surface.

Generated SDL, `develop` against the head: 629 lines added and none removed, in 13 hunks (5868 lines to 6497), the same counts as E2's diff against its base. Each side is `thoth-client/assets/schema.graphql` as the crate's build script wrote it: `develop`'s in the v1.10.0 build of section 6.1 item 6, whose code is the develop tip's, and the head's in the build at `60aff219`.

- 40 new declarations:
  - 13 enums: `AsyncAttemptPurpose`, `AsyncAttemptResult`, `AsyncControlCommandKind`, `AsyncControlCommandOutcome`, `AsyncEffectPhase`, `AsyncJobCompletionOutcome`, `AsyncJobEffectClass`, `AsyncJobFailureClass`, `AsyncJobLineage`, `AsyncJobReconcilerOutcome`, `AsyncJobRevisionMode`, `AsyncJobStaffOutcome`, `AsyncJobStatus`;
  - 14 input objects: `AsyncJobClaimInput`, `AsyncJobCommandInput`, `AsyncJobEvidenceEntryInput`, `AsyncJobFilterInput`, `AsyncJobKindVersionsInput`, `AsyncJobListPositionInput`, `ClaimAsyncJobsInput`, `CompleteAsyncJobInput`, `FailAsyncJobInput`, `MarkAsyncJobEffectConfirmedInput`, `RenewAsyncJobLeaseInput`, `ResolveAsyncJobReconciliationAsStaffInput`, `ResolveAsyncJobReconciliationInput`, `WaitAsyncJobInput`;
  - 13 object types: `AsyncJob`, `AsyncJobAttempt`, `AsyncJobBlockedJob`, `AsyncJobClaimResult`, `AsyncJobCommandResult`, `AsyncJobDetail`, `AsyncJobExecutorActivity`, `AsyncJobKindSignals`, `AsyncJobLeaseRenewal`, `AsyncJobRegressedTarget`, `AsyncJobTargetState`, `AsyncJobTransition`, `ClaimedAsyncJob`.
- 19 root fields, each with its description:
  - `QueryRoot`, 5: `asyncJobTargetState`, `asyncJob`, `asyncJobs`, `asyncJobKindSignals`, `asyncJobExecutorActivity`;
  - `MutationRoot`, 14: the 10 executor operations `claimAsyncJobs`, `renewAsyncJobLease`, `markAsyncJobEffectStarted`, `markAsyncJobEffectConfirmed`, `waitAsyncJob`, `completeAsyncJob`, `failAsyncJob`, `releaseAsyncJob`, `claimAsyncJobReconciliations` and `resolveAsyncJobReconciliation`, and the 4 operator commands `cancelAsyncJob`, `retryAsyncJob`, `resolveAsyncJobReconciliationAsStaff` and `redactAsyncJobDiagnostics`.
- Compared declaration by declaration, all 291 of `develop`'s declarations are in the head SDL, which has 331. 289 are byte-identical. `QueryRoot` and `MutationRoot` only gain the new fields (10 and 28 lines), and each keeps every one of its `develop` lines, in order. So the 12 BE-04 declarations and the 7 BE-04 fields are unchanged.

Generated schema/client updates: `cargo build` and `cargo test --workspace` compile the internal `thoth-client` at the head, and its build script wrote the head SDL above. Its queries use no new type, so the generated client is unchanged. E2's note that `cargo build -p thoth-client` alone fails without the workspace's feature unification (E2 report, R7) concerns code this task does not touch; it was not re-run.

Backwards compatibility: no existing type, field, argument, description or enum value changes; the SDL diff removes no line.

Deprecations: none.

Cross-repository dependencies: E2's section 7 conclusions stand. `thoth-dissemination` remains compatible (its BE-04 API is byte-identical); `thoth-app`, the standalone `thoth-client`, `metrics-dashboard`, `metrics-widget`, `baboon`, `thoth-pyramid` and the internal `thoth-client` remain compatible, because the change is additive only. No downstream task is needed for Release N.

## 8. Authorization and security

Authorization paths changed: `executor_profile` gains `Role::RorSyncService` in its `None` arm. A caller whose only role is `ROR_SYNC_SERVICE` gets no executor profile, so every executor operation refuses it, and every operator command and read still requires `SUPERUSER`. No other path changes.

Roles/scopes involved: `ROR_SYNC_SERVICE` (unscoped machine role, from `develop`), `DISSEMINATION_WORKER`, `SUPERUSER`.

Negative authorization tests: `ROR_SYNC_SERVICE` is now among the callers that E2's authorization tests refuse: the outsiders of every new operation, the refused executors and the refused operators (section 5, decision 4). They pass (section 9).

Secret or personal-data handling:

- No secret or credential was read, printed or searched for.
- The previous-image run (section 6.1 item 6) used a local identity stand-in on `127.0.0.1` and a throwaway 2048-bit RSA key. The key was generated for that run with `umask 077`, never printed, and deleted afterwards.
- The stand-in's log records only request paths, form keys and whether a client assertion was sent, never their values.

Security limitations: unchanged from E1 and E2.

## 9. Tests and checks

All runs: code head `60aff219`, `CARGO_INCREMENTAL=0`, PostgreSQL 17.10 and Redis local, the env file of handoff section 10 (`TEST_DATABASE_URL` names `thoth_test`).

### Formatting

Command:

```text
cargo fmt --all -- --check
```

Result:

```text
exit 0, no output (rustfmt 1.9.0-stable)
```

### Unit and database tests

Command: CI's `cargo test --workspace -vv`, run as three commands that together select the same targets (reading R6; the third rebuilt some dependencies):

```text
cargo test --workspace --lib --bins --no-fail-fast
cargo test --workspace --doc --no-fail-fast
cargo test --workspace --test graphql_permissions --test institution_ror_state --no-fail-fast
```

Result:

```text
thoth (lib)                          ok. 0 passed; 0 failed; 0 ignored
thoth (bin)                          ok. 40 passed; 0 failed; 0 ignored
thoth-api (lib)                      ok. 2630 passed; 0 failed; 7 ignored; finished in 1094.73s
thoth-api-server                     ok. 11 passed; 0 failed; 0 ignored
thoth-client                         ok. 4 passed; 0 failed; 0 ignored
thoth-errors                         ok. 15 passed; 0 failed; 0 ignored
thoth-export-server                  ok. 144 passed; 0 failed; 0 ignored
thoth-api graphql_permissions        ok. 13 passed; 0 failed; 0 ignored
thoth-api institution_ror_state      ok. 8 passed; 0 failed; 0 ignored
doc tests: thoth-api 0 passed, 8 ignored; thoth-client 6 passed; thoth-export-server 2 passed; the other three 0
exit 0 for each command
```

- The 7 ignored `thoth-api` tests are the explicitly run evidence tests: E1's 19.0 harness (run in section 6.1), the Metrics dashboard plan evidence, and five rollup performance-evidence tests. All 7 were ignored on `develop` or `feature/worker` already.
- The run includes the BE-04 regression suite unchanged, both BE-04 source guards (`the_worker_guard_is_one_explicit_predicate_with_no_inheritance_or_composition`, `no_generic_framework_universal_queue_or_shared_worker_convention_is_introduced`), the Metrics guard of E2's decision 10 (`the_claim_context_is_reachable_only_through_the_protected_claim_result`) and `the_be04_sdl_elements_are_byte_identical_to_the_base`. All pass.

### Lint/static analysis

Command:

```text
cargo clippy --all --all-targets --all-features -- -D warnings
```

Result:

```text
Rust 1.97.0, the container's stable (clippy 0.1.97, 2d8144b788, 2026-07-07): exit 0, finished in 1m 58s
Rust 1.100.0, the newest stable (clippy 0.1.100, 0ef1bc3860, 2026-11-09): exit 0, finished in 8m 30s
```

- Neither run printed a clippy warning. Each printed one cargo warning only: the future-incompatibility note for the dependency `proc-macro-error2` v2.0.1. This branch changes no Cargo file.
- The 1.100.0 run used a temporary toolchain and its own target directory. Both were removed afterwards.

### Workspace check

Command:

```text
cargo check --workspace
```

Result:

```text
Rust 1.97.0: exit 0, finished in 2m 17s
```

The only warning is the same `proc-macro-error2` future-incompatibility note.

### Other required checks

- BE-04, handoff section 6 item 7. The BE-04 paths are 16 files: the 15 whose path contains `distribution_job`, `distribution_platform`, `service_configuration` or `20260814_v1.7.0`, and `src/bin/commands/publisher_services.rs`.
  - `git diff --stat origin/develop HEAD -- <those paths>`: one file, `publisher_service_configuration/tests.rs`, 4 lines added (reading R4).
  - `git diff --stat origin/feature/worker HEAD -- <those paths>`: empty. So is the diff of ADR-0004, BE-04's decision record, against either side.
  - The SHA-256 pin test `the_be04_sdl_elements_are_byte_identical_to_the_base` passes.
- `git grep 20261115` at the head: E1's report only (section 6).
- The binaries embed the renamed migration: `strings target/debug/thoth` and the `thoth-api` test executable each show `20261115_v1.11.0` once and `20261115_v1.10.0` zero times.
- `git diff --check` on the report commit's staged changes printed nothing.

### 9.1 Migration revert guards

Every test that reverts a migration passes unchanged on the integrated chain, so no guard fix was needed (handoff section 5 item 3 was not used).

A static search found these tests: every `#[test]` function that calls `revert_last_migration`, or calls a helper that does, with comments excluded. It finds 64 tests in 20 modules. 63 ran and passed in the `thoth-api` run above. The 64th is E1's ignored 19.0 harness, which section 6.1 runs by hand.

The guards the handoff names:

- OPS-03: `model::metric_rollup_delta::rebuild_run_tests::the_migration_applies_reverts_and_reapplies_changing_no_other_schema_object`. #1032 made it revert later migrations first.
- ROR-SYNC C: `model::institution::tests::ror_state::the_migration_applies_to_empty_and_populated_tables_and_reverts` and `model::institution::tests::ror_state::the_manual_rollback_pair_then_the_runner_restores_the_schema`.
- Metrics B1: `model::metric_coverage_run::tests::reverting_through_the_b1_migration_removes_exactly_its_objects_and_reapplication_restores_them` and `model::metric_coverage_run::tests::the_migration_backfills_nothing_over_populated_raw_evidence`, as E1 changed them under its amendment.

The others, by module:

- BE-04 (`model::distribution_job::tests`): 5;
- `metric_platform`: 19;
- `metric_rollup_delta::tests`: 10;
- `metric_import_batch` and `work_relation`: 5 each;
- `metric_identifier_quarantine`: 3;
- `metric_source_registry_history`: 2;
- one each in `metric_coverage`, `metric_import`, `metric_operas_export`, `metric_operas_import`, `metric_operas_mapping`, `metric_publisher_platform_approval`, `metric_reconciliation_run`, `metric_registry_history` and `metric_source`.

All of them pass at the head, where the engine migration is the newest on the chain. BE-04's `the_migration_directory_keeps_its_exact_historical_name` also passes.

### 9.2 ADR conformance, re-bound per spec 3

This report cites ADR-0012 at blob `892cbcad50c6dbf9bd943590a3bd7027b78e0341` and ADR-0015 at blob `55c38c2e0b4856562e42a19e7eff4d865be0fc39`, both APPROVED and reachable from `develop` at `bd69aaaf`. The integration changes neither blob, and changes no engine code beyond the amendment's arm.

ADR-0015 section 9, "Release N obligations":

- **Section 4.3, part 1** (ADR-0012's rollout list, lines 3707-3734): item 1's job part (schema and job concurrency without external writes), item 4 (crash, lease renewal, `WAITING` and reconciliation recovery), item 5's job part (effect-scoped idempotency and revision ordering) and item 6 (priority, fairness, per-kind concurrency and bounded retry and wait deadlines). E1's tests for these are its section 9.1 groups: schema and triggers, claims, leases and tokens, effect phases, recovery, creation, targets and evidence, and deadlock freedom. They all pass on the integrated chain (section 9.3).
- **Section 4.4**, the matrix items that gate Release N: the production-like schema with v1.7.0 applied (section 6.1 item 2), the pre-v1.7 upgrade (item 3), the full empty-database chain (item 1) and the mixed-version rolling deploy during Phase A, as previous-image compatibility (item 6).
- **Section 4.6**, every item and part in the "Release N" column: E1's and E2's spec 19.1 traceability (E1 report section 9.1, E2 report section 9.1). Every one of those tests passes on the integrated chain (section 9.3).

The four duties of FBC-3 (`E2-review-carry.md` item 7), which section 9's list leaves out:

1. **Section 4.7: no event or coalescing field in the kind policy.** `AsyncJobKindPolicy` (`thoth-api/src/model/async_job/registry.rs:123-186`) has exactly spec 8's fields: kind, domains, risk class, capabilities, versions, effect class, revision mode, target namespaces, admission, retry and claim budgets, backoff, poll, lease and runtime bounds, effect window, waiting horizon, concurrency limits, `max_targets`, cancel and retry policy, staff outcomes and the four kind functions. None declares event consumption, routing or coalescing; a search of the engine for `event`, `coalesc`, `route` and `consum` field names finds none. MET.
2. **Section 4.8: the `ACCEPT_REGRESSION` limits.** MET.
   - A protected staff command resolving a job in `RECONCILIATION_REQUIRED`: `resolve_async_job_reconciliation_as_staff` (`thoth-api/src/graphql/mutation.rs:2551`) calls `authorize_async_command` (`mutation.rs:431`), which requires `SUPERUSER` (`mutation.rs:436`). `decide_resolution` (`thoth-api/src/model/async_job/commands.rs:794`) refuses `NOT_RECONCILIATION_REQUIRED` unless the job is in `RECONCILIATION_REQUIRED` (`commands.rs:806`).
   - One case only: `OUTCOME_NOT_ALLOWED` unless the kind is `EFFECTFUL` and `REVISION_BOUND` and its policy lists the outcome (`commands.rs:823-829`), and `NOT_A_REGRESSION` unless the stored evidence is newer, or records the same order with other content (`commands.rs:837-838`, through `is_regression_or_conflict`, `commands.rs:757`). The registry allows `ACCEPT_REGRESSION` only on an `EFFECTFUL` `REVISION_BOUND` kind, and requires it there (`registry.rs:349-355`).
   - Kind-validated evidence that the provider holds the job's own revision: the `AcceptRegression` arm (`commands.rs:923-941`) refuses a command without evidence (`commands.rs:924`), validates it with `EvidenceUse::Applied` (`commands.rs:927`) and writes the job's own source order and fingerprint (`commands.rs:935-940`). The command then deletes the holds, closes the attempt `REGRESSION_ACCEPTED` and ends the job `SUCCEEDED` (`commands.rs:1036-1053`). The database accepts a backwards write only from a `STAFF_RECONCILIATION` attempt of a `REVISION_BOUND` job writing its own order and fingerprint (`up.sql:899-911`).
   - An executor reconciler never accepts a regression: its evidence check, `check_entries_semantic`, fails on a regression and the call is refused (`thoth-api/src/model/async_job/engine.rs:2122-2125`).
   - Tests, all passing: E1's `regression_a_t6x_job_accepted_with_valid_evidence_succeeds_and_records_the_regression`, `regression_a_forward_write_below_the_pre_acceptance_order_keeps_it_and_one_above_or_equal_clears_it`, `regression_an_executor_reconcilers_regressing_succeeded_is_refused` and `registry_refuses_an_effectful_kind_without_the_required_staff_outcomes`; E2's `an_accepted_regression_is_recorded_through_the_api_and_cleared_only_by_ordinary_progress` (`thoth-api/src/graphql/async_job_tests.rs:2962`).
3. **Rule 1(h): every Release N idempotency key begins with its kind.** Creation builds `format!("{}:{}", input.kind, identity)` (`engine.rs:463`). A retry child's key is the parent's key plus `#retry:<parent id>` (`commands.rs:533`), so it keeps the prefix. The database enforces the rule with `CONSTRAINT async_job_idempotency_key_check CHECK (starts_with(idempotency_key, kind || ':') AND char_length(idempotency_key) <= 512)` (`up.sql:116-119`), and the key cannot change on update (`up.sql:652`). `schema_async_job_checks_accept_their_boundary_and_refuse_one_past_it` (`thoth-api/src/model/async_job/tests.rs:589`) shows a key `'OTHER:x'` refused (`tests.rs:648`). MET.
4. **Test 10: ascending idempotency-key order** for a Release N caller that creates more than one job in one transaction (ADR-0015 rule 8, test 10, line 441). This is a caller duty: spec 10.1 (line 486) and spec 11's "Creation" bullet (line 571). `create_job_on` (`engine.rs:642`) creates one job and takes no engine lock, and the only other insert is `RETRY_JOB`'s single child (`commands.rs:579`). No production code calls `create_job_on`: outside `async_job/`, only E2's GraphQL tests call it, to seed jobs (`async_job_tests.rs:997`). So no Release N path creates two jobs in one transaction, and the duty has nothing to order yet. MET; reading R5 notes the doc comment for step 4.

No ADR-0015 duty is unmet, so there is no HOLD.

### 9.3 Spec 19.2 to tests on the integrated chain

The mapping of spec 19.2 to named tests is E1's (E1 report section 9.1, 158 tests) and E2's (E2 report section 9.1, 41 tests, 35 of them mapped to 19.2 sentences). This task adds no test and removes none. Its only test change is the role lists of decision 4, inside existing E2 tests.

| Module | E1 and E2 tests | Result at `60aff219` |
|---|---|---|
| `thoth-api`, `model::async_job::tests` | 158 (E1), 5 (E2's carry tests) and the ignored 19.0 harness | 163 passed, 1 ignored |
| `thoth-api`, `graphql::async_job_tests` | 25 (E2) | 25 passed |
| `thoth-api`, `graphql::types::async_job` | 5 (E2's claim-memo tests) | 5 passed |
| `thoth-api-server`, `logger` | 6 (E2's access-log tests) | 6 passed |

That is 199 tests, all passing, and the counts match the E1 and E2 reports.

### 9.4 Concurrency-test method

Unchanged from E1 (E1 report section 9.3). Every concurrent test uses real, separate PostgreSQL connections, each on its own thread, and never sleeps. The tests are ordered by barriers, transactions held open, advisory-lock and row-lock pauses, and bounded polling of `pg_locks`. The 20 tests that run concurrent sessions or hold locks across connections read `pg_stat_database.deadlocks` before and after. All of them pass on the integrated chain.

## 10. Manual verification

Environment: local only (section 9).

Steps:

1. The spec 19.0 evidence (section 6.1 items 1 to 5) on disposable `g6_*` databases.
2. The v1.10.0 binary against the migrated database (section 6.1 item 6), and alone on a new database as the `develop` reference of item 2.
3. The SDL captures at `develop` and at the head, and their diff (section 7).

Observed result: as recorded in sections 6.1 and 7.

Evidence link/screenshot/log reference: none committed; the logs and dumps stayed in the session's scratch folder.

## 11. CI

CI status: NOT AVAILABLE in this task. This report records local results. CI runs on the orchestrator's draft PR into `develop` (`build-test-and-check`, `check-changelog`, `run-migrations` and the staging image check), and GitHub holds its record. The newest-stable clippy run of section 9 stands in for CI's `lint` job until then.

## 12. Rollout and rollback

Initial state after merge: inert, as E1 and E2 shipped it. The production registry is empty and there is no creation path.

Activation required: none in Release N.

Feature flag/configuration: none.

Migration sequence: `20261115_v1.11.0` runs after `20261113_v1.10.0`, through the normal path of the release that contains it (v1.11.0), under that release's own grant (spec 21).

Rollback/disable procedure: spec 21, unchanged. Before any `async_*` row exists: redeploy the previous image (the tables stay, unused; section 6.1 item 6 shows v1.10.0 serving against the migrated schema) or revert the migration in a non-production database. After rows exist: stop creation and claiming and keep the rows.

Monitoring required: none in Release N.

## 13. Known limitations and deferred work

- E1's and E2's limitations (spec 23) are unchanged.
- FBF-4, the SDL guard's word segmentation, stays carried to step 4 (amendment, handoff section 12).
- The other `E2-review-carry.md` items stay due at step 4 or at the first `REVISION_BOUND` kind, as that file says (items 1 to 4 and 6).
- Reading R5: step 4's first creation path should state the ascending key order where it creates more than one job in a transaction.

## 14. Unresolved issues

- For the reviewers: reading R4. The handoff expects the BE-04 diff against `develop` to be empty; it holds E1's spec-authorized `%job%` inventory assertion (4 lines), and is empty against `feature/worker`.
- For the reviewers: reading R6, the subset method for the suite and the harness invocation.
- For the orchestrator: reading R7. Building a previous release into the head's target directory, as handoff section 6 item 4 directs, leaves stale workspace artifacts that cargo takes as up to date. A later handoff that reuses the method should add `cargo clean -p` of the workspace crates after it.
- Carried from E2 (its reading R8): FBE-5's `pg_input_error_info` needs PostgreSQL 16 or later in every environment that runs this code. CI and `docker-compose` use 17; the production server's version needs confirming outside this repository before the v1.11.0 release.

## 15. Agent self-assessment

The agent may identify risks but may not approve the task.

Suggested review focus:

- The `schema.rs` union and the `CHANGELOG.md` line (section 5, decisions 1 and 2).
- The amendment: the arm, and that it is the only engine change (section 0).
- The role lists and reading R1.
- The rename: identical blobs, the embedded name, no path left naming the old directory.
- Section 6.1: dumps A and B, the independent `develop` reference, and the v1.10.0 run.
- Readings R4 and R7.

## 16. Corrections to the E1 and E2 reports

The E1 and E2 reports each gain one line under their title that points here. Nothing else in them changes.

16.1. **E2 decision 10 was a HOLD trigger that was not raised.** E2's section 5 says "Deviations from the specification requiring authorization: NONE." That is replaced by: decision 10 renamed the list position input to `AsyncJobListPositionInput` after the first name, `AsyncJobCursorInput`, failed an existing guard (`graphql::metric_ingestion_lifecycle_tests::the_claim_context_is_reachable_only_through_the_protected_claim_result`). An existing guard failing is a HOLD trigger (spec 22.6; E2 handoff sections 5 and 12), and E2 did not raise a HOLD. Both review legs ratified the rename itself, and the CTO ratified it through MERGE AUTH 6921120520. The guard passes unchanged.

16.2. **The `distribution_job` claim.** E2 section 4.1 says: "No added line contains `distribution_job` or `publisher_service_configuration`." It should read: "No added code or test line contains `distribution_job` or `publisher_service_configuration`." E2's own report adds 4 lines that contain them; no other added line does (`git diff 3464fdbf 59e967e3`, counted per file).

16.3. **E2's remaining gates.** E2's report has no remaining-gates entry. At E2's head they were: the draft PR into `feature/worker`, both CRITICAL review legs at the exact head, and a SHA-bound MERGE AUTH; then G6, this integration.

16.4. **E1's "no directory rename".** E1 section 1 says G6 "needs no directory rename" for `develop`'s `20261112` migration. That held at E1's time, for the date: `20261115` still sorts after `20261112`, `20261113` and `20261114`, and G6 keeps it. G6 renames the suffix only, `_v1.10.0` to `_v1.11.0`, because v1.10.0 shipped without the engine (section 6). E1's report keeps its 13 mentions of `20261115_v1.10.0`; they record E1's state.

Also recorded from `E2-review-carry.md` item 5:

16.5. E2 gave its SDL diff as counts only. Section 7 of this report names what the integrated SDL adds to `develop`'s.

16.6. E2's commits `c9fde12`, `b63bab9` and `1c30a24` fail the Metrics guard on their own. This is a bisect hazard only: E2's head passes. This task's commits change no GraphQL code after the merge, and the guard passes at the code head.

## Remaining gates

- The orchestrator's push of `feature/worker--integrate-01` and its draft PR into `develop`.
- Both CRITICAL review legs at the exact head.
- A SHA-bound MERGE AUTH.

Release, deployment and the production migration are not covered by G6. The engine migration ships with v1.11.0 under that release's own grant.
