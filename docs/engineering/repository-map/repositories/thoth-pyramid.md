# Repository: thoth-pub/thoth-pyramid

Evidence date: 2026-08-15; CI trigger coverage verified 2026-08-16

## Responsibility

Core multi-site website framework for Thoth publisher and catalogue
platforms, per the repository description. Renders publisher-facing catalogue
sites.

## Visibility

Private.

## Branches

GitHub default/release: `main`
Active development: `dev`
Verified `dev` head: `8f2d6faf70aabea61d11cbf361f602b719f9b3e2` (2026-08-16)
Target-policy state: normalization required if this repository is brought
under the `develop -> master` target topology in
`docs/engineering/repository-map/branch-topology.md`; no normalization task is
authorized by this record. Verify the current branch and PR target directly
before branching.

## CI

Single workflow: `.github/workflows/ci.yml`, one `test` job running
`npm install`, `npm run lint` and `npm test`.

Verified trigger coverage (2026-08-16, at `dev`
`8f2d6faf70aabea61d11cbf361f602b719f9b3e2`):

```text
push:          main, dev
pull_request:  main, dev
```

Pushes to `dev` and pull requests targeting `dev` therefore run repository CI.
The earlier gap — where both filters were `main` only, so a pull request
targeting the active `dev` branch received no GitHub Actions CI at all — is
repaired, by `CTRL-PYRAMID-CI-01` (issue
[#16](https://github.com/thoth-pub/thoth-pyramid/issues/16), PR
[#17](https://github.com/thoth-pub/thoth-pyramid/pull/17)).

**This records trigger coverage only.** It is not a statement that CI coverage
is adequate, and it does not replace scope-appropriate local validation. The
following remain separate, open concerns and are not addressed by it:

- broader CI quality and the depth of the existing `test` job;
- the repository's formatting gap;
- dependency and build concerns;
- codegen concerns, including verifying the schema source before regenerating;
- branch normalization.

See [CG-11](../control-gaps.md#cg-11---ci-gaps).

## Stack

- TanStack Start (React)
- TanStack Router (file-based routing)
- Vite
- Tailwind CSS
- Vitest
- `graphql-request` and GraphQL Code Generator (`@graphql-codegen/cli`,
  `@graphql-codegen/client-preset`)
- `thoth-blocks` package dependency
- `metrics-widget` package dependency (`^2.0.1`, verified in `package.json`
  `dependencies`), owned by `thoth-pub/metrics-widget`

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

Also depends directly on the published npm package `metrics-widget` (`^2.0.1`
in `package.json` `dependencies`), owned by `thoth-pub/metrics-widget` — a
package/library interface dependency, not a network API call.

See `docs/engineering/repository-map/contracts.md` sections 2.1, 2.2 and 2.4.
A breaking change to the Thoth GraphQL schema/export format, to Strapi's
Thoth-ID-linkage content types or content-delivery API shape, or to
`metrics-widget`'s public package interface, is a contract change this
repository must account for.

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
- Do not assume the `metrics-widget` package interface is stable without
  verifying `thoth-pub/metrics-widget` directly; this record does not
  authorize or imply any `metrics-widget` implementation or control task.
