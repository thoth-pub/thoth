# THOTH-ASYNC-01-ENGINE-01A Implementation Report

This report records E1, the inert core of the shared job engine (spec section 22.1), at code head `5ee645147fd78c208e6c96924bfa2e7f630e5130`, after review round 1. The commit that rewrites this report is a child of that head and changes only this file. The report was first added in `80fa214863a615dd6dab2800445ef482235ba1be`, over code head `0529a403edc66cd96cdb7f587e580f4619e13029`; both review legs reviewed `80fa2148`.

## Review round 1

Both CRITICAL review legs reviewed E1 at `80fa214863a615dd6dab2800445ef482235ba1be` and returned APPROVED (thoth-pub/thoth#1026, comment 6871598999):

- Opus leg: P0 0, P1 0, P2 1, P3 4 (OPE-1 to OPE-5).
- Fable leg: P0 0, P1 0, P2 1, P3 8 (FBE-1 to FBE-9).

Dispositions, one line per finding:

- **CI `lint` at `80fa2148`** (not a review finding): fixed in `4d3d48f` (section 11).
- **OPE-1 (P2)**, G6 and `develop`'s OPS-03 migration guard: recorded in section 14 (what G6 must re-run and resolve).
- **OPE-2 (P3)**, `is_non_blank` laxer than `[:space:]` under an ICU default collation: fixed in `663e87c`. U+001C to U+001F count as blank, the comment is corrected, and the blank samples of `bounds_a_blank_reason_is_payload_invalid_before_any_write` include the four characters.
- **OPE-3 (P3)**, a false invariant behind the 24-hour terminal counts: the comment is corrected in `5ee6451` and the effect is recorded in section 13. The query is unchanged, because no change is both exact and bounded by an existing index: no index of `async_job` has `terminal_at`, `created_at` does not bound when a job ended, so `(status, updated_at)` is the only index that can bound the window, and widening its bound would need a limit on transaction length that the engine does not have (an `_on` call runs inside the caller's transaction).
- **OPE-4 (P3)**, two report statements: recorded. Section 4.1 and reading 13 are reworded.
- **OPE-5 (P3)**, the key for the v1.9.0 server's JWT client assertion: recorded in reading 21 and section 6.1, item 6. Its local files were deleted in this round.
- **FBE-1 (P2)**, the recovery selection not bounded by due time: fixed in `fd9aa5b` (plan check below; reading 22).
- **FBE-2 (P3)**, the signals read the whole job table for their kinds and return `regressed_targets` without a bound: deferred to E2, the first caller of the signals (section 13).
- **FBE-3 (P3)**, reading 18 fills a gap in the spec: recorded as a spec clarification for E2 (section 13). The code is unchanged.
- **FBE-4 (P3)**, part A item 1 is tested on synthetic errors: recorded in section 9.2.
- **FBE-5 (P3)**, `json_measure` and the `AlreadyInTransaction` mapping classify too broadly: deferred to E2 (section 13). Neither path is reachable in E1.
- **FBE-6 (P3)**, a REVISION_BOUND READ_ONLY job with no targets can finish naming nothing: deferred to E2 (section 13). No REVISION_BOUND kind is registered in E1.
- **FBE-7 (P3)**, working detail in the report: recorded. The local paths, the pointers to a working folder and the decision-card wording are removed. Section 0 cites the amendment as #1026 comment 6870622120, and the commands quoted in section 6.1 reproduce the evidence.
- **FBE-8 (P3)**, 158 mapped tests against 159 `#[test]` functions: recorded in section 9.1.
- **FBE-9 (P3)**, a committed job row left in a test database: no change. `setup_test_db` truncates every `public` table at the start of each test (spec 7.10 allows it), so a row that an earlier run leaves behind affects no later test.

**The FBE-1 change.** `recover_rows` reads `clock_now` once, before its selection. It binds that instant as `$3::timestamptz` in both due-time comparisons and uses it for every row's handling (reading 22). There is no schema change. The 158 E1 tests and the B1 guard pass unchanged (section 9).

The plan was checked on PostgreSQL 17.10, in a scratch database copied with `CREATE DATABASE ... TEMPLATE` from the migrated, empty evidence database `ev19_fresh` and dropped afterwards:

- 60,000 probe rows were inserted with the triggers bypassed (`session_replication_role = replica`, as the test fixtures do): 20,000 claimed RUNNING rows (50 past their lease), 20,000 WAITING rows (50 past their deadline) and 20,000 SUCCEEDED rows. Then the table was analyzed.
- `EXPLAIN (GENERIC_PLAN)` of the selection as the engine now sends it shows both scans bounded by the due time:

  ```text
  ->  Bitmap Index Scan on async_job_lease_idx
        Index Cond: (lease_expires_at <= $3)
  ->  Bitmap Index Scan on async_job_waiting_deadline_idx
        Index Cond: (waiting_deadline <= $3)
  ```

- `EXPLAIN (ANALYZE, BUFFERS)` ran on prepared copies of the selection (same predicate, order, limit and locking, with a shorter select list), inside a transaction that was rolled back:
  - Before, with `clock_timestamp()` in the statement: the lease index returned all 20,000 claimed entries with no index condition. The WAITING branch read all 20,000 WAITING entries through `async_job_status_updated_at_idx` (`Index Cond: (status = 'WAITING')`). The filter removed 39,900 rows, and the statement touched 1,857 shared buffers.
  - After, with the bound instant: the custom plan shows the same two index conditions, with the literal instant in place of `$3`. Each scan returned 50 entries, and the statement touched 109 shared buffers.

## 0. The B1 revert guard: HOLD and release

**Before.** At `ebcbab7`, `cargo test -p thoth-api --features backend` on PostgreSQL 17.10 ended `test result: FAILED. 2490 passed; 1 failed; 3 ignored`. The one failure was the existing guard `model::metric_coverage_run::tests::reverting_through_the_b1_migration_removes_exactly_its_objects_and_reapplication_restores_them` (`thoth-api/src/model/metric_coverage_run/tests.rs`, then line 961):

```text
assertion `left == right` failed: only the B1 function is removed
```

The test reverted migrations until B1 (`20261006`) had been reverted, and expected only B1's one function and one trigger to leave the catalogue. E1's `20261115_v1.10.0` sorts after B1, so it was reverted on the way, and the two function lists differed by exactly E1's 12 trigger functions:

`async_control_command_decided_check`, `async_control_command_guard`, `async_effect_target_state_guard`, `async_job_attempt_check`, `async_job_attempt_guard`, `async_job_before_insert`, `async_job_before_update`, `async_job_hold_check`, `async_job_record_transition`, `async_job_target_guard`, `async_job_transition_guard`, `async_target_hold_guard`.

The guard passed at the base `da3e15a3` (2333 passed, 0 failed, 3 ignored). Its file was outside the E1 write budget, so this was a HOLD under spec 22.6 ("an existing catalog or source guard other than the two section 7.11 assertions fails"). It was reported on thoth-pub/thoth#1026 (comment 6870563757, 2026-11-14 02:55Z), and nothing touched the test while it stood.

**Release.** On 2026-11-14 at 03:00Z the CTO chose to allow the test-only fix in E1. The G4 amendment is recorded on thoth-pub/thoth#1026 (comment 6870622120). It adds `thoth-api/src/model/metric_coverage_run/tests.rs` to E1's write budget, for that test only (plus a helper in that file if the change needs one), test code only. It also fixes the change:

- revert every migration newer than `20261006` first and take the "before" lists there;
- then revert B1 and compare exactly as before;
- reapply everything and compare with the fully migrated state;
- on a database where B1 is the newest migration, check exactly what the test checked before.

**The change** (`dd38d9b`, one file, +39/-4):

1. The test first reads the fully migrated catalogue: the public function list, the trigger list, the B1 function and trigger definitions and the index definitions of `metric_coverage`, `metric_import` and `metric_record`. The definitions and index reads are the existing ones, at the same point.
2. A new private helper, `revert_migrations_newer_than_b1`, asserts that B1 is applied, then calls `revert_last_migration` while the newest applied version (the first entry of `applied_migrations`, which diesel orders newest first) is not B1.
3. The test then reads the function list, the trigger list and the raw-evidence digest. These are the "before" values of B1's revert.
4. `revert_through_b1_migration` is called as before. It now reverts B1 only.
5. The removal assertions are unchanged: B1's table, function, trigger and four indexes are gone, the lists equal the "before" lists minus B1's function and trigger, and the raw evidence is unchanged.
6. After `run_pending_migrations`, the definitions, index definitions and both lists are compared with the fully migrated values of step 1. The rest of the test (maintenance after the reapply, the verifier and the rebuild) is unchanged.

On a database where B1 is the newest migration, step 2 reverts nothing, step 3 reads the same catalogue as step 1, and every assertion compares the same values as before. No assertion was removed or loosened and no name is excluded. `revert_through_b1_migration`, its other caller (`the_migration_backfills_nothing_over_populated_raw_evidence`) and every other test are unchanged.

Two placements are readings of the amendment:

- The definitions compared after the reapply are read before any revert, because the amendment compares the reapplied catalogue with the fully migrated state. Where B1 is the newest migration, they are the values the test read before.
- The raw-evidence digest is read at B1, so it still measures B1's downgrade.

**After.**

- The test alone, with the change of `dd38d9b`, on PostgreSQL 17.10: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2494 filtered out; finished in 0.47s`.
- The full suite at the code head `0529a403`: `test result: ok. 2491 passed; 0 failed; 4 ignored` for the library tests, with the guard and the other caller of `revert_through_b1_migration` among the passes (section 9).
- At `5ee6451`, after review round 1, the guard passes again with the 158 E1 tests (section 9).

## 1. Repository state

Owning GitHub issue: thoth-pub/thoth#1026 (programme thoth-pub/thoth#957)
Repository: thoth-pub/thoth
Workflow: PROGRAMME_INTEGRATION
Base branch: `feature/worker`
Authorized base commit: `da3e15a364c4d234dc94e9e03c5992dead59ed33`
Actual base commit: `da3e15a364c4d234dc94e9e03c5992dead59ed33` (`git ls-remote` of `refs/heads/feature/worker` re-checked on 2026-11-14 during review round 1: unchanged; nothing was merged or rebased)
PR target: `feature/worker`
Programme integration branch: `feature/worker`
Task branch: `feature/worker--engine-01a`
Head commit: code head `5ee645147fd78c208e6c96924bfa2e7f630e5130`. The commit that rewrites this report follows it.
Pull request: thoth-pub/thoth#1033, into `feature/worker`
Expected branch deletion after merge: YES
Final programme PR required: YES (`feature/worker` to `develop`, G6, outside this slice)
Implementing model: Claude Opus 5.5 (it continued uncommitted work from an earlier, interrupted run)
Reasoning level: max

`develop` has moved past the base. At `6fe00b5a` (PR #1016, MET-WP4-03A-OPS-03, merged 2026-11-14 02:15Z) it adds migration `20261112_v1.10.0`, one table and one unique index. E1's `20261115_v1.10.0` still sorts after it, as the G1 date reservation anticipated (GRANT 6865502816: "after `20261112`, `20261113` and `20261114`"), so G6 needs no directory rename for it. That migration's guard, `the_migration_applies_reverts_and_reapplies_changing_no_other_schema_object`, had the same newest-migration assumption as the B1 guard. #1032 (stage #1031) fixed it on `develop`; it merged on 2026-11-14, after E1's base, so it is not on E1's branch. These `develop` facts come from the PR record; `develop` was not fetched for review round 1. E1's changed B1 guard reverts every later migration before it compares, so it does not depend on which later migrations the chain holds; it was run on E1's chain only. Section 14 lists what G6 must re-run.

## 2. Scope confirmation

Approved specification: `ENGINE-01-spec-R5.5.md`, SHA-256 `75e38f4b83697cdb860e4bb01bffc64dc98297127778afa952e47da0fe021baf` (verified), with carry-forward R5.5 part A items 1 to 5, under GRANT 6865502816 (G4) and its amendment 6870622120 (section 0).

Implemented objective: E1, the inert core of the shared job engine, consisting of:

- the migration of section 7 (nine enum types, nine tables, triggers and deferred checks, `down.sql`);
- `schema.rs`;
- the kind registry (section 8, production registry empty);
- the lifecycle and legal transitions (section 9);
- the engine operations (section 10), lock order (11), scheduling (12), and holds and evidence (13);
- the signals (15);
- the error mapping;
- the E1 tests of section 19.2;
- the migration evidence of section 19.0 (section 6.1);
- the B1 guard change the amendment authorizes (section 0);
- the fixes of review round 1 ("Review round 1"), all in E1's own new files.

Out-of-scope changes made: NONE.

## 3. Commits

All are authored and committed by `Javier Arias <javier@jarias.org>` and carry no trailers. This was checked with `git log --format='%h %an <%ae> | %cn <%ce> | %(trailers:only,unfold) | %s' da3e15a3..HEAD`.

- `4ed7ead` - Add the async job engine migration, schema and error variants
- `b6ee45f` - Add the async job engine module
- `513e542` - Add the async job engine database tests
- `cc1dbf3` - Follow the section 13.3 table for read-only completions
- `5115f06` - Add the mapping, carry-forward, time and phase tests
- `1682981` - Add the input bounds tests
- `24b51f8` - Add the database backstop and accepted regression tests
- `b1edd3d` - Add the command and lock concurrency tests
- `ebcbab7` - Add the section 9 addition tests
- `dd38d9b` - Make the B1 revert guard hold under later migrations
- `4e9058c` - Add the migration evidence harness
- `0529a40` - Add the changelog line for the async job engine core
- `80fa214` - Add the THOTH-ASYNC-01-ENGINE-01A implementation report
- `4d3d48f` - Remove the duplicate diesel prelude import from the engine tests
- `fd9aa5b` - Bind one clock reading in the recovery selection
- `663e87c` - Treat U+001C to U+001F as blank in the engine's text checks
- `5ee6451` - Correct the comment on the 24-hour terminal counts
- the commit that rewrites this report for review round 1

Every commit compiled. Each of the four round-1 code commits passed `cargo clippy --all --all-targets --all-features -- -D warnings` at its own state. Every push was a normal push, never forced.

## 4. Files changed

Code and tests (`git diff --numstat da3e15a3..5ee6451`, without this report: 16 files, 20421 insertions, 4 deletions):

| File | Lines | Budget |
|---|---|---|
| `thoth-api/migrations/20261115_v1.10.0/up.sql` (new) | +1225 | new file, authorized |
| `thoth-api/migrations/20261115_v1.10.0/down.sql` (new) | +40 | new file, authorized |
| `thoth-api/src/model/async_job/mod.rs` (new) | +1269 | new file, authorized |
| `thoth-api/src/model/async_job/registry.rs` (new) | +436 | new file, authorized |
| `thoth-api/src/model/async_job/engine.rs` (new) | +2499 | new file, authorized |
| `thoth-api/src/model/async_job/targets.rs` (new) | +473 | new file, authorized |
| `thoth-api/src/model/async_job/commands.rs` (new) | +1178 | new file, authorized |
| `thoth-api/src/model/async_job/signals.rs` (new) | +739 | new file, authorized |
| `thoth-api/src/model/async_job/test_support.rs` (new) | +885 | test-support module, authorized |
| `thoth-api/src/model/async_job/tests.rs` (new) | +11225 | new file, authorized (158 E1 tests and the ignored 19.0 harness) |
| `thoth-api/src/schema.rs` | +243 | new tables, SQL types, `joinable!` and `allow_tables_to_appear_in_same_query!` entries only |
| `thoth-api/src/model/mod.rs` | +2 | `#[cfg(feature = "backend")] pub mod async_job;` only |
| `thoth-errors/src/lib.rs` | +163 | 17 new variants and their GraphQL codes only |
| `thoth-api/src/model/publisher_service_configuration/tests.rs` | +4 | the four `async_job*` names in the 7.11 `%job%` inventory only |
| `thoth-api/src/model/metric_coverage_run/tests.rs` | +39 -4 | the B1 guard test and one helper only, under the amendment (section 0) |
| `CHANGELOG.md` | +1 | one Unreleased "Added" line for issue 1026 |

Review round 1 changed four of E1's own new files (`git diff --numstat 80fa2148..5ee6451`: `engine.rs` +11 -4, `mod.rs` +9 -4, `signals.rs` +9 -2, `tests.rs` +3 -2; 32 insertions and 12 deletions in all) and this report.

The report commits add and rewrite this file (`docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ENGINE-01A-implementation-report.md`, an authorized new file).

The 17 new error codes are:

- `ASYNC_JOB_BUSY`
- `ASYNC_JOB_DUPLICATE_CLAIM`
- `ASYNC_JOB_EFFECT_WINDOW_INSUFFICIENT`
- `ASYNC_JOB_EVIDENCE_INVALID`
- `ASYNC_JOB_IDEMPOTENCY_CONFLICT`
- `ASYNC_JOB_INTEGRITY_VIOLATION`
- `ASYNC_JOB_ISOLATION_INVALID`
- `ASYNC_JOB_KIND_ADMISSION_CLOSED`
- `ASYNC_JOB_KIND_NOT_AUTHORIZED`
- `ASYNC_JOB_KIND_UNKNOWN`
- `ASYNC_JOB_LEASE_EXPIRED`
- `ASYNC_JOB_NOT_FOUND`
- `ASYNC_JOB_PAYLOAD_INVALID`
- `ASYNC_JOB_PAYLOAD_VERSION_UNSUPPORTED`
- `ASYNC_JOB_STALE_CLAIM`
- `ASYNC_JOB_TRANSITION_REFUSED`
- `ASYNC_COMMAND_REQUEST_CONFLICT`

The evidence harness (`evidence_migration_harness` in `async_job/tests.rs`) is `#[ignore]`d. It applies migrations one at a time with `run_next_migration` up to a version, or reverts with `revert_last_migration` until a version has been reverted, and prints each version. It runs only against the database named by `ASYNC_EVIDENCE_DATABASE_URL` and refuses the test database.

Files deleted, moved or renamed: NONE.

### 4.1 Write-budget compliance

PASS, at `5ee6451`:

- `git diff da3e15a3..5ee6451` of every prohibited path is empty. The paths checked were `thoth-api/src/model/distribution_job*`, the four `*_v1.7.0` migrations, `thoth-api/src/graphql`, `thoth-api/src/policy.rs`, `thoth-api-server`, `thoth-export-server`, `thoth-client`, `src`, `.github`, `docs/engineering/decisions`, the `Cargo.toml` files, `Cargo.lock` and `thoth-errors/src/database_errors.rs`. No existing migration changed.
- No added code or test line references `distribution_job` or `publisher_service_configuration`. In `git diff da3e15a3..5ee6451 -- . ':!docs'` the only matching line is the `+++` file header of the 7.11 assertion's file.
- `metric_coverage_run/tests.rs` changes only inside the B1 guard test and its new helper, as the amendment allows.
- Review round 1 changed only E1's own new files and this report.

### 4.2 Authorized actions actually used

- repository inspection: YES, including, through the REST API and read only, the issue #1026 comments (to confirm the amendment before editing the guard), PR #1033's CI comment, and the check runs and `lint` job log at `80fa2148`
- source edit: YES (budget files only)
- new file creation: YES (budget files only)
- file deletion/move/rename: NO (in the repository)
- branch creation: NO (the slice branch was pre-created)
- commit: YES (the 18 commits of section 3)
- push: YES (`git push origin feature/worker--engine-01a`, normal pushes, never forced)
- PR creation/update: NO (a push moves PR #1033's head; nothing else of the PR was touched)
- issue/comment mutation: NO
- manual CI dispatch/rerun: NO
- provider/runtime read: NO
- provider/runtime write: NO
- migration execution: YES, only on disposable local PostgreSQL 17.10 databases: the test database (the suite's own migrations and revert-through tests) and the 19.0 databases `ev19_fresh`, `ev19_prod` and `ev19_pre17` (section 6.1)
- temporary local worktree of the latest release tag (19.0 item 6): YES. The tag's objects were fetched with `git fetch --no-tags origin refs/tags/v1.9.0`, which creates no ref. The detached worktree was built, used and removed during the 19.0 run, and nothing in it was committed or pushed.
- release/tag/publication: NO
- merge: NO
- deployment: NO
- production activation: NO
- other:
  - a local PostgreSQL 17.10 built from the Ubuntu orig tarball (port 5433, throwaway data directory) and a throwaway Redis on 6379;
  - for 19.0 item 6, a local identity-provider stand-in on 127.0.0.1 (section 6.1, item 6);
  - in review round 1: the deletion of the stand-in key's two local files (OPE-5); a scratch copy of `ev19_fresh` for the FBE-1 plan check, dropped afterwards; and a local Rust 1.99.0 toolchain with its own target directory for two extra clippy runs (section 9), both removed afterwards.

Unauthorized actions performed: NONE

### 4.3 Automatic and manual external effects

Automatic CI/provider effects: a push to the slice branch starts no workflow by itself. While PR #1033 is open, a push moves the PR's head, and the PR's CI runs on the new head (section 11). No image is published from a PR, and nothing deploys.

Manually initiated external actions: NONE. External writes/publication: NONE (branch pushes only).

## 5. Implementation decisions

Readings made within the approved design. Each is small and local; the tests named in section 9.1 exercise each one.

1. **Idempotent repeat: content equality.** Equality is by the canonical `jsonb` text. `jsonb` keeps numeric scale, so `1` and `1.0` are different payloads, as they are to a handler that reads the stored text (`creation_payload_equality_is_by_canonical_form`).
2. **Conflict "aborts the caller's transaction".** Both readings are satisfied. The engine returns `ASYNC_JOB_IDEMPOTENCY_CONFLICT` and also leaves the PostgreSQL transaction aborted, so a caller that ignores the error cannot commit its own work (`creation_conflict_aborts_the_callers_transaction`).
3. **Publisher Services comment.** The comment above the 7.11 inventory assertion still says "`BE-04`'s three". The budget allows the assertion only, and 7.11 says "changes nothing else in that file", so the comment is now stale and was left unchanged.
4. **Worker-write refusal reasons.** Worker writes refused by state use `ASYNC_JOB_TRANSITION_REFUSED` with the command refusal names as the reason: `RECONCILIATION_IN_PROGRESS` (an execution write on an RR job), `NOT_RECONCILIATION_REQUIRED` (a reconciliation write on a non-RR job) and `DEADLINE_EXCEEDED`.
5. **T18 with `checkpoint` or `resume_at` on an outcome other than WAITING** is a committed REFUSED with `WAIT_INVALID`.
6. **Code names the spec does not spell out:**
   - command decision outcomes: `RETRY_SCHEDULED`, `RECONCILIATION_REQUIRED`, `OPERATOR_STOP` with cause `T14E_OPERATOR_STOP`, and `RETRY_CREATED`;
   - redaction outcome: `REDACTED`.
7. **`fail` without an `error_code`** is `ASYNC_JOB_PAYLOAD_INVALID`, like a malformed one.
8. **T17 inside T18.** When T18 first applies T17 to an expired reconciliation claim, the deferred attempt check is not forced separately after the T17. It runs once, after T18, and both cases in 19.2 pass it (`commands_staff_resolution_of_unclaimed_and_expired_claims_and_refusal_while_live`).
9. **`resume_at` faults:**
   - in `wait` and in T16 WAITING (outside its bounds), they are `ASYNC_JOB_PAYLOAD_INVALID`, with the claim live;
   - in T18 WAITING, they are a committed REFUSED with `WAIT_INVALID`;
   - a T16 WAITING whose stored waiting deadline has already passed is `ASYNC_JOB_TRANSITION_REFUSED` with reason `DEADLINE_EXCEEDED`.
10. **T9 attempt result.** T9 closes the attempt with the reported class, following the 9.3 text "INDETERMINATE (or the reported class)".
11. **`RETRY_JOB` key collision.** A retry whose child idempotency key is already taken is a committed REFUSED with `RETRY_NOT_ALLOWED`. It is never an idempotent success.
12. **READ_ONLY completions follow the 13.3 table** (`cc1dbf3`). The earlier draft code allowed `NO_OP` and `SUPERSEDED` only for EFFECTFUL jobs ("reading B", from spec line 514 and ADR-0012's effect semantics). The table allows them for READ_ONLY jobs, and the table is now followed. This is recorded for reviewers as a change from the earlier draft code, not as a deviation from the spec.
13. **"None" cells.** Inputs in a "none" cell of the evidence-argument table are refused. This follows the 13.3 table's `WAITING` row ("T16 or T18 WAITING": none and none) and its `STILL_REQUIRED` row ("T16 STILL_REQUIRED": none and none, supplied inputs refused) (`bounds_an_input_in_a_none_cell_of_the_evidence_argument_table_is_refused`).
14. **CANCELLED at EFFECT_CONFIRMED.** A direct-SQL status change to CANCELLED at `EFFECT_CONFIRMED` is refused immediately by the 7.10 rule 4 phase trigger, before commit. The backstop test asserts that immediate refusal for this row of the commit-time sentence. The FAILED rows are refused at commit, as stated.
15. **Backoff.**
    - `backoff_delay_seconds` returns the maximum for a counter of 32 or more. It uses `checked_mul` and cannot overflow for any `i32` base below 32.
    - Jitter is `((hash % 21) + 21) % 21` per cent. It is tested through the same SQL expression with `i64::MIN` and `i64::MAX`, and the test also shows that `abs(i64::MIN)` would raise an error.
16. **Deadlock counter.** The deadlock test (`mapping_a_real_deadlock_is_busy`) is the one concurrency test that must deadlock. It asserts that the counter rises by exactly one; every other concurrency test asserts it unchanged.
17. **Superuser test role.** The tests rely on a superuser test role, as CI's does. They need it for:
    - `session_replication_role = replica` in fixtures that bypass the triggers;
    - setting `deadlock_timeout`;
    - `pg_terminate_backend` and `pg_cancel_backend`.
18. **T15 candidate order.** T15 (`claim_reconciliation_on`) filters its candidates by the requested payload versions, as claim does, and orders them by `available_at, async_job_id`. Spec 7.2 gives the reconciliation scan its own index, `(kind, available_at, async_job_id) WHERE status = 'RECONCILIATION_REQUIRED' AND claim_token IS NULL`. Section 12's within-kind order (`priority DESC, available_at ASC, created_at ASC, async_job_id ASC`) is the ready scan's, served by the ready-scan index. "Mirrors claim" in 10.4 is read as the claim procedure (the reconciler domain, `NS_RECON` kind locks, `reconciliation_concurrency_limit` and the `RECONCILIATION` poll rows), not the ready-scan order. The orchestrator confirmed this reading on 2026-11-14, and the code is unchanged. Both review legs found it sound, and it is recorded as a spec clarification for E2 (section 13).
19. **19.0 item 2's "latest `develop` migration"** is `20261006`, the newest migration in E1's chain and on `develop` at the authorized base. `develop`'s later `20261112` is not in E1's branch (section 1); G6 brings it in.
20. **19.0 item 3's "until `20260813_v1.6.3` is applied"** is taken in version order. The harness therefore also applies the three v1.7.0 migrations dated before it (`20260805`, `20260811`, `20260812`), and `cargo run migrate` then applies `20260814_v1.7.0` (BE-04) first.
21. **19.0 item 6's identity provider.** The v1.9.0 GraphQL server runs OIDC discovery against `ZITADEL_URL` when it starts and introspects each bearer token, and BE-04's report is superuser only. A local stand-in on 127.0.0.1 answered discovery, an empty key set and introspection, with exactly one token introspecting as the unscoped `SUPERUSER` role (section 6.1, item 6). No real identity provider or credential was used. The server signed its JWT client assertion with a 2048-bit RSA key that was generated locally with `openssl genrsa` for this run and given to the server only through its environment. The key was used only with the stand-in on 127.0.0.1, and its two local files were deleted in review round 1.
22. **Recovery's instant (FBE-1).** `recover_rows` reads `now` once, just before its selection, and binds it in both due-time comparisons, so the selection and each row's handling share one instant. Spec 9.3 reads `now` "after the transaction has its locks on the job row (for claims, after the kind locks)", so that a lock wait or a long caller transaction cannot make an expired lease look live. Claims still read it after their kind locks. The selection takes its row locks with `SKIP LOCKED`, which never waits, so no lock wait falls between the reading and the row locks. A lease that expires between the reading and the selection is left for the next pass; it is never handled as live. Before review round 1, the selection compared with `clock_timestamp()` and `now` was read just after it.

Deviations from the specification requiring authorization: NONE. The guard change of section 0 is authorized by the G4 amendment.

## 6. Database and migration effects

Migration added: YES

- migration files: `thoth-api/migrations/20261115_v1.10.0/up.sql` (blob `a78af3d6bee40ff914ffb9355c847380c0b317b4`) and `down.sql` (blob `4ba06ec14e081826d7cd38f43d21f8f5906bcb62`). Both blobs are unchanged at `5ee6451`: review round 1 changed no migration file.
- schema effect:
  - nine enum types and nine tables (`async_job`, `async_job_target`, `async_target_hold`, `async_effect_target_state`, `async_job_attempt`, `async_control_command`, `async_job_transition`, `async_recovery_failure`, `async_executor_poll`);
  - 12 trigger functions with their row triggers and deferred constraint triggers.
  - No function or trigger name contains `reconciliation` or `coverage` or starts with `maintain` (spec 7.11 naming guards: they pass).
- existing-data effect: none. Only `CREATE` statements on new objects. Item 2 shows the existing rows byte-identical across apply and revert.
- locking/downtime: no lock on any existing table; runtime negligible.
- empty database result: PASS (items 1 and 4).
- populated database result: PASS (item 2, with seeded Publisher Services rows).
- rollback/forward repair: `down.sql` drops the nine tables in the 7.11 order, then the functions and the nine enum types, without `CASCADE`. Item 2 shows the schema after the revert byte-identical to the schema before the apply. Once production holds any `async_*` row, rollback is a forward-repair decision (ADR-0012 section 10); the `up.sql` header says so.
- idempotency: PASS (item 5).

### 6.1 Spec 19.0 migration evidence

All six items PASS, on PostgreSQL 17.10 (the local server on port 5433), at code head `0529a403`. Review round 1 changed neither migration file nor the harness function, so the evidence stands for `5ee6451`. The commands quoted in this section reproduce it on any disposable PostgreSQL 17 server.

Method:

- Each item ran on its own new database, created empty with `CREATE DATABASE` (no relation in `public`): `ev19_fresh`, `ev19_prod` and `ev19_pre17`.
- The binary was built with `cargo build` and run with `cargo run migrate`, with `DATABASE_URL` naming the database, as CI's `run-migrations` workflow does (CI adds `-vv` to the build, which changes only its output).
- The harness ran as `cargo test -p thoth-api --features backend --lib -- model::async_job::tests::evidence_migration_harness --exact --ignored --nocapture`, with `ASYNC_EVIDENCE_DATABASE_URL` and `ASYNC_EVIDENCE_STEP` set.
- Schema dumps used PostgreSQL 17.10's `pg_dump --schema-only --restrict-key=ev19`, and data dumps `pg_dump --data-only --restrict-key=ev19`. The fixed `\restrict` key makes two dumps of one database byte-comparable.
- "E1 objects" below means 9 `async_` tables, 9 `async_` enum types, 12 `async_` functions and 16 triggers on `async_` tables.

**Item 1, fresh empty database, CI's sequence** (`ev19_fresh`):

```text
cargo build                    exit 0
cargo run migrate              exit 0   30 applied, newest 20261115, E1 objects present
cargo run migrate --revert     exit 0   0 applied; public holds only __diesel_schema_migrations
cargo run migrate              exit 0   30 applied, newest 20261115, E1 objects present
```

- The 30 versions, in order: `20250000 20260417 20260429 20260504 20260805 20260811 20260812 20260813 20260814 20260826 20260827 20260828 20260831 20260901 20260902 20260903 20260904 20260905 20260906 20260907 20260908 20260909 20260912 20260913 20260921 20260922 20260923 20260928 20261006 20261115`. The full chain includes `20260814_v1.7.0`, and E1's migration is last.
- After the revert, `public` held no other table, no enum, domain or composite type and no function.
- The schema dump after the second migrate is byte-identical to the dump after the first (sha256 `a8d4a706db373a501462d680e7b6a98d8c588ffcf387db9da161780d07c391d2`, 10096 lines).

**Item 4, whole-chain revert and migrate** (`ev19_fresh`): `cargo run migrate --revert` reverted all 30 migrations and `cargo run migrate` reapplied them, both with exit 0 (the third and fourth lines of item 1).

**Item 5, a second migrate is a no-op** (`ev19_fresh`): one more `cargo run migrate` exited 0. All 30 `__diesel_schema_migrations` rows (version and `run_on`) and the schema dump are unchanged.

**Item 2, production-like database** (`ev19_prod`):

1. The harness `apply-through:20261006` applied the 29 migrations of E1's base chain one at a time with `run_next_migration`, `20250000` to `20261006`, so v1.7.0 (`20260814`, BE-04) is applied (reading 19).
2. Seeded Publisher Services rows: 1 publisher, 1 enabled `INTERNET_ARCHIVE` assignment, and 1 PENDING `PUBLISHER_BACK_CATALOGUE` distribution job with its target.
3. Dump A: schema 8642 lines, sha256 `69d096c664853312943775db350134604d7671eb3f904ef80bf057a505405488`, and a data dump of 807 lines.
4. `cargo run migrate` applied E1's migration only: exit 0, 30 applied, newest `20261115`, E1 objects present. Against dump A, the applied schema dump only adds lines: `diff` shows 1454 added lines and none removed or changed.
5. The harness `revert-through:20261115` reverted with `revert_last_migration` until this version was reverted. It reverted `20261115` only. Afterwards 29 were applied, and no `async_` table, type, function or trigger was left.
6. Dump B is byte-identical to dump A, and data dump B is byte-identical to data dump A.
7. `cargo run migrate` reapplied E1's migration: exit 0. The schema dump is byte-identical to the first apply's (`a8d4a706...`).

**Item 3, pre-v1.7 path** (`ev19_pre17`):

- The harness `apply-through:20260813` applied, one at a time: `20250000 20260417 20260429 20260504 20260805 20260811 20260812 20260813` (reading 20).
- `cargo run migrate` then applied the 22 pending migrations, exit 0. In `run_on` order: `20260814` (BE-04's v1.7.0) first, then `20260826` to `20261006`, and `20261115` last. The `run_on` values strictly increase in version order.
- The final schema dump is byte-identical to items 1 and 2 (`a8d4a706...`), so all three paths end in the same schema.

**Item 6, previous-image compatibility** (the v1.9.0 binary against `ev19_prod` after step 7 of item 2: E1 applied, seeded rows present):

- Release: `v1.9.0`, GitHub's latest release (published 2026-11-07), tag object `132956aabd275969e8d0c74d0559b5e9856f3e78`, commit `f348e90b42f69413c7a2381b40a09811aba924db`.
- Build:
  - `git fetch --no-tags origin refs/tags/v1.9.0`, which stores no ref;
  - `git worktree add --detach <temporary path outside the checkout> f348e90b`;
  - `cargo build` there with `CARGO_TARGET_DIR` set to E1's target directory (exit 0, 116 s). `Cargo.lock` and every `Cargo.toml` at the tag are identical to the head's.
  - The binary reports `thoth 1.9.0`.
- v1.9.0 `migrate`: exit 0. `__diesel_schema_migrations` is unchanged (30 rows; the newest, `20261115`, is unknown to v1.9.0), and so is the schema dump. Nothing was pending: v1.9.0's 29 migrations are all applied.
- v1.9.0 `start graphql-api` on 127.0.0.1, with the identity stand-in of reading 21, served three requests:
  - BE-04's staff report, with the superuser token, HTTP 200:

    ```json
    {"data":{"publisherServiceConfigurations":[{"configuration":{"publisher":{"publisherId":"e1900000-0000-4000-8000-000000000001","publisherName":"EV19 Evidence Press"},"subscriptionPackage":"OBELISK","enabledDistributionPlatforms":[{"platform":"INTERNET_ARCHIVE","enabledAt":"2026-11-14T00:00:00Z"}]},"latestBackCatalogueJob":{"distributionJobId":"e1900000-0000-4000-8000-0000000000b1","kind":"PUBLISHER_BACK_CATALOGUE","status":"PENDING","attemptCount":0,"availableAt":"2026-11-14T00:00:00Z","targets":[{"platform":"INTERNET_ARCHIVE"}],"attempts":[]}}],"publisherServiceConfigurationCount":1}}
    ```

    The request was `publisherServiceConfigurations(jobStatuses: [PENDING])` and `publisherServiceConfigurationCount(jobStatuses: [PENDING])`, with `latestBackCatalogueJob`, its targets and its attempts.
  - The same request without a token: `Unauthorized` (`NO_ACCESS`), since the report is superuser only.
  - BE-02's public `publishersByDistributionPlatform(platform: INTERNET_ARCHIVE)` and its count: the seeded publisher, count 1.
- The stand-in received the discovery request, the key-set request and one introspection carrying a JWT client assertion. The server signed that assertion with a 2048-bit RSA key generated locally with `openssl genrsa` for this run and given to it only through its environment. The key was used only with the stand-in on 127.0.0.1, and its two local files were deleted in review round 1. The server log has no `ERROR`, `WARN` or panic line.
- Cleanup: `git worktree remove` and `git worktree prune` removed the worktree, and `git worktree list` no longer shows it. No local tag ref exists, the copied binary was deleted, and nothing from the worktree was committed or pushed.

## 7. API and compatibility effects

GraphQL/API changes: none in E1 (the API is E2). `git diff da3e15a3..HEAD -- thoth-api/src/graphql thoth-client` is empty, so the SDL is unchanged and has no diff to show. The internal `thoth-client` builds in the workspace clippy and check runs (section 9).
Generated schema/client updates: none.
Backwards compatibility: additive. Item 6 of section 6.1 shows that the released v1.9.0 binary migrates nothing and serves BE-04's report against an E1-migrated database.
Deprecations: none.
Cross-repository dependencies: none for E1. Of spec 18's table, E1 affects only the database/domain model (additive tables). The GraphQL, generated-client, authorization and job-payload rows are E2's. Every known consumer remains compatible, because E1 changes no API.

## 8. Authorization and security

Authorization paths changed: none. Engine functions take an executor identity or a staff actor; E2 binds them to roles.

Inertness (S1): the production registry is empty (`registry_production_is_empty_and_valid`), and no production code path calls the engine.

Secret or personal-data handling:

- credential-named keys are refused in payloads and checkpoints;
- command rows store digests only;
- staff redaction erases provider references, checkpoints and diagnostics, and the tests check every stored diagnostics column;
- no read returns a claim token.

Security limitations: error mapping assumes English server messages (spec 23). The mapping tests fail loudly otherwise.

## 9. Tests and checks

Environment:

- PostgreSQL 17.10 built locally, on port 5433, with a throwaway data directory.
- Redis 7 on 127.0.0.1:6379.
- `TEST_DATABASE_URL`, `TEST_REDIS_URL`, `THOTH_GRAPHQL_API` and `THOTH_EXPORT_API` as in CI's `build_test_and_check.yml`. The superuser role is used, as in CI.
- Rust 1.97.0. Builds after the HOLD used `CARGO_INCREMENTAL=0` and E1's one target directory. CI's runner uses a newer stable toolchain (section 11); two extra clippy runs in review round 1 used Rust 1.99.0 with its own target directory.

### Review round 1 checks (at `5ee6451`)

```text
cargo fmt --all -- --check
```

```text
exit 0
```

```text
cargo clippy --all --all-targets --all-features -- -D warnings
```

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 50s
exit 0
```

```text
cargo check --workspace
```

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 35s
exit 0
```

```text
cargo test -p thoth-api --features backend --lib -- model::async_job model::metric_coverage_run::tests::reverting_through_the_b1_migration_removes_exactly_its_objects_and_reapplication_restores_them
```

```text
test result: ok. 159 passed; 0 failed; 1 ignored; 0 measured; 2335 filtered out; finished in 20.50s
exit 0
```

- The 159 passes are the 158 E1 tests and the B1 guard; the ignored test is the 19.0 harness. This ran on PostgreSQL 17.10 with Redis running.
- Each round-1 code commit (`4d3d48f`, `fd9aa5b`, `663e87c`, `5ee6451`) passed the clippy command at its own state.
- The same clippy command with Rust 1.99.0 (`cargo +1.99.0 clippy ...`) fails at `80fa2148` exactly as CI's `lint` did (`error: unused import: diesel::prelude::*` at `thoth-api/src/model/async_job/tests.rs:23:5`, then `could not compile thoth-api (lib test) due to 1 previous error`, exit 101), and passes at `5ee6451` (exit 0, 4m 15s). The toolchain and its target directory were removed afterwards.
- `git diff --check da3e15a3..5ee6451`: no output, exit 0.
- The full suite was not rerun in review round 1; PR CI runs it. The full-suite result below is at `0529a403`, and review round 1 changed only E1's own files.

The only warning in the clippy and check runs is cargo's future-incompatibility note for the dependency `proc-macro-error2 v2.0.1`, which is pre-existing and not a lint.

### Formatting

```text
cargo fmt --all -- --check
```

```text
exit 0 (at 0529a403)
```

### Unit and database tests (`thoth-api`)

```text
cargo test -p thoth-api --features backend
```

```text
test result: ok. 2491 passed; 0 failed; 4 ignored; 0 measured; 0 filtered out; finished in 802.24s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.22s
test result: ok. 0 passed; 0 failed; 8 ignored; 0 measured; 0 filtered out; finished in 0.00s
exit 0
```

- This ran at `0529a403` on PostgreSQL 17.10: the library tests, `tests/graphql_permissions.rs` and the doc tests, in that order.
- The library has 2495 tests: the baseline's 2336, E1's 158 and the ignored 19.0 harness. The 4 ignored tests are the baseline's 3 and the harness.
- All 158 `model::async_job::tests` pass. The B1 guard and `the_migration_backfills_nothing_over_populated_raw_evidence` pass, and so does every other existing test.
- `graphql_permissions` (13 passed) and the doc tests (8 ignored) match the baseline.
- Before the amendment, at `ebcbab7`, the library tests ended `FAILED. 2490 passed; 1 failed; 3 ignored` on the B1 guard (section 0), and cargo stopped before the other targets.

### Lint/static analysis

```text
cargo clippy --all --all-targets --all-features -- -D warnings
```

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 2m 58s
exit 0 (at 0529a403)
```

```text
cargo check --workspace
```

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 2m 30s
exit 0 (at 0529a403)
```

Rust 1.97.0 accepted the duplicate glob import that CI's newer toolchain rejects (section 11).

### Workspace tests

```text
cargo test --workspace --exclude thoth-api --no-fail-fast
```

```text
thoth (lib)            test result: ok. 0 passed; 0 failed; 0 ignored
thoth (bin)            test result: ok. 40 passed; 0 failed; 0 ignored
thoth-api-server       test result: ok. 5 passed; 0 failed; 0 ignored
thoth-client           test result: ok. 4 passed; 0 failed; 0 ignored
thoth-errors           test result: ok. 15 passed; 0 failed; 0 ignored
thoth-export-server    test result: ok. 144 passed; 0 failed; 0 ignored
Doc-tests              thoth 0, thoth_api_server 0, thoth_client 6 passed, thoth_errors 0, thoth_export_server 2 passed
exit 0
```

This ran at `ebcbab7` and was not repeated. The later commits change `thoth-api` test code, two functions (`recover_rows` and `is_non_blank`) and some comments in `thoth-api`'s `async_job` module, which no other crate calls, the changelog and this report. The workspace clippy and check runs at `5ee6451` compile every crate's targets. The `thoth-api` part of `cargo test --workspace` is the full suite above: workspace feature unification gives `thoth-api` its `backend` feature, so it is the same test set.

Disk: the workspace selection unifies features differently from `-p thoth-api --features backend`, so it builds a second copy of the dependency tree. Its first two attempts filled the disk. They passed after the clippy and check caches were deleted.

### Other required checks

- `git diff --check da3e15a3..5ee6451`: no output, exit 0.
- `git status --short` before the report commit: only this report, modified.
- Advisory namespaces: at `5ee6451`, `git grep -n -E "109598139[0-3]"` finds `1095981390`, `1095981391` and `1095981392` in code only in `thoth-api/src/model/async_job/mod.rs` (3 lines). `1095981393` appears in no code.
- Registry: the production registry is empty, and its validation runs on every test registry (`registry_production_is_empty_and_valid`, `registry_every_test_registry_validates`).

### 9.1 Spec 19.2 to tests (E1)

158 tests in `thoth-api/src/model/async_job/tests.rs`; all pass. The file has 159 `#[test]` functions: the 158 E1 tests mapped below and the ignored 19.0 evidence harness (`evidence_migration_harness`), which is not a 19.2 test. Test names start with their 19.2 group. Counts are tests named for that paragraph; where a sentence is also covered by a test in another group, that test is listed in brackets.

| 19.2 paragraph | Tests | Count |
|---|---|---|
| Schema and triggers | `schema_async_job_checks_accept_their_boundary_and_refuse_one_past_it`, `schema_other_table_checks_accept_their_boundary_and_refuse_one_past_it` (check boundaries); `schema_every_illegal_status_pair_is_refused_exhaustive_matrix` (8x8); `schema_pending_and_retry_scheduled_require_pre_write`; `schema_terminal_rows_are_frozen_except_one_redaction`; `schema_identity_and_snapshot_columns_are_frozen`; `schema_effect_phase_counters_and_write_once_columns_never_go_back`; `schema_insert_guard_requires_a_fresh_pending_row`; `schema_attempt_immutability_and_redaction_rules`; `schema_command_rows_are_append_only_and_decided_before_commit`; `schema_s5_deferred_hold_check_refuses_direct_sql`; `schema_transition_log_names_every_engine_cause`; `schema_one_open_attempt_per_job`; `schema_target_and_hold_foreign_keys`; `schema_one_retry_child_per_job` | 15 |
| Creation | `creation_idempotent_repeat_returns_existing_in_any_state`; `creation_payload_equality_is_by_canonical_form`; `creation_conflict_aborts_the_callers_transaction`; `creation_key_collision_is_never_success_for_a_newer_revision_or_slot`; `creation_refuses_every_invalid_input_and_writes_nothing` (every listed refusal); `creation_inside_a_caller_transaction_that_rolls_back_leaves_nothing`; `creation_refuses_callers_outside_read_committed`; `creation_identifier_format_faults_are_payload_invalid` | 8 |
| Claims | `claims_four_connections_with_overlapping_kinds_never_claim_twice_and_keep_limits`; `claims_order_by_priority_within_a_kind`; `claims_respect_available_at`; `claims_cross_kind_fairness_does_not_starve_an_older_job`; `claims_skip_unsupported_and_non_executable_versions_and_leave_them_claimable`; `claims_report_unknown_kinds_and_still_claim_known_ones`; `claims_filter_by_risk_capability_revision_mode_and_domain` (including the executor-domain refusal); `claims_whose_candidates_cross_targets_finish_without_waiting_or_deadlock`; `claims_recovering_an_expired_holder_while_another_wants_its_key_finish_without_deadlock`; `claims_skip_a_key_whose_hold_lock_is_taken_without_entering_a_lock_wait` (short `statement_timeout` and `pg_locks`); `claims_report_a_blocked_job_with_its_holder` | 11 |
| Leases and tokens | `leases_renew_needs_the_current_token_and_a_live_lease_and_stops_at_the_runtime_deadline`; `leases_the_lease_rule_of_each_operation`; `leases_stale_tokens_are_refused_by_every_token_fenced_write`; `leases_a_superseded_worker_cannot_write_after_recovery_or_cancellation`; `leases_recovery_racing_a_late_completion_exactly_one_wins`; `leases_one_token_one_attempt`; `leases_no_read_returns_the_token` | 7 |
| Effect phases and outcomes | `effect_window_is_refused_on_the_lease_and_on_the_runtime_deadline`; `effect_crash_before_effect_started_counts_as_a_pre_write_failure_then_retries_or_fails`; `effect_crash_after_effect_started_goes_to_reconciliation`; `effect_confirmed_is_resumable_with_a_checkpoint_and_otherwise_goes_to_reconciliation`; `effect_waiting_atomically_checkpoints_and_releases_the_claim`; `effect_resume_is_a_new_attempt_that_sees_the_checkpoint_and_consumes_no_budget_or_cap`; `effect_wait_needs_the_minimum_resume_delay`; `effect_deadline_exhaustion_goes_to_rr_after_start_and_to_failed_before_it` (including `wait` after the deadline); `effect_failure_after_start_goes_to_reconciliation_whatever_the_class` (INDETERMINATE to RR); `effect_proven_no_effect_after_start_fails_only_where_the_kind_allows_with_valid_evidence`; `effect_release_loop_is_bounded_by_the_execution_claim_cap`; `effect_reconciliation_loop_is_bounded_by_its_cap_then_only_staff_resolves`; `effect_backoff_values_match_the_formula_for_both_counters` | 13 |
| Targets and evidence | `targets_two_kinds_sharing_a_target_never_overlap_in_running_waiting_or_rr`; `targets_holds_are_kept_through_waiting_and_reconciliation`; `targets_holds_are_released_at_each_listed_point`; `targets_current_state_rows_of_the_evidence_table`, `targets_revision_bound_rows_of_the_evidence_table`, `targets_key_only_rows_and_read_only_completion`, `targets_a_read_only_revision_bound_job_completes_superseded_or_completed_with_key_only_entries` (every 13.3 row, positive and negative, at PRE_WRITE and after effect start; [`bounds_every_cell_of_the_evidence_fault_table`]); `targets_an_older_retried_revision_cannot_overwrite_a_newer_one`; `targets_evidence_advances_only_with_the_current_token_under_the_hold`; `targets_a_provider_without_revision_ids_still_blocks_regression_through_local_order`; `targets_identifier_format_faults_in_entries_are_evidence_invalid_at_every_phase`; `targets_normal_completion_racing_a_reconciler_one_wins_and_the_loser_is_stale` | 12 |
| Cancellation, retry and reconciliation | `commands_cancel_from_each_state_per_the_table`; `commands_cancel_racing_effect_start_in_both_orders`; `commands_rr_cannot_be_cancelled_or_retried`; `commands_manual_retry_rules` (FAILED or CANCELLED at PRE_WRITE or after proven no-effect, kind policy, once per job, child runs normally); `commands_reconciliation_claim_and_resolution_are_token_fenced_and_recorded_as_attempts`; `commands_reconciliation_lease_expiry_and_backoff`; `commands_staff_resolution_of_unclaimed_and_expired_claims_and_refusal_while_live` (including T18 FAILED at EFFECT_CONFIRMED and T18 CANCELLED at EFFECT_STARTED after their own T17, then `RETRY_JOB` on the same latest attempt); `commands_staff_resolution_is_atomic_with_its_command_row`; `commands_idempotency_on_repeated_and_conflicting_request_ids`; `commands_against_a_missing_job_write_nothing` | 10 |
| Redaction | `redaction_erases_finished_attempts_and_terminal_diagnostics_and_leaves_an_open_attempt`; `redaction_repeated_is_applied_and_changes_no_row`; `redaction_command_rows_contain_digests_only`; `redaction_leaves_no_redacted_value_in_any_stored_diagnostics_column`; `redaction_identifier_columns_outside_redaction_refuse_spaces_and_257_characters` | 5 |
| Recovery and retirement | `recovery_expired_leases_and_deadlines_are_recovered_by_claims_and_by_recover_on`; `recovery_a_row_whose_recovery_fails_is_skipped_and_counted_without_aborting_the_claim`; `recovery_rows_of_a_kind_unknown_to_the_registry_recover_from_their_snapshot`; `recovery_an_admission_closed_kind_still_drains`; `recovery_a_version_removed_from_creatable_versions_but_still_executable_is_claimed`; `recovery_unknown_kind_and_non_executable_version_signals_count_premature_removals` | 6 |
| Signals (each signal set and cleared) | `signals_job_state_counts_set_and_clear`; `signals_blocked_jobs_set_and_clear`; `signals_ready_jobs_whose_version_no_executor_polled_set_and_clear`; `signals_accepted_regressions_set_and_clear`; `signals_terminal_counts_and_attempt_durations_set_and_clear`; `signals_executor_activity_set_and_clear`; [`effect_reconciliation_loop_is_bounded_by_its_cap_then_only_staff_resolves` for `RECONCILIATION_EXHAUSTED`; `recovery_a_row_whose_recovery_fails_is_skipped_and_counted_without_aborting_the_claim` for recovery failures; `recovery_unknown_kind_and_non_executable_version_signals_count_premature_removals` for the unknown-kind and non-executable-version signals] | 6 |
| Database backstops | `backstop_a_direct_sql_insert_of_a_non_pending_or_claimed_row_is_refused`; `backstop_each_phase_rule_of_rule_4_refuses_its_direct_sql_status_change`; `backstop_an_update_of_a_hold_or_target_row_is_refused`; `backstop_a_commit_leaving_a_failed_or_cancelled_job_without_its_closing_attempt_is_refused` (reading 14); `backstop_the_state_trigger_ignores_writer_supplied_regressed_values`; `backstop_t18_succeeded_clears_the_regressed_columns_only_above_the_pre_acceptance_order` (null and set `regressed_*`, forward and equal rewrites); `backstop_the_deferred_s5_and_attempt_checks_refuse_a_commit_that_breaks_them`; `backstop_the_state_trigger_refuses_a_non_holder_a_null_order_and_a_backwards_order` | 8 |
| Deadlock freedom | 20 tests assert the `pg_stat_database.deadlocks` counter. These are every test that runs real concurrent sessions or holds locks across connections: the four racing `claims_` tests, `leases_recovery_racing_a_late_completion_exactly_one_wins`, `commands_cancel_racing_effect_start_in_both_orders`, `targets_normal_completion_racing_a_reconciler_one_wins_and_the_loser_is_stale`, `time_a_lease_that_expires_while_a_call_waits_for_the_row_lock_is_expired`, the five `locks_` tests and the five BUSY `mapping_` tests. Also asserting it: `recovery_a_row_whose_recovery_fails_is_skipped_and_counted_without_aborting_the_claim` and `carry_a_backend_terminated_before_the_wrappers_commit_is_busy`. The deadlock test asserts exactly +1 (reading 16). For the BUSY and INTEGRITY mapping, see "Mapping" | (shared) |
| Time | `time_a_lease_that_expires_while_a_call_waits_for_the_row_lock_is_expired` (including the effect-window instant) | 1 |
| Input bounds | `bounds_u0000_in_every_input_is_refused_with_its_code_and_leaves_the_callers_transaction_usable`; `bounds_a_value_jsonb_refuses_is_refused_with_its_code_and_leaves_the_callers_transaction_usable`; `bounds_an_oversized_checkpoint_and_a_malformed_error_code_write_nothing_and_leave_the_claim_live`; `bounds_every_cell_of_the_evidence_fault_table`; `bounds_fail_with_no_effect_proven_is_refused_with_its_reason_and_the_claim_live`; `bounds_complete_with_an_outcome_its_phase_or_kind_does_not_allow_is_refused_with_the_claim_live`; `bounds_a_call_with_several_refusal_reasons_reports_the_first_in_the_stated_order`; `bounds_an_entry_missing_or_carrying_a_field_is_a_shape_fault_with_the_claim_live`; `bounds_an_input_in_a_none_cell_of_the_evidence_argument_table_is_refused`; `bounds_complete_after_effect_start_with_a_shape_and_a_semantic_fault_keeps_the_claim_live`; `bounds_an_omitted_required_evidence_object_after_effect_start_keeps_the_claim_live`; `bounds_an_unparseable_or_jsonb_refused_value_in_a_staff_command_is_payload_invalid_with_no_command_row`; `bounds_an_out_of_range_order_is_a_shape_fault`; `bounds_a_credential_named_key_in_a_commands_checkpoint_is_a_committed_wait_invalid`; `bounds_a_blank_reason_is_payload_invalid_before_any_write` (its blank samples include U+001C to U+001F since review round 1); `bounds_no_engine_path_surfaces_a_column_check_as_an_integrity_violation` | 16 |
| Phase rules | `phase_fail_indeterminate_at_pre_write_is_a_retryable_pre_write_failure`; `phase_t9n_is_refused_at_effect_confirmed`; `phase_wait_with_resume_at_after_the_waiting_deadline_is_refused` | 3 |
| Accepted regression (E1 part) | `regression_a_t6x_job_accepted_with_valid_evidence_succeeds_and_records_the_regression`; `regression_a_forward_write_below_the_pre_acceptance_order_keeps_it_and_one_above_or_equal_clears_it`; `regression_an_executor_reconcilers_regressing_succeeded_is_refused` | 3 |
| Commands and locks | `locks_two_different_commands_on_one_job_run_concurrently_in_both_orders`; `locks_two_retries_with_different_request_ids_run_concurrently_in_both_orders`; `locks_a_claim_and_a_reconciliation_claim_by_one_executor_identity_run_concurrently_in_both_orders`; `locks_recovery_with_limit_plus_one_failing_rows_still_recovers_a_newly_expired_lease`; `locks_a_row_made_inconsistent_behind_the_triggers_is_skipped_by_recovery_and_claims` (all assert the deadlock counter) | 5 |
| Section 9 additions | `s9_t3_renews_a_reconciliation_claim`; [`effect_deadline_exhaustion_goes_to_rr_after_start_and_to_failed_before_it` for T10d]; `s9_t16_refuses_failed_and_cancelled_at_effect_confirmed_with_the_job_rr_and_the_claim_live`; `s9_t18_at_effect_confirmed_refuses_cancelled_and_closes_failed_only_with_valid_not_applied_evidence`; `s9_a_polling_kinds_confirmed_submission_rejected_later_is_closed_by_a_staff_failed`; `s9_t16_and_t18_waiting_with_resume_at_after_the_waiting_deadline_are_refused`; `s9_t18_succeeded_with_invalid_evidence_is_a_committed_refused_evidence_invalid`; `s9_worker_writes_and_commands_on_a_kind_unknown_to_the_registry_are_refused_and_redaction_still_applies`; `s9_backoff_saturates_at_each_caps_maximum_counter_and_the_jitter_is_in_range_for_the_minimum_bigint_hash`; `s9_retry_job_after_a_proven_no_effect_failed_job_takes_its_policy_snapshot_from_the_current_registry`; `s9_accept_regression_is_accepted_after_every_route_into_reconciliation` (T6x; T12, T9 and T14e at EFFECT_CONFIRMED; T10 then T11; T6x, WAITING, T12; equal order with another fingerprint); `s9_accept_regression_is_refused_without_a_regression_or_valid_applied_evidence`; `s9_the_state_trigger_refuses_a_backwards_or_same_order_write_by_a_reconciliation_attempt`; `s9_a_pre_write_job_facing_an_equal_order_with_another_fingerprint_can_only_fail_permanent`; [`targets_key_only_rows_and_read_only_completion`, `targets_a_read_only_revision_bound_job_completes_superseded_or_completed_with_key_only_entries` for key-only SUPERSEDED and COMPLETED]; [`registry_refuses_a_multi_target_revision_bound_kind`, `registry_refuses_an_effectful_kind_without_the_required_staff_outcomes`] | 13 |
| Mapping | `mapping_a_real_lock_timeout_is_busy`; `mapping_a_real_statement_timeout_is_busy`; `mapping_a_real_deadlock_is_busy`; `mapping_a_query_cancel_is_busy`; `mapping_a_terminated_backend_is_busy`; `mapping_another_database_error_is_an_integrity_violation_with_its_fixed_message` (fails loudly on non-English messages) | 6 |
| Registry (section 8; acceptance criterion) | `registry_production_is_empty_and_valid`; `registry_every_test_registry_validates`; `registry_refuses_a_multi_target_revision_bound_kind`; `registry_refuses_an_effectful_kind_without_the_required_staff_outcomes`; `registry_refuses_cross_kind_namespace_and_duplicate_kind_conflicts`; `registry_refuses_every_bound_of_section_8` | 6 |
| Carry-forward part A | see 9.2 | 4 |
| Regression | The full `thoth-api` suite passes. The only existing test changed is the B1 guard, under the amendment (section 0), besides the 7.11 inventory assertion | - |

Total: 15 + 8 + 11 + 7 + 13 + 12 + 10 + 5 + 6 + 6 + 8 + 1 + 16 + 3 + 3 + 5 + 13 + 6 + 6 + 4 = 158, all E1 tests of 19.2 (the registry and carry-forward tests included). The E2 paragraphs (API and authorization, access log, API additions) are not part of E1.

### 9.2 Carry-forward part A

| Item | Test |
|---|---|
| 1. A failed send (`UnableToSendCommand`) at a wrapper's commit is `ASYNC_JOB_BUSY` | `carry_a_failed_send_or_closed_connection_at_commit_is_busy` |
| 2. What each caller does after a BUSY with an unknown outcome (no new executor read) | `carry_after_an_unknown_outcome_each_caller_retries_safely` |
| 3. The FATAL "terminating connection due to administrator command" maps to BUSY | `mapping_a_terminated_backend_is_busy`, `carry_a_failed_send_or_closed_connection_at_commit_is_busy` |
| 4. The backend terminated between the engine call and the wrapper's commit | `carry_a_backend_terminated_before_the_wrappers_commit_is_busy` |
| 5. T16 and T18 SUCCEEDED entries use the `EFFECT_APPLIED` row, so `sourceRef` is allowed | `carry_t16_and_t18_succeeded_entries_may_carry_a_source_ref` |

Item 1's test feeds synthetic Diesel errors to the commit mapper: `UnableToSendCommand`, `ClosedConnection`, the administrator-termination text, a lock timeout, a statement timeout, a deadlock, a query cancel and a `RollbackErrorOnCommit` that wraps a failed send, with a check violation as the counter-case. It does not lose a real connection. A real connection loss at commit is covered by item 4's test, which terminates the backend after the engine call and before the wrapper's commit.

### 9.3 Concurrency-test method

Every concurrent test uses real, separate PostgreSQL connections, each on its own thread (`Session`, `spawn_tx`, `run_together`), and never sleeps. The tests are ordered by:

- a `std::sync::Barrier`;
- transactions held open by the controlling thread;
- advisory-lock and row-lock pauses;
- bounded polling of `pg_locks` until a named backend is waiting (`wait_until_blocked`, 60 s bound, failing loudly).

Time moves through fixtures that rewrite a job's lease, availability or deadline. Each test reads `pg_stat_database.deadlocks` for the test database before and after. Every test of the file shares the test-database lock with the rest of the suite.

## 10. Manual verification

Environment: local only.
Steps: the spec 19.0 runs of section 6.1, including the v1.9.0 server run of item 6; in review round 1, the FBE-1 plan check ("Review round 1").
Observed result: as recorded in section 6.1 and in "Review round 1".
Evidence: the commands and results quoted there, which reproduce it.

## 11. CI

At `80fa214863a615dd6dab2800445ef482235ba1be`, PR #1033's first CI run (from the commit's check runs and the `lint` job log):

- `lint`: failure. `cargo clippy --all --all-targets --all-features -- -D warnings` ran on the runner's stable toolchain, which is newer than the local 1.97 (the job log does not print its version). It reported `error: unused import: diesel::prelude::*` at `thoth-api/src/model/async_job/tests.rs:23:5` and stopped with `could not compile thoth-api (lib test) due to 1 previous error`. The parent module imports the prelude (`async_job/mod.rs:34`), and the test module takes it through `use super::*`. `4d3d48f` deletes the line. No other `async_job` module has the same pattern: `tests.rs` is the only one that imports `super::*`, and the other child modules name what they take from the parent, so the prelude imports they have are their only source of it. Rust 1.99.0 reproduces the failure at `80fa2148` and passes at `5ee6451` (section 9).
- Passed: `format_check`, `build`, `test`, `run_migrations`, `build_and_push_staging_docker_image` (a build check; a PR publishes no image), `check-changelog` and three `classify` jobs.

PR CI runs again on the new head; GitHub records its result.

## 12. Rollout and rollback

Initial state after merge: inert (S1). No registered kind and no caller.
Activation required: none for E1. Kinds and callers come with later slices.
Feature flag/configuration: none.
Migration sequence: `20261115_v1.10.0` after `20261006_v1.9.0` (and after `develop`'s `20261112_v1.10.0` once G6 integrates).
Rollback/disable procedure: before any `async_*` row exists, `down.sql`; after that, forward repair (ADR-0012 section 10). Item 6 of section 6.1 shows the released v1.9.0 binary migrating nothing and serving reads against the migrated database, which is the basis for rolling the image back without reverting the migration (ADR-0012 section 11).
Monitoring required: none until a kind is registered; then the section 15 signals.

## 13. Known limitations and deferred work

The E1-relevant items of spec section 23 stand unchanged:

- polling-only resumes start at EFFECT_STARTED;
- multi-target starvation (visible in the blocked signal);
- one target for REVISION_BOUND kinds;
- accepted regressions are repaired only by a newer revision;
- no generic exit for a provider state that matches neither revision nor forward write;
- a confirmed effect later rejected is closed only by staff, and its identity cannot be resubmitted;
- large redactions are one update;
- `async_executor_poll` is never pruned in Release N;
- English server messages.

From review round 1:

- **24-hour terminal counts (OPE-3).** `kind_signals` counts a terminal job when both its `updated_at` and its `terminal_at` fall in the last 24 hours. `updated_at` is the start of the transaction that last changed the row, and `terminal_at` is read later in the terminal transaction. So a job whose terminal transaction started before the window's edge and committed after it can be left out of the 24-hour count. A redaction whose transaction began before the job's `terminal_at` can leave it out in the same way. The undercount is at the edge of the window only, bounded by that transaction's length. The `updated_at` bound stays because it lets the `(status, updated_at)` index bound the scan.

E2 carry items, recorded in review round 1:

- **Signal bounds (FBE-2).** `kind_signals` lists kinds with `SELECT DISTINCT kind FROM public.async_job`, a sequential scan and hash aggregate over a table that E1 never prunes, and it returns `regressed_targets` without a bound. E2, the first caller of the signals, should take the kinds from the per-kind aggregates over indexed status windows (or a loose index scan over `async_job_kind_status_idx`) and cap `regressed_targets` with a documented limit.
- **Error classification (FBE-5).** `json_measure` turns every non-busy database error in its savepoint into "not acceptable JSON", so an unrelated database fault there would be reported as a caller input error; only a data exception should be a refusal. `with_engine_transaction_on`, called on a connection that is already in a transaction, maps diesel's `AlreadyInTransaction` to `ASYNC_JOB_INTEGRITY_VIOLATION`; that fault is a caller bug, so it should at least log a distinct fixed message. Neither path is reachable in E1, which has no production caller.
- **REVISION_BOUND kinds without targets (FBE-6).** The per-target evidence checks are vacuous for a job with no targets, so a REVISION_BOUND READ_ONLY job with none could finish SUPERSEDED or NO_OP naming nothing. Spec 8 allows kinds without targets. When the first REVISION_BOUND kind is registered, registry validation should require at least one target for it, or the spec should state that the vacuous case is intended.
- **Spec clarification (FBE-3).** Reading 18, T15's candidate order `available_at, async_job_id`, belongs in the specification's next clarification list, so that E2 and later readers do not take it for a defect.

## 14. Unresolved issues

- NONE for E1's scope. The HOLD of section 0 was released by the G4 amendment, the T15 order is a confirmed reading (section 5, reading 18), and every review round 1 finding is fixed, recorded or carried to E2 ("Review round 1").
- For G6 (outside this slice, OPE-1): integrating E1 with `develop` must re-run every migration revert guard and 19.0 item 2 on the integrated chain, and resolve the textual conflicts in `schema.rs` and `CHANGELOG.md`. On that chain `develop`'s `20261112_v1.10.0` (OPS-03) precedes E1's migration. Its guard `the_migration_applies_reverts_and_reapplies_changing_no_other_schema_object` had the B1 guard's newest-migration assumption; #1032 (stage #1031) fixed it on `develop`, merged on 2026-11-14 after E1's base, so the fix is not on E1's branch (from the PR record; `develop` was not fetched).
- The gates that follow review round 1 under GRANT 6865502816 (G4) are: both CRITICAL review legs confirm the round-1 delta at the new head, PR CI is green at that head, and a SHA-bound MERGE AUTH is given; E2 follows after E1 lands. GitHub records their live state.

## 15. Agent self-assessment

The agent may identify risks but may not approve the task.

Suggested review focus for the round-1 delta (`80fa2148..` the new head):

- `recover_rows` (`fd9aa5b`): the instant read before the selection and bound in both comparisons, against spec 9.3 (reading 22).
- `is_non_blank` (`663e87c`): the four added characters and the corrected comment.
- The corrected comment in `kind_signals` (`5ee6451`) and its section 13 entry.
- The report changes listed in "Review round 1".

Earlier focus, still relevant:

- The state trigger `async_effect_target_state_guard`:
  - its regression bookkeeping (`regressed_*`);
  - the staff reconciliation exception for backwards and same-order writes.
- The deferred checks forced inside claim savepoints, and recovery's skip-and-count path (`async_recovery_failure` ordering).
- The 13.3 decision order in `targets.rs` and `commands.rs`, and the readings in section 5.
- The error mapping, which relies on message text, and the carry-forward commit-failure handling.
- The B1 guard change (`dd38d9b`) against the amendment's text, including the two placements read in section 0.
- The item 6 method: an identity stand-in instead of a real provider (reading 21).
