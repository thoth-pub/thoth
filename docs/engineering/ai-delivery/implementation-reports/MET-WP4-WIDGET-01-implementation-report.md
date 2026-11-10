# MET-WP4-WIDGET-01 Implementation Report

## 1. Repository state

Owning GitHub issue: thoth-pub/thoth#1007 (programme: Thoth Metrics, #766)
Repository: thoth-pub/thoth
Workflow: STANDARD
Base branch: `develop`
Authorized base commit: `cab25eba415386f3dced1571c9f6468e1fa536f1`
Actual base commit: `cab25eba415386f3dced1571c9f6468e1fa536f1`
PR target: `develop`
Programme integration branch: none
Task branch: `feature/metrics-widget/met-wp4-widget-01`
Head commit: the commit that adds this report (see the draft PR); the
implementation commits are listed in section 3
Pull request: DRAFT, opened by the orchestrator (not by the implementing agent)
Expected branch deletion after merge: YES
Final programme PR required: NO
Implementing model: Claude Opus 5.5
Reasoning level: high

## 2. Scope confirmation

Approved specification: the #1007 body, R10 (sections 1-11), authorized by
`ADR-0014` and the implementation grant on #1007 (comment 6777125645, CTO,
2026-11-10), with the grant's budget additions and corrections.

Implemented objective: a public, read-only `metricWidget` query for the
embeddable `metrics-widget` package, gated by `METRICS_WIDGET` and shipped
inactive behind `THOTH_METRIC_WIDGET` (default `OFF`). The engine in
`metric_dashboard/mod.rs` gains only the section 4.10 Rust-level parameters.

Out-of-scope changes made: NONE

## 3. Commits

- `c4f3ae36b4d420b7ac6efe9f23347345ca55adc2` - Add pair mode and a read deadline to the Metrics engine
- `81004348e112fa287862e0946fb9d95f3ef12b81` - Add the public metricWidget read
- `09273a3cb37193c9d4543f3ce24659c39b351f40` - Add the THOTH_METRIC_WIDGET switch, off by default
- `493aa2fdc8977d5bd791448809b5b78132524ca0` - Record the public metricWidget contract
- the head commit - Add the MET-WP4-WIDGET-01 implementation report

The gates (fmt, clippy, workspace tests) were run on the final tree, which
the head commit carries.

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
- `CHANGELOG.md` (one `[Unreleased]` `### Added` line)
- `docs/metrics/contract-register.md` (new section 4.3 and three corrections)
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
  - reason: section 4.10.
  - behavioural effect: none for the dashboard. The 36 message constants
    become `pub(crate)` (text unchanged; rustfmt rewraps some declarations).
    New `pub(crate)` `MetricReadDeadline` (instant, refusal flag,
    last-statement record), `MetricPairRead` with constructor functions
    (`new`, `with_timeline_grain`, `with_countries`, `with_institutions`,
    `with_capability`) and the pair-mode entry `metric_pair_read`. The body
    that runs from `PUBLISHERS_SQL` on is extracted from `evaluate` into
    `read_from_publishers`, defined immediately after `evaluate` and before
    `fn resolve_works(`; `evaluate` runs the prelude and calls it with the
    dashboard capability, no pair scope and no deadline. In pair mode the
    caller's works and publisher replace `WORKS_SQL`, the served pairs replace
    `KNOWN_SQL`/`REPRESENTED_SQL` and the combination bound, the grid holds
    exactly the pairs, and country rows are kept only for held pairs
    (`Grid::holds`). The module doc's "Deliberately absent" line points to
    `metric_widget`. `db.get()` occurs once, `diesel::sql_query(` 15 times,
    `fn evaluate(` once, and the `evaluate(connection, &request,
    statement_timeout_sql)` closure is unchanged.
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

PASS. `git diff --stat cab25eba` lists only budget paths (section 9).

## 4.2 Authorized actions actually used

- repository inspection: yes
- source edit: yes
- new file creation: yes
- file deletion/move/rename: no
- branch creation: yes (`feature/metrics-widget/met-wp4-widget-01`, in a
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

Automatic CI/provider effects observed: none yet; PR CI runs when the draft PR
is opened and on each push.

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
   `INTERNAL_ERROR` and ends with a wildcard arm; the deadline flag alone
   decides `METRIC_WIDGET_BUSY`.
6. The log decision `log_decision(&WidgetFacts) -> Vec<WidgetLogLine>` is a
   pure value; lines are written with the `log` crate already in use. No line
   field can hold a package value.
7. The slot limiter is one process-wide `static AtomicUsize` with a drop
   guard; checkout uses `PgPool::get_timeout(1 s)`.

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
and `TEST_REDIS_URL=redis://localhost:6379` (local), as CI sets them.

### Formatting

```text
cargo fmt --all -- --check
```

Result: exit 0, no output.

### Lint/static analysis

```text
cargo clippy --all --all-targets --all-features -- -D warnings
```

Result: exit 0 (`Finished`); only the pre-existing future-incompatibility note
for `proc-macro-error2`.

### Full workspace

```text
cargo test --workspace
```

Result:

```text
thoth (lib)                       ok. 0 passed
thoth (bin src/bin/thoth.rs)      ok. 40 passed; 0 failed
thoth_api (lib)                   ok. 2218 passed; 0 failed; 3 ignored
thoth_api tests/graphql_permissions ok. 13 passed; 0 failed
thoth_api_server (lib)            ok. 5 passed; 0 failed
thoth_client (lib)                ok. 4 passed; 0 failed
thoth_errors (lib)                ok. 15 passed; 0 failed
thoth_export_server (lib)         ok. 144 passed; 0 failed
doc-tests                         thoth_client 6 passed; thoth_export_server 2 passed;
                                  thoth_api 8 ignored; others 0
exit status 0
```

### Focused widget and regression tests

```text
cargo test --workspace --lib -p thoth-api -- model::metric_widget::
  ok. 49 passed; 0 failed
cargo test --workspace --lib -p thoth-api -- graphql::metric_widget_tests::
  ok. 9 passed; 0 failed
cargo test --workspace --lib -p thoth-api -- model::metric_dashboard::
  ok. 67 passed; 0 failed; 1 ignored (the existing #[ignore] evidence test);
  metric_dashboard/tests.rs unmodified
cargo test --workspace --lib -p thoth-api -- graphql::metric_dashboard_tests::
  ok. 20 passed; 0 failed
cargo test --workspace --lib -p thoth-api -- graphql::metric_registry_tests::
  ok. 12 passed; 0 failed
cargo test --workspace --bin thoth -- metric_widget_control
  ok. 9 passed; 0 failed
cargo test -p thoth-api-server
  ok. 5 passed; 0 failed (including a_widget_refusal_is_http_200_with_an_errors_entry
  and an_invalid_widget_date_is_http_400)
```

### Other required checks

- `git diff --check`: no output.
- `git diff --stat cab25eba`: only budget paths.
- Statement ceiling: the longest path's captured sequence is exactly
  `CHECKOUT_CHECK, BEGIN, CUSTOM_PLANS_SQL,
  METRIC_WIDGET_STATEMENT_TIMEOUT_SQL, PROJECTION_LOCK_SQL,
  METRIC_WIDGET_RESOLUTION_SQL, METRIC_WIDGET_PAIRS_SQL,
  METRIC_WIDGET_PAIRS_SQL, METRIC_WIDGET_LABELS_SQL`, then twice
  `PUBLISHERS_SQL, ACCOUNTS_SQL, CANONICAL_SQL, MONTHS_SQL, ASSERTIONS_SQL,
  IDENTIFIER_QUALITY_SQL`, then `COMMIT`: 19 named statements, 22 captured
  (`the_longest_path_runs_exactly_the_bounded_statement_sequence`).
- Profile values: 2 slots per process, 1 s checkout timeout,
  `SET LOCAL statement_timeout = '5s'`, 10 s deadline from taking the slot,
  12 months, 500 child works, 500 `chapterDois`, 40 pairs per section
  (`the_profile_values_are_the_specified_bounds`).
- The three changed existing assertions, and the only changes to existing
  tests:
  1. `graphql/metric_dashboard_tests.rs`,
     `the_sdl_declares_exactly_the_approved_read_contract`: the Metrics read
     field count is 4, including `metricWidget` (was 3).
  2. `graphql/metric_dashboard_tests.rs`,
     `deferred_dashboard_surface_is_absent`: `metricWidget` and
     `MetricWidget` leave the absent list; the remaining single entry is
     asserted directly, since a one-element loop fails
     `clippy::single_element_loop`.
  3. `graphql/metric_registry_tests.rs`,
     `the_sdl_exposes_exactly_the_nine_approved_operations`: a
     `MET-WP4-WIDGET-01` group `["metricWidget"]` excluded beside
     `wp4_02_query_fields`, an "appears exactly once" assertion, and the
     comment updated.
  `metric_dashboard/tests.rs` is unmodified.
- Coverage-run identities (four budget files are manifest entries; the
  manifests are unchanged, the content identities change as expected):

| | Before (`cab25eba`) | After |
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

CI status: PENDING (runs when the orchestrator opens the draft PR).
Checks: build-test-and-check.
Failures or warnings: none known.

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

## 13. Known limitations and deferred work

- At most 12 whole months per read, no all-time or by-year view until
  `MET-WP4-WIDGET-ALLTIME`.
- Native-grain history is not served; a native-grain record on any served
  platform x served measure makes the read unavailable (deliberate
  over-refusal).
- The engine's per-section cross product (up to 40 x 40) and its
  `ASSERTIONS_SQL` row volume are bounded only by the pair bound; the staging
  reference records them.
- Response caching is `MET-READ-CACHE-01`.

## 14. Unresolved issues

- NONE

## 15. Agent self-assessment

The agent may identify risks but may not approve the task.

Suggested review focus: the `evaluate` extraction and pair mode in
`metric_dashboard/mod.rs` (dashboard semantics unchanged; the unmodified
`metric_dashboard/tests.rs` passes), the deadline admission points and the
error mapping, the step 3 claim counter under fragments, and the
equivalence oracle test.
