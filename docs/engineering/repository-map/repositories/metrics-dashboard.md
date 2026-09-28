# Repository: thoth-pub/metrics-dashboard

Evidence date: 2026-07-24; branch heads and repository control re-verified
2026-08-16; permanent development/release topology recorded and branch heads
re-verified live 2026-09-21

## Responsibility

Publisher-facing analytics dashboard.

Current implementation combines Thoth metadata with the OPERAS metrics API. The metrics programme will replace this with one authenticated Thoth-owned server-side query path.

## Branches

GitHub default/release: `main`
Active development/integration: `dev`
Permanent workflow: `feature/* -> dev -> main`
Legacy stale ref: `develop` (not a workflow branch; not deleted)
Established `<release-branch>`: `main`, **preserved** (Vercel-backed)
`<development-branch>`: `dev`, approved permanent

Under [`ADR-0011`](../../decisions/ADR-0011-preserve-established-release-branch-names.md) `main` is this repository's established release/default branch and is preserved. No `master` branch is created, and Vercel production is **not** moved from `main` merely to adopt a different branch spelling.

`dev` is this repository's approved **permanent** development/integration
branch, and `main` remains its established release/default branch. The CTO
approved that topology as a repository-local development-branch decision, which
`ADR-0011` section 4.4 leaves independent of release-branch naming. There is
**no** planned `dev -> develop` normalization. The decision creates, renames,
moves and deletes no branch, and changes neither the release/default branch nor
Vercel production or default routing. It is reconciled into the shared controls
by `BR-DASH-01C-CONTROL-RECONCILIATION`
([thoth#931](https://github.com/thoth-pub/thoth/issues/931)). The
repository-local root `AGENTS.md` is reconciled to it separately by
`BR-DASH-01B`
([metrics-dashboard#12](https://github.com/thoth-pub/metrics-dashboard/issues/12));
GitHub records that task's live review and merge state.

Verified branch evidence (re-verified live 2026-09-21; all three heads are
unchanged since 2026-08-16):

- `main`: `92d90380e948b3f11f88821054fbad9a5a07f387`;
- `dev`: `963b0ea78a9a65153ab7d78b7c26e3cb35d763f4` (re-verified 2026-08-16;
  previously recorded as `1f81745e6d9e812baab62a19e41a0c0f3b9ff0c9`, which the
  repository-control merge below superseded);
- stale `develop`: `1619899076d16de81abf5c2c6abdd40d985512e6`;
- `dev` is 12 commits ahead of `develop`, with no commits behind, so `develop`
  is a strict stale ancestor of `dev`;
- `main`, `dev` and `develop` carry no branch protection and no ruleset.

Implementation work branches from the verified `dev` head and targets `dev`;
`dev` is released to `main`. Another base requires an explicit CTO exception.

Stale `develop` is a legacy ref, not a workflow branch. It must not be used as
an implementation base, a pull-request target or the source of
`feature/metrics`. It has **not** been deleted: its deletion is a separately
authorized repository-control action and is not authorized by this record.

Settling the topology closes only the development-branch question. The
remaining `BR-DASH-01` readiness work stays **open** and separate, each part
owned by its own separately scoped and separately authorized task:

- cleanup of the stale legacy `develop` ref;
- branch protection for `main` and `dev`;
- CI, tests and a lint and production-build gate
  ([CG-11](../control-gaps.md#cg-11---ci-gaps); see CI below).

Because the decision changes no branch, it requires no Vercel production or
default-routing change; no Vercel setting was read or changed for it. The
remaining task's risk is recorded in
[`branch-topology.md`](../branch-topology.md) section 5.

## Repository control

Repository-local root `AGENTS.md` merged onto `dev` through PR
[#10](https://github.com/thoth-pub/metrics-dashboard/pull/10), verified live
2026-08-16 at `963b0ea78a9a65153ab7d78b7c26e3cb35d763f4`. It is now part of this
repository's control surface: later work must read and preserve it rather than
add one as though absent.

That merge added the control file and nothing else. It did not add CI, add
tests or repair lint; each of those remains open below and in
[CG-11](../control-gaps.md#cg-11---ci-gaps). It changed no branch.

## Stack

- Next.js 16
- React 19
- TypeScript
- MUI
- TanStack Query
- GraphQL Request

## Mandatory orientation

Before editing:

- `README.md`
- `package.json`
- data services and query hooks
- chart/aggregation utilities
- export utilities
- environment configuration
- applicable metrics API specification

## Commands

```bash
npm ci
npm run lint
npm run build
```

No automated test script was detected.

## CI

No GitHub Actions workflow was detected.

[CG-11](../control-gaps.md#cg-11---ci-gaps) must be resolved before the final client migration:

- add CI;
- add tests for data transformations and failure semantics;
- add old/new comparison fixtures;
- require lint and production build.

## Deployment

Vercel team: Thoth
Node: 22.x
Production domain: `metrics.thoth.pub`
Production branch observed: `main`
Production branch under the permanent topology: `main`, unchanged

## Metrics migration invariants

- Browser code must not hold Thoth service credentials.
- Use a server-side route/BFF.
- Replace per-DOI OPERAS batching with bounded Thoth queries.
- Do not combine incompatible measures.
- Surface coverage, `dataThrough`, warnings and partial state.
- Dependency failure must not display as zero.
- Preserve a controlled fallback during the observation window.
