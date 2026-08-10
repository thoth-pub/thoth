# THOTH-GQL-OPS-02 - Mutation-guard mode-control path

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
[`THOTH-GQL-OPS-01`](THOTH-GQL-OPS-01.md) merged; this specification approved; a
freshly verified exact `develop` base; explicit CTO implementation authorization
Target branch name: `feature/shared-architecture/graphql-guard-mode-entrypoint`
(**must not exist** until implementation is authorized)
Production activation effect: NONE. Making the mode settable is not setting it.

Authority condition: this record is repository-authoritative when this exact
content is reachable from the repository's authoritative integration branch.
Live review, authorization and merge evidence is the GitHub pull-request record.

This specification does not authorize implementation, and it does not authorize
production activation. `OFF -> OBSERVE` and `OBSERVE -> ENFORCE` each remain
subject to their own separate explicit CTO production activation approval
(`ADR-0006` section 7.2.1).

## 1. Objective

Make `THOTH_GRAPHQL_MUTATION_GUARD_MODE` actually consumable on the command path
that the production deployment runs, so that once a guard-enabled release is
deployed a mode change is **possible at all** — while preserving every existing
`init` migration and startup semantic exactly as it is today.

This task closes **capability gap 1** of
[`THOTH-GQL-OPS-01`](THOTH-GQL-OPS-01.md). It does not close the
runtime-operations gate, it does not verify anything about a running fleet, and
it activates nothing.

## 2. Background and authority

Authoritative sources:

- [`THOTH-GQL-OPS-01`](THOTH-GQL-OPS-01.md), in particular sections 2.2.3,
  2.2.3.1, 3.12.1, 13.1 and 13.1.1;
- the [mutation-guard runtime-operations control record](../../repository-map/graphql-mutation-guard-runtime-operations.md),
  sections 3.4, 3.7, 4.3 and 4.4;
- [`ADR-0006`](../../decisions/ADR-0006-request-scoped-graphql-batching.md)
  sections 4.12.6.6, 7.2.1, 7.2.1.1 and 7.2.4;
- [`THOTH-GQL-BATCH-01`](THOTH-GQL-BATCH-01.md) and its
  [implementation report](../implementation-reports/THOTH-GQL-BATCH-01-implementation-report.md);
- [CG-13](../../repository-map/control-gaps.md#cg-13---thoth-runtime-operations-unmapped).

Current behaviour, established `[REPO]` and reproduced in an isolated probe:

- the container's default command is `init` (`Dockerfile`), built
  `cargo build --release`;
- `init` (`src/bin/commands/mod.rs`) does **not** register
  `arguments::mutation_guard_mode()`;
- `init` nevertheless dispatches into the **same** handler as
  `start graphql-api` (`src/bin/thoth.rs`), which reads
  `get_one::<String>("mutation-guard-mode") … .unwrap_or("OFF")`;
- in pinned `clap_builder` 4.6.0, `ArgMatches::verify_arg` returns
  `MatchesError::UnknownArgument` **only** under `cfg(debug_assertions)`.
  Therefore a **release** build silently yields `OFF`, and a **debug** build
  **panics**;
- consequently a guard-enabled container running `init` silently ignores
  `THOTH_GRAPHQL_MUTATION_GUARD_MODE` and its effective mode is unconditionally
  `OFF`;
- an **invalid** value is likewise not rejected on the `init` path: the
  `value_parser` never runs, so the process starts normally in `OFF` rather than
  failing to start.

Established `[EXTERNAL]`: the production GraphQL API service supplies no container
command override and so inherits the image default `init`. That is a property of
the **deployment path**; it is not a claim about the guard state of the pre-guard
binary currently deployed. Re-confirming it at this task's execution time is
governed by the evidence boundary of section 6.6.

Established `[REPO]`: `init` runs database migrations first and aborts startup if
they fail, then starts the GraphQL API; `start graphql-api` starts the API
**without** running migrations. The two are **not** interchangeable.

## 3. Explicit scope

The task must:

1. make the mutation-guard mode consumable on the **production-applicable**
   command path — the path the production deployment actually runs — in a
   **release** build;
2. eliminate the release/debug divergence, so that the same access is correct in
   both profiles and no build profile panics;
3. make an **invalid** value fail in the same way on the production-applicable
   path as it already does on `start graphql-api`, rather than being silently
   ignored;
4. preserve the declared default `OFF` on every path, so an absent value yields
   `OFF`;
5. preserve **all** existing `init` migration and startup semantics exactly:
   migrations run first, and startup aborts if they fail;
6. preserve every other existing `init` argument and its behaviour;
7. add the tests required by section 10;
8. add the changelog entry and the implementation report.

The remediation class is **feature-local and in-repository**: registering and
propagating the mutation-guard mode through the `init` command while preserving
those semantics, or another equally bounded in-repository solution that preserves
them.

The implementing task selects the mechanism within that class. `THOTH-GQL-OPS-01`
deliberately did **not** select it, and this specification does not narrow the
class further than the constraints above and the non-goals below require.

## 4. Non-goals

The task must not:

1. activate `OBSERVE`;
2. activate `ENFORCE`;
3. change the guard mode anywhere, in any environment, including preview;
4. set `THOTH_GRAPHQL_MUTATION_GUARD_MODE` anywhere. Making the mode settable is
   **not** setting it;
5. change, reorder, conditionalise or remove migration execution on the startup
   path;
6. change the production container command, or specify doing so as a
   feature-local fix. See section 13.1;
7. change mutation-guard semantics;
8. change batching or store semantics;
9. change the loader-store availability derivation;
10. change `ADR-0006` architecture;
11. add the fleet-verification mechanism — that is `THOTH-GQL-OPS-03`;
12. add runtime observability of any kind;
13. change the public GraphQL schema or the generated SDL;
14. add a database migration, schema change or data change;
15. change production infrastructure, or make any change to the private
    authoritative deployment source;
16. write any production configuration value, secret or resource identifier into
    this repository;
17. close CG-13, or record the runtime-operations gate as satisfied;
18. implement `THOTH-GQL-OPS-03` or `THOTH-GQL-OPS-04`, or create their branches;
19. modify `BE-02`, PR [#788](https://github.com/thoth-pub/thoth/pull/788) or
    issue [#765](https://github.com/thoth-pub/thoth/issues/765);
20. remediate or rotate any credential.

## 5. Invariants

The implementation must preserve:

1. the default mode is `OFF`, and the merged state remains inert: no environment
   is transitioned, and any guard-enabled candidate remains effectively `OFF`;
2. the loader store remains unavailable wherever the guard is not `ENFORCE`
   (`ADR-0006` invariant 30: store availability is derived only from the mode);
3. production request acceptance is unchanged;
4. `init` runs migrations **first** and aborts startup if they fail;
5. every existing `init` argument keeps its current name, environment binding,
   default and behaviour;
6. `start graphql-api` keeps its current behaviour, including its existing
   handling of the mode;
7. the public GraphQL schema and the generated SDL are unchanged;
8. no migration, schema or data change is introduced;
9. merge authorization and the two activation authorizations remain three
   distinct decisions;
10. CG-13 remains open and the runtime-operations gate remains `NOT SATISFIED`
    on this task's completion;
11. no secret or production configuration value enters the repository;
12. an environment running a **pre-guard** release is recorded as pre-guard and
    is never described as `MutationGuardMode::OFF`.

## 6. Required behaviour

### 6.1 Success behaviour

On the production-applicable command path, in a **release** build:

```text
THOTH_GRAPHQL_MUTATION_GUARD_MODE=OFF      -> effective mode OFF
THOTH_GRAPHQL_MUTATION_GUARD_MODE=OBSERVE  -> effective mode OBSERVE
THOTH_GRAPHQL_MUTATION_GUARD_MODE=ENFORCE  -> effective mode ENFORCE
THOTH_GRAPHQL_MUTATION_GUARD_MODE unset    -> effective mode OFF
THOTH_GRAPHQL_MUTATION_GUARD_MODE=<other>  -> process fails to start
```

and, on the same path, in a **debug** build: identical results, with **no**
panic.

Migration execution on that path is byte-for-byte unchanged in ordering and in
failure behaviour.

### 6.2 Failure behaviour

An invalid value must cause the process to **fail to start** rather than start in
an unintended mode. It must never be silently coerced to `OFF`.

Where the task discovers that the bounded in-repository class cannot achieve the
objective while preserving migration semantics, it must **stop and escalate**
(section 13). It must not reach for a container-command override, and it must not
relax a migration semantic to make the fix easier.

### 6.3 Authorization

The task performs no production access, executes no deployment, uses or changes
no credential, dispatches no workflow, and makes no change to the private
authoritative deployment source. It sets no mode in any environment.

**No AI agent or model — the implementing agent or any other, of any role, family
or session — performs a deployment in any environment, production or not,
dispatches a deployment workflow, transitions a mode in a real environment, uses
deployment credentials, or invokes deployment automation in place of an
authorized human/operator.** Any such action belongs to an **authorized non-agent
deployment actor**: an authorized human/operator, or existing deployment
automation executing under that human/operator's own control or initiation. An
automated deployment system is an execution mechanism, not an AI-agent
delegation.

External deployment facts are obtained only through the evidence boundary of
section 6.6; the implementing agent reads no secret-bearing production
configuration. Local, disposable and CI repository testing is ordinary work and
is **not** restricted by this paragraph.

### 6.4 Concurrency and idempotency

Not applicable — process-start configuration only. No concurrent or repeated
execution semantics are introduced or changed.

### 6.5 Compatibility

No API, schema, database, client or deployment-contract change. The public
GraphQL schema is untouched and the generated SDL is unchanged.

**This change is not behaviour-neutral, and must not be described as though it
were.** It is additive on the command-line surface — a newly accepted argument
and environment binding on a path that previously ignored it — but on that path
it deliberately **changes** what two classes of input do. Stating otherwise would
misrepresent the fix as cosmetic and would hide exactly the behaviour the task
exists to introduce.

Current `init` behaviour versus required post-`THOTH-GQL-OPS-02` behaviour:

| `THOTH_GRAPHQL_MUTATION_GUARD_MODE` on `init` | today (release build) | after this task | changed? |
|---|---|---|---|
| unset | `OFF` | `OFF` | no |
| `OFF` | `OFF` | `OFF` | no |
| `OBSERVE` | silently `OFF` | `OBSERVE` | **yes, intentionally** |
| `ENFORCE` | silently `OFF` | `ENFORCE` | **yes, intentionally** |
| invalid value | silently `OFF`, startup succeeds | **startup failure** | **yes, intentionally** |

Compatibility assessment, stated per deployment class:

```text
Known current production and test deployments:
    UNCHANGED. The variable is absent from both service definitions, so
    every one of them takes the unset row above.

Any existing `init` invocation that leaves the variable unset, or sets it
to OFF:
    UNCHANGED -> OFF.

Any existing `init` invocation already supplying OBSERVE or ENFORCE:
    INTENTIONALLY CHANGES BEHAVIOUR, from ignored/OFF to the supplied mode.
    Such an invocation is today silently not doing what it says; after this
    task it does. This is the defect being fixed, not a regression.

Any existing `init` invocation supplying an invalid value:
    INTENTIONALLY CHANGES BEHAVIOUR, from silent OFF with a successful
    startup to a startup failure. This aligns `init` with the existing
    `start graphql-api` behaviour and removes a silent-misconfiguration
    class.
```

**Deployment-facing consequence the implementing task must not gloss.** Because a
previously ignored invalid value will begin to fail startup, any environment that
happens to carry a malformed value would start failing to deploy after this
change. No such environment is known — the variable is absent everywhere — but
the task must re-confirm that at its own execution time rather than inheriting
this statement, and must record the result.

**How that re-confirmation is obtained is constrained.** It must come through the
section 6.6 evidence boundary — sanitized metadata carrying variable names
without values, or evidence supplied by an authorized operator — and never from a
direct implementing-agent read of secret-bearing production configuration. If
neither route is available, the re-confirmation is **`BLOCKED`** and AC-17 fails:
it is recorded as missing work, not inherited from this specification and not
obtained by widening access.

None of this activates anything: the default remains `OFF`, and making the mode
settable is not setting it.

### 6.6 Evidence boundary for external deployment facts — binding

This task needs two external deployment facts: that the production service
supplies no container-command override (section 2), and that no environment
supplies a mutation-guard value that would newly fail startup (section 6.5,
AC-17). Both must be **re-confirmed at this task's own execution time** rather
than inherited from this specification.

**The implementing agent must not obtain them by reading secret-bearing
production configuration directly**, and must not do so by any route it believes
to be narrowly scoped. This prohibition is stricter than, and **governs over**,
the scoped-read rules of `THOTH-GQL-OPS-01` section 2.2.5 — see section 6.6.1.

Each fact must reach the implementing agent through exactly one of:

```text
ROUTE A -- a SANITIZED METADATA-ONLY SOURCE that structurally cannot
           expose a production secret value: a values-suppressed or
           redacted export, a presence/absence listing of variable NAMES
           with values absent by construction, or an equivalent artefact.

ROUTE B -- EVIDENCE SUPPLIED BY AN EXPLICITLY AUTHORIZED HUMAN /
           OPERATOR or CONTROL OWNER, in sanitized non-secret form,
           attributed to a named role -- or a sanitized artefact
           generated under that non-agent human/operator's own control.

           NO AI AGENT OR MODEL is a valid Route B source, in any role,
           family or session. Evidence produced by an AI agent that
           itself inspected production runtime or secret-bearing
           configuration is NOT Route B evidence and must be refused.
```

If neither route can supply a fact, the criterion that needs it — AC-17 for the
mutation-guard value, and the section 2 re-confirmation for the container command
— is **`BLOCKED`**, and the report records it as missing work. It is never
satisfied by a direct implementing-agent read, and never by inheriting this
specification's statement.

If secret material is nevertheless exposed to the implementing agent, it must
**stop that source/read path immediately**, report the exposure at the minimum
safe level — the fact and the affected read path, with no value, location,
resource identifier or infrastructure detail — **perform no further read of that
source**, and record the dependent criteria as `BLOCKED`. Copying secret material
into any output is prohibited absolutely, and not copying does **not** make the
access acceptable: the encounter is a control/process exception requiring
escalation.

This does not broaden the task. The facts required are exactly those already
required; only the permitted route to them is fixed.

#### 6.6.1 Control limitation — the parent scoped-read rule does not govern here

`THOTH-GQL-OPS-01` section 2.2.5 permits a narrowly scoped direct read of the
secret-bearing source and treats an incidental encounter with secret material as
"not a breach" until the material is copied onward.

```text
The stricter repository/project prohibition on implementing-agent access
to production secrets GOVERNS successor execution. Where this
specification and THOTH-GQL-OPS-01 section 2.2.5 differ, THIS section
applies to THOTH-GQL-OPS-02.

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
Public API change:                           NO
Production mode change during implementation: NO
Migration execution semantics changed:       NO
```

Any contrary discovery is a stop and escalation condition (section 13).

## 8. Observability and operations

Required logs: none required by this task. Adding startup or runtime
observability of the effective mode belongs to
[`THOTH-GQL-OPS-03`](THOTH-GQL-OPS-03.md) and must not be absorbed here, so that
the verification mechanism is designed and reviewed as one bounded thing rather
than accreted across two tasks.

Required metrics/alerts: none.

Operational runbook changes: none by this task. The
[mode-transition runbook](../../repository-map/graphql-mutation-guard-mode-transition-runbook.md)
remains **PROVISIONAL** after this task merges, because capability gap 2 remains
open. Only `THOTH-GQL-OPS-04` may lift that marking.

## 9. Acceptance criteria

- [ ] **AC-1** On the production-applicable command path, a **release** build
      consumes `OFF`, `OBSERVE` and `ENFORCE` and yields the corresponding
      effective mode.
- [ ] **AC-2** On that path, an **absent** value yields effective mode `OFF`.
- [ ] **AC-3** On that path, an **invalid** value causes the process to fail to
      start, and never yields a silently coerced `OFF`.
- [ ] **AC-4** The debug/release divergence is eliminated: the same access is
      correct in both profiles, and no profile panics on any of the four cases
      above.
- [ ] **AC-5** `init` still runs migrations **first** and still aborts startup if
      they fail. Ordering and failure behaviour are unchanged.
- [ ] **AC-6** Every other existing `init` argument keeps its name, environment
      binding, default and behaviour.
- [ ] **AC-7** `start graphql-api` behaviour is unchanged.
- [ ] **AC-8** The default remains `OFF` and the merged state remains inert; no
      environment is transitioned and no guard-enabled candidate is activated.
- [ ] **AC-9** Loader-store availability remains derived **only** from the mode,
      and remains unavailable outside `ENFORCE`.
- [ ] **AC-10** No production container-command override is made or specified,
      and any mention of one carries the section 13.1 classification.
- [ ] **AC-11** No migration, schema, data or public-API change appears in the
      diff; the generated SDL is unchanged.
- [ ] **AC-12** No production configuration value, secret or resource identifier
      appears anywhere in the diff.
- [ ] **AC-13** CG-13 remains open and the runtime-operations gate remains
      recorded as `NOT SATISFIED`.
- [ ] **AC-14** `OBSERVE`, `ENFORCE` and `BE-02` remain recorded as
      `NOT AUTHORIZED`; PR #788 and issue #765 are unchanged.
- [ ] **AC-15** The `THOTH-GQL-OPS-03` and `THOTH-GQL-OPS-04` branches do not
      exist and neither task is implemented in this pull request.
- [ ] **AC-16** The section 6.5 compatibility matrix is stated accurately in the
      implementation report, per deployment class, and the two **intentional**
      behaviour changes on the `init` path — `OBSERVE`/`ENFORCE` ceasing to be
      silently ignored, and an invalid value ceasing to start successfully — are
      recorded as intentional rather than described as no change.
- [ ] **AC-17** It is re-confirmed at the task's own execution time that no
      environment currently supplies a value that would newly fail startup, and
      the result is recorded rather than inherited from this specification. The
      re-confirmation is obtained **only** through the section 6.6 evidence
      boundary — sanitized non-secret metadata, or authorized operator-supplied
      evidence — with no direct implementing-agent read of secret-bearing
      production configuration. If neither route can supply it, this criterion is
      **`BLOCKED`**, and `BLOCKED` is the required outcome rather than a widened
      read.
- [ ] **AC-18** **No AI agent or model of any role, family or session** — the
      implementing agent or any other — performed a deployment in any
      environment, production or not, dispatched a deployment workflow, used a
      deployment credential, invoked deployment automation in place of an
      authorized human/operator, or read secret-bearing production configuration.
      The report states this explicitly. Local, disposable and CI repository
      testing is not restricted by this criterion.

## 10. Required tests

### Unit

**Each row of the section 6.5 compatibility matrix must be pinned by its own
test**, so that a later refactor cannot silently reintroduce the silent-ignore
behaviour or silently drop the new failure behaviour:

| Case on the production-applicable path | Pinned expectation |
|---|---|
| unset | `MutationGuardMode::Off` |
| `OFF` | `MutationGuardMode::Off` |
| `OBSERVE` | `MutationGuardMode::Observe` — **not** `Off` |
| `ENFORCE` | `MutationGuardMode::Enforce` — **not** `Off` |
| invalid value | startup failure; **never** a coerced `Off` |

Each of the five must hold in **both** the release and the debug profile, with no
panic in either.

Additionally:

- the mode argument is present in the production-applicable command's registered
  argument set — a direct regression test for the exact defect this task fixes;
- `MutationGuardMode::store_available()` remains true only for `Enforce`.

### Integration/database

- migration execution on the `init` path runs **before** the API starts;
- a migration failure aborts startup and the API does not start;
- both are asserted against the existing behaviour rather than a rewritten one.

### Authorization/security

- no authorization path is changed. A negative test confirming that guard mode
  does not alter any authorization decision is retained or added;
- no secret is read, logged or emitted by the new code path.

### Regression

- the existing `THOTH-GQL-BATCH-01` guard and batching test suites pass unchanged;
- the `debug_assert()` CLI test (`src/bin/thoth.rs`, `test_cli`) passes;
- a debug-profile test exercises the production-applicable path and does **not**
  panic — the current defect's debug symptom;
- the generated SDL is unchanged.

### Manual verification

- build a **release** binary and confirm the section 6.1 matrix on the
  production-applicable path;
- build a **debug** binary and confirm the identical matrix with no panic;
- re-confirm, at this task's own execution time and **through the section 6.6
  evidence boundary only**, that no environment supplies a mutation-guard value
  that would newly fail startup under the section 6.5 change, and record the
  result — or record `BLOCKED` if neither route can supply it;
- confirm no environment was transitioned and no mode was set anywhere;
- confirm that **no AI agent or model of any role, family or session** performed
  a deployment in any environment, dispatched a deployment workflow, used a
  deployment credential, invoked deployment automation in place of an authorized
  human/operator, or read secret-bearing production configuration. Local,
  disposable and CI repository testing is unrestricted.

### Performance

Not applicable. The change affects process start only and adds no request-path
work. The guard's request-path cost is `ADR-0006` section 7.2.3 evidence and
belongs to the activation gate, not here.

## 11. Rollout

- **initial state after merge:** unchanged. The mode becomes *settable* on the
  production-applicable path; it is *not set*.

  ```text
  deployed production release       = pre-guard (no guard mode exists)
  guard-enabled candidate default   = OFF, loader store unavailable
  environments transitioned         = none
  production request acceptance     = unchanged
  runtime-operations gate           = NOT SATISFIED
  ```

- **feature flag/configuration:** none introduced. The guard mode is the existing
  control of a guard-enabled build;
- **repository-managed deployment configuration:** this repository holds none,
  and this task adds none;
- **staging/preview validation:** the timed rehearsal is defined by
  `THOTH-GQL-OPS-01` and executed at the later preview/staging gate. It cannot be
  executed after this task alone, because capability gap 2 remains open;
- **pilot:** not applicable. `OBSERVE` is itself the controlled pilot
  (`ADR-0006` section 7.2.2) and is not authorized by this task;
- **activation approval:** unchanged and still required;
- **observation period:** not applicable to this task.

## 12. Rollback

- **code rollback:** revert the merge commit. Because the merged state leaves the
  default `OFF` and sets no mode anywhere, the revert is a no-op for production
  behaviour;
- **data rollback or forward repair:** none. The task creates no persistent
  state;
- **feature disable/kill switch:** not applicable. The task introduces no
  activated behaviour to disable. Note that the guard mode itself is **not** a
  kill switch: changing it requires a configuration change **and** a deployment
  (control record section 5);
- **external side-effect handling:** none. The task performs no external action.

## 13. Stop conditions

The implementing agent must stop and report `BLOCKED` if:

- the objective cannot be achieved without changing, reordering,
  conditionalising or removing migration execution;
- the objective cannot be achieved without a production container-command
  override;
- the objective would require an `ADR-0006` architecture change;
- the objective would require a migration, schema, data or public-API change;
- the objective would require a change to the private authoritative deployment
  source, or to any other repository;
- the fix cannot be made without changing guard, batching or store semantics;
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
- the pinned `clap`/`clap_builder` behaviour at the task's own exact base differs
  from the behaviour recorded in section 2, such that the established diagnosis
  no longer holds;
- approved architecture would need to change;
- required production information or secrets are unavailable;
- scope cannot be completed without unrelated changes.

### 13.1 The production container-command override: binding classification

Reproduced wherever an override is mentioned at all:

```text
An explicit production command override is NOT an interchangeable
feature-local fix. It changes the current `init` execution path by removing
migration execution from deployment, and therefore requires separate
migration/deployment-control analysis and approval under the broader CG-13
migration/deployment problem.
```

Consequences, binding:

- an override is **out of bounded scope** for this task;
- it must not be offered as an option, a fallback, a "simpler alternative" or an
  expedient;
- if evidence shows that only an override can work, that is an **escalation to
  the CTO** under the migration/deployment half of CG-13 — not a decision this
  task may take;
- **no production command or configuration change is authorized** by this
  specification or by its approval.

## 14. Expected implementation report

The agent must use
[`implementation-report-template.md`](../implementation-report-template.md) and
must record: the exact base and head; the mechanism selected within the bounded
class and why; explicit confirmation that migration ordering and failure
behaviour are unchanged, with the test evidence; the release **and** debug
results for the section 6.1 matrix; the **section 6.5 compatibility assessment
stated per deployment class**, recording the two intentional `init`-path
behaviour changes as intentional rather than as no change, together with the
re-confirmation that no environment supplies a value that would newly fail
startup — stating which section 6.6 route supplied it, or recording it
`BLOCKED`; explicit confirmation that no mode was set in any environment and no
production action occurred; explicit confirmation that **no AI agent or model of
any role, family or session** performed a deployment in any environment,
dispatched a deployment workflow, used a deployment credential, invoked
deployment automation in place of an authorized human/operator, or performed a
direct read of secret-bearing production configuration — and, if secret material was nevertheless exposed, that
the read path was stopped immediately, the exposure reported at the minimum safe
level, no further read of that source performed and the dependent criteria
recorded `BLOCKED`, classified as a control/process exception rather than an
acceptable read; explicit confirmation that no container-command override was made
or specified; the CG-13 state and the runtime-operations gate state, both
unchanged; and CI status with the classification of each job.

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
  `THOTH-GQL-OPS-04`. It is independent of `THOTH-GQL-OPS-03` and the two may
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
