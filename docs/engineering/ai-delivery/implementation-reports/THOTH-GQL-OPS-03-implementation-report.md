# THOTH-GQL-OPS-03 Implementation Report

Implementation of the approved
[`THOTH-GQL-OPS-03`](../tasks/THOTH-GQL-OPS-03.md) specification: the
**effective-mode fleet-verification mechanism**.

The feasibility question the specification deliberately left open
(section 3.2.3) is answered **FEASIBLE**. A compliant mechanism exists inside
the approved section 3.2 boundary, and it is implemented here.

```text
feasibility:                           FEASIBLE
capability gap 2 (fleet verification): implementation candidate exists;
                                       NOT closed until the review/merge
                                       lifecycle completes
real fleet verified:                   NO - none, anywhere
deployment performed:                  NONE
runtime transition performed:          NONE

CG-13:                                 OPEN
Runtime-operations gate:               NOT SATISFIED
Runbook:                               PROVISIONAL
OBSERVE:                               NOT AUTHORIZED
ENFORCE:                               NOT AUTHORIZED
BE-02 runtime:                         NOT AUTHORIZED
THOTH-GQL-OPS-04:                      NOT IMPLEMENTED, branch absent
```

**Implementing a verifier is not verifying a fleet.** This task delivers the
middle term of the specification's binding distinction and nothing beyond it.

## 1. Repository state

Repository: `thoth-pub/thoth`
Programme: Shared Thoth GraphQL / Backend Architecture
Task ID: `THOTH-GQL-OPS-03`
Approved specification: [`THOTH-GQL-OPS-03`](../tasks/THOTH-GQL-OPS-03.md)
Risk: HIGH
Workflow: STANDARD
Base branch: `develop`
Authorized exact base: `2bec75e6698232f7643862120e5437452fcfa252`
PR target: `develop`
Programme integration branch: None
Task branch: `feature/shared-architecture/graphql-guard-mode-fleet-verification`
Head commit: see section 11 and the pull-request record
Pull request: [#799](https://github.com/thoth-pub/thoth/pull/799) — **DRAFT, UNMERGED**
Expected branch deletion after merge: YES
Final programme PR required: NO
Implementing model: Claude Opus 5
Reasoning level: HIGH

### 1.1 Authorization provenance

```text
specification merge          PR #798, merge commit
                             2bec75e6698232f7643862120e5437452fcfa252

specification approval       PR #798 comment 5252446279
                             (CTO / control owner, 2026-08-11)

implementation authorization PR #798 comment 5252526720
                             (CTO / control owner, 2026-08-11)
```

Comment `5252526720` was read directly from GitHub before the implementation
branch was created. It authorizes implementation of `THOTH-GQL-OPS-03` only,
anchored to the exact base above, restates the section 3.2 limits, and requires
`BLOCKED` rather than architecture widening if no compliant mechanism exists.

Preflight, executed before any change:

```text
worktree                       CLEAN
origin/develop                 2bec75e6698232f7643862120e5437452fcfa252  (== authorized base)
PR #798                        MERGED, merge commit == authorized base
comment 5252446279             present
comment 5252526720             present, materially as described
implementation branch          ABSENT before creation
OPS-03 implementation PR       ABSENT
branch created at              2bec75e6698232f7643862120e5437452fcfa252
```

Merge authorization, independent review and the merge itself remain terminal
GitHub evidence under [`ADR-0005`](../../decisions/ADR-0005-terminal-merge-evidence.md)
and are deliberately **not** recorded in the task file.

## 2. Scope confirmation

Implemented objective: the smallest separately reviewable mechanism that can
prove the process-effective mutation-guard mode of every member of an enumerated
serving population, detect a mixed-mode fleet, keep `UNKNOWN` distinct from
`OFF`, and expose the silent-adoption failure class.

Out-of-scope changes made: NONE.

Not done, deliberately: no mode activated anywhere, no environment transitioned,
no deployment, no workflow dispatch, no credential use, no real fleet verified,
no migration/schema/data change, no public GraphQL schema or SDL change, no
`THOTH-GQL-OPS-04` work, no `THOTH-GQL-OPS-02` reopening, no CG-13 closure, no
lifting of the runbook's `PROVISIONAL` marking.

## 3. Selected mechanism

### 3.1 What it is

```text
PRODUCER  Each serving process emits ONE structured record, once, at startup,
          on its OWN log stream:

              THOTH_MUTATION_GUARD_EFFECTIVE_MODE mode=<MODE> instance=<ID>

          It is read out of the SAME `Data<MutationGuardMode>` that is
          installed as the request path's `app_data`.

COLLECT   Out of band, through the orchestration/logging plane, per instance.
          The record never crosses the public listener.

VERIFIER  `thoth_api::graphql::verify_fleet(enumerated, observations)`
          -> CONSISTENT(mode) | MIXED | NOT ESTABLISHED
```

The producer lives in `thoth-api-server` (four lines of runtime change); the
record format, the identity type and the verifier live in one new module,
`thoth-api/src/graphql/fleet_verification.rs`.

### 3.2 Why it is the smallest compliant mechanism

It adds **no** HTTP route, **no** listener, **no** port, **no** configuration
option, **no** authorization decision, **no** third-party dependency, **no**
persistent state and **no** request-path work. The only new runtime behaviour is
one `log::info!` call executed once per process. Every other candidate below
costs strictly more and buys nothing section 3 requires.

It also needs **no change to the private authoritative deployment source**: a
process's own output stream is collected per instance by every orchestrator this
service can run under, so per-instance attribution is available without asking
for an infrastructure change. That is the decisive difference from the admin
listener alternative, and it is why this task returns `FEASIBLE` rather than
hitting the specification's stop condition on private-deployment-source changes.

### 3.3 Alternatives considered and rejected

| Option | Disposition |
|---|---|
| Public unauthenticated HTTP surface (new route, or a field on `GET /` `ApiConfig`) | **REJECTED** by approved section 3.2. It publishes request-acceptance policy to any caller and, behind a shared load balancer, cannot address an individual replica — so it fails section 3 item 5 on its own terms |
| A field on the public GraphQL schema | **REJECTED** by approved section 3.2, and forbidden as a schema/SDL change (non-goal 15) |
| Authenticated public surface | **REJECTED**: introduces a new authorization decision (non-goal 8, section 6.3) and still cannot address a replica through the shared load balancer |
| Separate administrative HTTP listener on its own port | **NOT SELECTED**. It is inside the boundary in principle, but it is strictly larger — a new bound socket, a new configuration option and a new network surface — and it cannot be reached per instance without a change to the private authoritative deployment source, which this task may specify but not make. The log record achieves the same per-instance attribution with none of that |
| Orchestrator `exec` into the container running a CLI subcommand | **REJECTED**: a second process re-derives the mode from configuration, so it is structurally incapable of reporting the *serving* process's stored value. That is exactly the "independently settable second signal" invariant 6 forbids |
| Inferring the mode from request behaviour (probing for a guard rejection) | **REJECTED**: distinguishes only `ENFORCE`, leaves `OFF` and `OBSERVE` indistinguishable, samples through the shared load balancer, and would make verification depend on sending mutations |
| A file written to disk by the process | **REJECTED**: a persistent side effect, needs a volume, and is not collected per instance by the orchestration plane |

### 3.4 Section 3.2 disclosure assessment

```text
public unauthenticated effective-mode surface   NONE - no HTTP surface added
public GraphQL schema                           UNCHANGED
generated SDL                                   BYTE-IDENTICAL
new authorization decision                      NONE
existing authorization decision changed         NONE
```

Reachability: the record exists only on the process's own standard error, which
the public listener never reads from and never writes to. There is no route to
probe, so the negative case is structural rather than a matter of a route being
correctly gated. It is asserted anyway, in three ways (section 9.4).

What the mechanism can carry is bounded by its types, not by convention:
`EffectiveModeObservation` has exactly two fields, and neither can hold a
secret, a credential, an environment-variable value, deployment configuration,
request data, a GraphQL document, variables, mutation argument values, or
publisher/user data. This is the same structural argument `ADR-0006` section 8.2
makes for the guard's own `GuardEvent`.

## 4. Effective-mode source of truth

### 4.1 The single stored value

`start_server` builds **one** `Data<MutationGuardMode>` from its
`mutation_guard_mode` parameter, reads the observation out of it, and installs
that same value as `app_data`:

```rust
let guard_mode = Data::new(mutation_guard_mode);
// ...
log::info!("{}", effective_mode_observation(&guard_mode).record());
// ...
.app_data(guard_mode.clone())
```

`Data` is reference-counted, so `guard_mode.clone()` shares the *same
allocation* the record was read from. The `POST /graphql` handler extracts
`Data<MutationGuardMode>` and derives both the guard decision and the request
context's store availability from it, exactly as before.

### 4.2 Why the two cannot disagree

- there is **one** stored value, and both paths read it;
- `EffectiveModeObservation::for_process(mode)` takes the effective mode as its
  only mode input; the type has no setter, no default and no interior
  mutability, so it cannot hold a mode nobody supplied;
- nothing in the mechanism re-reads `THOTH_GRAPHQL_MUTATION_GUARD_MODE`,
  re-parses configuration, consults the environment, or infers the mode from
  deployment intent or request behaviour;
- the record's mode token is parsed back by the **same** `FromStr` the command
  path uses, so the wire form cannot drift from the value the CLI accepts.

Proved at runtime, not only by inspection:
`the_observed_mode_is_the_mode_the_request_path_actually_uses` builds one
`Data<MutationGuardMode>` per mode, reads the observation from it, registers
that same value on the real routes, and asserts that the request path's own
behaviour — whether the guard declines a colliding mutation — matches the
observed mode in all three modes.

## 5. Correlation identity — field-by-field justification

The record carries exactly two fields. Every other candidate field was
considered and excluded as unnecessary.

| Field | Value | Why it is necessary | Why it discloses nothing further |
|---|---|---|---|
| `mode` | `OFF` \| `OBSERVE` \| `ENFORCE` | It *is* the observation. Without it there is nothing to verify | It is one of three fixed tokens; the type cannot hold anything else |
| `instance` | the process's OS-reported host name | The orchestrator-assigned instance name (task/pod identity) in the deployment shapes this service runs under. Without it an observation is anonymous and cannot be matched to an enumerated member, which section 3 item 4 and AC-3 require | It is the container's own name, not a secret, not a credential, not an environment-variable value and not deployment configuration. `InstanceIdentity::new` accepts only a single host-name-like token — alphanumerics, `-`, `.`, `_`, at most 253 characters — so the field structurally cannot carry a sentence, a URL, a configuration fragment or a forged extra field |

Excluded as unnecessary: process id, thread counts, bind address, port, region,
availability zone, container image or digest, service or cluster name, task
revision, uptime, build metadata, request counts. None is needed to attribute
one observation to one enumerated instance, so under AC-23 none is carried.

`instance` is **optional**. It is read from the operating system
(`/proc/sys/kernel/hostname`, then `/etc/hostname`), never from an environment
variable — an environment-variable value is outside the section 3.2 boundary,
and a process's identity must not depend on deployment configuration choosing to
supply it. Where no host name is published, the field is omitted and the
observation is *unattributable*, which verification reports as `UNKNOWN`. It is
never defaulted, and never invented. Resolution happens once, in a `OnceLock`,
so the identity is stable for the process lifetime.

`a_linux_process_can_always_identify_itself` asserts that on Linux — the
platform the service is containerised for — an identity always resolves, so a
production observation is always attributable. On a developer platform that
publishes no host name the graceful `UNKNOWN` path is exercised instead.

## 6. Fleet-verification model

### 6.1 Enumeration and complete coverage

`verify_fleet(enumerated, observations)` takes the running instance set read
from **live orchestrator state** as a required argument. Completeness is
therefore a property of the call, not of operator discipline: every enumerated
member is answered for, and a member with no attributable observation is
`UNKNOWN`.

Sampling traffic through the shared load balancer cannot produce this input at
all — it yields responses, not an enumeration — so a sample-only design cannot
satisfy the signature, let alone the assertions.

### 6.2 Outcomes

```text
CONSISTENT(mode)   every enumerated member established that one mode
MIXED              every enumerated member established a mode, and they differ
NOT ESTABLISHED    anything else
```

`NOT ESTABLISHED` is returned when any member is `UNKNOWN`, when the enumeration
is empty, when the enumeration repeats an identity, when a member's observations
contradict each other, or when any observation is unattributable or falls
outside the enumeration. The last case matters: evidence the enumeration cannot
account for means the enumeration is stale or incomplete, so the population must
not be pronounced consistent.

`confirms(intended)` answers `true` only for `Consistent(intended)`. `MIXED` and
`NOT ESTABLISHED` both answer `false`, so "2 of 3 instances agree" has no path
to success anywhere in the API.

### 6.3 Mixed-mode semantics

A fully covered population whose members established different modes is `MIXED`,
and `MIXED` confirms nothing in either direction. Rolling replacement producing
two live generations is exactly this case.

### 6.4 `UNKNOWN` semantics

`MemberMode` is an enum: `Established(MutationGuardMode)` or `Unknown`. `UNKNOWN`
is a distinct variant in the result shape itself, not a prose caveat, so no code
path can coerce it to `OFF`. This is also what makes invariant 12 hold: a
**pre-guard** release has no guard mode at all and emits no record, so it is
reported `UNKNOWN` and is never described as `MutationGuardMode::OFF`.

### 6.5 Incomplete-coverage failure evidence

- `an_uncovered_member_is_unknown_and_the_verification_is_not_established`
- `partial_agreement_is_not_fleet_consistency` (2 of 3 agree → `NOT ESTABLISHED`)
- `a_pre_guard_instance_is_reported_unknown_rather_than_off`
- `evidence_the_enumeration_does_not_account_for_establishes_nothing`
- `an_ambiguous_or_empty_enumeration_establishes_nothing`
- `contradictory_observations_for_one_member_leave_it_unknown`
- `a_process_that_cannot_identify_itself_emits_an_unattributable_record`
- `unknown_is_structurally_distinct_from_off_and_never_decays_into_it`
- real-process: `real_records_that_disagree_about_one_enumerated_member_establish_nothing`

Every one of these asserts `confirms(..) == false` for all three modes, or the
`NOT ESTABLISHED` outcome directly.

## 7. Silent-adoption detection

### 7.1 Test design

The divergence is constructed **deliberately**, from two genuinely different
real inputs, in
`silent_adoption_is_caught_because_the_process_reports_what_it_computed`:

```text
declared configured intent   THOTH_GRAPHQL_MUTATION_GUARD_MODE=OBSERVE
                             (what a deployment would set)

effective mode               an explicit --mutation-guard-mode OFF on the
                             command line, which clap gives precedence
                             over the environment

recorded by the process      mode=OFF        <- what it ACTUALLY computed
verifier                     confirms(OBSERVE) == false, and the
                             intent/effective divergence is reported
```

A **real** process is started, it starts normally, and the record reports the
computed value. The declared intent is then compared against it by
`divergences_from_declared_intent`.

### 7.2 The closed `THOTH-GQL-OPS-02` defect was not recreated

Explicitly: the defect `THOTH-GQL-OPS-02` closed — `init` not registering the
mutation-guard argument, so a configured value was silently unreadable in a
release build and panicked in a debug build — is **not** reintroduced, weakened,
depended on or assumed anywhere in this task. No argument registration, no
`value_parser`, no default and no `init` ordering was touched.

The fixture relies on argument parsing working *exactly as
`THOTH-GQL-OPS-02` left it*: both inputs are legitimate, precedence between them
is clap's documented behaviour, and neither is a bug. The test therefore remains
valid however the startup path is later refactored, which is what the
specification requires of it.

A verifier-level equivalent,
`silent_adoption_is_visible_as_a_divergence_from_declared_intent`, covers the
same class across a multi-member fleet.

## 8. Files changed

- `thoth-api/src/graphql/fleet_verification.rs` *(new)*
  - reason: the record format, the correlation identity type and the fleet
    verifier.
  - behavioural effect: none on any existing path. Pure data types and pure
    functions; no I/O beyond one cached read of the OS host name.
- `thoth-api/src/graphql/fleet_verification/tests.rs` *(new)*
  - reason: unit tests for the producer, the record format and the verifier.
  - behavioural effect: test-only.
- `thoth-api/src/graphql/mod.rs`
  - reason: declare the module and re-export its public items.
  - behavioural effect: none. Additive re-exports only; no existing item moved,
    renamed or altered.
- `thoth-api-server/src/lib.rs`
  - reason: build the one stored `Data<MutationGuardMode>`, emit the record from
    it once at startup, and install that same value as `app_data`; plus the
    server-level test module.
  - behavioural effect: one `log::info!` per process at startup. The `graphql`
    handler is untouched. `Data::new` moved out of the per-worker app factory so
    all workers share the one value that was observed.
- `tests/mutation_guard_fleet_verification.rs` *(new)*
  - reason: real-process evidence — an actual `thoth` process, started on the
    real `start graphql-api` command path, reports the mode it computed.
  - behavioural effect: test-only.
- `docs/engineering/ai-delivery/tasks/THOTH-GQL-OPS-03.md`
  - reason: lifecycle reconciliation — `Status: APPROVED`,
    `Implementation: AUTHORIZED`, `Approved by: CTO / control owner`, and the
    durable implementation authorization in section 17.2.
  - behavioural effect: control record only. No acceptance criterion, non-goal,
    invariant or architecture statement was altered.
- `docs/engineering/repository-map/graphql-mutation-guard-mode-transition-runbook.md`
  - reason: section 4 said "the verifier does not yet exist", which the merged
    state would falsify. New section 4.0 records the concrete mechanism, as
    section 8 of the specification permits.
  - behavioural effect: documentation only. The runbook remains `PROVISIONAL`
    and not executable; section 0.2's two-part status is unchanged.
- `CHANGELOG.md`
  - reason: required entry.

Not touched: migrations, database schema, `db/`, GraphQL schema or resolvers,
`Dockerfile`, `docker-compose.yml`, `.github/`, `Cargo.toml`, `Cargo.lock`,
`control-gaps.md`, the runtime-operations control record, `ADR-0006`,
`THOTH-GQL-OPS-02`, `BE-02`, PR #788, issue #765.

## 9. Tests and checks

Executed locally on `darwin`, against disposable local PostgreSQL 17 and Redis.
No remote environment was contacted.

### 9.1 Formatting, lint, type check

```text
cargo fmt --all -- --check                                          -> clean
git diff --check                                                    -> clean
cargo check --workspace                                             -> Finished, 0 errors
cargo clippy --workspace --all-targets --all-features -- -D warnings -> Finished, 0 warnings
```

### 9.2 Workspace test suite (debug)

```text
cargo test --workspace
-> 1206 passed; 0 failed; 8 ignored   (all suites `ok`)
```

Per-binary, the suites this task adds or touches:

```text
thoth-api            (lib)   996 passed   (973 before; +23 THOTH-GQL-OPS-03)
thoth-api-server     (lib)    10 passed   (  3 before; + 7 THOTH-GQL-OPS-03)
thoth  (bin)                  15 passed   (unchanged: THOTH-GQL-OPS-02 suite)
tests/mutation_guard_fleet_verification
                               5 passed   (new, real-process)
```

The 8 ignored tests are pre-existing and unrelated.

### 9.3 Unit tests — 23, `thoth-api`

Command:

```text
cargo test -p thoth-api --lib --features backend graphql::fleet_verification
```

Result:

```text
test result: ok. 23 passed; 0 failed; 0 ignored; 973 filtered out
```

Covering: reported mode equals the mode built from, for all three modes; no
second independently settable mode; identity present and stable for the process
lifetime; identity resolution source order, trimming and refusal of unusable
values; Linux identity always resolvable; exact record rendering
(minimum disclosure); round trip through the command path's own mode parser;
recovery from a prefixed log line; unattributable observation → `UNKNOWN`;
malformed/forged records refused; identity cannot carry a payload; consistent;
mixed; uncovered member; `UNKNOWN` ≠ `OFF`; pre-guard instance; partial
agreement; unaccounted evidence; ambiguous/empty enumeration; contradictory
observations; silent-adoption divergence; unknown is not a divergence;
repeatability and enumeration ordering; store availability follows from the
reported mode alone.

### 9.4 Server-level and security tests — 7, `thoth-api-server`

Command:

```text
cargo test -p thoth-api-server
```

Result:

```text
test result: ok. 10 passed; 0 failed; 0 ignored
```

- `the_observed_mode_is_the_mode_the_request_path_actually_uses` — one stored
  value, three modes, real routes; the request path's own behaviour matches the
  observed mode, and store availability follows from it;
- `the_record_reports_the_stored_mode_for_every_mode`;
- `observing_is_read_only_stable_and_concurrency_safe` — 16 concurrent observers
  of one shared value return one identical answer; the stored mode is unchanged
  by having been observed;
- `observing_needs_no_database_and_no_configuration` — no pool is constructed
  and no environment is read;
- **AC-11.1** `no_public_route_response_varies_with_the_effective_mode` —
  `GET /`, `GET /graphiql`, `GET /graphql`, `GET /schema.graphql` and
  `POST /graphql` are captured in all three modes and compared **byte for byte**
  (status, headers with `date` excluded, and body). This is stronger than a
  token search: a response that varied with the mode *at all* would fail;
- **AC-11.1** `no_public_route_discloses_a_mode_token_or_the_observation_record`
  — the same routes in all three modes disclose none of `OFF`, `OBSERVE`,
  `ENFORCE`, the record tag, `mutationGuardMode`, `mutation_guard_mode` or
  `THOTH_GRAPHQL_MUTATION_GUARD_MODE`, in body or headers;
- **AC-11.1** `the_public_listener_exposes_no_route_for_the_observation` —
  `/guard-mode`, `/effective-mode`, `/mutation-guard-mode`, `/admin/guard-mode`,
  `/health`, `/status` and `/metrics` are all `404`, disclosing nothing.

The three pre-existing `THOTH-GQL-BATCH-01` handler tests pass unchanged.

Note on scope of AC-11.1: the mechanism adds no public surface and no public
disclosure. The pre-existing fact that `ENFORCE` declines a *colliding* mutation
is approved `THOTH-GQL-BATCH-01` guard behaviour, unchanged by this task, and is
not a surface this task introduces.

### 9.5 Real-process integration tests — 5, root package

Command:

```text
cargo test --test mutation_guard_fleet_verification
cargo test --release --test mutation_guard_fleet_verification
```

Result, both profiles:

```text
test result: ok. 5 passed; 0 failed; 0 ignored
```

Each test starts an actual `thoth` process on the real `start graphql-api`
command path, with an unreachable database URL and a deliberately invalid
private key, and recovers the record from that process's own log stream:

- `a_real_process_reports_the_effective_mode_it_actually_computed` — `OFF`,
  `OBSERVE` and `ENFORCE`, one record per process, no field beyond mode and
  identity;
- `a_single_enumerated_instance_running_one_real_process_is_verifiable`;
- `real_records_that_disagree_about_one_enumerated_member_establish_nothing`;
- `silent_adoption_is_caught_because_the_process_reports_what_it_computed`;
- `a_real_process_record_is_reproducible_across_restarts`.

The unreachable database URL is itself evidence that the record path performs no
database access: the record is emitted regardless.

### 9.6 Regression

```text
THOTH-GQL-BATCH-01 guard/batching suites
  cargo test -p thoth-api --lib --features backend batching
  -> 93 passed; 0 failed

THOTH-GQL-OPS-02 mode-control suite, DEBUG
  cargo test --bin thoth
  -> 15 passed; 0 failed

THOTH-GQL-OPS-02 mode-control suite, RELEASE
  cargo test --release --bin thoth
  -> 15 passed; 0 failed
```

The `THOTH-GQL-OPS-02` matrix holds unchanged in both profiles: `OFF -> OFF`,
`OBSERVE -> OBSERVE`, `ENFORCE -> ENFORCE`, unset `-> OFF`, invalid value `->`
startup failure, `init` registers the argument and declares the `OFF` default,
migrations run first, and a migration failure aborts startup before the API
starts. `store_availability_is_derived_only_from_enforce` passes in both
profiles. None of those tests was modified.

Additionally verified by real binary, both profiles, in the local manual
verification: an unset variable yields `mode=OFF`.

### 9.7 Generated SDL

Built at the authorized base **before** any change, and again at the
implementation head. `thoth-client/assets/schema.graphql` is build-generated and
gitignored, so `git status` is not a valid check; the artefacts were hashed
directly.

```text
base 2bec75e6  sha256 1e08b46b565ef719c404bbe6b3131e6a733df09c7abdc4538b66c2b24d2d899c  160799 bytes
head            sha256 1e08b46b565ef719c404bbe6b3131e6a733df09c7abdc4538b66c2b24d2d899c  160799 bytes
```

**Byte-identical.** The public GraphQL schema is unchanged.

### 9.8 Database, migration and schema

```text
database migration added                 NO
database schema change                   NO
data change                              NO
migration execution semantics changed    NO
public GraphQL schema change             NO
generated SDL change                     NO
```

No file under `thoth-api/migrations/` or `db/` is touched. The mechanism opens
no connection and holds no persistent state.

### 9.9 Concurrency, read-only and side-effect freedom

Observing reads an already-computed `Copy` value out of a shared `Data` and
resolves a cached identity. It performs no write, no allocation of shared state,
no configuration parse, no environment read and no I/O beyond the one-time
`OnceLock` host-name read. `observing_is_read_only_stable_and_concurrency_safe`
drives 16 concurrent observers and asserts one identical answer and an unchanged
stored mode. `verify_fleet` is a pure function;
`verification_is_repeatable_and_order_independent_in_its_conclusion` asserts
equal results for repeated calls and enumeration-ordered output regardless of
collection order.

### 9.10 Request-path performance

The `POST /graphql` handler is **unchanged** — byte for byte, no line of it
appears in the diff. The record is emitted once per process, before
`HttpServer::new` is called, so it is not on any request path at all. The only
adjacent change moves `Data::new(mutation_guard_mode)` out of the per-worker app
factory, which means all workers now share one allocation instead of each
constructing its own; this removes work rather than adding it.

No request-path measurement is offered, because there is no request-path code
change to measure. `ADR-0006` section 7.2.3 guard performance evidence remains
the activation gate's, not this task's.

## 10. Manual verification

Environment: local developer machine, disposable local PostgreSQL 17 and Redis.
No remote environment, no orchestrator, no deployment, no credential.

Steps and observed result — a real debug binary, one process per mode:

```text
$ ./target/debug/thoth start graphql-api --mutation-guard-mode OFF ...
[... INFO  thoth_api_server] THOTH_MUTATION_GUARD_EFFECTIVE_MODE mode=OFF

$ ./target/debug/thoth start graphql-api --mutation-guard-mode OBSERVE ...
[... INFO  thoth_api_server] THOTH_MUTATION_GUARD_EFFECTIVE_MODE mode=OBSERVE

$ ./target/debug/thoth start graphql-api --mutation-guard-mode ENFORCE ...
[... INFO  thoth_api_server] THOTH_MUTATION_GUARD_EFFECTIVE_MODE mode=ENFORCE

$ ./target/debug/thoth start graphql-api ...          (variable unset)
[... INFO  thoth_api_server] THOTH_MUTATION_GUARD_EFFECTIVE_MODE mode=OFF
```

On this platform no host name source is published, so `instance` is absent and
the observation is unattributable — the specified `UNKNOWN` path, exercised
end to end by the real-process tests. On Linux, which is where the service is
containerised, `a_linux_process_can_always_identify_itself` asserts the identity
always resolves; that assertion runs in CI.

Locally demonstrated mixed fleet and incomplete coverage: two real processes in
different modes, enumerated as one member, are `NOT ESTABLISHED` with the member
`UNKNOWN`. A genuinely multi-host mixed fleet cannot be produced on one machine —
two processes on one host share one host name — so `MIXED` and multi-member
incomplete coverage are demonstrated over records in the real record format
through the real parser, in the verifier suite. Proving the mechanism against a
**real** deployed fleet belongs to `THOTH-GQL-OPS-04` and was not attempted.

Confirmed:

```text
environment transitioned                        NONE
mode set in any real environment                NONE
deployment performed                            NONE
deployment workflow dispatched                  NONE
deployment credential used                      NONE
real fleet verified                             NONE
orchestrator queried                            NONE
production configuration read                   NONE
```

## 11. CI

CI status: see the pull-request record for the exact head.
Expected workflows: `build-test-and-check` (`classify`, `build`, `test`,
`lint`, `format`), `run-migrations`, `check-changelog`,
`publish-to-dockerhub`.

Because this pull request changes runtime code, an unexpected classifier skip of
build, test, lint or format is to be investigated rather than read as success.
No CI evidence is inherited from PR #798 or from any earlier SHA.

Run identifiers and job-by-job conclusions are recorded on the pull request and
in the handoff.

## 12. External evidence and the secret boundary

External deployment facts relied on: **NONE**. No fact about the load-balancer
arrangement, the rolling-replacement semantics or the autoscaling model was
re-confirmed at this task's execution time; the mechanism was designed and
tested entirely in-repository against locally reproducible conditions, exactly
as section 6.6 anticipates. No section 6.6 Route A or Route B evidence was
therefore required or requested, and no criterion is `BLOCKED` for want of it.

Explicitly:

- the implementing agent performed **no** read of secret-bearing production
  configuration, by any route, narrow or otherwise;
- the private authoritative deployment source was **not** read and **not**
  changed;
- production was **not** accessed;
- no orchestrator or real fleet was queried;
- **no secret material was exposed** to the implementing agent during this task.

**No AI agent or model of any role, family or session** — the implementing agent
or any other — performed a deployment in any environment, production or not,
dispatched a deployment workflow, transitioned a mode in a real environment,
used a deployment credential, or invoked deployment automation in place of an
authorized human/operator. All testing was local, disposable and in CI, which
this criterion does not restrict.

`THOTH-GQL-OPS-03` section 6.6.1's recorded **control limitation** — that
`THOTH-GQL-OPS-01` section 2.2.5's weaker scoped-read rule must be corrected
before any successor requiring secret-bearing production-source access is
authorized — remains **OPEN**, owned by the CTO / control owner, and is not
closable by an implementing agent. It was not engaged by this task.

## 13. Acceptance criteria

| AC | Status | Evidence |
|---|---|---|
| AC-1 | PASS | One stored `Data<MutationGuardMode>`, read for the record and installed as `app_data`; no re-derivation, no second signal. Sections 4.1, 4.2; `the_observed_mode_is_the_mode_the_request_path_actually_uses`, `the_reported_mode_is_the_mode_the_observation_was_built_from` |
| AC-2 | PASS | `a_real_process_reports_the_effective_mode_it_actually_computed`, `the_record_reports_the_stored_mode_for_every_mode` |
| AC-3 | PASS | `instance` = OS host name = orchestrator-assigned instance name; section 5; `a_linux_process_can_always_identify_itself`, `a_single_enumerated_instance_running_one_real_process_is_verifiable` |
| AC-4 | PASS | `verify_fleet` requires the orchestrator enumeration as an argument and answers for every member; a sampled response cannot produce that input. Section 6.1; `partial_agreement_is_not_fleet_consistency` |
| AC-5 | PASS | `a_fully_covered_population_disagreeing_is_mixed` |
| AC-6 | PASS | `silent_adoption_is_caught_because_the_process_reports_what_it_computed` (real process, deliberate divergence), `silent_adoption_is_visible_as_a_divergence_from_declared_intent`. The closed `THOTH-GQL-OPS-02` defect is not recreated — section 7.2 |
| AC-7 | PASS | `MemberMode::Unknown` vs `Established(..)` is a distinction in the result type; `unknown_is_structurally_distinct_from_off_and_never_decays_into_it`, `a_pre_guard_instance_is_reported_unknown_rather_than_off` |
| AC-8 | PASS | The `graphql` handler is unchanged; `no_public_route_response_varies_with_the_effective_mode` compares status, headers and body byte for byte across all three modes; the three pre-existing handler tests pass unchanged |
| AC-9 | PASS | `mutation_guard.rs`, `batching.rs` and the loader store are untouched; 93 batching/guard tests pass; `store_availability_is_derived_only_from_enforce` passes in both profiles; `store_availability_follows_from_the_reported_mode_alone` |
| AC-10 | PASS | Two-field observation type; `a_record_carries_only_the_effective_mode_and_the_correlation_identity`, `an_identity_cannot_be_made_to_carry_a_payload`, `no_public_route_discloses_a_mode_token_or_the_observation_record` |
| AC-11 | PASS | Sections 3.1–3.4 record the mechanism, the rejected alternatives and the disclosure assessment |
| AC-11.1 | PASS | No HTTP surface is added at all; three negative suites in section 9.4 |
| AC-12 | PASS | Section 9.9 |
| AC-13 | PASS | Section 9.7 (SDL byte-identical) and 9.8 |
| AC-14 | PASS | No production configuration value, secret or resource identifier appears in the diff. Test values are `unused`, `test-access-key`, `127.0.0.1:1` and `https://api.test.invalid` |
| AC-15 | PASS | Section 10; no mode set anywhere, no environment transitioned |
| AC-16 | PASS | CG-13 `OPEN`, runtime-operations gate `NOT SATISFIED`; `control-gaps.md` and the control record are untouched, and no claim of fleet verification is made anywhere in this report |
| AC-17 | PASS | The runbook remains `PROVISIONAL` and not executable; section 0.2's two-part status is unchanged |
| AC-18 | PASS | `OBSERVE`, `ENFORCE` and `BE-02` remain `NOT AUTHORIZED`; PR #788 and issue #765 untouched |
| AC-19 | PASS | `THOTH-GQL-OPS-04` not implemented, branch absent; `THOTH-GQL-OPS-02` neither reopened nor modified, and nothing in the diff describes it as unimplemented or describes `init` as ignoring the guard mode |
| AC-20 | PASS (not engaged) | No external deployment fact was relied on; section 12 |
| AC-21 | PASS | Section 12 |
| AC-22 | PASS | Section 6.5 |
| AC-23 | PASS | Section 5, field by field, with exclusions |

No criterion was weakened, reinterpreted or silently narrowed.

## 14. Rollout and rollback

Initial state after merge: unchanged. The effective mode becomes *observable*.
No mode is changed, no environment is transitioned, no fleet is verified.

```text
guard-enabled candidate default   OFF, loader store unavailable
environments transitioned         none
production request acceptance     unchanged
runtime-operations gate           NOT SATISFIED
```

Activation required: none by this task, and none authorized by it. `OFF ->
OBSERVE` and `OBSERVE -> ENFORCE` each remain subject to their own separate
explicit CTO production activation approval.

Feature flag/configuration: none introduced. The mechanism has no configuration
of its own. It is emitted at `info`, which is the server's own default filter; a
deployment that suppresses `info` logging suppresses the record, and the affected
instances then verify as `UNKNOWN` — fail-closed, never `OFF`.

Migration sequence: none.

Rollback: revert the merge commit. Because the mechanism is inert with respect
to request acceptance, the revert is a no-op for production behaviour — it
removes the ability to observe the mode, not the mode itself. No persistent
state, no external side effect, nothing to repair.

## 15. Known limitations and deferred work

- **No fleet has been verified.** This task delivers the verifier.
  `THOTH-GQL-OPS-04` attempts the fleet, and it is not implemented here.
- Collection is out of band and manual by design: the verifier takes an
  enumeration and observations as inputs and does not itself talk to an
  orchestrator or a log API. Wiring it to a specific orchestration plane needs
  facts and credentials this task may not have, and belongs to
  `THOTH-GQL-OPS-04`.
- A genuinely multi-host mixed fleet cannot be produced on one machine, so
  `MIXED` over real processes is demonstrated over real records rather than over
  real hosts. Section 10.
- The identity sources are Linux host-name files. On a platform publishing
  neither, observations are unattributable and resolve to `UNKNOWN` — correct
  and specified, but it means local developer runs exercise the `UNKNOWN` path.
- Suppressing `info` logging suppresses the record. This is fail-closed, and it
  is a property an operator can check, but it is a real operational
  precondition for `THOTH-GQL-OPS-04` to note.

## 16. Unresolved issues

- NONE within this task's boundary. The `THOTH-GQL-OPS-03` section 6.6.1 control
  limitation remains `OPEN` and is not closable by an implementing agent.

## 17. Agent self-assessment

The implementing agent does not approve its own work. Fresh independent
exact-head review is required, and a separate explicit CTO merge authorization
remains required after it.

Suggested review focus:

- **the same-value claim**: that `Data::new` hoisting genuinely gives the record
  and the request path one allocation, and that nothing can set a second mode;
- **the disclosure boundary**: that a log record on the process's own stream is
  accepted as the approved "administrative/orchestration-plane or equivalent
  out-of-band per-instance mechanism", and that this reviewer agrees no public
  surface was added;
- **the correlation identity**: whether the OS host name is the right minimum,
  whether the optional/`UNKNOWN` behaviour on platforms without it is acceptable,
  and whether any excluded field is in fact necessary;
- **fail-closed completeness**: whether every path to `Consistent` genuinely
  requires complete coverage, including the unattributed-observation and
  ambiguous-enumeration cases;
- **the silent-adoption fixture**: whether the argv-over-environment divergence
  is accepted as deliberate construction rather than as a dependence on the
  closed `THOTH-GQL-OPS-02` defect;
- **the runbook edit**: whether recording the concrete mechanism in section 4.0
  is within this task's permitted documentation scope, and that nothing in it
  lifts `PROVISIONAL` or claims a gate satisfied.
