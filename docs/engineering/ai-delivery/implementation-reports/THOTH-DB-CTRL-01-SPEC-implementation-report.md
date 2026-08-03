# THOTH-DB-CTRL-01-SPEC Implementation Report

## 1. Repository state

Repository: `thoth-pub/thoth`
Programme: Shared Repository Controls
Task ID: `THOTH-DB-CTRL-01-SPEC`
Output implementation task: `THOTH-DB-CTRL-01`
Workflow: STANDARD
Risk: HIGH
Base branch: `develop`
Base commit: `35e4dc20864ae4896dccc2b20cbcdbe3fb733db8`
PR target: `develop`
Programme integration branch: None
Task branch: `feature/repository-controls/thoth-db-ctrl-01-spec`
Post-ready reviewed head: `a4db430cb3c80fc6c1fd4100821c643129e87d5f`
Post-ready review: `PRR_kwDODkn0bc8AAAABIODEmQ`
Numeric review ID: `4846568601`
Post-ready review result: `CHANGES REQUIRED`
Specification content head: `dabf30550a968f49e7e0a6d25984d0ef99e779ee`
Approval-state reviewed base: `35e4dc20864ae4896dccc2b20cbcdbe3fb733db8`
Approval-state reviewed head: `b652f28d222f6a6bb5d3aa34dd5595e52223c195`
Approval-state review result: `CHANGES REQUIRED`
Approval-state content head: `b74113c95cdf1e952f8c45d928cbf178f8b1e485`
Final specification status: `APPROVED`
Pull request: [#775](https://github.com/thoth-pub/thoth/pull/775), draft
Expected branch deletion after merge: YES
Final programme PR required: NO
Implementing model: Codex / GPT-5
Reasoning level: HIGH
Independent reviewer: separate cross-model reviewer, HIGH or MAXIMUM reasoning

### 1.1 Required task identity

```text
Programme: Shared Repository Controls
Repository: thoth-pub/thoth
Task ID: THOTH-DB-CTRL-01-SPEC approval-state correction
Approved specification: Javi, CTO approval of THOTH-DB-CTRL-01 written specification at b652f28d222f6a6bb5d3aa34dd5595e52223c195 on 2026-08-03
Risk: HIGH
Base branch and commit: develop at 35e4dc20864ae4896dccc2b20cbcdbe3fb733db8
PR target: develop
Task branch: feature/repository-controls/thoth-db-ctrl-01-spec
Dependencies: PR #774 merged as 35e4dc20864ae4896dccc2b20cbcdbe3fb733db8; approval-state review of PR #775 head b652f28d222f6a6bb5d3aa34dd5595e52223c195
Implementing agent/model: Codex / GPT-5, HIGH reasoning
Independent reviewer/model: separate cross-model reviewer, HIGH or MAXIMUM reasoning
```

The independent review of exact head
`3f0affd0e375975dd18ea895219ab77477b41325` did not state the reviewer's exact
model. This report does not infer one. The review returned `CHANGES REQUIRED`
with one P1 safe-target finding, which the third bounded documentation commit
addresses.

The independent review of exact head
`9247cc5e4dbc82a5f4ecc381f8b8b5084c9bc628` also did not state the reviewer's
exact model. It confirmed the safe-target correction and returned
`CHANGES REQUIRED` with one new P1: the future schema-only publisher-column
probe could not satisfy the required focused compile check. The fourth bounded
documentation commit replaces that future acceptance probe with a compile-valid
standalone table and explicit five-entry manifest.

The fresh post-ready Codex review
`PRR_kwDODkn0bc8AAAABIODEmQ` / `4846568601` reviewed exact head
`a4db430cb3c80fc6c1fd4100821c643129e87d5f` and returned
`CHANGES REQUIRED` with four P1 findings. The normative corrections are all in
exact specification content head
`dabf30550a968f49e7e0a6d25984d0ef99e779ee`. The following
report-finalization commit changes only this implementation report. It does not
alter the normative specification content or rerun the discovery experiments.

Previous independent approval comment `5123408720` and CTO authorization
comment `5169190785` are historical and do not authorize another ready
transition or merge after this remediation.

The approval-state review of exact base
`35e4dc20864ae4896dccc2b20cbcdbe3fb733db8` and exact head
`b652f28d222f6a6bb5d3aa34dd5595e52223c195` returned `CHANGES REQUIRED` with
one P1: repository-authoritative records still described the written
specification as a draft and unapproved. On 2026-08-03, Javi, CTO, explicitly
approved that exact written specification while explicitly withholding
implementation-branch and implementation-work authorization. Exact
approval-state content head
`b74113c95cdf1e952f8c45d928cbf178f8b1e485` records the resulting transition
from `DRAFT` to `APPROVED` without changing normative design content.

## 2. Scope confirmation

Approved specification: `THOTH-DB-CTRL-01` written specification approved by
Javi, CTO, on 2026-08-03 at reviewed head
`b652f28d222f6a6bb5d3aa34dd5595e52223c195`

Implemented objective: perform read-only repository discovery and isolated
disposable-database experiments, select one evidence-backed implementation
design, create the implementation-ready specification for
`THOTH-DB-CTRL-01`, remediate exact-head review findings, and record the explicit
written-specification approval in only the authorized documentation and control
records.

Out-of-scope changes made: NONE

This task is:

```text
THOTH-DB-CTRL-01-SPEC:
documentation, discovery, review remediation, and specification approval record
```

It is not:

```text
THOTH-DB-CTRL-01:
future separately reviewed implementation
```

No final Diesel control, migration, schema change, BE-01 object, Rust code,
GraphQL contract, workflow, Makefile target, or Diesel configuration was
implemented.

The first two review remediations changed only this report and the output task
specification. They corrected the safe-target contract and future acceptance
probe without rerunning or recharacterizing the completed discovery. The
post-ready specification-content correction changes only the output task
specification. The approval-state correction changes only the task
specification, CG-12 record, and repository map; this finalization changes only
this report. No implementation surface is changed.

## 3. Preconditions and branch evidence

Commands:

```bash
git status --short
git branch --show-current
git rev-parse HEAD
git fetch origin
git rev-parse origin/develop
git rev-parse develop
git log -1 --oneline origin/develop
git ls-remote --heads origin \
  feature/repository-controls/thoth-db-ctrl-01-spec \
  feature/repository-controls/thoth-db-ctrl-01
git branch --list \
  feature/repository-controls/thoth-db-ctrl-01-spec \
  feature/repository-controls/thoth-db-ctrl-01
```

Result:

```text
worktree: clean
current branch: develop
HEAD: 35e4dc20864ae4896dccc2b20cbcdbe3fb733db8
local develop: 35e4dc20864ae4896dccc2b20cbcdbe3fb733db8
origin/develop: 35e4dc20864ae4896dccc2b20cbcdbe3fb733db8
live origin develop: 35e4dc20864ae4896dccc2b20cbcdbe3fb733db8
origin/develop summary: 35e4dc20 docs: specify BE-01 publisher package model (#774)
specification branch before creation: absent locally and remotely
implementation branch: absent locally and remotely
```

Branch command:

```bash
git switch -c feature/repository-controls/thoth-db-ctrl-01-spec \
  35e4dc20864ae4896dccc2b20cbcdbe3fb733db8
```

Result: exit 0; only the specification branch was created.

## 4. Authoritative sources inspected

The following current repository sources were read:

```text
AGENTS.md
docs/engineering/AGENTS.md
thoth-api/AGENTS.md
docs/engineering/ai-delivery/operating-model.md
docs/engineering/ai-delivery/risk-classification.md
docs/engineering/ai-delivery/release-gates.md
docs/engineering/ai-delivery/branching-and-release-workflow.md
docs/engineering/ai-delivery/task-specification-template.md
docs/engineering/ai-delivery/implementation-report-template.md
docs/engineering/repository-map/control-gaps.md
docs/engineering/repository-map/repositories/thoth.md
docs/engineering/ai-delivery/tasks/BE-01.md
docs/publisher-services/README.md
docs/publisher-services/task-status.md
README.md
Cargo.toml
Cargo.lock
Makefile
diesel.toml
docker-compose.yml
.env.example
thoth-api/Cargo.toml
thoth-api/src/schema.rs
thoth-api/src/db.rs
src/bin/arguments/mod.rs
thoth-api/migrations/
.github/scripts/classify_ci_changes.py
.github/workflows/build_test_and_check.yml
.github/workflows/run_migrations.yml
.github/workflows/check_changelog.yml
CHANGELOG.md
```

Requested paths `docker-compose.yaml` and
`thoth-api/src/bin/arguments/mod.rs` are absent. The current migration argument
source is root `src/bin/arguments/mod.rs`.

History and search commands:

```bash
git log --all --oneline -- diesel.toml thoth-api/src/schema.rs Makefile
git log -p -- diesel.toml
git log -p -- Makefile
git log --stat -- thoth-api/src/schema.rs
git blame diesel.toml
git blame Makefile
rg -n \
  'diesel|print-schema|print_schema|schema.rs|make migration|cargo run migrate|revert_all_migrations' \
  . \
  --glob '!target/**' \
  --glob '!.git/**'
```

Concise result:

- `make migration` is the only repository migration-creation target;
- it creates the dated next-minor directory beneath `thoth-api/migrations/`;
- the root application binary embeds `thoth-api/migrations` and owns forward
  and full-chain revert execution;
- `diesel.toml` began with import-style schema configuration and was converted
  to `custom_type_derives` in 2023 without separating imports from derives;
- four missing commas accumulated in the current array;
- schema history contains long-lived manual compatibility conventions rather
  than an established reproducible generation command.

## 5. Commits

- `bfee1ca8356ac191521e112f835bf3a4af0993d3` -
  `docs: specify THOTH-DB-CTRL-01 Diesel control`
- `3f0affd0e375975dd18ea895219ab77477b41325` -
  `docs: report THOTH-DB-CTRL-01 specification`
- `9247cc5e4dbc82a5f4ecc381f8b8b5084c9bc628` -
  `docs: correct THOTH-DB-CTRL-01 target safety`
- `a4db430cb3c80fc6c1fd4100821c643129e87d5f` -
  `docs: make THOTH-DB-CTRL-01 probe compile-valid`
- `dabf30550a968f49e7e0a6d25984d0ef99e779ee` -
  `docs: harden THOTH-DB-CTRL-01 generation contract`
- `b652f28d222f6a6bb5d3aa34dd5595e52223c195` -
  `docs: report PR 775 post-ready remediation`
- `b74113c95cdf1e952f8c45d928cbf178f8b1e485` -
  `docs: approve THOTH-DB-CTRL-01 specification`
- the following report-finalization commit, which changes only this file -
  `docs: report THOTH-DB-CTRL-01 specification approval`

These are the eight ordered commit positions. A Git commit cannot embed its own
SHA in a file contained by that commit; the exact immutable approval-state
content head is therefore recorded above, while the final report-only head is
recorded in the PR body and superseding immutable evidence after creation. No
commit was amended, squashed, rebased, or force-pushed.

## 6. Files changed

- `CHANGELOG.md`
  - reason: record specification PR #775 under the existing Unreleased/Changed
    section;
  - behavioural effect: none.
- `docs/engineering/ai-delivery/tasks/THOTH-DB-CTRL-01.md`
  - reason: define the implementation-ready Diesel control; correct its
    Docker/GitHub Actions safe-target contract and controlled compile probe;
    and address the post-ready automatic-output, manifest, tracked-tooling, and
    authoritative-AGENTS findings; then record CTO approval of the written
    specification;
  - behavioural effect: changes the written specification from `DRAFT` to
    `APPROVED`; it does not authorize implementation.
- `docs/engineering/repository-map/control-gaps.md`
  - reason: mark the CG-12 specification approved while retaining implementation
    not started and CG-12 unresolved;
  - behavioural effect: BE-01 remains blocked.
- `docs/engineering/repository-map/repositories/thoth.md`
  - reason: record the selected future canonical procedure, written
    specification approval, separate implementation authorization boundary,
    and continuing regeneration prohibition;
  - behavioural effect: none.
- `docs/engineering/ai-delivery/implementation-reports/THOTH-DB-CTRL-01-SPEC-implementation-report.md`
  - reason: preserve exact discovery, test, decision, review-remediation,
    compile-validation, post-ready review, immutable specification-content
    head, approval-state review, immutable approval-state content head, and
    handoff evidence;
  - behavioural effect: none.

## 7. Tooling discovery

Commands:

```bash
rustc --version
cargo --version
docker --version
docker compose version
diesel --version
rg -n '^diesel|diesel =' Cargo.toml thoth-api/Cargo.toml Cargo.lock
```

Results:

```text
rustc 1.97.0 (2d8144b78 2026-07-07)
cargo 1.97.0 (c980f4866 2026-06-30)
Docker client/server 29.6.2
Docker Compose v5.3.1
installed Diesel CLI 2.2.4 with postgres, mysql, and sqlite
workspace diesel 2.3.10
Cargo.lock diesel 2.3.10
Cargo.lock diesel_derives 2.3.9
Cargo.lock diesel_migrations 2.3.2
```

The installed CLI did not match the repository. The isolated installation
command was:

```bash
cargo install diesel_cli \
  --version '2.3.10' \
  --root /private/tmp/thoth-db-ctrl-diesel-cli-2.3.10 \
  --no-default-features \
  --features postgres \
  --locked
```

Result: exit 0; `/private/tmp/thoth-db-ctrl-diesel-cli-2.3.10/bin/diesel
--version` reported `diesel 2.3.10`; no repository file, global binary, or user
toolchain configuration changed. The temporary installation was deleted after
evidence capture.

Help commands:

```bash
diesel --help
diesel print-schema --help
diesel migration run --help
```

Result: exit 0. `print-schema` accepts an explicit `--config-file` and emits
schema text to stdout. Migration commands update the configured
`print_schema.file`; `--locked-schema` can reject a resulting change.

## 8. Disposable database proof

### 8.1 Target

```text
client endpoint host: 127.0.0.1
client endpoint classification: loopback
client endpoint port: 55432
database: thoth_ctrl_01_spec
user: thoth_ctrl
connection source: explicit task-local environment variable
container: thoth_db_ctrl_01_spec_20260729
image: postgres:17
server: PostgreSQL 17.4 on Debian
host/repository mounts: none
database storage: one Docker anonymous volume
production, staging, shared-development, and public network access: none
```

The disposable password and complete URL are intentionally not recorded. No
production credential or secret was used.

This discovery proved that the client-facing endpoint was loopback and that the
database was the task-created disposable Docker container. It did not query or
record `inet_server_addr()`, `inet_server_port()`, `inet_client_addr()`, or
`inet_client_port()`. It therefore does not establish that PostgreSQL's
server-side accepted address was loopback. Under Docker port publication, that
address may instead be the container's private bridge address. The corrected
implementation specification requires those values to be queried, recorded,
classified, and tied to the verified local Docker or GitHub Actions
provenance.

Creation command, with the task-local credential variable redacted from the
report:

```bash
docker run -d --rm \
  --name thoth_db_ctrl_01_spec_20260729 \
  -e POSTGRES_DB=thoth_ctrl_01_spec \
  -e POSTGRES_USER=thoth_ctrl \
  -e POSTGRES_PASSWORD="$THOTH_DB_CTRL_DISPOSABLE_PASSWORD" \
  -p 127.0.0.1:55432:5432 \
  postgres:17
```

Result: exit 0. `docker inspect` proved loopback-only port publication,
`AutoRemove=true`, and only an anonymous `/var/lib/postgresql/data` volume.
The initial `public` schema contained zero application tables.

A secondary database, `thoth_ctrl_path_test`, was created inside the same
unique container only to prove configuration-relative path resolution. It had
no external owner or persistence.

### 8.2 Cleanup

Commands:

```bash
docker stop thoth_db_ctrl_01_spec_20260729
docker ps -a \
  --filter name=thoth_db_ctrl_01_spec_20260729 \
  --format '{{.ID}} {{.Names}} {{.Status}}'
docker volume inspect \
  1305746e95f408b4f5c6edd39f4fc195dd8339197450fa1b2b1ea0032c40b195
git worktree remove --force \
  /private/tmp/thoth-db-ctrl-01-spec-worktree
rm -rf \
  /private/tmp/thoth-db-ctrl-01-spec-artifacts \
  /private/tmp/thoth-db-ctrl-diesel-cli-2.3.10
```

Results:

```text
container stop: exit 0
container lookup after stop: empty
anonymous volume lookup after stop: exit 1, no such volume
temporary worktree removal: exit 0
temporary artifact and isolated CLI removal: exit 0
```

The container, both databases, anonymous volume, detached worktree, generated
schemas, patch experiments, and temporary CLI were deleted.

## 9. Migration-chain investigation

Exact application commands, run from the repository root with the safe
task-local URL in `THOTH_DB_CTRL_DATABASE_URL`:

```bash
cargo run migrate \
  --database-url "$THOTH_DB_CTRL_DATABASE_URL"

cargo run migrate --revert \
  --database-url "$THOTH_DB_CTRL_DATABASE_URL"

cargo run migrate \
  --database-url "$THOTH_DB_CTRL_DATABASE_URL"
```

Results:

| Phase | Exit | Applied migrations | Application tables | PostgreSQL enums |
|---|---:|---:|---:|---:|
| Initial empty database | 0 | 0 | 0 | 0 |
| First full apply | 0 | 4 | 55 | 21 |
| Full-chain revert | 0 | 0 | 0 | 0 |
| Full reapply | 0 | 4 | 55 | 21 |

Applied migration versions after each apply:

```text
20250000
20260417
20260429
20260504
```

The first command compiled and ran in 1 minute 44 seconds. All three commands
exited 0. Cargo emitted a non-blocking future-incompatibility warning for
`proc-macro-error2`; no migration error occurred. Source inspection confirmed
that `--revert` calls `revert_all_migrations`, so it is safe only on the
disposable database used here.

## 10. Diesel configuration investigation

### 10.1 Parse result

Current configuration was tested first without writing a repository path:

```bash
/private/tmp/thoth-db-ctrl-diesel-cli-2.3.10/bin/diesel \
  print-schema \
  --database-url "$THOTH_DB_CTRL_DATABASE_URL" \
  --config-file /Users/ja573/thoth/diesel.toml
```

Result: non-zero.

```text
Failed to parse config file: TOML parse error at line 20, column 3
missing comma between array elements, expected ','
```

The first unreachable-target probe returned the connection error before
configuration parsing. The reachable disposable target was therefore necessary
to establish the authoritative parse failure.

### 10.2 Path resolution

A temporary directory contained a syntactically valid copied config whose
`file` remained `src/schema.rs`. The exact migration command was:

```bash
/private/tmp/thoth-db-ctrl-diesel-cli-2.3.10/bin/diesel \
  migration run \
  --database-url "$THOTH_DB_CTRL_PATH_TEST_DATABASE_URL" \
  --migration-dir /Users/ja573/thoth/thoth-api/migrations \
  --config-file "$THOTH_DB_CTRL_TEMP_CONFIG"
```

Result: exit 0; all four migrations applied and Diesel wrote
`src/schema.rs` beneath the temporary config directory. It did not write
relative to the invocation directory. This proves root
`file = "src/schema.rs"` cannot identify canonical
`thoth-api/src/schema.rs`.

### 10.3 Derive semantics

A temporary config repaired only the missing commas and retained all current
values. The equivalent schema command was:

```bash
/private/tmp/thoth-db-ctrl-diesel-cli-2.3.10/bin/diesel \
  print-schema \
  --database-url "$THOTH_DB_CTRL_DATABASE_URL" \
  --config-file "$THOTH_DB_CTRL_REPAIRED_CONFIG"
```

Result: exit 0 and 40,585 output bytes, but the generated derive lists included
`diesel::sql_types::*` and application model paths. These are imports or types,
not derive macros, so the output is semantically invalid.

The minimal valid temporary config used:

```toml
[print_schema]
file = "schema.rs"
custom_type_derives = ["diesel::query_builder::QueryId"]
```

Diesel supplied `diesel::sql_types::SqlType` automatically.

## 11. Schema-generation experiments

All outputs were outside the repository. No command wrote a committed path.

The two exact baseline commands differed only by output filename:

```bash
/private/tmp/thoth-db-ctrl-diesel-cli-2.3.10/bin/diesel \
  print-schema \
  --database-url "$THOTH_DB_CTRL_DATABASE_URL" \
  --config-file "$THOTH_DB_CTRL_MINIMAL_CONFIG" \
  > /private/tmp/thoth-db-ctrl-01-spec-artifacts/schema-2.3.10-first.rs

/private/tmp/thoth-db-ctrl-diesel-cli-2.3.10/bin/diesel \
  print-schema \
  --database-url "$THOTH_DB_CTRL_DATABASE_URL" \
  --config-file "$THOTH_DB_CTRL_MINIMAL_CONFIG" \
  > /private/tmp/thoth-db-ctrl-01-spec-artifacts/schema-2.3.10-second.rs
```

Comparison commands:

```bash
cmp \
  /private/tmp/thoth-db-ctrl-01-spec-artifacts/schema-2.3.10-first.rs \
  /private/tmp/thoth-db-ctrl-01-spec-artifacts/schema-2.3.10-second.rs

diff -u \
  thoth-api/src/schema.rs \
  /private/tmp/thoth-db-ctrl-01-spec-artifacts/schema-2.3.10-first.rs

git status --short
```

Results:

```text
first generated output: 28,132 bytes
second generated output: 28,132 bytes
repeat cmp: exit 0, identical
committed schema: 28,603 bytes
normalized committed/generated diff: 382 insertions, 455 deletions
worktree after experiments: clean
```

The installed CLI `2.2.4` generated 28,101 bytes from the same database. Its
output differed from `2.3.10` by two insertions and one deletion, proving the
output is CLI-version-sensitive and supporting an exact version pin.

### 11.1 Difference classification

| Class | Evidence and disposition |
|---|---|
| Expected deterministic output | Two exact CLI `2.3.10` runs were byte-identical. |
| Configuration/path error | Current config does not parse and `src/schema.rs` resolves beside the root config. |
| CLI-version difference | CLI `2.2.4` and `2.3.10` differed by three diff lines. |
| Formatting-only difference | Macro qualification, grouping, whitespace, and ordering account for a large part of the raw diff. |
| Custom-type derive difference | Current config applies non-derive paths; minimal valid output adds `SqlType` and configured `QueryId`. |
| Committed manual modification | `work_abstract`, `work_title`, physical `title` aliasing, model column order, `Timestamptz` mappings, and supplemental `MarkupFormat` are maintained conventions. |
| Unexplained | None remained after database/catalog, schema, model, history, and compile comparison. |

Both raw and canonical schemas described 54 tables. Structural comparison
identified two table-module aliases, physical `title.title` identifier
handling, a broad but enumerable set of legacy timestamp mappings, and the
supplemental `MarkupFormat` type as the semantic differences beyond table and
column ordering.

### 11.2 Patch and compile rejection tests

A full raw-to-committed patch was 1,322 lines and 39,903 bytes. Diesel applied
it to the clean baseline, but after this controlled disposable change:

```sql
ALTER TABLE publisher
ADD COLUMN thoth_db_ctrl_probe text;
```

generation failed:

```text
Failed to apply patch: error applying hunk #16
```

A reduced semantic patch was 427 lines and 11,665 bytes. It admitted the
controlled column as exactly:

```text
thoth_db_ctrl_probe -> Nullable<Text>
```

and removing the column restored its deterministic baseline. However, replacing
only the schema in a detached temporary worktree and running:

```bash
cargo check -p thoth-api --features backend
```

returned exit 101:

```text
error: could not compile thoth-api (lib) due to 86 previous errors
```

The errors included unresolved `crate::schema::sql_types::MarkupFormat`, an
untranslated `title` joinable after the module alias, and many Diesel
`CompatibleType` failures explaining that query fields no longer matched Rust
struct fields in count, order, and type. This rejects both raw replacement and
the reduced patch as safe canonical generation.

### 11.3 Compile-valid acceptance-probe validation

The exact-head review of
`9247cc5e4dbc82a5f4ecc381f8b8b5084c9bc628` correctly identified that the
historical publisher-column experiment could not also serve as a successful
schema-only compile probe. `Publisher` has nine positional Diesel `Queryable`
fields, while its list and `by_zitadel_ids` paths load the complete
`publisher` table row; adding a tenth schema column without a model change is
therefore intentionally compile-incompatible.

The corrected future acceptance probe is instead:

```sql
CREATE TABLE public.thoth_db_ctrl_probe (
    probe_id uuid PRIMARY KEY,
    probe_value text
);
```

Its corrected version-2 expected manifest explicitly declares the table, both
complete columns with PostgreSQL/Diesel types, nullability and ordinals, the
ordered primary-key columns, and `allow_tables_to_appear_in_same_query!`
membership. It creates no join and has no consuming application model.

Before this documentation correction, a detached temporary worktree at
`9247cc5e4dbc82a5f4ecc381f8b8b5084c9bc628` was given only the equivalent
standalone Diesel `table!` block and allow-table membership in its temporary
`thoth-api/src/schema.rs`. Command:

```bash
cargo check -q -p thoth-api --features backend
```

Result: exit 0 with no output. The temporary candidate was not committed, and
the detached worktree and its build artifacts were removed. This validates only
that the chosen schema-only acceptance candidate compiles without a model; the
future synchronizer must still prove manifest equality, deterministic
generation, table removal, and byte-identical baseline restoration.

### 11.4 Post-ready review findings and disposition

The fresh post-ready review of exact head
`a4db430cb3c80fc6c1fd4100821c643129e87d5f` raised exactly these four P1
threads. Each normative correction is contained by exact specification content
head `dabf30550a968f49e7e0a6d25984d0ef99e779ee`; remote replies and thread
resolution are gated on pushed exact-head CI and recorded in superseding PR
evidence.

| Thread | Finding | Disposition in the specification content head |
|---|---|---|
| `PRRT_kwDODkn0bc6WDgUJ` | Prevent Diesel commands from bypassing the synchronizer | `diesel.toml` automatic output is ignored staging at `target/diesel-schema.rs`; only validated `generate` may atomically write the canonical schema, with direct-command bypass tests. |
| `PRRT_kwDODkn0bc6WDgUQ` | Encode complete structure in expected-change entries | String tokens are replaced by version-2 structural objects with complete column, key, join, allow-table and enum semantics plus exact four-way delta equality and negative tests. |
| `PRRT_kwDODkn0bc6WDgUT` | Include the authoritative AGENTS files in control rollout | Root `AGENTS.md` and `thoth-api/AGENTS.md` are added to the bounded future implementation scope with explicit replacement and consistency criteria. |
| `PRRT_kwDODkn0bc6WDgUX` | Record the exact reviewed head in the report | This report records the post-ready reviewed head, review IDs, result, and exact immutable specification-content head without an impossible self-referential commit claim. |

The specification also moves future synchronizer and test code from ignored
`scripts/` to tracked `.github/scripts/`. It does not add `.gitignore` to the
specification PR or the future implementation scope.

### 11.5 Approval-state review and disposition

The independent approval-state review of exact base
`35e4dc20864ae4896dccc2b20cbcdbe3fb733db8` and exact head
`b652f28d222f6a6bb5d3aa34dd5595e52223c195` returned:

```text
P0: none
P1: 1
P2: none
Decision: CHANGES REQUIRED
```

The P1 found that the normative task, CG-12 record, and repository map still
described the specification as draft or unapproved, which would conflict with
the intended post-merge implementation workflow. Javi, CTO, then explicitly
approved the written specification at reviewed head
`b652f28d222f6a6bb5d3aa34dd5595e52223c195` on 2026-08-03, stating that the
approval does not authorize the implementation branch or implementation work.

Commit `b74113c95cdf1e952f8c45d928cbf178f8b1e485` records the immutable approval
state across exactly the task specification, CG-12 record, and repository map:

```text
Specification: APPROVED
THOTH-DB-CTRL-01 implementation: NOT STARTED
Implementation branch: NOT AUTHORIZED
CG-12: unresolved
CG-13: open
BE-01 implementation: blocked
```

Specification approval does not authorize creation of
`feature/repository-controls/thoth-db-ctrl-01`, implementation work, migration
execution, schema or workflow changes, production access, release, deployment,
or activation. A fresh implementation-branch authorization and a later,
separate implementation merge authorization remain mandatory.

## 12. Selected implementation approach

Selected: retain root `diesel.toml` and canonical
`thoth-api/src/schema.rs`; direct automatic Diesel output only to ignored,
untrusted `target/diesel-schema.rs`; correct the derive list; pin exact CLI
`2.3.10`; add explicit convention control data; and add a fail-closed structural
synchronizer under tracked `.github/scripts/` that:

1. proves the target is local and disposable;
2. requires a loopback client endpoint, inspects server/client connection
   addresses and ports, and ties any private server address to verified local
   Docker or GitHub Actions provenance;
3. captures raw Diesel output privately and ignores or deletes automatic
   staging output;
4. compares physical database structure with the canonical contract through
   enumerated aliases, type overrides, supplemental types, and order;
5. preserves unchanged canonical bytes;
6. permits only an exact task-local structured version-2 expected-change
   manifest with complete column, key, join, allow-table and enum semantics;
7. requires exact equality among catalog, raw Diesel, convention-adjusted
   canonical, and manifest deltas;
8. acts as the sole canonical writer and atomically writes only after every
   validation and compile check passes;
9. proves direct Diesel commands and staging-file changes cannot write or
   promote the canonical schema;
10. preserves canonical bytes on every failure;
11. compiles a candidate and rejects unrelated file changes;
12. uses a standalone, model-independent table for the controlled expected-diff
    and compile-success probe;
13. updates root `AGENTS.md` and `thoth-api/AGENTS.md` to the same implemented
    procedure while preserving CG-13 as separate;
14. runs identically in local and CI verification.

This approach follows the evidence: database introspection is deterministic,
while the compiled repository contract has intentional semantics that raw
Diesel cannot infer. Explicit structural control data makes those semantics
reviewable without relying on a context-fragile text patch or manual,
unbounded editing.

The expected implementation changes exactly these thirteen paths:

```text
AGENTS.md
thoth-api/AGENTS.md
diesel.toml
Makefile
.github/scripts/diesel_schema.py
.github/scripts/test_diesel_schema.py
thoth-api/diesel-schema-control.toml
.github/scripts/classify_ci_changes.py
.github/workflows/run_migrations.yml
CHANGELOG.md
docs/engineering/repository-map/control-gaps.md
docs/engineering/repository-map/repositories/thoth.md
docs/engineering/ai-delivery/implementation-reports/THOTH-DB-CTRL-01-implementation-report.md
```

Exact command contracts are specified in
`docs/engineering/ai-delivery/tasks/THOTH-DB-CTRL-01.md`. Root `.gitignore` and
`thoth-api/src/schema.rs` are not implementation-change paths. No architectural
decision remains for the separate implementation agent.

## 13. Rejected alternatives

1. **Direct raw regeneration:** rejected by the 837-line normalized diff and
   86-error compile failure.
2. **Full Diesel patch:** rejected because a one-column controlled addition
   made the patch fail before output.
3. **Reduced patch:** rejected because it admitted the new column but lost
   compiled contract ordering and supplemental types.
4. **Manual post-processing:** rejected because it cannot prove deterministic
   no-op or exact bounded change.
5. **Crate-local config or schema relocation:** rejected because ignored root
   `target/` staging safely separates automatic output from the canonical file,
   and relocation is prohibited.
6. **Diesel version change:** rejected because exact locked `2.3.10` is
   deterministic and version changes cannot infer model conventions.

## 14. Database and migration effects

Migration added: NO
Schema change required: NO
Backfill required: NO
Production data effect: NONE

Only the uniquely named disposable local databases were changed. They and their
storage were deleted. No existing local, shared, staging, or production
database was contacted.

## 15. API and compatibility effects

GraphQL/API changes: NONE
Generated schema/client updates: NONE
Backwards compatibility: unchanged
Deprecations: NONE
Cross-repository dependencies: NONE

The future control must preserve model-compatible schema order and supplemental
types; those requirements were added because the compile experiment showed
that treating raw output as formatting-only would break existing code.
Automatic output at `target/diesel-schema.rs` is explicitly untrusted staging;
only the validated synchronizer may promote a compiled, manifest-equal candidate
to `thoth-api/src/schema.rs`.

## 16. Authorization and security

Authorization paths changed: NONE
Roles/scopes involved: NONE
Negative authorization tests: not applicable to documentation/discovery
Secret or personal-data handling: no production credentials, URLs, table
contents, personal data, or unbounded database output were logged
Security limitations: database target safety is specified but not implemented
by this task; the completed discovery proved a loopback client endpoint and
task-created disposable container, but did not record the PostgreSQL
server/client connection address and port functions

## 17. Tests and checks

### Formatting

Command:

```bash
git diff --check \
  35e4dc20864ae4896dccc2b20cbcdbe3fb733db8...HEAD
```

Result: recorded after the report commit in the immutable PR evidence; expected
output is empty.

### Unit tests

Not applicable: no executable repository code changed.

### Integration/database tests

Commands and exact results are recorded in sections 8 through 11:

- PostgreSQL 17 empty apply: exit 0, four migrations, 55 tables, 21 enums;
- full-chain revert: exit 0, zero application migrations/tables/enums;
- full reapply: exit 0, four migrations, 55 tables, 21 enums;
- exact CLI `2.3.10` generation twice: both exit 0 and `cmp` exit 0;
- raw/reduced replacement compile: exit 101 with 86 errors;
- controlled full-patch generation: non-zero at patch hunk 16;
- standalone-table schema-only compile validation:
  `cargo check -q -p thoth-api --features backend`, exit 0 with no output.

### Lint/static analysis

Not applicable to the five documentation-only repository paths. Exact-head CI
classification and skipped step arrays are recorded in immutable PR evidence.

### Other required checks

Before push, run and record:

```bash
git diff --check \
  35e4dc20864ae4896dccc2b20cbcdbe3fb733db8...HEAD
git diff --name-only \
  35e4dc20864ae4896dccc2b20cbcdbe3fb733db8...HEAD
git diff --name-only \
  a4db430cb3c80fc6c1fd4100821c643129e87d5f...HEAD
git diff --name-only \
  b652f28d222f6a6bb5d3aa34dd5595e52223c195...HEAD
git log --oneline \
  35e4dc20864ae4896dccc2b20cbcdbe3fb733db8..HEAD
git show --stat --oneline bfee1ca8356ac191521e112f835bf3a4af0993d3
git show --stat --oneline 3f0affd0e375975dd18ea895219ab77477b41325
git show --stat --oneline 9247cc5e4dbc82a5f4ecc381f8b8b5084c9bc628
git show --stat --oneline a4db430cb3c80fc6c1fd4100821c643129e87d5f
git show --stat --oneline dabf30550a968f49e7e0a6d25984d0ef99e779ee
git show --stat --oneline b652f28d222f6a6bb5d3aa34dd5595e52223c195
git show --stat --oneline b74113c95cdf1e952f8c45d928cbf178f8b1e485
git show --stat --oneline HEAD
python3 .github/scripts/classify_ci_changes.py --paths \
  CHANGELOG.md \
  docs/engineering/ai-delivery/implementation-reports/THOTH-DB-CTRL-01-SPEC-implementation-report.md \
  docs/engineering/ai-delivery/tasks/THOTH-DB-CTRL-01.md \
  docs/engineering/repository-map/control-gaps.md \
  docs/engineering/repository-map/repositories/thoth.md
git status --short
```

The exact outputs, final head, eight commit scopes
(`4 / 1 / 2 / 2 / 1 / 1 / 3 / 1`), and classifier JSON are recorded in the
superseding immutable PR evidence after the report-finalization commit exists.
The cumulative path set must remain the same five documentation paths. The
approval-state range after `b652f28d222f6a6bb5d3aa34dd5595e52223c195` must
contain exactly the three approval-state documents plus this report, with
commit scopes `3 / 1`.

## 18. Manual verification

Environment: repository checkout at the exact authorized base plus a unique
PostgreSQL 17 container with a loopback-published client endpoint and isolated
Diesel CLI `2.3.10`.

Steps:

1. inspected repository commands, source, history, generated contract, models,
   workflow, and control documents;
2. established config parsing and config-relative path behaviour without
   writing a committed path;
3. applied, reverted, and reapplied every migration;
4. generated and compared exact CLI outputs twice;
5. compared raw and canonical structure;
6. tested full and reduced patch approaches with one controlled disposable
   column;
7. compiled the candidate in a detached temporary worktree;
8. selected the only bounded approach that retains canonical location and
   makes intentional conventions explicit;
9. removed every disposable resource.

Observed result: discovery completed without production effect or repository
experiment residue.

For the second review remediation, the standalone-table schema candidate was
also compiled successfully without a consuming model in a detached temporary
worktree, after which that worktree and its artifacts were removed.

Evidence: this report, the original immutable top-level PR #775 evidence
comment, and the successive superseding post-remediation evidence comments.

## 19. CI

CI status: PENDING for the approval-report-finalization head

The approval-state reviewed head
`b652f28d222f6a6bb5d3aa34dd5595e52223c195` completed its documentation-only
checks, but that CI is historical after the approval-state commits. Fresh
exact-head PR CI must prove:

```text
build-test-and-check:
  classify: success
  build: skipped
  test: skipped
  lint: skipped
  format_check: skipped

run-migrations:
  classify: success
  run_migrations: skipped

publish-to-dockerhub:
  classify: success
  build_and_push_staging_docker_image: skipped

check-changelog:
  success
```

Every skipped job must have an empty or absent step array. Exact workflow run
and job IDs are recorded in the superseding immutable PR evidence only after
all required checks at the final head are terminal and successful. No workflow
is manually dispatched.

## 20. Rollout and rollback

Initial state after merge: documentation-only approved specification; no active
control, implementation, or runtime effect
Activation required: separate implementation task, fresh implementation-branch
authorization, fresh branch and PR, independent cross-model review, and
explicit CTO implementation merge authorization
Feature flag/configuration: none
Migration sequence: none for this task
Rollback/disable procedure: revert the documentation PR; no data rollback
Monitoring required: none

The future implementation's initial state keeps
`thoth-api/src/schema.rs` byte-identical while directing automatic Diesel output
to ignored `target/diesel-schema.rs`; only validated synchronizer `generate`
may later write the canonical file. Its rollback must restore the config,
tracked `.github/scripts/` tools, convention data, Makefile/workflow/classifier,
repository control records, and both authoritative AGENTS files together. If
that reopens ambiguity, dependent schema work blocks again.

Previous approval `5123408720` and CTO authorization `5169190785` are
historical. The 2026-08-03 CTO approval recorded at approval-state content head
`b74113c95cdf1e952f8c45d928cbf178f8b1e485` approves the written specification
only. Fresh exact-head independent review and fresh explicit CTO merge
authorization are required before another ready transition or merge attempt.

If the future implementation is rolled back and schema generation becomes
ambiguous again, CG-12 reopens and all dependent schema work returns to
`BLOCKED`.

## 21. Known limitations and deferred work

- The selected control specification is approved but not implemented.
- Root `diesel.toml` still has its discovered invalid configuration; no staging
  path, synchronizer, or canonical sole-writer enforcement exists until the
  separately authorized implementation.
- Root `AGENTS.md` and `thoth-api/AGENTS.md` retain their current gap language in
  this specification PR and must be replaced together by the future
  implementation.
- The structured version-2 manifest and tracked `.github/scripts/` tools are
  normative future requirements, not repository code added by this PR.
- The completed discovery did not record PostgreSQL's server/client connection
  addresses or ports; the implementation must add and test that evidence.
- The standalone-table candidate is compile-valid, but only the future
  synchronizer can prove the exact five complete version-2 objects, four-way
  structural equality, deterministic rendering, removal, and byte-identical
  restoration end to end.
- The initial convention file must enumerate and independently verify every
  existing timestamp override and ordering rule during implementation.
- CI does not yet run the exact-version structural check.
- CG-12 remains unresolved until the implementation merges with acceptance
  evidence.
- CG-13 runtime operations remain open.
- BE-01 remains blocked.

## 22. Unresolved issues

- All four post-ready P1 findings are normatively corrected at specification
  content head `dabf30550a968f49e7e0a6d25984d0ef99e779ee`; their GitHub replies and
  resolutions followed successful exact-head CI at
  `b652f28d222f6a6bb5d3aa34dd5595e52223c195` and are recorded in superseding
  immutable evidence.
- The approval-state P1 against
  `b652f28d222f6a6bb5d3aa34dd5595e52223c195` is corrected at immutable
  approval-state content head
  `b74113c95cdf1e952f8c45d928cbf178f8b1e485`.
- Fresh independent cross-model review of this specification is required.
- Fresh explicit CTO merge authorization is required before the specification
  PR may be marked ready or merged.
- Merging the specification will not authorize `THOTH-DB-CTRL-01`
  implementation.

## 23. Agent self-assessment

The agent does not approve its own work.

Suggested review focus:

- whether the task, CG-12 record, repository map, and this report consistently
  record `APPROVED` specification status while keeping implementation not
  started, its branch unauthorized, CG-12 unresolved, CG-13 open, and BE-01
  blocked;
- whether specification approval, implementation-branch authorization,
  implementation work, and implementation merge authorization remain distinct;
- whether automatic Diesel output is confined to ignored staging and every
  direct Diesel command, staging mutation, or validation failure is unable to
  change or promote the canonical schema;
- whether structured version-2 add/remove/change objects completely encode
  PostgreSQL/Diesel types, nullability, ordinals, ordered keys, complete joins,
  allow-table membership, and ordered enum labels with exact four-way equality;
- whether every current manual convention can be represented as bounded,
  reviewable control data;
- whether loopback client-endpoint enforcement, server/client address
  inspection, local Docker identity/mount/storage proof, and GitHub Actions
  workflow/job provenance prevent accidental shared or production access;
- whether the future thirteen-path scope correctly includes root `AGENTS.md`,
  `thoth-api/AGENTS.md`, and tracked `.github/scripts/` without `.gitignore` or
  canonical-schema changes;
- whether this report binds the normative content to exact head
  `dabf30550a968f49e7e0a6d25984d0ef99e779ee` and accurately separates the
  following report-only finalization commit;
- whether the compile-valid standalone probe, removal, compile gate, and CI
  sequence establish both clean no-op and isolated expected-diff behaviour;
- whether the exact implementation path list is sufficient without scope
  expansion.

No-effect assessment:

```text
Production migration: NO
Production schema or data change: NONE
Runtime or API behaviour: UNCHANGED
Deployment or release: NONE
diesel.toml or automatic output behaviour: UNCHANGED BY THIS SPECIFICATION PR
AGENTS implementation instructions: UNCHANGED BY THIS SPECIFICATION PR
THOTH-DB-CTRL-01 specification: APPROVED
BE-01 implementation: NOT STARTED
THOTH-DB-CTRL-01 implementation: NOT STARTED
THOTH-DB-CTRL-01 implementation branch: ABSENT AND NOT AUTHORIZED
CG-12: OPEN
CG-13: OPEN
```
