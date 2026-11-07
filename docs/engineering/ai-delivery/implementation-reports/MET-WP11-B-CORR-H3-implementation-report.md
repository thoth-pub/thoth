# MET-WP11-B-CORR-H3 Implementation Report

## 1. Repository state

Owning GitHub issue: [#989](https://github.com/thoth-pub/thoth/issues/989)
Parent programme-integration task: [#981](https://github.com/thoth-pub/thoth/issues/981)
Outer integration PR: [#984](https://github.com/thoth-pub/thoth/pull/984) (`feature/metrics` -> `develop`), observed OPEN / DRAFT / UNMERGED at N3/D before the branch was created (2026-10-30T16:43Z) and again at the pre-push re-check (2026-10-30T17:52:25Z); live state is held by GitHub
Repository: `thoth-pub/thoth`
Programme: Thoth Metrics, WP11-B — finite pre-B source correction, H3 third and last of H1 -> H2 -> H3 (#981 Amendment 8 as corrected by Amendment 8-C1; #989 as corrected by its correction 1)
Workflow: PROGRAMME_INTEGRATION
Risk: HIGH (canonical application authority of the managed-DRIVER batch and of unresolved-DOI reconciliation inside the CRITICAL B integration; one additive public SDL value); parent stage CRITICAL
Base branch: `feature/metrics`
Authorized base commit: `1511e66836ecaa813e6579fc2afd225b41a25a5a` (N3, tree `9f0ccd17b8b3cf6ed22e75fc61a46addecd11029`, ordered parents `5b8c47ff81f48cc7a29ed30d32376bce208245c0` and `69892b4d1cd788260f64072ac47e19d1af91377c` (merged H2 head))
Actual base commit: `1511e66836ecaa813e6579fc2afd225b41a25a5a` (tree `9f0ccd17b8b3cf6ed22e75fc61a46addecd11029`; `feature/metrics` was still N3 at 2026-10-30T17:52:25Z)
Frozen `develop` D: `b44303c214498baf76c9d9a0a7cab374377f9cbe` (tree `050e1e2954b518d62988e31d74a7d9c280b8822c`), unchanged at the same checks; no `develop` commit is imported
PR target: `feature/metrics`
Programme integration branch: `feature/metrics`
Task branch: `feature/metrics--wp11-b-corr-h3`, created from exact N3 at 2026-10-30T16:44:14Z in the isolated worktree `/home/claude/wt-h3`
Head commit: `8699205a40eefd3ca98757830ffdc2e35be319e5` (tree `3d327d72987e5862a8d77b45093277ae3e33d63d`) carries the complete source, test, contract, decision and changelog change; the commit that adds this report and the changelog PR link follows it. A commit cannot contain its own SHA, so the exact current head is the one recorded on the pull request, and the independent exact-head source review binds to that SHA.
Pull request: [#994](https://github.com/thoth-pub/thoth/pull/994), `feature/metrics--wp11-b-corr-h3` -> `feature/metrics`; opened as a draft at 2026-10-30T17:53:15Z and left draft; live review, readiness and merge state is held by GitHub
Expected branch deletion after merge: YES — but only through the stage-terminal A2 inventory, independent review, exact-name retirement authorization, guarded retirement and completion receipt that Amendment 8 item 7 defers it to; the retained refs `feature/metrics--wp11-b-corr-h1` (`2ce61d66…`) and `--h2` (`69892b4d…`) show that a merge does not delete the ref.
Final programme PR required: YES — the existing outer PR #984, separately gated.
Implementing model: Claude Fable 5.1 (`claude-fable-5-1`, as the session reports its served model), running as a subagent of the Claude Code cloud session `session_01RFKP5PX7vAhfXMtTRFPorB`, which is the control session: the implementer shares that session ID with control (the H2 precedent the handoff names) and was launched by it as a fresh instance with the issued handoff as its brief; the exact deployed build is not exposed.
Prior involvement, disclosed: none. This is a fresh instance started for H3 with no earlier role in the H3 specification, its specification review, or any H1/H2 implementation or review work; it holds no record of earlier sessions. The independent exact-head source review (a fresh Claude Opus reviewer, the CTO's recorded same-family departure) is arranged separately, and this agent issues no approval.
Reasoning level: maximum practical, as the handoff requires; the configured effort setting is not exposed to this agent.

### 1.1 Approval and authorization records

| Gate | Record | Outcome |
|---|---|---|
| Original task specification | #989 body, SHA-256 `a7f038b8138cb72ccf7ec21810ff5dbe2adc642795c10ea80da044331fab3e96` | governs technical scope |
| H3 correction 1 (F-2: Sphinx REQUIRES CHANGE via sphinx#38) | #989 [`6385432383`](https://github.com/thoth-pub/thoth/issues/989#issuecomment-6385432383), SHA-256 `2861ff5c6d71d50819612f26da26f7be5f8a27ad70b36c945f03fc7f3c264a0a` | supersedes only the Sphinx assessment |
| Sphinx consumer task record | [thoth-sphinx#38](https://github.com/thoth-pub/thoth-sphinx/issues/38) body, SHA-256 `1a1e8c4565ed7135ca0443a925239ebf8117c5bcc7f183ec4d8c4d970d64c559`, open | not implemented here |
| Parent specification | #981 Amendment 8 [`6370994558`](https://github.com/thoth-pub/thoth/issues/981#issuecomment-6370994558) and Amendment 8-C1 [`6385434946`](https://github.com/thoth-pub/thoth/issues/981#issuecomment-6385434946) | corrected eleven-record set |
| Independent CRITICAL specification review | #981 [`6385770904`](https://github.com/thoth-pub/thoth/issues/981#issuecomment-6385770904) | `APPROVED` |
| CTO specification approval, eleven hashes | #981 [`6385772335`](https://github.com/thoth-pub/thoth/issues/981#issuecomment-6385772335) | `APPROVED - BOUNDED CORRECTED SPECIFICATION ONLY` |
| H2 terminal merge evidence, source of base N3 | #988 [`6527711315`](https://github.com/thoth-pub/thoth/issues/988#issuecomment-6527711315) (rebind record [`6526666499`](https://github.com/thoth-pub/thoth/issues/988#issuecomment-6526666499)) | N3 = `1511e668…` |
| CTO H3 implementation authorization and issued handoff | #989 [`6528140158`](https://github.com/thoth-pub/thoth/issues/989#issuecomment-6528140158), posted by `ja573` at 2026-10-30T16:42:29Z | bounded actions used in section 4.2 |

The authorization comment names the approved proposed handoff by digest (`MET-WP11-B-CORR-H3-proposed-handoff.txt`, 10,241 bytes, SHA-256 `83dcd05371029639528aef56720289fa1df2d65b51c0e8c87292577a050e90f6`) and carries the issued handoff in one fenced block (11,796 bytes, SHA-256 `d41de69c2da9a5e4839bac9875be16514bb98e566492d6d8fc10800e7bd3cdcc`). The issued handoff file this agent was given, `/mnt/project-files/metrics-wp11/MET-WP11-B-CORR-H3-issued-handoff.txt`, has exactly that digest, and its text is the fenced block of the comment. The three specification digests above were re-hashed from the live GitHub bodies (raw UTF-8, no trailing newline) and match the manifest.

### 1.2 Preflight, performed before any source or branch mutation

1. Authorization comment `6528140158` exists on #989, is the one control-authored authorization record, names the handoff digests `83dcd053…` / `d41de69c…` and grants the section 6 actions (section 1.1). **PASS**
2. `feature/metrics` = N3 `1511e668…` / tree `9f0ccd17…` with ordered parents `5b8c47ff…`, `69892b4d…`; `develop` = D `b44303c2…` / tree `050e1e29…`; Sphinx `feature/metrics` `fa4b0f88751ee73617a0697d7fd6b1be221b6b74` and `develop` `e97444c20fb5626f9233397ed627a54d1a2a46d2` unchanged (both fetched read-only into a scratch clone). **PASS**
3. #989 open, two comments (correction 1 and the authorization), title and scope unchanged; #984 OPEN / DRAFT / UNMERGED at N3/D; no other H3 PR. **PASS**
4. Fail-closed namespace preflight (root `AGENTS.md` section 5.1): the slice namespace `feature/metrics--*` contained only `feature/metrics--wp11-b-corr-h1` at `2ce61d66…` and `--h2` at `69892b4d…`; `feature/metrics--wp11-b-corr-h3` and any descendant were absent; no ref was deleted, renamed or moved. **PASS**
5. Clean isolated worktree: `/home/claude/wt-h3` added detached at N3 after fetching `origin/feature/metrics`, then `git checkout -b feature/metrics--wp11-b-corr-h3` at 2026-10-30T16:44:14Z; the main checkout's branch was not touched; `git status --short` empty before edits; `user.name`/`user.email` set to `Javier Arias <javier@jarias.org>` in the worktree before any commit. **PASS**
6. Fresh disposable PostgreSQL 17 and Redis before any code: a `postgres:17` container (`PostgreSQL 17.11 (Debian 17.11-1.pgdg13+2)`, created 2026-10-30T16:45:55Z) on `localhost:5432` with the `thoth_test` database created empty for this task, and a `redis:7` container (Redis 7.4.12, created 2026-10-30T16:45:56Z) on `localhost:6379`. The container image's locally installed PostgreSQL 16 was not used as a server. **PASS**
7. The rejected-reason column `metric_import_error.error_code` is `TEXT` whose only constraint is the existing non-blank CHECK, and `metric_record_provenance.details` is unconstrained `JSONB`: no database enum or CHECK restricts the reason vocabulary, so no schema change is needed and no HOLD applies. **PASS**
8. All ten authorized existing paths read at N3; the report path was absent. The #989 line references (taken at M2) were re-located at N3: the canonical application path is `apply_canonical_application` in `metric_ingestion/mod.rs`, which takes the `metric_record` row lock `FOR UPDATE` by identity hash and then branches into duplicate/revision/conflict logic; `apply_candidate` classifies its outcome for ordinary ingestion; `process_one` in `metric_identifier_quarantine_reconciliation/mod.rs` maps the same outcome to reconciliation states. **PASS**
9. Workflows and `.github/scripts/classify_ci_changes.py` at N3 re-inspected and found as the handoff's section 7 describes (`build-test-and-check` with `postgres:17` and `redis:alpine` service containers; `check-changelog`; `run-migrations` gated on `run_migrations`; `publish-to-dockerhub` pushing `ghcr.io/thoth-pub/thoth:staging-pr-<n>` for draft PRs). The classifier run locally on the eleven changed paths printed `{"docs_only": "false", "run_build": "true", "run_docker": "true", "run_migrations": "false"}`. **PASS**
10. Base mismatch: none; N3 and D unchanged at every check. **PASS**

### 1.3 Mechanism and fix evidence

At N3 `apply_canonical_application` locked the existing `metric_record` of the candidate's v1 identity and went straight to the content-hash comparison, so a one-day `REPORTING_PERIOD` observation of a `DAY` record (or a one-month `REPORTING_PERIOD` observation of a `MONTH` record, in either order) with the same content was accepted as a `DUPLICATE`, with different content from the winner as a `REVISION`, and with different content from another account as a `CONFLICT`; the record's grain was never compared. The regression tests observe exactly that at N3: run with the two `mod.rs` files at N3 and only `error.rs` (the additive enum value) and the new tests applied (2026-10-30T17:01:31Z to 17:05:44Z, `test result: FAILED. 19 passed; 10 failed`), every new test failed with the silent acceptance — `a_mismatched_reporting_grain_for_the_same_identity_is_rejected_in_either_order` with `left: Duplicate, right: Rejected`, `mismatched_grains_in_one_batch_keep_the_first_row_and_reject_the_later_one` with `[(Winner, None), (Duplicate, None), (Winner, None), (Duplicate, None)]`, `concurrent_first_arrival_with_different_grains_commits_one_record_and_rejects_the_other` with `[[Duplicate], [Winner]]`, the reconciliation tests with `RESOLVED_DUPLICATE` / `RESOLVED_REVISION` / `BLOCKED_SOURCE_CONFLICT` counted as `(3, 2, 0, 1)` instead of `(3, 0, 0, 3)`, and the GraphQL mutation reporting `resolved: 1` instead of `blocked: 1`.

After the fix, immediately after the `FOR UPDATE` read of the existing record and before `current_revision_id` is even resolved, `apply_canonical_application` compares `record.reporting_grain` with `candidate.reporting_grain` and returns the new internal outcome `CanonicalApplicationOutcome::ReportingGrainMismatch` on a difference; nothing is inserted or updated. `apply_candidate` maps that outcome through the existing `write_rejected` path to `REJECTED` / `MetricIngestionErrorCode::ReportingGrainMismatch` (`REPORTING_GRAIN_MISMATCH`, field `reporting_grain`, static message "The reporting grain does not match the existing canonical record"), carrying the identity and content hashes and no record id; `process_one` maps it to the existing retryable `BLOCKED_INCONSISTENT_EVIDENCE` state with no record or revision id. The same tests pass on the corrected tree (section 9), and the same-grain duplicate, revision and conflict paths that follow a rejection in the tests are unchanged.

## 2. Scope confirmation

Exactly the #989 corrected scope, as the issued handoff section 9 binds it: the canonical reporting grain is immutable for its v1 identity; the grain is compared under the existing row lock before any duplicate/conflict/revision decision; a mismatch is the distinct internal `ReportingGrainMismatch` outcome; ordinary ingestion rejects the observation with the one additive closed-vocabulary reason through the existing rejected-provenance, import-error, `invalid_count` and replay paths; unresolved-DOI reconciliation maps it to the existing `BLOCKED_INCONSISTENT_EVIDENCE` state with the unchanged capped backoff and no mutation of quarantine, provenance, import or coverage evidence; all v1 hash bytes, goldens and `hash.rs` are unchanged; no historical record is normalized and no committed outcome or replay changes; changelog, contract register and decision summary are updated. No H4 work, no Sphinx or other-repository edit, no schema, migration, manifest, workflow, policy or generated-artifact change.

## 3. Commits

| Commit | Tree | Author / committer | Content |
|---|---|---|---|
| `8699205a40eefd3ca98757830ffdc2e35be319e5` | `3d327d72987e5862a8d77b45093277ae3e33d63d` | `Javier Arias <javier@jarias.org>` / same | source, tests, contract register, decision summary, changelog entry |
| (report commit; SHA recorded on the PR) | | `Javier Arias <javier@jarias.org>` / same | this report and the changelog PR link |

Both commits were checked with `git log --format='%an <%ae> | %cn <%ce>'` before each push; every line reads `Javier Arias <javier@jarias.org> | Javier Arias <javier@jarias.org>`, and neither commit message carries a `Co-Authored-By` or `Claude-Session` trailer (standing CTO rule of 2026-10-30).

## 4. Files changed

| Path | Change |
|---|---|
| `thoth-api/src/model/metric_ingestion/error.rs` | header doc amended; one additive variant `ReportingGrainMismatch` placed after `OverlappingPeriod` in the canonical-outcomes group; `field_name` returns `reporting_grain` for it; `message` returns the static sanitized text |
| `thoth-api/src/model/metric_ingestion/mod.rs` | module doc section "Canonical reporting grain (`MET-WP11-B-CORR-H3`)"; `CanonicalApplicationOutcome::ReportingGrainMismatch`; the grain comparison under the row lock in `apply_canonical_application`; the `write_rejected` arm in `apply_candidate` |
| `thoth-api/src/model/metric_ingestion/tests.rs` | inventory, field and message assertions for the new code; the N3 test `a_different_grain_label_for_the_same_identity_is_not_a_new_identity`, which asserted the silent `DUPLICATE` acceptance the correction removes, is replaced by seven tests (section 9) |
| `thoth-api/src/model/metric_identifier_quarantine_reconciliation/mod.rs` | doc on `BlockedInconsistentEvidence`; the `ReportingGrainMismatch` arm in `process_one` |
| `thoth-api/src/model/metric_identifier_quarantine_reconciliation/tests.rs` | two tests (section 9) |
| `thoth-api/src/graphql/metric_ingestion_lifecycle_tests.rs` | the vocabulary test now expects 48 values including `REPORTING_GRAIN_MISMATCH`; grain-parameterised batch document; one API-boundary test with a uniquely named, self-removing sentinel trigger |
| `thoth-api/src/graphql/metric_identifier_quarantine_reconciliation_tests.rs` | one API-boundary test |
| `docs/metrics/contract-register.md` | 3.2: the additive value, the canonical reporting-grain immutability bullet; 3.5: the extended meaning of `BLOCKED_INCONSISTENT_EVIDENCE`; section 8: the recorded consumer disposition |
| `docs/metrics/decisions.md` | new shared decision "Canonical reporting grain is immutable for its v1 identity"; implementation invariant 13 |
| `CHANGELOG.md` | Unreleased / Fixed entry (PR link added by the report commit) |
| `docs/engineering/ai-delivery/implementation-reports/MET-WP11-B-CORR-H3-implementation-report.md` | new (report commit) |

### 4.1 Write-budget compliance

PASS — `git diff --stat 1511e66836ecaa813e6579fc2afd225b41a25a5a` before the first commit listed exactly the ten authorized existing paths (`10 files changed, 1223 insertions(+), 27 deletions(-)`), and the report commit adds only the one authorized new file and the changelog PR link. `thoth-api/src/model/metric_ingestion/hash.rs` is byte-identical to N3 (SHA-256 `b756c3176541f118bfb4d4a322b9a6a57c37c7d455fdaf7ffdecb202319fa0db` at N3 and at head); `Cargo.lock`, the Cargo manifests, `thoth-api/src/schema.rs`, migrations, workflows, the classifier, authorization/policy code and every other path are unchanged. The generated SDL was produced only by `thoth-client/build.rs` during the workspace build and was never edited or committed. Nothing was deleted, moved or renamed.

### 4.2 Authorized actions actually used

- repository inspection: USED (GitHub reads through the `mcp__github__` tools only; `git ls-remote`; local object reads; read-only shallow clones of consumer repositories into the session scratch directory)
- source edit: USED, within the ten authorized paths
- new file creation: USED, the one authorized report
- file deletion/move/rename: NOT USED
- branch creation: USED, `feature/metrics--wp11-b-corr-h3` at exact N3 in an isolated worktree
- commit: USED, two bounded commits as `Javier Arias <javier@jarias.org>` (author and committer); `Cargo.lock` unchanged
- push: USED, only the named task branch (`git push -u origin feature/metrics--wp11-b-corr-h3` over HTTPS)
- PR creation/update: USED, one draft PR (#994) targeting `feature/metrics`; the creation tooling appended its own one-line "Generated by Claude Code" footer carrying the control session link, which was left in place as part of the required disclosure; the body was not otherwise updated; not marked ready
- issue/comment mutation: USED only for the bounded implementation/test/CI evidence comment on #989 that the handoff permits; no other issue, the parent #981, #984, #988, sphinx#38 or any PR body/state was mutated beyond the one PR described above
- manual CI dispatch/rerun: NOT USED
- provider/runtime read: NOT USED
- provider/runtime write: NOT USED
- migration execution: USED — the embedded test runner applied the existing migration chain to the fresh disposable local `thoth_test` database on the `postgres:17` container (PostgreSQL 17.11), as the handoff grants. No `cargo run migrate` and no non-disposable target.
- release/tag/publication: NOT USED
- merge: NOT USED
- deployment: NOT USED
- production activation: NOT USED
- other: local environment preparation inside the disposable cloud container only — the `postgres:17` and `redis:7` Docker containers on `localhost`, `THOTH_EXPORT_API=https://export.thoth.pub` set exactly as the CI workflows set it, `CARGO_INCREMENTAL=0` and a shared `CARGO_TARGET_DIR` to fit the container's disk; one leftover test function from an aborted focused run was dropped from the disposable database by hand (section 9)

Unauthorized actions performed: none identified against the issued handoff's path budget (section 4.1), action matrix (above) and execution conditions: a fresh instance implemented (section 1), local validation ran on a fresh disposable PostgreSQL 17 (section 9), no ref was deleted or moved, nothing was rebased, imported or force-pushed, no other repository was written, and no production, registry or provider access occurred.

### 4.3 Automatic and manual external effects

Automatic CI/provider effects observed:

Observed on the first commit `8699205a40eefd3ca98757830ffdc2e35be319e5` (PR [#994](https://github.com/thoth-pub/thoth/pull/994)). All four workflows were triggered by the push and PR creation alone (`pull_request` event) at 2026-10-30T17:53:19Z; nothing was dispatched, re-run or cancelled:

- `build-test-and-check` run [41102197537](https://github.com/thoth-pub/thoth/actions/runs/41102197537) (classify, build, test, lint, format_check);
- `check-changelog` run [41102197629](https://github.com/thoth-pub/thoth/actions/runs/41102197629);
- `run-migrations` run [41102198070](https://github.com/thoth-pub/thoth/actions/runs/41102198070) (classify; `run_migrations` expected skipped by classification);
- `publish-to-dockerhub` run [41102197820](https://github.com/thoth-pub/thoth/actions/runs/41102197820) (classify, build_and_push_staging_docker_image toward the automatic `ghcr.io/thoth-pub/thoth:staging-pr-994` image).

Their state at the evidence cutoff is recorded in section 11. The commit that adds this report triggers the same four workflows once more; every later result post-dates this report and is held on PR #994 and in the #989 evidence comment (job logs only; no registry access).

Manually initiated external actions: NONE

External writes/publication: none initiated by hand; the automatic `publish-to-dockerhub` workflow publishes `ghcr.io/thoth-pub/thoth:staging-pr-994`, the staging publication the handoff accepts; its digest is held in that job's log (no registry access).

## 5. Implementation decisions

1. The comparison lives in the shared authority, not in its callers. `apply_canonical_application` is the single winner/duplicate/revision/conflict authority for ordinary ingestion and quarantine reconciliation, so the grain check placed right after its `FOR UPDATE` read of the existing record is observed by both routes by construction, under the same lock and before `current_revision_id` is resolved or the current revision is locked. No second comparison exists in `apply_candidate` or `process_one`.
2. A distinct outcome variant rather than a reuse of `OverlappingPeriod` or `SourceConflict`: `ReportingGrainMismatch` carries no record or revision id, so neither caller can attach the rejected evidence to the record, and the exhaustive matches in both callers had to name it explicitly (the compiler enforces the mapping).
3. The additive enum value is placed in the "canonical outcomes" group of `MetricIngestionErrorCode`, directly after `OverlappingPeriod`; its SCREAMING_SNAKE_CASE name follows from the existing `strum`/`serde` derives and Juniper exposes it as the one new SDL value. `field_name` reuses the existing `reporting_grain` field and `message` is a fixed string under 120 bytes; `raw_value` stays `NULL` as for every other rejection.
4. Reconciliation maps to the existing `BlockedInconsistentEvidence` with `(None, None)` through the unchanged `persist_state`, so the capped 1h/2h/4h/8h/16h/24h schedule, `attempt_count`, `resolved_at IS NULL` and the absence of any evidence mutation are the existing behaviour; the variant's doc and the two registers record the extended meaning.
5. Replay is unchanged by construction: the rejection is persisted as ordinary `REJECTED` provenance with `details.reason_code = "REPORTING_GRAIN_MISMATCH"`, which the existing replay path parses with `from_str`; a batch committed before the correction keeps its persisted `DUPLICATE` outcome and is replayed from it without re-evaluation (test `a_historical_batch_committed_before_the_correction_replays_its_committed_outcome`).
6. The N3 test that pinned the old behaviour (`a_different_grain_label_for_the_same_identity_is_not_a_new_identity`) asserted a `DUPLICATE` for exactly the case the correction rejects, so it was replaced rather than kept; its identity claim (the grain does not create a new identity) is retained by the new tests, which assert equal identity hashes.
7. Deterministic concurrency in the tests uses observed barriers only: a `Barrier` for simultaneous first arrival; a held `FOR UPDATE` transaction on its own session plus a pinned single-connection pool, `pg_stat_activity` `wait_event_type = 'Lock'` and the backend's last query (`… FOR UPDATE`) to prove the mismatched batch queues on the canonical row lock before deciding; no sleeps as proofs.
8. The GraphQL database-failure case uses a `BEFORE INSERT` trigger on `metric_import_error` with a per-run unique name and a `Drop` guard that removes it, after a first focused run that aborted left a fixed-name function behind in the shared disposable database (section 9).
9. Environment: the `postgres:17` and `redis:7` containers were bound to `localhost`; `THOTH_EXPORT_API=https://export.thoth.pub` is required to compile `thoth-export-server`, as in the workflows; `CARGO_INCREMENTAL=0` and a shared target directory were used to fit the container's disk. The commands, toolchain and dependencies are unchanged.

List any deviation from the specification requiring authorization: none identified. This report authorizes nothing and claims no approval.

## 6. Database and migration effects

Migration added: NO

No schema, table, column, index, enum, data or configuration change. The new reason value is stored in the existing unconstrained `TEXT` column `metric_import_error.error_code` and inside the existing `JSONB` provenance details; the reconciliation state written is an existing value of the existing database enum.

## 7. API and compatibility effects

GraphQL/API changes: exactly one additive value, `REPORTING_GRAIN_MISMATCH`, on the existing enum `MetricIngestionErrorCode`, reachable only through `MetricIngestionRowResult.reasonCode` of the protected `ingestMetricBatch` mutation; no operation, input, object, field, state or aggregate changes.
Generated schema/client updates: the public SDL generated by `thoth-client/build.rs` is SHA-256 `2681b54ac18296f886451d08f5396c0a9ef8351762f63f7b3f80f319ae505496` at N3 and `7ee89f526cb02dbdaa6439aa9d5712a6f34151a77712db314c449b5fc3007068` at head; the unified diff is exactly the six added lines of one enum value (its four description lines, the value `REPORTING_GRAIN_MISMATCH` and the reopened description quote of the following value), with no removed line:

```text
@@ -2176,6 +2176,12 @@
     dimensional cell already covers an overlapping half-open period.
   """ OVERLAPPING_PERIOD
   """
+    Per-observation `REJECTED` (`MET-WP11-B-CORR-H3`, #989): the existing
+    canonical record of the same v1 identity was persisted with a different
+    reporting grain. The canonical grain is immutable for its identity, so
+    the evidence is neither a duplicate, a conflict nor a revision.
+  """ REPORTING_GRAIN_MISMATCH
+  """
     The `CONFLICT` reason: same identity, different content, from a DRIVER
     account that is not the canonical winner.
   """ SOURCE_CONFLICT
```

Backwards compatibility: additive on the producer; not automatically compatible for a closed-enum consumer. The internal Rust `thoth-client` and the export server compile unchanged in the workspace build (their queries do not select `reasonCode`).
Deprecations: none.
Cross-repository dependencies and consumer table:

| Consumer | Identity inspected | Evidence | Disposition |
|---|---|---|---|
| thoth-sphinx | `feature/metrics` `fa4b0f88…` (producer pin `52483ae2…`), `develop` `e97444c2…` (pin `8305de9d…`) | both subset SDLs enumerate 47 `MetricIngestionErrorCode` values and lack `REPORTING_GRAIN_MISMATCH`; the batch decoders map an unknown reason to `ContractViolation`, non-retryable in the retry classifier | **REQUIRES CHANGE** — owned by [thoth-sphinx#38](https://github.com/thoth-pub/thoth-sphinx/issues/38) against the merged H3 producer; recorded, not implemented |
| thoth-app | `cfbeb3f8…` (default branch, shallow clone) | no reference to `ingestMetricBatch`, `reasonCode` or `MetricIngestionErrorCode` | REMAINS COMPATIBLE |
| thoth-dissemination | `78181308…` | no such reference | REMAINS COMPATIBLE |
| Python thoth-client | `363ec737…` | no such reference | REMAINS COMPATIBLE |
| metrics-dashboard | `92d90380…` | no such reference | REMAINS COMPATIBLE |
| metrics-widget | `69dce715…` | no such reference | REMAINS COMPATIBLE |
| internal Rust thoth-client / export server | this workspace at head | `thoth-client/assets/queries.graphql` does not select `reasonCode`; both crates compile and their tests pass in `cargo test --workspace` | REMAINS COMPATIBLE |
| thoth-pyramid | not inspectable (private repository; clone and GitHub read refused for this session) | reasoned from `docs/engineering/repository-map/contracts.md`: a public-schema codegen (`graphql-codegen`) and metadata-export consumer, not a `METRICS_INGEST_SERVICE` caller; the enum is reachable only through the protected mutation's row result | REMAINS COMPATIBLE (reasoned, not live-inspected; section 13) |
| baboon | not inspectable (same) | same map: public GraphQL schema and metadata export API consumer (`src/thoth_graphql.rs`, `src/marc_export.rs`), not a `METRICS_INGEST_SERVICE` caller | REMAINS COMPATIBLE (reasoned, not live-inspected; section 13) |

No new `REQUIRES CHANGE` beyond Sphinx was found, so no HOLD applies.

## 8. Authorization and security

Authorization paths changed: none. `ingestMetricBatch` and `reconcileMetricIdentifierQuarantine` still authorize `METRICS_INGEST_SERVICE` before any operation-specific database access; the new API-boundary tests re-run the denied-principal matrix against the mismatched batch and assert no state change, and the existing matrices pass unchanged.
Roles/scopes involved: `METRICS_INGEST_SERVICE` (unchanged).
Negative authorization tests: existing lifecycle and reconciliation matrices unchanged and passing; the new GraphQL test adds the denial of every negative principal for the mismatched batch.
Secret or personal-data handling: none; the fixed message carries no SQL, constraint, grain value, connection or caller input, and a database failure while writing the rejection evidence is the existing sanitized `INTERNAL_DATABASE_ERROR` (asserted with an injected sentinel error whose text does not leak).
Security limitations: none new.

## 9. Tests and checks

Environment: disposable `thoth_test` database on a local `postgres:17` Docker container (PostgreSQL 17.11, created empty for this task; schema applied by the embedded test runner), `redis:7` container (Redis 7.4.12), `TEST_DATABASE_URL=postgres://thoth:…@localhost:5432/thoth_test`, `TEST_REDIS_URL=redis://localhost:6379`, `THOTH_EXPORT_API=https://export.thoth.pub`, toolchain `rustc 1.97.0 (2d8144b78 2026-07-07)`, `cargo 1.97.0 (c980f4866 2026-06-30)`, `clippy 0.1.97 (2d8144b788 2026-07-07)`; `diesel 2.3.10`, `juniper 0.16.2`, `r2d2 0.8.10` from the unchanged `Cargo.lock`. The workflows pin no toolchain and use the `ubuntu-latest` runner's preinstalled stable Rust, which may be newer than the local 1.97.0; natural CI is the exact-stack evidence. All results below are from the worktree at the first commit's tree, in one sequential pass started 2026-10-30T17:16:23Z (`CARGO_INCREMENTAL=0`) unless stated otherwise.

### Formatting

```text
cargo fmt --all -- --check
```

```text
exit 0, no output (nothing to reformat); 2026-10-30T17:16:23Z to 17:16:26Z
```

### Compile

```text
cargo check --workspace
```

```text
exit 0; 2026-10-30T17:16:26Z to 17:19:35Z; the only notice is the pre-existing future-incompatibility warning for the dependency `proc-macro-error2 v2.0.1`, unrelated to this change. `cargo build --workspace` (17:19:35Z to 17:21:49Z) then generated the head SDL hashed in section 7.
```

### Unit tests

```text
cargo test -p thoth-api --features backend
```

```text
exit 0; 2026-10-30T17:24:15Z to 17:36:31Z
lib: test result: ok. 2158 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 722.52s
tests/graphql_permissions.rs: test result: ok. 13 passed; 0 failed; 0 ignored
doc-tests: test result: ok. 0 passed; 0 failed; 8 ignored
```

### Integration/database tests

```text
cargo test --workspace
```

```text
exit 0; 2026-10-30T17:36:31Z to 17:51:46Z
thoth (bin): 31 passed; thoth_api lib: 2158 passed, 0 failed, 3 ignored (717.79s); graphql_permissions: 13 passed; thoth_api_server: 3 passed; thoth_client: 4 passed (+6 doc-tests); thoth_errors: 13 passed; thoth_export_server: 144 passed (+2 doc-tests); 0 failed anywhere
```

Focused regression evidence (same environment):

```text
# before the fix: both mod.rs files at N3, error.rs and the new tests applied (2026-10-30T17:01:31Z to 17:05:44Z)
cargo test -p thoth-api --features backend --no-fail-fast -- grain a_day_record_blocks a_historical_batch_committed the_three_domain_vocabularies import_error_field_names error_codes
test result: FAILED. 19 passed; 10 failed; 0 ignored  (the ten new behavioural tests failed on the silent acceptance; section 1.3)

# after the fix (2026-10-30T17:13:19Z to 17:16:23Z), same filter
test result: ok. 29 passed; 0 failed; 0 ignored
```

The eleven new tests and their neighbours:

- `model::metric_ingestion::tests::a_mismatched_reporting_grain_for_the_same_identity_is_rejected_in_either_order` — DAY→one-day REPORTING_PERIOD, the reverse, MONTH→one-month REPORTING_PERIOD and the reverse: the later evidence is `REJECTED` / `REPORTING_GRAIN_MISMATCH` with the winner's identity and content hashes and no record id; the record keeps its grain and value; only batch, provenance and import-error rows grow; counters `[2, 1, 0, 0, 0, 1]`
- `…::the_grain_is_checked_before_duplicate_revision_and_conflict_classification` — same and different content from the winner, same and different content from the losing account, all rejected with no duplicate, revision or conflict provenance; the same-grain revision, duplicate and conflict afterwards behave as before
- `…::mismatched_grains_in_one_batch_keep_the_first_row_and_reject_the_later_one` — within one batch the first arrival wins with its own grain for two identities and the later rows are rejected
- `…::concurrent_first_arrival_with_different_grains_commits_one_record_and_rejects_the_other` — barrier-synchronised first arrival from two accounts and from one account: exactly one winner carrying its own grain, one rejection, no duplicate or conflict
- `…::the_grain_check_waits_for_the_canonical_row_lock_and_rejects_once_it_is_released` — the mismatched batch is observed waiting on the `FOR UPDATE` row lock held by another session and rejects only after release
- `…::a_reporting_grain_mismatch_replays_its_committed_rejection_and_writes_nothing` — three replays (including on a fresh pool) return the committed rows and write nothing; the same batch submitted concurrently commits once and replays once
- `…::a_historical_batch_committed_before_the_correction_replays_its_committed_outcome` — a pre-H3 `DUPLICATE` outcome seeded as committed provenance is replayed as persisted, never re-evaluated
- `model::metric_identifier_quarantine_reconciliation::tests::a_reporting_grain_mismatch_blocks_as_retryable_inconsistent_evidence` — a REPORTING_PERIOD winner, then DAY evidence with the same content, different content and from the losing account: all `BLOCKED_INCONSISTENT_EVIDENCE`, attempt 1, 3600 s, `resolved_at` null, no record/revision id; historical snapshots (import, provenance, quarantine, coverage, checkpoint) unchanged; retry after `mark_due` gives attempt 2, 7200 s
- `…::a_day_record_blocks_later_one_day_reporting_period_evidence` — the reverse order blocks; a later same-grain duplicate still resolves
- `graphql::metric_ingestion_lifecycle_tests::a_reporting_grain_mismatch_is_a_bounded_rejected_row_with_the_exact_reason` — exact `REJECTED` / `REPORTING_GRAIN_MISMATCH` row with equal identity hash and null `recordId`; sanitized `INTERNAL_DATABASE_ERROR` under an injected sentinel failure with no leak and no state change; every denied principal refused without state change; bounded static import error; committed rejection replays unchanged
- `graphql::metric_identifier_quarantine_reconciliation_tests::a_reporting_grain_mismatch_is_counted_as_blocked_without_new_vocabulary` — the mutation reports `blocked: 1`, the state is `BLOCKED_INCONSISTENT_EVIDENCE`, the lifecycle snapshot is unchanged and the aggregate result exposes no new vocabulary
- `graphql::metric_ingestion_lifecycle_tests::the_three_domain_vocabularies_are_exposed_with_exactly_their_authoritative_values` (modified) — 48 SDL values, one per Rust variant, including `REPORTING_GRAIN_MISMATCH`
- v1 goldens: the existing hash tests (`reporting_grain_affects_the_request_hash_but_neither_canonical_hash`, the golden-vector tests in `metric_ingestion/tests.rs` and `hash.rs`) pass unchanged in both full suites with `hash.rs` byte-identical.

### Lint/static analysis

```text
cargo clippy --all --all-targets --all-features -- -D warnings
```

```text
exit 0; 2026-10-30T17:21:49Z to 17:24:15Z; no warning or error (the same pre-existing `proc-macro-error2` future-incompatibility notice only)
```

### Other required checks

```text
git diff --check
```

```text
exit 0, no output; 2026-10-30T17:51:46Z
```

```text
python3 .github/scripts/classify_ci_changes.py --paths <the eleven changed paths>
{"docs_only": "false", "run_build": "true", "run_docker": "true", "run_migrations": "false"}
```

Chronology note: the first focused post-fix run (17:06:04Z to 17:09:5xZ) failed only in the new GraphQL lifecycle test because the aborted red run had left a fixed-name sentinel function in the shared disposable database (`function … already exists`); the second (17:09:59Z to 17:12:56Z) failed only on the import-error `row_number` expectation (the GraphQL document submits `sourceRowNumber: 0`, and `metric_import_error.row_number` is the source row number, so the expectation `1` was corrected to `0`). The leftover function was dropped by hand, the sentinel was made uniquely named with a `Drop` guard, and the third focused run and both full suites passed. No production code changed between those runs.

## 10. Manual verification

Environment: as in section 9.
Steps: none beyond the automated tests.
Observed result: not applicable.
Evidence link/screenshot/log reference: the natural CI runs in section 11 and the #989 evidence comment.

## 11. CI

CI status: PENDING at the evidence cutoff (2026-10-30T18:01Z, before the report commit) — on `8699205a…` the `classify` jobs of the three classifying workflows had run and released their downstream jobs (`build` job 124221767474 in progress since 17:54:57Z; `format_check` 124221767427, `test` 124221768774 and `lint` 124221769207 created at 17:54:54Z and queued/starting), `check-changelog` (job 124221167651) and the `run-migrations` `classify` (job 124221168695) were in progress, and `build_and_push_staging_docker_image` (job 124221378096) had been running since 17:54:01Z toward `ghcr.io/thoth-pub/thoth:staging-pr-994`. No job had failed. The run on the commit that adds this report starts after this commit; every later head has its own natural CI, held on GitHub and reported in the #989 evidence comment.
Checks: `build-test-and-check` (classify, build, test, lint, format_check) run 41102197537; `check-changelog` run 41102197629; `run-migrations` run 41102198070 (classify; `run_migrations` expected skipped by classification); `publish-to-dockerhub` run 41102197820 (classify, build_and_push_staging_docker_image).
Failures or warnings: none observed at the cutoff.

## 12. Rollout and rollback

Initial state after merge: source correction only on the `feature/metrics` programme line; the managed-DRIVER lifecycle and reconciliation remain inactive under the existing collection, serving and production HOLDs; nothing is activated.
Activation required: NO (none authorized).
Feature flag/configuration: none.
Migration sequence: none.
Rollback/disable procedure: before any runtime write a revert is source-only and would reinstate the silent acceptance; after any runtime use, persisted `REPORTING_GRAIN_MISMATCH` values in `metric_import_error.error_code` and provenance details mean a revert needs a reviewed compatible forward repair, because the old closed decoders cannot read them. No production rollback or data operation is authorized.
Monitoring required: none new; rejections appear in the existing import-error and provenance ledgers and reconciliation blocks in the existing state table. Collection against the H3 producer is blocked until sphinx#38 lands.

## 13. Known limitations and deferred work

- The repair of already-committed mismatched evidence is out of scope by specification: records whose grain was silently accepted before H3 keep their first-arrival grain, their committed `DUPLICATE` / `REVISION` / `CONFLICT` provenance stays as committed, and no normalization, hash v2 or repair migration is started.
- Sphinx `REQUIRES CHANGE` (sphinx#38) is recorded, not implemented; a Sphinx at the pinned identities that receives a committed `REPORTING_GRAIN_MISMATCH` row treats it as a non-retryable contract violation, so collection against this producer must wait for that task.
- thoth-pyramid and baboon are private and could not be read by this session (clone and GitHub reads refused); their `REMAINS COMPATIBLE` dispositions are reasoned from the documented contract map and the enum's reachability only through the protected mutation, not from a live source inspection. Control may wish to confirm them.
- The local toolchain is `rustc 1.97.0`; the workflows pin none and use the runner's current stable, so the CI compiler may be newer; natural CI is the exact-stack evidence.
- The row-lock test's 20 s polling deadline is a hang guard that heavy CI load could in principle reach; its assertions use observed barriers only.
- H4 (#990), sphinx#38, terminal A2 and the fresh cumulative B review remain separately gated; nothing here starts them.

## 14. Unresolved issues

- None known at the evidence cutoff (2026-10-30T18:01Z). The independent exact-head source review, readiness, true merge and terminal A2 are open and separately gated; the live review, readiness and merge state is held on PR #994 and the owning issue #989.

## 15. Agent self-assessment

The agent may identify risks but may not approve the task.

Suggested review focus:

- the position of the grain comparison in `apply_canonical_application`: after the `FOR UPDATE` read of the existing record and before `current_revision_id` is resolved, the current revision is locked or any content comparison happens, with no write on that path;
- the two exhaustive match arms (`apply_candidate` and `process_one`) and that no other consumer of `CanonicalApplicationOutcome` exists;
- the replacement of the N3 test `a_different_grain_label_for_the_same_identity_is_not_a_new_identity`, whose identity claim is retained by the new tests' equal-identity-hash assertions;
- the wording of the new contract-register bullet, the 3.5 extension of `BLOCKED_INCONSISTENT_EVIDENCE`, the decision entry and invariant 13 against the implemented statements;
- the consumer table, in particular the two reasoned (not live-inspected) private-repository dispositions.
