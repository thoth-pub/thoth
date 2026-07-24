# Repository: thoth-pub/thoth-sphinx

## Responsibility

Planned metrics collection, normalization, orchestration, rollup application, OPERAS synchronization and reconciliation.

## Current state

Visibility: private
GitHub default branch: `main`
Active development branch: `develop`
Repository content: placeholder-only `README.md` on `main` and `develop` as of 2026-07-24
CI: none
Deployment: none verified

Both branches exist and contain only the placeholder `README.md`. The repository remains non-implementation-ready because it has no workspace, implementation, CI, protection evidence or runtime.

## Canonical naming

Use:

```text
thoth-sphinx
Sphinx
```

The private Metrics design contains an obsolete spelling. Its exact Drive revision is recorded in `docs/engineering/design-references.md`.

## Required branch normalization

BR-SPHINX-01 must:

- create `master` from current `main`;
- retain and verify the existing `develop` branch;
- align `develop` with the approved bootstrap base;
- make `master` the release/default branch;
- protect `master` and `develop`;
- retain `main` until references are confirmed absent.

The resulting flow is:

```text
develop -> feature/metrics -> feature/metrics/<slice> -> feature/metrics -> develop -> master
```

## Planned stack and boundaries

The private Metrics design expects a Rust workspace with boundaries equivalent to:

- core normalized types and driver traits;
- Thoth GraphQL client;
- storage/manifests;
- OPERAS adapter;
- runner/orchestration;
- platform drivers.

Sphinx:

- does not own canonical metrics;
- does not write directly to PostgreSQL;
- does not keep durable state only in local files, SQLite, S3 or GitHub Actions;
- does not construct browser-facing credentials;
- does not submit source-driver payloads directly to OPERAS.

## Required bootstrap task

SPHINX-BOOT-01 must add:

- Cargo workspace;
- stable Rust toolchain policy;
- crate/module boundaries;
- replace or expand the placeholder README and add a root `AGENTS.md`;
- license;
- formatting, clippy and tests;
- GitHub CI and secret-scanning baseline;
- configuration conventions;
- no-op executable/driver test;
- branch/release documentation;
- no AWS resources and no production behaviour.

## Planned runtime

Not provisioned:

- ECS/Fargate;
- EventBridge;
- private S3;
- AWS OIDC for manual Actions;
- Secrets Manager/SSM;
- CloudWatch.

A bootstrap PR must not provision or deploy these automatically.
