# MET-WP2-03 Implementation Report

Task: `MET-WP2-03 - Extend managed DRIVER claim context and checkpoint cursor`
Programme: Thoth Metrics / MOM-1 ([#766](https://github.com/thoth-pub/thoth/issues/766))
Risk: **CRITICAL**

Authority condition: this report records the implementation as committed on
the task branch. Live review, authorization, CI and merge evidence is GitHub
issue and pull-request history.

## 1. Repository state

| Item | Value |
|---|---|
| Owning GitHub issue | [#924](https://github.com/thoth-pub/thoth/issues/924) |
| Repository | `thoth-pub/thoth` |
| Workflow | `PROGRAMME_INTEGRATION` |
| Base branch | `feature/metrics` |
| Authorized base commit | `b8e60a157a0dfc7fc9d52edfe45f18137a50d9ea` (tree `ad422bc13754b21bf4b113b2cc7854e2d6bf8eb5`) |
| Actual base commit | `b8e60a157a0dfc7fc9d52edfe45f18137a50d9ea` (parent of the first task commit) |
| PR target / programme integration branch | `feature/metrics` |
| Task branch | `feature/metrics--wp2-03-claim-context-cursor` |
| Head commit | the head of PR #926: a commit cannot record its own SHA; the task commits are listed in section 3 |
| Pull request | [#926](https://github.com/thoth-pub/thoth/pull/926), DRAFT, from the task branch into `feature/metrics` |
| Expected branch deletion after merge | YES |
| Final programme PR required | YES (`feature/metrics -> develop`, separately authorized) |
| Implementing model | Claude Opus 5 |
| Reasoning level | max |
| Implementation session | `local_92be0564-b88a-4b66-82d3-3a14deabfba9` (scratchpad `9f4a6857-1ee6-42c7-ac41-08f508182a0f`) |
| Observed `develop` | `395cc16ac770bc8bbf8a708662a1b31d85b15398` |

### 1.1 Authorization provenance

| Record | #924 comment |
|---|---|
| Specification candidate | issue body |
| Independent CRITICAL specification review (changes required, R1-R8) | `5704272099` |
| Specification Amendment 1 (exact claim context, cursor, SDL, truth table, eight-path budget) | `5712168754` |
| Final independent CRITICAL specification approval | `5712318610` |
| Eight-path implementation authorization (historical; superseded for resumption) | `5714157833` |
| Specification / Write-Budget Amendment 2 (ninth path) | `5715914869` |
| Independent CRITICAL review of Amendment 2 (approved) | `5716109357` |
| Nine-path implementation reauthorization (controlling) | `5716650538` |

Predecessor prerequisite: `MET-WP2-02` / #908 CLOSED, state reason completed.

### 1.2 Preflight, hold and resumption history

1. Under `5714157833` the preflight verified `feature/metrics = b8e60a15`
   (tree `ad422bc1`), `develop = 395cc16a`, #908 closed as completed, no
   local or remote task branch, no task PR and no competing session, worktree,
   scratch directory or database cluster for the task. The task branch was
   created locally from exactly `b8e60a15` in an isolated worktree outside the
   repository.
2. Before any source edit, `the_lifecycle_objects_and_inputs_match_the_frozen_contract_exactly`
   in `thoth-api/src/graphql/metric_ingestion_lifecycle_tests.rs` was proven to
   pin the six-field `MetricSourceUnitClaim` signature that the approved SDL
   must change, outside the eight-path budget. Implementation stopped (HOLD)
   with no source edit, commit, push, PR or issue mutation.
3. Amendment 2 added that file as the ninth path for the narrow contract-test
   purpose, its independent review approved it, and `5716650538` reauthorized
   this session to resume on the same local branch.
4. The resumption preflight (2026-09-17T15:15:35Z) verified: current branch is
   the task branch; `HEAD = b8e60a15`; tree `ad422bc1`; merge-base with the
   authorized base is the base; `git log b8e60a15..HEAD` is empty; the
   worktree is clean; remote `feature/metrics` is unchanged; the remote task
   branch is absent; there is no task PR; and no other session owns the task.
5. The baseline SDL was regenerated from that untouched base through
   `thoth-client/build.rs` before any source edit (section 7).

## 2. Scope confirmation

Approved specification: the #924 specification candidate as amended by
`5712168754` and `5715914869`, approved by `5712318610` and `5716109357`;
implementation authority `5716650538`.

Implemented objective: a successful claim returns one internally consistent
runtime snapshot — the existing source defaults and typed account
configuration, the stable `platformCode` copied from the locked eligibility
platform row, and the typed `thoth-period-manifest-cursor/1` checkpoint
cursor decoded before the lease is written — and a successful checkpoint
update records the import's accepted period manifest in the existing
`metric_source_checkpoint.cursor` column, server-derived and atomic with
progress and release.

Out-of-scope changes made: NONE.

## 3. Commits

- `d7a1bbc01d8bb04bdf1acc86c8bb67b7669f6b74` (tree
  `4a2af3337c52e14e5cc6365ec50adf747925cee8`) — `MET-WP2-03: extend the
  managed-DRIVER claim context and checkpoint cursor`: production source,
  tests, contract register and changelog (eight paths).
- `a4bcdb5f512fd6843e4deaa6c572c6a7f8e5e3be` (tree
  `8a803da255995cb12cb0a6625687c94aa51087f5`) — `MET-WP2-03: prove claim
  rollback and one atomic checkpoint write`: lifecycle tests only, no
  production change.
- The commit adding this report (the ninth path) follows them; a commit cannot
  record its own SHA, so its identity is the PR head.

## 4. Files changed

Authorized existing paths and the new path, all within `5716650538`:

| # | Path | Status | Reason and behavioural effect |
|---|---|---|---|
| 1 | `CHANGELOG.md` | M | One `Unreleased / Added` entry for `MET-WP2-03`. No behaviour. |
| 2 | `docs/metrics/contract-register.md` | M | New section 3.3 records the durable claim-context and cursor contract; the section 3.2 statement that `cursor` is never written is corrected. No behaviour. |
| 3 | `thoth-api/src/graphql/model.rs` | M | `MetricSourceUnitClaim.platformCode` and `.periodManifestCursor` resolvers; the `MetricPeriodManifestCursor` and `MetricPeriodManifestCursorEntry` objects. |
| 4 | `thoth-api/src/model/metric_ingestion_lifecycle/mod.rs` | M | Claim carries the locked platform code and the claim-time decoded cursor; checkpoint update derives and records the cursor entry. |
| 5 | `thoth-api/src/model/metric_ingestion_lifecycle/tests.rs` | M | Cursor, claim-context, fail-closed, retention, replay and race evidence; `snapshot()` now also covers the cursor. |
| 6 | `thoth-api/src/model/metric_source_checkpoint/mod.rs` | M | The closed cursor codec, its two domain types, error type and two shared constants. |
| 7 | `thoth-api/src/model/metric_source_checkpoint/tests.rs` | M | Codec evidence; the Diesel round-trip fixture uses a canonical cursor. |
| 8 | `thoth-api/src/graphql/metric_ingestion_lifecycle_tests.rs` | M | Amendment 2 purpose only: the exact `MetricSourceUnitClaim` signature, the two cursor object signatures, reachability, no caller-supplied cursor, and the typed claim through GraphQL. |
| 9 | `docs/engineering/ai-delivery/implementation-reports/MET-WP2-03-implementation-report.md` | A | This report. |

Files deleted, moved or renamed: NONE.

### 4.1 Write-budget compliance

PASS. Exactly nine paths: eight modified authorized paths and the one
authorized new file. No migration, `schema.rs`, Cargo, dependency, workflow,
policy, generated-artifact, Sphinx or task-status path was touched. In path 8,
every pre-existing exact assertion is preserved except the one
`MetricSourceUnitClaim` signature, which gains exactly the two approved fields;
`UpdateMetricSourceCheckpointInput` stays pinned to `importId` and
`leaseToken`.

### 4.2 Authorized actions actually used

- repository inspection: yes
- source edit: yes, the eight existing authorized paths
- new file creation: yes, this report only
- file deletion/move/rename: no
- branch creation: yes, under `5714157833`, from exactly `b8e60a15`; not recreated on resumption
- commit: yes (section 3)
- push: yes, non-force, task branch only: the two task commits, then this report commit
- PR creation/update: yes, DRAFT #926 into `feature/metrics`, body updated for the handoff
- issue/comment mutation: no
- manual CI dispatch/rerun/cancel: no
- provider/runtime read: no
- provider/runtime write: no
- migration execution: no (local disposable test databases only)
- release/tag/publication: no
- merge: no
- deployment: no
- production activation: no
- other: none

Unauthorized actions performed: NONE.

### 4.3 Automatic and manual external effects

Automatic effects: the non-force pushes and DRAFT PR #926 trigger the
repository's natural `pull_request` workflows, including the existing automatic staging
image build. Their run identifiers and conclusions are recorded in the PR
history and the implementation handoff.

Manually initiated external actions: NONE.

External writes/publication beyond those automatic workflows: NONE.

## 5. Implementation decisions

1. **Codec ownership.** `metric_source_checkpoint` owns
   `PERIOD_MANIFEST_CURSOR_SCHEMA`, `PERIOD_MANIFEST_CURSOR_MAX_ENTRIES`,
   `MetricPeriodManifestCursor`, `MetricPeriodManifestCursorEntry` and
   `MetricPeriodManifestCursorError`. The types' fields are private and the
   only constructors are `decode` and `record_accepted_manifest`, so every
   value in memory already satisfies the representation and `encode` can only
   emit canonical JSON built from validated entries. The error carries no part
   of the stored value. The lifecycle implements no cursor rule of its own.
2. **Canonical dates.** A `periodStart` is accepted only if
   `NaiveDate::parse_from_str(_, "%Y-%m-%d")` succeeds and rendering the date
   back is byte-identical and exactly ten characters. Years outside 0000-9999
   therefore have no canonical rendering; `record_accepted_manifest` refuses
   them rather than emit a value the decoder would reject.
3. **Platform authority.** `check_eligibility` returns the pinned publisher
   and the `code` of the very `MetricPlatform` value it loaded (`FOR SHARE`
   under the claim's revalidation) and found enabled; `revalidate_claim_authority`
   carries it into the claim. No additional query, lock or later resolver is
   involved. `begin_metric_import` uses only the publisher, as before.
4. **Claim-time decode.** For each unit still eligible after revalidation, the
   stored cursor is read from the checkpoint row that the claim's
   `FOR UPDATE SKIP LOCKED` statement already locked, decoded, and only then is
   the lease written. A unit skipped by enumeration or revalidation is never
   decoded. A decoding failure returns `INTERNAL_STATE_INCONSISTENCY` through
   the existing coordinator vocabulary, which rolls back the whole claim
   transaction: no lease and no bootstrap insert of that call survives. The
   server log records the checkpoint identifier only.
5. **Server-derived advancement.** The strict managed-envelope decoder is one
   function, `decode_manifest_envelope`, that `expected_batch_keys` now
   delegates to unchanged. The update derives the digest only from the
   import's stored envelope and the period only from `metric_import.period_start`.
   The existing `successful` boolean is bound once and drives both
   `last_successful_period_end` and `cursor = CASE WHEN $2 THEN $5 ELSE cursor END`
   in the one existing `UPDATE` that also sets `last_completed_at` and releases
   the lease, so no second success predicate exists and no partial write is
   possible.
6. **Interpretation E1 — envelope mapping.** A terminal one-day import with a
   completion time whose stored envelope does not decode fails as
   `INTERNAL_STATE_INCONSISTENCY` on both the live and the released path. The
   existing refusals keep their precedence: a checkpoint that is neither live
   for this token nor released is `STALE_SOURCE_CLAIM`, and a non-terminal
   import, a non-one-day period or a missing completion time keep
   `INVALID_IMPORT_STATE` (live) or `STALE_SOURCE_CLAIM` (released). This
   interpretation was stated at the HOLD and follows Amendment 1 section 6 and
   Amendment 2 section 6.
7. **Interpretation E2 — cursor validation on update.** The stored cursor is
   decoded on every live update, successful or not, before any write; a live
   update over unsupported cursor state writes nothing. The released replay
   path never decodes, encodes or writes the cursor. This interpretation was
   also stated at the HOLD.
8. **GraphQL wiring.** The two objects are declared in `graphql/model.rs`.
   Juniper emits a `graphql_object` impl block as an inherent impl, so the
   domain accessors are named `retained_entries`, `period` and `digest` to
   leave `entries`, `period_start` and `manifest_digest` to the resolvers.
   `schemaVersion` returns the shared constant.
9. **No-consequence evidence.** The lifecycle test `snapshot()` now includes
   `cursor::text`, so every pre-existing "changes nothing" assertion in both
   lifecycle test modules also proves the cursor unchanged.

Deviation from the specification requiring authorization: NONE.

## 6. Database and migration effects

Migration added: NO. No table, column, index, constraint, trigger or
`schema.rs` change. `metric_source_checkpoint.cursor` stays generic nullable
JSONB at the database level
(`the_database_column_stays_generic_and_the_codec_is_the_gate`).

Existing data: no backfill and no rewrite. An existing non-null cursor outside
the representation is never repaired; a claim of its still-eligible unit, or a
live update over it, fails closed until a separately authorized repair.

## 7. API and compatibility effects

GraphQL delta, exactly and additively:

```graphql
type MetricPeriodManifestCursor {
  schemaVersion: String!
  entries: [MetricPeriodManifestCursorEntry!]!
}

type MetricPeriodManifestCursorEntry {
  periodStart: Date!
  manifestDigest: String!
}

# MetricSourceUnitClaim, after leaseExpiresAt
platformCode: String!
periodManifestCursor: MetricPeriodManifestCursor
```

Generated SDL (`thoth-client/assets/schema.graphql`, written by
`thoth-client/build.rs` during `cargo build`):

| Tree | Lines | SHA-256 |
|---|---|---|
| Base `b8e60a15` (untouched, before any edit) | 5384 | `bd33dc1b370ff9e5aa35de28b0d1fc0f30e6a90e75f0e767260d3f3001a406af` |
| Validated commit `a4bcdb5f` (production source identical to `d7a1bbc0`) | 5404 | `6a61ba366ef911ec11743267615a2fa8e1504a3a41d51ad224ddb52f60f74d6f` |

`diff -u` of the two files has 20 added lines (18 non-blank, 2 blank) and no
removed line: the two new object declarations with their descriptions, placed
alphabetically between `MetricMeasure` and `MetricPlatform`, and the two new
described fields appended to `MetricSourceUnitClaim`. Removing exactly those
additions from the head file reproduces the base file byte for byte. No existing declaration,
field, argument, input, enum value, scalar or description changed, and no type
moved.

Unchanged: `UpdateMetricSourceCheckpointInput` (`importId`, `leaseToken`), all
other inputs, the five operations and their return types, `MetricSourceCheckpoint`,
`MetricSource`, `MetricSourceAccount`, `MetricPlatform`, `QueryRoot` and every
public type.

Backwards compatibility: additive for existing selections. A claim now fails
closed when an eligible unit's stored cursor is unsupported; no such value can
be written through the protected surface.

Cross-repository: `thoth-sphinx` does not change here. Its producer-contract
rebind to the new SDL is a separate later gate after this task merges.

## 8. Authorization and security

- Authorization paths changed: none. All five resolvers still call
  `authorize_metric_ingestion_lifecycle` (`METRICS_INGEST_SERVICE`) before the
  coordinator (`every_resolver_authorizes_before_calling_the_lifecycle_coordinator`),
  and the complete negative matrix still fails closed before any database
  access (`every_operation_denies_every_principal_but_the_ingest_service_without_touching_state`,
  `authorization_is_decided_before_any_operation_specific_database_access`).
- Reachability: the cursor objects are returned only by
  `MetricSourceUnitClaim.periodManifestCursor`, the claim only by
  `claimMetricSourceUnits`, and `platformCode` is an output field of the claim
  only; `QueryRoot`, `MutationRoot`, `MetricSourceCheckpoint`, `MetricSource`,
  `MetricSourceAccount`, `MetricPlatform`, `MetricImport` and the public types
  mention neither cursor nor manifest
  (`the_claim_context_is_reachable_only_through_the_protected_claim_result`).
- Caller input: no cursor, manifest digest or period can be supplied to the
  checkpoint update; such documents fail validation before any resolver runs
  (`the_checkpoint_update_accepts_no_caller_supplied_cursor_or_manifest`).
- Sanitization: failures use the fixed coordinator message and code; no stored
  cursor, JSON, SQL, constraint name, token or provider detail reaches a
  client (`a_decoding_failure_reveals_nothing_of_the_stored_value`,
  `a_claim_returns_its_locked_platform_code_and_the_accepted_period_manifest_cursor`).
- Secrets and personal data: the representation admits only periods and
  SHA-256 digests. No credential, source object, request, viewer or session
  data can be stored through it.

## 9. Tests and checks

### 9.1 Repository gates (commit `a4bcdb5f`, clean tree)

Environment: section 10. Each command ran from a clean worktree at
`a4bcdb5f` (tree `8a803da2`) between 2026-09-17T15:57:50Z and 16:16:22Z,
against a freshly created test database.

| # | Command | Exit | Result |
|---|---|---|---|
| 1 | `cargo fmt --all -- --check` | 0 | no formatting difference |
| 2 | `cargo test -p thoth-api --features backend` | 0 | lib `1912 passed; 0 failed; 1 ignored`; `tests/graphql_permissions.rs` `13 passed`; doc-tests `0 passed; 8 ignored` |
| 3 | `cargo test --workspace` | 0 | every target `ok`: `thoth` bin 31; `thoth-api` lib 1912 (1 ignored); `graphql_permissions` 13; `thoth-api-server` 3; `thoth-client` 4 and 6 doc-tests; `thoth-errors` 13; `thoth-export-server` 144 and 2 doc-tests; `thoth-api` doc-tests 8 ignored — 2128 passed, 0 failed, 9 ignored |
| 4 | `cargo check --workspace` | 0 | no diagnostic from a workspace crate |
| 5 | `cargo clippy --all --all-targets --all-features -- -D warnings` | 0 | no diagnostic |
| 6 | `cargo build` | 0 | SDL generated by `thoth-client/build.rs` (section 7) |
| 7 | `git diff --check b8e60a157a0dfc7fc9d52edfe45f18137a50d9ea...HEAD` | 0 | no whitespace error |

Steps 4-6 print only the dependency future-incompatibility notice for
`proc-macro-error2 v2.0.1`, which the untouched base build also printed. This
task adds 24 tests: 12 codec, 9 lifecycle model and 3 GraphQL.

### 9.2 RED -> GREEN evidence

RED 1, runtime, production code unchanged. Only
`metric_ingestion_lifecycle/tests.rs` and
`graphql/metric_ingestion_lifecycle_tests.rs` had been edited:

```text
cargo test -p thoth-api --features backend --lib -- metric_ingestion_lifecycle
test result: FAILED. 37 passed; 11 failed; 0 ignored; 0 measured; 1851 filtered out; finished in 13.91s
```

| Failing test | Failure observed on the unchanged production code |
|---|---|
| `the_lifecycle_objects_and_inputs_match_the_frozen_contract_exactly` | claim signature lacks `platformCode` / `periodManifestCursor` |
| `the_claim_context_is_reachable_only_through_the_protected_claim_result` | no field returns `MetricPeriodManifestCursor` |
| `a_claim_returns_its_locked_platform_code_and_the_accepted_period_manifest_cursor` | `Unknown field "platformCode"` / `"periodManifestCursor"` |
| `a_successful_update_records_its_period_manifest_and_replaces_only_that_period` | cursor `None` after a successful update |
| `only_a_completed_import_with_exact_complete_coverage_advances_successful_progress` | cursor `None` for the advancing cases |
| `no_update_outside_the_successful_period_predicate_changes_an_accepted_cursor` | no accepted cursor to preserve |
| `stale_expired_reclaimed_and_foreign_tokens_never_write_the_cursor` | the live holder's update records no cursor |
| `the_sixty_fifth_retained_period_evicts_only_the_oldest_and_a_released_replay_never_resurrects_it` | cursor `None` after the first success |
| `an_unsupported_stored_cursor_fails_the_claim_closed_before_any_lease_is_written` | claim succeeds over a JSON `null` cursor |
| `an_unsupported_cursor_or_import_envelope_fails_the_update_with_no_partial_write` | update succeeds over a JSON `null` cursor |
| `a_current_and_a_stale_checkpoint_update_race_to_one_durable_outcome` | cursor `None` after the race |

The regression guard `the_checkpoint_update_accepts_no_caller_supplied_cursor_or_manifest`
passed on the unchanged code, as intended.

RED 2, compile, codec tests added before the codec:

```text
cargo test -p thoth-api --features backend --lib --no-run
error[E0432]: unresolved imports `super::MetricPeriodManifestCursor`, `super::MetricPeriodManifestCursorError`, `super::PERIOD_MANIFEST_CURSOR_MAX_ENTRIES`, `super::PERIOD_MANIFEST_CURSOR_SCHEMA`
```

GREEN, after the implementation:

```text
cargo test -p thoth-api --features backend --lib -- metric_ingestion_lifecycle metric_source_checkpoint
test result: ok. 70 passed; 0 failed; 0 ignored; 0 measured; 1842 filtered out; finished in 16.80s
```

The first GREEN run exposed one fixture defect, not an implementation defect:
the "uppercase digest" cases uppercased `digest(1)`, a digest made only of
decimal digits, which uppercasing leaves unchanged. The fixtures now uppercase a
digest containing hexadecimal letters (`0xabcdef`). The cases became stricter;
no assertion was removed or weakened.

The typed model-level claim assertions (`platform_code`,
`period_manifest_cursor`) were added with the implementation, because they
cannot compile against the unchanged types; the same behaviour was RED first at
the API boundary.

Self-review after GREEN strengthened the atomicity evidence in the separate
test-only commit `a4bcdb5f`, before any push:

- the claim fail-closed test was moved from the first unit to the second, so a
  failing claim must undo an earlier lease and a bootstrapped checkpoint from
  the same transaction. A frozen run shows the claim reaching that earlier
  lease write before it fails;
- `a_failed_cursor_write_commits_no_progress_and_no_release_either` was added.
  On the unchanged production code it would fail, because no cursor write
  occurs and the update succeeds. It was not run there: the test module also
  contains typed assertions that do not compile against the unchanged types.

The same focused command, on the tree then committed as `a4bcdb5f`:

```text
test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 1842 filtered out; finished in 18.11s
```

### 9.3 Evidence map

Codec (`model::metric_source_checkpoint::tests`):

| Requirement | Test |
|---|---|
| shared constants | `the_shared_constants_are_the_approved_schema_and_bound` |
| SQL `NULL` only; JSON `null`, `{}` and zero entries refused | `sql_null_is_the_only_empty_history` |
| 1-, 2- and 64-entry round trip | `a_supported_cursor_round_trips_exactly_at_one_and_sixty_four_entries` |
| exact schema version; exact top-level and entry members; non-object shapes | `only_the_exact_schema_version_and_key_sets_are_supported` |
| strict canonical date round trip | `a_period_start_must_round_trip_as_one_canonical_date` |
| 64 lowercase hexadecimal digest characters | `a_manifest_digest_must_be_64_lowercase_hexadecimal_characters` |
| strict order, uniqueness, 65 and 1000 entries refused | `entries_must_be_strictly_ascending_and_at_most_sixty_four` |
| deterministic canonical encoding | `encoding_is_canonical_and_deterministic` |
| insert, same-value replace, changed-digest replace, oldest-only eviction, older-than-retained eviction | `recording_inserts_or_replaces_one_period_and_evicts_only_the_oldest` |
| no unrepresentable result | `recording_never_produces_a_cursor_the_decoder_would_refuse` |
| sanitized error | `a_decoding_failure_reveals_nothing_of_the_stored_value` |
| column stays generic JSONB; codec is the gate | `the_database_column_stays_generic_and_the_codec_is_the_gate` |

Claim (`model::metric_ingestion_lifecycle::tests`):

| Requirement | Test |
|---|---|
| null cursor is no history; platform code on a fresh claim | `the_first_claim_bootstraps_one_default_checkpoint_per_account_and_leases_it_under_a_fresh_token` |
| each unit's own platform code; typed 64-entry snapshot equals the stored value | `a_claim_carries_each_units_own_locked_platform_code_and_its_decoded_cursor` |
| platform code from the platform row held locked while a real platform writer queued | `cr1_platform_authority_cannot_go_stale_between_eligibility_and_lease_grant` |
| unsupported cursor (15 shapes) on the second unit fails the claim; the earlier lease (reached, as shown by a claim frozen at that write) and the bootstrapped checkpoint of the same transaction do not survive; nothing is repaired; ineligible and revalidation-skipped units are not decoded | `an_unsupported_stored_cursor_fails_the_claim_closed_before_any_lease_is_written` |
| existing real source, account, platform and publisher claim/admin races | the four `cr1_*_cannot_go_stale_*` race tests, unchanged in intent |

Checkpoint update (`model::metric_ingestion_lifecycle::tests`):

| Requirement | Test |
|---|---|
| first advancement; another period; same-period same digest byte-identical; real managed `REVISION` replaces only its period; older period keeps progress | `a_successful_update_records_its_period_manifest_and_replaces_only_that_period` |
| advancement exactly under the successful-period predicate | `only_a_completed_import_with_exact_complete_coverage_advances_successful_progress` |
| zero coverage, `PARTIAL`, `UNKNOWN`, `COMPLETE`+`UNKNOWN`, `COMPLETED_WITH_ERRORS` keep an accepted cursor; non-terminal and `FAILED` mutate nothing | `no_update_outside_the_successful_period_predicate_changes_an_accepted_cursor`, `completed_with_errors_releases_the_lease_but_never_claims_success` |
| expired, reclaimed and foreign tokens write nothing; the live holder records | `stale_expired_reclaimed_and_foreign_tokens_never_write_the_cursor` |
| 65th retained period evicts only the oldest; released replays are read-only and never resurrect; an older-than-retained success is evicted at once without moving progress | `the_sixty_fifth_retained_period_evicts_only_the_oldest_and_a_released_replay_never_resurrects_it` |
| unsupported cursor or envelope: `INTERNAL_STATE_INCONSISTENCY`, no partial write, live and released, successful and unsuccessful; non-terminal keeps its refusal | `an_unsupported_cursor_or_import_envelope_fails_the_update_with_no_partial_write` |
| a failure injected into the checkpoint write leaves cursor, progress and lease all unchanged; without it the same update records all three | `a_failed_cursor_write_commits_no_progress_and_no_release_either` |
| real current-versus-stale race, both lock orders, final durable state | `a_current_and_a_stale_checkpoint_update_race_to_one_durable_outcome` |

The race test holds the checkpoint row `FOR UPDATE` on a separate session,
starts each update on a pinned connection, and proceeds only once
`pg_stat_activity` reports that backend waiting on a lock. It therefore fixes
the queue order through PostgreSQL itself, runs it once stale-first and once
current-first, and asserts the stored cursor, lease and progress, which are
identical in both orders. When the stale update is admitted first it is
refused; when it follows the live holder's release it is answered read-only.
Neither order writes.

GraphQL (`graphql::metric_ingestion_lifecycle_tests`): the exact signatures,
reachability, rejection of caller-supplied cursor input, and the typed claim
through the real schema (null, then the accepted manifest after a complete
unit, then a sanitized failure with no lease) are listed in section 8.

### 9.4 Locking, concurrency and idempotency

- No new lock or lock-order edge. The claim reads the cursor from a checkpoint
  row it already holds `FOR UPDATE`, after the existing account, source,
  platform and publisher `FOR SHARE` sequence, and takes the platform code from
  the platform row already locked in that sequence. The update reads and writes
  the cursor on the checkpoint row that `lock_checkpoint` already holds
  `FOR UPDATE`; the import `FOR SHARE` and the coverage read are unchanged.
- The cursor write is part of the existing single checkpoint `UPDATE`, so the
  cursor, progress and release commit or roll back together.
- Retry after an uncertain commit keeps the existing semantics: before commit
  nothing is written; after commit the released replay is read-only and
  independent of cursor content.

## 10. Manual verification

Environment:

- a dedicated Homebrew PostgreSQL 17.10 cluster on its own TCP port, database
  `thoth_test` created `UTF8` from `template0` with `C` collation
  (`char_length('é') = 1`);
- a dedicated Redis instance on its own port;
- the isolated git worktree outside the repository, with no `.env`;
  `THOTH_EXPORT_API` exported with CI's value, `TEST_DATABASE_URL` and
  `TEST_REDIS_URL` exported for the local services, and `CARGO_INCREMENTAL=0`
  with a shared local build cache.

No shared, staging or production database or provider was accessed.

## 11. CI

Natural PR-triggered CI only, with no manual dispatch, rerun or cancel. Run
identifiers and conclusions for the exact PR head are in the PR history.

## 12. Rollout and rollback

Initial state after merge: inactive. No cursor is written until an
authenticated `METRICS_INGEST_SERVICE` caller records a successful import of an
enabled, configured `DRIVER` source; existing cursor values are not touched by
the merge.

Activation required: the separately governed machine principal, real source
configuration and the Sphinx producer-contract rebind. None is part of this
task.

Feature flag/configuration: none. Migration sequence: none.

Rollback/disable procedure: withhold the role or disable the source or account.
A source revert is a separately authorized change. Cursor values written in the
meantime are ordinary JSONB in an existing column, and the reverted code
ignores them, as `MET-WP2-02` did.

Monitoring: an unsupported stored cursor is logged server-side with its
checkpoint identifier and surfaces to the caller as `INTERNAL_STATE_INCONSISTENCY`.

## 13. Known limitations and deferred work

- One unsupported stored cursor on an eligible unit fails every claim of its
  source that selects that unit, until a separately authorized repair; this is
  the approved fail-closed behaviour.
- A successful import whose period has no canonical `YYYY-MM-DD` rendering
  (year outside 0000-9999) cannot be recorded; its update fails closed. This is
  proven at the codec level only.
- `last_error`, checkpoint administration, multi-partition sources and
  non-`DRIVER` lifecycles remain deferred.
- Local validation used CI's environment variable names (with CI's
  `THOTH_EXPORT_API` value and local disposable database and Redis URLs) and a
  shared local build cache; CI remains the exact-head authority.

## 14. Unresolved issues

- NONE known to the implementing session.

## 15. Agent self-assessment

The agent may identify risks but may not approve the task.

Suggested review focus:

- interpretations E1 and E2 (section 5);
- that the claim decodes only after the locked revalidation and before the
  lease write, and that a decoding failure rolls back every lease of the call;
- that `platformCode` is copied from the same locked `MetricPlatform` value;
- the single `successful` binding shared by progress and cursor in one `UPDATE`;
- the race test's observation order and both lock orders (section 9.3);
- the exact nine-path footprint and that path 8 keeps every unrelated exact
  assertion.

Gate authority: this report is implementation evidence, not a gate status.
Implementation reports do not self-approve, and GitHub owns live lifecycle
state. The task head needs an independent CRITICAL exact-head source review,
and a merge needs a fresh CTO merge authorization bound to that reviewed head.
The downstream Sphinx rebind is a separate later gate.
