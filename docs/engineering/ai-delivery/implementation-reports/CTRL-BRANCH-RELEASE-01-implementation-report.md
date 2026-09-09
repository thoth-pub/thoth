# CTRL-BRANCH-RELEASE-01 Implementation Report

## 1. Repository state

Repository: `thoth-pub/thoth`
Task: `CTRL-BRANCH-RELEASE-01`
Decision: `ADR-0011 - Preserve established repository release branch names`
Owning GitHub issue: [#897](https://github.com/thoth-pub/thoth/issues/897)
Programme: Shared Engineering Control
Parent control programme: [`CTRL-DELIVERY-01`](https://github.com/thoth-pub/thoth/issues/818)
Metrics coordination: [#766](https://github.com/thoth-pub/thoth/issues/766)
Immediate dependent task: `BR-SPHINX-01`
Workflow: STANDARD documentation/control reconciliation
Risk: MEDIUM
Base branch: `develop`
PR target: `develop`
Programme integration branch: None
Task branch: `feature/engineering-control/ctrl-branch-release-01`
Expected branch deletion after merge: YES
Final programme PR required: NO
Implementing model: Claude Opus 5, high reasoning

### 1.1 Exact authorized base

```text
4546cb632428872b961ad6c17282984d298e3ade
```

This is the exact `develop` head named in the approved #897 specification, in
Specification Amendment 1, in the independent specification approval, in the CTO
architecture/specification approval, in the durable implementation preflight and
in the CTO implementation authorization. Live `origin/develop` resolved to
exactly this SHA immediately before the task branch was created, so no
authorization rebinding was required and no HOLD condition applied.

The task branch `feature/engineering-control/ctrl-branch-release-01` was created
from exactly this SHA. It did not exist, locally or remotely, before that.

### 1.2 Approval and authorization records

| Gate | Record | Outcome |
|---|---|---|
| Specification Amendment 1 | issue comment `5603738964` | final write budget and maximum implementation action budget |
| Independent specification review | issue comment `5603985920` | `APPROVED` |
| CTO architecture/specification approval | issue comment `5604021690` | `APPROVED - IMPLEMENTATION NOT AUTHORIZED` |
| Implementation preflight | issue comment `5604068628` | `PASS - IMPLEMENTATION STILL NOT AUTHORIZED` |
| CTO implementation authorization | issue comment `5604141046` | `IMPLEMENTATION AUTHORIZED - MERGE / RELEASE / DEPLOYMENT / ACTIVATION NOT AUTHORIZED` |

All five records, plus the complete issue body, were read in full immediately
before any edit and were materially unchanged from the state the authorization
describes. Issue #897 was verified `OPEN`.

Repository controls read before implementation: root `AGENTS.md` (including the
section 5.1 fail-closed namespace preflight and the section 6 granular
action-authorization model), `docs/engineering/AGENTS.md` (including the section
1.1 durable-versus-transient state rule), `operating-model.md`,
`branching-and-release-workflow.md`, `risk-classification.md`, `release-gates.md`,
`implementation-handoff-template.md`, `implementation-report-template.md`,
`repository-map/README.md`, the affected repository-map entries, `ADR-0005`,
`ADR-0009` and the decision register.

### 1.3 Preflight, performed before any edit or branch creation

```bash
git fetch origin --prune
git rev-parse origin/develop
git ls-remote origin 'refs/heads/*engineering-control*'
git ls-remote origin refs/heads/feature/engineering-control/ctrl-branch-release-01
git ls-remote origin refs/heads/feature/engineering-control
git rev-parse --verify refs/heads/feature/engineering-control/ctrl-branch-release-01
git status --short
git ls-tree -r --name-only 4546cb632428872b961ad6c17282984d298e3ade -- docs/engineering/decisions/
gh issue view 897 --repo thoth-pub/thoth --json state
gh pr list --repo thoth-pub/thoth --state all --head feature/engineering-control/ctrl-branch-release-01
```

Observed:

```text
origin/develop                                4546cb632428872b961ad6c17282984d298e3ade
issue #897                                    OPEN
working tree                                  clean
exact task branch, local and remote           absent
flat ref refs/heads/feature/engineering-control  absent
PRs owning that head, any state                  none
docs/engineering/decisions/ADR-0011-*.md      absent at the authorized base
```

### 1.4 Namespace preflight (root `AGENTS.md` section 5.1)

The authorized branch is a governed **descendant** ref
`feature/<area>/<task>`, so the applicable symmetric rule is that no flat parent
ref may occupy its location. Verified against live remote refs:

- `refs/heads/feature/engineering-control` does **not** exist, so the parent
  location is available as a ref namespace;
- the namespace already holds sibling descendants
  `feature/engineering-control/ctrl-branch-namespace-01` and
  `feature/engineering-control/ctrl-ci-clippy-01`, which independently confirms
  that location is a namespace rather than a ref;
- the exact ref `feature/engineering-control/ctrl-branch-release-01` is absent.

Preflight result: `PASS`. No branch was deleted, renamed, moved or otherwise
rearranged, and no namespace workaround was used.

Implementation was performed in an isolated Git worktree outside the repository
tree, so no repository change was made merely to create or ignore a worktree
directory.

## 2. Scope confirmation

Approved specification: issue #897 body plus Specification Amendment 1
(`5603738964`), independently approved (`5603985920`) and CTO-approved
(`5604021690`); implementation authorized by `5604141046`.

Implemented objective: record the exact approved `ADR-0011` and reconcile the
active shared branch-topology and control surface so that shared doctrine
standardizes the **role** of the release branch, written `<release-branch>`,
rather than a universal `master` spelling, while preserving every repository's
established release/default branch and retaining all independently justified
readiness work.

Out-of-scope changes made: NONE.

## 3. Commits

Bounded commits carry the documentation/control implementation and any review
corrections. The exact commit list and the final branch head are the GitHub
pull-request and branch record, which is authoritative for that fast-changing
state. This report does not restate them, and no later commit is added merely to
copy their SHAs here, per `ADR-0005` and `docs/engineering/AGENTS.md`
section 1.1.

## 4. Files changed

Authorized write paths (existing files), from the implementation authorization:

- `CHANGELOG.md`
- `docs/engineering/decisions/decision-register.md`
- `docs/engineering/repository-map/branch-topology.md`
- `docs/engineering/repository-map/control-gaps.md`
- `docs/engineering/repository-map/repositories/thoth-app.md`
- `docs/engineering/repository-map/repositories/thoth-dissemination.md`
- `docs/engineering/repository-map/repositories/thoth-sphinx.md`
- `docs/engineering/repository-map/repositories/thoth-pyramid.md`
- `docs/engineering/repository-map/repositories/thoth-strapi.md`
- `docs/engineering/repository-map/repositories/metrics-dashboard.md`
- `docs/engineering/repository-map/repositories/metrics-widget.md`
- `docs/engineering/repository-map/repositories/cc-license.md`
- `docs/metrics/task-status.md`

Authorized new-file paths:

- `docs/engineering/decisions/ADR-0011-preserve-established-release-branch-names.md`
- `docs/engineering/ai-delivery/implementation-reports/CTRL-BRANCH-RELEASE-01-implementation-report.md`

Actual files changed:

- `docs/engineering/repository-map/branch-topology.md`
  - reason: the shared target policy was the source of the universal `master`
    target. Section 1 now expresses the normal and programme-integration flows
    with `<development-branch>` and `<release-branch>` roles, states the
    `ADR-0011` preservation rules, records that development-branch policy is
    independent, and resolves the roles concretely for `thoth`, the standalone
    `thoth-client` and `baboon`. Section 3's observed-state table gains an
    explicit `<release-branch>` column recording each repository's preserved
    established branch, and its final column now records only independently
    justified remaining readiness work. Section 4's control rule requires each
    specification to record the repository's verified development and release
    branches from live state. Section 5 is retitled to branch **readiness** and
    each task's release-branch conversion is removed while its protection,
    development-branch, CI, provider and publication work is retained, with each
    risk restated from the remaining effects.
  - behavioural effect: none. Documentation and control only.
  - within authorized write budget: YES
- `docs/engineering/decisions/decision-register.md`
  - reason: register `ADR-0011` once in the ADR table using the existing row
    convention, add the corresponding narrative to the approval sequence, and
    refresh `Last updated`.
  - behavioural effect: none.
  - within authorized write budget: YES
- `docs/engineering/repository-map/control-gaps.md`
  - reason: reconcile CG-03 and CG-04. CG-03 records that `main` is preserved for
    Sphinx and that BR-SPHINX-01 no longer creates `master`, while stating
    explicitly that this removes a rename and not a gap, and that CG-03 remains
    **OPEN** for protection, `develop` alignment, control reconciliation and
    bootstrap. CG-04 is marked **OPEN**, gains the `<release-branch>` role
    statement, and enumerates exactly what remains open and unchanged, including
    the retained HIGH risks. CG-05's non-closure list and the CG-11 Pyramid note
    are reworded from "normalization" to "readiness" without changing their
    substance.
  - behavioural effect: none. No gap is closed by this change.
  - within authorized write budget: YES
- `docs/engineering/repository-map/repositories/thoth-app.md`
  - reason: record `main` as the preserved established `<release-branch>` and
    `develop` as the target development branch; restate BR-APP-01 as
    development-branch, protection, CI and Vercel-verification readiness; record
    that Vercel production remains `main`.
  - behavioural effect: none.
  - within authorized write budget: YES
- `docs/engineering/repository-map/repositories/thoth-dissemination.md`
  - reason: replace `Target release branch: master` with the preserved `main`,
    and restate BR-DIS-01 as protection and release/production external-write
    verification, retaining its HIGH risk.
  - behavioural effect: none.
  - within authorized write budget: YES
- `docs/engineering/repository-map/repositories/thoth-sphinx.md`
  - reason: record `main` as the preserved established `<release-branch>`;
    remove the `create master` / `make master the release/default branch`
    requirements from the BR-SPHINX-01 list and replace them with preserving
    `main` and protecting `main` and `develop`; correct the resulting flow's
    terminal branch to `main`; state that BR-SPHINX-01 must be re-specified
    against `ADR-0011`. The verified `main`/`develop` divergence evidence, SHAs
    and bootstrap-only conclusion are unchanged.
  - behavioural effect: none. No `thoth-sphinx` mutation is performed or
    authorized.
  - within authorized write budget: YES
- `docs/engineering/repository-map/repositories/thoth-pyramid.md`
  - reason: replace the conditional `develop -> master` normalization statement
    with the preserved `main` `<release-branch>` and a note that
    development-branch normalization is a separate question.
  - behavioural effect: none.
  - within authorized write budget: YES
- `docs/engineering/repository-map/repositories/thoth-strapi.md`
  - reason: as above, retaining the requirement that any future readiness work
    account for the repository's publication-capable pull-request workflow.
  - behavioural effect: none.
  - within authorized write budget: YES
- `docs/engineering/repository-map/repositories/metrics-dashboard.md`
  - reason: record the Vercel-backed `main` as preserved, state explicitly that
    Vercel production is not moved for branch spelling, restate BR-DASH-01 as
    development-history reconciliation plus protections and any Vercel preview
    configuration the development-branch change actually requires, and retain its
    HIGH risk with the rollback requirement.
  - behavioural effect: none. No provider access or change was performed.
  - within authorized write budget: YES
- `docs/engineering/repository-map/repositories/metrics-widget.md`
  - reason: record `main` as preserved and restate BR-WIDGET-01 as
    development-branch, CI-filter and npm release-protection readiness, retaining
    its HIGH risk because the release path publishes a public package.
  - behavioural effect: none.
  - within authorized write budget: YES
- `docs/engineering/repository-map/repositories/cc-license.md`
  - reason: record `main` as preserved and restate BR-LIC-01 as CI-filter,
    protection and crate-publication readiness, pointing at the still-unverified
    publication and rollback/yank procedure already recorded under "Release gap".
  - behavioural effect: none.
  - within authorized write budget: YES
- `docs/metrics/task-status.md`
  - reason: reconcile the active tracker. The BR-DASH-01 row no longer requires
    normalization to `develop -> master`; the BR-WIDGET-01 and BR-APP-01 rows
    record the preserved `main` and the separate development-branch move; a
    `CTRL-BRANCH-RELEASE-01` row and an explicit `BR-SPHINX-01` row are added;
    section 4 records the `ADR-0011` preservation rule; and section 5 item 6
    records the Sphinx lane order. Every task status, risk and work-package entry
    gate is unchanged.
  - behavioural effect: none.
  - within authorized write budget: YES
- `CHANGELOG.md`
  - reason: required `## [Unreleased]` entry under the existing `### Added`
    heading, following the established format.
  - behavioural effect: none.
  - within authorized write budget: YES

Actual new files created:

- `docs/engineering/decisions/ADR-0011-preserve-established-release-branch-names.md` - within authorized new-file list: YES
- `docs/engineering/ai-delivery/implementation-reports/CTRL-BRANCH-RELEASE-01-implementation-report.md` - within authorized new-file list: YES

Files deleted, moved or renamed: NONE.

### 4.1 Write-budget compliance

PASS.

Every changed path is one of the thirteen authorized existing files or one of the
two authorized new files. The complete changed set is `CHANGELOG.md` plus paths
beneath `docs/`. No file outside the budget was modified, and in particular
`AGENTS.md`, `docs/engineering/ai-delivery/branching-and-release-workflow.md`,
`docs/engineering/README.md`,
`docs/engineering/agent-instructions/rollout-plan.md` and every path beneath
`.github/` are unchanged.

## 4.2 Authorized actions actually used

- repository inspection: USED
- source edit (within budget): USED
- new file creation (two authorized paths): USED
- file deletion/move/rename: NOT USED (not authorized)
- branch creation (exact authorized branch from exact authorized base): USED
- commit: USED
- push: USED
- PR creation/update (one draft PR targeting `develop`): USED
- issue/comment mutation: NOT USED (not authorized)
- manual CI dispatch/rerun: NOT USED (not authorized)
- provider/runtime read: NOT USED (not authorized)
- provider/runtime write: NOT USED (not authorized)
- migration execution: NOT USED (not authorized)
- release/tag/publication: NOT USED (not authorized)
- merge: NOT USED (not authorized)
- deployment: NOT USED (not authorized)
- production activation: NOT USED (not authorized)
- other: an isolated Git worktree was used as the working directory for the
  authorized branch. It creates no repository content and is not committed.

Unauthorized actions performed: NONE.

## 4.3 Automatic and manual external effects

The complete changed-file set was classified with the repository's own
classifier before the pull request was opened, and returned the expected
documentation-only result recorded in section 9. On that classification the
PR-triggered container workflow's GHCR write path
(`build_and_push_staging_docker_image`) is skipped, so opening the draft pull
request is expected to perform no external write.

Manually initiated external actions: NONE. No workflow was dispatched, rerun or
cancelled.

External writes/publication (releases, tags, packages, registries, third-party
services): NONE.

Provider access: NONE. No Vercel, npm, crates.io, GHCR, AWS or other provider
was read or written. The statements in this change about Vercel production
branches and publication paths are reconciliations of existing recorded
repository-map evidence, not fresh provider reads.

## 5. Implementation decisions

Decisions made within the approved design:

1. `branch-topology.md` section 3's table gained an explicit `<release-branch>`
   column rather than encoding the preserved branch inside prose, so that the
   observed-state and target-policy records are consistent per row and a reader
   cannot infer a spelling by analogy.
2. The final table column was renamed from "Target-policy state" to "Remaining
   readiness work" and populated with the independently justified work only.
   Leaving it as "normalization required" would have been false once the rename
   is removed; blanking it would have falsely closed real gaps.
3. Section 5 was retitled "Required branch-readiness tasks" and each task's risk
   line was rewritten to state *why* the risk survives the rename's removal.
   `BR-APP-01`, `BR-DIS-01`, `BR-DASH-01` and `BR-WIDGET-01` remain HIGH;
   `BR-SPHINX-01` and `BR-LIC-01` remain MEDIUM. No risk was lowered.
4. CG-03 and CG-04 explicitly state that removing a rename removes a rename and
   not a gap, and CG-04 is marked **OPEN**, to prevent a later reader treating
   this reconciliation as gap closure.
5. `thoth`'s own flow is restated concretely in `branch-topology.md` section 1
   after the role definitions, so that out-of-budget documents which correctly
   reference `develop -> master` for `thoth`, the standalone `thoth-client` and
   `baboon` remain resolvable and truthful.
6. The `thoth-sphinx` "resulting flow" code block terminates at `main` rather
   than `master`, since `main` is that repository's preserved release branch.
7. `docs/metrics/task-status.md` gained an explicit `BR-SPHINX-01` row. The
   tracker previously named the task only as a dependency, and the dependency
   order `CTRL-BRANCH-RELEASE-01 -> BR-SPHINX-01 -> SPHINX-BOOT-01 -> MET-WP6-01`
   is recorded in section 5 item 6 without altering any work-package gate.
8. The ADR uses the established `ADR-0009` record shape, including the authority
   condition and the `ADR-0005` note, so that `Status: APPROVED` cannot be
   misread as repository authority.

Deviations from the specification requiring authorization: NONE.

## 6. Database and migration effects

Migration added: NO.

Database, schema, `thoth-api/src/schema.rs`, domain-model and data effects:
NONE. No migration file was added, changed or executed.

## 7. API and compatibility effects

GraphQL/API changes: NONE.
Generated schema/client updates: NONE.
Backwards compatibility: unaffected; no runtime contract is touched.
Deprecations: NONE.

Cross-repository contract impact: the database and domain model, GraphQL/API
schema and behaviour, generated clients and types, authorization semantics,
export formats, configuration and environment contracts, event and job payloads,
dissemination and platform behaviour, UI assumptions, CMS and site contracts,
package and library interfaces, and deployment and compatibility windows are all
**NOT AFFECTED**.

Cross-repository governance impact realized: shared branch-governance semantics
change for every managed repository — future branch-readiness tasks preserve each
repository's established `main` or `master` release branch instead of
normalizing the spelling. That governance effect is implemented only in
`thoth-pub/thoth`. No downstream repository source, branch, setting or provider
state is mutated by this task, and repository-local controls that still encode
the old universal `master` target are reconciled only by their own separately
scoped and separately authorized owning tasks.

## 8. Authorization and security

Authorization paths changed: NONE. `thoth-api/src/policy.rs` and every runtime
authorization surface are untouched.
Roles/scopes involved: none.
Negative authorization tests: not applicable.
Secret or personal-data handling: none. No secret, token or credential was read,
written or logged.
Security limitations: none introduced.

## 9. Tests and checks

This is a documentation/control-only change. The repository's documentation
evidence rule in root `AGENTS.md` section 8 and `docs/engineering/AGENTS.md`
section 6 applies; the Rust workspace gate is not applicable and was not run.

### Whitespace and diff integrity

Command:

```bash
git diff --check
```

Result:

```text
no output; exit status 0
```

### Changed-path enumeration

Command:

```bash
git status --short
git diff --stat
```

Result:

```text
13 modified files, all within the authorized existing-file budget;
2 new files, both within the authorized new-file budget;
0 files deleted, moved or renamed.
```

### Universal-`master` falsification search

Commands:

```bash
grep -n 'develop -> master' docs/engineering/repository-map/branch-topology.md
git diff -U0 | grep '^+' | grep 'master'
grep -n 'master' docs/engineering/repository-map/repositories/thoth-sphinx.md
grep -n 'master' docs/metrics/task-status.md
```

Result:

```text
Every surviving or added `master` occurrence in the changed active control
surface is one of:
  (a) a verified repository-local fact for `thoth`, the standalone
      `thoth-client` or `baboon`, all established on `master`;
  (b) an explicit prohibition ("do not create `master`", "no longer creates
      `master`", "must not be created ... merely because");
  (c) explicit contextual discussion of the rejected Option A inside ADR-0011.
No changed active control requires creating `master` in, or switching the
default branch of, a repository established on `main`, and none requires
converting a `master` repository to `main`.
```

### ADR uniqueness and registration

Commands:

```bash
ls docs/engineering/decisions/ | grep -c 'ADR-0011'
grep -c '| `ADR-0011` |' docs/engineering/decisions/decision-register.md
```

Result:

```text
1
1
```

### ADR-0009 non-regression

Command:

```bash
git diff --stat -- docs/engineering/decisions/ADR-0009-programme-integration-branch-namespace.md
```

Result:

```text
no output; ADR-0009 is byte-identical to the authorized base
```

### Relative-link resolution

Every relative Markdown link in the changed and new files was resolved against
the working tree.

Result:

```text
all relative links resolve
```

### Excluded-path verification

Command:

```bash
git status --porcelain | awk '{print $2}' | grep -E '^(\.github|thoth-|Cargo|migrations)'
```

Result:

```text
no matches; no workflow, runtime source, migration, dependency or generated
contract file changed
```

### CI classifier over the complete changed-path set

Command and verbatim output are recorded in section 11.

## 10. Manual verification

Environment: local isolated Git worktree of `thoth-pub/thoth` at the authorized
base, plus read-only GitHub API inspection.

Steps: read the complete owning issue and all five durable authority comments;
read root and applicable nested `AGENTS.md`, the ai-delivery control documents
and the repository-map entries; verify base, issue state, branch and namespace
availability, ADR-path availability and worktree cleanliness; read every
write-budget file before editing it; apply the bounded changes; run the checks in
section 9; classify the complete changed set; commit, push and open one draft
pull request.

Observed result: preflight passed on every condition; the change is confined to
the authorized budget; the classifier returns the expected documentation-only
result.

## 11. CI

The classifier was run against the complete changed-file set before the pull
request was opened. Its command and verbatim output, and the observed natural CI
state after the draft PR was opened, are recorded in the task's final
implementation handoff and in the GitHub pull-request record, which is
authoritative for live CI state. No CI result is asserted by this committed
report.

No manual CI dispatch, rerun or cancellation was performed, and none is
authorized. The PR-triggered container workflow contains an external GHCR write
path; on the documentation-only classification that write-capable job is
expected to be skipped, and no external write is an intended effect of this
task.

## 12. Rollout and rollback

Initial state after merge: `ADR-0011` becomes repository-authoritative and the
reconciled shared branch-topology and control surface takes effect for tasks
starting afterwards. No branch, repository setting, protection, CI
configuration, provider route or deployment changes.

Activation required: none. There is no runtime behaviour to activate.

Feature flag/configuration: not applicable.

Migration sequence: not applicable.

Rollback: before downstream tasks depend on `ADR-0011`, a normal revert of this
bounded documentation and control pull request. Once repository-local tasks rely
on it, a superseding ADR with impact analysis is required; the architecture must
not be silently reverted. No branch, provider or runtime rollback exists,
because no such mutation was performed.

Monitoring required: none.

## 13. Known limitations and deferred work

- This task reconciles the active shared control surface only. Repository-local
  controls inside `thoth-app`, `thoth-dissemination`, `thoth-sphinx`,
  `metrics-dashboard`, `metrics-widget`, `cc-license`, `thoth-pyramid` and
  `thoth-strapi` that still encode the old universal `master` target are
  reconciled by their own separately scoped and separately authorized owning
  tasks. No implementing agent for this task had write access to any of them.
- `docs/engineering/ai-delivery/release-gates.md` section 10 states "The
  production release path is `develop -> master`". It is outside the write budget
  and was left unchanged. It does not contradict `ADR-0011`: that document
  requires branching and releases to follow `branching-and-release-workflow.md`,
  which is explicitly scoped to `thoth-pub/thoth`, and `thoth`'s release path
  *is* `develop -> master` and is preserved. It is recorded here as a reviewer
  observation, not as an unresolved contradiction. The same applies to the
  `thoth`-scoped statements in `AGENTS.md`,
  `branching-and-release-workflow.md`, `docs/engineering/ai-delivery/README.md`
  and `task-specification-template.md`, each of which already requires
  repository-local verification for any other repository.
- `repositories/thoth-client.md` and `repositories/baboon.md` are outside the
  write budget and describe repositories established on `master`, so their
  "conforms to the `develop -> master` pattern" wording remains accurate; the
  pattern they reference is restated concretely in `branch-topology.md`
  section 1.
- Removing release-branch conversion from the `BR-` tasks does not close CG-03,
  CG-04 or CG-11, and does not lower any task's risk. Each affected task still
  requires its own re-specification, independent review and authorization.
- No branch-protection, ruleset, default-branch or provider state was inspected
  live for this task; provider and settings reads are not authorized. The
  repository-map statements reconciled here rest on the previously recorded
  verified evidence and its recorded evidence dates.

## 14. Unresolved issues

NONE.

## 15. Agent self-assessment

The implementing agent has not approved this work and may not. Independent
review must bind to the exact final head SHA.

Suggested review focus:

1. `ADR-0011` decision text against the exact approved decision in #897 and
   Specification Amendment 1 `5603738964`, clause by clause, including the
   fifteen numbered decisions and the risk-reassessment rule.
2. `branch-topology.md` sections 1, 3, 4 and 5 for internal consistency: that the
   role definitions, the per-row `<release-branch>` values and the per-task
   readiness lists agree, and that no row silently closes a gap.
3. That every remaining `master` mention in the changed active surface is a
   verified repository-local fact, an explicit prohibition, or contextual
   discussion of the rejected option.
4. CG-03 and CG-04 for false gap closure and for retained HIGH risks.
5. `docs/metrics/task-status.md` for unchanged work-package entry gates and for
   the preserved dependency order
   `CTRL-BRANCH-RELEASE-01 -> BR-SPHINX-01 -> SPHINX-BOOT-01 -> MET-WP6-01`.
6. `ADR-0009` non-regression and the absence of any historical-evidence rewrite.
7. Write-budget and action-authorization compliance against `5604141046`.
8. The classifier result and the natural CI state at the exact final head,
   specifically that the GHCR write-capable job remained skipped.

## 16. Remaining gates

At the point this report is committed, GitHub remains authoritative for
pull-request, CI and merge state, and none is asserted here.

1. The implementing agent has **not** approved its own source, and may not.
2. Independent exact-head source review is required. It binds to the exact
   implementation head SHA, and any later source commit invalidates it.
3. Separate CTO merge authorization at the exact reviewed head is required.
   **Merge is not authorized by this implementation.**
4. Merge into `develop`. `ADR-0011` becomes repository-authoritative only once
   the approved, reviewed content is reachable from `develop`; `Status: APPROVED`
   on this branch confers no repository authority.
5. `BR-SPHINX-01` must then be re-specified against the authoritative `ADR-0011`,
   with its own independent review and implementation authorization.
6. `SPHINX-BOOT-01` follows `BR-SPHINX-01`; `MET-WP6-01` follows that. None is
   authorized, begun or unblocked by this task.
7. Merging this control task creates no branch, changes no repository setting,
   protection, default branch, CI configuration or provider state, authorizes no
   repository-local `BR-` implementation, and activates no production behaviour.
