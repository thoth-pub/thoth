# MET-WP1-03 Implementation Report

Programme: Thoth Metrics - canonical ingestion, Sphinx orchestration and client cutover
Owning GitHub issue: [#863](https://github.com/thoth-pub/thoth/issues/863)
Parent programme issue: [#766](https://github.com/thoth-pub/thoth/issues/766)
Repository: `thoth-pub/thoth`
Task ID: MET-WP1-03 — Establish Metrics import and import-error state foundation
Risk: HIGH
Workflow: PROGRAMME_INTEGRATION

Authority condition: this report records what was implemented and measured. It
makes no approval decision and is not a self-approval. Live review,
merge-authorization and merge evidence is the GitHub pull-request and
owning-issue record.

## 1. Repository state

| Field | Value |
|---|---|
| Owning GitHub issue | [#863](https://github.com/thoth-pub/thoth/issues/863) |
| Repository | `thoth-pub/thoth` |
| Workflow | PROGRAMME_INTEGRATION |
| Programme integration branch | `feature/metrics` |
| Base branch / eventual PR target | `feature/metrics` |
| Authorized base commit | `e3fa9c7dfd68512d400dcd5dd6051e9648f5b568` |
| Actual base commit | `e3fa9c7dfd68512d400dcd5dd6051e9648f5b568` — reverified live immediately before branch creation and again immediately before push; the task branch was created from this exact SHA |
| Incorporated `develop` checkpoint | `665583371e0bbb64bf2d71836b4ce1a73e57d3a2` — reverified live and unmoved, so no programme-refresh check was triggered |
| Task branch | `feature/metrics--wp1-import-state` (ADR-0009 sibling spelling; namespace preflight re-run against live remote refs immediately before creation: exact ref absent, no PR, identifiers valid) |
| Head commit | the exact head recorded on #863 after the final push; it is the SHA the fresh independent exact-head review must be taken against and is deliberately not transcribed into this file |
| Pull request | NONE — PR creation is explicitly **not** authorized by `5456138970` |
| Expected branch deletion after merge | YES, under separate authorization |
| Final programme PR required | YES — the later coordinated `feature/metrics -> develop` integration is a separate gate and is not part of this task |
| Implementing model | Claude Opus 5 |
| Reasoning level | HIGH |

### 1.1 Authorization chain

The complete approved specification is:

1. issue #863 body (bounded WP1 import-state-foundation specification);
2. authoritative specification amendment `5455869065` — closes the CHECK
   inventory, fixes `created_by`/foreign-key semantics, fixes the exact
   rollback enum object set and binds test-fixture locality; **controls where
   it narrows the body**;
3. independent specification re-review `5456062629` — APPROVED;
4. CTO specification approval `5456079905` — APPROVED;
5. exact-SHA implementation authorization `5456138970` (controlling
   action-authorization record).

A fresh ACTIVE execution claim was posted on #863 before any mutation
(`5456179231`) and released on completion. Before that claim, all four earlier
`MET-WP1-03` lifecycle claims (`5455863012`, `5456059089`, `5456077706`,
`5456133764`) were verified released, so no unreleased ACTIVE execution claim
existed.

### 1.2 Preflight results

Every mandatory preflight check passed before any mutation:

| Check | Result |
|---|---|
| Live refs fetched | YES |
| `feature/metrics` equals the authorized base | YES — `e3fa9c7dfd68512d400dcd5dd6051e9648f5b568` |
| `develop` equals the authorized checkpoint | YES — `665583371e0bbb64bf2d71836b4ce1a73e57d3a2` |
| `feature/metrics--wp1-import-state` absent remotely | YES |
| No PR exists for the task branch | YES — `gh pr list --head ... --state all` returned `[]` |
| Migration convention reproduces the authorized path | YES — see 1.3 |
| Authorized migration path absent | YES — on `feature/metrics`, `develop` and `master` |
| No unreleased ACTIVE execution claim on #863 | YES |
| Task-branch push has no automatic Actions/GHCR side effect | YES — see 4.3 |

### 1.3 Migration path derivation

The repository `Makefile` `migration` target derives the directory as
`thoth-api/migrations/$(DATE)_v$(MAJOR).$(MINOR)+1.0` from the root
`Cargo.toml` version. At implementation time that version was `1.7.0`
(`MAJOR=1`, `MINOR=7`, so `v1.8.0`) and the date was `2026-08-28`, resolving
to exactly:

```text
thoth-api/migrations/20260828_v1.8.0
```

`make migration` was then run and created exactly that directory, matching the
authorized path. The path was verified absent beforehand. Earlier dated
`v1.8.0` directories (`20260826_v1.8.0`, `20260827_v1.8.0`) already exist;
duplicate semantic-version text across distinct dated directories is
established repository behaviour and did not justify a different scheme.

**Observation:** local wall-clock time crossed midnight into `2026-08-29`
during the later validation work. The migration directory had already been
created under the authorized `20260828` name while the date was still
`2026-08-28`, and the authorization binds the explicit literal path
`thoth-api/migrations/20260828_v1.8.0`, which is exactly what exists. A
re-run of `make migration` after midnight would now derive `20260829_v1.8.0`;
that is a property of the date-based generator, not a rebinding of the
authorized path, and no migration file was created or renamed after the date
changed.

## 2. Scope confirmation

Implemented exactly the approved additive, inactive import-state foundation:

- PostgreSQL enum `metric_import_status` with exactly the six approved labels;
- PostgreSQL enum `metric_import_error_severity` with exactly `ERROR` and
  `WARNING`;
- `public.metric_import` with the exact approved field, type, nullability,
  default, foreign-key and CHECK set;
- the design-fixed two-path idempotency contract as two mutually exclusive
  partial unique indexes;
- the single design-required `(status, created_at)` operational index;
- `public.metric_import_error` with the exact approved contract;
- manually maintained `thoth-api/src/schema.rs` under ADR-0003;
- matching Rust persistence/domain models and closed DB enums;
- focused database/model tests;
- bounded changelog/tracker/report consequences.

No GraphQL/API surface, queue/claim/lease/retry behaviour, ingestion runtime,
publisher upload, source-specific mapping, publisher-platform approval,
record/revision/provenance, coverage/rollup/OPERAS state, entitlement or
capability enforcement, `distribution_job*` reuse, Cargo/dependency change or
provider/runtime behaviour was introduced.

## 3. Commits

One bounded commit on `feature/metrics--wp1-import-state`, created only after
every required validation passed. No force push, no force-with-lease, and no
history-rewriting rebase or amend of pushed history.

## 4. Files changed

Exactly the authorized write budget, and no other path.

Existing files (4):

| Path | Change |
|---|---|
| `CHANGELOG.md` | one bounded `Unreleased`/`Added` entry |
| `docs/metrics/task-status.md` | `MET-WP1-03` row, header currency, WP1 dependency wording and two narrative sentences |
| `thoth-api/src/schema.rs` | two `sql_types` structs, two `table!` blocks, three `joinable!` lines, two `allow_tables_to_appear_in_same_query` entries — **purely additive, zero removed lines** |
| `thoth-api/src/model/mod.rs` | two `pub mod` registrations, alphabetically placed |

New files (7):

| Path | Purpose |
|---|---|
| `thoth-api/migrations/20260828_v1.8.0/up.sql` | forward migration |
| `thoth-api/migrations/20260828_v1.8.0/down.sql` | down migration |
| `thoth-api/src/model/metric_import/mod.rs` | `MetricImportStatus`, `MetricImport` |
| `thoth-api/src/model/metric_import/tests.rs` | focused import database/model tests |
| `thoth-api/src/model/metric_import_error/mod.rs` | `MetricImportErrorSeverity`, `MetricImportError` |
| `thoth-api/src/model/metric_import_error/tests.rs` | focused import-error database/model tests |
| `docs/engineering/ai-delivery/implementation-reports/MET-WP1-03-implementation-report.md` | this report |

### 4.1 Write-budget compliance

`git diff --name-status` against the exact base lists exactly these eleven
paths. No deletion, move or rename occurred. In particular
`thoth-api/src/model/metric_source_account/tests.rs` was **not** modified, as
required by amendment `5455869065` A3: its `insert_publisher_row` and
`delete_row` helpers are private at the reviewed baseline, so equivalent
fixtures were defined inside the two authorized new test modules. The
`pub(crate)` helpers `fixture_source_and_platform`, `insert_account_row`,
`insert_source_row`, `insert_platform_row`, `enum_labels`, `scalar_i64` and
`setup_registry_db` were consumed as-is, which their existing visibility
already permits.

### 4.2 Authorized actions actually used

Used: repository/doctrine inspection; an isolated worktree; local branch
creation from the exact base; edits confined to the budget; disposable local
PostgreSQL 17 only; the repository migration-generation convention at the
authorized path; test-first development; full local validation; one bounded
commit; pre-push reverification; one non-force push; durable reporting and
claim release.

Not used and not performed: PR creation/update, manual CI dispatch or rerun,
merge, task-branch deletion, force push or force-with-lease, staging or
production migration execution, provider/runtime access, production secrets,
release/tag/publication, deployment or production activation.

### 4.3 Automatic and manual external effects

Workflow triggers were re-inspected at the exact base and again immediately
before push:

| Workflow | Trigger | Task-branch push effect |
|---|---|---|
| `build_test_and_check.yml` | `push` to `master`/`develop`, `pull_request`, `workflow_dispatch` | none |
| `run_migrations.yml` | `push` to `master`/`develop`, `pull_request`, `workflow_dispatch` | none |
| `check_changelog.yml` | `pull_request` | none |
| `docker_build_and_push_to_dockerhub.yml` (GHCR staging) | `pull_request`, `workflow_dispatch` | none |
| `docker_build_and_push_to_dockerhub_release.yml` | `release: published` | none |

This matches the inventory accepted in authorization `5456138970` exactly, so
no automatic-side-effect rebinding was required. The authorized non-force push
of the task branch therefore triggers no Actions run and no GHCR publication.
The PR-triggered staging-image publication remains unauthorized and did not
occur.

## 5. Implementation decisions

All decisions are the approved specification's; none is novel architecture.

1. **Non-blank text** uses the established repository idiom
   `CHECK (col ~ '[^[:space:]]')`, identical to `MET-WP1-01`/`MET-WP1-02`.
2. **`created_by`** stays plain non-null text. No account foreign key,
   identity-provider binding, UUID requirement, actor namespace or format rule
   was added (amendment A1.1).
3. **Exactly two foreign keys** on `metric_import`, both non-cascading, so
   deleting a referenced source account or publisher fails rather than
   silently deleting durable import/audit evidence (A1.2).
4. **Exactly ten `metric_import` CHECK constraints** and **exactly two
   `metric_import_error` CHECK constraints** (A1.3, A2.1). Deliberately
   absent: any import-period ordering check (A1.4 — the approved design places
   that on `metric_record`, so malformed source period evidence stays
   representable), any counter-relationship check (A1.5), any `row_number`
   sign/range/origin check (A2.2) and any status/timestamp transition
   constraint.
5. **Idempotency** is expressed as two mutually exclusive partial unique
   indexes, preserving both nullable evidence columns. No constraint requires
   a newly inserted row to already carry idempotency evidence; the later
   upload/claim APIs own that rule. `ON CONFLICT DO NOTHING` was not used
   anywhere in the migration.
6. **Index inventory** is closed: primary keys, the two idempotency indexes
   and the single `(status, created_at)` operational index. No speculative
   secondary index.
7. **No GraphQL derive** on either enum or struct; the modules document why.

### 5.1 Exact idempotency predicates implemented

```sql
CREATE UNIQUE INDEX metric_import_source_account_id_upstream_report_id_idx
    ON public.metric_import (source_account_id, upstream_report_id)
    WHERE upstream_report_id IS NOT NULL;

CREATE UNIQUE INDEX metric_import_source_account_id_raw_sha256_format_version_idx
    ON public.metric_import (source_account_id, raw_sha256, format_version)
    WHERE upstream_report_id IS NULL AND raw_sha256 IS NOT NULL;
```

Read back from PostgreSQL after migration:

```text
CREATE UNIQUE INDEX metric_import_source_account_id_upstream_report_id_idx
  ON public.metric_import USING btree (source_account_id, upstream_report_id)
  WHERE (upstream_report_id IS NOT NULL)
CREATE UNIQUE INDEX metric_import_source_account_id_raw_sha256_format_version_idx
  ON public.metric_import USING btree (source_account_id, raw_sha256, format_version)
  WHERE ((upstream_report_id IS NULL) AND (raw_sha256 IS NOT NULL))
CREATE INDEX metric_import_status_created_at_idx
  ON public.metric_import USING btree (status, created_at)
```

The predicates partition on `upstream_report_id IS NOT NULL` / `IS NULL`, so
exactly one path applies to any row and the fallback can never replace or
broaden the upstream-report path.

## 6. Schema objects created

| Object | Kind | Detail |
|---|---|---|
| `metric_import_status` | enum | `UPLOADED`, `QUEUED`, `PROCESSING`, `COMPLETED`, `COMPLETED_WITH_ERRORS`, `FAILED` (verified in order) |
| `metric_import_error_severity` | enum | `ERROR`, `WARNING` (verified in order) |
| `metric_import` | table | 22 columns; PK `import_id`; UUID and `CURRENT_TIMESTAMP` defaults; six `BIGINT` counters defaulting to `0`; `manifest jsonb` defaulting to `'{}'` |
| `metric_import_error` | table | 9 columns; PK `import_error_id`; UUID and `CURRENT_TIMESTAMP` defaults |
| 2 foreign keys | constraint | `metric_import -> metric_source_account`, `metric_import -> publisher`; both non-cascading (no `ON DELETE` clause) |
| 1 foreign key | constraint | `metric_import_error -> metric_import`; non-cascading |
| 10 CHECKs | constraint | 4 non-blank text, 6 non-negative counters |
| 2 CHECKs | constraint | non-blank `error_code`, non-blank `message` |
| 4 indexes | index | `metric_import`: PK + 2 partial unique + 1 operational |
| 1 index | index | `metric_import_error`: PK only |

No row of any kind is seeded.

## 7. Test-first (TDD) evidence

New database behaviour was developed test-first in three observed stages.

**Stage 1 — RED (tests + models + `schema.rs`, no migration).** All 38 new
database-dependent assertions failed for the right reason:

```text
test result: FAILED. 2 passed; 36 failed; 0 ignored; 1292 filtered out

migration_seeds_no_import_row
  panicked: Failed to run scalar query:
  DatabaseError(Unknown, "relation \"metric_import\" does not exist")
import_error_severity_enum_has_exactly_the_approved_labels
  panicked: Failed to read enum labels:
  DatabaseError(Unknown, "type \"public.metric_import_error_severity\" does not exist")
```

The only two passes were the pure-Rust `from_str`/`Display` conversion tests,
which legitimately need no database.

**Stage 2 — minimal migration (enums, tables, PK/FK only; deliberately no
CHECK constraints and no indexes).** This proves the constraint and index
assertions are not vacuous:

```text
test result: FAILED. 26 passed; 12 failed
```

The 12 still-failing tests were exactly the constraint and index tests:
`blank_required_import_text_is_rejected`,
`negative_import_counters_are_rejected`,
`metric_import_has_exactly_the_authorized_check_constraints`,
`blank_required_import_error_text_is_rejected`,
`metric_import_error_has_exactly_the_authorized_check_constraints`,
`duplicate_upstream_report_id_is_rejected_within_a_source_account`,
`duplicate_raw_hash_and_format_version_is_rejected_when_no_upstream_report_id_is_supplied`,
`upstream_report_uniqueness_ignores_a_differing_format_version_and_raw_hash`,
`the_two_idempotency_indexes_are_partial_and_mutually_exclusive`,
`metric_import_has_no_speculative_secondary_index`,
`status_and_creation_time_have_the_required_operational_index` and one fixture
defect (an invalid `metric_platform_ownership_class` label in a locally
defined helper), which was corrected in the test module.

**Stage 3 — GREEN (authorized CHECK constraints and three indexes added).**

```text
test result: ok. 38 passed; 0 failed; 0 ignored; 1292 filtered out
```

No test was weakened, deleted or loosened to obtain green, and no runtime
claim/status-transition/idempotent-return behaviour is asserted as
implemented.

### 7.1 Focused test inventory (38 new tests)

`metric_import` (24): enum label set and order; string round-trip and
rejection of unknown/lowercase values; every status round-trips through
PostgreSQL/Diesel; no seeded row; full Diesel round-trip of every nullable
evidence, period and timestamp field, the JSON manifest and all six counters;
database defaults exercised through raw SQL rather than restated by a fixture;
blank `format_code`/`format_version`/`normalizer_version`/`created_by`
rejected across five whitespace forms; negative values rejected for all six
counters; zero and large positive counters accepted; duplicate
`(source_account_id, upstream_report_id)` rejected; the upstream path not
widened by differing `format_version`/`raw_sha256`; duplicate
`(source_account_id, raw_sha256, format_version)` rejected when no upstream
report ID is supplied; the fallback scoped to its `format_version`; the
fallback inapplicable when an upstream report ID is present; different source
accounts never collide; evidence-free imports repeatedly permitted; inverted
import periods representable; invalid source-account and publisher foreign
keys fail closed; deletion of a referenced source account or publisher
restricted rather than cascaded; the `(status, created_at)` index asserted
from `pg_indexes`; both idempotency indexes asserted unique and partial with
their exact predicates; exactly four indexes; exactly the ten authorized CHECK
constraints; exactly the two authorized foreign keys with no `ON DELETE`;
targeted revert-through and reapply.

`metric_import_error` (14): enum label set and order; string round-trip and
rejection of `INFO`/`FATAL`/`DEBUG`/lowercase; both severities round-trip
through PostgreSQL/Diesel; no seeded row; full Diesel round-trip including a
sparse finding with no row number, field name or raw value; database defaults
exercised; blank `error_code` and `message` rejected across five whitespace
forms; `row_number` deliberately unconstrained (negative, zero and large
values accepted); unknown import fails closed; deleting a parent import with
errors restricted rather than cascaded; exactly the two authorized CHECK
constraints; exactly one non-cascading foreign key; exactly one index.

## 8. Database and migration effects

- Migration source: additive schema only.
- Existing-data backfill: NONE.
- Existing canonical-data modification: NONE.
- Rows seeded: NONE.
- Production migration execution: NOT AUTHORIZED and NOT PERFORMED.

All database work used disposable local PostgreSQL 17.10 (Homebrew), UTF8
encoding with `C` collation, verified with `char_length('é') = 1`.

### 8.1 Empty-database procedure

```text
$ ./target/debug/thoth migrate          # DATABASE_URL -> disposable thoth_m03_empty
0.24s total
migrations=12  latest=20260828
import_tables=2  import_enums=2  rows=0
```

The full migration chain applies cleanly to an empty database, and the new
migration is the newest applied.

### 8.2 Populated-database procedure

A representative current-schema database was built by applying the full chain,
reverting this migration to base-equivalent state, then inserting
representative data: two publishers, an imprint, an active work with a
canonical title, a PDF publication with ISBN, an institution with a ROR, a
Publisher Services `publisher_distribution_platform` activation and a
`distribution_job`, plus a `metric_platform`, a `metric_source`, a
`metric_source_account` and a `metric_source_checkpoint` on top of the
`MET-WP1-01` seed measures.

Applying the migration inside one transaction:

```text
CREATE TYPE    0.808 ms
CREATE TYPE    0.173 ms
CREATE TABLE   3.356 ms     -- metric_import
CREATE INDEX   0.573 ms     -- upstream-report partial unique
CREATE INDEX   0.867 ms     -- raw-hash fallback partial unique
CREATE INDEX   0.564 ms     -- (status, created_at)
CREATE TABLE   1.605 ms     -- metric_import_error
COMMIT         0.463 ms
```

Total DDL time ≈ 7.9 ms.

**Locking.** Locks held on pre-existing populated relations during the
migration transaction were exactly:

```text
 AccessShareLock       | metric_source_account
 ShareRowExclusiveLock | metric_source_account
 AccessShareLock       | publisher
 ShareRowExclusiveLock | publisher
```

These are the standard locks PostgreSQL takes on a table referenced by a new
foreign key. No `AccessExclusiveLock` is taken on any populated table, and
`work`, `title`, `publication`, `institution`, `imprint`,
`publisher_distribution_platform`, `distribution_job` and the Metrics registry
tables are not locked at all. `ShareRowExclusiveLock` does briefly block
concurrent writes to `publisher` and `metric_source_account` for the
millisecond-scale duration of the transaction; it does not block reads.

**No table rewrite.** `pg_class.relfilenode` for all fourteen pre-existing
tables is byte-identical before the migration, after the migration, and after
a full down/up cycle, proving no populated existing table was rewritten.

**Data preservation.** `md5(string_agg(row_to_json(t)))` digests for all
fourteen tables — publisher, imprint, work, title, publication, institution,
`publisher_distribution_platform`, `distribution_job`, `metric_platform`,
`metric_measure`, `metric_platform_measure`, `metric_source`,
`metric_source_account`, `metric_source_checkpoint` — are identical before the
migration, after the migration, after revert, and after reapply.

### 8.3 Revert and reapply

```text
down.sql:  DROP TABLE metric_import_error
           DROP TABLE metric_import
           DROP TYPE  metric_import_error_severity
           DROP TYPE  metric_import_status
after revert:  import_tables=0  import_enums=0  wp1_01_02_tables=6
data digests:  identical to pre-migration
reapply:       0.088s -> latest=20260828  enums=2  tables=2  rows=0
enum labels after reapply:
  metric_import_status = UPLOADED,QUEUED,PROCESSING,COMPLETED,COMPLETED_WITH_ERRORS,FAILED
  metric_import_error_severity = ERROR,WARNING
data digests:  identical to pre-migration
```

The downgrade removes exactly the amendment-A5 object set in dependency-safe
order — `metric_import_error`, then `metric_import`, then
`metric_import_error_severity` and `metric_import_status` — and leaves all
`MET-WP1-01` registry schema, its two seed measures and all `MET-WP1-02`
source-state schema intact. Both enum types are absent after downgrade and
recreate cleanly with their exact labels on reapply.

`reverting_through_the_import_state_migration_removes_it_and_reapplication_restores_it`
additionally proves targeted revert-through and reapply through the embedded
Diesel harness, following the durable `revert_through_registry_migration` /
`revert_through_source_state_migration` pattern: it reverts down to and
including `20260828` rather than assuming it is the newest migration, and
asserts the measure seeds stay byte-identical across the cycle. The repository
CLI `cargo run migrate --revert` calls `revert_all_migrations` and therefore
cannot evidence single-migration rollback; the embedded harness is the
established mechanism for that, exactly as recorded for `MET-WP1-01` and
`MET-WP1-02`.

## 9. API and compatibility effects

| Contract | Impact |
|---|---|
| database/domain model | AFFECTED — owned by `thoth-pub/thoth` |
| GraphQL/API schema/behaviour | NOT AFFECTED |
| generated clients/types | NOT AFFECTED |
| authorization semantics | NOT AFFECTED |
| package capabilities | NOT AFFECTED |
| export formats | NOT AFFECTED |
| configuration/environment contracts | NOT AFFECTED |
| event/job payloads | NOT AFFECTED |
| dissemination/platform behaviour | NOT AFFECTED |
| UI assumptions | NOT AFFECTED |
| deployment compatibility | AFFECTED only as an additive future migration; no deployment authorized |

### 9.1 GraphQL SDL comparison

The public SDL is generated at build time by `thoth-client/build.rs` from
`thoth_api::graphql::create_schema().as_sdl()` into
`thoth-client/assets/schema.graphql`, which is gitignored. `git status` is
therefore not a valid "SDL unchanged" check, so both revisions were built and
the generated artefacts compared directly. A detached worktree at the exact
base `e3fa9c7dfd68512d400dcd5dd6051e9648f5b568` was built with
`cargo build --workspace` and its SDL compared byte-for-byte with the SDL
generated from the task head:

```text
521fba3b438c0013f21bfcbff62a24a3349cdd394738a40fd62e8f76fbf14226  base  schema.graphql
521fba3b438c0013f21bfcbff62a24a3349cdd394738a40fd62e8f76fbf14226  head  schema.graphql
$ cmp base/schema.graphql head/schema.graphql   # no output, exit 0
```

Both files are 177,616 bytes / 4,628 lines. The reachable public GraphQL SDL
is **byte-identical** to the exact base build, confirming this slice adds no
GraphQL type, field, query, mutation, input or enum. No generated client
update and no downstream repository change is required.

No downstream repository task is required. `thoth-sphinx` gains no GraphQL
contract and must not read these tables directly. `thoth-app` gains no upload,
history, status or error-download API.

## 10. Authorization and security effects

- New GraphQL queries/mutations: NONE.
- Authorization/policy changes: NONE; `src/policy.rs` untouched.
- Package capability mapping: NONE.
- Metrics machine roles, credentials, identity-provider provisioning: NONE.
- Secrets/provider configuration: NONE. `configuration`-style secret storage
  is not introduced by this slice, and `manifest` is generic non-secret JSON.
- Publisher-platform approval or entitlement enforcement: NONE; no approval is
  implied by the existence of these tables.
- Production secrets access: NONE.

The two new tables are unreachable from any API surface in this slice, so the
change introduces no new authorization boundary and cannot broaden access.

## 11. Rollout and rollback

Repository integration: merge only into `feature/metrics`, never directly to
`develop`, and only after fresh independent exact-head source approval and
explicit CTO merge authorization bound to that reviewed head. None of that is
requested or granted by this report.

Runtime rollout: none. Merging must not run production migrations, create
import rows, activate upload/claim/ingestion behaviour, provision credentials,
deploy Sphinx or alter app behaviour.

Rollback: before later dependent WP1 migrations merge, rollback is a
separately authorized revert of this bounded child integration plus use of the
tested down migration in disposable or non-production environments only. Once
later Metrics migrations depend on this import schema, use dependency-aware
reverse-order rollback or a separately reviewed forward-repair plan instead of
reverting it in isolation. No production rollback is authorized here.

## 12. Known limitations and deferred work

1. This is a schema and domain-type foundation only. Import lifecycle
   transitions, worker claiming, leases, retries, stale-claim recovery, queue
   semantics, counter mutation and the "return the existing import" duplicate
   path are deliberately unimplemented and belong to later WP2/WP3 slices.
2. The database enforces idempotency uniqueness, but nothing yet requires an
   import to carry idempotency evidence before it is queued or processed. That
   rule belongs to the later upload/claim API, exactly as specified.
3. `format_code`, `format_version` and `normalizer_version` are free text with
   only a non-blank constraint. No format registry or version grammar is
   approved yet.
4. `row_number` semantics and the error-code registry are intentionally
   undefined here and belong to later per-format normalizer contracts.
5. WP1 remains `IN PROGRESS`. This slice does not complete WP1 and does not
   authorize the next Metrics slice.
6. The disposable PostgreSQL cluster hosting the first validation pass was
   destroyed mid-run by the operating system's temporary-directory reaper
   (files under `/private/tmp` older than three days were deleted, including
   `global/pg_filenode.map`, after which the server shut down). This was an
   environmental failure, not a code defect: a fresh PostgreSQL 17.10 UTF8
   cluster was created and the complete migration, revert, reapply,
   populated-data and full test evidence recorded in this report was
   regenerated on that intact cluster.

## 13. Unresolved issues

None blocking. No HOLD or BLOCKED condition was triggered: the base and
`develop` checkpoints never moved, the migration path resolved and remained
free, the write budget was sufficient, no dependency or toolchain change was
needed, both idempotency paths were representable with current
PostgreSQL/Diesel patterns, and workflow side effects were unchanged.

## 14. Agent self-assessment

This report is a record, not an approval. The implementing agent does not and
may not approve its own work. `MET-WP1-03` requires a fresh independent
exact-head source review and separate CTO merge authorization, and separately
requires explicit authorization before any pull request is created and before
the PR-triggered GHCR staging publication may occur.
