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
   package (`thothlibrary` on PyPI) that is an independent, external client
   for **both** Thoth's public GraphQL API and its REST/export API, used by
   third parties and other organisations' tooling. See
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
| `thoth-pub/thoth-client` (standalone) | Public GraphQL schema **and** the Thoth REST/export API (verified: `thothlibrary/rest.py`, `thothlibrary/rest_cli.py`, `thothlibrary/rest_structures.py`, and README section "REST Usage" documenting `ThothRESTClient`) | YES for a breaking GraphQL schema change **or** a breaking REST/export API or export-format change; this is a published third-party-facing client and requires its own versioned release. Do not assume every export-format change is breaking — assess the specific change against `ThothRESTClient`'s documented usage. |
| `thoth-export-server` (internal, same repository) | GraphQL schema, via the internal `thoth-client` crate | Reviewed in the same PR; not a cross-repository concern |
| `thoth-pub/metrics-dashboard` | Public GraphQL schema (verified: `config/index.ts`, `url: process.env.NEXT_PUBLIC_THOTH_API_URL ?? 'https://api.thoth.pub/graphql'`) | YES for any breaking schema change |
| `thoth-pub/metrics-widget` | Public GraphQL schema (verified: `src/shared/config/index.ts`, `url: import.meta.env.VITE_THOTH_API_URL ?? 'https://api.thoth.pub/graphql'`) | YES for any breaking schema change |
| `thoth-pub/thoth-sphinx` | Planned: Thoth GraphQL client, per the private Metrics design | `UNVERIFIED` — Sphinx has no implementation yet (see section 3); record as a future consumer only |

**Current versus planned Metrics data path.** The rows above for
`metrics-dashboard` and `metrics-widget` record their **currently verified**
direct dependency on the public Thoth GraphQL API. The approved Thoth Metrics
architecture separately identifies both repositories as primary Metrics
client repositories and requires their **future** authenticated data path to
go through a protected Metrics GraphQL surface / BFF rather than the current
direct public-API call. That future architecture is not yet implemented and
does not change what is verified today: as of this record, both repositories
call the public Thoth GraphQL API directly, and a breaking change to that
public schema requires impact assessment for both today, independent of when
the protected Metrics path lands.

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
  PyPI, wrapping both the Thoth GraphQL API (`thothlibrary/graphql.py`,
  `client.py`, `query.py`, `mutation.py`) and the Thoth REST/export API
  (`thothlibrary/rest.py`, `rest_cli.py`, `rest_structures.py`); consumers are
  external and not fully enumerable from this repository's evidence. Treat any
  breaking GraphQL or REST/export change as public-API-breaking by default.
- `thoth-pub/thoth-pyramid` directly depends on the published npm package
  `metrics-widget` (`^2.0.1` in `thoth-pyramid/package.json`), owned by
  `thoth-pub/metrics-widget`. See section 2.4.

### 2.4 metrics-widget package dependency

Owning repository: `thoth-pub/metrics-widget`.

Verified consumer: `thoth-pub/thoth-pyramid` depends on the published
`metrics-widget` npm package (`^2.0.1`, verified in
`thoth-pyramid/package.json` `dependencies`). This is a package/library
interface dependency, not a network API call.

A breaking `metrics-widget` public API or package-interface change (for
example a breaking prop/export change, a peer-dependency bump incompatible
with Pyramid's stack, or a removed public entry point) is a contract change
that `thoth-pub/thoth-pyramid` must be assessed as a consumer of under the
cross-repository impact-analysis gate. This record does not start, authorize
or imply any `metrics-widget` implementation or control task; it records the
verified dependency only.

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
