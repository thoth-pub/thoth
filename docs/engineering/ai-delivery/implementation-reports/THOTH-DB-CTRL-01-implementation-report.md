# THOTH-DB-CTRL-01 Implementation Report

The implementing agent completes this report after pushing the task branch and
opening the draft PR. It does not approve its own work.

## 1. Repository state

Repository: thoth-pub/thoth
Workflow: STANDARD
Base branch: develop
Base commit: `4c53709befc91acb481beac54a1d314926b61d76`
PR target: develop
Programme integration branch: None
Task branch: feature/repository-controls/thoth-db-ctrl-01
Head commit: the records/report commit (commit 2); core implementation commit is
`c7c274650e14bba53df23507a76b3366fb69860a`
Pull request: https://github.com/thoth-pub/thoth/pull/777 (draft)
Expected branch deletion after merge: YES
Final programme PR required: NO
Implementing model: Claude (Opus)
Reasoning level: HIGH

## 2. Scope confirmation

Approved specification:
`docs/engineering/ai-delivery/tasks/THOTH-DB-CTRL-01.md`
Implemented objective: one repository-authoritative, deterministic, fail-closed
Diesel schema-synchronisation procedure with an independent two-phase
baseline-to-candidate comparison over the PostgreSQL catalog, raw
`diesel print-schema`, and canonical-schema representations, gated by an explicit
version-2 expected-change manifest (`change` or `none`).

Out-of-scope changes made: NONE

The cumulative branch diff is exactly thirteen authorized paths.
`thoth-api/src/schema.rs` is byte-identical to the base; no migration, model,
GraphQL, authorization, Docker, release, or production configuration changed.

## 3. Commits

- `c7c274650e14bba53df23507a76b3366fb69860a` - feat: implement THOTH-DB-CTRL-01 Diesel control (9 paths)
- (commit 2) - docs: report THOTH-DB-CTRL-01 implementation (4 paths)

## 4. Files changed

Implementation/control (commit 1):

- `diesel.toml`
  - reason: automatic Diesel output must be untrusted staging, not the canonical schema.
  - behavioural effect: `print_schema.file = target/diesel-schema.rs` (ignored), minimal
    `custom_type_derives = ["diesel::query_builder::QueryId"]`; config now parses.
- `thoth-api/diesel-schema-control.toml`
  - reason: enumerate every intentional raw-Diesel/canonical difference as reviewable control data.
  - behavioural effect: 1 supplemental type (`MarkupFormat`), 2 table aliases, 1 column-identifier
    (`title.title`), 55 `Timestamp`->`Timestamptz` overrides, canonical table order, 54 column orders.
- `.github/scripts/diesel_schema.py`
  - reason: the fail-closed synchronizer; sole authorized writer of the canonical schema.
  - behavioural effect: `check`/`generate` modes; safe-target gate; base-ref/worktree/ledger
    validation; independent catalog/raw/canonical snapshots; exact projection comparison; deterministic
    repeat; focused compile; atomic locked write.
- `.github/scripts/test_diesel_schema.py`
  - reason: unit, disposable-database integration, and security tests.
  - behavioural effect: 53 default tests (self-provisioned disposable container), plus an opt-in
    compile test.
- `Makefile`
  - reason: bounded repository-root targets.
  - behavioural effect: `check-diesel-schema`, `generate-diesel-schema`, `test-diesel-schema`
    (local docker or CI service; `DIESEL_BIN`-supplied CLI; `DATABASE_URL` never logged).
- `.github/scripts/classify_ci_changes.py`
  - reason: the control surfaces must run migration verification, never be documentation-only.
  - behavioural effect: eight new migration-control paths + deterministic self-tests (23 cases).
- `.github/workflows/run_migrations.yml`
  - reason: run the two-phase control in CI bound to exact SHAs.
  - behavioural effect: exact base/head binding, pinned Diesel CLI 2.3.10 install, provenance,
    `make check-diesel-schema`, full-chain revert + empty-verify + reapply.
- `AGENTS.md`, `thoth-api/AGENTS.md`
  - reason: replace the stale "open control gap" text with the implemented procedure.
  - behavioural effect: authoritative canonical procedure; CG-13 retained as separately open.

Records/report (commit 2): `CHANGELOG.md`,
`docs/engineering/repository-map/control-gaps.md`,
`docs/engineering/repository-map/repositories/thoth.md`, and this report.

## 5. Implementation decisions

1. Conventions derived empirically from a migrated clean PostgreSQL 17 database, the
   canonical schema, and consuming models; validated so any unused, duplicate, broad,
   missing, or conflicting entry fails closed.
2. Comparison is capability-aware: catalog is authoritative for PostgreSQL types,
   nullability, ordinals, primary keys, foreign keys, and ordered enum labels; raw and
   canonical Diesel are authoritative for generated types, joins, and allow-table
   membership; no representation is augmented from another before comparison.
3. Rendering preserves unchanged canonical blocks byte-for-byte and renders bounded
   additions deterministically; removals/changes require explicit manifest objects.
4. Migrations are applied with the pinned CLI from the candidate root using an explicit
   `--config-file`, so the base's pre-correction `diesel.toml` never affects application.
5. `make check-diesel-schema` runs the identical synchronizer locally (disposable Docker)
   and in CI (GitHub Actions service), selected by provenance.

Deviations from the specification: NONE

## 6. Database and migration effects

Migration added: NO

Schema change required: NO. Production migration required: NO. Backfill: NO.
Production data effect: NONE. `thoth-api/src/schema.rs` base and final SHA-256:
`46cbfd0dcfc51245d39d7ce6d6e6f0888476e2f9390104001142165f625c3a3a` (identical).

Disposable validation (local PostgreSQL 17 via Docker):

- baseline migration ledger: `20250000`, `20260417`, `20260429`, `20260504`;
  candidate ledger equals it (exact prefix; no pending candidate migration).
- controlled probe `public.thoth_db_ctrl_probe(probe_id uuid PRIMARY KEY, probe_value text)`
  yielded exactly five additions (1 table, 2 columns, 1 primary key, 1 allow-table; no join)
  across catalog/raw/canonical projections; the schema-only candidate compiled with
  `cargo check -p thoth-api --features backend` without a consuming model; dropping the probe
  and running `none` restored the byte-identical baseline.
- new-enum probe: absent at baseline, present with exact ordered labels in the candidate catalog.
- index-only probe in `none` mode: empty controlled projections (excluded effect).

## 7. API and compatibility effects

GraphQL/API changes: NONE. Generated schema/client updates: NONE (canonical schema
unchanged). Backwards compatibility: preserved. Deprecations: NONE.
Cross-repository dependencies: NONE.

## 8. Authorization and security

Authorization paths changed: NONE. Roles/scopes: none.
Database verification is restricted to a local disposable target or the ephemeral CI
service; production credentials and targets are prohibited. The safe-target gate rejects
non-loopback URLs, the default developer port, wrong database prefixes, confirmation
mismatches, missing/mismatched containers, bind or named/durable mounts, and public or
unverified server addresses; it prints no URL, credential, row content, or personal data.
All subprocesses use argument arrays with `shell=False`; shell metacharacters remain literal.

## 9. Tests and checks

### Formatting

Command:

```text
cargo fmt --all -- --check
```

Result:

```text
exit 0 (no formatting differences)
```

### Unit tests

Command:

```text
python3 .github/scripts/test_diesel_schema.py
```

Result:

```text
Ran 53 tests ... OK (skipped=1: opt-in compile test)
```

Command:

```text
python3 .github/scripts/classify_ci_changes.py --self-test
```

Result:

```text
PASS all_self_tests: 23 cases
```

### Integration/database tests

Command:

```text
make check-diesel-schema DIESEL_BIN=<diesel 2.3.10> THOTH_DIESEL_BASE_REF=4c53709b...
```

Result:

```text
THOTH_DIESEL_TARGET=SAFE_DISPOSABLE_LOCAL ... THOTH_DIESEL_DELTA=EXACT_PROJECTED_MATCH
THOTH_DIESEL_CLEANUP=COMPLETE THOTH_DIESEL_REPEAT=IDENTICAL THOTH_DIESEL_DIFF=CLEAN
THOTH_DIESEL_EXPECTED_PROJECTION=NONE (exit 0; container removed; schema unchanged)
```

Command (CI-mode gate, GitHub Actions provenance, simulated against localhost:5432/thoth):

```text
GITHUB_ACTIONS=true GITHUB_REPOSITORY=thoth-pub/thoth GITHUB_JOB=run_migrations \
  GITHUB_WORKFLOW_REF=.../run_migrations.yml make check-diesel-schema
```

Result:

```text
THOTH_DIESEL_TARGET=SAFE_DISPOSABLE_CI ... EXACT_PROJECTED_MATCH (exit 0)
```

Probe compile:

```text
cargo check -p thoth-api --features backend   # over the rendered probe candidate: exit 0
```

### Lint/static analysis

Command:

```text
cargo clippy --all --all-targets --all-features -- -D warnings
```

Result:

```text
exit 0 (only a transitive proc-macro-error2 future-incompat warning)
```

Command:

```text
cargo check --workspace   # and cargo check -p thoth-api --features backend
```

Result:

```text
exit 0
```

### Other required checks

```text
git diff --check 4c53709b...HEAD   -> clean
cmp <base schema.rs> thoth-api/src/schema.rs   -> identical
```

`cargo test --workspace` requires the CI service matrix (PostgreSQL `thoth_test`
plus Redis, `TEST_DATABASE_URL`/`TEST_REDIS_URL`); the workspace-wide, database-backed
tests run in the `build-test-and-check` workflow. This change modifies no Rust, so those
results match the base; see PR #777 CI.

## 10. Manual verification

Environment: local macOS, disposable PostgreSQL 17 Docker container on an ephemeral
loopback port, pinned Diesel CLI 2.3.10 installed into a temporary root outside the
repository.
Steps: applied the base migration chain; captured independent baseline snapshots; applied
candidate migrations (none pending) and the controlled probe; compared projections;
repeated from fresh disposable state; compiled the candidate; removed all transient
resources.
Observed result: `none`-mode byte-identical no-op and the exact five-addition probe;
deterministic repeat; complete cleanup (no residual container, volume, or worktree).
Evidence: safe status lines above; PR #777 CI.

## 11. CI

CI status: PENDING (draft PR #777; exact-head migration-control CI to be recorded in the
immutable PR evidence comment).
Checks: classify -> run-migrations (heavy path), build-test-and-check, check-changelog,
publish-to-dockerhub (normal PR-safe behaviour).
Failures or warnings: to be recorded from the exact-head run.

## 12. Rollout and rollback

Initial state after merge: inactive repository-control improvement; automatic Diesel output
confined to ignored staging; canonical writes confined to the synchronizer.
Activation required: none (no runtime/production effect).
Feature flag/configuration: none.
Migration sequence: none.
Rollback/disable procedure: revert the bounded PR, restoring the previous config, tools,
convention data, AGENTS instructions, workflow, and docs; no durable database state changes.
If rollback makes generation ambiguous again, CG-12 reopens and dependent schema tasks
(including BE-01) return to BLOCKED.
Monitoring required: none; CG-13 remains open.

## 13. Known limitations and deferred work

- The deterministic renderer supports byte-preserving `none` runs and bounded table
  additions with their allow-table membership (the specified probe class). Other change
  shapes fail closed (`RENDER_UNSUPPORTED_OPERATION`) rather than emitting an incomplete
  block, and would extend the renderer under a future authorized task.
- `none` certifies only the Diesel-controlled projection; excluded migration effects
  (indexes, check constraints, data, comments) remain migration-validation responsibilities.
- CG-13 (Thoth runtime operations) is not addressed by this control.

## 14. Unresolved issues

- NONE beyond the limitations above.

## 15. Agent self-assessment

The agent identifies risks but does not approve the task.

Suggested review focus:

- the capability-aware projection matrix and that no representation is cross-populated
  before comparison (catalog vs raw vs canonical vs manifest);
- the safe-target gate's local-Docker and GitHub Actions provenance branches and address
  classification;
- manifest version-2 semantics (mode exclusivity, `change`/`none` contracts, complete
  object shapes, enum label handling);
- the atomic locked canonical writer and complete cleanup on success and forced failure.
```

CG-12 remains unresolved pending merge; CG-13 remains open; BE-01 remains blocked. No
production or durable-data effect. The implementing agent does not approve its own work.
