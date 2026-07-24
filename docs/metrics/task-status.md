# Thoth Metrics Task Status

Status: ACTIVE TRACKER  
Programme owner: CTO  
Master issue: NOT YET CREATED  
Last updated: 2026-07-24

## 1. Control rule

A work package is not one implementation task. Each must be decomposed into bounded repository-local tasks with an approved specification, one slice branch/PR, actual base/target, risk, dependencies, tests, migration/rollout/rollback and independent review.

## 2. Foundation and readiness

| Task | Repository | Risk | Status | Base / target | Dependencies |
|---|---|---:|---|---|---|
| MET-CTRL-01 Programme controls | `thoth` | LOW | IN PROGRESS | PR #764 -> `develop` | independent review; master issue |
| ADR-0001 Package capability model | `thoth` | MEDIUM | PROPOSED | PR #764 -> `develop` | CTO decision |
| ADR-0002 Platform boundaries | `thoth` | MEDIUM | PROPOSED | PR #764 -> `develop` | CTO decision |
| SPHINX-BOOT-01 Repository bootstrap | `thoth-sphinx` | MEDIUM | BLOCKED | current `main`; approved bootstrap required | MET-CTRL-01; branch decision |
| THOTH-DB-CTRL-01 Diesel generation procedure | `thoth` | MEDIUM | BLOCKED | `develop` -> `develop` | verified procedure |
| BR-DASH-01 Dashboard branch readiness | dashboard | HIGH | BLOCKED | actual `develop`/`main` | Vercel rollback |
| BR-WIDGET-01 Widget branch readiness | widget | HIGH | BLOCKED | actual `dev`/`main` | npm release protection |
| BR-APP-01 App branch readiness | app | HIGH | BLOCKED | actual `dev`/`main` | Vercel branch plan |

## 3. Work packages

| WP | Scope | Repositories | Risk | Status | Blocking dependencies |
|---|---|---|---:|---|---|
| WP1 | Domain and database foundation | `thoth` | HIGH | BLOCKED | MET-CTRL-01; ADRs; Diesel control |
| WP2 | Canonical ingestion | `thoth` | CRITICAL | BLOCKED | WP1 |
| WP3 | Upload API and publisher UI | `thoth`, app | HIGH | BLOCKED | WP1/WP2; ADR-0001; BR-APP-01 |
| WP4 | Rollups and GraphQL | `thoth` | HIGH | BLOCKED | WP1/WP2; benchmark dataset |
| WP5 | Service auth and entitlements | `thoth`, clients | CRITICAL | BLOCKED | ADR-0001; role decision; WP4 |
| WP6 | Sphinx core | `thoth-sphinx` | HIGH | BLOCKED | bootstrap; pinned API contract |
| WP7 | CloudFront driver | `thoth-sphinx` | HIGH | BLOCKED | WP6; fixtures; methodology confirmation |
| WP8 | Additional drivers and COUNTER | Sphinx/app | HIGH | BLOCKED | WP6; source fixtures; COUNTER decision |
| WP9 | OPERAS and reconciliation | Thoth/Sphinx | CRITICAL | BLOCKED | WP1/WP2/WP6; mappings; completeness route |
| WP10 | Dashboard and widget clients | clients/Thoth | HIGH | BLOCKED | WP4/WP5; client CI/tests |
| WP11 | Deployment, monitoring, migration | multiple | CRITICAL | BLOCKED | WP1-WP10 |
| MET-E2E-01 | Integrated acceptance/cutover | multiple | CRITICAL | BLOCKED | all production slices |

## 4. Recommended decomposition

WP1 should be split into registry/seed, source/checkpoint/approval, import/error, record/revision/provenance/coverage, rollup, OPERAS ledgers and admin GraphQL slices.

WP2 should be split into normalized contract, identifier resolution, hashing, duplicate/conflict/revision transaction, overlap, publisher finality, bounded GraphQL ingestion and summaries/provenance.

WP4 should be split into rollup application, rebuild/watermark, entity fields, dashboard, widget, coverage, limits and evidence-led performance tuning.

WP6 should be split into workspace bootstrap completion, core, Thoth client, storage, runner, OPERAS boundary and no-op E2E.

## 5. Branch strategy

Target per repository:

```text
develop -> feature/metrics -> feature/metrics/<slice> -> feature/metrics -> develop
```

Do not create integration branches until verified `develop` and release protection exist.

## 6. Immediate next actions

1. Create the master issue.
2. Record its number.
3. Complete/review PR #764.
4. Approve or amend ADR-0001 and ADR-0002.
5. Scope SPHINX-BOOT-01.
6. Resolve THOTH-DB-CTRL-01.
7. Then prepare the first WP1 slice.
