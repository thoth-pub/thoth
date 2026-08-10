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
Expected terminal CG-13 disposition: **C — insufficient operational
capability/evidence; BLOCKED** (section 12.1). This is the expected outcome, not
a failure of the task.
Runtime-operations gate on completion: **NOT SATISFIED** (section 12.2).

Deployment-state distinction, binding on every statement in this specification
(established in section 2.2.0):

```text
merged develop state
    != deployed production release
    != production activation state
```

The currently deployed production release **predates** `THOTH-GQL-BATCH-01`. Its
binary contains no mutation guard at all, so it is recorded as **pre-guard** and
must never be described as running `MutationGuardMode::OFF`. That conclusion is
`[REPO + EXTERNAL]`: repository evidence establishes that the relevant release
code is pre-guard, and previously established scoped deployment metadata
establishes that it is the release production runs. Merging
`THOTH-GQL-BATCH-01` deployed nothing. Every guard-mode statement below applies
to a **guard-enabled candidate** — a build that contains the merged foundation —
and not to the pre-guard release currently serving production.

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
authorized and evidenced — and, where the operational capability required to do
any of that **does not currently exist**, identify the missing capability,
specify the bounded prerequisite task that must deliver it, and record the
runtime-operations gate as **unsatisfied**.

The subject of the control is the single value:

```text
THOTH_GRAPHQL_MUTATION_GUARD_MODE   in { OFF, OBSERVE, ENFORCE }
```

The task's output is a control record, a provisional runbook and a set of
specified prerequisite tasks. It is **not** a production change, and it is **not**
by itself a discharge of the `ADR-0006` runtime-operations gate.

**Binding limit on what this task can achieve.** Discovery (section 2.2)
establishes two capability gaps that this task cannot close by documenting them:

```text
1. the currently authoritative production deployment path cannot consume
   THOTH_GRAPHQL_MUTATION_GUARD_MODE, so a guard-enabled release deployed
   through that path would remain effectively OFF and no mode change would
   be performable;

2. no implemented, independently reviewed mechanism exists that can prove
   the effective mode of every serving instance.
```

Until **both** are delivered — implemented, independently reviewed and merged —
the mode of a guard-enabled release could not be changed in production, and a
change could not be verified if it were made. Documenting that fact, and
specifying the tasks that would fix it, is necessary work and is this task's
purpose; it is **not** equivalent to having the capability. Section 12 therefore
requires this task to terminate at disposition **C — BLOCKED**, and section 11
records the runtime-operations gate as **NOT SATISFIED** on completion.

At completion no environment has been transitioned. Environments running a
pre-guard release remain **pre-guard**; any guard-enabled candidate or
environment remains effectively `OFF` with the loader store unavailable; and
production request acceptance is unchanged.

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

`THOTH-GQL-OPS-01` opens the runtime-operations gate recorded in `ADR-0006`
section 12 and in the
[decision register](../../decisions/decision-register.md). It does **not** close
it. The materially correct sequence, which section 11.1 restates as the
authoritative form, is:

```text
THOTH-GQL-OPS-01 discovery/control work            <- THIS TASK
    -> entrypoint/configuration remediation
       implemented + reviewed + merged             (THOTH-GQL-OPS-02)
    -> fleet-verification mechanism
       implemented + reviewed + merged             (THOTH-GQL-OPS-03)
    -> fresh bounded runtime-operations
       verification/closure                        (THOTH-GQL-OPS-04)
    -> feature-specific CG-13 subset may become satisfied
    -> service-health signals / activation thresholds
    -> preview/staging performance + timed rollback rehearsal
    -> explicit CTO OFF -> OBSERVE authorization
```

The three successor tasks are **specified** by this one (section 3.12) and
implemented by nobody until each is separately approved and authorized. Their
branches must not exist, and none of them may be implemented in this task's
pull request.

### 2.2 Current behaviour, established at base `75f44aabc52d98596ea6ce69ab068b3698fcd524`

The findings below were verified against the repository at the base above and
against the pinned dependency sources resolved by the workspace `Cargo.lock`
(`clap` 4.6.1, `clap_builder` 4.6.0). An implementing agent **must refresh every
finding against its own exact base** before relying on it.

#### 2.2.0 Merged state, deployed release and activation state are three different things

This distinction is established first because every other finding in section 2.2
is conditional on it, and conflating the three would misdescribe the production
service.

```text
merged develop state
    != deployed production release
    != production activation state
```

The current-production conclusion below rests on **two** evidence classes, and
the split is binding. This repository can establish what a given release
*contains*; it cannot establish which release production is *running*. Attributing
the deployed-release identity to `[REPO]` alone would be an evidence-provenance
error of exactly the kind the section 2.2.5 labelling rule exists to prevent.

**Release-code state — `[REPO]`.** Established from this repository at the exact
base:

- the mutation guard, `MutationGuardMode` and the CLI argument
  `mutation_guard_mode()` exist on `develop`, delivered by
  [`THOTH-GQL-BATCH-01`](THOTH-GQL-BATCH-01.md) through merged PR
  [#791](https://github.com/thoth-pub/thoth/pull/791);
- they do **not** exist on the release line `master`: `master` contains no
  `MutationGuardMode`, its CLI defines no `mutation_guard_mode()`, and its
  GraphQL startup path has no mutation-guard wiring;
- the production release artefact is the container image published from a
  **published GitHub release**
  (`.github/workflows/docker_build_and_push_to_dockerhub_release.yml`), and the
  most recent release tag likewise contains no `MutationGuardMode`.

```text
Release-code state [REPO]:
The relevant release/master code contains no mutation guard and is PRE-GUARD.
```

This is the whole of what the repository establishes. It says nothing about which
release production runs.

**Deployment state — `[EXTERNAL]`.** Which release or image production is
actually running is deployment-state evidence and is **not** derivable from this
repository at all. It was established during this task's authorized read-only
discovery, from previously collected scoped metadata of the authoritative
deployment source named in section 2.2.4, under the scoped-read rules of section
2.2.5:

```text
Deployment state [EXTERNAL]:
Previously established scoped authoritative deployment metadata identifies
that release/image as the one currently deployed to production.
```

No value, identifier or configuration detail from that source is recorded here;
only the ownership/version fact the criterion requires.

**Combined conclusion — `[REPO + EXTERNAL]`.** Neither class establishes it
alone; the two together do:

```text
Combined conclusion [REPO + EXTERNAL]:
Current production is therefore PRE-GUARD and is not
MutationGuardMode::OFF.

- the currently deployed release predates THOTH-GQL-BATCH-01;
- its binary does not contain the mutation guard at all;
- it therefore does not literally have MutationGuardMode::OFF;
- it is recorded as PRE-GUARD, and no statement in this family of tasks may
  relabel a pre-guard binary as having an effective guard mode.
```

If an implementing agent cannot re-establish the `[EXTERNAL]` half from scoped
metadata at its own execution time, the deployed-state conclusion must be
downgraded to **`[UNVERIFIED]`** and reported as missing work. It must **not** be
re-derived from `[REPO]` evidence, and access must **not** be widened to obtain
it. The `[REPO]` half stands on its own regardless.

Merging `THOTH-GQL-BATCH-01` deployed nothing and activated nothing. The merged
foundation is inert *and* undeployed; those are two separate facts, established
by two different evidence classes, and neither implies the other.

**Vocabulary, binding on this specification and on every successor it
specifies:**

| Term | Meaning |
|---|---|
| **pre-guard** | a release, image or environment whose binary contains no mutation guard. It has no guard mode — not `OFF`, not any other value |
| **guard-enabled candidate** | a build, image or release containing the merged `THOTH-GQL-BATCH-01` foundation. It has a guard mode, whose default is `OFF` |
| **guard-enabled environment** | an environment actually running a guard-enabled candidate |
| **activation** | an authorized `OFF -> OBSERVE` or `OBSERVE -> ENFORCE` transition of a guard-enabled environment. None exists |

Sections 2.2.1, 2.2.2 and 2.2.3 describe the behaviour of a **guard-enabled**
build. They are statements about what would happen once such a build is deployed;
they are **not** descriptions of the pre-guard binary currently serving
production. The implementing agent must re-establish which release each
environment is actually running, under the scoped-read rules of section 2.2.5,
rather than assuming that `develop` and production agree.

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

This is **capability gap 2** of section 1. It is not merely an input to a later
activation checklist: while it holds, no mode change could be verified even if
one could be made, so the runtime-operations gate cannot be discharged. Section
3.5 requires the mechanism to be **specified**; section 3.12 requires it to be
delivered by a separate task (`THOTH-GQL-OPS-03`); and section 12.1 forbids
disposition A until that task has merged.

#### 2.2.3 The `init` entrypoint does not accept the mode — proven

This is the decisive finding of the discovery phase. Per section 2.2.0 it is a
finding about the **guard-enabled** code merged to `develop`: it describes what
the deployment path would do with a guard-enabled release, not the behaviour of
the pre-guard binary currently deployed.

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
When a GUARD-ENABLED container runs the image default command `init`,
THOTH_GRAPHQL_MUTATION_GUARD_MODE is silently ignored and that process's
effective mode is unconditionally OFF.
```

The failure is **silent** and **fail-safe**. It cannot cause an unintended
activation, it does not affect the correctness of the merged inert state, and it
is not a production incident — production is not running guard-enabled code at
all (section 2.2.0), and `OFF` is in any case the intended state of a
guard-enabled candidate. But it means that setting the environment variable would
appear to succeed while changing nothing, which is precisely the class of failure
the fleet-verification and partial-fleet requirements of this task exist to
detect.

Steps 1 to 6 are fully verifiable `[REPO]` from this repository and its pinned
dependencies. **Deployment-path applicability is established `[EXTERNAL]`:** the
production GraphQL API service does not override the container command and so
inherits the image default `init`. That is a property of the **deployment path**,
which a guard-enabled release would execute through unchanged; it is not a claim
about the guard state of the pre-guard binary currently deployed.

**Established consequence for this task:**

```text
Under the currently authoritative production deployment command path, a
guard-enabled release containing the merged foundation would execute
through `init`.

Until THOTH-GQL-OPS-02 is delivered, that path cannot consume
THOTH_GRAPHQL_MUTATION_GUARD_MODE.

Therefore an OFF -> OBSERVE transition of a guard-enabled candidate is not
operationally performable through the current deployment path.
```

This is **capability gap 1** of section 1, and it is the **OPS-02 capability
gap**. It is not a caveat on a later activation: while it holds,
`OFF -> OBSERVE` is not merely unauthorized, it is not **performable** by any
guard-enabled release deployed through this path. Section 12.1 forbids
disposition A until it is remediated by a separate task
(`THOTH-GQL-OPS-02`). The implementing agent must re-confirm the container
command from the authoritative source at execution time under the scoped-read
rules of section 2.2.5, and must separately re-confirm which release each
environment is running.

##### 2.2.3.1 `init` is not interchangeable with `start graphql-api`

This distinction is binding on every remediation discussion in this
specification, and getting it wrong would silently widen scope into the migration
half of CG-13.

`init` and `start graphql-api` are **not** two spellings of the same thing.
Established `[REPO]` from `src/bin/thoth.rs`:

```rust
Some(("init", arguments)) => {
    commands::run_migrations(arguments)?;
    commands::start::graphql_api(arguments)
}
```

so:

| Command | Runs database migrations | Starts the GraphQL API |
|---|---|---|
| `init` | **yes**, first, and aborts on failure | yes, only if migrations succeeded |
| `start graphql-api` | **no** | yes |

The `Dockerfile` states the same intent directly: "By default run `thoth init`
(runs migrations and starts the server on port 8080)".

**Consequence:** replacing the production container command with
`start graphql-api` would **remove migration execution from the deployment
path**. That is a change to the existing migration-execution and deployment
contract, it intersects the broader CG-13 migration/deployment problem that this
task explicitly does not address, and it is therefore **out of bounded scope**.
It must never be presented as an interchangeable feature-local fix for the mode
gap. Section 13.1 states the binding classification.

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
repository `thoth-pub/infrastructure`. It holds the authoritative deployment
definitions for the Thoth production and test services, including the container
image version, the container command and the task environment variables for the
production GraphQL API service, and it carries its own documented update
procedure.

That pointer is the **whole** of what this public repository records about that
source, and is recorded only because the specification cannot identify its
configuration authority without it. Deliberately **not** recorded here, in this
or any successor task: credential values, resource identifiers, account
identifiers, production topology, platform or orchestration detail, stack,
cluster or service names, hostnames beyond those already published in
`environments.md`, scaling parameters, and every value of every environment
variable. The implementing agent must establish current mechanism and values from
the authoritative source at execution time, under the scoped-read rules of
section 2.2.5, rather than from this document — and must not copy them here.

**Hazard, recorded because it constrains how this task may work.** The
authoritative deployment source is **secret-bearing**: it carries production
credential material inline in configuration. That single fact is the whole of
what this repository records about it, and it is recorded only because it
constrains the method of section 2.2.5. Remediating the exposure is **outside
this task's scope**; it remains a separate CTO-controlled security matter and
must be escalated rather than handled here, in this task or any successor
specified by it.

No further detail about that source may be published here or in any public
output. In particular: no credential value, no resource identifier, no account
identifier, no production topology, no stack, cluster or service name, and no
private configuration.

#### 2.2.5 Scoped-read rules for the private authoritative source

Binding on `THOTH-GQL-OPS-01` and on every successor task it specifies. These
rules exist because the source is secret-bearing, so the ordinary technique of
reading a configuration file whole is not available.

The implementing agent must:

1. use **narrowly scoped searches or line/range reads** targeted at the specific
   acceptance criterion being satisfied — never a whole-file read of a
   secret-bearing configuration file, and never a broad recursive dump;
2. retrieve **only the metadata the criterion requires** — typically the presence
   or absence of a setting, the name of a mechanism, an ownership record, or a
   rollout semantic — and stop there;
3. **never copy secret-bearing ranges** into a report, a specification, a
   changelog, a pull request, a commit message, a prompt or any other output,
   whether or not the value appears relevant;
4. treat the source as strictly **read-only**: make no change, open no pull
   request, dispatch no workflow and use no credential found there;
5. stop and report **`BLOCKED`** if the evidence a criterion needs cannot be
   obtained without exposing secret material. An unobtainable criterion is
   missing work, and a criterion is never satisfied by widening the read.

Incidental encounter with secret material during an otherwise scoped read is not
a breach, and must be reported as an escalation rather than quietly absorbed; it
becomes a breach only if the material is copied onward.

**Evidence-classification rule, binding.** Every operational statement the task
records must be labelled with the source class that establishes it:

```text
[REPO]            established by thoth-pub/thoth at the exact base
[EXTERNAL]        established by the authoritative deployment source
[REPO + EXTERNAL] established only by both together; neither class
                  establishes it alone, and it may not be attributed
                  to either one
[UNVERIFIED]      not established -- missing evidence is missing work
```

`[REPO + EXTERNAL]` exists because the two classes answer different questions and
must not be substituted for one another. This repository can establish what a
release *contains*; only the authoritative deployment source can establish which
release an environment *runs*. Every statement about the deployed state of an
environment — including the current-production pre-guard conclusion of section
2.2.0 — therefore carries `[REPO + EXTERNAL]` or `[UNVERIFIED]`, never `[REPO]`.
Where the `[EXTERNAL]` half cannot be obtained under the scoped-read rules above,
the conclusion is downgraded to `[UNVERIFIED]` and reported as missing work;
widening access to obtain it is forbidden, and re-deriving it from `[REPO]`
evidence is an evidence-provenance error.

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
4. **which command the production GraphQL API container actually runs**, and to
   confirm the section 2.2.3 established finding that it is the image default
   `init`. This is load-bearing because a guard-enabled release executing through
   `init` cannot consume the variable at all. This item asks **what the
   deployment path runs**; it must not be read as offering a change of command as
   a remedy — sections 2.2.3.1 and 13.1.1 put any such change out of bounded
   scope;
5. **which release each environment is actually running**, distinguishing a
   **pre-guard** release from a **guard-enabled** one per section 2.2.0. A
   pre-guard environment has no guard mode and must be recorded as pre-guard, not
   as `MutationGuardMode::OFF`. This is `[EXTERNAL]` evidence: the deployed
   release identity is not derivable from this repository, so the task must not
   infer it from the state of `develop`, `master` or any release tag. The
   resulting deployed-state conclusion is `[REPO + EXTERNAL]`, or `[UNVERIFIED]`
   if the `[EXTERNAL]` half cannot be obtained under section 2.2.5;
6. behaviour for an **absent** value in a guard-enabled build — established
   `[REPO]`: `clap` supplies the declared default `OFF`;
7. behaviour for an **invalid** value — established `[REPO]`: `clap`'s
   `value_parser` rejects any string outside `OFF`/`OBSERVE`/`ENFORCE`, so the
   process fails to start rather than starting in an unintended mode. The task
   must verify what the orchestrator does with a task that fails to start, since
   a rejected value manifests as a failing deployment rather than as a
   misconfigured running service;
8. who may **request** a change;
9. who may **execute** a change.

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

- it must be **specified** here and **implemented and reviewed separately**, as
  `THOTH-GQL-OPS-03` (section 3.12). The task must not add runtime observability
  in its own documentation PR, and must not add it silently anywhere;
- it must not expose the mode on an unauthenticated public surface unless the
  task establishes that doing so is acceptable, with reasoning — the guard mode
  describes a server-side request-acceptance policy, and publishing it is an
  information-disclosure decision, not a formatting choice;
- it must not change guard semantics, batching semantics or store semantics;
- it must remain inert with respect to request acceptance.

**Specifying the mechanism does not verify the fleet.** This is binding and is
the distinction the whole task turns on:

```text
a specification for a verifier   !=   a verifier
a verifier                       !=   a verified fleet
```

Satisfying acceptance criterion AC-8 means a mechanism has been **defined**. It
does not mean the effective mode of any instance has been established, and it
must never be recorded, summarised or reported as though it did. Until
`THOTH-GQL-OPS-03` is implemented, independently reviewed and merged, the
effective fleet mode remains **unverifiable**, and section 12.1 forbids
disposition A on that ground alone.

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
   `ENFORCE` is verified operationally rather than assumed. The runbook is
   necessarily **PROVISIONAL** and must be marked so: it describes procedures
   that cannot be executed until the section 3.12 prerequisites have merged, and
   it must not read as an executable runbook before then;
3. the three **prerequisite task specifications** required by section 3.12,
   written to `task-specification-template.md`, each `DRAFT` and each with
   implementation `NOT AUTHORIZED`;
4. the CG-13 disposition statement required by section 12, which under section
   12.1 must be **C — BLOCKED** unless both prerequisites have already merged;
5. an explicit statement that the `ADR-0006` runtime-operations gate is
   **NOT SATISFIED** on completion (section 12.2);
6. the changelog entry and the implementation report.

### 3.12 Required prerequisite task specifications

Discovery established two capability gaps (section 1) that this task cannot close
by documenting them. The task must **specify** the bounded successors that close
them, and must **not** implement any of them, create their branches, or treat
their specification as their delivery.

Task identifiers follow the repository's existing family convention
(`THOTH-DB-CTRL-01` / `-02`, `THOTH-GQL-BATCH-01`), and branch names follow
`feature/<programme-or-area>/<short-name>` per
[`task-specification-template.md`](../task-specification-template.md).

| Task | Closes | Type | Suggested branch (must not exist) |
|---|---|---|---|
| `THOTH-GQL-OPS-02` | capability gap 1 — mode-control path | runtime code | `feature/shared-architecture/graphql-guard-mode-entrypoint` |
| `THOTH-GQL-OPS-03` | capability gap 2 — effective-mode fleet verification | runtime code | `feature/shared-architecture/graphql-guard-mode-fleet-verification` |
| `THOTH-GQL-OPS-04` | the gate itself | documentation/control | `feature/shared-architecture/graphql-runtime-ops-closure` |

#### 3.12.1 `THOTH-GQL-OPS-02` — mutation-guard mode-control path

Objective: make `THOTH_GRAPHQL_MUTATION_GUARD_MODE` actually consumable on the
command path the production deployment runs, so that once a guard-enabled release
is deployed a mode change is possible at all.

Binding scope constraints:

- the remediation class is **feature-local and in-repository**: registering and
  propagating the mutation-guard mode through the `init` command **while
  preserving all existing `init` migration and startup semantics**, or another
  equally bounded in-repository solution that preserves those semantics;
- `init` must continue to run migrations first and to abort startup if they fail
  (section 2.2.3.1). A remediation that changes, reorders, conditionalises or
  removes migration execution is **out of scope** and must be escalated;
- the default remains `OFF`, and the merged state remains inert. Making the mode
  settable is **not** setting it;
- **an explicit production container-command override is not an alternative
  within this task.** See section 13.1 for the binding classification;
- `THOTH-GQL-OPS-01` must **not** select the fix. It records the class and the
  constraints; `THOTH-GQL-OPS-02`'s own approved specification selects the
  mechanism after the owning parties have been consulted.

Required evidence: that the mode is consumable on the production-applicable path
in a **release** build; that the debug-build panic path is also resolved; that
migration execution is unchanged; and that the default and merged state remain
`OFF` with the store unavailable.

#### 3.12.2 `THOTH-GQL-OPS-03` — effective-mode fleet-verification mechanism

Objective: implement the smallest mechanism specified under section 3.5, so the
effective mode of every serving instance can be established.

Binding scope constraints:

- it must satisfy every requirement of section 3.5, including per-instance
  attribution and proof of **effective** rather than intended mode;
- it must be able to detect the section 2.2.3 failure class specifically — a
  configured mode that the process silently did not adopt — since that failure is
  otherwise invisible;
- it must remain inert with respect to request acceptance, guard semantics,
  batching semantics and store semantics;
- it must expose no secret and no publisher or user data.

#### 3.12.3 `THOTH-GQL-OPS-04` — bounded runtime-operations verification and closure

Objective: after `THOTH-GQL-OPS-02` and `THOTH-GQL-OPS-03` have merged, verify
the mode-control path and the verification mechanism against the real runtime,
finalise the runbook from provisional to executable, and only then decide whether
the feature-specific CG-13 subset is satisfied.

`THOTH-GQL-OPS-04` is the **only** task in this family that may return CG-13
disposition **A**, and it may do so only on evidence, subject to its own
independent review and CTO decision. It may equally return **C** again if the
delivered capability proves insufficient.

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
    instead, and create or modify no security issue without separate explicit CTO
    authorization;
22. write any production configuration value, secret or resource identifier into
    this repository;
23. implement any of `THOTH-GQL-OPS-02`, `-03` or `-04`, or create their branches;
24. select the `THOTH-GQL-OPS-02` remediation mechanism (section 3.12.1);
25. change the production container command, or specify doing so as a
    feature-local fix (section 13.1);
26. return CG-13 disposition **A**, or record the runtime-operations gate as
    satisfied, while either capability gap of section 1 remains open
    (section 12.1).

## 5. Invariants

The implementation must preserve:

1. no production activation is performed by this task; no environment is
   transitioned to `OBSERVE` or `ENFORCE`; any guard-enabled candidate or
   environment remains effectively `OFF` unless separately authorized; and an
   environment running a pre-guard release is recorded as **pre-guard** and is
   never described as `MutationGuardMode::OFF` (section 2.2.0);
2. the loader store remains unavailable wherever a guard exists at all, which
   follows structurally from invariant 1 (`ADR-0006` invariant 30: store
   availability is derived only from the mode);
3. production request acceptance is unchanged;
4. no runtime, schema, migration, `Cargo` or workflow file is changed;
5. no production action of any kind is performed;
6. merge authorization and the two activation authorizations remain three
   distinct decisions;
7. CG-13 remains open except for any explicitly bounded, evidenced subset;
8. `BE-02` remains unauthorized;
9. no secret or production configuration value enters the repository;
10. no operational claim is recorded without a named evidence source and
    evidence class;
11. migration execution on the production startup path is unchanged, and no
    remediation that alters it is selected or specified here
    (sections 2.2.3.1 and 13.1);
12. the runtime-operations gate is recorded as **NOT SATISFIED**, and the CG-13
    disposition as **C**, unless both capability gaps of section 1 have already
    been closed by merged work (section 12.1).

## 6. Required behaviour

### 6.1 Success behaviour

The task succeeds when it has produced a complete, evidenced control record; a
**provisional** runbook; the three prerequisite task specifications of section
3.12; and an explicit, correct statement that the runtime-operations gate is
**NOT SATISFIED** and the CG-13 disposition is **C — BLOCKED**.

**Success is not an operator being able to act.** The earlier formulation of this
section — "an authorized operator could execute `OFF -> OBSERVE`, verify it
fleet-wide, detect a partial fleet and roll back" — is **withdrawn** as the
success condition for this task, because it is unachievable by documentation:
section 2.2.3 establishes that the mode cannot be changed at all on the
production-applicable path, and section 2.2.2 establishes that no change could be
verified. That formulation describes the success condition of
`THOTH-GQL-OPS-04`, after `-02` and `-03` have merged.

The distinction is binding:

```text
THOTH-GQL-OPS-01 succeeds by  establishing the control record,
                              proving the capability gaps and
                              specifying what must close them

THOTH-GQL-OPS-04 succeeds by  an operator actually being able to
                              execute, verify, detect and roll back
```

A `THOTH-GQL-OPS-01` result that claims the operator capability, or that reports
the gate as satisfied, is **wrong** regardless of how complete its documentation
is.

### 6.2 Failure behaviour

Where evidence is unavailable, the task records the **exact** missing evidence
and returns `BLOCKED` for the affected criterion. It does not substitute a
plausible mechanism, and it does not soften an unanswerable question into a
narrative. Missing evidence is missing work.

Where a required **capability** is unavailable, the same rule applies with equal
force: the task records the gap, specifies the task that would close it, and
does not treat the specification as the capability. Missing capability is missing
work.

### 6.3 Authorization

The task performs no production access, executes no deployment, uses or changes
no credential and dispatches no workflow. Inspection of any external
authoritative source is **read-only**, limited to ownership, mechanism and
configuration **metadata**, and bound by the scoped-read rules of section 2.2.5.
Because that source is secret-bearing, the task must expect to encounter secret
material and must handle it under those rules rather than assume it will not.

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
fleet-verification mechanism; it does not build one, and `THOTH-GQL-OPS-03`
delivers it. Until that task merges, the effective fleet mode remains
unverifiable. Service-health signals and activation thresholds are explicitly
**out of scope** and remain a separate gate — see section 11.

Operational runbook changes: this task produces the mode-transition runbook
required by `ADR-0006` section 8.3.5, marked **PROVISIONAL**: its procedures
cannot be executed until `THOTH-GQL-OPS-02` and `THOTH-GQL-OPS-03` have merged,
and it must not read as executable before then. The runbook is documentation. Its
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
      **AC-8 is satisfied by a definition and is deliberately not sufficient for
      AC-23 to AC-26; it must never be reported as fleet verification having
      occurred.**
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
      section 12, and consistent with the mandatory rule in section 12.1.
      *Evidence: `control-gaps.md` as delivered.*
- [ ] **AC-19** Monitoring and threshold work remains separately gated and is not
      absorbed. *Evidence: the delivered document; `ADR-0006` section 8.3.2.*
- [ ] **AC-20** `BE-02` remains unauthorized and untouched. *Evidence: the
      complete PR diff; PR #788 and issue #765 unchanged.*
- [ ] **AC-21** No environment is transitioned. Every guard-enabled candidate or
      environment remains effectively `OFF` with the store unavailable, and every
      environment running a pre-guard release is recorded as **pre-guard** rather
      than as `MutationGuardMode::OFF`. *Evidence: the deployment source,
      unchanged by this task; section 2.2.0.*
- [ ] **AC-22** The fleet-verification mechanism is specified but **not**
      implemented in this task's PR. *Evidence: the complete PR diff contains no
      runtime file.*

The following criteria exist so that the criteria above cannot, between them,
be read as discharging the gate. They are additions; none relaxes AC-1 to AC-22.

- [ ] **AC-23** Capability gap 1 — the inability of the current production
      deployment path to consume `THOTH_GRAPHQL_MUTATION_GUARD_MODE` once
      guard-enabled code is deployed — is recorded explicitly, as a
      deployment-path property, with its applicability established rather than
      left open, and without describing any pre-guard release as having a guard
      mode. *Evidence: sections 2.2.0, 2.2.3 and 2.2.3.1; `[EXTERNAL]`
      confirmation of the production container command under the section 2.2.5
      scoped-read rules.*
- [ ] **AC-24** Capability gap 2 — the absence of an implemented effective-mode
      verification mechanism — is recorded explicitly as an unclosed gap, and is
      **not** reported as closed by AC-8. *Evidence: section 2.2.2 and the
      delivered document.*
- [ ] **AC-25** The CG-13 disposition is **C — BLOCKED**, unless **both**
      `THOTH-GQL-OPS-02` and `THOTH-GQL-OPS-03` have already been implemented,
      independently reviewed and merged at the task's exact base. *Evidence:
      section 12.1; the merge state of those tasks on `develop`.*
- [ ] **AC-26** The `ADR-0006` runtime-operations gate is recorded as
      **NOT SATISFIED** on completion, in the specification, the control record,
      the implementation report and the pull-request body alike. *Evidence:
      section 12.2 and the delivered artefacts.*
- [ ] **AC-27** The three prerequisite task specifications of section 3.12 exist,
      each `DRAFT` with implementation `NOT AUTHORIZED`, and none of their
      branches exists. *Evidence: the delivered files; `git branch -r`.*
- [ ] **AC-28** No remediation that changes migration execution on the startup
      path is selected or specified, and any documented production
      container-command override carries the section 13.1 classification.
      *Evidence: the delivered document; section 2.2.3.1.*
- [ ] **AC-29** The runbook is marked **PROVISIONAL** and states that its
      procedures are not executable until the section 3.12 prerequisites merge.
      *Evidence: the delivered runbook.*
- [ ] **AC-30** Every read of the private authoritative source complied with the
      section 2.2.5 scoped-read rules, and any incidental encounter with secret
      material was escalated rather than copied onward. *Evidence: the
      implementation report's method statement.*

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
- no statement describes a **pre-guard** release, image or environment as having
  a guard mode, or as being in `MutationGuardMode::OFF` (section 2.2.0), and no
  statement implies that merging `THOTH-GQL-BATCH-01` deployed the guard;
- `OBSERVE` and `ENFORCE` remain recorded as NOT AUTHORIZED;
- `BE-02` remains recorded as NOT AUTHORIZED;
- a `CHANGELOG.md` entry exists under `## [Unreleased]`;
- the docs-only CI classification and the exact-head CI result are recorded.

### Manual verification

- re-derive sections 2.2.0, 2.2.1, 2.2.2 and 2.2.3 against the task's own exact
  base before relying on them;
- confirm the production container command from the authoritative source;
- confirm, separately from the command, **which release each environment is
  actually running**, and record pre-guard environments as pre-guard;
- confirm that no environment was transitioned and that no guard-enabled
  candidate was activated.

### Performance

Not applicable.

## 11. Rollout

- **initial state after merge:** unchanged. Documentation only.

  ```text
  deployed production release       = pre-guard (no guard mode exists)
  guard-enabled candidate default   = OFF, loader store unavailable
  environments transitioned         = none
  production request acceptance     = unchanged
  ```

- **feature flag/configuration:** none introduced. The guard mode is the existing
  control of a guard-enabled build and is not changed;
- **repository-managed deployment configuration:** this repository holds none. If
  the task discovers that it must touch any, the touched configuration must leave
  every guard-enabled candidate effectively `OFF` and must require separate
  production activation authorization;
- **staging/preview validation:** the rehearsal is **defined** by this task and
  **executed** at the later preview/staging gate (section 3.8);
- **pilot:** not applicable. `OBSERVE` is itself the controlled pilot
  (`ADR-0006` section 7.2.2) and is not authorized by this task;
- **activation approval:** unchanged and still required. Completing this task
  does **not** authorize `OBSERVE`;
- **observation period:** not applicable to this task.

### 11.1 The gates remaining after this task completes

This is the authoritative sequence, and it supersedes any shorter form. Note that
the runtime-operations gate this task belongs to is **still open** when the task
completes:

```text
THOTH-GQL-OPS-01 merged      (control record, provisional runbook,
                              prerequisite specifications;
                              runtime-operations gate NOT SATISFIED)
    -> THOTH-GQL-OPS-02  entrypoint/configuration remediation
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

Each arrow is a separate approval. `THOTH-GQL-OPS-02` and `-03` are runtime-code
tasks and carry their own risk classification, independent review and merge
authorization; neither is authorized by this specification's approval.

### 11.2 Monitoring boundary

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

C. insufficient operational capability/evidence;
   BLOCKED
```

CG-13 requires documentation of runtime, deployment, **migration execution**,
**rollback**, **restore verification** and approvers. This task addresses the
mutation-guard runtime-mode-control subset only. Migration execution, backup and
restore verification, and approver mapping for concerns other than this feature
are untouched by it, so **B is excluded on this task's evidence** and must not be
claimed merely because the guard mode has been operationally mapped.

Recording the result must keep CG-13 **open**. The task may add a durable
reference to the bounded successors; it may not mark CG-13 resolved.

### 12.1 Mandatory disposition rule

**This rule is binding and is not subject to the implementing agent's judgement.**

```text
Disposition A is FORBIDDEN while either of the following is true:

1. the currently authoritative production deployment path cannot consume
   THOTH_GRAPHQL_MUTATION_GUARD_MODE, so a guard-enabled release deployed
   through it would remain effectively OFF and no OFF -> OBSERVE transition
   would be performable
   (capability gap 1, the OPS-02 gap; sections 2.2.3 and 3.12.1);

2. no implemented, independently reviewed and merged mechanism exists that
   can prove the effective mode of every serving instance
   (capability gap 2; sections 2.2.2 and 3.12.2).

While either holds, the required disposition is C - BLOCKED.
```

At this specification's base both are true, so `THOTH-GQL-OPS-01`'s expected
terminal disposition is **C**. That is the correct outcome of the task, not a
failure of it: the task's value is establishing the gaps rigorously and
specifying what closes them.

**What does not satisfy the rule.** None of the following converts C into A:

- having *specified* the fleet-verification mechanism (AC-8). A specification for
  a verifier is not a verifier, and a verifier is not a verified fleet;
- having *specified* `THOTH-GQL-OPS-02` or `THOTH-GQL-OPS-03`. Specifying a task
  is not delivering it;
- having *documented* the entrypoint gap thoroughly. Documenting a missing
  capability does not supply it;
- the guard being inert everywhere already, and therefore "correct". That the
  intended state of a guard-enabled candidate is `OFF` — and that production is
  not even running guard-enabled code (section 2.2.0) — is why the gap is
  fail-safe; it is not evidence that the control works;
- an argument that the remaining work is small, obvious or low-risk. Size is not
  the criterion; delivery is.

**When A becomes reachable.** Only after both `THOTH-GQL-OPS-02` and
`THOTH-GQL-OPS-03` are implemented, independently reviewed and merged, and only
through `THOTH-GQL-OPS-04` (section 3.12.3), which must re-verify against the
real runtime and reach its own evidenced conclusion. `THOTH-GQL-OPS-04` may also
return C again.

### 12.2 Effect on the ADR-0006 runtime-operations gate

The `ADR-0006` section 7.2.4 / section 12 gate reads:

```text
runtime-operations evidence for mode control verified
```

`THOTH-GQL-OPS-01` **does not satisfy this gate.** The gate requires that mode
control be *verified*, and at this task's base the mode cannot be changed on the
production-applicable path and no change could be verified if it were. The task
must record, in the specification, the control record, the implementation report
and the pull-request body alike:

```text
Runtime-operations gate: NOT SATISFIED
Blocking prerequisites:  THOTH-GQL-OPS-02, THOTH-GQL-OPS-03
Earliest satisfaction:   THOTH-GQL-OPS-04, on evidence
```

Consequently `OFF -> OBSERVE` remains blocked on this gate in addition to every
other gate `ADR-0006` imposes, and nothing downstream of it — service-health
thresholds, preview/staging rehearsal, activation authorization — may proceed on
the basis that this gate has been discharged.

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

### 13.1 The entrypoint gap: a blocking prerequisite, not a stop condition

Section 2.2.3 establishes that a guard-enabled build running the `init`
entrypoint silently ignores the mode variable in a release build, and that the
production deployment path runs `init`. **A guard-enabled release deployed
through the current path would therefore have no controllable mode at all**, and
remediation is a hard, blocking prerequisite for `OFF -> OBSERVE` (section 12.1,
capability gap 1). This is a property of the deployment path, not a claim about
the pre-guard binary currently deployed (section 2.2.0).

It is a **blocking prerequisite** rather than a **stop condition** because the
approved architecture is intact: the `OFF`/`OBSERVE`/`ENFORCE` lifecycle needs no
change, the merged state is correct and fail-safe, and the gap is closable by
bounded runtime work. `THOTH-GQL-OPS-01` therefore proceeds, records the gap, and
terminates at disposition C — it does not abort.

The task must:

- record the finding with its evidence and its established production
  applicability;
- record it as **capability gap 1**, blocking under section 12.1;
- hand remediation to `THOTH-GQL-OPS-02` (section 3.12.1) as separately
  specified, separately reviewed, separately authorized work;
- require the section 3.5 mechanism to detect this exact failure class, since its
  defining characteristic is that it is silent;
- **not select the remediation mechanism**, and not implement one.

#### 13.1.1 Remediation classes, and the binding boundary between them

**Class 1 — feature-local, in scope for `THOTH-GQL-OPS-02`.**

Registering and propagating the mutation-guard mode through the `init` command
**while preserving all existing `init` migration and startup semantics**, or
another equally bounded in-repository solution that preserves those semantics.

The defining property of this class is that migration execution on the startup
path is **unchanged**: `init` still runs migrations first and still aborts
startup if they fail (section 2.2.3.1).

**Class 2 — production container-command override, NOT in this class and NOT
interchangeable.**

Replacing the production container command with `start graphql-api` is **not** an
alternative spelling of the class 1 fix and must never be documented as one.
Established in section 2.2.3.1: `init` runs migrations and then starts the API,
whereas `start graphql-api` starts the API **without running migrations**. An
override would therefore **remove migration execution from the deployment path**.

Binding classification, to be reproduced wherever an override is mentioned at
all:

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
- **no production command or configuration change is authorized** by this
  specification, by `THOTH-GQL-OPS-02`, or by their approval.

Recording all of this is required. Fixing it here is prohibited.

## 14. Expected implementation report

The agent must use
[`implementation-report-template.md`](../implementation-report-template.md) and
must record:

- exact base and head commits;
- actual files changed;
- the evidence class of every operational conclusion;
- the exact evidence that is missing, where any is;
- **the exact capability that is missing**, distinguished from missing evidence;
- the CG-13 disposition and its justification. Under section 12.1 this must be
  **C — BLOCKED** unless both prerequisites have already merged, and the report
  must state which of the two capability gaps remain open;
- an explicit statement that the **runtime-operations gate is NOT SATISFIED**,
  naming `THOTH-GQL-OPS-02` and `THOTH-GQL-OPS-03` as the blocking prerequisites
  and `THOTH-GQL-OPS-04` as the earliest point of satisfaction;
- explicit confirmation that the runbook is marked **PROVISIONAL**;
- explicit confirmation that no remediation altering migration execution was
  selected or specified, and that any documented production command override
  carries the section 13.1.1 classification;
- explicit confirmation that no production action, no mode change, no deployment,
  no credential use, change or rotation, and no production service or database
  access occurred;
- a **method statement** for every read of the private authoritative source,
  confirming compliance with the section 2.2.5 scoped-read rules and reporting
  any incidental encounter with secret material as an escalation. The report must
  **not** claim that no secret material was encountered unless that is literally
  true; the accurate and expected statement is that secret-bearing configuration
  was encountered during authorized read-only discovery and that no value was
  copied into any output;
- the deployment-state distinction of section 2.2.0, stated explicitly: which
  release each environment actually runs, with pre-guard environments recorded as
  **pre-guard** rather than as `MutationGuardMode::OFF`, and with capability gap 1
  stated as a property of the deployment path rather than of the deployed binary;
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
