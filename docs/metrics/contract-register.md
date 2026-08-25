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

## 3. Internal ingestion

Thoth owns bounded claim/checkpoint/import/batch/rollup/export/reconciliation operations consumed by Sphinx.

Required: least privilege, leases, stale-token rejection, idempotency, bounded batches, sanitized errors, exact row classifications and fail-closed behavior.

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

Within that convention, the following remain **unapproved** and are WP5-owned
bounded decisions under WP5's own approved specification:

- exact Metrics role codes — the sketched candidates

  ```text
  METRICS_READ_SERVICE
  METRICS_INGEST_SERVICE
  METRICS_SYNC_SERVICE
  ```

  remain proposals only and are not promoted by `ADR-0008` or by this
  register;
- the exact permissions/operation matrix per role;
- credential, provisioning and rotation arrangements.

Consumers must not substitute superuser: `SUPERUSER` is not a machine-service
shortcut.

## 8. Cross-repository gate

Record exact producer commit/preview, generated types, contract fixtures, compatibility, merge order, non-production credentials and independent rollback before final review.
