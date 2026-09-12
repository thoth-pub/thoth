# Metrics Source and Driver Inventory

Status: DESIGN INVENTORY; FIRST CLOUDFRONT SOURCE CONTRACT APPROVED UNDER `CF-GATE-01`; ALL OTHER SOURCE MAPPINGS NOT APPROVED
Owner: Thoth Metrics programme
Evidence date: 2026-07-24 (initial inventory); 2026-09-11 (`CF-GATE-01` outcome and `MET-WP1-13` administration contract)

## 1. Purpose

Track where activity occurred (`MetricPlatform`), how it arrived (`MetricSource`), the concrete account (`MetricSourceAccount`), measure/grain/dimensions, fixture readiness and OPERAS behavior.

Distribution assignments do not authorize metrics.

## 2. Initial candidates

| Source/route | Candidate platform | Candidate measure | Grain | Dimensions | Acquisition | Fixture state | Mapping state | Readiness |
|---|---|---|---|---|---|---|---|---|
| CloudFront logs | publisher website/CDN | `title_sessions` | DAY | work, country | DRIVER (`driver_key = cloudfront`) | protected samples inspected in place under `CF-GATE-01`; synthetic fixtures owned by `MET-WP7-01` | first source contract approved (private ledger); typed `cloudfront-source-account/1` administration configuration delivered by `MET-WP1-13` | CONTRACT APPROVED; DRIVER NOT IMPLEMENTED |
| Thoth CSV v1 | approved publisher platform | approved usage or `net_units` | DAY/MONTH/PERIOD | publication/country/institution optional | PUBLISHER_UPLOAD | canonical examples to create | common contract fixed | BLOCKED ON WP1/WP2 |
| COUNTER 5 | approved publisher platform | selected mappings | report-dependent | report-dependent | PUBLISHER_UPLOAD | representative reports required | unresolved | SAMPLE + MAPPING |
| OAPEN | OAPEN metric platform | source-defined usage | likely day/month | source-dependent | DRIVER or OPERAS | examples required | unresolved | SAMPLE + MAPPING |
| JSTOR | JSTOR metric platform | source-defined usage | likely month | source-dependent | TBD | examples required | unresolved | SAMPLE + MAPPING |
| OpenEdition | OpenEdition metric platform | source-defined usage | TBD | TBD | DRIVER or OPERAS | examples required | unresolved | SAMPLE + MAPPING |
| Other collectors | separate platforms | source-specific | TBD | TBD | DRIVER | inventory required | unresolved | DISCOVERY |
| OPERAS mirror | mapped external platforms | configured | remote grain | supported projection | OPERAS | snapshot/scan fixtures | URI mappings required | EXTERNAL BLOCKER |
| Admin historical import | approved mapping | preserved measure | source grain | source dimensions | ADMIN_IMPORT | retained inventory | versioned normalizer | MIGRATION ONLY |

## 3. Required fixture set per source

- valid ordinary/minimum/multi-work examples;
- supported periods and dimensions;
- malformed identifiers/dates/values;
- duplicates and regenerated reports;
- partial and empty successful reports;
- coverage/finalization evidence;
- stable report IDs or manifest fields;
- checksum/ETag behavior;
- credential-free sanitized data.

No fixture may contain secrets, IP addresses or prohibited personal data.

## 4. Driver scoping record

```text
Source code:
Metric platform:
Source account key:
Ownership class:
Measures:
Grains:
Country/institution/publication support:
Discovery:
Checkpoint:
Lookback:
Finalization:
Upstream report ID:
Retention:
Manifest:
Coverage:
Revision behavior:
Schedule:
Credential owner/storage:
Failure semantics:
OPERAS direct_collection:
Fixtures:
Normalizer version:
Methodology version:
```

## 5. CloudFront gate

`CF-GATE-01` is approved and complete in the private `thoth-pub/thoth-sphinx`
ledger. This public inventory records only the architectural facts the
source/administration contract depends on; exact provider identifiers,
hostnames, buckets, prefixes, publisher identity and operational topology are
private and are never copied into this repository.

Public truth established by the gate and consumed by `MET-WP1-13`:

- the first CloudFront source is a `DRIVER` acquisition route whose immutable
  `driver_key` is exactly `cloudfront`;
- its acquisition route is CloudFront standard logging (legacy) delivered to
  S3; standard-logging-v2 delivery is excluded from collection;
- a source account for it carries the typed configuration
  `cloudfront-source-account/1` with `logging.mode = LEGACY_S3` and non-secret
  `hostname`, `bucket` and `prefix` routing values, where the hostname equals
  the account's immutable `external_key`; such an account is created with a
  non-null `expected_publisher_id` naming the canonical publisher it reports
  for, which the managed `DRIVER` ingestion coordinator requires to match the
  import's publisher; credentials never enter that configuration;
- request qualification, DOI-path resolution to canonical Works, the pinned
  COUNTER bot resource, 30-minute IP + User-Agent sessionization with
  previous/target/following-day context, and the `COMPLETE`/`PARTIAL`/`UNKNOWN`
  coverage semantics are fixed by the gate's evidence and remain the driver's
  contract.

Not delivered by the gate or by `MET-WP1-13`: the CloudFront collection driver
itself, checkpoint/claim/lease runtime, the versioned GeoIP resource, synthetic
driver fixtures, and any real source, platform or source-account row. No
collection is active.

## 6. COUNTER gate

Require report type/version examples, desired metric types, identifiers, periods, institution handling, suppression/zero semantics, regeneration behavior and approval ownership. Unsupported sections fail explicitly.

## 7. OPERAS gate

Require stable event/uploader/measure mappings, complete cursor/snapshot/replication or explicit unverified mode, loop prevention, exported-event identification, dimension projection, finalization, divergence semantics and reconciliation fixtures.
