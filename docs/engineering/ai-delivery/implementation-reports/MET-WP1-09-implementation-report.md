# MET-WP1-09 - implementation report

Task: `MET-WP1-09 - Establish OPERAS export ledger persistence foundation`
Owning issue: [#884](https://github.com/thoth-pub/thoth/issues/884)
Parent programme: [#766](https://github.com/thoth-pub/thoth/issues/766)
Repository: `thoth-pub/thoth`
Risk: **HIGH**
Workflow: `PROGRAMME_INTEGRATION`

## 1. Exact binding

```text
implementation base (feature/metrics):
d4980e6fb3ff6a08acebb95c7cb87306750469f2

incorporated develop:
4546cb632428872b961ad6c17282984d298e3ade

task branch:
feature/metrics--wp1-operas-export

PR target:
feature/metrics

migration identity:
thoth-api/migrations/20260905_v1.9.0/up.sql
thoth-api/migrations/20260905_v1.9.0/down.sql

implementation commits authorized:
exactly 1
```

`20260905_v1.9.0` is a task-specific, CTO-authorized future-date migration
identity. The repository-conventional `20260903_v1.9.0` is MET-WP1-07-owned and
`20260904_v1.9.0` is MET-WP1-08-owned. This does not change the migration
naming convention, and no predecessor migration was reused or modified.

### 1.1 Authorization provenance

| Record | Issue comment |
|---|---|
| Independent specification review (`APPROVED`) | [#884 comment 5527746638](https://github.com/thoth-pub/thoth/issues/884#issuecomment-5527746638) |
| CTO specification approval (`APPROVED`) | [#884 comment 5527749656](https://github.com/thoth-pub/thoth/issues/884#issuecomment-5527749656) |
| Implementation authorization | [#884 comment 5529338532](https://github.com/thoth-pub/thoth/issues/884#issuecomment-5529338532) |
| Write-budget amendment (nine paths to ten) | [#884 comment 5539938966](https://github.com/thoth-pub/thoth/issues/884#issuecomment-5539938966) |

Approved design: **Thoth Metrics - Technical Design and Implementation Plan**,
Drive revision `6`.

Preflight verified before any source mutation: live `feature/metrics`, live
`develop` and the remote task branch all at the exact authorized SHAs above;
the task branch carrying zero commits beyond the base; a clean working tree;
#884 open; workspace version `1.8.0`; the `Makefile` migration target
unchanged (still deriving the `v1.9.0` suffix from the workspace version and
naming directories from the local calendar date); both bound migration paths
absent; and no competing MET-WP1-09 pull request.

## 2. Final exact ten-path inventory

```text
 1. CHANGELOG.md
 2. docs/metrics/task-status.md
 3. docs/engineering/ai-delivery/implementation-reports/MET-WP1-09-implementation-report.md
 4. thoth-api/migrations/20260905_v1.9.0/up.sql
 5. thoth-api/migrations/20260905_v1.9.0/down.sql
 6. thoth-api/src/schema.rs
 7. thoth-api/src/model/mod.rs
 8. thoth-api/src/model/metric_operas_export/mod.rs
 9. thoth-api/src/model/metric_operas_export/tests.rs
10. thoth-api/src/model/metric_operas_mapping/tests.rs
```

No deletion, move or rename occurred. No dependency, lockfile, workflow,
GraphQL, provider or runtime file was changed.

### 2.1 Why the tenth path was required

Implementation reached `HOLD - WRITE BUDGET AMENDMENT REQUIRED` on the original
nine-path budget. The merged MET-WP1-08 test module
`thoth-api/src/model/metric_operas_mapping/tests.rs` declared:

```rust
const DEFERRED_OPERAS_TABLES: [&str; 4] = [
    "metric_operas_export",
    "metric_operas_import",
    "metric_reconciliation_issue",
    "metric_reconciliation_run",
];
```

and its `no_operas_ledger_reconciliation_or_delivery_object_was_introduced`
test asserted a `pg_class` count of `0` for every entry, against a database
with the complete migration chain applied. MET-WP1-09 is specifically
authorized to create `metric_operas_export`, so that predecessor assertion
became false by design:

```text
thoth-api/src/model/metric_operas_mapping/tests.rs:935
assertion `left == right` failed:
  MET-WP1-08 must not create the deferred ledger table metric_operas_export
  left: 1
 right: 0
```

The failure was characterised against the exact base in a clean comparison
worktree with an isolated database rather than assumed pre-existing:

| Tree | Result for that test |
|---|---|
| base `d4980e6fb3ff6a08acebb95c7cb87306750469f2` | `ok. 1 passed; 0 failed` |
| MET-WP1-09 implementation tree | `FAILED` (`left: 1`, `right: 0`) |

No change confined to the nine authorized paths could satisfy the assertion,
because it lives in a file outside that budget. The alternatives — not creating
the table, renaming it, or weakening MET-WP1-09's own assertions — would each
have violated the approved schema or specification. The CTO therefore
authorized the tenth path in comment `5539938966` as a **predecessor-test
reconciliation only**.

The applied correction is exactly that and nothing more:

- `metric_operas_export` removed from `DEFERRED_OPERAS_TABLES`;
- the array narrowed from `[&str; 4]` to `[&str; 3]`;
- the constant doc comment updated to record that the outbound export ledger is
  now MET-WP1-09-owned and created by migration `20260905_v1.9.0`, while
  MET-WP1-08 itself still creates none of the listed tables;
- the test renamed
  `no_operas_ledger_reconciliation_or_delivery_object_was_introduced` to
  `no_deferred_ledger_reconciliation_or_delivery_object_was_introduced`, so the
  name no longer implies that no OPERAS ledger exists anywhere;
- the in-test comment updated to state which ledger moved and why.

`metric_operas_import`, `metric_reconciliation_issue` and
`metric_reconciliation_run` remain asserted absent. Every MET-WP1-08
mapping-schema, URI, foreign-key, uniqueness, index, deferred-mapping-column,
`direct_collection`, enum-absence and trigger-absence assertion is unchanged;
the file's diff is 15 insertions and 7 deletions confined to the constant, its
doc comment, the test name and one comment.

## 3. Implemented schema

```sql
CREATE TABLE public.metric_operas_export (
    export_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    record_revision_id uuid NOT NULL,
    mapping_id uuid NOT NULL,
    status text NOT NULL,
    attempt_count integer NOT NULL,
    remote_event_id text,
    request_hash text,
    last_error text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    completed_at timestamp with time zone,
    CONSTRAINT metric_operas_export_pkey PRIMARY KEY (export_id),
    CONSTRAINT metric_operas_export_record_revision_id_key
        UNIQUE (record_revision_id),
    CONSTRAINT metric_operas_export_record_revision_id_fkey
        FOREIGN KEY (record_revision_id)
        REFERENCES public.metric_record_revision (record_revision_id),
    CONSTRAINT metric_operas_export_mapping_id_fkey
        FOREIGN KEY (mapping_id)
        REFERENCES public.metric_operas_mapping (mapping_id),
    CONSTRAINT metric_operas_export_status_check
        CHECK (status ~ '[^[:space:]]'),
    CONSTRAINT metric_operas_export_attempt_count_check
        CHECK (attempt_count >= 0),
    CONSTRAINT metric_operas_export_remote_event_id_check
        CHECK (remote_event_id IS NULL OR remote_event_id ~ '[^[:space:]]'),
    CONSTRAINT metric_operas_export_request_hash_check
        CHECK (request_hash IS NULL OR request_hash ~ '[^[:space:]]')
);
```

Exactly the ten design-named section 6.14 fields. Exactly two database
defaults, both repository-standard: `uuid_generate_v4()` on `export_id` and
`CURRENT_TIMESTAMP` on `created_at`. `status`, `attempt_count`,
`remote_event_id`, `request_hash`, `last_error` and `completed_at` carry no
default. Exactly two indexes: the primary key and the
`UNIQUE(record_revision_id)` index. No seed row, no enum type, no trigger, no
stored procedure.

Down migration: a single non-cascading `DROP TABLE IF EXISTS
public.metric_operas_export;`. Nothing references the table and this slice
created no enum, trigger, sequence or standalone index, so both of its indexes
are dropped implicitly with the table.

### 3.1 Rust model

`thoth-api/src/model/metric_operas_export/mod.rs` defines:

```rust
pub struct MetricOperasExport {
    pub export_id: Uuid,
    pub record_revision_id: Uuid,
    pub mapping_id: Uuid,
    pub status: String,
    pub attempt_count: i32,
    pub remote_event_id: Option<String>,
    pub request_hash: Option<String>,
    pub last_error: Option<String>,
    pub created_at: Timestamp,
    pub completed_at: Option<Timestamp>,
}
```

with `#[cfg_attr(feature = "backend", derive(diesel::Queryable))]` and
`#[derive(Debug, Clone, PartialEq, Eq)]`, matching the merged
`MetricOperasMapping` and `MetricRollupDelta` conventions. The module is
registered in `thoth-api/src/model/mod.rs` as `pub mod metric_operas_export;`,
in alphabetical position. It introduces no GraphQL object, input or resolver,
no status enum or constants, no claim/lease/retry/backoff implementation, no
payload builder, no OPERAS client, no eligibility or capability lookup, no
Sphinx coupling and no dependency or Cargo feature change.

### 3.2 Diesel schema contract

`thoth-api/src/schema.rs` was edited directly and atomically with the
migration, per ADR-0003. No Diesel CLI generation was used.

```rust
table! {
    use diesel::sql_types::*;

    metric_operas_export (export_id) {
        export_id -> Uuid,
        record_revision_id -> Uuid,
        mapping_id -> Uuid,
        status -> Text,
        attempt_count -> Int4,
        remote_event_id -> Nullable<Text>,
        request_hash -> Nullable<Text>,
        last_error -> Nullable<Text>,
        created_at -> Timestamptz,
        completed_at -> Nullable<Timestamptz>,
    }
}
```

Unlike the MET-WP1-07 and MET-WP1-08 composite foreign keys, both MET-WP1-09
relationships are ordinary single-column foreign keys, so both take the
repository-conventional `joinable!` declaration required by the independent
review, the CTO specification approval and the implementation authorization:

```rust
joinable!(metric_operas_export -> metric_operas_mapping (mapping_id));
joinable!(metric_operas_export -> metric_record_revision (record_revision_id));
```

`metric_operas_export` was also added to
`allow_tables_to_appear_in_same_query!`. All three insertions preserve the
file's existing alphabetical ordering and formatting.

## 4. Reviewed relational decisions

### 4.1 One export row per canonical revision

`UNIQUE(record_revision_id)` permits at most one durable export row per
canonical revision. Outbound eligibility is defined in singular terms — the
revision must not already have been exported — and section 15.3 describes
creating *an* export row and then retrying or claiming that durable row rather
than creating a new row per attempt. The merged MET-WP1-08 registry already
permits at most one canonical mapping per platform/measure pair at a time, so
permitting a second export row for the same revision merely because a mapping
row changed would make duplicate-delivery prevention ambiguous.

This uniqueness defines no success/failure transition, no retry eligibility and
no reaction to later mapping administration. Those remain WP9 runtime
questions.

### 4.2 Mapping-to-revision correspondence deferral

A valid export row must ultimately use the mapping for the canonical revision's
own platform/measure pair. The design-shaped export row does not duplicate
`record_id`, `platform_id` or `measure_id`, so the database cannot express that
cross-table rule with a simple foreign key.

MET-WP1-09 deliberately adds **neither** those redundant columns **nor** a
trigger. The later export-enqueue/eligibility path must select and validate the
mapping from the revision's canonical record and fail closed if it does not
correspond. Because this slice exposes no runtime write path, an arbitrary pair
of individually valid foreign-key identifiers stays representable at raw
database level. That is the approved WP1/WP9 boundary, confirmed by the
independent specification review, and the test
`mapping_to_revision_correspondence_is_not_enforced_by_the_database` records it
explicitly rather than leaving it implicit — including asserting that no
trigger exists on the table.

### 4.3 Retry-time inconsistency and WP9 deferral

The approved design is internally inconsistent on this point:

- section 6.14 defines the export ledger as exactly the ten implemented fields
  and contains **no** retry-time or next-attempt timestamp column;
- section 14.4 nevertheless calls for OPERAS export indexes on status **and
  retry time**;
- the design fixes no final export status vocabulary, claim ownership, lease
  column, retry schedule, backoff or stale-claim recovery protocol.

The independent specification review and the CTO specification approval both
decided this is a **safe deliberate deferral to WP9**. MET-WP1-09 therefore
persists only the fields section 6.14 names. It invents no `retry_at`,
`next_attempt_at`, `retry_after`, `next_retry`, `lease_until`, `claim_until` or
equivalent field, and creates no status, retry, `(status, created_at)`, mapping
or claim index. WP9 owns resolving the missing retry-time representation,
defining the actual claim query and protocol, and only then adding the concrete
operational index with query-plan evidence.

As required by the review and the authorization, the boundary is recorded in
the migration `up.sql` itself under the heading `RETRY-TIME INCONSISTENCY AND
ITS DELIBERATE WP9 DEFERRAL (reviewed)`, in the model module documentation, and
here. The test
`no_retry_claim_status_enum_or_reconciliation_object_was_introduced` asserts the
absence of twelve retry/claim/lease-shaped column names, of the three deferred
ledger tables, of any duplicated `record_id`/`platform_id`/`measure_id` column,
and of any OPERAS or reconciliation enum type.

## 5. Test-driven development evidence

Production SQL, schema and model were **not** written first. The complete
red/green/refactor cycle was performed in the working tree; only one final
implementation commit was created, as authorized.

### 5.1 RED-1 — model and schema absent

The focused test module was written first, against a stub
`metric_operas_export/mod.rs` that declared only the test submodule.
`cargo test -p thoth-api --features backend --no-run`:

```text
error[E0432]: unresolved import `super::MetricOperasExport`
  --> thoth-api/src/model/metric_operas_export/tests.rs:44:5
   | no `MetricOperasExport` in `model::metric_operas_export`

error[E0432]: unresolved import `crate::schema::metric_operas_export`
  --> thoth-api/src/model/metric_operas_export/tests.rs:56:5
   | no `metric_operas_export` in `schema`

error: could not compile `thoth-api` (lib test) due to 2 previous errors
```

### 5.2 RED-2 — table absent

The model struct and the `schema.rs` entry were then added, with **no**
migration. The tests compiled and failed behaviourally:

```text
running 24 tests
test result: FAILED. 0 passed; 24 failed; 0 ignored; 0 measured; 1466 filtered out

thread '...::migration_seeds_no_operas_export_row' panicked at
  Failed to run scalar query:
  DatabaseError(Unknown, "relation \"metric_operas_export\" does not exist")
```

All 24 new tests failed, each ultimately because the table did not exist.

### 5.3 GREEN — migration applied

The bound migration was then written. Re-running the same 24 tests unchanged:

```text
running 24 tests
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 1466 filtered out
```

### 5.4 Test inventory

The 24 tests in `thoth-api/src/model/metric_operas_export/tests.rs` prove:

**Identity** — `export_id_is_generated_when_omitted_and_honoured_when_supplied`.

**Valid rows** — complete row round-trip through Diesel including a populated
`completed_at`; `attempt_count = 0` and a positive `attempt_count`; arbitrary
nonblank `status` text; all four nullable columns null; populated
`remote_event_id`, `request_hash` and `last_error` round-tripping untrimmed and
unnormalized.

**Integrity failures** — negative `attempt_count` rejected (including
`i32::MIN`); blank and whitespace-only `status` rejected across six variants;
blank and whitespace-only non-null `remote_event_id` and `request_hash`
rejected while NULL stays valid; nonexistent `record_revision_id` rejected;
nonexistent `mapping_id` rejected; duplicate `record_revision_id` rejected;
`NOT NULL` enforced on all six required columns; deleting a referenced
`metric_record_revision` restricted; deleting a referenced
`metric_operas_mapping` restricted.

**Exact metadata** — the ten columns with types and nullability in order; the
two defaults and the absence of any other; exactly four CHECK constraints with
their definitions verified; exactly two non-cascading foreign keys (asserted by
the absence of `ON DELETE` in each definition); exactly two indexes; no seed
row; no deferred retry/claim/lease column; no duplicated dimension column; no
OPERAS or reconciliation enum; `status` still plain `text`; no trigger; and the
three deferred ledger tables still absent.

**Migration behaviour** —
`reverting_through_the_operas_export_migration_removes_it_and_reapplication_restores_it`
performs an authentic targeted revert through the embedded Diesel migration
harness and a reapply, asserting predecessor survival on both sides.

No test asserts or implies that eligibility, finality, capability enforcement,
claiming, retry scheduling, payload generation, remote delivery, remote
idempotency, inbound synchronization or reconciliation is implemented. The
module documentation states this explicitly.

### 5.5 UUID-default evidence

Required by the independent review, the CTO approval and the implementation
authorization, and proven two ways.

Behaviourally, `export_id_is_generated_when_omitted_and_honoured_when_supplied`
inserts one row through raw SQL that omits `export_id` entirely and asserts the
stored value is not `Uuid::nil()`, then inserts a second row (against a second
canonical revision, because `UNIQUE(record_revision_id)` permits only one row
per revision) with an explicitly supplied `Uuid::new_v4()` and asserts it is
stored exactly as given and differs from the generated identity.

By metadata, `metric_operas_export_has_exactly_the_approved_columns` asserts
the complete default inventory is exactly:

```text
[("export_id", "uuid_generate_v4()"), ("created_at", "CURRENT_TIMESTAMP")]
```

which simultaneously proves the repository-standard UUID default is present and
that `status`, `attempt_count` and every nullable column have no default.

## 6. Migration safety validation

All database work used disposable local PostgreSQL only. Local server:
PostgreSQL **17.10 (Homebrew)**, matching the `postgres:17` image used by
`.github/workflows/run_migrations.yml`. No staging or production database was
accessed.

### 6.1 Empty database

| Step | Observation |
|---|---|
| Full chain apply | 18 migrations in ledger; 76 public tables; `metric_operas_export` present |
| Targeted revert of `20260905` | 17 in ledger; `20260905` absent; 75 tables; `metric_operas_export` absent; `metric_record_revision` and `metric_operas_mapping` both intact; 39 enum types |
| Targeted reapply | 18 in ledger; 76 tables; 2 indexes; 2 foreign keys; 4 CHECKs; 0 rows; still 39 enum types |

CI-parity full-chain sequence (`cargo run migrate --revert` then `cargo run
migrate`, as `run_migrations.yml` performs): ledger 18 → 0 → 18. After
revert-all only one public table remained (`__diesel_schema_migrations`), and
after re-apply `metric_operas_export` was present again.

### 6.2 Representative populated database

A separate disposable database was built to the predecessor state and populated
with ordinary bibliographic data plus MET-WP1-01 through MET-WP1-08 Metrics
state:

```text
publisher=2 work=2 publication=2 institution=1
platform=2 measure=2 platform_measure=4
source_account=1 checkpoint=1 import=1 import_error=1
record=2 revision=2 provenance=1 coverage=1
approval=1 rollup_delta=1 operas_mapping=1
```

Four snapshots were taken — S0 predecessor, S1 after apply, S2 after revert, S3
after reapply — each capturing the `pg_dump --schema-only` DDL, the migration
ledger, the table list, the enum list, the index list, and a per-table content
digest (`md5(string_agg(row::text ORDER BY row::text))`) for every ordinary
public table.

`pg_dump` 17 emits a random `\restrict` / `\unrestrict` nonce line per run;
those two lines are excluded from the DDL digests below, which are otherwise
byte-for-byte comparisons of the dump.

| Comparison | Result |
|---|---|
| S0 → S1 ledger | `+20260905` only |
| S0 → S1 tables | `+metric_operas_export` only |
| S0 → S1 enums | **identical** — no enum type added |
| S0 → S1 indexes | `+metric_operas_export_pkey`, `+metric_operas_export_record_revision_id_key` only |
| S0 → S1 content digests | every predecessor table digest **unchanged**; single new entry `metric_operas_export=EMPTY` |
| S0 → S1 DDL | 28 changed lines, all inside the `metric_operas_export` `CREATE TABLE` and its four `ALTER TABLE ... ADD CONSTRAINT` statements |
| S0 vs S2 (after revert) | ledger, tables, enums, indexes, content and DDL all **identical** |
| S1 vs S3 (after reapply) | ledger, tables, enums, indexes, content and DDL all **identical** |

Schema DDL digests:

```text
S0 predecessor  sha256 0c7f9b63c95fb1fcd1fd94d88fcca22dcf60afbd651458f50f91a0a6b03cbfe0
S1 applied      sha256 5b279a526d4de84a6c26a9ca8ad5c3e7b7d82040af574e069f5152cbde0b8dde
S2 reverted     sha256 0c7f9b63c95fb1fcd1fd94d88fcca22dcf60afbd651458f50f91a0a6b03cbfe0  (== S0)
S3 reapplied    sha256 5b279a526d4de84a6c26a9ca8ad5c3e7b7d82040af574e069f5152cbde0b8dde  (== S1)
```

No populated existing table was rewritten or backfilled, no seed was added or
removed, and the down migration removed only MET-WP1-09-owned schema.

### 6.3 Predecessor migration byte identity

Every file in every predecessor migration directory was compared blob-by-blob
against the authorized base:

```text
files compared: 35   mismatches: 0

aggregate digest, base:         145f6d1889bc9e08aac8049dcb6ad269d501f7133a23dd1d389b7c1651d6dcf4
aggregate digest, working tree: 145f6d1889bc9e08aac8049dcb6ad269d501f7133a23dd1d389b7c1651d6dcf4
```

`git diff <base> -- thoth-api/migrations/` reports no change to any tracked
migration file. The new migration's own digests:

```text
sha256 b968f1accd948cc1d54ad97474e782c9c5fc7a3460965d12a1b8f02e299d679a  20260905_v1.9.0/up.sql
sha256 bb4094af5cf61f7d5939d1f6bc1eba6524e22d048976ef2aab36ad333f3deb09  20260905_v1.9.0/down.sql
```

## 7. PostgreSQL locking evidence

Locks were **measured**, not assumed. The migration was replayed inside a
transaction held open with `pg_sleep`, against the populated disposable
database, while `pg_locks` was read from a second session and representative
concurrent operations were attempted from a third with `SET lock_timeout =
'2s'`.

Lock modes actually held by the migration transaction:

| Relation | Modes held |
|---|---|
| `metric_record_revision` | `AccessShareLock`, **`ShareRowExclusiveLock`** |
| `metric_operas_mapping` | `AccessShareLock`, **`ShareRowExclusiveLock`** |
| `__diesel_schema_migrations` | `RowExclusiveLock` |

Concurrent behaviour observed while that transaction was held:

| Concurrent operation | Result |
|---|---|
| `SELECT` on `metric_record_revision` (ACCESS SHARE) | proceeded |
| `SELECT` on `metric_operas_mapping` (ACCESS SHARE) | proceeded |
| `SELECT ... FOR UPDATE` on `metric_record_revision` (ROW SHARE) | proceeded |
| `INSERT` into `metric_record_revision` (ROW EXCLUSIVE) | **blocked** (hit the 2s `lock_timeout`) |
| `UPDATE` on `metric_operas_mapping` (ROW EXCLUSIVE) | **blocked** (hit the 2s `lock_timeout`) |
| `SELECT` on `metric_record` (untouched table) | proceeded |
| `INSERT` into `metric_rollup_delta` (untouched table) | proceeded |
| `INSERT` into `work` (bibliographic table) | proceeded |

**This migration is therefore not write-free on its two referenced tables.**
Creating the foreign keys takes `SHARE ROW EXCLUSIVE` on
`metric_record_revision` and `metric_operas_mapping`, which conflicts with
`ROW EXCLUSIVE` and so blocks concurrent inserts, updates and deletes on both
for the duration of the migration transaction. Reads, including
`SELECT ... FOR UPDATE`, are unaffected, as is every other table.

Statement duration on the populated disposable database, five consecutive runs:

```text
15.721 ms, 12.873 ms, 12.787 ms, 13.682 ms, 15.628 ms
```

These are **disposable-database observations, not production estimates**. Both
referenced tables held only fixture-scale data, and no production deployment or
migration execution is authorized by this task. The practical exposure is
nevertheless bounded by table creation plus two foreign-key validations against
whatever those tables contain at the time, and no deployment decision should be
taken from these figures without its own separately authorized measurement.

## 8. GraphQL and API compatibility

The public GraphQL SDL is generated at build time by `thoth-client/build.rs`
into `thoth-client/assets/schema.graphql`, which is gitignored. A stale
checked-in file was therefore **not** used. The SDL was regenerated from a
clean detached worktree at the exact base and from the final implementation
tree, and compared:

```text
base d4980e6fb3ff6a08acebb95c7cb87306750469f2
  sha256 091e11f293132fdec784de420e3addf251f5020ba7e387889b292a066be15d8e   178270 bytes

final implementation tree
  sha256 091e11f293132fdec784de420e3addf251f5020ba7e387889b292a066be15d8e   178270 bytes

cmp: BYTE-IDENTICAL
```

A case-insensitive search for `metric_operas_export` / `MetricOperasExport` in
the generated SDL returns zero matches. No generated client update is required
and no downstream repository source change is required.

## 9. Repository validation

Run against the final tree. Environment: `THOTH_EXPORT_API` and
`TEST_DATABASE_URL` exported, local Redis running.

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | clean |
| `cargo check --workspace` | clean |
| `cargo clippy --all --all-targets --all-features -- -D warnings` | clean |
| `cargo test -p thoth-api --features backend` | **1490 passed, 0 failed**; plus 13 `graphql_permissions` passed; 8 doc-tests ignored |
| `cargo test --workspace` | **1696 passed, 0 failed, 0 ignored** across all targets; doc-tests 6 passed, 8 ignored |
| `git diff --check` | clean |

`cargo test --workspace` per-target counts: `thoth` lib 0, `thoth` bin 31,
`thoth-api` lib 1490, `graphql_permissions` 13, `thoth-api-server` 3,
`thoth-client` 4, `thoth-errors` 11, `thoth-export-server` 144.

The single failure that triggered the write-budget amendment
(`metric_operas_mapping::tests::no_operas_ledger_reconciliation_or_delivery_object_was_introduced`)
is resolved by the authorized tenth-path reconciliation and the suite is fully
green. No failure was labelled pre-existing: the one failure encountered was
reproduced against the exact base in a clean comparison worktree and shown to
pass there, establishing that it was caused by this slice.

`cargo` emits one unrelated dependency notice —
`proc-macro-error2 v2.0.1` contains code that a future Rust version will reject
— on both the base and this tree. It is a transitive-dependency
future-incompatibility note, not a warning from workspace code, and it does not
fail any gate.

## 10. Effects

### 10.1 Migration and data

Additive new export-ledger table only. No backfill, no seed, no data migration.
No mutation of registry, source, import, record, revision, provenance,
coverage, approval, rollup-delta or OPERAS-mapping state. The MET-WP1-01
`metric_measure` seed rows are untouched. The down migration removes only
task-owned schema. Deployment ordering is unconstrained beyond requiring the
merged MET-WP1-04 and MET-WP1-08 migrations to have been applied first, which
the chain already guarantees. The migration is idempotent in the ordinary
Diesel sense: it is recorded in the ledger under version `20260905` and is not
replayed.

No production migration is authorized, executed or implied by this task.

### 10.2 Authorization and security

No GraphQL, API or authorization change. No service-role change. No
package/capability model change — ADR-0001 remains the entitlement authority
and WP5 remains responsible for protected-operation capability enforcement;
`METRICS_OPERAS_EXPORT` is not evaluated anywhere in this slice. No Metrics
entitlement table is created. No credentials or secrets were accessed, stored
or changed. No policy file was touched.

### 10.3 Cross-repository

The changed contract is owned by `thoth-pub/thoth`. `thoth-pub/thoth-sphinx` is
a future consumer through protected Thoth API operations; this slice creates no
GraphQL contract, grants Sphinx no direct canonical database authority and
guesses no unmerged claim/status/retry contract. Under ADR-0002, Sphinx remains
stateless orchestration and Thoth remains the sole canonical Metrics datastore
and owner of the OPERAS synchronization ledgers. `MetricPlatform` remains
separate from `DistributionPlatform`; no name-based or enum-order mapping is
introduced.

`thoth-app`, `metrics-dashboard`, `metrics-widget`, `thoth-client`,
`thoth-dissemination` and current GraphQL/export consumers require no
repository-local source change. No downstream repository work is authorized.

### 10.4 Provider and runtime

No runtime producer or consumer exists after this slice. No Sphinx adapter or
worker behaviour, no external OPERAS request or write, no provider or runtime
configuration, no deployment, no release, no production migration and no
activation. No provider, staging or production state was accessed at any point.

## 11. Limitations and deviations

1. **Write-budget amendment.** The task was specified and authorized for nine
   paths and completed on ten. The tenth path was not discretionary: work
   stopped at `HOLD` and resumed only after the CTO recorded comment
   `5539938966`. Section 2.1 records the cause and the exact bounded scope.
2. **Locking is not write-free.** Concurrent writes to
   `metric_record_revision` and `metric_operas_mapping` block for the duration
   of the migration transaction, as measured in section 7. This is reported
   rather than minimised.
3. **Timings are disposable-database observations.** They were measured against
   fixture-scale tables on a local Homebrew PostgreSQL 17.10 instance and are
   not production estimates.
4. **`pg_dump` nonce lines excluded.** The two per-run `\restrict` /
   `\unrestrict` lines PostgreSQL 17 emits were removed before the DDL digests
   in section 6.2. Every other byte was compared.
5. **Correspondence is representable but not enforced.** As approved, a raw
   database insert can pair a revision with a mapping for a different
   platform/measure. No runtime path in this slice can create such a row, and
   the fail-closed enqueue validation is WP9-owned.
6. **Pre-existing `Sphynx` spelling retained outside rewritten text.** The
   tracker's MET-WP1-08 row already contained the historical `Sphynx` spelling.
   The MET-WP1-08 status text rewritten by this task and all MET-WP1-09 text
   use canonical `Sphinx`; the untouched remainder of that pre-existing row is
   left as-is, since correcting unrelated spelling elsewhere in the document is
   outside this task's authorized scope.
7. **Lifecycle evidence is deliberately not baked in.** The eventual
   implementation head SHA, pull-request number and CI run identifiers are not
   recorded in this committed report, because doing so would require a second
   source commit after PR creation and only one implementation commit is
   authorized. They remain GitHub lifecycle evidence on the pull request and
   issue.

## 12. Remaining gates

```text
MET-WP1-09
IMPLEMENTATION: COMPLETE
DRAFT PR: OPEN, TARGET feature/metrics
WP1: IN PROGRESS - NOT COMPLETE

NEXT: FRESH INDEPENDENT EXACT-HEAD SOURCE REVIEW
THEN: EXPLICIT CTO SHA-BOUND MERGE AUTHORIZATION

MERGE: NOT AUTHORIZED
READY-FOR-REVIEW TRANSITION: NOT AUTHORIZED
STAGING/PRODUCTION MIGRATION: NOT AUTHORIZED
DEPLOYMENT / RELEASE / ACTIVATION: NOT AUTHORIZED
feature/metrics -> develop INTEGRATION: NOT AUTHORIZED
TASK-BRANCH DELETION: NOT AUTHORIZED
NEXT METRICS SLICE: NOT AUTHORIZED
```

The implementing agent has not approved and may not approve this work.
