# Agent Instruction Rollout Plan

Status: IN PROGRESS
Owner: CTO

## 1. Current state

| Repository | Current instruction state | Required action |
|---|---|---|
| `thoth` | root and scoped instruction hierarchy added and merged through PR #764 as `5b406e4ef9b5c192cc38eb8a97a41bbd0fc3bc06`; retrospective closeout remediation and fresh independent approval remain outstanding | complete PR #767 closeout remediation, fresh independent approval and CTO closeout merge |
| `thoth-app` | no verified root `AGENTS.md` | create from approved repository map |
| `thoth-dissemination` | existing root `AGENTS.md`, incomplete control coverage | revise without losing useful local guidance |
| `thoth-sphinx` | placeholder-only README; no verified `AGENTS.md` | add `AGENTS.md` and replace or expand the README during bootstrap |
| `metrics-dashboard` | no verified root `AGENTS.md` | create before metrics client migration |
| `metrics-widget` | no verified root `AGENTS.md` | create before metrics API migration/release |
| `cc-license` | no verified root `AGENTS.md` | create before LIC-01 |

## 2. Rollout sequence

1. Complete retrospective closeout of the already-merged `thoth` control foundation through PR #767, fresh independent approval and CTO merge authorization.
2. Update `thoth-dissemination`, because it already performs production-capable workflows.
3. Add `thoth-app` instructions before Publisher Services or metrics upload UI implementation.
4. Add `thoth-sphinx` instructions as part of its no-production bootstrap task.
5. Add dashboard and widget instructions before client cutover work.
6. Add `cc-license` instructions before changing the supported licence contract.

## 3. Per-repository minimum content

Each root `AGENTS.md` must record:

- repository responsibility and non-responsibilities;
- actual branch and release topology;
- required task metadata;
- standard install/build/test/lint commands;
- generated files and how to regenerate them;
- CI and release gates;
- authorization/security boundaries;
- deployment or package-publishing effects;
- prohibited production actions;
- completion report and independent review requirements;
- stop conditions.

## 4. Delivery model

Instruction rollout to other repositories requires repository-local PRs.

Use the repository's current verified development branch until an approved branch-normalization task completes.

Do not combine:

- branch normalization;
- CI modernization;
- functional implementation;
- agent-instruction rollout

unless the approved specification proves they are inseparable.

## 5. Source authority

Generate each file from:

- the merged repository/environment map;
- live repository code and workflows;
- the approved operating model;
- current deployment/release evidence.

Do not copy the `thoth` root file verbatim into another repository.
