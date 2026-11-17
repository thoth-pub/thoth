# THOTH-ASYNC-01-WORKER-01 Implementation Report

This report records step 2 of the async job delivery, the trusted in-process `thoth-worker` runtime (thoth-pub/thoth#1039), built on branch `feature/worker--runtime-01` from base `d3941ce5d87c5eea6151990d05949b82b7978069`.

- Spec: `WORKER-01-spec-R2.2.md`, SHA-256 `77647993e4254769c0cd2373e124e89100216d42b9fd9664a75d2222b3f09cc1` (checked before work started), APPROVED by both HIGH legs.
- Binding carries: OPW4-1, FBW4-1 and FBW4-2 from the R2.2 review (section 4).
- Model: Implementation: Opus 5.5, high.

## 1. What was built

The worker ships inert. `PRODUCTION_HANDLERS` and the engine's `PRODUCTION_REGISTRY` are empty, and `ASYNC_WORKER_KINDS` defaults to empty. With an empty kind list the worker builds no pool, opens no database connection and only keeps its liveness file fresh (W5).

New files:

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
```

`runtime.rs` passed 1,500 lines, so the claim loop, the heartbeat and the slots are submodules under `runtime/`, as section 3 allows.

Modified files: `thoth-api/src/lib.rs` (the `#[cfg(feature = "backend")] pub mod async_worker;` declaration), `thoth-api/Cargo.toml` (`env_logger = { version = "0.11.11", optional = true }`, added to `backend`), root `Cargo.toml` (tokio feature `signal`), `Cargo.lock` (only `env_logger` added to thoth-api's dependency list; no package added or removed), `src/bin/thoth.rs`, `src/bin/commands/mod.rs`, `src/bin/commands/start.rs`, `src/bin/arguments/mod.rs` (new worker arguments; `database()` unchanged) and `CHANGELOG.md` (one line under `[Unreleased]` / `### Added`).

No prohibited file changed: nothing under `thoth-api/src/model/async_job/`, `graphql/`, `migrations/`, the other crates, `Dockerfile` or `.github/`.

### 1.1 The binary

`thoth start worker` registers `arguments::database().hide_env_values(true)` and the seven `ASYNC_WORKER_*` arguments, each a plain string; the worker validates every value and reports every problem at once (W1). `commands::start::worker` calls `launch`, starts a thread with a tokio current-thread runtime that forwards `SIGTERM` and `SIGINT` to `WorkerHandle::request_shutdown` (the first starts the shutdown, a second ends the grace phase), calls `signal_setup_failed` if the listener cannot be set up, then runs the worker. `src/bin/thoth.rs` is the only caller of `std::process::exit`, with the run function's code. `thoth worker health` takes `--liveness-file` and `--max-age-seconds` and calls `health_command`. The binary does no logging of its own and gains no `log` or `env_logger` dependency.

## 2. Section 8 tests

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

`tests.rs` also holds `resume_plan_and_deadline_guard`, a unit test of the database-clock `resume_at` plan and the deadline guard. That makes 46 tests in `async_worker` and 5 W26 tests in the binary, 51 in all.

The test kinds are re-implemented in `test_support.rs` (owning domain `worker_test`); the engine's private fixtures are not used, as section 8 requires. Fault injection uses call-wrapper hooks (`Op`, `Fault`: busy without running, busy after a commit, integrity violation), gates (pause after a claim commits; pause a heartbeat renewal), forced pending migrations, an injected claim-loop panic and a hard stop. Synchronisation uses flags and bounded condition waits (`wait_for`, `Flag::wait`, 30 s bound).

Fixed sleeps, none needed for correctness: W10 sleeps 100 ms as an ordering aid before releasing the paused heartbeat renewal (the assertions hold whichever of the two takes the renewal lock first); W17's handler sleeps 20 ms so the concurrency sample has overlap to observe; W9's handler polls `should_yield` every 50 ms.

## 3. Readings and decisions

Each is the reading safer for jobs and effects where the spec left a choice.

1. **The claim loop and the stop signal.** Besides step 2's check, the loop checks the stop flag before each purpose's claim call in a tick, so no second claim call starts after the signal.
2. **`effect_settled_at`.** The runtime reads the job's attempts and the database clock before the reconciler runs (`effect_settled`, `settled_at`). If they cannot be read (busy on every try), the runtime resolves `StillRequired` without calling the reconciler, so a reconciler never runs without the bound.
3. **A reconciler's `Waiting` with no valid `resume_at`.** When the waiting deadline has passed, or is closer than `poll_interval_min_seconds + RESUME_MARGIN`, the runtime resolves `StillRequired` instead (FBW4-1).
4. **Fault fallbacks** (`WORKER_ENGINE_FAULT`) are counted with the action fallbacks in the summary line.
5. **The worker's clock query** (`db_clock`, `SELECT clock_timestamp()`) maps errors with the engine's `pub(crate)` `map_db_error`, so a statement timeout is `ASYNC_JOB_BUSY` exactly as in the engine. It reads no table.
6. **Heartbeat start.** Heartbeat progress and renewal health are recorded just before the heartbeat thread starts (startup step 9), so the first claim tick cannot read an unscheduled heartbeat as stalled.
7. **The live-job unrun rule** uses the same codes as the shutdown unrun rule (`WORKER_SHUTDOWN` where a `fail_on` is needed).
8. **OPW4-1 extended to `PossiblyStarted`.** When the T4 outcome is unknown and the stop signal is seen, `mark_effect_started` also marks the claim, so a following `Release` refused with `PHASE` gets `Indeterminate` `WORKER_SHUTDOWN` as a shutdown fallback.
9. **A payload that does not parse** into a `JobView` is handled as a handler panic (the handler is not called; fallback by tracked phase, `HANDLER_PANIC`).
10. **A READ_ONLY kind** whose `reconciler_domain` is `ThothWorker` joins the reconciliation set like any other kind; the engine never makes such a job `RECONCILIATION_REQUIRED`, so its reconciliation claims return nothing. W6 asserts one poll row per kind and purpose of the claim sets (four for its two kinds).
11. **Test drop.** A `TestWorker` dropped by a failing test hard-stops its runtime without asserting, so one failure cannot leave a runtime claiming the next test's jobs.

## 4. Carries

- **OPW4-1.** `ClaimStatus::shutdown_after_effect_mark` is set when `mark_effect_started` answers `ShutdownInProgress` after a committed T4 (and for `PossiblyStarted`, reading 8). The action fallback then turns a `Release` refused with `PHASE` into `fail_on` `Indeterminate` `WORKER_SHUTDOWN`, counted under shutdown fallbacks, not action fallbacks. Tested by `w14_opw4_1_release_after_shutdown_in_progress_is_a_shutdown_fallback` (the handler sees `ShutdownInProgress` with tracked phase `EFFECT_STARTED`; the job ends `RECONCILIATION_REQUIRED` with error code `WORKER_SHUTDOWN`, not `HANDLER_ACTION_INVALID`; the shutdown-fallback counter is 1 and the action-fallback counter 0; exit code 0).
- **FBW4-1.** `ReconciliationAction::waiting(checkpoint, resume_after)` takes a relative delay. `resolve_waiting` reads `clock_timestamp()` in the same engine transaction as `resolve_reconciliation_on` and sends `resume_at = clock + max(resume_after, poll_interval_min_seconds + RESUME_MARGIN)`, capped at the waiting deadline; reading 3 covers the passed or too-close deadline. The module docs state it. `w19_reconciliation_outcomes_and_effect_settled_at` asserts both a database-clock `resume_at` and the cap at the waiting deadline (`resume_after` of 7,200 s).
- **FBW4-2.** The module docs state, beside the cap, that on a first wait there is no deadline to cap at, so a `resume_after` beyond the kind's waiting horizon is a handler fault and gets the action fallback. `w20_panics_and_invalid_actions` asserts it: a first `Wait` past the horizon at pre-write ends `FAILED` with `HANDLER_ACTION_INVALID`.

The database accepted `renew_on` and `mark_effect_started_on` in one transaction (W6, W10, W22), so FBW3-1's HOLD did not arise.

## 5. Commands and results

Environment: `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 THOTH_EXPORT_API=https://export.thoth.pub TEST_DATABASE_URL=postgres://thoth:thoth@localhost/thoth_test TEST_REDIS_URL=redis://localhost:6379`, local PostgreSQL and Redis.

Baseline at `d3941ce5`: `cargo test -p thoth-api --features backend --lib` ended `test result: ok. 2630 passed; 0 failed; 7 ignored`.

At the final head:

```text
cargo fmt --all -- --check                                      exit 0, no output
cargo clippy --all --all-targets --all-features -- -D warnings  exit 0 (Finished; no warning from the workspace)
cargo check --workspace                                         exit 0 (Finished)
cargo test -p thoth-api --features backend --lib                test result: ok. 2676 passed; 0 failed; 7 ignored (2630 baseline + 46 async_worker)
cargo test -p thoth                                             test result: ok. 45 passed; 0 failed; 0 ignored (includes the 5 W26 tests)
```

`cargo test --workspace` (optional) was not run. The only warning cargo printed is the existing future-incompatibility note for the dependency `proc-macro-error2`.

## 6. Diff

```text
 CHANGELOG.md                                                                                          |    1 +
 Cargo.lock                                                                                            |    1 +
 Cargo.toml                                                                                            |    2 +-
 docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-WORKER-01-implementation-report.md |  156 +++++
 src/bin/arguments/mod.rs                                                                              |   84 +++
 src/bin/commands/mod.rs                                                                               |    1 +
 src/bin/commands/start.rs                                                                             |   87 ++-
 src/bin/commands/worker.rs                                                                            |   33 +
 src/bin/thoth.rs                                                                                      |  232 ++++++-
 thoth-api/Cargo.toml                                                                                  |    4 +-
 thoth-api/src/async_worker/config.rs                                                                  |  304 +++++++++
 thoth-api/src/async_worker/errors.rs                                                                  |  117 ++++
 thoth-api/src/async_worker/handler.rs                                                                 |  855 +++++++++++++++++++++++
 thoth-api/src/async_worker/health.rs                                                                  |  176 +++++
 thoth-api/src/async_worker/mod.rs                                                                     |  157 +++++
 thoth-api/src/async_worker/runtime.rs                                                                 | 1550 +++++++++++++++++++++++++++++++++++++++++
 thoth-api/src/async_worker/runtime/claims.rs                                                          |  450 ++++++++++++
 thoth-api/src/async_worker/runtime/heartbeat.rs                                                       |  135 ++++
 thoth-api/src/async_worker/runtime/slots.rs                                                           |  826 ++++++++++++++++++++++
 thoth-api/src/async_worker/test_support.rs                                                            | 1096 +++++++++++++++++++++++++++++
 thoth-api/src/async_worker/tests.rs                                                                   | 3341 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
 thoth-api/src/lib.rs                                                                                  |    2 +
 22 files changed, 9606 insertions(+), 4 deletions(-)
```

## 7. Deviations

- `thoth-api/src/lib.rs` gains two lines, the `#[cfg(feature = "backend")]` attribute and the `pub mod async_worker;` declaration, which section 3 counts as one.
- Readings 1 to 11 of section 3 fill gaps in the spec; none changes a stated rule.
- W8, W19 and W20 bound the database-clock delay from the attempt's `finished_at`, which the engine writes after the worker's clock read in the same transaction, so the lower bound is 14,000 ms rather than 15 s.

No HOLD condition arose.
