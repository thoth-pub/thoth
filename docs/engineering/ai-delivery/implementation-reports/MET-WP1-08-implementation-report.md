# MET-WP1-08 Implementation Report

## 1. Repository state

| Field | Value |
|---|---|
| Task | `MET-WP1-08` - Establish OPERAS mapping persistence foundation |
| Owning issue | [#882](https://github.com/thoth-pub/thoth/issues/882) |
| Parent programme | [#766](https://github.com/thoth-pub/thoth/issues/766) |
| Repository | `thoth-pub/thoth` |
| Risk | **HIGH** |
| Workflow | `PROGRAMME_INTEGRATION` |
| Exact authorized base | `feature/metrics @ 6093f0ca7f3b7221c656bf514d71b5812e39ac45` |
| Exact incorporated `develop` | `4546cb632428872b961ad6c17282984d298e3ade` |
| Task branch | `feature/metrics--wp1-operas-mapping` |
| PR target | `feature/metrics` |
| Migration identity | `thoth-api/migrations/20260904_v1.9.0/{up.sql,down.sql}` |
| Write budget | exactly nine paths |
| Runtime effect | **NONE** |
| Production migration / deployment / activation | **NOT AUTHORIZED, NOT PERFORMED** |

Authorization provenance:

| Record | Location | Decision |
|---|---|---|
| Independent specification review | #882 comment `5516271459` | **APPROVED** - required corrections NONE, architecture decision required NO |
| CTO specification approval | #882 comment `5516273603` | **APPROVED** at this exact baseline |
| Programme specification-gate record | #766 comment `5516275963` | recorded |
| Implementation authorization / durable binding | #882 comment `5516360686` | posted before branch creation and before any source mutation |

### 1.1 Migration-identity binding

The specification left the migration path deliberately `UNBOUND / HOLD`, because
the conventional 2026-09-02 path was already occupied by MET-WP1-06 and
`20260903_v1.9.0` was already owned by MET-WP1-07 under its task-specific
amendment.

The implementation authorization bound MET-WP1-08 to the **next unused
calendar-date identity**, `20260904_v1.9.0`. The repository's date-derived
migration identity is an organizational naming convention, not a semantic
runtime invariant; MET-WP1-07 established the accepted precedent that an unused
subsequent calendar-date identity may be bound rather than modifying or reusing
an existing migration.

Complete migration inventory observed at the exact authorized base (16
directories):

```text
20250000_v1.0.0  20260417_v1.1.0  20260429_v1.2.0  20260504_v1.2.0
20260805_v1.7.0  20260811_v1.7.0  20260812_v1.7.0  20260813_v1.6.3
20260814_v1.7.0  20260826_v1.9.0  20260827_v1.9.0  20260828_v1.9.0
20260831_v1.9.0  20260901_v1.9.0  20260902_v1.9.0  20260903_v1.9.0
```

`20260904_v1.9.0` was verified absent — as a directory, as `up.sql`, as
`down.sql` and in `git ls-files` — immediately before any source mutation.
`make migration` was **not** run. No predecessor migration was modified,
renamed, reused or reordered; §6.4 records the byte-level proof. Neither the
`Makefile` migration convention nor the workspace package version (`1.8.0`) was
changed.

Diesel derives a migration's ledger version from the directory-name text before
the first underscore, so this identity registers in
`__diesel_schema_migrations` as `20260904`.

### 1.2 Preflight verification

Performed against **live GitHub state** immediately before the binding comment
and before branch creation:

| Check | Required | Observed |
|---|---|---|
| `feature/metrics` | `6093f0ca…ac45` | `6093f0ca7f3b7221c656bf514d71b5812e39ac45` — match |
| `develop` | `4546cb63…3ade` | `4546cb632428872b961ad6c17282984d298e3ade` — match |
| `feature/metrics--wp1-operas-mapping` | absent | absent locally and on `origin` |
| MET-WP1-08 implementation PR | absent | absent; newest PR is #881 (merged) |
| `thoth-api/migrations/20260904_v1.9.0` | absent | absent |
| #882 materially edited after approval | no | `lastEditedAt: null`, no `userContentEdits` |

The complete current #882 issue body and both approval comments were re-read in
full at implementation time. `feature/metrics @ 6093f0ca` is itself the merge
commit of PR [#881](https://github.com/thoth-pub/thoth/pull/881) (MET-WP1-07),
which is why §4.3 records a factual predecessor-lifecycle correction in the
tracker.

### 1.3 Control-plane incident acknowledgement

During control-plane preflight, non-substantive `COMMENT` reviews were
accidentally created on the already-merged PR #881, with bodies including
`IGNORE`, a correction note, `STOP` and `DO NOT USE`.

Those comments were **not** treated as source reviews, approvals,
authorizations or task state. PR #881 was **not** modified and no cleanup was
attempted. They have no bearing on MET-WP1-08. The authoritative MET-WP1-07
state remains its completed merge and reconciliation.

### 1.4 Prompt-injection observation

No instruction-like content from repository files, issue text, tool output or
any other observed source was treated as authority. Authorization was taken
only from the task instruction and the durable GitHub approval records listed
in §1. The `.env` file in the working tree contains plaintext credentials; they
were neither used nor transmitted, and no provider, staging or production
system was contacted at any point.

## 2. Scope confirmation

Implemented: exactly one new additive, inactive table
`public.metric_operas_mapping`, its manually maintained Diesel schema contract,
one plain Rust persistence model, focused database/model tests, one CHANGELOG
entry, bounded tracker updates and this report.

Not implemented, by explicit non-goal: `metric_operas_export`,
`metric_operas_import`, `metric_reconciliation_run`,
`metric_reconciliation_issue`, OPERAS payload construction, OPERAS delivery,
claiming, leases, attempts, retries, backoff, status state machines, remote
event IDs, request hashes, delivery errors, inbound synchronization, loop
prevention, reconciliation, cursor/snapshot discovery, GraphQL administration,
internal worker mutations, Sphynx behaviour, publisher package or capability
changes, Metrics-specific entitlement tables, source/source-account changes,
publisher-platform approval behaviour, production migration, deployment and
activation.

No mapping row was seeded and no real `event_uri`, `measure_uri`,
`uploader_uri`, platform mapping or measure mapping value was invented,
inferred or approved. The two MET-WP1-01 `metric_measure` seed rows are
unchanged.

## 3. Commits

Exactly **one** implementation commit was created on
`feature/metrics--wp1-operas-mapping`, parented on
`6093f0ca7f3b7221c656bf514d71b5812e39ac45`. It was not amended, not rebased and
not force-pushed, and no second implementation commit was created.

This report deliberately records **no** final commit SHA, PR number or CI run
ID. Those facts do not exist until after the single authorized commit is
created, and manufacturing them inside the committed report would reintroduce
the circular reporting problem identified during MET-WP1-07. They belong in the
GitHub/PR evidence and in the task handoff.

## 4. Files changed

Exactly nine paths, matching the authorized write budget:

| # | Path | Change |
|---|---|---|
| 1 | `CHANGELOG.md` | one bounded `MET-WP1-08` Unreleased/Added entry |
| 2 | `docs/metrics/task-status.md` | header block, MET-WP1-08 row, WP1 row, plus the factual MET-WP1-07 lifecycle correction in §4.3 |
| 3 | `docs/engineering/ai-delivery/implementation-reports/MET-WP1-08-implementation-report.md` | new (this report) |
| 4 | `thoth-api/migrations/20260904_v1.9.0/up.sql` | new |
| 5 | `thoth-api/migrations/20260904_v1.9.0/down.sql` | new |
| 6 | `thoth-api/src/schema.rs` | new `table!` block + `allow_tables_to_appear_in_same_query!` entry |
| 7 | `thoth-api/src/model/mod.rs` | one `pub mod metric_operas_mapping;` registration |
| 8 | `thoth-api/src/model/metric_operas_mapping/mod.rs` | new |
| 9 | `thoth-api/src/model/metric_operas_mapping/tests.rs` | new |

### 4.1 Write-budget compliance

No tenth path. No deletion, move or rename. No dependency, lockfile, workflow,
CI, `Makefile`, package-version, GraphQL, authorization, provider or runtime
file was touched.

In particular `thoth-api/src/model/metric_platform_measure/tests.rs` was **not**
modified. Its `insert_measure_row`, `fixture_pair` and `insert_mapping_raw`
helpers are private; rather than widening that file for convenience, the new
test module reuses the already-public `pub(crate)` helpers
(`setup_registry_db`, `insert_platform_row`, `scalar_i64`,
`check_constraint_names`, `delete_row`, `foreign_keys`, `index_names`,
`index_definition`) and defines its own local raw-SQL `metric_measure` and
`metric_platform_measure` fixtures.

### 4.2 Authorized actions used

Branch creation from the exact base; bounded source edits; disposable/local
validation only; exactly one implementation commit; push of only the task
branch; one DRAFT PR targeting `feature/metrics`; natural PR CI; the
classifier-controlled automatic GHCR staging-PR image.

Not used: second commit, amend, rebase, force-push, manual CI
dispatch/rerun/cancel, mark-ready, merge, branch deletion, provider/runtime
mutation, staging or production migration execution, release, deployment,
activation, `feature/metrics -> develop` integration, issue closure, another
Metrics slice.

### 4.3 Predecessor-lifecycle correction in `docs/metrics/task-status.md`

The tracker at the base still described MET-WP1-07 as
`IMPLEMENTED ON feature/metrics--wp1-rollup-delta`, and the WP1 work-package row
repeated that wording. That became stale when PR
[#881](https://github.com/thoth-pub/thoth/pull/881) merged: its merge commit is
`6093f0ca7f3b7221c656bf514d71b5812e39ac45`, which is this task's authorized
base.

The MET-WP1-07 status was therefore corrected to
`MERGED TO feature/metrics - SEVENTH WP1 SLICE DELIVERED` with its PR and merge
commit recorded, and the WP1 row was updated to match. This is a directly
factual correction contained entirely within an already-authorized tracker
path; it claims nothing about MET-WP1-08's own lifecycle, which is recorded only
as implemented on its slice branch. WP1 remains `IN PROGRESS`.

## 5. Schema and model implemented

### 5.1 `public.metric_operas_mapping`

```sql
CREATE TABLE public.metric_operas_mapping (
    mapping_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    platform_id uuid NOT NULL,
    measure_id uuid NOT NULL,
    event_uri text NOT NULL,
    measure_uri text NOT NULL,
    uploader_uri text NOT NULL,
    enabled boolean NOT NULL,
    CONSTRAINT metric_operas_mapping_pkey PRIMARY KEY (mapping_id),
    CONSTRAINT metric_operas_mapping_platform_id_measure_id_key
        UNIQUE (platform_id, measure_id),
    CONSTRAINT metric_operas_mapping_platform_id_measure_id_fkey
        FOREIGN KEY (platform_id, measure_id)
        REFERENCES public.metric_platform_measure (platform_id, measure_id),
    CONSTRAINT metric_operas_mapping_event_uri_check
        CHECK (event_uri ~ '[^[:space:]]'),
    CONSTRAINT metric_operas_mapping_measure_uri_check
        CHECK (measure_uri ~ '[^[:space:]]'),
    CONSTRAINT metric_operas_mapping_uploader_uri_check
        CHECK (uploader_uri ~ '[^[:space:]]')
);
```

Observed post-migration inventory (`information_schema` / `pg_catalog`):

| Aspect | Observed |
|---|---|
| Columns | exactly 7, in the order above, all `NOT NULL` |
| Defaults | exactly one: `mapping_id` → `uuid_generate_v4()`; `enabled` has none |
| CHECK constraints | exactly 3, all the `~ '[^[:space:]]'` idiom |
| Foreign keys | exactly 1, composite, no `ON DELETE` clause |
| Indexes | exactly 2: `metric_operas_mapping_pkey`, `metric_operas_mapping_platform_id_measure_id_key` |
| Triggers | 0 |
| Enum types created | 0 |
| Seeded rows | 0 |

### 5.2 Implementation decisions

**Surrogate identity.** `mapping_id UUID PRIMARY KEY` with the
repository-standard Metrics UUID default. The approved design's shorthand
mapping list names no primary key, but its later conceptual
`metric_operas_export` row refers to a `mapping_id`; a surrogate key supplies
that referential target directly, without forcing a future export row to repeat
mutable configuration text.

**One mapping per registered pair.** `UNIQUE(platform_id, measure_id)`. The
design describes singular mapping configuration for one platform/measure and
phrases outbound eligibility as "the platform/measure has an enabled OPERAS
mapping". Several simultaneously canonical mappings would make both
enabled-state and later `mapping_id` selection ambiguous, and no version,
priority or effective-date model is defined. This constraint makes no real
mapping approved; the table is empty.

**Composite registry FK.** One non-cascading foreign key over
`(platform_id, measure_id)` against
`metric_platform_measure (platform_id, measure_id)`. Two independent
single-column keys would admit a mapping naming a real platform and a real
measure that are not registered together as a supported pair — a case the tests
prove is rejected here. Both columns are `NOT NULL`, so the MATCH SIMPLE
composite key is always enforced. No redundant standalone platform or measure
key was added. Non-cascading matches every other Metrics foreign key: deleting a
registry pair that still has mapping configuration fails rather than silently
erasing it.

**No duplicated `direct_collection`.** `metric_platform_measure.direct_collection`
remains the canonical direct/inbound flag and is not mirrored onto this table,
where it could drift. A test asserts the column exists exactly once in the
schema.

**URI text integrity only.** `event_uri`, `measure_uri` and `uploader_uri` carry
only the existing Metrics required-text CHECK idiom, rejecting blank and
whitespace-only values. No URI scheme restriction, parsing, normalization,
hostname rule, trailing-slash handling, remote validation or URI uniqueness was
introduced. An accepted value is evidence of nonblank-text storage only, never
of URI validity, reachability or OPERAS approval.

**`enabled`.** Required, with deliberately no database default, so a later
reviewed administrative write path must state activation explicitly.

**No timestamps.** The approved shorthand contains none, and none was added to
mirror other tables.

**Constraint naming.** PostgreSQL default shape
`<table>_<columns>_<pkey|key|fkey|check>`, matching merged MET-WP1-01 registry
keys such as `metric_platform_measure_platform_id_measure_id_key`.

**Indexes.** Exactly the primary-key index and the pair-uniqueness index.
PostgreSQL builds no index for the referencing side of a foreign key, and no
access path is encoded before the WP9 export query and its query-plan evidence
are approved.

### 5.3 Rust model

```rust
#[cfg_attr(feature = "backend", derive(diesel::Queryable))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricOperasMapping {
    pub mapping_id: Uuid,
    pub platform_id: Uuid,
    pub measure_id: Uuid,
    pub event_uri: String,
    pub measure_uri: String,
    pub uploader_uri: String,
    pub enabled: bool,
}
```

A plain persistence/domain type following the merged `MetricRollupDelta`
convention. No GraphQL object, input or resolver; no `juniper` derive; no
enum; no platform-name mapping; no URI validation client; no payload builder;
no export/import claim or apply method; no entitlement lookup; no Sphynx
coupling; no new dependency or Cargo feature; no runtime behaviour.

### 5.4 Diesel schema contract

`thoth-api/src/schema.rs` is manually maintained under ADR-0003 and was edited
atomically with the migration. Diesel CLI schema generation was **not** run.

Added: one `table!` block for `metric_operas_mapping (mapping_id)` in
alphabetical position, and one `metric_operas_mapping` entry in
`allow_tables_to_appear_in_same_query!`.

Deliberately **not** added: any `joinable!` declaration. The foreign key is
composite, and Diesel's `joinable!` expresses a single-column relationship;
declaring one would misrepresent the constraint. This follows the merged
`metric_rollup_delta` composite-FK precedent, which likewise has no `joinable!`.

### 5.5 Explicit statements required by the specification

- ADR-0001 remains the entitlement authority. No `metric_entitlement` table, no
  duplicate capability model, no package-name logic. This does **not** claim WP5
  capability enforcement is implemented; later OPERAS export eligibility must
  consume the shared `METRICS_OPERAS_EXPORT` capability.
- ADR-0002 remains binding: `MetricPlatform != DistributionPlatform`, with no
  name-based or enum-order conversion introduced.
- ADR-0003 remains schema-control authority: migration, `schema.rs`, model and
  tests changed atomically in one commit.
- Real source/platform/OPERAS URI mappings remain unapproved and unseeded.
- WP1 remains incomplete after this slice.

## 6. Database and migration effects

### 6.1 Environment

macOS (aarch64, Darwin 25.6.0). Homebrew PostgreSQL **17.10** — matching the
`postgres:17` image used by repository CI — with databases created
`ENCODING 'UTF8' TEMPLATE template0 LC_COLLATE 'C' LC_CTYPE 'C'`. Local Redis
8.x. Four throwaway databases dedicated to this task: `thoth_wp108_empty`
(empty-chain), `thoth_wp108_pop` (representative populated), `thoth_wp108_lock`
(lock observation), `thoth_wp108_timing` (chain timing), plus `thoth_test` for
the Rust harness. **No staging or production system was contacted at any point,
and no staging or production migration was executed.**

The embedded migration set was confirmed to include the new directory before
these runs — `strings target/debug/thoth | grep -oE '2026090[0-9]_v[0-9.]+'` →
`20260901_v1.9.0`, `20260902_v1.9.0`, `20260903_v1.9.0`, `20260904_v1.9.0` —
after `touch thoth-api/src/db.rs`, because `embed_migrations!` is expanded at
compile time and cargo does not track migration-directory additions on stable.
The base-worktree binary built from `6093f0ca` correspondingly embeds only up to
`20260903_v1.9.0`.

### 6.2 Empty-database chain validation

`thoth_wp108_empty`:

| Step | Command | Result |
|---|---|---|
| Full chain apply | `thoth migrate` on an empty database | 17 migrations applied; ledger head `20260904`; 75 tables; `metric_operas_mapping` present with 0 rows, 2 indexes, 3 CHECKs, 1 FK |
| Task migration revert | `down.sql` + ledger row removal | table dropped; 75 → 74 tables; ledger head `20260903`; MET-WP1-01 pair unique key intact; measure seeds intact (2); MET-WP1-07 table intact |
| Task migration reapply | `thoth migrate` | ledger head `20260904`; 74 → 75 tables; 0 rows, 2 indexes, 3 CHECKs, 1 FK restored |
| Full chain revert | `thoth migrate --revert` | 0 ledger rows; 1 relation remaining (`__diesel_schema_migrations`) |
| Full chain reapply | `thoth migrate` | 17 ledger rows; head `20260904`; 75 tables; 0 mapping rows |

The repository implements `--revert` as `revert_all_migrations`, so the full
chain revert exercises this task's `down.sql` inside the complete reverse chain.

The four deferred ledgers (`metric_operas_export`, `metric_operas_import`,
`metric_reconciliation_run`, `metric_reconciliation_issue`) were confirmed
absent after the full chain applied.

### 6.3 Representative populated-database validation

`thoth_wp108_pop` was built with the **base-commit binary** (chain head
`20260903`, 74 tables, `metric_operas_mapping` absent), then populated with
representative bibliographic state and MET-WP1-01..07 rows:

```text
publisher 1, imprint 1, work 1, publication 1, institution 1
metric_platform 1, metric_measure 2 (seeded), metric_platform_measure 2
  (one direct_collection = FALSE, one TRUE)
metric_source 1, metric_source_account 1, metric_source_checkpoint 1
metric_import 1, metric_import_error 1
metric_record 1, metric_record_revision 2 (SUPERSEDED + CURRENT chain),
metric_record_provenance 2 (one REVISION, one record-less REJECTED)
metric_coverage 1
metric_publisher_platform_approval 1
metric_rollup_delta 1
```

Snapshots captured: table count, enum count, the full migration ledger with
`run_on` values, md5 content hashes of **all 74** public tables, every relation
and index `relfilenode`, and the complete 394-row constraint inventory.

| Step | Observed |
|---|---|
| Apply `20260904` | Exactly four kinds of difference: table count 74 → 75; one new ledger row `20260904`; one new content hash for the empty `metric_operas_mapping` (`d41d8cd98f00b204e9800998ecf8427e`, the md5 of the empty string); three new relfilenodes (the table and its two indexes); six new constraints, all owned by `metric_operas_mapping` (3 `c`, 1 `p`, 1 `u`, 1 `f`). **Every predecessor table content hash identical. Every pre-existing relfilenode identical. Enum count unchanged at 39.** |
| Revert | Snapshot **byte-identical** to the pre-migration snapshot — relfilenodes and the full constraint inventory included. |
| Reapply | Identical to the post-apply snapshot apart from the new ledger row's `run_on` timestamp and the new relations' relfilenodes, both of which necessarily differ for newly created physical files. |

So: pre-existing content is unchanged; no populated table was rewritten or
backfilled; the migration ledger changes only by the new identity; and the down
migration removes only MET-WP1-08-owned objects.

### 6.4 Predecessor migration integrity

`diff -r` of `thoth-api/migrations` against the base worktree reported exactly
one difference:

```text
Only in thoth-api/migrations: 20260904_v1.9.0
```

A per-directory SHA-256 of concatenated `*.sql` confirmed all **16** predecessor
migration directories byte-identical, `20260902_v1.9.0` and `20260903_v1.9.0`
included.

### 6.5 Lock evidence

Measured on disposable PostgreSQL 17.10 (`thoth_wp108_lock`, populated as in
§6.3). Session A opened a transaction, ran the complete `up.sql`, then held the
transaction open with `pg_sleep(10)`; session B sampled `pg_locks` joined to
`pg_class` and `pg_stat_activity`, then attempted concurrent access with
`lock_timeout = '2s'`.

Locks observed against pre-existing tables, all granted:

| Relation | Lock mode | Granted |
|---|---|---|
| `metric_platform_measure` | `AccessShareLock` | yes |
| `metric_platform_measure` | `ShareRowExclusiveLock` | yes |

No other pre-existing relation was locked; the foreign key references only
`metric_platform_measure`. The new table's own `AccessExclusiveLock` on itself
is not reported, because the relation is not yet visible outside the creating
transaction — and is immaterial, since nothing can reference a table that does
not yet exist.

Empirically confirmed blocking behaviour while the DDL transaction was held:

| Concurrent statement | Target | Outcome |
|---|---|---|
| `SELECT count(*)` | `metric_platform_measure` | **Not blocked** — returned immediately |
| `DELETE … WHERE false` (`RowExclusiveLock`) | `metric_platform_measure` | **Blocked** — `canceling statement due to lock timeout` after 2 s |
| `SELECT count(*)` | `metric_platform` | **Not blocked** |
| `DELETE … WHERE false` | `metric_platform` | **Not blocked** |
| `SELECT count(*)` | `metric_measure` | **Not blocked** |
| `SELECT count(*)` | `work` | **Not blocked** |

Interpretation, stated precisely: `ShareRowExclusiveLock` does not conflict with
`AccessShareLock` or `RowShareLock`, so concurrent **reads** of
`metric_platform_measure` proceed. It **does** conflict with `RowExclusiveLock`,
so concurrent **writes** to `metric_platform_measure` — inserts, updates and
deletes — are blocked for the duration of the migration transaction. **No claim
of "zero write blocking" is made, because the observed evidence contradicts
it.**

Duration: the isolated `up.sql` executed in **3.109 ms / 3.134 ms / 3.190 ms**
across three runs on a small disposable database; the full 17-migration chain
applied to an empty database in **0.211 s**. The blocking window is therefore
expected to be very short in practice, but it is not zero, and on a
production-sized `metric_platform_measure` the foreign key's own validation scan
would extend it. `metric_platform_measure` is currently unseeded in the merged
Metrics schema, so the scan cost today is trivial. Production migration
execution is **not** authorized by this task and was not performed.

### 6.6 Migration/data effects summary

Additive new table only. No backfill. No seed. No mutation of registry, source,
import, record, revision, provenance, coverage, approval or rollup-delta state.
The down migration drops only `metric_operas_mapping`, whose two indexes and six
constraints go with it; no enum, trigger, sequence or standalone index was
created by this migration, so a single non-cascading `DROP TABLE IF EXISTS` is
exact.

## 7. API and compatibility effects

GraphQL/API changes: **NONE.** The module exposes no GraphQL object, input,
resolver or enum, and adds no query or mutation.

Byte-for-byte SDL comparison:

| Tree | SHA-256 of `thoth-client/assets/schema.graphql` | Bytes |
|---|---|---|
| Base `6093f0ca7f3b7221c656bf514d71b5812e39ac45` | `091e11f293132fdec784de420e3addf251f5020ba7e387889b292a066be15d8e` | 178270 |
| Final implementation tree | `091e11f293132fdec784de420e3addf251f5020ba7e387889b292a066be15d8e` | 178270 |

`cmp` reports the two files identical. The SDL is build-generated by
`thoth-client/build.rs` and gitignored, so `git status` is not a valid check;
both sides were regenerated by forcing `build.rs` to re-run, and the base value
was additionally reproduced independently in a `git worktree` checked out at the
exact base commit. Both independent base captures agree.

Other contracts:

| Contract | Impact |
|---|---|
| database/domain model | **AFFECTED** — owned by `thoth-pub/thoth` |
| GraphQL/API schema/behaviour | NOT AFFECTED (byte-identical SDL) |
| generated clients/types | NOT AFFECTED |
| authorization semantics | NOT AFFECTED |
| package capabilities (ADR-0001) | NOT AFFECTED |
| source-driver contract | NOT AFFECTED |
| OPERAS runtime/adaptor contract | NOT EXPOSED in this slice; future consumer |
| export formats/payloads | NOT AFFECTED |
| configuration/environment contracts | NOT AFFECTED |
| dissemination/platform behaviour | NOT AFFECTED |
| UI assumptions | NOT AFFECTED |
| deployment compatibility | AFFECTED only as a future additive migration; deployment not authorized |

`thoth-sphynx`, `thoth-app`, `metrics-dashboard`, `metrics-widget`,
`thoth-client` and `thoth-dissemination` require no repository-local source
change. This slice creates no GraphQL contract and grants Sphynx no direct
database authority.

## 8. Authorization and security

No GraphQL, API or auth change. No service-role, credential, secret or policy
change. No package/capability model change and no entitlement enforcement
activated. No credentials or secrets were read, used, stored or transmitted. No
external network call was made by the implementation or by its tests. No
provider, staging or production state was accessed.

## 9. Tests and checks

All commands were run on the final implementation tree.

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | **PASS** (no output, exit 0) |
| `cargo check --workspace` | **PASS** (finished in 2m 12s) |
| `cargo clippy --all --all-targets --all-features -- -D warnings` | **PASS** (finished in 2m 42s) |
| `cargo test -p thoth-api --features backend` | **PASS** — 1466 lib tests passed, 0 failed; 13 integration tests passed; 8 doctests ignored. One non-reproducing failure was observed on the very first run; see §9.2. |
| `cargo test --workspace` | **PASS** — 1672 passed, 0 failed, plus 8 doctests; see §9.3 |
| `git diff --check` | **PASS** (no output, exit 0) |

### 9.1 Focused database/model tests

19 new tests in `thoth-api/src/model/metric_operas_mapping/tests.rs`, all
passing:

| Test | Proves |
|---|---|
| `migration_seeds_no_operas_mapping_row` | migration seeds zero mapping rows |
| `a_complete_operas_mapping_round_trips_through_diesel` | valid complete row round-trip |
| `both_enabled_states_round_trip_without_an_implicit_default` | `enabled = true` and `enabled = false` round-trip |
| `arbitrary_nonblank_uri_text_round_trips_in_every_uri_column` | arbitrary nonblank `event_uri`, `measure_uri` and `uploader_uri` round-trip unchanged (including a padded value, proving no normalization) and do not leak across columns |
| `blank_and_whitespace_only_event_uri_is_rejected` | blank and whitespace-only `event_uri` rejected |
| `blank_and_whitespace_only_measure_uri_is_rejected` | blank and whitespace-only `measure_uri` rejected |
| `blank_and_whitespace_only_uploader_uri_is_rejected` | blank and whitespace-only `uploader_uri` rejected |
| `mapping_id_is_generated_when_omitted_and_honoured_when_supplied` | `mapping_id` generated when omitted |
| `operas_mapping_not_null_columns_are_enforced` | NOT NULL behaviour for every required column |
| `an_unknown_platform_measure_pair_is_rejected` | unknown platform/measure pair rejected |
| `a_real_platform_and_real_measure_not_registered_together_are_rejected` | individually valid ids with no joint `metric_platform_measure` registration rejected |
| `at_most_one_mapping_is_permitted_per_registered_pair` | duplicate `(platform_id, measure_id)` rejected |
| `deleting_a_referenced_platform_measure_is_restricted_and_does_not_cascade` | deleting a referenced registry row fails and does not cascade |
| `metric_operas_mapping_has_exactly_the_approved_columns` | exact column inventory, plus exactly one default (`mapping_id`) and none on `enabled` |
| `metric_operas_mapping_has_exactly_the_required_text_checks` | exact CHECK inventory, each the `~ '[^[:space:]]'` idiom |
| `metric_operas_mapping_has_exactly_the_authorized_non_cascading_foreign_key` | exact foreign-key inventory, composite shape, no `ON DELETE` |
| `metric_operas_mapping_has_exactly_the_required_indexes` | exact index inventory: primary key plus pair uniqueness only |
| `no_operas_ledger_reconciliation_or_delivery_object_was_introduced` | no `metric_operas_export`, `metric_operas_import`, `metric_reconciliation_run` or `metric_reconciliation_issue`; no deferred delivery/claim column; `direct_collection` present exactly once schema-wide; no OPERAS enum; no trigger |
| `reverting_through_the_operas_mapping_migration_removes_it_and_reapplication_restores_it` | targeted revert removes only this table, predecessors and seeds survive, reapply restores the table, both indexes, the composite FK and all three CHECKs |

No test claims that OPERAS export eligibility, capability enforcement, payload
construction, delivery, inbound synchronization, reconciliation or any runtime
behaviour is implemented. The URI round-trip test states explicitly that
acceptance is evidence of nonblank-text storage only.

### 9.2 Observed test flake

On the first full `cargo test -p thoth-api --features backend` run, the
pre-existing test
`graphql::distribution_platform_tests::loader_failure_matches_the_direct_baseline_error_shape`
failed once. It passed in isolation immediately afterwards, and two further
consecutive full-suite runs on the final tree passed with 1466/1466.

Characterisation performed:

| Tree | Full `--lib` runs | Result |
|---|---|---|
| Final implementation tree | 6 | 5 clean at 1466/1466; the single failure was the **first** run, executed against a `thoth_test` database inherited from a previous session rather than a freshly created one |
| Base worktree at `6093f0ca`, freshly recreated `thoth_test` | 3 | 3 clean at 1447/1447 — not reproduced |

The failure was therefore **not reproduced** in five subsequent final-tree runs
nor in three base runs, and this report does not claim to have attributed it.
What can be stated precisely: it is not deterministic; it does not reproduce in
isolation; and the only plausible shared-state mechanism is pre-existing rather
than introduced here. `setup_registry_db` reverts and reapplies the migration
chain, which drops and recreates the registry enum types and so changes their
PostgreSQL type OIDs — its own source comment documents exactly this hazard for
connections cached in the long-lived shared pool that
`test_db::setup_test_db` hands to tests such as this one. That interaction has
existed since MET-WP1-01 and is exercised by every predecessor Metrics slice;
this task adds 19 further such cycles but introduces no new mechanism.

An honest caveat: three base runs are not enough to prove the flake pre-exists,
only enough to show it is rare and was not reproduced. It is recorded here so a
reviewer can weigh it rather than discover it.

The test is a `tokio` multi-thread GraphQL error-shape comparison against a
deliberately failing connection pool. It touches no Metrics table and no code
path this task changes, and this task's GraphQL SDL is byte-identical to the
base, so the change cannot alter its behaviour.

### 9.3 Workspace test run

```text
cargo test --workspace
```

All suites green, 0 failures:

| Target | Passed |
|---|---|
| `thoth` (bin) | 31 |
| `thoth_api` (lib) | 1466 |
| `thoth_api` `tests/graphql_permissions.rs` | 13 |
| `thoth_api_server` | 3 |
| `thoth_client` | 4 |
| `thoth_errors` | 11 |
| `thoth_export_server` | 144 |
| **Total unit/integration** | **1672 passed, 0 failed** |
| Doc-tests (`thoth_client` 6, `thoth_export_server` 2) | 8 passed, 8 ignored |

`TEST_DATABASE_URL`, `TEST_REDIS_URL` and `THOTH_EXPORT_API` were exported into
the process environment; `thoth-api-server`'s three handler tests read
`TEST_DATABASE_URL` with `std::env::var` and nothing loads `.env` in that test
process, so without the export they fail `NotPresent` for environmental reasons
unrelated to any change.

## 10. Rollout and rollback

Repository integration only, and not yet performed:

1. merge only into repository-local `feature/metrics`, never directly into `develop`;
2. require fresh independent exact-head source approval;
3. require explicit CTO merge authorization bound to that exact reviewed source head;
4. preserve the reviewed source/tree under the authorized merge method;
5. verify the resulting `feature/metrics` commit/tree after merge;
6. reconcile #882 and the #766 programme ledger only after merge evidence is verified;
7. keep WP1 `IN PROGRESS`;
8. keep later WP1 slices, WP2–WP11 and MET-E2E gates separate.

Runtime rollout: none. Merging into `feature/metrics` must not run staging or
production migrations, create real mappings, activate OPERAS
export/import/reconciliation, deploy Sphynx, provision credentials or roles, or
change client behaviour.

Rollback: before later dependent OPERAS ledger/API migrations merge, repository
source rollback is a separately authorized revert of this bounded child
integration, with the tested `down.sql` used only in disposable/non-production
environments under applicable migration authorization. Once later schema or
runtime contracts depend on `metric_operas_mapping`, do not revert it in
isolation; use dependency-aware reverse-order rollback or a separately reviewed
forward-repair plan. No production rollback is authorized here.

## 11. Deviations

**NONE.** The implementation matches the approved specification, the
independent review's non-blocking observations (composite-FK `joinable!`
avoidance, nine-path budget preservation without widening
`metric_platform_measure/tests.rs`, and explicit UUID-default and NOT NULL
tests) and the implementation-time binding, including the exact migration
identity and the exact nine-path write budget.

## 12. Limitations

- Validation used disposable local PostgreSQL 17.10 only. Behaviour on a
  production-sized database — in particular the foreign key's validation scan
  against a populated `metric_platform_measure` — was not and could not be
  measured here.
- The lock measurement shows concurrent writes to `metric_platform_measure` are
  blocked for the migration transaction's duration. The measured duration is
  from a small database and is not a production estimate.
- Nothing consumes `metric_operas_mapping` yet. Its usefulness depends entirely
  on later separately specified WP5/WP9 work.
- Real OPERAS `event_uri`, `measure_uri` and `uploader_uri` values, and the
  platform/measure mappings themselves, remain unresolved external inputs. This
  slice deliberately does not resolve them, so the table cannot be populated
  until a source owner supplies and a reviewer approves those values.
- This report records no post-commit facts (final commit SHA, PR number, CI run
  IDs); see §3.

## 13. Remaining gates

```text
FRESH INDEPENDENT EXACT-HEAD SOURCE REVIEW REQUIRED
```

Then, and only then, explicit CTO merge authorization bound to the exact
reviewed source head.

Explicitly unauthorized next actions: mark ready, merge, deployment, production
migration, release, activation, branch deletion,
`feature/metrics -> develop` integration, another Metrics slice.

The implementing agent does not approve its own work.
