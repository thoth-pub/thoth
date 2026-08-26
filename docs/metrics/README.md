# Thoth Metrics

Status: PROGRAMME CONTROLS RECONCILED - IMPLEMENTATION NOT AUTHORIZED
Programme owner: CTO

Primary repositories:

- `thoth-pub/thoth`
- `thoth-pub/thoth-sphinx`
- `thoth-pub/metrics-dashboard`
- `thoth-pub/metrics-widget`

Related repository:

- `thoth-pub/thoth-app`

## 1. Purpose

This directory is the repository-backed control surface for making Thoth the canonical owner of usage and sales metrics.

It turns the [private approved Metrics design](https://docs.google.com/document/d/11AeQFGpm0kUZajBM5PrAqsttmzJlpUrt89tGYyVM8c0/edit), Drive revision `6`, into:

- an explicit decision summary;
- a bounded work-package tracker;
- a source and driver inventory;
- a cross-repository contract register;
- a historical migration inventory;
- acceptance evidence requirements;
- rollout, comparison, rollback and observation gates.

## 2. Target outcome

Thoth owns canonical metrics in its existing PostgreSQL database and GraphQL API.

Sphinx:

- discovers source units;
- retrieves or references source artifacts;
- normalizes source data;
- submits bounded idempotent batches to Thoth;
- applies restartable orchestration;
- delivers eligible OPERAS exports;
- imports only allowed OPERAS-only mappings;
- records reconciliation outcomes in Thoth.

Dashboard and widget clients stop querying OPERAS directly and use authenticated Thoth-owned server routes.

## 3. Canonical boundaries

### Thoth

Thoth alone decides whether an observation becomes canonical.

It owns metrics tables and constraints, identifier resolution, registries, publisher approvals, import state, hashes, duplicates, revisions, conflicts, coverage, rollups, authorization, entitlements, OPERAS ledgers and reconciliation records.

### Sphinx

Sphinx is stateless orchestration and interoperability.

It must not:

- write directly to PostgreSQL;
- keep canonical state in local SQLite or S3;
- decide canonical conflict winners outside Thoth;
- send driver output directly to OPERAS;
- store browser-facing service credentials;
- claim guaranteed OPERAS inbound completeness without a complete discovery mechanism.

### Clients

Dashboard and widget browser code must not hold Thoth machine credentials.

Use:

```text
Browser -> Thoth-owned server route -> authenticated Thoth GraphQL operation
```

## 4. Core invariants

1. Every accepted record resolves to a Thoth work by DOI.
2. Optional publication resolution uses ISBN or an unambiguous publication type.
3. Institutions resolve to existing ROR-backed Thoth institutions; ingestion never creates them.
4. Periods are half-open dates.
5. Usage values reject negatives.
6. Sales measures may allow signed integer units.
7. Canonical identity excludes source and value.
8. Publisher imports are final and cannot supersede accepted rows.
9. Managed sources may create audited revisions.
10. Missing or incomplete coverage is never represented as zero.
11. Directly collected platform/measure mappings are never imported back from OPERAS.
12. Current publisher, imprint and series attribution follows current Thoth metadata.
13. Durable leases, checkpoints, imports, exports and reconciliation live in Thoth.
14. Metrics code consumes capabilities rather than hardcoded package names.

## 5. Programme non-goals

The initial programme does not store raw clickstream events in Thoth, create a separate canonical metrics database, calculate sales revenue, let publishers submit managed platform data, create institutions from imports, use GitHub Actions as durable state, make OPERAS canonical, combine unlike measures, or expose unbounded public analytics queries.

## 6. Current programme decision

```text
BLOCKED FOR IMPLEMENTATION
```

No Metrics implementation is authorized. The Publisher Services -> Metrics
pivot is complete (recorded in
[#766](https://github.com/thoth-pub/thoth/issues/766) comment `5412873595`),
and the shared controls below are settled.

Completed shared/global controls:

- the shared engineering-control foundation is closed: PR #764 merged, and its
  closeout PR #767 was independently `APPROVED` and merged as
  `bac598e32abbd0d7e69ff467c82945ee00df02ba`, closing P0-01;
- `ADR-0001` publisher package capabilities is `APPROVED AND MERGED` (Javi,
  CTO, 2026-07-28, approval PR
  [#772](https://github.com/thoth-pub/thoth/pull/772), merged 2026-07-29 as
  `b2c91ff25b95ab0e10a477ff21dbd4702f5db8d4`). The shared package-capability
  dependency is satisfied without activating metrics collection, entitlement
  enforcement, serving, imports or exports;
- `ADR-0002` platform domain boundaries is `APPROVED` and merged (CTO,
  2026-07-27, approval PR
  [#769](https://github.com/thoth-pub/thoth/pull/769)). `MetricPlatform`
  remains separate from `DistributionPlatform` with no initial cross-domain
  mapping;
- the Diesel/schema-control blocker is resolved: `ADR-0003` (Architecture A)
  is repository-authoritative through `THOTH-DB-CTRL-02` and merged PR
  [#778](https://github.com/thoth-pub/thoth/pull/778) (merge commit
  `37b802776ae6853affe19d90156f3c1e0654ebe3`). `THOTH-DB-CTRL-01` is
  `SUPERSEDED`;
- `ADR-0008` machine roles and durable job primitives is `APPROVED` and
  repository-authoritative within its approved scope: it establishes the
  shared domain-specific least-privilege machine-role convention and seven
  durable-job conventions, and deliberately selects no Metrics role name,
  entitlement model, credential model or operation matrix;
- the Metrics programme-control reconciliation `MET-CTRL-01` (issue
  [#832](https://github.com/thoth-pub/thoth/issues/832)) is
  `MERGED - COMPLETE`: it was delivered through PR
  [#833](https://github.com/thoth-pub/thoth/pull/833) and is reachable from
  `develop`. The `MET-CTRL-01` dependency is satisfied and is no longer a
  Thoth WP1 entry gate. PR #833 is the parent lifecycle anchor; exact review
  and authorization provenance is retained in the owning task and closeout
  evidence, and this active tracker does not restate it.

Remaining Thoth-local gates for entering WP1:

1. separately authorized creation of the repository-local `feature/metrics`
   integration branch from a freshly verified `develop` head. No such branch
   exists, and this record does not authorize creating one.
2. one approved bounded repository-local WP1 child issue/specification. None
   exists, and this record does not authorize creating one.

Later gates, owned by their later work packages rather than blocking Thoth
WP1 entry:

- `thoth-sphinx` readiness (`BR-SPHINX-01`, `SPHINX-BOOT-01`) gates WP6 and
  later Sphinx work;
- client branch/CI readiness (`BR-DASH-01`, `BR-WIDGET-01`, `BR-APP-01`)
  gates the work packages that depend on those clients;
- representative source fixtures, COUNTER mappings and guaranteed OPERAS
  inbound discovery gate the applicable source-specific/driver/inbound work;
- exact Metrics service-role codes, permissions and credential/provisioning
  arrangements remain unapproved WP5-owned bounded decisions.

Discovery, benchmarking, fixture collection and task specification may
continue. This record authorizes no Metrics implementation: WP1 and every
later work package remain unauthorized until their own gates are satisfied.

## 7. Files

- `decisions.md`
- `task-status.md`
- `source-inventory.md`
- `contract-register.md`
- `migration-inventory.md`
- `acceptance-matrix.md`
- `rollout-plan.md`
- `master-issue.md`

Missing evidence is missing work.
