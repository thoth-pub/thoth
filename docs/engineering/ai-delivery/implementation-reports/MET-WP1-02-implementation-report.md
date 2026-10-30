# MET-WP1-02 Implementation Report

Programme: Thoth Metrics - canonical ingestion, Sphinx orchestration and client cutover
Owning GitHub issue: [#841](https://github.com/thoth-pub/thoth/issues/841)
Parent programme issue: [#766](https://github.com/thoth-pub/thoth/issues/766)
Repository: `thoth-pub/thoth`
Task ID: MET-WP1-02 — Establish Metrics source, source-account and checkpoint foundation
Risk: HIGH
Workflow: PROGRAMME_INTEGRATION

Authority condition: this report records what was implemented and measured. It
makes no approval decision. Live review, merge-authorization and merge evidence
is the GitHub pull-request and owning-issue record.

## 1. Repository state

| Field | Value |
|---|---|
| Owning GitHub issue | [#841](https://github.com/thoth-pub/thoth/issues/841) |
| Repository | `thoth-pub/thoth` |
| Workflow | PROGRAMME_INTEGRATION |
| Programme integration branch | `feature/metrics` |
| Base branch / PR target | `feature/metrics` |
| Authorized base commit | `377de62dd5a8b326288c9bf98e8b0e32d8f2d925` |
| Actual base commit | `377de62dd5a8b326288c9bf98e8b0e32d8f2d925` — reverified live immediately before branch creation; the task branch was created from this exact SHA |
| Task branch | `feature/metrics--wp1-source-state` (ADR-0009 sibling spelling; namespace preflight re-run against live remote refs immediately before creation: exact ref absent, no descendant occupancy, identifiers valid) |
| Head commit | the exact head recorded on the draft PR after the final push; it is the SHA the fresh independent exact-head review must be taken against and is deliberately not transcribed here |
| Pull request | one DRAFT PR from `feature/metrics--wp1-source-state` to `feature/metrics`, opened after all local gates passed; recorded in section 11 |
| Expected branch deletion after merge | YES |
| Final programme PR required | YES — the later coordinated `feature/metrics -> develop` integration is a separate gate and is not part of this task |
| Implementing model | Claude Fable 5 |
| Reasoning level | HIGH |

### 1.1 Authorization chain

The complete approved specification is:

1. issue #841 body (bounded WP1 source-state-foundation specification);
2. authoritative specification amendment `5439908702` (recorded namespace
   preflight; fixed automatic PR side-effect inventory; controls on conflict);
3. independent specification review `5440094787` — APPROVED;
4. CTO specification approval `5440148690` — APPROVED;
5. bounded implementation authorization `5440262769` (controlling
   action-authorization record).

Immediately before implementation, live GitHub was reverified: #841 remained
OPEN with exactly those four lifecycle comments and an unchanged body
(`updated_at` equal to the authorization timestamp), `feature/metrics`
remained exactly the authorized base SHA, no `MET-WP1-02` branch or PR
existed, and the current PR workflows/classifier remained materially identical
to the inventory accepted in amendment `5439908702` (reverified again
immediately before draft-PR creation).

### 1.2 Design-revision verification

The repository-authoritative record at the exact base
(`docs/metrics/README.md`, `docs/metrics/task-status.md`) records the approved
Metrics design as Drive revision `6`, and the same revision is bound by the
same-day CTO specification approval and implementation authorization. A direct
Google Drive metadata read to re-confirm the live revision number was blocked
by the local environment's tool-permission layer; no authoritative source
available to this implementation indicates any revision change. Recorded as a
limitation in section 13, not as a deviation.

## 2. Scope confirmation

Approved specification: #841 + amendment `5439908702` (section 1.1).

Implemented objective: the additive, inactive Metrics source-state foundation
— migration `20260827_v1.8.0` creating the closed
`metric_source_acquisition_type` enum and the `metric_source`,
`metric_source_account` and `metric_source_checkpoint` tables; the matching
manually maintained `schema.rs` contract; the three Rust model modules with
focused database/model tests; and the bounded durable consequences
(CHANGELOG entry, Metrics tracker update, this report).

Out-of-scope changes made: NONE.

## 3. Commits

- the bounded implementation commit(s) on `feature/metrics--wp1-source-state`
  recorded on the draft PR; the exact SHAs are visible on the PR and are not
  transcribed here because the final head is review-bound evidence.

## 4. Files changed

Authorized write paths (from authorization `5440262769`):

- `CHANGELOG.md`
- `docs/metrics/task-status.md`
- `thoth-api/src/schema.rs`
- `thoth-api/src/model/mod.rs`

Authorized new-file paths:

- `thoth-api/migrations/20260827_v1.8.0/up.sql`
- `thoth-api/migrations/20260827_v1.8.0/down.sql`
- `thoth-api/src/model/metric_source/mod.rs`
- `thoth-api/src/model/metric_source/tests.rs`
- `thoth-api/src/model/metric_source_account/mod.rs`
- `thoth-api/src/model/metric_source_account/tests.rs`
- `thoth-api/src/model/metric_source_checkpoint/mod.rs`
- `thoth-api/src/model/metric_source_checkpoint/tests.rs`
- `docs/engineering/ai-delivery/implementation-reports/MET-WP1-02-implementation-report.md`

Actual files changed:

- `CHANGELOG.md`
  - reason: required bounded `Unreleased` entry.
  - behavioural effect: none (documentation).
  - within authorized write budget: YES
- `docs/metrics/task-status.md`
  - reason: record `MET-WP1-02` as the merged second WP1 slice after
    integration; keep WP1 `IN PROGRESS`; update the header and the two
    narrative passages whose "only MET-WP1-01 exists / no further child
    specification exists" claims would otherwise become materially false when
    this slice merges. No transient PR/review/CI identifier was copied in.
  - behavioural effect: none (control documentation).
  - within authorized write budget: YES
- `thoth-api/src/schema.rs`
  - reason: ADR-0003 atomic schema-contract update: one new custom SQL type
    (`MetricSourceAcquisitionType`), three new `table!` blocks in the
    existing alphabetical position, four new `joinable!` lines, three new
    `allow_tables_to_appear_in_same_query!` entries. No unrelated
    reformatting; existing custom types, aliases and ordering preserved.
  - behavioural effect: compile-time Diesel contract for the new tables.
  - within authorized write budget: YES
- `thoth-api/src/model/mod.rs`
  - reason: register the three new model modules (alphabetical position).
  - behavioural effect: modules compiled into the crate.
  - within authorized write budget: YES

Actual new files created:

- `thoth-api/migrations/20260827_v1.8.0/up.sql` — within authorized list: YES
- `thoth-api/migrations/20260827_v1.8.0/down.sql` — within authorized list: YES
- `thoth-api/src/model/metric_source/mod.rs` — within authorized list: YES
- `thoth-api/src/model/metric_source/tests.rs` — within authorized list: YES
- `thoth-api/src/model/metric_source_account/mod.rs` — within authorized list: YES
- `thoth-api/src/model/metric_source_account/tests.rs` — within authorized list: YES
- `thoth-api/src/model/metric_source_checkpoint/mod.rs` — within authorized list: YES
- `thoth-api/src/model/metric_source_checkpoint/tests.rs` — within authorized list: YES
- `docs/engineering/ai-delivery/implementation-reports/MET-WP1-02-implementation-report.md` — within authorized list: YES

Files deleted, moved or renamed: NONE.

### 4.1 Write-budget compliance

PASS — `git status --short` at commit time lists exactly the four authorized
modified files and the nine authorized new files, and nothing else.

## 4.2 Authorized actions actually used

- repository inspection: USED (repository files, live refs, issue #841 and its
  four lifecycle comments, workflows/classifier).
- source edit: USED (the four authorized existing files only).
- new file creation: USED (the nine authorized paths only; the migration
  directory was created through the repository convention `make migration`,
  which produced exactly `thoth-api/migrations/20260827_v1.8.0/`).
- file deletion/move/rename: NOT USED.
- branch creation: USED (`feature/metrics--wp1-source-state` from the exact
  authorized base, after immediate base + namespace recheck).
- commit: USED (bounded implementation commits on the task branch only).
- push: USED (only `feature/metrics--wp1-source-state`).
- PR creation/update: USED (one DRAFT PR to `feature/metrics`; section 11).
- issue/comment mutation: NOT USED.
- manual CI dispatch/rerun: NOT USED.
- provider/runtime read: NOT USED.
- provider/runtime write: NOT USED.
- migration execution: USED — local disposable databases only (`thoth`,
  `thoth_test`, `thoth_wp102_populated` on local PostgreSQL 17); no staging or
  production database was accessed.
- release/tag/publication: NOT USED (the automatic staging-PR GHCR push is a
  workflow side effect of the authorized PR action; section 4.3).
- merge: NOT PERFORMED.
- deployment: NOT PERFORMED.
- production activation: NOT PERFORMED.
- other: a read-only `git worktree` of the exact base commit was created in
  the session scratchpad (outside the repository working tree) to build the
  base binary and baseline GraphQL SDL for the required byte-identical
  comparison; it modified no repository branch or file.

Unauthorized actions performed: NONE.

## 4.3 Automatic and manual external effects

Automatic CI/provider effects observed: opening the draft PR triggered the
four expected `pull_request` workflows (`build-test-and-check`,
`run-migrations`, `check-changelog`, `publish-to-dockerhub`); their observed
state is in section 11. `publish-to-dockerhub` performs the single authorized
automatic external write: a push of `ghcr.io/thoth-pub/thoth:staging-pr-<PR>`
after login with the GitHub Actions token. The live workflow files and
classifier were reverified immediately before PR creation and matched the
inventory accepted in amendment `5439908702` and authorization `5440262769`.

Manually initiated external actions: NONE.

External writes/publication: NONE beyond the automatic staging-PR image above.

## 5. Implementation decisions

Decisions within the approved design:

1. Constraint names follow the PostgreSQL/repository default naming used by
   MET-WP1-01 (`<table>_pkey`, `<table>_<cols>_key`, `<table>_<col>_check`,
   `<table>_<col>_fkey`), stated explicitly in the migration DDL.
2. Blank/whitespace-only rejection reuses the MET-WP1-01 idiom
   `CHECK (col ~ '[^[:space:]]')` for `code`, `external_key` and
   `partition_key`.
3. The non-negative day constraints are plain `CHECK (col >= 0)` on the two
   nullable integer columns; SQL NULL passes the check by three-valued logic,
   which is exactly the approved "NULL = unset" semantics.
4. `updated_at` on `metric_source_checkpoint` uses the repository-standard
   `DEFAULT CURRENT_TIMESTAMP NOT NULL` plus
   `SELECT diesel_manage_updated_at('public.metric_source_checkpoint')`,
   identical to every other trigger-maintained table.
5. The lease index is a plain btree
   `metric_source_checkpoint_lease_expires_at_idx (lease_expires_at)`; no
   partial-index predicate was invented because no query shape is approved
   yet.
6. The Rust acquisition enum mirrors the MET-WP1-01 registry enum pattern
   exactly (`diesel_derive_enum::DbEnum` with `ExistingTypePath`,
   SCREAMING_SNAKE_CASE serde/strum, no `Default`, no `GraphQLEnum`).
7. `configuration` and `cursor` map to `serde_json::Value` through the
   existing Diesel `Jsonb` mapping already used by the repository's history
   tables; `last_successful_period_end` maps to `chrono::NaiveDate` and the
   nullable timestamps to `Option<crate::model::Timestamp>`, all existing
   repository mappings. No dependency was added or changed.
8. The test modules reuse the shared MET-WP1-01 harness
   (`metric_platform::tests::setup_registry_db`), which already tolerates
   later migrations, and add a targeted
   `revert_through_source_state_migration` helper for this migration
   modelled on the durable revert-through pattern — no test assumes this
   migration remains the newest forever.
9. Models are plain `diesel::Queryable` structs with no CRUD/GraphQL
   machinery, matching the inactive-foundation scope.

Deviations from the specification requiring authorization: NONE.

## 6. Database and migration effects

Migration added: YES.

- migration files: `thoth-api/migrations/20260827_v1.8.0/up.sql`,
  `thoth-api/migrations/20260827_v1.8.0/down.sql`. Path produced by the
  repository convention (`make migration`; root version `1.7.0` → `1.8.0`,
  date 2026-08-27), matching the authorized path exactly; verified
  non-mutatingly from the Makefile before creation and then by running the
  convention itself.
- schema effect: creates enum `metric_source_acquisition_type` (`DRIVER`,
  `PUBLISHER_UPLOAD`, `OPERAS`, `ADMIN_IMPORT`) and tables `metric_source`,
  `metric_source_account`, `metric_source_checkpoint` with exactly the
  approved columns and nullability; unique `metric_source(code)`,
  `UNIQUE(source_id, external_key)`,
  `UNIQUE(source_account_id, partition_key)`; non-blank checks on the three
  identity keys; `>= 0` checks on the two optional day columns;
  non-cascading FKs account→source, account→`metric_platform`,
  account→`publisher` (optional), checkpoint→account;
  `configuration jsonb NOT NULL DEFAULT '{}'::jsonb`; nullable generic
  `cursor jsonb`; repository-standard `uuid_generate_v4()` PK defaults;
  trigger-maintained `updated_at` on checkpoints only. Index set is exactly
  the six constraint-derived indexes plus
  `metric_source_checkpoint_lease_expires_at_idx` (asserted from
  `pg_indexes` in-suite: 2 + 2 + 3 indexes per table, no others).
- existing-data effect: none — only new objects are created and **no row is
  seeded** in any table; the populated-database procedure below proves
  byte-identical preservation of representative existing data, including
  unchanged `relfilenode`s (no table rewrite).
- locking/downtime: on the populated database the apply ran **while a
  concurrent open transaction held `AccessShareLock` on `publisher` and
  `work`**; the migration neither waited on that reader nor blocked it (the
  reader's post-apply reads inside its still-open transaction succeeded
  normally), and a `pg_locks` sample immediately after the apply showed only
  the reader's granted `AccessShareLock`s on existing tables. The full binary
  invocation (`thoth migrate`, process start to exit) took **0.15 s wall**
  applying exactly `[20260827]`; the reverse DDL measured per statement in
  psql totals ~11 ms. Only new-object/catalog locks are taken; no lock is
  acquired on populated tables.
- empty database result: PASS (section 6.1).
- populated database result: PASS (section 6.2).
- rollback/forward repair: `down.sql` drops
  `metric_source_checkpoint` → `metric_source_account` → `metric_source` →
  the enum type, in dependency-safe order, removing only objects introduced
  by this migration and leaving the MET-WP1-01 registry schema and seed rows
  intact (proven in-suite and on the populated database). Targeted
  revert-through/reapply is proven with the embedded Diesel harness by
  `reverting_through_the_source_state_migration_removes_it_and_leaves_the_registry_intact`
  and by the shared `setup_registry_db` cycle every metrics test performs;
  the helper reverts down to and including `20260827`, tolerates later
  repository migrations, and never asserts `20260827` remains the newest.
  The CLI `cargo run migrate --revert` (revert-all) was additionally
  exercised on the empty-chain database as the CI-equivalent procedure, not
  as targeted-rollback evidence. Once later WP1/WP2 migrations depend on
  these tables, programme rollback must use dependency-aware reverse order
  or an approved forward-repair plan.
- idempotency: the migration ledger prevents reapplication (after apply,
  ledger max = `20260827`; a further `migrate` is a no-op), and the
  apply → targeted revert → reapply cycle succeeds repeatedly on disposable
  databases. DDL does not skip unexpected existing objects (`CREATE TYPE` /
  `CREATE TABLE` fail loudly on conflict), and there is no seed data to
  duplicate.

### 6.1 Empty-database procedure (disposable local PostgreSQL 17, UTF8)

Fresh `thoth` database, repository CLI (embedded migration runner):

1. `cargo run migrate`: applied the full chain
   `[20250000 … 20260826, 20260827]` cleanly to the empty database; ledger
   max `20260827`.
2. Verified from catalog metadata: the acquisition enum lists exactly
   `DRIVER, PUBLISHER_UPLOAD, OPERAS, ADMIN_IMPORT` in order; the three
   tables match the approved columns, types, nullability and defaults
   (`uuid_generate_v4()` PKs, `'{}'::jsonb`, `CURRENT_TIMESTAMP`);
   0 rows in all three new tables; `metric_measure` still holds exactly its
   2 seed rows.
3. `cargo run migrate --revert` (CI-equivalent revert-all): 0 `metric_%`
   tables and 0 `metric_%` types remained.
4. `cargo run migrate` reapplied the full chain: ledger max `20260827`,
   measure seeds = 2, `metric_source` rows = 0.

The exact-shape, constraint, FK, uniqueness, blank-rejection, negative-day,
UUID-default, trigger and index assertions are additionally enforced by the
28 in-suite tests listed in section 9, which run against a disposable UTF8
PostgreSQL 17 test database through the embedded harness.

### 6.2 Populated-database procedure (disposable local PostgreSQL 17, UTF8)

Fresh `thoth_wp102_populated` database (`ENCODING 'UTF8'`, C locale) brought
to the exact pre-MET-WP1-02 current schema by running the migration chain
with a binary built from the exact authorized base commit in a scratchpad
worktree (ledger max `20260826`), then populated with representative
current-schema data: 1 publisher, 1 imprint, 1 work, 1 publication,
1 institution, and MET-WP1-01 registry data — 1 locally inserted
`metric_platform` row and 1 `metric_platform_measure` row mapped to the
seeded `title_sessions` measure (fixture rows, not seeds).

Preservation evidence uses a repeatable snapshot: per-table row counts,
per-table MD5 over every row's `row_to_json` content (ordered), and the
`relfilenode` of each populated table
(`publisher`, `imprint`, `work`, `publication`, `institution`,
`metric_platform`, `metric_measure`, `metric_platform_measure`).

1. Pre-apply snapshot captured.
2. The branch binary applied exactly `[20260827]` (0.15 s wall for the whole
   process) concurrently with the open reader transaction described in
   section 6; the reader was neither blocked nor waited on.
3. Post-apply: ledger max `20260827`; 0 rows in each new table; snapshot
   **identical** to pre-apply (all counts, all MD5s, all `relfilenode`s — no
   rewrite of any populated table).
4. Targeted revert of `20260827` only (the `down.sql` statements plus the
   ledger-row removal, executed in a single transaction; ~11 ms of DDL):
   ledger max `20260826`; 0 source-state tables and 0 acquisition-enum
   types; `metric_platform` = 1, `metric_measure` = 2,
   `metric_platform_measure` = 1; snapshot **identical** to pre-apply.
5. The branch binary reapplied `[20260827]` (0.10 s wall): ledger max
   `20260827`, 0 rows in each new table, snapshot **identical** to
   pre-apply.

No production or provider database was read or written; all evidence
databases are disposable local databases created for this validation.

## 7. API and compatibility effects

GraphQL/API changes: NONE.

Generated schema/client updates: NONE required. The generated public GraphQL
SDL (`thoth-client/assets/schema.graphql`, build-generated and gitignored)
was compared between a clean workspace build of the exact authorized base
commit (scratchpad worktree) and a clean workspace build of the final source
state: **byte-identical**, SHA-256
`521fba3b438c0013f21bfcbff62a24a3349cdd394738a40fd62e8f76fbf14226` on both
sides (`cmp` exit 0). No generated-client file was written.

Backwards compatibility: fully additive database-only change; no existing
table, column, enum, API field or export format is touched.

Deprecations: NONE.

Cross-repository dependencies: NONE. Known consumers (`thoth-app`,
`thoth-client`, `thoth-dissemination`, `metrics-dashboard`, `metrics-widget`)
consume GraphQL/export contracts, which are unchanged. `thoth-sphinx` remains
a future consumer through separately specified protected GraphQL operations
and must not access these tables directly. No downstream task is required.

## 8. Authorization and security

Authorization paths changed: NONE (no GraphQL surface, no policy change, no
role added).

Roles/scopes involved: NONE.

Negative authorization tests: not applicable — no protected operation exists
for this schema; none was invented.

Secret or personal-data handling: `metric_source_account.configuration` is
documented at the SQL and Rust layers as generic **non-secret**
routing/configuration JSON; credentials must never be stored in it. Per the
approved specification, no heuristic SQL secret detection was created —
allowed-field validation belongs to the later protected write-path
specification. No secret, credential or personal source data was read,
stored or logged by this task.

Security limitations: none beyond the deferred write-path validation above.

## 9. Tests and checks

All results below are from fresh runs at the final source state on the local
disposable stack (PostgreSQL 17 UTF8, Redis 8), with
`THOTH_EXPORT_API=http://localhost:8181` and `TEST_DATABASE_URL` exported.

### Formatting

Command:

```text
cargo fmt --all -- --check
```

Result:

```text
exit 0, no diff
```

### Unit/integration — thoth-api backend

Command:

```text
cargo test -p thoth-api --features backend
```

Result:

```text
test result: ok. 1292 passed; 0 failed (lib) — includes the 28 new
MET-WP1-02 tests below; plus 13 passed (other targets); 8 doc-tests ignored
(pre-existing)
```

The 28 new focused tests (all passing):

```text
metric_source::tests — acquisition_type_enum_has_exactly_the_approved_labels,
  acquisition_type_string_conversion_round_trips_and_rejects_unknown_values,
  every_acquisition_type_round_trips_through_postgres,
  migration_seeds_no_source_row, duplicate_source_code_is_rejected,
  blank_source_code_is_rejected,
  negative_day_defaults_are_rejected_and_non_negative_values_accepted,
  source_deliberately_has_no_timestamp_columns,
  source_state_primary_keys_use_the_repository_standard_uuid_default,
  metric_source_rows_map_through_diesel,
  reverting_through_the_source_state_migration_removes_it_and_leaves_the_registry_intact
metric_source_account::tests — migration_seeds_no_source_account_row,
  source_account_deliberately_has_no_timestamp_columns,
  blank_external_key_is_rejected,
  account_identity_is_unique_per_source_and_external_key,
  account_foreign_keys_require_existing_rows,
  deleting_referenced_rows_is_restricted_rather_than_cascaded,
  configuration_is_non_null_jsonb_defaulting_to_an_empty_object,
  metric_source_account_rows_map_through_diesel
metric_source_checkpoint::tests — migration_seeds_no_checkpoint_row,
  checkpoint_deliberately_has_no_created_at_column,
  blank_partition_key_is_rejected,
  checkpoint_identity_is_unique_per_account_and_partition_key,
  checkpoint_foreign_key_requires_an_existing_account_and_restricts_deletion,
  metric_source_checkpoint_rows_map_through_diesel,
  checkpoint_updated_at_is_maintained_by_the_repository_standard_trigger,
  lease_expiry_has_the_required_operational_index,
  source_state_tables_have_no_speculative_secondary_index
```

No claim/lease/concurrency-protocol behaviour is implemented, and no test
pretends to validate it.

### Workspace tests

Command:

```text
cargo test --workspace
```

Result:

```text
all suites ok — 1506 tests passed, 0 failed (1292 thoth-api lib, 144, 31,
13, 11, 6, 4, 3, 2 across the other workspace targets), 8 doc-tests ignored
```

### Type check

Command:

```text
cargo check --workspace
```

Result:

```text
exit 0 (pre-existing upstream future-incompat note for proc-macro-error2,
unchanged from base)
```

### Lint/static analysis

Command:

```text
cargo clippy --all --all-targets --all-features -- -D warnings
```

Result:

```text
exit 0, no warnings
```

### Whitespace

Command:

```text
git diff --check
```

Result:

```text
exit 0, no output
```

### Write-budget audit

Command:

```text
git status --short
```

Result:

```text
exactly the 4 authorized modified files and 9 authorized new files;
no deletion, move or rename
```

## 10. Manual verification

Environment: local disposable PostgreSQL 17 (UTF8, C locale) via Homebrew,
local Redis 8; repository CLI binaries built from the final source state and
(in a scratchpad worktree) from the exact authorized base.

Steps and observed results: sections 6.1, 6.2 and 7 (empty-chain
apply/revert/reapply; populated-database apply/targeted-revert/reapply with
snapshot and `relfilenode` comparison and a concurrent reader; byte-identical
SDL comparison).

Evidence: command outputs summarized in this report; the in-suite tests
re-prove the schema assertions on every run.

## 11. CI

Draft PR: [#843](https://github.com/thoth-pub/thoth/pull/843)
(`feature/metrics--wp1-source-state` → `feature/metrics`, DRAFT). Opening it
automatically triggered exactly the four authorized workflows. Observed state
for the implementation head `a26e772d675bcab07fbcf8ab25bb2b7706453323`
(2026-08-27; this report-evidence commit itself only changes this file):

- `build-test-and-check`: classify PASS; `format_check` PASS (16s); `build`
  PASS (6m38s); `test` PASS (10m13s); **`lint` FAIL (5m8s)** — see below.
- `run-migrations`: classify PASS; `run_migrations` PASS (7m35s) — CI
  disposable PostgreSQL 17 build, `cargo run migrate`,
  `cargo run migrate --revert`, reapply.
- `check-changelog`: PASS (7s).
- `publish-to-dockerhub`: classify PASS (`run_docker=true`);
  `build_and_push_staging_docker_image` PASS (11m33s) — pushed
  `ghcr.io/thoth-pub/thoth:staging-pr-843`, image digest
  `sha256:bee0640c7622476eab7cdb9abc51e23c14650fe3305ddc2cb276f2a182caeab0`.
  This is the single authorized automatic external write side effect; it is
  not a release, deployment or activation.

### 11.1 The `lint` failure is a pre-existing runner-image condition

The failing job ran on the GitHub `ubuntu-24.04` image release
`20260823.283`, whose toolchain carries clippy for Rust 1.98. It rejects
three **pre-existing** `useless_format` sites at
`thoth-api/src/model/tests.rs:887/893/899` — a file this PR does not touch
(`git diff <base> HEAD -- thoth-api/src/model/tests.rs` is empty; the sites
are identical at the exact authorized base). Local clippy 1.97 and the CI
image generation still on the pre-rollout toolchain pass the same command;
the job outcome currently depends on which runner-image generation a run
lands on. This same condition was already observed and recorded on
2026-08-27 during MET-WP1-01 CI.

Consequences under the bounded authorization: the implementing agent may not
rerun CI and may not modify `thoth-api/src/model/tests.rs` (outside the write
budget). The failure is therefore reported as a finding for the independent
reviewer/CTO. Remediating it needs either a separately authorized bounded
correction of the three pre-existing sites, or a rerun/re-land once the
runner rollout is uniform — neither is performed here. No other check failed,
and the workspace clippy gate passes locally at the final source state.

## 12. Rollout and rollback

Initial state after merge: schema-only, inactive. No source, account or
checkpoint row exists; no code path reads or writes the new tables; no
GraphQL surface exposes them; no worker exists.

Activation required: later separately specified and authorized slices
(source-account administration write path, internal claim/checkpoint API,
WP2 ingestion, Sphinx orchestration) — none is authorized by this task.

Feature flag/configuration: none.

Migration sequence: `20260827_v1.8.0` after `20260826_v1.8.0`. Production
migration execution remains governed by CG-13 and separate operational
authorization; nothing was executed outside disposable local databases.

Rollback/disable procedure: repository rollback is a separately authorized
revert of the bounded integration. Database rollback is `down.sql`
(checkpoint → account → source → enum), validated on disposable databases
only; any future staging/production rollback requires separate operational
authorization and live-state verification.

Monitoring required: none until an activating slice exists.

## 13. Known limitations and deferred work

- The live Google Drive revision of the approved design could not be
  re-read directly (environment tool-permission block); verification rests
  on the repository-authoritative revision `6` record at the exact base and
  the same-day approval chain (section 1.2).
- `driver_key` semantics (driver registry, uniqueness, DRIVER-specific
  constraints) are deliberately deferred to the later driver specification.
- `configuration` allowed-field validation is deliberately deferred to the
  later protected source-account administration specification; the
  non-secret rule is currently documentation + review enforced.
- The lease/claim concurrency protocol (claim tokens, lease SQL,
  `FOR UPDATE SKIP LOCKED`, stale-lease recovery, retries) is deliberately
  absent and unproven; only the durable columns exist.
- Source/platform mappings remain unapproved; no row exists in any Metrics
  table beyond the two MET-WP1-01 measure seeds.
- WP1 remains `IN PROGRESS`; this is the second slice, not completion.

## 14. Unresolved issues

- NONE.

## 15. Agent self-assessment

The implementing agent may identify risks but may not approve the task.

Suggested review focus:

- exact field/constraint parity between `up.sql`, `schema.rs` and the Rust
  models (ADR-0003 atomicity);
- the tracker (`docs/metrics/task-status.md`) edits: confirm the two
  narrative updates stay within "record the durable consequence of this
  slice" and introduce no lifecycle transcription;
- the shared-harness reuse (`setup_registry_db`) and the new
  `revert_through_source_state_migration` helper for future-migration
  tolerance;
- confirmation that no lease/claim semantics leaked into SQL, models or
  tests;
- the populated-database and SDL comparison procedures in sections 6.2
  and 7.
