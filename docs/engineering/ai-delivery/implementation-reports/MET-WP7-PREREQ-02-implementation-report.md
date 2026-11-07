# MET-WP7-PREREQ-02 Implementation Report

Task: `MET-WP7-PREREQ-02 - Durable unresolved-DOI quarantine and source-progress semantics`
Programme: Thoth Metrics / MOM-1 ([#766](https://github.com/thoth-pub/thoth/issues/766))
Risk: **CRITICAL**

Authority condition: this report records the implementation as committed on
the task branch. Live review, authorization, CI and merge evidence is GitHub
issue and pull-request history. The implementing agent does not approve its
own work.

## 1. Repository state

| Item | Value |
|---|---|
| Owning GitHub issue | [#930](https://github.com/thoth-pub/thoth/issues/930) |
| Repository | `thoth-pub/thoth` |
| Workflow | `PROGRAMME_INTEGRATION` |
| Base branch | `feature/metrics` |
| Authorized base commit | `15b6efd089dee25d988cbb316c082be1780bf56f` (tree `53411810ad68014598a32376f6e4e929465002e8`) |
| Actual base commit | `15b6efd089dee25d988cbb316c082be1780bf56f` (tree `53411810ad68014598a32376f6e4e929465002e8`), parent of the first task commit |
| PR target / programme integration branch | `feature/metrics` |
| Task branch | `feature/metrics--wp7-prereq-02-doi-quarantine` |
| Head commit | the head of PR #933: a commit cannot record its own SHA; the task commits are listed in section 3 |
| Pull request | [#933](https://github.com/thoth-pub/thoth/pull/933), DRAFT, from the task branch into `feature/metrics` |
| Expected branch deletion after merge | YES |
| Final programme PR required | YES (`feature/metrics -> develop`, separately authorized) |
| Implementing model | Claude Opus 5 (`claude-opus-5`) |
| Implementation session | scratchpad `fe1167b6-ed8c-458d-b37e-a62eb2c8d981` |
| Observed `develop` | `395cc16ac770bc8bbf8a708662a1b31d85b15398` (unchanged from authorization) |
| Migration | `thoth-api/migrations/20260921_v1.9.0` |

### 1.1 Authorization provenance

| Record | Location |
|---|---|
| Approved and frozen specification (never edited after creation) | #930 issue body |
| CTO CRITICAL implementation authorization (14-path budget, exact base, branch, migration) | #930 comment `5759028300` |

Downstream: `thoth-pub/thoth-sphinx#18` stays OPEN, DRAFT and unmerged at
`52a535a170b229a4a59530c92fb4fdc433434bd6`; it was not modified.

### 1.2 Preflight (2026-09-21, before any mutation)

All eight authorized preflight checks passed, read-only:

1. live `feature/metrics` = `15b6efd0…` (`git ls-remote` and REST);
2. its tree = `53411810…`;
3. `feature/metrics--wp7-prereq-02-doi-quarantine` absent locally and remotely;
4. no PR for the task (by head branch, by `930`, `quarantine` and `PREREQ-02`
   searches, and no open PR into `feature/metrics`);
5. no competing session or worktree (peer sessions listed, none on this task;
   no worktree or scratch directory for it);
6. `thoth-api/migrations/20260921_*` absent on the authorized base, live
   `feature/metrics`, `develop`, all 90 remote branches and every open PR;
7. #930 OPEN, body never edited (`userContentEdits` empty), the authorization
   comment its only comment;
8. Sphinx #18 frozen as above.

The base was re-verified with `git ls-remote` immediately before the branch
was created from exactly `15b6efd0` in an isolated worktree outside the
shared clone.

## 2. Scope confirmation

Approved specification: the #930 issue body, frozen and unedited; implementation
authority #930 comment `5759028300`.

Implemented objective: Thoth owns durable unresolved-DOI quarantine. An
eligible CloudFront observation whose DOI parses but resolves to no Thoth work
stays `REJECTED` / `UNKNOWN_DOI`, creates no canonical record, revision or
rollup delta, is counted in `invalid_count`, and — only when eligible — gets
exactly one `metric_identifier_quarantine` row in the same coordinator
transaction. A CloudFront `COMPLETED_WITH_ERRORS` import consisting only of
durably quarantined `UNKNOWN_DOI` rows with exact one-day `COMPLETE` coverage
records its period-manifest cursor entry while remaining canonically
unsuccessful; provably inconsistent evidence fails closed; everything else
keeps its existing behaviour.

Out-of-scope changes made: NONE.

## 3. Commits

- `12a2a48e10f83b9b204080d341903327b5c19c3a` — tree
  `3be05fcaf23db3096927a6e94126dcd1786099b2`, parent
  `15b6efd089dee25d988cbb316c082be1780bf56f` — `MET-WP7-PREREQ-02: add
  durable CloudFront unresolved-DOI quarantine`. It contains the migration,
  schema, model, coordinator, lifecycle, tests, contract register and
  changelog: 12 paths.
- The commit adding this report (path 14) follows it. A commit cannot record
  its own SHA, so its identity is the PR head.

## 4. Files changed

The 14-path budget of #930 / `5759028300`; 13 paths used.

| # | Authorized path | Status | Reason and behavioural effect |
|---|---|---|---|
| 1 | `CHANGELOG.md` | M | One `Unreleased / Added` entry for `MET-WP7-PREREQ-02`. No behaviour. |
| 2 | `docs/metrics/contract-register.md` | M | New section 3.4 records the narrow CloudFront quarantine contract; the section 3.3 cursor sentence names that single exception. No behaviour. |
| 3 | `thoth-api/migrations/20260921_v1.9.0/up.sql` | A | Creates `metric_identifier_quarantine` only. |
| 4 | `thoth-api/migrations/20260921_v1.9.0/down.sql` | A | Locks the table, refuses while populated, otherwise drops it. |
| 5 | `thoth-api/src/schema.rs` | M | The table block, four `joinable!` entries and the `allow_tables_to_appear_in_same_query!` entry, maintained by hand per ADR-0003. |
| 6 | `thoth-api/src/model/mod.rs` | M | Declares `metric_identifier_quarantine`. |
| 7 | `thoth-api/src/model/metric_identifier_quarantine/mod.rs` | A | The `MetricIdentifierQuarantine` Diesel model and its domain documentation. No behaviour of its own. |
| 8 | `thoth-api/src/model/metric_identifier_quarantine/tests.rs` | A | Schema, constraint, DOI-verbatim, Diesel-mapping and migration evidence. |
| 9 | `thoth-api/src/model/metric_ingestion/mod.rs` | M | `RequestScope` retains the already-locked source; eligible `UNKNOWN_DOI` rejections write one quarantine row; `insert_provenance` returns its generated id. |
| 10 | `thoth-api/src/model/metric_ingestion/tests.rs` | M | `DurableState` also counts quarantine rows; nine new ingestion tests. |
| 11 | `thoth-api/src/model/metric_ingestion_lifecycle/mod.rs` | M | `update_metric_source_checkpoint` locks account and source after checkpoint and import, derives CloudFront quarantine evidence, and records the quarantine-only manifest. |
| 12 | `thoth-api/src/model/metric_ingestion_lifecycle/tests.rs` | M | Seven new lifecycle tests, including the lock-order proof. |
| 13 | `thoth-api/src/graphql/metric_ingestion_lifecycle_tests.rs` | — | **Not used.** No GraphQL source changed and the existing frozen-contract tests pass unchanged, so no GraphQL test change was needed (section 7). |
| 14 | `docs/engineering/ai-delivery/implementation-reports/MET-WP7-PREREQ-02-implementation-report.md` | A | This report. |

Files deleted, moved or renamed: NONE.

### 4.1 Write-budget compliance

PASS. 13 of the 14 authorized paths; every changed path is on the list, and
the conditional path 13 is unused. No policy, auth, role, `thoth-errors`,
Cargo, lockfile, workflow, GraphQL production source, dashboard/read source,
Sphinx or generated-artifact path was touched. `make migration` would create
exactly `thoth-api/migrations/20260921_v1.9.0` (Cargo version `1.8.0`, date
`20260921`); the directory was created with that name directly.

### 4.2 Authorized actions actually used

- repository inspection: yes
- source edit: yes, within the budget
- new file creation: yes, the five authorized new paths (3, 4, 7, 8, 14)
- file deletion/move/rename: no
- branch creation: yes, from exactly `15b6efd0`
- commit: yes
- push: yes, the task branch only
- PR creation/update: yes, one DRAFT PR into `feature/metrics`
- issue/comment mutation: no
- manual CI dispatch/rerun: no
- provider/runtime read or write: no
- migration execution: local disposable databases only; no shared, staging or
  production database
- release/tag/publication, merge, deployment, production activation: no

Unauthorized actions performed: NONE.

### 4.3 Automatic and manual external effects

Automatic effects of the push and the DRAFT PR: the repository's natural
`pull_request` workflows, with no manual dispatch, rerun or cancel.

- `check-changelog`.
- `build-test-and-check`.
- `run-migrations`: migrate, revert and reapply against the workflow's own
  ephemeral Postgres service container.
- `publish-to-dockerhub`: the repository's existing automatic PR staging
  image build. It pushes `ghcr.io/thoth-pub/thoth:staging-pr-933` to the
  GitHub Container Registry.

Manually initiated external actions: NONE. External writes or publication
beyond that automatic staging image: NONE. No release, tag, package or
production registry tag was created.

## 5. Implementation decisions

Decisions within the approved design:

1. **Eligibility is decided at one point.** Quarantine is planned only in the
   branch where an observation passed every input check, including
   `Doi::from_str`, and then failed resolution at the locked re-resolution
   barrier with exactly `UNKNOWN_DOI`. That guarantees a syntactically valid
   DOI and a fully validated reduced observation. The predicate is
   `code == UNKNOWN_DOI && locked source.driver_key == Some("cloudfront") &&`
   all five of `publication_isbn`, `publication_type`, `institution_ror`,
   `source_record_id`, `source_row_number` are `None`. It reuses the existing
   `metric_source_account::CLOUDFRONT_DRIVER_KEY` (`MET-WP1-13`) rather than a
   second constant, and matches exactly, with no trim or case-fold.
2. **No extra lock in the coordinator.** `RequestScope` keeps the
   `MetricSource` that `lock_request_scope` already locks `FOR SHARE`.
3. **Stored values.** `work_doi` is `observation.work_doi` verbatim;
   `schema_version` is the batch envelope's validated value;
   `source_account_id`, `platform_id` and `measure_id` are the locked rows the
   observation validated against; the rest are the observation's own fields.
4. **Transaction shape.** `insert_provenance` now returns the generated
   `record_provenance_id` (`RETURNING`), and `write_rejected` inserts the
   quarantine row after the provenance row and sanitized import error and
   before the invalid counter, all in the one coordinator transaction. The
   batch outcome row is identical with or without quarantine.
5. **Checkpoint lock order.** `update_metric_source_checkpoint` keeps
   checkpoint `FOR UPDATE` then import `FOR SHARE`, and then — after the
   existing stale-claim, terminal, one-day and envelope checks, and for every
   terminal one-day import — locks the source account and then the source
   `FOR SHARE`. Stale, foreign, expired and non-terminal calls return before
   these two locks exactly as before.
6. **Evidence derivation.** For a CloudFront `COMPLETED_WITH_ERRORS` import
   only, one grouped query over the import's `REJECTED` provenance rows
   (`jsonb_typeof` and text of `details.reason_code`, row count, quarantine
   count via `LEFT JOIN` on the unique provenance key) and one count of
   quarantine rows joined to any provenance row of the import give
   `rejected`, `unknown_doi`, `quarantine` and `quarantined_unknown_doi`. Each
   stored reason must be a JSON string parsing through the closed
   `MetricIngestionErrorCode` (`strum`, case-sensitive).
   `metric_import_error` is never read.
7. **Three-way partition.** Inconsistent (`rejected != invalid_count`,
   `quarantine != quarantined_unknown_doi`, or a missing, null, non-string or
   unparseable reason) returns `INTERNAL_STATE_INCONSISTENCY` before any write.
   It logs only the import id and returns no stored value. Otherwise the
   quarantine-only predicate is `conflict_count == 0 && invalid_count > 0 &&
   rejected == unknown_doi == quarantined_unknown_doi == invalid_count &&
   quarantine == quarantined_unknown_doi` plus the existing exact-period
   `COMPLETE` coverage test. Anything else is the unchanged non-success path.
8. **Cursor write flag.** The single checkpoint `UPDATE` gains one bind:
   `cursor = CASE WHEN $6 THEN $5 ELSE cursor END`, with
   `$6 = successful || quarantine_only`. `last_successful_period_end` still
   uses `$2 = successful` only. The existing `record_accepted_manifest`
   encoder, replacement rule and 64-period retention are reused unchanged.
9. **Replay.** The predicate, including inconsistency detection, is derived
   before the live/released split, as the envelope decode already was. The
   released branch is unchanged: recorded completion is `last_completed_at`
   (plus the successful period only for canonical success), and the cursor
   plays no part.
10. **Interpretation recorded for review.** The #930 body derives the
    evidence "for a CloudFront `COMPLETED_WITH_ERRORS` import". The handoff
    prompt's restatement adds "with ... COMPLETE exact-period coverage". The
    implementation follows the controlling issue body. Evidence is derived,
    and contradictions fail closed, for every terminal one-day CloudFront
    `COMPLETED_WITH_ERRORS` import whatever its coverage. Coverage only gates
    eligibility. The two texts differ only for a PARTIAL, UNKNOWN or zero
    coverage import with provably contradictory evidence, which fails closed
    here instead of releasing.
11. **Optional conflict cross-check: not implemented.** No
    conflict-provenance-versus-`conflict_count` check was added, to keep the
    contract exactly as approved.
12. **Nonblank checks.** `schema_version`, `work_doi` and
    `methodology_version` reuse the locale-independent 25-code-point negated
    bracket of `metric_source_driver_key_check`. It is byte-identical and the
    exact twin of the coordinator's `trim().is_empty()`, so no
    application-valid value can be refused by the database.
13. **No DOI CHECK** (approved default). `Doi::from_str` accepts bare and
    `http(s)://(www.)(dx.)doi.org/` prefixes case-insensitively and
    Unicode-aware `\d` registrant digits, so the `work` table's canonical-form
    CHECK is not equivalent. The test suite stores six such forms, including
    Arabic-Indic registrant digits, verbatim.
14. **No country shape CHECK.** The approved constraint list has none, and
    `country_code` is validated by the coordinator (`is_assigned_alpha2`)
    before any write.

Deviations from the specification requiring authorization: NONE.

## 6. Database and migration effects

Migration added: YES, `thoth-api/migrations/20260921_v1.9.0`.

- **Schema effect.** One new table, `metric_identifier_quarantine`, with 14
  columns exactly as approved:
  - `identifier_quarantine_id uuid` PK, default `uuid_generate_v4()`;
  - `record_provenance_id uuid NOT NULL UNIQUE`;
  - `source_account_id`, `platform_id` and `measure_id`, each `uuid NOT NULL`;
  - `schema_version text NOT NULL`, `work_doi text NOT NULL`;
  - `period_start date NOT NULL`, `period_end date NOT NULL`;
  - `reporting_grain metric_reporting_grain NOT NULL`;
  - `country_code text NULL`, `value bigint NOT NULL`;
  - `methodology_version text NOT NULL`;
  - `created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP`.

  Constraints:
  - three nonblank CHECKs and `period_end > period_start`;
  - four non-cascading foreign keys, with no `ON DELETE` or `ON UPDATE`
    action, to `metric_record_provenance`, `metric_source_account`,
    `metric_platform` and `metric_measure`.

  Indexes are only the PK and the unique key. No enum, type, trigger,
  function or existing object changes.
- **`schema.rs`.** Updated by hand per ADR-0003:
  - the table block, with `country_code -> Nullable<Text>` and
    `reporting_grain` using the existing `MetricReportingGrain` SQL type;
  - four `joinable!` entries;
  - the `allow_tables_to_appear_in_same_query!` entry.
- **Existing data.** Untouched: no seed, no backfill, no rewrite of Metrics
  data.
- **Locking and downtime.** `CREATE TABLE` of a new, empty table only; no
  existing table is rewritten. Creating the four foreign keys briefly takes
  `SHARE ROW EXCLUSIVE` on `metric_record_provenance`, `metric_source_account`,
  `metric_platform` and `metric_measure`. That blocks writes, not reads, to
  those tables for the migration's short transaction.
- **Rollback.** `down.sql`:
  1. takes `LOCK TABLE … IN ACCESS EXCLUSIVE MODE`;
  2. raises `MET-WP7-PREREQ-02 rollback refused: metric_identifier_quarantine
     holds N row(s)…` if any row exists;
  3. otherwise `DROP TABLE`.

  The migration runs in one transaction, so a refusal drops nothing.
  Recovery after activation is forward repair under separate authorization.
- **Empty and populated results, apply/revert/reapply.** See section 9.
- **Production execution: NONE.** Migration execution under CG-13 and release
  authorization remains separately gated.

## 7. API and compatibility effects

- **GraphQL/API changes.** NONE: no operation, field, input or enum value is
  added, removed or changed.
- **External batch result.** For a quarantined row it is still
  `classification: REJECTED`, `reasonCode: UNKNOWN_DOI`, with no record or
  hashes.
- **Generated SDL.** NONE. `thoth-client/assets/schema.graphql` regenerated at `12a2a48e` is
  byte-identical to the one regenerated from the untouched base `15b6efd0`.
  Both are SHA-256 `6a61ba366ef911ec11743267615a2fa8e1504a3a41d51ad224ddb52f60f74d6f`,
  5,404 lines and 223,616 bytes. That is the full producer SDL hash `thoth-sphinx`
  already pins in `crates/sphinx-thoth/graphql/schema.graphql`, at both its
  `feature/metrics` `7c1e2481` and the #18 head `52a535a1`.
- **Existing GraphQL frozen-contract tests.** Unchanged and passing, including:
  - `the_sdl_exposes_exactly_the_five_approved_operations_and_no_sixth`;
  - `the_lifecycle_objects_and_inputs_match_the_frozen_contract_exactly`.
- **Backwards compatibility.** Additive.
  - Non-CloudFront sources and `COMPLETED` imports behave exactly as before.
  - A CloudFront `COMPLETED_WITH_ERRORS` import changes in exactly two ways:
    it may now record a cursor entry, and it fails closed on provably
    inconsistent evidence.
- **Cross-repository.** `thoth-sphinx` requires no SDL regeneration. The
  bounded Option C correction of `thoth-sphinx#18` is separate downstream
  work. It is gated on this merge and a rebind to the merged producer, and it
  is designed there, not here. Sphinx was not touched.

## 8. Authorization and security

- **Authorization paths changed.** NONE. No policy, role, scope or
  entitlement change.
- **Roles.** `METRICS_INGEST_SERVICE` gating of the five lifecycle operations
  is unchanged.
- **Negative authorization tests.** The existing lifecycle GraphQL denial
  tests pass unchanged.
- **Personal and request data.** The quarantine table has no IP, user agent,
  cookie, query string, referrer, request id, raw log row, session identity,
  routing or credential column. A test asserts:
  - the exact 14-column contract;
  - a 22-name deny-list;
  - no JSON, binary or network-address column.
- **Errors and logs.** Inconsistency errors are the fixed
  `INTERNAL_STATE_INCONSISTENCY` code and message. The log line names only
  the import id.
- **Caller control.** None. Quarantine cannot be requested by a caller, and
  eligibility is never inferred from codes, hostnames, buckets or prefixes.

## 9. Tests and checks

### 9.1 Environment

- Task-isolated PostgreSQL 17.10 cluster (`initdb`, UTF8, `C` collation and
  ctype) on a private port, with `thoth_test` created from `template0`.
- Task-isolated Redis.
- `CARGO_TARGET_DIR`, `TMPDIR` and the harness test lock are private to this
  session.
- Environment variables mirror `.github/workflows/build_test_and_check.yml`
  (`THOTH_EXPORT_API`, `TEST_DATABASE_URL`, `TEST_REDIS_URL`).
- `rustc 1.97.0`, `cargo 1.97.0`.
- No shared, staging or production database was used.

### 9.2 Baseline (authorized base `15b6efd0`, before any edit)

```text
cargo test -p thoth-api --features backend
test result: ok. 1912 passed; 0 failed; 1 ignored   (lib)
test result: ok. 13 passed; 0 failed                (tests/graphql_permissions.rs)
test result: ok. 0 passed; 0 failed; 8 ignored      (doctests)
```

### 9.3 Repository gate (commit `12a2a48e`, clean tree)

| Step | Command | Result |
|---|---|---|
| Formatting | `cargo fmt --all -- --check` | exit 0 |
| Backend tests | `cargo test -p thoth-api --features backend` | exit 0. Lib 1941 passed, 0 failed, 1 ignored (1912 baseline + 29 new). `graphql_permissions` 13 passed. Doctests 8 ignored. |
| Workspace tests | `cargo test --workspace` | exit 0, 0 failed. `thoth` bin 31, `thoth-api` 1941 (1 ignored), `graphql_permissions` 13, `thoth-api-server` 3, `thoth-client` 4, `thoth-errors` 13, `thoth-export-server` 144, integration suites 6 and 2, doctests 8 ignored. |
| Check | `cargo check --workspace` | exit 0 |
| Lint | `cargo clippy --all --all-targets --all-features -- -D warnings` | exit 0. The only `warning:` line is the existing `proc-macro-error2` future-incompatibility notice, not a lint. |
| Build and SDL | `rm -f thoth-client/assets/schema.graphql && cargo clean -p thoth-client && cargo build` | exit 0. SDL regenerated, SHA-256 `6a61ba366ef911ec11743267615a2fa8e1504a3a41d51ad224ddb52f60f74d6f`, 5,404 lines, 223,616 bytes. Byte-identical (`cmp`) to the SDL regenerated from the untouched base in an isolated shallow clone. |
| Whitespace | `git diff --check 15b6efd0...HEAD` | exit 0, no output |

### 9.4 Migration workflow (repository CLI, disposable databases)

Binaries: `thoth migrate` built from `12a2a48e`, and the same command built
from the untouched base `15b6efd0` in an isolated clone. `thoth migrate
--revert` reverts all migrations. Schema dumps use
`pg_dump --schema-only --no-owner` with pg_dump 17's per-dump random
`\restrict` / `\unrestrict` key lines removed before comparison.

| Step | Result |
|---|---|
| Base binary `migrate` on an empty database | exit 0; 24 migrations through `20260913` |
| Head binary `migrate` on an empty database | exit 0; 25 migrations through `20260921`; table present |
| Head vs base schema | 0 lines removed, 74 added. Every added line is the new table, its comments, constraints or keys. |
| `migrate --revert` with an empty quarantine table | exit 0; every migration reverted, `__diesel_schema_migrations` only |
| Reapply | exit 0; schema identical to the first apply |
| Only this migration's `down.sql` plus deletion of its version row, as Diesel reverts one migration | exit 0; 24 migrations; schema identical to the base binary's |
| Reapply after that | exit 0; schema identical to the first apply |
| Populated revert (one quarantine row with valid references) | exit 1 with `MET-WP7-PREREQ-02 rollback refused: metric_identifier_quarantine holds 1 row(s)`. Still 25 migrations, the row is present, and the schema is identical, so nothing was dropped. |

The embedded-runner tests in section 9.5 prove the same properties through
`MigrationHarness`, including a rollback started while an uncommitted insert
holds the table. It waits on its `ACCESS EXCLUSIVE` lock, sees the committed
row and refuses.

### 9.5 Evidence map (new tests)

**Schema and migration** (`metric_identifier_quarantine::tests`, 13 tests):

- **Contract:** `migration_seeds_no_quarantine_row`;
  `the_quarantine_table_has_exactly_the_approved_column_contract` (14 columns
  with exact type, nullability and default; 22-name request-identity and
  resolution-state deny-list; no JSON, binary or inet column);
  `the_quarantine_table_has_exactly_the_approved_constraints_and_no_doi_rule`
  (four named CHECKs, only the PK and unique indexes, four FKs with no
  `ON DELETE` or `ON UPDATE`, no DOI shape).
- **Constraints:** `database_defaults_apply_and_every_required_column_is_not_null`;
  `blank_schema_versions_dois_and_methodologies_are_rejected_locale_independently`
  (including U+00A0, U+2003, U+3000 and U+202F);
  `the_period_must_be_ordered_and_half_open`;
  `exactly_one_quarantine_row_may_reference_one_provenance_row`;
  `every_reference_must_exist_and_no_parent_deletion_cascades`.
- **DOI and mapping:**
  `every_application_valid_doi_form_is_stored_verbatim_with_no_database_doi_rule`
  (six `Doi::from_str`-valid forms, including Arabic-Indic registrant digits);
  `quarantine_rows_map_through_diesel`.
- **Migration:**
  `the_migration_applies_from_its_exact_predecessor_and_an_empty_rollback_restores_it`
  (catalogue fingerprint: the predecessor is a strict subset, every addition
  names the table, up/down/up are exact);
  `a_populated_quarantine_refuses_rollback_before_dropping_anything`;
  `a_rollback_racing_a_concurrent_insert_waits_for_it_and_then_refuses`.

**Ingestion** (`metric_ingestion::tests`, 9 tests):

- **Eligible unknown:**
  `an_eligible_cloudfront_unknown_doi_is_rejected_with_exactly_one_quarantine_row`.
  It proves exact column values, the byte-exact non-canonical DOI
  `HTTP://DX.DOI.ORG/10.12345/Unresolved-Case`, a sanitized error, no
  record, revision or delta, and counters `[1,0,0,0,0,1]`.
- **Known DOI:** `a_known_cloudfront_doi_is_unchanged_and_never_quarantined`
  (winner, duplicate and revision).
- **Replay:**
  `a_replayed_quarantine_batch_writes_nothing_and_never_duplicates_quarantine`
  (immediate, fresh-pool and concurrent double submission).
- **Rollback:** `a_failed_quarantine_write_commits_no_provenance_error_counter_or_batch`
  (failures injected on the quarantine insert, at quarantine COMMIT, on the
  rejected provenance, on the import error and at counter COMMIT, then a
  clean commit).
- **Never quarantined:** `an_invalid_doi_is_never_quarantined`;
  `a_non_cloudfront_driver_unknown_doi_is_never_quarantined` (six key
  variants, then code and routing independence);
  `only_unknown_doi_rejections_are_ever_quarantined`.
- **Mixed batch:**
  `a_mixed_batch_keeps_every_valid_canonical_row_and_quarantines_only_eligible_unknowns`.
- **Excluded fields:** `each_quarantine_excluded_optional_field_keeps_an_ordinary_rejection`
  (each of the five fields).

`DurableState` now also counts quarantine rows, so every existing
zero-consequence and atomicity test also proves that no quarantine row
leaks.

**Lifecycle** (`metric_ingestion_lifecycle::tests`, 7 tests):

- **Quarantine-only acceptance:**
  `an_eligible_quarantine_only_import_records_its_manifest_but_never_claims_success`
  (fresh and alongside canonical success; released replay is read-only).
- **Cursor rules:**
  `a_quarantine_only_manifest_replaces_its_period_and_keeps_the_retention_bound`.
- **Ineligible partition:**
  `a_consistent_but_ineligible_completed_with_errors_import_releases_without_a_manifest`.
  Cases: an A1-ineligible unknown, mixed reasons, a partly quarantined
  unknown set, `PARTIAL`, `UNKNOWN`, zero coverage, `COMPLETE`+`PARTIAL`, a
  mismatched period, and a real conflict.
- **Inconsistent partition:**
  `provably_inconsistent_quarantine_evidence_fails_closed_with_no_write`.
  Ten corruptions, each proven with no write and the lease still held, on
  both the live and the released path. Restored, the update succeeds.
- **Non-CloudFront:**
  `a_non_cloudfront_completed_with_errors_import_keeps_its_existing_behaviour`
  (contradictory counters are never evaluated).
- **Tokens:**
  `stale_expired_reclaimed_and_foreign_tokens_never_record_a_quarantine_only_manifest`.
- **Lock order:** `the_checkpoint_update_locks_checkpoint_import_account_then_source`.
  Holding the import, the account and then the source, `NOWAIT` probes show
  which rows the queued update already holds, which it has not reached, and
  that no deadlock occurred.

Unchanged behaviour is also proven by the existing suites, which pass
unmodified, including:

- clean `COMPLETED` + `COMPLETE`: the successful-period and cursor tests;
- `completed_with_errors_releases_the_lease_but_never_claims_success`, an
  A1-ineligible unknown DOI;
- retention and eviction, released replay, and stale tokens;
- the current-versus-stale update race;
- the GraphQL frozen-contract and authorization tests.

### 9.6 Mutation checks

Each mutation was applied to the working tree, the targeted test was run, and
the file was restored from a byte-for-byte backup:

| Mutation | Result |
|---|---|
| Drop the `COMPLETE`-coverage requirement from quarantine-only acceptance | KILLED |
| Drop the `source_row_number` exclusion | KILLED |
| Accept inconsistent evidence | KILLED |
| Write the cursor only on canonical success | KILLED |
| Lock the source before the account | KILLED |
| Quarantine for any driver key | KILLED |

### 9.7 Defect found and fixed during implementation

The first draft of `up.sql` lost its `\uXXXX` regex escapes in the editing
tool, so the nonblank CHECKs accepted blank values. The locale-independence
test caught it. The bracket was regenerated programmatically and is now
byte-identical to `metric_source_driver_key_check`,
before any commit.

## 10. Manual verification

- **Environment:** the task-isolated cluster above.
- **Steps:**
  - section 9.4 `psql` and `pg_dump` inspection;
  - reading back the stored constraint definition
    (`pg_get_constraintdef`) for `metric_identifier_quarantine_work_doi_check`.
- **Observed result:** as recorded in 9.4.

## 11. CI

Natural PR-triggered CI only, with no manual dispatch, rerun or cancel.

For the implementation commit `12a2a48e10f83b9b204080d341903327b5c19c3a`
(PR #933 opened DRAFT), every workflow concluded `success`:

| Run | Workflow | Jobs |
|---|---|---|
| `35595653783` | `build-test-and-check` | `classify`, `build`, `test`, `lint`, `format_check`: all success |
| `35595653590` | `run-migrations` | `classify`, `run_migrations`: success |
| `35595653586` | `publish-to-dockerhub` | `classify`, `build_and_push_staging_docker_image`: success |
| `35595653584` | `check-changelog` | `check-changelog`: success |

The commit adding this report changes documentation only. The CI change
classifier diffs the whole PR (`base...head`), so its natural runs execute the
same jobs. Their identifiers and conclusions for the final head are in the PR
history.

## 12. Rollout and rollback

- **Initial state after merge.** The table exists and is empty. The new code
  paths are reachable only for a source whose `driver_key` is exactly
  `cloudfront`, and no such production source is approved or activated.
- **Activation required.** Yes, separately. CloudFront production activation
  stays prohibited until reconciliation (or an approved operational
  substitute) and the identifier-quality/read-warning gate exist.
- **Feature flag / configuration.** None.
- **Migration sequence.** Forward only, after the existing `20260913_v1.9.0`,
  under CG-13 release authorization.
- **Rollback.**
  - Code: revert the merge.
  - Schema: `down.sql` while the table is empty; once any row exists, forward
    repair under separate authorization.
- **Monitoring.** `INTERNAL_STATE_INCONSISTENCY` from
  `updateMetricSourceCheckpoint`, and the log line
  `metric import <id> holds inconsistent rejection or quarantine evidence`.

## 13. Known limitations and deferred work

- **Automatic quarantine reconciliation.** Out of scope and separately
  bounded: detecting a newly resolvable DOI, revalidating preserved evidence,
  idempotently applying canonical metrics, and recording resolution/audit
  state.
- **Identifier-quality and read warnings.** The `metricDashboard` read
  contract is unchanged and still downgrades `COMPLETED_WITH_ERRORS`
  coverage to `PARTIAL`.
- **Cursor-only progress.** A day accepted only through the quarantine
  exception advances the cursor and `last_completed_at` but never
  `last_successful_period_end`. Consumers must not read that field as "every
  day through here is canonically complete" for such days.
- **What the lifecycle can and cannot tell apart.** It cannot reconstruct the
  normalized payload, so an unquarantined `UNKNOWN_DOI` row is treated as
  deliberately ineligible rather than as corruption, exactly as approved.
- **Downstream order.**
  1. Independent exact-head CRITICAL review.
  2. CTO merge authorization.
  3. Merge.
  4. Sphinx #14 / #18 rebind and Option C correction.
  5. Reconciliation and read-quality gates before production activation.

## 14. Unresolved issues

- NONE blocking. Section 5 item 10 records one interpretation for the
  reviewer's attention.

## 15. Agent self-assessment

The implementing agent does not approve this work. Suggested review focus:

- **The three-way partition.** Check that no consistent-but-ineligible state
  can reach `INTERNAL_STATE_INCONSISTENCY`, and no contradictory state can
  release.
- **Coverage scope of the inconsistency check.** It covers any CloudFront
  `COMPLETED_WITH_ERRORS` import, not only COMPLETE ones (section 5 item 10).
- **The added account/source `FOR SHARE` locks** on every terminal one-day
  checkpoint update, including `COMPLETED` imports. Check their ordering
  against the source/source-account administration coordinators, which lock
  one row `FOR UPDATE` and wait on nothing the lifecycle holds.
- **Byte-exactness of `work_doi`** and the deliberate absence of a DOI CHECK.
- **The `down.sql` table lock and guard.**
