# AGENTS.md - `thoth-api`

This file extends the repository-root `AGENTS.md`.

It applies to the canonical domain, PostgreSQL model, GraphQL schema/resolvers, validation, authorization and migrations in `thoth-api`.

## 1. Ownership boundary

`thoth-api` owns:

- domain models and identifiers;
- database access and schema mapping;
- migrations;
- validation and canonicalization;
- GraphQL types, inputs, queries and mutations;
- authorization policies;
- durable jobs, leases and audit records implemented in Thoth.

Do not place HTTP transport configuration or UI-specific behaviour here.

## 2. Required orientation

Before editing, inspect:

- the relevant model module under `src/model/`;
- `src/graphql/`;
- `src/policy.rs`;
- `src/db.rs`;
- related migrations under `migrations/`;
- `src/schema.rs`;
- existing tests for the same domain;
- relevant error variants in `thoth-errors`;
- the approved ADR and task specification.

Follow existing model/module patterns before creating new abstractions.

## 3. Database migrations

Create migration directories through the repository convention:

```bash
make migration
```

Expected shape:

```text
thoth-api/migrations/YYYYMMDD_v<version>/
├── up.sql
└── down.sql
```

A migration task must document:

- new or altered tables, enums, columns, constraints and indexes;
- existing-data treatment;
- default and nullability behaviour;
- expected table locking and runtime;
- forward migration;
- down migration or explicit forward-repair decision;
- retry/idempotency behaviour;
- deployment ordering.

Do not make a destructive or lossy migration without explicit CTO approval and a tested backup/restore or forward-repair plan.

Backfill existing data before adding constraints that existing rows cannot satisfy.

Avoid application-side uniqueness checks without a matching database constraint.

## 4. Diesel schema control (THOTH-DB-CTRL-01)

The Diesel schema-generation procedure is implemented and authoritative. All
commands run from the repository root.

- Automatic `diesel.toml` output is untrusted staging only, written to
  `target/diesel-schema.rs` (under the ignored `target/` directory). It is never
  the canonical contract.
- `thoth-api/src/schema.rs` is the canonical, compiled contract. Direct manual
  replacement, or any direct `diesel print-schema`/`diesel migration` write to
  it, is prohibited.
- Only the validated synchronizer `.github/scripts/diesel_schema.py generate`
  writes the canonical schema, and only after safety, exact projection,
  deterministic-repeat, focused-compile, and cleanup checks pass.
- The Diesel CLI must be exactly `2.3.10` (PostgreSQL feature), supplied through
  `DIESEL_BIN`.

Every schema-changing task must:

1. write a complete version-2 expected-change manifest with `expected_projection`
   explicitly `change` or `none`;
2. run `make check-diesel-schema` (mandatory) from the repository root, which runs
   the same synchronizer used in CI over a disposable PostgreSQL 17 database;
3. use `make generate-diesel-schema` (the synchronizer's `generate` mode) if the
   canonical schema must change, never a manual edit;
4. record the command and resulting diff in the implementation report;
5. introduce no unrelated schema reformatting.

The validation, exact-projection, deterministic-repeat, compile, and cleanup
gates are mandatory. A control failure emits
`BLOCKED - THOTH DIESEL GENERATION CONTROL FAILED` and blocks the dependent
schema work; it must not be weakened to obtain green CI. The convention data at
`thoth-api/diesel-schema-control.toml` enumerates every intentional
raw-Diesel/canonical difference (supplemental `MarkupFormat`, the
`abstract`/`title` table aliases, the `title.title` identifier handling, every
`Timestamp`->`Timestamptz` override, and model-compatible table/column order).

A `none` result certifies only the Diesel-controlled projection; excluded
migration effects (indexes, check constraints, data, comments) remain the
responsibility of migration validation. CG-13 remains a separate open control.

## 5. Transactions, concurrency and idempotency

Use database-enforced correctness for concurrent operations.

As applicable, use:

- unique constraints;
- foreign keys;
- check constraints;
- exclusion constraints;
- row locks;
- advisory locks;
- leases with expiry;
- claim tokens;
- `FOR UPDATE SKIP LOCKED`;
- deterministic idempotency keys.

Tests must cover concurrent first-arrival, stale claims and repeat execution where the feature can be invoked more than once.

Do not rely on GitHub Actions or a single worker process as a lock manager.

## 6. GraphQL design

Prefer additive schema changes.

For new lists and reports:

- paginate deterministically;
- bound date ranges and result sizes;
- avoid N+1 access;
- use set-based SQL or batched loaders;
- define OR/AND filter semantics explicitly;
- preserve stable enum/API codes;
- distinguish zero, absent and unknown where the domain requires it.

Do not call an external API from a resolver for data that Thoth owns canonically.

## 7. Authorization

Use the central policy machinery in `src/policy.rs` and model-specific policies.

Do not authorize only in the resolver body or frontend.

Every new protected query/mutation must have explicit tests for:

- no authentication;
- wrong role;
- wrong publisher scope;
- correct publisher scope;
- superuser or approved machine role.

Service roles must be least-privilege and distinct where read, ingest and synchronization have different powers.

An authentication/introspection failure must not broaden access.

## 8. Validation and write-path coverage

Canonical validation must apply to every route that can write the field:

- direct GraphQL create/update;
- imports;
- bulk mutations;
- administrative ingestion;
- background worker mutations;
- migration/backfill tools where appropriate.

Do not fix only one entry point.

Use canonical libraries such as `cc-license` rather than copying domain parsing rules.

## 9. Errors and auditability

Use stable, machine-readable error variants where clients or imports need classification.

Preserve enough context for diagnosis without storing secrets or unbounded raw input.

For imports and reconciliation, no row or state transition may disappear silently. Record accepted, duplicate, conflict, revised and rejected outcomes explicitly where required by the design.

## 10. Required checks

At minimum:

```bash
cargo test -p thoth-api --features backend
cargo check --workspace
cargo clippy --all --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

For broad or shared-domain changes, run:

```bash
cargo test --workspace
```

Database changes also require:

```bash
cargo run migrate
cargo run migrate --revert
```

against disposable local databases, plus the populated-database procedure specified by the task.
