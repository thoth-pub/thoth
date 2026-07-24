# Branch Topology

Status: Verified observed state plus approved target policy  
Evidence date: 2026-07-24

## 1. Approved target policy

Thoth's target GitHub Flow is:

```text
feature/<area>/<task> -> develop -> master
```

Large programme flow:

```text
feature/<programme>/<slice> -> feature/<programme> -> develop -> master
```

`develop` is the development integration branch.  
`master` is the release/default branch.  
Merged task and slice branches are deleted.

## 2. Observed state

| Repository | GitHub default | Active development | Observed release flow | Conforms |
|---|---|---|---|---|
| `thoth` | `master` | `develop` | `develop -> master` | yes |
| `thoth-app` | `main` | `dev` | `dev -> main` | no |
| `thoth-dissemination` | `main` | `develop` | `develop -> main` | partial |
| `thoth-sphinx` | `main` | none | none | no |
| `metrics-dashboard` | `main` | `develop` | `develop -> main` | partial |
| `metrics-widget` | `main` | `dev` | releases from `main` | no |
| `cc-license` | `main` | `develop` | release branch `main` | partial |

## 3. Control rule

A task specification must record:

- observed base branch;
- observed PR target;
- desired target;
- whether a normalization task has completed;
- whether an approved temporary exception is being used.

No agent may create a branch from a name that has not been verified to exist.

## 4. Required normalization tasks

### BR-APP-01 - Normalize `thoth-app`

- create `master` from current `main`;
- create or rename `develop` from current `dev`;
- update GitHub default branch and branch protections;
- update Vercel production branch from `main` to `master`;
- verify previews build from feature/develop branches;
- preserve rollback to the last `main` production deployment;
- update documentation and CI branch filters;
- retain `main` and `dev` temporarily until references are verified.

Risk: High because Vercel production deployment routing changes.

### BR-DIS-01 - Normalize `thoth-dissemination`

- create `master` from current `main`;
- keep current `develop`;
- update GitHub default branch and protections;
- verify release/tag workflows;
- verify operational workflows that explicitly require `develop`;
- retain `main` temporarily until external references are checked.

Risk: High because the repository contains production external-write workflows.

### BR-SPHINX-01 - Establish `thoth-sphinx` topology

- create `master` from current `main`;
- make `master` the release/default branch;
- create `develop` from `master`;
- add branch protections;
- perform repository bootstrap on a task branch from `develop`;
- delete `main` only after references are confirmed absent.

Risk: Medium before runtime exists.

### BR-DASH-01 - Normalize `metrics-dashboard`

- create `master` from current `main`;
- retain current `develop`;
- update Vercel production branch to `master`;
- update default branch and protections;
- verify preview and production domains;
- retain `main` temporarily through an observation window.

Risk: High because production deployment routing changes.

### BR-WIDGET-01 - Normalize `metrics-widget`

- create `master` from current `main`;
- create or rename `develop` from `dev`;
- update CI branch filters;
- preserve GitHub-release-to-npm publishing;
- verify release-tag checkout uses `master`;
- retain old branches temporarily until consumers and automation are verified.

Risk: High because release automation publishes a public npm package.

### BR-LIC-01 - Normalize `cc-license`

- create `master` from current `main`;
- retain current `develop`;
- update CI branch filters and default branch;
- verify crate publication procedure;
- retain `main` temporarily until release references are checked.

Risk: Medium.

## 5. Programme integration branches

Do not create `feature/publisher-services` or `feature/metrics` in a repository until:

- the repository has a verified development branch;
- branch normalization is complete or a CTO-approved exception is recorded;
- the programme control documents exist;
- CI can validate the intended slices.

`thoth` may create programme branches from `develop` once CTRL-01/CTRL-02 and the relevant programme-control task are approved.
