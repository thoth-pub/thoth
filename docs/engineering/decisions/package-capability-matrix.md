# Package Capability Matrix

Status: APPROVED
Normative owner: `ADR-0001`
Decision owner: CTO
Approved by: Javi, CTO
Approval date: 2026-07-28
Approval PR: [#772](https://github.com/thoth-pub/thoth/pull/772)

## 1. Package order

The initial packages are:

```text
OASIS
OBELISK
SPHINX
PYRAMID
```

OASIS is the non-null default.

This matrix does not make package selection imply distribution-platform assignments.

## 2. Capability codes

| Capability | Meaning |
|---|---|
| `OAI_PMH` | Publisher works may be considered for OAI-PMH after work-level open-licence and lifecycle checks |
| `METRICS_COLLECT` | Thoth-managed drivers may collect and retain canonical metrics when a source account and platform/measure configuration are enabled |
| `METRICS_IMPORT` | Publisher users may submit approved publisher-controlled usage or sales reports |
| `METRICS_DASHBOARD` | A Thoth-owned authenticated service may serve publisher dashboard metrics |
| `METRICS_WIDGET` | A Thoth-owned authenticated service may serve bounded work-level widget metrics |
| `METRICS_OPERAS_EXPORT` | Eligible finalized canonical metrics may create and deliver OPERAS export claims |

Capability codes are stable API identifiers. Display labels are separate.

## 3. Approved initial mapping

| Package | OAI_PMH | METRICS_COLLECT | METRICS_IMPORT | METRICS_DASHBOARD | METRICS_WIDGET | METRICS_OPERAS_EXPORT |
|---|---:|---:|---:|---:|---:|---:|
| OASIS | No | No | No | No | No | No |
| OBELISK | Yes | Yes | No | No | No | No |
| SPHINX | Yes | Yes | Yes | Yes | Yes | Yes |
| PYRAMID | Yes | Yes | Yes | Yes | Yes | Yes |

## 4. Rationale

### OASIS has no managed collection

Thoth does not distribute OASIS files and therefore has no managed usage-data
source for OASIS publishers. OASIS has no `METRICS_COLLECT` capability, and the
system must not fabricate data or treat missing data as zero.

### Private, non-blocking OBELISK collection

OBELISK permits private background collection but does not permit publisher
import, dashboard serving, widget serving or OPERAS export.

Collection still requires all of:

- an enabled metric source account;
- valid source credentials;
- an enabled platform/measure mapping;
- direct collection configured for that mapping;
- an enabled operational schedule;
- successful source-specific validation.

OBELISK collection is operationally non-blocking. Missing configuration, source
outages, retries, reconciliation and collection failures must not block
distribution, metadata, package changes or other unrelated publisher services.
Non-blocking does not mean unconfigured collection.

### SPHINX and PYRAMID metrics access

SPHINX has every initial metrics capability. PYRAMID includes all SPHINX metrics
capabilities. Both packages permit collection, publisher import, dashboard and
widget serving, and OPERAS export when the additional configuration,
authorization and rollout requirements are satisfied.

### OAI-PMH

OAI package eligibility follows the approved Publisher Services design:

- OASIS is excluded;
- OBELISK, SPHINX and PYRAMID may be eligible;
- work-level licence and lifecycle rules still apply.

## 5. Effective-entitlement rules

A capability is effective only when:

1. the publisher's current package grants it;
2. any feature-specific configuration also permits it;
3. the caller has the required user or service authorization;
4. operational rollout has enabled the feature.

A package capability alone must never:

- enable a distribution platform;
- create a distribution job;
- create a metric source account;
- activate a driver schedule;
- bypass publisher-platform upload approval;
- bypass work-level licence or lifecycle checks.

## 6. Upgrade behaviour

Approved behaviour:

- changing to SPHINX or PYRAMID enables dashboard and widget access to retained canonical history;
- existing retained history is not rewritten;
- future publisher imports become available after the package change;
- new finalized eligible metric revisions may be exported to OPERAS;
- existing historical metric revisions do not automatically create a bulk OPERAS export.

Historical OPERAS export requires a separate, bounded, reviewed administrative backfill with:

- explicit publisher and date scope;
- dry run;
- expected record count;
- idempotency;
- rate limits;
- reconciliation;
- rollback/stop procedure;
- CTO activation approval.

## 7. Downgrade behaviour

Approved behaviour:

- canonical metric history is retained;
- on downgrade to OBELISK, validly configured private collection may continue;
- on downgrade to OASIS, Thoth-managed collection stops;
- new publisher imports are denied;
- dashboard and widget service responses are denied;
- no new OPERAS export work is created or delivered;
- distribution assignments remain unchanged;
- no canonical records are deleted.

In-flight work must re-check entitlement at the final write/delivery boundary and fail closed.

## 8. Implementation tests

An implementation of this matrix must test every package/capability pair.

It must also test:

- OASIS default for existing and new publishers;
- package changes do not modify distribution assignments;
- package changes do not create distribution jobs;
- publisher users cannot edit packages;
- non-owner publisher users cannot read another publisher's package/capabilities;
- superusers can read and change package configuration;
- metrics service checks use capabilities rather than package-name comparisons;
- OAI eligibility uses `OAI_PMH` plus licence/lifecycle checks;
- retained metrics remain inaccessible without dashboard/widget capability;
- upgrade does not enqueue uncontrolled historical OPERAS exports;
- downgrade stops new protected serving/export without deleting data.

## 9. Approval checklist

The CTO must explicitly confirm:

- [x] OASIS has no `METRICS_COLLECT` capability because Thoth has no managed
  OASIS usage-data source.
- [x] OBELISK has `METRICS_COLLECT` but no metrics serving/import/export.
- [x] OBELISK collection is configured, private and non-blocking for unrelated
  operations.
- [x] SPHINX has all metrics capabilities.
- [x] PYRAMID includes all SPHINX metrics capabilities.
- [x] Retained history becomes visible after upgrade.
- [x] Historical OPERAS export requires an explicit backfill.
- [x] Downgrade retains canonical data and does not change distribution assignments.

Approved by: Javi, CTO

Approval date: 2026-07-28
