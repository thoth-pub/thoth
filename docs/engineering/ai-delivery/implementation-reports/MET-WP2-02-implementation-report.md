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
