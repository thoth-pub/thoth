# Thoth Metrics Task Status

Status: ACTIVE TRACKER
Programme owner: CTO
Master issue: [#766](https://github.com/thoth-pub/thoth/issues/766)
Approved design: [private Google Doc](https://docs.google.com/document/d/11AeQFGpm0kUZajBM5PrAqsttmzJlpUrt89tGYyVM8c0/edit), Drive revision `6`
Last updated: 2026-09-21 (`BR-DASH-01C-CONTROL-RECONCILIATION`, issue
[#931](https://github.com/thoth-pub/thoth/issues/931): the BR-DASH-01 row and
the branch strategy reconciled to the approved permanent `metrics-dashboard`
`feature/* -> dev -> main` topology, with each repository's Metrics flow
resolved through its own verified `<development-branch>`. Task statuses, risks,
WP entry gates and the other repositories' rows are unchanged). Previously
2026-09-09 (`CTRL-BRANCH-RELEASE-01`, issue
[#897](https://github.com/thoth-pub/thoth/issues/897): branch-readiness rows
reconciled to `ADR-0011`, which preserves each repository's established
release/default branch, so no `BR-` task converts a release branch. Task
statuses, risks, dependencies and the WP entry gates are unchanged. Previously
2026-08-25 (`MET-CTRL-01-CLOSEOUT-01`, issue
[#834](https://github.com/thoth-pub/thoth/issues/834)): `MET-CTRL-01` recorded
as `MERGED - COMPLETE` and its dependency satisfied, so WP1's remaining entry
gates are `feature/metrics` authorization and one approved bounded WP1 child
specification; later Sphinx/client/source/WP5 gates unchanged)

## 1. Control rule

A work package is not one implementation task. Each must be decomposed into bounded repository-local tasks with an approved specification, one slice branch/PR, actual base/target, risk, dependencies, tests, migration/rollout/rollback and independent review.

## 2. Foundation and readiness

| Task | Repository | Risk | Status | Base / target | Dependencies | Issue |
|---|---|---:|---|---|---|---|
| MET-CTRL-01 Programme controls | `thoth` | LOW | MERGED - COMPLETE | `develop` -> `develop` | Programme-control reconciliation delivered through PR [#833](https://github.com/thoth-pub/thoth/pull/833) and reachable from `develop`. The `MET-CTRL-01` dependency is satisfied and no longer gates WP1 entry. Shared foundation closed (P0-01 closeout PR #767 merged as `bac598e32abbd0d7e69ff467c82945ee00df02ba`). PR #833 is the parent lifecycle anchor; exact review and authorization provenance is retained in the owning task and closeout evidence, and this active tracker does not restate it | [#832](https://github.com/thoth-pub/thoth/issues/832) |
| ADR-0001 Package capability model | `thoth` | MEDIUM | APPROVED | `develop` - proposal introduced by merged PR #764 | CTO approved 2026-07-28; approval PR [#772](https://github.com/thoth-pub/thoth/pull/772) | #766 |
| ADR-0002 Platform boundaries | `thoth` | MEDIUM | APPROVED | `develop` - proposal introduced by merged PR #764 | CTO approved 2026-07-27; approval PR [#769](https://github.com/thoth-pub/thoth/pull/769) | #766 |
| CTRL-BRANCH-RELEASE-01 Preserve established release branch names | `thoth` | MEDIUM | APPROVED | `develop` at `4546cb632428872b961ad6c17282984d298e3ade` -> `develop` | Delivers [`ADR-0011`](../engineering/decisions/ADR-0011-preserve-established-release-branch-names.md). The exact ADR-0011 decision and task specification hold CTO approval under [#897](https://github.com/thoth-pub/thoth/issues/897). `APPROVED` is the durable decision state; it does **not** make ADR-0011 repository-authoritative. Repository authority additionally requires independent exact-head source review, separate merge authorization, and merge into and reachability from `develop`. Documentation/control only: no runtime, schema, migration, API, auth, workflow, settings, provider or production effect. GitHub remains authoritative for live pull-request, CI and merge state | [#897](https://github.com/thoth-pub/thoth/issues/897) |
| BR-SPHINX-01 Sphinx branch readiness | `thoth-sphinx` | MEDIUM | BLOCKED | observed `main` default, `develop` active; `main` preserved as `<release-branch>` | CTRL-BRANCH-RELEASE-01 / `ADR-0011` repository-authoritative, then re-specification; approved readiness spec. Protect `main` and `develop`, align `develop` with the approved bootstrap base, reconcile repository-local controls. No `master` is created and the default branch is not switched | #766 |
| SPHINX-BOOT-01 Repository bootstrap | `thoth-sphinx` | MEDIUM | BLOCKED | current `develop`; target `develop` after BR-SPHINX-01 verification | MET-CTRL-01 (**satisfied**); BR-SPHINX-01; approved bootstrap spec | #766 |
| THOTH-DB-CTRL-01 Diesel generation procedure | `thoth` | HIGH | SUPERSEDED | `develop` -> `develop` | Structural-synchronizer architecture superseded by ADR-0003; implementation PR #777 closed unmerged with no code becoming authoritative. Replaced by THOTH-DB-CTRL-02. | #766 |
| THOTH-DB-CTRL-02 Repository-authoritative schema contract | `thoth` | HIGH | MERGED - REPOSITORY-AUTHORITATIVE | `develop` at `4c53709befc91acb481beac54a1d314926b61d76` -> `develop` | Delivered ADR-0003 (Architecture A) and directly related cleanup through PR [#778](https://github.com/thoth-pub/thoth/pull/778), merged into `develop` as `37b802776ae6853affe19d90156f3c1e0654ebe3`. CG-12 is resolved and the shared Diesel schema-control dependency is satisfied. | #766 |
| BR-DASH-01 Dashboard branch readiness | dashboard | HIGH | BLOCKED | permanent `feature/* -> dev -> main`: `dev` is the approved development/integration branch and `main` is preserved as `<release-branch>`; no `dev -> develop` normalization. Stale legacy `develop` is not a workflow branch and has not been deleted | Separately authorized stale `develop` cleanup, branch protections and CI/lint/test readiness (CG-11); Vercel branch settings verified under separate provider-read authorization, with rollback, before any change that affects Vercel. Vercel production stays on `main` and is not moved | #766 |
| BR-WIDGET-01 Widget branch readiness | widget | HIGH | BLOCKED | actual `dev`/`main`; `main` preserved as `<release-branch>`; development branch normalizes to `develop` | npm release protection | #766 |
| BR-APP-01 App branch readiness | app | HIGH | BLOCKED | actual `dev`/`main`; `main` preserved as `<release-branch>`; development branch normalizes to `develop` | Vercel branch plan for previews and builds; production stays on `main` | #766 |

## 3. Work packages

| WP | Scope | Repositories | Risk | Status | Blocking dependencies | Issue |
|---|---|---|---:|---|---|---|
| WP1 | Domain and database foundation | `thoth` | HIGH | BLOCKED | separately authorized `feature/metrics` creation; approved bounded WP1 child specification (the MET-CTRL-01 dependency is satisfied) | #766 |
| WP2 | Canonical ingestion | `thoth` | CRITICAL | BLOCKED | WP1 | #766 |
| WP3 | Upload API and publisher UI | `thoth`, app | HIGH | BLOCKED | WP1/WP2; BR-APP-01; approved bounded slice specifications | #766 |
| WP4 | Rollups and GraphQL | `thoth` | HIGH | BLOCKED | WP1/WP2; benchmark dataset | #766 |
| WP5 | Service auth and entitlements | `thoth`, clients | CRITICAL | BLOCKED | shared machine-role convention settled: [`ADR-0008`](../engineering/decisions/ADR-0008-machine-roles-and-durable-job-primitives.md) is `APPROVED` and repository-authoritative (see the note below) and selects no Metrics role name, entitlement model, credential model or operation matrix — those remain WP5-owned bounded decisions; WP4; approved bounded slice specifications | #766 |
| WP6 | Sphinx core | `thoth-sphinx` | HIGH | BLOCKED | bootstrap; pinned API contract | #766 |
| WP7 | CloudFront driver | `thoth-sphinx` | HIGH | BLOCKED | WP6; fixtures; methodology confirmation | #766 |
| WP8 | Additional drivers and COUNTER | Sphinx/app | HIGH | BLOCKED | WP6; source fixtures; COUNTER decision | #766 |
| WP9 | OPERAS and reconciliation | Thoth/Sphinx | CRITICAL | BLOCKED | WP1/WP2/WP6; mappings; completeness route | #766 |
| WP10 | Dashboard and widget clients | clients/Thoth | HIGH | BLOCKED | WP4/WP5; client CI/tests | #766 |
| WP11 | Deployment, monitoring, migration | multiple | CRITICAL | BLOCKED | WP1-WP10 | #766 |
| MET-E2E-01 | Integrated acceptance/cutover | multiple | CRITICAL | BLOCKED | all production slices | #766 |

The shared architectural dependencies (ADR-0001, ADR-0002, ADR-0003,
ADR-0008) are satisfied and the Diesel/schema-control blocker is resolved.
The `MET-CTRL-01` programme-control dependency is also satisfied. WP1 remains
blocked only on its two remaining entry gates: separately authorized
`feature/metrics` creation and one approved bounded WP1 child specification.
Every later work package remains blocked by its own
listed repository-readiness, design, fixture, contract and
bounded-specification dependencies, which stay attached to those work
packages rather than blocking WP1 entry. No Metrics implementation package is
ready or authorized.

### 3.1 WP5 and the shared machine-role convention

WP5's dependency previously recorded as a bare "role decision" is the **shared
machine-role convention**. That question is decided by
[`ADR-0008`](../engineering/decisions/ADR-0008-machine-roles-and-durable-job-primitives.md)
— machine roles and durable job primitives — which the CTO approved on
2026-08-14. Under it, machine and service authorization in `thoth` uses
dedicated, least-privilege, **domain-specific** project roles: there is no
generic `SERVICE`/`MACHINE`/`WORKER`/`SERVICE_ACCOUNT` catch-all role, an
unscoped machine role is permitted only for a genuinely global workload, every
machine role requires an explicit policy guard, an explicit authorization matrix
and least privilege, and `SUPERUSER` authority does not automatically imply
machine-role authority. That `SUPERUSER`/machine-role boundary is the whole of
what `ADR-0008` decides about how roles relate: it states no general
role-composition, role-aggregation or role-inheritance rule.

Those requirements are the whole of the approved cross-programme machine-role
rule. Enumerated permitted-operation lists, enumerated forbidden-operation lists
and separate provisioning/credential controls are **not** approved `ADR-0008`
architecture; they bind Metrics only where existing repository, deployment or
identity-provider controls, or WP5's own approved bounded specification,
independently require them. `ADR-0008` decides no provisioning mechanism,
credential store, rotation policy or identity-provider arrangement.

**Authority condition — satisfied.** Under the repository's existing process
controls — not as approved decision content — `ADR-0008` resolves that shared
convention for Metrics when its exact approved content is
repository-authoritative on `develop`, that is, independently reviewed at its
exact head and merged. That condition is satisfied: the approved `ADR-0008`
record (delivered through `ADR-0008-RECORD`, PR
[#815](https://github.com/thoth-pub/thoth/pull/815)) is merged and reachable
from `develop`, so the shared machine-role convention dependency is resolved.

**What `ADR-0008` does not decide for Metrics.** It selects no Metrics
machine-role name, entitlement model, credential model or operation matrix.
Metrics chooses those under its own approved bounded specification while applying
the shared convention. `DISSEMINATION_WORKER` is a Publisher-Services-specific
role for the BE-04/DIS-02 durable distribution workflow; it is not a Metrics
role, confers no Metrics operation and determines no Metrics role name or
permissions. Metrics must not reuse `BE-04`'s durable job tables, Rust domain
types or lifecycle APIs by analogy, and a reusable generic cross-programme job or
queue abstraction would require its own explicit cross-programme ADR.

**WP5 status is unchanged.** WP5 remains `CRITICAL` and `BLOCKED`. It still
depends on WP4 and on its own approved bounded slice specifications, and no
Metrics implementation is authorized — by `ADR-0008` or otherwise.

## 4. Branch strategy

```text
<development-branch>
  -> feature/metrics
  -> feature/metrics--<slice>
  -> feature/metrics
  -> <development-branch>
```

`<development-branch>` is each affected repository's own verified, approved
development branch as recorded in
[`branch-topology.md`](../engineering/repository-map/branch-topology.md); it is
not assumed from `thoth`'s spelling. In `thoth` it is `develop`, so the Thoth
flow is:

```text
develop -> feature/metrics -> feature/metrics--<slice> -> feature/metrics -> develop
```

In `metrics-dashboard` it is `dev`, that repository's approved permanent
development branch.

Each affected repository owns its own `feature/metrics` integration branch.
Focused Metrics child branches are created from it and target it; they do not
target the `<development-branch>` directly. Under
[`ADR-0009`](../engineering/decisions/ADR-0009-programme-integration-branch-namespace.md)
the child branch is a **sibling** of the integration branch, separated by the
reserved `--` token. `feature/metrics/<slice>` is not usable beneath a live
`feature/metrics` branch, because Git cannot hold a ref and a ref namespace at
the same path. `ADR-0009` standardizes the repository ref spelling only; it does
not amend the substantive Metrics architecture.

Each repository's release branch is its own verified established release/default
branch. Under
[`ADR-0011`](../engineering/decisions/ADR-0011-preserve-established-release-branch-names.md)
that branch is preserved: `main` stays `main` and `master` stays `master`, and
no `BR-` readiness task converts a release branch. `ADR-0011` changes no Metrics
architecture and creates no branch.

Do not create a repository's integration branch until its verified, approved `<development-branch>` and a release-protection decision exist.

Before creating any Metrics branch, run the fail-closed namespace preflight in
`AGENTS.md` section 5.1 against live refs.

For `metrics-dashboard`, `feature/metrics` is created only from a verified
`dev` head, under separate branch-creation authorization, and never from the
stale legacy `develop` ref. Because `dev` is that repository's approved
permanent development branch, no `dev -> develop` reconciliation precedes it.
The stale `develop` ref has not been deleted; its deletion is a separately
authorized action.

## 5. Immediate next actions

1. The shared foundation closeout is complete: PR #767 was independently
   `APPROVED` and merged as `bac598e32abbd0d7e69ff467c82945ee00df02ba`, closing
   P0-01, and the repository closeout record is reconciled.
2. `ADR-0001` package capabilities is `APPROVED` and merged (Javi, CTO,
   2026-07-28, approval PR
   [#772](https://github.com/thoth-pub/thoth/pull/772)); `ADR-0002` platform
   domain boundaries is `APPROVED` and merged (CTO, 2026-07-27, approval PR
   [#769](https://github.com/thoth-pub/thoth/pull/769)). Neither makes any
   work package ready by itself.
3. The Diesel schema-control question (CG-12) is resolved: `ADR-0003`
   (Architecture A) is repository-authoritative. `THOTH-DB-CTRL-01` is
   `SUPERSEDED`; its replacement `THOTH-DB-CTRL-02` delivered ADR-0003
   through PR [#778](https://github.com/thoth-pub/thoth/pull/778), merged
   into `develop` as `37b802776ae6853affe19d90156f3c1e0654ebe3`.
4. `MET-CTRL-01` (issue
   [#832](https://github.com/thoth-pub/thoth/issues/832)) is
   `MERGED - COMPLETE` through PR
   [#833](https://github.com/thoth-pub/thoth/pull/833); its dependency is
   satisfied and no further programme-control gate stands before WP1 entry.
5. Enter WP1 only through its two remaining gates: verify the fresh `develop`
   head and separately authorize creation of repository-local
   `feature/metrics` from that exact head; then create and approve one
   bounded repository-local WP1 child issue/specification and implement that
   slice on a child branch targeting `feature/metrics`. Neither the branch nor
   the child specification exists, and neither is authorized by this record.
6. Scope SPHINX-BOOT-01 (with BR-SPHINX-01) for WP6 and later Sphinx work, on
   its own path; it does not gate Thoth WP1 entry. The Sphinx lane order is:

   ```text
   CTRL-BRANCH-RELEASE-01
     -> BR-SPHINX-01
     -> SPHINX-BOOT-01
     -> MET-WP6-01
   ```

   `BR-SPHINX-01` must be re-specified against `ADR-0011` — preserving `main`,
   protecting `main` and `develop`, and creating no `master` — and may only be
   approved once `ADR-0011` is repository-authoritative, which requires
   independent exact-head review of `CTRL-BRANCH-RELEASE-01` and merge into
   `develop`. None of those tasks is authorized by this record.
