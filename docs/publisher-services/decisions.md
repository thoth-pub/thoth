# Publisher Services Decision Summary

Status: ACTIVE SUMMARY
Last updated: 2026-07-28
Owner: CTO

This file summarizes decisions. The approved technical design and approved ADRs remain authoritative.

## 1. Settled product decisions

### Packages

The package enum is:

```text
OASIS
OBELISK
SPHINX
PYRAMID
```

Every publisher has exactly one package.

OASIS is the non-null default.

Publisher users may read their own package. Only superusers may change it. Package values are not anonymous public data.

Package choice does not itself enable or disable distribution platforms.

### Distribution assignments

Every independently meaningful destination has its own `DistributionPlatform` value.

Assignments are explicit publisher configuration.

Disabled rows are retained for history. Re-enabling creates a new activation identity.

OAPEN and DOAB are separate destinations but initially form one linked selection. They map to one logical delivery adapter and must not upload twice.

### Licence authority

`thoth-pub/cc-license` is authoritative for supported Creative Commons licences and retained public-domain tools.

Thoth must not independently parse or normalize Creative Commons URLs.

A missing licence is `NULL` and means no declared open licence/All Rights Reserved.

### OAI-PMH eligibility

OAI-PMH is deferred.

A work is eligible only when:

1. the publisher has the approved OAI package capability;
2. the work has a licence recognized as open by `cc-license`;
3. existing lifecycle and metadata requirements pass.

OASIS is excluded. Non-open works are excluded for every package.

### Desired, job and observed state

Keep separate:

- desired publisher configuration;
- durable execution jobs;
- future observed per-work/per-platform delivery state.

This programme initially implements desired state and durable publisher back-catalogue jobs only.

## 2. Shared decisions

### ADR-0001 - Package capability model

Status: `APPROVED` (Javi, CTO, 2026-07-28, approval PR
[#772](https://github.com/thoth-pub/thoth/pull/772)).

Approved architecture:

- code-owned exhaustive capability mappings;
- Thoth ownership and stable GraphQL capability codes;
- no database capability rows or bespoke publisher overrides;
- entitlement remains separate from source accounts, credentials and other
  operational configuration;
- retained metrics visible after an entitled upgrade;
- no automatic historical OPERAS bulk export after upgrade;
- package changes use the resulting package's capabilities, and downgrades
  retain canonical metrics;
- package changes never alter distribution assignments.

The approved package matrix is:

| Package | OAI_PMH | METRICS_COLLECT | METRICS_IMPORT | METRICS_DASHBOARD | METRICS_WIDGET | METRICS_OPERAS_EXPORT |
|---|---:|---:|---:|---:|---:|---:|
| OASIS | No | No | No | No | No | No |
| OBELISK | Yes | Yes | No | No | No | No |
| SPHINX | Yes | Yes | Yes | Yes | Yes | Yes |
| PYRAMID | Yes | Yes | Yes | Yes | Yes | Yes |

OASIS is not entitled to Thoth-managed metrics collection. Under current
operations Thoth has no managed OASIS usage-data source because it does not
operationally distribute OASIS files. This metrics-entitlement decision does not
disable or remove OASIS distribution-platform assignments, prevent superuser
platform configuration, define dissemination eligibility, create a distribution
capability or change distribution-job behaviour. Any permanent OASIS
distribution prohibition requires a separately approved decision through ADR-01
or another cross-programme ADR.

Metrics collection must not infer entitlement from a distribution assignment or
remote location. OBELISK collection is private, requires valid source
credentials and source-specific configuration, and must not block distribution,
metadata, package changes or unrelated publisher services when configuration is
missing or a source fails.

Package changes use the resulting package's capabilities:

- `PYRAMID -> SPHINX` removes no initial capability;
- `SPHINX` or `PYRAMID -> OBELISK` retains OAI-PMH and configured private
  collection while denying publisher import, dashboard, widget and OPERAS
  export;
- any package `-> OASIS` denies all six initial capabilities and stops
  Thoth-managed collection;
- every downgrade retains canonical metric history, leaves distribution
  assignments unchanged and rechecks the relevant capability at the final
  boundary.

Approval settles the shared architecture but does not start or complete `BE-01`,
metrics entitlement work or `OAI-01`. Each remains subject to its own approved
bounded specification and other tracker dependencies.

### ADR-0002 - Platform domain boundaries

Status: `APPROVED` (CTO, 2026-07-27, approval PR
[#769](https://github.com/thoth-pub/thoth/pull/769)). Approved as written.

Approved architecture:

- `DistributionPlatform` and `MetricPlatform` remain separate types;
- no name-based conversion;
- no initial cross-domain mapping table;
- OAPEN/DOAB linkage exists only in the distribution domain unless separately approved.

Approval removes one dependency; it does not make `ADR-01`, `BE-02` or the
metrics platform registry ready.

Implementation dependency:

- Publisher Services `ADR-01` and `BE-02`
- metrics platform registry.

## 3. Decisions delegated to Publisher Services ADR-01

ADR-01 must finalize:

1. complete user-visible distribution destination inventory;
2. stable enum codes and display names;
3. current uploader, feed or manual mechanism;
4. linked groups;
5. `AutomaticPush`, `PullFeed` or `Manual` behaviour;
6. whether back-catalogue activation creates a job;
7. update and withdrawal support;
8. credential ownership model;
9. current publisher-configuration source;
10. whether the following are separate destinations:
   - Google Books and Google Play;
   - EBSCO KB and EBSCOHost;
   - multiple ProQuest products/destinations.

No `OTHER` enum value is permitted.

## 4. Operational invariants

1. API outages fail closed.
2. An empty publisher/platform assignment is a successful no-op, never "all".
3. Backfill of existing assignments creates no back-catalogue jobs.
4. One linked activation creates one logical multi-target job.
5. Pull-feed and manual destinations create no uploader job.
6. Claims are leased and stale claim tokens cannot complete current work.
7. Automatic job creation is initially inactive.
8. Comparison mode must be clean before legacy configuration cutover.
9. Production activation requires a pilot, monitoring, rollback and CTO approval.
10. Legacy configuration remains available through the observation period.

## 5. Future decisions explicitly deferred

- work-level `ALL_PUBLISHER_PLATFORMS`, `SELECTED_PLATFORMS`, `NONE`;
- exact work-level platform sets;
- metadata-change outbox;
- work upsert and withdrawal jobs;
- complete observed delivery state;
- delivery fingerprints and remote identifiers;
- scheduled reconciliation;
- Rust ports of uploaders;
- publisher-managed service changes;
- package-to-platform defaults.

These must reuse the approved package, platform, adapter, audit and job foundations.
