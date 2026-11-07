# CTRL-BRANCH-NAMESPACE-01 Implementation Report

## 1. Repository state

Repository: `thoth-pub/thoth`
Task: `CTRL-BRANCH-NAMESPACE-01`
Task issue: [#837](https://github.com/thoth-pub/thoth/issues/837)
Programme: Shared Engineering Control
Blocked consumer: `MET-WP1-01` (issue [#836](https://github.com/thoth-pub/thoth/issues/836)), which remains **HOLD**
Related programme issue: [#766](https://github.com/thoth-pub/thoth/issues/766)
Workflow: STANDARD documentation/control correction
Risk: MEDIUM
Base branch: `develop`
PR target: `develop`
Programme integration branch: None
Task branch: `feature/engineering-control/ctrl-branch-namespace-01`
Expected branch deletion after merge: YES
Final programme PR required: NO
Implementing model: Claude Opus 5

### 1.1 Exact authorized base

```text
e555b25217b0cdaeae40aa7b84ea6c15363a8282
```

This is the exact `develop` head named in the amended #837 specification, in the
independent reviews, in the CTO decision approval and in the bounded
implementation authorization. The task branch
`feature/engineering-control/ctrl-branch-namespace-01` already existed at exactly
this SHA when implementation began.

### 1.2 Approval and authorization records

| Gate | Record | Outcome |
|---|---|---|
| Independent review, round 1 | issue comment `5426849829` | `CHANGES REQUIRED` |
| Fresh independent review of amended specification | issue comment `5426855358` | `APPROVED` |
| CTO ADR-0009 decision approval | issue comment `5426997804` | `DECISION APPROVED - IMPLEMENTATION NOT AUTHORIZED` |
| Bounded implementation authorization | issue comment `5427107258` | `IMPLEMENTATION AUTHORIZED - MERGE / #836 RESUME NOT AUTHORIZED` |

All four records were re-read in full immediately before any edit and were
materially unchanged from the state the authorization describes.

### 1.3 Preflight, performed before any edit

```bash
git fetch origin --prune
git rev-parse origin/develop
git rev-parse origin/feature/engineering-control/ctrl-branch-namespace-01
git status --short
gh api --paginate repos/thoth-pub/thoth/branches --jq '.[].name'
gh api repos/thoth-pub/thoth/rulesets
gh api repos/thoth-pub/thoth/branches/develop/protection
```

Observed:

```text
origin/develop                                            e555b25217b0cdaeae40aa7b84ea6c15363a8282
origin/feature/engineering-control/ctrl-branch-namespace-01  e555b25217b0cdaeae40aa7b84ea6c15363a8282
working tree                                              clean
live branches                                             53
```

Implementation was performed in an isolated Git worktree outside the repository
tree, so no repository change was made merely to create or ignore a worktree
directory.

### 1.4 Namespace findings

- No live branch and no live tag in `thoth-pub/thoth` contains the reserved token
  `--`. The convention is therefore adopted with no migration and no conflict.
- No `feature/metrics/*` ref exists. The flat ref `refs/heads/feature/metrics`
  occupies that path at `e555b25217b0cdaeae40aa7b84ea6c15363a8282`, which is why
  the #836 child-branch creation returned HTTP 422 `Reference update failed`.
- A prefix scan over all 53 live branches found **no** case where one branch ref
  is a path prefix of another. Live state is consistent with the decision.
- No flat ref `feature/engineering-control` exists, so this task's own
  descendant task branch is namespace-valid.
- The live `feature/metrics` and `feature/metrics-control/...` pair confirms the
  rejected Option B collision class as a real, present condition rather than a
  hypothetical one.

### 1.5 Cross-repository doctrine findings

The repository map identifies the governed repositories; ownership was taken
from that map rather than inferred from repository names. Each mapped repository
plus the org-level `.github` and `infrastructure` repositories was inspected
**directly** via its live default-branch tree and file contents, not only through
indexed code search.

Repositories inspected directly: `baboon`, `cc-license`, `metrics-dashboard`,
`metrics-widget`, `thoth-app`, `thoth-client`, `thoth-dissemination`,
`thoth-pyramid`, `thoth-sphinx`, `thoth-strapi`, `.github`, `infrastructure`.

One hit was found outside `thoth-pub/thoth`:

```text
thoth-pub/thoth-dissemination :: AGENTS.md:104
normal task branches:   feature/<area>/<task-id> -> develop
```

That is the `STANDARD` form, which this decision **retains** unchanged. That
repository documents no programme-integration slice grammar and runs no
programme integration branch. **No other repository requires mutation, and none
was mutated.**

A supporting org-wide indexed search for `feature/<programme>`, `feature/metrics/`,
`programme-or-area` and `PROGRAMME_INTEGRATION` returned hits in
`thoth-pub/thoth` only.

## 2. Decision implemented

The exact CTO-approved ADR-0009 decision, unchanged:

```text
programme integration:       feature/<programme>
PROGRAMME_INTEGRATION slice: feature/<programme>--<slice>
STANDARD task:               feature/<area>/<task>
```

- The programme slice is a **sibling** of the integration branch, not a
  descendant of it.
- `--` is the reserved, deterministic programme/slice separator.
- Governed `<programme>`, `<area>`, `<slice>` and `<task>` identifiers must be
  non-empty, must each be a single Git path segment, and must not themselves
  contain `--`. No broader leading- or trailing-hyphen prohibition was
  introduced, because none was approved.
- Slice PRs continue to target `feature/<programme>`; final programme
  integration continues `feature/<programme> -> develop`.
- A symmetric fail-closed namespace preflight applies before any new governed
  flat ref and before any new governed descendant ref.
- ADR-0009 mechanically records all five approved alternatives A–E with their
  approved dispositions.

For Metrics the approved spellings are `feature/metrics` (integration) and
`feature/metrics--wp1-registry-foundation` (the future WP1 replacement slice).
**No Metrics branch was created by this task**, and none is authorized by it.

The approved Metrics design invariant is preserved: each affected repository owns
its own `feature/metrics` integration branch, focused children are created from
it, those children merge back into it, and they do not target `develop`
directly. ADR-0009 standardizes the repository ref spelling of programme slices
and does **not** amend the substantive Metrics architecture. The private Drive
design was not edited. #836 was not modified.

### 2.1 ADR-0009 status

`Status: APPROVED`, approved by Javi, CTO, on 2026-08-26, decision owner CTO.

`APPROVED` is the durable decision state. It is **not** a claim of repository
authority. ADR-0009's shared doctrine becomes repository-authoritative only after
this exact approved implementation receives independent exact-head source review
and is merged into `develop`. A branch carrying `APPROVED` is not
repository-authoritative. That distinction is stated explicitly in the ADR's
authority condition, in its approval section and in the decision register row.
No merge or repository authority is claimed by this implementation.

## 3. Files changed

Twelve paths, exactly the authorized budget. No file was deleted, moved or
renamed.

### 3.1 New files

| Path | Change |
|---|---|
| `docs/engineering/decisions/ADR-0009-programme-integration-branch-namespace.md` | New ADR carrying the exact approved decision: context and defect evidence, seven decision drivers, options A–E with dispositions, the branch grammar, governed-identifier rules, PR targeting, the symmetric fail-closed preflight, compatibility/migration, explicit exclusions, rollout/rollback, consequences, nine invariants, implementation impact, validation evidence and the approval record. |
| `docs/engineering/ai-delivery/implementation-reports/CTRL-BRANCH-NAMESPACE-01-implementation-report.md` | This report. |

### 3.2 Modified files

| Path | Change |
|---|---|
| `AGENTS.md` | Section 5 now distinguishes `STANDARD` `feature/<area>/<task>` from `PROGRAMME_INTEGRATION` `feature/<programme>` plus sibling `feature/<programme>--<slice>`, states why the descendant form is impossible, states the reserved-token and identifier rules, and adds new section 5.1 carrying the symmetric fail-closed namespace preflight and the no-workaround rule. Unrelated repository doctrine untouched. |
| `docs/engineering/ai-delivery/README.md` | Branching summary now states the `STANDARD` area form and the `PROGRAMME_INTEGRATION` sibling slice form with correct PR targets, and links ADR-0009. |
| `docs/engineering/ai-delivery/branching-and-release-workflow.md` | Section 2 no longer conflates programme and area: `STANDARD` naming is `feature/<area>/<task>`, with the invalid `feature/metrics/record-schema` example replaced by the live valid `feature/metrics-control/met-ctrl-01`, and a note that the descendant form is unavailable beneath a flat parent ref. Section 3 slice naming and examples corrected to the sibling form. Section 5 Metrics flow corrected to `feature/metrics--<slice>` while preserving child-from/child-to-`feature/metrics` and no direct `develop` targeting. Section 10 adds three prohibited patterns: the descendant slice form beneath a live integration branch, single-`-` separation or `--` inside a governed identifier, and resolving a collision by deleting/renaming/moving a branch. |
| `docs/engineering/ai-delivery/task-specification-template.md` | The generic `feature/[programme-or-area]/[task-id-or-short-name]` field is replaced by a workflow-gated field offering exactly the two approved forms, with an explicit prohibition on emitting `feature/<programme>/<slice>` under `PROGRAMME_INTEGRATION`, the reserved-token and identifier rules, and a preflight requirement. A future specification cannot mistakenly regenerate the descendant form. |
| `docs/engineering/ai-delivery/implementation-handoff-template.md` | The exact authorized branch field becomes `Authorized task/slice branch` with both workflow-specific forms and the approved grammar, plus a new preflight step 6 for the symmetric namespace check; the remaining preflight steps were renumbered. No automation or executable parsing was introduced. |
| `docs/engineering/repository-map/branch-topology.md` | Section 1 target programme-integration flow corrected to the sibling form with the reserved-token and identifier rules; section 2 Metrics policy corrected and given the `feature/metrics--<slice>` spelling with the "ref spelling only" boundary; section 4 control rule now requires recording the workflow-appropriate branch form and running the symmetric preflight; section 6 Metrics readiness adds the namespace-preflight condition. |
| `docs/engineering/repository-map/repositories/thoth-sphinx.md` | The forward-looking Metrics flow stored in this control repository is corrected to `feature/metrics--<slice>`, with base/target semantics preserved and an explicit statement that this is not authorization to mutate `thoth-pub/thoth-sphinx`. |
| `docs/metrics/task-status.md` | Section 4 "Branch strategy" — the active branch-strategy statement — corrected to `feature/metrics--<slice>` with the preserved child-from/child-to invariant, the ADR-0009 spelling boundary and a preflight requirement. Historical task, review, merge and lifecycle evidence in this file is unchanged; no old branch name was rewritten. |
| `docs/engineering/decisions/decision-register.md` | ADR-0009 registered in the existing table convention with status `APPROVED`, its approval and review comment identifiers, the decision content, the rejected forms and their reasons, the explicit statement that it is not yet repository-authoritative, and the #836 HOLD condition. An approval-sequence narrative was added in the existing style. `Last updated` advanced to 2026-08-26. No PR merge is claimed. |
| `CHANGELOG.md` | One entry under the existing `## [Unreleased]` / `### Added` structure. No unrelated entry was rewritten. |

## 4. Deviations

**NONE.** The approved decision was implemented exactly as recorded, with the
approved drafting clarifications only. No path outside the twelve authorized
paths was touched.

## 5. Effects

| Dimension | Effect |
|---|---|
| Migration | None. No database migration added, changed or run. |
| Data / schema | None. No `schema.rs`, model, table or Diesel change. |
| Runtime | None. No Rust, source, API, GraphQL or CLI change. |
| Authorization / security | None. No `policy.rs`, role, permission or credential change. |
| Dependencies | None. `Cargo.toml` and `Cargo.lock` untouched. |
| Workflows / CI configuration | None. No `.github/workflows/**` change and no CI classifier change. |
| Repository settings | None. No branch protection, ruleset or setting change. |
| Branches | None created, deleted, renamed or moved other than committing to the pre-existing authorized task branch. `feature/metrics` untouched. |
| Cross-repository | None. No other repository was read-write accessed or mutated. |
| Provider / runtime state | None accessed. |
| Deployment / release / production | None. |

## 6. Local validation

All commands were run in the isolated worktree at the implementation head.

### 6.1 Path budget

`git status --short` and `git diff --cached --name-status` reported exactly the
twelve authorized paths: eleven `M`, one `A` for the ADR, plus this report. No
`D`, `R` or `C` status appeared, confirming no deletion, rename or copy.

### 6.2 Whitespace

```bash
git diff --check
```

Result: clean, exit `0`.

### 6.3 Ref-format checks

```bash
git check-ref-format refs/heads/feature/metrics
git check-ref-format refs/heads/feature/metrics--wp1-registry-foundation
git check-ref-format refs/heads/feature/example-area/example-task
git check-ref-format refs/heads/feature/metrics--db-foundation
git check-ref-format refs/heads/feature/large-programme--slice-01
git check-ref-format refs/heads/feature/engineering-control/ctrl-branch-namespace-01
```

Result: exit `0` for every ref. The selected grammar is Git-valid.

Negative controls: `refs/heads/feature//task` and `refs/heads/feature/metrics..x`
were rejected with exit `1`, while `refs/heads/feature/metrics--` was **accepted**
by `git check-ref-format`. That last result is the reason ADR-0009 states the
non-empty-identifier rule as doctrine: Git's own ref-format check does not
enforce it.

The Option A impossibility was **explained from the Git ref storage rule and the
recorded #836 HTTP 422 failure**, and reproduced only in a throwaway scratch
repository created and deleted for the purpose. No live ref was deleted, renamed,
moved or rearranged to "test" it. In that scratch repository, with
`feature/metrics` present:

```text
fatal: cannot lock ref 'refs/heads/feature/metrics/wp1-registry-foundation':
'refs/heads/feature/metrics' exists;
cannot create 'refs/heads/feature/metrics/wp1-registry-foundation'
```

while `feature/metrics--wp1-registry-foundation` was created successfully
alongside `feature/metrics`.

### 6.4 Stale doctrine search

A repository-wide search was run for `feature/<programme>/<slice>` and its
brace/bracket variants, `feature/metrics/<x>`, `programme-or-area`, and the
rejected single-hyphen programme-slice grammar. Every remaining occurrence is
classified below; none is active doctrine prescribing a rejected form.

| Classification | Occurrences |
|---|---|
| Approved/current doctrine | `AGENTS.md`, `docs/engineering/ai-delivery/README.md`, `branching-and-release-workflow.md`, `task-specification-template.md`, `implementation-handoff-template.md`, `branch-topology.md`, `repositories/thoth-sphinx.md`, `docs/metrics/task-status.md`, `decision-register.md`, `ADR-0009`, `CHANGELOG.md` — all now describe `feature/<programme>`, `feature/<programme>--<slice>` and `feature/<area>/<task>` |
| Rejected-form explanatory text | The remaining descendant-form and single-hyphen mentions in `AGENTS.md`, the two templates, `branching-and-release-workflow.md`, `README.md`, `branch-topology.md`, `thoth-sphinx.md`, `task-status.md`, `decision-register.md`, `ADR-0009` and `CHANGELOG.md` are explicit statements that those forms are rejected, impossible or prohibited. They are required for the doctrine to be self-explaining. |
| Historical lifecycle evidence | `docs/engineering/ai-delivery/tasks/THOTH-GQL-OPS-01.md:894` references `feature/<programme-or-area>/<short-name>` in a `DRAFT`, `STANDARD`, `Programme integration branch: None` task specification whose actual prescribed branches are all valid `feature/<area>/<task>` form. It is a task specification, not active shared doctrine, and was deliberately **not** rewritten. |
| Repository-local follow-up evidence | None required. `thoth-pub/thoth-dissemination` `AGENTS.md:104` documents only the retained `STANDARD` form and needs no change. |

Three further paths outside the write budget were inspected and confirmed to
need no change, so no HOLD arose:

- `docs/engineering/ai-delivery/release-gates.md` — uses `feature/<programme>`
  and `feature/<programme> -> develop` only, both correct under ADR-0009;
- `docs/engineering/ai-delivery/operating-model.md:167` — creates
  `feature/<programme>` and describes slice base/target semantics without
  prescribing a slice branch **name**;
- `docs/engineering/ai-delivery/implementation-report-template.md:13` — carries
  only the `Workflow: STANDARD | PROGRAMME_INTEGRATION` label field.

### 6.5 Selected convention search

`feature/<programme>--<slice>` and `feature/<area>/<task>` are each present in all
nine active normative surfaces plus `CHANGELOG.md` and `ADR-0009`, with slice PRs
targeting `feature/<programme>` and final programme integration targeting
`develop` throughout.

### 6.6 CI classifier over the complete changed-path set

```bash
python3 .github/scripts/classify_ci_changes.py --paths \
  AGENTS.md \
  CHANGELOG.md \
  docs/engineering/ai-delivery/README.md \
  docs/engineering/ai-delivery/branching-and-release-workflow.md \
  docs/engineering/ai-delivery/task-specification-template.md \
  docs/engineering/ai-delivery/implementation-handoff-template.md \
  docs/engineering/repository-map/branch-topology.md \
  docs/engineering/repository-map/repositories/thoth-sphinx.md \
  docs/engineering/decisions/decision-register.md \
  docs/metrics/task-status.md \
  docs/engineering/decisions/ADR-0009-programme-integration-branch-namespace.md \
  docs/engineering/ai-delivery/implementation-reports/CTRL-BRANCH-NAMESPACE-01-implementation-report.md
```

Output, verbatim:

```json
{"docs_only": "false", "run_build": "false", "run_docker": "true", "run_migrations": "false"}
```

This matches the approved expectation exactly: `docs_only=false`,
`run_build=false`, `run_migrations=false`, `run_docker=true`. Root `AGENTS.md` is
not a documentation path to the classifier, so the change is correctly not
docs-only, and the Docker job will therefore publish the expected staging image
`ghcr.io/thoth-pub/thoth:staging-pr-<PR>`. That external write is explicitly
authorized. The classifier was **not** modified.

### 6.7 Final diff review

`git status --short`, `git diff --stat`, `git diff` and `git diff --check` were
run and inspected. Confirmed: no accidental path; no historical-evidence rewrite
beyond the specifically approved active doctrine; no semantic change outside
ADR-0009; no source, workflow or settings change.

## 7. Exact source state

Base: `e555b25217b0cdaeae40aa7b84ea6c15363a8282`
Branch: `feature/engineering-control/ctrl-branch-namespace-01`

The exact commit and final head SHA carrying this report are the GitHub
pull-request and branch record, which is authoritative for that fast-changing
state. This report does not restate them, and no later commit is added merely to
copy them in, per `ADR-0005`.

## 8. Remaining gates

At the point this report is committed, no pull request, CI result or merge
exists for this task; none is asserted here. GitHub remains authoritative for
pull-request, CI and merge state.

1. The implementing agent has **not** approved its own source, and may not.
2. A fresh independent exact-head source review is required. It binds to the
   exact implementation head SHA.
3. Any source commit after that approval invalidates the review, and a fresh
   exact-head review is then required.
4. Explicit CTO merge authorization at the exact reviewed head is required.
   **Merge is not authorized by this implementation.**
5. ADR-0009 becomes repository-authoritative only after the exact approved
   implementation is independently reviewed at its exact head and merged into
   `develop`. `Status: APPROVED` on this branch does not confer repository
   authority.
6. `MET-WP1-01` [#836](https://github.com/thoth-pub/thoth/issues/836) remains
   **HOLD** and does not resume from this task's merge. It then requires its own
   task-specific amendment changing its authorized child branch to
   `feature/metrics--wp1-registry-foundation`, fresh HOLD-sensitive verification
   of all its original conditions, fresh required independent review and fresh
   bounded implementation authorization.
7. Merging this control task does not create any Metrics branch, does not
   authorize any Metrics implementation, and activates no production behaviour.
