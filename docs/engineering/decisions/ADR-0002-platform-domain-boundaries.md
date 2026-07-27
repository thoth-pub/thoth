# ADR-0002 - Distribution and Metrics Platform Domain Boundaries

Status: APPROVED
Date: 2026-07-24
Decision owner: CTO
Programmes affected: Publisher Services, Thoth Metrics
Repositories affected: `thoth`, `thoth-app`, `thoth-dissemination`, `thoth-sphinx`, `metrics-dashboard`, `metrics-widget`
Supersedes: None
Superseded by: None

## 1. Context

Publisher Services needs a closed set of destinations that can be enabled for a publisher and mapped exhaustively to delivery behaviour.

Thoth Metrics needs a registry of services where measured activity occurred, together with supported measures, grains, dimensions and acquisition routes.

Some names may appear in both domains, such as OAPEN or JSTOR. That does not make the concepts identical.

Examples:

- OAPEN and DOAB are separate but linked distribution destinations and currently share one delivery adapter.
- An OAPEN metrics report may represent activity on OAPEN only.
- A publisher website or CDN may be a metrics platform without being a distribution destination.
- OCLC KB may be a distribution feed without supplying metrics.

A universal platform enum would create false equivalence and force unrelated lifecycle rules into one type.

## 2. Decision drivers

- Correct domain semantics.
- Exhaustive distribution adapter mapping.
- Extensible metrics registry.
- No accidental OAPEN/DOAB metrics linkage.
- No package or UI code inferring behaviour from display names.
- Independent evolution of delivery and measurement.
- Explicit cross-domain relationships only when a real use case exists.

## 3. Options considered

### Option A - One universal platform enum

Use one enum for distribution, metrics, source accounts and reporting.

Advantages:

- one list of names;
- superficially simple UI reuse.

Disadvantages:

- conflates desired delivery with measured activity;
- forces metrics-only and distribution-only values into every consumer;
- makes OAPEN/DOAB linkage ambiguous;
- encourages name-based assumptions;
- couples migrations and release schedules;
- cannot represent metric platform configuration cleanly.

Decision: Rejected.

### Option B - Shared base enum with domain-specific wrappers

Create one common enum and wrap it as `DistributionPlatform` and `MetricPlatform`.

Advantages:

- type distinction at some call sites.

Disadvantages:

- still forces a single lifecycle and inventory;
- still cannot express metrics registry data;
- new values affect unrelated domains;
- wrappers do not remove semantic coupling.

Decision: Rejected.

### Option C - Independent domain types with no inferred mapping

Use a closed distribution enum and a metrics registry table with stable codes.

Advantages:

- each domain models its real lifecycle;
- distribution remains exhaustive;
- metrics remains configurable and dimension-aware;
- overlapping names do not imply shared behaviour;
- clear tests and ownership.

Disadvantages:

- two registries may use similar labels;
- any future cross-domain report requires explicit mapping.

Decision: Recommended.

### Option D - Independent types plus an initial mapping table

Create a cross-domain mapping immediately.

Advantages:

- ready for combined reporting.

Disadvantages:

- no approved initial use case;
- semantics and cardinality are not yet established;
- risks creating another premature source of truth.

Decision: Rejected for the initial implementation.

A later ADR may introduce a mapping after defining its concrete consumer and cardinality.

## 4. Decision

### 4.1 Separate types and storage

Publisher Services uses:

```text
DistributionPlatform
```

as a PostgreSQL, Rust and GraphQL enum.

Its code-owned descriptor defines:

- display name;
- linked distribution platforms;
- delivery adapter;
- back-catalogue behaviour;
- update/withdrawal support where applicable.

Thoth Metrics uses:

```text
MetricPlatform
```

backed by `metric_platform` rows with stable codes.

Its registry defines:

- display name;
- ownership class;
- enabled state;
- supported measures through platform/measure mappings;
- source accounts and acquisition routes.

### 4.2 No name-based conversion

There is no automatic conversion between the types.

Do not:

- cast enum names;
- compare display labels;
- reuse one GraphQL enum for both;
- assume identical codes imply a relationship;
- infer metrics ownership from distribution assignment;
- infer distribution eligibility from metric source configuration.

### 4.3 OAPEN and DOAB

Within the distribution domain:

- OAPEN and DOAB remain separate enum values;
- linked normalization ensures they are enabled/disabled together initially;
- one logical multi-target job maps to the shared adapter;
- enabling both must not upload twice.

Within the metrics domain:

- an OAPEN metric platform is independent;
- DOAB is not automatically created, linked or aggregated;
- any DOAB metrics support requires its own verified source and metric-platform decision.

### 4.4 API boundaries

Use separate GraphQL surfaces:

```text
distributionPlatformOptions
metricPlatforms
metricMeasures
```

The frontend consumes backend descriptors for each domain.

A shared visual label component may be used, but it must receive a domain-specific descriptor and must not own business rules.

### 4.5 Initial cross-domain mapping

No cross-domain mapping column or table is added in the initial implementation.

If a future requirement needs a relationship, the new ADR/task must define:

- the exact consumer;
- one-to-one, one-to-many or many-to-many cardinality;
- ownership and migration;
- whether the relation is historical or current;
- authorization and public visibility;
- effects on jobs, metrics queries and reconciliation.

Until then, the absence of a mapping is intentional.

## 5. Consequences

### Positive

- Correct separation of desired delivery and observed activity.
- Distribution adapter mapping remains exhaustive.
- Metrics platforms can evolve without Rust enum migrations for every source.
- Linked distribution behaviour cannot leak into metrics.
- Metrics-only and distribution-only platforms are natural.
- Cross-domain coupling requires explicit review.

### Negative

- Similar platform names exist in separate registries.
- Combined administration may need two selectors.
- Future cross-domain reports require an explicit mapping design.

### Risks

- Developers may still assume code-name equality.
- UI code may accidentally duplicate descriptors.
- A future mapping may be added informally without an ADR.

## 6. Invariants created by this decision

1. `DistributionPlatform` and `MetricPlatform` are different domain types.
2. One cannot be substituted for the other.
3. Display names are not identifiers.
4. OAPEN/DOAB linkage exists only in distribution configuration unless separately approved.
5. Package changes do not create or remove values in either platform registry.
6. Distribution assignments do not authorize metrics uploads.
7. Metrics source accounts do not enable dissemination.
8. No initial cross-domain mapping table exists.

## 7. Implementation impact

Publisher Services:

- finalize `DistributionPlatform` through its platform-inventory ADR;
- implement exhaustive descriptors and adapter mapping;
- expose separate distribution GraphQL options.

Thoth Metrics:

- implement `metric_platform` and `metric_platform_measure`;
- use stable data codes;
- keep source-account configuration separate from publisher distribution assignments.

Clients:

- use domain-specific generated types;
- do not merge option arrays by matching code or label;
- render linkage only from the relevant backend descriptor.

Migrations:

- distribution enum and assignment tables are separate from metric tables;
- no cross-domain foreign key or mapping table in the initial design.

## 8. Validation

Required evidence:

- compile-time exhaustive distribution descriptor coverage;
- tests that OAPEN/DOAB produce one logical delivery adapter execution;
- tests that metrics OAPEN is not automatically linked to DOAB;
- tests that distribution assignment does not grant metric import;
- tests that metric source configuration does not enable distribution;
- GraphQL schema review showing separate types;
- search proving no name-based conversion helpers were introduced.

## 9. Rollout and rollback

Rollout:

- implement each domain independently;
- keep new code additive and inactive;
- compare legacy distribution configuration before cutover;
- load metrics platform registry only with verified mappings.

Rollback:

- disable the affected domain feature;
- retain separate persisted configuration;
- do not merge registries as a rollback shortcut.

## 10. Approval

Approval required from: CTO
Approved by: Javi, CTO
Approval date: 2026-07-27
Notes: Approved as written. DistributionPlatform and MetricPlatform remain
separate domain types, with no initial cross-domain mapping.
