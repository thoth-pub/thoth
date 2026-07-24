# Repository: thoth-pub/metrics-dashboard

## Responsibility

Publisher-facing analytics dashboard.

Current implementation combines Thoth metadata with the OPERAS metrics API. The metrics programme will replace this with one authenticated Thoth-owned server-side query path.

## Branches

GitHub default/release: `main`  
Development: `develop`  
Observed release: `develop -> main`  
Target: `develop -> master`

BR-DASH-01 must normalize the release branch and Vercel production branch.

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

CG-06 must be resolved before the final client migration:

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
