# THOTH-ASYNC-01-WORKER-01 Implementation Report

This report records step 2 of the async job delivery, the trusted in-process `thoth-worker` runtime (thoth-pub/thoth#1039), at code head `48477179cbdaa6e13b48a94cfc65001113d089a8`, after review round 1 and its re-check. The report was first added in `f6bf6d594bea44b2c8d463e8fd233425c11e9ec3`, which both review legs reviewed, and rewritten for review round 1 in `d2ffe71a244edc64124f1c6290350d7a19cac09b`, which both legs re-checked. The commit that last changes this report is a child of `d2ffe71a`; besides this file it changes effect rule 3 in the module docs of `thoth-api/src/async_worker/mod.rs` (FBC-R1) and no code.

## Review round 1

Both HIGH review legs reviewed `f6bf6d594bea44b2c8d463e8fd233425c11e9ec3` and returned APPROVED (COMMENT reviews on PR #1040; gate comment on #1039):

- Leg OPC: P0 0, P1 0, P2 0, P3 7 (OPC-1 to OPC-7); three core-rule mutants, all caught.
- Leg FBC: P0 0, P1 0, P2 0, P3 9 (FBC-1 to FBC-9); eight core-rule mutants, all caught.
- The Codex bot (advisory) left one P1 (this report's template fields) and one P2 (a liveness time in the future).

Dispositions, one line per finding. "Fixed" means fixed in `4847717`.

- **Codex P1**, the report's template fields: fixed in this revision. Sections 1 to 15 follow `implementation-report-template.md`: exact base and head, actual files, authorized actions and external effects, migration and operational effects, authorization and security, commands and results, CI, rollout and rollback, limitations.
- **Codex P2 = FBC-9**, a liveness time later than now counted as healthy: fixed. `is_healthy` answers unhealthy for it, so a stalled ticker cannot stay healthy until the clock catches up; `w24_health_function` asserts it.
- **OPC-1 = FBC-6**, `db_clock` uses the engine's crate-visible `map_db_error`: recorded as deviation 2 in section 5. Both legs accepted it; no engine file or visibility changed.
- **FBC-7**, engine log lines in the worker process carry the server's primary message: recorded in sections 8 and 13 for step 3.
- **OPC-2 = FBC-3**, a claim-loop panic never set `loop_exited`: fixed. A drop guard (`LoopExit`) sets it when the claim loop or the idle loop ends, by return or by panic. `w20_runtime_thread_panic_exits_1_and_withholds_liveness` now holds a handler in flight, so the ticker runs after the panic, and asserts that the shutdown logs no "claim call was still in flight" line.
- **OPC-3 = FBC-1**, an indefinite answer on a T4 retry left the tracked phase `PRE_WRITE`: fixed. After an unknown try, every error other than `ASYNC_JOB_EFFECT_WINDOW_INSUFFICIENT` (definite: the engine checks the phase first) sets `PossiblyStarted`, and the OPW4-1 mark when the stop signal is set. New W22 case `g`: busy after a commit, then an integrity violation.
- **FBC-2**, the extra renewal after `PHASE` did not re-check the window: fixed. After it, `mark_effect_started` answers `EffectWindowInsufficient` when the runtime deadline leaves less than the window, as a post-effect `confirm_claim` does. New W22 case `h`.
- **FBC-4**, the unrun rule recorded no loop progress: fixed. `apply_unrun` records progress before and after each of its calls.
- **OPC-4**, an older stored lease expiry could overwrite a newer one: fixed, differently from the suggestion. `store_renewal` keeps the larger expiry. The suggested `at >= last_renewal` guard would drop a newer expiry when a context renewal, which reads its instant before it waits for the renewal lock, commits after a heartbeat renewal. Expiries only grow in commit order (the request only grows and the deadline is fixed), so the largest is the latest.
- **OPC-5 = FBC-8**, coarse counters: fixed. A handler `Wait` that the runtime turns into `WAIT_DEADLINE_TOO_CLOSE` counts as a failure, and fault fallbacks have their own counter (`fault fallbacks` in the summary line; reading 4). `w20_fault_fallback_and_wait_resume_at` asserts both.
- **OPC-6**, test gaps: (c) fixed, the resume-delay bound is `14_000..=15_000` ms in W8, W19 and W20; (a), (b) and (d) deferred (section 13).
- **OPC-7**, a missing `--database-url` exits 1, not 2: recorded as decision 12 in section 5.
- **FBC-5**, weak test spots: (a) fixed, W6 asserts `last_heartbeat_at > started_at`; (b) fixed, W22 (f) pushes its fault before the job exists; (c) recorded in section 13.

Mutation check of the fixes, in two builds, then restored: each of six reverts makes its test fail.

| Revert | Caught by |
| --- | --- |
| No `PossiblyStarted` after an indefinite retry | `w22_effect_marks_with_unknown_outcomes` (case `g` reads `PreWrite`) |
| No window check after the extra renewal | `w22_effect_marks_with_unknown_outcomes` (case `h` ends `SUCCEEDED`) |
| `loop_exited` not set during a panic | `w20_runtime_thread_panic_exits_1_and_withholds_liveness` (one "still in flight" line) |
| A future liveness time healthy | `w24_health_function` |
| Fault fallbacks counted as action fallbacks | `w20_fault_fallback_and_wait_resume_at` (fault fallbacks 0, not 2) |
| `WAIT_DEADLINE_TOO_CLOSE` counted as a wait | `w20_fault_fallback_and_wait_resume_at` (no failure counted) |

## Re-check of review round 1

Both legs re-checked `d2ffe71a244edc64124f1c6290350d7a19cac09b` and returned APPROVED: every round-1 finding is closed or accepted as recorded or deferred, and each leg reverted two of the new fixes and saw its test fail. Each leg raised one new P3:

- **OPC-R1**, decision 12 gave a false reason: the database URL is required whatever the kind list (W1 asserts it), not optional for the idle default. Fixed: decision 12 is reworded. The behaviour is unchanged.
- **FBC-R1**, after `EffectWindowInsufficient` from `mark_effect_started` following a committed T4 (case `h`), a handler that returns `Release`, reading effect rule 3 as "before effect start", gets the action fallback's code `HANDLER_ACTION_INVALID`; the job state is right (`RECONCILIATION_REQUIRED`). Recorded: effect rule 3 in the module docs now says that `Release` after this answer from `mark_effect_started` stays right for the job, and that in this one case the attempt's code is `HANDLER_ACTION_INVALID`. A handler cannot tell the case apart, because no public accessor gives the tracked phase; a truthful code is deferred (section 13).

## 1. Repository state

Owning GitHub issue: thoth-pub/thoth#1039 (sub-issue of #957)
Repository: thoth-pub/thoth
Workflow: PROGRAMME_INTEGRATION
Base branch: `feature/worker`
Authorized base commit: `d3941ce5d87c5eea6151990d05949b82b7978069`
Actual base commit: `d3941ce5d87c5eea6151990d05949b82b7978069`
PR target: `feature/worker`
Programme integration branch: `feature/worker`
Task branch: `feature/worker--runtime-01`
Head commit: code head `48477179cbdaa6e13b48a94cfc65001113d089a8`; this file cannot name its own commit, so the PR head is the authority for the exact head
Pull request: thoth-pub/thoth#1040
Expected branch deletion after merge: YES
Final programme PR required: YES (`feature/worker` into `develop`, WORKER-01-INTEG)
Implementation: Opus 5.5, high

## 2. Scope confirmation

Approved specification: `WORKER-01-spec-R2.2.md`, SHA-256 `77647993e4254769c0cd2373e124e89100216d42b9fd9664a75d2222b3f09cc1` (checked before work started), APPROVED by both HIGH legs, with the binding carries OPW4-1, FBW4-1 and FBW4-2 from its final review (section 5.2).

Implemented objective: ADR-0012 section 8 item 2, the trusted in-process executor for job kinds whose executor or reconciler domain is `ThothWorker` and whose risk class is `Trusted`, shipped inert.

Out-of-scope changes made: NONE.

## 3. Commits

- `fe80c9f` - THOTH-ASYNC-01-WORKER-01: Add the in-process async worker runtime
- `a44aae4` - THOTH-ASYNC-01-WORKER-01: Add thoth start worker and thoth worker health
- `f6bf6d5` - THOTH-ASYNC-01-WORKER-01: Record the worker in the changelog and add the implementation report
- `4847717` - THOTH-ASYNC-01-WORKER-01: Address the review round's small findings
- `d2ffe71` - THOTH-ASYNC-01-WORKER-01: Complete the implementation report for review round 1
- the child of `d2ffe71` that records the re-check in this report and rewords effect rule 3 in the module docs

Every commit is authored and committed as `Javier Arias <javier@jarias.org>`.

## 4. Files changed

Authorized write paths and new-file paths: spec section 3 (the new module `thoth-api/src/async_worker/`, which may be split into submodules; `thoth-api/src/lib.rs`; `thoth-api/Cargo.toml`; root `Cargo.toml`; `Cargo.lock`, thoth-api's dependency list only; `src/bin/thoth.rs`, `src/bin/commands/`, `src/bin/arguments/mod.rs`; `CHANGELOG.md`; this report).

The worker ships inert. `PRODUCTION_HANDLERS` and the engine's `PRODUCTION_REGISTRY` are empty, and `ASYNC_WORKER_KINDS` defaults to empty. With an empty kind list the worker builds no pool, opens no database connection and only keeps its liveness file fresh (W5).

New files (all within the authorized new-file list):

```text
thoth-api/src/async_worker/mod.rs               public API; module docs (handler contract, effect rules, operator notes, constants)
thoth-api/src/async_worker/config.rs            WorkerArgs, WorkerConfig, validation, the AWS credential-source check
thoth-api/src/async_worker/errors.rs            ThothError -> fixed codes and refusal reasons
thoth-api/src/async_worker/handler.rs           JobHandler, JobView, contexts and fences, actions, HandlerRegistry, PRODUCTION_HANDLERS
thoth-api/src/async_worker/health.rs            liveness file format, withhold rules, `thoth worker health`
thoth-api/src/async_worker/runtime.rs           startup, pools, shared state, ticker, shutdown, exit code
thoth-api/src/async_worker/runtime/claims.rs    claim sets, claim loop, recovery, dispatch and the live-job rule
thoth-api/src/async_worker/runtime/heartbeat.rs heartbeat on its own one-connection pool
thoth-api/src/async_worker/runtime/slots.rs     slot threads, final actions, fallbacks, the unrun rule, database-clock resume_at
thoth-api/src/async_worker/test_support.rs      (cfg(test)) worker test kinds, scripted handlers, fault and gate hooks, fixtures, capturing logger
thoth-api/src/async_worker/tests.rs             (cfg(test)) the section 8 tests W1 to W25 and W16b
src/bin/commands/worker.rs                      `thoth worker health`
docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-WORKER-01-implementation-report.md   this report
```

`runtime.rs` passed 1,500 lines, so the claim loop, the heartbeat and the slots are submodules under `runtime/`, as section 3 allows.

Modified files, each within the write budget: `thoth-api/src/lib.rs` (the `#[cfg(feature = "backend")] pub mod async_worker;` declaration), `thoth-api/Cargo.toml` (`env_logger = { version = "0.11.11", optional = true }`, added to `backend`), root `Cargo.toml` (tokio feature `signal`), `Cargo.lock` (only `env_logger` added to thoth-api's dependency list; no package added or removed), `src/bin/thoth.rs`, `src/bin/commands/mod.rs`, `src/bin/commands/start.rs`, `src/bin/arguments/mod.rs` (new worker arguments; `database()` unchanged) and `CHANGELOG.md` (one line under `[Unreleased]` / `### Added`).

Files deleted, moved or renamed: NONE. No prohibited file changed: nothing under `thoth-api/src/model/async_job/`, `graphql/`, `policy.rs`, `db.rs`, `schema.rs`, `migrations/`, the other crates, `Dockerfile` or `.github/`.

`git diff --stat d3941ce5..48477179`:

```text
 CHANGELOG.md                                       |    1 +
 Cargo.lock                                         |    1 +
 Cargo.toml                                         |    2 +-
 ...OTH-ASYNC-01-WORKER-01-implementation-report.md |  156 +
 src/bin/arguments/mod.rs                           |   84 +
 src/bin/commands/mod.rs                            |    1 +
 src/bin/commands/start.rs                          |   87 +-
 src/bin/commands/worker.rs                         |   33 +
 src/bin/thoth.rs                                   |  232 +-
 thoth-api/Cargo.toml                               |    4 +-
 thoth-api/src/async_worker/config.rs               |  304 ++
 thoth-api/src/async_worker/errors.rs               |  117 +
 thoth-api/src/async_worker/handler.rs              |  873 +++++
 thoth-api/src/async_worker/health.rs               |  181 ++
 thoth-api/src/async_worker/mod.rs                  |  157 +
 thoth-api/src/async_worker/runtime.rs              | 1558 +++++++++
 thoth-api/src/async_worker/runtime/claims.rs       |  460 +++
 thoth-api/src/async_worker/runtime/heartbeat.rs    |  135 +
 thoth-api/src/async_worker/runtime/slots.rs        |  841 +++++
 thoth-api/src/async_worker/test_support.rs         | 1096 +++++++
 thoth-api/src/async_worker/tests.rs                | 3413 ++++++++++++++++++++
 thoth-api/src/lib.rs                               |    2 +
 22 files changed, 9734 insertions(+), 4 deletions(-)
```

### 4.1 Write-budget compliance

PASS.

### 4.2 Authorized actions actually used

Grant: the HIGH bundle G1 to G4 on #1039 (2026-11-17).

- repository inspection: used
- source edit: used, within the write budget
- new file creation: used, at the authorized paths
- file deletion/move/rename: not used
- branch creation: used (`feature/worker--runtime-01` from `d3941ce5`)
- commit: used
- push: used, to `feature/worker--runtime-01` only
- PR creation/update: used (#1040)
- issue/comment mutation: used (#1039, #957, the review and gate comments)
- manual CI dispatch/rerun: not used
- provider/runtime read: not used
- provider/runtime write: not used
- migration execution: not used (the local test database is migrated by the test harness)
- release/tag/publication: not used
- merge: not used
- deployment: not used
- production activation: not used

Unauthorized actions performed: NONE.

### 4.3 Automatic and manual external effects

Automatic CI/provider effects: each push to the PR runs CI. `build_and_push_staging_docker_image` builds the image and pushes the tag `ghcr.io/thoth-pub/thoth:staging-pr-1040`, an expected side effect of the grant. No service runs that image's worker.

Manually initiated external actions: NONE.

External writes/publication: NONE beyond the PR's staging image tag.

## 5. Implementation decisions

### 5.1 Readings and decisions

Each is the reading safer for jobs and effects where the spec left a choice.

1. **The claim loop and the stop signal.** Besides step 2's check, the loop checks the stop flag before each purpose's claim call in a tick, so no second claim call starts after the signal.
2. **`effect_settled_at`.** The runtime reads the job's attempts and the database clock before the reconciler runs (`effect_settled`, `settled_at`). If they cannot be read (busy on every try), the runtime resolves `StillRequired` without calling the reconciler, so a reconciler never runs without the bound.
3. **A reconciler's `Waiting` with no valid `resume_at`.** When the waiting deadline has passed, or is closer than `poll_interval_min_seconds + RESUME_MARGIN`, the runtime resolves `StillRequired` instead (FBW4-1).
4. **Counters.** Fault fallbacks (`WORKER_ENGINE_FAULT`) have their own counter, `fault fallbacks` in the summary line, apart from action fallbacks (`HANDLER_ACTION_INVALID`). A handler `Wait` that the runtime turns into `WAIT_DEADLINE_TOO_CLOSE` counts as a failure.
5. **The worker's clock query** (`db_clock`, `SELECT clock_timestamp()`) maps errors with the engine's `pub(crate)` `map_db_error`, so a statement timeout is `ASYNC_JOB_BUSY` exactly as in the engine. It reads no table. See deviation 2.
6. **Heartbeat start.** Heartbeat progress and renewal health are recorded just before the heartbeat thread starts (startup step 9), so the first claim tick cannot read an unscheduled heartbeat as stalled.
7. **The live-job unrun rule** uses the same codes as the shutdown unrun rule (`WORKER_SHUTDOWN` where a `fail_on` is needed).
8. **OPW4-1 extended to `PossiblyStarted`.** When the T4 outcome is unknown and the stop signal is seen, `mark_effect_started` also marks the claim, so a following `Release` refused with `PHASE` gets `Indeterminate` `WORKER_SHUTDOWN` as a shutdown fallback.
9. **A payload that does not parse** into a `JobView` is handled as a handler panic (the handler is not called; fallback by tracked phase, `HANDLER_PANIC`).
10. **A READ_ONLY kind** whose `reconciler_domain` is `ThothWorker` joins the reconciliation set like any other kind; the engine never makes such a job `RECONCILIATION_REQUIRED`, so its reconciliation claims return nothing. W6 asserts one poll row per kind and purpose of the claim sets (four for its two kinds).
11. **Test drop.** A `TestWorker` dropped by a failing test hard-stops its runtime without asserting, so one failure cannot leave a runtime claiming the next test's jobs.
12. **A missing `--database-url`** is a validation error with exit code 1 and a line naming the argument, like every other invalid value, not clap's usage code 2: the argument is not clap-`required`, so `WorkerConfig` reports it with every other problem at once (W1). The URL is required whatever the kind list, although the idle default opens no connection.

### 5.2 Carries

- **OPW4-1.** `ClaimStatus::shutdown_after_effect_mark` is set when `mark_effect_started` answers `ShutdownInProgress` after a committed T4 (and for `PossiblyStarted`, reading 8). The action fallback then turns a `Release` refused with `PHASE` into `fail_on` `Indeterminate` `WORKER_SHUTDOWN`, counted under shutdown fallbacks, not action fallbacks. Tested by `w14_opw4_1_release_after_shutdown_in_progress_is_a_shutdown_fallback`.
- **FBW4-1.** `ReconciliationAction::waiting(checkpoint, resume_after)` takes a relative delay. `resolve_waiting` reads `clock_timestamp()` in the same engine transaction as `resolve_reconciliation_on` and sends `resume_at = clock + max(resume_after, poll_interval_min_seconds + RESUME_MARGIN)`, capped at the waiting deadline; reading 3 covers the passed or too-close deadline. `w19_reconciliation_outcomes_and_effect_settled_at` asserts both a database-clock `resume_at` and the cap.
- **FBW4-2.** The module docs state that on a first wait there is no deadline to cap at, so a `resume_after` beyond the kind's waiting horizon is a handler fault and gets the action fallback. `w20_panics_and_invalid_actions` asserts it.

The database accepted `renew_on` and `mark_effect_started_on` in one transaction (W6, W10, W22), so FBW3-1's HOLD did not arise.

### 5.3 Deviations

1. `thoth-api/src/lib.rs` gains two lines, the `#[cfg(feature = "backend")]` attribute and the `pub mod async_worker;` declaration, which section 3 counts as one. Accepted by both review legs.
2. D2 says the worker uses only the engine's public API. `db_clock` uses the engine's crate-visible `map_db_error` (reading 5), so the busy classification cannot drift from the engine's. No engine file or visibility changed. Accepted by both review legs (OPC-1, FBC-6). Step 4 may add a public clock helper to the engine and remove this use.
3. W8, W19 and W20 bound the database-clock delay from the attempt's `finished_at`, which the engine writes after the worker's clock read in the same transaction, so the bound is `14_000..=15_000` ms rather than exactly 15 s.

No HOLD condition arose.

## 6. Database and migration effects

Migration added: NO. The worker runs no migration. At startup it checks, read only, that every embedded migration is applied (W23), and waits while one is pending. Its own SQL is the per-connection session settings (`statement_timeout` 15 s, `idle_in_transaction_session_timeout` 60 s), that read-only check, and `SELECT clock_timestamp()`. Everything else goes through the engine's `_on` functions in engine transactions.

## 7. API and compatibility effects

GraphQL/API changes: none. Generated schema/client updates: none.

New command-line surface: `thoth start worker` (the shared `--database-url` / `DATABASE_URL`, with its value hidden from `--help`, and `ASYNC_WORKER_NAME`, `ASYNC_WORKER_SLOTS`, `ASYNC_WORKER_POLL_SECONDS`, `ASYNC_WORKER_SHUTDOWN_GRACE_SECONDS`, `ASYNC_WORKER_KINDS`, `ASYNC_WORKER_CAPABILITIES`, `ASYNC_WORKER_LIVENESS_FILE`) and `thoth worker health` (`--liveness-file`, `--max-age-seconds` / `ASYNC_WORKER_HEALTH_MAX_AGE_SECONDS`). The other commands are unchanged (W26).

Backwards compatibility: additive. Deprecations: none.

Cross-repository dependencies: none now. Step 3 (thoth-pub/infrastructure) will run `thoth start worker` and use `thoth worker health` as the container health check, so this command and environment contract is what it consumes; step 4 moves thoth-dissemination and thoth-app work onto jobs.

## 8. Authorization and security

Authorization paths changed: none. `policy.rs` and the GraphQL layer are unchanged. The worker calls the engine's `_on` functions directly, which no API exposes; who can run a worker is decided by the database credential its deployment gets (step 3).

Roles/scopes involved: none. Negative authorization tests: not applicable (no authorization path).

Secret or personal-data handling:

- The database URL is never logged, and its value is hidden from `--help`. W21 runs the worker against an unreachable URL whose password is a sentinel and asserts that no log record holds it.
- The worker never reads the whole environment (W21's source guard). It refuses to start when any of `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `AWS_SESSION_TOKEN`, `AWS_PROFILE`, `AWS_SHARED_CREDENTIALS_FILE` or `AWS_CONFIG_FILE` is set (D6, W2).
- Worker log lines follow the section 7.1 allowlist: ids, kinds, codes, counts and durations, never a payload, checkpoint, evidence, error detail, provider reference, token, target key, database URL, environment value or driver text. The panic hook logs the location only, and the r2d2 error handler logs fixed text.

Security limitations: the engine's own log lines (step 1, unchanged) carry the database server's primary message for integrity violations, and they come out of the worker's logger without the `async worker:` prefix (FBC-7). They hold no row content.

## 9. Tests and checks

### 9.1 Section 8 tests

All section 8 tests are `#[test]` functions in `thoth-api/src/async_worker/tests.rs`, except W26, which is in `src/bin/thoth.rs` (module `worker_command`).

| Test | Functions |
| --- | --- |
| W1 | `w1_configuration_defaults_bounds_and_redaction` |
| W2 | `w2_aws_credential_sources_refuse_startup` |
| W3 | `w3_handler_registry_validation` |
| W4 | `w4_claim_sets_leave_out_with_one_line_each` |
| W5 | `w5_inert_default_builds_no_pool_and_changes_nothing` |
| W6 | `w6_success_read_only_and_effectful` |
| W7 | `w7_retryable_then_success_budget_exhausted_and_permanent` |
| W8 | `w8_wait_then_resume_with_checkpoint_and_confirm_claim` |
| W9 | `w9_heartbeat_renews_soonest_expiry_first`, `w9_heartbeat_extends_the_lease_never_past_the_deadline` |
| W10 | `w10_effect_lease_covers_the_window_with_a_heartbeat_renewal_in_flight`, `w10_resumed_fence_and_window_insufficient` |
| W11 | `w11_claim_taken_away_by_recovery_or_cancel`, `w11_lease_expired_refuses_t4_but_complete_after_t5_commits` |
| W12 | `w12_shutdown_at_pre_write` |
| W13 | `w13_shutdown_after_effect_start_and_resumable_confirmed` |
| W14 | `w14_fences_refuse_after_the_signal`, `w14_t4_committed_before_the_fallback_release_gets_indeterminate`, `w14_opw4_1_release_after_shutdown_in_progress_is_a_shutdown_fallback` |
| W15 | `w15_unrun_rule_after_the_signal` |
| W16 | `w16_abrupt_termination_is_recovered_by_a_second_runtime` |
| W16b | `w16b_connection_loss_is_survived` |
| W17 | `w17_two_runtimes_share_the_queue_exactly_once` |
| W18 | `w18_mixed_versions_older_runtime_leaves_newer_work` |
| W19 | `w19_reconciler_outcomes_are_exactly_five`, `w19_effect_settled_at_is_none_without_an_effect_start`, `w19_reconciliation_outcomes_and_effect_settled_at` |
| W20 | `w20_panics_and_invalid_actions`, `w20_fault_fallback_and_wait_resume_at`, `w20_runtime_thread_panic_exits_1_and_withholds_liveness`, `w20_panic_line_has_the_location_only` |
| W21 | `w21_no_secret_reaches_a_log_record`, `w21_source_guard_never_reads_the_whole_environment` |
| W22 | `w22a_row_lock_past_the_lock_timeout_and_busy_claims`, `w22_effect_marks_with_unknown_outcomes`, `w22f_signal_during_effect_mark_never_answers_ok`, `w22_post_effect_confirm_claim_returning_short_of_the_window` |
| W23 | `w23_pending_migrations_comparison`, `w23_pending_migrations_read_only_check` |
| W24 | `w24_health_function`, `w24_withhold_rules`, `w24_liveness_while_waiting_for_the_database`, `w24_liveness_during_migration_wait_and_busy_renewals` |
| W25 | `w25_recover_when_no_slot_is_free`, `w25_reclaimed_job_with_its_old_handler_still_running` |
| W26 | `worker_command::start_worker_has_exactly_the_section_4_arguments`, `worker_command::start_worker_help_never_prints_the_database_url`, `worker_command::the_shared_database_argument_is_unchanged_for_the_other_commands`, `worker_command::worker_health_has_its_two_arguments`, `worker_command::worker_health_answers_1_for_a_missing_liveness_file` |

`tests.rs` also holds `resume_plan_and_deadline_guard`, a unit test of the database-clock `resume_at` plan and the deadline guard. That makes 46 tests in `async_worker` and 5 W26 tests in the binary, 51 in all. Review round 1 added cases to existing functions (W22 `g` and `h`, the W20 counters, the W24 future time), not new functions.

The test kinds are re-implemented in `test_support.rs` (owning domain `worker_test`); the engine's private fixtures are not used, as section 8 requires. Fault injection uses call-wrapper hooks (`Op`, `Fault`: busy without running, busy after a commit, integrity violation), gates (pause after a claim commits; pause a heartbeat renewal), forced pending migrations, an injected claim-loop panic and a hard stop. Synchronisation uses flags and bounded condition waits (`wait_for`, `Flag::wait`, 30 s bound).

Fixed sleeps, none needed for correctness: W10 sleeps 100 ms as an ordering aid before releasing the paused heartbeat renewal (the assertions hold whichever of the two takes the renewal lock first); W17's handler sleeps 20 ms so the concurrency sample has overlap to observe; W9's handler polls `should_yield` every 50 ms. Both review legs accepted all three.

### 9.2 Commands and results

Environment: `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 THOTH_EXPORT_API=https://export.thoth.pub TEST_DATABASE_URL=postgres://thoth:thoth@localhost/thoth_test TEST_REDIS_URL=redis://localhost:6379`, local PostgreSQL 16 and Redis, toolchain cargo 1.97.

Baseline at `d3941ce5`: `cargo test -p thoth-api --features backend --lib` ended `test result: ok. 2630 passed; 0 failed; 7 ignored`.

At `f6bf6d59` (the reviewed head):

```text
cargo fmt --all -- --check                                      exit 0, no output
cargo clippy --all --all-targets --all-features -- -D warnings  exit 0 (Finished; no warning from the workspace)
cargo check --workspace                                         exit 0 (Finished)
cargo test -p thoth-api --features backend --lib                test result: ok. 2676 passed; 0 failed; 7 ignored (2630 baseline + 46 async_worker)
cargo test -p thoth                                             test result: ok. 45 passed; 0 failed; 0 ignored (includes the 5 W26 tests)
```

At code head `48477179` (review round 1):

```text
cargo fmt --all -- --check                                      exit 0, no output
cargo clippy --all --all-targets --all-features -- -D warnings  exit 0 (Finished; no warning from the workspace)
cargo check --workspace                                         exit 0 (Finished)
cargo test -p thoth-api --features backend --lib async_worker   test result: ok. 46 passed; 0 failed; 0 ignored
cargo test -p thoth-api --features backend --lib                test result: ok. 2676 passed; 0 failed; 7 ignored
cargo test -p thoth                                             test result: ok. 45 passed; 0 failed; 0 ignored (includes the 5 W26 tests)
```

`cargo test --workspace` (optional) was not run. The only warning cargo printed is the existing future-incompatibility note for the dependency `proc-macro-error2`.

## 10. Manual verification

Environment: local only. No manual run outside the tests: W5 runs the runtime with the default configuration (no kind) and asserts it builds no pool and opens no connection; W26 parses the binary's arguments and runs `thoth worker health` against a missing file.

## 11. CI

At `f6bf6d594bea44b2c8d463e8fd233425c11e9ec3`, PR #1040's first run passed every check: `format_check`, `build`, `lint`, `test`, `run_migrations`, `build_and_push_staging_docker_image`, `check-changelog` and three `classify` jobs.

PR CI runs again on each new head; GitHub records its result.

## 12. Rollout and rollback

Initial state after merge: inert. No kind is registered, `PRODUCTION_HANDLERS` is empty and `ASYNC_WORKER_KINDS` defaults to empty, and no service runs `thoth start worker`.
Activation required: step 3 (a Fargate service running `thoth start worker`), then kind registration and a non-empty `ASYNC_WORKER_KINDS`, each under its own grant.
Feature flag/configuration: `ASYNC_WORKER_KINDS` is the claim switch; empty means idle with no database connection.
Migration sequence: none.
Rollback/disable procedure: before step 3, revert the merge (nothing runs the worker). Once a service runs it: set `ASYNC_WORKER_KINDS` empty or stop the service; the engine recovers any claim the worker held at lease expiry (W16). The container's stop timeout must be at least `max(grace, 25) + 20 + 10` seconds (spec 6.8).
Monitoring required: none until step 3; then the liveness file through `thoth worker health`, the summary line and the outcome lines (7.1, 7.3), and the engine's kind signals.

## 13. Known limitations and deferred work

- **Engine log lines (FBC-7).** The engine's `map_db_error` logs the server's primary message for integrity violations under the `async job engine:` prefix, from the worker's logger. Step 3's log handling and metric filters should know this.
- **The `map_db_error` use (deviation 2).** Step 4 may give the engine a public clock helper and drop it.
- **W23 (OPC-6 a).** The test proves that `in_read_only` refuses DDL, not that `applied_migrations` uses it.
- **Hung-slot accounting (OPC-6 b).** W24 tests the hung-slot, all-slots-hung and heartbeat-stall withhold rules through the pure `withhold_reason`; `hung_durations` has no runtime test.
- **Long back-off liveness (OPC-6 d).** Covered by construction (the loop records progress around every sleep), not by a test.
- **The 14,000 ms lower bound (FBC-5 c).** It assumes less than 1 s between two statements of one transaction; a stalled CI host could fail it spuriously.
- **The code after a late window refusal (FBC-R1).** When `mark_effect_started` answers `EffectWindowInsufficient` after its T4 committed (case `h`) and the handler returns `Release`, as effect rule 3 allows, the attempt's code is `HANDLER_ACTION_INVALID`, although the handler followed the rules. The job ends `RECONCILIATION_REQUIRED` either way. Step 4, which brings the first handlers, should either give handlers a public tracked-phase accessor or make the runtime give this case a code of its own.
- **A panicked runtime exits quickly.** After a runtime-thread panic with no claim in flight, the process may exit before the ticker's next period; the exit code (1) is then the signal, and the ticker withholds only while the runtime is still up (W20).

## 14. Unresolved issues

- NONE.

## 15. Agent self-assessment

The agent may identify risks but may not approve the task.

Suggested review focus for the review-round delta (`f6bf6d59..48477179`):

- the new arm at the end of `mark_effect_started`'s retry loop and the window check after the extra renewal (`handler.rs`);
- `LoopExit` in both loops (`runtime/claims.rs`) and the rewritten W20 panic test;
- `store_renewal` keeping the larger expiry instead of the suggested guard (`runtime.rs`);
- the counter changes and the summary line (`runtime.rs`, `runtime/slots.rs`);
- `is_healthy` and the tightened tests.
