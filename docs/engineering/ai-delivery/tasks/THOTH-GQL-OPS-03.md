# THOTH-GQL-OPS-03 - Effective-mode fleet-verification mechanism

Status: APPROVED
Implementation: AUTHORIZED
Programme: Shared Thoth GraphQL / Backend Architecture
Repository: `thoth-pub/thoth`
Workflow: STANDARD
Base branch: `develop`
PR target: `develop`
Programme integration branch: None
Risk: HIGH
Owner: Shared backend architecture
Approved by: CTO / control owner
Dependencies, all required before implementation may begin:
[`ADR-0006`](../../decisions/ADR-0006-request-scoped-graphql-batching.md)
approved and repository-authoritative;
[`THOTH-GQL-BATCH-01`](THOTH-GQL-BATCH-01.md) merged;
[`THOTH-GQL-OPS-01`](THOTH-GQL-OPS-01.md) merged;
[`THOTH-GQL-OPS-02`](THOTH-GQL-OPS-02.md) **implemented, independently reviewed
and merged**; this specification approved, which is also the approval of the
section 3.2 information-disclosure boundary it selects; a freshly verified exact
`develop` base; explicit CTO implementation authorization
Target branch name:
`feature/shared-architecture/graphql-guard-mode-fleet-verification`
(**must not exist** until implementation is authorized)
Production activation effect: NONE. The mechanism observes; it does not
transition anything.

Dependency state at the time this approval candidate was prepared, from
repository and GitHub evidence rather than from narrative:

```text
ADR-0006                        approved / repository-authoritative
THOTH-GQL-BATCH-01              merged
THOTH-GQL-OPS-01                merged
THOTH-GQL-OPS-02                implemented, independently reviewed and
                                merged (PR #797)
section 3.2 disclosure decision RESOLVED by this approval candidate,
                                binding on implementation once this
                                specification is approved
fresh exact `develop` base      still required at implementation time
explicit CTO implementation
  authorization                 still required, and still separate
```

Approving this specification does **not** authorize implementation. The two are
distinct decisions, and neither implies the other.

Authority condition: this record is repository-authoritative when this exact
content is reachable from the repository's authoritative integration branch.
Live review, authorization and merge evidence is the GitHub pull-request record.

This specification does not authorize implementation, and it does not authorize
production activation. `OFF -> OBSERVE` and `OBSERVE -> ENFORCE` each remain
subject to their own separate explicit CTO production activation approval
(`ADR-0006` section 7.2.1).

## 1. Objective

Implement the **smallest** separately reviewable mechanism capable of proving the
**effective** mutation-guard mode of **every serving instance** of the Thoth
GraphQL API, so that a mode change can be verified fleet-wide, a mixed-mode fleet
can be detected, and the silent-adoption failure class can be caught.

This task closes **capability gap 2** of
[`THOTH-GQL-OPS-01`](THOTH-GQL-OPS-01.md). Capability gap 1 is already closed
in-repository by the merged [`THOTH-GQL-OPS-02`](THOTH-GQL-OPS-02.md); gap 2
remains open, and closing it is this task's whole objective. It does not close
the runtime-operations gate, it verifies no fleet by existing, and it activates
nothing.

**Binding distinction, which this task exists because of:**

```text
a specification for a verifier   !=   a verifier
a verifier                       !=   a verified fleet
```

This task delivers the middle term. `THOTH-GQL-OPS-04` uses it to attempt the
third.

## 2. Background and authority

Authoritative sources:

- [`THOTH-GQL-OPS-01`](THOTH-GQL-OPS-01.md), in particular sections 2.2.2, 3.5
  and 3.12.2;
- the [mutation-guard runtime-operations control record](../../repository-map/graphql-mutation-guard-runtime-operations.md),
  sections 4.2, 6 and 7. Its section 4.3 records the capability-gap-1 diagnosis
  as it stood at `THOTH-GQL-OPS-01`'s base; that gap is now closed in-repository
  by the merged `THOTH-GQL-OPS-02`, so section 4.3 is read here as historical
  evidence of the failure **class**, not as current `init` behaviour;
- the [mode-transition runbook](../../repository-map/graphql-mutation-guard-mode-transition-runbook.md)
  section 4, which is the consumer of this mechanism;
- [`ADR-0006`](../../decisions/ADR-0006-request-scoped-graphql-batching.md)
  sections 4.12.6.6, 7.2.4, 8.3, 8.3.1, 8.3.4 and 8.3.5;
- [`THOTH-GQL-OPS-02`](THOTH-GQL-OPS-02.md) **as merged**, and its
  [implementation report](../implementation-reports/THOTH-GQL-OPS-02-implementation-report.md);
- [CG-13](../../repository-map/control-gaps.md#cg-13---thoth-runtime-operations-unmapped).

Current behaviour, established `[REPO]`:

| Surface | Exposes the effective mode? |
|---|---|
| `GET /` (`ApiConfig`) | **no** — `api_name`, `api_version`, `api_schema`, `public_url`, `schema_explorer_url` only |
| `GET /graphiql`, `GET /graphql`, `GET /schema.graphql`, `POST /graphql` | **no** |
| startup logging | **no** — no log call records the effective mode at start |
| guard events | only on a **collision**, and only in `OBSERVE`/`ENFORCE` |

Consequently:

- no mechanism proves the effective mode of a serving instance;
- `OFF` and `OBSERVE` are externally **indistinguishable** — `OBSERVE` never
  rejects, and its only output is a server-side event emitted solely when a
  colliding document happens to arrive;
- the mode is read **once at process start** and there is no reload path, so
  effective mode is a fixed property of a process rather than a queryable
  setting.

Established `[EXTERNAL]`: serving instances sit behind a **shared load balancer**,
the service is deployed by **rolling replacement** so old and new instances serve
concurrently, and the service is **autoscaled**, so the serving population is a
range with a live current value rather than a fixed number. Any re-confirmation
of these facts at this task's execution time is governed by the evidence boundary
of section 6.6.

Established `[REPO]`: **configured intent is not proof of process-effective
mode.** Where the two diverge, they diverge with no symptom — the process starts
healthily, serves traffic normally, and reports nothing about the mode it
actually computed. This is the **silent-adoption failure class**, and it is the
specific failure class this mechanism must be able to catch. It is stated here as
a class rather than as any particular defect (section 2.1).

### 2.1 Post-`THOTH-GQL-OPS-02` reconciliation — binding

This specification was first drafted before `THOTH-GQL-OPS-02` existed, and its
earlier text used the then-current `init` defect as the worked example of the
silent-adoption class. **That defect no longer exists**, and no statement in this
specification may describe it as current repository behaviour.

```text
THOTH-GQL-OPS-02   CLOSED capability gap 1, in-repository.

                   The production-applicable `init` command now consumes
                   THOTH_GRAPHQL_MUTATION_GUARD_MODE:

                       OFF / OBSERVE / ENFORCE   ->  that effective mode
                       unset                     ->  OFF
                       invalid value             ->  startup failure

                   in both the release and the debug build profile.

THOTH-GQL-OPS-03   capability gap 2 remains OPEN.

                   No implemented mechanism proves the effective mode of
                   every serving process / fleet member.
```

**The failure class survives the fix; only that one instance of it is gone.** A
verifier is still required, because:

- configuration intent is not proof of process-effective state, and no mechanism
  currently establishes the latter;
- rolling replacement can produce mixed generations, and therefore mixed modes,
  concurrently behind one load balancer;
- an unknown or unreachable instance must remain **unknown**, and be visible as
  such;
- a future regression, a deployment mismatch or a divergence introduced anywhere
  on the startup path must be **detectable** rather than silent;
- complete fleet verification cannot be inferred by sampling the shared load
  balancer, which can only ever prove that *at least one* instance carries the
  observed mode.

`THOTH-GQL-OPS-02` made the mode **settable** on the production-applicable path.
It did not make any process's effective mode **observable**, and it added no
surface, log or signal that reports it — the current-behaviour table above is
re-derived at this specification's own base and still holds in full.

## 3. Explicit scope

The task must:

1. implement a mechanism that establishes the **effective** guard mode of a
   serving instance — the mode the process actually computed at start, read from
   the same value the request path uses, and **not** re-derived from
   configuration, from an environment variable or from any independently settable
   second signal;
2. distinguish `OFF`, `OBSERVE` and `ENFORCE`;
3. **attribute** an observed mode to a specific, identified serving instance, so
   that "some replicas are on the new mode" and "all replicas are on the new
   mode" are distinguishable;
4. carry a **runtime identity** for the reporting instance that can be correlated
   with the orchestrator's own instance identity, so an observation can be matched
   to an enumerated instance rather than to an anonymous responder;
5. **require** complete coverage of the actual serving population — enumeration
   of the running instance set from the orchestrator's live state, with a
   per-instance signal for each member — rather than sampling traffic through the
   shared load balancer. A sampled response proves only that *at least one*
   instance carries the observed mode and can never establish fleet consistency,
   and a result short of complete coverage is a failure rather than a partial
   success;
6. detect a **mixed-mode fleet**;
7. detect the **silent-adoption failure class** specifically — a process whose
   effective mode differs from the configured intent — since that divergence is
   otherwise invisible. The class is defined generically (section 2.1) and must
   not be reduced to any single historical defect;
8. satisfy the information-disclosure boundary **selected in section 3.2**, and
   record the mechanism chosen within it, the alternatives rejected, and the
   disclosure assessment for the choice;
9. add the tests required by section 10;
10. add the changelog entry and the implementation report.

### 3.1 Smallest-mechanism constraint

The mechanism must be the **smallest** thing that satisfies section 3. It must
not become a general observability, telemetry or admin subsystem, and it must not
grow a second responsibility. Reviewers should reject additions that are useful
but not required by section 3.

### 3.2 Information-disclosure boundary — RESOLVED, and binding on implementation

The guard mode describes a server-side **request-acceptance policy**. Publishing
it tells a caller whether baseline-valid duplicate top-level mutation response
keys are currently rejected, which has reconnaissance value for probing
request-acceptance behaviour.

`THOTH-GQL-OPS-01` deliberately left this decision open and did not grant a
public-disclosure acceptance. **This specification closes it, and closes it
narrowly.** The boundary below is the decision an approval of this specification
approves; it is not an implementation choice the implementing task may revisit.

#### 3.2.1 The selected boundary

```text
SELECTED BOUNDARY

The effective-mode verification signal MUST be available only through an
orchestration/administrative-plane or equivalent out-of-band per-instance
mechanism.

A public unauthenticated effective-mode surface is REJECTED.

The public GraphQL schema MUST remain unchanged.

No public unauthenticated HTTP endpoint may expose OFF / OBSERVE / ENFORCE.

The verification mechanism may expose only:

    1. the process's actual effective MutationGuardMode; and
    2. the minimum runtime identity necessary to correlate that observation
       to the orchestrator's enumerated serving instance.

It must expose no:

    - secret;
    - credential;
    - environment-variable value;
    - deployment configuration;
    - publisher data;
    - user data;
    - request data;
    - unnecessary topology or infrastructure metadata.
```

**Reason.** The mode is server-side request-acceptance policy, so it carries
reconnaissance value, and no public caller needs it in order for
`THOTH-GQL-OPS-03` to do its job. The administrative/out-of-band boundary
satisfies the fleet-verification requirement of section 3 in full while avoiding
a disclosure the task does not need. Choosing the narrower boundary costs the
task nothing it requires.

#### 3.2.2 Alternatives considered and rejected

| Option | Disposition |
|---|---|
| **Public unauthenticated HTTP surface** (a new route, or a new field on `GET /` `ApiConfig`) | **REJECTED.** It publishes request-acceptance policy to any caller, and — because instances sit behind a shared load balancer — it cannot address an individual replica, so it fails section 3 item 5 on its own terms. It buys the disclosure without buying the capability |
| **A field on the public GraphQL schema** | **REJECTED.** Same disclosure, plus a public schema/SDL change the task is forbidden to make (section 4 item 15, section 5 invariant 7) |
| **Authenticated public surface** | **NOT SELECTED.** It would introduce a new authorization decision, which section 4 item 8 and section 6.3 forbid, and it still cannot address an individual replica through the shared load balancer |
| **Administrative/orchestration-plane or out-of-band per-instance signal** | **SELECTED.** It is per-instance by construction, it is collected the same way the orchestrator enumerates the fleet, and it requires no public disclosure and no new authorization decision |

#### 3.2.3 What this decision does and does not fix

This decision fixes the **boundary**, not the **mechanism**. Within the boundary
above, the implementing task selects the smallest transport or signal that
satisfies section 3, and records that selection, the alternatives it rejected and
its disclosure assessment (section 3 item 8, AC-11).

```text
approved boundary   !=   preselected mechanism
```

**Feasibility is NOT ESTABLISHED at specification time, and this specification
does not assert it.** No evidence gathered while preparing this specification
establishes that a compliant mechanism exists inside the approved boundary.
Satisfying section 3 requires **both** a process-effective-mode signal **and** a
runtime identity correlatable to the orchestrator's enumerated instance;
whether any mechanism can supply both within this boundary is a question this
specification deliberately leaves to the implementing task, which must determine
it from evidence at its own execution time. No claim that such a mechanism is
already known to exist may be read into, or written into, this specification.

If implementation-time repository evidence shows that **no** permitted
administrative/orchestration-plane or out-of-band mechanism can satisfy section
3, implementation **stops and returns `BLOCKED`** (section 13). That outcome is a
legitimate result of the task, not a failure of it.

A public unauthenticated surface is **not** an available fallback in that case,
or in any other. Reopening the disclosure decision requires the CTO, not the
implementing task.

## 4. Non-goals

The task must not:

1. activate `OBSERVE`;
2. activate `ENFORCE`;
3. change the guard mode anywhere, in any environment, including preview;
4. change mutation-guard semantics;
5. change batching or store semantics;
6. change the loader-store availability derivation, which must remain derived
   **only** from the mode;
7. change **request acceptance** in any mode, or perturb the response, the
   resolver counts or the errors of any request;
8. add authorization logic, or change any existing authorization decision;
9. add a general telemetry pipeline, dashboard, alerting rule or on-call
   procedure;
10. invent service-health thresholds, latency or error-rate baselines, or
    availability SLOs — that remains the separate `ADR-0006` section 8.3.2 gate;
11. re-open, re-implement or modify the mode-control path delivered by the merged
    [`THOTH-GQL-OPS-02`](THOTH-GQL-OPS-02.md). Its `init` argument registration,
    its `OFF` default, its invalid-value startup failure and its migration
    ordering are settled and out of bounds here;
12. change the production container command, or specify doing so;
13. change, reorder, conditionalise or remove migration execution;
14. add a database migration, schema change or data change;
15. change the public GraphQL schema or the generated SDL;
16. change production infrastructure, or make any change to the private
    authoritative deployment source;
17. write any production configuration value, secret or resource identifier into
    this repository;
18. execute the rehearsal, or verify any real fleet. Implementing a verifier is
    not verifying a fleet;
19. close CG-13, or record the runtime-operations gate as satisfied;
20. lift the `PROVISIONAL` marking from the mode-transition runbook — only
    `THOTH-GQL-OPS-04` may do that;
21. implement `THOTH-GQL-OPS-04` or create its branch;
22. modify `BE-02`, PR [#788](https://github.com/thoth-pub/thoth/pull/788) or
    issue [#765](https://github.com/thoth-pub/thoth/issues/765);
23. expose the effective mode on a **public unauthenticated** surface of any
    kind, or otherwise depart from the section 3.2 boundary. That boundary is
    approved architecture for this task, not an implementation preference.

## 5. Invariants

The implementation must preserve:

1. the mechanism is **inert with respect to request acceptance**: no request is
   accepted or rejected differently in any mode because the mechanism exists;
2. guard, batching and store semantics are unchanged;
3. loader-store availability remains derived **only** from the mode, and remains
   unavailable outside `ENFORCE`;
4. the default mode remains `OFF` and the merged state remains inert; no
   environment is transitioned;
5. no publisher or user data, and no secret, is exposed by the mechanism in any
   mode;
6. the reported mode is the **same value** the request path uses, so the two can
   never disagree. A design in which the reported mode is a separate,
   independently settable value does **not** satisfy this specification;
7. the public GraphQL schema and the generated SDL are unchanged;
8. no migration, schema or data change is introduced;
9. CG-13 remains open and the runtime-operations gate remains `NOT SATISFIED` on
   this task's completion;
10. merge authorization and the two activation authorizations remain three
    distinct decisions;
11. no secret or production configuration value enters the repository;
12. an environment running a **pre-guard** release is recorded as pre-guard and
    is never described as `MutationGuardMode::OFF`. A pre-guard instance has no
    mode for the mechanism to report, and the mechanism's absence on such an
    instance must not be read as `OFF`;
13. **no public unauthenticated surface exposes the effective mode**, and the
    section 3.2 boundary holds in the merged state — not merely in the design
    narrative;
14. the mechanism discloses **only** the effective mode and the minimum runtime
    identity needed to correlate one observation to one orchestrator-enumerated
    instance. No environment-variable value, deployment configuration, request
    data, or topology/infrastructure metadata beyond that minimum is exposed;
15. `THOTH-GQL-OPS-02`'s merged behaviour is unchanged: the production-applicable
    path still consumes `OFF`/`OBSERVE`/`ENFORCE`, an absent value still yields
    `OFF`, an invalid value still fails startup, and `init` still runs migrations
    first and aborts if they fail.

## 6. Required behaviour

### 6.1 Success behaviour

For any serving instance of a guard-enabled release:

```text
the effective mode of THAT instance can be established, and attributed to
that instance, without sampling traffic through the shared load balancer;

the three modes are distinguished;

a fleet in which two enumerated instances report different modes is
detected as mixed;

an instance whose configured mode was not adopted by the process reports
the mode it ACTUALLY computed, so the divergence is visible;

an instance that cannot be attributed a mode is reported as such, and is
never defaulted to any mode.
```

### 6.2 Failure behaviour

- an instance that cannot be reached or attributed a mode is **unknown**, and
  unknown is a distinct outcome from `OFF`. It must never be silently coerced to
  a mode;
- an incomplete enumeration of the serving population is a **failed
  verification**, not a partial success. Partial coverage **cannot pass**: there
  is no result shape in which "most instances reported the intended mode" is a
  success;
- the mechanism must fail **closed** with respect to conclusions: it may report
  "not established", and it must never report consistency it did not observe.

### 6.3 Authorization

The task performs no production access, executes no deployment, uses or changes
no credential, dispatches no workflow, makes no change to the private
authoritative deployment source, and sets no mode in any environment.

**No AI agent or model — the implementing agent or any other, of any role, family
or session — performs a deployment in any environment, production or not,
dispatches a deployment workflow, transitions a mode in a real environment, uses
deployment credentials, or invokes deployment automation in place of an
authorized human/operator.** Any such action belongs to an **authorized non-agent
deployment actor**: an authorized human/operator, or existing deployment
automation executing under that human/operator's own control or initiation. An
automated deployment system is an execution mechanism, not an AI-agent
delegation.

Any external deployment fact the task needs is obtained only through the evidence
boundary of section 6.6; the implementing agent reads no secret-bearing
production configuration. Local, disposable and CI repository testing is ordinary
work and is **not** restricted by this paragraph.

The mechanism itself must not introduce a new authorization decision, and must
not weaken an existing one.

### 6.4 Concurrency and idempotency

Observing the effective mode must be a **read-only, side-effect-free** operation
that is safe to perform concurrently, repeatedly, and while the service is under
load or mid-rollout. Repeated observation of the same instance must yield the
same answer for the life of that process.

### 6.5 Compatibility

No database, client or deployment-contract change. The public GraphQL schema is
untouched and the generated SDL is unchanged. Any surface added must lie within
the section 3.2 boundary; where it is an HTTP surface, it is additive, it is
**not reachable by an unauthenticated public caller**, and it must not alter the
behaviour, status codes or payloads of existing routes. Because section 6.3 also
forbids introducing a new authorization decision, an HTTP surface that satisfies
both constraints is one the public caller cannot reach at all rather than one
gated by new authorization logic.

### 6.6 Evidence boundary for external deployment facts — binding

This task is designed so that its mechanism is built and tested **in-repository**,
against locally reproducible fleet conditions, and it introduces no requirement to
read production configuration. Where an external deployment fact is nevertheless
needed — the load-balancer arrangement, the rolling-replacement semantics or the
autoscaling model of section 2, if re-confirmed rather than inherited — the fact
must reach the implementing agent through exactly one of:

```text
ROUTE A -- a SANITIZED METADATA-ONLY SOURCE that structurally cannot
           expose a production secret value.

ROUTE B -- EVIDENCE SUPPLIED BY AN EXPLICITLY AUTHORIZED HUMAN /
           OPERATOR or CONTROL OWNER, in sanitized non-secret form,
           attributed to a named role -- or a sanitized artefact
           generated under that non-agent human/operator's own control.

           NO AI AGENT OR MODEL is a valid Route B source, in any role,
           family or session. Evidence produced by an AI agent that
           itself inspected production runtime or secret-bearing
           configuration is NOT Route B evidence and must be refused.
```

**The implementing agent must not read secret-bearing production configuration
directly**, by any route, including one it believes to be narrowly scoped. This
prohibition is stricter than, and **governs over**, the scoped-read rules of
`THOTH-GQL-OPS-01` section 2.2.5 — see section 6.6.1.

If neither route can supply a required fact, the criterion that needs it is
**`BLOCKED`** and is recorded as missing work. It is never satisfied by a direct
implementing-agent read.

If secret material is nevertheless exposed to the implementing agent, it must
**stop that source/read path immediately**, report the exposure at the minimum
safe level — the fact and the affected read path, with no value, location,
resource identifier or infrastructure detail — **perform no further read of that
source**, and record the dependent criteria as `BLOCKED`. Copying secret material
into any output is prohibited absolutely, and not copying does **not** make the
access acceptable: the encounter is a control/process exception requiring
escalation.

This does not broaden the task. It fixes the permitted route to facts the task
already needed, and adds no new fact to obtain.

#### 6.6.1 Control limitation — the parent scoped-read rule does not govern here

`THOTH-GQL-OPS-01` section 2.2.5 permits a narrowly scoped direct read of the
secret-bearing source and treats an incidental encounter with secret material as
"not a breach" until the material is copied onward.

```text
The stricter repository/project prohibition on implementing-agent access
to production secrets GOVERNS successor execution. Where this
specification and THOTH-GQL-OPS-01 section 2.2.5 differ, THIS section
applies to THOTH-GQL-OPS-03.

CONTROL LIMITATION, OPEN: the parent rule must be corrected before any
successor requiring secret-bearing production-source access is
authorized. Owner: CTO / control owner. Not closable by an implementing
agent.
```

## 7. Data and migration requirements

Migration required: NO

```text
Database/data change:                        NO
GraphQL schema change:                       NO
Public API change:                           see section 3.2; any surface is
                                             additive and requires the
                                             disclosure decision
Production mode change during implementation: NO
Migration execution semantics changed:       NO
```

Any contrary discovery is a stop and escalation condition (section 13).

## 8. Observability and operations

Required logs: as required by the selected mechanism, and no more. Any log or
signal the mechanism emits must stay within the section 3.2 boundary: it carries
the guard mode and the minimum correlation identity, and it must **never** carry
the full GraphQL document, variables, mutation argument values, any publisher or
user payload data (`ADR-0006` section 8.3), any environment-variable value, any
deployment configuration, or topology/infrastructure metadata beyond that
minimum.

Required metrics/alerts: none. This task identifies and delivers the
**effective-mode** signal only. Service-health signals and activation thresholds
are the separate `ADR-0006` section 8.3.2 gate and must not be absorbed here.

Operational runbook changes: the
[mode-transition runbook](../../repository-map/graphql-mutation-guard-mode-transition-runbook.md)
section 4 describes the verification procedure that consumes this mechanism. This
task may record the concrete mechanism there, but the runbook remains
**PROVISIONAL** after this task merges. Only `THOTH-GQL-OPS-04` may lift that
marking.

## 9. Acceptance criteria

- [ ] **AC-1** The effective mode of a serving instance can be established, and
      it is the same value the request path uses — not a re-derivation from
      configuration and not an independently settable second signal.
- [ ] **AC-2** `OFF`, `OBSERVE` and `ENFORCE` are distinguished.
- [ ] **AC-3** An observed mode is attributed to a specific, identified serving
      instance, and the instance identity can be correlated with the
      orchestrator's own instance identity.
- [ ] **AC-4** The mechanism **requires** complete enumeration of the running
      instance set and supplies a per-instance signal for every enumerated
      member. It does **not** rely on sampling traffic through the shared load
      balancer, and a design that can only sample does not satisfy this
      criterion.
- [ ] **AC-5** A mixed-mode fleet is detectable.
- [ ] **AC-6** The silent-adoption failure class is detectable: a test
      demonstrates a process whose configured intent and effective mode differ,
      and shows the mechanism reporting the **effective** value, so the
      divergence is visible. The test constructs the divergence deliberately and
      does **not** depend on, reintroduce or assume the `init` defect closed by
      `THOTH-GQL-OPS-02`.
- [ ] **AC-7** An unreachable or unattributable instance is reported as
      **unknown** and is never coerced to a mode. `UNKNOWN` is a distinct outcome
      from `OFF` in the result shape itself, not only in prose.
- [ ] **AC-8** Request acceptance is unchanged in every mode. A test asserts that
      identical requests produce identical responses, statuses and errors with
      the mechanism present.
- [ ] **AC-9** Guard, batching and store semantics are unchanged, and store
      availability remains derived only from the mode.
- [ ] **AC-10** No publisher or user data and no secret is exposed by the
      mechanism in any mode.
- [ ] **AC-11** The implementation conforms to the **approved section 3.2
      boundary**, and the report records the mechanism chosen within it, the
      alternatives rejected, and the disclosure assessment for the choice.
- [ ] **AC-11.1** **No public unauthenticated surface exposes the effective
      mode.** A negative test proves that an unauthenticated public caller cannot
      obtain `OFF`, `OBSERVE` or `ENFORCE` from any route, response body or
      header of the public listener, including `GET /` (`ApiConfig`) and every
      existing GraphQL route. A public unauthenticated mode surface is a **fail**,
      not a documented trade-off.
- [ ] **AC-12** The mechanism is read-only, side-effect-free and safe to invoke
      concurrently and during a rollout.
- [ ] **AC-13** No migration, schema, data or public GraphQL schema change
      appears in the diff; the generated SDL is unchanged.
- [ ] **AC-14** No production configuration value, secret or resource identifier
      appears anywhere in the diff.
- [ ] **AC-15** No environment is transitioned and no mode is set anywhere.
- [ ] **AC-16** CG-13 remains open and the runtime-operations gate remains
      recorded as `NOT SATISFIED`. Implementing the verifier is **not**
      verifying a fleet, and the report must not claim otherwise.
- [ ] **AC-17** The runbook remains marked `PROVISIONAL`.
- [ ] **AC-18** `OBSERVE`, `ENFORCE` and `BE-02` remain recorded as
      `NOT AUTHORIZED`; PR #788 and issue #765 are unchanged.
- [ ] **AC-19** `THOTH-GQL-OPS-04` is not implemented in this pull request and
      its branch does not exist. `THOTH-GQL-OPS-02` is merged and is neither
      re-opened nor modified here, and no statement in the diff describes it as
      unimplemented or describes the `init` path as ignoring the guard mode.
- [ ] **AC-20** Any external deployment fact relied on was obtained through the
      section 6.6 evidence boundary — sanitized non-secret metadata, or authorized
      operator-supplied evidence — and the implementing agent performed no direct
      read of secret-bearing production configuration. Where neither route could
      supply a required fact, the affected criterion is recorded **`BLOCKED`**.
- [ ] **AC-21** **No AI agent or model of any role, family or session** — the
      implementing agent or any other — performed a deployment in any
      environment, production or not, dispatched a deployment workflow, used a
      deployment credential, or invoked deployment automation in place of an
      authorized human/operator. The report states this explicitly. Local,
      disposable and CI repository testing is not restricted by this criterion.
- [ ] **AC-22** Incomplete fleet coverage **cannot pass**. A test drives the
      mechanism against an enumerated population in which at least one member is
      unreachable or unattributable, and shows the verification outcome is a
      failure/not-established result rather than a success, with the affected
      member reported `UNKNOWN`.
- [ ] **AC-23** **Minimum disclosure.** The observation carries only the
      effective mode and the minimum runtime identity needed to correlate it to
      one orchestrator-enumerated instance. A test or an enumerated field-by-field
      justification shows that no secret, credential, environment-variable value,
      deployment configuration, request data, publisher or user data, or
      unnecessary topology/infrastructure metadata is carried, and the report
      states why each disclosed identity field is necessary for correlation.

## 10. Required tests

### Unit

- the reported mode equals the mode the request path holds, for each of the three
  modes;
- the reported value is derived from the single stored mode, so the two cannot be
  set independently;
- instance identity is present and stable for the life of the process;
- an unattributable observation yields **unknown**, not a mode.

### Integration/database

- the mechanism is exercised against a running server instance in each of the
  three modes and reports the correct effective mode in each;
- **silent-adoption detection:** a process whose declared/configured intent
  differs from the mode it actually computed reports the **computed** value, and
  comparing the two surfaces the divergence. This is the defining test of this
  task. The divergence is constructed deliberately in the test fixture — it must
  **not** be produced by reintroducing, weakening or depending on the `init`
  defect that `THOTH-GQL-OPS-02` closed, and the test must keep passing however
  the startup path is later refactored;
- **incomplete coverage fails closed:** an enumerated population containing an
  unreachable or unattributable member yields a failed/not-established
  verification and reports that member `UNKNOWN`, never `OFF` and never omitted;
- concurrent observation during simulated load returns consistent answers and
  perturbs no request;
- no database access is introduced by the mechanism.

### Authorization/security

- the mechanism introduces no new authorization decision and changes none;
- negative test: the mechanism exposes no publisher or user data and no secret in
  any mode;
- **negative test for the section 3.2 boundary:** an unauthenticated public
  caller cannot obtain the effective mode from any route, response body or header
  of the public listener — `GET /` (`ApiConfig`), `GET /graphiql`,
  `GET /graphql`, `GET /schema.graphql` and `POST /graphql` are each asserted to
  disclose no mode;
- **minimum-disclosure test:** the observation's payload carries the effective
  mode and the correlation identity and nothing further — no environment-variable
  value, no deployment configuration, no request data;
- where a network surface is added, tests cover its reachability boundary
  explicitly — including a negative test for any caller class that must **not**
  reach it.

### Regression

- the existing `THOTH-GQL-BATCH-01` guard and batching suites pass unchanged;
- the `THOTH-GQL-OPS-02` mode-control suite passes unchanged, in **both** build
  profiles: the production-applicable path still consumes `OFF`/`OBSERVE`/
  `ENFORCE`, an absent value still yields `OFF`, an invalid value still fails
  startup, and migrations still run first and abort startup on failure;
- identical GraphQL requests produce identical responses, statuses and errors
  with the mechanism present, in every mode;
- `MutationGuardMode::store_available()` remains true only for `Enforce`;
- the generated SDL is unchanged.

### Manual verification

- observe the effective mode of a locally running instance in each mode;
- demonstrate detection of a two-instance mixed fleet locally;
- confirm no environment was transitioned and no mode was set anywhere;
- confirm that **no AI agent or model of any role, family or session** performed
  a deployment in any environment, dispatched a deployment workflow, used a
  deployment credential, invoked deployment automation in place of an authorized
  human/operator, or read secret-bearing production configuration.

Every step above is **local and disposable**, and local/disposable testing is
unrestricted. Proving the mechanism against a **real** deployed fleet belongs to
`THOTH-GQL-OPS-04`, where the operational actions are performed by an
**authorized non-agent deployment actor** — an authorized human/operator, or
deployment automation under that human/operator's own control — and never by an
AI agent of any role or family.

### Performance

The mechanism must add **no** measurable work to the GraphQL request path. Where
a surface is added, its own cost is bounded and is not on the `POST /graphql`
path. Guard request-path cost remains `ADR-0006` section 7.2.3 evidence belonging
to the activation gate, not to this task.

## 11. Rollout

- **initial state after merge:** unchanged. The effective mode becomes
  *observable*; no mode is changed and no fleet is verified.

  ```text
  deployed production release       = NOT RE-ESTABLISHED HERE.
                                      The pre-guard finding is historical
                                      THOTH-GQL-OPS-01 evidence owned by its
                                      own record, is not re-certified by this
                                      specification, and is not used for any
                                      conclusion. The implementing task
                                      re-establishes any external deployment
                                      fact it needs through section 6.6.
  guard-enabled candidate default   = OFF, loader store unavailable
  environments transitioned         = none
  production request acceptance     = unchanged
  runtime-operations gate           = NOT SATISFIED
  ```

  A pre-guard release, wherever one is running, has **no** guard mode at all and
  must never be described as `MutationGuardMode::OFF` (section 5 invariant 12).
  That rule binds regardless of which release any environment is currently
  running, so nothing above depends on establishing the current deployed state.

- **feature flag/configuration:** none introduced beyond what the selected
  mechanism strictly requires. Any configuration it does introduce must default
  to the safest option;
- **repository-managed deployment configuration:** this repository holds none,
  and this task adds none;
- **staging/preview validation:** the timed rehearsal is defined by
  `THOTH-GQL-OPS-01` and executed at the later preview/staging gate.
  `THOTH-GQL-OPS-02` has merged, so this task is the remaining capability
  prerequisite — but the rehearsal still sits behind `THOTH-GQL-OPS-04` and the
  service-health/threshold gate, and it is not executed here;
- **pilot:** not applicable;
- **activation approval:** unchanged and still required. Completing this task
  does **not** authorize `OBSERVE`;
- **observation period:** not applicable to this task.

## 12. Rollback

- **code rollback:** revert the merge commit. Because the mechanism is inert with
  respect to request acceptance, the revert is a no-op for production behaviour —
  it removes the ability to observe the mode, not the mode itself;
- **data rollback or forward repair:** none. The task creates no persistent
  state;
- **feature disable/kill switch:** where the selected mechanism is configurable,
  disabling it must be safe and must not affect request acceptance. Note that the
  guard mode itself is **not** a kill switch: changing it requires a
  configuration change **and** a deployment (control record section 5);
- **external side-effect handling:** none. The mechanism performs no external
  action.

## 13. Stop conditions

The implementing agent must stop and report `BLOCKED` if:

- no mechanism can satisfy section 3 without changing request acceptance, guard
  semantics, batching semantics or store semantics;
- per-instance attribution cannot be achieved without a change to the private
  authoritative deployment source or to another repository — note that
  **specifying** such a change is in scope, whereas **making** it is not;
- complete coverage of the serving population cannot be achieved by any
  mechanism this task may implement;
- **no administrative/orchestration-plane or out-of-band mechanism can satisfy
  section 3** — in which case the task stops and escalates. Falling back to a
  public unauthenticated surface is **not** an available resolution: the section
  3.2 boundary is approved architecture, and only the CTO may reopen it;
- the only workable mechanism requires a public unauthenticated effective-mode
  surface, or otherwise departs from the section 3.2 boundary;
- the mechanism cannot avoid exposing publisher or user data, or a secret;
- the mechanism would require a new authorization decision;
- an `ADR-0006` architecture change would be required;
- a migration, data change, schema change or public GraphQL schema change turns
  out to be required;
- a production action, deployment or mode change would be needed to satisfy an
  acceptance criterion;
- any step would require an AI agent or model — the implementing agent or any
  other, of any role, family or session — to deploy, dispatch a deployment
  workflow, transition a mode in a real environment, use deployment credentials
  or invoke deployment automation in place of an authorized human/operator — in
  production or in any non-production environment;
- an external deployment fact can be obtained only by a direct
  implementing-agent read of secret-bearing production configuration, with
  neither section 6.6 route available;
- secret material is exposed to the implementing agent — in which case it stops
  that read path immediately, reports the exposure at the minimum safe level,
  performs no further read of that source, and records the dependent criteria as
  `BLOCKED`;
- approved architecture would need to change;
- required production information is unavailable through the section 6.6
  evidence boundary;
- scope cannot be completed without unrelated changes.

## 14. Expected implementation report

The agent must use
[`implementation-report-template.md`](../implementation-report-template.md) and
must record: the exact base and head; the mechanism selected within the approved
section 3.2 boundary and the alternatives rejected, with the disclosure
assessment and a field-by-field justification of the correlation identity
disclosed; explicit confirmation that **no public unauthenticated surface exposes
the effective mode**, with the negative-test evidence; explicit evidence for the
silent-adoption detection test, stating how the divergence was constructed
without depending on the `init` defect closed by `THOTH-GQL-OPS-02`; explicit
evidence that incomplete fleet coverage fails closed and that `UNKNOWN` is
distinct from `OFF`; explicit confirmation that the merged `THOTH-GQL-OPS-02`
behaviour is unchanged; explicit confirmation that request acceptance,
guard, batching and store semantics are unchanged, with the regression evidence;
explicit confirmation that **no fleet was verified** and that implementing a
verifier is not verifying a fleet; explicit confirmation that no mode was set in
any environment and no production action occurred; explicit confirmation that **no AI
agent or model of any role, family or session** performed a deployment in any
environment, dispatched a deployment workflow, used a deployment credential,
invoked deployment automation in place of an authorized human/operator, or
performed a direct read of secret-bearing production configuration — and, if secret material was
nevertheless exposed, that the read path was stopped immediately, the exposure
reported at the minimum safe level, no further read of that source performed and
the dependent criteria recorded `BLOCKED`, classified as a control/process
exception rather than an acceptable read; the section 6.6 route supplying any
external deployment fact relied on; the CG-13 state and the runtime-operations
gate state, both unchanged; confirmation that the runbook remains `PROVISIONAL`;
and CI status with the classification of each job.

## 15. Recommended execution

Implementation model: Claude Opus, or the strongest available engineering model
Reasoning level: HIGH / maximum practical
Independent reviewer: an independent model family that did not author the
implementation
Review reasoning level: HIGH

## 16. Branch and integration plan

- branch source: a freshly verified exact `develop` head. `develop` has moved
  since this specification was first drafted — `THOTH-GQL-OPS-02` has merged —
  so no base recorded in this file may be reused;
- pull-request target: `develop`;
- expected merge order: after `THOTH-GQL-OPS-01` and after the now-merged
  `THOTH-GQL-OPS-02`, and before `THOTH-GQL-OPS-04`;
- parent programme branch refresh requirement: not applicable — STANDARD
  workflow, no programme integration branch;
- branch deletion after merge: YES;
- final programme PR required: NO;
- final release path: `develop -> master`.

## 17. Approval

### 17.1 What approving this specification decides

Approving this specification is a **specification** decision. It decides exactly
two things and nothing else:

```text
1. that this specification is the approved statement of THOTH-GQL-OPS-03;
2. that the section 3.2 information-disclosure boundary it selects --
   administrative / orchestration-plane or out-of-band only, with a public
   unauthenticated effective-mode surface REJECTED -- is the approved
   boundary, binding on the implementing task.
```

It does **not** authorize implementation, create the implementation branch,
authorize any deployment, or authorize `OFF -> OBSERVE`, `OBSERVE -> ENFORCE` or
`BE-02` runtime. Until implementation is separately and explicitly authorized,
this record stands at `Status: DRAFT` and
`Implementation: NOT AUTHORIZED`, and the implementation branch
`feature/shared-architecture/graphql-guard-mode-fleet-verification` must not
exist.

### 17.2 Implementation authorization

Approved for implementation by: CTO / control owner
Date: 2026-08-11
Notes: Implementation authorization recorded in PR
[#798](https://github.com/thoth-pub/thoth/pull/798) comment
[5252526720](https://github.com/thoth-pub/thoth/pull/798#issuecomment-5252526720),
anchored to exact `develop` base
`2bec75e6698232f7643862120e5437452fcfa252`.

Record only the durable implementation authorization here. Independent review
decisions, CTO merge authorization and the merge itself are terminal GitHub
evidence under [`ADR-0005`](../../decisions/ADR-0005-terminal-merge-evidence.md)
and must not be copied back into this file.
