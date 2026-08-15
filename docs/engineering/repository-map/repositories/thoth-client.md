# Repository: thoth-pub/thoth-client (standalone Python client)

Evidence date: 2026-08-15

This is the **standalone** Python API client repository. It is distinct from
the internal Rust `thoth-client` workspace member inside `thoth-pub/thoth`
(`thoth-client/` in that repository, depended on only by
`thoth-export-server`). See
`docs/engineering/repository-map/contracts.md` section 1 for the explicit
distinction. Do not conflate the two.

## Responsibility

Public Python client library for Thoth's public GraphQL API, published to
PyPI as `thothlibrary`. Supports the Thoth GraphQL schema and authenticates
with personal access tokens.

## Visibility

Public.

## Branches

GitHub default/release: `master`
Active development: `develop`
Observed release flow: feature branches merge into `develop`; `develop` merges
into `master` at release (verified via `develop` 1 commit ahead of `master`,
last common commit `Merge branch 'release/v1.2.0'`, and recent merged PRs
targeting `develop`)
Other observed branches: `launch/v1.0.0` (historical release-prep branch)
Target-policy state: conforms to the `develop -> master` pattern used
elsewhere in Thoth; no normalization task recorded.

## Stack

- Python
- `requests`-based GraphQL client
- packaged and published to PyPI as `thothlibrary`

## Contract relationship

Consumes: the public Thoth GraphQL schema owned by `thoth-pub/thoth`. See
`docs/engineering/repository-map/contracts.md` section 2.1.

A breaking Thoth GraphQL schema change is a contract change for this
repository. Because this package is published externally to PyPI, a breaking
change requires its own versioned release here; downstream consumers of
`thothlibrary` are external and not fully enumerable from Thoth-repository
evidence alone.

## CI and release

Observed: GitHub Actions run tests; releases are prepared on `launch/*`
branches and tagged from `master`. Verify current CI workflow files directly
before relying on this record for a task.

## Prohibited assumptions

- Do not assume this repository is the same artefact as the internal Rust
  `thoth-client` crate in `thoth-pub/thoth`.
- Do not assume every consumer of the published `thothlibrary` package is
  known; treat schema-breaking changes as public-API-breaking by default.
- Do not modify this repository under a task authorized only for
  `thoth-pub/thoth`.
