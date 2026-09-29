# ADR-0012 - Shared asynchronous event and job execution architecture

Status: PROPOSED
Date: 2026-09-29
Decision owner: CTO
Programmes affected: Shared Backend Architecture (owning programme); Publisher Services and Distribution Configuration; Thoth Hosting; future Thoth programmes requiring asynchronous work
Repositories affected: `thoth-pub/thoth` (shared durable engine and default worker runtime); `thoth-pub/thoth-dissemination` (domain executor/consumer); `thoth-pub/infrastructure` (worker runtime and IAM substrate)
Parent programme: [THOTH-ASYNC-01 #957](https://github.com/thoth-pub/thoth/issues/957)
Authoring task: [THOTH-ASYNC-01-ADR-01 #958](https://github.com/thoth-pub/thoth/issues/958)
Superseded by: None
Supersedes in part:
- ADR-0008 only where it denies a reusable cross-programme job framework/API: the final shared-framework prohibition in section 3.3, section 3.4's programme-local-only ownership rule, section 5.1 item 6, section 5.2 item 6 and rejected alternative D. ADR-0008 section 3.5 is **satisfied by this ADR, not superseded**. Section 3.3's approved convention list, its `approved primitive != mandatory mechanism` rule, and all machine-role, least-privilege and `SUPERUSER` separation rules remain binding.
- ADR-0010 only where invariant 15 and section 7.2 assume `distribution_job*` remains the long-term Publisher Services execution source. ADR-0010's Staff Operations Console, `ServiceOperation` audit seam, desired/execution/observed-state separation, attention/reconciliation model and staff-command gates remain binding as specified in section 1.3 below.

Decision: Thoth establishes one shared PostgreSQL-backed asynchronous event and
job engine for cross-programme durable work. Events record durable facts and may
fan out idempotently into jobs; jobs record executable asynchronous work and may
also be created directly for scheduled, reconciliation or operator-requested
work. The engine provides one shared lifecycle, claim/lease/idempotency model and
attempt history. A headless `thoth-worker` ECS/Fargate service is the default
always-on executor for handlers owned by `thoth`. Domain-specific executors in
other repositories may consume the same shared job protocol without moving their
business logic into `thoth`.

This ADR is architecture only. It authorizes no schema migration, source
implementation, worker deployment, IAM change, provider write, credential
provisioning, BE-04/DIS-02 activation, Hosting implementation or production
activation.

Verification baselines at programme setup:

```text
thoth-pub/thoth
develop @ 923545d5c9028bc04c40e38efeb7de674efed3fd

thoth-pub/thoth-dissemination
main @ 0be4dfba0661816914a7fffa477fe33043a99793

thoth-pub/infrastructure
master @ 52c79cf19392df4c9989e5c68ad199c8bc2846a8
```

These SHAs are architecture-authoring evidence only. Every implementation task
must bind its own fresh exact base.

---

## 1. Context

Thoth now has several workloads that are naturally asynchronous:

- Hosting provisioning and reconciliation;
- Publisher Services dissemination such as Crossref deposits;
- validation of uploaded files;
- EPUB post-processing and reader deployment;
- CDN invalidation;
- website/cache revalidation;
- search-index updates such as Algolia;
- future external integrations and scheduled reconciliation.

BE-04 introduced the first substantial durable-job implementation in `thoth`
using `distribution_job`, `distribution_job_target` and
`distribution_job_attempt`. ADR-0008 deliberately made those tables
Publisher-Services-local and required a later explicit cross-programme ADR before
a generic queue/job abstraction could exist.

That later decision is now required. Deploying a Hosting-specific worker and
retaining a separate distribution-job engine would create parallel durable
execution architectures immediately before multiple further asynchronous
consumers are expected.

### 1.1 Current BE-04 deployment fact

Migration `thoth-api/migrations/20260814_v1.7.0` has been applied to
production and therefore the `distribution_job*` schema is part of production
migration history. The tables have not been operationally used. The dev
environment has not applied that BE-04 migration.

Consequences:

- the historical v1.7.0 migration is immutable and must not be rewritten;
- a later separately authorized migration must remove the unused
  `distribution_job*` schema and introduce the approved shared async schema;
- no data-conversion requirement is currently known because the production
  tables have not been used, but the implementation task must verify emptiness
  immediately before any destructive migration is authorized;
- BE-04/DIS-02 deployment and activation remain on HOLD pending this
  supersession.

### 1.2 Existing dissemination boundary

`thoth-dissemination` owns platform-specific dissemination behaviour. Its
current DIS-02A worker claims Thoth jobs and executes Crossref through Python
domain code and an isolated runner. That ownership does not move into
`thoth` merely because job durability becomes shared.

The shared engine owns durable execution mechanics. The owning domain repository
continues to own provider-specific semantics.

### 1.3 Relationship to ADR-0008 and ADR-0010

ADR-0012 is the explicit later cross-programme decision that ADR-0008 section
3.5 required before a reusable generic job/queue abstraction could be introduced.

ADR-0008's seven approved durable-work conventions remain valid engineering
primitives. This ADR selects a shared implementation that uses several of those
primitives because the cross-programme workload now justifies them; it does not
turn every ADR-0008 primitive into a mandatory mechanism for unrelated work.

ADR-0010 remains the authority for the Staff Operations Console and the
cross-domain operational/audit seam. The relationship is binding:

```text
async_job / async_job_attempt
    canonical execution and attempt truth

ServiceOperation
    canonical staff operational/audit seam where ADR-0010 applies

domain observed state
    separately established external/service truth

attention / reconciliation projection
    whether human or reconciler action is required
```

`ServiceOperation` must project from or correlate to the canonical async
execution record; it must not create a second independently mutable attempt
history. Not every internal async job is automatically a Staff Operations
record, but any job important enough for staff to monitor, explain, retry,
replay, reconcile or report follows ADR-0010's canonical operational seam.

ADR-0010 invariants 14, 16, 22 and 23 and sections 4.6 and 4.11-4.13 remain
binding. In particular, retry, replay and current-state redistribution remain
different actions; machine execution authority remains separate from staff
authority; initial reconciliation remains report-only until separately
activated; and staff external-write controls require their own production gate.

A `RECONCILIATION_REQUIRED` async job may leave that state only through a
domain reconciler or protected staff action that can establish a safe next
state. The generic engine must never convert it into an ordinary retry merely
because time passed. Staff actions use ADR-0010's protected, audited command
seam with actor and reason. A reconciler may report evidence before write
activation, but may not silently repair external state unless its owning
specification and production activation authorize that action.

When ADR-0012 reaches approval-state reconciliation, ADR-0008 and ADR-0010
metadata/register entries must be updated to record these exact partial
supersessions. That is decision-record reconciliation, not runtime
implementation.

---

## 2. Decision drivers

1. One robust queue/state-machine implementation should serve all future Thoth
   asynchronous work rather than each programme rebuilding leases, retries and
   audit independently.
2. Business writes and the events they cause must not suffer a dual-write gap.
3. External writes are not generally exactly-once; ambiguous outcomes require
   reconciliation rather than blind replay.
4. Worker crashes, deploys and horizontal scaling must not lose or duplicate
   authoritative job state.
5. One always-on Fargate worker should provide broad value rather than spend most
   of its lifetime waiting for one Hosting-specific workload.
6. Shared execution mechanics must not erase domain/repository ownership.
7. A generic runtime must not become a generic all-powerful application role.
8. Operational complexity should remain proportionate: PostgreSQL is already the
   canonical durable store and is sufficient for expected Thoth volumes.
9. Credentials and provider authority should remain simple to operate while
   preventing unnecessary standing high-risk permissions.
10. Existing production migration history must remain truthful and immutable.

---

## 3. Decision

### 3.1 One shared durable engine

`thoth` owns the canonical asynchronous persistence model. The implementation
will introduce shared durable concepts equivalent to:

```text
async_event
async_job
async_job_attempt
```

The final table/column names are implementation-specification details, but the
semantic separation is architectural.

PostgreSQL is the authoritative owner of:

- event records;
- job state;
- routing/materialization state;
- leases and claim tokens;
- attempt history;
- retry availability;
- cancellation/terminal outcomes;
- reconciliation-required outcomes;
- correlation/causation metadata.

No local file, worker memory, GitHub Actions run, S3 object, message-bus message
or external provider is the sole durable owner of that state.

### 3.2 Events, routes and jobs are different

An **event** records a durable fact that has happened, for example
`WORK_UPDATED`, `FILE_UPLOADED` or `HOSTING_ENABLED`.

A **job** records executable work, for example `ALGOLIA_INDEX_WORK`,
`VALIDATE_FILE`, `CROSSREF_SYNC` or `HOSTING_PROVISION`.

One event may materialize zero, one or many jobs. Jobs may also be created
without an event for scheduled work, reconciliation, backfills or explicit
operator actions.

Where an event is caused by a PostgreSQL business mutation, the business change
and event record are written in the same transaction. Either both commit or
neither commits.

Every event has a unique durable `event_id` and a monotonically ordered durable
event sequence suitable for route activation boundaries. Event payload and
causation metadata are immutable after commit.

Every event consumer has a stable `route_key`. Routing/materialization records
are unique on `(event_id, route_key)` regardless of whether the resulting job
is pending, running or terminal. A routing crash after some consumers have been
materialized therefore resumes from durable route records and cannot create a
second job for an already-materialized route merely because the first job later
completed.

A route has a durable activation boundary. A route introduced by a later
software release receives only events at or after that boundary. Historical
events are processed by that new route only through an explicitly authorized
backfill/replay job; adding code must never silently subscribe the new route to
all historical events.

The implementation may represent route completion per consumer or by an
equivalent snapshotted eligible-route set, but it must make partial fan-out
restartable and database-enforced. Event routing correctness must not depend on
in-memory knowledge of which handlers happened to run before a crash.

### 3.3 Versioned contracts

Every event and job kind has:

- a stable kind identifier;
- an explicit payload schema version;
- deterministic validation;
- an owning domain;
- a defined compatibility policy.

Workers must reject or HOLD unsupported payload versions rather than guessing.
Changes that require old in-flight jobs to remain executable must preserve the
required compatibility window or provide an explicit migration/reconciliation
plan.

### 3.4 Job lifecycle, scheduling and claims

The common job lifecycle must support at least the semantic states:

```text
PENDING
RUNNING
WAITING
RETRY_SCHEDULED
SUCCEEDED
FAILED
CANCELLED
RECONCILIATION_REQUIRED
```

Exact enum spelling is implementation detail, but `WAITING` and
`RETRY_SCHEDULED` are semantically distinct: waiting means the job has
checkpointed and is waiting on an external/domain condition; retry-scheduled
means a failed attempt is eligible for another execution after `available_at`.

Every job carries first-class scheduling attributes including `priority`,
`available_at`, attempt count/max attempts and the owning kind. Ready work is
ordered by priority and availability within the scheduler's fairness/concurrency
rules; priority must not permit one job kind to starve all other enabled kinds.

Each job kind declares a bounded maximum attempt count. Exhaustion is terminal
`FAILED` and creates an attention condition where ADR-0010 applies. The full
legal transition table, including which transitions require a current claim
token, must be fixed by the shared-engine implementation specification and
tested as a database/domain invariant.

Claims use database-enforced concurrency with leases, unforgeable claim tokens
and `FOR UPDATE SKIP LOCKED`. The shared workload explicitly requires multiple
consumers to claim independent ready jobs without blocking each other.

Leases are renewable by heartbeat using the **current** claim token. Renewal has
a bounded maximum lease/attempt runtime, may extend only an unexpired current
claim, and can never revive or extend a superseded claim. A stale worker cannot
complete, fail, checkpoint, cancel or renew a job after its claim has been
superseded.

Every job kind declares whether execution is **read-only/non-effectful** or
**externally effectful**. Every attempt durably records an execution phase at
least equivalent to:

```text
PRE_WRITE
EFFECT_STARTED
EFFECT_CONFIRMED
```

A lease expiry while an attempt is provably `PRE_WRITE` may return the job to
normal retry when its attempt budget permits. A lease expiry for an effectful
attempt at or after `EFFECT_STARTED` transitions to
`RECONCILIATION_REQUIRED`, never directly to `PENDING` or
`RETRY_SCHEDULED`, unless a domain-specific durable checkpoint proves that
resumption cannot repeat the external effect.

Cancellation invalidates the current claim. Cancelling a pending/pre-write job
may produce terminal `CANCELLED`. Cancelling a running effectful job at or
after `EFFECT_STARTED` cannot assert that no external effect happened and
therefore produces `RECONCILIATION_REQUIRED` unless the provider/domain can
prove a stronger safe terminal outcome.

The engine must support horizontal scaling from one worker task to multiple
tasks without changing these semantics.

### 3.5 Idempotency and concurrency

Every logical job has a deterministic idempotency key. Database uniqueness of
that key is **absolute across job states**, not merely "while live": a completed
job does not free its logical effect identity for accidental re-materialization.

Retries create new attempts on the same job. An intentional replay/current-state
run creates a new logical job with a new idempotency key and an explicit
parent/replay/correlation relationship, following ADR-0010's distinction between
retry, replay and current-state redistribution.

Event-derived jobs additionally carry the unique `(event_id, route_key)`
materialization identity from section 3.2. Handler idempotency and route
idempotency solve different problems and neither substitutes for the other.

Handlers may define a concurrency key, for example:

```text
work:<uuid>:algolia
hosting:<target-id>
crossref:<doi>
```

Jobs with unrelated concurrency keys may execute in parallel. Jobs that would
mutate the same protected external/domain resource serialize where the owning
specification requires it.

The shared scheduler supports per-kind concurrency limits. Provider-specific
rate limits remain owned by handlers, but a single failing or rate-limited job
kind must not consume all worker capacity indefinitely.

### 3.6 Delivery semantics, checkpoints and ambiguous writes

The engine guarantees durable at-least-once execution opportunities. It does
**not** claim exactly-once external effects.

A handler result distinguishes at least:

- success;
- retryable failure proven to occur before a relevant external write;
- deterministic permanent failure;
- waiting/checkpointed external work that must be revisited at or after a
  specified time;
- indeterminate/ambiguous external-write outcome requiring reconciliation.

Before an effectful handler crosses the boundary at which an external write may
have begun, it durably records `EFFECT_STARTED` under its current claim token.
Where the provider returns a submission/correlation identifier, the handler
persists that identifier before yielding `WAITING`.

A resumed asynchronous-provider job with a durable provider identifier polls or
reconciles the existing provider operation. It does not submit a fresh write
unless the domain contract has established that no previous effect can exist.

An indeterminate provider write is never blindly replayed merely because the
worker did not receive an acknowledgement. Lease expiry, process crash or
shutdown after `EFFECT_STARTED` therefore fences the job into
`RECONCILIATION_REQUIRED` unless a durable domain checkpoint proves safe
resumption without duplicate effect.

The owning handler/reconciler must establish provider state or require protected
operator attention before another effectful attempt.

### 3.7 Attempts and diagnostics

Each execution attempt is append-only durable evidence containing enough
sanitized information to reconstruct:

- job and attempt identity;
- claim identity;
- worker/executor identity;
- start/end times;
- result classification;
- bounded error code/detail;
- provider request/correlation identifiers where safe;
- retry/reconciliation disposition.

Secrets, credentials and unbounded provider responses must not be written to
attempt records or ordinary logs.

### 3.8 Default `thoth-worker` runtime

The default always-on executor is a headless ECS/Fargate service using the
ordinary Thoth release image with a dedicated command, conceptually:

```text
thoth start worker
```

Initial runtime shape:

```text
same ECS cluster
same VPC/database
no ALB
no public hostname
DesiredCount = 1 initially
separate task definition/service
dedicated worker task role
```

The existing generic infrastructure `service.yml` is web-service/ALB-oriented
and must not be reused unchanged for the headless worker. The infrastructure
implementation must either provide a bounded headless worker template or
explicitly generalize load-balancer attachment under its own reviewed task.

The worker polls/claims durable PostgreSQL jobs. PostgreSQL
`LISTEN/NOTIFY` may be used as a wake-up optimization, but correctness must
never depend on receiving a notification; periodic durable polling remains the
fallback.

Graceful shutdown stops new claims first. A pre-write attempt may report a
retryable pre-write outcome or release/yield according to the shared transition
contract. An effectful attempt after `EFFECT_STARTED` must persist a safe
checkpoint and yield `WAITING`, or enter `RECONCILIATION_REQUIRED`; shutdown
must never intentionally abandon it to ordinary lease-expiry retry.

The runtime must prove clean recovery after abrupt termination as well as
graceful shutdown.

### 3.9 Handler registry and capabilities

The worker has a typed registry of supported job kinds. Each handler declares:

- job kind and supported payload version(s);
- owning domain;
- required runtime capability/configuration;
- timeout/lease requirements;
- retry/reconciliation policy;
- idempotency/concurrency rules.

A worker only claims job kinds whose required capabilities are configured.
Capability declarations are runtime/configuration boundaries and future
scheduling primitives; they do not imply one IAM role per handler.

### 3.10 AWS credentials and task role

The initial `thoth-worker` uses one dedicated least-privilege ECS task role
containing the union of its **approved routine AWS capabilities**.

AWS authority for the worker comes **exclusively** from that task role. The
worker must not receive static `AWS_ACCESS_KEY_ID`,
`AWS_SECRET_ACCESS_KEY`, session-token credentials or equivalent long-lived
AWS credentials through environment variables, committed configuration or any
other deployment mechanism.

Do not introduce one assumable IAM role per ordinary handler merely to mirror
code boundaries when the same process could assume all of them anyway.

The task role may eventually include bounded capabilities such as:

- approved Hosting tenant lifecycle;
- ACM operations needed by Hosting;
- writes to Thoth-controlled Hosting DNS;
- approved CDN invalidation;
- bounded S3 object access required by file-processing handlers.

The exact permission matrix remains an implementation/IAM task and must retain
negative tests and resource scoping.

Standing permissions that materially increase blast radius and are not required
for routine operation remain excluded.

The infrastructure implementation therefore intentionally departs from the
current shared `ECSTaskRole`/static-key pattern for existing Thoth web
services: it must create a dedicated worker task role and ensure no static AWS
keys are injected into the worker. Remediation/rotation of any pre-existing
static AWS credentials used by other services is separate security work and is
not authorized or performed by this ADR.

### 3.11 No automated legacy-CDN migration

The shared worker does not implement legacy CDN migration.

An existing publisher moving to the new Hosting model is provisioned as a new
Hosting target using the ordinary greenfield Hosting lifecycle. Content
migration and any legacy domain/file cutover are operator procedures under their
own runbook and authorization.

Therefore the worker architecture does not require:

- standing `cloudfront:UpdateDistribution`;
- `cloudfront:UpdateDomainAssociation`;
- a special Hosting MigrationRole;
- automated migration rollback state.

The R1C migration evidence remains useful operational evidence, but migration is
not a worker job family under this ADR.

### 3.12 External-service credentials

Non-AWS integration credentials such as Crossref or Algolia credentials are
provided to the relevant runtime through deployment environment variables.

This ADR does not require Secrets Manager.

Requirements:

- credential **values** are never committed to source, infrastructure templates,
  task specifications or GitHub records;
- deployment injects environment values from non-committed environment/provider
  configuration or deploy-time parameters; the infrastructure task chooses the
  concrete mechanism without changing the env-var application contract;
- application logs and durable job diagnostics must never dump the process
  environment;
- handlers fail closed when required credentials are absent;
- providers use distinct credentials where operationally available;
- rotation is performed by updating deployment configuration and replacing the
  affected task/runtime.

Environment variables are an operational delivery mechanism, not a security
isolation boundary between handlers in the same process.

This section does not apply to AWS credentials for `thoth-worker`; section
3.10 requires ECS task-role credentials exclusively.

### 3.13 Domain ownership and cross-repository executors

The shared engine standardizes **durability and execution protocol**, not every
domain implementation.

Handlers whose business logic is owned by `thoth` execute in
`thoth-worker`.

Where another repository owns the domain implementation, that repository may
consume the same shared job protocol using a domain-specific executor identity.
In particular, `thoth-dissemination` continues to own dissemination/provider
logic and may claim only dissemination job kinds allowed by its own authorization
matrix.

This does not create a second canonical queue. A downstream domain executor may
be persistent, scheduled or on-demand as its own approved implementation
specifies, but all authoritative job lifecycle state remains in `thoth`.

For external/domain executors the shared claim/report protocol is a protected,
versioned, kind-scoped Thoth API contract. Its implementation may be GraphQL in
the current architecture, but downstream repositories must pin the exact merged
contract and must not guess an unmerged schema.

The protocol must allow a domain executor to:

- claim only explicitly authorized job kinds;
- receive a versioned payload and claim/lease identity;
- heartbeat/checkpoint only the current claim where its kind permits it;
- report success/failure/waiting/reconciliation outcomes with the current claim
  token;
- leave all durable lifecycle and attempt state in `thoth`.

`thoth-dissemination` may continue to use a scheduled executor/runtime if that
is operationally appropriate; scheduling frequency is not job durability and
does not make GitHub Actions or another runner the system of record.

### 3.14 Application authorization

ADR-0008's machine-role decision remains binding.

This ADR creates no generic ZITADEL role named `WORKER`, `SERVICE`,
`MACHINE` or equivalent.

A domain executor that reaches protected Thoth API operations uses a
domain-specific, least-privilege authorization matrix. `DISSEMINATION_WORKER`
may be adapted to the generic job protocol for dissemination only; it does not
become authority for Hosting, Metrics, files, indexing or other job families.

The in-process `thoth-worker` does not need to call its own public GraphQL API
to perform internal domain operations merely to impersonate a machine role;
internal authorization/execution boundaries must instead be specified explicitly
by the owning implementation.

### 3.15 Operability, fairness and operator controls

The shared engine must expose durable/queryable operational signals at least by
job kind:

- ready queue depth;
- age of the oldest ready job;
- running jobs beyond their expected lease/runtime;
- retry-scheduled count;
- `RECONCILIATION_REQUIRED` count and age;
- terminal failure count/rate;
- execution latency/attempt distribution;
- worker liveness and last successful claim/heartbeat activity.

Each job kind defines bounded attempts, backoff and concurrency limits. Backoff
must prevent tight retry loops; exact algorithms and jitter are implementation
details. The scheduler must provide bounded fairness so one enabled kind cannot
indefinitely starve another. Provider-specific rate limiting remains the owning
handler's responsibility on top of engine-level concurrency limits.

A poison job terminates as `FAILED` or `RECONCILIATION_REQUIRED` according to
whether an external effect may be ambiguous; it is surfaced as attention rather
than retried forever. The architecture does not require a second dead-letter
queue.

Operator controls such as cancel, safe manual retry, replay and mark/reconcile
outcome are protected, audited operations through ADR-0010's staff operational
seam where staff control is applicable. They record authenticated actor, reason
and correlation to the underlying async job. Direct production SQL is not the
normal operator interface.

Manual retry is allowed only from a state whose domain contract proves replay is
safe. `RECONCILIATION_REQUIRED` cannot be converted to ordinary retry by a
generic "retry" button.

Job and attempt history is immutable while retained. Retention/archival/deletion
policy requires a separately reviewed specification; the initial engine must not
silently delete historical attempts.

Coalescing/debouncing may be defined by individual job kinds, but it must be
durable and auditable: it may reduce executable jobs only under an explicit
domain rule and must retain enough correlation to explain which events or
requests were represented by the surviving job.


---

## 4. BE-04 supersession and compatibility

Once ADR-0012 is approved and repository-authoritative, the shared async engine
supersedes the ADR-0008/ADR-0010 assumptions identified in the header and
section 1.3. It does **not** supersede ADR-0008's machine-role/least-privilege
rules or ADR-0010's staff operational/audit architecture.

BE-04 is already part of released Thoth code. The replacement therefore
distinguishes the **storage/execution model** from the **released API
compatibility surface**.

### 4.1 Storage and domain replacement

The next separately authorized shared-engine implementation must:

- introduce the generic async schema through a new forward migration;
- remove the unused `distribution_job`, `distribution_job_target` and
  `distribution_job_attempt` tables and their dedicated indexes;
- remove the four BE-04-only enum types
  `distribution_job_kind`, `distribution_job_status`,
  `distribution_job_attempt_result` and
  `distribution_job_cancellation_reason`;
- atomically reconcile `thoth-api/src/schema.rs`, persistence/domain models,
  CRUD/query code and tests as required by ADR-0003 and repository doctrine;
- **retain** `distribution_platform`, which remains shared Publisher Services
  domain state and is not a queue-only type.

The already-applied `20260814_v1.7.0` migration remains immutable.

### 4.2 Released BE-04 GraphQL compatibility window

The released BE-04 GraphQL lifecycle contract is not removed at the same moment
as the legacy tables.

During a bounded compatibility window, the existing released
claim/complete/fail/cancel job operations and their GraphQL types remain
available as a **deprecated compatibility facade** backed by the generic async
engine. Any existing BE-04 automatic job-creation path that remains exposed must
either create the equivalent generic job under the same default-OFF activation
semantics or be explicitly disabled through its separately approved deprecation
path; it must never continue writing removed legacy tables.

`thoth-dissemination` then migrates to the new protected generic,
kind-scoped claim/report contract under its own repository-local task.

Only after all downstream consumers are verified on the generic contract may a
later separately approved API-deprecation task remove the legacy BE-04 GraphQL
fields/types/toggle and update generated SDL/client surfaces. That removal must
follow the repository's approved deprecation-path rule; downstream repositories
must not be forced to guess a contract that has not merged.

### 4.3 Fresh preflight before removing legacy storage

The future migration/deployment task must verify immediately before production
storage removal:

1. all three legacy tables exist as expected;
2. all three contain zero rows and no operational history;
3. the deployed API's BE-04 creation toggle is OFF or absent;
4. the DIS-02 worker activation variable is not ON and no executor is actively
   claiming the legacy contract;
5. there is no current `DISSEMINATION_WORKER` legacy-job activity;
6. the compatibility facade has been changed to the generic storage before any
   request can reach dropped relations.

If any of those checks fail, the DROP is HOLD and data/consumer migration must
be designed explicitly.

### 4.4 Preserve proven BE-04 safety work

Superseding BE-04 storage does not discard its safety evidence. The generic
engine and dissemination adapter must preserve or improve the already-proven
principles:

- database claim fencing and stale-token refusal;
- absolute logical deduplication;
- bounded attempts;
- conservative classification of post-write ambiguity;
- no automatic replay of an abandoned predecessor when a provider write may
  already have happened;
- bounded/sanitized durable diagnostics.

Existing BE-04/DIS-02 source remains historical implementation evidence and may
be reused only where the new bounded specifications explicitly adopt and retest
it against the generic contract.

### 4.5 Development and fresh-database migration path

Because repository migrations execute in order, an environment currently before
v1.7.0, including dev as recorded at this decision, will apply v1.7.0 and then
the later replacement migration when brought fully current. A fresh database
does the same.

The replacement migration and test plan must therefore prove both the
production-like upgrade path from a schema where v1.7.0 is already present and
the full empty-database migration chain. It must not rely on dev and production
having identical starting versions by accident.

## 5. Initial job families

These are architectural examples/initial consumers, not implementation
authorization and not a promise that all ship in the first implementation.

```text
HOSTING_PROVISION
HOSTING_VERIFY_DOMAIN
HOSTING_RECONCILE

DISTRIBUTION / CROSSREF_SYNC

VALIDATE_FILE
DEPLOY_EPUB_READER
INVALIDATE_CDN

ALGOLIA_INDEX_WORK
WEBSITE_REVALIDATE
```

The implementation should prove the framework first with a bounded set of
representative handlers rather than enabling every possible integration in one
release.

---

## 6. Rejected alternatives

### A. Hosting-specific worker and queue

Rejected because it would create a second durable-job architecture immediately
before distribution, file-processing and indexing workloads need the same
primitives.

### B. Reuse BE-04 `distribution_job*` as the generic schema unchanged

Rejected because those tables and APIs were designed around Publisher Services
semantics. Renaming their interpretation by analogy would preserve programme
coupling and bypass the explicit ADR gate ADR-0008 established.

### C. SQS/Kafka/EventBridge as canonical queue now

Rejected for the initial architecture. PostgreSQL already provides transactional
co-location with Thoth business writes, durable state, row locking and sufficient
expected throughput. A later scale-driven ADR may add a delivery bus, but it
must not become the sole authoritative owner of job lifecycle state without a
new decision.

### D. One IAM role per handler

Rejected as the default because one process able to assume every handler role
does not materially isolate a compromise of that process and adds substantial
operational complexity. High-risk standing permissions may still justify
separate execution boundaries where evidence shows real blast-radius reduction.

### E. One generic catch-all application machine role

Rejected and still prohibited by ADR-0008. Shared queue mechanics do not imply
shared application authority.

### F. Put dissemination business logic into `thoth`

Rejected. Queue standardization is not repository-ownership transfer.
`thoth-dissemination` remains the owner of platform-specific dissemination
execution until a separate approved architecture decision says otherwise.

---

## 7. Invariants

1. PostgreSQL remains canonical for event/job lifecycle.
2. Business mutation plus caused event is atomic where both are PostgreSQL-owned.
3. Every event has durable identity and ordering; every `(event_id, route_key)`
   materialization is database-unique irrespective of job terminal state.
4. A newly registered route does not consume historical events without an
   explicit backfill/replay.
5. Job idempotency identity is absolute across states; retries are attempts on
   the same logical job.
6. No stale/superseded claim can finalize, checkpoint, cancel or renew a job.
7. Effectful attempts durably fence `EFFECT_STARTED` before crossing the
   external-write boundary.
8. Lease expiry/shutdown after an ambiguous effect cannot return the job to
   ordinary retry; it requires reconciliation unless a durable checkpoint proves
   safe resume.
9. No job handler may claim exactly-once external effects.
10. Indeterminate writes require reconciliation before unsafe replay.
11. Priority, delayed availability, bounded attempts, per-kind concurrency and
    fairness are first-class shared-engine concerns.
12. Cancellation never asserts that an already-started external effect did not
    happen.
13. `async_job`/`async_job_attempt` are canonical execution truth;
    `ServiceOperation` is the ADR-0010 staff operational/audit seam where
    applicable, not a second attempt ledger.
14. Domain ownership is explicit and preserved.
15. Generic execution does not create generic application authorization.
16. Historical migrations are never rewritten after deployment.
17. Worker AWS authority comes exclusively from its dedicated ECS task role; no
    static AWS access keys are injected into the worker.
18. Non-AWS credential values are not committed or emitted in logs/durable
    diagnostics.
19. A worker may execute only supported payload versions and configured
    capabilities.
20. Protected operator retry/reconcile controls are audited and may not bypass
    reconciliation safety.
21. Merge, migration, deployment, handler activation, provider access and
    production activation remain separate gates.

## 8. Implementation impact and decomposition

Approval of this ADR should lead to separate bounded tasks, at minimum:

1. **Shared schema/engine and BE-04 compatibility facade in `thoth`**
   - forward replacement migration;
   - event/route/job/attempt domain types;
   - claim/renewal/lease/retry/reconciliation primitives;
   - event routing/materialization;
   - execution-phase checkpoints;
   - deprecated BE-04 GraphQL facade backed by generic jobs;
   - atomic Diesel/model/query/generated-contract reconciliation;
   - tests including real PostgreSQL concurrency.

2. **Default `thoth-worker` runtime in `thoth`**
   - worker command;
   - handler registry;
   - graceful shutdown/checkpointing;
   - metrics/health/operational reporting;
   - capability/configuration filtering.

3. **Infrastructure**
   - headless Fargate worker service not bound to the ALB;
   - dedicated worker task role;
   - explicit prohibition of static AWS credentials for the worker;
   - non-committed deploy-time injection of external env-var credential values;
   - environment/network configuration;
   - no deployment/activation until separately authorized.

4. **Publisher Services / `thoth-dissemination`**
   - migrate from the deprecated BE-04 facade to generic kind-scoped job
     consumption;
   - preserve dissemination domain logic;
   - independently review external-write/idempotency/reconciliation semantics.

5. **BE-04 public-contract retirement**
   - after downstream migration, remove deprecated legacy GraphQL lifecycle
     fields/types/toggle under a separately approved deprecation task;
   - regenerate/update affected client SDL/types and verify all downstream
     consumers.

6. **Hosting**
   - adopt generic jobs for greenfield provisioning/reconciliation;
   - no automated legacy CDN migration.

7. **Existing AWS static-credential security debt**
   - inspect and remediate any existing service static AWS credential pattern
     under a separate security task and explicit provider/write authorization;
   - this is not silently folded into worker deployment.

Before ADR-0012 becomes repository-authoritative, its approval-state
reconciliation must also update ADR-0008 and ADR-0010 decision metadata/register
entries to record the exact partial supersessions selected here. That is
documentation/control reconciliation, not implementation authorization.

Each repository receives its own branch/PR and independent review. No downstream
repository guesses an unmerged upstream contract.

## 9. Migration and rollout

Architecture approval does not authorize migration execution.

The future database rollout must remain safe across both the production upgrade
path and the full migration chain. Before any production DROP of
`distribution_job*`, section 4.3's fresh preflight is mandatory.

The production replacement change must ensure no deployed code path can access
legacy relations after they are dropped. The deprecated GraphQL compatibility
facade must already target generic storage in the same bounded source/migration
release, and generated/client compatibility must be proven.

The development environment's currently unapplied v1.7.0 state is handled by
ordinary ordered migrations: v1.7.0 applies first, followed by the replacement.
The test matrix must include:

- production-like schema with v1.7.0 already applied;
- pre-v1.7 upgrade applying v1.7.0 plus replacement;
- full empty-database migration chain;
- forward repair/rollback strategy once generic async data exists.

Initial worker deployment is inert until job creation/activation is separately
authorized. Desired count starts at one unless deployment evidence justifies
otherwise.

Rollout should prove in order:

1. schema/event-route/job concurrency without external writes;
2. crash/lease renewal/expiry recovery;
3. idempotent partial fan-out recovery;
4. priority/fairness/per-kind concurrency and bounded retry behaviour;
5. one or more non-destructive representative handlers;
6. one effectful handler with durable `EFFECT_STARTED`, provider checkpoint and
   reconciliation evidence;
7. protected operator controls and operational signals;
8. only then broader handler activation.

Handler activation is independent: enabling Hosting must not automatically
enable Crossref, Algolia, file processing or another family merely because they
share the worker.

## 10. Rollback

Architecture rollback before implementation is ordinary documentation revert.

Once implemented, rollback must distinguish:

- stopping new event/job creation;
- stopping worker claiming;
- preserving durable event/job/attempt records;
- rolling back individual handler activation;
- schema rollback, which may be impossible after live async data exists and
  therefore requires its own migration plan.

Do not treat disabling the worker as equivalent to deleting durable jobs.

---

## 11. Validation requirements

Before shared implementation can be approved, evidence must include:

- real PostgreSQL concurrent claim tests with multiple workers;
- current-token lease renewal and refusal to renew superseded/expired claims;
- stale-token refusal for completion, failure, cancellation and checkpointing;
- crash immediately after claim;
- crash during event fan-out/materialization with exact resume and no duplicate
  `(event_id, route_key)` job;
- proof that newly registered routes do not consume historical events absent an
  explicit backfill;
- crash before and after `EFFECT_STARTED`;
- lease expiry after `EFFECT_STARTED` producing reconciliation rather than
  ordinary retry;
- provider-correlation checkpoint followed by WAITING/resume that polls rather
  than resubmits;
- deterministic absolute job idempotency tests;
- concurrency-key serialization tests;
- priority, `available_at`, bounded attempt and scheduler fairness tests;
- retry/backoff and poison-job exhaustion tests;
- cancellation before write and cancellation after effect-start tests;
- `INDETERMINATE` external-write reconciliation tests;
- unsupported payload-version refusal;
- graceful shutdown and abrupt restart recovery;
- worker capability filtering;
- authorization negative tests for every external/domain executor;
- protected/audited operator cancel/retry/reconcile tests;
- queryable queue-depth, oldest-ready-age, expired-lease,
  `RECONCILIATION_REQUIRED`, failure and worker-liveness signals;
- proof that logs and durable diagnostics do not contain configured credentials;
- proof that the worker receives no static AWS access-key credentials;
- migration tests from a schema containing the production v1.7.0
  `distribution_job*` relations;
- full empty-database migration-chain tests;
- compatibility tests showing released BE-04 GraphQL operations no longer
  depend on legacy tables during the compatibility window;
- deployment tests with more than one worker task even if production initially
  runs one.

Provider-specific handlers owe their own acceptance evidence in addition to the
shared-engine tests.

## 12. Approval and authority

This ADR is currently **PROPOSED**.

The CTO has selected the architecture direction recorded in programme issue
#957, but that does not make this exact written ADR content approved.

Before any implementation may rely on ADR-0012:

1. this exact proposed content receives independent review;
2. required corrections, if any, are incorporated;
3. the CTO explicitly approves the exact decision content;
4. status/approval metadata are reconciled under the repository decision process;
5. the exact approved content receives required independent exact-head review;
6. it is merged into `develop`.

No implementation task may treat the unmerged proposal as authoritative.

BE-04/DIS-02 deployment/activation and Hosting implementation that depends on
the async architecture remain HOLD throughout this gate.
