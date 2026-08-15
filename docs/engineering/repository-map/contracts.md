# Repository Ownership and Contracts

Status: VERIFIED OBSERVED STATE
Evidence date: 2026-08-15

This record describes verified repository ownership and contract/consumer
relationships across the repositories in the repository map, to guide the
cross-repository impact-analysis gate in
`docs/engineering/ai-delivery/operating-model.md` section 4.1 and root
`AGENTS.md` section 6.1.

Ownership and consumer relationships here are derived from live repository
evidence (README/package manifests/configuration/generated-client
configuration), not inferred from repository names. Where a relationship
could not be verified from live evidence or an approved architecture record,
it is marked `UNVERIFIED` rather than guessed. Do not treat an `UNVERIFIED`
row as either confirmed or absent.

## 1. Standalone Python `thoth-pub/thoth-client` versus internal Rust `thoth-client`

These are two distinct things that must not be conflated:

1. **`thoth-pub/thoth-client`** (standalone repository) - a public Python
   package (`thothlibrary` on PyPI) that is an independent, external GraphQL
   client for Thoth's public API, used by third parties and other
   organisations' tooling. See
   `docs/engineering/repository-map/repositories/thoth-client.md`.
2. **`thoth-client`** (internal Rust workspace member inside
   `thoth-pub/thoth`) - a Cargo workspace crate at `thoth-client/` in this
   repository, depended on only by `thoth-export-server` within the same
   workspace (`thoth-export-server/Cargo.toml`), used to query the Thoth
   GraphQL API for export generation. It is not published independently and
   is documented in
   `docs/engineering/repository-map/repositories/thoth.md`.

There is no dependency between these two projects; they share a name because
both are GraphQL clients for the same API, built independently in different
languages for different purposes.

## 2. Contract ownership

### 2.1 Canonical PostgreSQL domain, migrations, GraphQL API and export formats

Owning repository: `thoth-pub/thoth`.

Verified consumers:

| Consumer | Contract consumed | Change requires consumer action? |
|---|---|---|
| `thoth-pub/thoth-app` | GraphQL schema (via `graphql-codegen` against `https://api.test.thoth.pub/graphql`, see `thoth-app/codegen.ts`) | YES for any breaking schema change; regenerate types and follow `docs/engineering/ai-delivery/branching-and-release-workflow.md` section 6 |
| `thoth-pub/thoth-pyramid` | GraphQL schema (via `graphql-codegen` against `https://api.test.thoth.pub/graphql`, see `thoth-pyramid/codegen.ts`) and the metadata export API (`META_API_URL`, server-only) | YES for a breaking schema or export-format change |
| `thoth-pub/thoth-dissemination` | Thoth API for location write-back and publisher/work discovery | YES for a breaking API change affecting location write-back |
| `thoth-pub/thoth-client` (standalone) | Public GraphQL schema | YES for a breaking schema change; this is a published third-party-facing client and requires its own versioned release |
| `thoth-export-server` (internal, same repository) | GraphQL schema, via the internal `thoth-client` crate | Reviewed in the same PR; not a cross-repository concern |
| `thoth-pub/thoth-sphinx` | Planned: Thoth GraphQL client, per the private Metrics design | `UNVERIFIED` — Sphinx has no implementation yet (see section 3); record as a future consumer only |

### 2.2 Strapi CMS content contracts

Owning repository: `thoth-pub/thoth-strapi` (Strapi 4 CMS, `package.json`
confirms `@strapi/strapi` dependency; not a Thoth API consumer itself — no
Thoth GraphQL client dependency was found in its manifest).

Verified consumer: `thoth-pub/thoth-pyramid` reads Strapi content over HTTP,
configured via `STRAPI_URL`/`STRAPI_REQUEST_ORIGIN` in `.env.example`.
`thoth-strapi` content types include Thoth-ID-linkage fields (for example
`src/components/elements/thoth-id.json`,
`src/components/config/thoth-publisher-id.json`) used to correlate CMS
content with Thoth catalogue records by ID; this is a content-linkage
contract, not a live API call from Strapi into Thoth.

A change to those Thoth-ID-linkage content types, or to `thoth-strapi`'s
public content-delivery API shape, is a contract change that
`thoth-pub/thoth-pyramid` consumes.

### 2.3 Package/library interfaces

- `thoth-pub/thoth-client` (standalone, Python) publishes `thothlibrary` to
  PyPI; consumers are external and not fully enumerable from this
  repository's evidence. Treat any breaking change as public-API-breaking by
  default.

## 3. Repositories with no implementation yet

`thoth-pub/thoth-sphinx` has no implementation, CI or runtime as of this
record (see
`docs/engineering/repository-map/repositories/thoth-sphinx.md`). It is not
currently a live consumer of any contract. Its planned role as a Thoth
GraphQL client and OPERAS synchronizer is architecture, not a verified
current dependency, and must not be treated as an active consumer requiring
compatibility action until it exists.

## 4. Using this record

For a substantive or contract-affecting task:

1. identify the contract being changed from section 2 above;
2. list every verified consumer of that contract;
3. for each consumer, either create/reference a downstream repository-local
   task or record why it remains compatible;
4. do not guess a relationship not listed here — verify it live and, if it
   cannot be verified, mark it `UNVERIFIED` and escalate rather than assume
   either direction.

This record does not replace live verification. Re-verify before relying on
it for a HIGH or CRITICAL risk task, and correct it through a reviewed
documentation task when it is found stale.
