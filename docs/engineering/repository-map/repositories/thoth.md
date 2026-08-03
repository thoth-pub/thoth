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
- generated Diesel schema;
- migration ordering;
- locking/downtime.

## Generated artefacts

`thoth-api/src/schema.rs` is a generated/derived Diesel schema file.

The selected canonical procedure is specified by
[`THOTH-DB-CTRL-01`](../../ai-delivery/tasks/THOTH-DB-CTRL-01.md): run from the
repository root; retain root `diesel.toml`; use exact Diesel CLI `2.3.10`;
introspect only a proven disposable local PostgreSQL database; preserve
`thoth-api/src/schema.rs` as canonical; and pass raw Diesel output through a
fail-closed structural synchronizer that preserves explicit repository
conventions and admits only a declared expected change.

The written specification was previously approved at pre-correction content.
That approval is historical after the normative enum-projection correction.
The corrected specification is `DRAFT`, awaiting fresh independent exact-head
review and fresh explicit CTO specification approval. The implementation is not
started, and its branch is not authorized. Specification approval still would
not authorize implementation work, migration execution, schema or workflow
changes, production access, release, deployment, or activation. Until
`THOTH-DB-CTRL-01` receives separate implementation authorization and merges
after independent review with its acceptance evidence, do not regenerate,
overwrite, or relocate `thoth-api/src/schema.rs`. CG-12 remains unresolved,
CG-13 remains open, and dependent schema work, including BE-01, remains blocked
under
[CG-12](../control-gaps.md#cg-12---thoth-schema-generation-unclear).
Production migration, deployment, rollback, restore verification, and approver
mapping remain separately blocked on
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
