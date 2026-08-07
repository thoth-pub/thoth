# ADR-01-CLOSEOUT-01 - Post-merge control closeout for ADR-01 / PR #783

Status: APPROVED - PENDING MERGE; IMPLEMENTATION NOT AUTHORIZED
Programme: Publisher Services and Distribution Configuration
Repository: `thoth-pub/thoth`
Workflow: STANDARD
Base branch: `develop`
PR target: `develop`
Programme integration branch: None
Risk: MEDIUM
Owner: CTO
Approved content head: `3229b93c351b68c65b612eb944137bd9f9d2f6e6`
Independent review: `4882033346` - APPROVED
CTO approval: Javi, CTO, 2026-08-07, review `4882035533`
Approval scope: the ADR-01-CLOSEOUT-01 specification content at exact head
`3229b93c351b68c65b612eb944137bd9f9d2f6e6`
Repository authority: pending merge of PR
[#784](https://github.com/thoth-pub/thoth/pull/784) into `develop`
Closeout implementation: NOT AUTHORIZED - requires fresh explicit
implementation authorization from the then-current exact `develop` head after
this specification merges
Specification authoring base: `299b0eff3b9ac10cc0a3a7024ab311ddb135b7eb`
Target branch name: `feature/publisher-services/adr-01-closeout`
Dependencies: merged ADR-01 implementation PR
[#783](https://github.com/thoth-pub/thoth/pull/783)
Master programme issue:
[#765](https://github.com/thoth-pub/thoth/issues/765)

This file specifies the closeout. It does not perform it. The closeout
implementation requires this specification to be independently reviewed,
explicitly CTO-approved and merged, and then requires its own fresh explicit
implementation authorization from the then-current exact `develop` head.

## 1. Objective

Reconcile the repository control records with the merged ADR-01
implementation: record PR #783 as merged into `develop` at merge commit
`299b0eff3b9ac10cc0a3a7024ab311ddb135b7eb`, record ADR-0004 and the final
distribution-platform inventory as repository-authoritative through that
merge, mark ADR-01 delivery and post-merge closeout complete, close CG-07,
remove or historicalize the active pre-merge gate language that the merged
content deliberately left in place, and record accurately that BE-02's ADR-01
dependency is now satisfied while BE-02 itself remains blocked and
unauthorized. This is a documentation and control closeout only. No
substantive architecture, evidence, runtime or production behaviour changes.

## 2. Background and authority

The merged ADR-01 content intentionally left several repository control
records in their pre-merge state, because the merge itself was the event that
made them stale. The merged repository record is therefore internally
inconsistent: it simultaneously contains the approved decision and active
statements that the decision is not yet authoritative.

Known categories of active stale state at the specification authoring base:

- CG-07 in `docs/engineering/repository-map/control-gaps.md` still records
  CG-07 as open and PR #783 as "still a draft";
- the Publisher Services tracker
  (`docs/publisher-services/task-status.md`) still records ADR-01 as
  `IMPLEMENTATION DESIGN APPROVED - DRAFT PR OPEN, NOT MERGED` with active
  remaining gates;
- `docs/publisher-services/README.md` still records the final
  distribution-platform inventory as provisional;
- `docs/publisher-services/decisions.md` and
  `docs/publisher-services/rollout-plan.md` still describe the content
  approval as becoming repository-authoritative only on a future merge;
- `docs/engineering/decisions/decision-register.md` still records ADR-0004 as
  authoritative "only when that PR merges into `develop`";
- the ADR-0004 record, the final inventory and the evidence matrix still
  carry pre-merge approval-state framing;
- the ADR-01 implementation report
  (`docs/engineering/ai-delivery/implementation-reports/ADR-01-implementation-report.md`)
  ends at the pre-merge approval-state gate and lists remaining gates that the
  merge has satisfied.

This list is indicative. The implementation must perform its own exhaustive
search (section 9) rather than treating this list as complete or as an
automatic authorization to edit each path.

### 2.1 Authoritative merge evidence

The closeout implementation must freshly verify and record all of the
following. These are three distinct commits and must not be collapsed into
one SHA.

```text
Implementation PR:              #783
Approved substantive content head:
                                44e6f821535fbee56c830dd6eda237fc6d06fbfd
Substantive independent review: 4881233664 - APPROVED
CTO content approval:           4881279067
Approval-state / control reconciliation head (final PR head):
                                82874c2bfb0c211198252e4f4a0b669d31e14836
Final independent exact-head review:
                                4881832108 - APPROVED
CTO merge authorization:        4881847699
Merge commit:                   299b0eff3b9ac10cc0a3a7024ab311ddb135b7eb
Merged at:                      2026-08-07T10:02:34Z
Target:                         develop
```

The three heads mean different things and the closeout must keep them
distinct:

1. `44e6f821` - the head at which the substantive ADR-0004 and final-inventory
   content was independently reviewed and explicitly CTO-approved. The
   approved architecture content is exactly this head and does not change.
2. `82874c2b` - the head at which approval state and control records were
   reconciled, independently re-reviewed and merge-authorized.
3. `299b0eff` - the merge commit that made the approved content
   repository-authoritative on `develop`.

## 3. Explicit scope

The closeout implementation must:

1. verify `origin/develop` and PR #783 merge evidence freshly, before any
   edit and again before every push;
2. record PR #783 as merged into `develop` at merge commit
   `299b0eff3b9ac10cc0a3a7024ab311ddb135b7eb`, merged
   2026-08-07T10:02:34Z;
3. record ADR-0004 as `APPROVED AND REPOSITORY-AUTHORITATIVE`, with authority
   established through PR #783 and that merge commit, and with the approved
   content head preserved as exactly `44e6f821535fbee56c830dd6eda237fc6d06fbfd`;
4. record the final platform inventory as
   `FINAL INVENTORY APPROVED AND REPOSITORY-AUTHORITATIVE`, with no inventory
   content change;
5. record ADR-01 delivery and post-merge closeout as complete using the
   canonical wording in section 5;
6. transition CG-07 from `OPEN` to `RESOLVED`;
7. perform the exhaustive stale-state search in section 9, classify every
   hit, and remove or historicalize every active pre-merge statement;
8. reconcile the Publisher Services tracker, README, decisions, rollout plan,
   platform inventory, evidence matrix, the engineering decision register and
   the repository control-gap record;
9. extend the ADR-01 implementation record so it does not end at the
   pre-merge gate, in the manner section 7.2 makes authoritative;
10. record BE-02's dependency state exactly as specified in section 5.5;
11. create the closeout implementation report
    (`docs/engineering/ai-delivery/implementation-reports/ADR-01-CLOSEOUT-01-implementation-report.md`);
12. update `CHANGELOG.md`;
13. open one draft PR targeting `develop`, observe automatically triggered CI
    only, and post immutable exact-head evidence.

The implementation must make the smallest internally consistent documentation
change set that achieves the required end state. Locating the active stale
statements comes first; editing follows from what the search finds.

## 4. Non-goals

The closeout implementation must not:

1. change ADR-0004 architecture;
2. change platform inventory entries;
3. add or remove destinations;
4. modify evidence claims, classifications, counts, citations or provenance;
5. modify `docs/publisher-services/adr-01-evidence-ledger.md` in any way;
6. create `DistributionPlatform` runtime code;
7. create migrations;
8. change `schema.rs`;
9. change database models;
10. change GraphQL or API code;
11. change dissemination adapters;
12. change workflows;
13. change `thoth-app`;
14. start BE-02;
15. write the BE-02 implementation;
16. create BE-03 or BE-04 work;
17. use production access;
18. use credentials;
19. dispatch workflows;
20. deploy anything;
21. release anything;
22. execute production migrations;
23. close CG-11;
24. close CG-13;
25. change any cross-programme architecture;
26. edit the immutable evidence comments or descriptions on PR #783;
27. approve, mark ready or merge its own closeout PR.

If a substantive correction to ADR-0004 or to the inventory appears necessary,
the closeout must stop rather than fix it, with:

```text
BLOCKED - ADR-01 CLOSEOUT REQUIRES SUBSTANTIVE ARCHITECTURE CHANGE
```

## 5. Required status transitions

### 5.1 ADR-01

From active pre-merge language such as `IMPLEMENTATION DESIGN APPROVED`,
`DRAFT PR OPEN`, `NOT MERGED`, `MERGE PENDING` and
`POST-MERGE RECONCILIATION REQUIRED`, to the canonical final wording:

```text
MERGED - COMPLETE
```

`MERGED - COMPLETE` is the canonical repository wording for ADR-01 delivery
status and must be applied consistently wherever an active ADR-01 delivery
status is stated. It matches the existing repository precedent for
`ADR-01-SPEC-AMEND-01`.

Supporting prose must make the meaning explicit, in materially these terms:
the ADR-01 implementation design is approved and repository-authoritative
through PR #783, merge commit `299b0eff`, and post-merge closeout is
complete.

ADR-01 must not be described as runtime `IMPLEMENTED` or `PRODUCTION READY`.
ADR-01 was an evidence and architecture-decision task. No runtime
`DistributionPlatform` exists.

The `ADR-01.md` specification record's own specification-level approval
metadata (historical specification approval, corrected-content head
`1276c70a`, review `4873802457`, CTO approval comment `5203642323`,
approval-state head `bdfded20`, PR #781 authority) is historically accurate
and must not be rewritten. What must change in that record is any active
statement that ADR-01 delivery is still pending, plus the addition of the
ADR-01 delivery outcome (PR #783 merged, `MERGED - COMPLETE`).

### 5.2 ADR-0004

```text
APPROVED AND REPOSITORY-AUTHORITATIVE
```

Authority established through PR #783 and merge commit
`299b0eff3b9ac10cc0a3a7024ab311ddb135b7eb`.

Approved content remains exactly `44e6f821535fbee56c830dd6eda237fc6d06fbfd`,
under independent review `4881233664` (`APPROVED`) and CTO content approval
`4881279067`. No ADR-0004 decision, table, exclusion, consequence or
architecture statement changes.

### 5.3 Final platform inventory

```text
FINAL INVENTORY APPROVED AND REPOSITORY-AUTHORITATIVE
```

No inventory content changes. Only status metadata and the active
pre-merge authority prose change.

### 5.4 CG-07

```text
OPEN
↓
RESOLVED
```

CG-07 closes because, and only because, all of the following are true and
recorded:

- ADR-01 finalized the exhaustive platform inventory;
- ADR-0004 is approved;
- the final inventory is approved;
- approval state was recorded;
- the approval-state head `82874c2b` was independently reviewed
  (`4881832108` - APPROVED);
- CTO merge authorization `4881847699` was granted;
- PR #783 merged into `develop` as `299b0eff`;
- the merged control state is reconciled by this closeout.

The CG-07 record must retain its historical narrative. Resolution is recorded
by transition, not by deleting the history of the gap.

### 5.5 BE-02

BE-02 must not become implementation-authorized. Its dependency state becomes
materially:

```text
BLOCKED - ADR-01 DEPENDENCY SATISFIED; OWN APPROVED BOUNDED SPECIFICATION
AND EXPLICIT IMPLEMENTATION AUTHORIZATION REQUIRED
```

The closeout must make all of the following unambiguous:

- the ADR-01 dependency is now satisfied;
- ADR-01 is no longer the reason BE-02 is blocked;
- BE-02 still cannot start;
- BE-02 requires its own approved written bounded specification;
- BE-02 requires fresh explicit implementation authorization from the
  then-current exact `develop` head;
- BE-02 is HIGH risk.

Any active text stating that BE-02 must not finalize `DistributionPlatform`
from a *provisional* inventory must be corrected: the inventory is no longer
provisional. The correction is to the inventory's status, not to BE-02's
authorization state, which remains blocked.

### 5.6 CG-11 and CG-13

```text
CG-11: UNCHANGED
CG-13: OPEN / UNCHANGED
```

No production authority is created by this closeout.

## 6. Invariants

The closeout must preserve the approved ADR-01 substance exactly. The
following evidence counts must remain unchanged:

```text
Included destinations:                   17
Excluded candidates:                     10
Repository-verified claims:              34
Source-owner-confirmed claims:           21
Production-verified claims:              0
Unknown/provisional included values:     0
```

Every approved platform disposition is preserved. Specifically, the closeout
must preserve:

1. `DistributionPlatform` and `MetricPlatform` remain separate.
2. No universal platform enum.
3. No `OTHER`.
4. No fallback distribution destination.
5. Package selection does not imply platform assignment.
6. OAPEN and DOAB remain separate destinations.
7. OAPEN/DOAB remain linked and duplicate-safe.
8. OAPEN/DOAB linkage remains backend-owned.
9. `OCLC_KB` and `EX_LIBRIS_KB` remain separate destinations.
10. `OCLC_KB` and `EX_LIBRIS_KB` continue sharing `OCLC_KBART_PUBLIC`.
11. `JISC_NBK` remains included but inactive, non-assignable and job-free.
12. Push, pull-feed and manual mechanisms remain distinct.
13. Configuration failure continues to fail closed.
14. Empty assignment never broadens processing.
15. Descriptors remain code-owned metadata rather than database rows.
16. Destination remains distinct from adapter/feed profile.
17. The conservative update policy remains unchanged.
18. No automatic withdrawals are introduced.
19. The Thoth-managed automatic dissemination source-file invariant remains
    recorded but unimplemented.
20. The current ProQuest defect remains recorded as current.
21. Project MUSE remains historical/resolved.
22. The evidence-ledger provenance boundary remains unchanged.

`docs/publisher-services/adr-01-evidence-ledger.md` must not be modified.

If the closeout would require any evidence change, stop with:

```text
BLOCKED - ADR-01 CLOSEOUT REQUIRES EVIDENCE CHANGE
```

## 7. Expected implementation scope

### 7.1 Expected paths

The following list is indicative of where active stale pre-merge state is
expected. It is not an automatic authorization to edit every path. The
implementation must first locate all active stale statements (section 9) and
then change only what is required for an internally consistent record.

```text
docs/engineering/ai-delivery/tasks/ADR-01.md
docs/engineering/decisions/ADR-0004-distribution-platform-inventory.md
docs/engineering/decisions/decision-register.md
docs/engineering/repository-map/control-gaps.md
docs/publisher-services/platform-inventory.md
docs/publisher-services/decisions.md
docs/publisher-services/task-status.md
docs/publisher-services/rollout-plan.md
docs/publisher-services/README.md
docs/publisher-services/adr-01-evidence-matrix.md
docs/engineering/ai-delivery/implementation-reports/ADR-01-implementation-report.md
docs/engineering/ai-delivery/implementation-reports/ADR-01-CLOSEOUT-01-implementation-report.md
CHANGELOG.md
```

A path in this list that carries no active stale statement must be left
unchanged, and the implementation report must say so.

### 7.2 Closeout implementation report - authoritative pattern

A dedicated closeout implementation report is authoritative:

```text
docs/engineering/ai-delivery/implementation-reports/ADR-01-CLOSEOUT-01-implementation-report.md
```

This is the repository's established pattern. `P0-01-CLOSEOUT`,
`ADR-0001-POST-MERGE` and `ADR-01-SPEC-AMEND-01-CLOSEOUT-01` each carry their
own report rather than reopening the report of the task they close out. The
reason is that the pre-merge implementation report is an immutable record of
what was delivered and reviewed at the pre-merge heads; rewriting it as
complete would destroy that history and would misrepresent what the
independent reviewers actually reviewed.

The ADR-01 implementation report is therefore not rewritten as complete. It
may receive only a bounded, clearly-marked post-merge addendum that records
the merge outcome and points to the closeout report, and that corrects its
section 17.6 "Remaining gates" list from active gates to satisfied ones
without altering the historical record above it.

## 8. Acceptance criteria

- [ ] `origin/develop` was verified before branch creation and again before
  every push.
- [ ] PR #783 is verified merged.
- [ ] The merge commit is recorded as exactly
  `299b0eff3b9ac10cc0a3a7024ab311ddb135b7eb`.
- [ ] The final PR head is recorded as exactly
  `82874c2bfb0c211198252e4f4a0b669d31e14836`.
- [ ] The approved substantive content head is preserved as exactly
  `44e6f821535fbee56c830dd6eda237fc6d06fbfd` and is not conflated with the
  other two commits.
- [ ] Independent final review `4881832108` is preserved.
- [ ] CTO merge authorization `4881847699` is preserved.
- [ ] Substantive review `4881233664` and CTO content approval `4881279067`
  are preserved.
- [ ] Merged-at `2026-08-07T10:02:34Z` is recorded.
- [ ] ADR-0004 is recorded as approved and repository-authoritative.
- [ ] The final platform inventory is recorded as approved and
  repository-authoritative.
- [ ] ADR-01 is recorded as `MERGED - COMPLETE`, consistently, and is not
  described as runtime implemented or production ready.
- [ ] CG-07 is `RESOLVED`.
- [ ] No active control record states that PR #783 is draft, open,
  merge-pending or awaiting approval.
- [ ] Historical pre-merge statements remain valid where explicitly labelled
  historical.
- [ ] BE-02's ADR-01 dependency is recorded as satisfied.
- [ ] BE-02 remains unauthorized and blocked on its own approved bounded
  specification and explicit implementation authorization.
- [ ] CG-11 remains unchanged.
- [ ] CG-13 remains open and unchanged.
- [ ] No substantive ADR-0004 content changed.
- [ ] No inventory entry changed.
- [ ] No evidence claim, count, citation or provenance boundary changed.
- [ ] `docs/publisher-services/adr-01-evidence-ledger.md` is unchanged.
- [ ] No code, runtime, schema, migration, API, workflow, app or dissemination
  file changed.
- [ ] No credential was used and no production access occurred.
- [ ] All active stale pre-merge status language is removed or
  historicalized.
- [ ] All relative links resolve.
- [ ] `git diff --check` passes.
- [ ] Exact-head CI succeeds.
- [ ] Immutable exact-head evidence is posted.
- [ ] The implementer does not approve its own work.
- [ ] The closeout PR remains draft until independent review.
- [ ] No merge occurs without separate explicit CTO merge authorization.

## 9. Required stale-state search

The implementation must search the complete relevant documentation surface -
not only the expected paths in section 7.1 - for active occurrences of the
following phrases and their equivalents:

```text
PR #783 remains draft
PR #783 is still a draft
draft PR #783
not yet merged
awaiting merge
merge pending
CTO merge authorization required
fresh independent review required
approval-state head pending review
ADR-0004 ... PROPOSED
final inventory ... PROPOSED
final inventory ... PROVISIONAL
CG-07 remains open
CG-07: OPEN
ADR-01 ... DRAFT PR OPEN
ADR-01 ... NOT MERGED
repository-authoritative only on merge
becomes repository-authoritative on merge
remaining gates
```

Every hit must be classified as exactly one of:

```text
ACTIVE STALE STATE
HISTORICAL RECORD - PRESERVE
```

The classification and disposition of every hit must be recorded in the
closeout implementation report.

Global find-and-replace is prohibited. Historical evidence, prior
implementation reports, prior task records and immutable comments describe
states that were true when written and must be preserved. Only *active*
statements about the *current* state may be changed.

## 10. Required validation

- `git diff --check`;
- exact changed-path inspection;
- documentation-only confirmation (no code, schema, migration, API, workflow,
  app or dissemination path in the diff);
- the section 9 stale-state search, re-run after editing;
- relative-link resolution for every link touched or added;
- conflict-marker search;
- placeholder and `TBD` search for newly introduced content;
- sensitive-data validation (no credentials, tokens or private values);
- exact merge-evidence verification against the live PR;
- exact status-transition verification against section 5;
- re-verification of `origin/develop` immediately before each push.

No runtime test suite is required for a documentation-only closeout.
Automatic CI still applies.

Expected CI behaviour:

```text
build-test-and-check
  classify: success
  heavy jobs: skipped

run-migrations
  classify: success
  run_migrations: skipped

publish-to-dockerhub
  classify: success
  docker build/push: skipped

check-changelog
  success
```

Workflows must not be manually dispatched or rerun unless separately
authorized after a failure.

## 11. Rollout and rollback

Rollout:

```text
Documentation/control state only.
No runtime activation.
No schema effect.
No API effect.
No dissemination effect.
No app effect.
No production effect.
```

Rollback:

```text
Revert the closeout documentation PR.
```

A closeout rollback must not revert ADR-0004 substantive content or PR #783
itself unless separately authorized.

## 12. Post-closeout state

```text
ADR-01:
MERGED - COMPLETE

ADR-0004:
APPROVED AND REPOSITORY-AUTHORITATIVE

Final inventory:
FINAL INVENTORY APPROVED AND REPOSITORY-AUTHORITATIVE

CG-07:
RESOLVED

BE-02:
ADR-01 dependency satisfied
but still BLOCKED / NOT AUTHORIZED pending:
- its own approved bounded specification;
- fresh exact implementation base;
- explicit implementation authorization.

CG-11:
UNCHANGED

CG-13:
OPEN / UNCHANGED

Runtime behaviour:
UNCHANGED

Production behaviour:
UNCHANGED
```

## 13. Stop conditions

The implementing agent must stop and report with the exact label:

```text
BLOCKED - ADR-01 CLOSEOUT SPEC BASE MOVED

BLOCKED - ADR-01 MERGE EVIDENCE INCOMPLETE

BLOCKED - ADR-01 FINAL REVIEW EVIDENCE INCOMPLETE

BLOCKED - ADR-01 CTO MERGE AUTHORIZATION INCOMPLETE

BLOCKED - ADR-01 CLOSEOUT ALREADY EXISTS

BLOCKED - ADR-01 CLOSEOUT REQUIRES SUBSTANTIVE ARCHITECTURE CHANGE

BLOCKED - ADR-01 CLOSEOUT REQUIRES EVIDENCE CHANGE

BLOCKED - ADR-01 CLOSEOUT REQUIRES RUNTIME CHANGE

BLOCKED - ADR-01 CLOSEOUT CROSS-PROGRAMME DECISION REQUIRED

BLOCKED - ADR-01 CLOSEOUT CI FAILED
```

If BE-02 readiness cannot be reconciled without changing its architecture or
implementation scope:

```text
BLOCKED - BE-02 READINESS REQUIRES SEPARATE CTO DECISION
```

## 14. Expected implementation report

The agent must use
`docs/engineering/ai-delivery/implementation-report-template.md` and write to
`docs/engineering/ai-delivery/implementation-reports/ADR-01-CLOSEOUT-01-implementation-report.md`,
per section 7.2.

## 15. Recommended execution

```text
Implementation model: implementation-capable engineering agent
Reasoning level: HIGH
Risk: MEDIUM
Independent reviewer: separate model family from the implementer
Review reasoning level: HIGH
```

Reasoning rationale: the change is documentation-only with no runtime or
production effect, but it changes repository-authoritative programme and
control state; the dependency interpretation for BE-02 must be exact; and an
incorrect closeout could prematurely authorize downstream HIGH-risk work.

## 16. Branch and integration plan

- branch source: fresh from the then-current exact `develop` head, verified
  at creation;
- branch name: `feature/publisher-services/adr-01-closeout` (canonical);
- pull-request target: `develop`;
- one task, one branch, one PR;
- programme integration branch: None;
- branch deletion after merge: YES;
- final programme PR required: NO;
- final release path: `develop -> master`.

The implementation branch must be created only after this specification has
been:

1. independently reviewed;
2. explicitly CTO-approved;
3. merged.

Approval or merge of this specification does not itself authorize the
closeout implementation. Fresh explicit implementation authorization from the
then-current exact `develop` head is still required.

## 17. Review and merge gates

For this specification:

1. fresh independent exact-head review;
2. explicit CTO approval;
3. approval-state recording if required by review;
4. separate CTO merge authorization;
5. merge.

For the closeout implementation, afterwards:

6. fresh ADR-01-CLOSEOUT-01 implementation authorization from the
   then-current exact `develop` head;
7. bounded closeout implementation on a draft PR;
8. fresh independent exact-head review of that implementation;
9. separate explicit CTO merge authorization;
10. closeout merge.

The implementing agent must not approve, mark ready or merge its own work at
either stage.
