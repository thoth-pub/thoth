# Publisher Services and Distribution Configuration

Status: CONTROL FOUNDATION CLOSED; BE-01 CLOSED; ADR-01-SPEC-AMEND-01 CORRECTED CONTENT APPROVED (INDEPENDENT REVIEW AND CTO APPROVAL AT 1276c70a; MERGE PENDING); ADR-01 IMPLEMENTATION BLOCKED; FINAL PLATFORM INVENTORY STILL PROVISIONAL; ALL OTHER IMPLEMENTATION GATED
Programme owner: CTO
Primary coordinating repository: `thoth-pub/thoth`
Related repositories:

- `thoth-pub/thoth-app`
- `thoth-pub/thoth-dissemination`
- `thoth-pub/cc-license`

Deferred implementation:

- OAI-PMH work currently associated with `feature/oai-pmh-http` in `thoth`

## 1. Purpose

This directory is the repository-backed control surface for implementing publisher packages, explicit distribution-platform assignments, durable back-catalogue jobs, licence enforcement, staff interfaces, dissemination cutover, OCLC KB feed discovery and later OAI-PMH eligibility.

The approved design is the [private Google Doc](https://docs.google.com/document/d/1kr2Ft0Y4pxgcXGyFAKs_wfFx4I0jlxEvaceswE5Dus8/edit), Drive revision `3`, indexed in [`docs/engineering/design-references.md`](../engineering/design-references.md). These files turn that design into:

- a decision register;
- an executable task/dependency tracker;
- a verified platform-inventory baseline;
- acceptance evidence requirements;
- rollout and rollback gates.

## 2. Programme outcome

Thoth becomes authoritative for desired publisher service configuration:

- every publisher has exactly one non-null package;
- every publisher has an explicit set of enabled distribution platforms;
- package and platform configuration are independent;
- publisher users may read their own configuration;
- only superusers may change it;
- durable jobs represent automatic back-catalogue work;
- dissemination temporarily remains the push-delivery executor;
- legacy publisher-ID environment lists are removed only after comparison and observation;
- OAI-PMH later uses package capability plus canonical open-licence and lifecycle rules.

## 3. Programme non-goals

This programme does not initially:

- implement work-level distribution choices;
- port every uploader to Rust;
- add complete per-work/per-platform observed delivery state;
- add general metadata-change events or withdrawals;
- make package choice imply platform assignments;
- expose package values anonymously;
- merge distribution and metrics platform domains;
- activate deferred OAI-PMH before its dependencies and branch assessment pass.

## 4. Authority and required reading

Read in this order:

1. [private approved Publisher Services design](https://docs.google.com/document/d/1kr2Ft0Y4pxgcXGyFAKs_wfFx4I0jlxEvaceswE5Dus8/edit);
2. approved cross-programme ADRs under `docs/engineering/decisions/`;
3. this directory;
4. task specifications;
5. repository-local `AGENTS.md`;
6. live code, migrations, PRs and CI.

Where sources conflict, stop and escalate. Chat history is not authoritative.

## 5. Current programme decision

```text
CONTROL FOUNDATION CLOSED
BE-01 CLOSED (INACTIVE FOUNDATION MERGED THROUGH PR #779)
CG-12 RESOLVED BY ADR-0003 ARCHITECTURE A
ADR-01-SPEC-AMEND-01 CONTENT APPROVED - APPROVAL-STATE REVIEW AND MERGE
PENDING
CORRECTED ADR-01 CONTENT INDEPENDENTLY REVIEWED (4873802457 - APPROVED) AND
EXPLICITLY CTO-APPROVED (COMMENT 5203642323, 2026-08-06) AT EXACT HEAD
1276c70a81e73f57d833eecb0e6886bd0cabf69e
REPOSITORY AUTHORITY PENDING MERGE OF PR #781; APPROVAL-STATE HEAD REQUIRES
FRESH INDEPENDENT EXACT-HEAD REVIEW
HISTORICAL ADR-01 SPECIFICATION APPROVAL PRESERVED; APPLIES ONLY TO THE
SUPERSEDED PRE-AMENDMENT CONTENT
ADR-01 IMPLEMENTATION BLOCKED PENDING AMENDMENT MERGE PLUS FRESH
IMPLEMENTATION AUTHORIZATION AND A NEW EXACT DEVELOP BASE
FINAL DISTRIBUTION-PLATFORM INVENTORY REMAINS PROVISIONAL
ALL OTHER IMPLEMENTATION REMAINS GATED
```

Achieved:

- `ADR-0001` publisher package capabilities is `APPROVED AND MERGED` (Javi, CTO,
  2026-07-28, approval PR
  [#772](https://github.com/thoth-pub/thoth/pull/772)). The independently
  reviewed approval record merged on 2026-07-29 as
  `b2c91ff25b95ab0e10a477ff21dbd4702f5db8d4`. The shared capability
  decision dependency is satisfied. Approval and merge preserve package and
  distribution-platform independence and do not implement package storage,
  capability enforcement or distribution behaviour.
- `ADR-0002` platform domain boundaries is `APPROVED` (CTO, 2026-07-27, approval
  PR [#769](https://github.com/thoth-pub/thoth/pull/769)). This removes one
  shared-ADR dependency and does not, by itself, make ADR-01, BE-02 or any other
  task ready.
- The bounded
  [`BE-01` implementation specification](../engineering/ai-delivery/tasks/BE-01.md)
  is approved and repository-authoritative following the merge of
  documentation-only specification
  [PR #774](https://github.com/thoth-pub/thoth/pull/774). The current BE-01
  delivery state is recorded only in
  [`task-status.md`](task-status.md) and the BE-01 pull-request record.
- The shared Diesel schema-authority control is settled:
  [ADR-0003](../engineering/decisions/ADR-0003-repository-authoritative-schema-contract.md)
  selects Architecture A, delivered by `THOTH-DB-CTRL-02` through
  [PR #778](https://github.com/thoth-pub/thoth/pull/778), and
  [CG-12](../engineering/repository-map/control-gaps.md) is `RESOLVED`.
  `THOTH-DB-CTRL-01` is `SUPERSEDED`.
- Historical record: the pre-amendment
  [`ADR-01` implementation specification](../engineering/ai-delivery/tasks/ADR-01.md)
  at exact content head `820f9cfa22d284f8f347db338aa2461408f4ed12` was
  independently reviewed and explicitly CTO-approved (Javi, CTO, 2026-08-05),
  and that historical content became repository-authoritative when
  specification [PR #780](https://github.com/thoth-pub/thoth/pull/780)
  merged. That approval remains a valid historical record and applies only to
  the superseded pre-amendment content. It defined how the future ADR-01
  implementation determines the final distribution-platform inventory, and it
  finalized no inventory. The currently linked `ADR-01.md` is amended
  content with status `APPROVED CORRECTED CONTENT - MERGE PENDING`. ADR-01
  implementation is not authorized: it remains blocked pending fresh
  independent exact-head review of the approval-state head, separate CTO
  merge authorization, merge of the amendment, and fresh implementation
  authorization from a new verified `develop` base.
- The bounded
  [`ADR-01-SPEC-AMEND-01`](../engineering/ai-delivery/tasks/ADR-01-SPEC-AMEND-01.md)
  amendment task, authorized by the CTO on 2026-08-06, corrects and extends
  the historically approved pre-amendment ADR-01 specification from the CTO-approved
  [evidence ledger](adr-01-evidence-ledger.md) (EBSCO, ProQuest and
  knowledge-base distribution; Project MUSE defect reclassification;
  destination-versus-adapter distinction; Jisc NBK; shared OCLC KBART feed;
  conservative update/withdrawal policy; Thoth-managed source-file
  invariant). The corrected content was independently reviewed (review
  `4873802457`, `APPROVED`, no findings) and explicitly CTO-approved (Javi,
  CTO, 2026-08-06, PR #781 comment `5203642323`) at exact content head
  `1276c70a81e73f57d833eecb0e6886bd0cabf69e`; it becomes
  repository-authoritative when PR #781 merges. The approval-state
  documentation head recording this approval is status-only and requires
  fresh independent exact-head review and separate CTO merge authorization.
  The historical ADR-01 specification approval is preserved and applies only
  to the superseded pre-amendment content. ADR-01 implementation remains
  blocked pending the amendment merge plus fresh implementation
  authorization from a new exact `develop` base; the existing local
  pre-amendment `feature/publisher-services/adr-01` branch is clean,
  unpushed, not authoritative and not used.

Specification approval and gated implementation:

1. The P0-01 control foundation is `CLOSED`. It merged through
   [PR #764](https://github.com/thoth-pub/thoth/pull/764) at
   `5b406e4ef9b5c192cc38eb8a97a41bbd0fc3bc06`, and its closeout was
   independently `APPROVED` and merged through
   [PR #767](https://github.com/thoth-pub/thoth/pull/767) at
   `bac598e32abbd0d7e69ff467c82945ee00df02ba`. P0-01 closure records the
   engineering-control foundation only; it does not approve an ADR, approve the
   final inventory, satisfy branch readiness, or make any implementation task
   ready.
2. BE-01 will add only the package-storage and code-owned capability foundation.
   It creates no public or protected package query, no package mutation, no
   distribution-platform behaviour, and no OAI-PMH or Metrics activation.
3. The shared CG-12 schema-authority question is resolved by ADR-0003
   Architecture A: `thoth-api/src/schema.rs` is the repository-authoritative,
   manually maintained Diesel schema contract, and a task that changes the
   Diesel-representable contract updates the migration, `schema.rs`, models and
   tests atomically in one bounded PR.
4. An implementing agent consumes and verifies that merged shared control. It
   must not independently establish, redefine, or repair it inside a programme
   task.
5. Every implementation task records its exact base only when its branch is
   created from the then-current verified `develop`, under separate explicit
   authorization.
6. Protected package and effective-capability reads and the dedicated
   superuser package mutation remain BE-03 scope.
7. ADR-01 will determine the final distribution-platform inventory. Until an
   approved ADR-01 merges, [`platform-inventory.md`](platform-inventory.md)
   remains an explicitly provisional baseline and must not be treated as the
   final enum.

Reasons all other implementation remains gated:

1. Publisher Services `ADR-01` has not finalized or approved the
   distribution-platform enum or final distribution-platform inventory. `BE-02`
   requires the merged ADR-01 implementation, not the ADR-01 specification
   alone.
2. Every task still requires its own approved bounded specification, its
   applicable dependencies, and separate explicit authorization before any
   implementation branch or edit.
3. Branch-readiness tasks are required before work in repositories whose
   current topology differs from policy. `BR-APP-01` and the separately
   specified CG-11 CI closure task remain outstanding for `thoth-app`, and
   `CG-13` remains open for `thoth` runtime operations.
4. A specification task changes documentation and control state only. It
   changes no runtime code, migration, database, GraphQL/API, authorization,
   deployment, release, production service or production behaviour.

Discovery, review, documentation, and read-only orientation may continue. An
approved specification makes a task's requirements repository-authoritative; it
does not create the implementation branch, authorize an implementation edit, or
unlock `BE-02`, `BE-03`, `BE-04`, `APP-01`, OAI-PMH, release, deployment or
production work.

## 6. Files

- `decisions.md` - settled, proposed and unresolved decisions.
- `task-status.md` - task dependencies, repository, branch, risk and evidence status.
- `platform-inventory.md` - verified current dissemination baseline and ADR-01 questions.
- `adr-01-evidence-ledger.md` - sanitized CTO-approved evidence ledger for the ADR-01 specification amendment.
- `acceptance-matrix.md` - programme requirements mapped to evidence.
- `rollout-plan.md` - additive rollout, comparison, pilot, observation and rollback.
- `master-issue.md` - body for the programme's GitHub tracking issue.

## 7. Status vocabulary

- `PLANNED` - scoped in the approved design but not ready.
- `BLOCKED` - cannot safely start because a prerequisite is missing.
- `READY` - written specification and dependencies are approved.
- `IN PROGRESS` - one approved branch/PR is active.
- `CHANGES REQUIRED` - review found blocking work.
- `APPROVED` - independently reviewed and merge-ready.
- `MERGED` - repository merge complete.
- `ROLLED OUT` - intended environment activation complete.
- `CLOSED` - observation, reconciliation and tracker updates complete.

A task is not complete merely because code exists or CI passes.

## 8. Implementation rule

Each implementation task receives:

- an approved written specification;
- one bounded slice branch and PR;
- exact base and target branches;
- risk classification;
- required tests;
- migration, rollout and rollback sections;
- independent review.

Implementers may not merge, deploy, access production secrets or approve their own work.
