# Thoth Metrics Decision Summary

Status: ACTIVE SUMMARY  
Last updated: 2026-07-24  
Owner: CTO

The approved technical design and approved ADRs remain authoritative.

## 1. Fixed architectural decisions

### Canonical store and API

- Use the existing Thoth PostgreSQL instance and standard schema.
- Use `metric_`-prefixed tables with direct foreign keys.
- Use the existing Thoth GraphQL API.
- Sphinx writes through protected GraphQL operations, not direct database access.

### Canonical observation grain

The canonical unit is a source-reported aggregate at its finest reliable grain: day, month or explicit reporting period. Do not fabricate event-level precision. Periods are half-open.

### Identity

Work identity requires DOI. Optional publication identity uses ISBN or an unambiguous publication type. Institution identity uses an existing Thoth institution resolved by ROR. Ingestion does not create institutions.

### Values and records

Usage measures require non-negative integers. Sales measures may allow signed integer units. Identity excludes source and value. Same identity/same content is a duplicate. Managed winning sources may revise. Other changed values are conflicts. Publisher submissions are final.

### Coverage and attribution

Coverage is `COMPLETE`, `PARTIAL` or `UNKNOWN`. A missing record is zero only inside complete coverage. Historical metrics follow current work ownership and current imprint/series relationships.

### Rollups

Rollups are rebuildable projections. Canonical writes create durable rollup deltas. Revisions apply `new - old`.

### Publisher imports

Publisher uploads belong to the authenticated publisher, may use only approved `PUBLISHER_CONTROLLED` platforms, cannot submit managed data, become final after acceptance and produce explicit row classifications.

### CloudFront methodology

The target is daily country-level `title_sessions` using GET 200/206 requests, DOI paths, versioned bot filtering, IP plus user-agent transient sessions, a rolling 30-minute window, first UTC day attribution and deduplication by session/platform/DOI/country. No IP or user agent enters normalized output.

### OPERAS

Outbound uses durable Thoth claims. Sphinx delivers idempotently. Direct mappings are excluded inbound. Local corrections after remote delivery create divergence issues. Inbound completeness is not guaranteed without cursor, replication or complete snapshots.

## 2. Proposed shared decisions

`ADR-0001` is required for entitlements, serving, imports, export and collection policy.

`ADR-0002` is required for metric platform implementation. `MetricPlatform` remains separate from `DistributionPlatform`.

## 3. Decisions still required

### Service-role codes

Proposed:

```text
METRICS_READ_SERVICE
METRICS_INGEST_SERVICE
METRICS_SYNC_SERVICE
```

Approve exact codes, scope, rotation, audit ownership and whether rollup/reconciliation needs separate roles. Do not use `SUPERUSER` as a machine-service shortcut.

### Lease/job primitive reuse

Recommendation: share conventions and utility patterns, but keep distribution jobs and metric checkpoints/import/export claims domain-specific. A new ADR is required before one universal persisted job framework.

### CloudFront multi-country sessions

The design counts one title session per country when one session contains qualifying requests from multiple countries. Confirm before WP7 fixtures are finalized.

### OPERAS completeness

Guaranteed import requires a created-at cursor, replication, complete snapshot or complete incremental endpoint. Rolling scans must remain explicitly unverified.

### COUNTER subset

Select report types and metric mappings only after representative reports are supplied. Unsupported metrics fail explicitly.

## 4. Implementation invariants

1. No resolver calls OPERAS.
2. No driver writes canonical data outside Thoth.
3. No browser contains service credentials.
4. No dependency failure becomes zero.
5. No source field silently expands canonical identity.
6. New dimensions require explicit schema/API design.
7. GitHub Actions does not own checkpoints or leases.
8. Sphinx runs are restartable.
9. Every row receives an explicit classification.
10. Every source mapping is versioned and deterministic.
