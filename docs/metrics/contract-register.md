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
administration, checkpoint administration, any delete or bulk operation, any
audit-history query, and any real platform, platform-measure, source,
source-account or OPERAS seed. Source and source-account administration is
delivered by `MET-WP1-13` (section 2.2). The service registry list queries
`metricPlatforms` and `metricMeasures` are delivered by `MET-WP4-02` under
`METRICS_READ_SERVICE`, not by this administration surface (section 4.1).

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
  coordinator and by the `metric_source_driver_key_check` CHECK: "non-blank"
  means at least one character outside one explicit whitespace set, the 25
  code points of the Unicode `White_Space` property (U+0009..U+000D, U+0020,
  U+0085, U+00A0, U+1680, U+2000..U+200A, U+2028, U+2029, U+202F, U+205F,
  U+3000), spelled out at both boundaries so the decision never depends on a
  database locale. A valid key is stored exactly as supplied, surrounding
  whitespace included. No driver registry, driver-key uniqueness or approved
  driver value is implied.
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
- There is no callable rebuild operation. Delta state is reachable only
  through the two operations above. Projected values and the watermark time
  are read only through the `MET-WP4-02` coverage-aware `metricDashboard`
  (section 4.1), which exposes no projection row, delta, claim or sequence.
- Sphinx orchestrates and never writes the Thoth database. All projection
  arithmetic executes inside Thoth.

Error surface (known limitation): rollup rejections — an out-of-range limit,
an unknown, stale, foreign or reclaimed token, an expired lease, a blocked
frontier and an overflowing application — are returned as GraphQL errors with
stable messages and the generic `INTERNAL_ERROR` extension type. `MET-WP4-01`
introduced no dedicated `thoth-errors` variant, so a consumer that needs to
classify these programmatically requires a separately specified stable error
code.

### 3.2 Delivered managed-DRIVER ingestion lifecycle contract (`MET-WP2-02`)

`MET-WP2-02` (#908) delivers the claim, import and checkpoint half of this
contract for exactly one acquisition type, `DRIVER`: the protected transport
through which Sphinx runs one managed source unit restartably through Thoth.
Canonical ingestion itself is the unchanged `MET-WP2-01B` coordinator;
`PUBLISHER_UPLOAD`, `OPERAS` and `ADMIN_IMPORT` remain outside it.

Operations, all five requiring exactly `METRICS_INGEST_SERVICE` and all
authorized before any operation-specific database read, lock, claim or write:

```graphql
claimMetricSourceUnits(input: ClaimMetricSourceUnitsInput!): [MetricSourceUnitClaim!]!
beginMetricImport(input: BeginMetricImportInput!): MetricImport!
ingestMetricBatch(input: IngestMetricBatchInput!): MetricBatchResult!
completeMetricImport(input: CompleteMetricImportInput!): MetricImport!
updateMetricSourceCheckpoint(input: UpdateMetricSourceCheckpointInput!): MetricSourceCheckpoint!
```

`SUPERUSER`, `METRICS_READ_SERVICE`, `DISSEMINATION_WORKER`, publisher-scoped
roles and anonymous or failed-introspection callers are denied. The role never
supplies entitlement: an account is claimable, and an import can begin, only
when its pinned publisher's package grants `METRICS_COLLECT`.

The public GraphQL scalars are the repository's own: `Uuid`, `Timestamp` and
`Date`. `MetricImportStatus`, `MetricRecordProvenanceClassification` and
`MetricIngestionErrorCode` are the existing closed domain enums, exposed
unchanged; `MetricCoverageStatus` is reused from `MET-WP4-02`.

Contract:

- **Source unit.** One source account's single checkpoint
  `(source_account_id, "default")`. The partition is fixed by Thoth, never
  caller-selected, and the checkpoint is created lazily by the first claim
  with a conflict-safe insert.
- **Claim.** One transaction selects an enabled `DRIVER` source by exact code
  and its eligible accounts in ascending stable-code order: account enabled,
  platform enabled, a pinned publisher holding `METRICS_COLLECT`, and a stored
  configuration that the `MET-WP1-13` decoder accepts and that is compatible
  with the source. Unleased or expired checkpoints are locked
  `FOR UPDATE SKIP LOCKED` and leased. `limit` defaults to 10, is clamped to
  50 and claims nothing at or below 0; `leaseSeconds` defaults to 900 and is
  clamped to 60..3600. An ineligible account is skipped and gains no
  checkpoint.
- **Lease token.** Every claim or reclaim stores a fresh Thoth-generated UUID
  as canonical lowercase text in `metric_source_checkpoint.lease_owner`,
  returned only in that claim as `leaseToken`. Every later operation locks the
  checkpoint `FOR UPDATE` and requires that exact token and an unexpired
  lease; an unknown, foreign, expired, reclaimed or released token is
  `STALE_SOURCE_CLAIM` and changes nothing. `lease_owner` and `last_error` are
  never exposed, and `last_error` is never written. The checkpoint `cursor` is
  read and written only as section 3.3 defines, and is never exposed on
  `MetricSourceCheckpoint`.
- **Import envelope.** `beginMetricImport` creates the account's import for one
  one-day unit with status `PROCESSING`, the publisher taken from the account's
  pin, `created_by` from the authenticated principal, `raw_object_key` null,
  and a server-owned manifest
  `{"schemaVersion": "thoth-managed-driver-import/1", "manifestDigest", "expectedBatchKeys"}`
  whose keys are stored as an ascending set. A new import records
  `last_discovered_at`. A request for the same
  `(source_account_id, upstreamReportId)` returns the existing import in its
  current state, never reopened, when every immutable value and the manifest
  match; otherwise it is `IMPORT_IDEMPOTENCY_MISMATCH` and changes nothing.
  Bounds: `upstreamReportId` non-blank, at most 512 UTF-8 bytes; format,
  format version and normalizer version non-blank, at most 128 bytes;
  `rawSha256` and `manifestDigest` exactly 64 lowercase hexadecimal
  characters; 1 to 100 unique non-blank expected keys of at most 256 bytes.
- **Batch.** `ingestMetricBatch` maps the input one-to-one onto the
  `thoth-normalized-metrics/1` coordinator request; `value` must be a
  canonical base-10 signed 64-bit integer string and `sourceRowNumber`
  non-negative. The batch key must be one of the import's expected keys. A
  short guard transaction locks and validates the checkpoint and, still holding
  that lock, calls the unchanged coordinator on a second pooled connection, so
  no reclaim can replace the token during the call. The coordinator's limits,
  hashing, classifications, replay and idempotency apply unchanged; a replay of
  a committed batch returns its persisted outcome with `replayed: true`.
- **Completion.** `completeMetricImport` requires the import's committed batch
  keys to equal its expected set exactly (`IMPORT_INCOMPLETE` while any is
  missing) and sets `COMPLETED` when the coordinator's persisted invalid and
  conflict counters are zero, otherwise `COMPLETED_WITH_ERRORS`, with
  `completed_at` at the transaction timestamp. A terminal import is returned
  unchanged. `FAILED` is never manufactured.
- **Checkpoint progress.** `updateMetricSourceCheckpoint` accepts only a
  terminal one-day managed import. It moves `last_completed_at` forward to the
  import's `completed_at` and releases the lease. It moves
  `last_successful_period_end` forward to the import's period end only when
  the import is `COMPLETED` **and** has at least one coverage row **and** every
  coverage row covers exactly the import's period **and** is `COMPLETE`. Zero
  coverage rows, a mismatched period, `PARTIAL`, `UNKNOWN` or
  `COMPLETED_WITH_ERRORS` never advance it. Neither value moves backwards. A
  repeat after the lease was already released returns the checkpoint without
  writing when this import's progress is already recorded; otherwise it is
  `STALE_SOURCE_CLAIM`.
- **Errors.** Lifecycle failures use `extensions.type` from exactly
  `SOURCE_NOT_FOUND`, `SOURCE_ACCOUNT_NOT_FOUND`, `SOURCE_NOT_ELIGIBLE`,
  `METRICS_COLLECT_NOT_ENTITLED`, `STALE_SOURCE_CLAIM`, `IMPORT_NOT_FOUND`,
  `IMPORT_IDEMPOTENCY_MISMATCH`, `UNEXPECTED_BATCH_KEY`, `IMPORT_INCOMPLETE`,
  `INVALID_IMPORT_STATE` and `LIFECYCLE_LIMIT_EXCEEDED`, with fixed messages.
  A coordinator request failure carries its `MetricIngestionErrorCode` value
  unchanged, and a database failure is `INTERNAL_DATABASE_ERROR`. No message
  carries SQL, a constraint name, connection detail, a token, source
  configuration or caller input.

No migration, table, column or index is added: the existing checkpoint lease
columns and import idempotency index are the whole durable boundary. There is
no second queue, no process-local claim state and no generic job abstraction.

### 3.3 Claim context and period-manifest cursor (`MET-WP2-03`)

`MET-WP2-03` (#924) extends the section 3.2 lifecycle with the claim-time
context a real managed `DRIVER` needs and a bounded, producer-owned checkpoint
cursor. It adds no operation, input field, enum value, error code, role, table,
column or index: the five operations and their authorization are exactly those
of section 3.2.

The generated SDL gains exactly these declarations:

```graphql
type MetricPeriodManifestCursor {
  schemaVersion: String!
  entries: [MetricPeriodManifestCursorEntry!]!
}

type MetricPeriodManifestCursorEntry {
  periodStart: Date!
  manifestDigest: String!
}

# on MetricSourceUnitClaim only
platformCode: String!
periodManifestCursor: MetricPeriodManifestCursor
```

Contract:

- **Claimed context.** A claim already carries the source (`code`, `driverKey`,
  `defaultLookbackDays`, `defaultFinalizationDelayDays`) and the source account
  (`code`, typed `configuration`) it revalidated under its locks; consumers
  select those existing fields, which are not duplicated. `platformCode` is
  copied inside the claim transaction from the platform row the claim locked
  `FOR SHARE` and found enabled, and is never resolved afterwards.
- **Representation.** The only durable cursor is
  `metric_source_checkpoint.cursor`, whose database type stays generic JSONB.
  SQL `NULL` is the only representation of no accepted history and is returned
  as `periodManifestCursor: null`. Any other stored value must be exactly
  `{"schemaVersion": "thoth-period-manifest-cursor/1", "entries": [{"periodStart": "YYYY-MM-DD", "manifestDigest": "<64 lowercase hexadecimal characters>"}]}`
  with no other member at either level, 1 to 64 entries strictly ascending and
  unique by `periodStart`, and every `periodStart` the canonical ten-character
  rendering of one valid date. It holds no source object, request, viewer or
  credential data.
- **Absence means unknown.** 64 is a storage bound, not a lookback limit. A
  period that is absent, whether never recorded or evicted, is unknown and may
  be rediscovered; absence never means unchanged.
- **Claim-time validation.** For each unit still eligible after the locked
  revalidation, the stored cursor is decoded while its checkpoint is locked and
  before the lease is written. A stored cursor outside the representation fails
  the whole claim as `INTERNAL_STATE_INCONSISTENCY`: no lease is granted and
  nothing is repaired. A unit skipped as ineligible is not decoded.
- **Server-derived advancement.** No caller supplies cursor state, and
  `UpdateMetricSourceCheckpointInput` remains exactly `importId` and
  `leaseToken`. When the live holder records an import that satisfies the
  section 3.2 successful-period predicate, the single `UPDATE` that records
  progress and releases the lease also inserts or replaces the entry for the
  import's `period_start`, using the `manifestDigest` of its strict
  `thoth-managed-driver-import/1` envelope, and evicts the chronologically
  oldest entries beyond 64. `COMPLETED_WITH_ERRORS`, zero coverage rows, a
  mismatched period, `PARTIAL` and `UNKNOWN` leave the cursor unchanged;
  non-terminal and `FAILED` imports and stale, expired, reclaimed or foreign
  tokens change nothing. Successful reprocessing of an older period may replace
  its digest and never moves `last_successful_period_end` backwards.
- **Replay.** A repeat after release stays read-only. It returns the checkpoint
  without writing when progress is already recorded, the cursor plays no part in
  that decision, and an entry evicted later is never resurrected.
- **Fail-closed stored state.** A terminal import whose stored envelope does
  not decode, and a live update over a stored cursor outside the
  representation, fail as `INTERNAL_STATE_INCONSISTENCY` with no cursor,
  progress or lease write. The client-facing message is the fixed coordinator
  message and carries no stored value.
- **Reachability.** The two cursor objects are reachable only through
  `MetricSourceUnitClaim.periodManifestCursor`, and `platformCode` is an output
  field of the claim only. `MetricSourceCheckpoint`, `MetricSource`,
  `MetricSourceAccount`, `MetricPlatform`, `QueryRoot` and the superuser
  registry lookups expose no cursor state.

An existing stored cursor outside the representation is neither migrated nor
rewritten; a claim of its eligible unit fails closed until a separately
authorized repair.

## 4. Dashboard/widget

Thoth owns entity metrics, dashboard/widget operations and registry queries.

Required response concerns: distinct measure totals, timeline, breakdowns, coverage, freshness, watermark, warnings, partial state, BigInt strings and deterministic pagination.

Semantics: OR within lists, AND between dimensions, exclusive end date, bounded ranges and filtering before pagination.

### 4.1 Delivered minimum protected read contract (`MET-WP4-02`)

`MET-WP4-02` (issue #910) delivers the MOM-1 read surface: one coverage-aware
dashboard query and the two registry lists a service needs to form its UUID
filters.

Operations, each requiring exactly `METRICS_READ_SERVICE` through
`PolicyContext::require_metrics_read_service()` before any Metrics-specific
database access:

```graphql
metricDashboard(input: MetricDashboardInput!): MetricDashboard!
metricMeasures: [MetricMeasure!]!
metricPlatforms: [MetricPlatform!]!
```

```graphql
scalar BigInt

input MetricDashboardInput {
  selector: MetricSelectorInput!
  startDate: Date!
  endDate: Date!
  measures: [Uuid!]
  platforms: [Uuid!]
  timelineGrain: MetricTimelineGrain = "AUTO"
}

input MetricSelectorInput {
  publisherIds: [Uuid!]
}

enum MetricTimelineGrain { AUTO DAY MONTH }

type MetricDashboard {
  totals: [MetricTotal!]!
  timeline: [MetricTimeBucket!]!
  coverage: MetricCoverage!
  asOf: Timestamp!
  dataThrough: Date
  rollupWatermark: Timestamp!
  warnings: [MetricWarning!]!
  isPartial: Boolean!
}

type MetricTotal { platformId: Uuid! measureId: Uuid! value: BigInt }
type MetricTimeBucket {
  platformId: Uuid! measureId: Uuid! startDate: Date! endDate: Date! value: BigInt
}
type MetricCoverage { status: MetricCoverageStatus! items: [MetricCoverageItem!]! }
type MetricCoverageItem {
  platformId: Uuid! measureId: Uuid! status: MetricCoverageStatus!
  dataThrough: Date countryCoverage: Boolean! institutionCoverage: Boolean!
}
enum MetricCoverageStatus { COMPLETE PARTIAL UNKNOWN }
type MetricWarning { code: MetricWarningCode! message: String! }
enum MetricWarningCode { PARTIAL_COVERAGE UNKNOWN_COVERAGE ROLLUP_LAG }
```

The approved design's semantic `UUID` and `DateTime` are the repository's
existing `Uuid` and `Timestamp` scalars. `BigInt` is the one new scalar. The
generated SDL renders the enum input default as `"AUTO"`, which is Juniper's
rendering of every enum default in this schema.

Contract properties consumers may rely on:

- **Authorization matrix.** `METRICS_READ_SERVICE` is allowed, including
  alongside unrelated roles. Anonymous callers, authenticated callers without
  the role, `PUBLISHER_USER`, `PUBLISHER_ADMIN`, `WORK_LIFECYCLE`,
  `CDN_WRITE`, `SUPERUSER`, `DISSEMINATION_WORKER` and
  `METRICS_INGEST_SERVICE` are denied with `NO_ACCESS`, before any Metrics
  read, and never receive an empty or zero result.
- **Publisher entitlement is separate.** `metricDashboard` also requires every
  selected publisher's package to grant `METRICS_DASHBOARD` under ADR-0001.
  An existing publisher without it is `NO_ACCESS`. The role never supplies the
  capability, and no package name appears in Metrics code. The registry lists
  need no publisher and no entitlement.
- **Plural selector, one publisher in MOM-1.** `publisherIds` is plural, and
  the read is written over a publisher set. MOM-1 serves exactly one publisher
  per request: none, several or an unknown ID is `METRIC_QUERY_INVALID`, never
  a first-item choice, a truncation or a combination. That is a milestone
  restriction, not a statement that a dashboard can only represent one
  publisher. Serving authorized groups of publishers needs its own reviewed
  entitlement and aggregation contract.
- **Attribution.** Works are attributed to publishers through current
  `work -> imprint -> publisher` metadata at read time. A moved work's
  projected history follows it; nothing is rewritten.
- **Request bounds.** Half-open `[startDate, endDate)` with
  `startDate < endDate`; at most 366 days; at most 10 unique measures and 10
  unique platforms, whether explicit or resolved; at most 25 platform/measure
  combinations (the cross product of the served platforms and measures); at
  most 5,000 timeline cells (combinations x buckets). A bound is never met by
  truncation. Duplicate IDs are refused, and so is an explicit unknown
  measure or platform. Disabled registry identities stay addressable.
- **Omitted filters.** An omitted or empty `measures` or `platforms` resolves
  to every identity represented for the selected publishers and range, either
  in the work-day projection or in terminal coverage from an eligible managed
  account, restricted by the other dimension when that one is explicit.
- **Additivity.** Totals sum across works and time, so every served measure,
  explicit or resolved, must be `additiveAcrossTime` and
  `additiveAcrossWorks`. Otherwise the whole request is
  `METRIC_QUERY_INVALID`; the measure is never summed or dropped.
  `metricMeasures` still lists such measures.
- **Grains.** `DAY` gives one bucket per requested day. `MONTH` gives calendar
  months clipped to the range, each the exact sum of its daily rows. `AUTO` is
  `DAY`.
- **Dimensional representation (Specification Amendment 6).** The projection
  keeps optional publication, country and institution dimensions. For each
  base cell `(work, platform, measure, day)`, an undimensioned row is the
  aggregate and no dimensioned row of that cell is added to it. Without one,
  every row of the cell must share one presence mask of the three dimensions,
  and those rows are summed. Different masks without an undimensioned row make
  the whole request `MOM1_DIMENSION_SCOPE_AMBIGUOUS`; nothing is guessed,
  chosen or summed across them. Base cells of different works resolve
  independently. This is a MOM-1 serving restriction, not a permanent rule for
  later explicit dimension selection.
- **Values.** Totals and buckets are kept per platform and measure; nothing is
  combined across either. A cell with projected rows returns their exact sum,
  whatever the coverage. A cell without rows returns `"0"` only when every day
  of it is effectively `COMPLETE` and no unapplied work-day delta touches it;
  otherwise `null`. `BigInt` is a canonical base-10 string: no `Int`, no
  float, signed and exact, with checked arithmetic that fails rather than
  wraps.
- **Coverage.** Per platform, the eligible source is the one enabled `DRIVER`
  account on an enabled source whose `expected_publisher_id` is the selected
  publisher. Two or more is `MOM1_SOURCE_SCOPE_AMBIGUOUS`. None leaves
  projected values served and coverage `UNKNOWN`. For each day, the current
  assertion comes from `COMPLETED` or `COMPLETED_WITH_ERRORS` imports with a
  recorded `completed_at`: latest completion first, then greatest import UUID.
  Only if both tie within one self-contradictory import is its most
  conservative assertion preferred. `COMPLETED_WITH_ERRORS` downgrades
  `COMPLETE` to `PARTIAL`. A day without an assertion is `UNKNOWN`. An item is
  `COMPLETE` only if every day is, `UNKNOWN` if any day is, otherwise
  `PARTIAL`. When any value served for a platform and measure was taken from
  country rows, every day of that combination asserted `COMPLETE` without
  country coverage is `PARTIAL` for it, and likewise for institution rows.
  That includes days without a value, whose cells are therefore `null`, not
  `"0"`. Publication rows have no coverage flag, so they rely on the ordinary
  status. `countryCoverage` and `institutionCoverage` are true only if every
  day has a current assertion including that dimension. Top-level status is
  `COMPLETE` only if every item is, and `UNKNOWN` if any item is, or if
  nothing is served.
- **One snapshot.** Everything is read on one connection in one
  `READ ONLY, REPEATABLE READ` transaction: entitlement, the `MET-WP4-01`
  frontier, scope, projection, coverage and outstanding rollup work.
  `asOf` is that transaction's timestamp. `rollupWatermark` is exactly the
  frontier's `watermark_at`. The read never advances, claims, completes,
  repairs or records anything.
- **Freshness.** `ROLLUP_LAG` means at least one work-day delta above the
  applied frontier intersects the selected publishers, range, platforms and
  measures. Unrelated backlog is not lag. An item's `dataThrough` is the last
  day of the unbroken run from `startDate` in which every day is `COMPLETE`
  and no such delta falls in `[startDate, D + 1 day)`, or `null` if the first
  day fails. Backlog before `startDate` does not reduce it. Top-level
  `dataThrough` is the earliest item value, and `null` unless every item has
  one.
- **Warnings.** At most one per code, in the order `UNKNOWN_COVERAGE`,
  `PARTIAL_COVERAGE`, `ROLLUP_LAG`, with fixed text. `isPartial` is true
  exactly when a warning is present.
- **Registry lists.** Every row, enabled or disabled, ordered by exact `code`
  compared byte-wise, at most 500. A larger registry is
  `METRIC_REGISTRY_LIMIT_EXCEEDED`, never a truncated list. They return the
  existing `MetricMeasure` and `MetricPlatform` types and no source
  configuration.
- **Errors.** Every failure carries a fixed message and one `extensions.type`
  among `NO_ACCESS`, `METRIC_QUERY_INVALID`, `METRIC_QUERY_LIMIT_EXCEEDED`,
  `METRIC_REGISTRY_LIMIT_EXCEEDED`, `MOM1_SOURCE_SCOPE_AMBIGUOUS` and
  `MOM1_DIMENSION_SCOPE_AMBIGUOUS`. An
  aggregate outside the representable range is `METRIC_QUERY_LIMIT_EXCEEDED`.
  An unexpected database failure is `INTERNAL_ERROR` without detail. No SQL,
  database diagnostic, token or source configuration is returned.

Scope boundaries a consumer must not infer:

- Countries, institutions, works sections, pagination, the deferred selector
  dimensions (`imprintIds`, `seriesIds`, `workIds`, `dois`, `workTypes`,
  `languages`, funding and affiliation institutions, `includeDescendants`),
  `metricWidget` and entity-level Metrics fields are not part of this
  contract. Adding them with their approved names is additive and needs its
  own specification.
- The read calls no external service. Browser clients must not hold the
  `METRICS_READ_SERVICE` credential; they reach this contract through a
  Thoth-owned server route.
- The change is strictly additive. No existing type, field, argument,
  nullability, enum value or authorization changed.
  `MetricCoverageStatus` is the existing database enum, now exposed to
  GraphQL without any value change.

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
