# BE-04-SPEC Implementation Report

## 1. Repository state

Repository: `thoth-pub/thoth`
Workflow: STANDARD
Base branch: `develop`
Base commit: `fac86e38383e2059e8795698e1585932c35b5b6d`
PR target: `develop`
Programme integration branch: None
Task branch: `feature/publisher-services/be-04-spec`
Head commit: recorded in the pull request; this report is written at the branch
head that carries it
Pull request: live pull-request state, review state and CI evidence are
represented by the GitHub pull-request record. This committed report does not
duplicate transient PR lifecycle state (ADR-0005).
Expected branch deletion after merge: YES
Final programme PR required: NO
Implementing model: Claude Opus 5
Reasoning level: Extra High / xhigh

### 1.1 Preflight

**Historical authoring-time record.** This is what was verified immediately
before the first edit of this task. It is preserved as evidence of the state the
specification was authored against; it is **not** a claim about the current
repository, and ordinary lifecycle progression since — for example the opening of
this task's own specification pull request — does not falsify it.

```text
git fetch origin --prune                     = clean, no output

origin/develop                               = fac86e38383e2059e8795698e1585932c35b5b6d
  parents                                    = b51bcc0905ac17fc0c142b2002b11fec711331a3
                                               06cfab5e029a9b951f2512e8fd159c4542035013
  subject                                    = Merge pull request #813 from
                                               thoth-pub/feature/publisher-services/be-03-closeout
  body                                       = BE-03-CLOSEOUT-01: reconcile BE-03
                                               post-merge control state

PR #813                                      = MERGED (mergedAt 2026-08-14T08:28:38Z)
  title                                      = BE-03-CLOSEOUT-01: reconcile BE-03
                                               post-merge control state
  base / head                                = develop / feature/publisher-services/be-03-closeout
  mergeCommit.oid                            = fac86e38383e2059e8795698e1585932c35b5b6d

BE-02                                        = CLOSED (task-status.md tracker row)
BE-03                                        = CLOSED - INACTIVE FOUNDATION (tracker row)
BE-03-CLOSEOUT-01                            = present on develop
  tasks/BE-03-CLOSEOUT-01.md                                            present
  implementation-reports/BE-03-CLOSEOUT-01-implementation-report.md      present
  implementation-reports/BE-03-CLOSEOUT-01-SPEC-implementation-report.md present

docs/engineering/ai-delivery/tasks/BE-04.md  = ABSENT
feature/publisher-services/be-04-spec        = ABSENT (local and origin)
feature/publisher-services/be-04             = ABSENT (local and origin)
open BE-04 specification or implementation PR = NONE
  open PRs at preflight: #799, #752, #744, #742, #668 - all unrelated
Working tree                                 = clean
```

The three preflight assertions that had to be re-derived rather than assumed:

- **`origin/develop` is exactly the authorized base.** `git rev-parse
  origin/develop` returned `fac86e38383e2059e8795698e1585932c35b5b6d`. No
  intervening commit appeared, so the `STOP BLOCKED` condition on base movement
  did not fire.
- **The base is PR #813's merge commit**, and its parents are exactly the two
  expected SHAs, confirmed by `git log -1 --format='%H%n%P'` and by
  `gh pr view 813 --json state,mergeCommit`.
- **BE-02 and BE-03 statuses are tracker facts, not GitHub issue facts.** No
  GitHub issue is titled `BE-02`, `BE-03` or `BE-04`; the programme is tracked
  under master issue [#765](https://github.com/thoth-pub/thoth/issues/765) with
  per-task state held in `docs/publisher-services/task-status.md`. The statuses
  were therefore read from that tracker, which is the repository-authoritative
  location for them.

Durable across the whole of this task: `feature/publisher-services/be-04` was
absent at authoring time, was never created, and must not exist until separate
explicit CTO implementation authorization from a freshly verified `develop` head.

## 2. Scope confirmation

Approved specification: none exists for BE-04 by design. This is a
**specification-authoring task** following the BE-03-SPEC precedent, authorized
to produce the BE-04 specification candidate and its evidence/control records —
and explicitly **not** authorized to implement BE-04.

Implemented objective: produce the complete implementation-ready BE-04
specification candidate (`docs/engineering/ai-delivery/tasks/BE-04.md`), record
it durably in the programme tracker and changelog, and record this report.

Out-of-scope changes made: NONE.

Deliberately **not** done, each an explicit instruction boundary:

- no `feature/publisher-services/be-04` branch created;
- no runtime code written;
- no migration created, named, reserved or executed;
- `thoth-api/src/schema.rs` untouched;
- `thoth-api/src/policy.rs` untouched;
- no GraphQL runtime code altered;
- no worker role added to `policy.rs`;
- no client code generated;
- no deployment, no workflow dispatch, no job created, no automatic job creation
  activated, no production access;
- no cross-programme ADR created;
- `docs/publisher-services/decisions.md` deliberately **not** edited (section 5.3);
- no unrelated residual debt repaired.

## 3. Commits

- `3d552043775d133414a327622919753b73e13444` -
  `docs(publisher-services): specify BE-04 durable distribution jobs`
- `8686656855f8f8217fcb5176a19213d5586188ca` -
  `docs(publisher-services): record BE-04 specification candidate in programme controls`
- one further ordinary commit adds this report.

Ordinary commits only. No amend, no rebase, no squash, no force-push at any
point, and none is required.

## 4. Files changed

- `docs/engineering/ai-delivery/tasks/BE-04.md` **(NEW)**
  - reason: the BE-04 specification candidate required by this task.
  - behavioural effect: none. It states requirements; it changes no runtime
    behaviour, no schema, no contract and no authorization, and it authorizes no
    implementation.
- `docs/engineering/ai-delivery/implementation-reports/BE-04-SPEC-implementation-report.md` **(NEW)**
  - reason: the bounded evidence record for this specification task.
  - behavioural effect: none.
- `docs/publisher-services/task-status.md` **(MODIFIED)**
  - reason: record the BE-04 specification candidate durably, and keep BE-04's
    dependency and authorization state truthful.
  - behavioural effect: none. BE-04 remains `BLOCKED` and `NOT STARTED`; the
    BE-02 and BE-03 dependencies are recorded as satisfied without BE-04 becoming
    ready; implementation is recorded as `NOT AUTHORIZED`.
- `CHANGELOG.md` **(MODIFIED)**
  - reason: the repository requires every PR to update `## [Unreleased]`.
  - behavioural effect: none. One entry added under the existing `### Added`
    heading; no duplicate heading created.

Deliberately unmodified, as instructed: `docs/publisher-services/acceptance-matrix.md`,
`docs/publisher-services/rollout-plan.md`, `docs/publisher-services/README.md`,
`docs/publisher-services/decisions.md`, `docs/engineering/decisions/*`,
`BE-02.md`, `BE-03.md`, `BE-03-CLOSEOUT-01.md` and every historical
implementation report. The acceptance matrix and rollout plan already carry the
BE-04 controls this specification satisfies, so no edit was needed.

## 5. Implementation decisions

### 5.1 Decisions fixed by repository evidence

Every decision below is derived from merged code at the authoring base, recorded
in `BE-04.md` with its evidence, and is **not** left as `TBD`.

1. **The BE-03 seam is one named function.**
   `publisher_service_configuration::crud::replace_publisher_service_configuration`
   opens exactly one transaction on one connection and delegates its whole body
   to the private `replace_in_transaction`. BE-04 extends **that** transaction
   between the lifecycle writes and the single publisher `UPDATE`. Fixed by
   reading the merged coordinator, whose doc comment already states that BE-04
   extends it there.
2. **Job writes precede the publisher `UPDATE`, deliberately.** The publisher
   `UPDATE` fires `set_work_updated_at_with_relations`, whose single set-based
   `UPDATE work ... FROM imprint` takes row locks on all N of the publisher's
   works until commit. Inserting jobs before it shortens the widest part of the
   lock footprint at no cost.
3. **The deduplication identity is BE-02's `activation_id`.** `enable_on` mints
   one fresh `Uuid::new_v4()` and writes it to every linked member in one
   transaction, so it is already "one identity per logical activation". Deriving
   the key from it makes the one-job/two-target result, duplicate suppression and
   legitimate re-enable all fall out of one rule.
4. **The key is a single unique text column *and* is DB-verified against its
   formula.** `distribution_job_deduplication_key_key` gives kind-agnostic
   idempotency for future kinds; `distribution_job_deduplication_key_formula_check`
   proves the stored key equals
   `'PUBLISHER_BACK_CATALOGUE:' || publisher_id::text || ':' || activation_id::text`.
   The check uses only immutable expressions (enum inequality against a literal,
   `uuid::text`, `||`), so it is a genuine database guarantee and the text column
   cannot drift from the typed columns.
5. **A repair must not create a job, and the test is "was any member already
   enabled".** Members of a linked group share one adapter profile by
   construction — `OAPEN` and `DOAB` both resolve to `Profile::OapenDoabSword` —
   so any already-enabled member means the shared adapter was already active and
   the back catalogue already onboarded. `AssignmentLifecycleOutcome` is
   therefore widened to `Unchanged | Activated | Repaired | Disabled`, decidable
   from the `member_rows` `enable_on` already reads. BE-03 section 7.9 explicitly
   anticipated and permitted this widening.
6. **`AutomaticPush`/`PullFeed`/`Manual` is read from code-owned descriptors**,
   never inferred from names, and the qualifying-target rule filters group
   members by `back_catalogue_behaviour == AutomaticPush` with an empty result
   meaning "no job" — exhaustive and correct for a future mixed group, not only
   for today's inventory.
7. **The OFF switch reuses the merged mutation-guard convention exactly** —
   `clap::Arg` with `.env("THOTH_DISTRIBUTION_JOB_CREATION")`,
   `.default_value("OFF")`, `.value_parser(["OFF","ON"])`, registered on both
   `start graphql-api` **and** the image-default `init`, plumbed through
   `start_server` and `app_data`. Registering it on only one command is the exact
   defect `THOTH-GQL-OPS-02` had to fix, so the both-profile argument tests are
   required. Unlike the guard mode it must reach the resolver, so `Context` gains
   one field and the value is passed to the coordinator inside
   `ServiceConfigurationWriteContext` — preserving BE-03's rule that the
   coordinator takes policy as an explicit parameter and never infers it.
8. **`FOR UPDATE SKIP LOCKED` is selected against four recorded alternatives**
   (plain `FOR UPDATE`, optimistic update with retry, advisory locks, external
   queue), not defaulted to. It is already named as an approved primitive in
   `thoth-api/AGENTS.md` section 5.
9. **The claim-state check constraint is the load-bearing safety property.**
   Claim token, worker identity, claim time and lease expiry are non-null **if
   and only if** `status = 'RUNNING'`, which is what makes every stale-token
   statement structurally safe rather than safe by convention.
10. **`UNIQUE (claim_token)` on attempts** binds a token to exactly one attempt
    row for all time, so no statement can close a newer attempt.
11. **Worker identity is derived, not supplied.** `claimed_by` is
    `PolicyContext::user_id()`, exactly as BE-03 derives its audit `actor`.
    Accepting a worker-supplied identity would make the audit field spoofable.
12. **Lease-expiry recovery is performed by the claim call**, not by a scheduler.
    PostgreSQL runs no timer, and adding a background task would add process,
    permission and deployment surface for a recovery that is only useful when a
    worker is asking for work.
13. **No heartbeat/lease-extension operation.** A worker sizes its own lease
    within a clamped range, and an outlived lease is recovered and retried —
    which is safe precisely because the programme's acceptance matrix already
    requires DIS-02's worker to be at-least-once safe. Adding a fourth worker
    operation to avoid a hazard another task must already tolerate would be API
    invention.
14. **Retry is `failDistributionJob(retryable: true)`, with no separate
    mutation**, because only the worker knows retryability at failure time and
    the transition is fully expressed by the fail operation.
15. **A terminal `FAILED` job is not reopenable.** Recovery is a genuine
    re-activation, which mints a new `activation_id`, a new key and a new job
    through the same audited path. This keeps one creation path and avoids an
    operation that reopens a terminal state.
16. **`cancelDistributionJob` is added, and is justified by an existing approved
    requirement** — the rollout plan's stage 6 rollback requires cancelling
    pending pilot jobs and no other operator surface exists. It is superuser-only
    and fails closed from every terminal state.
17. **Assignment disable cancels `PENDING` jobs and leaves `RUNNING` jobs
    alone**, because cancelling cannot undo an upload already performed. The
    claim eligibility predicate additionally requires every target to still be
    enabled **under the job's own activation**, which is the database-side
    fail-closed backstop and is what stops a disable/re-enable cycle producing
    two live jobs for one destination.
18. **`SUPERUSER` is denied the three worker operations**, with five recorded
    reasons, while retaining cancellation and the staff report. `CDN_WRITE` reuse
    is explicitly rejected as simultaneously over-granting and under-scoping.
19. **"No job" is `null`, never a status value.** This follows BE-03's own merged
    precedent for `lastChange`, which is null rather than a fabricated
    placeholder.
20. **Bounds are code-owned constants, not runtime configuration**, because the
    retry budget, backoff curve and claim bound are correctness properties the
    tests pin. The only runtime-configurable value BE-04 adds is the on/off
    switch, because that alone is an activation decision.
21. **Three new ADR-0007 loaders**, not one shared one and not a second batching
    subsystem. BE-03's non-goal forbade a second loader *equivalent to BE-02's*;
    these have different keys, values and statements, and loader-first adoption
    is the approved pattern for a new N+1-prone field.
22. **`publisher_org_ids()` must skip `DISSEMINATION_WORKER` as well as
    `SUPERUSER`.** The merged function collects org ids from every role except
    the superuser key, so without this a worker account would appear to hold
    publisher organisations in the frontend switcher. This is a real behavioural
    detail found by reading the function, not a cosmetic note, and it carries its
    own required test.

### 5.2 Decisions recorded honestly rather than smoothed over

1. **"At least one target" is not database-enforced.** PostgreSQL cannot express
   it without a deferred constraint trigger, and BE-04 declines to add one. The
   specification says so plainly, enforces the invariant by single-site
   construction, and requires a `NOT EXISTS` assertion after every creation
   scenario. It does not claim database enforcement it does not have.
2. **Secret scrubbing is not attempted.** `error_detail` is bounded, truncated on
   character boundaries, and stripped of control characters — but the
   specification explicitly refuses to claim it detects secrets, because pattern
   matching produces false negatives that create false assurance and false
   positives that destroy diagnostics. Prohibited content is a contract on the
   writer, reviewed at DIS-02.
3. **Cancellation cannot undo an external upload.** Stated in the specification,
   in the field description requirement and in the operational section, rather
   than left for an operator to discover.
4. **`EntityNotFound` keeps its `INTERNAL_ERROR` mapping.** BE-03 already
   recorded that as a known limitation whose correction would alter merged
   contracts; BE-04 inherits the limitation rather than silently widening scope.
5. **The transaction's real write footprint is restated, not minimised.** BE-04
   inherits all four consequences BE-03 section 7.8 records, adds a bounded
   amount, and states that no production duration figure or safe catalogue size
   is derivable from the repository — with a stop condition rather than a guess
   if measurement is bad.
6. **`ON DELETE CASCADE` from `publisher` destroys job history with the
   publisher.** Consistent with BE-02 and BE-03, and recorded as a reviewed
   consequence of an already superuser-only destructive act.
7. **The `zitadel setup` role list already omits `WORK_LIFECYCLE` and
   `CDN_WRITE`.** Recorded as an observed gap and deliberately **not** repaired,
   per the instruction's prohibition on unrelated debt.

### 5.3 Unresolved decisions

**Programme-local decisions requiring CTO resolution before BE-04 is
implementable: NONE.** Every design decision the specification needed was
derivable from merged repository evidence, so
`docs/publisher-services/decisions.md` was deliberately **not** edited and no
`PROPOSED IN THIS SPECIFICATION CANDIDATE` entry was added. Adding one with
nothing genuinely unresolved would be noise in an active control document.

**One CTO judgement is surfaced rather than taken** — the cross-programme
adjacency finding of section 6, recorded in `BE-04.md` section 6.3 and as stop
condition 13. It does not block authoring, and on the evidence it does not block
implementation either; but whether it *should* is the CTO's call, not the
implementing agent's.

## 6. Cross-programme check

### 6.1 Conclusion

BE-04 as specified defines **programme-local durable distribution-job machinery
owned by `thoth-api`**, and establishes none of the prohibited shared
abstractions.

`thoth-api/AGENTS.md` section 1 already assigns "durable jobs, leases and audit
records implemented in Thoth" to this crate, and section 5 already names the
exact primitives BE-04 uses — unique constraints, foreign keys, check
constraints, row locks, leases with expiry, claim tokens, `FOR UPDATE SKIP
LOCKED` and deterministic idempotency keys. BE-04 is therefore a conventional
application of already-approved repository architecture.

Explicitly **not** created, and forbidden by `BE-04.md` non-goals 7 and 8 and
section 6.2:

- a generic job framework for unrelated programmes;
- a universal lease abstraction;
- a shared worker/service-role convention, `ServiceRole` type or role registry;
- a metrics-job abstraction;
- a universal queue;
- a new cross-programme identity model, machine-identity table, credential store
  or token-issuance mechanism.

Every type, table, column, enum, constant, module and role code is named for
distribution jobs and is unusable as a generic facility. `distribution_job_kind`
exists for **this programme's own** deferred kinds (work upsert, withdrawal), not
as an extension point for another programme.

### 6.2 Repository evidence establishing the position

- **No durable job machinery exists anywhere in this repository.** A search of
  `thoth-api/`, `thoth-api-server/`, `thoth-client/`, `thoth-errors/`,
  `thoth-export-server/` and `src/` for job tables, queues, claims, leases, claim
  tokens, `SKIP LOCKED`, idempotency or deduplication keys, retry state machines,
  cancellation primitives and machine worker APIs returns nothing. The only
  concurrency primitives present are `lock_publisher`'s `FOR UPDATE`
  (`model/publisher_distribution_platform/crud.rs`) and
  `pg_advisory_xact_lock(hashtext($1))` (`model/work_relation/crud.rs`). BE-04
  would be the first, which is why the boundary had to be stated explicitly
  rather than inherited.
- **No machine/service-role convention exists.** `policy.rs` defines five ZITADEL
  project roles. `SUPERUSER` is checked **unscoped** by `is_superuser()`; the
  other four are checked **scoped to an organisation** by `has_role_for_org`.
  There is no role inheritance. The unscoped-check pattern therefore already
  exists, applied to exactly one role, and `DISSEMINATION_WORKER` applies the
  same mechanism to a second role rather than inventing one.

### 6.3 The adjacency finding, surfaced for the CTO

`docs/metrics/task-status.md` records Thoth Metrics work package **WP5 - Service
auth and entitlements**, repositories `thoth` + clients, risk `CRITICAL`, status
`BLOCKED`, whose first listed blocking dependency is a **"role decision"**. That
concerns the same crate, the same `policy.rs`, the same `Role` enum and the same
ZITADEL project as BE-04's worker role.

Assessment, stated plainly:

1. **BE-04 does not require WP5's decision.** `DISSEMINATION_WORKER` reuses the
   existing unscoped-role pattern, adds one role, one predicate and one
   `require_*` helper, permits exactly three operations, and confers no publisher
   scope and no entitlement. Nothing in BE-04 answers "how do metrics clients
   authenticate and how are entitlements enforced", and nothing in BE-04
   forecloses any answer WP5 may choose.
2. **BE-04 is not blocked by WP5.** WP5 is blocked on its own dependencies (WP4,
   bounded slice specifications) independently of this task.
3. **What remains is precedence, not convention.** `DISSEMINATION_WORKER` would
   be the repository's first non-`SUPERUSER` unscoped project role, and WP5 will
   reasonably look at it. The specification deliberately does not convert
   precedence into a shared convention — see its section 6.2 boundary and
   non-goal 8.
4. **The judgement is the CTO's.** If the CTO decides the machine-identity
   question must be settled once for both programmes before either uses it, then
   BE-04's section 15 design is superseded and implementation is `BLOCKED` under
   `BE-04.md` stop condition 13 until that separate CTO-controlled decision
   exists.

No cross-programme ADR was created in this pull request, as instructed. If the
CTO takes reading 4, that ADR becomes its own bounded CTO-controlled task.

## 7. API and compatibility effects

GraphQL/API changes: **NONE**. This change is documentation and control records
only. No resolver, type, input, enum, argument or description is added, removed
or altered anywhere in the workspace.

Generated schema/client updates: **NONE**. `thoth-client/build.rs` produces the
same SDL as at the base, because no Rust GraphQL code changed.
`thoth-client/assets/queries.graphql` is unchanged.

Backwards compatibility: unaffected — there is no contract change to be
compatible with.

Deprecations: none.

Cross-repository dependencies: none created. `thoth-app` and
`thoth-dissemination` are untouched and unaffected; APP-01, APP-02, DIS-01 and
DIS-02 remain separately blocked with their recorded dependencies unchanged.

## 8. Authorization and security

Authorization paths changed: **NONE**. `thoth-api/src/policy.rs` is byte-identical
to the base. No role was added to the code, no ZITADEL project role was created
or granted, and no identity-provider configuration was read, changed or
approached.

Roles/scopes involved: the specification *describes* a proposed
`DISSEMINATION_WORKER` role and a complete least-privilege matrix. Describing a
role in a specification grants nothing and provisions nothing.

Negative authorization tests: not applicable to a documentation change. The
specification **requires** the full negative matrix — anonymous, authenticated
without role, `PUBLISHER_USER` (own and other publisher), `PUBLISHER_ADMIN`,
`WORK_LIFECYCLE`, `CDN_WRITE`, wrong machine role, superuser-without-worker-role,
and invalid/stale authentication — of the future implementation.

Secret or personal-data handling: none. No credential, token, endpoint, bucket,
host, account identifier or personal datum is recorded in any file changed by
this task. No production configuration or secret-bearing source was read.

Security limitations recorded rather than claimed absent: the specification does
not claim to scrub secrets from worker-supplied error text (section 5.2 item 2),
and it inherits BE-03's recorded `EntityNotFound` → `INTERNAL_ERROR` mapping.

## 9. Tests and checks

### Formatting and whitespace

Command:

```text
git diff --check
```

Result:

```text
clean - no whitespace errors, no output
```

### Documentation-only change verification

Command:

```text
git diff --stat fac86e38383e2059e8795698e1585932c35b5b6d..HEAD
```

Result:

```text
CHANGELOG.md
docs/engineering/ai-delivery/tasks/BE-04.md
docs/engineering/ai-delivery/implementation-reports/BE-04-SPEC-implementation-report.md
docs/publisher-services/task-status.md

path containment: CHANGELOG.md and docs/** only
```

### Path-containment verification

Command:

```text
git diff --name-only fac86e38..HEAD -- thoth-api/ thoth-api-server/ thoth-client/ \
    thoth-errors/ thoth-export-server/ .github/ Cargo.toml Cargo.lock src/ Makefile
```

Result:

```text
(no output) - zero changes under any code, workflow, manifest or build path
```

### Specific untouched-file verification

Command:

```text
git diff --name-only fac86e38..HEAD -- thoth-api/src/schema.rs thoth-api/src/policy.rs \
    thoth-api/migrations/
git status --porcelain thoth-api/migrations/
```

Result:

```text
(no output) - schema.rs unchanged, policy.rs unchanged, no migration created
```

### Branch verification

Command:

```text
git branch -a --list '*be-04*'
```

Result:

```text
feature/publisher-services/be-04-spec  (this specification branch only)
feature/publisher-services/be-04       ABSENT, never created
```

### Relative-link verification

Command:

```text
(extract every relative Markdown link target from each changed file and test it)
```

Result:

```text
BE-04.md: 22 distinct relative targets, all resolve
  AGENTS.md, thoth-api/AGENTS.md, task-specification-template.md,
  implementation-report-template.md, operating-model.md, design-references.md,
  control-gaps.md, ADR-0001/0002/0003/0004/0005/0007, BE-01.md, BE-02.md,
  BE-03.md, BE-03-CLOSEOUT-01.md, acceptance-matrix.md, decisions.md,
  platform-inventory.md, rollout-plan.md, task-status.md
task-status.md: all targets resolve, including the new BE-04.md link
0 broken links
```

### Unresolved-marker search

Command:

```text
grep -niE '\bTBD\b|\bTODO\b|FIXME|XXX|\?\?\?|to be decided|to be determined' BE-04.md
```

Result:

```text
2 matches, both prose about the absence of unresolved work, neither an
unresolved decision:
  line 79   "...no mandatory design decision is left as `TBD`..."
  line 3285 "...must not [...] return a synthetic [...] placeholder value"
0 unresolved implementation-critical decisions for job, lease, state, claim,
retry, cancellation, authorization, migration or API contract.
```

Every implementation-critical decision the instruction enumerates is fixed with a
concrete value in the specification: SQL types, nullability, foreign keys, unique
constraints, check constraints, indexes, the deduplication formula, the creation
matrix, the transaction step order, the complete transition graph, the claim
mechanism and its ordering/bounds/eligibility, the lease source and range, the
retry budget and backoff curve, cancellation semantics, the role code and matrix,
the API shape, the error variants, the error bounds and the migration procedure.
The **only** deliberately surfaced CTO matter is the cross-programme adjacency
finding of section 6.3, which is a judgement to raise, not a gap in the design.

### Rust workspace gate

Not run, and deliberately so: no Rust source, `Cargo.toml`, `Cargo.lock`,
migration or workflow file is touched by this change, so a workspace build,
clippy or test run would exercise nothing this task altered. The
documentation-only evidence set required by the root `AGENTS.md` section 8 is the
applicable gate and is discharged above. Repository CI is the independent check
and its actual result is the GitHub record.

## 10. Manual verification

Environment: local checkout at
`fac86e38383e2059e8795698e1585932c35b5b6d`, branch
`feature/publisher-services/be-04-spec`. No database, no service, no deployment,
no production access.

Steps: read every authoritative source listed in `BE-04.md` section 2.1; re-derive
each of the 25 observed repository facts in `BE-04.md` section 2.2 from merged
code, migrations and generated contracts; author the specification; verify links,
whitespace, path containment and untouched files as recorded in section 9.

Observed result: the specification is internally consistent, every required
template field is present (section 0 conformance table), and no
implementation-critical decision is unresolved.

Evidence reference: the diff on the pull request, and this report.

## 11. CI

CI status: to be reported from the actual GitHub checks on the pull request. This
committed report does not predict or transcribe it, and no commit will be created
merely to record it (ADR-0005). Because the change is documentation-only,
classified or skipped checks may be expected; the actual result is whatever
GitHub records, and GitHub remains the terminal live authority.

## 12. Rollout and rollback

Initial state after merge: **repository history changes only.** The BE-04
specification becomes repository-authoritative. Nothing is deployed, no migration
exists or is executed, no relation is created, no job exists, no role is
provisioned, no worker exists, automatic job creation does not exist let alone
activate, and no API behaviour changes for any client.

Activation required: not applicable. There is nothing to activate.

Feature flag/configuration: none. The `THOTH_DISTRIBUTION_JOB_CREATION` switch is
**specified**, not implemented; it does not exist in any binary.

Migration sequence: none. No migration is created or executed by this task.

Rollback/disable procedure: an ordinary code revert of the pull request under
normal review. No data, runtime or external state exists to roll back.

Monitoring required: none.

## 13. Known limitations and deferred work

1. **This is a specification, not an implementation.** Every behaviour it
   describes is unimplemented. `BE-04.md` remains `DRAFT` and its implementation
   authorization is `SEPARATE AND ABSENT`.
2. **The cross-programme adjacency finding is surfaced, not resolved** (section
   6.3). It is a CTO judgement, and it may supersede the specification's
   authorization design under stop condition 13.
3. **No heartbeat or lease-extension operation is specified.** Deferred to DIS-02
   with its reason recorded, if measurement shows it necessary.
4. **A terminal `FAILED` job cannot be reopened.** Recovery is a genuine
   re-activation. Recorded as a deferred capability with its rationale, not an
   oversight.
5. **`EntityNotFound` keeps its inherited `INTERNAL_ERROR` GraphQL mapping**, per
   BE-03's recorded known limitation.
6. **The `work_id` `ON DELETE CASCADE` choice is unobservable in BE-04** because
   no row populates the column; a future work-level job task must revisit it
   explicitly.
7. **The `thoth zitadel setup` role list already omits `WORK_LIFECYCLE` and
   `CDN_WRITE`.** Observed and deliberately unrepaired here.
8. **No production duration figure or safe catalogue-size threshold is stated**,
   because none is derivable from the repository. The specification requires
   measurement in a disposable environment and a stop condition if the result is
   bad.
9. **`docs/publisher-services/decisions.md` is unedited**, because no genuinely
   unresolved programme-local decision was discovered.

## 14. Unresolved issues

- **NONE that block specification review.** One CTO judgement is surfaced for
  decision — the cross-programme adjacency finding of section 6.3 — and is
  recorded in `BE-04.md` section 6.3 and stop condition 13 rather than resolved
  by this task.

## 15. Agent self-assessment

The agent may identify risks but may not approve the task. **No approval decision
is issued here.**

Suggested review focus, ordered by where an error would cost most:

1. **The activation classification of `BE-04.md` section 9.1** — whether
   `Activated` versus `Repaired` is correctly decidable from the rows `enable_on`
   already reads, and whether "any member already enabled means repair" is sound
   for every linked-group state. A wrong classification either re-pushes a back
   catalogue on a cosmetic repair, or fails to onboard a genuine re-activation.
2. **The deduplication formula and its check constraint** — whether
   `distribution_job_deduplication_key_formula_check` is accepted by the target
   PostgreSQL version and whether every expression in it is genuinely immutable.
3. **The claim eligibility predicate of section 12.4** — specifically the
   `activation_id` match, which is what prevents a disable/re-enable cycle from
   producing two live jobs, and whether requiring **all** targets to qualify has
   an unintended consequence.
4. **The interaction between assignment disable and running jobs** (section
   14.3) — whether cancelling `PENDING` while leaving `RUNNING` is the right
   split, and whether the post-expiry unclaimable state is acceptable or should
   instead terminalize.
5. **The worker authorization matrix** (section 15.2), especially the deliberate
   `SUPERUSER` denial of the three worker operations, and the
   `publisher_org_ids()` change.
6. **The cross-programme adjacency finding** (section 6.3) — whether the
   reviewer agrees the worker role is genuinely programme-local, or considers it
   a shared service-role convention that must be settled first.
7. **The transaction step placement** (section 10.1) — whether inserting job
   writes at 9a–9c, before the publisher `UPDATE`, is correct and whether any
   ordering consequence was missed.
8. **The claimed non-goals** — whether the specification anywhere smuggles in a
   generic framework, a second configuration transaction, an observed-delivery
   concept, a fabricated job status, or scope belonging to MIG-01, APP-01,
   APP-02, DIS-01 or DIS-02.
