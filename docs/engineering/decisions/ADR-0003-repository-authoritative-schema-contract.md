# ADR-0003 - Repository-authoritative Diesel schema contract

Status: APPROVED
Date: 2026-08-05
Decision owner: CTO
Programmes affected: Shared Repository Controls, Publisher Services, Thoth Metrics
Repositories affected: `thoth`
Supersedes: None
Superseded by: None

## 1. Context

Thoth stores its canonical relational model in PostgreSQL. Database evolution is
expressed as ordered migrations under `thoth-api/migrations/`, applied by an
embedded Diesel migration runner invoked through `cargo run migrate`. The
compile-time Rust representation of the database — the table and column contract
that Diesel query building and the Rust models depend on — is the checked-in
file `thoth-api/src/schema.rs`.

The repository also contained a root `diesel.toml` with a `[print_schema]`
section. That configuration was never part of any supported build, test,
migration, or schema-generation command. Disposable-database investigation
recorded under CG-12 established that the file did not parse (missing commas in
`custom_type_derives`), that its `file = "src/schema.rs"` output path did not
even identify the canonical `thoth-api/src/schema.rs`, and that raw
`diesel print-schema` output does not reproduce the checked-in contract's custom
SQL types, supplemental types, physical-to-Rust aliases, model-compatible column
ordering, timestamp mappings, or formatting.

CG-12 ("Thoth schema generation unclear") asked how `schema.rs` should be kept
in step with migrations. Task `THOTH-DB-CTRL-01` proposed answering that with a
bespoke subsystem: PostgreSQL catalog introspection, raw `diesel print-schema`,
a `thoth-api/diesel-schema-control.toml` convention file, and a custom
structural synchronizer that reconciled raw output against the canonical
contract under `make check-diesel-schema`. Its implementation, PR
[#777](https://github.com/thoth-pub/thoth/pull/777), was closed unmerged. No
code from PR #777 became repository-authoritative.

This ADR records the architecture the CTO selected on 2026-08-05 to replace that
approach and to unblock normal schema-bearing tasks such as BE-01.

## 2. Decision drivers

- Match established repository practice, where `schema.rs` is already maintained
  in reviewed source control alongside migrations and models.
- Avoid a permanent schema-reconciliation subsystem and the multiple competing
  schema representations it would create.
- Keep schema changes reviewable together with the migrations and models that
  motivate them.
- Avoid making incompatible raw Diesel CLI output authoritative over a
  hand-maintained contract that intentionally diverges from it.
- Remove a misleading, unused, non-parsing configuration file.
- Do not require the Diesel CLI to build, test, run, or migrate Thoth.

## 3. Options considered

### Option A - Repository-authoritative, manually maintained `schema.rs`

`thoth-api/src/schema.rs` remains the repository-authoritative Rust/Diesel
compile-time schema contract, maintained directly in reviewed source control.
Migrations, `schema.rs`, affected Rust models, and database-backed tests are
changed atomically in the same bounded task. The Diesel CLI and root
`diesel.toml` are not part of the supported build, migration, or
schema-generation workflow.

Advantages:

- matches established repository practice;
- avoids a bespoke schema-reconciliation subsystem;
- keeps changes reviewable with their migrations and models;
- avoids making incompatible raw CLI output authoritative;
- removes a misleading unused configuration;
- unblocks normal schema-bearing tasks such as BE-01.

Disadvantages:

- developers must update `schema.rs` deliberately;
- drift prevention depends on atomic PR discipline, compilation, database tests,
  and review rather than regeneration;
- migrations that do not affect `schema.rs` require an explicit explanation.

Decision: Selected.

### Option B - Generated-schema authority via a structural synchronizer (the PR #777 architecture)

Retain root `diesel.toml`, capture raw `diesel print-schema` output, describe
intentional conventions in `thoth-api/diesel-schema-control.toml`, and reconcile
the two with a custom fail-closed synchronizer enforced in CI.

Advantages:

- promises mechanical drift detection.

Disadvantages:

- creates multiple schema representations (raw catalog output, convention data,
  canonical contract) that must be permanently reconciled;
- introduces a bespoke synchronizer subsystem and a pinned `diesel_cli`
  dependency with no evidence the application requires the Diesel CLI;
- makes raw CLI output — which cannot reproduce the checked-in contract —
  structurally authoritative;
- larger, harder-to-review surface than the change it guards.

Decision: Rejected. Reference PR
[#777](https://github.com/thoth-pub/thoth/pull/777) as closed and unmerged. Its
implementation must not be reproduced.

## 4. Decision

### 4.1 Database authority

- The ordered migrations under `thoth-api/migrations/` define database
  evolution.
- The embedded Rust migration runner remains the supported way to apply and
  revert migrations.
- Existing commands based on `cargo run migrate` (and `cargo run migrate
  --revert`) remain authoritative.
- The Diesel CLI is not required to run, build, test, or migrate Thoth.

### 4.2 Rust schema authority

- `thoth-api/src/schema.rs` is the repository-authoritative Rust/Diesel
  compile-time schema contract.
- It is maintained directly in reviewed source control.
- It may intentionally contain:
  - custom SQL types;
  - supplemental types;
  - physical-to-Rust aliases;
  - model-compatible column ordering;
  - timestamp mappings;
  - formatting and conventions not reproduced by raw `diesel print-schema`.

### 4.3 Atomic schema-bearing changes

Any task that changes the Diesel-representable database contract must update
atomically, in the same bounded PR:

- migration `up.sql` and `down.sql`;
- `thoth-api/src/schema.rs`;
- affected Rust models;
- affected query or GraphQL code where applicable;
- focused database and model tests;
- migration, rollback, compatibility, rollout, and rollback evidence required by
  the task risk.

A migration that does not require a `schema.rs` change must state why. Examples
may include:

- a data-only migration;
- an index-only migration;
- a check constraint not represented in `schema.rs`;
- another PostgreSQL construct outside the checked-in Diesel table contract.

Absence of a `schema.rs` edit must be an explicit reviewed conclusion, not an
omission.

### 4.4 Verification model

Verification is based on the combined evidence from:

- compiler compatibility;
- database-backed tests;
- migration apply/revert/reapply;
- model and query tests;
- exact review of migrations, `schema.rs`, models, and operational effects.

No generated file or broad textual patch may silently override the checked-in
contract.

### 4.5 Diesel CLI status

- Root `diesel.toml` is retired and removed.
- `diesel print-schema` is not a supported canonical generation command.
- No CI job or Make target should install or require `diesel_cli`.
- An engineer may use external introspection tools diagnostically, but their
  output is untrusted and must never write directly to the canonical
  `schema.rs`.
- Removing `diesel.toml` does not remove or deprecate the Diesel Rust crates
  used by the application. The Diesel library and the embedded
  `diesel_migrations` runner remain in use.

## 5. Consequences

### Positive

- Established repository practice is confirmed rather than replaced.
- No bespoke schema-reconciliation subsystem is created or maintained.
- Schema changes stay reviewable with their migrations and models.
- Incompatible raw CLI output is never made authoritative.
- A misleading, unused, non-parsing configuration file is removed.
- Normal schema-bearing tasks such as BE-01 are unblocked.

### Negative

- Developers must update `schema.rs` deliberately.
- Drift prevention depends on atomic PR discipline, compilation, database tests,
  and review rather than regeneration.
- Migrations that do not affect `schema.rs` require an explicit explanation.

### Risks

- A developer could forget to update `schema.rs`; compilation and database tests
  are the primary guards, backed by review.
- A future engineer could reintroduce a Diesel CLI dependency; doing so requires
  a separate ADR that supersedes this one.

## 6. Invariants created by this decision

1. `thoth-api/src/schema.rs` is the repository-authoritative Rust schema
   contract, maintained directly in reviewed source control.
2. Ordered migrations under `thoth-api/migrations/` and the embedded
   `cargo run migrate` runner remain the supported database-evolution workflow.
3. Schema-bearing changes update migrations, `schema.rs`, models, and tests
   atomically in one bounded PR.
4. A migration without a `schema.rs` edit records an explicit reviewed reason.
5. The Diesel CLI, `diesel print-schema`, and a root `diesel.toml` are not part
   of the supported build, test, migration, or schema-generation workflow.
6. Reintroducing generated-schema authority or a Diesel CLI dependency requires a
   new ADR that supersedes this one.

## 7. Implementation impact

- Root `diesel.toml` is deleted.
- `AGENTS.md` and `thoth-api/AGENTS.md` describe the manual atomic workflow and
  stop implying that a structural synchronizer is required.
- `.github/workflows/run_migrations.yml` verifies that the full migration chain
  applies, reverts, and reapplies on a disposable database. It introduces no raw
  schema generation, catalog comparison, expected-change manifest, or new Python
  subsystem.
- `thoth-api/src/schema.rs`, the files under `thoth-api/migrations/`, Rust
  models, runtime code, and Cargo dependency declarations are unchanged by the
  control that records this decision.
- `THOTH-DB-CTRL-01` is superseded; its implementation PR #777 is closed
  unmerged; BE-01 no longer depends on the rejected generator and may edit
  `schema.rs` directly in its own future bounded task.

## 8. Migration, rollout and rollback effects

Rollout:

- The control that records this decision changes repository instructions and CI
  only. No runtime feature activates, no database migration is introduced, and
  no production service changes.
- After merge, future schema-bearing tasks follow this decision.

Rollback:

- Revert the complete merge commit if the decision or cleanup must be withdrawn.
- Restoring `diesel.toml` alone is not a valid rollback: it would recreate the
  misleading, unsupported configuration without restoring any supported
  workflow.
- Any future move to generated-schema authority requires a separate ADR and
  implementation task.

## 9. Validation

Required evidence for the recording control:

- root `diesel.toml` is deleted and no replacement Diesel CLI configuration,
  convention file, or expected-change manifest is added;
- `thoth-api/src/schema.rs` and the files under `thoth-api/migrations/` are
  byte-identical to the base;
- Rust models, runtime code, and Cargo dependency declarations are unchanged;
- the migration workflow applies, reverts, and reapplies the full chain on a
  disposable PostgreSQL database;
- repository search shows no active instruction requiring the Diesel CLI.

## 10. Approval

Approval required from: CTO
Approved by: Javi, CTO
Approval date: 2026-08-05
Notes: Architecture A selected. `thoth-api/src/schema.rs` remains the
repository-authoritative, manually maintained schema contract; the Diesel CLI
and root `diesel.toml` are retired from the supported workflow. The CTO
explicitly authorized recording this decision and its directly related cleanup
in one draft pull request. This authorization does not authorize merge,
deployment, release, production migration, or the BE-01 implementation.
