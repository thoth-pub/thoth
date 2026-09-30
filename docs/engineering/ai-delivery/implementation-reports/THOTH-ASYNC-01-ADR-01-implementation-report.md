# THOTH-ASYNC-01-ADR-01 implementation report

This report records the bounded ADR-0012 architecture-authoring task and its
independent-review correction rounds. Fast-changing review/approval state remains
in GitHub issues #957/#958 rather than being duplicated as repository status.

## Identity

```text
Programme: THOTH-ASYNC-01
Owning issue: thoth-pub/thoth#958
Parent programme: thoth-pub/thoth#957
Repository: thoth-pub/thoth
Task: THOTH-ASYNC-01-ADR-01
Risk: CRITICAL
Workflow: STANDARD
Authorized base: develop @ 923545d5c9028bc04c40e38efeb7de674efed3fd
Task branch: feature/async/adr-0012
PR target: develop
```

## Authorized write footprint

Exactly four repository paths are used:

1. `docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md`;
2. `docs/engineering/decisions/decision-register.md`;
3. `CHANGELOG.md`;
4. this implementation report.

No runtime source, migration, GraphQL implementation, workflow or
infrastructure path is changed.

## Initial candidate and review round 1

Initial candidate:

```text
a69921682cd02e060110304f8f9868d0c39da65e
```

The independent CRITICAL review returned `CHANGES REQUIRED`; durable receipt:
#958 comment `5896883454`.

F-01 through F-09 were accepted. Round-2 candidate:

```text
9d329e063b2731b691c62a412555b470f2a9fe39
```

The round-1 corrections established precise ADR-0008/ADR-0010 boundaries,
crash-safe route identity, effect-phase checkpoints, bounded scheduling,
released-contract handling, worker task-role-only AWS authority, operability and
documentation hygiene.

## Independent review round 2

The independent CRITICAL review of
`9d329e063b2731b691c62a412555b470f2a9fe39` again returned
`CHANGES REQUIRED`; durable receipt: #958 comment `5909933000`.

Seven blockers were accepted:

- R2-01 - incomplete WAITING/resume/attempt/concurrency/reconciliation lifecycle;
- R2-02 - resource-ambiguous idempotency keys and coalescing contradiction;
- R2-03 - event-routing completeness vulnerable to commit-order and mixed-version
  worker assumptions;
- R2-04 - incomplete BE-04 consumer/API coverage, including `thoth-app`;
- R2-05 - unsafe same-release BE-04 table DROP under the actual rolling
  migrate-on-start deployment model;
- R2-06 - untrusted publisher-file parsing sharing high Hosting authority;
- R2-07 - unbounded immutable payload snapshots/personal-data duplication.

## Round-3 corrections

The final tightening also explicitly qualifies ADR-0010 section 4.4's statement
that ADR-0008 remains fully binding: `ServiceOperation` remains non-queue audit
infrastructure, while ADR-0012 supplies the later explicit shared-framework
exception ADR-0008 anticipated.

### Job lifecycle

The ADR now requires:

- WAITING to atomically persist checkpoint state and release claim/lease;
- WAITING/poll attempts not to consume the pre-write retry budget, instead being
  bounded by the waiting deadline/poll policy;
- resume from WAITING as a new claim/attempt;
- separate bounded pre-write retry budget and post-write waiting/deadline;
- post-write deadline exhaustion to become `RECONCILIATION_REQUIRED`, not
  `FAILED`;
- WAITING and RECONCILIATION_REQUIRED to retain their concurrency key by
  default;
- effectful work to start only when its remaining lease/runtime can cover the
  declared effect window;
- reconciliation itself to be a claimed, claim-token-fenced, kind-authorized
  attempt;
- durable `EFFECT_CONFIRMED` evidence to permit non-duplicating recovery where
  a kind defines it.

### Effect-scoped idempotency and coalescing

Idempotency now identifies a specific intended effect, not merely a resource.

Keys are derived from the event/route, source revision/fingerprint, schedule
slot, command/request id or explicit coalescing window as appropriate.

Each `(event_id, route_key)` has its own durable route record. Several route
records may target one not-yet-started job only under an explicit coalescing
rule. Idempotency conflicts never silently mark a route complete.

### Routing completeness

Route registrations and activation boundaries are durable PostgreSQL state.
Activation uses a durable routing generation/epoch or equivalently strong
serialized database mechanism captured transactionally by events/routes, so
out-of-order commits cannot create an ambiguous boundary.

Routing completeness is determined per eligible route, not from worker-local
handler knowledge or an unproved sequence high-water mark. Mixed worker
versions cannot silently complete an unknown active route, and backfill uses the
same route materialization records.

### Payload minimization

Event/job payloads default to canonical IDs, source revision/fingerprint and
minimal command/routing parameters.

Credentials are prohibited. Personal data or immutable domain snapshots require
kind-specific justification and retention/erasure handling. Current-state work
reads canonical state at execution time.

### BE-04 expand-migrate-contract transition

The previous state-translating compatibility-facade design has been removed.

The architecture now requires:

1. **Expand** - Release N adds the generic schema/API while retaining legacy
   BE-04 schema/API and keeping legacy automatic job creation inactive.
2. **Migrate consumers** - independently migrate both known released consumers:
   `thoth-dissemination` and `thoth-app`.
3. **Retire legacy API** - later remove legacy lifecycle/read/filter/toggle
   GraphQL contract after all consumers are verified migrated.
4. **Contract storage** - only in a still-later migration, after old code is no
   longer a supported rollback target; the migration itself asserts all three
   legacy tables contain zero rows and aborts otherwise.

Known `thoth-app` dependencies are explicitly recorded:
`latestBackCatalogueJob`, `jobStatuses`,
`withoutBackCatalogueJob` and generated `DistributionJobStatus` contract.

Historical `20260814_v1.7.0` remains immutable.
`distribution_platform` remains.

### Deployment and rollback

The ADR now incorporates the authoritative current deployment property that the
GraphQL image defaults to `thoth init`, which runs migrations before serving,
while production uses multiple rolling ECS tasks.

Therefore destructive contraction cannot occur in the same release that moves
the API. The previous binary/schema remains rollback-compatible throughout
expand/migrate/API-retirement phases; after generic data exists, runtime rollback
must preserve that data and use forward recovery rather than deleting/reverting
it.

### Trust-separated worker pools

The default trusted worker still uses one routine task role containing the union
of approved routine trusted capabilities.

Handlers that parse publisher-controlled/untrusted content are a distinct
`UNTRUSTED_CONTENT` risk class and run in a separate task/service/process
boundary with minimal authority and no Hosting DNS/ACM/CloudFront
tenant-management permissions unless explicit CTO risk acceptance records the
blast radius. That pool also does not inherit the primary application database
credential: it must use the kind-scoped executor API or a separately reviewed
minimal database principal.

This is a real trust-boundary exception, not one IAM role per ordinary handler.

### Validation additions

The ADR now requires tests for:

- mixed worker versions;
- out-of-order commits;
- route completeness/backfill;
- effect-scoped idempotency and coalescing;
- WAITING claim release/resume;
- pre-write versus post-write exhaustion;
- concurrency-key retention;
- claimed reconciliation races;
- payload minimization;
- untrusted-worker IAM isolation;
- both downstream consumer migrations;
- rolling deploy/rollback safety;
- contraction abort on any legacy row.

## Migration/data effect

No migration was created, modified or executed.

Production's v1.7.0 legacy schema fact remains an architecture input. The ADR no
longer proposes immediate replacement/drop: generic schema expansion precedes
consumer migration/API retirement, and legacy storage contraction is a later
separately authorized migration.

## Authorization/security effect

No authorization implementation changed.

No IAM role/policy, credential, provider configuration or environment variable
was created, rotated or changed.

ADR-0008 domain-specific application authorization remains binding. Worker-pool
IAM authority and ZITADEL/application authorization remain separate concepts.

## External/runtime effects

```text
provider reads: 0
provider writes: 0
migration execution: 0
CI dispatch/rerun: 0
deployment: 0
release/publication: 0
production activation: 0
```

## Validation performed for this documentation task

- exact authorized base was preserved;
- all candidate commits remain on `feature/async/adr-0012`;
- cumulative diff is verified against the authorized base after each correction
  round;
- ADR remains `PROPOSED`;
- decision-register row remains inside the Markdown table;
- no PR has been created;
- no runtime test result is claimed by this documentation-only task.

The final round-3 head is recorded in #958 after this report commit. Any further
independent review is valid only against that exact SHA.

## Deviation history

During initial issue creation, the GitHub connector omitted the issue number in
its normalized create response, briefly producing a `#undefined` parent
reference in #958. The returned URLs established #957/#958 and the issue was
corrected before branch/source mutation.

No source-scope deviation remains.

## Remaining lifecycle gates

1. fresh independent CRITICAL exact-head review of the round-3 candidate;
2. exact-content CTO architecture approval only if that review returns
   `APPROVED`;
3. separate approval-state decision-record reconciliation, including ADR-0008
   and ADR-0010 metadata, under its own authorized write budget;
4. PR creation only under separate authorization;
5. required CI/document checks and independent exact-head review;
6. explicit CTO merge authorization;
7. merge to `develop`;
8. separately specified and authorized implementation, migration, downstream
   consumer, infrastructure, deployment and activation tasks.

This report authorizes none of those later actions.
