# MET-WP1-01 Implementation Report

Programme: Thoth Metrics - canonical ingestion, Sphinx orchestration and client cutover
Owning GitHub issue: [#836](https://github.com/thoth-pub/thoth/issues/836)
Parent programme issue: [#766](https://github.com/thoth-pub/thoth/issues/766)
Repository: `thoth-pub/thoth`
Task ID: MET-WP1-01 — Establish Metrics registry schema and Rust domain foundation
Risk: HIGH
Workflow: PROGRAMME_INTEGRATION

Authority condition: this report records what was implemented and measured. It
makes no approval decision. Live review, merge-authorization and merge evidence
is the GitHub pull-request and owning-issue record.

## 1. Repository state

| Field | Value |
|---|---|
| Owning GitHub issue | [#836](https://github.com/thoth-pub/thoth/issues/836) |
| Repository | `thoth-pub/thoth` |
| Workflow | PROGRAMME_INTEGRATION |
| Programme integration branch | `feature/metrics` |
| Base branch / PR target | `feature/metrics` |
| Authorized base commit | `a6c8cb2016179db635c4bc86ef366aae190829c2` (rebound base per amendment `5428313640`) |
| Actual base commit | `a6c8cb2016179db635c4bc86ef366aae190829c2` — reverified live immediately before resume; `feature/metrics` and the task branch were both at this exact SHA |
| Task branch | `feature/metrics--wp1-registry-foundation` (ADR-0009 sibling spelling; created from the exact authorized base under authorization `5428682009`, action 1) |
| Head commit | the exact head recorded on the draft PR after the final push; it is the SHA the fresh independent exact-head review must be taken against and is deliberately not transcribed here |
| Pull request | one DRAFT PR from `feature/metrics--wp1-registry-foundation` to `feature/metrics`, opened after all local gates passed; recorded in section 11 |
| Expected branch deletion after merge | YES |
| Final programme PR required | YES — the later coordinated `feature/metrics -> develop` integration is a separate gate and is not part of this task |
| Implementing model | Claude Fable 5 |
| Reasoning level | HIGH |

### 1.1 Authorization chain

The complete approved specification is the #836 issue body plus authoritative
amendments, applied in order:

1. issue #836 body (bounded WP1 first-slice specification);
2. technical amendment `5424713024` (naming, `supported_grains` contract, index
   decision, exact seed definitions, CG-08 semantics; controls on conflict);
3. CTO specification approval `5424932751`;
4. recovery amendment `5428313640` (rebound base
   `a6c8cb2016179db635c4bc86ef366aae190829c2`, replacement task branch
   `feature/metrics--wp1-registry-foundation` after ADR-0009);
5. independent recovery review / control-provenance reconciliation `5428558210`;
6. fresh bounded implementation authorization `5428682009`;
7. implementation-HOLD specification amendment `5429983905` (adds exactly
   `thoth-api/src/model/distribution_job/tests.rs` to the modification budget
   for the smallest test-only BE-04 correction);
8. independent amendment review reconciliation `5430132446` (APPROVED);
9. fresh implementation-resume authorization `5430172038`.

### 1.2 Implementation-HOLD and resume

Under authorization `5428682009` the implementation reached
`HOLD - REQUIRED FILE OUTSIDE WRITE BUDGET` during mandatory pre-commit
validation: `cargo test -p thoth-api --features backend` failed only on the
pre-existing BE-04 test
`model::distribution_job::tests::the_migration_directory_sorts_after_every_existing_one`,
whose `names.last()` assertion (BE-04 must remain the newest migration forever)
cannot coexist with any later migration, including the approved
`20260826_v1.8.0`. The implementing agent stopped before commit, push, PR
creation, tracker/CHANGELOG/report completion and any CI/GHCR side effect.

Amendment `5429983905` (independently reviewed APPROVED, `5430132446`) added
exactly that one existing file to the modification budget and bounded the
permitted change; resume authorization `5430172038` authorized continuing from
the existing uncommitted local work. Per that authorization, the prior
`1262/1263` run is HOLD evidence only: **every validation result in this
report is from a fresh post-correction run.**

## 2. Scope confirmation

Approved specification: #836 + amendments as listed in section 1.1.

Implemented objective: the smallest stable canonical Metrics registry
foundation — the `metric_platform`, `metric_measure` and
`metric_platform_measure` PostgreSQL tables, their four fixed registry enums,
the matching manually maintained Diesel `schema.rs` contract (ADR-0003), Rust
persistence/domain types, focused database/model tests, and exactly the two
design-fixed seed measures `title_sessions` and `net_units` — additive and
inactive, plus the narrowly authorized test-only BE-04 compatibility
correction and the durable tracker/changelog consequences.

Out-of-scope changes made: NONE.

## 3. Commits

| SHA | Subject |
|---|---|
| `6aa378b678663ef3ab91aceac4af2adda3182f5d` | `MET-WP1-01: correct stale BE-04 newest-migration test invariant` |
| `35b997598b6152dc89c5ab179c2812dd3cdcc1f9` | `MET-WP1-01: establish Metrics registry schema and Rust domain foundation` |
| recorded on the PR | `MET-WP1-01: reconcile programme trackers, changelog and implementation report` |
| recorded on the PR | `MET-WP1-01: record PR and observed CI state in implementation report` |

A commit cannot transcribe its own SHA, so the two documentation commits are
identified by subject; the exact SHAs and the final head are on the draft PR,
and the final head is the review target.

## 4. Files changed

Authorized modification paths (authorization `5428682009` as amended by
`5429983905` / `5430172038`):

- `CHANGELOG.md`
- `thoth-api/src/schema.rs`
- `thoth-api/src/model/mod.rs`
- `thoth-api/src/model/distribution_job/tests.rs` (added by amendment `5429983905`)
- `docs/metrics/README.md`
- `docs/metrics/task-status.md`
- `docs/metrics/rollout-plan.md`
- `docs/engineering/README.md`
- `docs/engineering/repository-map/control-gaps.md`

Authorized new-file paths:

- `thoth-api/migrations/20260826_v1.8.0/up.sql`
- `thoth-api/migrations/20260826_v1.8.0/down.sql`
- `thoth-api/src/model/metric_platform/mod.rs`
- `thoth-api/src/model/metric_platform/tests.rs`
- `thoth-api/src/model/metric_measure/mod.rs`
- `thoth-api/src/model/metric_measure/tests.rs`
- `thoth-api/src/model/metric_platform_measure/mod.rs`
- `thoth-api/src/model/metric_platform_measure/tests.rs`
- `docs/engineering/ai-delivery/implementation-reports/MET-WP1-01-implementation-report.md`

Actual files changed:

- `thoth-api/src/model/distribution_job/tests.rs`
  - reason: the narrowly authorized BE-04 correction (amendment `5429983905`).
    The test `the_migration_directory_sorts_after_every_existing_one` is
    renamed `the_migration_directory_keeps_its_exact_historical_name`; the
    sort and `names.last()` logic — whose sole purpose was the obsolete
    "BE-04 is newest forever" assertion — is removed; the retained body still
    enumerates the migrations directory, locates the BE-04 entry by its
    version prefix and asserts the exact historical directory name
    `20260814_v1.7.0`. The corrected test does not mention MET-WP1-01 or
    `20260826_v1.8.0`.
  - behavioural effect: test-only; no Publisher Services schema, migration,
    runtime or dependency change. All other tests in the file are untouched.
  - within authorized write budget: YES (path added by `5429983905`).
- `thoth-api/src/schema.rs`
  - reason: manually maintained repository-authoritative Diesel contract
    (ADR-0003), updated atomically with the migration: four custom SQL types,
    three `table!` declarations (with
    `supported_grains -> Array<MetricReportingGrain>`), two `joinable!` lines
    and three `allow_tables_to_appear_in_same_query!` entries.
  - behavioural effect: compile-time schema contract only; no runtime consumer
    is wired to the new tables.
  - within authorized write budget: YES.
- `thoth-api/src/model/mod.rs`
  - reason: register the three new model modules (alphabetical order).
  - behavioural effect: module registration only.
  - within authorized write budget: YES.
- `CHANGELOG.md`
  - reason: bounded `MET-WP1-01` entry under `[Unreleased]/Added`.
  - within authorized write budget: YES.
- `docs/metrics/task-status.md`, `docs/metrics/README.md`,
  `docs/metrics/rollout-plan.md`, `docs/engineering/README.md`,
  `docs/engineering/repository-map/control-gaps.md`
  - reason: durable consequences only — `feature/metrics` exists and the
    integration-branch gate is satisfied; claims that no such branch or
    approved WP1 child specification exists are removed; WP1 recorded as
    `IN PROGRESS` (started, not complete) upon integration of this slice;
    the registry foundation recorded as present on `feature/metrics`;
    CG-08 reconciled to `RESOLVED for WP1 entry` (2026-08-26) in the existing
    dated-resolution style with the four-entry-component reason fixed by
    amendment `5424713024` B5. CG-03, CG-04, CG-09, CG-10, CG-11 and CG-13
    are untouched; later Sphinx/client/source/WP5 gates stay attached to
    their owning work packages. No transient review/merge IDs are copied into
    active trackers.
  - within authorized write budget: YES.

Actual new files created (all within the authorized new-file list):

- `thoth-api/migrations/20260826_v1.8.0/up.sql` — YES
- `thoth-api/migrations/20260826_v1.8.0/down.sql` — YES
- `thoth-api/src/model/metric_platform/mod.rs` — YES
- `thoth-api/src/model/metric_platform/tests.rs` — YES
- `thoth-api/src/model/metric_measure/mod.rs` — YES
- `thoth-api/src/model/metric_measure/tests.rs` — YES
- `thoth-api/src/model/metric_platform_measure/mod.rs` — YES
- `thoth-api/src/model/metric_platform_measure/tests.rs` — YES
- `docs/engineering/ai-delivery/implementation-reports/MET-WP1-01-implementation-report.md` — YES (this file)

Files deleted, moved or renamed: NONE.

### 4.1 Write-budget compliance

PASS. `git status --porcelain` immediately before commit listed exactly the
nine modified budget paths and the nine authorized new files, and nothing
else. The full audit output is reproduced in section 9.

## 4.2 Authorized actions actually used

- repository inspection: USED (read-only; live GitHub state reverified before
  resume: both branches at `a6c8cb2016179db635c4bc86ef366aae190829c2`, no
  MET-WP1-01 PR, migration path still `20260826_v1.8.0` under the unchanged
  `make migration` convention, versions/dependencies/source-inventory
  unchanged).
- source edit: USED (within the amended write budget only).
- new file creation: USED (authorized list only).
- file deletion/move/rename: NOT USED.
- branch creation: NOT USED in the resume session — the task branch already
  existed at the exact authorized base from authorization `5428682009`;
  resume authorization `5430172038` authorized no further branch creation and
  none occurred.
- commit: USED (bounded commits listed in section 3).
- push: USED (task branch only).
- PR creation/update: USED (one DRAFT PR to `feature/metrics`).
- issue/comment mutation: NOT USED.
- manual CI dispatch/rerun: NOT USED.
- provider/runtime read: NOT USED.
- provider/runtime write: NOT USED.
- migration execution: local disposable-database validation only (PostgreSQL
  17.10 on localhost; empty and representative populated databases created
  and destroyed for this validation; no staging/provider/production database
  touched).
- release/tag/publication: NOT USED.
- merge: NOT USED.
- deployment: NOT USED.
- production activation: NOT USED.
- other: scratchpad-only evidence tooling outside the repository (a
  throwaway binary that drives the repository's embedded
  `thoth_api::db::MIGRATIONS` through `diesel_migrations::MigrationHarness`
  for timed apply / latest-only-revert cycles). It adds no repository file
  and changes no dependency.

Unauthorized actions performed: NONE.

## 4.3 Automatic and manual external effects

Expected automatic PR-triggered effects (verified against the current
workflow set and `.github/scripts/classify_ci_changes.py` before PR
creation): the changed set contains `.rs` files (`run_build=true`),
`up.sql`/`down.sql` (`run_migrations=true`) and is not documentation-only
(`run_docker=true`), so the PR triggers `build-test-and-check`,
`run-migrations` (disposable-PostgreSQL apply/revert/reapply),
`check-changelog` (CHANGELOG.md is updated), and
`publish-to-dockerhub`, which logs into `ghcr.io` and pushes the staging
image `ghcr.io/thoth-pub/thoth:staging-pr-<PR>`. That staging-image push is
the one authorized external write (authorizations `5428682009` item 8 and
`5430172038`).

Observed: all four workflows triggered automatically on draft PR
[#839](https://github.com/thoth-pub/thoth/pull/839) for each pushed head;
the staging image `ghcr.io/thoth-pub/thoth:staging-pr-839` was pushed
automatically on each run set (first-run digest
`sha256:e96651e82f3acc2281c31d5ea6542d815b85d417ba65d950346359214a2807de`).
Complete per-check results — including the first run set's `lint` failure on
pre-existing out-of-budget code under the newer runner toolchain, and the
runner-image root cause — are in section 11.

Manually initiated external actions: NONE (no manual workflow dispatch or
rerun).

External writes/publication beyond the automatic staging image: NONE.

## 5. Implementation decisions

Decisions made within the approved design:

1. Enum duplicate-grain rejection uses explicit
   `cardinality(array_positions(supported_grains, '<LABEL>')) <= 1` checks for
   each of the three closed enum values, per amendment `5424713024` B2 — no
   extension, helper function or dependency.
2. `updated_at` maintenance on `metric_platform` and `metric_measure` uses the
   repository-standard `diesel_manage_updated_at(...)` trigger, matching
   existing tables.
3. Blank-text rejection uses the repository-consistent
   `CHECK (col ~ '[^[:space:]]')` pattern for `code`, `display_name` and
   (on `metric_measure`) `definition`; no case/regex convention is invented
   for platform codes (deferred per advisory A6).
4. Seed insertion is deliberately unconditional (no `ON CONFLICT DO NOTHING`)
   so drift fails the migration loudly.
5. Rust enums are closed (no `Other`/`Unknown`/`Default`), derive
   `diesel_derive_enum::DbEnum` against the manually declared
   `ExistingTypePath` SQL types, and are deliberately **not**
   `juniper::GraphQLEnum`s — no GraphQL surface exists in this slice.
6. The shared registry test helper (`setup_registry_db`) restores the pristine
   post-migration registry state per test through the embedded harness's
   latest-only revert + reapply, asserting first that the latest applied
   migration is `20260826`; it returns a fresh dedicated pool because the
   revert/reapply cycle recreates the enum types with new OIDs, which a
   long-lived pooled connection's type cache would otherwise trip over.
7. The corrected BE-04 test keeps the prefix-locate + exact-name assertion
   (`20260814_v1.7.0`) — the directory name is the migration's version
   identity in `__diesel_schema_migrations` on every migrated database — and
   drops only the unsustainable "is newest" sort/`names.last()` logic.

Deviations from the specification requiring authorization: NONE.

## 6. Database and migration effects

Migration added: YES.

- migration files: `thoth-api/migrations/20260826_v1.8.0/up.sql`,
  `thoth-api/migrations/20260826_v1.8.0/down.sql` (path re-derived from the
  live Makefile convention — root version `1.7.0`, date 2026-08-26 — and
  reverified immediately before resume).
- schema effect: creates enum types `metric_platform_ownership_class`
  (`THOTH_MANAGED`, `PUBLISHER_CONTROLLED`, `EXTERNAL`),
  `metric_measure_category` (`USAGE`, `SALES`), `metric_measure_unit`
  (`COUNT`), `metric_reporting_grain` (`DAY`, `MONTH`, `REPORTING_PERIOD`);
  tables `metric_platform`, `metric_measure`, `metric_platform_measure` with
  the approved columns, design-fixed short identifiers, unique codes,
  non-blank checks, non-cascading FKs, the `(platform_id, measure_id)` unique
  pair and the non-empty / no-NULL / no-duplicate `supported_grains`
  constraint. Index set is exactly the six constraint-derived indexes (three
  primary keys, `metric_platform_code_key`, `metric_measure_code_key`,
  `metric_platform_measure_platform_id_measure_id_key`); the reviewed
  decision is that **no** additional secondary index is created in this
  slice (verified live: `pg_indexes` lists exactly those six).
- existing-data effect: none — the migration creates only new objects and the
  two `metric_measure` seed rows; the populated-database procedure below
  proves byte-identical preservation of representative existing data,
  including unchanged `relfilenode`s (no table rewrite).
- locking/downtime: the apply completed in 37.8 ms on the populated database
  **while a concurrent transaction held `AccessShareLock` on `publisher` and
  `work`** — the migration neither waited on that reader nor blocked it (the
  reader's post-migration reads inside the same open transaction succeeded
  normally), demonstrating that only catalog/new-object locks are taken and
  no `ACCESS EXCLUSIVE` lock is acquired on populated tables. A `pg_locks`
  sample immediately after the apply showed only the reader's granted
  `AccessShareLock`s on existing tables.
- empty database result: PASS (section 6.1).
- populated database result: PASS (section 6.2).
- rollback/forward repair: `down.sql` drops the three tables in
  dependency-safe order then the four enum types, removing only objects and
  data introduced by this migration. Latest-migration-only rollback and
  reapplication are proven both in-suite (every registry test performs a
  harness `revert_last_migration` + `run_pending_migrations` cycle via
  `setup_registry_db`, and
  `latest_only_rollback_removes_the_registry_and_reapplication_restores_it`
  asserts the reverted and restored states explicitly) and procedurally
  (sections 6.1–6.2). Per authorization `5428682009`/`5430172038`, the CLI
  `cargo run migrate --revert` was **not** used as rollback evidence: it
  dispatches to `revert_all_migrations`; all latest-only evidence uses the
  embedded Diesel migration harness (`revert_last_migration` on
  `thoth_api::db::MIGRATIONS`). Once later WP1 migrations depend on these
  tables, programme rollback must use dependency-aware reverse order or an
  approved forward-repair plan rather than reverting this migration in
  isolation.
- idempotency: the migration ledger prevents reapplication (`ledger max =
  20260826` after apply; a second `run_pending_migrations` is a no-op), and
  the apply → latest-only revert → reapply cycle succeeds repeatedly on
  disposable databases. DDL does not silently skip unexpected existing
  objects and the seed insert fails loudly on conflict.

### 6.1 Empty-database procedure (disposable PostgreSQL 17.10, UTF8)

Fresh `thoth_wp1_empty` database (`ENCODING 'UTF8'`, C locale), embedded
harness (`thoth_api::db::MIGRATIONS` + `MigrationHarness`):

1. `run_pending_migrations`: applied all 10 migrations
   `[20250000 … 20260814, 20260826]` in **207.6 ms**.
2. Verified: the four enum types with exactly the approved labels in order;
   the three registry tables; exactly 2 `metric_measure` rows; 0
   `metric_platform` rows; 0 `metric_platform_measure` rows; exactly the six
   constraint-derived indexes; the full expected constraint set (3 PK,
   3 UNIQUE beyond PKs counted with them, 6 CHECK, 2 FK as listed in
   section 6). Seeded attribute values verified per column, and both
   `definition` strings verified **byte-identical** to the approved amendment
   `5424713024` B4 text by MD5
   (`title_sessions`: `e51e14fc263ac177372d97f00ba6fcfa`, 260 chars;
   `net_units`: `1c93195395d3dbb8238707cb075f64f0`, 176 chars);
   `title_sessions.methodology_version = 'cloudfront-title-session/2'`,
   `net_units.methodology_version = NULL`.
3. `revert_last_migration`: reverted version `20260826` in **7.2 ms**.
   Verified: 0 `metric_%` tables, 0 `metric_%` enum types, ledger max
   `20260814`, pre-existing tables (`publisher`, `work`, `publication`,
   `institution`) still present.
4. `run_pending_migrations`: reapplied exactly `[20260826]` in **9.8 ms**.
   Verified: ledger max `20260826`, both seeds restored, still 0 platform and
   0 mapping rows.

### 6.2 Populated-database procedure (disposable PostgreSQL 17.10, UTF8)

Fresh `thoth_wp1_populated` database brought to the exact pre-MET-WP1-01
current schema (full chain applied, then harness `revert_last_migration`;
ledger max `20260814`), then seeded with representative current-schema data:
2 publishers, 2 imprints, 6 works, 4 publications, 2 institutions, and
Publisher Services rows — a linked enabled OAPEN/DOAB
`publisher_distribution_platform` pair sharing one activation, a disabled
ZENODO row, and one `publisher_service_configuration_history` audit row.

Preservation evidence uses a repeatable snapshot: per-table row counts,
per-table MD5 over every row's identifying content (ordered), and the
`relfilenode` of each populated table.

1. Pre-apply snapshot captured.
2. `run_pending_migrations` applied exactly `[20260826]` in **37.8 ms**,
   concurrently with an open reader transaction as described in section 6
   (locking/downtime).
3. Post-apply: registry present, seeds = 2, platforms = 0, mappings = 0,
   ledger max `20260826`; snapshot **identical** to pre-apply (all counts,
   all MD5s, all `relfilenode`s — no rewrite of any populated table).
4. `revert_last_migration` reverted `20260826` in **19.9 ms**; registry
   tables/types = 0, ledger max `20260814`; snapshot **identical** to
   pre-apply.
5. `run_pending_migrations` reapplied `[20260826]` in **21.1 ms**; seeds = 2,
   platforms = 0, mappings = 0, ledger max `20260826`; snapshot **identical**
   to pre-apply.

No production or provider database was read or written; both evidence
databases are disposable local databases created for this validation.

## 7. API and compatibility effects

GraphQL/API changes: NONE. No GraphQL type, query, mutation, input, output or
resolver is added or changed; the new enums are deliberately not
`juniper::GraphQLEnum`s and no model is registered with any GraphQL object.

Generated schema/client updates: NONE required. The generated SDL
(`thoth-client/assets/schema.graphql`, build-generated by
`thoth-client/build.rs` from `thoth_api::graphql::create_schema()`) was
compared between the exact base
`a6c8cb2016179db635c4bc86ef366aae190829c2` (built in a detached worktree) and
this branch: **byte-identical**, SHA-256
`521fba3b438c0013f21bfcbff62a24a3349cdd394738a40fd62e8f76fbf14226`
(177,616 bytes) on both sides.

Backwards compatibility: existing builds, tests, the public GraphQL schema,
export behaviour and existing database data are unchanged apart from the
additive presence of the new, unused registry schema and its two seed
measures after migration.

Deprecations: NONE.

Cross-repository dependencies: NONE. Known consumers consume GraphQL/export
contracts, which are unchanged; `thoth-sphinx` has no current implementation
and must not consume unmerged registry contracts. No downstream repository
task is required for this slice.

## 8. Authorization and security

Authorization paths changed: NONE (no new protected operation; no
resolver/policy change).
Roles/scopes involved: NONE.
Negative authorization tests: not applicable to this slice; the existing
authorization regression suite runs unchanged in the backend/workspace test
runs recorded below.
Secret or personal-data handling: none — no credentials, secrets, production
data, source logs, IP addresses or user agents were read or written.
Security limitations: no registry write path is exposed; future
registry-administration slices must add their own authorization and audit
decisions (including the deliberately deferred timestamp question on
`metric_platform_measure`).

## 9. Tests and checks

All commands were run fresh in this resume session, after the BE-04 test
correction, from the repository root with a local PostgreSQL 17.10 and Redis
available. The prior pre-HOLD `1262/1263` run was not reused as evidence.

### Formatting

Command:

```text
cargo fmt --all -- --check
```

Result:

```text
exit 0 (no diff)
```

### Focused MET-WP1-01 and corrected BE-04 tests

Commands:

```text
cargo test -p thoth-api --features backend --lib model::metric
cargo test -p thoth-api --features backend --lib \
  the_migration_directory_keeps_its_exact_historical_name -- --exact \
  model::distribution_job::tests::the_migration_directory_keeps_its_exact_historical_name
```

Results (final source):

```text
test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 1229 filtered out
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1263 filtered out
```

The 35 focused tests cover: exact enum label sets for all four registry
enums; exactly-two-measure seeding with per-column and byte-exact definition
assertions for both seeds; zero platform / zero mapping seeding; duplicate
platform code, duplicate measure code and duplicate `(platform_id,
measure_id)` rejection; blank `code`/`display_name`/`definition` rejection;
empty / NULL-element / duplicate `supported_grains` rejection; invalid FK
rejection; non-cascading delete behaviour for both referenced registries;
`updated_at` trigger behaviour; string/serde round-trips rejecting unknown
values; PostgreSQL round-trips for every enum value; Diesel row mapping for
all three tables; multi-grain `Vec<MetricReportingGrain>` order-preserving
round-trip; the deliberate absence of timestamp columns on
`metric_platform_measure`; and latest-only rollback/reapplication.

### Unit/integration — thoth-api backend

Command:

```text
cargo test -p thoth-api --features backend
```

Result:

```text
lib:            test result: ok. 1264 passed; 0 failed; 0 ignored (216.52s)
integration:    test result: ok. 13 passed; 0 failed; 0 ignored
doc-tests:      test result: ok. 0 passed; 0 failed; 8 ignored (pre-existing)
exit 0
```

### Workspace tests

Command:

```text
cargo test --workspace
```

Result:

```text
14 test binaries/doc-test sets across the workspace (final source):
1478 passed; 0 failed; 8 doc-tests ignored (pre-existing thoth-api set).
thoth lib 0 + bin 31; thoth-api lib 1264 + graphql_permissions 13;
four further crate lib sets 3 + 4 + 11 + 144; doc-tests
thoth_client 6 + thoth_export_server 2 (others 0).
exit 0
```

### Type check

Command:

```text
cargo check --workspace
```

Result:

```text
exit 0 (final-source run; the workspace was already fully built by the
preceding test/clippy gates, so the final check finished incrementally).
The only note is the pre-existing dependency future-incompat report for
proc-macro-error2 v2.0.1, also present on the base and unrelated to
this change.
```

### Lint/static analysis

Command:

```text
cargo clippy --all --all-targets --all-features -- -D warnings
```

Result:

```text
exit 0; zero lint warnings (the only note is the pre-existing dependency
future-incompat report for proc-macro-error2 v2.0.1, also present on the
base and unrelated to this change)
```

An earlier run of this command during the fresh validation surfaced two
`-D warnings` findings in the new focused tests
(`clippy::cmp_owned` and an unused binding, both in
`thoth-api/src/model/metric_platform/tests.rs`); they were fixed
in-budget and the complete backend, workspace, check, clippy and fmt
gates were then re-run from scratch on the final source. All results in
this report are from those final runs.

### Whitespace

Command:

```text
git diff --check
```

Result:

```text
exit 0 (clean)
```

### Write-budget audit

Command:

```text
git status --porcelain
```

Result (immediately before commit — exactly the amended budget, nothing
else):

```text
 M CHANGELOG.md
 M docs/engineering/README.md
 M docs/engineering/repository-map/control-gaps.md
 M docs/metrics/README.md
 M docs/metrics/rollout-plan.md
 M docs/metrics/task-status.md
 M thoth-api/src/model/distribution_job/tests.rs
 M thoth-api/src/model/mod.rs
 M thoth-api/src/schema.rs
?? docs/engineering/ai-delivery/implementation-reports/MET-WP1-01-implementation-report.md
?? thoth-api/migrations/20260826_v1.8.0/
?? thoth-api/src/model/metric_measure/
?? thoth-api/src/model/metric_platform/
?? thoth-api/src/model/metric_platform_measure/
```

### Classifier

Command:

```text
python3 .github/scripts/classify_ci_changes.py --paths <the 18 changed paths>
```

Result:

```text
{"docs_only": "false", "run_build": "true", "run_docker": "true", "run_migrations": "true"}
```

This matches the authorized expectation: substantive build/test/lint/format,
disposable-PostgreSQL migration validation, `check-changelog`, and the
automatic GHCR staging-image publication all run on the PR.

## 10. Manual verification

Environment: macOS host; disposable Homebrew PostgreSQL 17.10 (UTF8, C
locale) on localhost; local Redis for the workspace suites; no Docker, no
provider or production access.

Steps and observed results: the empty-database and populated-database
procedures in sections 6.1 and 6.2 (embedded-harness apply / latest-only
revert / reapply with timing, concurrent-reader locking observation,
registry/seed verification, and byte-identical preservation snapshots).

Evidence: command outputs summarized in sections 6.1, 6.2 and 9; migration
timings and lock observations as recorded there.

## 11. CI

Draft PR: [#839](https://github.com/thoth-pub/thoth/pull/839), from
`feature/metrics--wp1-registry-foundation` to `feature/metrics`, opened as
DRAFT after all local gates passed. Two automatic run sets have been
observed; **no manual dispatch or rerun was used at any point**.

Run set 1 — head `9e6a0cb754e798f3340102f112387ef16a5cea27`:

- `check-changelog`, `run-migrations` (disposable-PostgreSQL
  apply/revert/reapply), `classify`, `build`, `test`, `format_check`:
  success;
- `publish-to-dockerhub` / `build_and_push_staging_docker_image`: success —
  the authorized automatic external write occurred: image
  `ghcr.io/thoth-pub/thoth:staging-pr-839`, digest
  `sha256:e96651e82f3acc2281c31d5ea6542d815b85d417ba65d950346359214a2807de`,
  built from GitHub's synthetic PR merge revision
  `4400bdec1ffa5bb340917b56b2007099b9bea185` (the `pull_request` event's
  merge of the head into the unchanged `feature/metrics` base);
- `build-test-and-check` / `lint`: **failure** — exactly three
  `clippy::useless_format` errors, all in **`thoth-api/src/model/tests.rs`**
  (lines 887, 893, 899: the pre-existing `test_doi_with_domain`,
  `test_orcid_with_domain` and `test_ror_with_domain` assertions of the form
  `assert_eq!(format!("{}", …with_domain()), …)`).

Run set 2 — head after the PR/CI-state report commit: **all checks success**,
including a full ~6.5-minute `lint` job, and a fresh automatic staging-image
push for the same PR tag.

Root cause of the differing `lint` outcomes, established from the job logs:
GitHub's hosted `ubuntu-24.04` runner image is mid-rollout. The failing lint
job ran on image release `ubuntu24/20260823.283`, whose Rust stable is
**1.98.0**; clippy 1.98.0 extended `useless_format` coverage and flags the
three pre-existing assertions. The passing lint job ran on the older image
release `ubuntu24/20260816.277` (Rust stable 1.97.x), which — like the local
`rustc`/`clippy 1.97.0` toolchain used for the required local gates — does
not emit that lint. Which image a job receives is not controlled by the
repository, so **the `lint` job is currently nondeterministic for any
substantive Rust change** and will fail whenever it lands on a 1.98.0
runner, until the three sites are corrected.

The affected file is **byte-identical to the exact implementation base**
`a6c8cb2016179db635c4bc86ef366aae190829c2` (`git diff base..head` for the
path is empty; last modified 2026-08-14 by the BE-04 evidence commit). This
branch does not touch it, and it is **outside the amended MET-WP1-01 write
budget**, so this task may not correct it. The intervening merges (#835,
#838) were documentation/control changes for which the classifier skips the
lint job, so this PR is the first substantive Rust change to meet a 1.98.0
runner.

CI status at report time: **PASSING at the current head**, with the
nondeterminism above on record. Merge remains gated as always by fresh
independent exact-head source review and explicit CTO merge authorization;
no CI waiver is claimed or granted, and the pre-existing lint debt is
escalated in section 14.

## 12. Rollout and rollback

Initial state after merge to `feature/metrics`: the registry foundation
exists only on the programme integration branch; no production migration has
run; no registry API or admin mutation exists; no platform rows or mappings
exist; no collection/import/serving/export is active; WP1 is `IN PROGRESS`,
not complete.

Activation required: none in this slice; no runtime code path consumes the
registry, so no feature flag is needed.

Migration sequence: `20260826_v1.8.0` is the newest migration and applies
after the complete existing chain via the embedded runner.

Rollback/disable procedure: before later dependent WP1 migrations merge,
source rollback is the PR revert plus the tested latest-only `down.sql`
under separate migration authorization; after downstream migrations depend on
this foundation, use dependency-aware reverse order or a separately reviewed
forward-repair plan. Production rollback is out of scope because no
production migration is authorized.

Monitoring required: none introduced; production migration remains governed
by CG-13 and later WP11/release authorization.

## 13. Known limitations and deferred work

- Platform-code case/canonicalization and case-sensitivity semantics, and
  `public_description` empty/whitespace normalization, are deliberately
  deferred to the first platform-seeding or protected
  registry-administration child specification (amendment advisories A6/A7).
- `metric_platform_measure` deliberately has no timestamps; future protected
  registry-administration/audit work must decide mutation-history
  requirements before exposing a write path (advisory A3).
- Registry `methodology_version` is the measure's declared baseline
  methodology and does not replace per-batch/per-observation provenance,
  which later ingestion slices must record (advisory A1).
- The remaining WP1 slices (sources, accounts, records, etc.) are not
  specified or authorized; later slices must consume these registry rows
  rather than duplicate platform/measure definitions.

## 14. Unresolved issues

- Pre-existing lint debt outside this task's write budget:
  `thoth-api/src/model/tests.rs` lines 887/893/899 trip
  `clippy::useless_format` under Rust stable 1.98.0, which GitHub's
  `ubuntu-24.04` runner image is currently rolling out
  (`ubuntu24/20260823.283`; the older `ubuntu24/20260816.277` still ships
  1.97.x). Until those three sites are corrected, the repository's `lint`
  job is nondeterministically red/green for **every** substantive Rust
  change depending on which runner image a job lands on. The current PR
  head's automatic CI is fully green (section 11), but the debt is real and
  repository-wide rather than Metrics-specific; it needs either a further
  #836 write-budget amendment (the `5429983905` control class) or — more
  appropriately, since it is unrelated to Metrics — a small separate
  shared-control task, before or alongside the MET-WP1-01 merge decision.
  No out-of-budget edit, manual CI action or merge action was taken by this
  task.

## 15. Agent self-assessment

The agent may identify risks but may not approve the task.

Suggested review focus:

- exact conformance of `up.sql` to the approved column/constraint/seed
  contract, including the `supported_grains` duplicate-rejection checks and
  the byte-exact seed definitions;
- `schema.rs` ↔ migration ↔ Rust model atomic consistency (ADR-0003),
  especially `Array<MetricReportingGrain>` / `Vec<MetricReportingGrain>`;
- the corrected BE-04 test: confirm it still fails if the BE-04 migration
  directory were renamed or deleted, contains no MET-WP1-01 reference, and
  changes nothing else in the file;
- tracker/CG-08 reconciliation wording against amendment `5424713024` B5
  (specification approval vs merge-time WP1 `IN PROGRESS` distinction);
- the populated-database preservation and locking evidence in section 6.2.
