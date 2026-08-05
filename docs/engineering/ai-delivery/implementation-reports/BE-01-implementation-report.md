# [BE-01] Implementation Report

## 1. Repository state

Repository: `thoth-pub/thoth`
Workflow: STANDARD
Base branch: `develop`
Base commit: `37b802776ae6853affe19d90156f3c1e0654ebe3` (merge commit of PR #778; verified equal to `origin/develop` and to the pre-existing remote branch head, with a clean working tree, before any implementation edit)
PR target: `develop`
Programme integration branch: None
Task branch: `feature/publisher-services/be-01`
Head commit: the exact final head after the documentation commit cannot truthfully be embedded in that same commit; it is recorded in the immutable post-push evidence comment on the pull request
Pull request: [#779](https://github.com/thoth-pub/thoth/pull/779) (draft)
Expected branch deletion after merge: YES
Final programme PR required: NO
Implementing model: Claude Fable 5 (`claude-fable-5`)
Reasoning level: HIGH

Authorization: explicit CTO authorization for the bounded BE-01 implementation, 2026-08-05T13:08:00+01:00. The authorization covers the existing branch, commits and pushes to it, a draft PR targeting `develop`, disposable database validation and this report. It does not cover merge, deployment, release, production or shared-database access, production migration execution, commercial backfill, feature activation, BE-02/BE-03/BE-04, OAI/Metrics/licensing/dissemination behaviour, related-repository changes, or issue #765/#766 edits.

Preconditions verified before any edit:

- PR #778 merged into `develop` through merge commit `37b80277…` (CG-12 resolved);
- ADR-0003 present with Architecture A authoritative;
- Publisher Services tracker recorded BE-01 as `READY`;
- no existing BE-01 implementation PR;
- baseline `thoth-api/src/schema.rs` git blob: `60b97dcdcbb06d5b93b7f76adc92b228c5d16934`.

## 2. Scope confirmation

Approved specification: `docs/engineering/ai-delivery/tasks/BE-01.md` (merged via PR #774), under ADR-0001, the package capability matrix, ADR-0002 and ADR-0003.

Implemented objective: the inactive backend foundation for publisher packages — the PostgreSQL `thoth_package` enum; the non-null `publisher.subscription_package` column with database default `OASIS`; the closed `ThothPackage` and `PublisherCapability` Rust enums with the repository-standard Juniper GraphQL enum representation and stable SCREAMING_SNAKE_CASE codes; one code-owned, compile-time-exhaustive package-to-capability mapping; package-free ordinary create/patch inputs; no public GraphQL exposure; preserved CRUD and history behaviour; no activated consumer.

Out-of-scope changes made: NONE.

## 3. Commits

- `0fbe79a518fe7b14ca8e650e9e3931ea0ed9ffae` - feat(publisher): add inactive publisher package model (BE-01)
- `1c5a8f285dd222297db63f9399b302e11ce63f03` - test(publisher): cover BE-01 package model, boundary and authorization
- the documentation/evidence commit containing this report (SHA in the immutable PR evidence comment)

The specification's suggested commit structure separated the persisted type from the capability mapping; both live in `thoth-api/src/model/publisher/mod.rs`, so the actual bounded structure is (1) migration + schema contract + model + mapping, (2) focused tests, (3) documentation and evidence. This adaptation is permitted by the task instructions.

## 4. Files changed

- `thoth-api/migrations/20260805_v1.7.0/up.sql`
  - reason: create `public.thoth_package` and add `publisher.subscription_package`;
  - behavioural effect: every existing publisher row becomes `OASIS` via the non-null default; new inserts omitting the column receive `OASIS`; no other data changes.
- `thoth-api/migrations/20260805_v1.7.0/down.sql`
  - reason: migration reversibility evidence for the complete-chain disposable test;
  - behavioural effect: drops the column, then the enum. Not an approved operational rollback.
- `thoth-api/src/schema.rs`
  - reason: ADR-0003 Architecture A atomic schema-contract update;
  - behavioural effect: adds the `ThothPackage` SQL type, the publisher table import, and the `subscription_package -> ThothPackage` column (before the timestamp columns, matching the `location.checksum` precedent). Diff contains nothing else. Baseline blob `60b97dcd…`, resulting blob `31f32cf5ea6be492b34d75edb577dab5e4abed20`.
- `thoth-api/src/model/publisher/mod.rs`
  - reason: domain types and persisted field;
  - behavioural effect: closed `ThothPackage` (DbEnum + GraphQLEnum, `OASIS` default, exact `db_rename` codes) and `PublisherCapability` (GraphQLEnum, not persisted); one canonical `capabilities()` / `has_capability()` mapping over three module-private `&'static` slices with no wildcard or fallback arm; `subscription_package: ThothPackage` added to persisted `Publisher` with `#[serde(default)]` so pre-BE-01 JSON remains deserializable. `NewPublisher` and `PatchPublisher` unchanged.
- `thoth-api/src/model/publisher/tests.rs`
  - reason: unit and database evidence (details in section 9).
- `thoth-api/src/graphql/tests.rs`
  - reason: GraphQL non-exposure and authorization evidence (details in section 9).
- `CHANGELOG.md`, `docs/publisher-services/task-status.md`, this report
  - reason: required documentation.

## 5. Implementation decisions

1. Column placement: `subscription_package` sits before `created_at`/`updated_at` in both the schema contract and the `Publisher` struct, matching the existing convention for later-added columns (`location.checksum`); no existing field was reordered.
2. `#[serde(default)]` on `Publisher.subscription_package`: publisher history stores serialized `Publisher` snapshots, and pre-BE-01 snapshots (and any API payload from the unexposed GraphQL object) lack the field. The default makes deserialization tolerate its absence with the documented `OASIS` semantics. Unknown *values* are still rejected by serde, strum, Juniper and PostgreSQL; this is a missing-field default identical to the database default, not a value fallback.
3. The mapping uses three module-private `&'static [PublisherCapability]` constants with four explicit match arms (SPHINX and PYRAMID reference the same constant). This is allocation-free, compile-time exhaustive, has no wildcard/fallback arm, and keeps a single copy of each matrix row.
4. `PublisherField` gained no `SubscriptionPackage` variant: package sorting/filtering is an explicit non-goal.
5. GraphQL enum representation: both enums derive `juniper::GraphQLEnum` (the repository standard). Because no public surface references them, they are absent from the generated SDL; recorded as the expected Juniper reachability result (section 7). No public field was added to force reachability.

List any deviation from the specification: NONE.

## 6. Database and migration effects

Migration added: YES — `thoth-api/migrations/20260805_v1.7.0/` created with `make migration` (correct next version for workspace version 1.6.1 and date 2026-08-05; sorts after `20260504_v1.2.0`).

- migration files: `up.sql` (CREATE TYPE + ALTER TABLE ADD COLUMN … DEFAULT 'OASIS' NOT NULL), `down.sql` (DROP COLUMN, then DROP TYPE);
- schema effect: enum `public.thoth_package` with values exactly `OASIS, OBELISK, SPHINX, PYRAMID` in that order; column `publisher.subscription_package thoth_package NOT NULL DEFAULT 'OASIS'::thoth_package`;
- existing-data effect: every existing publisher row reads `OASIS`; on PostgreSQL 17 this is applied as a metadata default (`pg_attribute.atthasmissing = true`, `attmissingval = {OASIS}`) with no row rewrite; IDs and unrelated values unchanged; no backfill, job, assignment, audit or external record;
- locking/downtime: `ALTER TABLE` takes a brief ACCESS EXCLUSIVE lock on `publisher`; with the metadata-only default the hold time is milliseconds (evidence: a disposable 500,000-row probe table altered in 9.4 ms with an unchanged `pg_relation_filenode`, i.e. no table rewrite). `CREATE TYPE` is trivial. The publisher table is small in production, so expected duration is milliseconds, subject to acquiring the lock;
- empty database result: complete-chain apply/revert/reapply passed (section 9);
- populated database result: representative populated forward migration passed (section 9);
- rollback/forward repair: the down migration executed successfully within the complete-chain revert (reversibility evidence only). The approved operational rollback after merge retains the column, enum, stored values, domain types, canonical mapping, publisher history, canonical Metrics history and distribution assignments; defects use reviewed forward repair; the destructive down migration is not executed operationally; removing the foundation requires an ADR change;
- idempotency/retry: the embedded Diesel harness records each applied version in `__diesel_schema_migrations` inside the migration's transaction, so the migration applies exactly once; after a partial failure the transaction rolls back, no version row is recorded, and a retry re-runs the whole migration cleanly (verified behaviourally by the harness's version accounting in the disposable runs). BE-01 adds no concurrent mutation or job path.

Operational assessment caveats: the metadata-only/no-rewrite claim is evidenced on PostgreSQL 17.10 (matching the repository's compose and CI `postgres:17`); the deployed production PostgreSQL version, runtime and migration execution path are not repository-verified — CG-13 remains open and no production execution is authorized. Safe deployment ordering for a future authorized release: apply the migration before or with the new binary (the old binary never selects the new column; the new binary requires it, so do not run the new binary against an unmigrated database).

## 7. API and compatibility effects

GraphQL/API changes: none public. Both enums implement the mandatory Rust-level Juniper GraphQL enum contract; no field, query, filter, report or mutation references them, `NewPublisher`/`PatchPublisher` are unchanged, and the public `Publisher` object exposes no package or capability data.

Generated schema/client updates: `thoth-client/assets/schema.graphql` regenerated during builds and byte-identical (the unreferenced enums are omitted from the SDL — the expected Juniper reachability result; not worked around by widening the public API). The internal Rust client compiles and its query documents are untouched.

Backwards compatibility: ordinary publisher inputs and all existing fields unchanged; nothing removed or renamed. `thoth-app` generated-client regeneration would produce identical output; dissemination consumers, exports, the OAI branch and Metrics branches require no change (no public contract or behaviour changed; no capability is consumed anywhere). No downstream repository edit is needed and none was made.

Deprecations: none.
Cross-repository dependencies: none.

Publisher-history compatibility: history rows store serialized `Publisher` JSON; no current path deserializes historical snapshots into the current struct. New snapshots naturally include `"subscriptionPackage"`; pre-BE-01 snapshots remain readable and (defensively) deserializable via `#[serde(default)]`. No history backfill, rewrite or deletion.

Package/platform separation: the diff contains no distribution-platform, dissemination, job, publisher-ID environment, OAI or Metrics change, and no inference of package or entitlement from locations or platform configuration. Capabilities are entitlement primitives only.

## 8. Authorization and security

Authorization paths changed: none. No new query or mutation surface exists.
Roles/scopes involved in tests: anonymous, authenticated ordinary user, superuser.
Negative authorization tests: anonymous/authenticated/superuser package selection rejected by schema validation; ordinary create and patch reject `subscriptionPackage` for every caller including superusers; ordinary mutations cannot change or reset a stored package (verified against a database-set `OBELISK` value); no UI-only restriction is relied upon.
Secret or personal-data handling: none; no new logging.
Security limitations: the protected read and superuser mutation are deliberately deferred to BE-03.

## 9. Tests and checks

All commands ran from the repository root at the pushed implementation head, against disposable local services (PostgreSQL 17.10 initialized in the session scratchpad; local Redis; no production or shared resource). `THOTH_EXPORT_API=http://localhost:8181` was set for the compile-time `env!` requirement, as CI does.

### Formatting

Command:

```text
cargo fmt --all -- --check
```

Result:

```text
exit 0, no diff
```

### Unit tests

Command:

```text
cargo test -p thoth-api --features backend
```

Result:

```text
869 passed; 0 failed (lib) + 13 passed; 0 failed (integration); includes the new BE-01 suites:
- all 24 package/capability pairs against the approved matrix
- has_capability agrees with capabilities; duplicate-free slices
- OASIS empty; OBELISK exactly {OAI_PMH, METRICS_COLLECT}; SPHINX == PYRAMID == all six
- exact codes across Display, serde and GraphQL; Rust default OASIS
- aliases, lowercase values, OTHER and unknown codes rejected (serde + strum)
- GraphQL enum registration/roundtrip with exact SCREAMING_SNAKE_CASE values
- pre-BE-01 publisher JSON without the field deserializes to OASIS
```

### Integration/database tests

Command:

```text
cargo test -p thoth-api --features backend   # database-backed tests included above
```

Result:

```text
Included in the 869: exact PostgreSQL enum values and order via pg_enum;
NOT NULL and 'OASIS'::thoth_package default via information_schema;
Diesel roundtrip of all four values; new-publisher default when NewPublisher
omits the column; NULL and unknown-value rejection; history snapshot contains
subscriptionPackage; publisher CRUD regression suites unchanged and passing.
```

Disposable complete-chain validation (database `thoth_be01_chain`, PostgreSQL 17.10):

```text
DATABASE_URL=postgres://…/thoth_be01_chain cargo run migrate
  -> versions 20250000,20260417,20260429,20260504,20260805 applied;
     enum OASIS,OBELISK,SPHINX,PYRAMID; column NOT NULL DEFAULT 'OASIS'::thoth_package
DATABASE_URL=… cargo run migrate --revert
  -> complete-history revert; BE-01 down migration executed; 0 versions recorded;
     thoth_package absent; only __diesel_schema_migrations remains
DATABASE_URL=… cargo run migrate
  -> full chain reapplied; enum, column, default and nullability verified again
```

`--revert` is a complete-history operation; this is migration-reversibility evidence only, not the operational rollback.

Representative populated forward migration (database `thoth_be01_pop`):

```text
Pre-BE-01 state: versions 20250000..20260504 applied via the embedded runner
Fixtures recorded: publishers 11111111-…, 22222222-…, 33333333-… (3 rows, mixed
null/non-null shortname, url, zitadel_id); 2 pre-BE-01 publisher_history
snapshots (JSON without subscriptionPackage); 1 imprint; 55 public tables
Apply BE-01 as the pending migration: cargo run migrate -> only 20260805 applied
After: all 3 publishers OASIS; 0 NULL; IDs, names, shortnames, zitadel_ids
unchanged; history rows present and readable; imprint unchanged; 55 tables;
atthasmissing=true, attmissingval={OASIS}; no job/assignment/audit/external rows
Full-history revert NOT run against these fixtures
```

Note: `embed_migrations!` embeds migrations at compile time; the pre-BE-01 state was produced by temporarily holding out the (then-uncommitted) BE-01 migration directory with all pre-existing migrations byte-identical to the base, then restoring it and rebuilding before applying it as the pending migration. CI always builds fresh, so this stable-toolchain rebuild caveat does not affect CI.

### Lint/static analysis

Command:

```text
cargo clippy --all --all-targets --all-features -- -D warnings
```

Result:

```text
exit 0, no warnings
```

### Other required checks

```text
cargo test --workspace   -> all suites ok (14 "test result: ok" lines, 0 failures)
cargo check --workspace  -> exit 0
git status (thoth-client/assets/schema.graphql) -> unchanged after builds
```

Environment notes recorded without conversion to a pass: the `cargo check -p thoth-api` no-default-features (frontend) build fails identically at the authorized base and at the head (pre-existing, unrelated to BE-01); local Redis was started so the pre-existing redis tests ran rather than being skipped.

## 10. Manual verification

Environment: disposable PostgreSQL 17.10 and local build at the implementation head.
Steps: inspected the generated SDL for package/capability absence (also enforced by test); inspected `NewPublisher`/`PatchPublisher` for `subscriptionPackage` absence; inspected the cumulative diff for absence of platform, job, audit, OAI, Metrics, licence and related-repository changes; inspected the `thoth-api/src/schema.rs` diff for only the approved enum and publisher column with no unrelated reformatting.
Observed result: all confirmed.
Evidence link: exact diffs in PR #779; SQL outputs summarized above.

## 11. CI

CI status: recorded in the immutable exact-head evidence comment on PR #779 after the required workflows reach a terminal result at the final head.
Checks: `build_test_and_check.yml` (test/check/clippy/format against postgres:17), `run_migrations.yml` (full-chain apply/revert/reapply on a disposable database), `check_changelog.yml`.
Failures or warnings: see the evidence comment.

## 12. Rollout and rollback

Initial state after any future authorized merge: package storage and the capability mapping exist; all publishers are `OASIS`; no consumer, protected package API, package mutation, UI, distribution, Metrics or OAI behaviour is activated; no job is created.
Activation required: none exists to activate; later consumers (BE-03 onwards) carry their own HIGH-risk controls.
Feature flag/configuration: none required (no consuming feature).
Migration sequence: single additive migration via the embedded runner; apply before or with the new binary.
Rollback/disable procedure: retained-foundation operational rollback — keep `publisher.subscription_package`, `thoth_package`, stored values, domain types, canonical mapping, publisher history, canonical Metrics history and distribution assignments; do not activate consumers; use reviewed forward repair; do not execute the down migration operationally. Removing the foundation requires an approved ADR change and a separately authorized migration task (`BLOCKED - ROLLBACK REQUIRES ADR CHANGE`).
Monitoring required: none for BE-01 (no runtime behaviour changes).

## 13. Known limitations and deferred work

- The protected package/capability read surface and the superuser package mutation are deferred to BE-03 by design.
- `ThothPackage` and `PublisherCapability` are absent from the generated SDL until BE-03 references them (expected Juniper reachability result; deliberately not worked around).
- The commercial package mapping/backfill is deferred to MIG-01 (CRITICAL, separately approved).
- The metadata-only migration claim is evidenced for PostgreSQL 17; the deployed production version remains unverified under open CG-13.
- The pre-existing no-default-features build failure of `thoth-api` (present at the base) is unrelated and untouched.

## 14. Unresolved issues

- NONE.

## 15. Agent self-assessment

The agent may identify risks but may not approve the task.

Suggested review focus:

- exactness of the 24-pair capability matrix against ADR-0001 and `package-capability-matrix.md`;
- the `schema.rs` diff containing only the approved enum and column change;
- the `#[serde(default)]` decision on the persisted field (missing-field default vs. the prohibited value-fallback class);
- consistency of the rollback narrative with ADR-0001's retained-foundation requirements, and rejection of any reading that treats the tested down migration as an authorized post-merge rollback;
- confirmation that no public GraphQL surface, filter, report or mutation exposes package or capability data.

No production action occurred: all validation used disposable scratchpad-local PostgreSQL/Redis instances and local builds; no production, staging or shared database, credential, workflow dispatch or release was touched.
