# MET-WP4-03A-OPS-03 Implementation Report

## 1. Repository state

Owning GitHub issue: thoth-pub/thoth#1012 (programme: Thoth Metrics, #766)
Repository: thoth-pub/thoth
Workflow: STANDARD
Base branch: `develop`
Authorized base commit: `ce8cc1fd782675b029a531063ec77845cff4b14d`
Actual base commit: `ce8cc1fd782675b029a531063ec77845cff4b14d`
PR target: `develop`
Programme integration branch: none
Task branch: the task branch named in the stage brief (the pull request's
head branch)
Head commit: recorded on the pull request; the PR head is the authority. The
commits are listed in section 3
Pull request: opened by the orchestrator, not by the implementing agent; its
live state is recorded on GitHub
Expected branch deletion after merge: YES
Final programme PR required: NO
Implementing model: not named in this public report (stage brief rule)
Reasoning level: not recorded

## 2. Scope confirmation

Approved specification: the OPS-03 specification R9, approved in review round
9 by both CRITICAL reviewers, with erratum R9.1 (the six P3 items carried into
implementation), under the CTO's CRITICAL implementation grant of 2026-11-12
on #1012 (branch, commits authored as the CTO, push, and one PR to `develop`
opened by the orchestrator). Migration date reserved: `20261112_v1.10.0`.
After the first evidence run missed its bounds (section 14), the CTO chose
the amendment path on 2026-11-13: budget amendment R14, approved in review
round 14 by both CRITICAL reviewers, with erratum R14.1 (the P3 items carried
into implementation). It sets three step and page budgets from the first
run's evidence (named R11 in the specification), reworks the section 17
harness and asks for a re-run on CI's PostgreSQL major version (17). This
report covers both: the first run (sections 10.1 to 10.6, kept as history)
and the re-run (section 10.7).

Implemented objective: a resumable, caller-driven rebuild and paged
verification of the six derived monthly and yearly rollup projection families
for corpora above the OPS-02 bound. A run is one row of the new
`metric_rollup_month_rebuild_run` table: a durable cursor over the unit space
`(work_id, year_start)`. The caller drives every step through four
`MutationRoot` operations (`startMetricRollupMonthRebuild`,
`advanceMetricRollupMonthRebuild`, `abandonMetricRollupMonthRebuild`,
`metricRollupMonthRebuildRun`). A build step verifies its range and, only if
the range is not exact, replaces exactly that range with the completion's own
keyed statements and verifies it again before commit; a verify page checks a
range in one `REPEATABLE READ` snapshot and writes only the run row. Thoth
runs nothing on its own.

Out-of-scope changes made: NONE

## 3. Commits

Every commit is authored and committed as `Javier Arias <javier@jarias.org>`,
with no trailer.

1. `01e3473296c3c2d751a0ccc5350f05016df324be` - the run table migration, the
   rebuild run (start, advance, abandon, status, build step, verify page,
   sizing, classifier, logging, run-grant aggregate), the range verifier, the
   GraphQL operations and types, every section 16 test, the register
   amendments and the changelog line.
2. `67301a4ce3ad31957e854d2470124daa4593d1cd` - the section 17 evidence
   harness as ignored tests.
3. `c303579776fe386ff23e3817f06415b3b7274b9f` - statement profiles,
   diagnostic hold budgets and the production-budget F4 probe in the
   harness.
4. `fb0e9979070da17583b919a2d47d10184dfb9031` - the F2 and F6 evidence
   report a `verifyMetricRollupMonths` statement timeout instead of failing
   (section 10.5), and the number of F3 works becomes a knob.
5. `f14df4bf17d944fb6718b5b3738edbba3c3e66cd` - this report and the
   register 3.8 evidence of the first run.
6. `cde020c20d8b974d9bdcb68beab8bac885e32951` - the three budgets of
   amendment R14 (`STEP_MAX_WORK_DAY_ROWS` 8,000, `STEP_MAX_REPLACING_READ`
   600,000, `PAGE_MAX_WORK_DAY_ROWS` 150,000), their register 3.8 quotations
   and the test that pins both.
7. `c71c35e15538b22a4cd3f3c2639475b2a92987da` - the section 17 harness as
   amended by R14 and erratum R14.1 (section 10.7).
8. `0c3a685e3528d1f946c3d923499bba94f115fbfd` - the re-run's evidence in
   this report and register 3.8, with register 3.8's other R14.1 text.
9. The commit that states F5's growth limits on build and repair steps in
   register 3.8 and this report (section 5, deviations).

## 4. Files changed

Authorized write paths (specification section 18):

- `thoth-api/migrations/20261112_v1.10.0/up.sql` (new)
- `thoth-api/migrations/20261112_v1.10.0/down.sql` (new)
- `thoth-api/src/schema.rs` (the new table only)
- `thoth-api/src/model/metric_rollup_delta/mod.rs`
- `thoth-api/src/model/metric_rollup_delta/crud.rs` (visibility only)
- `thoth-api/src/model/metric_rollup_delta/verification.rs` (range verifier;
  the rendered `VERIFICATION_SQL` unchanged)
- `thoth-api/src/model/metric_rollup_delta/rebuild_run.rs` (new)
- `thoth-api/src/model/metric_rollup_delta/rebuild_run_tests.rs` (new)
- `thoth-api/src/model/metric_rollup_delta/tests.rs` (guard tests and shared
  helpers)
- `thoth-api/src/graphql/model.rs`
- `thoth-api/src/graphql/mutation.rs`
- `thoth-api/src/graphql/metric_rollup_tests.rs`
- `docs/metrics/contract-register.md`
- `docs/engineering/ai-delivery/implementation-reports/MET-WP4-03A-OPS-03-implementation-report.md`
  (new)
- `CHANGELOG.md` (one short line per user-facing change)

Actual files changed:

- `thoth-api/migrations/20261112_v1.10.0/up.sql`
  - reason: section 5, the run table.
  - behavioural effect: creates `metric_rollup_month_rebuild_run` with every
    section 5 `CHECK` and the partial unique index
    `metric_rollup_month_rebuild_run_one_open_idx` (at most one open run).
    No other schema object, no backfill, no lock on an existing table.
  - within authorized write budget: YES
- `thoth-api/migrations/20261112_v1.10.0/down.sql`
  - reason: section 5 rollback.
  - behavioural effect: drops exactly the index and the table, without
    `IF EXISTS`.
  - within authorized write budget: YES
- `thoth-api/src/schema.rs`
  - reason: the Diesel declaration of the new table.
  - behavioural effect: the `metric_rollup_month_rebuild_run` `table!` block
    and its entry in `allow_tables_to_appear_in_same_query!`; nothing else.
  - within authorized write budget: YES
- `thoth-api/src/model/metric_rollup_delta/mod.rs`
  - reason: sections 6 and 10.
  - behavioural effect: the run's types, inputs and enums, the section 10
    constants (`METRIC_ROLLUP_RUN_*`) and the module wiring of `rebuild_run`
    and its tests.
  - within authorized write budget: YES
- `thoth-api/src/model/metric_rollup_delta/rebuild_run.rs`
  - reason: sections 4 to 12.
  - behavioural effect: start, advance, abandon and status; the build step
    (sizing, lock order, re-size, pre-verify, range delete, the completion's
    keyed recompute statements, post-verify); the verify page; the per-statement
    deadlines and budget clocks; the classifier and the local error boundary;
    structured logging; the `pub` run-grant aggregate
    `METRIC_ROLLUP_RUN_GRANT_AGGREGATE_SQL` with the step budgets as binds.
  - within authorized write budget: YES
- `thoth-api/src/model/metric_rollup_delta/verification.rs`
  - reason: section 9, the independent range verifier.
  - behavioural effect: `verify_range` over `(lo, hi]` with its own SQL;
    the rendered `VERIFICATION_SQL` is byte-identical and hash-pinned.
  - within authorized write budget: YES
- `thoth-api/src/model/metric_rollup_delta/rebuild_run_tests.rs`
  - reason: section 16 tests, and the section 17 evidence as ignored tests.
  - behavioural effect: tests only.
  - within authorized write budget: YES
- `thoth-api/src/model/metric_rollup_delta/tests.rs`
  - reason: the guard tests learn the one new table; shared helpers become
    `pub(crate)` for the new test module.
  - behavioural effect: tests only.
  - within authorized write budget: YES
- `thoth-api/src/graphql/model.rs`
  - reason: section 6.
  - behavioural effect: the GraphQL objects of the run, its build and verify
    progress and its six families, every counter as a decimal string.
  - within authorized write budget: YES
- `thoth-api/src/graphql/mutation.rs`
  - reason: section 6.
  - behavioural effect: `startMetricRollupMonthRebuild`,
    `advanceMetricRollupMonthRebuild`, `abandonMetricRollupMonthRebuild` and
    `metricRollupMonthRebuildRun`, each requiring `METRICS_INGEST_SERVICE`.
  - within authorized write budget: YES
- `thoth-api/src/graphql/metric_rollup_tests.rs`
  - reason: section 16.1 and 16.21, and the existing OPS-02 name guards
    extended for the new names.
  - behavioural effect: tests only.
  - within authorized write budget: YES
- `docs/metrics/contract-register.md`
  - reason: section 14.
  - behavioural effect: new section 3.8; amendments to 3.6, 3.7 and 4.2,
    including the Q3 option (a) and Q5 wording.
  - within authorized write budget: YES
- `CHANGELOG.md`
  - reason: one user-facing line under `[Unreleased]` / `Added`.
  - within authorized write budget: YES
- this report
  - within authorized write budget: YES

Not changed: `crud.rs` (the statements and helpers the build step reuses were
already crate-visible, so no visibility change was needed).

Actual new files created:

- `thoth-api/migrations/20261112_v1.10.0/up.sql` - within authorized
  new-file list: YES
- `thoth-api/migrations/20261112_v1.10.0/down.sql` - YES
- `thoth-api/src/model/metric_rollup_delta/rebuild_run.rs` - YES
- `thoth-api/src/model/metric_rollup_delta/rebuild_run_tests.rs` - YES
- this report - YES

Files deleted, moved or renamed: NONE

### 4.1 Write-budget compliance

PASS

## 4.2 Authorized actions actually used

- repository inspection: YES
- source edit: YES, inside section 18
- new file creation: YES, the five new files above
- file deletion/move/rename: NO
- branch creation: YES. The local task branch already existed in the
  checkout, pointing at `origin/master` (`f348e90`); it was moved to the
  authorized base `ce8cc1f` before the first commit. The remote branch did
  not exist before the first push.
- commit: YES, as `Javier Arias <javier@jarias.org>`
- push: YES, of the task branch only, with `git push -u origin` and the
  branch's name
- PR creation/update: NO (the orchestrator opens the PR)
- issue/comment mutation: NO
- manual CI dispatch/rerun: NO
- provider/runtime read: NO
- provider/runtime write: NO
- migration execution: local disposable databases only (section 6)
- release/tag/publication: NO
- merge: NO
- deployment: NO
- production activation: NO
- other: local only, for the re-run: the `uuid-ossp` extension built into the
  local PostgreSQL 17 installation (section 5, deviations)

Unauthorized actions performed: NONE

## 4.3 Automatic and manual external effects

Automatic CI/provider effects observed: pushing the task branch triggers no
workflow (the `push` triggers name only `master` and `develop`). Opening the
PR, which the orchestrator does, runs the `pull_request` workflows, among
them `publish-to-dockerhub`, which may push a `staging-pr-<n>` image to
`ghcr.io/thoth-pub/thoth` when its classifier selects it.

Manually initiated external actions: NONE

External writes/publication: NONE

## 5. Implementation decisions

Decisions made within the approved design:

1. Every statement inside a step or a page runs under its own deadline:
   `SET LOCAL statement_timeout = <ms>` immediately before it, where `<ms>`
   is the smaller of the 30 s ceiling and the time left in the current
   budget (pre-size, lock, hold or page), never caller input; a check between
   statements refuses the next one once the budget is spent (`rebuild_run.rs`,
   `Clock` and `set_deadline`).
2. The budgets are one `RunBudgets` value. Production always passes
   `RunBudgets::PRODUCTION`; crate-private `_within` variants accept other
   budgets for tests only, following the OPS-02 `_within` pattern.
3. Sizing is one recursive statement per `SIZING_BATCH_WORKS` works, walking
   the `(work_id, day)` index work by work from the cursor; `PER_YEAR_SQL`
   splits a first work that alone exceeds a budget by year. The run-grant
   aggregate computes the same rows, keys and figure from its own SQL, with
   the step budgets as binds `$1` to `$3`.
4. A replacing step issues the completion's own keyed statements
   (`MONTH_SOURCE_WATERMARK_SQL`, `MONTH_INSERT_STATEMENTS`,
   `YEAR_COUNTRY_INSERT_SQL`, `YEAR_INSTITUTION_INSERT_SQL`) in the
   completion's chunks of 50 keys, without the six keyed deletes, which the
   range delete makes provably empty (section 8.1; test 16.22 pins both).
5. The classifier works on the raw `diesel::result::Error` before any
   conversion, and the first classified error of a step survives a failed
   rollback.
6. `metricRollupMonthRebuildRun` without `runId` orders the open run first,
   then `started_at` (set by the insert) descending, ties broken by `run_id`.
7. Every counter is a decimal string in GraphQL, so no value is truncated to
   a 32-bit `Int`.
8. Two operation descriptions say "durable position" rather than "cursor":
   an existing guard
   (`the_claim_context_is_reachable_only_through_the_protected_claim_result`)
   forbids that word in the `MutationRoot` SDL block, and section 6 does not
   fix the descriptions.
9. The section 17 evidence is ignored tests in `rebuild_run_tests.rs`
   (module `evidence`). Step holds are measured from the statement log of the
   run's connection (from the finish of the state-row lock to the end of the
   transaction), page times from `BEGIN` to the end of the transaction, and
   both replacing-read terms of every step are counted directly from the
   work-day projection, independently of sizing. Every replacing step and
   every refusal also records where its time went, per statement.
10. Because steps that need more than `STEP_HOLD_BUDGET` are refused, and a
   refused step reports no hold, the harness measures a refused range once
   under labelled diagnostic budgets (pre-size 60 s, step hold 120 s, page
   120 s; the lock budget and every size budget as in production, so the
   range is the same), counts it as a refusal of its class, and moves its
   pass on (R14.1; the first run instead raised the hold budget for a whole
   run through `THOTH_EVIDENCE_STEP_HOLD_SECS`, which is gone). Every pass
   runs under the section 10 budgets, and no acceptance line can pass on a
   diagnostic measurement. Production code is unchanged: the budgets are
   injected only through the tests' `_within` entry points (decision 2). F6
   never uses the diagnostic budgets.
11. At F2's size `verifyMetricRollupMonths` exceeds its 30 s statement
   timeout, the register 3.7 bound past which section 3.8 is the supported
   path. The F2 and F6 evidence report that outcome instead of failing after
   their `RECONCILED` passes; an answer, when one comes, must be exact. The
   number of F3 works is a knob (default 4), so that F6 can run without the
   works whose replacing steps exceed the production hold budget on the
   evidence host (section 10.6).

Deviations from the specification requiring authorization:

- NONE in the implementation. In the first run's evidence, F6 ran on F2
  without its embedded F3 and F4 works (sections 10.6 and 14); the re-run's
  F6 ran on the whole of F2 with them (section 10.7).
- Re-run environment (approved by the orchestrator before use): the
  PostgreSQL 17.10 installation lacked the `uuid-ossp` extension, which the
  Thoth migrations create (`20250000_v1.0.0`), because its configure had no
  `--with-uuid`, so no database test could run on it. Only
  `contrib/uuid-ossp` was built, from the same verified 17.10 source with the
  server build's flags plus `--with-uuid=e2fs`, against the system libuuid
  (`libuuid1` and `uuid-dev` 2.39.3-9ubuntu6.6), and its four files (the
  library, its control file and two scripts) were added to the
  installation; the server binaries were not rebuilt or changed. The first
  run's distribution build has the extension (its configure has
  `--with-uuid=e2fs`). The extension build's configure line:

  ```text
  ./configure --prefix=<prefix> --without-readline --with-icu --with-llvm --with-uuid=e2fs
    LLVM_CONFIG=/usr/bin/llvm-config-18 CLANG=/usr/bin/clang-18
    CPPFLAGS="-Wdate-time -D_FORTIFY_SOURCE=3"
    CFLAGS="-g -O2 -fno-omit-frame-pointer -mno-omit-leaf-frame-pointer -flto=auto
      -ffat-lto-objects -fstack-protector-strong -fstack-clash-protection -Wformat
      -Werror=format-security -fcf-protection -fno-omit-frame-pointer"
    LDFLAGS="-Wl,-Bsymbolic-functions -flto=auto -ffat-lto-objects -Wl,-z,relro -Wl,-z,now
      -Wl,--as-needed"
  make -C contrib/uuid-ossp
  ```
- F5's growth limits, stricter than specified (decided by the
  orchestrator after the re-run): section 17 defines each growth's largest
  unit within 4 s on its repair steps. On PostgreSQL 17 the build step was
  the dearer one at growth 3's top levels (4,452.9 ms against 3,893.2 ms at
  261 rows per key-month, the repair-only limit), so register 3.8 states
  each growth's limit as the largest level at which every step, build and
  repair alike, committed within 4 s: growth 3 at 220 rows per key-month
  (10,560 rows, figure 380,160, whole-work 84,480) instead of 261 (12,528
  rows, figure 451,008, whole-work 100,224); growths 1 and 2 are the same
  under both readings (27 and 7 rows per key-month). The reading is never
  more lenient: a unit the run grant passes on it also passes on the
  repair-only reading. T stays 107,520, from F4. Section 10.7.8 keeps the
  repair-only figures beside it, and the harness's `F5` and `THRESHOLD`
  lines still print the repair-only reading.
- Re-run host check: `pg_jit_available()` is false whenever `jit` is off,
  so the check that the build can JIT was made in a transaction that sets
  `jit` on, where it is true (`HOSTJIT`, section 10.7.1). The cluster's `jit`
  stayed off.

## 6. Database and migration effects

Migration added: YES

- migration files: `thoth-api/migrations/20261112_v1.10.0/up.sql` and
  `down.sql`.
- schema effect: one new table, `metric_rollup_month_rebuild_run`, with its
  section 5 checks and the one-open-run partial unique index. No other table,
  index, trigger or function.
- existing-data effect: none; no backfill.
- locking/downtime: `CREATE TABLE` and `CREATE UNIQUE INDEX` on the new,
  empty table only; no lock on an existing table.
- empty database result: on a fresh local database, `cargo run migrate`
  applied all 30 migrations (latest `20261112`; 94 tables, the run table and
  its partial unique index present), `cargo run migrate --revert` reverted
  them (0 tables), and a second `cargo run migrate` reapplied them (94
  tables, the run table present). Each command exited 0.
- populated database result: test 16.2
  (`the_migration_applies_reverts_and_reapplies_changing_no_other_schema_object`)
  applies, reverts and reapplies the migration on the test database with rows
  present and compares a catalog snapshot of every other schema object.
- rollback/forward repair: `down.sql` drops exactly the index and the table.
  Abandon any open run first. Projections need no rollback: every committed
  build step left its range exact, and an interrupted step rolled back.
- idempotency: neither file uses `IF EXISTS` or `IF NOT EXISTS`; a second
  apply or revert fails loudly, as the migration runner expects.

## 7. API and compatibility effects

GraphQL/API changes: four new `MutationRoot` fields
(`startMetricRollupMonthRebuild`, `advanceMetricRollupMonthRebuild`,
`abandonMetricRollupMonthRebuild`, `metricRollupMonthRebuildRun`), their
input types, the run, progress and family types, and the mode and status
enums. The SDL change is strictly additive (test 16.21).
Generated schema/client updates: none (the generated client schema is outside
the write budget and is not used by these operations).
Backwards compatibility: no existing field, type, error code or behaviour
changes. `verifyMetricRollupMonths` and `rebuildMetricRollupMonths` (OPS-02)
are unchanged and stay the fast path for small corpora.
Deprecations: none.
Cross-repository dependencies: the orchestrator that drives runs is a separate,
separately authorized stage; nothing here activates it.

## 8. Authorization and security

Authorization paths changed: four new operations, each requiring
`METRICS_INGEST_SERVICE`, checked before any connection, transaction or
rollup statement.
Roles/scopes involved: `METRICS_INGEST_SERVICE` only.
Negative authorization tests:
`every_row_of_the_authorization_matrix_holds_for_the_run_operations` runs the
complete matrix for all four operations and asserts, from the statement log,
that a denied call opens no transaction and issues no rollup-specific
statement.
Secret or personal-data handling: no message carries a frontier, row, SQL,
principal, cursor or connection detail
(`the_run_refusals_reach_the_caller_with_fixed_messages_and_no_identity`);
results expose counters, frontiers and the cursor's position only, never an
identity.
Security limitations: none known.

## 9. Tests and checks

Local runs on the evidence host (PostgreSQL 17.10 and Redis, test database
`thoth_test`), on the final tree of section 3, after the R14 re-run's
section 17 evidence:

- `cargo fmt --all -- --check`: PASS.
- `cargo clippy --all --all-targets --all-features -- -D warnings` (CI's
  command): PASS, no warning; only the existing future-incompatibility note
  for the `proc-macro-error2` dependency.
- `cargo test --workspace`: PASS, 2,589 passed, 0 failed, 14 ignored. Of
  the 6 ignored in `thoth-api`, 3 are the section 17 evidence tests (F1, F2
  with F3 to F6 embedded, and F7) and 3 are earlier evidence tests; the
  other 8 ignored are doc tests in `thoth-api` that existed before.
  `thoth-api` alone: 2,360 passed, 0 failed, 6 ignored. The first run had a
  fourth section 17 evidence test, `f5_growths`; R14.1 embeds F5 in F2's
  test, so it was folded into `f2_census_corpus_with_f3_f4_and_f6` (commit
  7), which is why one fewer test is ignored.
- `git diff --check`: PASS.

The first run's checks (PostgreSQL 16.15) also passed `cargo check
--workspace`, and `cargo run migrate` and `cargo run migrate --revert` on a
fresh local database (section 6); the migration is unchanged since.

No existing test was skipped, ignored or weakened.

### Section 16 tests and where they live

`rebuild_run_tests.rs` is `thoth-api/src/model/metric_rollup_delta/rebuild_run_tests.rs`;
`metric_rollup_tests.rs` is `thoth-api/src/graphql/metric_rollup_tests.rs`.

1. Authorization: `metric_rollup_tests.rs`
   `every_row_of_the_authorization_matrix_holds_for_the_run_operations`.
2. Migration: `rebuild_run_tests.rs`
   `the_run_table_accepts_every_legal_row_and_each_check_rejects_its_violation`,
   `at_most_one_run_is_open_and_terminal_runs_accumulate`,
   `the_migration_applies_reverts_and_reapplies_changing_no_other_schema_object`;
   the guard tests in `tests.rs` learn the one new table.
3. State machine: `rebuild_run_tests.rs`
   `a_rebuild_run_walks_building_verifying_reconciled_and_a_verify_run_walks_verifying`,
   `a_start_while_a_run_is_open_is_refused_and_concurrent_first_arrivals_make_one_run`,
   `terminal_stale_and_unknown_advances_write_nothing`,
   `abandon_works_in_every_state_and_keeps_cursor_counters_and_frontiers`,
   `the_status_read_prefers_the_open_run_and_otherwise_the_latest_started`,
   `abandon_against_an_in_flight_step_or_page_waits_and_wins_or_is_retryable`.
4. Concurrent advance:
   `two_concurrent_build_advances_with_one_count_make_exactly_one_step`,
   `two_concurrent_pages_with_one_count_make_one_page_and_one_serialization_refusal`,
   and the lost-response retry in `terminal_stale_and_unknown_advances_write_nothing`.
5. Equivalence: `a_build_from_empty_reaches_reconciled_and_equals_the_ops02_rebuild`.
6. Healthy REBUILD: `a_healthy_rebuild_writes_no_projection_row_and_keeps_every_surrogate`.
7. Corruption in each family:
   `verify_reports_each_corruption_in_its_family_and_rebuild_repairs_it`,
   `a_work_day_row_above_the_frontier_blocks_both_modes_until_abandoned`.
8. Orphans:
   `every_orphan_is_deleted_by_exactly_one_step_and_counted_by_exactly_one_page`.
9. Partition additivity:
   `per_range_counts_add_up_to_the_whole_table_verification_for_every_cut`.
10. Interleaving:
    `completions_between_steps_keep_the_built_prefix_exact_and_the_run_ends_exact`.
11. Concurrent canonical ingestion:
    `concurrent_canonical_ingestion_never_fails_and_steps_are_at_worst_retryable`.
12. Pages under change:
    `a_completion_during_a_non_final_page_gives_no_false_mismatch_and_ends_verified`,
    `a_completion_during_the_final_page_may_end_reconciled_at_the_frontier_it_saw`,
    `completions_between_pages_never_yield_reconciled`.
13. OPS-02 interplay:
    `the_full_rebuild_between_two_steps_leaves_the_run_able_to_finish_exact`,
    `the_full_rebuild_waits_for_a_page_holding_access_share`,
    `a_page_behind_the_full_rebuilds_truncate_is_retryable_or_sees_the_rebuilt_state`.
14. Readers: `a_dashboard_reader_holding_access_share_does_not_delay_a_step`.
15. Failure paths:
    `an_injected_error_before_any_statement_of_a_step_rolls_the_whole_step_back`,
    `lock_waits_behind_a_held_state_row_or_work_row_are_retryable_and_write_nothing`,
    `a_post_verify_mismatch_or_a_delete_count_mismatch_is_blocked_and_rolled_back`,
    `an_overflow_in_a_pre_verify_a_page_or_a_re_executed_insert_is_blocked`,
    `a_session_ended_by_the_idle_timeout_is_retryable_and_its_connection_is_discarded`,
    `an_exhausted_pool_is_retryable_within_the_checkout_wait_and_issues_no_statement`,
    `each_operation_takes_exactly_one_connection_with_a_bounded_wait`,
    `the_dispatch_and_status_reads_behind_an_exclusive_run_table_lock_are_retryable_within_a_second`,
    the row above `W` in test 7, the page serialization failure in test 4, the
    hold budget in test 16, and one test per classifier message
    (`a_serialization_failure_is_retryable`,
    `a_closed_connection_is_retryable`,
    `the_one_open_run_violation_is_run_open_and_any_other_unique_violation_is_internal`,
    `the_lock_timeout_message_is_retryable`,
    `the_statement_timeout_message_is_retryable`,
    `the_deadlock_message_is_retryable`,
    `the_idle_in_transaction_timeout_message_is_retryable`,
    `the_bigint_overflow_message_is_blocked`,
    `any_other_message_or_error_is_internal_and_never_retryable`,
    `the_server_produces_exactly_the_messages_the_classifier_matches`).
16. Hold bound: `the_state_row_and_work_row_holds_never_exceed_their_budgets`,
    `a_page_never_exceeds_its_budget`.
17. Range verifier:
    `the_range_verifier_over_everything_equals_the_whole_table_verification`,
    `the_range_verifier_shares_no_sql_with_the_writer_and_only_reads`,
    `the_rendered_whole_table_verification_statement_is_byte_identical`.
18. Pages write only the run row: `a_page_writes_only_the_run_row`.
19. Lock order and modes:
    `steps_and_pages_take_no_exclusive_lock_and_lock_works_then_state_then_run`.
20. More than 100,000 keys:
    `a_sparse_corpus_above_one_hundred_thousand_keys_builds_to_reconciled`.
21. GraphQL: `metric_rollup_tests.rs`
    `the_sdl_declares_exactly_the_eight_approved_operations`,
    `the_verification_and_rebuild_types_are_bounded_and_exactly_the_approved_shape`,
    `the_added_types_expose_progress_state_and_no_read_surface`,
    `the_run_operations_round_trip_through_the_resolvers`,
    `an_invalid_expected_step_count_is_rejected_by_the_resolver_without_any_statement`,
    `the_run_refusals_reach_the_caller_with_fixed_messages_and_no_identity`;
    `rebuild_run_tests.rs`
    `the_expected_step_count_is_a_bounded_decimal_string`.
22. Statement fidelity:
    `a_replacing_step_issues_the_completions_own_statements_with_one_deadline_each`.
23. Sizing: `sizing_cuts_at_each_budget_and_its_figure_equals_a_direct_count`,
    `the_run_grant_aggregate_returns_exactly_the_units_sizing_finds_over_a_step_budget`,
    `the_register_quotes_the_run_grant_aggregate_and_the_step_budgets_exactly`,
    `sizing_continues_after_a_full_batch_of_256_works_and_stops_on_a_short_one`,
    `a_slow_sizing_statement_is_retryable_within_the_presize_budget_before_any_work_lock`.
24. Re-size:
    `a_new_work_between_the_presize_and_the_resize_cuts_the_range_or_empties_it`.
25. Page statement sequence:
    `a_page_issues_only_set_local_then_its_table_lock_then_the_isolation_check`.

### Erratum R9.1 items and where they are met

1. Activation window and the latest-run check: register 3.8 "Activation
   window" (the evidence run's own advances and read-only status reads are
   allowed; no other run is started, advanced or abandoned, and no OPS-02 call
   is made); register 3.6 activation bullet and 4.2 (the check immediately
   before the restart, `completedAt` after the activation grant, the two
   reads for a `verifyMetricRollupMonths` marker, no clocks compared, run
   facts in PostgreSQL, an OPS-02 outcome only as the citation, a `VERIFY`
   run to `RECONCILED` preferred). Code: `LATEST_RUN_SQL` and `START_SQL`
   (`started_at` set by the insert) in `rebuild_run.rs`. Test 16.3:
   `the_status_read_prefers_the_open_run_and_otherwise_the_latest_started`.
2. The cost fit: the named model (intercept, statements per step, month term,
   year term, whole-work rows) fitted by exact non-negative least squares,
   corners at the largest measured statement count and the whole-work
   threshold, and the F3-F4-only fit beside the pooled fit (first run).
   Amendment R14 replaces it with the statement-group fits and keeps a
   six-term pooled fit for information only (next list, item 6).
3. F5's growths: each starts from a unit inside every step budget, at F4's
   section-row ratio (every row with a country and an institution distinct
   within its key-month), and the third growth runs at F3's four series
   (first run). Amendment R14 keeps three growths, all at F4's density,
   grown inside F2 (next list, item 9); register 3.8 "Evidence" states all
   three.
4. The run-grant aggregate: `METRIC_ROLLUP_RUN_GRANT_AGGREGATE_SQL` takes the
   step budgets as binds `$1` to `$3`; register 3.8 quotes the aggregate and
   the bound values; `the_register_quotes_the_run_grant_aggregate_and_the_step_budgets_exactly`
   pins both, and
   `the_run_grant_aggregate_returns_exactly_the_units_sizing_finds_over_a_step_budget`
   runs under injected budgets so that units over a budget exist. Register 3.8
   "Run grants and the run-grant aggregate" opens with the aggregate's shape.
5. Production numbers: register 3.8 "Floor" and "Whole-work threshold" say
   the registry pair count and the largest whole-work total come from the
   census's counts-only read or a separately authorized read, never from the
   implementation grant; that F3's total is the floor if production does not
   yet hold the legacy history; and that the run-grant aggregate stays the
   binding check. No production number was read for this report; F4 was run
   with k = 40 (section 10) and is judged once the pair count is stated on
   #1012.
6. Wording: the repair pass repeated without an intervening vacuum is report
   only (register 3.8 "The F4 line": dead entries are reported, not accepted
   against; the harness labels that pass "report only"); register 3.8 quotes
   the hold / fill line; "Limits" refers to F5's growths.

### Amendment R14 and erratum R14.1 items and where they are met

Names below are in `rebuild_run_tests.rs`, module `evidence`, unless stated.

1. The three budgets: `mod.rs` (`STEP_MAX_WORK_DAY_ROWS` 8,000,
   `STEP_MAX_REPLACING_READ` 600,000, `PAGE_MAX_WORK_DAY_ROWS` 150,000);
   register 3.8 quotes them with the grant's bind values, and
   `the_register_quotes_the_run_grant_aggregate_and_the_step_budgets_exactly`
   pins the constants and the quoted lines. No other constant, statement,
   index, state or invariant changed.
2. Pass order: `f2_census_corpus_with_f3_f4_and_f6` runs F2's from-empty
   build (F3 and F4 embedded), ANALYZE, the `EXPLAIN` line, F4's two repair
   passes, F2's corruption cycle to `RECONCILED`, then F6; only then mix (c)
   (`mix_c`, its works seeded, analysed, measured and deleted) and F5 (`f5`).
3. Diagnostic budgets per range: `DIAGNOSTIC` and `drive_measured`. A range
   refused under the section 10 budgets is measured once under them
   (`DIAGNOSTIC` lines), counts as a refusal of its class
   (`Driven::budget_refusals`), and its pass moves on. A refusal of mix (c)
   or of the second length counts only against that class.
4. ANALYZE after every fixture's build and before any `EXPLAIN`, after mix
   (c)'s seeding and at each F5 level (`analyse`, `growth_level`).
   `print_plan` reads each table's row count when the statement runs
   (`PLANTABLES` lines), counts a sequential scan only of a table holding
   rows, and reports scans of empty tables apart.
5. F2's ambiguity rows: on one key-day of about 2% of base month keys,
   `seed_shape` drops the row that carries neither a country nor an
   institution and keeps an institution row beside the country rows, so the
   cell has several masks and no unmasked row, as the section 16 ambiguity
   tests build them (`AMBIGUITY` line); F3, F4, mix
   (c) and F5 carry none. The `CORRUPTED` line reports how many of the 200
   corruptions applied.
6. F4 classes at F4's density: `dense` and `seed_shape` (mix (a) 8 series x
   80 rows per key-month x 8 years; the second length 5 x 128 x 14; mix (b)
   k = 39 x 16 x 1; mix (c) in `mix_c`). Fill is the smaller of row and
   figure fill (`Terms::fill`). Each class is reported apart (`mix_line`,
   `f4_work_facts`): k, rows per key-month, years, both fills, hold / fill,
   whole-work total and distinct countries.
7. Profile labels: `statement_label` and `step_label` profile the re-size
   (with its per-year sizing) apart from the pre-size; `LABEL_GROUPS` maps
   every label as section 10 states, and `print_label_map` prints the map
   (`LABELMAP` lines).
8. Statement-group fits: `Groups`, `fit` (exact non-negative least squares
   by `nnls`), `GroupFits` and `corner`. The walk's drivers are summed over a
   step's works (`Terms`, `walk_keys` and `walk_months`); walk, inserts and
   verify are fitted on the first repair pass of each F4 class, the rest on
   the from-empty replacing steps of F2, F3, F4 and F7 (`f7_groups` reads
   F7's own log). The six-term pooled fit is printed with its residuals for
   information only.
9. T: from F4's from-empty and first repair steps that committed within
   4 s and F5's units within 4 s, at least F3's whole-work total; after a
   second-length miss, from the classes that met both lines only, and F6
   runs without the second length's works (`SECONDLENGTH`, `THRESHOLD`).
10. F5: three growths at F4's density on chosen low work ids (`low_work`;
    each later growth sorts before every earlier one), without a TRUNCATE
    or the OPS-02 rebuild. Each level builds every year of its growth work,
    analyses, and takes the measured repair step; the level's run is
    abandoned once that work's range is replaced. Levels step coarsely, then
    by one row per key-month near each crossing; growth 2 starts at 3 rows
    per key-month at k = 39 (`run_growth`). Register 3.8 states each
    growth's limit on build and repair steps alike (section 5,
    deviations).
11. F6 on F2 with its F3 and F4 works, under the production budgets and the
    section 15 pacing, with the baseline run (`f6`).
12. Host facts: `host_facts` prints the version, `pg_config` configure,
    CFLAGS and LDFLAGS (installation prefix shown as `<prefix>`), each
    database's encoding, collate, ctype and locale provider, the settings
    not at their defaults (including `jit`) and the round-trip latency, in
    every run.
13. JIT per `EXPLAIN`ed statement from its `EXPLAIN (ANALYZE)` JIT lines
    (`PLANJIT` lines).
14. Register 3.8: the floor at R <= 13, mix (b)'s k, F5 dominance in all
    four of rows, keys, figure and whole-work rows with the whole-work
    bound, the three growths, T with its cap rule, the few-series limit, the
    run grant's collation and registry pair-count reads, and the re-run's
    evidence.

## 10. Manual verification

The section 17 evidence ran as the ignored tests of `rebuild_run_tests.rs`
(module `evidence`), one at a time, each on the local test database after
`setup()` emptied it:

```text
cargo test --workspace -- --exact --ignored --nocapture --test-threads 1 \
    model::metric_rollup_delta::rebuild_run_tests::evidence::<test>
```

with `THOTH_EVIDENCE_*` knobs where stated (the defaults are the
specification's sizes).

Every line below is printed by the harness; "hold" is the state-row hold of a
committed step (from the state-row lock to the end of the transaction), "page
time" runs from `BEGIN` to the end of a page's transaction, and "fill" is a
step's replacing-read figure over `STEP_MAX_REPLACING_READ`.

### 10.1 Evidence host

```text
PostgreSQL 16.15 (Ubuntu 16.15-0ubuntu0.24.04.1); round trip SELECT 1 p50 0.1 ms, p95 0.1 ms
4 CPUs, 15 GiB RAM, local disk; shared_buffers 128MB, work_mem 4MB, effective_cache_size 4GB,
max_wal_size 1GB, autovacuum on, jit on, random_page_cost 4
test binary: debug build (statement time is the server's)
```

The specification asks for the PostgreSQL major version production runs, and
for CI's as well if it differs. Production's version and host class were not
available to this stage, and CI's PostgreSQL 17 could not be installed on the
evidence host, so every number here is PostgreSQL 16 on this host only.

Correction (R14): an earlier version of this section said that no measured
plan used JIT. That is wrong. With `jit` on, one `EXPLAIN (ANALYZE)` of the
first run did: F4's mix (a) range verifier spent about 154 ms of its 449 ms
in JIT (249 functions; generation 11.9 ms, optimisation 6.9 ms, emission
134.8 ms), in the second run's log. JIT time is part of every hold and page
time measured in the first run. The re-run's cluster has `jit` off (section
10.7).

### 10.2 F1 (OPS-02 fixture, production budgets)

```text
FIXTURE|F1|1132043 work-day rows|24000 month keys|2000 works
F1 REBUILD from empty  replacing steps n=25  hold p50 1906.3 ms, p95 2044.1 ms, max 2051.9 ms
                       statements in the step's transaction 248 (median); sizing p95 113.0 ms
                       pages n=3  page time p50 3190.9 ms, p95 3298.7 ms
                       build 47.8 s, verify 7.8 s, 432 keys per second, no refusal
                       projected 1066 s at 460,000 keys, 2317 s at 1,000,000 keys
F1 healthy REBUILD     non-replacing steps n=25  hold p50 310.5 ms, p95 363.2 ms, max 363.8 ms
                       pages n=3  page time p50 2900.2 ms, p95 2933.5 ms; no refusal
                       build 8.4 s, verify 7.1 s; projected 297 s and 646 s
F1 VERIFY              pages n=3  page time p50 3001.8 ms, p95 3027.3 ms; no refusal
                       verify 7.3 s; projected 139 s and 302 s
statements per page 28 to 32
ACCEPT  verifyMetricRollupMonths exact after RECONCILED                         MET
ACCEPT  non-replacing step hold p95 <= 2 s     p95 363.2 ms, 0 budget refusals  MET
ACCEPT  replacing step hold p95 <= 3 s         p95 2044.1 ms, 0 budget refusals MET
ACCEPT  verify page time p95 <= 5 s (n=9)      p95 3298.7 ms, 0 budget refusals MET
```

Where a replacing F1 step's time went (one step, 1,000 keys): range verifier
(pre and post) 517 ms; month country inserts 462 ms; month institution
inserts 343 ms; month total inserts 137 ms; month ambiguity inserts 116 ms;
year institution and country inserts 158 ms; sizing 85 ms; month watermarks
41 ms; 127 `SET LOCAL` 29 ms.

### 10.3 F7 (sparse corpus, production budgets)

```text
FIXTURE|F7|100000 works|100000 month keys, one per unit
F7 REBUILD from empty  replacing steps n=100  hold p50 236.7 ms, p95 281.0 ms, max 304.4 ms
                       332 statements in the step's transaction; sizing p95 41.3 ms
                       pages n=11  page time p50 260.4 ms, p95 324.3 ms
                       build 27.6 s, verify 2.7 s, 3293 keys per second, no refusal
                       projected 140 s at 460,000 keys, 304 s at 1,000,000 keys
F7 healthy REBUILD     non-replacing steps n=101  hold p50 34.2 ms, p95 42.8 ms, max 59.2 ms
                       pages n=11  page time p95 351.9 ms; no refusal
                       build 7.0 s, verify 2.8 s; projected 45 s and 98 s
statements per page 26 to 104 (a sparse page sizes about 10,000 single-key works)
ACCEPT  non-replacing step hold p95 <= 2 s     p95 42.8 ms, 0 budget refusals   MET
ACCEPT  replacing step hold p95 <= 3 s         p95 281.0 ms, 0 budget refusals  MET
ACCEPT  verify page time p95 <= 5 s (n=22)     p95 324.3 ms, 0 budget refusals  MET
```

### 10.4 F5 (growths, production budgets, repair steps over fully built tables)

Each level is one work at F4's section-row ratio (every row has its own
country and institution within its key-month), damaged in every unit and
repaired by a `REBUILD`. A level stops the growth when a step is refused at
`STEP_HOLD_BUDGET` (two refusals in a row) or holds more than 5 s.

```text
growth 1: 40 series, one year (unit = whole work)
  rows/key-month  unit rows  keys  figure     whole rows  replacing steps  max hold
  10              4,800      480   384,000    4,800       1                1171.0 ms
  20              9,600      480   768,000    9,600       1                2156.1 ms
  30              14,400     480   1,152,000  14,400      1                3024.5 ms  <- largest within 4 s
  40              19,200     480   1,536,000  19,200      1                4177.5 ms
  60              28,800     480   2,304,000  28,800      0                refused twice at 5.0 s
growth 2: 40 series, eight years, every year the same rows
  10              4,800      480   1,728,000  38,400      4                3392.8 ms
  20              9,600      480   3,456,000  76,800      8                3161.1 ms  <- largest within 4 s
  30              14,400     480   5,184,000  115,200     8                4581.9 ms
  40              19,200     480   6,912,000  153,600     0                refused twice at 5.0 s
growth 3: F3's four series, eight years
  10              480        48    17,280     3,840       1                1064.2 ms
  20              960        48    34,560     7,680       1                1983.0 ms
  30              1,440      48    51,840     11,520      1                2816.0 ms
  40              1,920      48    69,120     15,360      1                3577.4 ms  <- largest within 4 s
  60              2,880      48    103,680    23,040      0                refused twice at 5.0 s
THRESHOLD|F5|largest whole-work total of a replacing step within 4 s|76800
```

Sizing of the largest unit's work within 4 s: growth 1 16.3 ms, growth 2 p95
258.3 ms, growth 3 16.1 ms. In growth 3 one step took the whole work (all
eight units, 15,360 rows), so its hold is that step's.

### 10.5 F2 with F3 and F4 embedded (full scale, diagnostic hold budget)

Under the production budgets the F2 build cannot finish on this host: every
F4 step is refused at the 5 s hold budget on every retry (the probe below, and
the first reduced-scale run, which stopped after 20 consecutive refusals at
the first F4 mix (b) range), and F3's seven-unit steps, which held 7.7 to 8.9
s, cannot commit inside it either. So that every step's real hold could be
measured, F2 ran with the step hold budget raised to 120 s
(`THOTH_EVIDENCE_STEP_HOLD_SECS=120`); every other budget was production's.
Each acceptance line counts a hold above 5 s as a budget refusal, so the
relaxed budget cannot make a line pass.

F2 ran twice in full on the same fixture. Run 1 stopped after its repair
`REBUILD` (see "Whole-table verification at this size" below), so run 2
repeated every pass. The per-step detail below is run 1's; the acceptance lines, the
whole-work threshold, the fit and the probe are run 2's, and run 2's
summaries are given beside run 1's.

Fixture (seeded and analysed in 584 s; disk before the run 16 GB free, the
database 8.5 GB after the from-empty build):

```text
books     2000 works  12,348,000 rows  392,000 keys  196 keys/work  31.5 rows/key  15% key-days with an institution  40-160 countries/work
chapters  2600 works   3,439,796 rows  109,200 keys   42 keys/work  31.5 rows/key  15%                                   40-121 countries/work
F3           4 works     227,638 rows    1,536 keys  ~56,910 rows/work, 8 years, 4 series, 160 countries/work
F4 mix (a)   2 works     276,480 rows    4,608 keys  24 keys per work-month, 8 years, 60 rows per key-month,
                                                     138,240 country and 138,240 institution month rows per work
F4 mix (b)   8 works     384,000 rows    3,840 keys  k = 40 keys per work-month, 1 year, 100 rows per key-month,
                                                     48,000 country and 48,000 institution month rows per work
F2 total 16,675,914 work-day rows, 511,184 month keys, one shared registry of 48 series
FLOOR  F2 density 31.50 rows per key: floor figure 3402 R^2, STEP_MAX_REPLACING_READ admits R <= 34
SIZE   after the from-empty build: work_day 5495 MB, country_month 1465 MB, country_year 695 MB,
       institution_month 419 MB, institution_year 245 MB, month 127 MB; database 8522 MB
```

REBUILD from empty in run 1 (573 steps, 52 pages, build 1,801 s, verify 157
s, 261 keys per second, projected 1,762 s at 460,000 keys and 3,830 s at
1,000,000 keys; no refusal under the relaxed budget; 26 statements per page):

```text
class  replacing steps  hold p50     p95        max        above 3 s  above 5 s  statements in the transaction (median)
base   541              2734.9 ms    3145.2 ms  3572.0 ms  77         0          238
F3     8                7695.7 ms*   7959.7 ms  7959.7 ms  8          4          228
F4a    16               10459.1 ms   11646.5 ms 11759.9 ms 16         16         108
F4b    8                12182.9 ms   12493.9 ms 12493.9 ms 8          8          144
(* F3's p50 sits between its four seven-unit steps, 7.7-8.0 s, and its four mixed steps, 3.6-3.9 s)
pages  base n=42 p50 2831.2 ms p95 3042.2 ms; F3 n=4 p95 4541.2 ms; F4a n=2 max 6935.2 ms; F4b n=4 p95 6336.7 ms
sizing per step: base p95 68.7 ms; F3 p95 172.0 ms; F4a p95 397.4 ms; F4b p95 155.5 ms
```

Run 2's from-empty build (build 1,841 s, verify 152 s, the same 573 steps
and 52 pages, no refusal):

```text
class  replacing steps  hold p50     p95        max        above 3 s  above 5 s
base   541              2809.6 ms    3242.4 ms  3806.4 ms  132        0
F3     8                7671.5 ms    8852.8 ms  8852.8 ms  8          4   (seven-unit steps 7.7-8.9 s, mixed 3.4-4.1 s)
F4a    16               10020.8 ms   11056.0 ms 11080.2 ms 16         16  (max hold / fill 11.3 s)
F4b    8                12264.8 ms   12445.1 ms 12445.1 ms 8          8   (max hold / fill 12.9 s)
pages  base n=42 p95 3077.8 ms; F3 n=4 p95 4381.7 ms; F4a n=2 max 6752.5 ms; F4b n=4 p95 3838.3 ms
```

F4's replacing steps from empty, one row per step (rows and keys of the
step; month and year terms of its figure; the whole-work rows it reads;
statements in its transaction):

```text
mix  hold        fill   hold/fill   rows    keys  month term  year term  whole rows  statements
F4a   9543.4 ms  0.933  10228.7 ms  17280   288    414720     3317760    138240      108
F4a   9807.3 ms  0.933  10511.6 ms  17280   288    414720     3317760    138240      108
F4a   9939.4 ms  0.933  10653.2 ms  17280   288    414720     3317760    138240      108
F4a   9996.5 ms  0.933  10714.4 ms  17280   288    414720     3317760    138240      108
F4a  10028.5 ms  0.933  10748.7 ms  17280   288    414720     3317760    138240      108
F4a  10146.3 ms  0.933  10874.9 ms  17280   288    414720     3317760    138240      108
F4a  10194.4 ms  0.933  10926.5 ms  17280   288    414720     3317760    138240      108
F4a  10365.0 ms  0.933  11109.3 ms  17280   288    414720     3317760    138240      108
F4a  10459.1 ms  0.933  11210.2 ms  17280   288    414720     3317760    138240      108
F4a  10480.3 ms  0.933  11232.9 ms  17280   288    414720     3317760    138240      108
F4a  10494.3 ms  0.933  11247.9 ms  17280   288    414720     3317760    138240      108
F4a  10522.5 ms  0.933  11278.1 ms  17280   288    414720     3317760    138240      108
F4a  10579.8 ms  0.933  11339.5 ms  17280   288    414720     3317760    138240      108
F4a  10781.0 ms  0.933  11555.2 ms  17280   288    414720     3317760    138240      108
F4a  11646.5 ms  0.982  11860.0 ms  31390   736    455288     3472984    152350      198
F4a  11759.9 ms  0.974  12073.8 ms  26099   568    444702     3451788    147059      168
F4b  11750.6 ms  0.960  12240.2 ms  48000   480   1920000     1920000     48000      144
F4b  11760.7 ms  0.962  12225.3 ms  49322   522   1922644     1925288     49322      154
F4b  11866.2 ms  0.960  12360.6 ms  48000   480   1920000     1920000     48000      144
F4b  11922.8 ms  0.960  12419.6 ms  48000   480   1920000     1920000     48000      144
F4b  12182.9 ms  0.960  12690.5 ms  48000   480   1920000     1920000     48000      144
F4b  12331.6 ms  0.960  12845.4 ms  48000   480   1920000     1920000     48000      144
F4b  12352.2 ms  0.962  12840.1 ms  49320   522   1922640     1925280     49320      154
F4b  12493.9 ms  0.960  13014.5 ms  48000   480   1920000     1920000     48000      144
```

Where a step's time went (one step of each class, from empty; hold, then the
largest statement groups, each the sum over the step's chunks):

```text
base 2518 ms (24 units, 868 keys, 27,346 rows, fill 0.152)
     month country inserts 642 ms; range verifier (pre and post) 402 ms; month institution 312 ms;
     month total 291 ms; month ambiguity 261 ms; month watermarks 248 ms; year inserts 293 ms
F3 7696 ms (7 units of one work, 336 keys, 49,798 rows, fill 0.448)
     month country inserts 2875 ms; month institution 926 ms; month total 868 ms;
     month ambiguity 841 ms; range verifier 821 ms; month watermarks 800 ms; year inserts 410 ms
F4a 9939 ms (1 unit, 288 keys, 17,280 rows, fill 0.933)
     month institution inserts 2355 ms; month country 2265 ms; month watermarks 1445 ms;
     month ambiguity 1403 ms; month total 1384 ms; range verifier 478 ms; year inserts 422 ms
F4b 12183 ms (1 unit, 480 keys, 48,000 rows, fill 0.960)
     month institution inserts 3734 ms; month country 3291 ms; range verifier 1534 ms;
     month total 872 ms; month ambiguity 852 ms; month watermarks 825 ms; year inserts 941 ms
```

`EXPLAIN (ANALYZE, BUFFERS)` of the statements as executed, on a median
replacing step of each class:

- Sizing, `RANGE_MONTH_KEYS_SQL`, the six range deletes and the range
  verifier read the work-day projection through
  `metric_rollup_work_day_work_id_day_idx` or
  `metric_rollup_work_day_identity_key`, led by `work_id`; none used the
  series index (MET).
- Eight sequential scans, all of `metric_rollup_work_month_ambiguity`, which
  F2 leaves empty (cost 0.00, 0 rows): range delete 4 and the range verifier
  in each class. Every populated projection table was read through its
  identity key (MISSED as written; question in section 14).
- The completion's month watermark and month inserts read the work-day
  projection through `metric_rollup_work_day_identity_key` with only
  `work_id` bounding the scan (`publication_id`, its second column, is
  unbound), so each month key walks its work's whole history: an F4 mix (a)
  month total insert touched 153,400 buffers for 50 keys of 60 rows each; an
  F3 month country insert 54,205 for 50 keys of 148 rows.
- Of a month country insert's time, the foreign-key checks of the inserted
  section rows took 237 ms of 539 ms (F3, 5,800 rows) and 204 ms of 389 ms
  (F4 mix (b), 5,000 rows).

F4's repair passes damaged one row in each of the 24 F4 units and ran a
`REBUILD` over the fully built tables. Autovacuum had not run on any
projection table between the from-empty build and the first repair pass
(every autovacuum counter unchanged; the dead tuples then present were left by
the `EXPLAIN (ANALYZE)` runs); it ran on the two institution tables during
that pass, and on the month country and institution tables during the repeated
pass. F4's replacing steps in every pass of run 1:

```text
pass                                   mix  steps  hold            fill         max hold / fill
REBUILD from empty                     F4a  16      9.5 - 11.8 s   0.933-0.982  12.1 s
                                       F4b   8     11.8 - 12.5 s   0.960-0.962  13.0 s
F4 repair pass                         F4a  16     10.2 - 12.2 s   0.933-0.982  12.4 s
                                       F4b   8     13.2 - 14.1 s   0.960-0.962  14.7 s
repair repeated without vacuum         F4a  16     10.2 - 11.6 s   0.933-0.982  11.9 s
  (report only)                        F4b   8     12.2 - 13.5 s   0.960-0.962  14.1 s
F2 repair REBUILD (corrupted ranges)   F4a   6     10.3 - 11.9 s   0.933-0.982  12.1 s
                                       F4b   8     12.5 - 13.6 s   0.960-0.962  14.2 s
```

In the F4 repair passes every base and F3 step was non-replacing (base p95
447 ms and 442 ms, F3 p95 621 ms and 762 ms), each pass took 573 steps and 52
pages (build 460 s and 452 s, verify 158 s and 155 s), and the pages were as
in the from-empty build.

Corruption and repair. Of the 200 scattered corruptions, 167 were applied: F2
holds no ambiguity row, so the ambiguity table's share had nothing to corrupt.
The others went to the month (34), month country (34), month institution
(33), year country (33) and year institution (33) tables, about half as
deleted rows and half as changed values. The `VERIFY` run (52 pages, 160.8 s,
base page p95 3,251 ms) ended `MISMATCHED` with, per family, expected /
actual / missing / extra / mismatched rows:

```text
total             511,184 /   511,167 / 17 / 0 / 17
country         5,850,656 / 5,850,639 / 17 / 0 / 17
institution     1,418,730 / 1,418,714 / 16 / 0 / 17
ambiguity               0 /         0 /  0 / 0 /  0
country year    2,754,752 / 2,754,736 / 16 / 0 / 17
institution year  851,685 /   851,669 / 16 / 0 / 17
```

The repair `REBUILD` ended `RECONCILED` (573 steps, build 709 s, verify 158
s): 126 base steps replaced their ranges (hold p50 2,978 ms, p95 3,445 ms,
max 3,656 ms) and 415 were non-replacing (p95 447 ms); F3 replaced 3 ranges
(4.0 to 8.5 s) and F4 14 (above).

Whole-table verification at this size. Once that repair `REBUILD` had
reached `RECONCILED`, run 1's harness called `verifyMetricRollupMonths`, which
exceeded its 30 s statement timeout and failed the test before its acceptance,
threshold, fit and probe lines. That timeout is expected here: register 3.7
bounds the OPS-02 pair at 100,000 keys or a whole-table verification near its
30 s statement, past which section 3.8 is the supported path. The harness now
reports the outcome instead of failing (commit 4), and run 2 repeated F2 in
full on the same fixture (the seeding is deterministic). Outside the harness,
the same comparison statement (as the server logged it) then ran on run 1's
end state in one `READ ONLY`, `REPEATABLE READ` transaction without the 30 s
timeout: with the default `work_mem` of 4 MB it was stopped after 87 s by a
4 GB `temp_file_limit` set to protect the host's disk; with `work_mem` 512 MB
and no parallel workers it completed in 97 s and found all six families exact
(total 511,184, country 5,850,656, institution 1,418,730, ambiguity 0, country
year 2,754,752 and institution year 851,685 rows, none missing, extra or
mismatched, and no watermark above the frontier 16,675,914).

Run 2 repeated every pass on the same fixture. Its F4 steps, with run 1's
(hold / fill from the logged fill, rounded to three decimals, for run 1; as
the harness computed it for run 2):

```text
pass                                   mix  run 1: hold        max h/f   run 2: hold        max h/f
REBUILD from empty                     F4a   9.5 - 11.8 s      12.1 s     9.4 - 11.1 s      11.3 s
                                       F4b  11.8 - 12.5 s      13.0 s    12.0 - 12.4 s      12.9 s
F4 repair pass                         F4a  10.2 - 12.2 s      12.4 s     9.9 - 11.7 s      11.9 s
                                       F4b  13.2 - 14.1 s      14.7 s    12.9 - 13.7 s      14.2 s
repair repeated without vacuum         F4a  10.2 - 11.6 s      11.9 s     9.8 - 11.2 s      11.4 s
  (report only)                        F4b  12.2 - 13.5 s      14.1 s    12.4 - 13.1 s      13.7 s
```

Run 2's other passes:

```text
F4 repair pass            base 541 non-replacing, hold p50 268.3 ms, p95 440.6 ms, max 639.4 ms
                          F3 8 non-replacing, p95 595.8 ms; build 447.9 s, verify 157.5 s
repeated (report only)    base p95 451.0 ms; F3 p95 584.9 ms; build 442.6 s, verify 169.2 s
VERIFY after corruptions  52 pages, 172.8 s, base page p95 3492.3 ms, max 5112.1 ms; MISMATCHED,
                          family counts identical to run 1's
repair REBUILD            RECONCILED, 573 steps, build 724.2 s, verify 157.4 s
                          base 121 replacing (p50 3017.9 ms, p95 3592.1 ms, max 3724.9 ms), 420 non-replacing
                          (p95 474.6 ms); F3 3 replacing (4.7 to 8.4 s); F4a 9 and F4b 7 replacing (10.4 to 14.2 s)
WHOLE                     verifyMetricRollupMonths exceeded its 30 s statement timeout after 33.8 s
```

The two runs corrupted the same number of rows per table but not the same
rows: the picks follow the projection rows' surrogate keys, which each build
generates afresh, so the ranges the repair `REBUILD` replaced differ.

Acceptance lines, run 2, as the harness printed them (a hold above 5 s,
committed under the relaxed budget, counts as a budget refusal):

```text
F2 non-replacing step hold p95 <= 2 s               n=1502  p50 272.1 ms  p95 453.4 ms   max 639.4 ms   MET
F2 replacing step hold p95 <= 3 s (from empty)      n=541   p50 2809.6 ms p95 3242.4 ms  max 3806.4 ms  MISSED
F3 non-replacing step hold p95 <= 2 s               n=21    p50 489.8 ms  p95 595.8 ms   max 651.5 ms   MET
F3 replacing step hold p95 <= 3 s (from empty)      n=8     p50 7671.5 ms p95 8852.8 ms  4 above 5 s    MISSED
F2 replacing step hold p95 <= 3 s (its repair run;  n=124   p50 3025.2 ms p95 3668.3 ms  max 8434.5 ms
   F2 and F3 ranges)                                                                     1 above 5 s    MISSED
F2 repair run, every replacing step including F4    n=140   p50 3104.2 ms p95 11581.2 ms max 14150.7 ms
   ranges (report only)                                                                  17 above 5 s
F2 verify page time p95 <= 5 s                      n=210   p50 2816.8 ms p95 3264.4 ms  max 5112.1 ms  MET
F3 verify page time p95 <= 5 s                      n=20    p50 3321.0 ms p95 4698.1 ms  max 4810.7 ms  MET
F4a verify page time p95 <= 5 s                     n=10    p50 6615.6 ms p95 8051.9 ms                 MISSED
F4b verify page time p95 <= 5 s                     n=20    p50 3826.8 ms p95 6400.9 ms  max 6616.0 ms  MISSED
F4 mix (a) from-empty steps: max hold / fill <= 4 s n=16  fill 0.933-0.982  11349.8 ms  16 above 5 s  MISSED
F4 mix (a) repair steps                             n=16  fill 0.933-0.982  11884.2 ms  16 above 5 s  MISSED
F4 mix (a) repair repeated without vacuum           n=16  fill 0.933-0.982  11429.8 ms  (report only)
F4 mix (b) from-empty steps: max hold / fill <= 4 s n=8   fill 0.960-0.962  12937.0 ms  8 above 5 s   MISSED
F4 mix (b) repair steps                             n=8   fill 0.960-0.962  14222.8 ms  8 above 5 s   MISSED
F4 mix (b) repair repeated without vacuum           n=8   fill 0.960-0.962  13694.1 ms  (report only)
EXPLAIN: no sequential scan of metric_rollup_work_day or a projection table
                                                    8 found, all of the empty ambiguity table      MISSED
EXPLAIN: sizing, RANGE_MONTH_KEYS_SQL, the range deletes and the range verifier led by work_id   MET
```

No pass of either run was refused under the relaxed budget, so every "budget
refusal" above is a committed hold above 5 s.

Whole-work threshold (run 2):

```text
THRESHOLD|F4 largest whole-work total of a replacing step within 4 s 0|F5 76800|whole-work threshold 76800|
          F3 works' totals [56907, 56912, 56910, 56909]
```

No F4 replacing step committed within 4 s, so F4 contributes no total and
F5's second growth sets the threshold at 76,800 rows, above F3's totals (the
census maximum). Production's largest whole-work total was not read by this
stage.

The cost fit (run 2; non-negative least squares of the hold on an intercept,
the statements in the step's transaction, the month term, the year term and
the step's whole-work rows; corners at full fill, at the largest measured
statement count, 248, and at the threshold's 76,800 whole-work rows):

```text
pooled F2-F4 (n=737: every replacing step of the from-empty build, the F4 repair pass and the repair REBUILD)
  intercept 572.027 ms, per statement 2.8889 ms, per month-term row 3.9136e-3 ms,
  per year-term row 2.1154e-3 ms, per whole-work row 6.2732e-3 ms
  absolute residual median 126.8 ms, max 3432.5 ms
  year-only corner (month 0, year 4,000,000)            predicted hold 10231.8 ms  bound 4500 ms  MISSED
  mix (b) corner (month 2,000,000 = 40 x 50,000, year 2,000,000)  13828.3 ms        bound 4500 ms  MISSED
F3-F4 only (n=75)
  intercept 3239.516 ms, per statement 0, per month-term row 3.2814e-3 ms,
  per year-term row 1.7372e-3 ms, per whole-work row 0
  absolute residual median 418.8 ms, max 2191.4 ms
  year-only corner 10188.4 ms; mix (b) corner 13276.7 ms
```

The per-step residuals are in the evidence log (`RESIDUAL` lines).

The probe (run 2): after one row was damaged in each F4 unit, a `REBUILD`
under the production budgets committed 107 non-replacing steps (105 base and
2 F3, hold p95 337.2 ms) up to the first F4 range, a mix (b) range, which was
then refused with `statement_timeout` at 5.0 s three times in a row; each
attempt had run its pre-verify (0.8 to 1.0 s) and about four of its chunks of
month statements (month institution and month country inserts 1.4 to 1.7 s
in total each) when the hold budget ran out. The probe stops after three
consecutive refusals; it abandoned the run, and the damage was then repaired
under the relaxed budget (`RECONCILED`, 24 ranges replaced, build 459 s,
verify 158 s). So under the production budgets a `REBUILD` of F2 does not get
past its first F4 range on this host. Run 2 completed in 152 minutes with
every assertion passing.

### 10.6 F6 (concurrency, production budgets)

F6 uses the production budgets, under which a run over the whole of F2 stops
at the first F3 or F4 range on this host (section 10.5), so F6 ran on F2
without its F3 and F4 works (`THOTH_EVIDENCE_F3_WORKS=0`,
`THOTH_EVIDENCE_F4A_WORKS=0`, `THOTH_EVIDENCE_F4B_WORKS=0`, and
`THOTH_EVIDENCE_F2_PASSES=0` to skip F2's own passes): 2,000 book and 2,600
chapter works, 15,787,796 work-day rows and 501,200 month keys over the same
registry of 48 series (seeded and analysed in 581 s). A setup `REBUILD` under
the relaxed hold budget built every projection (`RECONCILED`, build 1,531 s,
verify 135 s).

The workload runs on 80 of F2's book works in two sets of 40:

- two `DAY` ingestion streams, each a loop of canonical batches (a new `DAY`
  record, its current revision and its `PENDING` delta, whose insert takes
  the state row) for one or two works of the current set, the works locked
  `FOR UPDATE` in `work_id` order first, under the ingestion's 30 s statement
  timeout, 25 ms apart; the two streams pick overlapping works, so a batch
  can wait behind the other stream's batch and waits can be transitive;
- one completion stream, claiming and completing only the previous set's
  deltas;
- a coordinator that, every 20 s, waits until the completions have drained
  the previous set and then switches the sets, so ingestion and completions
  are on disjoint works at every instant;
- a sampler that reads `pg_blocking_pids()` of every workload backend every
  100 ms and attributes a wait to a step when the step's backend blocks it
  directly or appears anywhere in its blocking chain. A wait lasts from the
  first sample that saw it to the last, plus one interval, so 100 ms is the
  resolution and a wait shorter than one interval may go unseen.

The phases: the workload alone for 300 s, without a run (the baseline); then,
with the projections truncated, a `REBUILD` from empty under the production
budgets, paced by the section 15 rule (a pause of `max(1 s, 2 x the call's
elapsed time)` after every advance call made while `BUILDING`), with the
workload running throughout its build phase; then, with the workload stopped
and its deltas applied, the run's verify phase under the production budgets
and a `verifyMetricRollupMonths` call.

```text
baseline, 300 s without a run
  workload  18,013 ingestion batches, 35,725 works ingested, 676 claims, 33,480 deltas completed; no failure
  waits     state row (DAY inserts, claims, completions), all causes  n=723   p50 100.0 ms  p95 100.0 ms   max 200.2 ms
            ingestion work row, all causes                             n=26    p50 100.0 ms  p95 100.0 ms   max 100.0 ms
            ingestion statements, all causes                           n=722   p50 100.0 ms  p95 100.0 ms   max 200.2 ms
paced REBUILD from empty, build phase (production budgets, 4,835 s)
  steps     542 replacing; hold p50 2929.0 ms, p95 3448.2 ms, max 3985.9 ms; call p95 3476.2 ms, max 4018.9 ms;
            sizing p95 66.0 ms; 238 statements in the transaction (median); 104 keys per second
  refusals  none: by reason {}, longest run of consecutive refusals 0, no hold above 5 s
  workload  185,668 ingestion batches, 368,240 works ingested, 7,453 claims, 366,828 deltas completed; no failure
  waits     state row, all causes                                      n=9791  p50 100.0 ms  p95 3004.8 ms  max 3913.9 ms
            ingestion work row, all causes                             n=567   p50 100.0 ms  p95 2904.7 ms  max 3306.7 ms
            ingestion statements, all causes                           n=9907  p50 100.0 ms  p95 2906.4 ms  max 3913.9 ms
verify phase after the workload: 51 pages, page time p50 2862.0 ms, p95 3253.1 ms, max 3404.2 ms; RECONCILED in 143.4 s
WHOLE  verifyMetricRollupMonths exceeded its 30 s statement timeout after 33.7 s
```

The acceptance lines, with the harness's values and verdicts:

```text
F6 zero ingestion or completion failures                        0 and 0                                         MET
F6 no deadlock report naming a step's backend                   step refusals for a deadlock 0,
                                                                workload deadlocks 0                            MET
F6 direct state-row waits attributable to steps, max <= 6 s     n=557   p50 2905.3 ms  p95 3406.6 ms  max 3913.9 ms  MET
F6 direct ingestion work-row waits attributable to steps,       n=3     p50 2906.4 ms  p95 3005.6 ms  max 3005.6 ms  MET
   max <= 8 s
F6 ingestion statement waits with a step anywhere in the        n=1086  p50 2905.4 ms  p95 3407.0 ms  max 3913.9 ms  MET
   blocking chain (whole wait), max <= 13 s
F6 the build phase completes                                    VERIFYING after 542 steps and 4,835 s, no refusal  MET
```

Next to the baseline, whose longest workload wait was 0.2 s (two sampling
intervals), a wait caused by a step lasted about one step hold (p95 3.4 s,
max 3.9 s). Of the waits a step caused directly, 557 were on the state row
and 3 on a work row: a step locks only the works of its own range
`FOR KEY SHARE`, and the workload's 80 works lie in few of the 542 ranges.
The longest wait with a step anywhere in its blocking chain (3.9 s) was no
longer than the longest direct one, well inside the 13 s bound and section
9's "about 12 s". With the pacing pauses the build phase took 4,835 s,
against 1,531 s for the setup `REBUILD`, which ran unpaced and without the
workload. F6 has no hold line of its own; its replacing steps held a little
longer than F2's from-empty steps (p95 3.4 s against 3.2 s) with the workload
running beside them. The unpaced run, which section 17 reports as
information only, was not run. The per-step lines are in the evidence log
(`RAW` and `PROFILE` lines).

### 10.7 Re-run under amendment R14 and erratum R14.1 (PostgreSQL 17)

Every section 17 line was re-run with the R14 budgets and the R14.1
harness (commit 7 of section 3), from the committed tree, one test at a time
on the local test database, in this order: F1, F7, then F2 with F3, F4, F6,
mix (c) and F5 embedded in one test (`THOTH_EVIDENCE_F7_LOG` names F7's log,
which carries the F7 steps of the rest fit). No knob was set: every fixture
ran at the specified size, every pass under the section 10 budgets.

#### 10.7.1 Evidence host, build and cluster

```text
HOST|PostgreSQL 17.10 (170010)|round trip SELECT 1 p50 0.1 ms, p95 0.1 ms, max 0.1 ms|cpus 4|
    shared_buffers 128MB|work_mem 4MB|effective_cache_size 4GB|max_wal_size 1GB|autovacuum on|
    random_page_cost 4|jit off
HOSTJIT|jit off|jit_provider llvmjit|pg_jit_available() false (false whenever jit is off)|
    pg_jit_available() with jit set on in a transaction true
HOSTDB|thoth_test: encoding UTF8, collate C.UTF-8, ctype C.UTF-8, locale provider libc
    (postgres, template0, template1 and thoth the same)
HOSTSETTINGS|settings not at their defaults: those initdb writes (DateStyle, TimeZone,
    default_text_search_config, dynamic_shared_memory_type, lc_messages, lc_monetary,
    lc_numeric, lc_time = C.UTF-8, log_timezone, max_connections 100, max_wal_size 1024 MB,
    min_wal_size 80 MB, shared_buffers 128 MB), port 5433 and jit = off
```

The same host as the first run (4 CPUs, 15 GiB RAM, local disk), with CI's
PostgreSQL major version (17): 17.10 built from the verified official
release source with the first run's distribution compiler and linker flags.
Both builds' `pg_config` (the installation prefix shown as `<prefix>`):

```text
PostgreSQL 17.10 (the re-run)
  --configure '--prefix=<prefix>' '--without-readline' '--with-icu' '--with-llvm'
    'LLVM_CONFIG=/usr/bin/llvm-config-18' 'CLANG=/usr/bin/clang-18'
    'CPPFLAGS=-Wdate-time -D_FORTIFY_SOURCE=3'
    'CFLAGS=-g -O2 -fno-omit-frame-pointer -mno-omit-leaf-frame-pointer -flto=auto
     -ffat-lto-objects -fstack-protector-strong -fstack-clash-protection -Wformat
     -Werror=format-security -fcf-protection -fno-omit-frame-pointer'
    'LDFLAGS=-Wl,-Bsymbolic-functions -flto=auto -ffat-lto-objects -Wl,-z,relro -Wl,-z,now
     -Wl,--as-needed'
  --cflags -Wall -Wmissing-prototypes -Wpointer-arith -Wdeclaration-after-statement
    -Werror=vla -Wendif-labels -Wmissing-format-attribute -Wimplicit-fallthrough=3
    -Wcast-function-type -Wshadow=compatible-local -Wformat-security -fno-strict-aliasing
    -fwrapv -fexcess-precision=standard -Wno-format-truncation -Wno-stringop-truncation -g -O2
    -fno-omit-frame-pointer -mno-omit-leaf-frame-pointer -flto=auto -ffat-lto-objects
    -fstack-protector-strong -fstack-clash-protection -Wformat -Werror=format-security
    -fcf-protection -fno-omit-frame-pointer
  --ldflags -Wl,-Bsymbolic-functions -flto=auto -ffat-lto-objects -Wl,-z,relro -Wl,-z,now
    -Wl,--as-needed -Wl,--as-needed -Wl,-rpath,'<prefix>/lib',--enable-new-dtags

PostgreSQL 16.15 (Ubuntu 16.15-0ubuntu0.24.04.1) (the first run)
  --configure '--build=x86_64-linux-gnu' '--prefix=/usr' '--includedir=${prefix}/include'
    '--mandir=${prefix}/share/man' '--infodir=${prefix}/share/info' '--sysconfdir=/etc'
    '--localstatedir=/var' '--disable-option-checking' '--disable-silent-rules'
    '--libdir=${prefix}/lib/x86_64-linux-gnu' '--runstatedir=/run'
    '--disable-maintainer-mode' '--disable-dependency-tracking' '--with-tcl' '--with-perl'
    '--with-python' '--with-pam' '--with-openssl' '--with-libxml' '--with-libxslt'
    '--mandir=/usr/share/postgresql/16/man' '--docdir=/usr/share/doc/postgresql-doc-16'
    '--sysconfdir=/etc/postgresql-common' '--datarootdir=/usr/share/'
    '--datadir=/usr/share/postgresql/16' '--bindir=/usr/lib/postgresql/16/bin'
    '--libdir=/usr/lib/x86_64-linux-gnu/' '--libexecdir=/usr/lib/postgresql/'
    '--includedir=/usr/include/postgresql/'
    '--with-extra-version= (Ubuntu 16.15-0ubuntu0.24.04.1)' '--enable-nls'
    '--enable-thread-safety' '--enable-debug' '--enable-dtrace' '--disable-rpath'
    '--with-uuid=e2fs' '--with-gnu-ld' '--with-gssapi' '--with-ldap' '--with-pgport=5432'
    '--with-system-tzdata=/usr/share/zoneinfo' 'AWK=mawk' 'MKDIR_P=/bin/mkdir -p'
    'PROVE=/usr/bin/prove' 'PYTHON=/usr/bin/python3' 'TAR=/bin/tar'
    'XSLTPROC=xsltproc --nonet'
    'CFLAGS=-g -O2 -fno-omit-frame-pointer -mno-omit-leaf-frame-pointer -flto=auto
     -ffat-lto-objects -fstack-protector-strong -fstack-clash-protection -Wformat
     -Werror=format-security -fcf-protection -fno-omit-frame-pointer'
    'LDFLAGS=-Wl,-Bsymbolic-functions -flto=auto -ffat-lto-objects -Wl,-z,relro -Wl,-z,now'
    '--enable-tap-tests' '--with-icu' '--with-llvm' 'LLVM_CONFIG=/usr/bin/llvm-config-17'
    'CLANG=/usr/bin/clang-17' '--with-lz4' '--with-zstd' '--with-systemd' '--with-selinux'
    'build_alias=x86_64-linux-gnu' 'CPPFLAGS=-Wdate-time -D_FORTIFY_SOURCE=3'
    'CXXFLAGS=-g -O2 -fno-omit-frame-pointer -mno-omit-leaf-frame-pointer -flto=auto
     -ffat-lto-objects -fstack-protector-strong -fstack-clash-protection -Wformat
     -Werror=format-security -fcf-protection'
  --cflags (as 17.10's, with -g twice)
  --ldflags -Wl,-Bsymbolic-functions -flto=auto -ffat-lto-objects -Wl,-z,relro -Wl,-z,now
    -Wl,--as-needed
```

The speed-relevant options match (-O2, link-time optimisation, hardening,
ICU, LLVM, no assertions); 17.10 uses LLVM 18 where the 16.15 package used
LLVM 17. The `uuid-ossp` extension, which the Thoth migrations create, was
built separately from the same source (section 5, deviations). The cluster
was created with `initdb -E UTF8 --locale=C.UTF-8 --locale-provider=libc`,
the first run's encoding, locale and collation provider, and `jit` set off,
every other setting at its default. With `jit` off no JIT is expected: every
`EXPLAIN (ANALYZE)` statement printed no JIT lines (`PLANJIT`), and the
profiled statements include no JIT time.

#### 10.7.2 Statement-label map

Printed at the start of every run (`LABELMAP` lines). The hold starts when
the state lock finishes and ends when the step's COMMIT finishes.

```text
outside the hold, no group   checkout check; transaction control and set local outside the
                             hold; run read; isolation assertion; pre-sizing and its per-year
                             sizing; lock works; lock state
rest                         run lock; set local in the hold; month keys; run update;
                             transaction control in the hold; anything else in the hold
walk                         month watermark, month total insert, month ambiguity insert
                             (charged at 5/3)
inserts                      month country insert, month institution insert (less 2/3 of the
                             walk's three statements), year country insert, year institution
                             insert
verify                       range verifier (pre and post), range deletes 1 to 6, re-sizing
                             and its per-year sizing
a verify page's statements   no group
```

#### 10.7.3 F1 and F7

Logs: `f1-1.log` and `f7-1.log` (hand-back paths). Both met every line.

```text
F1  1,132,043 work-day rows, 24,000 month keys, 2,000 works
    REBUILD from empty: 143 replacing steps, hold p50 409.5 ms, p95 508.8 ms, max 542.9 ms;
      statements per step 90; build 61.6 s, verify 9.1 s; 339 keys/s
      (projected 1,355 s at 460,000 keys, 2,946 s at 1,000,000)
    healthy REBUILD: 143 non-replacing steps, hold p50 57.3 ms, p95 86.6 ms, max 115.1 ms
    pages (24 over the three runs): p50 1,158.4 ms, p95 1,553.2 ms, max 1,585.0 ms
    verifyMetricRollupMonths exact after RECONCILED; no refusal
F7  100,000 works of one month key each
    REBUILD from empty: 100 replacing steps (338 statements each), hold p50 271.3 ms,
      p95 325.4 ms, max 356.1 ms; 1 non-replacing step 8.5 ms
    healthy REBUILD: 101 non-replacing steps, hold p50 40.4 ms, p95 49.1 ms, max 53.5 ms
    pages (22): p50 344.3 ms, p95 456.7 ms, max 464.8 ms; sizing up to 282 ms per page
    no refusal
```

#### 10.7.4 F2 with F3 and F4 embedded: fixture and floor

Log: `f2-full-1.log`.

```text
F2 books     2,000 works, 12,346,804 rows, 392,000 month keys (196 per work), 31.5 rows per
             key, institution rows on 15.2% of key-days, 40 to 160 countries per work
F2 chapters  2,600 works, 3,439,372 rows, 109,200 month keys (42 per work), 31.5 rows per key
F3           4 works, 227,638 rows (56,907 to 56,912 per work), 384 month keys per work,
             160 countries, 4 series, 8 years
F4 mix (a)   2 works, 8 series x 80 rows per key-month x 8 years: 61,440 rows per work
F4 second    2 works, 5 series x 128 rows per key-month x 14 years: 107,520 rows per work
F4 mix (b)   8 works, k = 39 series x 16 rows per key-month x 1 year: 7,488 rows per work
whole corpus 16,411,638 work-day rows, 509,696 month keys, registry 48 series
             (seeded and analysed in 634.6 s; database 5,427 MB, 8,234 MB after the build)
ambiguity    10,022 ambiguity rows after the from-empty build, over 501,200 base month keys
             (2.0%)
FLOOR|F2 rows per key 31.50|floor figure 3402 R^2 for R registry pairs|
     STEP_MAX_REPLACING_READ admits R <= 13|at R = 13: 574879|at R = 14: 666723
```

Every F4 work has one distinct country and one distinct institution per
work-day row within each key-month (distinct per key-month over its rows
1.000), so both monthly section tables hold as many rows as the work-day
rows: 61,440 (mix (a)), 107,520 (second length) and 7,488 (mix (b)) country
and institution month rows per work. Over their whole histories the works
carry 414, 657 and 315 distinct countries, above the census range per work,
which the specification allows (R11).

#### 10.7.5 F2, F3 and F4: the gated lines

No pass of F2's gated sequence had a refusal or a diagnostic measurement.

```text
pass                                   steps (replacing)  pages  build    verify  keys/s
F2 REBUILD from empty                  2,513 (2,513)       113    1,885 s  160 s   249
F4 repair pass                         2,513 (52*)         113      367 s  157 s   973
F4 repair pass repeated (report only)  2,513 (52*)         113      365 s  165 s   962
F2 VERIFY after 200 corruptions        -                   113      -      167 s   3,056
F2 repair REBUILD                      2,513 (191)         113      400 s  159 s   912
* the replacing steps of the F4 classes: 16 (mix (a)), 28 (second length), 8 (mix (b))
projected from empty: 1,846 s at 460,000 keys, 4,013 s at 1,000,000 keys
```

```text
line                                                       n      p50       p95       max       result
F2 non-replacing step hold p95 <= 2 s                      7,114  73.6 ms   102.3 ms  369.8 ms  MET
F2 replacing step hold p95 <= 3 s (from empty)             2,429  721.3 ms  861.7 ms  1,408.8   MET
F3 non-replacing step hold p95 <= 2 s                      95     119.9 ms  162.9 ms  209.0 ms  MET
F3 replacing step hold p95 <= 3 s (from empty)             32     1,276.2   1,399.1   1,439.7   MET
F2 replacing step hold p95 <= 3 s (its repair run;         174    799.1 ms  950.0 ms  1,316.7   MET
  F2 and F3 ranges)
F2 repair run, every replacing step with F4 ranges         191    804.7 ms  3,033.2   3,604.3   report only
F2 verify page time p95 <= 5 s                             485    1,388.7   1,602.1   1,890.4   MET
F3 verify page time p95 <= 5 s                             20     1,474.3   1,660.8   1,662.9   MET
F4 mix (a) verify page time p95 <= 5 s                     10     1,956.9   2,412.5   2,412.5   MET
F4 second length verify page time p95 <= 5 s               10     2,651.2   2,746.8   2,746.8   MET
F4 mix (b) verify page time p95 <= 5 s                     40     1,456.6   1,626.6   1,762.7   MET
every line: budget refusals 0, diagnostic measurements 0
```

F3's replacing steps took 50.9 ms (p50) and 74.9 ms (p95) to pre-size and
46.5 ms and 71.7 ms to re-size; F3's whole-work totals are 56,907 to 56,912
rows.

The F4 line (max of hold / fill, fill the smaller of row and figure fill):

```text
class (steps)            pass                          row fill  figure fill  fill   max hold / fill  result
mix (a) (16)             from empty                    0.960     0.922        0.922  2,943.4 ms       MET
                         first repair                  0.960     0.922        0.922  3,269.3 ms       MET
                         repeat (report only)          0.960     0.922        0.922  3,253.8 ms       -
second length (28)       from empty                    0.960     0.960        0.960  3,472.1 ms       MET
                         first repair                  0.960     0.960        0.960  3,807.9 ms       MET
                         repeat (report only)          0.960     0.960        0.960  3,539.8 ms       -
mix (b) (8)              from empty                    0.936     0.973        0.936  2,548.9 ms       MET
                         first repair                  0.936     0.973        0.936  2,614.9 ms       MET
                         repeat (report only)          0.936     0.973        0.936  2,739.1 ms       -
k: 8, 5 and 39 month keys per work-month; whole-work totals 61,440, 107,520 and 7,488;
walk rate per (month key x whole-work row) over the five keyed month statements, first
repair: 206.1, 235.3 and 188.1 ns; budget refusals 0 and diagnostic measurements 0 in
every class
```

Holds of the F4 replacing steps (p50 / p95 / max): mix (a) 2,610.7 /
2,709.9 / 2,712.6 ms from empty and 2,821.6 / 3,006.5 / 3,013.0 ms in the
first repair pass; the second length 2,970.2 / 3,232.9 / 3,333.3 ms and
3,238.1 / 3,545.4 / 3,655.6 ms; mix (b) 2,245.5 / 2,385.8 / 2,385.8 ms and
2,326.4 / 2,447.6 / 2,447.6 ms. The second length met both its lines
(`SECONDLENGTH|F4 line true|page line true`), so T may come from F4 and F5.
Autovacuum did not run on any of the eight tables between the build and the
first repair pass, nor between the two repair passes (the counts in the
`VACUUM` lines do not change). It ran during the passes: once on
`metric_rollup_work_institution_month` during the first repair pass, and on
the country month, institution month (twice) and institution year tables
during the repeat. Each repair pass damaged 52 rows, one per F4 unit.

Statements per step, in total: F2's replacing steps 60 to 110 (median 100)
and its non-replacing steps 30 to 34; F3's replacing steps 60 to 64; F4's
mix (a) and second length 70 to 74, mix (b) 150; every page 26.

`EXPLAIN (ANALYZE, BUFFERS)` of the 16 statements on a median replacing range
of each of F2, F3, F4 mix (a), the second length and mix (b), after ANALYZE
(80 plans):

```text
ACCEPT|EXPLAIN on F2, F3 and F4: no sequential scan of metric_rollup_work_day or of a
       projection table holding rows|0 found|MET
ACCEPT|EXPLAIN: sizing, RANGE_MONTH_KEYS_SQL, the range deletes and the range verifier are
       led by work_id|0 read work-day rows through the series index|MET
scans of tables holding no rows: none (every table held rows: 16,411,638 work-day rows,
       5.59 M country and 1.16 M institution month rows, 2.73 M and 0.83 M year rows,
       509,686 month rows and 10,022 ambiguity rows at the F4 range verifier)
```

The five keyed month statements read the work-day rows through
`metric_rollup_work_day_identity_key` bounded by `work_id` (the walk);
sizing and `RANGE_MONTH_KEYS_SQL` through the `(work_id, day)` and identity
indexes; each range delete and the range verifier through its table's
identity index.

The corruption cycle: all 200 scattered corruptions applied (34 month, 34
country month, 33 institution month, 33 ambiguity, 33 country year and 33
institution year rows); the `VERIFY` run reached `MISMATCHED` and counted
every one of them, as missing plus mismatched rows per family (17 + 17,
17 + 17, 16 + 17, 33 + 0, 16 + 17 and 16 + 17, no extra row); the repair
`REBUILD` replaced 191 ranges and reached `RECONCILED`. As in the first run,
`verifyMetricRollupMonths` on the whole of F2 exceeded its 30 s statement
timeout (33.9 s), the register 3.7 bound past which section 3.8 is the
supported path; the `RECONCILED` passes are the exactness evidence.

The line "F2 repair run, every replacing step with F4 ranges" is printed for
information: its p95 (3,033.2 ms) includes the F4 ranges, which the F4 line
judges (their hold / fill is within 4 s above). The gated F2 repair-run line
covers F2's and F3's ranges, as in the first run (section 14).

#### 10.7.6 F6 on the whole of F2, F3 and F4 included

F6 ran after F2's gated passes, on the same corpus with its F3 and F4 works
(the second length's included, since it met both its lines), before any
late work. The workload and sampler are those of section 10.6, on 80 of
F2's book works; the baseline ran 300 s without a run. Then the projections
were emptied and a `REBUILD` from empty ran under the production budgets,
paced by the section 15 rule, while the workload ran through its whole
build phase; its verify phase ran after the workload stopped and its deltas
were applied.

```text
baseline without a run (300 s)
  ingestion batches 17,537, works ingested 34,780, claims 657, deltas completed 32,564,
  no failure; every wait (state row, work row, ingestion statement) at most 200.2 ms
paced REBUILD build phase
  2,513 replacing steps, hold p50 739.2 ms, p95 988.0 ms, max 3,502.3 ms; no refusal,
  longest run of refusals 0, build phase 5,915.7 s (86 keys per second)
  workload: ingestion batches 231,841, works ingested 459,817, claims 9,333,
  deltas completed 459,563
  all causes: state-row waits p95 802.3 ms, max 3,507.3 ms; ingestion work-row waits
  p95 901.4 ms, max 3,106.2 ms; ingestion statement waits p95 801.8 ms, max 3,507.3 ms
ACCEPT|F6 zero ingestion or completion failures|0 and 0|MET
ACCEPT|F6 no deadlock naming a step's backend|step refusals for a deadlock 0|
       workload deadlocks 0|MET
WAITS|direct state-row waits attributable to steps|n=2572|p50 701.2 ms|p95 1001.8 ms|
      max 3507.3 ms|bound max <= 6 s|MET
WAITS|direct ingestion work-row waits attributable to steps|n=6|p50 701.2 ms|
      p95 1001.8 ms|max 1001.8 ms|bound max <= 8 s|MET
WAITS|ingestion statement waits with a step in the blocking chain (whole wait)|n=5027|
      p50 701.3 ms|p95 1001.9 ms|max 3507.3 ms|bound max <= 13 s|MET
ACCEPT|F6 the build phase completes|Verifying|refusals by reason {}|longest run of
       refusals 0|build phase 5915.7 s|MET
verify phase after the workload: 113 pages, page time p50 1,414.3 ms, p95 1,705.8 ms,
  max 2,448.4 ms; RECONCILED
```

As after F2's repair, `verifyMetricRollupMonths` on the whole table
exceeded its 30 s statement timeout (34.9 s). The unpaced run, information
only, was not run.

#### 10.7.7 Mix (c) (report only)

Seeded after F6, analysed, measured as a build and then as a repair pass
(one row damaged per unit), and deleted before F5. Neither work's pass had a
refusal or a diagnostic measurement.

```text
work                                     pass    steps  hold p50 / p95 / max            walk per entry
F4c1  1 series, 10 light years at 67     build   14     2,101.9 / 3,123.9 / 4,518.0 ms  461.2 ns
      rows per key-month before 12       repair  14     2,268.5 / 3,337.4 / 4,443.3 ms  494.0 ns
      heavy years at 625 (98,040 rows)
F4c2  2 series, 10 light years at 33     build   16     2,303.3 / 3,138.8 / 3,593.4 ms  347.9 ns
      before 12 heavy years at 312       repair  16     2,470.6 / 3,024.3 / 3,286.7 ms  340.1 ns
      (97,776 rows)
walk per entry: per (month key x whole-work row) over the five keyed month statements
```

F4c1's first step took six light years (72 month keys, 4,824 rows, figure
593,064: row fill 0.603, figure fill 0.988) and held 4,518.0 ms from empty
and 4,443.3 ms in repair, inside `STEP_HOLD_BUDGET` (5,000 ms) by about
0.5 s; F4c2's light-year steps took three years each (figure fill 0.986, row
fill 0.297) and held at most 3,593.4 ms. The walk per entry is about twice
F4's (188 to 235 ns, section 10.7.5), as section 10's few-series limit
expects. Mix (c) counts against no line, enters no fit and is not used for T.

#### 10.7.8 F5 (report only)

Three growths at F4's density, each on a growth work with a low `work_id`
placed first in run order. At each level the growth work's rows were
replaced by every year at the level, analysed, built (`build`), analysed
again, damaged in one row per unit, and the measured repair pass took its
range (`repair`); each run was abandoned once that range was replaced. At
growth 1's levels of 2 and 8 rows per key-month the step also took six units
of three neighbouring F2 works (594 month keys, 4,903 and 7,711 rows); every
other step took only its growth work. Holds are the maxima over the pass's
replacing steps. "Within 4 s" is given twice: on the repair steps alone, as
section 17 defines F5, and on every step of the level, build and repair
alike, the stricter reading register 3.8 states (section 5, deviations).

```text
growth 1: 39 series per work-month, one year of history (one unit)
rows per key-month   unit rows  figure      whole-work  repair max   build max    within 4 s: repair / both
 2                      936        73,008       936     1,019.5 ms   1,245.3 ms   yes / yes
 8                    3,744       292,032     3,744     1,819.0 ms   1,942.7 ms   yes / yes
16                    7,488       584,064     7,488     2,298.4 ms   2,450.9 ms   yes / yes
24                   11,232       876,096    11,232     3,561.0 ms   3,443.9 ms   yes / yes
25                   11,700       912,600    11,700     3,515.3 ms   3,593.5 ms   yes / yes
26                   12,168       949,104    12,168     3,709.3 ms   3,585.4 ms   yes / yes
27                   12,636       985,608    12,636     3,850.2 ms   3,863.5 ms   yes / yes (largest in both)
28                   13,104     1,022,112    13,104     4,117.6 ms   3,863.1 ms   no / no
32                   14,976     1,168,128    14,976     4,252.2 ms   4,345.0 ms   no / no

growth 2: 39 series per work-month, eight years of history (eight equal units)
 3                    1,404       492,804    11,232     1,679.5 ms   1,598.2 ms   yes / yes
 6                    2,808       985,608    22,464     3,352.9 ms   3,183.5 ms   yes / yes
 7                    3,276     1,149,876    26,208     3,739.3 ms   3,764.8 ms   yes / yes (largest in both)
 8                    3,744     1,314,144    29,952     4,727.1 ms   4,376.3 ms   no / no
 9                    4,212     1,478,412    33,696     4,355.2 ms   4,096.2 ms   no / no

growth 3: F3's four series, eight years of history (eight equal units)
 20                     960        34,560     7,680     2,706.9 ms   2,578.0 ms   yes / yes (one step, 8 units)
 60                   2,880       103,680    23,040     1,986.8 ms   2,096.6 ms   yes / yes (steps of 2 units)
100                   4,800       172,800    38,400     1,638.9 ms   1,820.2 ms   yes / yes
140                   6,720       241,920    53,760     2,300.0 ms   2,307.2 ms   yes / yes
180                   8,640       311,040    69,120     2,700.9 ms   2,732.1 ms   yes / yes
220                  10,560       380,160    84,480     3,466.8 ms   3,463.1 ms   yes / yes (largest in both)
260                  12,480       449,280    99,840     3,834.3 ms   4,241.8 ms   yes / no
261                  12,528       451,008   100,224     3,893.2 ms   4,452.9 ms   yes / no (largest on repair)
262                  12,576       452,736   100,608     4,039.4 ms   4,052.6 ms   no / no
265                  12,720       457,920   101,760     4,136.0 ms   4,595.8 ms   no / no
270                  12,960       466,560   103,680     4,329.4 ms   4,642.1 ms   no / no
280                  13,440       483,840   107,520     4,171.1 ms   4,258.6 ms   no / no
300                  14,400       518,400   115,200     4,625.1 ms   4,705.2 ms*  no / no
* one build step refused at STEP_HOLD_BUDGET (statement_timeout, hold 5,000.8 ms) under the
  section 10 budgets, measured once under the diagnostic budgets at 4,705.2 ms
```

The largest unit in each growth at which every step of the level, build and
repair alike, committed within 4 s, as register 3.8 states it (no step of
these levels was refused):

```text
growth  rows per   rows    month  figure     whole-work  years  holds p50 / p95 / max                 pre-sizing p50 / p95
        key-month          keys                                                                       (re-sizing)
1       27         12,636  468      985,608  12,636      1      repair 3,850.2 ms (one step)          repair 20.5 ms (18.1 ms)
                                                                build 3,863.5 ms (one step)           build 20.6 ms (16.2 ms)
2       7           3,276  468    1,149,876  26,208      8      repair 3,493.0 / 3,739.3 / 3,739.3 ms repair 35.8 / 49.6 ms (33.8 / 57.9 ms)
                                                                build 3,629.1 / 3,764.8 / 3,764.8 ms  build 36.6 / 66.0 ms (37.6 / 68.7 ms)
3       220        10,560   48      380,160  84,480      8      repair 3,310.3 / 3,466.8 / 3,466.8 ms repair 91.3 / 119.6 ms (88.4 / 96.7 ms)
                                                                build 3,303.4 / 3,463.1 / 3,463.1 ms  build 91.0 / 169.1 ms (90.4 / 101.7 ms)
```

On the repair steps alone, as section 17 defines F5 and as the harness's
`F5` lines print it, growths 1 and 2 give the same levels, and growth 3
reaches 261 rows per key-month: 12,528 rows, 48 month keys, figure 451,008,
whole-work 100,224 rows, repair holds 3,740.3 / 3,893.2 / 3,893.2 ms,
pre-sizing 90.1 / 126.9 ms (re-sizing 101.7 / 147.3 ms). Its build steps
held up to 4,452.9 ms, the first build step each time: from 260 rows per
key-month up, growth 3's build step was the dearer one.

F5 counts against no zero-refusal or page line; the refusal at growth 3's
300 rows per key-month is the measurement.

#### 10.7.9 The whole-work threshold T

```text
THRESHOLD|F4 classes ["F4a", "F4a2", "F4b"]: largest whole-work total of a replacing step
          within 4 s 107520|F5's units within 4 s: largest whole-work total 100224|
          whole-work threshold T 107520|F3's whole-work total 56912|T >= F3's total: MET
```

The second length met both its lines, so T is the largest whole-work total
among F4's replacing steps and F5's units within 4 s: 107,520, mix (a)'s
second length. F5's largest is 84,480 rows on build and repair steps alike
(growth 3 at 220 rows per key-month; the harness's line prints 100,224, the
repair-only reading), and F3's is 56,912 (the largest of F3's four works,
56,907 to 56,912 rows). Either reading gives the same T.

#### 10.7.10 Statement-group fits and the corners

Non-negative least squares per group, each against its own drivers; the
walk, inserts and verify on the first repair pass of the three F4 classes
(52 steps), the rest on the from-empty replacing steps of F2, F3, F4 and F7
(2,613 steps; F7's 100 read from `f7-1.log`).

```text
group    n      fitted costs                                                  absolute residuals
                                                                              median / p95 / max
walk     52     1.6999e-4 ms per (month key x whole-work row),                61.0 / 202.1 / 240.4 ms
                3.2122e-4 ms per (work-month x whole-work row)
inserts  52     1.6851e-1 ms per work-day row, 0 per year-term row            81.8 / 199.4 / 306.9 ms
verify   52     3.5304e-2 ms per work-day row,                                30.0 / 91.6 / 113.2 ms
                1.3323e-3 ms per whole-work row
rest     2,613  11.469 ms + 1.0022e-1 ms per statement in the transaction    2.3 / 7.5 / 316.9 ms
                + 1.0512e-4 ms per whole-work row
```

The largest measured statement count of a replacing step (in the
transaction) is 332 (F7's steps); a step at `STEP_MAX_MONTH_KEYS` has 320,
so the corners use 332. Each corner is a repair step with 8,000 work-day
rows and the figure full:

```text
corner                                  series per   whole-work  walk        inserts     verify    rest     predicted  bound     result
                                        work-month   rows                                                   hold
mix (a) at T = 107,520                  5.19         107,520     1,553.6 ms  1,348.1 ms  425.7 ms  56.0 ms  3,383.4 ms  4,500 ms  MET
mix (b), one year                       37.50        8,000       642.8 ms    1,348.1 ms  293.1 ms  45.6 ms  2,329.6 ms  4,500 ms  MET
ACCEPT|F4 statement-group prediction at both corners <= 4.5 s|MET
```

The six-term pooled fit over the 2,756 replacing steps of F2 to F4
(information only, with every residual in the log's `RESIDUAL` lines):
intercept 0, 2.3436 ms per statement, 0 per work-day row, 3.4150e-3 ms per
month-term row, 2.9432e-3 ms per year-term row and 1.1685e-2 ms per
whole-work row; absolute residuals median 43.2 ms, p95 173.7 ms, max
628.6 ms.

#### 10.7.11 Run time

The F2 test took 12,731 s (3 h 32 min): seeding and analysing the corpus
635 s; F2's from-empty `REBUILD` 2,045 s (build 1,885 s, verify 160 s; 249
keys per second, projected 1,846 s at 460,000 keys and 4,013 s at 1,000,000);
F4's repair pass 524 s and its repeat 530 s; the `VERIFY` after the
corruptions 167 s; the repair `REBUILD` 559 s; F6's baseline 300 s, paced
build phase 5,916 s and verify phase 163 s; mix (c) 151 s of passes; the
remaining 1,741 s covers, among others, `ANALYZE` and `EXPLAIN`, the
corruptions, the two whole-table verifies, emptying the projections before
F6, mix (c)'s seeding and F5's 27 levels. F1 took 2.7 min
and F7 about 1 min. The disk had 21 GB free before F2 and 11 GB after it.

#### 10.7.12 Summary against section 17

Every gated line was met on PostgreSQL 17 under the R11 budgets, with no
budget refusal in any gated pass and no diagnostic measurement outside F5.

```text
section 17 line                                          fixture (log)                  measured                          result
non-replacing step hold p95 <= 2 s, zero refusals         F1 (f1-1.log)                  86.6 ms                           MET
                                                          F2 (f2-full-1.log)             102.3 ms                          MET
                                                          F3 (f2-full-1.log)             162.9 ms                          MET
                                                          F7 (f7-1.log)                  49.1 ms                           MET
replacing step hold p95 <= 3 s, zero refusals             F1 from empty                  508.8 ms                          MET
                                                          F2 from empty                  861.7 ms                          MET
                                                          F3 from empty                  1,399.1 ms                        MET
                                                          F7 from empty                  325.4 ms                          MET
                                                          F2 repair run (F2, F3 ranges)  950.0 ms                          MET
verify page time p95 <= 5 s, zero refusals                F1                             1,553.2 ms                        MET
                                                          F2                             1,602.1 ms                        MET
                                                          F3                             1,660.8 ms                        MET
                                                          F4 mix (a), second, mix (b)    2,412.5, 2,746.8, 1,626.6 ms      MET
                                                          F7                             456.7 ms                          MET
F4 max of hold / fill <= 4 s, zero refusals               mix (a) from empty / repair    2,943.4 / 3,269.3 ms              MET
                                                          second length                  3,472.1 / 3,807.9 ms              MET
                                                          mix (b)                        2,548.9 / 2,614.9 ms              MET
F4 statement-group prediction <= 4.5 s                    mix (a) corner, mix (b) corner 3,383.4 ms, 2,329.6 ms            MET
T >= F3's whole-work total                                F4 and F5                      107,520 >= 56,912                 MET
F1 verifyMetricRollupMonths exact after RECONCILED        F1                             exact                             MET
F2 corruptions -> MISMATCHED -> REBUILD -> RECONCILED     F2                             200 of 200 applied; 191 replaced  MET
F6 zero ingestion or completion failures                  F6 (f2-full-1.log)             0 and 0                           MET
F6 no deadlock naming a step's backend                    F6                             0                                 MET
F6 direct state-row waits max <= 6 s                      F6                             3,507.3 ms                        MET
F6 direct ingestion work-row waits max <= 8 s             F6                             1,001.8 ms                        MET
F6 transitive ingestion statement wait max <= 13 s        F6                             3,507.3 ms                        MET
F6 the build phase completes                              F6                             5,915.7 s, 0 refusals             MET
EXPLAIN: no sequential scan of a table holding rows       F2, F3, F4                     0                                 MET
EXPLAIN: sizing, month keys, deletes, verifier by work_id F2, F3, F4                     0 through the series index        MET
six-term pooled fit                                       F2 to F4                       residual median 43.2 ms           information
mix (c)                                                   F4c1, F4c2                     max hold 4,518.0 ms, 0 refusals   report only
F5 largest units within 4 s (build and repair steps)      growths 1 to 3                 27, 7, 220 rows per key-month     report only
unpaced F6                                                -                              not run                           information
```

## 11. CI

CI status: NOT AVAILABLE (the branch push triggers no workflow; CI runs when
the orchestrator opens the PR)
Checks: see section 9 for the local runs of the CI commands.
Failures or warnings: none locally; clippy prints only the existing
future-incompatibility note for the `proc-macro-error2` dependency.

## 12. Rollout and rollback

Initial state after merge: one empty run table. Nothing runs: Thoth never
starts or advances a run on its own, and no consumer is wired.
Activation required: the orchestrator stage, every non-disposable run (each
with its own run grant per environment), deployment and activation are
separate gates (register 3.6, 3.8 and 4.2).
Feature flag/configuration: none; the operations require
`METRICS_INGEST_SERVICE`.
Migration sequence: `20261112_v1.10.0` runs through `thoth init` at
deployment, which the deployment grant must name.
Rollback/disable procedure (specification section 19): abandon any open run,
then deploy the previous binary and keep the table, which that binary ignores.
Dropping the table is optional and separate: run `down.sql` with this stage's
binary before deploying the previous one. Projections need no rollback.
Monitoring required: the structured `metric_rollup_month_rebuild` log lines
(refusal code and reason per call), and, during runs, the section 15 pacing
and refusal rules.

## 13. Known limitations and deferred work

- All section 17 numbers come from one local evidence host: the re-run
  (section 10.7) on CI's PostgreSQL major version (17), the first run
  (sections 10.1 to 10.6) on PostgreSQL 16, each cluster at its default
  settings except those section 10.7.1 lists. The F4 line assumes that the
  host stands in for production's; production's behaviour is judged again
  at each run grant (register 3.8).
- F4's mix (b) reached k = 39, and T is 107,520. The production registry
  pair count (against the floor, R <= 13) was stated on #1012 on 2026-11-13,
  within the floor, from a read outside this stage's grant. Production's
  largest whole-work total (against T) is still to be stated. Every
  `REBUILD` run grant checks both again.
- The few-series limit (section 10, Limits) is measured by mix (c), report
  only: its dearest step held 4.52 s, inside `STEP_HOLD_BUDGET`. A step of
  that shape that could not finish would be refused on every retry, a HOLD,
  never a wrong write.
- The orchestrator that drives runs, every non-disposable run and its run
  grant, deployment and activation remain separate stages and gates.

## 14. Unresolved issues

The R14 re-run (section 10.7) met every gated section 17 line on PostgreSQL
17 under amendment R14's budgets, with no budget refusal in any gated pass,
so specification section 19 raises no HOLD. The first run's HOLD, which the
amendment answered, is kept below as history.

1. **Inputs for the run grants** (register 3.8): the production registry
   pair count, which must be at most 13 (the floor), was stated on #1012 on
   2026-11-13 and was within it. Production's largest whole-work total,
   which must be at most T (107,520), is still to be stated, from the
   census's counts-only read or a separately authorized read. Every
   `REBUILD` run grant checks both again.
2. **Evidence not run**: the unpaced F6 run, which is information only.
   PostgreSQL 16 was not run again; the first run's numbers stay in
   sections 10.1 to 10.6.
3. **Deviations** (section 5): the `uuid-ossp` extension was built
   separately for the PostgreSQL 17 installation; and register 3.8 states
   F5's growth limits on build and repair steps alike, stricter than
   section 17's repair-only definition, because at growth 3's top levels
   the build step was the dearer (section 10.7.8).
4. **Questions for the reviewers**:
   - the printed line "F2 repair run, every replacing step including F4
     ranges" (report only) has a p95 of 3,033.2 ms, above 3 s, because it
     includes F4's ranges. The gated line covers F2's and F3's ranges
     (950.0 ms), as in the first run, and F4's ranges are judged by the F4
     line (hold / fill at most 3,807.9 ms). Is section 17's "F2 (its repair
     run)" meant to cover F4's ranges too?
   - two `MutationRoot` descriptions say "durable position" rather than
     "cursor" because an existing guard forbids that word in the
     `MutationRoot` SDL (section 5, decision 8).

### The first run's HOLD (history)

The first run's section 17 evidence (sections 10.1 to 10.6, PostgreSQL 16,
the R9 budgets) missed bounds, and specification section 19 made that a
HOLD. Nothing was changed to meet a bound: every constant, statement and
bound was as specified, and the diagnostic hold budget existed only in the
ignored evidence tests. F1, F7 and F6 (on F2 without its F3 and F4 works)
met every bound. The CTO chose the amendment path (section 2): amendment
R14 and erratum R14.1, which also settled its questions: a sequential scan of a
table holding no rows is reported and not counted, F4's distinct countries
per work may exceed the census range, F6 runs on the whole of F2 with its F3
and F4 works, and F2 carries ambiguity rows so that every corruption
applies. The items as written at the time:

1. **Bounds missed on the evidence host** (section 10):
   - the F4 line, in both mixes, from-empty and repair steps alike: max hold /
     fill 11.3 to 14.7 s against 4 s (both runs);
   - the pooled fit's predicted hold at full fill: 10.2 s at the year-only
     corner and 13.8 s at the mix (b) corner, against 4.5 s (the F3-F4-only
     fit: 10.2 s and 13.3 s);
   - the replacing-step hold p95 of F2 from empty (3,145 and 3,242 ms in the
     two runs) and in its repair run (3,668 ms over F2's and F3's ranges in
     run 2), and of F3 from empty (7,960 and 8,853 ms), against 3 s;
   - the verify page time p95 of F4's pages (mix (a) 8,052 ms, mix (b) 6,401
     ms in run 2) against 5 s;
   - as written, the sequential-scan line: every sequential scan (eight per
     run) read `metric_rollup_work_month_ambiguity`, which F2 leaves empty;
     every populated table was read through an index.

   Under the production budgets every F4 step is refused at
   `STEP_HOLD_BUDGET` on every retry (run 2's probe: three consecutive
   `statement_timeout` refusals at 5.0 s at the first F4 range), and F3's
   seven-unit steps cannot commit inside it either, so a `REBUILD` of F2
   cannot finish on this host.
2. **Where the time goes** (statement profiles and `EXPLAIN (ANALYZE,
   BUFFERS)`, section 10.5). Two costs that the replacing-read figure does
   not count dominate the dense steps:
   - every section row a step writes runs the four foreign-key checks of the
     section tables (work, publication, platform, measure), about 40 us per
     row here, so the monthly country and institution inserts grow with the
     section rows written, which at F4's section-row ratio equal the
     work-day rows;
   - the completion's keyed month statements read the work-day projection
     through `metric_rollup_work_day_identity_key` (`work_id`,
     `publication_id`, `platform_id`, `measure_id`, `day`, ...), and with
     `publication_id` unbound PostgreSQL 16 bounds that scan by `work_id`
     alone, so each month key walks the index entries of its work's whole
     history (for an F4 mix (a) work, about 3,000 buffers per key to return
     60 rows). The specification notes that skip scan in PostgreSQL 18
     changes these plans; production's major version is not known here.
   Both are properties of the reused completion statements and of the
   existing schema, which this stage may not change.
3. **Evidence not run as specified**:
   - F6 ran on F2 without its embedded F3 and F4 works (section 10.6):
     under the production budgets, which F6 uses, a run over the whole of F2
     stalls at the first F3 or F4 range on this host, so F6 over the full F2
     was not run; nor was the unpaced F6 run, which is information only;
   - production's PostgreSQL major version and CI's PostgreSQL 17 were not
     measured, and the host's comparability with production's was not
     established (section 13);
   - of F2's 200 scattered corruptions, 167 were applied, because F2 holds no
     ambiguity row to corrupt (section 10.5).
4. **What releases the HOLD**: a CTO decision on #1012, informed by this
   evidence, on an amendment of the section 10 bounds, the replacing-read
   figure or the section 17 bounds, and/or a re-run of section 17 on a host
   accepted as comparable to production and on production's PostgreSQL
   major version. F4's acceptance also needs the production registry pair
   count, and the whole-work threshold production's largest whole-work
   total, both stated on #1012 from a separately authorized read.
5. **Questions for the reviewers**:
   - whether a sequential scan of an empty projection table (cost 0, zero
     rows) counts against the `EXPLAIN` line;
   - F4's fixture draws its distinct countries per key-month (60 and 100,
     inside the census range) from a rotating pool, so a work carries 506
     (mix (a)) or 406 (mix (b)) distinct countries over its history, above
     the census range per work; the specification fixes the per-key-month
     ratio only;
   - whether F6 on F2 without its F3 and F4 works is acceptable evidence for
     F6, or F6 is to be repeated over the whole of F2 once the F3 and F4
     bounds are settled;
   - two `MutationRoot` descriptions say "durable position" rather than
     "cursor" because an existing guard forbids that word in the
     `MutationRoot` SDL (section 5, decision 8).

## 15. Agent self-assessment

The agent may identify risks but may not approve the task.

Suggested review focus:

- the build step's lock order, re-size and per-statement deadlines against
  sections 7 and 8 (`rebuild_run.rs`), and test 16.16's hold bounds;
- the replacing-read figure in `SIZING_SQL`, `PER_YEAR_SQL` and
  `METRIC_ROLLUP_RUN_GRANT_AGGREGATE_SQL` against section 10, and test 16.23;
- the statement-fidelity test 16.22, which pins the step's reuse of the
  completion's keyed statements;
- the evidence in section 10 of this report against the section 17 bounds,
  and the open evidence items in section 14.
