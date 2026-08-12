# BE-02 Implementation Report

## 1. Repository state

Repository: `thoth-pub/thoth`
Workflow: STANDARD
Base branch: `develop`
Base commit: `1c752a522f7048963efde00b50565379d7c14b4d` (PR #788 merge commit; verified live before any edit, `0` commits added since authorization)
PR target: `develop`
Programme integration branch: None
Task branch: `feature/publisher-services/be-02` (created from exactly the authorized base)
Head commit: recorded on the pull request; see section 3 for the commit series
Pull request: [#805](https://github.com/thoth-pub/thoth/pull/805) - OPEN, DRAFT, UNMERGED
Expected branch deletion after merge: YES
Final programme PR required: NO
Implementing model: Claude Opus 5 (`claude-opus-5`), implementation agent
Reasoning level: HIGH / maximum practical
Independent reviewer/model: NOT PERFORMED BY THE IMPLEMENTATION AGENT - a different agent/model must review the exact head

### 1.1 Preflight record

| Item | Observed |
|---|---|
| `origin/develop` | `1c752a522f7048963efde00b50565379d7c14b4d` |
| `feature/publisher-services/be-02` at start | `1c752a522f7048963efde00b50565379d7c14b4d` |
| Working tree at start | clean |
| Ahead / behind `develop` at start | `+0 / -0` |
| Commits added to `develop` since authorization | `0` |
| Commits already on the implementation branch | `0` |
| Implementation PR already open | none |
| PR #799 | OPEN, DRAFT, untouched |

No rebase, force-push or history rewrite was performed.

## 2. Scope confirmation

Approved specification: [`docs/engineering/ai-delivery/tasks/BE-02.md`](../tasks/BE-02.md), merged
through PR [#788](https://github.com/thoth-pub/thoth/pull/788) as `1c752a52`.

Implemented objective: the complete approved BE-02 specification as an inactive additive
foundation - the closed `DistributionPlatform` inventory, the PostgreSQL enum, the
`publisher_distribution_platform` relation, the assignment activation lifecycle, linked
OAPEN/DOAB normalization, code-owned exhaustive descriptors, the four public GraphQL read
surfaces, and the first production adoption of the ADR-0007 request-local non-cached
DataLoader.

Out-of-scope changes made: NONE.

Explicit non-goals honoured: no BE-03, BE-04, MIG-01, Publisher Services application/UI
work, distribution worker, distribution job, dissemination activation, OAI implementation,
Thoth Metrics change, mutation-guard change, `OBSERVE`/`ENFORCE` transition, PR #799 action,
Juniper upgrade, `async-graphql` migration, ADR-0006 A2 machinery or new general batching
architecture. No `thoth-app` change. No Diesel CLI, root `diesel.toml` or schema
synchronisation subsystem was introduced.

## 3. Commits

- `26bf83fb` - `feat(publisher-services): add BE-02 distribution platform persistence model`
- `4da22a28` - `feat(publisher-services): add BE-02 public GraphQL contract and DataLoader`
- a third commit records the changelog, tracker and this report.

Three coherent bounded commits were used instead of the suggested six, because each of the
three is independently reviewable and compiles on its own; splitting further would have
separated a module from its own test file.

## 4. Files changed

```text
thoth-api/migrations/20260812_v1.7.0/up.sql                         |   45 +
thoth-api/migrations/20260812_v1.7.0/down.sql                       |    3 +
thoth-api/src/schema.rs                                             |   22 +
thoth-api/src/model/mod.rs                                          |    1 +
thoth-api/src/model/publisher_distribution_platform/mod.rs          |  607 +
thoth-api/src/model/publisher_distribution_platform/crud.rs         |  263 +
thoth-api/src/model/publisher_distribution_platform/tests.rs        | 1221 +
thoth-api/src/model/publisher/crud.rs                               |  112 +
thoth-api/src/graphql/dataloader.rs                                 |  185 +-
thoth-api/src/graphql/dataloader/fixture.rs                         |   11 +-
thoth-api/src/graphql/model.rs                                      |   90 +-
thoth-api/src/graphql/query.rs                                      |   44 +
thoth-api/src/graphql/mod.rs                                        |    2 +
thoth-api/src/graphql/distribution_platform_tests.rs                | 1269 +
thoth-errors/src/lib.rs                                             |    2 +
CHANGELOG.md
docs/publisher-services/task-status.md
docs/engineering/ai-delivery/implementation-reports/BE-02-implementation-report.md
```

Material files:

- `thoth-api/migrations/20260812_v1.7.0/up.sql`
  - reason: create the approved PostgreSQL enum, table, constraints, partial index and trigger.
  - behavioural effect: additive schema only; inserts no data and creates zero assignment rows.
- `thoth-api/migrations/20260812_v1.7.0/down.sql`
  - reason: migration-chain reversibility evidence.
  - behavioural effect: drops the table before the enum type; not the operational rollback path.
- `thoth-api/src/schema.rs`
  - reason: repository-authoritative Diesel contract, edited manually and atomically per ADR-0003 Architecture A.
  - behavioural effect: adds the `DistributionPlatform` SQL type, the composite-key table, one `joinable!` and one `allow_tables_to_appear_in_same_query!` entry. No unrelated reformatting.
- `thoth-api/src/model/publisher_distribution_platform/mod.rs`
  - reason: closed inventory, GraphQL/internal descriptor vocabularies, code-owned descriptors, persisted row and public projections.
  - behavioural effect: descriptor coverage is compile-time exhaustive; no default, fallback or wildcard exists.
- `thoth-api/src/model/publisher_distribution_platform/crud.rs`
  - reason: the only supported domain writes and the set-based read used by the loader.
  - behavioural effect: six-transition lifecycle, linked normalization, publisher-row locking, fail-closed non-assignable guard. `Crud` is deliberately not implemented.
- `thoth-api/src/model/publisher/crud.rs`
  - reason: reverse publisher lookup and count by enabled platform.
  - behavioural effect: two additive set-based join queries with deterministic ordering; existing `Crud` behaviour is untouched.
- `thoth-api/src/graphql/dataloader.rs`
  - reason: first production loader added to the existing `RequestLoaders` bundle.
  - behavioural effect: `RequestLoaders::for_request` now takes the pool and constructs the typed assignment loader; `configured_loader`, `SharedBatchError` and `FieldErrorConvention` become production code paths.
- `thoth-api/src/graphql/dataloader/fixture.rs`
  - reason: test-only `SqlProbe::captured_statements` and `BatchStats::record` visibility for field-specific SQL classification.
  - behavioural effect: test infrastructure only; existing foundation tests are unchanged.
- `thoth-api/src/graphql/model.rs`
  - reason: `Context` wiring, the two new GraphQL object types and the loader-backed `Publisher.distributionPlatforms` resolver.
  - behavioural effect: one new field on `Publisher`; no existing field changed.
- `thoth-api/src/graphql/query.rs`
  - reason: the three new public root query fields.
  - behavioural effect: additive; no existing query changed.
- `thoth-errors/src/lib.rs`
  - reason: one new stable variant `DistributionPlatformNotAssignable(String)`.
  - behavioural effect: appended to an enum documented as not exhaustively matched; the catch-all `IntoFieldError` and `ResponseError` arms already cover it, so no consumer breaks.

## 5. Implementation decisions

Decisions made within the approved design:

1. The persisted row type is `PublisherDistributionPlatform` and the publicly exposed
   projection is `PublisherDistributionPlatformAssignment { platform, enabled_at }`. The
   loader's value type is the projection, so activation identity and disabled history
   cannot reach a resolver by accident.
2. The transition timestamp is read once per transaction with
   `SELECT CURRENT_TIMESTAMP` and reused for every row written by that transition, giving
   provably identical `enabled_at`/`disabled_at` values across a linked pair. This is the
   transaction timestamp the specification requires, not `clock_timestamp()`.
3. `linked_members()` returns canonical declaration order, so a linked write always writes
   OAPEN before DOAB. This makes the injected second-row rollback test deterministic.
4. The reverse lookup applies the `publisher_id ASC` tie-breaker uniformly through the
   existing `apply_directional_order!` macro, including when the primary sort field is
   already `publisher_id`. That one case emits a redundant but harmless secondary sort key;
   uniformity was preferred over a special case so the determinism rule holds everywhere.
5. Loader-backed SQL is classified in tests by its `FROM` clause rather than by table name,
   because `publishersByDistributionPlatform` legitimately reaches the same table through
   an `INNER JOIN`. Root and child statement counts are therefore reported separately.
6. Batch-size observation uses a `#[cfg(all(test, feature = "backend"))]`-only `stats`
   field on the production batcher. The constructor, configuration, batcher and SQL under
   measurement are the production ones; only the counter is test-only.
7. `SqlProbe` (the ADR-0007 foundation's Diesel instrumentation) was reused rather than
   duplicated, with one additive unfiltered accessor.

Deviations from the specification: NONE.

Two places where the specification's wording met a repository fact are recorded in
section 13 rather than treated as deviations: the impossibility of duplicate publisher
names, and Juniper's index-free serialized error path.

## 6. Database and migration effects

Migration added: YES

- migration files: `thoth-api/migrations/20260812_v1.7.0/up.sql`, `down.sql`
- schema effect: one PostgreSQL enum `public.distribution_platform` (17 labels in canonical
  order); one table `publisher_distribution_platform` with composite primary key
  `(publisher_id, platform)`, `ON DELETE CASCADE` FK to `publisher(publisher_id)`, the named
  `publisher_distribution_platform_enabled_state_check`, `enabled`/`activation_id`/`enabled_at`
  `NOT NULL` with no `enabled` default, `disabled_at` as the only nullable lifecycle field, and
  `created_at`/`updated_at` defaulting to `CURRENT_TIMESTAMP`; one partial index
  `publisher_distribution_platform_enabled_idx` on `(platform, publisher_id) WHERE enabled`; one
  `set_updated_at` trigger via `diesel_manage_updated_at`.
- existing-data effect: none. The migration alters no existing table and inserts no data.
- locking/downtime: see section 6.3. No production duration is claimed.
- empty database result: see section 6.1.
- populated database result: see section 6.2.
- rollback/forward repair: the committed `down.sql` is migration-chain reversibility
  evidence. The specified operational response after an environment migration is retained
  foundation plus forward repair.
- idempotency: the migration is a single forward DDL step managed by the embedded
  `diesel_migrations` runner; re-running is a no-op because the version is recorded.

All evidence below was produced against **disposable local PostgreSQL 17.10 databases**.
No production database, credential, configuration or workflow was accessed.

### 6.1 Empty-database evidence

Database `thoth_be02_empty`, created empty.

Command:

```text
DATABASE_URL=postgres://.../thoth_be02_empty cargo run migrate
```

Result: exit `0`; `__diesel_schema_migrations` head becomes `20260812`.

Catalog verification:

```text
enum labels (pg_enum, by enumsortorder):
  1 INTERNET_ARCHIVE   2 OAPEN     3 DOAB      4 SCIENCE_OPEN
  5 CAMBRIDGE_UNIVERSITY_LIBRARY   6 CROSSREF  7 FIGSHARE  8 ZENODO
  9 PROJECT_MUSE      10 JSTOR    11 EBSCO_HOST
 12 PROQUEST_EBOOK_CENTRAL        13 GOOGLE_PLAY
 14 BKCI              15 OCLC_KB  16 EX_LIBRIS_KB          17 JISC_NBK
 (17 rows)

columns (information_schema.columns):
 1 publisher_id  uuid                        NOT NULL  no default
 2 platform      distribution_platform       NOT NULL  no default
 3 enabled       boolean                     NOT NULL  no default
 4 activation_id uuid                        NOT NULL  no default
 5 enabled_at    timestamp with time zone    NOT NULL  no default
 6 disabled_at   timestamp with time zone     NULL     no default
 7 created_at    timestamp with time zone    NOT NULL  CURRENT_TIMESTAMP
 8 updated_at    timestamp with time zone    NOT NULL  CURRENT_TIMESTAMP

constraints (pg_constraint):
 publisher_distribution_platform_enabled_state_check  c  CHECK (((enabled AND (disabled_at IS NULL)) OR ((NOT enabled) AND (disabled_at IS NOT NULL))))
 publisher_distribution_platform_publisher_id_fkey    f  FOREIGN KEY (publisher_id) REFERENCES publisher(publisher_id) ON DELETE CASCADE
 publisher_distribution_platform_pkey                 p  PRIMARY KEY (publisher_id, platform)

indexes (pg_indexes):
 publisher_distribution_platform_enabled_idx  CREATE INDEX ... USING btree (platform, publisher_id) WHERE enabled
 publisher_distribution_platform_pkey         CREATE UNIQUE INDEX ... USING btree (publisher_id, platform)

triggers (pg_trigger, non-internal):
 set_updated_at  CREATE TRIGGER set_updated_at BEFORE UPDATE ON public.publisher_distribution_platform FOR EACH ROW EXECUTE FUNCTION diesel_set_updated_at()

row count:
 0
```

Revert:

```text
DATABASE_URL=... cargo run migrate --revert
```

Result: exit `0`; public tables `1` (only `__diesel_schema_migrations`),
`distribution_platform` type present `0`, `publisher_distribution_platform` present `0`,
migration rows `0`.

Reapply:

```text
DATABASE_URL=... cargo run migrate
```

Result: exit `0`; `enum_labels 17 | columns 8 | constraints 3 | indexes 2 | rows 0`.

Full-history revert is migration-chain reversibility evidence, not authorization for a
destructive operational rollback.

### 6.2 Representative populated-database evidence

Database `thoth_be02_pop`, migrated to the **pre-BE-02** head (`20260805`, the BE-01
package migration) and then seeded with representative records: 4 publishers covering all
four BE-01 packages (`OASIS`, `OBELISK`, `SPHINX`, `PYRAMID`), with and without
shortname/URL/Zitadel ID, plus 2 imprints, 1 work, 1 title, 1 publisher-history row and 1
contact.

Pre-migration baseline:

```text
row census (non-zero): contact=1 | imprint=2 | publisher=4 | publisher_history=1 | title=1 | work=1
publisher fingerprint (md5 over id+name+shortname+url+zitadel+package+created_at+updated_at):
  e7d273d3781c00d160b5f5e07e53984e
related fingerprint (imprint+work+title+publisher_history+contact):
  f04616aba4c5bb3dbb02ff8e6b0c6afc
```

Forward migration through the embedded runner: exit `0`; migration head `20260812`.

Post-migration:

```text
row census: unchanged, plus publisher_distribution_platform=0
publisher fingerprint: e7d273d3781c00d160b5f5e07e53984e   (identical)
related fingerprint:   f04616aba4c5bb3dbb02ff8e6b0c6afc   (identical)
publisher_distribution_platform rows: 0
```

All existing publishers, package values, history and unrelated rows are preserved exactly,
and the migration created zero assignment rows and no other record.

Constraint behaviour observed directly on the populated database:

```text
INSERT enabled=true  with disabled_at set     -> ERROR: violates check constraint "publisher_distribution_platform_enabled_state_check"
INSERT enabled=false with disabled_at NULL    -> ERROR: violates check constraint "publisher_distribution_platform_enabled_state_check"
INSERT with unknown publisher_id              -> ERROR: violates foreign key constraint "publisher_distribution_platform_publisher_id_fkey"
INSERT duplicate (publisher_id, platform)     -> ERROR: duplicate key value violates unique constraint "publisher_distribution_platform_pkey"
INSERT same publisher, different platform     -> accepted (2 rows)
DELETE FROM publisher                         -> assignments for that publisher: before=2, after=0 (ON DELETE CASCADE)
EXPLAIN ... WHERE platform='OAPEN' AND enabled -> Bitmap Index Scan on publisher_distribution_platform_enabled_idx
```

No full-history destructive revert was run against the populated preservation fixture.

### 6.3 Lock evidence

PostgreSQL major version: **17.10 (Homebrew)**. Disposable populated database
`thoth_be02_pop`, with the FK referencing a populated `publisher` table.

Method: the exact `up.sql` was executed inside an open transaction in session A, which then
held the transaction open; session B observed `pg_locks` and attempted a read and a write.

Locks held by the migration session:

```text
publisher                        AccessShareLock          granted
publisher                        ShareRowExclusiveLock    granted
publisher_distribution_platform  AccessExclusiveLock      granted
publisher_distribution_platform  AccessShareLock          granted
publisher_distribution_platform  ShareLock                granted
publisher_distribution_platform  ShareRowExclusiveLock    granted
```

Effect on `publisher`, observed from a second session while the lock was held:

```text
SELECT count(*) FROM publisher;                     -> 4          (reads unaffected)
SET lock_timeout='3s'; UPDATE publisher SET ...;    -> ERROR: canceling statement due to lock timeout
                                                       (row verified unchanged afterwards)
```

Conclusion: establishing the foreign key takes `SHARE ROW EXCLUSIVE` on the referenced
`publisher` table, which is exactly what the specification predicted. Publisher reads using
`ACCESS SHARE` remain available; ordinary publisher writes using `ROW EXCLUSIVE` are blocked
while the lock is held. The child table is created empty, so there is **no child-row FK
validation scan**; the operational concern is lock acquisition queueing behind conflicting
transactions, not scan duration.

Lock-timeout conclusion: a migration-session `lock_timeout` is **appropriate** for the
environment migration, because the DDL must acquire `SHARE ROW EXCLUSIVE` on a table that
serves live writes. A bounded `lock_timeout` makes the migration fail fast and retryable
instead of queueing behind a long transaction and holding back every subsequent publisher
write. Setting it is a release-task decision, not part of this repository change.

**No production lock duration is claimed and no production database was observed.**

## 7. API and compatibility effects

GraphQL/API changes: exactly the additive inventory of specification section 12.1 -
3 root query fields, 1 field on `Publisher`, 2 object types, 3 enums, and 0 mutations,
inputs, scalars and interfaces. Existing `PublisherOrderBy`, `Direction` and `Timestamp`
are reused.

Generated schema/client updates: the authoritative SDL is generated by
`thoth-client/build.rs` into `thoth-client/assets/schema.graphql`, which is
build-generated and gitignored. It was therefore compared by building both revisions:

```text
base 1c752a52  (built in a throwaway git worktree)
  160799 bytes  sha256 1e08b46b565ef719c404bbe6b3131e6a733df09c7abdc4538b66c2b24d2d899c
head
  164152 bytes  sha256 0ba96aa1aa15006e8bf8b9f4a711f9e493eec4ce51911eebc32fb99d1ba53a67

diff base -> head:  0 lines removed, 63 lines added
```

The 63 added lines are exactly: the `BackCatalogueBehaviour` enum (3 values), the
`DistributionPlatform` enum (17 values), the `DistributionPlatformGroup` enum (1 value),
the `DistributionPlatformOption` type (5 fields), the
`PublisherDistributionPlatformAssignment` type (2 fields), the
`Publisher.distributionPlatforms` field, and the three root query fields. No existing type,
field, argument, nullability or enum value is removed or changed.

The three internal Rust enums (`AssignmentAvailability`, `MechanismReadiness`,
`DistributionAdapterProfile`) are absent from SDL, as asserted by a dedicated test, along
with adapter/feed profile codes, `activationId`, `disabledAt`, `subscriptionPackage` and
any BE-03 configuration type.

Backwards compatibility: additive only. `thoth-client/queries.graphql` is unchanged, so no
generated client type changes; `thoth-client` compiles and its tests pass against the new
schema, and `thoth-export-server` compiles with its 144 tests passing. This is a verified
result, not an assumption. No `thoth-app` change is made.

Deprecations: none.

Cross-repository dependencies: none introduced. No downstream repository is asked to guess
an unmerged schema.

## 8. Authorization and security

Authorization paths changed: none. All four BE-02 read surfaces are intentionally
public/anonymous, matching the current production read architecture and the approved design
("platform assignments are public; package values are not"). No new role, policy branch or
`src/policy.rs` change is introduced.

Roles/scopes involved: none. The loader key requires no user scope because the field is
intentionally public. This creates no precedent for a protected loader omitting
authorization context, and ADR-0007 section 4.11 continues to govern that case.

Negative authorization tests: BE-02 adds no protected operation, so the applicable negative
evidence is **non-exposure** rather than role rejection, and it is asserted directly against
the generated SDL and against responses:

- `activationId`, `disabledAt`, `enabled`, `createdAt`, `updatedAt` and `publisherId` are
  absent from `PublisherDistributionPlatformAssignment`, which carries exactly `platform`
  and `enabledAt`;
- `adapterProfile`, `mechanismReadiness`, `endpoint`, `host`, `bucket`, `account`,
  `credential` and `secret` are absent from `DistributionPlatformOption`;
- `subscriptionPackage`, `PublisherServiceConfiguration` and
  `replacePublisherServiceConfiguration` are absent from the whole schema;
- no distribution mutation, input, scalar or interface exists;
- disabled assignments are excluded from every public read.

Positive anonymous access to all four surfaces is asserted with a context whose
`user` is `None`.

Secret or personal-data handling: none. No new log statement is added, and no token,
credential, personal datum, sensitive URL or unbounded payload is recorded.

Security limitations: the assignment surface is public by approved design. A caller can
therefore learn which publishers have enabled which destinations. That is the approved
BE-02 exposure decision, recorded here so review can confirm it deliberately rather than
discover it.

## 9. Tests and checks

Environment: local disposable PostgreSQL 17.10 and local Redis. `TEST_DATABASE_URL` and
`THOTH_EXPORT_API` point at local services. No test was pointed at production or any
shared service.

### Formatting

Command:

```text
cargo fmt --all -- --check
```

Result:

```text
no diff reported (clean)
```

### Whitespace

Command:

```text
git diff --check
```

Result:

```text
exit 0, no output
```

### Build / type check

Command:

```text
cargo check --workspace
```

Result:

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 56.95s
no errors, no thoth warnings
```

### Lint/static analysis

Command:

```text
cargo clippy --all --all-targets --all-features -- -D warnings
```

Result:

```text
no clippy errors or warnings
(only the pre-existing dependency note: proc-macro-error2 v2.0.1 future-incompatibility,
 which is present at the authorized base and unrelated to this change)
```

### Unit and integration tests (`thoth-api`)

Command:

```text
cargo test -p thoth-api --features backend
```

Result:

```text
test result: ok. 976 passed; 0 failed; 0 ignored
(909 at the authorized base; 67 new BE-02 tests)
```

### Focused BE-02 model/lifecycle tests

Command:

```text
cargo test -p thoth-api --features backend publisher_distribution_platform
```

Result:

```text
test result: ok. 40 passed; 0 failed; 0 ignored; finished in 2.86s
```

### Focused BE-02 GraphQL/DataLoader tests

Command:

```text
cargo test -p thoth-api --features backend distribution_platform_tests
```

Result:

```text
test result: ok. 27 passed; 0 failed; 0 ignored; finished in 3.34s
```

### Workspace regression

Command:

```text
cargo test --workspace
```

Result:

```text
all suites passed; 0 failed across the workspace
  thoth-api lib            976 passed
  thoth-api graphql_permissions  13 passed
  thoth-api-server           3 passed
  thoth-client               4 passed / 11 passed
  thoth-export-server      144 passed / 6 passed / 2 passed
  thoth-errors              14 passed
  (8 ignored, pre-existing)
```

### Migration commands

Commands:

```text
DATABASE_URL=<disposable empty DB>     cargo run migrate
DATABASE_URL=<disposable empty DB>     cargo run migrate --revert
DATABASE_URL=<disposable empty DB>     cargo run migrate            (reapply)
DATABASE_URL=<disposable populated DB> cargo run migrate
```

Results: all exit `0`; catalog and preservation evidence in sections 6.1 and 6.2.

### 9.1 Evidence coverage against the specification test plan

| Specification requirement | Evidence |
|---|---|
| 18.1 enum conversion | `inventory_is_exactly_seventeen_values_in_canonical_order`, `inventory_contains_no_fallback_or_excluded_value`, `string_conversion_round_trips_and_rejects_unknown_values`, `serde_round_trips_and_rejects_unknown_values`, `there_is_no_default_distribution_platform`, `distribution_platform_declares_no_metric_platform_conversion`, plus the catalog label/order check in 6.1 |
| 18.2 descriptors | `every_platform_has_exactly_one_descriptor_matching_the_approved_table` (full section 8 table), `descriptor_lookup_returns_the_same_static_without_allocating`, `only_oapen_and_doab_belong_to_a_linked_group`, `options_expose_descriptor_metadata_in_canonical_order` |
| 18.3 lifecycle/row invariants | `absent_to_enabled_inserts_one_activated_row`, `enabled_to_enabled_is_an_idempotent_no_op_that_moves_no_timestamp`, `enabled_to_disabled_retains_the_row_and_its_activation`, `disabled_to_disabled_is_an_idempotent_no_op_that_moves_no_timestamp`, `disabled_to_enabled_creates_a_new_activation_on_the_retained_row`, `absent_to_disabled_succeeds_without_creating_a_never_activated_row`, `transition_sequence_never_violates_the_row_invariant`, `database_rejects_rows_whose_enabled_flag_contradicts_disabled_at`, `deleting_a_publisher_cascades_to_its_assignments`, `concurrent_enables_of_one_platform_produce_one_activation`, `concurrent_enable_and_disable_serialize_to_a_valid_state`, `transitions_for_different_publishers_are_independent` |
| 18.4 linked OAPEN/DOAB | `enabling_either_linked_member_enables_both_with_one_shared_activation`, `disabling_either_linked_member_disables_both`, `linked_enable_is_a_no_op_only_when_the_pair_is_normalized_fully_enabled`, `linked_disable_is_a_no_op_when_no_member_is_enabled`, `linked_enable_repairs_a_one_sided_pair`, `linked_enable_repairs_a_disabled_member`, **`linked_enable_normalizes_a_split_activation_pair`**, **`linked_enable_normalizes_a_split_enabled_at_pair`**, `a_failure_writing_the_second_linked_row_rolls_the_whole_transition_back`, `concurrent_linked_enables_produce_one_shared_group_activation` |
| 18.5 OCLC / Ex Libris | `oclc_and_ex_libris_share_a_profile_but_are_not_linked`, `oclc_and_ex_libris_assignments_and_activations_are_independent` |
| 18.6 JISC_NBK | `jisc_nbk_is_the_only_inactive_non_assignable_platform`, `enabling_jisc_nbk_fails_closed_before_any_write`, `empty_reverse_reads_are_successful_and_never_broaden_scope` (JISC aliases), option `assignable: false` in `options_return_seventeen_descriptors_in_canonical_order` |
| 18.7 GraphQL contract | `options_return_seventeen_descriptors_in_canonical_order`, `publisher_field_returns_enabled_assignments_only_in_canonical_order`, `publisher_field_is_an_empty_list_without_assignments`, `reverse_lookup_and_count_cover_the_same_enabled_population`, `reverse_lookup_paginates_deterministically_through_ordering_ties`, `all_four_surfaces_answer_anonymously`, `an_invalid_platform_value_fails_input_coercion`, `sdl_adds_exactly_the_approved_public_inventory`, `sdl_exposes_no_internal_or_protected_distribution_state`, `sdl_adds_no_distribution_mutation_input_scalar_or_interface` |
| 18.7a loader configuration/scheduler | `the_production_bundle_uses_the_explicit_200_10_configuration`, `the_field_resolver_is_loader_first_and_uses_try_load_only`, `assignment_sql_count_tracks_configured_chunks_not_parent_count` (1/100/200/201), `five_hundred_ready_keys_dispatch_in_three_configured_chunks`, `batch_boundaries_hold_on_a_current_thread_runtime` |
| 18.7b real SQL query counts | `publishers_shape_250_parents_use_two_set_based_assignment_statements`, `reverse_lookup_shape_250_parents_use_two_set_based_assignment_statements` |
| 18.7c request-local/non-cached | `two_request_contexts_share_no_assignment_loader_state`, `a_completed_load_is_not_cached_within_one_request`, `pending_duplicate_keys_coalesce_into_one_dispatch` |
| 18.7d failure equivalence | `a_backend_failure_errors_every_key_with_no_retry_and_no_fallback_sql`, `loader_failure_matches_the_direct_baseline_error_shape`, `loader_and_direct_paths_agree_on_successful_membership_and_order`, `the_batch_is_total_for_a_publisher_with_no_enabled_assignments` |
| 18.8 database/index/catalog | sections 6.1-6.3 |
| 18.9 regression | `cargo test --workspace`, section 9 |

## 10. Manual verification

Environment: local disposable PostgreSQL 17.10 (`thoth_be02_empty`, `thoth_be02_pop`, and
the standard local test database), local Redis. No production or shared service.

Steps and observed results are recorded inline in sections 6.1, 6.2, 6.3, 7 and 11:
catalog verification of the migrated schema, populated-database preservation fingerprints,
`pg_locks` observation with a concurrent reader and a concurrent writer, and the
base-versus-head SDL diff produced from two real builds.

### 10.1 DataLoader implementation record

| Contract item | Implementation |
|---|---|
| `RequestLoaders` change | one typed field `publisher_distribution_platforms` added to the existing request-local bundle; no second subsystem |
| typed key | `Uuid` = `publisher_id`; no argument/result-shape dimension exists, and a future result-changing argument requires revisiting the key contract |
| configuration | `configured_loader(...)` -> `max_batch_size = 200`, `yield_count = 10`, asserted explicitly |
| load API | `try_load` only; a source assertion proves the resolver contains no `.load(` |
| loader-first | the resolver's first `.await` is the target `try_load`, asserted by a source check and corroborated by the measured chunking |
| set-based SQL | one statement per chunk: `... FROM "publisher_distribution_platform" WHERE (("publisher_distribution_platform"."publisher_id" = ANY($1)) AND ("publisher_distribution_platform"."enabled" = $2)) ORDER BY "publisher_distribution_platform"."publisher_id" ASC, "publisher_distribution_platform"."platform" ASC` |
| `spawn_blocking` ownership | only `Arc<PgPool>` and an owned `Vec<Uuid>` move in; the connection is acquired, used and dropped inside the closure; only the closure result is awaited |
| totality | every requested key is seeded with `Ok(Vec::new())` before grouping; a failure replaces every key's value with the shared error; no key is ever omitted |
| failure behaviour | one error per key in the failed chunk, conventional message-only Juniper shape via `SharedBatchError`, no retry, no per-parent fallback SQL |
| ordering | rows arrive ordered by `(publisher_id, platform)`; `platform` is a PostgreSQL enum, so its sort order is the canonical declaration order |

### 10.2 Query-count evidence

Measured through Diesel connection instrumentation (`SqlProbe`), an external observation of
the statements actually sent, not an implementation-side batch counter. Root and child
statements are classified by `FROM` clause because the reverse root query legitimately
joins the same table.

Reference case, `publishers` parent shape:

```text
250 parent publishers, selection includes distributionPlatforms
  loader chunks                       [200, 50]
  assignment SQL statements           2      (both `= ANY`, both enabled-filtered, both ORDER BY)
  per-parent assignment statements    0
  root publisher statements           1
```

Reference case, `publishersByDistributionPlatform` parent shape:

```text
250 parent publishers, selection includes distributionPlatforms
  loader chunks                       [200, 50]
  assignment SQL statements           2      (both `= ANY`)
  per-parent assignment statements    0
  set-based reverse-lookup statements 1
```

Boundaries, production constructor and production batcher, `publishers` parent shape:

```text
N=1    chunks [1]             assignment statements 1
N=100  chunks [100]           assignment statements 1
N=200  chunks [200]           assignment statements 1
N=201  chunks [200, 1]        assignment statements 2   (multi-thread runtime)
N=201  chunks [200, 1]        assignment statements 2   (current-thread runtime)
N=500  chunks [200, 200, 100] assignment statements 3
```

The statement count tracks `ceil(N / 200)`, not `N`. These are the shapes the loader-first
resolver actually creates; no universal guarantee is claimed for arbitrary scheduling, and
the foundation's own delayed-cohort fragmentation characterization remains the reason
loader-first is binding.

Membership, ordering, empty-list, request-isolation and non-caching correctness are asserted
separately from the counts, as are backend failure, absence of retry and absence of fallback.

## 11. CI

CI status: recorded on the pull request after the exact head was pushed. Every relevant
workflow/job is listed individually as PASS / FAIL / SKIPPED in the PR evidence rather than
summarised as a single workflow conclusion.

No workflow was manually dispatched, and no production or write-capable workflow was
triggered.

## 12. Rollout and rollback

Initial state after merge: only Git history changes. No environment is modified, no
migration runs, and no assignment row exists anywhere.

Activation required: yes, and separately authorized at every step:

```text
1. implementation PR merge into develop
2. environment deployment
3. environment migration execution
4. assignment creation/backfill
5. runtime consumer/dissemination cutover
```

None of these authorizes the next.

Feature flag/configuration: none. No BE-02 feature flag is needed to carry the inactive
foundation because no distribution consumer is activated. That does not waive release or
deployment controls for the public additive GraphQL surface or for the migration.

Migration sequence: deploy the code, then run the migration through the embedded runner,
under the current release controls and CG-13. A bounded migration-session `lock_timeout` is
recommended (section 6.3).

After a separately authorized deployment and migration in an environment with zero
assignment rows: `distributionPlatformOptions` returns 17 code-owned descriptors,
`Publisher.distributionPlatforms` returns empty lists, `publishersByDistributionPlatform`
returns empty pages, `publisherCountByDistributionPlatform` returns 0, no distribution job
exists and dissemination is unchanged.

Rollback/disable procedure: before any environment adoption, a bounded repository revert is
possible subject to normal review, and must not silently alter ADR-0004 or ADR-0007
authority. After an environment migration, the specified response to a defect is **retained
foundation plus forward repair**: retain the enum type, table, constraints, index, trigger
and any stored rows, retain the persisted enum/domain types needed to read stored data, keep
downstream consumers inactive, and forward-fix under review. The committed down migration is
reversibility evidence, not the normal production rollback. Dropping populated assignment
state is destructive and requires a separate CTO-approved task with a data-preservation plan.

Monitoring required: none new for merge. During any later authorized preview/staging
release, observe existing GraphQL latency, error rate and availability plus database and
query behaviour for the new public field before wider release. This task introduces no new
telemetry subsystem; if existing telemetry cannot support a safe release assessment, that is
a release-task stop condition.

## 13. Known limitations and deferred work

- `publisher_uniq_idx` is a unique index on `lower(publisher_name)`, so literally duplicate
  publisher names cannot exist in this schema. The specification's "duplicate publisher
  names and ordering ties" requirement is therefore evidenced by the equivalent achievable
  tie: every fixture publisher has a NULL `publisher_shortname`, so the primary sort key is
  identical across the page set and only the mandatory `publisher_id ASC` tie-breaker can
  make pagination deterministic. Both sort directions are asserted.
- Juniper's serialized error `path` for a list element does not include the element index,
  so the direct-versus-loader equivalence test asserts the stable
  `["publishers", "distributionPlatforms"]` path. Message, path and extensions convention
  are asserted equal between the two execution paths, and `location` is not asserted because
  it is not stable across the two schemas.
- For the same reason, GraphQL-level error counts for a multi-key failed batch are not a
  reliable per-key signal. Per-key failure totality is therefore asserted at the loader
  level (three concurrent `try_load` calls, each returning a present-but-`Err` value from
  one dispatch), with the GraphQL level asserting null propagation, non-empty errors, a
  single dispatch and zero fallback SQL.
- `MetricPlatform` does not exist in the repository at this base. "No shared universal enum
  and no name-based conversion" is therefore evidenced by construction plus a source
  assertion, not by a cross-type test.
- Batch-size observation uses a `cfg(test)`-only field on the production batcher. The
  constructor, configuration, batcher and SQL under measurement are the production ones, but
  a reviewer should confirm that this observation hook is acceptable rather than assume it.
- The lock evidence is empirical on a disposable PostgreSQL 17.10 database. Behaviour under
  production concurrency, and the choice of a concrete `lock_timeout` value, belong to the
  separately authorized release activity.
- BE-02 adds no way to create an assignment through the API. The lifecycle functions are
  domain-internal, which is intentional: BE-03 owns the protected mutation surface.

## 14. Unresolved issues

NONE arising from this implementation.

Pre-existing and unchanged: CG-13 (production migration/release execution control) remains
open; the pinned-Juniper duplicate top-level mutation execution finding remains a live,
separately controlled concern; PR #799 / `THOTH-GQL-OPS-03` remains open and untouched. The
`proc-macro-error2 v2.0.1` future-incompatibility note is a dependency-level warning present
at the authorized base.

## 15. Agent self-assessment

The implementing agent may identify risks but may not issue the approval decision. This
implementation has **not** been independently reviewed.

Suggested review focus:

1. The linked-normalization predicate `is_normalized_fully_enabled` and the
   insert-on-conflict write in `enable`: confirm that no supported path can commit a
   one-sided, split-activation or split-timestamp enabled pair, including under the
   publisher-row lock.
2. The transaction-timestamp choice: confirm that reading `CURRENT_TIMESTAMP` once per
   transaction and reusing it satisfies the "one transaction timestamp per state transition"
   rule as intended, and that `updated_at` movement through the trigger is acceptable.
3. The SQL classification used for query-count evidence: confirm that classifying child
   statements by `FROM "publisher_distribution_platform"` cannot miscount the reverse
   lookup's `INNER JOIN` statement in either direction.
4. The `cfg(test)`-only `stats` field on the production batcher: confirm this is acceptable
   scaffolding rather than production contamination.
5. The uniform `publisher_id ASC` tie-breaker, including the redundant secondary sort when
   the primary field is already `publisher_id`.
6. The public exposure decision: confirm that publishing which publishers have enabled which
   destinations is the intended approved behaviour.
7. The SDL diff: confirm the 63 added lines are exactly the approved section 12.1 inventory
   and that nothing internal leaked.
8. The new `ThothError` variant's placement at the end of the enum and its interaction with
   the existing catch-all `IntoFieldError` and `ResponseError` arms.
