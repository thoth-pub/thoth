# Branch Topology

Status: VERIFIED OBSERVED STATE PLUS APPROVED TARGET POLICY
Evidence date: 2026-07-24 for `thoth`, `thoth-app`, `thoth-dissemination`,
`metrics-dashboard`, `metrics-widget`, `cc-license`; independently re-verified
2026-08-15 for `thoth-sphinx`, and newly added and verified 2026-08-15 for
`thoth-client` (standalone), `thoth-pyramid`, `thoth-strapi`.

## 1. Target repository policy

Normal task flow:

```text
feature/<area>/<task> -> develop -> master
```

Approved programme-integration flow:

```text
feature/<programme>/<slice> -> feature/<programme> -> develop -> master
```

`develop` is the target development branch. `master` is the target release/default branch. Merged task and slice branches are deleted.

An approved programme design decides whether it uses direct task PRs or a programme integration branch.

## 2. Programme-specific policy

### Publisher Services

The approved design requires one fresh branch and one PR per task. Each task targets the repository's verified development branch. There is no long-lived `feature/publisher-services` branch.

### Thoth Metrics

The Metrics design requires one repository-local `feature/metrics` integration branch per affected repository after branch readiness. Bounded child branches target that integration branch, followed by a final repository-local PR to `develop`.

## 3. Observed state

| Repository | GitHub default | Active development | Observed release flow | Target-policy state |
|---|---|---|---|---|
| `thoth` | `master` | `develop` | `develop -> master` | conforms |
| `thoth-app` | `main` | `dev` | `dev -> main` | normalization required |
| `thoth-dissemination` | `main` | `develop` | `develop -> main` | release-branch normalization required |
| `thoth-sphinx` | `main` | `develop` | none; `main` is placeholder-README-only, `develop` is ahead by the repository-control root `AGENTS.md` only; bootstrap-only, zero workflows | `master`/protection/bootstrap required |
| `thoth-client` (standalone `thoth-pub/thoth-client`) | `master` | `develop` | `develop -> master` (feature PRs merge to `develop`; `develop` is 1 commit ahead of `master` via a release merge) | conforms to the `develop -> master` pattern |
| `thoth-pyramid` | `main` | `dev` | not yet observed as a completed release cycle | normalization required if brought under target topology |
| `thoth-strapi` | `main` | `develop` | not yet observed as a completed release cycle | normalization required if brought under target topology |
| `metrics-dashboard` | `main` | `dev` | `dev -> main` | development/release/Vercel normalization required |
| `metrics-widget` | `main` | `dev` | releases from `main` | normalization required |
| `cc-license` | `main` | `develop` | release branch `main` | release-branch normalization required |

### 3.1 2026-08-15 re-verification notes

`thoth-sphinx` was re-verified live on 2026-08-15. This note supersedes the
earlier 2026-08-15 statement that `main` and `develop` were identical and both
placeholder-only; that statement is no longer accurate. Both branches exist and
have **diverged**:

- `main` is at `0896e4061e06bc640f917f1aaf25c14b6e25269a` and remains the
  original placeholder-only branch, containing `README.md` alone;
- `develop` is at `7d6d4a24fde1ee0473f2ac66387167998f67ebb1` and contains a
  root `AGENTS.md` plus the same, unchanged placeholder `README.md`;
- `compare/main...develop` reports `ahead_by: 5, behind_by: 0, status: ahead`,
  with `AGENTS.md` the only changed file.

The divergence is solely the completed repository-control work that added the
repository-local root `AGENTS.md`. The repository has zero GitHub Actions
workflows and no runtime, bootstrap, CI or provider implementation, so it
remains bootstrap-only and non-implementation-ready. `BR-SPHINX-01` and
`SPHINX-BOOT-01` remain separate, separately authorized tasks; see
`repositories/thoth-sphinx.md`. The row above is corrected accordingly and no
branch normalization is performed.

`thoth-client` (standalone), `thoth-pyramid` and `thoth-strapi` were added and
verified live for the first time on 2026-08-15. See
`repositories/thoth-client.md`, `repositories/thoth-pyramid.md` and
`repositories/thoth-strapi.md` for full detail, and `contracts.md` for the
explicit distinction between the standalone Python `thoth-pub/thoth-client`
and the internal Rust `thoth-client` workspace member in `thoth-pub/thoth`.

No branch normalization is performed by this record. Rows marked
"normalization required" describe a gap against the target topology only; a
normalization task remains separately scoped and separately authorized.

## 4. Control rule

Every task specification records:

- verified existing base branch;
- verified PR target;
- approved target topology;
- normalization dependency or temporary CTO exception;
- programme workflow: `STANDARD` or `PROGRAMME_INTEGRATION`.

No agent creates a branch from a name that has not been verified to exist.

## 5. Required normalization tasks

### BR-APP-01 - Normalize `thoth-app`

- create `master` from current `main`;
- create or rename `develop` from current `dev`;
- update default branch and protections;
- update Vercel production branch from `main` to `master`;
- verify previews from feature/develop branches;
- preserve rollback to the last `main` deployment;
- retain `main` and `dev` until references are verified.

Risk: HIGH because Vercel production routing changes.

### BR-DIS-01 - Normalize `thoth-dissemination`

- create `master` from current `main`;
- retain `develop`;
- update default branch and protections;
- verify release/tag and production-write workflows;
- retain `main` until external references are checked.

Risk: HIGH because the repository contains production external-write workflows.

### BR-SPHINX-01 - Complete `thoth-sphinx` topology

- create `master` from current `main`, which is placeholder-README-only and
  behind `develop`;
- retain the existing `develop` branch, preserving the root `AGENTS.md` it
  already carries;
- align `develop` with the approved bootstrap base before implementation;
- make `master` the release/default branch;
- add protections to `master` and `develop`;
- perform SPHINX-BOOT-01, which is a separate task, on a task branch from
  `develop`;
- retain `main` until references are confirmed absent.

Risk: MEDIUM before runtime exists.

### BR-DASH-01 - Normalize `metrics-dashboard`

- verify `dev`, `develop`, `main`, recent merged pull requests and Vercel branch
  settings;
- reconcile the active `dev` history into the target `develop` branch, using a
  fast-forward only when Git proves it is safe;
- otherwise use a separately reviewed merge or replacement plan that preserves
  history;
- create `master` from the verified current `main`;
- update the default branch, protections and Vercel production branch only under
  this separately approved HIGH-risk task;
- retain `main`, `dev` and the old `develop` reference until all external
  references and rollback requirements are verified;
- prohibit creation of `feature/metrics` from stale `develop`.

Risk: HIGH because production deployment routing changes.

### BR-WIDGET-01 - Normalize `metrics-widget`

- create `master` from current `main`;
- create or rename `develop` from `dev`;
- update CI filters;
- preserve GitHub-release-to-npm publishing;
- verify release tags use `master`;
- retain old branches until automation/consumers are verified.

Risk: HIGH because release automation publishes a public package.

### BR-LIC-01 - Normalize `cc-license`

- create `master` from current `main`;
- retain `develop`;
- update CI filters and default branch;
- verify crate publication;
- retain `main` until release references are checked.

Risk: MEDIUM.

## 6. Programme branch readiness

Publisher Services tasks may begin only when their repository's actual development branch and PR target are verified. They use standard task branches, not a programme branch.

A repository-local Metrics `feature/metrics` branch may be created only when:

- that repository has a verified `develop` branch;
- normalization is complete or a CTO exception is recorded;
- Metrics control task `MET-CTRL-01` is merged;
- required shared ADRs are approved;
- CI can validate the intended slices.
