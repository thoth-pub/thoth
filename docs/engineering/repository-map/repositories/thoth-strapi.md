# Repository: thoth-pub/thoth-strapi

Evidence date: 2026-08-15

## Responsibility

Strapi CMS instance powering publisher-facing content for Thoth catalogue
platforms (repository description: "Strapi CMS to power a publisher
website").

## Visibility

Private.

## Branches

GitHub default/release: `main`
Active development: `develop`
Other observed branches: `feature/import-export-plugin`, `feature/series`
Target-policy state: normalization required if this repository is brought
under the `develop -> master` target topology in
`docs/engineering/repository-map/branch-topology.md`; no normalization task is
authorized by this record. Verify the current branch and PR target directly
before branching.

## Stack

- Strapi 4 (`@strapi/strapi` 4.20.1)
- `@strapi/plugin-i18n`, `@strapi/plugin-users-permissions`
- `@strapi/provider-email-amazon-ses`, `@strapi/provider-upload-aws-s3`
- Docker (`Dockerfile`, `Dockerfile.dev`, `docker-compose.dev.yml`)
- GitHub Actions (`build_docker.yml`, `build_docker_release.yml`)

## Contract relationships

Ownership: this repository owns the CMS content-type schema and
content-delivery API it exposes. Its `package.json` shows no Thoth GraphQL
client dependency; it is not a verified consumer of the Thoth API.

Its content types include Thoth-ID-linkage fields (for example
`src/components/elements/thoth-id.json`,
`src/components/config/thoth-publisher-id.json`,
`src/api/config/content-types/config/schema.json`,
`src/api/series-page/content-types/series-page/schema.json`,
`src/api/imprint-page/content-types/imprint-page/schema.json`) used to
correlate CMS content with Thoth catalogue records by ID.

Verified consumer: `thoth-pub/thoth-pyramid` reads this repository's
content-delivery API over HTTP (`STRAPI_URL`). See
`docs/engineering/repository-map/contracts.md` section 2.2.

A change to the content-type schema, the Thoth-ID-linkage fields, or the
content-delivery API shape is a contract change for `thoth-pub/thoth-pyramid`
and must be assessed under the cross-repository impact-analysis gate before
scope is approved.

## CI and release

GitHub Actions build and (on release) push a Docker image, per
`build_docker.yml` and `build_docker_release.yml`. Verify current workflow
files directly before relying on this record; do not dispatch these workflows
manually without separate authorization.

## Prohibited assumptions

- Do not assume this repository consumes the Thoth GraphQL API; no such
  dependency was found in its manifest.
- Do not assume its Thoth-ID-linkage content types are stable without
  verifying both this repository and `thoth-pub/thoth-pyramid` directly.
- Do not dispatch its Docker build/release workflows without explicit
  authorization.
