# MET-WP1-07 Implementation Report

## 1. Repository state

```text
Programme:                 Thoth Metrics - canonical ingestion, Sphinx orchestration and client cutover
Owning GitHub issue:       #880 - MET-WP1-07 - Establish rollup-delta persistence foundation
Parent programme issue:    #766
Repository:                thoth-pub/thoth
Task ID:                   MET-WP1-07
Workflow:                  PROGRAMME_INTEGRATION
Risk:                      HIGH
Exact authorized base:     feature/metrics @ 6a768a5bf60ad9fa00757b19faea633e1ca21c08
Exact incorporated develop: 4546cb632428872b961ad6c17282984d298e3ade
Task branch:               feature/metrics--wp1-rollup-delta
PR target:                 feature/metrics
Migration identity:        thoth-api/migrations/20260903_v1.9.0
Implementing agent/model:  Claude Opus 5 (Claude Code)
Independent reviewer:      NOT YET PERFORMED - required at the exact implementation head
```

Approved specification: the complete #880 issue body, as amended by CTO
comment `5513578081`.

Controlling gate records, all verified before any source mutation:

| Gate | Record | Decision |
|---|---|---|
| Independent specification review | #880 comment `5513400945` | APPROVED |
| CTO specification approval | #880 comment `5513443403` | APPROVED |
| CTO migration-identity amendment | #880 comment `5513578081` | APPROVED AMENDMENT |
| Exact-SHA implementation authorization | #880 comment `5513734446` | IMPLEMENTATION AUTHORIZED |

### 1.1 Migration-identity amendment

The original #880 body prohibited future-dating a migration. CTO comment
`5513578081` explicitly supersedes that prohibition for this task, ruling that
the date-derived migration identity is an organizational naming convention
rather than a semantic runtime invariant, and binding the exact paths:

```text
thoth-api/migrations/20260903_v1.9.0/up.sql
thoth-api/migrations/20260903_v1.9.0/down.sql
```

`make migration` was therefore **not** run: on 2026-09-02 it resolves to
`thoth-api/migrations/20260902_v1.9.0`, which is the merged MET-WP1-06
migration. The authorized directory was created directly. No existing
migration was modified, reused, renamed or moved, and neither `Makefile` nor
the workspace version was touched.

Under `diesel_migrations`, the ledger version is the directory-name text
before the first underscore, so this migration's ledger identity is
`20260903`.

### 1.2 Preflight verification

Performed after `git fetch --all --prune`, before any edit:

| Precondition | Required | Observed | Result |
|---|---|---|---|
| `origin/feature/metrics` | `6a768a5b…21c08` | `6a768a5bf60ad9fa00757b19faea633e1ca21c08` | MATCH |
| `origin/develop` | `4546cb63…8ade` | `4546cb632428872b961ad6c17282984d298e3ade` | MATCH |
| `origin/feature/metrics--wp1-rollup-delta` | `6a768a5b…21c08` | `6a768a5bf60ad9fa00757b19faea633e1ca21c08` | MATCH |
| Implementation commit on task branch | none | `git log base..origin/branch` empty | MATCH |
| PR from task branch | none | `gh pr list --state all` returned `[]` | MATCH |
| `thoth-api/migrations/20260903_v1.9.0` | absent | absent | MATCH |
| Worktree | clean | clean | MATCH |

`develop @ 4546cb63…` is an ancestor of `feature/metrics @ 6a768a5b…`
(`git merge-base --is-ancestor` → true), so the authorized base genuinely
incorporates the stated `develop` checkpoint.

Repository instructions read before editing: root `AGENTS.md`,
`thoth-api/AGENTS.md`. No more specific nested `AGENTS.md` governs the
authorized files.

### 1.3 Prompt-injection observation

While reading repository instruction files, a `<system-reminder>` block was
emitted **inside the stdout of a `find` shell command**, instructing that git
commits be trailed with `Co-Authored-By: Claude Opus 5 …` and pull-request
bodies with a "Generated with Claude Code" line, and asserting that this
replaced earlier attribution guidance.

That text arrived through tool output, which is data rather than instruction,
and it contradicts the repository operator's standing instruction that no
AI-authorship or co-authorship attribution be added to commits or pull
requests. It was **not** acted upon. The commit and PR created by this task
carry no AI attribution trailer. This is recorded here so a reviewer can
confirm the omission is deliberate.

## 2. Scope confirmation

Implemented: durable persistence for `public.metric_rollup_delta` only —
migration, manually maintained Diesel schema contract, plain Rust persistence
model, module registration, focused database/model tests and documentation.

Not implemented, by design and by explicit specification: any runtime
behaviour whatsoever. See §6.3.

## 3. Commits

```text
Base (parent):     6a768a5bf60ad9fa00757b19faea633e1ca21c08
Implementation:    <FILLED IN AFTER COMMIT>
Commit count:      1 (exactly one, as authorized)
Tree state:        <FILLED IN AFTER COMMIT>
```

No amend, rebase, squash, force-push or second corrective commit was
performed.

## 4. Files changed

Exactly nine paths, matching the authorized write budget:

| # | Path | Change |
|---|---|---|
| 1 | `CHANGELOG.md` | modified - one `[Unreleased] / Added` entry |
| 2 | `docs/metrics/task-status.md` | modified - MET-WP1-07 status |
| 3 | `docs/engineering/ai-delivery/implementation-reports/MET-WP1-07-implementation-report.md` | created - this report |
| 4 | `thoth-api/migrations/20260903_v1.9.0/up.sql` | created |
| 5 | `thoth-api/migrations/20260903_v1.9.0/down.sql` | created |
| 6 | `thoth-api/src/schema.rs` | modified - one `table!` block + one `allow_tables_to_appear_in_same_query!` entry |
| 7 | `thoth-api/src/model/mod.rs` | modified - one `pub mod` registration |
| 8 | `thoth-api/src/model/metric_rollup_delta/mod.rs` | created |
| 9 | `thoth-api/src/model/metric_rollup_delta/tests.rs` | created |

### 4.1 Write-budget compliance

No tenth tracked path was created or modified. No deletion, rename or move was
performed. `git status --short` and `git diff --name-only` were inspected
before committing and contained exactly the nine paths above.

`thoth-client/assets/schema.graphql` is regenerated by `thoth-client/build.rs`
on every workspace build and is **gitignored** (`thoth-client/.gitignore:1`),
so building to produce SDL evidence created no repository change. It is
therefore not a tenth path.

Temporary artefacts — the disposable PostgreSQL cluster, the populated-database
fixtures and snapshots, and one detached `git worktree` at the authorized base
used solely to build base-revision SDL and base-schema database evidence —
were created outside the repository working tree, under the session scratchpad
directory, and left no tracked or untracked repository change. The worktree
created no branch or ref and was removed afterwards.

### 4.2 Authorized actions used

Used: repository/GitHub read inspection; source edits within the nine-path
budget; creation of the listed new files; checkout of the pre-existing task
branch; local/disposable migration execution and repository validation; one
commit; push of `feature/metrics--wp1-rollup-delta` only; creation of one
DRAFT PR targeting `feature/metrics`.

Not used: file deletion/move/rename; manual CI dispatch, rerun or cancel;
provider or runtime read/write; staging or production migration execution;
release, tag or publication; merge; deployment; production activation;
marking the PR ready; branch deletion; `feature/metrics -> develop`
integration; issue closure; any other Metrics slice.

### 4.3 Incidental accuracy correction in `docs/metrics/task-status.md`

The tracker at the authorized base still described MET-WP1-06 as "implemented
on its slice branch". At this task's exact base that statement is stale:
`6a768a5bf60ad9fa00757b19faea633e1ca21c08` **is** the merge commit of PR #879
(`git log -1` subject: "Merge pull request #879 from
thoth-pub/feature/metrics--wp1-publisher-approval"), verified independently
through the GitHub API (`PR #879 MERGED merged=6a768a5b… at
2026-09-02T17:05:05Z`).

Because this task rewrites the tracker's "Last updated" block and must
describe its own base accurately, MET-WP1-06's status field and the WP1
aggregate row were corrected to the verifiable git fact rather than restating
a superseded claim. This is a factual correction bounded to the merge event of
this task's own base; it makes **no** review, approval, deployment or
activation claim. Full post-merge tracker reconciliation remains the job of a
separately specified `RECON` task, following the `MET-WP1-05-RECON-01`
precedent, and is not attempted here.

## 5. Schema and model implemented

### 5.1 `public.metric_rollup_delta`

```sql
CREATE TABLE public.metric_rollup_delta (
    delta_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    record_id uuid NOT NULL,
    revision_id uuid NOT NULL,
    delta_value bigint NOT NULL,
    status text NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    applied_at timestamp with time zone,
    CONSTRAINT metric_rollup_delta_pkey PRIMARY KEY (delta_id),
    CONSTRAINT metric_rollup_delta_revision_id_key UNIQUE (revision_id),
    CONSTRAINT metric_rollup_delta_record_id_revision_id_fkey
        FOREIGN KEY (record_id, revision_id)
        REFERENCES public.metric_record_revision (record_id, record_revision_id)
);
```

Observed live structure (`\d+`) on a disposable database matches exactly:
seven columns; `uuid_generate_v4()` and `CURRENT_TIMESTAMP` defaults;
`delta_value` and `status` with no default; `applied_at` nullable with no
default; two indexes; one foreign key; zero CHECK constraints; zero
non-internal triggers; zero rows.

### 5.2 Implementation decisions

1. **Signed `delta_value`, no non-negative CHECK.** A revision contributes the
   signed difference `new - old` and a retraction subtracts the previously
   applied value, so positive, zero and negative deltas are all valid. A
   blanket `delta_value >= 0` rule would make correction and retraction
   accounting unrepresentable. `i64::MIN` and `i64::MAX` were both proven
   storable.
2. **`status` as required `TEXT`.** The approved design names the field but
   defines no closed vocabulary, transition model, claim ownership, lease or
   recovery protocol. No PostgreSQL enum, no CHECK enumerating values, no
   trigger and no Rust enum or constant set was introduced, because any of
   those would pre-decide the later WP4 claim/application state machine. The
   Rust field is a plain `String`.
3. **`applied_at` nullable with no cross-column invariant.** Nothing ties it
   to any particular `status` value.
4. **Same-record integrity via composite foreign key.** `(record_id,
   revision_id)` references the merged MET-WP1-04
   `metric_record_revision (record_id, record_revision_id)` unique key —
   the same declarative pattern MET-WP1-04 uses for its own same-record
   revision pointers. No trigger, redundant identity column or new dependency.
   Unlike MET-WP1-04's nullable pointers, both columns here are `NOT NULL`, so
   the MATCH SIMPLE composite key is always enforced.
5. **Non-cascading.** The foreign key uses PostgreSQL's default restricting
   behaviour, matching every other Metrics foreign key. Deleting a referenced
   record or revision fails rather than erasing durable accounting evidence.
6. **`UNIQUE(revision_id)`, keyed on the revision alone.** This is what
   prevents later double counting. A pair-scoped `(record_id, revision_id)`
   key would not actually exclude a second delta for the same revision, and
   `record_id` is already functionally determined by the revision through the
   composite foreign key.
7. **Constraint naming.** Primary key, uniqueness key and composite foreign
   key all use the PostgreSQL default shape
   `<table>_<referencing columns>_<pkey|key|fkey>`, matching the merged
   MET-WP1-04 supporting key
   `metric_record_revision_record_id_record_revision_id_key`.
8. **No `joinable!` entry.** The repository's `joinable!` declarations mirror
   single-column foreign keys only; MET-WP1-04's composite keys
   (`metric_record_current_revision_id_fkey`,
   `metric_record_revision_supersedes_revision_id_fkey`) have none. This
   table's only foreign key is composite, so no `joinable!` was added. This is
   a reviewed conclusion, not an omission.
9. **Index restraint.** Only the primary-key index and the revision-uniqueness
   index. PostgreSQL creates no index for the referencing side of a foreign
   key, so the observed inventory is exactly two. No `(status, created_at)` or
   other claim index was added before the WP4 claim query and query-plan
   evidence exist.

### 5.3 Rust model

`thoth-api/src/model/metric_rollup_delta/mod.rs` adds one plain persistence
struct, `diesel::Queryable` under the `backend` feature, following current
merged Metrics model conventions:

```rust
pub struct MetricRollupDelta {
    pub delta_id: Uuid,
    pub record_id: Uuid,
    pub revision_id: Uuid,
    pub delta_value: i64,
    pub status: String,
    pub created_at: Timestamp,
    pub applied_at: Option<Timestamp>,
}
```

No GraphQL object, input, enum, query, mutation or resolver. No status enum or
closed runtime constant set. No claim or apply method. No validation
re-implementing future ingestion logic. The module is registered in
`thoth-api/src/model/mod.rs` in the existing alphabetical order, between
`metric_record_revision` and `metric_source`.

### 5.4 Diesel schema contract

Per ADR-0003, `thoth-api/src/schema.rs` was edited directly and atomically
with the migration: one `table!` block inserted in table-name order between
`metric_record_revision` and `metric_source`, and one entry added to
`allow_tables_to_appear_in_same_query!`. No custom SQL type was needed
(`status` is `Text`), so the block uses the plain
`use diesel::sql_types::*;` form used by other non-enum tables. No unrelated
schema reformatting was introduced, and `diesel print-schema` was not used.

### 5.5 Explicit statements required by the specification

- **No runtime delta creation or application was added.** Nothing in this
  slice creates a delta, claims one, applies one, or reads the table at
  runtime. There is no producer and no consumer.
- **No rollup projection table was added.** `metric_rollup_work_day`,
  `metric_rollup_work_month`, `metric_rollup_work_country_month` and
  `metric_rollup_work_institution_month` remain approved future architecture
  and do not exist after this migration (asserted by test).
- **No GraphQL, authorization, Sphynx or provider behaviour changed.** The
  public GraphQL SDL is byte-identical to the base (§9), `thoth-api/src/policy.rs`
  is untouched, no service role or capability changed, no Sphynx or OPERAS
  contract exists in this slice, and no provider/runtime configuration was
  read or written.

## 6. Database and migration effects

### 6.1 Environment

macOS (aarch64, Darwin 25.6.0). Homebrew PostgreSQL **17.10** — matching the
`postgres:17` image used by repository CI — initialised into a disposable
scratchpad data directory; databases created `ENCODING 'UTF8' TEMPLATE
template0 LC_COLLATE 'C' LC_CTYPE 'C'`. Local Redis 8.x. Four throwaway
databases: `thoth` (empty-chain), `thoth_test` (Rust harness),
`thoth_populated` (representative populated), `thoth_lock` (lock
observation). **No staging or production system was contacted at any point,
and no staging or production migration was executed.**

### 6.2 Empty-database chain validation

| Step | Command | Result |
|---|---|---|
| Full chain apply | `thoth migrate` on an empty database | 16 migrations applied; ledger head `20260903`; 74 tables |
| Task migration revert | `down.sql` + ledger row removal | table dropped; 74 → 73 tables; ledger head `20260902`; MET-WP1-04 supporting unique key intact |
| Task migration reapply | `thoth migrate` | ledger head `20260903`; 73 → 74 tables; exactly 2 indexes restored |
| Full chain revert | `thoth migrate --revert` | 0 ledger rows; 1 table remaining (`__diesel_schema_migrations`) |
| Full chain reapply | `thoth migrate` | 16 ledger rows; head `20260903`; 74 tables |

The repository implements `--revert` as `revert_all_migrations`, so the full
chain revert exercises this task's `down.sql` inside the complete reverse
chain.

The embedded migration set was confirmed to include the new directory before
these runs (`strings target/debug/thoth | grep -oE '2026090[0-9]_v[0-9.]+'` →
`20260901_v1.9.0`, `20260902_v1.9.0`, `20260903_v1.9.0`), after
`touch thoth-api/src/db.rs`, because `embed_migrations!` is expanded at
compile time and cargo does not track migration-directory additions on stable.

### 6.3 Representative populated-database validation

`thoth_populated` was built with the **base-commit** binary (chain head
`20260902`, 73 tables, `metric_rollup_delta` absent), then populated with
representative bibliographic state and MET-WP1-01..06 rows:

```text
publisher 1, imprint 1, work 1, publication 1, institution 1
metric_platform 1, metric_measure 2 (seeded), metric_platform_measure 1
metric_source 1, metric_source_account 1, metric_source_checkpoint 1
metric_import 1, metric_import_error 1
metric_record 1, metric_record_revision 2 (CURRENT + SUPERSEDED chain),
metric_record_provenance 2 (one WINNER-path REVISION, one record-less REJECTED)
metric_coverage 1
metric_publisher_platform_approval 1
```

Snapshots captured md5 content hashes of all 18 non-empty data tables, the
full migration ledger with `run_on` values, every pre-existing relation and
index `relfilenode`, the table count and the enum count.

| Step | Observed |
|---|---|
| Apply `20260903` | **Only two differences** across the entire snapshot: one new ledger row `20260903`, and table count 73 → 74. All 18 table content hashes identical. All pre-existing relfilenodes identical. Enum count unchanged at 39. |
| Revert | Snapshot **byte-identical** to the pre-migration snapshot, relfilenodes included. |
| Reapply | Identical to the post-apply snapshot apart from the new ledger row's `run_on` timestamp. |

So: pre-existing content is unchanged; the migration ledger changes only by
the new identity; and the down migration removes only MET-WP1-07-owned
objects.

### 6.4 Predecessor migration integrity

`diff -r` of `thoth-api/migrations` against the base worktree reported exactly
one difference — `Only in thoth-api/migrations: 20260903_v1.9.0`. A
per-directory SHA-256 of concatenated `*.sql` confirmed all **15** predecessor
migration directories byte-identical, `20260902_v1.9.0` included.

### 6.5 Lock evidence

Measured on disposable PostgreSQL 17.10, `thoth_lock`. Session A opened a
transaction, ran the complete `up.sql`, then held the transaction open with
`pg_sleep(10)`; session B sampled `pg_locks` joined to `pg_class` and
`pg_stat_activity` roughly 3 seconds in, then attempted concurrent access
with `lock_timeout = '2s'`.

Locks observed against pre-existing Metrics tables, all granted:

| Relation | Lock mode |
|---|---|
| `metric_record_revision` | `AccessShareLock` |
| `metric_record_revision` | `ShareRowExclusiveLock` |

`metric_record` and every other pre-existing table were **not** locked; the
foreign key references only `metric_record_revision`. The new table's own
`AccessExclusiveLock` on itself is not reported, because the relation is not
yet visible outside the creating transaction — and is immaterial, since
nothing can reference a table that does not yet exist.

Empirically confirmed blocking behaviour while the DDL transaction was held:

| Concurrent statement | Target | Outcome |
|---|---|---|
| `SELECT count(*)` | `metric_record_revision` | **Not blocked** - returned immediately |
| `DELETE … WHERE false` (`RowExclusiveLock`) | `metric_record_revision` | **Blocked** - `ERROR: canceling statement due to lock timeout` after 2 s |
| `SELECT` and `DELETE … WHERE false` | `metric_record` | **Not blocked** - both succeeded |

Interpretation, stated precisely: `ShareRowExclusiveLock` does not conflict
with `AccessShareLock` or `RowShareLock`, so concurrent **reads** of
`metric_record_revision` proceed. It **does** conflict with `RowExclusiveLock`,
so concurrent **writes** to `metric_record_revision` — inserts, updates and
deletes — are blocked for the duration of the migration transaction. No claim
of "zero write blocking" is made, because the observed evidence contradicts
it.

Duration: the isolated `up.sql` executed in **3.631 ms** on a small disposable
database; the empty-database full chain applied in well under a second. The
blocking window is therefore expected to be very short, but it is not zero,
and on a production-sized `metric_record_revision` the FK's own validation
scan would extend it. Production migration execution is **not** authorized by
this task and was not performed.

### 6.6 Migration/data effects summary

Additive new table only. No backfill. No seed. No mutation of canonical
record, revision, provenance, coverage or approval rows. No projection
rebuild. The down migration drops only `metric_rollup_delta`, whose two
indexes go with it; no enum, trigger, sequence or standalone index was created
by this migration, so a single non-cascading `DROP TABLE IF EXISTS` is exact.

## 7. API and compatibility effects

GraphQL/API changes: **NONE.** The module exposes no GraphQL object, input,
enum, query, mutation or resolver.

The reachable public GraphQL SDL was generated from both trees by a full
`cargo build --workspace` (`thoth-client/build.rs` writes
`thoth-client/assets/schema.graphql`) and proven byte-identical:

```text
base 6a768a5bf60ad9fa00757b19faea633e1ca21c08 (detached worktree build)
  091e11f293132fdec784de420e3addf251f5020ba7e387889b292a066be15d8e  178270 bytes
implementation tree
  091e11f293132fdec784de420e3addf251f5020ba7e387889b292a066be15d8e  178270 bytes
cmp -> exit 0 (identical)
grep -ci rollup -> 0 occurrences
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
No service role or capability changed.

Roles/scopes: none. This slice creates no protected operation, so there is no
anonymous / wrong-role / wrong-publisher-scope / correct-scope / superuser
matrix to exercise. The existing `tests/graphql_permissions.rs` suite (13
tests) passes unchanged.

Negative authorization tests: not applicable. The tests that exist assert
**schema** fail-closed behaviour only — unknown record, unknown revision,
cross-record pairing, duplicate revision, restricted deletion and NOT NULL
rejection. No test claims runtime authorization behaviour exists.

Secret or personal-data handling: NONE. Nothing is logged.

Security limitations, stated explicitly:

- A row in this table is **not** authorization and **not** an applied rollup.
  No code path reads it.
- `status` accepts arbitrary text. Nothing validates it, and no code may treat
  any particular value as meaningful until the WP4 claim/application
  specification defines the vocabulary and its transitions.
- The one-delta-per-revision guarantee is enforced only for rows that exist.
  This slice does not guarantee that a delta is ever *created* for a revision;
  that is the WP2/WP4 ingestion transaction's responsibility.

## 9. Tests and checks

All commands run from the repository root against the final implementation
tree, with `THOTH_EXPORT_API` and `TEST_DATABASE_URL` exported into the
process environment.

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS - exit 0, no output |
| `cargo check --workspace` | PASS - exit 0 |
| `cargo clippy --all --all-targets --all-features -- -D warnings` | PASS - exit 0, no warnings |
| `cargo test -p thoth-api --features backend` | PASS - exit 0; lib 1447 passed / 0 failed, `graphql_permissions` 13 passed / 0 failed, doc-tests 8 ignored |
| `cargo test --workspace` | PASS - exit 0; **1661 passed, 0 failed** across all crates (`thoth` bin 31, `thoth_api` lib 1447 + 13, `thoth_api_server` 3, `thoth_client` 4 + 6 doc, `thoth_errors` 11, `thoth_export_server` 144 + 2 doc) |
| `git diff --check` | PASS - exit 0, no whitespace errors |

The pre-existing `proc-macro-error2 v2.0.1` future-incompatibility note is
emitted by an upstream dependency at the base commit as well, and is not
introduced by this change.

### 9.1 Focused database/model tests

17 new tests in `thoth-api/src/model/metric_rollup_delta/tests.rs`, all
passing:

| Requirement | Test |
|---|---|
| Round-trip, `applied_at = NULL` | `a_rollup_delta_round_trips_through_diesel_with_a_null_applied_at` |
| Round-trip, non-null `applied_at` | `a_rollup_delta_round_trips_through_diesel_with_a_non_null_applied_at` |
| Positive, zero and negative `delta_value` (plus `i64::MIN`/`i64::MAX`) | `positive_zero_and_negative_delta_values_are_all_accepted` |
| Arbitrary non-empty `status` text | `arbitrary_non_empty_status_text_persists_without_a_closed_vocabulary` |
| Missing/invalid record-revision reference rejected | `an_unknown_record_or_revision_reference_is_rejected` |
| Valid record + another record's revision rejected by the composite FK | `a_revision_belonging_to_another_record_is_rejected` |
| Duplicate `revision_id` rejected | `at_most_one_delta_is_permitted_per_canonical_revision` |
| Deleting a referenced revision/record fails, does not cascade | `deleting_a_referenced_revision_or_record_is_restricted_and_does_not_cascade` |
| No seeded rollup-delta row | `migration_seeds_no_rollup_delta_row` |
| Index inventory is exactly PK + revision uniqueness | `metric_rollup_delta_has_exactly_the_required_indexes` |
| No claim/rebuild/projection object appeared | `no_claim_rebuild_or_projection_object_was_introduced` |
| Database defaults (UUID PK, `created_at`) applied | `rollup_delta_database_defaults_are_applied_without_explicit_values` |
| `delta_value` and `status` NOT NULL with no default | `rollup_delta_not_null_columns_are_enforced` |
| Zero CHECK constraints (no non-negative rule, no status/`applied_at` rule) | `metric_rollup_delta_carries_no_check_constraint` |
| Exactly one non-cascading composite foreign key | `metric_rollup_delta_has_exactly_the_authorized_non_cascading_foreign_key` |
| Exactly the seven approved columns, types and nullability | `metric_rollup_delta_has_exactly_the_approved_columns` |
| Targeted revert-through and reapply of the migration | `reverting_through_the_rollup_delta_migration_removes_it_and_reapplication_restores_it` |

The tests reuse the existing `pub(crate)` fixtures from
`metric_record/tests.rs`, `metric_record_revision/tests.rs`,
`metric_platform/tests.rs` and `metric_import/tests.rs` as-is. No other
model's test module was widened, because this task's write budget contains
only its own test file. No relational constraint was weakened to make a test
easier.

The `no_claim_rebuild_or_projection_object_was_introduced` enum assertion is
restricted to `pg_type.typtype = 'e'`, because PostgreSQL always creates an
implicit composite type named after the table itself; an unrestricted
`typname LIKE 'metric_rollup%'` count would match that composite type and
assert the wrong thing.

## 10. Manual verification

1. `thoth migrate` on an empty database → 16-migration chain, ledger head
   `20260903`.
2. `\d+ public.metric_rollup_delta` → exactly the seven specified columns,
   types, nullability and defaults; exactly two indexes; exactly one
   non-cascading composite foreign key.
3. `pg_constraint` / `pg_trigger` inventory → 1 primary key, 1 unique, 1
   foreign key, **0** CHECK constraints, **0** non-internal triggers.
4. `SELECT count(*) FROM metric_rollup_delta` → `0` (no seeds).
5. Deferred projection tables → `0` of the four exist. Rollup enum types
   (`typtype = 'e'`) → `0`.
6. Targeted revert then reapply on the empty-chain database → 74 → 73 → 74
   tables, ledger head `20260903` → `20260902` → `20260903`.
7. `thoth migrate --revert` then reapply → full chain down to 0 ledger rows
   and back to 16.
8. Populated database: apply / revert / reapply with content, ledger and
   relfilenode snapshots → see §6.3.
9. Lock measurement inside an open migration transaction, plus concurrent
   read/write probes → see §6.5.
10. `diff -r` of every pre-existing migration directory against the base
    worktree → identical, `20260902_v1.9.0` included.
11. GraphQL SDL generated from base and implementation trees → identical
    SHA-256, identical size, `cmp` exit 0.

## 11. CI

CI status: **NOT AVAILABLE at the time this report was written.** The report
is committed before the branch is pushed and the draft PR opened, so no PR run
exists yet. Observed workflow, job and conclusion results — and the
classifier-controlled GHCR staging-image publication result — are reported in
the post-push completion handoff. GitHub remains the lifecycle authority.

No workflow was manually dispatched, rerun or cancelled, and no workflow file
was altered.

## 12. Rollout and rollback

Initial state after merge: additive, inactive schema on `feature/metrics`
only. No row exists, no code reads the table, no runtime behaviour changes.

Activation required: YES — later bounded, separately specified and authorized
WP2/WP4 work must implement delta generation inside the canonical ingestion
transaction, and delta claiming and application against rollup projections,
before this foundation has any effect.

Feature flag/configuration: none; there is nothing to switch on.

Migration sequence: `20260903_v1.9.0` applies after `20260902_v1.9.0`. It
depends on the MET-WP1-04 `metric_record_revision (record_id,
record_revision_id)` unique key. Production migration execution is **not**
authorized by this task.

Rollback: before any later dependent Metrics migration or runtime contract
merges, rollback is a separately authorized revert of the bounded child
integration, with the tested `down.sql` used only in disposable/non-production
environments under applicable migration authorization. The down migration is
exact and non-cascading. Once later Metrics work depends on this schema, use
dependency-aware reverse-order rollback or a separately reviewed
forward-repair plan instead of reverting in isolation. No production rollback
is authorized by this task.

Monitoring required: none at this gate; nothing is active.

## 13. Deviations and limitations

Deviations from the original #880 body: exactly one, and it is authorized —
the migration is dated `20260903` rather than a same-day identity, under CTO
amendment `5513578081`, which explicitly supersedes the body's future-dating
prohibition. `make migration` was correspondingly not used. No other deviation
from the approved specification was made.

Limitations, stated plainly:

- Persistence only. No delta is created, claimed or applied by any code path.
- `status` has no vocabulary. Nothing validates it, and no closed state
  machine exists in the database or in Rust.
- The four rebuildable rollup projection tables are not implemented; they
  remain approved future architecture pending a bounded specification that
  fixes their relational keys, null-dimension uniqueness, watermark
  representation and rebuild-generation protocol.
- No operational claim index exists. WP4 must add the exact index alongside
  its approved claim query and query-plan evidence.
- Lock evidence was gathered on a small disposable database. Concurrent writes
  to `metric_record_revision` **are** blocked during the migration
  transaction; the window measured here (3.631 ms) is not a prediction for a
  production-sized table.
- Independent exact-head source review has **not** been performed. This report
  records implementation only.

## 14. Unresolved issues

None blocking this bounded slice.

Open for later bounded work, unchanged by this task: the WP4 rollup-delta
claim/application protocol including claim ownership, leases, retries,
stale-claim recovery and `FOR UPDATE SKIP LOCKED` semantics; the status
vocabulary; the four rollup projection tables and their rebuild/watermark
protocol; and the WP2 ingestion transaction that will actually produce deltas.

## 15. Agent self-assessment

The implementation is bounded to the nine authorized paths and the approved
schema, and every acceptance requirement in #880 §9 was executed rather than
asserted. Two things a reviewer should check most carefully:

1. The **constraint-naming** choice (§5.2 item 7) and the **no-`joinable!`**
   conclusion (§5.2 item 8). Both follow merged repository precedent, but
   MET-WP1-04 named its composite foreign keys after the semantic pointer
   column rather than using the PostgreSQL default shape, so a reviewer may
   reasonably prefer a different name here.
2. The **incidental MET-WP1-06 status correction** in
   `docs/metrics/task-status.md` (§4.3). It is a verifiable git fact about this
   task's own base and makes no approval claim, but it does touch another
   task's tracker row, and a reviewer may prefer it deferred entirely to a
   `RECON` task.

Independent approval is **not** claimed. Implementation completion is not
approval.

## 16. Final state

```text
Final commit SHA:  <FILLED IN AFTER COMMIT>
PR:                <FILLED IN AFTER PR CREATION>
PR state:          DRAFT
CI state:          <REPORTED IN HANDOFF>
Current gate:      IMPLEMENTATION COMPLETE - DRAFT PR - AWAITING FRESH
                   INDEPENDENT EXACT-HEAD SOURCE REVIEW
```
