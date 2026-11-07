# Thoth Metrics Decision Summary

Status: ACTIVE SUMMARY
Last updated: 2026-10-30
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

## 2. Shared decisions

`ADR-0001` is `APPROVED` (Javi, CTO, 2026-07-28, approval PR
[#772](https://github.com/thoth-pub/thoth/pull/772)). It establishes the
code-owned exhaustive package-capability model for later entitlement, serving,
import, export and collection tasks:

| Package | OAI_PMH | METRICS_COLLECT | METRICS_IMPORT | METRICS_DASHBOARD | METRICS_WIDGET | METRICS_OPERAS_EXPORT |
|---|---:|---:|---:|---:|---:|---:|
| OASIS | No | No | No | No | No | No |
| OBELISK | Yes | Yes | No | No | No | No |
| SPHINX | Yes | Yes | Yes | Yes | Yes | Yes |
| PYRAMID | Yes | Yes | Yes | Yes | Yes | Yes |

OASIS has no `METRICS_COLLECT` entitlement. Under current operations Thoth has no
managed OASIS usage-data source because it does not operationally distribute
OASIS files. That context does not create a package-to-platform rule: ADR-0001
does not disable or remove OASIS distribution assignments, prevent superuser
platform configuration, define dissemination eligibility, create a distribution
capability or change distribution-job behaviour. Any permanent OASIS
distribution prohibition requires a separately approved Publisher Services
decision through ADR-01 or another cross-programme ADR.

Metrics collection must check `METRICS_COLLECT` on the current package and must
not infer entitlement from a distribution assignment or remote location.
OBELISK collection is private: there is no publisher import, dashboard serving,
widget serving or OPERAS export. It requires a valid source account, credentials
and source-specific configuration, and missing configuration, source outages,
retries, reconciliation or collection failure must not block unrelated
distribution, metadata, package-change or publisher-service operations. Missing
data is not zero.

Retained canonical history becomes available after an upgrade only when the new
package grants the relevant serving capability. Historical OPERAS export
requires a separately scoped, reviewed and explicitly activated backfill.
Every package change uses the resulting package's capabilities:

- `PYRAMID -> SPHINX` removes no initial capability; collection, import,
  dashboard, widget, OAI-PMH and eligible OPERAS export remain subject to normal
  configuration, authorization and rollout requirements;
- `SPHINX` or `PYRAMID -> OBELISK` retains OAI-PMH and validly configured private
  collection while denying publisher import, dashboard, widget and OPERAS
  export;
- any package `-> OASIS` denies all six initial capabilities and stops
  Thoth-managed collection;
- every downgrade retains canonical history, leaves distribution-platform
  assignments unchanged and rechecks the relevant capability at the final
  boundary.

This approval settles the shared matrix but does not start implementation or
make any Metrics work package ready.

`ADR-0002` is `APPROVED` (CTO, 2026-07-27, approval PR [#769](https://github.com/thoth-pub/thoth/pull/769)) as written, and is required for metric platform implementation. `MetricPlatform` remains a separate domain type from `DistributionPlatform`, with no name-based conversion and no initial cross-domain mapping.

`ADR-0003` is `APPROVED` and repository-authoritative through
`THOTH-DB-CTRL-02` and merged PR
[#778](https://github.com/thoth-pub/thoth/pull/778):
`thoth-api/src/schema.rs` is the repository-authoritative, manually maintained
Diesel schema contract (Architecture A), and schema-bearing Metrics slices
must atomically change migrations, `schema.rs`, models and database-backed
tests in one bounded task. The previously open Diesel-generation question is
resolved.

`ADR-0008` is `APPROVED` and repository-authoritative within its approved
scope: machine/service authorization uses dedicated, least-privilege,
domain-specific project roles with explicit policy guards and authorization
matrices; `SUPERUSER` authority does not imply machine-role authority; and
exactly seven durable-job conventions are shared as conventions, not as a
framework. `ADR-0008` selects no Metrics role name, entitlement model,
credential model or operation matrix.

### Canonical reporting grain is immutable for its v1 identity (`MET-WP11-B-CORR-H3`)

Decision (issue [#989](https://github.com/thoth-pub/thoth/issues/989), a
pre-B correction under #981 Amendment 8 as corrected by Amendment 8-C1 and
#989 correction 1): the v1 canonical identity hash deliberately excludes the
reporting grain, so a `DAY` observation and a one-day `REPORTING_PERIOD`
observation (or a `MONTH` observation and a one-month `REPORTING_PERIOD`
observation) of the same cell and period share one canonical identity. The
grain persisted on `metric_record` by first arrival is the record's grain for
as long as that identity exists. Later evidence for the same v1 identity with
a different grain is never silently accepted:

- the shared canonical application authority compares the incoming grain with
  the persisted grain under the existing `metric_record` row lock, before any
  content-duplicate, source-conflict or revision decision, and a mismatch is
  the distinct internal outcome `ReportingGrainMismatch`, which rewrites no
  canonical grain, revises no canonical content, emits no rollup delta,
  chooses no new winner and is never classified as a duplicate or a conflict;
- ordinary ingestion records the observation as `REJECTED` with the additive
  closed-vocabulary reason `REPORTING_GRAIN_MISMATCH` (bounded field
  `reporting_grain`, static sanitized message) through the existing
  rejected-provenance, import-error and `invalid_count` paths, keeping their
  cardinality and replay semantics;
- automatic unresolved-DOI reconciliation maps the same outcome to the
  existing retryable `BLOCKED_INCONSISTENT_EVIDENCE` state, whose meaning is
  extended to include a canonical reporting-grain mismatch, under the
  unchanged capped 1h, 2h, 4h, 8h, 16h, 24h schedule; it never mutates the
  original quarantine evidence, provenance, import counters, status or
  coverage, and no new persisted state or reason is introduced.

Explicitly not decided here: no v1 hash byte, golden vector or `hash.rs` line
changes; no historical record is normalized, no committed historical outcome
is altered and no committed batch replay changes; first-arrival grain
selection stays; source grains are not distinguished in identity; and there is
no hash v2, repair migration or schema change. The additive GraphQL enum value
is not automatically compatible with a closed-enum consumer: Sphinx
`REQUIRES CHANGE` through thoth-pub/thoth-sphinx#38 against the merged
producer contract before any collection against it.

## 3. Decisions still required

### Service-role codes (WP5-owned)

The shared machine-role convention itself is decided by `ADR-0008` and is no
longer open. Still required — as WP5-owned bounded decisions under WP5's own
approved specification — are the exact Metrics role codes, scope, the
permissions/operation matrix, credential/provisioning and rotation
arrangements, audit ownership and whether rollup/reconciliation needs
separate roles. The codes previously sketched for discussion:

```text
METRICS_READ_SERVICE
METRICS_INGEST_SERVICE
METRICS_SYNC_SERVICE
```

remain proposals only and are not approved by `ADR-0008` or by this summary.
Do not use `SUPERUSER` as a machine-service shortcut.

### Lease/job primitive reuse (decided by ADR-0008)

Decided as recommended: conventions and utility patterns are shared, while
distribution jobs and metric checkpoints/import/export claims stay
domain-specific. BE-04's job tables, domain types and lifecycle APIs are
programme-local and must not be reused by Metrics by analogy. A new explicit
cross-programme ADR is required before one universal persisted job framework.

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
11. OBELISK collection failures and missing configuration do not block unrelated
    operations.
12. Missing or unavailable metric data is never fabricated or treated as zero.
13. An existing canonical record's reporting grain is immutable for its v1
    identity; mismatched later evidence is rejected or blocked, never silently
    accepted, and never changes the record.
