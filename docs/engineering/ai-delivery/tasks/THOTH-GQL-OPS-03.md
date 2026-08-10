# THOTH-GQL-OPS-03 - Effective-mode fleet-verification mechanism

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
[`THOTH-GQL-OPS-01`](THOTH-GQL-OPS-01.md) merged; this specification approved;
an explicit approved decision on the information-disclosure boundary of section
3.2; a freshly verified exact `develop` base; explicit CTO implementation
authorization
Target branch name:
`feature/shared-architecture/graphql-guard-mode-fleet-verification`
(**must not exist** until implementation is authorized)
Production activation effect: NONE. The mechanism observes; it does not
transition anything.

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
[`THOTH-GQL-OPS-01`](THOTH-GQL-OPS-01.md). It does not close the
runtime-operations gate, it verifies no fleet by existing, and it activates
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
  sections 4.2, 4.3, 6 and 7;
- the [mode-transition runbook](../../repository-map/graphql-mutation-guard-mode-transition-runbook.md)
  section 4, which is the consumer of this mechanism;
- [`ADR-0006`](../../decisions/ADR-0006-request-scoped-graphql-batching.md)
  sections 4.12.6.6, 7.2.4, 8.3, 8.3.1, 8.3.4 and 8.3.5;
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

Established `[EXTERNAL]`, under the `THOTH-GQL-OPS-01` section 2.2.5 scoped-read
rules: serving instances sit behind a **shared load balancer**, the service is
deployed by **rolling replacement** so old and new instances serve concurrently,
and the service is **autoscaled**, so the serving population is a range with a
live current value rather than a fixed number.

Established `[REPO]`: a guard-enabled container running the image default `init`
silently ignores the configured mode and runs unconditionally in `OFF`. This is
the **silent-adoption failure class**: configured intent and effective mode
diverge with no symptom. It is the specific failure this mechanism must be able
to catch.

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
5. support **complete coverage** of the actual serving population — enumeration
   of the running instance set from the orchestrator's live state, with a
   per-instance signal for each member — rather than sampling traffic through the
   shared load balancer. A sampled response proves only that *at least one*
   instance carries the observed mode and can never establish fleet consistency;
6. detect a **mixed-mode fleet**;
7. detect the **silent-adoption failure class** specifically — a configured mode
   that the process did not adopt — since that failure is otherwise invisible;
8. resolve and document the information-disclosure boundary of section 3.2;
9. add the tests required by section 10;
10. add the changelog entry and the implementation report.

### 3.1 Smallest-mechanism constraint

The mechanism must be the **smallest** thing that satisfies section 3. It must
not become a general observability, telemetry or admin subsystem, and it must not
grow a second responsibility. Reviewers should reject additions that are useful
but not required by section 3.

### 3.2 Information-disclosure boundary — an explicit decision, not a formatting choice

The guard mode describes a server-side **request-acceptance policy**. Publishing
it tells a caller whether baseline-valid duplicate top-level mutation response
keys are currently rejected, which has reconnaissance value for probing
request-acceptance behaviour.

Binding:

- a **public unauthenticated** mode surface must **not** be selected casually. If
  one is proposed, this specification's approval must record the
  information-disclosure implications explicitly and the CTO must accept them;
- `THOTH-GQL-OPS-01` did **not** grant that acceptance, and this specification
  does not grant it either. It is a listed dependency above;
- the mechanism must expose **no** secret and **no** publisher or user data under
  any option;
- a surface reachable only from the orchestration/administrative plane, or a
  per-instance signal consumed out of band, avoids the disclosure question
  entirely and should be preferred unless evidence shows it cannot satisfy
  section 3.

The task must record which option was selected, the alternatives considered, and
the disclosure assessment for the selected one.

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
11. implement the mode-control path — that is
    [`THOTH-GQL-OPS-02`](THOTH-GQL-OPS-02.md);
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
21. implement `THOTH-GQL-OPS-02` or `THOTH-GQL-OPS-04`, or create their branches;
22. modify `BE-02`, PR [#788](https://github.com/thoth-pub/thoth/pull/788) or
    issue [#765](https://github.com/thoth-pub/thoth/issues/765).

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
    instance must not be read as `OFF`.

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
  verification**, not a partial success;
- the mechanism must fail **closed** with respect to conclusions: it may report
  "not established", and it must never report consistency it did not observe.

### 6.3 Authorization

The task performs no production access, executes no deployment, uses or changes
no credential, dispatches no workflow, makes no change to the private
authoritative deployment source, and sets no mode in any environment.

The mechanism itself must not introduce a new authorization decision, and must
not weaken an existing one.

### 6.4 Concurrency and idempotency

Observing the effective mode must be a **read-only, side-effect-free** operation
that is safe to perform concurrently, repeatedly, and while the service is under
load or mid-rollout. Repeated observation of the same instance must yield the
same answer for the life of that process.

### 6.5 Compatibility

No database, client or deployment-contract change. The public GraphQL schema is
untouched and the generated SDL is unchanged. Any HTTP surface added is additive
and must not alter the behaviour, status codes or payloads of existing routes.

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
signal the mechanism emits must carry the guard mode and the instance identity,
and must **never** carry the full GraphQL document, variables, mutation argument
values, or any publisher or user payload data (`ADR-0006` section 8.3).

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
- [ ] **AC-4** The mechanism supports complete coverage of the running instance
      set by per-instance signal, and does **not** rely on sampling traffic
      through the shared load balancer.
- [ ] **AC-5** A mixed-mode fleet is detectable.
- [ ] **AC-6** The silent-adoption failure class is detectable: a test
      demonstrates an instance whose configured intent and effective mode differ,
      and shows the mechanism reporting the **effective** value.
- [ ] **AC-7** An unreachable or unattributable instance is reported as
      **unknown** and is never coerced to a mode.
- [ ] **AC-8** Request acceptance is unchanged in every mode. A test asserts that
      identical requests produce identical responses, statuses and errors with
      the mechanism present.
- [ ] **AC-9** Guard, batching and store semantics are unchanged, and store
      availability remains derived only from the mode.
- [ ] **AC-10** No publisher or user data and no secret is exposed by the
      mechanism in any mode.
- [ ] **AC-11** The information-disclosure boundary is documented: the option
      selected, the alternatives considered, and the assessment for the selected
      one. If a public unauthenticated surface was selected, the explicit
      approval of that disclosure is recorded.
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
- [ ] **AC-19** The `THOTH-GQL-OPS-02` and `THOTH-GQL-OPS-04` branches do not
      exist and neither task is implemented in this pull request.

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
- **silent-adoption regression:** an instance started on a command path that does
  not consume the configured value reports the mode it actually computed, and the
  divergence from configured intent is visible. This is the defining test of this
  task;
- concurrent observation during simulated load returns consistent answers and
  perturbs no request;
- no database access is introduced by the mechanism.

### Authorization/security

- the mechanism introduces no new authorization decision and changes none;
- negative test: the mechanism exposes no publisher or user data and no secret in
  any mode;
- where a network surface is added, tests cover its reachability boundary
  explicitly — including a negative test for any caller class that must **not**
  reach it.

### Regression

- the existing `THOTH-GQL-BATCH-01` guard and batching suites pass unchanged;
- identical GraphQL requests produce identical responses, statuses and errors
  with the mechanism present, in every mode;
- `MutationGuardMode::store_available()` remains true only for `Enforce`;
- the generated SDL is unchanged.

### Manual verification

- observe the effective mode of a locally running instance in each mode;
- demonstrate detection of a two-instance mixed fleet locally;
- confirm no environment was transitioned and no mode was set anywhere.

### Performance

The mechanism must add **no** measurable work to the GraphQL request path. Where
a surface is added, its own cost is bounded and is not on the `POST /graphql`
path. Guard request-path cost remains `ADR-0006` section 7.2.3 evidence belonging
to the activation gate, not to this task.

## 11. Rollout

- **initial state after merge:** unchanged. The effective mode becomes
  *observable*; no mode is changed and no fleet is verified.

  ```text
  deployed production release       = pre-guard (no guard mode exists)
  guard-enabled candidate default   = OFF, loader store unavailable
  environments transitioned         = none
  production request acceptance     = unchanged
  runtime-operations gate           = NOT SATISFIED
  ```

- **feature flag/configuration:** none introduced beyond what the selected
  mechanism strictly requires. Any configuration it does introduce must default
  to the safest option;
- **repository-managed deployment configuration:** this repository holds none,
  and this task adds none;
- **staging/preview validation:** the timed rehearsal is defined by
  `THOTH-GQL-OPS-01` and executed at the later preview/staging gate, after both
  this task and `THOTH-GQL-OPS-02` have merged. It is not executed here;
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
- the only workable mechanism requires a public unauthenticated surface **and**
  that disclosure has not been explicitly approved;
- the mechanism cannot avoid exposing publisher or user data, or a secret;
- the mechanism would require a new authorization decision;
- an `ADR-0006` architecture change would be required;
- a migration, data change, schema change or public GraphQL schema change turns
  out to be required;
- a production action, deployment or mode change would be needed to satisfy an
  acceptance criterion;
- approved architecture would need to change;
- required production information or secrets are unavailable;
- scope cannot be completed without unrelated changes.

## 14. Expected implementation report

The agent must use
[`implementation-report-template.md`](../implementation-report-template.md) and
must record: the exact base and head; the mechanism selected and the alternatives
rejected, with the section 3.2 disclosure assessment; explicit evidence for the
silent-adoption detection test; explicit confirmation that request acceptance,
guard, batching and store semantics are unchanged, with the regression evidence;
explicit confirmation that **no fleet was verified** and that implementing a
verifier is not verifying a fleet; explicit confirmation that no mode was set in
any environment and no production action occurred; the CG-13 state and the
runtime-operations gate state, both unchanged; confirmation that the runbook
remains `PROVISIONAL`; and CI status with the classification of each job.

## 15. Recommended execution

Implementation model: Claude Opus, or the strongest available engineering model
Reasoning level: HIGH / maximum practical
Independent reviewer: an independent model family that did not author the
implementation
Review reasoning level: HIGH

## 16. Branch and integration plan

- branch source: a freshly verified exact `develop` head;
- pull-request target: `develop`;
- expected merge order: after `THOTH-GQL-OPS-01`, and before
  `THOTH-GQL-OPS-04`. It is independent of `THOTH-GQL-OPS-02` and the two may
  proceed in either order;
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
