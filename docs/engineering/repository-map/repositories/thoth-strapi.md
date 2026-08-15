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
distinct, separately triggered, publication-capable GHCR paths — do not
conflate them with each other, and do not conflate a path being triggered with
an image having been published:

1. **Qualifying pull-request events -> automatic staging GHCR publication
   attempt.** `build_docker.yml` declares `on: pull_request` with **no
   explicit `types:` list**, so it runs on GitHub's default `pull_request`
   activity types — `opened`, `reopened` and `synchronize` — and carries no
   `paths`/`paths-ignore` filter and no documentation-only classifier, unlike
   `thoth-pub/thoth`'s own PR workflow. It unconditionally logs into
   `ghcr.io`, runs `docker/build-push-action@v5` with `push: true`, and its
   configured tag is `staging-pr-<PR number>` on
   `ghcr.io/thoth-pub/thoth-strapi`.

   Separate the **trigger** from the **outcome**, because they are different
   facts with different evidence:

   - *Trigger (certain, from the workflow file).* Opening or reopening a pull
     request against this repository, or synchronizing/updating its head,
     automatically triggers the publication-capable `build_docker.yml`
     workflow — including for a documentation-only head change. There is no
     path filter and no documentation-only exemption, so a qualifying event is
     never exempted by the file paths it changes. Such an event is therefore a
     **publication-capable external side effect** and starts an **automatic
     publication attempt**, not merely a build or a check. Record the
     verified activity types rather than claiming that every possible
     pull-request metadata event triggers the workflow: labelling, assignment,
     review requests, edits to the title or body and similar activity types
     are not among the defaults and do not start a run.
   - *Outcome (must be observed, never assumed).* A run that completes
     successfully publishes the `staging-pr-*` staging image. A run may also
     fail **before** publication: the registry login and the image push are
     separate steps, and a failure in `Build and push` leaves nothing
     published while the login has already happened. Whether an image was
     actually published must be established from the observed run conclusion
     and step results for that specific run, and from the registry, rather
     than inferred from the fact that a qualifying pull-request event
     occurred.

   Observed example (verified 2026-08-15): both `pull_request` runs for PR
   [#5](https://github.com/thoth-pub/thoth-strapi/pull/5) — run IDs
   `31897054063` and `31898765737` — concluded `failure`, with
   `Login to Container registry` succeeding, `Build and push` failing and
   `Image digest` skipped. The publication attempt was triggered
   automatically in both cases; neither run reached publication. This is the
   distinction in practice: the attempt is guaranteed by the trigger, the
   published image is not.

   Neither reading weakens the authorization requirement. Because opening,
   reopening or synchronizing a pull request here is publication-capable, it
   requires explicit pull-request-mutation authorization for this repository
   under the granular, non-transitive action-authorization model (root
   `AGENTS.md` section 6), and read/inspection authorization never implies it.
   Manual `workflow_dispatch`, release publication, provider/runtime actions,
   deployment and production activation remain **distinct** actions, each
   separately authorized, and none of them is authorized by
   pull-request-mutation authorization.
2. **Release publication -> release GHCR publication.** `build_docker_release.yml`
   triggers `on: release: types: [published]` and pushes semver-tagged images
   (`{{version}}`, `{{major}}.{{minor}}`, `{{major}}`) to the same
   `ghcr.io/thoth-pub/thoth-strapi`. This only runs when a GitHub release is
   published, a separate, explicitly authorized action.
3. **Manual workflow dispatch -> separate mutation requiring explicit
   authorization.** `build_docker.yml` also declares `on: workflow_dispatch`
   separately from `pull_request`, so it can additionally be triggered
   manually, independent of any pull request. Manual dispatch is its own
   action under the granular action-authorization model (root `AGENTS.md`
   section 6) and is never authorized by pull-request-mutation authorization
   or by read/inspection authorization.

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
- Do not assume that opening or reopening a pull request against this
  repository, or synchronizing/updating its head, is publication-free because
  the change is documentation-only or otherwise low-risk: `build_docker.yml`
  has no path filter and no docs-only classifier, and attempts a
  `staging-pr-*` push to `ghcr.io/thoth-pub/thoth-strapi` on every qualifying
  event, unconditionally. Treat pull-request mutation here as a
  publication-capable action requiring explicit authorization.
- Conversely, do not assume the staging image **was** published because a
  qualifying pull-request event occurred, or because the workflow was
  triggered. The trigger is guaranteed; the publication is not. Observe the
  run conclusion, its step results and the registry before recording a
  publication as having occurred.
