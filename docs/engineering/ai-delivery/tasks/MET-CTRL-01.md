# MET-CTRL-01 - Reconcile Metrics programme controls and open the Thoth WP1 gate

Status: APPROVED
Programme: Thoth Metrics
Stage: Stage 0 - Control/readiness
Owning GitHub issue: [thoth-pub/thoth#832](https://github.com/thoth-pub/thoth/issues/832)
Repository: thoth-pub/thoth
Workflow: STANDARD
Base branch: `develop`
Exact authorized base commit: `250554dd7351c97af46d59b5033abd391d9eec16`
PR target: `develop`
Programme integration branch: None
Risk: LOW
Owner: CTO
Approved by: CTO, through the amended issue #832 specification and its fresh
independent non-implementing specification review (`APPROVED`)
Dependencies: parent programme issue
[#766](https://github.com/thoth-pub/thoth/issues/766); Publisher Services ->
Metrics pivot (recorded in #766 comment `5412873595`); merged ADR-0001
(PR [#772](https://github.com/thoth-pub/thoth/pull/772)), ADR-0002
(PR [#769](https://github.com/thoth-pub/thoth/pull/769)), ADR-0003
(PR [#778](https://github.com/thoth-pub/thoth/pull/778), merge commit
`37b802776ae6853affe19d90156f3c1e0654ebe3`) and ADR-0008
Target branch name: `feature/metrics-control/met-ctrl-01`

Authority condition: this record is repository-authoritative when this exact
content is reachable from `develop`. Live review, authorization and merge
evidence is the GitHub pull-request record.

If the verified current head of `develop` differs from the exact authorized
base commit above, implementation must not silently rebase the authorization:
return `HOLD - AUTHORIZED BASE MOVED` with the observed SHA.

## 1. Objective

Reconcile the repository-backed Thoth Metrics programme controls with the
actual live state after subsequent shared-architecture and Publisher Services
work, so that the remaining gates for beginning the first bounded Thoth WP1
slice are accurate, durable and independently reviewable. The task is
control/documentation work only.

## 2. Background and authority

Authoritative sources:

- issue [#832](https://github.com/thoth-pub/thoth/issues/832) (complete
  current body, including the 2026-08-25 specification amendment);
- parent programme issue [#766](https://github.com/thoth-pub/thoth/issues/766);
- approved private Metrics technical design, Drive revision `6`;
- [ADR-0001](../../decisions/ADR-0001-publisher-package-capability-model.md),
  [ADR-0002](../../decisions/ADR-0002-platform-domain-boundaries.md),
  [ADR-0003](../../decisions/ADR-0003-repository-authoritative-schema-contract.md)
  and
  [ADR-0008](../../decisions/ADR-0008-machine-roles-and-durable-job-primitives.md),
  all `APPROVED` and repository-authoritative;
- the current Metrics programme documents under `docs/metrics/` and the
  engineering-control documents under `docs/engineering/`.

Current behaviour (authoritative reconstruction baseline at task creation and
amendment review):

- Publisher Services `MIG-01` is complete and the Publisher Services ->
  Metrics pivot is durably recorded in #766 comment `5412873595`;
- `thoth/develop` is `250554dd7351c97af46d59b5033abd391d9eec16`;
- `thoth/master` is `40e9c06d4ab76217c3ef277dd539d3b5580e2bb8`;
- no repository-local `feature/metrics` branch exists;
- ADR-0001 is `APPROVED` and merged through PR #772 as
  `b2c91ff25b95ab0e10a477ff21dbd4702f5db8d4`;
- ADR-0002 is `APPROVED` and merged through PR #769 as
  `e124221f8444bd738228f1b609c536639be8789e`;
- ADR-0003 is `APPROVED` and repository-authoritative through merged PR #778,
  merge commit `37b802776ae6853affe19d90156f3c1e0654ebe3`;
- `THOTH-DB-CTRL-01` is superseded; `THOTH-DB-CTRL-02`/ADR-0003 is the current
  schema-control authority;
- ADR-0008 is `APPROVED` and repository-authoritative; it establishes the
  shared domain-specific machine-role convention but deliberately does not
  select Metrics role names, entitlement models, credential models or
  operation matrices;
- `thoth-sphinx` remains separately blocked on `BR-SPHINX-01` and
  `SPHINX-BOOT-01`; that blocks Sphinx/WP6 work but is not by itself a reason
  to keep Thoth WP1 blocked once Thoth-local control gates are satisfied;
- client branch normalization remains repository-local future work for the
  work packages that depend on those clients;
- source fixtures, COUNTER mappings and OPERAS completeness remain
  source-specific/later-work-package gates;
- several active Metrics and engineering-control documents still record the
  pre-merge/pre-pivot state and require reconciliation.

Where the old #766 body or current Metrics/control documents conflict with the
live merged state above, merged repository/PR evidence and current approved
ADRs control.

## 3. Explicit scope

The task must:

1. create this task specification and an implementation report at
   `docs/engineering/ai-delivery/implementation-reports/MET-CTRL-01-implementation-report.md`;
2. reconcile `docs/metrics/README.md` with the current authoritative state
   (pivot complete; ADR-0001/0002/0003/0008 authority; Diesel/schema-control
   blocker resolved through ADR-0003/`THOTH-DB-CTRL-02`; Sphinx and client
   readiness retained as owning-repository/later-work-package gates; Metrics
   implementation remains unauthorized);
3. reconcile `docs/metrics/task-status.md` (`MET-CTRL-01` active;
   `THOTH-DB-CTRL-01` `SUPERSEDED`; `THOTH-DB-CTRL-02`/ADR-0003 merged and
   repository-authoritative; ADR-0001/0002 approved/merged; ADR-0008 bounded
   to its actual shared conventions; WP1 blocked only on the controls that
   actually apply to a bounded Thoth WP1 slice; Sphinx/client/source-specific
   blockers attached to their owning later work packages);
4. reconcile `docs/metrics/decisions.md` only where current shared-decision
   status is stale or ambiguous, without inventing new architecture;
5. reconcile `docs/metrics/rollout-plan.md` Stage 0 to distinguish completed
   shared/global controls, the remaining Thoth-local WP1 entry gate,
   Sphinx/WP6 readiness, client-specific readiness, source/driver-specific
   readiness and WP5 service-role work;
6. reconcile the `docs/metrics/contract-register.md` service-role section so
   it no longer implies the entire machine-role architecture is undecided
   while keeping exact Metrics role codes/permissions an unapproved WP5-owned
   bounded decision;
7. review `docs/metrics/master-issue.md` and edit it only if a concrete
   active stale statement must change for consistency;
8. reconcile `docs/engineering/repository-map/control-gaps.md` CG-08 so it
   distinguishes Thoth WP1 entry from later Sphinx/client/source/WP5 gates and
   no longer names resolved controls as open prerequisites, without weakening
   CG-03, CG-04, CG-09, CG-10, CG-11 or CG-13;
9. reconcile `docs/engineering/README.md` only where active Metrics readiness
   language presents resolved Diesel/schema control or later
   repository-specific readiness as blanket implementation blockers;
10. reconcile `docs/engineering/repository-map/repositories/thoth.md` only
    where ADR-0003/`THOTH-DB-CTRL-02`/PR #778 is still written as
    prospectively pending merge;
11. reconcile `docs/engineering/decisions/decision-register.md` only where
    ADR-0003's authority text is temporally stale, recording PR #778 as
    merged with exact merge commit `37b802776ae6853affe19d90156f3c1e0654ebe3`;
12. add one bounded `CHANGELOG.md` entry;
13. record the exact proposed post-merge #766 synchronization comment in the
    implementation report without posting it;
14. document the immediate post-MET-CTRL sequence for Thoth: verify fresh
    `develop` head; separately authorize creation of repository-local
    `feature/metrics` from that exact head; create/approve one bounded
    repository-local WP1 child issue/specification; implement that slice on a
    child branch targeting `feature/metrics`.

## 4. Non-goals

The task must not:

1. create `feature/metrics`;
2. create or implement a WP1 schema slice, SQL migration, `schema.rs` change,
   Rust, GraphQL, generated-client or API behaviour change;
3. modify workflows, CI, branch protection or repository settings;
4. modify `thoth-sphinx`, `metrics-dashboard`, `metrics-widget` or
   `thoth-app`;
5. perform `BR-SPHINX-01`, `SPHINX-BOOT-01`, client branch normalization or
   source-fixture collection;
6. choose exact Metrics service-role names, entitlement rules, credentials or
   operation matrices;
7. introduce or reuse a generic cross-programme job framework;
8. approve COUNTER mappings, source mappings, OPERAS completeness or
   CloudFront source fixtures;
9. run migrations, deployment, release, provider actions, production
   reads/writes or activation;
10. mark WP1 implementation complete or authorized merely because programme
    controls are reconciled;
11. weaken unrelated open engineering control gaps merely to make Metrics
    appear ready;
12. rewrite historical point-in-time task records or implementation reports
    (for example `THOTH-DB-CTRL-02.md`) merely because their original wording
    is now past; only active current-state controls are reconciled.

## 5. Cross-repository impact

Affected contracts:

- database/domain model: NOT AFFECTED
- GraphQL/API schema and behaviour: NOT AFFECTED
- generated clients/types: NOT AFFECTED
- authorization semantics: NOT AFFECTED
- export formats: NOT AFFECTED
- configuration/environment contracts: NOT AFFECTED
- event/job payloads: NOT AFFECTED
- dissemination/platform behaviour: NOT AFFECTED
- UI assumptions: NOT AFFECTED
- CMS/site contracts: NOT AFFECTED
- package/library interfaces: NOT AFFECTED
- deployment/compatibility windows: NOT AFFECTED

Every contract is `NOT AFFECTED`: the task changes repository documentation
and control records only.

## 6. Invariants

The implementation must preserve:

1. Thoth remains the sole canonical Metrics datastore/API owner; Sphinx
   remains orchestration/interoperability.
2. `MetricPlatform` remains a separate domain from `DistributionPlatform`; no
   name-based conversion or inferred mapping is introduced.
3. Metrics code consumes package capabilities rather than hardcoded package
   names.
4. ADR-0003 remains the schema-control authority: schema-bearing tasks
   atomically update migrations, `thoth-api/src/schema.rs`, affected
   models/code and tests; no Diesel CLI/generated-schema authority is
   introduced.
5. ADR-0008 remains bounded to shared machine-role/durable-job conventions;
   Metrics exact roles and permissions remain WP5-owned.
6. Sphinx bootstrap/readiness remains a separate repository-local dependency
   for WP6 and later Sphinx work.
7. Client branch readiness remains a repository-local dependency for the work
   packages that use those clients.
8. Source fixtures/mappings remain source-specific gates and are not falsely
   marked complete.
9. CG-03, CG-04, CG-09, CG-10, CG-11 and CG-13 remain accurate and are not
   weakened by this control reconciliation.
10. No runtime or production behaviour changes.
11. The implementing agent may not independently approve its own work.

## 7. Required behaviour

### 7.1 Success behaviour

After this task merges, the active Metrics and engineering-control documents
accurately record: the completed shared controls (ADR-0001/0002/0003/0008,
resolved Diesel/schema control, closed foundation); the remaining Thoth-local
WP1 entry gate (MET-CTRL-01 closure, separately authorized `feature/metrics`
creation, one approved bounded WP1 child specification); and the later
Sphinx/WP6, client, source-specific and WP5 gates attached to their owning
work packages. No stale active statement presents a resolved control as open
or an unresolved control as resolved.

### 7.2 Failure behaviour

Any discovery that an out-of-budget active file must change, that a shared ADR
no longer matches the approved reconstruction, or that reconciling CG-08 would
require weakening another open control gap is a stop condition (section 17),
not a licence to widen scope.

### 7.3 Authorization

Specification approval is not merge, deployment, migration or production
authorization. Merge requires fresh independent exact-head review and explicit
CTO merge authorization. #766 synchronization, `feature/metrics` creation and
WP1 implementation each remain separately authorized actions.

### 7.4 Concurrency and idempotency

Not applicable (documentation-only change).

### 7.5 Compatibility

Documentation/control only. No API, database, client or deployment
compatibility effect.

## 8. Data and migration requirements

Migration required: NO

## 9. Observability and operations

Required logs: none (documentation-only).

Required metrics/alerts: none.

Operational runbook changes: none.

## 10. Acceptance criteria

- [ ] Task starts from the exact authorized `develop` base
      `250554dd7351c97af46d59b5033abd391d9eec16`; HOLD if it moved before
      branch creation.
- [ ] Changed files are a subset of the amended approved write budget
      (section 14).
- [ ] No Rust, SQL, migration, workflow, generated-contract,
      branch-protection or runtime file changes.
- [ ] Metrics docs no longer describe the Diesel/schema-control procedure as
      unresolved.
- [ ] `THOTH-DB-CTRL-01` is consistently `SUPERSEDED` and
      `THOTH-DB-CTRL-02`/ADR-0003 is consistently repository-authoritative.
- [ ] ADR-0001 and ADR-0002 are consistently recorded as approved and merged.
- [ ] ADR-0008 is represented exactly within its approved scope and does not
      pre-select Metrics roles/permissions.
- [ ] CG-08 distinguishes Thoth WP1 readiness from Sphinx/WP6, client,
      source-specific and WP5 role gates, and no longer names the resolved
      Diesel/schema control as an open prerequisite.
- [ ] `docs/engineering/README.md` no longer presents resolved Diesel/schema
      control or later repository-specific readiness as blanket Metrics
      implementation blockers.
- [ ] `docs/engineering/repository-map/repositories/thoth.md` records
      ADR-0003/PR #778 as actually merged/repository-authoritative.
- [ ] `docs/engineering/decisions/decision-register.md` records ADR-0003/PR
      #778's actual merged authority with the exact merge commit.
- [ ] Sphinx/client readiness blockers are attached to their owning
      repository/work-package path rather than used as a blanket Thoth WP1
      blocker.
- [ ] Unrelated open control gaps remain intact.
- [ ] `MET-CTRL-01` remains the current gate until independent exact-head
      approval and merge.
- [ ] After successful closeout, the documented next Thoth step is a separate
      authorization for `feature/metrics` plus a separately approved bounded
      WP1 child specification.
- [ ] No WP1 implementation branch, schema, migration or runtime behaviour is
      created by this task.
- [ ] `git diff --check` passes; relative links and issue/PR references
      resolve.
- [ ] Normal required CI for the documentation/control PR is green at the
      exact reviewed head.
- [ ] A fresh non-implementing reviewer inspects the exact complete diff and
      returns `APPROVED` before merge; CTO merge authorization is separately
      bound to the exact reviewed head.
- [ ] Post-merge #766 synchronization remains a separately authorized GitHub
      mutation.

## 11. Required tests

### Unit

- Not applicable (documentation-only).

### Integration/database

- Not applicable.

### Authorization/security

- Not applicable.

### Regression

- The issue #832 stale-state search over `docs/metrics`,
  `docs/engineering/README.md`,
  `docs/engineering/repository-map/control-gaps.md`,
  `docs/engineering/repository-map/repositories/thoth.md` and
  `docs/engineering/decisions/decision-register.md` returns no active
  stale-state occurrence; every remaining match is classified as deliberate
  historical evidence or a generic authority/process rule.
- `grep -n '^Status: APPROVED$'` matches all four ADRs
  (ADR-0001/0002/0003/0008).
- `grep -nE '^### CG-(03|04|09|10|11|13)'` still matches all six preserved
  gaps in `control-gaps.md`.

### Manual verification

- `git diff --check` and `git diff --name-only` against the exact base;
  changed-file list verified against section 14.

### Performance

Not applicable.

## 12. Rollout

- initial state after merge: reconciled controls are repository-authoritative;
  no runtime effect;
- feature flag/configuration: none;
- staging/preview validation: not applicable;
- pilot: not applicable;
- activation approval: not applicable — this task activates nothing; merge
  requires fresh exact-head independent `APPROVED` review and explicit CTO
  merge authorization;
- observation period: not applicable.

Post-merge: separately synchronize #766 only after the exact proposed comment
is reviewed and authorized; do not create `feature/metrics` or a WP1
implementation task as a side effect of merge.

## 13. Rollback

- code rollback: revert the complete documentation/control merge if required;
- data rollback or forward repair: not applicable;
- feature disable/kill switch: not applicable;
- external side-effect handling: any GitHub issue synchronization rollback
  must re-fetch the live issue/comment state, preserve unrelated later edits
  and use only a freshly reviewed minimal reversal under explicit
  authorization.

## 14. Authorized write paths, new files and prohibited paths

Existing files this task may modify:

- `CHANGELOG.md`
- `docs/metrics/README.md`
- `docs/metrics/task-status.md`
- `docs/metrics/decisions.md`
- `docs/metrics/rollout-plan.md`
- `docs/metrics/contract-register.md`
- `docs/metrics/master-issue.md` (only if a concrete active stale statement
  requires correction for consistency)
- `docs/engineering/repository-map/control-gaps.md`
- `docs/engineering/README.md`
- `docs/engineering/repository-map/repositories/thoth.md`
- `docs/engineering/decisions/decision-register.md`

New files this task may create:

- `docs/engineering/ai-delivery/tasks/MET-CTRL-01.md`
- `docs/engineering/ai-delivery/implementation-reports/MET-CTRL-01-implementation-report.md`

The actual changed-file set may be a subset of this allowlist and should be
the smallest subset required.

Prohibited paths: file deletion, move or rename is prohibited. Any other path
is denied by default. If implementation discovers that another active file
must change for internal consistency, it must stop and return
`SCOPE AMENDMENT REQUIRED` with the exact file, the exact active
contradiction and why it cannot remain unchanged.

## 15. Action authorization matrix

| Action | Authorized |
|---|---|
| repository/GitHub read inspection | YES |
| source/worktree edits within the write budget | YES |
| create the authorized new files | YES |
| delete/move/rename files | NO |
| branch creation from the exact authorized base | YES |
| commit | YES |
| push | YES |
| open/update draft PR | YES |
| issue/comment mutation | NO |
| manual CI dispatch/rerun | NO |
| provider/runtime read | NO |
| provider/runtime write | NO |
| migration execution | NO |
| release/tag/publication | NO |
| merge | NO |
| deployment | NO |
| production activation | NO |

## 16. Automatic side effects

Pushing the task branch and opening the draft PR triggers the repository's
normal PR CI (build/test/clippy/format/changelog checks). That automatic CI is
an expected, authorized side effect of the authorized push/PR actions. For a
documentation-only diff no external write (for example a container-registry
push) is expected. No manual dispatch or rerun of any workflow is authorized.

## 17. HOLD/STOP conditions

The implementing agent must stop and report if:

- `develop` moved from the exact authorized base before branch creation
  (`HOLD - AUTHORIZED BASE MOVED`);
- the working tree is not clean before starting;
- a required shared ADR is no longer repository-authoritative or no longer
  matches the approved reconstruction;
- a live repository fact conflicts with the task reconstruction;
- the approved private Metrics design has moved from revision `6` without a
  reviewed programme decision;
- another active file requires modification outside the amended write budget
  (`SCOPE AMENDMENT REQUIRED`);
- resolving CG-08 would require weakening another open control gap;
- implementation starts making an architectural decision, or exact Metrics
  roles/permissions would need to be selected;
- any runtime/source/schema/migration/workflow change appears necessary;
- any provider/production action appears necessary;
- implementation would begin creating `feature/metrics`, bootstrapping
  Sphinx, normalizing client branches or widening into another programme.

## 18. Expected implementation report

`docs/engineering/ai-delivery/implementation-reports/MET-CTRL-01-implementation-report.md`,
following `docs/engineering/ai-delivery/implementation-report-template.md`,
including the exact proposed post-merge #766 synchronization comment as text
only.

## 19. Recommended execution

Implementation model: bounded coding/documentation agent
Reasoning level: high
Independent reviewer: fresh non-implementing context
Review reasoning level: high

## 20. Branch and integration plan

- branch source: `develop @ 250554dd7351c97af46d59b5033abd391d9eec16`;
- pull-request target: `develop`;
- expected merge order: single bounded PR, no dependencies;
- parent programme branch refresh requirement: none (no programme integration
  branch exists; `feature/metrics` creation is a later separately authorized
  action);
- branch deletion after merge: YES
- final programme PR required: NO
- final release path: `develop -> master`

## 21. Approval

Approved for implementation by: CTO, via issue #832 (amended 2026-08-25) and
its fresh independent non-implementing specification review (`APPROVED`),
with explicit bounded authorization of the exact base, branch creation,
documentation writes, commit, push and draft-PR creation.
Date: 2026-08-25
Notes: specification approval is not merge, #766-synchronization,
`feature/metrics`-creation, WP1, deployment, migration or production
authorization.
