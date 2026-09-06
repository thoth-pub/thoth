# MET-WP1-11 - implementation report

Task: `MET-WP1-11` - establish reconciliation ledger persistence foundation
Owning issue: [#890](https://github.com/thoth-pub/thoth/issues/890)
Parent programme: [#766](https://github.com/thoth-pub/thoth/issues/766)
Repository: `thoth-pub/thoth`
Work package: WP1 - Metrics domain and database foundation
Risk: **HIGH**

## 1. Exact binding

| Item | Value |
|---|---|
| Authorized base | `feature/metrics @ 27e0811ada32b38ef43458c47db5c3c94e4b4680` |
| Incorporated `develop` checkpoint | `4546cb632428872b961ad6c17282984d298e3ade` |
| Task branch | `feature/metrics--wp1-reconciliation-ledger` |
| PR target | `feature/metrics` |
| Migration identity | `thoth-api/migrations/20260907_v1.9.0` |
| Migration-date status | task-specific future-dated exception, explicitly authorized on 2026-09-06, non-transitive |
| Implementation commit | `9a0f1f922139e21503f36f6058c18c5a767d70d5` |
| Write budget | 14 paths maximum |

The exact final head and tree are durably recorded in the pull request and the
fresh independent exact-head review, rather than inside a commit this report is
part of.

### 1.1 Authorization provenance

| Record | Location |
|---|---|
| Specification | #890 issue body |
| Specification Amendment 1 | #890 comment `5561650425` |
| Fresh independent amended-specification approval | #890 comment `5562163633` |
| CTO specification approval and implementation binding | #890 comment `5562208154` |

All three durable records were read in full before any mutation. Where the
original body and Amendment 1 conflict, the amendment controls: this
implementation follows Amendment A/B (`started_at` has no default), Amendment C
(the replacement `started_at` verification set), Amendment D (a closed PK-only
index inventory) and Amendment E (predecessor reconciliation-enum absence
assertions are preserved).

### 1.2 Preflight, verified before any source mutation

| Check | Result |
|---|---|
| live `feature/metrics` | `27e0811ada32b38ef43458c47db5c3c94e4b4680` — matches |
| live `develop` | `4546cb632428872b961ad6c17282984d298e3ade` — matches |
| `feature/metrics--wp1-reconciliation-ledger` | absent locally and on `origin` |
| PR using that head branch | none (`gh pr list --head … --state all` returned `[]`) |
| `thoth-api/migrations/20260907_v1.9.0` | absent on both the authorized base and the incorporated `develop` |
| working tree | clean, starting from local `feature/metrics--wp1-operas-import @ 1f8f0c60` |
| #890 durable records | Amendment 1, independent approval and CTO binding all present |

The branch was then created directly from
`27e0811ada32b38ef43458c47db5c3c94e4b4680`.

## 2. Exact thirteen-path inventory

Thirteen of the fourteen authorized paths were used; the fourteenth is this
report. No path outside the budget was created, modified, deleted, moved or
renamed, and no `HOLD - WRITE BUDGET AMENDMENT REQUIRED` condition arose.

```text
CHANGELOG.md                                          |    1 +
docs/metrics/task-status.md                           |  116 +-
thoth-api/migrations/20260907_v1.9.0/down.sql         |   24 +
thoth-api/migrations/20260907_v1.9.0/up.sql           |  210 ++
thoth-api/src/model/metric_operas_export/tests.rs     |   43 +-
thoth-api/src/model/metric_operas_import/tests.rs     |   36 +-
thoth-api/src/model/metric_operas_mapping/tests.rs    |   47 +-
thoth-api/src/model/metric_reconciliation_issue/mod.rs|  121 +
thoth-api/src/model/metric_reconciliation_issue/tests.rs | 1162 +
thoth-api/src/model/metric_reconciliation_run/mod.rs  |  106 +
thoth-api/src/model/metric_reconciliation_run/tests.rs| 1121 +
thoth-api/src/model/mod.rs                            |    2 +
thoth-api/src/schema.rs                               |   32 +
13 files changed, 2879 insertions(+), 142 deletions(-)
```

No Cargo manifest, lockfile, workflow, script, GraphQL file, existing migration,
other model module or unrelated document was touched.

## 3. Schema delivered

Migration `20260907_v1.9.0` creates the parent before the child and nothing
else. Metadata below is read back from a disposable PostgreSQL 17 database with
the complete migration chain applied.

### 3.1 `metric_reconciliation_run`

| # | Column | Type | Nullable | Default |
|---|---|---|---|---|
| 1 | `run_id` | `uuid` | NO | `uuid_generate_v4()` |
| 2 | `scope` | `jsonb` | NO | — |
| 3 | `status` | `text` | NO | — |
| 4 | `started_at` | `timestamp with time zone` | NO | — |
| 5 | `completed_at` | `timestamp with time zone` | YES | — |
| 6 | `summary` | `jsonb` | NO | `'{}'::jsonb` |

Constraints:

```text
metric_reconciliation_run_pkey          PRIMARY KEY (run_id)
metric_reconciliation_run_status_check  CHECK ((status ~ '[^[:space:]]'::text))
```

Foreign keys: **none**. The run is the parent of the audit unit and references
nothing; the approved run shorthand names no relationship and `scope` is
deliberately schemaless JSONB.

### 3.2 `metric_reconciliation_issue`

| # | Column | Type | Nullable | Default |
|---|---|---|---|---|
| 1 | `issue_id` | `uuid` | NO | `uuid_generate_v4()` |
| 2 | `run_id` | `uuid` | NO | — |
| 3 | `issue_type` | `text` | NO | — |
| 4 | `severity` | `text` | NO | — |
| 5 | `record_id` | `uuid` | YES | — |
| 6 | `remote_event_id` | `text` | YES | — |
| 7 | `details` | `jsonb` | NO | `'{}'::jsonb` |
| 8 | `resolved_at` | `timestamp with time zone` | YES | — |

Constraints:

```text
metric_reconciliation_issue_pkey                   PRIMARY KEY (issue_id)
metric_reconciliation_issue_run_id_fkey            FOREIGN KEY (run_id)
                                                     REFERENCES metric_reconciliation_run(run_id)
metric_reconciliation_issue_record_id_fkey         FOREIGN KEY (record_id)
                                                     REFERENCES metric_record(record_id)
metric_reconciliation_issue_issue_type_check       CHECK ((issue_type ~ '[^[:space:]]'::text))
metric_reconciliation_issue_severity_check         CHECK ((severity ~ '[^[:space:]]'::text))
metric_reconciliation_issue_remote_event_id_check  CHECK (((remote_event_id IS NULL)
                                                     OR (remote_event_id ~ '[^[:space:]]'::text)))
```

Neither foreign key carries `ON DELETE`, so both use PostgreSQL's default
restricting behaviour.

### 3.3 Index inventory — exact and closed

```text
metric_reconciliation_run    metric_reconciliation_run_pkey
    CREATE UNIQUE INDEX … ON public.metric_reconciliation_run USING btree (run_id)
metric_reconciliation_issue  metric_reconciliation_issue_pkey
    CREATE UNIQUE INDEX … ON public.metric_reconciliation_issue USING btree (issue_id)
```

Two indexes total, both primary keys. There is no index on `run_id`,
`record_id`, `remote_event_id`, `issue_type`, `severity`, `resolved_at`,
`scope`, `status`, `started_at`, `completed_at`, `summary` or `details`.

### 3.4 Absent objects

```text
triggers on either reconciliation table                              0
stored procedures matching '%reconciliation%'                        0
enum types matching 'metric_reconciliation%'                         0
seeded reconciliation rows after apply and after reapply             0
```

### 3.5 Reviewed decisions honoured

* **Identity.** Both tables use repository-standard UUID surrogate primary keys
  with the standard `uuid_generate_v4()` default, matching merged
  `metric_operas_export.export_id`. No natural-key or composite uniqueness is
  invented, and in particular there is no uniqueness over
  `(run_id, issue_type, record_id, remote_event_id)`.
* **`scope`.** Required JSONB, no default, no database-level schema, no required
  key and no completeness flag.
* **`status`, `issue_type`, `severity`.** Required TEXT with only the existing
  Metrics nonblank required-text CHECK. No enum, closed vocabulary, default,
  trigger or transition graph. The design's section 15.6 prose examples are
  treated as illustrative, not exhaustive, and its operational mention of
  high-severity alerts is not turned into a severity domain.
* **`started_at` (Amendment A/B).** Required TIMESTAMPTZ with **no** default.
  This deliberately departs from the repository-standard current-time
  `created_at` idiom used elsewhere in Metrics, because `started_at` is the
  actual reconciliation-execution start supplied by the writer. Verified below.
* **`completed_at`.** Nullable, no default, and no status/timestamp
  state-machine CHECK.
* **`summary`, `details`.** Required JSONB defaulting to `'{}'`, matching the
  merged `metric_import.manifest` and `metric_record_provenance.details` idiom,
  with no required keys and no interpretation of an empty object.
* **`run_id` relationship.** Required, single-column, non-cascading.
* **`record_id` relationship.** Nullable, single-column, non-cascading, to
  `metric_record (record_id)`. No key to `metric_record_revision`.
* **`remote_event_id`.** Nullable opaque TEXT, nonblank when supplied, with no
  foreign key and no uniqueness. No `remote_instance`, `operas_import_id`,
  `export_id`, `mapping_id` or `record_revision_id` column was added.
* **`resolved_at`.** Nullable, no default, no companion resolution-workflow
  column or state machine.

## 4. Index reconciliation (Amendment D)

Amendment D closed this decision, so the implementation exercised no discretion.
The migration creates the two primary-key indexes and nothing else, and the
tests assert that exact inventory by equality rather than by absence of specific
names. The supporting reasons recorded in the migration and in the tests are the
amendment's own: PostgreSQL does not require a child-side referencing index to
enforce these foreign keys during ordinary child `INSERT`/`UPDATE`; the closest
merged Metrics parent/child precedent, `metric_import_error (import_id)`,
deliberately carries no child-side FK index; the approved design's section 14.4
names no reconciliation index; and this slice is additive, empty and has no
reconciliation reader or writer, so no query plan exists to justify one. WP9 may
add reconciliation indexes only from an actual access pattern with query-plan
evidence.

## 5. Remote-event boundary and the merged inbound identity contract

`MET-WP1-10` established canonical remote identity as the composite
`(remote_instance, remote_event_id)` and deliberately established that a bare
`remote_event_id` is not globally unique. This slice therefore adds **no**
foreign key from `metric_reconciliation_issue.remote_event_id` to
`metric_operas_import` or `metric_operas_export`, and no global uniqueness on
that column. Two tests prove the consequence rather than asserting it:

* a remote event that no `metric_operas_import` row mentions stays
  representable, which a foreign key would forbid and which the design-named
  "unexpected remote record" category requires;
* one bare remote event ID observed on two distinct remote instances — genuinely
  seeded as two `metric_operas_import` rows sharing one `remote_event_id` —
  remains representable on two issues, with the test additionally asserting that
  no foreign key definition mentions `remote_event_id` and that no unique index
  covers it.

## 6. Section 15.5 completeness boundary

Persisting reconciliation runs and issues does not solve, weaken or narrow the
OPERAS inbound-completeness blocker, and nothing in this slice may be read as
doing so. Guaranteed inbound completeness remains externally blocked without an
adequate cursor/created-at event stream, replication, a complete snapshot or
export, or an equivalent reliable incremental mechanism. Accordingly the
migration adds no completeness flag, coverage assertion, cursor, scan or
snapshot identifier and determines no completeness; the run-table tests assert
the absence of `completeness`, `cursor`, `is_complete`, `scan_id`, `snapshot_id`
and `unverified` columns among others. WP9 retains ownership of completeness
reporting and must surface unverified completeness rather than claim it.

## 7. Diesel contract (ADR-0003)

`thoth-api/src/schema.rs` is maintained manually under ADR-0003. The additions
are:

```rust
table! { metric_reconciliation_issue (issue_id) {
    issue_id -> Uuid, run_id -> Uuid, issue_type -> Text, severity -> Text,
    record_id -> Nullable<Uuid>, remote_event_id -> Nullable<Text>,
    details -> Jsonb, resolved_at -> Nullable<Timestamptz>,
} }

table! { metric_reconciliation_run (run_id) {
    run_id -> Uuid, scope -> Jsonb, status -> Text, started_at -> Timestamptz,
    completed_at -> Nullable<Timestamptz>, summary -> Jsonb,
} }

joinable!(metric_reconciliation_issue -> metric_reconciliation_run (run_id));
joinable!(metric_reconciliation_issue -> metric_record (record_id));
```

plus both tables in `allow_tables_to_appear_in_same_query!`. Table, `joinable!`
and `allow_tables_…` entries are inserted in the file's existing lexicographic
positions. The nullable `record_id` relationship uses the current single-column
`joinable!` precedent unchanged; there is deliberately no relationship through
`remote_event_id`.

`thoth-api/src/model/mod.rs` registers `metric_reconciliation_issue` and
`metric_reconciliation_run` in the same lexicographic position. Both model
modules derive only `diesel::Queryable` behind the `backend` feature plus
`Debug, Clone, PartialEq, Eq`; no GraphQL derive, resolver, mutation, query,
input or output type is introduced. JSONB columns are represented as
`serde_json::Value`, matching merged `MetricRecordProvenance.details` and
`MetricImport.manifest`.

## 8. Predecessor-test reconciliation

The three merged predecessor modules each asserted that both reconciliation
tables were absent from a fully migrated database. Those assertions are the only
thing this slice makes false, and they were budgeted from the outset.

### 8.1 RED evidence — the assertions genuinely failed

With the reconciliation migration applied and the three predecessor test files
restored to their exact base content
(`27e0811ada32b38ef43458c47db5c3c94e4b4680`):

```text
test model::metric_operas_import::tests::no_discovery_reconciliation_export_linkage_enum_or_trigger_was_introduced ... FAILED
test model::metric_operas_mapping::tests::no_deferred_ledger_reconciliation_or_delivery_object_was_introduced ... FAILED
test model::metric_operas_export::tests::no_retry_claim_status_enum_or_reconciliation_object_was_introduced ... FAILED

metric_operas_import/tests.rs:945  MET-WP1-10 must not create the deferred reconciliation table
                                   metric_reconciliation_issue   left: 1  right: 0
metric_operas_mapping/tests.rs:943 MET-WP1-08 must not create the deferred ledger table
                                   metric_reconciliation_issue   left: 1  right: 0
metric_operas_export/tests.rs:1284 MET-WP1-09 must not create the deferred ledger table
                                   metric_reconciliation_issue   left: 1  right: 0

test result: FAILED. 0 passed; 3 failed
```

### 8.2 What was changed

In each of the three modules the deferred-table constant contained **only** the
two now-created reconciliation tables, so the now-empty constant and its loop
were removed and replaced by a comment recording that these ledgers are
MET-WP1-11-owned and created by migration `20260907_v1.9.0`, in the same shape
the predecessors already used when a later authorized slice landed:

| File | Removed | Net |
|---|---|---|
| `metric_operas_mapping/tests.rs` | `DEFERRED_OPERAS_TABLES` + loop | 47 lines changed |
| `metric_operas_export/tests.rs` | `DEFERRED_LEDGER_TABLES` + loop | 43 lines changed |
| `metric_operas_import/tests.rs` | `DEFERRED_RECONCILIATION_TABLES` + loop | 36 lines changed |

Total across the three files: 37 insertions, 89 deletions.

### 8.3 What was preserved (Amendment E)

Each module's assertion that **no PostgreSQL enum matching
`metric_reconciliation%` exists** is retained verbatim and still passes, because
this slice creates no reconciliation enum:

```text
metric_operas_mapping/tests.rs   1 occurrence of metric_reconciliation%
metric_operas_export/tests.rs    1 occurrence of metric_reconciliation%
metric_operas_import/tests.rs    1 occurrence of metric_reconciliation%
```

No other assertion in any of the three modules was removed or weakened. Every
mapping field/default/FK/uniqueness/index/migration test, every export
field/default/FK/uniqueness/retry-deferral/migration test and every inbound
identity, payload, nullable-import, completeness-deferral, index and migration
test is unchanged, as is each module's exact column, CHECK, foreign-key and
index inventory assertion — which is what continues to prove that those slices
create only their own table.

## 9. RED evidence for the `started_at` contract

The Amendment A/B decision is the single most load-bearing departure from
repository convention in this slice, so it was verified to fail when violated.
With `started_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL`
temporarily substituted into `up.sql` and the migration reapplied:

```text
test model::metric_reconciliation_run::tests::metric_reconciliation_run_has_exactly_the_approved_columns ... FAILED
  left:  [("run_id", "uuid_generate_v4()"), ("started_at", "CURRENT_TIMESTAMP"), ("summary", "'{}'::jsonb")]
  right: [("run_id", "uuid_generate_v4()"), ("summary", "'{}'::jsonb")]

test model::metric_reconciliation_run::tests::started_at_has_no_database_default_and_must_be_supplied_by_the_writer ... FAILED
  started_at must carry no database default: substituting the insertion time would
  overstate reconciliation execution-start semantics   left: 1  right: 0

test result: FAILED. 0 passed; 2 failed
```

`up.sql` was then restored and both tests pass. The tests assert all five
Amendment C requirements: no column default in the database metadata; an INSERT
omitting `started_at` is rejected (`NotNullViolation`); an explicitly supplied
value — `2026-03-04T05:06:07.891011Z`, deliberately not "now" — round-trips
exactly including sub-second precision; `completed_at = NULL` remains valid; and
no status/timestamp state-machine invariant exists.

## 10. Focused database tests

34 new focused tests, all passing:

| Module | Tests |
|---|---|
| `metric_reconciliation_run/tests.rs` | 15 |
| `metric_reconciliation_issue/tests.rs` | 19 |

Run-table coverage: exact column names/types/nullability and order; the exact
two-entry default inventory; UUID identity generation; arbitrary structured
`scope` round-trip across eight shapes including objects, arrays, scalars and
`null`; `scope` required with no default; arbitrary nonblank opaque `status`
round-trip across eight values including padded and Unicode text; blank and
whitespace-only `status` rejection across six variants; no reconciliation status
enum; `started_at` no-default metadata, omission failure and exact round-trip;
`completed_at` nullable with no default and no status invariant; `summary`
`'{}'` default plus arbitrary structured round-trip with no required keys; the
exact PK-only index inventory; the absence of any foreign key; and the absence
of enum, trigger, stored procedure and GraphQL surface.

Issue-table coverage: exact column names/types/nullability and order; the exact
two-entry default inventory; UUID identity generation with no inferred
deduplication identity; valid run FK accepted; nonexistent run FK rejected; NULL
run FK rejected; referenced run deletion restricted and non-cascading; valid
canonical `record_id` accepted; nonexistent `record_id` rejected; referenced
`metric_record` deletion restricted and non-cascading; `record_id = NULL`
accepted both omitted and explicit; recordless issues representable with no
canonical record created; arbitrary nonblank `issue_type` and `severity`
round-trip; blank and whitespace-only rejection for `issue_type`, `severity` and
non-null `remote_event_id`; `remote_event_id` nullable and unconstrained beyond
nonblank text; a remote event absent from `metric_operas_import` representable;
one bare remote event ID representable across two distinct remote instances with
no FK and no unique index covering it; `details` `'{}'` default plus arbitrary
structured round-trip; `resolved_at` nullable with no resolution workflow; the
exact three-entry CHECK inventory with the nullable escape only on
`remote_event_id`; the exact two-entry non-cascading foreign-key inventory; the
exact PK-only index inventory; and that no run or issue is created
automatically by writing predecessor Metrics evidence.

The GraphQL negative assertion is direct rather than inferred: the run module
generates the public SDL through `crate::graphql::create_schema().as_sdl()` and
asserts it contains no case-insensitive occurrence of "reconciliation".

## 11. Repository validation

Every command was run on the implementation tree against a local disposable
PostgreSQL 17 and Redis. No staging, production or provider database was
accessed.

| Command | Result |
|---|---|
| `cargo test -p thoth-api --features backend metric_reconciliation` (focused) | **pass** — 34 passed, 0 failed |
| focused + the three predecessor tests | **pass** — 37 passed, 0 failed |
| `cargo test -p thoth-api --features backend` | **pass** — 1543 passed, 0 failed (lib); 13 passed (`graphql_permissions`); 8 doc-tests ignored |
| `cargo test --workspace` | **pass** — 1543 + 31 + 13 + 3 + 4 + 11 + 144 + 6 + 2 = 1757 passed, 0 failed |
| `cargo check --workspace` | **pass** |
| `cargo clippy --all --all-targets --all-features -- -D warnings` | **pass** — no warnings |
| `cargo fmt --all -- --check` | **pass** (rustfmt applied before committing) |
| `git diff --check` | **pass** — no whitespace errors |

No test failed, and no failure had to be classified as flaky or unrelated. The
only warning emitted by any of these commands is the pre-existing
`proc-macro-error2 v2.0.1` future-incompatibility notice from a transitive
dependency, which is present at the authorized base and unrelated to this
change.

The base's lib suite contains 1509 tests; the implementation tree contains 1543,
which is exactly the 34 new focused tests and no other change to the suite size.

## 12. Migration safety verification

Performed on a purpose-created disposable PostgreSQL 17 database
(`thoth_wp1_11`, UTF8), populated with representative Metrics rows spanning
`publisher`/`imprint`/`work`/`publication`/`institution`, `metric_platform`,
`metric_platform_measure`, `metric_source`, `metric_source_account`,
`metric_import`, `metric_record`, `metric_record_revision`,
`metric_operas_mapping`, `metric_operas_export`, `metric_operas_import` (two
rows sharing one `remote_event_id` across two remote instances), plus two
reconciliation runs and three reconciliation issues.

1. **Full chain applies cleanly from scratch.** `thoth migrate` applied the
   complete chain to an empty database in ~5 s; the migration ledger's newest
   version is `20260907`. The binary was confirmed to embed the new directory
   (`strings target/debug/thoth | grep 20260907_v1.9.0`) after touching
   `thoth-api/src/db.rs`, because `embed_migrations!` expands at compile time
   and cargo does not track new migration directories on stable.
2. **Targeted revert.** Reverting exactly `20260907` — the newest applied
   migration, so Diesel's supported `revert_last_migration` targets precisely
   it — dropped both reconciliation tables and removed only the `20260907`
   ledger row, leaving `20260906` newest. The in-repo test
   `reverting_through_the_reconciliation_migration_removes_it_and_reapplication_restores_it`
   exercises this through the genuine `MigrationHarness` and additionally
   asserts that all seventeen MET-WP1-01..10 tables, the `metric_record` primary
   key, the bibliographic schema and the MET-WP1-01 measure seeds survive. The
   disposable-database replay used the identical single-transaction semantics
   (`BEGIN; \i down.sql; DELETE FROM __diesel_schema_migrations WHERE version =
   '20260907'; COMMIT;`) so that representative data could be inspected, which
   the in-repo test cannot do because its harness truncates tables.
3. **Down migration removes only the two WP1-11 tables.** Diffing
   `pg_dump --schema-only` before and after the revert yields exclusively the
   two `CREATE TABLE` bodies and their eight constraints:
   `metric_reconciliation_run_pkey`, `metric_reconciliation_run_status_check`,
   `metric_reconciliation_issue_pkey`,
   `metric_reconciliation_issue_run_id_fkey`,
   `metric_reconciliation_issue_record_id_fkey`,
   `metric_reconciliation_issue_issue_type_check`,
   `metric_reconciliation_issue_severity_check` and
   `metric_reconciliation_issue_remote_event_id_check`. No other object appears
   in the diff. `down.sql` drops the child before the parent and uses no
   `CASCADE`, so the referenced `metric_record (record_id)` primary key is never
   reachable.
4. **Representative data preserved.** Row counts across all fifteen populated
   predecessor tables are identical before the revert, after the revert and
   after reapply, and the content digest of every `metric_record`,
   `metric_record_revision`, `metric_operas_export`, `metric_operas_import`,
   `metric_operas_mapping`, `metric_import` and `metric_source_account` row is
   `f678e47a57d7319aaafe075818675ddd` at all three points. No existing row was
   modified, and no backfill or seed was performed.
5. **Reapply restores the exact schema.** After `thoth migrate` reapplied
   `20260907`, the normalised `pg_dump --schema-only` digest is
   `34c3563e989cf715cfe85ba09b21cd5c292fbce49933dea78d2f80e37857c219`, byte-for-byte
   identical to the pre-revert dump (`diff` empty). Normalisation strips only
   pg_dump's own `\restrict`/`\unrestrict` nonce lines, which carry a fresh
   random token per dump. Both reconciliation tables come back empty
   (`0/0` rows), which is correct for a drop-and-recreate.
6. **Constraint, default and index metadata.** Read back from the reapplied
   database and reproduced verbatim in section 3: six and eight columns in the
   approved order and nullability, two defaults per table, one and three CHECKs,
   zero and two foreign keys, two primary-key indexes in total, and zero
   triggers, stored procedures and enums.
7. **Predecessor migration identities unchanged.** No file under any existing
   `thoth-api/migrations/*` directory is in the diff; the only migration paths
   in the change set are the two new `20260907_v1.9.0` files.

## 13. PostgreSQL locking evidence

Measured, not assumed. This is **not** a zero-blocking migration, and no such
claim is made.

Method: the migration was replayed inside
`BEGIN; \i up.sql; SELECT pg_sleep(35); COMMIT;` in a backgrounded `psql`
session (backend pid 77596) against the disposable database containing a
representative `metric_record` row. `pg_locks` was read from a second session
and third-session probes ran with `SET lock_timeout = '2s'`.

Locks held on **existing** relations for the life of the migration transaction:

```text
metric_record   AccessShareLock         granted
metric_record   ShareRowExclusiveLock   granted
```

`metric_reconciliation_run` is created inside the same transaction, so its locks
are on a relation no other session can see; no other pre-existing relation is
locked at all.

Measured effect on `metric_record` while the transaction was open:

| Probe | Lock mode | Result |
|---|---|---|
| `SELECT count(*) FROM metric_record` | ACCESS SHARE | **not blocked** |
| `SELECT record_id FROM metric_record FOR UPDATE` | ROW SHARE | **not blocked** |
| `DELETE FROM metric_record WHERE false` | ROW EXCLUSIVE | **BLOCKED** — `55P03 canceling statement due to lock timeout` |
| `UPDATE metric_record SET identity_hash = identity_hash WHERE false` | ROW EXCLUSIVE | **BLOCKED** — `55P03` |

Unrelated tables were entirely unaffected: probes taking ROW EXCLUSIVE on
`metric_import`, `metric_operas_import` and `work` all completed immediately.

Interpretation and duration: `SHARE ROW EXCLUSIVE` conflicts with `ROW
EXCLUSIVE`, so **reads of `metric_record` continue normally while concurrent
inserts, updates and deletes on `metric_record` are blocked for the duration of
the migration transaction**. Adding the foreign key validates no rows because
the child table is created empty. With the artificial sleep removed, the two
`CREATE TABLE` statements measured 6.090 ms and 5.550 ms on this disposable
database, so the write-blocking window is the migration transaction's own
length — approximately 12 ms here. That figure comes from a small local database
and is indicative, not a production guarantee: on a production system the window
is however long the enclosing migration transaction runs, including any other
statement batched into it, and it can be extended arbitrarily if the migration
must itself wait behind a long-running transaction holding a conflicting lock on
`metric_record`. This is operational evidence for later deployment gating only;
no production migration is authorized or executed.

## 14. GraphQL compatibility

The public SDL is build-generated at `thoth-client/assets/schema.graphql` by
`thoth-client/build.rs` and is gitignored, so `git status` is not a valid check.
Both revisions were built and the artefacts compared directly. The base was
built in a dedicated `git worktree` checked out at the authorized base SHA.

| Item | Value |
|---|---|
| Base SHA | `27e0811ada32b38ef43458c47db5c3c94e4b4680` |
| Head | implementation tree at commit `9a0f1f922139e21503f36f6058c18c5a767d70d5` |
| Base SDL SHA-256 | `091e11f293132fdec784de420e3addf251f5020ba7e387889b292a066be15d8e` |
| Head SDL SHA-256 | `091e11f293132fdec784de420e3addf251f5020ba7e387889b292a066be15d8e` |
| Byte length | 178270 (both) |
| Byte-identical | **yes** (`cmp` reports no difference) |
| Occurrences of "reconciliation" in the SDL | 0 |

No generated client or downstream file changed. The worktree was removed after
the comparison.

## 15. Migration and data effects

* Migration source: additive schema only, two new tables.
* Production migration execution: **NOT AUTHORIZED and NOT PERFORMED**.
* Staging migration execution: **NOT AUTHORIZED and NOT PERFORMED**.
* Existing-data backfill: none.
* Existing canonical-data modification: none.
* Existing table, column, constraint, index, enum or trigger modification: none.
* Seed data: none.
* Real OPERAS instance, event identifier, reconciliation scope, status, issue
  type, severity or mapping approved or seeded: none. Every value in the tests
  and in the disposable-database fixtures is explicitly fixture text.

## 16. Authorization and security effects

* GraphQL/API behaviour: none. No derive, resolver, query, mutation, input or
  output type; `recordMetricReconciliation` is not implemented.
* Authorization, policy, service role or capability behaviour: none. ADR-0001
  remains the entitlement authority and no Metrics-specific entitlement state is
  created; WP5 remains responsible for protected-operation capability
  enforcement.
* Package or capability mapping: none.
* Provider, network or OPERAS access: none. No provider or runtime credential
  was read, and no staging or production system was contacted.
* ADR-0002 remains binding: `MetricPlatform` is not `DistributionPlatform` and
  no conversion between them is introduced. Sphinx remains stateless
  orchestration with no direct canonical database authority and gains no
  contract from this slice.

## 17. WP1/WP9 non-goal confirmation

None of the following is implemented, and no source in this slice implies any of
them already exists: reconciliation execution; comparison of source manifests,
canonical records, rollups or OPERAS ledgers; runtime scope decision; run or
issue creation at runtime; issue classification; closed status, issue-type or
severity vocabularies or transition rules; issue resolution or reopening
workflow; outbound/inbound OPERAS evidence matching; OPERAS loop prevention;
payload-divergence handling; snapshot or rolling-scan modes; completeness
determination or reporting; claims, leases, retries or backoff; scheduling or
worker behaviour; `recordMetricReconciliation` or any other protected API
operation; Sphinx changes; publisher, app, client or dissemination changes; real
OPERAS URI or mapping configuration; source, platform or measure seeds;
auth/service-role/capability enforcement; deployment or activation behaviour.

WP1 remains `IN PROGRESS`. This slice completes no work package and authorizes
no successor slice.

## 18. Deviations and limitations

1. **`started_at` departs from repository convention deliberately.** Every other
   Metrics timestamp of this kind (`created_at`) carries
   `DEFAULT CURRENT_TIMESTAMP`. Amendment A/B requires `started_at` to have no
   default, and the migration, model documentation and tests all record why, so
   a future reader does not "fix" it back into line with the surrounding
   convention. This is an authorized deviation from convention, not from
   specification.
2. **The disposable-database targeted revert used raw SQL rather than the CLI.**
   `thoth migrate --revert` reverts *all* migrations, so it cannot express a
   targeted revert. The in-repo test performs the genuine
   `MigrationHarness::revert_last_migration` sequence; the disposable-database
   replay reproduced Diesel's exact single-transaction semantics so that
   representative row preservation could be inspected, which the in-repo
   harness cannot show because it truncates tables. Both results are reported
   above and agree.
3. **The lock-duration figure is indicative.** The ~12 ms measurement comes from
   a small local database; the lock *modes* and the blocked/not-blocked
   classification are the durable findings, not the timing.
4. **RED evidence covers the two highest-risk assertion groups.** The
   predecessor deferral assertions and the `started_at` no-default contract were
   empirically shown to fail when violated. The remaining negative assertions
   (index inventory, foreign-key inventory, CHECK inventory, column inventory)
   are exact-equality comparisons against enumerated expected values rather than
   absence checks, so a violation necessarily changes the compared value; they
   were not additionally perturbed, to avoid unnecessary migration churn.
5. **The private revision-6 design document was not read directly.** This
   implementation followed the CTO-approved specification and Amendment 1, which
   the preceding independent reviews established against that document, together
   with the merged baseline source. No design claim in this report rests on an
   unread source.

No other deviation. No `HOLD`, `BLOCKED` or `STOP` condition arose.

## 19. Compliance with the authorization

| Requirement | Status |
|---|---|
| Base exactly `27e0811ada32b38ef43458c47db5c3c94e4b4680` | met |
| Branch exactly `feature/metrics--wp1-reconciliation-ledger` | met |
| Migration exactly `20260907_v1.9.0` | met |
| Within the 14-path write budget | met — 13 paths plus this report |
| No deletion, move or rename | met |
| Additive, empty, initially inactive migration | met |
| Local disposable PostgreSQL only | met |
| No staging/production/provider access | met |
| No manual CI dispatch, rerun or cancellation | met |
| No merge, deployment, release or activation | met |
| No issue comment or issue-body mutation | met |
| Implementation agent does not approve its own work | met |

## 20. Remaining gates

1. Fresh independent exact-head source review of the pushed head. Any further
   source commit invalidates a review already given.
2. Explicit CTO merge authorization bound to the exact reviewed head.
3. Merge into `feature/metrics` only, followed by merge-evidence verification.
4. Separate later authorization for any staging or production migration
   execution, deployment, release or activation.
5. `feature/metrics -> develop` integration remains unauthorized, as does any
   successor Metrics slice.
