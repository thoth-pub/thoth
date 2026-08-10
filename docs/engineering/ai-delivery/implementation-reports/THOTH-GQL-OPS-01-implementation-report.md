# THOTH-GQL-OPS-01 Implementation Report

Implementation of the approved, merged
[`THOTH-GQL-OPS-01`](../tasks/THOTH-GQL-OPS-01.md) specification.

**Expected terminal disposition reached.** This task's approved scope is
control/discovery work whose correct outcome is `BLOCKED` at the runtime-operations
gate. Reaching that outcome is the task succeeding, not failing.

```text
THOTH-GQL-OPS-01:        completed as control/discovery work
CG-13 disposition:       C - insufficient operational capability/evidence; BLOCKED
Runtime-operations gate: NOT SATISFIED

OPS-02:                  specified, NOT implemented
OPS-03:                  specified, NOT implemented
OPS-04:                  specified, NOT implemented

CG-13:                   OPEN
OBSERVE:                 NOT AUTHORIZED
ENFORCE:                 NOT AUTHORIZED
BE-02 runtime:           NOT AUTHORIZED
```

## 1. Repository state

Repository: `thoth-pub/thoth`
Workflow: STANDARD
Base branch: `develop`
Base commit: `8b144a6de75ad6289f481c4e17e02c4c5f0f6328`
PR target: `develop`
Programme integration branch: None
Task branch: `feature/shared-architecture/graphql-runtime-ops`
Head commit: recorded on the pull request; the PR head is the authority
Pull request: opened as **DRAFT**; not marked ready, not approved, not
self-reviewed, not merged
Expected branch deletion after merge: YES
Final programme PR required: NO
Implementing model: Claude Opus 5
Reasoning level: HIGH

### 1.1 Pre-implementation verification

Performed before any branch was created:

```text
git fetch origin --prune
git rev-parse origin/develop
  -> 8b144a6de75ad6289f481c4e17e02c4c5f0f6328   MATCHES the authorized base

git branch -a --list '*graphql-runtime-ops*'
  -> only feature/shared-architecture/graphql-runtime-ops-spec (the merged
     OPS-01-SPEC branch). The required implementation branch
     feature/shared-architecture/graphql-runtime-ops did NOT exist.

git ls-remote --heads origin | grep -E 'ops-0[234]|runtime-ops'
  -> no OPS-02/03/04 implementation branch exists.
```

`develop` had **not** moved from the authorized base, so no rebase, no
re-evaluation and no escalation was required. The branch was created directly
from `8b144a6de75ad6289f481c4e17e02c4c5f0f6328`.

CTO implementation authorization was verified as terminal GitHub evidence on
merged PR [#792](https://github.com/thoth-pub/thoth/pull/792), naming the exact
authorized base `develop @ 8b144a6de75ad6289f481c4e17e02c4c5f0f6328`.

## 2. Scope confirmation

Approved specification:
[`THOTH-GQL-OPS-01`](../tasks/THOTH-GQL-OPS-01.md), merged through PR
[#792](https://github.com/thoth-pub/thoth/pull/792). Where the implementation
prompt and the merged specification could differ, the merged specification was
treated as authoritative.

Implemented objective: establish from current evidence how the merged mutation
guard's `THOTH_GRAPHQL_MUTATION_GUARD_MODE` is configured, changed, deployed,
propagated, verified fleet-wide, detected when partially applied, rolled back,
authorized and evidenced; record the capability gaps that documentation cannot
close; specify the bounded prerequisite tasks that would close them; and record
the runtime-operations gate as **NOT SATISFIED** with CG-13 disposition
**C — BLOCKED**. Without activating anything.

Out-of-scope changes made: NONE.

## 3. Commits

```text
Exact commit sequence and final head:
GitHub PR #793 is terminal authority.
```

The branch was squashed to a single commit during independent-review remediation
(section 3.1), so no commit SHA is restated here: any SHA written into a file on
the branch is invalidated by the act of writing it, and `ADR-0005` makes the
GitHub pull-request record the authority for lifecycle facts of exactly this
kind.

### 3.1 Branch-history rewrite during remediation

Independent review required that excess public detail in the security record be
removed from the **mergeable branch history**, not merely from the working tree.
Before rewriting, all three hygiene preconditions were verified and held:

```text
1. the branch contained only the implementing agent's own work
     -> 3 commits, all authored and committed under the same identity

2. no third-party or concurrent commits had appeared
     -> local branch tip == origin branch tip; no other author present

3. no other branch depended on the intermediate commits
     -> `git branch -a --contains` and a scan of every remote head
        returned only this implementation branch
```

The branch was then squashed to one commit and force-pushed deliberately. Had any
concurrent third-party work existed, the correct action would have been to stop
and report rather than overwrite it.

The rewrite necessarily produced a **new exact head**, which requires **fresh
independent review**. The previously reviewed head
`491d67dea7402641be42b9acc4d18b58b169f2b8` no longer exists on the branch.

## 4. Files changed

All changes are documentation. **Zero runtime files.**

**12 files**: 11 under `docs/`, plus `CHANGELOG.md`. This count includes this
implementation report, which is itself part of the pull request.

- `docs/engineering/repository-map/graphql-mutation-guard-runtime-operations.md`
  **(new)**
  - reason: deliverable A — the operational-control record answering
    specification sections 3.1 to 3.10, every conclusion carrying an evidence
    source and an evidence class.
  - behavioural effect: none. Documentation. It authorizes nothing and changes no
    production behaviour.
- `docs/engineering/repository-map/graphql-mutation-guard-mode-transition-runbook.md`
  **(new)**
  - reason: deliverable B — the mode-transition runbook required by `ADR-0006`
    section 8.3.5, marked **PROVISIONAL** and stating that it is not executable
    until OPS-02 and OPS-03 are implemented, independently reviewed and merged.
  - behavioural effect: none. It is explicitly not executable and authorizes no
    transition.
- `docs/engineering/ai-delivery/tasks/THOTH-GQL-OPS-02.md` **(new)**
  - reason: deliverable C — mode-control path specification. `DRAFT`,
    implementation `NOT AUTHORIZED`.
  - behavioural effect: none. No branch created, no implementation performed.
- `docs/engineering/ai-delivery/tasks/THOTH-GQL-OPS-03.md` **(new)**
  - reason: deliverable C — fleet-verification mechanism specification. `DRAFT`,
    implementation `NOT AUTHORIZED`.
  - behavioural effect: none. No branch created, no implementation performed.
- `docs/engineering/ai-delivery/tasks/THOTH-GQL-OPS-04.md` **(new)**
  - reason: deliverable C — bounded verification and closure specification.
    `DRAFT`, implementation `NOT AUTHORIZED`.
  - behavioural effect: none. No branch created, no implementation performed.
- `docs/engineering/repository-map/control-gaps.md`
  - reason: deliverable D — record the CG-13 feature-subset disposition `C`, the
    gate state, the confirmed capability gaps, the newly established pre-guard
    status of the **test** environment, and the unresolved evidence.
  - behavioural effect: none. CG-13 remains **OPEN**; no partial closure claimed.
- `docs/engineering/repository-map/environments.md`
  - reason: deliverable D — record that one bounded subset is now mapped, and
    that both the production and test GraphQL API releases are **pre-guard** on
    combined evidence.
  - behavioural effect: none.
- `docs/engineering/decisions/decision-register.md`
  - reason: deliverable D — minimal durable update to the `ADR-0006` dependency
    sequence: the successors now exist as specifications, remain `DRAFT` and
    `NOT AUTHORIZED`, and none of their branches exists.
  - behavioural effect: none.
- `docs/engineering/repository-map/README.md`
  - reason: deliverable E — index the two new records.
  - behavioural effect: none.
- `docs/engineering/ai-delivery/README.md`
  - reason: deliverable E — index OPS-02/03/04 as `DRAFT` / `NOT AUTHORIZED` and
    point OPS-01 at its delivered output.
  - behavioural effect: none.
- `CHANGELOG.md`
  - reason: deliverable F — `## [Unreleased]` entry. It states explicitly that no
    runtime or production behaviour changed.
  - behavioural effect: none.

## 5. Implementation decisions

Decisions made within the approved design:

1. **Placement.** The two new records were placed flat in
   `docs/engineering/repository-map/`, alongside `environments.md` and
   `control-gaps.md`, rather than in a new `runbooks/` or `operations/` hierarchy.
   That directory already owns runtime, environment and control-gap records, and
   the specification requires using repository-native paths rather than inventing
   a hierarchy. No new directory was created.
2. **The control record and the runbook are separate files.** The control record
   is evidence; the runbook is procedure. Keeping them separate lets
   `THOTH-GQL-OPS-04` lift the `PROVISIONAL` marking from the runbook without
   rewriting the evidence record, and prevents an operator from reading evidence
   as instruction.
3. **The `PROVISIONAL` marking is a blocking banner, not a footnote.** It is the
   first thing in the runbook, in a box, with an explicit "do not execute any
   step below". A marking an operator can skim past would not satisfy AC-29 in
   substance.
4. **No exact external value was recorded anywhere**, including values that are
   arguably not sensitive — the deployed release version, the desired instance
   count, the maximum instance count, the log-retention integer, the rollout
   percentages and the command values of other services. Conclusions are recorded
   instead. This follows the merged specification's own convention and its
   prohibition on recording scaling parameters and private configuration.
5. **Execution capability and accountable ownership are kept strictly apart.**
   Control record §2.0 states the distinction; §2.1 records what an access record
   actually establishes — who is technically **able** to apply a change — and
   §2.1.1 records that the accountable production runtime **owner** is not
   established at all, because what is missing is a CTO designation rather than a
   readable fact. Write access is never relabelled as runtime ownership.
6. **The observation sign-off owner is recorded as a proposal, not a
   designation.** §2.3 proposes the CTO and shows the reasoning, deliberately, so
   the CTO has something concrete to confirm or reject rather than an open
   question. It is graded `[UNVERIFIED]`.
7. **Four criteria are graded FAIL/BLOCKED, and the grading was not optimised.**
   AC-1, AC-2, AC-7 and AC-13 each lack evidence that this task is not authorized
   to obtain — three need a CTO decision, one needs a live orchestrator read.
   Grading any of them PASS on a definition, a capability record or a derivation
   would be exactly the "specification is not the capability" error the
   specification forbids. A task terminating at disposition **C** is expected to
   retain failed criteria; see §11.1.
8. **The `THOTH-GQL-OPS-02` mechanism was deliberately not selected.** OPS-02
   records the bounded class and its constraints and leaves the mechanism to its
   own approved specification, per specification section 3.12.1.

Deviations from the specification:

- NONE.

### 5.1 Findings that refine the specification's narrative

The specification requires the implementing agent to re-derive its findings
rather than rely on them. Re-derivation **confirmed** every load-bearing finding
and added three that the specification did not state. None refutes it.

1. **Invalid values are also silently ignored on the `init` path.** The
   specification's section 3.2 item 7 expects an invalid value to make the process
   fail to start. That holds on `start graphql-api` only. On `init` the
   `value_parser` never runs, because the argument is not registered, so an
   invalid value yields effective mode `OFF` and exit status 0. This is a third
   instance of the same silent-ignore class, and `THOTH-GQL-OPS-02` carries it as
   an explicit acceptance criterion.
2. **The test environment is also pre-guard.** The specification established
   production as pre-guard. Re-derivation established the same for the test
   environment. Consequently there is currently **no** environment in which a mode
   could be changed, and the future rehearsal must first deploy a guard-enabled
   candidate to a non-production environment — recorded as a prerequisite in both
   the control record and the runbook.
3. **A deployment circuit breaker with automatic rollback is configured.** This
   answers the specification's question about what the orchestrator does with a
   task that fails to start: the deployment is rolled back automatically. It is a
   genuine fail-safe, but it protects only the path on which an invalid value
   actually fails, and it cannot detect a silently ignored value or a mode that
   was configured but not adopted — because such a task starts healthily.

## 6. Database and migration effects

Migration added: NO

```text
Database/data change:                         NONE
Migration files:                              NONE
Migration execution semantics changed:        NONE
```

The `init` migration boundary was re-established and is recorded, not changed:
`init` runs `commands::run_migrations(arguments)?` and only then
`commands::start::graphql_api(arguments)`, whereas `start graphql-api` runs no
migrations. No remediation altering migration execution was selected or
specified, and every mention of a production container-command override carries
the specification section 13.1.1 classification verbatim.

## 7. API and compatibility effects

GraphQL/API changes: NONE
Generated schema/client updates: NONE — the generated SDL is untouched
Backwards compatibility: unaffected; no runtime code changed
Deprecations: NONE
Cross-repository dependencies: none created. The private authoritative deployment
source was read only, and was not modified.

## 8. Authorization and security

Authorization paths changed: NONE
Roles/scopes involved: none changed. Roles are **recorded** in the control record
as an evidence finding; no permission, role or scope was created or altered.
Negative authorization tests: not applicable — no code changed.

### 8.1 Secret and personal-data handling — method statement

Required by specification section 2.2.5 and AC-30.

**Method.** Every read of the private authoritative deployment source
(`thoth-pub/infrastructure`) was **read-only**, **metadata-only** and **narrowly
scoped** to the specific acceptance criterion being satisfied. Reads were
performed as targeted key-scoped extractions and bounded line-range reads — for
example, presence/absence checks for a single environment-variable name,
structural-key listings with values suppressed, and single-key reads for one
parameter. **No whole-file read of a secret-bearing configuration file was
emitted, and no broad recursive dump was performed.**

**Secret material was encountered.** This report does **not** claim otherwise.

Under section 2.2.5 an incidental encounter during an otherwise scoped read is
**not a breach**, and it is reported here as an **escalation** rather than
quietly absorbed. It would become a breach only if the material were copied
onward.

**Escalation, recorded at the approved minimum and deliberately no further:**

```text
During authorized narrowly scoped read-only discovery, the implementing
agent encountered existing secret-bearing configuration in the private
authoritative deployment source.

No secret value was copied into any repository file, PR text, report,
changelog, prompt or commit message; no credential was used, changed or
rotated.

The exposure remains a separate CTO-controlled security matter and is
outside this task's scope.

No security issue was created or modified.
```

**The public record stops there, deliberately.** Characterising *where* the
material was found, *what kind* it was, or *what it relates to* would itself be
disclosure — a public description precise enough to direct a reader is a partial
leak even when no value is copied. The parent specification permits the public
repository to record only the minimum fact, and this report records only that.
The private source was **not** re-read for this remediation.

**Actions not taken, deliberately:** no credential was used, changed or rotated;
no change of any kind was made to the private repository; no workflow was
dispatched; no production service or database was accessed; access was never
widened to force an answer to a criterion.

Security limitations: the control record deliberately publishes the *existence*
and *ownership* of the authoritative deployment source and the *fact* that it is
secret-bearing, because the specification cannot identify configuration authority
without it. It publishes no platform, topology, stack, cluster, service or account
detail, no resource identifier, no hostname beyond those already in
`environments.md`, no scaling parameter and no environment-variable value.

## 9. Tests and checks

This task changes no code, so the repository's Rust test commands are not
applicable to its diff. The documentation validation the specification requires
was run in full.

### Formatting

Command:

```text
git diff --check
```

Result:

```text
clean - no whitespace or conflict-marker errors
```

### Unit tests

Command:

```text
not applicable - the task changes no code
```

Result:

```text
N/A. No .rs, Cargo.toml, Cargo.lock, migration, schema, Dockerfile,
docker-compose.yml, Makefile or workflow file appears in the diff.
```

### Integration/database tests

Command:

```text
not applicable - no database, migration or schema effect
```

Result:

```text
N/A
```

### Lint/static analysis

Command:

```text
not applicable to a documentation-only diff
```

Result:

```text
N/A
```

### Other required checks

**Changed-path classification.**

```text
git diff --name-only <base>..<head>
  -> 12 files: 11 under docs/, plus CHANGELOG.md
     (the count includes this implementation report)
  -> runtime files changed: 0
  -> prohibited paths (*.rs, Cargo.toml, Cargo.lock, migrations/**,
     thoth-api/src/schema.rs, thoth-api/src/policy.rs, Dockerfile,
     docker-compose.yml, Makefile, .github/workflows/**): NONE
  PASS
```

**Relative Markdown-link validation.**

```text
every relative link target in every changed .md file resolved on disk
  -> 0 broken links
  PASS
```

**Internal heading/cross-reference validation.**

```text
every '#fragment' anchor in every changed .md file resolved against the
target file's actual headings
  -> 0 broken anchors
  PASS
```

**Task-template completeness validation.**

```text
THOTH-GQL-OPS-02, -03 and -04 each contain all 17 top-level sections of
task-specification-template.md and all five 6.x subsections.
No unreplaced bracketed template placeholder remains except the section 17
approval metadata the template permits to be blank.
  PASS
```

**Implementation-report-template completeness validation.**

```text
this report contains all 15 sections of implementation-report-template.md
  PASS
```

**Secret / configuration-value scan.**

```text
scan of the added diff for credential material, resource identifiers,
account identifiers, private hostnames, platform/stack/cluster/service
names, the deployed release version, instance counts, retention values,
rollout percentages and environment-variable slots
  -> 0 occurrences of any value
  -> the only matches on words such as 'secret' or 'credential' are the
     PROHIBITIONS themselves
  PASS
```

**Evidence-classification sweep.**

The sweep asserts an **invariant**, not a headcount. Exact totals are
deliberately not recorded: they change with every editorial pass, cannot be
verified from a checkout without re-running the sweep, and a stale total is worse
than no total. All four evidence classes — `[REPO]`, `[EXTERNAL]`,
`[REPO + EXTERNAL]` and `[UNVERIFIED]` — are in use in the control record.

```text
Operational conclusions lacking an evidence class: 0
  PASS
```

**Current-production PRE-GUARD provenance sweep.**

```text
no pre-guard/deployed-state conclusion is attributed to [REPO] alone;
every one carries [REPO + EXTERNAL] or [UNVERIFIED].
No statement describes a pre-guard release, image or environment as having
a guard mode or as being MutationGuardMode::OFF.
No statement implies that merging THOTH-GQL-BATCH-01 deployed the guard;
the only such sentence asserts the opposite.
  PASS
```

**CG-13 state sweep.**

```text
no statement marks CG-13 resolved, closed or globally satisfied.
CG-13 is asserted OPEN in 8 places across the changed and existing records.
  PASS
```

**Runtime-operations gate sweep.**

```text
every occurrence of the gate is 'NOT SATISFIED', except the deliberate
forward-looking statements in THOTH-GQL-OPS-04 and the ai-delivery index
naming OPS-04 as the earliest point at which it MAY become satisfied, and
the OPS-02/-03 non-goals forbidding either task from recording it satisfied.
  PASS
```

**OPS-02/03/04 authorization sweep.**

```text
all three files carry 'Status: DRAFT' and 'Implementation: NOT AUTHORIZED'
in their first four lines.
  PASS
```

**OBSERVE / ENFORCE / BE-02 authorization sweep.**

```text
OFF -> OBSERVE:     NOT AUTHORIZED    (asserted in 4 records)
OBSERVE -> ENFORCE: NOT AUTHORIZED    (asserted in 4 records)
BE-02 runtime:      NOT AUTHORIZED    (asserted in the changed records)
No statement grants, implies or substitutes for any activation approval.
  PASS
```

**Prerequisite-branch absence check.**

```text
git fetch origin --prune
git branch -a --list '*guard-mode-entrypoint*' '*fleet-verification*' \
  '*runtime-ops-closure*'                                    -> none
git ls-remote --heads origin | grep -E 'guard-mode-entrypoint|
  fleet-verification|runtime-ops-closure'                    -> none
  PASS - no OPS-02/03/04 branch exists locally or remotely
```

**PR #788 and issue #765.**

```text
neither was opened for modification, commented on, or changed in any way.
  PASS
```

## 10. Manual verification

Environment: local checkout at the exact authorized base, plus read-only,
metadata-only access to the private authoritative deployment source, plus an
isolated throwaway Rust probe built **outside** this repository.

Steps and observed results:

1. **Re-derived the mode-control code path `[REPO]`** against
   `src/bin/arguments/mod.rs`, `src/bin/commands/mod.rs`,
   `src/bin/commands/start.rs`, `src/bin/thoth.rs`,
   `thoth-api-server/src/lib.rs` and `Dockerfile`. Confirmed: the mode is read
   once at process start; it is stored in the `HttpServer::new` closure as
   `app_data`; there is no signal handler, watcher, admin route or re-read
   anywhere in the workspace; no route or log exposes the effective mode; and
   `init` registers eleven arguments, none of them the guard argument.
2. **Re-derived the pinned `clap_builder` 4.6.0 chain** from the vendored
   registry source rather than from the specification's narrative:
   `get_one` → `MatchesError::unwrap(id, try_get_one(id))` → `try_get_arg_t` →
   `try_get_arg` → `verify_arg`, where `verify_arg` returns
   `Err(MatchesError::UnknownArgument)` **only** under `cfg(debug_assertions)`.
   Confirmed.
3. **Reproduced both branches in an isolated probe** outside this repository,
   mirroring the exact argument definition and the exact access pattern. No
   repository code was built, modified or added.

   ```text
   RELEASE  init  + ENFORCE  -> effective mode = OFF
   RELEASE  init  + OBSERVE  -> effective mode = OFF
   RELEASE  init  + unset    -> effective mode = OFF
   RELEASE  init  + invalid  -> effective mode = OFF, exit 0
   RELEASE  start + ENFORCE  -> effective mode = ENFORCE
   RELEASE  start + OBSERVE  -> effective mode = OBSERVE
   RELEASE  start + unset    -> effective mode = OFF
   RELEASE  start + invalid  -> usage error, exit 2
   DEBUG    init  + ENFORCE  -> panic "Mismatch between definition and
                                access of `mutation-guard-mode`", exit 101
   DEBUG    start + ENFORCE  -> effective mode = ENFORCE
   ```

   The `Dockerfile` builds `--release`, so the shipped image takes the release
   branch. **Confirmed, not refuted.**
4. **Confirmed the production container command `[EXTERNAL]`** by a scoped
   structural read: the production GraphQL API service supplies no command
   override, and the template resolves an unsupplied command to "no value", so
   the image default `init` applies. A different Thoth service in the same
   definition **does** override its command, so this is a deliberate
   configuration state rather than a template limitation.
5. **Confirmed which release each environment runs.** `[REPO]`: the most recent
   release tag and `origin/master` contain no `MutationGuardMode` and define no
   `mutation_guard_mode()`. `[EXTERNAL]`: both the production and the test
   GraphQL API services declare that same release. `[REPO + EXTERNAL]`: **both
   environments are PRE-GUARD** and neither may be described as
   `MutationGuardMode::OFF`.
6. **Confirmed the migration boundary `[REPO]`**: `init` runs migrations first
   and aborts on failure, then starts the API; `start graphql-api` runs no
   migrations. Recorded, not changed.
7. **Confirmed configuration precedence.** `[EXTERNAL]`: the container
   environment is built from a single numbered-parameter mechanism, with no
   secret-store or environment-file alternative, so no competing source exists.
   `[REPO]`: pinned `dotenv` 0.15.0 sets a variable only when it is not already
   set, and the production image is `FROM scratch`, so `.env` cannot override the
   container environment and cannot exist in the image.
8. **Confirmed rollout, autoscaling and rollback semantics `[EXTERNAL]`**:
   rolling replacement with concurrent old and new tasks, so a mixed-mode window
   is structurally guaranteed; autoscaling enabled with target tracking, so the
   expected population is a range with a live current value; a deployment circuit
   breaker with automatic rollback; and a documented change path requiring a
   pushed commit before deployment.
9. **Confirmed that no environment was transitioned and no guard-enabled
   candidate was activated.** No mode was set anywhere.

Evidence link: the pull-request diff, this report, and the control record's
per-conclusion evidence sources.

## 11. CI

```text
Exact-head CI results: GitHub PR #793 is terminal authority.
```

No head-specific CI result is transcribed into this file. The branch history was
rewritten during remediation (section 3.1), so any head recorded here would name
a commit that no longer exists — the recursive-transcription failure `ADR-0005`
exists to prevent.

CI status at the time of writing: **PASSING**, with **no failing job**.

Checks: the repository's changelog check plus the classifier-driven jobs.

**PASS and SKIPPED are classified independently.** The change is
documentation-only, so the repository's own classifier deliberately classifies
the Rust build/test/lint/format and migration jobs, and the staging image build,
out of scope. A job skipped by that classifier is **not** a failure and is not
counted as one. No job was skipped by error.

The exact per-job PASS / SKIPPED / FAIL breakdown at the current head is recorded
as a comment on PR #793 and is visible on the pull request itself, which remains
the authority.

## 11.1 Acceptance criteria — explicit AC-1 to AC-30 matrix

Every criterion of the approved specification, graded individually.

**This matrix is not optimised for a high PASS count.** A `THOTH-GQL-OPS-01`
implementation whose terminal disposition is **C — BLOCKED** is *expected* to
retain failed criteria: the failures are the evidence for the disposition. Four
criteria are recorded FAIL/BLOCKED, and each names the exact missing evidence and
who must obtain it.

```text
PASS         26
FAIL/BLOCKED  4   -- AC-1, AC-2, AC-7, AC-13
```

| Criterion | Status | Evidence source | Evidence class | Reason |
|---|---|---|---|---|
| **AC-1** production runtime owner identified as a role | **FAIL/BLOCKED** | no designation exists; `environments.md` §3 still lists production deployment owners as missing; CG-13 open; `ADR-0006` §7.2.1.1 records the owner as "not yet identifiable" | `[UNVERIFIED]` | Only **execution capability** is established (control record §2.1), from an access record. That is not accountable ownership (§2.0). What is missing is a **CTO designation**, which no further reading can supply. Write access must not be relabelled as runtime ownership. |
| **AC-2** observation sign-off owner identified as a role | **FAIL/BLOCKED** | `ADR-0006` §7.2.1.1 defers this and records it as not yet identifiable; `release-gates.md` §8 | `[UNVERIFIED]` | The control record §2.3 **proposes** the CTO, with its reasoning shown so the CTO has something concrete to confirm. A proposal awaiting confirmation is not an identified owner. `THOTH-GQL-OPS-04` must obtain explicit confirmation. |
| **AC-3** authoritative configuration source identified, incl. precedence | PASS | authoritative deployment source, structural keys only; `src/bin/thoth.rs`; pinned `dotenv` 0.15.0 `src/iter.rs`; `Dockerfile` | `[REPO + EXTERNAL]` | Sole env mechanism; no secret-store or env-file alternative; `dotenv` cannot override and cannot exist in a `FROM scratch` image. Precedence settled with no competing source. |
| **AC-4** exact configuration mechanism, incl. container command, confirmed or refuted | PASS | `Dockerfile`; `src/bin/commands/mod.rs`; `src/bin/thoth.rs`; `src/bin/commands/start.rs`; pinned `clap_builder` 4.6.0; the production service definition and the template's command-construction logic | `[REPO + EXTERNAL]` | **Confirmed, not refuted.** No command override; the template resolves an unsupplied command to "no value", so the image default `init` applies. A different Thoth service in the same definition *does* override, making the inheritance deliberate. |
| **AC-5** restart/redeploy requirement proven, `[REPO]` and `[EXTERNAL]` separated | PASS | `thoth-api-server/src/lib.rs`; absence of any reload path; the deployment source's documented update procedure | separated: `[REPO]` §5.1, `[EXTERNAL]` §5.2 | Read once at process start, no reload path → new process always required. Deployment side established separately. Conclusion: **both** a configuration change **and** a deployment. |
| **AC-6** propagation mechanism proven, incl. instances started during a transition | PASS | deployment configuration and autoscaling resources, structural keys only | `[EXTERNAL]` | Rolling replacement with concurrent old/new tasks; a task started mid-rollout may start under either revision and is treated as unknown-mode until attributed. |
| **AC-7** expected fleet as a decidable predicate from **live** orchestrator state | **FAIL/BLOCKED** | control record §6.3 defines the predicate; §6.2 establishes the autoscaled range | definition `[REPO + EXTERNAL]`; live value `[UNVERIFIED]` | The predicate is defined and the range established, but the criterion's own evidence line requires **live orchestrator state at execution time**, which this task is not authorized to read. Grading PASS on the definition alone would be the "specification is not the capability" error. |
| **AC-8** effective-mode fleet-verification mechanism **defined** | PASS | control record §7.2 (ten required properties) plus §4.2's finding that none exists | `[REPO]` | Satisfied by a **definition**. Explicitly not reported as verification having occurred; §7 states `a specification for a verifier != a verifier != a verified fleet`. |
| **AC-9** partial-fleet state detectable by that mechanism | PASS | runbook §6.2; control record §8.2 | `[REPO]` | Detection procedure defined in terms of the §7 mechanism. |
| **AC-10** partial fleet treated as **failed** activation, with abort and rollback trigger | PASS | runbook §6; control record §8.2 | `[REPO]` | "A partial fleet is a FAILED activation" stated in both, with abort criteria and rollback triggers per transition. |
| **AC-11** `OBSERVE`/`ENFORCE` window analysed separately from `OFF`/`OBSERVE` | PASS | `ADR-0006` §4.12.6.6 mode table; rollout semantics of the deployment source | `[REPO + EXTERNAL]` | Three windows tabulated separately; the observation-gap row is explicitly distinguished from the request-acceptance-inconsistency rows and the distinction is marked as not to be flattened. |
| **AC-12** rollback defined per transition, distinguishing all four kinds | PASS | control record §9.1; runbook §8.1; `ADR-0006` §7.3 | `[REPO + EXTERNAL]` | Four-row table in both; code revert explicitly recorded as **not** a remedy for a live configuration state. |
| **AC-13** rollback authority defined, and any difference from forward-change authority stated | **FAIL/BLOCKED** | control record §9.3; runbook §8.3 | `[UNVERIFIED]` | Execution *capability* is established (same team, same mechanism). Whether rollback **additionally requires CTO approval** is explicitly unresolved and is not invented. Until that approval authority is resolved, the criterion is not met. |
| **AC-14** no secret values anywhere in the diff | PASS | full-diff secret/configuration-value scan | `[REPO]` | 0 values. Word matches are the prohibitions themselves. Public security record reduced to the approved minimum fact. |
| **AC-15** no invented propagation or rollback duration anywhere | PASS | full-diff duration scan | `[REPO]` | 0 durations. Every timing field marked `TO BE MEASURED AT PREVIEW/STAGING GATE`. Rollback latency explicitly `[UNVERIFIED]` and not inferred from the forward change. |
| **AC-16** rehearsal requirement explicit, with four measurements and execution boundary settled | PASS | control record §10; runbook §7; `THOTH-GQL-OPS-04` §3.2 | `[REPO]` | Reassessed after the Finding 1 correction. The four measurements are stated, and the execution boundary is now internally consistent with the approved sequence: the rehearsal is owned by the **downstream preview/staging gate**, not by `THOTH-GQL-OPS-04`. |
| **AC-17** production activation remains unauthorized, and the document says so | PASS | control record §11; runbook header | `[REPO]` | `OFF -> OBSERVE` and `OBSERVE -> ENFORCE` recorded `NOT AUTHORIZED` in four records. |
| **AC-18** CG-13 disposition explicit, classified A/B/C, consistent with §12.1 | PASS | `control-gaps.md` as delivered; control record §13.1 | `[REPO]` | **C — BLOCKED**. B excluded on evidence; A forbidden while either capability gap is open. |
| **AC-19** monitoring/threshold work remains separately gated, not absorbed | PASS | control record §10.4 and §13.3; `THOTH-GQL-OPS-04` non-goals 10–13 | `[REPO]` | `ADR-0006` §8.3.2 referenced as a separate downstream gate in four places; OPS-04 explicitly forbidden from deriving or approving thresholds. |
| **AC-20** `BE-02` remains unauthorized and untouched | PASS | complete PR diff; PR #788 and issue #765 | `[REPO]` | Neither opened for modification, commented on, nor changed. |
| **AC-21** no environment transitioned; pre-guard recorded as pre-guard | PASS | the deployment source, unchanged by this task; control record §3.5 | `[REPO + EXTERNAL]` | Both production **and** test established pre-guard. No statement describes a pre-guard environment as `MutationGuardMode::OFF`. |
| **AC-22** fleet-verification mechanism specified but **not** implemented | PASS | complete PR diff | `[REPO]` | 0 runtime files. |
| **AC-23** capability gap 1 recorded as a deployment-path property, applicability established | PASS | control record §4.3; `[EXTERNAL]` confirmation of the container command | `[REPO + EXTERNAL]` | Stated as a property of the path, not of the deployed binary, and not weakened by production not yet running guard-enabled code. |
| **AC-24** capability gap 2 recorded as unclosed, **not** reported as closed by AC-8 | PASS | control record §4.2 and §7.3 | `[REPO]` | Explicitly separated; the "definition is not a verifier" rule is stated where AC-8 is satisfied. |
| **AC-25** CG-13 disposition is **C — BLOCKED** | PASS | control record §13.1; merge state of OPS-02/03 on `develop` | `[REPO]` | Neither prerequisite is implemented or merged, so C is mandatory. |
| **AC-26** runtime-operations gate recorded **NOT SATISFIED** in all four artefacts | PASS | specification, control record, implementation report, PR body | `[REPO]` | Stated identically in each. |
| **AC-27** three prerequisite specs exist, `DRAFT` / `NOT AUTHORIZED`, no branches | PASS | delivered files; `git branch -a`; `git ls-remote` | `[REPO]` | All three carry both markers in their first four lines. **No branch exists locally or remotely.** Specifications existing ≠ tasks authorized ≠ branches existing. |
| **AC-28** no migration-altering remediation selected/specified; override carries §13.1 classification | PASS | control record §4.4; runbook §9; `THOTH-GQL-OPS-02` §13.1; `THOTH-GQL-OPS-04` §13.1 | `[REPO]` | Classification reproduced verbatim at every mention. The OPS-02 mechanism is deliberately **not** selected. |
| **AC-29** runbook marked **PROVISIONAL** and states it is not executable until prerequisites merge | PASS | delivered runbook | `[REPO]` | Blocking banner plus a two-part status (§0.2) making clear that even after `THOTH-GQL-OPS-04` the runbook is not production-executable. |
| **AC-30** every private-source read complied with §2.2.5; incidental encounter escalated | PASS | this report §8.1 | `[REPO]` | Method statement records scoped, read-only, metadata-only reads and reports the encounter as an escalation at the approved minimum, without claiming none occurred. |

### 11.1.1 The four failures, and why they are not defects of this task

```text
AC-1   accountable production runtime owner       -> needs a CTO DESIGNATION
AC-2   observation sign-off owner                 -> needs a CTO CONFIRMATION
AC-7   live expected fleet population             -> needs a LIVE ORCHESTRATOR READ
AC-13  rollback approval authority                -> needs a CTO DECISION
```

None is obtainable by further discovery within this task's authorization. Three
require a decision only the CTO can make; the fourth requires production
orchestrator access this task is not authorized to perform. All four are recorded
in control record §14 as missing work, and all four are assigned to
`THOTH-GQL-OPS-04`.

Recording them as failures is what makes the disposition honest. Converting any
of them to PASS would require substituting a plausible answer for an absent one —
precisely what the specification's failure-behaviour rule forbids.

## 12. Rollout and rollback

Initial state after merge:

```text
deployed production release       = pre-guard, no guard mode exists
                                    [REPO + EXTERNAL]
deployed test release             = pre-guard, no guard mode exists
                                    [REPO + EXTERNAL]
guard-enabled candidate default   = OFF, loader store unavailable   [REPO]
environments transitioned         = none
production request acceptance     = unchanged
runtime-operations gate           = NOT SATISFIED
CG-13                             = OPEN
```

Activation required: none by this task, and none authorized by it. `OFF ->
OBSERVE` and `OBSERVE -> ENFORCE` each remain blocked on this gate in addition to
every other gate `ADR-0006` imposes, and each requires its own separate explicit
CTO production activation approval.

Feature flag/configuration: none introduced. No production configuration value
was written into this repository.

Migration sequence: not applicable.

Rollback/disable procedure: revert the merge commit. The change is documentation
only, so a revert has no production effect.

Note that the guard mode itself is **not** a kill switch: the control record
establishes that changing it requires a configuration change **and** a
deployment. What that record establishes about a guard-mode rollback is limited
to the **technical execution mechanism** — a rollback uses the same
configuration/deployment mechanism and is technically executed by the same
execution-capability team as a forward transition. Its actual latency/duration
remains `[UNVERIFIED]`, and whether it additionally requires CTO approval remains
`[UNVERIFIED]`. No authorization equivalence is inferred from sharing the
technical mechanism.

Monitoring required: none by this task. Service-health signals and activation
thresholds remain the separate `ADR-0006` section 8.3.2 gate and were **not**
absorbed here.

## 13. Known limitations and deferred work

**Capability gaps — missing capability, distinguished from missing evidence.**
Neither is closed by this task, and documenting them is not supplying them.

```text
CAPABILITY GAP 1 (the OPS-02 gap)                            OPEN
  The currently authoritative production deployment path cannot consume
  THOTH_GRAPHQL_MUTATION_GUARD_MODE. A guard-enabled release deployed
  through it would remain effectively OFF, so an OFF -> OBSERVE transition
  is not operationally PERFORMABLE -- not merely unauthorized.
  Stated as a property of the DEPLOYMENT PATH, not of the deployed binary.

CAPABILITY GAP 2 (the OPS-03 gap)                            OPEN
  No implemented, independently reviewed and merged mechanism can prove the
  effective mode of every serving instance, so a change could not be
  verified even if one could be made.
```

**Unresolved evidence — missing evidence is missing work.**

Each item is labelled with the part of the runbook's section 0.2 status that owns
it, so that no downstream-owned item is misread as a `THOTH-GQL-OPS-04`
obligation. This mirrors control record section 14 and runbook section 10.

| # | Missing | Class | Part | Who must obtain it |
|---|---|---|---|---|
| 1 | accountable production runtime owner — a designation, not an access record | `[UNVERIFIED]` | **1** | explicit CTO designation, obtained by `THOTH-GQL-OPS-04` |
| 2 | observation sign-off owner — confirmation of the proposal, or a different designation | `[UNVERIFIED]` | **1** | explicit CTO confirmation, obtained by `THOTH-GQL-OPS-04` |
| 3 | whether operational rollback additionally requires CTO approval | `[UNVERIFIED]` | **1** | explicit CTO decision, obtained by `THOTH-GQL-OPS-04` |
| 4 | live current expected replica population | `[UNVERIFIED]` | **1** | `THOTH-GQL-OPS-04`, from live orchestrator state |
| 5 | whether the running service matches its declared definition (drift) | `[UNVERIFIED]` | **1** | `THOTH-GQL-OPS-04` |
| 6 | the effective mode of any serving instance | `[UNVERIFIED]` | **1** | requires `THOTH-GQL-OPS-03` first |
| 7 | approved `OBSERVE` observation-window duration | `[UNVERIFIED]` | **2** | downstream activation gate |
| 8 | whether the current finite retention covers that approved window | `[UNVERIFIED]` | **2** | downstream activation gate, after item 7 |
| 9 | retention **remedy**, if item 8 proves one necessary — selected, implemented and verified | `[UNVERIFIED]` | **2** | downstream activation gate; **not** `THOTH-GQL-OPS-04` |
| 10 | propagation duration | `[UNVERIFIED]` | **2** | measured at the downstream preview/staging rehearsal |
| 11 | mixed-window duration bound | `[UNVERIFIED]` | **2** | measured at the downstream preview/staging rehearsal |
| 12 | rollback latency/duration | `[UNVERIFIED]` | **2** | measured at the downstream preview/staging rehearsal |

`THOTH-GQL-OPS-04` obtains items **1 to 6** only. Items 7 to 12 belong to
downstream gates, and its inability to answer them is **not** a reason to leave
the runtime-operations gate unresolved.

On retention specifically, the dependency runs 7 → 8 → 9: a remedy cannot be
selected before the duration it must cover is approved. `THOTH-GQL-OPS-04`
records the requirement and re-establishes that current retention is a finite
configured duration; it selects nothing.

**Deferred work, each separately specified, approved, reviewed and authorized:**
`THOTH-GQL-OPS-02`, `THOTH-GQL-OPS-03`, `THOTH-GQL-OPS-04`; then service-health
signals and activation thresholds; then the preview/staging performance and timed
rollback rehearsal; then explicit CTO `OFF -> OBSERVE` authorization.

**Explicitly outside CG-13 feature-specific scope and untouched:** migration
execution controls, backup and restore verification, approver mapping for
concerns other than this feature, and general Thoth runtime ownership.

## 14. Unresolved issues

1. **Secret-bearing configuration in the private authoritative deployment
   source.** Escalated in section 8.1 at the approved minimum. During authorized
   narrowly scoped read-only discovery, the implementing agent encountered
   existing secret-bearing configuration in that source. No secret value was
   copied into any repository file, PR text, report, changelog, prompt or commit
   message; no credential was used, changed or rotated. The exposure remains a
   separate CTO-controlled security matter and is outside this task's scope. No
   security issue was created or modified, and no further characterisation is
   recorded publicly.
2. **Observation-evidence retention is unresolved, and so is the window it must
   cover.** Established `[EXTERNAL]`: runtime log retention is configured to a
   finite duration. `[UNVERIFIED]`, and all three **downstream** (part 2): the
   approved `OBSERVE` observation-window duration; whether the configured
   retention covers it; and any remedy that coverage proves necessary. The
   binding requirement stands — observation evidence must be retained for at
   least the complete approved observation window and remain available through
   review and sign-off — but **no remedy is selected here, and selecting one is
   not `THOTH-GQL-OPS-04`'s job either**, because it would presuppose a window
   duration nobody has set.
3. **No environment currently runs guard-enabled code.** Both production and test
   are pre-guard, so any future non-production verification and the downstream
   rehearsal must first deploy a guard-enabled candidate — an additional
   prerequisite recorded in the control record and the runbook.
4. **Accountable production runtime ownership is not established.** Execution
   *capability* is established from an access record; an accountable *owner* is
   not, and no further reading can supply it — it requires a CTO designation.
   This blocks AC-1.
5. **The observation sign-off owner is a proposal, not a designation.** The
   control record proposes the CTO with its reasoning shown, precisely so the CTO
   has something concrete to confirm or reject. It is not treated as established.
   This blocks AC-2.
6. **Rollback approval authority is unresolved.** What is established is limited
   to the technical execution mechanism: rollback uses the same
   configuration/deployment mechanism and is technically executed by the same
   execution-capability team. Whether it *additionally* requires CTO approval is
   undecided, and no authorization equivalence is inferred from sharing that
   mechanism. This blocks AC-13.
7. **Rollback latency is unmeasured.** Sharing a mechanism is not sharing a
   measured time; the duration remains `[UNVERIFIED]` and belongs to the
   downstream preview/staging rehearsal.
8. **Observation-evidence retention has an unresolved dependency chain.** The
   requirement is established and current retention is a finite configured
   duration, but the observation-window duration, the coverage question and any
   remedy are all downstream — a remedy cannot be chosen before the duration it
   must cover is approved.

## 15. Agent self-assessment

The agent may identify risks but may not approve the task. This implementation
was not self-reviewed, and the pull request was left as a **DRAFT** that the
agent did not mark ready, approve or merge.

Suggested review focus:

1. **Disposition discipline** — confirm the record terminates at `C — BLOCKED`,
   that nothing in the diff converts C into A by implication, and that neither
   "we specified the verifier" nor "we specified OPS-02/03" is anywhere treated
   as delivery.
2. **Evidence provenance** — re-derive the `[REPO]` half of the pre-guard
   conclusion from `master`, the most recent release tag and the release
   workflow; confirm that no deployed-state statement anywhere in the diff is
   attributed to `[REPO]` alone; and confirm the newly added test-environment
   pre-guard conclusion carries `[REPO + EXTERNAL]`.
3. **The three added findings** (section 5.1) — the `init`-path invalid-value
   silence, the test environment being pre-guard, and the deployment circuit
   breaker. These are new relative to the merged specification. Confirm each is
   correct, correctly classified, and correctly scoped as a refinement rather
   than a contradiction.
4. **The four FAIL/BLOCKED grades (section 11.1)** — AC-1, AC-2, AC-7 and AC-13.
   Confirm each is the honest reading; specifically, that execution capability is
   nowhere relabelled as accountable runtime ownership, that a proposed sign-off
   owner is nowhere treated as a designated one, and that grading any of the four
   PASS on a definition, an access record or a derivation would have been the
   error. Confirm the matrix was not optimised for a high PASS count.
5. **Secret discipline and the public boundary** — confirm no production value,
   identifier, hostname, platform detail, scaling parameter or environment-variable
   value appears anywhere in the diff; that the section 8.1 method statement
   accurately reports that secret material *was* encountered rather than claiming
   it was not; and that the public record stops at the approved minimum fact,
   carrying no description of where the material was found, what kind it was, or
   what it relates to — **including in the squashed branch history**.
6. **Migration boundary** — confirm no remediation altering migration execution
   is selected or specified, and that every mention of a production
   container-command override carries the section 13.1.1 classification.
7. **Non-implementation** — confirm the diff contains zero runtime files, that
   the fleet-verification mechanism is specified rather than built, that no
   OPS-02/03/04 branch exists, and that no environment was transitioned.
8. **Authorization boundaries** — confirm merge authorization, `OFF -> OBSERVE`
   authorization and `OBSERVE -> ENFORCE` authorization remain three separate
   decisions and that nothing in the diff grants any of them.
9. **Runbook provisionality and the two-part status** — confirm the
   `PROVISIONAL` marking is unmissable, that no step reads as approved procedure,
   and that runbook section 0.2 correctly separates the runtime-operations
   procedure (part 1, resolvable by `THOTH-GQL-OPS-04`) from production
   transition readiness (part 2, resolvable only by the downstream gates).
10. **Dependency-order discipline** — confirm `THOTH-GQL-OPS-04` no longer
    absorbs the timed rehearsal, its four measurements, the preview/staging
    acceptance gate or threshold derivation, and that every document describes the
    same single lifecycle: OPS-02 → OPS-03 → OPS-04 → service-health/thresholds →
    preview/staging timed rehearsal → CTO activation. Confirm this matches
    `decision-register.md`, which was not changed on this point.
11. **Retention and rollback precision** — confirm the record claims only that
    retention is a **finite** configured duration `[EXTERNAL]`, that the
    observation-window duration and the coverage question are both
    `[UNVERIFIED]`, that no comparison between them is asserted, that no remedy
    is pre-selected, and that rollback is described as sharing the forward
    change's *mechanism and execution team* but **not** its latency.
12. **OPS-02 compatibility accuracy** — confirm the specification no longer
    claims that no existing invocation changes behaviour, and that the two
    intentional `init`-path changes are recorded as intentional and pinned by
    tests.
13. **Scope discipline** — confirm the work addresses only the mutation-guard
    runtime-mode-control subset and has not drifted into service-health
    thresholds, migration execution, restore verification or general CG-13
    closure.
