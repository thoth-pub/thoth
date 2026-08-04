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
Approved by: Not approved for the projection-mode-corrected content
Approval date: Not applicable; the previous approval is historical
Dependencies: PR #774 merged as `35e4dc20864ae4896dccc2b20cbcdbe3fb733db8`
Target branch name: `feature/repository-controls/thoth-db-ctrl-01`

## 1. Objective

Establish one repository-authoritative, repeatable, fail-closed procedure for
creating and applying Thoth migrations and synchronising the migrated
PostgreSQL schema with the canonical Diesel contract at
`thoth-api/src/schema.rs`. The procedure must make a clean baseline a
byte-for-byte no-op, admit only an explicit `change` or `none` projection
expectation, preserve the repository's required custom types and
model-compatible ordering, and provide the same verification locally and in
CI.

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
3. correct that config so automatic Diesel output targets only ignored,
   untrusted staging at `target/diesel-schema.rs`, never the canonical schema;
4. replace invalid `custom_type_derives` entries with the minimal valid derive
   set required for generated SQL types;
5. pin Diesel CLI compatibility to exact `2.3.10`, matching the locked Diesel
   crate, and fail when the executable differs;
6. add `thoth-api/diesel-schema-control.toml`, explicitly enumerating every
   current supplemental SQL type, physical-to-Rust table or column alias,
   database-to-contract type override, and order-preservation rule;
7. add `.github/scripts/diesel_schema.py`, a literal-safe structural
   synchronizer with `check` and `generate` modes;
8. require an immutable full-SHA `THOTH_DIESEL_BASE_REF` equal to the task's
   authorized base and ancestral to the candidate head, and use a detached
   temporary worktree at that ref;
9. apply baseline migrations from that worktree to an empty proven disposable
   PostgreSQL database, capture independently derived baseline catalog, raw
   Diesel, and canonical snapshots, apply only the candidate's pending
   migrations to the same database, and capture corresponding candidate
   snapshots;
10. make the synchronizer capture raw `diesel print-schema` output in private
    temporary files for both phases, ignoring or deleting automatic staging
    output before validation;
11. preserve `thoth-api/src/schema.rs` byte-for-byte when the database matches
   committed migrations and the convention file accounts for every intentional
   difference;
12. require an explicit expected-change manifest for any generated change and
    fail if any independently observed baseline-to-candidate projection is
    smaller, larger, or different;
13. preserve required custom PostgreSQL type derives, supplemental types,
    aliases, column order, and unchanged formatting;
14. make the synchronizer's validated `generate` mode the sole canonical writer
    and permit it to write only `thoth-api/src/schema.rs`, atomically, after all
    safety, capability-aware exact projection comparison, expected-diff, and
    compile checks pass;
15. add bounded Makefile targets for local check and generation;
16. extend migration CI to run the exact baseline-to-candidate two-phase
    procedure and fail on stale, nondeterministic, empty, or unexpected output;
17. add focused parser, base-ref, baseline-snapshot, target-safety,
    expected-diff, automatic-output-bypass, cleanup, literal-safety, and
    integration tests;
18. replace the stale Diesel control-gap instructions in root `AGENTS.md` and
    `thoth-api/AGENTS.md` with the implemented canonical procedure while
    preserving CG-13 as a separate open control;
19. update `CHANGELOG.md`, repository control records, and the implementation
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
| Authorized baseline ref | exact full SHA in `THOTH_DIESEL_BASE_REF` |
| Baseline canonical schema | `<THOTH_DIESEL_BASE_REF>:thoth-api/src/schema.rs` |
| Diesel configuration | `diesel.toml` |
| Diesel CLI | exact `2.3.10`, PostgreSQL feature |
| Automatic Diesel staging output | `target/diesel-schema.rs` |
| Canonical schema | `thoth-api/src/schema.rs` |
| Convention file | `thoth-api/diesel-schema-control.toml` |
| Synchronizer | `.github/scripts/diesel_schema.py` |
| Synchronizer tests | `.github/scripts/test_diesel_schema.py` |

The implementation must correct root `diesel.toml` to the following effective
configuration:

```toml
[print_schema]
file = "target/diesel-schema.rs"
custom_type_derives = ["diesel::query_builder::QueryId"]
```

Diesel already adds its `SqlType` derive. Application model paths and
`diesel::sql_types::*` are imports, not derives, and must not appear in
`custom_type_derives`.

`target/` is already ignored by the repository. The configured
`target/diesel-schema.rs` is untrusted raw staging only;
`thoth-api/src/schema.rs` remains the canonical compiled contract and must never
be the configured automatic `print_schema.file`.

The synchronizer's validated `generate` mode is the sole canonical writer. A
direct `diesel print-schema`, `diesel migration run`, `diesel migration redo`,
`diesel migration revert`, or any other Diesel CLI command must never write
`thoth-api/src/schema.rs`. Automatic Diesel output may land only at
`target/diesel-schema.rs`, and changing that staging file must never cause
canonical promotion.

### 6.2 Safe target gate

`.github/scripts/diesel_schema.py` must:

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

The safe-target gate remains active without interruption while the database is
empty, while baseline migrations are applied and observed, while candidate
migrations are applied and observed, and during cleanup. The baseline and
candidate phases must use the same proven disposable database connection; no
second, shared, staging, or production database may supply either snapshot.

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
2. resolve `diesel.toml`, the automatic staging file, the convention file, and
   the canonical schema through absolute paths beneath that root;
3. parse `diesel.toml` and assert its effective output path is exactly
   `target/diesel-schema.rs`, is beneath the ignored `target/` directory, and is
   not the canonical path;
4. execute `diesel --version` and require exact `diesel 2.3.10`;
5. reject symlinks or resolved output paths outside the repository;
6. require `THOTH_DIESEL_BASE_REF` to be exactly 40 lowercase hexadecimal
   characters, resolve it as a commit without symbolic-ref or abbreviated-SHA
   expansion, require exact equality with the task's authorized base SHA, and
   require `git merge-base --is-ancestor "$THOTH_DIESEL_BASE_REF" HEAD` to
   succeed;
7. create a detached temporary worktree outside the repository at exactly
   `THOTH_DIESEL_BASE_REF`, verify its `HEAD`, and reject submodules, symlink
   escape, dirty state, or any base/candidate ref movement;
8. record the candidate canonical file's bytes before invoking Diesel and read
   the baseline canonical snapshot only from
   `<THOTH_DIESEL_BASE_REF>:thoth-api/src/schema.rs` in the verified worktree;
9. invoke exact Diesel CLI `2.3.10` and capture raw schema with the equivalent
   of:

   ```bash
   diesel print-schema \
     --config-file /absolute/repository/diesel.toml \
     --database-url "$DATABASE_URL"
   ```

   into a private temporary file, never the canonical path;
10. ignore or delete `target/diesel-schema.rs` before structural processing and
   never treat its content or modification as an authorization to promote;
11. apply convention and expected-change validation, compile the candidate, and
   atomically write the canonical file only in validated `generate` mode after
   every check passes;
12. prove the canonical bytes remain unchanged after every failed validation and
    after any direct Diesel CLI migration command.

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

The comparison model is capability-aware exact comparison over independently
representable projections. Each representation must be derived independently,
and exact comparison applies only to the projection that the representation can
expose:

| Structural fact | PostgreSQL catalog | Raw Diesel | Canonical schema | Manifest |
|---|---:|---:|---:|---:|
| Table identity | yes | yes | yes | yes |
| Column identity | yes | yes | yes | yes |
| PostgreSQL column type | yes | no | no | yes |
| Canonical Diesel column type | mapped projection | yes | yes | yes |
| Nullability | yes | yes | yes | yes |
| Ordinal/column order | yes | yes | yes | yes |
| Primary-key columns | yes | yes | yes | yes |
| Foreign-key endpoints | yes | mapped as join | mapped as join | yes |
| Join declaration | mapped projection | yes | yes | yes |
| Allow-table membership | no | yes | yes | yes |
| SQL-type identity | yes | yes | yes | yes |
| Ordered PostgreSQL enum labels | yes | no | no | yes |
| Supplemental canonical-only type | no | no | yes | convention data |

`yes` means that the representation independently exposes the fact. `mapped
projection` means that a deterministic documented mapping is required. `no`
means that absence must not be treated as a mismatch. A representation must
never be augmented with facts copied from another leg before comparison.

Unchanged canonical blocks and whitespace must be emitted byte-for-byte.
Expected new structures use one deterministic renderer. Removals require
explicit manifest entries. A parse ambiguity or unsupported macro form fails
closed.

### 6.5 Expected-change manifest

The manifest is a task-local UTF-8 TOML document with `version = 2` and exactly
one required projection expectation:

```toml
version = 2
expected_projection = "change"
```

or:

```toml
version = 2
expected_projection = "none"
```

Missing, duplicated, unknown, or case-variant modes fail closed. Every `add`,
`remove`, and `change` entry is an independently parsed and validated object;
string tokens, partial names, wildcards, catch-all entries, and broad exemptions
are prohibited.

`expected_projection = "change"` requires at least one non-empty independently
observed baseline-to-candidate catalog, raw-Diesel, or canonical projection.
Every controlled difference must have a complete matching manifest object, and
every applicable projection must equal its manifest projection exactly. An
all-empty controlled result fails even when candidate migrations are pending.
A label-only enum change uses `change` because its catalog projection is
non-empty even though its raw-Diesel and canonical projections are empty.

`expected_projection = "none"` requires no manifest `add`, `remove`, or
`change` objects, empty independently observed catalog, raw-Diesel, and
canonical projections, and a byte-identical canonical schema. Pending candidate
migrations are permitted in this mode. Any hidden table, column, key, join,
allow-table, SQL-type, or enum change fails closed.

The `none` result certifies only that the candidate migrations do not alter the
Diesel-controlled projection. It does not validate or approve their excluded
effects. Indexes, check constraints, data changes, comments, and other
structures outside this representation boundary remain subject to the task's
own migration validation and acceptance evidence.

The complete structural object schemas are:

| `kind` | Required fields |
|---|---|
| `table` | `schema`, `name` |
| `column` | `schema`, `table`, `name`, `postgres_type`, `diesel_type`, `nullable`, `ordinal` |
| `primary-key` | `schema`, `table`, ordered `columns` |
| `allow-table` | `schema`, `table` |
| `join` | `child_schema`, `child_table`, ordered `child_columns`, `parent_schema`, `parent_table`, ordered `parent_columns` |
| `sql-type` | `schema`, physical `name`, canonical `diesel_type`, and a complete `definition` |

For an enum `sql-type`, `definition` must contain `kind = "enum"` and ordered
database `labels`. This prevents a future `thoth_package` enum from matching by
name while containing the wrong labels or label order. Unsupported SQL-type
definitions fail closed rather than being represented incompletely.

The complete enum intent remains in manifest version 2:

```toml
kind = "sql-type"
schema = "public"
name = "<physical_type_name>"
diesel_type = "<canonical_type_name>"

[definition]
kind = "enum"
labels = ["FIRST", "SECOND"]
```

The complete object has different exact comparison projections:

```text
catalog projection:
schema, name, kind, ordered labels

raw Diesel projection:
schema/type identity and canonical Diesel type identity

canonical projection:
canonical SQL-type identity and expected derives

manifest:
complete object
```

Enum labels are mandatory manifest and catalog facts. They must not be made
optional merely because raw or canonical Diesel does not encode them.

The controlled probe manifest is exactly:

```toml
version = 2
expected_projection = "change"

[[add]]
kind = "table"
schema = "public"
name = "thoth_db_ctrl_probe"

[[add]]
kind = "column"
schema = "public"
table = "thoth_db_ctrl_probe"
name = "probe_id"
postgres_type = "uuid"
diesel_type = "Uuid"
nullable = false
ordinal = 1

[[add]]
kind = "column"
schema = "public"
table = "thoth_db_ctrl_probe"
name = "probe_value"
postgres_type = "text"
diesel_type = "Text"
nullable = true
ordinal = 2

[[add]]
kind = "primary-key"
schema = "public"
table = "thoth_db_ctrl_probe"
columns = ["probe_id"]

[[add]]
kind = "allow-table"
schema = "public"
table = "thoth_db_ctrl_probe"
```

A `remove` entry must contain the complete existing structural object being
removed, using the same schema as an `add` entry. A `change` entry must contain
complete `before` and `after` structural objects. Neither operation may be
expressed only as a name, a field-specific token, or a broad exemption.

For every representation-specific projection, manifest operations have exact
baseline-to-candidate semantics:

```text
add:
object absent from baseline projection
object exactly present in candidate projection

remove:
complete object exactly present in baseline projection
object absent from candidate projection

change.before:
exactly equal to the independently observed baseline object

change.after:
exactly equal to the independently observed candidate object
```

An `after` match cannot compensate for a false, stale, fabricated, or
unobserved `before` object. An undeclared removal, rename, reorder, or other
baseline-to-candidate difference fails closed.

The synchronizer must require capability-aware exact projection comparison.
Each comparison leg remains independently derived; catalog facts must never be
injected into raw or canonical Diesel merely to make objects appear equal.

#### 6.5.1 Authoritative baseline-to-candidate snapshots

The delta must be computed from two independently observed states on one proven
disposable database. A final-state-only comparison is prohibited.

1. Require `THOTH_DIESEL_BASE_REF` as an immutable full 40-character lowercase
   SHA equal to the task's authorized base, resolve it without abbreviation or
   symbolic indirection, and prove it is an ancestor of the candidate `HEAD`.
2. Create a detached temporary worktree outside the repository at exactly that
   commit and verify the worktree `HEAD`, clean state, and expected repository
   identity.
3. Prove the target database is empty, then apply the baseline migrations from
   the detached base worktree.
4. Capture an independently derived baseline PostgreSQL catalog snapshot,
   including complete ordered enum labels; capture baseline raw Diesel by
   running the exact pinned CLI against that database into a private temporary
   file; and parse the baseline canonical snapshot from
   `git show "${THOTH_DIESEL_BASE_REF}:thoth-api/src/schema.rs"`, verifying
   those bytes equal the file in the detached worktree.
5. Record the baseline migration ledger and require it to be an exact prefix of
   the candidate migration ledger. Do not reset, recreate, or substitute the
   database between phases.
6. From the unchanged candidate worktree, apply only the pending candidate
   migrations to the same database. Require at least one pending candidate
   migration for a migration-bearing task. Enforce the manifest's explicit
   projection expectation: `change` requires at least one non-empty controlled
   projection, while `none` requires every controlled projection to remain
   empty and the canonical schema to remain byte-identical.
7. Capture independently derived candidate catalog and raw-Diesel snapshots
   from the resulting database. Parse the candidate canonical snapshot from the
   current candidate tree or, in validated `generate` mode, from the generated
   candidate before promotion.
8. Compute baseline-to-candidate catalog, raw-Diesel, and canonical deltas and
   compare each with the matching manifest projection. Neither snapshot may be
   reconstructed from manifest claims or from the other phase.
9. On success or failure, remove the detached worktree, private snapshot files,
   generated candidates, database contents, container, and disposable storage.
   Cleanup failure is a control failure, and cleanup evidence must expose only
   safe aggregate statuses.

Both raw snapshots use the candidate control's validated pinned CLI and
configuration against the independently established database state; the
baseline worktree supplies the baseline migration set and canonical bytes. Raw
or catalog snapshot data must not be copied into the canonical leg.

The standalone-table probe uses the same acquisition mechanism: capture the
clean migrated baseline, apply only the controlled disposable DDL to the same
database, capture candidate snapshots, and compare the exact resulting delta.

#### 6.5.2 Catalog-to-manifest comparison

Require exact equality between the independently observed baseline-to-candidate
PostgreSQL catalog delta and the manifest projected to catalog-representable
fields. This includes physical schema and object identities, PostgreSQL column
types, nullability, ordinal positions, ordered primary-key columns, foreign-key
endpoints and columns, SQL-type identities, and complete ordered enum labels.
For enums, the two PostgreSQL catalog snapshots are the independently observed
sources of before and after label truth.

#### 6.5.3 Raw-Diesel comparison

Require exact equality between the independently observed baseline-to-candidate
raw Diesel delta and the manifest projected to raw-Diesel-representable fields.
This comparison includes only facts independently present in each
`diesel print-schema` snapshot: SQL-type identity; table and column identities;
generated Diesel types; nullability; table and column order; primary keys;
generated joins; and allow-table membership. It must not include ordered enum
labels.

#### 6.5.4 Canonical-schema comparison

Require exact equality between the baseline-to-candidate
convention-adjusted canonical delta and the manifest projected to
canonical-schema-representable fields. This comparison includes SQL-type
identity and derives, canonical aliases, canonical Diesel column types,
nullability, canonical ordering, primary keys, joins, allow-table membership,
and supplemental convention-controlled structures. It must not claim to encode
PostgreSQL enum labels.

#### 6.5.5 Shared-field cross-checks and complete intent

Every fact represented by more than one independent leg must agree exactly
after its documented deterministic mapping. Catalog SQL-type identity must
agree with raw, canonical, and manifest identity; catalog PostgreSQL column
types must map to the manifest's canonical Diesel type and agree with raw and
canonical output; catalog foreign keys must map to the expected raw and
canonical joins; and raw and canonical allow-table membership must agree with
the manifest.

The manifest remains the complete declared intent. A fact that is not
representable in raw or canonical Diesel must still be validated wherever it is
independently representable. No unrepresented field may be silently ignored.
Manifest entries do not override database truth and cannot exempt unrelated
differences. A new table requires explicit complete objects for the table,
every column, its primary key, every generated join, and its
`allow_tables_to_appear_in_same_query!` membership; none of those structural
effects may be inferred or hidden by another entry.

#### 6.5.6 Enum-specific behaviour

For a new enum, the type must be absent from the baseline catalog and exactly
present in the candidate catalog with the manifest's schema, physical type
name, and ordered labels. The baseline-to-candidate raw Diesel and canonical
schema deltas must contain the new SQL-type identity and canonical Diesel type
name declared by the manifest; raw and canonical legs are not expected to
contain labels. A baseline type that is already present or a missing raw or
canonical SQL-type declaration is a failure.

For an enum label addition, insertion, rename, or order change that does not
alter SQL-type identity, the required result is:

```text
baseline catalog:
old complete ordered labels

candidate catalog:
new complete ordered labels

manifest before:
exactly old complete ordered labels

manifest after:
exactly new complete ordered labels

baseline-to-candidate raw Diesel delta:
empty

baseline-to-candidate canonical delta:
empty
```

This is valid only when all applicable comparisons match. The canonical
`thoth-api/src/schema.rs` must remain byte-identical for a label-only change
unless another independently representable schema-contract change is also
declared.

An empty raw or canonical delta is acceptable only when the manifest projection
for that representation is also empty. It must not permit an undeclared catalog
enum-label change, a wrong label or label order, a missing new SQL-type
identity, an unexpected raw or canonical change, or a broad manifest exemption.

Indexes, check constraints, and other structures not represented by Diesel's
schema contract remain responsibilities of migration validation. This manifest
must not claim or imply that it validates those structures.

The implementation task's controlled probe must create only this standalone
table on its disposable database:

```sql
CREATE TABLE public.thoth_db_ctrl_probe (
    probe_id uuid PRIMARY KEY,
    probe_value text
);
```

Its expected manifest is exactly the five `[[add]]` objects above. The probe has
no consuming Rust model and must create no join. It must be dropped after
testing and must not create or mention a BE-01 database object in generated
output.

### 6.6 Canonical commands

After exporting a disposable `DATABASE_URL`, setting
`THOTH_DIESEL_CONFIRM_DATABASE` to its exact database name, setting
`THOTH_DIESEL_BASE_REF` to the task's authorized full base SHA, and setting
`DIESEL_BIN` to exact CLI `2.3.10`, the check command is:

```bash
make check-diesel-schema
```

It must run the complete baseline-to-candidate acquisition and check twice from
fresh proven disposable state, compare the two temporary candidates with
`cmp`, compare the candidate with `thoth-api/src/schema.rs`, verify `git status
--short` is unchanged, and exit zero only with:

```text
THOTH_DIESEL_TARGET=SAFE_DISPOSABLE_LOCAL
THOTH_DIESEL_CLI=2.3.10
THOTH_DIESEL_BASE_REF=VERIFIED_FULL_AUTHORIZED_ANCESTOR_SHA
THOTH_DIESEL_BASELINE=CAPTURED_INDEPENDENTLY
THOTH_DIESEL_CANDIDATE=CAPTURED_INDEPENDENTLY
THOTH_DIESEL_CONFIG=diesel.toml
THOTH_DIESEL_SCHEMA=thoth-api/src/schema.rs
THOTH_DIESEL_CLIENT_ENDPOINT=LOOPBACK
THOTH_DIESEL_SERVER_ADDRESS=LOOPBACK_OR_VERIFIED_PRIVATE_CONTAINER
THOTH_DIESEL_EXPECTED_PROJECTION=CHANGE_OR_NONE
THOTH_DIESEL_DELTA=EXACT_PROJECTED_MATCH
THOTH_DIESEL_CLEANUP=COMPLETE
THOTH_DIESEL_REPEAT=IDENTICAL
THOTH_DIESEL_DIFF=CLEAN
```

The controlled generation command is:

```bash
python3 .github/scripts/diesel_schema.py generate \
  --base-ref "$THOTH_DIESEL_BASE_REF" \
  --expected-change "$THOTH_DIESEL_EXPECTED_CHANGE_FILE" \
  --output thoth-api/src/schema.rs
```

It is the sole canonical writer and must atomically replace only the canonical
schema after both snapshots, exact projected comparisons, compilation, and
cleanup pass. It must print only safe named statuses, the validated projection
mode, and aggregate added/removed/changed counts. In `none` mode, no manifest
operations are permitted and empty controlled projections must leave the
canonical schema byte-identical even when candidate migrations are pending. In
`change` mode, an all-empty controlled result must not pass.

### 6.7 Required success behaviour

A clean baseline-to-candidate run on a proven disposable database must produce
deterministic snapshots and a deterministic candidate. Repeating the complete
two-phase run must be identical. In `none` mode, when no candidate migration
changes the controlled projection and the manifest has no operations, check
mode must report no structural or textual difference and must not change the
worktree. The result must state that excluded migration effects were not
validated by the Diesel projection control.

The controlled standalone disposable probe table must produce exactly the five
manifest-approved structural additions: its table, two columns, primary key,
and `allow_tables_to_appear_in_same_query!` membership. Repeating generation
must reproduce the same candidate. The schema-only candidate, with no
application-model fixture, must pass
`cargo check -p thoth-api --features backend` when substituted only in a
temporary worktree. Dropping the probe table and using a `none`-mode manifest
with no operations must restore the byte-for-byte clean baseline.

Every applicable independently observed baseline-to-candidate catalog,
raw-Diesel, and canonical-schema projection must match the corresponding
manifest projection exactly. A label-only enum change must succeed only when
the baseline catalog equals `change.before`, the candidate catalog equals
`change.after`, the raw and canonical deltas are empty, and the canonical
schema remains byte-identical.

### 6.8 Required failure behaviour

The procedure must fail non-zero before repository writes if:

- the database target is not approved, local, and disposable;
- `THOTH_DIESEL_BASE_REF` is absent, abbreviated, symbolic, not the exact
  authorized base SHA, not an ancestor of candidate `HEAD`, or moves during the
  run;
- candidate `HEAD` is not the exact authorized candidate SHA or moves during
  the run;
- the detached baseline worktree is not exact, clean, isolated, or removable;
- migrations do not apply;
- the baseline database is not empty before baseline migration application;
- the baseline migration ledger is not an exact prefix of the candidate
  migration ledger;
- either baseline or candidate catalog, raw-Diesel, or canonical snapshot is
  absent, derived from the manifest, derived from the other phase, or changes
  after capture;
- configuration does not parse;
- the configured or requested path is wrong;
- Diesel CLI is absent or not exact `2.3.10`;
- raw output or a repeated candidate is nondeterministic;
- the catalog delta differs from the manifest's catalog projection;
- the raw Diesel delta differs from the manifest's raw-Diesel projection;
- the convention-adjusted canonical delta differs from the manifest's
  canonical-schema projection;
- shared fields disagree after their documented deterministic mapping;
- one representation is augmented with facts copied from another before
  comparison;
- a declared fact is silently ignored instead of being checked in every
  independent representation that can expose it;
- enum labels or label order differ between catalog and manifest;
- a manifest `change.before` differs from the independently observed baseline
  object even when `change.after` matches the candidate;
- an `add` object exists at baseline, a `remove` object is absent at baseline or
  present in the candidate, or an undeclared removal, rename, or reorder occurs;
- a new enum is missing its raw or canonical SQL-type identity;
- a label-only enum change produces an unexpected raw or canonical delta;
- `expected_projection` is missing, duplicated, unknown, or case-variant;
- `expected_projection = "change"` produces empty catalog, raw-Diesel, and
  canonical baseline-to-candidate projections;
- `expected_projection = "none"` contains any manifest `add`, `remove`, or
  `change` object;
- `expected_projection = "none"` produces any catalog, raw-Diesel, or canonical
  baseline-to-candidate difference or changes canonical schema bytes;
- automatic raw output does not match the manifest and conventions;
- a convention entry is missing, unused, broad, or conflicting;
- required custom derives or supplemental types are lost;
- a candidate fails the focused Rust compile check;
- an unrelated repository file changes;
- a resolved path leaves the repository;
- the working directory is not the repository root;
- shell metacharacters alter argument boundaries or any subprocess uses a shell;
- baseline worktree, database state, private snapshots, generated artifacts,
  container, or disposable storage cannot be removed.

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

Automatic Diesel staging is not a compatible canonical contract and must never
be consumed by Rust code, downstream generators, or review tooling as though it
were `thoth-api/src/schema.rs`.

### 6.12 Authoritative AGENTS rollout

The implementation must update root `AGENTS.md` to replace the stale statement
that the Diesel procedure has an open control gap with authoritative
instructions covering the repository-root working directory, exact Diesel CLI
`2.3.10`, automatic staging at `target/diesel-schema.rs`, synchronizer-only
canonical writes, `make check-diesel-schema`, complete expected-change
manifests with explicit `change` or `none` projection expectations, and
fail-closed behaviour. It must explicitly retain CG-13 as a separate open
control.

The implementation must replace the `thoth-api/AGENTS.md` "Diesel schema
control gap" section with the same implemented canonical procedure. It must
state that automatic `diesel.toml` output is staging only,
`thoth-api/src/schema.rs` is canonical, direct manual replacement is
prohibited, every schema task uses the synchronizer with a complete
expected-change manifest, validation and compile gates are mandatory, and
dependent schema work blocks if the control fails.

## 7. Data and migration requirements

Production migration required: NO
Schema change required: NO
Backfill required: NO
Production data effect: NONE

Disposable migration-chain validation is required:

1. verify the full authorized base SHA and create its detached temporary
   worktree;
2. start from an empty PostgreSQL 17 database and retain the safe-target proof
   for the full run;
3. from the base worktree, run
   `cargo run migrate --database-url "$DATABASE_URL"`;
4. verify the baseline migrations, tables, types, constraints, indexes, and
   migration ledger, then capture baseline catalog, raw-Diesel, and canonical
   snapshots;
5. from the candidate worktree, apply only pending candidate migrations to the
   same database;
6. capture candidate catalog, raw-Diesel, and canonical snapshots and compute
   the exact baseline-to-candidate projected deltas;
7. require the explicit `change` or `none` projection expectation and enforce
   its mode contract; for `change`, require complete manifest `add`, `remove`,
   `change.before`, and `change.after` objects to match the independently
   observed states exactly;
8. run no-op, index-only, check-constraint-only, data-only, controlled
   standalone-table, new-enum, and label-only verification, while validating
   excluded migration effects separately from the Diesel projection control;
9. separately run the existing full-chain revert on the proven disposable
   database, verify the application schema is empty, and reapply the full chain;
10. repeat the two-phase verification deterministically from fresh disposable
    state;
11. compile the candidate and permit canonical promotion only after every
    comparison and cleanup check succeeds;
12. remove the detached worktree, database state, snapshots, generated
    artifacts, container, and disposable storage on success or failure.

Full-chain revert is prohibited outside the uniquely named disposable target.
Populated-database migration proof remains a requirement of each future schema
task; this control implementation introduces no populated-data change.

## 8. Observability and operations

Required output consists only of the named safe statuses in section 6, the
validated projection mode, aggregate structural counts, and, for `none`, the
explicit excluded-effects disclaimer. CI must retain command exit status and
the safe status lines as evidence.

No runtime metrics, alerts, deployment, production activation, or operational
runbook change is required. CG-13 remains open.

## 9. Acceptance criteria

- [ ] The exact repository-root working directory and commands are documented.
- [ ] Migration creation is exactly `make migration`.
- [ ] Forward and disposable full-chain revert commands are exact.
- [ ] `THOTH_DIESEL_BASE_REF` is a full immutable SHA equal to the task's
      authorized base and is proven ancestral to the candidate head.
- [ ] A detached clean worktree at the exact base supplies baseline migrations
      and canonical bytes; it is removed on success and failure.
- [ ] Root `diesel.toml` parses and targets only ignored automatic staging at
      `target/diesel-schema.rs`; it never targets `thoth-api/src/schema.rs`.
- [ ] Direct Diesel CLI migration execution cannot change the canonical schema.
- [ ] Automatic Diesel output lands only at `target/diesel-schema.rs`.
- [ ] Changing the staging file cannot cause canonical promotion.
- [ ] Validated `generate` is the sole canonical writer.
- [ ] The canonical file remains byte-identical when any validation fails.
- [ ] Automatic raw output matching neither the manifest nor conventions is
      rejected rather than copied.
- [ ] Diesel CLI exact `2.3.10` and the isolated installation policy are enforced.
- [ ] The convention file explicitly accounts for every current intentional raw/canonical difference without catch-all patterns.
- [ ] A clean migrated PostgreSQL 17 database produces a byte-identical no-op.
- [ ] Two consecutive candidates are byte-identical.
- [ ] `MarkupFormat`, aliases, timestamp semantics, model-compatible column order, and required derives are preserved.
- [ ] A controlled standalone-table expected-diff test admits exactly its five
      complete version-2 objects, including types, nullability, ordinals,
      ordered primary-key columns, and allow-table membership.
- [ ] Every independently derived representation is compared exactly with the
      manifest projection for that representation, using the normative
      capability matrix and documented mappings.
- [ ] Baseline migrations are applied to an empty proven disposable database;
      only candidate pending migrations are then applied to the same database.
- [ ] Baseline and candidate catalog, raw-Diesel, and canonical snapshots are
      independently derived and the actual baseline-to-candidate deltas are
      computed before manifest comparison.
- [ ] Manifest `add`, `remove`, `change.before`, and `change.after` semantics
      exactly enforce absence/presence and independently observed complete
      before/after objects.
- [ ] Ordered PostgreSQL enum labels are compared exactly between independently
      observed catalog data and complete manifest intent, never injected into
      the raw or canonical Diesel legs.
- [ ] A new enum requires catalog/manifest label equality plus matching raw and
      canonical SQL-type identity and canonical Diesel type name.
- [ ] A label-only enum change accepts matching non-empty catalog/manifest
      before/after label deltas with empty raw/canonical projections and a
      byte-identical canonical schema.
- [ ] Every manifest declares exactly one valid `expected_projection` mode.
- [ ] `change` mode requires at least one non-empty independently observed
      controlled projection and exact manifest equality for every difference.
- [ ] `none` mode permits pending migrations only with no manifest operations,
      empty catalog/raw/canonical projections, and byte-identical canonical
      schema; any hidden controlled change fails.
- [ ] A `none` result explicitly disclaims validation or approval of indexes,
      constraints, data changes, comments, and other excluded migration effects.
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
- [ ] Baseline and candidate phases retain the same safe-target proof, and all
      worktrees, snapshots, artifacts, database state, containers, and
      disposable storage are removed.
- [ ] The standalone-table schema candidate passes the focused backend compile
      check without any application-model fixture.
- [ ] Rollback restores repository state without touching database data.
- [ ] BE-01 remains blocked and no BE-01 implementation object is created.
- [ ] Root `AGENTS.md` no longer describes CG-12 as an open procedural gap after
      implementation.
- [ ] `thoth-api/AGENTS.md` no longer describes an unexplained configured output
      path after implementation.
- [ ] Root `AGENTS.md` and `thoth-api/AGENTS.md` specify the same repository-root
      commands, exact CLI, automatic staging output, synchronizer-only canonical
      writes, complete expected-change manifests, explicit `change`/`none`
      projection contract, and fail-closed boundary.
- [ ] `docs/engineering/AGENTS.md` contains no contradictory instruction.
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
- Validate version-2 expected-change objects, including complete add/remove and
  before/after change records; reject missing fields and broad or catch-all
  entries.
- Require exactly one `expected_projection` value of `change` or `none`; reject
  missing, duplicate, unknown, and case-variant modes.
- Require a non-empty catalog, raw-Diesel, or canonical projection in `change`
  mode, and accept a label-only enum change because its catalog projection is
  non-empty.
- In `none` mode, reject any manifest operation or any independently observed
  table, column, key, join, allow-table, SQL-type, or enum difference.
- Enforce exact operation semantics: `add` is absent/present, `remove` is
  present/absent, and `change.before`/`change.after` equal independently
  observed baseline/candidate objects.
- Reject a missing, abbreviated, uppercase, symbolic, wrong, non-ancestral, or
  moving `THOTH_DIESEL_BASE_REF`.
- Reject `integer` where `uuid` was expected, nullable `probe_id`, non-null
  `probe_value`, reversed or additional columns, and the wrong primary-key
  column.
- Reject an additional or missing join, wrong join child or parent columns,
  wrong enum labels or label order, and incomplete SQL-type definitions.
- For a new enum, accept correct ordered labels and require raw and canonical
  SQL-type identity additions; reject a missing raw SQL-type identity and a
  wrong canonical Diesel type name.
- For label-only enum changes, accept an appended label and an insertion before
  or after an existing label only with the correct resulting order; reject a
  false `before` label list, false baseline label order, wrong `after` value or
  order, omitted catalog label change, manifest-only label change, and
  unexpected raw or canonical delta.
- Prove raw Diesel facts are parsed only from raw Diesel output, catalog enum
  labels only from PostgreSQL catalog data, and canonical facts only from the
  canonical candidate. Prove catalog labels are never injected into raw or
  canonical representations before comparison.
- Invoke subprocesses with argument arrays and `shell=False`; prove literal
  metacharacters cannot become shell syntax.

### Integration/database

- Apply the clean empty-database migration chain on PostgreSQL 17.
- Revert the full chain on that disposable database and reapply it.
- Create the exact detached base worktree, apply its migrations to an empty
  disposable database, capture independent baseline snapshots, apply only
  candidate pending migrations to the same database, and capture candidate
  snapshots.
- Run no-op generation twice and require `cmp` success.
- Preserve custom SQL types, derives, aliases, overrides, and order.
- Create only `public.thoth_db_ctrl_probe(probe_id uuid PRIMARY KEY,
  probe_value text)`, require exactly the five expected table, column,
  primary-key, and allow-table additions, and prove no join is generated.
- Compile the schema-only standalone-table candidate successfully without a
  consuming model, repeat generation deterministically, drop the table, and
  restore the byte-identical clean baseline with a `none`-mode manifest that
  contains no operations.
- Reject a stale canonical schema.
- Reject an unlisted database change.
- Reject an unexpected output path or unrelated changed file.
- Run direct Diesel migration commands and prove they can write only automatic
  staging, never `thoth-api/src/schema.rs`.
- Change `target/diesel-schema.rs` and prove it cannot trigger canonical
  promotion.
- Force each validation failure and prove the canonical bytes remain unchanged.
- Reject automatic raw output that matches neither the complete manifest nor
  conventions instead of copying it.
- Add a new enum and require exact catalog/manifest labels plus matching raw and
  canonical SQL-type identities; prove it is absent at baseline and exactly
  present in the candidate.
- Append and insert enum labels and require exact catalog order while the raw
  and canonical projections remain empty and the canonical schema remains
  byte-identical.
- Reject a false `before` label list even when `after` matches, false baseline
  label order, pre-existing baseline label mismatch, wrong or omitted candidate
  labels, undeclared removal/rename/reorder, manifest-only label changes,
  missing new SQL-type identities, and unexpected raw or canonical deltas.
- Pass index-only, check-constraint-only, and data-only migrations in `none`
  mode when every controlled projection is empty and canonical bytes are
  identical; separately validate each migration's excluded effect.
- Reject any hidden table, column, key, join, allow-table, SQL-type, or enum
  change in `none` mode, and reject any non-empty manifest in `none` mode.
- Reject empty controlled projections in `change` mode, while accepting a
  label-only enum change whose catalog projection is non-empty.
- Prove a migration-ledger advance alone does not imply a Diesel-controlled
  change and that a `none` result does not claim excluded effects were validated.
- Prove the base worktree, database state, private snapshots, generated
  artifacts, container, and disposable storage are removed after success and
  every forced failure.

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
- Reject any target or connection-identity change between baseline and
  candidate phases.
- Verify safe output contains no URL, credential, table content, or personal
  data.

### Regression

- `cargo check -p thoth-api --features backend` with both the no-op candidate
  and the standalone-table schema candidate, without an application-model
  fixture.
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
2. supplies the exact authorized full base SHA through
   `THOTH_DIESEL_BASE_REF`; for pull requests, requires it to equal the trusted
   `github.event.pull_request.base.sha`, checks out and verifies the exact
   `github.event.pull_request.head.sha` rather than a synthetic merge ref,
   verifies equality and ancestry, and creates a detached temporary base
   worktree;
3. proves the workflow-controlled `localhost:5432` service is empty and safe;
4. applies baseline migrations from the base worktree and captures independent
   baseline catalog, raw-Diesel, and canonical snapshots;
5. applies only candidate pending migrations from the candidate checkout to the
   same service and captures independent candidate snapshots;
6. computes the actual baseline-to-candidate deltas and runs
   `make check-diesel-schema` with the expected CI database confirmation,
   explicit `change` or `none` projection mode, and manifest projections;
7. runs the existing full-chain revert, verifies the application schema is
   empty, reapplies migrations, and repeats the two-phase check from fresh
   disposable state;
8. records safe aggregate base-ref, snapshot, delta, cleanup, server/client
   address, port, and GitHub Actions provenance statuses;
9. removes the temporary worktree and every snapshot or generated artifact;
10. fails on any non-zero control, provenance, projection-mode, exact-delta, or
    cleanup result, including all-empty `change` mode or any controlled
    difference in `none` mode.

The workflow must retain explicit job-level `contents: read`, use no production
environment or secret, and preserve the existing protected check identity. The
CI classifier must treat the `.github/scripts/` synchronizer and tests,
convention file, config, Makefile, canonical schema, authoritative AGENTS files,
and migration workflow as migration-control paths so those changes cannot be
misclassified as documentation-only.

## 12. Rollout

- initial state after merge: inactive repository-control improvement with
  automatic Diesel output confined to ignored staging and canonical writes
  confined to the synchronizer;
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
  previous config, commands, tracked `.github/scripts/` tools, convention data,
  authoritative AGENTS instructions, workflow, and docs;
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

Exactly thirteen paths are expected. Root `.gitignore` is already sufficient
because automatic output is under ignored `target/`; `.gitignore` is not in
scope.

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

Specification: DRAFT
Owner: CTO
Approved by: Not approved for the projection-mode-corrected content
Previous specification approval: historical, bound to pre-correction content
Corrected specification: DRAFT
THOTH-DB-CTRL-01 implementation: NOT STARTED
Implementation branch: NOT AUTHORIZED

Notes:

- The previous approval at pre-correction content is historical after the
  normative enum-projection correction.
- The corrected specification requires fresh independent exact-head review and
  fresh explicit CTO specification approval before it may return to
  `APPROVED`.
- Specification approval still does not authorize creation of
  `feature/repository-controls/thoth-db-ctrl-01`, implementation work, migration
  execution, schema or workflow changes, production access, release,
  deployment, or activation.
- The implementation requires a separately authorized fresh branch from the
  then-current verified `develop`, complete acceptance evidence, independent
  cross-model review, and explicit CTO merge authorization.
- BE-01 remains blocked, CG-12 remains unresolved until the implementation
  merges with acceptance evidence, and CG-13 remains open.

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

Rejected because root configuration discovery is established, ignored root
`target/` staging keeps automatic Diesel writes away from the compiled canonical
contract, and relocating the canonical schema is outside scope.

### Diesel upgrade or downgrade

Rejected because exact `2.3.10` is already locked by the repository and
generated deterministically. Version change does not solve model-specific
ordering and supplemental-type requirements and needs separate approval.

### Candidate-final catalog comparison without a baseline

Rejected because a final catalog can prove only the candidate `after` state.
It cannot independently prove manifest `change.before`, actual baseline label
order, absence for an `add`, presence for a `remove`, or the claimed
baseline-to-candidate delta. The authoritative comparison therefore requires
the two-phase baseline and candidate snapshots defined in section 6.5.1.
