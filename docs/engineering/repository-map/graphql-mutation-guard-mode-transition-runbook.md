# GraphQL Mutation-Guard Mode-Transition Runbook — **PROVISIONAL**

```text
+---------------------------------------------------------------+
|                                                               |
|   S T A T U S :   P R O V I S I O N A L                       |
|                                                               |
|   THIS RUNBOOK IS NOT EXECUTABLE.                             |
|                                                               |
|   Its procedures cannot be carried out until BOTH             |
|   THOTH-GQL-OPS-02 and THOTH-GQL-OPS-03 have been             |
|   implemented, independently reviewed and merged.             |
|                                                               |
|   Until then:                                                 |
|     - the mode cannot be changed on the production-applicable |
|       deployment path at all (capability gap 1);              |
|     - a change could not be verified if it were made          |
|       (capability gap 2).                                     |
|                                                               |
|   And even once both merge and THOTH-GQL-OPS-04 resolves the  |
|   runtime-operations procedure, this runbook is still NOT     |
|   production-executable: the service-health/threshold gate,   |
|   the preview/staging timed rehearsal and explicit CTO        |
|   activation authorization remain downstream. See section 0.2.|
|                                                               |
|   Do not execute any step below.                              |
|   Do not treat any step below as approved procedure.          |
|                                                               |
+---------------------------------------------------------------+
```

Status: **PROVISIONAL — NOT EXECUTABLE** (both parts of section 0.2 outstanding)
Owner: CTO
Required by: [`ADR-0006`](../decisions/ADR-0006-request-scoped-graphql-batching.md)
section 8.3.5
Produced by: [`THOTH-GQL-OPS-01`](../ai-delivery/tasks/THOTH-GQL-OPS-01.md)
Evidence base: [mutation-guard runtime-operations control record](./graphql-mutation-guard-runtime-operations.md)

Authority condition: this record is repository-authoritative when this exact
content is reachable from the repository's authoritative integration branch.
Live review, authorization and merge evidence is the GitHub pull-request record.

```text
Runtime-operations gate: NOT SATISFIED
CG-13:                   OPEN
OFF -> OBSERVE:          NOT AUTHORIZED
OBSERVE -> ENFORCE:      NOT AUTHORIZED
```

**This runbook authorizes nothing.** It is documentation. Its existence changes no
production behaviour, and it grants no transition approval. Every transition
below additionally requires its own explicit CTO production activation approval,
which this document does not supply and cannot supply.

---

## 0. Why this runbook is provisional

`ADR-0006` section 8.3.5 requires a mode-transition runbook. Its contents depend
on facts that must be established operationally, and on capabilities that do not
yet exist. Writing an executable runbook now would produce a procedure that
cannot be followed and whose verification steps cannot be performed.

Two capabilities are missing. Both are established in the control record and are
restated here because they gate every section below.

```text
CAPABILITY GAP 1 -- the mode cannot be set on the production-applicable path

  The production GraphQL API container inherits the image default command
  `init`. In a release build, `init` does not register the mutation-guard
  argument, so THOTH_GRAPHQL_MUTATION_GUARD_MODE is silently ignored and the
  process's effective mode is unconditionally OFF.

  Consequence: setting the variable would appear to succeed while changing
  nothing. Closed by THOTH-GQL-OPS-02.

CAPABILITY GAP 2 -- a change could not be verified

  No surface exposes the effective mode of a serving instance. OFF and OBSERVE
  are externally indistinguishable. Instances sit behind a shared load
  balancer, so no client can address an individual replica.

  Consequence: fleet consistency, partial-fleet state and successful rollback
  are all currently undecidable. Closed by THOTH-GQL-OPS-03.
```

Additionally, at the time this runbook was written, **both the production and the
test environments are running a pre-guard release** — a binary containing no
mutation guard, and therefore having no guard mode at all, not even `OFF`. There
is currently no environment in which a mode could be changed.

### 0.1 What must happen before this runbook becomes executable

1. `THOTH-GQL-OPS-02` specified, approved, implemented, independently reviewed
   and merged;
2. `THOTH-GQL-OPS-03` specified, approved, implemented, independently reviewed
   and merged;
3. `THOTH-GQL-OPS-04` re-verifies both capabilities against the real runtime,
   obtains the unresolved runtime-operations decisions listed in section 10, and
   resolves **part 1** of the status below;
4. service-health signals and activation thresholds verified and approved
   (`ADR-0006` section 8.3.2) — a **separate downstream** gate;
5. preview/staging acceptance of the exact candidate, including the performance
   evidence of `ADR-0006` section 7.2.3 and the **timed rollback rehearsal**
   (section 7) — a **separate downstream** gate, which supplies this runbook's
   timing fields;
6. explicit CTO production activation approval for the specific transition.

### 0.2 The two-part status, and who resolves each part

This runbook's `PROVISIONAL` state has **two** parts, and they close at different
gates. Collapsing them would either block the runtime-operations gate on
downstream work or, worse, let a runtime-operations result read as production
readiness.

```text
PART 1 -- runtime-operations procedure

  Whether the mode-control and fleet-verification capabilities exist and
  operate, who owns execution, approval, sign-off and rollback, and how the
  fleet predicate is instantiated.

  On observation-evidence retention, part 1 establishes only:
    - the retention REQUIREMENT (section 5.4);
    - that current runtime log retention is a FINITE configured duration;
    - that the observation-window duration is NOT yet established;
    - that coverage is therefore NOT yet established;
    - and hence that the actual remedy is DOWNSTREAM.
  Part 1 does NOT select, implement or confirm a retention remedy.

  Resolved by:  THOTH-GQL-OPS-04, on evidence.
  Blocked by:   capability gaps 1 and 2.

PART 2 -- production transition readiness

  Whether a transition may actually be performed in production.

  On retention, part 2:
    - approves the observation-window duration;
    - determines whether current retention covers it;
    - if not, selects and implements a remedy;
    - verifies the final retention arrangement before activation.

  Resolved by:  the downstream gates, in order --
                  service-health/threshold gate
                  preview/staging rehearsal (supplies every timing field)
                  explicit CTO activation authorization
  NOT resolved by THOTH-GQL-OPS-04.
```

**Why the retention split runs this way.** A remedy cannot be chosen before the
duration it must cover is approved. Requiring `THOTH-GQL-OPS-04` to pick one
would force it either to invent a window duration or to verify coverage of a
duration nobody has set — both of which would manufacture the evidence this
runbook exists to record as missing.

So, once `THOTH-GQL-OPS-04` succeeds, the correct recorded state is:

```text
runtime-operations procedure established

production transition still BLOCKED by:
- service-health/threshold gate
- preview/staging rehearsal
- explicit CTO activation
```

**Timing fields are part 2.** Every duration in this runbook is marked
`TO BE MEASURED AT PREVIEW/STAGING GATE`. Populating them is **not** a condition
of resolving part 1, and `THOTH-GQL-OPS-04` must not treat it as one.

---

## 1. Scope and vocabulary

The subject of every procedure below is the single value:

```text
THOTH_GRAPHQL_MUTATION_GUARD_MODE   in { OFF, OBSERVE, ENFORCE }
```

| Term | Meaning |
|---|---|
| **pre-guard** | a release, image or environment whose binary contains no mutation guard. It has **no** guard mode |
| **guard-enabled candidate** | a build containing the merged `THOTH-GQL-BATCH-01` foundation. Its default mode is `OFF` |
| **guard-enabled environment** | an environment actually running a guard-enabled candidate |
| **the verifier** | the effective-mode fleet-verification mechanism specified in the control record section 7 and delivered by `THOTH-GQL-OPS-03`. **It does not yet exist** |
| **E** | the expected serving population: the live desired/running instance count read from the orchestrator at the moment of the change |

Mode effects, from `ADR-0006` section 4.12.6.6:

| Mode | Rejects? | Observation event | Loader store | Request acceptance |
|---|---|---|---|---|
| `OFF` | no | none | unavailable | unchanged |
| `OBSERVE` | **no** | one per would-be rejection | unavailable | unchanged |
| `ENFORCE` | yes, mutations only | one per actual rejection | may become available | duplicate top-level mutation response keys rejected |

---

## 2. Standing preconditions for any transition

Every transition below requires **all** of the following. Any one unmet is a stop.

- [ ] `THOTH-GQL-OPS-02` merged, so the mode is consumable on the deployment path;
- [ ] `THOTH-GQL-OPS-03` merged, so the verifier exists;
- [ ] `THOTH-GQL-OPS-04` merged, so **part 1** of section 0.2 — the
      runtime-operations procedure — is resolved;
- [ ] a **guard-enabled** release is deployed to the target environment, confirmed
      from the authoritative deployment source — not inferred from `develop`;
- [ ] **part 2** of section 0.2 resolved, namely:
  - [ ] service-health signals and activation thresholds verified and approved
        (`ADR-0006` section 8.3.2);
  - [ ] the preview/staging acceptance gate passed, including the timed rehearsal
        of section 7, with its measured timings recorded and approved and this
        runbook's timing fields populated from them;
- [ ] observation-evidence retention resolved, in dependency order (section 10,
      items 5 to 7), all of which are **part 2**:
  - [ ] the `OBSERVE` observation-window duration approved;
  - [ ] whether the current finite configured runtime log retention covers that
        approved window determined;
  - [ ] if it does not, a remedy selected, implemented and verified.

        The binding requirement is that observation evidence be retained for at
        least the complete approved observation window and remain available
        through review and sign-off. **How** that is achieved is not pre-selected
        anywhere in this runbook, and `THOTH-GQL-OPS-04` neither selects nor
        verifies it;
- [ ] **explicit CTO production activation approval for this specific
      transition**, recorded on the authorizing GitHub record. Merge
      authorization is not activation authorization, and approval of one
      transition is never approval of another.

---

## 3. Mechanism: how a mode change is actually made

Established in the control record sections 3 and 5.

```text
A mode change is BOTH a configuration change AND a deployment.
It is NOT a configuration change without a deploy.
```

```text
1. edit the mutation-guard environment entry in the Thoth GraphQL API service
   definition, in the private authoritative deployment source;
2. commit and push that change -- the deployment tooling refuses to deploy
   unless the local branch is in sync with its upstream, so the push is a
   hard precondition and is also the durable audit record of the change;
3. apply the infrastructure-as-code stack update, producing a new
   service-definition revision;
4. the orchestrator replaces running tasks so that new processes start under
   the new definition.
```

Binding constraints on this mechanism:

- the application reads the mode **once at process start** and has **no reload
  path**. A new process is necessary in every case;
- a **forced redeployment** restarts tasks under the *current* definition. It
  therefore **cannot** change the mode, and must never be used as a substitute
  for step 3;
- a mode change does **not** require publishing a new image or cutting a new
  release; image version and environment are independent parameters;
- **no production container-command override may be used.** See section 9.

No secret, credential, resource identifier, hostname or configuration value from
the authoritative deployment source may be copied into this repository, into a
pull request, into a changelog or into any commit message — including while
executing this runbook.

---

## 4. Fleet verification

**The verifier does not yet exist.** This section states what verification must
establish once it does; it cannot be performed before then.

### 4.1 What "verified" means

```text
A deployment reporting success is evidence that the deployment finished.
It is NOT evidence that every serving instance carries the intended mode.

Never substitute the former for the latter.
```

### 4.2 The procedure

1. **Read E from live orchestrator state** at the moment the change is applied.
   E is a **range with a live current value**, not a fixed number: the service is
   autoscaled on processor utilisation, memory utilisation and per-target request
   count, each with its own cooldown. Do **not** use any count copied from a
   document, including this one — none is recorded here.
2. **Pin the transition window.** Record E at the start and re-read it, because
   autoscaling may change E *during* the transition.
3. **Enumerate the actual running instance set from the orchestrator.** Do not
   sample traffic: instances sit behind a shared load balancer, so a sampled
   response proves only that *at least one* instance carries the observed mode.
4. **Establish the effective mode of every enumerated instance** using the
   verifier. Effective mode means the mode the process actually computed — not
   the configured intent. Configured intent is insufficient evidence, because
   capability gap 1 is precisely a case where the two diverge silently.
5. **Treat any instance started during the window as unknown-mode** until the
   verifier has attributed a mode to it. Both service-definition revisions are
   live during a rollout, so a task started mid-window may start under either.
6. **Propagation is complete** when, for a re-read E, every enumerated instance
   reports the intended effective mode and no instance reports any other mode.

### 4.3 Expected propagation interval

```text
TO BE MEASURED AT PREVIEW/STAGING GATE.

No propagation interval is recorded here, because none is established.
Inventing one is prohibited. The value measured by the downstream
preview/staging rehearsal (section 7) becomes the recorded expectation, and
this field is populated then -- not by THOTH-GQL-OPS-04, which resolves only
part 1 of section 0.2.
```

### 4.4 Verifying store unavailability outside `ENFORCE`

Required by `ADR-0006` section 8.3.5, so the fail-closed coupling of section
4.12.6.6 is confirmed operationally rather than assumed.

Store availability is derived **only** from the mode
(`MutationGuardMode::store_available()` returns true only for `Enforce`), so
confirming the effective mode of an instance is `OFF` or `OBSERVE` is itself the
confirmation that its store is unavailable. The verifier must therefore report
effective mode in a form from which store availability follows without a second,
independently settable signal — a design in which the two are separate values
kept consistent by operator discipline does **not** satisfy this.

---

## 5. Transition: `OFF -> OBSERVE`

**NOT AUTHORIZED. NOT EXECUTABLE.**

`OBSERVE` is itself a HIGH-risk production activation: it parses and selects an
operation for **every** GraphQL request. It requires its own explicit CTO
production activation approval.

### 5.1 Effect

Request acceptance is **unchanged** — `OBSERVE` never rejects. The store remains
unavailable. What changes is added work on the common request path, and the
emission of one structured event per would-be rejection.

### 5.2 Procedure

1. confirm every standing precondition in section 2;
2. record the pre-transition state: the exact candidate/release SHA, the
   image/release identity actually deployed, and the prior mode as verified by
   the verifier — not as assumed;
3. read and record **E** from live orchestrator state;
4. apply the change by the section 3 mechanism;
5. record the identity of the configuration and deployment actions;
6. verify per section 4, re-reading E;
7. monitor for the partial-fleet condition throughout (section 6);
8. declare the transition complete **only** when section 4.2 step 6 holds;
9. begin the explicit, recorded observation window.

### 5.3 Stop conditions during the transition

Stop and roll back if any holds:

- the fleet remains mixed beyond the measured propagation bound;
- any instance reports a mode that is neither `OFF` nor `OBSERVE`;
- the running instance set cannot be completely enumerated;
- a service-health regression attributable to the change appears
  (`ADR-0006` section 8.3.2 thresholds — **not yet established**);
- the verifier itself cannot attribute a mode to an instance.

### 5.4 Observation window

Record, per `ADR-0006` section 7.2.3:

```text
number of mutation requests inspected
number of would-be duplicate-response-key rejections
operation names, where supplied
colliding response keys
period observed
```

**Never** record: full GraphQL documents, variables, mutation argument values, or
any publisher or user payload data.

**Annotate the record with any mixed-mode window that occurred.** During an
`OFF` + `OBSERVE` window, instances still in `OFF` evaluate nothing and emit
nothing, so the would-be-rejection count **understates** real traffic. An
unannotated count would be read as complete when it is not.

**Retain the evidence for the whole window.** Observation evidence must be
retained for at least the **complete approved observation window** and must
remain available through review and sign-off. An observation window whose
evidence expires before it is reviewed is not evidence.

Established `[EXTERNAL]`: runtime log retention for this service is configured to
a **finite** duration.

Not established, and resolved **downstream** in this order (section 10, items 5
to 7): the approved observation-window duration; then whether the configured
retention covers it; then, only if it does not, a remedy — selected, implemented
and verified before activation.

`THOTH-GQL-OPS-04` records the requirement and re-establishes the finite
retention. It does **not** select a remedy, and the means of meeting the
requirement is **not** pre-selected anywhere here.

---

## 6. Partial-fleet handling

**A partial fleet is a FAILED activation. It is never a completed one.**

### 6.1 The three mixed windows are not equally safe

Rolling replacement means old and new tasks serve the same load balancer
concurrently, so a mixed-mode window is **structurally guaranteed** rather than
exceptional. An atomic fleet-wide mode change is not available.

| Mixed window | Client-visible acceptance | Store | Character |
|---|---|---|---|
| `OFF` + `OBSERVE` | **identical** — neither rejects | unavailable in both | **Observation gap.** Relatively benign: an evidence-completeness problem, not a correctness problem. Annotate the observation record; the gap alone is **not** a rollback trigger |
| `OBSERVE` + `ENFORCE` | **differs** — the same document is accepted or rejected depending on which instance serves it | differs | **Request-acceptance inconsistency.** A client retrying an identical request sees non-deterministic acceptance. Not tolerable as an indefinite state |
| `OFF` + `ENFORCE` | **differs** | differs | as above |

Do not flatten the first row into the others. The first is an evidence problem;
the others are client-visible correctness problems.

### 6.2 Detection

Enumerate the live instance set and establish each instance's effective mode via
the verifier. The fleet is mixed when two enumerated instances report different
modes, **or** when any instance cannot be attributed a mode.

### 6.3 Abort criteria

- the fleet remains mixed beyond the measured propagation bound (**to be
  measured**; no bound is invented here);
- any instance reports a mode outside {previous, intended};
- the instance set cannot be completely enumerated;
- for `OBSERVE -> ENFORCE`: **any** legitimate-client rejection.

### 6.4 Rollback trigger

- for `OFF -> OBSERVE`: abort criteria met, or a service-health regression
  attributable to the change. The observation gap alone is not a trigger;
- for `OBSERVE -> ENFORCE`: abort criteria met, **or any legitimate-client
  rejection** — which makes rollback **mandatory**, not optional.

### 6.5 Evidence required after recovery

- the enumerated instance set, re-read after recovery;
- the effective mode of every member;
- an explicit statement that no member reports the aborted mode;
- the elapsed time;
- the observation record annotated with the mixed window;
- for `ENFORCE` rollbacks: confirmation that no client-visible rejection
  persists.

---

## 7. Rehearsal — a downstream gate, required before any production transition

**Defined here. Not executed here. Executing it is not authorized by this
document.**

**Owner: the downstream preview/staging gate — not `THOTH-GQL-OPS-04`.** This
rehearsal belongs to **part 2** of section 0.2. It runs *after* the
runtime-operations gate has been satisfied and after the service-health/threshold
gate, and it is what supplies every timing field in this runbook.
`THOTH-GQL-OPS-04` proves that the capabilities **operate**; this rehearsal
measures how **long** they take.

### 7.1 Prerequisites

1. `THOTH-GQL-OPS-02` merged;
2. `THOTH-GQL-OPS-03` merged;
3. `THOTH-GQL-OPS-04` merged, so the runtime-operations gate is satisfied and
   part 1 of section 0.2 is resolved;
4. the service-health/threshold gate passed (`ADR-0006` section 8.3.2);
5. a **guard-enabled candidate deployed to the non-production environment** —
   required, because that environment is currently **pre-guard** and so has no
   mode to change;
6. the rehearsal separately authorized.

### 7.2 Measurements

```text
time to apply mode change
time to verify fleet consistency
time to rollback
time to verify rollback
```

Each measured using the verifier, so that "verified" means verified **effect**.

These four measurements are owned by **this gate**, not by `THOTH-GQL-OPS-04`.
They populate the runbook fields currently marked
`TO BE MEASURED AT PREVIEW/STAGING GATE`.

### 7.3 Required observations

- deliberately observe a **partial-fleet period at least once**, proving section
  6.2 detection rather than asserting it;
- deliberately exercise the capability-gap-1 failure class — configure a mode and
  confirm whether the process actually adopted it — proving the silent-ignore
  class is detectable;
- confirm operationally that the loader store is unavailable outside `ENFORCE`.

### 7.4 Thresholds

```text
No acceptable numeric limit is stated here.

The measured timings are EVIDENCE for the later activation gate, not a target
set by this runbook. Deriving and approving thresholds is separate work under
ADR-0006 section 8.3.2.
```

---

## 8. Rollback

**NOT A KILL SWITCH.** A rollback requires an edit to a private repository, a
push, a stack update and a full task replacement.

Established, and limited to the technical execution mechanism: rollback uses the
**same configuration/deployment mechanism** and is **technically executed by the
same execution-capability team** as a forward transition.

Not established: its actual latency/duration, which remains **`[UNVERIFIED]`**
and must be measured at the downstream preview/staging rehearsal (section 8.4);
and whether it **additionally requires CTO approval**, which likewise remains
**`[UNVERIFIED]`** (section 8.3). No authorization equivalence is inferred from
sharing the technical mechanism.

It must not be described as immediate or deploy-free.

### 8.1 Four different things, which must not be conflated

| # | Thing | Fixes a live wrong mode? |
|---|---|---|
| 1 | **configuration rollback** — restore the previous value in the authoritative deployment source | **Not on its own** |
| 2 | **deployment/restart action** — the stack update and task replacement that make the restored value take effect | **Yes**, and it is **always** required |
| 3 | **code rollback** — revert the merge commit | **No.** The merged state is `guard OFF, store unavailable`, so a revert is a no-op for production behaviour, and reverting code without replacing processes changes nothing on a running fleet |
| 4 | **release rollback** — return to a previous image version | Only incidentally; a heavier action with its own migration considerations. It is not the mode control |

**Operational rollback of a mode is 1 AND 2 together.**

### 8.2 Procedure, for each transition

Applies identically to `OBSERVE -> OFF`, and later to `ENFORCE -> OBSERVE` and
`ENFORCE -> OFF`:

1. restore the previous mode value in the authoritative deployment source;
2. commit and push;
3. apply the stack update, producing a new service-definition revision;
4. allow task replacement to complete;
5. **verify per section 4** — a rollback is not complete because a deployment
   reported success; it is complete when every enumerated instance reports the
   restored effective mode;
6. record the rollback evidence of section 6.5.

Each transition restores the prior request-acceptance behaviour, and because
store availability is derived only from the mode, each also makes the store
unavailable — so no path is left depending on a guarantee no longer enforced, and
the direct fallback carries every affected field.

### 8.3 Rollback authority

```text
ESTABLISHED -- technical execution mechanism only:
  rollback uses the same configuration/deployment mechanism as a forward
  transition, and is technically executed by the same execution-capability
  team.

  This says HOW a rollback is applied. It says nothing about how long it
  takes, and nothing about who must approve it.

NOT ESTABLISHED -- timing:
  actual rollback latency/duration remains [UNVERIFIED]. See section 8.4.

NOT ESTABLISHED -- authorization:
  whether rollback ADDITIONALLY requires CTO approval remains [UNVERIFIED].

  No authorization equivalence is inferred from sharing the technical
  mechanism. This runbook does not invent an answer in either direction:
  asserting that rollback needs the same approval, and asserting that it
  needs none, are both authorization claims the evidence does not support.

  THOTH-GQL-OPS-04 must obtain an explicit CTO decision and record it here
  as part of resolving part 1 of section 0.2.
```

### 8.4 Duration

```text
TO BE MEASURED AT PREVIEW/STAGING GATE.

Actual rollback latency/duration remains [UNVERIFIED]. No duration is
recorded here, and none is inferred from the forward change: the two share a
mechanism, not a measured time.
```

---

## 9. Prohibited actions

Binding. Reproduced wherever a command override is mentioned at all:

```text
An explicit production command override is NOT an interchangeable
feature-local fix. It changes the current `init` execution path by removing
migration execution from deployment, and therefore requires separate
migration/deployment-control analysis and approval under the broader CG-13
migration/deployment problem.
```

Established: `init` runs database migrations first and aborts startup if they
fail, then starts the GraphQL API; `start graphql-api` starts the API **without**
running migrations. Replacing the production container command would therefore
**remove migration execution from the deployment path**.

Consequently, while executing any procedure in this runbook, do **not**:

1. change the production container command, or offer doing so as a fallback,
   simpler alternative or expedient;
2. change, reorder, conditionalise or remove migration execution;
3. use a forced redeployment as a substitute for a configuration change;
4. treat a deployment reporting success as fleet verification;
5. treat configured intent as effective mode;
6. treat a partial fleet as a completed activation;
7. describe a pre-guard release, image or environment as having a guard mode, or
   as being in `MutationGuardMode::OFF`;
8. copy any secret, credential, resource identifier, private hostname, topology
   detail or configuration value from the authoritative deployment source into
   this repository or any public output;
9. remediate, rotate or otherwise act on credential material encountered in that
   source — escalate it instead;
10. activate `OBSERVE` or `ENFORCE` without that transition's own explicit CTO
    production activation approval.

---

## 10. Unresolved decisions this runbook is waiting on

Each item is labelled with the part of section 0.2 it belongs to, so that no
downstream-owned item is mistaken for a `THOTH-GQL-OPS-04` obligation.

| # | Unresolved | Part | Who resolves it |
|---|---|---|---|
| 1 | accountable production Thoth GraphQL runtime owner — **not** merely who has execution capability | **1** | explicit CTO designation, obtained by `THOTH-GQL-OPS-04` |
| 2 | confirmation of the post-activation observation sign-off owner | **1** | explicit CTO confirmation, obtained by `THOTH-GQL-OPS-04` |
| 3 | whether operational rollback needs CTO approval or may be executed on the technical team's own authority | **1** | explicit CTO decision, obtained by `THOTH-GQL-OPS-04` |
| 4 | the live expected replica population and any configuration drift | **1** | `THOTH-GQL-OPS-04`, from live orchestrator state |
| 5 | the approved `OBSERVE` observation-window duration | **2** | the activation gate; `[UNVERIFIED]` today |
| 6 | whether current finite runtime log retention covers that approved window | **2** | the activation gate, once item 5 exists |
| 7 | the observation-evidence retention **remedy**, if item 6 shows one is needed — selected, implemented and verified | **2** | the activation gate; CTO decision, executed by the execution-capability team |
| 8 | measured propagation interval | **2** | downstream preview/staging rehearsal (section 7) |
| 9 | measured mixed-window bound | **2** | downstream preview/staging rehearsal (section 7) |
| 10 | measured rollback and rollback-verification durations | **2** | downstream preview/staging rehearsal (section 7) |
| 11 | service-health signals and activation thresholds | **2** | separate downstream gate, `ADR-0006` section 8.3.2 |

`THOTH-GQL-OPS-04` must obtain items **1 to 4** and record the answers here. It
must **not** attempt items 5 to 11, and its inability to answer them is **not** a
reason to leave part 1 unresolved.

On retention specifically, `THOTH-GQL-OPS-04` records the **requirement** and
re-establishes that current retention is a finite configured duration; items 5,
6 and 7 then run in that order downstream. Selecting a remedy before item 5
exists would mean choosing a remedy for an unknown target.

---

## 11. Related records

- [mutation-guard runtime-operations control record](./graphql-mutation-guard-runtime-operations.md)
  — the evidence base for every statement here;
- [`THOTH-GQL-OPS-01`](../ai-delivery/tasks/THOTH-GQL-OPS-01.md);
- [`THOTH-GQL-OPS-02`](../ai-delivery/tasks/THOTH-GQL-OPS-02.md) — `DRAFT`,
  `NOT AUTHORIZED`;
- [`THOTH-GQL-OPS-03`](../ai-delivery/tasks/THOTH-GQL-OPS-03.md) — `DRAFT`,
  `NOT AUTHORIZED`;
- [`THOTH-GQL-OPS-04`](../ai-delivery/tasks/THOTH-GQL-OPS-04.md) — `DRAFT`,
  `NOT AUTHORIZED`;
- [`ADR-0006`](../decisions/ADR-0006-request-scoped-graphql-batching.md) sections
  4.12.6.6, 7.2.1, 7.2.1.1, 7.2.4, 7.3, 8.3.1, 8.3.2, 8.3.3 and 8.3.5;
- [CG-13](./control-gaps.md#cg-13---thoth-runtime-operations-unmapped).
