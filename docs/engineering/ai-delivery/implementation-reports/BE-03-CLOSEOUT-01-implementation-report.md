# BE-03-CLOSEOUT-01 Implementation Report

## 1. Repository state

Repository: `thoth-pub/thoth`
Workflow: STANDARD
Base branch: `develop`
Base commit: `b51bcc0905ac17fc0c142b2002b11fec711331a3`
PR target: `develop`
Programme integration branch: None
Task branch: `feature/publisher-services/be-03-closeout`
Head commit: recorded in the pull request; this report is written at the branch
head that carries it
Pull request: draft pull request against `develop`; live state is the GitHub
pull-request record
Expected branch deletion after merge: YES
Final programme PR required: NO
Implementing model: Claude Opus 5
Reasoning level: Extra High / xhigh

### 1.1 Exact authorized base

The CTO authorized implementation against exact `develop` head:

```text
b51bcc0905ac17fc0c142b2002b11fec711331a3
```

### 1.2 Preflight, performed before branch creation and before any edit

```text
origin/develop                   = b51bcc0905ac17fc0c142b2002b11fec711331a3  MATCH
b51bcc09 parents                 = a4585a8d89166577da5ce6f46ce51ddb134b3f7e (first)
                                   d952a83206ad846c7a01f70c29fe694bc8cd5561 (spec content)
b51bcc09 identity                = merge commit of PR #812
PR #812                          = MERGED, base develop,
                                   head feature/publisher-services/be-03-closeout-spec,
                                   mergeCommit b51bcc09
BE-03-CLOSEOUT-01.md at base     = present; blob 2f23caf076ba963bc38b24d3c8b8f5a5e9261429,
                                   byte-identical to the blob at d952a832
PR #809                          = MERGED, base develop,
                                   mergeCommit 3ba4452c316399d80cd8d85e7d5e1bd05e252664
PR #809 ancestry                 = 3ba4452c is an ancestor of b51bcc09 (verified with
                                   git merge-base --is-ancestor)
BE-03 implementation at base     = thoth-api/migrations/20260812_v1.7.0/{up,down}.sql present
Local branch  feature/publisher-services/be-03-closeout  = does not exist
Remote branch feature/publisher-services/be-03-closeout  = does not exist
Open BE-03-CLOSEOUT-01 implementation PR                 = none
                                   (open PRs: #799, #752, #744, #742, #668)
Committed competing closeout implementation record       = none
Working tree                     = clean
```

`origin/develop` matched the authorized SHA exactly, so no intervening-commit
inspection was required. The pre-existing branch
`feature/publisher-services/be-03-closeout-spec` is the merged specification
branch, not a competing implementation branch.

Repository instructions read before any edit: root
[`AGENTS.md`](../../../../AGENTS.md),
[`docs/engineering/AGENTS.md`](../../AGENTS.md), the approved specification
[`BE-03-CLOSEOUT-01`](../tasks/BE-03-CLOSEOUT-01.md) and
[ADR-0005](../../decisions/ADR-0005-terminal-merge-evidence.md). No more
specific `AGENTS.md` applies: the diff touches no workspace member and no
workflow directory.

Branch creation:

```bash
git switch -c feature/publisher-services/be-03-closeout b51bcc0905ac17fc0c142b2002b11fec711331a3
```

## 2. Scope confirmation

Approved specification:
[`BE-03-CLOSEOUT-01`](../tasks/BE-03-CLOSEOUT-01.md)

Implemented objective: correct the materially incorrect active Publisher
Services programme and dependency state left behind after the BE-03
implementation merged, so that active controls durably record BE-03 as
`CLOSED - INACTIVE FOUNDATION` and record each downstream dependency
accurately — satisfied where the merge satisfied it, and still blocked
everywhere else.

Out-of-scope changes made: NONE. Every changed path is one of the paths the
specification lists in its section 3. No additional documentation path was
required.

## 3. Commits

- `docs(publisher-services): reconcile BE-03 post-merge state` - the single
  bounded commit carrying this closeout. Its SHA is the branch head recorded in
  the pull request.

## 4. Files changed

- `docs/publisher-services/task-status.md`
  - reason: the active programme tracker recorded BE-03 as
    `IMPLEMENTATION IN REVIEW` with a `(draft)` PR cell, an acceptance cell
    reading `IMPLEMENTATION DELIVERED, NOT MERGED`, a `Last updated` note and
    next-actions 10, 11 and 12 describing BE-03 as unmerged and awaiting fresh
    review and CTO merge authorization, and listed BE-03 as an unsatisfied
    blocking dependency of BE-04, MIG-01, APP-01 and APP-02.
  - behavioural effect: none. BE-03 is now recorded as `CLOSED` with acceptance
    `CLOSED - INACTIVE FOUNDATION`; BE-04, MIG-01, APP-01 and APP-02 record the
    BE-03 dependency as satisfied while retaining every remaining blocker and
    their `BLOCKED` / `NOT STARTED` status; the transient review-lifecycle
    narrative is replaced by durable prose that points at the pull-request
    record under ADR-0005.

- `docs/publisher-services/README.md`
  - reason: the header status line and the section 5 decision block asserted
    `BE-03 IMPLEMENTATION NOT AUTHORIZED`; gating reason 1 repeated it; the
    closing paragraph said an approved specification does not "unlock `BE-03`".
  - behavioural effect: none. The decision block now records BE-03 as a closed
    inactive foundation and states the downstream position for BE-04, MIG-01,
    APP-01 and APP-02 explicitly, together with the expanded
    `NOT AUTHORIZED` list.

- `docs/publisher-services/rollout-plan.md`
  - reason: the BE-02 implementation state block ended with "`BE-03`
    implementation is `NOT AUTHORIZED`", and section 2.2 bound "the later
    `BE-03`" in the future tense with a future-tense item 1.
  - behavioural effect: none. A `BE-03 implementation state` block records the
    merged inactive foundation on the same terms as the BE-02 block; section
    2.2 now binds APP-01 and states that the reserved control binds against
    BE-03's reviewed implementation head. The control's substance is unchanged.

- `docs/publisher-services/decisions.md`
  - reason: the section 3a APP-01 reconciliation asserted in the present tense
    that APP-01 "remains blocked on BE-03 exposing the approved protected
    API", which the BE-03 merge falsified; the `Last updated` line needed to
    reflect this change.
  - behavioural effect: none. See section 5.2 for the explicit control ruling
    and its application. The self-resolving ADR-0005 authority-condition
    construction is preserved verbatim.

- `docs/engineering/ai-delivery/tasks/BE-03.md`
  - reason: the approved BE-03 specification carried `Status: DRAFT`, a header
    implementation-authorization block reading "**separate and absent** ... The
    branch `feature/publisher-services/be-03` must not exist", and a section 23
    lifecycle boundary asserting "BE-03 implementation status is
    `NOT AUTHORIZED`".
  - behavioural effect: none. Only the three lifecycle-boundary sites the
    specification authorizes are changed. No requirement, invariant,
    architecture, API contract, authorization matrix, migration decision,
    acceptance criterion, test obligation, operational consequence or BE-04
    transaction seam is altered.

- `docs/engineering/ai-delivery/tasks/BE-03-CLOSEOUT-01.md`
  - reason: this task's own durable specification decision state was `DRAFT`
    and its section 18 approval record was empty, both of which the
    specification's CTO approval and merge through PR #812 falsified.
  - behavioural effect: none. `Status: APPROVED`; section 18 records the
    durable implementation authorization and the exact authorized base. The
    header rule "Implementation authorization: **separate.**" is retained
    unchanged as a durable rule.

- `docs/engineering/ai-delivery/implementation-reports/BE-03-CLOSEOUT-01-implementation-report.md`
  - reason: required by the specification section 14 and root `AGENTS.md`
    section 14.
  - behavioural effect: none; new evidence record.

- `CHANGELOG.md`
  - reason: root `AGENTS.md` section 13 requires an entry under
    `## [Unreleased]`.
  - behavioural effect: none. One entry added as the first item under the
    existing `### Added` heading. No heading was created or duplicated.

## 5. Implementation decisions

Decisions made within the approved design:

1. **Durable rather than transient prose throughout.** Every corrected sentence
   was re-read against the ADR-0005 section 6 test and stays truthful before
   review, after review, before merge and after merge. No corrected sentence
   says "awaiting review", "pending merge", "draft PR" or "merge authorization
   outstanding" as current programme state. This closeout therefore does not
   create the need for another closeout when it merges.
2. **No lifecycle metadata transcribed.** No independent review identifier, CTO
   approval identifier, merge-authorization identifier, PR #809 merge commit
   SHA or merge timestamp is written into any repository file. Where the
   previous prose narrated the BE-03 review history (the earlier BLOCKED
   exact-head review and the authorized bounded remediation), that narrative
   was removed from the active tracker rather than updated, because ADR-0005
   section 5 makes the GitHub pull-request record the authority for it. Pointers
   to PR #808 and PR #809 are retained; they are references, not transcriptions.
3. **`BE-03.md` corrections confined to the three authorized sites.** The
   `Status` line uses the house form established by `ADR-01.md`
   (`APPROVED AND REPOSITORY-AUTHORITATIVE - ...`) rather than a bare
   `APPROVED`, so the line records both the specification's durable decision
   state and the inactive-foundation delivery boundary in one place.
4. **`rollout-plan.md` gains a `BE-03 implementation state` block** modelled on
   the existing BE-02 block rather than an inline clause, so the inactive
   foundation and the satisfied-but-not-ready downstream position are stated
   once, coherently, in the document that owns rollout state. The block is
   deliberately undated: BE-03's merge date is transient GitHub evidence.
5. **`README.md` decision block expanded per downstream task.** The block
   previously carried a single BE-03 line. It now states BE-04, MIG-01, APP-01
   and APP-02 positions explicitly, because a bare "BE-03 CLOSED" would leave a
   reader to infer the downstream consequence, which is exactly the inference
   the specification forbids.
6. **Two `Last updated` metadata lines refreshed** (`task-status.md` line 7 and
   `decisions.md` line 4). Both were accurate at the authorized base and became
   inaccurate as a direct consequence of this change; refreshing them follows
   the BE-02-CLOSEOUT-01 precedent.

Deviations from the specification: NONE.

Scope deviations on touched paths: NONE. The changed set is exactly the
specification's expected set. `platform-inventory.md`, `acceptance-matrix.md`,
`master-issue.md` and `control-gaps.md` were searched and left untouched, as
expected.

### 5.1 Two fresh findings not present in the authoring-time Annex A

Both fall inside an already-expected touched path and inside the approved
scope; neither required a new path.

1. `task-status.md` next-action 6: "BE-03 later exposes the protected
   package/capability contract under its own approval gates" — a future-tense
   BE-03 lifecycle assertion about something that has now happened. Classified
   `ACTIVE STALE STATE - CORRECT` and corrected to the past tense, retaining the
   "under its own approval gates" qualifier and adding "as a merged but equally
   inactive foundation".
2. `task-status.md` next-action 9: "BE-03 later exposes the protected service
   configuration under its own approval gates" — same class, same correction.

Annex A did not record either, which is why the specification requires the
search to be repeated rather than applied from the annex.

### 5.2 `decisions.md` section 3a - classification and CTO control ruling

The authoring-time Annex A classifies `decisions.md` section 3a
`CURRENT AND CORRECT - PRESERVE` "in its entirety", while the normative
specification section 2 and its `ACTIVE STALE STATE` list identify one
present-tense sentence inside the same section as stale. The CTO resolved this
wording collision explicitly, and the normative sections control over the
informative annex.

**Ruling applied.** The self-resolving ADR-0005 authority-condition
construction is preserved as written, including:

- `Decision state: PROPOSED IN THIS SPECIFICATION CANDIDATE` (line 247,
  unchanged);
- the two-part authority condition (lines 251-256, unchanged);
- the rule that once both parts hold it is an `APPROVED PROGRAMME DECISION`
  "without requiring a separate lifecycle-status edit to this file" (lines
  258-261, unchanged);
- the explanation of why a literal mutable `APPROVED` token is not required
  (lines 263-269, unchanged);
- the deliberate "This decision candidate **refines and, in that narrow
  respect, supersedes**..." phrasing that belongs to that construction (line
  336, unchanged).

`PROPOSED` was **not** mechanically replaced with `APPROVED`, and no
architecture was rewritten.

**Corrected.** One sentence only, in the APP-01 reconciliation:

```text
before: ... and remains blocked on BE-03 exposing the approved protected API,
        app readiness controls, the exact-SHA schema pinning control and its
        own approved bounded specification.

after:  ... Its dependency on BE-03 exposing the approved protected API is
        satisfied for the configuration-only surface enumerated below, BE-03
        having merged that surface; APP-01 itself remains blocked on app
        readiness controls, the exact-SHA schema pinning control and its own
        approved bounded specification, and its job-aware elements remain
        dependent on BE-04 as set out below.
```

Classification: `ACTIVE STALE STATE - CORRECT`.

Applying the ruling required **no** change to the BE-03/BE-04/APP-01
architecture. The enumerated configuration-only scope (items 1-6 immediately
below the corrected sentence), the four BE-04-dependent elements, the APP-02
dependency on both BE-03 and BE-04, and the consequences list are all unchanged
and continue to say exactly what they said. The stop condition "if applying this
ruling would require changing the BE-03/BE-04/APP-01 architecture, STOP BLOCKED"
therefore did not fire.

## 6. Database and migration effects

Migration added: NO

No migration, schema, catalog or data change of any kind.
`thoth-api/src/schema.rs` and `thoth-api/migrations/` are untouched, as proved
by the negative path check in section 9.

## 7. API and compatibility effects

GraphQL/API changes: none.
Generated schema/client updates: none; `thoth-client/assets/schema.graphql` is
byte-identical to the base.
Backwards compatibility: unaffected.
Deprecations: none.
Cross-repository dependencies: none created. The reserved BE-03/APP-01 exact-SHA
schema-pinning control continues to bind against BE-03's reviewed implementation
head, not against this task's head; section 2.2 of `rollout-plan.md` now says so
explicitly. This task produces no new contract for a downstream repository to
pin.

## 8. Authorization and security

Authorization paths changed: none.
Roles/scopes involved: none. `thoth-api/src/policy.rs` and every authorization
test are untouched.
Negative authorization tests: not applicable; no authorization code is in this
diff.
Secret or personal-data handling: none. No credential, token, endpoint, bucket
or account identity is introduced into the diff.
Security limitations: none introduced.

## 9. Tests and checks

Root `AGENTS.md` section 8 prescribes the documentation-only evidence set and
reserves the full workspace gate for Rust/domain changes. No file under any
workspace member is modified, so the workspace gate has no changed input and was
not run.

### Formatting

Command:

```text
git diff --check
```

Result:

```text
no output; no whitespace error
```

### Path containment

Command:

```text
git diff --name-only b51bcc0905ac17fc0c142b2002b11fec711331a3..HEAD
```

Result:

```text
CHANGELOG.md
docs/engineering/ai-delivery/implementation-reports/BE-03-CLOSEOUT-01-implementation-report.md
docs/engineering/ai-delivery/tasks/BE-03-CLOSEOUT-01.md
docs/engineering/ai-delivery/tasks/BE-03.md
docs/publisher-services/README.md
docs/publisher-services/decisions.md
docs/publisher-services/rollout-plan.md
docs/publisher-services/task-status.md
```

Every path matches `^docs/` or is exactly `CHANGELOG.md`.

### Negative runtime-path proof

Command:

```text
git diff --name-only b51bcc0905ac17fc0c142b2002b11fec711331a3..HEAD \
  | grep -E '^(thoth-api|thoth-api-server|thoth-client|thoth-errors|thoth-export-server|\.github|Cargo\.)'
```

Result:

```text
no output
```

No runtime, migration, schema, GraphQL, generated client, workflow or Cargo file
changed.

### Unit tests

Not applicable: documentation-only change, no workspace member modified.

### Integration/database tests

Not applicable: no migration, schema or database-backed code is touched.

### Lint/static analysis

Not applicable to the changed paths.

### Documentation link verification

Command:

```text
resolve every relative markdown link target in the changed files against the
filesystem
```

Result:

```text
relative links checked: 92
broken: 0
```

## 10. Manual verification

Environment: local checkout of `feature/publisher-services/be-03-closeout` at
the branch head, created from `b51bcc0905ac17fc0c142b2002b11fec711331a3`.

Steps: the classified search was performed from the authorized base with
`git grep -n 'BE-03' -- docs/ CHANGELOG.md` (27 files), every hit was read in
context, and the search was re-run after the edits.

### 10.1 Complete fresh classified stale-state findings

#### `ACTIVE STALE STATE - CORRECT`

- `docs/publisher-services/task-status.md` line 7 — `Last updated` note
  describing BE-03 as delivered as a draft pull request with the new exact head
  "awaiting a fresh independent review". Corrected.
- `docs/publisher-services/task-status.md` BE-03 row — status
  `IMPLEMENTATION IN REVIEW`; blocking-dependency cell listing a fresh
  independent review and CTO merge authorization as remaining gates; PR cell
  `(draft)`; acceptance cell `IMPLEMENTATION DELIVERED, NOT MERGED`. Corrected
  to `CLOSED` / `CLOSED - INACTIVE FOUNDATION`.
- `docs/publisher-services/task-status.md` BE-04 row — BE-03 listed as an
  unsatisfied blocking dependency. Corrected to satisfied, with BE-04 retained
  as `BLOCKED` / `NOT STARTED` and the explicit statement that no
  `distribution_job`, `distribution_job_target`, `distribution_job_attempt`,
  automatic back-catalogue job creation or dissemination exists.
- `docs/publisher-services/task-status.md` MIG-01 row — same, corrected to
  satisfied with MIG-01 retained as `CRITICAL` / `BLOCKED` and its audit,
  backfill, dry-run and CG-13 prerequisites retained.
- `docs/publisher-services/task-status.md` APP-01 row — BE-03 listed as an
  unsatisfied blocking dependency, and "the candidate phase boundary".
  Corrected: the backend-contract dependency is satisfied for the
  configuration-only scope, APP-01 remains `BLOCKED` by its other controls, the
  job-aware elements remain BE-04-dependent, and the boundary is referred to as
  approved.
- `docs/publisher-services/task-status.md` APP-02 row — BE-03 listed as an
  unsatisfied blocking dependency. Corrected to satisfied-only, with APP-02
  retained as blocked on BE-04 and APP-01 and an explicit statement that this
  does not make it ready.
- `docs/publisher-services/task-status.md` next-action 6 — future-tense "BE-03
  later exposes the protected package/capability contract". Fresh finding, not
  in Annex A; corrected.
- `docs/publisher-services/task-status.md` next-action 9 — future-tense "BE-03
  later exposes the protected service configuration". Fresh finding, not in
  Annex A; corrected.
- `docs/publisher-services/task-status.md` next-action 10 — "delivered ... as a
  **draft** pull request", the exact-head review narrative, "BE-03 is **not
  merged**", the fresh-review requirement, "explicit CTO merge authorization
  remains outstanding", and "programme-decision candidate". Corrected to durable
  prose.
- `docs/publisher-services/task-status.md` next-action 11 — "APP-01 remains
  blocked pending BE-03" and "the candidate phase boundary". Corrected.
- `docs/publisher-services/task-status.md` next-action 12 — "the separately
  authorized, delivered-but-unmerged BE-03 implementation". Corrected, and the
  unauthorized-action list extended to name package commercial backfill, durable
  job creation, dissemination and workflow change/dispatch explicitly.
- `docs/publisher-services/README.md` line 3 header status — did not record
  BE-03. Corrected.
- `docs/publisher-services/README.md` section 5 decision block — `BE-03
  DEPENDENCIES ON BE-01 AND BE-02 SATISFIED; BE-03 IMPLEMENTATION NOT
  AUTHORIZED`. Corrected, and the per-downstream position added.
- `docs/publisher-services/README.md` gating reason 1 — "`BE-03` implementation
  remains `NOT AUTHORIZED` pending its own approved bounded specification and
  separate explicit authorization". Corrected.
- `docs/publisher-services/README.md` closing paragraph — an approved
  specification does not "unlock `BE-03`". Corrected by removing `BE-03` from
  the list only.
- `docs/publisher-services/rollout-plan.md` BE-02 implementation state block —
  "`BE-03` implementation is `NOT AUTHORIZED`". Corrected, and a `BE-03
  implementation state` block added.
- `docs/publisher-services/rollout-plan.md` section 2.2 — "It binds the later
  `BE-03` and `APP-01` tasks" and item 1's future-tense "`BE-03` produces an
  exact generated GraphQL SDL at its reviewed head". Corrected without changing
  the control's substance.
- `docs/publisher-services/decisions.md` section 3a APP-01 reconciliation — the
  present-tense APP-01 dependency assertion. Corrected under the CTO control
  ruling; see section 5.2.
- `docs/engineering/ai-delivery/tasks/BE-03.md` line 3 — `Status: DRAFT`.
  Corrected.
- `docs/engineering/ai-delivery/tasks/BE-03.md` header
  implementation-authorization block — "**separate and absent** ... must not
  exist until the CTO separately and explicitly authorizes implementation".
  Corrected.
- `docs/engineering/ai-delivery/tasks/BE-03.md` section 23 — "BE-03
  implementation status is `NOT AUTHORIZED`. The branch
  `feature/publisher-services/be-03` must not exist...". Corrected.
- `docs/engineering/ai-delivery/tasks/BE-03-CLOSEOUT-01.md` line 3 —
  `Status: DRAFT`, falsified by this specification's own CTO approval and merge
  through PR #812. Corrected to `APPROVED`.
- `docs/engineering/ai-delivery/tasks/BE-03-CLOSEOUT-01.md` section 18 — empty
  approval record. Completed with the durable implementation authorization.

#### `HISTORICAL RECORD - PRESERVE`

- `docs/engineering/ai-delivery/implementation-reports/BE-03-implementation-report.md`
  in its entirety (30 BE-03 references), including its base and head commits,
  exact test commands, CI record and every reference to
  `thoth-api/migrations/20260813_v1.7.0` and to `20260812_v1.7.0` as BE-02's
  migration. Untouched.
- `docs/engineering/ai-delivery/implementation-reports/BE-03-SPEC-implementation-report.md`
  (111 references) — untouched.
- `docs/engineering/ai-delivery/implementation-reports/BE-03-CLOSEOUT-01-SPEC-implementation-report.md`
  (43 references) — the specification task's own exact-head record. Untouched.
- `docs/engineering/ai-delivery/implementation-reports/BE-02-implementation-report.md`,
  `BE-02-CLOSEOUT-01-implementation-report.md`, `BE-02-SPEC-implementation-report.md`,
  `BE-01-implementation-report.md`, `BE-01-SPEC-implementation-report.md`,
  `ADR-01-SPEC-implementation-report.md`,
  `ADR-0002-APPROVE-implementation-report.md`,
  `P0-01-CLOSEOUT-implementation-report.md`,
  `P0-01-FINALIZE-implementation-report.md` — untouched.
- `docs/engineering/ai-delivery/tasks/BE-02-CLOSEOUT-01.md` lines 32, 74, 100
  and 252 — the completed BE-02 closeout's own scope, non-goals and merge-order
  record, correct as written for that task. Untouched.
- `docs/engineering/ai-delivery/tasks/ADR-01-CLOSEOUT-01.md` line 183 — that
  task's non-goal. Untouched.
- `docs/publisher-services/adr-01-evidence-matrix.md` line 256 — "future
  BE-02/BE-03 implementation; not yet implemented at the inspected commits", an
  ADR-01 evidence record explicitly bound to the commits it inspected.
  Untouched.
- `CHANGELOG.md`'s existing `BE-03` entry from PR #809, `BE-03-SPEC` entry from
  PR #808 and `BE-03-CLOSEOUT-01-SPEC` entry from PR #812 — append-only records
  of what each pull request contained when written, including the
  `BE-03-SPEC` entry's closing "`BE-03` implementation remains **NOT
  AUTHORIZED** and `feature/publisher-services/be-03` must not exist". Preserved,
  on the same basis BE-02-CLOSEOUT-01 used for the equivalent PR #805 entry.
- `docs/engineering/ai-delivery/tasks/BE-03-CLOSEOUT-01.md` sections 1, 2, 3, 9,
  10 and Annex A — this specification's own approved record of the pre-correction
  state it exists to correct, including every quotation of the stale wording.
  These are the approved specification's own content and its acceptance
  criteria; rewriting them would rewrite the approved specification and
  invalidate its own tests. Only `Status` and section 18 were changed.

#### `CURRENT AND CORRECT - PRESERVE`

- `docs/publisher-services/decisions.md` section 3a authority-condition
  construction — `Decision state:` line, the two-part condition, the
  self-resolution rule, the rationale against a mutable `APPROVED` token, and
  the "decision candidate" phrasing belonging to it. Preserved verbatim; see
  section 5.2.
- `docs/publisher-services/decisions.md` section 3a tension, proposed
  resolution, transaction-boundary note, configuration-only scope list,
  BE-04-dependent list, consequences and boundary-of-decision paragraphs — the
  architecture itself. Untouched.
- `docs/publisher-services/decisions.md` operational invariants 3 and 7 —
  backfill creates no back-catalogue jobs; automatic job creation is initially
  inactive. Untouched.
- `docs/publisher-services/README.md` section 5 item 6 — protected package and
  effective-capability reads and the dedicated superuser package mutation
  remain BE-03 scope. Untouched.
- `docs/publisher-services/rollout-plan.md` dependency graph and the structural
  statement "`BE-03` depends on both `BE-01` and `BE-02`". Untouched.
- `docs/publisher-services/rollout-plan.md` control "`thoth-app` must not begin
  `APP-01` implementation until `BE-03` exposes the approved protected API" — a
  durable rule whose condition is now satisfied; its wording stays truthful.
  Untouched.
- `docs/publisher-services/rollout-plan.md` Stage 1-5 deliverables, controls,
  exit evidence and rollback — rules and plan, not status claims. Untouched.
- `docs/publisher-services/adr-01-evidence-matrix.md` line 192 — the unpinned
  `codegen.ts` observation confirming the reserved BE-03/APP-01 contract
  control's concern; still true. Untouched.
- `docs/publisher-services/platform-inventory.md` — jobs are BE-04 scope and
  automatic job creation is inactive; no BE-03 reference at this base.
  Untouched.
- `docs/engineering/repository-map/control-gaps.md` CG-13 activation block,
  including `BE-02 runtime: NOT AUTHORIZED`; CG-13 remains `OPEN`. No BE-03
  reference at this base. Untouched.
- `docs/engineering/ai-delivery/tasks/BE-03.md` section 24 approval boundary —
  a statement about what specification approval does not authorize, still true.
  Untouched.
- `docs/engineering/decisions/ADR-0001-publisher-package-capability-model.md`
  line 334 — an architectural affected-task reference, not a lifecycle claim.
  Untouched.

#### `OUT OF SCOPE - PRESERVE`

- `docs/publisher-services/acceptance-matrix.md` lines 14-20 — maps requirements
  to owning tasks, evidence and activation gate; carries no BE-03 lifecycle
  status claim. Untouched, as the specification expected.
- `docs/publisher-services/master-issue.md` — no BE-03 statement. Untouched.
- `docs/engineering/ai-delivery/tasks/BE-01.md` (9 references),
  `BE-02.md` (5), `ADR-01.md` (11) — other tasks' approved specifications. Their
  BE-03 references are structural task-decomposition and scope-boundary
  statements ("BE-03 later owns the protected mutation", "`BE-03` depends on
  both `BE-01` and `BE-02`", "the dedicated protected read and superuser
  mutation belong to BE-03") rather than BE-03 lifecycle status claims.
  Amending another task's approved specification is outside this task's scope
  and is forbidden by its non-goal 9. Untouched.
- `docs/engineering/ai-delivery/tasks/THOTH-GQL-BATCH-01.md` line 1636 — the
  acceptance criterion asserting that `BE-02` remains unimplemented with no
  `DistributionPlatform` enum. A different programme's `DRAFT`,
  `NOT AUTHORIZED` specification, and a scope-containment test on that task's
  own future diff. Its premise is outdated; correcting it belongs to that task
  and programme. Untouched, per the explicit instruction not to repair it here.
- `docs/publisher-services/README.md` section 5 item 7 — "no
  `DistributionPlatform` enum exists in code". Materially false, but a BE-02
  assertion, not a BE-03 one. See section 13. Untouched.
- issue [#765](https://github.com/thoth-pub/thoth/issues/765), issue
  [#766](https://github.com/thoth-pub/thoth/issues/766) and PR
  [#799](https://github.com/thoth-pub/thoth/pull/799) — explicitly untouched.

### 10.2 Migration-path classification

Current repository migration names after PR #811, used by any new prose in this
change:

```text
BE-02 current:               thoth-api/migrations/20260811_v1.7.0/
BE-03 current:               thoth-api/migrations/20260812_v1.7.0/
v1.6.3 chapter hotfix:       thoth-api/migrations/20260813_v1.6.3/
```

Historical implementation-head names, preserved wherever they appear:

```text
BE-02 implementation head:   20260812_v1.7.0
BE-03 implementation head:   20260813_v1.7.0
```

Classification: every old-path reference in this repository lives in the BE-02,
BE-02-CLOSEOUT-01, BE-03, BE-03-SPEC and BE-03-CLOSEOUT-01-SPEC implementation
reports. Each is `HISTORICAL RECORD - PRESERVE`: correct at the exact head it
describes. None was rewritten. The BE-03 implementation report's rollout note
"Migration sequence: `20260813_v1.7.0` applies after `20260812_v1.7.0`" is
preserved: it was true as written and the ordering it asserts still holds under
the renamed directories.

**Active path-related control debt: none found.** No active Publisher Services
or shared engineering-control document makes a present-tense operational
assertion about the BE-03 migration path. The finding matches Annex A's. No
migration was renamed or modified, no migration ordering was changed, and PR
#811 was not reviewed, reverted or repaired.

This closeout's own new prose does not name a migration directory at all, so the
"use `thoth-api/migrations/20260812_v1.7.0/`" rule had no occasion to apply
outside this report.

### 10.3 Historical implementation reports untouched

Confirmed by the path containment result in section 9: no path under
`docs/engineering/ai-delivery/implementation-reports/` appears in the diff other
than this new report. The BE-02, BE-02-CLOSEOUT-01, BE-03, BE-03-SPEC and
BE-03-CLOSEOUT-01-SPEC implementation reports and all of their migration-path
references are byte-identical to the base.

### 10.4 Post-edit classified re-run

Command:

```text
git grep -n 'BE-03' -- docs/ CHANGELOG.md
git grep -n -iE 'BE-03[^.]{0,80}(not merged|unmerged|NOT AUTHORIZED|awaiting|in review|must not exist|draft)' \
  -- docs/publisher-services/ docs/engineering/ai-delivery/tasks/ docs/engineering/repository-map/ docs/engineering/decisions/
```

Result: every surviving hit of the stale-language pattern is inside
`docs/engineering/ai-delivery/tasks/BE-03-CLOSEOUT-01.md`, in that
specification's own background, scope, acceptance criteria, manual-verification
commands and Annex A — that is, its approved record of the state it exists to
correct. No hit remains in `task-status.md`, `README.md`, `rollout-plan.md`,
`decisions.md`, `BE-03.md`, `control-gaps.md` or any other active control
document.

Observed result against each required assertion:

- no active control says BE-03 is unmerged — confirmed;
- no active control says the BE-03 implementation is awaiting authorization,
  review or merge — confirmed;
- no active control says BE-03 is an unsatisfied dependency where the merge
  satisfied it — confirmed;
- BE-04 remains `BLOCKED` / `NOT STARTED`, unspecified and unauthorized, with no
  durable job, target, attempt, automatic onboarding or dissemination
  represented as existing — confirmed
  (`git grep -n 'BE-04\|distribution_job' -- docs/publisher-services/`);
- MIG-01 remains `CRITICAL` / `BLOCKED` with its audit, backfill, dry-run and
  CG-13 prerequisites — confirmed;
- APP-01 remains `BLOCKED` by BR-APP-01 or an explicit CTO exception, the CG-11
  CI closure work, exact-SHA contract pinning and its own approved
  specification — confirmed;
- APP-01 job-aware state remains BE-04-dependent — confirmed in `task-status.md`,
  `README.md` and `decisions.md`;
- APP-02 remains blocked on BE-04 and APP-01 and is not represented as ready —
  confirmed;
- production, environment, migration-execution, backfill, job-creation,
  dissemination, activation, `OBSERVE`/`ENFORCE`, workflow and production-access
  actions remain `NOT AUTHORIZED` — confirmed
  (`git grep -n 'NOT AUTHORIZED' -- docs/publisher-services/ docs/engineering/repository-map/control-gaps.md`);
- `decisions.md` authority-condition construction intact — confirmed by
  inspecting the two-hunk diff for that file;
- historical implementation reports and their migration paths untouched —
  confirmed;
- PR #799 untouched and not represented as a Publisher Services dependency —
  confirmed (`git grep -n '799' -- docs/`: every hit is a pre-existing non-goal
  or exclusion statement, none changed by this diff).

### 10.5 Durability re-read

Each corrected paragraph was re-read against the ADR-0005 section 6 test. No
corrected sentence depends on this pull request's own review, authorization or
merge state, so merging this closeout does not falsify any sentence it writes
and does not create the need for a further closeout.

### 10.6 Changelog

One entry added as the first item under the existing `## [Unreleased]` /
`### Added` headings. Verified: exactly one `## [Unreleased]` heading and
exactly one `### Added` heading in that section; no heading created or
duplicated.

Evidence link: the pull-request diff and check record.

## 11. CI

CI status: PENDING at the time of writing. GitHub is the live authority; the
result is the pull request's check record. The change is expected to classify as
documentation-only. Actual results will be reported rather than assumed.

## 12. Rollout and rollback

Initial state after merge: repository documentation only; no runtime,
deployment, migration, backfill, activation or production effect.
Activation required: none; this task activates nothing.
Feature flag/configuration: none.
Migration sequence: not applicable; no migration is added, renamed or modified.
Rollback/disable procedure: ordinary revert of the documentation pull request
under normal review.
Monitoring required: none.

## 13. Known limitations and deferred work

- **Residual BE-02 control debt, deliberately not fixed here.**
  `docs/publisher-services/README.md` section 5 item 7 states "It is a decision
  record, not an implemented enum; no `DistributionPlatform` enum exists in
  code." This is materially false — BE-02 merged the 17-value enum through PR
  #805 — but it is a BE-02 assertion, not a BE-03 one. BE-02-CLOSEOUT-01's
  acceptance criterion targeted exactly this class of statement and corrected
  the `platform-inventory.md` and `control-gaps.md` instances; this README
  instance survived. Recorded as deferred control debt for its own bounded task.
  Recommended owner: CTO, as a bounded BE-02 residual-debt correction.
- **Different-programme stale premise, deliberately not fixed here.**
  `docs/engineering/ai-delivery/tasks/THOTH-GQL-BATCH-01.md` line 1636 carries an
  acceptance criterion premised on BE-02 being unimplemented. It belongs to a
  different programme's `DRAFT`, `NOT AUTHORIZED` specification. Recommended
  owner: that task and programme.
- **Adjacent observation, not actioned.** PR #811 placed the v1.6.3 hotfix
  migration at `thoth-api/migrations/20260813_v1.6.3/`, which sorts after the
  v1.7.0 publisher-services migrations at `20260811_v1.7.0` and
  `20260812_v1.7.0`. That is PR #811's business, is out of scope here, and does
  not affect the BE-02-then-BE-03 apply order. Recorded for the CTO only.
- **`BE-02.md` header.** The BE-02 specification header still reads
  "Implementation authorization: separate and absent" although BE-02 has merged.
  It is a BE-02 record, outside this task's scope, and is noted only so the
  observation is not lost.

## 14. Unresolved issues

NONE.

No repository authority conflict was found about what BE-03's merge satisfies.
The one wording collision inside the approved specification — Annex A's
"section 3a in its entirety" versus the normative sections' identification of
the APP-01 dependency sentence as stale — was resolved by explicit CTO control
ruling and is recorded in full in section 5.2.

## 15. Agent self-assessment

This agent implemented the change and does not approve it. The pull request
remains draft pending fresh independent exact-head review and separate CTO merge
authorization.

Confirmations:

- no runtime, schema, migration, GraphQL, generated-contract, client-artifact,
  Cargo, workflow, deployment or environment change occurred (section 9 negative
  path proof);
- issues #765 and #766 and PR #799 were untouched; no GitHub issue or pull
  request other than this task's own draft PR was created or modified;
- BE-04, MIG-01, APP-01 and APP-02 were not started, specified or authorized,
  and each retains its recorded blockers;
- no production or environment action occurred: no deployment, no migration
  execution, no backfill, no assignment or job creation, no dissemination, no
  activation, no `OBSERVE`/`ENFORCE` transition, no workflow dispatch and no
  production access or credential use;
- no review, approval or merge-authorization identifier, merge commit SHA or
  merge timestamp was newly transcribed into any repository file.

Suggested review focus:

1. **The `decisions.md` section 3a ruling application** (section 5.2). Confirm
   the self-resolving authority-condition construction is intact and that the
   single corrected sentence changes dependency state only, not architecture.
2. **Durability of every corrected sentence.** Confirm no sentence written here
   becomes false when this pull request merges.
3. **The two fresh findings in section 5.1** that Annex A did not record, and
   whether the classified search missed any comparable future-tense assertion.
4. **Downstream containment.** Confirm that satisfying the BE-03 dependency did
   not make BE-04, MIG-01, APP-01 or APP-02 read as ready anywhere.
5. **`BE-03.md` blast radius.** Confirm only the three authorized
   lifecycle-boundary sites changed and that no requirement, invariant,
   acceptance criterion, test obligation or BE-04 transaction seam moved.
