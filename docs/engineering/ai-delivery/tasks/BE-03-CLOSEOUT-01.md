# BE-03-CLOSEOUT-01 - Post-merge control correction for BE-03

Status: DRAFT
Programme: Publisher Services and Distribution Configuration
Repository: `thoth-pub/thoth`
Workflow: STANDARD
Base branch: `develop`
PR target: `develop`
Programme integration branch: None
Risk: LOW
Owner: CTO
Approved by: specification approval authority is the CTO; approval evidence is
the GitHub pull-request record for the pull request carrying this content
Dependencies: merged BE-03 implementation PR
[#809](https://github.com/thoth-pub/thoth/pull/809)
Target branch name: `feature/publisher-services/be-03-closeout`
Specification branch: `feature/publisher-services/be-03-closeout-spec`
Master programme issue:
[#765](https://github.com/thoth-pub/thoth/issues/765)

Authority condition: this record is repository-authoritative when this exact
content is reachable from the repository's authoritative integration branch
(`develop`). Live review, authorization and merge evidence is the GitHub
pull-request record and is not copied here.

Implementation authorization: **separate.** This document specifies the BE-03
closeout; it does not authorize it. The implementing agent must not create
`feature/publisher-services/be-03-closeout` or edit any file in scope without
explicit CTO authorization against a freshly verified exact `develop` head.
Section 17 defines the approval boundary.

Specification authoring base: `a4585a8d89166577da5ce6f46ce51ddb134b3f7e` (the
merge commit of forward-integration PR
[#811](https://github.com/thoth-pub/thoth/pull/811), whose first parent
`3ba4452c316399d80cd8d85e7d5e1bd05e252664` is the merge commit of BE-03
implementation PR [#809](https://github.com/thoth-pub/thoth/pull/809)). That SHA
is a specification-authoring record only. It is not an implementation base and
confers no implementation authorization.

## 1. Objective

Correct the materially incorrect active Publisher Services programme and
dependency state left behind after the BE-03 implementation merged: committed
control documents still describe BE-03 as an unmerged draft awaiting a fresh
independent review and CTO merge authorization, still assert that BE-03
implementation is `NOT AUTHORIZED`, still forbid the existence of the
`feature/publisher-services/be-03` branch, and still list BE-03 as an
unsatisfied blocking dependency of BE-04, MIG-01, APP-01 and APP-02.

This is a documentation and control correction only. It closes BE-03 as a
merged but inactive foundation, and records each downstream dependency
accurately — satisfied where the merge satisfied it, and still blocked
everywhere else.

## 2. Background and authority

Authoritative sources:

- [ADR-0005](../../decisions/ADR-0005-terminal-merge-evidence.md), section 8 -
  a post-merge task is *required* when "a committed tracker contains materially
  incorrect programme state", and *not* required merely to record that a pull
  request merged, its merge SHA, its review identifiers or its authorization;
  section 9 preserves historical records as written;
- [`docs/engineering/AGENTS.md`](../../AGENTS.md) section 1.1 - durable versus
  transient state;
- merged BE-03 implementation PR
  [#809](https://github.com/thoth-pub/thoth/pull/809) and the merged migration,
  schema contract, write coordinator, GraphQL surfaces and tests it delivered;
- the [BE-03 specification](BE-03.md) and the
  [BE-03 implementation report](../implementation-reports/BE-03-implementation-report.md);
- the [BE-02-CLOSEOUT-01](BE-02-CLOSEOUT-01.md) precedent, which established the
  house form for this class of correction.

Current behaviour. `docs/publisher-services/task-status.md` records BE-03 as
`IMPLEMENTATION IN REVIEW` with PR #809 `(draft)` and an acceptance cell reading
`IMPLEMENTATION DELIVERED, NOT MERGED`; its `Last updated` note and next-action
10 describe the new exact head as awaiting a fresh independent review and
outstanding CTO merge authorization; next-action 12 calls the BE-03
implementation "delivered-but-unmerged". `docs/publisher-services/README.md`
asserts `BE-03 IMPLEMENTATION NOT AUTHORIZED` in its section 5 decision block
and repeats it in gating reason 1. `docs/publisher-services/rollout-plan.md`
ends its BE-02 implementation state block with "`BE-03` implementation is
`NOT AUTHORIZED`", and section 2.2 binds "the later `BE-03`" in the future
tense. [`BE-03.md`](BE-03.md) section 23 states that "BE-03 implementation
status is `NOT AUTHORIZED`" and that `feature/publisher-services/be-03` "must
not exist", while its header still reads `Status: DRAFT`. BE-04, MIG-01, APP-01
and APP-02 each still list BE-03 among their unsatisfied blocking dependencies.

Every one of those statements is now false, and each is operationally
misleading about what the programme may and may not do next.

What the merged repository now contains, as an inactive additive foundation:

- the additive BE-03 database and schema foundation - the canonical
  optimistic-concurrency token `publisher.service_configuration_updated_at`, the
  closed two-value `publisher_service_configuration_source` type and the
  append-only `publisher_service_configuration_history` audit table, with
  `thoth-api/src/schema.rs` maintained atomically in the same pull request under
  ADR-0003 Architecture A;
- the protected service-configuration read, staff report and mutation surfaces -
  `publisherServiceConfiguration`, `publisherServiceConfigurations`,
  `publisherServiceConfigurationCount` and
  `replacePublisherServiceConfiguration`;
- the single canonical service-configuration write coordinator, which owns every
  committed write of package, enabled-platform desired state, configuration
  version token and audit history in one transaction on one connection;
- configuration audit history;
- optimistic concurrency, including a distinct stale-configuration error;
- effective package capability exposure derived from BE-01's code-owned
  `ThothPackage::capabilities()`;
- the connection-scoped BE-02 lifecycle seam required for later BE-04
  composition.

Repository integration is not deployment and not activation. The migration
creates zero audit rows, changes no package and no assignment, and no
distribution job, dissemination or activation exists.

### 2.1 Relationship to forward-integration PR #811

Between the BE-03 merge and this specification, forward-integration PR
[#811](https://github.com/thoth-pub/thoth/pull/811) merged the v1.6.3 hotfix
from `master` into `develop` and renamed migration directories:

```text
BE-02 migration
  original implementation-head path: thoth-api/migrations/20260812_v1.7.0/
  current develop path:              thoth-api/migrations/20260811_v1.7.0/

BE-03 migration
  original implementation-head path: thoth-api/migrations/20260813_v1.7.0/
  current develop path:              thoth-api/migrations/20260812_v1.7.0/

THOTH-CHAPTER-01 / v1.6.3 migration
  current develop path:              thoth-api/migrations/20260813_v1.6.3/
```

PR #811 is legitimate intervening repository history and is accepted as part of
this task's base. The relative apply order of the BE-02 and BE-03 migrations is
unchanged: BE-03's migration still applies after BE-02's.

The migration-path references inside exact-head historical implementation and
specification reports are **historical evidence** and are out of scope for
correction. A statement such as "BE-03 created `20260813_v1.7.0`" remains
historically correct for the PR #809 implementation head and must not be
rewritten to the current directory name. This closeout is not responsible for
reviewing, repairing or replaying PR #811, and must not revert, rename or
otherwise modify its migration changes or the THOTH-CHAPTER-01 implementation.

Where this closeout's own new prose needs to identify the current repository
migration, it uses `thoth-api/migrations/20260812_v1.7.0/` and, if both facts
are needed, distinguishes the implementation-head path from the current path
explicitly rather than blurring them.

## 3. Explicit scope

The task must:

1. perform the classified stale-state search again from its freshly authorized
   implementation base, over the active Publisher Services and shared
   engineering-control surface, classifying every BE-03 statement as
   `ACTIVE STALE STATE - CORRECT`, `HISTORICAL RECORD - PRESERVE`,
   `CURRENT AND CORRECT - PRESERVE` or `OUT OF SCOPE - PRESERVE`, and reading
   each statement in context rather than by pattern;
2. correct only the statements classified `ACTIVE STALE STATE`, so that active
   controls durably represent:
   - **BE-03**: `CLOSED - INACTIVE FOUNDATION`, with a durable acceptance
     statement describing the merged foundation listed in section 2 and the
     inactivity of everything it enables;
   - **BE-04**: its BE-03 dependency satisfied; BE-04 itself still `BLOCKED` and
     `NOT STARTED`, neither specified nor authorized, with no
     `distribution_job`, `distribution_job_target` or `distribution_job_attempt`
     runtime behaviour, no automatic back-catalogue job creation and no
     dissemination existing merely because BE-03 merged;
   - **MIG-01**: its BE-03 dependency satisfied; MIG-01 still `CRITICAL` and
     `BLOCKED` by its other recorded prerequisites, including the production
     audit and backfill controls, with no backfill or production migration
     authorized;
   - **APP-01**: its BE-03 backend-contract dependency satisfied for APP-01's
     configuration-only scope — own-publisher reads of package, effective
     capability codes and enabled platforms; superuser read and edit;
     backend-normalized linked-platform behaviour; optimistic-concurrency
     handling; server-normalized resulting configuration — while APP-01 itself
     remains `BLOCKED` by its app readiness controls (BR-APP-01 or an explicit
     CTO exception, and the separately specified CG-11 CI closure task), the
     reserved exact-SHA GraphQL contract-pinning control of
     `rollout-plan.md` section 2.2, and its own approved bounded specification;
     and while any APP-01 element rendering durable back-catalogue job status,
     attempt state, failure state or pending-onboarding state remains dependent
     on BE-04;
   - **APP-02**: its BE-03 dependency satisfied only; APP-02 remains `BLOCKED`
     on BE-04 **and** APP-01 and must not be represented as ready;
   - deployment, environment migration execution, production migration
     execution, package commercial backfill, distribution assignment
     creation/backfill, durable job creation, dissemination, distribution
     activation, `OBSERVE`/`ENFORCE` transitions, workflow changes or manual
     workflow dispatch, and production access: each `NOT AUTHORIZED`;
   - PR [#799](https://github.com/thoth-pub/thoth/pull/799): untouched and
     outside the Publisher Services dependency set;
3. correct the BE-03 specification's own lifecycle-boundary prose in
   [`BE-03.md`](BE-03.md) — its `Status` line, the header
   implementation-authorization block and section 23 — so that it no longer
   asserts that BE-03 implementation is `NOT AUTHORIZED` or that the
   `feature/publisher-services/be-03` branch must not exist, without altering
   any approved requirement, acceptance criterion, test obligation or
   architectural statement in that document;
4. resolve the internal contradiction in `task-status.md`, which asserts in
   next-action 10 that both halves of the phase-boundary authority condition
   hold and the decision is approved, while the APP-01 row and next-action 11
   still describe it as "the candidate phase boundary";
5. record this bounded task and its implementation report;
6. add the required `CHANGELOG.md` entry under `## [Unreleased]`.

Expected touched paths, and no others:

```text
docs/publisher-services/task-status.md
docs/publisher-services/README.md
docs/publisher-services/rollout-plan.md
docs/publisher-services/decisions.md
docs/engineering/ai-delivery/tasks/BE-03.md
docs/engineering/ai-delivery/tasks/BE-03-CLOSEOUT-01.md
docs/engineering/ai-delivery/implementation-reports/BE-03-CLOSEOUT-01-implementation-report.md
CHANGELOG.md
```

`docs/publisher-services/platform-inventory.md`,
`docs/publisher-services/acceptance-matrix.md`,
`docs/publisher-services/master-issue.md` and
`docs/engineering/repository-map/control-gaps.md` carry no active BE-03
lifecycle assertion at the authoring base and are expected to remain untouched.
If the implementation's own classified search finds one, it is corrected under
the same rules and recorded as a deviation in scope, not silently.

## 4. Non-goals

The task must not:

1. change any runtime code, migration, `schema.rs`, GraphQL contract, generated
   SDL, client artifact, Cargo file, workflow, deployment or environment
   configuration;
2. create an approval-state-only commit, or copy independent review
   identifiers, CTO approval identifiers, merge-authorization identifiers, PR
   #809's merge commit SHA, the merged timestamp or the bare statement "the PR
   is now merged" into repository files merely to transcribe them;
3. rewrite historical implementation-time evidence, including the BE-03, BE-03
   SPEC and BE-02 implementation reports and their migration-path references,
   merely because the pull request later merged or because PR #811 later renamed
   those directories;
4. edit the historical body or comments of PR #809 or PR #811;
5. modify issue [#765](https://github.com/thoth-pub/thoth/issues/765) or issue
   [#766](https://github.com/thoth-pub/thoth/issues/766);
6. use global find/replace;
7. state or imply that BE-03 is deployed, migrated, activated or
   production-ready;
8. take any action on PR [#799](https://github.com/thoth-pub/thoth/pull/799) or
   on mutation-guard mode;
9. start, specify, authorize or pull forward BE-04, MIG-01, APP-01, APP-02 or
   any other task;
10. amend the BE-03/BE-04/APP-01 phase-boundary architecture, or replace the
    ADR-0005 authority-condition construction in
    `docs/publisher-services/decisions.md` section 3a with a literal `APPROVED`
    status token (see section 5, invariant 6);
11. broaden into a migration-documentation reconciliation task for the PR #811
    renames.

## 5. Invariants

The implementation must preserve:

1. ADR-0005 terminal merge evidence: GitHub remains the authority for review,
   authorization and merge lifecycle facts, and no repository commit exists
   merely to restate them;
2. durable status prose that stays truthful before review, after review, before
   merge and after merge, so that this task's own merge does not create the next
   closeout;
3. every historical record, including implementation reports, superseded
   approvals and migration-path references that were correct at the head they
   describe;
4. the merged BE-03 public GraphQL surfaces, the single-write-coordinator rule
   and the inactivity of the merged foundation;
5. the separately gated status of every other Publisher Services task;
6. the ADR-0005 authority-condition construction in
   `docs/publisher-services/decisions.md` section 3a. That section states its own
   resolution rule — after both halves of the condition hold it is an approved
   programme decision "without requiring a separate lifecycle-status edit to
   this file" — and both halves do hold, because the exact CTO-approved
   `BE-03-SPEC` content merged through PR
   [#808](https://github.com/thoth-pub/thoth/pull/808). The construction is
   therefore materially true and already establishes the approved decision. It
   is preserved as written, including its `Decision state:` line and its
   internal "candidate" phrasing, precisely to avoid the approval-state churn
   ADR-0005 section 4.1 item 10 prohibits. Only references **outside** that
   self-resolving construction, which do not carry it and therefore understate
   the decision, are corrected;
7. the migration apply order and every migration directory name as PR #811 left
   them.

## 6. Required behaviour

### 6.1 Success behaviour

Active control documents describe BE-03 as a closed, merged, inactive
foundation; describe each downstream dependency accurately, distinguishing the
BE-03 dependency that the merge satisfied from every dependency that remains
outstanding; and record every operational and production action as
`NOT AUTHORIZED`. No active control statement asserts that BE-03 is unmerged,
in review, awaiting authorization, or an unsatisfied dependency where the merge
satisfied it. No downstream task's `BLOCKED` status changes.

### 6.2 Failure behaviour

Not applicable: no runtime behaviour changes.

### 6.3 Authorization and security impact

**None.** No authorization path changes. No protected surface is added, removed
or altered. `thoth-api/src/policy.rs` and every authorization test are
untouched. No credential, token, endpoint, bucket or account identity is
introduced into the diff.

### 6.4 Concurrency and idempotency

Not applicable.

### 6.5 Compatibility boundary

**No API, database, client or deployment compatibility surface is touched.** The
merged GraphQL contract is unchanged, so the reserved BE-03/APP-01 exact-SHA
schema-pinning control in `rollout-plan.md` section 2.2 continues to bind
against the BE-03 implementation head rather than against this task's head. This
task produces no new contract for a downstream repository to pin, and
`thoth-client/assets/schema.graphql` is byte-identical to the base.

## 7. Data and migration requirements

Migration required: **NO**

No migration, schema, catalog or data change of any kind.
`thoth-api/src/schema.rs` and `thoth-api/migrations/` are untouched.

## 8. Observability and operations impact

**None.**

Required logs: none.
Required metrics/alerts: none.
Operational runbook changes: none.

This task activates nothing, executes no migration, dispatches no workflow and
touches no environment or production configuration.

## 9. Acceptance criteria

- [ ] every BE-03 statement in the searched surface carries a recorded
      classification, produced by a context-read classified search rather than
      pattern replacement;
- [ ] no statement classified `HISTORICAL RECORD`, `CURRENT AND CORRECT` or
      `OUT OF SCOPE` is modified;
- [ ] `docs/publisher-services/task-status.md` records BE-03 as
      `CLOSED - INACTIVE FOUNDATION` with a durable acceptance statement;
- [ ] no active control document asserts that BE-03 is unmerged, a draft, in
      review, awaiting independent review, awaiting merge authorization,
      `NOT AUTHORIZED`, or that `feature/publisher-services/be-03` must not
      exist;
- [ ] BE-04, MIG-01, APP-01 and APP-02 each record their BE-03 dependency as
      satisfied while retaining their remaining blockers and their `BLOCKED` /
      `NOT STARTED` status;
- [ ] no active control document represents BE-04 as specified, authorized or
      implemented, or represents any durable job, target, attempt or automatic
      onboarding behaviour as existing;
- [ ] MIG-01 remains `CRITICAL` and blocked by its recorded audit, backfill and
      production prerequisites;
- [ ] APP-01 remains separately gated by its app readiness controls, the
      exact-SHA contract-pinning control and its own approved specification, and
      its job-aware elements remain BE-04-dependent;
- [ ] APP-02 remains blocked on BE-04 and APP-01;
- [ ] deployment, environment migration execution, production migration
      execution, package commercial backfill, assignment creation/backfill,
      durable job creation, dissemination, distribution activation,
      `OBSERVE`/`ENFORCE`, workflow changes or dispatch, and production access
      are each recorded as `NOT AUTHORIZED`;
- [ ] PR #799 is untouched and not represented as a Publisher Services
      dependency;
- [ ] `docs/publisher-services/decisions.md` section 3a retains its
      authority-condition construction unchanged;
- [ ] no review, approval or merge identifier, merge SHA or merge timestamp is
      newly transcribed into a repository file;
- [ ] no migration-path reference in any implementation report is rewritten, and
      any new prose naming the current BE-03 migration uses
      `thoth-api/migrations/20260812_v1.7.0/`;
- [ ] the diff touches documentation paths and `CHANGELOG.md` only;
- [ ] `git diff --check` reports no whitespace error;
- [ ] `CHANGELOG.md` has one entry under `## [Unreleased]`, with no duplicate
      heading created.

## 10. Required tests

### Unit

Not applicable: documentation-only change. Root
[`AGENTS.md`](../../../../AGENTS.md) section 8 prescribes the documentation-only
evidence set below and reserves the full workspace gate for Rust/domain changes.
No file under any workspace member is modified, so the workspace gate has no
changed input and is not required.

### Integration/database

Not applicable. No migration, schema or database-backed code is touched.

### Authorization/security

Not applicable. No authorization code exists in this diff.

### Regression

Path containment. Run:

```bash
git diff --name-only <authorized-base>..HEAD
```

Every path must match `^docs/` or be exactly `CHANGELOG.md`. Then prove the
negative:

```bash
git diff --name-only <authorized-base>..HEAD | grep -E '^(thoth-api|thoth-api-server|thoth-client|thoth-errors|thoth-export-server|\.github|Cargo\.)' 
```

This must produce no output, proving that no runtime, migration, schema,
GraphQL, generated client, workflow or Cargo file changed.

### Manual verification

1. **Formatting.** `git diff --check` reports no whitespace error.
2. **Documentation link verification.** Every relative link and path introduced
   or touched by the change resolves against the filesystem.
3. **Classified stale-state re-run.** Re-run the classified BE-03 search over
   the active surface:

   ```bash
   git grep -n 'BE-03' -- docs/ CHANGELOG.md
   ```

   Read each hit in context and confirm that no active control statement says
   BE-03 is unmerged, a draft, in review, awaiting review or authorization, or
   remains an unsatisfied dependency where the merge satisfied it. Confirm that
   every remaining such statement is a classified and preserved historical
   record.
4. **Unauthorized-action verification.** Confirm that deployment, environment
   and production migration execution, package commercial backfill, assignment
   creation/backfill, durable job creation, dissemination, distribution
   activation, `OBSERVE`/`ENFORCE`, workflow dispatch and production access each
   remain recorded as `NOT AUTHORIZED`, and that PR #799 remains untouched:

   ```bash
   git grep -n 'NOT AUTHORIZED' -- docs/publisher-services/ docs/engineering/repository-map/control-gaps.md
   git grep -n '799' -- docs/
   ```

5. **BE-04 verification.** Confirm no active document represents BE-04 as
   implemented, specified or authorized, and that no `distribution_job`,
   `distribution_job_target` or `distribution_job_attempt` behaviour is
   represented as existing:

   ```bash
   git grep -n 'BE-04\|distribution_job' -- docs/publisher-services/ docs/engineering/ai-delivery/tasks/
   ```

6. **APP-01 verification.** Confirm APP-01 remains `BLOCKED` and separately
   gated despite its BE-03 API dependency becoming satisfied, and that its
   job-aware elements remain BE-04-dependent.
7. **Durability re-read.** Re-read each corrected paragraph and confirm the
   wording stays truthful before and after this task's own pull request merges.
8. **Changelog.** One entry under the existing `## [Unreleased]` heading; no
   duplicate heading.
9. **Repository CI.** The live result is the GitHub pull-request check record;
   the documentation-only gating is expected to classify this change as docs-only.

### Performance

Not applicable.

## 11. Rollout

- initial state after merge: repository documentation only; no runtime,
  deployment, migration, backfill, activation or production effect;
- feature flag/configuration: none;
- staging/preview validation: not applicable;
- pilot: not applicable;
- activation approval: not applicable; this task activates nothing;
- observation period: none.

## 12. Rollback

- code rollback: ordinary revert of the documentation pull request under normal
  review;
- data rollback or forward repair: not applicable;
- feature disable/kill switch: not applicable;
- external side-effect handling: none. Any transcription comment on PR #809 is
  immutable GitHub evidence and is not removed by a repository revert.

## 13. Stop conditions

The implementing agent must stop and report `BLOCKED` if:

- the exact authorized `develop` base has moved before branch creation, or
  PR #809 is not merged and reachable from it;
- a competing BE-03 closeout branch, pull request or committed record already
  exists;
- repository authority conflicts about what BE-03's merge satisfies;
- the BE-03/BE-04/APP-01 phase boundary would need an architectural amendment
  rather than a control correction;
- an active, non-historical control document makes a present-tense operational
  assertion about the BE-03 migration path that materially affects architecture,
  migration ordering, release safety or BE-04 planning, rather than merely
  naming a historical and a current path differently;
- the correction would require a runtime, schema, migration, GraphQL, workflow
  or environment change;
- the correction would require rewriting historical evidence;
- issue #765 or #766 would need mutation;
- PR #799 would need action;
- approved architecture would need to change;
- scope cannot remain bounded, or cannot be completed without unrelated changes.

## 14. Expected implementation report

The agent must use:

`docs/engineering/ai-delivery/implementation-report-template.md`

Written to
`docs/engineering/ai-delivery/implementation-reports/BE-03-CLOSEOUT-01-implementation-report.md`.

Section 10 must record the complete classified stale-state findings under the
four classification headings, including the explicit classification of the
PR #811 migration-path references and any active path-related control debt
discovered but deliberately not corrected. Section 13 must record any such debt
as deferred work with a recommended owner.

## 15. Recommended execution

Implementation model: Claude Opus 5
Reasoning level: Extra High / xhigh
Independent reviewer: fresh independent strong model/context
Review reasoning level: High or above

Rationale: the work is small in diff size but high in judgement density. The
failure modes are silently broadening scope, transcribing lifecycle metadata
that ADR-0005 prohibits, opportunistically rewriting historical evidence, and
marking a downstream task ready when only one of its dependencies was satisfied.
Each is a reading-comprehension failure rather than a coding failure, so
reviewer independence matters more than tooling.

## 16. Branch and integration plan

- branch source: the exact `develop` head verified at authorization time, which
  must have PR #809's merge commit
  `3ba4452c316399d80cd8d85e7d5e1bd05e252664` as an ancestor;
- branch name: `feature/publisher-services/be-03-closeout`;
- pull-request target: `develop`;
- expected merge order: after this specification's own pull request merges, and
  before any BE-04, MIG-01, APP-01 or APP-02 specification work begins, so that
  those tasks are planned against corrected dependency state;
- parent programme branch refresh requirement: none; the programme uses no
  integration branch;
- branch deletion after merge: YES
- final programme PR required: NO
- final release path: `develop -> master`
- no rebase, amend, squash, force-push or merge of another branch into the task
  branch without fresh CTO instruction.

## 17. Approval boundary

Specification approval settles **what the BE-03 closeout must do**. It does not:

- authorize the closeout implementation;
- authorize deployment or environment migration execution;
- authorize production migration, package commercial backfill, assignment
  creation or backfill;
- authorize durable job creation, dissemination, distribution activation or any
  `OBSERVE`/`ENFORCE` transition;
- authorize workflow changes, manual workflow dispatch, production access or
  credential use;
- start, specify or authorize BE-04, MIG-01, APP-01 or APP-02.

Approval for implementation, when given, is limited to documentation and control
correction of materially stale active BE-03 programme state.

## 18. Approval

Approved for implementation by:
Date:
Notes:

Record only the durable implementation authorization here. Independent review
decisions, CTO merge authorization and the merge itself are terminal GitHub
evidence under ADR-0005 and must not be copied back into this file.

---

## Annex A - Authoring-time classified stale-state findings (informative)

Recorded at specification-authoring base
`a4585a8d89166577da5ce6f46ce51ddb134b3f7e`. This annex is **informative**. The
implementing agent must repeat the classified search from its own freshly
authorized base and must not treat this annex as a substitute, a checklist to
apply mechanically, or a guarantee that the surface is unchanged.

### `ACTIVE STALE STATE - CORRECT`

- `docs/publisher-services/task-status.md` line 7 - `Last updated` note
  describing BE-03 as delivered as a draft pull request with the new exact head
  "awaiting a fresh independent review";
- `docs/publisher-services/task-status.md` BE-03 row - status
  `IMPLEMENTATION IN REVIEW`; blocking-dependency cell listing fresh independent
  review and CTO merge authorization as remaining gates; PR cell `(draft)`;
  acceptance cell `IMPLEMENTATION DELIVERED, NOT MERGED`;
- `docs/publisher-services/task-status.md` BE-04, MIG-01, APP-01 and APP-02
  rows - BE-03 listed as an unsatisfied blocking dependency;
- `docs/publisher-services/task-status.md` APP-01 row and next-action 11 - "the
  candidate phase boundary", contradicting next-action 10 in the same file,
  which states that both halves of the authority condition hold;
- `docs/publisher-services/task-status.md` next-action 10 - "BE-03 is **not
  merged**", the fresh-review requirement and "explicit CTO merge authorization
  remains outstanding";
- `docs/publisher-services/task-status.md` next-action 11 - "APP-01 remains
  blocked pending BE-03";
- `docs/publisher-services/task-status.md` next-action 12 - "the separately
  authorized, delivered-but-unmerged BE-03 implementation";
- `docs/publisher-services/README.md` section 5 decision block - `BE-03
  DEPENDENCIES ON BE-01 AND BE-02 SATISFIED; BE-03 IMPLEMENTATION NOT
  AUTHORIZED`;
- `docs/publisher-services/README.md` gating reason 1 - "`BE-03` implementation
  remains `NOT AUTHORIZED` pending its own approved bounded specification and
  separate explicit authorization";
- `docs/publisher-services/README.md` closing paragraph - an approved
  specification does not "unlock `BE-03`";
- `docs/publisher-services/rollout-plan.md` BE-02 implementation state block -
  "`BE-03` implementation is `NOT AUTHORIZED`";
- `docs/publisher-services/rollout-plan.md` section 2.2 - "It binds the later
  `BE-03` and `APP-01` tasks" and item 1's future-tense "`BE-03` produces an
  exact generated GraphQL SDL at its reviewed head", both of which have
  happened;
- `docs/publisher-services/decisions.md` section 3a APP-01 reconciliation -
  APP-01 "remains blocked on BE-03 exposing the approved protected API", where
  that specific dependency is now satisfied and only the other gates remain;
- `docs/engineering/ai-delivery/tasks/BE-03.md` line 3 - `Status: DRAFT`,
  contradicting `task-status.md`, which records the specification as
  repository-authoritative through PR #808;
- `docs/engineering/ai-delivery/tasks/BE-03.md` header
  implementation-authorization block - "**separate and absent** ... The branch
  `feature/publisher-services/be-03` must not exist until the CTO separately and
  explicitly authorizes implementation";
- `docs/engineering/ai-delivery/tasks/BE-03.md` section 23 - "BE-03
  implementation status is `NOT AUTHORIZED`. The branch
  `feature/publisher-services/be-03` must not exist until separate explicit CTO
  authorization from a freshly verified base."

### `HISTORICAL RECORD - PRESERVE`

- `docs/engineering/ai-delivery/implementation-reports/BE-03-implementation-report.md`
  in its entirety, including its base and head commits, exact test commands, CI
  record and **every** reference to `thoth-api/migrations/20260813_v1.7.0`
  (lines 41, 87, 94, 276, 289, 290) and to `20260812_v1.7.0` as BE-02's
  migration (line 280). Correct for the PR #809 implementation head. Its
  rollout note at line 1327, "Migration sequence: `20260813_v1.7.0` applies
  after `20260812_v1.7.0`", is preserved: it was true as written, and the
  ordering it asserts still holds under the renamed directories;
- `docs/engineering/ai-delivery/implementation-reports/BE-03-SPEC-implementation-report.md`
  lines 1144 and 1312 - references to BE-02's migration as `20260812_v1.7.0`,
  correct at the specification-authoring head;
- `docs/engineering/ai-delivery/implementation-reports/BE-02-implementation-report.md`
  lines 69, 70, 91, 94 and 162 - references to BE-02's migration as
  `20260812_v1.7.0`, correct at the BE-02 implementation head;
- every BE-01, BE-02, ADR-01, ADR-01-SPEC-AMEND-01, ADR-01-CLOSEOUT-01,
  BE-02-CLOSEOUT-01, P0-01 and THOTH-GQL-* task record and implementation
  report;
- `CHANGELOG.md`'s existing `BE-03` entry from PR #809 and `BE-03-SPEC` entry
  from PR #808 - delivered-change records describing what each pull request
  contained at the time. The `BE-03-SPEC` entry closes with "`BE-03`
  implementation remains **NOT AUTHORIZED** and `feature/publisher-services/be-03`
  must not exist", which was an accurate statement of that change's boundary
  when written. The changelog is an append-only record of shipped changes, not
  an active tracker, and `BE-02-CLOSEOUT-01` classified the equivalent PR #805
  entry the same way. It is preserved.

### `CURRENT AND CORRECT - PRESERVE`

- `docs/publisher-services/decisions.md` section 3a in its entirety, including
  `Decision state: PROPOSED IN THIS SPECIFICATION CANDIDATE`, the authority
  condition, the "This decision candidate refines..." phrasing in the APP-01
  reconciliation, and the closing paragraph. See section 5, invariant 6: the
  construction is self-resolving, materially true, and already establishes the
  approved decision;
- `docs/publisher-services/decisions.md` operational invariants 3 and 7 -
  backfill creates no back-catalogue jobs; automatic job creation is initially
  inactive;
- `docs/publisher-services/README.md` section 5 item 6 - protected package and
  effective-capability reads and the dedicated superuser package mutation remain
  BE-03 scope;
- `docs/publisher-services/rollout-plan.md` dependency graph and the structural
  statement "`BE-03` depends on both `BE-01` and `BE-02`";
- `docs/publisher-services/rollout-plan.md` control "`thoth-app` must not begin
  `APP-01` implementation until `BE-03` exposes the approved protected API" - a
  durable rule whose condition is now satisfied; the rule's wording stays
  truthful;
- `docs/publisher-services/rollout-plan.md` Stage 2 and Stage 4 deliverables and
  controls - rules rather than status claims;
- `docs/publisher-services/platform-inventory.md` line 57 - jobs are BE-04
  scope and automatic job creation is inactive;
- `docs/engineering/repository-map/control-gaps.md` CG-13 activation block,
  including `BE-02 runtime: NOT AUTHORIZED` - every entry is a production
  activation transition and remains true; CG-13 remains `OPEN`;
- `docs/engineering/ai-delivery/tasks/BE-03.md` section 24 approval boundary -
  a statement about what specification approval does not authorize, still true;
- `docs/engineering/decisions/ADR-0001` line 334 - an architectural affected-task
  reference, not a lifecycle claim.

### `OUT OF SCOPE - PRESERVE`

- `docs/publisher-services/acceptance-matrix.md` - maps requirements to owning
  tasks and evidence; carries no BE-03 lifecycle status claim;
- `docs/publisher-services/master-issue.md` - no BE-03 statement;
- `docs/engineering/repository-map/control-gaps.md` - no BE-03 reference at this
  base;
- `docs/engineering/ai-delivery/tasks/THOTH-GQL-BATCH-01.md` line 1636 - the
  acceptance criterion "`BE-02` remains unimplemented: no `DistributionPlatform`,
  no `publisher_distribution_platform`, no `BE-02` GraphQL field". This is a
  different programme's `DRAFT`, `NOT AUTHORIZED` specification, and the
  criterion is a scope-containment test on **that** task's own future diff. Its
  premise is now outdated, but correcting it belongs to that task and programme;
- `docs/publisher-services/README.md` section 5 item 7 - "It is a decision
  record, not an implemented enum; no `DistributionPlatform` enum exists in
  code." This is **materially false** — BE-02 merged the 17-value enum — but it
  is a BE-02 assertion, not a BE-03 one. `BE-02-CLOSEOUT-01` acceptance criterion
  "no active control document asserts that no `DistributionPlatform` enum is
  implemented" targeted exactly this class of statement and corrected the
  `platform-inventory.md` and `control-gaps.md` instances; this README instance
  survived. It is recorded here as **residual BE-02 control debt** and is
  deliberately left to its own bounded task, since correcting it is not
  necessary to make the BE-03 closeout internally coherent and folding it in
  would silently broaden this task;
- issue #765, issue #766 and PR #799 - explicitly untouched.

### Active path-related control debt

**None found in the active control surface.** Every reference to a renamed
migration directory is confined to implementation reports, which are historical
evidence under the ruling recorded in section 2.1. No active Publisher Services
or shared engineering-control document makes a present-tense operational
assertion that the current BE-03 migration path is `20260813_v1.7.0`.

One adjacent observation, recorded for the CTO and explicitly **not** actioned
by this task: PR #811 placed the v1.6.3 hotfix migration at
`thoth-api/migrations/20260813_v1.6.3/`, which sorts after the v1.7.0
publisher-services migrations at `20260811_v1.7.0` and `20260812_v1.7.0`. That
is PR #811's business, is out of scope here under section 3 of the authorizing
instruction, and does not affect the BE-02-then-BE-03 apply order this task
depends on.
