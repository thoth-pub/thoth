# GraphQL Mutation-Guard Runtime Operations — Control Record

Status: ACTIVE
Owner: CTO
Task: [`THOTH-GQL-OPS-01`](../ai-delivery/tasks/THOTH-GQL-OPS-01.md)
Scope: the mutation-guard runtime-mode-control subset of
[CG-13](./control-gaps.md#cg-13---thoth-runtime-operations-unmapped) — and
nothing else
Subject of control: the single value
`THOTH_GRAPHQL_MUTATION_GUARD_MODE in { OFF, OBSERVE, ENFORCE }`

Authority condition: this record is repository-authoritative when this exact
content is reachable from the repository's authoritative integration branch.
Live review, authorization and merge evidence is the GitHub pull-request record.

**This record authorizes nothing.** It establishes how a mutation-guard mode
change *would* be controlled, and proves that the capability to make one and to
verify one does not currently exist. It performs no production action, changes
no production configuration, and grants no production authorization.

```text
CG-13 disposition (this feature subset):  C - insufficient operational
                                          capability/evidence; BLOCKED
ADR-0006 runtime-operations gate:         NOT SATISFIED
Blocking prerequisites:                   THOTH-GQL-OPS-02, THOTH-GQL-OPS-03
Earliest possible satisfaction:           THOTH-GQL-OPS-04, on evidence
OFF -> OBSERVE:                           NOT AUTHORIZED
OBSERVE -> ENFORCE:                       NOT AUTHORIZED
```

---

## 1. How to read this record

### 1.1 The three-state distinction, binding on every statement below

```text
merged develop state
    != deployed production release
    != production activation state
```

Conflating any two of these misdescribes the production service. Each is
established by different evidence, and none implies another.

### 1.2 Evidence classes, binding

Every operational conclusion in this record carries an evidence class:

```text
[REPO]            established by thoth-pub/thoth at the exact base
[EXTERNAL]        established by the authoritative deployment source
[REPO + EXTERNAL] established only by both together; neither class
                  establishes it alone, and it may not be attributed
                  to either one
[UNVERIFIED]      not established -- missing evidence is missing work
```

This repository can establish what a release *contains*. It cannot establish
which release an environment *runs*. Every statement about the deployed state of
an environment therefore carries `[REPO + EXTERNAL]` or `[UNVERIFIED]`, never
`[REPO]` alone.

### 1.3 Vocabulary, binding

| Term | Meaning |
|---|---|
| **pre-guard** | a release, image or environment whose binary contains no mutation guard. It has **no** guard mode — not `OFF`, not any other value |
| **guard-enabled candidate** | a build, image or release containing the merged `THOTH-GQL-BATCH-01` foundation. It has a guard mode, whose default is `OFF` |
| **guard-enabled environment** | an environment actually running a guard-enabled candidate |
| **activation** | an authorized `OFF -> OBSERVE` or `OBSERVE -> ENFORCE` transition of a guard-enabled environment. None exists |

### 1.4 The authoritative deployment source, and how it was read

The authoritative deployment configuration for the Thoth production and test
GraphQL API services is owned **outside this repository**, in the private
repository `thoth-pub/infrastructure`. That pointer is the whole of what this
repository records about that source.

**The source is secret-bearing.** Every read performed for this record was
**read-only**, **metadata-only** and **narrowly scoped** to the specific criterion
being satisfied, per `THOTH-GQL-OPS-01` section 2.2.5.

During that narrowly scoped read-only discovery, the implementing agent
encountered existing secret-bearing configuration in the private authoritative
deployment source. No secret value was copied into any repository file,
pull-request text, report, changelog, prompt or commit message; no credential was
used, changed or rotated.

**That encounter is a control/process exception requiring escalation, not an
acceptable routine read pattern.** The absence of onward copying limits its
consequence; it does not make the access itself acceptable.

**Binding on every successor task, and stricter than section 2.2.5.** An
implementing agent must **not** read secret-bearing production configuration
directly. External deployment facts must be obtained from a sanitized
metadata-only source that structurally cannot expose a secret value, or from
evidence supplied by an explicitly authorized **human/operator or control
owner** — or a sanitized artefact generated under that non-agent
human/operator's own control. **No AI agent or model is a valid evidence source
for this purpose**, in any role, family or session; evidence produced by an AI
agent that itself inspected production runtime or secret-bearing configuration
must be refused. If neither route is available the affected criterion is
**`BLOCKED`**. If secret material is nevertheless exposed to an implementing
agent, it must stop that source/read path immediately, report the exposure at the
minimum safe level, and perform no further read of that secret-bearing source for
the task.

**Control limitation, OPEN.** `THOTH-GQL-OPS-01` section 2.2.5 treats such an
incidental encounter as "not a breach" until the material is copied onward. The
stricter repository/project prohibition above governs successor execution. The
merged parent specification is not amended here — that requires its own explicit
authorization — and the conflict must be corrected before any successor requiring
secret-bearing production-source access is authorized. Owner: CTO / control
owner; not closable by an implementing agent.

The exposure remains a separate CTO-controlled security matter and is outside
this task's scope. No further characterisation of it is recorded here, and none
may be added by any successor task.

Deliberately **not** recorded here or in any successor task: credential values,
resource identifiers, account identifiers, production topology, platform or
orchestration detail, stack, cluster or service names, hostnames beyond those
already published in [`environments.md`](./environments.md), scaling parameters,
and every value of every environment variable.

---

## 2. Runtime ownership (specification section 3.1)

### 2.0 Execution capability is not runtime ownership

This distinction is binding on the whole of section 2, because collapsing it
would convert an access fact into an accountability claim that no evidence
supports.

```text
EXECUTION CAPABILITY
  who is technically able to make the change.
  Established from an access record.

ACCOUNTABLE RUNTIME OWNERSHIP
  who is designated as responsible for the production runtime, and is
  therefore the "explicit activation owner" release-gates.md section 5
  requires.
  Established only from a designation. NOT established here.
```

Holding write access to a configuration source establishes that a team **can**
apply a change. It does not establish that the team **owns** the runtime, and
this record does not treat the former as evidence of the latter.

### 2.1 Execution capability for a mode change

```text
Evidence source: access control on the authoritative deployment source
                 (organisation team permissions), read as metadata only
Evidence class:  [EXTERNAL]
Conclusion:      The role technically able to execute a guard-mode change is the
                 organisation's technical team -- the single team holding
                 write/maintain permission on the authoritative deployment
                 source. No role outside it can change the value, because
                 no other configuration source can set it (section 3).

                 This is EXECUTION CAPABILITY only, per section 2.0. It is a
                 role rather than an individual, and it is not offered as
                 evidence of accountable runtime ownership.
```

### 2.1.1 Accountable production runtime owner — NOT established

```text
Evidence source: no designation exists. environments.md section 3 still lists
                 "production deployment owners" among the controls required but
                 not held; control-gaps.md CG-13 is open on the same point; and
                 ADR-0006 section 7.2.1.1 records the owner as "not yet
                 identifiable, and that is itself an activation blocker"
Evidence class:  [UNVERIFIED]
Conclusion:      The authoritative, accountable production Thoth GraphQL runtime
                 owner is NOT established.

                 What is missing is a DESIGNATION, not an access record. No
                 amount of further reading of the deployment source can supply
                 it, because the fact is not recorded anywhere: it must be
                 decided and stated by the CTO.

                 Per release-gates.md section 5, which requires an explicit
                 activation owner, this remains an activation blocker.
                 THOTH-GQL-OPS-04 must obtain the designation.
```

This is why acceptance criterion **AC-1 is recorded as FAIL/BLOCKED** rather than
satisfied by section 2.1.

### 2.2 Request and approval authority

```text
Evidence source: ADR-0006 section 7.2.1.1; release-gates.md sections 5 and 8;
                 operating-model.md section 2.1
Evidence class:  [REPO]
Conclusion:      Using the repository's existing control terminology and
                 inventing no parallel vocabulary:

                 merge authorization                   -> CTO
                 OFF -> OBSERVE activation approval    -> CTO
                 OBSERVE -> ENFORCE activation approval -> CTO, separately
                 execution of an approved mode change  -> the technical team
                                                          of section 2.1
```

Any role may **request** a change; only the CTO may **approve** one; only the
technical team of section 2.1 is technically **able** to execute one.

### 2.3 Post-activation observation sign-off owner — NOT established

```text
Evidence source: ADR-0006 section 7.2.1.1, which defers this to the
                 runtime-operations control and records the owner as not yet
                 identifiable; release-gates.md section 8
Evidence class:  [UNVERIFIED]
Conclusion:      The post-activation observation sign-off owner is NOT
                 established.

                 This record PROPOSES the CTO, with the reasoning below. A
                 proposal is not a designation, and this record does not treat
                 its own derivation as established ownership.

                 THOTH-GQL-OPS-04 must obtain the CTO's explicit confirmation
                 (or a different designation). Until it does, this remains
                 missing work.
```

**The proposal, and the reasoning behind it**, recorded so the CTO has something
concrete to confirm or reject rather than an open question:
`release-gates.md` section 8 requires the observation period to end with an
explicit sign-off; `ADR-0006` section 8.3.3 makes that sign-off the decision that
gates `OBSERVE -> ENFORCE`; and `ADR-0006` section 7.2.1.1 already assigns that
activation approval to the CTO. Assigning the sign-off elsewhere would split one
decision across two owners. On that reasoning the technical team of section 2.1
would **prepare** the observation evidence and the CTO would **sign it off**.

This is a derivation from existing control terminology, not a reading of any
pre-existing ownership record — no such record exists. It is therefore why
acceptance criterion **AC-2 is recorded as FAIL/BLOCKED**: the criterion asks for
an *identified* owner, and a proposal awaiting confirmation is not one.

### 2.4 Summary of what section 2 does and does not establish

| Question | Status | Class |
|---|---|---|
| who is technically able to execute a mode change | **established** | `[EXTERNAL]` |
| who approves merge and each activation | **established** — CTO | `[REPO]` |
| who may request a change | **established** — any role | `[REPO]` |
| **accountable production runtime owner** | **NOT established** | `[UNVERIFIED]` |
| **post-activation observation sign-off owner** | **NOT established** (proposed only) | `[UNVERIFIED]` |
| **whether rollback needs CTO approval** | **NOT established** (section 9.3) | `[UNVERIFIED]` |

The broader Thoth runtime ownership question also remains open under CG-13 and is
untouched here.

---

## 3. Configuration authority (specification section 3.2)

### 3.1 Where the value enters the process

```text
Evidence source: src/bin/arguments/mod.rs, `mutation_guard_mode()`
Evidence class:  [REPO]
Conclusion:      A clap argument with `Arg::env("THOTH_GRAPHQL_MUTATION_GUARD_MODE")`,
                 `default_value("OFF")` and
                 `value_parser(["OFF", "OBSERVE", "ENFORCE"])`.
                 It is a process-start configuration input and nothing more.
```

### 3.2 Which configuration source is authoritative, and precedence

```text
Evidence source: the authoritative deployment source's generic service template,
                 read for structural keys only
Evidence class:  [EXTERNAL]
Conclusion:      The container environment is built exclusively from numbered
                 environment-variable parameters supplied by the service
                 definition. That template parameter set is the SOLE source of
                 container environment for this service. No secret-store
                 reference mechanism and no environment-file mechanism exists in
                 the template, so no competing source can set the value and no
                 precedence conflict is possible at the deployment layer.
```

```text
Evidence source: src/bin/thoth.rs (`dotenv::dotenv().ok()`); pinned dotenv 0.15.0
                 `src/iter.rs` `load()`; Dockerfile (`FROM scratch`)
Evidence class:  [REPO]
Conclusion:      The application also calls `dotenv()`, which loads a `.env` file
                 if present. Pinned dotenv sets a variable ONLY when it is not
                 already set (`if env::var(&key).is_err()`), so it can never
                 override the container environment. The production image is
                 built `FROM scratch` and contains only the binary and CA
                 certificates, so no `.env` file exists in it.

                 Precedence, settled:
                   container environment (deployment source)  WINS
                   `.env` via dotenv                          cannot override,
                                                              and cannot exist
                                                              in the production
                                                              image
                   clap default `OFF`                         applies only when
                                                              neither supplies a
                                                              value
```

### 3.3 Is the value currently configured anywhere?

```text
Evidence source: narrowly scoped presence check of the production and test
                 Thoth GraphQL API service definitions in the authoritative
                 deployment source
Evidence class:  [EXTERNAL]
Conclusion:      `THOTH_GRAPHQL_MUTATION_GUARD_MODE` is ABSENT from both the
                 production and the test service definitions. It is configured
                 nowhere.
```

```text
Evidence source: repository-wide search at the exact base
Evidence class:  [REPO]
Conclusion:      The variable name appears only in `src/bin/arguments/mod.rs`.
                 It is set by no workflow, no compose file and no Dockerfile in
                 this repository.
```

### 3.4 The command the production GraphQL API container actually runs

This is load-bearing: a guard-enabled release executing through `init` cannot
consume the variable at all (section 4).

```text
Evidence source: Dockerfile
Evidence class:  [REPO]
Conclusion:      ENTRYPOINT ["/thoth"], CMD ["init"], built with
                 `cargo build --release`.
```

```text
Evidence source: the production Thoth GraphQL API service definition in the
                 authoritative deployment source; and the generic service
                 template's command-construction logic, both read for structural
                 keys only
Evidence class:  [EXTERNAL]
Conclusion:      The production GraphQL API service supplies NO container
                 command override. The template resolves an unsupplied command
                 to "no value", so the container runs the image default `init`.

                 CONFIRMED, not refuted: the THOTH-GQL-OPS-01 section 2.2.3
                 finding holds at this task's execution time.

                 The override mechanism EXISTS and is exercised by a different
                 Thoth service defined in the same file, which does supply an
                 explicit command. The GraphQL API's inheritance of `init` is
                 therefore a deliberate configuration state, not a template
                 limitation.
```

### 3.5 Which release each environment is actually running

```text
Evidence source: the release tag `v1.6.1` and `origin/master` in this repository
Evidence class:  [REPO]
Conclusion:      The most recent release tag contains no `MutationGuardMode`,
                 defines no `mutation_guard_mode()` CLI argument and has no
                 mutation-guard wiring on its GraphQL startup path. The release
                 line `master` likewise contains none.

                 Release-code state [REPO]: the relevant release/master code
                 contains no mutation guard and is PRE-GUARD.

                 This is the WHOLE of what this repository establishes. It says
                 nothing about which release any environment runs.
```

```text
Evidence source: the version parameter of the production and test Thoth GraphQL
                 API service definitions in the authoritative deployment source,
                 read as a single scoped key
Evidence class:  [EXTERNAL]
Conclusion:      Both the production and the test GraphQL API services declare
                 the same release version, and it is the pre-guard release
                 identified above. The exact value is not recorded here.
```

```text
Evidence class:  [REPO + EXTERNAL]
Conclusion:      Production is PRE-GUARD. The test environment is ALSO PRE-GUARD.

                 Neither has a guard mode at all. Neither may be described as
                 running `MutationGuardMode::OFF`.

                 Merging THOTH-GQL-BATCH-01 deployed nothing and activated
                 nothing. The merged foundation is inert AND undeployed; those
                 are two separate facts established by two different evidence
                 classes, and neither implies the other.
```

```text
Evidence class:  [UNVERIFIED]
Conclusion:      Whether the RUNNING service matches its declared definition --
                 that is, whether configuration drift exists between the
                 authoritative deployment source and the live orchestrator --
                 is NOT established. Establishing it requires a live orchestrator
                 read, which this task is not authorized to perform. This is the
                 same evidence that capability gap 2 (section 7) exists to
                 supply.
```

**Consequence for the rehearsal of section 10.** Because the test environment is
also pre-guard, the rehearsal cannot begin by changing a mode there: it must
first deploy a guard-enabled candidate to that environment. That is an additional
prerequisite the rehearsal specification must carry.

**Actor, binding.** That deployment — and any mode transition, mixed-fleet
creation or rollback in a real environment, production or not — is performed by
an **AUTHORIZED NON-AGENT DEPLOYMENT ACTOR**, which has exactly two permitted
forms:

```text
FORM 1 -- an AUTHORIZED HUMAN / OPERATOR acting under the relevant
          authorization;

FORM 2 -- EXISTING DEPLOYMENT AUTOMATION OR INFRASTRUCTURE that executes
          under that authorized human/operator's own control or
          initiation.

There is no third form, and NO AI AGENT OR MODEL qualifies -- in any
role, family or session, and whether or not it is separately or
independently controlled.

No AI agent or model may initiate, trigger, dispatch or execute a
deployment; perform a real-environment mode transition; create,
manipulate or restore real fleet state; execute a real-environment
rollback; use deployment credentials; or substitute for the authorized
human/operator by invoking deployment automation.

An automated deployment system is an EXECUTION MECHANISM, not an AI-agent
delegation: it must be initiated and controlled by the authorized
non-agent operator under the relevant authorization.
```

The prohibition is not limited to the implementing agent of any one task; it
binds every assisting, reviewing, orchestrating, supervising, delegated or
sub-agent model equally. An AI agent may specify what is required, evaluate the
sanitized evidence returned and record the outcome. Where no authorized
non-agent deployment actor or its evidence is available, the dependent criterion
is **`BLOCKED`** — an AI agent stepping in is not an available actor.

Ordinary local, disposable and CI repository testing by an implementing agent is
unaffected by this paragraph, which governs real operational environments and
actions only.

### 3.6 Behaviour for an absent value

```text
Evidence source: src/bin/arguments/mod.rs; reproduced in an isolated probe
                 (section 4.3)
Evidence class:  [REPO]
Conclusion:      On the guard-registering path, clap supplies the declared
                 default `OFF`. On the `init` path the value is absent from the
                 matches and the handler's `unwrap_or("OFF")` yields `OFF`.
                 Both routes yield OFF. The default is fail-safe.
```

### 3.7 Behaviour for an invalid value — and a silent-failure finding

```text
Evidence source: src/bin/arguments/mod.rs `value_parser`; reproduced in an
                 isolated probe (section 4.3)
Evidence class:  [REPO]
Conclusion:      On `start graphql-api` -- the path that registers the argument --
                 an invalid value is rejected by clap and the process exits with
                 a usage error rather than starting in an unintended mode.

                 On `init` -- the production-applicable path -- an invalid value
                 is NOT rejected. The `value_parser` never runs, because the
                 argument is not registered. The process starts normally with
                 effective mode `OFF`.
```

This refines the `THOTH-GQL-OPS-01` section 3.2 item 7 expectation rather than
contradicting it: the "process fails to start" conclusion holds **only** on the
guard-registering path. On the production-applicable path an invalid value is a
third instance of the same silent-ignore class as section 4, and
`THOTH-GQL-OPS-02` must eliminate it together with the others.

```text
Evidence source: the generic service template's deployment configuration,
                 read for structural keys only
Evidence class:  [EXTERNAL]
Conclusion:      The service is deployed with a deployment circuit breaker
                 enabled and automatic rollback enabled. A task that fails to
                 start therefore causes the deployment to be rolled back
                 automatically to the previous task-definition revision.

                 This is a genuine fail-safe, but it protects only the path on
                 which an invalid value actually fails. It cannot detect a value
                 that is silently ignored, because such a task starts healthily.
                 It also cannot detect a mode that is set correctly in
                 configuration but not adopted by the process.
```

### 3.8 Request and execution authority

```text
Evidence source: sections 2.1 and 2.2 of this record
Evidence class:  [REPO + EXTERNAL]
Conclusion:      Request: any role.
                 Approve: CTO, separately for each transition.
                 Execute: the technical team holding write/maintain permission
                          on the authoritative deployment source -- stated as
                          EXECUTION CAPABILITY, not as accountable runtime
                          ownership (section 2.0).

                 NOT established here: the accountable production runtime owner
                 (section 2.1.1) and the observation sign-off owner
                 (section 2.3), both [UNVERIFIED].
```

---

## 4. The mode-control code path, re-derived (specification section 3.3)

Every finding below was independently re-derived against the exact authorized
base rather than taken from the specification's narrative.

### 4.1 Where the mode is read, and when

| Question | Finding | Evidence | Class |
|---|---|---|---|
| where the value enters | `clap` argument with `Arg::env(..)`, default `OFF`, parser restricted to the three modes | `src/bin/arguments/mod.rs` | `[REPO]` |
| where it is parsed | once, in the `start graphql-api` handler, into `MutationGuardMode` | `src/bin/commands/start.rs`, `graphql_api()` | `[REPO]` |
| where it is stored | captured by the `move` closure passed to `HttpServer::new`, registered as `app_data(Data::new(mutation_guard_mode))` | `thoth-api-server/src/lib.rs`, `start_server()` | `[REPO]` |
| when it is read | **once, at process start**, before the HTTP server binds | same | `[REPO]` |
| is there a reload path | **no.** No signal handler, no watcher, no admin route, no re-read anywhere in the workspace | absence verified across `src/bin/` and `thoth-api-server/src/` | `[REPO]` |

```text
Evidence class: [REPO]
Conclusion:     Changing the effective mode of a RUNNING process is impossible.
                The mode changes only by starting a new process with a different
                environment. Any claim that the mode can be changed "without a
                restart" is false for this codebase.
```

### 4.2 The effective mode is not observable on a running instance

| Surface | Exposes the mode? | Evidence | Class |
|---|---|---|---|
| `GET /` (`ApiConfig`) | **no** — `api_name`, `api_version`, `api_schema`, `public_url`, `schema_explorer_url` only | `thoth-api-server/src/lib.rs` | `[REPO]` |
| `GET /graphiql`, `GET /graphql`, `GET /schema.graphql`, `POST /graphql` | **no** | the complete route set | `[REPO]` |
| startup logging | **no** — no log call records the effective mode at start | absence verified | `[REPO]` |
| guard events | only on a **collision**, and only in `OBSERVE`/`ENFORCE` | `thoth-api/src/graphql/mutation_guard.rs` | `[REPO]` |

```text
Evidence class: [REPO]
Conclusion:     There is today NO mechanism that proves the effective mode of a
                serving instance. `OFF` and `OBSERVE` are externally
                indistinguishable: OBSERVE never rejects, and its only output is
                a server-side event emitted solely when a colliding document
                happens to arrive.

                This is CAPABILITY GAP 2 (section 7).
```

### 4.3 The `init` entrypoint does not accept the mode — re-proven

This is a finding about the **guard-enabled** code merged to `develop`. It
describes what the deployment path would do with a guard-enabled release. It is
**not** a description of the pre-guard binary currently deployed.

1. `Dockerfile`: `CMD ["init"]`, built `cargo build --release`. `[REPO]`
2. `src/bin/commands/mod.rs`, `INIT`: registers `database`, `host`, `port`,
   `threads`, `keep_alive`, `gql_url`, `key`, `zitadel_url` and the three AWS
   arguments — and **not** `arguments::mutation_guard_mode()`. `[REPO]`
3. `src/bin/thoth.rs`: `init` dispatches into the **same** handler as
   `start graphql-api`, passing its own `ArgMatches`. `[REPO]`
4. `src/bin/commands/start.rs`: the handler reads
   `get_one::<String>("mutation-guard-mode") … .unwrap_or("OFF")`. `[REPO]`
5. Pinned `clap_builder` 4.6.0, chain re-derived from the vendored source:
   `get_one` → `MatchesError::unwrap(id, try_get_one(id))` → `try_get_arg_t` →
   `try_get_arg` → `verify_arg`. `verify_arg` returns
   `Err(MatchesError::UnknownArgument)` **only under `cfg(debug_assertions)`**;
   otherwise it returns `Ok(())` and `self.args.get(id)` yields `None`.
   Therefore:
   - **release build** — `get_one` returns `None` and `unwrap_or("OFF")` yields
     `OFF`;
   - **debug build** — `MatchesError::unwrap` panics with
     `Mismatch between definition and access of 'mutation-guard-mode'`. `[REPO]`
6. Both branches were **reproduced**, not merely read, in an isolated throwaway
   probe outside this repository mirroring the exact argument definition and the
   exact access pattern. No repository code was built, modified or added for that
   reproduction. Results:

   ```text
   RELEASE  init  + ENFORCE  -> effective mode = OFF
   RELEASE  init  + OBSERVE  -> effective mode = OFF
   RELEASE  init  + unset    -> effective mode = OFF
   RELEASE  init  + invalid  -> effective mode = OFF, exit 0
   RELEASE  start + ENFORCE  -> effective mode = ENFORCE
   RELEASE  start + OBSERVE  -> effective mode = OBSERVE
   RELEASE  start + unset    -> effective mode = OFF
   RELEASE  start + invalid  -> usage error, exit 2
   DEBUG    init  + ENFORCE  -> panic, exit 101
   DEBUG    start + ENFORCE  -> effective mode = ENFORCE
   ```

   The `Dockerfile` builds `--release`, so the shipped image takes the **release**
   branch. `[REPO]`

```text
Evidence class: [REPO]
Conclusion:     CONFIRMED, not refuted.

                When a GUARD-ENABLED container runs the image default command
                `init`, THOTH_GRAPHQL_MUTATION_GUARD_MODE is silently ignored and
                that process's effective mode is unconditionally OFF.

                Guard-enabled `start graphql-api` CAN consume the variable.
                Guard-enabled `init` currently does NOT register the argument.
```

The failure is **silent** and **fail-safe**. It cannot cause an unintended
activation, and it is not a production incident — production is not running
guard-enabled code at all (section 3.5), and `OFF` is in any case the intended
state of a guard-enabled candidate. But it means setting the environment variable
would appear to succeed while changing nothing, which is precisely the class of
failure the fleet-verification requirement of section 7 exists to detect.

```text
Evidence class: [REPO + EXTERNAL]
Conclusion:     CAPABILITY GAP 1, stated as a property of the DEPLOYMENT PATH
                and not of the deployed binary:

                Under the currently authoritative production deployment command
                path, a guard-enabled release containing the merged foundation
                would execute through `init`.

                Until THOTH-GQL-OPS-02 is delivered, that path cannot consume
                THOTH_GRAPHQL_MUTATION_GUARD_MODE.

                Therefore an OFF -> OBSERVE transition of a guard-enabled
                candidate is not operationally PERFORMABLE through the current
                deployment path -- not merely unauthorized.
```

### 4.4 `init` is not interchangeable with `start graphql-api` — the migration boundary

```text
Evidence source: src/bin/thoth.rs
Evidence class:  [REPO]
Conclusion:
```

```rust
Some(("init", arguments)) => {
    commands::run_migrations(arguments)?;
    commands::start::graphql_api(arguments)
}
```

| Command | Runs database migrations | Starts the GraphQL API |
|---|---|---|
| `init` | **yes**, first, and aborts on failure | yes, only if migrations succeeded |
| `start graphql-api` | **no** | yes |

The `Dockerfile` states the same intent directly: "By default run `thoth init`
(runs migrations and starts the server on port 8080)".

**Binding classification, reproduced wherever a command override is mentioned at
all:**

```text
An explicit production command override is NOT an interchangeable
feature-local fix. It changes the current `init` execution path by removing
migration execution from deployment, and therefore requires separate
migration/deployment-control analysis and approval under the broader CG-13
migration/deployment problem.
```

Consequences, binding:

- an override is **out of bounded scope** for `THOTH-GQL-OPS-01` and for
  `THOTH-GQL-OPS-02`;
- it must not be offered as an option, a fallback, a "simpler alternative" or an
  expedient anywhere in this family of tasks;
- if evidence later shows that only an override can work, that is an escalation
  to the CTO under the migration/deployment half of CG-13 — not a decision either
  task may take;
- **no production command or configuration change is authorized** by this record.

The feature-local `THOTH-GQL-OPS-02` remediation class must preserve **all**
current `init` migration and startup semantics. This record does **not** select
the `THOTH-GQL-OPS-02` remediation mechanism.

---

## 5. Restart and redeployment semantics (specification section 3.3)

The two evidence classes are kept separate, because they answer different
questions.

### 5.1 What the application requires

```text
Evidence source: thoth-api-server/src/lib.rs; src/bin/commands/start.rs;
                 absence of any reload path across the workspace
Evidence class:  [REPO]
Conclusion:      The value is read ONCE at process start and there is no reload
                 path of any kind. A NEW PROCESS is therefore necessary in every
                 case. This is a lower bound and is not in question.
```

### 5.2 What the deployment control system requires

```text
Evidence source: the authoritative deployment source's documented update
                 procedure and its generic service template, read as metadata
Evidence class:  [EXTERNAL]
Conclusion:      Container environment is a property of the deployed service
                 definition, which is generated from the infrastructure-as-code
                 templates. Changing the value therefore requires:

                   1. an edit to the service definition in the private
                      authoritative source;
                   2. that change committed and pushed -- the deployment tooling
                      refuses to deploy unless the local branch is in sync with
                      its upstream;
                   3. an infrastructure-as-code stack update, which produces a
                      new service-definition revision;
                   4. replacement of the running tasks by the orchestrator so
                      that new processes start under the new definition.

                 A separate forced-redeployment path exists that replaces tasks
                 without a template change. It restarts processes under the
                 CURRENT definition and therefore cannot itself change the mode.

                 Changing the mode does NOT require publishing a new image or
                 cutting a new release: the image version and the environment
                 are independent parameters of the same service definition.
```

### 5.3 The plain statement the specification requires

```text
Evidence class: [REPO + EXTERNAL]
Conclusion:     A mode change is BOTH a configuration change AND a deployment.

                It is a configuration change because the value lives in
                deployment configuration rather than in the image.
                It is a deployment because the application cannot adopt a new
                value without a new process, and the control system starts new
                processes only by revising the service definition and replacing
                tasks.

                The withdrawn claim that the change is "a configuration change
                without a deploy" (ADR-0006 sections 7.2.4 and 7.3) is NOT
                resurrected. Current evidence positively contradicts it.
```

### 5.4 Timing

```text
Evidence class: [UNVERIFIED]
Conclusion:     No authoritative propagation, replacement or rollback interval
                is established, and none is invented here. Every such duration
                is TO BE MEASURED in the downstream rehearsal of section 10.
```

---

## 6. Propagation and the expected fleet (specification section 3.4)

### 6.1 How a change reaches serving instances

```text
Evidence source: the generic service template's deployment configuration and
                 autoscaling resources, read for structural keys only
Evidence class:  [EXTERNAL]
Conclusion:      The service is deployed by ROLLING REPLACEMENT. The deployment
                 configuration permits the running count to exceed the desired
                 count during a rollout and permits it to fall below it, so old
                 and new tasks serve the same load balancer CONCURRENTLY while a
                 rollout is in progress.

                 A mixed-mode window is therefore STRUCTURALLY GUARANTEED, not
                 exceptional. An atomic fleet-wide mode change is not available.
                 ADR-0006 section 7.2.4's alternative applies: rollout semantics
                 and verification must make mixed-mode periods safe and bounded.
```

### 6.2 The expected fleet is a range, not a number

```text
Evidence source: the autoscaling resources and their target-tracking policies in
                 the generic service template, and the autoscaling parameters of
                 the production GraphQL API service definition, read for
                 structural keys only
Evidence class:  [EXTERNAL]
Conclusion:      Autoscaling is ENABLED for the production GraphQL API service.
                 Minimum capacity is the desired count; maximum capacity is
                 substantially higher. Scaling is target-tracking on processor
                 utilisation, memory utilisation and per-target request count,
                 each with its own scale-out and scale-in cooldown.

                 The expected replica population is therefore a RANGE WITH A LIVE
                 CURRENT VALUE. It cannot be a static number copied into this
                 repository, and no static number is recorded here.
```

### 6.3 The decidable predicate

```text
Evidence class: [REPO + EXTERNAL] for the definition
Conclusion:     "All replicas carry the intended mode" is made decidable as:

                1. at the moment the change is applied, read the LIVE desired
                   and running instance counts from the orchestrator. This is
                   the expected population E;
                2. pin the transition window and record E and every subsequent
                   scaling event within it. Autoscaling may change E DURING the
                   transition, so E is re-read rather than assumed constant;
                3. enumerate the ACTUAL running instance set from the
                   orchestrator, not from a sample of traffic;
                4. establish the EFFECTIVE mode of each enumerated instance
                   using the section 7 mechanism;
                5. propagation is COMPLETE when, for a re-read E, every
                   enumerated instance reports the intended effective mode and
                   no instance reports any other mode.

                A deployment reporting success is evidence that the deployment
                finished. It is NOT evidence that every serving instance carries
                the intended mode, and it must never be substituted for step 5.
```

### 6.4 Instances started during a transition

```text
Evidence source: the orchestrator's service-definition revision model, as
                 expressed in the deployment source's template
Evidence class:  [EXTERNAL], partial
Conclusion:      A new task started after a new service-definition revision has
                 become current -- whether by autoscaling scale-out, by task
                 replacement, or by health-check-driven restart -- starts under
                 that current revision and therefore in the NEW mode.

                 A new task started while the rollout is still in progress may
                 start under EITHER revision, because both are live during the
                 window. The transition procedure must therefore treat any
                 instance started during the window as unknown-mode until the
                 section 7 mechanism has attributed a mode to it.
```

```text
Evidence class: [UNVERIFIED]
Conclusion:     The LIVE current value of E is not established by this record.
                Obtaining it requires a live orchestrator read, which this task
                is not authorized to perform. THOTH-GQL-OPS-04 must obtain it,
                and the transition procedure of the runbook re-reads it at
                execution time.
```

### 6.5 Propagation duration

```text
Evidence class: [UNVERIFIED]
Conclusion:     No propagation interval is established, and none is invented.
                It is TO BE MEASURED in the downstream rehearsal of section 10,
                and the measured value becomes the recorded expectation.
```

---

## 7. Fleet-verification mechanism — specified, not implemented (specification section 3.5)

```text
Evidence source: section 4.2 of this record
Evidence class:  [REPO]
Conclusion:      No compliant mechanism exists today.
```

**Binding distinction, which this whole record turns on:**

```text
a specification for a verifier   !=   a verifier
a verifier                       !=   a verified fleet
```

Delivering this section means a mechanism has been **defined**. It does **not**
mean the effective mode of any instance has been established, and it must never
be recorded, summarised or reported as though it did.

### 7.1 Why a public endpoint cannot, on its own, satisfy this

Serving instances sit behind a shared load balancer, so a client cannot address
an individual replica. A sample that returns the new mode proves only that **at
least one** instance carries it. Polling a public HTTP endpoint therefore cannot
prove fleet-wide consistency, and cannot detect a mixed fleet.

Additionally, the guard mode describes a server-side request-acceptance policy.
Publishing it on an unauthenticated public surface is an **information-disclosure
decision**, not a formatting choice: it tells an unauthenticated caller whether
duplicate top-level mutation response keys are currently rejected, which is
reconnaissance value for probing request-acceptance behaviour. It must not be
selected casually. If `THOTH-GQL-OPS-03` proposes any public surface, its
specification must make the disclosure implications explicit and obtain an
explicit decision; this record does not grant one.

### 7.2 Required properties of a compliant mechanism

A compliant mechanism must:

1. prove **effective** mode — the mode the process actually computed — and not
   configured intent. Section 4.3 is why this is binding: intent and effect
   diverge silently;
2. distinguish `OFF`, `OBSERVE` and `ENFORCE`;
3. **attribute** an observed mode to a specific, identified serving instance, so
   that "some replicas are on the new mode" and "all replicas are on the new
   mode" are distinguishable;
4. **detect mixed-mode fleets**;
5. **enumerate or otherwise completely cover** the actual serving population,
   derived from the orchestrator's live state rather than from a sample of
   traffic or a static count;
6. detect the section 4.3 failure class specifically — a configured mode that the
   process silently did not adopt — since that failure is otherwise invisible;
7. expose **no** publisher or user data;
8. expose **no** secret;
9. make **no** change to request acceptance;
10. make **no** change to guard, batching or store semantics.

### 7.3 What this record does not do

This record **specifies** the mechanism and does **not** implement it.
Implementation belongs to [`THOTH-GQL-OPS-03`](../ai-delivery/tasks/THOTH-GQL-OPS-03.md)
and is `NOT AUTHORIZED`. No runtime observability is added by this record's pull
request, here or anywhere else.

This record also does **not** select the mechanism's implementation form. That
selection belongs to `THOTH-GQL-OPS-03`'s own approved specification.

---

## 8. Partial-fleet handling (specification section 3.6)

A partial fleet must **never** be treated as successful activation.

### 8.1 The three mixed windows, analysed separately

```text
Evidence source: ADR-0006 sections 4.12.6.6 and 7.2.4; section 6.1 of this record
Evidence class:  [REPO + EXTERNAL]
```

| Mixed window | Client-visible acceptance | Store availability | Assessment |
|---|---|---|---|
| `OFF` + `OBSERVE` | **identical** — neither rejects | unavailable in both | **Observation gap, not an acceptance defect.** Relatively benign. The cost is under-counted observation evidence during the window: instances still in `OFF` evaluate nothing and emit nothing, so the would-be-rejection count understates real traffic. The observation record must account for the window explicitly rather than treat the count as complete |
| `OBSERVE` + `ENFORCE` | **differs** — the same document is accepted or rejected depending on which instance serves it | differs | **Request-acceptance inconsistency.** Not tolerable as an indefinite state. A client retrying an identical request sees non-deterministic acceptance. Must be bounded, detected and evidenced |
| `OFF` + `ENFORCE` | **differs** | differs | as above |

The distinction between the first row and the other two is binding and must not
be flattened: the first is an **evidence-completeness** problem, the others are
**client-visible correctness** problems.

### 8.2 Detection, abort, rollback and recovery

Defined for each transition. Every check uses the section 7 mechanism, which does
not yet exist.

| | `OFF -> OBSERVE` | `OBSERVE -> ENFORCE` (future) |
|---|---|---|
| **Detection** | enumerate the live instance set and establish each instance's effective mode; a fleet is mixed when two enumerated instances report different modes, or when any instance cannot be attributed a mode | as left |
| **Abort criteria** | the fleet remains mixed beyond the measured propagation bound established in section 10; or any instance reports a mode that is neither the previous nor the intended mode; or the instance set cannot be completely enumerated | as left, **plus** any legitimate-client rejection observed during the window |
| **Rollback trigger** | abort criteria met; or a service-health regression attributable to the change; the observation-gap window alone is **not** a rollback trigger, because acceptance is unchanged | abort criteria met; **or any legitimate-client rejection**, which makes rollback mandatory rather than optional |
| **Rollback authority** | section 9.3 | section 9.3 |
| **Evidence required after recovery** | the enumerated instance set re-read after recovery; the effective mode of every member; a statement that no member reports the aborted mode; the elapsed time; the observation record annotated with the mixed window so the evidence is not read as complete | as left, **plus** confirmation that no client-visible rejection persists |

### 8.3 Bounding the window

```text
Evidence class: [UNVERIFIED]
Conclusion:     No numeric bound on a mixed-mode window is established, and none
                is invented. The bound is TO BE MEASURED in the downstream
                rehearsal of section 10 and approved before activation.
```

---

## 9. Rollback (specification section 3.7)

### 9.1 Four different things, which must not be conflated

| # | Thing | What it is | Does it fix a live wrong mode? |
|---|---|---|---|
| 1 | **configuration rollback** | restoring the previous mode value in the authoritative deployment source | **Not on its own.** The value must then take effect |
| 2 | **deployment/restart action** | the stack update and task replacement that start new processes under the restored value | **Yes** — and section 5.1 establishes it is **always** required |
| 3 | **code rollback** | reverting the merge commit | **No.** The merged state is `guard OFF, store unavailable`, so a code revert is a no-op for production behaviour, and reverting code without replacing processes changes nothing on a running fleet |
| 4 | **production release rollback** | returning to a previously released image version | Only incidentally, and it is a heavier action with its own migration considerations. It is not the mode control |

```text
Evidence class: [REPO] for rows 1, 3; [EXTERNAL] for row 2;
                [REPO + EXTERNAL] for row 4
Conclusion:     Operational rollback of a mode is rows 1 AND 2 together.
                A code revert alone does NOT correct a live process
                configuration state, and this record does not claim it does.
```

### 9.2 The transitions

```text
OBSERVE -> OFF          restore the previous value, then replace tasks
                        (rows 1 + 2). Stops evaluation entirely. Restores the
                        prior request-acceptance behaviour, which OBSERVE had
                        not changed. Store remains unavailable throughout.

ENFORCE -> OBSERVE      as above. Stops rejecting; keeps collecting evidence.
(future)                Store becomes unavailable, so no path is left depending
                        on a guarantee no longer enforced.

ENFORCE -> OFF          as above. Stops evaluation entirely.
(future)
```

### 9.3 Rollback authority, stated plainly

Three separate questions, deliberately not collapsed. Sharing a technical
mechanism establishes nothing about timing and nothing about authorization.

```text
Evidence source: sections 2.1 and 2.2 of this record
Evidence class:  [REPO + EXTERNAL]
Conclusion:      ESTABLISHED -- technical execution mechanism only.

                 Rollback uses the same configuration/deployment mechanism as a
                 forward transition -- an edit to a private repository, a push,
                 a stack update and a full task replacement -- and is
                 technically executed by the same execution-capability team.

                 This is a statement about HOW the change is applied. It is not
                 a statement about how long it takes, and it is not a statement
                 about who must approve it.
```

```text
Evidence class: [UNVERIFIED]
Conclusion:      Actual rollback latency/duration remains [UNVERIFIED].
                 See section 9.4.
```

```text
Evidence class: [UNVERIFIED]
Conclusion:      Whether rollback ADDITIONALLY requires CTO approval remains
                 [UNVERIFIED].

                 No authorization equivalence is inferred from sharing the
                 technical mechanism. That two operations are applied the same
                 way says nothing about whether they need the same approval,
                 and this record does not treat the former as evidence of the
                 latter.

                 THOTH-GQL-OPS-04 must obtain an explicit CTO decision on
                 whether an operational rollback may be executed on the
                 execution-capability team's own authority. This record does not
                 make that decision.
```

This is why acceptance criterion **AC-13 is recorded as FAIL/BLOCKED**: the
criterion asks for rollback authority to be *defined*, and only the execution
mechanism is established.

```text
Evidence class: [REPO + EXTERNAL]
Conclusion:     THIS IS NOT A KILL SWITCH.

                A rollback requires an edit to a private repository, a push, a
                stack update and a full task replacement. It must not be
                described as immediate, as deploy-free, or as a kill switch
                anywhere.

                Note what is NOT claimed here: no expedited path is known to
                exist, but neither its absence nor its presence is established
                as an authorization fact -- only the technical mechanism above
                is.
```

### 9.4 Duration

```text
Evidence class: [UNVERIFIED]
Conclusion:     Actual rollback latency/duration remains [UNVERIFIED] and must be
                measured at the downstream preview/staging rehearsal.

                It is NOT inferred from the forward transition. Sharing a
                mechanism is not sharing a measured time: the two differ in the
                state each starts from, and neither has been timed. No duration
                is recorded here and none is invented.
```

---

## 10. Rehearsal — defined here, executed at a downstream gate (specification section 3.8)

**This record does not execute the rehearsal, and executing it is not authorized.**
The rehearsal requires the section 7 mechanism to exist, and that mechanism is
delivered by `THOTH-GQL-OPS-03`.

**Owner: the downstream preview/staging gate — not `THOTH-GQL-OPS-04`.** The
repository-authoritative sequence places the rehearsal *after* the
runtime-operations gate and *after* the service-health/threshold gate:

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

`THOTH-GQL-OPS-04` proves the mode-control and fleet-verification capabilities
**operate**; this rehearsal measures how **long** they take. Proving operation is
not measuring timing, and `THOTH-GQL-OPS-04` must not absorb this gate.

### 10.1 Prerequisites before the rehearsal can be attempted

1. `THOTH-GQL-OPS-02` implemented, independently reviewed and merged;
2. `THOTH-GQL-OPS-03` implemented, independently reviewed and merged;
3. `THOTH-GQL-OPS-04` merged, so the runtime-operations gate is satisfied;
4. the service-health/threshold gate passed (`ADR-0006` section 8.3.2);
5. a **guard-enabled candidate deployed to the non-production environment** —
   required because section 3.5 establishes that the test environment is
   currently **pre-guard**, so there is no mode there to change;
6. the rehearsal itself separately authorized.

### 10.2 Required measurements

```text
time to apply mode change
time to verify fleet consistency
time to rollback
time to verify rollback
```

Each is measured using the section 7 mechanism, so that "verified" means verified
**effect** and not a deployment reported as complete.

These four measurements are owned by **this downstream gate**. They are not
`THOTH-GQL-OPS-04`'s to produce or to claim, and populating the runbook's timing
fields from them is not a precondition of satisfying the runtime-operations gate.

### 10.3 Required observations

- the rehearsal must deliberately **observe a partial-fleet period at least
  once**, so that section 8's detection is proven rather than asserted. Section
  6.1 establishes that rolling replacement makes such a period structurally
  available;
- the rehearsal must exercise the section 4.3 failure class deliberately —
  configuring a mode and confirming whether the process adopted it — so that the
  silent-ignore class is proven detectable;
- the rehearsal must confirm operationally that the loader store is
  **unavailable** outside `ENFORCE`, rather than assuming the fail-closed
  coupling of `ADR-0006` section 4.12.6.6.

### 10.4 Thresholds

```text
Evidence class: [UNVERIFIED]
Conclusion:     NO acceptable numeric limit is stated here. The measured timings
                are EVIDENCE for the later activation gate, not a target set by
                this record. Deriving and approving thresholds from measured
                baselines is separate work under ADR-0006 section 8.3.2 and is
                explicitly out of scope here.
```

---

## 11. Authorization model (specification section 3.9)

```text
implementation authorization
    != implementation merge authorization
    != OFF -> OBSERVE activation authorization
    != OBSERVE -> ENFORCE activation authorization
```

These are four distinct decisions over different effects. This record **may**
identify who performs an authorized production change, and does so in section 2.
It **may not** grant any production authorization, and grants none.

```text
OFF -> OBSERVE:     NOT AUTHORIZED
OBSERVE -> ENFORCE: NOT AUTHORIZED
BE-02 runtime:      NOT AUTHORIZED
```

Completion of `THOTH-GQL-OPS-01`, and merge of its pull request, authorize
nothing in production.

---

## 12. Audit and evidence model (specification section 3.10)

### 12.1 What a future mode transition must retain

| Evidence | Where it lives | Create a repository commit for it? |
|---|---|---|
| exact candidate/release SHA | GitHub | no |
| image/release identity actually deployed | the authoritative deployment source's own history | no |
| prior mode | the authoritative deployment source's own history | no |
| intended mode | the authoritative deployment source's own history | no |
| CTO transition authorization | GitHub, on the authorizing record | no |
| identity of the configuration/deployment action | the authoritative deployment source's commit, plus the deployment system's own record | no |
| expected serving population | captured at execution time from live orchestrator state | no |
| verified serving population | as above | no |
| per-instance effective-mode evidence | the section 7 mechanism's output | no |
| propagation timing | measured at execution time | no |
| rollback evidence | as above | no |
| observation-window evidence | runtime logs plus the observation record | no |

Per `ADR-0005` and `operating-model.md` section 5.1, **do not create a repository
commit merely to restate an event that GitHub, the deployment source's history or
the runtime already holds.** A material post-merge correction still requires its
own bounded task and pull request.

**No secret and no production configuration value may be duplicated into this
repository, in any file, including the runbook.**

### 12.2 Log retention versus the observation window

Three separate facts, deliberately not collapsed. Only the first is established.

```text
Evidence source: the log-retention parameter of the production Thoth GraphQL API
                 service definition in the authoritative deployment source, read
                 as a single scoped key
Evidence class:  [EXTERNAL]
Conclusion:      Runtime log retention for this service is configured to a
                 FINITE duration. It is not unbounded. The exact value is not
                 recorded here.
```

```text
Evidence class: [UNVERIFIED]
Conclusion:      The approved or planned OBSERVE observation-window duration is
                 NOT established. ADR-0006 section 7.2.3 requires an "explicit,
                 recorded observation window" but fixes no duration, and no
                 authoritative duration or selection criterion exists anywhere in
                 this repository or in the evidence gathered here.
```

```text
Evidence class: [UNVERIFIED]
Conclusion:      Whether the configured retention covers that future window is
                 therefore NOT established, and cannot be, until the window is.
                 No comparison between the two is asserted here in either
                 direction.
```

**Binding requirement, which does not depend on either unresolved value:**

```text
Before activation, observation evidence must be retained for at least the
complete approved observation window and must remain available through
review and sign-off.

An observation window whose evidence expires before it is reviewed is not
evidence.
```

The guard's compatibility events (`ADR-0006` section 8.3.1) are runtime log
events, so this requirement binds on runtime log retention specifically.

**No remedy is selected here, and selecting one is not `THOTH-GQL-OPS-04`'s
job either.** Whether the requirement is met by extending retention, by capturing
evidence out of band, or by choosing a window the existing retention already
covers, is an open choice that depends on the window duration nobody has yet set.
Pre-selecting one would invent the very evidence this record is recording as
missing.

The dependency order is therefore binding:

```text
PART 1 -- THOTH-GQL-OPS-04, runtime-operations gate
  record the retention requirement;
  re-establish that current retention is a FINITE configured duration;
  record the observation-window duration as NOT established;
  record coverage as therefore NOT established;
  record that the remedy is DOWNSTREAM.
  Select nothing. Implement nothing. Confirm nothing in place.

PART 2 -- downstream activation gate, in this order
  approve the observation-window duration;
  determine whether current retention covers it;
  if not, select and implement a remedy;
  verify the final retention arrangement before production activation.
```

Changing retention would be a change to the private deployment source and is
**not** authorized by this record.

---

## 13. CG-13 disposition and the runtime-operations gate

### 13.1 Disposition

```text
A. feature-specific CG-13 subset satisfied; broad CG-13 remains open
B. evidence proves the same control genuinely resolves all of CG-13
C. insufficient operational capability/evidence; BLOCKED
```

```text
Selected: C - insufficient operational capability/evidence; BLOCKED
```

**B is excluded on this record's evidence.** CG-13 requires documentation of
runtime, deployment, **migration execution**, **rollback**, **restore
verification** and approvers. This record addresses the mutation-guard
runtime-mode-control subset only. Migration execution, backup and restore
verification, and approver mapping for concerns other than this feature are
untouched.

**A is forbidden**, under `THOTH-GQL-OPS-01` section 12.1, while either capability
gap remains open. Both remain open:

```text
CAPABILITY GAP 1  (section 4.3, the OPS-02 gap)   OPEN
  the currently authoritative production deployment path cannot consume
  THOTH_GRAPHQL_MUTATION_GUARD_MODE, so a guard-enabled release deployed
  through it would remain effectively OFF and no OFF -> OBSERVE transition
  would be performable

CAPABILITY GAP 2  (section 4.2, the OPS-03 gap)   OPEN
  no implemented, independently reviewed and merged mechanism exists that
  can prove the effective mode of every serving instance
```

None of the following converts C into A, and none is claimed: having *specified*
the fleet-verification mechanism; having *specified* `THOTH-GQL-OPS-02` or
`THOTH-GQL-OPS-03`; having *documented* the entrypoint gap thoroughly; the guard
being inert everywhere already; or an argument that the remaining work is small.
Size is not the criterion; delivery is.

### 13.2 The gate

```text
Runtime-operations gate: NOT SATISFIED
Blocking prerequisites:  THOTH-GQL-OPS-02, THOTH-GQL-OPS-03
Earliest satisfaction:   THOTH-GQL-OPS-04, on evidence
```

`CG-13` remains **OPEN**. Consequently `OFF -> OBSERVE` remains blocked on this
gate in addition to every other gate `ADR-0006` imposes, and nothing downstream
of it — service-health thresholds, preview/staging rehearsal, activation
authorization — may proceed on the basis that this gate has been discharged.

### 13.3 The sequence that remains

```text
THOTH-GQL-OPS-01  control record, provisional runbook, prerequisite
                  specifications; gate NOT SATISFIED          <- THIS RECORD
    -> THOTH-GQL-OPS-02  mode-control path
                         specified, approved, implemented,
                         independently reviewed and merged
    -> THOTH-GQL-OPS-03  fleet-verification mechanism
                         specified, approved, implemented,
                         independently reviewed and merged
    -> THOTH-GQL-OPS-04  fresh bounded runtime-operations
                         verification and closure
    -> feature-specific CG-13 subset MAY become satisfied
       (runtime-operations gate satisfied at the earliest here)
    -> service-health signals / activation thresholds
                                        (ADR-0006 section 8.3.2)
    -> preview/staging performance + timed rollback rehearsal
    -> explicit CTO OFF -> OBSERVE authorization
```

Each arrow is a separate approval.

---

## 14. Unresolved evidence, recorded as missing work

| # | Missing | Class | Who must obtain it | Blocks AC |
|---|---|---|---|---|
| 1 | **accountable production runtime owner** — a designation, not an access record (section 2.1.1) | `[UNVERIFIED]` | explicit CTO designation, obtained by `THOTH-GQL-OPS-04` | AC-1 |
| 2 | **observation sign-off owner** — confirmation of the section 2.3 proposal, or a different designation | `[UNVERIFIED]` | explicit CTO confirmation, obtained by `THOTH-GQL-OPS-04` | AC-2 |
| 3 | the live current expected replica population | `[UNVERIFIED]` | `THOTH-GQL-OPS-04`, from live orchestrator state | AC-7 |
| 4 | **whether operational rollback needs CTO approval** or may be executed on the technical team's own authority (section 9.3) | `[UNVERIFIED]` | explicit CTO decision, obtained by `THOTH-GQL-OPS-04` | AC-13 |
| 5 | whether the running service matches its declared definition (drift) | `[UNVERIFIED]` | `THOTH-GQL-OPS-04` | — |
| 6 | the effective mode of any serving instance | `[UNVERIFIED]` | requires `THOTH-GQL-OPS-03` first | — |
| 7 | the approved `OBSERVE` observation-window duration (section 12.2) | `[UNVERIFIED]` | the **downstream** activation gate | — |
| 8 | whether the finite configured retention covers that window (section 12.2) | `[UNVERIFIED]` | the **downstream** activation gate, once item 7 exists | — |
| 9 | the retention **remedy**, if item 8 shows one is needed — selected, implemented and verified (section 12.2) | `[UNVERIFIED]` | the **downstream** activation gate; **not** `THOTH-GQL-OPS-04` | — |
| 10 | propagation duration | `[UNVERIFIED]` | measured at the **downstream** preview/staging rehearsal | — |
| 11 | mixed-window duration bound | `[UNVERIFIED]` | measured at the **downstream** preview/staging rehearsal | — |
| 12 | rollback latency/duration (section 9.4) | `[UNVERIFIED]` | measured at the **downstream** preview/staging rehearsal | — |
| 13 | service-health signals and activation thresholds | out of scope | separate downstream gate, `ADR-0006` section 8.3.2 | — |

Missing evidence is missing work. None of the above is softened into a narrative,
and no plausible mechanism is substituted for any of it.

Items 1, 2, 3 and 4 are the reason four acceptance criteria are recorded as
**FAIL/BLOCKED** rather than satisfied. Retaining failed criteria is the correct
outcome for a task whose terminal disposition is **C — BLOCKED**; converting any
of them into a PASS would require evidence that does not exist.

---

## 15. Related records

- [`THOTH-GQL-OPS-01`](../ai-delivery/tasks/THOTH-GQL-OPS-01.md) — the approved
  specification this record delivers;
- [mutation-guard mode-transition runbook](./graphql-mutation-guard-mode-transition-runbook.md)
  — **PROVISIONAL**, not executable;
- [`THOTH-GQL-OPS-02`](../ai-delivery/tasks/THOTH-GQL-OPS-02.md) — `DRAFT`,
  implementation `NOT AUTHORIZED`;
- [`THOTH-GQL-OPS-03`](../ai-delivery/tasks/THOTH-GQL-OPS-03.md) — `DRAFT`,
  implementation `NOT AUTHORIZED`;
- [`THOTH-GQL-OPS-04`](../ai-delivery/tasks/THOTH-GQL-OPS-04.md) — `DRAFT`,
  implementation `NOT AUTHORIZED`;
- [`ADR-0006`](../decisions/ADR-0006-request-scoped-graphql-batching.md) sections
  4.12.6.6, 7.2.1, 7.2.1.1, 7.2.4, 7.3, 8.3.2 and 8.3.5;
- [CG-13](./control-gaps.md#cg-13---thoth-runtime-operations-unmapped);
- [`environments.md`](./environments.md).
