# THOTH-ASYNC-01-ADR-01 Implementation Report

This report records the bounded ADR-0012 architecture task for
THOTH-ASYNC-01: the original architecture authoring, its independent-review
correction rounds, the historical approval-state reconciliation, the
programme-integration compatibility reconciliation, the route-disposition
clarification and the Route-B material activation-epoch correction. It records
durable task evidence only. Live head, CI, review, authorization and merge state
is GitHub-owned lifecycle evidence under `ADR-0005` and is not copied here.

Sections 1-15 follow the repository implementation-report template for the
Route-B correction. Appendix A preserves the earlier authoring, review,
approval-state, reconciliation and validation record. Appendix B records the
durable control conditions.

## 1. Repository state

Owning GitHub issue: [#958](https://github.com/thoth-pub/thoth/issues/958)
Parent programme: [#957](https://github.com/thoth-pub/thoth/issues/957) (THOTH-ASYNC-01)
Repository: `thoth-pub/thoth`
Risk: CRITICAL
Workflow: PROGRAMME_INTEGRATION, retaining the approved transitional exception
that keeps the pre-topology task branch `feature/async/adr-0012` instead of a
`feature/worker--<slice>` branch (base specification, #958 comment
`5942203087`, section 1). The task originally ran as STANDARD against `develop`.
Base branch: `feature/worker`
Authorized base commit: `b44303c214498baf76c9d9a0a7cab374377f9cbe`
Actual base commit: `b44303c214498baf76c9d9a0a7cab374377f9cbe`
PR target: `feature/worker`
Programme integration branch: `feature/worker`
Task branch: `feature/async/adr-0012`
Head commit: the candidate commit that contains this report; it is not
self-pinned here (section 3). Its publication as the PR #960 head is a
post-commit, GitHub-owned lifecycle fact under `ADR-0005`.
Pull request: [#960](https://github.com/thoth-pub/thoth/pull/960), targeting
`feature/worker`; its live state, head and checks are GitHub-owned lifecycle
evidence.
Expected branch deletion after merge: YES
Final programme PR required: YES - `feature/worker -> develop` is a separate
programme-level gate.
Implementing model: Route-B candidate stage - Claude (Opus 5.5), bounded
implementing agent. Earlier rounds are recorded in Appendix A.
Reasoning level: standard

Historical and current control identities:

```text
original authorized base:
develop @ 923545d5c9028bc04c40e38efeb7de674efed3fd

historical programme integration base:
feature/worker @ 345a7a04131e7c0539f7518c6ea457fa5cb6a462

historical feature/worker refresh after CTRL-ADR-AMEND-01 (#964):
345a7a04131e7c0539f7518c6ea457fa5cb6a462
-> 8ac771c55937f7a4fe4131b3c21c466f0b4d7abc

Route-B feature/worker refresh after CTRL-ADR-PR-FIRST-01 (#966 / PR #967):
8ac771c55937f7a4fe4131b3c21c466f0b4d7abc
-> b44303c214498baf76c9d9a0a7cab374377f9cbe

repository-authoritative Route-B doctrine:
docs/engineering/decisions/README.md
blob de7769a4ee6a50c79c1b57ae6993d7eb4fa673d7

original CTO architecture-content approval source:
631e28d1f05495f24ce88369d4557f1033f970b3
ADR-0012 blob 9eb21363aa2f60a86db1a2293d697545ec3809e9

historical approval-state source and stale earlier programme pin:
bd5223deee44b465dd860ff96b75d81ad41cbf2d
ADR-0012 blob cffee08b0c4326823f36387e6cec2abe49594273

current prior ADR-0012 blob (task head before this correction):
dec6665353804e42f22474954bf878308092b717

task head before this correction / candidate parent 1:
b4670b8b7cb6d61bc42fb8fd2c40b1d8acf70b20

refreshed feature/worker / candidate parent 2:
b44303c214498baf76c9d9a0a7cab374377f9cbe

unmodified Route-B merge-tree baseline:
e15607878e66d2ff12b819dcea13293a46674f13
```

Both `feature/worker` refreshes were separately authorized control actions
recorded on #958 before any correction source was prepared; the Route-B refresh
receipt and post-refresh geometry check are #958 comment `5993603036`. Neither
refresh is part of this task's source write budget.

## 2. Scope confirmation

Approved specification:

- base post-#964 reconciliation specification: #958 comment `5942203087`;
- Specification Amendment 1: #958 comment `5948725125`;
- Specification Amendment 2 / Route B: #958 comment `5958029093`;
- Specification Amendment 3: #958 comment `5958882383`, which ratifies
  Amendment 2 as the selected Route-B task authorization;
- fresh independent CRITICAL specification approval receipt: #958 comment
  `5992836778`;
- candidate-stage implementation authorization: #958 comment `5993634460`;
- replacement-candidate authorization: #958 comment `5998119577` (section 5).

Implemented objective: the Route-B **candidate stage** of a material
architectural correction of ADR-0012 before repository authority. The candidate
implements the CTO-selected append-only activation-epoch architecture (#958
comment `5935475916`) that resolves review finding `R-b467-01` - a single
activation/deactivation pair cannot represent repeated route activation without
rewriting historical eligibility or changing the route identity - and carries
`Status: PROPOSED` together with a `PROPOSED` decision-register row, one
`PROPOSED` changelog correction entry and this reconciled report.

Out-of-scope changes made: NONE

### 2.1 Classification, route and eligibility

```text
classification:
MATERIAL ARCHITECTURAL CORRECTION (base specification section 3)

selected approval route:
Route B - PR-first staged approval (Amendment 2 B2, ratified by Amendment 3)
```

Eligibility for the material pre-authority correction is keyed to the ADR
number, not to one pathname (Amendment 1 A1;
`docs/engineering/decisions/README.md`). At candidate validation time
`b44303c214498baf76c9d9a0a7cab374377f9cbe` was simultaneously `develop` and the
refreshed `feature/worker`, and `master` was
`4fa7eaa9ccb60d39c41ccd8feb257edf28c173ff`. The following literal commands were
run against both commits:

```text
git log --format=%h <ref> -- ':(glob)**/*ADR-0012*'
result for both refs: no commits

git log --format=%h -G'^# ADR-0012' <ref>
result for both refs: no commits

git log --format=%h -G'^\| `ADR-0012`' <ref> -- docs/engineering/decisions/decision-register.md
result for both refs: no commits

git grep -l ADR-0012 <ref>
result for b44303c214498baf76c9d9a0a7cab374377f9cbe:
docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-CTRL-01-implementation-report.md
docs/engineering/decisions/ADR-0013-programme-integration-decision-reliance.md
result for 4fa7eaa9ccb60d39c41ccd8feb257edf28c173ff: no files
```

No path matching `*ADR-0012*`, no `# ADR-0012` decision heading and no
decision-register ADR-0012 row exists anywhere in the current history of
`develop`, `master` or `feature/worker`. The only ADR-0012 decision path,
`docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md`,
and every decision-register blob carrying an ADR-0012 row are reachable only from
`feature/async/adr-0012`. The two `develop` files that mention the identifier are
ADR-0013 and the THOTH-ASYNC-01-CTRL-01 control report; neither makes ADR-0012
approved or repository-authoritative. No evidence was found of any approved
ADR-0012 version ever being reachable from `develop`. Git objects removed from
every reachable ref cannot be proven from current history; no evidence indicates
that such a removed repository-authoritative ADR-0012 version existed.

### 2.2 Stale controls

Under the repository material-correction rules, this correction makes stale, for
the corrected version:

- the CTO architecture-content approval bound to
  `631e28d1f05495f24ce88369d4557f1033f970b3` / ADR blob
  `9eb21363aa2f60a86db1a2293d697545ec3809e9`, and the approval-state record
  carried by ADR blob `cffee08b0c4326823f36387e6cec2abe49594273`;
- every independent review bound to an earlier source head, including the
  approval-state review of `bd5223deee44b465dd860ff96b75d81ad41cbf2d`;
- the earlier #957 `ADR-0013` programme-integration pin to
  `bd5223deee44b465dd860ff96b75d81ad41cbf2d` / ADR blob
  `cffee08b0c4326823f36387e6cec2abe49594273`;
- any merge authorization bound to an earlier PR #960 head;
- the earlier branch-local decision-register assertion that ADR-0012 is
  `APPROVED`.

Those records remain historical evidence for their exact versions. They are
not approval of the corrected version, which requires fresh CTO content
approval, a single approval-state commit, CTO exact-final-blob approval, fresh
independent exact-head review and separate merge authorization under Route B.

## 3. Commits

Route-B candidate commit (this stage):

- exactly one genuine two-parent merge commit on `feature/async/adr-0012` -
  `THOTH-ASYNC-01-ADR-01: propose Route-B activation-epoch correction`
  - parent 1: `b4670b8b7cb6d61bc42fb8fd2c40b1d8acf70b20`
  - parent 2: `b44303c214498baf76c9d9a0a7cab374377f9cbe`
  - tree: the unmodified merge-tree
    `e15607878e66d2ff12b819dcea13293a46674f13` overlaid only by the four
    correction paths in section 4

The candidate commit SHA and its tree are not embedded here because this report
is part of that tree. They are recorded in the post-publication handoff; once
the candidate is published, the PR #960 head and head tree are GitHub-owned
lifecycle evidence under `ADR-0005`. The authorized topology is a fast-forward
of the task branch from parent 1 to the candidate; no rebase, squash, amend,
force update or third parent is authorized.

Earlier task-branch history from `923545d5c9028bc04c40e38efeb7de674efed3fd`,
first-parent order, newest first:

- `b4670b8b7cb6d61bc42fb8fd2c40b1d8acf70b20` - THOTH-ASYNC-01-ADR-01: correct validation provenance
- `227057cae8a4c10f7f7795e3afe68271e56681b9` - THOTH-ASYNC-01-ADR-01: clarify route dispositions
- `7f7e62824eb40308688b46351ffea182e9e52942` - THOTH-ASYNC-01-ADR-01: reconcile programme integration base (two parents: `bd5223deee44b465dd860ff96b75d81ad41cbf2d`, `345a7a04131e7c0539f7518c6ea457fa5cb6a462`)
- `bd5223deee44b465dd860ff96b75d81ad41cbf2d` - THOTH-ASYNC-01: align approved ADR amendment rule
- `c43aa750ea4d971fe4f803cf761868876f0783bd` - THOTH-ASYNC-01: clean approval metadata formatting
- `42d79d4617a73696efd328943c1cc547e807d39d` - THOTH-ASYNC-01: reconcile ADR-0012 approval state
- `631e28d1f05495f24ce88369d4557f1033f970b3` - THOTH-ASYNC-01: bind trusted promotion to validated bytes
- `d25bb1ecd1665cc4e635b16dbd5fd09582a5889b` - THOTH-ASYNC-01: address round-4 architecture review
- `ced4cb19224d7acc1824863a85d83b2387617449` - THOTH-ASYNC-01: record round-4 architecture corrections
- `ca31d792aa3f5212a9715278464e9d5ed928dd0c` - THOTH-ASYNC-01: update round-4 proposal summary
- `88fbe6f88a864fceb713b7055acf77860c2d6e43` - THOTH-ASYNC-01: update round-4 ADR proposal record
- `359c3b7c33cc1a6794dca48b46818da039c1d627` - THOTH-ASYNC-01: address round-3 review findings
- `2d5eb1ae1e25bc222c4ca94a4f6dc5e81ecceba8` - THOTH-ASYNC-01: finalize round-3 correction report
- `6af54e45478f3ecb9422c395a4f7c070700de240` - THOTH-ASYNC-01: make route activation concurrency-safe
- `482e2df9b719a230488936079edaf334df02a3ee` - THOTH-ASYNC-01: tighten round-3 trust and ADR boundaries
- `907ea34f8dbcf88ea1c822492fe28aae3b5de993` - THOTH-ASYNC-01: record round-3 architecture corrections
- `3402308e8ed75953bf262dd436788cefecc94633` - THOTH-ASYNC-01: update round-3 proposal summary
- `5e9af9fee5a8f1b499ee74ba78091ed9f398fb8e` - THOTH-ASYNC-01: update ADR-0012 round-3 proposal record
- `28658c6672181a41c03e7b0d974ec4c0ebe41ac0` - THOTH-ASYNC-01: address round-2 architecture findings
- `9d329e063b2731b691c62a412555b470f2a9fe39` - THOTH-ASYNC-01: record ADR-0012 correction round
- `f06ba7ffdf351e8f0829a6eb0e55dd8256bccc2b` - THOTH-ASYNC-01: update ADR-0012 proposal summary
- `aa08cd38531f17ce98c1a44fdcfe0b1b258872e1` - THOTH-ASYNC-01: reconcile ADR-0012 proposal record
- `4808d42364a621fc5a86e47cea65d86354df8af8` - THOTH-ASYNC-01: address ADR-0012 review findings
- `a69921682cd02e060110304f8f9868d0c39da65e` - THOTH-ASYNC-01: add ADR-0012 authoring report
- `fb8c1d4e9373153d367e2336b6a4dc349f165fc2` - THOTH-ASYNC-01: document ADR-0012 proposal
- `30efbc63945c474d924fb233adf4ce4f4aaefb20` - THOTH-ASYNC-01: register ADR-0012 proposal
- `83f3ec29db091724b6d084385050bc7cb239d5a6` - THOTH-ASYNC-01: propose shared async architecture

## 4. Files changed

Authorized correction write paths (base specification section 5, unchanged by
Amendments 1-3):

- `CHANGELOG.md`
- `docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md`
- `docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md`
- `docs/engineering/decisions/decision-register.md`

Authorized cumulative PR footprint against refreshed `feature/worker`:

- `CHANGELOG.md`
- `docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md`
- `docs/engineering/decisions/ADR-0008-machine-roles-and-durable-job-primitives.md`
- `docs/engineering/decisions/ADR-0010-staff-operations-console.md`
- `docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md`
- `docs/engineering/decisions/decision-register.md`

Authorized new-file paths: NONE for this stage; this report already existed.

Actual files changed by the candidate relative to the unmodified merge-tree
baseline `e15607878e66d2ff12b819dcea13293a46674f13`:

- `docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md`
  - reason: implement the CTO-selected append-only activation-epoch
    architecture and record the corrected version as `PROPOSED`.
  - behavioural effect: architecture proposal only. The header carries
    `Status: PROPOSED` and no current approver or approval date. Section 3.2
    replaces the single activation/deactivation pair per route with a stable
    logical `route_key` and an append-only history of activation epochs
    (half-open `[activation_generation, deactivation_generation)` intervals,
    at most one open epoch, total order by activation generation, write-once
    close facts, serialized open/close/retirement, terminal permanent
    retirement, historical eligibility through any open or closed epoch, no
    retroactive gap coverage, subordinate epoch identifiers) and states the
    disposition model consistently with section 3.5 and invariant 8. The
    event-emission floor, route health signals (section 3.15), invariants 3-6
    and 8, implementation decomposition, rollout, rollback and validation
    requirements are aligned with the epoch model. Section 12 records the
    `PROPOSED` state, binds the 2026-09-30 approval to the historical
    pre-correction versions only, keeps the repository-authority condition and
    the architecture-versus-implementation separation, and aligns the ADR's
    amendment clause with `docs/engineering/decisions/README.md`. All unrelated
    architecture is unchanged.
  - within authorized write budget: YES
- `docs/engineering/decisions/decision-register.md`
  - reason: the material correction invalidates the earlier exact-version
    approval represented by the ADR-0012 row.
  - behavioural effect: only the ADR-0012 row changes. It records `PROPOSED`,
    binds the 2026-09-30 approval to the historical pre-correction versions,
    summarizes the activation-epoch correction and preserves the partial
    supersession boundaries, the authority condition, the ADR-0013
    programme-local reliance distinction and the implementation-authorization
    distinction. No other row or header line changes.
  - within authorized write budget: YES
- `CHANGELOG.md`
  - reason: required changelog entry.
  - behavioural effect: exactly one new `THOTH-ASYNC-01-ADR-01-CORRECTION`
    entry at the top of `[Unreleased] -> Changed`, describing the corrected
    version as `PROPOSED` and stating that the earlier ADR-0012 approval and
    clarification entries apply only to the historical pre-correction exact
    versions. No new heading; existing ADR-0012 and ADR-0013 entries are
    byte-preserved.
  - within authorized write budget: YES
- `docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md`
  - reason: reconcile the report to the current template and the Route-B
    candidate state while preserving material history.
  - behavioural effect: documentation only. Sections 1-15 follow the template;
    the earlier record is preserved in Appendix A with headings demoted and
    only the tense or framing changes needed to keep it truthful as history.
  - within authorized write budget: YES

Inherited unchanged from refreshed `develop` through parent 2 (the closed
inherited set between the task head and the merge-tree baseline):

- `CHANGELOG.md` - inherited merge contribution of two `[Unreleased] ->
  Changed` entries (`CTRL-ADR-PR-FIRST-01`, `CTRL-ADR-AMEND-01`), separate from
  the single new correction entry above;
- `docs/engineering/decisions/README.md` - blob
  `de7769a4ee6a50c79c1b57ae6993d7eb4fa673d7`;
- `docs/engineering/ai-delivery/implementation-reports/CTRL-ADR-AMEND-01-implementation-report.md`
  - blob `7bf7425f0527dbaa72e718df5264e399035a148b`;
- `docs/engineering/ai-delivery/implementation-reports/CTRL-ADR-PR-FIRST-01-implementation-report.md`
  - blob `d86cf215866750979eafea040426c51374d5155c`.

Byte-preserved task decision and control paths:

- ADR-0008 - blob `622060ad90eec41c792f110c373efffbb11a4b56`;
- ADR-0010 - blob `aca2142a3387785e80db908b3a5c1b0afd82ab51`;
- ADR-0013 - blob `d928f957bdf775d03e99fc73888ec8e2dec5f80b`.

Actual new files created: NONE

Files deleted, moved or renamed: NONE

### 4.1 Write-budget compliance

PASS

The candidate differs from the unmodified merge-tree baseline in exactly the
four authorized correction paths and from refreshed `feature/worker` in exactly
the six cumulative PR paths (section 9). No ADR-0008, ADR-0010, ADR-0013,
engineering-control doctrine, workflow, `AGENTS.md`, runtime, migration or
schema path is edited.

## 4.2 Authorized actions actually used

For the Route-B candidate stage authorized by #958 comments `5993634460` and
`5998119577`. This matrix states only what had been done when the commit that
contains this report was created.

Done up to and including the creation of this candidate commit:

- repository inspection: USED (Git history, refs, doctrine and the #957/#958/PR
  #960 records, read-only)
- source edit: USED (the four authorized correction paths only)
- new file creation: NOT USED
- file deletion/move/rename: NOT USED (not authorized)
- branch creation: NOT USED (no branch is created; a detached local worktree is
  used for construction, and the existing task branch is the authorized
  fast-forward target)
- commit: USED (the two-parent replacement candidate merge commit that contains
  this report; see section 5 for the earlier unpublished local candidate)
- PR creation/update: USED for one PR #960 state change only - Ready -> Draft,
  performed before any candidate was constructed and independently verified by
  control. Draft -> Ready is not authorized at this stage.

Authorized post-commit actions - GitHub-owned lifecycle evidence under
`ADR-0005`, not self-recorded here:

- the non-force push that fast-forwards `feature/async/adr-0012` from
  `b4670b8b7cb6d61bc42fb8fd2c40b1d8acf70b20` to this candidate;
- post-push verification of the remote head, tree and ordered parents;
- the truthful PR #960 description update for the published candidate;
- the automatic PR CI and any automatic review activity those actions cause.

These actions necessarily follow the creation of the commit that contains this
report. Their outcomes are recorded in the post-publication handoff and the
GitHub ledger rather than written into the candidate source tree as completed.

Not authorized at this stage, and separate later gates:

- candidate content approval, the approval-state commit, Draft -> Ready,
  final-head Codex/app review and the final independent review: later Route-B
  gates outside this stage; the approval-state commit and Draft -> Ready are not
  authorized here
- issue/comment mutation: NOT USED (not authorized)
- review-thread resolution/dismissal: NOT USED (not authorized)
- manual CI dispatch/rerun: NOT USED (not authorized)
- manual Codex/app review request: NOT USED (not authorized)
- provider/runtime read: NOT USED (not authorized)
- provider/runtime write: NOT USED (not authorized)
- migration execution: NOT USED (not authorized)
- release/tag/publication: NOT USED (not authorized)
- programme-pin reconciliation: NOT USED (not authorized)
- merge: NOT USED (not authorized; the implementing agent must not merge its
  own work)
- deployment: NOT USED (not authorized)
- production activation: NOT USED (not authorized)
- other: NONE

Unauthorized actions performed: NONE

## 4.3 Automatic and manual external effects

Automatic CI/provider effects observed (for example a workflow triggered by
opening the PR, and whether it performed any external write such as a
container-registry push):

Pushing the candidate to the PR head triggers the repository's
`pull_request` workflows (`build-test-and-check`, `check-changelog`,
`publish-to-dockerhub`, `run-migrations`). Every path in the change set is
`CHANGELOG.md` or under `docs/`, which `.github/scripts/classify_ci_changes.py`
classifies as documentation-only, so the `classify` and `check-changelog` jobs
are expected to run and pass while the build, test, lint, format, migration and
Docker build/push jobs are expected to be skipped. No container-registry push
and no migration execution is expected. Under Amendment 3 C2/C11, configured
Codex/app review is not expected from a push to a draft PR; final-head
Codex/app review is expected at the separately authorized Draft -> Ready
transition after the approval-state commit. The actual job outcomes and any
automatic review for this candidate exist only after publication; they are
GitHub-owned lifecycle evidence recorded in the pull request and the
post-publication handoff, not in this file.

Historical automatic CI for earlier PR heads followed the same
documentation-only classification, with no manual dispatch or rerun:

```text
7f7e62824eb40308688b46351ffea182e9e52942: 4 success, 6 skipped
227057cae8a4c10f7f7795e3afe68271e56681b9: 4 success, 6 skipped
b4670b8b7cb6d61bc42fb8fd2c40b1d8acf70b20: 4 success (classify x3, check-changelog), 6 skipped
```

Manually initiated external actions (anything the implementing agent
triggered outside the normal push/PR flow, e.g. a manual workflow dispatch):
NONE

External writes/publication (releases, tags, packages, registries, third-party
services): NONE

## 5. Implementation decisions

List decisions made within the approved design:

1. An activation epoch is expressed as the half-open routing-generation
   interval `[activation_generation, deactivation_generation)`; an open epoch
   is the absence of a deactivation boundary (Amendment 1 A2.2).
2. Closing an epoch is a write-once durable close/deactivation fact or an
   equivalently strong append-only close record; once closed, its boundary
   cannot change and the epoch cannot reopen, so the immutability rule never
   implies rewriting an already-written boundary (Amendment 3 C7).
3. To make "closing never rewrites historical eligibility" (A2.8) a property of
   the serialized mechanism rather than an aspiration, the ADR requires the
   mechanism to allocate each deactivation boundary so that closing an epoch
   never removes eligibility from an event that captured a routing generation
   inside it.
4. Retirement versus reactivation follows the durable serialization order
   exactly as A2.6 specifies: retirement after a reactivation closes the open
   epoch; reactivation after retirement is rejected.
5. Any epoch identifier is storage/audit metadata subordinate to `route_key`
   and is excluded from the `(event_id, route_key)` disposition key (A2.11).
6. Section 3.2 now states the disposition model already ruled by the CTO and
   recorded in section 3.5 and invariant 8: exactly one disposition per
   `(event_id, route_key)`; a materialized disposition points to exactly one
   job; a terminal non-materialized disposition points to no job.
7. A retired route's successor, if separately authorized, is a different
   logical route with its own epochs and dispositions and receives earlier
   events only through explicitly authorized backfill/replay; the ADR defines
   no successor mechanism.
8. The header drops the current `Approved by` / `Approval date` fields.
   Section 12 binds the 2026-09-30 approval to the historical pre-correction
   versions without embedding their identities; those identities live in this
   report and the owning issue.
9. Section 12's approval, authority and supersession text is written to remain
   true in both the `PROPOSED` and `APPROVED` states, so the later
   approval-state delta is limited to the status line, the approver/date and
   the sentence that says the corrected content is not yet approved.
10. The decision-register header `Last updated` line is not changed, because
    the authorization permits only the ADR-0012 row to change.
11. The report is restructured to the template rather than appended to, so its
    durable current state is readable without implying that historical
    approvals cover the corrected version.

List any deviation from the specification requiring authorization:

- An earlier unpublished local candidate,
  `a0e0c2cd6809fb2b88d15235d07086f32e4a699c`, carried a version of this report
  that described the push, the task-branch fast-forward, the PR description
  update, the PR head and the PR diff as already complete before they had
  happened. Control rejected it before publication and authorized this
  replacement in #958 comment `5998119577`. The rejected candidate was not
  pushed and is not a parent of this candidate. This replacement is rebuilt
  from the same two authorized parents with the same ADR-0012,
  decision-register and changelog bytes; only this report differs, to correct
  that lifecycle wording.

## 6. Database and migration effects

Migration added: NO

No schema, data, index, constraint or `thoth-api/src/schema.rs` effect. The
activation-epoch model is architecture for a future separately authorized
shared-engine implementation; it creates no migration obligation by itself.

## 7. API and compatibility effects

GraphQL/API changes: NONE
Generated schema/client updates: NONE
Backwards compatibility: unaffected; documentation/architecture only
Deprecations: NONE
Cross-repository dependencies: NONE for this correction. The verified BE-04
per-consumer impact record for the ADR's later legacy-contract retirement is
preserved unchanged in Appendix A; the activation-epoch correction changes no
released contract and adds no consumer.

## 8. Authorization and security

Authorization paths changed: NONE
Roles/scopes involved: NONE
Negative authorization tests: NOT APPLICABLE
Secret or personal-data handling: NONE
Security limitations: NONE

## 9. Tests and checks

Record exact commands and outcomes.

Validation environment: a full local Git worktree of `thoth-pub/thoth`,
detached at `b4670b8b7cb6d61bc42fb8fd2c40b1d8acf70b20`, with every object
fetched from `origin`. Immediately before construction, `git ls-remote origin`
returned `develop` and `feature/worker` at
`b44303c214498baf76c9d9a0a7cab374377f9cbe` and `feature/async/adr-0012` at
`b4670b8b7cb6d61bc42fb8fd2c40b1d8acf70b20`.

The candidate index was built by merging parent 2 into parent 1 without
committing, recording the unmodified merge tree, then overlaying only the four
correction paths. The checks below ran against that exact staged index before
the candidate commit was created from it. Verification of the created commit's
parents and tree, and of the remote head after push, follows the creation of the
commit that contains this report and is recorded in the post-publication
handoff. Commands that compare with a fixed commit or tree use the full
identities shown.

### Formatting

Command:

```text
git diff --check
git diff --cached --check b44303c214498baf76c9d9a0a7cab374377f9cbe
git diff --cached --check e15607878e66d2ff12b819dcea13293a46674f13
git diff --cached --check b4670b8b7cb6d61bc42fb8fd2c40b1d8acf70b20
```

Result:

```text
each: exit 0; no output (no whitespace errors)
```

### Unit tests

Command:

```text
NOT APPLICABLE - documentation/architecture-only change; no Rust source changed
```

Result:

```text
NOT RUN
```

### Integration/database tests

Command:

```text
NOT APPLICABLE - no migration, schema or database code changed
```

Result:

```text
NOT RUN
```

### Lint/static analysis

Command:

```text
NOT APPLICABLE - no Rust source changed
```

Result:

```text
NOT RUN
```

### Other required checks

Merge geometry:

```text
git merge-tree --write-tree b4670b8b7cb6d61bc42fb8fd2c40b1d8acf70b20 b44303c214498baf76c9d9a0a7cab374377f9cbe
exit 0; e15607878e66d2ff12b819dcea13293a46674f13 (no conflicts)

git merge --no-ff --no-commit b44303c214498baf76c9d9a0a7cab374377f9cbe   (from detached b4670b8b7cb6d61bc42fb8fd2c40b1d8acf70b20)
"Automatic merge went well; stopped before committing as requested"

git write-tree   (before any correction byte was staged)
e15607878e66d2ff12b819dcea13293a46674f13
```

Cumulative footprint against refreshed `feature/worker`:

```text
git diff --cached --name-only b44303c214498baf76c9d9a0a7cab374377f9cbe
CHANGELOG.md
docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
docs/engineering/decisions/ADR-0008-machine-roles-and-durable-job-primitives.md
docs/engineering/decisions/ADR-0010-staff-operations-console.md
docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
docs/engineering/decisions/decision-register.md
```

Correction delta against the unmodified merge-tree baseline:

```text
git diff --cached --name-only e15607878e66d2ff12b819dcea13293a46674f13
CHANGELOG.md
docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
docs/engineering/decisions/decision-register.md
```

Delta against the task head before this correction (four correction paths plus
the closed inherited set; `CHANGELOG.md` belongs to both):

```text
git diff --cached --name-only b4670b8b7cb6d61bc42fb8fd2c40b1d8acf70b20
CHANGELOG.md
docs/engineering/ai-delivery/implementation-reports/CTRL-ADR-AMEND-01-implementation-report.md
docs/engineering/ai-delivery/implementation-reports/CTRL-ADR-PR-FIRST-01-implementation-report.md
docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
docs/engineering/decisions/README.md
docs/engineering/decisions/decision-register.md

git diff --numstat b4670b8b7cb6d61bc42fb8fd2c40b1d8acf70b20 e15607878e66d2ff12b819dcea13293a46674f13
2	0	CHANGELOG.md
362	0	docs/engineering/ai-delivery/implementation-reports/CTRL-ADR-AMEND-01-implementation-report.md
382	0	docs/engineering/ai-delivery/implementation-reports/CTRL-ADR-PR-FIRST-01-implementation-report.md
181	9	docs/engineering/decisions/README.md
```

Changelog and register composition:

```text
git diff --cached --numstat e15607878e66d2ff12b819dcea13293a46674f13 -- CHANGELOG.md docs/engineering/decisions/decision-register.md
1	0	CHANGELOG.md
1	1	docs/engineering/decisions/decision-register.md
```

The two inherited changelog lines are the `CTRL-ADR-PR-FIRST-01` and
`CTRL-ADR-AMEND-01` entries from refreshed `develop`; the single added line on
top of the merge-tree baseline is the new `THOTH-ASYNC-01-ADR-01-CORRECTION`
entry. The `[Unreleased]` section retains exactly one `### Added`, one
`### Changed` and one `### Fixed` heading, and every earlier ADR-0012 and
ADR-0013 entry is unchanged. The register's single changed line is the ADR-0012
row.

Staged blob identities (condensed `git ls-files -s` output; mode and stage
omitted, paths abbreviated):

```text
git ls-files -s -- <path>
CHANGELOG.md                                          83882c489df11c0b7c8bf3ce76e7f25fe372cd18
ADR-0012                                              a88aed0a0a17b6dd7a7b852a0e82d8da3fa19b2c
decision-register.md                                  ce8811566c27d5ff769c4fbb14b92ac120d71744
ADR-0008                                              622060ad90eec41c792f110c373efffbb11a4b56
ADR-0010                                              aca2142a3387785e80db908b3a5c1b0afd82ab51
ADR-0013                                              d928f957bdf775d03e99fc73888ec8e2dec5f80b
decisions/README.md                                   de7769a4ee6a50c79c1b57ae6993d7eb4fa673d7
CTRL-ADR-AMEND-01 implementation report               7bf7425f0527dbaa72e718df5264e399035a148b
CTRL-ADR-PR-FIRST-01 implementation report            d86cf215866750979eafea040426c51374d5155c

git hash-object --no-filters docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
a88aed0a0a17b6dd7a7b852a0e82d8da3fa19b2c
```

Candidate ADR-0012 blob (`PROPOSED`): `a88aed0a0a17b6dd7a7b852a0e82d8da3fa19b2c`. This report's own blob
and the candidate tree depend on this report's bytes, so they are not
self-pinned here; they are recorded in the post-publication handoff and, once
published, are GitHub-owned lifecycle evidence.

Commit topology checks, run after the commit that contains this report is
created and before any push; their results are recorded in the
post-publication handoff:

```text
git rev-list --parents -n 1 HEAD
required: <candidate> b4670b8b7cb6d61bc42fb8fd2c40b1d8acf70b20 b44303c214498baf76c9d9a0a7cab374377f9cbe

git rev-parse HEAD^{tree}
required: equal to the git write-tree output of the validated index
```

Content checks:

- the ADR heading outline is unchanged; the header carries `Status: PROPOSED`
  and no current approver or approval date;
- ADR-0012's `Supersedes in part` header text is unchanged and still matches
  the partial-supersession text in ADR-0008 and ADR-0010;
- no ADR-0012 sentence outside section 12 asserts that the corrected content is
  approved;
- internal links in the changed files resolve to existing repository files.

## 10. Manual verification

Environment: the local worktree described in section 9.
Steps: compare the corrected ADR against the prior blob
`dec6665353804e42f22474954bf878308092b717` for every Amendment 1 A2 invariant,
Amendment 3 C7 and base-specification section 3; confirm that no unrelated
architecture changed; confirm the register, changelog and report state
`PROPOSED` and bind the 2026-09-30 approval to the historical versions only.
Observed result: all twelve A2 invariants are expressed in section 3.2 and
invariant 3 and exercised by section 11 validation requirements; C7's
write-once close wording is used; text unrelated to route activation, the
disposition model and approval state is byte-identical to the prior blob.
Evidence link/screenshot/log reference: the local validation in section 9.
PR #960 diffs of earlier heads are historical, pre-candidate evidence only. The
live PR #960 diff of this candidate against `feature/worker` and against
`b4670b8b7cb6d61bc42fb8fd2c40b1d8acf70b20` exists only after publication and is
GitHub-owned lifecycle evidence.

## 11. CI

CI status: GitHub-owned lifecycle evidence for each PR head; expected
documentation-only classification (`classify` and `check-changelog` pass;
build, test, lint, format, migration and Docker jobs skipped).
Checks: as listed in section 4.3.
Failures or warnings: NONE EXPECTED. Under Amendment 3 C10 an automatic CI
failure, or any migration execution or Docker publication, is a HOLD condition.

## 12. Rollout and rollback

Initial state after merge: not applicable to this candidate. Under Route B the
`PROPOSED` candidate is not a merge candidate. Merge requires CTO content
approval of this exact candidate, one approval-state commit, CTO
exact-final-blob approval, Draft -> Ready, final-head CI and Codex/app review,
fresh independent CRITICAL exact-head review and separate CTO merge
authorization, followed by a guarded merge into `feature/worker`.
Activation required: NONE. Merging the eventual approved version would make no
runtime, schema, provider or deployment change.
Feature flag/configuration: NONE
Migration sequence: NONE
Rollback/disable procedure: before merge, stop at the current gate; no history
rewrite, force push or amend is permitted, and any further source change needs a
fresh CTO specification amendment/authorization (Amendment 3 C10).
Monitoring required: NONE

## 13. Known limitations and deferred work

- This candidate stage does not include the approval-state commit, the
  exact-final-blob approval, Draft -> Ready, the final-head review or merge.
- Programme-local reliance on ADR-0012 under ADR-0013 is not effective. The
  stale #957 pin must be superseded using the corrected exact ADR-0012 blob and
  the fresh independently reviewed PR #960 head; dependent `feature/worker`
  slices must be rechecked and recorded as affected or `NONE` with the inspected
  evidence (Amendment 1 A9, Amendment 2 B9, Amendment 3 C9) before that
  supersession; ADR-0013 conditions 7 and 8 then follow the actual merge and
  reachability evidence.
- The repository-wide Route-B recovery ambiguity raised on PR #967 (finding
  `4168223564`) is not resolved here. For #958, Amendment 3 C10 fails closed:
  any correction needing another candidate commit, another approval-state
  commit, amend, rebase, force push or history rewrite requires a fresh CTO
  specification amendment/authorization.
- The projection-evidence limitation of the historical heads
  `227057cae8a4c10f7f7795e3afe68271e56681b9` and
  `b4670b8b7cb6d61bc42fb8fd2c40b1d8acf70b20` (Appendix A) remains historical;
  section 9 is exact-worktree evidence for this candidate.

## 14. Unresolved issues

- The PR #960 review threads `4146838604` and `4148985175` concern
  implementation-report validation evidence. Their Route-B disposition is the
  approval-state report update required by Amendment 3 C5 (literal
  candidate-stage and final validation commands and results, the candidate ADR
  blob, the candidate head as approval-state parent, the six-path and four-path
  footprints and CI applicability, without self-referential containing-commit
  metadata).
- Thread `4146838614` concerns the disposition-to-job mapping; the prior blob
  corrected section 3.5 and invariant 8, and section 3.2 now states the same
  model.
- Thread `4148985154` concerns BE-04 consumer impact; the verified
  per-consumer record is in Appendix A.
- Thread `4156929919` concerns the task-branch namespace; the base
  specification retains the approved transitional exception for
  `feature/async/adr-0012` (section 1).
- Thread `4157508169` concerns repeated route activation; it is the
  `R-b467-01` defect that the activation-epoch correction addresses.

Adjudication of every thread belongs to the fresh independent CRITICAL
final-head review. This report neither resolves nor dismisses any thread.

## 15. Agent self-assessment

The agent may identify risks but may not approve the task.

The implementing agent does not approve, mark merge-ready or merge its own
work. It also prepared the earlier uncommitted activation-epoch candidate
referred to in Amendment 1 A10, so it is not eligible to act as an independent
reviewer of this task.

Suggested review focus:

- that every Amendment 1 A2 invariant and the Amendment 3 C7 write-once close
  rule are expressed in section 3.2 and invariant 3 without changing route
  identity or the `(event_id, route_key)` disposition boundary;
- that the deactivation-boundary allocation requirement (section 5, decision 3)
  is a faithful expression of A2.8 rather than a new mechanism;
- that the emission-floor, rollback and recovery text matches
  base-specification section 3;
- that no current approver, approval date or approval statement presents the
  corrected bytes as approved, and that section 12 remains true after the
  approval-state commit apart from the enumerated approval-state delta;
- that section 12's amendment clause mirrors
  `docs/engineering/decisions/README.md` without loosening it;
- that ADR text unrelated to route activation, the disposition model and
  approval state is byte-identical to the prior blob.

## Appendix A - Historical authoring and correction record

This appendix preserves the earlier task record. Headings are demoted and only
the tense or framing needed to keep it truthful as history is changed.
Statements such as "the ADR now requires" describe ADR-0012 as of the round in
which they were written; later rounds and the Route-B activation-epoch
correction supersede them where they differ. In particular, the round-4 route
model of one activation boundary and one optional deactivation boundary per
route is superseded by the append-only activation-epoch model in the current
section 3.2, and the 2026-09-30 approval and every review recorded here are
bound to the exact historical versions and heads they name.

### Initial candidate and review round 1

Initial candidate:

```text
a69921682cd02e060110304f8f9868d0c39da65e
```

The independent CRITICAL review returned `CHANGES REQUIRED`; durable receipt:
#958 comment `5896883454`.

F-01 through F-09 were accepted. Round-2 candidate:

```text
9d329e063b2731b691c62a412555b470f2a9fe39
```

The round-1 corrections established precise ADR-0008/ADR-0010 boundaries,
crash-safe route identity, effect-phase checkpoints, bounded scheduling,
released-contract handling, worker task-role-only AWS authority, operability and
documentation hygiene.

### Independent review round 2

The independent CRITICAL review of
`9d329e063b2731b691c62a412555b470f2a9fe39` again returned
`CHANGES REQUIRED`; durable receipt: #958 comment `5909933000`.

Seven blockers were accepted:

- R2-01 - incomplete WAITING/resume/attempt/concurrency/reconciliation lifecycle;
- R2-02 - resource-ambiguous idempotency keys and coalescing contradiction;
- R2-03 - event-routing completeness vulnerable to commit-order and mixed-version
  worker assumptions;
- R2-04 - incomplete BE-04 consumer/API coverage, including `thoth-app`;
- R2-05 - unsafe same-release BE-04 table DROP under the actual rolling
  migrate-on-start deployment model;
- R2-06 - untrusted publisher-file parsing sharing high Hosting authority;
- R2-07 - unbounded immutable payload snapshots/personal-data duplication.

### Round-3 corrections

The final tightening also explicitly qualifies ADR-0010 section 4.4's statement
that ADR-0008 remains fully binding: `ServiceOperation` remains non-queue audit
infrastructure, while ADR-0012 supplies the later explicit shared-framework
exception ADR-0008 anticipated.

#### Job lifecycle

The ADR now requires:

- WAITING to atomically persist checkpoint state and release claim/lease;
- WAITING/poll attempts not to consume the pre-write retry budget, instead being
  bounded by the waiting deadline/poll policy;
- resume from WAITING as a new claim/attempt;
- separate bounded pre-write retry budget and post-write waiting/deadline;
- post-write deadline exhaustion to become `RECONCILIATION_REQUIRED`, not
  `FAILED`;
- WAITING and RECONCILIATION_REQUIRED to retain their concurrency key by
  default;
- effectful work to start only when its remaining lease/runtime can cover the
  declared effect window;
- reconciliation itself to be a claimed, claim-token-fenced, kind-authorized
  attempt;
- durable `EFFECT_CONFIRMED` evidence to permit non-duplicating recovery where
  a kind defines it.

#### Effect-scoped idempotency and coalescing

Idempotency now identifies a specific intended effect, not merely a resource.

Keys are derived from the event/route, source revision/fingerprint, schedule
slot, command/request id or explicit coalescing window as appropriate.

Each `(event_id, route_key)` has its own durable route record. Several route
records may target one not-yet-started job only under an explicit coalescing
rule. Idempotency conflicts never silently mark a route complete.

#### Routing completeness

Route registrations and activation boundaries are durable PostgreSQL state.
Activation uses a durable routing generation/epoch or equivalently strong
serialized database mechanism captured transactionally by events/routes, so
out-of-order commits cannot create an ambiguous boundary.

Routing completeness is determined per eligible route, not from worker-local
handler knowledge or an unproved sequence high-water mark. Mixed worker
versions cannot silently complete an unknown active route, and backfill uses the
same route materialization records.

#### Payload minimization

Event/job payloads default to canonical IDs, source revision/fingerprint and
minimal command/routing parameters.

Credentials are prohibited. Personal data or immutable domain snapshots require
kind-specific justification and retention/erasure handling. Current-state work
reads canonical state at execution time.

#### BE-04 expand-migrate-contract transition

The previous state-translating compatibility-facade design has been removed.

The architecture now requires:

1. **Expand** - Release N adds the generic schema/API while retaining legacy
   BE-04 schema/API and keeping legacy automatic job creation inactive.
2. **Migrate consumers** - independently migrate both known released consumers:
   `thoth-dissemination` and `thoth-app`.
3. **Retire legacy API** - later remove legacy lifecycle/read/filter/toggle
   GraphQL contract after all consumers are verified migrated.
4. **Contract storage** - only in a still-later migration, after old code is no
   longer a supported rollback target; the migration itself asserts all three
   legacy tables contain zero rows and aborts otherwise.

Known `thoth-app` dependencies are explicitly recorded:
`latestBackCatalogueJob`, `jobStatuses`,
`withoutBackCatalogueJob` and generated `DistributionJobStatus` contract.

Historical `20260814_v1.7.0` remains immutable.
`distribution_platform` remains.

#### Deployment and rollback

The ADR now incorporates the authoritative current deployment property that the
GraphQL image defaults to `thoth init`, which runs migrations before serving,
while production uses multiple rolling ECS tasks.

Therefore destructive contraction cannot occur in the same release that moves
the API. The previous binary/schema remains rollback-compatible throughout
expand/migrate/API-retirement phases; after generic data exists, runtime rollback
must preserve that data and use forward recovery rather than deleting/reverting
it.

#### Trust-separated worker pools

The default trusted worker still uses one routine task role containing the union
of approved routine trusted capabilities.

Handlers that parse publisher-controlled/untrusted content are a distinct
`UNTRUSTED_CONTENT` risk class and run in a separate task/service/process
boundary with minimal authority and no Hosting DNS/ACM/CloudFront
tenant-management permissions unless explicit CTO risk acceptance records the
blast radius. That pool also does not inherit the primary application database
credential: it must use the kind-scoped executor API or a separately reviewed
minimal database principal.

This is a real trust-boundary exception, not one IAM role per ordinary handler.

#### Validation additions

The ADR now requires tests for:

- mixed worker versions;
- out-of-order commits;
- route completeness/backfill;
- effect-scoped idempotency and coalescing;
- WAITING claim release/resume;
- pre-write versus post-write exhaustion;
- concurrency-key retention;
- claimed reconciliation races;
- payload minimization;
- untrusted-worker IAM isolation;
- both downstream consumer migrations;
- rolling deploy/rollback safety;
- contraction abort on any legacy row.

### Independent review round 3

The independent CRITICAL review of
`2d5eb1ae1e25bc222c4ca94a4f6dc5e81ecceba8` returned
`CHANGES REQUIRED`; durable receipt: #958 comment `5911717880`.

Four blockers were accepted:

- R3-01 HIGH - route activation had no durable deactivation/retirement boundary
  and was not gated on every serving/rollback binary emitting the consumed event
  kind;
- R3-02 MEDIUM - Publisher Services still accessed `distribution_job*` outside
  the named GraphQL surfaces, making later DROP unsafe during rolling deploy and
  rollback;
- R3-03 MEDIUM - untrusted-content isolation still allowed served-object writes
  or trusted-handler steering through untrusted destination data;
- R3-04 MEDIUM - job kinds did not declare current-state versus revision-bound
  semantics, permitting an older retried revision to overwrite newer applied
  state.

### Round-4 corrections

#### Route activation, retirement and emission floor

Routes now have durable activation and optional deactivation boundaries under the
same routing-generation mechanism.

Activation/deactivation is separately authorized and audited rather than a side
effect of deploying handler code. Events eligible before deactivation remain
owed after route retirement; route registrations are never deleted to erase
outstanding obligations. An eligible route that cannot be materialized requires
an explicit authorized/audited terminal route disposition.

A route for event kind/version K may activate only after every serving binary
and supported rollback binary capable of the relevant business mutation emits K
atomically. Rollback below that emission floor requires prior deactivation or an
explicitly authorized current-state reconciliation/backfill covering the exact
gap.

#### Revision semantics

Every job kind now declares either:

```text
CURRENT_STATE
REVISION_BOUND
```

CURRENT_STATE jobs read current canonical state when they execute and may safely
coalesce/no-op under their declared rule.

REVISION_BOUND jobs bind to a retained revision/fingerprint and must not apply an
older revision after a newer revision of the same effect target has already been
applied unless the operation is an explicitly authorized historical
replay/rollback.

#### Full BE-04 runtime retirement before contraction

The Phase-C retirement release now covers every runtime access to
`distribution_job*`, not only named legacy GraphQL fields.

The ADR explicitly includes the current
`replacePublisherServiceConfiguration` runtime dependencies:

- automatic back-catalogue creation;
- the fail-closed AUTOMATIC_PUSH activation guard;
- assignment-disable legacy-job cancellation.

Those paths must be redirected to the generic lifecycle or remain conservatively
fail-closed before the legacy tables can be dropped.

A repository-wide source/runtime audit must prove the deployed and supported
rollback Phase-C binaries have zero runtime legacy-table access.

Phase D then performs only the storage DROP plus removal of remaining inert
schema/model declarations. Its migration takes locks preventing a concurrent
legacy write, performs the final zero-row assertion and DROP in the same
fail-closed migration transaction, and retains `distribution_platform`.

#### Untrusted-content quarantine and trusted promotion

UNTRUSTED_CONTENT remains part of the same canonical async engine but executes
in a separate minimal-authority pool.

It may read authorized input and write only quarantine/staging output. It cannot
write served publisher/Hosting locations or choose final domain/bucket/key/
Hosting targets.

Untrusted completion results are explicitly untrusted data. A trusted promotion
job validates the artifact and resolves all authority-bearing destination values
from canonical Thoth state before publishing.

The untrusted worker may report only its declared completion/evidence facts and
cannot create arbitrary downstream work.

If direct database access is chosen instead of the protected kind-scoped
executor API, ordinary table grants are insufficient: the separately reviewed
database boundary must enforce row/kind scope through RLS, security-definer
functions/procedures or an equivalent mechanism.

#### Additional safety clarifications

Round 4 also records that:

- `EFFECT_CONFIRMED` is anti-replay execution evidence and is not by itself
  ADR-0010 observed publication/acceptance/current-state truth;
- attempt diagnostics/provider evidence are covered by data
  minimization/retention/erasure rules;
- jobs blocked behind WAITING/RECONCILIATION_REQUIRED concurrency keys must be
  operationally visible;
- high-volume job families require retention/coalescing/admission/backpressure
  policy before activation;
- graceful-shutdown rules apply to every worker pool.

### Independent review round 4

The independent CRITICAL review of
`ced4cb19224d7acc1824863a85d83b2387617449` returned `CHANGES REQUIRED`.

Three MEDIUM blockers were accepted:

- R4-01 - durable route obligations lacked route-level backlog/materializer
  visibility and could be terminally dispositioned without explicit divergence
  attention;
- R4-02 - UNTRUSTED_CONTENT IAM/database separation did not prevent cross-tenant
  persistence or network access to shared private services;
- R4-03 - stale-write protection and concurrency were still kind-scoped rather
  than shared by every job kind mutating the same external effect target.

The accepted non-blocking clarifications also covered the BE-04 migration-backfill
runtime path, queue-specific error-contract retirement, canonical writes outside
normal event-emitting binaries, safe job-kind retirement, explicit upstream
Publisher Services generic creation ownership and contraction-revert semantics.

### Round-5 corrections

#### Route obligations and materializer health

The ADR now requires per-route count/oldest-age signals for eligible events with
no disposition, explicit visibility when no compatible materializer is deployed,
and route activation only after both the event-emission floor and a compatible
materializer exist. A terminal route disposition without covering
reconciliation/backfill is an accepted divergence requiring durable attention;
bulk actions still create one audited disposition per event/route.

#### Canonical external effect targets

Effectful kinds now resolve domain-owned canonical external effect-target
identities independent of job kind. Every writer of one target shares the same
target-scoped concurrency namespace and durable applied-revision/fingerprint
evidence. Batch jobs decompose by target by default; retained multi-target jobs
must acquire complete target serialization deterministically and maintain
per-target evidence.

#### Stronger untrusted-content isolation

UNTRUSTED_CONTENT execution now requires cross-publisher process/state isolation,
job-scoped staging authority, independently verified trusted promotion and a
deny-by-default network boundary. The pool cannot inherit private-subnet access to
Redis, EFS, unrelated RDS/database endpoints or other internal services merely
because IAM/database credentials are restricted.

#### Retirement and rollback clarifications

The ADR now names the current migration-backfill runtime path and released
`DISTRIBUTION_JOB_CREATION_DISABLED` compatibility surface in Phase C, requires
non-emitting migration/repair writes to emit transactionally or receive exact-scope
reconciliation/backfill, prevents job-kind retirement from orphaning non-terminal
work, assigns the Publisher Services generic creation/cancellation/activation
contract upstream to `thoth`, and makes Phase-D contraction a forward-repair
boundary rather than an automatic restoration of BE-04.

#### Validation additions

Round 5 adds explicit validation for route backlog/materializer health, accepted
divergence, cross-kind target serialization/evidence, provider-without-revision
ordering, cross-publisher untrusted isolation, per-job staging, private-network
denial, trusted staging provenance, job-kind retirement, non-binary domain writes,
migration-backfill/error-contract retirement and contraction-revert behaviour.

### Independent review round 5

The independent CRITICAL review of
`d25bb1ecd1665cc4e635b16dbd5fd09582a5889b` returned `CHANGES REQUIRED`.
R4-01 through R4-03 were verified closed with no regression identified in the
earlier F/R2/R3 findings.

One MEDIUM blocker was accepted:

- R5-01 - trusted promotion validated a staged object but did not bind
  publication to the exact immutable bytes/version validated or require the
  untrusted writer's authority to end first, leaving a validate-then-swap TOCTOU
  path.

The missing round-4/round-5 lifecycle chain was reconciled in GitHub under #958
comment `5912886635`; the programme gate was reconciled under #957 comment
`5912887392`.

### Round-6 corrections

#### Trusted promotion integrity

Untrusted staging-write authority is now current-claim-scoped and bounded in
lifetime. Trusted validation cannot begin until write authority over the selected
artifact has ended (or an unrevocable capability has expired / an immutable
provider-version boundary is established). The trusted side seals and resolves
the artifact identity, independently validates it, and may publish only the exact
immutable bytes/version that passed validation. A mutable key cannot bridge the
validation/publication boundary; any replacement requires fresh validation.

#### Effect-target fencing and identity

Target evidence is now current-claim-fenced, updated only while holding the
target serialization boundary and committed atomically with the durable local
effect/job outcome. It is monotonic except under authorized historical replay.
CURRENT_STATE fingerprints cover every canonical input determining the effect.
Exactly one domain owns a shared effect-target identity; identity changes have
explicit old/new-target disposition rules; multi-target locking is all-or-nothing
rather than waiting while holding a subset.

#### Untrusted execution and egress

The isolation rule is job-to-job, including jobs for the same publisher. Staging
capabilities are claim-bound and excluded from durable payload/attempt records.
The deny-by-default network boundary explicitly covers outbound traffic, limits
object storage to approved Thoth input/staging locations and requires explicit
allowlisting of other dependencies.

#### Operator and validation tightening

Staff-created terminal route dispositions use ADR-0010's protected audited command
seam. Validation now covers complete CURRENT_STATE fingerprints, target-evidence
claim fencing and monotonicity, target-identity transitions, all-or-nothing
multi-target locking, same-publisher job isolation, staging-capability expiry,
egress restrictions and staged-object replacement after validation.

### Migration/data effect

No migration was created, modified or executed.

Production's v1.7.0 legacy schema fact remains an architecture input. The ADR no
longer proposes immediate replacement/drop: generic schema expansion precedes
consumer migration/API retirement, and legacy storage contraction is a later
separately authorized migration.

### Authorization/security effect

No authorization implementation changed.

No IAM role/policy, credential, provider configuration or environment variable
was created, rotated or changed.

ADR-0008 domain-specific application authorization remains binding. Worker-pool
IAM authority and ZITADEL/application authorization remain separate concepts.

### Approval-state and review history (pre-correction versions)

The architecture content at
`631e28d1f05495f24ce88369d4557f1033f970b3` received CTO exact-content
approval before the durable approval-state reconciliation.

That approval-state reconciliation made the following durable control changes
without altering the selected architecture:

- ADR-0012 recorded `Status: APPROVED`, `Approved by: Javi, CTO`,
  `Approval date: 2026-09-30` and the approval/authority rule in section 12;
- ADR-0008's header recorded only the explicit partial-supersession boundary
  selected by ADR-0012, leaving its unaffected machine-role and least-privilege
  controls binding;
- ADR-0010's header recorded only the explicit partial-supersession boundary
  selected by ADR-0012, leaving its unaffected Staff Operations,
  `ServiceOperation`, desired/execution/observed-state, attention,
  reconciliation and staff-command controls binding;
- the engineering decision register recorded ADR-0012 as `APPROVED` with its
  repository-authority condition and preserved the distinction between
  architecture approval and implementation authorization;
- the Unreleased changelog gained the durable ADR-0012 approval entry while
  preserving the earlier proposal entry.

The five-path approval-state source authorization existed before that mutation.
Its omitted GitHub authorization receipt was reconciled as historical provenance
in #958 comment `5914033493`; the reconciliation explicitly does not portray
the later comment as retroactive authorization.

The approval-state candidate reached
`c43aa750ea4d971fe4f803cf761868876f0783bd` and received
`CHANGES REQUIRED` for AS-01. The direct-child AS-01 correction
`bd5223deee44b465dd860ff96b75d81ad41cbf2d` restored the repository ADR
amendment rule. Fresh independent CRITICAL review then returned `APPROVED`
for that exact head in #958 comment `5914564111`.

The later programme-integration compatibility reconciliation produced
`7f7e62824eb40308688b46351ffea182e9e52942` against
`feature/worker @ 345a7a04131e7c0539f7518c6ea457fa5cb6a462`. Fresh independent
CRITICAL review of that exact integration candidate returned `CHANGES REQUIRED`
in #958 comment `5919509096`. The accepted blockers were missing exact
validation evidence, transient/incomplete durable report history and the
contradiction between section 3.2's terminal route dispositions and the
section 3.5/invariant-8 every-route-record-maps-to-a-job wording. The latter is
a factual clarification of the already-selected section 3.2 semantics, not a
new architecture decision.

### Verified BE-04 consumer impact

The independent consumer-impact check for the retiring BE-04 identifiers is
durably recorded as follows:

- `thoth-pub/thoth-app`: downstream migration required;
- `thoth-pub/thoth-dissemination`: downstream migration required;
- `thoth-pub/thoth-pyramid`: unaffected by the retiring BE-04 identifiers;
- standalone `thoth-pub/thoth-client`: unaffected;
- internal Rust `thoth-client` / `thoth-export-server`: unaffected;
- `thoth-pub/metrics-dashboard`: unaffected;
- `thoth-pub/metrics-widget`: unaffected;
- `thoth-pub/baboon`: unaffected;
- `thoth-pub/thoth-sphinx`: not a verified active consumer;
- `thoth-pub/infrastructure`: runtime/IAM substrate, not a GraphQL consumer.

This record is compatibility evidence for the BE-04 retirement plan. It does
not authorize either downstream migration or removal of the legacy contract.

### Programme-integration compatibility reconciliation

After ADR-0013 became repository-authoritative and THOTH-ASYNC-01 explicitly
opted into programme-integration reliance, the programme created
`feature/worker` from current `develop` and retargeted PR #960 to that
programme integration branch.

The compatibility reconciliation is bound to:

```text
reviewed ADR-0012 source before reconciliation:
bd5223deee44b465dd860ff96b75d81ad41cbf2d

programme integration base:
feature/worker @ 345a7a04131e7c0539f7518c6ea457fa5cb6a462

historical common base:
923545d5c9028bc04c40e38efeb7de674efed3fd
```

The histories had diverged. The only content changed on both sides since the
historical common base was `CHANGELOG.md` and
`docs/engineering/decisions/decision-register.md`.

The compatibility reconciliation preserved the then-reviewed ADR-0012,
ADR-0008 and ADR-0010 decision blobs byte-for-byte, while carrying the
repository-authoritative ADR-0013, branching workflow and operating-model
doctrine from `feature/worker` unchanged. The subsequent factual ADR-0012
clarification changed the exact ADR-0012 version but did not alter ADR-0013 or
those shared control documents.

ADR-0012's normal repository-authority condition remains reachability from
`develop`. ADR-0013's narrower programme-local reliance mechanism applies only
to an exact approved and independently reviewed ADR version that is merged into
the designated `feature/worker` line and durably pinned by the programme. A
change to ADR-0012 makes any earlier exact-version programme pin stale until the
changed version has received fresh independent CRITICAL review, explicit CTO
approval and separate programme-pin reconciliation. This report does not assert
that programme-local reliance is currently effective.

### Historical external/runtime effects

```text
provider reads: 0
provider writes: 0
migration execution: 0
CI dispatch/rerun: 0
deployment: 0
release/publication: 0
production activation: 0
```

### Historical validation evidence and provenance

#### Correction at `227057cae8a4c10f7f7795e3afe68271e56681b9`

Before the route-disposition clarification commit, the proposed source bytes
were frozen, the three modified-file blob identities were computed, and the
created GitHub blobs matched those expected identities exactly. The cumulative
six-path footprint, the three-path correction boundary, the unchanged control
blobs and full-file whitespace/composition checks were also verified.

The literal commands recorded by the report version at that head:

```text
git diff --check HEAD
git diff --name-only HEAD
```

were run in a controlled six-path **projection** workspace. That workspace did
not contain all six exact GitHub file bytes. Those command results are therefore
historical projection evidence only and are not represented here as exact-byte
Git validation of that candidate. This provenance defect was recorded
durably in #958 comment `5934574460` and the corresponding programme HOLD in
#957 comment `5934575236`.

After push, GitHub verification of exact head
`227057cae8a4c10f7f7795e3afe68271e56681b9` established:

```text
tree:
1645d0ebdca5df231d3fa72ede229cccab76771e

parent:
7f7e62824eb40308688b46351ffea182e9e52942

correction paths:
CHANGELOG.md
docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md

cumulative PR paths against feature/worker:
CHANGELOG.md
docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
docs/engineering/decisions/ADR-0008-machine-roles-and-durable-job-primitives.md
docs/engineering/decisions/ADR-0010-staff-operations-console.md
docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
docs/engineering/decisions/decision-register.md
```

Exact final blobs at that head were:

```text
CHANGELOG.md:
66cfbb5b56dfe7a9382614e274a872c68df26dcb

implementation report:
104df8c02d8aa8077e720a63bb3c16415afe54c3

ADR-0008:
622060ad90eec41c792f110c373efffbb11a4b56

ADR-0010:
aca2142a3387785e80db908b3a5c1b0afd82ab51

ADR-0012:
dec6665353804e42f22474954bf878308092b717

decision register:
fc64dddbf6beddd602d31d694d9dae7c40c417c2

ADR-0013:
d928f957bdf775d03e99fc73888ec8e2dec5f80b

branching-and-release-workflow:
6edce35dd29307b0554bcd122ed5518038f57df5

operating-model:
fc8b9c90a7d0ae98a44645d2e316a0573c13ce2a
```

Automatic PR CI for that head completed with four successful jobs and six
classification-skipped jobs, with no failure or in-progress job and no manual
dispatch/rerun.

#### Report-only provenance correction at `b4670b8b7cb6d61bc42fb8fd2c40b1d8acf70b20`

The correction authorized by #958 comment `5934661557` changed only the
implementation report from parent
`227057cae8a4c10f7f7795e3afe68271e56681b9`.

Before commit, the complete candidate bytes for all six cumulative PR paths were
read from the exact GitHub parent, with only the report replaced by its
proposed corrected bytes. Full-file byte-level whitespace checks found no
trailing whitespace on any candidate file.

The exact cumulative-base blobs are:

```text
CHANGELOG.md:
cf662733b61354f08f7d6b26ead3b8079f10d8ba

implementation report:
ABSENT

ADR-0008:
19273fe60520b8a907c8b5a28e76d4fe741bb832

ADR-0010:
0e3b047435dbdcd2fdd15cbfc9ac4c1fc2e601ce

ADR-0012:
ABSENT

decision register:
ed03bcc6766f05fb7f026468689727b3c42f4c94
```

The five unchanged candidate paths retained their exact parent blob identities:

```text
CHANGELOG.md:
66cfbb5b56dfe7a9382614e274a872c68df26dcb

ADR-0008:
622060ad90eec41c792f110c373efffbb11a4b56

ADR-0010:
aca2142a3387785e80db908b3a5c1b0afd82ab51

ADR-0012:
dec6665353804e42f22474954bf878308092b717

decision register:
fc64dddbf6beddd602d31d694d9dae7c40c417c2
```

The proposed report bytes were Git-blob hashed before commit; the exact proposed
blob is recorded in the owning #958 completion handoff rather than self-pinned
inside the bytes whose hash it would change.

GitHub's exact base-to-parent comparison contains exactly the six cumulative PR
paths listed above. Because that correction changed only the report, the
cumulative path set remained those same six paths and its direct correction
delta was exactly:

```text
docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
```

The execution shell available for that control step could not resolve GitHub
and could not materialize an exact repository worktree. Consequently that report
version did **not** claim that literal local Git commands were executed against a full
exact-byte worktree.

The required literal commands were nevertheless re-run without restrictive
pathspecs in a six-path path/projection workspace after the exact GitHub-byte
checks:

```text
git diff --check HEAD

exit: 0
stdout: ""
stderr: ""

git diff --name-only HEAD

exit: 0
stdout:
CHANGELOG.md
docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md
docs/engineering/decisions/ADR-0008-machine-roles-and-durable-job-primitives.md
docs/engineering/decisions/ADR-0010-staff-operations-console.md
docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md
docs/engineering/decisions/decision-register.md
stderr: ""
```

Those literal command results are projection evidence only. Exact-byte evidence
was instead the complete GitHub file reads, Git blob identities, full-file
whitespace checks and exact GitHub compare path sets above. This limitation
remains part of the historical record; it is not converted into stronger
evidence by assertion.

Composition and doctrine checks at that head:

- changelog had one `[Unreleased]` structure and retained ADR-0013, ADR-0012
  proposal, approval and factual-clarification entries;
- decision register was byte-identical to its parent and recorded ADR-0012 as
  `APPROVED` while retaining ADR-0013's programme-integration rule;
- section 3.2's terminal-disposition semantics remained unchanged and section 3.5
  plus invariant 8 used the same materialized-versus-terminal disposition model;
- ADR-0013, branching workflow and operating-model doctrine remained
  byte-preserved;
- repository names used canonical spellings, including `thoth-sphinx`.

Test applicability:

```text
runtime/unit tests: NOT APPLICABLE
database/migration tests: NOT APPLICABLE
provider/runtime tests: NOT APPLICABLE
manual CI dispatch/rerun: NO
final-head automatic CI: GitHub-owned post-push evidence
```

### Historical deviation history

During initial issue creation, the GitHub connector omitted the issue number in
its normalized create response, briefly producing a `#undefined` parent
reference in #958. The returned URLs established #957/#958 and the issue was
corrected before branch/source mutation.

The source write budget remained compliant. The validation-evidence provenance
deviation at head `227057cae8a4c10f7f7795e3afe68271e56681b9` is recorded in
#958 comment `5934574460`; the report-only correction at
`b4670b8b7cb6d61bc42fb8fd2c40b1d8acf70b20` removed the unsupported
exact-worktree claim rather than retroactively curing it.

## Appendix B - Durable control conditions

These conditions remain true regardless of the current live PR/review/merge
stage:

1. ADR-0012 is an architecture decision only. Approval or merge does not
   authorize runtime implementation, schema/data migration, provider/IAM/runtime
   mutation, external writes, deployment, release or production activation.
2. Any exact ADR-0012 version used for merge-readiness or programme-local
   reliance requires the review/approval evidence required by repository
   doctrine for that exact version; a later source change invalidates
   exact-head approval for the earlier version.
3. ADR-0012 becomes repository-authoritative only under its section-12 authority
   condition, including reachability from `develop`.
4. ADR-0013 programme-local reliance, when used, is exact-version-bound: the
   relied-upon ADR version must be approved, independently reviewed, merged into
   the designated `feature/worker` line and durably pinned. Any later ADR-0012
   change makes the earlier programme pin stale until separately reconciled.
5. Every async implementation slice remains separately specified, authorized,
   independently reviewed and integrated. Downstream repositories may not guess
   an unmerged upstream contract and must use an explicitly authorized pinned
   version/preview where applicable.
6. Merge authorization, merge, migration execution, provider/IAM/runtime
   mutation, deployment, release, production activation and observation remain
   separate lifecycle gates.
7. Historical migration `20260814_v1.7.0` remains immutable; BE-04 retirement
   continues to require the approved expand -> consumer migration -> runtime/API
   retirement -> rollback-floor closure -> later storage-contraction sequence.
8. The terminal route-disposition clarification does not permit silent work
   loss: terminal non-materialization remains explicit, authorized, audited and
   subject to the section 3.2 reconciliation/backfill and attention semantics.
9. The activation-epoch model does not permit silent work loss either: events
   eligible through any epoch remain owed after deactivation and retirement,
   and events from an inactive inter-epoch gap are processed only through
   explicitly authorized backfill/replay under the same `(event_id, route_key)`
   deduplication boundary.
10. A materially corrected ADR-0012 version may carry `APPROVED` only after CTO
    approval of that exact corrected content; earlier approvals and reviews
    remain bound to the earlier exact versions and heads they name.

This report authorizes none of those later actions.
