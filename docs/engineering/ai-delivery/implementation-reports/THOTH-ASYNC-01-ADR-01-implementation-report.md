# THOTH-ASYNC-01-ADR-01 implementation report

This report records the bounded ADR-0012 architecture-authoring task, its
independent-review correction rounds, approval-state reconciliation and the
later programme-integration compatibility reconciliation. GitHub issues
#957/#958 remain the live authority for lifecycle state; this committed report
records durable authoring and correction evidence without asserting the current
live review or merge gate.

## Identity

```text
Programme: THOTH-ASYNC-01
Owning issue: thoth-pub/thoth#958
Parent programme: thoth-pub/thoth#957
Repository: thoth-pub/thoth
Task: THOTH-ASYNC-01-ADR-01
Risk: CRITICAL
Original workflow: STANDARD; delivery topology later amended to PROGRAMME_INTEGRATION
Original authorized base: develop @ 923545d5c9028bc04c40e38efeb7de674efed3fd
Programme integration base: feature/worker @ 345a7a04131e7c0539f7518c6ea457fa5cb6a462
Task branch: feature/async/adr-0012
Current PR target: feature/worker
```

## Authorized write footprint

The original architecture-authoring task used four paths. Approval-state
reconciliation expanded the cumulative durable ADR task footprint to exactly
six repository paths:

1. `CHANGELOG.md`;
2. `docs/engineering/ai-delivery/implementation-reports/THOTH-ASYNC-01-ADR-01-implementation-report.md`;
3. `docs/engineering/decisions/ADR-0008-machine-roles-and-durable-job-primitives.md`;
4. `docs/engineering/decisions/ADR-0010-staff-operations-console.md`;
5. `docs/engineering/decisions/ADR-0012-shared-asynchronous-event-and-job-execution.md`;
6. `docs/engineering/decisions/decision-register.md`.

No runtime source, migration, GraphQL implementation, workflow or
infrastructure path is changed by this ADR task.

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

## Independent review round 3

The independent CRITICAL review of
`2d5eb1ae1e25bc222c4ca94a4f6dc5e81ecceba8` returned
`CHANGES REQUIRED`; durable receipt: #958 comment `5911717880`.

Four blockers were accepted:

- R3-01 HIGH - route activation had no durable deactivation/retirement boundary
  and was not gated on every serving/rollback binary emitting the consumed event
  kind;
- R3-02 MEDIUM - Publisher Services still accessed `distribution_job*` outside
  the named GraphQL surfaces, making later DROP unsafe during rolling deploy and
  rollback;
- R3-03 MEDIUM - untrusted-content isolation still allowed served-object writes
  or trusted-handler steering through untrusted destination data;
- R3-04 MEDIUM - job kinds did not declare current-state versus revision-bound
  semantics, permitting an older retried revision to overwrite newer applied
  state.

## Round-4 corrections

### Route activation, retirement and emission floor

Routes now have durable activation and optional deactivation boundaries under the
same routing-generation mechanism.

Activation/deactivation is separately authorized and audited rather than a side
effect of deploying handler code. Events eligible before deactivation remain
owed after route retirement; route registrations are never deleted to erase
outstanding obligations. An eligible route that cannot be materialized requires
an explicit authorized/audited terminal route disposition.

A route for event kind/version K may activate only after every serving binary
and supported rollback binary capable of the relevant business mutation emits K
atomically. Rollback below that emission floor requires prior deactivation or an
explicitly authorized current-state reconciliation/backfill covering the exact
gap.

### Revision semantics

Every job kind now declares either:

```text
CURRENT_STATE
REVISION_BOUND
```

CURRENT_STATE jobs read current canonical state when they execute and may safely
coalesce/no-op under their declared rule.

REVISION_BOUND jobs bind to a retained revision/fingerprint and must not apply an
older revision after a newer revision of the same effect target has already been
applied unless the operation is an explicitly authorized historical
replay/rollback.

### Full BE-04 runtime retirement before contraction

The Phase-C retirement release now covers every runtime access to
`distribution_job*`, not only named legacy GraphQL fields.

The ADR explicitly includes the current
`replacePublisherServiceConfiguration` runtime dependencies:

- automatic back-catalogue creation;
- the fail-closed AUTOMATIC_PUSH activation guard;
- assignment-disable legacy-job cancellation.

Those paths must be redirected to the generic lifecycle or remain conservatively
fail-closed before the legacy tables can be dropped.

A repository-wide source/runtime audit must prove the deployed and supported
rollback Phase-C binaries have zero runtime legacy-table access.

Phase D then performs only the storage DROP plus removal of remaining inert
schema/model declarations. Its migration takes locks preventing a concurrent
legacy write, performs the final zero-row assertion and DROP in the same
fail-closed migration transaction, and retains `distribution_platform`.

### Untrusted-content quarantine and trusted promotion

UNTRUSTED_CONTENT remains part of the same canonical async engine but executes
in a separate minimal-authority pool.

It may read authorized input and write only quarantine/staging output. It cannot
write served publisher/Hosting locations or choose final domain/bucket/key/
Hosting targets.

Untrusted completion results are explicitly untrusted data. A trusted promotion
job validates the artifact and resolves all authority-bearing destination values
from canonical Thoth state before publishing.

The untrusted worker may report only its declared completion/evidence facts and
cannot create arbitrary downstream work.

If direct database access is chosen instead of the protected kind-scoped
executor API, ordinary table grants are insufficient: the separately reviewed
database boundary must enforce row/kind scope through RLS, security-definer
functions/procedures or an equivalent mechanism.

### Additional safety clarifications

Round 4 also records that:

- `EFFECT_CONFIRMED` is anti-replay execution evidence and is not by itself
  ADR-0010 observed publication/acceptance/current-state truth;
- attempt diagnostics/provider evidence are covered by data
  minimization/retention/erasure rules;
- jobs blocked behind WAITING/RECONCILIATION_REQUIRED concurrency keys must be
  operationally visible;
- high-volume job families require retention/coalescing/admission/backpressure
  policy before activation;
- graceful-shutdown rules apply to every worker pool.

## Independent review round 4

The independent CRITICAL review of
`ced4cb19224d7acc1824863a85d83b2387617449` returned `CHANGES REQUIRED`.

Three MEDIUM blockers were accepted:

- R4-01 - durable route obligations lacked route-level backlog/materializer
  visibility and could be terminally dispositioned without explicit divergence
  attention;
- R4-02 - UNTRUSTED_CONTENT IAM/database separation did not prevent cross-tenant
  persistence or network access to shared private services;
- R4-03 - stale-write protection and concurrency were still kind-scoped rather
  than shared by every job kind mutating the same external effect target.

The accepted non-blocking clarifications also covered the BE-04 migration-backfill
runtime path, queue-specific error-contract retirement, canonical writes outside
normal event-emitting binaries, safe job-kind retirement, explicit upstream
Publisher Services generic creation ownership and contraction-revert semantics.

## Round-5 corrections

### Route obligations and materializer health

The ADR now requires per-route count/oldest-age signals for eligible events with
no disposition, explicit visibility when no compatible materializer is deployed,
and route activation only after both the event-emission floor and a compatible
materializer exist. A terminal route disposition without covering
reconciliation/backfill is an accepted divergence requiring durable attention;
bulk actions still create one audited disposition per event/route.

### Canonical external effect targets

Effectful kinds now resolve domain-owned canonical external effect-target
identities independent of job kind. Every writer of one target shares the same
target-scoped concurrency namespace and durable applied-revision/fingerprint
evidence. Batch jobs decompose by target by default; retained multi-target jobs
must acquire complete target serialization deterministically and maintain
per-target evidence.

### Stronger untrusted-content isolation

UNTRUSTED_CONTENT execution now requires cross-publisher process/state isolation,
job-scoped staging authority, independently verified trusted promotion and a
deny-by-default network boundary. The pool cannot inherit private-subnet access to
Redis, EFS, unrelated RDS/database endpoints or other internal services merely
because IAM/database credentials are restricted.

### Retirement and rollback clarifications

The ADR now names the current migration-backfill runtime path and released
`DISTRIBUTION_JOB_CREATION_DISABLED` compatibility surface in Phase C, requires
non-emitting migration/repair writes to emit transactionally or receive exact-scope
reconciliation/backfill, prevents job-kind retirement from orphaning non-terminal
work, assigns the Publisher Services generic creation/cancellation/activation
contract upstream to `thoth`, and makes Phase-D contraction a forward-repair
boundary rather than an automatic restoration of BE-04.

### Validation additions

Round 5 adds explicit validation for route backlog/materializer health, accepted
divergence, cross-kind target serialization/evidence, provider-without-revision
ordering, cross-publisher untrusted isolation, per-job staging, private-network
denial, trusted staging provenance, job-kind retirement, non-binary domain writes,
migration-backfill/error-contract retirement and contraction-revert behaviour.

## Independent review round 5

The independent CRITICAL review of
`d25bb1ecd1665cc4e635b16dbd5fd09582a5889b` returned `CHANGES REQUIRED`.
R4-01 through R4-03 were verified closed with no regression identified in the
earlier F/R2/R3 findings.

One MEDIUM blocker was accepted:

- R5-01 - trusted promotion validated a staged object but did not bind
  publication to the exact immutable bytes/version validated or require the
  untrusted writer's authority to end first, leaving a validate-then-swap TOCTOU
  path.

The missing round-4/round-5 lifecycle chain was reconciled in GitHub under #958
comment `5912886635`; the programme gate was reconciled under #957 comment
`5912887392`.

## Round-6 corrections

### Trusted promotion integrity

Untrusted staging-write authority is now current-claim-scoped and bounded in
lifetime. Trusted validation cannot begin until write authority over the selected
artifact has ended (or an unrevocable capability has expired / an immutable
provider-version boundary is established). The trusted side seals and resolves
the artifact identity, independently validates it, and may publish only the exact
immutable bytes/version that passed validation. A mutable key cannot bridge the
validation/publication boundary; any replacement requires fresh validation.

### Effect-target fencing and identity

Target evidence is now current-claim-fenced, updated only while holding the
target serialization boundary and committed atomically with the durable local
effect/job outcome. It is monotonic except under authorized historical replay.
CURRENT_STATE fingerprints cover every canonical input determining the effect.
Exactly one domain owns a shared effect-target identity; identity changes have
explicit old/new-target disposition rules; multi-target locking is all-or-nothing
rather than waiting while holding a subset.

### Untrusted execution and egress

The isolation rule is job-to-job, including jobs for the same publisher. Staging
capabilities are claim-bound and excluded from durable payload/attempt records.
The deny-by-default network boundary explicitly covers outbound traffic, limits
object storage to approved Thoth input/staging locations and requires explicit
allowlisting of other dependencies.

### Operator and validation tightening

Staff-created terminal route dispositions use ADR-0010's protected audited command
seam. Validation now covers complete CURRENT_STATE fingerprints, target-evidence
claim fencing and monotonicity, target-identity transitions, all-or-nothing
multi-target locking, same-publisher job isolation, staging-capability expiry,
egress restrictions and staged-object replacement after validation.

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

## Programme-integration compatibility reconciliation

After ADR-0013 became repository-authoritative and THOTH-ASYNC-01 explicitly
opted into programme-integration reliance, the programme created
`feature/worker` from current `develop` and retargeted PR #960 to that
programme integration branch.

The compatibility reconciliation is bound to:

```text
reviewed ADR-0012 source before reconciliation:
bd5223deee44b465dd860ff96b75d81ad41cbf2d

programme integration base:
feature/worker @ 345a7a04131e7c0539f7518c6ea457fa5cb6a462

historical common base:
923545d5c9028bc04c40e38efeb7de674efed3fd
```

The histories had diverged. The only content changed on both sides since the
historical common base was `CHANGELOG.md` and
`docs/engineering/decisions/decision-register.md`.

The reconciliation preserves the reviewed ADR-0012, ADR-0008 and ADR-0010
decision blobs byte-for-byte. It carries the repository-authoritative ADR-0013,
branching workflow and operating-model doctrine from `feature/worker`
unchanged. Manual text reconciliation is limited to:

- `CHANGELOG.md`: retain the ADR-0013 control entry and both ADR-0012
  proposal/approval entries;
- `docs/engineering/decisions/decision-register.md`: retain ADR-0013 and its
  programme-integration reliance rule while restoring the approved ADR-0012
  row;
- this implementation report: reconcile durable task topology and approval
  state without copying transient live review or merge status into source.

ADR-0012 remains not repository-authoritative until its own authority condition
is satisfied on `develop`. Under ADR-0013, programme-local reliance is also
not effective merely because this reconciliation exists: the exact reconciled
ADR-0012 version must first merge into `feature/worker`, and the programme
ledger must record the merge/reachability evidence required by ADR-0013
eligibility conditions 7 and 8.

Any source head created by this reconciliation requires fresh independent
CRITICAL exact-head review against the exact `feature/worker` base before
merge authorization.

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

- the original authorized base remains recorded as historical authoring evidence;
- ADR-0012 is `APPROVED`, with Javi as approver and approval date 2026-09-30;
- PR #960 exists and its programme target is `feature/worker`;
- the programme integration base for this reconciliation is
  `345a7a04131e7c0539f7518c6ea457fa5cb6a462`;
- ADR-0012, ADR-0008 and ADR-0010 decision bytes are preserved from the
  previously reviewed ADR source head;
- the final candidate against `feature/worker` is required to remain exactly
  the cumulative six-path ADR task footprint;
- no runtime test result is claimed by this documentation/control-only task;
- live candidate head, CI, review and merge evidence belongs in GitHub #957/#958
  rather than in a self-invalidating committed status line.

Independent review is valid only against the exact candidate head it names.

## Deviation history

During initial issue creation, the GitHub connector omitted the issue number in
its normalized create response, briefly producing a `#undefined` parent
reference in #958. The returned URLs established #957/#958 and the issue was
corrected before branch/source mutation.

No source-scope deviation remains.

## Remaining lifecycle gates

1. fresh independent CRITICAL exact-head review of the reconciled PR #960
   candidate against exact programme base `feature/worker @ 345a7a04131e7c0539f7518c6ea457fa5cb6a462`;
2. explicit CTO merge authorization only if that review returns `APPROVED`;
3. merge PR #960 into `feature/worker`;
4. record the exact ADR-0012 merge/reachability evidence required by ADR-0013
   eligibility conditions 7 and 8;
5. only after programme-local reliance is effective, separately specify,
   authorize, implement and independently review each repository-local async
   slice and any pinned cross-repository preview dependency;
6. retain separate migration execution, provider/IAM/runtime, deployment,
   release and production-activation gates.

Merging ADR-0012 into `feature/worker` does not make ADR-0012
repository-authoritative on `develop`, does not itself authorize
implementation and does not authorize any external effect.

This report authorizes none of those later actions.
