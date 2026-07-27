# Thoth Metrics

Status: CONTROL FOUNDATION IN PROGRESS
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

Achieved:

- `ADR-0002` platform domain boundaries is `APPROVED` (CTO, 2026-07-27, approval
  PR [#769](https://github.com/thoth-pub/thoth/pull/769)) and is no longer a
  blocking control. `MetricPlatform` remains separate from `DistributionPlatform`
  with no initial cross-domain mapping. Approval removes one shared-ADR
  dependency and does not make any work package ready.

Blocking controls:

1. The shared engineering-control foundation is closed: PR #764 merged, and its
   closeout PR #767 was independently `APPROVED` and merged as
   `bac598e32abbd0d7e69ff467c82945ee00df02ba`, closing P0-01. The Metrics
   programme control task `MET-CTRL-01` nevertheless remains `CHANGES REQUIRED`
   and is not yet closed.
2. `ADR-0001` is `PROPOSED`.
3. `thoth-sphinx` is placeholder-only and has not been bootstrapped.
4. Thoth Diesel generation procedure is unresolved.
5. Branch topology differs in Sphinx and client repositories.
6. Service-role codes require approval before WP5.
7. Representative source fixtures and COUNTER mappings are missing for source-specific work.
8. Guaranteed OPERAS inbound discovery is unavailable.

Discovery, benchmarking, fixture collection and task specification may continue.

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
