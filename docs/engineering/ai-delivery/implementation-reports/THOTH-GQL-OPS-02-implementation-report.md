# THOTH-GQL-OPS-02 Implementation Report

Implementation of the approved
[`THOTH-GQL-OPS-02`](../tasks/THOTH-GQL-OPS-02.md) specification: the
mutation-guard **mode-control path**.

**The repository-local mechanism is complete and fully tested.** The two
*external* deployment facts the specification requires to be re-confirmed at this
task's own execution time were initially unobtainable and recorded `BLOCKED`;
authorized **Route B** operator evidence supplied on **2026-08-11** has since
supplied both, and they are now `PASS`. See sections 8.1 and 11.1.

**AC-18 has been disposed of by the control owner.** During a CTO-directed
control review, a reviewer-side read path unexpectedly encountered secret
material. This report previously recorded AC-18 `BLOCKED - control-owner
disposition required` rather than self-certifying it. The CTO/control owner has
since issued that disposition and determined **AC-18: PASS** for this execution.
The review-time exception remains recorded, and the disposition creates no
standing waiver. See sections 8.2, 8.2.1 and 11.1.

```text
capability gap 1 (mode-control path):  CLOSED in-repository
AC-17 (external re-confirmation):      PASS - Route B operator evidence,
                                       supplied 2026-08-11
section 2 container-command re-check:  PASS - same Route B evidence
AC-18 (no-AI-agent boundary):          PASS - explicit CTO/control-owner
                                       disposition, PR #797 comment
                                       5251314845 (section 8.2.1)

CG-13:                                 OPEN
Runtime-operations gate:               NOT SATISFIED
Runbook:                               PROVISIONAL
OBSERVE:                               NOT AUTHORIZED
ENFORCE:                               NOT AUTHORIZED
BE-02 runtime:                         NOT AUTHORIZED
OPS-03 / OPS-04:                       NOT IMPLEMENTED, no branches
```

The new evidence authorizes **no** runtime transition. Making the mode settable
is still not setting it.

## 1. Repository state

Repository: `thoth-pub/thoth`
Programme: Shared Thoth GraphQL / Backend Architecture
Task ID: `THOTH-GQL-OPS-02`
Approved specification: [`THOTH-GQL-OPS-02`](../tasks/THOTH-GQL-OPS-02.md),
reachable from `develop` at the authorized base
Risk: HIGH
Workflow: STANDARD
Base branch: `develop`
Authorized exact base: `e2a44c54bac49079e3ee18b65af3336838023417`
PR target: `develop`
Programme integration branch: None
Task branch: `feature/shared-architecture/graphql-guard-mode-entrypoint`
Dependencies: `ADR-0006` approved; `THOTH-GQL-BATCH-01` merged;
`THOTH-GQL-OPS-01` merged (PR
[#793](https://github.com/thoth-pub/thoth/pull/793)); specification approved and
implementation authorized by the CTO
Live review, readiness, authorization, exact-head and merge evidence:
the GitHub pull request for this branch
Expected branch deletion after merge: YES
Final programme PR required: NO
Implementing model: Claude Opus 5
Reasoning level: HIGH
Independent reviewer: an independent model family that did not author the
implementation

**Durable rule, independent of lifecycle state.** The implementing agent may not
approve or merge its own work, and did not: it opened a draft pull request and
left review, readiness, merge authorization and merge to the separate actors who
own those decisions. Current lifecycle state is live GitHub evidence under
[`ADR-0005`](../../decisions/ADR-0005-terminal-merge-evidence.md) and is
deliberately not transcribed here.

### 1.1 Pre-implementation verification

Performed before the branch was created:

```text
git fetch origin --prune
git rev-parse origin/develop
  -> e2a44c54bac49079e3ee18b65af3336838023417   MATCHES the authorized base

git status --short
  -> clean

git branch -a --list '*graphql-guard-mode-entrypoint*'
git ls-remote --heads origin \
  'refs/heads/feature/shared-architecture/graphql-guard-mode-entrypoint'
  -> absent locally AND remotely; the branch was created by this task

gh pr view 793
  -> MERGED; merge commit e2a44c54bac49079e3ee18b65af3336838023417,
     which is the authorized base itself
```

CTO specification approval and implementation authorization were verified as
GitHub lifecycle evidence: PR #793 comment
[`5245229651`](https://github.com/thoth-pub/thoth/pull/793#issuecomment-5245229651),
which names this exact base and this exact branch, and which explicitly does not
authorize deployment, `OBSERVE`, `ENFORCE`, `BE-02`, any production or runtime
transition, production-secret access, or any change outside OPS-02.

No newer authoritative repository decision contradicts the specification: the
control record, the runbook and `control-gaps.md` at this base still record
capability gap 1 as open, CG-13 as `OPEN` and the runtime-operations gate as
`NOT SATISFIED`.

## 2. Scope confirmation

Implemented objective: make `THOTH_GRAPHQL_MUTATION_GUARD_MODE` actually
consumable on the production-applicable command path (`init`, the image's default
command), in both build profiles, while preserving every existing `init`
migration and startup semantic exactly.

Out-of-scope changes made: NONE.

### 2.1 The defect, re-derived at this exact base

The specification's narrative was **not** taken on trust. Re-derived directly
from the code and the pinned dependency at `e2a44c54`:

```text
Dockerfile                     CMD ["init"], built with cargo build --release
src/bin/commands/mod.rs        INIT registered 11 arguments and NOT
                               arguments::mutation_guard_mode()
src/bin/thoth.rs               init dispatches into the SAME handler as
                               start graphql-api
src/bin/commands/start.rs      that handler reads
                               get_one::<String>("mutation-guard-mode")
                                 .unwrap_or("OFF")
Cargo.lock                     clap 4.6.1 / clap_builder 4.6.0
```

The profile-dependent half was confirmed in the vendored source rather than
inferred:

```text
clap_builder-4.6.0/src/parser/matches/arg_matches.rs
  fn verify_arg(&self, _arg: &str) -> Result<(), MatchesError> {
      #[cfg(debug_assertions)]
      { ... return Err(MatchesError::UnknownArgument {}); }
      Ok(())
  }

clap_builder-4.6.0/src/parser/error.rs
  MatchesError::unwrap  ->  panic!("Mismatch between definition and
                                    access of `{id}`. ...")
```

So `get_one` on an **unregistered** argument returned `Err(UnknownArgument)` and
panicked under `cfg(debug_assertions)`, and in a release build returned
`Ok(None)`, which `.unwrap_or("OFF")` silently converted into `OFF`. The
`value_parser` never ran on that path either, so an invalid value was silently
accepted as `OFF` instead of failing startup.

**Confirmed, not refuted.** The diagnosis holds exactly at this base.

## 3. Commits

```text
Exact commit sequence and final head:
the GitHub pull request for this branch is terminal authority.
```

No SHA is transcribed here: a SHA written into a file on the branch is
invalidated by the act of writing it, and `ADR-0005` makes the pull-request
record the authority for lifecycle facts of this kind.

## 4. Files changed

All changes are in the binary crate's CLI layer. **Zero** migration, schema,
model, GraphQL, policy, `Cargo`, `Dockerfile` or workflow files.

- `src/bin/commands/mod.rs`
  - reason: register the existing mutation-guard argument on `INIT`, the
    production-applicable command; and factor the `init` sequence into
    `run_init` so its ordering and failure guarantees are directly testable.
  - behavioural effect: the `init` command now accepts
    `--mutation-guard-mode` / `THOTH_GRAPHQL_MUTATION_GUARD_MODE`, applies its
    `value_parser` and its declared `OFF` default. `run_init` is semantically
    identical to the previous inline sequence.
- `src/bin/commands/start.rs`
  - reason: extract the existing mode resolution into
    `mutation_guard_mode(&ArgMatches)` so tests exercise the exact accessor the
    running server uses rather than a copy of it.
  - behavioural effect: none. The expression is moved verbatim; `graphql_api`
    calls it.
- `src/bin/thoth.rs`
  - reason: dispatch `init` through `commands::run_init`, and add the
    `THOTH-GQL-OPS-02` test module.
  - behavioural effect: none in `main`. Migrations still run first and startup
    still aborts if they fail.

## 5. Implementation decisions

1. **Mechanism: register the existing argument on `init`.** The single line
   `.arg(arguments::mutation_guard_mode())` on `INIT` is the smallest change that
   satisfies the whole specification, because `init` already dispatches into the
   same handler as `start graphql-api`. Registering the argument simultaneously
   fixes all four required behaviours: the `value_parser` now runs (invalid ->
   startup failure), the declared `default_value("OFF")` now applies (unset ->
   `OFF`), the `.env()` binding now applies (`OBSERVE`/`ENFORCE` are honoured),
   and `verify_arg` now succeeds (no debug panic). No new argument, no new
   environment variable, no parallel resolution path and no duplicated
   definition was introduced.
2. **Rejected: a second, `init`-specific argument or a bespoke environment
   read.** Either would duplicate the definition and allow the two paths to
   drift, which is the class of defect this task exists to remove.
3. **Rejected, and never offered: a production container-command override.**
   Binding classification reproduced in section 5.1. It is out of bounded scope,
   was not implemented, and was not proposed as an alternative or fallback.
4. **Argument position.** Placed after `zitadel_url()` and before the AWS
   arguments, mirroring `start graphql-api` exactly, so the two registrations
   read identically.
5. **`mutation_guard_mode` extracted in `start.rs`.** Required so the tests
   assert against the production accessor. Behaviour-preserving.
6. **`run_init` extracted in `commands/mod.rs`.** Required so the specification's
   integration assertions — migrations run *first*, and a migration failure
   aborts startup — can be proven without a database or a bound socket.
   Behaviour-preserving.
7. **Test surfaces.** `clap`'s `Arg::env()` captures the environment value when
   the `Arg` is **constructed**, and the commands are `lazy_static`. A running
   process therefore reads the variable exactly once, which is correct for the
   binary, but means in-process mutation cannot vary it across tests. The matrix
   is consequently pinned on two surfaces — the real `INIT` command driven by
   argv, and a freshly constructed command carrying the same
   `arguments::mutation_guard_mode()` driven by the environment variable — and
   the environment path is additionally verified end-to-end against real release
   and debug binaries in section 10.

Deviations from the specification: NONE.

### 5.1 The production container-command override: binding classification

Reproduced because an override is mentioned at all:

```text
An explicit production command override is NOT an interchangeable
feature-local fix. It changes the current `init` execution path by removing
migration execution from deployment, and therefore requires separate
migration/deployment-control analysis and approval under the broader CG-13
migration/deployment problem.
```

No override was made, specified, offered as an option or relied upon. The fix is
entirely in-repository and preserves migration execution on the `init` path.

## 6. Database and migration effects

Migration added: NO

```text
Database/data change:                        NONE
Migration files:                             NONE
Migration execution semantics changed:       NONE
GraphQL schema change:                       NONE
Public API change:                           NONE
```

`init` runs `commands::run_migrations(arguments)?` and only then
`commands::start::graphql_api(arguments)`. This task moves that sequence into
`commands::run_init`, which applies `?` to the migration step exactly as before,
so ordering and abort-on-failure are preserved. Both are asserted by tests
(section 9) rather than described.

## 7. API and compatibility effects

GraphQL/API changes: NONE
Generated schema/client updates: NONE — the generated SDL is byte-identical
(section 9)
Backwards compatibility: see the per-deployment-class assessment below
Deprecations: NONE
Cross-repository dependencies: none created

### 7.1 Compatibility — this change is NOT behaviour-neutral

Stating otherwise would misrepresent the fix as cosmetic and hide exactly the
behaviour the task exists to introduce.

| `THOTH_GRAPHQL_MUTATION_GUARD_MODE` on `init` | before (release) | after | changed? |
|---|---|---|---|
| unset | `OFF` | `OFF` | no |
| `OFF` | `OFF` | `OFF` | no |
| `OBSERVE` | silently `OFF` | `OBSERVE` | **yes, intentionally** |
| `ENFORCE` | silently `OFF` | `ENFORCE` | **yes, intentionally** |
| invalid value | silently `OFF`, startup succeeds | **startup failure** | **yes, intentionally** |

Per deployment class:

```text
Any `init` invocation that leaves the variable unset, or sets it to OFF:
    UNCHANGED -> OFF.

Any `init` invocation already supplying OBSERVE or ENFORCE:
    INTENTIONALLY CHANGES BEHAVIOUR, from ignored/OFF to the supplied mode.
    Such an invocation is today silently not doing what it says; after this
    task it does. This is the defect being fixed, not a regression.

Any `init` invocation supplying an invalid value:
    INTENTIONALLY CHANGES BEHAVIOUR, from silent OFF with a successful
    startup to a startup failure. This aligns `init` with the existing
    `start graphql-api` behaviour and removes a silent-misconfiguration
    class.

Known current Production and Test deployments:
    UNCHANGED.

    THOTH_GRAPHQL_MUTATION_GUARD_MODE is absent in both current
    deployment definitions, confirmed through authorized Route B
    operator evidence at this task's review time (2026-08-11).

    Both therefore take the unset -> OFF row after OPS-02.
```

That conclusion is **not** inherited from the specification. It rests on
authorized operator-supplied evidence obtained at review time and recorded in
section 8.1.

**The deployment-facing consequence is not glossed.** Because a previously
ignored invalid value will begin to fail startup, an environment that happens to
carry a malformed value would start failing to deploy after this change. The
Route B evidence establishes that neither current Production nor current Test
carries such a value, so no currently identified environment is affected. The
claim extends no further than the deployment classes the operator evidence
actually covers.

None of this activates anything: the default remains `OFF`, and making the mode
settable is not setting it.

## 8. Authorization and security

Authorization paths changed: NONE. `thoth-api/src/policy.rs` is untouched, and
the guard mode participates in no authorization decision.
Roles/scopes involved: none created or altered.
Negative authorization tests: the existing suites are unchanged and pass; the
guard mode alters no authorization outcome.
Secret or personal-data handling: the new code path reads one non-secret
enumerated value restricted to `OFF`/`OBSERVE`/`ENFORCE`, and logs nothing. A
test asserts that the invalid-value startup failure leaks no value bound to
`DATABASE_URL`, `PRIVATE_KEY` or `AWS_SECRET_ACCESS_KEY`.

### 8.1 External deployment facts — satisfied by Route B operator evidence

The specification requires two external facts to be **re-confirmed at this
task's own execution time** and explicitly forbids inheriting them:

```text
FACT 1  the production GraphQL API service still supplies no container-command
        override, and so still inherits the image default `init`
        -> specification section 2

FACT 2  no environment currently supplies a mutation-guard value that would
        newly fail startup under the section 6.5 change
        -> specification section 6.5 and AC-17
```

#### 8.1.1 Evidence provenance

```text
ROUTE B   Authorized CTO / control-owner operator evidence
          Sanitized, non-secret deployment facts
          Supplied 2026-08-11, at this task's review time

ROUTE A   NOT used. No sanitized metadata-only artefact was available to
          this task, and no Route A success is claimed.
```

The evidence source is an **authorized human/operator**, which section 6.6
Route B admits explicitly. No AI agent is a valid Route B source, and none was
used as one.

#### 8.1.2 The sanitized facts supplied

```text
PRODUCTION GraphQL API
  - deployed with the standard Thoth service deployment template
  - supplies NO container-command override
  - therefore inherits the image default command `init`
  - does NOT configure THOTH_GRAPHQL_MUTATION_GUARD_MODE

TEST GraphQL API
  - same deployment configuration as Production for the relevant facts
  - NO container-command override
  - THOTH_GRAPHQL_MUTATION_GUARD_MODE ABSENT

CONTROL COMPARISON, sanitized
  - another service deployed by the SAME mechanism, the Export API,
    explicitly supplies its command as `start` / `export-api`
  - command overrides are therefore EXPRESSIBLE by this mechanism, so
    their ABSENCE on the GraphQL API is meaningful rather than merely
    unobserved

RESULT   FACT 1  PASS
         FACT 2  PASS  -> AC-17 PASS
```

No secret value was supplied as Route B evidence, and none is recorded here. No
infrastructure definition, resource identifier or configuration value from the
private deployment source has been copied into this repository.

**Coverage check, performed rather than assumed.** AC-17 speaks of "no
environment". Repository evidence — `environments.md` section 1 and the
runtime-operations control record — identifies exactly **two** current Thoth
GraphQL API environments, production and test, and the operator evidence covers
both. The preview/staging environment referenced elsewhere in the programme is a
**downstream** gate that does not yet exist as a deployed GraphQL API
environment; `THOTH-GQL-OPS-04` section 3.1 exists precisely because a
guard-enabled non-production environment must still be created. There is
therefore no current relevant environment left uncovered, and no coverage is
claimed beyond the two classes the operator evidence names.

**Implementing-agent conduct is unchanged by this reconciliation.** The
implementing agent did not read the private authoritative deployment source, by
any route, narrow or otherwise. It opened no secret-bearing production
configuration and encountered no secret material. The facts above reached it as
sanitized operator-supplied statements, and it did not attempt to verify them by
opening protected configuration.

**What this means for the change.** The residual risk section 7.1 names — an
environment carrying a malformed `THOTH_GRAPHQL_MUTATION_GUARD_MODE` beginning to
fail startup — is now closed for both current deployment classes on authorized
evidence. It is not closed by assumption, and it is not extended to environments
the evidence does not cover.

### 8.2 Operational-actor statement, and the AC-18 disposition

Three things are stated separately, because they differ and collapsing them
would misreport the evidence.

**IMPLEMENTATION EXECUTION — every limb satisfied:**

```text
Deployment performed by the implementing agent:              NONE
Deployment workflow / automation dispatched by the
  implementing agent:                                        NONE
Real-environment mode transition by the implementing agent:  NONE
Real fleet state created, manipulated or restored by the
  implementing agent:                                        NONE
Deployment credentials used or held by the implementing
  agent:                                                     NONE
Secret-bearing production configuration read by the
  implementing agent:                                        NONE
Private authoritative deployment source accessed by the
  implementing agent:                                        NONE
Production configuration changed by the implementing agent:  NONE
Production action of any kind by the implementing agent:     NONE
Security issue created or modified:                          NONE
Mode set in any environment:                                 NONE
```

Each line above is scoped to the **implementing agent** deliberately. An
unqualified, lifecycle-wide `NONE` would contradict the control-review exception
recorded immediately below, and this report does not make that claim.

Local, disposable and CI repository testing is ordinary work and is not
restricted by that boundary; all testing in section 9 was local and disposable.

**CONTROL REVIEW — an exception, recorded at the minimum safe level:**

```text
During CTO-directed independent control review, an AI reviewer
unexpectedly encountered secret material while inspecting a protected
deployment source.

The source/read path was stopped immediately.

The material was not used or reproduced.

No credential was used. No deployment was performed or dispatched. No
production configuration was changed. No guard mode was changed.

The source supplied NEITHER OPS-02 external acceptance fact.

The accepted deployment evidence was supplied separately through
authorized Route B operator evidence (section 8.1).

Classification:
    REVIEW-TIME CONTROL/PROCESS EXCEPTION

Route A evidence:
    NONE

Control-owner disposition:
    PR #797 comment 5251314845
```

Deliberately not recorded, here or anywhere in this pull request: the secret
value, its location, credential contents, resource identifiers, account or stack
identifiers, task-definition identifiers, or any other infrastructure detail
derived from that read.

#### 8.2.1 AC-18 — control-owner disposition, not self-certification

The approved AC-18 reads, in full, and is **unchanged**:

> **AC-18** **No AI agent or model of any role, family or session** — the
> implementing agent or any other — performed a deployment in any environment,
> production or not, dispatched a deployment workflow, used a deployment
> credential, invoked deployment automation in place of an authorized
> human/operator, **or read secret-bearing production configuration**. The report
> states this explicitly. Local, disposable and CI repository testing is not
> restricted by this criterion.

An earlier revision of this report recorded
`AC-18: BLOCKED - control-owner disposition required`, because the actor clause
is unrestricted as to role and session and carries no temporal qualifier, so the
implementing agent could neither certify it `PASS` against the review-time
exception nor narrow the wording to resolve the tension itself. Narrowing,
waiving or amending would each have been an implementing agent disposing of a
control question reserved to the control owner.

**The control owner has now exercised exactly that disposition.**

```text
AC-18: PASS

Provenance: explicit CTO / control-owner disposition
            PR #797 comment 5251314845
Determined: 2026-08-11
```

The control owner's stated basis, recorded here without reinterpretation or
broadening: the review inspection was **instructed by the CTO/control owner**;
the reviewer unexpectedly encountered secret material and stopped immediately;
no credential was used; no deployment was performed or dispatched; no production
configuration and no guard mode was changed; the protected source supplied
neither OPS-02 acceptance fact; and the control owner classifies that incident as
a control-review exception arising from their own instruction, **not** as the AI
agent acting as a deployment actor, credential user, or production-configuration
evidence source for `THOTH-GQL-OPS-02`.

**This `PASS` is attributed to the control owner, not self-certified by the
implementing agent.** The implementing agent did not narrow AC-18, did not amend
the specification, and takes no independent position on the criterion's
lifecycle semantics.

**Bounds of the disposition, stated because they bind successors:**

```text
AC-18 remains unchanged and fully binding.

The disposition creates NO standing exception, waiver or authorization
for any AI agent to access secret-bearing production configuration, in
this task or any successor task.

No further AI access to that source is authorized.

The disposition does NOT authorize merge, deployment, OFF -> OBSERVE,
OBSERVE -> ENFORCE, BE-02 runtime, or any other production or runtime
transition.
```

The review-time exception is **not** erased by the disposition: it remains
recorded above, it is still not valid Route A evidence, and the protected source
is still recorded as having supplied no acceptance fact.

## 9. Tests and checks

All commands were run at the branch head, against local disposable PostgreSQL and
Redis services. No command was pointed at a production service or database.

### Formatting

Command:

```text
cargo fmt --all -- --check
git diff --check
```

Result:

```text
cargo fmt --all -- --check   exit 0, no diff reported
git diff --check             exit 0, no whitespace error
```

### Unit tests — debug profile

Command:

```text
cargo test --bin thoth
```

Result:

```text
running 15 tests
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The 15 comprise the pre-existing `test_cli` (`clap`'s `debug_assert()`) plus the
14 added by this task.

### Unit tests — release profile

Command:

```text
cargo test --release --bin thoth
```

Result:

```text
running 15 tests
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

Identical set, identical outcome to the debug run above. The defect was
profile-dependent, so this is load-bearing evidence, not a formality.
```

### Workspace tests

Command:

```text
cargo test --workspace
```

Result:

```text
test result: ok.    0 passed;  0 failed;  0 ignored   (thoth doc/bin harnesses)
test result: ok.   15 passed;  0 failed;  0 ignored   (thoth bin -- this task)
test result: ok.  973 passed;  0 failed;  0 ignored   (thoth-api lib)
test result: ok.   13 passed;  0 failed;  0 ignored   (thoth-api integration)
test result: ok.    3 passed;  0 failed;  0 ignored   (thoth-api-server lib)
test result: ok.    4 passed;  0 failed;  0 ignored
test result: ok.   11 passed;  0 failed;  0 ignored
test result: ok.  144 passed;  0 failed;  0 ignored   (thoth-export-server)
test result: ok.    0 passed;  0 failed;  8 ignored
                 -----------------------------------
TOTAL            1163 passed;  0 failed;  8 ignored

The 8 ignored are pre-existing `#[ignore]` tests, untouched by this task.

The `THOTH-GQL-BATCH-01` guard and batching suites are inside the 973 and pass
unchanged.

Environment note, recorded because it cost a full cycle: the three
`thoth-api-server` handler tests read `TEST_DATABASE_URL` with `std::env::var`
and nothing loads `.env` in that test process, so without it exported they fail
`NotPresent`. That is environmental and cannot be caused by this change --
`thoth-api-server` depends on `thoth-api` and `thoth-errors` only, never on the
root binary crate this task modifies. With `TEST_DATABASE_URL` exported, all
three pass, as recorded above.
```

### Lint/static analysis

Command:

```text
cargo check --workspace
cargo clippy --all --all-targets --all-features -- -D warnings
```

Result:

```text
cargo check --workspace    Finished `dev` profile, exit 0
cargo clippy ...  Finished `dev` profile, exit code 0

No warning and no error attributable to this workspace. The only emitted
warning is the pre-existing future-incompatibility note for the transitive
dependency `proc-macro-error2 v2.0.1`, present at the base and unrelated.
```

### Generated SDL

The generated SDL is written by `thoth-client/build.rs` to
`thoth-client/assets/schema.graphql`, which is **gitignored and
build-generated**, so `git status` is not a valid check. The base was built in a
separate `git worktree` and the two artefacts compared byte-for-byte.

Command:

```text
git worktree add <scratch>/base-sdl e2a44c54bac49079e3ee18b65af3336838023417
cargo check -p thoth-client                       # in each tree
shasum -a 256 thoth-client/assets/schema.graphql  # in each tree
```

Result:

```text
base   e2a44c54  1e08b46b565ef719c404bbe6b3131e6a733df09c7abdc4538b66c2b24d2d899c
head             1e08b46b565ef719c404bbe6b3131e6a733df09c7abdc4538b66c2b24d2d899c
                 160799 bytes in both trees

IDENTICAL. The generated SDL is unchanged.

Structurally this could not have been otherwise, and that was checked
independently: `git diff --stat` between the base and this head over
thoth-api/, thoth-client/, thoth-api-server/, thoth-errors/,
thoth-export-server/, Cargo.toml and Cargo.lock is EMPTY -- the entire diff
is three files under src/bin/. `thoth-client/build.rs` generates the SDL from
thoth-api and never from the root binary crate, so every input to the
generator is byte-identical to the base. The rebuild confirms it empirically.

Note: the base tree must be built with `--workspace`. `cargo check -p
thoth-client` alone fails with 26 `cannot find 'graphql' in crate` errors,
because only the workspace edge enables thoth-api's `backend` feature. That is
pre-existing and unrelated to this task.
```

### Compatibility matrix — pinned per row, both profiles

Each row of section 7.1 has its own test, and the module compiles and runs under
both profiles:

| Case on the production-applicable path | Test | Pinned expectation |
|---|---|---|
| unset | `unset_yields_off` | `MutationGuardMode::Off` |
| `OFF` | `off_yields_off` | `MutationGuardMode::Off` |
| `OBSERVE` | `observe_yields_observe_and_never_off` | `Observe`, asserted **not** `Off` |
| `ENFORCE` | `enforce_yields_enforce_and_never_off` | `Enforce`, asserted **not** `Off` |
| invalid | `invalid_value_fails_startup_and_is_never_coerced_to_off` | `ErrorKind::InvalidValue`; no mode produced |

Supporting tests:

| Test | Property |
|---|---|
| `init_registers_the_mutation_guard_mode_argument` | direct regression test for the exact defect |
| `init_binds_the_documented_environment_variable_and_off_default` | env binding and `OFF` default preserved |
| `resolving_the_mode_on_init_does_not_panic_in_this_profile` | no panic, in whichever profile is running |
| `start_graphql_api_behaviour_is_unchanged` | the other path still resolves the mode |
| `every_other_init_argument_keeps_its_binding` | all 11 pre-existing `init` arguments keep name, env binding and default |
| `store_availability_is_derived_only_from_enforce` | `ADR-0006` invariant 30 |
| `init_runs_migrations_before_starting_the_api` | migrations run **first** |
| `a_migration_failure_aborts_startup_and_the_api_never_starts` | failure aborts; API never starts |
| `an_invalid_mode_error_leaks_no_secret_bearing_value` | no secret value in the startup-failure output |

No existing test was weakened, deleted or relaxed to make the implementation
pass.

## 10. Manual verification

Environment: local developer machine, disposable. No production service,
database or credential was involved, and no environment was transitioned.

Steps and observed results:

```text
Binaries under test:
    target/release/thoth   built with `cargo build --release --bin thoth`
    target/debug/thoth     built with `cargo build --bin thoth`

1. Argument registration, in BOTH profiles -- identical output:

     --mutation-guard-mode <THOTH_GRAPHQL_MUTATION_GUARD_MODE>
       GraphQL mutation request guard mode: OFF, OBSERVE or ENFORCE
       [env: THOTH_GRAPHQL_MUTATION_GUARD_MODE=] [default: OFF]
       [possible values: OFF, OBSERVE, ENFORCE]

   The real binary shows the argument registered on `init`, bound to the
   documented variable, defaulting to OFF, with the value set restricted.

2. Matrix, driven by THOTH_GRAPHQL_MUTATION_GUARD_MODE against a deliberately
   UNREACHABLE database, so that a value which parses must then fail at
   MIGRATIONS -- which is itself the ordering evidence:

   RELEASE                                    DEBUG
   unset           exit=1  accepted, failed   exit=1  accepted, failed
                           at migrations              at migrations
   OFF             exit=1  accepted, failed   exit=1  accepted, failed
                           at migrations              at migrations
   OBSERVE         exit=1  accepted, failed   exit=1  accepted, failed
                           at migrations              at migrations
   ENFORCE         exit=1  accepted, failed   exit=1  accepted, failed
                           at migrations              at migrations
   SOMETHING_ELSE  exit=2  PARSE REJECTED     exit=2  PARSE REJECTED
                           (startup failure)          (startup failure)

   NO PANIC in either profile, on any row. Before this task the same debug
   invocation panicked and the same release invocation silently resolved OFF.

3. Ordering and abort, observed in the real binary: every accepted value fails
   at the migration step and the API never starts, confirming that `init` runs
   migrations FIRST and aborts when they fail.

Why the effective mode value itself is not read back from the running binary:
no surface exposes the effective mode of a serving instance. That absence IS
capability gap 2, owned by `THOTH-GQL-OPS-03`, and this task must not add such
a surface (specification section 8). The manual runs therefore prove
acceptance, rejection, ordering and the absence of panics in real binaries,
while the unit tests resolve the mode value itself through the production
accessor in both profiles.

No production service, database or credential was involved; the database URL
used was deliberately unroutable. No environment was transitioned and no mode
was set anywhere.
```

## 11. CI

Exact-head CI and its per-job PASS / SKIPPED / FAIL classification are recorded
on the pull request for this branch, which is the authority. No CI verdict is
asserted in this file.

### 11.1 Acceptance criteria — AC-1 to AC-18

```text
PASS      18
BLOCKED    0
```

Two criteria reached `PASS` after the original implementation round, and neither
by self-certification. AC-17 was satisfied by the authorized **Route B** operator
evidence of section 8.1. AC-18 was satisfied by the explicit **CTO/control-owner
disposition** of section 8.2.1, PR #797 comment `5251314845`. The review-time
control exception behind AC-18 remains recorded rather than erased.

| Criterion | Status | Evidence |
|---|---|---|
| **AC-1** release build consumes `OFF`/`OBSERVE`/`ENFORCE` on the production-applicable path | PASS | `cargo test --release --bin thoth` matrix rows; manual release-binary run (section 10) |
| **AC-2** absent value yields `OFF` | PASS | `unset_yields_off` in both profiles; declared `default_value("OFF")` pinned |
| **AC-3** invalid value fails startup, never a coerced `OFF` | PASS | `invalid_value_fails_startup_and_is_never_coerced_to_off`; manual run exits non-zero |
| **AC-4** debug/release divergence eliminated, no panic in either profile | PASS | the same module passes under both profiles; `resolving_the_mode_on_init_does_not_panic_in_this_profile` |
| **AC-5** `init` still runs migrations first and aborts on failure | PASS | `init_runs_migrations_before_starting_the_api`; `a_migration_failure_aborts_startup_and_the_api_never_starts`; manual failed-migration run |
| **AC-6** every other `init` argument keeps name, env binding, default, behaviour | PASS | `every_other_init_argument_keeps_its_binding`, covering all 11 |
| **AC-7** `start graphql-api` unchanged | PASS | `start_graphql_api_behaviour_is_unchanged`; its registration is untouched in the diff |
| **AC-8** default remains `OFF`, merged state inert, nothing transitioned | PASS | default pinned; no mode set anywhere in the diff; section 8.2 |
| **AC-9** loader-store availability derived only from the mode, unavailable outside `ENFORCE` | PASS | `store_availability_is_derived_only_from_enforce`; `store_available()` untouched |
| **AC-10** no container-command override made or specified; any mention carries the classification | PASS | section 5.1; no deployment file in the diff |
| **AC-11** no migration, schema, data or public-API change; generated SDL unchanged | PASS | section 6; SDL byte-comparison in section 9 |
| **AC-12** no production configuration value, secret or resource identifier in the diff | PASS | diff is three CLI source files; scope sweep in section 13 |
| **AC-13** CG-13 open, runtime-operations gate `NOT SATISFIED` | PASS | neither document is touched; both still record those states |
| **AC-14** `OBSERVE`, `ENFORCE`, `BE-02` remain `NOT AUTHORIZED`; PR #788 and issue #765 unchanged | PASS | untouched by this PR |
| **AC-15** `THOTH-GQL-OPS-03`/`-04` branches do not exist and neither is implemented here | PASS | `git branch -a` and `git ls-remote` both empty for those names |
| **AC-16** compatibility matrix stated accurately per deployment class, both intentional changes labelled intentional | PASS | section 7.1, now recording current Production and Test as `unset -> OFF` on Route B evidence |
| **AC-17** execution-time re-confirmation that no environment supplies a newly failing value | **PASS** | section 8.1 — authorized **Route B** CTO/control-owner operator evidence supplied 2026-08-11 confirms `THOTH_GRAPHQL_MUTATION_GUARD_MODE` is absent from both current Production and Test GraphQL API deployments. Obtained through the section 6.6 boundary, not inherited from the specification, and not obtained by any widened read. Coverage of "no environment" checked against `environments.md` and the control record: exactly two current GraphQL API environments exist, and both are covered |
| **AC-18** no AI agent of any role performed a deployment, workflow dispatch, credential use or secret-bearing read | **PASS** — by explicit **CTO/control-owner disposition**, PR #797 comment [`5251314845`](https://github.com/thoth-pub/thoth/pull/797#issuecomment-5251314845), **not** self-certified | section 8.2 and 8.2.1. Implementation execution satisfies every limb, scoped explicitly to the implementing agent. The CTO-directed review-time exception remains recorded at minimum safe level; the control owner classified it as a control-review exception arising from their own instruction rather than the AI acting as deployment actor, credential user or production-configuration evidence source. AC-18 is unchanged and fully binding, and the disposition creates no standing waiver and no further AI access |

## 12. Rollout and rollback

Initial state after merge: unchanged. The mode becomes *settable* on the
production-applicable path; it is *not set*.

```text
deployed production release       = pre-guard (no guard mode exists)
guard-enabled candidate default   = OFF, loader store unavailable
environments transitioned         = none
production request acceptance     = unchanged
runtime-operations gate           = NOT SATISFIED
```

Activation required: YES, and unchanged. `OFF -> OBSERVE` and
`OBSERVE -> ENFORCE` each remain subject to their own separate explicit CTO
production activation approval (`ADR-0006` section 7.2.1). This task grants
neither.

Feature flag/configuration: none introduced.

Migration sequence: none.

Rollback/disable procedure: revert the merge commit. Because the merged state
leaves the default `OFF` and sets no mode anywhere, the revert is a no-op for
production behaviour. The guard mode is **not** a kill switch: changing it
requires a configuration change **and** a deployment.

Monitoring required: none added. Observability of the effective mode is
`THOTH-GQL-OPS-03` and is deliberately not absorbed here.

## 13. Known limitations and deferred work

- **AC-18 was disposed of by the control owner, and the exception it arose from
  remains on the record (section 8.2.1).** The `PASS` is the CTO/control owner's
  determination in PR #797 comment `5251314845`, not the implementing agent's.
  AC-18 itself is unchanged and fully binding: the disposition creates **no**
  standing exception, waiver or authorization for AI access to secret-bearing
  production configuration in this or any successor task, and no further AI
  access to that source is authorized. Successor tasks must not cite it as
  precedent.
- **AC-17 and the section 2 container-command re-confirmation are now `PASS`**,
  on authorized Route B operator evidence supplied 2026-08-11 (section 8.1). The
  deployment-facing consequence in section 7.1 is therefore closed for both
  current deployment classes — but only for those two. The claim is not extended
  to any environment the operator evidence does not cover, and it is evidence at
  a point in time: a later change to either deployment definition would need its
  own re-confirmation.
- **Capability gap 2 remains open.** No mechanism proves the effective mode of a
  serving instance. That is `THOTH-GQL-OPS-03`, unimplemented, with no branch.
- **The runtime-operations gate remains `NOT SATISFIED` and CG-13 remains
  `OPEN`.** Making the mode settable is not verifying a fleet, and this task
  closes neither.
- **The mode-transition runbook remains `PROVISIONAL`.** Only `THOTH-GQL-OPS-04`
  may lift that marking, and only on evidence.
- **Pre-existing, out of scope, and reported rather than fixed: `thoth <cmd>
  --help` renders the *values* of env-bound arguments.** `main` calls
  `dotenv::dotenv().ok()` before parsing, and `clap` renders `[env: NAME=value]`
  in help output, so a `--help` invocation prints whatever is set for
  `DATABASE_URL`, `PRIVATE_KEY` and `AWS_SECRET_ACCESS_KEY`, among others. This
  is present at the base, is not introduced or worsened by this task -- the
  guard-mode variable is a non-secret enumerated value -- and fixing it would
  change help output for arguments outside this task's boundary, which the
  specification's non-goals forbid. It is recorded here so it is not lost:
  `Arg::hide_env_values(true)` on the secret-bearing arguments would be the
  bounded remedy, in its own separately specified task. No value observed while
  running the local manual verification appears in this repository, this report,
  the pull request or any commit.
- **The environment binding is read once per process.** That is correct for the
  binary and matches production, but it is why the in-process test matrix uses
  two surfaces (section 5 item 7) and why the environment path is additionally
  verified against real binaries.

## 14. Unresolved issues

1. **NONE outstanding against this task's acceptance criteria.** AC-1 to AC-18
   are `PASS`.
2. **Resolved, and recorded rather than erased:** the AC-18 control question is
   disposed of by the CTO/control owner (section 8.2.1, PR #797 comment
   `5251314845`), and the review-time control exception it arose from remains
   recorded at minimum safe level.
3. **Resolved:** the two external deployment facts of section 8.1, by authorized
   Route B operator evidence supplied 2026-08-11. They are point-in-time evidence
   for the current Production and Test deployment classes, not a standing
   guarantee; a later change to either deployment definition would need its own
   re-confirmation.
4. **Carried forward, outside this task:** the pre-existing `--help`
   env-value disclosure recorded in section 13, which is not introduced or
   worsened here and needs its own bounded task.

## 15. Agent self-assessment

The agent may identify risks but may not approve the task. This implementation
was not self-reviewed: the implementing agent opened a draft pull request and
left readiness, review, merge authorization and merge to the separate actors who
own those decisions.

Suggested review focus:

1. **The AC-18 disposition record (sections 8.2 and 8.2.1).** Confirm the `PASS`
   is attributed to the CTO/control owner's comment `5251314845` and is nowhere
   presented as self-certified; that the review-time exception is still recorded
   rather than erased; that the approved AC-18 wording is quoted unchanged and
   was not narrowed; and that the disposition's stated bounds — no standing
   waiver, no further AI access, no merge or activation authority — are carried
   accurately.
2. **The scoped `NONE` claims (section 8.2).** Confirm every actor line is scoped
   to the implementing agent, and that no unqualified lifecycle-wide "no AI agent
   read secret-bearing configuration" claim survives anywhere in the report, the
   changelog or the pull-request body.
3. **The Route B evidence (section 8.1).** Confirm the evidence is correctly
   attributed to an authorized human/operator rather than to Route A, that no AI
   agent was treated as a Route B source, that the protected source is nowhere
   credited with satisfying AC-17, and that the coverage argument for
   "no environment" — exactly two current GraphQL API environments, both covered
   — matches the reviewer's own reading of `environments.md` and the control
   record.
2. **The mechanism's minimality.** Confirm that registering the existing argument
   is the smallest fix, and that the two extractions (`mutation_guard_mode`,
   `run_init`) are behaviour-preserving rather than refactors of convenience.
3. **Migration semantics.** Confirm from the diff, not from this report, that
   `run_init` applies `?` to the migration step exactly as the previous inline
   sequence did, and that the two tests genuinely pin ordering and abort.
4. **The intentional behaviour changes.** Confirm they are recorded as
   intentional everywhere they appear — report, changelog and PR body — and
   nowhere described as neutral.
5. **Both profiles.** Confirm the matrix genuinely ran under `--release` as well
   as debug, since the defect was profile-dependent.
6. **Boundaries.** Confirm no container-command override, no migration change, no
   schema or SDL change, no mode set anywhere, and no production or
   private-source access.
