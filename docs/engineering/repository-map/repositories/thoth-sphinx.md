# Repository: thoth-pub/thoth-sphinx

## Responsibility

Planned metrics collection, normalization, orchestration, rollup application, OPERAS synchronization and reconciliation.

## Current state

Visibility: private  
GitHub default branch: `main`  
Repository content: empty as of 2026-07-24  
CI: none  
Deployment: none verified

The repository is not implementation-ready.

## Canonical naming

Use:

```text
thoth-sphinx
Sphinx
```

Use only `thoth-sphinx` and `Sphinx` in new material.

## Required branch bootstrap

BR-SPHINX-01 must establish:

```text
develop -> master
```

Task branches then follow:

```text
feature/metrics/<slice> -> feature/metrics -> develop -> master
```

## Planned stack and boundaries

The approved metrics design expects a Rust workspace with boundaries equivalent to:

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

The bootstrap must add:

- Cargo workspace;
- stable Rust toolchain policy;
- crate/module boundaries;
- README;
- AGENTS.md;
- license;
- formatting, clippy and tests;
- GitHub CI;
- secret-scanning baseline;
- configuration conventions;
- a no-op executable/driver test;
- branch/release documentation;
- no AWS resources and no production behaviour.

## Planned runtime

Not yet provisioned:

- ECS/Fargate;
- EventBridge;
- private S3;
- AWS OIDC for manual Actions;
- Secrets Manager/SSM;
- CloudWatch.

A bootstrap PR must not provision or deploy these automatically.
