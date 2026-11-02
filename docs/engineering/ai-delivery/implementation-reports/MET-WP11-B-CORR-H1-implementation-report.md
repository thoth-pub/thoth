# MET-WP11-B-CORR-H1 Implementation Report

## 1. Repository state

Owning GitHub issue: [#987](https://github.com/thoth-pub/thoth/issues/987)
Parent programme-integration task: [#981](https://github.com/thoth-pub/thoth/issues/981)
Outer integration PR: [#984](https://github.com/thoth-pub/thoth/pull/984) (`feature/metrics` -> `develop`), observed OPEN / DRAFT / UNMERGED at M2/D by the original implementation from its preflight (2026-10-27T18:38:56Z) to its implementation evidence cutoff (#987 [`6453540926`](https://github.com/thoth-pub/thoth/issues/987#issuecomment-6453540926), 2026-10-27T21:05:27Z); live state is held by GitHub
Repository: `thoth-pub/thoth`
Programme: Thoth Metrics, WP11-B — finite pre-B source correction, H1 first of H1 -> H2 -> H3 (#981 Amendment 8 as corrected by Amendment 8-C1)
Workflow: PROGRAMME_INTEGRATION
Risk: HIGH (concurrency and transaction semantics of a protected read inside the CRITICAL B integration)
Base branch: `feature/metrics`
Authorized base commit: `59d43a167089836d9db58198792f444d15ce6355` (M2, tree `4e8100ffda8c9427b90dfb30cf56f248ead35ccc`, ordered parents `d5451f8516fa18126f6f4969fb3409775408e242`, `1014fb7ea244181d0a3112cb38e3bdd8a3d9bd3c`)
Actual base commit: `59d43a167089836d9db58198792f444d15ce6355` (tree `4e8100ffda8c9427b90dfb30cf56f248ead35ccc`)
Frozen `develop` D: `b44303c214498baf76c9d9a0a7cab374377f9cbe` (tree `050e1e2954b518d62988e31d74a7d9c280b8822c`), unchanged; no `develop` commit is imported
PR target: `feature/metrics`
Programme integration branch: `feature/metrics`
Task branch: `feature/metrics--wp11-b-corr-h1`
Head commit: `df94b5503e830cebf1baa3107b6bec4c520577e6` carries the complete source, test, contract, doctrine and changelog change; the commit that adds this report and the changelog PR link follows it — `bea5ee676e894e10671eff4d1593ce0e13f397cc` (tree `5ac335ac410fb997dd7547a8ab9461ea7bd3bda6`), the original implementation head — and the R1 documentation-correction commit of section 1.4 follows that. A commit cannot contain its own SHA, so the exact current head is the one recorded on the pull request; each independent exact-head source review binds to that SHA.
Pull request: [#991](https://github.com/thoth-pub/thoth/pull/991), `feature/metrics--wp11-b-corr-h1` -> `feature/metrics`; opened as a draft and observed draft at the original implementation evidence cutoff (#987 `6453540926`, 2026-10-27T21:05:27Z); live review, readiness and merge state is held by GitHub
Expected branch deletion after merge: YES — but only through the stage-terminal A2 inventory, independent review, exact-name retirement authorization, guarded retirement and completion receipt that Amendment 8 item 7 defers it to; `delete_branch_on_merge` was `false` at preflight, so a merge does not delete the ref.
Final programme PR required: YES — the existing outer PR #984, separately gated.
Implementing model: Claude Fable 5.1 (`claude-fable-5-1`, configured and last-served), session `session_01DzL8ejmvtAMrj9dVbPz8WE`; exact deployed build not exposed.
Prior involvement, disclosed: this same session performed the independent specification reviews of Amendment 8 (CHANGES REQUIRED, #981 `6385434946` attributed) and of the corrected eleven-record set (APPROVED, #981 `6385770904` attributed). It authored no specification, correction or prior implementation. The independent exact-head source review is arranged separately, cross-family, and this agent issues no approval. Because the handoff requires a fresh Claude coding instance, this reuse of the specification-review session is departure D-1 from the issued handoff (section 5.1).
Reasoning level: maximum practical, as the handoff requires; the operator session reports effort `xhigh`.
R1 documentation-correction author, separate from the implementer above: section 1.4.

### 1.1 Approval and authorization records

| Gate | Record | Outcome |
|---|---|---|
| Original task specification | #987 body, SHA-256 `3fcf5d5eb80b50e13bb247727d4f05357d5060c26289bf4b7a9461d39ab26888` | governs technical scope |
| H1 correction (F-4, deferred B2 dependency) | #987 `6385429854`, SHA-256 `c05169387dba243bd785b4505120f1e98709e0cee542a477aa9aeae8f23b6179` | supersedes the reader-first wording only |
| Parent specification | #981 Amendment 8 `6370994558` and Amendment 8-C1 `6385434946` | corrected eleven-record set |
| Independent CRITICAL specification review | #981 `6385770904` | `APPROVED` (attributed) |
| CTO specification approval, eleven hashes | #981 `6385772335` | `APPROVED - BOUNDED CORRECTED SPECIFICATION ONLY` |
| CTO H1 implementation authorization and handoff | #987 `6423207508`, SHA-256 `5f8ea6214c86b436a5f04c4d46f18db9274d22185479f288b6ea025a75d9d246` | bounded actions used in section 4.2 |
| Independent exact-head source review of `bea5ee67…` | #987 [`6475538703`](https://github.com/thoth-pub/thoth/issues/987#issuecomment-6475538703) | `CHANGES REQUIRED`; its findings are what section 1.4 corrects |
| CTO R1 documentation-correction authorization | #987 [`6475553675`](https://github.com/thoth-pub/thoth/issues/987#issuecomment-6475553675) | bounded R1 actions of section 1.4; also the owning-issue record of the CTO's forward disposition of departures D-1 and D-2 (section 5.1) |

Every one of the eleven approved bodies was re-fetched and re-hashed before any mutation; all eleven digests matched the CTO manifest.

### 1.2 Preflight, performed before any mutation (2026-10-27T18:38:56Z to 18:40:18Z)

1. Handoff comment `6423207508` exists on #987, is the CTO authorization naming this handoff, and its body digest matches `5f8ea621…`. **PASS**
2. `feature/metrics` = M2 `59d43a16…` / tree `4e8100ff…` with the expected ordered parents; `develop` = D `b44303c2…` / tree `050e1e29…`; Sphinx `feature/metrics` `fa4b0f88…` and `develop` `e97444c2…` unchanged. **PASS**
3. #984 OPEN / DRAFT / UNMERGED, head M2, base D, untouched since 2026-10-20T17:49:43Z; #987 open with exactly the H1 correction and the authorization comment; eight open PRs, none from a slice head and none targeting `feature/metrics`; #981 at 50 comments. **PASS**
4. Namespace preflight (root `AGENTS.md` section 5.1): zero live `feature/metrics--*` refs; the exact ref `feature/metrics--wp11-b-corr-h1` and any descendant beneath it absent; `metrics` and `wp11-b-corr-h1` are single path segments without `--`. **PASS**
5. Clean local checkout; the branch was created from exact M2 in a dedicated worktree outside the main checkout (`git worktree add … -b feature/metrics--wp11-b-corr-h1 59d43a16…`); `git status --short` empty before edits. **PASS**
6. All nine authorized existing paths exist at M2; the report path was absent. **PASS**
7. Workflows and `.github/scripts/classify_ci_changes.py` at M2 match the handoff's section 7: a PR diff containing `.rs` paths and no migration or `db.rs` path classifies `docs_only=false, run_build=true, run_migrations=false, run_docker=true`, and the `publish-to-dockerhub` workflow pushes `ghcr.io/thoth-pub/thoth:staging-pr-<n>` for a draft PR. **PASS**
8. `delete_branch_on_merge=false` (preflight evidence only). **PASS**

### 1.3 Mechanism and fix evidence

At M2 the read froze its `REPEATABLE READ` snapshot with the publishers query before touching any derived projection and took no table lock, while `rebuildMetricRollupMonths` truncates the four monthly and the two yearly tables under `ACCESS EXCLUSIVE`. PostgreSQL's `TRUNCATE` is not MVCC-safe, so a snapshot frozen before a concurrent truncate sees the tables it had not yet read as empty, and complete coverage turns that emptiness into a justified, false zero. The rollup verifier already closes the same anomaly with `VERIFICATION_LOCK_SQL` before its first query.

The new test `a_snapshot_frozen_before_a_rebuild_truncate_sees_the_projection_as_empty` replays the pre-fix statement order from the module's own constants on a disposable database and observes the stale snapshot counting zero monthly rows while the committed rebuilt rows exist; the corrected read serves the exact value. The corrected read now executes, before its first query and in this order, `SET LOCAL plan_cache_mode = force_custom_plan`, `SET LOCAL statement_timeout = '30s'` and one `LOCK TABLE` on exactly the six projection tables in the verifier's order `IN ACCESS SHARE MODE`, and holds the lock until the transaction ends.

### 1.4 R1 documentation correction (`MET-WP11-B-CORR-H1-R1`)

This subsection and the corrections it lists were written by a separate correction author. Everything else in this report remains the original implementer's record, and its evidence remains attributed to the original implementation.

- Trigger: the fresh independent exact-head source review of `bea5ee676e894e10671eff4d1593ce0e13f397cc` (tree `5ac335ac410fb997dd7547a8ab9461ea7bd3bda6`) against M2, #987 `6475538703`, returned `CHANGES REQUIRED`: one P1 finding on this report's authorization-compliance assertions and two P2 findings, on the rebuilder-first rollback wording of `docs/metrics/contract-register.md` and on this report's undated lifecycle observations.
- Authority: the CTO R1 authorization #987 `6475553675`, limited to this report and `docs/metrics/contract-register.md`, with no new file and no local database, build or test execution.
- Starting point, verified before editing on 2026-10-28: head `bea5ee67…` / tree `5ac335ac…` on `feature/metrics--wp11-b-corr-h1`; `feature/metrics` at M2 and `develop` at D; #991 and #984 observed open, draft and unmerged; all eleven approved bodies and the handoff body re-hashed and matching; edits made in a clean, isolated worktree of the existing branch.
- Correction author: Claude Opus 5.5 (`claude-opus-5-5`, as the session reports it), Claude Code desktop session `local_34714e14-324a-4214-913d-556aa072a006`, created 2026-10-28T17:23:14Z, effort `xhigh`; exact deployed build not exposed. This is a new instance, distinct from `session_01DzL8ejmvtAMrj9dVbPz8WE`, with no earlier role in H1's specification, specification review, implementation or source review. The operator's local agent memory, available to it as background, holds earlier sessions' notes on adjacent WP11 tasks; this instance performed none of that work.
- Corrections:
  1. Compliance assertions (sections 1, 2, 4.1, 4.2, 5, 13 and 14): the unsupported "Unauthorized actions performed: NONE", "none affecting scope or authorization", "deviation from the specification requiring authorization: NONE" and unresolved-issues "NONE" entries, and the migration-execution entry's "only in the form the handoff grants", are withdrawn, and both departures from the issued handoff are stated in section 5.1. The source change's path-budget compliance stays distinct from compliance with the handoff's execution conditions.
  2. Lifecycle observations (sections 1, 3, 4.2, 4.3 and 11): the #984 and #991 status fields are qualified as observations at the original implementation evidence cutoff, live state is located in GitHub, and the original implementation head `bea5ee67…` is named now that it is known.
  3. `docs/metrics/contract-register.md` section 4.2: the rebuilder-first clause now distinguishes a successful rebuild commit, after which the read sees the rebuilt committed state, from a rollback, after which it sees the previously committed state; a statement timeout still fails the read closed. Pre-snapshot locking, table order, timeouts, the statement ceiling and every other contract are unchanged.
- Preserved: the original implementer's identity and prior-involvement disclosure, chronology, environment, every validation command and result, the SDL comparison and the CI and staging observations. R1 did not rerun, reproduce or extend any of them, and executed no local database, build or test. The original commits and the original #987 evidence comment `6453540926` are not rewritten.
- Not changed: no runtime, test, migration, schema, workflow, dependency or changelog change; the existing `CHANGELOG.md` entry is unchanged.
- R1's own validation, commit, CI and staging evidence, and the live review, readiness and merge state, are held on GitHub (PR #991 and owning issue #987).
- Suggested R1 review focus: that the diff from `bea5ee67…` touches only the two documents; that section 5.1 and the withdrawn entries state the departures without granting, waiving or accepting them; that no original evidence or attribution changed; and the contract clause against the CTO's required wording.

## 2. Scope confirmation

Approved specification: #987 as corrected by `6385429854`, under #981 Amendment 8 / 8-C1, implemented under the exact handoff `6423207508`.
Implemented objective: protect the `metricDashboard` snapshot against the exceptional rebuild's truncate before the first snapshot-establishing query; preserve the read-only `REPEATABLE READ` transaction, the planner setting, authorization before database access, selectors, entitlements, coverage, ambiguity and zero-versus-null semantics and the sanitized `INTERNAL_ERROR` failure path; raise the current reader's statement ceiling to 18 including the pool checkout / 17 excluding it; add deterministic concurrency, timeout, auth and count tests; record the contract, doctrine, changelog and this report.

Out-of-scope changes made: NONE (source and path scope only; compliance with the handoff's execution conditions is a separate question, answered in section 5.1)

## 3. Commits

- `df94b5503e830cebf1baa3107b6bec4c520577e6` - MET-WP11-B-CORR-H1: lock the six projection tables before the dashboard snapshot (#987)
- `bea5ee676e894e10671eff4d1593ce0e13f397cc` - MET-WP11-B-CORR-H1: add the implementation report and changelog PR link (#987) — the original implementation head
- the following R1 commit - MET-WP11-B-CORR-H1-R1: correct the report's compliance assertions and the rebuilder-first wording (#987) — its SHA is recorded on PR #991 (section 1.4)

## 4. Files changed

Authorized write paths (from the handoff):

- `thoth-api/src/model/metric_dashboard/mod.rs`
- `thoth-api/src/model/metric_dashboard/tests.rs`
- `thoth-api/src/graphql/metric_dashboard_tests.rs`
- `docs/metrics/contract-register.md`
- `docs/engineering/ai-delivery/branching-and-release-workflow.md`
- `docs/engineering/repository-map/branch-topology.md`
- `docs/metrics/README.md`
- `docs/metrics/task-status.md`
- `CHANGELOG.md`

Authorized new-file paths:

- `docs/engineering/ai-delivery/implementation-reports/MET-WP11-B-CORR-H1-implementation-report.md`

Actual files changed:

- `thoth-api/src/model/metric_dashboard/mod.rs`
  - reason: the correction itself
  - behavioural effect: adds `STATEMENT_TIMEOUT_SQL` and `PROJECTION_LOCK_SQL`; `evaluate` executes the planner setting, the timeout and the lock in that order before the first query; `metric_dashboard` delegates to a private `metric_dashboard_with_statement_timeout` that takes the timeout statement so tests can prove the contention failure path with a short timeout; module and function documentation record the mechanism and the 17/18 ceiling. No other statement, binding, assembly, authorization or error mapping changes.
  - within authorized write budget: YES
- `thoth-api/src/model/metric_dashboard/tests.rs`
  - reason: updated ceiling, sequence and source-shape assertions; new deterministic regression, concurrency, timeout, queued-reader and maintenance tests (section 9)
  - behavioural effect: tests only
  - within authorized write budget: YES
- `thoth-api/src/graphql/metric_dashboard_tests.rs`
  - reason: API-level proof that the resolver path locks before its first query within the 18-statement ceiling and waits for an exclusive holder before its snapshot
  - behavioural effect: tests only
  - within authorized write budget: YES
- `docs/metrics/contract-register.md`
  - reason: section 4.2 execution contract rewritten for the pre-snapshot lock, the 30 s per-statement timeout, the 18/17 ceiling, contention semantics, bounded failure and the deferred B2 reconciliation prerequisite
  - behavioural effect: documentation
  - within authorized write budget: YES
- `docs/engineering/ai-delivery/branching-and-release-workflow.md`, `docs/engineering/repository-map/branch-topology.md`, `docs/metrics/README.md`, `docs/metrics/task-status.md`
  - reason: the four post-A2 programme-flow paragraphs now describe the approved finite three-task pre-B correction route and its deferred terminal retirement, by stable owning-issue links only, preserving the ordinary procedure and the historical one-time `feature/metrics--wp11-b-clippy` exception
  - behavioural effect: documentation
  - within authorized write budget: YES
- `CHANGELOG.md`
  - reason: `## [Unreleased]` / `### Fixed` entry for `MET-WP11-B-CORR-H1`
  - behavioural effect: documentation
  - within authorized write budget: YES

Actual new files created:

- `docs/engineering/ai-delivery/implementation-reports/MET-WP11-B-CORR-H1-implementation-report.md` - within authorized new-file list: YES

Files deleted, moved or renamed: NONE

### 4.1 Write-budget compliance

PASS — `git diff --stat 59d43a167089836d9db58198792f444d15ce6355..HEAD` lists exactly the nine authorized existing paths plus the one authorized new file; `Cargo.lock`, manifests, migrations, `schema.rs`, workflows, policy and every other path are unchanged.

This result concerns the write budget only, not the handoff's execution conditions (section 5.1). The R1 correction changes two of these same paths, both within its own two-document grant (section 1.4).

## 4.2 Authorized actions actually used

- repository inspection: USED (GitHub REST and `git ls-remote` reads, local object reads)
- source edit: USED, within the nine authorized paths
- new file creation: USED, the one authorized report
- file deletion/move/rename: NOT USED
- branch creation: USED, `feature/metrics--wp11-b-corr-h1` at exact M2 in an isolated worktree
- commit: USED, two bounded commits; `Cargo.lock` unchanged
- push: USED, only the named task branch
- PR creation/update: USED, one draft PR targeting `feature/metrics`; not marked ready by the original implementation (observation at its evidence cutoff)
- issue/comment mutation: USED only for the bounded implementation/CI evidence comment on #987 permitted by the handoff; no other issue, the parent #981 or any PR body/state was mutated
- manual CI dispatch/rerun: NOT USED
- provider/runtime read: NOT USED (no registry, provider or runtime access; image evidence from GitHub job logs only)
- provider/runtime write: NOT USED
- migration execution: USED — the embedded test runner applied the existing migration chain to a fresh disposable local database created for this task; no `cargo run migrate` against any non-disposable target. The original entry said this was "only in the form the handoff grants"; that is withdrawn, because the grant names fresh disposable PostgreSQL 17 and the database was PostgreSQL 16.15 (departure D-2, section 5.1)
- release/tag/publication: NOT USED
- merge: NOT USED
- deployment: NOT USED
- production activation: NOT USED
- other: local environment preparation inside the disposable container only — a fresh `thoth_test` database on the locally installed PostgreSQL 16 cluster (departure D-2, section 5.1), a local Redis, the `libpq-dev` package required to link, and `THOTH_EXPORT_API` set exactly as the CI workflows set it

Unauthorized actions performed: the original entry `NONE` is withdrawn as unsupported. Measured against the issued handoff's execution conditions rather than its path budget:

- D-1: the implementation as a whole, every action recorded as USED above, was performed by a session that did not satisfy the handoff's fresh-Claude-coding-instance requirement (section 5.1);
- D-2: the local builds and tests, and the disposable migration execution by the embedded test runner, ran on PostgreSQL 16.15 rather than the fresh PostgreSQL 17 the grant names, instead of the HOLD the handoff requires for an unsupported environment (section 5.1).

Neither was authorized when it occurred. The source change itself stayed within the authorized paths (section 4.1).

## 4.3 Automatic and manual external effects

Automatic CI/provider effects observed:

Observed on the first commit `df94b5503e830cebf1baa3107b6bec4c520577e6` (PR [#991](https://github.com/thoth-pub/thoth/pull/991); GitHub ran every workflow on its `pull_request` merge ref `f1307071af59948feef08001f5df64d5d0e55e9d`, the merge of `df94b550` into `59d43a16`, which carries the same tree as `df94b550` because the base is the merge-base). All four workflows were triggered by the push and PR creation alone; nothing was dispatched, re-run or cancelled.

- `build-test-and-check` run `40579140261`: `classify` succeeded with exactly the predicted classification `{"docs_only": "false", "run_build": "true", "run_docker": "true", "run_migrations": "false"}`; `build` (`cargo build -vv`) succeeded; `lint` (`cargo clippy --all --all-targets --all-features -- -D warnings`) succeeded; `format_check` (`cargo fmt --all -- --check`) succeeded; `test` (`cargo test --workspace -vv` against the workflow's `postgres:17` and `redis:alpine` service containers, Redis 8.10.3) succeeded — the PostgreSQL 17 evidence that the local PostgreSQL 16 run cannot give.
- `check-changelog` run `40579140447`: succeeded.
- `run-migrations` run `40579140453`: `classify` succeeded and the `run_migrations` job was skipped, as the classification requires for a diff without migration or `db.rs` paths.
- `publish-to-dockerhub` run `40579140478`: `classify` succeeded; `build_and_push_staging_docker_image` succeeded and pushed the automatic staging image `ghcr.io/thoth-pub/thoth:staging-pr-991`, digest `sha256:abefbea536b6ee79fbf1a49b7d06d452e3d815f79c73cdd8be4ed3cb46411599`, OCI label `org.opencontainers.image.revision=f1307071af59948feef08001f5df64d5d0e55e9d`, created `2026-10-27T20:34:02Z` (job log only; no registry access).

The commit that adds this report, the original implementation head `bea5ee67…`, triggers the same four workflows once more; those results post-date the report and are recorded on PR #991 and in the original #987 evidence comment `6453540926`. Later pushes to the branch, including R1's, repeat these automatic effects; their evidence is held on GitHub (section 1.4).

Manually initiated external actions: NONE

External writes/publication: the automatic `publish-to-dockerhub` workflow pushed `ghcr.io/thoth-pub/thoth:staging-pr-991` as the handoff accepts; details above. No other registry, release, tag, package or third-party write.

## 5. Implementation decisions

1. Separate `SET LOCAL statement_timeout = '30s'` and `LOCK TABLE` statements, exactly as #987 specifies, mirroring the verifier; the combined `set_config` settings statement of #980 Amendment 2 belongs to deferred B2 and is not implemented, so the current reader's ceiling is 18 including the pool checkout and 17 excluding it.
2. `PROJECTION_LOCK_SQL` is this module's own constant rather than a reference to the verifier's, so the dashboard shares no statement with maintenance code; a test pins it equal to `VERIFICATION_LOCK_SQL` and to the exact table list of `REBUILD_TRUNCATE_SQL`.
3. The timeout statement is passed through a private `metric_dashboard_with_statement_timeout`; the public entry always passes the production constant (asserted from source), and only the module's tests inject a 250 ms timeout to prove the contention failure path without a 30 s wait.
4. Deterministic synchronization uses Diesel's connection instrumentation to pause a connection just before its first query or just after the rebuild's `TRUNCATE`, and `pg_stat_activity` / `pg_locks` observations to establish that a backend is waiting or holding; no proof is a sleep. Polling loops are bounded by a 60 s deadline.
5. A real `rebuildMetricRollupMonths` is forced to truncate by one stray yearly country row that nothing in the complete-month request reads, so reader-first snapshots keep their exact values while the rebuild genuinely replaces the six tables.
6. Deviations from the handoff's stated environment. The original qualifier "none affecting scope or authorization" is withdrawn: the first item below is departure D-2 (section 5.1).
   - local validation ran on the container's PostgreSQL 16.15 (UTF8) rather than PostgreSQL 17: the Docker daemon could be started but Docker Hub refused the `postgres:17` pull with HTTP 429 on the one permitted retry, and the PGDG apt repository is unreachable from this container; natural CI on `postgres:17` is the exact-version evidence;
   - `libpq-dev` had to be installed in the container to link the test binaries, and `THOTH_EXPORT_API` set as the workflows set it to compile `thoth-export-server`;
   - `cargo build -p thoth-client` alone does not compile at M2 (feature unification), so the base and candidate SDL were generated by the supported `build.rs` path during `cargo build --workspace`, never hand-edited or committed.

List any deviation from the specification requiring authorization: the original entry `NONE` is withdrawn as unsupported. Two departures from the issued handoff's execution conditions, D-1 and D-2, are recorded in section 5.1. Neither was authorized when it occurred, and this report does not authorize, waive or accept either.

### 5.1 Departures from the issued handoff

The facts of both departures were disclosed in the original report (sections 1, 5, 9 and 13), but neither was identified there as requiring authorization. They concern the execution conditions of handoff `6423207508`, not the source change's write budget (section 4.1).

- **D-1: reuse of the specification-review session.** Handoff section 1 requires implementation by a fresh Claude coding instance. The implementation was performed by session `session_01DzL8ejmvtAMrj9dVbPz8WE`, which had conducted both independent specification-review passes on #981 (section 1, prior involvement), so the implementer was not a fresh instance. The separate cross-family independent source review does not make it one.
- **D-2: local PostgreSQL 16.15 instead of PostgreSQL 17.** Handoff section 6 grants local builds and tests, and migration execution on disposable local test databases, with fresh disposable PostgreSQL 17 and Redis. Section 13 requires HOLD for an unsupported environment, without silent waiver. When PostgreSQL 17 was unavailable (section 5, item 6), local validation continued on PostgreSQL 16.15 rather than HOLD. Every local result in section 9, including the migration chain applied by the embedded test runner, is PostgreSQL 16.15 evidence. Natural CI on `postgres:17` is technical exact-version evidence; it does not retroactively authorize the local substitution.

Neither departure was authorized when it occurred, and nothing in this report, including the R1 correction, authorizes, waives or accepts either. Their forward disposition is the CTO's and is recorded on the owning issue [#987](https://github.com/thoth-pub/thoth/issues/987); that issue and PR #991 also hold the live review, readiness and merge state. The original identity, chronology, environment and evidence in this report are preserved as recorded and remain attributed to the original implementation.

## 6. Database and migration effects

Migration added: NO

No schema, table, index, data or configuration change. The only database effect is the transaction-scoped `ACCESS SHARE` lock on six tables and the transaction-local statement timeout; both end with the read's transaction.

## 7. API and compatibility effects

GraphQL/API changes: none to the public SDL; the existing sanitized `INTERNAL_ERROR` / "The Metrics read could not be completed." path can now also result from a lock wait or statement that reaches 30 s.
Generated schema/client updates: none. The public SDL generated by `thoth-client/build.rs` at the authorized base and at the candidate head is byte-identical: SHA-256 `2681b54ac18296f886451d08f5396c0a9ef8351762f63f7b3f80f319ae505496` at both; the base hash equals the hash the B review attributed to the old-M SDL.
Backwards compatibility: preserved — identical SDL, identical authorization, zero/null, coverage, ambiguity, selector and replay semantics; the internal Rust `thoth-client` compiles unchanged.
Deprecations: none.
Cross-repository dependencies: none required. `thoth-app`, `thoth-pyramid`, `thoth-dissemination`, the standalone Python `thoth-client`, `metrics-dashboard`, `metrics-widget`, `baboon`, Sphinx and the internal Rust `thoth-client` / export server are expected compatible without source edits because the public signatures and the existing error contract are unchanged; this is reasoned from the byte-identical SDL, not a fresh inspection of those repositories. Sphinx #38 is the later H3 dependency and is untouched.

## 8. Authorization and security

Authorization paths changed: none. `metricDashboard` still authorizes through `require_metrics_read_service()` before any database access, and every selected publisher's `METRICS_DASHBOARD` entitlement is still checked inside the snapshot.
Roles/scopes involved: `METRICS_READ_SERVICE` (unchanged).
Negative authorization tests: the existing matrix (anonymous, no role, publisher-scoped own and other org, superuser, dissemination worker, both service roles) and `denial_is_decided_before_any_metrics_database_access` pass unchanged; the new API tests use the read-service caller only.
Secret or personal-data handling: none; the failure message stays the fixed text and leaks no SQL, table or lock detail (the existing leak assertions remain in force).
Security limitations: none new.

## 9. Tests and checks

Environment: disposable local `thoth_test` database on PostgreSQL 16.15 (UTF8, freshly created for this task; schema applied by the embedded test runner), local Redis 7.0.15, `TEST_DATABASE_URL=postgres://thoth:thoth@localhost:5432/thoth_test`, `TEST_REDIS_URL=redis://localhost:6379`, `THOTH_EXPORT_API=https://export.thoth.pub`, toolchain `cargo 1.97.0` / `rustc 1.97.0` stable.

### Formatting

Command:

```text
cargo fmt --all -- --check
```

Result:

```text
exit 0, no output (nothing to reformat)
```

### Unit tests

Command:

```text
cargo test -p thoth-api --features backend
```

Result:

```text
     Running unittests src/lib.rs (target/debug/deps/thoth_api-5dd0e12ad663407d)
test result: ok. 2132 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 1661.93s
     Running tests/graphql_permissions.rs (target/debug/deps/graphql_permissions-762a90930dc46b12)
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.74s
   Doc-tests thoth_api
test result: ok. 0 passed; 0 failed; 8 ignored; 0 measured; 0 filtered out; finished in 0.00s
exit 0
```

### Integration/database tests

Command:

```text
cargo test --workspace
```

Result:

```text
thoth (lib): 0 passed; 0 failed — thoth (bin): 31 passed; 0 failed
thoth_api (lib): 2132 passed; 0 failed; 3 ignored; finished in 1160.87s
thoth_api tests/graphql_permissions.rs: 13 passed; 0 failed
thoth_api_server: 3 passed; 0 failed — thoth_client: 4 passed; 0 failed
thoth_errors: 13 passed; 0 failed — thoth_export_server: 144 passed; 0 failed
Doc-tests: thoth 0; thoth_api 0 passed, 8 ignored; thoth_api_server 0; thoth_client 6 passed; thoth_errors 0; thoth_export_server 2 passed — 0 failed anywhere
exit 0
```

Focused evidence command (run after the full gates, same environment, same binary):

```text
cargo test -p thoth-api --features backend -- <the fifteen test names below>
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 2120 filtered out; finished in 16.50s
```

Focused evidence (all within the runs above; names are exact):

- regression of the original race: `a_snapshot_frozen_before_a_rebuild_truncate_sees_the_projection_as_empty`
- reader-first success and queued reader: `a_reader_holding_the_pre_snapshot_lock_blocks_the_rebuild_until_it_commits` (reader paused after its lock with six `AccessShareLock` grants observed, rebuild observed waiting at `TRUNCATE TABLE`, second reader observed waiting at `LOCK TABLE` with no query sent, derived state untouched, release, both reads exact, rebuild `rebuilt: true` and verified)
- reader-first timeout: `a_reader_holding_the_lock_past_the_rebuild_lock_timeout_fails_the_rebuild_closed` (rebuild fails with the database's lock-timeout cancellation after its existing 5 s, derived state intact, locks still held by the reader, reader exact, later rebuild succeeds)
- rebuilder-first: `a_reader_arriving_during_a_rebuild_waits_before_its_snapshot_and_sees_the_rebuilt_state` (rebuild paused after `TRUNCATE`, reader observed waiting at its lock having sent no query, release, rebuilt state served)
- reader timeout and connection return: `lock_contention_past_the_statement_timeout_fails_closed_and_returns_the_connection` (`Err(Unavailable)` = `INTERNAL_ERROR` / fixed message, statements exactly checkout, `BEGIN`, planner, timeout, lock, `ROLLBACK`, pool idle again, no lock or open transaction left, exact values once the holder releases)
- incremental maintenance: `incremental_maintenance_is_not_blocked_by_a_reader_holding_the_pre_snapshot_lock`
- lock text and order: `the_pre_snapshot_lock_names_exactly_the_six_truncated_tables_in_the_verifier_order`, `the_read_path_is_one_read_only_repeatable_read_transaction_of_selects`
- ceiling and sequence: `the_statement_sequence_is_fixed_bounded_and_planned_for_its_arguments` (longest path exactly 18 including the checkout check, exact statement texts, `statement_timeout` set inside the transaction only), `the_statement_count_does_not_grow_with_works_days_or_rows`
- API boundary: `the_api_read_locks_the_projections_before_its_first_query_within_the_ceiling`, `the_api_read_waits_for_an_exclusive_projection_holder_before_its_snapshot`, plus the unchanged authorization matrix, `denial_is_decided_before_any_metrics_database_access`, `every_bounded_failure_carries_its_frozen_classification` and `the_sdl_declares_exactly_the_approved_read_contract`

### Lint/static analysis

Command:

```text
cargo clippy --all --all-targets --all-features -- -D warnings
```

Result:

```text
exit 0 — `Finished dev profile [unoptimized + debuginfo] target(s) in 1m 53s`, no Clippy warning or error. The only diagnostic is the pre-existing dependency notice `the following packages contain code that will be rejected by a future version of Rust: proc-macro-error2 v2.0.1`, unrelated to this change and present at the base.
```

### Other required checks

```text
cargo check --workspace
exit 0 — `Finished dev profile [unoptimized + debuginfo] target(s) in 3.71s` (same pre-existing proc-macro-error2 future-incompatibility notice)

git diff --check
exit 0, no output

cargo build --workspace   # runs thoth-client/build.rs at base and at head
base SDL (M2):  2681b54ac18296f886451d08f5396c0a9ef8351762f63f7b3f80f319ae505496
head SDL:       2681b54ac18296f886451d08f5396c0a9ef8351762f63f7b3f80f319ae505496
cmp: byte-identical (`cmp` exit 0); generated at head df94b550 by `cargo build --workspace` (exit 0, `Finished dev profile in 2m 03s`)
```

## 10. Manual verification

Environment: as in section 9.
Steps: none beyond the automated tests; the `#[ignore]`d query-plan and latency harness was not run (its ceiling assertion was updated to 18 and compiles).
Observed result: not applicable.
Evidence link/screenshot/log reference: the natural CI runs in section 11 and the #987 evidence comment.

## 11. CI

CI status: green on `df94b550` — every executed job of all four natural workflows succeeded; the run on the original implementation head `bea5ee67…` (the commit adding this report) starts after this commit and is recorded on PR #991 and in the original #987 evidence comment `6453540926`; each later head, including R1's, has its own natural CI, held on GitHub
Checks: `build-test-and-check` (classify, build, test, lint, format_check) run 40579140261; `check-changelog` run 40579140447; `run-migrations` run 40579140453 (classify; `run_migrations` skipped by classification); `publish-to-dockerhub` run 40579140478 (classify, build_and_push_staging_docker_image)
Failures or warnings: none. The only notices are the runners' Node.js 20 deprecation warnings for third-party actions, unrelated to this change.

## 12. Rollout and rollback

Initial state after merge: source correction only on the `feature/metrics` programme line; the read remains inactive under the existing serving, rebuild and production HOLDs; nothing is activated.
Activation required: NO (none authorized).
Feature flag/configuration: none.
Migration sequence: none.
Rollback/disable procedure: a later code revert needs separate approval and would reinstate the H1 source blocker; no production rollback or data operation is authorized.
Monitoring required: none new; a read that reaches its 30 s statement timeout surfaces as the existing sanitized `INTERNAL_ERROR`.

## 13. Known limitations and deferred work

- Local validation ran on PostgreSQL 16, not 17 (section 5, item 6); CI is the PostgreSQL 17 evidence. This is departure D-2 from the issued handoff (section 5.1), not only a limitation.
- The reader-first-timeout test necessarily waits the rebuild's existing 5 s lock timeout; the reader-first-success test releases the first reader well inside that window after observed synchronization.
- The production 30 s timeout is exercised only through the injected 250 ms timeout of the private entry; the production constant is asserted from source and in the captured statement text.
- Deferred B2 (#980) must reconcile its combined-settings statement and ceilings with this lock before it resumes; nothing in this task touches #980.
- Pre-existing documentation inaccuracy outside this task's remit: contract register section 3.7 still describes the verifier's lock as covering "exactly the four monthly projection tables", while `VERIFICATION_LOCK_SQL` at M2 names the six tables this task's section 4.2 lists; recorded for a separate documentation task.
- H2, H3, H4, Sphinx #38, terminal A2 and the fresh cumulative B review remain separately gated.

## 14. Unresolved issues

- The original entry `NONE` is withdrawn as unsupported. When this report was first committed, departures D-1 and D-2 (section 5.1) had not been dispositioned. Their forward disposition is the CTO's and is recorded on the owning issue #987, which, with PR #991, holds the live review, readiness and merge state.

## 15. Agent self-assessment

The agent may identify risks but may not approve the task.

Suggested review focus:

- the order of the three non-query statements in `evaluate` and that no query precedes the lock (source-shape test);
- that `SET LOCAL` and `LOCK TABLE` take no snapshot, so the snapshot is frozen by the publishers query after the lock is granted (the regression test observes the anomaly only under the pre-fix order);
- the private timeout-parameterized entry: its only callers are the public entry with the production constant and the module's tests;
- the concurrency tests' observed synchronization against `pg_stat_activity` and `pg_locks`, and the 5 s window of the reader-first-success test;
- the four doctrine paragraphs for truthfulness before and after each later merge, and the contract register 4.2 wording against the implemented statements.
