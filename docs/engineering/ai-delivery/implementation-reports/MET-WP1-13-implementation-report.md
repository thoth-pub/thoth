# MET-WP1-13 - implementation report

Task: `MET-WP1-13` - establish Metrics source and source-account administration
Owning issue: [#904](https://github.com/thoth-pub/thoth/issues/904)
Parent programme: [#766](https://github.com/thoth-pub/thoth/issues/766)
Repository: `thoth-pub/thoth`
Work package: WP1 - Metrics domain and database foundation
Risk: **HIGH**

## 1. Repository state

| Item | Value |
|---|---|
| Workflow | `PROGRAMME_INTEGRATION` |
| Base branch | `feature/metrics` |
| Authorized base commit | `9feddceeed5c09d7d560aaae5d2b4e5df70450e6` |
| Actual base commit | `9feddceeed5c09d7d560aaae5d2b4e5df70450e6` |
| Observed `develop` | `395cc16ac770bc8bbf8a708662a1b31d85b15398` |
| PR target / programme integration branch | `feature/metrics` |
| Task branch | `feature/metrics--wp1-source-admin` |
| Head commit | the PR head: the final commit on the task branch (a commit cannot record its own SHA; the implementation commit SHA is listed in section 3 and the exact head is recorded in the DRAFT PR body) |
| Pull request | DRAFT PR from `feature/metrics--wp1-source-admin` into `feature/metrics`; URL recorded in section 3 |
| Migration identity | `20260912` / `thoth-api/migrations/20260912_v1.9.0` |
| Write budget | 26 exact paths (23 frozen in `5624022951`; `metric_registry_tests.rs` added by `5632939633`; `metric_import_batch/tests.rs` added by `5636312947`; `metric_ingestion/tests.rs` added by `5636873961`) |
| Expected branch deletion after merge | YES |
| Final programme PR required | YES (`feature/metrics -> develop`, separately authorized) |
| Implementing model | Claude (Fable 5.1) |
| Reasoning level | default |

### 1.1 Authorization provenance

| Record | Location |
|---|---|
| Specification | #904 issue body |
| Specification Amendment 1 (typed configuration, compatibility, fail-closed decoder, JSONB equality) | #904 comment `5623620823` |
| Fresh final independent specification approval | #904 comment `5623809261` |
| Implementation-preflight binding (base, migration identity, 23-path budget) | #904 comment `5624022951` |
| CTO implementation authorization | #904 comment `5624260349` |
| Implementation Authorization Amendment 1 (24th path: `thoth-api/src/graphql/metric_registry_tests.rs`) | #904 comment `5632939633` |
| Independent `CHANGES REQUIRED` specification review | #904 comment `5623611717` |
| Specification Amendment 2 (current-base rebind; CloudFront `expectedPublisherId` pin) | #904 comment `5635355936` |
| Fresh current-base independent specification approval | #904 comment `5635950708` |
| Current CTO implementation authorization | #904 comment `5636000017` |
| Implementation Authorization Amendment 2 (25th path: `thoth-api/src/model/metric_import_batch/tests.rs`; local recovery) | #904 comment `5636312947` |
| Implementation Authorization Amendment 3 (26th path: `thoth-api/src/model/metric_ingestion/tests.rs`; DRIVER-fixture compatibility) | #904 comment `5636873961` |
| Migration reservation | #766 comment `5624024830` |

All records, the root `AGENTS.md`, `docs/engineering/AGENTS.md`,
`thoth-api/AGENTS.md`, `thoth-errors/AGENTS.md`,
`docs/engineering/ai-delivery/README.md` and the implementation-report
template were read in full before any write.

### 1.2 Preflight history

1. 2026-09-10, base `79c13845`: items 1-8 passed; HOLD on item 9 because the
   MET-WP1-12 point-in-time guards in
   `thoth-api/src/graphql/metric_registry_tests.rs` necessarily break and the
   file was outside the 23-path budget. Resolved by Implementation
   Authorization Amendment 1 (24 paths).
2. 2026-09-11, base `79c13845`: implementation completed inside the 24 paths;
   the full `thoth-api` suite exposed the MET-WP2-01A rollback-guard harness
   in `thoth-api/src/model/metric_import_batch/tests.rs`, which assumed
   `20260909` was the newest migration - HOLD, no commit.
3. 2026-09-11, `feature/metrics` moved to `9feddcee` (PR #905 / MET-WP2-01B).
   Specification Amendment 2 rebound the base and added the CloudFront
   `expectedPublisherId` pin; fresh approval and CTO authorization followed.
   Re-preflight found the same unlisted-path requirement, proven base-exact -
   HOLD.
4. Implementation Authorization Amendment 2 added the 25th path and the local
   recovery authority. After the recovery and the 25th-path repair, the full
   `thoth-api` suite exposed a second merged-predecessor fixture: the
   MET-WP2-01B shared ingestion fixture in
   `thoth-api/src/model/metric_ingestion/tests.rs` created its CloudFront
   managed `DRIVER` source without a `driver_key` (53 identical
   `metric_source_driver_key_check` failures) and its three
   `AcquisitionTypeDeferred` cases flipped `acquisition_type` without touching
   the key - HOLD, no commit.
5. Implementation Authorization Amendment 3 added the 26th path for exactly
   that fixture compatibility repair. Resume preflight before any source
   mutation (`feature/metrics` still `9feddcee`; `20260912` reserved and
   collision-free; no remote task branch; no PR) passed, and the existing
   exact-base worktree was retained as authorized. The earlier resume
   preflight, recorded here for completeness:

| Check | Result |
|---|---|
| remote `feature/metrics` after `git fetch --prune` | `9feddceeed5c09d7d560aaae5d2b4e5df70450e6` - matches |
| remote `develop` | `395cc16ac770bc8bbf8a708662a1b31d85b15398` - matches |
| `20260912_*` in any local or remote branch | none; #766 reservation `5624024830` intact and exclusive |
| remote `feature/metrics--wp1-source-admin` | absent |
| PR from that branch | none (`gh pr list … --state all` -> `[]`) |
| lossless backup of both stale local worktrees | 24 files each, byte-verified against the extracted archive |
| backed-up paths vs the 25-path envelope | 24 paths, **0 outside**; the only envelope path not present was the still-untouched 25th |
| local recovery | both stale worktrees and the old-base local branch removed (no remote ref touched); branch recreated from exactly `9feddcee`; files restored byte-identical to the backup |

6. After the Amendment 3 repair, the full `thoth-api` suite exposed one
   further MET-WP2-01B `metric_source` mutation in the same 26th path
   (`replay_survives_import_completion_and_configuration_changes`:
   `UPDATE metric_source SET acquisition_type = 'ADMIN_IMPORT', enabled =
   FALSE`, which left `driver_key = 'cloudfront'` on a non-DRIVER row) that
   Amendment 3 did not enumerate - HOLD before that mutation, no commit.
   Implementation Authorization Amendment 4 (`5637018261`) authorized exactly
   that one statement to also set `driver_key = NULL`. The resume preflight
   immediately before that edit, on 2026-09-11:

| Check | Result |
|---|---|
| remote `feature/metrics` after `git fetch` | `9feddceeed5c09d7d560aaae5d2b4e5df70450e6` - matches; zero commits since the base |
| remote `develop` | `395cc16ac770bc8bbf8a708662a1b31d85b15398` |
| `20260912*` migration directory on `origin/feature/metrics`, `origin/develop`, `origin/master` and every remote branch | none; #766 reservation `5624024830` intact and exclusive |
| remote `feature/metrics--wp1-source-admin` | absent |
| PR from that branch (`--state all`) | none; the only open PR into `feature/metrics` is the independent MET-WP5-01 draft, which is not merged and was not inspected or touched |
| doctrine files (`AGENTS.md` set, `CLAUDE.md`) in the task worktree | identical to `HEAD` |
| task worktree | `feature/metrics--wp1-source-admin` at exactly `9feddcee`, 18 modified + 8 untracked = exactly the 26 envelope paths, no other change |
| lossless backup before the edit | tracked diff byte-identical to the saved patch; 8 untracked files archived and SHA-256 verified (0 mismatches) |

7. Parallel-task isolation for the final validation. The first focused run
   after the Amendment 4 edit used a fresh task-isolated database but still
   the repository-wide shared Cargo target directory; seven MET-WP1-13 tests
   then failed with raw (unmapped) constraint messages although the ingestion
   suite and rollback guards passed. Cause, verified: Cargo does not hash a
   workspace member's path into its artifact name, so the parallel MET-WP5-01
   worktree's more recent compile of the base `thoth-errors` (which lacks the
   14 WP1-13 constraint mappings) was reused as "fresh" by this worktree's
   test binary. The source was correct throughout; the linked artifact was
   another task's. All final validation was therefore rerun from a clean,
   task-isolated `CARGO_TARGET_DIR` in addition to the isolated database and
   Redis namespace. The earlier results were discarded and are not cited.

## 2. Scope confirmation

Approved specification: #904 + Specification Amendment 1 + Specification
Amendment 2, as bound above.

Implemented objective: the protected SUPERUSER-only administration of exactly
`metric_source` and `metric_source_account` - six operations, the driver-key
invariant at both boundaries, the closed typed source-account configuration
with its fail-closed stored-value decoder, the parallel append-only
`metric_source_registry_history` audit, serialized one-row-lock concurrency,
bounded error sanitization, the Amendment 2 CloudFront `expectedPublisherId`
create-time pin, strictly additive SDL, the reconciliation of MET-WP1-12's
point-in-time guards, and the narrow MET-WP2-01A rollback-harness repair.

Out-of-scope changes made: NONE.

## 3. Commits

- implementation commit (SHA recorded by the follow-up report commit) - `MET-WP1-13: establish Metrics source and source-account administration`
  (all 26 envelope paths, validated as recorded in section 9);
- report commit (the PR head) - records the implementation commit SHA and the PR URL in this file only.

Pull request: recorded by the follow-up report commit.

## 4. Files changed

All 26 authorized paths were touched. Per file:

| Path | New | Reason / behavioural effect |
|---|---|---|
| `CHANGELOG.md` | | one bounded Unreleased entry |
| `docs/metrics/contract-register.md` | | new section 2.2 recording the delivered contract; section 2.1's deferred list narrowed to checkpoint administration |
| `docs/metrics/source-inventory.md` | | status, CloudFront row and section 5 updated to the public post-`CF-GATE-01` truth; no private value |
| `docs/metrics/task-status.md` | | header narrative, new `MET-WP1-13` row, own clause appended to the WP1 summary row; the `MET-WP1-12` row is byte-identical |
| `thoth-api/src/schema.rs` | | two audit `sql_types`, `metric_source_registry_history` `table!`, `allow_tables_to_appear_in_same_query!` entry; no `joinable!` (no FK) |
| `thoth-api/src/graphql/mod.rs` | | registers `metric_source_registry_tests` |
| `thoth-api/src/graphql/model.rs` | | `MetricSource` and `MetricSourceAccount` object types (identifiers only, typed configuration only) |
| `thoth-api/src/graphql/mutation.rs` | | four resolvers through the existing `authorize_metric_registry_admin` guard |
| `thoth-api/src/graphql/query.rs` | | two lookups through `require_superuser()` |
| `thoth-api/src/model/mod.rs` | | registers `metric_source_registry_history` |
| `thoth-api/src/model/metric_source/mod.rs` | | `Serialize` on the row, additive `GraphQLEnum` exposure of the acquisition enum, `NewMetricSource`, `PatchMetricSource`, `accepts_driver_key` |
| `thoth-api/src/model/metric_source/tests.rs` | | DRIVER fixture now carries a driver key; coordinator/invariant/audit/lock/concurrency suite |
| `thoth-api/src/model/metric_source_account/mod.rs` | | `Serialize` on the row, the closed typed configuration types, canonicalization and fail-closed decoder, compatibility matrix |
| `thoth-api/src/model/metric_source_account/tests.rs` | | decoder truth tables, coordinator, unsupported-stored-value, no-op, lock, parent-lock runtime and concurrency suite |
| `thoth-errors/src/database_errors.rs` | | 14 bounded mappings for every newly reachable named constraint, with a sanitization test |
| `thoth-api/src/model/metric_source/crud.rs` | yes | source coordinator |
| `thoth-api/src/model/metric_source_account/crud.rs` | yes | account coordinator |
| `thoth-api/src/model/metric_source_registry_history/mod.rs` | yes | audit model and `record_create`/`record_update` |
| `thoth-api/src/model/metric_source_registry_history/tests.rs` | yes | audit shape, driver-key CHECK truth table, empty and populated migration evidence |
| `thoth-api/src/graphql/metric_source_registry_tests.rs` | yes | six-operation surface, authorization matrix, sanitized failures, exact SDL, negative scope |
| `thoth-api/src/graphql/metric_registry_tests.rs` | | bounded reconciliation of three stale guards (section 5.1) |
| `thoth-api/src/model/metric_import_batch/tests.rs` | | narrow harness repair of `try_revert_ingestion_contract_migration` (section 5.2) |
| `thoth-api/src/model/metric_ingestion/tests.rs` | | narrow DRIVER-fixture compatibility repair (section 5.3) |
| `thoth-api/migrations/20260912_v1.9.0/up.sql` | yes | driver-key CHECK, two audit enums, audit table |
| `thoth-api/migrations/20260912_v1.9.0/down.sql` | yes | exact reverse |
| `docs/engineering/ai-delivery/implementation-reports/MET-WP1-13-implementation-report.md` | yes | this report |

Files deleted, moved or renamed: NONE.

### 4.1 Write-budget compliance

PASS - `git diff --name-only` against the base lists exactly the 26 authorized
paths and nothing else (verified before commit; see section 9).

### 4.2 Authorized actions actually used

- repository inspection: used (GitHub reads of #904/#766 records; git reads)
- source edit: used, within budget
- new file creation: used, exactly the authorized new paths
- file deletion/move/rename: NOT USED
- branch creation: used (`feature/metrics--wp1-source-admin` from the exact base, in an isolated git worktree - deviation D1)
- commit: used
- push: used
- PR creation/update: used (one DRAFT PR)
- issue/comment mutation: NOT USED
- manual CI dispatch/rerun: NOT USED
- provider/runtime read: NOT USED
- provider/runtime write: NOT USED
- migration execution: used against disposable local PostgreSQL only. Earlier runs used the repository's local `thoth_test` harness database, which another parallel task also used; because that left `20260912` applied on a database the other task had to revert, all final validation was moved to a task-isolated disposable database `met_wp1_13_test` (created empty, migrated only by this worktree's embedded harness) with the harness pointed at it via `TEST_DATABASE_URL`, and Redis isolated to a dedicated logical database via `TEST_REDIS_URL` (`.../13`); the separate `met_wp1_13_disposable` database served the CLI apply/revert/reapply evidence. No other task's database was migrated, reverted, reset or truncated; shared, staging and production databases were never accessed
- release/tag/publication: NOT USED
- merge: NOT USED
- deployment: NOT USED
- production activation: NOT USED
- other: a temporary throwaway git worktree of the old exact base was built to generate the base SDL for the additive-schema proof, then removed; the local recovery (backup, cleanup of my own stale worktrees/branch, recreation from the exact base) authorized by `5636312947` was performed as described in section 1.2; the final validation used a task-isolated Cargo target directory outside the repository (section 1.2 item 7) so that no artifact was shared with, or written into, another task's build cache

Unauthorized actions performed: NONE.

### 4.3 Automatic and manual external effects

Automatic CI/provider effects observed: opening the DRAFT PR triggers the repository's normal PR workflows, including the previously documented automatic `ghcr.io/thoth-pub/thoth:staging-pr-<PR number>` container-image publication. Observed read-only; status in section 11.

Manually initiated external actions: NONE.

External writes/publication: NONE beyond the authorized push and DRAFT PR.

## 5. Implementation decisions

1. **Guard reuse.** All four mutations use MET-WP1-12's
   `authorize_metric_registry_admin` and both lookups use
   `require_superuser()`, exactly the WP1-12 shape. No new authorization
   subsystem, role or policy.
2. **Application-level validation error variant.** `thoth-errors/src/lib.rs`
   is outside the budget, so no `ThothError` variant was added. Validation
   failures the coordinator decides itself (driver-key invariant, configuration
   truth table, compatibility matrix, hostname/external-key equality,
   unsupported stored configuration) are returned as
   `ThothError::DatabaseConstraintError(Cow::Borrowed(<fixed message>))`: a
   fixed, non-interpolated, bounded message with the same client-facing shape
   as every constraint failure already produced by this surface (`extensions.type
   = INTERNAL_ERROR`, message verbatim). The driver-key message is
   byte-identical between the coordinator pre-check and the database mapping,
   so the two boundaries are indistinguishable to a client. Review focus: this
   reuses existing safe machinery rather than a purpose-named variant.
3. **Driver-key CHECK and SQL three-valued logic.** The first draft of the
   CHECK omitted `driver_key IS NOT NULL`; under PostgreSQL's three-valued
   logic a NULL key makes the regex predicate NULL and a NULL CHECK passes.
   The tests caught it (`DRIVER` + NULL was accepted); the shipped constraint
   carries the explicit `IS NOT NULL` and the tests prove the full truth table.
4. **Stored-configuration decoder placement.** The decoder is pure Rust in
   `metric_source_account/mod.rs`, used by the lookup, the update (before any
   write and before `before_state` serialization) and the GraphQL
   `configuration` field. Compatibility with the immutable source is checked
   in the coordinator with an ordinary non-locking read of the source row.
5. **Semantic JSONB equality.** No-op detection compares `serde_json::Value`
   (map equality, key-order independent) and the tests replant the same value
   with a different key order and whitespace to prove text is not a contract
   semantic.
6. **Migration fails closed on a pre-existing violation.** `ALTER TABLE ... ADD
   CONSTRAINT` validates existing rows; the populated-database test plants a
   `DRIVER` row without a key and proves the migration refuses, leaves no
   partial object, deletes and rewrites nothing, and applies once the row is
   removed. No backfill or cleanup is performed by the migration, per #904.
7. **Predecessor guard reconciliation** - see 5.1 and 5.2.
8. **Amendment 2 pin.** `create_metric_source_account` refuses a
   `CLOUDFRONT_LEGACY_S3_V1` account whose `expected_publisher_id` is `None`
   with the fixed message of `ConfigurationError::CloudFrontRequiresExpectedPublisher`,
   decided after source resolution and the compatibility matrix and before
   canonicalization or any write; an unknown publisher still fails at the FK
   with its bounded mapping. The SDL field `expectedPublisherId: Uuid` stays
   nullable (an `EMPTY` account may omit it), the column stays nullable, and
   the pin is immutable because `PatchMetricSourceAccount` has no such field
   and the `UPDATE` never writes it.

### 5.1 Bounded change to `thoth-api/src/graphql/metric_registry_tests.rs`

Authorized by Implementation Authorization Amendment 1. Three assertions were
reconciled; every positive assertion for the nine MET-WP1-12 operations and
every unrelated SDL, authorization, audit, type and sanitization assertion is
unchanged:

- `the_sdl_exposes_exactly_the_nine_approved_operations`: the "exactly 3
  query / 6 mutation Metrics fields" counts now exclude the six approved
  WP1-13 operation names by name and still assert exactly 3 and 6 registry
  fields; the blanket deferred `"metricSource"` substring prohibition is
  replaced by bounded negatives for the still-deferred `metricSources`,
  `metricSourceAccounts` and `metricSourceCheckpoint`.
- `the_resolvers_authorize_before_reaching_a_coordinator`: the global
  `authorize_metric_registry_admin(context)` count 6 -> 10 and the global
  `require_superuser()` count 5 -> 7, with the "exactly one shared guard"
  assertion retained. The WP1-13 call sites are proven individually in
  `metric_source_registry_tests.rs`.

### 5.2 Narrow change to `thoth-api/src/model/metric_import_batch/tests.rs`

Authorized by Implementation Authorization Amendment 2. Only
`try_revert_ingestion_contract_migration` changed: it now loops
`revert_last_migration` while the newest applied version is greater than
`20260909`, asserts the MET-WP2-01A migration is still applied, then attempts
exactly the `20260909` rollback and returns its error as before. The doc
sentence stating that MET-WP2-01A is the newest migration was replaced. The
three fail-closed guards (`rollback_fails_closed_when_a_batch_row_exists`,
`…_provenance_hash_is_null`, `…_stable_code_was_assigned`), their assertions
and every other test in the file are byte-identical.

### 5.3 Narrow change to `thoth-api/src/model/metric_ingestion/tests.rs`

Authorized by Implementation Authorization Amendment 3. Exactly three
changes: the shared fixture's `INSERT INTO metric_source` for the CloudFront
managed `DRIVER` source now supplies `driver_key = 'cloudfront'`; each of the
three `AcquisitionTypeDeferred` cases sets `driver_key = NULL` in the same
`UPDATE` that changes `acquisition_type` to `PUBLISHER_UPLOAD`, `OPERAS` or
`ADMIN_IMPORT`; and each restore sets `acquisition_type = 'DRIVER',
driver_key = 'cloudfront'` in one statement. Implementation Authorization
Amendment 4 authorized exactly one further statement in
`replay_survives_import_completion_and_configuration_changes`: the existing
`UPDATE metric_source SET acquisition_type = 'ADMIN_IMPORT', enabled = FALSE`
now also sets `driver_key = NULL` in the same statement. No assertion,
expected error code or result (the replay still succeeds with rows equal to
the original result and an unchanged snapshot; the first-time `b2` batch
still fails with `ImportNotProcessing`), coordinator, hashing, authorization,
fixture identity or concurrency behaviour changed; the file's other tests are
byte-identical. The complete diff of this path against `9feddcee` is those
five statements (section 9, envelope reconciliation).

Deviations from the specification requiring authorization:

- D1. **Working tree at the first re-preflight (old base).** The main
  checkout was on the user's in-progress WP2-01B branch with a stat-dirty
  entry for `thoth-api/src/model/metric_ingestion/tests.rs` (`git diff`
  empty). It was not stashed, discarded or touched; the task branch was
  created in an isolated gitignored worktree whose tree was clean.
- D2. **Throwaway base worktree for the SDL proof.** A detached worktree of
  the old base was built with a separate target directory to generate the base
  schema, then removed. No repository file was changed by it.
- D3. **Post-HOLD local mutation (recorded by the CTO in `5636312947`).**
  After identifying the unlisted-path HOLD on the rebound base, I applied the
  in-budget patch to a temporary detached worktree of `9feddcee` to reproduce
  the three guard failures base-exactly. External effects were none, but it
  exceeded the stop-at-HOLD boundary; it is recorded as a bounded
  authorization-compliance deviation and the pattern is not repeated.

## 6. Database and migration effects

Migration added: YES - `thoth-api/migrations/20260912_v1.9.0`.

- schema effect (`up.sql`):
  - `ALTER TABLE metric_source ADD CONSTRAINT metric_source_driver_key_check
    CHECK ((acquisition_type = 'DRIVER' AND driver_key IS NOT NULL AND
    driver_key ~ '[^[:space:]]') OR (acquisition_type <> 'DRIVER' AND
    driver_key IS NULL))`;
  - `CREATE TYPE metric_source_registry_history_entity AS ENUM ('SOURCE',
    'SOURCE_ACCOUNT')`;
  - `CREATE TYPE metric_source_registry_history_action AS ENUM ('CREATE',
    'UPDATE')`;
  - `CREATE TABLE metric_source_registry_history` with
    `metric_source_registry_history_id uuid PK DEFAULT uuid_generate_v4()`,
    `entity`, `entity_id uuid` (no FK), `action`, `actor text`,
    `before_state jsonb NULL`, `after_state jsonb NOT NULL`, `created_at
    timestamptz DEFAULT CURRENT_TIMESTAMP`, constraints
    `metric_source_registry_history_pkey`,
    `metric_source_registry_history_actor_check` and
    `metric_source_registry_history_action_before_state_check`; no index
    beyond the primary key, no trigger.
- `schema.rs` effect: two `sql_types` structs, one `table!`, one
  `allow_tables_to_appear_in_same_query!` entry, no `joinable!`. The CHECK
  itself has no `schema.rs` representation (reviewed conclusion).
- existing-data effect: none rewritten. The CHECK is validated against
  existing `metric_source` rows on apply and the migration fails closed if a
  row violates it; `metric_source_account.configuration` values are untouched.
- locking/downtime: measured on the disposable database at `9feddcee` by running `up.sql`
  inside one transaction against the pre-migration state and inspecting
  `pg_locks` before `ROLLBACK`. Locks on pre-existing objects: exactly one -
  `AccessExclusiveLock` on `metric_source` for `ADD CONSTRAINT`, held for the
  statement (single-digit milliseconds here; on a production table it is the
  duration of one sequential scan validating existing rows). `metric_source_account`,
  `metric_registry_history` and every other existing table are not locked.
  The remaining locks are on the new table and its primary key only. Whole
  migration under 15 ms; no downtime.
- empty database result: `cargo run -- migrate -D <disposable>` on the empty
  UTF8/C-locale `met_wp1_13_disposable` applied all 23 migrations; the CHECK,
  both enums (`{SOURCE,SOURCE_ACCOUNT}`, `{CREATE,UPDATE}`) and the audit table
  are present and no source, account, platform or audit row exists. `cargo run
  -- migrate --revert -D <disposable>` (the runner's revert-all) then removed
  every migration and every `metric_%` table, and a second `migrate` restored
  all 23 with `metric_source_driver_key_check` present, the audit table empty
  and the two measure seeds intact.
- populated database result: proven by
  `metric_source_registry_history::tests::applying_to_a_populated_database_preserves_valid_rows_and_fails_closed_on_a_violation`
  (four sources of every acquisition type, one platform, one account with
  generic pre-existing JSON: byte-identical after apply; a planted violating
  row makes apply fail closed with no partial object).
- rollback/forward repair: `down.sql` drops the audit table, the two enums and
  the CHECK, in that order, and touches nothing else. Targeted revert through
  `20260912` and reapply proven by
  `the_migration_reverts_and_reapplies_leaving_every_predecessor_intact`
  (21 predecessor tables, the measure seeds and `metric_source`'s five
  predecessor constraints intact; reapply seeds nothing). CLI evidence in
  section 9.
- idempotency: Diesel's migration ledger; reapplying after revert restores the
  identical contract.

## 7. API and compatibility effects

GraphQL/API changes: strictly additive. Added: `type MetricSource`,
`type MetricSourceAccount`, `type MetricSourceAccountConfiguration`,
`type MetricCloudFrontLegacyS3Configuration`, `input NewMetricSource`,
`input PatchMetricSource`, `input NewMetricSourceAccount`,
`input PatchMetricSourceAccount`, `input MetricSourceAccountConfigurationInput`,
`input MetricCloudFrontLegacyS3ConfigurationInput`,
`enum MetricSourceAcquisitionType` (additive exposure of the existing closed
database enum), `enum MetricSourceAccountConfigurationKind`, the four
mutations and two queries listed in section 2.2 of the contract register.
Nullability matches Amendment 1 exactly (`configuration:
MetricSourceAccountConfigurationInput!` on both inputs;
`MetricSourceAccount.configuration: MetricSourceAccountConfiguration!`;
`cloudfrontLegacyS3` nullable on input and output). No raw JSON scalar or
field exists.

Generated schema/client updates: `thoth-client/assets/schema.graphql` is
build-generated and gitignored; no committed generated artifact exists or was
changed. Base-vs-head SDL diff: the exact base `9feddcee` was built in a
throwaway detached worktree (`cargo check --workspace`, separate target
directory, removed afterwards) and its generated schema (SHA-256
`e639c59f2de50030ba0fdea8e7efc2f45c9f6b499bb821e41348ea2241cd36bf`, 188,673
bytes) diffed against the head's (SHA-256
`29aa897b2af558f75c59c45156420bf65eb27409f9bd0baf76ce8e477bb945cb`, 196,933
bytes; regenerated by the final-source build in the task-isolated target
directory after the last source change): `diff` reports **0 lines present
only in the base and 126 lines present only in the head**, and the set of
non-blank base lines is entirely contained in the head (0 missing); the added
declarations are exactly `enum MetricSourceAccountConfigurationKind`,
`enum MetricSourceAcquisitionType`, `input MetricCloudFrontLegacyS3ConfigurationInput`,
`input MetricSourceAccountConfigurationInput`, `input NewMetricSource`,
`input NewMetricSourceAccount`, `input PatchMetricSource`,
`input PatchMetricSourceAccount`, `type MetricCloudFrontLegacyS3Configuration`,
`type MetricSource`, `type MetricSourceAccount`,
`type MetricSourceAccountConfiguration`, and the six root fields.
`NewMetricSourceAccount.expectedPublisherId` remains the nullable `Uuid`
Amendment 1 froze; the Amendment 2 requirement is enforced by the coordinator.

Backwards compatibility: no existing type, field, argument, nullability,
default, enum value or authorization behaviour changed; no consumer requires a
change.

Deprecations: NONE.

Cross-repository dependencies: none required. Sphinx / the CloudFront driver
may consume this contract only once merged and pinned.

## 8. Authorization and security

Authorization paths changed: none added. All six operations reuse the
existing SUPERUSER guard; authorization runs before any operation-specific
database access.

Roles/scopes involved: `SUPERUSER` only. Denied and proven denied:
anonymous, authenticated without role, `PUBLISHER_USER`, `PUBLISHER_ADMIN`,
`WORK_LIFECYCLE`, `CDN_WRITE`, `DISSEMINATION_WORKER`, and a principal holding
every non-superuser role at once.

Negative authorization tests: `metric_source_registry_tests::every_operation_is_denied_to_every_caller_that_is_not_a_superuser`
(state and both audit tables untouched) and
`authorization_is_decided_before_any_database_access` (unreachable pool
still yields `NO_ACCESS`).

Secret or personal-data handling: no credential field exists in any input,
output or stored shape; the diff was scanned for private `CF-GATE-01`
identifiers, provider hostnames, bucket names, distribution IDs, publisher
UUIDs, account IDs and credential patterns - none present. Unsupported stored
configuration is never echoed: tests plant secret-looking keys and prove no
fragment reaches any error.

Security limitations: application-level validation failures share the
`INTERNAL_ERROR` `extensions.type` of every existing constraint failure on
this surface (decision 2).

## 9. Tests and checks

All final runs were executed after the last source change (the Amendment 4
statement), from the task worktree, with `TEST_DATABASE_URL` pointing at the
task-isolated `met_wp1_13_test` database (UTF8 / C locale, created empty and
migrated only by this worktree's embedded harness), `TEST_REDIS_URL` at a
dedicated Redis logical database, and a task-isolated `CARGO_TARGET_DIR`
outside the repository (section 1.2 items 6-7). No earlier result is cited.

### Predecessor compatibility (Amendments 2-4), run first

Command:

```text
cargo test -p thoth-api --features backend --lib metric_ingestion::
cargo test -p thoth-api --features backend --lib metric_import_batch::tests::rollback_fails_closed
```

Result:

```text
metric_ingestion::  test result: ok. 78 passed; 0 failed; 0 ignored (all MET-WP2-01B tests, including
                    replay_survives_import_completion_and_configuration_changes ... ok, with their
                    assertions and expected outcomes unchanged; the previous
                    metric_source_driver_key_check failures are gone)
rollback guards     test result: ok. 3 passed; 0 failed (rollback_fails_closed_when_a_batch_row_exists,
                    ..._when_a_stable_code_was_assigned, ..._when_a_provenance_hash_is_null)
```

### MET-WP1-13 focused suites

Command:

```text
cargo test -p thoth-api --features backend --lib metric_source::tests
cargo test -p thoth-api --features backend --lib metric_source_account::tests
cargo test -p thoth-api --features backend --lib metric_source_registry_history::tests
cargo test -p thoth-api --features backend --lib graphql::metric_source_registry_tests
cargo test -p thoth-api --features backend --lib graphql::metric_registry_tests
cargo test -p thoth-errors
```

Result:

```text
metric_source::tests                    ok. 24 passed; 0 failed
metric_source_account::tests            ok. 29 passed; 0 failed
metric_source_registry_history::tests   ok. 10 passed; 0 failed
graphql::metric_source_registry_tests   ok. 11 passed; 0 failed
graphql::metric_registry_tests          ok. 12 passed; 0 failed
thoth-errors                            ok. 13 passed; 0 failed (12 at base + the new
                                        metrics_source_administration_constraints_map_to_bounded_messages)
```

### Formatting

Command:

```text
cargo fmt --all -- --check
git diff --check
```

Result:

```text
exit=0
exit=0
```

### Unit and integration/database tests (complete final-source runs)

Command:

```text
cargo test -p thoth-api --features backend
cargo test --workspace
```

Result:

```text
cargo test -p thoth-api --features backend
  test result: ok. 1754 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 311.90s
  test result: ok. 13 passed; 0 failed  (bin)
  test result: ok. 0 passed; 0 failed; 8 ignored  (doctests)
  exit=0
cargo test --workspace
  thoth-api lib: ok. 1754 passed; 0 failed   thoth-errors: ok. 13 passed
  every other crate/binary/doctest target: ok (31, 3, 4, 13, 144, 6, 2 passed; 8 ignored doctests); 0 failed anywhere
  exit=0
```

### Lint/static analysis

Command:

```text
cargo check --workspace
cargo clippy --all --all-targets --all-features -- -D warnings
```

Result:

```text
cargo check --workspace: exit=0 (only the pre-existing proc-macro-error2 future-incompat note)
cargo clippy ... -D warnings: exit=0
```

### Migration evidence on the dedicated disposable database (final binary)

Command:

```text
psql ... -f thoth-api/migrations/20260912_v1.9.0/down.sql   (returned the database to 20260909, own task database)
cargo run -- migrate -D <met_wp1_13_disposable>              (apply)
cargo run -- migrate --revert -D <met_wp1_13_disposable>     (runner revert-all)
cargo run -- migrate -D <met_wp1_13_disposable>              (reapply)
```

Result:

```text
apply:   exit=0; ledger max=20260912, 23 rows;
         metric_source_driver_key_check = CHECK ((((acquisition_type = 'DRIVER') AND (driver_key IS NOT NULL) AND (driver_key ~ '[^[:space:]]')) OR ((acquisition_type <> 'DRIVER') AND (driver_key IS NULL))))
         metric_source_registry_history_action = {CREATE,UPDATE}; ..._entity = {SOURCE,SOURCE_ACCOUNT}
         audit-table constraints: _action_before_state_check, _actor_check, _pkey; audit rows = 0
revert:  exit=0; ledger rows = 0; metric_% tables = 0; driver-key CHECK = 0
reapply: exit=0; ledger max=20260912 / 23 rows; CHECK present = 1; audit rows = 0; metric_measure seeds = 2
lock profile (up.sql in one transaction, pg_locks before ROLLBACK, re-measured on the final source):
         pre-existing relations: metric_source AccessExclusiveLock only;
         new objects: metric_source_registry_history AccessExclusiveLock+ShareLock, its pkey AccessExclusiveLock
populated-database and targeted revert/reapply evidence: the two harness tests named in section 6, green above.
```

### Envelope reconciliation

Command:

```text
{ git diff --name-only 9feddceeed5c09d7d560aaae5d2b4e5df70450e6; git ls-files --others --exclude-standard; } | sort -u
```

Result: exactly the 26 authorized paths, 0 outside. Predecessor-test paths:
`metric_registry_tests.rs` - only the 5.1 reconciliation (44+/13-);
`metric_import_batch/tests.rs` - only the 5.2 harness loop (one hunk);
`metric_ingestion/tests.rs` - exactly the five statements of 5.3 (fixture
INSERT with `driver_key = 'cloudfront'`, the three `AcquisitionTypeDeferred`
mutate/restore pairs, and the Amendment 4 statement), nothing else.

### Observation, not a change

`cargo check -p thoth-client` on its own (which compiles `thoth-api` without
the `backend` feature under package-level feature resolution) fails at the
base and at the head alike with unresolved `crate::graphql` / `crate::policy`
imports in base-owned modules (`model/abstract`, `model/affiliation`, ...,
`model/mod.rs:1`, all authored before this task). Workspace-level builds,
which CI uses, unify features and pass. Nothing in this slice touches that
gating; recorded for a separate task.

## 10. Manual verification

Environment: local Homebrew PostgreSQL 17.10 (task-isolated `met_wp1_13_test`,
UTF8 / C locale like the repository test database, for the final harness
runs; dedicated `met_wp1_13_disposable` for the embedded-runner run), local
Redis on a task-isolated logical database, task-isolated Cargo target
directory.
Steps and observed results: section 9.

## 11. CI

CI status: PENDING at DRAFT PR creation; natural PR checks only, observed read-only; no manual dispatch, rerun or cancel. The settled result is recorded in the PR body.

## 12. Rollout and rollback

Initial state after merge: additive, inactive. The migration adds a CHECK,
two enums and one empty table; no row is seeded and no behaviour runs until a
superuser invokes an operation.

Activation required: none for the administration surface itself. Creating a
real source/account is a separately authorized administrative act under the
approved source contract; collection remains unimplemented.

Feature flag/configuration: none.

Migration sequence: after `20260909_v1.9.0`; production execution remains
governed by CG-13 and separate release authorization.

Rollback/disable procedure: `down.sql` (lossy only for audit rows written
after activation); source rollback is a separately authorized revert.

Monitoring required: none new.

## 13. Known limitations and deferred work

- Checkpoint administration and claim/lease runtime, the CloudFront driver,
  synthetic driver fixtures, the GeoIP resource, list/search/delete/bulk
  source operations and any audit-history query remain deferred.
- A new non-empty configuration for another source requires a separately
  reviewed additive configuration kind/version.
- Repair of an account whose stored configuration is unsupported is
  separately reviewed work; this surface fails closed on it by design.

## 14. Unresolved issues

- NONE.

## 15. Agent self-assessment

The agent may identify risks but may not approve the task.

Suggested review focus:

- decision 2 (validation errors via `DatabaseConstraintError`);
- the exact CHECK expression and its `IS NOT NULL` clause;
- the decoder's exact-key-set rules and the compatibility check in
  `ensure_supported`;
- the bounded reconciliations in `metric_registry_tests.rs` (5.1),
  `metric_import_batch/tests.rs` (5.2) and `metric_ingestion/tests.rs` (5.3);
- deviation D1.
