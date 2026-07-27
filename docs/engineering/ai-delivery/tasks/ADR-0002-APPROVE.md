# ADR-0002-APPROVE - Record approval of Distribution and Metrics Platform Domain Boundaries

Status: APPROVED
Programme: Cross-programme Engineering Control
Affected programmes:

- Publisher Services and Distribution Configuration
- Thoth Metrics

Repository: thoth-pub/thoth
Workflow: STANDARD
Base branch: develop
Verified base commit: `f2e09bd9b138e8ba2ca47a791533f4aae4ffab28`
PR target: develop
Programme integration branch: None
Risk: MEDIUM
Owner: CTO
Approved by: Javi, CTO
Approval date: 2026-07-27
Implementation reasoning: High
Independent review reasoning: High
Target branch name: `feature/engineering/adr-0002-approve`

Dependencies:

- ADR-0002 proposal present in `develop`, introduced by merged PR
  [#764](https://github.com/thoth-pub/thoth/pull/764) as
  `5b406e4ef9b5c192cc38eb8a97a41bbd0fc3bc06`;
- GitHub issue [#765](https://github.com/thoth-pub/thoth/issues/765) OPEN at
  baseline `updatedAt: 2026-07-27T15:50:33Z`;
- GitHub issue [#766](https://github.com/thoth-pub/thoth/issues/766) OPEN at
  baseline `updatedAt: 2026-07-24T17:17:11Z`;
- an independent reviewer who did not implement this task.

## 0. Implementing-agent provenance

This specification is implemented by the actual agent that executed the task, not
a guessed or preselected identity:

- Implementing agent: Claude Code (Anthropic official CLI), acting as the
  implementing engineering context in this session.
- Model: `claude-opus-4-8` (Opus 4.8).
- Implementation reasoning: High.

The independent reviewer must be a fresh non-implementing context with high
reasoning and is preferably a different model family.

## 1. Objective

Record the CTO's approval of `ADR-0002 - Distribution and Metrics Platform Domain
Boundaries` exactly as written, and reconcile every active engineering-control,
Publisher Services and Metrics record with that approval, without amending the
architectural decision, without approving any other decision, and without
enabling any implementation work or producing any runtime effect.

## 2. CTO approval

The exact CTO decision is:

> I approve ADR-0002 - Distribution and Metrics Platform Domain Boundaries as
> written.
> Do not amend the architectural decision.

Approved by: Javi, CTO. Approval date: 2026-07-27.

The approval covers only the recording of approval metadata and the reconciliation
of dependent control records. It does not amend the ADR body, approve any other
ADR, approve the platform inventory, record branch readiness, or make any
implementation task ready.

## 3. Background and authority

Authoritative sources, in precedence order:

1. `thoth-pub/thoth` branch `develop` at
   `f2e09bd9b138e8ba2ca47a791533f4aae4ffab28`;
2. the existing `ADR-0002` body (the authoritative architectural decision);
3. `docs/engineering/decisions/` and the decision register;
4. `docs/publisher-services/` and `docs/metrics/` control records;
5. GitHub issues #765 and #766;
6. repository agent instructions and delivery controls.

Current verified state at specification time:

- `origin/develop` points at `f2e09bd9b138e8ba2ca47a791533f4aae4ffab28`; the
  expected base is unchanged and no intervening commits alter this scope.
- `ADR-0002` is `PROPOSED` (line 3 of the ADR body).
- `ADR-0001` is `PROPOSED`.
- The platform inventory is `VERIFIED BASELINE; FINAL ENUM NOT APPROVED`.
- `MET-CTRL-01` is `CHANGES REQUIRED`.
- Issue #765 is OPEN at `updatedAt: 2026-07-27T15:50:33Z`.
- Issue #766 is OPEN at `updatedAt: 2026-07-24T17:17:11Z`.

## 4. Explicit scope

The task must:

1. Commit this approved task specification first.
2. Update only the approval state and approval record of
   `docs/engineering/decisions/ADR-0002-platform-domain-boundaries.md` to
   `APPROVED`, recording the approver, approval date and an approval note,
   preserving the original proposal date and every other section unchanged.
3. Reconcile the active stale statements about ADR-0002 in:
   - `docs/engineering/decisions/decision-register.md`;
   - `docs/engineering/README.md`;
   - `docs/engineering/repository-map/control-gaps.md`;
   - `docs/publisher-services/README.md`;
   - `docs/publisher-services/decisions.md`;
   - `docs/publisher-services/task-status.md`;
   - `docs/publisher-services/rollout-plan.md`;
   - `docs/metrics/README.md`;
   - `docs/metrics/decisions.md`;
   - `docs/metrics/task-status.md`.
4. Add one bounded `Changed` CHANGELOG entry referencing this approval PR.
5. Create the implementation report
   `docs/engineering/ai-delivery/implementation-reports/ADR-0002-APPROVE-implementation-report.md`.
6. Record the live issue #765 and #766 baselines, complete-body hashes, and the
   exact proposed post-merge synchronization bodies inside the report, without
   writing either issue.

## 5. Non-goals

This task must not:

- change the ADR-0002 architecture (options, recommended option, decision,
  invariants, implementation impact, validation, rollout or rollback);
- approve ADR-0001;
- approve Publisher Services ADR-01;
- approve the distribution-platform inventory;
- record branch readiness;
- modify issue #765 or #766;
- make any implementation task `READY`;
- change Rust, SQL, migrations, GraphQL or generated code;
- change workflows or repository protection;
- deploy, release or activate behaviour;
- create a cross-domain mapping;
- start Publisher Services or Metrics implementation;
- merge its own PR.

## 6. Invariants

The final branch must preserve:

```text
ADR-0001: PROPOSED
ADR-0002: APPROVED only after this PR merges
Publisher Services ADR-01: unapproved
Platform inventory: FINAL ENUM NOT APPROVED
MET-CTRL-01: CHANGES REQUIRED
All Publisher Services implementation tasks: BLOCKED
All Metrics work packages: BLOCKED
No shared platform abstraction or mapping
No runtime effect
```

Approving ADR-0002 removes one dependency. It does not by itself make ADR-01,
BE-02, Metrics WP1 or any other task ready.

## 7. Approved file allowlist

```text
CHANGELOG.md
docs/engineering/README.md
docs/engineering/decisions/ADR-0002-platform-domain-boundaries.md
docs/engineering/decisions/decision-register.md
docs/engineering/repository-map/control-gaps.md
docs/engineering/ai-delivery/tasks/ADR-0002-APPROVE.md
docs/engineering/ai-delivery/implementation-reports/ADR-0002-APPROVE-implementation-report.md
docs/publisher-services/README.md
docs/publisher-services/decisions.md
docs/publisher-services/task-status.md
docs/publisher-services/rollout-plan.md
docs/metrics/README.md
docs/metrics/decisions.md
docs/metrics/task-status.md
```

The actual changed-file set may be a subset of this allowlist. A file is edited
only when it contains an active stale statement or is required evidence. If
another active file must change to make repository state consistent, stop and
report `SCOPE AMENDMENT REQUIRED` with the exact additional file, active stale
statement and proposed correction. Historical evidence (prior task
specifications, implementation reports and review briefs) records the state at
the time it was written and must not be rewritten.

## 8. Acceptance criteria

- [ ] The task specification was committed before any other change.
- [ ] The changed-file set is a subset of the approved allowlist.
- [ ] `docs/engineering/decisions/ADR-0002-platform-domain-boundaries.md` shows
  `Status: APPROVED`, `Approved by: Javi, CTO`, `Approval date: 2026-07-27` and
  the required approval note; the original proposal date is preserved.
- [ ] Only approval metadata changed in the ADR; options, recommended option,
  decision, invariants, implementation impact, validation, rollout and rollback
  are byte-for-byte unchanged.
- [ ] The decision register shows `Last updated: 2026-07-27`, ADR-0002
  `APPROVED` with its blocker recorded as satisfied and the approval PR
  referenced, ADR-0001 remains `PROPOSED`, and the merge/implementation rules are
  unchanged.
- [ ] `docs/engineering/README.md` shows ADR-0002 `APPROVED`, ADR-0001
  `PROPOSED`, records the issue #765 synchronization completed on 2026-07-27 and
  still open, and does not claim either programme is implementation-ready.
- [ ] `docs/engineering/repository-map/control-gaps.md` CG-06 no longer says both
  shared ADRs remain proposed; it records ADR-0002 approved on 2026-07-27 and
  retains ADR-0001 as the unresolved shared-ADR gate; all other gaps retained.
- [ ] `docs/publisher-services/README.md` removes ADR-0002 from active blockers,
  records ADR-0002 approved, retains `BLOCKED FOR IMPLEMENTATION` and the
  ADR-0001, ADR-01/inventory and branch-readiness blockers.
- [ ] `docs/publisher-services/decisions.md` records ADR-0002 as an approved
  shared decision with its exact architectural meaning; ADR-0001 remains proposed.
- [ ] `docs/publisher-services/task-status.md` shows `Last updated: 2026-07-27`,
  ADR-01 `BLOCKED` with the P0-01/ADR-0002 dependencies removed and its blocker
  narrowed to the missing approved bounded specification and final inventory,
  BE-02 `BLOCKED` by ADR-01 without separately listing ADR-0002, the completed
  issue #765 synchronization removed from next actions, ADR-0002 approval
  recorded, ADR-0001 an immediate decision, and no task `READY`.
- [ ] `docs/publisher-services/rollout-plan.md` Stage 0 achieved evidence records
  P0-01 closed, issue #765 synchronized and still open, and ADR-0002 approved on
  2026-07-27; outstanding evidence drops issue #765 sync and ADR-0002 approval,
  retains ADR-0001, ADR-01/inventory, branch readiness and task specs; no later
  stage activated.
- [ ] `docs/metrics/README.md` records ADR-0002 approved and no longer blocking,
  retains `BLOCKED FOR IMPLEMENTATION`, `MET-CTRL-01 CHANGES REQUIRED` and all
  other Metrics blockers.
- [ ] `docs/metrics/decisions.md` moves ADR-0002 from proposed to approved shared
  architecture, records `MetricPlatform` separate from `DistributionPlatform`
  with no initial mapping, ADR-0001 remains proposed, no Metrics-specific
  unresolved decision changed.
- [ ] `docs/metrics/task-status.md` shows `Last updated: 2026-07-27`, ADR-0002
  row `APPROVED` with CTO approval date and approval PR, ADR-0001 `PROPOSED`,
  WP1's generic `ADRs` blocker narrowed to `ADR-0001`, every work package
  `BLOCKED`, `MET-CTRL-01 CHANGES REQUIRED`, no task `READY`, immediate next
  actions recording ADR-0002 achieved.
- [ ] A bounded `Changed` CHANGELOG entry references this PR.
- [ ] The implementation report records base, branch, draft PR, agent, model,
  commits, changed files, the CTO quotation, proof the ADR body was unchanged,
  tracker changes, no-runtime-effect assessment, CI evidence, residual blockers,
  the issue #765/#766 baselines and hashes, and both proposed issue bodies.
- [ ] ADR-0001 remains `PROPOSED`; the platform inventory remains `FINAL ENUM NOT
  APPROVED`; `MET-CTRL-01` remains `CHANGES REQUIRED`.
- [ ] `git diff --check` passes and no runtime file changed.
- [ ] Required final-head CI is green across all seven jobs.
- [ ] Issue #765 remains OPEN at `updatedAt: 2026-07-27T15:50:33Z` and issue #766
  remains OPEN at `updatedAt: 2026-07-24T17:17:11Z` (no write by this task).
- [ ] An independent reviewer returns `APPROVED` before merge.

## 9. Issue synchronization guards

This task does not write to issue #765 or #766. It only records exact proposed
post-merge synchronization bodies in the implementation report.

Both issue writes are deferred and require, in order:

1. this repository PR independently approved and merged;
2. an immediate pre-write re-fetch of the complete live body and `updatedAt`;
3. an exact baseline comparison;
4. a stop on any mismatch;
5. fresh independent review of any regenerated diff;
6. separate explicit CTO authorization.

Proposed minimal issue changes:

- Issue #765: update its synchronization guard baseline to the current reviewed
  body and `updatedAt: 2026-07-27T15:50:33Z`, and change only
  `- [ ] ADR-0002 approved` to `- [x] ADR-0002 approved`; preserve every other
  line and checkbox; keep the issue open.
- Issue #766: add an equivalent forward and rollback synchronization guard based
  on the complete current body and `updatedAt: 2026-07-24T17:17:11Z`, and change
  only `- [ ] ADR-0002 approved` to `- [x] ADR-0002 approved` apart from the
  guard section; preserve every other line and checkbox; keep the issue open.

## 10. Verification

```bash
git diff --check origin/develop...HEAD
git diff --name-only origin/develop...HEAD

git diff --name-only origin/develop...HEAD \
  | grep -Ev '^(CHANGELOG\.md|docs/)' \
  && exit 1 || true

grep -n '^Status: APPROVED$' \
  docs/engineering/decisions/ADR-0002-platform-domain-boundaries.md

grep -n '^Status: PROPOSED$' \
  docs/engineering/decisions/ADR-0001-publisher-package-capability-model.md

grep -n 'FINAL ENUM NOT APPROVED' \
  docs/publisher-services/platform-inventory.md

grep -n 'MET-CTRL-01.*CHANGES REQUIRED' \
  docs/metrics/task-status.md

grep -n 'ADR-0002.*APPROVED' \
  docs/engineering/decisions/decision-register.md \
  docs/metrics/task-status.md

grep -n 'ADR-01.*BLOCKED' \
  docs/publisher-services/task-status.md
```

Required CI jobs at the final head:

```text
build
format_check
lint
test
build_and_push_staging_docker_image
check-changelog
run_migrations
```

All seven jobs must conclude `success` at the exact final head. A
documentation-only no-build path is acceptable but is not a substitute for
independent review.

## 11. Rollout

Initial state after merge:

- ADR-0002 becomes `APPROVED` in the repository, removing one dependency;
- the Publisher Services and Metrics programmes remain blocked for
  implementation;
- issue #765 and #766 synchronization become separately authorized external
  mirrors.

Feature flag or configuration: none. Staging or preview: none. Pilot: none.

Activation approval:

- CTO approval is required to merge this approval PR;
- separate CTO authorization is required for each post-merge issue
  synchronization.

## 12. Rollback

Code rollback: revert this documentation PR; no runtime effect.

Issue rollback (guarded; no unconditional full-body overwrite): re-fetch the
complete live body and `updatedAt`, compare against the expected state, stop on
any mismatch, generate a minimal reversal preserving unrelated edits, obtain
fresh independent review and explicit CTO authorization, and apply only the
reviewed minimal reversal. Never restore an old complete snapshot blindly.

## 13. Prohibition on implementation work

This is a documentation and control-record task only. The implementing agent must
not write Rust, SQL, migrations, GraphQL or generated code, must not change
workflows or repository protection, must not deploy, release or activate
behaviour, must not create any cross-domain mapping, and must not start Publisher
Services or Metrics implementation. If reconciling repository state would require
any such change, stop and report `BLOCKED`.

## 14. Independent-review requirement

The implementing context is ineligible to approve this work. A fresh
non-implementing reviewer with high reasoning must inspect the exact final head,
all commits in order, the complete cumulative diff, ADR-0002 before and after,
proof that only approval metadata changed in the ADR, all Publisher Services and
Metrics tracker changes, all remaining blockers, exact-head CI, the complete
issue #765 and #766 baselines, both proposed issue bodies, and the absence of
runtime changes. The reviewer must return exactly one verdict: `APPROVED`,
`CHANGES REQUIRED` or `BLOCKED`. Approval is permitted only when there is no
unresolved P0 or P1 finding. The reviewer must not mark the PR ready, merge it or
edit either issue.

## 15. Expected implementation report

The agent must use:

`docs/engineering/ai-delivery/implementation-report-template.md`

and record the concrete evidence required by Section 4 and Section 6 of this
specification.

## 16. Approval

Approved for implementation by: Javi, CTO. Date: 2026-07-27.

Notes:

- Documentation and control-record changes only.
- This approval records the CTO's approval of ADR-0002 as written and does not
  amend the architectural decision.
- This approval does not approve ADR-0001, the platform inventory, branch
  readiness, any implementation task, migration, workflow, deployment, release or
  production activation.
- This approval does not authorize any write to issue #765 or #766; each is a
  separately reviewed and separately authorized step after this PR merges.
- The implementing agent may create the task branch, commit, push and open a
  draft PR, but may not approve or merge it.
