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
authorized and evidenced — and, where the operational capability to do any of
that does not exist, identifying the gap, specifying the bounded task that must
close it, and recording the runtime-operations gate as **NOT SATISFIED**.
Without activating anything.

Out-of-scope changes made: NONE.

Independent review: `CHANGES REQUIRED` at head
`27becee16e048eef30017fc5ff509362f0808ba3` (two P1 specification defects, one
security/audit correction). All three are remediated on this same branch and
pull request; no new branch or PR was created. See section 5.1.

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

1. **The specification is bounded to mode control, and its mandatory terminal
   disposition is C.** CG-13's migration execution, restore verification and
   general approver mapping are left open and are explicitly listed as non-goals.

   ```text
   THOTH-GQL-OPS-01 is bounded to mutation-guard mode-control discovery.

   Its mandatory expected terminal disposition on the currently established
   evidence is C - BLOCKED.

   Disposition A is forbidden while either capability gap remains open and
   becomes reachable only through THOTH-GQL-OPS-04 after OPS-02 and OPS-03
   have been implemented, independently reviewed and merged.
   ```

   *Historical note, superseded:* the pre-remediation head
   `27becee16e048eef30017fc5ff509362f0808ba3` did default to disposition **A**.
   That statement is **withdrawn** and is recorded here only as superseded
   history; it is not a current statement of the specification.

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

6. **The `init` entrypoint finding is recorded as a blocking prerequisite, not as
   a stop condition.** It is a bounded runtime-code gap, not an architecture
   change: the approved `OFF`/`OBSERVE`/`ENFORCE` lifecycle is intact and the
   merged inert state remains correct and fail-safe. Section 13.1 records it,
   hands remediation to a separate task, forbids selecting the mechanism here,
   and requires the fleet-verification mechanism to detect exactly this silent
   failure class.

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

### 5.1 Decisions added by remediation of the independent review

The review returned `CHANGES REQUIRED` at head
`27becee16e048eef30017fc5ff509362f0808ba3` with two P1 specification defects and
one security/audit correction. All three are addressed.

10. **P1-1 — the specification could satisfy the runtime-operations gate while
    the mechanisms it depends on were absent.** *(Historical, describing the
    superseded head `27becee16e048eef30017fc5ff509362f0808ba3`.)* That text was
    internally inconsistent: its success behaviour promised an operator who could
    execute, verify and roll back a mode change, while AC-8/AC-22 required the
    verifier only to be *specified*, and its section 12 defaulted to disposition
    **A**. All of that is **superseded**.

    Corrected by making the two capability gaps first-class and terminal:

    - section 1 names them, states that documenting them is not closing them, and
      records the expected terminal disposition as **C — BLOCKED**;
    - section 6.1 **withdraws** the operator-capability success condition and
      reassigns it to `THOTH-GQL-OPS-04`;
    - new section 12.1 makes disposition **A forbidden** while either gap holds,
      and enumerates what specifically does *not* convert C into A — including
      having specified the verifier, and having specified the prerequisite tasks;
    - new section 12.2 requires the runtime-operations gate to be recorded
      **NOT SATISFIED** in specification, control record, report and PR body;
    - new section 3.12 specifies the three bounded successors;
    - section 11.1 restates the full dependency sequence with the gate satisfied
      no earlier than `THOTH-GQL-OPS-04`;
    - AC-8 is annotated rather than weakened, and eight criteria (**AC-23** to
      **AC-30**) were **added**. No existing criterion was relaxed or removed.

11. **P1-2 — `start graphql-api` was implicitly offered as an equivalent fix.**
    The original section 13.1 listed "setting an explicit container command" beside
    the `init` fix as interchangeable options. That was wrong: `init` runs
    migrations then starts the API, so an override would remove migration
    execution from the deployment path and intersect the broader CG-13
    migration/deployment problem.

    Corrected by new section 2.2.3.1, which establishes the difference from
    `src/bin/thoth.rs` and the `Dockerfile`, and by rewritten section 13.1.1,
    which splits remediation into class 1 (feature-local, preserves all `init`
    migration and startup semantics — in scope for `THOTH-GQL-OPS-02`) and class 2
    (production command override — **not** interchangeable, out of bounded scope,
    requiring separate migration/deployment-control analysis and approval). New
    non-goals 24 and 25 and invariant 11 enforce it, and the specification does
    not select the fix.

12. **Security/audit correction.** The report's "no secret was retrieved" is
    withdrawn as materially inaccurate and replaced with the accurate account in
    section 8. New specification section 2.2.5 binds this task and every successor
    to scoped reads of the secret-bearing source. Section 2.2.4 was reduced to the
    minimum durable hazard and control boundary, with no further detail about
    credential values, resource identifiers, production topology, account
    identifiers, or stack, cluster or service names. Secret remediation remains a
    separate CTO-controlled task and is not in this PR; no security issue was
    created or modified.

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

Secret or personal-data handling — **corrected during remediation.** An earlier
revision of this report stated that "no secret was retrieved". That wording is
**withdrawn** as materially inaccurate: it is inconsistent with the discovery
actually performed, and it understated what a reader needs to know.

The accurate statement is:

> During authorized read-only discovery, the implementing agent encountered
> existing secret-bearing configuration in the private authoritative deployment
> source.
>
> No secret value was copied into the public `thoth` repository, the PR body, the
> specification, the implementation report or the changelog; no credential was
> used, changed or rotated; and no production service or database was accessed.
>
> The discovery is a process/security escalation and is not part of this
> specification's implementation scope.

**No secret value, credential identifier, ARN, account identifier, private key,
password, token or sensitive configuration appears anywhere in this PR.** The
specification records only the minimum durable hazard and control boundary
necessary for the task — that the source is secret-bearing, and that reads of it
must therefore be scoped — and publishes no further detail about credential
values, resource identifiers, production topology, account identifiers, stack,
cluster or service names, or private configuration.

Remediation of the exposure is **not** in this PR and is not this task's to
perform. It remains a **separate CTO-controlled security matter**. No security
issue was created or modified, and none may be without separate explicit CTO
authorization.

Constraint added for successor work: specification section 2.2.5 now binds
`THOTH-GQL-OPS-01` and every task it specifies to scoped reads of that source —
narrowly scoped searches or line/range reads, retrieval of only the metadata a
criterion requires, never copying secret-bearing ranges into reports or prompts,
and stopping with `BLOCKED` if required evidence cannot be obtained without
exposing secret material.

Security limitations: this task performed read-only inspection only. It changed
no credential, rotated no secret, used no credential, and accessed no production
service or database.

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

   **Method statement.** That source is secret-bearing, and secret-bearing
   configuration was encountered during this read-only discovery. No secret value
   was copied into any output; see section 8. Section 2.2.5 of the specification
   now binds all successor reads to narrowly scoped searches or line/range reads,
   metadata-only retrieval, and `BLOCKED` where a criterion cannot be satisfied
   without exposing secret material.

8. **Deployment-path applicability of the entrypoint gap, established — and
   distinguished from the deployed release, with evidence provenance kept
   separate.** The repository establishes what a release *contains*; only the
   authoritative deployment source establishes what production *runs*. The two
   are recorded separately and combined explicitly:

   - *release-code state.* `[REPO]` The release line `master` contains no
     `MutationGuardMode`, defines no `mutation_guard_mode()` CLI argument and has
     no mutation-guard wiring on its GraphQL startup path; the most recent
     release tag likewise contains none; and the production image is built from a
     published GitHub release
     (`.github/workflows/docker_build_and_push_to_dockerhub_release.yml`).

     ```text
     Release-code state [REPO]:
     The relevant release/master code contains no mutation guard and is
     PRE-GUARD.
     ```

     This is the whole of what the repository establishes. **It does not
     establish which release production is running**, and it is not offered as
     evidence of that;

   - *deployment state.* `[EXTERNAL]` Which release or image production actually
     runs is deployment-state evidence and is not derivable from this repository.
     It rests on the scoped authoritative deployment metadata already collected
     during this task's authorized read-only discovery (section 10, finding 7),
     under the section 2.2.5 scoped-read rules. **No further read of that source
     was performed for this correction**, and no value, identifier or
     configuration detail from it is recorded here.

     ```text
     Deployment state [EXTERNAL]:
     Previously established scoped authoritative deployment metadata identifies
     that release/image as the one currently deployed to production.
     ```

   - *combined conclusion.* `[REPO + EXTERNAL]` Neither class establishes it
     alone:

     ```text
     Combined conclusion [REPO + EXTERNAL]:
     Current production is therefore PRE-GUARD and is not
     MutationGuardMode::OFF.
     ```

     It is not described as having any guard mode, and merging
     `THOTH-GQL-BATCH-01` deployed nothing. Had the `[EXTERNAL]` half been
     unobtainable under the scoped-read rules, the correct action would have been
     to downgrade this conclusion to `[UNVERIFIED]` — never to re-derive it from
     `[REPO]` evidence, and never to widen access;

   - *the deployment path.* `[EXTERNAL]` The production GraphQL API service does
     not override the container command and so inherits the image default `init`.
     This is a separate `[EXTERNAL]` fact from the deployed-release identity above
     and is likewise drawn from the metadata already collected.

   Combined with finding 5, the established consequence is a property of the
   **path**, not of the deployed binary:

   ```text
   Under the currently authoritative production deployment command path, a
   guard-enabled release containing the merged foundation would execute
   through `init`.

   Until THOTH-GQL-OPS-02 is delivered, that path cannot consume
   THOTH_GRAPHQL_MUTATION_GUARD_MODE.

   Therefore an OFF -> OBSERVE transition of a guard-enabled candidate is not
   operationally performable through the current deployment path.
   ```

   This is capability gap 1 — the OPS-02 gap. It is a hard blocking
   prerequisite, and it is not weakened by the fact that production is not yet
   running guard-enabled code.

9. **`init` and `start graphql-api` are not interchangeable.** `src/bin/thoth.rs`
   shows `init` running `commands::run_migrations(arguments)?` before
   `commands::start::graphql_api(arguments)`, while `start graphql-api` runs no
   migrations; the `Dockerfile` comment states the same intent. An explicit
   production command override would therefore remove migration execution from
   the deployment path, so it is **not** an interchangeable feature-local fix and
   is classified out of bounded scope in specification section 13.1.1.

10. **Negative evidence recorded.** A connected infrastructure provider account
    was checked read-only and contains no Thoth service; it is not the Thoth
    production runtime. No platform was inferred from the Docker image or the
    release workflow.

11. **No production action.** No mode was changed in any environment, no
    deployment or workflow was dispatched, no credential was used, changed or
    rotated, and no production service or database was accessed.

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
deployed production release       = pre-guard, no guard mode exists
                                    [REPO + EXTERNAL]
guard-enabled candidate default   = OFF, loader store unavailable   [REPO]
environments transitioned         = none
production request acceptance     = unchanged
```

Merge, release and activation remain three distinct events:

```text
merged develop state
    != deployed production release
    != production activation state
```

Activation required: none by this change. Merging it authorizes nothing, and
merging `THOTH-GQL-BATCH-01` deployed nothing. `OFF -> OBSERVE` and
`OBSERVE -> ENFORCE` each still require their own separate explicit CTO
production activation approval, and each applies only to a guard-enabled
environment.

Feature flag/configuration: none introduced.
Migration sequence: not applicable.
Rollback/disable procedure: revert the documentation commit. There is no runtime
or production effect to roll back.
Monitoring required: none introduced by this change.

## 13. Known limitations and deferred work

- The specification is `DRAFT`. It requires fresh independent exact-head review
  and explicit CTO specification approval before `THOTH-GQL-OPS-01`
  implementation may be authorized.
- **The runtime-operations gate is NOT SATISFIED, and `THOTH-GQL-OPS-01` cannot
  satisfy it.** Its expected terminal CG-13 disposition is **C — BLOCKED**. The
  gate is satisfiable no earlier than `THOTH-GQL-OPS-04`, and only after
  `THOTH-GQL-OPS-02` and `THOTH-GQL-OPS-03` have merged.
- **Capability gap 1 (the OPS-02 gap):** the current production deployment path
  runs the image default `init`, and `init` does not accept the mode, so that
  path cannot consume `THOTH_GRAPHQL_MUTATION_GUARD_MODE` once guard-enabled code
  is deployed. An `OFF -> OBSERVE` transition of a guard-enabled candidate is
  therefore not operationally performable through the current deployment path.
  This is a deployment-path property; the release currently deployed to
  production is **pre-guard** and has no guard mode at all — a
  `[REPO + EXTERNAL]` conclusion, not a repository-only one (section 10, finding
  8). Recorded, not fixed; `THOTH-GQL-OPS-02` must close it.
- **Capability gap 2:** no implemented mechanism proves the effective mode of a
  serving instance. The specification requires the smallest separately reviewable
  mechanism to be defined; it is neither designed in detail nor implemented here.
  `THOTH-GQL-OPS-03` must close it.
- The runbook `THOTH-GQL-OPS-01` produces is **PROVISIONAL**. Its procedures
  cannot be executed until both prerequisites merge.
- The three prerequisite task specifications are **named and scoped** here but
  are **not written**; writing them is `THOTH-GQL-OPS-01` implementation work,
  and none of their branches exists.
- Propagation interval, rollback duration and fleet-consistency timing remain
  unmeasured by design. They are required to be measured in the later
  preview/staging rehearsal, and no numeric value was invented.
- Service-health signals, activation thresholds and any latency, error-rate or
  availability baseline remain outside this task and are the separate next gate.
- The credential exposure in the authoritative deployment source is recorded only
  as the minimum hazard and control boundary. It is not remediated here, is not
  this task's to remediate, and remains a separate CTO-controlled security
  matter.

## 14. Unresolved issues

- The production runtime execution owner and the observation sign-off owner are
  not yet identified as roles. Both are acceptance criteria of the output task
  and stop conditions if unobtainable.
- The `THOTH-GQL-OPS-02` remediation mechanism is deliberately **unselected**.
  The specification records the permitted class and its migration-preserving
  constraint; selecting the mechanism belongs to that task's own approved
  specification, after the owning parties have been consulted.
- Whether an explicit production command override could ever be appropriate is
  **not** decided here. It is classified out of bounded scope and escalated to
  the migration/deployment half of CG-13.

## 15. Agent self-assessment

The agent may identify risks but may not approve the task. No approval decision
is made or implied here.

Suggested review focus:

1. **P1-1 closure — can the gate still be claimed?** Attempt to construct a
   reading of the remediated specification under which `THOTH-GQL-OPS-01` returns
   disposition A or reports the runtime-operations gate as satisfied. Section 12.1
   is intended to make that impossible; if any route remains, the remediation is
   incomplete.
2. **Criteria not weakened** — confirm AC-1 to AC-22 are intact, that AC-8 gained
   only a clarifying annotation, and that AC-23 to AC-30 are genuine additions
   rather than replacements.
3. **P1-2 closure — migration semantics** — re-derive section 2.2.3.1 from
   `src/bin/thoth.rs` and the `Dockerfile`, and confirm that no remaining text
   presents a production command override as an interchangeable feature-local
   fix, and that the class 2 classification appears wherever an override is
   mentioned.
4. **Prerequisite task boundaries** — confirm `THOTH-GQL-OPS-02`, `-03` and `-04`
   are scoped without being implemented, that no branch was created, and that the
   `-02` mechanism is genuinely left unselected.
5. **The `init` finding** — re-derive it independently from `Dockerfile`,
   `src/bin/commands/mod.rs`, `src/bin/thoth.rs`, `src/bin/commands/start.rs` and
   the pinned `clap_builder` sources, and challenge its classification as a
   blocking prerequisite rather than a stop condition.
6. **Secret hygiene and the audit correction** — confirm the diff contains no
   credential, resource identifier or configuration value; that section 8's
   corrected wording is materially accurate; that section 2.2.5's scoped-read
   rules are workable; and that the published hazard detail is the minimum
   necessary for a public repository.
7. **Scope discipline** — confirm the specification still addresses only the
   mutation-guard runtime-mode-control subset and has not drifted into
   service-health thresholds, migration execution, restore verification or
   general CG-13 closure.
8. **Authorization boundaries** — confirm merge authorization, `OFF -> OBSERVE`
   authorization and `OBSERVE -> ENFORCE` authorization remain three separate
   decisions, and that nothing in the diff grants any of them.
9. **Non-implementation** — confirm the diff contains no runtime file, that the
   fleet-verification mechanism is specified rather than built, and that no
   environment was transitioned.
10. **Deployment-state accuracy and evidence provenance** — re-derive the
    `[REPO]` half of specification section 2.2.0 from `master`, the most recent
    release tag and the release workflow, and confirm it is not stretched to
    cover the deployed-release identity, which is `[EXTERNAL]`. Confirm that the
    current-production pre-guard conclusion is labelled `[REPO + EXTERNAL]`
    everywhere it appears, that no deployed-state statement anywhere in the diff
    is attributed to `[REPO]` alone, that no document describes a pre-guard
    release as `MutationGuardMode::OFF`, that nothing implies merging
    `THOTH-GQL-BATCH-01` deployed the guard, and that capability gap 1 is still
    stated as a deployment-path property without being weakened.
