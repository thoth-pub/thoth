# MET-WP1-12 - implementation report

Task: `MET-WP1-12` - establish the protected Metrics registry administration GraphQL foundation
Owning issue: [#894](https://github.com/thoth-pub/thoth/issues/894)
Parent programme: [#766](https://github.com/thoth-pub/thoth/issues/766)
Repository: `thoth-pub/thoth`
Work package: WP1 - Metrics domain and database foundation
Risk: **HIGH**

## 1. Exact binding

| Item | Value |
|---|---|
| Authorized base | `feature/metrics @ b5ca7069c9ac21a42092fdbd7400ec01b069b99e` |
| Incorporated `develop` checkpoint | `4546cb632428872b961ad6c17282984d298e3ade` |
| Task branch | `feature/metrics--wp1-registry-admin` |
| PR target | `feature/metrics` |
| Migration identity | `thoth-api/migrations/20260908_v1.9.0` |
| Write budget | 25 paths maximum |
| Workflow | `PROGRAMME_INTEGRATION` |

### 1.1 Authorization provenance

| Record | Location |
|---|---|
| Specification | #894 issue body |
| Specification Amendment 1 | #894 comment `5574219884` |
| Specification Amendment 2 | #894 comment `5581498851` |
| Fresh final independent specification approval | #894 comment `5584094951` |
| CTO specification approval and implementation authorization | #894 comment `5584211971` |

All five durable records were read in full before any mutation, together with
the root `AGENTS.md` and `thoth-api/AGENTS.md`. Conflicts resolve in the
specified order: Amendment 2 controls over Amendment 1, which controls over the
original body. Concretely this implementation follows Amendment 1 sections B
(the exact nine-operation contract and `data:` argument naming), D (complete
replacement, not sparse patch), E/F/G (the mutable/immutable boundaries), I
(Metrics-local append-only `metric_registry_history`), J (audit lifecycle and
transactional atomicity) and K (serialized last-write-wins); and Amendment 2
sections A (exact `TEXT` codes, no normalization), B (one canonical-row lock,
no joined multi-table `FOR UPDATE`), C (bounded constraint sanitization through
`thoth-errors/src/database_errors.rs` only), D (the 25-path budget) and F (a
seed-preserving test fixture).

### 1.2 Preflight, verified before any source mutation

| Check | Result |
|---|---|
| live `feature/metrics` | `b5ca7069c9ac21a42092fdbd7400ec01b069b99e` — matches |
| live `develop` | `4546cb632428872b961ad6c17282984d298e3ade` — matches |
| `feature/metrics--wp1-registry-admin` | absent locally and on `origin` (`git ls-remote` returned nothing) |
| descendant namespace occupancy | none beneath that prospective flat ref |
| PR using that head branch | none (`gh pr list --head … --state all` returned `[]`) |
| `20260908_*` migration | absent on `feature/metrics`, `develop` and `master` |
| Amendment 2 constraint names | all nine literally correct — see §1.3 |
| repository contradiction check | none — see §1.4 |
| working tree | clean; no stash, worktree or unrelated local work disturbed |

The branch was then created directly from
`b5ca7069c9ac21a42092fdbd7400ec01b069b99e`.

### 1.3 Amendment 2 constraint-name verification

The nine names were not taken from the migration text alone. The complete
20-migration chain of the exact authorized base was applied to a disposable
PostgreSQL 17.10 database and `pg_constraint` was queried directly:

```sql
WITH expected(name) AS (VALUES
  ('metric_platform_code_key'), ('metric_platform_code_check'),
  ('metric_platform_display_name_check'), ('metric_measure_code_key'),
  ('metric_measure_code_check'), ('metric_measure_display_name_check'),
  ('metric_measure_definition_check'),
  ('metric_platform_measure_platform_id_measure_id_key'),
  ('metric_platform_measure_supported_grains_check'))
SELECT e.name, c.conname IS NOT NULL AS present, c.contype, c.conrelid::regclass
FROM expected e LEFT JOIN pg_constraint c ON c.conname = e.name ORDER BY e.name;
```

All nine returned `present = t`, with the expected `contype` and owning table:

| Constraint | `contype` | Table |
|---|---|---|
| `metric_measure_code_check` | `c` | `metric_measure` |
| `metric_measure_code_key` | `u` | `metric_measure` |
| `metric_measure_definition_check` | `c` | `metric_measure` |
| `metric_measure_display_name_check` | `c` | `metric_measure` |
| `metric_platform_code_check` | `c` | `metric_platform` |
| `metric_platform_code_key` | `u` | `metric_platform` |
| `metric_platform_display_name_check` | `c` | `metric_platform` |
| `metric_platform_measure_platform_id_measure_id_key` | `u` | `metric_platform_measure` |
| `metric_platform_measure_supported_grains_check` | `c` | `metric_platform_measure` |

No name differed, so no HOLD arose and no observed name was substituted. Each is
an explicitly named `CONSTRAINT` in `20260826_v1.9.0/up.sql`, so PostgreSQL uses
the literal name rather than a positional one.

### 1.4 Repository/specification contradiction check

| Specification assumption | Exact base |
|---|---|
| `require_superuser()` is the authoritative superuser guard | present on `PolicyContext`, `thoth-api/src/policy.rs:201` — not superseded |
| `context.user_id()` is the authoritative actor identity | present on `PolicyContext`, `thoth-api/src/policy.rs:189` |
| no Metrics query or mutation exists yet | confirmed: no `metric` field in `graphql/query.rs` or `graphql/mutation.rs` |
| `metric_registry_history` does not exist | confirmed: no occurrence anywhere in the tree |
| the three registries have the stated columns | confirmed against `20260826_v1.9.0/up.sql` and the live migrated database |
| `metric_platform_measure` has no timestamps | confirmed |
| a seed-preserving fixture exists | `setup_registry_db()` in `metric_platform/tests.rs` reverts through `20260826` and reapplies, restoring the migration-owned seeds |

One non-blocking correction to the independent review's non-blocking note:
`thoth-errors/src/database_errors.rs` **does** already contain an inline
`#[cfg(test)] mod tests`. This does not affect the budget; the new mapping test
was added inside that existing module.

## 2. Exact 25-path containment

All 25 authorized paths were used, including this report. No 26th path was
created or modified, and nothing was deleted, moved or renamed, so no
`HOLD - WRITE BUDGET AMENDMENT REQUIRED` condition arose.

Cumulative `git diff --stat` against the authorized base:

```text
 CHANGELOG.md                                       |   1 +
 .../MET-WP1-12-implementation-report.md            | 789 ++++++++++++++++++
 docs/metrics/contract-register.md                  |  68 ++
 docs/metrics/task-status.md                        | 160 ++--
 thoth-api/migrations/20260908_v1.9.0/down.sql      |  21 +
 thoth-api/migrations/20260908_v1.9.0/up.sql        |  63 ++
 thoth-api/src/graphql/metric_registry_tests.rs     | 904 +++++++++++++++++++++
 thoth-api/src/graphql/mod.rs                       |   2 +
 thoth-api/src/graphql/model.rs                     | 204 +++++
 thoth-api/src/graphql/mutation.rs                  | 113 +++
 thoth-api/src/graphql/query.rs                     |  52 ++
 thoth-api/src/model/metric_measure/crud.rs         | 165 ++++
 thoth-api/src/model/metric_measure/mod.rs          | 105 ++-
 thoth-api/src/model/metric_measure/tests.rs        | 335 +++++++-
 thoth-api/src/model/metric_platform/crud.rs        | 159 ++++
 thoth-api/src/model/metric_platform/mod.rs         |  90 +-
 thoth-api/src/model/metric_platform/tests.rs       | 467 ++++++++++-
 .../src/model/metric_platform_measure/crud.rs      | 194 +++++
 thoth-api/src/model/metric_platform_measure/mod.rs | 108 ++-
 .../src/model/metric_platform_measure/tests.rs     | 486 ++++++++++-
 thoth-api/src/model/metric_registry_history/mod.rs | 208 +++++
 .../src/model/metric_registry_history/tests.rs     | 514 ++++++++++++
 thoth-api/src/model/mod.rs                         |   1 +
 thoth-api/src/schema.rs                            |  25 +
 thoth-errors/src/database_errors.rs                |  68 ++
 25 files changed, 5178 insertions(+), 124 deletions(-)
```

The 124 deletions are the replaced `Last updated` narrative in
`docs/metrics/task-status.md` and the superseded doc comments in the three
registry `mod.rs` files, which had recorded that no administration surface
existed. No line of any predecessor slice's test or source was removed.

The specifically prohibited paths `thoth-errors/src/lib.rs`,
`thoth-api/src/policy.rs`, `thoth-api/src/graphql/tests.rs`,
`thoth-api/src/graphql/mutation_guard.rs`, `Cargo.toml`, `Cargo.lock`, every
existing migration, all workflows/scripts and every downstream repository are
untouched.

## 3. Schema delivered

Migration `thoth-api/migrations/20260908_v1.9.0` creates two closed enums and
one table, and nothing else.

```text
metric_registry_history_entity   PLATFORM | MEASURE | PLATFORM_MEASURE
metric_registry_history_action   CREATE | UPDATE
```

```text
metric_registry_history
  metric_registry_history_id  uuid                     NOT NULL DEFAULT uuid_generate_v4()
  entity                      metric_registry_history_entity NOT NULL
  entity_id                   uuid                     NOT NULL
  action                      metric_registry_history_action NOT NULL
  actor                       text                     NOT NULL
  before_state                jsonb                    NULL
  after_state                 jsonb                    NOT NULL
  created_at                  timestamptz              NOT NULL DEFAULT CURRENT_TIMESTAMP
```

Verified directly from `pg_attribute`/`pg_attrdef` on the migrated database:
every column has exactly the type, nullability and default above, and the table
has exactly these eight columns.

Constraints, all explicitly named so the error boundary can map them
deterministically:

| Constraint | Definition |
|---|---|
| `metric_registry_history_pkey` | `PRIMARY KEY (metric_registry_history_id)` |
| `metric_registry_history_actor_check` | `CHECK (actor ~ '[^[:space:]]')` |
| `metric_registry_history_action_before_state_check` | `CHECK ((action = 'CREATE' AND before_state IS NULL) OR (action = 'UPDATE' AND before_state IS NOT NULL))` |

Deliberate absences, each asserted by a test:

- no `updated_at` column and no trigger — the table is append-only;
- no foreign key on the polymorphic `entity_id` — audit evidence must not be
  cascade-deleted with the registry state it describes;
- no `DELETE` action value — no delete mutation exists;
- no secondary index — the primary key is the complete intended index set,
  because no approved audit access path exists yet;
- no timestamp column added to `metric_platform_measure`.

## 4. Diesel contract (ADR-0003)

`thoth-api/src/schema.rs` was edited directly, never generated. Three additions,
each in its existing alphabetical position, and no unrelated reformatting:

1. `sql_types::MetricRegistryHistoryEntity` and
   `sql_types::MetricRegistryHistoryAction`;
2. the `metric_registry_history` `table!` block, between `metric_record_revision`
   and `metric_rollup_delta`;
3. `metric_registry_history` in `allow_tables_to_appear_in_same_query!`.

No `joinable!` entry was added: the table has no foreign key by design.

## 5. GraphQL surface added

Six mutations and three administrative lookups, all SUPERUSER only:

```graphql
createMetricPlatform(data: NewMetricPlatform!): MetricPlatform!
updateMetricPlatform(data: PatchMetricPlatform!): MetricPlatform!
createMetricMeasure(data: NewMetricMeasure!): MetricMeasure!
updateMetricMeasure(data: PatchMetricMeasure!): MetricMeasure!
createMetricPlatformMeasure(data: NewMetricPlatformMeasure!): MetricPlatformMeasure!
updateMetricPlatformMeasure(data: PatchMetricPlatformMeasure!): MetricPlatformMeasure!

metricPlatformByCode(code: String!): MetricPlatform!
metricMeasureByCode(code: String!): MetricMeasure!
metricPlatformMeasureByCodes(platformCode: String!, measureCode: String!): MetricPlatformMeasure!
```

Rust resolver names are the repository's snake_case equivalents. Supporting
types: three object types (`MetricPlatform`, `MetricMeasure`,
`MetricPlatformMeasure`), six input objects, and the additive GraphQL exposure
of four **existing** closed database enums
(`MetricPlatformOwnershipClass`, `MetricMeasureCategory`, `MetricMeasureUnit`,
`MetricReportingGrain`) with no value added, removed or renamed.

The audit table is exposed nowhere: there is no audit object type, input, enum
or query in the schema.

## 6. Mutability boundary

| Entity | Selector | Mutable | Immutable / not expressible in the patch input |
|---|---|---|---|
| Platform | `code` | `displayName`, `enabled`, `publicDescription` | `code`, `ownershipClass` |
| Measure | `code` | `displayName`, `publicVisibility`, `definition`, `methodologyVersion`, `enabled` | `code`, `category`, `unit`, `allowNegative`, `additiveAcrossTime`, `additiveAcrossWorks` |
| Platform-measure | `(platformCode, measureCode)` | `supportedGrains`, `supportsCountry`, `supportsInstitution`, `supportsPublication`, `directCollection`, `enabled` | the mapped platform/measure identity |

Immutability is enforced structurally rather than by validation: the immutable
fields are absent from the `Patch...` inputs, so an attempt to change one is a
GraphQL schema error, and they are absent from the coordinators' `SET` clauses,
so no code path can write them. There is no delete mutation; retirement is
`enabled = false`, an ordinary audited update.

`Patch...` is a complete replacement of the approved mutable set. For the two
nullable mutable fields, `MetricPlatform.public_description` and
`MetricMeasure.methodology_version`, omission and explicit GraphQL `null` both
store SQL `NULL`; retaining a value requires sending it. No tri-state wrapper
was added.

## 7. Exact-code contract (Amendment 2 section A)

Codes are ordinary PostgreSQL `TEXT` under the existing `UNIQUE(code)`
constraints, and `MET-WP1-12` adds no alternative normalized identity.

There is **no** application-side trimming, case folding, `lower()`/`upper()`,
`ILIKE`, Unicode normalization, whitespace normalization or aliasing at any
entry point. Every read and selector is a plain Diesel `.eq()` on the `code`
column, and every create persists `data.code` unchanged. The same semantics
apply to platform-measure code resolution, which reuses the platform and
measure modules' own `by_code` readers rather than a second lookup rule.

A single grep over the three coordinators finds no normalizing call, and the
behaviour is asserted at both layers — see §11.3.

## 8. Audit lifecycle and atomicity

Both write paths run as one transaction on one connection
(`connection.transaction(|connection| …)`), and the audit insert takes
`&mut PgConnection` rather than a pool, so it is structurally impossible to
commit an audit row outside the caller's transaction.

**CREATE.** Insert the canonical row `RETURNING` all columns, then append
exactly one audit row with `action = CREATE`, `before_state = NULL`,
`after_state =` the exact persisted row (including the generated identifier and
database-authored timestamps), and `actor =` the authenticated principal.

**UPDATE.** Lock the canonical row with `FOR UPDATE` and read the current state
under that lock; if the requested replacement equals the current mutable state,
return the current row with **no** `UPDATE` statement, **no** timestamp movement
and **no** audit row; otherwise update only the approved mutable fields
`RETURNING` all columns, then append exactly one audit row carrying the exact
before and after states.

Unauthorized, rejected, failed and genuinely no-op requests create no audit
entry. Authorization precedes every coordinator call, so a denied request never
opens a transaction at all.

The actor is derived from `context.user_id()` and is never accepted from the
request, so the audit trail is not spoofable by a caller.

## 9. Concurrency and lock topology (Amendment 1 section K, Amendment 2 section B)

Required semantics are `serialized last-write-wins`. No optimistic-concurrency
token was added: no single coherent version token spans the three registries,
and `metric_platform_measure` deliberately has no timestamp or version column.

Each UPDATE requests **exactly one** application `FOR UPDATE` lock, on the
canonical row being updated:

| Coordinator | Locked row | Unlocked reads |
|---|---|---|
| `update_metric_platform` | the `metric_platform` row selected by exact `code` | — |
| `update_metric_measure` | the `metric_measure` row selected by exact `code` | — |
| `update_metric_platform_measure` | the `metric_platform_measure` row for the resolved pair | the platform and measure rows, resolved by ordinary non-locking reads |

No joined multi-table `FOR UPDATE` exists, no coordinator takes a second lock
target after its canonical-row lock, and CREATE takes no application lock at
all — a concurrent duplicate loses at PostgreSQL's unique index rather than at
an application pre-check. PostgreSQL's own unique/FK/statement locks remain
authoritative and are not replaced by hand-written locking.

Because the three coordinators each take a single lock, and the mapping
coordinator deliberately does not lock its parents, no application-defined lock
cycle between them is constructible.

## 10. Database-error boundary (Amendment 2 section C)

Only `thoth-errors/src/database_errors.rs` was extended, with eleven entries in
the existing `DATABASE_CONSTRAINT_ERRORS` map: the nine Amendment 2 baseline
registry constraints plus the two CHECK constraints this migration introduces.
No `ThothError` variant, no GraphQL error type and no Metrics-specific public
error vocabulary was added, and `thoth-errors/src/lib.rs` is untouched.

| Constraint | Bounded message |
|---|---|
| `metric_platform_code_key` | A metric platform with this code already exists. |
| `metric_platform_code_check` | Metric platform code must not be an empty string. |
| `metric_platform_display_name_check` | Metric platform display name must not be an empty string. |
| `metric_measure_code_key` | A metric measure with this code already exists. |
| `metric_measure_code_check` | Metric measure code must not be an empty string. |
| `metric_measure_display_name_check` | Metric measure display name must not be an empty string. |
| `metric_measure_definition_check` | Metric measure definition must not be an empty string. |
| `metric_platform_measure_platform_id_measure_id_key` | A mapping between this metric platform and this metric measure already exists. |
| `metric_platform_measure_supported_grains_check` | Supported grains must list at least one reporting grain, with no duplicates. |
| `metric_registry_history_actor_check` | Metric registry history actor must not be an empty string. |
| `metric_registry_history_action_before_state_check` | Metric registry history is invalid: a creation must record no previous state and an update must record one. |

Sanitization matters because an **unmapped** constraint falls through to
`ThothError::DatabaseError(info.message())`, which carries PostgreSQL's own
text. Both new audit CHECKs are mapped even though only the actor check is
plausibly reachable through the nine operations: mapping a constraint that
cannot fire costs nothing, whereas leaving a reachable one unmapped leaks raw
driver text, so the fail-safe choice was taken.

### 10.1 Foreign-key constraints: reviewed conclusion

`metric_platform_measure_platform_id_fkey` and
`metric_platform_measure_measure_id_fkey` are deliberately **not** mapped, and
are not in Amendment 2's list. The mapping coordinator resolves both codes with
ordinary reads *inside its transaction* before inserting, so an unknown code
fails as `EntityNotFound` and never reaches the foreign key. The only way to
reach an FK violation would be a concurrent deletion of a referenced registry
row, and no mutation in the repository deletes one. This is recorded as a
reviewed conclusion rather than an omission; if a delete path is ever added,
these two names must be mapped in the same slice.

## 11. Tests

`MET-WP1-12` adds 83 tests: 8 for the audit table's shape and migration, 21/20/22
for the platform, measure and mapping coordinators, and 12 for the GraphQL
surface, plus one inline mapping test in `thoth-errors`.

### 11.1 Authorization

| Evidence | Test |
|---|---|
| all nine operations denied to seven distinct non-superuser principals — anonymous, authenticated-with-no-role, `PUBLISHER_USER`, `PUBLISHER_ADMIN`, `WORK_LIFECYCLE`, `CDN_WRITE` and the `DISSEMINATION_WORKER` machine role — 63 executions, every one `NO_ACCESS`/`Unauthorized` | `every_operation_is_denied_to_every_caller_that_is_not_a_superuser` |
| a denied call leaves the registry and the audit history byte-identical: the platform count, the untouched display name, the empty mapping table, the two seeds and the empty audit table are all asserted after those 63 denials | same test |
| **authorization precedes any database access** — the context's pool points at an unreachable database, and every one of the nine operations still returns the fail-closed denial rather than a connection error | `authorization_is_decided_before_any_database_access` |
| a superuser drives create → lookup → map → update across all three registries, and all four committed changes are attributed to the authenticated principal | `a_superuser_can_drive_the_whole_administrative_lifecycle` |
| every coordinator has exactly one call site and it is guarded; there is exactly one guard function | `the_resolvers_authorize_before_reaching_a_coordinator` |

The unreachable-pool test is the strongest of these: it converts "the guard is
written first" from a code-reading claim into an observable one.

### 11.2 Audit semantics

| Evidence | Test |
|---|---|
| CREATE records `before_state = NULL` and an `after_state` byte-equal to the persisted row | `create_persists_the_exact_values_and_audits_the_persisted_row` (×3 registries) |
| UPDATE records exact before and after states | `update_replaces_only_the_mutable_fields_and_audits_before_and_after` and equivalents |
| a genuine no-op writes nothing, moves no timestamp and audits nothing | `a_true_no_op_update_writes_nothing_and_audits_nothing`, `update_replaces_only_the_mutable_fields_and_a_no_op_audits_nothing`, `a_no_op_update_through_the_api_changes_and_audits_nothing` |
| **a failing audit write rolls back the canonical change** — a blank actor violates `metric_registry_history_actor_check` *after* the canonical INSERT has already succeeded inside the transaction, and neither row survives | `a_failing_audit_write_rolls_back_the_canonical_change` |
| rejected creates and unknown-code updates audit nothing | `a_duplicate_code_fails_atomically_and_is_sanitised`, `an_unknown_code_is_reported_as_not_found_and_audits_nothing` |
| the polymorphic `entity_id` really does outlive its subject: deleting the referenced platform leaves the audit row | `deleting_a_registry_row_leaves_its_audit_evidence_behind` |
| the two CHECK constraints reject a blank actor and every malformed action/before-state pair | `the_actor_check_rejects_a_blank_actor`, `the_action_check_binds_before_state_to_the_action` |

The rollback test deserves emphasis: it is a true RED-capable proof. If the
canonical write and the audit write were not one transaction, the platform row
would survive without its audit evidence and the assertion would fail.

### 11.3 Exact-code semantics

`a_code_is_stored_and_matched_exactly_with_no_normalisation` creates the
platform codes `Mixed_Case`, `mixed_case`, ` leading`, `trailing `,
`inner space`, `unícode` and `UNÍCODE` — all seven coexist, each stored
byte-identically — and then proves that `MIXED_CASE`, `leading`, `trailing`,
`innerspace` and `unicode` resolve to **nothing** rather than folding onto a
stored code.

The measure test additionally creates `Title_Sessions` alongside the seeded
`title_sessions` and proves they are different rows with different identifiers
and that the seed's definition is untouched.
`code_resolution_is_exact_at_every_entry_point` proves the same for both halves
of a mapping's code pair, across create, lookup **and** update selectors, and
`codes_are_matched_exactly_through_the_api` repeats it at the GraphQL boundary.

### 11.4 Lock topology and concurrency (real multi-connection PostgreSQL)

| Evidence | Test |
|---|---|
| two competing updates to one contended row serialize, and the audit chain matches actual commit order — the second entry's `before_state` is byte-equal to the first entry's `after_state`, with no gap, and the last `after_state` equals the committed row | `two_competing_updates_serialise_and_their_audit_chain_matches_commit_order` (platform and measure) |
| **a mapping update does not block behind a lock on its platform row** — one connection holds a real `SELECT … FROM metric_platform … FOR UPDATE` in an open transaction while another completes the mapping update; if the coordinator locked its parents this would deadlock against the 15-second bound and fail | `updating_a_mapping_does_not_lock_its_platform_or_measure_rows` |
| a concurrent platform update and mapping update both succeed, with no application-defined lock cycle | `concurrent_platform_and_mapping_updates_do_not_deadlock` |
| each coordinator's source contains exactly one `.for_update()`, on the canonical row, with no `inner_join`/`left_join`/`left_outer_join`/`joinable!` anywhere, and no second lock after it; `resolve_pair` contains no lock at all | `the_coordinator_takes_exactly_one_application_row_lock` (×2), `the_coordinator_locks_only_the_mapping_row` |

The parent-lock test is the decisive one for Amendment 2 section B: it is a
behavioural proof, not a source-shape assertion, that the mapping coordinator
does not lock the rows it must not lock.

### 11.5 Sanitized database-error boundary

`constraint_failures_reach_the_client_bounded_and_sanitised` drives five real
failures through the **GraphQL** boundary — duplicate platform code, duplicate
measure code, duplicate pair, blank code and empty `supportedGrains` — and
asserts both the exact bounded message and that the whole rendered response
contains none of: `metric_platform_code_key`, `metric_measure_code_key`,
`metric_platform_measure_platform_id_measure_id_key`,
`metric_platform_code_check`, `metric_platform_measure_supported_grains_check`,
`duplicate key value`, `violates`, `INSERT INTO`, `SELECT `, `DETAIL:`,
`CONTEXT:`, `pg_`, `postgres://`, `cardinality(` or `Database error`.

Model-level equivalents assert the same for the coordinator return values, and
`metrics_registry_constraints_map_to_bounded_messages` in
`thoth-errors/src/database_errors.rs` asserts that all eleven mapped names yield
`DatabaseConstraintError` — not the raw-text `DatabaseError` variant — and that
no message leaks its own constraint name or SQL.

### 11.6 Registry semantics and non-goals

Immutability, the mapping identity being unmovable, `supported_grains` CHECK
enforcement (empty, and both duplicate shapes), ordered-array round-tripping,
`EntityNotFound` for unknown codes, the seeded measures remaining exactly two
and unrewritten, and the absence of timestamps on `metric_platform_measure` are
each asserted. `the_sdl_exposes_exactly_the_nine_approved_operations` counts the
Metrics root fields — exactly 3 on `QueryRoot`, exactly 6 on `MutationRoot` — so
a tenth operation cannot be added unnoticed, and asserts the deferred names
`metricPlatforms`, `metricMeasures`, `metricPlatformMeasures`, `deleteMetric`,
`metricRegistryHistory`, `metricPublisherPlatformApproval` and `metricSource`
are absent.

### 11.7 Exact commands and results

```text
$ cargo test -p thoth-api --features backend
test result: ok. 1591 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1294.62s
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.03s
test result: ok. 0 passed; 0 failed; 8 ignored; 0 measured; 0 filtered out; finished in 0.00s

$ cargo clippy --all --all-targets --all-features -- -D warnings
Finished `dev` profile in 11m 40s        exit code 0

$ cargo check --workspace
exit code 0

$ cargo fmt --all -- --check
exit code 0

$ git diff --check
exit code 0

$ ./target/debug/thoth migrate                 # DATABASE_URL=…/thoth_migrate
exit code 0; 20260908 applied; metric_registry_history present
$ ./target/debug/thoth migrate --revert        # full-chain revert, as CI runs it
exit code 0; 0 migrations remain; 0 metric_ tables; 0 audit enums
$ ./target/debug/thoth migrate                 # reapply
exit code 0; 3 audit constraints; 1 index; 2 measure seeds; 0 audit rows
```

The only warning emitted by any of these is the pre-existing dependency notice
`proc-macro-error2 v2.0.1`, which is present on the authorized base and is not a
lint against repository code.

An earlier run of the same suite reported `1589 passed; 2 failed`. Both failures
were defects in *test* code, not in the implementation, and both are fixed:

1. `the_sdl_exposes_exactly_the_nine_approved_operations` compared operation
   signatures by exact substring, but Juniper writes each argument's description
   inline (`metricPlatformByCode("The platform's stable code…" code: String!)`),
   so the comparison could never match. It now strips quoted strings and
   collapses whitespace before comparing, which still matches names, argument
   types, return types and nullability exactly.
2. `metric_reconciliation_run::tests::no_reconciliation_enum_trigger_procedure_or_graphql_surface_was_introduced`
   — a `MET-WP1-11` guard asserting the SDL never contains the word
   "reconciliation" — was tripped by this slice's own
   `createMetricPlatformMeasure` description, which ended "Starts no collection,
   import, export or reconciliation". The guard is a whole-SDL substring check
   and cannot distinguish a real surface from a denial of one. The fix was to
   reword **this slice's** description to "Creating a mapping starts, schedules
   and enqueues no downstream activity of any kind";
   `metric_reconciliation_run/tests.rs` remains byte-identical to the authorized
   base, so no write-budget amendment was required. The other guarded
   vocabularies (`rollup`, `coverage`, `checkpoint`, `ingestion`, `operas`,
   `provenance`, `import`) were checked and none is guarded; only
   `metric_reconciliation_run/tests.rs` calls `as_sdl()`.

## 12. Migration safety verification

### 12.1 Constraint and column contract on a real database

The complete 20-migration chain of the authorized base plus this migration was
applied to a disposable PostgreSQL 17.10 cluster, and the delivered contract was
read back from the catalogues rather than asserted from the SQL text. Results
are in §3: eight columns with exactly the specified types, nullability and
defaults; three constraints under exactly the specified names; one index; no
foreign key; no trigger.

### 12.2 Apply, revert and re-apply with populated predecessor data

The audit table was populated, then reverted and re-applied:

```text
audit rows before revert:            1
measure seeds before revert:         2
-- down.sql --
audit table after revert:            0
audit enums after revert:            0
predecessor metric tables surviving: 19
measure seeds after revert:          2
-- up.sql --
constraints after reapply:           3
indexes after reapply:               1
audit rows after reapply:            0
measure seeds after reapply:         2
```

The downgrade removes exactly the objects this migration created and touches no
`MET-WP1-01..11` Metrics table, no bibliographic table and neither seed. The
re-apply restores the full contract and seeds nothing.

Reverting is a genuinely lossy downgrade of audit evidence — it discards every
recorded registry mutation. That is stated explicitly in `down.sql` and accepted
because the table is empty at deployment and the canonical registry rows are
untouched by the revert.

### 12.3 Lock profile

Measured rather than asserted. The migration was replayed inside a probe
transaction and `pg_locks` was joined to `pg_class` for the backend:

```sql
SELECT c.relname, l.mode FROM pg_locks l JOIN pg_class c ON c.oid = l.relation
WHERE l.pid = pg_backend_pid() AND c.relnamespace = 'public'::regnamespace
  AND c.relname LIKE 'metric%' AND c.relname <> 'metric_registry_history';
```

The result is **empty**: the migration takes **no lock on any pre-existing
table**. The only lock it holds is `AccessExclusiveLock` on its own new
primary-key index, an object no other session can see before commit.

This is stronger than the usual Metrics-table profile precisely because the
audit table has **no foreign key**. A new Metrics table with an FK takes
`SHARE ROW EXCLUSIVE` on each parent, which blocks concurrent writes to the
parent for the duration of the migration. Here there is no parent, so concurrent
reads *and writes* to every existing table proceed throughout. Expected
production runtime is milliseconds on an empty table.

## 13. GraphQL compatibility

### 13.1 Base and final SDL

The SDL is build-generated into `thoth-client/assets/schema.graphql` by
`thoth-client/build.rs` and is gitignored, so `git status` is not a valid
comparison. Both revisions were built and the generated files captured:

| SDL | Lines | SHA-256 |
|---|---|---|
| base, clean tree at `b5ca7069c9ac21a42092fdbd7400ec01b069b99e` | 4630 | `091e11f293132fdec784de420e3addf251f5020ba7e387889b292a066be15d8e` |
| final, this implementation | 4813 | `d5ac40b8f381f12f62e8c507f24130199189b40750c040f5a6f41997e1e4963b` |

### 13.2 Strictly additive: structural proof

A line-level `difflib.SequenceMatcher` opcode analysis over the two files:

```text
base lines: 4630   head lines: 4813
deleted: 0   replaced: 0   inserted: 183
STRICTLY ADDITIVE
```

`deleted = 0` and `replaced = 0` means the base SDL is an exact subsequence of
the final SDL: **no existing line changed in any way**. That covers, by
construction, every one of the required properties — no type, field, input or
enum deletion; no nullability change; no enum-variant change; no argument or
default change; and no change to any existing description.

`diff -u` independently reports zero removal lines.

### 13.3 What was added

Thirteen new top-level declarations and nine new root fields, and nothing else:

```text
type MetricPlatform            input NewMetricPlatform          enum MetricPlatformOwnershipClass
type MetricMeasure             input PatchMetricPlatform        enum MetricMeasureCategory
type MetricPlatformMeasure     input NewMetricMeasure           enum MetricMeasureUnit
                               input PatchMetricMeasure         enum MetricReportingGrain
                               input NewMetricPlatformMeasure
                               input PatchMetricPlatformMeasure
```

```text
QueryRoot:     metricPlatformByCode, metricMeasureByCode, metricPlatformMeasureByCodes
MutationRoot:  createMetricPlatform, updateMetricPlatform,
               createMetricMeasure, updateMetricMeasure,
               createMetricPlatformMeasure, updateMetricPlatformMeasure
```

The four enums are additive **exposures** of database enums that already existed
on the base; no value was added, removed or renamed, which the test in §11
asserts value-by-value against the merged inventory.

No authorization behaviour of any existing operation changed: the only
authorization code added is one new guard helper used by the six new mutations,
and the three new lookups reuse `require_superuser()` exactly as the two
pre-existing superuser queries do.

### 13.4 Downstream consumer impact

No downstream repository was modified, and none needs to change. Because the
change is provably additive, every existing document, query, fragment and
generated client continues to compile and execute unchanged.

| Consumer | Impact | Why |
|---|---|---|
| `thoth-pub/thoth-app` | none required | additive-only schema; no existing field, type or nullability changed. Registry administration UI is not in scope for `MET-WP1-12` |
| `thoth-pub/thoth-pyramid` | none required | consumes existing queries only |
| `thoth-pub/thoth-dissemination` | none required | its consumed surface is untouched |
| standalone `thoth-client` / `thothlibrary` | none required | additive schema; regeneration optional, not required |
| internal Rust `thoth-client` / `thoth-export-server` | none required | `thoth-client/assets/queries.graphql` is unchanged, and the crate builds against the regenerated schema in this same PR |
| `thoth-pub/metrics-dashboard` | none required | reads no Metrics operation yet; the service/dashboard query surface remains deferred |
| `thoth-pub/metrics-widget` | none required | as above |
| `thoth-pub/baboon` | none required | consumes existing queries only |
| Sphinx (`thoth-pub/thoth-sphinx`, planned) | none required | not yet a consumer; when it becomes one it consumes the merged contract |

No `HOLD - CROSS-REPOSITORY TASK REQUIRED` condition arose.

## 14. Migration and data effects

- One new table and two new enums; nothing existing altered or dropped.
- No seed, backfill, data migration or rewrite of any kind. The migration-owned
  `title_sessions` and `net_units` measures are untouched, and no platform,
  platform-measure mapping, source, source account, OPERAS mapping or event URI
  is created or guessed.
- The new table is empty at deployment and stays empty until a superuser invokes
  one of the six mutations.
- No production or staging database was touched by this task. All validation ran
  against disposable local PostgreSQL 17 instances.

## 15. Authorization and security effects

- Adds one protected global write surface: six mutations and three lookups, all
  SUPERUSER only, all fail-closed, all authorized before any mutation-specific
  database access.
- Denies anonymous callers, authenticated users with no applicable role,
  publisher users, publisher admins, work-lifecycle and CDN-write roles, and the
  `DISSEMINATION_WORKER` machine role. A denied call changes neither registry nor
  audit state.
- Introduces no ZITADEL role, no Metrics service role, no credential or secret,
  no package/capability change and no policy change. `thoth-api/src/policy.rs` is
  untouched, and `SUPERUSER` is used as the existing human/global administration
  boundary, not as a machine-service shortcut.
- Broadens no existing permission: the guard is used only by the nine new
  operations.
- Logs nothing. No token, secret, credential, object URL or personal data is
  written by any new code path. The audit `actor` is the ZITADEL user identifier
  already used by the merged service-configuration audit trail.
- Sanitized error boundary: expected registry and audit constraint failures
  return bounded messages with no PostgreSQL text, constraint name, SQL fragment,
  driver diagnostic, connection detail or infrastructure information.

## 16. External and runtime effects

**None.** This task performs no provider read or write, starts no collection,
ingestion, normalization, rollup, OPERAS synchronization, export, import or
reconciliation, enqueues no job, invalidates no cache, sends no request to any
external service, and activates no production behaviour. Changing
`directCollection` on a mapping is a configuration write and nothing more: no
WP1 runtime path reads it yet.

The only authorized automatic external side effect is the natural PR CI, whose
`publish-to-dockerhub` workflow may publish `ghcr.io/thoth-pub/thoth:staging-pr-<PR>`.
That is CI publication, not deployment, release or production activation.

## 17. WP1 non-goal confirmation

Not implemented, and deliberately so: publisher-platform approval
administration; source, source-account and source-checkpoint administration; the
service/dashboard registry queries `metricPlatforms` and `metricMeasures`; any
delete or bulk mutation; any audit-history query, object type or index; any real
platform, platform-measure, source, source-account or OPERAS seed; any
package-to-capability change; any Metrics machine role or credential model; any
ingestion, rollup, OPERAS synchronization or reconciliation runtime; any
thoth-app, dashboard, widget, Sphinx, dissemination, Pyramid or Baboon
implementation; and any production migration, deployment, release or activation.

The WP1 seed obligation remains an open programme gate: the repository source
inventory still records source/platform mappings as unapproved, so this slice
creates the administrative surface through which approved mappings can later be
entered, and enters none.

## 18. Deviations

```text
NONE
```

Every element of the approved specification is implemented as written. The
migration identity, branch name, PR target, GraphQL operation and argument
names, mutability boundaries, audit contract, lock topology, error-boundary path
and 25-path budget are exactly as bound by the CTO authorization. No HOLD
condition arose at any point.

Two conservative choices are recorded explicitly so a reviewer can challenge
them rather than discover them:

1. **Both** new audit CHECK constraints are mapped in
   `thoth-errors/src/database_errors.rs`, although only
   `metric_registry_history_actor_check` is demonstrably reachable (through a
   blank actor). Mapping the action/before-state CHECK as well costs one map
   entry and removes any possibility of raw PostgreSQL text escaping if a future
   change makes it reachable. See §10.
2. The two `metric_platform_measure` foreign-key constraints are deliberately
   **not** mapped, with the reasoning recorded in §10.1 as a reviewed
   conclusion rather than an oversight.

## 19. Known limitations

- **Audit history is write-only in WP1.** There is no query, index or export for
  `metric_registry_history`; it is read in this slice only by tests, through raw
  SQL. That is the approved shape — no access path is authorized yet — but it
  does mean an operator cannot inspect the trail through the API until a later
  bounded slice adds one.
- **Reverting the migration discards audit evidence.** Stated in `down.sql` and
  in §12.2. Acceptable while the table is empty; it would need a deliberate
  decision once real administration history exists.
- **No optimistic concurrency.** `serialized last-write-wins` is the approved
  contract: a caller that read a row, thought, and then submitted an update will
  overwrite an intervening change without being told. The audit chain records
  what happened, but nothing prevents it. This is an explicit Amendment 1
  section K trade-off for a low-frequency superuser-only surface.
- **`Patch` is a full replacement.** A caller that omits a nullable field
  intending "leave it alone" will null it. This is the approved semantics and is
  documented in every relevant GraphQL description, but it is a real
  foot-gun for a hand-written mutation and is worth surfacing in any future
  administration UI.
- **`ownership_class` has no in-band repair path.** A platform created with the
  wrong ownership class cannot be corrected through this API at all; correction
  needs separately reviewed work.
- **The registry remains empty.** No platform or mapping exists, so the surface
  is inactive in practice until approved source/platform mappings arrive through
  a separate slice.
- **`docs/metrics/migration-inventory.md` is not updated.** It is not in the
  authorized 25-path budget, and the predecessor slice did not update it either.
  Flagged here rather than silently fixed; if the programme wants that inventory
  kept current, it needs its own authorization.

## 20. Compliance with the authorization

| Authorized action | Performed |
|---|---|
| freshly reverify all bindings before mutation | yes — §1.2, §1.3 |
| create the task branch from the exact base | yes |
| create/modify only the 25 authorized paths | yes — §2 |
| create the bound `20260908_v1.9.0` migration | yes |
| use disposable local PostgreSQL for validation | yes — PostgreSQL 17.10, three throwaway databases |
| run the specification-required local validation | yes — §11 |
| commit the bounded implementation | yes |
| push without force-push | yes |
| create a DRAFT PR targeting `feature/metrics`, linked to #894 and #766 | yes |
| allow natural PR CI to run | yes, unmodified |

| Not authorized | Performed |
|---|---|
| manual CI dispatch, rerun or cancellation | no |
| staging or production migration execution | no |
| provider/runtime access beyond the automatic GHCR staging image | no |
| marking the PR ready for review | no |
| merge | no |
| branch deletion | no |
| `feature/metrics -> develop` integration | no |
| deployment, release or production activation | no |
| issue/comment mutation on #894 or #766 | no |

## 21. Remaining gates

1. **Fresh independent exact-head source review.** Required, and it must be
   bound to the exact PR head SHA. The implementing agent may not approve its own
   work, and any subsequent commit invalidates a completed source approval.
2. **CTO merge authorization**, bound to the exact independently reviewed head.
3. Marking the PR ready for review, and merge into `feature/metrics`.
4. `feature/metrics -> develop` integration — separately authorized, not part of
   this task.
5. Deployment, staging/production migration execution, release and production
   activation — all separate, none authorized here.
6. The WP1 seed obligation (first real platform and platform-measure mappings)
   remains blocked on approved source/platform mapping values.
