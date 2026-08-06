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

Issue #765 exists. ADR-01 must finalize enum values, mechanisms and ambiguous
destinations. CG-07 remains **open**.

State as of 2026-08-06: `ADR-01-SPEC-AMEND-01` proposes corrected ADR-01
specification content from the CTO-approved evidence ledger
(`docs/publisher-services/adr-01-evidence-ledger.md`). The corrected content
is pending fresh independent review and explicit CTO approval; the historical
ADR-01 specification approval (content head `820f9cfa`, PR #780) applies only
to the superseded pre-amendment content. ADR-01 implementation remains blocked
pending the approved and merged amendment plus fresh implementation
authorization from a new exact `develop` base. The final platform inventory
remains provisional. CG-11 and CG-13 are unchanged by the amendment.

### CG-08 - Metrics readiness open

Issue #766 exists. Sphinx normalization/bootstrap, Diesel control, branch readiness and service-role decisions remain prerequisites.

## Production-slice controls

### CG-09 - Source fixtures/mappings incomplete

Metrics fixtures, COUNTER mappings, finalization settings and OPERAS projections are missing.

### CG-10 - OPERAS inbound completeness unavailable

No verified complete cursor, replication or snapshot route exists.

### CG-11 - CI gaps

App lacks explicit lint/build/codegen; dashboard lacks detected CI/tests; widget lacks unit tests; cc-license uses old Actions.

### CG-12 - Thoth schema generation (RESOLVED via Architecture A)

Disposable PostgreSQL 17 testing established that root `diesel.toml` was
syntactically and semantically invalid, its output path did not identify
canonical `thoth-api/src/schema.rs`, and raw Diesel output does not preserve the
compiled contract's aliases, supplemental type, timestamp semantics, column
order, or formatting.

The originally specified answer, `THOTH-DB-CTRL-01`, was an exact-version,
fail-closed structural synchronizer built on root `diesel.toml`, raw
`diesel print-schema`, a convention file, and a custom reconciliation subsystem.
That approach was rejected. Its implementation PR
[#777](https://github.com/thoth-pub/thoth/pull/777) was closed unmerged, and no
code from it became repository-authoritative.

On 2026-08-05 the CTO selected Architecture A, recorded in
[ADR-0003](../decisions/ADR-0003-repository-authoritative-schema-contract.md):
`thoth-api/src/schema.rs` is the repository-authoritative, manually maintained
Diesel schema contract; migrations, `schema.rs`, models, and database-backed
tests change atomically in one bounded task; and the Diesel CLI and root
`diesel.toml` are retired from the supported workflow.
[`THOTH-DB-CTRL-01`](../ai-delivery/tasks/THOTH-DB-CTRL-01.md) is marked
`SUPERSEDED`, and [`THOTH-DB-CTRL-02`](../ai-delivery/tasks/THOTH-DB-CTRL-02.md)
delivers ADR-0003 and its directly related cleanup through PR
[#778](https://github.com/thoth-pub/thoth/pull/778).

CG-12 is **RESOLVED** by Architecture A (ADR-0003), delivered by
[`THOTH-DB-CTRL-02`](../ai-delivery/tasks/THOTH-DB-CTRL-02.md) through PR
[#778](https://github.com/thoth-pub/thoth/pull/778). The merged Architecture A
control is the repository answer to how `schema.rs` tracks migrations, and BE-01
is `READY` for separately authorized implementation. `READY` does not authorize
implementation by itself: creating the BE-01 branch and making any
implementation edit require separate explicit authorization, and the branch
remains absent until then. This record becomes authoritative when PR #778 merges
into `develop`; the merge itself remains subject to independent exact-head review
and explicit CTO merge authorization. CG-13 remains open.

### CG-13 - Thoth runtime operations unmapped

Document runtime, deployment, migration execution, rollback, restore verification and approvers.

## Verification gaps

Branch protections, required checks, environment reviewers, crate publication, secret ownership, production database configuration and Thoth hosting/rollback remain unverified. Missing evidence is missing work.
