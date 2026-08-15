# Agent Instruction Rollout Plan

Status: IN PROGRESS
Owner: CTO

## 1. Current state

| Repository | Current instruction state | Required action |
|---|---|---|
| `thoth` | root and scoped instruction hierarchy added and merged through PR #764 as `5b406e4ef9b5c192cc38eb8a97a41bbd0fc3bc06`; retrospective closeout independently `APPROVED` and merged through PR #767 as `bac598e32abbd0d7e69ff467c82945ee00df02ba`; P0-01 `CLOSED`; issue #765 synchronized on 2026-07-27 and remains open | none for the completed `thoth` foundation closeout; continue with the remaining programme and repository-readiness gates |
| `thoth-app` | no verified root `AGENTS.md` | create from approved repository map |
| `thoth-dissemination` | existing root `AGENTS.md`, incomplete control coverage | revise without losing useful local guidance |
| `thoth-sphinx` | repository-local root `AGENTS.md` merged onto `develop` and verified live 2026-08-15; `README.md` remains the placeholder on both `main` and `develop` | repository-local instruction rollout is complete; no instruction-creation action remains. SPHINX-BOOT-01 must preserve and build on the existing root `AGENTS.md` rather than add it as though absent, and must replace or expand the placeholder README. BR-SPHINX-01 remains separate |
| `metrics-dashboard` | no verified root `AGENTS.md` | create before metrics client migration |
| `metrics-widget` | no verified root `AGENTS.md` | create before metrics API migration/release |
| `cc-license` | no verified root `AGENTS.md` | create before LIC-01 |

## 2. Rollout sequence

1. The retrospective closeout of the already-merged `thoth` control foundation is complete: PR #767 was independently `APPROVED` and merged as `bac598e32abbd0d7e69ff467c82945ee00df02ba`, closing P0-01. Issue #765 was synchronized on 2026-07-27 and remains open as the external programme mirror. No foundation-closeout action remains in `thoth`.
2. Update `thoth-dissemination`, because it already performs production-capable workflows.
3. Add `thoth-app` instructions before Publisher Services or metrics upload UI implementation.
4. `thoth-sphinx` instructions are already added and merged onto its `develop`
   branch, so no instruction-creation step remains here. Its no-production
   bootstrap task (SPHINX-BOOT-01) must instead preserve and update the
   existing root `AGENTS.md` where bootstrap changes what it must say.
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
