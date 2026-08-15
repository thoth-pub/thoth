# Repository: thoth-pub/thoth-app

## Responsibility

Authenticated publisher and staff management UI for Thoth metadata and administrative workflows.

## Branches

GitHub default/release: `main`
Development: `dev`
Observed release: `dev -> main`
Target: `develop -> master`

BR-APP-01 must normalize the branch topology before a long-lived programme integration branch is created, unless the CTO records a temporary exception.

## Stack

- Next.js 16 App Router
- React 19
- TypeScript 5.9
- MUI 7
- Tailwind CSS 4
- TanStack Query
- XState
- GraphQL Request and GraphQL Code Generator
- NextAuth and ZITADEL
- Vitest and Testing Library

Architecture follows Feature-Sliced Design.

## Mandatory orientation

Before editing:

- `DOCUMENTATION.md`
- `package.json`
- `codegen.ts`
- relevant `app/`, `src/entities/`, `src/features/`, `src/widgets/`, `src/shared/`
- auth and proxy configuration
- relevant tests
- `.github/workflows/test.yml`
- applicable API schema and task specification

## Commands

```bash
npm ci
npm run generate
npm run lint
npm test -- --run --coverage
npm run build
```

Run `npm run generate` when GraphQL documents or consumed schema fields change.

## Generated artefacts

GraphQL codegen:

- schema source: `https://api.test.thoth.pub/graphql`
- documents: `app/**/*.{ts,tsx}`, `src/**/*.{ts,tsx}`
- output: `gql/`

A cross-repository API slice must ensure the test API or pinned preview exposes the exact schema before regenerating.

## CI

Current GitHub CI:

```bash
npm ci
npm test -- --run --coverage
```

Lint, production build and codegen consistency are not current GitHub gates and must be run manually until [CG-11](../control-gaps.md#cg-11---ci-gaps) is resolved.

## Deployment

Vercel team: Thoth
Node: 22.x
Production domain: `admin.thoth.pub`
Production branch observed: `main`
Preview branch observed: `dev`

Changing branch topology requires coordinated Vercel configuration and rollback evidence.

## Contract relationships

Verified consumer of the Thoth GraphQL schema owned by `thoth-pub/thoth` (see
"Generated artefacts" above and
`docs/engineering/repository-map/contracts.md` section 2.1). A breaking
schema change requires this repository to regenerate types and is a
cross-repository impact that must be assessed before scope is approved.

## Programme effects

Publisher Services:

- read-only publisher package/platform view;
- superuser configuration UI;
- staff subscription report;
- API-backed licence options.

Metrics:

- publisher uploads;
- import status/errors;
- publisher-platform approvals;
- no browser service credentials.

## Prohibited assumptions

- Do not encode linked platform rules independently from backend descriptors.
- Do not ship machine credentials to browser code.
- Do not treat Vercel build success as complete test evidence.
- Do not regenerate types from an API contract different from the branch being implemented.
