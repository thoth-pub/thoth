# MET-WP4-WIDGET-ALLTIME Implementation Report

## 1. Repository state

Owning GitHub issue: thoth-pub/thoth#1019 (programme: Thoth Metrics, #766)
Repository: thoth-pub/thoth
Workflow: STANDARD
Base branch: `develop`
Authorized base commit: `bc6b45bd243a44ba658de9f690a3a2d3ba3bff2d`
Actual base commit: `bc6b45bd243a44ba658de9f690a3a2d3ba3bff2d`. `develop`
had moved to `6af22824d824d14a8795602658a93397e12c8416` (#1018, Thema v1.7:
one `CHANGELOG.md` line under `### Changed` and two
`thoth-api/src/model/subject/` files), which the grant records as no HOLD;
the branch merges with it without conflict (section 9, "Other required
checks")
PR target: `develop`
Programme integration branch: none
Task branch: `feature/metrics-widget/met-wp4-widget-alltime`
Head commit: recorded on the pull request; the PR head is the authority. The
commits are listed in section 3
Pull request: opened as a draft by the orchestrator (not by the implementing
agent); its number and live state are recorded on GitHub
Expected branch deletion after merge: YES
Final programme PR required: NO
Implementing model: Claude Opus 5.5
Reasoning level: high

## 2. Scope confirmation

Approved specification: the #1019 body, R2.1 (sections 1-11), under
`ADR-0014` and the parent contract #1007 R11; implementation grant on #1019
(comment 6855165023, CTO, 2026-11-13 14:01Z), which confirms A1 (the served
start is the first served month) and A2 (no year cap, no ADR change).

Implemented objective: `metricWidget` gains an optional `timelineGrain`
(`MONTH` by default, or `YEAR`) and an optional `startDate`. A `MONTH` read is
R11's read unchanged (start required, at most 12 months). A `YEAR` read with
a `startDate` serves yearly buckets over any range inside R11's domain. A
`YEAR` read without one serves all history: a new widget-owned statement
finds each section's served pairs and their earliest month, and both sections
are read from one served start, the earliest served month of either section.
The engine (`metric_dashboard/`) is unchanged; the switch stays `OFF`.

Out-of-scope changes made: NONE

## 3. Commits

- `f0ceb2a997455baed8df4f0eed9b50cc18d34666` - Add yearly and all-history metricWidget reads
- `6337de8c2feaecdfc049c36c7a6664d55b98e03b` - Record the yearly and all-history metricWidget contract
- `b08e7262afd437df56bc347e6a81156585538c16` - Add the MET-WP4-WIDGET-ALLTIME implementation report
- `6e5133164904eb534a14f797453c4520f673070f` - Strengthen all-time widget tests
  (fix pass 1, tests only; section 9, "Fix pass 1")
- the commit that carries this revision of the report - Correct the empty
  all-history sequence in the report (fix pass 1)

The code and its tests are one commit, so every commit builds and tests. The
gates in section 9 ran on the tree of `6337de8c` (tree
`6f13283b58b71a06b734aeb70349a1f721e3b4e4`); the report commit adds only this
file, which no cargo command reads.

Every commit is authored and committed as Javier Arias
<javier@jarias.org>, with no trailers.

## 4. Files changed

Authorized write paths (spec section 9):

- `thoth-api/src/model/metric_widget/mod.rs` (sections 3 and 4)
- `thoth-api/src/model/metric_widget/tests.rs` (new tests; existing code only
  as section 9 lists)
- `thoth-api/src/graphql/metric_widget_tests.rs` (the input signature at
  `:204`, `timelineGrain` out of the input's absent list, the new cases, the
  12-month text at `:385`)
- `thoth-api/src/graphql/query.rs` (the `metricWidget` field description and
  its `input` argument description only)
- `docs/metrics/contract-register.md` (section 4.4 and one bullet of 4.3)
- `CHANGELOG.md` (one `[Unreleased]` `### Added` line)

Authorized new-file paths:

- `docs/engineering/ai-delivery/implementation-reports/MET-WP4-WIDGET-ALLTIME-implementation-report.md`

Actual files changed:

- `thoth-api/src/model/metric_widget/mod.rs`
  - reason: sections 3 and 4.
  - behavioural effect: `MetricWidgetInput.startDate` becomes `Date`
    (nullable); new input field `timelineGrain: MetricWidgetGrain =
    MONTH` (an omitted or null grain is `MONTH`); new enum
    `MetricWidgetGrain { MONTH, YEAR }`, mapped to the engine's
    `MetricTimelineGrain`. Every section 3 description is the specified text.
    Step 4 follows section 4.1's order (below). New constant
    `METRIC_WIDGET_HISTORY_PAIRS_SQL`, the section 4.4 text, run by the new
    `history_pairs` for a `YEAR` read without a start, once per section,
    admitted against the deadline under its own name; a read with a start
    runs `served_pairs` and `METRIC_WIDGET_PAIRS_SQL` as before (both byte
    unchanged). New step 11a (`served_start`): after the pair bound and
    before the labels statement, no statement, logged as `WidgetStep::Pairs`;
    the served start is the given start, or the earliest `first_month` over
    both sections, or the first day of the month before `endDate` when
    neither section serves a pair; a served start before 2000-01-01 is
    refused. `ValidatedInput` carries the grain and whether the read is an
    all-history read; `served_from` gives the copy whose start is the served
    start, which both `engine_section` calls and the response take.
    `engine_section` adds `with_timeline_grain` of the mapped grain. The
    outcome line gains `grain=`, `history=` and `buckets=` (section 5).
  - within authorized write budget: YES
- `thoth-api/src/model/metric_widget/tests.rs`
  - reason: section 8's tests and section 9's permitted edits (section 9
    below lists every edited existing item).
  - behavioural effect: tests only.
  - within authorized write budget: YES
- `thoth-api/src/graphql/metric_widget_tests.rs`
  - reason: the input signature, the absent list, the 12-month text and
    seven new tests (four before fix pass 1).
  - behavioural effect: tests only.
  - within authorized write budget: YES
- `thoth-api/src/graphql/query.rs`
  - reason: the field and argument descriptions of section 3.
  - behavioural effect: description text only.
  - within authorized write budget: YES
- `docs/metrics/contract-register.md`
  - reason: section 9's register edit.
  - behavioural effect: documentation only. The 4.3 bullet at `:1778-1779`
    now points to section 4.4; section 4.4 records the grain, the optional
    start, step 4, steps 11 and 11a, the served-start rule, the pair-mode
    step-7 family and native-overlap rules, the bounds (the `YEAR` range is
    bounded by the date domain alone), the error rows, the logging fields,
    the activation gate and the consumer rules. No other line outside
    section 4.4 changes.
  - within authorized write budget: YES
- `CHANGELOG.md`
  - reason: one line, directly after the `[1013]` line under `[Unreleased]`
    `### Added`.
  - behavioural effect: none.
  - within authorized write budget: YES

Actual new files created:

- `docs/engineering/ai-delivery/implementation-reports/MET-WP4-WIDGET-ALLTIME-implementation-report.md` - within authorized new-file list: YES

Files deleted, moved or renamed: NONE

### 4.1 Write-budget compliance

PASS. `git diff --stat bc6b45bd` lists only the six budget paths and this
report; `git diff bc6b45bd -- thoth-api/src/model/metric_dashboard/` is
empty, and no migration changes.

## 4.2 Authorized actions actually used

- repository inspection: yes
- source edit: yes
- new file creation: yes (this report)
- file deletion/move/rename: no
- branch creation: yes (`feature/metrics-widget/met-wp4-widget-alltime`, in a
  worktree)
- commit: yes
- push: yes
- PR creation/update: no (the orchestrator opens the draft PR)
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

Automatic CI/provider effects: PR CI runs on each push to the pull request;
its runs and results are recorded on the pull request (section 11).

Manually initiated external actions: NONE

External writes/publication: NONE

## 5. Implementation decisions

1. **Signatures kept.** `served_pairs` and `engine_section` keep their R11
   signatures, as section 9 prefers. The grain and the history flag ride in
   `ValidatedInput`; the history statement runs in its own function
   (`history_pairs`), and `section_pairs` chooses between the two. The
   served start reaches `engine_section` as a `ValidatedInput` copy from
   `served_from`, so the direct calls of `served_pairs` and `engine_section`
   in `tests.rs` are unchanged.
2. **Step 4 order.** The grain resolves first (`unwrap_or_default`, `MONTH`);
   then `doi`; then, with a start, R11's dates, domain and (for `MONTH`
   only) 12-month checks in R11's order; without a start, `MONTH` is
   `METRIC_QUERY_INVALID` "startDate is required for a MONTH timeline.",
   an `endDate` that is not a first of month is R11's dates message
   (`METRIC_QUERY_INVALID`), and an `endDate` not after 2000-01-01 or more
   than 366 days after today is the omitted-start domain message
   (`METRIC_QUERY_LIMIT_EXCEEDED`); then the `chapterDois` checks.
3. **The empty-history placeholder.** Until step 11a, an all-history
   request's `start` holds the first day of the month before `endDate`, the
   value the empty response serves. It is never bound to a statement:
   `history_pairs` binds only the works, `endDate` and 41.
4. **Buckets.** `buckets` is the bucket count per pair over the served range:
   for `MONTH` the months in `[start, end)`; for `YEAR` the calendar years
   from the start's year to the year of the day before `endDate`. It is set
   once step 11a fixes the start (empty responses included) and is `-`
   before that and on the floor refusal.
5. **Log fields.** `grain` and `history` are copied onto the outcome line
   only once step 4 has passed, so lines of reads refused at steps 2 to 4
   are byte-for-byte R11's.
6. **The 12-month message** changes text only; its constant, type and
   position in step 4 are R11's.

Deviation from the specification requiring authorization: NONE

## 6. Database and migration effects

Migration added: NO. No table, index or role is added; the new statement
reads the identity keys R11's pairs statement reads (section 9, `EXPLAIN`).

## 7. API and compatibility effects

GraphQL/API changes (additive):

- `input MetricWidgetInput`: `startDate: Date` (was `Date!`); new
  `timelineGrain: MetricWidgetGrain = MONTH`.
- new `enum MetricWidgetGrain { MONTH YEAR }`.
- `MetricWidget.startDate` is the served start; descriptions per section 3.
- the `metricWidget` field and argument descriptions.

New messages (fixed, widget-owned):

| Condition | `extensions.type` | Message |
| --- | --- | --- |
| `MONTH` without `startDate` | `METRIC_QUERY_INVALID` | startDate is required for a MONTH timeline. |
| `YEAR` without `startDate`, `endDate` not after 2000-01-01 or more than 366 days after today | `METRIC_QUERY_LIMIT_EXCEEDED` | When startDate is omitted, endDate must be after 2000-01-01 and at most 366 days after today (UTC). |
| Served start before 2000-01-01 (step 11a) | `METRIC_QUERY_LIMIT_EXCEEDED` | Metrics for this work or its child works start before 2000-01-01. Give a startDate on or after 2000-01-01. |

Changed message: "The date range may cover at most 12 months." becomes "A
MONTH timeline may cover at most 12 months." (`METRIC_QUERY_LIMIT_EXCEEDED`,
unchanged). Every other message and mapping is R11's. No new or changed
message equals or contains an engine message
(`no_widget_error_carries_an_engine_message_sql_or_a_package_value`
passes with the new messages in `widget_errors()`).

Generated schema/client updates: none committed; `thoth-client` regenerates
its schema at build time and builds in the workspace run (section 9).
Backwards compatibility: every R11 request is valid and answered as before
(an omitted grain is `MONTH`, and `MONTH` requires a start), apart from the
changed 12-month message text.
Deprecations: none.
Cross-repository dependencies: none; WIDGET-02 consumes this later (spec
section 7).

## 8. Authorization and security

Authorization paths changed: none. Resolution, entitlement, the uniform
refusal, child selection and exclusion, slots, checkout, the statement
timeout and the deadline are R11's.
Roles/scopes involved: none (anonymous read, gated by `METRICS_WIDGET`).
Negative authorization tests: R11's pass unchanged; the floor refusal comes
after step 9, so only an entitled publisher's work reaches it
(`a_served_month_before_2000_is_refused_after_the_pair_statements`).
Secret or personal-data handling: none. The new log fields are two fixed
enumerations and a count.
Security limitations: the `YEAR` range is bounded by the date domain alone,
so moving its floor or ceiling widens the public surface (recorded in the
register, section 4.4).

## 9. Tests and checks

Environment for every cargo command, from the worktree:
`LIBRARY_PATH=<local libpq dir> THOTH_EXPORT_API=https://export.thoth.pub
CARGO_INCREMENTAL=0`, and for tests also
`TEST_DATABASE_URL=postgres://thoth:thoth@localhost:5432/thoth_test
TEST_REDIS_URL=redis://localhost:6379` (local PostgreSQL 16 and Redis only).

### Formatting

Command:

```text
cargo fmt --all -- --check
```

Result:

```text
exit 0, no output
```

### Lint/static analysis

Command:

```text
cargo clippy --all --all-targets --all-features -- -D warnings
```

Result:

```text
exit 0, no warning from the workspace crates; the only note is cargo's
future-incompatibility notice for the dependency proc-macro-error2 v2.0.1
```

### Full workspace

Command:

```text
cargo test --workspace
```

Result:

```text
exit 0
thoth (lib)                      0 passed
thoth (bin src/bin/thoth.rs)    40 passed
thoth_api (lib)               2325 passed; 0 failed; 3 ignored (1911.74s)
graphql_permissions             13 passed
thoth_api_server (lib)           5 passed
thoth_client (lib)               4 passed (built with its regenerated schema)
thoth_errors (lib)              15 passed
thoth_export_server (lib)      144 passed
doc-tests: thoth_api 8 ignored; thoth_client 6 passed;
           thoth_export_server 2 passed; others 0
```

### Focused widget and regression tests

```text
cargo test --workspace --lib -p thoth-api -- model::metric_widget::
  86 passed; 0 failed (60 R11 tests and 26 new); after fix pass 1:
  89 passed; 0 failed (60 R11 tests and 29 new)
cargo test --workspace --lib -p thoth-api -- graphql::metric_widget_tests::
  13 passed; 0 failed (9 R11 tests and 4 new); after fix pass 1:
  16 passed; 0 failed (9 R11 tests and 7 new)
cargo test --workspace --lib -p thoth-api -- model::metric_dashboard::
  98 passed; 0 failed; 1 ignored
cargo test --workspace --lib -p thoth-api -- graphql::metric_dashboard_tests::
  23 passed; 0 failed
cargo test --workspace --lib -p thoth-api -- graphql::metric_registry_tests::
  12 passed; 0 failed
```

### Statement sequences and the cell bound

Captured sequence of an all-history `YEAR` read serving both sections, with
an eligible account (asserted by
`an_all_history_read_of_whole_years_reads_years_only_in_both_engine_runs`
and its two siblings, through `assert_the_ceiling`):

```text
CHECKOUT_CHECK, BEGIN, CUSTOM_PLANS_SQL, METRIC_WIDGET_STATEMENT_TIMEOUT_SQL,
PROJECTION_LOCK_SQL, METRIC_WIDGET_RESOLUTION_SQL,
METRIC_WIDGET_HISTORY_PAIRS_SQL, METRIC_WIDGET_HISTORY_PAIRS_SQL,
METRIC_WIDGET_LABELS_SQL,
  PUBLISHERS_SQL, ACCOUNTS_SQL, CANONICAL_SQL, <step 7>, ASSERTIONS_SQL,
  IDENTIFIER_QUALITY_SQL   (work section)
  PUBLISHERS_SQL, ACCOUNTS_SQL, CANONICAL_SQL, <step 7>, ASSERTIONS_SQL,
  IDENTIFIER_QUALITY_SQL   (chapters section)
COMMIT
```

22 captured, 19 named: R11's ceiling, unchanged. The step-7 text is the same
in both sections:

- `MONTHS_YEAR_ONLY_SQL` when the served start and `endDate` are both
  1 January (`an_all_history_read_of_whole_years_reads_years_only_in_both_engine_runs`);
- `MONTHS_YEAR_SQL` when the window holds a whole year and either end is not
  1 January (`an_all_history_read_with_a_whole_year_and_clipped_ends_reads_months_and_years`);
- `MONTHS_SQL` when it holds no whole year
  (`an_all_history_read_crossing_one_january_reads_months_only`).

A `YEAR` read with a `startDate` runs the same sequence with
`METRIC_WIDGET_PAIRS_SQL` in place of the history statement
(`a_year_read_with_a_start_runs_the_month_bounded_pair_statement`); a
500-child book read without a start keeps it
(`a_five_hundred_child_book_read_without_a_start_keeps_its_sequence`). No
`YEAR` read runs a scan text, `WORKS_SQL`, `KNOWN_SQL`, `REPRESENTED_SQL` or
a day statement (`DAY_SUMS_SQL`, `DIMENSION_CELLS_SQL`, `DAY_SECTIONS_SQL`).
With no served pair the read runs both history statements and the labels
statement, and no engine statement, serving the first day of the month
before `endDate`
(`with_no_served_pair_the_start_is_the_month_before_the_end_and_no_engine_statement_runs`);
the floor refusal comes after both pairs statements and before the labels
statement. In both cases a pairs statement runs once per section with served
works: for the `work` section always, and for the `chapters` section only
when a child work is served (the floor test,
`a_served_month_before_2000_is_refused_after_the_pair_statements`, has no
child work, so it pins one history statement and then `ROLLBACK`).

Cell bound per section: `MONTH` at most 40 pairs x 12 buckets = 480; `YEAR`
at most 40 pairs x 28 buckets = 1,120 in 2026 (2000 to 2027 inside the
domain), against the engine's 5,000, which cannot trip before 2124.
`forty_pairs_over_all_history_are_served_and_forty_one_refused_before_the_start`
checks the 40-pair bound over all history.

### Edited existing tests and test code

`thoth-api/src/model/metric_widget/tests.rs` (section 9 permission in
brackets):

- `input_for` (`:194`): `start_date: Some(start)` and
  `timeline_grain: Some(MetricWidgetGrain::Month)` [the `:194` literal];
- `input_over` (`:203`) and `dates_must_be_whole_months_inside_the_domain_and_at_most_twelve`'s
  `with_dates` (`:1281`): `Some(..)`, the rest spread [the `:203` and
  `:1281` literals];
- `start_date` assignments in
  `the_resolver_decides_its_log_lines_once_per_resolution` (`:941`),
  `the_first_input_failure_in_step_4_order_is_returned` (`:1356`, `:1366`,
  `:1372`) and `twelve_months_are_served_and_output_is_ordered` (`:2164`):
  `Some(..)` [the five assignments];
- `widget_statement_names` (`:293-331`): adds
  `METRIC_WIDGET_HISTORY_PAIRS_SQL` [`widget_statement_names`];
- `widget_errors()` (`:1068-1094`): the three new errors; and the table in
  `every_widget_error_has_its_fixed_type_and_message` (`:1158-1235`): the
  changed 12-month text and three new rows [`widget_errors()` and the
  table];
- `facts()` (`:754-771`): `grain: Some(Month)`, `history: Some(false)`,
  `buckets: Some(1)`; `fresh_facts()` (`:1624-1638`): the three fields
  `None` [the `WidgetFacts` literals];
- `single_pair_read_over`, `single_pair_input` and
  `assert_each_served_pair_equals_its_dashboard_read_over`: now `MONTH`
  wrappers around `single_pair_read_at`, `single_pair_input_at` and
  `assert_each_served_pair_equals_its_dashboard_read_at`, which take the
  grain; the oracle's assertion label also names the grain [the oracle
  helpers].

No expected value of an existing test changes apart from the message table.
No existing expected outcome line needed the new fields: the existing
outcome-line assertions check substrings (`contains`), which the appended
fields do not disturb. The direct calls of
`served_pairs` (`:2378`) and `engine_section` (`:2978-2988`), and those of
`transact` and `select_children`, are unchanged.

`thoth-api/src/graphql/metric_widget_tests.rs`:

- `the_sdl_declares_exactly_the_widget_contract` (`:204`): the input
  signature becomes
  `doi:Doi!chapterDois:[Doi!]startDate:DateendDate:Date!timelineGrain:MetricWidgetGrain=`
  [the `:204` signature];
- `the_field_description_is_one_line_and_names_no_pinned_surface`
  (`:285-305`): `timelineGrain` is skipped for `input MetricWidgetInput {`
  only; every output type still must not contain it [the absent list];
- `input_failures_are_identical_for_every_work_and_follow_step_4_order`
  (`:385`): "A MONTH timeline may cover at most 12 months." [the 12-month
  text].

New tests: 29 in `model::metric_widget::tests` and 7 in
`graphql::metric_widget_tests` (26 and 4 before fix pass 1; the section 8 list: validation order and
messages, the outcome line, the served-start rule and what does not move
it, the floor, the three step-7 families, the oracle at both grains over
all history, the past-today coverage signal, the native-overlap and
ambiguity refusals, the pair bound, the SDL, the descriptions and the
omitted and null grain and start).

### Fix pass 1

Tests and this report only; `mod.rs`, `query.rs`, the register and the
`CHANGELOG.md` are unchanged, and no test that exists at `bc6b45bd` was
edited. Every new test passed at the code of `b08e7262`.

New tests:

- `model::metric_widget::tests`:
  - `an_unknown_doi_and_an_unentitled_publisher_refuse_an_all_history_read_after_resolution`:
    an unknown-DOI and an unentitled-publisher `YEAR` read without
    `startDate` (both works hold history) each capture `CHECKOUT_CHECK,
    BEGIN, CUSTOM_PLANS_SQL, METRIC_WIDGET_STATEMENT_TIMEOUT_SQL,
    PROJECTION_LOCK_SQL, METRIC_WIDGET_RESOLUTION_SQL, ROLLBACK`, with no
    `METRIC_WIDGET_HISTORY_PAIRS_SQL`, and return the same error type and
    message (`METRIC_WIDGET_UNAVAILABLE`); the entitled book's read runs the
    history statement.
  - `an_all_history_read_of_two_whole_years_reads_years_only_in_both_runs`:
    a new fixture with values from January 2024, read over all history to
    2026-01-01: served start 2024-01-01, the 22-statement sequence with
    `MONTHS_YEAR_ONLY_SQL` in both engine runs, two year buckets and
    `buckets=2`; the explicit-start read at 2024-01-01 runs the same sequence
    with `METRIC_WIDGET_PAIRS_SQL`, also with `MONTHS_YEAR_ONLY_SQL`, and
    equals the all-history response except for `asOf`; the single-pair
    dashboard oracle at `YEAR` holds for both sections.
  - `a_country_ambiguous_month_years_before_the_end_refuses_the_all_history_read`:
    a country-ambiguous-only month (April 2023; the ambiguity row is checked
    to be `country_ambiguous` and not `total_ambiguous`) inside a whole year
    of the served range makes the all-history `YEAR` read
    `METRIC_WIDGET_UNAVAILABLE` with reason `DimensionScopeAmbiguous`; a
    `MONTH` read of 2026 is served.
- `graphql::metric_widget_tests`:
  - `a_disabled_switch_refuses_year_requests_without_the_database`: with
    `test_db::failing_pool()` and the switch off, `{doi, endDate:
    "2026-04-01", timelineGrain: YEAR}` and the same with `startDate:
    "2025-01-01"` are `METRIC_WIDGET_UNAVAILABLE` for every DOI form, so no
    checkout is taken.
  - `off_unknown_and_unentitled_year_requests_are_the_month_refusal`: the
    switched-off, unknown-DOI and unentitled responses to both `YEAR` shapes
    are byte-equal to each other and to R11's `MONTH` refusal response; the
    book is served for both shapes with the switch on.
  - `inline_null_grain_and_start_literals_equal_their_variable_cases`:
    inline `timelineGrain: null` (with a start, and without one) and inline
    `startDate: null` on a `YEAR` read, with no variables, give the same
    responses as the variable cases (served responses compared without
    `asOf`; the refusal compared by message and type, since the error's
    location differs between the two documents).

Edited tests that this stage added (none exists at `bc6b45bd`):

- `with_no_served_pair_the_start_is_the_month_before_the_end_and_nothing_else_runs`
  is renamed `..._and_no_engine_statement_runs`; its body is unchanged.
- `a_native_record_ending_on_the_served_start_does_not_refuse_the_read`: the
  explicit-start refusal is read through `resolve_with` and asserts the
  reason `UnsupportedSourceGrain`.
- `native_history_on_one_served_platform_overlapping_another_platforms_values_refuses`:
  a served control first (the same fixture before the two native records:
  served from 2025-07-01, with both pairs, and equal to its explicit-start
  read), then the refusal through `resolve_with` with the reason
  `UnsupportedSourceGrain`.
- `an_omitted_or_null_grain_is_a_month_read_and_needs_a_start` (GraphQL):
  the window is 2025-12-01 to 2026-04-01, so omitted, `null` and `MONTH`
  give 4 timeline entries per pair and are equal, and explicit `YEAR` gives
  2 (`[2025-12-01, 2026-01-01)` null, `[2026-01-01, 2026-04-01)` "5"). The
  `START_REQUIRED` half is unchanged.

Commands (same environment as above):

```text
cargo fmt --all -- --check                                     exit 0
cargo clippy --all --all-targets --all-features -- -D warnings exit 0 (only
  the proc-macro-error2 future-incompatibility notice)
cargo test --workspace --lib -p thoth-api -- model::metric_widget::
  89 passed; 0 failed
cargo test --workspace --lib -p thoth-api -- graphql::metric_widget_tests::
  16 passed; 0 failed
cargo test --workspace --lib -p thoth-api -- graphql::metric_dashboard_tests::
  23 passed; 0 failed
cargo test --workspace --lib -p thoth-api -- graphql::metric_registry_tests::
  12 passed; 0 failed
git diff --check b08e7262                                      exit 0
git diff --stat b08e7262                                       the two test
  files and this report only
```

The full `cargo test --workspace` was not re-run for this tests-and-report
pass; its result above is for the code, which this pass does not change.

### EXPLAIN of `METRIC_WIDGET_HISTORY_PAIRS_SQL`

Run on the local `thoth_test` database only (PostgreSQL 16.15, migrations
applied), in one transaction that is rolled back:

```text
BEGIN;
SET LOCAL enable_seqscan = off;
SET LOCAL plan_cache_mode = force_custom_plan;
-- fixture rows: every month 2000-01 to 2026-03 for each fixture work x
-- platform x measure (5,040 rows) and one total-ambiguous month each (16)
INSERT ...; ANALYZE ...;
PREPARE widget_history_pairs(uuid[], date, bigint) AS <the constant's text>;
SELECT array_agg(work_id)::text AS works FROM public.work \gset
EXPLAIN EXECUTE widget_history_pairs(:'works'::uuid[], '2026-04-01', 41);
ROLLBACK;
```

Command: `psql -h localhost -U thoth -d thoth_test -f explain.sql`. Plan (the
four fixture work ids shortened to `{w1,w2,w3,w4}`):

```text
Limit  (cost=783.66..801.51 rows=41 width=36)
  ->  GroupAggregate  (cost=783.66..852.43 rows=158 width=36)
        Group Key: mm.platform_id, mm.measure_id
        ->  Incremental Sort  (cost=783.66..849.67 rows=158 width=36)
              Sort Key: mm.platform_id, mm.measure_id
              Presorted Key: mm.platform_id
              ->  Merge Join  (cost=783.27..842.56 rows=158 width=36)
                    Merge Cond: (mm.platform_id = p.platform_id)
                    ->  Sort  (cost=783.13..783.92 rows=316 width=36)
                          Sort Key: mm.platform_id
                          ->  Hash Join  (cost=547.25..770.01 rows=316 width=36)
                                Hash Cond: (mm.measure_id = m.measure_id)
                                ->  Append  (cost=492.79..702.13 rows=5056 width=36)
                                      ->  Bitmap Heap Scan on metric_rollup_work_month mm  (cost=492.79..663.99 rows=5040 width=36)
                                            Recheck Cond: ((work_id = ANY ('{w1,w2,w3,w4}'::uuid[])) AND (month_start < '2026-04-01'::date))
                                            ->  Bitmap Index Scan on metric_rollup_work_month_identity_key  (cost=0.00..491.53 rows=5040 width=0)
                                                  Index Cond: ((work_id = ANY ('{w1,w2,w3,w4}'::uuid[])) AND (month_start < '2026-04-01'::date))
                                      ->  Index Scan using metric_rollup_work_month_ambiguity_identity_key on metric_rollup_work_month_ambiguity a  (cost=0.14..12.86 rows=16 width=36)
                                            Index Cond: ((work_id = ANY ('{w1,w2,w3,w4}'::uuid[])) AND (month_start < '2026-04-01'::date))
                                            Filter: total_ambiguous
                                ->  Hash  (cost=54.15..54.15 rows=25 width=16)
                                      ->  Index Scan using metric_measure_pkey on metric_measure m  (cost=0.15..54.15 rows=25 width=16)
                                            Filter: (enabled AND public_visibility AND additive_across_time AND additive_across_works)
                    ->  Index Scan using metric_platform_pkey on metric_platform p  (cost=0.15..55.65 rows=250 width=16)
                          Filter: enabled
```

Both branches reach their table through its identity key with
`work_id = ANY(...)` (and the month bound) as the index condition. The
production plan is evidenced by the staging reference (spec section 11), not
by this local plan.

### Other required checks

```text
git diff --check bc6b45bd HEAD                                   exit 0, no output
git diff bc6b45bd HEAD -- thoth-api/src/model/metric_dashboard/ \
  thoth-api/migrations                                          empty
git diff --stat bc6b45bd HEAD
  CHANGELOG.md                                 |    1 +
  docs/metrics/contract-register.md            |  228 ++--
  thoth-api/src/graphql/metric_widget_tests.rs |  317 +++++-
  thoth-api/src/graphql/query.rs               |    6 +-
  thoth-api/src/model/metric_widget/mod.rs     |  363 ++++++-
  thoth-api/src/model/metric_widget/tests.rs   | 1480 +++++++++++++++++++++++++-
  6 files changed, 2274 insertions(+), 121 deletions(-)
  (before the report commit, which adds only this file)
git merge-tree --write-tree HEAD origin/develop (6af22824)       exit 0, no conflict
```

## 10. Manual verification

Environment: none beyond the local test database.
Steps: none; the behaviour is covered by the database-backed tests and the
`EXPLAIN` above.
Observed result: not applicable.
Evidence link/screenshot/log reference: not applicable.

## 11. CI

CI status: recorded on the pull request at the exact head.
Checks: the repository's PR workflow.
Failures or warnings: recorded on the pull request.

## 12. Rollout and rollback

Initial state after merge: the switch `THOTH_METRIC_WIDGET` stays `OFF`;
nothing is served.
Activation required: yes, under R11 section 11's gate, extended by spec
section 11 (a staging performance reference of the all-history `YEAR` read,
a census of overlapping native-grain books, and a response cache keyed on
the rollup frontier), recorded in the register section 4.4.
Feature flag/configuration: unchanged.
Migration sequence: none.
Rollback/disable procedure: the switch, or a revert of the two commits.
Monitoring required: the outcome line's `grain`, `history` and `buckets`
fields, and the existing error and deadline lines.

## 13. Known limitations and deferred work

- Native-grain history is not served; a book whose native and day-grain
  history overlap is unavailable for the all-history read (spec section
  4.3, A1).
- Cost grows with the years served; the 5 s statement timeout and 10 s
  deadline bound it, and the staging reference is an activation gate (spec
  sections 4.5 and 11).
- The `EXPLAIN` is from local PostgreSQL 16 on small fixture data.

## 14. Unresolved issues

- NONE

## 15. Agent self-assessment

The agent may identify risks but may not approve the task.

Suggested review focus:

- step 4's order for both grains and both start forms, and the reuse of
  R11's dates message for an omitted start;
- step 11a: one served start for both sections, the floor check before the
  labels statement, and the empty-response start;
- the equivalence oracle at `YEAR` grain over the three step-7 families;
- that `METRIC_WIDGET_PAIRS_SQL` and `metric_dashboard/` are byte
  unchanged.
