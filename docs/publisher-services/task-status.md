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
| [ADR-01 Platform inventory/final architecture](../engineering/ai-delivery/tasks/ADR-01.md) | `thoth` | MEDIUM | READY | exact base recorded when the implementation branch is created from then-current `develop`; then-current `develop` / `develop` | approved ADR-01 specification merged. `READY` does not authorize implementation: creating `feature/publisher-services/adr-01` and making any implementation edit require separate explicit authorization; the branch remains absent until then. ADR-01 is not blocked by BE-01. | [#765](https://github.com/thoth-pub/thoth/issues/765) | Specification TBD until opened; TBD for implementation | APPROVED SPECIFICATION - READY FOR SEPARATELY AUTHORIZED IMPLEMENTATION - BRANCH NOT AUTHORIZED - IMPLEMENTATION NOT STARTED - final distribution-platform inventory remains provisional |
| LIC-01 Expand `cc-license` | `cc-license` | MEDIUM | BLOCKED | `develop` / `develop` | P0-01; BR-LIC-01 or CTO exception; approved spec | #765 | TBD | NOT STARTED |
| LIC-02 Enforce supported licences | `thoth` | HIGH | BLOCKED | `develop` / `develop` | LIC-01 release; production licence audit plan | #765 | TBD | NOT STARTED |
| [BE-01 Publisher package model](../engineering/ai-delivery/tasks/BE-01.md) | `thoth` | HIGH | READY | exact base recorded when the implementation branch is created from then-current `develop`; then-current `develop` / `develop` | approved BE-01 specification merged; ADR-0003 / `THOTH-DB-CTRL-02` (PR #778) merged into `develop`, resolving CG-12 and recording BE-01 `READY`. `READY` does not authorize implementation: creating `feature/publisher-services/be-01` and making any implementation edit require separate explicit authorization; the branch remains absent until then. | [#765](https://github.com/thoth-pub/thoth/issues/765) | Specification [#774](https://github.com/thoth-pub/thoth/pull/774); TBD for implementation | APPROVED SPECIFICATION - READY FOR SEPARATELY AUTHORIZED IMPLEMENTATION (Architecture A) - BRANCH ABSENT - IMPLEMENTATION NOT STARTED |
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

1. The
   [`BE-01` specification](../engineering/ai-delivery/tasks/BE-01.md) is
   repository-authoritative: specification
   [PR #774](https://github.com/thoth-pub/thoth/pull/774) was independently
   approved and merged.
2. The shared Diesel schema control is Architecture A (ADR-0003):
   `thoth-api/src/schema.rs` is maintained directly, and BE-01 edits it in its
   own bounded PR. `THOTH-DB-CTRL-01` is `SUPERSEDED`; its replacement
   `THOTH-DB-CTRL-02` delivers ADR-0003 through PR #778 and merges into
   `develop` after independent review and explicit CTO merge authorization.
3. Merging PR #778 resolves CG-12 and records BE-01 as `READY`; no separate
   control update is required. `READY` does not authorize implementation by
   itself.
4. Creating `feature/publisher-services/be-01` and making any implementation
   edit require separate explicit authorization. The branch is then created from
   the freshly verified then-current `develop`, with the exact base recorded
   before any implementation edit, and remains absent until that authorization.
5. ADR-01 specification and final distribution-platform inventory work may
   proceed separately under their own approval gates.
6. BE-01-SPEC unlocks no BE-01 implementation edit, BE-02, BE-03, OAI-PMH,
   deployment, release or
   production work; all licence, migration, app, dissemination and operational
   tasks remain blocked.
