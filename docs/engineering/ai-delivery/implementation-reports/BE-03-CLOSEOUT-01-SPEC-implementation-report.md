# BE-03-CLOSEOUT-01-SPEC Implementation Report

## 1. Repository state

Repository: `thoth-pub/thoth`
Workflow: STANDARD
Base branch: `develop`
Base commit: `a4585a8d89166577da5ce6f46ce51ddb134b3f7e`
PR target: `develop`
Programme integration branch: None
Task branch: `feature/publisher-services/be-03-closeout-spec`
Head commit: recorded in the pull request; this report is written at the branch
head that carries it
Pull request: draft pull request against `develop`. Live pull-request state,
review state and CI evidence are represented by the GitHub pull-request record.
This committed report does not duplicate transient PR lifecycle state
(ADR-0005).
Expected branch deletion after merge: YES
Final programme PR required: NO
Implementing model: Claude Opus 5
Reasoning level: Extra High / xhigh

Preflight, performed before any edit or branch creation:

```text
origin/develop                     = a4585a8d89166577da5ce6f46ce51ddb134b3f7e
PR #811 state                      = MERGED (base develop)
PR #811 merge commit               = a4585a8d89166577da5ce6f46ce51ddb134b3f7e
a4585a8d first parent              = 3ba4452c316399d80cd8d85e7d5e1bd05e252664
PR #809 state                      = MERGED (base develop)
PR #809 merge commit               = 3ba4452c316399d80cd8d85e7d5e1bd05e252664
PR #809 head (second parent)       = c678bdcec33c2aa01be1f887a85ff851dfe35891
Competing BE-03 closeout branch    = none (local or origin)
Competing BE-03 closeout PR        = none (open PRs: 799, 752, 744, 742, 668)
Competing BE-03 closeout task file = none
Root AGENTS.md at a4585a8d         = identical to the read copy
docs/engineering/AGENTS.md         = identical to the read copy
Working tree                       = clean
```

All eight checks passed. The branch was created only afterwards, directly from
`a4585a8d89166577da5ce6f46ce51ddb134b3f7e`.

### 1.1 Prior blocked attempt

An earlier authorization for this task named
`3ba4452c316399d80cd8d85e7d5e1bd05e252664` as the exact expected `develop` head.
At preflight, `origin/develop` had advanced to
`a4585a8d89166577da5ce6f46ce51ddb134b3f7e` through the merge of
forward-integration PR [#811](https://github.com/thoth-pub/thoth/pull/811). That
attempt stopped `BLOCKED` without creating a branch or editing any file, and did
not adopt the newer head silently. The CTO then reviewed PR #811, re-authorized
specification authoring from `a4585a8d`, and issued the migration-rename ruling
recorded in section 5, decision 4.

## 2. Scope confirmation

Approved specification: this is a specification-authoring task. Its authority is
the CTO instruction re-authorizing bounded specification authoring from base
`a4585a8d89166577da5ce6f46ce51ddb134b3f7e`.

Implemented objective: author the implementation-complete specification for
`BE-03-CLOSEOUT-01`, the bounded post-merge documentation and control correction
required by ADR-0005 section 8 because active committed programme state
materially misdescribes BE-03 after its merge.

Out-of-scope changes made: NONE. No BE-03 closeout correction was performed in
`task-status.md`, `README.md`, `decisions.md`, `rollout-plan.md`,
`platform-inventory.md`, `control-gaps.md` or any other active control document.

## 3. Commits

- one commit adds the `BE-03-CLOSEOUT-01` specification, this implementation
  report and the `CHANGELOG.md` entry.

Exact SHAs are recorded in the pull request.

## 4. Files changed

- `docs/engineering/ai-delivery/tasks/BE-03-CLOSEOUT-01.md` (new)
  - reason: root [`AGENTS.md`](../../../../AGENTS.md) section 1 and
    [`docs/engineering/AGENTS.md`](../../AGENTS.md) section 3 require an
    approved written specification containing the listed fields before a task
    may be implemented.
  - behavioural effect: none.
- `docs/engineering/ai-delivery/implementation-reports/BE-03-CLOSEOUT-01-SPEC-implementation-report.md`
  (new)
  - reason: root `AGENTS.md` section 14 requires an implementation report before
    review.
  - behavioural effect: none.
- `CHANGELOG.md`
  - reason: root `AGENTS.md` section 13 requires an `## [Unreleased]` entry per
    pull request.
  - behavioural effect: none. Added under the existing `### Added` heading; no
    duplicate heading created.

No additional specification-registration file is mandatory. The task-index
requirement is satisfied by the file's presence in
`docs/engineering/ai-delivery/tasks/`, which is how `BE-02-CLOSEOUT-01`,
`ADR-01-CLOSEOUT-01` and `P0-01-CLOSEOUT` are registered; none of them has a
separate registration entry. Scope was therefore not broadened.

## 5. Implementation decisions

1. **The future task is a state correction, not a transcription.** ADR-0005
   section 4.1 item 6 and section 8 are explicit: no commit may exist merely to
   record that PR #809 merged, its merge SHA, its merged timestamp, an
   independent review identifier, a CTO approval identifier or a
   merge-authorization identifier. The specification's section 4 non-goal 2 and
   its acceptance criteria forbid exactly that, and the legitimate trigger is
   stated as materially false programme and dependency state. The specification
   deliberately references PR #809 and PR #811 as links rather than restating
   their lifecycle facts.

2. **The BE-02-CLOSEOUT-01 precedent is followed, not reinvented.** The
   specification mirrors that document's structure, its four-way classification
   vocabulary, its non-goals, its documentation-only evidence set and its
   "correct the dependency cell without changing the blocked status" rule. Two
   deliberate departures are recorded in decisions 5 and 6.

3. **Downstream dependency transitions are specified individually, not
   uniformly.** BE-04, MIG-01, APP-01 and APP-02 each have a different residual
   gate, so the specification states each one explicitly rather than instructing
   the implementer to "mark BE-03 satisfied". In particular it requires that
   APP-01's satisfied BE-03 backend-contract dependency be scoped to APP-01's
   configuration-only surface, and that APP-01's job-aware elements remain
   BE-04-dependent; and that APP-02 not become ready, since it depends on BE-04
   and APP-01 as well as BE-03. No task's `BLOCKED` status changes.

4. **The PR #811 migration renames are classified as historical evidence, per
   the CTO ruling.** The renames moved BE-02's migration from `20260812_v1.7.0`
   to `20260811_v1.7.0` and BE-03's from `20260813_v1.7.0` to
   `20260812_v1.7.0`. Every repository reference to the pre-rename paths is
   inside an exact-head implementation or specification report, so each is
   `HISTORICAL RECORD - PRESERVE` and must not be rewritten. Where the closeout's
   own new prose must name the current migration it uses
   `thoth-api/migrations/20260812_v1.7.0/`, and the specification requires the
   implementation-head path and the current path to be distinguished rather than
   blurred. I verified that the rename preserved both the BE-03 migration's
   content (same enum, table and index) and the BE-02-then-BE-03 apply order, so
   nothing about architecture, migration ordering, release safety or BE-04
   planning is affected. No stop condition was triggered.

5. **`BE-03.md` section 23 and its header are classified as active stale state,
   and their correction is specified.** This departs from BE-02-CLOSEOUT-01,
   which did not touch `BE-02.md`. The reasoning: `BE-03.md` section 23 states
   "BE-03 implementation status is `NOT AUTHORIZED`. The branch
   `feature/publisher-services/be-03` must not exist until separate explicit CTO
   authorization", and the header repeats it as "**separate and absent**". These
   are present-tense operative instructions, not implementation-time evidence,
   and a future agent reading `BE-03.md` alone would conclude that BE-03 is
   unimplemented and that its branch is prohibited. The `Status: DRAFT` line is
   separately contradicted by `task-status.md`, which records the specification
   as repository-authoritative through PR #808, and by `BE-03.md`'s own authority
   condition. The specification confines the correction to that lifecycle-boundary
   prose and forbids altering any approved requirement, acceptance criterion,
   test obligation or architectural statement. This is flagged for reviewer
   attention: the conservative alternative is to leave `BE-03.md` untouched and
   accept the contradiction, and a reviewer may reasonably prefer it.

6. **The `decisions.md` section 3a authority-condition construction is preserved
   in full, including its "candidate" wording.** Both halves of its authority
   condition hold: the exact CTO-approved `BE-03-SPEC` content merged through
   PR #808 as `3b6b3a31`, which is an ancestor of `develop`. The section states
   its own resolution rule — after both halves hold it is an approved programme
   decision "without requiring a separate lifecycle-status edit to this file" —
   and explains, correctly, that writing a literal `APPROVED` token would require
   a further commit that moves the head the approval was bound to, producing
   exactly the approval-state churn ADR-0005 section 4.1 item 10 prohibits. The
   construction is therefore materially true and already establishes the approved
   decision. It is preserved as `CURRENT AND CORRECT`, and invariant 6 of the
   specification protects it. No substantive contradiction was found in it, so no
   architectural amendment is proposed and no stop condition was triggered.

   The references **outside** that construction are treated differently, because
   they do not carry it. `task-status.md` says in next-action 10 that both halves
   hold and the decision is approved, while its APP-01 row and next-action 11
   still say "the candidate phase boundary". That is an internal contradiction
   within one active tracker, and the specification requires it resolved in
   favour of the approved reading.

7. **One materially false statement was deliberately left out of scope.**
   `docs/publisher-services/README.md` section 5 item 7 still asserts "no
   `DistributionPlatform` enum exists in code", which BE-02's merge falsified.
   `BE-02-CLOSEOUT-01` carried the acceptance criterion "no active control
   document asserts that no `DistributionPlatform` enum is implemented" and
   corrected the `platform-inventory.md` and `control-gaps.md` instances; this
   README instance survived. It is residual **BE-02** control debt, not BE-03
   state. Correcting it is not necessary to make the BE-03 closeout internally
   coherent, and folding it in would silently broaden the task. It is recorded in
   Annex A of the specification as `OUT OF SCOPE - PRESERVE` with the debt named
   explicitly, and it is raised again in section 13 below for a separate bounded
   task.

8. **Durable status prose in the new documents themselves.** `BE-03.md`'s
   "Implementation authorization: **separate and absent**" formulation is
   precisely the construction that went stale and that this closeout must now
   repair. The new specification therefore writes "Implementation authorization:
   **separate.**" followed by a durable rule addressed to the implementing agent,
   rather than an assertion about the world that merging falsifies. The same
   applies to its `Status: DRAFT` line, which is the durable CTO-owned decision
   vocabulary retained by ADR-0005 section 6 and `docs/engineering/AGENTS.md`
   section 1, not transient pull-request state.

9. **Validation is the documentation-only contract, as the repository defines
   it.** Root `AGENTS.md` section 8 prescribes `git diff --check` plus link,
   terminology, duplicate-source, changelog and CI verification for a
   documentation-only change, and reserves the full workspace gate for
   Rust/domain changes. The specification requires that set plus the six
   task-specific proofs the authorizing instruction demands, and does not require
   `cargo test --workspace`. No file under any workspace member is modified by
   either this task or the specified future one, so the workspace gate has no
   changed input.

10. **The annexed findings are marked informative.** The specification requires
    the implementing agent to repeat the classified search from its own freshly
    authorized base. Annex A is a cross-check, explicitly not a checklist to
    apply mechanically, because `develop` has already moved once during this
    task's own life.

Deviations from the authorizing instruction: NONE.

## 6. Database and migration effects

Migration added: NO

No migration, schema, catalog or data change of any kind.
`thoth-api/src/schema.rs` and `thoth-api/migrations/` are untouched. The PR #811
migration directory names and their apply order are untouched.

## 7. API and compatibility effects

GraphQL/API changes: NONE
Generated schema/client updates: NONE. `thoth-client/assets/schema.graphql` is
untouched and no client build was required.
Backwards compatibility: unaffected.
Deprecations: NONE
Cross-repository dependencies: NONE. The reserved BE-03/APP-01 exact-SHA schema
pinning control is preserved unchanged and continues to bind against the BE-03
implementation head.

## 8. Authorization and security

Authorization paths changed: NONE
Roles/scopes involved: none
Negative authorization tests: not applicable; no authorization code exists in
this diff
Secret or personal-data handling: none. No credential, token, endpoint, bucket
or account identity appears in the diff.
Security limitations: none introduced.

## 9. Tests and checks

### Formatting

Command:

```text
git diff --check
```

Result:

```text
no output; exit status 0
```

### Unit tests

Not run, and not required. This change is documentation-only; root `AGENTS.md`
section 8 reserves the full workspace gate for Rust/domain changes and no file
under any workspace member is modified.

### Integration/database tests

Not applicable. No migration, schema or database-backed code is touched.

### Lint/static analysis

Not applicable to a Markdown-only diff. No Rust source file is modified, so
`cargo clippy` and `cargo fmt` have no changed input.

### Other required checks

Path containment:

```text
git diff --name-only a4585a8d89166577da5ce6f46ce51ddb134b3f7e..HEAD
```

Result:

```text
CHANGELOG.md
docs/engineering/ai-delivery/implementation-reports/BE-03-CLOSEOUT-01-SPEC-implementation-report.md
docs/engineering/ai-delivery/tasks/BE-03-CLOSEOUT-01.md
```

Negative path proof:

```text
git diff --name-only a4585a8d89166577da5ce6f46ce51ddb134b3f7e..HEAD \
  | grep -E '^(thoth-api|thoth-api-server|thoth-client|thoth-errors|thoth-export-server|\.github|Cargo\.)'
```

Result: no output; exit status 1 (no match). No runtime, migration, schema,
GraphQL, generated client, workflow or Cargo path appears.

Relative link resolution: every relative link introduced by this change was
resolved against the filesystem and exists.

Changelog: one entry added under the existing `## [Unreleased]` / `### Added`
heading; no duplicate heading created.

## 10. Manual verification

Environment: local checkout at base
`a4585a8d89166577da5ce6f46ce51ddb134b3f7e`.

Steps: every BE-03 statement in the active Publisher Services and shared
engineering-control surface was located, read in context and classified. The
search was `git grep -ln 'BE-03'` across the repository, followed by per-file
line-level reading; no global find/replace was used and nothing was corrected.

Observed result - classified stale-state findings: recorded in full in
[Annex A of the specification](../tasks/BE-03-CLOSEOUT-01.md). Summary counts:

```text
ACTIVE STALE STATE - CORRECT      17 statements across 5 files
HISTORICAL RECORD - PRESERVE      3 implementation reports (incl. all
                                  migration-path references) + the BE-01/BE-02/
                                  ADR-01/THOTH-GQL-* task records + CHANGELOG
CURRENT AND CORRECT - PRESERVE    10 statements, incl. decisions.md section 3a
                                  in full and the CG-13 activation block
OUT OF SCOPE - PRESERVE           6 items, incl. one materially false residual
                                  BE-02 assertion (decision 7) and one
                                  cross-programme DRAFT acceptance criterion
```

Explicit classification of the PR #811 migration-path references, as required:

```text
BE-03-implementation-report.md   lines 41, 87, 94, 276, 289, 290, 1327
  -> HISTORICAL RECORD - PRESERVE  (20260813_v1.7.0, correct at PR #809 head)
BE-03-implementation-report.md   line 280
  -> HISTORICAL RECORD - PRESERVE  (20260812_v1.7.0 as BE-02's migration)
BE-03-SPEC-implementation-report.md lines 1144, 1312
  -> HISTORICAL RECORD - PRESERVE  (20260812_v1.7.0 as BE-02's migration)
BE-02-implementation-report.md   lines 69, 70, 91, 94, 162
  -> HISTORICAL RECORD - PRESERVE  (20260812_v1.7.0, correct at PR #805 head)
```

Active path-related control debt: **none found**. No active control document
makes a present-tense operational assertion that the current BE-03 migration
path is `20260813_v1.7.0`. Every such reference is inside a historical report.

Supporting verification of the rename's effect, performed to confirm that no
stop condition applies:

```text
BE-03 migration content at 3ba4452c, dir 20260813_v1.7.0:
  CREATE TYPE publisher_service_configuration_source
  CREATE TABLE publisher_service_configuration_history
  CREATE INDEX publisher_service_configuration_history_publisher_created_idx
same three objects at a4585a8d, dir 20260812_v1.7.0  -> content preserved
BE-02 migration relocated 20260812_v1.7.0 -> 20260811_v1.7.0 -> order preserved
```

Evidence link: PR [#809](https://github.com/thoth-pub/thoth/pull/809) and PR
[#811](https://github.com/thoth-pub/thoth/pull/811) are the GitHub lifecycle
evidence. Neither is modified by this task.

## 11. CI

CI status: PENDING at the time of writing; the repository's documentation-only
gating is expected to classify this change as docs-only and skip the heavy jobs.
The live result is the GitHub pull-request check record.
Checks: `check-changelog` plus the `classify` step of the gated workflows.
Failures or warnings: none known.

## 12. Rollout and rollback

Initial state after merge: repository documentation only. No runtime,
deployment, migration, backfill, activation or production effect. The
`BE-03-CLOSEOUT-01` specification becomes repository-authoritative as to **what**
the closeout must do; it authorizes no implementation.
Activation required: none; this task activates nothing.
Feature flag/configuration: none.
Migration sequence: none.
Rollback/disable procedure: ordinary revert of this documentation pull request
under normal review.
Monitoring required: none.

## 13. Known limitations and deferred work

- **Residual BE-02 control debt.** `docs/publisher-services/README.md` section 5
  item 7 asserts "no `DistributionPlatform` enum exists in code", which is
  materially false since BE-02's merge. It is deliberately out of scope here
  (decision 7) and warrants its own bounded correction, most naturally folded
  into whichever Publisher Services documentation task next touches that file.
- **Cross-programme stale premise.**
  `docs/engineering/ai-delivery/tasks/THOTH-GQL-BATCH-01.md` line 1636 carries
  the acceptance criterion "`BE-02` remains unimplemented". That specification is
  `DRAFT` and `NOT AUTHORIZED`, and the criterion is a scope-containment test on
  its own future diff, so it is left to that task and programme. If BATCH-01 is
  ever authorized, that criterion must be revisited first.
- **`BE-03.md` correction is a judgement call.** Decision 5 departs from the
  BE-02-CLOSEOUT-01 precedent by specifying correction of the BE-03
  specification's own lifecycle-boundary prose. A reviewer may prefer the
  conservative alternative of leaving it untouched; that would leave `BE-03.md`
  asserting that its own implementation branch must not exist.
- **Annex A can go stale.** `develop` moved once during this task's own life.
  The specification requires the implementing agent to repeat the classified
  search from its freshly authorized base and treats Annex A as informative only.
- **The tracker's `Last updated` line remains a manually maintained field.**
- **Rollout-plan stage status is not asserted.** Whether `rollout-plan.md`
  Stage 2 is now partially delivered is a programme judgement for the CTO, not a
  stale-state correction, and the specification deliberately does not require it.

## 14. Unresolved issues

- NONE. The specification contains no unresolved mandatory `TBD`, no
  architectural guess and no unspecified downstream dependency transition.

## 15. Agent self-assessment

The agent may identify risks but may not approve the task. This specification is
not self-approved and the closeout implementation it describes remains
unauthorized.

Suggested review focus:

- confirm decision 5 — whether correcting `BE-03.md`'s `Status`, header
  implementation-authorization block and section 23 is in scope for a control
  correction, or whether the task specification should be left untouched as
  BE-02-CLOSEOUT-01 left `BE-02.md`;
- confirm decision 6 — that preserving `decisions.md` section 3a in full,
  including its `Decision state:` line and internal "candidate" phrasing, is the
  correct reading of the ADR-0005 authority-condition construction, and that
  correcting only the references outside it is the right boundary;
- confirm decision 7 — that the materially false residual BE-02
  `DistributionPlatform` assertion in `README.md` is correctly excluded rather
  than opportunistically repaired;
- confirm that the specification requires no new transcription of any review,
  approval or merge identifier, merge SHA or merge timestamp;
- confirm that the four downstream dependency transitions are each stated with
  their correct residual gates, and that none of them makes a task ready;
- confirm that the specified validation set matches the repository's actual
  documentation-only contract and does not over- or under-specify it;
- confirm that this specification-authoring diff is documentation-only and
  performs none of the closeout corrections itself.
