# Publisher Services Task Status

Status: ACTIVE TRACKER
Programme owner: CTO
Master issue: [#765](https://github.com/thoth-pub/thoth/issues/765)
Approved design: [private Google Doc](https://docs.google.com/document/d/1kr2Ft0Y4pxgcXGyFAKs_wfFx4I0jlxEvaceswE5Dus8/edit), Drive revision `3`
Last updated: 2026-08-12

## 1. Control rule

Publisher Services follows the approved design's one-task/one-branch/one-PR workflow. It does not use a long-lived programme integration branch.

No task moves to `READY` without an approved specification, architecture dependencies, verified repository/base/target, branch-readiness completion or CTO exception, named implementation/review models, tests, migration, rollout and rollback.

`TBD` is missing work, not implicit approval.

## 2. Task tracker

| Task | Repository | Risk | Status | Verified base / PR target | Blocking dependencies | Issue | PR | Acceptance |
|---|---|---:|---|---|---|---|---|---|
| P0-01 Control documents and tracker | `thoth` | LOW | CLOSED | `develop` at `5b406e4ef9b5c192cc38eb8a97a41bbd0fc3bc06` / `develop` | None; issue #765 synchronization is a separately authorized external mirror of the completed repository closeout | [#765](https://github.com/thoth-pub/thoth/issues/765) | Foundation [#764](https://github.com/thoth-pub/thoth/pull/764); closeout [#767](https://github.com/thoth-pub/thoth/pull/767) merged as `bac598e3`; finalization [#768](https://github.com/thoth-pub/thoth/pull/768) | CLOSED - PR #767 independently `APPROVED` and merged as `bac598e32abbd0d7e69ff467c82945ee00df02ba` on 2026-07-27; reviewed content head `d72137893ddea512c0d05c81d310eb59d045cd2b`; repository-finalized |
| [ADR-01-SPEC-AMEND-01 Specification amendment](../engineering/ai-delivery/tasks/ADR-01-SPEC-AMEND-01.md) | `thoth` | MEDIUM | MERGED - COMPLETE | `develop` at `590ff437bbd25b8aa5fde800dd8a38772b7e453e` / `develop` | None remaining: approved content head `1276c70a81e73f57d833eecb0e6886bd0cabf69e` (independent review `4873802457` - APPROVED; CTO approval comment `5203642323` - 2026-08-06); approval-state head `bdfded20e8cac65fcd7713b07d189052e0eba745` (final independent review `4874093991` - APPROVED; CTO merge authorization review `4874128610`); merged as `a511e01c83c5e805a75e0fdaeb3b5297c39ef291` on 2026-08-06T11:29:53Z. Historical ADR-01 specification approval preserved and applying only to the superseded pre-amendment content. Post-merge reconciliation is delivered by [ADR-01-SPEC-AMEND-01-CLOSEOUT-01](../engineering/ai-delivery/tasks/ADR-01-SPEC-AMEND-01-CLOSEOUT-01.md). | [#765](https://github.com/thoth-pub/thoth/issues/765) | Amendment PR [#781](https://github.com/thoth-pub/thoth/pull/781) (merged) | MERGED - COMPLETE - corrected ADR-01 specification repository-authoritative through merge commit `a511e01c` |
| [ADR-01 Platform inventory/final architecture](../engineering/ai-delivery/tasks/ADR-01.md) | `thoth` | MEDIUM | MERGED - COMPLETE | `develop` at `32123d363a6806d377ac322e3814fb432a803453` (verified before any edit; fresh explicit CTO implementation authorization 2026-08-06) / `develop` | None remaining: content gates satisfied at approved content head `44e6f821535fbee56c830dd6eda237fc6d06fbfd` (independent exact-head review `4881233664` - `APPROVED`; explicit CTO approval `4881279067` of ADR-0004 and the final inventory, 2026-08-07); approval-state head `82874c2bfb0c211198252e4f4a0b669d31e14836` (final independent review `4881832108` - `APPROVED`; CTO merge authorization `4881847699`); merged as `299b0eff3b9ac10cc0a3a7024ab311ddb135b7eb` on 2026-08-07T10:02:34Z. Delivered on `feature/publisher-services/adr-01`: [ADR-0004](../engineering/decisions/ADR-0004-distribution-platform-inventory.md) (`APPROVED AND REPOSITORY-AUTHORITATIVE`), the complete [evidence matrix](adr-01-evidence-matrix.md), the approved [final inventory](platform-inventory.md) and the [implementation report](../engineering/ai-delivery/implementation-reports/ADR-01-implementation-report.md). Post-merge control reconciliation is delivered by [ADR-01-CLOSEOUT-01](../engineering/ai-delivery/tasks/ADR-01-CLOSEOUT-01.md). ADR-01 was an evidence/architecture task: it is not runtime implemented and not production ready, and no runtime `DistributionPlatform` implementation exists. BE-02's ADR-01 dependency is satisfied; BE-02 remains blocked and unauthorized. The historical specification approval (content head `820f9cfa`, PR #780) is preserved and applies only to the superseded pre-amendment content. | [#765](https://github.com/thoth-pub/thoth/issues/765) | Specification [#780](https://github.com/thoth-pub/thoth/pull/780); amendment [#781](https://github.com/thoth-pub/thoth/pull/781) (merged); implementation [#783](https://github.com/thoth-pub/thoth/pull/783) (merged) | MERGED - COMPLETE - ADR-0004 and the final inventory approved and repository-authoritative through merge commit `299b0eff`; CG-07 `RESOLVED`; not runtime implemented and not production ready |
| LIC-01 Expand `cc-license` | `cc-license` | MEDIUM | BLOCKED | `develop` / `develop` | P0-01; BR-LIC-01 or CTO exception; approved spec | #765 | TBD | NOT STARTED |
| LIC-02 Enforce supported licences | `thoth` | HIGH | BLOCKED | `develop` / `develop` | LIC-01 release; production licence audit plan | #765 | TBD | NOT STARTED |
| [BE-01 Publisher package model](../engineering/ai-delivery/tasks/BE-01.md) | `thoth` | HIGH | CLOSED | `develop` at `37b802776ae6853affe19d90156f3c1e0654ebe3` (PR #778 merge commit, verified before any edit) / `develop` | None remaining for BE-01 itself: the separately authorized bounded implementation was delivered on `feature/publisher-services/be-01` under ADR-0003 Architecture A and merged into `develop` through implementation PR [#779](https://github.com/thoth-pub/thoth/pull/779) after fresh independent exact-head review and explicit CTO merge authorization, as required for every HIGH-risk merge. Production migration/release execution remains separately gated by open CG-13, and the MIG-01 commercial backfill remains a separately approved CRITICAL task. | [#765](https://github.com/thoth-pub/thoth/issues/765) | Specification [#774](https://github.com/thoth-pub/thoth/pull/774); implementation [#779](https://github.com/thoth-pub/thoth/pull/779) | CLOSED - INACTIVE FOUNDATION - all publishers `OASIS`; no consumer, package API, mutation, UI, distribution, OAI or Metrics behaviour activated; retained-foundation operational rollback applies; evidence in the [BE-01 implementation report](../engineering/ai-delivery/implementation-reports/BE-01-implementation-report.md) and the immutable exact-head comments on PR #779 |
| [BE-02 Distribution platform model](../engineering/ai-delivery/tasks/BE-02.md) | `thoth` | HIGH | BLOCKED - IMPLEMENTATION NOT AUTHORIZED | `develop` / `develop` | ADR-01/final inventory is satisfied through PR #783. ADR-0007 shared GraphQL architecture is satisfied through PR #800. The request-local non-cached DataLoader foundation required for `Publisher.distributionPlatforms` is merged through PR #802 (`8dcf031d`). Remaining gates: independent exact-head review and CTO approval of the reconciled BE-02 specification on PR #788; merge of the approved specification so it is repository-authoritative; then fresh exact-`develop` preflight and separate explicit CTO implementation authorization before `feature/publisher-services/be-02` may be created. | #765 | Specification [#788](https://github.com/thoth-pub/thoth/pull/788) | SPECIFICATION CANDIDATE - platform/migration/lifecycle/API design reconciled to ADR-0007 first-production-DataLoader adoption; runtime implementation not started or authorized |
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
2. `ADR-01-SPEC-AMEND-01` is `MERGED - COMPLETE`: the corrected ADR-01
   specification content, drafted from the CTO-approved
   [evidence ledger](adr-01-evidence-ledger.md) under the CTO drafting
   authorization of 2026-08-06, was independently reviewed (review
   `4873802457`, `APPROVED`) and explicitly CTO-approved (comment
   `5203642323`, 2026-08-06) at exact content head
   `1276c70a81e73f57d833eecb0e6886bd0cabf69e`; the approval-state head
   `bdfded20` received final independent review `4874093991` (`APPROVED`)
   and CTO merge authorization (review `4874128610`); and amendment PR
   [#781](https://github.com/thoth-pub/thoth/pull/781) merged into
   `develop` as `a511e01c83c5e805a75e0fdaeb3b5297c39ef291` on
   2026-08-06T11:29:53Z, making the corrected specification
   repository-authoritative.
3. The ADR-01 implementation was separately and explicitly authorized by the
   CTO on 2026-08-06 from exact base
   `32123d363a6806d377ac322e3814fb432a803453` and delivered as a draft PR on
   `feature/publisher-services/adr-01`:
   [ADR-0004](../engineering/decisions/ADR-0004-distribution-platform-inventory.md)
   (`APPROVED`), the complete [evidence matrix](adr-01-evidence-matrix.md),
   the approved [final inventory](platform-inventory.md), programme-control
   reconciliation and the
   [implementation report](../engineering/ai-delivery/implementation-reports/ADR-01-implementation-report.md).
   The content gates were satisfied at approved content head
   `44e6f821535fbee56c830dd6eda237fc6d06fbfd`: independent exact-head review
   `4881233664` (`APPROVED`) and explicit CTO approval `4881279067`
   (2026-08-07). The approval-state head
   `82874c2bfb0c211198252e4f4a0b669d31e14836` received final independent
   review `4881832108` (`APPROVED`) and CTO merge authorization
   `4881847699`, and PR #783 merged into `develop` as
   `299b0eff3b9ac10cc0a3a7024ab311ddb135b7eb` on 2026-08-07T10:02:34Z.
   ADR-01 is `MERGED - COMPLETE`; post-merge control reconciliation is
   delivered by
   [ADR-01-CLOSEOUT-01](../engineering/ai-delivery/tasks/ADR-01-CLOSEOUT-01.md).
   ADR-01 and BE-01 remain independent tasks; neither blocks the other.
4. The final distribution-platform inventory is `FINAL INVENTORY APPROVED AND
   REPOSITORY-AUTHORITATIVE` at content head `44e6f821...` as the exactly
   reviewed content (17 included destinations, 10 recorded exclusions, no
   `OTHER`, no fallback, no unknown or provisional included value). It became
   repository-authoritative on merge of
   [PR #783](https://github.com/thoth-pub/thoth/pull/783) as `299b0eff`, and
   the inventory is no longer provisional. No enum is implemented from it,
   and none may be implemented outside a separately approved and authorized
   task. ADR-01 is `MERGED - COMPLETE`: an evidence and architecture-decision
   task, not runtime implemented and not production ready. CG-07 is
   `RESOLVED`; CG-11 and CG-13 are unchanged.
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
8. BE-02's ADR-01 dependency is satisfied: the ADR-01 implementation was
   independently approved and merged (PR #783, merge commit `299b0eff`), and
   the final inventory is approved and repository-authoritative rather than
   provisional. ADR-0007 is repository-authoritative and the request-local
   non-cached DataLoader foundation required for the first production child
   adoption is merged through PR #802 (`8dcf031d`). Existing specification PR
   [#788](https://github.com/thoth-pub/thoth/pull/788) is the single bounded
   BE-02 specification candidate and is reconciled to that architecture. The
   next gate is fresh independent exact-head specification review, followed by
   explicit CTO specification approval and merge. Only after that exact
   approved spec is repository-authoritative may the CTO separately authorize
   BE-02 implementation against a freshly verified exact `develop` head.
9. BE-03 remains blocked pending BE-01 and BE-02 and its own approved bounded
   specification.
10. APP-01 remains blocked pending BE-03, app readiness controls (BR-APP-01 or
   an explicit CTO exception, and the separately specified CG-11 CI closure),
   the exact BE-03 SHA/schema-pinning contract control, and its own approved
   bounded specification.
11. Beyond the delivered and merged documentation-only ADR-01 implementation,
    ADR-01-CLOSEOUT-01 control reconciliation and shared DataLoader foundation,
    no BE-02 runtime, BE-03, BE-04, APP-01, OAI-PMH, deployment, release,
    production migration, assignment backfill, distribution activation,
    `OBSERVE`/`ENFORCE` transition or PR #799 action is authorized; all licence,
    migration, app, dissemination and operational tasks remain blocked under
    their recorded dependencies. Repository authority and specification work
    do not authorize runtime change, credential use, workflow dispatch or
    production access.
