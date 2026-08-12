# BE-02-SPEC Implementation Report

Documentation/control-only reconciliation of the existing BE-02 specification
candidate on PR #788. This task implements no BE-02 runtime code and authorizes
nothing beyond specification authoring/reconciliation.

## 1. Repository state

Repository: `thoth-pub/thoth`
Workflow: STANDARD
Base branch: `develop`
Reconciliation base: `8dcf031d76beaf0ad1ac2b6abd3673e37f9a9d55` (merge of PR #802; verified as current `develop` before reconciliation)
PR target: `develop`
Programme integration branch: None
Specification branch: `feature/publisher-services/be-02-spec`
Existing pull request: #788 (DRAFT)
Inherited specification head: `d411d4935a507804f28d8798419d405e32880d02`
Future implementation branch: `feature/publisher-services/be-02` - MUST NOT exist until separate CTO implementation authorization
Risk of future implementation: HIGH
Specification approval authority: CTO
Implementation authorization: NOT GRANTED by this task

The existing PR was 3 commits ahead of its historical base and 89 commits behind
current `develop` at reconciliation preflight. It already owned exactly four
files: the BE-02 specification, this report, Publisher Services task status and
CHANGELOG. The reconciliation deliberately reuses the same branch and PR rather
than creating a competing specification.

The final exact reconciliation head is the live PR #788 head and is intentionally
not embedded in its own containing commit. Exact-head review and lifecycle facts
belong to GitHub under ADR-0005.

## 2. Scope confirmation

Governing controls:

- root `AGENTS.md`;
- `thoth-api/AGENTS.md`;
- engineering operating model, task-spec template, risk classification and
  release gates;
- ADR-0002, ADR-0003, ADR-0004, ADR-0005 and ADR-0007;
- final distribution-platform inventory;
- merged `THOTH-GQL-DATALOADER-01` foundation (PR #802);
- approved Publisher Services design, Drive revision 3.

Objective: reconcile the already-authored BE-02 specification against the
current repository after ADR-0007 superseded ADR-0006 and PR #802 merged the
request-local non-cached DataLoader foundation. Preserve the mature platform,
migration, lifecycle, descriptor, public-API, authorization and rollback design;
replace only the obsolete shared GraphQL N+1 premise/gate and stale preflight
state with the repository-authoritative first-production-DataLoader contract.

Out-of-scope changes: NONE.

Explicitly not done:

- no BE-02 runtime code;
- no migration, SQL, `schema.rs`, model, GraphQL resolver, DataLoader field,
  dependency or test implementation;
- no implementation branch;
- no deployment, release, production migration, backfill or activation;
- no mutation-guard mode change;
- no PR #799 action;
- no issue #765 mutation;
- no ADR/inventory redesign.

## 3. Historical review state and reconciliation reason

The inherited PR #788 had already received independent review with
`CHANGES REQUIRED`. The branch remediated the substantive platform/model review
findings and deliberately retained one named blocker:

```text
BLOCKED - N+1 CONTROL REQUIRES ARCHITECTURE DECISION
```

That blocker was correct at the old base: no repository-approved child-field
batching mechanism existed then. It is no longer current repository truth.

Subsequent authoritative work selected and delivered the mechanism:

1. ADR-0007 approved conventional request-scoped non-cached DataLoader on pinned
   Juniper async execution and superseded ADR-0006;
2. `THOTH-GQL-DATALOADER-01` implemented that foundation, was independently
   reviewed, CTO merge-authorized and merged as PR #802;
3. current `RequestLoaders` is created per GraphQL request;
4. `configured_loader` applies max batch size 200 and yield count 10 explicitly;
5. database loaders use `try_load`, total batches, set-based Diesel behind
   `spawn_blocking`, and safe non-serde error projection;
6. ADR-0007 explicitly identifies BE-02 as an eligible first production
   consumer under a freshly reconciled and approved BE-02 specification.

The reconciliation therefore retires the old options A-D escalation rather than
asking the CTO to choose it again.

## 4. Files changed by the reconciled specification PR

Exactly four documentation/control files:

1. `docs/engineering/ai-delivery/tasks/BE-02.md`
   - rewritten/reconciled against current authority;
   - preserves the 17-value inventory, assignment schema/lifecycle, descriptors,
     GraphQL contract, migration controls and rollout/rollback;
   - replaces the obsolete N+1 blocker with binding ADR-0007 production-consumer
     requirements.
2. `docs/engineering/ai-delivery/implementation-reports/BE-02-SPEC-implementation-report.md`
   - records this reconciliation and evidence boundary.
3. `docs/publisher-services/task-status.md`
   - reconciles BE-02 dependencies to include the satisfied ADR-0007/DataLoader
     foundation and the remaining specification/authorization gates.
4. `CHANGELOG.md`
   - records the documentation/control reconciliation under `Unreleased`.

Runtime files changed: NONE.

## 5. Preserved substantive BE-02 decisions

The reconciliation intentionally preserves these previously developed and
reviewed decisions:

- exactly 17 ADR-0004 `DistributionPlatform` values with no `OTHER`/fallback;
- separate DistributionPlatform and MetricPlatform domains;
- PostgreSQL enum `public.distribution_platform` in canonical order;
- `publisher_distribution_platform` natural/composite primary key
  `(publisher_id, platform)`;
- `enabled`, `activation_id`, `enabled_at` NOT NULL; `enabled` no default;
  `disabled_at` only nullable lifecycle field;
- row invariant `enabled == (disabled_at IS NULL)`;
- application-owned activation UUID and transaction timestamps;
- disabled rows retained and genuine re-enable gets a new activation;
- per-publisher row locking + transactionally atomic transitions;
- OAPEN/DOAB group normalization, shared activation and state repair;
- OCLC_KB and EX_LIBRIS_KB independent destination assignments sharing one
  internal feed profile;
- JISC_NBK included in inventory but inactive/non-assignable/job-free;
- static compile-time-exhaustive code-owned descriptors;
- exactly four additive public GraphQL surfaces;
- public platform assignments but no public package/protected/internal data;
- deterministic reverse pagination with `publisher_id` tie-breaker;
- fail-closed empty/error semantics;
- additive migration with zero assignment rows;
- empirical PostgreSQL lock evidence requirement;
- merge/deploy/migrate/backfill/cutover as separate events;
- retained-foundation operational rollback/forward repair.

## 6. New ADR-0007 first-production-consumer contract

The reconciled specification now binds BE-02 to the already-merged shared
foundation rather than creating a local architecture:

- add a typed field-specific loader to `RequestLoaders`;
- non-cached, request-local ownership only;
- use existing `configured_loader` or visibly equivalent 200/10 construction;
- key `Publisher.distributionPlatforms` by `publisher_id`;
- loader-first resolver entry before unrelated awaited work;
- DataLoader `try_load` only;
- total batch result over every requested key, including successful empty list
  for a publisher with zero enabled assignments;
- set-based `publisher_id = ANY(keys)`/equivalent enabled-row SQL;
- synchronous Diesel entirely inside `tokio::task::spawn_blocking`, connection
  acquired/used/dropped inside closure;
- no per-key SQL loop, fallback or retry;
- conventional GraphQL error semantics for the new field unless exact-base
  discovery proves an approved different field-family convention;
- safe non-serde batch-error projection;
- no mutation-guard coupling and no ADR-0006 machinery.

Field-specific evidence is strengthened to require actual SQL observation,
including the reference case:

```text
250 publishers -> [200, 50] target chunks -> exactly two set-based assignment SQL statements
```

for both the existing `publishers` parent list and the new
`publishersByDistributionPlatform` parent list. The spec also requires
field-specific batching boundaries, request-local/non-cached behavior and
backend-failure equivalence rather than relying solely on the generic foundation
tests.

## 7. Database and migration effects

Migration added by this specification task: NO.

The future BE-02 implementation requires an additive HIGH-risk migration under
ADR-0003 Architecture A, with empty/populated DB evidence and empirical lock
verification. No migration command was run against any production/shared DB in
this specification task.

## 8. API and compatibility effects

GraphQL/API change in this specification task: NONE.

The future implementation contract remains exactly additive:

- 3 new root query fields;
- 1 new Publisher child field;
- 2 new object types;
- 3 new GraphQL enums;
- 0 new inputs;
- 0 mutations;
- 0 scalars/interfaces;
- 3 internal descriptor enums explicitly absent from SDL.

The reconciled spec correctly changes the future SDL acceptance condition from
"byte-identical" (appropriate to the DataLoader foundation) to **exact additive
inventory agreement** (appropriate to BE-02).

## 9. Authorization and security

Authorization paths changed by this specification task: NONE.

Future BE-02 read surfaces remain intentionally public. The spec requires
positive anonymous-access tests and negative package/capability/protected/internal
exposure tests. DataLoader is explicitly not an authorization layer.

No credential, secret-bearing deployment source, production data or private
runtime configuration was accessed or recorded.

## 10. Validation for this specification task

### 10.1 Locally executed checks

Required minimum docs/control validation, run on the remediated working tree:

```text
Command:
git diff --check

Result:
PASS - no whitespace or conflict-marker errors reported.
```

Manual verification (locally performed) confirmed:

- all relative links and canonical names resolve;
- no unresolved obsolete N+1 options/gate remains;
- no stale claim that the repository lacks DataLoader/request-scoped Context
  state;
- no contradictory OAPEN/DOAB linked-normalization language remains (see the
  review-finding remediation in section 13);
- no runtime/Cargo/schema/migration/workflow files in the final PR diff;
- `CHANGELOG.md` present under `Unreleased`;
- exactly one BE-02 specification PR/branch is being used;
- no future implementation branch created;
- current `develop` remains reconciled into the branch without discarding
  intervening repository changes.

### 10.2 GitHub exact-head CI

Under ADR-0005 the live pull-request CI is the authoritative record and run
identifiers belong to GitHub rather than this committed file. At the remediated
exact head the change is classified documentation-only, so the exact-head
workflows report:

- `check-changelog`: PASS (documentation changelog gate);
- `classify`: PASS (docs-only classifier);
- `build`: SKIPPED by docs-only classifier;
- `test`: SKIPPED by docs-only classifier;
- `run_migrations`: SKIPPED by docs-only classifier;
- `format_check`: SKIPPED by docs-only classifier;
- `lint`: SKIPPED by docs-only classifier;
- `build_and_push_staging_docker_image`: SKIPPED by docs-only classifier.

SKIPPED runtime jobs are **not** recorded as PASS. The overall CI concludes
successfully because the classifier correctly skips runtime jobs for a
documentation-only change; that success is not evidence that the Rust build,
tests or migrations executed. No workflow was manually dispatched to manufacture
evidence.

## 11. Rollout and rollback

This specification task has no runtime rollout.

Rollback of an unmerged spec candidate is ordinary branch/PR revision. After
specification merge, material corrections require another bounded reviewed
change; no approval-state-only closeout PR is required under ADR-0005.

The future implementation's rollout/rollback is fully specified in BE-02.md and
remains separately authorized.

## 12. Known limitations and deferred work

- BE-02 implementation is not authorized by this reconciliation.
- The final implementation base is intentionally unknown until fresh preflight.
- BE-03, BE-04, MIG-01, dissemination, app, OAI and production operations remain
  separate tasks/gates.
- PR #799 remains blocked and outside BE-02.
- OBSERVE and ENFORCE remain not authorized.

## 13. Review focus

Fresh independent exact-head specification review should focus on:

1. whether every previously settled BE-02 product/persistence decision survived
   reconciliation;
2. exact consistency with ADR-0004's 17-value inventory and descriptor rules;
3. exact consistency with ADR-0007 and current `RequestLoaders`/200/10/
   `try_load`/`spawn_blocking` foundation;
4. whether the old N+1 blocker is fully retired without weakening the standing
   N+1 control;
5. whether 250 -> [200,50] -> 2 actual SQL statements is a sufficient and
   correctly scoped first-consumer acceptance reference;
6. migration locks, lifecycle invariants, and the clarified OAPEN/DOAB linked
   normalization -- a linked enable no-ops only when the pair is already
   *normalized fully enabled* (both rows present, enabled, same `activation_id`,
   same `enabled_at`) and otherwise atomically repairs split-activation or
   otherwise non-normalized enabled pairs -- JISC fail-closed behavior and
   deterministic pagination;
7. public-v-protected API boundary and exact additive SDL inventory;
8. authorization boundaries: spec approval != implementation authorization !=
   merge authorization != production authorization.

## 14. Agent self-assessment

This authoring/reconciliation agent does not approve its own work.

```text
Specification candidate: ready only after exact-head CI/docs validation
Independent specification approval: NOT PROVIDED BY AUTHORING AGENT
CTO specification approval: NOT INFERRED
BE-02 implementation authorization: NOT GRANTED
BE-02 implementation branch: MUST NOT BE CREATED YET
CTO implementation merge authorization: future separate decision
Production authorization: NOT GRANTED
OBSERVE: NOT AUTHORIZED
ENFORCE: NOT AUTHORIZED
PR #799: untouched
```
