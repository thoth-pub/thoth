# CTRL-MERGE-01 Implementation Report

Authority condition: this report describes the implementation delivered on the
`CTRL-MERGE-01` branch. The exact head commit, pull-request number, review,
authorization, CI and merge records are the GitHub pull-request record and are
deliberately not transcribed here, consistent with the decision this task
implements.

## 1. Repository state

Repository: `thoth-pub/thoth`
Workflow: STANDARD
Base branch: `develop`
Base commit: `d534ff52b8bda966ee5cfa7a1d03353ef59476ec`
PR target: `develop`
Programme integration branch: None
Task branch: `feature/shared-control/ctrl-merge-01`
Head commit: the exact head of the `CTRL-MERGE-01` pull request; see the GitHub
pull-request record
Pull request: the `CTRL-MERGE-01` pull request into `develop`, opened as draft
Expected branch deletion after merge: YES
Final programme PR required: NO
Implementing model: Claude Opus
Reasoning level: HIGH

## 2. Scope confirmation

Approved specification:
[issue #786](https://github.com/thoth-pub/thoth/issues/786), recorded canonically
in [`CTRL-MERGE-01.md`](../tasks/CTRL-MERGE-01.md).

Implemented objective: replace the recursive approval-state and post-merge
closeout convention with a repository-authoritative terminal merge evidence
rule, without weakening any substantive engineering control.

Out-of-scope changes made: NONE.

## 3. Commits

One bounded documentation commit on `feature/shared-control/ctrl-merge-01`,
subject `docs(control): adopt terminal merge evidence`. No amend, rebase, squash
or force-push was used.

## 4. Files changed

- `docs/engineering/decisions/ADR-0005-terminal-merge-evidence.md`
  - reason: new Shared Engineering Control decision recording the terminal merge
    evidence rule, the lifecycle-evidence authority model, the durable/transient
    writing rule, the retained controls and the rejected alternative.
  - behavioural effect: none until merged; `PROPOSED` on arrival.
- `docs/engineering/ai-delivery/tasks/CTRL-MERGE-01.md`
  - reason: canonical repository task record for this bounded control task.
  - behavioural effect: documentation only.
- `docs/engineering/decisions/decision-register.md`
  - reason: register `ADR-0005` and record the decision narrative.
  - behavioural effect: index only; no existing row altered.
- `docs/engineering/ai-delivery/operating-model.md`
  - reason: add section 3.1 lifecycle-evidence authority order; strengthen Gate 4
    with explicit exact-head binding and the prohibition on commits that copy a
    review into the repository; add sections 5.1 terminal merge evidence and 5.2
    normal lifecycle.
  - behavioural effect: defines how lifecycle evidence is recorded. No change to
    who may review, approve, merge, deploy or activate.
- `docs/engineering/ai-delivery/release-gates.md`
  - reason: add section 1.1 guarded merge and section 1.2 terminal merge
    evidence; state that explicitly gated merges require CTO authorization at any
    risk level; note that closure needs no commit declaring it.
  - behavioural effect: the merge-ready gate's substantive conditions are
    unchanged; the merge mechanism is now explicitly head-guarded.
- `docs/engineering/AGENTS.md`
  - reason: add section 1.1 durable-versus-transient status-writing guidance.
  - behavioural effect: constrains how control documents are written.
- `docs/engineering/ai-delivery/task-specification-template.md`
  - reason: remove the transient lifecycle values from the `Status` field, add an
    authority condition, and state that review/authorization/merge metadata is
    not copied back into the specification.
  - behavioural effect: new specifications record durable state only.
- `docs/engineering/ai-delivery/decision-record-template.md`
  - reason: add the authority condition and the same prohibition for ADRs.
  - behavioural effect: as above.
- `docs/engineering/ai-delivery/independent-review-template.md`
  - reason: state exact-head binding, that the review is recorded on the pull
    request rather than committed, and reviewer independence.
  - behavioural effect: reinforces existing review controls.
- `docs/engineering/ai-delivery/README.md`
  - reason: index the new task record and summarize the lifecycle-evidence rule.
  - behavioural effect: index only.
- `CHANGELOG.md`
  - reason: required changelog entry.
  - behavioural effect: none.
- `docs/engineering/ai-delivery/implementation-reports/CTRL-MERGE-01-implementation-report.md`
  - reason: this report, required by `operating-model.md` Gate 2 step 8.
  - behavioural effect: none.

## 5. Implementation decisions

1. Treated issue #786 as the approved written specification. `operating-model.md`
   Gate 1 accepts a specification "committed or attached to an authoritative
   GitHub issue", and root `AGENTS.md` section 1 accepts a GitHub issue carrying
   the template's required information. No repository rule requires a separately
   merged specification pull request, so no `BLOCKED - CTRL-MERGE-01 REQUIRES
   SEPARATE COMMITTED SPECIFICATION` condition arose. A canonical repository task
   record was still added.
2. Found no authoritative rule mandating approval-state-only commits or
   automatic post-merge closeout pull requests. The recursion is established
   practice, not written control, so no `BLOCKED - CURRENT CONTROL REQUIRES
   RECURSIVE CLOSEOUT` condition arose and no transition exception is needed.
3. Confirmed `ADR-0005` was free; `ADR-0004` was the highest existing number. No
   deviation to record.
4. Scoped the decision explicitly to lifecycle evidence (ADR-0005 section 4.2) so
   it cannot be read as general permission to skip documentation pull requests.
5. Retained the `PROPOSED`/`APPROVED`/`SUPERSEDED`/`REJECTED` ADR vocabulary and
   the `docs/engineering/AGENTS.md` section 1 status terms. These record durable
   decision state owned by the CTO, not transient pull-request state, so they are
   outside the durable/transient rule. Stated this explicitly in both ADR-0005
   section 6 and `AGENTS.md` section 1.1 to prevent over-application.
6. Narrowed the task specification template `Status` field to `DRAFT | APPROVED`.
   The removed values (`IMPLEMENTING`, `IN REVIEW`, `MERGE READY`, `RELEASED`,
   `CLOSED`) are precisely the transient states that GitHub already holds and
   that force a post-merge editing commit.
7. Added the guarded-merge mechanism explicitly to `release-gates.md` with a
   concrete `--match-head-commit` example, since the decision depends on the
   merge failing when the reviewed head has moved.
8. Wrote this report's head commit and pull-request number as references to the
   GitHub record rather than transcribed values. Transcribing them is impossible
   before the commit exists and would otherwise require a second commit - the
   exact recursion being removed. The report satisfies Gate 2 step 8 with
   substantive implementation evidence, which ADR-0005 does not restrict; only
   review/approval/merge metadata copying is prohibited.
9. Preserved all historical records. Every prior mention of approval-state heads,
   closeout tasks and merge identifiers in the Publisher Services, Metrics and
   engineering trackers, registers and ADRs was classified as `HISTORICAL RECORD
   - PRESERVE` and left untouched. No global find-and-replace was performed.

Deviations from the specification: NONE.

## 6. Database and migration effects

Migration added: NO. No schema, data or runtime state is affected.

## 7. API and compatibility effects

GraphQL/API changes: none.
Generated schema/client updates: none.
Backwards compatibility: unaffected; no runtime surface changed.
Deprecations: none.
Cross-repository dependencies: none. The decision describes a shared control
model that other repositories may adopt, but this change modifies only
`thoth-pub/thoth`.

## 8. Authorization and security

Authorization paths changed: none.
Roles/scopes involved: none in code. The document set retains CTO merge
authorization for HIGH and CRITICAL risk and for any explicitly gated merge, and
retains the prohibition on an implementing agent approving or merging its own
work.
Negative authorization tests: not applicable.
Secret or personal-data handling: none. No credentials, secrets, tokens or
personal data were introduced or accessed.
Security limitations: none identified. The change does not alter any
authorization boundary.

## 9. Tests and checks

Unit, integration/database, lint and formatting suites are not applicable: the
change touches Markdown control documents only and no Rust workspace member,
migration, workflow or configuration file.

### Whitespace and diff hygiene

Command:

```text
git diff --cached --check
```

Result:

```text
no output; exit status 0
```

### Changed-path verification

Command:

```text
git diff --cached --stat
```

Result:

```text
12 files changed; all under docs/engineering/ or CHANGELOG.md;
no runtime, schema, migration, workflow, settings or app path
```

### Relative link resolution

Command:

```text
python3 link check over the changed Markdown files, resolving every
relative (non-http) Markdown link target against the repository
```

Result:

```text
broken: 0
```

## 10. Manual verification

Environment: local checkout of `feature/shared-control/ctrl-merge-01`.

Steps and observed results:

- verified `origin/develop` equalled `d534ff52b8bda966ee5cfa7a1d03353ef59476ec`
  before branching, before committing and before pushing - unchanged each time;
- verified issue #786 open and materially matching the authorized scope;
- searched branches, open and closed pull requests, task specifications, ADRs and
  control records for `CTRL-MERGE-01`, terminal merge evidence, non-recursive
  closeout and approval-state-only commits - no equivalent implementation exists;
- verified `ADR-0005` unused;
- read the diff in full and confirmed no wording permits implementer
  self-approval, weakens HIGH-risk merge control, implies merge equals production
  activation, or introduces transient status text that merge would falsify;
- confirmed no programme-specific architecture, inventory, evidence claim or
  rollout decision changed.

Evidence reference: the pull-request diff and the immutable exact-head evidence
comment on the `CTRL-MERGE-01` pull request.

## 11. CI

CI status: see the GitHub checks on the pull request's exact head. This change
classifies as documentation-only under the CI gating added by PR
[#771](https://github.com/thoth-pub/thoth/pull/771), so the heavy Rust,
migration and Docker jobs are expected to be skipped while the protected check
contexts still report.

Only automatically triggered CI was observed. No workflow was manually
dispatched or rerun.

Failures or warnings: recorded on the pull request.

## 12. Rollout and rollback

Initial state after merge: the terminal merge evidence rule becomes authoritative
for tasks starting after the merge. Tasks already in flight may complete under
either form.
Activation required: none; there is no runtime effect.
Feature flag/configuration: not applicable.
Migration sequence: not applicable.
Rollback/disable procedure: revert the documentation pull request; the previous
convention returns immediately with no runtime or data consequence.
Monitoring required: none.

## 13. Known limitations and deferred work

- Lifecycle auditing now depends on GitHub availability and retention. The
  repository's authority order already relied on GitHub review threads and CI, so
  this extends an existing dependency rather than creating one.
- The decision is written for `thoth-pub/thoth`. Other repositories adopting the
  shared control model need their own bounded change.
- Option C in ADR-0005 (mechanically generated evidence artefacts) is rejected
  without prejudice; it would require workflow changes that are out of scope.

## 14. Unresolved issues

Two items of pre-existing control debt were observed and deliberately **not**
fixed, since issue #786 does not extend to them and the task's non-goals
prohibit opportunistic repair:

1. `docs/engineering/ai-delivery/operating-model.md` and
   `branching-and-release-workflow.md` both carry the header line
   `Status: Proposed until merged and approved`, although both merged through PR
   [#764](https://github.com/thoth-pub/thoth/pull/764). This is exactly the stale
   transient-status pattern ADR-0005 addresses, but correcting it changes the
   approval semantics of the whole operating model and belongs in its own bounded
   task.
2. `decision-register.md` row `ADR-0003` still reads "Becomes
   repository-authoritative on merge into `develop`" although PR
   [#778](https://github.com/thoth-pub/thoth/pull/778) merged. This is the
   previously identified `ADR-0003` / PR #778 stale wording that the task
   explicitly excludes from scope.

Both are recorded here as control debt for a separate CTO-authorized task.

## 15. Agent self-assessment

The agent may identify risks but may not approve the task. This implementation is
not self-approved and has not been merged.

Suggested review focus:

- whether ADR-0005 section 4.1 genuinely removes the recursion rather than
  renaming it;
- whether any retained control in ADR-0005 section 10 was weakened in the
  operating-model, release-gates or template edits - particularly exact-head
  binding, implementer/reviewer separation and HIGH-risk merge authorization;
- whether the durable/transient rule is workable in practice, and whether
  narrowing the task-specification `Status` field loses information that some
  programme actually relies on;
- whether any new ambiguity was introduced around production activation, which
  must remain separate from merge;
- whether the section 6 carve-out for ADR and document status vocabulary is
  clear enough to prevent over-application of the durable/transient rule;
- whether any historical record was improperly rewritten (expected: none).
