# ADR-0012 - Shared asynchronous event and job execution architecture

Status: PROPOSED
Date: 2026-09-29
Decision owner: CTO
Programmes affected: Shared Backend Architecture (owning programme); Publisher Services and Distribution Configuration; Thoth Hosting; future Thoth programmes requiring asynchronous work
Repositories affected: `thoth-pub/thoth` (shared durable engine and default worker runtime); `thoth-pub/thoth-dissemination` (dissemination executor/consumer); `thoth-pub/thoth-app` (released BE-04 read-surface consumer that must migrate before retirement); `thoth-pub/infrastructure` (worker runtime and IAM substrate)
Parent programme: [THOTH-ASYNC-01 #957](https://github.com/thoth-pub/thoth/issues/957)
Authoring task: [THOTH-ASYNC-01-ADR-01 #958](https://github.com/thoth-pub/thoth/issues/958)
Superseded by: None
Supersedes in part:
- ADR-0008 only where it denies a reusable cross-programme job framework/API: the final shared-framework prohibition in section 3.3, section 3.4's programme-local-only ownership rule, section 5.1 item 6, section 5.2 item 6 and rejected alternative D. ADR-0008 section 3.5 is **satisfied by this ADR, not superseded**. Section 3.3's approved convention list, its `approved primitive != mandatory mechanism` rule, and all machine-role, least-privilege and `SUPERUSER` separation rules remain binding.
- ADR-0010 only where invariant 15 and section 7.2 assume `distribution_job*` remains the long-term Publisher Services execution source, and where section 4.4's final sentence says ADR-0008 remains *fully* binding without the later shared-framework exception introduced here. Section 4.4's substantive rule that `ServiceOperation` is not itself a queue/framework/executor API remains binding. ADR-0010's Staff Operations Console, `ServiceOperation` audit seam, desired/execution/observed-state separation, attention/reconciliation model and staff-command gates remain binding as specified in section 1.3 below.

Decision: Thoth establishes one shared PostgreSQL-backed asynchronous event and
job engine for cross-programme durable work. Events record durable facts and may
fan out idempotently into jobs; jobs record executable asynchronous work and may
also be created directly for scheduled, reconciliation or operator-requested
work. The engine provides one shared lifecycle, claim/lease/idempotency model and
attempt history. A headless `thoth-worker` ECS/Fargate service is the default
always-on executor for trusted handlers owned by `thoth`. The shared engine may
use additional worker pools only where a materially different trust/authority
boundary requires process/task isolation; this ADR specifically requires such an
isolation boundary for publisher-controlled untrusted-content parsing. Domain-
specific executors in other repositories may consume the same shared job
protocol without moving their business logic into `thoth`.

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
cross-domain operational/audit seam. Its section 4.4 statement that ADR-0008
remains fully binding is qualified only by the exact ADR-0008 partial
supersession recorded by this later ADR; the `ServiceOperation` seam itself does
not become the generic queue or executor API. The relationship is binding:

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

Every event has a unique durable `event_id`. Route eligibility and routing
completion are determined from durable database state, not from worker-local
handler registration and not from a bare PostgreSQL sequence high-water mark.
A sequence may be used as an identifier/order aid, but the implementation must
not assume sequence allocation order equals commit order.

Every event consumer has a durable route registration with:

- a stable `route_key`;
- an explicit activation boundary;
- an optional explicit deactivation boundary;
- the supported event kind/version contract;
- enough durable state to decide whether an event is eligible for that route.

Activation and deactivation are durable, separately authorized and audited
operations. They are not side effects of merely deploying or deleting handler
code.

Route boundaries use a durable routing generation/epoch, or an equivalently
strong serialized database mechanism, that an event captures inside its
creating transaction and a route records when it is activated or deactivated.
An event is eligible exactly when its captured routing generation is at or after
the route's activation generation and, when a deactivation generation exists,
strictly before that deactivation generation.

That interval must partition events deterministically even when transactions
commit out of sequence. Sequence allocation order and commit observation order
alone are insufficient.

For every committed event, the engine must eventually establish a durable route
disposition for every route for which that event is eligible. The normal
disposition materializes a job and is unique on `(event_id, route_key)`
regardless of whether the resulting job is pending, running, waiting or
terminal.

A route may not be "fixed" by deleting its registration. Deactivation affects
only events at or after the deactivation boundary. Events already eligible
before that boundary remain owed after retirement. If an eligible route can no
longer be materialized safely, each affected event requires an explicit,
authorized, audited terminal route disposition describing why it was not
materialized and what reconciliation/backfill, if any, covers it.

A crash after some consumers are materialized therefore resumes from durable
route records. A worker must never mark an event fully routed merely because it
processed all routes known to its own binary. During a mixed-version deployment,
if a worker encounters an eligible route whose kind/version it cannot
materialize, it leaves that route incomplete for a compatible worker rather than
silently completing the event.

A route introduced by a later release receives only events in its active
routing-generation interval. Historical events are processed only through an
explicitly authorized backfill/replay using the same `(event_id, route_key)`
identity; backfill cannot bypass route deduplication.

### Event-emission deployment floor

A route consuming event kind/version `K` may be activated only after deployment
evidence proves that **every running binary that can perform the relevant
business mutation, and every binary still retained as a supported production
rollback target, emits K atomically with that mutation**.

The event-emission floor is therefore established before route activation.
Activating a route while some serving/rollback binary can still perform the
business mutation without emitting K is prohibited.

If production must roll back below that emission-capable floor after activation,
the route is deactivated before the older binary can perform relevant writes, or
the rollback is accompanied by an explicitly authorized current-state
reconciliation/backfill that covers the precise emission gap. Silent loss of
business changes is not an acceptable rollback behaviour.

The implementation may optimize discovery/scanning, but routing completeness
must be proven from durable event/route/disposition records. Any cursor or
high-water optimization must prove that no lower/in-flight event can commit
later and be skipped; absent such proof, it is not authoritative.

### 3.3 Versioned contracts, payload minimization and revision semantics

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

Event/job payloads default to **canonical references**, not copied domain
snapshots. A payload should normally contain only:

- canonical entity/resource identifiers;
- the source revision/version/fingerprint needed to bind the requested effect;
- minimal routing/command parameters;
- correlation/causation identifiers.

Credentials, tokens and secrets are forbidden in event/job payloads.

Personal data or immutable domain snapshots may be embedded only when a
kind-specific approved specification proves that references are insufficient,
bounds the data, and defines retention/erasure behaviour. The generic engine's
immutability/retention rules must not make a data-erasure obligation impossible.

Every job kind declares its **revision semantics**. The initial architectural
modes are:

```text
CURRENT_STATE
REVISION_BOUND
```

A `CURRENT_STATE` job means "make the external/derived target reflect the
latest canonical state when this job executes". It reads canonical state at
execution time, records the revision/fingerprint actually acted on, may coalesce
older triggers under its explicit kind rule, and may complete as a deterministic
no-op when the required current revision is already applied.

A `REVISION_BOUND` job means "process exactly this retained source
revision/fingerprint". Before any external write, the handler must determine
whether a newer revision for the same effect/concurrency target has already been
applied. If so, the older job must not overwrite/regress that newer state: it
records a deterministic superseded/no-effect disposition unless it is an
explicitly authorized historical replay/rollback operation.

Each REVISION_BOUND kind therefore defines the revision ordering/evidence by
which "newer already applied" is established. Merely serializing execution by a
concurrency key is not sufficient because an older retry may run after a newer
job has completed.

Historical replay is permitted only when the referenced historical revision is
actually retained and the replay/rollback is explicitly authorized. A stale
embedded snapshot must never silently substitute for either current-state
execution or an authorized historical replay.

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

Exact enum spelling is implementation detail.

`WAITING` and `RETRY_SCHEDULED` are semantically distinct:

- `WAITING` means an effectful or asynchronous operation has durably
  checkpointed and is waiting on an external/domain condition;
- `RETRY_SCHEDULED` means a failed **pre-write** attempt is eligible for
  another execution after `available_at`.

Transitioning to `WAITING` atomically persists the durable checkpoint and
releases the current claim and lease. A later resume is a new claim and a new
attempt that reads the checkpoint. Waiting/polling attempts do **not** consume
the pre-write retry budget; they are bounded by the kind's waiting deadline,
poll/backoff policy and any separately defined reconciliation budget. They are
not silently treated as fresh external submissions.

Every job carries first-class scheduling attributes including `priority`,
`available_at`, the owning kind and counters/deadlines appropriate to its
lifecycle. Ready work is ordered by priority and availability within the
scheduler's fairness/concurrency rules; priority must not permit one job kind to
starve all other enabled kinds.

Every job kind declares separately:

- a bounded **pre-write retry budget**;
- for asynchronous/post-write work, a bounded waiting/reconciliation deadline
  or equivalent domain-specific completion horizon;
- expected effect window/runner deadline;
- maximum claim/attempt runtime.

Exhausting the pre-write retry budget is terminal `FAILED`. Exhausting a
waiting/post-write deadline after `EFFECT_STARTED` is never `FAILED` merely
because time elapsed; it becomes `RECONCILIATION_REQUIRED` unless durable
provider/domain evidence proves a stronger terminal outcome.

Claims use database-enforced concurrency with leases, unforgeable claim tokens
and `FOR UPDATE SKIP LOCKED`. Leases are renewable by heartbeat using the
**current** claim token only. Renewal may extend only an unexpired current claim,
has a bounded maximum runtime, and can never revive or extend a superseded
claim. A stale worker cannot complete, fail, checkpoint, cancel, renew or
reconcile a job after its claim is superseded.

Every job kind declares whether execution is **read-only/non-effectful** or
**externally effectful**. Every attempt durably records an execution phase at
least equivalent to:

```text
PRE_WRITE
EFFECT_STARTED
EFFECT_CONFIRMED
```

An effectful attempt must not cross `EFFECT_STARTED` unless the remaining
lease/runtime budget covers the kind's declared effect window. Long operations
must checkpoint/yield before their maximum claim runtime rather than rely on an
expired lease.

A lease expiry while an attempt is provably `PRE_WRITE` may schedule another
pre-write attempt when its retry budget permits. Lease expiry/crash/shutdown for
an effectful attempt at or after `EFFECT_STARTED` transitions to
`RECONCILIATION_REQUIRED`, never directly to `PENDING` or
`RETRY_SCHEDULED`, unless a domain-specific durable checkpoint proves that
resumption cannot repeat the external effect.

`WAITING` and `RECONCILIATION_REQUIRED` retain exclusive ownership of their
concurrency key by default. A kind may release it only under an explicit,
reviewed rule proving that overlapping work cannot create conflicting external
effects.

Cancellation invalidates the current claim. Cancelling pending/pre-write work
may produce terminal `CANCELLED`. Cancelling an effectful job at or after
`EFFECT_STARTED` cannot assert that no external effect happened and therefore
produces `RECONCILIATION_REQUIRED` unless provider/domain evidence proves a
stronger safe terminal outcome.

Resolving `RECONCILIATION_REQUIRED` is itself a claimed, claim-token-fenced,
kind-authorized action recorded as a durable attempt. A domain reconciler or
protected staff command may establish `SUCCEEDED`, `FAILED`, `CANCELLED`,
a safe `WAITING` checkpoint, or a new explicitly authorized effectful attempt
only from evidence permitted by that kind's reconciliation contract.

The full legal transition table, including claim-token requirements and attempt
budget/deadline effects, is fixed by the shared-engine implementation
specification and tested as a database/domain invariant.

The engine must support horizontal scaling from one worker task to multiple
tasks without changing these semantics.

### 3.5 Idempotency, coalescing and concurrency

Every logical job has a deterministic idempotency key. Database uniqueness of
that key is absolute across job states, but the key identifies a **specific
intended effect**, not merely a resource.

A kind's key therefore incorporates the dimension that makes a later effect
meaningfully new, for example:

- `(event_id, route_key)` or an event-derived source revision;
- a source revision/fingerprint;
- a schedule slot;
- an explicit command/request id;
- an approved coalescing/debounce window.

A completed Crossref sync, search-index update, invalidation or reconciliation
must not suppress a later effect for a newer revision merely because both target
the same DOI/work/hosting target.

Retries create new attempts on the same logical job. Intentional replay,
current-state execution and backfill create new logical jobs with new effect
identity and explicit parent/replay/correlation metadata, following ADR-0010's
distinction between retry, replay and current-state redistribution.

Event routing identity and job idempotency are related but distinct. Every
`(event_id, route_key)` route record points to exactly one materialized job,
while several route records may point to one **not-yet-started** job only when
the kind defines an explicit durable coalescing rule. Coalescing therefore forms
a many-route-to-one-job relationship; it never erases the per-event route
records needed for audit/completeness.

An idempotency conflict never silently means "route satisfied". It either:

1. attaches the route to an existing not-yet-started job when the kind's
   coalescing rule explicitly permits it; or
2. produces a deterministic routing/materialization error surfaced for
   attention/recovery.

Handlers additionally define a concurrency key where overlapping effects must
serialize, for example:

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
persists that identifier before atomically yielding `WAITING` and releasing
its claim/lease.

A later WAITING resume is a new claimed attempt. With a durable provider
identifier it polls or reconciles the existing provider operation; it does not
submit a fresh write unless the domain contract has established that no previous
effect can exist.

If durable `EFFECT_CONFIRMED` evidence exists before a crash, a later claimed
recovery may complete `SUCCEEDED` without repeating the effect, where the
kind's contract defines that evidence as sufficient. `EFFECT_CONFIRMED` is
execution evidence that rules out replay of that effect; it is **not** by itself
ADR-0010 observed publication, acceptance or current external-state truth.

An indeterminate provider write is never blindly replayed merely because the
worker did not receive an acknowledgement. Lease expiry, process crash,
shutdown, waiting-deadline exhaustion or cancellation after
`EFFECT_STARTED` therefore fences the job into
`RECONCILIATION_REQUIRED` unless a durable domain checkpoint proves a safe
non-duplicating continuation.

The owning handler/reconciler must establish provider state or require protected
operator attention before another effectful attempt.

### 3.7 Attempts, diagnostics and evidence retention

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

Payload minimization and data-protection rules apply equally to attempt
diagnostics and provider evidence. Personal data or provider response excerpts
may be retained only where the kind-specific specification proves they are
required, bounds/sanitizes them and defines retention/erasure behaviour.
Append-only auditability does not override an applicable erasure obligation;
where evidence must be redacted/tombstoned, the system retains the non-sensitive
fact that evidence existed and why it was altered.

### 3.8 Worker runtime and trust-separated pools

The default always-on executor is a headless ECS/Fargate service using the
ordinary Thoth release image with a dedicated command, conceptually:

```text
thoth start worker
```

The default **trusted** worker pool handles Thoth-owned jobs whose inputs do not
cross a materially different trust boundary. Initial runtime shape:

```text
same ECS cluster
same VPC/database
no ALB
no public hostname
DesiredCount = 1 initially
separate task definition/service
dedicated trusted-worker task role
```

The existing generic infrastructure `service.yml` is web-service/ALB-oriented
and must not be reused unchanged for the headless worker. The infrastructure
implementation must either provide a bounded headless worker template or
explicitly generalize load-balancer attachment under its own reviewed task.

The worker polls/claims durable PostgreSQL jobs. PostgreSQL
`LISTEN/NOTIFY` may be used as a wake-up optimization, but correctness must
never depend on receiving a notification; periodic durable polling remains the
fallback.

### Untrusted-content pool

A handler that parses or executes against publisher-controlled/untrusted file
content is a separate **UNTRUSTED_CONTENT** risk class. It must not run in a task
that also holds Hosting DNS, ACM or CloudFront tenant-management authority.

Such handlers **must use the same canonical async engine/protocol** through a
separate task/service or equivalent process-isolated worker pool with a minimal
dedicated task role. This is a trust-separated executor pool, not a second queue
architecture.

The untrusted-content pool:

- may read only the input objects required by its authorized jobs;
- writes transformed/generated output only to an isolated quarantine/staging
  location, never directly to publisher-served or Hosting production paths;
- cannot choose or write the final publisher domain, served bucket, served key
  or Hosting target;
- cannot create arbitrary downstream events/jobs; it may report only the
  declared bounded completion/evidence facts for its authorized job kinds;
- receives no broad canonical-domain mutation authority.

Untrusted completion payloads/results are themselves **untrusted data**.
A trusted promotion/deployment job validates the staged artifact, verifies its
hash/size/type/evidence, and resolves publisher identity, destination domain,
bucket, key and Hosting/CDN target from canonical Thoth state. Trusted handlers
must never take authority-bearing destination values from an untrusted result.

The untrusted-content pool must not inherit the primary application database
credential. It uses either:

- the protected kind-scoped executor API; or
- a separately reviewed database principal whose database enforcement limits it
  to the exact async/file-processing rows and operations required by that pool.

Table-level grants on shared async tables are not sufficient kind isolation.
Direct database access therefore requires row/kind enforcement such as RLS,
security-definer functions/procedures or an equivalently reviewed database
boundary.

Its AWS/provider role likewise excludes Hosting DNS, ACM and CloudFront
tenant-management authority and direct write access to publisher-served
locations unless an explicit CTO risk acceptance identifies the concrete need
and blast radius.

A handler may cross that boundary only through explicit CTO risk acceptance
that identifies the concrete parser/runtime, AWS/provider authority, database
authority, served-content authority and blast radius. This is a trust-boundary
exception to the single routine role, not a return to one IAM role per ordinary
handler.

### Shutdown/recovery for every pool

Graceful shutdown rules apply to every worker pool. A pool stops new claims
first. A pre-write attempt may report a retryable pre-write outcome or
release/yield according to the shared transition contract. An effectful attempt
after `EFFECT_STARTED` must persist a safe checkpoint and yield `WAITING`, or
enter `RECONCILIATION_REQUIRED`; shutdown must never intentionally abandon it
to ordinary lease-expiry retry.

Every pool must prove clean recovery after abrupt termination as well as
graceful shutdown.

### 3.9 Handler registry, capabilities, risk class and revision mode

Each worker pool has a typed registry of supported job kinds. Each handler
declares:

- job kind and supported payload version(s);
- owning domain;
- required runtime capability/configuration;
- trust/risk class, including whether it handles untrusted content;
- revision semantics: `CURRENT_STATE` or `REVISION_BOUND`;
- effect classification and expected effect window;
- timeout/lease/waiting-deadline requirements;
- retry/reconciliation policy;
- idempotency/coalescing/concurrency rules;
- for high-volume families, retention/coalescing/admission/backpressure rules
  required before activation.

A worker only claims job kinds whose required capabilities, revision mode and
risk class are configured for that pool. Capability declarations are
runtime/configuration boundaries and scheduling primitives; they do not imply
one IAM role per ordinary handler.

External/domain executors use the same kind scoping: their authorization permits
only the job kinds assigned to that domain executor. A dissemination executor
cannot claim/checkpoint/complete/reconcile a Hosting, Metrics, file-processing
or indexing job merely because those jobs share the same database.

### 3.10 AWS credentials and task roles

The default trusted `thoth-worker` pool uses one dedicated least-privilege ECS
task role containing the union of its **approved routine trusted capabilities**.

AWS authority for each worker pool comes **exclusively** from that pool's ECS
task role. Worker tasks must not receive static `AWS_ACCESS_KEY_ID`,
`AWS_SECRET_ACCESS_KEY`, session-token credentials or equivalent long-lived
AWS credentials through environment variables, committed configuration or any
other deployment mechanism.

Do not introduce one assumable IAM role per ordinary handler merely to mirror
code boundaries when the same trusted process could assume all of them anyway.

The trusted-worker role may eventually include bounded capabilities such as:

- approved Hosting tenant lifecycle;
- ACM operations needed by Hosting;
- writes to Thoth-controlled Hosting DNS;
- approved CDN invalidation;
- bounded S3 object access required by trusted handlers.

Handlers in the `UNTRUSTED_CONTENT` risk class use a **separate** worker
pool/task role that excludes Hosting DNS, ACM and CloudFront tenant-management
authority unless explicitly risk-accepted by the CTO. This separation exists
because compromise of an untrusted parser is a materially different blast-radius
case, not because handler boundaries should mechanically map to IAM roles.

The exact permission matrices remain implementation/IAM tasks and must retain
negative tests and resource scoping. Standing permissions that materially
increase blast radius and are not required for that worker pool remain excluded.

The infrastructure implementation therefore intentionally departs from the
current shared `ECSTaskRole`/static-key pattern for existing Thoth web
services: it must create dedicated worker-pool task roles and ensure no static
AWS keys are injected into worker tasks. Remediation/rotation of any
pre-existing static AWS credentials used by other services is separate security
work and is not authorized or performed by this ADR.

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

## 4. BE-04 supersession and expand-migrate-contract transition

Once ADR-0012 is approved and repository-authoritative, the shared async engine
supersedes the ADR-0008/ADR-0010 assumptions identified in the header and
section 1.3. It does **not** supersede ADR-0008's machine-role/least-privilege
rules or ADR-0010's staff operational/audit architecture.

BE-04 is already part of released Thoth code and is consumed by more than the
dissemination worker. The transition therefore uses **expand -> migrate
consumers -> retire legacy API -> contract storage**. It does not translate the
new lifecycle through the old five-state GraphQL enum.

### 4.1 Release N - expand without contracting legacy BE-04

A separately authorized Release N introduces:

- the generic async schema/domain/API;
- event/route/job/attempt primitives;
- the new kind-scoped claim/checkpoint/complete/reconcile contract;
- generic job creation for newly specified consumers.

Release N **retains unchanged legacy BE-04 storage and released GraphQL
contract**. It does not drop:

- `distribution_job`;
- `distribution_job_target`;
- `distribution_job_attempt`;
- the four BE-04-only queue enum types;
- BE-04 lifecycle/query GraphQL fields/types.

The existing BE-04 automatic job-creation path/toggle remains OFF/inactive while
legacy consumers are migrated. No state-translating compatibility facade is
introduced.

Historical `20260814_v1.7.0` remains immutable.

### 4.2 Known released consumers that must migrate

At ADR authoring time, the known released consumers include at least:

1. **`thoth-pub/thoth-dissemination`**
   - BE-04 claim/complete/fail lifecycle operations;
   - current DIS-02 semantics and worker authorization.

2. **`thoth-pub/thoth-app`**
   - `PublisherServiceConfigurationSummary.latestBackCatalogueJob`;
   - service-configuration list/count filters including `jobStatuses`;
   - `withoutBackCatalogueJob`;
   - generated `DistributionJobStatus` and related GraphQL client contract.

The implementation programme must inspect all repository-authoritative contract
consumers again immediately before retirement; this list is not permission to
assume no additional consumer exists.

`thoth-app` and `thoth-dissemination` each receive their own repository-local
bounded migration task/branch/PR and must consume an exact merged generic
contract. No downstream repository guesses the new schema.

### 4.3 Consumer migration

After Release N is the deployed/rollback-safe Thoth version:

- `thoth-dissemination` migrates to the generic kind-scoped
  claim/checkpoint/complete/reconcile contract;
- `thoth-app` migrates its back-catalogue job reads/filters/status presentation
  to the new operational contract selected by its approved task;
- generated SDL/client surfaces are refreshed from the merged Thoth contract;
- each downstream deployment is independently verified.

Until both known consumers (and any newly discovered consumers) are verified on
the generic contract, legacy BE-04 API/storage remains present and unchanged.

### 4.4 Legacy API and runtime retirement

Only after all downstream consumers are verified migrated may a later
separately approved Thoth release retire the legacy BE-04 contract.

Retirement covers **every runtime path that can read or write
`distribution_job*`**, not merely the named GraphQL job fields/types/filters.
Before this release is approved, a repository-wide source audit must identify
and disposition all runtime legacy-table access.

At the current architecture baseline this includes at least:

- legacy BE-04 claim/complete/fail/cancel/query/filter GraphQL surfaces;
- `PublisherServiceConfigurationSummary` legacy job reads;
- `replacePublisherServiceConfiguration` automatic back-catalogue creation;
- its fail-closed `AUTOMATIC_PUSH` activation guard;
- assignment-disable cancellation through
  `cancel_pending_jobs_for_disabled_group_on`.

The Publisher Services desired-state mutation retains its atomic/fail-closed
invariant throughout retirement:

- if an activation requires asynchronous onboarding work and the approved
  generic creation path is active, the desired-state change and caused generic
  event/job intent commit atomically;
- if that generic path is not active/available, the activation remains rejected
  rather than committing without its required work;
- assignment disable/cancellation/supersession is redirected to the generic
  lifecycle or remains conservatively blocked; it does not query a legacy table
  that a later release may remove.

The legacy creation toggle/guard is therefore replaced by the generic rule or
retained as a fail-closed gate until replacement is available; it is never
simply deleted while allowing jobless activations.

This retirement release follows the repository's approved deprecation path and
updates generated SDL/client evidence. It does not collapse
`WAITING`/`RECONCILIATION_REQUIRED` into the old five-value
`DistributionJobStatus`.

A retirement gate must prove, by repository-wide runtime-reference search and
tests, that the deployed/rollback-safe Phase-C binary performs **zero runtime
queries or writes against the legacy job tables**. Compile-time/inert schema or
type declarations may remain temporarily for the later contraction task, but no
request, background path or feature toggle may touch those relations.

Only after that zero-runtime-access release is deployed and its rollback window
is closed may storage contraction proceed.

### 4.5 Later storage contraction

Legacy storage is removed only in a **later destructive migration** after:

1. the generic schema/API release is the sole running production version;
2. the legacy API/runtime retirement release is deployed and repository evidence
   proves zero runtime legacy-table access;
3. all known downstream consumers are on the generic contract;
4. the previous deployment rollback target is the zero-runtime-access Phase-C
   binary and therefore no supported rollback binary requires the legacy
   relations;
5. no executor can claim/write the legacy contract;
6. the destructive migration obtains locks that prevent a concurrent legacy
   write, verifies all three legacy tables exist as expected and contain zero
   rows, and performs the DROP under the same fail-closed migration transaction.

If any legacy row exists, required relation is unexpected, or the final locked
emptiness assertion cannot be established, the migration aborts and production
deployment is HOLD. An earlier read-only preflight is useful evidence but is not
sufficient by itself.

The contraction removes:

- `distribution_job`;
- `distribution_job_target`;
- `distribution_job_attempt`;
- their dedicated indexes;
- `distribution_job_kind`;
- `distribution_job_status`;
- `distribution_job_attempt_result`;
- `distribution_job_cancellation_reason`.

It **retains** `distribution_platform`, which remains Publisher Services domain
state.

Because all runtime legacy access was already removed in Phase C, the Phase-D
source/migration change removes only remaining inert legacy declarations such as
Diesel `schema.rs` table/type definitions and obsolete model/type declarations,
atomically with the DROP as required by ADR-0003 and repository doctrine. An
older Phase-C task may coexist during the rolling Phase-D deployment because it
contains no runtime access to the relations being removed.

### 4.6 Preserve proven BE-04 safety work

Superseding BE-04 storage does not discard its safety evidence. The generic
engine and dissemination adapter must preserve or improve:

- database claim fencing and stale-token refusal;
- effect-scoped deterministic deduplication;
- bounded pre-write retries;
- runner/effect-window checks before starting an external write;
- conservative classification of post-write ambiguity;
- no automatic replay of an abandoned predecessor when a provider write may
  already have happened;
- bounded/sanitized durable diagnostics.

Existing BE-04/DIS-02 source remains historical implementation evidence and may
be reused only where new bounded specifications explicitly adopt and retest it
against the generic contract.

### 4.7 Development and fresh-database migration path

Repository migrations still execute in order. An environment currently before
v1.7.0, including dev as recorded at this decision, first applies v1.7.0, then
Release N's additive generic schema migration, and only later the separately
authorized contraction migration after the consumer/API retirement gates are
satisfied.

A fresh database follows the same ordered history. Tests must therefore cover:

- production-like upgrade from v1.7.0 with legacy relations present;
- pre-v1.7 upgrade through v1.7.0 and the additive generic migration;
- full empty-database migration chain;
- the later contraction migration with zero legacy rows;
- fail-closed contraction when any legacy row exists.

No environment is allowed to skip or rewrite the historical v1.7.0 migration.

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

Rejected as the default because one trusted process able to assume every
ordinary handler role does not materially isolate compromise of that process and
adds substantial operational complexity.

This rejection does **not** apply across materially different trust boundaries.
Publisher-controlled untrusted-content parsing is explicitly separated into a
different worker pool/task role because a parser exploit gaining Hosting
DNS/ACM/tenant authority is a concrete blast-radius increase.

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
3. Route registration, activation and deactivation boundaries are durable
   database state; eligibility is a durable routing-generation interval.
4. Route activation occurs only after all serving and supported rollback binaries
   that can perform the relevant business mutation emit the consumed event
   kind/version atomically.
5. Deactivation never erases previously eligible work: pre-deactivation events
   remain owed until materialized or given an explicit authorized/audited route
   disposition.
6. A newly activated route does not consume historical events without an
   explicit backfill/replay through the same route materialization records.
7. Job idempotency identifies a specific intended effect; a resource-only key
   must not suppress later revisions/schedule slots/commands.
8. Every event-route record maps to exactly one job; many route records may map
   to one job only under an explicit not-yet-started coalescing rule.
9. Every job kind declares `CURRENT_STATE` or `REVISION_BOUND` semantics.
   A REVISION_BOUND effect must not regress a newer already-applied revision
   except through explicit historical replay/rollback authorization.
10. No stale/superseded claim can finalize, checkpoint, cancel, renew or
    reconcile a job.
11. WAITING atomically checkpoints and releases claim/lease; resume is a new
    claimed attempt.
12. Effectful attempts durably fence `EFFECT_STARTED` before crossing the
    external-write boundary and start only when the remaining effect window fits
    within the current lease/runtime.
13. Lease expiry/shutdown/cancellation/deadline exhaustion after an ambiguous
    effect cannot return the job to ordinary retry or terminal FAILED; it
    requires reconciliation unless durable evidence proves a stronger outcome.
14. `EFFECT_CONFIRMED` prevents duplicate replay where defined; it is not by
    itself ADR-0010 observed-state/publication/acceptance truth.
15. WAITING and RECONCILIATION_REQUIRED retain concurrency-key exclusion by
    default, and blocked successors are operationally visible.
16. Reconciliation is itself kind-authorized, claim-token-fenced and recorded as
    an attempt.
17. No job handler may claim exactly-once external effects.
18. Priority, delayed availability, bounded pre-write attempts,
    post-write/waiting deadlines, per-kind concurrency and fairness are
    first-class shared-engine concerns.
19. `async_job`/`async_job_attempt` are canonical execution truth;
    `ServiceOperation` is the ADR-0010 staff operational/audit seam where
    applicable, not a second attempt ledger.
20. Event/job payloads default to canonical references plus revision/fingerprint
    and minimal parameters; credentials are forbidden and personal/snapshot data
    requires explicit retention/erasure justification. The same rule applies to
    attempt diagnostics/provider evidence.
21. Domain ownership is explicit and preserved.
22. Generic execution does not create generic application authorization.
23. Historical migrations are never rewritten after deployment.
24. Trusted-worker AWS authority comes exclusively from its dedicated ECS task
    role.
25. UNTRUSTED_CONTENT jobs use the same canonical engine but a separate
    minimal-authority execution boundary; they write only quarantine/staging
    output and cannot choose or write served/Hosting destinations.
26. Trusted promotion resolves authority-bearing destinations from canonical
    state and treats untrusted results as untrusted data.
27. No worker receives static AWS access keys.
28. Non-AWS credential values are not committed or emitted in logs/durable
    diagnostics.
29. A worker/executor may claim only supported job versions/kinds and configured
    capabilities/risk classes.
30. High-volume job kinds define retention/coalescing/admission/backpressure
    rules before activation.
31. Protected operator retry/reconcile controls are audited and may not bypass
    reconciliation safety.
32. BE-04 retirement follows expand -> migrate consumers -> remove every runtime
    legacy-table dependency -> close rollback window -> contract storage.
33. Legacy storage is never dropped while a supported running/rollback binary
    can query or write it.
34. Merge, migration, deployment, handler activation, provider access and
    production activation remain separate gates.

## 8. Implementation impact and decomposition

Approval of this ADR should lead to separate bounded tasks, at minimum:

1. **Release N shared schema/engine/API in `thoth`**
   - additive generic async migration;
   - durable route registration/activation/deactivation/materialization model;
   - job/attempt/checkpoint lifecycle;
   - claim/renewal/wait/reconciliation primitives;
   - effect-scoped idempotency and explicit coalescing;
   - revision-semantics contract;
   - new kind-scoped executor API;
   - generic operational/read contract required by downstream consumers;
   - legacy BE-04 schema/API left intact and inactive;
   - real PostgreSQL concurrency/mixed-version/out-of-order-commit tests.

2. **Trusted default `thoth-worker` runtime in `thoth`**
   - worker command;
   - handler registry;
   - graceful shutdown/checkpointing;
   - metrics/health/operational reporting;
   - capability/kind/revision-mode filtering.

3. **Infrastructure - trusted worker**
   - headless Fargate service not bound to the ALB;
   - dedicated trusted-worker task role;
   - no static AWS credentials;
   - non-committed deploy-time injection of non-AWS env-var credential values;
   - environment/network configuration.

4. **Infrastructure/runtime - untrusted-content pool**
   - separate process/task/service boundary for publisher-controlled parsers;
   - minimal task role with no Hosting DNS/ACM/CloudFront tenant authority;
   - quarantine/staging-only output permissions;
   - protected executor API or row/kind-enforced minimal database principal;
   - explicit job-kind/risk-class filtering;
   - trusted promotion path from staged artifact to served content;
   - deployment only when an approved untrusted-content handler is ready.

5. **`thoth-dissemination` consumer migration**
   - move from legacy BE-04 operations to generic kind-scoped
     claim/checkpoint/complete/reconcile;
   - consume the exact merged upstream generic Publisher Services job-creation
     and executor contracts;
   - preserve dissemination domain logic and external-write safety;
   - independently review retry/reconciliation/revision behaviour.

6. **`thoth-app` consumer migration**
   - migrate `latestBackCatalogueJob`, `jobStatuses`,
     `withoutBackCatalogueJob` and related presentation/filter semantics to
     the exact merged generic operational/read contract;
   - regenerate/update GraphQL client surfaces;
   - independently verify staff UX semantics.

7. **Legacy BE-04 API/runtime retirement in `thoth`**
   - only after all consumers are deployed on generic contracts;
   - remove legacy lifecycle/read/filter fields/types/toggle;
   - redirect/remove Publisher Services runtime creation/cancellation/guard paths
     so no code accesses `distribution_job*`;
   - preserve fail-closed AUTOMATIC_PUSH activation through the generic rule;
   - repository-wide source audit proving zero runtime legacy-table access;
   - update generated SDL/client compatibility evidence.

8. **Later BE-04 storage contraction in `thoth`**
   - only after the zero-runtime-access retirement release is deployed and its
     older rollback window is closed;
   - migration-local lock + zero-row assertion;
   - remove three legacy tables/indexes/four queue-only enums;
   - retain `distribution_platform`;
   - remove only remaining inert schema/model declarations atomically with DROP.

9. **Hosting**
   - adopt generic jobs for greenfield provisioning/reconciliation;
   - no automated legacy CDN migration.

10. **Existing AWS static-credential security debt**
   - inspect/remediate any existing service static AWS credential pattern under a
     separate security task and explicit provider/write authorization;
   - this is not silently folded into worker deployment.

11. **Approval-state ADR reconciliation**
   - update ADR-0008/ADR-0010 metadata/register under a separate authorized write
     budget after exact-content CTO approval;
   - this documentation/control step is not implementation authorization.

Each repository receives its own branch/PR and independent review. No downstream
repository guesses an unmerged upstream contract.

## 9. Migration and rollout

Architecture approval does not authorize migration execution.

The production transition is deliberately non-atomic across releases/repos.

### Phase A - expand and establish event-emission floor

Release N adds the generic async schema/API and may deploy the inert trusted
worker runtime. Legacy BE-04 schema/API remains present and unchanged, with
legacy automatic job creation/activation OFF.

Any new event route remains INACTIVE until every serving binary and supported
rollback target capable of the relevant business mutation emits the event
kind/version atomically. Route activation is a separate authorized operation
after deployment evidence establishes that emission floor.

Release N must be fully deployed and become a valid rollback target before any
downstream consumer migration starts.

### Phase B - migrate consumers

`thoth-dissemination` and `thoth-app` migrate independently to the exact
merged generic contracts. Any newly discovered consumer is added to the same
gate.

No legacy GraphQL field/type is removed until every consumer is verified
deployed on the generic contract.

### Phase C - retire legacy API and all runtime table access

A later Thoth release removes the legacy BE-04
lifecycle/read/filter/toggle contract **and every other runtime dependency on
`distribution_job*`**, including Publisher Services creation/cancellation and
activation-guard paths.

Publisher-service activation either atomically creates its required generic
event/job intent under the separately activated generic rule or remains
fail-closed. It must not become a jobless activation merely because the legacy
toggle disappears.

A repository-wide source audit and runtime tests must prove the Phase-C binary
does not query/write legacy job tables.

That release is deployed and observed until all supported production rollback
targets are also zero-runtime-access binaries.

### Phase D - contract legacy storage

Only a still-later destructive migration removes `distribution_job*` storage
and queue-only enum types. The migration obtains locks preventing concurrent
legacy writes, performs the final zero-row assertion and DROP under the same
fail-closed transaction, and aborts on any unexpected row/state.

Because the current production GraphQL image runs the default `thoth init`
path, which performs migrations before serving, this separation is mandatory:
the first task in a rolling deploy must never drop relations still needed by old
tasks or by automatic rollback.

The development environment's currently unapplied v1.7.0 state is handled by
ordered migrations: v1.7.0 applies first, followed by additive generic schema,
and only later the separately authorized contraction migration after the same
consumer/runtime retirement gates.

The migration test matrix must include:

- production-like schema with v1.7.0 already applied;
- pre-v1.7 upgrade applying v1.7.0 plus additive generic schema;
- full empty-database migration chain;
- rolling deploy with mixed old/new application versions during Phase A;
- event-route activation only after emission-capable deployment/rollback floor;
- route deactivation before rollback below an event-emission floor;
- current-state reconciliation/backfill covering an authorized emission gap;
- later API/runtime-retirement rollback window;
- proof Phase-C binaries have zero runtime legacy-table access;
- contraction with final locks and zero legacy rows;
- contraction abort with any legacy row present.

Initial worker deployment is inert until job creation/route/handler activation is
separately authorized. Effectful handler activation is prohibited until its
`RECONCILIATION_REQUIRED` exit path is itself available and approved.

High-volume handlers such as indexing/revalidation additionally require
kind-specific retention, coalescing and admission/backpressure limits before
activation.

Rollout should prove in order:

1. schema/event-route/job concurrency without external writes;
2. mixed-version route completeness and out-of-order transaction commits;
3. activation/deactivation and event-emission-floor behaviour;
4. crash/lease renewal/WAITING/reconciliation recovery;
5. effect-scoped idempotency, revision ordering and coalescing;
6. priority/fairness/per-kind concurrency and bounded retry/wait deadlines;
7. one or more non-destructive representative handlers;
8. protected operator controls and reconciliation exit path;
9. one effectful handler with durable `EFFECT_STARTED`, provider checkpoint and
   reconciliation evidence;
10. quarantine-to-trusted-promotion flow for one untrusted-content handler;
11. only then broader handler activation.

Handler activation is independent: enabling Hosting must not automatically
enable Crossref, Algolia, file processing or another family merely because they
share the engine.

## 10. Rollback

Architecture rollback before implementation is ordinary documentation revert.

Once implemented, rollback must distinguish:

- stopping new event/job creation;
- deactivating routes before rolling below their event-emission floor;
- stopping worker claiming;
- preserving durable event/route/job/attempt/checkpoint records;
- rolling back individual handler activation;
- rolling back application binaries while preserving schema compatibility;
- schema rollback, which may be impossible after live async data exists and
  therefore requires its own forward-repair/migration plan.

During the expand/migrate phases, rollback to a previous Thoth image is permitted
only while all schema/API that image needs remains present **and** route
activation remains compatible with that image's event-emission behaviour.

If rollback below an already-active route's emission floor is unavoidable, the
route is deactivated before old writers resume or a specifically authorized
current-state reconciliation/backfill covers the exact gap. Existing
pre-deactivation eligible route obligations remain durable; deactivation does
not delete them.

Legacy BE-04 storage contraction cannot occur until every supported rollback
binary has zero runtime legacy-table access.

After generic async data exists, rolling the runtime back does not authorize
dropping/rewriting that data. A rollback plan must either run a binary that
understands the generic schema or stop job production/claiming and preserve the
durable queue for forward recovery.

Do not treat disabling a worker or deactivating a route as equivalent to deleting
durable jobs/events/route obligations.

## 11. Validation requirements

Before shared implementation can be approved, evidence must include:

- real PostgreSQL concurrent claim tests with multiple workers;
- durable route registration/activation/deactivation tests;
- concurrent event creation versus route activation/deactivation under the
  routing generation/epoch mechanism;
- proof that pre-deactivation eligible events remain owed after route retirement;
- explicit authorized terminal route-disposition tests when an eligible route
  can no longer be materialized;
- event-emission-floor tests proving a route cannot activate while any serving
  or supported rollback binary can perform the mutation without emitting its
  event kind/version;
- rollback-below-emission-floor tests requiring prior deactivation or explicit
  current-state reconciliation/backfill;
- mixed worker-version routing where an older worker cannot silently complete an
  unknown eligible route;
- out-of-order transaction commit tests proving no event can be skipped by a
  sequence/cursor optimization;
- crash during partial fan-out with exact resume and no duplicate route record;
- explicit historical backfill through the same `(event_id, route_key)`
  materialization identity;
- effect-scoped idempotency tests across newer source revisions/schedule slots;
- CURRENT_STATE tests that read canonical latest state and safely no-op/coalesce
  when already applied;
- REVISION_BOUND tests proving an older retried revision cannot overwrite a newer
  already-applied revision without explicit historical replay authorization;
- allowed many-route-to-one-job coalescing tests and deterministic conflict
  attention when coalescing is not permitted;
- current-token lease renewal and refusal to renew superseded/expired claims;
- stale-token refusal for completion, failure, cancellation, checkpointing and
  reconciliation;
- WAITING transition atomically persisting checkpoint and releasing claim/lease;
- resume from WAITING as a new claim/attempt that polls/reconciles rather than
  resubmits;
- pre-write retry-budget exhaustion producing FAILED;
- post-write waiting/deadline exhaustion producing
  RECONCILIATION_REQUIRED rather than FAILED;
- concurrency-key retention across WAITING/RECONCILIATION_REQUIRED unless an
  explicit safe-release rule exists;
- operational signal for jobs blocked behind a retained concurrency key;
- effect-window test proving an effectful runner is not started without enough
  remaining lease/runtime;
- crash before and after `EFFECT_STARTED`;
- durable `EFFECT_CONFIRMED` recovery where supported by the kind, plus proof
  that this evidence is not conflated with ADR-0010 observed-state truth;
- cancellation before write and cancellation after effect-start tests;
- `INDETERMINATE` external-write reconciliation tests;
- claimed/token-fenced/kind-authorized reconciliation race tests;
- priority, `available_at`, bounded attempt and scheduler fairness tests;
- retry/backoff and poison-job exhaustion tests;
- unsupported payload-version refusal;
- payload tests proving credentials are rejected and personal/snapshot data is
  absent unless explicitly allowed by the kind contract;
- diagnostics/provider-evidence retention and erasure tests;
- current-state execution reading canonical state/revision rather than stale
  embedded snapshots;
- graceful shutdown and abrupt restart recovery for every worker pool;
- worker capability/kind/risk-class/revision-mode filtering;
- authorization negative tests proving domain executors cannot mutate another
  domain's jobs;
- protected/audited operator cancel/retry/reconcile tests;
- queryable queue-depth, oldest-ready-age, expired-lease,
  `RECONCILIATION_REQUIRED`, failure, blocked-successor and worker-liveness
  signals;
- proof that logs and durable diagnostics do not contain configured credentials;
- proof that trusted/untrusted worker tasks receive no static AWS access keys;
- proof that untrusted-content worker roles exclude Hosting DNS/ACM/CloudFront
  tenant authority and served-content write paths;
- quarantine/staging tests proving untrusted output cannot directly become
  publisher-served content;
- trusted-promotion tests proving publisher/domain/bucket/key/target are resolved
  from canonical state rather than untrusted result fields;
- negative tests proving an untrusted worker cannot create arbitrary follow-on
  jobs/events;
- if direct DB access is used by an untrusted pool, proof of row/kind enforcement
  through RLS/security-definer functions or an equivalent database boundary;
- Phase-A migration tests retaining legacy BE-04 storage/API;
- `thoth-dissemination` generic-contract compatibility tests;
- `thoth-app` migration tests for latest job/status/filter semantics;
- generated SDL/client contract checks;
- Phase-C repository-wide source/runtime tests proving zero
  `distribution_job*` runtime access, including service-configuration
  activation/disable paths;
- proof AUTOMATIC_PUSH remains fail closed unless generic job/event creation is
  atomically available;
- rolling deployment/rollback tests proving old tasks remain valid during Phase
  A/B and Phase-C rollback targets need no legacy relations;
- contraction migration tests from production v1.7.0 state;
- contraction lock + zero-row assertion in the same fail-closed migration
  transaction;
- contraction abort when any legacy row exists;
- full empty-database migration-chain tests;
- per-kind retention/coalescing/admission tests before high-volume family
  activation;
- deployment tests with more than one compatible worker task even if production
  initially runs one.

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
