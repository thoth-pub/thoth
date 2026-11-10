# MET-WP1-04 Implementation Report

## 1. Repository state

Owning GitHub issue: [#872](https://github.com/thoth-pub/thoth/issues/872)
Repository: `thoth-pub/thoth`
Workflow: PROGRAMME_INTEGRATION
Base branch: `feature/metrics`
Authorized base commit: `30185af82c4f222127cf25adea6321a25c5307df`
Actual base commit: `30185af82c4f222127cf25adea6321a25c5307df` (verified identical immediately before branch creation)
Observed `develop` checkpoint: `4546cb632428872b961ad6c17282984d298e3ade` (verified identical; no programme refresh required)
PR target: `feature/metrics`
Programme integration branch: `feature/metrics`
Task branch: `feature/metrics--wp1-record-schema`
Head commit: recorded in the control-plane handoff for the exact pushed head; this report is committed as the second commit on the branch
Pull request: **NOT CREATED - NOT AUTHORIZED.** PR creation and its automatic GHCR staging-image publication are a separate later gate.
Expected branch deletion after merge: YES
Final programme PR required: YES (`feature/metrics -> develop`, separately gated and not implied here)
Implementing model: Claude Opus 5
Reasoning level: extended

Controlling records read in full before implementation: issue #872; fresh
independent specification review comment `5477319756` (APPROVED); CTO
specification approval comment `5477482506`; exact-SHA bounded implementation
authorization comment `5477620090`.

## 2. Scope confirmation

Approved specification: the exact #872 written specification, independently
reviewed in `5477319756` and CTO-approved in `5477482506`.

Implemented objective: the additive, initially inactive canonical Metrics
record-history foundation — `metric_record`, `metric_record_revision`,
`metric_record_provenance`, the closed `metric_record_revision_status` and
`metric_record_provenance_classification` enums, the manually maintained
Diesel schema contract, the matching Rust persistence/domain modules and
focused database tests.

Out-of-scope changes made: NONE.

## 3. Commits

- `c900463749c4fba995f72d5bc40781bd74ecd96b` - MET-WP1-04: establish the canonical Metrics record, revision and provenance foundation
- second commit - MET-WP1-04: add the implementation report (this file); its SHA is reported in the control-plane handoff

## 4. Files changed

Authorized write paths (existing files):

- `CHANGELOG.md`
- `docs/metrics/task-status.md`
- `thoth-api/src/schema.rs`
- `thoth-api/src/model/mod.rs`

Authorized new-file paths:

- `thoth-api/migrations/20260831_v1.9.0/up.sql`
- `thoth-api/migrations/20260831_v1.9.0/down.sql`
- `thoth-api/src/model/metric_record/mod.rs`
- `thoth-api/src/model/metric_record/tests.rs`
- `thoth-api/src/model/metric_record_revision/mod.rs`
- `thoth-api/src/model/metric_record_revision/tests.rs`
- `thoth-api/src/model/metric_record_provenance/mod.rs`
- `thoth-api/src/model/metric_record_provenance/tests.rs`
- `docs/engineering/ai-delivery/implementation-reports/MET-WP1-04-implementation-report.md`

Actual files changed:

- `thoth-api/src/schema.rs`
  - reason: ADR-0003 requires the manually maintained Diesel contract to change atomically with the migration.
  - behavioural effect: adds the two `sql_types` structs, the three `table!` blocks, ten `joinable!` declarations and three `allow_tables_to_appear_in_same_query!` entries. No existing table, column, type, alias or ordering is altered; the file's zero-comment convention is preserved.
  - within authorized write budget: YES
- `thoth-api/src/model/mod.rs`
  - reason: register the three new model modules.
  - behavioural effect: three `pub mod` lines added in the existing alphabetical position. No other change.
  - within authorized write budget: YES
- `CHANGELOG.md`
  - reason: repository rule that every PR updates `## [Unreleased]`.
  - behavioural effect: one bounded `### Added` bullet, placed newest-first per existing convention. No existing entry altered; the v1.9 migration-path truth is preserved.
  - within authorized write budget: YES
- `docs/metrics/task-status.md`
  - reason: durable programme-truth reconciliation required by #872 §4.10.
  - behavioural effect: records the already-completed `MET-MIG-V1.9-RECON-01` migration-identity state (#868), which the tracker did not previously reflect; adds the `MET-WP1-04` row stating truthfully that the slice is implemented on its branch and **not merged**; keeps WP1 `IN PROGRESS`. No transient review, CI or merge-authorization identifier is copied in.
  - within authorized write budget: YES

Actual new files created — all nine listed above, each within the authorized new-file list: YES.

Files deleted, moved or renamed: NONE.

### 4.1 Write-budget compliance

PASS. The complete `git diff --name-status` set against the authorized base is
exactly the thirteen authorized paths (four modified, eight created by the first
commit, plus this report). No deletion, rename or copy appears.

## 4.2 Authorized actions actually used

- repository inspection: USED
- source edit (within write budget): USED
- new file creation (authorized paths only): USED
- file deletion/move/rename: NOT USED (not authorized)
- branch creation (`feature/metrics--wp1-record-schema` from `30185af8…`): USED
- local validation: USED
- local disposable PostgreSQL 17 migration execution: USED (disposable local cluster only)
- commit: USED
- push (ordinary, non-force): USED
- PR creation/update: NOT USED (not authorized)
- issue/comment mutation: NOT USED (not authorized)
- manual CI dispatch/rerun/cancel: NOT USED (not authorized)
- provider/runtime read: NOT USED (not authorized)
- provider/runtime write: NOT USED (not authorized)
- staging/production/shared-persistent migration execution: NOT USED (not authorized)
- persistent Diesel migration-ledger edit: NOT USED (not authorized)
- release/tag/publication: NOT USED (not authorized)
- merge: NOT USED (not authorized)
- deployment: NOT USED (not authorized)
- production activation: NOT USED (not authorized)
- other: an isolated `git worktree` at the exact authorized commit, and a
  disposable local PostgreSQL 17 cluster on a dedicated port, both purely
  local.

Unauthorized actions performed: NONE.

## 4.3 Automatic and manual external effects

Automatic CI/provider effects observed: NONE. The current workflow inventory
does not trigger the build or migration workflows on a task-branch push; only
the later, separately authorized PR would trigger build/test/lint/format,
migration validation, changelog validation and — for this non-documentation
diff — the classifier's `run_docker=true` path that pushes
`ghcr.io/thoth-pub/thoth:staging-pr-<PR>`. No such run was created here.

Manually initiated external actions: NONE.

External writes/publication: NONE.

## 5. Implementation decisions

1. **Same-record integrity by composite foreign key.** The approved design's
   circular record/current-revision relationship and the same-record
   `supersedes` invariant are both enforced declaratively: a
   `(record_id, record_revision_id)` unique key on `metric_record_revision`
   supports a composite FK from `metric_record (record_id,
   current_revision_id)` and a self-referential composite FK from
   `metric_record_revision (record_id, supersedes_revision_id)`. Under MATCH
   SIMPLE neither is enforced while its nullable column is NULL, so a record
   still needs no revision at creation and an initial revision needs no
   predecessor. This is the shape the independent review anticipated
   ("database-native composite-key/foreign-key integrity and a supporting
   unique key"), and it uses no trigger and no new dependency. `HOLD -
   DEPENDENCY/ARCHITECTURE AMENDMENT REQUIRED` was therefore not reached.
2. **Migration ordering.** `up.sql` creates the enums, then `metric_record`
   with a nullable `current_revision_id` and no revision FK, then
   `metric_record_revision`, then the partial unique current-revision index,
   then the circular constraint by `ALTER TABLE`, then the record access
   indexes, then `metric_record_provenance` and its audit indexes. `down.sql`
   drops provenance, then drops the circular constraint **explicitly** (no
   `DROP ... CASCADE`), then revisions, then records, then exactly this task's
   two enums. PostgreSQL refuses to drop `metric_record_revision` while that
   constraint exists, so the explicit drop is load-bearing, not decorative.
3. **Metrics country representation.** `country_code` is a nullable
   `character(2)` column with `CHECK (country_code ~ '^[A-Z]{2}$')`. Because
   `CHAR(n)` blank-pads and blank-strips, a one-character or empty value is
   rejected by that same shape check and a value longer than two characters is
   rejected by the column type. The existing bibliographic alpha-3
   `country_code` enum is untouched, and a test asserts it still carries its
   249 three-letter labels. Full ISO 3166-1 alpha-2 membership validation is
   deliberately absent and belongs to WP2.
4. **Diesel mapping for `CHAR(2)`.** `schema.rs` uses `Nullable<Bpchar>`, and
   the Rust field is `Option<String>`. `diesel::sql_types::Bpchar` is Diesel's
   own alias for PostgreSQL `character(n)` and resolves to `VarChar`/`Text`,
   so this introduces **no** dependency, feature or behavioural change; it
   records the physical column type in a file that carries no comments. A
   plain `Nullable<Text>` would be type-identical.
5. **`updated_at` behaviour.** `metric_record` uses
   `SELECT diesel_manage_updated_at('public.metric_record')`, the
   repository-standard mechanism already used by `metric_platform`,
   `metric_measure` and `metric_source_checkpoint`, as #872 §4.4 directs
   ("repository-standard current-time defaults/updated-at behavior"). The
   specification's "no triggers" requirement is scoped to the same-record
   integrity mechanism and to the revision state machine; neither is
   implemented with a trigger. A test asserts that `set_updated_at` is the
   **only** non-internal trigger on any `metric_record*` table.
   `metric_record_revision` and `metric_record_provenance` are immutable and
   have no `updated_at` column and no trigger.
6. **`joinable!` for the circular pair.** Diesel's `joinable!` generates a
   `JoinTo` implementation in both directions, so a pair cannot be declared
   twice. The conventional child-to-parent direction
   `metric_record_revision -> metric_record (record_id)` is declared, and the
   `metric_record -> metric_record_revision (current_revision_id)` direction is
   deliberately omitted. `supersedes_revision_id` is a self-join and is not
   expressible as a `joinable!` at all. Both are query conveniences; neither
   affects the enforced schema contract.
7. **Index inventory.** Exactly the required set: three primary keys, the
   unique `identity_hash`, four single-column `metric_record` access indexes
   (`work_id`, `platform_id`, `measure_id`, `period_start`),
   `UNIQUE(record_id, revision_number)`, the `(record_id,
   record_revision_id)` unique key that carries same-record integrity, the
   partial unique current-revision index, and three provenance audit indexes.
   The supporting unique key is the mechanism §4.8 mandates, not a speculative
   secondary index. Tests assert the complete index name list per table, so a
   later speculative index fails the suite.
8. **No overlap or arbitration primitive.** No exclusion constraint, no
   `btree_gist`, no advisory lock. A test inserts two records with overlapping
   periods and asserts they are both accepted, and asserts the absence of any
   exclusion constraint and of the `btree_gist` extension.
9. **Tracker truthfulness.** `docs/metrics/task-status.md` records
   `MET-WP1-04` as implemented on its slice branch and explicitly **not**
   merged, because it is not. It also records the completed `#868` v1.9
   migration-identity reconciliation the tracker was previously missing.

Deviations from the specification requiring authorization: NONE.

## 6. Database and migration effects

Migration added: YES.

- migration files: `thoth-api/migrations/20260831_v1.9.0/up.sql`,
  `thoth-api/migrations/20260831_v1.9.0/down.sql`. The path was reproduced
  from the repository convention immediately before writing: workspace version
  `1.8.0`, `MAJOR=1`, `MINOR=8`, `DATE=20260831`, so `make migration` resolves
  to `thoth-api/migrations/20260831_v1.9.0`, which was absent. `make migration`
  was then actually run and produced exactly that directory.
- schema effect: two closed enums; three new tables; twelve foreign keys, none
  cascading; seven CHECK constraints; fourteen indexes across the three tables.
  Nothing existing is altered.
- existing-data effect: NONE. No backfill, no update, no delete, no seed row.
- locking/downtime: the migration acquires `ShareRowExclusiveLock` (plus
  `AccessShareLock`) on exactly the seven referenced pre-existing tables —
  `work`, `publication`, `institution`, `metric_platform`, `metric_measure`,
  `metric_source_account`, `metric_import` — which is PostgreSQL's normal lock
  for adding a foreign key that references a table. It takes **no**
  `AccessExclusiveLock` on any existing table and takes no lock at all on
  `publisher`, `imprint`, `distribution_job`,
  `publisher_distribution_platform`, `metric_source`,
  `metric_source_checkpoint` or `metric_import_error`. Total DDL time on the
  populated database was ~16 ms across all statements (slowest single
  statement 3.8 ms), so the write-blocking window on those seven tables is
  brief. Reads are never blocked. **No table rewrite occurs**: every
  pre-existing table kept its `pg_class.relfilenode` across a full
  apply/revert/reapply cycle.
- empty database result: the full current migration chain applied cleanly to
  an empty disposable PostgreSQL 17.10 database. Verified afterwards: both
  enums present with exactly the approved labels in order; all three tables
  present with the approved columns, types, nullability and defaults
  (`country_code` is `character(2)`); twelve non-cascading foreign keys; seven
  CHECK constraints; fourteen indexes; zero exclusion constraints; `btree_gist`
  not installed; the only non-internal trigger is the repository-standard
  `set_updated_at` on `metric_record`; zero rows in all three tables; the
  existing alpha-3 `country_code` enum still carries 249 labels; the
  MET-WP1-01 measure seeds intact.
- populated database result: see section 10.
- rollback/forward repair: `down.sql` was executed against both the empty and
  the representative populated database and reapplied afterwards. It removes
  provenance, then the circular constraint, then revisions, then records, then
  exactly the two new enums, leaving `metric_reporting_grain` and all
  MET-WP1-01/02/03 objects and seed rows in place. The Diesel-harness path is
  additionally covered by
  `reverting_through_the_record_schema_migration_removes_it_and_reapplication_restores_it`.
- idempotency: the migration is pure additive DDL executed once by the Diesel
  ledger. It contains no data mutation, so repeat execution is not a scenario;
  revert-then-reapply was proven to restore the identical object set.

## 7. API and compatibility effects

GraphQL/API changes: NONE. The persistence enums are deliberately not
`juniper::GraphQLEnum` and no resolver, type, query, mutation or input was
added.

Generated schema/client updates: NONE required. The public SDL generated by
`thoth-client/build.rs` is **byte-identical** to the exact base build:

```text
BASE  sha256 091e11f293132fdec784de420e3addf251f5020ba7e387889b292a066be15d8e  (178270 bytes, 4630 lines)
AFTER sha256 091e11f293132fdec784de420e3addf251f5020ba7e387889b292a066be15d8e  (178270 bytes, 4630 lines)
```

The base artefact was produced by building the unmodified worktree at
`30185af82c4f222127cf25adea6321a25c5307df`; the after artefact by deleting the
generated file and rebuilding the final implementation. `cmp` reports no
difference, and the SDL contains zero occurrences of `MetricRecord`.

Backwards compatibility: fully additive and inactive.
Deprecations: NONE.
Cross-repository dependencies: NONE. `thoth-sphinx`, `thoth-app`,
`metrics-dashboard`, `metrics-widget`, `thoth-client` and
`thoth-dissemination` consume GraphQL/export contracts, which are unchanged.
No downstream repository task is required, and no downstream consumer may read
these tables directly.

## 8. Authorization and security

Authorization paths changed: NONE. `thoth-api/src/policy.rs` is untouched.
Roles/scopes involved: NONE. No machine role, credential or entitlement is
introduced.
Negative authorization tests: not applicable — this slice exposes no protected
operation. The existing 13 `graphql_permissions` integration tests still pass
unchanged.
Secret or personal-data handling: NONE. No credential, token or personal data
is stored, logged or transmitted. `details` is generic JSONB with no
source-specific schema.
Security limitations: none introduced. The tables are unreachable from any
runtime path in this slice.

## 9. Tests and checks

All commands were run in the isolated worktree at the final committed source,
against a disposable local PostgreSQL 17.10 cluster and a local Redis.

### Formatting

Command:

```text
cargo fmt --all -- --check
```

Result:

```text
exit 0, no diff reported
```

### Unit tests

Command:

```text
cargo test --workspace
```

Result:

```text
thoth (bin)            31 passed;  0 failed
thoth-api (lib)      1400 passed;  0 failed   (1343 pre-existing + 57 new)
graphql_permissions    13 passed;  0 failed
thoth-api-server        3 passed;  0 failed
thoth-client            4 passed;  0 failed
thoth-errors           11 passed;  0 failed
thoth-export-server   144 passed;  0 failed
doc-tests: thoth_client 6 passed; thoth_api 8 ignored; others 0
overall exit 0
```

### Integration/database tests

Command:

```text
cargo test -p thoth-api --features backend
```

Result:

```text
1400 passed; 0 failed; 0 ignored   (finished in 137.17s)
   13 passed; 0 failed             (graphql_permissions)
    0 passed; 8 ignored            (doc-tests)
overall exit 0
```

Command (the 57 tests added by this slice):

```text
cargo test -p thoth-api --features backend metric_record
```

Result:

```text
57 passed; 0 failed; 0 ignored; 1343 filtered out; finished in 8.99s
```

### Lint/static analysis

Command:

```text
cargo clippy --all --all-targets --all-features -- -D warnings
```

Result:

```text
exit 0, no warnings
(one pre-existing unrelated future-incompat note for proc-macro-error2 v2.0.1, also present on the base)
```

### Other required checks

Command:

```text
cargo check --workspace
```

Result:

```text
exit 0
```

Command:

```text
git diff --check
```

Result:

```text
exit 0, no whitespace errors
```

Command:

```text
git status --short ; git diff --stat
```

Result:

```text
working tree clean after commit; the committed diff is
12 files changed, 3180 insertions(+), 12 deletions(-)
```

### What the new tests prove

The 57 tests assert schema behaviour only. Coverage includes: every
revision-status and provenance-classification value round-tripping through
PostgreSQL/Diesel and through string conversion (with unknown values
rejected); complete row round-trips for all three tables including nullable
publication, country, institution and current-revision fields, signed negative
revision values, optional `supersedes`, nullable provenance record and source
fields and JSON `details`; database defaults exercised through raw SQL rather
than restated by fixtures; blank identity/content hash rejection; duplicate
identity rejection; invalid foreign keys failing closed for all six record
references and both revision and provenance references; referenced-row
deletion restricted rather than cascaded; null country accepted; exactly two
uppercase ASCII letters accepted; lowercase, mixed-case, non-letter, empty,
whitespace, one-character and three-character country representations
rejected; half-open period ordering enforced (inverted and empty rejected);
`revision_number > 0`; per-record uniqueness of `revision_number` with global
reuse permitted; cross-record current-revision and cross-record `supersedes`
references rejected; a second `CURRENT` revision per record rejected while
multiple `SUPERSEDED`/`RETRACTED` are retained; nullable provenance
`record_id` supporting rejected and conflicting evidence without creating a
canonical record; and the complete required index inventory asserted from
`pg_indexes` metadata per table.

The suite also asserts what is **not** implemented, so no absent behaviour is
falsely represented: two records with identical canonical dimensions but
different supplied identity hashes are both accepted (no identity derivation,
no first-arrival arbitration, no duplicate detection); overlapping periods are
accepted and no exclusion constraint or `btree_gist` exists; a record pointer
may reference a non-`CURRENT` revision because no WP1 constraint ties them; the
`metric_reporting_grain` enum is reused rather than duplicated; and
`set_updated_at` is the only non-internal trigger on any `metric_record*`
table.

## 10. Manual verification

Environment: disposable local PostgreSQL 17.10 (Homebrew), UTF8/C, dedicated
TCP port, isolated data directory, created and used only for this task. No
staging, production or shared persistent database was contacted.

**A. Empty full-chain apply.** Applied the complete current migration chain
including MET-WP1-04 to an empty database via the embedded runner
(`cargo run migrate`, after `touch thoth-api/src/db.rs` to force
`embed_migrations!` to pick up the new directory). Result: success, ledger
version `20260831` recorded. Object verification is listed in section 6.

**B. Targeted revert/reapply.** On the empty database, `down.sql` ran in
2.5 ms/statement or less (six statements) and removed all three tables and both
enums while leaving all eight MET-WP1-01/02/03 tables, `metric_reporting_grain`
and both measure seed rows intact. `up.sql` then restored 3 tables, 2 enums,
12 FKs, 7 CHECKs and 14 indexes. The same cycle is additionally covered through
the Diesel harness by a repository test.

**C. Representative populated database.** Built a second disposable database
carrying representative existing state: 2 publishers, 2 imprints, 3 works
(active and forthcoming, one with a DOI), 3 publications (PDF, Paperback,
Epub; one with an ISBN), 2 ROR-backed institutions; Publisher Services state
(2 `publisher_distribution_platform` activations, 1 completed
`distribution_job`); MET-WP1-01 registry state (2 platforms, 1
platform-measure mapping, both seeded measures); MET-WP1-02 source state (1
source, 1 source account, 1 checkpoint with cursor); MET-WP1-03 import state
(1 import with counters and manifest, 2 import errors of both severities).

Evidence method: for each of the 16 pre-existing tables, an order-independent
MD5 over every row rendered as JSON, the row count, and `pg_class.relfilenode`
(which changes if and only if PostgreSQL rewrites the table).

Observed result across apply → revert → reapply:

```text
apply    : every one of the 15 populated tables byte-identical (row count, content MD5,
           relfilenode). The ONLY difference anywhere was __diesel_schema_migrations
           gaining its 13th row, as expected.
revert   : snapshot byte-identical to the pre-apply snapshot, including the ledger.
           MET-WP1-04 objects absent; metric_reporting_grain and all MET-WP1-01/02/03
           tables and seeds present; circular constraint count 0.
reapply  : again only the ledger row differs; 3 tables, 2 enums, 12 FKs, 7 CHECKs,
           14 indexes restored; 0 rows in the three new tables.
relfilenode: UNCHANGED for all 16 tables across the entire cycle.
```

Locking was observed by running the migration inside an open transaction and
reading `pg_locks` for the backend; the result is recorded in section 6.

## 11. CI

CI status: NOT AVAILABLE — no pull request exists and none is authorized.
Checks: none ran. A task-branch push does not trigger the build or migration
workflows under the current workflow inventory.
Failures or warnings: none observed. No automatic run was created by the push;
if one appears it must be reported to the control plane rather than acted on
here.

## 12. Rollout and rollback

Initial state after any later authorized merge to `feature/metrics`: the
schema exists and is completely inactive. No runtime path reads or writes
these tables.
Activation required: YES, by later WP2 ingestion, WP4 rollup/GraphQL and WP11
operational slices, each separately specified and authorized.
Feature flag/configuration: none; inactivity is structural, not flagged.
Migration sequence: this migration must run after `20260828_v1.9.0`
(MET-WP1-03), because `metric_record_revision` and `metric_record_provenance`
reference `metric_import`. The Diesel ledger enforces the order.
Rollback/disable procedure: before any later dependent Metrics migration
merges, rollback is a separately authorized revert of the bounded child
integration plus, in disposable or non-production environments only, the tested
down migration. Once later migrations or runtime contracts depend on this
schema, do not revert it in isolation: use dependency-aware reverse-order
rollback or a separately reviewed forward-repair plan. **No production
rollback is authorized by this task.**
Monitoring required: none while inactive.

## 13. Known limitations and deferred work

- Country storage enforces representation shape only. Semantic ISO 3166-1
  alpha-2 membership is not validated, so an unassigned code such as `AA` is
  storable. This is the approved WP2 boundary and is asserted by test.
- `identity_hash` and `content_hash` accept any non-blank text. No algorithm,
  encoding or length is fixed, by design; WP2 owns hashing.
- The database does not assert that an optional `publication_id` belongs to
  `work_id`, or that an optional `institution_id` carries a ROR. Those are
  design-required semantic resolutions performed by the later ingestion path.
- No relationship is enforced between `metric_record.current_revision_id` and
  revision `status`; a pointer at a `RETRACTED` revision is accepted at schema
  level. WP2 owns that transaction.
- No period-overlap detection or concurrency protocol exists; overlapping
  records are accepted.
- The `metric_record -> metric_record_revision (current_revision_id)`
  `joinable!` direction is omitted because Diesel cannot hold both directions
  of a pair, and the self-referential `supersedes` join is not expressible as a
  `joinable!`. Neither affects the enforced contract.
- Migration timings were measured on a local disposable database holding a
  handful of rows. Because the migration provably performs no table rewrite and
  acquires no `AccessExclusiveLock` on an existing table, the cost is expected
  to remain small at production scale, but that expectation is not production
  evidence and no production migration is authorized.

## 14. Unresolved issues

NONE.

## 15. Agent self-assessment

The implementing agent may identify risks but may not approve this task. This
report is implementation evidence, not self-approval.

Suggested review focus:

- The composite-foreign-key mechanism for both same-record invariants, and
  whether MATCH SIMPLE semantics are the intended behaviour for the nullable
  columns (a NULL `current_revision_id` or `supersedes_revision_id` leaves the
  constraint unenforced, which is what permits a record to exist before its
  first revision).
- The down migration's explicit drop of the circular constraint, and its
  ordering relative to the two dependent tables.
- The decision to use the repository-standard `diesel_manage_updated_at`
  trigger on `metric_record`, against the specification's "no triggers"
  language, which the implementation reads as scoped to the same-record
  integrity mechanism and the revision state machine.
- The `Nullable<Bpchar>` mapping for `CHAR(2)` in `schema.rs`, and whether the
  reviewer prefers the type-identical `Nullable<Text>`.
- The completeness and closure of the asserted CHECK, foreign-key and index
  inventories, since those tests are what prevent later speculative additions.
- The `docs/metrics/task-status.md` wording, specifically that `MET-WP1-04` is
  recorded as **not merged** and that WP1 remains `IN PROGRESS`.
