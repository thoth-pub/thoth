# Publisher Services Rollout Plan

Status: PROPOSED CONTROLLED SEQUENCE
Owner: CTO
Production activation: explicit CTO approval required

## 1. Safety principles

- Add schema and APIs before changing live behaviour.
- Keep automatic job creation inactive initially.
- Audit and backfill before strict enforcement or cutover.
- Run legacy and API configuration in comparison mode.
- Fail closed.
- Pilot one bounded publisher/platform activation.
- Observe before cleanup.
- Preserve rollback until reconciliation proves stability.

## 2. Stage 0 - Control foundation

Required:

- P0-01 documents and master issue;
- ADR-0001 approved;
- ADR-0002 approved;
- repository/branch readiness recorded;
- task specifications and review assignments.

Achieved evidence:

- PR #764 merged into `develop` as
  `5b406e4ef9b5c192cc38eb8a97a41bbd0fc3bc06`;
- master issue [#765](https://github.com/thoth-pub/thoth/issues/765) exists and
  links every programme task;
- P0-01 `CLOSED`: closeout PR
  [#767](https://github.com/thoth-pub/thoth/pull/767) (reviewed content head
  `d72137893ddea512c0d05c81d310eb59d045cd2b`) was independently `APPROVED` and
  merged into `develop` as `bac598e32abbd0d7e69ff467c82945ee00df02ba` on
  2026-07-27, making the repository the authoritative P0-01 closure record.

Outstanding evidence:

- separately authorized issue #765 synchronization as an external mirror of the
  completed repository closeout (it does not approve architecture or unlock
  implementation);
- ADR-0001 and ADR-0002 approval;
- Publisher Services ADR-01 specification and final platform-inventory
  approval;
- applicable repository/branch-readiness decisions;
- approved task specifications and review assignments for implementation work;
- no unresolved control contradiction.

Rollback:

- revert documentation PR; no runtime effect.

## 3. Stage 1 - Licence and package foundations

Deliver:

- LIC-01;
- BE-01;
- LIC-02 only after production licence audit planning.

Controls:

- additive package column with OASIS default;
- package capability mapping inactive outside read paths;
- no platform assignment changes;
- no strict licence enforcement before audit.

Exit evidence:

- migration and capability matrix tests;
- supported licence inventory;
- zero unexplained current licence values.

Rollback:

- disable consuming paths;
- use forward repair for persisted package values if required;
- do not delete audit evidence.

## 4. Stage 2 - Platform and protected configuration

Deliver:

- ADR-01;
- BE-02;
- BE-03;
- BE-04 storage/API with automatic job creation disabled.

Controls:

- linked normalization in backend;
- optimistic concurrency;
- audit transaction;
- job creation feature flag/config default off;
- worker role cannot alter publisher configuration.

Exit evidence:

- authorization matrix;
- linked-state tests;
- job concurrency/lease tests;
- no live jobs created by deployment.

Rollback:

- disable configuration mutation;
- preserve additive tables;
- revert client exposure.

## 5. Stage 3 - Audit and backfill

Deliver:

- MIG-01 dry-run tool;
- approved package mapping;
- approved platform mapping;
- licence normalization plan.

Required dry-run output:

- publisher count by package;
- publisher/platform assignments;
- unmatched legacy identifiers;
- linked-state anomalies;
- unsupported licences;
- expected inserts/updates;
- expected job count, which must be zero;
- rerun result.

Production run controls:

- explicit input checksum/version;
- no-job import mode;
- transaction/batching plan;
- bounded logs;
- reconciliation queries;
- backup/restore readiness;
- CTO approval immediately before execution.

Stop conditions:

- any unexpected job;
- unexplained publisher omission;
- ambiguous platform mapping;
- unsupported licence without disposition;
- counts differ from reviewed dry run.

Rollback:

- transaction rollback where possible;
- otherwise approved reverse/forward repair using captured before-state;
- automatic jobs remain disabled.

## 6. Stage 4 - Interfaces

Deliver:

- APP-01;
- APP-02;
- APP-03.

Controls:

- package read-only for publisher users;
- superuser-only mutations/reports;
- server result replaces optimistic linked-platform state;
- generated GraphQL types pinned to merged schema;
- no frontend-owned capability or linked-platform matrix.

Exit evidence:

- authorization E2E tests;
- concurrency failure UX;
- CSV/report count parity;
- API-backed licence option parity.

Rollback:

- hide/disable routes;
- retain backend additive APIs;
- no data rollback.

## 7. Stage 5 - Dissemination comparison

Deliver DIS-01 in modes:

```text
env
compare
api
```

Initial production mode:

```text
compare
```

Comparison report must show:

- legacy publisher set;
- API publisher set;
- additions;
- omissions;
- linked-platform normalization;
- unsupported platform descriptors;
- duplicate adapter routes.

Cutover gate:

- differences resolved or explicitly approved;
- API outages tested fail closed;
- empty API assignment tested as no-op;
- no fallback to all publishers/works;
- rollback to `env` documented and exercised.

Activation:

- switch one pathway/platform at a time where practical;
- CTO approval per high-risk cutover.

## 8. Stage 6 - Back-catalogue worker pilot

Deploy DIS-02 with:

- schedule disabled or tightly bounded;
- protected environment;
- explicit worker identity;
- low concurrency;
- retry/backoff;
- metrics and alerts;
- no automatic job creation.

Pilot:

1. choose one publisher;
2. choose one verified `AutomaticPush` platform;
3. review eligible catalogue count;
4. enable one activation/job;
5. observe claim, attempts, adapter outcomes and completion;
6. reconcile external state and Thoth locations;
7. exercise retry or controlled failure;
8. record decision.

Do not pilot OAPEN/DOAB first unless linked multi-target behaviour is the explicit test objective.

Rollback:

- stop worker schedule;
- revoke/disable worker role;
- cancel pending pilot jobs;
- restore legacy path;
- reconcile partial external delivery.

## 9. Stage 7 - General enablement

Prerequisites:

- successful pilot;
- no unresolved P0/P1 findings;
- acceptable comparison history;
- alerting operational;
- support/runbook ownership assigned.

Enable:

- automatic job creation for approved destinations;
- scheduled worker;
- controlled platform-by-platform adoption.

Maintain:

- legacy comparison or fallback;
- reconciliation reports;
- bounded concurrency;
- activation audit.

## 10. Stage 8 - Downstream services

### OCLC KBART

Enable EXP-01 after:

- OCLC_KB enum approval;
- publisher backfill reconciliation;
- public output/privacy review.

### OAI-PMH

Begin OAI-01 only after:

- BE-01 merged;
- LIC-02 merged and enforced safely;
- deferred branch divergence assessed;
- fresh branch/port decision recorded;
- complete package/licence/lifecycle test matrix.

## 11. Stage 9 - Observation and cleanup

Minimum observation record includes:

- activation dates;
- comparison differences;
- failed/retried jobs;
- stale leases;
- unexpected external duplicates;
- support incidents;
- rollback events;
- publisher configuration corrections.

Cleanup requires:

- E2E-01 approval;
- stable observation period set by CTO;
- no unresolved high-severity reconciliation issues;
- explicit cleanup task and rollback assessment.

Only then remove:

- publisher-ID environment lists;
- comparison mode;
- duplicate licence lists;
- superseded scripts/configuration.

Cleanup is irreversible operational change and requires independent review plus CTO approval.
