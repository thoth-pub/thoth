# MET-WP4-MULTI-ACCOUNT-01 Implementation Report

## 1. Repository state

Owning GitHub issue: thoth-pub/thoth#1030 (programme: Thoth Metrics, #766;
Metrics owning issue #981)
Repository: thoth-pub/thoth
Workflow: STANDARD
Base branch and commit: `develop` @ `da3e15a364c4d234dc94e9e03c5992dead59ed33`
PR target: `develop`
Task branch: `feature/metrics/met-wp4-multi-account-01`
Head commit: recorded on the pull request; the PR head is the authority. The
commits are listed in section 3
Pull request: opened as a draft by the orchestrator, not by the implementing
agent
Implementing model: Claude Opus 5.5, reasoning level high

## 2. Scope

Approved specification: `MET-WP4-MULTI-ACCOUNT-01` spec R2.1 (sha256
`76827884b277472a167d467189341c555299b1507bcd604b89284708dc841c02`),
implementation granted by the CTO on #1030 (comment 6870007875,
2026-11-14 02:12Z). Risk: HIGH.

Implemented objective: `metricDashboard` and `metricWidget` serve a
represented publisher with any number of eligible managed source accounts on
a served platform. Their coverage is merged day by day across every eligible
account of every represented publisher (spec section 3). Values are
unchanged. `MOM1_SOURCE_SCOPE_AMBIGUOUS` is retired.

Out-of-scope changes made: NONE. No migration, `schema.rs`, ingestion,
coverage-run derivation, rollup, GraphQL type or description change. The only
statement text change is the added `sa.source_account_id` in the
`ASSERTIONS_SQL` select list.

## 3. Commits

Every commit is authored and committed as Javier Arias <javier@jarias.org>,
with no trailers.

- `10dea818` - Combine the coverage of several eligible accounts per
  publisher and platform (engine, error retirement and every test; builds
  and tests on its own)
- `c39542c9` - Record the multi-account coverage rule in the Metrics contract
  register (register sections 4.1 to 4.4 and the `CHANGELOG.md` line)
- the commit that adds this report

## 4. What changed

`thoth-api/src/model/metric_dashboard/mod.rs`:

- Step 5 runs `ACCOUNTS_SQL` (text unchanged) and keeps its rows as a
  `BTreeSet<EligibleAccount>` of `(source_account_id, publisher_id,
  platform_id)`; the count and the refusal are removed.
- `ASSERTIONS_SQL` selects `sa.source_account_id` first; `FROM`, joins,
  predicates and parameters are unchanged. `RunRow` and `CoverageRun` gain
  `source_account_id`.
- `Evaluation` gains `accounts: &BTreeSet<EligibleAccount>`.
  `Evaluation::required_streams(P)` derives `required(P)` by iterating the
  represented publishers and counting each one's distinct eligible accounts
  on `P`, with 1 for a publisher with none. No production path passes a
  count.
- `coverage_streams` first keeps a run only if its whole triple is in the
  eligible set and its publisher is represented, then sorts by `(platform,
  measure, source_account_id, start)` and refuses overlap per account,
  platform and measure as `Unavailable`.
- `segments` and `Tally::combined` take `required`; `combined(required)`
  asserts only when `required > 0` and the open-run count equals it.
  `PairCoverage::of` takes the pair's platform as a new argument.
- `MetricReadError::SourceScopeAmbiguous`, its code and its message are
  removed. The comments that stated the one-account premise (spec section 5
  list) are corrected.

`thoth-api/src/model/metric_widget/mod.rs`: `WidgetReason::SourceScopeAmbiguous`
and its `engine_failure` arm are removed.

`docs/metrics/contract-register.md`: the spec section 5 text in sections 4.1
("Coverage", the new "Several accounts on one platform" bullet, "Errors",
"Scope boundaries"), 4.2 ("Section-specific coverage"), 4.3 ("Coverage
runs") and 4.4 (the `METRIC_WIDGET_UNAVAILABLE` prose and the error table).

`CHANGELOG.md`: the spec section 9 line, under `[Unreleased]`, in the first
`### Changed` list.

## 5. Tests (spec section 6)

`thoth-api/src/model/metric_dashboard/tests.rs` unless stated:

- T1 `several_eligible_driver_accounts_combine_their_coverage_and_ineligible_ones_are_ignored`
  replaces `more_than_one_eligible_driver_account_is_ambiguous_and_ineligible_ones_are_ignored`.
  Every ineligible case is kept. With a second eligible account and values
  on days 1 and 3: the read succeeds; `ASSERTIONS_SQL` returns both
  accounts' runs, each with its own `source_account_id`; only A asserting
  gives `UNKNOWN` every day, served values and `null` empty cells; both
  `COMPLETE` gives `COMPLETE` and `"0"`; B `PARTIAL` (day 2) and B from a
  `COMPLETED_WITH_ERRORS` import (day 4) give `PARTIAL`; `dataThrough` is
  day 1, the last day of the leading run both assert `COMPLETE`; B without
  the country dimension gives `countryCoverage` false.
- T2 `overlapping_runs_of_one_account_are_refused_and_runs_of_two_accounts_are_not`.
- T3 `every_eligible_account_of_every_represented_publisher_must_assert_a_day`
  (A1, A2, B1: all assert `COMPLETE`/`"0"`; A1 and B1 only `UNKNOWN`/`null`;
  A1 and A2 only `UNKNOWN`/`null`; B with no account `UNKNOWN`/`null`), and
  the changed block of
  `only_represented_publishers_take_part_in_coverage_and_their_assertions_combine`:
  the second account of the excluded publisher changes nothing for the
  excluded-works request; once represented, the request is served with every
  day `UNKNOWN`, and once the new account asserts `COMPLETE` everywhere the
  merged response equals the earlier two-publisher response.
- T4 `disabling_another_account_or_its_source_restores_the_baseline_and_keeps_its_values`:
  the baseline is the read before B exists; B admits one value on A's pair
  and no quarantine; disabling B, then (re-enabled) its source, restores the
  baseline's coverage, items, `dataThrough`, warnings and `isPartial`, while
  B's value is still served.
- T5 `graphql::metric_dashboard_tests::every_bounded_failure_carries_its_frozen_classification`:
  with the fixture account asserting, the read is `COMPLETE`; with a second
  eligible account it has data, no error, and coverage `UNKNOWN`.
- T6 `metric_widget/tests.rs`: `engine_variants` and the expected mapping in
  `every_engine_error_maps_to_its_widget_classification` drop the variant;
  new `two_eligible_accounts_on_a_served_platform_are_served_and_each_pair_equals_its_dashboard_read`
  (served, sessions `UNKNOWN` after the second account's last day, units
  `COMPLETE`, and both sections equal the single-pair `metricDashboard`
  reads).
- T7 `the_interval_engine_equals_the_day_oracle_on_random_fixtures` (420
  cases). `EngineCase` carries `accounts` separately from `runs`: 0 to 3 per
  represented publisher and platform (at most one in a third of the cases),
  accounts without runs, runs of an unlisted account of a represented
  publisher (sometimes overlapping), of a publisher outside the request, and
  with an eligible account ID under another publisher or platform label,
  plus forced `Trap` shapes on one platform. `Evaluation` is built from
  `case.accounts`. `oracle_assertions` first ignores runs outside the
  eligible set, then keeps one slot per eligible account; `day_oracle`
  asserts a day only if every represented publisher has a listed account and
  every listed account's slot is filled. Neither uses a count. Overlap cases
  are injected on an existing eligible account; the new
  `Overlap::AcrossAccounts` (two accounts of one publisher on the same days)
  must be served. Bookkeeping: `(refused_inside, served_before,
  served_across) == (60, 60, 60)`, and each of these is non-zero: the five
  existing outcome counters, cases with at most one account per publisher
  and platform (86), multi-account cases (274), cases with an account
  without runs (146), served cases with overlapping ineligible runs (54),
  with mislabelled runs (132), with an outside publisher (72), and, on pairs
  whose platform has a publisher with two or more accounts, `COMPLETE` items
  (58), `null` buckets caused only by one silent account (353) and by a
  publisher without an account (291). Mutation check (not committed):
  replacing `required_streams` by the publisher count, or turning the
  eligibility filter's `&&` into `||`, fails this test.
- T8 `no_engine_structure_grows_with_the_days_of_the_range`: the existing
  case has one account per publisher and platform and keeps its sizes
  (`runs` 5,000, `events` 400, `segments` 5, ...); a new two-account case
  states `runs` 10,000 and `events` 800, the rest unchanged.
- T9: no expected value changes in any test outside the list above; the
  whole library suite passes (section 7).
- T10 `an_all_time_read_starts_at_the_earliest_account_and_is_unknown_before_the_later_one`.
- T11 `each_measure_combines_only_the_accounts_that_assert_it`.

### Forced mechanical edits outside the tests section 6 names

- `metric_dashboard/tests.rs:1184` and `:1190`,
  `an_unrepresentable_aggregate_fails_rather_than_wrapping`: the `Evaluation`
  literal gains `accounts: &BTreeSet::new()` and the `PairCoverage::of` call
  gains the platform argument `Uuid::nil()`. Forced by the new field and
  argument; no assertion changes (named in spec section 6).

No other existing test changed. `run_rows` and the raw-winner tests read
`RunRow` by field and need no edit.

## 6. Deviations

None.

## 7. Validation

Local PostgreSQL 16 and Redis, UTF-8 `thoth_test` database;
`TEST_DATABASE_URL=postgres://thoth:thoth@localhost:5432/thoth_test`,
`TEST_REDIS_URL=redis://localhost:6379`,
`THOTH_EXPORT_API=https://export.thoth.pub` (the CI value, needed to compile
`thoth-export-server`).

```text
cargo fmt --all -- --check                                          exit 0
cargo clippy --workspace --all-targets --all-features -- -D warnings exit 0
  (only cargo's future-incompatibility notice for proc-macro-error2)
cargo test -p thoth-api --features backend --lib                    exit 0
  2339 passed; 0 failed; 3 ignored (1165.71s)
git diff --check da3e15a3                                           exit 0
```

Named tests (`cargo test -p thoth-api --features backend --lib -- <names>`):
the eleven tests above plus
`an_unrepresentable_aggregate_fails_rather_than_wrapping` and
`no_engine_structure_grows_with_the_days_of_the_range`: 13 passed, 0 failed.

CI on the pull request is checked by the orchestrator.

## 8. Rollout and rollback

No migration, configuration or data change. The read relaxes one refusal;
every request served before gets the same response apart from `asOf`.
Rollback is a revert, which brings the refusal back for Casemate. Before the
production deploy that carries this stage, PRE-1 (spec section 7) and the F1
runbook line are required; both are outside this repository change.

## 9. Review focus

- `coverage_streams`: the eligibility filter runs before the overlap check,
  and the overlap key is the account.
- `Evaluation::required_streams`: iterates the represented publishers, so a
  publisher without an account contributes 1.
- `Tally::combined`: no false `COMPLETE` when an eligible account has not
  asserted the day (T1, T3, T7).
