# Repository: thoth-pub/metrics-dashboard

Evidence date: 2026-07-24; branch heads and repository control re-verified
2026-08-16

## Responsibility

Publisher-facing analytics dashboard.

Current implementation combines Thoth metadata with the OPERAS metrics API. The metrics programme will replace this with one authenticated Thoth-owned server-side query path.

## Branches

GitHub default/release: `main`
Active development: `dev`
Legacy stale branch: `develop`
Observed release: `dev -> main`
Target after BR-DASH-01: `develop -> master`

Verified branch evidence:

- `main`: `92d90380e948b3f11f88821054fbad9a5a07f387`;
- `dev`: `963b0ea78a9a65153ab7d78b7c26e3cb35d763f4` (re-verified 2026-08-16;
  previously recorded as `1f81745e6d9e812baab62a19e41a0c0f3b9ff0c9`, which the
  repository-control merge below superseded);
- stale `develop`: `1619899076d16de81abf5c2c6abdd40d985512e6`;
- `dev` is 12 commits ahead of `develop`, with no commits behind.

Until BR-DASH-01 reconciles the branches, implementation work must branch from
the verified `dev` branch. Another base requires an explicit CTO exception.

BR-DASH-01 must reconcile the active development history before normalizing the
development branch, release branch and Vercel configuration. PR #764 does not
perform that normalization, and neither does the repository-control merge
recorded below.

## Repository control

Repository-local root `AGENTS.md` merged onto `dev` through PR
[#10](https://github.com/thoth-pub/metrics-dashboard/pull/10), verified live
2026-08-16 at `963b0ea78a9a65153ab7d78b7c26e3cb35d763f4`. It is now part of this
repository's control surface: later work must read and preserve it rather than
add one as though absent.

That merge added the control file and nothing else. It did not add CI, add
tests, repair lint, or normalize any branch; each of those remains open below
and in [CG-11](../control-gaps.md#cg-11---ci-gaps).

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

## Metrics migration invariants

- Browser code must not hold Thoth service credentials.
- Use a server-side route/BFF.
- Replace per-DOI OPERAS batching with bounded Thoth queries.
- Do not combine incompatible measures.
- Surface coverage, `dataThrough`, warnings and partial state.
- Dependency failure must not display as zero.
- Preserve a controlled fallback during the observation window.
