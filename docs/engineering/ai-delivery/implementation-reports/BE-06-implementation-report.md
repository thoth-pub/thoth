# BE-06 Implementation Report

Work-level Crossref upsert and Crossref write permits.

The implementation agent does not approve its own work. This report records what was implemented and what was
executed. It is evidence for the independent exact-head CRITICAL review, not a substitute for it.

> **T166 correction.** Sections 1-16 are the original report for head `bfef2daa`, preserved as written. That head
> was not approved on independent review. Section 17 records the correction authorized by #848 comment `5701857879`
> under CTO specification correction `5701572954`, which supersedes section 5.4 item 1.
>
> **Post-review correction.** Section 17's head `aabb5e0d` was not approved on independent review either; sections
> 1-17 are preserved as written. Section 18 records the correction authorized by #848 comment `5703254463` under CTO
> specification correction `5703204194`.
>
> **Merge-readiness correction.** Section 18's head `86917f9f` was approved on independent review (#848 comment
> `5715923725`) and PR #925 was opened. The merge-readiness review then required changes; sections 1-18 are preserved
> as written. Section 19 records the correction authorized by #848 comment `5718898530`.

## 1. Repository state

| Item | Value |
|---|---|
| Owning GitHub issue | [thoth-pub/thoth#848](https://github.com/thoth-pub/thoth/issues/848) |
| Repository | `thoth-pub/thoth` |
| Workflow | PROGRAMME_INTEGRATION |
| Authorized base branch | `feature/publisher-services-v1-10` |
| Authorized base commit | `b23ba05c7d5e2e3f4909ed54ab05b0e8ed2fccc3` (tree `1058e80091a3cca93f5775778e0bfae47695c2e6`) |
| Actual base commit | the same; re-verified at completion: `origin/feature/publisher-services-v1-10` = `b23ba05c…`, tree `1058e800…` |
| Task branch | `feature/publisher-services-v1-10--be-06` |
| Eventual PR target | `feature/publisher-services-v1-10` |
| Pull request | none; PR creation is not authorized |
| Implementation authorization | #848 comment `5687443066` (sole current authorization) |
| Verified source head | `c4411993d8d396ba9da858d354b6e94ee2a788e9` (tree `d96ada8a03bcefa90faf65dd0a6ca901c8a52245`) |
| Report commit | the commit that adds this file, a documentation-only child of `c4411993`; its SHA is the pushed head |
| Implementing model | Claude Opus 5 (`claude-opus-5`) |

Every test result in section 9 was executed on the source at `c4411993`. The commit adding this report changes only
this file. The diff-dependent gates (S1, the allowlist gate, `cargo fmt --check`) were re-run on the report commit
before the push.

## 2. Scope confirmation

Approved specification: `docs/publisher-services/specifications/BE-06-R52B.md` (SHA-256
`584683ca02611afb064335db83d1fb22685fc3dae87f4eeeee2e1490b320f3ce`, blob `cabeddb8`, 699,166 bytes, 5,107 lines), as
modified by #848 Amendments 1-3. The Amendment 3 artifact used has SHA-256
`d9c04dec84945f3e1ff13a9101372cd65b964b466e5c02ca34da25982e4b0831` (242,281 bytes, 1,121 lines), verified before use.
The CTO R-3 decision is #848 comment `5686341130`, and the Amendment 3 approval is `5686347512`. The persisted R52B
file is unchanged.

Implemented objective: R52B as modified by Amendments 1-3. That is:
- Alternative A commit-time capture with one ascending flush per transaction;
- the scheduler-independent durable residue;
- `seed -> exact generation-coverage census -> exact-binding admission -> drain`;
- the three generic creation sites and one creation helper;
- the `P -> W -> G -> J -> A` lock order with the execution gate `Q`;
- the fence, deletion and recovery invariants;
- the Crossref permit state machine, with its four routes, finalisation, report, voids, reconciliation and clearance;
- the database-local version floor;
- the 17 mutations and 16 query fields;
- the scoped database-error boundary EB1-EB4;
- the export server's prepared deposit route.

Out-of-scope changes made: NONE. No `thoth-dissemination`, `thoth-app` or `thoth-client` change, no Metrics path,
and no G-6, G-7, #919 or BE-07 work.

## 3. Commits

All commits are authored and committed as Javier Arias `<javier@jarias.org>`, with no AI attribution trailer.

| # | SHA | Message |
|---|---|---|
| 1 | `e155c192` | docs: add BE-06 implementation task record |
| 2 | `05826bdb` | feat(api): add BE-06 migrations, schema and protected test reset |
| 3 | `0c79ca5e` | feat(errors): add BE-06 error codes and scoped database-error conversion |
| 4 | `52719fb7` | feat(api): add BE-06 profile registry, input policy and Generation scalar |
| 5 | `24dd146f` | feat(api): add the BE-06 work-level transaction error boundary |
| 6 | `344aff8e` | feat(api): add BE-06 work-level control transitions |
| 7 | `c862fe3c` | feat(api): add Crossref eligibility, census and the bounded seed |
| 8 | `d0decdeb` | feat(api): add BE-06 Crossref admission and the admissions report |
| 9 | `f3839328` | feat(api): add the BE-06 materialization unit and single creation helper |
| 10 | `73b2d52f` | feat(api): add the BE-06 materialization drain |
| 11 | `79f08e36` | fix(errors): use the fixed BE-06 version-floor refusal messages |
| 12 | `6c821504` | feat(api): bind the legacy claim to its kinds and add the work-level claim |
| 13 | `bce7d772` | feat(api): add the four BE-06 Crossref write reservations |
| 14 | `9b71265e` | feat(api): add BE-06 Crossref write finalisation |
| 15 | `aa48fc38` | feat(api): add BE-06 Crossref outcome reports, voids and reconciliation |
| 16 | `9bf95ecf` | feat(api): add the BE-06 Crossref version floor advance |
| 17 | `12a091d2` | feat(api): add BE-06 completion, failure and cancellation guards and T2 |
| 18 | `6f7a2c34` | feat(api): add the BE-06 Work, Imprint and Publisher deletion units |
| 19 | `de261371` | refactor(api): name the BE-06 Crossref eligibility clauses |
| 20 | `53abe9fb` | feat(api): add the BE-06 work-level and Crossref report model |
| 21 | `e95dc5f1` | feat(api): add the BE-06 GraphQL surface |
| 22 | `1a67d75b` | test(api): satisfy clippy in the BE-06 tests |
| 23 | `09635151` | test(api): add the BE-06 GraphQL authorization matrix and refusal order |
| 24 | `9d5b8ad9` | feat(export): add the prepared Crossref deposit route |
| 25 | `95b227e1` | BE-06 U9a: capture matrix, flush properties and section 10.5 drift guards |
| 26 | `2633632e` | BE-06 U9b: reservation, digest, projection and allowlist tests |
| 27 | `814083fa` | BE-06 U9c: race harness, T205-T208, and the fenced-cancellation snapshot fix |
| 28 | `a6431729` | BE-06 U9d: race rows T209-T223 against the implementation |
| 29 | `a17e5452` | BE-06 U9e: the permit state machine against the migration (T237-T240, T269) |
| 30 | `8f580e37` | BE-06 U9f: held-publisher and execution-gate schedules (T270-T275, T280-T289) |
| 31 | `d2c59fe3` | BE-06 U9g: drain, seed and admission races (M5, M6, M10, M12, M14, D10/T290, D11, T291, T292, R7) |
| 32 | `9a514580` | BE-06 U9h: X12 released error behaviour, F16/T167 and T203 containment |
| 33 | `a06d7305` | BE-06 U9i: the section 16.11 extractor (T242, T243) and timestamp parity (T248) |
| 34 | `dce4604f` | BE-06 U9j: serializer dependency guard (T38), membership parity (T244) and order (T245) |
| 35 | `2c424035` | BE-06 U9k: the conformance transcript (T258) with R13, R14, R15 and B4 |
| 36 | `b164b061` | BE-06 U9l: floor advance concurrency (F13) and live error-boundary leakage (X9) |
| 37 | `159dfcf3` | BE-06 U9m: deletion and capture schedules on the Alternative A schema (T325-T328) |
| 38 | `81885b31` | BE-06 U9n: timestamp and allocation rows (T249, T250, T252, T254, T255, T265) |
| 39 | `8c550ed4` | BE-06 U9o: the non-blank rule under ICU and C collation (N5) |
| 40 | `90e5771e` | BE-06 U9p: the inert state (T166) |
| 41 | `c4411993` | docs: BE-06 changelog entry, snapshot queries and task execution log |
| 42 | (report commit) | docs: BE-06 implementation report |

Two commits were amended before any push to fix a clippy finding: `09635151` replaced `b9fb22d1`, and `81885b31`
replaced a first `U9n` commit. No commit was ever pushed before the final push, and no force-push occurred.

## 4. Files changed

### 4.1 The diff

`git diff --name-status b23ba05c7d5e2e3f4909ed54ab05b0e8ed2fccc3..c4411993` (40 paths), and `git diff --numstat`:

| Status | Path | + | - |
|---|---|---|---|
| M | `CHANGELOG.md` | 1 | 0 |
| A | `docs/engineering/ai-delivery/BE-06-snapshot-queries.sql` | 198 | 0 |
| A | `docs/engineering/ai-delivery/tasks/BE-06.md` | 169 | 0 |
| A | `thoth-api/migrations/20260910_v1.10.0/down.sql` | 6 | 0 |
| A | `thoth-api/migrations/20260910_v1.10.0/up.sql` | 8 | 0 |
| A | `thoth-api/migrations/20260911_v1.10.0/down.sql` | 77 | 0 |
| A | `thoth-api/migrations/20260911_v1.10.0/up.sql` | 1001 | 0 |
| M | `thoth-api/src/graphql/distribution_job_tests.rs` | 145 | 21 |
| M | `thoth-api/src/graphql/mod.rs` | 1 | 0 |
| M | `thoth-api/src/graphql/model.rs` | 52 | 1 |
| M | `thoth-api/src/graphql/mutation.rs` | 167 | 0 |
| M | `thoth-api/src/graphql/query.rs` | 240 | 0 |
| A | `thoth-api/src/graphql/work_upsert.rs` | 849 | 0 |
| A | `thoth-api/src/model/crossref_write_permit/crud.rs` | 1571 | 0 |
| A | `thoth-api/src/model/crossref_write_permit/mod.rs` | 261 | 0 |
| A | `thoth-api/src/model/crossref_write_permit/tests.rs` | 6186 | 0 |
| M | `thoth-api/src/model/distribution_job/crud.rs` | 478 | 2 |
| M | `thoth-api/src/model/distribution_job/mod.rs` | 49 | 4 |
| M | `thoth-api/src/model/distribution_job/tests.rs` | 255 | 27 |
| M | `thoth-api/src/model/imprint/crud.rs` | 70 | 2 |
| M | `thoth-api/src/model/mod.rs` | 87 | 10 |
| M | `thoth-api/src/model/publisher/crud.rs` | 70 | 2 |
| M | `thoth-api/src/model/tests.rs` | 618 | 4 |
| M | `thoth-api/src/model/work/crud.rs` | 208 | 3 |
| A | `thoth-api/src/model/work_upsert/crud.rs` | 1456 | 0 |
| A | `thoth-api/src/model/work_upsert/mod.rs` | 461 | 0 |
| A | `thoth-api/src/model/work_upsert/policy.rs` | 204 | 0 |
| A | `thoth-api/src/model/work_upsert/registry.rs` | 69 | 0 |
| A | `thoth-api/src/model/work_upsert/tests.rs` | 7261 | 0 |
| M | `thoth-api/src/schema.rs` | 177 | 0 |
| A | `thoth-api/tests/crossref_permit_concurrency.rs` | 499 | 0 |
| A | `thoth-api/tests/crossref_serializer_dependency_guard.rs` | 511 | 0 |
| M | `thoth-api/tests/support/mod.rs` | 44 | 0 |
| A | `thoth-api/tests/work_upsert_capture.rs` | 1727 | 0 |
| A | `thoth-api/tests/work_upsert_deletion.rs` | 851 | 0 |
| A | `thoth-api/tests/work_upsert_lifecycle.rs` | 1629 | 0 |
| M | `thoth-errors/src/lib.rs` | 729 | 0 |
| M | `thoth-export-server/src/specification/handler.rs` | 129 | 1 |
| M | `thoth-export-server/src/specification/mod.rs` | 5 | 1 |
| M | `thoth-export-server/src/xml/doideposit_crossref.rs` | 899 | 33 |
| | **total** | **29,418** | **111** |

With this report the diff has **41** paths: this file is added with status `A`.

### 4.2 Write-budget compliance: PASS

- Every changed path is in the 43-path allowlist of Amendment 3 section 2, with the status its list permits: new paths
  `A` and modified paths `M`. This was checked mechanically at `c4411993`: 40 paths, 0 violations. The test `S1` also
  checks it on every run.
- There is no `D`, `R` or `C` status, with or without rename and copy detection (`-M -C --find-copies-harder`).
- No prohibited path changed. These were checked by path with `git diff --name-only` and returned 0 paths:
  `thoth-errors/src/database_errors.rs`, `thoth-errors/Cargo.toml`, `thoth-api/src/graphql/dataloader.rs`,
  `thoth-export-server/src/xml/mod.rs`, `thoth-export-server/src/record.rs`, `thoth-dissemination`, `thoth-app`,
  `thoth-client`, `.github`, `docs/superpowers` and `diesel.toml`.
- No migration directory other than BE-06's two changed.
- Authorized but unused paths: `thoth-api/src/db.rs` and `thoth-api/src/policy.rs`. Neither was needed; no change
  was made merely to make a path appear.
- Scoped paths were inspected in the diff:
  - `graphql/model.rs` adds only the six `DistributionJob` and three `DistributionJobAttempt` field resolvers and
    the `Generation` import.
  - `graphql/distribution_job_tests.rs` changes only the leakage loop (S5) and adds its negative controls (S6).
  - `model/tests.rs` changes only the reset statement, exposed as a constant, and adds the H1-H7 and H9 module.
  - `tests/support/mod.rs` changes only the reset statement, which is byte-identical (H6).

### 4.3 Authorized actions actually used

| Action | Used |
|---|---|
| repository inspection | yes |
| source edit, new files | yes, within the allowlist |
| file deletion, move or rename | no |
| branch creation | no; the local task branch already existed at the base with no commits. A local scratch branch was created and deleted during an S1 red check (section 5.3) and was never pushed. |
| commit | yes (section 3) |
| push | the task branch only, after this report is committed (section 14) |
| PR creation or update | no |
| issue or comment mutation | no |
| manual CI dispatch or rerun | no |
| provider or runtime read or write | no |
| migration execution | disposable local PostgreSQL 17 databases only |
| release, tag, publication, merge, deployment, production activation | no |

Unauthorized actions performed: NONE committed or pushed. Two self-caught local incidents are recorded in section 5.3.

## 5. Implementation decisions and deviations

### 5.1 Decisions within the approved design

1. **X3 wording.** Rule X3 forbids the literal `impl From<diesel` in the five model files, while Amendment 3 needs
   `WorkUpsertTxError::Database` to convert from Diesel's error. The files alias the type
   (`use diesel::result::Error as DatabaseError`). The intent is preserved: no released conversion is used, as X3's
   own static test asserts.
2. **The abstract clause fails closed.** The released query orders canonical abstracts with no tie-break, so the
   evaluated abstract set is a superset of what the serializer can emit: the root's canonical abstracts, plus every
   non-canonical one when fewer than two are canonical. A Work is excluded if any abstract that could be emitted
   fails to normalise.
3. **Group B on `WORK_UPSERT`** follows R52B section 11.5: an activation that differs gives `BINDING_SUPERSEDED`; an
   assignment that is not enabled gives `ASSIGNMENT_DISABLED`.
4. **C2 replay** of a finalisation-voided permit returns its recorded outcome, as R52B states, without comparing the
   new observation.
5. **Back-catalogue outer cancellation** keeps released behaviour: the cancellation proceeds and the unit permits are
   untouched (T222).
6. **Failure while `RESERVED` or `AUTHORIZED`** is refused for both kinds. `INDETERMINATE` refuses only a terminal
   close of a back-catalogue outer attempt.
7. **The EB1 and X4 static rules** are implemented as tests over source text.
8. **Over-capture of `affiliation.position` and `reference.retrieval_date`.** Both tables take whole-table triggers,
   as R52B section 8.1 permits, so these non-dependency columns advance their owners. The capture tests record this.

### 5.2 Defect found and fixed during implementation

T207 showed an administrative cancellation cancelling a job whose attempt had been fenced. The cancellation had
waited on the job row while finalisation fenced the attempt. Its single `SELECT … FOR UPDATE OF j` statement read the
attempt's `fenced_at` with the snapshot it started with. Under READ COMMITTED it therefore missed the fence committed
while it waited, and `WORK_UPSERT_CANCELLATION_REFUSED_FENCED_ATTEMPT` was not raised.

The guard in `distribution_job/crud.rs` now locks the job in one statement and reads the fence in a later statement.
The failing test was recorded first, then the fix, then green, in commit `814083fa`. The deletion unit's retirement
statement has the same shape, but it always waits at `W` in an earlier statement, so it reads a fresh snapshot. T208
and T221 confirm this in both orders.

### 5.3 Incidents, self-caught, never committed or pushed

1. **U8 prohibited path.** During U8, `thoth-export-server/src/xml/mod.rs`, a prohibited path, was edited in the
   working tree to add a re-export. The edit was reverted with `git checkout` before any commit. The need was met
   instead with an associated function in the allowlisted serializer file. The prohibited path is unchanged in
   every commit.
2. **S1 red-check probe.** During S1's red check, a local scratch branch was created and a probe commit made with
   `commit -a`. That swept uncommitted test edits into the probe commit, and deleting the branch removed them from
   the working tree. Both test files were restored from the probe commit through the reflog, and the probe's
   `README.md` change was discarded. Nothing was pushed, and the scratch branch no longer exists.

### 5.4 Deviations and specification observations requiring the reviewer's attention

1. **T166 and admission while inert.** R52B T166 lists admission among the rows that cannot exist while the control
   row is `(false, false)`. Amendment 3 section 9.3's frozen admission precedence (and R52B section 21.3) contains no
   capture condition. An explicit superuser admission therefore succeeds while inert. The implementation follows
   the frozen section 9.3 contract; adding a refusal would change a frozen API semantic. The T166 test asserts that
   released paths and every gated entry point create no BE-06 state, and records that admission remains an explicit
   superuser act. **Not implementation-blocking. Reviewer decision requested.**
   *Superseded by section 17: the CTO resolved this conflict in favour of T166 (#848 comment `5701572954`).*
2. **N5 and the non-blank CHECKs.** The database CHECKs that back the non-blank rule are weaker than the Rust rule.
   For example, the admission CHECK accepts U+0001. They also differ between ICU and C collation on inputs the code
   already refuses, such as U+00A0 and U+2000-U+200A under `[[:space:]]`. API results are identical under both
   providers, and no CHECK refuses a value the code accepts, which is what N5 requires. The CHECKs are defence in
   depth.
3. **T302, the candidate `thoth migrate --revert`.** Over a migrated database it exited 0 and reverted every
   migration, leaving one table. R52B recorded exit 1 at the first migration's `down.sql`, with 55 tables, against a
   different base. The released-binary result matches R52B exactly (exit 1, ledger and 68 tables unchanged). Both
   confirm R52B section 23.5: `--revert` is revert-all and never a BE-06 rollback.
4. **S1 on shallow checkouts.** S1 needs the base commit. The repository's CI test job checks out without
   `fetch-depth: 0`, so there S1 prints `S1 NOT EVALUATED` and asserts nothing. It evaluated fully locally. The
   completion allowlist gate was also run separately (section 4.2).

## 6. Database and migration effects

Migrations added: YES, created only by `make DATE=20260910 MAJOR=1 MINOR=9 migration` and
`make DATE=20260911 MAJOR=1 MINOR=9 migration`, which produced exactly `thoth-api/migrations/20260910_v1.10.0/` and
`thoth-api/migrations/20260911_v1.10.0/`. `schema.rs` is maintained by hand. No `diesel print-schema`,
`diesel_cli` or `diesel.toml` was used; `schema_rs_matches_the_migration_for_all_three_relations` and the lib suite
verify it.

- **Diesel-version collision evidence.** Before creation, `git ls-remote` over all live heads showed `20260910` and
  `20260911` unused, with no within-head duplicates. At completion the census was re-run over 86 live remote heads
  (`git fetch origin --prune`, then `git ls-tree` of `thoth-api/migrations` per head, with the version derived as the
  text before the first `_`, hyphens removed): `20260910` unused, `20260911` unused, no within-head duplicate
  version, no missing objects. G6 asserts the exact directories and distinct derived versions in the tree.
- **Migration 1** (`20260910`): the enum labels `WORK_UPSERT`, `BINDING_SUPERSEDED` and `WORK_DELETED`. Its down
  migration is documented as label-preserving.
- **Migration 2** (`20260911`):
  - 8 tables: 4 generic, `work_upsert_generation`, `work_upsert_capture_queue`, `work_upsert_control` and
    `work_upsert_admission`; and 4 Crossref, `crossref_write_permit`, `crossref_write_permit_doi`,
    `work_crossref_version_floor` and `crossref_version_floor_audit`;
  - 5 enum types;
  - 6 `distribution_job` columns and 4 `distribution_job_attempt` columns;
  - 27 functions and 35 triggers: 16 capture triggers, the flush, and the guards, the state machine and the
    target-set triggers;
  - the replacement of `distribution_job_work_id_fkey`;
  - the seed rows: control `('CROSSREF', false, false)` and floor `0`;
  - migration-time assertions: the 40-value DOI corpus and the 13 timestamp vectors.
- **Inventory.** G2 asserts it; green on the final source:
  - released: 60 tables, 57 user triggers, 173 indexes, 30 functions, 28 enum types;
  - migrated: 68, 92, 191, 57, 33.
  - G3 asserts the 35-name trigger manifest and the reset's 15-member subset, and G4 asserts the five enums, eight
    tables and 27 functions, with no removed object.
- **Up, down, up.** G1 runs `up M1 → up M2 → down M2 → up M2 → down M2` on the released schema. After each `down M2`
  the catalog equals the released one by name in every object class except Migration 1's three labels, and
  `distribution_job_work_id_fkey` is `ON DELETE CASCADE` again. G5 shows that both migrations in one transaction fail
  with `unsafe use of new value`. G1-G6 are green on the final source.
- **Rollback.** A BE-06 rollback is `down M2` only. `thoth migrate --revert` is revert-all (section 5.4 item 3).
- **Existing data and locking.** Migration 2 adds columns and tables and replaces one foreign key. The compatibility
  window (section 9.9) ran the released suite and `thoth migrate` against the migrated schema.
- **Inert on migration.** No job, target, attempt, admission, permit or audit row is created. Only capture advances
  generation rows (T166, T296).

## 7. API and compatibility effects

### 7.1 GraphQL

The built SDL gains:
- **17 mutations** (`MutationRoot` 98 → 115; S3): `materializeWorkUpsertJobs`, `claimWorkUpsertJobs`,
  `reserveWorkUpsertCrossrefWrite`, `reserveBackCatalogueCrossrefWrite`, `reserveLegacyScheduledCrossrefWrite`,
  `reserveManualRecoveryCrossrefWrite`, `finaliseCrossrefWrite`, `reportCrossrefWrite`,
  `voidCrossrefWriteReservation`, `voidCrossrefWriteReservationAsSuperuser`, `reconcileCrossrefWritePermit`,
  `enableWorkUpsertCapture`, `setWorkUpsertExecution`, `seedCrossrefWorkUpsert`, `admitCrossrefWorkUpsert`,
  `advanceCrossrefVersionFloor`, `materializeWorkUpsertJob`.
- **16 query fields** (`QueryRoot` 88 → 104; S3): `workUpsertResolution`, `workUpsertResolutionCount`,
  `workUpsertResidue`, `workUpsertStaleBindings`, `workUpsertJobs`, `workUpsertJob`, `workUpsertAttempts`,
  `workUpsertBlockedByRecovery`, `workUpsertCaptureLag`, `workUpsertAdmissions`, `workUpsertControl`,
  `crossrefWritePermits`, `crossrefUnresolvedPermits`, `crossrefVersionFloor`, `crossrefBlockingWritePermitCount`,
  `crossrefDrained`.
- **Fields on released types:** six on `DistributionJob` and three on `DistributionJobAttempt` (S2).
- **New types and enums:** exactly those Amendment 3 section 4.3 freezes, with no placeholder (S4, B1, B5, D1, A8).

The SDL has no removed baseline vocabulary (S9, A1). The only leakage exemption is `executionProfile` on
`DistributionJob` (S5, S6). `reservationToken` is an output of `CrossrefWriteReservation` only, and an input of the
finalise, report and route-owner-void inputs only (S7, S8/T170).

### 7.2 OpenAPI

The export server's generated `openapi.json` was captured from the local `v1.8.0` and candidate binaries, bound to
`127.0.0.1` with the GraphQL endpoint pointed at an unreachable local port. Added paths:
`/specifications/{specification_id}/work/{work_id}/prepared/{crossref_timestamp}` (`GET`, tag `Specifications`, three
path parameters). Removed paths: none. Changed paths: none.

### 7.3 Compatibility

- The released `claimDistributionJobs` refuses `WORK_UPSERT` and binds an empty kind list to the legacy kinds
  (T163-T164).
- The released back-catalogue coordinator, claim, completion and cancellation behave as released (the full lib
  suite and T296).
- The released public export route keeps its 14-digit timestamp, and its cache is unchanged.
- **Cross-repository.** `thoth-dissemination` #106 (DIS-04) consumes the contract. The job-call table and the
  protocol-stop set are fixed in the conformance client (R15). No downstream repository was changed. `thoth-client`
  is unchanged; its query document is read by the serializer guard only.

## 8. Authorization and security

- **Authorization order.** Every BE-06 mutation checks, in order: the coarse role, route metadata read by MVCC, the
  route-derived role, the operation's own input, then protected state (Amendment 3 section 10.1). T257 covers five
  principals (anonymous, worker-only, superuser-only, both roles, publisher-scoped) across the 17 mutations, the
  superuser-only reports and the route-derived operations on three routes. P4, N1, F17, C6 and the replay, repeat
  and role-removal cases are covered too.
- **Information disclosure.**
  - No refusal carries SQL, constraint, index, trigger, SQLSTATE, driver, host, port or role text. This is shown for
    every live provocation through the scoped conversion and GraphQL (X9), for a pool that cannot connect and for a
    held pool (X10), and by static message scans (R11, F20).
  - `CROSSREF_PERMIT_ATTEMPT_ALREADY_RESERVED` discloses nothing about the permit in any of the six states (R6).
  - No query returns a reservation token (T170, R14).
- **Crossref replay and fence.** Finalisation fences in the same transaction. Completion requires the fence and an
  `ACCEPTED` permit at the claimed generation. A fenced attempt refuses cancellation and deletion. Fenced abandonment
  blocks the claim until reconciliation truth clears it. The profile is `ReplayBlocked`. The T205-T289 schedules
  exercise all of these.
- **Provider traffic.** None. No BE-06 source or test file carries an HTTP client or a provider endpoint (F16,
  T167). The conformance provider is an in-file function.
- **Secrets and personal data:** none handled.
- **#919 boundary.** `advanceCrossrefVersionFloor` is database-local, with no GitHub client or configuration (F16).
  G-6 and G-7 are not executed.
- **Metrics isolation:** no Metrics path in the diff.

## 9. Tests and checks

All commands below ran on the final source `c4411993` unless marked as an earlier execution. Environment:
- PostgreSQL 17 disposable local cluster, `127.0.0.1:54411`;
- `TEST_DATABASE_URL=postgres://thoth@127.0.0.1:54411/thoth_test`;
- Redis `127.0.0.1:6391`;
- `CARGO_INCREMENTAL=0`.

### 9.1 Formatting

```text
cargo fmt --all -- --check
exit 0
```

### 9.2 Lint

```text
cargo clippy --all --all-targets --all-features -- -D warnings
exit 0 (one dependency future-incompatibility notice: proc-macro-error2 v2.0.1)
```

### 9.3 The full workspace suite

```text
cargo test --workspace --no-fail-fast
exit 0
thoth (bin)                                   31 passed
thoth-api lib                               1437 passed, 0 failed, 0 ignored (139.27 s)
thoth-api tests/crossref_permit_concurrency    3 passed
thoth-api tests/crossref_serializer_dependency_guard 4 passed
thoth-api tests/graphql_permissions           13 passed
thoth-api tests/work_upsert_capture           31 passed
thoth-api tests/work_upsert_deletion           4 passed
thoth-api tests/work_upsert_lifecycle          7 passed
thoth-api-server lib                           3 passed
thoth-client lib                               4 passed
thoth-errors lib                              11 passed
thoth-export-server lib                      152 passed
doc-tests                                      thoth_client 6 passed; thoth_export_server 2 passed;
                                               thoth_api 0 passed, 8 ignored (released ignores); others 0
```

The baseline at `b23ba05c`, before any change, was: `thoth-api` lib 1242/1242; `graphql_permissions` 13/13; bin 31;
api-server 3; client 4; errors 11; export server 144; exit 0.

### 9.4 By family (all within the run in 9.3)

| Family | Tests |
|---|---|
| Budget and SDL | S1 (evaluated against the base), S2, S3, S4/B1/B5/D1/A8, S5 and S6 (`graphql::distribution_job_tests`), S7/S8/S9/B2, A1 |
| Harness and reset | H1-H7, H9 (`model::tests::be06_reset`), H8 (`crossref_permit_concurrency`), U1-U5 |
| Migration and inventory | G1-G6, the DOI canonicalisation corpus and the 13 timestamp boundary vectors, T171-T174 by G1/G5/G6 |
| Version floor | F1-F3, F5-F12, F14, F15, F18, F19, F20, F21, F22 (lib); F4, F6, F17 through GraphQL; F13 (integration, two sessions); F16/T167 (static) |
| Control | C1-C4 (C4 two sessions), C5 by X5, C6 |
| Seed and census | D1-D9, D10 = T290 (four schedules), D11, D12 by T257, D13, D14, T291 |
| Admission | A1-A8, A9 by T257, X8, T292 |
| Drain and materialization | M1-M4, M5 with M10, M6, M7, M8, M9, M11, M12 (both variants), M13, M14, M15, T40, T41-T53 |
| Reservations | R1-R6, R7, R8 by X5, R9, R10 by T257 and R1/R2, R11, R12 with B3/B6, R13, R14, R15 |
| Payload digest | P1 (with a held permit lock), P2, P3 by X5, P4 |
| Non-blank inputs | N1, N2, N3 (84 code points), N4, N5 (ICU and C databases), N6 |
| Error boundary | X0, X1-X5, X6/X7 (live), X8, X9 (lib and integration), X10, X11, X12 |
| Binding projection | B1-B6 |
| Capture and flush | T1-T37, T39, T317-T322, T324, drift guards 1 and 2 (section 10.5) |
| Deletion | T98-T102, T105, T325-T328 |
| Permit state machine | T237, T238, T239/T269 (30 ordered pairs: exactly 7 permitted), T240 |
| Timestamps and allocation | T249, T250, T252, T253, T254, T255, T259 (domain maximum), T260 (boundary vectors), T265, T248 parity |
| Extraction and serializer | T38, T242, T243, T244, T245, T246, T111, T109 |
| Races | T205-T223, T270, T272-T275, T280-T289 |
| Conformance | T258 over the four routes, with B4 |
| Inert state | T166 |
| Authorization | T257, T170 |
| Containment | T53, T203, D9, X3, X4, X11, H7 |

### 9.5 Concurrency method and transcripts

Every race test uses real sessions. A test-only pause point holds one transaction inside its open transaction: a
trigger in schema `be06_test` that blocks on an advisory lock the controller holds, armed once through
`FOR UPDATE SKIP LOCKED`, optionally conditioned on a row, and removed on drop. An `AFTER` trigger pauses while
holding the row it wrote. Each test reads the second session's wait from `pg_locks` (`locktype:mode` of the
ungranted lock), then releases the pause and asserts both results. No test observed `40P01`. Transcripts:
- the reservation, finalisation, void, report and cancellation waits in T205-T216 and T222 are
  `transactionid:ShareLock`;
- the overlapping-reservation wait in T217 is `advisory:ExclusiveLock`;
- T218 is a real `hashtext` collision (`10.12345/t218-206394` and `10.12345/t218-363313` share one key). Both
  reservations acquire the smaller key first, observed as two `advisory:ExclusiveLock` waits, and the second is
  refused `CROSSREF_PERMIT_BLOCKED`;
- the gate waits in T280-T289 are `advisory:ExclusiveLock` for the pause behind a holder and `advisory:ShareLock` for
  a consumer behind the pause, and T284 shows `pg_blocking_pids` naming both the holder and the queued pause;
- in T327 (S01-S13) both sides commit in every schedule. Waits observed while the first was held:
  `transactionid:ShareLock` in S07, S08, S09, S11, S12 and S13, and none in S01-S06 and S10.

The race suites passed twice in succession before commit (lib permit tests 74/74 twice; work_upsert and permit
172/172 twice; deletion 4/4 twice; capture 31/31 three times).

### 9.6 Flush properties with negative controls

The negative controls replace the migration's flush function with a variant missing exactly one element and restore
the original on drop.
- **T318.** With the immediacy defence present, all four forcing shapes write no generation row before completion:
  0 written and 1 arming row queued, then both edits applied at commit. With the defence removed, the forcing fires
  the queued arming row at once (1 written, 0 arming rows queued) in the between-edits, twice and named shapes.
  In the before-any-edit shape the arming row fires inside its own `INSERT` before the armed flag is set, so no early
  write is observed; the test does not assert one there.
- **T319.** With the defence present, both commit, no generation row is locked while the first waits, and the second
  never waits. With the defence removed, the early flush holds the generation rows, the second waits, and exactly one
  transaction aborts with `deadlock detected`.
- **T321.** The deleter is paused after its flush, inside its commit, by a deferred constraint trigger. The writer
  records the root as an owner directly, exactly as capture does, and so reaches its own flush for that root.
  Released editorial statements lock every capture owner before capture fires (drift guard 2), so an editorial
  writer would wait at the Work row and resolve no owner. The writer waits (`transactionid:ShareLock`), and no orphan
  remains. With the vanished-root cleanup removed, exactly one orphan remains. A direct `INSERT` of a generation row
  for a missing Work is accepted, while a supported deletion still removes its row.

### 9.7 Extraction and timestamp parity

- **T242.** Six synthetic fixtures, each carrying every excluded DOI class, return exactly the registration set; the
  naive collection over-collects at least 3 foreign DOIs in each. Three malformed artifacts are refused: a DTD,
  whitespace inside a registration `doi`, and the wrong namespace.
- **T243.** Nine real serializer documents (released and prepared) return exactly their registration sets, and the
  naive collection over-collects on at least 3. A document that registers nothing is refused. A chapter without a
  landing page cannot be serialized at all (`Missing chapter Landing Page`), so that shape has no document.
- **T244.** `crossref_deposit_membership` equals the registration set on six shapes.
- **T245.** Code-point order equals Rust `sort`: `a-b a.b a_b ab`. ICU `en-US-x-icu` orders `a_b a-b a.b ab`. The
  function pins `COLLATE "C"`.
- **T248.** The export server's test records the Rust validator's verdict over a deterministic corpus of 356,483
  strings: 39 targeted, 100,000 random 17-digit strings, random valid instants with their successors, and 60,000
  corruptions (second 60, hour 24, day 32). It records 196,763 valid, digest `7442931097035189911`. The `thoth-api`
  test regenerates the identical corpus (corpus digest checked against the recorded one) and requires PostgreSQL's
  `crossref_ts_decode` verdict to give the same valid set, string for string. It does. The naive `parse_from_str`
  round trip is shown not equivalent. The corpus size differs from R52B's 360,039 because the generator is
  deterministic and skips invalid random dates.

### 9.8 Conformance transcript (T258)

A scripted GraphQL client drives each route through every branch of R52B section 28.7 that the API can observe:
- **`WORK_UPSERT`:**
  - success, then complete;
  - `VOIDED_RETRYABLE`, then a retryable fail;
  - `VOIDED_JOB_RETIRED`, with no job call;
  - group C `CROSSREF_PERMIT_CLAIM_STALE`: void with the token, stop, no job call;
  - extraction refused: void, then `CROSSREF_ARTIFACT_REFUSED`;
  - digest-recheck failure: `NONE_ATTEMPTED`, then `CROSSREF_PROVIDER_NONE_ATTEMPTED`;
  - POST timeout: `INDETERMINATE`, then `CROSSREF_PROVIDER_INDETERMINATE` non-retryable, then superuser
    reconciliation;
  - a crash before finalisation, cleaned up by the owner void and, separately, by the superuser void found through
    `crossrefWritePermits` by job identity, after which the next attempt reserves;
  - a crash after `AUTHORIZED`: fenced abandonment, reconciliation clearance observed, and a strictly later timestamp;
  - a lost finalisation response that replays.
- **`LEGACY_SCHEDULED` and `MANUAL_RECOVERY`:** every applicable branch.
- **`PUBLISHER_BACK_CATALOGUE`:** an accepted unit, `CROSSREF_UNIT_ALREADY_DEPOSITED_IN_JOB`, a voided unit, a refused
  artifact, an `INDETERMINATE` unit blocking the outer completion with `OUTER_ATTEMPT_HAS_OPEN_PERMITS` until
  reconciled, then completion, and a stale claim afterwards.
- **R13:** all six permit states after the protocol-stop, with no job call.
- **R14:** a lost response recovered without the token. In its variant, a first reservation that failed inside its
  transaction through a one-shot test trigger surfaces as `INTERNAL_ERROR` and the repeat reserves.
- **R15:** the job-call table is disjoint from the protocol-stop set.

The client works only from the reservation, the finalisation result and its run state (B4). The prepared artifact is
a stand-in built from the reservation's timestamp, batch id and DOIs in the serializer's structure; the export
server's prepared route is exercised by that crate's own tests (T109, T111, T246).

### 9.9 Compatibility window (R52B section 23.6; T294-T302)

These ran in scratch copies only. `v1.8.0` (`9ae1e567`, the release in production) was extracted with `git archive`
and built with its own target directory. Its test reset was adapted as R52B section 25.21 describes (truncate under
a transaction-local replica mode, and re-seed the two permanent rows where they exist) and run identically over both
schemas. All databases were disposable, on the local cluster.
- **T294 and T295.**
  - The released `thoth migrate` gives 9 migrations and 60 tables. The candidate `thoth migrate` gives 11 migrations
    (`20260910`, `20260911`) and 68 tables, the control row `CROSSREF:false/false` and floor `0`.
  - The released `thoth migrate` over the migrated schema exits 0 with the ledger unchanged.
- **T296**, the released suite, `cargo test --workspace --no-fail-fast`:
  - released schema: 1,398 passed, 0 failed, 8 ignored;
  - migrated schema: 1,394 passed, 4 failed, 8 ignored.
  - Test by test, exactly four outcomes differ, all BE-04 catalogue-shape tests:
    `every_enum_label_exists_in_pg_enum_with_the_exact_spelling_and_order`,
    `every_named_constraint_of_sections_7_2_to_7_4_exists_in_the_catalog`,
    `only_distribution_job_is_diesel_managed_and_the_indexes_are_exactly_the_specified_three` and
    `schema_rs_matches_the_migration_for_all_three_relations`.
  - An instrumented rerun over the migrated schema gave identical outcomes. The instrumentation refused, and
    logged, any write to the permit, membership, admission and audit tables and to the control and floor rows, and
    any BE-06 label or column on jobs or attempts. It recorded:
    - 0 refused writes;
    - generation rows: 2,798 inserts and 833 advances, each for an existing Work, and 7 deletions after their Work
      was gone;
    - capture queue: 6,192 inserts and 6,192 matching deletes (ARM 1,279, PROBE 1,279, OWNERS 3,627, DELETED 7),
      with no queue row left;
    - the control row `(false, false)` and floor `0` unchanged.
- **T302.** The released `--revert` exits 1 with `Unable to find migration version 20260911`, leaving the ledger and
  68 tables unchanged. The candidate `--revert` is covered in section 5.4 item 3.
- **Not executed:**
  - T299, the end-to-end sequence with the released server in service;
  - T300, the released read of a `WORK_UPSERT` row;
  - T301, the candidate `init` on an unmigrated database, which needs a running server with key, Zitadel and AWS
    configuration.
  - R52B section 27.3 gate 9 requires these against the exact `v1.10.0` release artifact and the release then in
    production.

### 9.10 Rows not executed, and why

- **Out of scope by authorization:** the G-6 and G-7 ledger and authority corpus (R52B section 25.13, T154-T161;
  section 22.14's fixture corpus), owned by #919. These are explicitly prohibited.
- **Negative controls and investigation rows against withdrawn topologies or candidates:** R52B marks these as
  specification evidence. They are T218's string-sorted `40P01` control, T271, T276, T277/T278 (T257 is the
  implementation test), T279 (withdrawn), T297/T298, T303's withdrawn-schema `40P01`, and T305-T316.
- **Evidence obtained here by other tests:**
  - T329 is covered by T296's instrumented run, with the same counts.
  - T330's migration mechanics are covered by T294/T295.
  - T331 is covered by G1-G6.
  - T323 is a configuration fact: `max_prepared_transactions` is 0 on the local cluster (`SHOW max_prepared_transactions`), the PostgreSQL default.
- **Partial coverage:**
  - T106-T108, a source change reflected in the prepared artifact end to end across the export server and the
    database, is not wired, because the two crates do not share a test harness. The API side is covered by the
    witness tests (T219, the `SOURCE_CHANGED_DURING_PREPARATION` voids); the export side by T109, T111 and T246.
  - T145-T153 back-catalogue combinations are covered by the T258 back-catalogue transcript, T222, T256 and the
    outer-close guard tests, not as a separate combination matrix.

## 10. Manual verification

Scratch environment only; section 9.9. The `BE-06-snapshot-queries.sql` file was executed whole against the migrated
compatibility database with `ON_ERROR_STOP=1` and completed. It opens a `READ ONLY` transaction and ends with
`ROLLBACK`.

## 11. CI

CI status: NOT AVAILABLE for this branch. The repository's `push` workflows trigger only on `master` and `develop`,
and pull-request workflows need a PR, which is not authorized. No workflow was dispatched.

## 12. Rollout and rollback

- **Initial state after migration and deployment:** inert. The control row is `('CROSSREF', false, false)`, the
  floor is `0`, no work-level job can be claimed, and no Crossref traffic occurs.
- **Activation required:** yes. It follows R52B section 27.3 gates 9-20, each separately authorized: capture enable,
  per-publisher seed, census and admission, G-6, the dissemination release, execution enable, the G-7 floor advance,
  and so on.
- **Migration sequence:** Migration 1 then Migration 2, by the release artifact's own `thoth migrate`, before the
  backend deployment (gates 9-11).
- **Rollback:** `down M2`, per R52B section 23.5. Never `thoth migrate --revert`.
- **Monitoring:** `BE-06-snapshot-queries.sql` and the superuser reports.

## 13. Known limitations and deferred work

- The items in sections 5.4, 9.9 and 9.10.
- DIS-04 (`thoth-dissemination` #106) implements the worker. BE-06 hands over the contract and the conformance
  transcript only.

## 14. Side effects

- **Repository writes:** the 41 paths of section 4, on the task branch only.
- **Commits:** section 3.
- **Push:** `feature/publisher-services-v1-10--be-06` only, without force, after this report is committed. The remote
  task branch was verified absent before the push.
- **Automatic workflow executions caused by the push:** none expected. The push-triggered workflows at the base
  (`build_test_and_check.yml`, `run_migrations.yml`) trigger only on `master` and `develop`; the others trigger on
  pull requests, releases or dispatch. This was re-inspected immediately before the push.
- **Provider, network and production effects:** none. The only network use was `git fetch` and `git ls-remote`
  against `origin`, then the push. Local servers were bound to `127.0.0.1` for the OpenAPI capture and pointed at
  unreachable local ports.

## 15. Remaining gates

- The implementation agent does not approve its own work.
- PR creation is still unauthorized.
- The next gate is a fresh independent exact-head CRITICAL source, migration, authorization and concurrency review
  of the pushed head.
- After source approval, the PR, merge, release, deployment, production migration, G-6, G-7 and activation each
  remain separate, separately authorized gates.

## 16. Suggested review focus

1. The two migrations against R52B section 18 and Amendment 3 section 3: the capture function and flush, the permit
   state machine, the guards and the inventory.
2. The cancellation-guard fix of section 5.2, and any other `FOR UPDATE` statement whose predicate reads another
   table in the same statement.
3. The observations of section 5.4, especially T166 and admission while inert.
4. The EB1/EB2 boundary in the five model files and the three deletion units.
5. The race harness's pause-point technique, and whether its wait transcripts are sufficient evidence.

## 17. T166 correction

### 17.1 Finding and authority

- Head `bfef2daa0369a6cb8b45d8f610949bda865cef72` (tree `3146c97dc2d78504130c278e14ec66fd365f33af`) failed
  independent review because T166 had been weakened: it asserted that explicit admission succeeds while the control
  row is `(false, false)`, as section 5.4 item 1 describes.
- The defect was a conflict between specification authorities, retained R52B T166 against Amendment 3 section 9.3's
  frozen admission precedence, not merely an editorial test problem. Section 5.4 item 1 chose section 9.3; the
  implementation agent had no authority to resolve that conflict either way.
- CTO specification correction #848 comment `5701572954` resolved it: `admitCrossrefWorkUpsert` requires
  `capture_enabled = true`, checked after the publisher `P FOR SHARE` lock and existence proof and before activation
  resolution, the existing-admission return and the census, refusing with the existing `WORK_UPSERT_CAPTURE_NOT_ENABLED`.
  It placed `5687443066` on HOLD for further writes.
- The correction authority is #848 comment `5701857879`, bound to base `bfef2daa…` and four paths.
- No approval of `bfef2daa` or of any later head exists.

### 17.2 Preflight

`git fetch origin --prune`; local `HEAD` and `origin/feature/publisher-services-v1-10--be-06` both `bfef2daa…`, tree
`3146c97d…`; clean worktree; no PR for the branch; the #848 comment list ended with `5701572954` and `5701857879`.

### 17.3 RED

T166 was rewritten before any production change. While `(false, false)` it requires admission to be refused with
`WORK_UPSERT_CAPTURE_NOT_ENABLED`, and zero admission, permit, audit, `WORK_UPSERT` job, target and attempt rows, with
generation rows the only BE-06 state. It then enables capture through `enable_work_upsert_capture` and requires one
admission (`EV`, `admin`) and no job.

```text
cargo test -p thoth-api --features backend --lib work_upsert::tests::t166
panicked at thoth-api/src/model/work_upsert/tests.rs:7234:5:
assertion `left == right` failed
  left: Ok("admin")
 right: Err(WorkUpsertCaptureNotEnabled)
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1436 filtered out
```

The failure is the approved defect: admission succeeded while inert.

### 17.4 GREEN and the exact diff

`admit_crossref_work_upsert` gained one check, after the publisher block and before `crossref_activation`:

```rust
if !capture_enabled(connection, DistributionPlatform::Crossref)? {
    return Err(ThothError::WorkUpsertCaptureNotEnabled.into());
}
```

It reuses the helper the seed already uses. The rerun of the same command gave `1 passed; 0 failed`.

The first full workspace run then failed exactly one test, A6/A7 (`a6_a7_activation_drift_and_missing_bindings`):
`left: Err(WorkUpsertCaptureNotEnabled)`, `right: Err(CrossrefPublisherNotCovered)`. Its uncovered-publisher
refusal ran with capture disabled, so it met the corrected precedence. A6/A7 now asserts
`WORK_UPSERT_CAPTURE_NOT_ENABLED` for that publisher, enables capture, and keeps its original
`CROSSREF_PUBLISHER_NOT_COVERED` assertion and the rest of the test unchanged.

Correction commit `23c7be6180aa13cde7a036ac1d1abb0181ab9399` (tree `68f1cc7231349b9508d7f40340f66f925ee10d58`):

```text
git show --numstat 23c7be61
5   0  thoth-api/src/model/work_upsert/crud.rs   (the check and a two-line comment)
30  7  thoth-api/src/model/work_upsert/tests.rs  (T166 and A6/A7)
```

The documentation commit that adds this section changes only this file and `tasks/BE-06.md`; its SHA is the pushed
head.

### 17.5 Validation on the correction commit

```text
cargo fmt --all -- --check                                           exit 0
cargo clippy --all --all-targets --all-features -- -D warnings       exit 0 (proc-macro-error2 notice only)
cargo test --workspace --no-fail-fast                                exit 0
thoth (bin)                                   31 passed
thoth-api lib                               1437 passed, 0 failed, 0 ignored
thoth-api tests/crossref_permit_concurrency    3 passed
thoth-api tests/crossref_serializer_dependency_guard 4 passed
thoth-api tests/graphql_permissions           13 passed
thoth-api tests/work_upsert_capture           31 passed
thoth-api tests/work_upsert_deletion           4 passed
thoth-api tests/work_upsert_lifecycle          7 passed
thoth-api-server lib 3; thoth-client lib 4; thoth-errors lib 11; thoth-export-server lib 152
doc-tests: thoth_client 6; thoth_export_server 2; thoth_api 0 passed, 8 ignored
```

Within that run, each passed:

- T166; admission A2, A3, A4, A6/A7, X8/A5 (concurrent exact admissions), S4/A8 (frozen types), T257 (A9 roles);
  D10/T290, D11, T291, T292 (admission against a concurrent activation change and capture), T273;
- control and precedence: C1, C2/C3, C4, D8, the GraphQL refusal order `f4_f6_f17_c6_m7_x9_refusals_through_graphql`;
- error mapping: X2, X6, `the_boundary_commits_rolls_back_and_converts_exactly`, `thoth-errors` 11/11;
- static containment: S1 (43-path allowlist, evaluated), X1 (both), X3, A1, D9, T203;
- the integration lifecycle, capture and permit-concurrency suites.

### 17.6 Semantics

- Capture disabled: admission returns `WORK_UPSERT_CAPTURE_NOT_ENABLED` inside the transaction before any activation
  read, admission lookup, census or insert; the transaction writes nothing.
- Capture enabled: the rest of section 9.3 is unchanged: exact-binding activation, idempotent return of the existing
  row, the census refusal, `ON CONFLICT DO NOTHING` with the later read, and no job creation.
- The evidence-reference refusal and `SUPERUSER` authorization still precede the transaction. Unknown publishers
  still give `ENTITY_NOT_FOUND` before the capture read.
- No lock class, GraphQL shape, result field, error code or mapping, schema, migration or other path changed.

### 17.7 Side effects and remaining gate

- **Repository writes:** the four authorized paths only; `git diff --name-status bfef2daa..HEAD` lists
  `crud.rs`, `tests.rs`, `tasks/BE-06.md` and this report.
- **Commits:** the correction commit and this documentation commit, as Javier Arias, without AI attribution. No
  earlier commit was amended, rebased or rewritten.
- **Push:** the task branch only, without force, fast-forward from `bfef2daa`. Push-triggered workflows still trigger
  only on `master` and `develop`, re-inspected immediately before the push.
- **No** PR, issue or comment mutation, CI dispatch, provider access, Crossref traffic, production access, G-6, G-7,
  #919 operation or BE-07 work. Tests ran only against the disposable local PostgreSQL 17 and Redis.
- **Gate:** the implementation agent does not approve its own work. `bfef2daa` remains unapproved. The pushed head
  requires a fresh independent exact-head CRITICAL source, migration, authorization and concurrency review. PR
  creation remains unauthorized.

## 18. Post-review correction

### 18.1 Why `aabb5e0d` was not approved

Independent exact-head CRITICAL review of `aabb5e0dace9e3d613ee1a2b347496326cec5d36` (tree
`de2dfb0dead84066914e68f12fd5a62fb955e743`), the head that carried section 17, found two blockers:

1. **Finalisation replay identity.** Amendment 3 section 4.6 described replay as an identical presentation of six
   elements: permit, reservation token, observed DOI set, observed batch id, observed timestamp and payload digest.
   R52B section 16.6 C2 and T229, which the source follows, replay an `AUTHORIZED` permit on the persisted digest and
   a finalisation-voided permit on its recorded outcome. A finalisation void persists no digest (the void CHECK forces
   `payload_digest` to NULL) and no observed value, so the six-element identity cannot be evaluated for it without new
   persisted evidence.
2. **Failure and completion against a concurrent reservation.** `attempt_permit_guard` read the attempt's permits by
   MVCC before its transaction held the job row.

A read-only investigation on a disposable copy of `aabb5e0d`, outside the repository, reproduced blocker 2 on real
sessions and found a third race (section 18.5). CTO specification correction #848 comment `5703204194` resolved all
three inside the approved architecture, and #848 comment `5703254463` authorized this correction from `aabb5e0d`
with a seven-path budget. `aabb5e0d` is not approved.

### 18.2 Preflight

`git fetch origin --prune`; local `HEAD`, `origin/feature/publisher-services-v1-10--be-06` and `git ls-remote` all
`aabb5e0d…`, tree `de2dfb0d…`; zero `git status` entries; `gh pr list --head feature/publisher-services-v1-10--be-06
--state all` empty; #848 had 32 comments, ending with `5703204194` and `5703254463`; no workflow run existed for the
branch, and the push-triggered workflows at the base trigger only on `master` and `develop`.

### 18.3 Replay: the clarification against the source

Comment `5703204194` section 1 withdraws Amendment 3 section 4.6's six-element wording where it conflicts with R52B
C2. The production finalisation path already implemented the clarified contract and is unchanged.
`t229_replay_is_decided_by_the_reservation_token_and_the_persisted_digest` is therefore a characterization test: it
passed against the `aabb5e0d` production files and after the correction. Every presentation in the table was also
checked to write nothing, by a fingerprint of the permit, job and attempt rows including `xmin`.

| Permit | Presentation | Result |
|---|---|---|
| `AUTHORIZED` (`WORK_UPSERT`) | the reservation token and the persisted digest, with the first presentation's fields, another observed DOI set, no observed DOI, another batch id, another timestamp, no claim token, another claim token, or all of these at once | `AUTHORIZED`, the persisted digest unchanged |
| `AUTHORIZED` | each of those with another valid digest | `CROSSREF_PERMIT_ILLEGAL_TRANSITION` |
| `VOIDED` by finalisation, group A `SOURCE_CHANGED_DURING_PREPARATION` (job still `RUNNING`) | each variation, with the first digest or another valid digest | `VOIDED_RETRYABLE`, the same reason |
| `VOIDED` by finalisation, group A `ARTIFACT_BATCH_ID_MISMATCH` (legacy route) | the presentation that matches the reservation, which a first call would have authorised, and each variation, with either digest | `VOIDED_RETRYABLE`, the same reason |
| `VOIDED` by finalisation, group B `BINDING_SUPERSEDED` (job `CANCELLED`) | each variation, with either digest | `VOIDED_JOB_RETIRED`, the same reason |
| `VOIDED` by the route owner, and by the superuser | each variation, with either digest | `CROSSREF_PERMIT_ILLEGAL_TRANSITION` |
| each permit above | a valid digest (either) and a wrong reservation token | `CROSSREF_PERMIT_REQUIRES_RESERVATION_TOKEN` |
| each permit above | a malformed digest (upper case, or 63 characters), with the right or a wrong token | `CROSSREF_PAYLOAD_DIGEST_INVALID` |
| each permit above | a route authorization that refuses, with a valid or a malformed digest | `Unauthorised` |

No replay fingerprint, column, table, enum, trigger or migration was added.

### 18.4 Terminal transitions: root cause

- `fail_distribution_job` ran `attempt_permit_guard` for every job, and `complete_distribution_job` ran it for every
  job other than `WORK_UPSERT`, before any statement of theirs locked the job row. The guard found the open attempt
  and read its permits by MVCC.
- A `WORK_UPSERT` or back-catalogue reservation takes `J FOR UPDATE`, then `A FOR UPDATE`, and inserts its permit
  later in the same transaction. A guard running in that window saw no permit. The released job `UPDATE` that
  followed waited on the reservation's row lock. When the reservation committed, READ COMMITTED re-evaluated only
  that `UPDATE`'s own predicate against the job row, which the reservation had locked but not changed. The `UPDATE`
  applied, the attempt closed, and the guard never ran again.
- At `aabb5e0d` (section 18.9): a retryable `WORK_UPSERT` failure returned `PENDING` and closed the attempt `FAILED`
  under a `RESERVED` permit; a terminal back-catalogue failure returned `FAILED`, and a back-catalogue completion
  `SUCCEEDED`, each over a new `RESERVED` unit permit.
- The opposite order was already safe: the reservation's claim re-check under `J` refused `CROSSREF_PERMIT_CLAIM_STALE`.

### 18.5 Accepted-unit race: root cause

- Amendment 3 section 9.6 orders the back-catalogue reservation as step 9, an `ACCEPTED` permit for this root in this
  outer job; step 10, membership; step 11, `K`, `F`, the blocking check, history, allocation and insertion.
  `reportCrossrefWrite` takes only the permit row `X`; reconciliation takes `A` of the permit's own attempt, then `X`.
- A unit permit that was `AUTHORIZED` or `INDETERMINATE` at step 9 passed that check. If a report, or a
  reconciliation of a permit from an earlier attempt of the same outer job, committed `ACCEPTED` before the blocking
  check, the blocking check found nothing, because `ACCEPTED` does not block. The reservation then issued a second
  permit for a unit already deposited in the outer job.
- By derivation, a reconciliation of a permit on the reservation's own attempt serialises with it at `A`, so the
  reachable reconciliation window is the earlier-attempt case.

### 18.6 The source correction

```text
git diff --numstat aabb5e0d 073447cc
34    16  thoth-api/src/model/crossref_write_permit/crud.rs
1307   0  thoth-api/src/model/crossref_write_permit/tests.rs
23     2  thoth-api/src/model/distribution_job/crud.rs
```

1. `distribution_job/crud.rs`, `attempt_permit_guard`: a new first statement locks the job row by id,
   `SELECT status, claim_token … FOR UPDATE`, and returns `Ok(())` unless the row is `RUNNING` with the presented
   token, so the released statement still classifies a stale, missing or terminal claim as before. The existing join
   of the job to its open attempt and the permit read now follow in later statements. Refusal order and codes are
   unchanged. The comments at both call sites say the row stays held.
2. `crossref_write_permit/crud.rs`: the step 9 query moved unchanged into `unit_deposited_in_job`, called at the same
   place. `issue` calls it again on the back-catalogue route, after the blocking check found no blocker and before
   history, allocation and insertion, refusing `CROSSREF_UNIT_ALREADY_DEPOSITED_IN_JOB`. The other routes are
   unchanged.
3. Unchanged: `finalise_crossref_write`, `report_crossref_write`, reconciliation, both voids, the `WORK_UPSERT`
   completion guard and T2, cancellation, lease recovery, both claims and the deletion units. No migration,
   `schema.rs`, GraphQL, error definition or mapping, public API, enum, trigger, dependency or workflow changed.

### 18.7 Semantics

- **Where `J` is taken.** In `attempt_permit_guard`'s first statement: for failure of every job, and for completion of
  every job that does not take the `WORK_UPSERT` prefix (every kind other than `WORK_UPSERT`, and a `WORK_UPSERT` row
  without its Work or profile). None of those paths takes `P`, `W` or `G`, so `J` is their first row lock and their
  order is `J → A`, as before.
- **Claim re-check.** The lock statement returns the job row's latest committed version after any wait, and the guard
  compares status and token in Rust. The attempt join and the permit read are later statements, so their snapshots
  include every transaction that held `J` before this one. The permit read takes no lock.
- **Retention.** Row locks end only with the transaction: through the released job `UPDATE` (on the row already held),
  `close_open_attempt` (`A`) and the commit, or through the rollback of a refusal.
- **Both orders.** A reservation or finalisation holding `J` makes the failure or completion wait; once it commits,
  the guard sees `RESERVED` or `AUTHORIZED` and refuses, leaving the job `RUNNING` and the attempt open. A failure or
  completion holding `J` makes the reservation or finalisation wait at its own `J`; once that commits, the claim
  re-check refuses `CROSSREF_PERMIT_CLAIM_STALE`. No newly committed `RESERVED` or `AUTHORIZED` permit is left on an
  attempt that the generic transition closes.
- **`WORK_UPSERT` completion.** Unchanged, and still `P → W → G → J → A`. It holds `G` before it reads the fence and
  the accepted permit, and a reservation or finalisation of the same attempt holds `G` from before its own `J` until
  its commit, so the two serialise at `G` (section 18.8). R52B section 10.4's stability argument still holds: one
  permit per attempt, a write-once fence, and `ACCEPTED` terminal. No `J → P`, `J → W` or `J → G` edge was added.
- **Deadlock.** The lock sequence of a failure or outer completion is unchanged, `J` then `A`; only the moment `J` is
  taken moved, to before the MVCC reads. When it requests `J` it holds no row lock (the completion's job-kind read
  before it is MVCC), so that wait cannot close a cycle, and its one hold-and-wait, `J` held while it waits for `A`,
  existed before. A stale or terminal claim now also takes `J` briefly before the released statement classifies it.
  No wait-for cycle is added.
- **Late deposited check.** An MVCC read in a statement after `K`, `F` and the blocking check. A report or
  reconciliation that commits before the blocking check's snapshot lets that check pass, and the later statement sees
  `ACCEPTED`. One that commits later leaves the permit blocking in that snapshot, and the reservation is refused
  `CROSSREF_PERMIT_BLOCKED`. Either way no second permit is issued for a unit whose overlapping permit is accepted in
  the outer job. A permit whose DOIs no longer overlap the root's current membership never blocks, so a report that
  commits after the late check is not observed; that is also the sequential behaviour of Amendment 3 section 9.6,
  whose deposited check reads the state at reservation time.
- **Report and reconciliation locks.** Not widened. Comment `5703204194` section 3 keeps their approved leaf-lock
  behaviour; the reservation observes their commits instead.

### 18.8 Regression tests

All in `thoth-api/src/model/crossref_write_permit/tests.rs`, module `post_review_correction`. The race tests use the
section 9.5 pause point. While the second session waits they read its lock from `pg_locks`, the statement it is
executing from `pg_stat_activity`, and the job's committed permit count. They assert both results and the final
rows, formatted `status|claimed|attempt_count|last_error_code attempts=… permits=state@attempt`.

| Test | Schedule | Final state after the correction |
|---|---|---|
| `a_failure_behind_a_work_upsert_reservation_waits_on_the_job_row_and_refuses` | reservation paused before its permit insert; retryable failure | failure waits in `distribution_job FOR UPDATE` with 0 committed permits, then `ATTEMPT_HAS_OPEN_RESERVATION`; `RUNNING`, attempt open, permit `RESERVED` |
| `a_reservation_behind_a_work_upsert_failure_waits_on_the_job_row_and_finds_the_claim_stale` | failure paused after its job `UPDATE`; reservation | failure `PENDING`; reservation waits in `distribution_job FOR UPDATE`, then `CROSSREF_PERMIT_CLAIM_STALE`; attempt `FAILED`; no permit |
| `a_failure_behind_a_finalisation_waits_on_the_job_row_and_refuses_the_authorization` | finalisation paused before its permit `UPDATE`; terminal failure | finalisation `AUTHORIZED`; failure waits in `distribution_job FOR UPDATE`, then `ATTEMPT_HAS_AUTHORIZED_PERMIT`; `RUNNING`, attempt open, permit `AUTHORIZED` |
| `a_work_upsert_completion_behind_a_reservation_waits_on_the_generation_row_and_refuses` | reservation paused before its permit insert; completion | completion waits in `work_upsert_generation FOR UPDATE`, then `WORK_UPSERT_COMPLETION_REQUIRES_FENCE`; `RUNNING`, attempt open, permit `RESERVED` |
| `a_terminal_outer_failure_behind_a_unit_reservation_waits_on_the_job_row_and_refuses` | unit reservation paused before its permit insert; outer failure, not retryable | failure waits in `distribution_job FOR UPDATE`, then `ATTEMPT_HAS_OPEN_RESERVATION`; `RUNNING`, attempt open, unit permit `RESERVED` |
| `a_unit_reservation_behind_a_terminal_outer_failure_waits_on_the_job_row_and_finds_the_claim_stale` | outer failure paused after its job `UPDATE`; unit reservation | failure `FAILED`; reservation waits in `distribution_job FOR UPDATE`, then `CROSSREF_PERMIT_CLAIM_STALE`; attempt `FAILED`; no permit |
| `an_outer_completion_behind_a_unit_reservation_waits_on_the_job_row_and_refuses` | unit reservation paused before its permit insert; outer completion | completion waits in `distribution_job FOR UPDATE`, then `ATTEMPT_HAS_OPEN_RESERVATION`; `RUNNING`, attempt open, unit permit `RESERVED` |
| `a_unit_reservation_behind_an_outer_completion_waits_on_the_job_row_and_finds_the_claim_stale` | outer completion paused after its job `UPDATE`; unit reservation | completion `SUCCEEDED`; reservation waits in `distribution_job FOR UPDATE`, then `CROSSREF_PERMIT_CLAIM_STALE`; attempt `SUCCEEDED`; no permit |
| `a_report_accepting_the_unit_after_the_early_check_is_seen_by_the_late_deposited_check` | unit permit `AUTHORIZED`; another session holds the unit's DOI key; the repeat reservation waits on it (`advisory:ExclusiveLock`), past step 9; `reportCrossrefWrite` `ACCEPTED` commits; the key is released | report `ACCEPTED`; reservation `CROSSREF_UNIT_ALREADY_DEPOSITED_IN_JOB`; `RUNNING`, attempt open, one permit, `ACCEPTED` |
| `a_unit_reservation_reading_the_permit_before_the_report_commits_is_blocked` | report paused after its permit `UPDATE`; the repeat reservation runs to its end without waiting | reservation `CROSSREF_PERMIT_BLOCKED`; report `ACCEPTED`; one permit |
| `a_reconciliation_accepting_the_unit_after_the_early_check_is_seen_by_the_late_deposited_check` | unit permit `INDETERMINATE` from attempt 1; attempt 1 failed retryably and the job was claimed again; the key held; the attempt 2 reservation waits on it; reconciliation to `ACCEPTED` commits; the key is released | reconciliation `ACCEPTED`; reservation `CROSSREF_UNIT_ALREADY_DEPOSITED_IN_JOB`; attempts `FAILED,OPEN`; one permit, `ACCEPTED` on attempt 1 |
| `a_unit_reservation_reading_the_permit_before_the_reconciliation_commits_is_blocked` | the same, with reconciliation paused after its permit `UPDATE` | reservation `CROSSREF_PERMIT_BLOCKED`; reconciliation `ACCEPTED`; one permit |
| `a_work_upsert_failure_over_each_committed_permit_state` | sequential: retryable and terminal failure over each of the six permit states, and a retryable failure after a `VOIDED_RETRYABLE` finalisation | `RESERVED` → `ATTEMPT_HAS_OPEN_RESERVATION` and `AUTHORIZED` → `ATTEMPT_HAS_AUTHORIZED_PERMIT`, each with the job `RUNNING` and the attempt open; `VOIDED`, `NONE_ATTEMPTED`, `INDETERMINATE` and `ACCEPTED` → `PENDING` or `FAILED`, attempt `FAILED` |
| `an_outer_back_catalogue_completion_or_failure_over_each_committed_unit_permit_state` | sequential: completion, retryable failure and terminal failure over each of the six unit-permit states, and a retryable failure at `attempt_count = 5` | `RESERVED` and `AUTHORIZED` refuse all three; `INDETERMINATE` refuses completion and terminal failure with `OUTER_ATTEMPT_HAS_OPEN_PERMITS`, allows the retryable failure (`PENDING`), and refuses it at the budget; `VOIDED`, `NONE_ATTEMPTED` and `ACCEPTED` allow all three |
| `t229_replay_is_decided_by_the_reservation_token_and_the_persisted_digest` | section 18.3 | section 18.3 |

### 18.9 RED

The tests were written before any production change. Both RED runs used this command:

```text
cargo test -p thoth-api --features backend --lib -- --test-threads=1 \
  model::crossref_write_permit::tests::post_review_correction
```

1. In the task worktree, with only the test file changed (the production files byte-identical to `aabb5e0d`) and
   before the finalisation-first test was added: `test result: FAILED. 9 passed; 5 failed`.
2. In a disposable `git archive aabb5e0d` copy, with only `crossref_write_permit/tests.rs` replaced by the committed
   file from `073447cc` (SHA-256 `755d30a6…`) and its own target directory: `test result: FAILED. 9 passed;
   6 failed`. `left` is what `aabb5e0d` did; `right` is the corrected contract:

```text
a_failure_behind_a_work_upsert_reservation_waits_on_the_job_row_and_refuses
  left: (Ok(()), Ok(Pending), Waited { locks: ["transactionid:ShareLock"], statements: ["released distribution_job UPDATE"], committed_permits: 0 }, "PENDING|false|1|CROSSREF_PREPARED_FETCH_FAILED attempts=FAILED permits=RESERVED@FAILED")
 right: (Ok(()), Err(AttemptHasOpenReservation), Waited { locks: ["transactionid:ShareLock"], statements: ["distribution_job FOR UPDATE"], committed_permits: 0 }, "RUNNING|true|1|- attempts=OPEN permits=RESERVED@OPEN")
a_terminal_outer_failure_behind_a_unit_reservation_waits_on_the_job_row_and_refuses
  left: (Ok(()), Ok(Failed), Waited { locks: ["transactionid:ShareLock"], statements: ["released distribution_job UPDATE"], committed_permits: 0 }, "FAILED|false|1|CROSSREF_ARTIFACT_REFUSED attempts=FAILED permits=RESERVED@FAILED")
 right: (Ok(()), Err(AttemptHasOpenReservation), Waited { locks: ["transactionid:ShareLock"], statements: ["distribution_job FOR UPDATE"], committed_permits: 0 }, "RUNNING|true|1|- attempts=OPEN permits=RESERVED@OPEN")
an_outer_completion_behind_a_unit_reservation_waits_on_the_job_row_and_refuses
  left: (Ok(()), Ok(Succeeded), Waited { locks: ["transactionid:ShareLock"], statements: ["released distribution_job UPDATE"], committed_permits: 0 }, "SUCCEEDED|false|1|- attempts=SUCCEEDED permits=RESERVED@SUCCEEDED")
 right: (Ok(()), Err(AttemptHasOpenReservation), Waited { locks: ["transactionid:ShareLock"], statements: ["distribution_job FOR UPDATE"], committed_permits: 0 }, "RUNNING|true|1|- attempts=OPEN permits=RESERVED@OPEN")
a_report_accepting_the_unit_after_the_early_check_is_seen_by_the_late_deposited_check
  left: (Ok(0659b22a-d3f2-425e-8744-8130b570d7b2), Ok(Accepted), ["advisory:ExclusiveLock"], "RUNNING|true|1|- attempts=OPEN permits=ACCEPTED@OPEN,RESERVED@OPEN")
 right: (Err(CrossrefUnitAlreadyDepositedInJob), Ok(Accepted), ["advisory:ExclusiveLock"], "RUNNING|true|1|- attempts=OPEN permits=ACCEPTED@OPEN")
a_reconciliation_accepting_the_unit_after_the_early_check_is_seen_by_the_late_deposited_check
  left: (Ok(ef3231e7-fd2e-436f-897c-405adb2a6269), Ok(Accepted), ["advisory:ExclusiveLock"], "RUNNING|true|2|CROSSREF_PROVIDER_INDETERMINATE attempts=FAILED,OPEN permits=ACCEPTED@FAILED,RESERVED@OPEN")
 right: (Err(CrossrefUnitAlreadyDepositedInJob), Ok(Accepted), ["advisory:ExclusiveLock"], "RUNNING|true|2|CROSSREF_PROVIDER_INDETERMINATE attempts=FAILED,OPEN permits=ACCEPTED@FAILED")
a_failure_behind_a_finalisation_waits_on_the_job_row_and_refuses_the_authorization
  the second session did not wait: Err(AttemptHasOpenReservation); the first: Ok(Authorized)
```

The first five are the reproduced defects. In the sixth, `aabb5e0d`'s failure read the still-`RESERVED` permit and
refused at once, without serialising with the finalisation. That result was safe but not the specified order. The
nine tests that passed at `aabb5e0d` are the three opposite orders on the job row, the two orders in which a report
or reconciliation commits after the reservation's read, the `WORK_UPSERT` completion at `G`, both sequential controls
and T229.

### 18.10 GREEN and validation

- Before the commit: module `post_review_correction` 15 passed in three consecutive runs;
  `cargo test -p thoth-api --features backend --no-fail-fast`: lib 1452 passed, `crossref_permit_concurrency` 3,
  `crossref_serializer_dependency_guard` 4, `graphql_permissions` 13, `work_upsert_capture` 31,
  `work_upsert_deletion` 4, `work_upsert_lifecycle` 7.
- On source commit `073447cca036ff4d91c540003cd4d49f9c9adaac` (tree `81f452076958741e1564cface4b4f0bb952b88ef`), with
  a clean worktree:

```text
cargo fmt --all -- --check                                      exit 0
cargo clippy --all --all-targets --all-features -- -D warnings  exit 0 (every workspace crate re-checked; proc-macro-error2 notice only)
cargo test --workspace --no-fail-fast                           exit 0; 1723 passed, 0 failed, 8 ignored
thoth (bin)                                          31 passed
thoth-api lib                                      1452 passed (1437 + 15)
thoth-api tests/crossref_permit_concurrency           3 passed
thoth-api tests/crossref_serializer_dependency_guard  4 passed
thoth-api tests/graphql_permissions                  13 passed
thoth-api tests/work_upsert_capture                  31 passed
thoth-api tests/work_upsert_deletion                  4 passed
thoth-api tests/work_upsert_lifecycle                 7 passed
thoth-api-server lib 3; thoth-client lib 4; thoth-errors lib 11; thoth-export-server lib 152
doc-tests: thoth_client 6; thoth_export_server 2; thoth_api 0 passed, 8 ignored
```

- Within that run: `model::crossref_write_permit::tests` 99 (84 existing and 15 new), `model::distribution_job::tests`
  74, `model::work_upsert::tests` 99 and `graphql::` 163 passed. Among them: T166; the existing replay and
  finalisation tests (`a_work_upsert_finalisation_authorises_and_fences_in_one_transaction`, groups A0, A, B and C,
  P1, P2, T216); the completion and failure guards
  (`completion_requires_the_fence_and_an_accepted_permit_at_the_claimed_generation`,
  `an_outer_back_catalogue_attempt_cannot_close_over_an_open_unit_permit`, T2, R6, R7); T256
  (`a_back_catalogue_unit_is_deposited_at_most_once_per_outer_job`); the races T205-T223, T270, T272-T275 and
  T280-T289; admission A2, A3, A4, X8/A5, A6/A7, S4/A8 and T257/A9, with D10/T290, D11, T291 and T292; deletion, fence
  and recovery T325-T328; the GraphQL refusal order (`f4_f6_f17_c6_m7_x9_refusals_through_graphql`) and X9; F13 and
  H8; X4 for the shared job operations and for the deletion units; T203; the serializer dependency guard; and S1,
  evaluated.
- **Deadlocks.** `pg_stat_database.deadlocks` rose by exactly one in each run that included
  `tests/work_upsert_capture.rs`. Every `deadlock detected` line in the disposable server's log is
  `t319_negative_control_without_the_defence_deadlocks`, which requires that deadlock. The correction's module runs
  left the counter unchanged, and every race result is asserted exactly, so no race order met `40P01`.

### 18.11 Diff

`git diff --name-status aabb5e0dace9e3d613ee1a2b347496326cec5d36..HEAD`, with the documentation commit:

```text
M	docs/engineering/ai-delivery/implementation-reports/BE-06-implementation-report.md
M	docs/engineering/ai-delivery/tasks/BE-06.md
M	thoth-api/src/model/crossref_write_permit/crud.rs
M	thoth-api/src/model/crossref_write_permit/tests.rs
M	thoth-api/src/model/distribution_job/crud.rs
```

Five of the seven authorized paths, all modifications; no addition, deletion, rename or copy.
`thoth-api/src/model/distribution_job/tests.rs` and `thoth-api/tests/crossref_permit_concurrency.rs` were not needed
and are untouched. The documentation commit changes only this file and `tasks/BE-06.md`; its SHA is the pushed head.

### 18.12 Side effects and remaining gate

- **Repository writes:** the five paths above, on the task branch only.
- **Commits:** `073447cc` and the documentation commit, as Javier Arias, without AI attribution. No earlier commit was
  amended, rebased or rewritten.
- **Push:** `feature/publisher-services-v1-10--be-06` only, a fast-forward from `aabb5e0d` without force. The remote
  branch, the PR list, #848 and the push triggers are re-checked immediately before it.
- **No** PR, issue or comment mutation, CI dispatch, provider access, Crossref traffic, production access, migration
  outside the disposable database, G-6, G-7, #919 operation or BE-07 work. Tests ran only against the disposable local
  PostgreSQL 17 and Redis. The disposable `aabb5e0d` copy and its build directory are outside the repository.
- **Gate:** the implementation agent does not approve its own work. `aabb5e0d` remains an unapproved historical head.
  The pushed head requires a fresh independent exact-head CRITICAL source, migration, authorization and concurrency
  review of the entire BE-06 implementation, not only this correction. PR creation remains unauthorized; merge,
  release, deployment, production migration, G-6, G-7 and activation remain separate gates.

## 19. Merge-readiness correction

### 19.1 Finding and authority

- Independent exact-head CRITICAL review approved `86917f9f909a8e84f0682af72b48d43245d9858e` (tree
  `2190191b4827a1624a48f3cb7c4c64c975c2f766`), the head that carried section 18, in #848 comment `5715923725`. PR #925
  was then opened against `feature/publisher-services-v1-10 @ b23ba05c…` and marked ready for review.
- The pre-merge sweep of PR #925's review threads found two findings from an automated review of `86917f9f`
  (`chatgpt-codex-connector`, review `5238132704`; thread comments `4038781259` and `4038781276`), both citing
  `thoth-api/AGENTS.md` section 6:
  1. `thoth-api/src/model/work_upsert/crud.rs`: the materializer loads the complete candidate set before it applies
     `limit`.
  2. `thoth-api/src/model/crossref_write_permit/crud.rs`: a permit report that returns N permits runs N further
     `membership_of` queries.
- #848 comment `5718898530` is the CTO's merge-readiness decision, CHANGES REQUIRED, and the sole authority for this
  correction. It rejects finding 1, confirms finding 2 as blocking, and authorizes a test-first correction of finding
  2 from `86917f9f` within three paths: `crossref_write_permit/crud.rs`, `crossref_write_permit/tests.rs` and this
  report. It authorizes one correction commit, a fast-forward push, and the natural PR #925 workflows that push
  triggers. The push makes approval `5715923725` historical.

### 19.2 Preflight

Read-only, before any write: #848 comment `5718898530` read in full (author `ja573`, created
`2026-09-17T17:54:51Z`, never edited); it was the last of 38 comments and no timeline event followed it.
`git ls-remote` and the GitHub API both gave `feature/publisher-services-v1-10--be-06` = `86917f9f…` (tree
`2190191b…`) and `feature/publisher-services-v1-10` = `b23ba05c…`. PR #925 was open, not a draft, unmerged and
mergeable, with head `86917f9f…`, base `b23ba05c…` and 46 commits. Its two review threads were unresolved. PR #927
was a draft at `4372841c…`. The work was done in a fresh clone of the repository on the existing task branch, with a
clean worktree and `HEAD` = `86917f9f…`; no branch was created.

### 19.3 The rejected materializer finding

No materializer source changed; `thoth-api/src/model/work_upsert/crud.rs` is byte-identical to `86917f9f`. Finding 1
asks for the batch `limit` to be applied before candidates are loaded and for `remainingCandidates` to become a
count. Approved Amendment 3 (SHA-256 `d9c04dec84945f3e1ff13a9101372cd65b964b466e5c02ca34da25982e4b0831`) section 9.4
specifies the opposite, deliberately:

- the selector is one `READ COMMITTED` statement "returning every member of `C` over the requested profiles — with no
  `LIMIT`", together with the abstract content the pure Rust drainability filter needs;
- "`remainingCandidates` repeats that read after the call's last unit commits";
- "`limit` bounds the number of unit transactions, and so the rows a call locks and writes; it does not bound the
  rows a call reads."

Section 9.4 states that cost and rests its convergence argument on it: no unit is ever spent on an undrainable
candidate. Bounding the read would reintroduce the starvation that section removed. The finding conflicts with the
approved specification, and comment `5718898530` authorizes no materializer change.

### 19.4 Permit-report N+1: root cause

Reports 11 and 12, `crossref_write_permits` and `crossref_unresolved_permits`, each load their permit rows in one
statement and pass them to the private helper `with_dois`. At `86917f9f` that helper mapped every permit to
`membership_of(connection, permit.permit_id)`, the single-permit read
`SELECT doi FROM public.crossref_write_permit_doi WHERE permit_id = $1 ORDER BY doi COLLATE "C"`. A report returning N
permits therefore ran N membership statements after its permit statement. Report 11 accepts any `limit`, and report
12 is unbounded by contract, so N is not bounded. `thoth-api/AGENTS.md` section 6 requires new lists and reports to
avoid N+1 access and to use set-based SQL or batched loaders. Nothing in R52B or Amendment 3 requires one statement
per permit: the contract fixes which permits a report returns, their order, and each membership and its order.

`membership_of` is correct for what it was written for, `permit_with_dois`, which reads one known permit inside the
mutations. That path is unchanged.

### 19.5 Test mechanism

The regression counts the statements a report runs, at run time, on the real disposable PostgreSQL:

- Diesel 2.3's per-connection instrumentation (`Connection::set_instrumentation`) receives
  `InstrumentationEvent::StartQuery` for every statement a connection starts, with the statement's text. The repository
  already counts statements from that event: `graphql::dataloader::fixture::SqlProbe`. That fixture is private to the
  `graphql` module and installs a process-wide default, so the model tests cannot reach it without a fourth path.
- The new tests install the same hook more narrowly. A test builds its own one-connection pool over
  `test_db::test_db_url()`, as `x10_a_pool_whose_only_connection_is_held…` already does, with an r2d2
  `CustomizeConnection` that sets the instrumentation on that pool's connection only. Both reports take `&PgPool` and
  run wholly inside `work_upsert_transaction`, so every statement of a report runs on the measured connection.
- Nothing process-wide is installed (`set_default_instrumentation` is not used). There is no database extension, no
  production hook, no new dependency and no path outside `tests.rs`. Fixture writes use the ordinary test pool and are
  not measured.
- The assertion is exact: the number of logged statements naming `crossref_write_permit_doi`, and the number of
  statements of a report over five permits against the same report over one permit.

### 19.6 RED

The regression test was written before any production change. Every RED run used:

```text
cargo test -p thoth-api --features backend --lib -- \
  model::crossref_write_permit::tests::merge_readiness_correction
```

1. With only `tests.rs` changed and `crud.rs` byte-identical to `86917f9f` (blob `89b557f3…`), the test as first
   written stopped at its first assertion: `crossrefWritePermits returned 5 permits and read the membership table 5
   times`, `left: 5`, `right: 1`.
2. The test was then changed, still before any production change, to observe both reports before asserting
   (`tests.rs` blob `eb530b6c…`). Against the same unchanged `crud.rs`:

```text
(membership reads of report 11, of report 12, statements of report 11, of report 12) over 5 permits, against one
membership read and the statements of a one-permit page
report 11: [ "BEGIN", "SELECT … FROM \"crossref_write_permit\" … LIMIT $1 OFFSET $2",
             "SELECT doi FROM public.crossref_write_permit_doi WHERE permit_id = $1 ORDER BY doi COLLATE \"C\"" × 5,
             "COMMIT" ]
report 12: [ "BEGIN", "SELECT * FROM public.crossref_write_permit WHERE … ORDER BY issued_at, permit_id",
             "SELECT doi FROM public.crossref_write_permit_doi WHERE permit_id = $1 ORDER BY doi COLLATE \"C\"" × 5,
             "COMMIT" ]
one permit: [ "BEGIN", "SELECT … LIMIT $1 OFFSET $2", "SELECT doi FROM … WHERE permit_id = $1 …", "COMMIT" ]
  left: (5, 5, 8, 8)
 right: (1, 1, 4, 4)
test result: FAILED. 0 passed; 1 failed
```

   The five logged membership statements carried five different permit ids as their bind. That is the confirmed
   defect: one membership read per returned permit, in both reports.
3. The tests of section 19.9 were added after the correction, and the regression's fixture was then shared with
   them, so its permits alternate between two publishers; its assertions did not change. The committed `tests.rs`
   (blob `6f577759…`) was therefore run once more against the `86917f9f` `crud.rs`, swapped into the worktree for that
   run only and restored afterwards (blob `1b988a9a…` re-verified): `test result: FAILED. 4 passed; 2 failed`. The two failures are the two
   statement-count tests: the regression, `(5, 5, 8, 8)` against `(1, 1, 4, 4)` again, and the 120-permit test,
   `left: 120`, `right: 1` for report 12. The four tests that pass against the old source are the behaviour tests, so
   they describe what the reports already returned.

### 19.7 The source correction

```text
git diff --numstat 86917f9f -- thoth-api/src
42      4       thoth-api/src/model/crossref_write_permit/crud.rs
584     0       thoth-api/src/model/crossref_write_permit/tests.rs
```

`crud.rs` (blob `1b988a9a310a54fb96ac5ecc3bf5fc74882c5c7b`):

1. New private `memberships_of(connection, permit_ids)`. For an empty slice it returns an empty map and runs no
   statement. Otherwise it runs one statement, with the ids bound as one `uuid[]`, and groups the rows into
   `HashMap<Uuid, Vec<String>>` in row order:

   ```text
   SELECT permit_id, doi FROM public.crossref_write_permit_doi
    WHERE permit_id = ANY($1) ORDER BY permit_id, doi COLLATE "C"
   ```

2. `with_dois` collects the ids of the permits it was given, calls `memberships_of` once, and then maps the permits
   vector in its own order, taking each permit's DOIs out of the map by `permit_id`. It is the shape
   `load_job_payloads` already uses for `workUpsertJobs` and `workUpsertJob`.
3. `use std::collections::HashMap;`.

Unchanged: both report functions, their permit statements, filters, `limit.max(0)` and `offset.max(0)`, their order
`(issued_at, permit_id)` and their signatures; `membership_of` and `permit_with_dois`; every reservation,
finalisation, report, void, reconciliation and floor operation. No migration, `schema.rs`, GraphQL, error definition,
policy, enum, dependency, workflow, changelog or task record changed, and no file was added.

### 19.8 Semantics

- **Permit order.** It comes only from the report's own permit statement. The membership statement's row order
  decides nothing but the order of DOIs inside one permit, and the map is read by `permit_id`.
- **DOI order.** `ORDER BY permit_id, doi COLLATE "C"` makes each permit's rows arrive ascending by code point, the
  order `membership_of` gives. `(permit_id, doi)` is the table's primary key, so that order is total.
- **Membership identity.** Rows are grouped by their own `permit_id`, so a permit receives exactly the rows the table
  holds for it. `permit_id` is the permit table's primary key and both permit statements read that table alone, so an
  id occurs once in a report and taking its entry out of the map loses nothing.
- **Snapshot.** Both reports were, and remain, one `READ COMMITTED` transaction whose statements each take a
  snapshot. Membership rows are inserted in the transaction that inserts their permit and can never be updated or
  deleted (`crossref_write_permit_doi_immutable`), so any statement that runs after the permit statement sees the
  complete membership of every permit it returned. One later statement returns what N later statements returned.
- **Empty report.** No id, no membership statement, an empty list.
- **A permit without membership rows** cannot exist: `doi_set_cardinality >= 1`, and the deferred
  `crossref_permit_membership_agreement` triggers compare it with the row count at commit. Both the old and the new
  helper would give such a permit an empty list.
- **Errors.** The new statement's `?` is inside the same `work_upsert_transaction` closure, so a database error takes
  the same scoped conversion as before. Authorization is the resolver's and is untouched.
- **Cost.** A report now runs two statements for any N ≥ 1, and one for N = 0.

### 19.9 Tests

All in `thoth-api/src/model/crossref_write_permit/tests.rs`, module `merge_readiness_correction`, on the real disposable
PostgreSQL. The fixture issues legacy-route permits over two covered publishers, alternating, with memberships of
three, one, two, one and three DOIs. The chapters are related in an order that is not canonical, and the DOI suffixes
differ in `-`, `.` and `_`, which code-point order and a linguistic collation rank differently (checked outside the
repository: `COLLATE "en-US-x-icu"` orders the first and last memberships differently from `COLLATE "C"`).

| Test | What it establishes |
|---|---|
| `a_permit_report_reads_the_memberships_it_returns_in_one_statement` | the regression of section 19.6: five permits, one membership statement in each report, and as many statements as a one-permit page |
| `both_reports_return_each_permit_in_issue_order_with_its_own_membership` | permits are issued until issue order differs from `permit_id` order, the order of the membership statement; each report equals, entry for entry, the unchanged single-permit read `permit_with_dois` in issue order; every membership equals the literal canonical list, the reservation's `dois`, the rows the test reads from the table itself, and `doi_set_cardinality` |
| `report_11_keeps_its_filters_and_pagination` | over permits in `RESERVED`, `AUTHORIZED`, `INDETERMINATE`, `VOIDED` and `ACCEPTED`: no filter; each publisher; one root Work; one state, several states, a state with no permit; a publisher and states together; a job and an attempt no permit has; pages `(2,0)`, `(2,2)`, `(2,4)`, `(2,5)`, `(100,3)`, a page of a filtered report; `limit` 0 and -1; `offset` -3. Each result equals the expected slice of whole entries |
| `report_12_returns_every_blocking_permit_oldest_first` | of those five, exactly the `RESERVED`, `AUTHORIZED` and `INDETERMINATE` permits, oldest first, with their three-, one- and two-DOI memberships; `crossref_blocking_write_permit_count` agrees |
| `a_report_over_more_permits_than_a_default_page_is_complete_and_reads_the_memberships_once` | 120 blocking permits: report 12 returns all 120 in issue order, each with its own DOI; report 11 returns the resolver's default page of 100 and then the remaining 20; one membership statement each time |
| `an_empty_report_is_empty_and_reads_no_membership` | with no permit, and with five `VOIDED` permits under a filter, a `limit` of 0 and report 12: an empty list and no membership statement |

Unchanged and passing: `reports_11_to_14_list_permits_unresolved_permits_the_floor_and_drain`, which covers the job
and attempt filters on a `WORK_UPSERT` permit; the frozen query-field list and T257's superuser-only check of report
12 through GraphQL; and `tests/work_upsert_lifecycle.rs`, which reads report 11 through GraphQL by job identity
and state.

### 19.10 GREEN and validation

- With `tests.rs` unchanged from RED run 2 (blob `eb530b6c…`), the regression test passed once `crud.rs` was
  corrected: `1 passed`, and `model::crossref_write_permit::tests` `100 passed`. With the committed `tests.rs`: module
  `merge_readiness_correction` `6 passed`, and `model::crossref_write_permit::tests` `105 passed` (99 existing and 6
  new), `0 failed`.
- On the worktree whose `crud.rs` and `tests.rs` are the committed blobs (`1b988a9a…` and `6f577759…`), before this
  section was written:

```text
cargo fmt --all -- --check                                      exit 0
cargo clippy --all --all-targets --all-features -- -D warnings  exit 0 (proc-macro-error2 notice only)
cargo test --workspace --no-fail-fast                           exit 0; 1729 passed, 0 failed, 8 ignored
thoth (bin)                                          31 passed
thoth-api lib                                      1458 passed (1452 + 6)
thoth-api tests/crossref_permit_concurrency           3 passed
thoth-api tests/crossref_serializer_dependency_guard  4 passed
thoth-api tests/graphql_permissions                  13 passed
thoth-api tests/work_upsert_capture                  31 passed
thoth-api tests/work_upsert_deletion                  4 passed
thoth-api tests/work_upsert_lifecycle                 7 passed
thoth-api-server lib 3; thoth-client lib 4; thoth-errors lib 11; thoth-export-server lib 152
doc-tests: thoth_client 6; thoth_export_server 2; thoth_api 0 passed, 8 ignored
```

- Within that run: `model::crossref_write_permit::tests` 105, `model::distribution_job::tests` 74,
  `model::work_upsert::tests` 99 and `graphql::` 163 passed. Among them: the static gates over `crud.rs` (T253, T203,
  F16/T167, X1, X3, X11 and `only_finalisation_moves_a_permit_to_authorized`), and S1, evaluated.
- **API.** The GraphQL SDL the build generates (`thoth-client/assets/schema.graphql`, untracked) is byte-identical to
  the one generated from `86917f9f`: SHA-256 `6ee55071f58a7593163118ce1e9522e655bbcb6750b18225a726c90a147c2311`.
- This section is the only later change, and it is documentation. `cargo fmt --all -- --check` and module
  `merge_readiness_correction` were run again on the worktree that carries it, before the commit. S1 reads the
  committed diff, so it is run again on the commit before the push; the handoff records that result.
- **Deadlocks.** `pg_stat_database.deadlocks` rose from 0 to 1 over the workspace run. The one `deadlock detected`
  entry in the disposable server's log is the pair of `UPDATE title` statements of `t319_schedule`, the deadlock
  `t319_negative_control_without_the_defence_deadlocks` requires (section 18.10). That log holds no error for either
  membership statement.
- Tests ran only against a disposable local PostgreSQL 17.10 and Redis. `THOTH_EXPORT_API`, a compile-time string,
  was set to a localhost URL. Nothing contacted Crossref, a provider or production.

### 19.11 Diff

`git diff --name-status 86917f9f909a8e84f0682af72b48d43245d9858e..HEAD`:

```text
M	docs/engineering/ai-delivery/implementation-reports/BE-06-implementation-report.md
M	thoth-api/src/model/crossref_write_permit/crud.rs
M	thoth-api/src/model/crossref_write_permit/tests.rs
```

The three authorized paths and no fourth; all modifications; no addition, deletion, rename or copy. This report
gains this section and the third paragraph of the note at its head, and loses no line.

The correction is one commit whose only parent is `86917f9f909a8e84f0682af72b48d43245d9858e`. A commit cannot contain
its own SHA or tree, and the authorization allows one commit, so they are not written here: they are the head of PR
#925 after the push and are given in the handoff. The two source blobs above identify the corrected source
independently of this file.

### 19.12 Record, side effects and remaining gate

- **Model.** Claude Fable 5.1 (`claude-fable-5-1`); role: implementation; reasoning: maximum. The task brief
  recommended Claude Sonnet 5 at high reasoning; the session's model was not changed.
- **Limitation.** The implementing session is the one that performed the independent exact-head review of `86917f9f`
  recorded in `5715923725`, which did not raise this finding. It is not independent for any later BE-06 review.
- **Repository writes:** the three paths above, on the task branch only.
- **Commit:** one, as Javier Arias, without AI attribution. No earlier commit was amended, rebased or rewritten.
- **Push:** `feature/publisher-services-v1-10--be-06` only, a fast-forward from `86917f9f` without force. The remote
  branch, the integration target, PR #925 and #848 are re-checked immediately before it.
- **CI:** the push naturally triggers PR #925's `build-test-and-check`, `run-migrations`, `publish-to-dockerhub`
  (the `staging-pr-925` image) and `check-changelog` workflows, as `5718898530` authorizes. They are observed
  read-only; none is dispatched, re-run or cancelled. Their run IDs do not exist until after the push and are given
  in the handoff.
- **No** #848 comment, PR body or metadata edit, review-thread reply or resolution, PR #927 change, merge, release,
  deployment, migration outside the disposable database, G-6, G-7, provider or Crossref access, production access or
  activation.
- **Gate:** the implementation agent does not approve its own work. `5715923725` approved `86917f9f` only and is
  historical once this commit is pushed. The pushed head requires a fresh independent cold-start CRITICAL review of
  the entire BE-06 implementation, not only this correction, before any new merge authorization. Both automated
  review threads on PR #925 remain open for the control plane.
