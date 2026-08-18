# BE-03-SPEC Implementation Report

## 1. Repository state

Repository: `thoth-pub/thoth`
Workflow: STANDARD
Base branch: `feature/publisher-services/be-02-closeout` (see section 1.1)
Base commit: `1f2cb585b25336ab9806adaeff9538b1ac3fa8ea`
PR target: `develop`
Programme integration branch: None
Task branch: `feature/publisher-services/be-03-spec`
Head commit: recorded in the pull request; this report is written at the branch
head that carries it
Pull request: live pull-request state, review state and CI evidence are
represented by the GitHub pull-request record. This committed report does not
duplicate transient PR lifecycle state (ADR-0005).
Expected branch deletion after merge: YES
Final programme PR required: NO
Implementing model: Claude Opus 5
Reasoning level: Extra High / xhigh

### 1.1 Branch sequencing

This specification branch was deliberately created from the final
`BE-02-CLOSEOUT-01` head rather than from `develop`, because the BE-03 tracker
state builds on the corrected BE-02 dependency state that the closeout
delivers. Basing on `develop` would have required either duplicating the
closeout's corrections or asserting a BE-03 dependency state that contradicts
the tracker.

The closeout has since merged into `develop` as
`7aeb715f9815a41b6357d8b6a7037ac62ebb25bb`, so the closeout content is now an
ancestor of `develop` and the merge base of this branch is unchanged at
`1f2cb585b25336ab9806adaeff9538b1ac3fa8ea`. The branch therefore diverges from
`develop` only by that merge commit, and a pull request against `develop` shows
only the bounded BE-03-SPEC diff. **No rebase, merge, squash, amend or
force-push was performed at any point in this branch's history**, and none is
required; the branch was deliberately not rebased merely because `develop`
advanced.

The branch history must not be squashed or rewritten.

### 1.1.1 Specification remediation passes

Three bounded remediation passes were applied, each as one ordinary commit on top
of the then-current head, with no earlier commit amended, rebased, squashed or
force-pushed:

1. a **control-side pre-review** of head
   `20aa3a9ac4887d1bb03112d078a96f608211bed1` found four material defects,
   resolved in section 5.1;
2. a **fresh independent specification review** of head
   `5dc2b1dd651176540aa2e49cba54c586e1f58782` returned `CHANGES REQUIRED` with
   one P1 and six P2 findings, resolved in section 5.2;
3. a **second fresh independent specification review** of head
   `a1acd3f60b5b3631c2b61de2df175b4994a292ab` independently confirmed all seven
   previous findings `RESOLVED` and re-confirmed the settled architecture, and
   returned `CHANGES REQUIRED` with one P1 and one P2, resolved in section 5.3.

All three remediations are documentation-only: no runtime code, migration,
`schema.rs`, generated SDL, client artifact, `Cargo` file or workflow is
touched, no implementation branch is created, and no authorization is granted or
implied.

### 1.2 Preflight

**Historical authoring-time record.** This is what was verified immediately
before the first edit of this task, and unchanged from the `BE-02-CLOSEOUT-01`
preflight recorded in that task's report. It is preserved as evidence of the
state the specification was authored against; it is **not** a claim about the
current repository, and ordinary lifecycle progression since — for example the
opening of this task's own specification pull request — does not falsify it:

```text
origin/develop                     = bcb6ce3081abb14467798b372fcc3e6af9da1c6a
PR #805                            = MERGED (merge commit = the SHA above)
BE-01 (PR #779)                    = MERGED
BE-02 (PR #805)                    = MERGED
THOTH-GQL-DATALOADER-01 (PR #802)  = MERGED
ADR-0001/0002/0003/0004/0005/0007  = present and merged on develop
Existing BE-03 spec branch/PR/file = none
Existing BE-03 implementation branch/PR = none
feature/publisher-services/be-03   = absent
Working tree                       = clean
```

Durable across the whole of this task: `feature/publisher-services/be-03` was
absent at authoring time, was never created, and must not exist until separate
explicit CTO implementation authorization from a freshly verified `develop`
head.

## 2. Scope confirmation

Approved specification: this is itself a specification task. Its authority is
the approved Publisher Services Technical Design and Implementation Plan, Drive
revision `3`, the merged ADRs, and the merged BE-01/BE-02 implementations.

Implemented objective: produce a complete, repository-authoritative
implementation specification candidate for BE-03, and surface the one programme
decision the approved sources leave contradictory.

Out-of-scope changes made: NONE

## 3. Commits

- `a3d4b1a77567273dc7289ecbbe59bf0601c03356` - docs(publisher-services):
  specify BE-03 protected service configuration
- `20aa3a9ac4887d1bb03112d078a96f608211bed1` - docs(publisher-services): record
  BE-03-SPEC evidence
- `5dc2b1dd651176540aa2e49cba54c586e1f58782` - docs(publisher-services):
  remediate BE-03-SPEC pre-review findings (the four control-side pre-review
  findings, across `BE-03.md`, this report, `decisions.md`, `task-status.md` and
  the existing `CHANGELOG.md` entry)
- `a1acd3f60b5b3631c2b61de2df175b4994a292ab` - docs(publisher-services):
  remediate BE-03-SPEC independent review findings (the one P1 and six P2
  findings of the first fresh independent specification review, across the same
  five files)
- one further ordinary commit remediates the one P1 and one P2 finding of the
  **second** fresh independent specification review, across `BE-03.md`, this
  report, `decisions.md` and the existing `CHANGELOG.md` entry. Its exact SHA is
  the branch head recorded in the pull request.

No earlier commit is amended, rebased, squashed or force-pushed at any point.

## 4. Files changed

- `docs/engineering/ai-delivery/tasks/BE-03.md` (new)
  - reason: the bounded written implementation specification BE-03 requires
    before it may be authorized. The first remediation pass corrected the
    section 11.1 authorization matrix and its prose, prescription and tests
    (finding 1); the mutation sequence, no-op definition, lifecycle composition,
    audit policy, concurrency requirements, acceptance criteria and
    linked-platform tests (finding 2); the APP-01 scope split in section 12.2
    (finding 3); and the source model, audit schema contract, write coordinator,
    mutation design, MIG-01 seam, acceptance criteria, stop conditions and
    evidence requirements (finding 4). The second remediation pass added the
    ADR-0001 effective-capability exposure (sections 1, 2.1 item 4, 3, 4, 5,
    7.1, 10, 10.1, 11.1, 12.2, 14, 15, 18.1, 18.2, 18.5, 19 and 20), the
    `publisher.updated_at`/`publisher_history` consequence (sections 6.4, 7.3
    step 8, 18.4 and 20), the connection-scoped assignability invariant
    (sections 7.7 and 18.6), the corrected list-argument nullability (sections
    12, 14.1, 14.3 and 18.1), the corrected bypass-search scope (sections 2.1
    item 7, 18.9 and 20) and the durable programme-decision authority condition
    (stop condition 5). The third remediation pass recorded the **second**
    `public.publisher` `UPDATE` trigger and its work-freshness cascade (sections
    2.1 item 8, 6.3, 6.4, 7.3 step 9, 7.8, 7.9, 17.1, 18.1, 18.4, 20 and new
    stop condition 19, with the former condition 19 renumbered 20).
  - behavioural effect: none. It authorizes nothing.
- `docs/engineering/ai-delivery/implementation-reports/BE-03-SPEC-implementation-report.md`
  (new)
  - reason: the required implementation report, including the remediation
    records in sections 5.1, 5.2 and 5.3.
  - behavioural effect: none.
- `docs/publisher-services/decisions.md`
  - reason: records the BE-03/BE-04/APP-01 phase-boundary programme-decision
    candidate as section 3a. The first remediation added the explicit APP-01
    reconciliation, the statement that the candidate refines the narrow earlier
    back-catalogue-status clause, and the coordinator/backfill consequences. The
    second replaced the self-blocking `PROPOSED - AWAITING CTO DECISION` status
    with the durable ADR-0005 authority condition, recorded that capability
    exposure is settled ADR-0001 architecture rather than part of this
    candidate, and extended the APP-01 configuration scope and the section 1
    package statement to include effective capability codes. The third added one
    factual note to section 3a recording that BE-04's extension of the same
    transaction inherits the publisher-triggered work freshness cascade; the
    ownership boundary, the APP-01 reconciliation and the authority condition are
    unchanged.
  - behavioural effect: none.
- `docs/publisher-services/task-status.md`
  - reason: the BE-03 row and next-action 10 point at the written specification
    and record `IMPLEMENTATION NOT AUTHORIZED` in durable ADR-0005 language. The
    first remediation additionally recorded the APP-01 scope boundary on the
    APP-01 row and in next actions 10 and 11. The second reconciled the BE-03
    row, the APP-01 row and next actions 10 and 11 with the decision's authority
    condition and with the capability scope, and replaced the BE-03 row's `TBD`
    pull-request cell with the durable specification-PR reference.
  - behavioural effect: none.
- `CHANGELOG.md`
  - reason: root `AGENTS.md` section 13. Each remediation amended the existing
    `BE-03-SPEC` entry only where it had become materially false or incomplete —
    first the audit actor column, the least-privilege matrix, the write
    coordinator, the reachable linked-state repair and the APP-01
    reconciliation; then the effective-capability exposure, the
    `publisher.updated_at` consequence and the decision's authority condition;
    then the second publisher trigger, the work-freshness cascade and its
    required measurement. No second entry was created.
  - behavioural effect: none. Under the existing `### Added` heading.

## 5. Implementation decisions

Design decisions settled in the specification, each derived from merged code
rather than from the design document alone:

1. **Authorization uses the existing model, at least privilege.**
   `PolicyContext::require_publisher_for` is
   `require_role_for_publisher(value, Role::PublisherUser)`, so for a
   non-superuser it requires **exactly** `PUBLISHER_USER` for the publisher's
   ZITADEL organisation; `require_superuser` expresses superuser-only. The
   specification's matrix states exactly that and nothing wider: superuser ALLOW
   for any publisher; `PUBLISHER_USER` scoped to the target publisher ALLOW;
   `PUBLISHER_USER` scoped only elsewhere DENY; `PUBLISHER_ADMIN`,
   `WORK_LIFECYCLE` or `CDN_WRITE` **without** `PUBLISHER_USER` DENY;
   authenticated with no applicable role DENY; anonymous DENY. Writes are
   superuser-only for every caller. No new ownership table, role, helper,
   framework or account table is introduced, and **no role inheritance is
   invented**. Because an account may hold scoped roles for several
   organisations, ownership is specified as a per-publisher role check rather
   than a single-publisher identity.
2. **Actor identity is an explicit `actor text NOT NULL` column whose provenance
   is fixed by `source`.** For `SUPERUSER_API` it is exactly
   `IntrospectedUser.user_id` from `PolicyContext::user_id()`. The column follows
   the repository's established actor representation — a free-text actor column
   with **no** foreign key to any local account, which is what
   `publisher_history.user_id` became when the ZITADEL migration removed the
   `account` table and its constraint — but it is deliberately **not** named
   `user_id`, because section 9.1 also admits a controlled non-ZITADEL migration
   control identity and a `user_id` name would assert something untrue. A
   `CHECK (btrim(actor) <> '')` constraint makes a blank actor impossible for any
   write path. `publisher_history` is not modified, and its legacy `timestamp
   without time zone` type is deliberately **not** copied; the new table uses
   `timestamptz`, as BE-02 does.
3. **The concurrency token is a dedicated column,
   `publisher.service_configuration_updated_at`.** Two alternatives were
   evaluated against the merged schema and rejected on evidence:
   - `publisher.updated_at` fails in **both** directions. The
     `diesel_manage_updated_at` trigger moves it on any publisher row update, so
     an unrelated metadata edit — one a publisher user may perform through
     `updatePublisher` — would invalidate a superuser's in-flight token; and an
     assignment-only configuration change writes only
     `publisher_distribution_platform`, so it would not move it at all;
   - a computed maximum across publisher and assignment timestamps requires
     reading several independently changing values after the lock check, yields
     a derived rather than stored token, cannot be compared in one atomic
     predicate, and does not move for a package-only change on a publisher whose
     assignments are newer.
   A dedicated `bigint` counter was also considered and rejected only because
   the approved design specifies a timestamp-shaped `updatedAt` /
   `expectedUpdatedAt` contract.
4. **Strict per-publisher monotonicity is required, not decorative.**
   `CURRENT_TIMESTAMP` is `transaction_timestamp()`, so a transaction that
   started earlier but blocked on the row lock commits later with an earlier
   value. Ordinary staleness detection survives that, because the check is
   equality against the stored token. The residual hazard is an ABA case in
   which a later write stores a value **equal to a token some client still
   holds**, letting that client's stale write wrongly succeed. The specification
   therefore requires
   `GREATEST(CURRENT_TIMESTAMP, previous + interval '1 microsecond')`, computed
   from the value already read under the lock.
5. **Application-level lock order is identical to BE-02 and cannot invert — but
   the transaction writes more than the rows BE-03 names.**
   `publisher_distribution_platform::crud::lock_publisher` is the **only**
   `FOR UPDATE` in `thoth-api/src`. BE-03 takes the same lock, on the same
   single row, as the first statement of its transaction, and the coordinator
   manages one publisher's configuration, so BE-03 introduces no second explicit
   lock target and its own statements cannot invert. That is a statement about
   BE-03's **explicit** ordering only. The publisher row `UPDATE` that writes the
   token fires the existing `AFTER UPDATE`
   `set_work_updated_at_with_relations` trigger, whose one set-based statement
   also writes every work of that publisher, so the committed transaction writes
   one publisher row **plus** bounded configuration/audit rows **plus** N related
   work rows. See decision 18.
6. **A real composition hazard was found and resolved additively.** BE-02's
   `enable` and `disable` take `&PgPool`, call `db.get()?`, and open their own
   transaction on their own pooled connection. Calling either from inside the
   BE-03 transaction would run on a *different* connection and block on the
   publisher row lock the BE-03 transaction itself holds — a self-block
   resolving only by timeout. This is **not** a lock-order conflict and is not a
   stop condition; the specification requires extracting connection-scoped
   `enable_on`/`disable_on` functions and reducing the existing public functions
   to delegation, so BE-02's semantics and tests are unchanged and BE-03 reuses
   the domain invariants rather than re-implementing them. The connection-scoped
   functions must additionally **return a two-state outcome**
   (`Unchanged`/`Changed`), because BE-02's `ThothResult<()>` return leaves a
   caller unable to distinguish a genuine no-op from a linked-state repair; the
   pool-level wrappers discard that outcome and keep their merged signature.
7. **The mutation must not gate lifecycle calls on a membership diff.** BE-02
   decides linked-group work with its private `is_normalized_fully_enabled`
   predicate, which requires both rows present, both enabled, `disabled_at IS
   NULL`, one shared `activation_id` and one identical `enabled_at`. Enabled
   membership therefore does not imply normalization: a persisted pair of
   `OAPEN` enabled with activation A at T1 and `DOAB` enabled with activation B
   at T2 already has membership `{OAPEN, DOAB}`, which any request naming either
   member normalizes to — an empty membership diff. A diff-gated mutation would
   never call BE-02's normalization and the split pair would survive, breaking the
   specification's own repair requirement. The specification therefore requires
   the coordinator to call the enable primitive **once per desired linked group,
   unconditionally**, and to let the primitive decide no-op versus repair. A true
   no-op requires all three of: unchanged package; equal normalized enabled
   membership; and every requested enabled group already fully normalized — with
   the third condition decided by the primitive's outcome, never re-derived by
   BE-03. A repair is a committed change: it bumps the token and writes exactly
   one audit row whose `before_state`/`after_state` differ only in
   `configurationVersion`, and activation identifiers are **not** added to the
   audit JSON to make it visible.
8. **One canonical write authority.** The claim that the token changes on every
   committed configuration change, and that configuration changes produce
   canonical audit history, is false unless exactly one production write path
   exists: BE-02's pool-level `enable`/`disable` can modify desired platform
   assignments without touching the token or the audit table. The specification
   therefore fixes an internal **service configuration write coordinator** owning
   all four of package, enabled-platform desired state, canonical version token
   and audit history; it runs one caller-owned transaction and takes an explicit
   source/actor context; and the GraphQL mutation must call it. BE-02's
   pool-level functions are retained **unchanged** and reclassified as
   lower-level domain/compatibility functions, prohibited from new production
   configuration call sites. This is currently costless and verifiable: a
   repository search shows both have **zero production call sites** today —
   referenced only from `graphql/distribution_platform_tests.rs` and the model's
   `#[cfg(all(test, feature = "backend"))]` tests — and BE-02 exposes neither
   through GraphQL. The boundary's limitation is recorded honestly: it is enforced
   by specification, review and the required call-site/bypass-search evidence, not
   by the type system.
9. **No second DataLoader.** BE-02's request-local
   `publisher_distribution_platforms` loader is keyed by `publisher_id` and
   already returns enabled assignments in canonical order — exactly the
   protected field's shape — so the specification requires reusing it, per
   ADR-0007 and the programme's reuse rule. The report's `lastChange` needs no
   loader because it is fetched eagerly for the page in one set-based
   `DISTINCT ON` statement.
10. **Change metadata lives only on the superuser report row type.** Keeping
    `PublisherServiceConfiguration` identical for owner and superuser callers
    avoids conditionally-visible fields and their failure modes, and keeps actor
    and source off the owner surface entirely. Audit
    `before_state`/`after_state` JSON is exposed to **nobody** through GraphQL in
    BE-03.
11. **`ThothPackage` becomes SDL-reachable for the first time.** It is currently
    absent from `thoth-client/assets/schema.graphql` because BE-01 added no
    GraphQL surface. The specification records this as an expected SDL diff item
    to be confirmed, not discovered, and notes that type reachability is not
    value exposure.
12. **Exactly one new error variant and one new `into_field_error` arm.**
    `into_field_error` currently gives a distinct type only to
    `InvalidSubjectCode` and `Unauthorised`. The stale case needs a distinct
    `STALE_SERVICE_CONFIGURATION` type so APP-01 can render "configuration
    changed; reload". Every other family keeps its current mapping;
    `DistributionPlatformNotAssignable` and `EntityNotFound` continuing to
    surface as `INTERNAL_ERROR` is recorded as a known limitation rather than
    silently normalized.
13. **Only a *true* no-op moves no token and writes no audit row.** A version that
    changed without a configuration change would force needless retries on every
    other client and fill the audit table with rows identical in every key. This
    also matches BE-02, whose genuinely same-state transitions already move no
    timestamp — and which equally treats a split linked pair as **not**
    same-state. A stale request still fails even when it would have been a true
    no-op, and even when it would have repaired a split pair, because the version
    check precedes validation and every lifecycle call. Per decision 7, a
    membership-equal repair is a committed change and therefore does bump the
    token and write one audit row.
14. **Not-found precedes the role check** for the single-publisher query, so an
    unknown ID returns `EntityNotFound` to any authenticated caller. This
    mirrors the existing `load_current` convention and discloses nothing new,
    because publisher existence is already public through the anonymous
    `publisher`/`publishers` queries. It is recorded as a deliberate choice.
15. **The protected surface exposes effective capability codes, because ADR-0001
    section 4.4 already requires it.** This is not a BE-03 design choice to
    justify; it is approved architecture to follow. The field is
    `effectiveCapabilities: [PublisherCapability!]!`, derived on read from the
    canonical `subscription_package` through BE-01's merged
    `ThothPackage::capabilities()`, ordered by that function's `&'static` slice,
    persisted nowhere, not settable as an input, not present in the audit JSON,
    and protected by the same single section 11.1 read decision as the rest of
    the type. `PublisherCapability` consequently becomes SDL-reachable alongside
    `ThothPackage`. The alternative — deferring capabilities to a later task —
    was rejected outright: it would require a new decision to justify not doing
    what an approved ADR already mandates, and BE-01 explicitly left the enum
    unreferenced *in anticipation of BE-03 using it*.
16. **Storing the token on `publisher` moves `publisher.updated_at` too, and
    that is accepted rather than worked around.** The `set_updated_at` trigger
    fires on any real publisher row update, so every committed configuration
    change — including a platform-only change and a linked-state repair — also
    moves the public `Publisher.updatedAt`, while a stale request and a true
    no-op move neither timestamp. Suppressing or special-casing the trigger was
    rejected as breaking a repository-wide convention for one column;
    `Publisher.updatedAt` is explicitly **not** the concurrency token, and the
    public timestamp continues to disclose the fact of a change without
    disclosing any package, capability, platform or commercial value. The
    related `publisher_history.data` additional-key consequence is recorded, and
    the coordinator deliberately does not route the package write through
    `Crud::update`, so BE-03's mutation writes no `publisher_history` row.
    Decision 18 records the **second** publisher trigger, which the earlier
    passes did not account for, and restates the token-location trade-off in its
    light.
17. **Fail-closed non-assignability is a property of the primitive, not of
    caller discipline.** The extraction required by decision 6 could have left
    the connection-scoped `enable_on` dependent on the coordinator having
    pre-validated its input. The specification instead requires `enable_on` to
    call `platform.is_assignable()` itself and fail before any write, keeping
    the pool-level wrapper's merged early check and the coordinator's whole-set
    pre-validation as well. Two checks of the same merged predicate returning
    the same merged error is defence in depth, not a second algorithm.
18. **`public.publisher` has a second `UPDATE` trigger, so a committed
    configuration change also refreshes the publisher's whole catalogue — and
    that is accepted, made explicit and made measurable rather than engineered
    around.** Verified in `thoth-api/migrations/20250000_v1.0.0/up.sql`:
    alongside `set_updated_at` (`BEFORE UPDATE`, `diesel_set_updated_at()`),
    `public.publisher` carries `set_work_updated_at_with_relations`
    (`AFTER UPDATE`, `publisher_work_updated_at_with_relations()`), which on
    `NEW IS DISTINCT FROM OLD` executes one set-based
    `UPDATE work SET updated_at_with_relations = current_timestamp FROM imprint
    WHERE work.imprint_id = imprint.imprint_id AND imprint.publisher_id =
    NEW.publisher_id`. A committed BE-03 configuration change therefore has
    three observable persisted consequences, not one, and
    `work.updated_at_with_relations` is a **public downstream freshness signal**
    — resolved as `Work.updatedAtWithRelations`, a filter argument on six
    anonymous queries, an orderable field, and the value
    `thoth-export-server`'s `MetadataRecord` compares against its Redis cache
    timestamp through the generated `WorkLastUpdatedQuery` /
    `WorksLastUpdatedQuery`.

    Two consequences are specific to BE-03 and are stated rather than glossed:
    a **platform-only** change and an **OAPEN/DOAB linked-state repair** will
    **newly** trigger this cascade, because BE-03 bumps the canonical token on
    the publisher row where merged BE-02 only takes the publisher lock and writes
    `publisher_distribution_platform` (which has no work-freshness trigger); and
    the transaction's write footprint is one publisher row plus bounded
    configuration/audit rows plus N related work rows, so row-lock footprint and
    transaction duration grow with catalogue size while the publisher
    `FOR UPDATE` lock is held. BE-04 inherits all of it with the transaction
    boundary.

    **The token stays on `publisher`.** Moving it to a separate
    service-configuration table would genuinely avoid the cascade for
    platform-only and version-only persistence — that benefit is real and is now
    stated honestly in `BE-03.md` section 6.3 rather than dismissed — but it
    would **not** avoid it for canonical package changes, because BE-01 already
    made `publisher.subscription_package` canonical and a package change must
    `UPDATE publisher` wherever the token lives. Removing the cascade entirely
    would require moving `subscription_package` off `publisher`, redefining the
    shared publisher trigger, changing which publisher fields participate in work
    freshness, or suppressing trigger behaviour — all wider changes to approved
    BE-01 and shared publisher semantics that this review finding does not
    justify. The bounded decision is therefore to **accept, document, test and
    measure** the cascade: section 18.4 requires all six cases against a real
    disposable PostgreSQL database with at least two imprints, at least two
    target works and a control work of another publisher, plus a catalogue-scale
    write-amplification and lock-footprint measurement, and section 20 puts that
    evidence in front of the independent HIGH-risk implementation reviewer. This
    is **not** a claim that the cost is acceptable at any scale: new stop
    condition 19 requires the implementing agent to return `BLOCKED` and escalate
    a separate architecture decision if the evidence is bad, and forbids it from
    silently changing shared publisher trigger or storage semantics instead.

Deviations from an approved source:

- The approved design's API section implies that the replace mutation creates
  the required jobs and that the staff report includes job state. That
  contradicts the approved task decomposition, which places job persistence in
  BE-04, and the rollout, which holds automatic job creation inactive. Rather
  than guess, the contradiction is surfaced as a `PROPOSED` programme decision
  in `docs/publisher-services/decisions.md` section 3a and recorded as a BE-03
  stop condition if the CTO decides otherwise.
- The same approved design makes `APP-01` depend on BE-03 **and** lets superusers
  inspect back-catalogue status. Those cannot both hold of a BE-03-only
  dependency once job state is correctly deferred to BE-04. The programme
  decision candidate therefore explicitly **refines and, in that narrow respect,
  supersedes** the clause assigning back-catalogue-status inspection to a
  BE-03-only dependency, and splits APP-01 into a BE-03-satisfiable
  configuration scope and BE-04-dependent job-aware elements. The rest of the
  approved APP-01 record is unchanged, the refinement is recorded as a clearly
  marked `PROPOSED` candidate rather than applied silently, and no agent marks it
  approved.

### 5.1 Control-side pre-review remediation

The pre-review of head `20aa3a9a` found four material defects. All four are
remediated in this branch; none required a `BLOCKED` return, and the evidence for
each is recorded below.

**Finding 1 — protected-read authorization contradiction. RESOLVED, least
privilege.** Section 11.1 previously allowed "`PUBLISHER_USER`, `PUBLISHER_ADMIN`,
or any publisher-scoped role satisfying the check", which contradicted the
prescribed `PolicyContext::require_publisher_for(&publisher)` call. Verified
against `thoth-api/src/policy.rs`: `require_publisher_for` is
`require_role_for_publisher(value, Role::PublisherUser)`, and
`require_role_for_publisher` short-circuits for superusers and otherwise tests
`has_role_for_org` for **exactly** the one role passed. There is no inheritance
anywhere in the file. Corroborating evidence that `PUBLISHER_ADMIN` is not
independently privileged for publisher-scoped reads:
`PublisherPermissions::permissions_for_org` exposes `publisher_admin`,
`work_lifecycle` and `cdn_write` as independent booleans and omits
`PublisherUser` entirely; `require_publisher_admin_for` is used only by
`imprint/policy.rs`; and the merged `PublisherPolicy::can_update` gates publisher
metadata edits on `require_publisher_for`, i.e. on `PUBLISHER_USER`, so an
account that administers publisher metadata already holds `PUBLISHER_USER`.
`docs/publisher-services/decisions.md` section 1 likewise says "Publisher users
may read their own package". **No repository authority makes `PUBLISHER_ADMIN`
independently sufficient**, so the least-privilege reading was adopted rather
than returning `BLOCKED`, and it costs no genuine access. The matrix, prose,
implementation prescription, required tests, non-goals, acceptance criteria and
stop conditions now state one identical rule, and a new stop condition 15
routes any future widening to an explicit CTO authorization decision.

**Finding 2 — OAPEN/DOAB split-state repair unreachable. RESOLVED.** The previous
algorithm diffed desired membership against current enabled membership and applied
only the difference, so a split-but-both-enabled pair produced an empty diff and
BE-02's normalization was never called. The corrected algorithm calls the
connection-scoped enable primitive **once per desired linked group,
unconditionally**, and the primitive's existing `is_normalized_fully_enabled`
predicate decides no-op versus repair; BE-03 must not re-derive it, so there
remains exactly one linked-platform algorithm, in BE-02's module. The primitives
must now return `Unchanged`/`Changed` so the coordinator can tell a no-op from a
repair. The true no-op is redefined as the conjunction of unchanged package, equal
normalized enabled membership, and every requested enabled group already fully
normalized. A repair is a committed change: token bump, exactly one audit row,
same transaction, `before_state`/`after_state` differing only in
`configurationVersion`, and no activation identifier added to the audit JSON.
Updated: mutation sequence (7.3), no-op definition (7.4), lifecycle composition
(7.7), audit policy (8.3), concurrency requirements (18.3), acceptance criteria
(18.1) and linked-platform tests (18.5).

**Finding 3 — BE-03/BE-04 decision did not fully reconcile APP-01. RESOLVED.**
The architectural direction is unchanged: BE-03 owns desired configuration; BE-04
owns `distribution_job`, `distribution_job_target`, `distribution_job_attempt`,
job-creation rules, the worker role, leases, claims, the
completion/failure/retry/cancellation lifecycle and durable back-catalogue job
state; BE-03 creates no pseudo job and no fabricated status. The decision
candidate adds an explicit APP-01 reconciliation naming the BE-03-satisfiable
capabilities and the four BE-04-dependent ones, states that it refines and in
that narrow respect supersedes the earlier back-catalogue-status clause, keeps
`APP-02` dependent on BE-03 **and** BE-04, and remains a candidate that no agent
may declare approved — its authority condition is stated in section 5.2. BE-03
section 12.2 mirrors it, and the tracker's APP-01 row and next actions record
the scope boundary. This is programme-local: one
programme, one repository, no shared component, no cross-programme abstraction
and no ADR — so no `BLOCKED` return was required.

**Finding 4 — canonical token/audit semantics bypassable. RESOLVED.** Section 7.6
now fixes one authoritative internal service-configuration write coordinator
owning package, enabled-platform desired state, canonical version token and audit
history; it executes one caller-owned transaction, takes an explicit source/actor
context, and is the mutation's only write path. BE-02's pool-level
`enable`/`disable` are retained with their merged behaviour and tests intact but
reclassified as lower-level domain/compatibility functions barred from new
production configuration call sites, with required call-site enumeration and
bypass-search evidence (18.9). Verified enabling fact: both have **zero
production call sites** today. BE-02's public GraphQL read contract is unchanged.
For the MIG-01 seam, the source enum keeps `SUPERUSER_API` and
`MIGRATION_BACKFILL`, and the audit actor column is redefined as
`actor text NOT NULL` with a non-blank check constraint and a source-scoped
provenance contract, so both an authenticated superuser id and a controlled,
non-secret migration control identity satisfy it without inventing credentials.
Section 9.3 binds MIG-01 to the same coordinator (or an explicitly controlled
internal mode with identical persistence invariants), the same token update, the
same audit row, `source = MIGRATION_BACKFILL`, no job, no dissemination and
idempotence. The specification explicitly does **not** claim that a
`MIGRATION_BACKFILL` actor is an `IntrospectedUser.user_id`.

*Rejected alternative for Finding 4's MIG-01 seam:* shipping `SUPERUSER_API`
alone and letting MIG-01 add its own source value later under a separately
reviewed additive migration. Rejected because the repository already supports a
coherent non-user actor contract — the actor representation is a free-text column
with no account foreign key, and no `account` table exists — and because BE-03
owns the new table and can therefore define the column name and contract
coherently in one place instead of inheriting the misleading `user_id` name. With
`source` fixing the namespace and the check constraint forbidding a blank actor,
the reserved value's semantics are satisfiable rather than incoherent, and
deferral would cost an extra `ALTER TYPE ... ADD VALUE` migration plus a second
additive SDL enum change for no safety gain, since BE-03 writes only
`SUPERUSER_API` either way.

### 5.2 Independent specification review remediation

The fresh independent specification review of head `5dc2b1dd` returned `CHANGES
REQUIRED` with one P1 and six P2 findings and no `BLOCKED`-level finding. It
explicitly did **not** reject the authorization model, the coordinator, the
transaction composition, the OAPEN/DOAB normalization, the concurrency design,
the audit model or the BE-03/BE-04 programme boundary; those areas are unchanged
except where consistency with the seven fixes required it. All seven are
remediated in one ordinary commit; none required a `BLOCKED` return.

**P1 — ADR-0001 capability exposure omitted. RESOLVED by following ADR-0001, not
by deferral.** The reviewer verified an authority conflict:
[ADR-0001](../../decisions/ADR-0001-publisher-package-capability-model.md)
section 4.4 requires protected publisher service configuration to expose the
current package, the **effective capability codes** and the enabled distribution
platforms; ADR-0001 section 7 names BE-03 among the affected tasks;
`docs/publisher-services/README.md` records that protected package and
effective-capability reads remain BE-03 scope; and [BE-01](../tasks/BE-01.md)
excludes `Publisher.capabilities` from its own scope while anticipating that
BE-03 is the task which makes the enum SDL-reachable. The previous BE-03-SPEC
exposed package, platforms and `updatedAt` only, with a closed SDL inventory and
stop conditions that would have prevented an implementer adding capabilities.

The correction uses the **merged, code-owned** capability model with no new
architecture. Verified directly in
`thoth-api/src/model/publisher/mod.rs`: `PublisherCapability` already exists with
the six ADR-0001 values and already derives `juniper::GraphQLEnum` under the
`backend` feature; `ThothPackage::capabilities(self) -> &'static
[PublisherCapability]` is an exhaustive `match` returning ordered `&'static`
constants; `has_capability` delegates to it; and no capability column, table or
override exists in `schema.rs` or in `thoth-api/migrations`. `grep -n 'capabilit'
thoth-client/assets/schema.graphql` returns nothing, confirming the enum is not
yet SDL-reachable — exactly the outcome BE-01 recorded as acceptable until BE-03
uses it.

- exact protected field: `effectiveCapabilities: [PublisherCapability!]!` on
  `PublisherServiceConfiguration` (`BE-03.md` sections 10 and 10.1);
- value: exactly `ThothPackage::capabilities()` for the publisher's canonical
  `subscription_package`, with the slice's own order preserved, `OASIS`
  returning an empty list;
- capabilities are **derived on read and persisted nowhere** — no column, table,
  override, cache or second mapping — so no package/capability inconsistency is
  representable and no backfill or reconciliation exists;
- capabilities are **not** an input to the mutation and are **not** added to the
  audit JSON, whose canonical key set stays at exactly three keys because
  capabilities are a pure function of the already-recorded
  `subscriptionPackage`;
- authorization is the **single** section 11.1 decision taken once for the whole
  type: owner `PUBLISHER_USER` or superuser, with no field-level exception and
  no separate capability query. `PublisherCapability` must **not** appear on the
  public `Publisher` type or on any anonymous surface;
- SDL implications: `PublisherCapability` joins `ThothPackage` as an enum
  **becoming SDL-reachable**, so the expected diff now contains both new enum
  blocks with BE-01's existing value descriptions (`BE-03.md` sections 14.1 and
  14.2). Type reachability remains distinct from value exposure;
- tests added to the specification: every `ThothPackage` returns exactly
  `ThothPackage::capabilities()`; deterministic ordering asserted as an exact
  sequence rather than set equality; owner `PUBLISHER_USER` may read its own
  capabilities; a superuser may read any publisher's; another publisher is
  denied; anonymous is denied; unrelated `PUBLISHER_ADMIN`/`WORK_LIFECYCLE`/
  `CDN_WRITE` without `PUBLISHER_USER` are denied; no separate persisted
  capability state exists (catalog assertion); a package change changes the
  derived capabilities automatically, asserted for one upgrade and one
  downgrade; and no package/capability inconsistency is representable
  (`BE-03.md` sections 18.2 and 18.5).

This **reconciles BE-03 with ADR-0001 and BE-01. It is not a new architectural
decision**, and no programme decision was created to defer already-approved
scope. No ADR is edited, proposed or superseded.

**P2 — `service_configuration_updated_at` also moves `Publisher.updatedAt`.
RESOLVED as an explicit accepted consequence.** New `BE-03.md` section 6.4
records that, because the token is a column on `publisher` under the existing
`diesel_manage_updated_at` trigger, any real publisher `UPDATE` that writes the
token also moves `publisher.updated_at`; that this is unavoidable given the
section 6.1 decision and is accepted deliberately; that `Publisher.updatedAt` is
**not** the service-configuration concurrency token and may never be submitted
as `expectedUpdatedAt`; that it may move for package-only, platform-only and
linked-state-repair changes even where no ordinary publisher metadata changed;
that a stale request and a true semantic no-op perform no publisher `UPDATE` and
therefore move neither timestamp; that the public timestamp reveals **that** the
record changed while continuing to expose no package, capability, platform or
commercial value; and that this is a documented compatibility and
observable-semantic consequence rather than a defect. Non-goal 8 was reassessed
and states a value-exposure prohibition only, so no non-goal now implies zero
observable timestamp movement.

The same section records the distinct `publisher_history` consequence: because
`service_configuration_updated_at` is appended to the `Publisher` struct that
the shared `Crud::update` macro serializes into `publisher_history.data jsonb`,
the field **may appear as an additional key** in future publisher-history
snapshots wherever that path runs. The section states explicitly that the
generic `publisher_history` table and the section 8 configuration audit are
**not** the same thing, and section 7.3 step 8 now requires the coordinator to
write `subscription_package` directly rather than through `Crud::update`, so the
mutation writes no `publisher_history` row. Section 18.4 adds the required
six-case timestamp-movement table — package-only, platform-only and repair each
move both timestamps; true no-op, stale request and rollback move neither — plus
a test that a committed change writes no `publisher_history` row, and section 20
requires the report to record the additional-key consequence explicitly.

**P2 — `enable_on` must itself reject non-assignable platforms. RESOLVED.**
`BE-03.md` section 7.7 item 2 now requires the connection-scoped enable
primitive to execute `platform.is_assignable()` itself and fail with
`ThothError::DistributionPlatformNotAssignable` **before any write**,
independently of its caller, while the pool-level wrapper keeps its merged
early check before acquiring a connection (verified in
`thoth-api/src/model/publisher_distribution_platform/crud.rs`, where the check
precedes `db.get()?`). The coordinator must still pre-validate the whole
requested normalized set (section 7.3 step 7). This is stated as intentional
defence in depth preserving BE-02 fail-closed semantics for any internal caller,
and reconciled with the one-algorithm rule: both call the **same** merged
predicate and return the **same** existing error, which is not a second
algorithm. Section 18.6 adds the direct regression — call `enable_on(JISC_NBK)`
inside a caller-owned transaction and assert the error, no row created, no
existing row changed, no hidden mutation whether the transaction is rolled back
or committed, and token and audit unaffected because the coordinator's
committed-change phase is never reached — and keeps the coordinator's own
pre-validation regression separate so neither masks the other. Section 18.1 and
stop condition 18 were updated consistently.

**P2 — candidate SDL list nullability differed from repository convention.
RESOLVED.** Verified in the merged artifacts: `thoth-api/src/graphql/query.rs`
declares equivalent list filters as `Option<Vec<T>>` with `#[graphql(default =
vec![], …)]`, and `thoth-client/assets/schema.graphql` renders them as a
**nullable outer list of non-null members** — `publishers: [Uuid!] = []` on
`imprints`, `publishers` and `workCount`, and `workTypes: [WorkType!] = []` /
`workStatuses: [WorkStatus!] = []` on `works`. The candidate SDL in section 12
and the exact additive inventory in section 14.1 now read `publishers: [Uuid!] =
[]`, `packages: [ThothPackage!] = []` and `enabledPlatforms:
[DistributionPlatform!] = []`, with an explicit prohibition on introducing a
stricter non-null-list convention for BE-03. Section 14.3 item 2 requires the
regenerated SDL lines to be quoted verbatim and compared explicitly against
`imprints`, `publishersByDistributionPlatform` and `works`, and requires the
exact diff to match the corrected candidate. Section 14.3 item 7 now states that
the APP-01 contract pinning covers the exact generated schema at the reviewed
head, so any recorded schema SHA reflects the corrected contract.

**P2 — write-bypass search scope was wrong and incomplete. RESOLVED.**
`thoth-app` is a separate repository: the root `Cargo.toml` `[workspace]
members` list is exactly `thoth-api`, `thoth-api-server`, `thoth-client`,
`thoth-errors` and `thoth-export-server`, and no `thoth-app` directory exists
here, so a search of `thoth-app/src` matches nothing and proves nothing.
Sections 2.1 item 7, 18.9 and 20 now require the search to cover
`thoth-api/src`, `thoth-api-server/src`, `thoth-client/src`,
`thoth-errors/src`, `thoth-export-server/src`, the root binary crate's `src`,
and `thoth-api/migrations` — the actual location of this repository's
migrations; there is no top-level `migrations` directory — plus a check of the
workspace declaration to confirm no local production crate was omitted. The
evidence must record exact commands, the exact repository paths searched, the
complete relevant matches, a classification of every production call site, and
explicit confirmation that the lower-level BE-02 enable/disable functions are
not used as production service-configuration writers. Because the
single-write-coordinator invariant is not type-enforced, section 18.9 now states
that this evidence is a **mandatory acceptance condition**, with stop condition
9 applying if it cannot be made clean. Section 10 of this report is corrected in
the same way: the earlier `thoth-app/src` mention is replaced by the crates this
repository actually contains.

**P2 — committed report contained transient false state. RESOLVED.** The
statements "Pull request: intentionally deferred", "The BE-03-SPEC pull request
is **not opened yet**" and "CI status: PENDING; no pull request is open yet"
were falsified by ordinary lifecycle progression. They are removed and replaced
with durable ADR-0005 wording: live pull-request state, review state and CI
evidence are represented by the GitHub pull-request record, and this committed
report does not duplicate transient PR lifecycle state. A complete
transient-state sweep of this report, `BE-03.md`, `decisions.md`,
`task-status.md` and the `CHANGELOG.md` entry found and removed the further
instances recorded in section 9, and the previously empty "Observed results
feeding the specification" heading in section 10 is filled with the durable
observations it was intended to carry. Durable facts are retained: branch name,
task, scope, validation performed before publication, that no workflow was
dispatched manually, and that runtime implementation is unauthorized. Historical
statements are preserved only where explicitly framed as authoring-time
evidence. No status-only follow-up requirement was created.

**P2 — programme decision status / stop condition was self-blocking. RESOLVED
with a durable authority condition.** `decisions.md` section 3a previously read
`PROPOSED - AWAITING CTO DECISION` and said specification approval settles it,
while `BE-03.md` stop condition 5 blocked implementation unless the decision was
literally `APPROVED` at implementation time — guaranteeing a false block after
approval unless a further repository edit changed the word, which is exactly the
approval-state-only churn ADR-0005 section 4.1 item 10 prohibits. Section 3a now
reads:

```text
Decision state: PROPOSED IN THIS SPECIFICATION CANDIDATE

Authority condition. This decision becomes approved and repository-authoritative
when both of the following hold:
1. the exact BE-03-SPEC content containing this decision receives explicit CTO
   specification approval; and
2. that exact approved content is reachable from develop.

Before both conditions hold: NOT AUTHORITATIVE FOR IMPLEMENTATION.
After both hold: APPROVED PROGRAMME DECISION for BE-03 implementation purposes,
without requiring a separate lifecycle-status edit.
```

and stop condition 5 now tests that same authority condition, blocking if the
exact BE-03-SPEC content carrying the decision has not received CTO
specification approval and become reachable from `develop`, with an explicit
statement that a mutable literal `APPROVED` status word is **not** required.
GitHub remains terminal evidence for the exact-head approval and merge
lifecycle. `decisions.md` and `BE-03.md` state one identical rule, `task-status.md`
mirrors it, and the accompanying transient-state sweep found no other text that
would create a second recursive approval-state update.

### 5.3 Second independent specification review remediation

The second fresh independent specification review, of head `a1acd3f6`,
independently confirmed **all seven** findings of the first review as `RESOLVED`
and independently re-confirmed that the protected effective capabilities, the
`PUBLISHER_USER`/`SUPERUSER` authorization, the canonical write coordinator, the
connection-scoped BE-02 lifecycle composition, the OAPEN/DOAB normalization and
repair reachability, the true-no-op semantics, the optimistic concurrency, the
audit/source model, the MIG-01 seam, the BE-03/BE-04 ownership boundary, the
APP-01/APP-02 reconciliation, the migration shape, the GraphQL protected
exposure, the DataLoader approach and the rollout/rollback remain sound. None of
those areas is reopened here. It returned `CHANGES REQUIRED` with one P1 and one
P2, both remediated in one ordinary commit; neither required a `BLOCKED` return.

**P1 — a second existing `publisher` trigger was unaccounted for. RESOLVED by
accepting, documenting, testing and measuring the cascade, not by moving it.**
The reviewer found that `public.publisher` carries two material `UPDATE`
triggers, not one. Re-verified directly in
`thoth-api/migrations/20250000_v1.0.0/up.sql` before any edit:

```sql
CREATE TRIGGER set_updated_at BEFORE UPDATE ON public.publisher
    FOR EACH ROW EXECUTE FUNCTION public.diesel_set_updated_at();

CREATE TRIGGER set_work_updated_at_with_relations AFTER UPDATE ON public.publisher
    FOR EACH ROW EXECUTE FUNCTION public.publisher_work_updated_at_with_relations();
```

and `publisher_work_updated_at_with_relations()` is, on
`NEW IS DISTINCT FROM OLD`, exactly one set-based statement:

```sql
UPDATE work
SET updated_at_with_relations = current_timestamp
FROM imprint
WHERE work.imprint_id = imprint.imprint_id
  AND imprint.publisher_id = NEW.publisher_id;
```

Corroborating repository facts verified for the observability claim:
`Work.updatedAtWithRelations` is resolved on the public `Work` type
(`graphql/model.rs`); `updated_at_with_relations: Option<TimeExpression>` is an
argument on `works`, `work_count`, `books`, `book_count`, `chapters` and
`chapter_count` in `graphql/query.rs`, rendered in
`thoth-client/assets/schema.graphql`; `WorkField::UpdatedAtWithRelations` is
orderable (`model/work/crud.rs`); `thoth-client/assets/queries.graphql` defines
`WorkLastUpdatedQuery` and `WorksLastUpdatedQuery` selecting it; and
`thoth-export-server/src/specification/handler.rs` passes those values into
`MetadataRecord`, whose `load_or_generate` serves the Redis-cached record only
while `cached_timestamp >= self.last_updated`. `publisher_distribution_platform`
carries only `set_updated_at`, so merged BE-02 platform changes refresh no work
rows today.

Remediation, all in `BE-03.md` unless stated:

- **section 2.1 item 8** now records **both** triggers with their exact
  declarations and the trigger function's exact statement, plus the
  public/downstream consumption of `work.updated_at_with_relations` and the
  contrast with `publisher_distribution_platform`. Publisher is no longer
  described as having one material `UPDATE` trigger;
- **section 6.4** is rewritten from "`publisher.updated_at` also moves" to the
  three-consequence form: the token moves, `publisher.updated_at` moves, and
  `work.updated_at_with_relations` moves for every work of that publisher through
  its imprints. It states explicitly that a **platform-only** change and an
  **OAPEN/DOAB linked-state repair** will **newly** cause the cascade relative to
  merged BE-02, that works of other publishers are never touched, that stale,
  true-no-op and rolled-back cases fire neither trigger, and that the public
  signal reveals that related catalogue state was refreshed while not directly
  exposing package, capabilities, enabled platforms, actor, source or audit
  content. It no longer says the effect occurs only "at the granularity it
  already had", and it **does not overclaim confidentiality**: timing correlation
  is expressly acknowledged as still possible. The downstream export-cache
  invalidation effect is stated;
- **section 18.4** extends the timestamp table to a **three-value, six-case**
  table including target `work.updated_at_with_relations`, requires all six cases
  against a **real disposable PostgreSQL database**, fixes the fixture (one
  target publisher, at least two imprints, at least two works across them, at
  least one control work of another publisher), requires the control work
  asserted unchanged in every case, and adds a separate **catalogue-scale
  measurement** — target work count, statements attributable to the operation,
  work rows changed by the trigger, disposable-environment transaction duration,
  unrelated-publisher works unchanged, and confirmation that no per-work
  application loop exists — expressly as write-amplification and lock-footprint
  evidence and **not** a production SLA or extrapolation;
- **sections 7.3 step 9, 7.8 and this report's decision 5** are corrected. The
  claims that the mutation "touches exactly one publisher", that "there is no
  multi-object ordering" and that the work is "bounded by the closed 17-value
  inventory" now apply only to BE-03's **explicit application-level** lock order
  and statement count. Section 7.8 is retitled "Lock ordering and write
  footprint" and states the real footprint as one publisher row + bounded
  configuration/audit rows + N related work rows, that row-lock footprint and
  transaction duration grow with catalogue size, that the publisher `FOR UPDATE`
  lock is held while the trigger's work executes, and that the trigger issues a
  **set-based** `UPDATE` rather than a per-work application loop. No production
  duration and no safe catalogue-size threshold is invented;
- **section 6.3** re-weighs the separate-table alternative honestly. Its genuine
  benefit — avoiding both triggers for platform-only/version-only persistence —
  is now stated, and the earlier "larger without being more robust" framing is
  recorded as incomplete. It remains rejected because it would not remove the
  same cascade for canonical package changes while `subscription_package` stays
  on `publisher` under approved BE-01 architecture, because eliminating the
  cascade fully would require wider changes to BE-01 storage or shared publisher
  trigger semantics that this finding does not justify, because one publisher row
  remains the natural serialization boundary, and because the consequence is
  measurable and reviewable. The rejection is recorded as **evidence-conditional,
  not permanent**;
- **section 7.9** states that BE-04, extending the same coordinator transaction,
  **inherits** the cascade, its row-lock footprint, its duration behaviour and its
  downstream freshness effect unless a separately approved architecture change
  alters that. BE-04 is not solved here. `decisions.md` section 3a carries the
  same factual note without changing the ownership boundary;
- **section 17.1** no longer permits a reading of "no observable downstream data
  change". After separately authorized deployment and migration, a configuration
  change still creates no distribution job, performs no upload and triggers no
  dissemination worker — **but** the existing trigger may refresh
  `work.updated_at_with_relations` across that publisher's catalogue. That
  cascade is expressly **not** distribution activation;
- **new stop condition 19** requires the implementing agent to return `BLOCKED`
  with the measurements attached and escalate a separate architecture decision if
  the evidence shows a material operational problem, and forbids resolving it by
  dropping, disabling, rewriting or conditionalizing the publisher triggers, by
  moving `subscription_package` off `publisher`, or by otherwise altering shared
  publisher trigger or storage semantics. The former condition 19 is renumbered
  20;
- **sections 18.1 and 20** add the corresponding acceptance criteria and
  implementation-report requirements, so the evidence reaches the independent
  HIGH-risk implementation reviewer rather than staying in a test file.

The canonical token design is deliberately **unchanged**: it remains
`publisher.service_configuration_updated_at`, and no second
service-configuration or version table is introduced by this remediation.

**P2 — inaccurate write-bypass search count. RESOLVED with a reproducible
command.** The reviewer confirmed the conclusion — **zero** production call sites
of BE-02's pool-level enable/disable — but found the numeric claim "the only
references are the 49 in ..." inaccurate. Re-run at this branch's head, the
figures are 14 in `graphql/distribution_platform_tests.rs` and 44 in
`model/publisher_distribution_platform/tests.rs`, i.e. 58 in total. Section 10 of
this report now quotes the exact command, its exact per-file output and the total,
records **zero** non-test production hits, states that all matches are confined to
those two named test files, and records that both files are `#[cfg(test)]`-gated.
The approximate count is removed rather than adjusted. Because this is
specification-authoring evidence rather than future implementation evidence, it is
stated so that a reviewer can reproduce it in one command. It is explicitly **not
a waiver**: section 18.9 still requires the future implementation to rerun the
complete search against its own implementation head.

## 6. Database and migration effects

Migration added: NO

This task creates no migration. It **specifies** the additive migration BE-03
would require — one `publisher` column, one closed two-value source enum, and one
audit table with an `actor text NOT NULL` column, a named non-blank actor check
constraint, a primary key, an `ON DELETE CASCADE` foreign key and one composite
index — and explicitly excludes `distribution_job`, `distribution_job_target`,
`distribution_job_attempt`, worker-role persistence and any credential or
configuration-secret table.

## 7. API and compatibility effects

GraphQL/API changes: NONE in this task. The specification defines the exact
additive inventory BE-03 would add and requires the SDL diff to equal it. That
inventory now includes `PublisherCapability` alongside `ThothPackage` as an enum
becoming SDL-reachable, and fixes the three new list arguments at the merged
repository shape `[Uuid!] = []`, `[ThothPackage!] = []` and
`[DistributionPlatform!] = []`.
Generated schema/client updates: NONE. `thoth-client/assets/schema.graphql` is
untouched.
Backwards compatibility: unaffected by this diff. The specification additionally
records, for the future implementation, three observable consequences documented
rather than discovered later: `Publisher.updatedAt` will move on every committed
configuration change; `work.updated_at_with_relations` will move for every work
of that publisher through its imprints, which is a public downstream freshness
signal that invalidates that publisher's cached export records and changes what
incremental consumers select; and `service_configuration_updated_at` may appear
as an additional key in future `publisher_history.data` snapshots. None of them
is a distribution job, upload or dissemination.
Deprecations: NONE
Cross-repository dependencies: the reserved BE-03/APP-01 exact-SHA schema
pinning control is preserved and reinforced; the specification requires BE-03's
implementation report to record the exact backend head SHA for APP-01 pinning,
and states that the pinned contract is the schema generated at that head,
including the corrected argument nullability and the `PublisherCapability`
block. `thoth-app` is a separate repository and is not part of
`thoth-pub/thoth`; it and `thoth-dissemination` are untouched and are assessed
only through the generated GraphQL contract.

## 8. Authorization and security

Authorization paths changed: NONE. No authorization code exists in this diff.
Roles/scopes involved: none changed, and none widened. The specification consumes
the existing `Superuser` and `PublisherUser` ZITADEL roles only, and explicitly
does **not** treat `PublisherAdmin`, `WorkLifecycle` or `CdnWrite` as implying
`PublisherUser`.
Negative authorization tests: not applicable to a documentation change; the
specification **requires** the full negative matrix in section 18.2, including
anonymous, no-role, other-publisher,
`PublisherAdmin`/`WorkLifecycle`/`CdnWrite`-without-`PublisherUser`-cannot-read,
`PublisherUser`- and `PublisherAdmin`-cannot-mutate, and `zitadel_id IS NULL`
cases. Every read case must now be asserted for the configuration **including
its `effectiveCapabilities` field**, so commercial entitlement visibility is
proven to follow the same single decision as package visibility and is not
widened by any unrelated scoped role.
Secret or personal-data handling: none. No credential, token, endpoint, bucket
or account identity appears in the diff. The specification forbids them in the
audit JSON and in the `source`/`actor` values, and requires a key-set assertion
test. The audit actor is an identity **name** for an accountable control context,
never a means of authenticating one, and the specification requires no credential
or operational secret for either source value.
Security limitations: the pre-existing `ThothError::DatabaseError` rendering of
driver text is recorded honestly as pre-existing, not claimed absent. The
specification also declines to overclaim confidentiality for the
publisher-triggered work freshness cascade: `work.updated_at_with_relations` does
not directly expose package, capabilities, enabled platforms, actor, source or
audit content, but an observer polling it — or watching export cache
regeneration — can infer that something changed and when, so timing correlation
remains possible and is stated rather than denied (`BE-03.md` section 6.4).

## 9. Tests and checks

### Formatting

Command:

```text
git diff --check
```

Result:

```text
no output; exit status 0
```

### Unit tests

Not run, and not required. Documentation-only change; root `AGENTS.md` section
8 prescribes `git diff --check` plus documentation verification for a
documentation-only change and reserves the full workspace gate for Rust/domain
changes. No file under any workspace member is modified.

### Integration/database tests

Not applicable. No migration, schema or database-backed code is touched.

### Lint/static analysis

Not applicable to a Markdown-only diff.

### Other required checks

Path containment, re-run after each remediation commit:

```text
git diff --name-only 1f2cb585b25336ab9806adaeff9538b1ac3fa8ea..HEAD
```

Result:

```text
CHANGELOG.md
docs/engineering/ai-delivery/implementation-reports/BE-03-SPEC-implementation-report.md
docs/engineering/ai-delivery/tasks/BE-03.md
docs/publisher-services/decisions.md
docs/publisher-services/task-status.md
```

Only `docs/**` and `CHANGELOG.md` appear. No runtime path, no `thoth-api/`,
`thoth-client/` or `thoth-errors/` path, no `thoth-api/src/schema.rs`, no
migration path, no `Cargo.*` and no `.github/` path appears. No generated SDL or
client artifact is touched.

History integrity: the earlier commits `a3d4b1a7`, `20aa3a9a`, `5dc2b1dd` and
`a1acd3f6` remain present, in order, with unchanged SHAs; each remediation is one
ordinary commit on top of the then-current head. No rebase, merge, squash, amend or
force-push was performed at any point, and the branch was deliberately **not**
rebased even though `develop` contains the parent closeout merge commit
`7aeb715f9815a41b6357d8b6a7037ac62ebb25bb`. No CI workflow was dispatched
manually at any point.

Reserved branch absence, re-verified after each remediation:

```text
git branch --list feature/publisher-services/be-03            -> empty
git ls-remote --heads origin feature/publisher-services/be-03 -> empty
```

No BE-03 runtime implementation exists: no `feature/publisher-services/be-03`
branch, no migration, no `schema.rs` change, no Rust code, no test and no
generated contract change.

Relative links: every relative link in the four changed documentation files was
resolved against the filesystem and exists.

Template completeness: `BE-03.md` covers every section of
`task-specification-template.md` — objective, background and authority, scope,
non-goals, invariants, required behaviour including success, failure,
authorization, concurrency/idempotency and compatibility, data and migration
requirements, observability and operations, acceptance criteria, required
tests, rollout, rollback, stop conditions, expected implementation report,
recommended execution, branch and integration plan, and approval. No required
field is left unresolved and no mandatory design decision is left as `TBD`; the
only intentionally empty fields are the approval signature block, which the
approver completes, and which the template requires to remain empty until then.

Changelog: one entry under the existing `## [Unreleased]` / `### Added` heading,
amended in place by each remediation where it had become materially false or
incomplete; no second entry and no duplicate heading created.

Internal consistency: after each remediation, the authorization rule, the no-op
versus repair rule, the coordinator rule and the source/actor contract were
re-read across `BE-03.md` sections 1, 2.1, 3, 4, 5, 7.2, 7.3, 7.4, 7.5, 7.6, 7.7,
8, 9, 11.1, 12.2, 15, 18 and 19, this report, `decisions.md` section 3a,
`task-status.md` and the `CHANGELOG.md` entry, and each states the same rule. The
second remediation additionally re-read the capability rule across `BE-03.md`
sections 1, 2.1 item 4, 3, 4, 5, 7.1, 10, 10.1, 11.1, 12.2, 14, 15, 18.1, 18.2,
18.5 and 20, `decisions.md` sections 1 and 3a, `task-status.md` and the
changelog entry; the list-argument shape across `BE-03.md` sections 12, 14.1,
14.3 and 18.1; the assignability invariant across sections 7.3, 7.7, 18.1, 18.5,
18.6 and 19; and the decision authority condition across `decisions.md` section
3a, `BE-03.md` stop condition 5 and `task-status.md`. The third remediation
re-read the two-trigger consequence across `BE-03.md` sections 2.1 item 8, 6.3,
6.4, 7.3 step 9, 7.8, 7.9, 17.1, 18.1, 18.4, 20 and stop conditions 19 and 20,
this report's decisions 5, 16 and 18 and sections 7, 8, 13 and 15, and
`decisions.md` section 3a, confirming that each states the same rule: three
observable consequences on a committed change, none on a true no-op, stale
request or rollback, no work of another publisher affected, a set-based trigger
`UPDATE` rather than an application loop, the token retained on `publisher`, and
escalation rather than trigger modification if the evidence is bad. The stop
condition renumbering (former 19 to 20, new 19 inserted) was checked against
every cross-reference to a stop condition number in all five files; the only
numbered references are to conditions 5, 9, 15, 18 and the new 19, and all
resolve correctly.

Transient-state sweep (ADR-0005): all five changed files were swept for prose
that ordinary lifecycle progression would falsify. Removed: "Pull request:
intentionally deferred"; "The BE-03-SPEC pull request is **not opened yet**, by
instruction"; "CI status: PENDING; no pull request is open yet"; and
`decisions.md`'s `PROPOSED - AWAITING CTO DECISION` status line together with
the stop condition that depended on a literal `APPROVED` word appearing in a
Markdown file. `task-status.md`'s BE-03 `TBD` pull-request cell was replaced
with the durable specification-PR reference. Retained deliberately: durable
statements of branch, task, scope, base commits, validation performed before
publication, the absence of any manual workflow dispatch, and the unauthorized
status of runtime implementation — none of which merging or reviewing can
falsify. Retained as explicitly historical: the authoring-time preflight in
section 1.2 and the authoring-time observations in section 10, both framed as
records of what was verified when the specification was written. No status-only
follow-up task or requirement was created by this sweep.

## 10. Manual verification

Environment: local checkout at base
`1f2cb585b25336ab9806adaeff9538b1ac3fa8ea`.

Steps: the merged BE-01 and BE-02 implementations were read directly —
`policy.rs`, `model/publisher/mod.rs`, `model/publisher/policy.rs`,
`model/publisher_distribution_platform/{mod,crud}.rs`,
`graphql/{model,query,dataloader}.rs`, `schema.rs`, `thoth-errors/src/lib.rs`,
both migrations, `thoth-client/build.rs` and
`thoth-client/assets/schema.graphql` — before any design decision was recorded.
The first remediation pass re-read `policy.rs`, `model/publisher/policy.rs`,
`model/imprint/policy.rs`, `model/publisher_distribution_platform/crud.rs`,
`model/publisher_distribution_platform/mod.rs`, `schema.rs` and both migrations
before changing any rule, and ran the call-site and account-table searches
recorded below. The second remediation pass re-read
`model/publisher/mod.rs` (the capability model), `graphql/query.rs` (the merged
list-filter argument convention), `thoth-client/assets/schema.graphql` (the
rendered argument shapes and enum reachability), the root `Cargo.toml`
workspace declaration, ADR-0001 sections 4.1 to 4.7 and 7, ADR-0005 sections 4.1
and 6, `docs/publisher-services/README.md` and `BE-01.md` before changing any
rule. The third remediation pass re-read
`thoth-api/migrations/20250000_v1.0.0/up.sql` (both `public.publisher` triggers
and the `diesel_set_updated_at` and `publisher_work_updated_at_with_relations`
function bodies), `thoth-api/migrations/20260812_v1.7.0/up.sql` (BE-02's
`publisher_distribution_platform` trigger installation),
`thoth-api/src/graphql/model.rs`, `thoth-api/src/graphql/query.rs`,
`thoth-api/src/model/work/crud.rs`, `thoth-client/assets/schema.graphql`,
`thoth-client/assets/queries.graphql`, `thoth-client/src/lib.rs`,
`thoth-export-server/src/specification/handler.rs` and
`thoth-export-server/src/record.rs`, and re-ran the write-bypass search, before
changing any rule.

Every observation below is an **authoring-time record** of what was verified in
the merged code when this specification was written. It is retained as
historical evidence and is not a claim about any later repository state.

Observed results feeding the specification:

- `require_publisher_for` resolves to `PUBLISHER_USER`, so the protected read is
  a per-publisher role check rather than an identity, and the least-privilege
  matrix of section 11.1 is expressible with **no** new helper, role, ownership
  table or authorization framework;
- BE-01's package storage exists with no GraphQL surface, and BE-01's
  `PublisherCapability` and `ThothPackage::capabilities()` already provide a
  complete code-owned, exhaustively mapped, deterministically ordered capability
  model with **no** persisted capability state — so ADR-0001 section 4.4's
  effective-capability exposure is satisfiable by BE-03 with no new type, no new
  mapping and no migration beyond the token and audit table;
- BE-02's assignment lifecycle, linked-group normalization, activation identity
  and non-assignability rules are complete and correct for BE-03's purposes, so
  the specification reuses them and prohibits a second implementation;
- the only `FOR UPDATE` in `thoth-api/src` is BE-02's publisher-row lock, so
  BE-03 can adopt one identical lock order with no possibility of inversion;
- `publisher_history`'s account-FK-free `user_id text NOT NULL` establishes the
  repository's actor representation, so a free-text `actor` column with a
  source-scoped provenance contract is a continuation of existing practice
  rather than a new identity model;
- the repository's pagination, ordering, list-filter and error conventions are
  uniform enough that BE-03's new query, report and count surfaces can follow
  them exactly, which is what makes the exact additive SDL inventory of section
  14.1 predictable in advance of implementation.

Observed results feeding the remediations:

- `require_publisher_for` = `require_role_for_publisher(value,
  Role::PublisherUser)`; `require_role_for_publisher` short-circuits for
  superusers and otherwise calls `has_role_for_org` with exactly the one role
  passed. No inheritance exists in `policy.rs`;
- `PublisherPermissions` carries `publisher_admin`, `work_lifecycle` and
  `cdn_write` as independent booleans and no `publisher_user` field;
  `require_publisher_admin_for` is referenced only by `model/imprint/policy.rs`;
  `PublisherPolicy::can_update` uses `require_publisher_for`;
- `PublisherDistributionPlatform::enable` gates its work on the private
  `is_normalized_fully_enabled`, which requires equal member count, all enabled,
  `disabled_at` none, one shared `activation_id` and one shared `enabled_at`;
  both `enable` and `disable` return `ThothResult<()>`;
- **zero non-test production call sites** of BE-02's pool-level
  `PublisherDistributionPlatform::enable`/`::disable`. This is stated as a
  reproducible command with its exact output rather than as an approximate
  count. Command, run from the repository root at this branch's head:

  ```bash
  grep -rEn '::(enable|disable)\(' thoth-api/src thoth-api-server/src thoth-client/src thoth-errors/src thoth-export-server/src src thoth-api/migrations | awk -F: '{print $1}' | sort | uniq -c
  ```

  Output:

  ```text
       14 thoth-api/src/graphql/distribution_platform_tests.rs
       44 thoth-api/src/model/publisher_distribution_platform/tests.rs
  ```

  All 58 matches are confined to those **two test files**, and no other file in
  any searched path matches. Both files are test-only:
  `thoth-api/src/graphql/mod.rs` declares `distribution_platform_tests` under
  `#[cfg(test)]`, and `thoth-api/src/model/publisher_distribution_platform/mod.rs`
  declares its `tests` module under `#[cfg(all(test, feature = "backend"))]`.
  Non-test production hits: **zero**. Neither function is exposed through
  GraphQL. The root `Cargo.toml` `[workspace] members` list is exactly
  `thoth-api`, `thoth-api-server`, `thoth-client`, `thoth-errors` and
  `thoth-export-server`, confirming no local production crate was omitted from
  that scope. **`thoth-app` is a separate repository and has no directory
  here**, so no `thoth-app/src` path was searched and none is claimed as
  evidence. This is **authoring-time specification evidence bound to this
  branch's head, and it is not a waiver**: the future BE-03 implementation must
  still rerun the complete section 18.9 search against its own implementation
  head and record the result there;
- `publisher_history` is `(publisher_history_id, publisher_id, user_id text NOT
  NULL, data jsonb NOT NULL, "timestamp" timestamp without time zone)` with only
  a primary key and a `publisher_id` foreign key. The `account` table and
  `publisher_history_account_id_fkey` exist only in `20250000_v1.0.0/down.sql`,
  i.e. they were removed by that migration, and `schema.rs` declares no `account`
  table;
- BE-02's merged migration already uses named `CHECK` constraints
  (`publisher_distribution_platform_enabled_state_check`), so the specified
  non-blank actor constraint follows existing repository style.

- `lock_publisher` is the only `FOR UPDATE` in `thoth-api/src`;
- BE-02's `enable`/`disable` acquire their own pooled connection and
  transaction;
- `diesel_manage_updated_at` is installed on `publisher`, so
  `publisher.updated_at` moves on any publisher row update;
- `publisher_history` stores `user_id text NOT NULL` and `data jsonb NOT NULL`
  with a legacy `timestamp without time zone`;
- `ThothPackage` does not appear in `thoth-client/assets/schema.graphql`, and
  `type Publisher` in that SDL has no `subscriptionPackage` field;
- `into_field_error` gives a distinct GraphQL type only to `InvalidSubjectCode`
  and `Unauthorised`;
- `publishersByDistributionPlatform` applies a mandatory `publisher_id ASC`
  tie-breaker;
- `PatchPublisher` enumerates its columns explicitly, so a new `publisher`
  column is not clobbered by `updatePublisher`;
- the `Publisher` struct field order matches `schema.rs` column order, so the
  new column must be appended in both.

Observed in the second remediation pass:

- `thoth-api/src/model/publisher/mod.rs` defines `PublisherCapability` with the
  six ADR-0001 values in declaration order, each carrying a GraphQL description,
  deriving `juniper::GraphQLEnum` under the `backend` feature;
  `ThothPackage::capabilities` is an exhaustive `match` returning the ordered
  `&'static` constants `OASIS_CAPABILITIES` (empty), `OBELISK_CAPABILITIES` and
  `SPHINX_AND_PYRAMID_CAPABILITIES`, and `has_capability` delegates to it;
- `grep -n 'capabilit' thoth-client/assets/schema.graphql` returns nothing, so
  `PublisherCapability` is not yet SDL-reachable — matching BE-01's recorded
  expectation that Juniper omits the unreferenced enum until BE-03 uses it;
- no capability column, table, override or index appears in `schema.rs` or in
  `thoth-api/migrations`; the only durable capability input is
  `publisher.subscription_package`;
- ADR-0001 section 4.4 requires the protected surface to expose current package,
  effective capability codes and enabled distribution platforms, and section 7
  lists `BE-03` among the affected tasks; `docs/publisher-services/README.md`
  records that protected package and effective-capability reads remain BE-03
  scope; `BE-01.md` excludes `Publisher.capabilities` from BE-01 while
  anticipating BE-03's use of the enum;
- `thoth-api/src/graphql/query.rs` declares equivalent list filters as
  `Option<Vec<T>>` with `#[graphql(default = vec![], …)]`, and
  `thoth-client/assets/schema.graphql` renders them as a nullable outer list of
  non-null members: `publishers: [Uuid!] = []` on `imprints`, `publishers` and
  `workCount`; `workTypes: [WorkType!] = []` and `workStatuses: [WorkStatus!] =
  []` on `works`. No merged list filter uses a non-null outer list;
- `thoth-api/src/model/publisher_distribution_platform/crud.rs` performs
  `if !platform.is_assignable() { return Err(...DistributionPlatformNotAssignable) }`
  **before** `db.get()?`, i.e. before any connection is acquired or transaction
  opened — confirming both that the wrapper's early check must be preserved and
  that the extracted primitive would otherwise inherit no check of its own;
- the root `Cargo.toml` declares `members = ["thoth-api", "thoth-api-server",
  "thoth-client", "thoth-errors", "thoth-export-server"]`; the repository's
  top-level directories are `db`, `docs`, `reports`, `scripts`, `src`, and the
  five crate directories. There is no `thoth-app` directory and no top-level
  `migrations` directory — migrations live in `thoth-api/migrations`;
- ADR-0005 section 4.1 item 10 prohibits approval-state-only commits, and
  section 6 requires durable rather than transient status prose, which is the
  basis for the authority-condition form adopted in `decisions.md` section 3a.

Observed in the third remediation pass:

- `public.publisher` carries **two** `UPDATE` triggers in
  `20250000_v1.0.0/up.sql`: `set_updated_at` (`BEFORE UPDATE`,
  `diesel_set_updated_at()`) and `set_work_updated_at_with_relations`
  (`AFTER UPDATE`, `publisher_work_updated_at_with_relations()`). The earlier
  passes recorded only the first;
- `diesel_set_updated_at()` sets `NEW.updated_at := current_timestamp` when
  `NEW IS DISTINCT FROM OLD AND NEW.updated_at IS NOT DISTINCT FROM
  OLD.updated_at`, so an explicit `updated_at` write suppresses it;
- `publisher_work_updated_at_with_relations()` is, on
  `NEW IS DISTINCT FROM OLD`, exactly one set-based
  `UPDATE work SET updated_at_with_relations = current_timestamp FROM imprint
  WHERE work.imprint_id = imprint.imprint_id AND imprint.publisher_id =
  NEW.publisher_id` and then `RETURN NULL`. It is a set-based statement, not a
  per-row loop, and its predicate bounds it to the target publisher;
- BE-02's `20260812_v1.7.0/up.sql` installs only
  `diesel_manage_updated_at('public.publisher_distribution_platform')` — there is
  **no** work-freshness trigger on the assignment table, so merged BE-02
  lifecycle changes refresh no work rows;
- `work.updated_at_with_relations` is publicly consumed: resolved on the `Work`
  type in `graphql/model.rs`; a `TimeExpression` argument on `works`,
  `work_count`, `books`, `book_count`, `chapters` and `chapter_count` in
  `graphql/query.rs` and in the generated
  `thoth-client/assets/schema.graphql`; orderable through
  `WorkField::UpdatedAtWithRelations` in `model/work/crud.rs`; and selected by
  `WorkLastUpdatedQuery`/`WorksLastUpdatedQuery` in
  `thoth-client/assets/queries.graphql`, exposed as
  `ThothClient::get_work_last_updated`/`get_works_last_updated`;
- `thoth-export-server/src/specification/handler.rs` calls exactly those two
  client methods in `by_work` and `by_publisher` and passes the value into
  `MetadataRecord::new`; `record.rs`'s `load_or_generate` serves the cached
  record only while `cached_timestamp >= self.last_updated` and otherwise
  regenerates and calls `update_cache`. Moving a publisher's work freshness
  therefore invalidates that publisher's cached export output;
- the write-bypass search was re-run and its exact output recorded above: 14
  matches in `graphql/distribution_platform_tests.rs`, 44 in
  `model/publisher_distribution_platform/tests.rs`, 58 total, zero non-test
  production hits. `graphql/mod.rs` declares `distribution_platform_tests` under
  `#[cfg(test)]` and `model/publisher_distribution_platform/mod.rs` declares
  `tests` under `#[cfg(all(test, feature = "backend"))]`.

Evidence link: none required; this is a documentation task.

## 11. CI

Live CI evidence is represented by the GitHub pull-request record, bound to its
exact head. This committed report does not duplicate transient CI state
(ADR-0005).

Durable facts: **no workflow was dispatched manually** at any point in this
branch's history. The change touches only `docs/**` and `CHANGELOG.md`, so the
repository's gating is expected to classify it as documentation-only. The
relevant checks are `check-changelog` plus the `classify` step of the gated
workflows.

## 12. Rollout and rollback

Initial state after merge: repository documentation only. The BE-03
requirements become repository-authoritative; nothing is implemented,
authorized, deployed or activated.
Activation required: none.
Feature flag/configuration: none.
Migration sequence: none.
Rollback/disable procedure: ordinary revert of the documentation pull request.
Monitoring required: none.

## 13. Known limitations and deferred work

- The BE-03/BE-04/APP-01 phase boundary is a specification candidate, not a
  decided decision, including its APP-01 reconciliation. It becomes an approved
  programme decision only when its stated authority condition is satisfied:
  explicit CTO specification approval of the exact content carrying it, and that
  exact content reaching `develop`. BE-03 implementation should not be
  authorized before then, because a decision the other way blocks BE-03 behind
  the BE-04 job schema.
- `Publisher.updatedAt` will move on every committed configuration change,
  including platform-only changes and linked-state repairs where no ordinary
  publisher metadata changed. This is accepted and documented in `BE-03.md`
  section 6.4 rather than avoided; it is an unavoidable consequence of storing
  the dedicated token on `publisher` under the existing `set_updated_at`
  trigger. The public timestamp discloses only that the record changed, never
  any package, capability, platform or commercial value.
- **A committed configuration change will also refresh
  `work.updated_at_with_relations` for every work of that publisher through its
  imprints**, because `public.publisher` also carries the existing `AFTER UPDATE`
  `set_work_updated_at_with_relations` trigger. This is the most consequential
  known limitation of the token-location decision and is deliberately not hidden:
  - it is a **public downstream freshness signal**, so a configuration change
    invalidates that publisher's cached export records and re-selects its works
    for incremental consumers. It is **not** distribution activation: no job,
    upload, feed or dissemination is created;
  - a **platform-only** change and an **OAPEN/DOAB linked-state repair** cause it
    **newly** relative to merged BE-02, which touches only
    `publisher_distribution_platform` and refreshes no work rows;
  - the committed transaction therefore writes one publisher row + bounded
    configuration/audit rows + N related work rows, so **row-lock footprint and
    transaction duration grow with catalogue size** while the publisher
    `FOR UPDATE` lock is held. The work rows are changed by one set-based trigger
    statement, not by any application loop, and BE-03 must not introduce one;
  - **BE-04 inherits this** with the coordinator's transaction boundary;
  - it is accepted at specification level because avoiding it fully would require
    changing BE-01's canonical package storage or the shared publisher trigger —
    wider changes this review finding does not justify — and because a separate
    version table would still not avoid it for package changes;
  - **acceptance is evidence-conditional, not unconditional.** `BE-03.md`
    sections 18.4 and 20 require the six-case database evidence with target and
    control works and a catalogue-scale write-amplification and lock-footprint
    measurement, put in front of the independent HIGH-risk implementation
    reviewer; stop condition 19 requires `BLOCKED` and a separate architecture
    escalation if the evidence is bad, and forbids the implementing agent from
    altering shared publisher trigger or storage semantics instead. No production
    duration or safe catalogue size is asserted anywhere, and the disposable
    measurement must not be presented as a production prediction.
- `service_configuration_updated_at` may appear as an additional key in future
  `publisher_history.data` snapshots, because the shared `Crud::update` macro
  serializes the whole `Publisher` struct into that untyped `jsonb` column. No
  existing key changes meaning and no existing row is rewritten, but consumers
  reading that column should expect the extra key.
- The single-coordinator boundary's bypass evidence is a repository search, so
  it is only as good as its scope. The specification now fixes that scope
  explicitly to the crates this repository contains, and records that
  `thoth-app` is external and cannot be covered by it — `thoth-app` remains
  assessed through the generated GraphQL contract under a separate task.
- The single-coordinator boundary is enforced by specification, review and the
  required section 18.9 call-site and bypass-search evidence, **not** by the type
  system. BE-02's retained pool-level `enable`/`disable` remain capable of writing
  desired platform assignments without moving the configuration token or writing
  an audit row; they are prohibited from new production configuration call sites
  rather than made incapable. Making that structurally impossible would change
  BE-02's merged public API and its tests, which is out of BE-03's scope.
- The audit `actor` column is deliberately not named `user_id`, so it does not
  match the older `publisher_history` column name. The specification records that
  as an intentional divergence: `source` fixes the actor's provenance, and a
  `user_id` name would misdescribe a future controlled backfill identity.
  `publisher_history` itself is untouched.
- BE-03 cannot enforce that a future MIG-01 uses the coordinator. Section 9.3
  states the requirement and makes failure to satisfy it a MIG-01 stop condition;
  the actual guarantee will come from MIG-01's own specification and review.
- `DistributionPlatformNotAssignable` and `EntityNotFound` will continue to
  surface as `INTERNAL_ERROR`. Only the stale-configuration case gets a distinct
  GraphQL type, because normalizing the other families would change BE-02's
  merged contract and is out of scope.
- The fast-default expectation for `ADD COLUMN ... DEFAULT CURRENT_TIMESTAMP`
  is stated as an expectation requiring measurement, not as an established
  fact; the specification requires a `relfilenode` comparison rather than an
  assertion.
- The specification fixes the audit source vocabulary at `SUPERUSER_API` and
  `MIGRATION_BACKFILL`. If MIG-01 later needs finer provenance, that is an
  additive enum migration with the recorded `ALTER TYPE ... ADD VALUE`
  constraints.
- No BE-03 implementation, branch, migration, code or test exists.

## 14. Unresolved issues

- The BE-03/BE-04/APP-01 phase boundary, including the APP-01 reconciliation,
  requires an explicit CTO decision. It is recorded as a specification candidate
  under a durable authority condition in
  `docs/publisher-services/decisions.md` section 3a, and tested by stop
  condition 5 in `BE-03.md`. No agent may declare it approved.
- Whether `PUBLISHER_ADMIN` should ever read the commercial service configuration
  **without** `PUBLISHER_USER` is left as an explicit CTO authorization decision.
  No repository authority currently supports it, so the specification adopts the
  least-privilege denial and routes any widening through stop condition 15 rather
  than deciding it here.

## 15. Agent self-assessment

The agent may identify risks but may not approve the task.

Suggested review focus:

- **the concurrency token.** Confirm that a dedicated column is the smallest
  robust choice, that the two rejections in `BE-03.md` section 6.3 are
  factually right about the `set_updated_at` trigger and about assignment-only
  changes, and that the `GREATEST` monotonicity rule in section 6.2 genuinely
  closes the ABA hazard described;
- **the BE-02 composition hazard.** Confirm that BE-02's pool-based
  `enable`/`disable` really would self-block inside the coordinator's
  transaction, and that the connection-scoped refactor in section 7.7 is the
  right resolution rather than a lock-order stop condition;
- **the authorization matrix (finding 1).** Confirm that
  `require_publisher_for` requires exactly `PUBLISHER_USER`, that no repository
  authority makes `PUBLISHER_ADMIN`, `WORK_LIFECYCLE` or `CDN_WRITE`
  independently sufficient, that the matrix, prose, prescription and tests in
  sections 7.2, 11.1 and 18.2 now say one identical thing, and that the
  `zitadel_id IS NULL` path fails closed;
- **the reachability of linked-state repair (finding 2).** Confirm that the
  unconditional per-group primitive call genuinely reaches BE-02's normalization
  for a membership-equal split pair, that the three-part no-op definition in
  section 7.4 is exactly right, that the outcome value in section 7.7 is
  sufficient and minimal, that a repair's single audit row is distinguishable
  through `configurationVersion` alone, and that no second linked-platform
  algorithm is introduced;
- **the single write authority (finding 4).** Confirm the coordinator's ownership
  and contract in section 7.6, that routing the mutation through it makes the
  token/audit claims true for production paths, that retaining BE-02's
  pool-level functions unchanged is the right compatibility choice, that the
  zero-production-call-site evidence holds at the reviewed head, and that the
  honest limitation is stated rather than overclaimed;
- **the audit source/actor contract (finding 4, MIG-01 seam).** Confirm that
  `actor text NOT NULL` plus source-scoped provenance plus the non-blank check
  constraint is satisfiable by both source values without inventing credentials,
  and that keeping `MIGRATION_BACKFILL` now — rather than deferring it — is the
  better of the two alternatives recorded in section 5.1;
- **the not-found-before-authorization ordering** in section 11.2, which is a
  deliberate disclosure decision;
- **the BE-03/BE-04/APP-01 boundary (finding 3)**, which is a programme decision
  for the CTO, not an engineering conclusion. Confirm in particular that the
  APP-01 scope split is complete and that refining the earlier
  back-catalogue-status clause is a programme-local decision rather than a
  shared-architecture change requiring an ADR;
- **the audit shape.** Confirm the canonical state is bounded, that actor and
  source are columns rather than JSON fields, and that the key-set assertion
  test is strict enough to prevent later leakage, including of activation
  identifiers;
- **the SDL inventory.** Confirm that section 14.1 is complete, that
  `ThothPackage` and `PublisherCapability` reachability are the only implicit
  additions, and that the three list arguments are specified in the merged
  `[T!] = []` shape rather than a stricter one;
- **the ADR-0001 capability reconciliation (P1).** Confirm that ADR-0001 section
  4.4 does require effective capability codes on the protected surface, that
  `effectiveCapabilities: [PublisherCapability!]!` derived from
  `ThothPackage::capabilities()` satisfies it without persisting anything or
  writing a second mapping, that the ordering guarantee is real rather than
  incidental, that the single section 11.1 read decision genuinely covers the
  field, and that treating this as reconciliation with approved architecture —
  rather than as a new decision or a fresh deferral — is correct;
- **the `Publisher.updatedAt` consequence (first review, P2).** Confirm that the
  trigger really does move it whenever the token is written, that a stale request
  and a true no-op genuinely perform no publisher `UPDATE`, that the disclosure
  analysis is honest, and that the `publisher_history.data` additional-key
  statement is accurate and clearly separated from the configuration audit;
- **the work-freshness cascade (second review, P1) — the highest-value review
  target in this pass.** Confirm against the migrations that `public.publisher`
  really carries both `set_updated_at` and `set_work_updated_at_with_relations`;
  that section 2.1 item 8 and section 6.4 describe the trigger function's
  set-based statement and its `imprint.publisher_id = NEW.publisher_id` bound
  correctly; that the three-value six-case table in section 18.4, its fixture and
  its control work are sufficient to prove the behaviour rather than assume it;
  that section 7.8's write-footprint statement (one publisher row + bounded
  configuration/audit rows + N work rows) has fully replaced the earlier
  "touches exactly one publisher" / "no multi-object ordering" / "bounded by the
  17-value inventory" claims wherever they were load-bearing; that section 6.3's
  re-weighing of the separate-table alternative states its real benefit honestly
  instead of dismissing it; that retaining the token on `publisher` is the right
  bounded decision given BE-01's canonical package storage; that section 7.9's
  BE-04 inheritance statement is accurate; that section 17.1 no longer permits a
  "no observable downstream data change" reading and does not describe the
  cascade as distribution activation; and that stop condition 19 is the right
  control — escalate, never silently change shared trigger or storage semantics —
  and does not fire on the mere existence of the cascade;
- **the connection-scoped assignability invariant (P2).** Confirm that requiring
  the primitive to check `is_assignable()` itself is defence in depth rather
  than a second algorithm, that the pool-level wrapper's merged behaviour is
  genuinely unchanged, and that the direct `enable_on(JISC_NBK)` regression in
  section 18.6 proves failure *before* any write rather than reliance on
  rollback;
- **the corrected bypass-search scope (first review, P2) and its corrected
  evidence statement (second review, P2).** Confirm that the listed paths cover
  every local production crate, that the workspace-declaration check is the right
  completeness control, that treating `thoth-app` as external is correct, and
  that section 10's search statement is now exactly reproducible — one quoted
  command, its exact per-file output, zero non-test production hits, all matches
  confined to the two named `#[cfg(test)]`-gated files, and no approximate count
  anywhere. Confirm also that it is framed as authoring-time evidence which does
  **not** waive section 18.9's requirement that the future implementation rerun
  the search at its own head;
- **the decision authority condition (P2).** Confirm that the wording in
  `decisions.md` section 3a and stop condition 5 is one identical rule, that it
  is truthful before and after approval and before and after merge, that it
  cannot produce a false block, and that it creates no requirement for a
  status-only follow-up commit.

The remediated specification has **not** been self-reviewed and must receive a
fresh independent specification review by a model and context that did not author
or remediate it.
