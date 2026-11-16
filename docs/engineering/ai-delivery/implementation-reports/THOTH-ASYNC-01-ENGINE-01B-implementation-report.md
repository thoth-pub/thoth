# THOTH-ASYNC-01-ENGINE-01B Implementation Report

E2 of `THOTH-ASYNC-01-ENGINE-01`: the protected GraphQL API of the shared PostgreSQL job engine that E1 (`THOTH-ASYNC-01-ENGINE-01A`) built. Risk CRITICAL. The task is complete and no HOLD is open. Section 5 lists the readings. Section 14 lists two items for the reviewers to confirm and one for the orchestrator.

## 1. Repository state

Owning GitHub issue: thoth-pub/thoth#1026 (programme thoth-pub/thoth#957)
Repository: thoth-pub/thoth
Workflow: PROGRAMME_INTEGRATION
Base branch: `feature/worker`
Authorized base commit: `3464fdbfe576bb23156eb05b0648768ea75b88f3` (E1's merge commit)
Actual base commit: `3464fdbfe576bb23156eb05b0648768ea75b88f3`. The mandatory preflight confirmed `origin/feature/worker` at this SHA, and `git ls-remote origin refs/heads/feature/worker` still printed it after the code head was pushed (2026-11-16, 15:00 UTC, just before the report commit). Nothing was merged or rebased.
PR target: `feature/worker`
Programme integration branch: `feature/worker`
Task branch: `feature/worker--engine-01b`
Head commit: the code head is `28be993e243d6d1e053cbc3a8dd0f98e0f15f78c`. This report and the `CHANGELOG.md` line are in the commit that follows it.
Pull request: none yet. The orchestrator opens the draft PR into `feature/worker`.
Expected branch deletion after merge: YES
Final programme PR required: YES (`feature/worker` to `develop`, G6, outside this slice)
Implementing model: Claude Opus 5.5 (one implementer across several context compactions; no work was lost)
Reasoning level: max

## 2. Scope confirmation

Approved specification: `ENGINE-01-spec-R5.5.md`, SHA-256 `75e38f4b83697cdb860e4bb01bffc64dc98297127778afa952e47da0fe021baf` (verified). The work runs under GRANT 6865502816 (G5), with the CTO's G5 amendment on #1026 (decision 2026-11-16 11:14Z). The amendment brings the E2 carry fixes FBE-2 and FBE-5 into the E2 write budget: in E1's files, for those two fixes only, with tests in `thoth-api/src/model/async_job/tests.rs`.

Implemented objective: E2 as the specification states it:

- The ten executor operations of section 14.1 (`claimAsyncJobs`, `renewAsyncJobLease`, `markAsyncJobEffectStarted`, `markAsyncJobEffectConfirmed`, `waitAsyncJob`, `completeAsyncJob`, `failAsyncJob`, `releaseAsyncJob`, `claimAsyncJobReconciliations`, `resolveAsyncJobReconciliation`) and the executor query `asyncJobTargetState`.
  - They are authorized through `require_dissemination_worker` and the code-owned executor map (`executor.rs`).
  - The engine checks the kind domain on every claim and on every token-fenced call.
- The four operator commands of section 14.2 and the four reads of section 14.3, superuser only.
- The per-request claim memo in `Context`, and the registry carried by `Context`, which is `&PRODUCTION_REGISTRY` unless a test injects one.
- The error mapping of section 14.4 reaching clients. E1 already had the variants.
- The access-log rule of section 16, and the `reason` field description.
- The SDL guards of sections 17 and 19.2.
- Every E2 test of section 19.2.
- The FBE-2 and FBE-5 carry fixes, under the amendment.

Out-of-scope changes made: NONE.

## 3. Commits

All commits are authored and committed by `Javier Arias <javier@jarias.org>` and carry no trailers. This was checked with `git log --format='%h %an <%ae> | %cn <%ce> | %(trailers:only,unfold) | %s' 3464fdbf..HEAD`.

Oldest first:

- `e968e35` - Read the signal kinds by a loose index scan and bound the regressed list (FBE-2)
- `d55d399` - Refuse only data exceptions as jsonb input and log nested wrappers apart (FBE-5)
- `51427f5` - Add the async job executor map, API types and per-request claim memo
- `c9fde12` - Expose the async job executor operations, operator commands and reads
- `b63bab9` - Omit async job request bodies from the access log
- `1c30a24` - Add the async job API tests
- `28be993` - Name the async job list position input without the word cursor (decision 10)
- the report commit, which adds this report and the `CHANGELOG.md` line

## 4. Files changed

Authorized write paths (handoff section 5, from spec 22.4's E2 line, and the G5 amendment):

- New: `thoth-api/src/graphql/async_job_tests.rs`, `thoth-api/src/graphql/types/async_job.rs`, `thoth-api/src/model/async_job/executor.rs`, this report.
- Edit: `thoth-api/src/graphql/{mutation.rs, query.rs, model.rs, mod.rs, sdl_support.rs}` and `thoth-api/src/graphql/types/mod.rs` (additive).
- Edit: `thoth-api/src/policy.rs`, the two documentation comments only.
- Edit: `thoth-api/src/graphql/metric_registry_tests.rs`, the `require_superuser()` count only.
- Edit: `thoth-api/src/model/async_job/`, for the API adapters and the access-log predicate (`mod.rs` gains the `executor` declaration).
- Edit: `thoth-errors/src/lib.rs`, only if needed.
- Edit: `thoth-api-server/src/logger.rs`, the section 16 rule and its unit tests only.
- Edit: `CHANGELOG.md`, one line.
- Under the amendment: `thoth-api/src/model/async_job/signals.rs` (FBE-2), `thoth-api/src/model/async_job/mod.rs` (FBE-5), and their tests in `thoth-api/src/model/async_job/tests.rs`.

Actual files changed (`git diff --numstat 3464fdbf..HEAD`, lines added and removed). Every file is within the authorized write budget.

| File | +/- | Reason and behavioural effect |
|---|---|---|
| `thoth-api/src/graphql/types/async_job.rs` (new) | +1675 | The GraphQL enums mirroring the engine's, the input and output types, and their conversions to and from the engine types. Also the per-request claim memo, with 5 unit tests |
| `thoth-api/src/graphql/mutation.rs` | +363 | The 10 executor mutations and 4 operator commands, `run_async_job_claim` and `authorize_async_command`. Existing resolvers are unchanged |
| `thoth-api/src/graphql/query.rs` | +127 -1 | `asyncJobTargetState` and the four superuser reads. The removed line is an import list that `rustfmt` reflowed |
| `thoth-api/src/graphql/model.rs` | +22 | The `Context` registry field (`&PRODUCTION_REGISTRY` from `Context::new`, whose signature is unchanged), `with_async_registry`, and the claim memo field |
| `thoth-api/src/graphql/mod.rs` | +2 | The `async_job_tests` module declaration |
| `thoth-api/src/graphql/types/mod.rs` | +1 | The `async_job` module declaration |
| `thoth-api/src/graphql/async_job_tests.rs` (new) | +3846 | The 25 API tests (section 9.1) |
| `thoth-api/src/graphql/metric_registry_tests.rs` | +3 -2 | The `require_superuser()` count in `query.rs`, 7 to 11, and its message |
| `thoth-api/src/policy.rs` | +21 -10 | The `DisseminationWorker` role and `require_dissemination_worker` documentation comments only. They now list the BE-04 operations and the generic executor operations |
| `thoth-api/src/model/async_job/executor.rs` (new) | +221 | The executor map and `authorize_executor`, the API adapters, and the access-log predicate with its two constants |
| `thoth-api/src/model/async_job/mod.rs` | +54 -10 | `pub mod executor;` and FBE-5 (amendment) |
| `thoth-api/src/model/async_job/signals.rs` | +48 -9 | FBE-2 (amendment) |
| `thoth-api/src/model/async_job/tests.rs` | +245 -1 | The 5 FBE-2 and FBE-5 tests (amendment). The removed line is the module comment's list of test groups, which now names them |
| `thoth-api-server/src/logger.rs` | +133 | The body-omission rule of section 16 and its 6 unit tests |
| `CHANGELOG.md` | +1 | One Unreleased line linking #1026 |
| this report (new) | | |

`sdl_support.rs`, `thoth-errors/src/lib.rs` and every other file are unchanged. E1 added all 17 error variants of section 14.4, so `thoth-errors` needed nothing.

Files deleted, moved or renamed: NONE.

### 4.1 Write-budget compliance

PASS. Every changed path is in the list above:

- `policy.rs` changes only `///` lines. A diff filtered to non-comment lines is empty.
- `metric_registry_tests.rs` changes only the count (`7` to `11`) and its message.
- `git diff 3464fdbf -- '*distribution_job*' '*publisher_service_configuration*' thoth-api/migrations` is empty.
- No added line contains `distribution_job` or `publisher_service_configuration`.
- No `Cargo.toml` or `Cargo.lock` changed.

### 4.2 Authorized actions actually used

- repository inspection: YES
- source edit: YES, within the budget
- new file creation: YES, the four listed files
- file deletion/move/rename: NO repository file. Only build output was deleted:
  - old test executables under `target/debug/deps`, as the handoff allows, to keep free disk above 3 GB;
  - two stale `thoth-errors` build units, so that cargo rebuilt them from the current source (section 9, Workspace).
- branch creation: NO (the branch existed at the base)
- commit: YES
- push: YES, ordinary pushes of `feature/worker--engine-01b` only
- PR creation/update: NO
- issue/comment mutation: NO
- manual CI dispatch/rerun: NO
- provider/runtime read: NO
- provider/runtime write: NO
- migration execution: on disposable local databases only. These were the test suite's `thoth_test` on the local PostgreSQL 17.10, and the scratch database `e2_explain`, built for the FBE-2 plan check and dropped afterwards (section 10).
- release/tag/publication: NO
- merge: NO
- deployment: NO
- production activation: NO
- other: a temporary Rust 1.100.0 toolchain, the newest stable (stable channel manifest dated 2026-11-12), for the one newest-stable clippy run, with its own target folder.
  - A first attempt was stopped by its disk guard before it finished, and gave no result. The run was repeated with the target folder in RAM (section 9).
  - The toolchain and both target folders were removed afterwards, as handoff section 6 allows.

Unauthorized actions performed: NONE known. Section 14 asks the orchestrator to confirm the build-output deletion.

### 4.3 Automatic and manual external effects

Automatic CI/provider effects observed: none. A push to the task branch starts no workflow. The draft PR's CI runs when the orchestrator opens it.

Manually initiated external actions: NONE.

External writes/publication: NONE, other than the pushes of the task branch.

## 5. Implementation decisions

Decisions within the approved design:

1. **Executor map.** `executor.rs` holds the map.
   - `executor_profile(Role)` is a function over the existing `Role` enum and adds no variant. It returns a profile only for `DisseminationWorker`: domain `Dissemination`, risk class `Trusted`, capability `dissemination`, revision modes `CURRENT_STATE` and `REVISION_BOUND`.
   - `authorize_executor` calls the existing `require_dissemination_worker` guard, then takes that profile. `SUPERUSER` never satisfies it.
   - It lives in `executor.rs` beside the map, not in `policy.rs`, which may change only in two comments.
   - `executor_principal` comes from the token's user id, and `claimedBy` is the executor name.
   - The engine's `authorize` refuses the whole claim with `ASYNC_JOB_KIND_NOT_AUTHORIZED` when a known requested kind is not this executor's, so no `ThothWorker` kind is claimable through GraphQL.
   - Every token-fenced call builds its `WorkerClaim` with the profile's domain. The engine's prelude refuses a job of another domain before checking the token.
2. **Authorization runs before the claim memo.** A refused caller stores nothing, so the memo only ever holds an authorized executor's claim.
3. **Claim memo.** `AsyncJobClaimMemo` is a `std::sync::Mutex<Option<StoredClaim>>` in `Context`, held for the whole synchronous claim.
   - It stores the field (execution or reconciliation), the full input and the result, an error included.
   - A later field with equal field and input returns the stored result. Any other field is refused with `ASYNC_JOB_DUPLICATE_CLAIM` before the engine runs.
   - A poisoned mutex refuses with `ASYNC_JOB_BUSY`.
   - Both claim fields are nullable (`AsyncJobClaimResult`, no `!`).
   - Its `Debug` prints no claim token.
4. **Single-job read.** `asyncJob` returns `AsyncJobDetail { job, attempts, transitions }`. Attempts and transitions are only on that wrapper, so `asyncJobs` (a list of `AsyncJob`) cannot reach them and has no N+1 read.
5. **`input` argument style.** Every executor operation and command takes one `input` object, as BE-04's operations do.
   - `asyncJobTargetState` also takes `input: AsyncJobClaimInput` (`asyncJobId`, `claimToken`), so its claim token is never a top-level argument.
   - The reads take plain arguments: `asyncJob(asyncJobId)`, `asyncJobs(filter, limit, after)`, `asyncJobKindSignals(kinds)` and `asyncJobExecutorActivity(since)`, as section 14.3 names them.
6. **Required and optional inputs.**
   - `requestId` is required on every command, so every command is idempotent.
   - `noEffectProven` is a required `Boolean!` on `failAsyncJob`.
   - `maxJobs` and `limit` are required; there is no silent default.
   - `completeAsyncJob` takes a required `entries` list, because `complete_on` takes one entry per job target for every outcome (section 10.3). A job without targets sends an empty list.
   - `resolveAsyncJobReconciliation` and `resolveAsyncJobReconciliationAsStaff` take nullable `entries`, because the evidence-argument table of section 13.3 requires them for `SUCCEEDED` and refuses them with every other outcome.
7. **What the operations return.** `waitAsyncJob`, `completeAsyncJob`, `failAsyncJob`, `releaseAsyncJob`, `resolveAsyncJobReconciliation` and both effect marks return the job (`AsyncJob`) after the write. Commands return `AsyncJobCommandResult`: the command id, its kind, outcome and outcome code, the job, the new job for a retry, and `replayed` (true when a repeated `requestId` returned the stored outcome).
8. **Signal reads.**
   - `asyncJobKindSignals(kinds)` with `kinds` absent or empty returns every kind of the registry or the table (section 15). A kind in neither returns no row.
   - `asyncJobExecutorActivity(since)` with `since` absent returns all activity.
   - Every count and every `source_order`, `applied_order` and `regressed_from_order` is the existing `BigInt` scalar.
9. **Test registry.** The tests inject their own registry through `Context::with_async_registry`. E1's test registries are private to E1's test module, and E2's tests need kinds that E1 has no reason to declare, such as a `ThothWorker` kind and a split-reconciler kind. Six kinds cover:
   - `CURRENT_STATE` and `REVISION_BOUND`;
   - a capability-gated kind;
   - an `UntrustedContent` kind;
   - a kind reconciled by `ThothWorker`;
   - a `ThothWorker` kind.

   `the_test_registries_validate_and_only_dissemination_worker_maps_to_an_executor` validates every test registry, as the production registry is validated.
10. **List position input.** The position input of `asyncJobs(..., after)` is named `AsyncJobListPositionInput`. The specification names only the argument `after`.
    - The first name, `AsyncJobCursorInput`, failed an existing guard in the first full-suite run: `graphql::metric_ingestion_lifecycle_tests::the_claim_context_is_reachable_only_through_the_protected_claim_result`, assertion "`type QueryRoot {` must not mention `cursor`". That Metrics guard forbids the words `cursor` and `manifest` anywhere in the `QueryRoot` block.
    - The type was renamed in `28be993`, inside the budget, and the guard passes unchanged. Section 14 records this for the reviewers.
11. **Access log.** The predicate is in `thoth-api`'s `async_job` module (`executor.rs`): `access_log_omits_body(body)`.
    - It decodes the body into `GraphQLRequest`, the handler's type, and omits the body when the query text contains `syncJob`.
    - A body it cannot decode is omitted when its lossy raw text contains `syncJob`.
    - `logger.rs` writes `[async job operation body omitted]` in place of the body. It keeps the `%{QUERY}xi` value's leading newline, so the log line keeps its shape.
    - Every other body goes through the unchanged formatting. A test compares it with a verbatim copy of the old code.
12. **FBE-2 and FBE-5** are fixed as section 9.2 records.

Readings the specification supports, recorded for the reviewers:

- R1. **Execution order of claim fields.**
  - juniper 0.16.2 executes top-level mutation fields serially, in document order.
  - Section 14.1 asks for "whichever claim field the executor runs first (not necessarily the first in document order)". The memo is order-independent, because it keys on the first field that runs.
  - The tests run each scenario as two documents, one per order. Each asserts which field ran first from the stored result, and checks that both orders occurred.
- R2. **The specification's escaped-name example.** The example written with the `c` of `claimAsyncJobs` as a JSON unicode escape still contains `syncJob` in its raw text, so on its own it does not show that the predicate decodes. The logger tests add two bodies whose escapes fall inside the marker (its `s`, then all of it). Their raw text does not contain `syncJob`, and they are still omitted.
- R3. **Residual for an undecodable body.** A body `GraphQLRequest` cannot decode, whose raw text hides `syncJob` behind a JSON escape, is logged. Section 16 states this rule (raw-text match for undecodable bodies). The handler refuses such a body too, so it runs no operation.
- R4. **Errors unreachable through the Release N API.** Five codes cannot reach a client:
  - `ASYNC_JOB_KIND_ADMISSION_CLOSED`, `ASYNC_JOB_PAYLOAD_VERSION_UNSUPPORTED` and `ASYNC_JOB_IDEMPOTENCY_CONFLICT` occur only on job creation, and no creation path exists.
  - `ASYNC_JOB_ISOLATION_INVALID` cannot occur, because every resolver runs the engine in its own engine-owned READ COMMITTED transaction.
  - `ASYNC_JOB_INTEGRITY_VIOLATION` would need a database fault.

  The mapping unit test covers all 17 codes, with their `type` and message. The other 12 reach a client in the database tests.
- R5. **SDL guard for `claimToken`.** Two older output types also carry `claimToken`: BE-04's `ClaimedDistributionJob` and the Metrics `MetricRollupDeltaClaim`. The guard is therefore stated for the new API:
  - the new root fields reach exactly the 13 new output types;
  - among them only `ClaimedAsyncJob` carries the token;
  - only `AsyncJobClaimResult` refers to it, only `MutationRoot` refers to that, and only the two claim fields return it.
- R6. **Root-field marker check.** The marker test checks both directions on the generated SDL, signature included. A root field whose name contains `syncJob` must take or return an `Async` type, and a root field that takes or returns one must contain `syncJob`. That is stronger than comparing names with a list.
- R7. **`cargo build -p thoth-client` alone fails at the base and at the head in the same way.** Built without the workspace's feature unification, `thoth-api` lacks the `backend` feature. Its lib then stops with unresolved backend-only paths (E0432, E0433): `crate::graphql` 20 times, and `diesel`, `crate::schema`, `crate::policy`, `crate::db` and `metric_ingestion`. That is 37 errors at the base and 37 at `28be993`, none of them in an E2 file. This predates E2. The internal client is therefore built through `cargo check --workspace` (section 7).
- R8. **FBE-5 uses `pg_input_error_info`,** which needs PostgreSQL 16 or later. CI (`build_test_and_check.yml`) and `docker-compose` use `postgres:17`. The production server's version is not visible from this repository; section 14 asks the reviewers to confirm it.

Deviations from the specification requiring authorization: NONE.

## 6. Database and migration effects

Migration added: NO. E2 adds no migration and changes no schema object. FBE-2 changes two read queries in `signals.rs` only.

## 7. API and compatibility effects

GraphQL/API changes: additive only. The schema gains:

- 5 queries: `asyncJob`, `asyncJobs`, `asyncJobKindSignals`, `asyncJobExecutorActivity`, `asyncJobTargetState`.
- 14 mutations: the 10 executor operations and the 4 operator commands.
- 40 declarations: 13 enums, 14 input objects and 13 object types.

Generated SDL, base against head: 629 lines added and none removed, in 13 hunks (5716 lines to 6345). Each side is `thoth-client/assets/schema.graphql` as `cargo check --workspace` wrote it, at `3464fdbf` and at `28be993`.

- The added lines are the 40 declarations and the 19 root fields, with their descriptions.
- An earlier head capture differed from the final one only in the list position input's name (decision 10).
- The 12 BE-04 declarations and 7 BE-04 fields are byte-identical to the base. They are pinned by SHA-256 in `the_be04_sdl_elements_are_byte_identical_to_the_base`.
- The base SDL contains no `Async`, `async` or `syncJob`.

Generated schema/client updates:

- The internal `thoth-client` build script writes the gitignored `thoth-client/assets/schema.graphql` from `create_schema()` and generates its client from `assets/queries.graphql`.
- `cargo check --workspace` builds it against the new schema: exit 0 at `28be993`, and its build script rewrote the schema file. Its queries use no new type, so the generated client is unchanged.
- `cargo build -p thoth-client` alone fails identically at base and head (R7).

Backwards compatibility: no existing type, field, argument, description or enum value changes. The diff removes no line.

Deprecations: none.

Cross-repository dependencies (section 18 conclusions, confirmed by the SDL diff):

- `thoth-pub/thoth-dissemination` REMAINS COMPATIBLE: the BE-04 API it uses is byte-identical. It adopts the generic executor API in step 4 under its own task.
- `thoth-pub/thoth-app` REMAINS COMPATIBLE: its codegen sees only additive superuser-only types.
- `thoth-pub/thoth-client` (standalone), `metrics-dashboard`, `metrics-widget`, `baboon`, `thoth-pyramid` and the internal `thoth-client` REMAIN COMPATIBLE: the schema change is additive only.
- No downstream task is needed for Release N.

## 8. Authorization and security

Authorization paths changed: new resolvers only. No existing guard changed.

- The 10 executor operations and `asyncJobTargetState` use `authorize_executor`: the `require_dissemination_worker` guard, then the executor map.
- The engine's domain check then applies, on claims and on every token-fenced call.
- The 4 commands and 4 reads use `require_superuser`.

Roles/scopes involved:

- `DISSEMINATION_WORKER` gains exactly the 11 executor fields, for kinds whose executor or reconciler domain is `Dissemination`, and no command or read.
- `SUPERUSER` gains the 4 commands and 4 reads, and no executor operation (ADR-0008).
- No role variant was added. The publisher-scope rows of the `thoth-api/AGENTS.md` section 7 matrix are "not applicable": every Release N operation is unscoped. The tests prove that publisher-scoped roles are refused.

Negative authorization tests: every one of the 19 fields is called by each of these callers:

- no authentication;
- an authenticated user with no role;
- each of the four publisher-scoped roles, and all four together;
- `METRICS_INGEST_SERVICE` and `METRICS_READ_SERVICE`;
- the wrong privileged role: `SUPERUSER`, and `SUPERUSER` with every other role, for executor operations; `DISSEMINATION_WORKER`, and `DISSEMINATION_WORKER` with every role except `SUPERUSER`, for commands and reads.

Each refusal must be the `NO_ACCESS` error "Unauthorized" on that field's path, with null data, and must leave every async table unchanged. The correct role succeeds.

Secret or personal-data handling:

- A claim token is returned only by the two claim fields.
- `Debug` of the memo prints no token.
- The access log omits every body that names an async field (section 16).
- Both `reason` fields say: "Do not paste provider diagnostics or personal data into it".
- The engine logs no payload, checkpoint, evidence, error detail or token (E1).

Security limitations:

- Section 16's residuals: a `GET /graphql` request line is logged as before, and an undecodable body whose raw text hides the marker is logged (R3).
- Section 14.1's client-contract residual: a request with a claim beside a failing non-null field can lose sight of its claimed jobs until lease recovery.

## 9. Tests and checks

All checks ran locally:

- on PostgreSQL 17.10 (port 5433, test database `thoth_test`) and Redis;
- with `CARGO_INCREMENTAL=0`;
- with Rust 1.97 (the local stable), except where a check names another toolchain.

### Formatting

Command:

```text
cargo fmt --all -- --check
```

Result:

```text
exit 0 (at 28be993, rustfmt 1.9.0-stable of Rust 1.97.0)
```

### Unit and database tests (`thoth-api`)

Command:

```text
cargo test -p thoth-api --features backend
```

Result:

```text
test result: ok. 2526 passed; 0 failed; 4 ignored; 0 measured; 0 filtered out; finished in 1007.91s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.35s
test result: ok. 0 passed; 0 failed; 8 ignored; 0 measured; 0 filtered out; finished in 0.00s
exit 0
```

- This ran at `28be993`, from 14:15 to 14:35 UTC: the library tests, `tests/graphql_permissions.rs` and the doc tests, in that order.
- The library has 2530 tests: E1's 2495 and E2's 35 (the 25 API tests, the 5 memo unit tests and the 5 carry tests). The 4 ignored tests are the same as at the base: three evidence tests and E1's 19.0 harness.
- `model::async_job` passes 163 tests: E1's 158 and the 5 carry tests.
- Every existing guard passes. Among them are the Metrics guard of decision 10, both BE-04 source guards and the two `the_resolvers_authorize_before_reaching_a_coordinator` guards. The only changed assertion is the 7-to-11 count.
- `graphql_permissions` (13 passed) and the doc tests (8 ignored) match the base.
- The first full run, at `1c30a24`, ended `FAILED. 2525 passed; 1 failed; 4 ignored` on the Metrics guard (decision 10), and cargo stopped before the other targets. `28be993` fixed the name, and this run passed.

### API server tests (`thoth-api-server`, the access-log rule)

Command: the `thoth-api-server` part of the workspace run below (`cargo test --workspace --exclude thoth-api --no-fail-fast`, at `28be993`).

Result:

```text
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
```

- The 11 tests are the 5 existing ones and the 6 access-log tests of section 16 (`logger::tests`).
- At `b63bab9`, the 6 logger tests also passed alone (`6 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out`).

### Lint/static analysis

Command:

```text
cargo clippy --all --all-targets --all-features -- -D warnings
```

Result:

```text
Rust 1.97.0, at 28be993:
Finished `dev` profile [unoptimized + debuginfo] target(s) in 2m 27s
exit 0

Rust 1.100.0 (0ef1bc386 2026-11-09), the newest stable, at 28be993, own target folder:
cargo +1.100.0 clippy --all --all-targets --all-features -- -D warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) in 4m 46s
exit 0
```

- The only warning in the clippy, check and build runs is cargo's future-incompatibility note for the dependency `proc-macro-error2 v2.0.1`. It predates E2 and is not a lint.
- The first newest-stable attempt kept its target folder on disk. Its disk guard stopped it at 3.1 GB free, after 477 dependency units and before any workspace crate, so it gave no result.
- The second attempt kept the target folder in RAM (`/dev/shm`, 3.0 GB at the end), and free disk stayed at 5.5 GB. The toolchain and both target folders were removed afterwards.

### Workspace

Commands:

```text
cargo check --workspace
cargo test --workspace --exclude thoth-api --no-fail-fast
```

Result:

```text
cargo check --workspace: Finished `dev` profile [unoptimized + debuginfo] target(s) in 2m 05s
exit 0 (at 28be993)

thoth (lib)            test result: ok. 0 passed; 0 failed; 0 ignored
thoth (bin)            test result: ok. 40 passed; 0 failed; 0 ignored
thoth-api-server       test result: ok. 11 passed; 0 failed; 0 ignored
thoth-client           test result: ok. 4 passed; 0 failed; 0 ignored
thoth-errors           test result: ok. 15 passed; 0 failed; 0 ignored
thoth-export-server    test result: ok. 144 passed; 0 failed; 0 ignored
Doc-tests              thoth 0, thoth_api_server 0, thoth_client 6 passed, thoth_errors 0, thoth_export_server 2 passed
exit 0 (at 28be993)
```

- `--exclude thoth-api` follows E1's precedent, for disk. The workspace selection unifies features differently from `-p thoth-api --features backend`, so it builds its own copy of every test executable. `thoth-api`'s two (about 1.8 GB) would have taken free disk below 3 GB: the run's lowest point was about 3.4 GB without them.
- The `thoth-api` part of `cargo test --workspace` is the full suite above. Workspace feature unification gives `thoth-api` its `backend` feature, so it is the same test set.
- Against E1's run, the only change is `thoth-api-server`, from 5 tests to 11: the 6 access-log tests.
- The first attempt did not compile: `thoth-api` found none of the async variants in `ThothError`.
  - The cause was two stale `thoth-errors` build units in the shared target folder. They were built on 2026-11-14 at 03:28 from a pre-E1 `thoth-errors/src/lib.rs`, and cargo still counted them as fresh, because the source file's mtime (2026-11-13 22:15) is older than they are.
  - Those two units were deleted (build output only; `thoth-errors` is identical to `HEAD`). Cargo then rebuilt them from the current source, and the run passed.
  - No other check used them. The full suite, `cargo check --workspace` and both clippy runs compiled code that uses the async variants, so they built against a current `thoth-errors`.

### Other required checks

These ran at `28be993` with this report and the `CHANGELOG.md` line in the worktree, using a read-only script:

```text
git diff --check 3464fdbf..HEAD; git diff --check                         -> exit 0, exit 0
git diff 3464fdbf..HEAD -- '*distribution*' '*publisher_service*' \
  '*service_configuration*' src/bin/commands/publisher_services.rs \
  thoth-api/migrations | wc -l                                              -> 0
added lines containing distribution_job or publisher_service_configuration -> 0
git diff 3464fdbf..HEAD -- Cargo.toml Cargo.lock '*/Cargo.toml' | wc -l    -> 0
git diff 3464fdbf..HEAD -- docs/engineering/decisions .github \
  thoth-export-server src thoth-errors thoth-api-server/src/lib.rs \
  thoth-api-server/src/main.rs <E1 report> | wc -l                          -> 0
git grep 1095981390|1095981391|1095981392 outside async_job/mod.rs          -> E1's report only (no code)
non-ASCII characters in the E2 Rust files                                  -> none
```

The BE-04 SDL elements are pinned by SHA-256, computed on the base SDL, in `the_be04_sdl_elements_are_byte_identical_to_the_base`, which passes (section 7).

### 9.1 Spec 19.2 to tests (E2)

E2 adds 41 tests:

- 25 in `thoth-api/src/graphql/async_job_tests.rs`;
- 5 claim-memo unit tests in `thoth-api/src/graphql/types/async_job.rs`;
- 6 access-log tests in `thoth-api-server/src/logger.rs`;
- 5 carry tests in `thoth-api/src/model/async_job/tests.rs` (section 9.2).

35 of them map to the E2 sentences of 19.2. `debug_never_prints_a_claim_token` supports section 16, and the 5 carry tests support section 15 and the E1 review carry, not a 19.2 sentence. All pass.

In the table, tests in brackets are listed under another row too, and are counted once, in the row where they appear without brackets.

| 19.2 sentence (E2 part) | Tests | Count |
|---|---|---|
| API and authorization: every query and mutation for no authentication, wrong role (publisher-scoped, `METRICS_*`, `SUPERUSER` for executor operations, `DISSEMINATION_WORKER` for commands and reads) and the correct role | `with_the_empty_production_registry_the_worker_claims_nothing_and_every_other_caller_is_refused` (the inert state, against `PRODUCTION_REGISTRY`); `both_claim_fields_admit_only_the_dissemination_worker`; `every_token_fenced_operation_admits_only_the_dissemination_worker` (9 fenced fields; each refusal leaves every async table byte-identical); `every_operator_command_admits_only_the_superuser`; `every_operator_read_admits_only_the_superuser_and_pages_newest_first` | 5 |
| API and authorization: a dissemination executor cannot claim or write a `ThothWorker` or other-domain kind, even with a valid token | `a_dissemination_executor_cannot_claim_or_write_another_domains_job_even_with_a_valid_token` (a claim naming a `ThothWorker` kind, alone or beside a `Dissemination` kind, and a reconciliation claim of a kind that `ThothWorker` reconciles, are refused whole with `ASYNC_JOB_KIND_NOT_AUTHORIZED`; all 9 fenced fields are refused, with nothing written, under three live `ThothWorker` tokens, which stay live for their own domain); `the_executor_identity_comes_from_the_role_map_and_the_token_never_from_input` (an `UntrustedContent` kind is refused because the map's risk class is `Trusted`; a kind requiring the `dissemination` capability is claimed; `claimedBy` is stored as the executor name and `executorPrincipal` as the token's user, also on the activity read); `the_test_registries_validate_and_only_dissemination_worker_maps_to_an_executor` | 3 |
| API and authorization: operator command audit rows | `every_operator_command_writes_one_audit_row_and_a_repeat_replays_it_without_another` (each command, a `REFUSED` cancel included, writes exactly one row with its kind, request id, the token's principal, reason, job, outcome, code and result job; a refusal changes no job; a repeat returns the stored decision with `replayed: true` and no new row; the same request id for another job is `ASYNC_COMMAND_REQUEST_CONFLICT` and writes nothing) | 1 |
| API and authorization: SDL guard (new types exist; `claimToken` only on the claim result; BE-04 SDL unchanged; no generic `WORKER`/`SERVICE` role or field) | `the_generated_sdl_declares_exactly_the_forty_new_async_job_types`; `only_a_claim_result_carries_a_claim_token_and_only_the_claim_fields_return_one` (R5); `the_be04_sdl_elements_are_byte_identical_to_the_base`; `the_new_sdl_names_no_generic_worker_or_service_role_or_field`; `every_async_enum_mirrors_its_engine_enum_label_for_label`; `the_reason_fields_warn_staff_and_the_claim_input_names_no_executor_identity` | 6 |
| API and authorization: error codes | `every_async_error_maps_to_its_stable_graphql_type_and_its_display_message` (all 17); `every_reachable_error_code_reaches_the_client_with_its_type_and_message` (`NOT_FOUND`, `STALE_CLAIM`, `KIND_UNKNOWN`, `KIND_NOT_AUTHORIZED`, `PAYLOAD_INVALID`, `LEASE_EXPIRED`, `TRANSITION_REFUSED` with its reason, `EFFECT_WINDOW_INSUFFICIENT`, `EVIDENCE_INVALID`, and `BUSY` on four fields from a pool that cannot connect); `a_bigint_order_reaches_the_engine_untruncated_and_only_the_scalar_refuses_malformed_text` (an `order` outside `bigint` but inside `i128` reaches the engine untruncated; `completeAsyncJob` gets the engine's `ASYNC_JOB_EVIDENCE_INVALID`, with nothing written and the claim live, and the staff command gets T18's committed `REFUSED` with `EVIDENCE_INVALID`; non-canonical text, or a value outside `i128`, is a GraphQL input error before the engine runs, with no command row); [`DUPLICATE_CLAIM` and `ASYNC_COMMAND_REQUEST_CONFLICT` in the rows below and above] | 3 |
| Accepted regression (E2 part) | `an_accepted_regression_is_recorded_through_the_api_and_cleared_only_by_ordinary_progress`. It covers: T6x; an executor reconciler's regressing `SUCCEEDED` refused with nothing written and the claim live; `ACCEPT_REGRESSION` with valid applied evidence ending `SUCCEEDED` with `REGRESSION_ACCEPTED`, holds released, and the replaced order and fingerprint and `regressedAt` shown; a forward write below the pre-acceptance order keeping all three columns; one above it clearing them; an equal order with the pre-acceptance fingerprint clearing them on a second target | 1 |
| Access log: an async request's body is omitted; a BE-04 or unrelated body is logged exactly as before | `a_body_naming_an_async_job_field_is_omitted`; `every_other_body_is_logged_exactly_as_before` (two BE-04 bodies, ordinary bodies, a variable value containing `syncJob`, malformed JSON, non-UTF-8, empty, a positional array without the marker); `the_omitted_body_text_is_the_specified_placeholder` | 3 |
| API additions: a duplicated response key and an aliased identical claim field return the one stored result and claim once | `a_duplicated_claim_key_and_an_aliased_identical_claim_return_one_stored_result_and_claim_once`; `the_first_claim_field_run_claims_and_later_fields_answer_from_the_memo` | 2 |
| API additions: a field with different arguments is refused with `ASYNC_JOB_DUPLICATE_CLAIM`, claims nothing, and nulls only its own field, for both execution orders | `a_claim_field_with_other_arguments_is_refused_whichever_claim_field_runs_first`; `a_claim_and_a_reconciliation_claim_in_one_request_claim_once_whichever_runs_first` (both orders each, R1) | 2 |
| API additions: an identical field after a failed claim returns the stored error without a second claim | `an_identical_claim_field_after_a_failed_claim_returns_the_stored_error_without_claiming` (a real `ASYNC_JOB_BUSY` from a cancelled lock wait, section 9.3); `a_failed_claim_is_stored_and_repeated_without_a_second_claim`; `the_memo_mutex_is_held_for_the_whole_claim_and_a_later_field_waits_for_it`; `a_poisoned_memo_refuses_every_later_claim_field_as_busy_without_a_claim` | 4 |
| API additions: both claim fields are nullable in the generated SDL | `only_the_two_claim_fields_are_nullable` (every other new root field is non-null) | 1 |
| API additions: the access-log rule omits an escaped field name, an aliased field and a positional array, and logs a BE-04 body exactly as before | `a_json_escaped_field_name_is_matched_on_the_decoded_query_text` (R2); `a_positional_array_body_is_matched_on_its_query_text`; [`a_body_naming_an_async_job_field_is_omitted` for the alias and the fragment; `every_other_body_is_logged_exactly_as_before` for BE-04] | 2 |
| API additions: every new `QueryRoot` and `MutationRoot` field contains `syncJob` and no existing one does | `every_new_root_field_names_an_async_job_and_no_other_root_field_does` (R6) | 1 |
| API additions: a malformed body containing `syncJob` is omitted | `an_undecodable_body_containing_the_marker_is_omitted` (truncated JSON, a batch array, a non-string `query`, non-UTF-8) | 1 |
| API additions: `policy.rs` still passes both BE-04 source guards | existing, unchanged: `graphql::distribution_job_tests::the_worker_guard_is_one_explicit_predicate_with_no_inheritance_or_composition`, `graphql::distribution_job_tests::no_generic_framework_universal_queue_or_shared_worker_convention_is_introduced` | (existing) |
| Regression: the full `thoth-api` suite passes except the two 7.11 assertions; `cargo test --workspace` passes | the full suite (section 9); the one changed assertion is the `require_superuser()` count in `graphql::metric_registry_tests::the_resolvers_authorize_before_reaching_a_coordinator`, 7 to 11; E1 changed the catalog inventory | (suite) |

Total: 5 + 3 + 1 + 6 + 3 + 1 + 3 + 2 + 2 + 4 + 1 + 2 + 1 + 1 = 35 tests mapped to the E2 sentences of 19.2. The 36th E2 test, `debug_never_prints_a_claim_token`, supports section 16.

**Test strength (temporary mutation check).** To show the tests fail when the property breaks, four mutations were applied locally to E2's own code and never committed:

1. `SUPERUSER` admitted as an executor.
2. `DISSEMINATION_WORKER` admitted to the commands.
3. `DISSEMINATION_WORKER` admitted to `asyncJob`.
4. The memo answering nothing.

`cargo test -p thoth-api --features backend --lib -- graphql::async_job_tests` then failed 9 tests, each on its own property:

- the empty-registry, claim-field and fenced-operation authorization tests (mutation 1);
- the command and read authorization tests (mutations 2 and 3);
- all four memo tests (mutation 4).

`git checkout` restored the four files, and `git status` was clean afterwards.

### 9.2 E2 carry items (`E2-carry-from-E1.md`)

| Item | Disposition |
|---|---|
| 1. FBE-2: `kind_signals` listed kinds with `SELECT DISTINCT kind` over the never-pruned table, and returned `regressed_targets` unbounded | Fixed in `e968e35` under the G5 amendment. See below. Tests: `carry_fbe2_signals_list_every_kind_of_the_registry_or_the_table`, `carry_fbe2_regressed_targets_are_counted_exactly_and_listed_up_to_the_bound` |
| 2. FBE-5: (a) `json_measure` turned every non-busy database error into a jsonb refusal; (b) a nested engine transaction logged the generic line | Fixed in `d55d399` under the G5 amendment. See below. Tests: `carry_fbe5_jsonb_refusals_are_the_data_exception_class_only`, `carry_fbe5_any_other_database_error_is_an_integrity_violation_not_a_refusal`, `carry_fbe5_a_nested_engine_transaction_runs_nothing_and_keeps_its_code` |
| 3. FBE-6: a REVISION_BOUND READ_ONLY job with zero targets | Not E2 work: it is decided when the first REVISION_BOUND kind is registered. No change |
| 4. Readings 18 and 22 | Recorded as clarifications, with no code change. Reading 18: T15 reconciliation candidates are ordered by `available_at, async_job_id`, the only order the 7.2 reconciliation index serves. Reading 22: recovery reads `now` just before its non-waiting `SKIP LOCKED` row locks; both E1 review legs ruled it sound against spec 9.3. OPE-6 (move reading 22 beside reading 18 in E1's report) is not done, because E1's report is outside E2's budget |
| 5. OPE-3 | Recorded, no change. The 24 h terminal counts can undercount at the window's edge by the length of the terminal transaction (`updated_at` is the transaction start, `terminal_at` is later). An exact count needs a `terminal_at` index (a schema change) or an unbounded scan |
| 6. OPE-1 / G6 | Not E2 work: migration revert guards, 19.0 item 2 on the integrated chain, the migration directory rename and the `schema.rs` and `CHANGELOG.md` conflicts belong to G6 |
| 7. CI note (newest stable clippy) | Done before the final push (section 9) |

**FBE-2 (`e968e35`).**

- The kind list is now a recursive loose index scan over `async_job_kind_status_idx`: one index probe per distinct kind. It still lists every kind present in the table (terminal-only kinds included) or in the registry, as section 15 requires.
- Each kind now carries an exact `regressed_target_count`, exposed as `regressedTargetCount`, and at most 100 `regressed_targets` (`REGRESSED_LIST_LIMIT`), ascending by target key. The list is read through `async_effect_target_state_regressed_idx`, like the blocked list.

The plan check was run on a scratch database (`e2_explain`). It held:

- 300,000 terminal jobs over about 347 days;
- 3,000 non-terminal jobs;
- 50,000 applied-state targets, 150 of them regressed;
- 7 kinds.

`EXPLAIN (ANALYZE)` of the old and new kind lists:

```text
old: Unique -> Gather Merge -> Unique -> Parallel Index Only Scan using async_job_kind_status_idx (actual rows=101000 loops=3); Execution Time: 17.882 ms
new: CTE Scan on kinds (actual rows=7) -> Recursive Union -> Limit -> Index Only Scan using async_job_kind_status_idx (actual rows=1 loops=7), Index Cond: (kind > k.kind); Execution Time: 0.254 ms
regressed list: WindowAgg (Run Condition: row_number() <= 100) -> Index Scan using async_effect_target_state_regressed_idx (actual rows=150); Execution Time: 0.679 ms
```

**FBE-5 (`d55d399`).**

- (a) Diesel exposes no SQLSTATE. `json_measure` therefore first asks `pg_input_error_info($1, 'jsonb')`, which reports a refused input's SQLSTATE without raising.
  - Only a data exception (class 22) is a refusal (`ASYNC_JOB_PAYLOAD_INVALID`, or an evidence fault).
  - A refusal in any other class, and every other database error, is `ASYNC_JOB_INTEGRITY_VIOLATION`.
  - The busy class stays `ASYNC_JOB_BUSY`.
- (b) A nested `with_engine_transaction_on` still runs nothing and keeps `ASYNC_JOB_INTEGRITY_VIOLATION`, but now logs its own fixed line.

### 9.3 Concurrency-test method

E2 has no racing writers of its own. The ordering-sensitive behaviour is the claim memo inside one request. The method:

- **Real requests.** Every API test runs real GraphQL requests through the production `Schema` and resolvers, with a fresh `Context` per request (as production builds one), against the shared test database. Engine fixtures use a dedicated `PgConnection` and E1's `_on` functions in engine transactions.
- **Both execution orders.** The memo tests run one document per order and identify the field that ran first from the stored result. Each test then asserts the set of first-run fields, so a test fails if an order was not exercised (R1).
- **The failed-claim test.** `an_identical_claim_field_after_a_failed_claim_returns_the_stored_error_without_claiming` makes a real claim fail:
  - A blocker connection holds the claim's per-kind advisory lock (`pg_advisory_lock(NS_KIND, hashtext('API_EFFECT'))`).
  - An observer thread polls `pg_locks` until the claim's backend waits on that lock. The poll is bounded at 60 s, yields between polls and never sleeps.
  - The observer then cancels the backend with `pg_cancel_backend`, so the claim fails with `ASYNC_JOB_BUSY`.
  - The test asserts: the identical field returns the stored `BUSY` without claiming; the different field is `ASYNC_JOB_DUPLICATE_CLAIM`; the job is still `PENDING` with no attempt; the async tables are byte-identical; the `pg_stat_database.deadlocks` counter is unchanged. A new request then claims normally.
- **Memo unit tests** use threads and channels to show the mutex is held for the whole claim. A later field waits for it, and a panic poisons it.
- **No sleeps.** No E2 test sleeps. `grep -n sleep` over the E2 test code finds only three comments saying so. The tests share the suite's test-database lock.

The failed-claim test takes the kind lock through the async_job module's own `NS_KIND` constant, in test code only. The advisory namespace values themselves still appear only in `thoth-api/src/model/async_job/mod.rs` (acceptance criterion 5).

## 10. Manual verification

Environment: local only.

Steps:

1. The FBE-2 plan check on the populated scratch database `e2_explain` (section 9.2).
2. The SDL captures at the base and the head (`cargo check --workspace` writes `thoth-client/assets/schema.graphql`), with their diff.
3. The SHA-256 pins of the BE-04 SDL elements, computed on the base SDL.

Observed result: as recorded in sections 7 and 9.2.

## 11. CI

CI status: NOT AVAILABLE. No PR exists yet; the orchestrator's draft PR into `feature/worker` starts `build-test-and-check`, `check-changelog`, `run-migrations` and the staging image check. The newest-stable clippy run of section 9 stands in for CI's `lint` job until then.

## 12. Rollout and rollback

Initial state after merge: inert.

- The production registry is empty and nothing creates jobs.
- `DISSEMINATION_WORKER` gets an empty claim result with every requested kind in `skippedKinds`.
- Every other caller of an executor operation is refused before the engine runs.
- The commands and reads find no job.

Activation required: none in Release N. A kind and its creation path come with step 4.

Feature flag/configuration: none.

Migration sequence: none (E2 adds no migration).

Rollback/disable procedure: revert the E2 merge, or redeploy the previous image. No data depends on E2.

Monitoring required: none until a kind exists. After that, use `asyncJobKindSignals` and `asyncJobExecutorActivity`.

## 13. Known limitations and deferred work

- The section 14.1 client-contract residual: a claim must be the only field of its request (step 4's executor contract).
- The section 16 residuals: the `GET /graphql` request line, and R3.
- R4: five error codes cannot reach a Release N client. They are covered by the mapping unit test.
- An observation from the FBE-2 plan check, outside FBE-2's scope and unchanged: the blocked-job signal queries (S4 and S5 in the evidence) join the holds to `async_job_target` by a merge join that walks `async_job_target_target_key_idx`.
  - On the scratch database that was 43,776 index rows for 400 holds, 30 ms and 22 ms.
  - Their cost grows with the target table, which is never pruned, rather than with the holds.
  - It is bounded only by history retention or a reshaped query. It is recorded for the reviewers and the history-retention specification.
- Section 23 of the specification applies unchanged, including publisher-scoped reads and the worker runtime, which come in later steps.

## 14. Unresolved issues

- For the reviewers: decision 10 renamed E2's list position input to `AsyncJobListPositionInput`, so that the existing Metrics guard `the_claim_context_is_reachable_only_through_the_protected_claim_result` passes unchanged.
  - The guard forbids `cursor` and `manifest` anywhere in `QueryRoot`.
  - No existing guard was edited, and the final suite has no failing guard.
  - If the reviewers read handoff section 12 ("an existing guard ... fails") as requiring a HOLD even for a failure that E2's own naming caused and fixed in-budget, this is where it happened.
- For the reviewers: R8. FBE-5's `pg_input_error_info` needs PostgreSQL 16 or later in every environment that runs this code. CI and `docker-compose` use 17; the production version needs confirming outside this repository.
- For the orchestrator: build output deleted beyond the old test executables.
  - Two stale `thoth-errors` build units were deleted from `target/debug`, so that cargo rebuilt them from the current source (section 9, Workspace).
  - They are build output, not repository files, and no other file was deleted.
  - If the "delete, move or rename files: NO" row of handoff section 6 also covers build output, this went beyond the handoff's one stated exception, the old test executables.

## 15. Agent self-assessment

The agent may identify risks but may not approve the task.

Suggested review focus:

- The executor boundary:
  - `authorize_executor` and `ExecutorProfile::worker_claim`, which carry the caller's domain into every fenced call;
  - the engine prelude order in E1;
  - that no executor-identity field is input.
- The claim memo: equality on the whole input, the stored error, the mutex scope, poisoning, and authorization before the memo.
- The access-log predicate and its six tests. Every other body must be logged byte-for-byte as before.
- The SDL guards: the 40 declarations, the reachability claim of R5, and the BE-04 pins.
- FBE-2's loose scan: every kind is still listed, terminal-only kinds included. FBE-5's class-22 rule and its PostgreSQL 16 dependency (R8).
- Decision 10 and section 14.
