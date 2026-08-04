# Repository Control Gaps

Status: ACTIVE FINDINGS
Evidence date: 2026-07-29

## Foundation closeout

### CG-01 - Independent review of PR #764 (RESOLVED 2026-07-27)

PR #764 merged into `develop` as
`5b406e4ef9b5c192cc38eb8a97a41bbd0fc3bc06` without a submitted independent
review. This gap is now resolved retrospectively. After remediation, the
complete PR #764 foundation and PR #767 closeout received an independent
`APPROVED` review at content head
`d72137893ddea512c0d05c81d310eb59d045cd2b`, and PR #767 merged into `develop`
as `bac598e32abbd0d7e69ff467c82945ee00df02ba` on 2026-07-27, closing P0-01. The
merged repository is the authoritative P0-01 closure record. The earlier
`CHANGES REQUIRED` and `BLOCKED` review cycles are historical evidence only.

### CG-02 - External Metrics design spelling

The private Metrics design contains an obsolete repository/component spelling. The exact Google Doc and Drive revision are recorded in `docs/engineering/design-references.md`. Repository documents use `thoth-sphinx` / `Sphinx`. Correcting the private source remains a source-owner follow-up and does not imply a second component.

## Blocking before programme implementation

### CG-03 - `thoth-sphinx` is placeholder-only

`main` and `develop` exist and contain only a placeholder README. The repository has no workspace, implementation, CI, protections or runtime. Complete BR-SPHINX-01 and SPHINX-BOOT-01 before WP6 or driver work.

### CG-04 - Branch topology differs

Use verified actual branches until normalization or explicit exceptions complete. Publisher Services uses standard task branches. Metrics uses repository-local integration branches only after readiness.

### CG-05 - Related repositories lack complete instructions

App, Sphinx, dashboard, widget and cc-license remain outstanding. Dissemination has incomplete controls.

### CG-06 - Shared ADR approvals (RESOLVED 2026-07-29)

ADR-0002 was approved by the CTO on 2026-07-27 and recorded as `APPROVED` through
approval PR [#769](https://github.com/thoth-pub/thoth/pull/769); its independent
review and merge close this part of the gate. ADR-0001 was approved by the CTO on
2026-07-28 with the final OASIS/OBELISK collection distinction and recorded
through independently reviewed approval PR
[#772](https://github.com/thoth-pub/thoth/pull/772), which merged on 2026-07-29
as `b2c91ff25b95ab0e10a477ff21dbd4702f5db8d4`. Both approval records are
merged, so CG-06 is fully resolved and no remaining dependency requires PR #772
to merge. Dependent work still requires its own approved bounded specification
and remaining programme controls. Resolving this shared-ADR gap does not make
any implementation task ready.

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

### CG-12 - Thoth schema generation unclear (SPECIFICATION APPROVED - IMPLEMENTATION NOT STARTED)

Task `THOTH-DB-CTRL-01` is specified in
[`docs/engineering/ai-delivery/tasks/THOTH-DB-CTRL-01.md`](../ai-delivery/tasks/THOTH-DB-CTRL-01.md)
through specification PR
[#775](https://github.com/thoth-pub/thoth/pull/775). Disposable PostgreSQL 17
testing established that root `diesel.toml` is syntactically and semantically
invalid, its output path does not identify canonical
`thoth-api/src/schema.rs`, and raw Diesel output does not preserve the compiled
contract's aliases, supplemental type, timestamp semantics, column order, or
formatting.

The selected implementation is an exact-version, fail-closed structural
synchronizer: root `diesel.toml` remains authoritative,
`thoth-api/src/schema.rs` remains canonical, current intentional conventions
become explicit control data, clean generation is byte-identical, and only a
declared `change` or `none` projection expectation may pass.

The written specification was previously approved at pre-correction content.
That approval is historical after the normative enum-projection,
catalog-baseline, and projection-mode corrections.
The projection-mode-corrected written specification is approved by Javi, CTO,
on 2026-08-04, bound to exact base
`35e4dc20864ae4896dccc2b20cbcdbe3fb733db8`, exact reviewed head
`50ff3248b2af4a19422df924260c4f17832c0378`, normative content head
`aec8295f22bc8c7cab4ce13e09890ef78b8586fa`, and independent approval comment
`5177640752`. Specification approval does not authorize the implementation
branch or implementation work. PR #775 does not resolve CG-12 and does not
start the implementation. BE-01 remains `BLOCKED`. CG-12 closes only after
`THOTH-DB-CTRL-01` receives separate implementation authorization, passes its
complete acceptance evidence and independent exact-head review, and merges with
explicit CTO authorization. CG-13 remains open.

### CG-13 - Thoth runtime operations unmapped

Document runtime, deployment, migration execution, rollback, restore verification and approvers.

## Verification gaps

Branch protections, required checks, environment reviewers, crate publication, secret ownership, production database configuration and Thoth hosting/rollback remain unverified. Missing evidence is missing work.
