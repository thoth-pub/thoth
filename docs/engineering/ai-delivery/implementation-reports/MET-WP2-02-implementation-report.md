# MET-WP2-02 Implementation Report

Task: `MET-WP2-02 - Protected Sphinx lifecycle + ingestion GraphQL`
Programme: Thoth Metrics / MOM-1 ([#766](https://github.com/thoth-pub/thoth/issues/766))
Risk: **CRITICAL**

Authority condition: this report records the implementation as committed on
the task branch. Live review, authorization, CI and merge evidence is GitHub
pull-request history.

## 1. Repository state

| Item | Value |
|---|---|
| Owning GitHub issue | [#908](https://github.com/thoth-pub/thoth/issues/908) |
| Repository | `thoth-pub/thoth` |
| Workflow | `PROGRAMME_INTEGRATION` |
| Base branch | `feature/metrics` |
| Authorized base commit | `04b5ef6a0fb5cfee24095a8614555d5f9a20bf7e` (tree `e45ae9fa3277430265bfea4c5a9dffbc953a34db`) |
| Actual base commit | `04b5ef6a0fb5cfee24095a8614555d5f9a20bf7e` (parent of the implementation commit) |
| PR target / programme integration branch | `feature/metrics` |
| Task branch | `feature/metrics--wp2-02-sphinx-ingestion-graphql` |
| Head commit | the PR head: a commit cannot record its own SHA; the implementation commit is listed in section 3 and the exact head is visible on the DRAFT PR |
| Pull request | one DRAFT PR from the task branch into `feature/metrics` |
| Expected branch deletion after merge | YES |
| Final programme PR required | YES (`feature/metrics -> develop`, separately authorized) |
| Implementing model | Claude Opus 5 |
| Reasoning level | default |
| Observed `develop` | `395cc16ac770bc8bbf8a708662a1b31d85b15398` |

### 1.1 Authorization provenance

| Record | #908 comment |
|---|---|
| Original specification | issue body |
| Specification Amendment 1 (import envelope, checkpoint lifecycle) | `5635972749` |
| Specification Amendment 2 (bootstrap, lease token, frozen GraphQL lifecycle, transaction model) | `5636364932` |
| Specification Amendment 3 (non-vacuous coverage, frozen result/error contract) | `5636932061` |
| Final functional specification approval | `5637086667` |
| Implementation rebind / path-budget amendments and footprint reviews (historical) | `5649099725`, `5649136472`, `5652293957`, `5652350457`, `5652564960`, `5652601971` |
| Superseded authorizations bound to `6b9ba1b3` (inactive; not used) | `5652359088`, `5653377613` |
| Exact-base rebind amendment (`04b5ef6a`, 15 paths) | `5653897262` |
| Fresh exact-base / exact-15-path footprint approvals | `5653922947`, `5654244786` |
| CTO implementation re-authorizations (rebound base, 15 paths) | `5654247017`, `5654298634` |

### 1.2 Preflight and branch history

1. Under `5652359088` the task branch was created locally at `6b9ba1b3`. Before
   any source edit, two out-of-budget predecessor SDL guards were proven to
   conflict with the frozen contract, and implementation stopped (HOLD). No
   source change was made.
2. Under `5653377613` (16 paths, still `6b9ba1b3`) the resume preflight found
   `feature/metrics` had moved to `04b5ef6a` (PR #916 / MET-WP4-02 merged), so
   implementation stopped again before any source edit (HOLD).
3. Under `5654298634` the preflight verified `feature/metrics = 04b5ef6a`
   (tree `e45ae9fa`), no remote task branch, no #908 PR, and that the old
   local branch was clean, unpublished, rooted at `6b9ba1b3` and carried no
   commit or source change. As that record authorizes, the old local branch
   and worktree were removed and the branch was recreated from exactly
   `04b5ef6a`. Nothing from the old base was carried forward.

## 2. Scope confirmation

Approved specification: #908 body as amended by Specification Amendments 1-3,
approved in `5637086667`, rebound by `5653897262`.

Implemented objective: exactly the five protected managed-`DRIVER` lifecycle
mutations `claimMetricSourceUnits`, `beginMetricImport`, `ingestMetricBatch`,
`completeMetricImport` and `updateMetricSourceCheckpoint`, each requiring
exactly `METRICS_INGEST_SERVICE`; a repository-internal lifecycle coordinator
over the existing checkpoint, import, batch and coverage persistence; unchanged
delegation of canonical ingestion to the merged #900 coordinator; additive
GraphQL exposure of the three existing domain enums; and reconciliation of the
two predecessor point-in-time SDL guards.

Out-of-scope changes made: NONE.

## 3. Commits

- `fa8e11159444844554fcb3c28cf5a5f0b7fdbdfa` — `MET-WP2-02: add the protected
  managed-DRIVER ingestion lifecycle` (parent `04b5ef6a`, tree
  `c9d529ba68aba70ae9336f3ffaaaaaef9d5b60ce`). Every Rust, test, changelog and
  contract-register change. All results in section 9 were produced on exactly
  this tree.
- A following documentation-only commit adds this report and changes no
  compiled or test artifact.

## 4. Files changed

Authorized maximum write budget (15 exact paths, `5654298634`): `CHANGELOG.md`,
`docs/metrics/contract-register.md`, `thoth-api/src/model/mod.rs`,
`thoth-api/src/graphql/mod.rs`, `thoth-api/src/graphql/model.rs`,
`thoth-api/src/graphql/mutation.rs`,
`thoth-api/src/model/metric_ingestion_lifecycle/mod.rs`,
`thoth-api/src/model/metric_ingestion_lifecycle/tests.rs`,
`thoth-api/src/graphql/metric_ingestion_lifecycle_tests.rs`,
`docs/engineering/ai-delivery/implementation-reports/MET-WP2-02-implementation-report.md`,
`thoth-api/src/model/metric_import/mod.rs`,
`thoth-api/src/model/metric_record_provenance/mod.rs`,
`thoth-api/src/model/metric_ingestion/error.rs`,
`thoth-api/src/graphql/metric_registry_tests.rs`,
`thoth-api/src/graphql/metric_source_registry_tests.rs`.

| Path | New | Reason / behavioural effect | In budget |
|---|---|---|---|
| `thoth-api/src/model/metric_ingestion_lifecycle/mod.rs` | yes | lifecycle coordinator: inputs/outputs, bounded error type, claim, begin, guarded batch, completion, checkpoint progress | YES |
| `thoth-api/src/model/metric_ingestion_lifecycle/tests.rs` | yes | 26 PostgreSQL tests: bootstrap, bounds, eligibility, races, reclaim, stale tokens, idempotency, guard lock, completion, progress, crash/retry, rollback | YES |
| `thoth-api/src/graphql/metric_ingestion_lifecycle_tests.rs` | yes | 7 API-boundary tests: exact SDL, enums, role matrix, auth-before-DB, end-to-end lifecycle, error mapping | YES |
| `docs/engineering/ai-delivery/implementation-reports/MET-WP2-02-implementation-report.md` | yes | this report | YES |
| `thoth-api/src/model/mod.rs` | | registers the backend-only lifecycle module | YES |
| `thoth-api/src/graphql/mod.rs` | | registers the lifecycle GraphQL test module | YES |
| `thoth-api/src/graphql/model.rs` | | `MetricSourceUnitClaim`, `MetricSourceCheckpoint`, `MetricImport`, `MetricBatchResult`, `MetricIngestionRowResult` objects with exactly the frozen fields | YES |
| `thoth-api/src/graphql/mutation.rs` | | the five resolvers and the `authorize_metric_ingestion_lifecycle` guard helper | YES |
| `thoth-api/src/model/metric_import/mod.rs` | | additive `juniper::GraphQLEnum` on `MetricImportStatus` with value descriptions; stale "not a GraphQL enum" comment corrected | YES |
| `thoth-api/src/model/metric_record_provenance/mod.rs` | | additive `juniper::GraphQLEnum` on `MetricRecordProvenanceClassification` with value descriptions; stale comment corrected | YES |
| `thoth-api/src/model/metric_ingestion/error.rs` | | additive `juniper::GraphQLEnum` derive and enum description on `MetricIngestionErrorCode`; module comment extended | YES |
| `thoth-api/src/graphql/metric_registry_tests.rs` | | guard reconciliation: `updateMetricSourceCheckpoint` excluded from the registry mutation set by exact name and pinned as its own group | YES |
| `thoth-api/src/graphql/metric_source_registry_tests.rs` | | guard reconciliation: the lifecycle mutation excluded by exact name from the four-administration-mutation count; the two lifecycle checkpoint operations pinned exactly once and kept off `QueryRoot`; blanket `MetricSourceCheckpoint` / `claimMetricSource` deferrals narrowed to still-deferred checkpoint administration names | YES |
| `CHANGELOG.md` | | one Unreleased entry | YES |
| `docs/metrics/contract-register.md` | | new section 3.2 recording the delivered lifecycle contract | YES |

Files deleted, moved or renamed: NONE. `thoth-api/src/model/metric_coverage/mod.rs`
was not modified (its `MetricCoverageStatus` GraphQL exposure is reused from
MET-WP4-02).

### 4.1 Write-budget compliance

PASS. `git diff --name-status 04b5ef6a0fb5cfee24095a8614555d5f9a20bf7e HEAD`
lists only paths from the 15-path budget (14 in the implementation commit, the
15th being this report), with no deletion, rename or move, and
`git status --short` is clean. The paths narrowly limited by `5653897262` carry
only their permitted change: enum exposure plus descriptions and comment
corrections (paths 11-13, no variant, database, serde or strum change), and
predecessor-guard reconciliation (paths 14-15, every WP1-12, WP1-13, WP4-01
and WP4-02 positive assertion preserved).

### 4.2 Authorized actions actually used

- repository inspection: used (GitHub reads of #908 and related records; git reads)
- source edit: used, within budget
- new file creation: used, exactly the four authorized new paths
- file deletion/move/rename: NOT USED
- branch creation: used; the clean unpublished local branch was recreated from the rebound base as `5654298634` authorizes
- commit: used
- push: used, normal fast-forward, no force
- PR creation/update: one DRAFT PR
- issue/comment mutation: NOT USED
- manual CI dispatch/rerun: NOT USED
- provider/runtime read: NOT USED
- provider/runtime write: NOT USED
- migration execution: only the embedded test-harness migrations against a disposable local database
- release/tag/publication: NOT USED
- merge: NOT USED
- deployment: NOT USED
- production activation: NOT USED
- other: task-isolated local PostgreSQL 17.10 cluster, Redis instance and Cargo target directory (section 10)

Unauthorized actions performed: NONE.

### 4.3 Automatic and manual external effects

Automatic CI/provider effects: opening the DRAFT PR triggers the repository's
normal PR workflows, including the automatic
`ghcr.io/thoth-pub/thoth:staging-pr-<PR>` image publication authorized by
`5654247017`.

Manually initiated external actions: NONE.

External writes/publication: NONE beyond the authorized push and DRAFT PR.

## 5. Implementation decisions

1. **One coordinator module.** All cross-table protocol lives in
   `model/metric_ingestion_lifecycle`. It consumes #904's
   `MetricSourceAccount::decoded_configuration` and
   `check_source_compatibility`, #907's guard, #900's `ingest_metric_batch`,
   and the existing checkpoint/import/batch/coverage persistence. No foundation
   module, decoder or coordinator was changed or duplicated.
2. **Authorization placement.** Each resolver calls
   `authorize_metric_ingestion_lifecycle` (`require_metrics_ingest_service()`
   then `user_id()`) before the coordinator is called. `created_by` is the
   authenticated principal.
3. **Claim transaction.** One transaction: source by exact code, which must be
   an enabled `DRIVER` source (`SOURCE_NOT_FOUND` / `SOURCE_NOT_ELIGIBLE`);
   enabled accounts in ascending code order; per-account eligibility by plain
   reads; `INSERT ... ON CONFLICT (source_account_id, partition_key) DO NOTHING`
   for every eligible account; then
   `SELECT ... ORDER BY account.code LIMIT n FOR UPDATE OF checkpoint SKIP LOCKED`
   over unleased or expired rows, and a fresh `Uuid::new_v4()` token and
   `transaction_timestamp() + lease` for each. `limit <= 0` returns before any
   database access.
4. **Lease check.** Every later operation locks the account's `default`
   checkpoint `FOR UPDATE` first and computes liveness in SQL
   (`lease_expires_at > transaction_timestamp()`), comparing `lease_owner`
   with the token's canonical text.
5. **Begin.** The account comes from its exact code, and its lease is checked
   first. Account and source are then read `FOR SHARE`, and platform and
   publisher `FOR SHARE` in #900's order. Eligibility is revalidated:
   `METRICS_COLLECT_NOT_ENTITLED` for the capability, `SOURCE_NOT_ELIGIBLE`
   otherwise. The existing import is found by
   `(source_account_id, upstream_report_id)`. Otherwise a new `PROCESSING`
   import is inserted and `last_discovered_at` is set in the same transaction.
6. **Batch guard.** `ingest_metric_batch_under_claim` holds a guard transaction
   with the checkpoint lock while the unchanged coordinator runs on a second
   pooled connection, per Amendment 2 section 8. The import read in the guard
   takes no lock, so it never contends with the coordinator's own
   `metric_import FOR UPDATE`.
7. **Completion.** The import is locked `FOR UPDATE` under the checkpoint
   lock. Terminal imports are returned unchanged. Committed batch keys must
   equal the expected set: a key outside it is `INVALID_IMPORT_STATE`, and a
   missing key is `IMPORT_INCOMPLETE`. The status comes from the persisted
   counters.
8. **Checkpoint progress.** A single `UPDATE` uses `GREATEST` for both progress
   columns and clears the lease. Success is a SQL count over the import's
   coverage rows: `total >= 1` and every row `COMPLETE` with exactly the
   import's period.

Interpretations within the approved contract, recorded for review:

- **I1. Database failures** surface as `INTERNAL_DATABASE_ERROR`, a value of
  the authoritative #900 vocabulary, because the frozen lifecycle inventory has
  no database classification and a new code is not authorized.
- **I2. Transport format violations** map to `LIFECYCLE_LIMIT_EXCEEDED`: a
  non-canonical `value` string, a negative `sourceRowNumber`, a non-hex or
  wrongly sized digest, a period that is not exactly one day, and duplicate or
  over-long keys. This is the only frozen lifecycle code describing an input
  outside its approved bounds.
- **I3. Expected batch keys are a set.** They are stored in ascending byte
  order in the server-owned manifest, so a begin retry listing the same keys in
  another order is the same request.
- **I4. Released-lease repeat of `updateMetricSourceCheckpoint`.** Amendment 2
  requires the update to be idempotent after a timeout. It also requires that
  a stale token advance nothing. After a successful update the lease is
  cleared, so the original token can no longer be matched. A repeat while the
  lease is released therefore returns the checkpoint read-only, but only when
  this import's progress is already recorded on it. Otherwise it is
  `STALE_SOURCE_CLAIM`. While any other claim holds the unit, the old token is
  always `STALE_SOURCE_CLAIM`.
- **I5. Claim-time eligibility** uses plain reads, as an intentional
  point-in-time decision. Begin revalidates under `FOR SHARE`, and the
  coordinator under its own locks. The claim creates checkpoints for every
  eligible account, not only up to `limit`. Each insert is idempotent and
  leases nothing.
- **I6. `MetricIngestionErrorCode` value descriptions** in the SDL are the
  enum's existing Rust doc comments, which Juniper picks up by default. They
  were not rewritten, because doing so would widen the narrow change to that
  path.

Deviations from the specification requiring authorization: NONE.

## 6. Database and migration effects

Migration added: NO. `thoth-api/src/schema.rs`: unchanged (reviewed
conclusion: no table, column, index, enum value or constraint is added).

Runtime data effects, all through existing columns:

- `metric_source_checkpoint`: rows `(source_account_id, 'default')` are
  created lazily by claims. `lease_owner` and `lease_expires_at` are set on
  claim and cleared on progress update. `last_discovered_at`,
  `last_completed_at` and `last_successful_period_end` are maintained as
  specified. `cursor` and `last_error` are never written.
- `metric_import`: managed-DRIVER imports are inserted with a
  `thoth-managed-driver-import/1` manifest. `status` and `completed_at` are set
  by completion.
- All canonical, provenance, batch, coverage, counter and rollup-delta writes
  remain the unchanged #900 coordinator's.

## 7. API and compatibility effects

GraphQL/API changes: strictly additive (section 9, SDL). Added: five
`MutationRoot` fields, five object types (`MetricSourceUnitClaim`,
`MetricSourceCheckpoint`, `MetricImport`, `MetricBatchResult`,
`MetricIngestionRowResult`), seven input types (`ClaimMetricSourceUnitsInput`,
`BeginMetricImportInput`, `IngestMetricBatchInput`,
`NormalizedMetricObservationInput`, `NormalizedMetricCoverageAssertionInput`,
`CompleteMetricImportInput`, `UpdateMetricSourceCheckpointInput`) and three
enums (`MetricImportStatus`, `MetricRecordProvenanceClassification`,
`MetricIngestionErrorCode`). Scalars follow the approved ruling:
`UUID -> Uuid`, `DateTime -> Timestamp`, `Date -> Date`. No scalar was added.

Generated schema/client updates: `thoth-client/assets/schema.graphql` is
build-generated and gitignored (`thoth-client/.gitignore`). No generated
artifact is committed. The workspace build regenerated it, and the internal
client compiled and tested against it (section 9).

Backwards compatibility: no existing type, field, argument, nullability,
default or enum value changed.

Deprecations: NONE.

Cross-repository dependencies: `thoth-pub/thoth-sphinx#10` consumes this
contract only after this change is independently reviewed and merged. No
Sphinx change is made here.

## 8. Authorization and security

Authorization paths changed: five new resolvers, all gated by the existing
`PolicyContext::require_metrics_ingest_service()`. `policy.rs` is unchanged.

Roles/scopes involved: `METRICS_INGEST_SERVICE` only.

Negative authorization tests: the following were each proven `NO_ACCESS` for
all five operations, with durable state byte-identical afterwards:

- anonymous / failed introspection;
- authenticated with no role;
- `PUBLISHER_USER`, `PUBLISHER_ADMIN`, `WORK_LIFECYCLE`, `CDN_WRITE`;
- `SUPERUSER`, `DISSEMINATION_WORKER` and `METRICS_READ_SERVICE`, each alone;
- every non-ingest role together.

With an unreachable pool, every denied principal still gets `NO_ACCESS`, while
the ingest principal gets `INTERNAL_DATABASE_ERROR`. That proves the
authorization decision precedes database access.

Entitlement: the machine role never substitutes for `METRICS_COLLECT`. An
`OASIS` publisher's accounts are not claimable, and begin returns
`METRICS_COLLECT_NOT_ENTITLED`.

Secret or personal-data handling: no credential, provider value or raw source
data is accepted, stored or returned. Error messages are fixed strings. Tests
assert that no SQL, table or constraint name, token, configuration value or
injected secret-looking database text reaches a message.

Security limitations: each `ingestMetricBatch` call holds two pool connections
for the duration of one coordinator call. Concurrent batch capacity is
therefore about half the connection pool (r2d2 default 10). Under exhaustion,
the call fails with `INTERNAL_DATABASE_ERROR` after the pool timeout and
commits nothing.

## 9. Tests and checks

Environment: section 10. Every result below was produced on commit
`fa8e11159444844554fcb3c28cf5a5f0b7fdbdfa`, with a clean worktree.

### Repository gates

```text
cargo test -p thoth-api --features backend
  exit 0; lib: ok. 1883 passed; 0 failed; 1 ignored (the pre-existing MET-WP4-02
  query-plan evidence test, #[ignore] on the base); tests/graphql_permissions.rs: 13 passed;
  doctests: 8 ignored
cargo test --workspace
  exit 0; thoth-api lib 1883 passed, 0 failed; every other target ok
  (31, 13, 3, 4, 13, 144, 6, 2 passed); 0 failed anywhere
cargo check --workspace                                         exit 0
cargo clippy --all --all-targets --all-features -- -D warnings  exit 0
cargo fmt --all -- --check                                      exit 0
git diff --check 04b5ef6a0fb5cfee24095a8614555d5f9a20bf7e HEAD  exit 0
```

### Focused and predecessor suites (from the complete `thoth-api` run)

```text
model::metric_ingestion_lifecycle::tests      26 passed, 0 failed   (MET-WP2-02 PostgreSQL)
graphql::metric_ingestion_lifecycle_tests      7 passed, 0 failed   (MET-WP2-02 API boundary)
model::metric_ingestion::tests                78 passed, 0 failed   (#900 coordinator, unchanged)
graphql::metric_registry_tests                12 passed, 0 failed   (MET-WP1-12, reconciled guard)
graphql::metric_source_registry_tests         11 passed, 0 failed   (MET-WP1-13, reconciled guard)
graphql::metric_rollup_tests                   8 passed, 0 failed   (MET-WP4-01)
model::metric_rollup_delta::tests             49 passed, 0 failed   (MET-WP4-01)
graphql::metric_dashboard_tests               14 passed, 0 failed   (MET-WP4-02)
model::metric_dashboard::tests                30 passed, 0 failed   (MET-WP4-02)
model::metric_source_checkpoint::tests         9 passed, 0 failed
model::metric_import::tests                   25 passed, 0 failed
model::metric_import_batch::tests             15 passed, 0 failed   (MET-WP2-01A rollback guards)
model::metric_coverage::tests                 15 passed, 0 failed
model::metric_record_provenance::tests        29 passed, 0 failed
model::metric_source::tests                   27 passed, 0 failed
model::metric_source_account::tests           29 passed, 0 failed
graphql::mutation_guard_tests                 18 passed, 0 failed
model::metric_reconciliation_run::tests       15 passed, 0 failed   (includes the no-"reconciliation"-in-SDL guard)
```

Before the guard reconciliation, running `cargo test -p thoth-api --features
backend --lib graphql::` with the new operations produced exactly two failures:
`metric_registry_tests::the_sdl_exposes_exactly_the_nine_approved_operations`
and `metric_source_registry_tests::the_sdl_exposes_exactly_the_six_approved_operations`
(208 passed). These are the two guards the budget amendment anticipated, and
no other test failed.

### Concurrency, idempotency and lifecycle evidence (`model::metric_ingestion_lifecycle::tests`)

| Required evidence | Test |
|---|---|
| first claim bootstraps one `default` checkpoint per account; fresh token stored as canonical text; live lease not reclaimable | `the_first_claim_bootstraps_one_default_checkpoint_per_account_and_leases_it_under_a_fresh_token` |
| exact bounds: `limit <= 0` claims and writes nothing; limit clamped to 50; lease clamped to 60 / 3600, default 900 | `a_non_positive_limit_claims_nothing_and_writes_nothing`, `claim_limits_and_lease_durations_are_clamped_exactly`, `more_than_fifty_eligible_accounts_yield_at_most_fifty_claims` (55 accounts: 50 then 7) |
| source code exact; disabled / non-DRIVER source fails closed; disabled account, disabled platform, missing pin, missing `METRICS_COLLECT`, unsupported configuration each skip the account and create no checkpoint | `source_resolution_and_eligibility_fail_closed` |
| two-worker (8-thread) first-claim race: one checkpoint per account, each account held by at most one worker, stored token is the winner's | `concurrent_first_claimers_bootstrap_one_checkpoint_and_exactly_one_wins_each_unit` |
| expired-unit reclaim race (6 threads × 5 rounds): exactly one live claimant with a new token | `concurrent_reclaimers_of_one_expired_unit_produce_exactly_one_live_claimant` |
| expiry/reclaim; expired-not-reclaimed, old and foreign tokens rejected by begin, batch, completion and progress, with state unchanged | `an_expired_lease_is_reclaimed_under_a_new_token_and_every_old_or_foreign_token_is_stale` |
| begin derives authority (account, pinned publisher, actor, null raw key, manifest) and records discovery | `begin_creates_a_processing_import_from_canonical_authority_and_records_discovery` |
| identical begin retry → same import, no state change, discovery not moved; key order irrelevant | `an_identical_begin_retry_returns_the_same_import_without_moving_discovery` |
| changed period, format, version, normalizer, raw hash, digest or key set → `IMPORT_IDEMPOTENCY_MISMATCH`, no change | `a_changed_begin_request_for_the_same_upstream_report_fails_closed` |
| 15 begin bound violations → `LIFECYCLE_LIMIT_EXCEEDED` with no write, before database access; exact boundaries accepted | `begin_input_bounds_are_enforced_before_any_database_access` |
| entitlement and eligibility revalidated under the claim; account code exact; another account's claim refused | `begin_revalidates_account_authority_and_entitlement_under_the_claim` |
| terminal import returned by begin, never reopened | `a_terminal_import_is_returned_by_begin_and_never_reopened` |
| batch delegated: WINNER; timeout-after-commit replay `replayed: true`, same batch id/hash/rows, zero durable change; changed payload keeps `IDEMPOTENCY_KEY_REUSED`; row rejection keeps `UNKNOWN_DOI` | `a_batch_is_delegated_to_the_coordinator_and_its_replay_repeats_no_write` |
| unexpected key, non-canonical values, negative row number, unknown import, empty batch, 501 observations, wrong schema version — each with its code and no write; unmanaged import refused | `batch_transport_bounds_and_expected_keys_fail_before_the_coordinator` |
| guard holds the checkpoint row lock for the whole coordinator call: `FOR UPDATE NOWAIT` fails, a lease-expiry write times out, a concurrent claim skips without waiting; the paused batch then commits | `the_batch_guard_holds_the_checkpoint_lock_for_the_whole_coordinator_call` |
| completion needs every expected batch (`IMPORT_INCOMPLETE`, stays `PROCESSING`); `COMPLETED` vs `COMPLETED_WITH_ERRORS` from counters; repeat unchanged | `completion_requires_every_expected_batch_and_derives_the_terminal_status` |
| `FAILED` import and escaped batch key → `INVALID_IMPORT_STATE`; unknown import | `completion_refuses_unmanaged_failed_and_escaped_state` |
| six concurrent completions close the import exactly once | `concurrent_completions_of_one_import_close_it_exactly_once` |
| non-vacuous success: zero rows, PARTIAL, UNKNOWN, COMPLETE+PARTIAL, longer period, exact+mismatched → no advance; one or two exact COMPLETE rows → advance; lease released in every case | `only_a_completed_import_with_exact_complete_coverage_advances_successful_progress` |
| `COMPLETED_WITH_ERRORS` releases and records completion but never success | `completed_with_errors_releases_the_lease_but_never_claims_success` |
| no progress or release before a terminal import | `checkpoint_progress_waits_for_a_terminal_import_and_keeps_the_lease_until_then` |
| monotonic `last_successful_period_end` and `last_completed_at`; released repeat read-only; old token stale once reclaimed; unrecorded released repeat stale | `checkpoint_progress_is_monotonic_and_a_released_repeat_is_read_only` |
| crash after claim, timeout after begin, timeout after batch, crash before completion, crash after completion before checkpoint, timeout after checkpoint — one import, one record, one rollup delta, no duplicate batch | `every_lifecycle_crash_boundary_recovers_without_a_duplicate_durable_effect` |
| failed write inside begin rolls back the import insert; sanitized code and fixed message | `a_failed_lifecycle_transaction_leaves_no_partial_state` |

**Negative control.** With only the `evidence.total >= 1` condition removed
from the success rule, the non-vacuous coverage test fails on its
`zero coverage rows` case (`left: Some(2026-03-11)`, `right: None`). The
module was then restored byte-identically (`cmp`).

### API-boundary evidence (`graphql::metric_ingestion_lifecycle_tests`)

- `the_sdl_exposes_exactly_the_five_approved_operations_and_no_sixth`: each
  exact signature is declared once. No sixth lifecycle-like `MutationRoot`
  field exists, and nothing reaches `QueryRoot`, `Work`, `Publisher`,
  `Imprint` or `Publication`.
- `the_lifecycle_objects_and_inputs_match_the_frozen_contract_exactly`: exact
  field lists of the five objects and seven inputs, including defaults
  `limit: Int = 10` and `leaseSeconds: Int = 900`. No `leaseOwner`, raw key,
  upstream id, manifest, creator, counter, cursor or error field is exposed,
  and no `UUID` or `DateTime` scalar exists.
- `the_three_domain_vocabularies_are_exposed_with_exactly_their_authoritative_values`:
  exact values of `MetricImportStatus` (6) and
  `MetricRecordProvenanceClassification` (5). All 47 `MetricIngestionErrorCode`
  Rust variants appear, and no other values; each value round-trips through
  strum and serde. No parallel transport enum exists.
- `every_resolver_authorizes_before_calling_the_lifecycle_coordinator`: in each
  of the five resolvers, the guard call precedes the coordinator call. The
  helper uses only `require_metrics_ingest_service()`.
- `every_operation_denies_every_principal_but_the_ingest_service_without_touching_state`
  and `authorization_is_decided_before_any_operation_specific_database_access`:
  section 8.
- `a_complete_unit_runs_through_the_five_operations_with_bounded_errors`: the
  full unit runs through GraphQL, from claim through begin, a retry, two
  batches (WINNER, then DUPLICATE) and a replay, to completion and progress.
  The lease is released at the end. Bounded, sanitized
  `extensions.type` values are checked for `STALE_SOURCE_CLAIM`,
  `IMPORT_IDEMPOTENCY_MISMATCH`, `LIFECYCLE_LIMIT_EXCEEDED`,
  `IDEMPOTENCY_KEY_REUSED`, `UNEXPECTED_BATCH_KEY`, `IMPORT_INCOMPLETE`,
  `INVALID_IMPORT_STATE`, `IMPORT_NOT_FOUND` and `SOURCE_NOT_FOUND`.

### SDL: exact base vs head

Both files were generated by `cargo build --workspace` (`thoth-client/build.rs`)
from the exact base worktree and from `fa8e1115`:

```text
base 04b5ef6a  sha256 06cd640b32b125a5009b1abd1dc4f5bce9da02cbb1d5fa30120d96352a9fdf7a  205,696 bytes  5,081 lines
head fa8e1115  sha256 bd33dc1b370ff9e5aa35de28b0d1fc0f30e6a90e75f0e767260d3f3001a406af  222,582 bytes  5,384 lines
diff: 14 hunks, 0 base lines deleted or changed, 303 lines added
declarations added (15): BeginMetricImportInput, ClaimMetricSourceUnitsInput, CompleteMetricImportInput,
  IngestMetricBatchInput, MetricBatchResult, MetricImport, MetricImportStatus, MetricIngestionErrorCode,
  MetricIngestionRowResult, MetricRecordProvenanceClassification, MetricSourceCheckpoint,
  MetricSourceUnitClaim, NormalizedMetricCoverageAssertionInput, NormalizedMetricObservationInput,
  UpdateMetricSourceCheckpointInput
declarations removed: none
MutationRoot: +5 fields (the five operations); QueryRoot: unchanged
```

The generated artifact is not committed. The unchanged internal
`thoth-client` compiled and passed its tests against the regenerated schema in
`cargo test --workspace`.

## 10. Manual verification

Environment:

- a dedicated Homebrew PostgreSQL 17.10 cluster, UTF8 with `C` collation and
  ctype, started for this task on its own port and socket directory. Database
  `thoth_test_wp202` was created from `template0` (`char_length('é') = 1`).
- a dedicated Redis instance on its own port;
- a task-isolated `CARGO_TARGET_DIR`, created as a copy-on-write clone of a
  dependency cache with every `thoth*` workspace artifact removed before the
  first build;
- an isolated git worktree outside the repository, with no `.env`.
  `THOTH_EXPORT_API`, `TEST_DATABASE_URL` and `TEST_REDIS_URL` were exported
  as CI sets them.

No shared, staging or production database or provider was accessed. Steps and
results: section 9.

## 11. CI

Natural PR-triggered CI only, observed read-only, with no manual dispatch,
rerun or cancel. Run identifiers and conclusions for the exact PR head are in
the PR history.

## 12. Rollout and rollback

Initial state after merge: inactive. No checkpoint, import or lease exists
until an authenticated `METRICS_INGEST_SERVICE` caller claims a unit of an
enabled, configured `DRIVER` source whose pinned publisher holds
`METRICS_COLLECT`.

Activation required: a separately governed provisioning of a machine
principal with `METRICS_INGEST_SERVICE`, real source/account configuration,
and the Sphinx consumer (`thoth-sphinx#10`). None of that is part of this task.

Feature flag/configuration: none.

Migration sequence: none.

Rollback/disable procedure: withhold the role, or disable the source or
account (claims then return nothing and begin fails closed). Source rollback
is a separately authorized revert. Durable rows written through the lifecycle
are ordinary checkpoint and import rows.

Monitoring required: none new. Database failures are logged server-side and
surface as `INTERNAL_DATABASE_ERROR`.

## 13. Known limitations and deferred work

- `PUBLISHER_UPLOAD`, `OPERAS` and `ADMIN_IMPORT` lifecycles, operational
  `FAILED` transitions, checkpoint administration, `cursor` / `last_error`
  semantics, multi-partition sources and Metrics read surfaces remain
  deferred.
- Batch concurrency is bounded by the two-connection guard (section 8).
- `MetricIngestionErrorCode` SDL value descriptions are the enum's existing
  Rust doc comments (I6).

## 14. Unresolved issues

- NONE.

## 15. Agent self-assessment

The agent may identify risks but may not approve the task.

Suggested review focus:

- the two-connection guard and its lock order relative to the coordinator (decision 6);
- interpretation I4 (read-only released repeat of `updateMetricSourceCheckpoint`);
- interpretations I1-I3 (error mapping and expected-key set semantics);
- claim-time eligibility by plain reads, and bootstrap of every eligible account (I5);
- the reconciled predecessor guards in `metric_registry_tests.rs` and `metric_source_registry_tests.rs`.

## 16. MET-WP2-02-CR1: claim-time canonical authority correction (post-merge)

Sections 1-15 above are the historical implementation evidence of PR #920 as
committed on `fa8e1115` / `8d0f5dea`. They are preserved unchanged. This
section records the forward correction of the CRITICAL defect found by the
independent exact-head review after that evidence was written.

### 16.1 Incident and control references

| Record | Reference |
|---|---|
| Independent exact-head review, CHANGES REQUIRED, defines CR-1 | #908 comment `5654942925` (2026-09-13) |
| PR #920 merge | `feature/metrics @ 3db0699cdc9d5a212b0f48ec9fa674e499776cc2`, tree `14210f97498946740b2927419f993432977b4560` (merged 2026-09-13 17:55Z, after `5654942925` superseded approval `5654939879`) |
| Post-merge control reconciliation / CR-1 forward-correction authorization (**current implementation authority**) | #908 comment `5665416112` (2026-09-14) |
| Programme reconciliation / downstream HOLD | #766 comment `5665419874` |
| Sphinx producer pin invalidated pending this correction | thoth-sphinx#10 comment `5665424624` |
| Superseded as current source approval | `5654939879` (approval), `5655029012` (post-merge reconciliation) |

The functional contract is unchanged: the approved final #908 specification
with Amendments 1-3. The five-operation GraphQL design is not reopened.

### 16.2 Correction binding

| Item | Value |
|---|---|
| Stage | `MET-WP2-02-CR1` (owned by #908, CRITICAL) |
| Authorized base | `feature/metrics @ 3db0699cdc9d5a212b0f48ec9fa674e499776cc2` |
| Authorized base tree | `14210f97498946740b2927419f993432977b4560` |
| Defective source tree | `14210f97498946740b2927419f993432977b4560` (the merged #920 tree) |
| Correction branch | `feature/metrics--wp2-02-cr1-claim-authority`, created from exactly `3db0699c` after preflight |
| PR target | `feature/metrics` |
| Source correction commit | `0fba1102c7f8c216675f904ab76fb0df779a1eea` (parent `3db0699c`, tree `b8b596a17795c9aec8969f7baed28882bc21abeb`) |
| Report commit | the following documentation-only commit; the PR head is visible on the DRAFT PR |
| Database migration | NONE |
| Public GraphQL contract change | NONE (section 16.13) |
| Implementing session | CCD session `local_009f4622-9a55-4ea5-9626-2aa2ef31648c`, designated in the CTO's correction prompt; it is not the original #920 session `local_dd82a4d5` (section 16.15) |

Preflight (all read-only, before any mutation): `origin/feature/metrics` was
`3db0699c` with tree `14210f97`; no remote branch
`feature/metrics--wp2-02-cr1-claim-authority` existed; no PR existed for that
head; the workspace was clean; no other running session owned #908 work; the
historical #920 branch was not reused or rewritten (it remains at `8d0f5dea` in
its old worktree, untouched).

### 16.3 Exact changed paths (write budget: three existing paths, maximum)

| Path | Changed | Effect |
|---|---|---|
| `thoth-api/src/model/metric_ingestion_lifecycle/mod.rs` | yes | claim-time lock/revalidation (`revalidate_claim_authority`, `SelectedCheckpoint`), documentation of the claim lock order |
| `thoth-api/src/model/metric_ingestion_lifecycle/tests.rs` | yes | four deterministic PostgreSQL authority-race tests and their helpers |
| `docs/engineering/ai-delivery/implementation-reports/MET-WP2-02-implementation-report.md` | yes | this section |

`git diff --name-status 3db0699c...HEAD` lists exactly those three paths, all
`M`. No file was added, deleted, renamed or moved. `#904`, `#900`, `policy.rs`,
ZITADEL, `thoth-errors`, Cargo, migrations, `schema.rs`, GraphQL resolvers and
Sphinx are untouched. Nothing was copied from the stood-down scratch
implementation.

### 16.4 Root cause

At `14210f97`, `claim_metric_source_units` decided eligibility with plain reads
(`check_eligibility(..., false)`), then ensured and locked checkpoints
`FOR UPDATE SKIP LOCKED`, wrote the lease and returned `source` /
`source_account` objects loaded *before* the checkpoint lock. No lock was held
on any authority row, so a canonical writer could commit between the
eligibility decision and the lease grant:

```text
claim                                  admin
-----                                  -----
read source/account/platform/publisher
(all eligible, no lock)
                                       lock authority row FOR UPDATE
                                       disable / reconfigure / revoke
                                       commit
insert/lock checkpoint
write lease
return claim built from the stale objects
```

The mutable eligibility set is `metric_source.enabled` (and the immutable
acquisition type), `metric_source_account.enabled` and `configuration`,
`metric_platform.enabled`, the account's `expected_publisher_id` pin and
`publisher.subscription_package` (the `METRICS_COLLECT` entitlement).
`beginMetricImport`'s later revalidation does not close the boundary: the
claim payload is Sphinx's route for the configuration used for external
collection before `beginMetricImport`.

One incidental partial protection existed and explains why a naive
first-claim probe can look safe: a *bootstrap* insert of a checkpoint takes an
FK `KEY SHARE` on the account row, which blocks the #904 account writer's
`FOR UPDATE` until the claim commits. It covers only the account row, only on
the first claim of that account, and nothing on the reclaim path. It is not a
correction and was not relied on.

### 16.5 Live lock topology and the final total order

Production lock sites at `3db0699c` (only what is relevant to claim
eligibility):

| Operation | Locks, in acquisition order |
|---|---|
| claim (corrected) | checkpoints `FOR UPDATE SKIP LOCKED` (never waits) -> per locked checkpoint, in account-code order: account `FOR SHARE` -> source `FOR SHARE` -> platform `FOR SHARE` -> publisher `FOR SHARE` -> `UPDATE` of the already-locked checkpoint |
| beginMetricImport | checkpoint `FOR UPDATE` -> account `S` -> source `S` -> platform `S` -> publisher `S` -> insert `metric_import` -> update checkpoint |
| ingestMetricBatch guard (connection 1) | checkpoint `FOR UPDATE`, held across the coordinator call; `metric_import` read without lock |
| #900 coordinator (connection 2) | `metric_import` `FOR UPDATE` -> account `S` -> source `S` -> platform `S` -> publisher `S` -> measures / mappings `S` -> imprint, publication, institution `S` -> work `FOR UPDATE` -> advisory cell locks -> `metric_record` `FOR UPDATE` |
| completeMetricImport | checkpoint `FOR UPDATE` -> `metric_import` `FOR UPDATE` |
| updateMetricSourceCheckpoint | checkpoint `FOR UPDATE` -> `metric_import` `FOR SHARE` |
| #904 `update_metric_source` | `metric_source` `FOR UPDATE` (by code) -> `UPDATE` same row -> insert history. One locked row. |
| #904 `update_metric_source_account` | `metric_source_account` `FOR UPDATE` (by code) -> plain read of source -> `UPDATE` same row -> insert history. One locked row. `code`, `source_id`, `platform_id`, `external_key`, `expected_publisher_id` are not writable here. |
| MET-WP1-12 `update_metric_platform` | `metric_platform` `FOR UPDATE` (by code) -> `UPDATE` same row -> insert history. One locked row. |
| BE-01 `replace_publisher_service_configuration` (the only production writer of `publisher.subscription_package`; `PatchPublisher` has no package field) | `publisher` `FOR UPDATE` (`lock_publisher`, first statement) -> `publisher_distribution_platform` rows -> `distribution_job` rows -> `UPDATE publisher` (its `AFTER UPDATE` trigger locks the publisher's `work` rows) -> insert history |
| generic `updatePublisher` | plain `UPDATE publisher` (`FOR NO KEY UPDATE` tuple lock, `FOR UPDATE` if `publisher_name` changes) -> insert `publisher_history`. Cannot change the package. |

Final total order used by every lifecycle operation (each uses a prefix or a
subset):

```text
metric_source_checkpoint (FOR UPDATE / FOR UPDATE SKIP LOCKED)
  < metric_source_account (FOR SHARE)
  < metric_source (FOR SHARE)
  < metric_platform (FOR SHARE)
  < publisher (FOR SHARE)
  < [coordinator tail: metric_import*, measures, mappings, imprint, publication,
     institution, work, cell advisory locks, metric_record]
```

`*` The coordinator takes `metric_import FOR UPDATE` before the authority
shares, and completion/update take it after the checkpoint; the claim never
touches `metric_import`, so it adds no edge there.

### 16.6 Lock modes and writer conflicts

The correction takes `FOR SHARE` on the account, source, platform and
publisher rows of every locked checkpoint, after the checkpoint lock and
before the lease write, and builds the returned claim from those locked rows.

| Lock | Serializes against | Why that writer conflicts | Why weaker would not suffice |
|---|---|---|---|
| account `FOR SHARE` | #904 `update_metric_source_account` (`FOR UPDATE` on the account row, then `UPDATE`) | `FOR UPDATE` conflicts with `FOR SHARE`; the row's `enabled`/`configuration` cannot change until the claim commits | `FOR KEY SHARE` conflicts with `FOR UPDATE` but not with a plain `UPDATE` (`FOR NO KEY UPDATE`); `FOR SHARE` conflicts with every `UPDATE` of the row regardless of how the writer locked it |
| source `FOR SHARE` | #904 `update_metric_source` (`FOR UPDATE`, then `UPDATE`) | same | same |
| platform `FOR SHARE` | MET-WP1-12 `update_metric_platform` (`FOR UPDATE`, then `UPDATE`) | same | same |
| publisher `FOR SHARE` | BE-01 `replace_publisher_service_configuration` (`FOR UPDATE` first) and generic `updatePublisher` (`FOR NO KEY UPDATE`/`FOR UPDATE`) | both conflict with `FOR SHARE` | `FOR KEY SHARE` would not conflict with `updatePublisher`'s `FOR NO KEY UPDATE` |
| not stronger than `FOR SHARE` | | | `FOR NO KEY UPDATE`/`FOR UPDATE` would make concurrent claims of one source mutually exclusive on the shared source/platform/publisher rows, would block begin's and the coordinator's `FOR SHARE` on the same rows, and would block the FK `KEY SHARE` of `create_metric_source_account`, for no additional protection |

`FOR SHARE` is also the mode begin and the #900 coordinator already take on
exactly these rows, so the corrected claim is share-compatible with both.

The checkpoint lock keeps its `FOR UPDATE SKIP LOCKED` semantics; the lease
write, bounds, token, expiry/reclaim, stale/foreign rejection, released-token
retry, bootstrap and the #900 two-connection guard are unchanged.

### 16.7 Deadlock analysis

A cycle needs two transactions that each hold something the other waits for.

| Pair | Analysis |
|---|---|
| claim <-> claim | Checkpoints are taken `SKIP LOCKED`: a claim never waits on a checkpoint. Authority locks are `FOR SHARE` on both sides: share never waits for share. Two claims can wait only on a concurrent uncommitted bootstrap insert of the same `(account, default)` row (unique index), and that inserter waits only on single-row writers (below). No cycle. |
| claim <-> source admin (#904) | The writer holds exactly one row (`metric_source`) and waits on nothing while holding it (its `UPDATE` is on the row it already locked; its history insert locks nothing shared). A single-lock transaction cannot close a cycle. Either it commits first and the claim's locked reload sees the change, or it queues behind the claim's share until commit. |
| claim <-> source-account admin (#904) | Same: one row (`metric_source_account`). The claim may also wait on this writer at a bootstrap insert (FK `KEY SHARE`); the writer still waits on nothing. |
| claim <-> platform admin | Same: one row (`metric_platform`). |
| claim <-> publisher/package admin (BE-01) | Holds `publisher` first, then assignment, job and `work` rows. The claim takes none of those later rows, so the writer can only wait on the claim's publisher share, which the claim releases at commit without waiting on anything the writer holds. The #900 coordinator takes publisher `S` before `work U`, the same direction as BE-01 (`publisher` -> `work`), so that pre-existing pair is unchanged. |
| claim <-> beginMetricImport | begin: checkpoint `U` then shares; claim: checkpoint (skip) then shares. Same direction; begin may wait on a checkpoint the claim holds, and the claim then waits only on single-row writers. The shares are compatible. |
| claim <-> ingestMetricBatch guard | Guard holds checkpoint `U` only; the claim skips it. Coordinator holds `metric_import U` and shares; the claim takes no import lock and only shares. |
| claim <-> completeMetricImport | checkpoint `U` then `metric_import U`: the claim skips the checkpoint and never touches the import. |
| claim <-> updateMetricSourceCheckpoint | checkpoint `U` then `metric_import S`: same. |

No operation acquires the relevant rows in the reverse order of the total
order in 16.5: every lifecycle operation takes the checkpoint before any
authority row, and the authority rows in the same account -> source ->
platform -> publisher sequence; every admin writer takes exactly one authority
row (BE-01 continues only into rows the lifecycle takes after publisher or not
at all). Operations that use a subset (guard, completion, update: checkpoint
and import only; admin writers: one row) are noted as such. A liveness note,
pre-existing and unchanged: PostgreSQL does not queue a compatible `FOR SHARE`
behind a waiting `FOR UPDATE`, so a continuous stream of claims/begins/batches
on one account could delay (not deadlock) an administrative write.

### 16.8 Red concurrency evidence (pre-correction tree `14210f97` + the new tests)

Synchronization mechanism (no sleeps decide anything): a test-only trigger on
`metric_source_checkpoint` executes `pg_advisory_xact_lock(987654321202)`
while the test holds the same key session-level, freezing the claim at an
exact statement. `BEFORE INSERT` freezes it after the unlocked eligibility
enumeration and before its checkpoint lock (the bootstrap insert fires the
trigger even when `ON CONFLICT DO NOTHING` then discards the row; verified on
PostgreSQL 17.10). `BEFORE UPDATE` freezes it at the first lease write. The
claim runs on a one-connection pool whose backend pid is known, and
`pg_stat_activity` (`wait_event_type = 'Lock'`, `wait_event = 'advisory'`) is
polled to know it is frozen. A writer's queueing is proven the same way on its
own pinned backend (`wait_event_type = 'Lock'`), or its completion by
`JoinHandle::is_finished`. Deadlines exist only as hang guards. Every writer is
the repository's real coordinator: `update_metric_source_account`,
`update_metric_source`, `update_metric_platform`,
`replace_publisher_service_configuration` (OBELISK -> OASIS, the only
production path that changes `subscription_package`).

Direction 1 ("writer commits inside the claim window"), all four tests, base
code, `cargo test ... cr1_ -- --test-threads=1`: `0 passed; 4 failed`.

| Test | Pre-fix result | Expected invariant |
|---|---|---|
| `cr1_source_account_authority_cannot_go_stale_between_eligibility_and_lease_grant` | `left: ["acct-a", "acct-b"]`, `right: ["acct-b"]`: A was leased although its disable had committed before the lease | no fresh claim carries an account whose disable committed before the grant; a configuration replacement committed before the grant is what the claim carries |
| `cr1_source_authority_cannot_go_stale_between_eligibility_and_lease_grant` | `Ok([A, B])` with `source.enabled: true` although the source was disabled | `Err(SourceNotEligible)`, nothing leased |
| `cr1_platform_authority_cannot_go_stale_between_eligibility_and_lease_grant` | `[A, B]` leased although the platform was disabled | empty claim, no lease |
| `cr1_publisher_capability_cannot_go_stale_between_eligibility_and_lease_grant` | `[A, B]` leased although the publisher had become OASIS | empty claim, no lease |

Direction 2 ("claim at the lease write, writer arrives"), a temporary
uncommitted probe on the base code over existing (reclaim-path) checkpoints:
every writer reported `Finished` inside the claim window (account, source,
platform, publisher), and the claim then returned `[acct-a, acct-b]` from the
pre-mutation state. On the first-claim (bootstrap) path the account writer
alone reported `Blocked`, which is the incidental FK `KEY SHARE` of 16.4, not a
protection of source, platform or publisher. The probe was removed before the
correction; the committed tests assert `Blocked` for all four.

### 16.9 Green concurrency evidence (corrected tree)

`cargo test -p thoth-api --features backend --lib model::metric_ingestion_lifecycle::tests::cr1_ -- --test-threads=1`:
`4 passed; 0 failed`.

| Authority | Direction 1: writer commits first | Direction 2: claim locks first |
|---|---|---|
| source account | A's disable committed inside the window: claim returns only B; A's bootstrapped checkpoint stays unleased. A configuration replacement inside the window: the claim carries the replaced canonical configuration, equal to the committed row. | `WriterOutcome::Blocked`; claim returns A with `enabled: true`; the disable commits after the claim; `lease_owner` is the claim's token; `beginMetricImport` then fails closed `SourceNotEligible` |
| source | `Err(SourceNotEligible)`, zero leases, the whole transaction rolled back | `Blocked`; claim returns A and B with `source.enabled: true`; source disabled afterwards; begin fails closed |
| platform | empty claim, zero leases | `Blocked`; claim returns A and B; platform disabled afterwards; begin fails closed |
| publisher `METRICS_COLLECT` | empty claim, zero leases | `Blocked`; claim returns A and B; package OASIS afterwards; begin fails closed `MetricsCollectNotEntitled` |

The forbidden interleaving (authority mutation commits -> lease granted from
pre-mutation state -> stale claim returned) no longer occurs in any of the
four.

### 16.10 Non-vacuity

Temporary uncommitted mutation of `mod.rs`: the
`revalidate_claim_authority(...)` call in the claim loop was replaced by the
pre-fix behaviour (clone the enumeration-time `source`, look the account up in
the enumeration-time `eligible` vector, no lock). Result of the four CR-1
tests: `0 passed; 4 failed`, each on its direction-1 assertion (A leased
although disabled; `Ok([A, B])` instead of `SourceNotEligible`; `[A, B]`
instead of empty for platform and publisher). The authorized `mod.rs` was then
restored from a copy taken before the mutation and verified with
`shasum -a 256 -c` and `cmp` (byte-identical); `git diff --name-only` showed
only the two authorized source paths.

### 16.11 Existing lifecycle regression results

From the complete `cargo test -p thoth-api --features backend` run on the
corrected tree (section 16.12):

```text
model::metric_ingestion_lifecycle::tests      30 passed, 0 failed   (26 historical + 4 CR-1 races)
graphql::metric_ingestion_lifecycle_tests      7 passed, 0 failed
model::metric_ingestion::tests                78 passed, 0 failed   (#900 coordinator, unchanged)
graphql::metric_registry_tests                12 passed, 0 failed
graphql::metric_source_registry_tests         11 passed, 0 failed
model::metric_source_checkpoint::tests         9 passed, 0 failed
model::metric_import::tests                   25 passed, 0 failed
model::metric_import_batch::tests             15 passed, 0 failed
model::metric_coverage::tests                 15 passed, 0 failed
model::metric_source::tests                   27 passed, 0 failed   (#904, unchanged)
model::metric_source_account::tests           29 passed, 0 failed   (#904, unchanged)
model::metric_platform::tests                 21 passed, 0 failed
model::publisher_service_configuration::tests 45 passed, 0 failed   (BE-01, unchanged)
focused: cargo test -p thoth-api --features backend --lib model::metric_ingestion_lifecycle::tests::cr1_ -- --test-threads=1
         4 passed, 0 failed
```

Every pre-existing lifecycle test is unchanged and green: two-worker (8-thread)
first claim and bootstrap, `SKIP LOCKED` during the guard, six-thread reclaim
of one expired unit, expiry/reclaim with old and foreign tokens, checkpoint
monotonicity, released-token read-only retry, begin idempotency and mismatch,
batch replay/idempotency, concurrent completions, and the coverage rules for
successful-period progress.

### 16.12 Full validation (corrected tree, fresh)

Environment: isolated git worktree outside the repository with no `.env`;
`THOTH_EXPORT_API`, `TEST_DATABASE_URL` and `TEST_REDIS_URL` exported as CI
sets them; a dedicated Homebrew PostgreSQL 17.10 cluster on its own port and
data directory (`thoth_test` created UTF8, `C` collation); the machine's local
Redis on 6379; a task-isolated `CARGO_TARGET_DIR` built from scratch for this
correction.

```text
cargo fmt --all -- --check                                      exit 0
cargo test -p thoth-api --features backend                      exit 0; lib: 1887 passed, 0 failed, 1 ignored
                                                                (pre-existing MET-WP4-02 query-plan test);
                                                                tests/graphql_permissions.rs: 13 passed; doctests: 8 ignored
cargo test --workspace                                          exit 0; thoth-api lib 1887 passed / 0 failed;
                                                                thoth bin 31, graphql_permissions 13, thoth-api-server 3,
                                                                thoth-client 4 (+6 doctests), thoth-errors 13,
                                                                thoth-export-server 144 (+2 doctests); 0 failed anywhere
cargo check --workspace                                         exit 0
cargo clippy --all --all-targets --all-features -- -D warnings  exit 0
cargo build                                                     exit 0
git diff --check 3db0699cdc9d5a212b0f48ec9fa674e499776cc2 HEAD  exit 0
(the only compiler output is cargo's pre-existing future-incompatibility note for proc-macro-error2 v2.0.1)
```

### 16.13 SDL: base vs head

Both generated by `thoth-client/build.rs` during `cargo build --workspace`,
the head after `cargo clean -p thoth-api -p thoth-client` in the same target
directory so the build script re-ran against the corrected `thoth-api`:

```text
base 3db0699c  sha256 bd33dc1b370ff9e5aa35de28b0d1fc0f30e6a90e75f0e767260d3f3001a406af  222,582 bytes  5,384 lines
head 0fba1102  sha256 bd33dc1b370ff9e5aa35de28b0d1fc0f30e6a90e75f0e767260d3f3001a406af  222,582 bytes  5,384 lines
semantic diff: NONE (the two files are byte-identical: cmp exit 0)
```

### 16.14 Effects

- Migration / schema / index: NONE. `schema.rs` unchanged.
- Auth / security: no authorization path, role, policy or resolver changed.
  The correction narrows what a fresh claim can return: only authority that is
  valid at lease grant and cannot change before the claim commits. Error
  vocabulary and messages are unchanged.
- Data: no new write. The claim still writes only `lease_owner` /
  `lease_expires_at` on selected checkpoints. A checkpoint lazily bootstrapped
  for an account that becomes ineligible before revalidation is left unleased
  (a state the limit-bounded claim already produced). A source that becomes
  ineligible before revalidation fails the claim closed and rolls back the
  bootstrap inserts. Lock footprint: four additional `FOR SHARE` row locks per
  leased unit, held until the claim commits.
- External effects: none beyond the authorized push, the DRAFT PR and its
  normal PR-triggered CI (including the already-approved automatic
  `ghcr.io/thoth-pub/thoth:staging-pr-<PR>` image). No manual CI action, no
  issue comment, no provider, credential, deployment, release or activation.

### 16.15 Ownership note

Record `5665416112` names the original PR #920 implementation session as sole
owner and requires a HOLD for explicit ownership transfer if it is
unavailable. This correction was implemented by CCD session
`local_009f4622-9a55-4ea5-9626-2aa2ef31648c` under the CTO's direct correction
prompt naming it the sole designated implementation owner (2026-09-14); the
original session `local_dd82a4d5` was not running. No source, commit or
unpublished state of the stood-down scratch session was used. The ledger
should record this designation explicitly; it is reported here rather than
assumed.

### 16.16 Remaining gates

Fresh independent CRITICAL exact-head source review of the new head; separate
exact-head CTO merge authorization; merge; fresh downstream Sphinx
producer-contract rebind (thoth-sphinx#10); later integration, deployment and
activation gates. Nothing here is self-approved.

## 17. MET-WP2-02-CR1 follow-up: control reconciliation and changelog

Sections 1-16 above are preserved unchanged. Sections 1-15 are the PR #920
evidence. Section 16 is the CR-1 correction evidence written at PR #921 head
`4e5337d2`, including its ownership note (16.15) as written at the time. This
section records the bounded follow-up that reconciles control on PR #921 and
adds the mandatory changelog entry. It does not reopen, redesign or re-validate
the CR-1 source correction.

### 17.1 Control references

| Record | Reference |
|---|---|
| CR-1 forward-correction authorization; sole owner = the original #920 session, HOLD if unavailable | #908 comment `5665416112` (2026-09-14) |
| Control reconciliation, adoption of PR #921, footprint amendment (three to four paths), changelog requirement | #908 comment `5667624555` (2026-09-14 16:58:45Z) |
| Remaining-work ownership transfer; `5667624555` stays authoritative except for its sole-owner designation | #908 comment `5678927229` (2026-09-15 10:47:05Z) |

### 17.2 Reconciliation outcome

- **Historical deviation, not retroactively authorized.** PR #921 was produced
  by CCD session `local_009f4622-9a55-4ea5-9626-2aa2ef31648c`, not by the owner
  named in `5665416112`. That session's branch creation, source writes, commits,
  push and DRAFT PR creation are **not** retroactively authorized by
  `5667624555` or `5678927229`. They remain a recorded control deviation.
- **Adoption.** `5667624555` adopted the existing PR #921 artifact at exact
  head `4e5337d2ac42b07792544a6031bbba5575f51e3a` (tree
  `c57e1463b3caab184d83a1d093e412396d3b1128`) prospectively as the candidate
  correction artifact. PR #921 remains that artifact. Adoption is neither source
  approval nor merge approval.
- **Review state at the pre-follow-up head.** Per `5667624555`, the independent
  CRITICAL exact-head review of `4e5337d2` found no blocking source defect in
  the CR-1 correction itself. Its decision was BLOCKED on two control
  requirements: the implementing session did not match the owner named in
  `5665416112`, and the three-path footprint omitted the mandatory
  `CHANGELOG.md`, which left exact-head CI red. The follow-up commit creates a
  new PR head and so invalidates that exact-head review.
- **Ownership.** `5667624555` named `local_009f4622` as sole owner of the
  remaining work. That session was archived and did not perform this follow-up.
  On 2026-09-14 a follow-up prompt reached a different session,
  `local_0ba55677-ce1d-43d3-98cd-23e488bee658`. It HELD with no mutation because
  the designation did not name it. `5678927229` then transferred sole ownership
  of the remaining bounded work prospectively to
  `local_0ba55677-ce1d-43d3-98cd-23e488bee658`, which performed this follow-up.
  The prior owner is no longer authorized for this stage. The transfer does not
  retroactively authorize any historical action.

### 17.3 Follow-up binding and preflight

| Item | Value |
|---|---|
| Base | `feature/metrics @ 3db0699cdc9d5a212b0f48ec9fa674e499776cc2`, tree `14210f97498946740b2927419f993432977b4560` |
| Pre-follow-up PR head | `4e5337d2ac42b07792544a6031bbba5575f51e3a`, tree `c57e1463b3caab184d83a1d093e412396d3b1128` |
| Existing PR commits (not amended or rewritten) | `0fba1102c7f8c216675f904ab76fb0df779a1eea` (source, tree `b8b596a17795c9aec8969f7baed28882bc21abeb`); `4e5337d2` (report) |
| Branch | `feature/metrics--wp2-02-cr1-claim-authority` (existing; non-force push) |
| Follow-up commit | the documentation/changelog commit whose parent is `4e5337d2`. Its SHA and the workflow runs it triggers do not exist when this section is written; they are reported in the implementation handoff |
| Implementing session | `local_0ba55677-ce1d-43d3-98cd-23e488bee658` (under `5678927229`) |

Preflight (read-only, re-run on 2026-09-15 before any write):

- `git fetch origin`, then `origin/feature/metrics` = `3db0699c`, tree
  `14210f97`. `git ls-remote` agrees.
- PR #921 is OPEN, DRAFT and unmerged, with head `4e5337d2` (tree `c57e1463`),
  base `feature/metrics`, no labels and no review requests.
- The remote branch head is `4e5337d2`. #908 has no comment newer than
  `5678927229`, and PR #921 has no comments or reviews.
- The session worked in a fresh detached worktree at `4e5337d2` in its own
  scratch directory, clean at creation. The previous owner's worktree was not
  used and nothing was copied from any other implementation tree.

### 17.4 Footprint

Final permitted PR #921 footprint (`5667624555` section 4, restated by
`5678927229` section 3): exactly four existing paths, all modifications, with no
addition, deletion, move or rename.

| Path | Final footprint | Writable in this follow-up | Edited in this follow-up |
|---|---|---|---|
| `CHANGELOG.md` | yes | yes | yes (section 17.5) |
| `thoth-api/src/model/metric_ingestion_lifecycle/mod.rs` | yes | **no, frozen at `4e5337d2`** | no (section 17.6) |
| `thoth-api/src/model/metric_ingestion_lifecycle/tests.rs` | yes | **no, frozen at `4e5337d2`** | no (section 17.6) |
| `docs/engineering/ai-delivery/implementation-reports/MET-WP2-02-implementation-report.md` | yes | yes | yes (this section only) |

### 17.5 Changelog requirement and correction

Repository-root `AGENTS.md` section 13: "Every PR must update `CHANGELOG.md`
under `## [Unreleased]`." It also requires the appropriate heading, the PR
number when available, and no duplicate headings in the same Unreleased
section. The `check-changelog` workflow (`tarides/changelog-check-action@v4`, on
`pull_request`) failed at `4e5337d2` (run `34862028300`) because the three-path
footprint did not touch `CHANGELOG.md`. The `no changelog` label was not used.

Correction: one bullet was added as the first entry under the existing
`### Fixed` heading of the existing `## [Unreleased]` section. No heading was
added or duplicated.

```text
  - `MET-WP2-02-CR1`: fix the `MET-WP2-02` managed-DRIVER claim lifecycle on the `feature/metrics` programme integration branch so a fresh source claim is granted only from canonical source, source-account, platform and publisher authority revalidated under lock at lease grant (PR [921](https://github.com/thoth-pub/thoth/pull/921), issue [908](https://github.com/thoth-pub/thoth/issues/908)).
```

### 17.6 Frozen Rust verification

Blob IDs were checked before any edit and again after both edits, immediately
before commit:

| Path | Blob at `4e5337d2` (`git rev-parse 4e5337d2:<path>`) | Index (`git ls-files -s`) | Working tree (`git hash-object`) |
|---|---|---|---|
| `thoth-api/src/model/metric_ingestion_lifecycle/mod.rs` | `3028cf8003856fc877ec4e9093555c9eb759e0fb` | `3028cf8003856fc877ec4e9093555c9eb759e0fb` | `3028cf8003856fc877ec4e9093555c9eb759e0fb` |
| `thoth-api/src/model/metric_ingestion_lifecycle/tests.rs` | `393261f6b23e14115f5d2503c1ba86ead311e43a` | `393261f6b23e14115f5d2503c1ba86ead311e43a` | `393261f6b23e14115f5d2503c1ba86ead311e43a` |

The pre-commit checks `git diff --quiet 4e5337d2 -- <both paths>` (working
tree) and `git diff --cached --quiet 4e5337d2 -- <both paths>` (index) both
exit 0. The post-commit exact-blob comparison against the new head is reported
in the implementation handoff.

### 17.7 Local validation for this follow-up

Pre-commit, in the follow-up worktree (HEAD `4e5337d2`, both edits staged):

```text
git diff --cached --check                                             exit 0 (no output)
git diff --check 3db0699cdc9d5a212b0f48ec9fa674e499776cc2             exit 0 (no output)
git diff --cached --name-status 4e5337d2                              exit 0; exactly:
                                                                        M CHANGELOG.md
                                                                        M docs/engineering/ai-delivery/implementation-reports/MET-WP2-02-implementation-report.md
git diff --cached --name-status 3db0699cdc9d5a212b0f48ec9fa674e499776cc2
                                                                      exit 0; exactly four paths, all M:
                                                                        M CHANGELOG.md
                                                                        M docs/engineering/ai-delivery/implementation-reports/MET-WP2-02-implementation-report.md
                                                                        M thoth-api/src/model/metric_ingestion_lifecycle/mod.rs
                                                                        M thoth-api/src/model/metric_ingestion_lifecycle/tests.rs
git diff --cached --numstat 4e5337d2 -- CHANGELOG.md                  exit 0; 1 insertion, 0 deletions
git diff --cached 4e5337d2 -- <report>                                exit 0; additions only, all after the last line of section 16.16
git status --porcelain (untracked/unstaged)                           nothing outside the two staged paths
frozen Rust blob / diff --quiet checks                                exit 0 (section 17.6)
```

Post-commit, `git diff --check 3db0699c...HEAD` and
`git diff --name-status 3db0699c...HEAD` are re-run against the new head, and
their results are reported in the implementation handoff.

Rust build, tests, Clippy and SDL generation were not re-run for this
follow-up, as scoped by `5667624555` and `5678927229`. The Rust paths are
byte-identical to `4e5337d2`, whose full local validation is in sections 16.12
and 16.13 and whose exact-head `build-test-and-check` (run `34862028270`) and
`run-migrations` (run `34862028367`) succeeded. Natural exact-head CI on the new
head remains authoritative.

### 17.8 Effects

- Rust source, tests, migrations, `schema.rs`, GraphQL SDL, authorization and
  runtime behaviour: unchanged by this follow-up.
- External effects are limited to the non-force push to the existing PR #921
  branch and the normal PR `synchronize` workflows, including any
  already-approved automatic `ghcr.io/thoth-pub/thoth:staging-pr-921` image.
- None of the following: manual CI dispatch, rerun or cancel; issue or comment
  mutation; `no changelog` label; ready-for-review transition; reviewer
  request; merge; change to `feature/metrics`; Sphinx change or producer
  rebind; deployment; release; production migration; provider, credential or
  service-role access; activation.

### 17.9 Remaining gates

Fresh independent CRITICAL exact-head review of the new PR head, including its
exact natural CI; separate exact-head CTO merge authorization; merge; fresh
downstream Sphinx producer-contract rebind (thoth-sphinx#10); later
integration, deployment and activation gates. Nothing here is self-approved.
