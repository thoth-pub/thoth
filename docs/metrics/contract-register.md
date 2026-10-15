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
- The two normal rollup operations above remain the orchestration surface:
  Sphinx claims a bounded batch and completes it, while Thoth applies the
  work-day changes and continuously recomputes the affected
  `MET-WP4-03A` monthly projection keys inside that same completion
  transaction (section 3.6). There is no end-of-month rebuild job.
- A full monthly rebuild is exceptional derived-state maintenance: initial
  historical population after the 03A migration, separately authorized
  repair/recovery, or explicit validation. It is not monthly processing.
  `MET-WP4-03A-OPS-02` (section 3.7) delivers the protected internal
  verification and rebuild operations that Sphinx orchestrates for that
  purpose; delta state remains reachable only through the two normal
  operations above, and neither maintenance operation reads or writes a
  delta.
- Sphinx orchestrates and never writes the Thoth database or supplies monthly
  projection values. All normal projection arithmetic and every full rebuild
  execute inside Thoth.

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
  mismatched period, `PARTIAL` and `UNKNOWN` leave the cursor unchanged, with
  the single narrow exception of the section 3.4 CloudFront quarantine-only
  import; non-terminal and `FAILED` imports and stale, expired, reclaimed or
  foreign tokens change nothing. Successful reprocessing of an older period may
  replace its digest and never moves `last_successful_period_end` backwards.
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

### 3.4 CloudFront unresolved-DOI quarantine (`MET-WP7-PREREQ-02`)

`MET-WP7-PREREQ-02` (#930) makes Thoth the owner of durable evidence for a
CloudFront observation whose DOI is syntactically valid but resolves to no
Thoth work, and adds one narrow source-progress exception for it. It adds no
operation, field, input, enum value, error code, role or entitlement, and no
consumer SDL regeneration is required. This is a CloudFront-specific source
policy, not a universal Metrics identifier rule.

Contract:

- **Unchanged classification.** Such an observation is still `REJECTED` with
  reason `UNKNOWN_DOI` in the batch result and in `metric_record_provenance`,
  which remains the sole authoritative per-row classification. It creates no
  `metric_record`, `metric_record_revision` or rollup delta, it is counted in
  `invalid_count`, and its import is still `COMPLETED_WITH_ERRORS`. Quarantine
  is not canonical acceptance.
- **Durable quarantine.** When, and only when, the coordinator's locked source
  has `driver_key` exactly `cloudfront` and the observation carries none of
  `publicationIsbn`, `publicationType`, `institutionRor`, `sourceRecordId` or
  `sourceRowNumber`, the same transaction that writes the rejected provenance,
  its sanitized import error and the invalid counter also writes exactly one
  `metric_identifier_quarantine` row linked to that provenance. It keeps the
  source account, platform, measure, schema version, the DOI byte for byte as
  supplied, the period, grain, optional country, value and methodology, and no
  request identity, raw log row, routing or credential. No caller flag, code,
  hostname or routing value can request or infer quarantine. `INVALID_DOI`,
  every other rejection and every non-CloudFront source are never quarantined.
  A batch replay writes nothing and never duplicates a quarantine row.
- **Quarantine-only manifest acceptance.** For a CloudFront
  `COMPLETED_WITH_ERRORS` import only, `updateMetricSourceCheckpoint` derives
  its rejected, `UNKNOWN_DOI`, quarantine and quarantined-`UNKNOWN_DOI` counts
  from provenance and quarantine rows. When nothing conflicted, every invalid
  row is a quarantined `UNKNOWN_DOI` rejection, and every coverage row is
  `COMPLETE` for exactly the import's one-day period, the update records
  `last_completed_at`, records or replaces that period's entry in the
  section 3.3 cursor under its unchanged rules, and releases the lease, while
  the import remains canonically unsuccessful and `last_successful_period_end`
  does not move. Coverage keeps describing source evidence, so
  `COMPLETE` source evidence with pending identifier quarantine is a valid
  state.
- **Everything else is unchanged.** A conflict, any other rejection reason, or
  any `UNKNOWN_DOI` rejection without quarantine keeps the ordinary
  `COMPLETED_WITH_ERRORS` release: progress recorded, lease released, cursor
  and successful period untouched. `COMPLETED_WITH_ERRORS` in general never
  records cursor progress, and non-CloudFront imports never evaluate this
  exception.
- **Fail-closed evidence.** A rejected-row count different from
  `invalid_count`, a quarantine row linked to anything other than a `REJECTED`
  / `UNKNOWN_DOI` provenance row, or a rejected row whose reason is absent,
  null, not a string or outside the closed `MetricIngestionErrorCode`
  vocabulary fails the update as `INTERNAL_STATE_INCONSISTENCY` with no
  progress, cursor, successful-period or lease write, and the message carries
  no stored value.
- **Locks and replay.** The update locks the checkpoint `FOR UPDATE`, then the
  import, the source account and the source, each `FOR SHARE`. The whole
  predicate is derived before the live/released split: a repeat after release
  stays read-only and recognizes a recorded quarantine-only import from
  `last_completed_at` alone, never from the cursor.

### 3.5 Automatic unresolved-DOI reconciliation (`MET-WP7-PREREQ-03`)

`MET-WP7-PREREQ-03` (#935) adds one protected, bounded reconciliation sweep
over the immutable evidence from section 3.4:

```graphql
reconcileMetricIdentifierQuarantine(
  limit: Int!
): MetricIdentifierQuarantineReconciliationBatch!

type MetricIdentifierQuarantineReconciliationBatch {
  attempted: Int!
  resolved: Int!
  pending: Int!
  blocked: Int!
}
```

The operation requires exactly `METRICS_INGEST_SERVICE`, accepts only
`limit` in `1..=50`, and exposes aggregate counts only. Row selection,
retry timing, DOI/Work authority, canonical identity, source ownership and
outcome are entirely server-owned.

Each quarantine row is attempted in its own transaction under
`FOR UPDATE SKIP LOCKED`. Historical quarantine, provenance, import counters,
status, coverage and checkpoint state are never rewritten. Reconciliation
state is additive in
`metric_identifier_quarantine_reconciliation`; resolved states are terminal,
while pending/blocked states use the server-owned 1h, 2h, 4h, 8h, 16h, then
24h-capped retry schedule. Deterministically contradictory historical evidence
becomes retryable `BLOCKED_INCONSISTENT_EVIDENCE` without canonical effect;
an unexpected database/internal failure rolls that row back and returns a
fixed redacted error.

Current DOI -> Work metadata remains authoritative. A current Work must still
belong to the publisher pinned on the original import. Reconciliation reuses
ordinary ingestion's canonical identity/content hashing, dimensional-cell
advisory lock, overlap lookup, existing-record/current-revision locking,
winner/duplicate/source-conflict/revision application and checked rollup-delta
authority. Historical same-source content is ordered by the immutable original
import `created_at`: older evidence is terminal `RESOLVED_SUPERSEDED`, newer
evidence may revise, the same import is blocked, and distinct equal timestamps
are blocked rather than ordered by UUID.

The exact durable state vocabulary is:
`PENDING_UNKNOWN_DOI`;
`BLOCKED_AMBIGUOUS_DOI`, `BLOCKED_PUBLISHER_SCOPE_MISMATCH`,
`BLOCKED_SOURCE_CONFLICT`, `BLOCKED_OVERLAPPING_PERIOD`,
`BLOCKED_SAME_IMPORT_ORDER`, `BLOCKED_IMPORT_ORDER_AMBIGUOUS`,
`BLOCKED_DELTA_OVERFLOW`, `BLOCKED_INCONSISTENT_EVIDENCE`;
and terminal `RESOLVED_WINNER`, `RESOLVED_DUPLICATE`,
`RESOLVED_REVISION`, `RESOLVED_SUPERSEDED`.

Identifier-quality read warnings remain separately owned by
`MET-WP7-PREREQ-04` (#936). This reconciliation contract does not activate
CloudFront collection, execute a production migration, or change any provider
configuration.

### 3.6 Derived monthly serving projections (`MET-WP4-03A`)

`MET-WP4-03A` (issue #940) adds the derived monthly serving foundation
beneath the `MET-WP4-01` completion transaction. It changes no operation,
type, field, argument, scalar, role or entitlement of sections 3.1 or 4.1,
and it adds no reader: this section records durable derived state and the
rules that maintain it, for the separately specified `MET-WP4-03B` read
contract to consume.

Objects, created empty by migration `20260923_v1.9.0`, all derived and
rebuildable:

```text
metric_rollup_work_month
  identity: UNIQUE NULLS NOT DISTINCT (work_id, publication_id, platform_id, measure_id, month_start)
  value bigint, requires_country_coverage, requires_institution_coverage, watermark

metric_rollup_work_country_month
  identity: UNIQUE NULLS NOT DISTINCT (work_id, publication_id, platform_id, measure_id, month_start, country_code)
  value bigint, requires_institution_coverage, watermark

metric_rollup_work_institution_month
  identity: UNIQUE NULLS NOT DISTINCT (work_id, publication_id, platform_id, measure_id, month_start, institution_id)
  value bigint, requires_country_coverage, watermark

metric_rollup_work_month_ambiguity
  identity: UNIQUE (work_id, platform_id, measure_id, month_start)
  total_ambiguous, country_ambiguous, institution_ambiguous (at least one true), watermark
```

Every table carries a UUID primary key, a first-of-month check on
`month_start`, a positive-watermark check, non-cascading foreign keys to the
represented work, publication, platform, measure and institution rows, the
two-uppercase-letter country-code check where a country is represented, and
no secondary index. The accepted `MET-WP4-03-BENCH-01` evidence rejected
every candidate secondary index; adding one requires exact-head query-plan
evidence and an amendment.

Contract properties a later consumer may rely on:

- **Derived, never canonical.** Canonical Metrics authority remains
  `metric_record`, `metric_record_revision`, the durable rollup deltas and
  the `MET-WP4-01` work-day frontier. The four monthly tables are written only
  by `completeMetricRollupDeltas`, inside its transaction and beneath its
  `metric_rollup_work_day_state` lock, and their only source is
  `metric_rollup_work_day`. Nothing reads canonical records, revisions or
  deltas to derive a monthly row, and nothing rewrites a canonical value to
  repair a derived one.
- **Day is resolved before month.** For every daily base cell
  `(work, platform, measure, day)` the representation is resolved first,
  and only the resolved daily contributions are summed into the month. Raw
  day rows are never compacted into a month and then re-resolved; the
  raw-month design was falsified by the benchmark.
- **Total representation, unchanged from section 4.1.** An undimensioned
  row is authoritative for its base cell. Without one, exactly one
  represented optional-dimension mask is summed. Otherwise the cell is
  `total_ambiguous` and contributes nothing to `metric_rollup_work_month`.
- **Country and institution representation.** Among the represented masks
  that contain the target dimension, the unique least mask under set
  inclusion is selected; its non-target dimensions are summed away. No
  qualifying mask contributes no value. Two incomparable minimal masks
  (`{country, institution}` beside `{publication, country}` for country;
  `{country, institution}` beside `{publication, institution}` for
  institution) contribute nothing and set `country_ambiguous` or
  `institution_ambiguous`. The least-representation rule is never applied to
  totals.
- **Publication identity is additive.** A monthly row retains
  `publication_id` exactly when the selected daily representation carries
  it, and is `NULL` otherwise. Within one month a `NULL` row and
  publication-specific rows arising from different resolved days are
  separate, additive contributions; no month-level precedence is applied
  between them.
- **Coverage dependency flags.** `requires_country_coverage` and
  `requires_institution_coverage` are the section 4.1 range-wide dependency
  semantics, OR-ed across the resolved daily contributions grouped into a
  row: true when at least one contributing representation was broken down by
  that dimension. Country rows carry only `requires_institution_coverage`;
  institution rows carry only `requires_country_coverage`. They are semantic
  state a reader must honour, not performance hints.
- **Sparse ambiguity.** One `metric_rollup_work_month_ambiguity` row exists
  per `(work, platform, measure, month)` while at least one flag is true,
  including for a month that has no value row in the ambiguous section. The
  three flags are independent. Ambiguity never blocks completion or frontier
  progress; when every flag clears, the row is removed rather than kept for
  its old watermark. A reader must treat a true flag as fail-closed
  evidence for that section.
- **Exact row watermarks.** A value row's `watermark` is the greatest
  `metric_rollup_work_day.watermark` over exactly the current resolved daily
  source rows that contribute to it; ignored alternative representations do
  not advance it. An ambiguity row's `watermark` is the greatest watermark
  over the union of daily source rows whose current represented state
  establishes any currently-true flag. A row watermark is local
  derived-state evidence only: it is never a serving boundary, and it never
  exceeds the `metric_rollup_work_day_state.applied_through_sequence` the
  same completion establishes. The only safe global serving frontier remains
  that `applied_through_sequence`.
- **Fixed-statement set-wise maintenance.** After the approved per-delta
  work-day application, the completion derives the distinct affected
  `(work, platform, measure, month)` keys of the batch, passes the whole set
  as one relational input (four parallel `unnest` arrays) and recomputes the
  four monthly datasets for exactly those keys in nine SQL statements — one
  source-watermark bound, four keyed deletes, four inserts — whose count is
  identical for 1, 10 and 50 affected keys. The claim batch remains at most
  50. Only then are the deltas terminalized and the frontier advanced.
- **Atomic with the completion.** Work-day updates, monthly recomputation,
  delta terminalization and the frontier advance commit together. A failure
  anywhere in the monthly recomputation rolls the whole transaction back:
  work-day rows, monthly rows, delta status and the frontier stay exactly as
  they were, and the batch remains claimed until its lease expires. A
  timeout-after-commit replay of an applied token stays read-only and
  rewrites no monthly row. Values are signed 64-bit and a monthly sum that
  overflows fails the batch closed; nothing wraps or narrows.
- **Rebuild from work-day state only.** The reviewed full-rebuild procedure
  locks the same state row, checks that no day row is watermarked above the
  durable frontier, truncates the four tables, reads every month key
  represented in `metric_rollup_work_day`, and replays the completion's own
  nine-statement keyed recomputation over those keys in chunks of at most 50.
  It executes the same statements, resolution text and index paths as
  incremental maintenance and reads nothing else. At the same work-day state
  and frontier it reproduces incrementally maintained identities, values,
  dependency flags, ambiguity flags and row watermarks exactly, in both
  directions. `MET-WP4-03A-OPS-02` (section 3.7) exposes this procedure as
  the protected `rebuildMetricRollupMonths` operation, guarded by an
  independent verification before and after; it remains exceptional
  maintenance, never normal monthly processing.
- **Initially empty; activation is gated.** The migration populates
  nothing, which is safe because `MET-WP4-03A` adds no reader. In test and
  production the approved Thoth deployment runs pending migrations through
  `thoth init` before starting the API; this automatic migration side effect
  is expected and must be named in the deployment authorization rather than
  replaced by an artificial pre-deployment migration run. No consumer may
  serve from the monthly tables until the migration has succeeded, Sphinx has
  orchestrated a separately authorized full historical rebuild through the
  protected Thoth operation, and the rebuilt state has been independently
  reconciled — exact values, dependency flags, ambiguity state and row
  watermarks — at a recorded `applied_through_sequence`. `MET-WP4-03B`
  serving activation and downstream dashboard cutover remain prohibited while
  the tables are empty, partially rebuilt or unreconciled. After a downgrade
  and reapplication the same gate applies again, because normal completion
  maintains only the months it subsequently touches.
- **No native MONTH or REPORTING_PERIOD serving.** These projections are
  aggregations of resolved `DAY`-grain work-day rows. Canonical records of
  any other grain remain outside the work-day stream and are not projected
  here; native monthly serving of such records is separately specified.

Scope boundaries a consumer must not infer:

- No GraphQL operation, type, field, argument or scalar was added or
  changed; at `MET-WP4-03A`, `metricDashboard` (section 4.1) still served
  from `metric_rollup_work_day` only. The monthly tables, their flags and
  ambiguity state reached no API in `MET-WP4-03A`; their one reader is the
  `MET-WP4-03B` dashboard contract of section 4.2, which reads them and
  never writes them.
- The only monthly maintenance surface is the pair of protected operations
  of section 3.7: a read-only consistency verification and an exceptional,
  self-verifying full rebuild. There is no repair, row-level patching or
  generic full-projection reconciliation-ledger operation. No
  `metric_rollup_work_month_state`, `metric_work_dimension` or secondary
  index is implied.
- Sphinx owns orchestration, including the initial/exceptional rebuild and
  any periodic verification workflow, but never writes the Thoth database or
  computes monthly projection values. Normal incremental maintenance and
  every full rebuild remain Thoth-owned database operations. A verification
  mismatch must alert/HOLD rather than silently auto-rebuild; Thoth never
  invokes a rebuild on its own.

### 3.7 Protected monthly verification and rebuild contract (`MET-WP4-03A-OPS-02`)

`MET-WP4-03A-OPS-02` (issue #951, through Specification Amendments 1 and 2)
delivers the minimum protected producer contract Sphinx needs to establish
that the four derived monthly projections of section 3.6 exactly match the
authoritative work-day projection at one durable frontier, and to request
one exceptional full rebuild at a caller-pinned frontier. Continuous normal
incremental maintenance inside `completeMetricRollupDeltas` (sections 3.1
and 3.6) is unchanged. Thoth remains sole owner of projection arithmetic,
locking and PostgreSQL mutation.

Operations, both requiring exactly `METRICS_INGEST_SERVICE` and both
authorized before any rollup-specific database read, lock or write:

```graphql
verifyMetricRollupMonths: MetricRollupMonthVerification!
rebuildMetricRollupMonths(input: RebuildMetricRollupMonthsInput!): MetricRollupMonthRebuildResult!
```

```graphql
input RebuildMetricRollupMonthsInput {
  expectedAppliedThroughSequence: String!
}

type MetricRollupMonthProjectionVerification {
  expectedRows: String!
  actualRows: String!
  missingRows: String!
  extraRows: String!
  mismatchedRows: String!
}

type MetricRollupMonthVerification {
  appliedThroughSequence: String!
  nextSequence: String!
  watermarkAt: Timestamp!
  maxWorkDayWatermark: String
  workDayRowCount: String!
  representedMonthKeyCount: String!
  total: MetricRollupMonthProjectionVerification!
  country: MetricRollupMonthProjectionVerification!
  institution: MetricRollupMonthProjectionVerification!
  ambiguity: MetricRollupMonthProjectionVerification!
  matches: Boolean!
}

type MetricRollupMonthRebuildResult {
  rebuilt: Boolean!
  verification: MetricRollupMonthVerification!
}
```

Authorization matrix, identical for both operations: `METRICS_INGEST_SERVICE`
is allowed; anonymous callers, `SUPERUSER`, `METRICS_READ_SERVICE`,
publisher-scoped human roles, `DISSEMINATION_WORKER` and every other machine
role are denied, and a denied request opens no transaction and issues no
rollup-specific statement. Both operations live on `MutationRoot`:
verification is database-read-only in effect but is an internal operational
service command, not a product query surface, and no field on `QueryRoot`
or on any public type reaches any of these types.

Contract properties consumers may rely on:

- **Verification is one read-only snapshot.** `verifyMetricRollupMonths`
  runs on one pooled connection in one PostgreSQL transaction opened
  `READ ONLY` at `REPEATABLE READ`, with `SET LOCAL statement_timeout =
  '30s'`, no `FOR UPDATE` and zero database writes; the operation asserts
  the transaction mode it was granted before reading anything and fails
  closed otherwise. Within that snapshot it reads
  `metric_rollup_work_day_state` (`nextSequence`, `appliedThroughSequence =
  W`, `watermarkAt`), the work-day facts (`maxWorkDayWatermark`,
  `workDayRowCount`, `representedMonthKeyCount`), the actual state of all
  four monthly tables and the independently derived expected state.
- **One table-level read lock closes the `TRUNCATE` anomaly.** Immediately
  after the statement timeout and before its first query — and therefore
  before its `REPEATABLE READ` snapshot is frozen — the verifier executes
  `LOCK TABLE` on exactly the four monthly projection tables `IN ACCESS
  SHARE MODE`, and holds that lock until the transaction ends. PostgreSQL's
  `TRUNCATE` is not MVCC-safe: a snapshot taken before a concurrent
  rebuild's truncate would see the tables it had not yet accessed as empty.
  `ACCESS SHARE` is a table-level read lock, not a row lock: it conflicts
  only with `ACCESS EXCLUSIVE`, which the exceptional rebuild's `TRUNCATE`
  takes, and not with the `ROW EXCLUSIVE` locks of normal incremental
  completion, so routine monthly maintenance is never blocked by a
  verification. If the verifier holds the lock first, a rebuild waits at its
  `TRUNCATE` until the verification commits; if a rebuild already holds the
  tables exclusively, the verifier waits before establishing its snapshot,
  bounded by its 30-second statement timeout, and then sees the committed
  rebuilt state. Verification remains database-read-only; the lock exists
  solely to prevent that anomaly and changes no specification architecture.
- **Independent expected state.** The expected monthly state is derived by
  separate set-based PostgreSQL SQL from `metric_rollup_work_day` alone. It
  shares no statement, constant or macro with the monthly writer, never
  calls the writer's recomputation, never proves correctness by rebuilding
  and reading the rebuild, and never loads the work-day projection into
  application memory. Where the writer resolves each daily base cell through
  a fixed case table, the verifier derives the resolution of every possible
  represented mask set from the generic minimality rule of section 3.6 (the
  Amendment 6 total rule; the unique least mask containing the target
  dimension under set inclusion, with incomparable minima recorded as
  ambiguity) and compares by logical identity, `NULL`-safe on
  `publication_id` exactly as the tables' `UNIQUE NULLS NOT DISTINCT`
  identities are.
- **Bounded counts only.** Per family — `total`, `country`, `institution`,
  `ambiguity` — the result carries `expectedRows`, `actualRows`,
  `missingRows` (expected identity absent from the table), `extraRows`
  (table identity absent from the expected state) and `mismatchedRows`
  (same identity on both sides with a differing value, publication,
  country or institution identity, dependency flag, ambiguity flag or
  watermark). No row identity, value or mismatch detail leaves Thoth.
  `matches` is true only when every family has zero missing, extra and
  mismatched rows and no monthly row is watermarked above `W`.
- **Lag is not corruption; damage fails closed.** `nextSequence - 1 > W` is
  ordinary pending rollup lag and is reported as such. A work-day row
  watermarked above `W` is an internal invariant violation: both operations
  fail closed rather than describe it. An expected monthly sum that would
  overflow a signed 64-bit total fails both operations closed.
- **Rebuild input is the pinned frontier only.** The caller supplies
  `expectedAppliedThroughSequence` as a decimal string holding a
  non-negative 64-bit integer, and nothing else: no work, month, platform or
  measure identity, value, dimension, coverage or ambiguity flag, watermark,
  SQL or repair instruction. An invalid string is a bounded validation error
  decided before any database access.
- **Self-verifying all-or-nothing rebuild.** `rebuildMetricRollupMonths`
  runs one transaction that, in order: sets `SET LOCAL lock_timeout = '5s'`
  and `SET LOCAL statement_timeout = '30s'`; locks the singleton
  `metric_rollup_work_day_state` row `FOR UPDATE`; requires the current
  `appliedThroughSequence` to equal the pinned frontier; requires no
  work-day row above it; requires at most 100000 represented month keys;
  independently verifies the current monthly state; and, only if that
  verification is not exact, truncates exactly the four monthly tables,
  recomputes every represented month key in deterministic chunks of at most
  50 through the completion's own reviewed recomputation, independently
  verifies the rebuilt state again in the same transaction, and commits only
  an exact result. A stale frontier, a work-day row above the frontier, more
  than 100000 keys, a lock or statement timeout, an injected or database
  failure at any stage, an inexact post-rebuild verification, an overflow,
  or more than 120 seconds of server-side elapsed time — the clock starts
  before the initial verification and is checked after it, after every
  chunk, immediately before and after the post-rebuild verification and
  immediately before commit — fails the operation and rolls the complete
  transaction back. No partially rebuilt monthly state can commit.
- **Healthy rebuild is a no-op.** When the monthly state already verifies
  exact at the pinned frontier, the result is `rebuilt: false` with that
  verification, and no `TRUNCATE`, `DELETE`, `INSERT` or monthly surrogate-id
  rewrite occurs. A repeated call is therefore idempotent in effect.
- **What a rebuild never touches.** Only the four monthly tables of section
  3.6 are written. `metric_rollup_work_day`, every rollup delta and its
  status, sequence, claim and lease, `metric_rollup_work_day_state`
  (`next_sequence`, `applied_through_sequence`, `watermark_at`), canonical
  `metric_record`/`metric_record_revision` state and the generic
  `metric_reconciliation_run`/`metric_reconciliation_issue` ledgers are left
  unchanged. The rebuild cannot advance the frontier.
- **Serialization through the state row.** The rebuild holds the same
  singleton state-row lock every claim, completion and work-day sequence
  allocation takes. Normal completion and allocation wait behind it, the
  rebuild commits at `W`, the waiting work resumes, and later completions
  maintain the affected months incrementally on top of the rebuilt state.
  No maintenance mode, coordination table, queue, pause flag, scheduler or
  process-wide mutex exists.
- **Bounds are safety envelopes, not SLOs.** `MAX_REBUILD_MONTH_KEYS =
  100000`, chunk size 50, verification `statement_timeout = 30s`, rebuild
  `lock_timeout = 5s`, rebuild `statement_timeout = 30s`, total rebuild
  elapsed ceiling 120s. They are not raised silently; a larger corpus or a
  different envelope needs a separately approved amendment.
- **Positions and counts are strings.** Every 64-bit sequence, count and
  watermark position crosses GraphQL as a decimal `String`, never as the
  32-bit `Int`.

Supported orchestration lifecycle:

```text
verify
-> exact: healthy, no rebuild

initial empty state, or separately authorized repair/recovery
-> capture W from the verification
-> rebuild(expectedAppliedThroughSequence = W)
-> the rebuild commits only if its own independent verification is exact
-> verify again later to establish current health
```

Scope boundaries a consumer must not infer:

- Sphinx is the eventual orchestrator of verification and of the
  initial/exceptional rebuild. It never supplies projection values,
  dimensions, flags, watermarks or SQL, and never writes PostgreSQL.
- A rebuild is initial/exceptional maintenance only. It is not a monthly
  job, is never scheduled by Thoth, and is never invoked automatically when
  a verification mismatches; a mismatch is an alert/HOLD for a separately
  authorized decision.
- The generic WP9 reconciliation persistence (`metric_reconciliation_run`,
  `metric_reconciliation_issue`) is a separate domain. Neither operation
  creates, reads or updates a row of it, and no `recordMetricReconciliation`
  or similar operation exists. Persisting a rollup verification in that
  ledger would need its own cross-work-package decision.
- Neither operation is a reader of monthly values: the one read surface
  remains the `MET-WP4-03B` dashboard contract of section 4.2.
- Merging this contract activates nothing: no migration, table, index,
  dependency, role, provider or runtime behaviour is introduced, and any
  real verification or rebuild execution, Sphinx consumer binding,
  deployment and production activation remain separately authorized gates.

Error surface (known limitation): as for section 3.1, rejections — an
invalid or stale expected frontier, more than 100000 represented month keys,
the elapsed ceiling, an overflowing expected total, a work-day row above the
frontier and a post-rebuild mismatch — are returned as GraphQL errors with
stable messages and the generic `INTERNAL_ERROR` extension type, and a lock
or statement timeout surfaces as the database's own cancellation message. No
message carries a frontier value, row, SQL, principal or connection detail.
A consumer that needs to classify these programmatically requires a
separately specified stable error code.

## 4. Dashboard/widget

Thoth owns entity metrics, dashboard/widget operations and registry queries.

Required response concerns: distinct measure totals, timeline, breakdowns, coverage, freshness, watermark, warnings, partial state, BigInt strings and deterministic pagination.

Semantics: OR within lists, AND between dimensions, exclusive end date, bounded ranges and filtering before pagination.

### 4.1 Delivered minimum protected read contract (`MET-WP4-02`)

`MET-WP4-02` (issue #910) delivers the MOM-1 read surface: one coverage-aware
dashboard query and the two registry lists a service needs to form its UUID
filters. `MET-WP4-03B` (section 4.2) completes the dashboard contract
additively: every property below still holds except where section 4.2
states how it is extended.

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
enum MetricWarningCode {
  PARTIAL_COVERAGE
  UNKNOWN_COVERAGE
  UNRESOLVED_IDENTIFIERS
  ROLLUP_LAG
}
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
  the read is written over a publisher set. MOM-1 served exactly one
  publisher per request: none, several or an unknown ID was
  `METRIC_QUERY_INVALID`, never a first-item choice, a truncation or a
  combination. `MET-WP4-03B` replaces that milestone restriction with the
  reviewed one-to-three publisher entitlement and aggregation contract of
  section 4.2.
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
  to every identity represented for the selected publishers and range in the
  work-day projection, terminal coverage from an eligible managed account, or
  identifier-unresolved quarantine evidence. The quarantine branch is scoped
  by immutable `metric_import.publisher_id`, half-open date overlap and any
  explicit other dimension; it does not require the historical source or
  account to remain enabled. Quarantine-derived pairs participate in every
  ordinary platform/measure/combination/timeline bound and are never truncated.
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
  whatever the coverage or identifier quality. A cell without rows returns
  `"0"` only when every day of it is effectively `COMPLETE`, no unapplied
  work-day delta touches it and no identifier-unresolved quarantine evidence
  overlaps it; otherwise `null`. `BigInt` is a canonical base-10 string:
  no `Int`, no float, signed and exact, with checked arithmetic that fails
  rather than wraps.
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
  frontier, represented scope, projection, coverage, outstanding rollup work
  and identifier quality. `asOf` is that transaction's timestamp.
  `rollupWatermark` is exactly the frontier's `watermark_at`. The read never
  advances, claims, completes, repairs, reconciles or records anything.
- **Freshness.** `ROLLUP_LAG` means at least one work-day delta above the
  applied frontier intersects the selected publishers, range, platforms and
  measures. Unrelated backlog is not lag. An item's `dataThrough` is the last
  day of the unbroken run from `startDate` in which every day is `COMPLETE`
  and no such delta falls in `[startDate, D + 1 day)`, or `null` if the first
  day fails. Backlog before `startDate` does not reduce it. Top-level
  `dataThrough` is the earliest item value, and `null` unless every item has
  one.
- **Identifier quality (`MET-WP7-PREREQ-04`).** A quarantine observation is
  unresolved exactly when no reconciliation row exists or its reconciliation
  row has `resolved_at IS NULL`. That includes never-attempted evidence,
  `PENDING_UNKNOWN_DOI` and every `BLOCKED_*` state. The four terminal
  `RESOLVED_*` states are resolved and no longer warn. Scope is the immutable
  import publisher plus the quarantine platform, measure and half-open period;
  current Work ownership, current DOI resolution and current source enablement
  are deliberately not consulted. Every overlapped served day is marked
  identifier-incomplete. This read metadata changes neither coverage nor
  reconciliation/canonical/rollup state.
- **Warnings.** At most one per code, in the order `UNKNOWN_COVERAGE`,
  `PARTIAL_COVERAGE`, `UNRESOLVED_IDENTIFIERS`, `ROLLUP_LAG`, with fixed
  text. `UNRESOLVED_IDENTIFIERS` renders exactly "Some source evidence in
  this request still has unresolved work identifiers, so values may be
  incomplete and an otherwise empty cell is not a zero." `isPartial` is true
  exactly when a warning is present. Identifier quality alone does not change
  coverage status/items, dimension coverage, `dataThrough` or
  `rollupWatermark`.
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
  `metricWidget` and entity-level Metrics fields were not part of the
  `MET-WP4-02` contract. `MET-WP4-03B` adds the country and institution
  sections and every one of those selector dimensions except `dois` and
  `includeDescendants` (section 4.2); the rest still needs its own
  specification.
- The read calls no external service. Browser clients must not hold the
  `METRICS_READ_SERVICE` credential; they reach this contract through a
  Thoth-owned server route.
- The read contract remains additive. No existing field, argument, nullability
  or authorization changed; `MET-WP7-PREREQ-04` adds only the
  `UNRESOLVED_IDENTIFIERS` value to `MetricWarningCode` and reconciles the
  documented zero/partial semantics above. `MetricCoverageStatus` is the
  existing database enum, exposed to GraphQL without any value change.
  Generated/exhaustive clients must tolerate the new warning enum before
  CloudFront production activation, and consumers must not coerce `null`
  Metrics values to zero.

### 4.2 Complete dashboard contract (`MET-WP4-03B`)

`MET-WP4-03B` (issue #946, through Specification Amendments 1 and 2)
completes
`metricDashboard` for the dashboard cutover: one server-side query resolves
current metadata selectors, serves complete calendar months from the
`MET-WP4-03A` monthly projections (section 3.6) and clipped edge days from
the work-day projection, and returns graph-ready totals, timeline, countries
and institutions with section-aware coverage. It preserves the
`MET-WP7-PREREQ-04` identifier-quality contract of section 4.1 unchanged
except for the represented-publisher scope below. No operation, role,
capability, migration, table or index is added; `metricMeasures` and
`metricPlatforms` are unchanged.

Additive GraphQL contract (every existing field, argument, nullability, enum
value and authorization of section 4.1 is unchanged):

```graphql
input MetricSelectorInput {
  publisherIds: [Uuid!]
  imprintIds: [Uuid!]
  seriesIds: [Uuid!]
  workIds: [Uuid!]
  workTypes: [WorkType!]
  languages: [LanguageCode!]
  fundingInstitutionIds: [Uuid!]
  affiliationInstitutionIds: [Uuid!]
}

input MetricDashboardInput {
  # ...section 4.1 fields...
  includeCountries: Boolean = true
  includeInstitutions: Boolean = true
}

type MetricDashboard {
  # ...section 4.1 fields...
  countries: [MetricCountryTotal!]!
  institutions: [MetricInstitutionTotal!]!
}

type MetricCountryTotal {
  platformId: Uuid! measureId: Uuid! countryCode: String! value: BigInt!
}
type MetricInstitutionTotal {
  platformId: Uuid! measureId: Uuid! institutionId: Uuid!
  institutionName: String! ror: Ror value: BigInt!
}
```

The one new error classification is `METRIC_QUERY_UNSUPPORTED_SOURCE_GRAIN`.
`UNRESOLVED_IDENTIFIERS` is the existing `MET-WP7-PREREQ-04` warning, not a
`MET-WP4-03B` addition.

Contract properties consumers may rely on:

- **Publishers.** `publisherIds` holds one to three unique IDs. None, more
  than three or a duplicate is `METRIC_QUERY_INVALID`, and so is an unknown
  publisher. Every selected publisher must independently hold
  `METRICS_DASHBOARD`; one that does not makes the whole request `NO_ACCESS`,
  even when the selector would exclude its works. There is no entitled-subset
  response, no `SUPERUSER` shortcut and no new role: the resolver still
  requires exactly `METRICS_READ_SERVICE` first.
- **Selector.** Values within one list are alternatives (OR); different
  lists must all match (AND); an omitted or empty optional list does not
  restrict. Works are resolved from current metadata, before any Metrics
  value is read: current `work -> imprint -> publisher` ownership; imprint;
  explicit work; work type; a series by direct issue membership or, for a
  `book-chapter`, through a current `is-child-of` parent issued in it; any
  language record of the work whatever its relation; any funding by a
  selected institution; any contribution affiliated with a selected
  institution. No metadata copy is stored for Metrics.
- **Selector bounds and validity.** `workIds` at most 500; `imprintIds`,
  `seriesIds`, `workTypes`, `languages`, `fundingInstitutionIds` and
  `affiliationInstitutionIds` at most 50 each (work types have only six
  values, so more than six is always a duplicate). One value beyond a bound
  is `METRIC_QUERY_LIMIT_EXCEEDED`; a duplicate value is
  `METRIC_QUERY_INVALID`; an explicit ID that exists nowhere is
  `METRIC_QUERY_INVALID`. An existing ID that belongs to another publisher,
  or matches none of the other selected works, is valid and may resolve to
  no works. Unknown `WorkType` or `LanguageCode` values are refused by
  GraphQL input coercion before the resolver. A selection resolving to more
  than 2,000 works is `METRIC_QUERY_LIMIT_EXCEEDED`. No bound is met by
  truncation.
- **Represented publishers.** After resolution only the publishers owning at
  least one resolved work take part in source accounts, coverage, lag,
  values and identifier quality. An omitted `platforms` or `measures`
  resolves from the state represented for the resolved works: the
  projections, terminal coverage from an eligible account of a represented
  publisher, or unresolved identifier quarantine of a represented publisher.
- **No works.** A valid selection resolving to no works succeeds. With an
  omitted platform or measure dimension nothing is represented, so the
  response is the section 4.1 empty shape: no totals, timeline, countries,
  institutions or items, `UNKNOWN`, `dataThrough` `null`, one
  `UNKNOWN_COVERAGE` warning, `isPartial` true. With both dimensions
  explicit the combinations are listed with `null` values and `UNKNOWN`
  coverage: no zero is invented from an empty metadata scope. With no
  represented publisher, unresolved quarantine neither discovers a pair nor
  emits `UNRESOLVED_IDENTIFIERS`.
- **Identifier quality (Amendment 2).** The `MET-WP7-PREREQ-04` rules of
  section 4.1 hold unchanged for the represented publishers: unresolved
  quarantine is scoped by the immutable import publisher, the quarantine
  platform and measure and the half-open period, ignores current source
  enablement, is never matched to a resolved work, DOI or current imprint,
  series, language, funding or affiliation metadata, and is therefore more
  conservative than the work-scoped values and lag. An explicitly selected
  publisher without a resolved work contributes none of it. Every overlapped
  served day is identifier-incomplete; one day mask governs complete-month
  buckets, clipped buckets and totals alike, so an otherwise empty value is
  `null` while a projected value stays exact. It sets the fixed
  `UNRESOLVED_IDENTIFIERS` warning (in the section 4.1 order) and
  `isPartial`, whatever the section flags, and it never changes coverage
  status, items, dimension flags, `dataThrough`, `rollupWatermark`,
  dimensional rows or ambiguity.
- **Complete months and edge days.** The calendar months wholly inside
  `[startDate, endDate)` are read from `metric_rollup_work_month`,
  `metric_rollup_work_country_month`, `metric_rollup_work_institution_month`
  and `metric_rollup_work_month_ambiguity`, summing their resolved rows
  without re-resolving or re-ranking them; the clipped leading and trailing
  days are read from `metric_rollup_work_day` and resolved per base cell as
  section 4.1 and section 3.6 resolve them. Totals, countries and
  institutions add both parts. `AUTO` remains `DAY`. A `DAY` timeline reads
  every day from the work-day projection; a `MONTH` timeline takes a complete
  month's bucket from the monthly projection and a clipped bucket from its
  days. The values equal the section 4.1 day-by-day values for the same
  resolved request, which the tests prove against an independent oracle.
- **Countries and institutions.** Only known values are listed: no synthetic
  unknown country or institution, and no zero for an absent one. Country
  values use the target-dimension rule of section 3.6 (the unique least
  country-bearing representation); `countryCode` is the Metrics uppercase
  ISO 3166-1 alpha-2 code, not Thoth's alpha-3 `CountryCode`. Institution
  values use the mirror rule, with the name and nullable ROR of the current
  Thoth institution read set-wise in the same snapshot. Countries are ordered
  by `(platformId, measureId, countryCode)` and institutions by
  `(platformId, measureId, institutionId)`, never by display text. At most
  2,000 institution rows are returned; more is
  `METRIC_QUERY_LIMIT_EXCEEDED`, never a truncated list. Country rows are
  bounded by the combinations and the alpha-2 code space.
- **Section inclusion.** `includeCountries` and `includeInstitutions`
  default to true. A section that is false returns no rows, cannot fail the
  request through its own ambiguity and cannot downgrade the shared coverage
  through its own dimension; it never removes a dimensional dependency the
  totals and timeline themselves have.
- **Ambiguity.** A total-ambiguous base cell or month fails the request with
  `MOM1_DIMENSION_SCOPE_AMBIGUOUS` whatever the section flags. A country- or
  institution-ambiguous month or edge day fails it only when that section is
  returned. Complete months use the monthly ambiguity state; edge days use
  the equivalent day-level resolution.
- **Section-specific coverage (Amendment 1).** Per platform, measure and
  day, coverage is first combined across the represented publishers: a
  publisher without an assertion makes the day `UNKNOWN`, otherwise the worst
  status wins (`UNKNOWN`, then `PARTIAL`, then `COMPLETE`, with
  `COMPLETED_WITH_ERRORS` downgrading `COMPLETE` to `PARTIAL` first), and the
  country and institution flags combine by AND. The totals and timeline then
  use only the dimensions their own served representation depends on,
  range-wide per combination exactly as section 4.1, and that alone decides
  their zero or `null`. A returned country section additionally needs
  country coverage on every day, and institution coverage when any country
  value came from rows also broken down by institution; a returned
  institution section mirrors it. The shared `coverage.items[].status`,
  `coverage.status`, item and top-level `dataThrough`, warnings and
  `isPartial` take, per day, the worst of the totals and every returned
  section, so requesting an incompletely covered section can make them more
  conservative without changing any total or timeline value.
  `countryCoverage` and `institutionCoverage` remain the combined dimension
  facts.
- **Native grains.** The projections serve `DAY` canonical records only. If
  a canonical record of `MONTH` or `REPORTING_PERIOD` grain for a resolved
  work, served platform and served measure overlaps `[startDate, endDate)`
  and its `current_revision_id` names a `CURRENT` revision, the whole request
  fails with `METRIC_QUERY_UNSUPPORTED_SOURCE_GRAIN` and a fixed message;
  nothing is silently omitted. A record whose pointer names a `RETRACTED`
  revision, with no `CURRENT` revision, is withdrawn and does not fail it.
  Contradictory committed state — no pointer, a pointer to a `SUPERSEDED`
  revision, or a `RETRACTED` pointer beside a `CURRENT` revision — fails
  closed as `INTERNAL_ERROR`.
- **Lag scope.** `ROLLUP_LAG` and the lag part of `dataThrough` count only
  unapplied work-day deltas above the frontier for the resolved works,
  range, served platforms and served measures. Identifier quality does not
  widen it.
- **Execution.** Everything still runs on one connection in one
  `READ ONLY, REPEATABLE READ` transaction, starting with
  `SET LOCAL plan_cache_mode = force_custom_plan` so each statement is
  planned for its actual arrays and bounds. The longest request path is
  exactly sixteen database statements counted conservatively — the pool's
  own checkout `SELECT 1`, `BEGIN`, the planner setting, twelve queries
  including the identifier-quality query, and `COMMIT` — whatever the number
  of publishers, works, days, months, countries, institutions or quarantine
  rows: the frontier is read with the publishers, explicit selector IDs are
  checked while the works are resolved, native grains and lag share one
  statement, each monthly and edge-day section statement runs at most once,
  and no work or institution is looked up individually. The per-publisher
  coverage day series is a `MATERIALIZED` CTE.
- **Activation gate.** The monthly projections are served only once, under
  separate authorization, the `MET-WP4-03A` migration has run and a full
  historical rebuild has been reconciled at a recorded work-day frontier
  (section 3.6). Merging this contract activates nothing.

Scope boundaries a consumer must not infer:

- `dois`, `includeDescendants`, works sections and pagination,
  `metricWidget` and entity-level Metrics fields remain deferred.
- No native `MONTH` or `REPORTING_PERIOD` serving is implemented; such
  records only fail an intersecting request.
- The read writes nothing: no projection, monthly row, delta, frontier or
  coverage is repaired or recorded, and no external service is called.

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
