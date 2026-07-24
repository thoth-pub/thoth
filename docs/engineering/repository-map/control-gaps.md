# Repository Control Gaps

Status: Active findings  
Evidence date: 2026-07-24

## Blocking before programme implementation

### CG-01 - Incorrect Sphinx naming in authoritative design

The canonical component is `thoth-sphinx` / `Sphinx`.

The Thoth Metrics design uses an obsolete spelling for the Sphinx repository and component. Correct the source document and replace the Project source rather than retaining two versions.

### CG-02 - `thoth-sphinx` is empty

The repository has no workspace, CI, tests, branch topology, agent instructions or release boundary.

A bootstrap specification is required.

### CG-03 - Branch topology differs from approved policy

See `branch-topology.md`.

Agents must use actual branches until explicit normalization tasks complete.

### CG-04 - Repository-local AI instructions missing

Only `thoth-dissemination` has an `AGENTS.md`.

Create repository-specific files from this approved map in a separate bounded task per repository or a coordinated documentation rollout.

## Required before affected production slices

### CG-05 - `thoth-app` CI is incomplete

Current GitHub CI runs coverage tests but does not independently run:

- lint;
- production build;
- GraphQL generation consistency.

Vercel build success is useful evidence but is not a replacement for explicit repository CI gates.

### CG-06 - `metrics-dashboard` lacks detected CI and tests

Before client migration:

- add CI;
- add at least service/data transformation tests;
- require lint and production build;
- add comparison fixtures for old/new data paths.

### CG-07 - `metrics-widget` lacks detected unit tests

Current CI covers lint, build and consumer smoke only.

Add tests for data fetching, coverage semantics, partial results and rendering before the canonical API migration.

### CG-08 - `cc-license` CI uses old actions

Modernize checkout/toolchain actions before or alongside LIC-01, without mixing functional licence changes into the CI-only PR.

### CG-09 - Thoth schema generation is unclear

`thoth-api/src/schema.rs` exists, but the root `diesel.toml` declares `src/schema.rs`.

Confirm and document the canonical command and working directory before metrics or publisher-service migrations.

### CG-10 - Thoth runtime operations are not mapped

Document:

- runtime platform;
- deployment trigger;
- database migration execution;
- rollback;
- backup/restore verification;
- production approvers.

## Verification gaps

The connector did not verify:

- branch-protection rules;
- required GitHub checks;
- environment reviewers;
- current crate publication process;
- all secret names or owners;
- production database version/configuration;
- Thoth API/export hosting.

Missing evidence is missing work.
