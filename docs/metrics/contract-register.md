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

#### Account windows, successions and native-month checks (`MET-WP4-MONTH-01`)

`MET-WP4-MONTH-01` (thoth#1042) adds, additively, an optional date window and
an optional succession to every source account, and the repair of the
native-month projection of section 3.6:

```graphql
# MetricSourceAccount (output) and NewMetricSourceAccount (input) gain
windowStart: Date
windowEnd: Date
succession: String

# SUPERUSER only, authorized before any operation-specific read, lock or write
updateMetricSourceAccountWindow(
  code: String!
  windowStart: Date
  windowEnd: Date
  succession: String
): MetricSourceAccount!
recomputeMetricNativeMonths(publisherId: Uuid!): MetricNativeMonthRecomputeResult!

type MetricNativeMonthRecomputeResult {
  works: Int!
  rows: Int!
}
```

- **Window.** Half-open, `[windowStart, windowEnd)`, either end optional,
  `null` meaning unbounded. A window that does not end after it starts is
  refused: "A metric source account window must end after it starts."
- **Succession.** A label of 1 to 64 lowercase letters, digits and single
  hyphens, or `null`; anything else is refused: "A metric source account
  succession must be 1 to 64 lowercase letters, digits and single hyphens."
  Each succession of one publisher and platform is one slot, and each account
  without a succession is a slot of its own. A slot is keyed by publisher,
  platform and label, so a label also used by another publisher or platform
  is harmless.
- `createMetricSourceAccount` validates the three optional fields before any
  write, so the database checks `metric_source_account_window_check` and
  `metric_source_account_succession_check`, which map to the same two
  messages, never fire from the API.
- `updateMetricSourceAccountWindow` is a complete replacement of
  `windowStart`, `windowEnd` and `succession`, `null` clearing one. A request
  that matches the stored values changes nothing and records no audit entry;
  a real change records one `UPDATE` before/after entry in
  `metric_source_registry_history`, atomically, and the audited account state
  carries the three keys. An account whose stored configuration is
  unsupported is neither updated nor audited, as by `updateMetricSourceAccount`.
  `PatchMetricSourceAccount` is unchanged, so `updateMetricSourceAccount`
  never touches the three fields.
- A window limits only which days the account's coverage counts for. It does not restrict claims or collection, and it never hides a value. A window alone never lowers what a day requires: an account replaces another only inside a succession, which asserts that its members collect one source of traffic. Operators set a succession only for accounts that collect the same site, checked as for enabling another account.
- `recomputeMetricNativeMonths` deletes and re-derives the native-month rows
  of every work now under the publisher's imprints (section 3.6). An unknown
  publisher fails as not found before any write. `works` is the number of
  works recomputed and `rows` the number of rows written.

**Months and days.** The reads serve the days and leave the month out where a
`MONTH` record and `DAY` records share a base cell (section 4.1), so:

- **Operator rule.** For each work, platform and measure, months imported as
  `MONTH` and days collected as `DAY` never share a calendar month, and a
  monthly account's window starts and ends on the first day of a month. So a
  month in which a live `DAY` account starts stays out of the monthly history.
- **Detection.** One stray day displaces a whole month's value, and nothing
  refuses or alarms, so detection is a duty: V-CONFLICT and V-PLAN below, when
  stated there. A history import checks its planned month keys against
  V-CONFLICT's `day_live` as V-PLAN checks days. A hit is a `HOLD` for that
  work and goes to the CTO.
- **Repair.** The CTO decides which side is wrong. A wrong native side is
  zeroed, by the history importer under its own grant, only if the days are
  the month's whole value: its import's coverage still asserts the month, so
  the days alone can read `COMPLETE`. Otherwise it is revised to its true
  value or left, and the conflict stands. A wrong day side cannot be zeroed
  today (a `DAY` revision must come from the record's winning account, and no
  collection run revises a day to 0), so the conflict stands and the month is
  `PARTIAL` in every read until a day-zeroing path exists. The CTO records
  each standing cell (work, platform, measure, month) per environment, and
  V-CONFLICT's gates except exactly those cells. The native side is never
  zeroed just to clear a conflict: the stray days alone would read as a
  `COMPLETE` month, an undercount no retraction can undo. Any other zeroing
  needs the CTO's written acceptance of that outcome, per work and month. A
  zero month record stays: it adds and conflicts with nothing, but still marks
  the month cut wherever a cell cuts it.

**Checks.** Four read-only checks, each run as written:

**V-MONTH (PRE-1)**, before each deploy of a server at or after
`MET-WP4-MONTH-01`, at each monthly-serving activation step and after every
`MONTH` import chunk. `HOLD` unless `invalid_periods` and
`inconsistent_pointers` are 0 for both grains and `non_additive` is 0 for
`MONTH`; a grain with no row passes. `active` reports how many records a
server with monthly serving `ON` serves (`MONTH`) or refuses on
(`REPORTING_PERIOD`), and an `earliest` period before 2000-01-01 is a
`HOLD`: it would refuse every all-time read of its scope, as any evidence
before the floor does.

```sql
SELECT r.reporting_grain::text, count(*) AS records,
  count(*) FILTER (WHERE v.status = 'CURRENT') AS active,
  count(*) FILTER (WHERE r.reporting_grain IN ('MONTH', 'QUARTER')
  AND NOT (EXTRACT(DAY FROM r.period_start) = 1
  AND (r.reporting_grain = 'MONTH' OR EXTRACT(MONTH FROM r.period_start) IN (1, 4, 7, 10))
  AND r.period_end = (r.period_start + CASE r.reporting_grain WHEN 'QUARTER'
    THEN interval '3 months' ELSE interval '1 month' END)::date)) AS invalid_periods,
  count(*) FILTER (WHERE (v.status = 'CURRENT') IS NOT TRUE
  AND NOT (COALESCE(v.status = 'RETRACTED', FALSE)
  AND NOT EXISTS (SELECT 1 FROM metric_record_revision o
  WHERE o.record_id = r.record_id AND o.status = 'CURRENT'))) AS inconsistent_pointers,
  count(*) FILTER (WHERE v.status = 'CURRENT'
  AND NOT (m.additive_across_time AND m.additive_across_works)) AS non_additive,
  min(r.period_start) AS earliest  -- HOLD if before 2000-01-01 (MET-WP4-MONTH-01 section 9's preconditions)
FROM metric_record r JOIN metric_measure m ON m.measure_id = r.measure_id
LEFT JOIN metric_record_revision v
  ON v.record_id = r.record_id AND v.record_revision_id = r.current_revision_id
WHERE r.reporting_grain <> 'DAY' GROUP BY 1;
```

**V-NATIVE**, after each deploy, before each switch to `ON`, after every
`MONTH` import chunk, at the interval the CTO sets at production activation,
and after any time a server older than `MET-WP4-MONTH-01` ran. It derives the
expected rows of the four native-month tables (section 3.6) from
`metric_record` and its current revisions, in SQL that shares no text with the
writer's, and returns, per table, the rows expected but absent (`missing`),
present but not expected (`unexpected`) and differing in any column
(`differing`). The twelve counts come from one statement, so from one
snapshot, and must all be 0. It scans every non-`DAY` record, so it runs in a
`READ ONLY` transaction under `SET LOCAL statement_timeout`. A non-zero count
is a `HOLD`: `ON` is not switched on, or the CTO decides whether it stays on,
until `recomputeMetricNativeMonths` for the publishers of those keys brings it
to 0. A count that returns is a defect, not a repair.

```sql
WITH record AS (
  SELECT r.work_id, r.platform_id, r.measure_id, r.period_start, r.period_end,
    r.reporting_grain::text AS grain, r.country_code, r.institution_id, p.value,
    CASE r.reporting_grain WHEN 'QUARTER' THEN 3 ELSE 1 END AS span,
    CASE WHEN r.publication_id IS NULL THEN 0 ELSE 4 END
      + CASE WHEN r.country_code IS NULL THEN 0 ELSE 2 END
      + CASE WHEN r.institution_id IS NULL THEN 0 ELSE 1 END AS mask,
    CASE
      WHEN p.status = 'RETRACTED' AND NOT EXISTS (SELECT 1 FROM metric_record_revision c
        WHERE c.record_id = r.record_id AND c.status = 'CURRENT') THEN 'WITHDRAWN'
      WHEN p.status = 'CURRENT' AND r.reporting_grain = 'REPORTING_PERIOD' THEN 'ACTIVE'
      WHEN p.status = 'CURRENT' AND r.reporting_grain IN ('MONTH', 'QUARTER')
        AND r.period_start = date_trunc('month', r.period_start::timestamp)::date
        AND (r.reporting_grain = 'MONTH' OR EXTRACT(MONTH FROM r.period_start) IN (1, 4, 7, 10))
        AND r.period_end = (r.period_start::timestamp
          + CASE r.reporting_grain WHEN 'QUARTER' THEN interval '3 months'
            ELSE interval '1 month' END)::date
        THEN 'SERVED'
      ELSE 'INCONSISTENT'
    END AS class
  FROM metric_record r
  LEFT JOIN metric_record_revision p
    ON p.record_id = r.record_id AND p.record_revision_id = r.current_revision_id
  WHERE r.reporting_grain <> 'DAY'),
clash AS (
  SELECT DISTINCT a.work_id, a.platform_id, a.measure_id, a.period_start
  FROM record a JOIN record b
    ON (b.work_id, b.platform_id, b.measure_id) = (a.work_id, a.platform_id, a.measure_id)
    AND ((b.period_start = a.period_start AND b.span <> a.span)
      OR (b.period_start <> a.period_start AND b.period_start < a.period_end
        AND b.period_end > a.period_start))
  WHERE a.class = 'SERVED' AND b.class = 'SERVED'),
served AS (
  SELECT * FROM record s WHERE s.class = 'SERVED' AND NOT EXISTS (
    SELECT 1 FROM clash c
    WHERE (c.work_id, c.platform_id, c.measure_id, c.period_start)
      = (s.work_id, s.platform_id, s.measure_id, s.period_start))),
cell AS (
  SELECT work_id, platform_id, measure_id, period_start AS month_start,
    min(span) AS span_months,
    array_agg(DISTINCT mask) AS masks, bool_or(value <> 0) AS live
  FROM served GROUP BY work_id, platform_id, measure_id, period_start),
choice AS (
  SELECT cell.*,
    CASE WHEN 0 = ANY (masks) THEN 0 WHEN cardinality(masks) = 1 THEN masks[1] END AS total_mask,
    ARRAY(SELECT m FROM unnest(masks) AS m WHERE m & 2 = 2 AND NOT EXISTS (
      SELECT 1 FROM unnest(masks) AS n WHERE n & 2 = 2 AND n <> m AND n & m = n)) AS country_minima,
    ARRAY(SELECT m FROM unnest(masks) AS m WHERE m & 1 = 1 AND NOT EXISTS (
      SELECT 1 FROM unnest(masks) AS n WHERE n & 1 = 1 AND n <> m AND n & m = n)) AS institution_minima
  FROM cell),
expected_month AS (
  SELECT c.work_id, c.platform_id, c.measure_id, c.month_start,
    (SELECT sum(s.value) FROM served s
      WHERE (s.work_id, s.platform_id, s.measure_id, s.period_start)
        = (c.work_id, c.platform_id, c.measure_id, c.month_start)
        AND s.mask = c.total_mask)::bigint AS value,
    c.span_months, c.live,
    COALESCE(c.total_mask & 2 = 2, FALSE) AS requires_country_coverage,
    COALESCE(c.total_mask & 1 = 1, FALSE) AS requires_institution_coverage,
    cardinality(c.country_minima) > 1 AS country_ambiguous,
    cardinality(c.institution_minima) > 1 AS institution_ambiguous
  FROM choice c),
expected_country AS (
  SELECT s.work_id, s.platform_id, s.measure_id, s.period_start AS month_start, s.country_code,
    sum(s.value)::bigint AS value, bool_or(s.mask & 1 = 1) AS requires_institution_coverage,
    min(s.span) AS span_months
  FROM served s JOIN choice c
    ON (c.work_id, c.platform_id, c.measure_id, c.month_start)
      = (s.work_id, s.platform_id, s.measure_id, s.period_start)
  WHERE cardinality(c.country_minima) = 1 AND s.mask = c.country_minima[1]
  GROUP BY s.work_id, s.platform_id, s.measure_id, s.period_start, s.country_code),
expected_institution AS (
  SELECT s.work_id, s.platform_id, s.measure_id, s.period_start AS month_start, s.institution_id,
    sum(s.value)::bigint AS value, bool_or(s.mask & 2 = 2) AS requires_country_coverage,
    min(s.span) AS span_months
  FROM served s JOIN choice c
    ON (c.work_id, c.platform_id, c.measure_id, c.month_start)
      = (s.work_id, s.platform_id, s.measure_id, s.period_start)
  WHERE cardinality(c.institution_minima) = 1 AND s.mask = c.institution_minima[1]
  GROUP BY s.work_id, s.platform_id, s.measure_id, s.period_start, s.institution_id),
expected_refusal AS (
  SELECT work_id, platform_id, measure_id, period_start, period_end,
    bool_or(class <> 'ACTIVE') AS inconsistent
  FROM record r WHERE class IN ('ACTIVE', 'INCONSISTENT')
    OR (class = 'SERVED' AND EXISTS (
      SELECT 1 FROM clash c
      WHERE (c.work_id, c.platform_id, c.measure_id, c.period_start)
        = (r.work_id, r.platform_id, r.measure_id, r.period_start)))
  GROUP BY work_id, platform_id, measure_id, period_start, period_end),
diff AS (
  SELECT 'metric_native_month' AS relation,
    CASE WHEN a.work_id IS NULL THEN 'missing' WHEN e.work_id IS NULL THEN 'unexpected'
      ELSE 'differing' END AS kind,
    COALESCE(e.work_id, a.work_id) AS work_id, COALESCE(e.platform_id, a.platform_id) AS platform_id,
    COALESCE(e.measure_id, a.measure_id) AS measure_id,
    COALESCE(e.month_start, a.month_start) AS period_start, NULL::text AS detail
  FROM expected_month e FULL JOIN metric_native_month a
    ON (a.work_id, a.platform_id, a.measure_id, a.month_start)
      = (e.work_id, e.platform_id, e.measure_id, e.month_start)
  WHERE a.work_id IS NULL OR e.work_id IS NULL
    OR (e.value, e.span_months, e.live, e.requires_country_coverage,
        e.requires_institution_coverage, e.country_ambiguous, e.institution_ambiguous)
      IS DISTINCT FROM (a.value, a.span_months, a.live, a.requires_country_coverage,
        a.requires_institution_coverage, a.country_ambiguous, a.institution_ambiguous)
  UNION ALL
  SELECT 'metric_native_country_month',
    CASE WHEN a.work_id IS NULL THEN 'missing' WHEN e.work_id IS NULL THEN 'unexpected'
      ELSE 'differing' END,
    COALESCE(e.work_id, a.work_id), COALESCE(e.platform_id, a.platform_id),
    COALESCE(e.measure_id, a.measure_id), COALESCE(e.month_start, a.month_start),
    COALESCE(e.country_code::text, a.country_code::text)
  FROM expected_country e FULL JOIN metric_native_country_month a
    ON (a.work_id, a.platform_id, a.measure_id, a.month_start, a.country_code::text)
      = (e.work_id, e.platform_id, e.measure_id, e.month_start, e.country_code::text)
  WHERE a.work_id IS NULL OR e.work_id IS NULL
    OR (e.value, e.span_months, e.requires_institution_coverage)
      IS DISTINCT FROM (a.value, a.span_months, a.requires_institution_coverage)
  UNION ALL
  SELECT 'metric_native_institution_month',
    CASE WHEN a.work_id IS NULL THEN 'missing' WHEN e.work_id IS NULL THEN 'unexpected'
      ELSE 'differing' END,
    COALESCE(e.work_id, a.work_id), COALESCE(e.platform_id, a.platform_id),
    COALESCE(e.measure_id, a.measure_id), COALESCE(e.month_start, a.month_start),
    COALESCE(e.institution_id, a.institution_id)::text
  FROM expected_institution e FULL JOIN metric_native_institution_month a
    ON (a.work_id, a.platform_id, a.measure_id, a.month_start, a.institution_id)
      = (e.work_id, e.platform_id, e.measure_id, e.month_start, e.institution_id)
  WHERE a.work_id IS NULL OR e.work_id IS NULL
    OR (e.value, e.span_months, e.requires_country_coverage)
      IS DISTINCT FROM (a.value, a.span_months, a.requires_country_coverage)
  UNION ALL
  SELECT 'metric_native_refusal',
    CASE WHEN a.work_id IS NULL THEN 'missing' WHEN e.work_id IS NULL THEN 'unexpected'
      ELSE 'differing' END,
    COALESCE(e.work_id, a.work_id), COALESCE(e.platform_id, a.platform_id),
    COALESCE(e.measure_id, a.measure_id), COALESCE(e.period_start, a.period_start),
    COALESCE(e.period_end, a.period_end)::text
  FROM expected_refusal e FULL JOIN metric_native_refusal a
    ON (a.work_id, a.platform_id, a.measure_id, a.period_start, a.period_end)
      = (e.work_id, e.platform_id, e.measure_id, e.period_start, e.period_end)
  WHERE a.work_id IS NULL OR e.work_id IS NULL OR e.inconsistent IS DISTINCT FROM a.inconsistent)
SELECT
  count(*) FILTER (WHERE relation = 'metric_native_month' AND kind = 'missing') AS month_missing,
  count(*) FILTER (WHERE relation = 'metric_native_month' AND kind = 'unexpected') AS month_unexpected,
  count(*) FILTER (WHERE relation = 'metric_native_month' AND kind = 'differing') AS month_differing,
  count(*) FILTER (WHERE relation = 'metric_native_country_month' AND kind = 'missing') AS country_missing,
  count(*) FILTER (WHERE relation = 'metric_native_country_month' AND kind = 'unexpected') AS country_unexpected,
  count(*) FILTER (WHERE relation = 'metric_native_country_month' AND kind = 'differing') AS country_differing,
  count(*) FILTER (WHERE relation = 'metric_native_institution_month' AND kind = 'missing') AS institution_missing,
  count(*) FILTER (WHERE relation = 'metric_native_institution_month' AND kind = 'unexpected') AS institution_unexpected,
  count(*) FILTER (WHERE relation = 'metric_native_institution_month' AND kind = 'differing') AS institution_differing,
  count(*) FILTER (WHERE relation = 'metric_native_refusal' AND kind = 'missing') AS refusal_missing,
  count(*) FILTER (WHERE relation = 'metric_native_refusal' AND kind = 'unexpected') AS refusal_unexpected,
  count(*) FILTER (WHERE relation = 'metric_native_refusal' AND kind = 'differing') AS refusal_differing
FROM diff;
```

V-NATIVE's key form lists the rows behind the counts, identifiers only: the
same statement with its final `SELECT` replaced by

```sql
SELECT relation, kind, work_id, platform_id, measure_id, period_start, detail
FROM diff
ORDER BY relation, kind, work_id, platform_id, measure_id, period_start, detail;
```

**V-CONFLICT**, once rollups report `exhausted`: at the staging check of
monthly history, before production `ON`, after every `MONTH` import chunk,
after every `DAY` backfill or onboarding on a platform with monthly history,
and, from production `ON`, after every scheduled production rollups run. The
number of base cells where both sides are live (section 4.1); it must be 0, or
its key list must hold only the standing cells the CTO has recorded. It reads
the canonical records, not the projection, and scans three monthly tables, so
it runs under a `statement_timeout`. To locate a hit, the same query selects
the key columns (identifiers only).

```sql
WITH native_live AS (
  SELECT DISTINCT r.work_id, r.platform_id, r.measure_id,
    (r.period_start + make_interval(months => g))::date AS month_start
  FROM metric_record r JOIN metric_record_revision v
  ON v.record_id = r.record_id AND v.record_revision_id = r.current_revision_id
  CROSS JOIN LATERAL generate_series(0, CASE r.reporting_grain WHEN 'QUARTER' THEN 2 ELSE 0 END) g
  WHERE r.reporting_grain IN ('MONTH', 'QUARTER')
  AND v.status = 'CURRENT' AND v.value <> 0),
day_live AS (
  SELECT work_id, platform_id, measure_id, month_start FROM metric_rollup_work_month WHERE value <> 0
  UNION SELECT work_id, platform_id, measure_id, month_start FROM metric_rollup_work_country_month WHERE value <> 0
  UNION SELECT work_id, platform_id, measure_id, month_start FROM metric_rollup_work_institution_month WHERE value <> 0
  UNION SELECT work_id, platform_id, measure_id, month_start FROM metric_rollup_work_month_ambiguity)
SELECT count(*) FROM native_live JOIN day_live USING (work_id, platform_id, measure_id, month_start);
```

**V-PLAN**, with no `MONTH` import chunk in progress, before a `DAY` backfill
or onboarding of a publisher (`$1`) on a platform (`$2`) over the days `$3` to
`$4`, inclusive: V-CONFLICT's `native_live` restricted to that publisher
(through `work` and `imprint`), that platform and those days. Its count must
be 0.

```sql
WITH native_live AS (
  SELECT DISTINCT r.work_id, r.platform_id, r.measure_id, r.period_start AS month_start
  FROM metric_record r JOIN metric_record_revision v
  ON v.record_id = r.record_id AND v.record_revision_id = r.current_revision_id
  JOIN work w ON w.work_id = r.work_id JOIN imprint i ON i.imprint_id = w.imprint_id
  WHERE r.reporting_grain IN ('MONTH', 'QUARTER')
  AND v.status = 'CURRENT' AND v.value <> 0
  AND i.publisher_id = $1 AND r.platform_id = $2
  AND r.period_start <= $4 AND r.period_end > $3)
SELECT count(*) FROM native_live;
```

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
  `MONTH` deltas stay outside the stream under `MET-WP4-MONTH-01`; nothing
  applies or reads them.
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
unchanged by `MET-WP2-02`; `MetricCoverageStatus` is reused from `MET-WP4-02`.
`MET-WP11-B-CORR-H3` (issue #989) later adds exactly one value to
`MetricIngestionErrorCode`, `REPORTING_GRAIN_MISMATCH`, and nothing else to the
SDL (see the canonical reporting-grain bullet below); every other value, its
name and its meaning are unchanged.

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
  that lock, calls the unchanged coordinator on a second pooled connection in
  its own transaction, so no reclaim can replace the token during the call.
  The coordinator's limits, hashing, classifications, replay and idempotency
  apply unchanged; a replay of a committed batch returns its persisted outcome
  with `replayed: true`.
- **Canonical reporting grain immutability (`MET-WP11-B-CORR-H3`, issue
  #989).** The v1 canonical identity hash excludes the reporting grain, so
  observations of the same cell and half-open period share one identity
  whatever grain label they carry (a `DAY` observation and a one-day
  `REPORTING_PERIOD` observation, a `MONTH` observation and a one-month
  `REPORTING_PERIOD` observation). The grain persisted on `metric_record` by
  the first-arriving winner is immutable for that identity. Under the
  existing `metric_record` row lock, and before any content-duplicate,
  source-conflict or revision decision, the shared canonical application
  authority compares the observation's grain with the persisted grain; a
  mismatch is a per-observation `REJECTED` row with the additive reason
  `REPORTING_GRAIN_MISMATCH`, `metric_import_error.field_name`
  `reporting_grain`, the static message "The reporting grain does not match
  the existing canonical record" and `raw_value` null, written through the
  existing rejected-provenance, import-error and `invalid_count` paths with
  the row's identity and content hashes recorded. Whether the content is the
  same or different, whether the account is the winning account or another
  account, whether the rows arrive in separate batches, in one batch or in
  concurrent first arrivals, the grain check precedes classification: the
  mismatch never rewrites the canonical grain, revises the canonical content,
  emits a rollup delta, chooses a new winner or is reported as `DUPLICATE` or
  `CONFLICT`. A committed batch replays its persisted outcome unchanged, so
  rows classified before this correction keep their committed classification;
  no historical record is normalized and no hash changes. The reason is one
  additive `MetricIngestionErrorCode` value; a consumer that decodes the enum
  closed must accept it before any collection against this producer (the
  pinned Sphinx subsets do not: see section 8 and thoth-pub/thoth-sphinx#38).
- **Batch capacity admission and bounded guard waits (`MET-WP11-B-CORR-H2`,
  issue #988).** The two-connection, two-transaction guard above is unchanged
  in what it locks, validates and commits; what changes is when the second
  connection is taken and how long the guard may wait. After the request-shape
  checks, the batch checks out its guard connection with the pool's ordinary
  blocking checkout and then, before the guard transaction opens and before
  any checkpoint lock is taken, reserves the coordinator's connection with
  exactly one non-blocking `try_get()`. That reservation takes an already-idle
  connection, not nominal capacity: it can refuse below the pool's maximum
  size, for instance while reaped idle connections are being replaced. A
  refusal releases the guard connection and fails the request as
  `INTERNAL_DATABASE_ERROR` with no durable batch, checkpoint or canonical
  effect; there is no blocking fallback checkout, third checkout, pool growth
  or scheduler, and the identical request may be retried. With both
  connections held, the guard transaction executes `SET LOCAL lock_timeout =
  '1s'` before it locks the checkpoint row, so a checkpoint held by another
  transaction — a concurrent batch of the same account, a reclaim or a
  checkpoint update — fails the guard closed after one second through the
  existing Diesel error conversion to `INTERNAL_DATABASE_ERROR`, with nothing
  committed, instead of queueing indefinitely; a caller never waits for pool
  capacity while it holds the guard. Every canonical attempt, including each
  of the coordinator's at most three authority-drift retries, runs on the
  reserved connection inside a transaction that first executes `SET LOCAL
  statement_timeout = '30s'`, a limit on each statement individually; a
  statement or lock wait that reaches it aborts and rolls back that attempt
  and fails the request as `INTERNAL_DATABASE_ERROR`, never as a partial
  batch. Both `SET LOCAL` settings end with their transaction, so no
  connection returns to the pool carrying them. Neither timeout is a
  whole-request deadline: the pool's checkout health check and connection
  establishment can still block, and no total latency, starvation or fairness
  guarantee is made. The two-transaction seam is preserved: a canonical commit
  may survive a later guard failure, and a retry of the same batch replays the
  committed outcome with `replayed: true` and no duplicated revision, delta,
  provenance, coverage or counter.
- **Completion.** `completeMetricImport` requires the import's committed batch
  keys to equal its expected set exactly (`IMPORT_INCOMPLETE` while any is
  missing) and sets `COMPLETED` when the coordinator's persisted invalid and
  conflict counters are zero, otherwise `COMPLETED_WITH_ERRORS`, with
  `completed_at` at the transaction timestamp. A terminal import is returned
  unchanged. `FAILED` is never manufactured.
- **Checkpoint progress.** `updateMetricSourceCheckpoint` accepts only a
  terminal one-day managed import. It moves `last_completed_at` forward to the
  import's `completed_at` and releases the lease. It moves
  `last_successful_period_end` forward to the import's period end (the second
  day of the month for a MONTH unit) only when the import is `COMPLETED`
  **and** has at least one coverage row **and** every coverage row is
  accepted. A coverage row is accepted by one of two rules. Rule A, judged from
  the row alone: it is `COMPLETE` and covers exactly the import's one-day
  period. Rule B, evaluated only for a source whose driver key is not
  `cloudfront`: the registry lists `MONTH` for the row's platform-measure pair,
  the row's period is the calendar month that starts on the import's day (from
  the 1st to the next 1st), and the row is `COMPLETE`, or `PARTIAL` for exactly
  one of the two months of the `PARTIAL` allowlist. The allowlist is a code
  constant: platform `oapen`, measure `downloads`, 2026-01 and 2026-08, whose
  figures are estimated; codes compare exactly, and extending the list is a
  code change under its own release. A registry pair row that is missing is not
  `MONTH`-capable, and no coverage row drops out of the judgement. Zero
  coverage rows, any `UNKNOWN` row, any row neither rule accepts and
  `COMPLETED_WITH_ERRORS` never advance it, and a CloudFront source is judged
  by rule A alone whatever the registry holds. Coverage evidence that
  contradicts itself (group counts that do not add up to the row count, a row
  with no platform or measure, an unknown status) fails as
  `INTERNAL_STATE_INCONSISTENCY` with no write. Neither value moves backwards,
  and `last_successful_period_end` stays a high-water mark: a later month moves
  it past an unimported earlier one, so the driver's expected-start rule, not
  Thoth, keeps months from being skipped. A repeat after the lease was already
  released returns the checkpoint without writing when this import's progress
  is already recorded; otherwise it is `STALE_SOURCE_CLAIM`. The repeat judges
  the import again on the registry as it is then, so a registry change after
  the import can flip the judgement either way: a recorded success that is now
  judged unsuccessful still returns the recorded checkpoint, and an import
  that is now judged successful, while the checkpoint has not reached its
  period end, is `STALE_SOURCE_CLAIM` until a fresh claim re-runs
  `beginMetricImport` (which returns the terminal import unchanged) and the
  update, which then advances. The published descriptions of
  `lastSuccessfulPeriodEnd` ("successfully and completely collected") and of the
  `updateMetricSourceCheckpoint` mutation ("all of which cover exactly the
  import period completely") predate rule B and are unchanged; this paragraph
  is the authority for them.
- **Errors.** Lifecycle failures use `extensions.type` from exactly
  `SOURCE_NOT_FOUND`, `SOURCE_ACCOUNT_NOT_FOUND`, `SOURCE_NOT_ELIGIBLE`,
  `METRICS_COLLECT_NOT_ENTITLED`, `STALE_SOURCE_CLAIM`, `IMPORT_NOT_FOUND`,
  `IMPORT_IDEMPOTENCY_MISMATCH`, `UNEXPECTED_BATCH_KEY`, `IMPORT_INCOMPLETE`,
  `INVALID_IMPORT_STATE` and `LIFECYCLE_LIMIT_EXCEEDED`, with fixed messages. The citation
  operations (CITATIONS-01, below) add `CITATION_WORK_NOT_IN_PUBLISHER`,
  `CITATION_REFRESH_NOT_FOUND`, `CITATION_REFRESH_NOT_RUNNING` and
  `CITATION_BATCH_IDEMPOTENCY_MISMATCH`.
  A coordinator request failure carries its `MetricIngestionErrorCode` value
  unchanged, and a database failure is `INTERNAL_DATABASE_ERROR`; a refused
  coordinator-connection reservation, a guard lock-timeout expiry and a
  canonical statement timeout are database failures and carry that same
  code with its fixed message. No message carries SQL, a constraint name,
  connection detail, a token, source configuration or caller input.

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
  section 3.2 successful-period predicate (rule A for a day; for a source that
  is not CloudFront also rule B, a `MONTH` unit or an allowlisted `PARTIAL`
  month), the single `UPDATE` that records progress and releases the lease also
  inserts or replaces the entry for the import's `period_start`, using the
  `manifestDigest` of its strict `thoth-managed-driver-import/1` envelope, and
  evicts the chronologically oldest entries beyond 64. For a `MONTH` account an
  entry is one month, so the 64 entries are the newest 64 months and older
  months are evicted by period, as days are. `COMPLETED_WITH_ERRORS`, zero
  coverage rows and any row that neither rule accepts (a mismatched period, a
  `PARTIAL` row outside the allowlist, any `UNKNOWN` row) leave the cursor
  unchanged, with the single narrow exception of the section 3.4 CloudFront
  quarantine-only import; non-terminal and `FAILED` imports and stale, expired,
  reclaimed or foreign tokens change nothing. Successful reprocessing of an
  older period may replace its digest and never moves
  `last_successful_period_end` backwards.
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
  `COMPLETE` for exactly the import's one-day period (rule A of section 3.2,
  the strict day test: rule B, the `MONTH` rule, never applies to this
  exception, so month-shaped coverage leaves the import ineligible), the update
  records `last_completed_at`, records or replaces that period's entry in the
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
fixed redacted error. Since `MET-WP11-B-CORR-H3` (issue #989)
`BLOCKED_INCONSISTENT_EVIDENCE` also covers a canonical reporting-grain
mismatch: quarantined evidence whose DOI now resolves to a record of the same
v1 identity whose persisted `reporting_grain` differs from the quarantine
row's grain is blocked with that existing state, before any duplicate,
source-conflict or revision decision, under the same capped 1h, 2h, 4h, 8h,
16h, 24h schedule; the original quarantine row, provenance, import counters,
import status and coverage are not mutated, only the existing reconciliation
attempt/backoff bookkeeping changes, and no new persisted state, reason or
column exists.

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
  `metric_rollup_work_day_state` lock, by the section 3.7 rebuild and by the
  section 3.8 build steps, each beneath the same state-row lock, and their
  only source is `metric_rollup_work_day`. Nothing reads canonical records, revisions or
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
  serve from the monthly tables until the migration has succeeded and, under
  separate authorization, the derived state has been shown exact — values,
  dependency flags, ambiguity state and row watermarks — at a recorded
  `applied_through_sequence` by either marker:
  - `verifyMetricRollupMonths` (section 3.7) exact at a recorded frontier,
    whether or not a section 3.7 rebuild preceded it; or
  - a section 3.8 run of either mode that has reached `RECONCILED`, with its
    run id and frontier.

  In either case the evidence is taken in the activation window (section
  3.8), under the activation grant, which names the evidence run and is its
  run grant, and is used at once. The latest-run check is made with
  `metricRollupMonthRebuildRun` (no `runId`) immediately before the restart
  that switches serving on: for a `RECONCILED` marker it must still show that
  run as the latest, and the run counts only if its `completedAt` falls after
  the activation grant; for a `verifyMetricRollupMonths` marker, the latest
  run is read before the verify is sent and again before the switch, and both
  reads must show the same terminal run, or none. No clocks are compared.
  Between the evidence and the switch no other section 3.8 run is started,
  advanced or abandoned, and no `verifyMetricRollupMonths` or
  `rebuildMetricRollupMonths` call is made. Evidence is never carried
  forward: if anything happened in between, or there is any doubt, fresh
  evidence is taken. Run facts stay in PostgreSQL, so no record outside Thoth
  is needed, and Sphinx keeps one invocation's evidence in memory only. A
  `verifyMetricRollupMonths` outcome is returned to its caller and is not
  stored; it survives only as the citation on the stage issue, so activation
  should prefer a `VERIFY` run that reaches `RECONCILED`, whose run row stays
  in PostgreSQL. The run id, frontier and time are cited on the stage issue as
  the record of what was used, not as standing evidence. `VERIFIED` is never
  sufficient; a `VERIFY` run reaching `RECONCILED` is enough on its own,
  however the state was built. `MET-WP4-03B` serving activation and
  downstream dashboard cutover remain prohibited while the tables are empty,
  partially rebuilt or unreconciled; after activation, section 3.8 records
  one exception, for post-activation repair. After a downgrade and
  reapplication the same gate applies again, because normal completion
  maintains only the months it subsequently touches, and reapplying takes
  fresh evidence the same way.
- **Only resolved `DAY` rows.** These projections remain aggregations of
  resolved `DAY` rows only, and records of any other grain remain outside the
  work-day stream. `MONTH` records reach the reads only through the separate
  native-month projection below (`MET-WP4-MONTH-01`).

Scope boundaries a consumer must not infer:

- No GraphQL operation, type, field, argument or scalar was added or
  changed; at `MET-WP4-03A`, `metricDashboard` (section 4.1) still served
  from `metric_rollup_work_day` only. The monthly tables, their flags and
  ambiguity state reached no API in `MET-WP4-03A`; their one reader is the
  `MET-WP4-03B` dashboard contract of section 4.2, which reads them and
  never writes them.
- The monthly maintenance surfaces are the pair of protected operations of
  section 3.7 (a read-only consistency verification and an exceptional,
  self-verifying full rebuild) and the resumable run of section 3.8, whose
  build steps replace only inexact ranges, each atomically, and whose pages
  only verify. There is no row-level patching or generic full-projection
  reconciliation-ledger operation. No `metric_rollup_work_month_state`,
  `metric_work_dimension` or secondary index is implied.
- Sphinx owns orchestration, including the initial/exceptional rebuild, the
  section 3.8 runs and any periodic verification workflow, but never writes
  the Thoth database or computes monthly projection values. Normal
  incremental maintenance, every full rebuild and every section 3.8 step
  remain Thoth-owned database operations. A verification
  mismatch must alert/HOLD rather than silently auto-rebuild; Thoth never
  invokes a rebuild on its own.

#### Native-month projection (`MET-WP4-MONTH-01`)

`MET-WP4-MONTH-01` (thoth#1042) adds four derived tables, created and
populated by migration `20261116_v1.12.0`, with no foreign key, trigger or
secondary index. Each primary key is the identity, work first:

- `metric_native_month`: `work_id`, `platform_id`, `measure_id` and
  `month_start` (the first day of a month), all `NOT NULL`, as the key;
  `value bigint NULL` (`NULL`: total-ambiguous); `live`,
  `requires_country_coverage`, `requires_institution_coverage`,
  `country_ambiguous` and `institution_ambiguous`, each `boolean NOT NULL`;
- `metric_native_country_month`: the same four and `country_code
  character(2)` (two capital letters) as the key; `value bigint NOT NULL`,
  `requires_institution_coverage boolean NOT NULL`;
- `metric_native_institution_month`: the four and `institution_id` as the
  key; `value bigint NOT NULL`, `requires_country_coverage boolean NOT NULL`;
- `metric_native_refusal`: `work_id`, `platform_id`, `measure_id`,
  `period_start` and `period_end` as the key; `inconsistent boolean NOT NULL`.

The migration populates them from every non-`DAY` record by the rule below,
in set-based statements, while it holds `metric_source_account` in `ACCESS
EXCLUSIVE` mode to its commit, so no canonical write runs meanwhile and the
tables are exact from their first commit.

- **Keys.** A key is `(work, platform, measure, period_start)`: every non-`DAY`
  record that starts on that day, whatever its end or grain (the columns of
  `metric_record_native_grain_idx`). So a write to one period never removes
  another period's rows. `apply_canonical_application` takes a set of keys and
  adds the record's key on every `Winner` and `Revision` outcome of a
  non-`DAY` record, so no caller can apply a candidate without one. Each
  caller recomputes its set after its last canonical write and before it
  commits: ingestion once per request, quarantine reconciliation once per row.
  An empty set runs no statement.
- **Recompute.** Eight statements, whatever the number of keys (passed as
  parallel arrays): four delete the keys' rows from the four tables, and four
  insert them again from the keys' records, read through
  `metric_record_native_grain_idx` and each revision by its primary key. Each
  record is classified by its pointer, as the native-grain check of section
  4.2 does:
  - *withdrawn* (the pointer names a `RETRACTED` revision and the record has
    no `CURRENT` one): ignored;
  - *active reporting period*: a `REPORTING_PERIOD` record whose pointer
    names a `CURRENT` revision;
  - *served month*: a `MONTH` record spanning exactly one calendar month whose
    pointer names a `CURRENT` revision of that record; its value is that
    revision's;
  - *inconsistent*: anything else of either grain, a `MONTH` record of any
    other period included.
- **Rows.** Per key it writes:
  - one `metric_native_refusal` row per period of the key that holds an
    active reporting period or an inconsistent record, `inconsistent` true if
    it holds an inconsistent one;
  - one `metric_native_month` row if the key holds a served month: `live`
    (some served record's value is not 0), and the cell resolved exactly as
    03A resolves a day (Specification Amendment 6): the undimensioned row if
    present, else the single represented mask, summed, else total-ambiguous
    (`value` `NULL`, both dependency flags false); the two dependency flags of
    the selected mask; and the two section-ambiguity flags (incomparable
    minimal masks);
  - the cell's country and institution rows: the rows of the unique least
    target-bearing mask, summed per country or institution, with the
    cross-dependency flag.
- **Atomic and exact.** Sums are signed `bigint`; an overflow, or any failure,
  fails the caller's transaction, so nothing commits. The recompute takes no
  lock beyond its own rows, after every lock ingestion takes. Every canonical
  write holds its work row `FOR UPDATE`, so writers of one key are serialized
  and each recompute sees every committed record of its key; records and rows
  commit together. A native row is a function of the current records of its
  own key, so recomputing it is idempotent and needs no order.
- **Other writers.** Any later path that changes a non-`DAY` record or its
  revisions (a retraction path included) recomputes the key in the same
  transaction, under the work row lock. A change made outside Thoth leaves the
  tables stale until V-NATIVE finds it: under `ON` the projection, not the
  records, decides a value and a refusal, so a missing row can read as a
  justified `"0"`. V-NATIVE is therefore a safety gate (section 2.2).
- **Whatever the switch.** The tables are maintained whatever
  `THOTH_METRIC_MONTH_SERVING` says, and hold no frontier, watermark,
  position, claim or lag. Only a server with monthly serving `ON` reads them
  (sections 4.1 to 4.4), and its month/day conflict rule relies on the 03A
  tables being exact (section 3.8).

V-NATIVE, the same text as section 2.2 gives with when it runs; its key form
is there too:

```sql
WITH record AS (
  SELECT r.work_id, r.platform_id, r.measure_id, r.period_start, r.period_end,
    r.reporting_grain::text AS grain, r.country_code, r.institution_id, p.value,
    CASE r.reporting_grain WHEN 'QUARTER' THEN 3 ELSE 1 END AS span,
    CASE WHEN r.publication_id IS NULL THEN 0 ELSE 4 END
      + CASE WHEN r.country_code IS NULL THEN 0 ELSE 2 END
      + CASE WHEN r.institution_id IS NULL THEN 0 ELSE 1 END AS mask,
    CASE
      WHEN p.status = 'RETRACTED' AND NOT EXISTS (SELECT 1 FROM metric_record_revision c
        WHERE c.record_id = r.record_id AND c.status = 'CURRENT') THEN 'WITHDRAWN'
      WHEN p.status = 'CURRENT' AND r.reporting_grain = 'REPORTING_PERIOD' THEN 'ACTIVE'
      WHEN p.status = 'CURRENT' AND r.reporting_grain IN ('MONTH', 'QUARTER')
        AND r.period_start = date_trunc('month', r.period_start::timestamp)::date
        AND (r.reporting_grain = 'MONTH' OR EXTRACT(MONTH FROM r.period_start) IN (1, 4, 7, 10))
        AND r.period_end = (r.period_start::timestamp
          + CASE r.reporting_grain WHEN 'QUARTER' THEN interval '3 months'
            ELSE interval '1 month' END)::date
        THEN 'SERVED'
      ELSE 'INCONSISTENT'
    END AS class
  FROM metric_record r
  LEFT JOIN metric_record_revision p
    ON p.record_id = r.record_id AND p.record_revision_id = r.current_revision_id
  WHERE r.reporting_grain <> 'DAY'),
clash AS (
  SELECT DISTINCT a.work_id, a.platform_id, a.measure_id, a.period_start
  FROM record a JOIN record b
    ON (b.work_id, b.platform_id, b.measure_id) = (a.work_id, a.platform_id, a.measure_id)
    AND ((b.period_start = a.period_start AND b.span <> a.span)
      OR (b.period_start <> a.period_start AND b.period_start < a.period_end
        AND b.period_end > a.period_start))
  WHERE a.class = 'SERVED' AND b.class = 'SERVED'),
served AS (
  SELECT * FROM record s WHERE s.class = 'SERVED' AND NOT EXISTS (
    SELECT 1 FROM clash c
    WHERE (c.work_id, c.platform_id, c.measure_id, c.period_start)
      = (s.work_id, s.platform_id, s.measure_id, s.period_start))),
cell AS (
  SELECT work_id, platform_id, measure_id, period_start AS month_start,
    min(span) AS span_months,
    array_agg(DISTINCT mask) AS masks, bool_or(value <> 0) AS live
  FROM served GROUP BY work_id, platform_id, measure_id, period_start),
choice AS (
  SELECT cell.*,
    CASE WHEN 0 = ANY (masks) THEN 0 WHEN cardinality(masks) = 1 THEN masks[1] END AS total_mask,
    ARRAY(SELECT m FROM unnest(masks) AS m WHERE m & 2 = 2 AND NOT EXISTS (
      SELECT 1 FROM unnest(masks) AS n WHERE n & 2 = 2 AND n <> m AND n & m = n)) AS country_minima,
    ARRAY(SELECT m FROM unnest(masks) AS m WHERE m & 1 = 1 AND NOT EXISTS (
      SELECT 1 FROM unnest(masks) AS n WHERE n & 1 = 1 AND n <> m AND n & m = n)) AS institution_minima
  FROM cell),
expected_month AS (
  SELECT c.work_id, c.platform_id, c.measure_id, c.month_start,
    (SELECT sum(s.value) FROM served s
      WHERE (s.work_id, s.platform_id, s.measure_id, s.period_start)
        = (c.work_id, c.platform_id, c.measure_id, c.month_start)
        AND s.mask = c.total_mask)::bigint AS value,
    c.span_months, c.live,
    COALESCE(c.total_mask & 2 = 2, FALSE) AS requires_country_coverage,
    COALESCE(c.total_mask & 1 = 1, FALSE) AS requires_institution_coverage,
    cardinality(c.country_minima) > 1 AS country_ambiguous,
    cardinality(c.institution_minima) > 1 AS institution_ambiguous
  FROM choice c),
expected_country AS (
  SELECT s.work_id, s.platform_id, s.measure_id, s.period_start AS month_start, s.country_code,
    sum(s.value)::bigint AS value, bool_or(s.mask & 1 = 1) AS requires_institution_coverage,
    min(s.span) AS span_months
  FROM served s JOIN choice c
    ON (c.work_id, c.platform_id, c.measure_id, c.month_start)
      = (s.work_id, s.platform_id, s.measure_id, s.period_start)
  WHERE cardinality(c.country_minima) = 1 AND s.mask = c.country_minima[1]
  GROUP BY s.work_id, s.platform_id, s.measure_id, s.period_start, s.country_code),
expected_institution AS (
  SELECT s.work_id, s.platform_id, s.measure_id, s.period_start AS month_start, s.institution_id,
    sum(s.value)::bigint AS value, bool_or(s.mask & 2 = 2) AS requires_country_coverage,
    min(s.span) AS span_months
  FROM served s JOIN choice c
    ON (c.work_id, c.platform_id, c.measure_id, c.month_start)
      = (s.work_id, s.platform_id, s.measure_id, s.period_start)
  WHERE cardinality(c.institution_minima) = 1 AND s.mask = c.institution_minima[1]
  GROUP BY s.work_id, s.platform_id, s.measure_id, s.period_start, s.institution_id),
expected_refusal AS (
  SELECT work_id, platform_id, measure_id, period_start, period_end,
    bool_or(class <> 'ACTIVE') AS inconsistent
  FROM record r WHERE class IN ('ACTIVE', 'INCONSISTENT')
    OR (class = 'SERVED' AND EXISTS (
      SELECT 1 FROM clash c
      WHERE (c.work_id, c.platform_id, c.measure_id, c.period_start)
        = (r.work_id, r.platform_id, r.measure_id, r.period_start)))
  GROUP BY work_id, platform_id, measure_id, period_start, period_end),
diff AS (
  SELECT 'metric_native_month' AS relation,
    CASE WHEN a.work_id IS NULL THEN 'missing' WHEN e.work_id IS NULL THEN 'unexpected'
      ELSE 'differing' END AS kind,
    COALESCE(e.work_id, a.work_id) AS work_id, COALESCE(e.platform_id, a.platform_id) AS platform_id,
    COALESCE(e.measure_id, a.measure_id) AS measure_id,
    COALESCE(e.month_start, a.month_start) AS period_start, NULL::text AS detail
  FROM expected_month e FULL JOIN metric_native_month a
    ON (a.work_id, a.platform_id, a.measure_id, a.month_start)
      = (e.work_id, e.platform_id, e.measure_id, e.month_start)
  WHERE a.work_id IS NULL OR e.work_id IS NULL
    OR (e.value, e.span_months, e.live, e.requires_country_coverage,
        e.requires_institution_coverage, e.country_ambiguous, e.institution_ambiguous)
      IS DISTINCT FROM (a.value, a.span_months, a.live, a.requires_country_coverage,
        a.requires_institution_coverage, a.country_ambiguous, a.institution_ambiguous)
  UNION ALL
  SELECT 'metric_native_country_month',
    CASE WHEN a.work_id IS NULL THEN 'missing' WHEN e.work_id IS NULL THEN 'unexpected'
      ELSE 'differing' END,
    COALESCE(e.work_id, a.work_id), COALESCE(e.platform_id, a.platform_id),
    COALESCE(e.measure_id, a.measure_id), COALESCE(e.month_start, a.month_start),
    COALESCE(e.country_code::text, a.country_code::text)
  FROM expected_country e FULL JOIN metric_native_country_month a
    ON (a.work_id, a.platform_id, a.measure_id, a.month_start, a.country_code::text)
      = (e.work_id, e.platform_id, e.measure_id, e.month_start, e.country_code::text)
  WHERE a.work_id IS NULL OR e.work_id IS NULL
    OR (e.value, e.span_months, e.requires_institution_coverage)
      IS DISTINCT FROM (a.value, a.span_months, a.requires_institution_coverage)
  UNION ALL
  SELECT 'metric_native_institution_month',
    CASE WHEN a.work_id IS NULL THEN 'missing' WHEN e.work_id IS NULL THEN 'unexpected'
      ELSE 'differing' END,
    COALESCE(e.work_id, a.work_id), COALESCE(e.platform_id, a.platform_id),
    COALESCE(e.measure_id, a.measure_id), COALESCE(e.month_start, a.month_start),
    COALESCE(e.institution_id, a.institution_id)::text
  FROM expected_institution e FULL JOIN metric_native_institution_month a
    ON (a.work_id, a.platform_id, a.measure_id, a.month_start, a.institution_id)
      = (e.work_id, e.platform_id, e.measure_id, e.month_start, e.institution_id)
  WHERE a.work_id IS NULL OR e.work_id IS NULL
    OR (e.value, e.span_months, e.requires_country_coverage)
      IS DISTINCT FROM (a.value, a.span_months, a.requires_country_coverage)
  UNION ALL
  SELECT 'metric_native_refusal',
    CASE WHEN a.work_id IS NULL THEN 'missing' WHEN e.work_id IS NULL THEN 'unexpected'
      ELSE 'differing' END,
    COALESCE(e.work_id, a.work_id), COALESCE(e.platform_id, a.platform_id),
    COALESCE(e.measure_id, a.measure_id), COALESCE(e.period_start, a.period_start),
    COALESCE(e.period_end, a.period_end)::text
  FROM expected_refusal e FULL JOIN metric_native_refusal a
    ON (a.work_id, a.platform_id, a.measure_id, a.period_start, a.period_end)
      = (e.work_id, e.platform_id, e.measure_id, e.period_start, e.period_end)
  WHERE a.work_id IS NULL OR e.work_id IS NULL OR e.inconsistent IS DISTINCT FROM a.inconsistent)
SELECT
  count(*) FILTER (WHERE relation = 'metric_native_month' AND kind = 'missing') AS month_missing,
  count(*) FILTER (WHERE relation = 'metric_native_month' AND kind = 'unexpected') AS month_unexpected,
  count(*) FILTER (WHERE relation = 'metric_native_month' AND kind = 'differing') AS month_differing,
  count(*) FILTER (WHERE relation = 'metric_native_country_month' AND kind = 'missing') AS country_missing,
  count(*) FILTER (WHERE relation = 'metric_native_country_month' AND kind = 'unexpected') AS country_unexpected,
  count(*) FILTER (WHERE relation = 'metric_native_country_month' AND kind = 'differing') AS country_differing,
  count(*) FILTER (WHERE relation = 'metric_native_institution_month' AND kind = 'missing') AS institution_missing,
  count(*) FILTER (WHERE relation = 'metric_native_institution_month' AND kind = 'unexpected') AS institution_unexpected,
  count(*) FILTER (WHERE relation = 'metric_native_institution_month' AND kind = 'differing') AS institution_differing,
  count(*) FILTER (WHERE relation = 'metric_native_refusal' AND kind = 'missing') AS refusal_missing,
  count(*) FILTER (WHERE relation = 'metric_native_refusal' AND kind = 'unexpected') AS refusal_unexpected,
  count(*) FILTER (WHERE relation = 'metric_native_refusal' AND kind = 'differing') AS refusal_differing
FROM diff;
```

**Repair.** `recomputeMetricNativeMonths(publisherId: Uuid!):
MetricNativeMonthRecomputeResult!` is **SUPERUSER only**, authorized before
any operation-specific read, lock or write. For the works now under the
publisher's imprints, in ascending id and in chunks of at most 50, each chunk
one transaction, it locks the works `FOR UPDATE`, deletes their rows from the
four tables and inserts them again by the rule above. `works` is the number of
works recomputed and `rows` the number of rows written. It is idempotent, it
writes no canonical row and no audit entry, and it runs beside ingestion and
reads. A failed chunk (a deadlock with a publisher or imprint change included)
leaves the earlier ones committed and exact, and is retried by calling it
again. An unknown publisher fails as not found before any write.

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
  No maintenance mode, queue, pause flag, scheduler or process-wide mutex
  exists. `MET-WP4-03A-OPS-03` adds exactly one coordination table,
  `metric_rollup_month_rebuild_run`, for the caller-driven runs of section
  3.8.
- **Bounds are safety envelopes, not SLOs.** `MAX_REBUILD_MONTH_KEYS =
  100000`, chunk size 50, verification `statement_timeout = 30s`, rebuild
  `lock_timeout = 5s`, rebuild `statement_timeout = 30s`, total rebuild
  elapsed ceiling 120s. They are not raised silently; a larger corpus or a
  different envelope needs a separately approved amendment. Above 100000
  represented month keys, or once whole-table verification nears its 30 s
  statement, the supported path is the resumable run of section 3.8.
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

### 3.8 Resumable rebuild and paged verification (`MET-WP4-03A-OPS-03`)

`MET-WP4-03A-OPS-03` (issue #1012) adds a resumable, caller-driven rebuild and
a paged verification of the six derived rollup families (the four monthly
tables of section 3.6 and the two yearly tables of section 4.2). Both work at
any corpus size, in bounded steps that each commit on their own, while rollup
completions and canonical ingestion keep running. A run records durable
progress and ends in a durable terminal status. `RECONCILED` means a complete
exact traversal at one recorded frontier, and it is the marker a
serving-activation gate may cite (section 3.6). The section 3.7 pair is
unchanged: its SDL, SQL text, bounds and behaviour. Thoth runs nothing on its
own: there is no worker, queue, lease, scheduler or timer, and the caller
drives every step.

A damaged 03A can hide a month/day conflict from reads, which then count both
sides, until it is rebuilt (`MET-WP4-MONTH-01`, section 4.1).

Object, created empty by migration `20261112_v1.10.0`, with no lock on any
existing table:

```text
metric_rollup_month_rebuild_run
  run_id uuid PK, mode (REBUILD | VERIFY), status (BUILDING | VERIFYING |
  RECONCILED | VERIFIED | MISMATCHED | ABANDONED), step_count,
  cursor_work_id, cursor_year_start, started_by (stored, never returned),
  started_at, updated_at, build_completed_at, completed_at,
  start_applied_through_sequence, build_* and verify_* counters and
  first/last frontiers, verify_<family>_{expected,actual,missing,extra,mismatched}_rows
  for the six families

metric_rollup_month_rebuild_run_one_open_idx
  UNIQUE ON ((true)) WHERE status IN ('BUILDING', 'VERIFYING')
```

Closed vocabularies are `CHECK`s. Nineteen `CHECK` constraints pin the row's
shape: the cursor pair is both `NULL` or both set at a 1 January, and is `NULL`
at `RECONCILED`, `VERIFIED` and `MISMATCHED`; `completed_at` is `NULL` exactly
while the run is open; `BUILDING` only for `REBUILD`, with no verify counter or
frontier yet; a `VERIFY` run has no build counter, frontier or
`build_completed_at`; a `REBUILD` run past its build phase has
`build_completed_at`, and that implies at least one step and both build
frontiers; an abandoned `REBUILD` run that never finished building has no
verify counter or frontier, while an abandoned `VERIFY` run keeps its pages;
frontier pairs are both `NULL` or both set and never decrease
(`start <= build_first <= build_last <= verify_first <= verify_last`, bridged
when the build pair is `NULL`); every terminal verify status implies at least
one page and both verify frontiers; `RECONCILED` implies equal verify
frontiers and the 18 missing, extra and mismatched counts at zero, `VERIFIED`
a moved frontier and the same zeros, `MISMATCHED` at least one non-zero count;
`started_by` is not blank; every counter is non-negative and `step_count =
build_steps + verify_pages`. There is no foreign key on the cursor: it is a
position in unit order. `down.sql` drops the index and the table; no other
table, column, index, trigger, function or enum changes.

Operations, all on `MutationRoot`, all requiring exactly
`METRICS_INGEST_SERVICE`, each authorized before any rollup-specific read,
lock or write:

```graphql
startMetricRollupMonthRebuild(input: StartMetricRollupMonthRebuildInput!): MetricRollupMonthRebuildRun!
advanceMetricRollupMonthRebuild(input: AdvanceMetricRollupMonthRebuildInput!): MetricRollupMonthRebuildAdvance!
abandonMetricRollupMonthRebuild(input: AbandonMetricRollupMonthRebuildInput!): MetricRollupMonthRebuildRun!
metricRollupMonthRebuildRun(runId: Uuid): MetricRollupMonthRebuildRun
```

```graphql
enum MetricRollupMonthRebuildMode { REBUILD VERIFY }
enum MetricRollupMonthRebuildStatus { BUILDING VERIFYING RECONCILED VERIFIED MISMATCHED ABANDONED }

input StartMetricRollupMonthRebuildInput   { mode: MetricRollupMonthRebuildMode! }
input AdvanceMetricRollupMonthRebuildInput { runId: Uuid! expectedStepCount: String! }
input AbandonMetricRollupMonthRebuildInput { runId: Uuid! }

type MetricRollupMonthRebuildFamilies {
  total: MetricRollupMonthProjectionVerification!
  country: MetricRollupMonthProjectionVerification!
  institution: MetricRollupMonthProjectionVerification!
  ambiguity: MetricRollupMonthProjectionVerification!
  countryYear: MetricRollupMonthProjectionVerification!
  institutionYear: MetricRollupMonthProjectionVerification!
}

type MetricRollupMonthRebuildBuildProgress {
  steps: String!
  replacedSteps: String!
  units: String!
  monthKeys: String!
  workDayRows: String!
  foundMissingRows: String!
  foundExtraRows: String!
  foundMismatchedRows: String!
  firstAppliedThroughSequence: String
  lastAppliedThroughSequence: String
}

type MetricRollupMonthRebuildVerifyProgress {
  pages: String!
  units: String!
  monthKeys: String!
  workDayRows: String!
  firstAppliedThroughSequence: String
  lastAppliedThroughSequence: String
  singleFrontier: Boolean!
  counts: MetricRollupMonthRebuildFamilies!
}

type MetricRollupMonthRebuildRun {
  runId: Uuid!
  mode: MetricRollupMonthRebuildMode!
  status: MetricRollupMonthRebuildStatus!
  stepCount: String!
  positionPermille: Int!
  startedAt: Timestamp!
  updatedAt: Timestamp!
  buildCompletedAt: Timestamp
  completedAt: Timestamp
  startAppliedThroughSequence: String!
  build: MetricRollupMonthRebuildBuildProgress
  verify: MetricRollupMonthRebuildVerifyProgress!
}

type MetricRollupMonthRebuildAdvance {
  advanced: Boolean!
  run: MetricRollupMonthRebuildRun!
}
```

The authorization matrix is section 3.7's for all four operations: anonymous
callers, `SUPERUSER`, `METRICS_READ_SERVICE`, publisher-scoped human roles,
`DISSEMINATION_WORKER` and every other machine role are denied, and a denied
call opens no transaction and issues no rollup-specific statement. No field on
`QueryRoot` or on any public type reaches these types. `build` is `null` for a
`VERIFY` run; `MetricRollupMonthProjectionVerification` is section 3.7's type,
reused unchanged; the `counts` totals are sums over pages, which are not a
snapshot total when pages saw different frontiers; `singleFrontier` is true
when both verify frontiers are set and equal. `positionPermille` estimates the
current phase's cursor position as `floor(first 32 bits of the cursor work id
x 1000 / 2^32)`: 0 with no cursor mid-phase, 1000 at `RECONCILED`, `VERIFIED`
and `MISMATCHED`, the retained cursor at `ABANDONED`; it is never a count. No
cursor, work, platform, measure or principal identity is ever returned: only
counts, frontiers, timestamps and the run's own id leave Thoth. The caller
supplies no size, range, key, value or SQL; every budget is server-fixed.

Definitions:

- **Unit.** `(work_id, year_start)`, `year_start` the 1 January of a calendar
  year. A row's unit is its `work_id` and the year of its `day`, `month_start`
  or `year_start`. Units are ordered by `work_id` (PostgreSQL `uuid` order),
  then `year_start`, and every comparison is made in SQL. Every row of the six
  families and every work-day row belongs to exactly one unit, and every year
  group `(work, platform, measure, year)` lies inside one unit.
- **Range.** `(lo, hi]` in unit order: `lo` is the phase cursor or minus
  infinity, `hi` a unit or plus infinity. A range covers every row of the six
  families and of the work-day projection whose unit lies in it, orphans (rows
  whose unit has no work-day row) included. The work-day range is the `(work_id,
  day)` index range `> (lo work, 31 December of lo year)` and `<= (hi work, 31
  December of hi year)`; a projection range uses `work_id` bounds, which every
  identity index leads with, and a filter on the row's year.
- **Frontier.** `applied_through_sequence` (`W`). `next_sequence - 1 > W` is
  ordinary lag, never corruption.
- **Exact range at `W`.** The six families restricted to the range equal what
  the independent range verifier derives from the range's work-day rows
  (identities, values, publication, country and institution identity,
  dependency flags, ambiguity flags and row watermarks), and no derived row in
  the range is watermarked above `W`.
- **Step, page, hold.** A step is one committed build transaction and a page
  one committed verify transaction; `stepCount` counts both. The hold is the
  time from acquiring the state row to the end of the step's transaction.

Run state machine:

```text
start(REBUILD) -> BUILDING (cursor NULL)
BUILDING  --build step--> BUILDING (cursor advanced)
BUILDING  --build step whose range reached plus infinity--> VERIFYING (cursor NULL, build_completed_at set)
start(VERIFY) -> VERIFYING (cursor NULL)
VERIFYING --page--> VERIFYING (cursor advanced)
VERIFYING --page whose range reached plus infinity-->
      RECONCILED  every missing/extra/mismatched count 0 and verify_first_frontier = verify_last_frontier
      VERIFIED    every such count 0 and the frontier moved
      MISMATCHED  any such count non-zero
BUILDING or VERIFYING --abandon--> ABANDONED
RECONCILED, VERIFIED, MISMATCHED and ABANDONED are terminal.
```

Contract properties consumers may rely on:

- **Start.** One `READ COMMITTED` transaction under a 5 s `lock_timeout` and a
  30 s `statement_timeout` reads the frontier without a lock and inserts the
  run, recording the authenticated principal. A second open run violates the
  one-open-run index and is refused `METRIC_REBUILD_RUN_OPEN`, so concurrent
  first arrivals yield exactly one run. After a lost response the caller reads
  `metricRollupMonthRebuildRun` and adopts the open run if its mode matches
  and its `stepCount` is 0.
- **Advance.** `expectedStepCount` must be a decimal string holding a
  non-negative 64-bit integer, decided before any database access; otherwise
  `METRIC_QUERY_INVALID`. An unlocked read of the run, in its own short
  transaction under a 1 s lock and statement timeout, chooses the build step or
  the verify page; an unknown run is `METRIC_QUERY_INVALID`, and a terminal run
  or a different `step_count` returns `advanced: false` with the run
  unchanged. The chosen transaction re-checks status and `step_count` under
  its locks and writes nothing if either changed. A retry after a lost
  response is therefore safe.
- **Abandon.** `READ COMMITTED`: it locks the run row under the 5 s
  `lock_timeout`, sets `ABANDONED` and `completed_at` if the run is open, and
  otherwise returns it unchanged; an unknown run is `METRIC_QUERY_INVALID`.
  Against an in-flight step or page it waits for it, or returns
  `METRIC_REBUILD_RETRYABLE`. The cursor, counters and frontiers are kept, and
  the projections stay exactly as the committed steps left them.
- **Status read.** `metricRollupMonthRebuildRun(runId)` returns that run or
  `null`. Without `runId` it returns the open run when one exists, otherwise
  the run with the greatest `started_at` (set by the run's insert; ties broken
  by `run_id`), or `null`, so a `RECONCILED` run stops being the latest once any
  later run starts. It is read-only, under the dispatch read's 1 s lock and
  statement timeouts.
- **One connection, bounded checkout.** Each operation checks out exactly one
  pooled connection with a 2 s bounded wait; an advance runs its dispatch read
  and its step or page on that connection. An expired wait is
  `METRIC_REBUILD_RETRYABLE`. Counted from the start of the resolver, an
  advance ends on the server within about 15 s for a step and about 18 s for a
  page.
- **Per-statement deadline.** Every transaction asserts its isolation level
  and fails closed on a mismatch. Before each statement inside a budget it runs
  `SET LOCAL statement_timeout = <ms>`, an integer the code computes: the
  smaller of 30,000 and the milliseconds left in the current budget, with a
  floor of 1. A check between statements refuses the next one once the budget
  is spent, and the timeout also cuts lock waits. Steps and pages also set
  `idle_in_transaction_session_timeout = '2s'`, so a stalled API process holds
  nothing longer than its budget plus 2 s.
- **Build step (`BUILDING`, `READ COMMITTED`).** Three consecutive budgets:
  `STEP_PRESIZE_BUDGET` before any row lock, `STEP_LOCK_BUDGET` from the first
  work-row lock until the state row is held, `STEP_HOLD_BUDGET` from the
  state-row lock to the end of the transaction. In order: session settings
  (`lock_timeout = '5s'`, `statement_timeout = 30000`, the idle timeout,
  `plan_cache_mode = force_custom_plan`); pre-size from the cursor without the
  state lock; lock the candidate range's works with one `SELECT ... FOR KEY
  SHARE` in ascending `work_id` order (canonical ingestion's own order); lock
  `metric_rollup_work_day_state` `FOR UPDATE` and read `W` (the hold starts);
  lock the run row `FOR UPDATE` and re-check it; re-size under the locks,
  admitting only units whose work is locked, ending the range before the first
  unit that needs an unlocked work (an empty re-size is
  `METRIC_REBUILD_RETRYABLE`); pre-verify the range at `W` with the range
  verifier. If the range is exact, no projection row is written. Otherwise the
  step deletes every row of the six families in the range (each family's
  deleted count must equal the pre-verify's actual count), lists the range's
  month keys from the `(work_id, day)` range, and recomputes them with the
  completion's own statements, issued one at a time in the completion's order
  and with its binds: per chunk of at most 50 month keys the source-watermark
  bound and the four monthly inserts, then per chunk of at most 50 year keys
  the country-year and institution-year inserts. It skips the completion's six
  keyed deletes, which the range delete has made provably empty, and calls no
  completion function. It then post-verifies the range and commits only an
  exact result. The run row advances its cursor to the range's last unit, or at
  plus infinity clears it, sets `VERIFYING` and `build_completed_at`; it adds
  the step's counters and the pre-verify's found counts, and records the build
  frontiers. A step never writes `metric_rollup_work_day`, a delta, the state
  row, a canonical record or revision, or a reconciliation-ledger row, never
  moves the frontier and never truncates or takes `ACCESS EXCLUSIVE`.
- **Verify page (`VERIFYING`, `REPEATABLE READ`).** Only `SET LOCAL`
  statements precede its table locks (`statement_timeout` = the page budget in
  milliseconds, `lock_timeout = '5s'`, the idle timeout, `plan_cache_mode`);
  then `LOCK TABLE` of the six projection tables in section 3.7's fixed order,
  `metric_rollup_work_day` and `metric_rollup_work_day_state`, each `IN ACCESS
  SHARE MODE`, all before the first query, so a concurrent section 3.7
  `TRUNCATE` or a table-rewriting migration cannot cross the snapshot. The
  isolation assertion is the first query and freezes the snapshot. The run row
  is locked `FOR UPDATE`; a run row updated after the snapshot is a
  serialization failure, `METRIC_REBUILD_RETRYABLE`. The page reads `W`
  without a lock in the same snapshot, sizes itself with the page budgets,
  verifies its range and updates only the run row: cursor, counters, per-family
  counts and frontiers, and at plus infinity the terminal status. A page takes
  no state-row or work-row lock and writes nothing but the run row. The page
  budget runs from the first `LOCK TABLE` to the end of the transaction.
- **Independent range verifier.** One statement in `verification.rs` applies
  section 3.7's generic rules (mask-set resolution by anti-join minimality,
  the Amendment 6 totals, the yearly state derived straight from work-day
  rows) to one range's work-day rows and actual rows. It returns section 3.7's
  five counts per family, the greatest actual watermark per family and the
  range's work-day row and month-key counts. It shares no SQL text, constant or
  macro with the writer. Its bounds are bind parameters, and every predicate is
  index-bounded by `work_id`. The rendered text of section 3.7's whole-table
  verification statement is byte-identical, pinned by a hash test.
- **Exactness invariants.** Assuming no out-of-band write reaches the
  projection or work-day tables: after a step commits with cursor `c`, every
  later committed state is exact on `(minus infinity, c]` at its own frontier
  (I1; this and `VERIFIED` also assume the completion's keyed recomputation is
  correct). Beyond the cursor nothing is claimed until its step (I2). A
  committed step either replaces a range with an exact one in one transaction
  or leaves it untouched; readers never wait for a step and see each range
  before or after it, never half-replaced; a range changes only if it was
  inexact (I3). Runs never write work-day rows, deltas, the state row,
  canonical rows or reconciliation ledgers and never move the frontier; pages
  write only the run row; a `VERIFY` run never writes a projection row (I4). A
  phase that reaches plus infinity tiles the whole unit space without gap or
  overlap, and every orphan lies in exactly one range (I5). `RECONCILED` proves
  the six families exact at one `W` as of the last page's snapshot, without the
  completion assumption: every page saw the same `W`, so no completion
  committed during the traversal and no step ran. `VERIFIED` proves each range
  exact at its own frontier and is health evidence only (I6).
- **Concurrency.** Canonical ingestion takes works `FOR UPDATE` and then the
  state row, the order a step uses; a direct wait behind a step is at most 7 s
  on a work row (`STEP_LOCK_BUDGET + STEP_HOLD_BUDGET`) and at most 5 s on the
  state row, each plus commit time, and under the pacing rule below a step adds
  at most about 12 s to any ingestion statement, leaving at least 18 s of its
  30 s statement timeout. Completions, claims and allocation serialize with
  steps on the state row and are not blocked by pages; a completion's
  foreign-key `FOR KEY SHARE` on works is compatible with a step's. Section
  3.7's rebuild serializes with steps on the state row, and its `TRUNCATE`
  waits for a page's `ACCESS SHARE`. Section 3.7's verification and the
  dashboard and widget reads are never blocked by a step or a page. A second
  caller on the same step gets `advanced: false` (or `RETRYABLE` if its lock
  budget runs out first); on the same page it gets `RETRYABLE`.
- **Lock order.** Step: works `FOR KEY SHARE` (ascending), the state row, the
  run row, then `ROW EXCLUSIVE` on the projection tables with their
  foreign-key `FOR KEY SHARE` on publication, platform, measure and
  institution. Page: the eight tables in `ACCESS SHARE`, then its run row. A
  step adds no lock-order edge and never waits for a work while holding the
  state row. The completion's pre-existing order (state row before its
  foreign-key lock on works) is unchanged; a step never deadlocks with
  ingestion unless that completion inversion is live.

Bounds, server-fixed in `metric_rollup_delta/mod.rs`; changing any needs an
amendment with evidence:

```text
STEP_PRESIZE_BUDGET         5 s    (pre-sizing, before any row lock)
STEP_LOCK_BUDGET            2 s    (first work-row lock until the state row is held)
STEP_HOLD_BUDGET            5 s    (state-row hold, enforced per statement)
STEP_MAX_MONTH_KEYS         1,000
STEP_MAX_WORK_DAY_ROWS      8,000      (budget amendment R11; was 50,000)
STEP_MAX_REPLACING_READ     600,000    (budget amendment R11; was 4,000,000; replacing-read figure of a
                                       step; provisional, F4 confirms)
STEP_CHUNK_KEYS             50     (= the section 3.7 chunk; months and years)
SIZING_BATCH_WORKS          256    (works per sizing statement)
PAGE_BUDGET                 15 s   (from LOCK TABLE to commit, enforced per statement)
PAGE_MAX_MONTH_KEYS         10,000
PAGE_MAX_WORK_DAY_ROWS      150,000    (budget amendment R11; was 500,000)
lock_timeout                5 s    (run row, LOCK TABLE, start and abandon)
statement_timeout ceiling   30 s
dispatch and status reads   1 s    (lock_timeout and statement_timeout)
pool checkout               2 s    (every operation, one connection)
```

- **Sizing.** One recursive sizing statement walks forward from the cursor
  work by work (the next `work_id` by `ORDER BY work_id LIMIT 1` over the
  `(work_id, day)` index, then that work's counts through an index range on
  that work alone), carrying running totals, and stops after the first work
  that fills a budget or after `SIZING_BATCH_WORKS` works. Whole works are
  taken while they fit; if the first work alone exceeds a budget, one more
  statement returns its per-year counts and the range takes its years while
  they fit; at least one unit is always taken. Another sizing statement is
  issued only when the batch limit stopped the walk with budgets unfilled.
- **Replacing-read figure.** The sum of a month term (over the step's
  work-months, the month keys of that work-month times the work's work-day rows
  in that month) and a year term (over the step's year keys, the work's total
  work-day rows across its whole history, years before the cursor included).
  Sizing computes it from the same per-work counts, and
  `STEP_MAX_REPLACING_READ` is a third step budget beside keys and rows. Every
  build step is sized by it, so units that each fit cannot add up past it.
- **What a replacing step costs.** Besides the figure, a replacing step pays
  per work-day row: each row is written into every monthly and yearly section
  row it reaches, and every section row runs its foreign-key checks, which
  `STEP_MAX_WORK_DAY_ROWS` bounds. The five keyed month statements reach the
  work-day rows through the identity index bounded by `work_id` alone, so each
  month key walks its work's whole history: the walk, month keys x whole-work
  rows, at most 12 times the year term. The pre-verify, range delete,
  post-verify and re-size read the work's whole history once per step.
- **The F4 line.** "Max of hold / fill <= 4 s in each mix", where fill is the
  smaller of the step's row fill (its work-day rows over
  `STEP_MAX_WORK_DAY_ROWS`) and its figure fill (its figure over
  `STEP_MAX_REPLACING_READ`), shows that every admitted step of at least the
  mix (a) corner's series count commits inside `STEP_HOLD_BUDGET` (fewer
  series: Limits below). It rests on these assumptions: the hold is close to
  linear in the step's work-day rows, its walk and its whole-work rows, at
  per-row costs that F4's density (a distinct country and institution on
  nearly every row) makes the dearest; the reads outside the figure stay near
  F4's, bounded by the whole-work threshold below; the dearest step is a
  repair step over fully built tables (measured); in exact state each section
  table holds at most one row per work-day row; and the evidence host stands
  in for production's, whose behaviour each run grant judges again. The
  admitted steps peak near two corners, each with the rows at
  `STEP_MAX_WORK_DAY_ROWS` and the figure full: mix (a), few series over a
  long history, with the whole-work rows at the threshold T (about
  600,000 / (8,000 + T) series); and mix (b), one year of history, whole-work
  rows 8,000, the terms equal at about 38 month keys per work-month. Each
  corner's hold in repair is predicted statement group by statement group
  from the per-statement profiles of the replacing steps: the walk (the month
  watermark, ambiguity and total inserts, charged at 5/3 for all five keyed
  month statements), the inserts (the two month section inserts less the walk
  charged to them, and the two year inserts), the verify (pre-verify, range
  deletes, post-verify and the re-size with its per-year sizing) and the rest
  of the hold. Each group is fitted by non-negative least squares against its
  own drivers (walk and inserts and verify on the first repair pass of each
  F4 class, the rest on the from-empty replacing steps of F2, F3, F4 and F7),
  at the larger of the largest measured statement count and that of a step at
  `STEP_MAX_MONTH_KEYS`; both predictions must be at most 4.5 s. A six-term
  pooled fit is reported for information only. Dead entries left until vacuum
  are reported, not accepted against.
- **Floor.** One work-year carrying every (platform, measure) pair of the
  production registry (every pair, enabled or not), each at F2's density over
  twelve months, with eight years of history, must have a figure within
  `STEP_MAX_REPLACING_READ`. Its figure is computed at F2's measured rows per
  key, as the evidence prints it: about 3,402 R^2 for R pairs at
  F2's 31.5 rows per key (Evidence below), so 600,000 admits
  R <= 13. The production pair count comes from the census's counts-only read
  or a separately authorized read, never from the `MET-WP4-03A-OPS-03`
  implementation grant, and is stated on the stage issue before the evidence
  is commissioned; above 13 the stage HOLDs for an amendment. While
  production's registry is not yet populated, the floor is also judged on the
  pair count of the legacy history to be imported, or on a planned registry
  the CTO states. Every `REBUILD` run grant reports the registry pair count
  against the floor (Operator procedure below).
- **Whole-work threshold T.** The largest whole-work row total among the F4
  replacing steps that committed within 4 s and F5's units within 4 s (every
  build and repair step of the level; Evidence below), every one at F4's
  density. If mix (a)'s second history length alone misses the F4 line or the
  page line, T is taken only from the F4 classes whose steps met the F4 line
  and whose pages met the page line, not from F5. T must be at least F3's
  whole-work total (the census maximum) and at least production's largest
  whole-work total; otherwise the stage HOLDs for an amendment. Production's
  total, like the pair count, comes from the census's counts-only read or a
  separately authorized read, never from the implementation grant; if
  production does not yet hold the legacy history, F3's total is the floor.
  The run-grant aggregate stays the binding check.
- **Limits.** No corpus-size ceiling: run time scales with the corpus and is
  reported, not bounded. A single unit above a size budget is still attempted
  alone; it is a HOLD only if it cannot finish inside `STEP_HOLD_BUDGET`, and
  then it fails the same step on every retry. F5's growths measure the
  largest units whose every step, build and repair alike, commits within
  4 s, which may be above the budgets. A step of several light years of one
  work with fewer series than
  the mix (a) corner's can have the corner's figure and walk while its walk
  costs more per entry; it is inside every budget, so the run-grant aggregate
  cannot see it. It needs a work with several light years before a heavy
  history, and it fails safe: a step that cannot finish inside
  `STEP_HOLD_BUDGET` is refused on every retry, the HOLD above. F4's mix (c)
  measures it, report only; it is not gated. A range whose orphan deletion
  cannot finish is the same HOLD. None of these is resolved by a silent
  constant change or a new index.

Section 17 evidence that sets or confirms these values: see "Evidence" below.

Errors: a local error boundary with fixed messages and a stable
`extensions.type`. No message carries a frontier, row, SQL, principal, cursor
or connection detail, and `thoth-errors` is unchanged.

| Code | When | Caller action |
| --- | --- | --- |
| `METRIC_QUERY_INVALID` | A malformed `expectedStepCount`, or an unknown `runId`. | Fix the request. |
| `METRIC_REBUILD_RUN_OPEN` | `start` while a run is open. | Adopt the run, or let it finish or abandon it. |
| `METRIC_REBUILD_RETRYABLE` | A lock timeout, a statement timeout, the step or page budget, a serialization failure, a deadlock, a session ended by the idle timeout or a closed connection, an expired 2 s pool checkout, or an empty re-sized range. Nothing was committed. | Back off at least 1 s in either phase, and while `BUILDING` at least the pacing pause; retry with the same `expectedStepCount` a bounded number of times, then HOLD. |
| `METRIC_REBUILD_BLOCKED` | A work-day row in the range watermarked above `W`, a source watermark above `W`, a `bigint` overflow, a delete-count mismatch, or a post-step verification that is not exact. Nothing was committed, and it will repeat. | HOLD for a separately authorized repair. |
| `INTERNAL_ERROR` | Anything else. | HOLD. |

The classifier works on the database error before any conversion: SQLSTATE
40001 (serialization failure) and a closed connection are retryable; the
server messages `canceling statement due to lock timeout`, `canceling
statement due to statement timeout`, `deadlock detected` and `terminating
connection due to idle-in-transaction timeout` are matched exactly and are
retryable; a unique violation on the one-open-run index is
`METRIC_REBUILD_RUN_OPEN`; `bigint out of range` is `METRIC_REBUILD_BLOCKED`;
anything unclassified is `INTERNAL_ERROR`, so a misclassification fails safe
as a HOLD and never causes a write. Exact matching assumes English server
messages (`lc_messages`), which the tests assert and every run grant checks.
After a crash, a deploy or a lost response the caller reads the run and
continues with its `stepCount`: committed steps and pages are durable and an
interrupted one rolled back.

Logging: one `info` line per committed step or page, one `warn` line per
refusal, with fixed text (`metric_rollup_rebuild run=<uuid> mode=... phase=...
step=<n> outcome=<replaced|exact|verified> units=... month_keys=...
work_day_rows=... missing=... extra=... mismatched=... frontier=<W>
position_permille=... hold_ms=...`; a range line `range_first=<work_id>/<year>
range_last=<work_id>/<year>` only when a count is non-zero; a refusal line
`refused=<RETRYABLE|BLOCKED|INTERNAL> reason=<fixed reason id>
range_first=... hold_ms=...`; and one line per terminal status with the step
count and both verify frontiers). Range identities go to server logs only,
never to GraphQL.

Operator procedure:

- **Run grants and the run-grant aggregate.** Every non-disposable run needs
  its own grant per environment, and every run grant checks `lc_messages`.
  Each `REBUILD` run grant first runs the read-only aggregate below over
  `metric_rollup_work_day` (counts only, a full scan, once per grant), in a
  `READ ONLY` transaction with a `statement_timeout` stated in the grant,
  through a read-only database session, never through a test command. It runs
  the constant `METRIC_ROLLUP_RUN_GRANT_AGGREGATE_SQL` from the merged source
  at the deployed tag. Taking each unit alone, it returns the units that exceed
  a step budget (rows, keys or figure), each with its work-day rows, month keys
  and replacing-read figure, and two maxima on every row: the largest
  month-key count per work-month and the largest whole-work row total (one row
  with `NULL` unit columns when no unit exceeds a budget). A unit over a step
  budget passes only if one of F5's growths (Evidence below) dominates it in
  all four of rows, keys, figure and whole-work rows; otherwise it is a HOLD
  before the run starts, as a precaution. For whole-work rows the growth's
  whole-work total must be at least the smaller of the environment's largest
  whole-work total and 12 x the unit's figure divided by its month keys, both
  from the aggregate's output (a unit's whole-work rows are at most its figure
  divided by its year keys, and it has at least one year key per 12 month
  keys). A unit within every step budget is not compared with F5; F4 covers
  it. A month-key count per work-month above the k that F4's mix (b) reached,
  or a whole-work total above the whole-work threshold, is a HOLD too. A
  `VERIFY` run never replaces, so the check does not apply to it. Every run
  grant also reports the database's collation and its collation provider
  beside the aggregate, and every `REBUILD` run grant reports the registry
  pair count (every pair, enabled or not) against the floor; above 13 it is a
  HOLD for an amendment. Both are read-only reads. The step budgets are bind
  parameters, bound by the grant to the bounds above:
  `$1` = 8000 (`STEP_MAX_WORK_DAY_ROWS`),
  `$2` = 1000 (`STEP_MAX_MONTH_KEYS`) and
  `$3` = 600000 (`STEP_MAX_REPLACING_READ`).
  A test compares this quotation and these values with the constants, and
  another asserts that the aggregate returns exactly the units sizing finds
  over a step budget on every fixture.

```sql
WITH series_month AS (
    SELECT r.work_id, r.platform_id, r.measure_id,
           date_trunc('month', r.day)::date AS month_start,
           count(*) AS rows
    FROM public.metric_rollup_work_day r
    GROUP BY r.work_id, r.platform_id, r.measure_id, date_trunc('month', r.day)
),
work_month AS (
    SELECT work_id, month_start,
           count(*) AS month_keys,
           SUM(rows) AS rows
    FROM series_month
    GROUP BY work_id, month_start
),
work_year AS (
    SELECT work_id, date_trunc('year', month_start)::date AS year_start,
           SUM(rows) AS rows,
           SUM(month_keys) AS month_keys,
           SUM(month_keys * rows) AS month_term
    FROM work_month
    GROUP BY work_id, date_trunc('year', month_start)
),
year_keys AS (
    SELECT work_id, date_trunc('year', month_start)::date AS year_start,
           count(DISTINCT (platform_id, measure_id)) AS year_keys
    FROM series_month
    GROUP BY work_id, date_trunc('year', month_start)
),
whole_work AS (
    SELECT work_id, SUM(rows) AS rows
    FROM work_year
    GROUP BY work_id
),
sized AS (
    SELECT y.work_id, y.year_start,
           y.rows::bigint AS work_day_rows,
           y.month_keys::bigint AS month_keys,
           (y.month_term + k.year_keys * w.rows)::bigint AS replacing_read_figure
    FROM work_year y
    JOIN year_keys k ON k.work_id = y.work_id AND k.year_start = y.year_start
    JOIN whole_work w ON w.work_id = y.work_id
),
maxima AS (
    SELECT (SELECT COALESCE(MAX(month_keys), 0) FROM work_month)::bigint
               AS max_month_keys_per_work_month,
           (SELECT COALESCE(MAX(rows), 0) FROM whole_work)::bigint
               AS max_whole_work_rows
)
SELECT s.work_id, s.year_start, s.work_day_rows, s.month_keys, s.replacing_read_figure,
       m.max_month_keys_per_work_month, m.max_whole_work_rows
FROM maxima m
LEFT JOIN sized s
  ON s.work_day_rows > $1 OR s.month_keys > $2 OR s.replacing_read_figure > $3
ORDER BY s.work_id, s.year_start
```

- **Activation window.** Activation (section 3.6) runs under its own grant,
  which names the evidence run and is its run grant. From taking the evidence
  to switching serving on, no other run of this section is started, advanced
  or abandoned, and no `verifyMetricRollupMonths` or
  `rebuildMetricRollupMonths` call is made; ad hoc section 3.7 tasks are not
  run. The invocations that advance the evidence run (a `VERIFY` run may span
  several) and read-only status reads are allowed, among them the latest-run
  check of section 3.6, made immediately before the restart that switches
  serving on.
- **Rules while a run is open.** One orchestrator per environment. No section
  3.7 rebuild. No deployment whose migrations need `ACCESS EXCLUSIVE` on the
  six tables, `metric_rollup_work_day` or `metric_rollup_work_day_state`: it
  would queue behind a page's `ACCESS SHARE` and then block readers and
  completions.
- **Pacing.** After every advance call made while the run is `BUILDING`,
  whatever its outcome (advanced, `advanced: false`, any refusal or a
  transport error), the orchestrator pauses at least `max(1 s, 2 x the elapsed
  time of that call)`, and a `RETRYABLE` back-off is never shorter. After a
  transport error or client timeout, in either phase, it pauses at least 25 s,
  because the server transaction can outlive the call; it also pauses 25 s
  before the first advance of every invocation, and its client timeout is at
  least 30 s. This keeps the state row and the step's works free at least two
  thirds of the time. Pages need no pause.

```text
REBUILD (initial population or repair):
 1. metricRollupMonthRebuildRun -> no open run (or abandon a stale one, with a reason recorded).
 2. startMetricRollupMonthRebuild(REBUILD) -> runId, stepCount 0.
 3. loop: advanceMetricRollupMonthRebuild(runId, stepCount), with pacing
      advanced=true  -> continue with run.stepCount
      advanced=false -> re-read; continue if open, stop if terminal
      RETRYABLE      -> back off (at least 1 s, and the pacing pause while BUILDING), retry the
                        same stepCount; HOLD after the bounded attempts
      BLOCKED / INTERNAL_ERROR -> stop, HOLD, alert. The run stays open at its cursor. Record the
                                  refusal, abandon the run, and escalate. A new run blocks at the
                                  same range until a separately authorized repair.
 3a. Schedule build invocations clear of the daily collection run where possible. Steps that
     meet a long ingestion batch or completion refuse RETRYABLE and are retried under pacing.
 4. Run the verify phase in a window with no rollup application, for example after the
    daily rollup run has finished and well before the next one. Then the frontier does not
    move and the run can end RECONCILED.
 5. RECONCILED -> exact at one frontier. For activation, the evidence is taken in the
                  activation window and used at once (section 3.6); a RECONCILED run is never
                  carried forward as standing evidence.
    VERIFIED   -> healthy, but not activation evidence. Repeat as a VERIFY run in a quiet window.
    MISMATCHED -> HOLD. Incident: maintenance disagreed with the verifier. Never auto-rebuild.

VERIFY (health check or activation evidence): the same loop with mode VERIFY. It writes no
projection row. Once verifyMetricRollupMonths nears its 30 s envelope, periodic health checks
move to VERIFY runs.

Stop: abandonMetricRollupMonthRebuild(runId). This is safe at any time (I3).
```

- **Repair after activation.** After activation, a section 3.8 `REBUILD` may
  repair inexact ranges while readers stay on. Each range changes atomically
  from inexact to exact, no exact range is rewritten, and readers never wait
  for a step. Readers may see repaired and unrepaired ranges side by side until
  the run ends. This records an exception, for post-activation repair only, to
  the rule that serving never sees a partially rebuilt state (issue #1012 and
  section 3.6); its "at least as correct" rests on I1, and so on the
  completion assumption.

Evidence (section 17 of the specification; raw numbers in the implementation
report, section 10.7):

- **Host.** One local evidence host, 4 CPUs and 15 GiB RAM, with CI's
  PostgreSQL major version (17): 17.10 built from the official release
  source with distribution compiler and linker flags (-O2, link-time
  optimisation, hardening, ICU, LLVM, no assertions), a cluster in UTF8 with
  the C.UTF-8 locale and the libc provider, `jit` off and every other setting
  at its default (`shared_buffers` 128 MB). Round trip 0.1 ms. Every value
  below holds for this host; production's behaviour is judged again at each
  run grant.
- **Outcome.** Every gated section 17 line was met under the budgets above,
  with no budget refusal in any gated pass: the hold and page lines of F1,
  F2, F3 and F7, F4's page line and the F4 line in each of its classes, the
  statement-group prediction at both corners, the `EXPLAIN` line, F2's
  corruption cycle, F1's exactness and every F6 line. Mix (c) and F5 are
  report only.
- **F1, F2, F3 and F7.** No pass had a budget refusal. Replacing-step hold
  p95: F2 862 ms from empty and 950 ms in its repair run (F2's and F3's
  ranges), F3 1,399 ms, F7 325 ms, F1 509 ms, against 3 s. Non-replacing-step
  hold p95: F2 102 ms, F3 163 ms, F7 49 ms, F1 87 ms, against 2 s. Verify page
  time p95: F2 1,602 ms, F3 1,661 ms, F7 457 ms, F1 1,553 ms, against 5 s.
  F2's from-empty `REBUILD` of 509,696 month keys took 2,045 s (249 keys per
  second).
- **F4 (`STEP_MAX_REPLACING_READ`).** Every F4 row carries a country and an
  institution distinct within its key-month, so each work has as many
  country and institution month rows as work-day rows. Mix (a): two works of
  8 series x 80 rows per key-month over eight years (61,440 rows each); row
  fill 0.960, figure fill 0.922; max hold / fill 2.94 s from empty and 3.27 s
  in the first repair pass. Its second history length: two works of 5 series
  x 128 rows per key-month over fourteen years (107,520 rows each); fill
  0.960; 3.47 s and 3.81 s. Mix (b): eight works of k = 39 series x 16 rows
  per key-month over one year (7,488 rows each); row fill 0.936, figure fill
  0.973; 2.55 s and 2.61 s. Every class met the F4 line (at most 4 s) with no
  refusal, and its verify pages met the page line (p95 at most 2.75 s). Every
  F4 step was a full step: hold up to 3.66 s against `STEP_HOLD_BUDGET`.
- **Statement groups and corners.** Non-negative least squares per group:
  the walk, 1.70e-4 ms per (month key x whole-work row) and 3.21e-4 ms per
  (work-month x whole-work row); the inserts, 0.169 ms per work-day row; the
  verify, 0.0353 ms per work-day row and 1.33e-3 ms per whole-work row (these
  three on the first repair pass of the F4 classes, 52 steps); the rest,
  11.5 ms plus 0.100 ms per statement and 1.05e-4 ms per whole-work row (on
  2,613 from-empty replacing steps of F2, F3, F4 and F7). At 332 statements
  (the largest measured; a step at `STEP_MAX_MONTH_KEYS` has 320), 8,000
  work-day rows and the figure full, the predicted hold in repair is 3.38 s
  at the mix (a) corner (T = 107,520 whole-work rows, 5.19 series) and 2.33 s
  at the mix (b) corner (37.5 month keys, 8,000 whole-work rows), against
  4.5 s. The six-term pooled fit (2,756 steps, information only): 2.34 ms per
  statement, 3.42e-3 ms per month-term row, 2.94e-3 ms per year-term row and
  0.0117 ms per whole-work row, nothing as intercept or per work-day row;
  absolute residuals median 43 ms, max 629 ms.
- **Floor.** F2's density is 31.5 work-day rows per key, so the floor's
  figure is 3,402 R^2 and 600,000 admits R <= 13 (574,879 at R = 13). It is
  judged against the production pair count, from the read named under
  "Floor" above.
- **Month keys per work-month.** F4's mix (b) reached k = 39, the k every run
  grant compares with.
- **Whole-work threshold.** 107,520 rows, from mix (a)'s second history
  length, which met the F4 line and the page line; F5's largest unit within
  4 s, build and repair steps alike, had a whole-work total of 84,480 rows
  (growth 3). It is above F3's
  whole-work totals (56,907 to 56,912 rows), the census maximum. It must
  also cover production's largest whole-work total, from the read named under
  "Whole-work threshold T" above.
- **F5's growths** (at F4's density, embedded in F2, each level's growth
  work first in run order; every level built, then repaired over fully built
  tables). The largest unit at which every step of the level, build and
  repair alike, committed within 4 s, per growth: (1) 39 series, one year of
  history, 27 rows per key-month: 12,636 rows, 468 month keys, figure
  985,608, whole-work 12,636 rows, holds up to 3.86 s, sizing 21 ms; (2) 39
  series, eight years, 7 rows per key-month: 3,276 rows, 468 keys, figure
  1,149,876, whole-work 26,208 rows, holds up to 3.76 s, sizing up to 66 ms;
  (3) F3's four series, eight years, 220 rows per key-month: 10,560 rows, 48
  keys, figure 380,160, whole-work 84,480 rows, holds up to 3.47 s, sizing
  up to 169 ms. The next level measured above each (28, 8 and 260 rows per
  key-month) had a step above 4 s (4.12 s in repair, 4.73 s in repair and
  4.24 s in build). This reading is stricter than section 17's, which judges
  repair steps only: at growth 3's top levels the build step was the dearer.
  On repair steps alone growth 3 reached 261 rows per key-month (12,528
  rows, figure 451,008, whole-work 100,224 rows, repair holds up to 3.89 s,
  its build steps up to 4.45 s).
- **Mix (c)** (report only). A work of one series (ten light years at 67
  rows per key-month before twelve heavy years at 625; 98,040 rows) and one
  of two series (33 before 312; 97,776 rows), measured after every gated
  pass, with no refusal. The one-series work's step of six light years
  (4,824 rows, figure 593,064) held 4.52 s from empty and 4.44 s in repair,
  with a walk of 461 to 494 ns per (month key x whole-work row), about twice
  F4's; the two-series work held at most 3.59 s. Both are inside
  `STEP_HOLD_BUDGET`.
- **F6 (concurrency).** Run on the whole of F2 with its F3 and F4 works. A
  `REBUILD` from empty under the production budgets, paced by the pacing
  rule above, with two overlapping `DAY` ingestion streams, claims and
  completions running throughout its build phase: no ingestion or completion
  failure and no deadlock; the longest wait attributable to a step was 3.5 s
  on the state row (p95 1.0 s), 1.0 s on a work row and 3.5 s with a step
  anywhere in the blocking chain, against bounds of 6, 8 and 13 s; without a
  run the same workload waited at most 0.2 s. The build phase (2,513 steps,
  5,916 s) had no refusal, so its longest run of consecutive refusals is 0,
  and the verify phase ended `RECONCILED`.
- **`EXPLAIN`.** On a median replacing range of F2, F3 and each F4 class, after
  ANALYZE, no statement read `metric_rollup_work_day` or a projection table
  by a sequential scan (every table held rows), and sizing,
  `RANGE_MONTH_KEYS_SQL`, the range deletes and the range verifier were led
  by `work_id`. No plan used JIT. About 2% of F2's month keys carried an
  ambiguity row (10,022), and all 200 scattered corruptions applied; `VERIFY`
  found each and the repair `REBUILD` reached `RECONCILED`.

Scope boundaries a consumer must not infer:

- The run row is a maintenance cursor, not a job, attempt or event: Thoth runs
  nothing asynchronously, and every step is a synchronous request from the
  orchestrator. There is no generic job framework, lease or cross-programme
  claim protocol.
- Section 3.7's pair stays the fast path for small corpora. It is not run
  while a run of this section is open. Above 100000 represented month keys, or
  once whole-table verification nears its 30 s statement, the supported path
  is this section.
- A mismatch never starts a rebuild, and Thoth never advances a run on its
  own. `VERIFIED` is never activation evidence.
- Every run fact lives in PostgreSQL. The orchestrator keeps one invocation's
  evidence in memory only, and needs no durable state of its own.
- Merging this contract activates nothing. The consumer and infrastructure
  stages, every non-disposable run, deployment and activation remain
  separately authorized gates.

## 4. Dashboard/widget

Thoth owns entity metrics, dashboard/widget operations and registry queries.

Required response concerns: distinct measure totals, timeline, breakdowns, coverage, freshness, watermark, warnings, partial state, BigInt strings and deterministic pagination.

Semantics: OR within lists, AND between dimensions, exclusive end date, bounded ranges and filtering before pagination.

### 4.1 Delivered minimum protected read contract (`MET-WP4-02`)

`MET-WP4-02` (issue #910) delivers the MOM-1 read surface: one coverage-aware
dashboard query and the two registry lists a service needs to form its UUID
filters. `MET-WP4-03B` (section 4.2) completes the dashboard contract
additively, and `MET-WP4-03C-B2` (section 4.3) extends it to a publisher's
whole history: every property below still holds except where section 4.2 or
section 4.3 states how it is extended. The SDL below already shows the
section 4.3 changes to the types it declares.

Operations, each requiring exactly `METRICS_READ_SERVICE` through
`PolicyContext::require_metrics_read_service()` before any Metrics-specific
database access:

```graphql
metricDashboard(input: MetricDashboardInput!): MetricDashboard!
metricMeasures: [MetricMeasure!]!
metricPlatforms: [MetricPlatform!]!
metricDataVersion: MetricDataVersion!
```

`metricDataVersion` (`METRIC-DATA-VERSION-01`) takes no input and returns
`version`, an opaque string that is equal for two reads when the rollup
frontier (`applied_through_sequence`, `watermark_at`) is the same, and
`rollupWatermark`. It reads one row of `metric_rollup_work_day_state`, reads no
publisher or value and uses no dashboard read slot. It is not a coverage or
freshness claim: coverage and import status are read live by `metricDashboard`
and the exceptional monthly rebuild does not move the frontier, so a consumer
that caches by this version also keeps a maximum age. The version does not
cover the native-month projection read live under month serving, `next_sequence`
(which drives `ROLLUP_LAG` and `isPartial`), metadata, selector, entitlement,
source-account-window or registry changes, or the month-serving configuration.
A consumer reads the version before the dashboard and stores that earlier
version with the cached answer.

```graphql
type MetricDataVersion {
  version: String!
  rollupWatermark: Timestamp!
}
```

```graphql
scalar BigInt

input MetricDashboardInput {
  selector: MetricSelectorInput!
  startDate: Date
  endDate: Date!
  measures: [Uuid!]
  platforms: [Uuid!]
  timelineGrain: MetricTimelineGrain = "AUTO"
}

input MetricSelectorInput {
  publisherIds: [Uuid!]
}

enum MetricTimelineGrain { AUTO DAY MONTH YEAR }

type MetricDashboard {
  totals: [MetricTotal!]!
  timeline: [MetricTimeBucket!]!
  coverage: MetricCoverage!
  asOf: Timestamp!
  dataThrough: Date
  rollupWatermark: Timestamp!
  warnings: [MetricWarning!]!
  isPartial: Boolean!
  startDate: Date!
  endDate: Date!
  timelineGrain: MetricTimelineGrain!
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
  reviewed publisher entitlement and aggregation contract of section 4.2.
- **Attribution.** Works are attributed to publishers through current
  `work -> imprint -> publisher` metadata at read time. A moved work's
  projected history follows it; nothing is rewritten.
- **Request bounds.** Half-open `[startDate, endDate)` with
  `startDate < endDate`; at most 366 days for a `DAY` timeline, and no
  maximum range for a `MONTH` or `YEAR` timeline inside the date domain of
  section 4.3; at most 10 unique measures and 10 unique platforms, whether
  explicit or resolved; at most 100 platform/measure combinations (the cross
  product of the served platforms and measures); at most 5,000 timeline cells
  (combinations x buckets). A bound is never met by truncation. Duplicate IDs
  are refused, and so is an explicit unknown measure or platform. Disabled
  registry identities stay addressable.
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
  months clipped to the range, each the exact sum of its daily rows. `YEAR`
  gives calendar years clipped to the range (section 4.3). `AUTO` chooses
  `DAY`, then `MONTH`, then `YEAR` as section 4.3 states.
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
- **Coverage.** Per platform, a represented publisher's eligible accounts
  are its enabled accounts of an enabled source whose acquisition type is
  `DRIVER` and whose `expected_publisher_id` is that publisher. There may be
  any number of them (`MET-WP4-MULTI-ACCOUNT-01`; MOM-1 refused two or more
  as `MOM1_SOURCE_SCOPE_AMBIGUOUS`). A publisher with none leaves projected
  values served and coverage `UNKNOWN`. For each account and day, the current
  assertion comes from `COMPLETED` or `COMPLETED_WITH_ERRORS` imports with a
  recorded `completed_at`: latest completion first, then greatest import UUID.
  Only if both tie within one self-contradictory import is its most
  conservative assertion preferred. `COMPLETED_WITH_ERRORS` downgrades
  `COMPLETE` to `PARTIAL`. A day without an assertion is `UNKNOWN`. Across
  accounts and publishers, a day of a served platform and measure is
  asserted only if every represented publisher has an eligible account on
  that platform and every slot has its required streams asserting the day.
  It is then the worst of those assertions, and includes a dimension only if
  every one of them
  does. Otherwise it is `UNKNOWN`. Enabling another account therefore makes
  its publisher's coverage on that platform `UNKNOWN` for every day the new
  account has not asserted, until its history is collected; values do not
  change. A measure that one of a publisher's accounts on a platform never
  asserts stays `UNKNOWN` there. An item is
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
- **Several accounts on one platform.** The canonical identity of section 3
  has no source account, so the first account to report a cell holds it.
  Another account's observation of the same work, publication, platform,
  measure, period, country and institution is never added. It is `DUPLICATE`
  when its content is identical, which is silent, and, when the content
  differs, `CONFLICT`, which ends its import `COMPLETED_WITH_ERRORS` and so
  shows its days as `PARTIAL`. Through quarantine reconciliation the same
  collision is `RESOLVED_DUPLICATE`, which is silent, or
  `BLOCKED_SOURCE_CONFLICT`, which leaves an unresolved-identifier warning
  that does not clear. Accounts that report one work, platform, measure and
  day in different dimensional shapes do not collide, but the
  dimensional-representation rule above then drops one account's values or
  refuses the request. Several eligible accounts of one publisher and
  platform are therefore known to give correct values when no two of them
  report the same work, platform, measure and day; in practice, each work's
  files are on one site. Operators check this before enabling another
  account. The read cannot tell a lost value from no traffic. Only a
  succession (section 2.2) lets accounts stand in for each other; a window
  alone never does.
- **Native months (`MET-WP4-MONTH-01`).** A server with monthly serving `ON`
  (`THOTH_METRIC_MONTH_SERVING`, default `OFF`, read once at process start)
  serves `MONTH` records; with it `OFF` they refuse as section 4.2 says. A
  month is never split into days: for a cell `[from, to)`, a served month
  `[M, M + 1 month)` adds its value only if it lies wholly inside the cell. A
  month that a cell cuts adds nothing to that cell, and its days are treated
  like `PARTIAL` days: zero is not justified there, the pair's status is at
  worst `PARTIAL`, `PARTIAL_COVERAGE` is raised and `dataThrough` stops
  before them. A served month is cut when it is conflicted, or unless it lies
  wholly inside the range and wholly inside the bucket that contains its
  first day: on a `DAY` timeline every served month is cut, and on `MONTH`
  and `YEAR` timelines exactly the conflicted months and those the range
  cuts. Zero day rows of a cut month still show `"0"`, as `PARTIAL`. For one
  work, platform, measure and month, the native side is live when any served
  month record of it has a value other than 0, and the day side is live when
  any 03A monthly total, country or institution row of that key has a value
  other than 0, or a 03A ambiguity row exists for it. When both are live the
  cell is conflicted: the days are served as they would be without the month,
  the month adds no value, section row, dependency flag or ambiguity, and it
  is cut for that pair in every cell, so its days are never `COMPLETE`.
  Nothing is refused. When every 03A row of the key is 0, the two sides add
  with nothing counted twice, whatever the measure's sign. A conflict is
  decided in each read and stored nowhere; a repaired month (one side zeroed,
  section 2.2) stays cut wherever a cell cuts it. `AUTO` is resolved first, so
  a short `AUTO` request resolves to `DAY` and shows its monthly pairs as
  `null` buckets and `PARTIAL`. A native month is read from its projection in
  the read's own snapshot: it is as of `asOf`, raises no `ROLLUP_LAG`, and
  `rollupWatermark` says nothing about it.
- **Account windows and successions (`MET-WP4-MONTH-01`, under either switch
  value).** An eligible account's window `[windowStart, windowEnd)` (section
  2.2) limits where its runs and terminal-import coverage count: each run is
  clipped to the window and dropped if empty, and only then is same-stream
  overlap refused, so an overlap outside a window refuses nothing. Per
  publisher and platform, each succession is one slot and each account
  without one is a slot of its own. On a day `d` a platform requires, per
  represented publisher, one stream if the publisher has no eligible account
  on it, and otherwise, per slot, `max(1, members whose window contains d)`.
  Outside every member's window a slot still requires one stream that no
  clipped run supplies, so the day stays `UNKNOWN`. With no window and no
  succession this is the rule above. The represented scope and the all-time
  start count a run or a terminal-import coverage row only where it
  intersects its account's window, from the later of its start and the
  window start. A window never hides a value: values carry no account.
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
  `METRIC_REGISTRY_LIMIT_EXCEEDED` and `MOM1_DIMENSION_SCOPE_AMBIGUOUS`, plus
  `METRIC_QUERY_UNSUPPORTED_SOURCE_GRAIN` (section 4.2) and the retryable
  `METRIC_READ_BUSY` (section 4.3). An
  aggregate outside the representable range is `METRIC_QUERY_LIMIT_EXCEEDED`.
  An unexpected database failure is `INTERNAL_ERROR` without detail. No SQL,
  database diagnostic, token or source configuration is returned.
  `MOM1_SOURCE_SCOPE_AMBIGUOUS` is retired by `MET-WP4-MULTI-ACCOUNT-01` and
  is never returned. A client may keep handling it.

Scope boundaries a consumer must not infer:

- Under `ON`, `MET-WP4-MONTH-01` serves requests that a `MONTH` record
  overlaps, which `OFF` refuses, and judges each pair of an all-time read from
  its own start. With no `MONTH` record and no account window or succession,
  every other response is unchanged apart from `asOf`.
- Countries, institutions, works sections, pagination, the deferred selector
  dimensions (`imprintIds`, `seriesIds`, `workIds`, `dois`, `workTypes`,
  `languages`, funding and affiliation institutions, `includeDescendants`),
  `metricWidget` and entity-level Metrics fields were not part of the
  `MET-WP4-02` contract. `MET-WP4-03B` adds the country and institution
  sections and every one of those selector dimensions except `dois` and
  `includeDescendants` (section 4.2), and `MET-WP4-WIDGET-01` adds the
  public `metricWidget` read as a separate contract (section 4.4); the rest
  still needs its own specification.
- The read calls no external service. Browser clients must not hold the
  `METRICS_READ_SERVICE` credential; they reach this protected read through a
  Thoth-owned server route. The public `metricWidget` read, which browsers
  call directly with no credential, is a separate contract (section 4.4).
- The read contract remains additive. No existing field, argument, nullability
  or authorization changed; `MET-WP7-PREREQ-04` adds only the
  `UNRESOLVED_IDENTIFIERS` value to `MetricWarningCode` and reconciles the
  documented zero/partial semantics above. `MetricCoverageStatus` is the
  existing database enum, exposed to GraphQL without any value change.
  Generated/exhaustive clients must tolerate the new warning enum before
  CloudFront production activation, and consumers must not coerce `null`
  Metrics values to zero. `MET-WP4-MULTI-ACCOUNT-01` no longer refuses a
  request because a publisher has several eligible accounts on a platform;
  such a request is served or meets another refusal. Every request served
  before gets the same response.

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
  includeChapters: Boolean
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

- **Publishers.** `publisherIds` holds one to 100 unique IDs. None, a
  duplicate or an unknown publisher is `METRIC_QUERY_INVALID`; more than 100
  is `METRIC_QUERY_LIMIT_EXCEEDED`. Every selected publisher must
  independently hold `METRICS_DASHBOARD`; one that does not makes the whole
  request `NO_ACCESS`, even when the selector would exclude its works. There
  is no entitled-subset response, no `SUPERUSER` shortcut and no new role:
  the resolver still requires exactly `METRICS_READ_SERVICE` first.
- **Selector.** Values within one list are alternatives (OR); different
  lists must all match (AND); an omitted or empty optional list does not
  restrict. Works are resolved from current metadata, before any Metrics
  value is read: current `work -> imprint -> publisher` ownership; imprint;
  explicit work; work type; a series by direct issue membership or, for a
  `book-chapter`, through a current `is-child-of` parent issued in it; any
  language record of the work whatever its relation; any funding by a
  selected institution; any contribution affiliated with a selected
  institution. No metadata copy is stored for Metrics.
- **`includeChapters` (MET-WP10-BOOKS-01A).** Optional, default false. When true, a
  listed `workIds` entry also selects every `book-chapter` whose current
  `is-child-of` parent it is; the chapter still has to belong to a selected
  publisher and satisfy every other list. Without `workIds` it changes
  nothing. `workIds` still counts listed books only; chapters count toward the
  25,000 resolved-works bound.
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
  than 25,000 works is `METRIC_QUERY_LIMIT_EXCEEDED`. No bound is met by
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
  institutions add both parts. A `DAY` timeline reads every day from the
  work-day projection; a `MONTH` timeline takes a complete month's bucket
  from the monthly projection and a clipped bucket from its days, and a
  `YEAR` timeline regroups the same reads by calendar year. `AUTO` and the
  yearly section reads are as section 4.3 states. The values equal the
  section 4.1 day-by-day values for the same resolved request, which the
  tests prove against an independent oracle.
- **Countries and institutions.** Only known values are listed: no synthetic
  unknown country or institution, and no zero for an absent one. Country
  values use the target-dimension rule of section 3.6 (the unique least
  country-bearing representation); `countryCode` is the Metrics uppercase
  ISO 3166-1 alpha-2 code, not Thoth's alpha-3 `CountryCode`. Institution
  values use the mirror rule, with the name and nullable ROR of the current
  Thoth institution read set-wise in the same snapshot. Countries are ordered
  by `(platformId, measureId, countryCode)` and institutions by
  `(platformId, measureId, institutionId)`, never by display text. At most
  100,000 institution rows are returned; more is
  `METRIC_QUERY_LIMIT_EXCEEDED`, never a truncated list. Each of the at most
  two statements that read institution rows stops after 100,001 grouped rows,
  and the request fails when the distinct institution rows across both exceed
  100,000. Country rows are bounded by the combinations and the alpha-2 code
  space.
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
  day, coverage is first combined across every eligible account of every
  represented publisher (section 4.1): a represented publisher without an
  eligible account on the platform, or an eligible account without an
  assertion, makes the day `UNKNOWN`; otherwise the worst status wins
  (`UNKNOWN`, then `PARTIAL`, then `COMPLETE`, with
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
  facts. With monthly serving `ON` (`MET-WP4-MONTH-01`), the days of a served
  month that is cut or conflicted (section 4.1) are added, clipped to the
  range, to each served combination's unjustified days and to its first
  break, for the totals, the timeline and every returned section, and a
  combination with such a day in a segment whose shared status is `COMPLETE`
  is `PARTIAL`. Nothing else changes, so an `UNKNOWN` day stays `UNKNOWN`.
- **Native grains.** With monthly serving `OFF` (`MET-WP4-MONTH-01`, section
  4.1), as follows. The projections serve `DAY` canonical records only. If
  a canonical record of `MONTH` or `REPORTING_PERIOD` grain for a resolved
  work, served platform and served measure overlaps `[startDate, endDate)`
  (with `startDate` omitted, starts before `endDate`) and its
  `current_revision_id` names a `CURRENT` revision, the whole request fails
  with `METRIC_QUERY_UNSUPPORTED_SOURCE_GRAIN` and a fixed message;
  nothing is silently omitted. A record whose pointer names a `RETRACTED`
  revision, with no `CURRENT` revision, is withdrawn and does not fail it.
  Contradictory committed state — no pointer, a pointer to a `SUPERSEDED`
  revision, or a `RETRACTED` pointer beside a `CURRENT` revision — fails
  closed as `INTERNAL_ERROR`. The check's predicate matches the partial
  index `metric_record_native_grain_idx`, which changes only its cost.
  With monthly serving `ON`, `MONTH` records are served from the native-month
  projection (section 3.6), which alone decides a value and a refusal, and no
  read touches a non-`DAY` record: an active `REPORTING_PERIOD` record refuses
  with the same code and fixed message, a projected inconsistency of either
  grain (a `MONTH` record of any other period included) is `INTERNAL_ERROR`,
  and a month/day conflict refuses nothing (section 4.1).
- **Descriptions (`MET-WP4-MONTH-01`)** are static and hold under both switch
  values. The last sentence of `metricDashboard`'s is "A request intersecting
  Metrics reported for a reporting period fails with
  METRIC_QUERY_UNSUPPORTED_SOURCE_GRAIN, and so does one intersecting Metrics
  reported for a whole month unless this server serves monthly Metrics; a
  served month counts only in a value whose dates contain all of it, and where
  days of that month are also reported the days are served and the month is
  marked incomplete". `MetricCoverageItem` and its `status`, `dataThrough`,
  `countryCoverage` and `institutionCoverage`, and the widget's
  `MetricWidgetPairCoverage` `status`, `dataThrough` and `countryCoverage` and
  `MetricWidgetSection.isPartial`, end with "With startDate omitted on a
  server that serves monthly Metrics, each platform and measure is judged from
  its own first evidence, not from the served startDate".
- **Lag scope.** `ROLLUP_LAG` and the lag part of `dataThrough` count only
  unapplied work-day deltas above the frontier for the resolved works,
  range, served platforms and served measures. Identifier quality does not
  widen it.
- **Execution.** Everything still runs on one connection in one
  `READ ONLY, REPEATABLE READ` transaction. Before its first query, and
  therefore before its snapshot is frozen, the read executes exactly three
  non-query statements in this order: `SET LOCAL plan_cache_mode =
  force_custom_plan`, so each statement is planned for its actual arrays and
  bounds; `SET LOCAL statement_timeout = '30s'`, a limit on each later
  statement individually (the whole-read deadline of section 4.3 is
  separate); and one `LOCK TABLE` on exactly the six derived projection
  tables the exceptional rebuild truncates — `metric_rollup_work_month`,
  `metric_rollup_work_country_month`,
  `metric_rollup_work_institution_month`, `metric_rollup_work_month_ambiguity`,
  `metric_rollup_work_country_year` and `metric_rollup_work_institution_year`,
  in the order the verifier of section 3.7 locks them — `IN ACCESS SHARE
  MODE`, held until the transaction ends (`MET-WP11-B-CORR-H1`, issue #987).
  The longest request path is exactly eighteen database statements counted
  conservatively — the pool's own checkout `SELECT 1`, `BEGIN`, the planner
  setting, the statement timeout, the lock, twelve queries including the
  identifier-quality query, and `COMMIT` — and seventeen excluding the pool's
  checkout, whatever the number of publishers, works, days, months, countries,
  institutions or quarantine rows: the frontier is read with the publishers,
  explicit selector IDs are checked while the works are resolved, native
  grains, lag and the all-time start share one statement, each monthly and
  edge-day section statement runs at most once, and no work or institution is
  looked up individually. Section 4.3 gives the ceiling of each request form.
  Coverage is read as eligible coverage runs clipped to the served range: no
  statement expands the days of the range.
  With monthly serving `ON` (`MET-WP4-MONTH-01`), step 3b, `NATIVE_MONTHS_SQL`,
  runs once per read on the same snapshot, after the works are known and
  before platforms and measures are resolved, whenever the resolved works are
  non-empty. It reads, through their primary keys, the `metric_native_month`
  and `metric_native_refusal` rows of the resolved works and of the explicit
  platform and measure filters (every one when a filter is not given) whose
  period overlaps the range. It settles conflicts in one set-based pass over
  the 03A monthly total and ambiguity rows of the works that hold a live cell,
  reading the country and institution rows only for cells whose total rows are
  all 0 and that have no ambiguity row, and returns rows per platform and
  measure, never per work. Every pair with a served month joins the
  represented platforms and measures, so a monthly-only pair is served, and
  native rows are read before the combination and cell limits are checked. A
  boolean parameter folds `CANONICAL_SQL`'s native-grain branch away; in its
  place, in the same order, a projected inconsistency on a served platform
  and measure fails as `INTERNAL_ERROR`, then an active reporting period as
  `METRIC_QUERY_UNSUPPORTED_SOURCE_GRAIN`. Then come the 03A monthly
  ambiguity, the native ambiguity (`MOM1_DIMENSION_SCOPE_AMBIGUOUS`), and the
  day and dimension-cell ambiguity. `PAIR_STARTS_SQL` runs after the
  canonical statement when `startDate` is omitted and two or more pairs are
  served (section 4.3). The longest request path is then twenty statements,
  nineteen excluding the pool's checkout. With it `OFF`, neither statement
  runs.
- **Snapshot protection and contention.** PostgreSQL's `TRUNCATE` is not
  MVCC-safe: a `REPEATABLE READ` snapshot frozen before a concurrent
  `rebuildMetricRollupMonths` truncate would see the projection tables it had
  not yet read as empty, and complete coverage would turn that emptiness into
  a successful, false zero. The pre-snapshot `ACCESS SHARE` lock closes that
  anomaly exactly as the verifier's does. It conflicts only with `ACCESS
  EXCLUSIVE`, which the rebuild's `TRUNCATE` takes, and not with the `ROW
  EXCLUSIVE` locks of normal incremental completion, so routine monthly and
  yearly maintenance is never blocked by a dashboard read. Reader first: a
  rebuild cannot truncate while a read holds the six locks; it either acquires
  its lock after the read ends and proceeds under its own existing limits, or
  expires its own existing 5-second `lock_timeout`, fails closed and rolls
  back. Rebuilder first: the read waits at its `LOCK TABLE`, before
  establishing its snapshot, until the rebuild commits or rolls back, subject
  to its own 30-second statement timeout. After a successful rebuild commit it
  sees the rebuilt committed state; after rollback it sees the previously
  committed state. A statement timeout fails the read closed. A read arriving
  while a `TRUNCATE` is queued waits behind it under PostgreSQL's lock
  ordering, bounded the same way. No wait is indefinite, and no fairness or
  whole-request latency is promised.
- **Bounded failure.** A statement timeout, a lock wait that reaches it, a
  read past its whole-read deadline (section 4.3), or any other SQL failure
  aborts the whole read, rolls its transaction back, returns the connection
  to the pool and fails as `INTERNAL_ERROR` with the fixed message `The
  Metrics read could not be completed.`; no partial response and no
  fabricated zero is ever returned.
- **Step 0 under `MET-WP4-03C-B2`.** Specification Amendment 3 section 4A
  (issue #980) reconciles the `MET-WP4-03C-B2` reader with this sequence: the
  planner setting, the statement timeout and the lock stay three separately
  named statements, in this order, before the first query, with nothing
  before them in the transaction. No combined settings statement exists, and
  B2 adds no statement name; its form ceilings are in section 4.3.
- **Activation gate.** The monthly projections are served only once, under
  separate authorization, the `MET-WP4-03A` migration has run and the derived
  state has been shown exact at a recorded work-day frontier, either by an
  exact `verifyMetricRollupMonths` or by a section 3.8 run of either mode that
  reached `RECONCILED`, with the evidence taken in the activation window and
  used at once, as section 3.6 states (the latest-run check, and no other run
  or section 3.7 call between the evidence and the switch). `VERIFIED` is
  never sufficient. After activation, a section 3.8 `REBUILD` may repair
  inexact ranges while readers stay on (section 3.8). Merging this contract
  activates nothing.

Scope boundaries a consumer must not infer:

- `dois`, `includeDescendants`, works sections and pagination and
  entity-level Metrics fields remain deferred. `metricWidget` is not part of
  this protected contract; it is the public widget read of section 4.4.
- With monthly serving `OFF`, no native `MONTH` or `REPORTING_PERIOD`
  serving is implemented; such records only fail an intersecting request.
  With it `ON` (`MET-WP4-MONTH-01`), `MONTH` records are served as section
  4.1 says and `REPORTING_PERIOD` records still fail an intersecting request.
- The read writes nothing: no projection, monthly row, delta, frontier or
  coverage is repaired or recorded, and no external service is called.

### 4.3 All-time history (`MET-WP4-03C-B2`)

`MET-WP4-03C-B2` (issue #980, through Specification Amendments 1, 2 and 3)
lets a dashboard show a publisher's whole Metrics history. `MONTH` and
`YEAR` timelines have no calendar cap, `startDate` may be omitted, and
coverage is evaluated over coverage runs instead of individual days. Every
property of sections 4.1 and 4.2 still holds except where this section says
how it changes. Each range-relative rule there applies to the served range
`[startDate, endDate)`, whose `startDate` the response returns. No
operation, role, capability, migration, table or index is added, and
`metricMeasures` and `metricPlatforms` are unchanged.

Additive GraphQL contract (already shown in the section 4.1 SDL):

```graphql
input MetricDashboardInput {
  startDate: Date   # was Date!; absent or null means all time
  # ...every other field unchanged...
}

enum MetricTimelineGrain { AUTO DAY MONTH YEAR }

type MetricDashboard {
  # ...section 4.1 and 4.2 fields...
  startDate: Date!
  endDate: Date!
  timelineGrain: MetricTimelineGrain!
}
```

`MetricTimelineGrain` is now also an output type, so strict clients must
accept `YEAR`. The one new error classification is `METRIC_READ_BUSY`.

Contract properties consumers may rely on:

- **Envelope.** A `DAY` timeline covers at most 366 days. `MONTH` and `YEAR`
  timelines have no maximum range. Every timeline must fit the 5,000-cell
  bound, and an explicit grain never switches. A request selects 1 to 100
  publishers and resolves to at most 25,000 works, and a response holds at
  most 100,000 institution rows (section 4.2). The other bounds of sections
  4.1 and 4.2 are unchanged.
- **Date domain.** Only when `startDate` is omitted or the range is longer
  than 366 days: an explicit `startDate` must be on or after 2000-01-01;
  `endDate` may be at most 366 days after the API process's current UTC date;
  with `startDate` omitted, `endDate` must be later than 2000-01-01. A request
  outside the domain fails as `METRIC_QUERY_LIMIT_EXCEEDED` with a fixed
  message, before any database work. An explicit range of at most 366 days
  is served whatever its dates, as before.
- **AUTO.** `AUTO` is decided once the served platforms, measures and start
  are known: `DAY` when the served range is at most 366 days and its timeline
  fits, otherwise `MONTH` when its timeline fits, otherwise `YEAR`, which
  always fits inside the date domain. `timelineGrain` in the response is the
  grain served, never `AUTO`; `startDate` and `endDate` are the served range.
- **YEAR.** One bucket per calendar year, clipped to the served range. A
  `YEAR` timeline is a `MONTH` timeline for every purpose but bucketing: the
  same statements, month split, day reads and section composition. A bucket
  follows the section 4.1 value rule over its days: the exact sum when any
  projected row exists; `"0"` only when every day is effectively `COMPLETE`,
  with no unapplied delta and no unresolved identifier evidence; otherwise
  `null`. It is therefore not the sum of `MONTH` values when any of them is
  `null`. Totals and the timeline never read a yearly projection.
- **Yearly sections.** For a `MONTH` or `YEAR` timeline, the country and
  institution sections read the complete calendar years inside the range
  from `metric_rollup_work_country_year` and
  `metric_rollup_work_institution_year`, the remaining complete months from
  the monthly section projections and the clipped edge days from the
  work-day projection, in the one complete-month statement. The three parts
  are disjoint, and monthly ambiguity stays authoritative over a year read
  from the yearly projections.
- **All time.** An absent and a null `startDate` are the same. With
  `startDate` omitted, the read finds, in the same snapshot, the earliest
  evidence before `endDate` for the resolved works, the represented
  publishers and the served platforms and measures: a work-day projection
  row, an eligible coverage run, coverage from a terminal import without a
  completion time, unresolved identifier quarantine and an unapplied work-day
  delta. The served `startDate` is that exact day, and the first bucket is
  clipped there. With no evidence, no resolved work or no served pair it is
  the first day of the month containing the day before `endDate`. Evidence
  dated before 2000-01-01 fails the request as `METRIC_QUERY_LIMIT_EXCEEDED`;
  it is never clamped. Omitted platforms and measures resolve over the whole
  history before `endDate`. Every fail-closed rule applies to all that
  history: an active or inconsistent native-grain record anywhere before
  `endDate`, and an ambiguous month or day or same-stream run overlap in the
  served range. `startDate` cannot be omitted with a `DAY`
  timeline (`METRIC_QUERY_INVALID`). A successful all-time response equals
  the explicit request over the served range at the served grain.
  Under either monthly serving value (`MET-WP4-MONTH-01`), an eligible
  coverage run or terminal-import coverage row is evidence only where it
  intersects its account's window, from the later of its start and the
  window start. With monthly serving `ON`, the evidence also includes the
  first day of the earliest served month of a served pair before `endDate`;
  the first fail-closed rule is a projected refusal (an active reporting
  period, or an inconsistency of either grain) anywhere before `endDate`;
  native ambiguity applies to all history before `endDate`; a read serving
  two or more pairs whose earliest pair start (below) is not its served start
  fails closed as `INTERNAL_ERROR`; and the equality above holds apart from
  the coverage fields of the pair-start rule: an item's `status`,
  `dataThrough`, `countryCoverage` and `institutionCoverage`, and the
  top-level `status`, `dataThrough`, `warnings` and `isPartial`.
- **Filtered start.** With explicit platforms or measures, the start is the
  earliest evidence of the selected pairs. It is found from the monthly
  identity keys and at most one month of work-day rows per work, without
  reading a monthly value or scanning a monthly projection. Evidence of a
  platform or measure that is not selected never moves it.
- **All-time coverage.** With monthly serving `OFF`: an all-time response
  reports the worst coverage over its whole served history. With several
  platforms or measures, an item whose evidence starts after the served
  `startDate` is `UNKNOWN` for its leading days, its `dataThrough` is `null`,
  and so is the top-level `dataThrough`. Clients must not present
  `dataThrough` as "data up to" for all-time views.
  With it `ON` (`MET-WP4-MONTH-01`), an all-time read judges each pair from
  its own start. The served `startDate`, the buckets and every value are as
  above, but a pair's `status`, `dataThrough` and two coverage flags are
  derived only from the days on or after its start `S`: the earlier of the
  pair's own earliest evidence (the evidence above, per platform and measure)
  and its earliest served month, and never before the served start. An item
  without evidence is judged from the served start. Zero is justified as
  before, over the whole range: a bucket before `S` stays `null` unless
  coverage asserts it. So the rule adds no `"0"` and changes no value, and
  each item equals the item of the same all-time read restricted to its
  pair, at the same grain. An item's `dataThrough` is `null` when its first
  break is on or before `S`; the top-level `dataThrough` is the earliest of
  the items', and `null` unless every item has one. The response carries no
  pair start: a pair can be `COMPLETE` with a `dataThrough` under a served
  `startDate` years before its own start, and the bucket that holds its start
  is a part-period value with no mark. Clients must not present
  `dataThrough` as "data up to" for all-time views, and must not present a
  status or `dataThrough` as holding from `startDate`. An explicit
  `startDate` changes nothing.
- **Coverage runs.** Coverage is read from the eligible
  `metric_coverage_run` runs, clipped to the served range, with the
  semantics of sections 4.1 and 4.2 unchanged. Eligibility is checked at
  request time against the current source account, as before. Two eligible
  runs of one stream (one account, platform, publisher and measure) that
  cover the same served day are invalid derived state and fail the request
  as `INTERNAL_ERROR`; overlap outside the served range is never read. Runs
  of different eligible accounts may cover the same day; section 4.1
  combines them. Evaluation is interval-based: apart from the
  timeline buckets themselves, nothing the read builds, in SQL or in memory,
  grows with the number of days in the range.
- **Represented pairs.** Omitted platforms or measures resolve from the
  section 4.2 evidence, with ordinary coverage represented by eligible
  coverage runs and terminal imports without a completion time read
  directly. Its monthly branches read only the identity-key columns of the
  monthly projections, never a monthly value.
- **Complete-month strategy.** From 1,200 resolved works the complete-month
  statement reads its monthly total, monthly ambiguity and yearly section
  branches in one filtered pass instead of one probe per work, and from
  1,600 its monthly section branches too. This is a physical read strategy,
  not a product limit: responses are identical on both sides of each
  threshold.
- **Concurrency.** One API process runs at most three `metricDashboard`
  reads at once. A further read waits at most one second in total for a
  slot, holding no database connection, and otherwise fails whole as the
  retryable `METRIC_READ_BUSY` with the fixed message "The Metrics service
  is busy. Try again shortly." It is a resolver error (HTTP 200 with an
  `errors` entry), and clients retry on the code. A request refused by
  validation takes no slot, and a slot is given back on success, error,
  rollback and panic alike. The limit is per process: database-wide
  concurrency is three times the number of running API processes. A read
  that would have waited or run on before can now fail with
  `METRIC_READ_BUSY`, or as `INTERNAL_ERROR` past the deadline below.
- **Whole-read deadline.** A read must start every statement from `BEGIN`
  to the identifier-quality query within 20 seconds of being granted its
  slot. Otherwise it rolls back and fails as `INTERNAL_ERROR` with the fixed
  message of section 4.2; `COMMIT` is not checked. The pool checkout waits at
  most the lesser of the pool's connection timeout and the time left, and a
  checkout the deadline cuts short is a deadline expiry. A statement already
  started still runs under the 30-second statement timeout, so a read holds
  its slot for at most about 50 seconds. A busy refusal and a deadline expiry
  are logged with fixed text and no request detail.
- **Rebuild under read load.** A `rebuildMetricRollupMonths` that has to
  truncate waits at most its own 5-second `lock_timeout` for its
  `ACCESS EXCLUSIVE` lock. A dashboard read holds `ACCESS SHARE` on the six
  projection tables from step 0 until its transaction ends: it starts no
  statement after its 20-second deadline, and each statement runs under the
  30-second statement timeout, so one read holds the locks for at most about
  50 seconds. Reads that arrive while the rebuild waits queue behind it. If
  the rebuild fails on its lock timeout, it has rolled back and changed
  nothing. Retry it when dashboard read traffic is low, so that no running
  read outlasts that 5-second wait.
- **Statement ceilings.** Step 0 is the section 4.2 sequence, unchanged.
  Every statement on the read connection counts, the pool's checkout
  `SELECT 1` included, and a failed read is not a ceiling example. With
  exactly one of the known-pair and represented-pair queries, the same for an
  explicit and an omitted start, and with the forms as Specification
  Amendment 3 section 4A (issue #980) defines them:

  ```text
  form                                       OFF      ON
  aligned MONTH or YEAR                      <= 14    <= 15
  aligned DAY                                <= 15    <= 16
  aligned DAY with a base cell to resolve    <= 16    <= 17
  clipped / omitted-section                  <= 16    <= 17
  clipped MONTH or YEAR with sections        <= 17    <= 18
  clipped DAY with sections                  <= 17    <= 18
  ```

  The `ON` column is monthly serving `ON` (`MET-WP4-MONTH-01`): every form
  gains one statement, `NATIVE_MONTHS_SQL`, and an all-time request serving
  two or more pairs one more, `PAIR_STARTS_SQL`. The longest path under
  `ON` is twenty statements, nineteen excluding the checkout: the eighteen
  below, native months after works, and pair starts after canonical state.

  The two DAY rows at 16 and 17 run the same day reads as the other form
  with the same ceiling.
  "Aligned" means both dates are the first of a month. A request in which
  both pair queries run carries exactly one more statement, and no form may
  exceed its own ceiling. The longest path is exactly the eighteen statements
  of section 4.2: checkout, `BEGIN`, the three step-0 statements, publishers,
  works, known pairs, represented pairs, accounts, canonical state, complete
  months, day sums, dimension cells, edge-day sections, coverage runs,
  identifier quality and `COMMIT`.
- **All-time default gate.** A client may make the all-time view its
  default, or offer a control that omits `startDate`, only after this
  reader is activated with its representative performance references met;
  after either a response cache is deployed or evidence shows that the
  uncached default meets those references at three concurrent reads per
  process, with the busy rate accepted; and after a read-only production
  census shows which entitled publishers' default request would be refused,
  and why, with that result accepted or a fallback specified.
- **Rollback.** A reader rolled back to a binary before `MET-WP4-03C-B2`
  fails every request that omits `startDate`, sends `YEAR` or selects the new
  output fields at GraphQL validation. A client using them must therefore be
  revertible on its own, and is switched off first: client, then reader,
  then service health verification, then any producer rollback.

Scope boundaries a consumer must not infer:

- No response cache is added; a cache is separate work.
- History is only what has been ingested; importing earlier history is
  separate work.
- `metricWidget` does not use this concurrency limit or this omitted-start
  rule; its own all-history read is in section 4.4.

### 4.4 Public widget read contract (`MET-WP4-WIDGET-01`)

`MET-WP4-WIDGET-01` (issue #1007, specification R11, under
[`ADR-0014`](../engineering/decisions/ADR-0014-public-widget-metrics.md))
adds one public, read-only query for the embeddable `metrics-widget`
package: one work (a book or a chapter) and its child works, by DOI, with a
monthly or yearly timeline, for publishers whose package grants
`METRICS_WIDGET`, serving only measures the registry marks public, with the
per-pair value and coverage semantics of sections 4.1-4.3 over its own
month-aligned range. `MET-WP4-WIDGET-ALLTIME` (issue #1019, specification
R2.1) adds the widget-owned `YEAR` timeline and lets a `YEAR` read omit
`startDate` to serve all history, from one served start shared by both
sections. The widget serves `YEAR` and an omitted start under its own rules
(below), and none of section 4.3's concurrency limit, 20-second deadline (the
widget has its own 10-second deadline on the same engine mechanism) or
`METRIC_READ_BUSY`. No role, migration, table or index is added, and
`metricDashboard`, `metricMeasures`, `metricPlatforms` and every other
operation keep their schema, results and authorization. Merging it activates
nothing: the read is off until activation (below).

Additive GraphQL contract:

```graphql
# QueryRoot. No role required. Fails closed when the server switch is off.
metricWidget(input: MetricWidgetInput!): MetricWidget!

input MetricWidgetInput {
  doi: Doi!                 # the embedded work (book or chapter)
  chapterDois: [Doi!]       # optional subset of the work's child works; omitted or [] = all
  startDate: Date           # first day of a month; may be omitted only for YEAR (all served history)
  endDate: Date!            # first day of a month, exclusive
  timelineGrain: MetricWidgetGrain = "MONTH"
}

enum MetricWidgetGrain {    # widget-owned; not MetricTimelineGrain
  MONTH
  YEAR
}

type MetricWidget {
  work: MetricWidgetSection!
  chapters: MetricWidgetSection
  excludedChildCount: Int!
  platforms: [MetricWidgetPlatform!]!
  measures: [MetricWidgetMeasure!]!
  startDate: Date!          # the served start
  endDate: Date!
  asOf: Timestamp!
  rollupWatermark: Timestamp!
}

type MetricWidgetSection {
  totals: [MetricTotal!]!
  timeline: [MetricTimeBucket!]!
  countries: [MetricCountryTotal!]!
  coverage: [MetricWidgetPairCoverage!]!
  dataThrough: Date
  isPartial: Boolean!
}

type MetricWidgetPairCoverage {
  platformId: Uuid!
  measureId: Uuid!
  status: MetricCoverageStatus!
  dataThrough: Date
  countryCoverage: Boolean!
}

type MetricWidgetPlatform { platformId: Uuid! code: String! displayName: String! publicDescription: String }
type MetricWidgetMeasure  { measureId: Uuid! code: String! displayName: String! unit: MetricMeasureUnit! definition: String! }
```

`MetricTotal`, `MetricTimeBucket`, `MetricCountryTotal`,
`MetricCoverageStatus` and `BigInt` are the section 4.1 and 4.2 types,
unchanged. The grain is the widget's own `MetricWidgetGrain`, never the
dashboard's `MetricTimelineGrain`: `MONTH` (one bucket per calendar month of
the served range) or `YEAR` (one bucket per calendar year, clipped to the
served range, each the exact sum of the daily values it contains, under the
section 4.3 `YEAR` rule). An omitted and a null `timelineGrain` are both
`MONTH`, and an omitted and a null `startDate` are the same. No warnings and
no institution fields are served. The change from the first widget contract
is additive: `startDate` became nullable, and `timelineGrain` and
`MetricWidgetGrain` are new; no output field changed type.

- **Access.** Public and capability-gated: no credential and no role; the
  caller's identity is never consulted. The read is served only while the
  server switch `THOTH_METRIC_WIDGET` (`OFF` | `ON`, default `OFF`, read once
  at process start, registered on `start graphql-api` and `init`) is `ON`,
  and only for a work whose current imprint's publisher's package grants
  `METRICS_WIDGET`. The engine re-checks that capability in its own
  entitlement statement.
- **DOIs.** `doi` and every `chapterDois` entry are parsed as DOIs, so
  `10.x/y`, `doi.org/10.x/y` and `http(s)://(www.)(dx.)doi.org/10.x/y` are
  accepted, case-insensitively; every comparison and the lookup use the
  canonical form lower-cased. A string that does not parse is
  `METRIC_QUERY_INVALID` (with the switch on); it is not a request-level
  error.
- **Order of operations.** (1) GraphQL coercion; a request-level error is
  HTTP 400 before the resolver. (2) Switch `OFF`: `METRIC_WIDGET_UNAVAILABLE`,
  with no slot, connection or statement. (3) One resolution per HTTP request,
  counted on the request context, after fragment expansion: a later
  `metricWidget` selection of the same request is
  `METRIC_QUERY_LIMIT_EXCEEDED` with no slot or statement. (4) Input-only
  validation, with no database, in this order, once the grain is resolved:
  `doi` parses; a `MONTH` read gives `startDate`; then the dates. With
  `startDate`, for either grain: both dates are first days of months and
  `startDate < endDate`; `startDate` is on or after 2000-01-01 and `endDate`
  at most 366 days after the current UTC date; and, for `MONTH` only, the
  range is at most 12 months. Without it (`YEAR` only): `endDate` is the
  first day of a month, then later than 2000-01-01 and at most 366 days after
  the current UTC date. Last, `chapterDois` has at most 500 entries, each
  parses, none repeats in canonical form. (5) A process-wide widget slot,
  without waiting; the 10 s request deadline starts here. (6) One pool
  connection with a 1 s checkout timeout, and one `READ ONLY, REPEATABLE READ`
  transaction. (7) The prelude, before any query: `CUSTOM_PLANS_SQL`, the
  widget's `SET LOCAL statement_timeout = '5s'` and `PROJECTION_LOCK_SQL`.
  (8) One resolution statement: the work, its type, its publisher and
  package, and its current `has-child` works with their DOIs and publishers
  (at most 501 rows). (9) No such work, or a publisher without
  `METRICS_WIDGET`: `METRIC_WIDGET_UNAVAILABLE`, from one code path after the
  same single statement. (10) Child validation. (11) Served-pair discovery,
  one statement per non-empty section (over all history before `endDate`
  when `startDate` was omitted), then the pair bound; then, with no
  statement, the served start (below), which is refused before labels when
  it falls before 2000-01-01. (12) One labels
  statement: the served platforms' and measures' public labels,
  `transaction_timestamp()` and the rollup frontier, on every path that
  reaches it. (13) The engine in pair mode for each section with a served
  pair. (14) Commit; the connection and the slot are released on every exit
  path, panic unwinding included. The deadline is checked before every
  statement from step 8 on, the engine's included: one engine deadline value,
  created at step 5 with the widget's own instant, serves both engine runs,
  and the widget checks its own statements against the same instant. Past it
  the read is `METRIC_WIDGET_BUSY` and runs no further statement. A statement
  that started before the deadline runs to completion or to its own timeout,
  and a failure is classified as that failure. `COMMIT` is not checked.
- **Public scope.** A measure is eligible only if it is public, enabled and
  additive across time and across works, whatever its category (a public
  `SALES` measure is served, negative values included); a platform only if it
  is enabled. Eligibility is applied in pair discovery, so nothing else can be
  served, bounded, checked or read, and no eligible measure means nothing is
  served, never "unrestricted". Institutions are never served.
- **Work and child works.** A `book-chapter` work is served alone: `chapters`
  is null and a non-empty `chapterDois` is `METRIC_QUERY_INVALID`. Otherwise
  the child works are the work's current `has-child` related works of any
  type (a child's own children are ignored). More than 500 child works is
  `METRIC_QUERY_LIMIT_EXCEEDED`, whatever `chapterDois` selects, never
  truncation. `chapterDois` omitted or `[]` selects every child; otherwise
  every entry must be a child (`METRIC_QUERY_INVALID`), and children without a
  DOI are selected only when every child is. Selected children whose current
  publisher is not the work's publisher are never served and are counted in
  `excludedChildCount`; unselected children are not counted. `chapters` is
  null iff the work is a chapter or no selected child is served. Work status
  is not considered.
- **Served pairs.** Per section, the distinct eligible platform/measure pairs
  with a monthly projection row, or a total-ambiguous month, for the section's
  works in the range, ordered by platform ID and measure ID; when `startDate`
  is omitted, in any month before `endDate`, with each pair's earliest such
  month. A pair with no such row is not shown at all, never as a zero;
  coverage alone does not serve a pair. More than 40 served pairs in a
  section is `METRIC_QUERY_LIMIT_EXCEEDED`, never truncation.
- **Served start.** With `startDate`, the served start is `startDate`. With
  it omitted, it is the first day of the earliest month before `endDate`
  with a served value (a monthly projection row, or a total-ambiguous month,
  of an eligible pair of the section's served works, after child selection
  and exclusion) in either section; with no served pair in either section,
  the first day of the month before `endDate`. Both sections, and so both
  timelines, start on that day, and a pair whose own history starts later
  shows its earlier buckets as `null` or `"0"` under the section 4.3 bucket
  rule, never as missing buckets. The first `YEAR` bucket runs from the
  served start to the next 1 January and the last from the last 1 January
  before `endDate` to `endDate`. Only served values move the start: coverage
  runs, terminal imports without a completion time, unresolved identifiers,
  unapplied rollup work and native-grain records never do, unlike section
  4.3's all-time start. So months before a work's first value are not
  served, and history that rollup has not yet applied is not served until it
  is (the start can move earlier as it is, and until then an all-time total
  can understate the history without a flag, since `isPartial` reflects lag
  inside the served range only). A served start before 2000-01-01 is
  `METRIC_QUERY_LIMIT_EXCEEDED`, never clamped. Every eligible row before
  `endDate` lies in the served range, so an all-history response equals,
  except for `asOf`, the response to the same request with `startDate` set to
  its served start, read against the same database state.
- **Pair mode.** Each section with a served pair is read by the section
  4.1-4.3 engine on the same connection and snapshot, with the section's works
  and the work's publisher as its scope, exactly the served pairs as its
  combinations, the served start (one for both sections) and `endDate` as its
  explicit range, the request's grain, countries on and institutions off: at
  most six statements (`PUBLISHERS_SQL`, `ACCOUNTS_SQL`, `CANONICAL_SQL`, the
  step-7 text, `ASSERTIONS_SQL`, `IDENTIFIER_QUALITY_SQL`). The step-7 text is
  `MONTHS_YEAR_ONLY_SQL` when the served range is exactly whole calendar
  years, `MONTHS_YEAR_SQL` when it holds at least one whole calendar year and
  other whole months, and `MONTHS_SQL` otherwise; whole years read their
  countries from the yearly country projection. It never runs the dashboard's
  work, known-scope, represented-pair, scan or day statements, since every
  widget range is month-aligned and a section holds at most 500 works, and it
  has no combination bound. With an explicit start, `CANONICAL_SQL`'s lag and
  native-grain branches are bounded below by the served start. Monthly total,
  country and coverage-run rows of any other platform/measure pair are dropped
  as they are read, before any check, so coverage-run state on an unserved
  cross pair refuses nothing. Each pair's total, buckets, countries and
  coverage equal a single-pair `metricDashboard` read over the same works,
  served range and grain; the section's `dataThrough` and `isPartial` are
  section 4.1's aggregation over exactly the served pairs. A total or country
  dimension ambiguity, or an active native `MONTH` or `REPORTING_PERIOD`
  record on any served platform x served measure (a deliberate over-refusal
  that reaches unserved cross pairs) refuses the read as
  `METRIC_WIDGET_UNAVAILABLE`. Over
  all history that reaches every such record and month since the served start:
  a native-grain record on a served platform x served measure that ends on or
  before the served start is not read, and one that ends after it refuses the
  read, including when one served platform's or one section's day-grain
  history starts before another served platform's or the other section's
  native history ends; a refusal of either section refuses the whole request.
  Two overlapping coverage runs of one stream on a served pair refuse the read
  as `INTERNAL_ERROR`, as they refuse the single-pair `metricDashboard` read
  of that pair. With monthly serving `OFF`, native-grain records are never
  served. Each engine run's frontier must equal the labels statement's
  (`INTERNAL_ERROR` otherwise).
- **Monthly serving `ON` (`MET-WP4-MONTH-01`).** `METRIC_WIDGET_PAIRS_SQL` and
  `METRIC_WIDGET_HISTORY_PAIRS_SQL` (with the month's first day) also read the
  `metric_native_month` rows of the section's works, with the same range and
  eligibility joins; each stays one text, with one boolean parameter that
  folds that branch away under `OFF`. So a monthly-only pair is served, and
  native months move the all-history served start like any served value.
  Pair mode runs `NATIVE_MONTHS_SQL` as step 3b, right after
  `PUBLISHERS_SQL`, and an all-history read of a section with a served pair
  also runs `PAIR_STARTS_SQL`: at most eight engine statements per section,
  and at most 23 named statements per read (26 captured). In an all-history
  read each pair is judged by section 4.3's pair-start rule, with `S` its
  first evidence, not its first value month, or the served start if that is
  later: so an `UNKNOWN` day between a pair's first run and its first value
  counts, and a pair whose coverage starts mid-month is judged from that day.
  The all-history equality above and the pair-mode equality with the
  single-pair `metricDashboard` read hold apart from that rule's coverage
  fields (a pair's `status`, `dataThrough` and `countryCoverage`, and the
  section's `dataThrough` and `isPartial`). A conflicted month shows as a
  `PARTIAL` pair with `isPartial`, never as a refusal. The over-refusal above
  and the refusal table below name an active `REPORTING_PERIOD` record
  instead of a `MONTH` one, and inconsistent native-grain state is a projected
  inconsistency. Under `OFF`: the same statements apart from that parameter,
  the same counts and refusals, no pair start, and neither new statement is
  ever run.
- **Empty sections.** A section with no served pair runs no engine statement
  and returns empty lists, `dataThrough: null` and `isPartial: true`. `asOf`
  and `rollupWatermark` always come from the labels statement.
- **Ordering.** `platforms` by `platformId`; `measures` by `measureId`;
  `totals` and `coverage` by `platformId`, `measureId`; `timeline` then by
  bucket start; `countries` then by country code.
- **Bounds and cost.** At most 2 widget reads per process (one process-wide
  limiter, separate from any dashboard control, section 4.3's concurrency
  limit included), 1 s checkout, a 5 s statement timeout, a 10 s request
  deadline from taking the slot (which alone governs a widget read; section
  4.3's 20-second deadline does not apply), 12 months for a `MONTH` read,
  500 child works, 500 `chapterDois`, 40 pairs per section and at most 19
  named statements per read (22 captured with the checkout check, `BEGIN`
  and `COMMIT`), for an all-history read too. A `YEAR` read's range is
  bounded by the technical date domain alone (from 2000-01-01 to at most 366
  days after the current UTC date: at most 28 calendar years in 2026), so
  moving that floor or ceiling widens the public surface. Cells per section
  are at most 480 (40 x 12) for `MONTH` and 1,120 (40 x 28) for `YEAR` in
  2026, below the engine's 5,000. An all-history read's pair discovery and
  monthly total read walk every month of the section's works since its first
  value, so its cost grows with the years served; the statement timeout and
  the request deadline still bound every read, and a work whose all-history
  read exceeds them fails as `INTERNAL_ERROR` (or `METRIC_WIDGET_BUSY` past
  the deadline) on every request. No bound is met by truncation.
- **Errors.** Resolver errors, so HTTP 200 with the error in `errors`; `data`
  is `null` unless a named fragment spread or typed inline fragment encloses
  the failing selection. Clients decide on `errors` and `extensions.type`,
  never on the status or on whether `data` is null, and an error response is
  never cached. Fixed widget messages only; no engine message, SQL,
  configuration, package value or token is returned.

| Condition | `extensions.type` |
| --- | --- |
| Switch off; unknown DOI; publisher without `METRICS_WIDGET`; engine capability, dimension-scope or native-grain refusal | `METRIC_WIDGET_UNAVAILABLE` |
| No free slot; checkout timeout or failure; deadline passed before a statement; the prelude lock failing | `METRIC_WIDGET_BUSY` |
| A second `metricWidget` selection in the request; dates outside the technical domain; a `MONTH` read over 12 months ("A MONTH timeline may cover at most 12 months."); an omitted `startDate` with `endDate` not after 2000-01-01 or more than 366 days after today ("When startDate is omitted, endDate must be after 2000-01-01 and at most 366 days after today (UTC)."); a served start before 2000-01-01 ("Metrics for this work or its child works start before 2000-01-01. Give a startDate on or after 2000-01-01."); over 500 `chapterDois`; over 500 child works; over 40 pairs in a section; an aggregate outside the representable range | `METRIC_QUERY_LIMIT_EXCEEDED` |
| A DOI that does not parse; a `MONTH` read without `startDate` ("startDate is required for a MONTH timeline."); dates not first of month or not ordered; a repeated `chapterDois` entry; `chapterDois` on a chapter or naming a non-child | `METRIC_QUERY_INVALID` |
| Any other engine error, `Busy` included (only section 4.3's dashboard limit produces it); an engine failure without a deadline refusal (database failure, statement timeout, inconsistent native-grain state, same-stream coverage-run overlap on a served pair); a missing or mismatched frontier | `INTERNAL_ERROR` |

- **Logging.** With the switch on, each resolution writes one info-level
  outcome line (outcome, elapsed time, per-section served pairs and
  cross-product sizes; for a read that passed step 4, `grain=MONTH|YEAR`,
  `history=true|false` (whether `startDate` was omitted) and `buckets=<n>`,
  the timeline buckets per pair over the served range, or `-` before the
  served start is fixed and on the 2000-01-01 refusal; the DOI only after
  step 9), the first refused second
  selection of a request is info and later ones debug, reasons for
  unavailable and busy are debug (the DOI only after step 8), a checkout
  failure is also warn, and `INTERNAL_ERROR` is error with its reason, section,
  step, last engine statement and elapsed time. A deadline refusal inside the
  engine also writes section 4.3's warn line "metricDashboard read stopped:
  the whole-read deadline passed before a statement could start", at most
  once per read; it names the dashboard, and the same request's widget lines
  (`METRIC_WIDGET_BUSY`, reason `DeadlinePassed`) show it came from a widget
  read. A refusal at a widget-owned step writes no such line. Nothing above
  debug is written with the switch off. No line can hold a package value.
- **Activation gate.** The switch is `OFF` in every environment until a
  separate activation grant per environment, which is CRITICAL unless the CTO
  decides otherwise at that grant (`ADR-0014` section 4.7). Activation also
  waits for the WAF rate rule, `MET-READ-CACHE-01` or a named waiver, the
  public-scope audit, the worker-exposure acceptance, data readiness for
  native-grain history, and the activation signals. Because the all-history
  read becomes the default on every embed, activation also carries section
  4.3's all-time default gate over to the widget: a staging performance
  reference for the all-history `YEAR` read of the largest real books (most
  child works, longest history), reporting rows returned per statement and
  section, the history statement's time and the total latency, which holds
  activation for a CTO decision if any statement exceeds 2.5 s or any read
  exceeds 5 s on production-like data, and, unless a response cache is
  deployed, also measures the read at two concurrent reads per process and
  reports the `METRIC_WIDGET_BUSY` rate for CTO acceptance; and a read-only
  census of the all-history read of every work of each entitled publisher (or
  an agreed sample), reporting the share refused by class (native-grain
  overlap, over 40 pairs, over 500 children, the 2000-01-01 floor,
  ambiguity) and latency, accepted by the CTO or answered by a client
  fallback. A response cache keyed on the rollup frontier alone is not
  enough. Under `ON`, served months change a response without the frontier
  moving; under either value, so do account windows, successions and the
  coverage of imports that apply no `DAY` delta. No stamp exists for them, so
  a cache must expire within a bound the CTO accepts. The switch takes effect
  only on restart. Monthly serving (`THOTH_METRIC_MONTH_SERVING`,
  `MET-WP4-MONTH-01`) is a separate server switch, also read only at process
  start, `OFF` until its own activation grant per environment.

Scope boundaries a consumer must not infer:

- A `MONTH` read covers at most 12 whole months. All-time totals, a by-year
  timeline and all-time countries come from one `YEAR` read with `startDate`
  omitted; a year by month from a separate `MONTH` read of that calendar
  year. Clients must not stitch ranges or sum months into years: a year
  bucket is not the sum of its months when any month is `null`.
- Each bucket is labelled from its own dates: the first and last `YEAR`
  buckets may be part-years, and `startDate` is shared by both sections. A
  pair can have `null` or `"0"` years before its own first value.
- A read whose range reaches past today carries no coverage signal: every
  pair is `UNKNOWN` with `countryCoverage: false` and `isPartial: true`. An
  `endDate` of the first day of the current month avoids that, without the
  current month's values. In an all-history read a pair whose coverage starts
  after the served start is `UNKNOWN`, with `countryCoverage: false` and a
  null `dataThrough`, whatever the `endDate`; `countryCoverage: false` means
  only that some served day lacks complete country coverage, never that the
  pair has no country breakdown. As in section 4.3, clients must not present
  an all-history `dataThrough` as "data up to". With monthly serving `ON`
  that note applies from the pair's own first evidence, not from the served
  start.
- With monthly serving `OFF`, native-grain history (for example
  OPERAS-imported monthly data) is not served; with it `ON`, `MONTH` history
  is served as section 4.1 says and `REPORTING_PERIOD` history is not. A work
  with an active record of an unserved native grain on a served platform
  x served measure that overlaps the served range is unavailable; for an
  all-history read, that is any such record ending after the served start.
  On `METRIC_WIDGET_UNAVAILABLE` or the 2000-01-01 refusal from the
  all-history read, the client shows no all-time view.
- No institutions, warnings, entity-level Metrics fields or role-protected
  data are served, and the protected read service of sections 4.1 and 4.2 is
  unchanged.

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
| `METRICS_INGEST_SERVICE` | Sphinx orchestration caller for managed-source ingestion | `claimMetricSourceUnits`, `updateMetricSourceCheckpoint`, `beginMetricImport`, `ingestMetricBatch`, `completeMetricImport`, `claimMetricRollupDeltas`, `completeMetricRollupDeltas`, `beginCitationRefresh`, `citationRefreshWorks`, `submitCitationBatch`, `completeCitationRefresh` | `MET-WP2-02` (ingestion, checkpoint and import lifecycle), `MET-WP4-01` (rollup-delta application), `CITATIONS-01` C2 (citation write path) |
| `METRICS_READ_SERVICE` | Thoth-owned server-side Metrics query clients | `metricDashboard`, `metricMeasures`, `metricPlatforms`, `metricDataVersion` | `MET-WP4-02`, `METRIC-DATA-VERSION-01` |

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

Recorded disposition for `MET-WP11-B-CORR-H3` (issue #989): the producer SDL
gains exactly one additive `MetricIngestionErrorCode` value,
`REPORTING_GRAIN_MISMATCH`, reachable only through the protected
`ingestMetricBatch` result (`MetricIngestionRowResult.reasonCode`). The Sphinx
consumer subsets pinned at Sphinx `feature/metrics`
`fa4b0f88751ee73617a0697d7fd6b1be221b6b74` (producer
`52483ae29febdc4e253d3c8d70f7206c6e6de48b`) and `develop`
`e97444c20fb5626f9233397ed627a54d1a2a46d2` (producer
`8305de9d465abfec2216184d32e4646791d35188`) enumerate 47 reasons, omit the
new value and decode an unknown reason as a non-retryable contract violation:
Sphinx `REQUIRES CHANGE` through the separate repository-local task
thoth-pub/thoth-sphinx#38, which pins the merged H3 producer contract before
any collection against it. No other known consumer selects `reasonCode` or
decodes this enum.

## Citation write path (CITATIONS-01, thoth#1074, slice C2)

Four operations for the `METRICS_INGEST_SERVICE` role, under the same claim as
the managed-DRIVER lifecycle (`claimMetricSourceUnits`) and only for a source
whose `driver_key` is `openalex-citations`; the pinned publisher must hold
`METRICS_COLLECT`. They write only the `metric_citation_*` tables and the
checkpoint's `last_discovered_at`, `last_completed_at` and lease; the checkpoint
`cursor` is never written. Inputs, bounds and rules:

- `beginCitationRefresh(input)`: creates the claim's one RUNNING refresh, or
  returns it. A refresh still RUNNING under an earlier token is set FAILED.
- `citationRefreshWorks(input)` (query): up to 200 works of the claimed
  publisher with a DOI, not withdrawn or cancelled, not looked up in the last
  seven days, never-attempted first. A work submitted in this refresh, of any
  outcome, is not returned again. `done` is true when none remain or 400 works
  were submitted under this claim.
- `submitCitationBatch(input)`: at most 50 works, 2,000 links and 1 MiB. Adds
  and updates links, never withdraws. Text is sanitised at the boundary (NUL,
  C0/C1 and bidi controls stripped, whitespace collapsed, NFC, then truncated by
  characters). A DOI that is not in shape, or a URL that is not plain https, is
  stored NULL; a link is refused only for a blank or over-long id or for
  self-citation, and every id read counts as seen. Replay of a committed key and
  payload returns its outcome; the same key with another payload is refused. A
  work spanning batches declares `enumerationComplete` and the same
  `enumeratedTotal` in every entry.
- `completeCitationRefresh(input)`: the only place a link is withdrawn, as a
  sweep for each work that was matched, fully enumerated and passes the fixed
  server guard (previous count at most 2, or at least 50% seen, or the same lower
  count twice). Releases the claim. NOT_FOUND and FAILED lookups never withdraw.

Errors: the lifecycle codes above plus `CITATION_WORK_NOT_IN_PUBLISHER`,
`CITATION_REFRESH_NOT_FOUND`, `CITATION_REFRESH_NOT_RUNNING` and
`CITATION_BATCH_IDEMPOTENCY_MISMATCH`. The read (`metricCitations`) and its
serving switch are slice C3.
