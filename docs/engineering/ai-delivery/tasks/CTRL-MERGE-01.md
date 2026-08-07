# CTRL-MERGE-01 - Terminal merge evidence and non-recursive closeout

Programme: Shared Engineering Control
Repository: `thoth-pub/thoth`
Task ID: `CTRL-MERGE-01`
Workflow: STANDARD
Base branch: `develop`
Authorized base commit: `d534ff52b8bda966ee5cfa7a1d03353ef59476ec`
PR target: `develop`
Programme integration branch: None
Implementation branch: `feature/shared-control/ctrl-merge-01`
Risk: MEDIUM
Reasoning: HIGH
Owner: CTO
Authoritative specification: [issue #786](https://github.com/thoth-pub/thoth/issues/786)
Dependencies: None
Production/runtime effect: NONE

Authority condition: this record is repository-authoritative when this exact
content is reachable from `develop`. Live review, authorization and merge
evidence is the GitHub pull-request record for the implementing pull request.

## 1. Objective

Replace the recursive approval-state and post-merge-closeout convention with a
repository-authoritative terminal merge evidence rule, so that the GitHub
review, authorization, CI and merge record is accepted as terminal lifecycle
evidence, without weakening any substantive engineering control.

## 2. Background and authority

Authoritative sources:

- [issue #786](https://github.com/thoth-pub/thoth/issues/786) - approved written
  scope for this control task;
- [ADR-0005](../../decisions/ADR-0005-terminal-merge-evidence.md) - the decision
  this task records;
- `docs/engineering/ai-delivery/operating-model.md`;
- `docs/engineering/ai-delivery/release-gates.md`;
- `docs/engineering/AGENTS.md`;
- root `AGENTS.md`.

Gate 1 of the operating model accepts a specification that is "committed or
attached to an authoritative GitHub issue", and root `AGENTS.md` section 1
accepts a GitHub issue that carries the information required by
`task-specification-template.md`. Issue #786 carries that information, so this
task does not require a separately merged specification pull request. This
record is the canonical repository copy.

Current behaviour: no repository control rule requires an approval-state-only
commit or an automatic post-merge closeout pull request. The recursion is
established practice rather than written control. Committed documents carry
transient status such as `PENDING MERGE` and `becomes repository-authoritative
on merge`; review and authorization identifiers are then copied into those
files, which moves the head and invalidates the exact-head review that justified
the copy; after merge the transient prose is stale and prompts a further
closeout pull request.

## 3. Explicit scope

The task must:

1. add this canonical `CTRL-MERGE-01` task record;
2. add `ADR-0005 - Terminal merge evidence and non-recursive closeout` as a
   Shared Engineering Control decision;
3. record `ADR-0005` in the decision register;
4. update `operating-model.md` so the lifecycle recognizes terminal merge
   evidence and prohibits approval-state-only commits;
5. update `release-gates.md` so the merge-ready and closure gates use guarded
   merge and terminal evidence;
6. update `docs/engineering/AGENTS.md` with durable-versus-transient
   status-writing guidance;
7. update the task specification, decision record and independent review
   templates so they do not require transient lifecycle status in committed
   files;
8. update `docs/engineering/ai-delivery/README.md` for the new documents;
9. update `CHANGELOG.md`.

## 4. Non-goals

The task must not:

1. change runtime code, database schema, migrations, APIs, GraphQL,
   dissemination or `thoth-app`;
2. change deployment, release automation or CI workflow code;
3. change GitHub branch protection, repository settings or auto-merge;
4. reduce review independence or permit implementer self-approval;
5. authorize `BE-02` or any other blocked implementation task;
6. change Publisher Services or Thoth Metrics architecture;
7. access production or use credentials;
8. rewrite historical pull requests, reviews, evidence comments or
   implementation reports;
9. retrospectively modify merged pull-request descriptions or comments;
10. repair unrelated historical stale state, including the `ADR-0003` /
    PR [#778](https://github.com/thoth-pub/thoth/pull/778) wording, unless issue
    #786 is explicitly extended by the CTO.

## 5. Invariants

The change must preserve every control listed in
[ADR-0005](../../decisions/ADR-0005-terminal-merge-evidence.md) section 10, in
particular:

1. one bounded task per branch and pull request;
2. approved written specifications, explicit scope, non-goals, invariants,
   acceptance criteria, migration, rollout and rollback requirements;
3. independent substantive review, with actual diff, test/CI, migration and
   authorization inspection;
4. no implementer self-approval and no self-merge;
5. exact-head review binding, and stop when the reviewed head changes;
6. guarded merge against the expected reviewed head;
7. CTO merge authorization for HIGH and CRITICAL risk, and wherever a
   specification or control explicitly requires it;
8. explicit, separate production activation, deployment and release
   authorization;
9. feature flags, comparison mode, pilots, monitoring, rollback and observation
   where applicable;
10. cross-programme ADR and CTO escalation;
11. material post-merge corrections still require a bounded task and pull
    request;
12. missing evidence is missing work.

## 6. Required behaviour

### 6.1 Success behaviour

The updated controls state that the GitHub merge event and merge commit are
terminal task evidence, that approval-state-only commits are prohibited when
their sole purpose is metadata copying, and that no closeout pull request is
required merely to record a merge.

### 6.2 Failure behaviour

Where a merge reveals materially incorrect repository content, a bounded
corrective task and pull request is still required, under normal review and
authorization controls.

### 6.3 Authorization

Unchanged. The task alters how lifecycle evidence is recorded, not who may
approve, merge, deploy or activate.

### 6.4 Concurrency and idempotency

Not applicable.

### 6.5 Compatibility

Historical records remain valid as written and are not rewritten. Tasks in
flight when this merges may complete under either form.

## 7. Data and migration requirements

Migration required: NO

## 8. Observability and operations

Required logs: none.

Required metrics/alerts: none.

Operational runbook changes: agents and reviewers follow the updated lifecycle
in `operating-model.md` section 5 and the updated merge-ready gate in
`release-gates.md` section 1.

## 9. Acceptance criteria

- [ ] Issue #786, the authorized base and ADR numbering verified before editing.
- [ ] No equivalent implementation already exists.
- [ ] Canonical `CTRL-MERGE-01` task record added.
- [ ] Shared Engineering Control ADR added.
- [ ] Decision register updated.
- [ ] Canonical delivery and control workflow updated.
- [ ] Relevant templates updated.
- [ ] The terminal merge evidence rule is explicit.
- [ ] GitHub review, authorization, CI and merge records are recognized as
      authoritative lifecycle evidence.
- [ ] Approval-state-only commits are prohibited when their only purpose is
      metadata copying.
- [ ] Automatic post-merge closeout pull requests are not required merely to
      record a merge.
- [ ] Durable-versus-transient status guidance is explicit.
- [ ] Exact-head review remains mandatory, and head changes invalidate prior
      exact-head review.
- [ ] Implementer cannot self-approve.
- [ ] HIGH-risk CTO merge authorization remains mandatory, and explicitly gated
      MEDIUM/LOW merge authorization remains mandatory.
- [ ] Production activation remains separately controlled.
- [ ] Genuine material post-merge corrections still require bounded pull
      requests.
- [ ] No programme-specific architecture changed.
- [ ] No runtime, code, schema, migration, API, workflow or deployment change.
- [ ] `git diff --check` passes and links resolve.
- [ ] No sensitive information introduced.
- [ ] Documentation-only CI succeeds.
- [ ] Fresh independent exact-head review completed.
- [ ] Explicit CTO merge authorization obtained for this pull request.

## 10. Required tests

Unit, integration/database, authorization/security, regression, performance: not
applicable; this task changes documentation and control records only.

Manual verification:

- `git diff --check` passes;
- `git diff --stat` shows documentation and control paths only;
- relative links resolve;
- no contradictory lifecycle rule remains;
- no wording permits implementer self-approval;
- no wording weakens HIGH-risk merge control;
- no wording implies merge equals production activation;
- no transient status text is introduced that merge would immediately falsify.

## 11. Rollout

- initial state after merge: the new lifecycle is authoritative for tasks
  starting after merge; tasks in flight may complete under either form;
- feature flag/configuration: not applicable;
- staging/preview validation: not applicable;
- pilot: not applicable;
- activation approval: not applicable; there is no runtime effect;
- observation period: not applicable.

## 12. Rollback

- code rollback: revert the documentation pull request; the prior convention
  returns;
- data rollback or forward repair: not applicable;
- feature disable/kill switch: not applicable;
- external side-effect handling: none.

## 13. Stop conditions

The implementing agent must stop and report `BLOCKED` if:

- `origin/develop` has moved from the authorized base
  (`BLOCKED - CTRL-MERGE-01 BASE MOVED`);
- issue #786 has materially changed
  (`BLOCKED - CTRL-MERGE-01 SPECIFICATION DRIFT`);
- an equivalent implementation already exists
  (`BLOCKED - CTRL-MERGE-01 ALREADY EXISTS`);
- current authoritative control explicitly requires an approval-state-only
  commit or an automatic post-merge closeout pull request for this change
  (`BLOCKED - CURRENT CONTROL REQUIRES RECURSIVE CLOSEOUT`);
- the solution would require runtime, CI, settings or branch-protection changes
  (`BLOCKED - CTRL-MERGE-01 REQUIRES OUT-OF-SCOPE IMPLEMENTATION`);
- approved architecture would need to change;
- scope cannot be completed without unrelated changes.

## 14. Expected implementation report

The agent must use
[`implementation-report-template.md`](../implementation-report-template.md), as
required by `operating-model.md` Gate 2 step 8. The report is delivered at
[`implementation-reports/CTRL-MERGE-01-implementation-report.md`](../implementation-reports/CTRL-MERGE-01-implementation-report.md).

An implementation report is substantive implementation evidence, not
approval-state metadata, and remains required under
[ADR-0005](../../decisions/ADR-0005-terminal-merge-evidence.md). The report
references the pull-request record for the exact head, review, authorization, CI
and merge facts rather than transcribing them, so that no second commit is
needed to complete it.

## 15. Recommended execution

Implementation model: Claude Opus
Reasoning level: HIGH
Independent reviewer: a different agent instance or model family
Review reasoning level: HIGH

## 16. Branch and integration plan

- branch source: `develop` at `d534ff52b8bda966ee5cfa7a1d03353ef59476ec`;
- pull-request target: `develop`;
- expected merge order: independent, no dependencies;
- parent programme branch refresh requirement: not applicable;
- branch deletion after merge: YES;
- final programme PR required: NO;
- final release path: `develop -> master`.

## 17. Review and merge gates

- fresh independent exact-head review of the final head is required;
- the implementing agent may not approve or merge this work;
- explicit CTO merge authorization is required for this Shared Engineering
  Control pull request, bound to the independently reviewed exact head;
- merge is guarded by that expected head;
- the policy is not authoritative until this pull request merges into `develop`;
- the unmerged ADR may not be used as authority to waive any existing
  repository-authoritative control.
