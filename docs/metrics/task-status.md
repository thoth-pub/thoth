# Thoth Metrics Task Status

Status: ACTIVE TRACKER
Programme owner: CTO
Master issue: [#766](https://github.com/thoth-pub/thoth/issues/766)
Approved design: [private Google Doc](https://docs.google.com/document/d/11AeQFGpm0kUZajBM5PrAqsttmzJlpUrt89tGYyVM8c0/edit), Drive revision `6`
Last updated: 2026-07-27

## 1. Control rule

A work package is not one implementation task. Each must be decomposed into bounded repository-local tasks with an approved specification, one slice branch/PR, actual base/target, risk, dependencies, tests, migration/rollout/rollback and independent review.

## 2. Foundation and readiness

| Task | Repository | Risk | Status | Base / target | Dependencies | Issue |
|---|---|---:|---|---|---|---|
| MET-CTRL-01 Programme controls | `thoth` | LOW | CHANGES REQUIRED | PR #764 merged into `develop` as `5b406e4ef9b5c192cc38eb8a97a41bbd0fc3bc06` | Shared foundation closed (P0-01 closeout PR #767 independently `APPROVED` and merged as `bac598e32abbd0d7e69ff467c82945ee00df02ba`); MET-CTRL-01's own `CHANGES REQUIRED` remediation outstanding | [#766](https://github.com/thoth-pub/thoth/issues/766) |
| ADR-0001 Package capability model | `thoth` | MEDIUM | PROPOSED | `develop` - proposal introduced by merged PR #764 | CTO decision | #766 |
| ADR-0002 Platform boundaries | `thoth` | MEDIUM | APPROVED | `develop` - proposal introduced by merged PR #764 | CTO approved 2026-07-27; approval PR [#769](https://github.com/thoth-pub/thoth/pull/769) | #766 |
| SPHINX-BOOT-01 Repository bootstrap | `thoth-sphinx` | MEDIUM | BLOCKED | current `develop`; target `develop` after BR-SPHINX-01 verification | MET-CTRL-01; BR-SPHINX-01; approved bootstrap spec | #766 |
| THOTH-DB-CTRL-01 Diesel generation procedure | `thoth` | MEDIUM | BLOCKED | `develop` -> `develop` | verified procedure | #766 |
| BR-DASH-01 Dashboard branch readiness | dashboard | HIGH | BLOCKED | observed `dev -> main`; reconcile stale `develop`, then normalize to `develop -> master` | Vercel rollback | #766 |
| BR-WIDGET-01 Widget branch readiness | widget | HIGH | BLOCKED | actual `dev`/`main` | npm release protection | #766 |
| BR-APP-01 App branch readiness | app | HIGH | BLOCKED | actual `dev`/`main` | Vercel branch plan | #766 |

## 3. Work packages

| WP | Scope | Repositories | Risk | Status | Blocking dependencies | Issue |
|---|---|---|---:|---|---|---|
| WP1 | Domain and database foundation | `thoth` | HIGH | BLOCKED | MET-CTRL-01; ADR-0001; Diesel control | #766 |
| WP2 | Canonical ingestion | `thoth` | CRITICAL | BLOCKED | WP1 | #766 |
| WP3 | Upload API and publisher UI | `thoth`, app | HIGH | BLOCKED | WP1/WP2; ADR-0001; BR-APP-01 | #766 |
| WP4 | Rollups and GraphQL | `thoth` | HIGH | BLOCKED | WP1/WP2; benchmark dataset | #766 |
| WP5 | Service auth and entitlements | `thoth`, clients | CRITICAL | BLOCKED | ADR-0001; role decision; WP4 | #766 |
| WP6 | Sphinx core | `thoth-sphinx` | HIGH | BLOCKED | bootstrap; pinned API contract | #766 |
| WP7 | CloudFront driver | `thoth-sphinx` | HIGH | BLOCKED | WP6; fixtures; methodology confirmation | #766 |
| WP8 | Additional drivers and COUNTER | Sphinx/app | HIGH | BLOCKED | WP6; source fixtures; COUNTER decision | #766 |
| WP9 | OPERAS and reconciliation | Thoth/Sphinx | CRITICAL | BLOCKED | WP1/WP2/WP6; mappings; completeness route | #766 |
| WP10 | Dashboard and widget clients | clients/Thoth | HIGH | BLOCKED | WP4/WP5; client CI/tests | #766 |
| WP11 | Deployment, monitoring, migration | multiple | CRITICAL | BLOCKED | WP1-WP10 | #766 |
| MET-E2E-01 | Integrated acceptance/cutover | multiple | CRITICAL | BLOCKED | all production slices | #766 |

## 4. Branch strategy

```text
develop -> feature/metrics -> feature/metrics/<slice> -> feature/metrics -> develop
```

Do not create integration branches until a verified `develop` branch and release-protection decision exist.

For `metrics-dashboard`, do not create `feature/metrics` from the stale
`develop` branch. BR-DASH-01 must first reconcile active `dev` history into the
target `develop` branch, or an explicit CTO exception must authorize another
verified base.

## 5. Immediate next actions

1. The shared foundation closeout is complete: PR #767 was independently
   `APPROVED` and merged as `bac598e32abbd0d7e69ff467c82945ee00df02ba`, closing
   P0-01, and the repository closeout record is reconciled. `MET-CTRL-01`
   remains `CHANGES REQUIRED` pending its own remediation.
2. `ADR-0002` platform domain boundaries is `APPROVED` (CTO, 2026-07-27, approval
   PR [#769](https://github.com/thoth-pub/thoth/pull/769)); this removes one
   shared-ADR dependency and does not make any work package ready.
3. Approve or amend `ADR-0001`.
4. Scope SPHINX-BOOT-01.
5. Resolve THOTH-DB-CTRL-01.
6. Prepare the first bounded WP1 slice only after those gates.
