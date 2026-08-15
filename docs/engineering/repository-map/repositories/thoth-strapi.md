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

Verified directly from `.github/workflows/build_docker.yml` and
`build_docker_release.yml` (`develop`, 2026-08-15). This repository has three
distinct, separately triggered GHCR-publishing paths — do not conflate them:

1. **PR creation/update -> automatic staging GHCR publication.**
   `build_docker.yml` triggers `on: pull_request` (every open/update event,
   with no path filter and no documentation-only classifier gating it,
   unlike `thoth-pub/thoth`'s own PR workflow). It unconditionally logs into
   `ghcr.io`, runs `docker/build-push-action@v5` with `push: true`, and tags
   the image `staging-pr-<PR number>` on `ghcr.io/thoth-pub/thoth-strapi`.
   Opening or updating any PR against this repository — including a
   documentation-only change — publishes a staging image automatically. This
   is not conditional on file paths changed.
2. **Release publication -> release GHCR publication.** `build_docker_release.yml`
   triggers `on: release: types: [published]` and pushes semver-tagged images
   (`{{version}}`, `{{major}}.{{minor}}`, `{{major}}`) to the same
   `ghcr.io/thoth-pub/thoth-strapi`. This only runs when a GitHub release is
   published, a separate, explicitly authorized action.
3. **Manual workflow dispatch -> separate mutation requiring explicit
   authorization.** `build_docker.yml` also declares `on: workflow_dispatch`,
   so it can additionally be triggered manually, independent of any PR. Manual
   dispatch is its own action under the granular action-authorization model
   (root `AGENTS.md` section 6) and is never authorized by PR-open/update
   authorization or by read/inspection authorization.

Verify current workflow files directly before relying on this record for a
task; do not dispatch either workflow manually without separate, explicit
authorization.

## Prohibited assumptions

- Do not assume this repository consumes the Thoth GraphQL API; no such
  dependency was found in its manifest.
- Do not assume its Thoth-ID-linkage content types are stable without
  verifying both this repository and `thoth-pub/thoth-pyramid` directly.
- Do not dispatch its Docker build/release workflows manually without
  explicit authorization.
- Do not assume opening or updating a PR against this repository is
  publication-free because a change is documentation-only or otherwise
  low-risk: `build_docker.yml` has no docs-only classifier and pushes a
  `staging-pr-*` image to `ghcr.io/thoth-pub/thoth-strapi` on every PR
  open/update, unconditionally.
