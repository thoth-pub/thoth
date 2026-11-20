# MET-WP4-MONTH-01 Implementation Report

## 1. Repository state

Owning GitHub issue: thoth-pub/thoth#1042 (programme: Thoth Metrics)
Repository: thoth-pub/thoth
Workflow: CRITICAL
Base branch: `develop`
Authorized base commit: `94d504e0ac6d2a886c01034b0aa18aed4d1c4901`
Actual base commit: `94d504e0ac6d2a886c01034b0aa18aed4d1c4901`
PR target: `develop`
Task branch: `claude/project-thread-wquopu`. The implementation brief names
this branch, and it governs over the specification's
`feature/metrics-reader/met-wp4-month-01`
Head commit: recorded on the pull request; the PR head is the authority. The
commits are listed in section 3
Pull request: not opened by the implementing agent
Implementing model: Claude Opus 5.5
Reasoning level: max

## 2. Scope confirmation

Approved specification: `MET-WP4-MONTH-01` R4.3 with its annex R3.3, both
checked by hash; implementation grant by the CTO, 2026-11-20 13:59Z.

Implemented objective:

- Migration `20261116_v1.12.0`: `window_start`, `window_end` and
  `succession` on `metric_source_account` with their two `CHECK`s, and the
  four derived tables `metric_native_month`, `metric_native_country_month`,
  `metric_native_institution_month` and `metric_native_refusal` (no foreign
  key, trigger or secondary index), populated in set-based statements while
  `metric_source_account` is held in `ACCESS EXCLUSIVE` mode. `down.sql`
  refuses while any window or succession is set.
- New module `metric_native_month`: the eight-statement recompute of a set
  of `(work, platform, measure, period_start)` keys, run by ingestion once
  per request and by quarantine reconciliation once per row, after the last
  canonical write and before commit; and the superuser repair
  `recomputeMetricNativeMonths`, in chunks of at most 50 works.
- Account windows and successions: `createMetricSourceAccount` takes the
  three optional fields; `updateMetricSourceAccountWindow` (superuser)
  replaces them, with one audited `UPDATE` on a real change. Both reads
  require, per day, one slot per succession or unsuccessioned account, and
  clip coverage runs to windows.
- `THOTH_METRIC_MONTH_SERVING` / `--metric-month-serving`, `OFF` by default,
  on `init` and `start graphql-api`, carried through the GraphQL `Context` to
  new `_with_month_serving` entry points. Under `OFF` both reads refuse
  `MONTH` and `REPORTING_PERIOD` records as before and run neither new
  statement.
- Under `ON`: `metricDashboard` and `metricWidget` read `NATIVE_MONTHS_SQL`
  (step 3b) and serve a native month only in a value whose dates contain all
  of it; a month whose days are also reported serves the days and is marked
  incomplete; projected refusals and native ambiguity fail closed; an
  all-time read judges each pair from its own first evidence
  (`PAIR_STARTS_SQL`).
- Contract register (sections 2.2, 3.1, 3.6, 3.8, 4.1 to 4.4 and the checks
  V-MONTH, V-NATIVE, V-CONFLICT and V-PLAN), `decisions.md`, the repository
  map and `CHANGELOG.md`.

Out-of-scope changes made: NONE

## 3. Commits

- `963f8ae46bc85973a4e4f5fbebc2b6d317ee19a0` - MET-WP4-MONTH-01: Serve
  monthly Metrics records behind a switch (code, tests and the contract
  register, which the tests read)
- `3a732bff4c87ea063956ff0488c14a8319bdaa02` - MET-WP4-MONTH-01: Record
  native monthly serving and account windows (`decisions.md`, the
  repository map, `CHANGELOG.md`)
- `2a05ee188255c1680865425cd215b5fbc6aeb9fd` - MET-WP4-MONTH-01: Add the
  implementation report (its first version)
- `ada5b121a40bbc611836e94fb0e32e59507c4ab0` - MET-WP4-MONTH-01: Complete
  the acceptance tests (the tests the first version of this report listed
  as missing or partial: the three test files, and in
  `metric_native_month/mod.rs` only the `#[cfg(test)]` module's
  visibility; see section 9)
- `6bbba388a01e3d27e21c1fabb9b213f33d522f4d` - MET-WP4-MONTH-01: Count
  COMPLETE pairs in T14 (one counter in T14)
- the commit that carries this revision of the report

Every commit is authored and committed by the repository maintainer's
identity, with no trailers. The code and its tests are one commit, and the
later test commit changes no production code, so every commit builds and
tests.

## 4. Files changed

The 33 paths of R4.3 section 10, and no other:

| # | Path | Kind |
|---|------|------|
| 1 | `thoth-api/migrations/20261116_v1.12.0/up.sql` | new |
| 2 | `thoth-api/migrations/20261116_v1.12.0/down.sql` | new |
| 3 | `thoth-api/src/schema.rs` | changed |
| 4 | `thoth-api/src/model/mod.rs` | changed (the new module's line) |
| 5 | `thoth-api/src/model/metric_native_month/mod.rs` | new |
| 6 | `thoth-api/src/model/metric_native_month/tests.rs` | new |
| 7 | `thoth-api/src/model/metric_ingestion/mod.rs` | changed |
| 8 | `thoth-api/src/model/metric_identifier_quarantine_reconciliation/mod.rs` | changed |
| 9 | `thoth-api/src/model/metric_dashboard/mod.rs` | changed |
| 10 | `thoth-api/src/model/metric_dashboard/tests.rs` | changed |
| 11 | `thoth-api/src/model/metric_widget/mod.rs` | changed |
| 12 | `thoth-api/src/model/metric_widget/tests.rs` | changed |
| 13 | `thoth-api/src/model/metric_source_account/mod.rs` | changed |
| 14 | `thoth-api/src/model/metric_source_account/crud.rs` | changed |
| 15 | `thoth-api/src/model/metric_source_account/tests.rs` | changed |
| 16 | `thoth-api/src/graphql/mutation.rs` | changed |
| 17 | `thoth-api/src/graphql/model.rs` | changed |
| 18 | `thoth-api/src/graphql/query.rs` | changed |
| 19 | `thoth-api/src/graphql/metric_dashboard_tests.rs` | changed |
| 20 | `thoth-api/src/graphql/metric_widget_tests.rs` | changed |
| 21 | `thoth-api/src/graphql/metric_source_registry_tests.rs` | changed |
| 22 | `thoth-api/src/graphql/metric_registry_tests.rs` | changed |
| 23 | `thoth-errors/src/database_errors.rs` | changed |
| 24 | `thoth-api-server/src/lib.rs` | changed |
| 25 | `src/bin/arguments/mod.rs` | changed |
| 26 | `src/bin/commands/mod.rs` | changed |
| 27 | `src/bin/commands/start.rs` | changed |
| 28 | `src/bin/thoth.rs` | changed |
| 29 | `docs/metrics/contract-register.md` | changed |
| 30 | `docs/metrics/decisions.md` | changed |
| 31 | `docs/engineering/repository-map/contracts.md` | changed (one line) |
| 32 | `docs/engineering/ai-delivery/implementation-reports/MET-WP4-MONTH-01-implementation-report.md` | new (this report) |
| 33 | `CHANGELOG.md` | changed (one entry) |

### 4.1 Write-budget compliance

PASS: 33 of 33 budget paths, `git diff --name-only 94d504e0` lists exactly
these.

### 4.2 Authorized actions actually used

- repository inspection: yes
- source edit, new files: yes
- file deletion/move/rename: no
- branch creation: yes (in a worktree)
- commit: yes
- push: yes, `git push -u origin claude/project-thread-wquopu`, never forced
- PR creation/update, issue or comment mutation: no
- migration execution: local disposable test database only
- release, merge, deployment, production activation: no (the switch stays
  `OFF` everywhere)

Unauthorized actions performed: NONE. External writes other than the push:
NONE.

## 5. Implementation decisions

Where R4.3 or its annex left a choice, the reading taken is the one that can
never give a false `COMPLETE` or `"0"`, a double count, or a change under
`OFF`:

1. The `month_start` `CHECK (EXTRACT(DAY FROM month_start) = 1)` is on all
   three month tables: "the same four" columns of the country and
   institution tables carry it.
2. The window fields are validated before any database read, so a malformed
   window or succession is refused with its fixed message even for an
   unknown source or platform; the two `CHECK`s map to the same messages.
3. A total-ambiguous `metric_native_month` row has `value` `NULL` and both
   dependency flags false, as the register states.
4. A refusal row's `inconsistent` is true when any record of that period is
   inconsistent; a period holding only an active reporting period has
   `inconsistent` false.
5. Ingestion collects the keys in `apply_canonical_application` on every
   `Winner` and `Revision` outcome of a non-`DAY` record and recomputes once
   per request after its last canonical write; quarantine reconciliation
   recomputes once per resolved row, in the row's transaction. An empty key
   set runs no statement.
6. The repair: an unknown publisher fails as not found before any write;
   `works` counts every work recomputed (whether or not it holds native
   rows) and `rows` the rows inserted into the four tables; it records no
   audit entry.
7. Only accounts with a window or a succession enter the windows map; an
   account with neither is an unbounded slot of its own, as today.
8. The pair start `S` is the pair's first evidence (values, native months,
   or clipped coverage), never earlier than the served start. Segments
   before `S` are skipped for status, `dataThrough` and the two dimension
   flags but stay unjustified. `S` changes no value: a bucket before `S` is
   `null` because its days hold no contribution and are unjustified, which
   in a database read they always are, `S` being the first evidence (T8's
   oracle states the rule the same way). Cut days are unjustified wherever
   they fall, and on or after `S` they break the run and make a `COMPLETE`
   segment `PARTIAL`.
9. `apply_native` fails closed (`INTERNAL_ERROR`) if the statement's
   wholly-inside flag disagrees with the engine's own window and buckets,
   and a read whose earliest pair start is not its served start fails the
   same way.
10. The institution limit is checked again after native rows are folded in,
    and native ambiguity is checked after step 7, so a native section can
    neither exceed the limit nor be served when ambiguous.
11. In widget pair mode `PAIR_STARTS_SQL` runs for every all-history section
    that has a served pair; the multi-pair earliest-start check runs in
    dashboard mode only, where the read itself has several pairs.
12. A native row of any value, zero or total-ambiguous included, makes its
    pair represented for widget pair discovery, so the pair is served and
    refused or marked as its cells require rather than hidden.
13. `metric_dashboard` and `metric_widget` keep their signatures and mean
    `OFF`; since production now calls only the `_with_month_serving`
    variants, the two `OFF` entry points are compiled for tests only, which
    keeps `-D warnings` clean. The dashboard resolver imports its entry point
    as `read_metric_dashboard`, as the widget resolver already imported
    `read_metric_widget`, so the guarded call stays one expression.
14. The GraphQL block of the register names the window fields' scalar as
    `Date`, the scalar every other date of the API renders as.
15. The test harness of `thoth-api-server/src/lib.rs` registers the switch at
    its default `OFF`, as it registers the widget switch.
16. Register placement: the scope-boundary sentence is in 4.1's scope list,
    the descriptions bullet in 4.2, 4.4's text replaces the frontier-key
    sentence, V-NATIVE is the same text in sections 2.2 and 3.6 (a test
    compares them), V-PLAN takes `$1` to `$4`, the CTO records standing
    cells per environment, and the V-MONTH comment points at section 9's
    preconditions.
17. T16 case (a): `OFF` refuses the monthly pair, so the case asserts the
    expected values and the `OFF` refusal instead of an `ON`/`OFF`
    equality. T16's per-item oracle is the all-time read restricted to the
    pair, at the same grain.

## 6. Database and migration effects

One migration, `20261116_v1.12.0`. `up.sql` adds three nullable columns and
two `CHECK`s to `metric_source_account` (every existing row `NULL`, so every
response without a window or succession is unchanged) and the four derived
tables, populated from every non-`DAY` record. `down.sql` takes the account
lock first and refuses with a fixed message while any account has a window
or a succession; otherwise it restores the catalog, and `up.sql` re-runs.
The stream, the 03A tables and `metric_record`'s indexes are unchanged.

## 7. API and compatibility effects

- `MetricSourceAccount` and `NewMetricSourceAccount` gain `windowStart`,
  `windowEnd` and `succession`; new mutations
  `updateMetricSourceAccountWindow` and `recomputeMetricNativeMonths`
  (`MetricNativeMonthRecomputeResult { works, rows }`).
- The `metricDashboard` description and the coverage descriptions of both
  reads gain the `ON` sentences. Under `OFF` every response differs from
  today's only in `asOf`.
- Statement ceilings: the dashboard's longest path stays 18 under `OFF`; it
  is 19 under `ON` and 20 for an all-time `ON` read of two or more pairs.
  The widget stays 19 named and 22 captured under `OFF`; under `ON` it is 21
  and 24, and 23 and 26 for an all-history read.

## 8. Authorization and security

Both new mutations are superuser only, through the one Metrics
administration guard, authorized before any operation-specific read, lock or
write (the guard count is now twelve). The reads keep their guards. No
secret, identity or log line changes.

## 9. Tests and checks

Local runs, as the brief limits them: targeted tests only, against a
disposable UTF-8 PostgreSQL 16 test database and a local Redis, with
`CARGO_INCREMENTAL=0` and no debug info. The full suite runs on CI.

### Formatting and lint

- `cargo fmt --all -- --check`: clean.
- `cargo clippy -p thoth-api -p thoth-errors -p thoth-api-server -p thoth
  --all-targets --all-features -- -D warnings`: clean, after two findings in
  this change were fixed (a type alias for the requirement slot key, and a
  `clone` of a `Copy` timestamp in a test helper). Clean again with the
  completed acceptance tests, after two findings in them were fixed (a type
  alias for one served pair's answer, and iterating a map's values).
- `git diff --check`: clean.

### Targeted tests

- `cargo test -p thoth-api --features backend --lib` filtered to the
  modules this change touches or relies on (`metric_dashboard::tests`,
  `metric_widget::tests`, `metric_native_month::tests`,
  `metric_source_account::tests`, `graphql::metric_registry_tests`,
  `graphql::metric_source_registry_tests`, `graphql::metric_dashboard_tests`,
  `graphql::metric_widget_tests`, `graphql::metric_rollup_tests`,
  `metric_ingestion::tests`, `metric_identifier_quarantine_reconciliation::tests`,
  `metric_record::tests`, `institution::ror_merge_tests`, and the pinned
  relation, index and register tests of `metric_rollup_delta` and
  `metric_coverage_run`): 498 passed, 3 failed, 1 ignored. The three
  failures were pins of this change (the rendered SDL of the window fields,
  `Date` and comma-separated arguments, and the aliased resolver call); after
  the pins were corrected the GraphQL suites passed (12 passed). The
  ingestion and reconciliation suites pass unchanged although their `MONTH`
  fixtures now also write native rows.
- After T1's stream test and T12's size case were added: both pass, with the
  three GraphQL suites (47 passed before the comma fix, then 12 passed).
- `metric_dashboard_query_plan_and_latency_evidence` (ignored by default;
  edited for the new `CANONICAL_SQL` parameters), run with `--ignored`:
  1 passed (916 s).
- `cargo test -p thoth-errors`: 16 passed.
- `cargo test -p thoth` (T13, the switch control matrix and the other
  controls): 54 passed.
- With the completed acceptance tests, `cargo test -p thoth-api --features
  backend --lib` filtered to each module whose tests changed:
  `metric_native_month::tests` 12 passed; `metric_widget::tests` 97 passed;
  `metric_dashboard::tests` 127 passed, 1 ignored (the plan and latency
  evidence test above, not changed since). T11's plan test passed three
  runs in a row (under two seconds each). T14 was then given its `COMPLETE`
  counter, and `metric_native_month::tests` passed again (12 passed).

### New tests

- `metric_native_month/tests.rs`: T15 (randomized projection exactness
  against an oracle from plain reads, V-NATIVE after every request and on
  injected drift), its named reporting-period case, the eight statements
  only for a request that writes a native key, rollback on a failure in any
  native table or an overflow, concurrent requests on one cell, a month
  resolved by quarantine reconciliation, the repair (chunks, superuser only,
  unknown publisher), T3 (catalog, population, `down.sql` refusal and
  `up.sql` re-run), T1's stream part, and the register checks V-MONTH,
  V-CONFLICT and V-PLAN.
- `metric_dashboard/tests.rs`: T6 and T7 cases (a month counted once and
  added to other works' days, a conflicted month served from its days and
  cleared by zeroing either side, lag and conflict without a double count,
  cut months never `COMPLETE`, refusal precedence and ambiguity), T16 (pair
  starts: monthly and later daily pairs, two daily pairs, values before the
  first run, starts set by terminal-import coverage, quarantine and a lag
  day, a cross-product pair, a second publisher, and the disagreement check),
  T4 and T11 statement sequences for every listed form, T9 (successions,
  windows alone, two publishers sharing a label, a second site from its
  go-live day) and T12's size case.
- `metric_widget/tests.rs`: T10 (a monthly-only pair served and equal to its
  dashboard read, the all-history start, `REPORTING_PERIOD` unavailability,
  a conflict served `PARTIAL` in both sections, the statement ceilings, and
  an all-history pair judged from its own first evidence).
- `metric_source_account/tests.rs`: T9's administration cases (refusals
  before any read, no-op without audit, one audited change, `null` clearing,
  an unsupported stored configuration, the two `CHECK` messages).
- `src/bin/thoth.rs`: T13.

The acceptance tests completed after the first version of this report
(commit `ada5b121a40bbc611836e94fb0e32e59507c4ab0`, and
`6bbba388a01e3d27e21c1fabb9b213f33d522f4d` for T14's `COMPLETE` counter),
by file, name and what each asserts:

- `metric_dashboard/tests.rs`:
  - T8, `the_interval_engine_equals_the_oracle_with_native_months_windows_and_pair_starts`:
    480 seeded fixtures of the interval engine (the existing generator plus
    native months that are zero, cut, conflicted or wholly inside, at every
    grain and on ranges with no complete month; windows and successions,
    unbounded, open-ended, with gaps, two accounts of one succession with
    adjacent or overlapping windows, one label shared by two publishers, a
    window without a succession, runs outside windows; and a start per pair:
    none, the served start, later, inside an `UNKNOWN` segment) equal an
    engine-independent day oracle in totals, buckets, statuses,
    `dataThrough`, both dimension flags and warnings, and every outcome T8
    names is reached at least once.
  - T9, `windows_alone_never_raise_a_status_or_justify_a_zero_over_the_t8_fixtures`:
    over T8's generator, seed and traps, no setting of windows alone, on any
    account, raises any day's status or justifies a zero that was not
    justified.
  - T6, `every_dimension_variant_in_either_order_serves_the_day_side_alone`:
    each of the seven native variants of 4.2 (undimensioned, country only,
    an aggregate with a country, institution only, all three dimensions,
    total-ambiguous, country-ambiguous) meets an undimensioned, a country and
    an institution day side, native first and day first (42 months), beside
    another work's month: the total, the `MONTH` and `YEAR` buckets and both
    sections are the day side plus the other work's month, nothing refuses,
    a `DAY` read cutting a month serves the day side alone, `OFF` refuses as
    today, and over this database V-NATIVE is 0, V-CONFLICT 42, V-PLAN
    non-zero exactly on plans whose first or last day lies in a live month,
    and V-MONTH clear.
  - T6, `a_day_side_is_live_by_any_03a_row_and_a_month_netting_to_zero_everywhere_adds`:
    a 03A ambiguity row with zero values, an undimensioned 0 beside a country
    row, a signed month whose countries net to 0 and a zero total beside an
    institution row each conflict their month (served from its days, never
    `COMPLETE`); a signed month whose every 03A row nets to 0 is not live and
    both sides add, each once; V-CONFLICT counts exactly the four live day
    sides.
  - T6, `country_only_native_rows_give_their_sum_and_a_country_dependency`:
    country-only native rows give their sum and a country dependency
    (`PARTIAL` without country coverage); an aggregate beside country rows
    gives the aggregate, no dependency, and the country rows as the section.
  - T6, `projected_contradictions_are_internal_and_a_withdrawn_month_is_not_served`:
    a withdrawn month is not served and the read equals `OFF`'s; each
    contradictory pointer (a superseded revision, no pointer, a retracted
    pointer beside a current revision) and a non-calendar `MONTH` record,
    once projected, are `INTERNAL_ERROR` under `ON` (the pointers under `OFF`
    too, as today); a non-additive measure represented only by months
    refuses; V-MONTH is non-zero exactly on these fixtures, V-NATIVE and
    V-CONFLICT 0.
  - T5, `named_day_only_reads_are_equal_under_on_and_off`: named `DAY`-only
    reads at every grain with both sections, the `YEAR` composition and
    all-time reads (unfiltered, filtered, pairs omitted) equal `OFF` apart
    from `asOf`; a pair starting after the served start keeps `OFF`'s values
    and its coverage equals T16's oracle.
  - T5, `every_refusing_fixture_refuses_alike_under_on_and_off`: every
    request limit, the combination, cell, implicit-platform, institution and
    resolved-work limits, evidence before the floor, and an active or
    contradictory `REPORTING_PERIOD` record refuse with the same error under
    both switch values.
  - T11, `the_native_reads_plan_through_their_keys_and_on_holds_no_native_grain_branch`:
    JSON `EXPLAIN` plans (no `ANALYZE` option) with sequential and bitmap
    scans off, on a fixture where one work has three years of daily rows on
    another platform: `NATIVE_MONTHS_SQL` reads the four native tables only
    through their primary keys and no record or revision row; each 03A scan
    of its conflict pass is an index scan of the table's identity key with
    the work in its condition, never on the inner side of a nested loop;
    `CANONICAL_SQL` under `ON` reads no `metric_record` or revision row,
    while under `OFF` it reads `metric_record`; `PAIR_STARTS_SQL` reads
    work-day rows only through `metric_rollup_work_day_work_id_day_idx`
    within one month per probe. It runs in under two seconds, so it is not
    ignored.
- `metric_widget/tests.rs`:
  - T6, `under_on_a_contradictory_or_non_calendar_month_is_internal_and_a_withdrawn_one_is_not_served`:
    through the widget, a withdrawn month is served as before it existed and
    as under `OFF`; each contradictory pointer is `INTERNAL_ERROR` under both
    values, and a non-calendar `MONTH` record is unavailable under `OFF` and
    `INTERNAL_ERROR` under `ON`, each `ON` failure logged with the
    engine-unavailable reason.
- `metric_native_month/tests.rs`:
  - T1's `ON` part, `under_on_ingested_months_are_served_at_once_with_no_lag_and_the_same_watermark`:
    `MONTH` records just ingested, with no rollup run since, are served at
    once by `metricDashboard` and the widget, `COMPLETE`, with no
    `ROLLUP_LAG` and the `rollupWatermark` of a read made before them;
    `OFF` refuses as today.
  - T2, `mixed_month_and_day_records_leave_03a_equal_to_the_per_day_oracle`:
    with `MONTH` and `DAY` records mixed (every mask, both measures, three
    works, revisions up, down and to 0) and rolled up through the real claim
    and completion, the work-day rows are exactly the `DAY` records' current
    values, the 03A and yearly tables equal the existing per-day oracles, no
    section row lacks a total or ambiguity row of its cell, and
    `verifyMetricRollupMonths` reports a match.
  - T14, `ingested_months_and_rolled_up_days_read_equal_to_an_independent_oracle`:
    a seeded plan over twelve months, three works and both measures (`MONTH`
    records over all eight masks and `DAY` records, a third revised up, down
    or to 0, eleven designed cells for conflicts, zero sides, a zero total
    beside a non-zero section row, signed months netting to 0 and ambiguity
    on either side) ingested through the real coordinator and rolled up;
    `NATIVE_MONTHS_SQL`'s `TOTAL` rows carry the oracle's sums and conflict
    flags; `metricDashboard` under `ON` (every grain, aligned and clipped
    ranges, filtered works, explicit and omitted starts, 43 reads) and the
    widget in pair mode (12 reads) equal an oracle from plain reads of
    `metric_record`, its current revisions and `metric_rollup_work_day` in
    totals, buckets, sections, statuses, `dataThrough` and refusals; over
    this database V-NATIVE is 0, V-CONFLICT the oracle's conflicted cells,
    V-MONTH clear and V-PLAN the oracle's live native cells for four plans;
    and each of its 22 counters is non-zero. With its seed: 14 conflicted
    cells, 28 served and 15 refused dashboard reads, 50 `PARTIAL` and 6
    `COMPLETE` served pairs, 27 pairs with a `null` bucket, 8 served and 4
    refused widget reads, and revisions up, down and to 0 of both grains.

### Edited existing tests (old and new pinned values)

- `graphql/metric_registry_tests.rs`: the exact mutation set gains
  `updateMetricSourceAccountWindow`; the guard count `10` becomes `12`, and
  "each of the ten Metrics administration mutations" becomes "twelve".
- `graphql/metric_source_registry_tests.rs`: the source-family mutation count
  `4` becomes `5`, with
  `updateMetricSourceAccountWindow(code:String!,windowStart:Date,windowEnd:Date,succession:String):MetricSourceAccount!`
  added to the expected SDL; the frozen `NewMetricSourceAccount` and
  `MetricSourceAccount` field strings gain
  `windowStart:DatewindowEnd:Datesuccession:String`; the guarded call sites
  gain `update_metric_source_account_window`.
- `graphql/metric_dashboard_tests.rs`: the pinned resolver call
  `metric_dashboard(&context.db, &input)` becomes
  `read_metric_dashboard(&context.db, &input, context.metric_month_serving)`;
  the two pinned coverage descriptions gain the sentence "With startDate
  omitted on a server that serves monthly Metrics, each platform and measure
  is judged from its own first evidence, not from the served startDate".
- `graphql/metric_widget_tests.rs`: the four pinned all-time descriptions
  (`isPartial`, `status`, `dataThrough`, `countryCoverage`) gain the same
  sentence.
- `metric_source_account/tests.rs`: the `CHECK` set gains
  `metric_source_account_succession_check` and
  `metric_source_account_window_check` (two to four names); the `SET`-clause
  test covers two `diesel::update(` calls (was one), the patch writing
  exactly `configuration` and `enabled` and the window update exactly
  `window_start`, `window_end` and `succession`; the lock test expects two
  `.for_update()` (was one), each in a `let current` read by exact code.
- `thoth-errors/src/database_errors.rs`: `REACHABLE` gains the two `CHECK`
  names (`14` becomes `16`).
- `metric_dashboard/tests.rs`: the pinned entry chain splits on
  `metric_dashboard_with_month_serving(` (was `metric_dashboard(`) and its
  three calls gain `month_serving`, with `db.get_timeout(` still once;
  `READ_STATEMENTS` `19` becomes `21` entries (`NATIVE_MONTHS_SQL`,
  `PAIR_STARTS_SQL`); `date_arrays` `["DIMENSION_CELLS_SQL"]` becomes
  `["CANONICAL_SQL", "CANONICAL_SQL", "PAIR_STARTS_SQL", "PAIR_STARTS_SQL",
  "DIMENSION_CELLS_SQL"]`; the tests that prepare or explain `CANONICAL_SQL`
  directly gain the types `boolean, date[], date[]` and the `OFF` values
  `FALSE` and one `NULL` window per account row; the T5 hook in
  `assert_serving_matches_the_day_oracle` also reads under `ON` and compares
  with `OFF`.
- `metric_widget/tests.rs`: `ENGINE_STATEMENTS` `19` becomes `21` entries;
  the statements absent from the `OFF` longest path gain `NATIVE_MONTHS_SQL`
  and `PAIR_STARTS_SQL`.
- `thoth-api-server/src/lib.rs`: the test harness gains
  `.app_data(Data::new(MetricMonthServing::default()))`.

Every other edit to an existing test is a signature or struct-literal update
passing `MetricMonthServing::Off` or an empty windows map, which the annex
allows without listing; no `DAY`-only expectation changed.

With the completed acceptance tests (commit
`ada5b121a40bbc611836e94fb0e32e59507c4ab0`), existing tests and helpers
changed as follows; no expected value changed:

- `metric_dashboard/tests.rs`:
  - `the_interval_engine_equals_the_day_oracle_on_random_fixtures` (T5):
    each fixture is also assembled as `ON` builds it (no native month, each
    pair starting at the served start, every account's window spelled out
    as unbounded) and must equal the `OFF` result.
  - `assert_serving_matches_the_day_oracle` (T5): the all-time read of each
    grain is also compared under `ON` with `OFF` (the explicit-range hook of
    the first version stays).
  - The day oracle of the randomized engine test (`day_oracle`,
    `oracle_assertions` and the new `oracle_pair_days`, `oracle_slots` and
    `oracle_in_window`) and `EngineCase` (three new fields, empty in the
    existing fixtures) gain windows, successions, native months and pair
    starts for T8; on the existing fixtures they compute what they computed
    before, and the existing randomized test passes unchanged.
  - `Native` and `native_record` become `pub(crate)`, for the widget's T6
    test.
- `metric_widget/tests.rs`: `assert_year_oracle` (T5) takes the request and
  ends by comparing it under `ON` with `OFF`: equal apart from `asOf` with
  an explicit start; for an all-history read, equal apart from each
  section's coverage, `dataThrough` and `isPartial`, each coverage item then
  equal to its pair's own all-time `ON` dashboard read, and the section's
  `dataThrough` and `isPartial` their minimum and OR. Its six callers pass
  the request.
- `metric_native_month/tests.rs`: the register-check helpers `v_native`,
  `v_native_total`, `v_month`, `v_conflict` and `v_plan` become
  `pub(crate)`, and `metric_native_month/mod.rs` declares its test module
  `pub(crate)`, so T6's tests run the checks as the register gives them.

## 10. Known limitations and deferred work

Every acceptance test of R4.3 section 6, with the annex's T1 and T2, is
implemented (section 9), and the register's checks run over T6's, T14's and
T15's databases. The first version of this report listed T8, T9's property
test, T14, T11's `EXPLAIN` evidence, T1's `ON` part, T2, part of T5 and part
of T6 as missing; they are the tests of commit
`ada5b121a40bbc611836e94fb0e32e59507c4ab0`.

Readings taken in the tests, each the one that cannot hide a false
`COMPLETE` or `"0"`, a double count or a change under `OFF`:

1. T2 is written as a new test, because the annex requires it, although the
   stage changes no stream or 03A code.
2. Ingestion refuses a record whose exact dimensional cell already holds an
   overlapping period (`OVERLAPPING_PERIOD`), so a `MONTH` record and `DAY`
   records naming the same publication, country and institution cannot both
   exist. T2's and T14's generator redraws a cell until its two sides share
   no dimension tuple, and both tests assert that every row was a winner or
   a revision. T6's dashboard tests write `MONTH` records directly, so they
   also hold the undimensioned-beside-undimensioned pairing that ingestion
   cannot produce; the read serves the day side there too.
3. T11's fixture adds, beside the long daily history R4.3 names, four
   hundred works outside every request with three daily rows on each pair,
   and runs the `ANALYZE` statistics command (the plans are still `EXPLAIN`
   without its `ANALYZE` option). On the fixture's three works alone, the
   planner probed `PAIR_STARTS_SQL`'s work-day rows through
   `(measure_id, platform_id, day)`, still within one month per probe; with
   a work as selective as in production it uses `(work_id, day)`, which the
   test asserts. Under `OFF` the test asserts only that `CANONICAL_SQL`
   reads `metric_record`, through whichever index the planner picks, as the
   contrast that makes the `ON` assertion meaningful.
4. `AGGREGATE_OUT_OF_RANGE` needs a sum beyond the `i128` range, which no
   database of `BIGINT` values within the read limits can produce, so T5's
   refusing fixtures cannot include it; the existing engine test of it runs
   the code both switch values share.
5. The widget always reads countries and never institutions, so T14
   compares institution sections through `metricDashboard` only.

Mutation evidence for the new tests: each mutation below was applied alone
to the production code, the tests were rebuilt and run, and the file was
restored with `git checkout`.

| Mutation of the production code | What it would let through | New tests that fail |
|---|---|---|
| A conflicted cell's native total is summed (`TOTAL` without its `NOT conflict` filter) | a double count | T14; T6 `every_dimension_variant_in_either_order_serves_the_day_side_alone` and `a_day_side_is_live_by_any_03a_row_and_a_month_netting_to_zero_everywhere_adds` |
| A day side live only through a non-zero section row is not seen | a double count | T14; T6 `a_day_side_is_live_by_any_03a_row_and_a_month_netting_to_zero_everywhere_adds` |
| A 03A ambiguity row no longer makes the day side live | a double count | T6 `a_day_side_is_live_by_any_03a_row_and_a_month_netting_to_zero_everywhere_adds` (T14 holds no day side live only through an ambiguity row) |
| Any 03A total row, even 0, makes the day side live | a native month dropped | T14; T6 `a_day_side_is_live_by_any_03a_row_and_a_month_netting_to_zero_everywhere_adds` |
| A cut day no longer makes its pair `PARTIAL` | a false `COMPLETE` | T8; T14; T6 `every_dimension_variant_in_either_order_serves_the_day_side_alone` and `a_day_side_is_live_by_any_03a_row_and_a_month_netting_to_zero_everywhere_adds` |
| Native month totals are never added | a month not served | T8; T14; T1's `ON` part; T6 `every_dimension_variant_in_either_order_serves_the_day_side_alone`, `a_day_side_is_live_by_any_03a_row_and_a_month_netting_to_zero_everywhere_adds` and `country_only_native_rows_give_their_sum_and_a_country_dependency` |
| A month running past the range's end counts as wholly inside | native sections and ambiguity beyond the range | T14 |
| Account windows are ignored | coverage outside a window counted | T8; T9's property test |
| Succession labels are ignored | the accounts of one succession required as separate slots | T8 |
| The pair start is ignored | a status or `dataThrough` judged from the served start | T8 |

T8 and T9 ran in a first pass over the engine mutations, and the database
tests (T1's `ON` part, T6's four dashboard tests, T6's widget test and T14)
in a second; the widget's T6 test, about pointers and periods, fails under
none of them, as expected.

Operational limits are those of R4.3 (sections 3.3, 4.8): one stray day
displaces a whole month's value until the CTO's repair rule resolves the
conflict, and a change made outside Thoth leaves the projection stale until
V-NATIVE finds it.

## 11. Unresolved issues

`HOLD` items: NONE.
