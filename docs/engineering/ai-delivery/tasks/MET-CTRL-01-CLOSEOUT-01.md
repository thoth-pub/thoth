# MET-CTRL-01-CLOSEOUT-01 - Reconcile post-merge Metrics controls and hand off WP1 entry

Task ID: `MET-CTRL-01-CLOSEOUT-01`
Programme: Thoth Metrics - canonical ingestion, Sphinx orchestration and client cutover
Parent task: `MET-CTRL-01` (issue [#832](https://github.com/thoth-pub/thoth/issues/832))
Programme issue: [#766](https://github.com/thoth-pub/thoth/issues/766)
Task issue: [#834](https://github.com/thoth-pub/thoth/issues/834)
Repository: `thoth-pub/thoth`
Risk: LOW
Workflow: STANDARD documentation/control closeout
Authorized base: `develop @ e62a476e87916d0b44a963652c9c5e7ab6afa10e`
Task branch: `feature/metrics-control/met-ctrl-01-closeout`
PR target: `develop`
Runtime effect: NONE
Migration/data effect: NONE
Auth/security effect: NONE
Provider/production effect: NONE

Status: APPROVED - IMPLEMENTED

Authority condition: this record is repository-authoritative when this exact
content is reachable from `develop`. Live specification-review, implementation
authorization, independent review, merge authorization and merge evidence is
the GitHub issue and pull-request history for [#834](https://github.com/thoth-pub/thoth/issues/834);
this document does not restate it.

## 1. Objective

Correct the durable Thoth Metrics programme state that the merge of
`MET-CTRL-01` made materially wrong.

After that merge, several active repository controls still asserted that
`MET-CTRL-01` was an unsatisfied programme/WP1 entry gate. That is a material
programme-state error under `ADR-0005` section 8, because it misstates what
work is permitted next.

This closeout therefore:

1. records `MET-CTRL-01` as `MERGED - COMPLETE` / satisfied in active controls;
2. removes or historicalizes active pre-merge gate language;
3. leaves the remaining Thoth WP1 entry path explicit and limited to exactly
   two gates;
4. preserves both historical parent control-process provenance exceptions
   without curing either;
5. keeps every later Sphinx/client/source/WP5 gate attached to its owning work;
6. authorizes and starts no Metrics implementation.

## 2. Background and authority

`MET-CTRL-01` was the bounded Metrics programme-control reconciliation. It was
delivered through PR [#833](https://github.com/thoth-pub/thoth/pull/833) and is
reachable from `develop`.

Under [`ADR-0005`](../../decisions/ADR-0005-terminal-merge-evidence.md), GitHub
is the terminal authority for lifecycle evidence. A post-merge repository task is
legitimate only when repository content is materially wrong, not merely because
GitHub lifecycle metadata changed. This task exists for the former reason and
must not become a lifecycle ledger.

### 2.1 Parent lifecycle evidence map

Three distinct records carry the parent lifecycle. They must not be collapsed
into one another:

- PR [#833](https://github.com/thoth-pub/thoth/pull/833) is terminal GitHub
  evidence for the implementation head, CI, the ready/merge event, the merge
  commit and the merge timestamp;
- issue [#832](https://github.com/thoth-pub/thoth/issues/832) comment
  `5414236565` is the durable independent exact-head `APPROVED` source-review
  record;
- CTO merge authorization was explicitly granted in the control-plane
  conversation before the merge was executed, bound to the reviewed exact head.

Active programme trackers may reference PR #833 as the parent lifecycle anchor.
They must not assert that PR #833 alone contains every review and authorization
record, and they are not required to reproduce the reviewed source SHA, the
merge SHA, review identifiers or merge-authorization identifiers.

### 2.2 Historical control-process provenance exceptions

Two distinct historical exceptions attach to the parent task. Both are preserved
and neither is cured by this closeout:

- **Exception A - implementation authorization.** Parent implementation
  mutations - branch creation, documentation edits, commit, push and draft-PR
  creation - occurred before a separate explicit implementation authorization was
  durably recorded.
- **Exception B - merge-authorization ledger.** Merge was explicitly authorized
  by the CTO before execution, bound to the reviewed exact head, but that
  authorization was not durably recorded on PR #833 or #832 before the merge was
  executed.

Exception B is a missing durable GitHub record, not a missing authorization. It
must not be restated as "the merge was unauthorized", and no pre-merge GitHub
authorization record may be invented, backdated or reconstructed.

## 3. Explicit scope

### 3.1 Approved write budget

Existing files:

```text
CHANGELOG.md
docs/metrics/README.md
docs/metrics/task-status.md
docs/metrics/rollout-plan.md
docs/engineering/README.md
docs/engineering/repository-map/control-gaps.md
```

New files:

```text
docs/engineering/ai-delivery/tasks/MET-CTRL-01-CLOSEOUT-01.md
docs/engineering/ai-delivery/implementation-reports/MET-CTRL-01-CLOSEOUT-01-implementation-report.md
```

The smallest necessary subset of this budget must be used. If an active stale
statement outside the budget must change for consistency, the implementation
stops and reports `SCOPE AMENDMENT REQUIRED` with the exact path, statement and
reason.

### 3.2 Required per-file outcome

`docs/metrics/README.md`:

- the programme is no longer described as
  `PROGRAMME CONTROLS UNDER RECONCILIATION (MET-CTRL-01)`;
- `MET-CTRL-01` is recorded as satisfied in durable programme terms;
- implementation remains unauthorized and WP1 remains blocked;
- exactly two remaining WP1 entry gates are shown;
- later Sphinx/client/source/WP5 gates are unchanged in ownership and scope.

`docs/metrics/task-status.md`:

- the closeout provenance header is refreshed;
- `MET-CTRL-01` moves from `ACTIVE`/current merge gate to
  `MERGED - COMPLETE` / dependency satisfied;
- `MET-CTRL-01` closure is removed as an unsatisfied WP1 dependency;
- WP1 stays `BLOCKED` with its two remaining entry gates;
- where `SPHINX-BOOT-01` lists `MET-CTRL-01` as a dependency, only that
  dependency is marked satisfied; `BR-SPHINX-01`, the approved bootstrap
  specification and every other WP6/Sphinx blocker stay intact;
- later work-package dependencies are otherwise unchanged.

`docs/metrics/rollout-plan.md`:

- `MET-CTRL-01` moves out of the remaining Stage-0 WP1-entry gates and into the
  completed shared/global controls;
- later rollout gates are unchanged.

`docs/engineering/README.md`:

- the statement that Thoth WP1 still waits on `MET-CTRL-01` closure is removed;
- the remaining authorization and specification gates are preserved.

`docs/engineering/repository-map/control-gaps.md`, CG-08:

- the `MET-CTRL-01` component is recorded satisfied;
- CG-08 remains `OPEN`;
- `feature/metrics` authorization/creation remains outstanding;
- an approved bounded WP1 child specification remains outstanding;
- CG-03, CG-04, CG-09, CG-10, CG-11 and CG-13 are neither weakened nor closed.

`CHANGELOG.md`:

- one bounded entry describing the material programme-state correction;
- the already-merged `MET-CTRL-01` entry containing point-in-time gate language
  is historical evidence, is explicitly classified as historical during
  validation, and is not rewritten merely because that wording later became
  historical.

### 3.3 Stale-state sweep

Active Metrics and engineering controls are searched exhaustively for statements
materially equivalent to:

- `MET-CTRL-01` is still `ACTIVE`/current;
- `MET-CTRL-01` is awaiting exact-head review;
- PR #833 is awaiting merge;
- `MET-CTRL-01` closure is an unsatisfied WP1 dependency;
- programme controls remain under `MET-CTRL-01` reconciliation.

Every hit is classified as `ACTIVE STALE STATE - correct if inside budget` or
`HISTORICAL EVIDENCE - preserve unchanged`.

## 4. Explicitly out of scope / historical records

The original parent task and implementation report are point-in-time historical
evidence and are not edited to make their pre-merge sections look current:

```text
docs/engineering/ai-delivery/tasks/MET-CTRL-01.md
docs/engineering/ai-delivery/implementation-reports/MET-CTRL-01-implementation-report.md
```

Also not edited:

- `docs/metrics/decisions.md` - architectural/shared-decision state is current;
- `docs/metrics/master-issue.md` - no post-merge stale task status requiring
  source reconciliation;
- `docs/engineering/decisions/ADR-0005-terminal-merge-evidence.md` - this task
  applies it and must not change it;
- `docs/engineering/repository-map/branch-topology.md` - its requirement that
  `MET-CTRL-01` be merged is durable and now satisfied, not stale.

## 5. Non-goals

This closeout must not: create `feature/metrics`; create or implement any WP1
child task; implement WP1; create or modify schema, migrations or
`thoth-api/src/schema.rs`; modify Rust, SQL, GraphQL/API, models or generated
contracts; modify workflows or branch protection; modify Sphinx or any client
repository; perform `BR-SPHINX-01` or `SPHINX-BOOT-01`; choose Metrics service
roles, permissions or credentials; approve source fixtures, mappings, COUNTER
mappings or OPERAS completeness; access provider, runtime or production state;
deploy, release, run migrations or activate production behaviour; erase, backdate
or retroactively cure either provenance exception; misstate the parent merge as
unauthorized; mutate #832 or #766 during source implementation; weaken unrelated
engineering control gaps; create approval-state-only or lifecycle-metadata-only
repository changes prohibited by `ADR-0005`; or create another source closeout
merely to record that this closeout PR later merged.

## 6. Invariants

1. Thoth remains the sole canonical Metrics datastore/API owner; Sphinx remains
   orchestration and interoperability.
2. `ADR-0001`, `ADR-0002`, `ADR-0003`, `ADR-0005` and `ADR-0008` authority is
   unchanged.
3. `MetricPlatform` remains separate from `DistributionPlatform`.
4. `ADR-0003` remains the schema-control authority.
5. `ADR-0008` still selects no Metrics role name, permission set, credential
   model or entitlement model.
6. Exception A remains historical and preserved.
7. Exception B remains historical and preserved, and is not restated as an
   absent authorization.
8. Neither exception is erased, backdated or retroactively cured.
9. `ADR-0005` is binding for lifecycle evidence, and the parent evidence map in
   section 2.1 is preserved exactly.
10. This closeout is justified only as a material programme-state correction and
    must not become recursive lifecycle-metadata work.
11. WP1 remains not implemented and not authorized.
12. Sphinx/client/source/WP5 later gates remain attached to their owning work.
13. CG-03, CG-04, CG-09, CG-10, CG-11 and CG-13 remain accurate.
14. No runtime or production behaviour changes.
15. The implementing agent cannot approve its own closeout work.

## 7. Data, migration, API and security requirements

None. This task changes documentation and control records only. There is no
schema change, no migration, no `schema.rs` change, no GraphQL/API surface
change, no generated-contract change, no authorization or policy change, no
workflow change and no provider, runtime or production effect.

## 8. Acceptance criteria

- [x] Implementation starts from exact authorized base
      `develop @ e62a476e87916d0b44a963652c9c5e7ab6afa10e`.
- [x] Changed files are a subset of the approved write budget.
- [x] No Rust, SQL, migration, GraphQL/API, workflow, generated-contract,
      provider or runtime file changes.
- [x] `MET-CTRL-01` is consistently recorded as `MERGED - COMPLETE` / satisfied
      in active controls.
- [x] Active trackers correct durable programme state without asserting that
      PR #833 alone contains all review/authorization evidence.
- [x] The closeout report preserves the corrected parent evidence map.
- [x] Both historical provenance exceptions remain distinct, preserved and
      uncured.
- [x] `docs/metrics/README.md` no longer says programme controls are under
      `MET-CTRL-01` reconciliation.
- [x] `docs/metrics/task-status.md` no longer lists `MET-CTRL-01` as
      `ACTIVE`/current merge gate.
- [x] The `SPHINX-BOOT-01` dependency reference to `MET-CTRL-01` is explicitly
      marked satisfied while all other Sphinx blockers remain intact.
- [x] `docs/metrics/rollout-plan.md` no longer lists `MET-CTRL-01` closure as an
      unsatisfied WP1-entry gate.
- [x] `docs/engineering/README.md` no longer says WP1 waits on `MET-CTRL-01`
      closure.
- [x] CG-08 records `MET-CTRL-01` satisfied but remains `OPEN`.
- [x] The existing `MET-CTRL-01` changelog entry is classified as historical and
      not rewritten.
- [x] `feature/metrics` still does not exist as a side effect of this task.
- [x] WP1 remains blocked, unimplemented and unapproved.
- [x] Later Sphinx/client/source/WP5 gates remain intact.
- [x] No approval-state-only or lifecycle-metadata-only repository change
      prohibited by `ADR-0005` is introduced.
- [x] `git diff --check` passes.
- [ ] Normal documentation/control PR CI is green at the exact reviewed head
      before merge.
- [ ] Fresh independent exact-head review returns `APPROVED` before merge.
- [ ] Explicit CTO merge authorization for this closeout is bound to the final
      reviewed head and durably recorded before merge.
- [ ] #832 final reconciliation/closure and #766 synchronization remain separate
      post-merge GitHub mutations.
- [ ] After this closeout merges, no second repository closeout task or PR is
      created solely to transcribe that merge.

The unchecked criteria are live gates owned by GitHub and the CTO. They are not
repository state and are not made true by any commit on this branch.

## 9. Required validation

```bash
git status --short
git diff --check
git diff --stat e62a476e87916d0b44a963652c9c5e7ab6afa10e...HEAD
git diff --name-only e62a476e87916d0b44a963652c9c5e7ab6afa10e...HEAD
grep -nE '^### CG-(03|04|09|10|11|13)' docs/engineering/repository-map/control-gaps.md
git branch -a --list '*feature/metrics'
```

Changed files must be a subset of the approved budget. The stale-state sweep of
section 3.3 must be performed and every hit classified.

## 10. Rollout and rollback

Rollout: one bounded task branch from the exact authorized base; one draft PR to
`develop`; automatic CI only; fresh independent exact-head source review;
explicit CTO merge authorization durably recorded before merge and bound to the
exact reviewed head; merge only after those gates. After merge, reconcile #832
and #766 through separate GitHub comments/issue state under `ADR-0005`. Do not
create a second repository closeout to record that this closeout merged. Only
after durable closeout, separately authorize `feature/metrics` and scope and
approve the first bounded WP1 child task.

Rollback: revert the complete documentation/control closeout merge if a material
source rollback is required. Any GitHub issue/comment reconciliation rollback
must preserve later unrelated state and requires separate explicit authorization.

## 11. Stop conditions

Stop if work begins to: create `feature/metrics`; create a WP1 implementation
issue or specification; implement schema, runtime, auth, Sphinx or client work;
select Metrics roles or source mappings; mutate #832 or #766 during source
implementation without separate authorization; access provider, runtime or
production state; erase, backdate or retroactively cure either provenance
exception; claim that merge authorization did not exist, or claim it was durably
recorded on GitHub before merge; weaken unrelated engineering control gaps;
create approval-state-only or lifecycle-metadata-only repository changes
prohibited by `ADR-0005`; or widen into another programme.

## 12. Approval boundary

The implementing agent cannot approve its own work. Merge requires fresh
independent exact-head review and separate explicit CTO merge authorization
durably recorded before merge.
