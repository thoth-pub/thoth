# MET-WP1-06 Implementation Report

## 1. Repository state

Owning GitHub issue: [#878](https://github.com/thoth-pub/thoth/issues/878)
Parent programme issue: [#766](https://github.com/thoth-pub/thoth/issues/766)
Repository: `thoth-pub/thoth`
Workflow: PROGRAMME_INTEGRATION
Base branch: `feature/metrics`
Authorized base commit: `eb51e0681e4e6406c22f0553396884768e19ba38`
Actual base commit: `eb51e0681e4e6406c22f0553396884768e19ba38` (live `feature/metrics` verified identical immediately before the first mutation of this continuation)
Observed `develop` checkpoint: `4546cb632428872b961ad6c17282984d298e3ade` (verified identical to the authorized incorporated checkpoint; no programme refresh required)
PR target: `feature/metrics`
Programme integration branch: `feature/metrics`
Task branch: `feature/metrics--wp1-publisher-approval`
Head commit: recorded in the post-push completion handoff for the exact pushed head; this report is committed inside that same bounded implementation commit and therefore cannot contain its own SHA
Pull request: not yet created at the time this report was written. The draft PR is opened after this commit is pushed, and GitHub remains the lifecycle authority for its number, state, review and CI
Expected branch deletion after merge: YES (not authorized here)
Final programme PR required: YES (`feature/metrics -> develop`, separately gated and not implied here)
Implementing model: Claude Opus 5

### 1.1 Recovery context

This slice was implemented as a **recovery continuation**, not a fresh start.

Local work-in-progress from an earlier, interrupted implementation attempt
was retained on the task branch. A read-only forensic review classified it:

```text
B - RECOVERABLE WITH BOUNDED CORRECTIONS
```

The CTO recovery-control record,
[#878 comment `5511593270`](https://github.com/thoth-pub/thoth/issues/878#issuecomment-5511593270),
rejected reset/discard, authorized retention of that WIP at the already-bound
migration path, and required three bounded corrections before commit. That
record supplements, and does not replace, the original implementation
authorization.

Controlling records read in full before any mutation:

- the complete #878 issue body (approved specification);
- fresh independent specification review [comment `5506339400`](https://github.com/thoth-pub/thoth/issues/878#issuecomment-5506339400) — APPROVED;
- CTO specification approval [comment `5506620387`](https://github.com/thoth-pub/thoth/issues/878#issuecomment-5506620387) — APPROVED;
- implementation authorization / handoff [comment `5507118350`](https://github.com/thoth-pub/thoth/issues/878#issuecomment-5507118350) — AUTHORIZED;
- recovery-control record [comment `5511593270`](https://github.com/thoth-pub/thoth/issues/878#issuecomment-5511593270);
- repository root `AGENTS.md` and `thoth-api/AGENTS.md`;
- ADR-0003 (repository-authoritative schema contract) and the merged
  MET-WP1-01/02/03/04/05 migration and model conventions on the authorized base.

### 1.2 Recovery preflight

Every premise was reverified before the first mutation:

| Check | Result |
|---|---|
| live `feature/metrics` | `eb51e0681e4e6406c22f0553396884768e19ba38` — unchanged |
| live `develop` | `4546cb632428872b961ad6c17282984d298e3ade` — unchanged |
| local HEAD | `eb51e0681e4e6406c22f0553396884768e19ba38` |
| current branch | `feature/metrics--wp1-publisher-approval` |
| commits beyond base | zero (`git log base..HEAD` empty) |
| upstream configured | none |
| remote task branch | absent (`git ls-remote` empty) |
| PR for this head, any state | none (`gh pr list --state all` → `[]`) |
| staged content | none |
| merge/rebase/cherry-pick in progress | none |
| paths modified/untracked outside the nine-path budget | none |
| comment `5511593270` | present, `created_at == updated_at` (unedited) |
| comment `5507118350` | present, `created_at == updated_at` (unedited) |

The retained WIP was compared byte-for-byte against the forensic snapshot.
All eight pre-correction SHA-256 values matched exactly:

```text
099a79606143c236f207ad14329ea7727f36d1d0d845a226a954da7482fe42b3  CHANGELOG.md
00b7f5d3fab2a6f9c1932c157ff8810ac9a4837d44b466036e54c2db68538528  docs/metrics/task-status.md
14d02c1401d2c1357bd95a6ee37f39c6880e738740773f2072922e296e8c8abd  thoth-api/src/schema.rs
bcf943234e065bba2bc1d2a6a366d89c3114e8c515e1b4a088e4de723204e813  thoth-api/src/model/mod.rs
971233b62e82588fc9a7f61e62e94facac16ec8f09e250044abc42db0b846f8b  thoth-api/migrations/20260902_v1.9.0/up.sql
c0463a48349e99ce67ea49f54abf1157bbaf29d3cc4eb6a2488ab67d456e9643  thoth-api/migrations/20260902_v1.9.0/down.sql
49ce2d055cfdf40a88eecd499738257bbf9ab3466a0b5a3fcbcfe4003425ffba  thoth-api/src/model/metric_publisher_platform_approval/mod.rs
be5365e31ba8d57f2d284306152a348f579f6f1a72dc032a11c22b5fcb378019  thoth-api/src/model/metric_publisher_platform_approval/tests.rs
```

The implementation report was absent, as the forensic snapshot recorded. No
`HOLD` condition was triggered.

### 1.3 Corrections applied to the retained WIP

Exactly the three corrections required by comment `5511593270`, and nothing else:

1. **Tracker truthfulness** — `docs/metrics/task-status.md`. The retained WIP
   asserted in three places that a DRAFT PR was already open. All three were
   removed: the `Last updated` header block, the `MET-WP1-06` slice-table
   Status column and description cell, and the WP1 dependencies cell. The
   active tracker now records no PR number, no DRAFT/OPEN PR state, no CI
   state, no review state and no merge state. It records only what the
   implementation commit itself makes true — that the approval foundation is
   implemented on its slice branch. WP1 remains `IN PROGRESS`.
2. **SQL convention** — `thoth-api/migrations/20260902_v1.9.0/up.sql`.
   `approved_at timestamptz` became `approved_at timestamp with time zone`.
   `timestamptz` appeared nowhere else in `thoth-api/migrations/`, while
   eight existing migrations use the spelled-out form. This is a spelling
   change only; the resulting PostgreSQL type is unchanged, and no other
   semantic expansion was made.
3. **Test coverage** — `thoth-api/src/model/metric_publisher_platform_approval/tests.rs`.
   Added a non-null `approved_at` round-trip assertion (folded into the
   complete-row Diesel round-trip, which previously omitted the column
   entirely), and a new test
   `usage_and_sales_submission_flags_are_independently_representable`
   exercising the sales-only combination
   (`usage_submission_enabled = false, sales_submission_enabled = true`).
   No existing test was weakened or removed.

The optional `MET_WP1_06_MIGRATION_VERSION` visibility change was **not**
made, and the existing 63-byte unique-constraint name was **not** renamed, as
comment `5511593270` directs.

## 2. Scope confirmation

Approved specification: the exact written #878 issue body, independently
reviewed (`5506339400`) and CTO-approved (`5506620387`).

Implemented objective: an additive, inactive, persistence-only publisher-platform
approval foundation — the closed `metric_publisher_platform_approval_status`
enum, the `metric_publisher_platform_approval` table, the manually maintained
`thoth-api/src/schema.rs` contract, the matching Rust domain module and focused
database/model tests.

Out-of-scope changes made: NONE.

## 3. Commits

One bounded implementation commit on `feature/metrics--wp1-publisher-approval`.
Its exact SHA and tree SHA are reported in the post-commit completion handoff;
this report is part of that commit's own tree.

## 4. Files changed

Authorized write paths (existing files):

- `CHANGELOG.md`
- `docs/metrics/task-status.md`
- `thoth-api/src/schema.rs`
- `thoth-api/src/model/mod.rs`

Authorized new-file paths:

- `thoth-api/migrations/20260902_v1.9.0/up.sql`
- `thoth-api/migrations/20260902_v1.9.0/down.sql`
- `thoth-api/src/model/metric_publisher_platform_approval/mod.rs`
- `thoth-api/src/model/metric_publisher_platform_approval/tests.rs`
- `docs/engineering/ai-delivery/implementation-reports/MET-WP1-06-implementation-report.md`

Actual files changed:

- `CHANGELOG.md`
  - reason: one bounded `## [Unreleased]` / `### Added` entry for MET-WP1-06.
  - behavioural effect: none; documentation only.
  - within authorized write budget: YES
- `docs/metrics/task-status.md`
  - reason: record the durable consequence of this slice in the active Metrics tracker.
  - behavioural effect: none; documentation only. WP1 stays `IN PROGRESS`; no PR/CI/review/merge state is pre-recorded.
  - within authorized write budget: YES
- `thoth-api/src/schema.rs`
  - reason: ADR-0003 requires the manually maintained Diesel contract to change atomically with the migration. Adds the `MetricPublisherPlatformApprovalStatus` custom SQL type, the `metric_publisher_platform_approval` table block, two `joinable!` declarations and one `allow_tables_to_appear_in_same_query!` entry.
  - behavioural effect: compile-time schema contract only; no runtime behaviour.
  - within authorized write budget: YES
- `thoth-api/src/model/mod.rs`
  - reason: register the new module (`pub mod metric_publisher_platform_approval;`), inserted in existing alphabetical order.
  - behavioural effect: none beyond module visibility.
  - within authorized write budget: YES

Actual new files created:

- `thoth-api/migrations/20260902_v1.9.0/up.sql` — within authorized new-file list: YES
- `thoth-api/migrations/20260902_v1.9.0/down.sql` — within authorized new-file list: YES
- `thoth-api/src/model/metric_publisher_platform_approval/mod.rs` — within authorized new-file list: YES
- `thoth-api/src/model/metric_publisher_platform_approval/tests.rs` — within authorized new-file list: YES
- `docs/engineering/ai-delivery/implementation-reports/MET-WP1-06-implementation-report.md` — within authorized new-file list: YES

Files deleted, moved or renamed: NONE.

### 4.1 Write-budget compliance

**PASS.** `git status --porcelain --untracked-files=all` immediately before
commit listed exactly the nine authorized paths and nothing else. No existing
migration directory was modified: `diff -r` against the base worktree
confirmed every pre-existing migration directory, including
`20260901_v1.9.0`, is byte-identical.

## 4.2 Authorized actions actually used

- repository inspection: YES
- source edit: YES (within the nine-path budget)
- new file creation: YES (the five authorized new paths)
- file deletion/move/rename: NO
- branch creation: NO — `feature/metrics--wp1-publisher-approval` already existed from the recovered attempt, created from the exact authorized base
- commit: YES (one bounded commit)
- push: YES (task branch only, normal push; no force, no amend, no rebase)
- PR creation/update: YES (exactly one DRAFT PR to `feature/metrics`)
- issue/comment mutation: NO
- manual CI dispatch/rerun/cancel: NO
- provider/runtime read: NO
- provider/runtime write: NO
- migration execution: YES — **disposable local PostgreSQL 17 only**. No staging or production database was accessed at any point.
- release/tag/publication: NO
- merge: NO
- deployment: NO
- production activation: NO
- other: one detached `git worktree` at the authorized base commit, created under the scratchpad directory purely to produce base-build GraphQL SDL and base-schema database evidence, and removed afterwards. It created no branch or ref and wrote nothing into the repository working tree.

Unauthorized actions performed: NONE.

## 4.3 Automatic and manual external effects

Automatic CI/provider effects expected on PR creation, as explicitly
authorized by comment `5507118350`: full Rust build/test/lint/format CI;
GitHub-hosted disposable PostgreSQL migration apply/revert/reapply; changelog
validation; and automatic publication of the PR-scoped staging container image
to `ghcr.io/thoth-pub/thoth` under a workflow-generated `staging-pr-*` tag.
That image publication is CI evidence only — not deployment, release, runtime
activation or production use.

Observed results are reported in the post-push completion handoff, not
pre-recorded here.

Manually initiated external actions: NONE.

External writes/publication by the implementing agent: NONE.

## 5. Implementation decisions

1. **Enum type name.** `metric_publisher_platform_approval_status`, matching
   the merged Metrics convention of `<table>_<attribute>` enum names. The
   approved design fixes the labels but not a PostgreSQL type identifier;
   the independent specification review verified this name.
2. **Primary-key default.** `uuid DEFAULT public.uuid_generate_v4() NOT NULL`,
   identical to the merged MET-WP1-01 through MET-WP1-05 Metrics tables.
3. **Non-cascading foreign keys.** Both FKs are declared with no `ON DELETE`
   clause, so PostgreSQL's default `NO ACTION` restricts deletion of a
   referenced publisher or platform rather than silently destroying approval
   evidence. Asserted by test.
4. **No `approved_by` foreign key.** Preserved as nullable `UUID` with no FK,
   no ZITADEL-string-to-UUID conversion, no user/account table and no
   value-generation rule, exactly as #878 §4.6 requires.
5. **No invented defaults.** `usage_submission_enabled`,
   `sales_submission_enabled` and `approval_status` are `NOT NULL` with no
   default; `approved_at` has no timestamp default.
6. **No deferred semantics encoded.** No trigger, stored procedure or CHECK
   constraint asserts any approval-transition, audit-field or
   `PUBLISHER_CONTROLLED` ownership-class rule. Those belong to later bounded
   runtime/administrative work.
7. **Index set.** Exactly the primary-key index and the index PostgreSQL
   creates for the `(publisher_id, platform_id)` UNIQUE constraint. No
   speculative secondary index. Asserted by test against `pg_indexes`.
8. **Not a GraphQL enum.** `MetricPublisherPlatformApprovalStatus` is
   deliberately not a `juniper::GraphQLEnum`; this slice exposes no API surface.
9. **Down-migration ordering.** The table is dropped before its enum type,
   because the table's `approval_status` column depends on the type.

Deviations from the specification requiring authorization: **NONE**, other
than the three explicitly approved recovery corrections recorded in §1.3.

## 6. Database and migration effects

Migration added: **YES**

- migration files:
  - `thoth-api/migrations/20260902_v1.9.0/up.sql`
  - `thoth-api/migrations/20260902_v1.9.0/down.sql`
  - Diesel migration version: `20260902` (Diesel derives the version from the
    text before the first underscore). The `v1.9.0` suffix matches the
    workspace package version `1.8.0` incremented per repository convention,
    and the directory was freshly confirmed unused on the authorized base.

- schema effect: creates one enum type and one table.

  ```text
  CREATE TYPE public.metric_publisher_platform_approval_status
      AS ENUM ('PENDING', 'APPROVED', 'REVOKED');

  public.metric_publisher_platform_approval
      publisher_platform_approval_id  uuid  NOT NULL  DEFAULT uuid_generate_v4()
      publisher_id                    uuid  NOT NULL
      platform_id                     uuid  NOT NULL
      usage_submission_enabled        boolean  NOT NULL
      sales_submission_enabled        boolean  NOT NULL
      approval_status                 metric_publisher_platform_approval_status  NOT NULL
      approved_by                     uuid  NULL
      approved_at                     timestamp with time zone  NULL
      notes                           text  NULL
      PRIMARY KEY (publisher_platform_approval_id)
      UNIQUE (publisher_id, platform_id)
      FOREIGN KEY (publisher_id) REFERENCES publisher(publisher_id)
      FOREIGN KEY (platform_id) REFERENCES metric_platform(platform_id)
  ```

  Verified live with `\d+` on a migrated disposable database: exactly nine
  columns with the nullability above, exactly two indexes (the PK index and
  the pair-uniqueness index), exactly two foreign keys with no `ON DELETE`
  clause, **zero** CHECK constraints and **zero** non-internal triggers.

- existing-data effect: **NONE**. No existing table is altered, backfilled or
  rewritten. `pg_class.relfilenode` was captured for all 72 pre-existing
  public tables before and after applying the migration to a representative
  populated database; every relfilenode was unchanged and the only difference
  was the addition of the new table. No approval row is seeded (`SELECT
  count(*)` = 0 after apply and after reapply).

- locking/downtime: creating the two foreign keys takes
  **`ShareRowExclusiveLock` on the referenced tables** `publisher` and
  `metric_platform` (measured directly from `pg_locks` while the migration DDL
  was open in a transaction, alongside `AccessShareLock` on each). That lock
  permits concurrent reads but **blocks concurrent writes** to `publisher` and
  `metric_platform` for the duration of the migration. This report therefore
  does **not** claim zero write blocking. Because the new table is empty at
  creation time, PostgreSQL has no rows to validate, so the FK creation itself
  is O(1) rather than proportional to the referenced tables' size; the whole
  pending-migration apply completed in **0.11 s real** on the representative
  populated database and **0.08 s real** on reapply. The practical blocking
  window is therefore short, but it is not nil, and on a large production
  instance it is still gated behind the normal lock queue: the migration must
  wait for in-flight write transactions on `publisher` and `metric_platform`
  to finish before it can acquire the lock. Production migration execution is
  not authorized by this task in any case.

  Full measured lock set inside the migration transaction:

  ```text
  metric_platform :: AccessShareLock
  metric_platform :: ShareRowExclusiveLock
  metric_publisher_platform_approval :: AccessExclusiveLock
  metric_publisher_platform_approval :: AccessShareLock
  metric_publisher_platform_approval :: ShareLock
  metric_publisher_platform_approval :: ShareRowExclusiveLock
  publisher :: AccessShareLock
  publisher :: ShareRowExclusiveLock
  ```

- empty database result: the complete 15-migration chain applied cleanly to a
  freshly `initdb`-created, empty PostgreSQL 17.10 database in 4.06 s. The
  ledger recorded all 15 identities; 73 public tables resulted. The full
  reverse chain (`thoth migrate --revert`, which the repository implements as
  `revert_all_migrations`) then reverted every migration cleanly, leaving one
  residual table (`__diesel_schema_migrations`) and zero `metric%` enum types.
  Reapplication restored all 15 migrations and 73 tables.

- populated database result: a second disposable database was migrated to the
  **authorized base schema only** (14 migrations, head `20260901`,
  MET-WP1-06 objects absent) using a binary built from the base worktree at
  `eb51e0681e4e6406c22f0553396884768e19ba38`, then populated with
  representative bibliographic state (publisher, imprint, work, publication,
  institution), Publisher Services state
  (`publisher_distribution_platform`), and MET-WP1-01 through MET-WP1-05 state
  (`metric_platform`, both seeded `metric_measure` rows, `metric_source`,
  `metric_source_account`, `metric_import`, `metric_record`,
  `metric_record_revision`, `metric_coverage`). Per-table `md5(string_agg(
  row_to_json(...)))` content hashes were taken before and after applying the
  pending MET-WP1-06 migration. **All 19 data-table hashes were byte-identical
  after the migration.** The only snapshot difference was the migration ledger
  gaining exactly one row (14 → 15); the 14 pre-existing ledger identities
  hashed identically (`faeb6848dc068e971130120adf971a70`) including their
  `run_on` timestamps.

- rollback/forward repair: the down migration drops the table, then the enum,
  and removes **only** MET-WP1-06 objects. Executed against the populated
  database, it reduced public tables 73 → 72 and public enum types 39 → 38,
  and the resulting data-plus-ledger snapshot was byte-identical to the
  pre-migration snapshot. Reapplication then reproduced the post-migration
  state exactly; the sole hash difference was the `run_on` timestamp of the
  re-inserted `20260902` ledger row, which is expected. A targeted
  revert-through/reapply at this dependency point is additionally asserted in
  Rust by
  `reverting_through_the_approval_migration_removes_it_and_reapplication_restores_it`,
  which reverts through the migration via the embedded harness, proves the
  MET-WP1-01..05 tables and the two seeded measures survive byte-identically,
  then reapplies.

- idempotency: the migration is not self-idempotent (`CREATE TYPE` /
  `CREATE TABLE` without `IF NOT EXISTS`), matching every merged Metrics
  migration; the Diesel ledger provides run-once semantics. The down
  migration uses `DROP ... IF EXISTS` and is safe to repeat.

## 7. API and compatibility effects

GraphQL/API changes: **NONE.** The module exposes no GraphQL object, input,
enum, query, mutation or resolver.

Generated schema/client updates: none required. The reachable public GraphQL
SDL was generated from both trees by a full `cargo build --workspace`
(`thoth-client/build.rs` writes `thoth-client/assets/schema.graphql`) and
proven byte-identical:

```text
base  eb51e0681e4e6406c22f0553396884768e19ba38 (detached worktree build)
  091e11f293132fdec784de420e3addf251f5020ba7e387889b292a066be15d8e  178270 bytes
implementation tree
  091e11f293132fdec784de420e3addf251f5020ba7e387889b292a066be15d8e  178270 bytes
cmp → exit 0 (identical)
```

Backwards compatibility: fully preserved; the change is purely additive
database schema plus a compile-time Diesel contract.

Deprecations: NONE.

Cross-repository dependencies: NONE. `thoth-app`, `thoth-sphinx`,
`metrics-dashboard`, `metrics-widget`, `thoth-client` and
`thoth-dissemination` require no source change, because no contract they
consume changed.

## 8. Authorization and security

Authorization paths changed: **NONE.** `thoth-api/src/policy.rs` is untouched.

Roles/scopes involved: none. This slice creates no protected operation, so
there is no anonymous / wrong-role / wrong-publisher-scope / correct-scope /
superuser matrix to exercise. The existing full `thoth-api` suite, including
`tests/graphql_permissions.rs` (13 tests), passes unchanged.

Negative authorization tests: not applicable to this slice. The tests that do
exist assert **schema** fail-closed behaviour only — invalid publisher FK,
invalid platform FK, restricted deletion of referenced rows, duplicate pair
rejection, and NOT NULL rejection for the usage, sales and status columns. No
test claims runtime authorization behaviour exists.

Secret or personal-data handling: NONE. Nothing is logged.

Security limitations, stated explicitly:

- A row in this table is **not** authorization. No code path reads it, and
  nothing in this slice can treat it as permission to submit data.
- The binding runtime invariant that a publisher upload cannot target a
  `THOTH_MANAGED` platform is **not** enforced in the database here. It must
  be enforced at the later publisher-import boundary.
- `approved_by` is **not** currently a complete authenticated-actor audit
  trail. It accepts NULL and any syntactically valid UUID, with no FK and no
  identity contract. Before any write path populates it, a separately
  reviewed specification must reconcile it with the repository-authoritative
  ZITADEL string-identity model.

## 9. Tests and checks

All commands were run fresh from the repository root against the final
corrected tree, with a disposable local PostgreSQL 17.10 and Redis.

### Formatting

Command:

```text
cargo fmt --all -- --check
```

Result:

```text
exit 0; no output (no reformatting required)
```

### Unit tests

Command:

```text
cargo test --workspace
```

Result:

```text
exit 0; 1644 passed; 0 failed; 8 ignored across all workspace test binaries
(includes thoth-api lib 1430 passed, thoth-api graphql_permissions 13 passed,
thoth-api-server 3 passed, thoth-client 31 passed, thoth-export-server 144 passed)
```

### Integration/database tests

Command:

```text
cargo test -p thoth-api --features backend
```

Result:

```text
exit 0; 1430 passed; 0 failed; 0 ignored in 169.33s (lib)
                13 passed; 0 failed (tests/graphql_permissions.rs)
                 0 passed; 8 ignored (remaining integration target)
```

Command (focused MET-WP1-06):

```text
cargo test -p thoth-api --features backend metric_publisher_platform_approval
```

Result:

```text
exit 0; 15 passed; 0 failed; 0 ignored; 1415 filtered out; finished in 6.31s

approval_status_enum_has_exactly_the_approved_labels ......................... ok
approval_status_string_conversion_round_trips_and_rejects_unknown_values ..... ok
every_approval_status_round_trips_through_postgres ........................... ok
approval_rows_map_through_diesel ............................................. ok
approval_database_defaults_are_applied_without_explicit_values ............... ok
usage_and_sales_submission_flags_are_independently_representable ............. ok
nullable_audit_and_notes_fields_round_trip ................................... ok
usage_sales_and_status_fields_are_non_null ................................... ok
invalid_approval_foreign_keys_fail_closed .................................... ok
deleting_a_referenced_publisher_or_platform_is_restricted .................... ok
duplicate_publisher_platform_pair_is_rejected ................................ ok
migration_seeds_no_approval_row .............................................. ok
metric_publisher_platform_approval_has_exactly_the_authorized_non_cascading_foreign_keys ... ok
metric_publisher_platform_approval_has_no_index_beyond_its_authorized_constraints .......... ok
reverting_through_the_approval_migration_removes_it_and_reapplication_restores_it .......... ok
```

Mapping to the #878 §9.3 acceptance cases:

| §9.3 case | Covering test |
|---|---|
| all three enum values round-trip | `every_approval_status_round_trips_through_postgres`, `approval_status_enum_has_exactly_the_approved_labels` |
| unknown enum values fail closed | `approval_status_string_conversion_round_trips_and_rejects_unknown_values` (`OTHER`, `ACTIVE`, lowercase `pending` all rejected) |
| complete row round-trip | `approval_rows_map_through_diesel` |
| usage-only combination | `approval_rows_map_through_diesel` (`usage=true, sales=false`) |
| sales-only combination | `usage_and_sales_submission_flags_are_independently_representable` (`usage=false, sales=true`) |
| non-null `approved_at` round-trip | `approval_rows_map_through_diesel` (`2026-09-02T11:45:00Z` reloaded unchanged) |
| null `approved_by` / `approved_at` / `notes` | `approval_database_defaults_are_applied_without_explicit_values`, `nullable_audit_and_notes_fields_round_trip` |
| arbitrary valid `approved_by` UUID with no FK | `nullable_audit_and_notes_fields_round_trip` |
| invalid publisher/platform FKs fail | `invalid_approval_foreign_keys_fail_closed` |
| referenced deletions restrict, not cascade | `deleting_a_referenced_publisher_or_platform_is_restricted` |
| duplicate pair fails | `duplicate_publisher_platform_pair_is_rejected` |
| null usage/sales/status fails | `usage_sales_and_status_fields_are_non_null` |
| exact index inventory | `metric_publisher_platform_approval_has_no_index_beyond_its_authorized_constraints`, `..._has_exactly_the_authorized_non_cascading_foreign_keys` |
| no seeded row | `migration_seeds_no_approval_row` |
| no false runtime-authorization claim | reviewed: every assertion is schema-level; the module header states the absence explicitly |

### Lint/static analysis

Command:

```text
cargo clippy --all --all-targets --all-features -- -D warnings
```

Result:

```text
exit 0; no errors, no lint warnings
(one unrelated pre-existing cargo note about proc-macro-error2 future incompatibility)
```

Command:

```text
cargo check --workspace
```

Result:

```text
exit 0; Finished `dev` profile in 1m 46s; no errors, no warnings
```

### Other required checks

Command:

```text
git diff --check
```

Result:

```text
exit 0; no whitespace errors
```

## 10. Manual verification

Environment: macOS (aarch64), Homebrew PostgreSQL 17.10 initialised into a
disposable scratchpad data directory (UTF8 encoding, C collation), local
Redis 8.10.0. Three throwaway databases: `thoth` (empty-chain), `thoth_test`
(Rust harness), `thoth_populated` (representative populated). No staging or
production system was contacted at any point.

Steps and observed results:

1. `thoth migrate` on an empty database → full 15-migration chain applied in
   4.06 s; ledger head `20260902`; the new enum reported exactly
   `{PENDING,APPROVED,REVOKED}`.
2. `\d+ public.metric_publisher_platform_approval` → exactly the nine
   specified columns, types and nullability, including `timestamp with time
   zone` for `approved_at`; exactly two indexes; exactly two non-cascading
   foreign keys.
3. `pg_constraint` / `pg_trigger` inventory → 1 primary key, 1 unique, 2
   foreign keys, **0** CHECK constraints, **0** non-internal triggers.
4. `SELECT count(*) FROM metric_publisher_platform_approval` → `0` (no seeds).
5. `thoth migrate --revert` → reverted the full chain cleanly (the repository
   implements `--revert` as `revert_all_migrations`, so this exercises the new
   `down.sql` inside the complete reverse chain); reapply restored 15
   migrations and 73 tables.
6. Base-schema populated database built with the base-commit binary, populated
   with representative data, snapshotted, migrated, re-snapshotted → all 19
   data-table content hashes identical; ledger +1 row; all 72 pre-existing
   relfilenodes unchanged.
7. Lock measurement inside an open migration transaction → `ShareRowExclusiveLock`
   observed on both `publisher` and `metric_platform` (see §6).
8. Down migration executed against the populated database → tables 73 → 72,
   enums 39 → 38, data-plus-ledger snapshot byte-identical to pre-migration;
   reapply reproduced the post-migration state.
9. `diff -r` of every pre-existing migration directory against the base
   worktree → identical, `20260901_v1.9.0` included.
10. GraphQL SDL generated from base and implementation trees → identical
    SHA-256 and `cmp` exit 0.

## 11. CI

CI status: **NOT AVAILABLE at the time this report was written.** The report
is committed before the branch is pushed and the draft PR opened, so no PR
run exists yet. Observed workflow, job and conclusion results — and the GHCR
`staging-pr-*` publication result — are reported in the post-push completion
handoff. GitHub remains the lifecycle authority.

Checks: pending PR creation.
Failures or warnings: none observed locally.

## 12. Rollout and rollback

Initial state after merge: additive, inactive schema on `feature/metrics`
only. No row exists, no code reads the table, no runtime behaviour changes.

Activation required: YES — later bounded, separately specified and authorized
work must implement approval administration, publisher-import authorization,
`PUBLISHER_CONTROLLED` enforcement and package-capability consumption before
this foundation has any effect.

Feature flag/configuration: none; there is nothing to switch on.

Migration sequence: `20260902_v1.9.0` applies after `20260901_v1.9.0`. It
depends on canonical `publisher` and on `metric_platform` from MET-WP1-01.
Production migration execution is **not authorized** by this task.

Rollback/disable procedure: before any later dependent Metrics migration or
runtime contract merges, rollback is a separately authorized revert of the
bounded child integration, with the tested `down.sql` used only in
disposable/non-production environments under applicable migration
authorization. Once later Metrics work depends on this schema, use
dependency-aware reverse-order rollback or a separately reviewed
forward-repair plan instead of reverting in isolation.

Monitoring required: none at this gate; nothing is active.

## 13. Known limitations and deferred work

- Persistence only. No approval creation, transition or revocation service
  exists.
- No `PUBLISHER_CONTROLLED` ownership-class enforcement; the design's binding
  runtime invariant must be enforced at the later import boundary.
- No package/capability entitlement check. ADR-0001 remains the sole shared
  package/capability authority and is unchanged.
- No publisher-import authorization and no current-publisher ownership check.
- `approved_by` has no FK, no actor-identity contract and no population rule.
- No GraphQL surface, so `thoth-app` cannot yet administer approvals.
- No source/platform/measure mapping and no seeded approval row.
- WP1 remains `IN PROGRESS`; this is the sixth WP1 slice, not WP1 completion.
- The clean-database timing figures come from a laptop-scale disposable
  instance with an empty new table; they are not a production-capacity
  prediction, and the write-blocking lock on `publisher` and `metric_platform`
  described in §6 still applies wherever this migration is eventually run.

## 14. Unresolved issues

- NONE within this slice's scope.
- Carried forward for a later specification, not resolvable here: the
  `approved_by UUID` actor/audit contract must be reconciled with the
  repository-authoritative ZITADEL string-identity model before any
  administrative write path populates the field. #878 §4.6 explicitly
  prohibits resolving it in this slice.

## 15. Agent self-assessment

The agent may identify risks but may not approve the task. This
implementation has **not** been independently reviewed and must not be merged
on the strength of this report.

Suggested review focus:

1. That the three recovery corrections in §1.3 are exactly what comment
   `5511593270` required, with no drift beyond them — in particular that the
   active tracker now pre-records no PR, CI, review or merge state.
2. The measured `ShareRowExclusiveLock` on `publisher` and `metric_platform`
   in §6, and whether the write-blocking characterisation is stated
   conservatively enough for a HIGH-risk schema slice.
3. That `approved_by` remains free of any invented FK, conversion or identity
   semantics in both SQL and Rust.
4. That the deliberate absences hold: no CHECK constraint, no trigger, no
   speculative index, no seeded row, no invented default on the usage, sales,
   status or `approved_at` columns.
5. That `thoth-api/src/schema.rs` matches the migration exactly and preserves
   the repository's manual formatting conventions under ADR-0003.
6. That the tests assert schema behaviour only and nowhere imply runtime
   authorization, approval transitions, capability enforcement or GraphQL
   behaviour exists.
