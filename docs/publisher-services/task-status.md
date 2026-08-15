# Publisher Services Task Status

Status: ACTIVE TRACKER
Programme owner: CTO
Master issue: [#765](https://github.com/thoth-pub/thoth/issues/765)
Approved design: [private Google Doc](https://docs.google.com/document/d/1kr2Ft0Y4pxgcXGyFAKs_wfFx4I0jlxEvaceswE5Dus8/edit), Drive revision `3`
Last updated: 2026-08-15 (BE-04's corrected specification is repository-authoritative, and its implementation candidate has been reconciled against it. Three authority facts hold together and none cancels another. The **approved baseline** — the pre-addendum BE-04 specification merged into `develop` through PR [#814](https://github.com/thoth-pub/thoth/pull/814) — is CTO-approved and is preserved as valid history, as is the CTO's explicit implementation authorization on that pull request against `develop` at `ed32712766c8f5a1951bb53ec3192e18f067c7d2`, under which the original implementation candidate was properly authorized work. **`BE-04-SPEC-ADDENDUM-01`**, merged through PR [#817](https://github.com/thoth-pub/thoth/pull/817), carries the corrected specification content and is repository-authoritative; it reconciled three defects a fresh independent review found by measuring the candidate — a non-NULL-safe attempt-error `CHECK`, an unsatisfiable exact report statement-count contract, and a mandatory `cargo test -p thoth-client` gate that does not build at the authorized base. The baseline authorization was insufficient for the corrected contract, so the CTO issued a **fresh implementation reconciliation authorization**, GitHub-owned on PR [#816](https://github.com/thoth-pub/thoth/pull/816) and bound to `develop` at `8c0c54bd7b2e58a645ffe39abd8ceeee86e47686`; PR #816 carries that reconciliation. BE-04 now also has a repository-local owning issue, [#821](https://github.com/thoth-pub/thoth/issues/821) under parent programme issue #765, which is its durable live task-ledger entry; under a bounded review-remediation authorization recorded there, the implementation branch additionally incorporates `develop` at `ec7868a4a44b3d52da5638975995bb66a488b3b4` by ordinary merge, so the candidate is built under the current granular, deny-by-default, non-transitive repository-control doctrine merged through PR [#820](https://github.com/thoth-pub/thoth/pull/820), and its cross-repository impact is assessed against `docs/engineering/repository-map/contracts.md` with every currently verified public-GraphQL consumer recorded as remaining compatible under BE-04's additive schema change. Repository authority for the implementation itself depends on the exact approved implementation content becoming reachable from `develop`, and live review, authorization and merge state is the GitHub record (`ADR-0005`). Runtime remains inactive throughout: automatic job creation is `OFF` by default and unactivated, no `distribution_job`, `distribution_job_target` or `distribution_job_attempt` row exists anywhere, no worker and no credential exists, and no dissemination occurs. Earlier state unchanged: BE-03 closed as an inactive merged foundation, with its dependency satisfied for BE-04, MIG-01, APP-01 and APP-02 without any of them becoming ready, and `ADR-0008` repository-authoritative on `develop`, satisfying that BE-04 control prerequisite without approving the BE-04 specification or authorizing implementation)

## 1. Control rule

Publisher Services follows the approved design's one-task/one-branch/one-PR workflow. It does not use a long-lived programme integration branch.

No task moves to `READY` without an approved specification, architecture dependencies, verified repository/base/target, branch-readiness completion or CTO exception, named implementation/review models, tests, migration, rollout and rollback.

`TBD` is missing work, not implicit approval.

## 2. Task tracker

| Task | Repository | Risk | Status | Verified base / PR target | Blocking dependencies | Issue | PR | Acceptance |
|---|---|---:|---|---|---|---|---|---|
| P0-01 Control documents and tracker | `thoth` | LOW | CLOSED | `develop` at `5b406e4ef9b5c192cc38eb8a97a41bbd0fc3bc06` / `develop` | None; issue #765 synchronization is a separately authorized external mirror of the completed repository closeout | [#765](https://github.com/thoth-pub/thoth/issues/765) | Foundation [#764](https://github.com/thoth-pub/thoth/pull/764); closeout [#767](https://github.com/thoth-pub/thoth/pull/767) merged as `bac598e3`; finalization [#768](https://github.com/thoth-pub/thoth/pull/768) | CLOSED - PR #767 independently `APPROVED` and merged as `bac598e32abbd0d7e69ff467c82945ee00df02ba` on 2026-07-27; reviewed content head `d72137893ddea512c0d05c81d310eb59d045cd2b`; repository-finalized |
| [ADR-01-SPEC-AMEND-01 Specification amendment](../engineering/ai-delivery/tasks/ADR-01-SPEC-AMEND-01.md) | `thoth` | MEDIUM | MERGED - COMPLETE | `develop` at `590ff437bbd25b8aa5fde800dd8a38772b7e453e` / `develop` | None remaining: approved content head `1276c70a81e73f57d833eecb0e6886bd0cabf69e` (independent review `4873802457` - APPROVED; CTO approval comment `5203642323` - 2026-08-06); approval-state head `bdfded20e8cac65fcd7713b07d189052e0eba745` (final independent review `4874093991` - APPROVED; CTO merge authorization review `4874128610`); merged as `a511e01c83c5e805a75e0fdaeb3b5297c39ef291` on 2026-08-06T11:29:53Z. Historical ADR-01 specification approval preserved and applying only to the superseded pre-amendment content. Post-merge reconciliation is delivered by [ADR-01-SPEC-AMEND-01-CLOSEOUT-01](../engineering/ai-delivery/tasks/ADR-01-SPEC-AMEND-01-CLOSEOUT-01.md). | [#765](https://github.com/thoth-pub/thoth/issues/765) | Amendment PR [#781](https://github.com/thoth-pub/thoth/pull/781) (merged) | MERGED - COMPLETE - corrected ADR-01 specification repository-authoritative through merge commit `a511e01c` |
| [ADR-01 Platform inventory/final architecture](../engineering/ai-delivery/tasks/ADR-01.md) | `thoth` | MEDIUM | MERGED - COMPLETE | `develop` at `32123d363a6806d377ac322e3814fb432a803453` (verified before any edit; fresh explicit CTO implementation authorization 2026-08-06) / `develop` | None remaining: content gates satisfied at approved content head `44e6f821535fbee56c830dd6eda237fc6d06fbfd` (independent exact-head review `4881233664` - `APPROVED`; explicit CTO approval `4881279067` of ADR-0004 and the final inventory, 2026-08-07); approval-state head `82874c2bfb0c211198252e4f4a0b669d31e14836` (final independent review `4881832108` - `APPROVED`; CTO merge authorization `4881847699`); merged as `299b0eff3b9ac10cc0a3a7024ab311ddb135b7eb` on 2026-08-07T10:02:34Z. Delivered on `feature/publisher-services/adr-01`: [ADR-0004](../engineering/decisions/ADR-0004-distribution-platform-inventory.md) (`APPROVED AND REPOSITORY-AUTHORITATIVE`), the complete [evidence matrix](adr-01-evidence-matrix.md), the approved [final inventory](platform-inventory.md) and the [implementation report](../engineering/ai-delivery/implementation-reports/ADR-01-implementation-report.md). Post-merge control reconciliation is delivered by [ADR-01-CLOSEOUT-01](../engineering/ai-delivery/tasks/ADR-01-CLOSEOUT-01.md). ADR-01 was an evidence/architecture task: it is itself not runtime implemented and not production ready. BE-02's ADR-01 dependency was satisfied by this merge, and the runtime `DistributionPlatform` implementation was delivered separately by BE-02. The historical specification approval (content head `820f9cfa`, PR #780) is preserved and applies only to the superseded pre-amendment content. | [#765](https://github.com/thoth-pub/thoth/issues/765) | Specification [#780](https://github.com/thoth-pub/thoth/pull/780); amendment [#781](https://github.com/thoth-pub/thoth/pull/781) (merged); implementation [#783](https://github.com/thoth-pub/thoth/pull/783) (merged) | MERGED - COMPLETE - ADR-0004 and the final inventory approved and repository-authoritative through merge commit `299b0eff`; CG-07 `RESOLVED`; not runtime implemented and not production ready |
| LIC-01 Expand `cc-license` | `cc-license` | MEDIUM | BLOCKED | `develop` / `develop` | P0-01; BR-LIC-01 or CTO exception; approved spec | #765 | TBD | NOT STARTED |
| LIC-02 Enforce supported licences | `thoth` | HIGH | BLOCKED | `develop` / `develop` | LIC-01 release; production licence audit plan | #765 | TBD | NOT STARTED |
| [BE-01 Publisher package model](../engineering/ai-delivery/tasks/BE-01.md) | `thoth` | HIGH | CLOSED | `develop` at `37b802776ae6853affe19d90156f3c1e0654ebe3` (PR #778 merge commit, verified before any edit) / `develop` | None remaining for BE-01 itself: the separately authorized bounded implementation was delivered on `feature/publisher-services/be-01` under ADR-0003 Architecture A and merged into `develop` through implementation PR [#779](https://github.com/thoth-pub/thoth/pull/779) after fresh independent exact-head review and explicit CTO merge authorization, as required for every HIGH-risk merge. Production migration/release execution remains separately gated by open CG-13, and the MIG-01 commercial backfill remains a separately approved CRITICAL task. | [#765](https://github.com/thoth-pub/thoth/issues/765) | Specification [#774](https://github.com/thoth-pub/thoth/pull/774); implementation [#779](https://github.com/thoth-pub/thoth/pull/779) | CLOSED - INACTIVE FOUNDATION - all publishers `OASIS`; no consumer, package API, mutation, UI, distribution, OAI or Metrics behaviour activated; retained-foundation operational rollback applies; evidence in the [BE-01 implementation report](../engineering/ai-delivery/implementation-reports/BE-01-implementation-report.md) and the immutable exact-head comments on PR #779 |
| [BE-02 Distribution platform model](../engineering/ai-delivery/tasks/BE-02.md) | `thoth` | HIGH | CLOSED | `develop` at `1c752a522f7048963efde00b50565379d7c14b4d` (PR #788 merge commit, verified before any edit) / `develop` | None remaining for BE-02 itself: ADR-01/the final inventory is satisfied through PR #783, ADR-0007 through PR #800, and the request-local non-cached DataLoader foundation through PR #802 (`8dcf031d`). The reconciled BE-02 specification was independently reviewed, CTO-approved and merged through PR #788, making it repository-authoritative at `1c752a52`; the CTO then separately authorized implementation against that exact `develop` SHA, and the bounded implementation was delivered on `feature/publisher-services/be-02` and merged into `develop` through implementation PR [#805](https://github.com/thoth-pub/thoth/pull/805) after fresh independent exact-head review and explicit CTO merge authorization, as required for every HIGH-risk merge. Deployment, environment migration execution, production migration, assignment creation/backfill and distribution activation remain separately gated and unauthorized. | #765 | Specification [#788](https://github.com/thoth-pub/thoth/pull/788); implementation [#805](https://github.com/thoth-pub/thoth/pull/805) | CLOSED - INACTIVE FOUNDATION - the 17-value `DistributionPlatform` enum, the `publisher_distribution_platform` migration and repository-authoritative `schema.rs`, the assignment lifecycle, linked OAPEN/DOAB normalization, four additive public GraphQL read surfaces and the first production ADR-0007 DataLoader adoption are merged, with evidence in the [BE-02 implementation report](../engineering/ai-delivery/implementation-reports/BE-02-implementation-report.md) and the immutable exact-head comments on PR #805. The migration creates zero assignment rows, no distribution behaviour is activated, and merge is not deployment, migration execution, backfill or activation authorization |
| [BE-03 Protected service configuration](../engineering/ai-delivery/tasks/BE-03.md) | `thoth` | HIGH | CLOSED | `develop` at `3b6b3a31f9358011f0c998015dfd0c2508380e83` (specification PR #808 merge commit, verified before any edit) / `develop` | None remaining for BE-03 itself: BE-01 (CLOSED, satisfied) and BE-02 (CLOSED, satisfied). The specification at [`tasks/BE-03.md`](../engineering/ai-delivery/tasks/BE-03.md) is repository-authoritative: its exact CTO-approved content merged into `develop` through PR [#808](https://github.com/thoth-pub/thoth/pull/808) as `3b6b3a31`, which also satisfies the authority condition of the [BE-03/BE-04/APP-01 phase-boundary programme decision](decisions.md) with no separate lifecycle-status edit required. The CTO then separately authorized implementation against that exact `develop` SHA, and the bounded implementation was delivered on `feature/publisher-services/be-03` and merged into `develop` through implementation PR [#809](https://github.com/thoth-pub/thoth/pull/809) after fresh independent exact-head review and explicit CTO merge authorization, as required for every HIGH-risk merge. Post-merge control reconciliation is delivered by [BE-03-CLOSEOUT-01](../engineering/ai-delivery/tasks/BE-03-CLOSEOUT-01.md). Deployment, environment migration execution, production migration, package commercial backfill, assignment creation/backfill, durable job creation, dissemination and distribution activation remain separately gated and unauthorized. | #765 | Specification [#808](https://github.com/thoth-pub/thoth/pull/808); implementation [#809](https://github.com/thoth-pub/thoth/pull/809) | CLOSED - INACTIVE FOUNDATION - the additive migration (configuration token, closed audit-source type, append-only audit table), repository-authoritative `schema.rs`, the single service-configuration write coordinator, the protected owner-and-superuser read, the superuser-only staff report and replace mutation, derived effective capabilities and the connection-scoped `BE-02` lifecycle refactor are merged, with evidence in the [BE-03 implementation report](../engineering/ai-delivery/implementation-reports/BE-03-implementation-report.md) and the immutable exact-head comments on PR #809. The migration creates zero audit rows and changes no package or assignment; no distribution job, dissemination or activation exists; and merge authorized repository integration only, not deployment, environment or production migration execution, backfill or activation |
| [BE-04 Durable distribution jobs](../engineering/ai-delivery/tasks/BE-04.md) | `thoth` | HIGH | IMPLEMENTATION CONTROLLED BY PR #816 - RUNTIME INACTIVE Specification base `develop` at `8c0c54bd7b2e58a645ffe39abd8ceeee86e47686` (the merge commit of specification-addendum PR [#817](https://github.com/thoth-pub/thoth/pull/817), verified before any reconciliation edit); the implementation branch additionally incorporates `develop` at `ec7868a4a44b3d52da5638975995bb66a488b3b4` by ordinary merge, so it is built under the current repository-control doctrine merged through PR [#820](https://github.com/thoth-pub/thoth/pull/820) / `develop` | BE-02 (CLOSED, satisfied); BE-03 (CLOSED, satisfied); [`ADR-0008`](../engineering/decisions/ADR-0008-machine-roles-and-durable-job-primitives.md) (satisfied and repository-authoritative through PR [#815](https://github.com/thoth-pub/thoth/pull/815); a **necessary and not a sufficient** condition, since it approves `DISSEMINATION_WORKER` as a Publisher-Services-specific machine role and keeps BE-04's `distribution_job*` tables, types and lifecycle APIs programme-local, while fixing no operation-level authorization matrix and authorizing no implementation). **Specification authority.** [`tasks/BE-04.md`](../engineering/ai-delivery/tasks/BE-04.md) is repository-authoritative in its corrected form: the approved baseline merged through PR [#814](https://github.com/thoth-pub/thoth/pull/814), and `BE-04-SPEC-ADDENDUM-01` merged through PR [#817](https://github.com/thoth-pub/thoth/pull/817) as `8c0c54bd7b2e58a645ffe39abd8ceeee86e47686`. The baseline and the CTO's implementation authorization on PR #814 against `develop` at `ed32712766c8f5a1951bb53ec3192e18f067c7d2` are **preserved as valid history**: the original candidate was properly authorized work, and no statement here may assert that BE-04 lacked an approved specification or an implementation authorization. That baseline authorization was nevertheless **insufficient** for the corrected contract, for the ordinary reason that the contract it was bound to changed. **Implementation authority.** The CTO issued a fresh implementation reconciliation authorization bound to `develop` at `8c0c54bd7b2e58a645ffe39abd8ceeee86e47686` and to the corrected contract; that authority is GitHub-owned on PR [#816](https://github.com/thoth-pub/thoth/pull/816), which carries the reconciliation on `feature/publisher-services/be-04`. A further bounded review-remediation authorization is GitHub-owned on the repository-local owning issue [#821](https://github.com/thoth-pub/thoth/issues/821), under which the branch incorporates `develop` at `ec7868a4a44b3d52da5638975995bb66a488b3b4` by ordinary merge and carries bounded documentation and control-record corrections; that authorization also covers, narrowly and non-transitively, the automatic `staging-pr-*` container image the repository's normal pull-request workflow publishes from an authorized push, which is a CI side effect and neither a release nor a deployment. Authorization remains granular and non-transitive throughout: none of these authorizes merge, deployment, migration execution or production activation. Repository authority for the implementation depends on the exact approved implementation content becoming reachable from `develop`; live review, authorization, CI and merge state is the GitHub record (`ADR-0005`) and is deliberately not transcribed here. **Runtime.** Automatic job creation is `OFF` by default and is **not activated**: no `distribution_job`, `distribution_job_target` or `distribution_job_attempt` row exists anywhere, no worker exists, no credential exists, and no dissemination occurs. Deployment, environment and production migration execution — whose separate authorization must account for the `SHARE ROW EXCLUSIVE` lock window the migration takes on the existing `publisher` and `work` tables while its two foreign keys are established — worker credential provisioning, worker deployment, the `OFF -> ON` activation, a pilot, dissemination and distribution activation all remain separately gated and unauthorized | Owning repository-local issue [#821](https://github.com/thoth-pub/thoth/issues/821); parent programme issue [#765](https://github.com/thoth-pub/thoth/issues/765). GitHub owns BE-04's live gate, review, authorization and merge state (`ADR-0005`) | Specification [#814](https://github.com/thoth-pub/thoth/pull/814) (baseline, merged); specification addendum `BE-04-SPEC-ADDENDUM-01` [#817](https://github.com/thoth-pub/thoth/pull/817) (merged); implementation and its corrected-contract reconciliation [#816](https://github.com/thoth-pub/thoth/pull/816) on `feature/publisher-services/be-04`. Live review, authorization and merge state for each is the GitHub record | IMPLEMENTATION CONTROLLED BY PR #816 - RUNTIME INACTIVE - the additive migration with the NULL-safe attempt-error `CHECK`, repository-authoritative `schema.rs`, the programme-local domain model, the in-place extension of BE-03's single coordinator transaction, the OFF-by-default fail-closed creation switch, the complete claim/lease/retry/cancellation state machine, the `DISSEMINATION_WORKER` role and its least-privilege matrix, the four additive worker mutations, and the additive job-aware staff report backed by **one** first-level composite `ADR-0007` loader keyed by `publisher_id` are implemented and reconciled with the corrected specification, with evidence in the [BE-04 implementation report](../engineering/ai-delivery/implementation-reports/BE-04-implementation-report.md). The migration creates zero job rows and changes no existing row. Repository integration of this implementation depends on its exact content becoming reachable from `develop`, and merge would authorize repository integration only - not deployment, environment or production migration execution, worker provisioning or deployment, `OFF -> ON` activation, a pilot or dissemination |
| MIG-01 Audit/production backfill | `thoth` + operations | CRITICAL | BLOCKED | dedicated task branch -> `develop`; separately approved production run | BE-01, BE-02 and BE-03 (CLOSED, satisfied); licence audit; dry run; the production audit and backfill controls recorded in [`rollout-plan.md`](rollout-plan.md) and open [CG-13](../engineering/repository-map/control-gaps.md). No package commercial backfill and no production migration execution is authorized | #765 | TBD | NOT STARTED |
| APP-01 Service configuration UI | `thoth-app` | MEDIUM | BLOCKED | current `dev` / `dev` pending BR-APP-01 or exception | BE-03 backend contract (CLOSED, satisfied for APP-01's configuration-only scope: the approved protected API is merged and available). APP-01 itself remains BLOCKED by its other controls: app readiness (BR-APP-01 or explicit CTO exception; the separately specified CG-11 CI closure task); a generated API contract pinned to the exact BE-03 commit SHA per the reserved contract control; and its own approved bounded specification. Scope boundary: under the approved [BE-03/BE-04/APP-01 phase boundary](decisions.md), BE-03 alone supports only APP-01's configuration scope — own-publisher reads of package, effective capability codes and enabled platforms; superuser read/edit; capability-driven UI affordances; backend-driven linked-platform behaviour; optimistic-concurrency handling; and server-normalized state. APP-01 elements rendering durable back-catalogue job status, attempt state, failure state or pending-onboarding state additionally depend on BE-04 and must not be planned against BE-03 alone | #765 | TBD | NOT STARTED |
| APP-02 Staff subscription report | `thoth-app` | MEDIUM | BLOCKED | current `dev` / `dev` pending BR-APP-01 or exception | BE-03 (CLOSED, satisfied); BE-04; APP-01. Its satisfied BE-03 dependency does not make APP-02 ready: it remains blocked on BE-04 and APP-01 | #765 | TBD | NOT STARTED |
| APP-03 API-backed licence options | `thoth-app` | MEDIUM | BLOCKED | current `dev` / `dev` pending BR-APP-01 or exception | LIC-02 | #765 | TBD | NOT STARTED |
| DIS-01 API discovery/comparison | `thoth-dissemination` | HIGH | BLOCKED | `develop` / `develop` | BE-02 (CLOSED, satisfied); MIG-01; BR-DIS-01 or exception | #765 | TBD | NOT STARTED |
| DIS-02 Back-catalogue worker | `thoth-dissemination` | CRITICAL | BLOCKED | `develop` / `develop` | BE-04; DIS-01 clean comparison; production controls | #765 | TBD | NOT STARTED |
| EXP-01 OCLC KBART index | `thoth` | MEDIUM | BLOCKED | `develop` / `develop` | BE-02 (CLOSED, satisfied); ADR-01 OCLC decision; own approved bounded specification | #765 | TBD | NOT STARTED |
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
   the inventory is no longer provisional. The enum was implemented from it
   only by the separately approved and authorized BE-02 task, and no further
   enum may be implemented from it outside another such task. ADR-01 is
   `MERGED - COMPLETE`: an evidence and architecture-decision task, itself
   not runtime implemented and not production ready. CG-07 is `RESOLVED`;
   CG-11 and CG-13 are unchanged.
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
   Metrics behaviour is activated. BE-03 has since exposed the protected
   package/capability contract under its own approval gates, as a merged but
   equally inactive foundation, and the MIG-01 commercial backfill remains a
   separately approved CRITICAL task.
7. Production migration execution and release for BE-01 remain separately
   gated by open CG-13; nothing in BE-01 authorizes production action.
8. BE-02 is `CLOSED`. Its dependencies were all satisfied — ADR-01/the final
   inventory merged through PR #783 (`299b0eff`), ADR-0007 is
   repository-authoritative through PR #800, and the request-local non-cached
   DataLoader foundation merged through PR #802 (`8dcf031d`) — and the
   reconciled BE-02 specification on PR
   [#788](https://github.com/thoth-pub/thoth/pull/788) was independently
   reviewed, explicitly CTO-approved and merged into `develop` as
   `1c752a522f7048963efde00b50565379d7c14b4d`, making it
   repository-authoritative. The CTO then separately authorized BE-02
   implementation against that exact `develop` SHA, and the bounded
   implementation was delivered on `feature/publisher-services/be-02` and
   merged into `develop` through implementation PR
   [#805](https://github.com/thoth-pub/thoth/pull/805), following ADR-0003
   Architecture A (direct `thoth-api/src/schema.rs` edit in the same bounded
   PR as the migration, models, GraphQL contract and tests). Evidence is
   recorded in the
   [BE-02 implementation report](../engineering/ai-delivery/implementation-reports/BE-02-implementation-report.md)
   and the immutable exact-head evidence comments on PR #805; live review,
   authorization and merge evidence lives only in that pull request's record
   under ADR-0005 and is not restated here.
9. The merged BE-02 foundation is inactive and closed: the migration creates
   zero assignment rows, no publisher has an enabled distribution-platform
   assignment created by it, and no distribution consumer, job, dissemination
   or activation exists. BE-03 has since exposed the protected service
   configuration under its own approval gates. Merge authorized repository
   integration only; deployment, environment migration execution, production
   migration, assignment creation or backfill and distribution activation
   remain separately gated and unauthorized.
10. BE-03's BE-01 and BE-02 dependencies are satisfied and its specification is
   **repository-authoritative**: the exact CTO-approved content of
   [`tasks/BE-03.md`](../engineering/ai-delivery/tasks/BE-03.md) merged into
   `develop` through PR
   [#808](https://github.com/thoth-pub/thoth/pull/808) as
   `3b6b3a31f9358011f0c998015dfd0c2508380e83`. Both halves of the authority
   condition therefore hold, so the BE-03/BE-04/APP-01 phase boundary is an
   approved programme decision with no further status edit required. The CTO
   separately authorized implementation against that exact freshly verified
   `develop` head, and the bounded implementation was delivered on
   `feature/publisher-services/be-03` and merged into `develop` through
   implementation PR
   [#809](https://github.com/thoth-pub/thoth/pull/809) following
   ADR-0003 Architecture A (direct `thoth-api/src/schema.rs` edit in the same
   bounded PR as the migration, models, GraphQL contract and tests). Evidence is
   recorded in the
   [BE-03 implementation report](../engineering/ai-delivery/implementation-reports/BE-03-implementation-report.md);
   live review, authorization and merge evidence lives only in that pull
   request's record under ADR-0005 and is not restated here. BE-03 is `CLOSED`
   as an inactive foundation: merge authorized repository integration only —
   deployment, environment migration execution, production migration, package
   commercial backfill, assignment creation or backfill, durable job creation,
   dissemination and distribution activation remain separately gated and
   unauthorized. Post-merge control reconciliation is delivered by
   [BE-03-CLOSEOUT-01](../engineering/ai-delivery/tasks/BE-03-CLOSEOUT-01.md).
   The specification carried one programme decision — the
   [BE-03/BE-04/APP-01 phase boundary](decisions.md), under which BE-03 owns
   desired configuration only and creates no durable job, no placeholder job and
   no fabricated job status, and under which APP-01's job-aware elements depend
   on BE-04 rather than on BE-03 alone. The protected configuration surface
   follows ADR-0001 section 4.4 and exposes the current package, the effective
   capability codes derived from BE-01's code-owned
   `ThothPackage::capabilities()` and stored nowhere, and the enabled
   distribution platforms. The implementation fixes one authoritative
   service-configuration write coordinator, so the canonical configuration
   version token and the configuration audit history cannot be bypassed by a
   production writer, and it implements the protected-read authorization as a
   least-privilege per-publisher `PUBLISHER_USER` (or superuser) check with no
   role inheritance, covering package and capability codes alike.
11. APP-01's BE-03 backend-contract dependency is satisfied for its
   configuration-only scope: the approved protected API is merged and
   available. APP-01 itself remains blocked by its other controls — app
   readiness (BR-APP-01 or an explicit CTO exception, and the separately
   specified CG-11 CI closure), the exact BE-03 SHA/schema-pinning contract
   control, and its own approved bounded specification. Under the approved
   phase boundary, only APP-01's configuration scope is satisfiable from
   BE-03 — including reads of effective capability codes; durable
   back-catalogue job status, attempt state, failure state and
   pending-onboarding state require BE-04.
12. Beyond the delivered and merged documentation-only ADR-01 implementation,
    ADR-01-CLOSEOUT-01 control reconciliation, the shared DataLoader
    foundation and the merged inactive BE-02 and BE-03 foundations, no BE-04,
    MIG-01, APP-01, APP-02, OAI-PMH, deployment, release, environment or
    production migration, package commercial backfill, assignment creation or
    backfill, durable job creation, dissemination, distribution activation,
    `OBSERVE`/`ENFORCE` transition, workflow change or manual dispatch, or PR
    #799 action is authorized; all licence, migration, app, dissemination and
    operational tasks remain blocked under their recorded dependencies.
    Repository authority, specification work and a merged implementation do
    not authorize runtime change, credential use, workflow dispatch or
    production access.
13. The cross-programme machine-role and durable-job questions raised by BE-04
    are decided by
    [`ADR-0008`](../engineering/decisions/ADR-0008-machine-roles-and-durable-job-primitives.md),
    which the CTO approved on 2026-08-14. Machine authorization uses
    dedicated, least-privilege, domain-specific project roles with no generic
    catch-all machine role; `DISSEMINATION_WORKER` is approved as a
    Publisher-Services-specific machine role for the BE-04/DIS-02 durable
    distribution workflow; the seven listed durable-job and concurrency
    conventions are shared conventions rather than a shared job framework, so
    `FOR UPDATE SKIP LOCKED` and each other convention must still be justified
    by the adopting task, while every other concurrency or retry mechanism BE-04
    uses remains governed by the existing repository controls and BE-04's own
    approved specification rather than by `ADR-0008`; and BE-04's
    `distribution_job`, `distribution_job_target` and
    `distribution_job_attempt` tables, Rust domain types and lifecycle APIs
    remain programme-local and may not be reused by another programme by
    analogy. Under the repository's existing process controls, rather than as
    approved decision content, the decision is repository-authoritative only
    when its exact approved content is independently reviewed at its exact head
    and reachable from `develop`; it is not effective from an unmerged branch.
    **That condition is now met**: the exact approved `ADR-0008` content merged
    into `develop` through PR
    [#815](https://github.com/thoth-pub/thoth/pull/815), so `ADR-0008` is
    repository-authoritative and this BE-04 architecture/control prerequisite is
    **satisfied**. Satisfying it is a necessary and not a sufficient condition:
    `ADR-0008` fixes no
    BE-04 operation-level authorization matrix, approves no BE-04
    specification candidate, and authorizes no machine-role creation, role
    provisioning, identity-provider change, migration, GraphQL change, worker
    deployment, durable job creation, dissemination, deployment or production
    access. BE-04 does not become `READY` merely because `ADR-0008` is
    satisfied. That prerequisite was satisfied before, and independently of, the
    CTO's own approval of the BE-04 specification and authorization of BE-04
    implementation, both of which are recorded on PR
    [#814](https://github.com/thoth-pub/thoth/pull/814); `ADR-0008` supplied
    neither. BE-04 remains `HIGH`, and its corrected specification content
    in `BE-04-SPEC-ADDENDUM-01` is repository-authoritative through PR
    [#817](https://github.com/thoth-pub/thoth/pull/817); item 15 records what it
    corrected and item 16 records the reconciliation performed against it.
14. BE-04's BE-02, BE-03 and `ADR-0008` dependencies are **satisfied**, and the
    bounded BE-04 specification
    [`tasks/BE-04.md`](../engineering/ai-delivery/tasks/BE-04.md) was authored on
    `feature/publisher-services/be-04-spec` and **merged into `develop` through
    PR [#814](https://github.com/thoth-pub/thoth/pull/814)** after independent
    review and explicit CTO specification approval. That merged content is the
    **approved baseline** and is repository-authoritative. The CTO then
    separately and explicitly **authorized BE-04 implementation** on the same
    pull request, bound to the freshly verified base `develop` at
    `ed32712766c8f5a1951bb53ec3192e18f067c7d2`. Both facts are preserved as
    historical authority and neither is withdrawn by anything recorded here;
    live approval, authorization and merge evidence is the GitHub pull-request
    record (`ADR-0005`). The corrected content of `BE-04-SPEC-ADDENDUM-01` is a
    separate amendment whose approval authority is the CTO and whose authority
    condition is that the exact CTO-approved content be reachable from `develop`.
    The baseline authorization is insufficient for implementation against it:
    that requires the corrected content to be repository-authoritative, a freshly
    verified base, and a **new** explicit CTO implementation authorization bound
    to that base. An implementation branch `feature/publisher-services/be-04` and an
    implementation pull request exist; that is observed repository state and is
    **not** an approval or a delivery. The corrected-contract reconciliation
    those gates required was subsequently authorized and performed, and is
    recorded in item 16. The specification settles the
    durable job/target/attempt model,
    the deterministic deduplication identity derived from BE-02's shared
    linked-group `activation_id`, the complete creation matrix, the extension of
    BE-03's existing single configuration transaction, the claim/lease/retry/
    cancellation state machine, a narrow least-privilege `DISSEMINATION_WORKER`
    machine role, the additive worker API and the additive job-aware staff
    report. The cross-programme machine-role and durable-job question it
    escalated is **resolved** by `ADR-0008` under item 13, which approves
    `DISSEMINATION_WORKER` as Publisher-Services-specific, keeps BE-04's
    `distribution_job*` tables, Rust domain types and lifecycle APIs
    programme-local, and creates no generic shared job framework; Thoth Metrics
    WP5 does not use that role, and its eventual role name and permissions remain
    its own work. The candidate itself settles no shared service-role convention,
    creates no ADR and adds no programme decision, and `ADR-0008` approves no part
    of it. Historical pre-merge record: independent review of the specification
    candidate identified remediation requirements — `OFF`-mode onboarding loss,
    the five-attempt budget, a repair being treated as delivery evidence, and a
    claim statement that returned no rows — which were corrected on the
    specification branch together with the `ADR-0008` reconciliation, after which
    the content received explicit CTO specification approval and merged as the
    approved baseline. Three further specification defects, found by measuring
    the implementation candidate against that baseline, are reconciled by
    `BE-04-SPEC-ADDENDUM-01` under item 15, whose approval authority is the CTO
    and whose authority condition is that the exact CTO-approved content be
    reachable from `develop`.
    Authoring the specification candidate created **no** relation, migration,
    runtime code, GraphQL surface, role or job. The later implementation
    candidate does contain a migration and runtime code, but it is **unmerged**,
    so nothing it contains is repository-authoritative and nothing it contains
    has been executed anywhere: on `develop` there is still no
    `distribution_job`, `distribution_job_target` or `distribution_job_attempt`
    relation, **no distribution job exists**, automatic job creation remains
    nonexistent and **not active**, and no dissemination, deployment, migration
    execution or production action has occurred or is authorized.
15. **`BE-04-SPEC-ADDENDUM-01` reconciles three defects that a fresh independent
    review found by measuring the BE-04 implementation candidate against the
    approved baseline specification.** They are specification defects, not only
    implementation defects, so they are corrected in
    [`tasks/BE-04.md`](../engineering/ai-delivery/tasks/BE-04.md) section 34
    rather than worked around in code:
    - **Finding A — the attempt-error `CHECK` was not NULL-safe.** PostgreSQL
      rejects a row only when a `CHECK` evaluates to `FALSE` and admits it when
      the result is `UNKNOWN`, so the specified
      `(error_code IS NULL AND error_detail IS NULL) OR result = 'FAILED'`
      accepted an **open** attempt carrying error fields. Section 7.4 now
      specifies a NULL-safe expression with its truth table, and section 25.4
      fixes the required rejections and acceptances.
    - **Finding B — the exact report statement-count contract was
      unsatisfiable.** Two of the three specified DataLoaders were keyed by
      `distribution_job_id`, which exists only after the latest-job loader
      resolves, so their dispatch count is scheduler-dependent and provably
      bounded only by the page size. Section 17.4 now requires **one**
      first-level composite loader keyed by `publisher_id` that resolves the job,
      its targets and its attempts in three set-based statements per dispatch
      chunk. The arithmetic is stated **per dispatch chunk** —
      `2 + 3 * C_job_nonempty + 1 * C_job_empty + 1 * C_assign` — giving five
      (job-only) and six (full report) on a page containing at least one job, and
      three and four on a page with none; loader dispatch expectations are stated
      per loader rather than as a blanket rule; and the shared `ADR-0007`
      chunking property is named as `ADR-0007`'s rather than restated as BE-04's.
      The contract was corrected by design change, **not** by weakening it to
      "usually five" or "observed bounded", and no `ADR-0007` value was changed.
      An unexpected chunk count blocks BE-04 and must be classified on evidence
      before anything is concluded about the shared foundation; no claim is made
      about BE-02's or any other field's loader without verifying that field.
    - **Finding C — a mandatory command that fails at the authorized base.**
      `cargo test -p thoth-client` does not build at `develop` either, because
      `thoth-api` does not compile with its `backend` feature off and only the
      workspace and build-dependency edges enable it. It is removed as a BE-04
      gate, replaced by the workspace runs the repository's own CI already uses
      with an explicit requirement that `thoth-client`'s tests be shown to have
      executed within them, and recorded as pre-existing repository
      packaging/test-mode debt for a separate task. It is **not** reported as
      passing and **not** silently waived.

    The addendum is documentation and control only. It changes no runtime code,
    migration, generated contract, workflow or manifest; it approves no
    specification; and it authorizes no implementation. It neither extends nor
    withdraws the baseline implementation authorization of item 14, which was
    real, was bound to `develop` at `ed32712766c8f5a1951bb53ec3192e18f067c7d2`
    and to the baseline contract, and remains valid history. That authorization
    is insufficient for implementation against the corrected contract, for the
    ordinary reason that both the requirements and the base have moved. The
    continuation gates it set out — independent review of the addendum, explicit
    CTO approval of the corrected specification, merge into `develop`, fresh
    verification of the new `develop` head, and a **new** explicit CTO
    implementation authorization bound to that head — are the gates item 16
    records as satisfied.
16. **The implementation candidate has been reconciled against the corrected
    contract on PR [#816](https://github.com/thoth-pub/thoth/pull/816), under a
    fresh CTO implementation reconciliation authorization** bound to `develop` at
    `8c0c54bd7b2e58a645ffe39abd8ceeee86e47686` — the merge commit of PR #817 —
    and to the corrected contract. That authority is GitHub-owned and is not
    transcribed here. The reconciliation incorporated the authorized `develop`
    base into `feature/publisher-services/be-04` by ordinary merge, without
    rewriting published history, and made exactly the two bounded corrections the
    addendum required: the NULL-safe attempt-error `CHECK`, proven as a full
    three-valued truth table on `INSERT` and `UPDATE`; and **one** first-level
    request-local composite `ADR-0007` loader keyed by `publisher_id` whose value
    is the complete `latestBackCatalogueJob` field, replacing the rejected nested
    chain, with the second-level target and attempt loaders retained only for the
    single-job mutation payloads and recording zero dispatches on the report path.
    The section 25.12 test now derives every expected statement count from the
    measured per-chunk classification. No `ADR-0007` value was changed, no
    look-ahead or request-scoped result store was introduced, no unrelated loaders
    were merged, and `BE-04.md` itself was not edited. Evidence is in the
    [BE-04 implementation report](../engineering/ai-delivery/implementation-reports/BE-04-implementation-report.md).
    Repository authority for the implementation depends on its exact content
    becoming reachable from `develop`; live review, authorization, CI and merge
    state is the GitHub record (`ADR-0005`). Runtime is unchanged and inactive:
    automatic job creation remains `OFF` by default and unactivated, no
    `distribution_job`, `distribution_job_target` or `distribution_job_attempt`
    row exists anywhere, no worker and no credential exists, and no dissemination,
    deployment, environment or production migration execution, worker
    provisioning, pilot or production access has occurred or is authorized.
