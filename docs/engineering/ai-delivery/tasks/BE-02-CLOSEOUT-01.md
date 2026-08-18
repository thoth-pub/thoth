# BE-02-CLOSEOUT-01 - Post-merge control correction for BE-02

Status: APPROVED
Programme: Publisher Services and Distribution Configuration
Repository: `thoth-pub/thoth`
Workflow: STANDARD
Base branch: `develop`
PR target: `develop`
Programme integration branch: None
Risk: LOW
Owner: CTO
Approved by: CTO, explicit instruction transcribed on merged implementation PR
[#805](https://github.com/thoth-pub/thoth/pull/805)
Dependencies: merged BE-02 implementation PR
[#805](https://github.com/thoth-pub/thoth/pull/805)
Target branch name: `feature/publisher-services/be-02-closeout`
Master programme issue:
[#765](https://github.com/thoth-pub/thoth/issues/765)

Authority condition: this record is repository-authoritative when this exact
content is reachable from the repository's authoritative integration branch
(`develop`). Live review, authorization and merge evidence is the GitHub
pull-request record and is not copied here.

## 1. Objective

Correct the materially incorrect active Publisher Services programme and
dependency state left behind after the BE-02 implementation merged: committed
control documents still describe BE-02 as unmerged and still awaiting
independent review and merge authorization, still assert that no
`DistributionPlatform` enum is implemented, and still list BE-02 as a blocking
dependency of BE-03, BE-04, MIG-01, DIS-01 and EXP-01.

This is a documentation and control correction only.

## 2. Background and authority

Authoritative sources:

- [ADR-0005](../../decisions/ADR-0005-terminal-merge-evidence.md), section 8 -
  a post-merge task is *required* when "a committed tracker contains materially
  incorrect programme state", and *not* required merely to record that a pull
  request merged, its merge SHA, its review identifiers or its authorization;
- [`docs/engineering/AGENTS.md`](../../AGENTS.md) section 1.1 - durable versus
  transient state;
- merged BE-02 implementation PR
  [#805](https://github.com/thoth-pub/thoth/pull/805) and the merged code,
  migration and generated contract it delivered;
- the [BE-02 specification](BE-02.md) and
  [BE-02 implementation report](../implementation-reports/BE-02-implementation-report.md).

Current behaviour: `docs/publisher-services/task-status.md` records BE-02 as
`IMPLEMENTED - AWAITING INDEPENDENT REVIEW / MERGE AUTHORIZATION` with PR #805
`(DRAFT, unmerged)`; `docs/publisher-services/README.md`,
`decisions.md`, `rollout-plan.md`, `platform-inventory.md` and
`docs/engineering/repository-map/control-gaps.md` each assert that BE-02
"remains blocked and unauthorized"; `platform-inventory.md` additionally asserts
that no `DistributionPlatform` enum is implemented. Each statement is now false,
and each is operationally misleading about what the programme may and may not do
next.

## 3. Explicit scope

The task must:

1. perform a classified stale-state search over the active Publisher Services
   and engineering control surface, classifying every BE-02 statement as
   `ACTIVE STALE STATE - CORRECT`, `HISTORICAL RECORD - PRESERVE`,
   `CURRENT AND CORRECT - PRESERVE` or `OUT OF SCOPE - PRESERVE`;
2. correct only the statements classified `ACTIVE STALE STATE`, so that active
   controls durably represent:
   - BE-02: `CLOSED - INACTIVE FOUNDATION`;
   - the BE-02 repository implementation: merged;
   - the BE-03 dependency on BE-02: satisfied;
   - deployment, environment migration execution, production migration,
     assignment creation/backfill, distribution activation and
     `OBSERVE`/`ENFORCE`: `NOT AUTHORIZED`;
   - PR #799: untouched and outside the Publisher Services dependency set;
3. record this bounded task and its implementation report;
4. add the required `CHANGELOG.md` entry under `## [Unreleased]`.

## 4. Non-goals

The task must not:

1. change any runtime code, migration, schema, GraphQL contract, generated SDL,
   client artifact, workflow or configuration;
2. create an approval-state-only commit, or copy review identifiers, approval
   identifiers, merge-authorization identifiers, the merge commit SHA or the
   merged timestamp into repository files merely to transcribe them;
3. rewrite historical implementation-time evidence, including the explicitly
   historical sections of the BE-02 implementation report, merely because the
   pull request later merged;
4. edit the historical body or comments of PR #805;
5. modify issue #765;
6. use global find/replace;
7. state or imply that BE-02 is deployed, activated or production-ready;
8. take any action on PR [#799](https://github.com/thoth-pub/thoth/pull/799) or
   on mutation-guard mode;
9. start, specify or authorize BE-03, BE-04, MIG-01, APP-01 or any other task.

## 5. Invariants

The implementation must preserve:

1. ADR-0005 terminal merge evidence: GitHub remains the authority for review,
   authorization and merge lifecycle facts;
2. durable status prose that stays truthful before review, after review, before
   merge and after merge;
3. every historical record, including superseded approvals that applied to
   pre-amendment content;
4. the merged BE-02 public GraphQL surfaces and the inactivity of the merged
   foundation;
5. the separately gated status of every other Publisher Services task.

## 6. Required behaviour

### 6.1 Success behaviour

Active control documents describe BE-02 as a closed, merged, inactive
foundation, and describe every downstream dependency accurately.

### 6.2 Failure behaviour

Not applicable: no runtime behaviour changes.

### 6.3 Authorization

No authorization path changes. No protected surface is added, removed or
altered.

### 6.4 Concurrency and idempotency

Not applicable.

### 6.5 Compatibility

No API, database, client or deployment compatibility surface is touched.

## 7. Data and migration requirements

Migration required: NO

## 8. Observability and operations

Required logs: none.

Required metrics/alerts: none.

Operational runbook changes: none.

## 9. Acceptance criteria

- [ ] every BE-02 statement in the searched surface carries a recorded
      classification;
- [ ] no statement classified `HISTORICAL RECORD`, `CURRENT AND CORRECT` or
      `OUT OF SCOPE` is modified;
- [ ] `docs/publisher-services/task-status.md` records BE-02 as `CLOSED` with a
      durable acceptance statement;
- [ ] no active control document asserts that BE-02 is unmerged, awaiting
      review, awaiting merge authorization, blocked or unauthorized;
- [ ] no active control document asserts that no `DistributionPlatform` enum is
      implemented;
- [ ] deployment, migration execution, backfill, activation and
      `OBSERVE`/`ENFORCE` are recorded as `NOT AUTHORIZED`;
- [ ] no review, approval or merge identifier, merge SHA or merge timestamp is
      newly transcribed into a repository file;
- [ ] the diff touches documentation paths only;
- [ ] `git diff --check` reports no whitespace error;
- [ ] `CHANGELOG.md` has one entry under `## [Unreleased]`.

## 10. Required tests

### Unit

Not applicable: documentation-only change.

### Integration/database

Not applicable.

### Authorization/security

Not applicable.

### Regression

- confirm the diff contains no `thoth-api/`, `thoth-api-server/`,
  `thoth-client/`, `thoth-export-server/`, `thoth-errors/`, `migrations/` or
  `.github/` path.

### Manual verification

- re-read each corrected paragraph and confirm the wording stays truthful
  before and after this task's own pull request merges;
- confirm every relative link resolves.

### Performance

Not applicable.

## 11. Rollout

- initial state after merge: repository documentation only; no runtime,
  deployment or production effect;
- feature flag/configuration: none;
- staging/preview validation: not applicable;
- pilot: not applicable;
- activation approval: not applicable; this task activates nothing;
- observation period: none.

## 12. Rollback

- code rollback: ordinary revert of the documentation pull request;
- data rollback or forward repair: not applicable;
- feature disable/kill switch: not applicable;
- external side-effect handling: none. The transcription comment on PR #805 is
  immutable GitHub evidence and is not removed by a repository revert.

## 13. Stop conditions

The implementing agent must stop and report `BLOCKED` if:

- `origin/develop` is not the merge commit of PR #805, or PR #805 is not merged;
- a competing BE-02 closeout branch, pull request or committed record already
  exists;
- the correction would require a runtime, schema, migration, contract or
  workflow change;
- the correction would require rewriting historical evidence;
- approved architecture would need to change;
- required production information or secrets are unavailable;
- scope cannot be completed without unrelated changes.

## 14. Expected implementation report

The agent must use:

`docs/engineering/ai-delivery/implementation-report-template.md`

## 15. Recommended execution

Implementation model: Claude Opus 5
Reasoning level: Extra High / xhigh
Independent reviewer: fresh independent strong model/context
Review reasoning level: High or above

## 16. Branch and integration plan

- branch source: the exact verified `develop` head that is the merge commit of
  PR #805;
- pull-request target: `develop`;
- expected merge order: before the BE-03 specification pull request, which is
  prepared from this branch's final head;
- parent programme branch refresh requirement: none; the programme uses no
  integration branch;
- branch deletion after merge: YES
- final programme PR required: NO
- final release path: `develop -> master`

## 17. Approval

Approved for implementation by: CTO
Date: 2026-08-12
Notes: explicit CTO instruction, transcribed by the authoring agent onto merged
PR [#805](https://github.com/thoth-pub/thoth/pull/805). Authorization is limited
to documentation and control correction of materially stale active BE-02
programme state. It authorizes no runtime, schema, migration, API, generated
contract, workflow, deployment or production action.

Record only the durable implementation authorization here. Independent review
decisions, CTO merge authorization and the merge itself are terminal GitHub
evidence under ADR-0005 and must not be copied back into this file.
