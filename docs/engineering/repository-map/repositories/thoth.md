# Repository: thoth-pub/thoth

## Responsibility

Canonical bibliographic and metrics domain, PostgreSQL model, GraphQL API, authorization, import validation, rollups, export API and database migrations.

## Branches

GitHub default/release: `master`
Development: `develop`
Normal task: `feature/... -> develop`
Release: `develop -> master`

This repository conforms to the approved target topology.

## Stack

- Rust 2021
- Cargo workspace
- PostgreSQL
- Diesel
- Redis
- GraphQL API
- REST/export server
- ZITADEL integration

Workspace members observed:

- `thoth-api`
- `thoth-api-server`
- `thoth-client`
- `thoth-errors`
- `thoth-export-server`

## Mandatory orientation

Before editing:

- `README.md`
- `Cargo.toml`
- `Makefile`
- relevant crate `Cargo.toml`
- `thoth-api/migrations/`
- `thoth-api/src/schema.rs`
- relevant authorization/policy modules
- `.github/workflows/build_test_and_check.yml`
- `.github/workflows/run_migrations.yml`
- `CHANGELOG.md`
- applicable ADR/task specification

## Commands

```bash
make build
make test
make check
make clippy
make check-format
make check-all
```

Underlying required checks:

```bash
cargo test --workspace
cargo check --workspace
cargo clippy --all --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

CI uses PostgreSQL 17 and Redis.

## Migration controls

Migration creation target:

```bash
make migration
```

Observed convention:

```text
thoth-api/migrations/YYYYMMDD_v<next-minor>/
  up.sql
  down.sql
```

Every schema task must verify:

- empty database;
- populated database;
- constraints and indexes;
- forward migration;
- downgrade or forward-repair strategy;
- the checked-in Diesel schema contract (`thoth-api/src/schema.rs`), edited
  atomically with the migration;
- migration ordering;
- locking/downtime.

## Schema contract

`thoth-api/src/schema.rs` is the repository-authoritative, manually maintained
Rust/Diesel compile-time schema contract, per
[ADR-0003](../../decisions/ADR-0003-repository-authoritative-schema-contract.md).
It is not regenerated through a Diesel CLI workflow, and there is no root
`diesel.toml`.

The canonical workflow (Architecture A): run from the repository root; create
migrations with `make migration`; apply and revert them with the embedded runner
(`cargo run migrate`, `cargo run migrate --revert`); and edit
`thoth-api/src/schema.rs` directly so that migrations, `schema.rs`, affected
models, and database-backed tests change atomically in one bounded PR. A
migration with no `schema.rs` impact must record that as an explicit reviewed
conclusion. `diesel print-schema` must never be the canonical writer, and a
Diesel CLI dependency must not be reintroduced without a separately approved ADR
that supersedes ADR-0003.

ADR-0003 supersedes the `THOTH-DB-CTRL-01` structural-synchronizer approach,
whose implementation PR
[#777](https://github.com/thoth-pub/thoth/pull/777) was closed unmerged.
[`THOTH-DB-CTRL-02`](../../ai-delivery/tasks/THOTH-DB-CTRL-02.md) delivers
ADR-0003 and its directly related cleanup through PR
[#778](https://github.com/thoth-pub/thoth/pull/778).

[CG-12](../control-gaps.md#cg-12---thoth-schema-generation-resolved-via-architecture-a)
is resolved by the merged Architecture A control (ADR-0003), delivered by
`THOTH-DB-CTRL-02` through PR #778; this record becomes authoritative when the
change merges into `develop`, and the merge itself remains subject to
independent review and explicit CTO merge authorization. On merge, BE-01 is
`READY` for separately authorized implementation — `READY` does not authorize
implementation by itself: creating the branch and making any implementation edit
require separate explicit authorization, and the branch remains absent until
then. Production migration, deployment, rollback, restore verification, and
approver mapping remain separately blocked on
[CG-13](../control-gaps.md#cg-13---thoth-runtime-operations-unmapped).

## CI and release

PR checks include:

- build;
- workspace tests;
- clippy;
- formatting;
- migrations;
- changelog check.

Every PR requires a `CHANGELOG.md` change under `## [Unreleased]`.

Published GitHub releases build and publish:

```text
ghcr.io/thoth-pub/thoth
```

The production runtime and deployment/rollback process require separate verification.

## Contract relationships

This repository owns the canonical PostgreSQL domain, migrations, GraphQL API
and export formats consumed by `thoth-app`, `thoth-pyramid`,
`thoth-dissemination` and the standalone `thoth-client`. See
`docs/engineering/repository-map/contracts.md` for verified consumers and
required compatibility handling.

The `thoth-client` workspace member listed above is an **internal** Rust
crate, depended on only by `thoth-export-server` within this same workspace.
It is not the same project as the standalone, separately published
`thoth-pub/thoth-client` Python repository documented in
`repositories/thoth-client.md`. See `contracts.md` section 1.

## Programme effects

Publisher Services:

- package and distribution configuration;
- audit history;
- durable jobs;
- worker API;
- OCLC feed index;
- licence enforcement.

Metrics:

- canonical metric tables;
- ingestion;
- rollups;
- entitlements;
- protected metrics GraphQL;
- OPERAS ledgers.

## Prohibited assumptions

- Do not write directly to production PostgreSQL.
- Do not allow Sphinx to bypass the GraphQL write boundary.
- Do not broaden authorization on API failure.
- Do not merge database, API and unrelated refactors in one slice.
