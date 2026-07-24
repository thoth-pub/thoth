# Branch Topology

Status: VERIFIED OBSERVED STATE PLUS APPROVED TARGET POLICY
Evidence date: 2026-07-24

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
| `thoth-sphinx` | `main` | `develop` | none; repository empty | `master`/protection/bootstrap required |
| `metrics-dashboard` | `main` | `develop` | `develop -> main` | release/Vercel normalization required |
| `metrics-widget` | `main` | `dev` | releases from `main` | normalization required |
| `cc-license` | `main` | `develop` | release branch `main` | release-branch normalization required |

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

- create `master` from current `main`;
- retain the existing `develop` branch;
- align `develop` with the approved bootstrap base before implementation;
- make `master` the release/default branch;
- add protections to `master` and `develop`;
- perform SPHINX-BOOT-01 on a task branch from `develop`;
- retain `main` until references are confirmed absent.

Risk: MEDIUM before runtime exists.

### BR-DASH-01 - Normalize `metrics-dashboard`

- create `master` from current `main`;
- retain `develop`;
- update Vercel production branch, default branch and protections;
- verify preview/production domains;
- retain `main` through an observation window.

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
