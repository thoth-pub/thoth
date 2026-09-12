# Thoth Metrics Contract Register

Status: ACTIVE DESIGN REGISTER
Owner: Thoth Metrics programme

## 1. Normalized observation

Version:

```text
thoth-normalized-metrics/1
```

Required: source account, platform, measure, work DOI, period start/end, grain, value and methodology version. Optional: publication ISBN/type, country, ROR, source record and row.

Owner: `thoth`. Producer: `thoth-sphinx`.

Sphinx constructs batches; Thoth resolves codes and identifiers. Contract changes require versioning or explicit compatibility review.

## 2. Registry and administration

Thoth owns platform, measure, platform-measure, source account, publisher approval and entitlement operations. Codes are stable, mutations are protected, credentials never enter configuration JSON, and generated clients must be refreshed.

### 2.1 Delivered administration surface (`MET-WP1-12`)

`MET-WP1-12` delivers the first protected slice of this contract: administration
of exactly `metric_platform`, `metric_measure` and `metric_platform_measure`.

Operations, all **SUPERUSER only** and all authorized before any
mutation-specific database read, row lock or write:

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

Contract properties consumers may rely on:

- **Stable code identity.** Every operation addresses a row by its stable
  `code`, and a mapping by its `(platformCode, measureCode)` pair. No mutation
  requires the caller to know a database-generated UUID.
- **Exact `TEXT` matching.** Codes are stored as supplied and compared literally.
  There is no trimming, case folding, `ILIKE`, Unicode normalization, whitespace
  normalization or aliasing at any entry point, so case and whitespace variants
  are distinct codes. Any future normalization is a separately reviewed
  schema/API change.
- **Replacement, not patch.** A `Patch...` input carries the complete approved
  mutable field set. For a nullable mutable field, omission and explicit `null`
  both store SQL `NULL`; retaining a value requires sending it.
- **Immutability.** A platform's `code` and `ownershipClass`, a measure's `code`,
  `category`, `unit`, `allowNegative`, `additiveAcrossTime` and
  `additiveAcrossWorks`, and a mapping's platform/measure identity cannot be
  changed and are absent from the patch inputs.
- **No delete.** Retirement is `enabled = false`, an ordinary audited update.
- **Audited and atomic.** Every committed create/update writes the canonical row
  and exactly one `metric_registry_history` row in one transaction, recording the
  authenticated actor and the exact persisted before/after state. Rejected,
  unauthorized, failed and genuine no-op requests write neither, and a no-op
  moves no canonical timestamp. The audit table is exposed nowhere in the schema.
- **Serialized last-write-wins.** Each update takes exactly one canonical-row
  `FOR UPDATE` lock. There is no optimistic-concurrency token: unlike publisher
  service configuration, no single coherent version token spans these three
  entities, and `metric_platform_measure` deliberately carries no timestamp.
- **Sanitized failures.** PostgreSQL remains authoritative for code uniqueness,
  pair uniqueness, foreign keys, nonblank CHECKs and the `supported_grains`
  CHECK. Reachable violations return bounded messages exposing no PostgreSQL
  text, constraint name, SQL, driver diagnostic or connection detail.

Downstream impact: the change is **strictly additive** — three object types, six
inputs, four additive enum exposures, six mutations and three queries, with no
existing type, field, nullability, enum variant or authorization behaviour
changed. No consumer requires a change to keep working; a consumer that wants to
administer the registry refreshes its generated client against the merged
contract.

Still deferred to separately specified work: publisher-platform approval
administration, checkpoint administration, the service/dashboard registry list
queries `metricPlatforms` and `metricMeasures`, any delete or bulk operation,
any audit-history query, and any real platform, platform-measure, source,
source-account or OPERAS seed. Source and source-account administration is
delivered by `MET-WP1-13` (section 2.2).

### 2.2 Delivered source and source-account administration (`MET-WP1-13`)

`MET-WP1-13` delivers administration of exactly `metric_source` and
`metric_source_account`. It does not administer `metric_source_checkpoint`.

Operations, all **SUPERUSER only** and all authorized before any
operation-specific database read, row lock or write:

```graphql
createMetricSource(data: NewMetricSource!): MetricSource!
updateMetricSource(data: PatchMetricSource!): MetricSource!
metricSourceByCode(code: String!): MetricSource!

createMetricSourceAccount(data: NewMetricSourceAccount!): MetricSourceAccount!
updateMetricSourceAccount(data: PatchMetricSourceAccount!): MetricSourceAccount!
metricSourceAccountByCode(code: String!): MetricSourceAccount!
```

Typed source-account configuration, the only configuration representation the
API accepts or returns:

```graphql
enum MetricSourceAccountConfigurationKind {
  EMPTY
  CLOUDFRONT_LEGACY_S3_V1
}

input MetricSourceAccountConfigurationInput {
  kind: MetricSourceAccountConfigurationKind!
  cloudfrontLegacyS3: MetricCloudFrontLegacyS3ConfigurationInput
}

input MetricCloudFrontLegacyS3ConfigurationInput {
  hostname: String!
  bucket: String!
  prefix: String!
}

type MetricSourceAccountConfiguration {
  kind: MetricSourceAccountConfigurationKind!
  cloudfrontLegacyS3: MetricCloudFrontLegacyS3Configuration
}

type MetricCloudFrontLegacyS3Configuration {
  hostname: String!
  bucket: String!
  prefix: String!
}
```

Contract properties consumers may rely on:

- **Stable code identity, exact `TEXT` matching.** As in section 2.1. Account
  creation names its source and platform by their exact stable codes; no
  operation requires a database-generated UUID.
- **Driver-key invariant.** `DRIVER` requires a non-blank `driverKey`; every
  other `acquisitionType` requires none. Enforced identically by the
  coordinator and by the `metric_source_driver_key_check` CHECK. No driver
  registry, driver-key uniqueness or approved driver value is implied.
- **Immutability.** A source's `code`, `acquisitionType` and `driverKey`, and
  an account's `code`, source, platform, `externalKey` and `expectedPublisherId`
  cannot be changed and are absent from the patch inputs. Correcting any of
  them is separately reviewed repair work; changing `code` alone never
  bypasses the `(source_id, external_key)` identity.
- **Replacement, not patch.** `PatchMetricSource` replaces exactly `enabled`,
  `defaultLookbackDays` and `defaultFinalizationDelayDays` (omitted and `null`
  both store SQL `NULL`); `PatchMetricSourceAccount` replaces exactly
  `configuration` and `enabled`.
- **Closed configuration.** `EMPTY` stores semantic `{}` and rejects a
  payload. `CLOUDFRONT_LEGACY_S3_V1` requires its payload and stores exactly
  `{"schemaVersion": "cloudfront-source-account/1", "hostname", "logging":
  {"mode": "LEGACY_S3", "bucket", "prefix"}}` with the caller's exact
  non-blank values; `schemaVersion` and `logging.mode` are derived, never
  caller-controlled; the hostname must equal the immutable `externalKey`
  exactly. A resolved `DRIVER` source with `driverKey = cloudfront` must use
  `CLOUDFRONT_LEGACY_S3_V1`; every other current source must use `EMPTY`. No
  raw JSON input, raw JSON output, alternate escape hatch or credential field
  exists; a new non-empty configuration for another source is a separately
  reviewed additive kind/version.
- **CloudFront expected-publisher pin (Specification Amendment 2).** Creating
  a `CLOUDFRONT_LEGACY_S3_V1` account requires a non-null `expectedPublisherId`
  naming an existing publisher, because the merged managed `DRIVER` ingestion
  coordinator fails closed unless the account's expected publisher is present
  and equals the import's publisher. `expectedPublisherId` stays nullable in
  the schema and the database for `EMPTY` and future kinds, and is immutable
  through `PatchMetricSourceAccount`.
- **Fail-closed stored configuration.** Every account returned, updated or
  audited has passed the closed decoder against its immutable source and
  `externalKey`. Any other stored value yields one fixed sanitized failure,
  discloses nothing stored, performs no update and writes no audit row.
- **Audited, atomic, no-op-silent, serialized.** As in section 2.1, against
  the separate append-only `metric_source_registry_history` (`SOURCE`,
  `SOURCE_ACCOUNT`; `CREATE`, `UPDATE`). Configuration equality is semantic
  JSONB equality, never key order, whitespace or bytes. Each update takes
  exactly one canonical-row `FOR UPDATE` lock and never locks the source,
  platform or publisher rows.
- **Sanitized failures.** Reachable constraint and validation failures return
  bounded messages exposing no PostgreSQL text, constraint name, SQL, driver
  diagnostic, connection detail or stored configuration.

Downstream impact: the change is **strictly additive** — two object types,
four input types plus the four typed configuration types, the additive
exposure of the existing `MetricSourceAcquisitionType` enum and the new
`MetricSourceAccountConfigurationKind` enum, four mutations and two queries,
with no existing type, field, nullability, enum variant or authorization
behaviour changed. No consumer requires a change to keep working.

Still deferred: checkpoint administration and claim/lease runtime, the
CloudFront collection driver, any list, search, delete or bulk source
operation, any audit-history query, and any real source, account or platform
row.

## 3. Internal ingestion

Thoth owns bounded claim/checkpoint/import/batch/rollup/export/reconciliation operations consumed by Sphinx.

Required: least privilege, leases, stale-token rejection, idempotency, bounded batches, sanitized errors, exact row classifications and fail-closed behavior.

### 3.1 Delivered minimum rollup application contract (`MET-WP4-01`)

`MET-WP4-01` delivers the rollup half of this contract: turning committed
canonical rollup deltas into the one MOM-1 work-level projection, with
exactly-once effect and a durable watermark.

Operations, both requiring exactly `METRICS_INGEST_SERVICE` and both
authorized before any rollup-specific database read, row lock, claim or write:

```graphql
claimMetricRollupDeltas(limit: Int!): [MetricRollupDeltaClaim!]!
completeMetricRollupDeltas(input: CompleteMetricRollupDeltasInput!): MetricRollupWatermark!
```

```graphql
type MetricRollupDeltaClaim {
  deltaId: Uuid!
  sequence: String!
  claimToken: Uuid!
  leaseExpiresAt: Timestamp!
}

input CompleteMetricRollupDeltasInput {
  claimToken: Uuid!
}

type MetricRollupWatermark {
  appliedThroughSequence: String!
  watermarkAt: Timestamp!
}
```

Contract properties consumers may rely on:

- **Strict single frontier.** One claim begins at exactly
  `appliedThroughSequence + 1` and takes a contiguous run of at most 50
  positions. It never skips a position held by a live claim, never reaches
  across a missing one, and never applies out of order. An empty result means
  "nothing is yours to take right now", not an error: a worker polls, it does
  not alert.
- **One token per batch.** Every row of one claim carries the same claim token
  and the same lease expiry. Completion addresses the batch by that token
  alone.
- **Server-fixed 900-second lease.** The duration is not a request parameter.
  An expired claim is reclaimable by any holder of the role, under a fresh
  token; the superseded token can no longer apply anything.
- **Derived, never supplied.** Completion accepts the claim token and nothing
  else. Every projected value, every aggregate dimension and the resulting
  watermark are derived inside Thoth from the durable claimed batch and the
  canonical records it names, so a claimant cannot choose what its completion
  adds, to which aggregate, or how far the watermark moves.
- **Whole-batch atomicity.** Projection updates, delta terminalization and the
  watermark advance commit in one PostgreSQL transaction. Any failure rolls
  the whole batch back: the projection, every delta's status and the watermark
  stay exactly as they were, and the batch remains claimed until its lease
  expires.
- **Idempotent replay.** Repeating an already-applied token returns the
  current watermark without writing, provided the whole batch is applied, was
  applied by the same principal, and sits at or below the durable frontier.
  That is what makes a completion which timed out *after* committing safe to
  retry. Stale, foreign and reclaimed tokens fail without mutation.
- **Fail-closed arithmetic.** Values are signed 64-bit. An application that
  would overflow or underflow aborts the whole batch rather than wrapping, and
  the frontier then blocks pending separately authorized repair. Nothing skips
  a poison position to restore progress.
- **Progress positions are strings.** `sequence` and `appliedThroughSequence`
  are decimal strings, because GraphQL's `Int` is 32-bit and a durable
  ordering identity must never be approximated.
- **Watermark meaning.** `appliedThroughSequence = W` means every committed
  work-day position `1..=W` is applied. It never moves backwards and never
  crosses an unapplied position. `watermarkAt` is when the current `W` was
  established — a fact about the boundary, not a claim that every canonical
  row is projected. A consumer detects outstanding rollup lag by comparing `W`
  with the highest allocated position, not by reading `watermarkAt`.

Scope boundaries a consumer must not infer:

- The MOM-1 progress stream covers exactly canonical records with
  `reporting_grain = DAY` spanning exactly one calendar day. Deltas of other
  grains are durable and pending, carry no position, and never block the
  work-day frontier; their projection and claim design is deferred.
- `metric_rollup_work_day` is the only projection delivered. The monthly,
  work-country-month and work-institution-month projections remain future
  architecture.
- There is no callable rebuild operation, and no read surface. Rollup
  projections, delta state and the watermark are reachable only through the
  two operations above; the coverage-aware read contract is `MET-WP4-02`.
- Sphinx orchestrates and never writes the Thoth database. All projection
  arithmetic executes inside Thoth.

Error surface (known limitation): rollup rejections — an out-of-range limit,
an unknown, stale, foreign or reclaimed token, an expired lease, a blocked
frontier and an overflowing application — are returned as GraphQL errors with
stable messages and the generic `INTERNAL_ERROR` extension type. `MET-WP4-01`
introduced no dedicated `thoth-errors` variant, so a consumer that needs to
classify these programmatically requires a separately specified stable error
code.

## 4. Dashboard/widget

Thoth owns entity metrics, dashboard/widget operations and registry queries.

Required response concerns: distinct measure totals, timeline, breakdowns, coverage, freshness, watermark, warnings, partial state, BigInt strings and deterministic pagination.

Semantics: OR within lists, AND between dimensions, exclusive end date, bounded ranges and filtering before pagination.

## 5. Publisher import

Flow: initialize private upload, direct browser upload, complete/queue, Sphinx claim/normalize, Thoth validate/commit, Sphinx complete, app reads result.

Publisher derives from authentication; object keys are server-selected; MIME/size/checksum/decompression are bounded; managed platforms are prohibited.

## 6. OPERAS

Thoth owns canonical export claims and ledgers. Sphinx projects/delivers. Configuration includes platform/measure mapping, event URI, measure URI, uploader URI, enabled state and finalization. Drivers never construct OPERAS payloads.

## 7. Authentication (service roles)

The shared machine-role architecture is decided:
[`ADR-0008`](../engineering/decisions/ADR-0008-machine-roles-and-durable-job-primitives.md)
(`APPROVED`, repository-authoritative) establishes dedicated, least-privilege,
**domain-specific** machine-role conventions — no generic catch-all service
role, an explicit policy guard and authorization matrix per machine role, and
no `SUPERUSER` shortcut for machine services.

### 7.1 Delivered minimum service-role contract (`MET-WP5-01`)

`MET-WP5-01` (issue #907) fixes the two MOM-1 Metrics service roles as
repository-authoritative contract. Both are **unscoped** ZITADEL project roles
implemented in the central policy module (`thoth-api/src/policy.rs`) as
`Role::MetricsIngestService` and `Role::MetricsReadService`, with the
predicates `UserAccess::is_metrics_ingest_service()` /
`UserAccess::is_metrics_read_service()` and the guards
`PolicyContext::require_metrics_ingest_service()` /
`PolicyContext::require_metrics_read_service()`. The `zitadel setup` bootstrap
role list (`src/bin/commands/zitadel.rs`) declares both role keys; see the
provisioning boundary below.

| Role code | Purpose | Complete MOM-1 protected operation set | Consuming slice |
| --- | --- | --- | --- |
| `METRICS_INGEST_SERVICE` | Sphinx orchestration caller for managed-source ingestion | `claimMetricSourceUnits`, `updateMetricSourceCheckpoint`, `beginMetricImport`, `ingestMetricBatch`, `completeMetricImport`, `claimMetricRollupDeltas`, `completeMetricRollupDeltas` | `MET-WP2-02` (ingestion, checkpoint and import lifecycle), `MET-WP4-01` (rollup-delta application) |
| `METRICS_READ_SERVICE` | Thoth-owned server-side Metrics query clients | `metricDashboard`, `metricMeasures`, `metricPlatforms` | `MET-WP4-02` |

#### Policy invariants (tested)

These are properties of the role, predicate and guard implementation in
`thoth-api/src/policy.rs`, each exercised by the tests colocated in that module
(the guard tests run under the `backend` feature):

1. Each predicate and guard is satisfied only by the presence of its exact role
   key. Case and whitespace variants of the keys, generic service keys and the
   deferred `METRICS_SYNC_SERVICE` key satisfy neither, and an unauthenticated
   caller fails both guards.
2. There is no role inheritance. `SUPERUSER`, `DISSEMINATION_WORKER` and the
   publisher-scoped roles satisfy neither Metrics guard; neither Metrics role
   satisfies the other; and neither Metrics role satisfies the `SUPERUSER` or
   `DISSEMINATION_WORKER` guard or implies either predicate.
3. Neither role confers any publisher-scoped role or any
   `PublisherPermissions`. Both are excluded from `publisher_org_ids()`, so a
   Metrics-machine-only account never appears to hold publisher organisations,
   while publisher-scoped roles held alongside them still contribute theirs.

#### Downstream contract requirements (specified, not tested here)

These are requirements this register places on the consuming slices —
`MET-WP2-02` (#908), `MET-WP4-01` (#909) and `MET-WP4-02` (#910).
`MET-WP5-01` neither implements nor tests them; each consuming slice satisfies
them under its own approved specification, authorization matrix and negative
tests:

1. The operation sets above are complete for MOM-1. Any addition, any other
   role code (including the sketched `METRICS_SYNC_SERVICE`, which remains a
   deferred proposal only) and any role composition requires its own approved
   specification.
2. `MET-WP5-01` defines the roles, predicates and guards only and wires no
   GraphQL operation to either role. Each listed operation is delivered, and
   authorized through the guard for its role, by the consuming slice named in
   the table.
3. Neither role supplies package/capability entitlement.
   `METRICS_READ_SERVICE` never supplies publisher entitlement for the data it
   may serve (for example `METRICS_DASHBOARD`); that remains a separate
   ADR-0001 check performed by the consuming operation.

#### Provisioning boundary

The bootstrap role list declares both role keys, and the bootstrap source adds
no Metrics role grant. `MET-WP5-01` did not execute the bootstrap and did not
inspect or change any identity-provider state; this register makes no claim
about live provider state. Live role provisioning, grants, credentials and
rotation remain separately governed operational actions outside this register
and outside the repository.

Consumers must not substitute superuser: `SUPERUSER` is not a machine-service
shortcut.

## 8. Cross-repository gate

Record exact producer commit/preview, generated types, contract fixtures, compatibility, merge order, non-production credentials and independent rollback before final review.
