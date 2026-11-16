# MET-WP4-WIDGET-01 Implementation Report

## 1. Repository state

Owning GitHub issue: thoth-pub/thoth#1007 (programme: Thoth Metrics, #766)
Repository: thoth-pub/thoth
Workflow: STANDARD
Base branch: `develop`
Authorized base commit: `af6719f69353db32afa72f2aa0748072e6879d18` (R11; the
branch was built on `cab25eba415386f3dced1571c9f6468e1fa536f1` and refreshed
onto `af6719f6`, where `MET-WP4-03C-B2` landed, by merging `develop`)
Actual base commit: `af6719f69353db32afa72f2aa0748072e6879d18`, merged into
the branch in `55c4e2b3` (section 17)
PR target: `develop`
Programme integration branch: none
Task branch: `feature/metrics-widget/met-wp4-widget-01`
Head commit: recorded on the pull request; the PR head is the authority. The
commits are listed in section 3
Pull request: thoth-pub/thoth#1011, opened as a draft by the orchestrator (not
by the implementing agent); its live state is recorded on GitHub
Expected branch deletion after merge: YES
Final programme PR required: NO
Implementing model: Claude Opus 5.5
Reasoning level: high

## 2. Scope confirmation

Approved specification: the #1007 body, R11 (sections 1-11), which
re-verifies R10 against `af6719f6` after B2 (`MET-WP4-03C-B2`, PR #1010)
landed first; authorized by `ADR-0014` and the implementation grant on #1007
(comment 6777125645, CTO, 2026-11-10), with the grant's budget additions and
corrections. The refresh onto `af6719f6` follows R11 sections 4.10, 9 and 11
and applies the notes of the two R11 reviews (section 17).

Implemented objective: a public, read-only `metricWidget` query for the
embeddable `metrics-widget` package, gated by `METRICS_WIDGET` and shipped
inactive behind `THOTH_METRIC_WIDGET` (default `OFF`). The engine in
`metric_dashboard/mod.rs` (B2's, at `af6719f6`) gains only the section 4.10
Rust-level parameters.

Out-of-scope changes made: NONE

## 3. Commits

- `c4f3ae36b4d420b7ac6efe9f23347345ca55adc2` - Add pair mode and a read deadline to the Metrics engine
- `81004348e112fa287862e0946fb9d95f3ef12b81` - Add the public metricWidget read
- `09273a3cb37193c9d4543f3ce24659c39b351f40` - Add the THOTH_METRIC_WIDGET switch, off by default
- `493aa2fdc8977d5bd791448809b5b78132524ca0` - Record the public metricWidget contract
- `f58d0905dea45d8bbb3ae73426a7e927e9445893` - Add the MET-WP4-WIDGET-01 implementation report

Review fix pass (section 16):

- `8e5f62f0322d3cfedec4c417185a03926ef2f664` - Take widget slots with a compare-exchange loop
- `4ecd79eb978f6a60419924d3dcc3313a509c9339` - Refuse a repeated child row in the widget resolution
- `39b2bfbcdd3e1dc684cc6acd6a2f06d699da7f3f` - Test widget cross pairs, engine deadlines and the frontier check
- `e220981ab1c65495227e82e7fa28bd6c45a6efa6` - Re-wrap the metricWidget clause in the contracts map
- `b2cb0730401ae01a3f31f3ef6203b5243a27e384` - Update the MET-WP4-WIDGET-01 implementation report for the review fixes

Refresh onto B2 (R11, section 17):

- `55c4e2b3ed6443cbcdb9985634cab470e7796679` - Merge develop (B2) into the metricWidget branch
- `f15ce2a19804757c25603474eb78d96d98ca9179` - Re-apply pair mode and the deadline record on B2's engine
- `0bdcc1c07547d559a3e02a88ebc7b423310dcb55` - Add the R11 pair-mode, deadline and oracle tests
- `f4715db3ad1958393a5e5808fe982657d8c96cf4` - Record the widget contract beside B2's register section and changelog
- the commit that carries this revision of the report - Update the MET-WP4-WIDGET-01 implementation report for the B2 refresh

The gates in section 9 were run on the code of `f4715db3` (tree
`678e905d57e93f36be4f1f340c154d3b22b22d0c`); only line re-wraps in
`docs/metrics/contract-register.md`, which no cargo command reads, changed
during the run, and the `git diff` checks were re-run on the final tree. The
report commit after it changes only this file. The fix pass's gates ran on the tree of `e220981a`
(tree `5d194af80ad4fc7d76273153cc2db514736c48de`).

Every commit is authored and committed as Javier Arias
<javier@jarias.org>, with no trailers.

## 4. Files changed

Authorized write paths (spec section 9 plus the grant):

- `thoth-api/src/model/metric_dashboard/mod.rs` (section 4.10 only)
- `thoth-api/src/model/mod.rs` (module registration)
- `thoth-api/src/graphql/query.rs` (the `metricWidget` field)
- `thoth-api/src/graphql/model.rs` (switch and claim counter on `Context`)
- `thoth-api/src/graphql/mod.rs` (one test-module line)
- `thoth-api/src/graphql/metric_dashboard_tests.rs` (exactly two edits)
- `thoth-api/src/graphql/metric_registry_tests.rs` (exactly one edit)
- `thoth-api/src/model/publisher/mod.rs` (the `MetricsWidget` description)
- `src/bin/arguments/mod.rs`, `src/bin/commands/start.rs`,
  `src/bin/commands/mod.rs` (the switch argument and its registration)
- `src/bin/thoth.rs` (switch control tests only)
- `thoth-api-server/src/lib.rs` (switch plumbing, test app, status tests)
- `CHANGELOG.md` (one `[Unreleased]` line, appended to B2's `### Added`
  list, with no new heading)
- `docs/metrics/contract-register.md` (new section 4.4, after B2's 4.3, and
  three corrections)
- `docs/metrics/README.md` line 100 and
  `docs/engineering/repository-map/contracts.md` lines 81-82 (grant addition
  F-01 / R10-O4)

Authorized new-file paths:

- `thoth-api/src/model/metric_widget/mod.rs`
- `thoth-api/src/model/metric_widget/tests.rs`
- `thoth-api/src/graphql/metric_widget_tests.rs`
- `docs/engineering/ai-delivery/implementation-reports/MET-WP4-WIDGET-01-implementation-report.md`

Actual files changed:

- `thoth-api/src/model/metric_dashboard/mod.rs`
  - reason: section 4.10, on B2's engine at `af6719f6`.
  - behavioural effect: none for the dashboard. The 41 message constants
    become `pub(crate)` (text unchanged; rustfmt rewraps some declarations).
    B2's `ReadDeadline` gains exactly: a last-statement field
    (`Cell<Option<&'static str>>`, `None` from `new`); a record call directly
    after each of the 12 checks that admit a named statement (step 7 records
    the name of the text `month_statement` chose); `expired()` without
    `#[cfg(test)]`; and `pub(crate) fn last_statement()`. `check()`,
    `remaining()`, `checkout()`, the warn line and every check position are
    unchanged; the check before `BEGIN`, the one in `checkout()` and the
    three prelude checks in `evaluate` record nothing. New `pub(crate)`
    `MetricPairRead` with constructor functions (`new`,
    `with_timeline_grain`, `with_countries`, `with_institutions`,
    `with_capability`) and the pair-mode entry `metric_pair_read(connection,
    read, deadline)`. The body that runs from `PUBLISHERS_SQL` on is
    extracted from `evaluate` into `read_from_publishers`, defined
    immediately after `evaluate`; `evaluate` keeps its four-argument
    signature and prelude and calls it with the dashboard capability and no
    pair scope. In pair mode the caller's works and publisher replace
    `WORKS_SQL`, the pairs replace `KNOWN_SQL`/`REPRESENTED_SQL` and the
    combination bound, and the engine's `combinations` (the per-pair facts'
    keys, `serve`'s cell count and the output loop) are the pair list. In pair
    mode only, monthly total, country and coverage-run rows of an unheld pair
    are dropped as they are read, before any parse or check: in
    `apply_sections` and before step 10's run-row range check, so they reach
    neither the sections, the range check nor `coverage_streams`' same-stream
    overlap check. The pair-less `AMBIGUITY` row is kept. `CANONICAL_SQL`'s
    `$8`/`$9` are bound by the shared code exactly as for a dashboard read
    with an explicit start. The module doc's "Deliberately absent" paragraph
    points to `metric_widget`. The source-text pins hold: 22
    `diesel::sql_query(`, 22 `_SQL: &str`, one `db.get_timeout(`, no
    `db.get()`, one `Array<Date>`, one `candidate_days.push(`, one
    `fn evaluate(`, and the entry points and transaction closure unchanged.
  - within authorized write budget: YES
- `thoth-api/src/model/mod.rs` - registers `metric_widget` (backend). YES
- `thoth-api/src/graphql/query.rs` - the `metricWidget` resolver, no role
  guard, single-line description. YES
- `thoth-api/src/graphql/model.rs` - `Context.metric_widget` (default `OFF`),
  `Context.metric_widget_claims` (an `AtomicUsize` initialised in
  `Context::new`, whose signature is unchanged) and the builder
  `with_metric_widget`. YES
- `thoth-api/src/graphql/mod.rs` - `#[cfg(test)] mod metric_widget_tests;`. YES
- `thoth-api/src/graphql/metric_dashboard_tests.rs` - the two named edits
  (section 9 below). YES
- `thoth-api/src/graphql/metric_registry_tests.rs` - the one named edit. YES
- `thoth-api/src/model/publisher/mod.rs` - the `MetricsWidget` description,
  exactly the grant's text. YES
- `src/bin/arguments/mod.rs`, `src/bin/commands/start.rs`,
  `src/bin/commands/mod.rs` - `metric_widget()` argument (`--metric-widget`,
  env `THOTH_METRIC_WIDGET`, default `OFF`, values `OFF`/`ON`), registered on
  `start graphql-api` and `init`, the `commands::start::metric_widget`
  accessor, passed to `api_server`. YES
- `src/bin/thoth.rs` - `metric_widget_control` test module. YES
- `thoth-api-server/src/lib.rs` - `start_server` takes the switch, registers
  it as app data, the handler extracts it and applies `with_metric_widget`;
  the in-file test app registers the default; two new status tests. YES
- `CHANGELOG.md`, `docs/metrics/contract-register.md`,
  `docs/metrics/README.md`, `docs/engineering/repository-map/contracts.md` -
  as authorized. YES

Actual new files created: the four authorized new files - YES for each.

Files deleted, moved or renamed: NONE

### 4.1 Write-budget compliance

PASS. `git diff --stat af6719f6` lists only budget paths (section 9).

## 4.2 Authorized actions actually used

- repository inspection: yes
- source edit: yes
- new file creation: yes
- file deletion/move/rename: no
- branch creation: yes (`feature/metrics-widget/met-wp4-widget-01`, in a
  worktree)
- commit: yes
- push: yes
- PR creation/update: no (the orchestrator opened the draft PR, thoth-pub/thoth#1011)
- issue/comment mutation: no
- manual CI dispatch/rerun: no
- provider/runtime read: no
- provider/runtime write: no
- migration execution: local disposable test database only
- release/tag/publication: no
- merge: no
- deployment: no
- production activation: no (the switch stays `OFF` everywhere)
- other: none

Unauthorized actions performed: NONE

## 4.3 Automatic and manual external effects

Automatic CI/provider effects: PR CI (`build-test-and-check`) runs on each
push to the pull request; its runs and results are recorded on the pull
request (section 11).

Manually initiated external actions: NONE

External writes/publication: NONE

## 5. Implementation decisions

1. The read is `resolve(db, switch, claims, input, profile, sink)` behind the
   production entry `metric_widget`; `Profile` carries the statement timeout,
   deadline, checkout timeout, the current date and a per-step hook, so tests
   inject a shorter timeout, a past deadline, a fixed date or a panic
   (section 4.10 allows a private parameterised entry).
2. The resolution statement uses a `LEFT JOIN LATERAL` over `has-child`
   relations, skipped for `book-chapter`, with `LIMIT 501`, and reads the
   package through the `ThothPackage` SQL type; the step 9 decision calls
   `ThothPackage::has_capability(PublisherCapability::MetricsWidget)`.
3. Pair discovery applies eligibility (public, enabled, additive across time
   and works; enabled platform) inside the statement and stops after 41 rows.
4. The labels statement unions the frontier row
   (`transaction_timestamp()`, `watermark_at`) with the served platforms' and
   measures' labels; a missing frontier, missing label or unexpected row is
   `INTERNAL_ERROR`.
5. The engine error mapping names only the variants that do not map to
   `INTERNAL_ERROR` and ends with a wildcard arm, which B2's `Busy` reaches;
   `METRIC_WIDGET_BUSY` comes only from the engine's deadline flag
   (`ReadDeadline::expired`) or the widget's own refused check.
6. The log decision `log_decision(&WidgetFacts) -> Vec<WidgetLogLine>` is a
   pure value; lines are written with the `log` crate already in use. No line
   field can hold a package value.
7. The slot limiter is one process-wide `static AtomicUsize` with a drop
   guard; a slot is taken with a `compare_exchange_weak` loop that never
   takes one when `METRIC_WIDGET_SLOTS` are in use; checkout uses
   `PgPool::get_timeout(1 s)`.
8. Step 10 refuses a child ID that the resolution returns twice as
   `INTERNAL_ERROR` (`UnexpectedRow`). `work_relation_relator_related_uniq`
   (`UNIQUE (relator_work_id, related_work_id)`) rules such a row out, so
   the 500-child bound is a row count and never rests on deduplication.
9. One engine `ReadDeadline`, created at step 5 with the widget's own instant
   (`METRIC_WIDGET_DEADLINE` from then), serves both section runs; the
   widget's private `Deadline` holds it and admits the widget's own
   statements against the same instant with `check()`'s comparison (refused
   when `now >= at`). A widget-step refusal sets no engine flag and writes no
   B2 line. The error line's last statement is the engine's
   `last_statement()` when an engine run started after the last widget-owned
   statement, otherwise that widget-owned statement. The PR's own
   `MetricReadDeadline` is gone.
10. The refresh merges `develop` into the branch (no rebase). The merge
    commit takes `develop`'s `metric_dashboard/mod.rs` verbatim and does not
    compile on its own; the next commit re-applies section 4.10 on B2's
    engine and compiles (section 17).

Deviation from the specification requiring authorization: NONE

## 6. Database and migration effects

Migration added: NO. No index or table is added.

## 7. API and compatibility effects

GraphQL/API changes: additive only - `QueryRoot.metricWidget` and the input
`MetricWidgetInput` and types `MetricWidget`, `MetricWidgetSection`,
`MetricWidgetPairCoverage`, `MetricWidgetPlatform`, `MetricWidgetMeasure`.
The `PublisherCapability.METRICS_WIDGET` description text changes. Every other
operation keeps its schema, results and authorization.

Generated schema/client updates: `thoth-client` regenerates its git-ignored
`assets/schema.graphql` at build; no tracked client change.

Backwards compatibility: additive; nothing removed or renamed.

Deprecations: none.

Cross-repository dependencies: `metrics-widget` (WIDGET-02) is the consumer;
the infrastructure templates gain `THOTH_METRIC_WIDGET` only at activation.

## 8. Authorization and security

Authorization paths changed: one new public field with no role, gated by the
server switch and the `METRICS_WIDGET` capability (`ADR-0014` s4.4). No role,
policy or `require_*` guard changes.

Roles/scopes involved: none; the caller's identity is never read.

Negative tests: switch off, unknown DOI and an unentitled (`OBELISK`)
publisher give one indistinguishable refusal after the same single statement;
anonymous, no-role, other-publisher, `METRICS_READ_SERVICE` and superuser
callers get identical results; the engine re-checks the capability passed to
it (`OBELISK` passes with `METRICS_COLLECT`, is refused with
`METRICS_WIDGET`).

Secret or personal-data handling: no SQL, configuration, package value, token
or engine message is returned or logged; the DOI (public data) appears only
on lines after steps 8 and 9.

Security limitations: under anonymous load, statement timeouts after the
prelude produce error-level lines (accepted, bounded by the slots); the WAF
rate rule is an activation precondition.

## 9. Tests and checks

All commands were run from the worktree with
`TEST_DATABASE_URL=postgres://thoth:thoth@localhost:5432/thoth_test` (a local
disposable PostgreSQL 16 cluster), `THOTH_EXPORT_API=https://export.thoth.pub`
and `TEST_REDIS_URL=redis://localhost:6379` (local), as CI sets them, and
`CARGO_INCREMENTAL=0`. The results below are for the refresh's code tree
(section 3) unless a line says otherwise.

### Formatting

```text
cargo fmt --all -- --check
```

Result: exit 0, no output.

### Lint/static analysis

```text
cargo clippy --all --all-targets --all-features -- -D warnings
```

| Tree | Toolchain | Result |
| --- | --- | --- |
| `f58d0905` | local default: `rustc 1.97.0 (2d8144b78 2026-07-07)`, `clippy 0.1.97 (2d8144b788 2026-07-07)` | exit 0 |
| `f58d0905` | CI `lint` job, the runner's preinstalled stable (the job does not print its version) | failed: `use of deprecated method std::sync::atomic::Atomic::<usize>::fetch_update: renamed to try_update for consistency` at `thoth-api/src/model/metric_widget/mod.rs:833:10`, made an error by `-D warnings` |
| fix pass | local default: `rustc 1.97.0 (2d8144b78 2026-07-07)`, `clippy 0.1.97 (2d8144b788 2026-07-07)` | exit 0 |
| fix pass | current stable: `rustc 1.99.1 (58cf8f9c3 2026-10-13)`, `clippy 0.1.99 (58cf8f9c35 2026-10-13)`, run as `cargo +1.99.1 clippy ...` with a separate target directory | exit 0 |
| refresh | local default: `rustc 1.97.0 (2d8144b78 2026-07-07)`, `clippy 0.1.97 (2d8144b788 2026-07-07)` | exit 0 |
| refresh | current stable: `rustc 1.99.1 (58cf8f9c3 2026-10-13)`, `clippy 0.1.99 (58cf8f9c35 2026-10-13)`, run as `cargo +1.99.1 clippy ...` with a separate target directory | exit 0 |

Every passing run printed only the pre-existing future-incompatibility note
for `proc-macro-error2`. `AtomicUsize::fetch_update` is deprecated on 1.99.1
but not on 1.97.0, and the repository pins no toolchain, so the local run at
`f58d0905` could not show the CI failure. The fix (section 16, F1) replaces
the call with a `compare_exchange_weak` loop, which compiles without warnings
on both; it adds no `#[allow(deprecated)]` and does not use `try_update`,
since CI's toolchain is unpinned.

### Full workspace

```text
cargo test --workspace
```

Result (refresh, 1.97.0):

```text
thoth (lib)                       ok. 0 passed
thoth (bin src/bin/thoth.rs)      ok. 40 passed; 0 failed
thoth_api (lib)                   ok. 2263 passed; 0 failed; 3 ignored
thoth_api tests/graphql_permissions ok. 13 passed; 0 failed
thoth_api_server (lib)            ok. 5 passed; 0 failed
thoth_client (lib)                ok. 4 passed; 0 failed
thoth_errors (lib)                ok. 15 passed; 0 failed
thoth_export_server (lib)         ok. 144 passed; 0 failed
doc-tests                         thoth_client 6 passed; thoth_export_server 2 passed;
                                  thoth_api 8 ignored; others 0
exit status 0
```

The refresh adds seven `thoth_api` widget tests (section 17); the rest of the
growth from the fix pass's 2222 is B2's own tests on `develop`.

### Focused widget and regression tests

```text
cargo test --workspace --lib -p thoth-api -- model::metric_widget::
  ok. 60 passed; 0 failed (53 + 7 new)
cargo test --workspace --lib -p thoth-api -- graphql::metric_widget_tests::
  ok. 9 passed; 0 failed
cargo test --workspace --lib -p thoth-api -- model::metric_dashboard::
  ok. 98 passed; 0 failed; 1 ignored (the existing #[ignore] evidence test);
  metric_dashboard/tests.rs byte-identical to af6719f6
cargo test --workspace --lib -p thoth-api -- graphql::metric_dashboard_tests::
  ok. 23 passed; 0 failed
cargo test --workspace --lib -p thoth-api -- graphql::metric_registry_tests::
  ok. 12 passed; 0 failed
```

With `--workspace`, cargo runs the lib tests of every workspace crate with
the filter; the counts above are the `thoth_api` lib binary's, and every
other crate's lib binary ran 0 tests. The switch control tests
(`metric_widget_control`, 9) and the `thoth-api-server` status tests
(`a_widget_refusal_is_http_200_with_an_errors_entry`,
`an_invalid_widget_date_is_http_400`) ran and passed in the full workspace run
above. The refresh's five new database tests and the delta D1 test ran three more
times on their own: 6 passed each time.

### Other required checks

- `git diff --check af6719f6`: no output.
- `git diff --stat af6719f6`: only budget paths (21 files).
- `metric_dashboard/tests.rs` is byte-identical to `af6719f6`
  (`git diff af6719f6 -- thoth-api/src/model/metric_dashboard/tests.rs` is
  empty), and every test in it passes unmodified.
- Statement ceiling: 19 named statements, 22 captured. The longest path's
  captured sequence over March 2026 is exactly `CHECKOUT_CHECK, BEGIN,
  CUSTOM_PLANS_SQL, METRIC_WIDGET_STATEMENT_TIMEOUT_SQL, PROJECTION_LOCK_SQL,
  METRIC_WIDGET_RESOLUTION_SQL, METRIC_WIDGET_PAIRS_SQL,
  METRIC_WIDGET_PAIRS_SQL, METRIC_WIDGET_LABELS_SQL`, then twice
  `PUBLISHERS_SQL, ACCOUNTS_SQL, CANONICAL_SQL, MONTHS_SQL, ASSERTIONS_SQL,
  IDENTIFIER_QUALITY_SQL`, then `COMMIT`
  (`the_longest_path_runs_exactly_the_bounded_statement_sequence`). Over
  January 2026 to January 2027 it is the same sequence with
  `MONTHS_YEAR_ONLY_SQL` in place of `MONTHS_SQL` in both engine runs
  (`the_january_to_january_longest_path_reads_years_only_in_both_engine_runs`).
- Profile values: 2 slots per process, 1 s checkout timeout,
  `SET LOCAL statement_timeout = '5s'`, 10 s deadline from taking the slot,
  12 months, 500 child works, 500 `chapterDois`, 40 pairs per section
  (`the_profile_values_are_the_specified_bounds`); the date floor and ceiling
  equal `METRIC_DASHBOARD_EARLIEST_DATE` and
  `METRIC_DASHBOARD_MAX_FUTURE_DAYS`
  (`the_date_floor_and_ceiling_are_the_dashboards_technical_domain`).
- The three changed existing assertions, and the only changes to existing
  tests outside this PR's own files, re-checked against `af6719f6`:
  1. `graphql/metric_dashboard_tests.rs:1090` at `af6719f6`,
     `the_sdl_declares_exactly_the_approved_read_contract`: the Metrics read
     field count is 4, including `metricWidget` (was 3).
  2. `graphql/metric_dashboard_tests.rs:1376` at `af6719f6`,
     `deferred_dashboard_surface_is_absent`: `metricWidget` and
     `MetricWidget` leave the absent list; the remaining single entry is
     asserted directly, since a one-element loop fails
     `clippy::single_element_loop`.
  3. `graphql/metric_registry_tests.rs`,
     `the_sdl_exposes_exactly_the_nine_approved_operations`: a
     `MET-WP4-WIDGET-01` group `["metricWidget"]` excluded beside
     `wp4_02_query_fields`, an "appears exactly once" assertion, and the
     comment updated.
  B2 changed neither test, so the merge carried these edits unchanged.
  `metric_dashboard/tests.rs` is unmodified.
- Coverage-run identities (four budget files are manifest entries; the
  manifests are unchanged, the content identities change as expected),
  recomputed from `producer_manifest.txt` and `verifier_manifest.txt` with
  the test's formula. B2 changed no manifest entry, so the values at
  `af6719f6` equal those at `cab25eba`; the refresh changes no manifest entry,
  so the values at its head equal the fix pass's:

| | Before (`af6719f6`, equal to `cab25eba`) | After (refresh head, equal to `f58d0905` and the fix pass) |
| --- | --- | --- |
| Producer manifest | `eba01150ba2fcb72711bc495ed5d761752b582db3781eb32ff4b9761bb97b013` | unchanged |
| Producer identity | `14eb74146962c1ac782ccf90fedd8c1721da17555583935f1db3ddf1f5d6e115` | `f7b39bdb86f5b3d3af6f1681a90d6cd4aacd730eaa8610375b73b9e87e4413a3` |
| Verifier manifest | `2041df342dcb06ee39381d75a2550df9adbb1641a9395b480307cbe2f4b08307` | unchanged |
| Verifier identity | `ea38d0693793477102400f117b5e1d672022ad6570e421b873a654c300e08ec2` | `e89401f4ff46fa0faa2bc54a2ba586327e517497da8f4ca805224a3ca7dfca35` |

## 10. Manual verification

Environment: local disposable database only.
Steps: none beyond the automated suite; no server was started.
Observed result: n/a.
Evidence: section 9.

## 11. CI

CI status: recorded on the pull request at the exact head; GitHub is the live
authority.
Checks: build-test-and-check.
Recorded history: at `f58d0905` the `lint` job failed on the deprecated
`AtomicUsize::fetch_update` under the runner's newer stable toolchain
(section 9); the fix pass replaces the call (section 16, F1). The refresh
passes clippy locally on 1.97.0 and 1.99.1 (section 9).
Failures or warnings at any later head: recorded on the pull request.

## 12. Rollout and rollback

Initial state after merge: inactive; `THOTH_METRIC_WIDGET` defaults to `OFF`,
so every `metricWidget` selection is `METRIC_WIDGET_UNAVAILABLE` with no slot,
connection or statement.
Activation required: a separate activation grant per environment, which is
CRITICAL unless the CTO decides otherwise at that grant (`ADR-0014` s4.7),
after the WAF rate rule, `MET-READ-CACHE-01` or a named waiver, the S5 audit,
the S4 worker-exposure acceptance, native-grain data readiness and the
activation signals (spec section 11).
Feature flag/configuration: `THOTH_METRIC_WIDGET` / `--metric-widget`, read
once at start; changing it needs a restart.
Migration sequence: none.
Rollback/disable procedure: before activation, revert; after, switch off and
restart.
Monitoring required: the info-level outcome lines (rates, latency, per-book
served pairs and cross-product sizes), warn lines for pool exhaustion and
error lines for `INTERNAL_ERROR`.
Runbook note: B2's warn line "metricDashboard read stopped: the whole-read
deadline passed before a statement could start" can come from a widget read,
since the widget runs B2's engine under B2's `ReadDeadline` (with its own
10 s instant). It is written at most once per read, only for a deadline
refusal inside the engine. The same request's widget lines tell the two
apart: a widget read writes an info outcome line `METRIC_WIDGET_BUSY` at
step 13 and a debug reason line `DeadlinePassed`. Do not count that warn
line as a dashboard-only signal.

## 13. Known limitations and deferred work

- At most 12 whole months per read, no all-time or by-year view until
  `MET-WP4-WIDGET-ALLTIME`; the widget adopts none of B2's all-time range,
  omitted start, `YEAR` grain, concurrency limit, 20-second deadline or
  `METRIC_READ_BUSY`.
- Native-grain history is not served; a native-grain record on any served
  platform x served measure makes the read unavailable (deliberate
  over-refusal).
- The engine's per-section cross product (up to 40 x 40) and its
  `ASSERTIONS_SQL` row volume are bounded only by the pair bound; the staging
  reference records them.
- Response caching is `MET-READ-CACHE-01`.

## 14. Unresolved issues

- None known to the implementing agent at the refresh. Review findings and
  their dispositions are in sections 16 and 17; later review state is on the
  pull request.

## 15. Agent self-assessment

The agent may identify risks but may not approve the task.

Suggested review focus: the `evaluate` extraction and pair mode on B2's
engine in `metric_dashboard/mod.rs` (dashboard semantics unchanged; the
unmodified `metric_dashboard/tests.rs` passes), the three pair-mode row
filters and their position before B2's range and overlap checks, the
`ReadDeadline` additions and the widget's own admission against the same
instant, the error mapping, the step 3 claim counter under fragments, and the
equivalence oracle, including the cross-pair, run-overlap and
January-to-January cases.

## 16. Review fixes

The exact-head reviews of `f58d0905` (an Opus 5.5 max leg, findings F1-F6,
and a Fable leg, findings F-01 to F-07) both required changes. Each finding
and its disposition:

| Finding | Severity | Disposition | Commit |
| --- | --- | --- | --- |
| Opus F1 / Fable F-01: CI `lint` fails on the deprecated `AtomicUsize::fetch_update` | P1 | Fixed. `try_take_slot` takes a slot with a `compare_exchange_weak` loop that never takes one when `METRIC_WIDGET_SLOTS` are in use. No `#[allow(deprecated)]` and no `try_update`. The slot tests pass unchanged; clippy passes on 1.97.0 and 1.99.1 (section 9). | `8e5f62f0` |
| Fable F-02: the 500-child bound applies after deduplication | P2 | The premise does not hold: `work_relation_relator_related_uniq UNIQUE (relator_work_id, related_work_id)` (`thoth-api/migrations/20250000_v1.0.0/up.sql:3932`) allows one `has-child` row per child of a relator, so the deduplicated count equals the row count. Hardened anyway: a repeated child ID is `INTERNAL_ERROR` (`UnexpectedRow`), so the bound never rests on deduplication. Pure unit test `a_repeated_child_row_is_internal_never_deduplicated`. The SQL is unchanged. | `4ecd79eb` |
| Opus F2 / Fable F-03: the equivalence oracle has no unserved cross pair | P2 | Fixed. `a_served_pair_beside_an_unserved_cross_pair_equals_its_single_pair_read` serves (platform, sessions) and (other platform, units), each completely covered with countries, and puts an unapplied delta, a `PARTIAL` coverage assertion and unresolved identifier evidence on the unserved cross pair (platform, units). A single-pair read of the cross pair shows all three (lag, partial and unresolved-identifier warnings). The widget serves exactly the two diagonal pairs in coverage, totals, timeline and countries; each equals its single-pair `metricDashboard` read (shared oracle helper, countries on, institutions off); the section is not partial and its `dataThrough` is 2026-03-31, the aggregation over the two served pairs only. | `39b2bfbc` |
| Opus F3: the engine deadline is tested only at the first engine statement | P3 | Fixed. `a_deadline_passing_between_engine_statements_is_busy_before_the_next_one` holds `metric_source_account`, releases it from a thread after 1.5 s, and reads with a 1 s deadline and the production 5 s statement timeout: `METRIC_WIDGET_BUSY` (`DeadlinePassed`), captured statements ending `PUBLISHERS_SQL`, `ACCOUNTS_SQL`, `ROLLBACK` with no `CANONICAL_SQL`, step 13, section `work`, last statement `ACCOUNTS_SQL`. The debug reason line carries no last statement (section 4.8 puts it on the error line only), so the test reads it from the facts the read records. | `39b2bfbc` |
| Fable F-04: no test reaches `FrontierMismatch` | P3 | Fixed without a new production path, hook or `cfg(test)` branch. A whole read cannot produce the mismatch, since the labels statement and the engine read `transaction_timestamp()` and the frontier row in one repeatable-read snapshot. `an_engine_frontier_other_than_the_labels_frontier_is_internal` runs the existing step 13 function directly in such a transaction: with the labels' frontier the section is served; with `asOf` or the watermark one second later it is `INTERNAL_ERROR` (`FrontierMismatch`). | `39b2bfbc` |
| Opus F4: a test comment says "No database" | P3 | Fixed. The comment now says the capture log stays empty and every hook panics. | `39b2bfbc` |
| Opus F5 / Fable F-07: this report's lint and CI sections | P3 | Fixed in this revision: toolchains with each lint result, the CI failure at `f58d0905` and its fix (section 9), and durable wording in sections 1, 3, 4.3, 11 and 14. | the report commit (section 3) |
| Fable F-05: an over-long line in `contracts.md` | P3 | Fixed. The edited clause is re-wrapped to the paragraph's width; its wording is unchanged and nothing else in the file changes. | `e220981a` |
| Fable F-06: the orchestrator-side report's focused test commands | P3 | Fixed outside the repository, in the orchestrator's report. | none |
| Opus F6: the `CHANGELOG.md` #1008 line reads stale beside #1007 | P3 note | No change in this PR; it is for release preparation. | none |

## 17. Refresh onto B2 (R11)

B2 (`MET-WP4-03C-B2`) merged into `develop` as `af6719f6` (PR #1010) before
this PR. R11 re-verifies the specification against it; the branch was
refreshed by merging `af6719f6` (never a rebase, since the PR is shared).

Merge (`55c4e2b3`) and conflict resolution:

- `thoth-api/src/model/metric_dashboard/mod.rs`: `develop`'s file taken
  verbatim, so the merge commit itself does not compile (the widget still
  named the PR's `MetricReadDeadline`). `f15ce2a1` re-applies section 4.10 on
  B2's engine (section 4) and makes the tree compile.
- `docs/metrics/contract-register.md`: B2's "4.3 All-time history" kept as it
  is, the widget subsection placed after it as "4.4 Public widget read
  contract (`MET-WP4-WIDGET-01`)", before "## 5", and the three pointers
  re-pointed to section 4.4. B2's line on `metricWidget` in its 4.3 is
  unchanged.
- `CHANGELOG.md` and `thoth-api/src/graphql/metric_dashboard_tests.rs` merged
  textually and were checked by hand: the latter carries exactly the two
  named edits; the CHANGELOG line moved into B2's `### Added` list in
  `f4715db3`, and the PR's own heading is gone.

Existing widget tests adjusted only as B2's engine requires (`f15ce2a1`): the
named engine statements grow from 12 to 19 (B2's eight step-7 texts); the
engine message list grows from 36 to 41 constants (B2's five new ones);
`MetricReadError::Busy` joins the variant list and maps to `INTERNAL_ERROR`
(a new row, through the wildcard); the capability test passes a
`ReadDeadline`, and two direct step tests create the widget's `Deadline`,
since B2's engine takes a deadline on every run. No expected value, status,
statement sequence or error class changed. Delta D1 (`0bdcc1c0`) changes the
setup and timing of one deadline test, not what it asserts.

Review notes applied (the two R11 reviews, both APPROVED):

| Note | Severity | Disposition | Commit |
| --- | --- | --- | --- |
| Opus R11-O1: same-stream run overlap on a served pair | P2 | Kept as B2's rule and tested: `overlapping_runs_on_a_served_pair_are_internal_as_the_dashboard_refuses_them` gives `INTERNAL_ERROR` (`EngineUnavailable`, section `work`, step 13, last statement `IDENTIFIER_QUALITY_SQL`) with the fixed message and no engine message; the single-pair read of that pair fails `Unavailable` too. Register 4.4 pair mode and error table say so. | `0bdcc1c0`, `f4715db3` |
| Opus R11-O2: B2's terms in pair mode | P3 | `combinations` is the pair list; the pair-less `AMBIGUITY` row is never filtered; the widget refuses at `now >= at`, as `check()` does; `ReadDeadline`'s doc describes the widget's use. | `f15ce2a1` |
| Opus R11-O3 / Fable F-01: the total filter is a defence | P3 / P2 | All three filters are implemented as section 4.10 requires. The run filter is live and tested (`overlapping_runs_on_an_unserved_cross_pair_do_not_refuse_the_section`); the country filter is tested by direct projection rows, monthly and yearly; no end-to-end test can reach the total filter. | `f15ce2a1`, `0bdcc1c0` |
| Opus R11-O4 / Fable F-03: `metric_dashboard(` in the absence list | P3 | `the_widget_names_neither_the_dashboard_entry_nor_its_guard_nor_its_deadline` checks `metric_dashboard(`, `metric_dashboard_with_controls`, `ReadGuard`, `ReadControls`, `METRIC_DASHBOARD_READS`, `METRIC_DASHBOARD_MAX_CONCURRENT_READS`, `METRIC_DASHBOARD_SLOT_WAIT` and `METRIC_DASHBOARD_READ_DEADLINE`. | `0bdcc1c0` |
| Opus R11-O5 / Fable F-05: B2's warn line from a widget read | P3 | Recorded in register 4.4's logging bullet and in this report's runbook note (section 12). | `f4715db3`, this report |
| Opus R11-O6: the widget's date domain is stricter than B2's | P3 | Unchanged behaviour (applied to every request); `the_date_floor_and_ceiling_are_the_dashboards_technical_domain` pins the floor and ceiling to `METRIC_DASHBOARD_EARLIEST_DATE` and `METRIC_DASHBOARD_MAX_FUTURE_DAYS`. | `0bdcc1c0` |
| Opus R11-O7: a January-to-January oracle case | P3 | `each_served_pair_equals_its_single_pair_read_from_january_to_january`: 2025, both sides read `MONTHS_YEAR_ONLY_SQL`, yearly country rows pass the filter, and a yearly country row on the unserved cross pair is dropped. The longest path's January-to-January sequence is pinned too. | `0bdcc1c0` |
| Opus R11-O8: wording | P3 | Register 4.4 names section 4.3's 20-second deadline beside the widget's own 10-second deadline on the same engine mechanism. | `f4715db3` |
| Opus delta D1 | P3 | `a_deadline_passing_between_engine_statements_is_busy_before_the_next_one` calls `capturing_pool()` before `hold_lock` and holds the lock for 2 s instead of 1.5 s. | `0bdcc1c0` |
| Fable F-02: day rows are parsed before the lookup | P3 | No change: no day statement runs in pair mode (both ceiling tests pin their absence). | none |
| Fable F-04: a filter in front of the range check | P3 | The filters are the only additions to the shared body; `coverage_streams`, `segments` and `PairCoverage` are unchanged. | `f15ce2a1` |
| Fable F-06: the prelude checks in `evaluate` | P3 | They record nothing; no caller reads the value there. | `f15ce2a1` |
| Fable F-07: `CANONICAL_SQL` `$8`/`$9` | P3 | No code change: the shared body binds `$8 = start.is_none()` (false in pair mode) and `$9 = platforms or measures given` (true), exactly as a single-pair dashboard read with an explicit start, so the `START` branch is folded away. | `f15ce2a1` |

New R11 section 8 tests (`0bdcc1c0`, `model/metric_widget/tests.rs`):
`overlapping_runs_on_an_unserved_cross_pair_do_not_refuse_the_section`,
`overlapping_runs_on_a_served_pair_are_internal_as_the_dashboard_refuses_them`,
`the_widget_names_neither_the_dashboard_entry_nor_its_guard_nor_its_deadline`,
`the_date_floor_and_ceiling_are_the_dashboards_technical_domain`,
`each_served_pair_equals_its_single_pair_read_from_january_to_january`,
`a_widget_step_refusal_sets_no_engine_flag_and_an_engine_refusal_sets_it`
and `the_january_to_january_longest_path_reads_years_only_in_both_engine_runs`.
With the three pair-mode filters disabled, exactly three tests fail: the two
unserved-cross-pair country cases and the unserved-cross-pair run overlap.
