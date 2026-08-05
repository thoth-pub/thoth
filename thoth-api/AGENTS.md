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

## 4. Repository-authoritative schema contract

Per [ADR-0003](../docs/engineering/decisions/ADR-0003-repository-authoritative-schema-contract.md), `thoth-api/src/schema.rs` is the repository-authoritative, manually maintained Rust/Diesel compile-time schema contract. It is not regenerated through a Diesel CLI workflow, and there is no root `diesel.toml`.

Work from the repository root. Create migrations with `make migration` and apply or revert them with the embedded runner (`cargo run migrate`, `cargo run migrate --revert`).

Every schema-changing task must update, atomically in the same bounded PR:

1. the migration `up.sql` and `down.sql` under `migrations/`;
2. `thoth-api/src/schema.rs`, edited directly to match the migration;
3. the affected Rust models and any query/GraphQL code;
4. focused database and model tests.

`thoth-api/src/schema.rs` may intentionally contain custom SQL types, supplemental types, physical-to-Rust aliases, model-compatible column ordering, timestamp mappings, and formatting that raw `diesel print-schema` does not reproduce. Preserve those conventions and introduce no unrelated schema reformatting.

When a migration has no `thoth-api/src/schema.rs` impact (for example a data-only, index-only, or check-constraint migration outside the checked-in Diesel table contract), record that explicitly in the task/implementation report as a reviewed conclusion.

Never use `diesel print-schema` as the canonical writer. External introspection tools may be consulted diagnostically, but their output is untrusted and must never write directly to `thoth-api/src/schema.rs`. Do not introduce a Diesel CLI dependency, a `diesel.toml`, or a schema-synchronization subsystem without a separately approved ADR that supersedes ADR-0003.

Production migration execution and rollback remain governed by CG-13 and separate release authorization.

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
