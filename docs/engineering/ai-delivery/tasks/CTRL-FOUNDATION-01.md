# CTRL-FOUNDATION-01 - Engineering Control Foundation

Status: IMPLEMENTATION COMPLETE; INDEPENDENT REVIEW REQUIRED
Programme: Shared Engineering Control
Repository: `thoth-pub/thoth`
Risk: LOW
Workflow: STANDARD
Base branch: `develop`
Base commit: `652a499dfdfbaa7594537e0865c41ec617f52dc2`
Task branch: `feature/ai-delivery-operating-model`
PR target: `develop`
Pull request: [#764](https://github.com/thoth-pub/thoth/pull/764)
Publisher Services issue: [#765](https://github.com/thoth-pub/thoth/issues/765)
Thoth Metrics issue: [#766](https://github.com/thoth-pub/thoth/issues/766)
Production effect: None

## 1. Objective

Establish the complete repository-backed control and orientation foundation required before AI agents implement Publisher Services or Thoth Metrics.

## 2. Scope delivered

1. operating model;
2. branching and programme integration workflow;
3. risk/model guidance;
4. task/report/review/decision templates;
5. release gates;
6. repository/environment map;
7. root and scoped Thoth agent instructions;
8. related-repository instruction rollout;
9. Publisher Services controls and issue;
10. Metrics controls and issue;
11. proposed cross-programme ADRs;
12. top-level engineering index;
13. changelog entry;
14. independent-review brief.

## 3. Non-goals

No application code, database migration, GraphQL/export API, CI workflow, branch setting, Vercel setting, Sphinx bootstrap, AWS resource, deployment, release, production secret or production behavior is changed.

The external Metrics design spelling correction remains a source-owner action outside this PR.

## 4. Acceptance status

### Operating model

- [x] roles and authority explicit;
- [x] task lifecycle and decisions explicit;
- [x] no self-approval/merge/deploy;
- [x] CTO approval for high-risk activation.

### Branching and repository context

- [x] normal and programme workflows documented;
- [x] repository deviations recorded;
- [x] affected repositories mapped;
- [x] missing environment/deployment evidence explicit;
- [x] Sphinx recorded as empty.

### Agent and programme controls

- [x] root/scoped Thoth instructions exist;
- [x] related rollout tracked;
- [x] Publisher Services controls and issue exist;
- [x] Metrics controls and issue exist;
- [x] shared ADRs remain proposed CTO gates;
- [x] dependent implementation remains blocked.

### Quality

- [x] canonical Sphinx spelling used in repository additions;
- [x] one top-level engineering index;
- [x] programme tracker links present;
- [x] consolidated changelog entry present;
- [ ] final `git diff --check` at review head;
- [ ] final CI green at review head;
- [ ] independent complete-diff review;
- [ ] independent decision `APPROVED`.

## 5. Final verification

```bash
git diff --check
grep -Rni "sphynx" AGENTS.md .github thoth-* docs || true
grep -Rni "NOT YET CREATED" docs || true
find . -name AGENTS.md -print | sort
```

Also verify the full final changed-file set, no source/migration/workflow behavior changes, repository links, ADR statuses, issues #765/#766, changelog, CI and absence of secrets.

## 6. Rollout and rollback

1. independent review;
2. remediation on this branch;
3. CTO merge approval;
4. merge into `develop`;
5. replace stale Project sources;
6. begin separately specified readiness tasks.

Rollback is reverting PR #764. There is no runtime/data rollback.

## 7. Review model

Implementation: ChatGPT control conversation, GPT-5.6 Thinking.

Independent review: separate Codex or Claude reviewer at high reasoning inspecting actual GitHub diff and CI.

Merge approval: CTO.
