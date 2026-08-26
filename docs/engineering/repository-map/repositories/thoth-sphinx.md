# Repository: thoth-pub/thoth-sphinx

## Responsibility

Planned metrics collection, normalization, orchestration, rollup application, OPERAS synchronization and reconciliation.

## Current state

Visibility: private
GitHub default branch: `main`
Active development branch: `develop`
CI: none — `actions/workflows` reports zero workflows on the repository
Deployment: none verified

Repository content, re-verified live 2026-08-15 after the repository's own
control-reconciliation work completed. This supersedes any earlier record
describing `main` and `develop` as identical and placeholder-only; that
description is not accurate:

- `main` remains the GitHub default branch, is at
  `0896e4061e06bc640f917f1aaf25c14b6e25269a`, and remains the original
  placeholder commit: `README.md` and nothing else;
- `develop` is the active development branch, is at
  `ff7de985d03f0c94d5ad8d60727f9cf85b6435cd`, and contains a root `AGENTS.md`
  plus the same, unchanged placeholder `README.md` (identical `README.md` blob
  `ba19f1ba2a27adc8640691aea407aeb51b1a6f32` on both branches);
- `main` and `develop` are therefore **not** identical:
  `compare/main...develop` reports `ahead_by: 8, behind_by: 0,
  status: ahead`, and the root `AGENTS.md` is the only content difference
  between the two branches.

`develop` has diverged from `main` solely through completed
repository-control and reconciliation work: the commits that added the
repository-local root `AGENTS.md` and subsequently corrected its recorded
content (`CTRL-REPO-SPHINX-01` and its reconciliation follow-ups, merged
through that repository's own pull requests). That divergence is
repository-control history. It is **not** branch normalization, and it is not
runtime or bootstrap implementation: no `master` branch has been established
by it, and no runtime, bootstrap, Cargo, CI or provider implementation exists
on either branch — there is no Cargo workspace, no crate or module structure,
no GitHub Actions workflow, no protection evidence and no provisioned runtime.

The repository therefore remains **bootstrap-only and
non-implementation-ready**. Branch normalization (`BR-SPHINX-01`) and
bootstrap (`SPHINX-BOOT-01`) remain separate, separately scoped and separately
authorized tasks, and both remain unimplemented. Neither is performed nor
authorized by this record, and this record performs no branch normalization.

It is not currently a verified consumer of any contract; see
`docs/engineering/repository-map/contracts.md` section 3.

## Canonical naming

Use:

```text
thoth-sphinx
Sphinx
```

The private Metrics design contains an obsolete spelling. Its exact Drive revision is recorded in `docs/engineering/design-references.md`.

## Required branch normalization

BR-SPHINX-01 is a separate, separately authorized task. It remains distinct
from SPHINX-BOOT-01 and is not performed by this record. It must:

- create `master` from current `main`;
- retain and verify the existing `develop` branch;
- align `develop` with the approved bootstrap base, preserving the root
  `AGENTS.md` already present on `develop`;
- make `master` the release/default branch;
- protect `master` and `develop`;
- retain `main` until references are confirmed absent.

Because `main` is currently behind `develop` by the repository-control and
reconciliation commits described above, any normalization plan must state
explicitly what `master` created from `main` contains, and must not assume the
two branches are interchangeable.

The resulting flow is:

```text
develop -> feature/metrics -> feature/metrics--<slice> -> feature/metrics -> develop -> master
```

Focused Metrics child branches are created from `feature/metrics` and target
`feature/metrics`; they do not target `develop` directly. Under
[`ADR-0009`](../../decisions/ADR-0009-programme-integration-branch-namespace.md)
the child branch is a **sibling** of the integration branch, separated by the
reserved `--` token; `feature/metrics/<slice>` is not usable beneath a live
`feature/metrics` branch. This is a forward-looking flow recorded in this
control repository; it is not authorization to mutate `thoth-pub/thoth-sphinx`.

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

SPHINX-BOOT-01 is a separate, separately authorized task, distinct from
BR-SPHINX-01, and is not performed or authorized by this record. It must add:

- Cargo workspace;
- stable Rust toolchain policy;
- crate/module boundaries;
- replace or expand the placeholder `README.md`;
- preserve and build on the **existing** root `AGENTS.md` already present on
  `develop` — amend it where bootstrap changes what it must say, but do not
  treat it as absent, re-add it as a new file, or overwrite it without
  reconciling the existing content;
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
