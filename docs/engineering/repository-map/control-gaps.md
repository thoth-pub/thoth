# Repository Control Gaps

Status: Active findings  
Evidence date: 2026-07-24

## Blocking before programme implementation

### CG-01 - Incorrect Sphinx naming in authoritative design

Canonical: `thoth-sphinx` / `Sphinx`. Correct the Metrics design source and replace the Project source.

### CG-02 - `thoth-sphinx` is empty

No workspace, CI, tests, branch topology, instructions or release boundary. Bootstrap before WP6/drivers.

### CG-03 - Branch topology differs

Use actual branches until normalization/exception tasks complete.

### CG-04 - Repository-local instructions incomplete

Thoth hierarchy is in PR #764. Dissemination has partial instructions. App, Sphinx, dashboard, widget and cc-license still need root instructions.

### CG-05 - Shared ADRs unapproved

ADR-0001 and ADR-0002 remain proposed. Do not start dependent implementation.

### CG-06 - Publisher Services controls incomplete

Documents exist, but P0-01 needs master issue, recorded number, independent approval/merge and final ADR blockers. ADR-01 must finalize distribution enum.

### CG-07 - Metrics controls incomplete

Documents exist, but MET-CTRL-01 needs master issue, recorded number, independent approval/merge and final ADR blockers. Implementation remains blocked on Sphinx bootstrap, Diesel control and branch readiness.

## Required before production slices

### CG-08 - Metrics service roles unapproved

Approve role codes/scope/rotation/audit before WP5. Do not use superuser as a shortcut.

### CG-09 - Source fixtures/mappings incomplete

Representative period/dimension/regenerated files, COUNTER mappings, finalization settings and OPERAS projections are missing.

### CG-10 - OPERAS inbound completeness unavailable

No verified cursor, replication or complete snapshot. Rolling scans must state unverified completeness.

### CG-11 - `thoth-app` CI incomplete

Missing explicit lint, production build and codegen verification.

### CG-12 - Dashboard CI/tests missing

Add CI, transformation tests, lint/build and comparison fixtures.

### CG-13 - Widget unit tests missing

Add data/coverage/partial/rendering tests before migration/publication.

### CG-14 - cc-license CI old

Modernize in a bounded task.

### CG-15 - Thoth schema generation unclear

Resolve `thoth-api/src/schema.rs` vs root `diesel.toml` before migrations.

### CG-16 - Thoth runtime operations unmapped

Document runtime, deployment, migration execution, rollback, backup/restore and approvers.

## Verification gaps

Branch protections, required checks, environment reviewers, crate publication, secret ownership, production DB config and Thoth hosting/rollback remain unverified. Missing evidence is missing work.
