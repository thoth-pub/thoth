# THOTH-GQL-OPS-03-SPEC Implementation Report

Specification-preparation task for the output specification
[`THOTH-GQL-OPS-03`](../tasks/THOTH-GQL-OPS-03.md).

This report covers the **reconciliation and finalization of a specification**. It
does not report an implementation of that specification, it creates no
implementation branch, it changes no runtime or infrastructure code, and it
records no production action.

```text
THOTH-GQL-OPS-02:                  MERGED (PR #797)
THOTH-GQL-OPS-03 specification:    reconciled; approval candidate
THOTH-GQL-OPS-03 implementation:   NOT AUTHORIZED, NOT IMPLEMENTED
OPS-03 implementation branch:      ABSENT
section 3.2 selected boundary:     ADMINISTRATIVE / ORCHESTRATION-PLANE
                                   OR OUT-OF-BAND ONLY
public unauthenticated surface:    REJECTED
CG-13:                             OPEN
Runtime-operations gate:           NOT SATISFIED
Runbook:                           PROVISIONAL
OBSERVE / ENFORCE / BE-02 runtime: NOT AUTHORIZED
THOTH-GQL-OPS-04:                  NOT IMPLEMENTED, no branch
CL-1 (control limitation):         OPEN, unchanged
```

## 1. Repository state

Repository: `thoth-pub/thoth`
Programme: Shared Thoth GraphQL / Backend Architecture
Control task: `THOTH-GQL-OPS-03-SPEC`
Output specification: [`THOTH-GQL-OPS-03`](../tasks/THOTH-GQL-OPS-03.md)
Risk: HIGH
Workflow: STANDARD
Base branch: `develop`
Authorized exact base: `d0f71ee10d3c3f3482fd76796f1ded31cbb2de8b`
PR target: `develop`
Programme integration branch: None
Task branch:
`feature/shared-architecture/graphql-guard-mode-fleet-verification-spec`
Head commit: recorded on the pull request; the PR head is the authority
Pull request: opened as **DRAFT**, not approved by its author, not merged
Expected branch deletion after merge: YES
Final programme PR required: NO
Implementing model: Claude Opus 5
Reasoning level: HIGH
Independent reviewer: an independent model family that did not author this
specification work

**Durable rule, independent of lifecycle state.** The authoring agent may not
approve its own specification, and did not. Live review, authorization and merge
evidence is the GitHub pull-request record under
[`ADR-0005`](../../decisions/ADR-0005-terminal-merge-evidence.md) and is
deliberately not transcribed here.

### 1.1 Pre-branch verification

Performed before the branch was created:

```text
git fetch origin --prune
git rev-parse origin/develop
  -> d0f71ee10d3c3f3482fd76796f1ded31cbb2de8b   MATCHES the authorized base

git status --short
  -> clean

git branch -a --list '*graphql-guard-mode-fleet-verification*'
git ls-remote --heads origin \
  'refs/heads/feature/shared-architecture/graphql-guard-mode-fleet-verification' \
  'refs/heads/feature/shared-architecture/graphql-guard-mode-fleet-verification-spec'
  -> BOTH ABSENT, locally and remotely. The specification branch was created
     by this task; the implementation branch was NOT created and remains absent
```

`THOTH-GQL-OPS-02` merge evidence, from GitHub rather than from narrative:

```text
PR #797                 state MERGED, merged 2026-08-11T10:34:48Z
                        base develop
                        head feature/shared-architecture/
                             graphql-guard-mode-entrypoint
merge commit            d0f71ee10d3c3f3482fd76796f1ded31cbb2de8b
origin/develop head     d0f71ee10d3c3f3482fd76796f1ded31cbb2de8b

The authorized base IS the OPS-02 merge commit.
```

Re-derived from the code at that base rather than taken from the OPS-02 report:

```text
src/bin/commands/mod.rs   INIT registers .arg(arguments::mutation_guard_mode())
src/bin/commands/start.rs mutation_guard_mode(&ArgMatches) is the single
                          accessor; graphql_api() calls it
src/bin/thoth.rs          `init` dispatches through commands::run_init
thoth-api-server/src/lib.rs
                          the resolved mode is stored once as
                          app_data(Data::new(mutation_guard_mode)); ApiConfig
                          still exposes only api_name, api_version, api_schema,
                          public_url and schema_explorer_url, and the route set
                          (`/`, `/graphiql`, `/graphql`, `/schema.graphql`)
                          discloses no mode
```

So capability gap 1 is closed and capability gap 2 is open, exactly as the
reconciled specification now states.

## 2. Scope confirmation

Objective: reconcile the pre-`THOTH-GQL-OPS-02` `THOTH-GQL-OPS-03` specification
to the actual post-merge repository state, resolve its mandatory section 3.2
information-disclosure decision, re-check it as a coherent implementable
HIGH-risk task, and prepare it for fresh independent specification review.

Out-of-scope changes made: NONE. No Rust file, workflow, Dockerfile, migration,
schema, GraphQL document or infrastructure file appears in the diff.

## 3. Commits

Recorded on the pull request. This report is not the authority for commit
identifiers; the PR is.

## 4. Files changed

- `docs/engineering/ai-delivery/tasks/THOTH-GQL-OPS-03.md`
  - reason: the specification being reconciled and finalized.
  - behavioural effect: none. `Status: DRAFT`,
    `Implementation: NOT AUTHORIZED`.
- `docs/engineering/ai-delivery/implementation-reports/THOTH-GQL-OPS-03-SPEC-implementation-report.md`
  - reason: this report, per the repository's specification-task convention
    (compare `THOTH-GQL-OPS-01-SPEC`).
  - behavioural effect: none.
- `docs/engineering/ai-delivery/README.md`
  - reason: the task index still described `THOTH-GQL-OPS-02` as `DRAFT` with
    its branch forbidden, which the merge made false, and did not carry the
    now-resolved OPS-03 disclosure boundary.
  - behavioural effect: none. No gate, disposition or authorization changed.
- `docs/engineering/decisions/decision-register.md`
  - reason: the `ADR-0006` dependency sequence still described the `init` path
    as unable to consume the guard mode and described `THOTH-GQL-OPS-02` as
    `DRAFT` with no branch. Both are stale post-merge.
  - behavioural effect: none. `ADR-0006` itself is untouched; no decision,
    approval or gate state changed. CG-13 stays `OPEN`, the runtime-operations
    gate stays `NOT SATISFIED`, and both activations stay unauthorized.
- `CHANGELOG.md`
  - reason: repository convention requires an entry per PR.
  - behavioural effect: none.

Not changed, deliberately — see section 13:
`docs/engineering/repository-map/graphql-mutation-guard-runtime-operations.md`
and `docs/engineering/repository-map/graphql-mutation-guard-mode-transition-runbook.md`.

## 5. Specification decisions

### 5.1 The section 3.2 information-disclosure boundary — RESOLVED

```text
SELECTED

The effective-mode verification signal MUST be available only through an
orchestration/administrative-plane or equivalent out-of-band per-instance
mechanism.

A public unauthenticated effective-mode surface is REJECTED.
The public GraphQL schema MUST remain unchanged.
No public unauthenticated HTTP endpoint may expose OFF / OBSERVE / ENFORCE.

MINIMUM PERMITTED DISCLOSURE

  1. the process's actual effective MutationGuardMode; and
  2. the minimum runtime identity necessary to correlate that observation
     to the orchestrator's enumerated serving instance.

AND NOTHING ELSE -- no secret, credential, environment-variable value,
deployment configuration, publisher data, user data, request data, or
unnecessary topology or infrastructure metadata.
```

Reason, recorded rather than assumed: the guard mode describes server-side
**request-acceptance policy**, so publishing it has reconnaissance value;
`THOTH-GQL-OPS-03` does not need a public caller to know it; and the
administrative/out-of-band boundary satisfies the fleet-verification requirement
in full while avoiding a disclosure the task does not need.

Alternatives recorded in the specification with their dispositions: a public
unauthenticated HTTP surface (**rejected** — publishes policy, and cannot address
an individual replica behind the shared load balancer, so it fails the coverage
requirement on its own terms); a public GraphQL schema field (**rejected** — same
disclosure plus a forbidden schema/SDL change); an authenticated public surface
(**not selected** — it would introduce a new authorization decision the task
forbids, and still cannot address an individual replica).

**The decision fixes the boundary, not the mechanism.** The specification
deliberately does not name a transport: selecting the smallest mechanism inside
the approved boundary remains the implementing task's work under section 3.1. It
records only that at least one mechanism class inside the boundary is available
in-repository without infrastructure change, so the boundary is not vacuous. No
`ADR-0006` change and no `THOTH-GQL-OPS-01` amendment is required — `OPS-01`
section 3.5 left the disclosure open and required an explicit decision, and this
boundary is strictly narrower than what `OPS-01` would have permitted.

If implementation-time evidence proves no administrative or out-of-band mechanism
can satisfy section 3, the specification now makes that a **stop**, not a licence
to fall back to a public surface.

### 5.2 Stale post-`THOTH-GQL-OPS-02` statements corrected

| Stale statement | Correction |
|---|---|
| section 2, `[REPO]`: "a guard-enabled container running the image default `init` silently ignores the configured mode and runs unconditionally in `OFF`" | replaced by the generic class statement — **configured intent is not proof of process-effective mode** — with the closed defect removed as a current example |
| dependency header: OPS-02 absent; the disclosure decision listed as an unmet external dependency | OPS-02 recorded as **implemented, independently reviewed and merged**; the disclosure decision recorded as resolved by this candidate |
| section 1: gap 1 status unstated | gap 1 recorded closed in-repository by merged OPS-02; gap 2 recorded as the one still open |
| section 2 source list: control record section 4.3 cited as current behaviour | cited as **historical** evidence of the failure class, not as current `init` behaviour |
| section 4 non-goal 11: "implement the mode-control path — that is `THOTH-GQL-OPS-02`" | rewritten as "do not re-open, re-implement or modify" the merged OPS-02 path |
| section 4 non-goal 21: "implement `THOTH-GQL-OPS-02` or `THOTH-GQL-OPS-04`, or create their branches" | reduced to `THOTH-GQL-OPS-04` |
| AC-19: "the `THOTH-GQL-OPS-02` and `THOTH-GQL-OPS-04` branches do not exist" | rewritten: OPS-04 unimplemented with no branch; OPS-02 merged, neither re-opened nor modified, and no diff statement may describe `init` as ignoring the mode |
| section 10 integration test: "an instance started on a command path that does not consume the configured value" | rewritten so the divergence is constructed deliberately in the fixture and must **not** depend on, reintroduce or assume the closed `init` defect |
| section 11: rehearsal "after both this task and `THOTH-GQL-OPS-02` have merged" | OPS-02 has merged; this task is the remaining capability prerequisite, with the rehearsal still behind OPS-04 and the service-health gate |
| section 16: "independent of `THOTH-GQL-OPS-02`; the two may proceed in either order" | merge order recorded as after the now-merged OPS-02 and before OPS-04, with an explicit warning that `develop` has moved |
| `README.md`: OPS-02 `DRAFT`, branch must not exist | recorded as merged through PR #797, capability gap 1 closed |
| `decision-register.md`: the `init` path "does not register the guard argument … no `OFF -> OBSERVE` transition would be performable"; "`-02`, `-03` and `-04` are … `DRAFT` … none of their branches exists" | OPS-02 recorded merged with the consumed-value matrix; `DRAFT`/no-branch statement narrowed to `-03` and `-04`; explicit note that merging OPS-02 closed a capability gap and **no** gate |

### 5.3 Acceptance-criteria changes — strengthened, never weakened

No criterion was removed, relaxed, or made easier to satisfy. Existing numbering
is preserved so external references stay valid.

| Criterion | Change |
|---|---|
| **AC-4** | strengthened: complete enumeration is now **required**, not merely "supported", and a design that can only sample explicitly fails |
| **AC-6** | reworded generically, and now forbids satisfying the test by depending on the closed `init` defect |
| **AC-7** | strengthened: `UNKNOWN` must be distinct from `OFF` **in the result shape**, not only in prose |
| **AC-11** | rewritten from "document whichever option was selected, and record approval if it was public" to "conform to the **approved** boundary and record the mechanism, alternatives and assessment" |
| **AC-11.1** | **new**: a negative test must prove no unauthenticated public caller can obtain the mode from any route, body or header of the public listener. A public unauthenticated surface is a fail, not a documented trade-off |
| **AC-19** | rewritten for the post-merge world (see section 5.2) |
| **AC-22** | **new**: incomplete fleet coverage must fail closed, proven against a population containing an unreachable member |
| **AC-23** | **new**: minimum disclosure, with a field-by-field justification of every disclosed identity field |

Supporting changes that make the above objectively reviewable: section 5 gains
invariants 13 (no public unauthenticated surface in the merged state), 14
(minimum disclosure) and 15 (merged OPS-02 behaviour preserved); section 6.2
states that partial coverage cannot pass; section 6.5 constrains any added
surface to the boundary and explains why an HTTP surface satisfying both the
boundary and the no-new-authorization rule is one the public caller cannot reach
at all; section 8 constrains emitted signals to the boundary; section 10 adds a
public-listener negative test, a minimum-disclosure test, an incomplete-coverage
test and an OPS-02 regression suite in both build profiles; section 13 adds two
stop conditions; section 14 extends the required report contents.

## 6. Database and migration effects

```text
Database/data change:                         NONE
Migration files:                              NONE
Migration execution semantics changed:        NONE
GraphQL schema change:                        NONE
Public API change:                            NONE
```

Documentation only.

## 7. API and compatibility effects

GraphQL/API changes: NONE. Generated schema/client updates: NONE — no crate is
touched. Backwards compatibility: not applicable; no code changed. Deprecations:
NONE. Cross-repository dependencies: none created.

## 8. Authorization and security

Authorization paths changed: NONE. Roles/scopes: none created or altered.

Secret and protected-source handling:

```text
Private infrastructure repository inspected:        NO
Secret-bearing production configuration read:       NO
Production credential used or held:                 NO
Deployment performed or dispatched:                 NO
Real environment or fleet accessed:                 NO
Mode set in any environment:                        NO
Production configuration value, secret or resource
  identifier entering the diff:                     NONE
```

No external deployment fact was required by this task, so neither Route A nor
Route B evidence was needed and none is claimed. Every fact used is `[REPO]` or
GitHub lifecycle evidence.

**CL-1 is unchanged and remains OPEN.** The control limitation recorded in
`THOTH-GQL-OPS-03` section 6.6.1 — that `THOTH-GQL-OPS-01` section 2.2.5's
scoped-read rule conflicts with the stricter prohibition on implementing-agent
access to secret-bearing production configuration — is neither closed, narrowed
nor widened here. It is owned by the CTO/control owner and is not closable by an
agent. The reconciled specification keeps `THOTH-GQL-OPS-03` implementable
without any direct AI read of secret-bearing production configuration, and the
section 6.6 Route A / Route B boundary is preserved verbatim.

The `THOTH-GQL-OPS-02` AC-18 control-owner disposition is **not** cited as
precedent anywhere in this work, consistent with the bound its own report placed
on it.

## 9. Tests and checks

Documentation-only change; there is no code to test.

### Formatting and whitespace

```text
git diff --check    exit 0, no whitespace error reported
```

### Link and reference validation

Every relative Markdown link in the changed files was resolved against the
working tree, and every internal section reference added or edited was checked
against the headings actually present in the reconciled specification. Results
are recorded in section 10.

### Changed-path classification

```text
Runtime code (src/, thoth-*/src/):        0 files
Migrations:                               0 files
GraphQL schema / generated SDL:           0 files
Dockerfile / workflows / Cargo:           0 files
Documentation and control records:        5 files
```

## 10. Manual verification

```text
1. git diff --check                        clean
2. relative links in the four changed
   documents resolved on disk             all resolve
3. internal section references in
   THOTH-GQL-OPS-03.md (3.1, 3.2, 3.2.1
   to 3.2.3, 4, 5, 6.2, 6.3, 6.5, 6.6,
   6.6.1, 10, 13, 17.1, 17.2, AC-11,
   AC-11.1, AC-22, AC-23)                 all resolve to existing headings
                                          or list items
4. task-template completeness             all seventeen template sections
                                          present, headings unchanged
5. stale-premise sweep: no statement
   says current `init` ignores the guard
   mode; none says OPS-02 is
   unimplemented; none says the verifier
   exists; none says any fleet is
   verified; none says the gate is
   satisfied; none closes CG-13; none
   authorizes OBSERVE, ENFORCE or BE-02   confirmed by grep over the diff and
                                          by reading each changed section
6. every mention of a public
   effective-mode surface                 either rejects it or describes it as
                                          a rejected/not-selected alternative
7. implementation branch                  ABSENT locally and on origin,
                                          re-checked after the edits
8. no secret or configuration value in
   the diff                               confirmed; the diff contains no
                                          value of any environment variable
```

## 11. CI

Exact-head CI and its per-job PASS / SKIPPED / FAIL classification are recorded
on the pull request, which is the authority. This is a documentation-only change,
so a legitimate skip of the Rust jobs by the repository's path classifier is an
expected outcome and is not treated as a missing check. No required check may be
red.

## 12. Rollout and rollback

Initial state after merge: unchanged in every respect. A reconciled specification
becomes reachable from `develop`; no capability, mode, environment or
authorization changes.

```text
deployed production release       = pre-guard (no guard mode exists)
guard-enabled candidate default   = OFF, loader store unavailable
environments transitioned         = none
production request acceptance     = unchanged
runtime-operations gate           = NOT SATISFIED
```

Activation required: none by this task, and none granted. Rollback: revert the
merge commit; it is a documentation revert with no runtime effect.

## 13. Known limitations and deferred work

- **The `THOTH-GQL-OPS-01` deliverables still carry pre-OPS-02 language, and this
  task did not edit them.** The
  [runtime-operations control record](../../repository-map/graphql-mutation-guard-runtime-operations.md)
  section 4.3 and section 13.1, and the
  [mode-transition runbook](../../repository-map/graphql-mutation-guard-mode-transition-runbook.md)
  section 0, still describe capability gap 1 as open and the `init` path as
  silently ignoring the mode. The control record is an evidence document written
  at `THOTH-GQL-OPS-01`'s own base and carries an explicit authority condition,
  so correcting it is a control decision for its owner rather than a side effect
  of an OPS-03 specification task, and both files sit outside this task's
  permitted paths. **Surfaced to CTO control rather than silently edited.**
  `THOTH-GQL-OPS-04` already owns re-establishing every external fact and
  finalising the runbook; whether an earlier bounded reconciliation is wanted is
  the control owner's call. Nothing in this deferral affects the OPS-03
  specification's coherence: the reconciled specification cites section 4.3
  explicitly as historical.
- **The section 3.2 boundary is selected but not yet approved.** It is the
  decision this approval candidate proposes. Until independent review and
  explicit CTO specification approval, the specification remains `DRAFT` with
  implementation `NOT AUTHORIZED`.
- **CL-1 remains open** (section 8) and is not closable by an agent.
- **Capability gap 2 remains open.** Nothing here implements a verifier, and a
  specification for a verifier is neither a verifier nor a verified fleet.

## 14. Unresolved issues

1. **Specification approval itself.** Not granted by this task, and not
   grantable by its author.
2. **The pre-OPS-02 language in the two `THOTH-GQL-OPS-01` deliverables**
   (section 13). Owner: CTO / control owner, with `THOTH-GQL-OPS-04` as the
   already-specified natural home.
3. **CL-1**, unchanged and open.
4. Nothing else is outstanding against this task's own boundary.

## 15. Agent self-assessment

The agent may identify risks but may not approve its own specification. This work
was not self-reviewed: a draft pull request was opened and readiness, review,
approval, merge authorization and merge were left to the actors who own those
decisions.

Suggested review focus:

1. **The section 3.2 decision.** Confirm the boundary is stated identically in
   scope, non-goals, invariants, required behaviour, acceptance criteria, tests,
   stop conditions and approval, and that no sentence anywhere leaves a public
   unauthenticated surface open as a fallback.
2. **That no criterion was weakened.** Compare AC-1 to AC-21 against the base
   revision; three criteria were tightened, three added, and none relaxed.
3. **The stale-premise sweep.** Confirm from the diff, not from this report, that
   the closed `init` defect appears only as explicitly historical context, and
   that the silent-adoption class survives as a class the verifier must still
   catch.
4. **The dependency reconciliation.** Confirm that specification approval is
   nowhere presented as implementation authorization, and that the implementation
   branch is absent.
5. **The deferral in section 13.** Confirm that leaving the `THOTH-GQL-OPS-01`
   deliverables untouched is the right boundary call rather than an omission,
   and that the OPS-03 specification does not depend on their stale wording.
