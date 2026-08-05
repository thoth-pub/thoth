# Publisher Services Task Status

Status: ACTIVE TRACKER
Programme owner: CTO
Master issue: [#765](https://github.com/thoth-pub/thoth/issues/765)
Approved design: [private Google Doc](https://docs.google.com/document/d/1kr2Ft0Y4pxgcXGyFAKs_wfFx4I0jlxEvaceswE5Dus8/edit), Drive revision `3`
Last updated: 2026-08-05

## 1. Control rule

Publisher Services follows the approved design's one-task/one-branch/one-PR workflow. It does not use a long-lived programme integration branch.

No task moves to `READY` without an approved specification, architecture dependencies, verified repository/base/target, branch-readiness completion or CTO exception, named implementation/review models, tests, migration, rollout and rollback.

`TBD` is missing work, not implicit approval.

## 2. Task tracker

| Task | Repository | Risk | Status | Verified base / PR target | Blocking dependencies | Issue | PR | Acceptance |
|---|---|---:|---|---|---|---|---|---|
| P0-01 Control documents and tracker | `thoth` | LOW | CLOSED | `develop` at `5b406e4ef9b5c192cc38eb8a97a41bbd0fc3bc06` / `develop` | None; issue #765 synchronization is a separately authorized external mirror of the completed repository closeout | [#765](https://github.com/thoth-pub/thoth/issues/765) | Foundation [#764](https://github.com/thoth-pub/thoth/pull/764); closeout [#767](https://github.com/thoth-pub/thoth/pull/767) merged as `bac598e3`; finalization [#768](https://github.com/thoth-pub/thoth/pull/768) | CLOSED - PR #767 independently `APPROVED` and merged as `bac598e32abbd0d7e69ff467c82945ee00df02ba` on 2026-07-27; reviewed content head `d72137893ddea512c0d05c81d310eb59d045cd2b`; repository-finalized |
| ADR-01 Platform inventory/final architecture | `thoth` | MEDIUM | BLOCKED | `develop` / `develop` | missing approved bounded ADR-01 specification; final distribution-platform inventory decision | #765 | TBD | NOT STARTED |
| LIC-01 Expand `cc-license` | `cc-license` | MEDIUM | BLOCKED | `develop` / `develop` | P0-01; BR-LIC-01 or CTO exception; approved spec | #765 | TBD | NOT STARTED |
| LIC-02 Enforce supported licences | `thoth` | HIGH | BLOCKED | `develop` / `develop` | LIC-01 release; production licence audit plan | #765 | TBD | NOT STARTED |
| [BE-01 Publisher package model](../engineering/ai-delivery/tasks/BE-01.md) | `thoth` | HIGH | IMPLEMENTED | `develop` at `37b802776ae6853affe19d90156f3c1e0654ebe3` (PR #778 merge commit, verified before any edit) / `develop` | Separately authorized bounded implementation delivered on `feature/publisher-services/be-01` under ADR-0003 Architecture A. Merge of implementation PR [#779](https://github.com/thoth-pub/thoth/pull/779) requires fresh independent exact-head cross-model review and explicit CTO merge authorization; production migration/release remains separately gated by CG-13. This row becomes authoritative when PR #779 merges into `develop`. | [#765](https://github.com/thoth-pub/thoth/issues/765) | Specification [#774](https://github.com/thoth-pub/thoth/pull/774); implementation [#779](https://github.com/thoth-pub/thoth/pull/779) | INACTIVE FOUNDATION MERGED - all publishers `OASIS`; no consumer, package API, mutation, UI, distribution, OAI or Metrics behaviour activated; retained-foundation operational rollback applies; see the [BE-01 implementation report](../engineering/ai-delivery/implementation-reports/BE-01-implementation-report.md) |
| BE-02 Distribution platform model | `thoth` | HIGH | BLOCKED | `develop` / `develop` | ADR-01 | #765 | TBD | NOT STARTED |
| BE-03 Protected service configuration | `thoth` | HIGH | BLOCKED | `develop` / `develop` | BE-01; BE-02 | #765 | TBD | NOT STARTED |
| BE-04 Durable distribution jobs | `thoth` | HIGH | BLOCKED | `develop` / `develop` | BE-02; BE-03 | #765 | TBD | NOT STARTED |
| MIG-01 Audit/production backfill | `thoth` + operations | CRITICAL | BLOCKED | dedicated task branch -> `develop`; separately approved production run | BE-01/02/03; licence audit; dry run | #765 | TBD | NOT STARTED |
| APP-01 Service configuration UI | `thoth-app` | MEDIUM | BLOCKED | current `dev` / `dev` pending BR-APP-01 or exception | BE-03; generated API contract | #765 | TBD | NOT STARTED |
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

1. The bounded BE-01 implementation was separately authorized by the CTO on
   2026-08-05 and delivered on `feature/publisher-services/be-01` from the
   verified base `37b802776ae6853affe19d90156f3c1e0654ebe3` through
   implementation [PR #779](https://github.com/thoth-pub/thoth/pull/779),
   following ADR-0003 Architecture A (direct `thoth-api/src/schema.rs` edit in
   the same bounded PR as the migration, models and tests). Evidence is
   recorded in the
   [BE-01 implementation report](../engineering/ai-delivery/implementation-reports/BE-01-implementation-report.md).
2. PR #779 merges only after fresh independent exact-head cross-model review
   and explicit CTO merge authorization. The merged foundation remains
   inactive: no consumer, package API, mutation, UI, distribution, OAI or
   Metrics behaviour is activated, and MIG-01 commercial backfill remains a
   separately approved CRITICAL task.
3. Production migration execution and release remain separately gated by
   CG-13; nothing in BE-01 authorizes production action.
4. ADR-01 specification and final distribution-platform inventory work may
   proceed separately under their own approval gates.
5. BE-01 unlocks no BE-02, BE-03, BE-04, OAI-PMH, deployment, release or
   production work; all licence, migration, app, dissemination and operational
   tasks remain blocked under their recorded dependencies.
