# CTRL-FOUNDATION-01 - Engineering Control Foundation

Status: APPROVED FOR DOCUMENTATION IMPLEMENTATION; MERGE APPROVAL PENDING  
Programme: Shared Engineering Control  
Repository: `thoth-pub/thoth`  
Risk: LOW  
Workflow: STANDARD  
Base branch: `develop`  
Base commit at branch creation: `652a499dfdfbaa7594537e0865c41ec617f52dc2`  
Task branch: `feature/ai-delivery-operating-model`  
PR target: `develop`  
Pull request: `thoth-pub/thoth#764`  
Dependencies: None  
Production effect: None

## 1. Objective

Establish the complete repository-backed control and orientation foundation required before AI agents implement Publisher Services or Thoth Metrics.

The foundation must make authority, task boundaries, review independence, branch flow, repository context, risk, release gates and programme prerequisites explicit.

## 2. Scope

The task includes documentation only:

1. AI-led engineering operating model.
2. GitHub Flow and large-programme integration-branch rules.
3. Risk classification and model-selection guidance.
4. Task, implementation-report, independent-review and decision templates.
5. Merge, release, production and closure gates.
6. Verified repository, branch, CI, release and environment map.
7. Root and scoped `AGENTS.md` instructions for `thoth`.
8. Agent-instruction rollout plan for related repositories.
9. Publisher Services programme-control documents.
10. Thoth Metrics programme-control documents.
11. Cross-programme decisions or ADRs required before shared implementation.
12. Final terminology, link, source-authority and consistency review.
13. One consolidated changelog entry.

Items 9-11 may remain explicitly blocked on a named CTO decision, but the blocker and required decision record must be present.

## 3. Non-goals

This task does not:

- implement application behaviour;
- add or alter database migrations;
- change GraphQL or export APIs;
- change CI workflow YAML;
- rename branches or change GitHub settings;
- modify Vercel configuration;
- bootstrap `thoth-sphinx`;
- create AWS resources;
- deploy, publish or release;
- access production secrets;
- modify the approved product designs except through a separately approved source correction.

## 4. Invariants

1. GitHub and committed designs remain authoritative.
2. Observed state and target policy are distinct.
3. The canonical repository name is `thoth-pub/thoth-sphinx`.
4. One implementing agent cannot approve its own work.
5. Missing evidence is missing work.
6. No document claims planned infrastructure is deployed.
7. No secret values are committed.
8. Programme scopes remain distinct.
9. Cross-programme decisions require a shared ADR or programme decision.
10. The documentation does not activate production behaviour.

## 5. Acceptance criteria

### Operating model

- [ ] Roles and authority are explicit.
- [ ] Task lifecycle and decisions use `APPROVED`, `CHANGES REQUIRED` or `BLOCKED`.
- [ ] Implementers cannot merge, deploy or self-approve.
- [ ] Production activation requires appropriate CTO approval.

### Branching

- [ ] Normal `develop -> feature -> develop -> master` flow is documented.
- [ ] Large-programme slice/integration flow is documented.
- [ ] Repository-specific branch deviations are recorded.
- [ ] No document assumes a missing branch exists.

### Repository context

- [ ] Every affected repository has a verified map entry.
- [ ] Build/test/generation commands are recorded.
- [ ] CI, release and deployment boundaries are recorded.
- [ ] Missing deployment and branch-protection evidence is explicit.
- [ ] Sphinx is recorded as empty until bootstrapped.

### Agent instructions

- [ ] `thoth` has a root `AGENTS.md`.
- [ ] High-risk directories have scoped instructions.
- [ ] The instructions link to the operating model and repository map.
- [ ] Migration, authorization, generated-contract and workflow safety are explicit.
- [ ] Related-repository rollout is tracked without claiming it is complete.

### Programme readiness

- [ ] Publisher Services control documents exist or have a precise completion plan.
- [ ] Metrics control documents exist or have a precise completion plan.
- [ ] Shared package/capability decisions are recorded or explicitly blocked for CTO decision.
- [ ] No production implementation is described as ready while a required foundation is absent.

### Quality

- [ ] No obsolete Sphinx spelling appears in added files.
- [ ] Internal paths and indexes are valid.
- [ ] No duplicate active control index/design is introduced.
- [ ] `CHANGELOG.md` has one consolidated entry.
- [ ] `git diff --check` passes.
- [ ] Repository CI is green.
- [ ] An independent reviewer inspects the complete final diff.
- [ ] Final decision is `APPROVED`.

## 6. Required verification

```bash
git diff --check
grep -Rni "sphynx" AGENTS.md .github thoth-* docs/engineering || true
find . -name AGENTS.md -print | sort
```

Also:

- inspect every changed file;
- validate relative links and referenced paths;
- compare the repository map with live GitHub/Vercel evidence;
- inspect PR CI at the final head;
- verify no workflow, migration or application source was changed;
- verify the changelog check passes.

## 7. Migration requirements

None.

No SQL, schema or existing data changes are permitted.

## 8. Rollout

1. Keep PR #764 open while documentation chunks are completed.
2. Request independent review only after the complete foundation is present.
3. Resolve findings on the same branch.
4. Obtain CTO merge approval.
5. Merge into `develop`.
6. Replace uploaded Project sources with the final approved versions.
7. Begin repository-local rollout and programme tasks only after the merged foundation is authoritative.

## 9. Rollback

Revert PR #764.

There is no runtime or data rollback.

## 10. Stop conditions

Return `BLOCKED` if:

- the foundation contradicts an approved design;
- a cross-programme decision is presented as settled without CTO approval;
- repository facts cannot be verified;
- the task expands into runtime, CI, branch-setting or deployment changes;
- independent review cannot be obtained;
- the final diff contains secrets or production configuration.

## 11. Implementation and review model

Documentation implementation:

- ChatGPT control conversation, GPT-5.6 Thinking;
- Codex may apply repository edits at medium reasoning.

Independent review:

- a separate Claude or Codex reviewer at high reasoning;
- reviewer must inspect the final GitHub diff and CI;
- the implementing conversation cannot approve the task.

Merge approval:

- CTO.
