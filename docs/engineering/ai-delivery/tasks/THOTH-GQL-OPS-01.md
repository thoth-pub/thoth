# THOTH-GQL-OPS-01 - GraphQL mutation-guard runtime operations

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
[`THOTH-GQL-BATCH-01`](THOTH-GQL-BATCH-01.md) merged; this specification
approved; a freshly verified exact `develop` base; explicit CTO implementation
authorization
Suggested target branch name: `feature/shared-architecture/graphql-runtime-ops`
(**must not exist** until implementation is authorized)
Production activation effect: NONE. This task establishes how a mode change
would be controlled. It does not perform one.

Authority condition: this record is repository-authoritative when this exact
content is reachable from the repository's authoritative integration branch.
Live review, authorization and merge evidence is the GitHub pull-request record.

This specification does not authorize implementation, and it does not authorize
production activation. `OFF -> OBSERVE` and `OBSERVE -> ENFORCE` each remain
subject to their own separate explicit CTO production activation approval
(`ADR-0006` section 7.2.1).

---

## 1. Objective

Establish, from verified evidence, how the merged GraphQL mutation guard's
operating mode is configured, changed, deployed, propagated across every serving
replica, verified fleet-wide, detected when only partially applied, rolled back,
authorized and evidenced — so that a later `OFF -> OBSERVE` activation becomes
operationally controllable, without this task activating anything.

The subject of the control is the single value:

```text
THOTH_GRAPHQL_MUTATION_GUARD_MODE   in { OFF, OBSERVE, ENFORCE }
```

The task's output is an approved operational-control record and runbook, not a
production change. At completion the guard mode remains `OFF`, the loader store
remains unavailable, and production request acceptance remains unchanged.

## 2. Background and authority

Authoritative sources:

- [`ADR-0006`](../../decisions/ADR-0006-request-scoped-graphql-batching.md),
  approved and repository-authoritative — in particular sections 4.12.6.6 (guard
  operating modes and the fail-closed store coupling), 7.2.1 (activation
  lifecycle), 7.2.1.1 (activation ownership), 7.2.4 (runtime-operations
  prerequisite), 7.3 (rollback), 8.3.2 (service-health signals and thresholds)
  and 8.3.5 (runbook obligation);
- [`THOTH-GQL-BATCH-01`](THOTH-GQL-BATCH-01.md), merged, and its
  [implementation report](../implementation-reports/THOTH-GQL-BATCH-01-implementation-report.md);
- merged PR [#791](https://github.com/thoth-pub/thoth/pull/791) and its terminal
  evidence;
- [`release-gates.md`](../release-gates.md) sections 4, 5 and 8;
- [`operating-model.md`](../operating-model.md) sections 2 and 8;
- [`risk-classification.md`](../risk-classification.md);
- [CG-13](../../repository-map/control-gaps.md#cg-13---thoth-runtime-operations-unmapped);
- [`environments.md`](../../repository-map/environments.md);
- [`repositories/thoth.md`](../../repository-map/repositories/thoth.md).

### 2.1 Why this task exists

`ADR-0006` section 7.2.4 records that the merged foundation establishes a
configuration **input** for the mode and nothing more. It does not establish
whether the value is dynamically reloadable, whether changing it requires a
restart or a deployment, how long propagation takes, whether all replicas
observe the same value, which system owns the value, who may change it, or how a
change or rollback is verified on the running service.

Control gap [CG-13](../../repository-map/control-gaps.md#cg-13---thoth-runtime-operations-unmapped)
states only:

```text
Thoth runtime operations unmapped
```

and requires documentation of runtime, deployment, migration execution,
rollback, restore verification and approvers. `THOTH-GQL-OPS-01` is a **bounded
feature-specific successor**. It addresses the mutation-guard runtime-mode-control
subset of CG-13 and nothing else. Closing all of CG-13 is an explicit non-goal
(section 4), and the disposition rule is section 12.

This task is the **first** item of the authoritative dependency sequence recorded
in `ADR-0006` section 12 and in the
[decision register](../../decisions/decision-register.md):

```text
runtime-operations evidence for mode control          <- THIS TASK
    -> service-health signals and activation thresholds
    -> preview/staging acceptance + performance evidence
       + timed rollback rehearsal
    -> explicit CTO OFF -> OBSERVE authorization
```

### 2.2 Current behaviour, established at base `75f44aabc52d98596ea6ce69ab068b3698fcd524`

The findings below were verified against the repository at the base above and
against the pinned dependency sources resolved by the workspace `Cargo.lock`
(`clap` 4.6.1, `clap_builder` 4.6.0). An implementing agent **must refresh every
finding against its own exact base** before relying on it.

#### 2.2.1 Where the mode is read, and when

| Question | Finding | Evidence |
|---|---|---|
| where the value enters the process | a `clap` argument with `Arg::env(..)`, default `OFF`, `value_parser` restricted to `OFF`/`OBSERVE`/`ENFORCE` | `src/bin/arguments/mod.rs`, `mutation_guard_mode()` |
| where it is parsed | once, in the `start graphql-api` handler, into `MutationGuardMode` | `src/bin/commands/start.rs`, `graphql_api()` |
| where it is stored | captured by the `move` closure passed to `HttpServer::new` and registered as `app_data(Data::new(mutation_guard_mode))` | `thoth-api-server/src/lib.rs`, `start_server()` |
| when it is read | **once, at process start**, before the HTTP server binds | same |
| is there a reload path | **no.** No signal handler, no watcher, no admin route, no re-read anywhere in the workspace | absence verified across `src/bin/`, `thoth-api-server/src/` |

**Consequence, established:** changing the effective mode of a running process is
impossible. The mode changes only by starting a new process with a different
environment. Any claim that the mode can be changed "without a restart" is false
for this codebase.

#### 2.2.2 The effective mode is not observable on a running instance

| Surface | Exposes the mode? | Evidence |
|---|---|---|
| `GET /` (`ApiConfig`) | **no** — `api_name`, `api_version`, `api_schema`, `public_url`, `schema_explorer_url` only | `thoth-api-server/src/lib.rs`, `ApiConfig` |
| `GET /graphiql`, `GET /graphql`, `GET /schema.graphql`, `POST /graphql` | **no** | the complete route set in `thoth-api-server/src/lib.rs` |
| startup logging | **no** — no `log::` call records the effective mode at start | absence verified in `thoth-api-server/src/` and `src/bin/` |
| guard events | only on a **collision**, via `log::warn!`, and only in `OBSERVE`/`ENFORCE` | `thoth-api/src/graphql/mutation_guard.rs` |

**Consequence, established:** there is today **no** mechanism that proves the
effective mode of a serving instance. In particular `OFF` and `OBSERVE` are
externally indistinguishable — `OBSERVE` never rejects, and its only output is a
server-side event emitted solely when a colliding document happens to arrive.
This is a hard blocker for acceptance criterion AC-7 and is why section 3.5
requires a mechanism to be specified.

#### 2.2.3 The `init` entrypoint does not accept the mode — proven

This is the decisive finding of the discovery phase.

1. The container's default command is `init`:

   ```text
   Dockerfile:  CMD ["init"]
   ```

2. The `init` subcommand **does not register** `arguments::mutation_guard_mode()`.
   It registers `database`, `host`, `port`, `threads`, `keep_alive`, `gql_url`,
   `key`, `zitadel_url` and the three AWS arguments, and no others
   (`src/bin/commands/mod.rs`, `INIT`).
3. `init` nevertheless dispatches into the **same** handler as
   `start graphql-api`, passing its own `ArgMatches`
   (`src/bin/thoth.rs`: `commands::run_migrations(arguments)?;
   commands::start::graphql_api(arguments)`).
4. That handler reads the value with
   `arguments.get_one::<String>("mutation-guard-mode") … .unwrap_or("OFF")`
   (`src/bin/commands/start.rs`).
5. In pinned `clap_builder` 4.6.0, `ArgMatches::verify_arg` returns
   `MatchesError::UnknownArgument` **only under `cfg(debug_assertions)`**
   (`src/parser/matches/arg_matches.rs`). Therefore:
   - **release build** — `verify_arg` returns `Ok(())`, the id is absent from
     `self.args`, `try_get_one` returns `Ok(None)`, `get_one` returns `None`, and
     `unwrap_or("OFF")` yields `OFF`;
   - **debug build** — `MatchesError::unwrap` panics with
     `Mismatch between definition and access of 'mutation-guard-mode'`
     (`src/parser/error.rs`).
6. Both branches were **reproduced**, not merely read, in an isolated throwaway
   probe outside this repository that mirrors the exact argument definition and
   the exact access pattern. Release build: `init` yields `OFF` with
   `THOTH_GRAPHQL_MUTATION_GUARD_MODE` set to `ENFORCE`, to `OBSERVE`, and unset.
   Debug build: `init` panics. No repository code was built, modified or added
   for that reproduction.

   The `Dockerfile` builds with `cargo build --release`, so the shipped image
   takes the **release** branch.

**Established consequence:**

```text
When the GraphQL API container runs the image default command `init`,
THOTH_GRAPHQL_MUTATION_GUARD_MODE is silently ignored and the effective
mode is unconditionally OFF.
```

The failure is **silent** and **fail-safe**. It cannot cause an unintended
activation, it does not affect the correctness of the merged inert state, and it
is not a production incident: `OFF` is exactly the merged and intended state. But
it means that setting the environment variable would appear to succeed while
changing nothing, which is precisely the class of failure the fleet-verification
and partial-fleet requirements of this task exist to detect.

Steps 1 to 6 are fully verifiable from this repository and its pinned
dependencies. Whether the **production** GraphQL API container runs `init` or an
explicit `start graphql-api` command is a property of the deployment
configuration, which is owned outside this repository — see section 2.2.4 and
requirement 3.2.

#### 2.2.4 What this repository knows about production runtime

Established here:

- the release artefact is a container image published to
  `ghcr.io/thoth-pub/thoth` on a published GitHub release
  (`.github/workflows/docker_build_and_push_to_dockerhub_release.yml`);
- a per-pull-request staging image is published as
  `staging-pr-<n>` (`.github/workflows/docker_build_and_push_to_dockerhub.yml`);
- the image entrypoint is `/thoth` with default command `init` (`Dockerfile`);
- `docker-compose.yml` is a **local development** composition (Postgres, Redis,
  Zitadel) and defines no Thoth API service. It is not deployment configuration
  and must not be read as such;
- the repository declares **no** deployment manifest, orchestration
  configuration, environment injection or release-ownership record of any kind;
- GitHub repository **environments**: none configured (`total_count: 0`);
  GitHub **deployments**: none. GitHub is therefore not the deployment control
  system and holds no environment reviewers or deployment approvers for this
  repository.

`environments.md` already records, for Thoth, that "the production compute
platform, database migration execution path, deployment approval and rollback
procedure remain unverified under control gap CG-13".

**The authoritative deployment configuration is owned outside this repository.**
Discovery identified the owning control system from evidence as the private
repository `thoth-pub/infrastructure`, which holds the AWS CloudFormation
definitions for the Thoth production and test services, including the container
image version, the container command, and the task environment variables for the
production GraphQL API service. Its recorded update procedure is a
CloudFormation stack update driven from that repository.

Deliberately **not** recorded in this public repository: account identifiers,
resource ARNs, stack, cluster or service names, hostnames beyond those already
published in `environments.md`, autoscaling parameter values, and every value of
every environment variable. The implementing agent must read the current values
from the authoritative source at execution time rather than from this document,
and must not copy them here.

**Hazard, recorded because it constrains how this task may work.** The
authoritative deployment source carries production credential material inline in
template parameters. The implementing agent must therefore treat that source as
read-only, must not reproduce any value from it in the repository, the pull
request, the implementation report or the runbook, and must reference
configuration by **name and location only**. Remediating that exposure is
**outside this task's scope**; it is a separate matter for the CTO and must be
escalated rather than handled here.

**Evidence-classification rule, binding.** Every operational statement the task
records must be labelled with the source class that establishes it:

```text
[REPO]      established by thoth-pub/thoth at the exact base
[EXTERNAL]  established by the authoritative deployment source
[UNVERIFIED] not established -- missing evidence is missing work
```

An `[UNVERIFIED]` answer to any acceptance criterion in section 9 is a failure of
that criterion, not a permitted outcome.

## 3. Explicit scope

The task must establish authoritative answers to the following, each with a named
evidence source and an evidence class.

### 3.1 Runtime owner

Identify the **role or team** responsible for the production Thoth GraphQL
runtime, and separately the role responsible for the post-activation observation
sign-off required by `release-gates.md` section 8 and `ADR-0006` section 7.2.1.1.

- name a role, not an individual;
- record the authoritative source establishing the ownership;
- where the repository's existing control terminology already names an owner
  (`ADR-0006` section 7.2.1.1 assigns merge and both activation approvals to the
  **CTO**), reuse it rather than inventing a parallel vocabulary;
- if the execution owner or the observation sign-off owner cannot be established
  from evidence, that is a stop condition (section 13), because
  `release-gates.md` section 5 requires an explicit activation owner and section
  8 requires an explicit sign-off.

### 3.2 Configuration authority

Establish:

1. where the mutation-guard mode is configured;
2. which source is **authoritative** when more than one could set it;
3. whether the value is an environment variable, a deployment parameter or
   another mechanism;
4. **which command the production GraphQL API container actually runs** — the
   image default `init`, or an explicit `start graphql-api`. This is load-bearing
   because of section 2.2.3: under `init` the variable has no effect at all;
5. behaviour for an **absent** value — established `[REPO]`: `clap` supplies the
   declared default `OFF`;
6. behaviour for an **invalid** value — established `[REPO]`: `clap`'s
   `value_parser` rejects any string outside `OFF`/`OBSERVE`/`ENFORCE`, so the
   process fails to start rather than starting in an unintended mode. The task
   must verify what the orchestrator does with a task that fails to start, since
   a rejected value manifests as a failing deployment rather than as a
   misconfigured running service;
7. who may **request** a change;
8. who may **execute** a change.

No secret value may be recorded. The mode itself is not a secret; its three
values are already public in this repository.

### 3.3 Restart and redeploy semantics

Establish whether changing the mode requires a process restart, a container
replacement, a deployment, a release, or another mechanism, and separate the two
evidence classes explicitly:

- `[REPO]` — established in section 2.2.1: the value is read **once at process
  start** and there is no reload path, so a **new process is necessary** in every
  case. This is a lower bound and is not in question;
- `[EXTERNAL]` — whether a new process additionally requires a new task/container
  definition, a stack or deployment update, and whether it can be achieved
  without publishing a new image version.

The task must state plainly whether a mode change is a configuration change, a
deployment, or both. It must **not** repeat the withdrawn claim that the change
is "a configuration change without a deploy" (`ADR-0006` sections 7.2.4 and 7.3
withdraw it) unless it proves that claim from `[EXTERNAL]` evidence.

### 3.4 Propagation

Establish how a mode change reaches **every** production GraphQL instance, and
specify how to determine:

```text
expected replica population
actual updated replica population
```

Binding requirements:

- the expected population must be derived from the orchestrator's live state at
  the time of the change, **not** from a value copied into this repository. If
  the population is autoscaled rather than fixed, the task must record that the
  expected population is a **range with a live current value**, and must define
  how the population is pinned or re-read during a transition so that
  "all replicas" is a decidable predicate;
- the task must establish whether new instances started **during** a transition —
  by autoscaling, task replacement or health-check-driven restart — start in the
  old or the new mode;
- **do not invent a propagation SLA.** If no authoritative propagation interval
  exists, the task must require it to be **measured** in the rehearsal of section
  3.8, and the measured value becomes the recorded expectation;
- define the observable condition that means propagation is **complete**, in
  terms of the fleet-verification mechanism of section 3.5 rather than in terms
  of the deployment system reporting success. A deployment reporting success is
  evidence that the deployment finished, not that every serving instance carries
  the intended mode.

### 3.5 Fleet verification

Define an evidence mechanism that proves the **effective** mode on **every**
serving instance.

The mechanism must:

- distinguish `OFF`, `OBSERVE` and `ENFORCE`;
- detect inconsistent replicas;
- attribute an observed mode to an identified instance, so that "some replicas
  are on the new mode" and "all replicas are on the new mode" are distinguishable;
- prove **effective** mode as the process actually computed it — not configured
  intent. Section 2.2.3 is the reason this distinction is binding: intent and
  effect can diverge silently;
- expose no secret and no publisher or user data.

**Established constraint the mechanism must satisfy.** Section 2.2.2 establishes
that no such mechanism exists today. Section 2.2.3 establishes that configuration
intent is not sufficient evidence. In addition, instances serve behind a shared
load balancer, so a client cannot address an individual replica: **polling a
public HTTP endpoint cannot, on its own, prove fleet-wide consistency**, because
a sample that returns the new mode proves only that at least one instance carries
it. A compliant mechanism must therefore enumerate the **actual running instance
set from the orchestrator** and establish the effective mode for each member, or
provide an equivalently complete per-instance signal.

If current application telemetry cannot prove effective fleet mode — and section
2.2.2 establishes that it cannot — the task must specify the **smallest**
separately reviewable mechanism that can. Binding constraints on that
specification:

- it must be **specified** here and **implemented and reviewed separately**. The
  task must not add runtime observability in its own documentation PR, and must
  not add it silently anywhere;
- it must not expose the mode on an unauthenticated public surface unless the
  task establishes that doing so is acceptable, with reasoning — the guard mode
  describes a server-side request-acceptance policy, and publishing it is an
  information-disclosure decision, not a formatting choice;
- it must not change guard semantics, batching semantics or store semantics;
- it must remain inert with respect to request acceptance.

### 3.6 Partial-fleet handling

Define what happens when a mode transition reaches only part of the fleet.

This is load-bearing rather than hygiene, because `OBSERVE` versus `ENFORCE`
changes request acceptance, and `ENFORCE` additionally changes loader-store
availability (`ADR-0006` sections 4.12.6.6 and 7.2.4).

**Established structural finding the task must confirm and act on.** A rolling
replacement in which old and new instances serve the same load balancer
concurrently makes a mixed-mode window **guaranteed** rather than exceptional. It
follows that an atomic fleet-wide mode change is not available, and `ADR-0006`
section 7.2.4's alternative therefore applies: the task must establish rollout
semantics and verification that make mixed-mode periods **safe and bounded**.

The task must analyse the two transitions separately, because they are not
equally safe, and must record the conclusion with evidence:

| Mixed window | Client-visible acceptance | Store availability | Assessment to be confirmed |
|---|---|---|---|
| `OFF` + `OBSERVE` | identical — neither rejects | unavailable in both | tolerable; the cost is under-counted observation evidence during the window, which the observation record must account for |
| `OBSERVE` + `ENFORCE` | **differs** — the same document is accepted or rejected depending on which instance serves it | differs | **not** tolerable as an indefinite state; must be bounded, detected and evidenced |
| `OFF` + `ENFORCE` | **differs** | differs | as above |

A partial fleet must **never** be treated as successful activation.

Define, for each transition:

- **detection** — the concrete check, using the section 3.5 mechanism;
- **abort criteria** — the conditions under which the transition stops;
- **rollback trigger** — what makes rollback mandatory rather than optional;
- **authority to initiate rollback** — see section 3.7;
- **required evidence after recovery** — what must be true and recorded before
  the fleet is declared consistent again.

### 3.7 Rollback

Define a concrete operational path for:

```text
OBSERVE -> OFF
```

and, once applicable:

```text
ENFORCE -> OBSERVE
ENFORCE -> OFF
```

The task must distinguish four different things and must not conflate them:

1. **configuration rollback** — restoring the previous mode value;
2. **deployment/restart action** — the action that makes a restored value take
   effect on running instances. Section 2.2.1 establishes this is always
   required;
3. **code rollback** — reverting the merge commit. Established `[REPO]`: because
   the merged state is `guard OFF, store unavailable`, a code revert is a no-op
   for production behaviour and is therefore **not** a remedy for a live
   configuration problem;
4. **production release rollback** — returning to a previously released image
   version.

Binding:

- **do not claim a code revert is sufficient if the live problem is
  configuration.** Section 2.2.1 makes clear it is not: reverting code without
  restarting instances changes nothing on a running fleet;
- **do not claim rollback is immediate or deploy-free without evidence.**
  `ADR-0006` sections 7.2.4 and 7.3 explicitly withdraw both claims;
- record who authorizes a rollback and whether that authority differs from the
  authority to make the forward change. A rollback that requires the same
  approval latency as the forward change is not a kill switch, and the task must
  say so plainly if that is what the evidence shows.

### 3.8 Timed rehearsal requirement

The task must **require** a preview/staging rehearsal that measures:

```text
time to apply mode change
time to verify fleet consistency
time to rollback
time to verify rollback
```

Binding:

- **do not invent acceptable numeric limits.** The measured timings are
  evidence for the later activation gate, not a target set here;
- the measurement must use the section 3.5 fleet-verification mechanism, so that
  "verified" means verified effect rather than a deployment reported as complete;
- the rehearsal must exercise a **partial-fleet** observation at least once, so
  section 3.6's detection is proven rather than asserted;
- the rehearsal must be performed in a **non-production** environment. This
  repository already publishes a per-PR staging image and `environments.md`
  records a Thoth test environment, so a preview/staging target exists; the task
  must confirm the current one from the authoritative source.

**Boundary, settled explicitly as required.** The rehearsal belongs to the later
preview/staging step of the dependency sequence, **not** to this task, and this
task must not perform it. The reason is that a rehearsal requires the section 3.5
verification mechanism to exist, and that mechanism is specified here and
implemented under separate review. `THOTH-GQL-OPS-01` therefore defines the
rehearsal, its measurements, its pass conditions and its evidence format, and
hands execution to the preview/staging gate. If implementation discovers that the
rehearsal cannot be meaningfully defined without first executing part of it, that
is a stop condition (section 13) rather than a licence to execute it.

### 3.9 Authorization boundaries

Preserve, without weakening:

```text
merge authorization
    != OFF -> OBSERVE activation authorization
    != OBSERVE -> ENFORCE activation authorization
```

The task **may** establish who performs an authorized production change. It
**may not** itself grant that authorization. Explicit CTO approval remains
required for each production activation (`ADR-0006` section 7.2.1).

Completion of this task, and merge of its pull request, authorize nothing in
production.

### 3.10 Audit and evidence

Define what must be retained for a mode transition:

- the exact candidate/release SHA and the image version actually deployed;
- the intended mode;
- the previous mode;
- the authorized transition and the authorization record;
- the identity of the deployment or configuration action;
- the fleet-verification result, including the expected and actual instance
  counts;
- the observed propagation interval;
- the rollback result where rehearsed or executed;
- the observation-window record where applicable.

Binding:

- use GitHub, release and runtime records as appropriate, per `ADR-0005` and
  `operating-model.md` section 5.1 — do not create a repository commit merely to
  restate an event those systems already hold;
- **do not duplicate secrets or sensitive production configuration into the
  repository**, in any file, including the runbook;
- the task must establish whether the runtime log retention available to the
  chosen evidence mechanism is long enough to cover a recorded observation
  window, and must record the answer. Where retention is shorter than the window,
  evidence must be captured out of band or the window adjusted; an observation
  window whose evidence expires before it is reviewed is not evidence.

### 3.11 Required deliverables

1. an operational-control record answering sections 3.1 to 3.10, every statement
   carrying its evidence source and evidence class;
2. the mode-transition **runbook** required by `ADR-0006` section 8.3.5, covering
   every item that section lists, including how store unavailability outside
   `ENFORCE` is verified operationally rather than assumed;
3. a **separately reviewable specification** for the smallest fleet-verification
   mechanism (section 3.5), not its implementation;
4. the CG-13 disposition statement required by section 12;
5. the changelog entry and the implementation report.

## 4. Non-goals

The task must **not**:

1. activate `OBSERVE`;
2. activate `ENFORCE`;
3. change the guard mode anywhere, in any environment, including preview;
4. deploy the batching foundation merely to test activation, unless separately
   authorized;
5. change `ADR-0006` architecture;
6. change mutation-guard semantics;
7. change batching or store semantics;
8. adopt a production loader;
9. modify `BE-02`;
10. merge PR [#788](https://github.com/thoth-pub/thoth/pull/788);
11. close issue [#765](https://github.com/thoth-pub/thoth/issues/765);
12. invent monitoring thresholds;
13. define GraphQL latency, error-rate or availability SLOs without evidence;
14. perform `BE-02` implementation;
15. change production credentials;
16. rotate secrets;
17. perform a production database migration;
18. close all of CG-13;
19. document unrelated migration or restore operations merely to claim CG-13 is
    complete;
20. implement the fleet-verification mechanism of section 3.5 in this task's
    pull request;
21. remediate the credential-exposure hazard noted in section 2.2.4 — escalate it
    instead;
22. write any production configuration value, secret or resource identifier into
    this repository.

## 5. Invariants

The implementation must preserve:

1. `THOTH_GRAPHQL_MUTATION_GUARD_MODE` remains `OFF` in every environment the
   task touches;
2. the loader store remains unavailable, which follows structurally from
   invariant 1 (`ADR-0006` invariant 30: store availability is derived only from
   the mode);
3. production request acceptance is unchanged;
4. no runtime, schema, migration, `Cargo` or workflow file is changed;
5. no production action of any kind is performed;
6. merge authorization and the two activation authorizations remain three
   distinct decisions;
7. CG-13 remains open except for any explicitly bounded, evidenced subset;
8. `BE-02` remains unauthorized;
9. no secret or production configuration value enters the repository;
10. no operational claim is recorded without a named evidence source and
    evidence class.

## 6. Required behaviour

### 6.1 Success behaviour

An approved specification and runbook exist from which an authorized operator
could execute `OFF -> OBSERVE`, verify it fleet-wide, detect a partial fleet,
and roll back — with every step evidenced and no step requiring an invented fact.

### 6.2 Failure behaviour

Where evidence is unavailable, the task records the **exact** missing evidence
and returns `BLOCKED` for the affected criterion. It does not substitute a
plausible mechanism, and it does not soften an unanswerable question into a
narrative. Missing evidence is missing work.

### 6.3 Authorization

The task performs no production access, executes no deployment, retrieves no
secret and dispatches no workflow. Inspection of any external authoritative
source is **read-only** and limited to ownership, mechanism and configuration
**metadata**.

### 6.4 Concurrency and idempotency

Not applicable — the task produces documentation only.

### 6.5 Compatibility

No API, schema, database, client or deployment compatibility effect. The public
GraphQL schema is untouched, and the generated SDL is unchanged.

## 7. Data and migration requirements

```text
Migration required:                      NO
Database/data change:                    NO
GraphQL schema change:                   NO
Public API change:                       NO
Production mode change during implementation: NO
```

Any contrary discovery is a stop and escalation condition (section 13).

## 8. Observability and operations

Required logs: none added by this task.

Required metrics/alerts: none added by this task. Section 3.5 **specifies** a
fleet-verification mechanism; it does not build one. Service-health signals and
activation thresholds are explicitly **out of scope** and remain a separate
gate — see section 11.

Operational runbook changes: this task produces the mode-transition runbook
required by `ADR-0006` section 8.3.5. The runbook is documentation. Its
existence changes no production behaviour and authorizes no transition.

## 9. Acceptance criteria

Every criterion must be satisfied with a named evidence source and an evidence
class. `[UNVERIFIED]` is a failure, not an outcome.

- [ ] **AC-1** Authoritative production runtime owner identified as a role, with
      the source that establishes it. *Evidence: authoritative ownership record
      named in the operational-control document.*
- [ ] **AC-2** Post-activation observation sign-off owner identified as a role.
      *Evidence: as AC-1; required by `release-gates.md` section 8 and
      `ADR-0006` section 7.2.1.1.*
- [ ] **AC-3** Authoritative configuration source identified, including which
      source wins if more than one could set the value. *Evidence: the
      authoritative deployment source, cited by name and location only.*
- [ ] **AC-4** Exact mutation-guard configuration mechanism identified,
      **including which command the production container runs**, with the
      section 2.2.3 finding either confirmed or refuted for production.
      *Evidence: `[REPO]` `Dockerfile`, `src/bin/commands/mod.rs`,
      `src/bin/thoth.rs`, `src/bin/commands/start.rs`, pinned `clap_builder`
      sources; `[EXTERNAL]` the container command in the authoritative
      deployment source.*
- [ ] **AC-5** Restart/redeploy requirement proven, with `[REPO]` and
      `[EXTERNAL]` evidence separated. *Evidence: `thoth-api-server/src/lib.rs`
      for the process-start read; the deployment source for the rest.*
- [ ] **AC-6** Propagation mechanism proven, including the behaviour of
      instances started during a transition. *Evidence: deployment/orchestration
      configuration.*
- [ ] **AC-7** Expected fleet definition established as a decidable predicate,
      derived from live orchestrator state rather than a copied value.
      *Evidence: orchestrator live state at execution time.*
- [ ] **AC-8** Effective-mode fleet verification mechanism defined, proving
      **effect** rather than intent, attributing mode to instance, and
      distinguishing all three modes. *Evidence: the section 3.5 specification,
      plus the section 2.2.2 finding that no such mechanism exists today.*
- [ ] **AC-9** Partial-fleet state detectable by that mechanism. *Evidence: the
      detection procedure in the runbook.*
- [ ] **AC-10** Partial-fleet state explicitly treated as **failed** activation,
      with abort criteria and rollback trigger. *Evidence: runbook.*
- [ ] **AC-11** The `OBSERVE`/`ENFORCE` mixed window is analysed separately from
      the `OFF`/`OBSERVE` mixed window, with a recorded safety conclusion.
      *Evidence: the mode table in `ADR-0006` section 4.12.6.6 plus the rollout
      semantics of the deployment source.*
- [ ] **AC-12** Rollback procedure defined for each applicable transition,
      distinguishing configuration rollback, deployment action, code rollback and
      release rollback. *Evidence: runbook; `ADR-0006` section 7.3.*
- [ ] **AC-13** Rollback authority defined, and any difference from forward-change
      authority stated. *Evidence: ownership record.*
- [ ] **AC-14** No secret values recorded anywhere in the diff. *Evidence: the
      complete PR diff.*
- [ ] **AC-15** No invented propagation or rollback duration appears anywhere.
      *Evidence: the complete PR diff; every duration is either measured or
      marked as to-be-measured.*
- [ ] **AC-16** Timing rehearsal requirement explicit, with its four measurements
      and its execution boundary settled. *Evidence: section 3.8 as delivered.*
- [ ] **AC-17** Production activation remains unauthorized, and the document says
      so. *Evidence: the delivered document.*
- [ ] **AC-18** Broad CG-13 disposition explicit and classified A, B or C per
      section 12. *Evidence: `control-gaps.md` as delivered.*
- [ ] **AC-19** Monitoring and threshold work remains separately gated and is not
      absorbed. *Evidence: the delivered document; `ADR-0006` section 8.3.2.*
- [ ] **AC-20** `BE-02` remains unauthorized and untouched. *Evidence: the
      complete PR diff; PR #788 and issue #765 unchanged.*
- [ ] **AC-21** Guard mode remains `OFF` and the store remains unavailable in
      every environment. *Evidence: the deployment source, unchanged by this
      task.*
- [ ] **AC-22** The fleet-verification mechanism is specified but **not**
      implemented in this task's PR. *Evidence: the complete PR diff contains no
      runtime file.*

## 10. Required tests

### Unit / Integration / Authorization / Regression

Not applicable. The task changes no code.

### Documentation validation

```bash
git diff --check
```

Also verify:

- every relative Markdown link resolves;
- no placeholder field remains except the approval metadata the task template
  permits;
- no runtime, schema, migration, `Cargo` or workflow path appears in the diff;
- CG-13 is not marked globally resolved;
- `OBSERVE` and `ENFORCE` remain recorded as NOT AUTHORIZED;
- `BE-02` remains recorded as NOT AUTHORIZED;
- a `CHANGELOG.md` entry exists under `## [Unreleased]`;
- the docs-only CI classification and the exact-head CI result are recorded.

### Manual verification

- re-derive section 2.2.1, 2.2.2 and 2.2.3 against the task's own exact base
  before relying on them;
- confirm the production container command from the authoritative source;
- confirm that no environment's guard mode was changed.

### Performance

Not applicable.

## 11. Rollout

- **initial state after merge:** unchanged. Documentation only.

  ```text
  THOTH_GRAPHQL_MUTATION_GUARD_MODE = OFF
  loader store                      = unavailable
  production request acceptance     = unchanged
  ```

- **feature flag/configuration:** none introduced. The guard mode is the existing
  control and is not changed;
- **repository-managed deployment configuration:** this repository holds none. If
  the task discovers that it must touch any, the touched configuration must
  remain `OFF` and must require separate production activation authorization;
- **staging/preview validation:** the rehearsal is **defined** by this task and
  **executed** at the later preview/staging gate (section 3.8);
- **pilot:** not applicable. `OBSERVE` is itself the controlled pilot
  (`ADR-0006` section 7.2.2) and is not authorized by this task;
- **activation approval:** unchanged and still required. Completing this task
  does **not** authorize `OBSERVE`;
- **observation period:** not applicable to this task.

**The gates remaining after this task completes:**

```text
service-health signals / activation thresholds   (ADR-0006 section 8.3.2)
    -> preview/staging performance + timed rollback evidence
    -> explicit CTO OFF -> OBSERVE authorization
```

### 11.1 Monitoring boundary

This task is immediately followed by the separate gate "service-health signals
and activation thresholds verified". That work must **not** be absorbed here
unless an inseparable architectural dependency is discovered and escalated.

This task **may** state what telemetry is required to verify effective mode and
fleet consistency, because that is its own subject matter. It **must not** invent
service-health thresholds, latency or error-rate baselines, or availability SLOs.
Deriving and approving those from verified baselines remains a separate bounded
task or an explicitly separate phase.

## 12. Relationship to CG-13

The task must classify its result as exactly one of:

```text
A. feature-specific CG-13 subset satisfied;
   broad CG-13 remains open

B. evidence proves the same control genuinely resolves all of CG-13;
   propose broader closure for independent review / CTO decision

C. insufficient evidence;
   BLOCKED
```

**Default conservatively to A** unless evidence genuinely supports B.

CG-13 requires documentation of runtime, deployment, **migration execution**,
**rollback**, **restore verification** and approvers. This task addresses the
mutation-guard runtime-mode-control subset only. Migration execution, backup and
restore verification, and approver mapping for concerns other than this feature
are untouched by it, so B is unlikely on this task's evidence and must not be
claimed merely because the guard mode has been operationally mapped.

Recording the result must keep CG-13 **open**. The task may add a durable
reference to the bounded successor; it may not mark CG-13 resolved.

## 13. Stop conditions

The implementing agent must stop and report `BLOCKED` if:

- production runtime ownership cannot be identified authoritatively;
- production configuration authority cannot be identified;
- the effective mode cannot be verified across all serving instances, and no
  smallest separately reviewable mechanism can be specified that would achieve
  it;
- partial-fleet state cannot be detected;
- rollback cannot be demonstrated without unapproved production access;
- required evidence lives in an inaccessible external system;
- the task would require **secret retrieval** rather than metadata and ownership
  evidence;
- changing the mode would require an **architecture change** — as distinct from a
  bounded runtime-code change such as the section 2.2.3 entrypoint gap, which is
  a prerequisite rather than an architecture change and must be recorded as such;
- the actual runtime cannot support the approved `OFF` / `OBSERVE` / `ENFORCE`
  lifecycle;
- resolving the problem requires a different repository or a cross-programme
  architectural decision that is not yet approved — note that **specifying** a
  change owned by another repository is in scope, whereas **making** it is not;
- the task would have to change a guard mode, deploy, or otherwise act in
  production to answer a question;
- a migration, data change, schema change or public API change turns out to be
  required.

### 13.1 Known prerequisite that is not, by itself, a stop condition

Section 2.2.3 establishes that the `init` entrypoint silently ignores the mode
variable in a release build. If the production GraphQL API container runs `init`,
then **the mode is not currently controllable in production at all**, and a
bounded remediation is a hard prerequisite for `OBSERVE`.

That remediation is **not** an architecture change: the `OFF`/`OBSERVE`/`ENFORCE`
lifecycle approved by `ADR-0006` is intact, and the merged inert state is
correct and fail-safe. The task must therefore:

- record the finding, with its evidence and its production applicability;
- specify the remediation options it identifies — for example registering the
  guard argument on the `init` command, or setting an explicit container command
  — **without implementing any of them**, and without asserting which is correct
  before the owning repository has been consulted;
- record the remediation as a **separately specified, separately reviewed,
  separately authorized** prerequisite of `OFF -> OBSERVE`;
- require that the fleet-verification mechanism of section 3.5 be able to detect
  this exact failure, since its defining characteristic is that it is silent.

Recording it is required. Fixing it here is prohibited.

## 14. Expected implementation report

The agent must use
[`implementation-report-template.md`](../implementation-report-template.md) and
must record:

- exact base and head commits;
- actual files changed;
- the evidence class of every operational conclusion;
- the exact evidence that is missing, where any is;
- the CG-13 disposition and its justification;
- explicit confirmation that no production action, no mode change, no deployment
  and no secret retrieval occurred;
- explicit confirmation that `OBSERVE`, `ENFORCE` and `BE-02` remain
  unauthorized;
- CI status and the docs-only classification.

## 15. Recommended execution

Implementation model: Claude Opus, or the strongest available engineering model
Reasoning level: HIGH / maximum practical
Independent reviewer: ChatGPT GPT-5.6 Sol, or another independent model family
that did not author the implementation
Review reasoning level: HIGH

## 16. Branch and integration plan

- branch source: a freshly verified exact `develop` head;
- pull-request target: `develop`;
- expected merge order: after `THOTH-GQL-BATCH-01` (already merged) and before
  the service-health/threshold gate;
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
