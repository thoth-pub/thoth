# ADR-0012 - Shared asynchronous event and job execution architecture

Status: PROPOSED
Date: 2026-09-29
Decision owner: CTO
Programmes affected: Shared Backend Architecture (owning programme); Publisher Services and Distribution Configuration; Thoth Hosting; future Thoth programmes requiring asynchronous work
Repositories affected: `thoth-pub/thoth` (shared durable engine and default worker runtime); `thoth-pub/thoth-dissemination` (domain executor/consumer); `thoth-pub/infrastructure` (worker runtime and IAM substrate)
Parent programme: [THOTH-ASYNC-01 #957](https://github.com/thoth-pub/thoth/issues/957)
Authoring task: [THOTH-ASYNC-01-ADR-01 #958](https://github.com/thoth-pub/thoth/issues/958)
Supersedes in part: ADR-0008 sections 3.3-3.5 and the corresponding programme consequences that prohibit a reusable cross-programme job/queue abstraction. ADR-0008's machine-role, least-privilege and SUPERUSER-separation decisions remain in force.

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

### 3.2 Events and jobs are different

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

Event routing itself must be durable and idempotent. A crash after materializing
some downstream jobs must be able to resume without duplicating already-created
jobs. An event that has completed routing must not silently gain new historical
subscribers merely because a later software version adds a new handler.

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

### 3.4 Job lifecycle and claims

The common job lifecycle must support at least the semantic states:

```text
PENDING
RUNNING
WAITING / RETRY_SCHEDULED
SUCCEEDED
FAILED
CANCELLED
RECONCILIATION_REQUIRED
```

Exact enum spelling is implementation detail.

Claims use database-enforced concurrency with leases and unforgeable claim
tokens. `FOR UPDATE SKIP LOCKED` is the default claiming primitive for the
shared queue because this workload explicitly requires multiple consumers to
claim independent ready jobs without blocking each other.

A stale worker must not be able to complete or fail a job after its lease/claim
has been superseded.

The engine must support horizontal scaling from one worker task to multiple
tasks without changing job semantics.

### 3.5 Idempotency and concurrency

Every externally meaningful job kind defines a deterministic idempotency key.
The database prevents duplicate live work for identities that the owning
handler declares equivalent.

Handlers may additionally define a concurrency key, for example:

```text
work:<uuid>:algolia
hosting:<target-id>
crossref:<doi>
```

Jobs with unrelated concurrency keys may execute in parallel. Jobs that would
mutate the same protected external/domain resource must serialize where the
owning specification requires it.

### 3.6 Delivery semantics and ambiguous writes

The engine guarantees durable at-least-once execution opportunities. It does
**not** claim exactly-once external effects.

A handler result distinguishes at least:

- success;
- retryable failure before a relevant external write;
- deterministic permanent failure;
- wait/retry at or after a specified time;
- indeterminate/ambiguous external-write outcome requiring reconciliation.

An indeterminate provider write is never blindly replayed merely because the
worker did not receive an acknowledgement. The owning handler must reconcile
provider state or require operator attention before another effectful attempt.

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

The worker polls/claims durable PostgreSQL jobs. PostgreSQL
`LISTEN/NOTIFY` may be used as a wake-up optimization, but correctness must
never depend on receiving a notification; periodic durable polling remains the
fallback.

The worker must shut down gracefully: stop claiming, finish or safely abandon
within the lease budget, and leave recoverable durable state.

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

The initial `thoth-worker` uses one least-privilege ECS task role containing
the union of its **approved routine AWS capabilities**.

Do not introduce one assumable IAM role per ordinary handler merely to mirror
code boundaries when the same process could assume all of them anyway.

The task role may eventually include bounded capabilities such as:

- approved Hosting tenant lifecycle;
- ACM operations needed by Hosting;
- writes to Thoth-controlled Hosting DNS;
- tenant/legacy invalidation where separately approved;
- bounded S3 object access required by file-processing handlers.

The exact permission matrix remains an implementation/IAM task and must retain
negative tests and resource scoping.

Standing permissions that materially increase blast radius and are not required
for routine operation remain excluded.

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

- credentials are never committed to source, infrastructure templates, task
  specifications or GitHub records;
- application logs and durable job diagnostics must never dump the process
  environment;
- handlers fail closed when required credentials are absent;
- providers use distinct credentials where operationally available;
- rotation is performed by updating deployment configuration and replacing the
  affected task/runtime.

Environment variables are an operational delivery mechanism, not a security
isolation boundary between handlers in the same process.

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

This does not require a second permanently running generic queue engine.
A downstream domain executor may be persistent, scheduled or on-demand as its
own approved implementation specifies. It must not create a second canonical job
database or lifecycle.

The shared protocol must therefore allow a domain executor to:

- claim only authorized job kinds;
- receive a versioned payload;
- report success/failure/indeterminate outcomes with the current claim token;
- leave all durable lifecycle state in `thoth`.

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

---

## 4. BE-04 supersession

Once ADR-0012 is approved and repository-authoritative, the shared async engine
supersedes ADR-0008's decision that BE-04 durable job machinery is the only
permitted programme-local implementation and that no shared framework exists.

It does **not** supersede ADR-0008's machine-role and least-privilege rules.

BE-04/DIS-02 must be re-specified before deployment:

- `distribution_job*` is not activated as a parallel production queue;
- the existing production tables are treated as unused legacy schema;
- a new migration drops those tables only after a fresh production preflight
  proves they contain no operational data;
- shared async tables are introduced through a new migration;
- Publisher Services job semantics are mapped onto generic jobs;
- `thoth-dissemination` consumes only its authorized generic job kinds;
- existing BE-04/DIS-02 implementation remains historical evidence and may be
  reused only where a new bounded specification explicitly adopts it.

The already-applied v1.7.0 migration remains immutable.

---

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
3. Routing and job creation are idempotent.
4. No stale claim can finalize a superseded lease.
5. No job handler may claim exactly-once external effects.
6. Indeterminate writes require reconciliation before unsafe replay.
7. Domain ownership is explicit and preserved.
8. Generic execution does not create generic application authorization.
9. Historical migrations are never rewritten after deployment.
10. Credentials are not committed or emitted in logs/durable diagnostics.
11. A worker may execute only supported payload versions and configured
    capabilities.
12. Merge, deployment, job activation and provider access remain separate gates.

---

## 8. Implementation impact and decomposition

Approval of this ADR should lead to separate bounded tasks, at minimum:

1. **Shared schema/engine in `thoth`**
   - replacement migration;
   - event/job/attempt domain types;
   - claim/lease/retry/reconciliation primitives;
   - event routing/materialization;
   - tests including real PostgreSQL concurrency.

2. **Default `thoth-worker` runtime in `thoth`**
   - worker command;
   - handler registry;
   - graceful shutdown;
   - metrics/health/operational reporting.

3. **Infrastructure**
   - headless Fargate worker service;
   - task role and networking;
   - environment-variable configuration;
   - no deployment/activation until separately authorized.

4. **Publisher Services / `thoth-dissemination`**
   - replace BE-04 job protocol with generic job-family consumption;
   - preserve dissemination domain logic;
   - independently review external-write/idempotency semantics.

5. **Hosting**
   - adopt generic jobs for greenfield provisioning/reconciliation;
   - no automated legacy CDN migration.

Each repository receives its own branch/PR and independent review. No downstream
repository guesses an unmerged upstream contract.

---

## 9. Migration and rollout

Architecture approval does not authorize migration execution.

The future database rollout must be additive/safe until the exact replacement
migration is independently approved. Before any production DROP of
`distribution_job*`:

1. verify the production tables exist from v1.7.0;
2. verify they contain no operational data;
3. verify no deployed process is reading/writing them;
4. verify the shared replacement schema has the required rollback/recovery plan;
5. receive explicit migration/deployment authorization.

The development environment's unapplied v1.7.0 state must be handled explicitly
by the migration specification; it must not rely on dev matching production by
accident.

Initial worker deployment is inert until job creation/activation is separately
authorized. Desired count starts at one unless deployment evidence justifies
otherwise.

Rollout should prove:

- schema and claim concurrency;
- crash/lease recovery;
- idempotent routing;
- one or more non-destructive representative handlers;
- one external-write handler with reconciliation evidence;
- observability and stuck-job procedures;

before broad adoption.

---

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
- stale-token/expired-lease refusal;
- crash between claim and completion;
- crash during event fan-out/materialization;
- deterministic deduplication/idempotency tests;
- concurrency-key serialization tests;
- bounded retry/backoff tests;
- INDETERMINATE external-write reconciliation tests;
- unsupported payload-version refusal;
- graceful shutdown/restart recovery;
- worker capability filtering;
- authorization negative tests for every external/domain executor;
- proof that logs and durable diagnostics do not contain configured credentials;
- migration tests from a schema containing the production v1.7.0
  `distribution_job*` relations;
- deployment tests with more than one worker task even if production initially
  runs one.

Provider-specific handlers owe their own acceptance evidence in addition to the
shared-engine tests.

---

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
