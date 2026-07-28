# ADR-0001 - Publisher Package Capability Model

Status: APPROVED
Date: 2026-07-24
Decision owner: CTO
Approved by: Javi, CTO
Approval date: 2026-07-28
Approval PR: [#772](https://github.com/thoth-pub/thoth/pull/772)
Programmes affected: Publisher Services, Thoth Metrics, OAI-PMH
Repositories affected: `thoth`, `thoth-app`, `thoth-sphinx`, `metrics-dashboard`, `metrics-widget`
Supersedes: None
Superseded by: None

## 1. Context

Publisher Services defines one non-null package per publisher:

```text
OASIS
OBELISK
SPHINX
PYRAMID
```

OASIS is the default.

Package selection does not enable or disable distribution platforms.

Thoth Metrics requires collection, import, dashboard, widget and OPERAS-export entitlements. Metrics logic must not hardcode package names in multiple resolvers, workers and clients.

OAI-PMH also needs package eligibility, while retaining separate work-level open-licence and lifecycle checks.

Without one shared capability model, programmes could implement conflicting package checks and make future package changes unsafe.

## 2. Decision drivers

- One canonical entitlement interpretation.
- Exhaustive compile-time handling of every package and capability.
- No duplicated package-name checks across programmes.
- Auditable changes through normal code review.
- Simple initial implementation.
- No runtime configuration that can silently broaden access.
- Clear separation between product entitlement and operational configuration.
- Safe upgrade and downgrade behaviour.
- No coupling between package and distribution assignments.

## 3. Options considered

### Option A - Hardcode package checks in each feature

Example:

```text
package == SPHINX || package == PYRAMID
```

Advantages:

- minimal initial code.

Disadvantages:

- repeated business rules;
- drift between OAI, metrics queries, imports and workers;
- difficult auditing;
- package changes require searching every feature;
- high risk of inconsistent downgrade behaviour.

Decision: Rejected.

### Option B - Database-owned capability catalogue and package mappings

Capabilities and package mappings are editable database rows.

Advantages:

- runtime configuration;
- no deployment needed for mapping changes.

Disadvantages:

- new schema, administration and authorization surface;
- configuration can drift across environments;
- accidental changes can immediately broaden access;
- requires audit, cache invalidation and migration rules before any demonstrated need;
- conflicts with the current package enum and code-owned descriptor direction.

Decision: Rejected for the initial implementation.

A future ADR may introduce database-managed commercial configuration if a real operational requirement emerges.

### Option C - Code-owned exhaustive capability mapping

Define stable package and capability enums in Thoth and one exhaustive mapping function.

Advantages:

- one reviewed source of truth;
- compile-time exhaustiveness;
- deterministic across environments;
- easy unit-test matrix;
- package changes remain visible in code review and releases;
- no runtime path can silently grant a new capability.

Disadvantages:

- mapping changes require a release;
- not suitable for bespoke per-publisher overrides.

Decision: Recommended.

### Option D - Hybrid code defaults with per-publisher overrides

Code defines defaults, but database rows may grant or revoke individual capabilities.

Advantages:

- flexible exceptions.

Disadvantages:

- creates hidden bespoke products;
- complicates authorization and support;
- makes effective entitlement harder to explain;
- requires override precedence and audit semantics;
- can silently invalidate the package model.

Decision: Rejected for the initial implementation.

## 4. Decision

### 4.1 Ownership

Thoth owns the package and capability model.

Use:

```text
ThothPackage
PublisherCapability
```

as closed Rust enums with stable GraphQL codes.

The PostgreSQL package enum stores the publisher package. Capability mappings are code-owned and are not persisted as independent publisher capability rows.

### 4.2 Exhaustive mapping

Implement one exhaustive mapping equivalent to:

```rust
impl ThothPackage {
    pub fn capabilities(self) -> &'static [PublisherCapability];
    pub fn has_capability(self, capability: PublisherCapability) -> bool;
}
```

Every package and capability pair must be covered by tests.

Feature code calls `has_capability`. It must not compare package names directly except inside the mapping implementation or its tests.

The normative approved matrix is:

```text
docs/engineering/decisions/package-capability-matrix.md
```

| Package | OAI_PMH | METRICS_COLLECT | METRICS_IMPORT | METRICS_DASHBOARD | METRICS_WIDGET | METRICS_OPERAS_EXPORT |
|---|---:|---:|---:|---:|---:|---:|
| OASIS | No | No | No | No | No | No |
| OBELISK | Yes | Yes | No | No | No | No |
| SPHINX | Yes | Yes | Yes | Yes | Yes | Yes |
| PYRAMID | Yes | Yes | Yes | Yes | Yes | Yes |

### 4.3 Capability set

The initial capabilities are:

```text
OAI_PMH
METRICS_COLLECT
METRICS_IMPORT
METRICS_DASHBOARD
METRICS_WIDGET
METRICS_OPERAS_EXPORT
```

Distribution-platform access is intentionally not a package capability. Desired distribution platforms remain explicit publisher assignments.

### 4.4 GraphQL exposure

Protected publisher service configuration exposes:

- current package;
- effective capability codes;
- enabled distribution platforms as a separate field.

Publisher users may read their own package and capabilities. Superusers may read any publisher and change the package.

Anonymous users and users scoped only to another publisher cannot read package or capability values.

Public distribution-platform queries do not expose package values.

### 4.5 Enforcement

Capabilities are enforced in Thoth at the owning operation:

- OAI query eligibility;
- publisher metric import initialization and completion;
- protected dashboard/widget metrics queries;
- metrics collection claim/configuration;
- OPERAS export claim and delivery.

The frontend may hide controls, but UI state is not authorization.

Machine roles and publisher-platform approvals remain additional checks.

### 4.6 Operational configuration

A capability means the package permits a feature. It does not configure or activate it.

Actual operation may additionally require:

- source accounts;
- platform/measure configuration;
- publisher-platform approval;
- feature flags;
- service credentials;
- schedules;
- rollout approval.

A package capability never fabricates operational configuration or metric data.
In particular:

- OASIS has no managed collection capability because Thoth does not distribute
  OASIS files and therefore has no managed usage-data source for those
  publishers;
- OBELISK background collection is private and may run only when a valid source
  account, credentials and source-specific operational configuration exist;
- missing OBELISK source configuration, source outages, retries, reconciliation
  and collection failures must not block distribution, metadata, package changes
  or other unrelated publisher services;
- missing or unavailable metric data must not be treated as zero.

### 4.7 Upgrade and downgrade

The upgrade and downgrade semantics in `package-capability-matrix.md` are part of this decision.

In particular:

- retained canonical metrics may become visible when serving capability is gained;
- a package upgrade does not automatically bulk-export historical metrics to OPERAS;
- historical OPERAS export requires a separately scoped, reviewed and explicitly
  activated backfill;
- a downgrade to OBELISK stops publisher import, dashboard, widget and OPERAS
  export behaviour, while validly configured private background collection may
  continue;
- a downgrade to OASIS stops Thoth-managed collection as well as publisher
  import, dashboard, widget and OPERAS export behaviour;
- neither downgrade deletes retained canonical metrics;
- package changes never modify distribution-platform assignments.

## 5. Consequences

### Positive

- One entitlement source of truth.
- Compile-time exhaustive package handling.
- Consistent OAI and metrics behaviour.
- Simple authorization tests.
- Product changes are visible in reviewed releases.
- No capability/configuration drift between environments.
- No accidental package-to-distribution coupling.

### Negative

- Commercial mapping changes require code deployment.
- No bespoke per-publisher capability overrides.
- A future package model expansion requires enum, GraphQL and migration work.

### Risks

- OBELISK private collection has storage and operational cost even though the
  package has no metrics serving, import or export capability.
- Missing source configuration or source outages could accidentally couple
  collection to unrelated operations unless the non-blocking invariant is
  enforced.
- In-flight work could cross a downgrade unless entitlement is rechecked.

## 6. Invariants created by this decision

1. Package capability checks are centralized in Thoth.
2. Feature code does not hardcode package-name combinations.
3. Package changes do not change distribution assignments.
4. Capabilities do not bypass authorization or feature-specific configuration.
5. Canonical metrics are not deleted on downgrade.
6. Historical OPERAS export is an explicit operation, not an upgrade side effect.
7. Publisher users cannot mutate their package.
8. Capability mappings are identical across environments running the same version.
9. OASIS has no managed metrics collection.
10. OBELISK collection remains private, configured and non-blocking for
    unrelated operations.
11. Missing or unavailable metric data is not fabricated or treated as zero.

## 7. Implementation impact

Affected tasks:

- Publisher Services `BE-01`, `BE-03`, `APP-01`, `OAI-01`;
- metrics entitlement, service-authentication and export tasks.

Required sequencing:

1. approve this ADR and matrix;
2. implement package enum and capability mapping in `thoth`;
3. expose protected effective capabilities;
4. consume the capability checks in later programme slices;
5. remove any temporary package-name checks before production activation.

Required migrations:

- `thoth_package` enum;
- non-null publisher package column with OASIS default;
- no capability mapping table;
- no capability override table.

Required client changes:

- generate GraphQL clients after the protected capability field is merged;
- do not duplicate the matrix in frontend code.

Required operational changes:

- document capability-affecting releases;
- monitor rejected operations after upgrades/downgrades;
- use explicit historical export backfills.

## 8. Validation

Required evidence:

- exhaustive package/capability unit test;
- migration tests for existing and new publishers;
- authorization tests;
- downgrade and upgrade tests;
- tests proving distribution assignments are unchanged;
- tests proving metrics and OAI call capability methods;
- test proving no automatic historical export is created;
- GraphQL compatibility review.

## 9. Rollout and rollback

Rollout:

1. deploy package storage and capability mapping additively;
2. backfill/confirm OASIS defaults;
3. expose read-only package/capability data;
4. introduce feature checks in inactive or comparison mode;
5. activate each consuming feature under its own rollout.

Rollback:

- disable consuming feature paths;
- retain the package column and mapping code;
- revert mapping code only through a reviewed release;
- do not delete package history or canonical metrics.

## 10. Approval

Approval required from: CTO
Approved by: Javi, CTO
Approval date: 2026-07-28
Approval PR: [#772](https://github.com/thoth-pub/thoth/pull/772)
Notes: Approved with the final matrix and operational decisions recorded above.
This approval does not state that implementation has started or completed.
