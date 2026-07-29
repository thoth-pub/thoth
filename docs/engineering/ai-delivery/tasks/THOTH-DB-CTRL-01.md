# THOTH-DB-CTRL-01 - Diesel generation procedure

Status: DRAFT
Programme: Shared Repository Controls
Repository: thoth-pub/thoth
Workflow: STANDARD
Base branch: develop
PR target: develop
Programme integration branch: None
Risk: HIGH
Owner: CTO
Approved by: Not approved; draft awaiting independent review and explicit CTO approval
Dependencies: PR #774 merged as `35e4dc20864ae4896dccc2b20cbcdbe3fb733db8`
Target branch name: `feature/repository-controls/thoth-db-ctrl-01`

## 1. Objective

Establish one repository-authoritative, repeatable, fail-closed procedure for
creating and applying Thoth migrations and synchronising the migrated
PostgreSQL schema with the canonical Diesel contract at
`thoth-api/src/schema.rs`. The procedure must make a clean baseline a
byte-for-byte no-op, admit only an explicitly declared schema change, preserve
the repository's required custom types and model-compatible ordering, and
provide the same verification locally and in CI.

### 1.1 Risk rationale

This repository control is HIGH risk because future populated-database changes
in multiple programmes will depend on it. A false clean result could permit a
stale or uncompilable schema contract, while an uncontrolled regeneration could
silently reorder model fields or discard manually maintained type semantics.
The implementation changes no production schema or runtime behaviour, but the
control it establishes must be independently reviewed and explicitly approved.

## 2. Background and authority

Authoritative sources, in precedence order:

1. merged migrations under `thoth-api/migrations/`, the migrated disposable
   PostgreSQL schema, `thoth-api/src/schema.rs`, and the Rust models that consume
   that schema;
2. `AGENTS.md`, `docs/engineering/AGENTS.md`, and `thoth-api/AGENTS.md`;
3. the engineering operating model, risk classification, release gates, and
   branching workflow under `docs/engineering/ai-delivery/`;
4. [CG-12 and CG-13](../../repository-map/control-gaps.md);
5. the [`thoth` repository map](../../repository-map/repositories/thoth.md);
6. BE-01 ([publisher package specification](BE-01.md)), which remains blocked
   on this control;
7. the Publisher Services [programme controls](../../../publisher-services/README.md)
   and [task status](../../../publisher-services/task-status.md).

### 2.1 Discovery findings

Repository and disposable-database testing at authorized base
`35e4dc20864ae4896dccc2b20cbcdbe3fb733db8` established:

- the canonical working directory is the repository root;
- `make migration` creates
  `thoth-api/migrations/YYYYMMDD_v<next-minor>/up.sql` and `down.sql`;
- `cargo run migrate --database-url "$DATABASE_URL"` applies every embedded
  migration, while adding `--revert` invokes `revert_all_migrations` and
  therefore reverts the full chain;
- root `diesel.toml` is the configuration Diesel discovers from the repository
  root, and Diesel resolves `print_schema.file` relative to the directory
  containing that configuration;
- the current `file = "src/schema.rs"` therefore targets root `src/schema.rs`,
  not the canonical `thoth-api/src/schema.rs`;
- current `diesel.toml` does not parse because four commas are missing from
  `custom_type_derives`;
- even with only those commas repaired in a temporary copy, the array is
  semantically invalid: import globs and application model types are configured
  as derives on every generated SQL type;
- the repository locks Diesel `2.3.10`; an isolated PostgreSQL-only Diesel CLI
  `2.3.10` generated identical output twice;
- raw Diesel `2.3.10` output is not the committed contract: the normalized diff
  was 382 insertions and 455 deletions, covering formatting, table and column
  aliases, column order, custom type handling, and maintained
  `Timestamptz` mappings;
- replacing the committed schema with a minimally normalized raw schema failed
  `cargo check -p thoth-api --features backend` with 86 errors, including a
  missing supplemental `MarkupFormat`, an unresolved renamed-table join, and
  widespread Diesel `CompatibleType` failures caused by column ordering;
- a full raw-to-committed Diesel patch reproduced the clean baseline but failed
  to apply after a deliberately controlled column addition; it is therefore not
  a safe evolving control;
- a reduced patch could admit the controlled column but still produced the
  uncompilable column ordering and supplemental-type failures.

The canonical contract cannot safely be replaced by unbounded `diesel
print-schema` output. It can be maintained safely without relocation by
combining raw database introspection with an explicit, reviewable convention
file and a bounded structural synchronizer.

### 2.2 Absent requested orientation paths

`docker-compose.yaml` and `thoth-api/src/bin/arguments/mod.rs` do not exist at
the authorized base. The repository uses `docker-compose.yml`; migration CLI
argument handling is in root `src/bin/arguments/mod.rs`.

## 3. Explicit scope

The implementation task must:

1. establish the repository root as the only supported working directory;
2. retain root `diesel.toml` as the authoritative Diesel configuration;
3. correct that config to target `thoth-api/src/schema.rs`;
4. replace invalid `custom_type_derives` entries with the minimal valid derive
   set required for generated SQL types;
5. pin Diesel CLI compatibility to exact `2.3.10`, matching the locked Diesel
   crate, and fail when the executable differs;
6. add `thoth-api/diesel-schema-control.toml`, explicitly enumerating every
   current supplemental SQL type, physical-to-Rust table or column alias,
   database-to-contract type override, and order-preservation rule;
7. add `scripts/diesel_schema.py`, a literal-safe structural synchronizer with
   `check` and `generate` modes;
8. make the synchronizer introspect a proven disposable local PostgreSQL
   database through raw `diesel print-schema` output captured in a temporary
   directory;
9. preserve `thoth-api/src/schema.rs` byte-for-byte when the database matches
   committed migrations and the convention file accounts for every intentional
   difference;
10. require an explicit expected-change manifest for any generated change and
    fail if the observed structural delta is smaller, larger, or different;
11. preserve required custom PostgreSQL type derives, supplemental types,
    aliases, column order, and unchanged formatting;
12. permit `generate` to write only `thoth-api/src/schema.rs`, atomically, after
    all safety and expected-diff checks pass;
13. add bounded Makefile targets for local check and generation;
14. extend migration CI to install the exact compatible CLI, validate the
    migrated schema, and fail on stale, nondeterministic, or unexpected output;
15. add focused parser, target-safety, expected-diff, literal-safety, and
    integration tests;
16. update `CHANGELOG.md`, repository control records, and the implementation
    report.

## 4. Non-goals

The implementation task must not:

1. implement BE-01;
2. add `publisher.subscription_package`;
3. add `thoth_package`;
4. create the BE-01 migration;
5. modify publisher models or GraphQL;
6. run a production migration;
7. access production services or secrets;
8. resolve CG-13 runtime operations;
9. upgrade Diesel unless separately approved;
10. relocate the canonical schema;
11. refactor unrelated migrations or Rust code;
12. change any approved package or capability architecture;
13. make raw Diesel output authoritative over current compiled model
    compatibility;
14. hide an unexplained database/contract difference in a broad patch.

## 5. Invariants

The implementation must preserve:

1. every existing committed migration and its ordering;
2. `thoth-api/src/schema.rs` as the canonical generated/derived schema;
3. every current table, column, relationship, custom SQL type, alias, and
   model-compatible column order;
4. current runtime, database, authorization, GraphQL, export, and internal
   client behaviour;
5. public and protected API contracts;
6. all database contents;
7. production inactivity;
8. BE-01's `BLOCKED` state;
9. the absence of `feature/publisher-services/be-01`;
10. the absence of `feature/repository-controls/thoth-db-ctrl-01` until this
    specification is independently approved and merged.

## 6. Required behaviour

### 6.1 Canonical tools, paths, and working directory

All commands run from the repository root.

| Purpose | Authoritative value |
|---|---|
| Migration creation | `make migration` |
| Migration directory | `thoth-api/migrations/YYYYMMDD_v<next-minor>/` |
| Forward migration | `cargo run migrate --database-url "$DATABASE_URL"` |
| Full-chain revert on disposable targets only | `cargo run migrate --revert --database-url "$DATABASE_URL"` |
| Diesel configuration | `diesel.toml` |
| Diesel CLI | exact `2.3.10`, PostgreSQL feature |
| Canonical schema | `thoth-api/src/schema.rs` |
| Convention file | `thoth-api/diesel-schema-control.toml` |
| Synchronizer | `scripts/diesel_schema.py` |

The implementation must correct root `diesel.toml` to the following effective
configuration:

```toml
[print_schema]
file = "thoth-api/src/schema.rs"
custom_type_derives = ["diesel::query_builder::QueryId"]
```

Diesel already adds its `SqlType` derive. Application model paths and
`diesel::sql_types::*` are imports, not derives, and must not appear in
`custom_type_derives`.

### 6.2 Safe target gate

`scripts/diesel_schema.py` must:

1. accept the database URL only through the existing `DATABASE_URL` environment
   variable and never log it;
2. require `THOTH_DIESEL_CONFIRM_DATABASE` to exactly equal the name parsed
   from the URL and returned by `SELECT current_database()`;
3. require the parsed client-facing URL host to be exactly `localhost`,
   `127.0.0.1`, or `::1`;
4. query and record `inet_server_addr()`, `inet_server_port()`,
   `inet_client_addr()`, and `inet_client_port()` over the established
   PostgreSQL connection;
5. classify the server-side accepted address independently from the
   client-facing URL and accept it only when it is:
   - loopback; or
   - a private container-network address whose provenance is established by
     the local Docker or GitHub Actions checks below;
6. reject a public, externally routable, unexplained, null, or unverified
   server-side address;
7. query and report only safe target metadata: provenance mode, client endpoint
   class and port, server and client connection addresses and ports, database
   name, database user, server version, migration count, table count, and enum
   count;
8. reject public, staging, production, shared-development, ambiguous, or
   unconfirmed targets before running Diesel;
9. never print credentials, the full URL, table contents, or personal data.

The client-facing database endpoint must always be loopback. PostgreSQL's
server-side accepted address is evidence to inspect and classify, but it is not
required to be loopback when Docker or GitHub Actions maps a loopback host port
to a PostgreSQL service on a private bridge network.

#### 6.2.1 Local Docker provenance

Local developer mode must additionally:

1. require a non-default explicit client port and a database name beginning
   `thoth_diesel_`;
2. require the test wrapper to supply the exact task-created container identity
   through `THOTH_DIESEL_CONTAINER`;
3. inspect that exact container and require it to be running, task-disposable,
   and mapped from the URL's loopback host and explicit port to PostgreSQL port
   `5432`;
4. require the queried server-side address to match an address assigned to that
   container and to be loopback or private container-network space;
5. require the URL, `THOTH_DIESEL_CONFIRM_DATABASE`, `current_database()`, and
   the container's expected database identity to agree;
6. require the connected database user to match the expected disposable user;
7. reject host or repository bind mounts, named or externally managed durable
   storage, and any container that cannot be tied to the task wrapper;
8. require the wrapper's cleanup proof to show that the container and its
   anonymous disposable storage no longer exist after the test.

#### 6.2.2 GitHub Actions provenance

The GitHub Actions PostgreSQL service is permitted only when all of the
following hold:

1. `GITHUB_ACTIONS=true`;
2. `GITHUB_REPOSITORY=thoth-pub/thoth`;
3. `GITHUB_WORKFLOW_REF` identifies
   `.github/workflows/run_migrations.yml` and `GITHUB_JOB=run_migrations`;
4. the workflow-controlled URL is `localhost:5432`, with database `thoth` and
   user `thoth`;
5. `THOTH_DIESEL_CONFIRM_DATABASE`, the parsed URL name, and
   `current_database()` all equal `thoth`;
6. the queried server-side address is loopback or private container-network
   space;
7. execution outside that approved migration job context fails closed.

### 6.3 Configuration and CLI validation

Before database introspection the synchronizer must:

1. require `Path.cwd()` and `git rev-parse --show-toplevel` to equal the
   repository root containing the script;
2. resolve `diesel.toml`, the convention file, and the canonical schema through
   absolute paths beneath that root;
3. parse `diesel.toml` and assert its effective output path is exactly the
   canonical path;
4. execute `diesel --version` and require exact `diesel 2.3.10`;
5. reject symlinks or resolved output paths outside the repository;
6. capture raw schema with the equivalent of:

   ```bash
   diesel print-schema \
     --config-file /absolute/repository/diesel.toml \
     --database-url "$DATABASE_URL"
   ```

   into a private temporary file, never a repository path.

The implementation documentation must provide this isolated installation
command:

```bash
cargo install diesel_cli \
  --version '2.3.10' \
  --root "$THOTH_DIESEL_CLI_ROOT" \
  --no-default-features \
  --features postgres \
  --locked
```

`THOTH_DIESEL_CLI_ROOT` must be a new temporary directory outside the
repository. The installed binary is supplied through `DIESEL_BIN`; no global
installation or user toolchain change is required.

### 6.4 Convention file and structural comparison

`thoth-api/diesel-schema-control.toml` is reviewable control data, not a
free-form patch. It must enumerate the current:

- supplemental `MarkupFormat` SQL type;
- `abstract` to `work_abstract` and `title` to `work_title` table aliases;
- `title.title` physical-to-Rust identifier handling;
- every database `Timestamp` to canonical `Timestamptz` override;
- existing table and column order that Rust `Queryable` models require.

The implementation must derive the initial entries from a migrated clean
database and the compiled canonical schema, then independently review every
entry. Unknown, unused, duplicate, conflicting, or missing entries fail
non-zero. The file may not contain a catch-all pattern.

The synchronizer must parse raw and canonical Diesel macro structures and
compare:

- SQL type names and derives;
- physical schema/table/column names after aliases;
- nullability and base type;
- primary keys;
- table and column order;
- `joinable!` relationships;
- `allow_tables_to_appear_in_same_query!` membership.

Unchanged canonical blocks and whitespace must be emitted byte-for-byte.
Expected new structures use one deterministic renderer. Removals require
explicit manifest entries. A parse ambiguity or unsupported macro form fails
closed.

### 6.5 Expected-change manifest

The manifest is a task-local UTF-8 TOML file with:

```toml
version = 1
add = ["column:public.publisher.thoth_db_ctrl_probe"]
remove = []
change = []
```

Allowed entry forms are exactly:

```text
sql-type:<schema>.<type>
table:<schema>.<table>
column:<schema>.<table>.<column>
primary-key:<schema>.<table>
join:<child-schema>.<child-table>-><parent-schema>.<parent-table>
type:<schema>.<table>.<column>:<old-type>-><new-type>
nullability:<schema>.<table>.<column>:<old>-><new>
```

Entries are unique and sorted. The synchronizer must calculate the structural
database delta before writing and require exact equality with the manifest.
Manifest entries do not override database truth and cannot exempt unrelated
differences. The implementation task's controlled probe must use only the
example `thoth_db_ctrl_probe` column on its disposable database and must remove
it after testing; it must not create or mention a BE-01 database object in
generated output.

### 6.6 Canonical commands

After exporting a disposable `DATABASE_URL`, setting
`THOTH_DIESEL_CONFIRM_DATABASE` to its exact database name, and setting
`DIESEL_BIN` to exact CLI `2.3.10`, the no-op command is:

```bash
make check-diesel-schema
```

It must run check mode twice, compare the two temporary candidates with `cmp`,
compare the candidate with `thoth-api/src/schema.rs`, verify `git status
--short` is unchanged, and exit zero only with:

```text
THOTH_DIESEL_TARGET=SAFE_DISPOSABLE_LOCAL
THOTH_DIESEL_CLI=2.3.10
THOTH_DIESEL_CONFIG=diesel.toml
THOTH_DIESEL_SCHEMA=thoth-api/src/schema.rs
THOTH_DIESEL_CLIENT_ENDPOINT=LOOPBACK
THOTH_DIESEL_SERVER_ADDRESS=LOOPBACK_OR_VERIFIED_PRIVATE_CONTAINER
THOTH_DIESEL_REPEAT=IDENTICAL
THOTH_DIESEL_DIFF=CLEAN
```

The controlled generation command is:

```bash
python3 scripts/diesel_schema.py generate \
  --expected-change "$THOTH_DIESEL_EXPECTED_CHANGE_FILE" \
  --output thoth-api/src/schema.rs
```

It must atomically replace only the canonical schema after all checks pass. It
must print only safe named statuses and an aggregate added/removed/changed
count. With an empty manifest on a clean migrated database it is a byte-for-byte
no-op.

### 6.7 Required success behaviour

A clean migrated disposable database must produce a deterministic candidate.
Repeated generation must be identical. When the database matches committed
migrations, check mode must report no structural or textual difference and
must not change the worktree.

A controlled disposable test column must produce exactly one manifest-approved
isolated schema addition. Repeating generation must reproduce the same
candidate. Removing the test column and using an empty manifest must restore
the byte-for-byte clean baseline. The candidate must pass
`cargo check -p thoth-api --features backend` when substituted only in a
temporary worktree.

### 6.8 Required failure behaviour

The procedure must fail non-zero before repository writes if:

- the database target is not approved, local, and disposable;
- migrations do not apply;
- configuration does not parse;
- the configured or requested path is wrong;
- Diesel CLI is absent or not exact `2.3.10`;
- raw output or a repeated candidate is nondeterministic;
- the observed structural delta differs from the manifest;
- a convention entry is missing, unused, broad, or conflicting;
- required custom derives or supplemental types are lost;
- a candidate fails the focused Rust compile check;
- an unrelated repository file changes;
- a resolved path leaves the repository;
- the working directory is not the repository root;
- shell metacharacters alter argument boundaries or any subprocess uses a shell.

The command must emit:

```text
BLOCKED - THOTH DIESEL GENERATION CONTROL FAILED
```

with one safe reason code and no partial write.

### 6.9 Authorization

No application authorization path changes. Database verification is restricted
to a local disposable target or the ephemeral CI service. Production
credentials and targets are prohibited.

### 6.10 Concurrency and idempotency

The synchronizer must take an exclusive lock beside the canonical schema,
write through a same-directory temporary file, `fsync`, and atomically rename.
Concurrent generation must fail rather than interleave. Check mode is
read-only. Repeating either mode against unchanged inputs must be idempotent.

### 6.11 Compatibility

The implementation must preserve all API and database contracts. The
model-compatible canonical column order and supplemental type are compatibility
requirements, not formatting preferences. A candidate must compile against
the existing `thoth-api` backend before it can replace the canonical file.

## 7. Data and migration requirements

Production migration required: NO
Schema change required: NO
Backfill required: NO
Production data effect: NONE

Disposable migration-chain validation is required:

1. start from an empty PostgreSQL 17 database;
2. run `cargo run migrate --database-url "$DATABASE_URL"`;
3. verify all committed migrations, tables, types, constraints, and indexes;
4. run `cargo run migrate --revert --database-url "$DATABASE_URL"`;
5. verify the application schema is empty;
6. reapply the full chain;
7. run no-op and controlled-diff schema verification.

Full-chain revert is prohibited outside the uniquely named disposable target.
Populated-database migration proof remains a requirement of each future schema
task; this control implementation introduces no populated-data change.

## 8. Observability and operations

Required output consists only of the named safe statuses in section 6 and
aggregate structural counts. CI must retain command exit status and the safe
status lines as evidence.

No runtime metrics, alerts, deployment, production activation, or operational
runbook change is required. CG-13 remains open.

## 9. Acceptance criteria

- [ ] The exact repository-root working directory and commands are documented.
- [ ] Migration creation is exactly `make migration`.
- [ ] Forward and disposable full-chain revert commands are exact.
- [ ] Root `diesel.toml` parses and targets `thoth-api/src/schema.rs`.
- [ ] Diesel CLI exact `2.3.10` and the isolated installation policy are enforced.
- [ ] The convention file explicitly accounts for every current intentional raw/canonical difference without catch-all patterns.
- [ ] A clean migrated PostgreSQL 17 database produces a byte-identical no-op.
- [ ] Two consecutive candidates are byte-identical.
- [ ] `MarkupFormat`, aliases, timestamp semantics, model-compatible column order, and required derives are preserved.
- [ ] A controlled expected-diff test admits only its isolated manifest entry.
- [ ] Removing the controlled change restores the clean baseline.
- [ ] Any stale or unexpected schema difference fails non-zero before writing.
- [ ] An unexpected repository file change fails.
- [ ] Unchanged schema content receives no unrelated formatting change.
- [ ] Local and CI checks call the same synchronizer.
- [ ] The client-facing database endpoint is loopback in local and CI modes.
- [ ] Server and client connection addresses and ports are queried, recorded,
      classified, and public or unexplained server addresses fail closed.
- [ ] Local execution is tied to the exact task-created disposable Docker
      container, loopback-published port, expected database/user, mount policy,
      and cleanup proof.
- [ ] CI execution is tied to the `thoth-pub/thoth`
      `run_migrations.yml`/`run_migrations` job, expected endpoint,
      database/user, and private service-container address.
- [ ] Database target validation logs no credential or URL.
- [ ] The generated candidate passes the focused backend compile check.
- [ ] Rollback restores repository state without touching database data.
- [ ] BE-01 remains blocked and no BE-01 implementation object is created.
- [ ] CG-12 closes only after this implementation passes independent review and merges with acceptance evidence.
- [ ] CG-13 remains open.

## 10. Required tests

### Unit

- Parse the valid corrected config and reject the current invalid form.
- Validate every convention-file entry and reject unused, duplicate, broad, or
  conflicting entries.
- Parse representative SQL types, aliases, table blocks, primary keys,
  joinables, nullable types, and ordered columns.
- Render unchanged blocks byte-for-byte and new structures deterministically.
- Validate sorted, unique expected-change manifests.
- Invoke subprocesses with argument arrays and `shell=False`; prove literal
  metacharacters cannot become shell syntax.

### Integration/database

- Apply the clean empty-database migration chain on PostgreSQL 17.
- Revert the full chain on that disposable database and reapply it.
- Run no-op generation twice and require `cmp` success.
- Preserve custom SQL types, derives, aliases, overrides, and order.
- Add only `publisher.thoth_db_ctrl_probe text`, require exactly one expected
  addition, repeat it deterministically, then remove it and restore clean.
- Reject a stale canonical schema.
- Reject an unlisted database change.
- Reject an unexpected output path or unrelated changed file.

### Authorization/security

- Reject a non-loopback client URL before connection.
- Query and record `inet_server_addr()`, `inet_server_port()`,
  `inet_client_addr()`, and `inet_client_port()`; accept only loopback or
  provenance-verified private container-network server addresses.
- Reject public, externally routable, null, unexplained, or unverified
  server-side addresses.
- Reject a database confirmation mismatch, default local developer port, wrong
  database prefix, symlink escape, and production-like name.
- Reject local execution without the exact task-created container identity,
  expected port mapping, database/user match, disposable storage, mount policy,
  or cleanup proof.
- Reject CI execution outside the expected repository, migration workflow and
  job, loopback endpoint, database/user, and service-address classification.
- Verify safe output contains no URL, credential, table content, or personal
  data.

### Regression

- `cargo check -p thoth-api --features backend` with the no-op candidate.
- `cargo test --workspace`.
- `cargo check --workspace`.
- `cargo clippy --all --all-targets --all-features -- -D warnings`.
- `cargo fmt --all -- --check`.
- `git diff --check`.

### Manual verification

- Independently inspect every initial convention entry against the migrated
  database, canonical schema, and consuming model.
- Verify only the approved implementation paths changed.
- Verify CG-12 remains open until the implementation PR merges.

### Performance

No runtime performance target applies. CI schema verification must complete
within the existing migration workflow timeout and must not contact an external
database.

## 11. CI verification

The implementation must update `.github/workflows/run_migrations.yml` so its
PostgreSQL 17 job:

1. installs exact PostgreSQL-only Diesel CLI `2.3.10` with `--locked`;
2. applies all migrations;
3. supplies the expected CI database confirmation and runs
   `make check-diesel-schema` against the workflow-controlled
   `localhost:5432` service;
4. runs the existing full-chain revert;
5. reapplies migrations and repeats the schema check;
6. records the queried server/client addresses and ports plus the verified
   GitHub Actions provenance status;
7. fails on any non-zero control or provenance result.

The workflow must retain explicit job-level `contents: read`, use no production
environment or secret, and preserve the existing protected check identity. The
CI classifier must treat the synchronizer, convention file, config, Makefile,
canonical schema, and migration workflow as migration-control paths so those
changes cannot be misclassified as documentation-only.

## 12. Rollout

- initial state after merge: inactive repository-control improvement;
- feature flag/configuration: none;
- staging/preview validation: none; disposable local and CI databases only;
- pilot: the controlled probe described in this specification;
- activation approval: explicit CTO approval to merge the implementation;
- observation period: exact-head CI plus the first separately approved schema
  task;
- production activation: none.

Merging this specification does not authorize implementation. Merging the
implementation does not authorize BE-01, a production migration, release, or
deployment.

## 13. Rollback

- code rollback: revert the bounded implementation commit or PR, restoring the
  previous config, commands, script, convention data, workflow, and docs;
- data rollback: none, because the control implementation changes no durable
  database state;
- feature disable: stop invoking the new repository command;
- external side effects: none.

If rollback makes generation ambiguous again, CG-12 reopens and every dependent
schema task returns to `BLOCKED`.

## 14. Stop conditions

The implementing agent must stop and report the exact applicable result:

```text
BLOCKED - SAFE DATABASE TARGET UNVERIFIED
BLOCKED - COMPATIBLE DIESEL CLI UNAVAILABLE
BLOCKED - THOTH-DB-CTRL-01 REQUIRES ARCHITECTURE DECISION
BLOCKED - THOTH DIESEL GENERATION CONTROL FAILED
BLOCKED - SCOPE EXPANSION REQUIRED
```

It must also stop if the approved base moved, the worktree contains unrelated
changes, a canonical relocation or migration-ownership change appears
necessary, current schema conventions cannot be enumerated safely, a
disposable migration or compile path cannot be tested, or production access
would be required.

## 15. Expected implementation files

The implementation is expected to change only:

```text
diesel.toml
Makefile
scripts/diesel_schema.py
scripts/test_diesel_schema.py
thoth-api/diesel-schema-control.toml
.github/scripts/classify_ci_changes.py
.github/workflows/run_migrations.yml
CHANGELOG.md
docs/engineering/repository-map/control-gaps.md
docs/engineering/repository-map/repositories/thoth.md
docs/engineering/ai-delivery/implementation-reports/THOTH-DB-CTRL-01-implementation-report.md
```

`thoth-api/src/schema.rs` must remain byte-identical for the control
implementation. If the implementation cannot meet the acceptance criteria
within these paths, it must return `BLOCKED - SCOPE EXPANSION REQUIRED` instead
of broadening scope.

## 16. Recommended execution

Implementation model: Codex / strongest approved implementation-capable model
Reasoning level: HIGH or MAXIMUM
Independent reviewer: separate cross-model reviewer
Review reasoning level: HIGH or MAXIMUM
Explicit CTO merge approval: REQUIRED

## 17. Branch and integration plan

- branch source: then-current verified `develop`, after this specification
  merges and a fresh implementation authorization is recorded;
- task branch: `feature/repository-controls/thoth-db-ctrl-01`;
- pull-request target: `develop`;
- expected merge order: specification, then separately authorized control
  implementation, then separate control-state update, then dependent schema
  work;
- parent programme branch refresh requirement: not applicable;
- branch deletion after merge: YES;
- final programme PR required: NO;
- final release path: `develop -> master`.

## 18. Approval

Approved for implementation by: Not approved
Date: Not approved
Notes: This document is a draft. Fresh independent cross-model review and
explicit CTO approval are required before the implementation branch may be
created. Approval of the specification PR does not authorize implementation,
BE-01, production migration, release, or deployment.

## 19. Rejected alternatives

### Direct raw regeneration

Rejected because deterministic Diesel `2.3.10` output differed from the
canonical contract and the normalized replacement failed compilation with 86
errors. Raw order and types do not preserve current `Queryable` compatibility
or supplemental `MarkupFormat`.

### Full Diesel patch file

Rejected because a patch that recreated the current file failed to apply after
one controlled column addition. It encodes unstable formatting context and
would block legitimate schema evolution.

### Reduced semantic patch

Rejected because it admitted the controlled column but retained Diesel database
ordinal order and omitted supplemental conventions, producing the same compile
class of failures.

### Manual regeneration and review

Rejected because it cannot prove no-op determinism, exact expected diff, or
fail-closed handling and would permit unrelated formatting or contract edits.

### Crate-local configuration or canonical relocation

Rejected because root configuration discovery is established, a corrected
root-relative path is sufficient, and relocating the canonical schema is
outside scope.

### Diesel upgrade or downgrade

Rejected because exact `2.3.10` is already locked by the repository and
generated deterministically. Version change does not solve model-specific
ordering and supplemental-type requirements and needs separate approval.
