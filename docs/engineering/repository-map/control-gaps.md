# Repository Control Gaps

Status: ACTIVE FINDINGS
Evidence date: 2026-07-24

## Foundation closeout

### CG-01 - Independent review of PR #764

The engineering-control foundation has implementation and remediation work but is not independently approved. Required: completed implementation report, final diff/CI review, finding resolution, CTO merge approval and merge into `develop`.

### CG-02 - External Metrics design spelling

The private Metrics design contains an obsolete repository/component spelling. The exact Google Doc and Drive revision are recorded in `docs/engineering/design-references.md`. Repository documents use `thoth-sphinx` / `Sphinx`. Correcting the private source remains a source-owner follow-up and does not imply a second component.

## Blocking before programme implementation

### CG-03 - `thoth-sphinx` is placeholder-only

`main` and `develop` exist and contain only a placeholder README. The repository has no workspace, implementation, CI, protections or runtime. Complete BR-SPHINX-01 and SPHINX-BOOT-01 before WP6 or driver work.

### CG-04 - Branch topology differs

Use verified actual branches until normalization or explicit exceptions complete. Publisher Services uses standard task branches. Metrics uses repository-local integration branches only after readiness.

### CG-05 - Related repositories lack complete instructions

App, Sphinx, dashboard, widget and cc-license remain outstanding. Dissemination has incomplete controls.

### CG-06 - Shared ADRs remain proposed

ADR-0001 and ADR-0002 require explicit CTO approval and independent review before dependent implementation.

### CG-07 - Publisher Services platform ADR open

Issue #765 exists. ADR-01 must finalize enum values, mechanisms and ambiguous destinations.

### CG-08 - Metrics readiness open

Issue #766 exists. Sphinx normalization/bootstrap, Diesel control, branch readiness and service-role decisions remain prerequisites.

## Production-slice controls

### CG-09 - Source fixtures/mappings incomplete

Metrics fixtures, COUNTER mappings, finalization settings and OPERAS projections are missing.

### CG-10 - OPERAS inbound completeness unavailable

No verified complete cursor, replication or snapshot route exists.

### CG-11 - CI gaps

App lacks explicit lint/build/codegen; dashboard lacks detected CI/tests; widget lacks unit tests; cc-license uses old Actions.

### CG-12 - Thoth schema generation unclear

Resolve `thoth-api/src/schema.rs` versus root `diesel.toml` before schema changes.

### CG-13 - Thoth runtime operations unmapped

Document runtime, deployment, migration execution, rollback, restore verification and approvers.

## Verification gaps

Branch protections, required checks, environment reviewers, crate publication, secret ownership, production database configuration and Thoth hosting/rollback remain unverified. Missing evidence is missing work.
