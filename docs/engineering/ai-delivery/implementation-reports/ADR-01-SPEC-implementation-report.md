# ADR-01-SPEC Implementation Report

## 1. Repository state

Repository: `thoth-pub/thoth`

Programme: Publisher Services and Distribution Configuration

Task ID: `ADR-01-SPEC`

Risk: MEDIUM

Workflow: STANDARD

Base branch: `develop`

Base commit: `37b802776ae6853affe19d90156f3c1e0654ebe3`

PR target: `develop`

Programme integration branch: None

Task branch: `feature/publisher-services/adr-01-spec`

Head commit: recorded in the immutable exact-head evidence comment on the pull
request.

Pull request: [#780](https://github.com/thoth-pub/thoth/pull/780)

Expected branch deletion after merge: YES

Final programme PR required: NO

Implementing model: Claude Opus 5

Reasoning level: High

Independent reviewer/model: independent exact-head reviews `4866458391`
(CHANGES REQUIRED at `de39ed3c`) and `4866683359` (APPROVED at `820f9cfa`)
were performed by a reviewer that did not author this specification.

Review reasoning level: MEDIUM or HIGH.

### 1.1 Base verification

`origin` was fetched and `origin/develop` was verified as
`37b802776ae6853affe19d90156f3c1e0654ebe3` immediately before branch creation
and again at the point of authorization. That SHA matches the SHA the
authorization expected, so no base movement occurred and no rebase or base
change was performed. The working tree was clean, the task branch existed
neither locally nor remotely, and no open or merged ADR-01-SPEC pull request
existed. The base was recorded before any edit.

### 1.2 Observed BE-01 state

Observed at branch creation, from GitHub and from Git ancestry:

- PR [#779](https://github.com/thoth-pub/thoth/pull/779) is `OPEN` and `DRAFT`;
- head `1dca1c1a0995b7e555ed7c0084f1a4e31c99b958`;
- `git merge-base --is-ancestor 1dca1c1a... origin/develop` returns non-zero:
  the head is **not** part of `develop`;
- `mergedAt` is null.

BE-01 had therefore not merged into this branch's verified base. No BE-01
post-merge state was copied into any committed record here. The BE-01 tracker
row and the tracker's `Next actions` list were deliberately left untouched, both
to respect that constraint and to avoid conflicting with PR #779, which rewrites
exactly those lines.

PR #779 coordination facts, requiring no change in this task: PR #779 is
currently open and draft. Its head is not part of the ADR-01-SPEC base. No
candidate BE-01 post-merge changes were copied into this branch. The BE-01
tracker row and related candidate changes were left untouched to avoid
cross-PR conflict. PR #779's committed candidate records may legitimately
describe the resulting authoritative state after merge, because that content
reaches `develop` only if PR #779 actually merges; this is the programme's
established control model for candidate committed content, and it is not a
defect. An earlier revision of this report characterized that wording as a
false merge claim requiring resolution in PR #779; that characterization was
incorrect and is withdrawn.

### 1.3 Approved design source

The approved private design was read from the configured project source, not
from recollection:

- Drive file ID `1kr2Ft0Y4pxgcXGyFAKs_wfFx4I0jlxEvaceswE5Dus8`;
- title `Publisher Services and Distribution Configuration - Technical Design
  and Implementation Plan`;
- `modifiedTime` `2026-07-23T20:32:36`, matching the reviewed revision `3`
  recorded in [`design-references.md`](../../design-references.md);
- source status `Approved for phased implementation`.

The source has not changed since the recorded revision. Section 8.2 of that
design defines the ADR-01 epic and is the primary authority for this
specification's objective, evidence scope and acceptance criteria. No content of
the private document is reproduced in the repository.

## 2. Scope confirmation

Task authorization: the CTO-authorized `ADR-01-SPEC` execution authorization
dated 2026-08-05. That authorization covered the drafting work only. Drafting
authorization is not approval of the resulting written specification. The
written specification subsequently received its own independent review and
explicit CTO approval, recorded in section 2.1.

Implemented objective: draft the bounded implementation specification for
`ADR-01 - Platform inventory and final architecture`, and reconcile the
Publisher Services dependency and rollout records with it, without performing
ADR-01. The specification becomes repository-authoritative on merge of
specification PR #780.

### 2.1 Recorded approval state

```text
Approved substantive content head:
820f9cfa22d284f8f347db338aa2461408f4ed12

Independent review:
4866683359 - APPROVED

CTO written-specification approval:
Javi, CTO
2026-08-05

Approval scope:
written ADR-01 specification only
```

- The approval-state commit that records this approval changes approval
  metadata and programme control state only. No substantive specification
  requirement, acceptance criterion, invariant, evidence rule, stop condition,
  platform candidate, dependency or non-goal changed after the independent
  review.
- Independent review `4866683359` applies to the substantive written content
  at `820f9cfa22d284f8f347db338aa2461408f4ed12`. The approval-state commit
  creates a new head, which requires its own fresh exact-head review and CI.
- Approval covers the written ADR-01 specification only. ADR-01 implementation
  remains unauthorized, `feature/publisher-services/adr-01` remains absent,
  and merge of PR #780 remains separately unauthorized.
- No platform decision, runtime work, workflow dispatch or production access
  is authorized by this approval.

Out-of-scope changes made: NONE.

One bounded correction was made beyond the ADR-01 records themselves, and is
declared in section 5.

## 3. Commits

- `76518ade` - `docs(publisher-services): specify ADR-01 platform inventory and
  architecture`
- `eea27901` - `docs(publisher-services): reconcile ADR-01 dependency and
  rollout records`
- `de39ed3c` - `docs(publisher-services): report ADR-01 specification` (this
  report, the changelog entries, and the recorded specification PR number)
- `820f9cfa` - `docs(publisher-services): correct ADR-01 specification gate
  and dependencies` (bounded documentation/evidence-only remediation of
  independent review `4866458391`: corrects the specification approval state to
  proposed/pending, corrects the PR #779 characterization, and replaces the
  linear dependency chain with an explicit dependency DAG)
- final commit - `docs(publisher-services): record CTO approval of ADR-01
  specification` (bounded approval-state commit recording the independent
  review `4866683359` and the explicit CTO approval of the written
  specification at content head `820f9cfa`; approval metadata and programme
  control state only, no substantive content change)

Exact SHAs for the final commit and head are recorded in the superseding
immutable exact-head evidence comment.

## 4. Files changed

- `docs/engineering/ai-delivery/tasks/ADR-01.md`
  - reason: create the ADR-01 implementation specification. Its written
    content was independently reviewed and explicitly CTO-approved (section
    2.1); it becomes repository-authoritative on merge of PR #780.
  - behavioural effect: none at runtime. It defines the read-only evidence scope
    across `thoth`, `thoth-dissemination` and `thoth-app`; the required
    per-destination record and its credential-recording rule; the evidence
    classification; the settled invariants ADR-01 may not reopen; the
    provisional and ambiguous destinations it must resolve; the decisions it
    must produce, including the `BE-02` descriptor contract and the
    dissemination mapping; its acceptance criteria; its required verification;
    its exact stop labels; its non-goals; the cross-repository coordination
    sequence; and the reserved `BE-03`/`APP-01` GraphQL contract control.

- `docs/publisher-services/task-status.md`
  - reason: record ADR-01's approved specification and sharpen the
    dependencies it governs.
  - behavioural effect: none. ADR-01 is `READY`: its written specification was
    independently reviewed and explicitly CTO-approved, with `READY` becoming
    repository-authoritative on merge of PR #780. The row states explicitly
    that `READY` does not authorize implementation, that the branch remains
    absent and unauthorized pending separate explicit authorization, that
    ADR-01 does not depend on BE-01, and that the final inventory remains
    provisional until the ADR-01 implementation is independently approved and
    merged. `BE-02` records that it requires the merged ADR-01 implementation
    rather than the specification alone. `BE-03` and `APP-01` record their own
    bounded-specification, app-readiness and schema-pinning dependencies. The
    BE-01 row is untouched.

- `docs/publisher-services/rollout-plan.md`
  - reason: record the coordinated cross-repository sequence and the reserved
    GraphQL contract control.
  - behavioural effect: none. Stage 0 records the independently reviewed and
    CTO-approved ADR-01 specification (repository-authoritative on merge of
    PR #780) and keeps the
    final inventory outstanding; new sections 2.1 and 2.2 record
    the dependency DAG, the parallel `thoth-app` readiness track and the
    schema-pinning control; Stage 2 records that ADR-01 must merge before
    `BE-02` finalizes `DistributionPlatform` and that no `OTHER` or fallback
    value exists; Stage 4 records exact-SHA pinning and the app readiness
    controls.

- `docs/publisher-services/README.md`
  - reason: record the independently reviewed and CTO-approved ADR-01
    specification, and correct a stale control claim.
  - behavioural effect: none. See section 5 for the declared correction.

- `docs/engineering/ai-delivery/implementation-reports/ADR-01-SPEC-implementation-report.md`
  - reason: required implementation report.
  - behavioural effect: none.

- `CHANGELOG.md`
  - reason: required changelog entry under `## [Unreleased]`.
  - behavioural effect: none. One `### Added` entry for the specification and
    one `### Changed` entry for the control reconciliation, both referencing
    PR #780. No duplicate headings were created.

`docs/publisher-services/platform-inventory.md` was deliberately **not**
changed. It remains the explicitly provisional baseline, which is the correct
state until an approved ADR-01 merges.

## 5. Implementation decisions

1. **ADR-01 is recorded as MEDIUM and not reclassified.** The specification
   states the risk rationale, states that MEDIUM is the current repository
   classification, and makes any proposed move to HIGH an explicit, justified
   control decision requiring CTO approval before merge. Downward
   reclassification to LOW is not available.

2. **ADR-01 is recorded as not depending on BE-01.** The two tasks are
   independent under the approved architecture: package selection and platform
   assignment are separate concerns. The BE-01 merge gate sits on the `BE-03`
   path. The specification and the rollout plan record the programme's hard
   dependencies as an explicit dependency DAG, distinguishing preferred
   delivery order from hard dependencies, so the sequence cannot be misread as
   a strict serial chain.

3. **The provisional inventory is left untouched.** Listing the provisional
   codes in the specification is explicitly recorded as *not* approval of those
   codes; each requires its own evidence and may be confirmed, renamed, split,
   merged or excluded. No ambiguous-destination question is resolved here.

4. **No task ID was invented for the CG-11 CI closure task.** A search of the
   repository documentation found `CG-11` referenced only as a control gap, with
   no task ID assigned. Both the specification and the rollout plan refer to it
   by description and state that the ID is assigned when that task is specified.

5. **Committed records state the truthful approval position.** Candidate
   committed content may describe the resulting authoritative state after
   merge, because it reaches `develop` only if the PR merges. The independent
   specification review (`4866683359`) and the explicit CTO approval of the
   written specification (Javi, CTO, 2026-08-05) have both occurred and are
   recorded in section 2.1; the committed tracker, rollout plan, README and
   this report therefore record the approved specification, with its
   repository-authoritative effect taking hold on merge of PR #780.
   Draft/open PR status lives in the PR body and the immutable evidence
   comments.

6. **The tracker's BE-01 row and `Next actions` list were left untouched**, both
   because PR #779 has not merged into this base and because PR #779 rewrites
   exactly those lines. The coordinated sequence was therefore recorded in the
   rollout plan, which is where the task authorization directs it, rather than
   duplicated into the tracker.

Deviation from the specification: **one declared bounded correction.**

- `docs/publisher-services/README.md` stated that "BE-01 IMPLEMENTATION BLOCKED
  ON SHARED DIESEL CONTROL" and that `THOTH-DB-CTRL-01` "must be independently
  approved and merged before BE-01 moves to `READY`". Both statements were
  already false on the verified base: `control-gaps.md` records CG-12 as
  `RESOLVED` by ADR-0003 Architecture A, `THOTH-DB-CTRL-01` as `SUPERSEDED`, and
  `task-status.md` records BE-01 as `READY`. This is a genuine
  repository-authoritative invariant correction within the task's allowance, and
  it corrects the README against `develop`'s own merged records — it does not
  import anything from the BE-01 branch. The correction deliberately makes no
  claim about BE-01's current delivery state, deferring that to the tracker row
  and the PR record, so that it stays true whether or not PR #779 merges first.

## 6. Database and migration effects

Migration added: NO.

No migration, PostgreSQL enum, `thoth-api/src/schema.rs` edit, model, table,
column, constraint or index was added or changed. No database of any kind was
contacted.

## 7. API and compatibility effects

GraphQL/API changes: NONE.

Generated schema/client updates: NONE. `thoth-client/build.rs` output is
unaffected, and `thoth-app/codegen.ts` is not changed.

Backwards compatibility: unaffected; no contract exists to break.

Deprecations: NONE.

Cross-repository dependencies: none created. The specification records the
future `BE-03`/`APP-01` schema-pinning control as reserved and not implemented,
and records that changing the app's code-generation schema source requires its
own approved task specification.

## 8. Authorization and security

Authorization paths changed: NONE.

Roles/scopes involved: none.

Negative authorization tests: not applicable; no authorization surface exists in
this change.

Secret or personal-data handling: no secret value, secret identifier, key or
token name, private environment content, private publisher list content or
sensitive object URL is recorded in any changed file. The specification
constrains future ADR-01 credential evidence to category and ownership only, and
explicitly permits recording the *structure* of configuration while prohibiting
its contents.

Security limitations: none introduced.

## 9. Tests and checks

This is a documentation-only change. The repository's documentation-only
evidence requirement applies.

### Whitespace and diff hygiene

Command:

```text
git diff --check
```

Result: clean; no output, no trailing-whitespace or conflict-marker findings.

### Changed-path classification

Command:

```text
git diff --name-only 37b802776ae6853affe19d90156f3c1e0654ebe3..HEAD
```

Result: every path is under `docs/` or is `CHANGELOG.md`. No runtime, schema,
migration, model, GraphQL, workflow, CI, `thoth-app` or `thoth-dissemination`
file is present.

### Link resolution

Every relative link introduced or changed was resolved against the repository
tree, including the cross-directory links from
`docs/engineering/ai-delivery/tasks/ADR-01.md` to the decisions, repository-map,
design-reference and Publisher Services documents, the `CG-07` anchor, and the
links added to the Publisher Services README and rollout plan.

### Content checks

- no unresolved placeholder remains; the specification PR number is recorded in
  the tracker rather than left as `TBD`;
- no secret-like value is present;
- no platform decision is presented as final, and
  `docs/publisher-services/platform-inventory.md` is unchanged;
- no BE-01 post-merge state is copied; the BE-01 tracker row and `Next actions`
  list are untouched;
- `CHANGELOG.md` uses existing `### Added` and `### Changed` headings under
  `## [Unreleased]` with no duplicates.

### Not run, and why

The Rust workspace gate (`cargo test`, `cargo check`, `cargo clippy`,
`cargo fmt`) was not run. No Rust source, manifest, migration or generated
contract is changed, so the gate has nothing to exercise; the repository's
documentation-only evidence rule applies instead. The exact-head CI record shows
whether the repository's own classifier reached the same conclusion.

## 10. Manual verification

Environment: local clone at the verified base; GitHub API for pull-request
state; the configured Drive source for the approved design.

Steps:

1. fetched `origin`, verified `origin/develop`, verified the clean working tree,
   and verified the absence of the task branch locally and remotely;
2. verified no open or merged ADR-01-SPEC pull request existed;
3. inspected PR #779's state, draft status, head and ancestry;
4. read the required control documents from `origin/develop` rather than the
   working tree, so no uncommitted or branch-local state influenced the
   specification;
5. read the approved design from the configured Drive source and confirmed its
   revision against the recorded one;
6. created the branch from the exact verified base and recorded it before any
   edit;
7. searched the repository for an authoritative CG-11 closure task ID and
   confirmed none exists;
8. confirmed `feature/oai-pmh-http` exists and was neither checked out,
   modified, rebased nor pushed.

Observed result: all preconditions held; the base did not move; the
specification and control reconciliation were produced within the authorized
documentation-only scope.

Evidence: recorded in this report and in the immutable exact-head evidence
comment on PR #780.

## 11. CI

CI status: recorded in the immutable exact-head evidence comment, which lists
workflow names, run IDs, conclusions, and whether the documentation-only
classifier correctly skipped the heavy Rust, migration and Docker jobs while
preserving the protected check contexts.

## 12. Rollout and rollback

Initial state after merge:

```text
ADR-01 specification: APPROVED AND MERGED
ADR-01 task: READY
ADR-01 implementation: NOT STARTED
ADR-01 implementation authorization: NOT GRANTED
ADR-01 implementation branch: ABSENT
```

- the final distribution-platform inventory remains provisional;
- `BE-02`, `BE-03`, `BE-04` and `APP-01` remain blocked under their recorded
  dependencies;
- no runtime, schema, API, app, dissemination or production behaviour changes.

Activation required: none. There is nothing to activate.

Feature flag/configuration: not applicable.

Migration sequence: not applicable.

Rollback: revert the documentation pull request. There is no runtime, data or
external effect to reverse.

Monitoring required: none.

## 13. Known limitations and deferred work

- The specification defines how the inventory will be determined; it determines
  no part of the inventory. Every provisional and ambiguous destination remains
  open by design.
- Source-owner evidence for OCLC arrangements, manually managed destinations,
  private publisher lists, credential ownership and update/withdrawal behaviour
  does not exist in any repository and must be obtained during ADR-01. Its
  absence is a stop condition there, not a gap here.
- The `BE-03`/`APP-01` GraphQL contract control is reserved and documented only.
  The app's current code generation still points at the shared test API; making
  that source pinnable requires its own approved task specification.
- No authoritative task ID exists for the CG-11 CI closure task. It is referred
  to by description until that task is specified.
- `CG-11`, `CG-13` and `BR-APP-01` are unchanged and remain open.

## 14. Unresolved issues

- Fresh independent exact-head review of the approval-state commit, explicit
  CTO merge authorization bound to the new exact head, and the merge of
  PR #780 itself are outstanding. The substantive written specification is
  independently reviewed and CTO-approved (section 2.1).

## 15. Agent self-assessment

This agent authored the specification and cannot approve it.

Suggested review focus:

1. **Stop conditions over completeness.** Confirm that the specification makes
   an incomplete inventory the correct outcome when evidence is missing, and
   that no wording lets an assumed value reach the enum.
2. **Evidence classification integrity.** Confirm that `provisional` and
   `unknown` cannot survive to approval, and that the `production-verified`
   class cannot be used to launder an unevidenced production claim given ADR-01
   performs no production access itself.
3. **Credential boundary.** Confirm the category/ownership rule cannot be
   satisfied by recording anything that aids secret retrieval, while still
   permitting the configuration-structure evidence `BE-02` and `MIG-01` need.
4. **Invariant preservation.** Confirm the sixteen settled invariants are stated
   correctly against ADR-0001, ADR-0002 and the approved design, and that
   reopening any of them is routed to a stop condition.
5. **The declared README correction** in section 5: confirm it is supported by
   `develop`'s own merged records, imports nothing from the BE-01 branch, and
   introduces no claim that would become false whichever way PR #779 resolves.
6. **Scope containment.** Confirm no changed file is outside documentation, that
   the provisional inventory is untouched, and that no ambiguous-destination
   question was answered here.
