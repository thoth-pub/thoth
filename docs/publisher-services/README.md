# Publisher Services and Distribution Configuration

Status: CONTROL FOUNDATION CLOSED; BE-01 SPECIFICATION APPROVED AFTER PR #774 MERGES; BE-01 IMPLEMENTATION BLOCKED ON SHARED DIESEL CONTROL; ALL OTHER IMPLEMENTATION GATED
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
BE-01 SPECIFICATION APPROVED AFTER PR #774 MERGES
BE-01 IMPLEMENTATION BLOCKED ON SHARED DIESEL CONTROL
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
  is approved through this documentation-only specification
  [PR #774](https://github.com/thoth-pub/thoth/pull/774). When PR #774 merges,
  the specification becomes repository-authoritative and read-only orientation
  may continue. BE-01 implementation remains blocked on the shared
  `THOTH-DB-CTRL-01` Diesel generation procedure.

BE-01 specification approval and blocked implementation:

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
3. `THOTH-DB-CTRL-01` is the shared repository task that must resolve CG-12.
   It must be independently approved and merged before BE-01 moves to `READY`,
   before `feature/publisher-services/be-01` is created, or before any
   migration, `schema.rs`, model, test, or other implementation edit.
4. The BE-01 implementing agent must consume and verify the merged
   repository-authoritative Diesel procedure. It must not independently
   establish, redefine, or repair that shared procedure inside BE-01.
5. The exact BE-01 base is recorded only after the shared control passes, when
   the implementation branch is created from the then-current verified
   `develop`.
6. Protected package and effective-capability reads and the dedicated
   superuser package mutation remain BE-03 scope.

Reasons all other implementation remains gated:

1. Publisher Services `ADR-01` has not finalized or approved the
   distribution-platform enum or final distribution-platform inventory.
2. Every task other than BE-01 still requires its own approved bounded
   specification and applicable dependencies.
3. Branch-readiness tasks are required before work in repositories whose
   current topology differs from policy.
4. BE-01-SPEC changes documentation and control state only. It changes no
   runtime code, migration, database, GraphQL/API, authorization, deployment,
   release, production service or production behaviour.

Discovery, review, documentation, and read-only orientation may continue. After
PR #774 merges, `THOTH-DB-CTRL-01` must resolve the shared procedure; only then
may a separate control update move BE-01 from `BLOCKED` to `READY`, followed by
fresh `develop` verification and implementation-branch creation. No BE-01
implementation edit, BE-02, BE-03, OAI-PMH, release, deployment or production
work is unlocked by BE-01-SPEC.

## 6. Files

- `decisions.md` - settled, proposed and unresolved decisions.
- `task-status.md` - task dependencies, repository, branch, risk and evidence status.
- `platform-inventory.md` - verified current dissemination baseline and ADR-01 questions.
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
