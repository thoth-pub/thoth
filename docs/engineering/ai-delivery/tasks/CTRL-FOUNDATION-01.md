# CTRL-FOUNDATION-01 - Engineering Control Foundation

Status: IN REVIEW
Programme: Shared Engineering Control
Repository: `thoth-pub/thoth`
Workflow: STANDARD
Base branch: `develop`
PR target: `develop`
Programme integration branch: None
Risk: LOW
Owner: Javier Arias, CTO
Approved by: Javier Arias, CTO
Approval date: 2026-07-24
Dependencies: None for documentation implementation; final review depends on authorized access to the private approved designs at the revisions recorded in `docs/engineering/design-references.md`, issues #765/#766 and current PR CI
Target branch: `feature/ai-delivery-operating-model`
Pull request: [#764](https://github.com/thoth-pub/thoth/pull/764)
Publisher Services issue: [#765](https://github.com/thoth-pub/thoth/issues/765)
Thoth Metrics issue: [#766](https://github.com/thoth-pub/thoth/issues/766)
Production effect: None

## 1. Objective

Establish a complete repository-backed control and orientation foundation for AI-assisted engineering delivery before implementation begins for Publisher Services or Thoth Metrics.

The outcome must make authority, task boundaries, review independence, repository state, branch flow, risk, release controls, programme prerequisites and missing evidence explicit.

## 2. Background and authority

Authoritative inputs:

1. [Private Publisher Services design](https://docs.google.com/document/d/1kr2Ft0Y4pxgcXGyFAKs_wfFx4I0jlxEvaceswE5Dus8/edit), Drive revision `3`
2. [Private Thoth Metrics design](https://docs.google.com/document/d/11AeQFGpm0kUZajBM5PrAqsttmzJlpUrt89tGYyVM8c0/edit), Drive revision `6`
3. [`docs/engineering/design-references.md`](../../design-references.md)
4. current repository state, migrations, workflows and generated contracts;
5. PR #764, issues #765/#766 and their CI evidence.

The Publisher Services design is approved for phased implementation and requires one fresh task branch and one PR per task, without a long-lived programme branch.

The Metrics design fixes the canonical architecture and requires repository-local `feature/metrics` integration branches after branch readiness; named source mappings, fixtures, capability assignments and complete OPERAS discovery remain dependencies.

The private Metrics design contains an obsolete spelling. Repository controls use `thoth-pub/thoth-sphinx` and `Sphinx`, while correction of the private source remains separately controlled.

## 3. Explicit scope

The task must add and reconcile:

1. the AI-led engineering operating model;
2. standard and approved programme-integration branch workflows;
3. risk classification and model-selection guidance;
4. task, implementation-report, review and decision templates;
5. merge, release, production and closure gates;
6. a verified repository, branch, CI, release and environment map;
7. root and scoped `AGENTS.md` instructions for `thoth`;
8. a rollout plan for related-repository agent instructions;
9. proposed cross-programme package/capability and platform-boundary ADRs;
10. Publisher Services programme controls and master issue;
11. Thoth Metrics programme controls and master issue;
12. private approved-design links and exact Drive revision metadata used for review;
13. a top-level engineering index, consolidated changelog entry and independent-review brief;
14. explicit control gaps for all unverified or deferred work.

## 4. Non-goals

The task must not:

1. change Rust application behaviour;
2. change GraphQL or export APIs;
3. add or alter database migrations or persisted data;
4. change GitHub Actions workflow behaviour;
5. change branch settings or protections;
6. change Vercel, AWS or runtime configuration;
7. bootstrap `thoth-sphinx`;
8. deploy, publish, release or activate production behaviour;
9. access or commit production secrets;
10. approve ADR-0001 or ADR-0002 merely by committing them;
11. publish, copy or silently alter the private approved designs.

## 5. Invariants

The implementation must preserve:

1. merged repository evidence outranks conversation and memory;
2. approved designs and ADRs outrank programme summaries;
3. proposed ADRs are not implementation authority;
4. one implementing agent cannot approve or merge its own work;
5. missing evidence is missing work;
6. observed state, approved target policy and planned architecture remain distinct;
7. Publisher Services uses one task branch and one PR per task;
8. Metrics may use repository-local integration branches only after readiness gates;
9. package selection never implicitly changes distribution assignments;
10. distribution and metrics platform domains remain separate;
11. no document describes planned infrastructure as deployed;
12. this PR remains documentation-only and has no production effect.

## 6. Required behaviour

### 6.1 Success behaviour

The merged foundation provides one navigable control surface that allows an independent implementer or reviewer to identify:

- programme, repository, task, base and target;
- approved specification and design inputs;
- risk, permissions and stop conditions;
- actual repository/branch/environment state;
- required tests and evidence;
- release, rollout and rollback controls;
- programme dependencies and blocked decisions.

### 6.2 Failure behaviour

The documents must require `BLOCKED` when an approved specification, design dependency, repository base, authorization path, migration evidence, production control or cross-programme decision is absent.

No failure to retrieve configuration or evidence may be interpreted as permission to broaden scope or continue production work.

### 6.3 Authorization

Not applicable to runtime behaviour.

The control documents must prohibit implementing agents from merging, deploying, accessing production secrets or approving their own work.

### 6.4 Concurrency and idempotency

Not applicable to this documentation-only task.

The programme controls must nevertheless require future operational tasks to specify concurrency, leases, retries and idempotency where relevant.

### 6.5 Compatibility

The task must be additive and documentation-only.

Existing source, API, schema, migration, package, deployment and release behaviour must remain unchanged.

## 7. Data and migration requirements

Migration required: NO

- schema changes: none;
- existing-data effect: none;
- locking/downtime: none;
- rollback: revert PR #764;
- empty/populated database tests: not applicable to the diff, while existing repository migration CI must remain green.

## 8. Observability and operations

Required logs/metrics/alerts: none for this documentation-only change.

Required operational evidence:

- repository CI at the exact reviewed head;
- no runtime or production configuration changes;
- issues #765/#766 available;
- all deferred operational controls explicitly recorded.

Operational runbook changes: none.

## 9. Acceptance criteria

### Operating model and authority

- [x] roles, authority hierarchy and task lifecycle are explicit;
- [x] implementers cannot self-approve, merge, deploy or access production secrets;
- [x] review decisions use `APPROVED`, `CHANGES REQUIRED` or `BLOCKED`;
- [x] high-risk production activation requires CTO approval.

### Branching

- [x] standard `feature -> develop -> master` flow is documented;
- [x] programme integration flow is available only where an approved design requires it;
- [x] Publisher Services is recorded as one fresh task branch/PR per task;
- [x] Metrics is recorded as repository-local `feature/metrics` integration flow after readiness;
- [x] repository branch deviations and normalization tasks are explicit;
- [x] no missing branch is invented.

### Repository context and instructions

- [x] every affected repository has a map entry;
- [x] build/test/generation commands are recorded where verified;
- [x] missing CI, deployment, branch-protection and schema-generation evidence is explicit;
- [x] root and scoped Thoth instructions exist;
- [x] related-repository instruction rollout remains tracked as incomplete.

### Programme controls

- [x] Publisher Services controls and issue #765 exist;
- [x] Metrics controls and issue #766 exist;
- [x] private approved-design links and exact Drive revision metadata are recorded;
- [x] ADR-0001 and ADR-0002 remain `PROPOSED`;
- [x] dependent implementation remains blocked.

### Quality and review

- [x] canonical `thoth-sphinx` / `Sphinx` spelling is used in committed repository documents;
- [x] stale CTRL/CG references identified by review are corrected;
- [x] one top-level engineering index exists;
- [x] changelog entry exists;
- [x] `git diff --check` passes through remediation head `104ffaa16b2436cea2b4e2779c241b016916f083`;
- [x] all required CI succeeds at remediation head `104ffaa16b2436cea2b4e2779c241b016916f083`;
- [x] the implementation report records exact evidence through remediation head `104ffaa16b2436cea2b4e2779c241b016916f083`;
- [ ] an independent reviewer inspects the complete final diff;
- [ ] final independent decision is `APPROVED`.

## 10. Required tests and verification

### Documentation/static checks

```bash
git diff --check 652a499dfdfbaa7594537e0865c41ec617f52dc2...HEAD

grep -RniE \
  '(^|[^[:alnum:]_-])CTRL-(01|02)([^[:digit:]]|$)' \
  docs \
  || true

grep -RniE \
  'CG-(05|06|09)' \
  docs/engineering/repository-map/environments.md \
  docs/engineering/repository-map/repositories \
  || true

grep -Rni 'Publisher Services integration bran[c]h' docs || true

grep -Rni 'NOT YET CREATE[D]' docs || true

grep -Rni 'sph[y]nx' AGENTS.md .github thoth-* docs || true

find . -name AGENTS.md -print | sort
```

Expected:

- no whitespace errors;
- no stale task/control IDs or obsolete Publisher Services integration-branch references;
- no obsolete Sphinx spelling in committed repository documents;
- expected instruction hierarchy present.

### Repository scope

```bash
git diff --name-only 652a499dfdfbaa7594537e0865c41ec617f52dc2...HEAD
```

Only `CHANGELOG.md`, `AGENTS.md` instruction documents and `docs/**` may change.

### CI

Required current-head workflows:

- `check-changelog`;
- `build-test-and-check`;
- `run-migrations`;
- any additional branch-required workflow that runs at the final head.

### Manual review

- verify repository-relative links;
- verify both private design file IDs, revision IDs and modification times through authorized Google Drive access;
- verify issue #765/#766 links;
- verify ADR statuses;
- verify no secrets or production configuration;
- verify the Sphinx branch map against authenticated GitHub evidence.

### Performance

Not applicable.

## 11. Rollout

- initial state after merge: documentation becomes authoritative in `develop`;
- feature flag/configuration: none;
- staging/preview validation: not applicable;
- pilot: not applicable;
- activation approval: CTO merge approval;
- observation: use the controls on the first bounded readiness/implementation tasks and correct defects through normal PRs.

After merge, replace stale ChatGPT Project sources with the merged repository copies.

## 12. Rollback

- code rollback: revert PR #764;
- data rollback: none;
- feature disable: none;
- external side effects: issue links may remain as historical records; update them if the foundation is reverted.

## 13. Stop conditions

Return `BLOCKED` when:

- the private approved designs are inaccessible to the reviewer or do not match their recorded Drive revisions;
- repository state materially differs from the map and cannot be resolved;
- a proposed decision is presented as approved;
- the diff expands beyond documentation and instruction files;
- a required check or CI workflow fails;
- production credentials or destructive access would be required;
- an independent reviewer cannot inspect the exact final head;
- an architecture conflict would need to be silently resolved.

## 14. Expected implementation report

Use:

`docs/engineering/ai-delivery/implementation-reports/CTRL-FOUNDATION-01-implementation-report.md`

The report must record the exact base, implementation/remediation head, commit list, material files, commands/results, CI run IDs, no-runtime-effect statement, known limitations and review focus.

## 15. Recommended execution

Implementation model: GPT-5.6 Thinking control conversation, with repository edits applied manually or by Codex
Reasoning level: high for control consolidation
Independent reviewer: separate Codex or Claude instance
Review reasoning level: high

## 16. Branch and integration plan

- branch source: `develop` at `652a499dfdfbaa7594537e0865c41ec617f52dc2`;
- pull-request target: `develop`;
- programme integration branch: none;
- expected merge order: PR #764 after independent approval and CTO merge authorization;
- branch deletion after merge: YES;
- final programme PR required: NO;
- release path: normal future `develop -> master` release; this PR itself activates no runtime behaviour.

## 17. Approval

Approved for implementation by: Javier Arias, CTO
Date: 2026-07-24
Notes: Documentation-only foundation approved through the CTO control conversation. Cross-programme ADRs remain proposed and require separate approval.
