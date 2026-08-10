# THOTH-GQL-OPS-01-SPEC Implementation Report

Specification-authoring task for the output task
[`THOTH-GQL-OPS-01`](../tasks/THOTH-GQL-OPS-01.md).

This report covers the authoring of a specification. It does not report an
implementation of that specification, and it records no production action.

## 1. Repository state

Repository: `thoth-pub/thoth`
Workflow: STANDARD
Base branch: `develop`
Base commit: `75f44aabc52d98596ea6ce69ab068b3698fcd524`
PR target: `develop`
Programme integration branch: None
Task branch: `feature/shared-architecture/graphql-runtime-ops-spec`
Head commit: recorded on the pull request; the PR head is the authority
Pull request: opened as **DRAFT**, not approved, not ready for review by the
author, not merged
Expected branch deletion after merge: YES
Final programme PR required: NO
Implementing model: Claude Opus 5
Reasoning level: HIGH

Base verification: `git fetch origin` followed by
`git rev-parse origin/develop` returned
`75f44aabc52d98596ea6ce69ab068b3698fcd524`, which matches the expected exact
starting base. No intervening commits required assessment, and no rebase or
overwrite of concurrent work occurred.

## 2. Scope confirmation

Approved specification: this is the specification-authoring task
`THOTH-GQL-OPS-01-SPEC`; its output is the `THOTH-GQL-OPS-01` specification.

Implemented objective: produce a bounded, evidence-driven operational-control
specification establishing how the merged mutation guard's
`THOTH_GRAPHQL_MUTATION_GUARD_MODE` is configured, changed, deployed,
propagated, verified fleet-wide, detected when partially applied, rolled back,
authorized and evidenced — without activating anything.

Out-of-scope changes made: NONE.

## 3. Commits

Recorded on the pull request. This report is not the authority for commit
identifiers; the PR is.

## 4. Files changed

- `docs/engineering/ai-delivery/tasks/THOTH-GQL-OPS-01.md`
  - reason: the specification itself.
  - behavioural effect: none. `Status: DRAFT`, `Implementation: NOT AUTHORIZED`.

- `docs/engineering/ai-delivery/implementation-reports/THOTH-GQL-OPS-01-SPEC-implementation-report.md`
  - reason: this report.
  - behavioural effect: none.

- `docs/engineering/ai-delivery/README.md`
  - reason: add the new task to the document index so it is discoverable.
  - behavioural effect: none.

- `docs/engineering/repository-map/control-gaps.md`
  - reason: add a durable reference from CG-13 to the proposed bounded
    feature-specific successor.
  - behavioural effect: none. **CG-13 remains OPEN.** No closure criterion was
    added, and no part of CG-13 was marked resolved.

- `docs/engineering/decisions/decision-register.md`
  - reason: name the bounded successor in the already-recorded dependency
    sequence so the sequence stays durable and accurate.
  - behavioural effect: none. The sequence, the gates and the authorization
    boundaries are unchanged; only the previously unnamed successor is now named.

- `CHANGELOG.md`
  - reason: mandatory under `AGENTS.md` section 13.
  - behavioural effect: none.

Runtime files changed: **NONE.** No Rust, SQL, migration, `schema.rs`,
`policy.rs`, `Cargo.toml`, `Cargo.lock`, `Dockerfile`, `docker-compose.yml`,
`Makefile` or `.github/workflows/` file appears in the diff.

## 5. Implementation decisions

1. **The specification is bounded to mode control.** CG-13's migration
   execution, restore verification and general approver mapping are left open and
   are explicitly listed as non-goals. Disposition class **A** is the default the
   specification mandates.

2. **Every operational statement carries an evidence class.** The specification
   introduces a binding `[REPO]` / `[EXTERNAL]` / `[UNVERIFIED]` labelling rule,
   and makes `[UNVERIFIED]` a criterion failure rather than a permitted outcome.
   This exists to prevent the specific failure mode `ADR-0006` sections 7.2.4 and
   7.3 had to withdraw twice — asserting operational properties that were never
   verified.

3. **Discovery findings are recorded as findings, not as conclusions the future
   task may skip.** Section 2.2 of the specification records what was established
   at this exact base and requires the implementing agent to re-derive each
   finding against its own base.

4. **No production platform was inferred from the Docker image or the release
   workflow.** The authoritative deployment source was identified from
   organisation evidence and inspected read-only, and the specification records
   only what that source establishes. No runtime mechanism was guessed, assumed
   or named without evidence — in particular, none of Kubernetes, Compose,
   Nomad, systemd, Fly.io, Render or Heroku was introduced, and the mechanism
   that *is* recorded is recorded because the authoritative source states it, not
   because the container image or the release workflow implied it.

5. **Production topology and configuration values are deliberately not recorded
   in this public repository.** The specification cites the owning source by name
   and location only, and requires the implementing agent to read current values
   from that source at execution time. Account identifiers, ARNs, stack, cluster
   and service names, autoscaling parameter values and every environment-variable
   value were excluded on purpose.

6. **The `init` entrypoint finding is recorded as a prerequisite, not as a stop
   condition.** It is a bounded runtime-code gap, not an architecture change:
   the approved `OFF`/`OBSERVE`/`ENFORCE` lifecycle is intact and the merged
   inert state remains correct and fail-safe. Section 13.1 records it, requires
   remediation options to be specified without being implemented, and requires
   the fleet-verification mechanism to be able to detect exactly this silent
   failure.

7. **The rehearsal boundary was settled explicitly, as required.** The rehearsal
   is defined here and executed at the later preview/staging gate, because it
   depends on a verification mechanism this task only specifies. Discovering that
   the rehearsal cannot be defined without executing part of it is made a stop
   condition rather than a licence to execute it.

8. **Fleet verification is specified, not built.** Section 3.5 requires the
   smallest separately reviewable mechanism, forbids adding runtime observability
   in the specification PR, and forbids adding it silently anywhere.

9. **The monitoring boundary is preserved.** The specification may state what
   telemetry proves effective mode and fleet consistency; it is forbidden from
   inventing service-health thresholds, latency or error-rate baselines, or
   availability SLOs. That work remains the separate next gate.

Deviations from the task brief: NONE.

## 6. Database and migration effects

Migration added: NO.

No schema, migration, `thoth-api/src/schema.rs`, model, data or database effect
of any kind. No database was accessed.

## 7. API and compatibility effects

GraphQL/API changes: NONE. The public GraphQL schema is untouched and the
generated SDL is unchanged.
Generated schema/client updates: NONE.
Backwards compatibility: unaffected.
Deprecations: NONE.
Cross-repository dependencies: the specification records that authoritative
deployment configuration is owned outside this repository and requires the
implementing agent to consult that owner. No change to any other repository was
made, proposed as a commit, or requested.

## 8. Authorization and security

Authorization paths changed: NONE. `thoth-api/src/policy.rs` is untouched.
Roles/scopes involved: none exercised.
Negative authorization tests: not applicable — no code changed.

Secret or personal-data handling: **no secret value was recorded anywhere in the
diff.** During authorized read-only discovery, the authoritative deployment
source was found to carry production credential material inline in template
parameters. No such value was copied into this repository, this report, the
specification, the pull request or the changelog. The specification records the
hazard as a constraint on how the future task may work, names remediation as
outside its scope, and requires escalation instead.

Security limitations: this task performed read-only inspection only. It changed
no credential, rotated no secret, and accessed no production database or service.

## 9. Tests and checks

The change is documentation-only. `AGENTS.md` section 8 requires the
documentation-only evidence set rather than the full workspace gate.

### Whitespace and diff hygiene

Command:

```text
git diff --check
```

Result:

```text
recorded on the pull request at the exact head
```

### Link and path validation

Every relative Markdown link introduced by this change was resolved against the
working tree. Targets referenced: `ADR-0005`, `ADR-0006`,
`THOTH-GQL-BATCH-01.md`, the `THOTH-GQL-BATCH-01` implementation report,
`release-gates.md`, `operating-model.md`, `risk-classification.md`,
`implementation-report-template.md`, `control-gaps.md`, `environments.md` and
`repositories/thoth.md`.

### Runtime-path exclusion check

Verified that the diff contains no Rust, SQL, migration, `Cargo`, `Dockerfile`,
`docker-compose.yml`, `Makefile` or workflow path.

### Unit / integration / database / lint

Not applicable. No code changed, so the workspace gate is not the required
evidence set for this change.

## 10. Manual verification

Environment: local working tree at base
`75f44aabc52d98596ea6ce69ab068b3698fcd524`, plus read-only inspection of pinned
dependency sources and of the authoritative deployment source.

Steps and observed results:

1. **Base verification.** `git rev-parse origin/develop` equals the expected
   base exactly.

2. **Guard configuration path.** `src/bin/arguments/mod.rs` defines the argument
   with `Arg::env("THOTH_GRAPHQL_MUTATION_GUARD_MODE")`, default `OFF`, and a
   `value_parser` restricted to the three modes.

3. **Process-start-only read.** `thoth-api-server/src/lib.rs` captures the parsed
   mode in the `HttpServer::new` closure and registers it as `app_data`. No
   reload, watcher, signal handler or admin route exists anywhere in the
   workspace. Changing the effective mode of a running process is therefore
   impossible.

4. **No effective-mode observability.** The complete route set is `/`,
   `/graphiql`, `/graphql` (GET and POST) and `/schema.graphql`; `ApiConfig`
   exposes no mode; no startup log records it; the guard emits `log::warn!` only
   on a collision, and only outside `OFF`. `OFF` and `OBSERVE` are therefore
   externally indistinguishable.

5. **`init` entrypoint finding, reproduced.** The `Dockerfile` default command is
   `init`; `src/bin/commands/mod.rs` does not register the guard argument on
   `init`; `src/bin/thoth.rs` nevertheless routes `init` into the same handler;
   and that handler falls back to `"OFF"`. In pinned `clap_builder` 4.6.0,
   `ArgMatches::verify_arg` returns `UnknownArgument` only under
   `cfg(debug_assertions)`.

   Reproduced in an isolated throwaway probe **outside this repository**, built
   against the same pinned `clap` 4.6.1, mirroring the exact argument definition
   and access pattern. Observed:

   ```text
   release build, subcommand `graphql-api`, env ENFORCE  -> ENFORCE
   release build, subcommand `init`,        env ENFORCE  -> OFF
   release build, subcommand `init`,        env OBSERVE  -> OFF
   release build, subcommand `init`,        env unset    -> OFF
   debug build,   subcommand `init`,        env ENFORCE  -> panic:
       "Mismatch between definition and access of `mutation-guard-mode`"
   ```

   The `Dockerfile` builds with `cargo build --release`, so the shipped image
   takes the release branch. No repository code was built, modified or added for
   this reproduction.

6. **Repository deployment knowledge.** The repository declares no deployment
   manifest, orchestration configuration or environment injection.
   `docker-compose.yml` is a local development composition with no Thoth API
   service. GitHub repository environments: `total_count: 0`. GitHub deployments:
   none. GitHub is therefore not the deployment control system and holds no
   environment reviewers for this repository.

7. **Owning control system identified from evidence, inspected read-only.** The
   authoritative deployment configuration for the production and test Thoth
   GraphQL API services is owned outside this repository. It was identified from
   organisation evidence, inspected read-only for ownership, mechanism and
   configuration metadata, and is cited in the specification by name and location
   only. Nothing was changed, dispatched or executed there, and no value was
   copied out.

8. **Negative evidence recorded.** A connected infrastructure provider account
   was checked read-only and contains no Thoth service; it is not the Thoth
   production runtime. No platform was inferred from the Docker image or the
   release workflow.

9. **No production action.** No mode was changed in any environment, no
   deployment or workflow was dispatched, no secret was retrieved, and no
   production database or service was accessed.

Evidence link: the pull request diff and this report.

## 11. CI

CI status: recorded on the pull request at the exact head.
Checks: the repository's changelog check plus the classifier-driven jobs. The
change is documentation-only, so the classifier is expected to skip the
Rust build/test/clippy/format and migration jobs; the exact classification and
the exact-head results are the authority and are recorded on the PR.
Failures or warnings: recorded on the PR.

## 12. Rollout and rollback

Initial state after merge:

```text
THOTH_GRAPHQL_MUTATION_GUARD_MODE = OFF
loader store                      = unavailable
production request acceptance     = unchanged
```

Activation required: none by this change. Merging it authorizes nothing.
`OFF -> OBSERVE` and `OBSERVE -> ENFORCE` each still require their own separate
explicit CTO production activation approval.

Feature flag/configuration: none introduced.
Migration sequence: not applicable.
Rollback/disable procedure: revert the documentation commit. There is no runtime
or production effect to roll back.
Monitoring required: none introduced by this change.

## 13. Known limitations and deferred work

- The specification is `DRAFT`. It requires fresh independent exact-head review
  and explicit CTO specification approval before `THOTH-GQL-OPS-01`
  implementation may be authorized.
- The `init` entrypoint gap means that, if the production GraphQL API container
  runs the image default command, the mode is **not currently controllable in
  production**. A bounded, separately specified and separately reviewed
  remediation is a hard prerequisite for `OBSERVE`. It is recorded, not fixed.
- No mechanism exists today that proves the effective mode of a serving
  instance. The specification requires the smallest separately reviewable
  mechanism to be defined; that mechanism is not designed in detail here and is
  not implemented.
- Propagation interval, rollback duration and fleet-consistency timing remain
  unmeasured by design. They are required to be measured in the later
  preview/staging rehearsal, and no numeric value was invented.
- Service-health signals, activation thresholds and any latency, error-rate or
  availability baseline remain outside this task and are the separate next gate.
- The credential-exposure hazard observed in the authoritative deployment source
  is recorded as a constraint and escalated. It is not remediated here and is not
  this task's to remediate.

## 14. Unresolved issues

- The production container command must be confirmed from the authoritative
  deployment source by the implementing agent at execution time. This report
  records the repository-side half of the finding as proven and the
  production-applicability half as requiring that confirmation.
- The production runtime execution owner and the observation sign-off owner are
  not yet identified as roles. Both are acceptance criteria of the output task
  and stop conditions if unobtainable.

## 15. Agent self-assessment

The agent may identify risks but may not approve the task. No approval decision
is made or implied here.

Suggested review focus:

1. **Scope discipline** — confirm the specification addresses only the
   mutation-guard runtime-mode-control subset and does not drift into
   service-health thresholds, migration execution, restore verification or
   general CG-13 closure.
2. **CG-13 disposition** — confirm CG-13 remains open, that only a durable
   forward reference was added, and that no closure criterion was smuggled in.
3. **Evidence discipline** — confirm no operational claim is recorded without an
   evidence source, that no propagation or rollback duration was invented, and
   that no runtime platform was inferred from the Docker image or the release
   workflow.
4. **The `init` finding** — re-derive it independently from `Dockerfile`,
   `src/bin/commands/mod.rs`, `src/bin/thoth.rs`, `src/bin/commands/start.rs` and
   the pinned `clap_builder` sources, and challenge its classification as a
   prerequisite rather than a stop condition.
5. **Secret hygiene** — confirm the diff contains no production configuration
   value, resource identifier or credential, and assess whether the level of
   deployment detail recorded is appropriate for a public repository.
6. **Authorization boundaries** — confirm merge authorization, `OFF -> OBSERVE`
   authorization and `OBSERVE -> ENFORCE` authorization remain three separate
   decisions, and that nothing in the diff grants any of them.
7. **Non-implementation** — confirm the diff contains no runtime file, that the
   fleet-verification mechanism is specified rather than built, and that no
   environment's guard mode was changed.
