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

**Execution actor, binding (section 2.2).** This specification authorizes the
implementing agent to prepare repository changes, run local and disposable tests,
evaluate supplied evidence and record outcomes. It does **not** authorize the
implementing agent to deploy, to dispatch a deployment workflow, or to perform a
mode transition in any real environment — including a non-production one. Every
such action belongs to an authorized human operator or other independently
controlled deployment actor. Approval of this specification, and any authorization
recorded in section 17, transfer no operational action to the implementing agent.

## 1. Objective

After `THOTH-GQL-OPS-02` and `THOTH-GQL-OPS-03` have merged, complete a **fresh**
bounded operational verification against the real runtime: re-establish every
external fact, confirm ownership, establish that the mode-control capability
actually operates, establish that the fleet-verification capability actually
operates, instantiate the live fleet predicate, obtain the outstanding part-1
runtime-operations decisions, and only then decide — on evidence — whether the
**mode-control** subset of CG-13 may become disposition **A**.

**The implementing agent does not perform the operational actions.** Every
deployment, mode transition, mixed-fleet creation and rollback required below is
performed by an **authorized deployment actor** who is not the implementing agent
(section 2.2). The implementing agent prepares repository changes, runs local and
disposable tests, evaluates the evidence that actor supplies, and records the
outcome. The evidence requirement is unchanged; only the actor performing the
operational action is fixed.

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
  in particular section 14, which lists unresolved evidence and identifies which
  items belong to this task versus downstream gates;
- the [mode-transition runbook](../../repository-map/graphql-mutation-guard-mode-transition-runbook.md),
  in particular section 10, which lists the decisions it is waiting on and labels
  each with the part of its section 0.2 status that owns it;
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
re-established at this task's own execution time, through the **evidence
boundary** of section 6.3 — which the implementing agent may not bypass by
reading secret-bearing production configuration itself.

Where a fact has changed, the change is a finding in its own right and may itself
block closure. Where a fact cannot be re-established through the evidence
boundary, the affected criterion is `BLOCKED` (section 6.2). `BLOCKED` is the
correct outcome; widening the implementing agent's access is not.

### 2.2 Execution actor model — binding

Two distinct actors execute this task. The separation is a **project control**,
not a convenience, and no approval recorded in this specification, in its
section 17, or in any successor prompt may transfer an action from the second
actor to the first.

**The IMPLEMENTING AGENT** — the model executing this task — **may**:

1. prepare repository changes: specifications, control records, runbook, report,
   changelog;
2. run local, disposable and CI tests, including the merged `THOTH-GQL-OPS-02`
   and `THOTH-GQL-OPS-03` suites at its own exact base;
3. inspect **allowed sanitized or read-only evidence** within the section 6.3
   boundary;
4. evaluate evidence supplied by the authorized deployment actor, and challenge
   it as insufficient;
5. update the control record, the runbook and the implementation report;
6. commit, push and open a **draft** pull request.

**The IMPLEMENTING AGENT must NOT**:

```text
- trigger a deployment, in any environment, production or not;
- dispatch, re-run or otherwise start a deployment workflow;
- perform a mode transition itself in ANY real environment, including a
  non-production one;
- create, modify or restore a real fleet state;
- execute a rollback in a real environment;
- use, request or hold deployment credentials;
- access production secret-bearing configuration (section 6.3).
```

**An AUTHORIZED HUMAN OPERATOR, or another independently controlled deployment
actor** — never the implementing agent — performs, under the relevant separate
authorization:

```text
- any required NON-PRODUCTION deployment of a guard-enabled candidate;
- the OFF / OBSERVE / ENFORCE mode changes in that environment;
- the deliberate mixed-fleet and silent-adoption states;
- the rollback and the restoration of the starting mode;
- the live orchestrator reads,
```

and then **supplies sanitized, non-secret evidence and results** to the
implementing agent.

`THOTH-GQL-OPS-04` then **evaluates those results** and decides `A` or `C`.

**If the authorized deployment actor is unavailable, or the evidence they supply
is inadequate, the affected criterion is `BLOCKED` and the task returns
disposition `C`.** Absence of the actor never converts into agent authority, and
never into a weaker evidence requirement.

## 3. Explicit scope

Each item below states what the task must **establish**. Where an item requires a
real operational action, that action is performed by the authorized deployment
actor of section 2.2 and the implementing agent evaluates the resulting evidence.

The task must:

1. **re-establish every external fact** the control record carries, at execution
   time, **through the section 6.3 evidence boundary**: the container command
   actually run; which release each environment actually runs, distinguishing
   pre-guard from guard-enabled; the configuration authority and precedence;
   rollout and replacement semantics; the autoscaling model; log retention; and
   access/ownership;
2. **confirm ownership** as roles: the execution owner, the request/approval
   authority, and — by explicit CTO confirmation rather than derivation — the
   post-activation observation sign-off owner;
3. **obtain the part-1 unresolved decisions** listed in control record section 14
   and runbook section 10 — including whether operational rollback additionally
   requires CTO approval or may be executed on the execution-capability team's
   own authority. It must **not** attempt the part-2 items, which are downstream.
   In particular, it **re-establishes the observation-evidence retention
   position** per section 3.4 — recording the requirement and the unresolved
   dependency — and does **not** select or implement a retention remedy;
4. **establish that the mode-control path operates** against the real runtime:
   that a guard-enabled release deployed through the production-applicable path
   actually consumes the configured value, in a **non-production** environment.
   The deployment and the mode changes are performed by the authorized deployment
   actor (section 2.2); the implementing agent evaluates the evidence returned;
5. **establish that fleet verification operates**: that the `THOTH-GQL-OPS-03`
   mechanism enumerates the actual serving population, attributes an effective
   mode to each member, and detects a mixed fleet, against a real deployed fleet
   in a **non-production** environment. The fleet states are created by the
   authorized deployment actor; the implementing agent evaluates the evidence;
6. **obtain the live expected replica population** from orchestrator state — read
   by the authorized deployment actor and supplied as sanitized evidence — and
   record the predicate as instantiated rather than as defined, and record any
   configuration drift reported between the authoritative deployment source and
   the live orchestrator;
7. **resolve the runtime-operations `PROVISIONAL` state of the runbook** — see
   section 3.2 — recording the resolved decisions and the proven capabilities,
   and leaving every downstream-owned field explicitly outstanding;
8. **decide the CG-13 mode-control subset disposition** — `A` or `C` — on
   evidence, and record it;
9. **record the runtime-operations gate state** consistently in the control
   record, the runbook, `control-gaps.md`, the decision register, the
   implementation report and the pull-request body;
10. add the changelog entry and the implementation report.

### 3.1 A guard-enabled non-production environment is a prerequisite — and the implementing agent does not create it

The test environment is **pre-guard**, so there is no mode there to change.
A **guard-enabled candidate deployed to a non-production environment** is
therefore a prerequisite of items 4 and 5. It is a **non-production** deployment
only; nothing in this task authorizes a production deployment.

**Actor, binding.** That deployment is performed by the **authorized deployment
actor** of section 2.2 — a human operator or another independently controlled
deployment actor — under its own separate authorization. The implementing agent
**must not** perform it, must not dispatch a workflow that performs it, and must
not hold or use the credentials that would perform it. "Separately authorized"
authorizes the *action*; it does not make the implementing agent the *actor*, and
no reading of this specification may make it one.

The implementing agent's role for this prerequisite is limited to:

1. stating precisely which candidate, environment and configuration the
   verification requires, so the deployment actor can act unambiguously;
2. stating precisely which observations must be returned, and in what sanitized
   non-secret form;
3. evaluating what is returned, and rejecting it as insufficient if it is.

If no authorized deployment actor performs the prerequisite, or the returned
evidence is inadequate, items 4 and 5 are `BLOCKED`, the dependent acceptance
criteria fail, and the disposition is `C`. The task does **not** substitute a
local, simulated or agent-performed deployment for the missing evidence, and does
**not** relax what the evidence must show.

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

The column above records which **gate owns** an item, not which actor performs the
operational step. Ownership by "this task" never means the implementing agent
performs a deployment or a mode transition: those remain with the authorized
deployment actor of section 2.2 in every row.

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

### 3.4 Observation-evidence retention — requirement here, remedy downstream

The retention question has a **dependency order** that this task must respect
rather than short-circuit. A retention remedy cannot be chosen before the
duration it must cover has been approved, and that duration is a **part 2**
decision belonging to the activation gate.

**This task (part 1) establishes only:**

```text
- observation evidence must be retained for the complete approved
  observation window and remain available through review/sign-off;
- current runtime log retention is a FINITE configured duration,
  re-established at this task's own execution time;
- the approved OBSERVE observation-window duration is NOT yet established;
- whether current retention is sufficient is therefore NOT yet
  established;
- consequently the actual retention remedy remains DOWNSTREAM.
```

**This task must NOT:**

```text
- choose between extended retention and out-of-band capture;
- prove that current retention covers a duration that has not been
  approved;
- implement any retention change;
- confirm that a specific retention remedy is in place.
```

**The downstream gate (part 2) then:**

```text
- approves the observation-window duration;
- determines whether current retention covers it;
- if not, selects and implements a remedy;
- verifies the final retention arrangement before production activation.
```

Recording the requirement and the unresolved dependency **is** the complete part-1
obligation. Leaving the remedy unselected is the correct outcome, not an omission,
and must not be treated as blocking the runtime-operations gate.

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
14. **select, implement or confirm an observation-evidence retention remedy**, or
    approve the observation-window duration it would depend on — both are
    downstream (section 3.4);
15. close **all** of CG-13. Migration execution, backup and restore verification,
    and approver mapping for concerns other than this feature remain open
    regardless of this task's outcome;
16. return CG-13 disposition **B**;
17. return disposition **A** while either capability gap remains open, or on any
    basis other than evidence obtained at this task's own execution time;
18. write any production configuration value, secret or resource identifier into
    this repository;
19. remediate or rotate any credential, or create or modify a security issue;
20. make any change to the private authoritative deployment source. The
    **non-production** deployment required by section 3.1 is performed by the
    authorized deployment actor under its own authorization, and is not a change
    this task's implementing agent makes;
21. **have the implementing agent trigger a deployment, dispatch a deployment
    workflow, perform a mode transition in any real environment, create or
    restore a real fleet state, execute a rollback in a real environment, or use
    deployment credentials** — in production or in any non-production
    environment (section 2.2);
22. **have the implementing agent read secret-bearing production configuration
    directly**, by any route, including one it believes to be narrowly scoped
    (section 6.3);
23. modify `BE-02`, PR [#788](https://github.com/thoth-pub/thoth/pull/788) or
    issue [#765](https://github.com/thoth-pub/thoth/issues/765);
24. implement `BE-02`.

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
    merged work **and** the runtime evidence obtained here supports it;
13. **the actor separation of section 2.2 holds throughout**: no deployment, no
    workflow dispatch, no real-environment mode transition, no real fleet
    manipulation, no rollback execution and no use of deployment credentials is
    performed by the implementing agent, in any environment;
14. **the evidence boundary of section 6.3 holds throughout**: the implementing
    agent reads no secret-bearing production configuration, and every external
    fact reaches it either as sanitized metadata that structurally cannot carry a
    production secret value, or as evidence supplied by an authorized operator;
15. where the actor of invariant 13 or the evidence of invariant 14 is
    unavailable, the affected criterion is `BLOCKED` — never satisfied by
    widening what the implementing agent may do.

## 6. Required behaviour

### 6.1 Success behaviour

The task succeeds when it has:

- re-established every external fact, with evidence source and class;
- confirmed ownership, including explicit CTO confirmation of the observation
  sign-off owner;
- obtained every **part-1** runtime-operations decision assigned to
  `THOTH-GQL-OPS-04` in control record section 14 and runbook section 10, while
  leaving every **part-2** decision explicitly unresolved and downstream. Those
  sections deliberately list both, so obtaining *every* entry in them is **not**
  the success condition and would mean absorbing downstream gates;
- established, against a real non-production runtime and on evidence supplied by
  the authorized deployment actor of section 2.2, that the mode-control path
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

### 6.3 Authorization and the evidence boundary

**Binding, and stricter than the parent rule it replaces.** The implementing
agent **must not inspect secret-bearing production configuration directly**. It
holds no read authorization over the private authoritative deployment source for
this task, and it may not create one for itself by scoping a read more narrowly.

Every external runtime or deployment fact this task requires — the container
command, the deployed release per environment, configuration authority and
precedence, rollout and replacement semantics, the autoscaling model, log
retention, access and ownership, the live replica population and any drift —
must reach the implementing agent through **exactly one** of:

```text
ROUTE A -- a SANITIZED METADATA-ONLY SOURCE that structurally cannot
           expose a production secret value: a redacted or
           values-suppressed export, a metadata-only listing, or an
           equivalent artefact from which secret values are absent by
           construction rather than by the reader's care.

ROUTE B -- EVIDENCE SUPPLIED BY AN EXPLICITLY AUTHORIZED human operator,
           control owner or other independently controlled actor, in
           sanitized non-secret form, attributed to a named role.
```

There is no route C. If a fact cannot be obtained by Route A or Route B, the
criterion that needs it is **`BLOCKED`**. A `BLOCKED` criterion is missing work
and is recorded as such; it is never satisfied by the implementing agent reading
the secret-bearing source itself.

**If secret material is nevertheless exposed to the implementing agent** — a
sanitized source turns out not to be sanitized, or supplied evidence carries more
than it should — the agent must:

```text
1. STOP that source/read path immediately;
2. REPORT the exposure at the minimum safe level -- the fact of the
   exposure and the affected read path, and nothing further: no value, no
   location, no resource identifier, no infrastructure detail;
3. PERFORM NO FURTHER READS of that secret-bearing source for this task;
4. record every criterion that depended on it as BLOCKED.
```

Copying secret material onward — into a report, a specification, a changelog, a
pull request, a commit message, a prompt or any other output — is prohibited
absolutely. **Not copying does not make the access acceptable.** An exposure is a
**control/process exception requiring escalation** whether or not anything was
copied, and this specification does not describe any such encounter as
permissible, routine or "not a breach".

Remediating any credential exposure remains **outside this task's scope** and is
a separate CTO-controlled security matter. This task creates and modifies no
security issue.

**Live orchestrator reads** required by section 3 item 6 are performed by the
authorized deployment actor of section 2.2, are **read-only**, must not extend to
production databases or to any mutating operation, and reach the implementing
agent as sanitized evidence under Route B.

#### 6.3.1 Control limitation — the parent scoped-read rule does not govern here

`THOTH-GQL-OPS-01` section 2.2.5 permits a narrowly scoped direct read of the
secret-bearing source and states that an incidental encounter with secret
material is "not a breach" until the material is copied onward.

```text
The stricter repository/project prohibition on implementing-agent access
to production secrets GOVERNS successor execution. Where this
specification and THOTH-GQL-OPS-01 section 2.2.5 differ, THIS section
applies to THOTH-GQL-OPS-04.

The merged parent specification is NOT amended by the pull request that
introduced this text: amending an approved specification requires its own
explicit authorization.

CONTROL LIMITATION, OPEN: the parent rule must be corrected before any
successor requiring secret-bearing production-source access is
authorized. Owner: CTO / control owner. Not closable by an implementing
agent.
```

### 6.4 Concurrency and idempotency

Not applicable to the repository output.

The non-production capability verification of section 3 items 4 and 5 must be
repeatable without leaving that environment in a changed mode: it ends with the
starting mode restored and the restoration verified. **Both the restoration and
its verification are actions of the authorized deployment actor of section 2.2**,
not of the implementing agent, which may only require them, evaluate the evidence
that they occurred, and record whether they did.

If the evidence does not show the starting mode restored and the restoration
verified, that is a stop condition (section 13) and the environment's reported
state is recorded explicitly. The implementing agent must not attempt to restore
the environment itself.

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
Non-production deployment:                   YES -- required by section 3.1,
                                             separately authorized, and
                                             PERFORMED BY THE AUTHORIZED
                                             DEPLOYMENT ACTOR of section 2.2

Deployment performed by the implementing agent:            NONE, in any
                                                           environment
Deployment workflow dispatched by the implementing agent:  NONE
Real-environment mode transition by the implementing agent: NONE
Deployment credentials used by the implementing agent:      NONE
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

Each criterion below is satisfied by **evidence**. Where the evidence requires a
real operational action, that action is performed by the authorized deployment
actor of section 2.2 and the implementing agent evaluates the result. A criterion
whose evidence is unavailable within the section 2.2 actor model and the section
6.3 evidence boundary is **`BLOCKED`**, never satisfied by widening either.

- [ ] **AC-1** Every external fact in the control record is re-established at this
      task's execution time **through the section 6.3 evidence boundary**, with
      evidence source, route (A or B) and class, and any change from the
      previously recorded state is reported as a finding.
- [ ] **AC-2** The container command actually run by the production GraphQL API
      service is re-confirmed through the section 6.3 evidence boundary.
- [ ] **AC-3** Which release each environment actually runs is re-confirmed, with
      pre-guard environments recorded as **pre-guard** and never as
      `MutationGuardMode::OFF`.
- [ ] **AC-4** The execution owner and the request/approval authority are
      re-confirmed as roles.
- [ ] **AC-5** The post-activation observation sign-off owner is confirmed by
      **explicit CTO confirmation**, not by derivation.
- [ ] **AC-6** Rollback authority is resolved by explicit CTO decision, and any
      difference from forward-change authority is stated.
- [ ] **AC-7** The observation-evidence **retention position** is established per
      section 3.4: the retention requirement is recorded; current runtime log
      retention is re-established as a finite configured duration; the approved
      observation-window duration and therefore the coverage question are
      recorded as unresolved and **downstream**; and **no remedy is selected,
      implemented or confirmed in place**. Selecting a remedy here would fail
      this criterion, not satisfy it.
- [ ] **AC-8** The mode-control path is **shown to operate** against a real
      non-production runtime: a guard-enabled release, deployed **by the
      authorized deployment actor** through the production-applicable path,
      consumes the configured value. The evidence is supplied by that actor and
      evaluated here; the implementing agent performs neither the deployment nor
      the mode change.
- [ ] **AC-9** Fleet verification is **shown to operate** against a real
      non-production fleet: the serving population is enumerated, each member is
      attributed an effective mode, and a mixed fleet is detected. The fleet is
      deployed and manipulated by the authorized deployment actor.
- [ ] **AC-10** A partial-fleet state is deliberately created **by the authorized
      deployment actor** in the non-production environment and shown to be
      **detected** by the `THOTH-GQL-OPS-03` mechanism. This shows detection
      **operates**; it does not measure how long a mixed window lasts, which is
      downstream.
- [ ] **AC-11** The silent-adoption failure class is deliberately exercised — by
      the authorized deployment actor — and shown to be detected.
- [ ] **AC-12** Store unavailability outside `ENFORCE` is confirmed
      **operationally**, not assumed, on evidence from that same verification.
- [ ] **AC-13** The live expected replica population is read from orchestrator
      state **by the authorized deployment actor** and supplied as sanitized
      evidence, and the fleet predicate is recorded as instantiated. Any drift
      reported between the authoritative deployment source and the live
      orchestrator is recorded.
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
- [ ] **AC-24** Every external fact was obtained through the section 6.3 evidence
      boundary — Route A sanitized metadata or Route B authorized
      operator-supplied evidence — and the implementing agent performed **no**
      direct read of secret-bearing production configuration. Any criterion whose
      evidence could not be obtained that way is recorded `BLOCKED`.
- [ ] **AC-25** If secret material was exposed to the implementing agent
      notwithstanding AC-24, the report records that the read path was stopped
      immediately, that the exposure was reported at the minimum safe level, that
      no further read of that source occurred, and that the dependent criteria
      were recorded `BLOCKED`. No detail beyond the minimum appears anywhere in
      the diff, and the encounter is recorded as a **control/process exception**,
      not as an acceptable read pattern.
- [ ] **AC-26** The section 2.2 actor separation held: the implementing agent
      triggered no deployment, dispatched no deployment workflow, performed no
      mode transition in any real environment, created or restored no real fleet
      state, executed no rollback in a real environment and used no deployment
      credentials. The report states this explicitly, for **non-production as
      well as production**.
- [ ] **AC-27** Every operational result relied on by AC-8 to AC-13 is attributed
      to the authorized deployment actor that produced it, by role, with its
      evidence class. No such result is recorded as having been produced by the
      implementing agent.
- [ ] **AC-28** Where the authorized deployment actor was unavailable or the
      supplied evidence was inadequate, the affected criteria are recorded
      `BLOCKED` and the disposition is `C`. No agent authority was expanded, no
      simulated or local substitute was accepted in place of real non-production
      evidence, and no evidence requirement was relaxed.
- [ ] **AC-29** No runtime, schema, migration, `Cargo` or workflow file appears
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

Split by actor, per section 2.2. The steps are not reassignable between the two
lists.

**Performed by the AUTHORIZED DEPLOYMENT ACTOR, under its own separate
authorization, and reported to the implementing agent as sanitized non-secret
evidence:**

- deploy a guard-enabled candidate to the **non-production** environment;
- exercise each of `OFF`, `OBSERVE` and `ENFORCE` there through the
  production-applicable command path, and capture the effective mode reported by
  the `THOTH-GQL-OPS-03` mechanism at each;
- enumerate the serving population from live orchestrator state and attribute an
  effective mode to every member;
- deliberately create a mixed fleet, and capture whether it is detected;
- deliberately exercise the silent-adoption failure class, and capture whether it
  is detected;
- exercise the loader store outside `ENFORCE` and capture the observed
  availability;
- exercise a rollback in the non-production environment far enough to show the
  rollback path **operates**, then restore the starting mode and verify the
  restoration. Do **not** time it, and do **not** produce the four rehearsal
  measurements: showing operation is this task's scope, and measuring timing is
  the downstream preview/staging rehearsal's (section 3.2).

**Performed by the IMPLEMENTING AGENT:**

- state, before the above, exactly which candidate, environment, configuration
  and observations the verification requires, and in what sanitized form the
  results must be returned;
- re-establish every external fact through the section 6.3 evidence boundary —
  Route A or Route B — performing no direct read of secret-bearing production
  configuration;
- evaluate the returned evidence against AC-8 to AC-13, reject it where it is
  insufficient, and record `BLOCKED` where it is absent;
- confirm from that evidence that the starting mode was restored and the
  restoration verified;
- confirm that **no production environment was touched**, and that no deployment,
  workflow dispatch, real-environment mode transition or credential use was
  performed by the implementing agent in **any** environment;
- run the merged `THOTH-GQL-OPS-02` and `THOTH-GQL-OPS-03` suites at this task's
  own exact base, locally and in CI.

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
  deployments performed by the implementing agent = none, in any
                                      environment
  ```

- **feature flag/configuration:** none introduced;
- **repository-managed deployment configuration:** this repository holds none;
- **staging/preview validation:** **not** performed by this task. This task
  records bounded non-production **capability verification** — evidence that the
  mode-control and fleet-verification capabilities operate, produced by the
  authorized deployment actor of section 2.2 and evaluated here. The
  implementing agent performs no part of that deployment or transition itself.
  The preview/staging **acceptance gate**, including the performance evidence of
  `ADR-0006` section 7.2.3 and the timed rollback rehearsal, is a separate
  downstream gate (section 3.2);
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
  ends with the starting mode restored and the restoration verified, before the
  task completes. **Both are actions of the authorized deployment actor of
  section 2.2**; the implementing agent requires them, evaluates the evidence
  that they occurred, and records the result. If the evidence does not show them,
  that is a stop condition (section 13), the non-production environment's
  reported state is recorded explicitly, and the implementing agent does **not**
  attempt the restoration itself.

## 13. Stop conditions

The implementing agent must stop and report `BLOCKED` if:

- either `THOTH-GQL-OPS-02` or `THOTH-GQL-OPS-03` is not merged at this task's
  exact base;
- production runtime ownership or the observation sign-off owner cannot be
  confirmed;
- configuration authority cannot be re-established;
- deployed-state evidence cannot be obtained through the section 6.3 evidence
  boundary — no sanitized metadata source and no authorized operator-supplied
  evidence is available;
- required metadata could only be retrieved by a direct implementing-agent read
  of secret-bearing production configuration;
- secret material is exposed to the implementing agent — in which case it stops
  that read path immediately, reports the exposure at the minimum safe level,
  performs no further read of that source, and records the dependent criteria as
  `BLOCKED` (section 6.3);
- **no authorized deployment actor is available** to perform the non-production
  deployment, the mode transitions, the mixed-fleet creation, the rollback or the
  restoration required by section 3.1;
- **the evidence that actor supplies is inadequate** to decide AC-8 to AC-13;
- **any step would require the implementing agent itself to deploy, dispatch a
  deployment workflow, transition a mode in a real environment or use deployment
  credentials** — including in a non-production environment;
- the mode-control path does not actually operate against the real runtime;
- the fleet-verification mechanism cannot achieve complete coverage of a real
  fleet;
- a partial-fleet state cannot be detected;
- the non-production capability verification cannot be completed, or the evidence
  does not show the starting mode restored and the restoration verified;
- the actual runtime contradicts the approved `OFF`/`OBSERVE`/`ENFORCE`
  lifecycle;
- an architecture change is required;
- a change to migration behaviour is required;
- a change to another repository is required, beyond the non-production
  deployment that the authorized deployment actor performs under its own
  authorization;
- runtime implementation is required — that is a new bounded task, not this one;
- a **production** action would be necessary to answer an acceptance criterion.

A stop condition does not authorize scope expansion. It does not authorize
returning disposition `A` on partial evidence, it does not authorize the
implementing agent to perform an operational action reserved to the authorized
deployment actor, and it does not authorize a direct read of secret-bearing
production configuration. The correct response to a missing actor or missing
evidence is `BLOCKED`.

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
observation sign-off owner; every resolved **part-1** decision from control record
section 14 and runbook section 10, including whether rollback additionally
requires CTO approval; the **retention position** of section 3.4 — the
requirement, the re-established finite configured retention, and the explicit
record that the observation-window duration and the coverage question remain
unresolved and downstream, **with no remedy selected**; the evidence that the
mode-control and fleet-verification capabilities **operate**, including the
partial-fleet detection and silent-adoption detection results; the instantiated
fleet predicate and any drift found; the CG-13 mode-control subset disposition
with its justification; the runtime-operations gate state, stated identically
everywhere it appears; whether the runbook's runtime-operations-specific
`PROVISIONAL` state was resolved and why; the two-part status of section 3.3,
stated without being collapsed; an explicit statement that no production
deployment, production mode change or production activation occurred; and CI
status with the classification of each job.

It must additionally record:

- an **actor statement**, per section 2.2: that the implementing agent triggered
  no deployment, dispatched no deployment workflow, performed no mode transition
  in any real environment, created or restored no real fleet state, executed no
  rollback in a real environment and used no deployment credentials — **in
  non-production as well as production** — and, for every operational result
  relied on, the authorized deployment actor role that produced it;
- an **evidence-boundary statement**, per section 6.3: that every external fact
  was obtained by Route A sanitized metadata or Route B authorized
  operator-supplied evidence, that the implementing agent performed no direct
  read of secret-bearing production configuration, and — if secret material was
  nevertheless exposed — that the read path was stopped immediately, the exposure
  reported at the minimum safe level with no further detail, no further read of
  that source performed, and the dependent criteria recorded `BLOCKED`. Such an
  encounter must be recorded as a **control/process exception requiring
  escalation**, and must not be described as acceptable, routine or "not a
  breach" on the ground that nothing was copied onward;
- for each `BLOCKED` criterion, whether it was blocked by a missing authorized
  deployment actor, by inadequate supplied evidence, or by the evidence boundary
  — and the confirmation that no agent authority was expanded in response.

The report must **not** claim the four rehearsal measurements, must **not** claim
the preview/staging acceptance gate, must **not** claim a selected or verified
retention remedy, and must **not** state or imply that the runbook is
production-executable. Those remain downstream (sections 3.2 and 3.4), and the
report must say so explicitly.

If the disposition is `C`, the report must specify the bounded successor task
that would close what remains.

## 15. Recommended execution

Implementation model: Claude Opus, or the strongest available engineering model
Reasoning level: HIGH / maximum practical
Independent reviewer: an independent model family that did not author the
implementation
Review reasoning level: HIGH

**The recommended implementation model is the implementing agent of section 2.2,
and nothing more.** It may inspect, edit, run local and disposable tests, commit,
push and open a draft pull request. It is **not** the deployment actor: it does
not deploy, does not dispatch a deployment workflow, does not transition a mode
in any real environment — production or non-production — and does not hold
deployment credentials.

Execution therefore additionally requires an **authorized deployment actor** — a
human operator or another independently controlled actor — to perform the
section 3.1 non-production deployment and the section 10 operational steps, and
to supply sanitized evidence. **Scheduling this task without that actor available
is scheduling a `BLOCKED` outcome**, and authorizing implementation does not
create the actor.

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

An authorization recorded above authorizes the implementing agent to execute the
**agent** half of section 2.2 only. It does not make the implementing agent the
deployment actor, does not authorize it to deploy or dispatch a deployment
workflow in any environment, and does not permit it to read secret-bearing
production configuration (section 6.3). The authorization of the deployment actor
is a separate decision recorded elsewhere.
