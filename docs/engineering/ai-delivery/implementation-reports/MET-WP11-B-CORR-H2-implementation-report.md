# MET-WP11-B-CORR-H2 Implementation Report

## 1. Repository state

Owning GitHub issue: [#988](https://github.com/thoth-pub/thoth/issues/988)
Parent programme-integration task: [#981](https://github.com/thoth-pub/thoth/issues/981)
Outer integration PR: [#984](https://github.com/thoth-pub/thoth/pull/984) (`feature/metrics` -> `develop`), observed OPEN / DRAFT / UNMERGED at N/D before the branch was created and again at the pre-commit re-check (2026-10-29T22:07:00Z, head `97de027d…`, base `b44303c2…`, last updated 2026-10-28T19:11:04Z); live state is held by GitHub
Repository: `thoth-pub/thoth`
Programme: Thoth Metrics, WP11-B — finite pre-B source correction, H2 second of H1 -> H2 -> H3 (#981 Amendment 8 as corrected by Amendment 8-C1; #988 as corrected by its correction 1)
Workflow: PROGRAMME_INTEGRATION
Risk: HIGH (connection admission, lock timing and statement timeout of the managed-DRIVER batch inside the CRITICAL B integration); parent stage CRITICAL
Base branch: `feature/metrics`
Authorized base commit: `97de027da15dcb0fd1578b37e313365ef5834ffd` (N, tree `a6bd4218d665538c659e61a01ce29a19e8365f30`, ordered parents `59d43a167089836d9db58198792f444d15ce6355` (M2) and `ef35d97007a1af15114488cdbd4c8bf2103d8a7c` (reviewed H1 R1 head))
Actual base commit: `97de027da15dcb0fd1578b37e313365ef5834ffd` (tree `a6bd4218d665538c659e61a01ce29a19e8365f30`; `git merge-base --is-ancestor` of N over the branch head holds, and `feature/metrics` was still N at 2026-10-29T22:04:27Z)
Frozen `develop` D: `b44303c214498baf76c9d9a0a7cab374377f9cbe` (tree `050e1e2954b518d62988e31d74a7d9c280b8822c`), unchanged at the same checks; no `develop` commit is imported
PR target: `feature/metrics`
Programme integration branch: `feature/metrics`
Task branch: `feature/metrics--wp11-b-corr-h2`, created from exact N at 2026-10-29T21:02:45Z in the isolated worktree `/home/claude/wt-h2`
Head commit: `5f81265bc44347ed9f6816a6036402f86ff3ead2` (tree `6d58af36b45cf588320f2395392edbd8de996dc2`) carries the complete source, test, contract and changelog change; the commit that adds this report and the changelog PR link follows it. A commit cannot contain its own SHA, so the exact current head is the one recorded on the pull request, and each independent exact-head source review binds to that SHA.
Pull request: [#993](https://github.com/thoth-pub/thoth/pull/993), `feature/metrics--wp11-b-corr-h2` -> `feature/metrics`; opened as a draft at 2026-10-29T22:52:56Z and left draft; live review, readiness and merge state is held by GitHub
Expected branch deletion after merge: YES — but only through the stage-terminal A2 inventory, independent review, exact-name retirement authorization, guarded retirement and completion receipt that Amendment 8 item 7 defers it to; `delete_branch_on_merge` was `false` at 2026-10-29T22:55:09Z, so a merge does not delete the ref.
Final programme PR required: YES — the existing outer PR #984, separately gated.
Implementing model: Claude Fable 5.1 (`claude-fable-5-1`, as the session reports its served model), the implementing agent of Claude Code cloud session `session_01RFKP5PX7vAhfXMtTRFPorB`; the exact deployed build is not exposed.
Prior involvement, disclosed: none. This is a fresh instance started for H2 with no earlier role in the H2 specification, its specification review, or any H1 implementation, review or R1 work; it holds no record of earlier sessions. The independent exact-head source review is arranged separately, cross-family, and this agent issues no approval.
Reasoning level: maximum practical, as the handoff requires; the configured effort setting is not exposed to this agent.

### 1.1 Approval and authorization records

| Gate | Record | Outcome |
|---|---|---|
| Original task specification | #988 body, SHA-256 `d31c2701967364b90ffc295483b0a2578ff1e79e1b0831bb2b07c4b204bd01e5` | governs technical scope |
| H2 correction 1 (F-3: lock-timeout code, retry behaviour, idle reservation limits) | #988 [`6385431141`](https://github.com/thoth-pub/thoth/issues/988#issuecomment-6385431141), SHA-256 `84f9feb54b4c46e2cb2f3fa18df5f8e5bad96f8e8c3cb65bdbc3ad6d66c13570` | supersedes only the passages it names |
| Parent specification | #981 Amendment 8 [`6370994558`](https://github.com/thoth-pub/thoth/issues/981#issuecomment-6370994558) (`18079c28…bf23c`) and Amendment 8-C1 [`6385434946`](https://github.com/thoth-pub/thoth/issues/981#issuecomment-6385434946) (`13e758de…4636e`) | corrected eleven-record set |
| Independent CRITICAL specification review | #981 [`6385770904`](https://github.com/thoth-pub/thoth/issues/981#issuecomment-6385770904) | `APPROVED` (attributed) |
| CTO specification approval, eleven hashes | #981 [`6385772335`](https://github.com/thoth-pub/thoth/issues/981#issuecomment-6385772335) | `APPROVED - BOUNDED CORRECTED SPECIFICATION ONLY` |
| H1 terminal merge evidence, source of base N | #987 [`6478614360`](https://github.com/thoth-pub/thoth/issues/987#issuecomment-6478614360) | N = `97de027d…` |
| CTO H2 implementation authorization and issued handoff | #988 [`6508149914`](https://github.com/thoth-pub/thoth/issues/988#issuecomment-6508149914), posted by `ja573` at 2026-10-29T20:57:12Z, body SHA-256 `ada2cd986ffaf830ab28f0629335d8fbb3c79965ebea972dcd8ec48083c46716` | bounded actions used in section 4.2 |

The authorization comment names the approved proposed handoff file by digest (`MET-WP11-B-CORR-H2-proposed-handoff.txt`, 14,542 bytes, SHA-256 `8587f214c2544791011285fb1b34558963d13539b6aef8b0f03dbaf158c7388e`; the copy read by this agent has exactly that digest) and carries the issued handoff in one fenced block (14,279 bytes, SHA-256 `0134cb18a10fff5bf3dde7da3d8f2efe6570c62c0de8867743aecf17bf407238`). A line diff of the block against the proposed file shows only the resolved pending-status wording: the six-line "PROPOSED - NOT ISSUED" preamble is removed, preflight bullet 1 now names this comment and the handoff digest, and section 6 is headed "granted" with the two sentences stating that the YES entries are granted and that NO entries and unlisted mutations stay denied. No path, budget, acceptance criterion or action differs.

All eleven approved bodies were re-fetched and re-hashed (SHA-256 over the decoded REST body as UTF-8 without an added newline) before any mutation (2026-10-29T20:59:33Z to 21:00:13Z) and again before commit (2026-10-29T22:07:00Z); all eleven matched the CTO manifest both times.

### 1.2 Preflight, performed before any source or branch mutation

1. Authorization comment `6508149914` exists on #988, is the one control-authored authorization record, names the handoff digest `8587f214…` and grants the section 6 actions (section 1.1). **PASS**
2. `feature/metrics` = N `97de027d…` / tree `a6bd4218…` with ordered parents `59d43a16…`, `ef35d970…`; `develop` = D `b44303c2…` / tree `050e1e29…`; Sphinx `feature/metrics` `fa4b0f88751ee73617a0697d7fd6b1be221b6b74` and `develop` `e97444c20fb5626f9233397ed627a54d1a2a46d2` unchanged (re-read 2026-10-29T22:55:09Z). **PASS**
3. #988 open, two comments (correction 1 and the authorization), title and scope unchanged; #984 OPEN / DRAFT / UNMERGED at N/D; eight open PRs, none from a slice head and none targeting `feature/metrics`; no other H2 PR. **PASS**
4. Fail-closed namespace preflight (root `AGENTS.md` section 5.1): the slice namespace `feature/metrics--*` contained only `feature/metrics--wp11-b-corr-h1` at `ef35d970…`; `feature/metrics--wp11-b-corr-h2` and any descendant were absent; no ref was deleted, renamed or moved. **PASS**
5. Clean isolated worktree: `git worktree add /home/claude/wt-h2 -b feature/metrics--wp11-b-corr-h2 97de027d…` after fetching `origin/feature/metrics`; the main checkout's branch was not touched; `git status --short` empty before edits. **PASS**
6. Fresh disposable PostgreSQL 17 and Redis before any code: a `postgres:17` container (`PostgreSQL 17.11 (Debian 17.11-1.pgdg13+2)`, created 2026-10-29T21:01:46Z) listening on `127.0.0.1:54329` only, with a `thoth_test` database created empty for this task, and a local `redis-server 7.0.15` on `127.0.0.1:63799`. The container's locally installed PostgreSQL 16 was not used as a server. **PASS**
7. All seven authorized existing paths read at N; the report path was absent. **PASS**
8. Workflows and `.github/scripts/classify_ci_changes.py` at N re-inspected and found as the handoff's section 7 describes (`build-test-and-check` with `postgres:17` and `redis:alpine` service containers running `cargo build -vv`, `cargo test --workspace -vv`, `cargo clippy --all --all-targets --all-features -- -D warnings` and `cargo fmt --all -- --check`; `check-changelog`; `run-migrations` gated on `run_migrations`; `publish-to-dockerhub` pushing `ghcr.io/thoth-pub/thoth:staging-pr-<n>` for draft PRs). The classifier run locally on the eight changed paths printed `{"docs_only": "false", "run_build": "true", "run_docker": "true", "run_migrations": "false"}`. **PASS**
9. The #988 line references hold at N: `metric_ingestion_lifecycle/mod.rs` 1132-1144 is the guard body that checked out one connection, opened its transaction and called the pooled `ingest_metric_batch(db, …)` from inside it, and `metric_ingestion/mod.rs` 242-247 is the coordinator's per-attempt `db.get()` and transaction. **PASS**
10. Base mismatch: none; N and D unchanged at every check (last `git ls-remote` 2026-10-29T22:04:27Z). **PASS**

### 1.3 Mechanism and fix evidence

At N the managed-DRIVER batch guard checked out one pooled connection, opened its transaction, locked the account's `metric_source_checkpoint` row `FOR UPDATE`, validated the lease and the batch key, and only then called the pooled coordinator, which checked out a second connection per attempt with the pool's blocking `get()`. On a pool with no idle second connection the guard therefore sat `idle in transaction`, holding the checkpoint lock, for the whole checkout timeout before rolling back. The regression test `the_guard_never_holds_a_transaction_while_waiting_for_a_second_connection` observes exactly that at N: run with the two `mod.rs` files at N and the new tests applied (2026-10-29T21:40:02Z to 21:45:17Z, `test result: FAILED. 0 passed; 2 failed`), it panicked with "the guard sits in an open transaction (last statement `SELECT source_checkpoint_id, lease_owner, (lease_expires_at IS NOT NULL AND lease_expires_at > transaction_timestamp()) AS lease_is_live FROM metric_source_checkpoint WHERE source_account_id = $1 AND partition_key = $2 FOR UPDATE`) while waiting for a second connection", and `a_batch_is_refused_before_any_guard_work_when_no_idle_second_connection_exists` panicked with "backend 215 ran a statement after its checkout: left `ROLLBACK`, right `SELECT 1`" — the guard connection had opened, locked and rolled back a transaction before the refusal.

After the fix the batch validates its shape, checks out the guard connection with `db.get()`, reserves the coordinator's connection with exactly one `db.try_get()`, and on `None` drops the guard connection and returns the existing sanitized `INTERNAL_DATABASE_ERROR` before any transaction exists. With both connections held, the guard transaction executes `SET LOCAL lock_timeout = '1s'` before `find_import` and the `FOR UPDATE` lease validation, and calls the crate-internal coordinator entry `ingest_metric_batch_on_connection(&mut canonical, &batch)`, which runs every attempt — each of the at most three authority-drift retries included — on that one reserved connection, each attempt a transaction that first executes `SET LOCAL statement_timeout = '30s'`. The same tests pass on the corrected tree (section 9).

## 2. Scope confirmation

Approved specification: #988 as corrected by `6385431141`, under #981 Amendment 8 / 8-C1 and the CTO eleven-record approval, implemented under the exact handoff issued by #988 `6508149914`.
Implemented objective: reserve the coordinator's connection with one non-blocking `try_get()` before the `ingestMetricBatch` guard opens its transaction or locks the checkpoint, refuse cleanly with the existing sanitized `INTERNAL_DATABASE_ERROR` when no idle connection exists (also below the pool maximum), bound the guard's checkpoint wait with `SET LOCAL lock_timeout = '1s'`, add the crate-internal coordinator entry on the reserved `&mut PgConnection` and run every canonical attempt there under `SET LOCAL statement_timeout = '30s'` in both entries, preserve shape-before-DB validation, v1 hashes, authorization, replay, drift, rollback, sanitization and the two-transaction seam, record the behaviour in contract-register section 3.2 and the Unreleased changelog, and add deterministic tests for every acceptance item.

Out-of-scope changes made: NONE (source and path scope: `git diff --stat` from N lists exactly the seven budgeted paths plus this report; section 4.1)

## 3. Commits

- `5f81265bc44347ed9f6816a6036402f86ff3ead2` - MET-WP11-B-CORR-H2: reserve the coordinator connection before the batch guard locks (#988)
- the following commit - MET-WP11-B-CORR-H2: add the implementation report and changelog PR link (#988) — its SHA is recorded on PR #993

## 4. Files changed

Authorized write paths (from the handoff, section 5):

- `thoth-api/src/model/metric_ingestion_lifecycle/mod.rs`
- `thoth-api/src/model/metric_ingestion_lifecycle/tests.rs`
- `thoth-api/src/model/metric_ingestion/mod.rs`
- `thoth-api/src/model/metric_ingestion/tests.rs`
- `thoth-api/src/graphql/metric_ingestion_lifecycle_tests.rs`
- `docs/metrics/contract-register.md`
- `CHANGELOG.md`

Authorized new-file paths:

- `docs/engineering/ai-delivery/implementation-reports/MET-WP11-B-CORR-H2-implementation-report.md`

Actual files changed:

- `thoth-api/src/model/metric_ingestion_lifecycle/mod.rs`
  - reason: the correction itself
  - behavioural effect: `ingest_metric_batch_under_claim` now checks out the guard connection with `db.get()`, reserves the coordinator's connection with exactly one `db.try_get()` before any transaction, drops the guard connection and returns `MetricLifecycleError::Ingestion(InternalDatabaseError)` with one fixed log line when nothing is idle, executes the new `GUARD_LOCK_TIMEOUT_SQL` (`SET LOCAL lock_timeout = '1s'`) as the guard transaction's first statement, and calls `ingest_metric_batch_on_connection(&mut canonical, &batch)` while holding the checkpoint lock. Import lookup, live-lease validation under the lock, reclaim exclusion, expected batch-key check, error mapping and the `MetricBatchResult` conversion are unchanged. Module and function documentation record the mechanism.
  - within authorized write budget: YES
- `thoth-api/src/model/metric_ingestion_lifecycle/tests.rs`
  - reason: ten new deterministic tests for the acceptance items of handoff section 10 (section 9), with tagged-pool, `pg_stat_activity`, advisory-lock pause, setting-probe and held-statement helpers
  - behavioural effect: tests only
  - within authorized write budget: YES
- `thoth-api/src/model/metric_ingestion/mod.rs`
  - reason: the crate-internal connection entry and the canonical statement timeout
  - behavioural effect: adds `CANONICAL_STATEMENT_TIMEOUT_SQL` (`SET LOCAL statement_timeout = '30s'`) and `pub(crate) fn ingest_metric_batch_on_connection(&mut PgConnection, &MetricIngestionBatch)`; both entries validate the request shape first and then share the private runner `ingest_validated_batch(connection, batch, statement_timeout_sql)`, whose every attempt is one transaction that executes the timeout statement before `run_attempt`. The public `ingest_metric_batch(db, batch)` keeps its signature and semantics and now checks out one connection for the whole call instead of one per attempt (section 5, decision 1). Hashing (`hash.rs` byte-identical), classifications, replay, drift, rollback and sanitization are unchanged.
  - within authorized write budget: YES
- `thoth-api/src/model/metric_ingestion/tests.rs`
  - reason: five new tests for the connection entry and the statement timeout on both entries (section 9)
  - behavioural effect: tests only
  - within authorized write budget: YES
- `thoth-api/src/graphql/metric_ingestion_lifecycle_tests.rs`
  - reason: API-level proof that a refused reservation is the fixed `INTERNAL_DATABASE_ERROR` message with no durable effect, that the one connection is returned, and that the identical retry commits once
  - behavioural effect: tests only
  - within authorized write budget: YES
- `docs/metrics/contract-register.md`
  - reason: section 3.2 — the batch bullet now says the coordinator runs on a second pooled connection in its own transaction; a new bullet "Batch capacity admission and bounded guard waits (`MET-WP11-B-CORR-H2`, issue #988)" records the reservation, its refusal semantics, the 1 s lock timeout, the 30 s per-statement timeout, the `SET LOCAL` scope, the absence of a whole-request deadline or fairness promise and the preserved two-transaction seam; the errors bullet names the three new `INTERNAL_DATABASE_ERROR` sources
  - behavioural effect: documentation
  - within authorized write budget: YES
- `CHANGELOG.md`
  - reason: `## [Unreleased]` / `### Fixed` entry for `MET-WP11-B-CORR-H2` (the PR link is added by the report commit)
  - behavioural effect: documentation
  - within authorized write budget: YES

Actual new files created:

- `docs/engineering/ai-delivery/implementation-reports/MET-WP11-B-CORR-H2-implementation-report.md` - within authorized new-file list: YES

Files deleted, moved or renamed: NONE

### 4.1 Write-budget compliance

PASS — `git diff --stat 97de027da15dcb0fd1578b37e313365ef5834ffd` before the first commit listed exactly the seven authorized existing paths (`7 files changed, 1591 insertions(+), 30 deletions(-)`), and the report commit adds only the one authorized new file and the changelog PR link. `thoth-api/src/model/metric_ingestion/hash.rs` is byte-identical to N (SHA-256 `b756c3176541f118bfb4d4a322b9a6a57c37c7d455fdaf7ffdecb202319fa0db` at N and at head); `Cargo.lock`, the Cargo manifests, `thoth-api/src/schema.rs`, `thoth-api/src/db.rs` (pool configuration), migrations, workflows, the classifier, authorization/policy code and the four doctrine files H1 updated are unchanged (`git diff --stat N -- <those paths>` empty). The generated SDL was produced only by `thoth-client/build.rs` during the workspace build and was never edited or committed.

## 4.2 Authorized actions actually used

- repository inspection: USED (GitHub reads through the GitHub MCP tools and the proxy-configured `gh api` for body hashing; `git ls-remote`; local object reads)
- source edit: USED, within the seven authorized paths
- new file creation: USED, the one authorized report
- file deletion/move/rename: NOT USED
- branch creation: USED, `feature/metrics--wp11-b-corr-h2` at exact N in an isolated worktree
- commit: USED, two bounded commits; `Cargo.lock` unchanged
- push: USED, only the named task branch (`git push -u origin feature/metrics--wp11-b-corr-h2`)
- PR creation/update: USED, one draft PR (#993) targeting `feature/metrics`; assigned to `ja573` as the control thread instructed; the review request from `ja573` was refused by GitHub ("Review cannot be requested from pull request author") because that account authored the PR; the body was updated once, immediately after creation, to restore the required closing attribution lines that the creation tooling had replaced with its own footer; not marked ready
- issue/comment mutation: USED only for the bounded implementation/test/CI evidence comment on #988 that the handoff permits; no other issue, the parent #981, #984, #987 or any PR body/state was mutated
- manual CI dispatch/rerun: NOT USED
- provider/runtime read: NOT USED (no registry, provider or runtime access; image evidence from GitHub job logs only)
- provider/runtime write: NOT USED
- migration execution: USED — the embedded test runner applied the existing migration chain to the fresh disposable local `thoth_test` database on the `postgres:17` container (PostgreSQL 17.11), as the handoff grants; the database was dropped and re-created empty once, after the disk incident of section 9, and the chain was applied again by the same runner. No `cargo run migrate` and no non-disposable target.
- release/tag/publication: NOT USED
- merge: NOT USED
- deployment: NOT USED
- production activation: NOT USED
- other: local environment preparation inside the disposable cloud container only — the `postgres:17` Docker container bound to `127.0.0.1:54329`, a local Redis on `127.0.0.1:63799`, the `libpq-dev` package (client library 16.15, used for linking only) and `THOTH_EXPORT_API` set exactly as the CI workflows set it; `CARGO_INCREMENTAL=0` for the validation re-run after the disk exhaustion (section 9)

Unauthorized actions performed: none identified against the issued handoff's path budget (section 4.1), action matrix (above) and execution conditions: a fresh instance implemented (section 1), local validation ran on a fresh disposable PostgreSQL 17 (section 9), no ref was deleted or moved, nothing was rebased, imported or force-pushed, and no production, registry or provider access occurred.

## 4.3 Automatic and manual external effects

Automatic CI/provider effects observed:

Observed on the first commit `5f81265bc44347ed9f6816a6036402f86ff3ead2` (PR [#993](https://github.com/thoth-pub/thoth/pull/993); GitHub ran every workflow on its `pull_request` merge ref `0bd7645c06c606a69823461d54a54f8b27e698e4`, the merge of `5f81265b` into N, which carries the same tree as `5f81265b` because the base is the merge-base). All four workflows were triggered by the push and PR creation alone at 2026-10-29T22:53:00Z; nothing was dispatched, re-run or cancelled. State at this report's evidence cutoff, 2026-10-29T23:01Z:

- `build-test-and-check` run [40969179775](https://github.com/thoth-pub/thoth/actions/runs/40969179775): `classify` (job 123781295214) succeeded with exactly the predicted classification `{"docs_only": "false", "run_build": "true", "run_docker": "true", "run_migrations": "false"}`; `format_check` (`cargo fmt --all -- --check`, job 123781355651) succeeded, 22:53:12Z to 22:53:20Z; `build` (`cargo build -vv`, job 123781355457) succeeded, 22:53:11Z to 22:57:48Z; `lint` (`cargo clippy --all --all-targets --all-features -- -D warnings`, job 123781355497) and `test` (`cargo test --workspace -vv` against the workflow's `postgres:17` and `redis:alpine` service containers, job 123781355643) were still in progress.
- `check-changelog` run [40969179748](https://github.com/thoth-pub/thoth/actions/runs/40969179748): succeeded (job 123781295174, 22:53:03Z to 22:53:10Z).
- `run-migrations` run [40969179662](https://github.com/thoth-pub/thoth/actions/runs/40969179662): `classify` (job 123781295212) succeeded and its log line is `{"docs_only": "false", "run_build": "true", "run_docker": "true", "run_migrations": "false"}`; the `run_migrations` job (123781370971) was skipped, as the classification requires for a diff without migration or `db.rs` paths.
- `publish-to-dockerhub` run [40969179731](https://github.com/thoth-pub/thoth/actions/runs/40969179731): `classify` (job 123781295024) succeeded; `build_and_push_staging_docker_image` (job 123781357155) had logged in to the container registry and was still in its `Build and push` step toward the automatic `ghcr.io/thoth-pub/thoth:staging-pr-993` image, so the publication and its digest were not yet observable from the log.

The commit that adds this report triggers the same four workflows once more; those results, and the completion of the runs above, post-date the report and are recorded on PR #993 and in the #988 evidence comment (job logs only; no registry access).

Manually initiated external actions: NONE

External writes/publication: none completed at the evidence cutoff; the automatic `publish-to-dockerhub` workflow was in progress toward `ghcr.io/thoth-pub/thoth:staging-pr-993` (run 40969179731, job 123781357155), the staging publication the handoff accepts, and its digest is held in that job's log once it completes. No other registry, release, tag, package or third-party write.

## 5. Implementation decisions

1. One checkout for the whole public call. The public `ingest_metric_batch(db, batch)` keeps its signature, shape-before-DB validation, hashing, replay, drift, rollback and sanitization, but now checks out its one connection once and runs every attempt on it through the shared runner, where at N it checked out a connection per attempt. The handoff requires the reserved connection to serve every attempt of the lifecycle entry and the public entry to stay compatible; sharing one runner keeps both entries' semantics identical by construction, and the checkout-failure path (log line, `INTERNAL_DATABASE_ERROR`) is unchanged. A reviewer may weigh this as the one behavioural difference in the public entry.
2. A private runner with the timeout statement as a parameter. `ingest_validated_batch(connection, batch, statement_timeout_sql)` is called by both entries with `CANONICAL_STATEMENT_TIMEOUT_SQL`; only the module's tests inject `SET LOCAL statement_timeout = '200ms'` to prove the abort, rollback and sanitization path without a 30 s wait (the pattern H1 used for the dashboard read). The production value is pinned by assertion and observed from inside the canonical transaction by a trigger probe on both entries.
3. The connection entry is `pub(crate)`, not public: it is the "internal coordinator entry" of handoff section 9.5, callable by the lifecycle module and the crate's tests only, so no public API or SDL surface changes.
4. Reservation and refusal exactly as specified: `db.get()?` for the guard, then `let Some(mut canonical) = db.try_get() else { drop(guard); … }`, returning `MetricLifecycleError::Ingestion(InternalDatabaseError)` — the same variant the existing pool-error conversion produces — with one fixed `log::error!` line carrying no request detail; no blocking fallback, third checkout, pool growth or scheduler.
5. `SET LOCAL lock_timeout = '1s'` is the guard transaction's first statement, before `find_import` and before the `FOR UPDATE` lease validation, so it bounds the checkpoint lock wait and nothing else in the request; both `SET LOCAL` settings end with their transaction, and tests assert the connection's `statement_timeout` is back to its default afterwards.
6. Deterministic synchronization in the tests uses observed barriers only: pools tagged by `application_name` so their backends can be found in `pg_stat_activity`; `wait_event_type = 'Lock'` with `pg_blocking_pids`, `state = 'idle in transaction'`, `xact_start` and the backend's last `query` as barriers; a `BEFORE INSERT` trigger on `metric_import_batch` taking a transaction-level advisory lock to freeze a batch inside its canonical transaction; a trigger probe reading `current_setting('statement_timeout')` inside that transaction; `pg_terminate_backend` on the guard backend to inject the guard failure after the canonical transaction is frozen; and held transactions on their own sessions for drift. Timing assertions are lower bounds only (`>= 1s`, `>= 200ms`); the 20 s polling deadlines are hang guards, not proofs. No test enlarges the production pool or sleeps as a proof.
7. The old nested-checkout wait is pinned by the regression test on a one-connection pool, which fails at N and passes at head (section 1.3), as handoff section 10 requires.
8. Environment: the `postgres:17` container and Redis were bound to `127.0.0.1` only; `libpq-dev` 16.15 is the client library the test binaries link against (the server is 17.11); `THOTH_EXPORT_API=https://export.thoth.pub` is required to compile `thoth-export-server`, as in the workflows; after the first validation pass exhausted the container's disk during Clippy, the build directory's incremental artifacts were removed, the disposable `thoth_test` database was dropped and re-created, and every validation command was re-run in one pass with `CARGO_INCREMENTAL=0`; the commands, toolchain and dependencies are unchanged.

List any deviation from the specification requiring authorization: none identified. The implementer is a fresh instance, local validation ran on fresh disposable PostgreSQL 17 and Redis, only the budgeted paths changed, only granted actions were used, and the environment facts above change no handoff command, toolchain or dependency. This report authorizes nothing and claims no approval.

## 6. Database and migration effects

Migration added: NO

No schema, table, column, index, data or configuration change. The only database effects are transaction-scoped: `SET LOCAL lock_timeout = '1s'` in the guard transaction and `SET LOCAL statement_timeout = '30s'` in each canonical attempt; both end with their transaction, so no pooled connection carries them.

## 7. API and compatibility effects

GraphQL/API changes: none to the public SDL or to any operation's shape; the existing sanitized `INTERNAL_DATABASE_ERROR` / "The metric ingestion request was refused." path can now also result from a refused reservation, a guard lock-timeout expiry or a canonical statement timeout.
Generated schema/client updates: none. The public SDL generated by `thoth-client/build.rs` at the authorized base N and at the candidate head is byte-identical: SHA-256 `2681b54ac18296f886451d08f5396c0a9ef8351762f63f7b3f80f319ae505496` at both (section 9); the base hash equals the hash the H1 report recorded at M2.
Backwards compatibility: preserved — identical SDL, identical authorization, shape, hash, replay, idempotency and error contract; the internal Rust `thoth-client` and export server compile unchanged in the workspace build.
Deprecations: none.
Cross-repository dependencies: none required. `thoth-app`, `thoth-pyramid`, `thoth-dissemination`, the standalone Python `thoth-client`, `metrics-dashboard`, `metrics-widget`, `baboon` and the internal Rust `thoth-client` / export server are expected compatible without source edits because the public signatures and the existing error contract are unchanged; this is reasoned from the byte-identical SDL, not from a fresh inspection of those repositories. Sphinx at the pinned identities (`feature/metrics` `fa4b0f88…`, `develop` `e97444c2…`, re-read unchanged during this task) treats `INTERNAL_DATABASE_ERROR` as retryable within its configured attempts according to the classifier and runner that correction 1 pins; that is source compatibility evidence for the refusal code, not a guarantee that a unit eventually succeeds. Sphinx #38 is H3 scope and was not started.

## 8. Authorization and security

Authorization paths changed: none. `ingestMetricBatch` still authorizes `METRICS_INGEST_SERVICE` before any operation-specific database access (`authorization_is_decided_before_any_operation_specific_database_access` passes unchanged), and the lease token, reclaim exclusion and expected batch-key checks still run under the checkpoint lock.
Roles/scopes involved: `METRICS_INGEST_SERVICE` (unchanged).
Negative authorization tests: the existing lifecycle matrix passes unchanged; the stale, expired and reclaimed token, unexpected batch key, disabled scope and auth cases remain fail-closed in the existing tests, which run in both full suites below.
Secret or personal-data handling: none; the fixed refusal message and the one new log line carry no SQL, constraint, connection, token, configuration or caller input.
Security limitations: none new.

## 9. Tests and checks

Environment: disposable `thoth_test` database on a local `postgres:17` Docker container (PostgreSQL 17.11, UTF8, created empty for this task; schema applied by the embedded test runner), local Redis 7.0.15, `TEST_DATABASE_URL=postgres://thoth:…@127.0.0.1:54329/thoth_test`, `TEST_REDIS_URL=redis://127.0.0.1:63799`, `THOTH_EXPORT_API=https://export.thoth.pub`, toolchain `rustc 1.97.0 (2d8144b78 2026-07-07)`, `cargo 1.97.0 (c980f4866 2026-06-30)`, `clippy 0.1.97 (2d8144b788 2026-07-07)`, `rustfmt 1.9.0-stable (2d8144b788 2026-07-07)`; `r2d2 0.8.10`, `diesel 2.3.10` from the unchanged `Cargo.lock`. All results below are from the worktree at the first commit's tree, in one sequential pass started 2026-10-29T22:02:52Z (`CARGO_INCREMENTAL=0`, see section 5 item 8) unless stated otherwise.

### Formatting

Command:

```text
cargo fmt --all -- --check
```

Result:

```text
exit 0, no output (nothing to reformat); 2026-10-29T22:02:52Z to 22:02:54Z (also exit 0 at 21:50:02Z on the same tree)
```

### Unit tests

Command:

```text
cargo test -p thoth-api --features backend
```

Result:

```text
     Running unittests src/lib.rs (target/debug/deps/thoth_api-5dd0e12ad663407d)
test result: ok. 2148 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 837.90s
     Running tests/graphql_permissions.rs (target/debug/deps/graphql_permissions-762a90930dc46b12)
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.22s
   Doc-tests thoth_api
test result: ok. 0 passed; 0 failed; 8 ignored; 0 measured; 0 filtered out; finished in 0.01s
exit 0 (2026-10-29T22:07:56Z to 22:26:10Z); 2148 = the 2132 lib tests at the base plus the 16 new tests
```

### Integration/database tests

Command:

```text
cargo test --workspace
```

Result:

```text
thoth (lib): 0 passed; 0 failed — thoth (bin): 31 passed; 0 failed
thoth_api (lib): 2148 passed; 0 failed; 3 ignored; finished in 798.77s
thoth_api tests/graphql_permissions.rs: 13 passed; 0 failed
thoth_api_server: 3 passed; 0 failed — thoth_client: 4 passed; 0 failed
thoth_errors: 13 passed; 0 failed — thoth_export_server: 144 passed; 0 failed
Doc-tests: thoth 0; thoth_api 0 passed, 8 ignored; thoth_api_server 0; thoth_client 6 passed; thoth_errors 0; thoth_export_server 2 passed — 0 failed anywhere
exit 0 (2026-10-29T22:26:10Z to 22:45:00Z)
```

Focused evidence commands (same environment and binary):

```text
# before the fix: the two `mod.rs` files at N, the new tests applied (2026-10-29T21:40:02Z to 21:45:17Z)
cargo test -p thoth-api --features backend -- --test-threads=1 metric_ingestion_lifecycle::tests::the_guard_never_holds_a_transaction_while_waiting_for_a_second_connection metric_ingestion_lifecycle::tests::a_batch_is_refused_before_any_guard_work_when_no_idle_second_connection_exists
test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 2143 filtered out; finished in 3.86s   (panics quoted in section 1.3)

# after the fix (2026-10-29T21:47:22Z to 21:49:36Z): the sixteen new tests and four existing neighbours
cargo test -p thoth-api --features backend -- <the twenty test names below>
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 2131 filtered out; finished in 16.96s

# after the full gates, three consecutive passes of the same twenty tests with --exact
cargo test -p thoth-api --features backend -- --exact <the twenty names below>
run 1: test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 2131 filtered out; finished in 17.25s   (exit 0, 2026-10-29T22:50:18Z to 22:50:48Z)
run 2: test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 2131 filtered out; finished in 16.27s   (exit 0, 22:50:48Z to 22:51:16Z)
run 3: test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 2131 filtered out; finished in 14.48s   (exit 0, 22:51:16Z to 22:51:42Z)
```

Focused evidence (all within the runs above; names are exact):

- regression of the original nested-checkout wait: `model::metric_ingestion_lifecycle::tests::the_guard_never_holds_a_transaction_while_waiting_for_a_second_connection` (fails at N, passes at head)
- exhausted pool and pool size one: `a_batch_is_refused_before_any_guard_work_when_no_idle_second_connection_exists` (one-connection pool and a three-connection pool with two held: refusal, the guard backend's last statement still the checkout `SELECT 1` with no transaction, connections returned, no durable effect, identical retry commits once)
- refusal with no idle second slot while nominal capacity remains: `a_batch_is_refused_when_no_connection_is_idle_even_below_the_pool_maximum` (`max_size 4`, `min_idle 0`: one backend established for the guard and returned, none opened for the coordinator, refusal repeated, retry commits once two are idle)
- guard checkpoint contention and the 1 s lock timeout: `a_held_checkpoint_row_fails_the_guard_closed_after_the_one_second_lock_timeout` (guard observed queued on the held row with both connections out, returns `INTERNAL_DATABASE_ERROR` after at least one second while the row is still held, guard rolled back, reserved connection untouched, both returned, nothing committed, retry commits)
- same-account backlog: `a_same_account_batch_behind_a_running_batch_is_refused_instead_of_queueing_on_the_checkpoint` (b1 frozen inside its canonical transaction; b2 refused at once with no connection of the pool queued on the checkpoint; claim still skips the locked unit; b1 commits; b2 retry commits) and `a_same_account_batch_with_spare_capacity_waits_at_most_one_second_for_the_held_checkpoint` (b2 reserves both spare connections, queues on b1's checkpoint lock and gives up after at least one second while b1 is still frozen; both connections returned)
- distinct-account contention: `a_distinct_account_batch_never_holds_its_guard_while_another_batch_occupies_the_pool` (B refused without locking its checkpoint and without waiting for A; A commits; B commits once capacity exists)
- canonical statement timeout on the lifecycle path: `every_canonical_attempt_of_a_lifecycle_batch_runs_under_the_thirty_second_statement_timeout` (trigger probe inside the canonical transaction accepts `30s` and rejects `31s`; the failure is sanitized, nothing durable, every connection returned, retry is a first commit)
- two-transaction seam: `a_canonical_commit_survives_an_injected_guard_failure_and_the_retry_replays_without_writing` (guard backend terminated while the canonical transaction is frozen; canonical outcome durable exactly once — one batch, record, revision, provenance, delta and the import counters — retry replays with `replayed: true` and no new revision; the unit completes)
- three drift attempts on the reserved connection, no third checkout: `authority_drift_retries_run_on_the_reserved_coordinator_connection_without_a_third_checkout` (each of three attempts observed queued on the work lock from the same backend pid while the guard holds its transaction and the pool has zero idle connections; `ConcurrentAuthorityChange`; both connections returned; retry commits)
- coordinator connection entry: `model::metric_ingestion::tests::the_connection_entry_validates_request_shape_before_touching_its_connection`, `the_connection_entry_replays_a_committed_batch_and_refuses_a_reused_key_without_writing`, `the_connection_entry_runs_all_three_drift_attempts_on_its_one_connection`, `a_canonical_statement_timeout_aborts_the_attempt_rolls_it_back_and_is_sanitized` (injected 200 ms timeout on both the established and a pooled connection: `INTERNAL_DATABASE_ERROR`, nothing committed, `statement_timeout` back to `0`, the same connection commits once the pause is released) and `every_canonical_attempt_sets_the_thirty_second_statement_timeout_on_both_entries` (constant pinned; `30s` observed inside the transaction on both entries; `31s` negative control sanitized with nothing durable)
- API boundary: `graphql::metric_ingestion_lifecycle_tests::a_capacity_refusal_is_the_fixed_internal_database_error_and_the_retry_commits` (one-connection pool: `INTERNAL_DATABASE_ERROR` with the fixed message, no durable effect, the connection returned so `completeMetricImport` still reaches the lifecycle, the identical request commits once on a two-connection pool), plus the unchanged `authorization_is_decided_before_any_operation_specific_database_access`
- unchanged neighbours: `the_batch_guard_holds_the_checkpoint_lock_for_the_whole_coordinator_call`, `repeated_lock_set_drift_through_three_attempts_fails_closed_and_stays_retryable`, `request_shape_failures_precede_database_access`

### Lint/static analysis

Command:

```text
cargo clippy --all --all-targets --all-features -- -D warnings
```

Result:

```text
exit 0 — `Finished dev profile [unoptimized + debuginfo] target(s) in 5m 01s` (2026-10-29T22:02:54Z to 22:07:56Z), no Clippy warning or error. The only diagnostic is the pre-existing dependency notice `the following packages contain code that will be rejected by a future version of Rust: proc-macro-error2 v2.0.1`, present at the base and unrelated to this change.
```

### Other required checks

```text
cargo check --workspace
exit 0 — `Finished dev profile [unoptimized + debuginfo] target(s) in 2m 23s` (2026-10-29T22:47:54Z to 22:50:18Z, run after the suites in the same conditions; also exit 0 from 21:50:04Z to 21:56:55Z on the same tree in the first pass); the same pre-existing proc-macro-error2 notice

git diff --check
exit 0, no output (2026-10-29T22:45:00Z)

cargo build --workspace   # runs thoth-client/build.rs at base and at head
base SDL (clean N, 2026-10-29T21:25:34Z):  2681b54ac18296f886451d08f5396c0a9ef8351762f63f7b3f80f319ae505496
head SDL (first commit's tree):            2681b54ac18296f886451d08f5396c0a9ef8351762f63f7b3f80f319ae505496
cmp against the saved base SDL: byte-identical (`cmp` exit 0 against the SDL saved from the clean-N build); head build exit 0, `Finished dev profile [unoptimized + debuginfo] target(s) in 2m 45s` (2026-10-29T22:45:00Z to 22:47:46Z)
```

Pre-commit review: the full diff was read; `git diff --stat` and `git status --short` listed exactly the seven budgeted paths; `git merge-base --is-ancestor 97de027d… HEAD` held; `git ls-remote` showed `feature/metrics` at N and `develop` at D.

A first validation pass on the same tree (started 2026-10-29T21:50:02Z) recorded `cargo fmt --all -- --check` exit 0 and `cargo check --workspace` exit 0 (`Finished` at 21:56:55Z) and then failed in Clippy with `No space left on device` (exit 101 at 21:57:10Z); the test runs that followed in that pass were invalid for the same environmental reason (PostgreSQL itself logged `could not extend file … No space left on device`) and are not evidence. After freeing the build directory's incremental artifacts and re-creating the disposable `thoth_test` database, the complete pass above was run.

## 10. Manual verification

Environment: as in section 9.
Steps: none beyond the automated tests.
Observed result: not applicable.
Evidence link/screenshot/log reference: the natural CI runs in section 11 and the #988 evidence comment.

## 11. CI

CI status: PENDING at the evidence cutoff (2026-10-29T23:01Z) — on `5f81265b…` every job that had completed succeeded or was skipped as predicted (`classify` in all three classifying workflows, `check-changelog`, `format_check`, `build`; `run_migrations` skipped), and `lint`, `test` and the staging image build were still running with no failure observed. The run on the commit that adds this report starts after this commit; every later head has its own natural CI, held on GitHub.
Checks: `build-test-and-check` (classify, build, test, lint, format_check) run 40969179775; `check-changelog` run 40969179748; `run-migrations` run 40969179662 (classify; `run_migrations` skipped by classification); `publish-to-dockerhub` run 40969179731 (classify, build_and_push_staging_docker_image).
Failures or warnings: none observed. The only notices in the completed job logs are the runners' Node.js 20 deprecation warnings for third-party actions, unrelated to this change.

## 12. Rollout and rollback

Initial state after merge: source correction only on the `feature/metrics` programme line; the managed-DRIVER lifecycle remains inactive under the existing collection, serving and production HOLDs; nothing is activated.
Activation required: NO (none authorized).
Feature flag/configuration: none; no pool configuration changes.
Migration sequence: none.
Rollback/disable procedure: a later code revert needs separate approval and would reinstate the H2 pre-B blocker; historical canonical commits cannot be made guard-atomic retroactively; no production rollback or data operation is authorized.
Monitoring required: none new; a refused reservation, a guard lock-timeout expiry or a canonical statement timeout surfaces as the existing sanitized `INTERNAL_DATABASE_ERROR`, logged server-side without request detail.

## 13. Known limitations and deferred work

- No whole-request deadline or fairness: `try_get()` validates the popped idle connection with the manager's health check (`SELECT 1`) and may validate several in turn, and the guard's `get()` may establish a connection, so reservation and checkout can still block on manager I/O; the 1 s and 30 s limits bound the checkpoint wait and each canonical statement respectively, nothing more. The register records this; no total-latency or starvation claim is made.
- The production 30 s statement timeout is exercised only through the injected 200 ms timeout of the private runner; the production constant is pinned by assertion and observed inside the canonical transaction by a trigger probe on both entries.
- The lock-timeout tests necessarily wait at least one second each; their assertions are lower bounds with observed barriers, and their 20 s polling deadlines are hang guards that heavy CI load could in principle reach.
- The guard-failure injection terminates the guard backend with `pg_terminate_backend`; other guard failures after a canonical commit (for example a commit-time network failure) follow the same seam but are not separately simulated.
- `try_get()` semantics are those of the locked `r2d2 0.8.10` (an already-idle connection only, never a new one); a dependency change would need its own task.
- Local validation linked against the `libpq` 16.15 client library (the server was PostgreSQL 17.11); natural CI on `ubuntu-latest` with the `postgres:17` service is the exact-stack evidence.
- The first local validation pass was lost to disk exhaustion and repeated (section 9).
- H3 (#989), H4 (#990), Sphinx #38, terminal A2 and the fresh cumulative B review remain separately gated; nothing here starts them.

## 14. Unresolved issues

- None known at the evidence cutoff (2026-10-29T23:01Z). The independent exact-head cross-model source review, readiness, true merge, H3 and terminal A2 are open and separately gated; the live review, readiness and merge state is held on PR #993 and the owning issue #988.

## 15. Agent self-assessment

The agent may identify risks but may not approve the task.

Suggested review focus:

- the statement order in `ingest_metric_batch_under_claim`: shape validation, `db.get()`, exactly one `db.try_get()`, `drop(guard)` on refusal, then `SET LOCAL lock_timeout = '1s'` as the guard transaction's first statement before `find_import` and the `FOR UPDATE` lease validation;
- decision 1: the public `ingest_metric_batch` now makes one checkout per call rather than one per attempt, and whether that reading of "keep the public pool entry compatible" is acceptable;
- that the private runner's only callers are the two entries with the production constant and the module's tests, and that `SET LOCAL` leaves no setting on a returned connection (asserted via `current_setting`);
- the tests' reliance on r2d2 defaults (`test_on_check_out` with diesel's `SELECT 1` ping, `min_idle` defaulting to `max_size`) and on `pg_stat_activity` fields as observed barriers, and their exposure to CI load through the 20 s hang guards;
- the contract-register 3.2 wording against the implemented statements, and the changelog entry.
