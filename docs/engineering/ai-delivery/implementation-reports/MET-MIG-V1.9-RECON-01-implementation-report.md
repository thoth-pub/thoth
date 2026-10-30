# MET-MIG-V1.9-RECON-01 Implementation Report

Task: `MET-MIG-V1.9-RECON-01` — reconcile the unreleased Metrics migration
directory release/display suffix to `v1.9.0`
Issue: [868](https://github.com/thoth-pub/thoth/issues/868)
Programme: Thoth Metrics / [766](https://github.com/thoth-pub/thoth/issues/766)
Workflow: `PROGRAMME_INTEGRATION`
Risk: **MEDIUM**
Date: 2026-08-31

This report is **evidence only**. It is not self-approval. The implementing
agent did not review, approve or merge its own work.

## 1. Repository state

### 1.1 Controlling authority

Implementation executed strictly under the complete effective specification:

1. the original #868 issue body;
2. controlling specification amendment
   [`5461405206`](https://github.com/thoth-pub/thoth/issues/868#issuecomment-5461405206)
   — controls every conflict with the original body;
3. independent specification approval
   [`5461505656`](https://github.com/thoth-pub/thoth/issues/868#issuecomment-5461505656);
4. post-refresh exact-base/inventory rebind
   [`5475285197`](https://github.com/thoth-pub/thoth/issues/868#issuecomment-5475285197);
5. CTO specification approval
   [`5475338856`](https://github.com/thoth-pub/thoth/issues/868#issuecomment-5475338856);
6. exact-SHA implementation authorization
   [`5475388112`](https://github.com/thoth-pub/thoth/issues/868#issuecomment-5475388112).

Per amendment `5461405206`, the stale pre-amendment HIGH-risk / "migration
identity" reasoning in the original body is superseded. The controlling
terminology is **migration directory release/display suffix** and the
controlling risk is **MEDIUM**.

### 1.2 Exact refs

| Item | Value |
|---|---|
| Repository | `thoth-pub/thoth` |
| Authorized base branch / PR target | `feature/metrics` |
| Exact authorized base commit | `5604a447c1744f4cd5cc715c15a43d156cc0e62b` |
| Base tree | `9511ead35bd4c3421f153ecae24348ab487c8f25` |
| Task branch | `feature/metrics--v1.9-migration-reconcile` |
| Implementation head | recorded in section 3 |

No rebase onto, and no silent adoption of, a later `feature/metrics` head
occurred. `feature/metrics` was never mutated directly. Work was carried out in
an isolated `git worktree` created exactly from the authorized base.

### 1.3 Preflight evidence

All eight mandatory preflight checks passed before any repository mutation.

| # | Check | Result |
|---|---|---|
| 1 | remote state fetched | PASS — `git fetch origin --prune` |
| 2 | `feature/metrics` exact head | PASS — `5604a447c1744f4cd5cc715c15a43d156cc0e62b` |
| 3 | base tree | PASS — `9511ead35bd4c3421f153ecae24348ab487c8f25` |
| 4 | task branch absent | PASS — absent locally and on `origin` |
| 5 | no competing open PR from that head | PASS — no open PR used that head; no open PR targeted `feature/metrics` |
| 6 | no competing current ACTIVE execution claim on #868 | PASS — claims `5461402296`, `5461503692`, `5475280685`, `5475336778`, `5475382977` were all RELEASED COMPLETE |
| 7 | migration inventory | PASS — exactly `20260826_v1.8.0/`, `20260827_v1.8.0/`, `20260828_v1.8.0/`; no fourth `_v1.8.0` Metrics migration |
| 8 | reference / write-budget inventory | PASS — exactly `CHANGELOG.md` (three WP1 Unreleased entries) plus the three Metrics test doc comments |

Supporting preflight facts verified at the exact base:

- root `Cargo.toml` is `1.8.0` and the live `Makefile` `migration` target
  derives `MAJOR.MINOR` from `Cargo.toml` and increments the minor, so the
  repository's current next generated migration suffix is `v1.9.0`. This
  confirms the approved task premise on the refreshed base. No
  `Cargo.toml`/`Makefile` change belongs to this task and none was made;
- the resolved Diesel migration stack is `diesel 2.3.10`,
  `diesel_migrations 2.3.2`, `migrations_internals 2.3.0`, exactly the stack
  the controlling amendment reasoned about;
- versions `20260826`, `20260827` and `20260828` are absent from both
  `origin/develop` and `origin/master`, confirming the three migrations remain
  unreleased programme-branch work;
- workflow trigger/publication behaviour was re-inspected at the exact base and
  is unchanged from the behaviour authorized in `5475388112` section 7, so
  `HOLD - AUTOMATIC SIDE-EFFECT AUTHORIZATION REBINDING REQUIRED` was not
  triggered.

A durable ACTIVE execution claim was recorded on #868 as comment
[`5475483846`](https://github.com/thoth-pub/thoth/issues/868#issuecomment-5475483846)
after successful preflight and before any substantive repository mutation.

## 2. Scope confirmation

Implemented exactly the bounded, already-approved reconciliation:

1. renamed exactly three unreleased Metrics migration directories from the
   `_v1.8.0` release/display suffix to `_v1.9.0`, preserving date prefixes,
   SQL bytes and file modes;
2. updated only the current forward-looking migration-path references in
   `CHANGELOG.md` and the three Metrics test doc comments;
3. added exactly one new evidence file (this report);
4. added exactly one bounded `## [Unreleased]` / `### Changed` entry for this
   task, as required by `AGENTS.md` section 13 ("Every PR must update
   `CHANGELOG.md` under `## [Unreleased]`") and expressly permitted by the
   specification. It does not duplicate or reinterpret the three WP1 schema
   descriptions.

No architecture decision was made. No scope was widened. No HOLD/BLOCKED/STOP
condition was encountered.

## 3. Commits

Bounded task commits on `feature/metrics--v1.9-migration-reconcile`, containing
only this task:

| # | Commit | Subject |
|---|---|---|
| 1 | `2327c020ddbe132133e0422d9fdd329072d7c9c0` | `MET-MIG-V1.9-RECON-01: reconcile unreleased Metrics migration suffix to v1.9.0` |
| 2 | this commit | `MET-MIG-V1.9-RECON-01: add bounded implementation report` |

Commit 1 carries the complete authorized source change: the six path-only SQL
moves and the four authorized content updates. Commit 2 adds only this evidence
file. A further evidence-only commit records the PR identity and observed CI
state (section 11); the exact resulting task head is reported in the #868
implementation ledger update.

Shared programme history was not rewritten. No force push, force-with-lease,
rebase or reset of shared history was performed.

## 4. Files changed

Complete changed-path inventory versus the authorized base
(`git diff --name-status --find-renames`):

```text
M       CHANGELOG.md
R100    thoth-api/migrations/20260826_v1.8.0/down.sql -> thoth-api/migrations/20260826_v1.9.0/down.sql
R100    thoth-api/migrations/20260826_v1.8.0/up.sql   -> thoth-api/migrations/20260826_v1.9.0/up.sql
R100    thoth-api/migrations/20260827_v1.8.0/down.sql -> thoth-api/migrations/20260827_v1.9.0/down.sql
R100    thoth-api/migrations/20260827_v1.8.0/up.sql   -> thoth-api/migrations/20260827_v1.9.0/up.sql
R100    thoth-api/migrations/20260828_v1.8.0/down.sql -> thoth-api/migrations/20260828_v1.9.0/down.sql
R100    thoth-api/migrations/20260828_v1.8.0/up.sql   -> thoth-api/migrations/20260828_v1.9.0/up.sql
M       thoth-api/src/model/metric_import/tests.rs
M       thoth-api/src/model/metric_platform/tests.rs
M       thoth-api/src/model/metric_source/tests.rs
A       docs/engineering/ai-delivery/implementation-reports/MET-MIG-V1.9-RECON-01-implementation-report.md
```

Every migration change is reported by Git as `R100` — a 100%-similarity rename
with zero content change.

### 4.1 Write-budget compliance

| Authorized item | Used | Notes |
|---|---|---|
| `CHANGELOG.md` | YES | three forward-looking path references + one bounded `### Changed` entry |
| `thoth-api/src/model/metric_platform/tests.rs` | YES | one doc-comment line |
| `thoth-api/src/model/metric_source/tests.rs` | YES | one doc-comment line |
| `thoth-api/src/model/metric_import/tests.rs` | YES | one doc-comment line |
| six SQL path-only moves | YES | blobs and modes preserved |
| one new implementation report | YES | this file |

Nothing outside the authorized write budget changed. Verified explicitly:

```text
thoth-api/src/schema.rs changed files: 0
Cargo.toml               changed files: 0
Cargo.lock               changed files: 0
Makefile                 changed files: 0
.github/                 changed files: 0
thoth-client/            changed files: 0
thoth-export-server/     changed files: 0
thoth-api-server/        changed files: 0
thoth-errors/            changed files: 0
src/                     changed files: 0
```

No historical WP1 implementation report was touched:

```text
git diff HEAD --name-only -- docs/engineering/ai-delivery/implementation-reports/
(no historical report listed)
```

The historical `MET-WP1-01`/`02`/`03` implementation reports deliberately retain
their original `_v1.8.0` wording (11, 9 and 6 occurrences respectively) as
immutable point-in-time provenance. Historical issues, PRs, reviews and
authorizations were likewise not rewritten.

No `HOLD - WRITE BUDGET AMENDMENT REQUIRED` was needed: no current
forward-looking reference outside the approved set required editing.

### 4.2 Authorized actions actually used

1. repository fetch/inspection;
2. durable #868 ACTIVE execution claim `5475483846` before substantive mutation;
3. isolated worktree; branch `feature/metrics--v1.9-migration-reconcile` created
   exactly from `5604a447c1744f4cd5cc715c15a43d156cc0e62b`;
4. the section-4 moves/content updates/new evidence file only;
5. local repository validation and disposable-PostgreSQL validation;
6. bounded task commit(s) containing only this task;
7. remote re-verification immediately before push;
8. ordinary non-force push of the task branch only;
9. creation of exactly one PR into `feature/metrics`;
10. issue/PR evidence recording and release of the ACTIVE claim.

Not used and not performed: merge; task-branch deletion; force push /
force-with-lease / shared-history rebase or reset; direct `feature/metrics`
mutation; deployment, release, tag or production activation; staging/production
or other persistent shared database migration; `__diesel_schema_migrations`
edits; manual CI dispatch/rerun/cancel; `feature/metrics -> develop`
integration; mutation of another repository; another Metrics slice; any
schema/API/data/auth/security/dependency change; self-approval.

### 4.3 Automatic and manual external effects

Automatic PR side effects explicitly authorized by `5475388112` section 7:

- `build-test-and-check`;
- `run-migrations`;
- `check-changelog`;
- `publish-to-dockerhub`, including its automatic build and push of the PR
  staging image to `ghcr.io/thoth-pub/thoth` under the workflow's
  `staging-pr-*` tag (`type=ref,event=pr,prefix=staging-pr-`).

Observed workflow runs are recorded in section 11.

No manual Actions dispatch, rerun or cancellation was performed. No provider or
runtime read/write occurred outside those automatic PR workflow side effects.
No staging, production or other persistent shared database was accessed.

## 5. Implementation decisions

No architecture decision was made; the design was fixed by the approved
specification. Bounded execution choices:

- **`git mv` for the renames.** Guarantees Git records true renames and makes
  blob preservation directly provable. All six moves are reported `R100`.
- **Targeted, anchored reference substitution.** Only the exact pattern
  `2026082[678]_v1.8.0` was rewritten to `2026082[678]_v1.9.0`. Occurrence
  counts before/after were asserted per file (CHANGELOG 3 → 0 remaining
  `_v1.8.0`, 3 new `_v1.9.0`; each test file 1 → 0 / 1). The executable
  `MET_WP1_0*_MIGRATION_VERSION` constants were untouched — the diff contains
  zero changed constant lines.
- **One `### Changed` CHANGELOG entry.** `AGENTS.md` section 13 requires every
  PR to update `CHANGELOG.md` under `## [Unreleased]` with an appropriate
  heading and no duplicate headings. `## [Unreleased]` previously had only
  `### Added` and `### Fixed`; a single new `### Changed` section was inserted
  between them, following Keep a Changelog ordering. The entry describes only
  this reconciliation.
- **Rebuilt embedded migration source for the post-rename proof.**
  `embed_migrations!` is expanded at compile time and Cargo does not track
  migration-directory additions/removals on stable, so `thoth-api/src/db.rs`
  was touched to force re-expansion before rebuilding. The pre- and post-rename
  binaries are provably distinct and embed different directory names
  (section 6.3).

## 6. Database and migration effects

### 6.1 Rename and blob proof

Diesel derives a migration **version** from the text before the first
underscore, so the `_v1.8.0` / `_v1.9.0` portion is the directory
release/display name and is **not** the ledger version.

| Migration | Before | After | Diesel version |
|---|---|---|---|
| MET-WP1-01 | `thoth-api/migrations/20260826_v1.8.0/` | `thoth-api/migrations/20260826_v1.9.0/` | `20260826` (unchanged) |
| MET-WP1-02 | `thoth-api/migrations/20260827_v1.8.0/` | `thoth-api/migrations/20260827_v1.9.0/` | `20260827` (unchanged) |
| MET-WP1-03 | `thoth-api/migrations/20260828_v1.8.0/` | `thoth-api/migrations/20260828_v1.9.0/` | `20260828` (unchanged) |

Before/after blob proof — every post-move blob equals the authorized pre-rename
anchor from `5475285197` / `5475388112`, and every file mode is preserved:

| Path (post-move) | Blob | Anchor match | Mode |
|---|---|---|---|
| `thoth-api/migrations/20260826_v1.9.0/up.sql` | `c8efef3c9c944c950fca546ab13459025d3f1943` | YES | `100644` |
| `thoth-api/migrations/20260826_v1.9.0/down.sql` | `2f8fa64811f0c406e0f530dc7186372b910d80c5` | YES | `100644` |
| `thoth-api/migrations/20260827_v1.9.0/up.sql` | `ca60e1cf78ea8e8c09eba5f12ec9e58ec9d37aec` | YES | `100644` |
| `thoth-api/migrations/20260827_v1.9.0/down.sql` | `8c90d7b781377c440df4042152d0a7a93a015b12` | YES | `100644` |
| `thoth-api/migrations/20260828_v1.9.0/up.sql` | `61d1dbfdffa3c8b67156eadce445f06a689c2cb7` | YES | `100644` |
| `thoth-api/migrations/20260828_v1.9.0/down.sql` | `0e7a0f6d220080f59547842c3e859056f68f9f10` | YES | `100644` |

Anchor failures: **0**.

Additional path/ordering proof:

- all six old `_v1.8.0` SQL paths are **absent**;
- all six new `_v1.9.0` SQL paths are **present**;
- exactly three directories moved; the migration directory count is unchanged
  at 12 and **no fourth migration was created**;
- zero `_v1.8.0` Metrics migration directories remain;
- raw content diff lines inside `thoth-api/migrations/` (`+`/`-`): **0**;
- the sorted Diesel version set is byte-identical before and after:
  `20250000 20260417 20260429 20260504 20260805 20260811 20260812 20260813
  20260814 20260826 20260827 20260828`, so migration ordering is unchanged.

### 6.2 Full migration chain — disposable PostgreSQL

Disposable local PostgreSQL 17.10 only. No staging, production or other
persistent shared database was touched, and `__diesel_schema_migrations` was
never edited by hand.

Pre-rename source, disposable database `met_v19_chain_proof`:

| Step | Command | Result |
|---|---|---|
| apply | `thoth migrate --database-url postgres://…/met_v19_chain_proof` | exit `0`; 12 ledger rows including `20260826`, `20260827`, `20260828` |
| revert | `thoth migrate --database-url … --revert` | exit `0`; 0 ledger rows; 0 `metric_*` tables remaining |
| reapply | `thoth migrate --database-url …` | exit `0`; the same 12 ledger rows |

Canonical schema dumps (`pg_dump --schema-only --no-owner --no-privileges`,
with comments, blank lines and pg_dump's random `\restrict`/`\unrestrict`
nonce lines stripped) are identical across apply and revert+reapply:
`6d8e63e2b1ea4a815dc2ffde62cb402cc0338b5a8e5afd5f77588ce3d19f36d2`.

Post-rename source, same chain on a freshly recreated empty
`met_v19_chain_proof`:

| Step | Result |
|---|---|
| apply | exit `0`; versions `20250000,20260417,20260429,20260504,20260805,20260811,20260812,20260813,20260814,20260826,20260827,20260828` |
| revert | exit `0`; 0 ledger rows |
| reapply | exit `0`; the same 12 versions |

Post-rename canonical schema hash is
`6d8e63e2b1ea4a815dc2ffde62cb402cc0338b5a8e5afd5f77588ce3d19f36d2` — **identical
to the pre-rename chain**. The final schema and seed result are therefore
equivalent to the pre-rename migration chain; the two approved `metric_measure`
seed rows (`net_units`, `title_sessions`) are present and unchanged.

### 6.3 Same-database ledger-neutrality proof

This mandatory proof used **one** disposable database, `met_v19_ledger_proof`,
across both phases.

**Phase A — pre-rename source.** The `thoth` binary was built from the
pre-rename worktree at the exact authorized base. It provably embeds the
pre-rename directory names:

```text
20260826_v1.8.0
20260827_v1.8.0
20260828_v1.8.0
```

binary SHA-256 `c7ef82b4c534117c69d5b0a76b5e5f879d2780edd97abed04688020ffc185969`.

The repository migration harness (`thoth migrate`, i.e.
`MigrationHarness::run_pending_migrations` over `embed_migrations!`) applied the
full chain: exit `0`. Recorded ledger state (`version|run_on`):

```text
20250000|2026-08-31 08:05:03.736305
20260417|2026-08-31 08:05:03.885829
20260429|2026-08-31 08:05:03.887085
20260504|2026-08-31 08:05:03.888408
20260805|2026-08-31 08:05:03.889827
20260811|2026-08-31 08:05:03.890626
20260812|2026-08-31 08:05:03.892875
20260813|2026-08-31 08:05:03.894627
20260814|2026-08-31 08:05:03.896945
20260826|2026-08-31 08:05:03.904005
20260827|2026-08-31 08:05:03.909308
20260828|2026-08-31 08:05:03.914253
```

Phase A ledger fingerprint (SHA-256 of the exact table above):
`2d71805bbbaa17fe467964e9aedebc63d61280545ceefae797ba6e2e54368ae8`.

**That database was kept.** It was not dropped, recreated or edited.

**Phase B — post-rename source, same database.** The three directories were
renamed, `thoth-api/src/db.rs` was touched to force `embed_migrations!`
re-expansion, and the binary was rebuilt. No stale pre-rename binary was reused
as evidence — the rebuilt binary is provably different:

- post-rename binary SHA-256
  `3ce9a7b61b36c3c84593ae93d7e57dbf9bc60837249dab54159171f98f1c4771`
  (≠ the Phase A binary);
- embedded Metrics directory names are now exactly `20260826_v1.9.0`,
  `20260827_v1.9.0`, `20260828_v1.9.0`;
- **zero** `2026082[678]_v1.8.0` strings remain embedded in the new binary.

Running the post-rename harness against the **same** `met_v19_ledger_proof`
returned exit `0`, and the ledger is byte-identical to Phase A:

```text
diff ledger-phaseA.txt ledger-phaseB.txt   ->  no differences
SHA-256 phase A = 2d71805bbbaa17fe467964e9aedebc63d61280545ceefae797ba6e2e54368ae8
SHA-256 phase B = 2d71805bbbaa17fe467964e9aedebc63d61280545ceefae797ba6e2e54368ae8
```

No row was added, no `run_on` timestamp changed, and the canonical schema hash
of that database is unchanged at
`6d8e63e2b1ea4a815dc2ffde62cb402cc0338b5a8e5afd5f77588ce3d19f36d2`.

**Conclusion.** Versions `20260826`, `20260827` and `20260828` were neither
reported pending nor replayed solely because their directory suffix changed
from `_v1.8.0` to `_v1.9.0`. The suffix-only rename is Diesel-ledger-neutral,
exactly as controlling amendment `5461405206` section A1 states.

### 6.4 Persistent-environment safety

No authoritative evidence was found that any persistent shared environment has
applied these unreleased Metrics migrations: versions `20260826`, `20260827` and
`20260828` are absent from both `origin/develop` and `origin/master`, and no WP1
child task authorized their execution against a persistent database.
`HOLD - UNAUTHORIZED PERSISTENT METRICS MIGRATION EXECUTION REQUIRES CONTROL
RECONCILIATION` was therefore not triggered. Only disposable local databases
were used, which by amendment `5461405206` section A4 do not trigger that HOLD.

## 7. API and compatibility effects

- public GraphQL/export behaviour: **unchanged** (no GraphQL/API source touched);
- generated client change required: **NO** (`thoth-client/` unchanged);
- downstream repository change required: **NO**;
- `thoth-api/src/schema.rs` contract: **unchanged**;
- Rust domain types: **unchanged**;
- dependency/Cargo/workspace version/`Makefile`: **unchanged**.

The only Rust source changes are three doc-comment lines.

## 8. Authorization and security

- no auth, authorization-role, credential, secret or security behaviour changed;
- no data, backfill or canonical-record change;
- no runtime behaviour change;
- no provider/configuration change;
- no secret was read, printed or introduced;
- no persistent migration ledger row was created, edited or deleted by hand.

## 9. Tests and checks

Local toolchain: `rustc 1.97.0`, `cargo 1.97.0`, `clippy 0.1.97`,
`rustfmt 1.9.0-stable`, PostgreSQL 17.10, Redis on `:6379`.

### Formatting

```bash
cargo fmt --all -- --check
```

Exit `0`, no output.

### Static analysis / lint

```bash
cargo clippy --all --all-targets --all-features -- -D warnings
```

Exit `0`, no warnings.

### Build check

```bash
cargo check --workspace
```

Exit `0`.

### Unit / integration / database tests

```bash
cargo test --workspace
```

Exit `0` — **1557 passed, 0 failed, 8 ignored** across all workspace targets.

```bash
cargo test -p thoth-api --features backend
```

Exit `0` — 1343 passed, 13 passed, 0 failed (8 ignored in the integration
target).

Targeted Metrics migration behaviour, all passing after the rename:

```text
model::metric_platform::tests::reverting_through_the_registry_migration_removes_it_and_reapplication_restores_it ... ok
model::metric_source::tests::reverting_through_the_source_state_migration_removes_it_and_leaves_the_registry_intact ... ok
model::metric_import::tests::reverting_through_the_import_state_migration_removes_it_and_reapplication_restores_it ... ok
model::metric_platform::tests::migration_seeds_no_platform_row ... ok
model::metric_source::tests::migration_seeds_no_source_row ... ok
model::metric_import::tests::migration_seeds_no_import_row ... ok
model::distribution_job::tests::the_migration_directory_keeps_its_exact_historical_name ... ok
```

The last one confirms the BE-04 historical-name invariant (`20260814_v1.7.0`)
is unaffected by this reconciliation.

### Other required checks

```bash
git diff --check
```

Exit `0` for unstaged, staged and `git diff HEAD` variants — no whitespace or
conflict-marker errors.

### Acceptance checklist

| Requirement | Result |
|---|---|
| exact implementation base is the authorized SHA | PASS |
| exactly three migration directories moved | PASS |
| no fourth migration created | PASS |
| all six old SQL paths absent | PASS |
| all six `_v1.9.0` SQL paths present | PASS |
| all six post-move blobs equal the exact anchors | PASS |
| versions remain `20260826`, `20260827`, `20260828` | PASS |
| no SQL content diff | PASS (0 content lines) |
| migration ordering unchanged | PASS |
| three CHANGELOG paths identify `_v1.9.0` | PASS |
| three Metrics test doc comments identify `_v1.9.0` | PASS |
| executable migration-version constants unchanged / date-only | PASS (0 changed constant lines) |
| historical evidence unchanged | PASS |
| nothing outside the authorized write budget changed | PASS |
| `git diff --check` passes | PASS |
| full chain apply/revert/reapply on disposable PostgreSQL | PASS |
| same-database ledger-neutrality proof | PASS |

No command that actually ran reported a code, test, lint or migration failure,
so `HOLD - NO PUSH` was not triggered.

## 10. Manual verification

Manually confirmed by direct inspection of the working tree and Git index:

- the three renamed directories each contain exactly `up.sql` and `down.sql`;
- the index records mode `100644` for all six files;
- `## [Unreleased]` contains `### Added`, `### Changed`, `### Fixed` with no
  duplicate headings;
- the three `MET_WP1_0*_MIGRATION_VERSION` constants still read `"20260826"`,
  `"20260827"`, `"20260828"`.

## 11. CI

### 11.1 PR identity and state

| Field | Value |
|---|---|
| PR | [#871](https://github.com/thoth-pub/thoth/pull/871) |
| State | OPEN (not draft), `MERGEABLE`, **not merged** |
| Head | `feature/metrics--v1.9-migration-reconcile` |
| Head commit at CI | `6691190a995110e7adeb63317848b9587e4a0de7` |
| Base | `feature/metrics` |
| Files changed | 11 (six SQL renames at `+0/-0`, four content files, one new report) |

### 11.2 Automatic workflow runs — all green at the exact head

Every run below is a permitted automatic consequence of the authorized PR
creation, per `5475388112` section 7. All four executed against head
`6691190a995110e7adeb63317848b9587e4a0de7`.

| Workflow | Run ID | Conclusion |
|---|---|---|
| `build-test-and-check` | [`33372926680`](https://github.com/thoth-pub/thoth/actions/runs/33372926680) | **success** |
| `run-migrations` | [`33372926670`](https://github.com/thoth-pub/thoth/actions/runs/33372926670) | **success** |
| `check-changelog` | [`33372926615`](https://github.com/thoth-pub/thoth/actions/runs/33372926615) | **success** |
| `publish-to-dockerhub` | [`33372926642`](https://github.com/thoth-pub/thoth/actions/runs/33372926642) | **success** |

All ten PR checks pass, none failed or was skipped:

```text
build                                pass
build_and_push_staging_docker_image  pass
check-changelog                      pass
classify                             pass   (x3, one per classifying workflow)
format_check                         pass
lint                                 pass
run_migrations                       pass
test                                 pass
```

`build-test-and-check` job conclusions: `classify` success, `build` success,
`test` success, `lint` success, `format_check` success. The CI `lint` job is the
authoritative Clippy result for this head and resolves the local toolchain skew
noted in section 13.

`run-migrations` independently reproduced the migration-chain proof on a
disposable `postgres:17` service container, with every step succeeding:

```text
Build binary:       success
Run migrations:     success
Revert migrations:  success
Reapply migrations: success
```

### 11.3 Authorized staging-image side effect

`publish-to-dockerhub` performed the explicitly authorized automatic external
registry write:

- image: `ghcr.io/thoth-pub/thoth:staging-pr-871`
  (from the workflow's `type=ref,event=pr,prefix=staging-pr-` tag rule);
- published manifest digest:
  `sha256:c7b80a18ca5196f5b55bdfd6f9067fc2e24ca0f7bbd4720e9e5efe43bb68b6c8`;
- `org.opencontainers.image.revision` label
  `fda03fcffa04a86a1d21eb80fd3742fb48fbb4e4` — the ephemeral
  `pull_request` merge ref GitHub generates for the event, not the task head;
  the task head remains `6691190a995110e7adeb63317848b9587e4a0de7`.

This is the only external publication produced by this task. No release image,
tag or GitHub Release was created.

### 11.4 Manual CI actions

**None.** No workflow was manually dispatched, rerun or cancelled.

## 12. Rollout and rollback

Repository-only change on an unreleased programme branch. No rollout, no
deployment, no release, no tag, no production activation.

Before merge, rollback is simply abandonment or ordinary revert of this bounded
source change. After merge, reverting the `_v1.9.0` suffix back to `_v1.8.0`
while preserving date prefixes and SQL bytes would remain ledger-neutral, but
should not be done casually because it would reverse the repository/release-number
coordination decision. An already-established date-prefix migration version or
SQL content must never be changed as a substitute for a forward
migration/repair once persistent environments depend on that schema.

## 13. Known limitations and deferred work

- **Local lint toolchain skew — resolved.** Clippy ran locally at `0.1.97`
  while the GitHub-hosted runner ships Clippy 1.98. The authoritative lint
  result for this head is the PR `build-test-and-check` run
  [`33372926680`](https://github.com/thoth-pub/thoth/actions/runs/33372926680),
  whose `lint` job **passed**. This change touches only three doc-comment lines
  and adds no executable code, so no lint-surface change arose.
- **Disposable databases only.** All migration evidence comes from disposable
  local PostgreSQL. No staging, production or other persistent shared database
  was accessed, by design.
- **Ledger-neutrality scope.** The proof establishes neutrality for a
  suffix-only rename with unchanged date prefixes and byte-identical SQL. It is
  not evidence for any rename that changes a date prefix or SQL content — both
  remain STOP conditions.
- **Release-forward reference cleanup is out of scope.** Historical WP1
  reports and prior issue/PR/review evidence intentionally keep `_v1.8.0`
  wording as provenance.

## 14. Unresolved issues

None encountered. No HOLD, BLOCKED or STOP condition was triggered.

## 15. Agent self-assessment

The bounded implementation matches the approved effective specification
exactly: three path-only migration directory renames with byte-identical SQL
and preserved modes, four authorized content files, one new evidence file, and
no change outside the authorized write budget. All required acceptance
evidence, repository-standard validation, full migration-chain validation and
the mandatory same-database ledger-neutrality proof were produced and passed.

This report is evidence, not approval. The implementing agent did not and may
not approve, review or merge its own work.

Resulting gate:

```text
MET-MIG-V1.9-RECON-01 IMPLEMENTATION COMPLETE AT EXACT TASK HEAD - PR CI / FRESH INDEPENDENT EXACT-HEAD SOURCE REVIEW REQUIRED
```

Refs #766
Refs #868
