# MET-HIST-01-CKPT Implementation Report

## 1. Repository state

Owning GitHub issue: thoth-pub/thoth#1052
Repository: thoth-pub/thoth
Workflow: STANDARD
Base branch: `develop`
Authorized base commit: `bf133b229e4cb1b85337b8e84105d7523d1bc2fd`
Actual base commit: `bf133b229e4cb1b85337b8e84105d7523d1bc2fd` (`git rev-parse origin/develop` after `git fetch origin develop`, on 2026-11-24, before the branch was created)
PR target: `develop`
Programme integration branch: None
Task branch: `feature/metrics-history/met-hist-01-ckpt` (the namespace preflight was re-run with `git ls-remote origin 'refs/heads/feature/metrics-history*'`: no ref)
Head commit: recorded on the pull request; the PR head is the authority. The commits are listed in section 3, and `064c40446e106e3fa055e81b9f4d8125f056b092` is the last change to code, tests and documentation
Pull request: opened by the author, not by the implementation; its number and state are on GitHub
Expected branch deletion after merge: YES
Final programme PR required: NO
Implementing model: not recorded in this file (standing CTO rule for repository files)
Reasoning level: not recorded in this file

## 2. Scope confirmation

Approved specification: `docs/engineering/ai-delivery/tasks/MET-HIST-01-CKPT.md`, a copy of the specification R2 (SHA-256 of the source `de36808f6bd3e4a357b4c01816616f3864328229d78d45e8b65820ed80ff2e17`, checked before work started), approved by the CTO's grant (comment 7128051136 on thoth#1052, 2026-11-24).

Implemented objective: a `COMPLETED` import advances `last_successful_period_end` and records its manifest in the period-manifest cursor when every coverage row is accepted by rule A (the unchanged day rule) or, for a source whose driver key is not `cloudfront`, by rule B (the exact calendar month that starts on the import's day, on a registry pair that lists `MONTH`, `COMPLETE`, or `PARTIAL` for exactly `oapen`/`downloads` in 2026-01 and 2026-08). The CloudFront path is judged by rule A alone.

Out-of-scope changes made: NONE

## 3. Commits

- `98651ebdf191b24492682faa0f6192f51c67f5d4` - MET-HIST-01-CKPT: Record the approved task specification
- `064c40446e106e3fa055e81b9f4d8125f056b092` - MET-HIST-01-CKPT: Advance the source checkpoint for MONTH imports
- the commit that adds this report (MET-HIST-01-CKPT: Add the implementation report)

Every commit has author and committer `Javier Arias <javier@jarias.org>`.

## 4. Files changed

Authorized write paths (from the task specification):

- `thoth-api/src/model/metric_ingestion_lifecycle/mod.rs`
- `thoth-api/src/model/metric_ingestion_lifecycle/tests.rs`
- `docs/metrics/contract-register.md`
- `CHANGELOG.md`

Authorized new-file paths:

- `docs/engineering/ai-delivery/tasks/MET-HIST-01-CKPT.md`
- `docs/engineering/ai-delivery/implementation-reports/MET-HIST-01-CKPT-implementation-report.md`
- `thoth-api/src/model/metric_ingestion_lifecycle/checkpoint_coverage.rs`

Actual files changed (added/removed lines against the base, `git diff --numstat`):

- `thoth-api/src/model/metric_ingestion_lifecycle/mod.rs` (+90/-32)
  - reason: one `mod checkpoint_coverage;` line and its import; the equality `COUNT(*) FILTER` query and `CoverageEvidence` replaced by one grouped query with `LEFT JOIN`s and the call to the pure function; `rule_b_enabled` derived from the locked source's driver key; `successful` and the quarantine-only exception fed from the two verdict parts; module and function documentation updated
  - behavioural effect: MONTH units and the two allowlisted PARTIAL months of non-CloudFront sources advance the checkpoint and the cursor; nothing else changes
  - within authorized write budget: YES
- `thoth-api/src/model/metric_ingestion_lifecycle/tests.rs` (+946/-1)
  - reason: 13 new database tests and their helpers (appended as a new section); the one removed line is the `chrono` import, which gains `Datelike`
  - behavioural effect: none (tests); no existing test or expectation changed
  - within authorized write budget: YES
- `docs/metrics/contract-register.md` (+55/-20)
  - reason: the three passages that state the predicate (checkpoint progress in section 3.2, the cursor paragraph of section 3.3, the quarantine paragraph of section 3.4)
  - behavioural effect: none
  - within authorized write budget: YES
- `CHANGELOG.md` (+2/-0)
  - reason: one entry under `## [Unreleased]` / `### Changed`
  - within authorized write budget: YES

Actual new files created:

- `thoth-api/src/model/metric_ingestion_lifecycle/checkpoint_coverage.rs` (923 lines: the pure function, the allowlist, the grouped-row types and 10 unit tests) - within authorized new-file list: YES
- `docs/engineering/ai-delivery/tasks/MET-HIST-01-CKPT.md` (321 lines) - within authorized new-file list: YES
- `docs/engineering/ai-delivery/implementation-reports/MET-HIST-01-CKPT-implementation-report.md` (this file) - within authorized new-file list: YES

Files deleted, moved or renamed: NONE

### 4.1 Write-budget compliance

PASS

## 4.2 Authorized actions actually used

- repository inspection: YES (this repository only; the project folder files named in the brief)
- source edit: YES (within the write budget)
- new file creation: YES (the three authorized files)
- file deletion/move/rename: NO
- branch creation: YES (from the exact base)
- commit: YES
- push: YES (the task branch only, no force)
- PR creation/update: NO
- issue/comment mutation: NO
- manual CI dispatch/rerun: NO
- provider/runtime read: NO
- provider/runtime write: NO
- migration execution: NO (the test suite applies the embedded migrations to a disposable local PostgreSQL database, which section 15 of the specification does not count as a provider or runtime)
- release/tag/publication: NO
- merge: NO
- deployment: NO
- production activation: NO
- other: local disposable PostgreSQL 16 and Redis for the focused tests

Unauthorized actions performed: NONE

## 4.3 Automatic and manual external effects

Automatic CI/provider effects observed: none. The workflows `build_test_and_check.yml` and `run_migrations.yml` run on `push` only for `master` and `develop`, and `check_changelog.yml` and `docker_build_and_push_to_dockerhub.yml` only on `pull_request`, so pushing the task branch starts no workflow. Opening the pull request starts the four (section 16 of the specification), including the staging image push to the GitHub container registry. The full `thoth-api` suite therefore runs only once the author opens the pull request.

Manually initiated external actions: NONE

External writes/publication: NONE

## 5. Implementation decisions

1. The pure function, the allowlist and the grouped-row types are in the new `checkpoint_coverage.rs` (allowed by section 14), with one `mod` line in `mod.rs`. The signature is `judge_coverage(groups, total, envelope, rule_b_enabled) -> Result<CheckpointCoverage { accepted, strict_day }, Inconsistent>`.
2. Rule B reuses `period_matches_grain(MetricReportingGrain::Month, s, t)` and adds `s == e0`; it is evaluated only when `rule_b_enabled` is true, which the caller sets to `source.driver_key.as_deref() != Some(CLOUDFRONT_DRIVER_KEY)`, read from the source row the function already locks.
3. The plain row count is read in the same statement as the groups, as an uncorrelated scalar subquery on `metric_coverage` (`(SELECT COUNT(*) FROM metric_coverage WHERE import_id = $1)`). Section 7.1 names `COUNT(*) OVER ()`; after a `GROUP BY` a window count counts the groups, not the rows, so it could not catch a dropped or duplicated row. The intent (same statement, independent of the joins) is kept.
4. The total is taken from the first group row; one statement evaluates the uncorrelated subquery once, so every row carries the same value. Zero groups mean zero rows.
5. Every group is checked before a verdict is given, so inconsistent evidence is never masked by a group that already failed. A status that is not exactly `COMPLETE`, `PARTIAL` or `UNKNOWN` (read as text and parsed with the enum's `FromStr`), a NULL platform or measure code, a non-positive group count or a sum that differs from the total is `Inconsistent`, which the caller maps to `INTERNAL_STATE_INCONSISTENCY` with no write. The query now runs for any terminal import, including `COMPLETED_WITH_ERRORS`; under the foreign keys and the enum type these inconsistencies cannot occur, so no existing path changes.
6. No log line was added (section 9: no new logs), although `quarantine_evidence` logs the import id on its inconsistency. A review may ask for the same line here.
7. The allowlist is a `const` array of two `PartialMonth` entries holding the codes, the year and the month; the month end is computed. A pin test asserts the exact content and the exact match for each entry, other months, other periods and case, whitespace and look-alike codes.
8. The CHANGELOG line links the owning issue (#1052), as the file's entries for issues do; the author may replace the link with the pull request number.
9. The specification copy differs from the source file only in `Status` (`APPROVED`), the base-commit confirmation, `Approved by`, section 21, and the removal of the sentence "(The table is the proposal for the grant.", whose second sentence ("Local disposable databases for the tests are not a provider or runtime.") is kept as its own line. The heading "Decisions this spec takes (recommendation, for the grant):" in section 7.1 is unchanged because the brief allows no other edit; the author may want to reword it.
10. The database tests turn the fixture's CloudFront source into a non-CloudFront one by SQL (driver key `archive`, empty configuration), rename the fixture platform to `oapen`, and create the measures `downloads` and `views` and their `MONTH`-capable pair rows, because no migration seeds them. The CloudFront test keeps the source as it is and adds a `MONTH`-capable pair to its platform.

Deviations from the specification: NONE (decision 3 differs from the literal `COUNT(*) OVER ()` and keeps its stated intent).

## 6. Database and migration effects

Migration added: NO. `thoth-api/src/schema.rs` impact: none. The change reads `metric_coverage`, `metric_platform`, `metric_measure` and `metric_platform_measure` and writes nothing new; the existing single `UPDATE` of the checkpoint row is unchanged.

## 7. API and compatibility effects

GraphQL/API changes: none in fields, types or descriptions. `updateMetricSourceCheckpoint` now advances `lastSuccessfulPeriodEnd` and the cursor for MONTH units of non-CloudFront sources.
Generated schema/client updates: none.
Backwards compatibility: a Sphinx runner built before this change keeps working.
Deprecations: none.
Cross-repository dependencies: the Thoth server carrying this change must be deployed before any MONTH unit is imported (H5d, H6, MET-HIST-02). The gates in section 12 belong to those stages' grants.

## 8. Authorization and security

Authorization paths changed: none. Roles/scopes involved: unchanged (`METRICS_INGEST_SERVICE`).
Negative authorization tests: the 12 existing tests of `graphql::metric_ingestion_lifecycle_tests` pass unchanged.
Secret or personal-data handling: none involved.
Security limitations: a false accept is irreversible (the checkpoint never moves back and the driver does not rediscover the month), which is why every unclear case fails closed: rule B needs the registry's `MONTH`, a missing pair row reads as unsupported, the allowlist is exact, and contradictory evidence is an error.

## 9. Tests and checks

All runs used a lean build (`CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 THOTH_EXPORT_API=https://export.thoth.pub`) and a disposable local PostgreSQL 16 database (UTF-8) with a local Redis. The full workspace suite was not run locally.

### Formatting

Command:

```text
cargo fmt --all -- --check
```

Result:

```text
no output, exit 0
```

### Unit tests

Command:

```text
cargo test -p thoth-api --features backend --lib checkpoint_coverage
```

Result:

```text
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 2756 filtered out
```

(The first run had one failure, in a case of the new table that expected a one-day row on the envelope day to be refused; a one-day `COMPLETE` row on the envelope day is rule A and is accepted, so the case was corrected and an explicit assertion added. The 10 pure-function tests were then re-run.)

### Integration/database tests

Command:

```text
cargo test -p thoth-api --features backend --lib metric_ingestion_lifecycle
```

Result (this filter also selects `graphql::metric_ingestion_lifecycle_tests`):

```text
test result: ok. 96 passed; 0 failed; 0 ignored; 0 measured; 2683 filtered out; finished in 84.91s
  model::metric_ingestion_lifecycle:          84 (61 existing, 10 pure-function, 13 new database tests)
  graphql::metric_ingestion_lifecycle_tests:  12 (all existing)
```

The same filter was run once before any new test existed, against the new code: 83 passed (the 73 existing and the 10 pure-function tests), so the existing tests pass unchanged.

The 13 new database tests:

- `a_perfect_month_unit_advances_the_checkpoint_to_its_second_day_and_the_next_month_follows`
- `monthly_cursor_entries_are_evicted_by_period_across_the_sixty_four_bound`
- `a_month_unit_with_several_measures_needs_every_row_accepted`
- `partial_coverage_advances_the_checkpoint_only_for_oapen_downloads_in_the_two_named_months`
- `the_allowlist_names_the_platform_so_the_same_partial_month_on_another_platform_never_advances`
- `the_checkpoint_is_a_high_water_mark_so_a_later_month_moves_it_past_an_unimported_earlier_one`
- `a_month_row_that_is_not_the_calendar_month_starting_on_the_envelope_day_never_advances`
- `a_month_unit_with_zero_coverage_rows_never_advances`
- `a_cloudfront_unit_is_judged_by_the_day_rule_alone_whatever_the_registry_lists`
- `a_missing_registry_pair_row_fails_rule_b_closed_and_leaves_rule_a_rows_accepted`
- `a_day_unit_is_accepted_although_its_own_registry_pair_row_is_gone`
- `a_registry_change_after_the_import_changes_the_replay_in_both_directions_and_never_writes`
- `an_allowlisted_partial_month_reads_as_one_partial_coverage_run_until_a_complete_import_replaces_it`

The existing lock-order and race tests (`the_checkpoint_update_locks_checkpoint_import_account_then_source`, `a_current_and_a_stale_checkpoint_update_race_to_one_durable_outcome`) are among the 61 and pass unchanged. The existing tests named in section 7.1 for the PARTIAL read (`metric_coverage_run/tests.rs` and `metric_dashboard/tests.rs`) were not re-run or repeated.

### Lint/static analysis

Commands:

```text
cargo clippy -p thoth-api --all-targets --all-features -- -D warnings
cargo clippy --all --all-targets --all-features -- -D warnings
```

Result:

```text
first run: 13 errors, all clippy::cloned_ref_to_slice_refs in the new pure-function tests (fixed with std::slice::from_ref)
re-run: Finished `dev` profile, no warnings (thoth-api), then the same for the workspace
(the workspace run prints a future-incompatibility note for the dependency proc-macro-error2, which is not this change)
```

### Other required checks

```text
git diff --check            no output
```

Mutation checks (extra evidence that the new tests are not vacuous; each mutation was applied to `mod.rs`, run against the whole `metric_ingestion_lifecycle` filter, and reverted; none is in the commits; they ran before the twelfth database test, which has only negative cases, was added):

- rule B disabled (`rule_b_enabled = false`): 8 tests failed (every positive MONTH and PARTIAL test, the cursor-bound test, the replay test and the coverage-run test)
- rule B enabled for every source (`rule_b_enabled = true`): 1 test failed, `a_cloudfront_unit_is_judged_by_the_day_rule_alone_whatever_the_registry_lists`
- the registry join changed from `LEFT JOIN` to `JOIN`: 1 test failed, `a_missing_registry_pair_row_fails_rule_b_closed_and_leaves_rule_a_rows_accepted` (the dropped row makes the sum check fail)

Not run: `cargo test --workspace` and the rest of `cargo test -p thoth-api --features backend` (about 12 minutes and 25 GB; they run as GitHub checks once the pull request exists).

## 10. Manual verification

Environment: none required beyond the section 12 read-backs.
Steps: none.
Observed result: not applicable.
Evidence link/screenshot/log reference: none.

## 11. CI

CI status: recorded on the pull request, which is the authority (a push to the task branch alone starts no workflow, see section 4.3; opening the pull request starts the checks)
Checks: see the pull request.
Failures or warnings: none observed locally.

## 12. Rollout and rollback

Initial state after merge: `develop` only; merge is not release.
Activation required: none separate. The change is active wherever it is deployed and affects only MONTH units of non-CloudFront sources and the allowlisted PARTIAL units; no MONTH account is enabled until the H5d, H6 and MET-HIST-02 grants.
Feature flag/configuration: none.
Migration sequence: none.
Rollback/disable procedure: revert the pull request and redeploy the previous image, each under its own grant. The revert is the kill switch; exposure is nil until a MONTH import. Checkpoints that MONTH units advanced before a revert stay advanced.
Monitoring required: the section 12 read-backs of the specification, as future steps under the grants that carry them:

1. after deployment, the first scheduled CloudFront drain still advances each live account's `lastSuccessfulPeriodEnd` by one day (for each account, the value read before the drain plus one day);
2. V-MONTH and V-NATIVE remain clean;
3. before the first MONTH import (that stage's own grant): the read-back of the registry codes `oapen` and `downloads` and of the pair's `MONTH` grain (D2), and the evidence that the emitted MONTH coverage rows are exactly `[1st, next 1st)` (section 5);
4. pilot: the first MONTH unit imported alone, then `metricSourceCheckpoint(X).lastSuccessfulPeriodEnd` equals the imported month's 1st plus one day and the driver summary's `next` equals the 1st of the next month.

These are read-backs and steps of later stages, each under its own grant; none belongs to this change.

## 13. Known limitations and deferred work

- Two published GraphQL descriptions become stale and stay as they are, because their text is in the client schema snapshot downstream repositories generate from: `lastSuccessfulPeriodEnd` ("successfully and completely collected", `graphql/model.rs` 4834 to 4837) and the `updateMetricSourceCheckpoint` mutation ("all of which cover exactly the import period completely", `graphql/mutation.rs` 676). The contract register now carries the nuance. The follow-up issue belongs to this repository and is opened when this lands.
- `thoth-api/src/model/metric_coverage_run/producer_manifest.txt` lists `metric_ingestion_lifecycle/mod.rs` (role `terminalization`), so the H1 coverage-run producer identity changes by construction. No test pins its digest (`metric_coverage_run/tests.rs` only prints it).
- The allowlist matches every account on the platform `oapen` (the driver key is not part of the key, D2); the H4 archive fence ends `oapen`/`downloads` at 2025-10, so nothing overlaps today.
- The allowlisted PARTIAL months are not rediscovered once recorded, and the Sphinx archive driver has no replay mode, so a corrected month cannot be re-imported by it (section 7.1, "PARTIAL and later replacement").
- The cursor holds 64 entries; for a MONTH account these are the newest 64 months.
- `last_successful_period_end` stays a high-water mark: a later month moves it past an unimported earlier one; the driver's expected-start rule is the fence.

## 14. Unresolved issues

- NONE

## 15. Agent self-assessment

The agent may identify risks but may not approve the task.

Suggested review focus:

- the grouped query in `update_metric_source_checkpoint`: the `LEFT JOIN`s, `GROUP BY` by output position, the scalar subquery for the plain count, and that no row can drop out of the evidence;
- `rule_b_enabled` (derived from the locked source row) and the use of `strict_day`, never `accepted`, for the CloudFront quarantine-only exception;
- exactness of the allowlist and of rule B (`s == e0` and the existing `period_matches_grain` for `MONTH`);
- the replay semantics in both registry directions and that a released replay still never writes;
- the wording of the three contract register passages and of the module and function documentation;
- whether the inconsistency check should also log the import id, as `quarantine_evidence` does (decision 6).
