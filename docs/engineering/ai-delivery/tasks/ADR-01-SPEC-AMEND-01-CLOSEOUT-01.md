# ADR-01-SPEC-AMEND-01-CLOSEOUT-01 - Post-merge control closeout for the ADR-01 specification amendment

Status: DRAFTED - PENDING INDEPENDENT REVIEW AND MERGE
Programme: Publisher Services and Distribution Configuration
Repository: `thoth-pub/thoth`
Workflow: STANDARD
Base branch: `develop`
PR target: `develop`
Programme integration branch: None
Risk: MEDIUM
Owner: CTO
Authorized exact base: `a511e01c83c5e805a75e0fdaeb3b5297c39ef291`
Target branch name: `feature/publisher-services/adr-01-spec-amend-01-closeout`
Dependencies: merged amendment PR
[#781](https://github.com/thoth-pub/thoth/pull/781)

## 1. Objective

Reconcile the repository control records with the merged
`ADR-01-SPEC-AMEND-01` amendment: record PR #781 as merged and the corrected
ADR-01 specification as repository-authoritative, preserve the complete
approval and review history, keep every remaining programme gate intact, and
safely remove the obsolete local pre-amendment ADR-01 branch. This is a
documentation and control closeout only.

## 2. Authoritative merge evidence

```text
Amendment PR: #781
Amendment head: bdfded20e8cac65fcd7713b07d189052e0eba745
Merge commit: a511e01c83c5e805a75e0fdaeb3b5297c39ef291
Merged at: 2026-08-06T11:29:53Z

Approved substantive content head: 1276c70a81e73f57d833eecb0e6886bd0cabf69e
Substantive independent review: 4873802457 - APPROVED
Approval-state final independent review: 4874093991 - APPROVED
CTO corrected-content approval: PR #781 comment 5203642323 (2026-08-06)
CTO merge authorization: review 4874128610
```

## 3. Explicit scope

The task must:

1. create this bounded closeout task record;
2. record `ADR-01-SPEC-AMEND-01` as `MERGED - COMPLETE` in its task record;
3. update `docs/engineering/ai-delivery/tasks/ADR-01.md` status and
   repository-authority metadata to
   `APPROVED AND REPOSITORY-AUTHORITATIVE - FRESH IMPLEMENTATION
   AUTHORIZATION REQUIRED`;
4. reconcile the active post-merge status in the Publisher Services README,
   decisions, platform inventory, rollout plan, task tracker and CG-07;
5. create the closeout implementation report;
6. update `CHANGELOG.md`;
7. delete the obsolete local pre-amendment `feature/publisher-services/adr-01`
   branch only after every safeguard in section 5 passes;
8. open one draft closeout PR, observe automatically triggered CI only, and
   post immutable exact-head evidence.

## 4. Non-goals

The task must not:

1. implement ADR-01 or authorize its implementation;
2. create ADR-0004;
3. start or unblock BE-02;
4. change any substantive ADR-01 requirement, invariant, platform
   disposition, evidence conclusion or acceptance criterion;
5. edit `docs/publisher-services/adr-01-evidence-ledger.md`;
6. change any runtime, migration, schema, GraphQL, API or workflow file;
7. edit `thoth-app` or `thoth-dissemination`;
8. use credentials or production access;
9. dispatch or rerun workflows;
10. delete any remote branch;
11. edit PR #781's immutable comments or description;
12. approve, mark ready or merge the closeout PR;
13. deploy or release anything.

## 5. Obsolete-branch cleanup safeguards

The local pre-amendment branch `feature/publisher-services/adr-01` may be
deleted locally only if all of the following are freshly verified:

- it is not currently checked out;
- it is not attached to another worktree;
- it points exactly to `590ff437bbd25b8aa5fde800dd8a38772b7e453e`;
- it contains no commits beyond that base;
- it has no remote tracking branch;
- no open pull request references it;
- it contains no unmerged or unique work.

If any condition fails the task stops with
`BLOCKED - OBSOLETE ADR-01 BRANCH NOT SAFE TO DELETE`. No remote branch is
deleted under any circumstances.

## 6. Required status transitions

```text
ADR-01-SPEC-AMEND-01: MERGED - COMPLETE

ADR-01 specification: APPROVED AND REPOSITORY-AUTHORITATIVE -
  FRESH IMPLEMENTATION AUTHORIZATION REQUIRED

ADR-01 implementation: NOT STARTED; requires a fresh task authorization
  from the then-current exact develop head

ADR-0004: NOT STARTED
Final platform inventory: PROVISIONAL
BE-02: BLOCKED
CG-07: OPEN (amendment complete; ADR-01, ADR-0004 and the final inventory
  are not)
CG-11: UNCHANGED
CG-13: UNCHANGED
```

No active text may continue to describe PR #781 as awaiting merge, merge
authorization as pending, or the corrected content as merge-pending.
Historical passages may retain earlier states when explicitly labelled
historical.

## 7. Acceptance criteria

- [ ] `origin/develop` was exactly `a511e01c83c5e805a75e0fdaeb3b5297c39ef291`
  at branch creation and before every push.
- [ ] PR #781 merge evidence is recorded accurately.
- [ ] All active `MERGE PENDING` status for the amendment is removed or
  historicalized.
- [ ] ADR-01 is recorded as approved and repository-authoritative; its
  implementation remains unauthorized.
- [ ] ADR-0004 remains not started; the final inventory remains provisional;
  BE-02 remains blocked; CG-07 remains open; CG-11 and CG-13 unchanged.
- [ ] No substantive ADR-01 content changed; the evidence ledger is
  untouched.
- [ ] No runtime, migration, schema, API, app or dissemination file changed.
- [ ] The obsolete local branch is safely deleted (or its absence recorded),
  with evidence.
- [ ] All relative links resolve; `git diff --check` passes; no conflict
  markers, placeholders or sensitive values are introduced.
- [ ] The closeout PR remains draft; no implementer self-approval or merge
  occurs.

## 8. Validation

- `git diff --check`;
- documentation-only changed-path confirmation;
- relative-link resolution;
- post-merge automatic workflow evidence for `a511e01c` recorded without
  dispatching anything;
- exact-head CI observation for the closeout head (documentation-only
  classifier behaviour expected);
- re-verification of `origin/develop` immediately before each push.

## 9. Rollout and rollback

- Initial state after merge: documentation and control records only; no
  runtime effect.
- Rollback: revert the closeout documentation PR; no operational effect.

## 10. Review and merge gates

1. fresh independent exact-head review of the closeout PR;
2. explicit CTO approval of the closeout records;
3. approval-state recording if required by review;
4. separate CTO merge authorization;
5. merge of the closeout PR;
6. fresh ADR-01 implementation authorization from the then-current exact
   `develop` head.

The implementing agent must not approve or merge its own closeout.

## 11. Stop conditions

Stop with a precise blocker if:

- `develop` moved from the authorized base;
- PR #781 merge evidence cannot be verified;
- the final independent approval or CTO merge authorization cannot be
  verified;
- an equivalent closeout task already exists;
- the obsolete local ADR-01 branch is not safe to delete;
- a substantive ADR-01 correction appears necessary;
- the evidence ledger would need modification;
- runtime or related-repository changes appear necessary;
- automatic CI fails.
