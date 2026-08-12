# BE-03 Implementation Report

## 1. Repository state

Repository: `thoth-pub/thoth`
Workflow: STANDARD
Base branch: `develop`
Base commit: `3b6b3a31f9358011f0c998015dfd0c2508380e83` (PR
[#808](https://github.com/thoth-pub/thoth/pull/808) merge commit; verified live
before any edit, `0` commits added since authorization)
PR target: `develop`
Programme integration branch: None
Task branch: `feature/publisher-services/be-03` (created by the control agent at
exactly the authorized base; not recreated by this agent)
Head commit: recorded on the pull request; see section 3 for the commit series
Pull request: [#809](https://github.com/thoth-pub/thoth/pull/809) - OPEN, DRAFT,
UNMERGED
Expected branch deletion after merge: YES
Final programme PR required: NO
Implementing model: Claude Opus 5 (`claude-opus-5`), implementation agent
Reasoning level: Extra High / xhigh
Independent reviewer/model: NOT PERFORMED BY THE IMPLEMENTATION AGENT - a
different agent/model must review the exact head

### 1.1 Preflight record

| Item | Observed |
|---|---|
| `origin/develop` | `3b6b3a31f9358011f0c998015dfd0c2508380e83` |
| `feature/publisher-services/be-03` at start | `3b6b3a31f9358011f0c998015dfd0c2508380e83` |
| Approved specification head `a3fc7064...` | ancestor of the base (`git merge-base --is-ancestor` succeeded) |
| Working tree at start | clean |
| Ahead / behind `develop` at start | `+0 / -0` |
| Commits added to `develop` since authorization | `0` (develop head equals the authorized base) |
| Competing BE-03 implementation PR | none (`gh pr list --state open` returned #806, #799, #752, #744, #742, #668 only) |
| PR #799 | OPEN, DRAFT, untouched |
| `thoth-api/migrations/20260813_v1.7.0` before `make migration` | did not exist |

No rebase, force-push or history rewrite was performed. `develop` did not move
during implementation, so no drift assessment was required.

## 2. Scope confirmation

Approved specification:
[`docs/engineering/ai-delivery/tasks/BE-03.md`](../tasks/BE-03.md), merged
through PR [#808](https://github.com/thoth-pub/thoth/pull/808) as `3b6b3a31`.
Both halves of the section 19 stop-condition 5 authority condition therefore
hold.

Implemented objective: the complete approved BE-03 specification as an inactive
additive foundation — the canonical configuration token, the closed audit-source
type and append-only audit table, the single authoritative service-configuration
write coordinator, the additive connection-scoped BE-02 lifecycle refactor, the
protected `PublisherServiceConfiguration` type with derived effective
capabilities, the owner-and-superuser read, the superuser-only staff report and
count, the superuser-only replace mutation, one new error variant and one new
`into_field_error` arm, and the section 18 test and evidence matrix.

Out-of-scope changes made: NONE.

Explicitly not done, and not authorized: merge; deployment; environment or
production migration execution; production access; MIG-01; assignment creation
or backfill; distribution activation; dissemination; BE-04; APP-01; APP-02;
`thoth-app` changes; shared publisher-trigger changes; `subscription_package`
storage relocation; `OBSERVE`/`ENFORCE` or mutation-guard changes; workflow
changes or `workflow_dispatch`; any action on PR
[#799](https://github.com/thoth-pub/thoth/pull/799).

## 3. Commits

- `[sha]` - `feat(publisher-services): implement BE-03 protected service configuration`

The exact SHA is recorded on the pull request under ADR-0005; the branch carries
one bounded implementation commit plus, if required, bounded remediation commits
appended without rebase, amend or force-push.

## 4. Files changed

- `thoth-api/migrations/20260813_v1.7.0/up.sql` (new)
  - reason: the additive BE-03 database foundation.
  - behavioural effect: adds `publisher.service_configuration_updated_at`, the
    `publisher_service_configuration_source` enum type and the
    `publisher_service_configuration_history` table with its primary key,
    `ON DELETE CASCADE` foreign key, named non-blank actor check constraint and
    composite index. Creates zero rows.
- `thoth-api/migrations/20260813_v1.7.0/down.sql` (new)
  - reason: reversibility evidence.
  - behavioural effect: drops the table, the type and the column.
- `thoth-api/src/schema.rs`
  - reason: ADR-0003 Architecture A — the repository-authoritative Diesel
    contract is edited manually and atomically with the migration.
  - behavioural effect: adds `sql_types::PublisherServiceConfigurationSource`,
    appends `service_configuration_updated_at` to the `publisher` table after
    `updated_at` so `Queryable` column order still matches the `Publisher`
    struct, adds the `publisher_service_configuration_history` table, its
    `joinable!` and its `allow_tables_to_appear_in_same_query!` entry. No
    unrelated schema reformatting.
- `thoth-api/src/model/publisher/mod.rs`
  - reason: the token is a `Publisher` column.
  - behavioural effect: appends `#[serde(default)] pub
    service_configuration_updated_at: Timestamp` after `updated_at`. No GraphQL
    field is added to the public `Publisher` type.
- `thoth-api/src/model/publisher_service_configuration/mod.rs` (new)
  - reason: the BE-03 domain module.
  - behavioural effect: the `PublisherServiceConfigurationSource` DB/GraphQL
    enum, the `PublisherServiceConfiguration`,
    `PublisherServiceConfigurationChange` and
    `PublisherServiceConfigurationSummary` projections, the audit row
    `Queryable`/`Insertable` pair, the bounded three-key
    `CanonicalServiceConfigurationState`, the
    `ServiceConfigurationWriteContext` and the
    `ReplacePublisherServiceConfigurationInput` GraphQL input.
- `thoth-api/src/model/publisher_service_configuration/crud.rs` (new)
  - reason: the canonical write coordinator and the staff-report queries.
  - behavioural effect: section 5 below.
- `thoth-api/src/model/publisher_service_configuration/tests.rs` (new)
  - reason: coordinator, audit, concurrency, linked-platform, trigger-cascade,
    report and migrated-contract evidence (45 tests).
- `thoth-api/src/model/publisher_distribution_platform/crud.rs`
  - reason: the additive connection-scoped lifecycle refactor of section 7.7.
  - behavioural effect: adds `AssignmentLifecycleOutcome`, extracts
    `enable_on`/`disable_on`, reduces the pool-level `enable`/`disable` to
    acquire-connection/open-transaction/delegate/discard, and widens
    `lock_publisher` to `pub(crate)`. Public signatures, semantics, ordering and
    error behaviour are unchanged.
- `thoth-api/src/model/publisher_distribution_platform/tests.rs`
  - reason: BE-02 regression evidence for the new primitives.
  - behavioural effect: **adds** two tests; no existing test's expectation is
    changed.
- `thoth-api/src/model/mod.rs`
  - reason: register the new model module.
- `thoth-api/src/graphql/model.rs`
  - reason: the protected GraphQL types.
  - behavioural effect: adds three `graphql_object` implementations. The public
    `Publisher` type is unchanged.
- `thoth-api/src/graphql/query.rs`
  - reason: the protected read and the superuser report/count.
- `thoth-api/src/graphql/mutation.rs`
  - reason: the superuser-only replace mutation and its authorize/build-context/
    delegate helper.
- `thoth-api/src/graphql/mod.rs`
  - reason: register the BE-03 GraphQL test module.
- `thoth-api/src/graphql/service_configuration_tests.rs` (new)
  - reason: authorization-matrix, capability, error-shape, query-count,
    loader-reuse and catalogue-scale evidence (23 tests).
- `thoth-api/src/graphql/tests.rs`
  - reason: **authorized SDL guard amendment** (section 12.3).
- `thoth-api/src/graphql/distribution_platform_tests.rs`
  - reason: **authorized SDL guard amendment** (section 12.3).
- `thoth-errors/src/lib.rs`
  - reason: exactly one new variant and exactly one new `into_field_error` arm.
- `CHANGELOG.md`, `docs/publisher-services/task-status.md`,
  `docs/engineering/ai-delivery/implementation-reports/BE-03-implementation-report.md`
  - reason: required control records.

## 5. Implementation decisions

Decisions taken within the approved design:

1. **Coordinator identity.** Name:
   `replace_publisher_service_configuration`. Module:
   `thoth-api/src/model/publisher_service_configuration/crud.rs`. Signature:

   ```rust
   pub(crate) fn replace_publisher_service_configuration(
       db: &PgPool,
       write_context: &ServiceConfigurationWriteContext<'_>,
       data: &ReplacePublisherServiceConfigurationInput,
   ) -> ThothResult<PublisherServiceConfiguration>
   ```

   It acquires one connection, opens exactly one transaction and executes
   specification section 7.3 steps 2–12 in a **private** module-local function
   `replace_in_transaction`. That helper is deliberately not `pub(crate)`: it is
   the coordinator's transaction body — the seam BE-04 will extend — and not a
   second entry point. The GraphQL mutation is its only production caller.
2. **Write context.** `ServiceConfigurationWriteContext { source, actor }` is a
   parameter. The resolver authorizes with `require_superuser()`, then builds
   `source = SUPERUSER_API` and `actor = PolicyContext::user_id()?`. The
   coordinator makes no authentication or authorization decision.
3. **`after_state` is read back from the database** after the lifecycle calls
   rather than assumed from the request. It equals the normalized desired set,
   and recording observed state is strictly more faithful for an audit row.
4. **Returned configuration.** The token `UPDATE` uses `RETURNING
   publisher::all_columns`, so the returned configuration carries the actual
   committed row — including the trigger-updated `publisher.updated_at` — with no
   extra statement. A true no-op returns the row read under the lock.
5. **Two publisher `UPDATE` statements on a package change.** Step 8 writes the
   package and step 10 writes the token, exactly in the specified order, because
   step 10 is conditional on step 9's outcome. The consequence is measured and
   reported rather than optimized away by reordering the specified sequence: on a
   change that includes a package change the publisher row is updated twice, so
   the existing `AFTER UPDATE` work-freshness trigger's single set-based
   statement runs twice over the same rows (section 9.6). A platform-only change
   or a linked repair updates the publisher row once.
6. **Accessor placement.** `subscriptionPackage`, `effectiveCapabilities` and
   `updatedAt` are defined **only** as GraphQL resolvers reading the one
   `publisher` row held by `PublisherServiceConfiguration`. They are deliberately
   not duplicated as model-layer inherent methods: one definition means a
   response can never report a package and a capability set that disagree.
7. **Report filters** are shared by the list and count queries through one
   `filtered_publishers` function, so they cannot diverge. `enabledPlatforms`
   uses a single grouped subquery with `HAVING count(*) = n` over the
   deduplicated requested set, which is exact `AND` semantics because
   `(publisher_id, platform)` is the assignment primary key.
8. **Error convention.** BE-03's resolvers map through
   `IntoFieldError::into_field_error`, which is what makes the section 13 table
   true (`NO_ACCESS`, `STALE_SERVICE_CONFIGURATION`, and `INTERNAL_ERROR` for
   `EntityNotFound`/`DistributionPlatformNotAssignable`/`DatabaseError`). No
   existing field's error shape is changed.
9. **`clippy::misnamed_getters`** is allowed with an explanatory comment on
   `PublisherServiceConfiguration::updated_at`, because the field must resolve to
   the configuration token and **not** to `publisher.updated_at` — the two are
   deliberately different values (specification section 6.4 item 3).

Deviations from the specification: NONE.

Two specification statements could not be satisfied as literally written, both
recorded here as reviewed conclusions rather than silent choices:

- **Section 14.3 item 3 — "commit the regenerated
  `thoth-client/assets/schema.graphql`".** That file is **generated and
  gitignored** in this repository (`thoth-client/.gitignore:1:assets/schema.graphql`),
  written by `thoth-client/build.rs` on every build. Merged repository evidence
  outranks the specification here (root `AGENTS.md` section 2 authority order),
  so the artifact is not committed. It was instead regenerated at the
  implementation head, diffed against the artifact regenerated from the
  authorized base in a separate worktree, and the exact diff is reproduced in
  section 7.2 with the artifact's SHA-256 for APP-01 pinning.
- **Section 18.4 — "the non-blank actor check constraint rejects a blank or
  whitespace-only actor".** The constraint's SQL is prescribed verbatim by
  section 8.1 as `CHECK (btrim(actor) <> '')`, and PostgreSQL's one-argument
  `btrim` trims **spaces only**. The constraint therefore rejects `''` and
  space-only actors but not a tab-only or newline-only actor. The prescribed SQL
  was implemented unchanged and the test asserts exactly what it guarantees; the
  gap is recorded in section 13 as a known limitation rather than closed by
  deviating from the approved DDL. No BE-03 write path can produce such a value:
  the only production writer takes the actor from `PolicyContext::user_id()`.

## 6. Database and migration effects

Migration added: YES

- migration files: `thoth-api/migrations/20260813_v1.7.0/up.sql` and
  `down.sql`.

  **DATE override authorization.** `20260812` is already occupied by merged
  BE-02 (`thoth-api/migrations/20260812_v1.7.0`), and `make migration` derives
  the directory name from `date +"%Y%m%d"`, which on the implementation day would
  have collided with it. The CTO explicitly authorized the repository-supported
  command with an overridden date:

  ```bash
  make migration DATE=20260813
  ```

  It produced exactly `thoth-api/migrations/20260813_v1.7.0/up.sql` and
  `thoth-api/migrations/20260813_v1.7.0/down.sql` and no other directory. The
  version suffix `v1.7.0` is derived by the Makefile from the workspace version
  `1.6.2`, unchanged. No existing migration was appended to, renamed or
  rewritten, and no migration was hand-created.
- schema effect: one new column on `publisher`, one new enum type with exactly
  two labels in order, one new table with one primary key, one `ON DELETE
  CASCADE` foreign key, one named check constraint and one composite index.
- existing-data effect: every existing publisher receives
  `service_configuration_updated_at` from the column default. No package value
  changes, no assignment row changes, zero audit rows are created and zero job
  tables exist.
- locking/downtime: `ADD COLUMN` takes a brief `ACCESS EXCLUSIVE` lock on
  `publisher`; the new table's foreign key additionally takes
  `SHARE ROW EXCLUSIVE`. Measured, not asserted — see section 9.1.
- empty database result: apply, revert, re-apply all clean — section 9.1.
- populated database result: `relfilenode` unchanged, packages unchanged,
  BE-02 assignments byte-identical, zero audit rows — section 9.1.
- rollback/forward repair: section 12.
- idempotency: not applicable; a one-shot DDL migration.

`thoth-api/src/schema.rs` was edited manually and atomically in this same pull
request under ADR-0003 Architecture A. `diesel print-schema` was not used, no
`diesel.toml` was introduced and no schema-synchronization subsystem was added.

Not created, by design: `distribution_job`, `distribution_job_target`,
`distribution_job_attempt`, any worker-role persistence, any credential or
configuration-secret table, and **any capability column, table, override or
cache**. Verified by catalog assertion — section 9.1.

## 7. API and compatibility effects

### 7.1 Additive inventory

Queries added: `publisherServiceConfiguration`,
`publisherServiceConfigurations`, `publisherServiceConfigurationCount`.
Mutations added: `replacePublisherServiceConfiguration`.
Object types added: `PublisherServiceConfiguration`,
`PublisherServiceConfigurationChange`,
`PublisherServiceConfigurationSummary`.
Input types added: `ReplacePublisherServiceConfigurationInput`.
Enums added: `PublisherServiceConfigurationSource`.
Enums becoming SDL-reachable: `ThothPackage`, `PublisherCapability`.

Nothing else. No scalar, interface or union; no field on an existing type; no
change to any existing field's type, nullability, arguments, defaults or
description.

### 7.2 Exact SDL diff

Generation command (from the repository root, which runs
`thoth-client/build.rs`):

```bash
cargo check --workspace
```

The base artifact was regenerated identically from a `git worktree` at
`3b6b3a31f9358011f0c998015dfd0c2508380e83` with a separate `CARGO_TARGET_DIR`,
and the two files compared.

```text
added lines:   72
removed lines: 0
```

The complete diff, in file order:

```diff
+"Capability that a subscription package may grant to a publisher. A capability permits a feature but does not configure or activate it"
+enum PublisherCapability {
+  "Publisher works may be considered for OAI-PMH after work-level open-licence and lifecycle checks" OAI_PMH
+  "Thoth-managed drivers may collect and retain canonical metrics when a source account and platform/measure configuration are enabled" METRICS_COLLECT
+  "Publisher users may submit approved publisher-controlled usage or sales reports" METRICS_IMPORT
+  "A Thoth-owned authenticated service may serve publisher dashboard metrics" METRICS_DASHBOARD
+  "A Thoth-owned authenticated service may serve bounded work-level widget metrics" METRICS_WIDGET
+  "Eligible finalized canonical metrics may create and deliver OPERAS export claims" METRICS_OPERAS_EXPORT
+}
+
+"How a recorded service-configuration change entered the system"
+enum PublisherServiceConfigurationSource {
+  "A committed superuser replacePublisherServiceConfiguration call; the actor is the authenticated account identifier" SUPERUSER_API
+  "A separately approved controlled historical backfill; the actor is the authorized control identity" MIGRATION_BACKFILL
+}
+
+"Subscription package determining which publisher services a publisher is entitled to"
+enum ThothPackage {
+  "Default package, with no publisher-service capabilities" OASIS
+  "Package permitting OAI-PMH eligibility and private managed metrics collection" OBELISK
+  "Package permitting OAI-PMH eligibility and all initial metrics capabilities" SPHINX
+  "Package permitting OAI-PMH eligibility and all initial metrics capabilities" PYRAMID
+}
+
+"Complete desired service configuration to store for a publisher. This is a replace, not a patch: the platform list is the complete desired enabled set, and an empty list means no destination is enabled"
+input ReplacePublisherServiceConfigurationInput {
+  publisherId: Uuid!
+  subscriptionPackage: ThothPackage!
+  enabledDistributionPlatforms: [DistributionPlatform!]!
+  expectedUpdatedAt: Timestamp!
+}
+
+  "Replace a publisher's complete desired service configuration under optimistic concurrency control. Superuser only. This stores desired configuration: it creates no distribution job and triggers no dissemination"
+  replacePublisherServiceConfiguration("Complete desired service configuration to store" data: ReplacePublisherServiceConfigurationInput!): PublisherServiceConfiguration!
+
+"The desired service configuration of one publisher."
+type PublisherServiceConfiguration {
+  "The publisher this configuration belongs to"
+  publisher: Publisher!
+  "Subscription package currently assigned to the publisher"
+  subscriptionPackage: ThothPackage!
+  "Capabilities the current subscription package grants this publisher, in canonical capability order. Derived from the package; a capability permits a feature but does not configure or activate it"
+  effectiveCapabilities: [PublisherCapability!]!
+  "Distribution platforms currently enabled for the publisher, in canonical platform order"
+  enabledDistributionPlatforms: [PublisherDistributionPlatformAssignment!]!
+  "Version token of this configuration; supply it as expectedUpdatedAt to replace the configuration"
+  updatedAt: Timestamp!
+}
+
+"Metadata of one recorded service-configuration change. The before and after states themselves are not exposed."
+type PublisherServiceConfigurationChange {
+  "When the change was committed"
+  changedAt: Timestamp!
+  "Identity that made the change: the account identifier for SUPERUSER_API, or the authorized control identity for a controlled backfill"
+  actor: String!
+  "How the change entered the system"
+  source: PublisherServiceConfigurationSource!
+}
+
+"A publisher's service configuration together with its latest change metadata."
+type PublisherServiceConfigurationSummary {
+  "The publisher's desired service configuration"
+  configuration: PublisherServiceConfiguration!
+  "Metadata of the most recent recorded configuration change, or null if none has been recorded"
+  lastChange: PublisherServiceConfigurationChange
+}
+
+  "Query the protected desired service configuration of one publisher. Readable only by a superuser or by a PUBLISHER_USER of that publisher"
+  publisherServiceConfiguration("Thoth publisher ID to search on" publisherId: Uuid!): PublisherServiceConfiguration!
+  "Query the protected desired service configuration of every publisher, with the metadata of its latest recorded change. Superuser only"
+  publisherServiceConfigurations(…): [PublisherServiceConfigurationSummary!]!
+  "Get the total number of publishers matching a protected service configuration report filter. Superuser only"
+  publisherServiceConfigurationCount(…): Int!
```

The two report lines are elided above only for width; their generated argument
lists are quoted verbatim below.

### 7.3 List-argument nullability, quoted verbatim

From the regenerated `thoth-client/assets/schema.graphql` at the implementation
head:

```graphql
publisherServiceConfigurations("If set, only shows results for publishers with these IDs" publishers: [Uuid!] = [], "If set, only shows results for publishers with these subscription packages" packages: [ThothPackage!] = [], "If set, only shows results for publishers that have every one of these distribution platforms enabled. Multiple values narrow the results rather than widening them" enabledPlatforms: [DistributionPlatform!] = [], "The number of items to return" limit: Int = 100, "The number of items to skip" offset: Int = 0, "The order in which to sort the results. Results are always additionally sorted by publisher ID ascending, so pagination is deterministic" order: PublisherOrderBy = {direction: "ASC", field: "PUBLISHER_NAME"}): [PublisherServiceConfigurationSummary!]!
publisherServiceConfigurationCount("If set, only counts publishers with these IDs" publishers: [Uuid!] = [], "If set, only counts publishers with these subscription packages" packages: [ThothPackage!] = [], "If set, only counts publishers that have every one of these distribution platforms enabled. Multiple values narrow the results rather than widening them" enabledPlatforms: [DistributionPlatform!] = []): Int!
```

The three required fragments, verbatim:

```graphql
publishers: [Uuid!] = []
packages: [ThothPackage!] = []
enabledPlatforms: [DistributionPlatform!] = []
```

Compared explicitly against the merged siblings named by specification section
14.3 item 2, from the same artifact:

| Merged sibling | Generated argument |
|---|---|
| `imprints` | `"If set, only shows results connected to publishers with these IDs" publishers: [Uuid!] = []` |
| `publishersByDistributionPlatform` | `platform: DistributionPlatform!, limit: Int = 100, offset: Int = 0, order: PublisherOrderBy = {direction: "ASC", field: "PUBLISHER_NAME"}` |
| `works` (merged enum-list precedent) | `"Specific types to filter by" workTypes: [WorkType!] = []` |

All three BE-03 filters render as **nullable outer lists of non-null members
with a `[]` default**, matching the merged convention exactly. None renders as
`[T!]!`.

### 7.4 Generated schema/client updates

- `thoth-client/assets/schema.graphql` is **build-generated and gitignored** in
  this repository, so it is not committed; see section 5. Its SHA-256 at the
  implementation head is
  `25329c1687d8b4222638c2f673bd2751a13adeda8c6f181d4ac83e869abac479`.
- `thoth-client/assets/queries.graphql`: **unchanged**, as a reviewed
  conclusion, not an omission. BE-03 adds protected operations the internal
  export client does not consume, and it changes no field the client already
  selects. `git status --porcelain thoth-client/assets/queries.graphql` is
  empty.
- No generated client enum conversion needed updating: the two newly reachable
  enums appear only on protected fields the client does not select.

### 7.5 APP-01 contract pinning

The pinned contract is the exact generated schema produced at the reviewed
BE-03 head. Record, at pinning time, **both**:

- the exact backend commit SHA of the reviewed BE-03 head (recorded on the pull
  request; see section 3);
- the schema artifact regenerated at that same head, whose SHA-256 at the head
  this report was written against is
  `25329c1687d8b4222638c2f673bd2751a13adeda8c6f181d4ac83e869abac479`.

It includes the corrected list-argument nullability of section 7.3 and the
`PublisherCapability` block of section 7.2. `thoth-app` is a **separate
repository** and was not modified; its codegen impact is additive-only (three
new queries, one new mutation, three new object types, one new input, three
newly reachable enums, no removal and no changed field), so an existing
`thoth-app` build against this contract continues to compile.

### 7.6 Backwards compatibility

Every existing public GraphQL surface is unchanged, including BE-02's four read
surfaces, their nullability, ordering and authorization — asserted verbatim by
the amended guard in `distribution_platform_tests.rs`. Deprecations: none.

## 8. Authorization and security

Authorization paths changed: three new protected operations. No existing
operation's authorization changed. No new role, ownership table, policy helper
or authorization framework was introduced.

| Caller | Read protected configuration | Replace configuration | Report / count |
|---|---|---|---|
| `SUPERUSER` | ALLOW, any publisher | ALLOW, any publisher | ALLOW |
| `PUBLISHER_USER` for the target publisher | ALLOW | DENY `NO_ACCESS` | DENY `NO_ACCESS` |
| `PUBLISHER_USER` only for another publisher | DENY `NO_ACCESS` | DENY `NO_ACCESS` | DENY `NO_ACCESS` |
| `PUBLISHER_ADMIN` for the target, no `PUBLISHER_USER` | DENY `NO_ACCESS` | DENY `NO_ACCESS` | DENY `NO_ACCESS` |
| `WORK_LIFECYCLE` for the target, no `PUBLISHER_USER` | DENY `NO_ACCESS` | DENY `NO_ACCESS` | DENY `NO_ACCESS` |
| `CDN_WRITE` for the target, no `PUBLISHER_USER` | DENY `NO_ACCESS` | DENY `NO_ACCESS` | DENY `NO_ACCESS` |
| all three of the above combined, no `PUBLISHER_USER` | DENY `NO_ACCESS` | DENY `NO_ACCESS` | DENY `NO_ACCESS` |
| authenticated, no applicable role | DENY `NO_ACCESS` | DENY `NO_ACCESS` | DENY `NO_ACCESS` |
| anonymous | DENY `NO_ACCESS` | DENY `NO_ACCESS` | DENY `NO_ACCESS` |
| account with `PUBLISHER_USER` for two publishers | ALLOW for both, DENY for a third | — | — |
| target publisher with `NULL` `zitadel_id` | DENY for every non-superuser; ALLOW for superuser | — | — |
| unknown publisher, authenticated caller | `EntityNotFound` (`INTERNAL_ERROR`) | — | — |
| unknown publisher, anonymous caller | DENY `NO_ACCESS` before any load | — | — |

Every row is asserted by a test in
`thoth-api/src/graphql/service_configuration_tests.rs`, each requesting
`effectiveCapabilities`, so capability exposure is proven to follow the same
single decision as the rest of the type. Adding `PUBLISHER_USER` to an otherwise
denied account opens the read, which proves the denial is the role check and not
an unrelated failure. Every mutation denial was taken before the database was
touched: the token is unchanged after the whole denied-caller sweep.

Secret or personal-data handling: the audit `actor` is exactly
`IntrospectedUser.user_id`, the repository's established free-text actor
representation with no foreign key to any local account (there is no local
account table). No credential, token, endpoint, bucket, host, adapter identity
or deployment identity is stored, logged or returned. The audit
`before_state`/`after_state` JSON is never exposed through GraphQL. The stale
error message contains no SQL, table name, column name, driver text or the
current stored token — disclosing the current token to a caller that just failed
a version check would let it blind-write over a change it never read.

Security limitations: recorded in section 13.

## 9. Tests and checks

All commands were run from the repository root against **disposable local
services** (a scratch PostgreSQL 17.10 cluster and a local Redis). Nothing was
pointed at production or at a shared service.

### 9.1 Migration and schema evidence

Empty-database apply, revert, re-apply (database `thoth_be03_empty`):

```bash
DATABASE_URL="postgres://thoth:thoth@localhost/thoth_be03_empty" cargo run migrate
DATABASE_URL="postgres://thoth:thoth@localhost/thoth_be03_empty" cargo run migrate --revert
DATABASE_URL="postgres://thoth:thoth@localhost/thoth_be03_empty" cargo run migrate
```

```text
apply    -> __diesel_schema_migrations head = 20260813
revert   -> 0 migrations remain; 0 publisher/audit tables; 0 source types
re-apply -> 20250000 20260417 20260429 20260504 20260805 20260812 20260813
            publisher_service_configuration_history rows = 0
```

Catalog verification (same database):

```text
publisher_service_configuration_source: 1 SUPERUSER_API, 2 MIGRATION_BACKFILL
publisher.service_configuration_updated_at | timestamp with time zone | NOT NULL | default CURRENT_TIMESTAMP
publisher_service_configuration_history columns: history_id uuid NOT NULL default uuid_generate_v4();
  publisher_id uuid NOT NULL; actor text NOT NULL; source USER-DEFINED NOT NULL;
  before_state jsonb NOT NULL; after_state jsonb NOT NULL;
  created_at timestamptz NOT NULL default CURRENT_TIMESTAMP
constraints:
  publisher_service_configuration_history_actor_check       CHECK ((btrim(actor) <> ''::text))
  publisher_service_configuration_history_pkey              PRIMARY KEY (publisher_service_configuration_history_id)
  publisher_service_configuration_history_publisher_id_fkey FOREIGN KEY (publisher_id)
      REFERENCES publisher(publisher_id) ON DELETE CASCADE
indexes:
  publisher_service_configuration_history_pkey
  publisher_service_configuration_history_publisher_created_idx
      btree (publisher_id, created_at DESC, publisher_service_configuration_history_id DESC)
non-internal triggers on the audit table: 0   (append-only: no updated_at column)
job tables: 0    capability tables: 0    capability columns: 0
```

Populated-database forward migration (database `thoth_be03_populated`: 500
publishers with all four packages, 500 imprints, 2 000 works, 875 BE-02
assignments including 375 normalized OAPEN/DOAB pairs and 125 retained disabled
rows). The database was first brought to the exact pre-BE-03 state by running
BE-03's own `down.sql` and deleting its `__diesel_schema_migrations` row, then
the **real runner** applied the migration forward:

```bash
DATABASE_URL="postgres://thoth:thoth@localhost/thoth_be03_populated" ./target/debug/thoth migrate
```

```text
                     BEFORE                                   AFTER
relfilenode        = 668819                       relfilenode        = 668819   (unchanged: no table rewrite)
packages           = OASIS:125,OBELISK:125,       packages           = identical
                     PYRAMID:125,SPHINX:125
assignment_digest  = 4fc7a9038cff481cc657d5bf     assignment_digest  = identical (md5 over every
                     eda1f39e                                          assignment column, ordered)
assignments        = 875                          assignments        = 875
works              = 2000                         works              = 2000
                                                  audit_rows            = 0
                                                  publishers_with_token = 500
                                                  distinct_token_values = 1
                                                  job_tables            = 0
real 0.04 (whole `thoth migrate` process, /usr/bin/time -p)
```

`relfilenode` is **unchanged**, confirming the `STABLE` `CURRENT_TIMESTAMP`
default was stored as a fast default with no table rewrite.

Observed locking, taken by replaying `up.sql` inside one explicit transaction on
a `createdb -T` copy and self-inspecting `pg_locks` before commit:

```text
BEGIN                 0.161 ms
ALTER TABLE           0.725 ms
CREATE TYPE           0.274 ms
CREATE TABLE          2.264 ms
CREATE INDEX          0.439 ms

relname                                                        | mode                  | granted
publisher                                                      | AccessExclusiveLock   | t
publisher                                                      | AccessShareLock       | t
publisher                                                      | ShareRowExclusiveLock | t
publisher_service_configuration_history                        | AccessExclusiveLock   | t
publisher_service_configuration_history                        | AccessShareLock       | t
publisher_service_configuration_history                        | ShareLock             | t
publisher_service_configuration_history                        | ShareRowExclusiveLock | t
publisher_service_configuration_history_publisher_created_idx  | AccessExclusiveLock   | t
COMMIT                0.220 ms
```

`schema.rs` matches the migrated database contract: asserted in-process by
`the_migrated_database_matches_the_schema_contract`, and structurally by the
whole `thoth-api` suite compiling and running every Diesel query in this report
against the migrated database.

### 9.2 Formatting

```bash
cargo fmt --all -- --check
```

```text
(no output; exit 0)
```

```bash
git diff --check
```

```text
(no output; exit 0)
```

### 9.3 Static analysis

```bash
cargo check --workspace
```

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 43s
```

```bash
cargo clippy --all --all-targets --all-features -- -D warnings
```

```text
Finished; no error and no clippy warning.
One pre-existing dependency notice remains, unrelated to BE-03:
"the following packages contain code that will be rejected by a future version
 of Rust: proc-macro-error2 v2.0.1"
```

### 9.4 Tests

```bash
cargo test -p thoth-api --features backend
```

```text
lib:                1047 passed; 0 failed
graphql_permissions:  13 passed; 0 failed
doc-tests:             0 passed; 8 ignored
```

```bash
cargo test --workspace
```

```text
thoth (lib)            0 passed; 0 failed
thoth (bin)           14 passed; 0 failed
thoth_api (lib)     1047 passed; 0 failed
graphql_permissions   13 passed; 0 failed
thoth_api_server       3 passed; 0 failed
thoth_client           4 passed; 0 failed
thoth_errors          11 passed; 0 failed
thoth_export_server  144 passed; 0 failed
doc-tests            8 passed; 8 ignored  (thoth_client 6, thoth_export_server 2)
TOTAL               1244 passed; 0 failed
```

BE-03 adds **72** tests: 45 in
`model::publisher_service_configuration::tests`, 23 in
`graphql::service_configuration_tests`, 2 connection-scoped primitive
regressions in `model::publisher_distribution_platform::tests`, and 2 replacing
1 in the amended `graphql::tests` SDL guard.

```bash
cargo test -p thoth-export-server
```

```text
144 passed; 0 failed  +  2 doc-tests passed
```

**`cargo test -p thoth-client` and `cargo build -p thoth-client` cannot be run
standalone in this repository, and this is pre-existing rather than caused by
BE-03.** Both fail with 26 `cannot find 'graphql' in crate` errors because only
the workspace/dev-dependency edge enables `thoth-api`'s `backend` feature, and
single-package feature unification drops it. The identical failure was
reproduced at the authorized base `3b6b3a31` in a clean worktree. The client is
therefore built and tested through the workspace forms above, where its 4 unit
tests and 6 doc-tests pass and `build.rs` regenerates the SDL.

### 9.5 Query-count evidence

Measured with the existing observed-loader harness
(`SqlProbe` + `RequestLoaders::for_request_observed`), which exercises the
**production** batcher, driving the real GraphQL report and requesting
`enabledDistributionPlatforms` on every summary:

| Page size | Publisher-page statements | Latest-change statements | Assignment statements | Loader dispatch chunks |
|---:|---:|---:|---:|---|
| 1 | 1 | 1 | 1 | `[1]` |
| 25 | 1 | 1 | 1 | `[25]` |
| 200 | 1 | 1 | 1 | `[200]` |

The latest-change statement contains `DISTINCT ON` and `= ANY`; the assignment
statement contains `= ANY`. The count does not grow with N and there is no
per-publisher loop. The single-publisher protected query issues exactly one
assignment statement, one loader dispatch of size 1, and **zero** history
statements. No second assignment loader exists: asserted by
`no_second_assignment_loader_was_introduced`, and the protected resolver is
loader-first and `try_load`-only, asserted by source inspection.

### 9.6 Publisher and work trigger evidence

Fixture, as specification section 18.4 requires: one target publisher, **two**
imprints belonging to it, **two** works distributed across those imprints, and a
**control work belonging to a different publisher**. Every value is read
directly from the database before and after, against a real disposable
PostgreSQL database with the migration applied, so both publisher triggers
actually execute.

| Case | `service_configuration_updated_at` | `publisher.updated_at` | every target `work.updated_at_with_relations` | control work |
|---|---|---|---|---|
| committed package-only change | moves | moves | moves, all | unchanged |
| committed platform-only change | moves | moves | moves, all | unchanged |
| committed linked repair (membership unchanged) | moves | moves | moves, all | unchanged |
| true semantic no-op | unmoved | unmoved | unmoved | unchanged |
| stale request | unmoved | unmoved | unmoved | unchanged |
| injected pre-commit failure / rollback | unmoved | unmoved | unmoved | unchanged |

The platform-only and repair rows are asserted deliberately, with a comment in
the test saying so, because **merged BE-02 alone would have moved neither**:
BE-02 does not `UPDATE publisher`, and `publisher_distribution_platform` carries
no work-freshness trigger. BE-03 moves them because the same transaction writes
the configuration token to the publisher row.

**A committed configuration change refreshes `work.updated_at_with_relations`
across the whole of that publisher's catalogue. That is a public,
downstream-consumed freshness signal**: it is resolved on the public `Work` type,
it is a filter and ordering key on the anonymous work queries, and
`thoth-export-server` uses it to decide Redis cache freshness — so the change
will cause that publisher's cached metadata records to be treated as stale and
regenerated on next request, and will cause incremental consumers to re-select
that publisher's works. **It is not distribution activation**: no distribution
job, job target, job attempt, upload, feed, message or dissemination is created
by it.

### 9.7 Catalogue-scale write-amplification and lock-footprint measurement

Disposable-environment measurement only, driven through the real GraphQL
mutation on a materially larger catalogue. **This is empirical evidence about
the shape of the cost. It is not a production SLA, it is not extrapolated to
production, and no "safe" catalogue size is derived from it.**

```text
target works:                                        2000
control works (a different publisher):                250
SQL statements issued by the configuration operation:  24
work rows changed by the publisher trigger:          2000
unrelated publisher work rows changed:                  0
request duration in this disposable environment:  86.35 ms
```

Of the 24 captured statements: 2 are pooled-connection health checks
(`SELECT 1`), `BEGIN`/`COMMIT` are 2, 3 are Diesel `pg_type` OID lookups, 1 is
the post-commit DataLoader read serving the mutation's own response, and the
remaining 16 are the coordinator's own reads and writes — the publisher lock
(taken 3 times: once by the coordinator and once inside each connection-scoped
primitive, harmless no-ops after the first), the publisher row read, the enabled
assignment reads, the package `UPDATE`, the linked-group member reads and
transaction timestamps, three assignment upserts, the token `UPDATE` and the
audit `INSERT`. **The count is bounded by the closed 17-value platform inventory
and does not grow with catalogue size.**

**No per-work application loop exists**: the request issues **no statement at
all** against the `work` table, asserted by the test. The 2 000 work rows are
changed by the existing `AFTER UPDATE` trigger's single set-based
`UPDATE work ... FROM imprint WHERE imprint.publisher_id = NEW.publisher_id`.
Because this request changed the package *and* the platforms, the publisher row
is updated twice (specification section 7.3 steps 8 and 10), so that set-based
statement runs **twice** over the same 2 000 rows; a platform-only change or a
linked repair updates the publisher row once and runs it once.

The transaction's real footprint is therefore `one publisher row + bounded
configuration/audit rows + N related work rows`, the work-row locks are held for
the remainder of the transaction, and the publisher `FOR UPDATE` lock is held
while the trigger's work executes, so concurrent configuration writers for the
same publisher serialize behind all of it. BE-04 inherits this with the
transaction boundary.

**Stop-condition 19 assessment: NOT TRIGGERED.** At 2 000 works the whole
request took 86 ms in this disposable environment, the statement count is
bounded and independent of catalogue size, the cascade is one set-based
statement per publisher `UPDATE` rather than a loop, and unrelated publishers
are provably untouched. The implementing agent does **not** judge this a
material operational problem at realistic catalogue sizes. The measurement is
placed before the independent reviewer to assess independently. No trigger,
package location or token architecture was altered.

### 9.8 Concurrency evidence

All against a real database with two threads and separate pooled connections:

- two clients holding one token: exactly one commits, the loser fails
  `StalePublisherServiceConfiguration`, exactly one audit row exists, and the
  final persisted state equals what the winner committed;
- the same with **linked OAPEN/DOAB** assignments: the loser leaves no partial or
  one-sided pair; both rows end enabled with one shared activation and one
  shared `enabled_at`;
- a stale request that would have been a **true no-op**: still fails, moves
  nothing;
- a stale request that would have **repaired a split pair**: still fails, the
  split pair survives byte-identically, no token movement, no audit row —
  proving the version check precedes every lifecycle call;
- two concurrent membership-equal **repair** requests: one repairs, one is
  stale, the final pair is normalized with one shared activation and exactly one
  audit row exists;
- concurrent replacements on **different** publishers: both commit, one audit
  row each, no contention;
- a replacement concurrent with a **direct BE-02** `disable`: both complete,
  serialized on the same publisher row lock, **no deadlock**, and every
  assignment row satisfies `enabled == (disabled_at IS NULL)` afterwards;
- **strict token monotonicity** per publisher across a sequence of committed
  changes that includes a repair: every successive token is strictly greater and
  all values are distinct.

### 9.9 Audit evidence

- exactly one audit row per committed change, including a change that updates
  the package **and** several linked groups and singletons in one request;
- `before_state` and `after_state` key sets are **exactly**
  `{configurationVersion, enabledDistributionPlatforms, subscriptionPackage}`;
  the test fails if any key is ever added, and separately asserts the serialized
  JSON contains no `activation`, `enabledAt`, `disabledAt`, `capabilit`,
  `zitadel`, `publisherName`, `credential`, `token`, `endpoint` or `bucket`;
- platforms are serialized in canonical `DistributionPlatform::ALL` order —
  observed audit value: `["OAPEN","DOAB","ZENODO"]` for a request naming
  `[OAPEN, ZENODO]`;
- for a linked-state repair the two states are equal in `subscriptionPackage`
  and `enabledDistributionPlatforms` and differ only in `configurationVersion`,
  with `before_state.configurationVersion` equal to the superseded token and
  `after_state.configurationVersion` equal to the new one;
- `actor` equals the caller's `PolicyContext::user_id()`;
- `source` is `SUPERUSER_API` for **every** row BE-03 writes, and no BE-03 path
  writes `MIGRATION_BACKFILL` — asserted over a three-change sequence;
- the non-blank actor check rejects `''`, `' '` and `'   '` (see the section 5
  `btrim` limitation);
- a committed configuration change writes **no** `publisher_history` row,
  confirming the coordinator does not route the package update through
  `Crud::update`;
- deleting a publisher cascades its audit rows away.

**`publisher_history.data` additive-key consequence.** `publisher_history` and
`publisher_service_configuration_history` are **not the same thing**.
`publisher_history` is the pre-existing generic entity-history table written by
the shared `Crud::update` macro for publisher metadata edits, keyed by
`user_id`, storing a whole-entity snapshot with a legacy
`timestamp without time zone`; BE-03's mutation never writes it. Because
`service_configuration_updated_at` is appended to the `Publisher` struct that
`Crud::update` serializes, the new field **may appear as an additional key in
future `publisher_history.data` snapshots wherever that path runs** — for
example an ordinary `updatePublisher` metadata edit. That is an additive JSON key
in an untyped `jsonb` column: no existing key changes meaning, no existing row is
rewritten, and the table, its columns and its triggers are not modified. This is
observed and accepted, not a defect.

### 9.10 Capability evidence

- field name and type: `effectiveCapabilities: [PublisherCapability!]!` on
  `PublisherServiceConfiguration`;
- for **every** `ThothPackage` the returned list equals
  `ThothPackage::capabilities()` for that package **as an exact ordered
  sequence**, not set equality, so a later sort, dedup or reorder fails the test;
  repeated reads return an identical sequence;
- `OASIS` returns an empty list, never `null`;
- the package reported in the same response always agrees with the capabilities,
  because both are read from the same locked publisher row — one derivation
  site, no second mapping;
- a package **upgrade** (`OASIS -> SPHINX`) and **downgrade**
  (`SPHINX -> OBELISK`) change the capabilities automatically, in the mutation's
  own returned configuration **and** in a subsequent query, with no separate
  capability write, migration, backfill or reconciliation;
- a platform-only change leaves the capabilities unchanged;
- **no capability state is persisted anywhere**: the migration creates no
  capability column, table, override or index, asserted by catalog query; the
  only durable input is `publisher.subscription_package`;
- `PublisherCapability` is SDL-reachable **only** through
  `PublisherServiceConfiguration.effectiveCapabilities` — exactly one field in
  the entire schema returns it — and no anonymous operation can select a
  capability or package value; the public `Publisher` type contains no package,
  capability or configuration-version field of any spelling.

### 9.11 Linked-platform and independence evidence

- requesting OAPEN alone, or DOAB alone, enables both with one shared activation
  and one shared `enabled_at`; omitting both disables both, retaining the rows;
- **membership-equal repairs**, each executed with the requested enabled
  membership already equal to the current membership so a membership-only diff
  would be empty:

  | Seeded split state | Request | Result |
  |---|---|---|
  | both enabled, **different `activation_id`** | `OAPEN` | repaired to one shared activation and timestamp; token bumped; exactly 1 audit row; states differ only in `configurationVersion` |
  | both enabled, same activation, **different `enabled_at`** | `DOAB` | as above |
  | **one-sided** (OAPEN enabled, DOAB absent) | `OAPEN` | as above |
  | **one-sided** (OAPEN enabled, DOAB absent) | `DOAB` | as above |
  | fully normalized pair | `OAPEN` | **true no-op**: nothing written, rows byte-identical, token unmoved, no audit row |

- a package-only change over already normalized state leaves every assignment
  row **byte-identical** (`activation_id`, `enabled_at`, `disabled_at`,
  `updated_at` all equal);
- a package change submitted together with a **split** requested group writes the
  package **and** repairs the group, under one token bump and one audit row;
- a platform-only change leaves `subscription_package` unchanged;
- `OCLC_KB` and `EX_LIBRIS_KB` receive independent activations and are disabled
  independently with no coupling;
- `JISC_NBK` fails with `DistributionPlatformNotAssignable`, writing nothing,
  moving no token and creating no audit row — asserted **both** when the request
  would otherwise have changed nothing **and** when it would otherwise have
  changed the package and enabled a valid platform, proving whole-set
  pre-validation precedes the first lifecycle call;
- duplicates are deduplicated with no error; an empty list disables everything
  and is never read as "all";
- a request naming several linked groups and several singletons writes one audit
  row and bumps the token once.

### 9.12 BE-02 regression evidence

- BE-02's **40** existing lifecycle tests in
  `model::publisher_distribution_platform::tests` pass **unchanged** against the
  refactored connection-scoped functions; no behavioural expectation was edited;
- BE-02's four public read surfaces return identical results, ordering and
  errors, and their generated SDL signatures are asserted **verbatim** by the
  amended guard;
- the pool-level `enable`/`disable` behaviour is unchanged, including that a
  non-assignable platform still fails **before** any connection is acquired or
  transaction opened (the check is the first statement of `enable`, ahead of
  `db.get()?`);
- the connection-scoped primitives' outcomes are asserted directly for every
  transition: absent row enabled -> `Changed`; already-enabled singleton ->
  `Unchanged`; enabled group disabled -> `Changed`; group with no enabled member
  disabled -> `Unchanged`; disabled row re-enabled -> `Changed`;
  already-normalized linked group -> `Unchanged`; split pair -> `Changed`;
- **direct `enable_on(JISC_NBK)` regression.** Called directly inside a
  caller-owned transaction, bypassing both the pool-level wrapper and the
  coordinator so nothing has pre-validated the platform, it returns
  `ThothError::DistributionPlatformNotAssignable("JISC_NBK")`; **no** row is
  created for `JISC_NBK` or any other platform; **no** existing assignment row
  changes (enabled state, `activation_id`, `enabled_at`, `disabled_at` and
  `updated_at` all byte-identical); the caller's transaction is then **both
  rolled back and committed** in two separate runs with no hidden mutation in
  either case, proving the primitive failed before any write rather than relying
  on rollback; and `publisher.service_configuration_updated_at`,
  `publisher.updated_at` and the audit table are all unaffected because this path
  never reaches the coordinator's committed-change phase;
- the coordinator's own whole-set pre-validation is proven separately
  (section 9.11), so the two checks are established independently rather than one
  masking the other.

### 9.13 Write-path containment evidence

Searched paths, all verified present:

```text
thoth-api/src            thoth-api-server/src     thoth-client/src
thoth-errors/src         thoth-export-server/src  src
thoth-api/migrations
```

Workspace-declaration check, so no local production crate was omitted from the
scope:

```bash
grep -n "members" Cargo.toml
```

```text
15:members = ["thoth-api", "thoth-api-server", "thoth-client", "thoth-errors", "thoth-export-server"]
```

`thoth-app` is a **separate repository** and does not exist here
(`ls -d thoth-app` -> `No such file or directory`), so no `thoth-app/src` path
was searched and none is presented as evidence.

**Call-site enumeration.**

```bash
grep -rn 'replace_publisher_service_configuration' <scope>
```

| Location | Classification |
|---|---|
| `thoth-api/src/model/publisher_service_configuration/crud.rs:67` | definition |
| `thoth-api/src/graphql/mutation.rs:85` | **production — the only production caller** (inside `replace_service_configuration`, called by the resolver at `mutation.rs:111`) |
| `thoth-api/src/graphql/service_configuration_tests.rs:145` | test fixture |
| `thoth-api/src/model/publisher_service_configuration/tests.rs:64` | test helper |
| `crud.rs:6`, `mod.rs:9`, `mutation.rs:36`, `tests.rs:19`, `service_configuration_tests.rs:25` | doc comment / import |

```bash
grep -rnE 'PublisherDistributionPlatform::(enable|disable)\(' <scope>
```

**66 hits, every one in a `tests.rs` file** — 14 in
`graphql/distribution_platform_tests.rs`, 50 in
`model/publisher_distribution_platform/tests.rs`, 2 in
`model/publisher_service_configuration/tests.rs` (the direct-BE-02 concurrency
test). **BE-02's pool-level lifecycle functions have zero production call
sites**, exactly as specification section 2.1 item 7 recorded, so BE-03
establishes the single production write path without displacing any existing
production caller.

```bash
grep -rnE '(enable_on|disable_on)\(' <scope>
```

| Location | Classification |
|---|---|
| `model/publisher_distribution_platform/crud.rs:103,175` | definitions |
| `model/publisher_distribution_platform/crud.rs:80,162` | production — BE-02's own pool-level wrappers delegating to them |
| `model/publisher_service_configuration/crud.rs:138,154` | **production — the coordinator, the only production configuration caller** |
| `model/publisher_distribution_platform/tests.rs:1200-1302` (9 hits) | tests |

**Bypass search.**

```bash
grep -rn  'subscription_package'                    <scope> | grep -v 'tests.rs:'
grep -rn  'service_configuration_updated_at'        <scope> | grep -v 'tests.rs:'
grep -rn  'publisher_service_configuration_history' <scope> | grep -v 'tests.rs:'
grep -rnE 'diesel::(insert_into|update|delete)'     thoth-api/src --include=*.rs \
  | grep -v 'tests.rs:' \
  | grep -E 'publisher::table|publisher_distribution_platform|publisher_service_configuration_history'
grep -rn  'sql_query'                               <scope> | grep -v 'tests.rs:' | grep -v 'fixture.rs'
grep -rn  'diesel(table_name = publisher'           thoth-api/src --include=*.rs | grep -v tests.rs
```

Complete relevant production matches, with every write classified:

| Write target | Production writer | Classification |
|---|---|---|
| `publisher.subscription_package` | `model/publisher_service_configuration/crud.rs:126` | the coordinator, step 8 — **the only writer** |
| `publisher.service_configuration_updated_at` | `model/publisher_service_configuration/crud.rs:177-180` | the coordinator, step 10 — **the only writer** |
| `publisher_distribution_platform` (INSERT) | `model/publisher_distribution_platform/crud.rs:122` | inside `enable_on` |
| `publisher_distribution_platform` (UPDATE) | `model/publisher_distribution_platform/crud.rs:192` | inside `disable_on` |
| `publisher_service_configuration_history` (INSERT) | `model/publisher_service_configuration/crud.rs:208` | the coordinator, step 11 — **the only writer** |

Every other production match of those identifiers is a `schema.rs` column
declaration, a struct field, a doc comment, a **read** (`crud.rs:436` package
filter, `crud.rs:443` assignment filter subquery, `crud.rs:480-493` latest-change
read, `graphql/model.rs:1399-1444` resolvers, `publisher/crud.rs:151,232` BE-02's
merged reverse-lookup joins) or a migration file.

**No implicit writer exists through Diesel changesets.** The only
`AsChangeset`/`Insertable` types targeting `publisher` are `NewPublisher` and
`PatchPublisher`, and **neither declares `subscription_package` or
`service_configuration_updated_at`**:

```rust
pub struct PatchPublisher {
    pub publisher_id: Uuid,
    pub publisher_name: String,
    pub publisher_shortname: Option<String>,
    pub publisher_url: Option<String>,
    pub zitadel_id: Option<String>,
    pub accessibility_statement: Option<String>,
    pub accessibility_report_url: Option<String>,
}
```

so `Publisher::update` (the `updatePublisher` mutation) writes the publisher row
without being able to touch either protected column, and `Publisher::create`
leaves both to their defaults. The only production raw SQL anywhere in scope is
`SET CONSTRAINTS ... DEFERRED` and `pg_advisory_xact_lock` in unrelated modules.

**Conclusion.** The GraphQL mutation calls the coordinator; there is no second
production configuration writer; BE-02's pool-level lifecycle functions have no
production call site at all and are therefore not production
service-configuration entry points; the connection-scoped primitives are used
for configuration purposes only by the coordinator; and nothing outside the
coordinator writes configuration package, platform, token or audit state.

**Token/audit coupling**, proven and honestly bounded: every committed change
made through the coordinator moves the token and writes exactly one audit row,
asserted across the committed-change, repair, multi-group, upgrade and downgrade
tests. Recorded as a limitation rather than an enforced property: the
lower-level primitives remain *capable* of writing assignments without the
token, so the single-writer invariant is enforced by this specification, this
review and the search evidence above — **not by the type system**. A future task
needing a second configuration writer must extend the coordinator rather than
bypass it.

## 10. Manual verification

Environment: local disposable PostgreSQL 17.10 (Homebrew, scratch cluster, trust
auth) and local Redis, both created for this task and holding no production
data. No production or shared service was contacted at any point.

Steps and observed results are the measured evidence in sections 9.1–9.13; every
one is an automated assertion or a recorded command output rather than an
unverified claim.

## 11. CI

CI status: **PASSING** — normal `pull_request`-triggered CI on the draft PR. No
workflow file was changed and no workflow was manually dispatched.

| Check | Result | Duration |
|---|---|---|
| `classify` (build/lint/test workflow) | pass | 8s |
| `classify` (migrations workflow) | pass | 5s |
| `classify` (staging image workflow) | pass | 9s |
| `check-changelog` | pass | 5s |
| `format_check` | pass | 6s |
| `build` | pass | 2m26s |
| `lint` | pass | 5m23s |
| `test` | pass | 6m5s |
| `run_migrations` | pass | 3m20s |
| `build_and_push_staging_docker_image` | pass | 9m11s |

Failures or warnings: none. `run_migrations` independently applies the new
migration in CI's own PostgreSQL service, and `build` regenerates the SDL through
`thoth-client/build.rs`, so the generated-contract path is exercised there as
well.

## 12. Rollout and rollback

Initial state after merge: **repository history changes only.** No deployment
occurs, no migration is executed anywhere, no configuration changes and no API
behaviour becomes available to any client.

Activation required: none, and none is authorized. Deployment and migration
execution remain separately gated by CG-13 and separate release authorization.

Feature flag/configuration: none. A protected superuser-only mutation does not
require one, and no repository or programme authority establishes one.

Migration sequence: `20260813_v1.7.0` applies after `20260812_v1.7.0`, in the
ordinary embedded-runner sequence.

Monitoring required: none. BE-03 activates nothing to monitor and adds no log,
metric or alert. The mutation logs no audit JSON, actor identity, token or
credential.

Rollback:

- **before environment adoption** — no deployment, no migration executed, no
  audit or configuration data exists: an ordinary code revert of the pull
  request is possible under normal review;
- **after environment adoption** — retain the additive foundation; forward
  repair is preferred. An incorrect publisher configuration is corrected through
  a reviewed and audited configuration change, which itself writes an audit row,
  never by editing or deleting history. **Do not destroy audit history as
  routine rollback**: dropping a populated
  `publisher_service_configuration_history` requires separate explicit
  authorization. `down.sql` is reversibility **evidence**, not an automatic
  production rollback procedure.

Even after a separately authorized deployment and migration, this is **not** a
claim of zero observable downstream data change: changing protected desired
configuration creates no distribution job, performs no upload and triggers no
dissemination worker, but the existing `set_work_updated_at_with_relations`
trigger **will** refresh `work.updated_at_with_relations` across that publisher's
catalogue and invalidate that publisher's cached export records (section 9.6).
That freshness cascade is an existing catalogue-freshness mechanism reacting to a
real publisher-row change; it must **not** be described as distribution
activation, which remains separately gated and unauthorized.

## 13. Known limitations and deferred work

1. **The single-write-coordinator invariant is not type-enforced.** It is held by
   specification, review and the section 9.13 search evidence. The lower-level
   BE-02 primitives remain capable of writing assignments without the token.
2. **`btrim(actor) <> ''` trims spaces only.** The approved DDL is implemented
   verbatim, so a tab-only or newline-only actor would satisfy the constraint. No
   BE-03 write path can produce one — the only production writer supplies
   `PolicyContext::user_id()` — and strengthening the predicate would deviate
   from the approved specification, so it is recorded here instead.
3. **`EntityNotFound` and `DistributionPlatformNotAssignable` still surface as
   `INTERNAL_ERROR`.** Changing that would alter BE-02's merged contract and
   normalize unrelated error families; recorded rather than silently changed.
4. **`ThothError::DatabaseError` still renders the underlying driver message.**
   Pre-existing behaviour; BE-03 adds nothing to that exposure.
5. **A committed configuration change is publicly observable by timing.** The
   protected *values* stay behind the section 8 read decision, but an observer
   polling `updatedAtWithRelations` or watching export-cache regeneration can
   infer that something changed for that publisher at that time. This is
   acknowledged, not denied.
6. **A package change updates the publisher row twice** (steps 8 and 10), so the
   work-freshness cascade runs twice over the same rows for such a change. This
   follows the specified transaction sequence; it is measured in section 9.7
   rather than optimized away by reordering approved steps.
7. **`thoth-client` cannot be built or tested as a single package** in this
   repository. Pre-existing and reproduced at the authorized base; the workspace
   forms are used instead.
8. **`thoth-client/assets/schema.graphql` is gitignored**, so the pinned contract
   is identified by commit SHA plus the artifact SHA-256 in section 7.5 rather
   than by a committed file.
9. Deferred by design and unauthorized here: BE-04 durable jobs, MIG-01 backfill
   (`MIGRATION_BACKFILL` is defined and never written), APP-01, APP-02, and any
   deployment, environment or production migration, assignment creation,
   distribution activation or `OBSERVE`/`ENFORCE` transition.

## 14. Unresolved issues

NONE.

## 15. Agent self-assessment

The implementing agent may identify risks but may not approve the task.

Suggested review focus:

1. **The coordinator's transaction sequence** against specification section 7.3,
   step by step — especially that staleness precedes validation and every
   lifecycle call, that `enable_on` is invoked unconditionally for every desired
   group, and that the token bump and single audit insert are gated on the
   aggregated outcome.
2. **The write-path containment evidence** in section 9.13, since the
   single-writer invariant is not type-enforced. Re-run the searches at the exact
   head.
3. **The two authorized SDL guard amendments**: confirm the replacements preserve
   and strengthen the original security intent rather than weakening it — the
   public `Publisher` type, the BE-02 types, `PublisherCapability` reachability
   through exactly one field, and BE-02's four verbatim read surfaces.
4. **The section 9.7 catalogue-scale measurement**, and whether the reviewer
   agrees with the implementing agent's stop-condition 19 assessment that the
   cascade is not a material operational problem.
5. **The `btrim` limitation** (section 13 item 2): whether implementing the
   approved DDL verbatim, and recording the gap, was the right call versus
   raising it as a specification correction.
6. **The two specification statements that could not be satisfied literally**
   (section 5): the gitignored generated schema, and the `btrim` semantics.
7. **Migration reversibility and the populated-database evidence**, including
   the unchanged `relfilenode` and the byte-identical assignment digest.
