# Publisher Services and Distribution Configuration

Status: CONTROL FOUNDATION IN PROGRESS
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
BLOCKED FOR IMPLEMENTATION
```

Reasons:

1. `P0-01` is still part of the unmerged engineering-control foundation.
2. `ADR-0001` package capability model is `PROPOSED`.
3. `ADR-0002` platform domain boundaries is `PROPOSED`.
4. Publisher Services `ADR-01` has not finalized the distribution-platform enum.
5. Branch-readiness tasks are required before work in repositories whose current topology differs from policy.

Discovery and documentation may continue. Production implementation must not start.

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
