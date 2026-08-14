# ADR-0008 - Machine roles and durable job primitives

Status: APPROVED
Date: 2026-08-14
Approved by: Javi, CTO
Approval date: 2026-08-14
Decision owner: CTO
Programmes affected: Shared Engineering Control / Shared Backend Architecture (owning programme); Publisher Services and Distribution Configuration; Thoth Metrics; future `thoth-api` programmes requiring machine/service identities or durable-job primitives
Repositories affected: `thoth-pub/thoth`
Supersedes: None
Superseded by: None

Decision: machine and service authorization in `thoth` uses dedicated,
least-privilege, **domain-specific** project roles, and no generic catch-all
machine role is established; `DISSEMINATION_WORKER` is approved as a
**Publisher-Services-specific** machine role for the BE-04/DIS-02 durable
distribution workflow, whose operation-level authorization matrix remains owned
by the BE-04 specification; the durable-job and concurrency conventions listed in
section 3.3 are approved shared engineering **conventions**, not a shared job
framework; BE-04's `distribution_job*` tables, Rust domain types and lifecycle
APIs remain **programme-local**; and any future reusable generic cross-programme
job or queue abstraction requires its own explicit cross-programme ADR before
implementation.

Approved Decision 5 is that this ruling must be recorded in a shared repository
ADR before BE-04 implementation is authorized. What follows from that under the
repository's existing process controls is stated next, and is **not** additional
approved decision content.

Authority condition: this record is repository-authoritative when this exact
approved content is reachable from the repository's authoritative integration
branch (`develop`). A branch carrying `APPROVED` is **not** repository-authoritative
before it merges, and no implementation task may rely on this decision until the
exact approved content has received independent exact-head review and has been
merged into `develop`. That exact-head-review and merge mechanism is an existing
repository control under [`ADR-0005`](ADR-0005-terminal-merge-evidence.md) and the
engineering controls, not a clause the CTO approved on 2026-08-14. Live
independent-review, merge-authorization, CI and merge evidence is the GitHub
pull-request record.

Approved architecture is not implementation authorization. Nothing in this ADR
authorizes runtime, role-provisioning, identity-provider, migration, GraphQL,
worker-deployment, job-creation, deployment or production action.

Verification base: `develop` at `fac86e38383e2059e8795698e1585932c35b5b6d`, the
merge commit of PR [#813](https://github.com/thoth-pub/thoth/pull/813). This ADR
was drafted against that exact commit.

Provenance: CTO-approved control ruling supplied for this bounded task on
2026-08-14.

---

## 1. Context

### 1.1 The repository has no machine-role convention

At the verification base, [`thoth-api/src/policy.rs`](../../../thoth-api/src/policy.rs)
defines five roles. `SUPERUSER` is an **unscoped** project role checked by
`is_superuser()`; `PUBLISHER_ADMIN`, `PUBLISHER_USER`, `WORK_LIFECYCLE` and
`CDN_WRITE` are **publisher-scoped** roles checked per ZITADEL organisation
through `has_role_for_org(...)`.

**No existing role is defined by repository policy as a dedicated
machine/service role.** The current policy distinguishes roles by permission and
scope; it does not encode a dedicated machine-principal role category. Nothing in
`policy.rs` states or tests what kind of principal holds a role, and this ADR
makes no claim about which principals currently hold any role, nor about current
identity-provider assignments.

The consequence is architectural, not empirical: the repository has no role that
was designed as a machine role, and therefore no convention describing how one
should be named, scoped, guarded, bounded or introduced.

[`thoth-api/AGENTS.md`](../../../thoth-api/AGENTS.md) section 7 already requires
that "service roles must be least-privilege and distinct where read, ingest and
synchronization have different powers", and requires an explicit machine-role
authorization test alongside the anonymous, wrong-role, wrong-scope,
correct-scope and superuser cases. That control states the required property. It
does not answer the architectural question of whether the repository should have
one machine role or several, nor who owns that answer.

### 1.2 Two programmes reached the same question independently

**Publisher Services.** The BE-04 durable distribution-job specification
candidate needs a non-human caller: a back-catalogue worker that claims jobs,
reports attempt outcomes and cannot be a publisher user. It proposes
`DISSEMINATION_WORKER` — which would be the repository's first non-`SUPERUSER`
unscoped project role — and explicitly escalated the adjacency rather than
deciding it, recording that the judgement is reserved to the CTO.

**Thoth Metrics.** [`docs/metrics/task-status.md`](../../metrics/task-status.md)
records work package **WP5 - Service auth and entitlements** (`thoth` + clients,
`CRITICAL`, `BLOCKED`) whose first listed blocking dependency is a **"role
decision"**. WP5 concerns the same crate, the same `policy.rs`, the same `Role`
enum and the same ZITADEL project.

The two programmes do not depend on each other, but they would both write into
the same authorization surface. Whichever landed first would silently set a
precedent for the other. Precedent established by delivery order is the weakest
possible form of shared architecture, and the repository's decision process
exists precisely so that a cross-programme question is answered by the CTO rather
than inherited from whoever shipped first.

### 1.3 The same applies to durable-job machinery

The root [`AGENTS.md`](../../../AGENTS.md) section 10 already makes PostgreSQL the
canonical durable owner of jobs, leases, checkpoints, canonical records,
reconciliation outcomes and audit history, and prohibits a local file, a GitHub
Actions workflow, an S3 object, browser state or an external service from being
the sole durable owner. [`thoth-api/AGENTS.md`](../../../thoth-api/AGENTS.md)
section 5 already names the concurrency primitives available for that purpose,
including unique constraints, row locks, advisory locks, leases with expiry,
claim tokens, `FOR UPDATE SKIP LOCKED` and deterministic idempotency keys.

Those controls are correct and are not in question. What they do not say is
whether a task that applies them is thereby building **shared infrastructure**
for other programmes. BE-04's proposed `distribution_job`,
`distribution_job_target` and `distribution_job_attempt` tables would be the
repository's first substantial durable-job implementation, and the same
precedence risk applies: a later programme could reuse them by analogy, turning a
bounded programme-local design into an unowned de-facto framework without any
decision having been taken.

### 1.4 Why the decision was escalated

The CTO was asked to settle, once and for both programmes:

1. whether machine authorization uses one generic role or domain-specific roles;
2. whether `DISSEMINATION_WORKER` is acceptable, and how far its approval reaches;
3. whether the durable-job primitives are conventions or a framework;
4. whether BE-04's job machinery is programme-local, and what gate applies before
   any future generic shared abstraction;
5. what authority condition must be satisfied before BE-04 implementation may be
   authorized.

Those five questions are the five decisions recorded here, and they map onto this
record as set out at the head of section 3.

This ADR records the CTO's answers. It reconsiders, weakens, broadens and
replaces nothing in the approved ruling, and it settles no bounded detail that
an owning specification still holds.

---

## 2. Decision drivers

1. Least privilege for non-human callers, which cannot be supervised, prompted or
   interrupted the way a human operator can.
2. A clear, testable authorization boundary for every machine caller, expressible
   in `policy.rs` and provable by negative tests.
3. Separation of human administrative authority from machine execution authority:
   they have different threat models, different blast radii and different
   credential lifecycles.
4. Correct scoping: a publisher-scoped role and a genuinely cross-publisher
   workload are not interchangeable in either direction.
5. Avoid premature shared abstraction. At the verification base, repository
   policy defines no dedicated machine/service role and the repository has zero
   durable-job tables; one implementation is not enough evidence to design a
   framework from.
6. Avoid cross-programme coupling that no programme owns, and that no programme
   can change safely.
7. Keep every reusable engineering primitive available without making any of them
   mandatory.
8. Keep bounded specification detail with its owning specification, so that this
   ADR does not silently pre-approve an operation-level authorization matrix it
   has not reviewed.
9. Keep architecture decisions separable from implementation authorization, in
   line with the existing ADR authority rule.
10. Make the shared convention durable and legible enough that a later programme
    can apply it without re-litigating it.

---

## 3. Decision

The CTO approved five decisions. They map onto this record as follows, and the
subsection numbering is **not** itself the decision numbering:

| Approved decision | Recorded in |
|---|---|
| 1 — domain-specific machine roles | section 3.1 |
| 2 — `DISSEMINATION_WORKER` Publisher-Services boundary | section 3.2 |
| 3 — shared durable-job/concurrency conventions, not a framework | section 3.3 |
| 4 — BE-04 programme-local ownership, including the requirement for a future explicit ADR before any generic shared abstraction | sections 3.4 **and** 3.5 |
| 5 — this ruling must be recorded in a shared repository ADR before BE-04 implementation is authorized | the header **and** section 8.1, which also distinguish the existing repository-process controls that determine when the record becomes repository-authoritative |

### 3.1 Domain-specific machine-role convention

Machine and service authorization in `thoth` uses **dedicated, least-privilege,
domain-specific project roles**.

The repository does **not** establish a generic universal role such as:

```text
SERVICE
MACHINE
WORKER
SERVICE_ACCOUNT
```

or any equivalent catch-all. No role may be introduced whose meaning is "this
caller is a machine".

An **unscoped** machine role is permitted only when the owning machine workload
genuinely operates globally rather than for one publisher or organisation. A
workload that acts for a single publisher or organisation uses a scoped role;
lack of a convenient scoping mechanism is not a reason to make a role global.

Every machine role requires all of the following, established by its own approved
specification:

1. an explicit policy predicate or guard in the authorization layer;
2. an explicit authorization matrix;
3. least privilege — the narrowest set of operations that makes the workload
   function.

Those three, together with the domain-specific and scoping rules above and the
`SUPERUSER` boundary below, are the whole of what this ADR approves as a
cross-programme machine-role rule.

Requirements an adopting task may also owe — an enumerated list of permitted
operations, an enumerated list of forbidden operations, and provisioning and
credential controls separate from the role's authorization design — are **not**
approved here as ADR-0008 cross-programme architecture. Where they apply, they
apply as **existing controls** or as requirements of the adopting task's own
approved specification, independently of this decision:
[`thoth-api/AGENTS.md`](../../../thoth-api/AGENTS.md) section 7 already requires
least-privilege, distinct service roles and explicit authorization tests for
every protected operation, and provisioning and credential handling remain
separately controlled by the owning implementation/deployment task and by the
identity provider outside this repository. This ADR describes and decides no
provisioning mechanism, credential store, rotation policy or identity-provider
arrangement, and converts none of those requirements into ADR-0008 architecture.

`SUPERUSER` authority does **not** automatically imply machine-role authority.
Holding `SUPERUSER` does not by itself confer a machine role's permitted
operations, and holding a machine role does not by itself confer administrative
authority.

That boundary is the whole of what this ADR decides about how roles relate.
Whether any future machine role may imply, aggregate or compose with another role
is **not decided here**: it belongs to the owning approved authorization matrix,
or to a later explicit architecture decision if a shared rule ever becomes
necessary.

### 3.2 `DISSEMINATION_WORKER` boundary

`DISSEMINATION_WORKER` is **approved as a Publisher-Services-specific machine
role** for the BE-04/DIS-02 durable distribution workflow.

It may later be implemented with **exactly** the permissions approved by the
BE-04 specification, after that specification has been independently reviewed and
approved in its own right.

This ADR does **not** fix BE-04's operation-level authorization matrix. That
bounded detail remains owned by the BE-04 specification, and approving this ADR
approves neither that specification nor any particular set of operations within
it.

What this role does:

- it establishes the shared enforcement convention of section 3.1 — a named,
  domain-specific, least-privilege role with an explicit guard and an explicit
  authorization matrix.

What this role does **not** do:

- it does **not** authorize any Thoth Metrics operation;
- it does **not** determine the eventual Metrics machine-role name;
- it does **not** determine Metrics permissions;
- it does **not** make Metrics WP5 ready for implementation.

Thoth Metrics must apply the shared convention of section 3.1 under its **own**
approved bounded specification. Everything Metrics needs beyond that convention
is decided by that specification and by the existing repository controls, not by
this ADR, which selects no Metrics role name, scope, entitlement model,
credential model or operation matrix.

### 3.3 Durable-job and concurrency conventions

The following are approved **shared engineering conventions** for `thoth-api`
durable work, where applicable to the workload:

- PostgreSQL durability;
- explicit state machines;
- database uniqueness;
- leases;
- claim tokens;
- deterministic idempotency;
- `FOR UPDATE SKIP LOCKED` where justified by the workload and the evidence.

That list is exhaustive. Other concurrency or retry mechanisms remain governed by
existing repository controls and the adopting task's own approved specification;
they are not approved here as additional cross-programme conventions. In
particular, [`thoth-api/AGENTS.md`](../../../thoth-api/AGENTS.md) section 5
already requires database-enforced correctness for concurrent operations and
already covers stale claims in its test obligations, independently of this ADR
and unchanged by it.

These are conventions and primitives. They create **neither** a shared generic job
framework **nor** a reusable cross-programme job API, and this ADR creates
neither.

The distinction is binding:

```text
approved primitive/convention != mandatory mechanism in every task
```

An adopting task must still justify each mechanism it uses against its own
workload and evidence. `FOR UPDATE SKIP LOCKED` in particular must be justified
by the adopting task rather than copied mechanically: it is an approved primitive
and available without a further architecture decision, and it is not an automatic
design choice. A task whose workload does not need non-blocking exclusive batch
claiming should not adopt it, and a task that does adopt it owes the concurrency
evidence its own specification requires.

The same applies to every other item in the list. "Approved" means "available
without a fresh architecture decision", not "required".

### 3.4 Programme-local BE-04 ownership

BE-04's future

```text
distribution_job
distribution_job_target
distribution_job_attempt
```

tables, Rust domain types and lifecycle APIs remain
**Publisher-Services-specific**.

They are **not**:

- a Metrics job model;
- a universal queue;
- a general `Job`/`Queue` API;
- a reusable cross-programme Rust abstraction.

Whatever BE-04's own approved specification defines within that boundary — its
GraphQL operations, its state machine, its worker protocol — is an implementation
detail of the programme-local lifecycle API, owned by that specification. This
ADR does not enumerate those separately as approved cross-programme categories,
and decides nothing about them.

Thoth Metrics, or any other programme, must **not** reuse BE-04's tables, types
or API merely by analogy. Similarity of shape is not authority to share
machinery. A programme that needs durable jobs applies the conventions of
section 3.3 within its own approved design, and owns the result.

### 3.5 Future shared-abstraction rule

This subsection expresses the future-abstraction portion of approved Decision 4;
it is not a separate approved decision.

A future proposal for a reusable **generic** job or queue abstraction requires its
**own explicit cross-programme ADR** before implementation.

That ADR must be raised, reviewed and approved through the ordinary decision
process in [`README.md`](README.md) and recorded in
[`decision-register.md`](decision-register.md). It may not be introduced inside a
programme implementation pull request, and it may not be inferred from two
programmes having independently applied the same primitives.

This ADR takes no position on whether such an abstraction will eventually be
justified. It fixes only the gate.

---

## 4. Authorization consequences

1. The repository's machine-role convention is domain-specific, least-privilege
   and explicit. Every future machine role is introduced by an approved
   specification satisfying section 3.1's requirements.
2. No generic `SERVICE`, `MACHINE`, `WORKER` or `SERVICE_ACCOUNT` role exists or
   may be introduced by analogy.
3. An unscoped machine role must be justified by a genuinely global workload;
   otherwise the role is scoped.
4. `SUPERUSER` remains an administrative role. Holding it does not by itself
   confer machine-role authority.
5. A machine role confers no publisher scope unless its own approved
   specification defines one explicitly.
6. The existing authorization-test obligations are unchanged and continue to
   apply to every protected operation a machine role can reach. Root
   [`AGENTS.md`](../../../AGENTS.md) section 9 and
   [`thoth-api/AGENTS.md`](../../../thoth-api/AGENTS.md) section 7 require the
   caller matrix — anonymous caller; authenticated caller without the role;
   caller scoped to another publisher; correctly scoped publisher role;
   superuser; machine/service role — to be tested as applicable, and require
   authorization failures to fail closed. This ADR therefore requires that:
   - **negative** authorization cases fail closed;
   - **positive** authorization cases succeed only where the owning approved
     specification's authorization matrix permits them;
   - this ADR does **not** pre-decide which callers are positive cases for any
     future machine operation. In particular, it does not decide whether
     `SUPERUSER` or any publisher-scoped role is a permitted caller of a machine
     operation; that belongs to the operation's own approved authorization
     matrix, subject to the least-privilege requirement in section 3.1.
7. This ADR decides nothing about provisioning or credential handling for a
   machine role. Those remain governed by the existing repository, deployment and
   identity-provider controls and by the owning approved specification,
   independently of this decision. This ADR performs and authorizes no role
   creation, no grant and no identity-provider change.
8. No general role-composition, role-aggregation or role-inheritance rule is
   decided by this ADR, beyond the `SUPERUSER`/machine-role boundary in item 4.
   Whether one machine role may imply or compose with another belongs to the
   owning approved authorization matrix, or to a later explicit architecture
   decision.

---

## 5. Programme consequences

### 5.1 Publisher Services

1. `DISSEMINATION_WORKER` is an approved Publisher-Services-specific machine
   role, under the boundary in section 3.2.
2. BE-04's operation-level authorization matrix is **not** settled by this ADR
   and remains owned by the BE-04 specification.
3. The BE-04 specification candidate is not approved by this ADR. It requires its
   own independent review and approval.
4. BE-04 implementation is **NOT AUTHORIZED** by this ADR. It requires its own
   approved bounded specification and separate explicit implementation
   authorization from the then-current exact `develop` head.
5. This ADR must be repository-authoritative before BE-04 implementation may be
   authorized (section 8).
6. BE-04's durable job storage, types and API are programme-local under
   section 3.4.
7. No `distribution_job`, `distribution_job_target` or `distribution_job_attempt`
   relation exists at the verification base, no automatic job creation exists,
   and none is created or activated by this ADR.
8. DIS-02's back-catalogue worker remains `CRITICAL` and blocked under its
   recorded dependencies. No worker deployment is authorized.

### 5.2 Thoth Metrics

1. This ADR resolves the **shared machine-role convention** that WP5's "role
   decision" dependency refers to, once the exact approved content is
   repository-authoritative on `develop`.
2. WP5 remains `CRITICAL` and `BLOCKED`.
3. WP5 still depends on WP4 and on its own approved bounded slice
   specifications.
4. This ADR does **not** select the Metrics machine-role name, entitlement model,
   credential model or operation matrix. Metrics chooses those under its own
   approved bounded specification, applying section 3.1.
5. Metrics does not use `DISSEMINATION_WORKER`, and does not inherit its
   permissions, scope or semantics.
6. Metrics may not reuse BE-04's job tables, types or API by analogy
   (section 3.4).
7. No Metrics implementation is authorized by this ADR.
8. `MET-CTRL-01` and every other recorded Metrics control debt is untouched by
   this decision.

---

## 6. Rejected alternatives

### A. One universal generic `MACHINE`/`SERVICE` role

Description: introduce a single catch-all role meaning "this caller is a
machine", and let every machine workload use it.

Rejected because of **over-broad privilege and premature shared auth
architecture**. One role cannot be least-privilege for two workloads with
different powers; the union of every machine workload's permissions becomes the
privilege of each. It would also fix a shared authorization architecture on the
evidence of zero implemented machine workloads.

### B. Reuse `SUPERUSER` for workers

Description: grant workers the existing unscoped `SUPERUSER` role rather than
adding a machine role.

Rejected because **human administrative authority and machine execution authority
have different threat and least-privilege boundaries**. A worker that can do
everything an administrator can do fails closed nowhere, and its credential
compromise is indistinguishable from an administrative compromise.

### C. Reuse a publisher-scoped role for global workers

Description: express a cross-publisher worker through the existing
publisher-scoped role mechanism.

Rejected because it is the **incorrect scope for genuinely cross-publisher
workloads**. It would require granting the role for every publisher
organisation — an unbounded, silently growing grant that is neither least
privilege nor auditable as a single decision.

### D. Make BE-04's `distribution_job` tables a repository-wide generic job system

Description: treat BE-04's durable job storage, types and API as the shared job
substrate for every programme.

Rejected because of **premature abstraction and cross-programme coupling**. The
design would be generalized from exactly one workload, and would leave two
programmes coupled through machinery neither owns and neither can change safely.

### E. Require every durable task to use `SKIP LOCKED`

Description: make `FOR UPDATE SKIP LOCKED` mandatory wherever a task performs
durable work.

Rejected because an **approved primitive is not an automatic design choice**;
workload-specific evidence still governs. Mandating a claiming mechanism for
workloads that do not claim would add concurrency machinery without a concurrency
problem, and would substitute a rule for the evidence each task owes.

---

## 7. Implementation boundaries

This ADR is documentation and control only. It authorizes no implementation.

Specifically, this decision does not authorize and does not perform:

- any Rust or runtime implementation;
- any edit to [`thoth-api/src/policy.rs`](../../../thoth-api/src/policy.rs);
- machine-role creation in code;
- any ZITADEL or identity-provider change;
- role provisioning or any grant;
- any database migration;
- any change to `thoth-api/src/schema.rs`;
- any GraphQL contract change;
- worker deployment;
- distribution job creation;
- BE-04 implementation;
- any Thoth Metrics implementation;
- deployment;
- production access;
- workflow dispatch;
- activation of automatic job creation.

Two distinctions are binding throughout this ADR:

```text
shared convention != shared implementation abstraction
approved architecture != implementation authorization
```

A convention constrains how each programme builds its own machinery. It does not
create machinery that programmes share. An approved architecture states what a
future implementation must satisfy. It does not authorize that implementation to
be built.

Every implementation that later applies this ADR requires its own approved
bounded specification, its own independent exact-head review, and its own
explicit authorization, exactly as it would have without this ADR.

---

## 8. Rollout and authority

### 8.1 Approved Decision 5, and the process consequences that follow

Approved Decision 5 is exactly this: **this ruling must be recorded in a shared
repository ADR before BE-04 implementation is authorized.** That is the whole of
what the CTO approved on 2026-08-14 about authority.

Everything in the rest of this section is an **existing repository process
control**, not additional approved decision content. Those controls determine
when this record becomes repository-authoritative, and they would apply to any
ADR whether or not this ruling mentioned them.

Under those controls, this ADR is repository-authoritative when its **exact
approved content** is reachable from `develop`. Concretely, that requires all of:

1. the exact approved decision content recorded here;
2. independent exact-head review of that content;
3. merge into `develop`.

The CTO approved the decision content itself in the control conversation on
2026-08-14, which is why the status above is `APPROVED`. Approval of the decision
and repository authority of this record are **different things**: `APPROVED`
content on an unmerged branch is not yet repository-authoritative and may not be
relied upon for implementation until it has been independently reviewed and
merged to `develop`.

Live independent-review, merge-authorization, CI and merge evidence is the GitHub
pull-request record, per [`ADR-0005`](ADR-0005-terminal-merge-evidence.md).

### 8.2 BE-04 gate

This ADR must be repository-authoritative **before** BE-04 implementation may be
authorized.

Being repository-authoritative is a necessary condition for that authorization,
not a sufficient one. When this ADR reaches `develop`, BE-04 implementation
remains unauthorized until its own specification is independently reviewed and
approved and the CTO separately authorizes implementation from a freshly verified
`develop` head.

This ADR is not BE-04 specification approval, and it is not BE-04 implementation
authorization.

### 8.3 Metrics gate

WP5's "role decision" dependency is resolved by this ADR — as the shared
convention only — once this exact approved content is repository-authoritative on
`develop`. Every other WP5 dependency is unchanged, and no Metrics implementation
is authorized.

### 8.4 Migration, rollback and production effect

Database migration: none. GraphQL contract change: none. Runtime change: none.
Production effect: none.

Rollback is the ordinary revert of a documentation change. Because this ADR
creates no code, no schema and no role, reverting it removes a decision record
and nothing else.

---

## 9. Consequences

### Positive

- machine authorization has one settled, testable convention before the
  repository defines its first dedicated machine role, rather than after;
- least privilege is the default rather than the exception;
- the boundary between administrative authority and machine-role authority is
  explicit;
- two programmes can proceed independently without either inheriting the other's
  authorization design by delivery order;
- durable-job primitives stay available to every programme without becoming
  mandatory;
- BE-04 keeps a bounded, owned design instead of an unowned de-facto framework;
- a genuinely shared abstraction, if ever justified, arrives through a decision
  rather than through drift.

### Costs and risks

- **Role proliferation.** Domain-specific roles mean more roles over time.
  Mitigated by section 3.1's requirement that each be justified and bounded under
  its own approved specification, and by the fact that a role nobody can name a
  workload for is a role that should not exist.
- **Duplicated job machinery.** Two programmes implementing durable jobs under
  the same conventions will write similar code. This is an accepted, explicit
  trade against premature coupling; section 3.5 is the route out if the
  duplication ever becomes the larger cost.
- **Convention drift.** Conventions applied independently can diverge. Mitigated
  by this ADR being the single referenced source, and by each adopting
  specification having to state its own guard and authorization matrix explicitly
  rather than by reference to another programme.
- **Misreading approval as authorization.** The most likely misuse of this
  record. Mitigated by the authority condition, section 7 and section 8.

---

## 10. Review checklist

A reviewer of this ADR should confirm each of the following.

- [ ] All five approved decisions are recorded, and mapped as in the table at the
      head of section 3: Decision 1 -> 3.1; Decision 2 -> 3.2; Decision 3 -> 3.3;
      Decision 4 -> 3.4 **and** 3.5; Decision 5 -> the header authority condition
      **and** section 8. The subsection numbering is not the decision numbering,
      and 3.5 is part of Decision 4 rather than a fifth decision.
- [ ] No generic `SERVICE`, `MACHINE`, `WORKER` or `SERVICE_ACCOUNT` role is
      created or permitted.
- [ ] `SUPERUSER` is not treated as implying machine-role authority, and is not
      characterized as anything beyond an administrative role.
- [ ] Each machine role's approved requirements — explicit guard, explicit
      authorization matrix, least privilege — are stated, and nothing further is
      presented as an approved cross-programme machine-role requirement. In
      particular, permitted-operation lists, forbidden-operation lists and
      separate provisioning/credential controls appear only as existing controls
      or as requirements owned by an adopting specification, clearly
      distinguished from what this ADR decides; no provisioning mechanism,
      credential store, rotation policy or identity-provider arrangement is
      decided here.
- [ ] No general role-composition, role-aggregation or role-inheritance rule is
      stated anywhere in this record. The only thing decided about how roles
      relate is that `SUPERUSER` authority does not automatically imply
      machine-role authority.
- [ ] No claim is made about which principals hold existing roles. The stated
      fact is that no existing role is defined by repository policy as a
      dedicated machine/service role, and that policy distinguishes roles by
      permission and scope without encoding a machine-principal category.
- [ ] Authorization-test semantics are correct: **negative** cases fail closed,
      **positive** cases succeed only where the owning approved specification's
      authorization matrix permits, and this ADR pre-decides no caller — neither
      `SUPERUSER` nor any publisher-scoped role — as a positive case for a future
      machine operation.
- [ ] BE-04's operation-level authorization matrix is left to the BE-04
      specification and is not fixed here.
- [ ] Thoth Metrics is not described as using `DISSEMINATION_WORKER`, and no
      Metrics role name, entitlement model, credential model or operation matrix
      is selected.
- [ ] WP5 is not described as ready; it remains `CRITICAL` and `BLOCKED` with its
      other dependencies intact.
- [ ] The durable-job list is exactly the seven approved conventions — PostgreSQL
      durability, explicit state machines, database uniqueness, leases, claim
      tokens, deterministic idempotency, `FOR UPDATE SKIP LOCKED` where
      justified — with no generic job framework created, and with no further
      mechanism (deduplication keys as a separate convention, bounded lease
      semantics, stale-token rejection, deterministic ordering,
      database-enforced concurrency) promoted into the approved cross-programme
      list.
- [ ] `FOR UPDATE SKIP LOCKED` is not mandated universally and requires
      workload-specific justification.
- [ ] BE-04's `distribution_job*` tables, Rust domain types and lifecycle APIs are
      programme-local, and reuse by analogy is prohibited. GraphQL operations, the
      state machine and any worker protocol are not separately elevated into
      approved cross-programme categories.
- [ ] A future generic cross-programme job or queue abstraction requires its own
      ADR.
- [ ] Approval scope is exact: no statement presents anything beyond the five
      approved decisions as CTO-approved ADR-0008 content, and the repository
      process consequences under `ADR-0005` are distinguished from approved
      Decision 5.
- [ ] Status is `APPROVED`, approval date is 2026-08-14, approver is Javi, CTO,
      decision owner is CTO.
- [ ] The authority condition requires independent exact-head review and merge to
      `develop`, and states that `APPROVED` on an unmerged branch is not
      repository-authoritative.
- [ ] No BE-04 implementation authorization, no Metrics implementation
      authorization, and no approval of any specification candidate is asserted
      anywhere in this record.
- [ ] `shared convention != shared implementation abstraction` and
      `approved architecture != implementation authorization` are both explicit.
- [ ] The change is documentation-only: no runtime, policy, schema, migration,
      GraphQL, workflow, identity-provider or production effect.
