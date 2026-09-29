# THOTH-ASYNC-01-ADR-01 implementation report

This report records the bounded ADR-0012 authoring task and its first
independent-review correction round. Review/approval state itself remains in
GitHub under issues #957 and #958 rather than being copied as transient
repository status.

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

No other source, migration, GraphQL, workflow or infrastructure path is changed.

## Initial candidate

The first complete proposal was handed to independent review at:

```text
a69921682cd02e060110304f8f9868d0c39da65e
```

It was based directly on:

```text
develop @ 923545d5c9028bc04c40e38efeb7de674efed3fd
```

and contained exactly the four authorized paths.

## Independent review and correction round

The independent CRITICAL review of
`a69921682cd02e060110304f8f9868d0c39da65e` returned
`CHANGES REQUIRED`. Durable control receipt is issue #958 comment
`5896883454`.

All nine blocking findings were accepted and corrected within the existing
four-path documentation budget:

### F-01 - ADR-0008 supersession

The ADR now identifies the exact ADR-0008 statements/items displaced by the new
shared framework, states that ADR-0008 section 3.5 is satisfied rather than
superseded, and preserves the approved durable-work convention list,
`approved != mandatory` rule and machine-role/least-privilege boundaries.

Approval-state reconciliation of ADR-0008 metadata/register is explicitly a
later decision-record action, not runtime implementation.

### F-02 - ADR-0010 relationship

The ADR now explicitly partially supersedes only ADR-0010's assumption that
`distribution_job*` remains the long-term execution source.

It defines:

```text
async_job / async_job_attempt
    canonical execution truth

ServiceOperation
    staff operational/audit seam where ADR-0010 applies

observed state
    separate domain truth

attention / reconciliation
    separate operational projection
```

`ServiceOperation` is not a second mutable attempt ledger. ADR-0010 staff
command, reconciliation and activation gates remain binding.

### F-03 - event route identity

The corrected proposal gives every event durable identity/order and every
consumer a stable `route_key`.

Materialization is database-unique on `(event_id, route_key)` regardless of
job terminal state. Route activation has a durable boundary so later code does
not silently subscribe to historical events; historical processing requires an
explicit backfill/replay.

### F-04 - effectful lease expiry

The proposal now distinguishes effectful jobs and requires durable attempt phases
equivalent to `PRE_WRITE`, `EFFECT_STARTED` and `EFFECT_CONFIRMED`.

Leases are renewable using the current claim token only; renewal cannot revive a
superseded claim.

Lease expiry, crash, cancellation or shutdown at/after `EFFECT_STARTED` enters
`RECONCILIATION_REQUIRED` unless a durable domain checkpoint proves safe
resume. Provider identifiers must be persisted before yielding asynchronous
work, and resumed work polls/reconciles rather than blindly resubmitting.

### F-05 - scheduling, bounded attempts and cancellation

Priority, `available_at`, bounded attempts, distinct WAITING versus
RETRY_SCHEDULED semantics, per-kind concurrency/fairness and cancellation
semantics are now first-class architecture.

Cancellation after a possible external effect does not assert that the effect
did not happen.

### F-06 - released BE-04 contract

The proposal now separates BE-04 storage replacement from released API
compatibility.

The future forward migration removes the three unused `distribution_job*`
tables, dedicated indexes and four queue-only enum types while retaining
`distribution_platform` and reconciling Diesel/domain code.

The released BE-04 GraphQL lifecycle API receives a bounded deprecated
compatibility facade backed by generic jobs. `thoth-dissemination` migrates to
the generic contract first; later API removal is a separate approved deprecation
task.

Fresh production preflight must prove zero rows, disabled legacy creation,
inactive legacy executor and no live legacy contract activity before storage
DROP.

### F-07 - worker AWS credential model

The worker's AWS authority is exclusively its dedicated ECS task role.

Static AWS access keys are prohibited for the worker regardless of deployment
mechanism. The infrastructure task must create a headless service with a
dedicated task role rather than silently inheriting the existing shared-role,
web/ALB-oriented service pattern.

Non-AWS credential values remain deployment environment variables but must be
injected from non-committed deployment configuration.

Existing service static-AWS-credential remediation is identified as separate
security work and is not performed by this task.

### F-08 - operability

The ADR now requires queryable queue/worker signals, per-kind bounded
attempts/concurrency/backoff, starvation protection, poison-job attention,
protected audited operator controls, immutable retained attempt history and
separately governed retention.

Generic retry cannot move `RECONCILIATION_REQUIRED` back to an effectful
attempt without domain proof of safety.

### F-09 - durable documentation hygiene

The ADR-0012 decision-register row is inside the Markdown decision table.

This report no longer carries a transient `Status: ... awaiting review` line.

## Architecture currently recorded

The corrected proposal includes:

- one PostgreSQL-backed cross-programme event/route/job engine;
- transactional event creation where coupled to PostgreSQL business writes;
- durable crash-safe route materialization;
- typed/versioned event and job contracts;
- absolute job idempotency plus separate route idempotency;
- priority, delayed availability, bounded attempts and concurrency/fairness;
- claim-token-fenced renewable leases;
- durable effect checkpoints and reconciliation-required handling;
- append-only sanitized attempt evidence;
- one default headless `thoth-worker` Fargate runtime for Thoth-owned handlers;
- domain-owned external executors such as `thoth-dissemination`;
- one dedicated routine worker AWS task role, with no static AWS worker keys;
- non-AWS credentials supplied as environment variables from non-committed
  deployment configuration;
- no automated legacy-CDN migration;
- explicit ADR-0008 and ADR-0010 partial-supersession boundaries;
- a BE-04 GraphQL compatibility window over generic storage;
- ADR-level observability and protected operator-control requirements.

## Migration/data effect

No migration was created, modified or executed.

The proposal records the CTO-provided environment fact that v1.7.0 has been
applied to production and its `distribution_job*` tables have not been
operationally used, while dev has not applied that migration.

Historical `20260814_v1.7.0` remains immutable.

Any future storage removal requires a separately authorized forward migration
and fresh zero-use production preflight.

## Authorization/security effect

No authorization implementation changed.

ADR-0008's domain-specific machine-role, least-privilege and `SUPERUSER`
separation rules remain in force.

No IAM role, policy, credential, provider configuration or environment variable
was created, rotated or changed.

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

Repository/GitHub mutations remain bounded to the task ledger, existing task
branch and authorized documentation footprint.

## Validation and evidence

Initial branch preflight established:

- `develop` exactly matched authorized base
  `923545d5c9028bc04c40e38efeb7de674efed3fd`;
- no conflicting `feature/async*` ref existed;
- ADR-0012 did not already exist;
- ADR-0011 was already allocated.

The first independent review independently verified the original candidate's
exact four-path footprint and exact head before returning `CHANGES REQUIRED`.

The corrected final branch head is recorded in issue #958 after this report
commit and is the SHA to which the next independent review must bind.

No runtime tests are claimed by this documentation-only task.

## Deviation history

During initial issue creation, the GitHub connector omitted issue numbers from
the normalized create response, causing #958 initially to contain
`#undefined` as its parent reference. The returned issue URLs established
#957/#958 and the ledger was corrected before branch/source mutation.

No source-scope deviation remains.

## Remaining lifecycle gates

The repository files themselves do not assert completion of these gates.

The task still requires, in order:

1. fresh independent CRITICAL exact-head review of the corrected final head;
2. exact-content CTO architecture approval if that review returns APPROVED;
3. approval-state decision-record reconciliation, including ADR-0008/ADR-0010
   supersession metadata;
4. PR creation only under separate authorization;
5. required CI and independent exact-head source/document review;
6. explicit CTO merge authorization for the CRITICAL task;
7. merge into `develop`;
8. separately specified and authorized implementation/migration/deployment tasks.

This report authorizes none of those later actions.
