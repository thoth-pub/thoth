# MET-WP1-10 - implementation report

Task: `MET-WP1-10 - Establish OPERAS import ledger persistence foundation`
Owning issue: [#888](https://github.com/thoth-pub/thoth/issues/888)
Parent programme: [#766](https://github.com/thoth-pub/thoth/issues/766)
Repository: `thoth-pub/thoth`
Risk: **HIGH**
Workflow: `PROGRAMME_INTEGRATION`

## 1. Exact binding

```text
authorized base (feature/metrics):
f66048fcc3f4eacf8c5ce1ac9c07e9fa3179eb0f

incorporated develop:
4546cb632428872b961ad6c17282984d298e3ade

task branch:
feature/metrics--wp1-operas-import

PR target:
feature/metrics

migration identity:
thoth-api/migrations/20260906_v1.9.0/up.sql
thoth-api/migrations/20260906_v1.9.0/down.sql

implementation commit (10 source paths):
295fc68f320c17b69bfa4766f19df0a4f184b80d
tree   636aba9061bbad21546ad83e103247775290c6c8
parent f66048fcc3f4eacf8c5ce1ac9c07e9fa3179eb0f
```

This report is the eleventh authorized path and is committed as the
immediately following commit on the same branch, whose parent is
`295fc68f320c17b69bfa4766f19df0a4f184b80d`. A commit cannot contain its own
hash, so the **exact final head, tree and parent** of the branch are recorded
in the pull-request body and in the implementation handoff, and must be read
from the live branch at review time. Commits beyond the authorized base: **2**
(one implementation commit, one report commit). No commit was amended, rebased
or force-pushed.

`20260906_v1.9.0` is a task-specific, CTO-authorized future-dated migration
identity. It was explicitly authorized for MET-WP1-10 only and does not alter
the repository's general migration naming convention; the MET-WP1-09
`20260905_v1.9.0` exception was not treated as transitive authority. No
predecessor migration was reused, renamed or modified, and neither the
`Makefile` nor any package version was changed to manufacture the path.

### 1.1 Authorization provenance

| Record | Issue comment |
|---|---|
| Independent specification review (`APPROVED`) | [#888 comment 5543618181](https://github.com/thoth-pub/thoth/issues/888#issuecomment-5543618181) |
| CTO specification approval + bounded implementation authorization, including the future-date migration exception | [#888 comment 5543621843](https://github.com/thoth-pub/thoth/issues/888#issuecomment-5543621843) |
| Programme synchronization | [#766 comment 5543625277](https://github.com/thoth-pub/thoth/issues/766#issuecomment-5543625277) |

Approved design: **Thoth Metrics - Technical Design and Implementation Plan**,
Google Drive document `11AeQFGpm0kUZajBM5PrAqsttmzJlpUrt89tGYyVM8c0`, revision
`6`.

### 1.2 Preflight, verified before any source mutation

| Premise | Result |
|---|---|
| `origin/feature/metrics` exactly `f66048fcc3f4eacf8c5ce1ac9c07e9fa3179eb0f` | confirmed |
| `origin/develop` exactly `4546cb632428872b961ad6c17282984d298e3ade` | confirmed |
| `develop` incorporated into the authorized base | confirmed (`git merge-base --is-ancestor` succeeds) |
| #888 OPEN, comments `5543618181` and `5543621843` current | confirmed |
| `thoth-api/migrations/20260906_v1.9.0` absent | confirmed (highest existing was `20260905_v1.9.0`) |
| `feature/metrics--wp1-operas-import` absent locally and on the remote | confirmed |
| No existing MET-WP1-10 pull request | confirmed |
| Workspace clean | confirmed |
| Repository doctrine read | root `AGENTS.md`, `thoth-api/AGENTS.md`, ADR-0003 |

PR [#886](https://github.com/thoth-pub/thoth/pull/886) was independently
confirmed `MERGED` with merge commit
`f66048fcc3f4eacf8c5ce1ac9c07e9fa3179eb0f`, which is the authorized base.

## 2. Exact eleven-path inventory

```text
 1  CHANGELOG.md                                                  modified
 2  docs/metrics/task-status.md                                   modified
 3  docs/engineering/ai-delivery/implementation-reports/
        MET-WP1-10-implementation-report.md                       added
 4  thoth-api/migrations/20260906_v1.9.0/up.sql                   added
 5  thoth-api/migrations/20260906_v1.9.0/down.sql                 added
 6  thoth-api/src/schema.rs                                       modified
 7  thoth-api/src/model/mod.rs                                    modified
 8  thoth-api/src/model/metric_operas_import/mod.rs               added
 9  thoth-api/src/model/metric_operas_import/tests.rs             added
10  thoth-api/src/model/metric_operas_mapping/tests.rs            modified
11  thoth-api/src/model/metric_operas_export/tests.rs             modified
```

No file was deleted, moved or renamed. No twelfth path was required, so no
write-budget amendment was requested. Nothing outside this list was edited, and
no scratch or plan file was created inside the repository.

## 3. Schema delivered

```sql
CREATE TABLE public.metric_operas_import (
    remote_instance text NOT NULL,
    remote_event_id text NOT NULL,
    payload_hash text NOT NULL,
    import_id uuid,
    status text NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT metric_operas_import_pkey
        PRIMARY KEY (remote_instance, remote_event_id),
    CONSTRAINT metric_operas_import_import_id_fkey
        FOREIGN KEY (import_id) REFERENCES public.metric_import (import_id),
    CONSTRAINT metric_operas_import_remote_instance_check
        CHECK (remote_instance ~ '[^[:space:]]'),
    CONSTRAINT metric_operas_import_remote_event_id_check
        CHECK (remote_event_id ~ '[^[:space:]]'),
    CONSTRAINT metric_operas_import_payload_hash_check
        CHECK (payload_hash ~ '[^[:space:]]'),
    CONSTRAINT metric_operas_import_status_check
        CHECK (status ~ '[^[:space:]]')
);
```

Observed from the migrated database, in ordinal order:

| # | column | type | nullable | default |
|---|---|---|---|---|
| 1 | `remote_instance` | `text` | NO | - |
| 2 | `remote_event_id` | `text` | NO | - |
| 3 | `payload_hash` | `text` | NO | - |
| 4 | `import_id` | `uuid` | YES | - |
| 5 | `status` | `text` | NO | - |
| 6 | `created_at` | `timestamp with time zone` | NO | `CURRENT_TIMESTAMP` |

### 3.1 Reviewed decisions

**Remote identity.** The primary key is exactly
`(remote_instance, remote_event_id)`. The approved design names no surrogate
inbound-ledger ID and deliberately carries `remote_instance` alongside
`remote_event_id`, so a bare remote event identifier is not established as
globally unique. One remote event observed repeatedly resolves to the same
durable row rather than creating duplicate remote-event evidence, while the
same `remote_event_id` remains representable for two distinct remote instances.
No surrogate UUID column, no global uniqueness on `remote_event_id`, no
uniqueness on `payload_hash` and no additional identity column was added.

**Opaque required text.** `remote_instance`, `remote_event_id`, `payload_hash`
and `status` carry only the existing Metrics nonblank required-text CHECK
(`~ '[^[:space:]]'`). No URI, hostname, tenant/environment enum, registry,
normalization or case-folding rule constrains `remote_instance`; no syntax,
length, UUID/URI or global-uniqueness rule constrains `remote_event_id`; no
algorithm, encoding, case, length or uniqueness rule constrains `payload_hash`;
and `status` carries no PostgreSQL enum, no CHECK enumerating values, no
default, no trigger, no stored procedure, no transition graph and no
cross-column rule tying it to `import_id` or `payload_hash`. WP9 owns those
semantics.

**Payload-hash cardinality.** `payload_hash` is deliberately non-unique. Two
genuinely different remote events may legitimately carry equal payload content,
and forbidding that would make ordinary duplicate content unrepresentable. This
is proved by a dedicated test.

**Nullable, non-unique `import_id`.** `import_id` is `UUID NULL` with a
single-column non-cascading foreign key to `metric_import (import_id)`.

It is nullable because the approved design requires the remote event and its
payload hash to be recorded **before normalization**, and because an event that
is linked or skipped for loop prevention may never require a canonical import
job of its own; a durable remote-event row must therefore be able to exist
before any `metric_import` does. It is non-unique because one `metric_import`
may represent an API response or batch containing many distinct remote events,
so several inbound-ledger rows must be able to reference the same import. It is
non-cascading, matching every other Metrics foreign key, so deleting a canonical
import while durable remote-event evidence references it fails rather than
silently erasing that evidence.

This rationale deliberately does **not** rely on any claim that
`normalizer_version` is necessarily unknowable before normalization. That claim
was explicitly excluded from the durable review rationale and is not used here.

The database decides neither when `import_id` is populated nor whether a
particular `status` requires it.

**Timestamp.** `created_at` is required `TIMESTAMPTZ` with the
repository-standard `CURRENT_TIMESTAMP` default and is deliberately the only
timestamp. No remote-created-at, discovery, scan, snapshot, normalized-at,
updated-at or completion timestamp was added.

**No export-ledger relationship.** There is no foreign key or stored
relationship to `metric_operas_export`, and no duplicated export, platform,
measure or mapping identifier. The approved inbound shorthand contains none, and
loop prevention is WP9 runtime and reconciliation logic rather than a stored
relational export identity on the inbound row.

**Constraint naming.** The composite primary key, the foreign key and the four
CHECKs use the PostgreSQL default naming shape
`<table>_<columns>_<pkey|fkey|check>`, matching the merged MET-WP1-08 and
MET-WP1-09 keys.

## 4. Indexing reconciliation (section 14.4)

The MET-WP1-10 index inventory is exactly the composite primary-key index:

```text
metric_operas_import_pkey
  CREATE UNIQUE INDEX metric_operas_import_pkey
    ON public.metric_operas_import USING btree (remote_instance, remote_event_id)
```

No index on `status`, `created_at`, `import_id`, `payload_hash`, bare
`remote_event_id` or any scan/cursor field exists.

The approved design's generic requirement for import status and creation-time
indexing is **already satisfied** by the merged
`metric_import_status_created_at_idx` on `metric_import`, created by MET-WP1-03
migration `20260828_v1.9.0` and verified present both before and after this
migration. There is therefore **no outstanding WP1-10 OPERAS-import operational
index requirement**. PostgreSQL builds no index for the referencing side of a
foreign key; that is accepted at this inactive foundation stage rather than
pre-empted with a speculative index. WP9 may add operational indexes only from
actual query and query-plan evidence.

This reconciliation is recorded in the migration commentary, in the model module
documentation and here.

## 5. Section 15.5 inbound-completeness boundary

**Creating this ledger does not imply guaranteed inbound discovery.** This is
recorded in the `up.sql` commentary, in `metric_operas_import/mod.rs` and here.

- Guaranteed inbound completeness remains **externally blocked** without an
  adequate cursor or created-at event stream, replication, a complete
  snapshot/export, or an equivalent reliable incremental mechanism. Nothing in
  this slice removes that blocker.
- **No cursor field** was added.
- **No remote-created-at field** was added.
- **No scan or snapshot identifier** was added.
- **No rolling-scan or snapshot behaviour** was implemented.
- **No provider or API access** occurs anywhere in this slice.
- **WP9 owns** inbound discovery modes, loop prevention, reconciliation and
  completeness reporting, and must surface unverified completeness rather than
  claim it.

A populated `metric_operas_import` is evidence only of the remote events that
were actually observed and recorded — never evidence that all of them were. The
persistence table does not solve section 15.5 and must not be read as doing so.

## 6. Diesel contract (ADR-0003)

`thoth-api/src/schema.rs` is the repository-authoritative, manually maintained
contract. It was edited directly; no Diesel CLI, `diesel.toml`,
`diesel print-schema` or other schema-generation mechanism was introduced or
run. The addition is purely additive and follows live ordering conventions
(alphabetically between `metric_operas_export` and `metric_operas_mapping`):

```rust
table! {
    use diesel::sql_types::*;

    metric_operas_import (remote_instance, remote_event_id) {
        remote_instance -> Text,
        remote_event_id -> Text,
        payload_hash -> Text,
        import_id -> Nullable<Uuid>,
        status -> Text,
        created_at -> Timestamptz,
    }
}

joinable!(metric_operas_import -> metric_import (import_id));

allow_tables_to_appear_in_same_query!(… metric_operas_import …);
```

The `joinable!` declaration expresses the actual single-column relationship and
is correct for a nullable foreign key: the merged
`joinable!(metric_import -> publisher (publisher_id));` is the live precedent,
where `publisher_id` is likewise `Nullable<Uuid>`. Unlike MET-WP1-08, whose
composite foreign key could not be honestly expressed by a single-column
`joinable!`, this relationship genuinely is single-column, so no false
relationship is emitted and no schema machinery was broadened to force a macro.

The Rust model `MetricOperasImport` mirrors the six columns with
`import_id: Option<Uuid>` and derives `diesel::Queryable` under the `backend`
feature only.

## 7. Predecessor-test reconciliation

Both merged predecessor guards asserted that `metric_operas_import` was still
absent from a fully migrated database. Creating the authorized table
necessarily falsifies them. Both files were inside the authorized write budget,
and the failures were observed **before** editing them:

```text
metric_operas_export/tests.rs:1278
  assertion `left == right` failed:
    MET-WP1-09 must not create the deferred ledger table metric_operas_import
    left: 1   right: 0

metric_operas_mapping/tests.rs:943
  assertion `left == right` failed:
    MET-WP1-08 must not create the deferred ledger table metric_operas_import
    left: 1   right: 0
```

Applied changes, strictly bounded to the directly related deferred-ledger
assertion:

| File | Change |
|---|---|
| `metric_operas_mapping/tests.rs` | `DEFERRED_OPERAS_TABLES` `[&str; 3]` -> `[&str; 2]`; `metric_operas_import` removed; doc comment and the one in-test comment updated to say the inbound ledger is now MET-WP1-10-owned via `20260906_v1.9.0` |
| `metric_operas_export/tests.rs` | `DEFERRED_LEDGER_TABLES` `[&str; 3]` -> `[&str; 2]`; `metric_operas_import` removed; doc comment and the one in-test comment updated identically |

Both constants continue to assert that `metric_reconciliation_issue` and
`metric_reconciliation_run` remain absent. Every mapping schema, URI, foreign
key, uniqueness and index assertion, and every export schema, identity,
retry-deferral, foreign key, uniqueness and index assertion, is preserved
unchanged — the diffs touch only the constant, its doc comment and one in-test
comment in each file. No other predecessor source file failed, so no
`HOLD - WRITE BUDGET AMENDMENT REQUIRED` was raised.

The predecessor `mod.rs` documentation in both modules still states that those
slices do not themselves create `metric_operas_import`, which remains true and
required no edit (and both files are outside the write budget).

## 8. TDD evidence

A genuine test-first cycle was used. No assertion was fabricated to manufacture
a failure.

**RED 1 - missing Diesel schema contract.** `metric_operas_import/mod.rs` and
its 19 focused tests were written first, with no `schema.rs` entry and no
migration:

```text
error[E0432]: unresolved import `crate::schema::metric_operas_import`
  --> thoth-api/src/model/metric_operas_import/tests.rs:59:5
   |
59 | use crate::schema::metric_operas_import;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ no `metric_operas_import` in `schema`
error: could not compile `thoth-api` (lib test) due to 1 previous error
```

**RED 2 - missing relation.** The `schema.rs` contract was then added, with the
migration still absent. The suite compiled and every test failed at runtime for
the right reason:

```text
running 19 tests
DatabaseError(Unknown, "relation \"metric_operas_import\" does not exist")
test result: FAILED. 0 passed; 19 failed; 0 ignored; 1490 filtered out
```

**GREEN.** Migration `20260906_v1.9.0` was then written — only enough
persistence to satisfy the approved contract — and `thoth-api/src/db.rs` was
touched to force the compile-time `embed_migrations!` expansion to pick up the
new directory:

```text
running 19 tests
test result: ok. 19 passed; 0 failed; 0 ignored; 1490 filtered out; finished in 4.36s
```

## 9. Focused database tests

19 tests in `thoth-api/src/model/metric_operas_import/tests.rs`, all passing.

| Area | Tests |
|---|---|
| Exact schema | `metric_operas_import_has_exactly_the_approved_columns` asserts the six columns in order with type and nullability, and that the **only** default is `created_at = CURRENT_TIMESTAMP` — proving no surrogate identity default and no status default |
| Round trip | `a_complete_operas_import_row_round_trips_through_diesel` (also proves `created_at` receives the database default when omitted) |
| Composite identity | `the_same_remote_event_on_one_instance_is_rejected_as_a_duplicate` (unique violation, even with differing hash/status/import); `one_remote_event_id_is_accepted_under_two_different_remote_instances`; `two_remote_event_ids_are_accepted_under_one_remote_instance` |
| Opaque required text | `arbitrary_nonblank_text_round_trips_in_every_required_column` puts each of 8 values — including a URI, a non-URI, a single character, a padded value, Unicode and a hex string — into all four required columns and asserts byte-for-byte round trip, proving no parser, normalization, trimming or case-folding; `blank_and_whitespace_only_text_is_rejected_in_every_required_column` rejects 6 blank/whitespace variants × 4 columns = 24 cases via `CheckViolation` |
| Payload-hash cardinality | `two_different_remote_events_may_share_one_payload_hash` — three rows, one distinct `payload_hash` |
| Import linkage | `a_remote_event_is_accepted_before_any_canonical_import_exists` (NULL accepted, and no `metric_import` is auto-created); existing import accepted in the round-trip test; `a_nonexistent_import_is_rejected` (FK violation); `deleting_a_referenced_canonical_import_is_restricted_and_does_not_cascade`; `several_remote_events_may_reference_one_canonical_import`, which also asserts **no unique index covers `import_id`** |
| NOT NULL | `import_not_null_columns_are_enforced` for all four required columns plus an explicit `NULL created_at` |
| Metadata | `metric_operas_import_has_exactly_the_approved_checks` (exactly four CHECKs, each the required-text idiom and nothing stronger, none carrying a nullable escape); `metric_operas_import_has_exactly_the_authorized_non_cascading_foreign_key` (exactly one FK, no `ON DELETE`, referencing `metric_import(import_id)`); `metric_operas_import_has_exactly_the_required_indexes` (exactly the composite PK index) |
| Seeds | `migration_seeds_no_operas_import_row` |
| Runtime/discovery absence | `no_discovery_reconciliation_export_linkage_enum_or_trigger_was_introduced` asserts both reconciliation tables absent; 16 forbidden columns absent (`cursor`, `sync_cursor`, `remote_created_at`, `remote_updated_at`, `discovered_at`, `last_seen_at`, `normalized_at`, `updated_at`, `snapshot_id`, `scan_id`, `export_id`, `mapping_id`, `platform_id`, `uploader_uri`, `completeness`, `is_complete`); no OPERAS/reconciliation enum type; `status` still `text`; no non-internal trigger; no stored procedure |
| Migration | `reverting_through_the_operas_import_migration_removes_it_and_reapplication_restores_it` |

The exact six-column inventory plus the forbidden-column assertions together
prove no unauthorized runtime or discovery field was introduced.

Fixtures reuse the existing `pub(crate)` helpers `fixture_source_account`,
`insert_import_row` and `check_constraint_names` from `metric_import/tests.rs`
and the shared helpers from `metric_platform/tests.rs` and
`metric_record/tests.rs`. No other test module was widened.

## 10. Repository validation

All commands run from the repository root against disposable local PostgreSQL 17
and Redis. Nothing was pointed at production or staging.

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | clean |
| `cargo check --workspace` | exit 0 |
| `cargo clippy --all --all-targets --all-features -- -D warnings` | exit 0, no lint emitted |
| `cargo test -p thoth-api --features backend` | **1509 passed, 0 failed**; +13 `graphql_permissions`; 8 doc-tests ignored |
| `cargo test --workspace` | exit 0 — **1723 passed, 0 failed** (0/31/1509/13/3/4/11/144/0/0/0/6/0/2), 8 doc-tests ignored |
| `git diff --check` | clean |

The base carried 1490 `thoth-api` lib tests; this slice adds 19, giving 1509.

`cargo clippy` emits one cargo-level notice that
`proc-macro-error2 v2.0.1` "contains code that will be rejected by a future
version of Rust". That is a pre-existing third-party dependency warning, not a
lint against repository code, and it is present at the authorized base.

### 10.1 One flaky pre-existing test, characterised

The **first** `cargo test --workspace` run reported one failure:

```text
graphql::dataloader::failure_tests::
  conventional_backend_failure_matches_direct_graphql_semantics_without_retry_or_fallback
thoth-api/src/graphql/dataloader/failure_tests.rs:69
  left:  "Internal error: timed out waiting for connection"
  right: "Internal error: timed out waiting for connection: connection to
          server at \"localhost\" … port 1 failed: Connection refused …"
```

The test deliberately connects to port 1 and compares the dataloader error text
against the direct-GraphQL error text. Under heavy parallel load the r2d2 pool
sometimes returns the bare timeout before libpq's cause string is attached, so
the two sides differ in detail only.

Characterisation:

- it touches **no** Metrics table, model, migration or schema, and nothing in
  this slice affects the connection pool or dataloader;
- it **passed** in the full `cargo test -p thoth-api --features backend` run on
  this tree (1509/1509);
- it passed **6/6** consecutive isolated runs on this tree;
- a **second** full `cargo test --workspace` run on the same tree passed
  **1509/1509** with overall exit 0.

It is therefore a pre-existing load-dependent flake, not a regression caused by
this slice. The recorded workspace result above is the clean second run; the
first run's single failure is disclosed here rather than omitted.

## 11. Migration safety verification

All migration work used a disposable, non-production database
(`thoth_wp110_probe`) plus the disposable `thoth_test` used by the suite.

| Step | Result |
|---|---|
| Full chain applies cleanly | `thoth migrate` applied through `20260906`; ledger `…,20260904,20260905,20260906` |
| Table created empty | present, `COUNT(*) = 0` |
| Targeted revert through `20260906` | removed `metric_operas_import` only |
| Predecessor Metrics objects intact | all **16** predecessor Metrics tables present after revert |
| `metric_import_pkey` intact | present |
| `metric_import_status_created_at_idx` intact | present |
| Bibliographic schema intact | `work`, `publication`, `institution`, `publisher` present |
| MET-WP1-01 measure seeds intact | `title_sessions`, `net_units` present |
| Reapply restores the empty ledger | `COUNT(*) = 0`, 1 index, 1 foreign key, 4 CHECKs |
| `down.sql` scope | single `DROP TABLE IF EXISTS public.metric_operas_import`, **no `CASCADE`** |
| No table rewritten or backfilled | `up.sql` contains exactly one statement, a `CREATE TABLE` — no `ALTER`, `UPDATE`, `INSERT`, `COPY` or `TRUNCATE` |
| No row seeded | confirmed |
| Predecessor migrations unchanged | `git diff <base> -- thoth-api/migrations/` reports no change to any tracked migration file |

Canonical schema-DDL digests (columns + constraints + indexes + enum labels +
non-internal triggers, sorted):

```text
S1 applied     sha256 4104187e65394b6f24a58d54c9c66d7719bd2eb15e10ba2200e3c922c4728e4e  (3189 facts)
S2 reverted    sha256 d808d457ad28cf8b808123b97ef3bd05e822c77c651085b82cc732598f4e1578  (3176 facts)
S3 reapplied   sha256 4104187e65394b6f24a58d54c9c66d7719bd2eb15e10ba2200e3c922c4728e4e  (== S1)
```

The revert removed exactly **13** DDL facts and added **0**. All 13 belong to
`metric_operas_import`: its 6 columns, its 6 constraints (1 primary key, 1
foreign key, 4 CHECKs) and its 1 index. Reapplication is byte-identical to the
applied state.

Migration file digests:

```text
sha256 fc5247795762e6c86c0306c2adcd193e4976a7945a096d8c1fcfbb38a045179d  20260906_v1.9.0/up.sql
sha256 02f5eef1652ddfae5e9aff4a47ca5419591a67fbdafe0d193c22aecdf4fa855d  20260906_v1.9.0/down.sql
```

## 12. PostgreSQL locking evidence

Locks were **measured, not assumed**. The migration was replayed inside a
transaction held open with `pg_sleep`, `pg_locks` was read from a second
session, and representative concurrent operations were attempted from a third
with `SET lock_timeout = '2s'`.

Locks held on the referenced `metric_import`:

```text
metric_import  ShareRowExclusiveLock  granted
metric_import  AccessShareLock        granted
```

Measured effect of that lock:

| Concurrent operation on `metric_import` | Lock requested | Observed |
|---|---|---|
| `SELECT count(*)` | ACCESS SHARE | **not blocked** |
| `SELECT … FOR UPDATE` | ROW SHARE | **not blocked** |
| `DELETE … WHERE false` | ROW EXCLUSIVE | **BLOCKED** — `55P03 canceling statement due to lock timeout` |
| `UPDATE … WHERE false` | ROW EXCLUSIVE | **BLOCKED** — `55P03 canceling statement due to lock timeout` |
| `SELECT` on `metric_import_error` | ACCESS SHARE | not blocked |
| `DELETE … WHERE false` on `metric_operas_export` | ROW EXCLUSIVE | not blocked |
| `DELETE … WHERE false` on `work` | ROW EXCLUSIVE | not blocked |
| `SELECT` on `metric_operas_mapping` | ACCESS SHARE | not blocked |

**This is not a zero-blocking migration.** Creating the foreign key takes
`SHARE ROW EXCLUSIVE` on `metric_import`, which conflicts with `ROW EXCLUSIVE`,
so **concurrent inserts, updates and deletes on `metric_import` block for the
duration of the migration transaction**. Reads on `metric_import` — including
`SELECT … FOR UPDATE` — are unaffected, and no other table is affected at all.

The blocking window is the length of the transaction. Because the migration
only creates one empty table and validates no existing rows, that window is
short in principle, but no production timing is claimed here: these were
disposable-fixture measurements and are deliberately **not** extrapolated into a
production estimate. Production migration execution remains governed by CG-13
and separate release authorization.

## 13. GraphQL compatibility

The public SDL is generated by `thoth-client/build.rs` into
`thoth-client/assets/schema.graphql`, which is build-generated and gitignored —
so `git status` is not a valid check. Both revisions were built in the same
working tree (`cargo build --workspace`, which is required because
`cargo build -p thoth-client` alone does not enable `thoth-api`'s `backend`
feature) and the outputs compared exactly.

```text
base f66048fcc3f4eacf8c5ce1ac9c07e9fa3179eb0f
  bytes  178270
  sha256 091e11f293132fdec784de420e3addf251f5020ba7e387889b292a066be15d8e

head (implementation tree)
  bytes  178270
  sha256 091e11f293132fdec784de420e3addf251f5020ba7e387889b292a066be15d8e

cmp  : no differing byte
diff : no differences
```

Exposure search, both files:

```text
metric_operas_import : 0 occurrences
MetricOperasImport   : 0 occurrences
```

The SDL is byte-identical and this slice exposes nothing through GraphQL. No
`thoth-client` file and no other repository was changed.

## 14. Migration and data effects

- One new empty table; no existing table, row, column, enum, constraint or
  index altered.
- No seed, no backfill, no data migration, no destructive or lossy operation.
- Reversible: `down.sql` drops only `metric_operas_import`, without `CASCADE`.
- Deployment ordering: none required. The table is inactive and unreferenced by
  any runtime path, so forward migration and application rollout are
  independent.
- Idempotency/retry: `down.sql` uses `DROP TABLE IF EXISTS`; the forward
  migration is a plain `CREATE TABLE` run once by the embedded runner.

## 15. Authorization and security effects

None. No GraphQL query, mutation, type or input; no resolver; no change to
`src/policy.rs` or any model policy; no service role, capability, entitlement or
publisher-scope change. ADR-0001 remains the entitlement authority and no
Metrics-specific entitlement state was created. Nothing logs a token, secret,
credential, object URL, personal data or upstream response body — no runtime
code path exists at all.

## 16. Cross-repository, provider and runtime effects

- **Cross-repository: none.** The GraphQL contract is byte-identical, so no
  consumer (`thoth-app`, `thoth-client` consumers, Sphinx, dissemination
  clients, CMS/site contracts) requires any change or is affected. `thoth-sphinx`
  gains no contract it could consume; under ADR-0002 Sphinx remains stateless
  orchestration with no direct canonical database authority.
- **Provider/runtime: none.** No OPERAS network or API access, no provider read
  or write, no runtime inspection, no external call of any kind occurred or was
  implemented.
- **Production: untouched.** No production or staging credential was used, no
  production or staging migration was executed, and no deployment, release,
  activation or branch deletion occurred.

## 17. CI and image-publication expectations

Only natural PR-triggered CI is expected on the draft pull request. No workflow
was manually dispatched, rerun or cancelled, and no workflow file was changed.
Any staging-image publication remains classifier-controlled by existing
repository workflows and was neither requested nor influenced by this task.

## 18. Non-goals confirmed absent

No OPERAS network/API access; no provider or runtime access; no discovery
cursor; no rolling scan; no snapshot import; no guaranteed-completeness claim;
no remote polling; no normalization; no canonical ingestion behaviour; no
automatic creation or completion of `metric_import`; no `direct_collection`
eligibility; no configured-uploader matching; no export echo matching or
skipping; no loop-prevention behaviour; no payload-divergence behaviour; no
reconciliation run or issue table; no inbound status vocabulary or state
machine; no worker claims, leases or retries; no `FOR UPDATE SKIP LOCKED`; no
GraphQL; no authorization or service-role change; no entitlement change; no
Sphinx source change; no `thoth-app` change; no dashboard or widget; no
`thoth-client` change; no dissemination change; no generic queue/job
abstraction; no reuse of Publisher Services `distribution_job*` state; no
production or staging migration; no deployment, release or activation.

Canonical naming: all newly written text uses `Sphinx` / `thoth-pub/thoth-sphinx`.
No new `Sphynx` spelling was introduced, and pre-existing historical provenance
(for example the merged MET-WP1-08 changelog entry) was deliberately left
unrewritten.

## 19. Deviations and limitations

- **No deviation from the approved schema, index boundary or write budget.** All
  eleven paths and only those paths were touched; no twelfth path was needed.
- The migration identity `20260906_v1.9.0` is future-dated relative to the
  implementation date. This is the explicitly CTO-authorized, task-specific
  exception recorded in #888 comment `5543621843`, and applies to MET-WP1-10
  only.
- Two commits exist beyond the base rather than one, because a commit cannot
  contain its own hash: the implementation commit carries the ten source paths
  and the following commit carries this report. Neither was amended, rebased or
  force-pushed.
- One pre-existing load-dependent flaky test is disclosed in section 10.1, with
  the evidence that it is unrelated to this slice.
- Lock measurements come from a disposable fixture database and are deliberately
  not extrapolated to production timings.
- Section 5's completeness blocker is unresolved **by design**; it is recorded,
  not fixed, and remains WP9-owned.

## 20. Remaining gates

```text
DRAFT PR - FRESH INDEPENDENT EXACT-HEAD SOURCE REVIEW REQUIRED
```

Still required, none of which this agent may perform:

1. fresh independent source review at the exact final head;
2. CTO merge authorization;
3. merge of the draft PR into `feature/metrics`;
4. later `feature/metrics -> develop` integration under separate authorization;
5. release, staging/production migration execution, deployment and activation,
   governed by CG-13 and separate release authorization;
6. authorization of the next Metrics slice.

The implementing agent has not approved its own source, has not marked the pull
request ready, has not merged, and has not deployed, migrated, released or
activated anything.
