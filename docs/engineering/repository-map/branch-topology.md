# Branch Topology

Status: VERIFIED OBSERVED STATE PLUS APPROVED TARGET POLICY
Evidence date: 2026-07-24 for `thoth`, `thoth-app`, `thoth-dissemination`,
`metrics-dashboard`, `metrics-widget`, `cc-license`; independently re-verified
2026-08-15 for `thoth-sphinx`, and newly added and verified 2026-08-15 for
`thoth-client` (standalone), `thoth-pyramid`, `thoth-strapi`; `baboon` newly
added and verified 2026-08-16. Target policy reconciled 2026-09-09 under
[`ADR-0011`](../decisions/ADR-0011-preserve-established-release-branch-names.md);
no branch was created, renamed, moved or deleted by that reconciliation.
`metrics-dashboard` permanent `dev`/`main` topology reconciled and its branch
heads re-verified 2026-09-21 under `BR-DASH-01C-CONTROL-RECONCILIATION`
([#931](https://github.com/thoth-pub/thoth/issues/931)); no branch was created,
renamed, moved or deleted by that reconciliation either.

## 1. Target repository policy

Normal task flow:

```text
feature/<area>/<task> -> <development-branch> -> <release-branch>
```

Approved programme-integration flow, where the slice branch is a **sibling** of the integration branch:

```text
feature/<programme>--<slice> -> feature/<programme> -> <development-branch> -> <release-branch>
```

`<development-branch>` is the repository's verified active development branch. `<release-branch>` is the repository's verified established release/default branch. Merged task and slice branches are deleted.

Under [`ADR-0011`](../decisions/ADR-0011-preserve-established-release-branch-names.md), shared doctrine standardizes the **role** of the release branch, not its spelling. `<release-branch>` resolves from each repository's own verified live state, not from a global default:

- a repository already established on `main` remains on `main`;
- a repository already established on `master` remains on `master`;
- `main` must not be created, renamed, moved or substituted merely because another repository uses `main`, and `master` must not be created, renamed, moved or substituted merely because another repository uses `master`;
- changing an established release/default branch requires a separately scoped repository-local decision and task, justified by a reason other than naming consistency, which assesses GitHub settings and protections, CI filters, deployment and provider routing, release and publication automation, external references, compatibility and rollback as applicable;
- branch protections, CI, release automation, deployment configuration and publication configuration bind to the repository's verified release branch rather than to a globally assumed `main` or `master`.

Development-branch policy is independent of release-branch naming. A repository-local readiness task may still normalize an active `dev` branch to `develop` where that is separately justified and approved. That normalization is not a mandate: an approved repository-local decision may instead retain `dev` permanently, as `metrics-dashboard` does (section 3.3).

For `thoth-pub/thoth` those roles resolve to `develop` and `master`, so this repository's own flow remains:

```text
feature/<area>/<task> -> develop -> master
```

The standalone `thoth-pub/thoth-client` and `baboon` likewise resolve `<release-branch>` to `master`. The repositories listed in section 3 as established on `main` resolve it to `main`.

An approved programme design decides whether it uses direct task PRs or a programme integration branch.

`--` is the reserved programme/slice separator. `feature/<programme>/<slice>` is not usable beneath a live `feature/<programme>` integration branch, because Git cannot hold a ref and a ref namespace at the same path. Governed `<programme>`, `<area>`, `<slice>` and `<task>` identifiers must each be non-empty, must each be a single Git path segment, and must not themselves contain `--`. See [`ADR-0009`](../decisions/ADR-0009-programme-integration-branch-namespace.md).

## 2. Programme-specific policy

### Publisher Services

The approved design requires one fresh branch and one PR per task. Each task targets the repository's verified development branch. There is no long-lived `feature/publisher-services` branch.

### Thoth Metrics

The Metrics design requires one repository-local `feature/metrics` integration branch per affected repository after branch readiness. Bounded child branches are created from that integration branch and target it, followed by a final repository-local PR to that repository's verified `<development-branch>`; they do not target the `<development-branch>` directly.

```text
<development-branch>
  -> feature/metrics
  -> feature/metrics--<slice>
  -> feature/metrics
  -> <development-branch>
```

`<development-branch>` resolves from each affected repository's own verified,
approved development branch in section 3, not from `thoth`'s spelling or from
shared naming convention. For `thoth` it is `develop`; for `metrics-dashboard`
it is `dev`, that repository's approved permanent development branch. This is
the resolution `ADR-0009` section 4.3 already states for final programme
integration, `feature/<programme> -> <repository development branch>`, and it
changes no substantive Metrics architecture.

Under `ADR-0009` those child branches are spelled:

```text
feature/metrics--<slice>
```

`ADR-0009` standardizes the repository ref spelling only. It does not amend the substantive Metrics architecture.

## 3. Observed state

The `<release-branch>` column records each repository's verified established release/default branch. Under `ADR-0011` that value is preserved, so no row requires release-branch renaming. The remaining-work column records only work that is independently justified by something other than branch spelling.

| Repository | GitHub default | Active development | `<release-branch>` | Observed release flow | Remaining readiness work |
|---|---|---|---|---|---|
| `thoth` | `master` | `develop` | `master` | `develop -> master` | none; conforms |
| `thoth-app` | `main` | `dev` | `main` (preserved) | `dev -> main` | development-branch normalization `dev -> develop`, protections, CI and readiness, with the Vercel verification those changes actually require (`BR-APP-01`) |
| `thoth-dissemination` | `main` | `develop` | `main` (preserved) | `develop -> main` | protection and release-readiness work, including its production external-write workflows (`BR-DIS-01`) |
| `thoth-sphinx` | `main` | `develop` | `main` (preserved) | none; `main` is placeholder-README-only, `develop` is ahead by the repository-control root `AGENTS.md` only; bootstrap-only, zero workflows | protect `main` and `develop`, preserve the existing `develop` lineage, reconcile repository-local controls (`BR-SPHINX-01`); bootstrap remains `SPHINX-BOOT-01` |
| `thoth-client` (standalone `thoth-pub/thoth-client`) | `master` | `develop` | `master` | `develop -> master` (feature PRs merge to `develop`; `develop` is 1 commit ahead of `master` via a release merge) | none; conforms |
| `thoth-pyramid` | `main` | `dev` | `main` (preserved) | not yet observed as a completed release cycle | any future development-branch normalization is separate; no normalization task is authorized |
| `thoth-strapi` | `main` | `develop` | `main` (preserved) | not yet observed as a completed release cycle | any future topology or readiness work is separate and must account for its publication-capable pull-request workflow; no normalization task is authorized |
| `metrics-dashboard` | `main` | `dev` (approved permanent) | `main` (preserved, Vercel-backed) | `dev -> main` | topology settled as permanent `feature/* -> dev -> main`, with no `dev -> develop` normalization (section 3.3); still open and separate: cleanup of the stale legacy `develop` ref (not deleted; deletion separately authorized), protections, and CI/lint/test readiness under CG-11 (`BR-DASH-01`); Vercel production is **not** moved |
| `metrics-widget` | `main` | `dev` | `main` (preserved) | releases from `main` | development-branch and CI normalization, npm release protection (`BR-WIDGET-01`) |
| `cc-license` | `main` | `develop` | `main` (preserved) | release branch `main` | publication readiness and protection (`BR-LIC-01`) |
| `baboon` | `master` | `develop` | `master` | `develop -> release/* -> master`, tagged and merged back into `develop` | none; conforms |

No repository in this table requires a release-branch rename. Rows whose remaining readiness work is non-empty describe genuinely open protection, development-branch, CI, provider, publication or bootstrap gaps, each owned by a separately scoped and separately authorized task. Removing the rename does **not** close those gaps and does not by itself lower any task's risk; see section 5.

### 3.1 2026-08-15 re-verification notes

`thoth-sphinx` was re-verified live on 2026-08-15, after that repository's own
control-reconciliation work completed. This note supersedes any earlier
statement that `main` and `develop` were identical and both placeholder-only;
that statement is not accurate. Both branches exist and have **diverged**:

- `main` remains the GitHub default branch, is at
  `0896e4061e06bc640f917f1aaf25c14b6e25269a`, and remains the original
  placeholder commit, containing `README.md` alone;
- `develop` is the active development branch, is at
  `ff7de985d03f0c94d5ad8d60727f9cf85b6435cd`, and contains a root `AGENTS.md`
  plus the same, unchanged placeholder `README.md`;
- `compare/main...develop` reports `ahead_by: 8, behind_by: 0, status: ahead`,
  with the root `AGENTS.md` the only content difference between the branches.

The divergence is completed repository-control and reconciliation work only:
the commits that added the repository-local root `AGENTS.md` and subsequently
corrected its recorded content. It is **not** branch normalization, and it is
not runtime, bootstrap, Cargo, CI or provider implementation. The repository has
zero GitHub Actions workflows and no such implementation, so it remains
bootstrap-only and non-implementation-ready. Under `ADR-0011` `main` remains
this repository's established release/default branch and no `master` branch is
required or authorized. `BR-SPHINX-01` and `SPHINX-BOOT-01` remain separate,
separately authorized and unimplemented tasks; see
`repositories/thoth-sphinx.md`. The row above is corrected accordingly and no
branch normalization is performed.

`thoth-client` (standalone), `thoth-pyramid` and `thoth-strapi` were added and
verified live for the first time on 2026-08-15. See
`repositories/thoth-client.md`, `repositories/thoth-pyramid.md` and
`repositories/thoth-strapi.md` for full detail, and `contracts.md` for the
explicit distinction between the standalone Python `thoth-pub/thoth-client`
and the internal Rust `thoth-client` workspace member in `thoth-pub/thoth`.

### 3.2 2026-08-16 `baboon` registration

`thoth-pub/baboon` was added to this map and verified live on 2026-08-16. It is
a private repository whose observed topology already matches the target
`develop -> master` pattern: `master` is the GitHub default and release branch,
`develop` is the active integration branch, and releases reach `master` through
`release/*` branches, are tagged (for example `v0.5.0`) and are merged back into
`develop`. Verified heads: `develop` `bdf0ee33b6e93179ac76b4ad514a6e71627825d3`,
`master` `36f83a176fc4b195a3ff24c75302c1f2dbf53b1c`, with
`compare/master...develop` reporting `ahead_by: 3, behind_by: 0`.

Because the observed flow already conforms, **no `BR-` normalization task is
created or required for `baboon`**, and none is authorized by this record. See
`repositories/baboon.md` for responsibility, contract relationships, external
state and the three distinct workflow classes, including the HIGH-risk
pull-request-triggered production SFTP scratch write.

No branch normalization is performed by this record. Rows with remaining
readiness work describe a gap against the target topology only; the owning task
remains separately scoped and separately authorized.

### 3.3 2026-09-21 `metrics-dashboard` permanent topology

`thoth-pub/metrics-dashboard` permanently uses `dev` as its
development/integration branch and its established `main` as its
release/default branch:

```text
feature/* -> dev -> main
```

This approved repository-local decision resolves the development-branch question
that `ADR-0011` section 4.4 leaves independent of release-branch naming. It
amends neither `ADR-0011` nor `ADR-0009`. No `dev -> develop` normalization is
planned. The stale legacy `develop` ref is not a workflow branch and has **not**
been deleted; its deletion remains a separately authorized action. Live branch
heads were re-verified on 2026-09-21 and are recorded in
`repositories/metrics-dashboard.md`. No branch, GitHub setting, protection, CI
or Vercel configuration was created or changed by this record, and the
remaining protection, stale-ref and CI readiness work in section 5 stays open.

## 4. Control rule

Every task specification records:

- verified existing base branch;
- verified PR target;
- the repository's verified `<development-branch>` and `<release-branch>`, taken
  from live state rather than from shared naming convention;
- approved target topology;
- normalization dependency or temporary CTO exception;
- programme workflow: `STANDARD` or `PROGRAMME_INTEGRATION`;
- the workflow-appropriate branch form, `feature/<area>/<task>` for `STANDARD` or `feature/<programme>--<slice>` for `PROGRAMME_INTEGRATION`.

No agent creates a branch from a name that has not been verified to exist.

Before creating any governed ref, run the fail-closed namespace preflight in `AGENTS.md` section 5.1 against live refs, symmetrically: a new flat ref requires that no descendant namespace already occupies its location, and a new descendant ref requires that no flat parent ref already occupies its location. On failure, HOLD. No collision may be worked around by deleting, renaming or moving another branch.

## 5. Required branch-readiness tasks

Under [`ADR-0011`](../decisions/ADR-0011-preserve-established-release-branch-names.md)
none of these tasks converts a release branch. Each repository keeps its
established release/default branch, and what remains below is the independently
justified development-branch, protection, CI, provider and publication readiness
work.

Removing the release-branch rename does **not** automatically lower a task's
risk. Each risk statement below is reassessed from the task's remaining CI,
provider, deployment, publishing and external-write effects only. None of these
tasks is authorized by this record.

### BR-APP-01 - `thoth-app` branch readiness

- preserve `main` as the established release/default branch; do not create
  `master` and do not change the GitHub default for naming consistency;
- create or rename `develop` from current `dev`, where separately justified;
- update protections for the verified release and development branches;
- verify previews from feature and development branches;
- verify that the Vercel production branch remains `main` and that rollback to
  the last `main` deployment is preserved;
- retain `dev` until references are verified.

Risk: HIGH. Vercel production routing is **not** moved for branch spelling, but
the development-branch change still touches preview and build configuration for
a production-serving Vercel project, and must be verified with rollback
evidence.

### BR-DIS-01 - `thoth-dissemination` branch readiness

- preserve `main` as the established release/default branch; do not create
  `master`;
- retain `develop`;
- update protections for the verified release and development branches;
- verify release/tag and production-write workflows against the verified release
  branch;
- verify that no external reference assumes a different release branch.

Risk: HIGH because the repository contains production external-write workflows,
which must be re-verified against the verified release branch regardless of its
name.

### BR-SPHINX-01 - Complete `thoth-sphinx` topology

- preserve `main` as the established release/default branch; do **not** create
  `master` and do not switch the default branch;
- retain the existing `develop` branch, preserving the root `AGENTS.md` it
  already carries and the repository-control lineage it records;
- align `develop` with the approved bootstrap base before implementation;
- add protections to `main` and `develop`;
- reconcile repository-local controls against the verified branch state,
  recording that `main` is behind `develop` by the repository-control and
  reconciliation commits;
- perform SPHINX-BOOT-01, which is a separate task, on a task branch from
  `develop`.

Risk: MEDIUM before runtime exists. This task must be re-specified against
`ADR-0011` once that decision is repository-authoritative, and is not authorized
by this record.

### BR-DASH-01 - `metrics-dashboard` branch readiness

Topology: **settled** as permanent `feature/* -> dev -> main` (section 3.3).
`dev` is the approved permanent development/integration branch and the
established Vercel-backed `main` remains the release/default branch. No
development-branch normalization and no `dev -> develop` reconciliation is
required. Do **not** create `master` and do **not** move Vercel production.

Remaining readiness work, still open, each part separately scoped and
separately authorized:

- verify the live `dev`, stale `develop` and `main` heads, recent merged pull
  requests and, under separate provider-read authorization, Vercel branch
  settings before any stale-ref deletion or protection change, with verified
  rollback for any change that affects Vercel;
- clean up the stale legacy `develop` ref, which is not a workflow branch and
  has **not** been deleted: retain it until its deletion is separately
  authorized, and never use it as a base or pull-request target;
- update protections for the verified `main` and `dev` branches;
- add CI, tests and the lint and production-build gate under
  [CG-11](./control-gaps.md#cg-11---ci-gaps);
- prohibit creation of `feature/metrics` from stale `develop`; a
  repository-local `feature/metrics` is created only from a verified `dev`
  head, under section 6.

The topology decision itself requires no Vercel production or default-routing
change.

Risk: HIGH, unchanged by the topology decision. This repository serves
production from Vercel on `main`. Settling the topology removes the
development-branch reconciliation from this task, but removing work does not by
itself lower risk; the remaining stale-ref, protection and CI work is reassessed
by its own separately scoped task.

### BR-WIDGET-01 - `metrics-widget` branch readiness

- preserve `main` as the established release/default branch; do not create
  `master`;
- create or rename `develop` from `dev`, where separately justified;
- update CI filters to the verified branches;
- preserve GitHub-release-to-npm publishing;
- verify release tags continue to resolve against the verified release branch;
- retain old branches until automation and consumers are verified.

Risk: HIGH because release automation publishes a public package, and CI-filter
and development-branch changes touch the path that leads to that publication.

### BR-LIC-01 - `cc-license` branch readiness

- preserve `main` as the established release/default branch; do not create
  `master`;
- retain `develop`;
- update CI filters and protections for the verified branches;
- verify crate publication against the verified release branch;
- record the publication command, credentials, approval and rollback/yank
  procedure, which remain unverified.

Risk: MEDIUM.

## 6. Programme branch readiness

Publisher Services tasks may begin only when their repository's actual development branch and PR target are verified. They use standard task branches, not a programme branch.

A repository-local Metrics `feature/metrics` branch may be created only when:

- that repository has a verified `<development-branch>` that is its approved
  development branch under sections 3 and 5 (for example `develop` in `thoth`
  and `dev` in `metrics-dashboard`), and `feature/metrics` is created from that
  branch's verified head, never from a stale or legacy ref;
- the remaining branch-readiness work in section 5 is complete or a CTO
  exception is recorded;
- Metrics control task `MET-CTRL-01` is merged;
- required shared ADRs are approved;
- CI can validate the intended slices;
- the fail-closed namespace preflight confirms no `feature/metrics/*` refs already occupy that repository's descendant namespace and no exact `feature/metrics` ref conflict exists.
