# Repository: thoth-pub/thoth-pyramid

Evidence date: 2026-08-15

## Responsibility

Core multi-site website framework for Thoth publisher and catalogue
platforms, per the repository description. Renders publisher-facing catalogue
sites.

## Visibility

Private.

## Branches

GitHub default/release: `main`
Active development: `dev`
Target-policy state: normalization required if this repository is brought
under the `develop -> master` target topology in
`docs/engineering/repository-map/branch-topology.md`; no normalization task is
authorized by this record. Verify the current branch and PR target directly
before branching.

## Stack

- TanStack Start (React)
- TanStack Router (file-based routing)
- Vite
- Tailwind CSS
- Vitest
- `graphql-request` and GraphQL Code Generator (`@graphql-codegen/cli`,
  `@graphql-codegen/client-preset`)
- `thoth-blocks` package dependency

## Generated artefacts

GraphQL codegen (`codegen.ts`):

- schema source: `https://api.test.thoth.pub/graphql`
- documents: `src/entities/**/model/*.graphql`
- output: `src/shared/api/__generated__/`

A cross-repository API slice must ensure the test API or a pinned preview
exposes the exact schema before regenerating.

## Contract relationships

Consumes (verified from `.env.example` and `codegen.ts`):

- the Thoth GraphQL schema owned by `thoth-pub/thoth` (`VITE_API_URL` /
  codegen schema source);
- the Thoth metadata export API (`META_API_URL`, server-only), owned by
  `thoth-pub/thoth`;
- the `thoth-pub/thoth-strapi` CMS content API (`STRAPI_URL`,
  `STRAPI_REQUEST_ORIGIN`).

See `docs/engineering/repository-map/contracts.md` sections 2.1 and 2.2. A
breaking change to either the Thoth GraphQL schema/export format or to
Strapi's Thoth-ID-linkage content types or content-delivery API shape is a
contract change this repository must account for.

Other external dependencies observed in `.env.example`: Algolia (search),
Matomo (analytics, optional), Snipcart (cart/checkout, optional), and an
account-request email/HMAC flow. These are not Thoth-owned contracts.

## Prohibited assumptions

- Do not assume this repository shares `thoth`'s `develop`/`master` branch
  names; its active development branch is `dev`.
- Do not regenerate GraphQL types from a schema source different from the
  branch being implemented.
- Do not assume Strapi's content-delivery API or Thoth-ID-linkage content
  types are stable without verifying `thoth-strapi` directly.
