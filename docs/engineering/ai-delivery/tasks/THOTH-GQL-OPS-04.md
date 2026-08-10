# THOTH-GQL-OPS-04 - Bounded runtime-operations verification and closure

Status: DRAFT
Implementation: NOT AUTHORIZED
Programme: Shared Thoth GraphQL / Backend Architecture
Repository: `thoth-pub/thoth`
Workflow: STANDARD
Base branch: `develop`
PR target: `develop`
Programme integration branch: None
Risk: HIGH
Owner: Shared backend architecture
Approved by: not yet approved
Dependencies, all required before implementation may begin:
[`ADR-0006`](../../decisions/ADR-0006-request-scoped-graphql-batching.md)
approved and repository-authoritative;
[`THOTH-GQL-BATCH-01`](THOTH-GQL-BATCH-01.md) merged;
[`THOTH-GQL-OPS-01`](THOTH-GQL-OPS-01.md) merged;
**[`THOTH-GQL-OPS-02`](THOTH-GQL-OPS-02.md) implemented, independently reviewed
and merged**;
**[`THOTH-GQL-OPS-03`](THOTH-GQL-OPS-03.md) implemented, independently reviewed
and merged**; this specification approved; a freshly verified exact `develop`
base; explicit CTO implementation authorization
Target branch name: `feature/shared-architecture/graphql-runtime-ops-closure`
(**must not exist** until implementation is authorized)
Production activation effect: NONE. This task decides whether a mode change
*could* be controlled and verified. It does not perform one.

Authority condition: this record is repository-authoritative when this exact
content is reachable from the repository's authoritative integration branch.
Live review, authorization and merge evidence is the GitHub pull-request record.

This specification does not authorize implementation, and it does not authorize
production activation. `OFF -> OBSERVE` and `OBSERVE -> ENFORCE` each remain
subject to their own separate explicit CTO production activation approval
(`ADR-0006` section 7.2.1), and **this task may not grant either**.

## 1. Objective

After `THOTH-GQL-OPS-02` and `THOTH-GQL-OPS-03` have merged, perform a **fresh**
bounded operational verification against the real runtime: re-establish every
external fact, confirm ownership, prove the mode-control capability actually
operates, prove the fleet-verification capability actually operates, instantiate
the live fleet predicate, resolve the outstanding runtime-operations decisions,
and only then decide — on evidence — whether the **mode-control** subset of CG-13
may become disposition **A**.

This is the **earliest** task in this family that may record:

```text
runtime-operations gate: SATISFIED
```

It may equally return **C** again. Returning C is a legitimate outcome, not a
failure of the task.

**Bounded to one gate.** This task closes the runtime-operations mode-control
gate and no other. The service-health/activation-threshold gate and the
preview/staging acceptance gate — including the **timed rollback rehearsal** and
its four measurements — are **downstream of this task** and must not be absorbed
into it (section 3.2). Satisfying this gate therefore does **not** make the
mode-transition runbook production-executable.

## 2. Background and authority

Authoritative sources:

- [`THOTH-GQL-OPS-01`](THOTH-GQL-OPS-01.md), in particular sections 2.2.0, 2.2.5,
  3.12.3, 12, 12.1 and 12.2;
- the [mutation-guard runtime-operations control record](../../repository-map/graphql-mutation-guard-runtime-operations.md),
  in particular section 14, which lists the unresolved evidence this task must
  obtain;
- the [mode-transition runbook](../../repository-map/graphql-mutation-guard-mode-transition-runbook.md),
  in particular section 10, which lists the decisions it is waiting on;
- [`THOTH-GQL-OPS-02`](THOTH-GQL-OPS-02.md) and
  [`THOTH-GQL-OPS-03`](THOTH-GQL-OPS-03.md), as merged, and their implementation
  reports;
- [`ADR-0006`](../../decisions/ADR-0006-request-scoped-graphql-batching.md)
  sections 7.2.1, 7.2.1.1, 7.2.4, 7.3, 8.3.2 and 8.3.5;
- [`release-gates.md`](../release-gates.md) sections 4, 5 and 8;
- [CG-13](../../repository-map/control-gaps.md#cg-13---thoth-runtime-operations-unmapped);
- [`environments.md`](../../repository-map/environments.md).

Current behaviour at the time this specification was written: both capability
gaps are open, the runtime-operations gate is `NOT SATISFIED`, CG-13 is `OPEN`,
the runbook is `PROVISIONAL`, and both the production and the test environments
run a **pre-guard** release — a binary with no mutation guard and therefore no
guard mode at all.

### 2.1 Every external fact must be re-established, not inherited

Deployment state is not durable repository state. Nothing recorded in the control
record about the deployed release, the container command, the fleet, the rollout
semantics, retention or ownership may be **relied on** by this task. Each must be
re-established from the authoritative source at this task's own execution time,
under the scoped-read rules restated in section 6.3.

Where a fact has changed, the change is a finding in its own right and may itself
block closure.

## 3. Explicit scope

The task must:

1. **re-establish every external fact** the control record carries, at execution
   time: the container command actually run; which release each environment
   actually runs, distinguishing pre-guard from guard-enabled; the configuration
   authority and precedence; rollout and replacement semantics; the autoscaling
   model; log retention; and access/ownership;
2. **confirm ownership** as roles: the execution owner, the request/approval
   authority, and — by explicit CTO confirmation rather than derivation — the
   post-activation observation sign-off owner;
3. **obtain the unresolved decisions** listed in control record section 14 and
   runbook section 10, including whether operational rollback requires CTO
   approval or may be executed on the technical team's own authority, and which
   observation-evidence retention remedy is adopted;
4. **prove the mode-control path operates** against the real runtime: that a
   guard-enabled release deployed through the production-applicable path actually
   consumes the configured value, in a **non-production** environment;
5. **prove fleet verification operates**: that the `THOTH-GQL-OPS-03` mechanism
   enumerates the actual serving population, attributes an effective mode to each
   member, and detects a mixed fleet, against a real deployed fleet in a
   **non-production** environment;
6. **read the live expected replica population** from orchestrator state and
   record the predicate as instantiated rather than as defined, and record any
   configuration drift found between the authoritative deployment source and the
   live orchestrator;
7. **resolve the runtime-operations `PROVISIONAL` state of the runbook** — see
   section 3.2 — recording the resolved decisions and the proven capabilities,
   and leaving every downstream-owned field explicitly outstanding;
8. **decide the CG-13 mode-control subset disposition** — `A` or `C` — on
   evidence, and record it;
9. **record the runtime-operations gate state** consistently in the control
   record, the runbook, `control-gaps.md`, the decision register, the
   implementation report and the pull-request body;
10. add the changelog entry and the implementation report.

### 3.1 A guard-enabled non-production environment is a prerequisite

The test environment is **pre-guard**, so there is no mode there to change.
Deploying a **guard-enabled candidate to a non-production environment** is
therefore a prerequisite of items 4 and 5, and is itself a deployment action
requiring its own authorization. It is a **non-production** deployment only;
nothing in this task authorizes a production deployment.

### 3.2 What this task may resolve, and what it must leave downstream

**Binding.** This task closes **one** gate — the runtime-operations mode-control
gate — and no other. The repository-authoritative sequence is:

```text
THOTH-GQL-OPS-02
    -> THOTH-GQL-OPS-03
    -> THOTH-GQL-OPS-04
       -> runtime-operations gate may become SATISFIED on evidence
    -> service-health signals and activation thresholds verified
    -> preview/staging acceptance of the exact candidate, including
       performance evidence and the timed rollback rehearsal
    -> explicit CTO OFF -> OBSERVE authorization
```

`THOTH-GQL-OPS-04` sits at the third step. The two steps after it are **separate
downstream gates that this task must not absorb**.

| Item | Owner |
|---|---|
| mode-control capability proven against a non-production runtime | **this task** |
| fleet-verification capability proven against a non-production runtime | **this task** |
| live fleet predicate instantiated | **this task** |
| ownership, rollback-approval authority and other outstanding decisions | **this task** |
| runtime-operations `PROVISIONAL` state resolved | **this task** |
| service-health signals and activation thresholds derived and approved | **downstream** — `ADR-0006` section 8.3.2 gate |
| preview/staging acceptance of the exact candidate | **downstream** — preview/staging gate |
| the **timed rollback rehearsal** and its four measurements | **downstream** — preview/staging gate |
| `OFF -> OBSERVE` authorization | **downstream** — explicit CTO decision |

Consequently this task **does not** execute the timed rehearsal, **does not** own
the four rehearsal measurements
(`time to apply mode change`, `time to verify fleet consistency`,
`time to rollback`, `time to verify rollback`), **does not** establish the
preview/staging acceptance gate, and **does not** derive or approve any
service-health threshold.

Proving that a capability **operates** is not the same as measuring how **long**
it takes. This task proves operation. The downstream rehearsal measures timing.

### 3.3 The two-part status this task must record

Resolving the runtime-operations `PROVISIONAL` state does **not** make the
runbook executable in production. The task must record both parts, and must not
collapse them:

```text
runtime-operations procedure established

production transition still BLOCKED by:
  - service-health/threshold gate
  - preview/staging rehearsal
  - explicit CTO activation
```

Timing fields in the runbook remain marked
`TO BE MEASURED AT PREVIEW/STAGING GATE`. Populating them is **not** a condition
of resolving the runtime-operations-specific `PROVISIONAL` state, and this task
must not treat it as one.

## 4. Non-goals

The task must not:

1. activate `OBSERVE` in production;
2. activate `ENFORCE` in production;
3. change the guard mode in **production**, in any way, for any duration;
4. grant, imply or substitute for the explicit CTO production activation approval
   that `OFF -> OBSERVE` requires;
5. deploy to production;
6. change the production container command, or specify doing so — see section
   13.1;
7. change, reorder, conditionalise or remove migration execution;
8. change mutation-guard, batching or store semantics;
9. change `ADR-0006` architecture;
10. invent, derive or approve service-health thresholds, latency or error-rate
    baselines, or availability SLOs — that remains the separate `ADR-0006`
    section 8.3.2 gate, which this task must **not** absorb;
11. **execute the timed rollback rehearsal, or own its four measurements** — that
    belongs to the downstream preview/staging gate (section 3.2);
12. **establish the preview/staging acceptance gate** — likewise downstream;
13. treat the population of the runbook's timing fields as a condition of
    resolving the runtime-operations-specific `PROVISIONAL` state (section 3.3);
14. close **all** of CG-13. Migration execution, backup and restore verification,
    and approver mapping for concerns other than this feature remain open
    regardless of this task's outcome;
15. return CG-13 disposition **B**;
16. return disposition **A** while either capability gap remains open, or on any
    basis other than evidence obtained at this task's own execution time;
17. write any production configuration value, secret or resource identifier into
    this repository;
18. remediate or rotate any credential, or create or modify a security issue;
19. make any change to the private authoritative deployment source beyond a
    separately authorized **non-production** deployment required by section 3.1;
20. modify `BE-02`, PR [#788](https://github.com/thoth-pub/thoth/pull/788) or
    issue [#765](https://github.com/thoth-pub/thoth/issues/765);
21. implement `BE-02`.

## 5. Invariants

The implementation must preserve:

1. **no production activation and no production deployment** is performed;
   production request acceptance is unchanged;
2. the production guard mode is not changed, and no production environment is
   transitioned;
3. an environment running a **pre-guard** release is recorded as pre-guard and is
   never described as `MutationGuardMode::OFF`;
4. the loader store remains unavailable wherever the guard is not `ENFORCE`;
5. no runtime, schema, migration, `Cargo` or workflow file is changed by this
   task — it is documentation and control work operating on already-merged
   capability;
6. merge authorization and the two activation authorizations remain three
   distinct decisions, and this task grants none of them;
7. CG-13 as a whole remains open, whatever the feature-subset disposition;
8. `BE-02` remains unauthorized;
9. no secret or production configuration value enters the repository;
10. no operational claim is recorded without a named evidence source and an
    evidence class;
11. every external fact is **re-established** at execution time and none is
    inherited from a predecessor record;
12. disposition `A` is recorded only if **both** capability gaps are closed by
    merged work **and** the runtime evidence obtained here supports it.

## 6. Required behaviour

### 6.1 Success behaviour

The task succeeds when it has:

- re-established every external fact, with evidence source and class;
- confirmed ownership, including explicit CTO confirmation of the observation
  sign-off owner;
- obtained every unresolved decision listed in control record section 14 and
  runbook section 10;
- proven, against a real non-production runtime, that the mode-control path
  operates and that fleet verification operates — **operation, not timing**;
- instantiated the live fleet predicate;
- resolved, or deliberately **not** resolved, the runbook's
  runtime-operations-specific `PROVISIONAL` state, consistently with the
  evidence, while leaving every downstream-owned field explicitly outstanding;
- recorded the two-part status of section 3.3 without collapsing it;
- recorded an evidenced disposition and an internally consistent gate state.

**Success is not disposition A.** Reaching `C` again on honest evidence is a
successful execution of this task.

**Success is also not a production-ready runbook.** Even a fully successful
execution leaves the production transition blocked by the service-health/threshold
gate, the preview/staging timed rehearsal and explicit CTO activation.

### 6.2 Failure behaviour

Where evidence is unavailable, the task records the **exact** missing evidence
and returns `BLOCKED` for the affected criterion. It does not substitute a
plausible mechanism and does not soften an unanswerable question into a
narrative. Missing evidence is missing work.

Where the delivered capability proves insufficient in practice — for example the
verifier cannot achieve complete coverage of a real fleet, or the mode-control
path works in isolation but not through the real deployment path — the task
records disposition **C** again, records precisely what is still missing, and
specifies the bounded successor that would close it.

### 6.3 Authorization

Inspection of the private authoritative deployment source is **read-only**,
limited to ownership, mechanism and configuration **metadata**, and bound by the
scoped-read rules of `THOTH-GQL-OPS-01` section 2.2.5, restated here as binding:

1. use **narrowly scoped searches or line/range reads** targeted at the specific
   criterion — never a whole-file read of a secret-bearing configuration file,
   and never a broad recursive dump;
2. retrieve **only the metadata the criterion requires** and stop there;
3. **never copy secret-bearing ranges** into a report, a specification, a
   changelog, a pull request, a commit message, a prompt or any other output;
4. treat the source as strictly **read-only**, except for the separately
   authorized non-production deployment of section 3.1, and use no credential
   found there;
5. stop and report `BLOCKED` if a criterion's evidence cannot be obtained without
   exposing secret material.

Incidental encounter with secret material during an otherwise scoped read is not
a breach and must be **escalated** rather than quietly absorbed; it becomes a
breach only if the material is copied onward. Remediating the credential exposure
remains **outside this task's scope** and is a separate CTO-controlled security
matter.

Live orchestrator reads required by section 3 item 6 are **read-only** and must
not extend to production databases or to any mutating operation.

### 6.4 Concurrency and idempotency

Not applicable to the repository output. The non-production capability
verification of section 3 items 4 and 5 must itself be repeatable without leaving
that environment in a changed mode: it ends by restoring the starting mode and
verifying the restoration.

### 6.5 Compatibility

No API, schema, database, client or deployment-contract change is made by this
task. The public GraphQL schema is untouched and the generated SDL is unchanged.

## 7. Data and migration requirements

Migration required: NO

```text
Database/data change:                        NO
GraphQL schema change:                       NO
Public API change:                           NO
Production mode change:                      NO
Production deployment:                       NO
Non-production deployment:                   YES -- separately authorized,
                                             required by section 3.1
```

Any contrary discovery is a stop and escalation condition (section 13).

## 8. Observability and operations

Required logs: none added by this task.

Required metrics/alerts: none added by this task. Service-health signals and
activation thresholds remain the separate `ADR-0006` section 8.3.2 gate and must
not be absorbed here, even though this task is the natural place to be tempted.

Operational runbook changes: this task resolves the **runtime-operations-specific**
`PROVISIONAL` state of the
[mode-transition runbook](../../repository-map/graphql-mutation-guard-mode-transition-runbook.md)
if and only if the evidence supports doing so — recording the resolved ownership,
the resolved rollback-approval authority, the confirmed sign-off owner, the
instantiated fleet predicate and the proven capabilities. If the evidence does
not support it, that state **remains unresolved** and the task records why.

It does **not** populate the runbook's timing fields. Those remain marked
`TO BE MEASURED AT PREVIEW/STAGING GATE` and are owned by the downstream
rehearsal. The runbook remains structurally complete but **not** production-executable
until the downstream gates close (section 3.3).

## 9. Acceptance criteria

- [ ] **AC-1** Every external fact in the control record is re-established at this
      task's execution time, with evidence source and class, and any change from
      the previously recorded state is reported as a finding.
- [ ] **AC-2** The container command actually run by the production GraphQL API
      service is re-confirmed.
- [ ] **AC-3** Which release each environment actually runs is re-confirmed, with
      pre-guard environments recorded as **pre-guard** and never as
      `MutationGuardMode::OFF`.
- [ ] **AC-4** The execution owner and the request/approval authority are
      re-confirmed as roles.
- [ ] **AC-5** The post-activation observation sign-off owner is confirmed by
      **explicit CTO confirmation**, not by derivation.
- [ ] **AC-6** Rollback authority is resolved by explicit CTO decision, and any
      difference from forward-change authority is stated.
- [ ] **AC-7** The observation-evidence retention remedy is decided and confirmed
      to be in place.
- [ ] **AC-8** The mode-control path is **proven to operate** against a real
      non-production runtime: a guard-enabled release deployed through the
      production-applicable path consumes the configured value.
- [ ] **AC-9** Fleet verification is **proven to operate** against a real
      non-production fleet: the serving population is enumerated, each member is
      attributed an effective mode, and a mixed fleet is detected.
- [ ] **AC-10** A partial-fleet state is deliberately created in the
      non-production environment and shown to be **detected** by the
      `THOTH-GQL-OPS-03` mechanism. This proves detection **operates**; it does
      not measure how long a mixed window lasts, which is downstream.
- [ ] **AC-11** The silent-adoption failure class is deliberately exercised and
      shown to be detected.
- [ ] **AC-12** Store unavailability outside `ENFORCE` is confirmed
      **operationally**, not assumed.
- [ ] **AC-13** The live expected replica population is read from orchestrator
      state, and the fleet predicate is recorded as instantiated. Any drift
      between the authoritative deployment source and the live orchestrator is
      recorded.
- [ ] **AC-14** No numeric threshold and no duration is invented. Every timing
      field in the runbook remains marked
      `TO BE MEASURED AT PREVIEW/STAGING GATE`, and thresholds remain the
      separate `ADR-0006` section 8.3.2 gate.
- [ ] **AC-15** The timed rollback rehearsal is **not** executed by this task and
      its four measurements are **not** claimed by it; the preview/staging
      acceptance gate is **not** established by it. Both remain downstream
      (section 3.2).
- [ ] **AC-16** The runbook's **runtime-operations-specific** `PROVISIONAL` state
      is resolved **or** deliberately left unresolved, consistently with the
      evidence, and the choice is justified. Resolving it is **not** conditioned
      on populating the timing fields.
- [ ] **AC-17** The two-part status of section 3.3 is recorded without being
      collapsed: the runtime-operations procedure may be established while the
      production transition remains blocked by the service-health/threshold gate,
      the preview/staging rehearsal and explicit CTO activation.
- [ ] **AC-18** The CG-13 feature-subset disposition is recorded as `A` or `C`,
      on evidence, and `B` is not claimed.
- [ ] **AC-19** The runtime-operations gate state is recorded **consistently** in
      the control record, the runbook, `control-gaps.md`, the decision register,
      the implementation report and the pull-request body.
- [ ] **AC-20** CG-13 as a whole remains **open**, whatever the feature-subset
      disposition.
- [ ] **AC-21** No production deployment, no production mode change and no
      production activation occurred, and the report says so explicitly.
- [ ] **AC-22** `OBSERVE`, `ENFORCE` and `BE-02` remain recorded as
      `NOT AUTHORIZED`; PR #788 and issue #765 are unchanged.
- [ ] **AC-23** No production configuration value, secret or resource identifier
      appears anywhere in the diff.
- [ ] **AC-24** Every read of the private authoritative source complied with the
      section 6.3 scoped-read rules, and any incidental encounter with secret
      material was escalated rather than copied onward.
- [ ] **AC-25** No runtime, schema, migration, `Cargo` or workflow file appears
      in the diff.

## 10. Required tests

### Unit / Integration/database / Authorization/security / Regression

Not applicable in the ordinary sense — this task changes no code. The verification
work is operational and its evidence is recorded under manual verification below.
The task must, however, confirm that the merged `THOTH-GQL-OPS-02` and
`THOTH-GQL-OPS-03` test suites still pass at its own exact base.

### Documentation validation

```bash
git diff --check
```

Also verify: every relative Markdown link resolves; no runtime, schema,
migration, `Cargo` or workflow path appears in the diff; CG-13 is not marked
globally resolved; no statement describes a pre-guard release, image or
environment as having a guard mode; `OBSERVE`, `ENFORCE` and `BE-02` remain
recorded as `NOT AUTHORIZED`; a `CHANGELOG.md` entry exists under
`## [Unreleased]`; and the gate state is identical in every place it appears.

### Manual verification

- re-establish every external fact under the section 6.3 scoped-read rules;
- deploy a guard-enabled candidate to the **non-production** environment under
  separate authorization;
- exercise each of `OFF`, `OBSERVE` and `ENFORCE` there through the
  production-applicable command path, and confirm the effective mode with the
  `THOTH-GQL-OPS-03` mechanism;
- enumerate the serving population from live orchestrator state and attribute an
  effective mode to every member;
- deliberately create and detect a mixed fleet;
- deliberately exercise and detect the silent-adoption failure class;
- confirm store unavailability outside `ENFORCE` operationally;
- exercise a rollback in the non-production environment far enough to prove the
  rollback path **operates** and to restore the starting mode, verifying the
  restoration. Do **not** time it, and do **not** record the four rehearsal
  measurements: proving operation is this task's scope, and measuring timing is
  the downstream preview/staging rehearsal's (section 3.2);
- confirm that **no production environment was touched**.

### Performance

Not applicable to this task. The guard's request-path performance evidence is
`ADR-0006` section 7.2.3 work belonging to the preview/staging gate.

## 11. Rollout

- **initial state after merge:** unchanged in production. Documentation and
  control only.

  ```text
  production release deployed       = unchanged by this task
  production guard mode             = unchanged by this task
  environments transitioned         = none in production
  production request acceptance     = unchanged
  ```

- **feature flag/configuration:** none introduced;
- **repository-managed deployment configuration:** this repository holds none;
- **staging/preview validation:** **not** performed by this task. This task
  performs bounded non-production **capability verification** — proving that the
  mode-control and fleet-verification capabilities operate. The preview/staging
  **acceptance gate**, including the performance evidence of `ADR-0006` section
  7.2.3 and the timed rollback rehearsal, is a separate downstream gate
  (section 3.2);
- **pilot:** not applicable. `OBSERVE` is itself the controlled pilot and is not
  authorized by this task;
- **activation approval:** unchanged and still required. Even if this task
  records the runtime-operations gate as `SATISFIED`, `OFF -> OBSERVE` still
  requires its own explicit CTO production activation approval, and the remaining
  `ADR-0006` gates — service-health signals and activation thresholds, and
  preview/staging acceptance including the timed rehearsal — still apply in full;
- **observation period:** not applicable to this task.

## 12. Rollback

- **code rollback:** revert the merge commit. The task changes no runtime code,
  so a revert is a no-op for production behaviour; it reverts a control record;
- **data rollback or forward repair:** none;
- **feature disable/kill switch:** not applicable. The task activates nothing;
- **external side-effect handling:** the non-production capability verification
  restores the starting mode and verifies the restoration before the task
  completes. If it cannot, that is a stop condition and the non-production
  environment's state is reported explicitly.

## 13. Stop conditions

The implementing agent must stop and report `BLOCKED` if:

- either `THOTH-GQL-OPS-02` or `THOTH-GQL-OPS-03` is not merged at this task's
  exact base;
- production runtime ownership or the observation sign-off owner cannot be
  confirmed;
- configuration authority cannot be re-established;
- deployed-state evidence cannot be safely obtained under the section 6.3 rules;
- required metadata cannot be retrieved without exposing secret material;
- the mode-control path does not actually operate against the real runtime;
- the fleet-verification mechanism cannot achieve complete coverage of a real
  fleet;
- a partial-fleet state cannot be detected;
- the non-production capability verification cannot be completed, or cannot
  restore the starting mode;
- the actual runtime contradicts the approved `OFF`/`OBSERVE`/`ENFORCE`
  lifecycle;
- an architecture change is required;
- a change to migration behaviour is required;
- a change to another repository is required beyond the separately authorized
  non-production deployment;
- runtime implementation is required — that is a new bounded task, not this one;
- a **production** action would be necessary to answer an acceptance criterion.

A stop condition does not authorize scope expansion, and it does not authorize
returning disposition `A` on partial evidence.

### 13.1 The production container-command override: binding classification

Reproduced wherever an override is mentioned at all:

```text
An explicit production command override is NOT an interchangeable
feature-local fix. It changes the current `init` execution path by removing
migration execution from deployment, and therefore requires separate
migration/deployment-control analysis and approval under the broader CG-13
migration/deployment problem.
```

It is out of bounded scope for this task, must not be offered as an option,
fallback or expedient, and if evidence shows only an override can work, that is
an escalation to the CTO under the migration/deployment half of CG-13.

## 14. Expected implementation report

The agent must use
[`implementation-report-template.md`](../implementation-report-template.md) and
must record: the exact base and head; every re-established external fact with its
evidence source and class, and every change from the previously recorded state;
the confirmed ownership, including the CTO's explicit confirmation of the
observation sign-off owner; every resolved decision from control record section
14 and runbook section 10; the evidence that the mode-control and
fleet-verification capabilities **operate**, including the partial-fleet
detection and silent-adoption detection results; the instantiated fleet predicate
and any drift found; the CG-13 mode-control subset disposition with its
justification; the runtime-operations gate state, stated identically everywhere
it appears; whether the runbook's runtime-operations-specific `PROVISIONAL` state
was resolved and why; the two-part status of section 3.3, stated without being
collapsed; an explicit statement that no production deployment, production mode
change or production activation occurred; a **method statement** for every read of
the private authoritative source confirming compliance with the section 6.3 rules
and reporting any incidental encounter with secret material as an escalation; and
CI status with the classification of each job.

The report must **not** claim the four rehearsal measurements, must **not** claim
the preview/staging acceptance gate, and must **not** state or imply that the
runbook is production-executable. Those remain downstream (section 3.2), and the
report must say so explicitly.

If the disposition is `C`, the report must specify the bounded successor task
that would close what remains.

## 15. Recommended execution

Implementation model: Claude Opus, or the strongest available engineering model
Reasoning level: HIGH / maximum practical
Independent reviewer: an independent model family that did not author the
implementation
Review reasoning level: HIGH

## 16. Branch and integration plan

- branch source: a freshly verified exact `develop` head, at which both
  `THOTH-GQL-OPS-02` and `THOTH-GQL-OPS-03` are merged;
- pull-request target: `develop`;
- expected merge order: strictly after both `THOTH-GQL-OPS-02` and
  `THOTH-GQL-OPS-03`, and before the service-health/threshold gate;
- parent programme branch refresh requirement: not applicable — STANDARD
  workflow, no programme integration branch;
- branch deletion after merge: YES;
- final programme PR required: NO;
- final release path: `develop -> master`.

## 17. Approval

Approved for implementation by:
Date:
Notes:

Record only the durable implementation authorization here. Independent review
decisions, CTO merge authorization and the merge itself are terminal GitHub
evidence under [`ADR-0005`](../../decisions/ADR-0005-terminal-merge-evidence.md)
and must not be copied back into this file.
