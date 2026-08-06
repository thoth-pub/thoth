# Publisher Services Task Status

Status: ACTIVE TRACKER
Programme owner: CTO
Master issue: [#765](https://github.com/thoth-pub/thoth/issues/765)
Approved design: [private Google Doc](https://docs.google.com/document/d/1kr2Ft0Y4pxgcXGyFAKs_wfFx4I0jlxEvaceswE5Dus8/edit), Drive revision `3`
Last updated: 2026-08-06

## 1. Control rule

Publisher Services follows the approved design's one-task/one-branch/one-PR workflow. It does not use a long-lived programme integration branch.

No task moves to `READY` without an approved specification, architecture dependencies, verified repository/base/target, branch-readiness completion or CTO exception, named implementation/review models, tests, migration, rollout and rollback.

`TBD` is missing work, not implicit approval.

## 2. Task tracker

| Task | Repository | Risk | Status | Verified base / PR target | Blocking dependencies | Issue | PR | Acceptance |
|---|---|---:|---|---|---|---|---|---|
| P0-01 Control documents and tracker | `thoth` | LOW | CLOSED | `develop` at `5b406e4ef9b5c192cc38eb8a97a41bbd0fc3bc06` / `develop` | None; issue #765 synchronization is a separately authorized external mirror of the completed repository closeout | [#765](https://github.com/thoth-pub/thoth/issues/765) | Foundation [#764](https://github.com/thoth-pub/thoth/pull/764); closeout [#767](https://github.com/thoth-pub/thoth/pull/767) merged as `bac598e3`; finalization [#768](https://github.com/thoth-pub/thoth/pull/768) | CLOSED - PR #767 independently `APPROVED` and merged as `bac598e32abbd0d7e69ff467c82945ee00df02ba` on 2026-07-27; reviewed content head `d72137893ddea512c0d05c81d310eb59d045cd2b`; repository-finalized |
| [ADR-01-SPEC-AMEND-01 Specification amendment](../engineering/ai-delivery/tasks/ADR-01-SPEC-AMEND-01.md) | `thoth` | MEDIUM | IN PROGRESS / CORRECTED CONTENT PROPOSED | `develop` at `590ff437bbd25b8aa5fde800dd8a38772b7e453e` / `develop` | CTO drafting authorization of 2026-08-06 and the CTO-approved [evidence ledger](adr-01-evidence-ledger.md). Corrected ADR-01 content pending fresh independent exact-head review and explicit CTO approval; historical ADR-01 specification approval preserved and applying only to the superseded pre-amendment content. | [#765](https://github.com/thoth-pub/thoth/issues/765) | Draft amendment PR (recorded on the PR itself) | CORRECTED CONTENT PROPOSED - NOT APPROVED - PR REMAINS DRAFT |
| [ADR-01 Platform inventory/final architecture](../engineering/ai-delivery/tasks/ADR-01.md) | `thoth` | MEDIUM | BLOCKED | exact base recorded when the implementation branch is created from a new then-current verified `develop` after the amendment merges; then-current `develop` / `develop` | approved and merged `ADR-01-SPEC-AMEND-01` corrected specification content plus fresh implementation authorization and a new exact `develop` base. The historical specification approval (Javi, CTO, 2026-08-05, content head `820f9cfa`, PR #780) is preserved and applies only to the superseded pre-amendment content. The existing local pre-amendment `feature/publisher-services/adr-01` branch is clean, unpushed, commit-free, not authoritative and not used; it must be deleted or archived before fresh authorization. ADR-01 is not blocked by BE-01. Final platform inventory remains provisional until the ADR-01 implementation is independently approved and merged. ADR-0004 (the ADR-01 decision record) is not started. | [#765](https://github.com/thoth-pub/thoth/issues/765) | Specification [#780](https://github.com/thoth-pub/thoth/pull/780); amendment PR draft; TBD for implementation | BLOCKED PENDING APPROVED AND MERGED AMENDMENT - IMPLEMENTATION NOT AUTHORIZED - FINAL PLATFORM INVENTORY PROVISIONAL |
| LIC-01 Expand `cc-license` | `cc-license` | MEDIUM | BLOCKED | `develop` / `develop` | P0-01; BR-LIC-01 or CTO exception; approved spec | #765 | TBD | NOT STARTED |
| LIC-02 Enforce supported licences | `thoth` | HIGH | BLOCKED | `develop` / `develop` | LIC-01 release; production licence audit plan | #765 | TBD | NOT STARTED |
| [BE-01 Publisher package model](../engineering/ai-delivery/tasks/BE-01.md) | `thoth` | HIGH | CLOSED | `develop` at `37b802776ae6853affe19d90156f3c1e0654ebe3` (PR #778 merge commit, verified before any edit) / `develop` | None remaining for BE-01 itself: the separately authorized bounded implementation was delivered on `feature/publisher-services/be-01` under ADR-0003 Architecture A and merged into `develop` through implementation PR [#779](https://github.com/thoth-pub/thoth/pull/779) after fresh independent exact-head review and explicit CTO merge authorization, as required for every HIGH-risk merge. Production migration/release execution remains separately gated by open CG-13, and the MIG-01 commercial backfill remains a separately approved CRITICAL task. | [#765](https://github.com/thoth-pub/thoth/issues/765) | Specification [#774](https://github.com/thoth-pub/thoth/pull/774); implementation [#779](https://github.com/thoth-pub/thoth/pull/779) | CLOSED - INACTIVE FOUNDATION - all publishers `OASIS`; no consumer, package API, mutation, UI, distribution, OAI or Metrics behaviour activated; retained-foundation operational rollback applies; evidence in the [BE-01 implementation report](../engineering/ai-delivery/implementation-reports/BE-01-implementation-report.md) and the immutable exact-head comments on PR #779 |
| BE-02 Distribution platform model | `thoth` | HIGH | BLOCKED | `develop` / `develop` | ADR-01 implementation independently approved and merged, not the ADR-01 specification alone; own approved bounded specification. BE-02 must not finalize `DistributionPlatform` from the provisional inventory. | #765 | TBD | NOT STARTED |
| BE-03 Protected service configuration | `thoth` | HIGH | BLOCKED | `develop` / `develop` | BE-01; BE-02; own approved bounded specification | #765 | TBD | NOT STARTED |
| BE-04 Durable distribution jobs | `thoth` | HIGH | BLOCKED | `develop` / `develop` | BE-02; BE-03 | #765 | TBD | NOT STARTED |
| MIG-01 Audit/production backfill | `thoth` + operations | CRITICAL | BLOCKED | dedicated task branch -> `develop`; separately approved production run | BE-01/02/03; licence audit; dry run | #765 | TBD | NOT STARTED |
| APP-01 Service configuration UI | `thoth-app` | MEDIUM | BLOCKED | current `dev` / `dev` pending BR-APP-01 or exception | BE-03 exposing the approved protected API; app readiness controls (BR-APP-01 or explicit CTO exception; the separately specified CG-11 CI closure task); generated API contract pinned to the exact BE-03 commit SHA per the reserved contract control; own approved bounded specification | #765 | TBD | NOT STARTED |
| APP-02 Staff subscription report | `thoth-app` | MEDIUM | BLOCKED | current `dev` / `dev` pending BR-APP-01 or exception | BE-03; BE-04; APP-01 | #765 | TBD | NOT STARTED |
| APP-03 API-backed licence options | `thoth-app` | MEDIUM | BLOCKED | current `dev` / `dev` pending BR-APP-01 or exception | LIC-02 | #765 | TBD | NOT STARTED |
| DIS-01 API discovery/comparison | `thoth-dissemination` | HIGH | BLOCKED | `develop` / `develop` | BE-02; MIG-01; BR-DIS-01 or exception | #765 | TBD | NOT STARTED |
| DIS-02 Back-catalogue worker | `thoth-dissemination` | CRITICAL | BLOCKED | `develop` / `develop` | BE-04; DIS-01 clean comparison; production controls | #765 | TBD | NOT STARTED |
| EXP-01 OCLC KBART index | `thoth` | MEDIUM | BLOCKED | `develop` / `develop` | BE-02; ADR-01 OCLC decision | #765 | TBD | NOT STARTED |
| OAI-01 Package/licence gating | `thoth` | HIGH | BLOCKED | fresh task branch; target decided after divergence review | BE-01; LIC-02; resolve design/target-policy branch conflict; its own approved bounded specification | #765 | TBD | NOT STARTED |
| OPS-01 Monitoring/runbooks/cleanup | multiple | HIGH | BLOCKED | repository-local task branches | implemented components; runtime ownership | #765 | TBD | NOT STARTED |
| E2E-01 Full workflow verification | multiple | HIGH | BLOCKED | controlled acceptance run | all activated components; pilot; rollback | #765 | TBD | NOT STARTED |

## 3. Branch rule

Recommended task names follow the design, for example:

```text
feature/publisher-services/adr-01
feature/publisher-services/be-01
feature/publisher-services/dis-01
```

Each branch starts from the repository's verified development branch and targets that branch directly. Merge and delete it after independent approval. There is no parent `feature/publisher-services` branch.

## 4. Next actions

1. The historical ADR-01 specification approval is preserved: specification
   [PR #780](https://github.com/thoth-pub/thoth/pull/780) was independently
   reviewed, explicitly CTO-approved and merged into `develop`. That approval
   applies only to the superseded pre-amendment content.
2. `ADR-01-SPEC-AMEND-01` is `IN PROGRESS / CORRECTED CONTENT PROPOSED`: the
   corrected ADR-01 specification content, drafted from the CTO-approved
   [evidence ledger](adr-01-evidence-ledger.md) under the CTO drafting
   authorization of 2026-08-06, is pending fresh independent exact-head
   review, explicit CTO approval of the corrected content, and separate merge
   authorization. The amendment PR remains draft; drafting authorization is
   not content approval.
3. ADR-01 implementation is `BLOCKED` pending the approved and merged
   amendment plus fresh implementation authorization and a new exact
   `develop` base. The existing local pre-amendment
   `feature/publisher-services/adr-01` branch is clean, unpushed, commit-free,
   not authoritative and not used. ADR-0004 is not started. ADR-01 and BE-01
   remain independent tasks; neither blocks the other.
4. The final distribution-platform inventory remains provisional until the
   ADR-01 implementation is independently approved and merged. CG-07 remains
   open; CG-11 and CG-13 are unchanged.
5. The bounded BE-01 implementation was separately authorized by the CTO on
   2026-08-05, delivered on `feature/publisher-services/be-01` from the
   verified base `37b802776ae6853affe19d90156f3c1e0654ebe3`, and merged into
   `develop` through implementation
   [PR #779](https://github.com/thoth-pub/thoth/pull/779), following ADR-0003
   Architecture A (direct `thoth-api/src/schema.rs` edit in the same bounded
   PR as the migration, models and tests). Evidence is recorded in the
   [BE-01 implementation report](../engineering/ai-delivery/implementation-reports/BE-01-implementation-report.md)
   and the immutable exact-head evidence comments on PR #779; transient
   delivery-workflow state lives only in that PR's metadata and comments.
6. The merged BE-01 foundation is inactive and closed: every publisher stores
   `OASIS`, and no consumer, package API, mutation, UI, distribution, OAI or
   Metrics behaviour is activated. BE-03 later exposes the protected
   package/capability contract under its own approval gates, and the MIG-01
   commercial backfill remains a separately approved CRITICAL task.
7. Production migration execution and release for BE-01 remain separately
   gated by open CG-13; nothing in BE-01 authorizes production action.
8. BE-02 remains blocked pending the independently approved and merged
   ADR-01 implementation - the specification alone is insufficient - plus its
   own approved bounded specification; it must not finalize
   `DistributionPlatform` from the provisional inventory.
9. BE-03 remains blocked pending BE-01 and BE-02 and its own approved bounded
   specification.
10. APP-01 remains blocked pending BE-03, app readiness controls (BR-APP-01 or
   an explicit CTO exception, and the separately specified CG-11 CI closure),
   the exact BE-03 SHA/schema-pinning contract control, and its own approved
   bounded specification.
11. No BE-02, BE-03, BE-04, APP-01, ADR-01 implementation, OAI-PMH,
    deployment, release or production work is authorized; all licence,
    migration, app, dissemination and operational tasks remain blocked under
    their recorded dependencies.
