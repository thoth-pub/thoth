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
Pull request: the specification pull request for this branch
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
record.

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
9. **A single-row `CHECK` constraint enforces the lifecycle invariant**; the
   cross-row OAPEN/DOAB invariant is enforced transactionally in the domain
   layer, with an explicit written justification for not adding a trigger or
   exclusion-constraint subsystem.
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
- Locking and downtime are specified as an assessment of the actual DDL
  operations. No production duration is claimed, and no production database
  access is authorized.

## 14. Unresolved issues

NONE. No unresolved architecture decision remains, and no `TBD` or placeholder
is present in the specification. Architecture escalations: NONE.

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
