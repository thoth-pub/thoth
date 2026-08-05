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
  2026-07-27, making the repository the authoritative P0-01 closure record;
- issue #765 synchronized on 2026-07-27 as an external mirror of the completed
  repository closeout; issue #765 remains open;
- `ADR-0002` platform domain boundaries approved by the CTO on 2026-07-27
  (approval PR [#769](https://github.com/thoth-pub/thoth/pull/769)); this removes
  one shared-ADR dependency and does not unlock implementation;
- `ADR-0001` publisher package capabilities approved by Javi, CTO, on
  2026-07-28 through approval PR
  [#772](https://github.com/thoth-pub/thoth/pull/772); this removes the shared
  decision dependency and does not unlock Publisher Services implementation.

- the bounded
  [`ADR-01` implementation specification](../engineering/ai-delivery/tasks/ADR-01.md)
  has been drafted and is proposed, defining the read-only evidence
  scope, the required per-destination record, the evidence classification, the
  decisions ADR-01 must produce and the exact stop labels that fire when
  evidence is missing. It resolves no platform question, finalizes no
  inventory, and authorizes no ADR-01 implementation. It becomes
  repository-authoritative only after independent specification review,
  explicit CTO approval of the written specification, and merge — a future
  transition, not the current state.

Outstanding evidence:

- independent review and explicit CTO approval of the written ADR-01
  specification, followed by its merge;
- Publisher Services ADR-01 implementation and final platform-inventory
  approval; the inventory in
  [`platform-inventory.md`](platform-inventory.md) remains explicitly
  provisional until an approved ADR-01 merges;
- applicable repository/branch-readiness decisions;
- approved task specifications and review assignments for implementation work;
- no unresolved control contradiction.

Rollback:

- revert documentation PR; no runtime effect.

### 2.1 Coordinated task sequence

The programme uses one fresh branch and one PR per task. There is no Publisher
Services programme integration branch, and backend and app remain separate
repositories with separate branches and PRs.

Dependency graph — the programme's hard dependencies form a directed acyclic
graph, not a single serial chain:

```text
ADR-01-SPEC -> ADR-01 -> BE-02

BE-01 ----+
          +-> BE-03 -> APP-01
BE-02 ----+

BE-03 ------------+
BR-APP-01 --------+
CG-11 closure ----+-> APP-01 implementation
APP-01 spec ------+
```

Explicitly: `ADR-01-SPEC` and `ADR-01` do not depend on `BE-01`; `BE-02`
depends on an approved and merged `ADR-01` implementation, not the ADR-01
specification alone; `BE-03` depends on both `BE-01` and `BE-02`; `APP-01`
implementation depends on `BE-03`, app branch readiness (`BR-APP-01` or an
explicit CTO exception), CG-11 closure, and its own approved specification. A
preferred delivery order may sequence independent tasks for coordination
convenience, but a preferred order is not a hard dependency.

Parallel `thoth-app` readiness track:

```text
BR-APP-01 branch-topology normalization
a separately specified CG-11 CI closure task
APP-01 specification
APP-01 implementation
```

No authoritative task ID exists in current repository records for the CG-11 CI
closure task; it is referred to by description until that task is specified and
its ID is recorded.

Controls:

- `thoth-app` must not begin `APP-01` implementation until `BE-03` exposes the
  approved protected API.
- `BR-APP-01` is a separate HIGH-risk task because it changes Vercel production
  and preview routing.
- `APP-01` must use the verified app development branch after normalization, or
  an explicit CTO exception.
- Cross-repository compatibility is bound through exact commit SHAs and an exact
  GraphQL schema contract, never through a moving branch name.

### 2.2 Reserved BE-03/APP-01 GraphQL contract control

Reserved and documented, not implemented. It binds the later `BE-03` and
`APP-01` tasks.

1. `BE-03` produces an exact generated GraphQL SDL at its reviewed head.
2. `APP-01` records the exact `BE-03` commit SHA.
3. `APP-01` code generation consumes a schema artifact pinned to that SHA, or a
   preview API proven to expose that exact schema.
4. `APP-01` must not generate against an unpinned moving test API and claim
   exact compatibility.
5. The app pull request records the backend PR, the backend SHA, the schema
   artifact or preview identity, the generated-code diff and the
   compatibility-test result.
6. Backend contract availability precedes app merge.
7. Backend additions remain backwards-compatible, so the existing app continues
   to function unchanged.
8. App rollback must not require removing the additive backend foundation.

Changing the app's code-generation schema source requires its own approved task
specification.

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

- the approved ADR-01 implementation merges before BE-02 finalizes
  `DistributionPlatform`; BE-02 must not derive the enum from the provisional
  inventory;
- every enum value has a code-owned descriptor, and no `OTHER` or fallback value
  exists;
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
- generated GraphQL types pinned to the exact reviewed backend commit SHA under
  the reserved contract control in section 2.2, never generated from an unpinned
  moving test API;
- app readiness controls satisfied: BR-APP-01 or an explicit CTO exception, and
  the separately specified CG-11 CI closure task;
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
