# MET-WP1-05 Implementation Report

## 1. Repository state

Owning GitHub issue: [#875](https://github.com/thoth-pub/thoth/issues/875)
Repository: `thoth-pub/thoth`
Workflow: PROGRAMME_INTEGRATION
Base branch: `feature/metrics`
Authorized base commit: `fcd5a1ecc988ad32ea2057d507259e451a8c9aaf`
Actual base commit: `fcd5a1ecc988ad32ea2057d507259e451a8c9aaf` (verified identical immediately before branch creation)
Observed `develop` checkpoint: `4546cb632428872b961ad6c17282984d298e3ade` (verified identical; no programme refresh required)
PR target: `feature/metrics`
Programme integration branch: `feature/metrics`
Task branch: `feature/metrics--wp1-coverage`
Head commit: recorded in the post-push #875 completion comment for the exact pushed head; this correction is committed as the fourth commit on the branch
Pull request: [#876](https://github.com/thoth-pub/thoth/pull/876) — **DRAFT**, open, targeting `feature/metrics`
Expected branch deletion after merge: YES
Final programme PR required: YES (`feature/metrics -> develop`, separately gated and not implied here)
Implementing model: Claude Sonnet 5

Controlling records read in full before implementation: issue #875; fresh
independent specification review comment `5483297437` (APPROVED); CTO
specification approval / implementation-hold comment `5483299819`; the
exact-SHA/migration-path-bound implementation authorization delivered
directly in chat by the repository owner (satisfying the six HOLD
conditions in `5483299819`).

**Control-record correction.** An implementation-authorization/handoff
comment was attempted before any write, via `gh api ... -f
body=@<local-path>`. That invocation did not read and substitute the file's
contents: comment
[`5491893730`](https://github.com/thoth-pub/thoth/issues/875#issuecomment-5491893730)
persisted only the literal local scratchpad path string, not the intended
authorization-handoff content. This was not discovered until raised by the
repository owner, after implementation had already proceeded on the (false)
assumption that the durable comment had posted correctly. The corrective
#875 reconciliation comment authorized alongside this correction records
the authorization content that comment `5491893730` should have carried,
and this report is corrected accordingly. No implementation content, file,
migration or test is affected by this control-record defect: the
authorization itself was granted directly in chat and independently
re-verified against live repository state before the first write, as
described above and in section 5.

## 2. Scope confirmation

Approved specification: the exact #875 written specification, independently
reviewed in `5483297437` and CTO-approved (specification only, implementation
held) in `5483299819`; the HOLD was lifted by the chat-delivered
implementation authorization, which supplied the six items the CTO approval
required (fresh SHA/version/path re-verification, exact unused migration
path binding, rebound write budget, and an explicit action-enumerated
authorization).

Implemented objective: the additive, initially inactive Metrics coverage
foundation — `metric_coverage`, the closed `metric_coverage_status` enum,
the manually maintained Diesel schema contract, the matching Rust
persistence/domain module and focused database tests.

Out-of-scope changes made: NONE.

## 3. Commits

- `fd61e4eba6787b98f5572311131fb15dd2d5ce6b` - MET-WP1-05: establish Metrics coverage foundation
- `2e51ed54c552b548c2a75b51af02e38283bf9283` - MET-WP1-05: record truthful tracker consequence
- `80849c9c4b92652c36ca5a2953e9430319b8b82c` - MET-WP1-05: add the implementation report
- fourth commit - MET-WP1-05: correct the malformed pre-implementation handoff comment reference (this corrective commit); its exact SHA and the resulting final head/tree are recorded in the post-push #875 completion comment, not restated here

## 4. Files changed

Authorized write paths (existing files):

- `CHANGELOG.md`
- `docs/metrics/task-status.md`
- `thoth-api/src/schema.rs`
- `thoth-api/src/model/mod.rs`

Authorized new-file paths:

- `thoth-api/migrations/20260901_v1.9.0/up.sql`
- `thoth-api/migrations/20260901_v1.9.0/down.sql`
- `thoth-api/src/model/metric_coverage/mod.rs`
- `thoth-api/src/model/metric_coverage/tests.rs`
- `docs/engineering/ai-delivery/implementation-reports/MET-WP1-05-implementation-report.md`

Actual files changed:

- `thoth-api/src/schema.rs`
  - reason: ADR-0003 requires the manually maintained Diesel contract to change atomically with the migration.
  - behavioural effect: adds one `sql_types` struct, one `table!` block, four `joinable!` declarations and one `allow_tables_to_appear_in_same_query!` entry, in alphabetical position. No existing table, column, type, alias or ordering is altered.
  - within authorized write budget: YES
- `thoth-api/src/model/mod.rs`
  - reason: register the new model module.
  - behavioural effect: one `pub mod metric_coverage;` line added in the existing alphabetical position. No other change.
  - within authorized write budget: YES
- `CHANGELOG.md`
  - reason: repository rule that every PR updates `## [Unreleased]`.
  - behavioural effect: one bounded `### Added` bullet, placed newest-first per existing convention. No existing entry altered.
  - within authorized write budget: YES
- `docs/metrics/task-status.md`
  - reason: truthful tracker consequence required by the authorization.
  - behavioural effect: adds the `MET-WP1-05` row stating truthfully that the slice is implemented on its branch with draft PR #876 open and **not merged**; keeps WP1 `IN PROGRESS`; updates the "Last updated" header and the WP1 summary row accordingly. No transient review/CI identifier is copied in beyond the durable issue/PR numbers.
  - within authorized write budget: YES

Actual new files created — all five listed above, each within the authorized new-file list: YES.

Files deleted, moved or renamed: NONE.

### 4.1 Write-budget compliance

PASS. `git diff --name-status fcd5a1ecc988ad32ea2057d507259e451a8c9aaf..HEAD`
is exactly the nine authorized paths (four modified, four created by the
first two commits, plus this report). No deletion, rename or copy appears.

## 4.2 Authorized actions actually used

- repository and GitHub read inspection: USED
- durable implementation-authorization/handoff comment on #875: ATTEMPTED before any write but posted malformed (comment `5491893730` persisted only a local file-path string, not its intended content); corrected by the reconciliation comment authorized alongside this correction — see the control-record correction note in section 1
- branch creation (`feature/metrics--wp1-coverage` from `fcd5a1ec…`): USED
- source/worktree edit (within write budget): USED
- new file creation (authorized paths only): USED
- file deletion/move/rename: NOT USED (not authorized)
- local disposable PostgreSQL 17 migration and database validation: USED (disposable local databases only, plus the shared local `thoth_test` development database used by the existing test-suite harness)
- local compilation, tests, linting, formatting, diff/schema/GraphQL compatibility verification: USED
- commit: USED (multiple bounded commits)
- push (ordinary, non-force): USED
- creation of a DRAFT PR targeting `feature/metrics`: USED (#876)
- automatic GitHub Actions caused by that PR: OBSERVED (see 4.3)
- automatic external registry write (PR staging image): OBSERVED as triggered; not independently confirmed complete (see 4.3)
- issue/comment mutation beyond the one authorized handoff comment: NOT USED (not authorized)
- manual CI dispatch/rerun/cancel: NOT USED (not authorized)
- provider/runtime read/write: NOT USED (not authorized)
- staging/production migration execution: NOT USED (not authorized)
- release/tag/publication, merge, deployment, production activation, branch deletion, `feature/metrics -> develop` integration: NOT USED (not authorized)
- other: an isolated `git worktree` at the exact authorized base commit (for the GraphQL SDL comparison), and disposable/local PostgreSQL 17 databases on the same local Postgres 17.10 cluster used elsewhere in this session, both purely local.

Unauthorized actions performed: NONE.

## 4.3 Automatic and manual external effects

Automatic CI effects observed: YES. PR #876 triggered `build-test-and-check`
(`classify`, `format_check`, `build`, `test`, `lint`), `check-changelog`,
`run-migrations` and `publish-to-dockerhub` (`classify` +
`build_and_push_staging_docker_image`), matching the authorization's
explicit acknowledgement of the automatic
`ghcr.io/thoth-pub/thoth:staging-pr-876` publication. At report time
`classify`, `check-changelog` and `format_check` had completed
successfully; `build`, `test`, `lint`, `run_migrations` and the staging
image push were still in progress. This automatic behaviour was expected
and pre-acknowledged in the implementation authorization; no result is
claimed here beyond what was directly observed, and full CI status must be
independently re-checked before review (see section 11).

Manually initiated external actions: NONE beyond the one authorized handoff
comment, the authorized push and the authorized draft-PR creation.

External writes/publication: the automatic staging container publish
described above; nothing else.

## 5. Implementation decisions

1. **No new coverage-specific bibliographic or Publisher Services fixture
   dependency.** `metric_coverage` references only `metric_source_account`,
   `metric_import`, `metric_platform` and `metric_measure` — all already
   established by MET-WP1-01/02/03. It has no `work`/`publication`/
   `institution` FK, matching the approved field list exactly (no such
   column exists in the design).
2. **`country_coverage`/`institution_coverage` are plain booleans, not the
   status enum.** They record whether the reported coverage includes each
   dimension, not how complete it is; conflating them with
   `MetricCoverageStatus` would have invented a cross-dimension semantic the
   design does not define. Both are non-null; a test asserts NULL is
   rejected for `coverage_status`, `country_coverage` and
   `institution_coverage` independently.
3. **No coverage uniqueness or secondary index.** The approved design and
   both review comments explicitly leave coverage uniqueness and any
   coverage-specific access pattern to later WP2/WP4 semantic work. A test
   asserts the index set on `metric_coverage` is exactly `{pkey}` and the
   CHECK-constraint set is exactly `{metric_coverage_period_check}`, so a
   later speculative addition fails the suite rather than passing silently.
4. **Migration path resolved fresh, not assumed.** Immediately before any
   write, `make migration`'s logic (`MAJOR.MINOR` from workspace version
   `1.8.0` plus today's date) was reproduced by hand and independently
   confirmed against `Makefile`: `new_minor = 8 + 1 = 9`, `DATE = 20260901`,
   giving exactly `thoth-api/migrations/20260901_v1.9.0`, which was absent.
   This is the exact path the CTO's HOLD gate required to be freshly bound
   before any implementation write.
5. **Tracker truthfulness.** `docs/metrics/task-status.md` records
   `MET-WP1-05` as implemented on its slice branch with draft PR #876 open
   and explicitly **not** merged, because it is not, and keeps WP1
   `IN PROGRESS`.
6. **Model/test structure mirrors MET-WP1-04's precedent exactly**: a closed
   `strum`+`diesel_derive_enum` status enum, a `diesel::Queryable` struct, a
   `pub(crate) mod tests` gated on `#[cfg(all(test, feature = "backend"))]`,
   and fixture/helper functions (`insert_coverage_fixture`,
   `insert_coverage_row`, `foreign_keys`, `index_names`,
   `check_constraint_names` reused from `metric_import::tests`) following
   the same shape as `metric_record`/`metric_record_revision`, so the same
   review vocabulary and assertions apply.

Deviations from the specification requiring authorization: NONE.

## 6. Database and migration effects

Migration added: YES.

- migration files: `thoth-api/migrations/20260901_v1.9.0/up.sql`,
  `thoth-api/migrations/20260901_v1.9.0/down.sql`.
- schema effect: one closed enum; one new table; four foreign keys, none
  cascading; one CHECK constraint; one index (the primary key). Nothing
  existing is altered.
- existing-data effect: NONE. No backfill, no update, no delete, no seed row.
- locking: the migration is `CREATE TYPE` + `CREATE TABLE ... REFERENCES`,
  which for a brand-new child table takes only the ordinary
  `AccessShareLock` on each of the four referenced pre-existing tables
  (`metric_source_account`, `metric_import`, `metric_platform`,
  `metric_measure`) — PostgreSQL's normal lock for validating a new foreign
  key's referenced side when the new table itself, not the referenced
  table, is being altered. It takes **no** `AccessExclusiveLock` and no
  lock at all on any other existing table. Across every apply/revert/reapply
  cycle run in this session (empty database and representative populated
  database, several repetitions), each `diesel migration run`/`revert`
  invocation completed in the low single-digit milliseconds per statement,
  consistent with WP1-04's precedent finding of no table rewrite.
- empty database result: the full current migration chain — including
  `20260901_v1.9.0` — applied cleanly to an empty disposable PostgreSQL
  17.10 database (`CREATE DATABASE ... ENCODING 'UTF8' TEMPLATE template0
  LC_COLLATE 'C' LC_CTYPE 'C'`, avoiding the SQL_ASCII trap). Verified
  afterwards: the enum present with exactly the three approved labels in
  order; the table present with the approved columns, types, nullability
  and defaults; four non-cascading foreign keys; one CHECK constraint; one
  index (the primary key); zero rows.
- populated database result: see section 10.
- rollback/forward repair: `down.sql` was executed against both the empty
  and the representative populated database and reapplied afterwards. It
  removes `metric_coverage` then `metric_coverage_status`, leaving every
  earlier Metrics slice and the bibliographic/Publisher Services schema in
  place. The Diesel-harness path is additionally covered by
  `reverting_through_the_coverage_migration_removes_it_and_reapplication_restores_it`,
  which ran successfully against the shared local `thoth_test` development
  database (see section 9).
- idempotency: pure additive DDL executed once by the Diesel ledger; no data
  mutation, so repeat execution is not a scenario. Revert-then-reapply was
  proven to restore the identical object set on both the empty and the
  populated database.

### 6.1 Manual scratch-database incident (transparency note)

While manually probing lock behaviour on the disposable
`thoth_wp105_pop` database (a database created solely for this task's own
local validation, never pushed, never shared, never touched by any other
process), a `diesel migration run` invocation unexpectedly re-applied and
then auto-rolled-back the **already-merged** `20260831_v1.9.0`
(MET-WP1-04) migration on that one disposable local database, after the
ledger and the physical `metric_coverage`/`metric_coverage_status` objects
on that database had drifted out of sync during earlier manual
revert/reapply steps. This was diagnosed, and the disposable database was
reconciled (the orphaned objects dropped, the full chain cleanly
reapplied) before the database was discarded entirely. At no point were
the immutable `thoth-api/migrations/20260831_v1.9.0/**` **files** read
for anything other than inspection, modified, or otherwise touched — `git
diff --stat` for the whole session confirms zero changes outside the
authorized write budget, and this incident is reported here purely for
transparency about local, disposable, already-discarded database state,
not as a deviation from the write budget or migration-file immutability
requirement.

## 7. API and compatibility effects

GraphQL/API changes: NONE. The persistence enum is deliberately not a
`juniper::GraphQLEnum` and no resolver, type, query, mutation or input was
added.

Generated schema/client updates: NONE required. The public SDL generated by
`thoth-client/build.rs` is **byte-identical** to the exact base build:

```text
BASE  sha256 091e11f293132fdec784de420e3addf251f5020ba7e387889b292a066be15d8e  (178270 bytes)
AFTER sha256 091e11f293132fdec784de420e3addf251f5020ba7e387889b292a066be15d8e  (178270 bytes)
```

The base artefact was produced by building an isolated `git worktree` at the
exact authorized base commit `fcd5a1ecc988ad32ea2057d507259e451a8c9aaf`
(`cargo build --workspace` — a single-package `-p thoth-client` build fails
independently of this change, because thoth-api's `backend` feature is only
enabled through workspace feature unification, a pre-existing environmental
characteristic, not a regression); the after artefact by the normal
workspace build of the final implementation. `diff` reports no difference.
The SDL contains zero occurrences of `MetricCoverage`.

Backwards compatibility: fully additive and inactive.
Deprecations: NONE.
Cross-repository dependencies: NONE. `thoth-sphinx`, `thoth-app`,
`metrics-dashboard`, `metrics-widget`, `thoth-client` and
`thoth-dissemination` consume GraphQL/export contracts, which are unchanged.
No downstream repository task is required, and no downstream consumer may
read this table directly.

## 8. Authorization and security

Authorization paths changed: NONE. `thoth-api/src/policy.rs` is untouched.
Roles/scopes involved: NONE. No machine role, credential or entitlement is
introduced.
Negative authorization tests: not applicable — this slice exposes no
protected operation. The existing 13 `graphql_permissions` integration
tests still pass unchanged.
Secret or personal-data handling: NONE. No credential, token or personal
data is stored, logged or transmitted.
Security limitations: none introduced. The table is unreachable from any
runtime path in this slice.

## 9. Tests and checks

Commands were run against a local Postgres 17.10 cluster (the same one used
by this repository's existing `thoth_test` development/test database) and a
local Redis instance.

### Formatting

Command: `cargo fmt --all -- --check`
Result: exit 0, no diff reported (after one `cargo fmt --all` pass to fix
two long-line wraps in the new test file — no logic change).

### Integration/database tests

Command: `cargo test -p thoth-api --features backend`
Result: `1415 passed; 0 failed; 0 ignored` (finished in ~230–279s across
runs) — `1400` pre-existing + `15` new `metric_coverage` tests; plus
`13 passed; 0 failed` (`graphql_permissions`); `0 passed; 8 ignored`
(doc-tests).

Command (just the 15 tests added by this slice):
`cargo test -p thoth-api --features backend metric_coverage`
Result: `15 passed; 0 failed; 0 ignored; 1400 filtered out`.

### Unit tests (full workspace)

Command: `cargo test --workspace`
Result: all suites `ok`, 0 failures —
`thoth` (bin) 31 passed;
`thoth-api` (lib) 1415 passed;
`graphql_permissions` 13 passed;
`thoth-api-server` 3 passed;
`thoth-client` 4 passed;
`thoth-errors` 11 passed;
`thoth-export-server` 144 passed;
doc-tests: `thoth_client` 6 passed, `thoth_api` 8 ignored, `thoth_export_server` 2 passed, others 0;
overall exit 0.

### Lint/static analysis

Command: `cargo clippy --all --all-targets --all-features -- -D warnings`
Result: exit 0, no warnings (one pre-existing unrelated future-incompat note
for `proc-macro-error2 v2.0.1`, also present on the base, matching WP1-04's
own report).

### Other required checks

Command: `cargo check --workspace` — exit 0.
Command: `git diff --check` — exit 0, no whitespace errors.
Command: `git status --short ; git diff --stat` (against the authorized
base) — working tree clean after each commit; the combined diff across the
first two commits is 8 files changed, 897 insertions(+).

### What the new tests prove

The 15 tests assert schema behaviour only. Coverage includes: the closed
`metric_coverage_status` enum's three labels round-tripping through
PostgreSQL/Diesel and through string conversion, with unknown/lowercase
values rejected; a full row round-trip through Diesel including nullable
`notes`; database defaults (`coverage_id`) exercised through raw SQL rather
than restated by fixtures; nullable `notes` round-tripping both unset and
set; half-open period ordering enforced (inverted and empty periods
rejected, a correctly ordered period accepted); `coverage_status`,
`country_coverage` and `institution_coverage` independently rejecting NULL;
all four foreign keys (`source_account_id`, `import_id`, `platform_id`,
`measure_id`) failing closed on an unknown reference; referenced-row
deletion restricted rather than cascaded for all four references, with the
coverage row surviving; the complete authorized CHECK-constraint set (one);
the complete authorized foreign-key set (four, all non-cascading); the
complete index set (exactly the primary key, no speculative index); and
targeted revert-through/reapply of the coverage migration via the embedded
Diesel harness, leaving the MET-WP1-01/02/03/04 schema and the two measure
seed rows byte-identical.

## 10. Manual verification

Environment: local PostgreSQL 17.10 (Homebrew), plus disposable databases on
the same cluster created and used only for this task. No staging,
production or shared external database was contacted.

**A. Empty full-chain apply.** Applied the complete current migration chain
including `20260901_v1.9.0` to a freshly created empty UTF8/C database via
`diesel migration run`. Result: success; table, enum, FKs, CHECK and index
verified exactly as specified in section 6.

**B. Targeted revert/reapply (empty database).** `diesel migration revert`
removed exactly `metric_coverage` and `metric_coverage_status`; `diesel
migration run` restored them, with zero rows. Repeated successfully.

**C. Representative populated database.** Built a second disposable
database carrying representative existing state, applied up through
`20260831_v1.9.0` (the pre-coverage state), then inserted: 2 publishers, 2
imprints, 2 works (one active with a DOI, one forthcoming), 2 publications
(PDF, Paperback; one with an ISBN), 1 ROR-backed institution; 1 Publisher
Services `publisher_distribution_platform` activation; MET-WP1-01 registry
state (1 platform, 1 platform-measure mapping, both seeded measures);
MET-WP1-02 source state (1 source, 1 source account, 1 checkpoint);
MET-WP1-03 import state (1 import, 1 import error); MET-WP1-04 canonical
history (1 record, 1 current revision, 1 provenance row).

Evidence method: for each of the 17 pre-existing populated tables, an
order-independent MD5 over every row rendered as JSON, the row count, and
`pg_class.relfilenode` (which changes if and only if PostgreSQL rewrites
the table).

Observed result:

```text
apply  : every one of the 17 populated tables byte-identical (row count, content
         MD5, relfilenode) before vs after applying 20260901_v1.9.0. The only
         textual difference in the raw psql transcript was \timing noise.
revert : snapshot byte-identical to the pre-apply snapshot. metric_coverage
         absent; all MET-WP1-01/02/03/04 tables and rows present and unchanged.
reapply: metric_coverage recreated empty; 4 FKs, 1 CHECK, 1 index restored.
```

(A separate, transparently reported scratch-database mishap during manual
lock probing on this same disposable database, and its reconciliation, is
recorded in section 6.1; it affected only local disposable database state,
not the evidence above, which was captured before that incident, nor any
migration file.)

Locking behaviour is as described in section 6 (ordinary
`AccessShareLock`-only referenced-table locking for a new child table's
foreign keys; no `AccessExclusiveLock` on any existing table).

## 11. CI

CI status: PR #876 is open and DRAFT. At report time: `classify`,
`check-changelog` and `format_check` completed `SUCCESS`; `build`, `test`,
`lint`, `run_migrations` and the staging Docker image publish were
`IN_PROGRESS`. This automatic behaviour — including the
`ghcr.io/thoth-pub/thoth:staging-pr-876` publication — was explicitly
pre-acknowledged in the implementation authorization. **Reviewers must
independently re-check the final CI status** (`gh pr checks 876`) before
approving, since this report captures an in-progress snapshot, not a final
result.
Failures or warnings observed so far: NONE.

## 12. Rollout and rollback

Initial state after any later authorized merge to `feature/metrics`: the
schema exists and is completely inactive. No runtime path reads or writes
this table.
Activation required: YES, by later WP2/WP4 coverage-calculation and
GraphQL/rollup slices, each separately specified and authorized.
Feature flag/configuration: none; inactivity is structural, not flagged.
Migration sequence: this migration must run after `20260831_v1.9.0`
(MET-WP1-04) in the current chain, because it depends on `metric_import`,
`metric_platform`, `metric_measure` and `metric_source_account` all being
present (none of which MET-WP1-04 introduced or changed — those are
MET-WP1-01/02/03 objects); the Diesel ledger enforces the order.
Rollback/disable procedure: before any later dependent Metrics migration
merges, rollback is a separately authorized revert of this bounded child
integration plus, in disposable or non-production environments only, the
tested down migration. **No production rollback is authorized by this
task.**
Monitoring required: none while inactive.

## 13. Known limitations and deferred work

- No coverage uniqueness constraint exists beyond the primary key: whether
  a source account may report overlapping or duplicate coverage for the
  same platform/measure/period is a later WP2/WP4 semantic question, by
  design.
- No coverage-specific secondary/access index exists; the approved design
  defines no access pattern for this foundation stage.
- `country_coverage`/`institution_coverage` record dimension presence only;
  no relationship is enforced between them and any future coverage-quality
  computation.
- No relationship is enforced between `metric_coverage` and
  `metric_record`/`metric_record_revision`; this slice does not implement
  coverage calculation, finalization or zero-versus-unknown query
  behaviour.
- Migration timings were measured on local disposable/development databases
  holding at most a handful of rows. Because the migration provably
  performs no table rewrite and acquires no `AccessExclusiveLock` on any
  existing table, the cost is expected to remain small at production scale,
  but that expectation is not production evidence and no production
  migration is authorized.
- Section 6.1 records a local, disposable, already-discarded
  scratch-database mishap during manual lock investigation; it affected no
  migration file and no shared or persistent database.

## 14. Unresolved issues

NONE.

## 15. Agent self-assessment

The implementing agent may identify risks but may not approve this task.
This report is implementation evidence, not self-approval.

Suggested review focus:

- Whether `country_coverage`/`institution_coverage` as plain booleans
  (rather than, say, an enum or a nullable indicator) is the reviewer's
  intended reading of the approved design's field list.
- The completeness and closure of the asserted CHECK, foreign-key and index
  inventories (one, four, one respectively), since those tests are what
  prevent a later speculative addition from passing silently.
- The `docs/metrics/task-status.md` wording, specifically that `MET-WP1-05`
  is recorded as implemented but **not merged**, with draft PR #876 open,
  and that WP1 remains `IN PROGRESS`.
- Final CI status on PR #876 (`build`, `test`, `lint`, `run_migrations`,
  staging image publish), which was still in progress when this report was
  written.
- Section 6.1's scratch-database transparency note, to confirm the reviewer
  agrees it constitutes no deviation from the write budget or from
  migration-file immutability.
