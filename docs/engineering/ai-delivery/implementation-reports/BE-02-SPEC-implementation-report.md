# BE-02-SPEC Implementation Report

Documentation-only specification-authoring task. It writes the bounded BE-02
task specification and its directly required control reconciliation. It
implements no BE-02 runtime code and authorizes nothing.

## 1. Repository state

Repository: `thoth-pub/thoth`
Workflow: STANDARD
Base branch: `develop`
Base commit: `5a8c27b1b7c11a4f6bd26d459556468099f8c1f4` (freshly verified by
`git ls-remote origin refs/heads/develop` and
`gh api repos/thoth-pub/thoth/git/refs/heads/develop` before any edit)
PR target: `develop`
Programme integration branch: None
Task branch: `feature/publisher-services/be-02-spec`
Head commit: the current head of that branch; the exact reviewed head is the
GitHub pull-request record
Pull request: [#788](https://github.com/thoth-pub/thoth/pull/788) (draft)
Expected branch deletion after merge: YES
Final programme PR required: NO
Implementing model: Claude Opus 5
Reasoning level: HIGH

## 2. Scope confirmation

Approved specification: this is the specification-authoring task itself. The
governing controls are root `AGENTS.md`, `operating-model.md` (Gate 1),
`task-specification-template.md`, `branching-and-release-workflow.md`,
`risk-classification.md`, `release-gates.md`, ADR-0002, ADR-0003, ADR-0004 and
ADR-0005.

Implemented objective: author the complete bounded BE-02 implementation
specification, settling every low-level representation choice from live
repository conventions, and record it as the repository-authoritative BE-02 task
record; then remediate the six independent-review findings recorded in
section 16.

Out-of-scope changes made: NONE.

Explicitly not done:

- no BE-02 runtime code, migration, PostgreSQL type, Rust enum, model, GraphQL
  field or query, descriptor, assignment table, schema change or test;
- no `feature/publisher-services/be-02` implementation branch;
- no modification of [issue #765](https://github.com/thoth-pub/thoth/issues/765);
- no ADR content, platform inventory, evidence ledger, evidence matrix or
  evidence count change;
- no unrelated ADR-0003, operating-model or historical control-record cleanup;
- no production, deployment, release, migration-execution, credential or
  workflow-dispatch action.

## 3. Commits

See the GitHub pull-request record for the exact commit list and SHAs. The
commits are documentation-only.

## 4. Files changed

- `docs/engineering/ai-delivery/tasks/BE-02.md`
  - reason: the bounded BE-02 task specification required by Gate 1.
  - behavioural effect: none at runtime. It becomes the repository-authoritative
    BE-02 task record when this exact content is reachable from `develop`.
- `docs/publisher-services/task-status.md`
  - reason: the BE-02 tracker row and next-action item pointed at a
    non-existent specification and carried `TBD`, which the controls treat as
    missing work. Linking the committed task record and stating the remaining
    gates is a material correction, not lifecycle metadata.
  - behavioural effect: none. BE-02 remains `BLOCKED` and unauthorized.
- `CHANGELOG.md`
  - reason: root `AGENTS.md` section 13 requires every PR to update
    `CHANGELOG.md` under `## [Unreleased]`.
  - behavioural effect: none.
- `docs/engineering/ai-delivery/implementation-reports/BE-02-SPEC-implementation-report.md`
  - reason: root `AGENTS.md` section 14 requires an implementation report before
    review.
  - behavioural effect: none.

Runtime files changed: NONE. No file under `thoth-api/`, `thoth-api-server/`,
`thoth-client/`, `thoth-errors/`, `thoth-export-server/`, `.github/workflows/`,
`Cargo.toml`, `Cargo.lock` or `Makefile` is touched.

## 5. Implementation decisions

Decisions taken while authoring the specification, each grounded in live
repository evidence at `5a8c27b1`:

1. **Specification mechanism: committed task record, not a GitHub issue.**
   `operating-model.md` Gate 1 and root `AGENTS.md` section 1 permit either a
   committed specification or an authoritative GitHub issue, and the issue route
   is the less redundant of the two in principle. It was attempted first and is
   not available here: GitHub rejects an issue body above 65,536 characters
   (`GraphQL: Body is too long (maximum is 65536 characters)`), and the complete
   specification is approximately 97,000 characters. Root `AGENTS.md` section 1
   makes an issue sufficient *only when it contains* the information required by
   the task specification template, so an issue cannot satisfy Gate 1 for this
   task without either deleting required content or splitting the specification
   across mutable comments, which would leave an exact-head review with an
   ambiguous target. The committed route is therefore the smallest compliant
   mechanism, and it matches the existing repository convention for every task
   specification to date (BE-01, ADR-01, THOTH-DB-CTRL-01).
2. **PostgreSQL type `public.distribution_platform`** - the identity named by
   ADR-0002 section 4.1 and the approved design, and consistent with the current
   enum type-name convention (`thoth_package`, `location_platform`). No approved
   stable identity is changed.
3. **Declaration order is binding** - PostgreSQL orders enum values by
   declaration order, and the specification uses that for deterministic ordering
   in `distributionPlatformOptions` and `Publisher.distributionPlatforms`.
4. **Rust enum follows the BE-01 `ThothPackage` pattern exactly**
   (`diesel_derive_enum::DbEnum` with `ExistingTypePath`, per-variant
   `db_rename`, serde and strum `SCREAMING_SNAKE_CASE`, `juniper::GraphQLEnum`),
   with `Default` deliberately **not** implemented because a default would act
   as a fallback value.
5. **`timestamp with time zone` for the new table** - the newer repository table
   convention (`award`, `book_review`, `additional_resource`), which also makes
   the physical type agree with the `Timestamptz` mapping already used
   throughout `thoth-api/src/schema.rs`, rather than relying on the legacy
   `without time zone` alias carried by older tables.
6. **Composite primary key `(publisher_id, platform)`** - required by the
   approved design. No existing Thoth table uses a composite primary key, so the
   specification flags this as a deliberate departure, requires the implementing
   agent to verify Diesel behaviour at its exact base, and makes an
   incompatibility a `BLOCKED` stop condition rather than grounds for silently
   substituting a surrogate key.
7. **`activation_id` is generated by the application, not by a column default** -
   a `DEFAULT uuid_generate_v4()` would generate two different values for a
   linked OAPEN/DOAB pair and would silently break the shared-activation
   invariant.
8. **Lifecycle timestamps use the transaction timestamp (`CURRENT_TIMESTAMP`)** -
   it is transaction-start time and therefore identical across every row written
   by one logical transition, which is exactly the determinism the linked pair
   requires. `clock_timestamp()` is prohibited.
9. **The row-level invariant is carried by `NOT NULL` declarations plus one
   single-row `CHECK`** (specification section 6.1.1). Because a row is created
   only by `ABSENT -> ENABLED`, every persisted row has an activation, so
   `activation_id` and `enabled_at` are `NOT NULL` and `disabled_at` is the only
   nullable lifecycle column. `enabled` carries **no** database default, because
   a `DEFAULT false` would advertise a never-activated disabled row as a valid
   persisted state. The `CHECK` then reduces to `enabled = (disabled_at IS
   NULL)`. The cross-row OAPEN/DOAB invariant remains enforced transactionally
   in the domain layer, with an explicit written justification for not adding a
   trigger or exclusion-constraint subsystem.
10. **Linked operations are evaluated at group level**, so a linked enable of a
    partially enabled group produces one new shared activation across both rows
    and repairs any pre-existing one-sided or split-activation state.
11. **Concurrency uses a `SELECT ... FOR UPDATE` lock on the publisher row plus
    `INSERT ... ON CONFLICT DO UPDATE`** - existing repository primitives named
    by `thoth-api/AGENTS.md` section 5; no new locking subsystem.
12. **`JISC_NBK` non-assignability is enforced in the domain layer, not in a
    database constraint** - assignability is code-owned descriptor metadata, and
    a database constraint would duplicate it and require a migration to activate
    `JISC_NBK` later.
13. **Descriptors use an exhaustive `match self` with no wildcard arm**, so an
    eighteenth variant fails to compile; `DistributionAdapterProfile` has 15
    values for 17 platforms, representing shared mechanism identity without
    collapsing destinations, and is internal only.
14. **Pagination reuses the repository's existing offset/limit style** (`limit`
    default 100, `offset` default 0, `order` input, `Vec<T>` plus a sibling
    `Int!` count). No connection, edge, cursor or `pageInfo` type is introduced,
    because none exists anywhere in the current schema.
15. **A mandatory `publisher_id` ascending tie-breaker** is specified for the
    reverse lookup, using the existing `apply_directional_order!`
    secondary-expression form. The existing `Publisher::all` has no tie-breaker;
    the specification explicitly forbids inheriting that defect.
16. **`activationId`, `disabledAt` and the adapter/feed profile are not exposed
    publicly**; `assignable` and `backCatalogueBehaviour` are, because the app
    and later consumers need them and they contain no credential or secret.
17. **`Crud` is deliberately not implemented for the assignment entity** -
    `Crud` is the generic contract for GraphQL CRUD entities with
    create/update/delete mutations, which BE-02 explicitly does not add.
18. **No committed generated-schema artefact is required** -
    `thoth-client/assets/schema.graphql` is written by `thoth-client/build.rs`
    at build time and is untracked; only `assets/queries.graphql` is tracked,
    and BE-02 adds no internal-client query.

Decisions taken during the independent-review remediation (section 16):

19. **No transient status field is committed.** `Status`, `Approved by` and
    `Approved for implementation by` were removed rather than corrected. Under
    ADR-0005 any value they could hold is either false before the event or stale
    after it, and the commit correcting them would invalidate the exact-head
    review that justified it. The durable substitute is the approval-authority
    triple in the header and section 24.
20. **The foreign key locks the referenced table.** Verified against the
    PostgreSQL 17 documentation: `ADD FOREIGN KEY` acquires `SHARE ROW
    EXCLUSIVE` on the referenced table in addition to the constrained table, and
    `SHARE ROW EXCLUSIVE` conflicts with `ROW EXCLUSIVE` but not with `ACCESS
    SHARE` or `ROW SHARE`. Publisher reads therefore continue; publisher writes
    are blocked while the lock is held. `NOT VALID` was considered and rejected:
    the child table is created empty, so there is no scan to defer, and it would
    not remove the referenced-table lock.
21. **`NOT VALID` and a deferred constraint were not adopted**, and the
    migration is not described as destructive or production-blocking. The
    conservative statement is that it briefly blocks writes to one populated
    table, with lock *acquisition* and queueing behind a long-running
    transaction as the dominant risk rather than the migration's own work.
22. **The N+1 mechanism was escalated, not chosen** (section 14). Every
    available option changes shared GraphQL architecture, adds a dependency,
    waives a standing control, or removes approved public API. The specification
    states the prohibition bindingly, keeps `thoth-api/AGENTS.md` section 6
    intact, and makes the CTO decision a hard gate before implementation
    authorization.
23. **Merge and environment state were separated structurally**, not by
    rewording. Section 14.1 states only what a merge guarantees in the
    repository; section 14.2 states environment behaviour conditionally on
    separately authorized deployment and migration execution.
24. **The GraphQL inventory was made binding and countable** in section 12.1,
    including an explicit table of the three internal Rust enums that must not
    appear in the generated SDL, so that an internal type cannot drift into the
    public contract by assumption.

Deviation from the governing controls: NONE.

## 6. Database and migration effects

Migration added: NO. This task changes documentation only. The specification
*requires* a future migration under ADR-0003 Architecture A, but adds none.

## 7. API and compatibility effects

GraphQL/API changes: NONE.
Generated schema/client updates: NONE.
Backwards compatibility: unaffected; no contract changes.
Deprecations: NONE.
Cross-repository dependencies: NONE. No other repository is changed, and no
downstream repository may act on this specification.

## 8. Authorization and security

Authorization paths changed: NONE.
Roles/scopes involved: NONE.
Negative authorization tests: not applicable to a documentation-only change; the
specification requires them of the future implementation (its section 18.7).
Secret or personal-data handling: none. No credential, token, publisher list,
private configuration value or personal datum is read into or recorded in this
task. No production or shared service was accessed.
Security limitations: none introduced.

## 9. Tests and checks

### Whitespace and diff hygiene

Command:

```text
git diff --check
```

Result: recorded in the pull-request record; no whitespace errors expected or
accepted.

### Documentation verification (per `docs/engineering/AGENTS.md` section 6)

Checked:

- relative paths and internal links from
  `docs/engineering/ai-delivery/tasks/BE-02.md` to `../../decisions/ADR-000*`,
  `../../design-references.md`, `../../../publisher-services/*`,
  `../operating-model.md`, `../task-specification-template.md` and `BE-01.md`;
- headings, section numbering and internal cross-references;
- durable status wording under ADR-0005 - the specification records a durable
  decision and an authority condition, and contains no `PENDING MERGE`,
  `AWAITING REVIEW` or `AWAITING CTO MERGE AUTHORIZATION` prose;
- no unresolved `TBD`, placeholder or ambiguous required field;
- canonical repository, branch and component names;
- the required `CHANGELOG.md` entry under `## [Unreleased]` with no duplicate
  heading;
- no duplicate or competing BE-02 specification exists.

### Rust/workspace checks

Not required and not run: no Rust, schema, migration, workflow or configuration
file is changed. `.github/workflows/` documentation-only gating applies.

### Repository state assertions

- `git status` was clean on `develop` at `5a8c27b1` before branching.
- No runtime file is in the diff.
- No branch named `feature/publisher-services/be-02` was created.

## 10. Manual verification

Environment: local read-only inspection of `thoth-pub/thoth` at
`5a8c27b1b7c11a4f6bd26d459556468099f8c1f4`, plus read-only GitHub inspection.

Steps and observed results:

1. Freshly verified the exact `develop` head by two independent routes (`git
   ls-remote` and the GitHub API): `5a8c27b1b7c11a4f6bd26d459556468099f8c1f4`.
2. Searched for any existing BE-02 equivalent: no
   `docs/engineering/ai-delivery/tasks/BE-02.md` or `BE-02-SPEC.md`; no branch
   containing `be-02`; no BE-02 issue; no BE-02 pull request; `DistributionPlatform`
   present only in documentation; `publisher_distribution_platform` absent
   entirely. No duplicate specification was created.
3. Confirmed ADR-0004 is `APPROVED AND REPOSITORY-AUTHORITATIVE` with exactly
   the 17 values, and the platform inventory is `FINAL INVENTORY APPROVED AND
   REPOSITORY-AUTHORITATIVE`.
4. Confirmed ADR-0005 is `APPROVED` and reachable from `develop`, and that the
   operating model and release gates carry the terminal-merge-evidence rules.
5. Confirmed ADR-0003 Architecture A remains authoritative and that no root
   `diesel.toml` exists.
6. Confirmed ADR-01 was architecture/evidence work only and implemented no BE-02
   runtime code.
7. Inspected the BE-01 precedent: PR #779's actual merged implementation
   (migration `20260805_v1.7.0`, `thoth-api/src/schema.rs`,
   `thoth-api/src/model/publisher/mod.rs`, `publisher/tests.rs`,
   `graphql/tests.rs`), not only its narrative report.
8. Inspected live conventions for migrations, PostgreSQL enums and constraints,
   UUID and timestamp types, foreign keys, partial indexes, primary keys,
   `updated_at` triggers, Diesel enum derivation, serde/strum/Juniper naming,
   model module layout, the `Crud` trait and `crud_methods!`, transactions and
   `apply_directional_order!`, GraphQL query organization, pagination and count
   conventions, public-query authorization behaviour, the database test harness
   and the generated-client contract.
9. Consulted the approved private design (Drive revision `3`) and recorded that
   its provisional section 5.2 platform list is superseded by ADR-0004.
10. Read [issue #765](https://github.com/thoth-pub/thoth/issues/765) read-only
    and did not modify it.

## 11. CI

CI status: PENDING at the time of writing; the live result is the GitHub
pull-request record. Documentation-only gating applies under PR #771.

## 12. Rollout and rollback

Initial state after merge: the BE-02 specification becomes
repository-authoritative. Nothing activates. BE-02 remains `BLOCKED` and
unauthorized, and the implementation branch must not exist.

Activation required: none. Merging this specification does not authorize BE-02
implementation.

Feature flag/configuration: not applicable.

Migration sequence: not applicable.

Rollback/disable procedure: revert the specification pull request. There is no
runtime effect. A later material correction to the specification requires its
own bounded task and pull request.

Monitoring required: none.

## 13. Known limitations and deferred work

- The specification pins no implementation base SHA. That is deliberate: the
  implementation base must be verified afresh at implementation time and bound
  to a separate explicit CTO authorization.
- The specification requires the implementing agent to re-verify Diesel's
  composite-primary-key behaviour at its own exact base, because no existing
  Thoth table uses one.
- The `EXPLAIN`-based partial-index verification is specified as "where
  practical", because planner choice depends on fixture size; the specification
  requires the implementing agent to record why if it is not practical.
- The migration is **not** lock-free. Establishing the foreign key takes a
  `SHARE ROW EXCLUSIVE` lock on the populated `publisher` table, which blocks
  publisher writes while held (reads are unaffected). The specification states
  this in section 13.3 and requires the implementation to reassess the actual
  generated DDL and verify the lock empirically via `pg_locks`. No production
  duration is claimed, no production database access is authorized, and
  production migration execution remains separately gated by CG-13.
- The N+1 mechanism for `Publisher.distributionPlatforms` is **not settled** and
  is escalated to the CTO (section 14 below).

## 14. Unresolved issues

**One architecture escalation is open and blocks implementation authorization.**

```text
BLOCKED - N+1 CONTROL REQUIRES ARCHITECTURE DECISION
```

`thoth-api/AGENTS.md` section 6 requires new lists and reports to avoid N+1
access and to use set-based SQL or batched loaders. Live inspection at the
preflight base found no mechanism to reuse: no DataLoader or `dataloader`
dependency, no `look_ahead` usage, no request-scoped state on the GraphQL
`Context` (which holds only `db`, `user`, `s3_client`, `cloudfront_client`), and
all 56 existing child-field resolvers in `thoth-api/src/graphql/model.rs` query
once per parent.

Adding `Publisher.distributionPlatforms` - which the approved design and
ADR-0002 section 4.4 both require - therefore cannot satisfy section 6 by
following any existing repository pattern. The exposure also arises through the
**pre-existing** `publishers` root query, so no change confined to BE-02's own
new root fields removes it.

The exact CTO decision required is recorded in specification sections 9.2.1 and
19.1, with four options: (A) authorize a request-scoped batching mechanism on
the GraphQL `Context`; (B) authorize a DataLoader dependency; (C) grant a
documented bounded exception to `thoth-api/AGENTS.md` section 6 for this field,
justified by the hard 17-row-per-publisher bound; or (D) defer the field out of
BE-02, which would contradict approved architecture.

This specification deliberately does not choose. Every option either changes
shared GraphQL architecture, adds a workspace dependency, waives a standing
repository control, or removes approved public API - none of which is an
implementing agent's decision. The `thoth-api/AGENTS.md` section 6 requirement
is neither weakened nor waived by the specification.

No other unresolved architecture decision remains, and no `TBD` or placeholder
is present in the specification.

## 15. Agent self-assessment

The authoring agent does not approve this specification or the future BE-02
implementation.

Suggested review focus:

1. Fidelity of the 17-value inventory, aliases, exclusions, display labels,
   behaviours, linked groups and adapter profiles against ADR-0004 sections 4.1,
   4.2, 4.3 and 5.
2. The activation-lifecycle table (specification section 7.1): whether all six
   transitions, their write/no-write behaviour, `activation_id` handling and
   idempotency are complete and internally consistent.
3. The group-level linked-transition rule (section 7.2), specifically whether
   the partially-enabled-group repair behaviour is the right reading of "linked
   platforms enabled in one logical operation share one activation identity".
4. Whether enforcing the cross-row OAPEN/DOAB invariant transactionally rather
   than in the database is the correct architectural boundary (section 7.2.1).
5. The composite-primary-key departure from existing repository practice
   (section 6.2) and its `BLOCKED` fallback.
6. The `timestamp with time zone` choice for the new table (section 6.1) against
   the mixed historical convention.
7. Whether the public GraphQL surface exposes the right fields and, in
   particular, whether withholding `activationId`, `disabledAt` and the
   adapter/feed profile is correct (sections 9.1 and 9.2).
8. Determinism of the reverse-lookup pagination and count/lookup agreement
   (sections 9.3, 9.4 and 18.7).
9. Completeness of the fail-closed table (section 10) against the invariant that
   an empty assignment set must never broaden processing.
10. That the specification authorizes nothing: no implementation, no branch, no
    production action.

Additional focus for the remediated content (section 16):

11. Whether the tightened row invariant (section 6.1.1) - `activation_id` and
    `enabled_at` `NOT NULL`, `disabled_at` nullable, no default on `enabled`,
    and `enabled = (disabled_at IS NULL)` - is genuinely equivalent to the
    section 7.1 state machine, and whether the row-existence rule that justifies
    the `NOT NULL` columns holds for every transition.
12. Whether the foreign-key lock statement (section 13.3) is accurate for
    PostgreSQL 17 and appropriately conservative, neither understating the
    `publisher` write block nor overstating it as destructive or
    production-blocking.
13. Whether the N+1 escalation (sections 9.2.1 and 19.1) is correctly blocked
    rather than silently resolved, and whether the four options are stated
    fairly and completely.
14. Whether merge, environment deployment, environment migration execution,
    assignment creation and activation are now kept strictly distinct
    (section 14), with no residual claim that an environment changed because a
    pull request merged.
15. Whether the section 12.1 contract inventory is exact and agrees with
    section 9 item for item, including the exclusion of the three internal Rust
    enums from the public surface.
16. Whether the lifecycle/approval metadata is durable under ADR-0005: that no
    committed field asserts a review, approval or merge state, and that
    repository authority, CTO specification approval and implementation
    authorization are kept distinct (header, sections 2.1, 23 and 24).

## 16. Independent-review remediation

An independent review of the specification returned `CHANGES REQUIRED` with six
findings. All six were addressed on this same branch and pull request; no new
branch, pull request or issue was created, and no runtime file was touched.

| # | Finding | Status | Resolution |
|---:|---|---|---|
| 1 | Lifecycle/approval wording was internally contradictory (`Status: DRAFT` alongside `Approved by: CTO` and `Approved for implementation by: CTO`, a dependency row asserting the specification was "approved", and prose saying CTO approval was still a future step) | RESOLVED | Removed `Status`, `Approved by` and `Approved for implementation by` entirely. Replaced with the durable triple `Specification approval authority: CTO` / `Specification approval evidence: GitHub pull-request record` / `Implementation authorization: separate and absent`. Section 2.1's last two rows became named **gates** rather than statuses. Section 23 became a ten-event table naming where each event's evidence lives, and separating repository authority from CTO approval and from implementation authorization. Section 24 now records approval *authority and effect* only, never whether approval occurred |
| 2 | The lock assessment claimed the migration "takes no lock on any populated table", which is wrong because the foreign key references `publisher` | RESOLVED | Section 13.3 rewritten against the PostgreSQL 17 documentation. `ADD FOREIGN KEY` takes `SHARE ROW EXCLUSIVE` on the referenced table as well as on the constrained table. Added per-operation lock table, a blocked/not-blocked table for concurrent `publisher` operations, a conservative assessment (no parent scan; the dominant risk is lock acquisition and queueing, not duration), an explicit rejection of `NOT VALID` as unhelpful here, and mandatory `pg_locks` verification. Explicitly forbids the old claim and any production-duration claim |
| 3 | The row check constraint permitted states the lifecycle forbids (disabled rows with null `activation_id`/`enabled_at`, enabled rows retaining `disabled_at`), and `enabled DEFAULT false` implied a valid never-activated row | RESOLVED | Tightened to `enabled boolean NOT NULL` (no default), `activation_id uuid NOT NULL`, `enabled_at timestamptz NOT NULL`, `disabled_at timestamptz NULL`, with the constraint reduced to `enabled = (disabled_at IS NULL)`. Added section 6.1.1 with the row-existence rule that justifies the `NOT NULL` columns, the two-state table, the rejected-state list, the per-operation satisfaction table and the Diesel type mapping. Model fields changed from `Option<Uuid>`/`Option<Timestamp>` to `Uuid`/`Timestamp` |
| 4 | N+1 behaviour was underspecified for `Publisher.distributionPlatforms` under list parents | **BLOCKED - escalated** | See section 14. The prohibition is stated bindingly and `thoth-api/AGENTS.md` section 6 is preserved unweakened, but the mechanism is a CTO decision. Added section 9.2.1 (evidence, four options, per-parent-shape behaviour, what is settled regardless), section 19.1 (named blocking gate and exact decision required), section 18.7b (measured query-count evidence) and a new step 3 in the section 22 sequence |
| 5 | Rollout conflated repository merge with environment state ("empty in every environment", queries returning empty "after merge") | RESOLVED | Section 14 split into 14.1 (what a merge actually guarantees - repository only) and 14.2 (environment behaviour, stated conditionally on separately authorized deployment and migration execution). Five events enumerated explicitly. Section 15.2 corrected to scope operational rollback to environments; section 18.11 corrected to stop attributing empty tables to merge |
| 6 | Compatibility claimed "four new enums" while naming three, and "two new root query fields" while specifying three | RESOLVED | Added section 12.1 as the binding inventory: 3 root query fields, 1 new `Publisher` field, 2 object types, 3 GraphQL enums, 0 inputs/mutations/scalars, plus an explicit table of the 3 internal Rust enums that are deliberately **not** GraphQL enums. Section 12.2 now refers to that inventory, generated-SDL expectations were tightened, and section 18.7a asserts the inventory against the generated schema |

Consequential updates made for consistency: acceptance criteria (row invariant,
contract inventory, N+1 gate, corrected lock criterion), required tests
(new 18.3a, 18.7a, 18.7b; extended 18.8), section 20 implementation-report
expectations, section 22 sequence renumbering, the Publisher Services tracker
and the `CHANGELOG.md` entry for PR #788. Pull-request body updated so its
summary and review-focus language match the remediated specification.

A classified search was performed for statements the corrections would falsify
(`Approved by`, `Approved for implementation`, `Status: DRAFT`, `four new
enums`, `two new root`, `no lock`, `every environment`, `enabled DEFAULT
false`, `Option<Uuid>`, `Option<Timestamp>`). Each match was classified
individually; no global find-and-replace was used, and historical content
outside this pull request was not rewritten.
