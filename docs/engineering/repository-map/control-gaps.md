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

### CG-04 - Repository-local AI instructions are incomplete

The `thoth` repository instruction hierarchy is included in the engineering-control foundation PR.

`thoth-dissemination` has an existing root `AGENTS.md`, but it does not yet include the complete control, authorization, operational and review requirements.

The following repositories still need root instructions:

- `thoth-app`
- `thoth-sphinx`
- `metrics-dashboard`
- `metrics-widget`
- `cc-license`

Track the rollout in `docs/engineering/agent-instructions/rollout-plan.md`.

### CG-05 - Shared package and platform ADRs require CTO approval

The following complete proposals exist:

- `docs/engineering/decisions/ADR-0001-publisher-package-capability-model.md`
- `docs/engineering/decisions/ADR-0002-platform-domain-boundaries.md`

They remain `PROPOSED`.

Do not start implementation that depends on either decision until the CTO:

- approves or amends the package-capability matrix;
- approves upgrade, downgrade and historical OPERAS export semantics;
- approves strict separation of distribution and metrics platform types;
- records the approval in the ADRs.

### CG-06 - Publisher Services control foundation is not complete

The required repository documents exist under `docs/publisher-services/`.

P0-01 remains incomplete until:

- the master GitHub issue is created from `master-issue.md`;
- its number is recorded in `task-status.md`;
- PR #764 receives independent approval and merges;
- ADR-0001 and ADR-0002 are approved or remain explicit implementation blockers.

Publisher Services ADR-01 must still finalize the distribution enum. The current platform inventory is a verified baseline, not an approved enum.

## Required before affected production slices

### CG-07 - `thoth-app` CI is incomplete

Current GitHub CI runs coverage tests but does not independently run:

- lint;
- production build;
- GraphQL generation consistency.

Vercel build success is useful evidence but is not a replacement for explicit repository CI gates.

### CG-08 - `metrics-dashboard` lacks detected CI and tests

Before client migration:

- add CI;
- add at least service/data transformation tests;
- require lint and production build;
- add comparison fixtures for old/new data paths.

### CG-09 - `metrics-widget` lacks detected unit tests

Current CI covers lint, build and consumer smoke only.

Add tests for data fetching, coverage semantics, partial results and rendering before the canonical API migration.

### CG-10 - `cc-license` CI uses old actions

Modernize checkout/toolchain actions before or alongside LIC-01, without mixing functional licence changes into the CI-only PR.

### CG-11 - Thoth schema generation is unclear

`thoth-api/src/schema.rs` exists, but the root `diesel.toml` declares `src/schema.rs`.

Confirm and document the canonical command and working directory before metrics or publisher-service migrations.

### CG-12 - Thoth runtime operations are not mapped

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
