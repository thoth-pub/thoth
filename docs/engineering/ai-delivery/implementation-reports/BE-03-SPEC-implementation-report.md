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
Pull request: intentionally deferred (see section 1.1)
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

Consequently the BE-03-SPEC pull request is **not opened yet**. Once the
closeout merges into `develop` with an ordinary merge commit, the closeout
content becomes an ancestor of `develop`, and this branch can be opened as a
draft pull request against `develop` showing only its own bounded diff — with
no rebase and no rewrite of this head.

The branch history must not be squashed or rewritten.

### 1.2 Preflight

Performed before any edit, and unchanged from the `BE-02-CLOSEOUT-01`
preflight recorded in that task's report:

```text
origin/develop                     = bcb6ce3081abb14467798b372fcc3e6af9da1c6a
PR #805                            = MERGED (merge commit = the SHA above)
BE-01 (PR #779)                    = MERGED
BE-02 (PR #805)                    = MERGED
THOTH-GQL-DATALOADER-01 (PR #802)  = MERGED
ADR-0001/0002/0003/0004/0005/0007  = present and merged on develop
Existing BE-03 spec branch/PR/file = none
Existing BE-03 implementation branch/PR = none
feature/publisher-services/be-03   = absent, and remains absent
Working tree                       = clean
```

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
- one further commit adds this implementation report

## 4. Files changed

- `docs/engineering/ai-delivery/tasks/BE-03.md` (new)
  - reason: the bounded written implementation specification BE-03 requires
    before it may be authorized.
  - behavioural effect: none. It authorizes nothing.
- `docs/publisher-services/decisions.md`
  - reason: records the `PROPOSED` BE-03/BE-04 phase-boundary programme
    decision as a new section 3a, bound to BE-03-SPEC approval.
  - behavioural effect: none.
- `docs/publisher-services/task-status.md`
  - reason: the BE-03 row and next-action 10 now point at the written
    specification and record `IMPLEMENTATION NOT AUTHORIZED` in durable
    ADR-0005 language.
  - behavioural effect: none.
- `CHANGELOG.md`
  - reason: root `AGENTS.md` section 13.
  - behavioural effect: none. Added under the existing `### Added` heading.

## 5. Implementation decisions

Design decisions settled in the specification, each derived from merged code
rather than from the design document alone:

1. **Authorization uses the existing model.** `PolicyContext::require_publisher_for`
   already expresses "superuser, or a `PUBLISHER_USER` role scoped to this
   publisher's ZITADEL organisation", and `require_superuser` expresses
   superuser-only. No new ownership table, role, framework or account table is
   introduced. Because an account may hold scoped roles for several
   organisations, ownership is specified as a per-publisher role check rather
   than a single-publisher identity.
2. **Actor identity is `IntrospectedUser.user_id` stored as `user_id text`.**
   The existing `publisher_history` table already stores exactly that, written
   in the same transaction as the entity change. The audit table follows that
   convention rather than inventing an identifier format, and does not modify
   `publisher_history`. The legacy `timestamp without time zone` type on that
   table is deliberately **not** copied; the new table uses `timestamptz`, as
   BE-02 does.
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
5. **Lock order is identical to BE-02 and cannot invert.**
   `publisher_distribution_platform::crud::lock_publisher` is the **only**
   `FOR UPDATE` in `thoth-api/src`. BE-03 takes the same lock, on the same
   single row, as the first statement of its transaction, and a mutation
   touches exactly one publisher, so there is no multi-object ordering.
6. **A real composition hazard was found and resolved additively.** BE-02's
   `enable` and `disable` take `&PgPool`, call `db.get()?`, and open their own
   transaction on their own pooled connection. Calling either from inside the
   BE-03 transaction would run on a *different* connection and block on the
   publisher row lock the BE-03 transaction itself holds — a self-block
   resolving only by timeout. This is **not** a lock-order conflict and is not a
   stop condition; the specification requires extracting connection-scoped
   `enable_on`/`disable_on` functions and reducing the existing public functions
   to delegation, so BE-02's semantics and tests are unchanged and BE-03 reuses
   the domain invariants rather than re-implementing them.
7. **No second DataLoader.** BE-02's request-local
   `publisher_distribution_platforms` loader is keyed by `publisher_id` and
   already returns enabled assignments in canonical order — exactly the
   protected field's shape — so the specification requires reusing it, per
   ADR-0007 and the programme's reuse rule. The report's `lastChange` needs no
   loader because it is fetched eagerly for the page in one set-based
   `DISTINCT ON` statement.
8. **Change metadata lives only on the superuser report row type.** Keeping
   `PublisherServiceConfiguration` identical for owner and superuser callers
   avoids conditionally-visible fields and their failure modes, and keeps actor
   and source off the owner surface entirely. Audit `before_state`/`after_state`
   JSON is exposed to **nobody** through GraphQL in BE-03.
9. **`ThothPackage` becomes SDL-reachable for the first time.** It is currently
   absent from `thoth-client/assets/schema.graphql` because BE-01 added no
   GraphQL surface. The specification records this as an expected SDL diff item
   to be confirmed, not discovered, and notes that type reachability is not
   value exposure.
10. **Exactly one new error variant and one new `into_field_error` arm.**
    `into_field_error` currently gives a distinct type only to
    `InvalidSubjectCode` and `Unauthorised`. The stale case needs a distinct
    `STALE_SERVICE_CONFIGURATION` type so APP-01 can render "configuration
    changed; reload". Every other family keeps its current mapping;
    `DistributionPlatformNotAssignable` and `EntityNotFound` continuing to
    surface as `INTERNAL_ERROR` is recorded as a known limitation rather than
    silently normalized.
11. **Semantic no-ops move no token and write no audit row.** A version that
    changed without a configuration change would force needless retries on every
    other client and fill the audit table with identical before/after rows. This
    also matches BE-02, whose same-state transitions already move no timestamp.
    A stale request still fails even when it would have been a no-op, because
    the version check precedes validation and diffing.
12. **Not-found precedes the role check** for the single-publisher query, so an
    unknown ID returns `EntityNotFound` to any authenticated caller. This
    mirrors the existing `load_current` convention and discloses nothing new,
    because publisher existence is already public through the anonymous
    `publisher`/`publishers` queries. It is recorded as a deliberate choice.

Deviations from an approved source:

- The approved design's API section implies that the replace mutation creates
  the required jobs and that the staff report includes job state. That
  contradicts the approved task decomposition, which places job persistence in
  BE-04, and the rollout, which holds automatic job creation inactive. Rather
  than guess, the contradiction is surfaced as a `PROPOSED` programme decision
  in `docs/publisher-services/decisions.md` section 3a and recorded as a BE-03
  stop condition if the CTO decides otherwise.

## 6. Database and migration effects

Migration added: NO

This task creates no migration. It **specifies** the additive migration BE-03
would require — one `publisher` column, one closed source enum, one audit table
and its index — and explicitly excludes `distribution_job`,
`distribution_job_target`, `distribution_job_attempt`, worker-role persistence
and any credential or configuration-secret table.

## 7. API and compatibility effects

GraphQL/API changes: NONE in this task. The specification defines the exact
additive inventory BE-03 would add and requires the SDL diff to equal it.
Generated schema/client updates: NONE. `thoth-client/assets/schema.graphql` is
untouched.
Backwards compatibility: unaffected.
Deprecations: NONE
Cross-repository dependencies: the reserved BE-03/APP-01 exact-SHA schema
pinning control is preserved and reinforced; the specification requires BE-03's
implementation report to record the exact backend head SHA for APP-01 pinning.
`thoth-app` and `thoth-dissemination` are untouched.

## 8. Authorization and security

Authorization paths changed: NONE. No authorization code exists in this diff.
Roles/scopes involved: none changed. The specification consumes the existing
`Superuser` and `PublisherUser` ZITADEL roles.
Negative authorization tests: not applicable to a documentation change; the
specification **requires** the full negative matrix in section 18.2, including
anonymous, no-role, other-publisher, `PublisherAdmin`-cannot-mutate and
`zitadel_id IS NULL` cases.
Secret or personal-data handling: none. No credential, token, endpoint, bucket
or account identity appears in the diff. The specification forbids them in the
audit JSON and requires a key-set assertion test.
Security limitations: the pre-existing `ThothError::DatabaseError` rendering of
driver text is recorded honestly as pre-existing, not claimed absent.

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

Path containment:

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

No runtime path, no `thoth-api/src/schema.rs`, no `migrations/` path, no
`Cargo.*` and no `.github/` path appears.

Reserved branch absence:

```text
git branch --list feature/publisher-services/be-03            -> empty
git ls-remote --heads origin feature/publisher-services/be-03 -> empty
```

Relative links: every relative link introduced by this change was resolved
against the filesystem and exists.

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

Changelog: one entry added under the existing `## [Unreleased]` / `### Added`
heading; no duplicate heading created.

## 10. Manual verification

Environment: local checkout at base
`1f2cb585b25336ab9806adaeff9538b1ac3fa8ea`.

Steps: the merged BE-01 and BE-02 implementations were read directly —
`policy.rs`, `model/publisher/mod.rs`, `model/publisher/policy.rs`,
`model/publisher_distribution_platform/{mod,crud}.rs`,
`graphql/{model,query,dataloader}.rs`, `schema.rs`, `thoth-errors/src/lib.rs`,
both migrations, `thoth-client/build.rs` and
`thoth-client/assets/schema.graphql` — before any design decision was recorded.

Observed results feeding the specification:

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

Evidence link: none required; this is a documentation task.

## 11. CI

CI status: PENDING; no pull request is open yet, by design (section 1.1). The
repository's documentation-only gating is expected to classify this change as
docs-only.
Checks: `check-changelog` plus the `classify` step of the gated workflows.
Failures or warnings: none known.

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

- The BE-03/BE-04 phase boundary is `PROPOSED`, not decided. BE-03
  implementation should not be authorized until the CTO decides it, because a
  decision the other way blocks BE-03 behind the BE-04 job schema.
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

- The BE-03/BE-04 phase boundary requires an explicit CTO decision. It is
  recorded as `PROPOSED` in `docs/publisher-services/decisions.md` section 3a
  and as stop condition 5 in `BE-03.md`.

## 15. Agent self-assessment

The agent may identify risks but may not approve the task.

Suggested review focus:

- **the concurrency token.** Confirm that a dedicated column is the smallest
  robust choice, that the two rejections in `BE-03.md` section 6.3 are
  factually right about the `set_updated_at` trigger and about assignment-only
  changes, and that the `GREATEST` monotonicity rule in section 6.2 genuinely
  closes the ABA hazard described;
- **the BE-02 composition hazard.** Confirm that BE-02's pool-based
  `enable`/`disable` really would self-block inside the BE-03 transaction, and
  that the connection-scoped refactor in section 7.6 is the right resolution
  rather than a lock-order stop condition;
- **the authorization matrix.** Confirm that `require_publisher_for` expresses
  owner-only reads correctly for accounts scoped to several publishers, and
  that the `zitadel_id IS NULL` path fails closed;
- **the not-found-before-authorization ordering** in section 11.2, which is a
  deliberate disclosure decision;
- **the BE-03/BE-04 boundary**, which is a programme decision for the CTO, not
  an engineering conclusion;
- **the audit shape.** Confirm the canonical state is bounded, that actor and
  source are columns rather than JSON fields, and that the key-set assertion
  test is strict enough to prevent later leakage;
- **the SDL inventory.** Confirm that section 14.1 is complete and that
  `ThothPackage` reachability is the only implicit addition.
