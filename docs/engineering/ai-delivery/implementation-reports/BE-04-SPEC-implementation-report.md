# BE-04-SPEC Implementation Report

## 1. Repository state

Repository: `thoth-pub/thoth`
Workflow: STANDARD
Base branch: `develop`
Base commit: `fac86e38383e2059e8795698e1585932c35b5b6d` (original
specification-authoring base; preserved as historical authoring evidence)
Repository-authoritative control state at remediation:
`8703dd5ca2080bb97debc9d14cca33db9956f7b4` (merge commit of `ADR-0008` PR
[#815](https://github.com/thoth-pub/thoth/pull/815)), merged into this branch by
the ordinary merge commit `1cf5675c4c2f065feab8ccfb3cde06c368588aa6`
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
- one further ordinary commit adds this report;
- `1cf5675c4c2f065feab8ccfb3cde06c368588aa6` - ordinary merge of
  repository-authoritative `develop` (`8703dd5c`, `ADR-0008` through PR #815)
  into this branch, after `ADR-0008` became repository-authoritative;
- `03b604c49fcc5eba2f70f5a2711c33f314d595df` -
  `docs(publisher-services): remediate BE-04 specification review findings` —
  the first review round's four findings (section 5.4.1) and the `ADR-0008`
  reconciliation;
- one further ordinary commit carries the second round's five findings (section
  5.4.2): the migration referenced-table locking model, the `lastError`
  semantics, the report statement-count arithmetic, the narrowed
  role-composition wording and the invalid-`errorCode` API contract.

Ordinary commits only. No amend, no rebase, no squash, no force-push at any
point, and none is required.

## 4. Files changed

- `docs/engineering/ai-delivery/tasks/BE-04.md` **(NEW, subsequently REMEDIATED TWICE)**
  - reason: the BE-04 specification candidate required by this task; then
    corrected for the first review round's four findings (section 5.4.1) and
    reconciled with the now-repository-authoritative `ADR-0008` (section 6.3);
    then corrected for the fresh review round's five findings (section 5.4.2).
  - behavioural effect: none. It states requirements; it changes no runtime
    behaviour, no schema, no contract and no authorization, and it authorizes no
    implementation.
- `docs/engineering/ai-delivery/implementation-reports/BE-04-SPEC-implementation-report.md` **(NEW, subsequently REMEDIATED TWICE)**
  - reason: the bounded evidence record for this specification task, extended
    with both review rounds' remediation records and the `ADR-0008`
    reconciliation.
  - behavioural effect: none.
- `docs/publisher-services/task-status.md` **(MODIFIED)**
  - reason: record the BE-04 specification candidate durably, and keep BE-04's
    dependency and authorization state truthful.
  - behavioural effect: none. BE-04 remains `BLOCKED` and `NOT STARTED`; the
    BE-02, BE-03 and `ADR-0008` dependencies are recorded as satisfied without
    BE-04 becoming ready; implementation is recorded as `NOT AUTHORIZED`.
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
5. **A repair must not create an automatic onboarding job, and the test is "was
   any member already enabled".** `Activated` and `Repaired` name **desired-state
   events**: `Activated` is a group moving from zero enabled members to a newly
   enabled desired-state group; `Repaired` is normalization of a group in which
   at least one member was already enabled. Automatic creation is tied to a
   **new** `Activated` event, so a repair creates no job because it is not a new
   zero-enabled-to-enabled activation — and for no other reason. This makes **no
   inference about observed delivery**: a linked group's shared adapter profile
   (`OAPEN` and `DOAB` both resolve to `Profile::OapenDoabSword`) is why one
   activation is one unit of work, not evidence that the profile was ever
   executed. `AssignmentLifecycleOutcome` is therefore widened to
   `Unchanged | Activated | Repaired | Disabled`, decidable from the
   `member_rows` `enable_on` already reads. BE-03 section 7.9 explicitly
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
12. **Lease-expiry recovery is performed by the claim call**, not by a scheduler,
    and is **split by the attempt budget**. PostgreSQL runs no timer, and adding
    a background task would add process, permission and deployment surface for a
    recovery that is only useful when a worker is asking for work. Recovery
    within budget returns the job to `PENDING` (T5a); recovery at or beyond the
    budget transitions it directly to `FAILED` (T5b), so an expired fifth attempt
    cannot become a sixth.
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
8. **`OFF` is a fail-closed refusal, not a silent skip.** Recorded plainly,
   including the operational consequence that a superuser attempting an
   `AutomaticPush` activation while automatic creation is disabled sees a
   refusal, and including the deliberate asymmetry with `MIGRATION_BACKFILL`,
   which stays job-free *by design* and therefore commits normally.
9. **"No job" carries no delivery meaning.** The specification states that a
   `null` `latestBackCatalogueJob` — for a repaired group, a pre-feature
   assignment or any other reason — means only that BE-04 holds no durable job,
   and that BE-04 neither stores nor infers observed delivery state.

### 5.3 Unresolved decisions

**Programme-local decisions requiring CTO resolution before BE-04 is
implementable: NONE.** Every design decision the specification needed was
derivable from merged repository evidence, so
`docs/publisher-services/decisions.md` was deliberately **not** edited and no
`PROPOSED IN THIS SPECIFICATION CANDIDATE` entry was added. Adding one with
nothing genuinely unresolved would be noise in an active control document.

**The one cross-programme matter this task surfaced rather than took is now
settled.** It was escalated to the CTO, decided in
[`ADR-0008`](../../decisions/ADR-0008-machine-roles-and-durable-job-primitives.md)
on 2026-08-14, and became repository-authoritative through PR
[#815](https://github.com/thoth-pub/thoth/pull/815). `BE-04.md` section 6.3 now
records that resolution as a durable boundary rather than an open question, and
former stop condition 13 has been replaced by an ADR-0008 compliance condition.
Satisfying ADR-0008 approves no part of this specification and authorizes no
implementation.

### 5.4 Review-finding remediation

The specification candidate has been through **two** independent review rounds on
this branch. Both rounds' findings are corrected here, and the specification
remains **not approved** after both.

#### 5.4.1 First round — four findings

Independent review of the specification candidate identified four findings. All
four are corrected in this branch; the specification remains **not approved**.

1. **`OFF` mode could lose an onboarding.** The former section 9.4 permitted a
   `SUPERUSER_API` `AutomaticPush` activation to commit with no job while
   automatic creation was `OFF`, after which no sweep, retry or later replacement
   would ever create one — a permanently un-onboarded publisher whose
   configuration read as correct. Section 9.4 is rewritten: such a transaction
   now **fails and rolls back in full**, with an enumerated zero-committed-change
   result (assignments, `activation_id`, configuration token, publisher row,
   audit row, job and target rows). `PullFeed`, `Manual`, package-only, repair,
   disable and `MIGRATION_BACKFILL` writes remain permitted, the last of these
   as an explicit migration boundary rather than a missed onboarding. No global
   sweep is introduced and turning the switch `ON` still enqueues nothing
   retroactively. One bounded new error,
   `ThothError::DistributionJobCreationDisabled` →
   `DISTRIBUTION_JOB_CREATION_DISABLED`, carries the public contract; the
   repository had no existing feature-disabled error, and reusing
   `StalePublisherServiceConfiguration` or `DistributionPlatformNotAssignable`
   would have distorted a merged contract. Section 16.3's count moves from two
   new errors to three, and scope item 10 with it.
2. **Five attempts could become six.** Lease-expiry recovery unconditionally
   returned an expired `RUNNING` job to `PENDING` without consulting the attempt
   budget, so an expired fifth attempt was claimable and T1 incremented to six.
   T5 is split into **T5a** (`attempt_count < MAX` ⇒ `PENDING`, `available_at`
   now, count unchanged) and **T5b** (`attempt_count >= MAX` ⇒ `FAILED`,
   `completed_at` set, never claimable again), decided inside the recovery
   statement from the row's own count. Claim eligibility independently requires
   `attempt_count < DISTRIBUTION_JOB_MAX_ATTEMPTS`, and
   `distribution_job_attempt_count_check` becomes
   `attempt_count >= 0 AND attempt_count <= 5`, making the previously overstated
   "hard-bounded" claim a database property. A test ties the Rust constant to the
   migration literal.
3. **Repair was treated as evidence of delivery.** The creation matrix justified
   `Repaired ⇒ no job` with "the adapter was already active", which desired-state
   rows cannot establish. The rule is unchanged — a repair creates no automatic
   job — but its stated reason is now the correct one: a repair is not a new
   zero-enabled-to-enabled activation. Sections 9.1, 9.2 and 17.3 now state
   explicitly that no inference about adapter execution, external upload or
   back-catalogue delivery is made, that `latestBackCatalogueJob: null` means
   only "no durable job", and that any future policy for onboarding historical or
   unknown-delivery state is separate and unauthorized here.
4. **The claim SQL returned no rows.** The normative section 12.3 statement ended
   in `INSERT ... SELECT FROM claimed`, which produces no result set, so
   `claimDistributionJobs` had nothing to return without a second, non-atomic
   query. The statement now carries an `inserted_attempts` CTE and a final
   `SELECT ... FROM claimed JOIN inserted_attempts` with the deterministic
   `ORDER BY`, returning exactly the jobs this invocation claimed — zero rows
   when nothing was claimed — in one atomic statement, with `SKIP LOCKED` and its
   justification retained. Target and payload resolution is specified as a
   bounded set of `= ANY($ids)` statements with no N+1 path.

The ADR-0008 reconciliation performed alongside those four is recorded in
section 6.

#### 5.4.2 Second round — five further findings

A **fresh independent full review** of the remediated specification identified
five further findings. All five are corrected in this branch. The four findings
above are preserved as corrected and were re-checked for regression; the
corrections below touch their wording only where consistency required it. The
specification is still **not approved**.

1. **The migration locking model was wrong.** Section 19.3 claimed the migration
   "takes **no lock on any existing table**" because every object it creates is
   new. That is false. `distribution_job` declares foreign keys to the existing
   `public.publisher` and `public.work`, and under PostgreSQL 17 establishing a
   foreign-key constraint takes a `SHARE ROW EXCLUSIVE` lock on the **referenced**
   table for the duration of the transaction. Section 19.3 now states plainly
   that the migration **does** acquire table-level locks on `publisher` and
   `work`, names the mode, and states what it blocks (concurrent writes, not
   reads) and what it can wait behind. The genuinely true parts are preserved and
   kept distinct: the `distribution_job*` relations are new and empty, no
   existing application table is rewritten, there is no backfill or validation
   scan, and the migration creates zero job rows. The foreign keys are **not**
   weakened, deferred, made `NOT VALID` or dropped to make the finding go away —
   that is stated as explicitly unauthorized. No production duration is claimed.
   Section 25.3 now requires observed `pg_locks` entries captured from a second
   session (naming `publisher` and `work` with their modes), the migration
   duration, a deterministic lock-contention fixture showing what happens when
   another session already holds a conflicting writer lock, the `relfilenode`
   proof that no rewrite occurred despite the locks, and the down migration on a
   populated database; section 26 item 2 and section 25.1 carry the matching
   evidence obligations. Sections 19.3 and 22 now state that production migration
   authorization must account for that lock window, with disposable-environment
   measurements as inputs to the decision rather than a production prediction.
2. **`lastError` semantics were self-contradictory.** Section 7.2 said
   `last_error_code`/`last_error_detail` "mirror the most recent attempt's
   values", but T5a and T5b both close the newest attempt as `ABANDONED` and
   leave `last_error_*` untouched, so the fields demonstrably did not mirror the
   newest attempt. One coherent semantic is now fixed: **the most recent
   worker-reported failure of the job** — set by T3 and T4, cleared by T2, and
   untouched by T5a, T5b, T6, T7 and T8. Its consequences are stated rather than
   left to be discovered: a T5b-terminalized job may legitimately have a null
   `lastError` and that null must not be patched with a synthetic code; a
   T5b-terminalized job with an earlier worker failure retains that earlier
   failure, which is explicitly **not** the reason the final attempt was
   abandoned and **not** the terminalization cause; attempt history remains the
   authoritative record that the final attempt was `ABANDONED` on lease expiry;
   and cancellation neither sets nor clears the fields. The mirror alternative
   was rejected on its own terms — `distribution_job_attempt_error_result_check`
   forbids error fields on a non-`FAILED` attempt, so mirroring an `ABANDONED`
   attempt could only be done by inventing an error no worker reported.
   Reconciled across sections 7.2, 11.2, 14.1, 16.2, 17.1, 18.2, 25.1, 25.10 and
   26 item 8, including six new required tests (T5a and T5b each with and without
   a previous `FAILED` attempt, success clearing, and cancellation).
3. **The staff-report statement count did not add up.** Section 17.4 claimed a
   five-statement bound while its own table described a publisher page, a
   configuration-change statement, the assignment loader, the latest-job loader,
   the target loader and the attempt loader. The target and attempt loaders are
   separate ADR-0007 DataLoaders and each issues its own set-based statement, so
   the full-field bound is **six**. The minimum correction was taken: the bound
   is now six, target and attempt are separate numbered rows, all six remain
   set-based and constant in N, and the three new loaders are preserved. The
   loaders are explicitly **not** combined to restore the number five. The bound
   is also now stated as per dispatch chunk, with the exact arithmetic required
   if any allowed page size ever produces more than one chunk, rather than
   assuming a single chunk. Sections 25.1, 25.12 and 26 item 16 now require the
   measurement at page sizes 1, 25 and 200 to use the **full job-aware selection
   set** and to equal the stated bound.
4. **The role-composition wording overreached ADR-0008.** Section 15.3 item 5
   said "roles compose additively in the merged model", which asserts a general
   rule ADR-0008 deliberately declines to make. It is replaced with the narrow,
   BE-04-owned statement: the BE-04 authorization matrix explicitly permits a
   principal holding both `SUPERUSER` and `DISSEMINATION_WORKER` to exercise the
   independently authorized operations of both roles, and **this is a
   BE-04-specific matrix decision that establishes no general role-composition,
   aggregation or inheritance rule**. `SUPERUSER` alone still receives no worker
   operations, `DISSEMINATION_WORKER` alone still receives no administrative
   ones, no Metrics consequence follows, and no generic machine-role rule beyond
   ADR-0008 is created. A sweep of the specification, this report and the PR body
   found no other equivalent general claim; section 25.11's existing "no
   composition rule introduced" assertion already agreed with the narrowed
   wording.
5. **The invalid-`errorCode` contract was unspecified.** Sections 18.1 and 25.10
   required a malformed or over-length worker `error_code` to be rejected with a
   "stable error", while section 16.3 fixed exactly three new `ThothError`
   variants and named none of them for this case — leaving the implementing agent
   to invent the contract or fall through to `INTERNAL_ERROR`. The merged
   `thoth-errors` model was inspected. Reuse was rejected on the evidence:
   `InvalidSubjectCode` is subject-code specific **and echoes the caller's input
   back**; `InvalidUuid`, `InvalidTimestamp`, `InvalidFileExtension` and
   `InvalidMetadataSpecification` name different subjects; and every other
   candidate falls through `into_field_error`'s catch-all arm to
   `INTERNAL_ERROR`, which is not acceptable for a deliberately specified
   validation contract (it would make a client contract violation
   indistinguishable from a database outage and invite an automated worker to
   retry it for ever). One bounded variant is therefore added:
   `ThothError::InvalidDistributionJobErrorCode` →
   `INVALID_DISTRIBUTION_JOB_ERROR_CODE`, with a fixed sanitized public message
   that echoes no part of the rejected value, raised at resolver entry so **no
   job or attempt state changes** and the claim token stays valid. Section 16.3's
   count moves from three to four variants and arms, and scope item 10, section
   25.1, section 25.10 and section 26 item 14 move with it. One factual
   correction was made while specifying it: `CompleteDistributionJobInput`
   carries no `errorCode`, so `failDistributionJob` is the only operation that
   can raise this error, and **no `errorCode` field is added to the complete
   input** to manufacture symmetry.

**Provisioning wording (consistency sweep only).** Section 15.5's existing
statement — that BE-04 implementation *may* add `DISSEMINATION_WORKER` to the
`zitadel.rs` `setup` role list but must not run the command, grant the role or
change any identity-provider configuration — was retained rather than rewritten.
One clarifying sentence separates the two halves explicitly: editing the list is
an ordinary repository source change inside BE-04's implementation scope, while
executing `zitadel setup`, creating the role, granting it and issuing or rotating
credentials are separately authorized operational actions outside this
specification's authority. No provisioning architecture is invented, and no scope
is widened.

## 6. Cross-programme check and `ADR-0008` reconciliation

### 6.1 Conclusion

BE-04 as specified defines **programme-local durable distribution-job machinery
owned by `thoth-api`**, and establishes none of the prohibited shared
abstractions. That position is now not only this specification's own claim: it is
what [`ADR-0008`](../../decisions/ADR-0008-machine-roles-and-durable-job-primitives.md)
section 3.4 decides.

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

### 6.3 The cross-programme question, escalated and now resolved

The cross-programme machine-role and durable-job question was **escalated rather
than decided by this specification**, and the CTO has since decided it in
`ADR-0008`, approved on 2026-08-14 and repository-authoritative through PR
[#815](https://github.com/thoth-pub/thoth/pull/815). `BE-04.md` section 6.3 now
records the resulting boundary durably; it is no longer an open finding, a
precedence observation or a pending judgement.

What the specification consumes, exactly and without broadening it:

1. **Domain-specific, least-privilege machine roles**, with no generic
   `SERVICE`/`MACHINE`/`WORKER`/`SERVICE_ACCOUNT` catch-all; an unscoped role only
   for a genuinely global workload; and an explicit guard, an explicit
   authorization matrix and least privilege for each. `SUPERUSER` authority does
   not automatically imply machine-role authority.
2. **`DISSEMINATION_WORKER` is approved as Publisher-Services-specific.**
   `ADR-0008` does not fix its operation-level matrix; `BE-04.md` section 15.2
   still owns it, and the specification now states that denying `SUPERUSER` the
   three worker operations is BE-04's own least-privilege choice rather than
   something `ADR-0008` requires either way.
3. **Thoth Metrics WP5 does not use the role** and inherits none of its
   permissions, scope or semantics; its eventual role name and permissions are
   its own work under its own approved specification. WP5 remains `CRITICAL` and
   `BLOCKED`.
4. **BE-04's `distribution_job*` tables, Rust domain types and lifecycle APIs
   remain programme-local**, not reusable cross-programme by analogy.
5. **There is no generic shared job framework.** `ADR-0008` section 3.3 approves
   **exactly seven** conventions — PostgreSQL durability, explicit state
   machines, database uniqueness, leases, claim tokens, deterministic
   idempotency, and `FOR UPDATE SKIP LOCKED` where justified — and that list is
   exhaustive. A new `BE-04.md` section 6.4 enumerates those seven and then
   separately attributes BE-04's own requirements — stale-token rejection,
   deterministic ordering, database-enforced concurrency, bounded lease
   semantics, the deduplication formula, the GraphQL worker operations and
   protocol, the permitted/forbidden operation lists, and credential
   provisioning — to BE-04's HIGH-risk requirements, to `thoth-api/AGENTS.md`, or
   to outside the task, rather than presenting any of them as additional
   `ADR-0008`-approved cross-programme architecture.
6. **The ADR prerequisite for BE-04 implementation is satisfied**, and is
   **necessary but not sufficient**. `ADR-0008` approves no part of this
   specification candidate and authorizes no implementation, no role creation, no
   provisioning, no identity-provider change, no migration and no deployment.

Former stop condition 13 — which contemplated the CTO later deciding that machine
identity must be settled first — is removed as spent and replaced by an
**ADR-0008 compliance** condition: implementation returns `BLOCKED` if it would
require violating `ADR-0008`, introducing a generic machine role, reusing BE-04's
job machinery cross-programme, creating a generic reusable job/queue abstraction
without a later ADR, or broadening the seven approved conventions.

No cross-programme ADR was created in this pull request. `ADR-0008` was authored,
reviewed and merged as its own separate bounded task, and neither it nor PR #815
is modified here.

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

Roles/scopes involved: the specification *describes* a `DISSEMINATION_WORKER`
role and a complete least-privilege matrix. `ADR-0008` approves that role as
Publisher-Services-specific and leaves its operation-level matrix to this
specification; approving an architecture and describing a role in a specification
both grant nothing and provision nothing. No role was created in code, none was
granted, and no identity-provider configuration was read, changed or approached.

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
That inherited limitation is **not** extended to the new validated worker input
contract: a malformed or over-length `errorCode` is specified to return
`INVALID_DISTRIBUTION_JOB_ERROR_CODE` rather than `INTERNAL_ERROR`, with a fixed
message that reflects no part of the caller's value and with no job or attempt
state change (section 5.4.2 finding 5).

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

**The baseline for this check is `origin/develop`, not the original authoring
base.** Once the base-reconciliation merge (`1cf5675c`) brought
repository-authoritative `develop` into this branch, a diff from the authoring
base `fac86e38` to `HEAD` also contains the `ADR-0008` files that arrived through
PR #815 — they are `develop`'s content, not this PR's contribution. The
three-dot diff against `origin/develop` is what states what this pull request
actually changes.

Command:

```text
git diff --name-status origin/develop...HEAD
```

Result:

```text
M	CHANGELOG.md
A	docs/engineering/ai-delivery/implementation-reports/BE-04-SPEC-implementation-report.md
A	docs/engineering/ai-delivery/tasks/BE-04.md
M	docs/publisher-services/task-status.md

path containment: CHANGELOG.md and docs/** only
```

### Path-containment verification

Command:

```text
git diff --name-only origin/develop...HEAD -- thoth-api/ thoth-api-server/ \
    thoth-client/ thoth-errors/ thoth-export-server/ .github/ Cargo.toml \
    Cargo.lock src/ Makefile
```

Result:

```text
(no output) - zero changes under any code, workflow, manifest or build path
```

### Specific untouched-file verification

Command:

```text
git diff --name-only origin/develop...HEAD -- thoth-api/src/schema.rs \
    thoth-api/src/policy.rs thoth-api/migrations/
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
BE-04.md: 23 distinct relative targets, all resolve
  AGENTS.md, thoth-api/AGENTS.md, task-specification-template.md,
  implementation-report-template.md, operating-model.md, design-references.md,
  control-gaps.md, ADR-0001/0002/0003/0004/0005/0007/0008, BE-01.md, BE-02.md,
  BE-03.md, BE-03-CLOSEOUT-01.md, acceptance-matrix.md, decisions.md,
  platform-inventory.md, rollout-plan.md, task-status.md
task-status.md: all targets resolve, including the BE-04.md and ADR-0008 links
this report: all targets resolve, including the ADR-0008 link
0 broken links
```

The ADR-0008 target is the one added by the remediation; the other 22 are the
authoring-time set and are unchanged.

### Unresolved-marker search

Command:

```text
grep -nEi '\bTBD\b|\bTODO\b|FIXME|\bXXX\b|\?\?\?|to be decided|to be determined|placeholder value' \
  docs/engineering/ai-delivery/tasks/BE-04.md
```

Result:

```text
2 matches, both prose about the absence of unresolved work, neither an
unresolved decision (line numbers at the remediated head):
  line 100  "...no mandatory design decision is left as `TBD`..."
  line 4079 "...must not [...] return a synthetic [...] placeholder value"
0 unresolved implementation-critical decisions for job, lease, state, claim,
retry, cancellation, authorization, migration or API contract.
```

Every implementation-critical decision the instruction enumerates is fixed with a
concrete value in the specification: SQL types, nullability, foreign keys, unique
constraints, check constraints (including the `attempt_count` upper bound),
indexes, the deduplication formula, the creation matrix, the `OFF`-mode
fail-closed rule and its rollback result, the transaction step order, the
complete transition graph including the T5a/T5b budget split, the claim mechanism
and its ordering/bounds/eligibility, the claim statement's return contract, the
lease source and range, the retry budget and backoff curve, cancellation
semantics, the role code and matrix, the API shape, the three error variants, the
error bounds and the migration procedure. No CTO matter is left surfaced: the
cross-programme question is decided by `ADR-0008` (section 6.3).

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
   authorization is `SEPARATE AND ABSENT`. The specification is **not approved**:
   independent review identified remediation requirements, which this change
   addresses; fresh exact-head independent review and explicit CTO specification
   approval both remain required.
2. **The cross-programme question is resolved by `ADR-0008`, not by this
   specification** (section 6.3). Satisfying it is necessary and not sufficient:
   it approves nothing here and authorizes no implementation.
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
   bad. This now explicitly includes the migration's lock window: BE-04's future
   migration **will** take `SHARE ROW EXCLUSIVE` locks on the existing
   `public.publisher` and `public.work` tables while its two foreign keys are
   established (section 5.4.2 finding 1), and how long that window is acceptable
   in production is a separate release-authorization decision informed by, never
   replaced by, disposable-environment measurement.
9. **`docs/publisher-services/decisions.md` is unedited**, because no genuinely
   unresolved programme-local decision was discovered.
10. **`OFF` mode is a refusal, and that is a real operational cost.** While
    automatic creation is disabled, an `AutomaticPush` activation cannot be
    recorded at all. That is the deliberate trade against silently losing an
    onboarding, and it is recorded in the rollout and observability sections so a
    runbook can carry it rather than an operator discovering it.
11. **A future policy for manually onboarding historical or unknown-delivery
    repaired state is not specified here.** BE-04 neither provides nor forecloses
    it; it would require its own specification and its own authorization.

## 14. Unresolved issues

- **NONE that block specification review.** The one cross-programme matter this
  task surfaced has been decided by the CTO in `ADR-0008` and is recorded as a
  durable boundary in `BE-04.md` section 6.3; the first round's four
  independent-review findings are remediated in section 5.4.1 and the second
  round's five in section 5.4.2. The specification is **not approved**, and fresh
  exact-head independent review plus explicit CTO specification approval remain
  required.

## 15. Agent self-assessment

The agent may identify risks but may not approve the task. **No approval decision
is issued here.**

Suggested review focus, ordered by where an error would cost most. The five most
recently corrected areas come first, because they are the least-reviewed text in
the document and because the first of them was a factually wrong claim that
survived a full review round:

1. **The migration locking model of section 19.3** — whether `SHARE ROW
   EXCLUSIVE` on the referenced table is the correct PostgreSQL 17 behaviour as
   stated, whether the stated blocking consequences (writes blocked, reads not)
   and wait behaviour are right, whether keeping both foreign keys is the correct
   trade against a shorter lock window, and whether section 25.3's required
   evidence — second-session `pg_locks` capture, duration, contention fixture,
   `relfilenode` proof — is actually sufficient to falsify the claim if it is
   still wrong.
2. **The `lastError` semantic of section 11.2** — whether "most recent
   worker-reported failure" is the right choice against the alternatives, whether
   it is now represented consistently by the state machine, the field
   descriptions, the report and the tests, and in particular whether a `FAILED`
   job with a null `lastError` (T5b with no prior reported failure) is acceptable
   to the operators and future APP-02 surfaces that will read it.
3. **The report statement-count bound of section 17.4** — whether six is now the
   correct arithmetic for the full job-aware selection set, whether keeping the
   target and attempt loaders separate is right, and whether the per-dispatch-chunk
   qualification covers every supported page size.
4. **The invalid-`errorCode` contract of section 16.3** — whether adding a fourth
   `ThothError` variant is the right call against reuse, whether
   `InvalidDistributionJobErrorCode`/`INVALID_DISTRIBUTION_JOB_ERROR_CODE` follows
   repository naming conventions, whether the fixed message and no-state-change
   guarantee are complete, and whether leaving `completeDistributionJob` without
   an `errorCode` input is correct.
5. **The narrowed role-composition wording of section 15.3 item 5** — whether it
   is now genuinely a BE-04-local matrix statement and creates no general
   composition, aggregation or inheritance rule, and whether any equivalent claim
   survives elsewhere.
6. **The `OFF`-mode fail-closed rule of `BE-04.md` section 9.4** — whether
   returning `DistributionJobCreationDisabled` at step 9a′ genuinely rolls back
   every lifecycle write step 9 made, whether the enumerated zero-committed-change
   result is complete, and whether the permitted set (`PullFeed`, `Manual`,
   package-only, repair, disable, `MIGRATION_BACKFILL`) is the right boundary.
7. **The attempt-budget split of section 11.2 (T5a/T5b) and its three guards** —
   whether the recovery statement's `CASE` correctly satisfies
   `distribution_job_completed_at_check` in both branches, whether the eligibility
   clause and the `attempt_count <= 5` check are together sufficient to make a
   sixth attempt unreachable, and whether terminalizing an expired fifth attempt
   is the right operational choice.
8. **The claim statement of section 12.3** — whether the
   `claimed`/`inserted_attempts` CTE shape returns exactly the claimed rows under
   the repository's pinned Diesel and PostgreSQL, whether one attempt per claimed
   job is guaranteed by construction, and whether the target/attempt resolution
   plan genuinely avoids an N+1 path.
9. **The activation classification of `BE-04.md` section 9.1** — whether
   `Activated` versus `Repaired` is correctly decidable from the rows `enable_on`
   already reads, and whether "any member already enabled means repair" is sound
   for every linked-group state. Also whether the specification is now free of
   every claim that a repair implies delivery.
10. **The deduplication formula and its check constraint** — whether
    `distribution_job_deduplication_key_formula_check` is accepted by the target
    PostgreSQL version and whether every expression in it is genuinely immutable.
11. **The claim eligibility predicate of section 12.4** — specifically the
    `activation_id` match, which is what prevents a disable/re-enable cycle from
    producing two live jobs, whether requiring **all** targets to qualify has
    an unintended consequence, and whether the attempt-budget clause interacts
    correctly with T5b.
12. **The interaction between assignment disable and running jobs** (section
    14.3) — whether cancelling `PENDING` while leaving `RUNNING` is the right
    split, and whether the post-expiry unclaimable state is acceptable or should
    instead terminalize.
13. **The worker authorization matrix** (section 15.2), especially the deliberate
    `SUPERUSER` denial of the three worker operations — now stated as BE-04's own
    least-privilege choice rather than an `ADR-0008` requirement — and the
    `publisher_org_ids()` change.
14. **The `ADR-0008` consumption** (sections 6.3 and 6.4) — whether the
    specification consumes the decision exactly, without broadening the seven
    approved conventions, without presenting BE-04-specific mechanisms as approved
    cross-programme architecture, and without claiming any approval or
    authorization the ADR does not give.
15. **The transaction step placement** (section 10.1) — whether inserting job
    writes at 9a–9c, before the publisher `UPDATE`, is correct, whether step 9a′
    is placed correctly, and whether any ordering consequence was missed.
16. **The claimed non-goals** — whether the specification anywhere smuggles in a
    generic framework, a second configuration transaction, an observed-delivery
    concept, a fabricated job status, or scope belonging to MIG-01, APP-01,
    APP-02, DIS-01 or DIS-02.
