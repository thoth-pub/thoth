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

### CG-07 - Publisher Services platform ADR (RESOLVED 2026-08-07)

Issue #765 exists. ADR-01 had to finalize enum values, mechanisms and
ambiguous destinations. CG-07 was **open** for as long as that decision was
outstanding or not yet repository-authoritative. The historical narrative
below is retained as evidence of how the gap was closed.

State as of 2026-08-06: `ADR-01-SPEC-AMEND-01` corrected the ADR-01
specification content from the CTO-approved evidence ledger
(`docs/publisher-services/adr-01-evidence-ledger.md`). The corrected content
was independently reviewed (review `4873802457`, `APPROVED`) and explicitly
CTO-approved (PR #781 comment `5203642323`, 2026-08-06) at exact content head
`1276c70a81e73f57d833eecb0e6886bd0cabf69e`; the approval-state head
`bdfded20` received final independent review `4874093991` (`APPROVED`) and
CTO merge authorization (review `4874128610`); and PR #781 merged into
`develop` as `a511e01c83c5e805a75e0fdaeb3b5297c39ef291` on
2026-08-06T11:29:53Z, making the corrected ADR-01 specification
repository-authoritative. The specification amendment is complete. The
historical ADR-01 specification approval (content head `820f9cfa`, PR #780)
applies only to the superseded pre-amendment content.

Later on 2026-08-06 the ADR-01 implementation was separately and explicitly
CTO-authorized from exact base
`32123d363a6806d377ac322e3814fb432a803453` and delivered as a
documentation-only draft PR on `feature/publisher-services/adr-01`,
producing
[ADR-0004](../decisions/ADR-0004-distribution-platform-inventory.md), the
complete
[evidence matrix](../../publisher-services/adr-01-evidence-matrix.md) and
the
[final inventory](../../publisher-services/platform-inventory.md).

State as of 2026-08-07: the ADR-0004 and final-inventory content was
independently reviewed at exact head
`44e6f821535fbee56c830dd6eda237fc6d06fbfd` (review `4881233664`,
`APPROVED`) and explicitly CTO-approved (review `4881279067`). Content
approval was recorded on PR #783, which then carried approval-state head
`82874c2bfb0c211198252e4f4a0b669d31e14836`.

CG-07 is **RESOLVED** as of 2026-08-07. All of its closure criteria are met
and recorded:

- ADR-01 finalized the exhaustive distribution-platform inventory (17
  included destinations, 10 recorded exclusions, no `OTHER`, no fallback, no
  unknown or provisional included value);
- [ADR-0004](../decisions/ADR-0004-distribution-platform-inventory.md) is
  approved at content head `44e6f821535fbee56c830dd6eda237fc6d06fbfd`
  (independent review `4881233664` - `APPROVED`; CTO content approval
  `4881279067`);
- the [final inventory](../../publisher-services/platform-inventory.md) is
  approved as exactly that reviewed content;
- approval state was recorded on PR #783;
- the approval-state head `82874c2bfb0c211198252e4f4a0b669d31e14836` received
  fresh independent exact-head review `4881832108` (`APPROVED`);
- CTO merge authorization `4881847699` was granted for that exact head;
- PR [#783](https://github.com/thoth-pub/thoth/pull/783) merged into
  `develop` as `299b0eff3b9ac10cc0a3a7024ab311ddb135b7eb` on
  2026-08-07T10:02:34Z, making ADR-0004 and the final inventory
  repository-authoritative;
- the merged control state was reconciled by
  [`ADR-01-CLOSEOUT-01`](../ai-delivery/tasks/ADR-01-CLOSEOUT-01.md).

ADR-01 is `MERGED - COMPLETE`. It was an evidence and architecture-decision
task: it is itself not runtime `IMPLEMENTED` and not `PRODUCTION READY`, and
resolving CG-07 made no implementation task ready. The runtime
`DistributionPlatform` implementation was delivered separately by `BE-02`. `BE-02`'s ADR-01 dependency was satisfied by
that merge; `BE-02` was subsequently specified, authorized, implemented and
merged as an inactive additive foundation through PR
[#805](https://github.com/thoth-pub/thoth/pull/805), which authorized
repository integration only and no deployment, migration execution, backfill
or distribution activation. CG-11 and CG-13 are unchanged by the ADR-01
implementation, by this closeout and by the BE-02 merge.

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

CG-13 is **OPEN**.

A bounded feature-specific successor addresses one subset of it:
[`THOTH-GQL-OPS-01`](../ai-delivery/tasks/THOTH-GQL-OPS-01.md), covering runtime
mode control for the merged GraphQL mutation guard
(`THOTH_GRAPHQL_MUTATION_GUARD_MODE`) — configuration authority, restart/redeploy
semantics, fleet propagation and verification, partial-fleet handling, rollback,
change authority and evidence. Its delivered output is the
[mutation-guard runtime-operations control record](./graphql-mutation-guard-runtime-operations.md)
and the **provisional**
[mode-transition runbook](./graphql-mutation-guard-mode-transition-runbook.md).

**Feature-subset disposition: `C` — insufficient operational capability/evidence;
BLOCKED.**

**`THOTH-GQL-OPS-01` cannot, by itself, satisfy even that subset.** Its discovery,
independently re-derived at implementation time against the exact base, confirms
two capability gaps that documentation cannot close:

1. the current production deployment path cannot consume
   `THOTH_GRAPHQL_MUTATION_GUARD_MODE`. Production configuration inherits the
   container image's default `init` command, and `init` does not register the
   guard argument, so a **guard-enabled** release deployed through that path
   would silently ignore the configured value and remain effectively `OFF`. An
   `OFF -> OBSERVE` transition of a guard-enabled candidate is therefore not
   operationally performable through the current deployment path. This is the
   `THOTH-GQL-OPS-02` capability gap;
2. no implemented mechanism can prove the effective mode of every serving
   instance, so a mode change could not be verified even if one could be made.

The deployed production release is a separate matter from either gap, and the
three states must not be conflated:

```text
merged develop state
    != deployed production release
    != production activation state
```

The releases currently deployed **predate** `THOTH-GQL-BATCH-01`: their binaries
contain no mutation guard at all, so they are recorded as **pre-guard** and are
not described as running `MutationGuardMode::OFF`. That conclusion rests on two
evidence classes and on neither alone — repository evidence establishes that the
relevant release code contains no mutation guard, and scoped authoritative
deployment metadata establishes that it is the release those environments run.
Implementation re-established the same conclusion for the **test** environment,
which is also pre-guard; there is consequently no environment in which a mode
could currently be changed. Merging `THOTH-GQL-BATCH-01` deployed nothing and
activated nothing. No environment has been transitioned to `OBSERVE` or
`ENFORCE`, and any guard-enabled candidate remains effectively `OFF` unless
separately authorized.

The terminal disposition is therefore **C — insufficient operational
capability/evidence; BLOCKED**, and the `ADR-0006` runtime-operations gate
remains **NOT SATISFIED**. Closing the two gaps requires separate bounded tasks —
[`THOTH-GQL-OPS-02`](../ai-delivery/tasks/THOTH-GQL-OPS-02.md) (mode-control
path) and [`THOTH-GQL-OPS-03`](../ai-delivery/tasks/THOTH-GQL-OPS-03.md)
(fleet-verification mechanism) — each implemented, independently reviewed and
merged on its own authority. Only then may
[`THOTH-GQL-OPS-04`](../ai-delivery/tasks/THOTH-GQL-OPS-04.md) re-verify against
the real runtime and decide, on evidence, whether the feature-specific subset is
satisfied. All three specifications are `DRAFT` and their implementation is
`NOT AUTHORIZED`; none of their branches exists.

```text
Runtime-operations gate: NOT SATISFIED
Blocking prerequisites:  THOTH-GQL-OPS-02, THOTH-GQL-OPS-03
Earliest satisfaction:   THOTH-GQL-OPS-04, on evidence
OFF -> OBSERVE:          NOT AUTHORIZED
OBSERVE -> ENFORCE:      NOT AUTHORIZED
BE-02 runtime:           NOT AUTHORIZED
```

None of these tasks closes CG-13. Migration execution, backup and restore
verification, and approver mapping for concerns other than this feature remain
open here regardless of their outcome. Any broader closure would need its own
evidence, independent review and CTO decision recorded in this register.

Beyond the two capability gaps, the control record leaves further evidence
explicitly unresolved and recorded as missing work. Four of these block an
acceptance criterion outright, and `THOTH-GQL-OPS-04` must obtain each:

- the **accountable production runtime owner**. Execution *capability* is
  established from an access record; accountable *ownership* is not, and is not
  derivable from one — it requires an explicit CTO designation. This repository
  still lists production deployment owners among the controls it lacks;
- the **post-activation observation sign-off owner**, which the control record
  proposes but does not establish, pending explicit CTO confirmation;
- whether **operational rollback additionally requires CTO approval**, or may be
  executed on the technical team's own authority;
- the **live expected replica population**, readable only from live orchestrator
  state.

Also unresolved, and not blocking a criterion: configuration drift between the
authoritative deployment source and the live orchestrator; the approved `OBSERVE`
observation-window duration and therefore whether the finite configured runtime
log retention covers it, with no remedy pre-selected; and the measured
propagation, mixed-window and rollback durations, which are owned by the
**downstream** preview/staging rehearsal rather than by `THOTH-GQL-OPS-04`.

## Verification gaps

Branch protections, required checks, environment reviewers, crate publication, secret ownership, production database configuration and Thoth hosting/rollback remain unverified. Missing evidence is missing work.
