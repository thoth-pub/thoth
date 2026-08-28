# ADR-0010 - Staff Operations Console and canonical publisher-service operations

Status: PROPOSED
Date: 2026-08-27
Decision owner: CTO
Programmes affected: Staff Operations Console; Publisher Services and Distribution Configuration; OAI-PMH; Thoth Metrics; shared backend/authorization architecture
Repositories affected: `thoth-pub/thoth`, `thoth-pub/thoth-app`, `thoth-pub/thoth-dissemination`; future contract impact may include `thoth-pub/thoth-sphinx`, `thoth-pub/metrics-dashboard`, `thoth-pub/metrics-widget` and other verified Thoth GraphQL consumers
Owning programme issue: [#846](https://github.com/thoth-pub/thoth/issues/846)
Architecture task: `ADM-ADR-01`
Verification base: `develop @ 91dd607d674fcb9a75c8df31891eea0355c9ca84`
Supersedes: None
Superseded by: None

Decision: establish an admin-first Staff Operations Console with a deliberately
entered publisher operating context; make `thoth` the canonical durable owner of
staff-facing publisher-service operational records; separate desired state,
execution, observed external state and attention/reconciliation state; introduce
a small cross-domain operational/audit seam without creating a generic job/queue
or universal platform abstraction; deliver distribution as the first deep
operational integration; and activate staff replay or repair controls only after
read-only operational truth and observed-state semantics are proven.

This record is a proposed architecture decision. It records no deployed state and
authorizes no implementation, merge, migration, provider access, deployment,
external write, replay, release or production activation.

Authority condition: under the repository ADR process, this record is usable as
architecture only after the exact decision content has been independently
reviewed, explicitly approved by the decision owner, and is reachable from the
repository's authoritative integration branch. Live review, authorization, CI
and merge evidence belongs in GitHub rather than being copied into this file.

---

## 1. Context

### 1.1 The current staff experience is publisher-first

`thoth-app` is the authenticated publisher and staff management UI. Its current
route hierarchy places ordinary publisher-workspace pages under `/admin`, even
though the application now has its own `admin.thoth.pub` host. Superuser-only
publisher administration is consequently an additional staff surface inside a
shell whose primary state is still an active publisher.

The current active-publisher flow also attempts to maintain an active publisher,
including restoring a persisted publisher and falling back to the first
authorized publisher. That is useful for ordinary publisher users but is the
wrong default for a global staff control plane: a superuser must be able to be in
Admin with no publisher selected.

The target product therefore requires an architectural inversion rather than a
new table bolted onto the existing shell.

### 1.2 Publisher Services provides desired configuration and durable work, not complete operational truth

Publisher Services makes Thoth authoritative for publisher package and explicit
distribution-platform configuration and provides durable back-catalogue jobs.
Its approved design deliberately separates desired state from durable work and
defers complete per-work/per-platform observed delivery state, general metadata
change events, withdrawals and periodic reconciliation.

The existing durable distribution-job model is programme-local. It records job
and attempt lifecycle state; worker-reported success is not proof that an
external destination accepted, published or retained metadata/files. The current
staff report correctly avoids presenting such success as observed delivery.

A staff control plane that must answer "what did Thoth do, for whom, where, why,
with what outcome, and what requires attention now?" therefore needs a canonical
operational layer in addition to desired configuration and execution queues.

### 1.3 The console spans several service domains

The initial product boundary is broader than distribution alone. It covers
publisher-facing services Thoth operates, executes, hosts or delivers on behalf
of a publisher, together with their entitlement/configuration, operational state,
history, failures and authorized staff interventions.

Relevant domains include:

- push dissemination and preservation destinations;
- Crossref DOI deposits as a distribution-domain metadata action;
- pull/feed services such as the shared OCLC KBART feed used for OCLC KB and
  Ex Libris assignments;
- OAI-PMH entitlement/exposure state;
- Thoth-managed publication-file and cover hosting/CDN state;
- later Metrics operational projections.

These domains do not share one lifecycle. The Staff Operations Console needs a
common way to observe and audit them without flattening their domain truth.

### 1.4 Existing cross-programme decisions constrain the solution

[`ADR-0002`](ADR-0002-platform-domain-boundaries.md) requires
`DistributionPlatform` and `MetricPlatform` to remain separate domain types with
no name-based conversion or initial inferred mapping.

[`ADR-0008`](ADR-0008-machine-roles-and-durable-job-primitives.md) requires
least-privilege domain-specific machine roles and explicitly keeps Publisher
Services `distribution_job*` tables/types/APIs programme-local. Its durable-work
primitives are shared conventions, not a shared job framework. A later programme
must not reuse `distribution_job*` merely because its own work has a similar
shape.

[`ADR-0005`](ADR-0005-terminal-merge-evidence.md) keeps transient review,
authorization, CI and merge evidence in GitHub.

[`ADR-0009`](ADR-0009-programme-integration-branch-namespace.md) governs
repository branch namespace where programme-integration workflows are later
adopted.

This ADR must extend those decisions compatibly rather than silently weaken them.

---

## 2. Decision drivers

1. Give staff one trustworthy operational view across publishers without making
   infrastructure logs or provider dashboards the system of record.
2. Make the superuser experience genuinely admin-first while preserving one
   canonical publisher workspace.
3. Preserve actor identity and authorization when a superuser works inside a
   publisher context; do not implement identity impersonation.
4. Preserve the difference between desired configuration, attempted work and
   observed external state.
5. Support complete historical audit without rewriting failed history after a
   successful repair or replay.
6. Make retry, replay and current-state redistribution unambiguous.
7. Fail closed where an external side effect is indeterminate or duplicate safety
   cannot be established.
8. Keep workers as executors of approved work rather than decision makers about
   what work should exist.
9. Keep service-domain truth local to the domain while giving Admin a small common
   read/audit vocabulary.
10. Preserve ADR-0002's Distribution/Metrics type separation.
11. Preserve ADR-0008's programme-local job models and machine-role boundaries.
12. Let the console grow from publisher-service operations toward a broader staff
    control plane without requiring another shell or routing inversion.
13. Deliver useful UI and read-only operational value before activating commands
    that can cause external writes.
14. Keep cross-repository contracts additive and explicitly ordered so downstream
    repositories never guess unmerged upstream behavior.

---

## 3. Options considered

### 3.1 Application-shell options

#### Option A - Explicit dual-mode shell with one publisher workspace

Use `/admin/*` only for the global superuser control plane. Move the publisher
workspace to non-`/admin` routes. A superuser explicitly enters a publisher
operating context and then uses the same core publisher workspace as an ordinary
publisher user.

Advantages:

- route structure reflects product semantics;
- new superuser sessions can start with no publisher selected;
- one canonical publisher workspace avoids duplicated routing/components;
- the operating context can be visibly and auditably distinct from identity;
- future global operational capabilities have a stable namespace.

Disadvantages:

- requires a deliberate route/context migration in `thoth-app`;
- legacy publisher bookmarks under `/admin` stop being ordinary publisher routes.

Decision: Selected.

#### Option B - Nest staff publisher workspaces under `/admin/publishers/:id/...`

Advantages:

- publisher identity is explicit in every staff URL.

Disadvantages:

- duplicates the publisher address space or significantly complicates routing;
- encourages a second copy of the publisher workspace;
- weakens link sharing and parity between publisher and staff views.

Decision: Rejected.

#### Option C - Keep the current `/admin/...` publisher routes and add another Admin sub-namespace

Advantages:

- lowest immediate route churn.

Disadvantages:

- preserves the legacy semantic collision indefinitely;
- `/admin` would continue to mean both publisher workspace and superuser control
  plane;
- future operational navigation remains awkward and role-dependent.

Decision: Rejected.

### 3.2 Operational-data options

#### Option D - Generalize Publisher Services `distribution_job*` into the universal model

Advantages:

- appears to reuse existing durable-work machinery.

Disadvantages:

- directly conflicts with ADR-0008's programme-local boundary;
- a job is not the same thing as observed external state or audit history;
- feed, OAI-PMH, hosting and Metrics actions have different lifecycles;
- would create a generic job framework from one programme by precedent.

Decision: Rejected.

#### Option E - Create a small common operational/audit envelope plus domain-owned truth

Use a common operation/attempt vocabulary for staff activity/audit and attach it
to domain-specific desired-state and observed-state models.

Advantages:

- gives Admin a stable cross-domain seam;
- does not require one execution model;
- preserves ADR-0002 and ADR-0008;
- supports consistent actor, correlation, retry/replay and attention links;
- allows domain-by-domain adoption.

Disadvantages:

- requires projection/integration work in each domain;
- similar domain concepts may intentionally remain represented differently.

Decision: Selected.

#### Option F - Adopt a universal event-sourced control plane immediately

Advantages:

- maximally general long-term architecture.

Disadvantages:

- much broader than the present need;
- would force event semantics, migrations and replay rules across unrelated
  domains before evidence exists;
- delays useful distribution/admin delivery.

Decision: Rejected as premature.

---

## 4. Decision

### 4.1 Admin-first application modes

`thoth-app` will have two explicit operating modes.

#### Admin mode

The `/admin` namespace is reserved for superusers and global/cross-publisher
operations. The target information architecture begins with:

```text
/admin
/admin/publishers
/admin/activity
/admin/attention
/admin/reports
```

Not every route must ship in the first app slice; the namespace and ownership are
normative.

Every new authenticated superuser session starts in Admin with **no active
publisher operating context**. Global Admin queries must not depend on whichever
publisher was last viewed.

A non-superuser who requests `/admin` or `/admin/*` receives an authorization
denial rather than a silent publisher-space redirect. The target presentation is:

```text
Access denied
You don't have permission to access this page.

[Go to dashboard]
```

The presentation must not reveal staff/superuser role details or describe what
exists behind the boundary. Backend authorization remains independent of this
route guard.

#### Publisher workspace

The normal publisher workspace moves out of the legacy `/admin` prefix. Target
routes include:

```text
/dashboard
/works
/works/new
/works/:workId
/series
/sets
/publisher
```

Ordinary publisher users use these routes normally. Superusers use the same core
workspace only after entering a publisher operating context.

The old publisher `/admin/...` namespace is retired rather than maintained as a
permanent alias. A migration task may define a finite compatibility mapping for
specific unambiguous legacy URLs, but a blanket `/admin/* -> /*` redirect is
prohibited because `/admin` and its descendants become valid superuser space.

### 4.2 Publisher operating context is not identity impersonation

A superuser entering a publisher workspace remains authenticated as that
superuser. The selected publisher is a separate operating context.

The context:

- is entered explicitly from Admin, including from a publisher directory or a
  cross-publisher operational item;
- survives ordinary navigation and browser refresh during the authenticated
  session;
- does not survive a new authenticated session;
- is cleared when the user returns to Admin;
- is visibly indicated in the publisher shell with a persistent publisher
  context treatment and a clear `Return to Admin` action;
- does not itself grant backend permission.

Publisher-user active-publisher selection and superuser publisher operating
context have different lifecycle semantics. They may share low-level state/cache
machinery, but they must not be represented by one persistence policy. In
particular, a superuser must not auto-select the first available publisher on a
new session.

The same publisher scope does not imply the same permissions or navigation. A
superuser in publisher context may receive an additional staff-only `Services`
operational area while ordinary publisher users continue to see the normal
publisher workspace.

### 4.3 Admin is an operational control plane

The target global Admin information architecture is:

- **Dashboard** - cross-publisher operational summary, current attention and
  recent activity;
- **Publishers** - publisher directory, package/service headline state and entry
  into publisher context;
- **Activity** - chronological operational record across publishers;
- **Attention** - conditions that currently warrant human consideration;
- **Reports** - derived/exportable operational reporting.

Admin may summarize publisher data across publishers. Detailed publisher data
and publisher-scoped intervention controls require deliberate entry into the
publisher operating context.

Selecting an operational row must carry explicit publisher and operation
identifiers and perform two deliberate client actions: enter the relevant
publisher context, then navigate to the relevant publisher-scoped detail. Merely
rendering or fetching an Admin row must never mutate publisher context.

### 4.4 `thoth` owns the canonical operational record used by Admin

If an action is important enough for staff to monitor, report, retry/replay or
explain to a publisher, its authoritative operational state must be available
through a canonical Thoth-owned API/model.

Worker logs, GitHub Actions, Railway/provider logs, object listings and external
provider dashboards may be diagnostic evidence, but they are not the sole
canonical Staff Operations record.

The architecture introduces a small common operational envelope, conceptually
named `ServiceOperation`, with one or more attempts beneath an operation. The
exact table/type/GraphQL names and field storage are bounded implementation
decisions, but the common semantics must support at least:

- publisher association where applicable;
- service domain and service/action identity without a universal platform enum;
- operation scope/subject references;
- trigger/initiator and authenticated actor where applicable;
- lifecycle timestamps/status;
- correlation/idempotency relationships;
- parent/replay relationships;
- safe error/evidence references;
- attempt history.

This common seam is for operational activity/audit and staff projections. It is
**not**:

- a generic queue;
- a universal job framework;
- a universal executor API;
- a replacement for programme-owned durable work models;
- authority to reuse Publisher Services `distribution_job*` tables/types/APIs;
- a universal `Platform` abstraction.

ADR-0008 therefore remains fully binding. A domain may have both programme-local
work/queue state and a canonical staff-facing operational record when those are
different concepts.

### 4.5 Desired state, operational work and observed state are distinct

The architecture preserves three separate questions:

```text
desired state
    what should apply or be delivered

operational work
    what Thoth tried to do, including attempts

observed state
    what can truthfully be established about the resulting external/service state
```

No layer may silently infer the next from the previous unless a domain contract
explicitly proves that implication.

Examples:

- worker-reported `SUCCEEDED` does not by itself prove that an external
  destination accepted or published metadata;
- successfully generating/publishing a feed does not prove that a consumer
  ingested it;
- an OAI-PMH endpoint being healthy does not prove that every entitled work is
  correctly exposed;
- an object upload completing does not mean the object has been disseminated.

The first deep observed-state implementation is distribution, eventually
answering per work/destination:

- whether delivery is desired;
- the current source revision/fingerprint;
- the latest relevant operation/attempt;
- observed acceptance/publication/evidence where available;
- last delivered revision/fingerprint where knowable;
- reconciliation state such as current, stale, missing, indeterminate or
  mismatch.

Exact domain enums and evidence rules remain bounded implementation decisions.

### 4.6 Attempts, retry, replay and current-state redistribution

An operation may have multiple execution attempts.

**Retry** means another attempt for the same logical operation and intended
effect. Automatic/transient retries remain worker/domain lifecycle behavior.

**Replay** means creating a new logical operation explicitly linked to a
historical terminal operation.

**Current-state redistribution / run again** means creating a new operation from
the current desired/source state rather than reproducing the historical
operation's source/effect.

These are not aliases. For example, if an operation for source revision A fails
and the source later changes to revision B, replaying A and distributing current
state B are different commands with different audit meaning.

Historical operations remain immutable facts. If `OP-100` failed and `OP-101`
replayed it successfully, `OP-100` remains failed and `OP-101` records the
successful later action.

### 4.7 Staff commands are backend-owned, audited and fail closed

The Admin application never calls adapters/workflows directly. A staff
intervention is a named backend command that performs authorization, lifecycle
validation, domain eligibility checks and idempotency/duplicate-safety checks
before creating or changing durable work.

The target command families include, where supported by a domain:

- cancel eligible pending/running work;
- replay a terminal historical operation as a new linked operation;
- redistribute/run current state as a new operation;
- automatic retry remains the owning worker/domain lifecycle, not a generic staff
  "try again" alias.

Staff-triggered external-write commands require the authenticated actor and a
short staff reason. The durable audit relationship must make it possible to
answer who requested a deliberate external write, when and why.

Command submission must be idempotent at the operation boundary so browser
retries/double clicks cannot create duplicate logical external operations.

`INDETERMINATE` external effect is semantically different from known failure. If
Thoth sent a request but cannot establish whether the external side effect
occurred, replay is denied unless service-specific semantics prove that a repeat
is safe. Reconciliation/evidence gathering may be required before a command is
available.

The backend should expose command availability and denial reasons in its
protected operational contract so `thoth-app` does not independently reproduce
domain lifecycle policy.

### 4.8 Authorization remains independent of client context

The `/admin` route boundary and publisher operating context are client-side
presentation/navigation mechanisms, not authorization grants.

For publisher-scoped staff commands the backend independently verifies at least:

- caller authorization;
- target publisher and operation relationship;
- action validity for the current lifecycle state;
- current service/platform eligibility and relevant desired state;
- equivalent in-flight/idempotent operation conflicts;
- service-specific duplicate/external-effect safety.

Machine execution and staff administration stay distinct. Workers execute
approved work with their domain-specific machine role and authorization matrix;
they do not decide that arbitrary staff-requested work should exist. Superuser
status does not automatically imply a machine role, and machine-role possession
does not imply staff administrative authority, consistent with ADR-0008.

### 4.9 Service domains attach to the common seam without losing ownership

#### Distribution

Distribution is the first deep integration.

It owns push-delivery semantics, per-work/per-destination observed delivery and
reconciliation rules. During the current transition, `thoth-dissemination`
remains the external execution engine and reports precise attempt/evidence facts
through a merged/pinned Thoth contract.

Crossref remains a distribution-domain destination/action for DOI deposit and
must not be double-recorded in a separate universal DOI-operation subsystem.

Linked destination/adapter semantics such as OAPEN/DOAB remain governed by the
approved DistributionPlatform architecture and must not result in duplicate
external execution.

#### Pull/feed services

OCLC KB and Ex Libris assignments must not be forced into push distribution
jobs. Their shared OCLC KBART feed has domain-specific generation/publication
state. Successful feed publication does not imply downstream consumer ingestion
unless explicit consumer evidence exists.

#### OAI-PMH

OAI-PMH operational state is domain-specific and may include entitlement,
exposure/eligibility projection and endpoint health/reconciliation. Raw external
harvest requests remain observability rather than canonical `ServiceOperation`
records unless a later product requirement explicitly promotes them.

#### Hosting

Thoth-managed publication files and covers are a future Hosting domain with
canonical asset state and meaningful operational history. Asset upload/storage
state remains distinct from dissemination state.

#### Metrics

Thoth Metrics remains governed by its own programme and approved design. The
Staff Operations Console may later consume Metrics-owned operational projections
through an explicit approved cross-programme contract.

This ADR does not:

- merge `MetricPlatform` with `DistributionPlatform`;
- infer a mapping from names/codes;
- define Metrics source/account/checkpoint or ingestion state;
- reuse Publisher Services `distribution_job*` machinery for Metrics;
- change Metrics entitlement, rollup, import or OPERAS architecture.

Any future cross-domain platform relationship still requires the concrete
mapping design required by ADR-0002.

### 4.10 Purpose-built read projections

The existing staff publisher-service configuration report remains useful as a
transitional cross-publisher read, but the operational console must not grow by
indefinitely overloading one publisher-summary query.

Protected global Admin projections should eventually provide:

- operational summary/counts;
- paginated/filterable service operations;
- attention items;
- Admin publisher summaries.

Publisher-context projections should eventually provide:

- publisher operational overview;
- complete operation/attempt/evidence history;
- distribution work-by-destination state;
- backend-authoritative available commands.

Pagination, date windows, filters, stable ordering, authoritative counts and
large export behavior belong at the backend/API boundary. The browser must not
fetch the complete operational ledger and filter it locally.

The same core publisher workspace is shared between publisher users and
superusers in publisher context, but staff-only operational routes/controls may
be present for an authorized superuser. Same publisher data scope does not mean
same identity, permissions or controls.

### 4.11 Attention is separate from operation lifecycle

Operation lifecycle describes what happened to work. Attention describes whether
something currently needs human consideration.

A common read projection must be able to distinguish, for example:

- a failed attempt that is automatically retrying and needs no staff action;
- a terminal failure after retry exhaustion;
- an operation that is running beyond a domain-defined expected window;
- an indeterminate external side effect;
- a remote rejection;
- desired/observed mismatch or stale delivery;
- a missing required asset;
- a blocked service configuration;
- reconciliation failure.

Attention reasons are backend/domain-derived, not frontend interpretations of
status strings.

Resolving an attention condition never rewrites historical operation/attempt
facts. A later successful replay or confirmed observed state resolves current
attention while preserving the original failure/uncertainty record.

### 4.12 Reconciliation is first-class and initially report-only

For distribution, reconciliation eventually compares:

```text
desired state
    + current source/revision
    + observed delivery/evidence
    -> reconciliation projection
```

Possible domain results include current, stale, missing, indeterminate and
mismatch. Exact values/evidence rules are fixed by the bounded distribution
observed-state specification.

Reconciliation actions themselves produce canonical operational evidence. The
initial reconciler reports discrepancies and may create attention items; it does
not automatically repair external state.

Automated repair requires a later explicit architecture and production
activation decision with its own risk classification, dry-run/comparison,
monitoring, rollback and stop conditions.

### 4.13 Error and evidence handling

Canonical operations may retain:

- stable machine-readable error classifications;
- bounded staff-safe error summaries/details;
- safe remote submission/job/reference identifiers;
- evidence needed for reconciliation and audit.

Arbitrary raw worker logs and provider payloads are not canonical Admin data.
Credentials, secrets and sensitive transport details must be excluded or
redacted **before** durable operational recording rather than relying on the UI
to hide them.

Unavailable data must remain distinguishable from a real zero/healthy state.
Partial domain failure must not make the global dashboard look falsely healthy.

---

## 5. Repository ownership and contract impact

### 5.1 `thoth-pub/thoth`

Verified repository responsibility includes the canonical PostgreSQL domain,
migrations, GraphQL API and authorization policy.

This repository therefore owns, when separately implemented:

- the common canonical operational/audit seam;
- protected Admin operational read/command contracts;
- distribution observed-state/reconciliation models owned by the backend;
- backend authorization and idempotency checks for staff commands.

No implementation follows from this ADR alone.

### 5.2 `thoth-pub/thoth-app`

Verified responsibility: authenticated publisher and staff management UI for
Thoth metadata and administrative workflows.

This repository owns, through separately authorized tasks:

- Admin-first routing/shell;
- superuser publisher operating-context lifecycle and cache safety;
- global Admin Dashboard/Publishers/Activity/Attention/Reports UI;
- publisher-context staff Services UI and command presentation.

It remains a generated GraphQL consumer and must wait for a merged or explicitly
pinned preview of upstream contracts before consuming new schema.

### 5.3 `thoth-pub/thoth-dissemination`

Verified responsibility: external metadata/file distribution and preservation
execution engine.

During migration it remains an executor and, through separately authorized work,
reports precise execution/remote evidence to the canonical backend contract. It
must not independently create arbitrary staff work, become the canonical Staff
Operations datastore or guess an unmerged upstream contract.

### 5.4 Other verified GraphQL consumers

The repository contract map currently identifies `thoth-app`, `thoth-pyramid`,
the standalone `thoth-client`, `metrics-dashboard`, `metrics-widget`, `baboon`
and same-repository export code as consumers of the Thoth GraphQL contract in
various forms. Each future API task must assess all verified consumers for its
specific schema/authorization/semantic change.

The intended operational API is additive and protected. This ADR approves no
breaking public schema change and no anonymous operational data exposure.

### 5.5 Metrics and Sphinx

Metrics programme issue #766 remains independent. `thoth-sphinx` is a planned
future GraphQL client under the approved Metrics architecture, not authority for
this programme to invent Metrics contracts.

A later Metrics/Admin integration requires a separately approved cross-programme
contract. Downstream consumers must not guess unmerged Metrics or Staff
Operations contracts.

---

## 6. Invariants

1. `/admin/*` is the global superuser control-plane namespace.
2. A new authenticated superuser session begins with no publisher operating
   context.
3. Publisher operating context is explicit, session-scoped and not identity
   impersonation.
4. Publisher context does not itself confer backend authorization.
5. Ordinary publisher users do not gain `/admin/*` access.
6. There is one canonical publisher workspace, not duplicated staff/publisher
   route trees.
7. `thoth` is the canonical durable owner of staff-facing operational truth.
8. Infrastructure/provider logs are evidence, not the sole Staff Operations
   source of truth.
9. Desired state, operational execution, observed state and attention/reconciliation
   state remain distinct.
10. Historical operation/attempt facts are immutable.
11. Retry, replay and current-state redistribution remain distinct concepts.
12. Indeterminate external effect is not silently classified as known failure.
13. Worker/job success alone never proves observed external success unless a
    domain contract explicitly establishes that implication.
14. The common operational seam is not a universal job/queue framework.
15. Publisher Services `distribution_job*` remains programme-local under
    ADR-0008.
16. Machine execution authority and human staff administration remain distinct.
17. `DistributionPlatform` and `MetricPlatform` remain separate under ADR-0002;
    no name-based mapping is introduced.
18. Service-specific observed-state semantics remain owned by the relevant
    domain.
19. Admin operational queries and commands are backend-authorized independently
    of route guards.
20. A downstream repository waits for merged/pinned upstream contracts.
21. Historical bridge/backfill may preserve known execution facts but must not
    fabricate observed acceptance/publication.
22. Initial reconciliation reports discrepancies; it does not auto-repair.
23. Staff external-write controls activate only after read-only operational truth
    is proven and under their own explicit production authorization.
24. Metrics remains a distinct programme/domain unless a later approved decision
    says otherwise.

---

## 7. Implementation impact and staged delivery

This architecture must be decomposed into bounded repository-local tasks. The
following identifiers describe the intended dependency order and are not
implementation authorizations.

### 7.1 Foundation

`APP-ADM-01` - `thoth-app`: Admin-first shell, route migration, access-denied
boundary and publisher operating context.

`BE-OPS-01` - `thoth`: additive common operational-record/attempt foundation,
initially inactive and not a generic queue.

These foundations may proceed independently once each has its own approved
specification and exact-base authorization. The app foundation must not invent
fake operational data while the backend seam is absent.

### 7.2 Distribution operational truth

`BE-DIST-OPS-01` - `thoth`: distribution operational projection and conservative
historical bridge from existing durable job/attempt facts.

`DIS-OPS-01` - `thoth-dissemination`: execution/evidence reporting against the
merged/pinned backend contract.

`BE-DIST-OBS-01` - `thoth`: per-work/per-destination observed delivery state and
revision/fingerprint comparison semantics.

Compatibility rule: the upstream backend contract merges first; the downstream
executor pins/consumes that exact contract. No downstream task guesses an
unmerged contract.

### 7.3 Read-only Admin operations

`BE-ADM-READ-01` - `thoth`: protected global/publisher operational read
projections.

`APP-ADM-02` - `thoth-app`: read-only Dashboard, Activity and Attention UI.

`APP-ADM-03` - `thoth-app`: publisher-context staff Services workspace.

Read-only production observation must precede staff external-write control
activation. During cutover the new operational projection must be compared with
existing execution lifecycle facts before it becomes the sole staff authority.

### 7.4 Staff commands

`BE-DIST-CMD-01` - `thoth`: replay/current-state redistribution command
foundation, initially inactive and fail-closed.

`APP-ADM-04` - `thoth-app`: staff intervention UI consuming backend command
availability; UI presence is not activation authority.

Replay/external-write activation is separately governed. It requires task-level
HIGH/CRITICAL risk classification as applicable, explicit authorization,
controlled pilot, idempotency/duplicate-safety evidence, stop conditions,
monitoring, rollback and post-action reconciliation.

### 7.5 Reconciliation and later domains

`BE-DIST-REC-01` - `thoth`: distribution reconciliation/reporting, initially
report-only.

Later bounded tasks integrate feeds, OAI-PMH and Hosting. Metrics integrates only
through a separately approved cross-programme contract with the Metrics
programme.

No one implementation agent receives unrestricted write access across multiple
repositories. Every repository change has its own issue/specification, branch,
write budget, review and PR; correctness spanning repositories also receives
integration review.

---

## 8. Migration and historical-data rules

### 8.1 Route migration

The `thoth-app` route migration is one-way at the architectural level: the
publisher workspace leaves the `/admin` namespace and `/admin` becomes the
superuser control plane.

A bounded implementation may provide a finite temporary compatibility map for
known unambiguous bookmarks, but must not create a blanket alias that makes
`/admin/*` ambiguous again.

### 8.2 Existing distribution-job history

Existing Publisher Services distribution jobs and attempts are useful historical
execution evidence. A later migration/bridge may project them into Staff
Operations history only for facts that are actually known, including publisher,
job kind/targets, lifecycle timestamps/status, attempts, worker-reported errors
and recorded provenance.

Unknown observed external state remains unknown. In particular:

```text
historical execution: SUCCEEDED
observed delivery: UNKNOWN
```

is valid and truthful. The migration must not convert historical job success into
external acceptance/publication success without evidence.

Any persisted migration/backfill receives its own HIGH-risk task specification,
dry-run counts, deterministic mapping, empty/populated-database validation,
reconciliation and rollback controls.

### 8.3 Operational cutover

New canonical operational reporting is additive first. During cutover the system
must compare new projections against existing execution lifecycle facts and
service-specific evidence before staff depend on them as the sole operational
view.

Observed-state capture is established and verified before reconciliation; report-
only reconciliation is established before automated repair is even considered.

---

## 9. Failure, attention and reconciliation semantics

A common operation read projection should support the semantic distinction among
pending, running, succeeded, failed, cancelled and indeterminate work, but exact
storage enums are owned by bounded implementation tasks and must not force every
service into one internal state machine.

Attention is a separate derived concern. Domain-owned reasons may include:

- retry exhausted;
- indeterminate external state;
- operation stuck/overdue under domain-defined timing rules;
- remote rejection;
- desired/observed mismatch;
- stale delivery;
- missing required asset;
- blocked service configuration;
- reconciliation failure.

A failed attempt that is retrying automatically need not be a staff attention
item. Conversely, a succeeded execution may still produce attention if observed
state later proves stale, missing or contradictory.

Stuck/overdue thresholds are backend/domain rules rather than frontend timers.

Resolution changes the current attention projection; it does not rewrite the
historical operation or attempt.

---

## 10. Security and authorization consequences

The architecture creates no permission merely by creating a route or selecting a
publisher.

Future protected reads/commands require task-specific authorization matrices and
negative tests. At minimum, implementation must prove that:

- non-superusers cannot access protected Admin data/actions;
- publisher context cannot be forged into authorization;
- an operation/publisher mismatch fails closed;
- workers cannot invoke staff administration commands unless a separately
  approved authorization matrix explicitly grants an operation;
- staff administration does not implicitly confer a machine role;
- direct GraphQL/API calls are protected independently of browser navigation;
- sensitive remote/provider payloads and credentials cannot enter staff-facing
  durable records unsafely.

Staff replay/current-state redistribution creates external effects and therefore
requires explicit backend idempotency/duplicate controls and independent
production activation gates.

---

## 11. Read and reporting consequences

Global Admin reads are superuser-only cross-publisher projections. Publisher
context exposes richer publisher-scoped operational detail and permitted
commands.

The backend owns pagination, filters, date windows, stable ordering,
authoritative counts and scalable export behavior. `thoth-app` must not fetch the
entire operation history and perform canonical filtering/counting locally.

Partial failures remain explicit. `unavailable` is not represented as `0`,
`healthy`, `no job` or `no attention`.

The target console can therefore answer progressively richer questions without
inventing facts:

- what services is this publisher entitled/configured for?;
- what did Thoth attempt and when?;
- what actually happened externally, where evidence exists?;
- what is current/stale/missing/indeterminate?;
- what currently requires staff attention?;
- which safe backend commands are available and why?;
- who deliberately replayed/redistributed/cancelled work and why?;
- what happened across all publishers in a requested time window?

---

## 12. Validation requirements for adopting tasks

### 12.1 Shell/context

Prove at least:

- new superuser session -> `/admin`, no publisher context;
- entered publisher context survives refresh/navigation;
- new authentication does not restore the old superuser publisher context;
- ordinary publisher user -> `/admin/*` -> access denied with `Go to dashboard`;
- returning to Admin clears publisher operating context;
- switching/clearing context invalidates publisher-scoped cached data safely;
- global Admin reads never depend on active publisher state.

### 12.2 Operational model

Prove at least:

- historical operations/attempts are not rewritten after repair;
- retry remains the same logical operation while replay creates a linked new
  operation;
- replay and current-state redistribution remain distinct;
- duplicate command submission is idempotent;
- indeterminate external effect is not silently classified as known failure;
- worker success alone does not create observed delivery success.

### 12.3 Authorization

Prove protected reads/commands with anonymous, wrong-role, wrong-scope/context,
operation/publisher mismatch and correct-authority cases appropriate to the
operation. Machine-role and superuser authority remain separate according to
ADR-0008.

### 12.4 Distribution integration

Prove at least:

- one intended logical delivery does not produce duplicate external execution;
- linked destination/adapter behavior remains consistent with the approved
  distribution inventory;
- historical bridge preserves unknown observed state;
- remote uncertainty produces indeterminate/evidence-required behavior;
- downstream executor consumes the exact merged/pinned upstream contract.

### 12.5 Reconciliation

Prove deterministic current/stale/missing/indeterminate/mismatch derivation for
the exact evidence model selected by the domain task. Successful execution alone
cannot yield observed-current unless domain evidence explicitly makes that safe.

---

## 13. Rollout

1. Record and approve this architecture independently of implementation.
2. Implement the Admin shell/context foundation and common backend operational
   foundation as separate repo-local tasks.
3. Integrate distribution operational history/evidence additively.
4. Run comparison/read-only observation before making the new ledger the sole
   staff view.
5. Add publisher-context operational detail.
6. Implement replay/current-state command source behind a separately controlled
   inactive gate.
7. Run a bounded external-write pilot only under explicit production activation
   authorization.
8. Reconcile pilot effects and observe before broader activation.
9. Add report-only reconciliation after observed-state capture is trustworthy.
10. Integrate feed, OAI-PMH and hosting domains separately.
11. Integrate Metrics only through its separately approved cross-programme
    contract.

Merge is not deployment. Deployment is not activation. Read-only production
observation does not authorize replay. Replay activation does not authorize
automated repair.

---

## 14. Rollback and stop conditions

### 14.1 Shell/context rollback

A bounded app task must define a safe rollback to the previous route/context
behavior for its deployment window, while avoiding permanent dual ownership of
the `/admin` namespace.

### 14.2 Operational-record rollback

Additive operational schema/source can be disabled from reads while preserving
recorded history for diagnosis. A rollback must not erase external-effect audit
history merely to restore an old UI.

### 14.3 Replay stop conditions

Replay/current-state external-write activation must stop on any evidence of:

- duplicate external submissions/effects;
- publisher/operation context mismatch;
- idempotency failure;
- incorrect source revision selection;
- unexplained divergence between operational and external observed state;
- secret/sensitive-data leakage into operational records;
- processing-scope broadening;
- inability to reconcile pilot effects.

Automated repair remains prohibited until separately designed and approved.

---

## 15. Consequences

### Positive

- Admin becomes a first-class staff operating mode rather than an add-on to the
  publisher shell.
- Publisher users and superusers share one publisher workspace without identity
  impersonation.
- Staff activity/reporting is based on canonical Thoth-owned operational facts.
- External uncertainty and stale/missing state can be represented truthfully.
- Replay becomes auditable and distinguishable from retry/current-state resend.
- Distribution can become reconciliation-ready without forcing feeds, OAI,
  hosting or Metrics into one job lifecycle.
- Metrics and Distribution remain correctly separated.
- The product can grow toward a broader staff control plane without another shell
  redesign.

### Negative

- route migration creates some bookmark churn;
- the backend gains a new operational/audit concern in addition to domain work
  queues/state;
- service integrations must explicitly project their relevant operations/evidence
  instead of relying on logs;
- observed-state truth requires provider/service-specific work and cannot be
  obtained uniformly;
- read-only operational maturity must precede the desired replay UX, increasing
  delivery stages.

### Risks

- a common `ServiceOperation` could accidentally drift into a generic job
  framework; ADR-0008 and the invariants above prohibit this;
- UI developers could confuse publisher context with authorization;
- historical migration could fabricate delivery success if execution and observed
  state are conflated;
- replay could duplicate external effects if indeterminate state/idempotency is
  mishandled;
- cross-domain labels could encourage forbidden Distribution/Metric platform
  mapping by name;
- Admin could show false health if unavailable projections are treated as zero;
- a downstream executor/client could guess an unmerged upstream contract.

These risks drive the staged, read-only-first, fail-closed rollout.

---

## 16. Non-goals of this ADR

This ADR does not:

1. define final PostgreSQL table/column names for the common operational seam;
2. create a generic job/queue framework;
3. replace Publisher Services `distribution_job*`;
4. merge Distribution and Metrics platform domains;
5. select a DistributionPlatform/MetricPlatform mapping;
6. implement work-level distribution configuration by itself;
7. define every provider's observed-state evidence contract;
8. define Metrics ingestion/storage/source/checkpoint behavior;
9. expose operational data anonymously or to ordinary publisher users;
10. grant publisher users staff replay controls;
11. decide whether any staff operational data becomes publisher-facing later;
12. activate external replay, redistribution or automated repair;
13. run migrations/backfills;
14. change CI/workflows;
15. deploy or release anything;
16. authorize any implementation task merely by becoming approved.

---

## 17. Approval

Approval required from: CTO
Approved by: Not yet approved
Approval date: Not yet approved

Required before approval is relied upon:

1. fresh independent architecture review of the exact ADR content against live
   repository authority, ADR-0002, ADR-0004, ADR-0005, ADR-0008, ADR-0009,
   programme #765, Metrics programme #766 and verified repository contracts;
2. resolution of every blocking review finding;
3. explicit CTO approval of the exact reviewed decision content;
4. independent exact-head source/document review and separate merge authorization
   under the repository controls;
5. merge into `develop` before any implementation task relies on the decision.

Approval of this ADR is architecture approval only. Every adopting task still
requires its own issue/specification, risk classification, exact base, write
budget, action authorization, validation, independent source review and relevant
merge/deployment/activation gates.