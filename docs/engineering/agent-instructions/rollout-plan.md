# Agent Instruction Rollout Plan

Status: INSTRUCTION ROLLOUT COMPLETE FOR EVERY MANAGED REPOSITORY
Owner: CTO

Every repository in this plan carries a merged, authoritative repository-local
root `AGENTS.md` on its active development branch. No instruction-creation step
remains. This plan is retained as the durable record of that rollout and of the
separate gates each repository still has; those gates are tracked in
[`control-gaps.md`](../repository-map/control-gaps.md), not here.

## 1. Current state

| Repository | Current instruction state | Required action |
|---|---|---|
| `thoth` | root and scoped instruction hierarchy added and merged through PR #764 as `5b406e4ef9b5c192cc38eb8a97a41bbd0fc3bc06`; retrospective closeout independently `APPROVED` and merged through PR #767 as `bac598e32abbd0d7e69ff467c82945ee00df02ba`; P0-01 `CLOSED`; issue #765 synchronized on 2026-07-27 and remains open | none for the completed `thoth` foundation closeout; continue with the remaining programme and repository-readiness gates |
| `thoth-app` | repository-local root `AGENTS.md` merged onto `dev` through PR [#114](https://github.com/thoth-pub/thoth-app/pull/114) and verified live 2026-08-15 at `7a4e7c6ceaec36fbdb201eaeb9ae36985a709889` | none for instruction rollout; the merged file is authoritative on `dev`. BR-APP-01 branch normalization and the [CG-11](../repository-map/control-gaps.md#cg-11---ci-gaps) CI gaps remain separate |
| `thoth-dissemination` | repository-local root `AGENTS.md` revised for complete control coverage and merged onto `develop` through PR [#96](https://github.com/thoth-pub/thoth-dissemination/pull/96), verified live 2026-08-15 at `71ef7724326e9e75ccea2c004b5ca5be8197f27e` | none for instruction rollout; the merged file is authoritative on `develop`. BR-DIS-01 branch normalization and the repository's README/environment-protection contradiction remain separate follow-ups |
| `thoth-client` (standalone) | repository-local root `AGENTS.md` merged onto `develop` through PR [#55](https://github.com/thoth-pub/thoth-client/pull/55) and verified live 2026-08-15 at `d6ffdc67c48cbf64f8a716f26d7d82eb541d1ecf` | none for instruction rollout. Distinct from the internal Rust `thoth-client` workspace member in this repository; see [`contracts.md`](../repository-map/contracts.md) section 1 |
| `thoth-pyramid` | repository-local root `AGENTS.md` merged onto `dev` through PR [#15](https://github.com/thoth-pub/thoth-pyramid/pull/15), revised by PR [#17](https://github.com/thoth-pub/thoth-pyramid/pull/17) to record repaired CI trigger coverage, and verified live 2026-08-16 at `8f2d6faf70aabea61d11cbf361f602b719f9b3e2` | none for instruction rollout; the merged file is authoritative on `dev`. Its `dev`-target CI **trigger** gap is repaired (see [CG-11](../repository-map/control-gaps.md#cg-11---ci-gaps)); its branch-topology normalization and its broader CI-quality, formatting, dependency/build and codegen concerns remain separate |
| `thoth-strapi` | repository-local root `AGENTS.md` merged onto `develop` through PR [#5](https://github.com/thoth-pub/thoth-strapi/pull/5) and verified live 2026-08-15 at `306220326189697252a708a203d6b4cc02f018cc` | none for instruction rollout; the merged file is authoritative on `develop`. Its Docker/Node CI defect remains a separate follow-up |
| `thoth-sphinx` | repository-local root `AGENTS.md` merged onto `develop` and verified live 2026-08-15; `README.md` remains the placeholder on both `main` and `develop` | repository-local instruction rollout is complete; no instruction-creation action remains. SPHINX-BOOT-01 must preserve and build on the existing root `AGENTS.md` rather than add it as though absent, and must replace or expand the placeholder README. BR-SPHINX-01 remains separate |
| `metrics-dashboard` | repository-local root `AGENTS.md` merged onto `dev` through PR [#10](https://github.com/thoth-pub/metrics-dashboard/pull/10) and verified live 2026-08-16 at `963b0ea78a9a65153ab7d78b7c26e3cb35d763f4` | none for instruction rollout; the merged file is authoritative on `dev`. BR-DASH-01 branch normalization and its absent CI/tests ([CG-11](../repository-map/control-gaps.md#cg-11---ci-gaps)) remain separate |
| `metrics-widget` | repository-local root `AGENTS.md` merged onto `dev` through PR [#2](https://github.com/thoth-pub/metrics-widget/pull/2) and verified live 2026-08-16 at `363bce443b4a87459f5197e38bb7f4cfd4518f60` | none for instruction rollout; the merged file is authoritative on `dev`. BR-WIDGET-01 branch normalization and its missing unit-test suite ([CG-11](../repository-map/control-gaps.md#cg-11---ci-gaps)) remain separate |
| `cc-license` | repository-local root `AGENTS.md` merged onto `develop` through PR [#2](https://github.com/thoth-pub/cc-license/pull/2) and verified live 2026-08-16 at `3dd497981da5d540739158d086394d22b3146b25`, after supporting repair PR [#4](https://github.com/thoth-pub/cc-license/pull/4) merged first | none for instruction rollout; the merged file is authoritative on `develop`. BR-LIC-01 branch normalization and its old-Actions modernization ([CG-11](../repository-map/control-gaps.md#cg-11---ci-gaps)) remain separate. No crate publication occurred |
| `baboon` | repository-local root `AGENTS.md` merged onto `develop` through PR [#16](https://github.com/thoth-pub/baboon/pull/16) and verified live 2026-08-16 at `bdf0ee33b6e93179ac76b4ad514a6e71627825d3` | none for instruction rollout; the merged file is authoritative on `develop`. Its topology already conforms, so no normalization task applies. Opening or updating a pull request there has a HIGH-risk automatic production SFTP scratch write; see [`baboon.md`](../repository-map/repositories/baboon.md) |

## 2. Rollout sequence

1. The retrospective closeout of the already-merged `thoth` control foundation is complete: PR #767 was independently `APPROVED` and merged as `bac598e32abbd0d7e69ff467c82945ee00df02ba`, closing P0-01. Issue #765 was synchronized on 2026-07-27 and remains open as the external programme mirror. No foundation-closeout action remains in `thoth`.
2. `thoth-dissemination`, `thoth-app`, `thoth-client` (standalone),
   `thoth-pyramid` and `thoth-strapi` instructions are added and merged onto
   their active development branches, so no instruction-creation step remains
   for any of them. Their merged root `AGENTS.md` files are the authoritative
   repository-local controls and must be read and preserved by later work in
   those repositories rather than recreated as though absent.
3. Adding instructions is therefore no longer a prerequisite of Publisher
   Services or metrics upload UI implementation in `thoth-app`, nor of
   production-capable work in `thoth-dissemination`. Each repository's own
   remaining gates — branch normalization, CI gaps and readiness — are
   separate and unaffected.
4. `thoth-sphinx` instructions are already added and merged onto its `develop`
   branch, so no instruction-creation step remains here. Its no-production
   bootstrap task (SPHINX-BOOT-01) must instead preserve and update the
   existing root `AGENTS.md` where bootstrap changes what it must say.
5. `metrics-dashboard`, `metrics-widget` and `cc-license` instructions are
   added and merged onto their active development branches under programme
   `CTRL-DELIVERY-02`, so no instruction-creation step remains for any of them.
   Adding instructions is therefore no longer a prerequisite of client cutover
   work, of metrics API migration or release, or of changing the supported
   licence contract. Each repository's own remaining gates are separate and
   unaffected.
6. `thoth-pub/baboon` was brought under management by the same programme and
   its instructions are merged onto `develop`. Work there must additionally
   respect its external-write controls: opening or updating a pull request
   triggers a live production SFTP scratch write, and the production MARC feed
   is a separately authorized manual operation.

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
