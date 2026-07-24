# Metrics Source and Driver Inventory

Status: INITIAL DESIGN INVENTORY; SOURCE MAPPINGS NOT APPROVED  
Owner: Thoth Metrics programme  
Evidence date: 2026-07-24

## 1. Purpose

Track where activity occurred (`MetricPlatform`), how it arrived (`MetricSource`), the concrete account (`MetricSourceAccount`), measure/grain/dimensions, fixture readiness and OPERAS behavior.

Distribution assignments do not authorize metrics.

## 2. Initial candidates

| Source/route | Candidate platform | Candidate measure | Grain | Dimensions | Acquisition | Fixture state | Mapping state | Readiness |
|---|---|---|---|---|---|---|---|---|
| CloudFront logs | publisher website/CDN | `title_sessions` | DAY | work, country | DRIVER | protected samples required | methodology fixed; path/resources required | SAMPLE REQUIRED |
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

Require protected previous/target/following-day samples, DOI path rules, bot/GeoIP versions, unknown-country handling, multi-country decision, deterministic expected aggregates and confirmation that raw personal data stays in place.

## 6. COUNTER gate

Require report type/version examples, desired metric types, identifiers, periods, institution handling, suppression/zero semantics, regeneration behavior and approval ownership. Unsupported sections fail explicitly.

## 7. OPERAS gate

Require stable event/uploader/measure mappings, complete cursor/snapshot/replication or explicit unverified mode, loop prevention, exported-event identification, dimension projection, finalization, divergence semantics and reconciliation fixtures.
