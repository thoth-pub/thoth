# Engineering Decision Register

Status: ACTIVE
Owner: CTO
Last updated: 2026-08-25

| ADR | Decision | Status | Programmes | Approval blocker |
|---|---|---|---|---|
| `ADR-0001` | Publisher package capability model | APPROVED | Publisher Services, Thoth Metrics, OAI-PMH | Satisfied - CTO approved the final package matrix, OASIS/OBELISK collection distinction and upgrade/downgrade/export semantics on 2026-07-28; independently reviewed PR [#772](https://github.com/thoth-pub/thoth/pull/772) merged into `develop` on 2026-07-29 as `b2c91ff25b95ab0e10a477ff21dbd4702f5db8d4` |
| `ADR-0002` | Distribution and metrics platform domain boundaries | APPROVED | Publisher Services, Thoth Metrics | Satisfied - CTO approved strict type separation and no initial cross-domain mapping on 2026-07-27 through PR [#769](https://github.com/thoth-pub/thoth/pull/769) |
| `ADR-0003` | Repository-authoritative Diesel schema contract | APPROVED AND REPOSITORY-AUTHORITATIVE | Shared Repository Controls, Publisher Services, Thoth Metrics | Satisfied - CTO selected Architecture A on 2026-08-05; recorded and implemented with its directly related cleanup by `THOTH-DB-CTRL-02` through PR [#778](https://github.com/thoth-pub/thoth/pull/778), which merged into `develop` as `37b802776ae6853affe19d90156f3c1e0654ebe3`, making `ADR-0003` repository-authoritative |
| `ADR-0004` | Distribution platform inventory | APPROVED AND REPOSITORY-AUTHORITATIVE | Publisher Services | Satisfied - the ADR-0004 and final `DistributionPlatform` inventory content at exact head `44e6f821535fbee56c830dd6eda237fc6d06fbfd` was independently reviewed under review `4881233664` (`APPROVED`) and explicitly approved by Javi, CTO, under review `4881279067` on 2026-08-07, together with the complete [evidence matrix](../../publisher-services/adr-01-evidence-matrix.md) and the approved [final inventory](../../publisher-services/platform-inventory.md). The approval-state head `82874c2bfb0c211198252e4f4a0b669d31e14836` received final independent exact-head review `4881832108` (`APPROVED`) and CTO merge authorization `4881847699`, and ADR-01 implementation PR [#783](https://github.com/thoth-pub/thoth/pull/783) merged into `develop` as `299b0eff3b9ac10cc0a3a7024ab311ddb135b7eb` on 2026-08-07T10:02:34Z, making `ADR-0004` repository-authoritative. `BE-02`'s ADR-01 dependency is satisfied; `BE-02` remains blocked and unauthorized pending its own approved bounded specification and explicit implementation authorization |
| `ADR-0005` | [Terminal merge evidence and non-recursive closeout](ADR-0005-terminal-merge-evidence.md) | APPROVED | Shared Engineering Control; all programmes using the AI-led delivery controls | Satisfied - CTO-approved Shared Engineering Control decision under [issue #786](https://github.com/thoth-pub/thoth/issues/786), 2026-08-07. Repository-authoritative when the exact approved ADR content is reachable from `develop`; the policy is not effective from an unmerged branch. Live independent-review, merge-authorization, CI and merge evidence is the GitHub pull-request record. Governs lifecycle evidence only: the GitHub review, authorization, CI and merge record is terminal task evidence; approval-state-only commits are prohibited when their sole purpose is copying GitHub review or approval metadata into repository files; no post-merge PR is required merely to record that a PR merged. Every substantive control is retained - exact-head review, no implementer self-approval, guarded merge, HIGH-risk and explicitly gated CTO merge authorization, separate production activation, and bounded PRs for material post-merge corrections. Documentation and control only; no runtime, schema, migration, API, workflow, settings or production effect |
| `ADR-0006` | [Request-scoped GraphQL batching and set-based child loading](ADR-0006-request-scoped-graphql-batching.md) | SUPERSEDED | Shared Thoth GraphQL / Backend Architecture; Publisher Services and Distribution Configuration; any programme resolving `thoth-api` GraphQL child fields | Superseded by `ADR-0007` (CTO approval 2026-08-11); the A2 look-ahead/store architecture and its batching-specific mutation-guard coupling are no longer the current batching direction, while the section 4.12.6 duplicate-mutation-execution finding remains a live, separately controlled concern. Historical approval record - the CTO explicitly approved the exact `ADR-0006` architecture merged through PR [#789](https://github.com/thoth-pub/thoth/pull/789) on 2026-08-08. Under the ADR authority rule, this decision is repository-authoritative when this approval-state content is reachable from `develop`; a branch carrying `APPROVED` is not authoritative before it merges. The approval covers the final F2 response-key-scoping architecture and the central mutation guard. It does **not** authorize [`THOTH-GQL-BATCH-01`](../ai-delivery/tasks/THOTH-GQL-BATCH-01.md) implementation, which remains `DRAFT` and `NOT AUTHORIZED`, nor either production activation. The ADR carries a **central mutation request guard** rejecting duplicate executable top-level mutation response keys, without which its mutation-isolation invariant does not hold on pinned Juniper. That guard changes the set of accepted GraphQL requests, so it was escalated for the CTO's own decision rather than treated as a consequence of the previously recorded direction; the 2026-08-08 approval covers it explicitly. It carries an explicit `OFF`/`OBSERVE`/`ENFORCE` lifecycle: the foundation would merge with the guard `OFF` and the loader store unavailable, so merge changes no production behaviour and adds no request-path overhead. **Both** `OFF -> OBSERVE` and `OBSERVE -> ENFORCE` require their own explicit CTO production activation approval, distinct from merge authorization and from each other, because in `OBSERVE`/`ENFORCE` the eligibility gate parses and selects an operation for **every** GraphQL request. Production activation is currently BLOCKED pending runtime-operations evidence (open control gap CG-13) and verified monitoring signals and activation thresholds |
| `ADR-0007` | [Conventional request-scoped GraphQL DataLoader and async resolver execution](ADR-0007-conventional-request-scoped-graphql-dataloader.md) | APPROVED | Shared Thoth GraphQL / Backend Architecture; Publisher Services and Distribution Configuration; Thoth Metrics; any programme resolving `thoth-api` GraphQL child fields | Satisfied - the CTO explicitly approved the exact `ADR-0007` architecture on 2026-08-11, following the independently reviewed `THOTH-GQL-DATALOADER-SPIKE-02` outcome (`APPROVED - B0 FEASIBLE`). `ADR-0007` supersedes `ADR-0006`: the shared batching architecture is the conventional request-scoped **non-cached DataLoader** on the pinned Juniper 0.16.x **async** execution path, with the binding loader-first adoption rule, `try_load`-only access, total batch functions, set-based SQL and the approved blocking Diesel boundary. Under the ADR authority rule, the decision is repository-authoritative only when the exact approved content is independently reviewed and reachable from `develop`; a branch carrying `APPROVED` is not authoritative before it merges. Architecture approval is **not** implementation authorization: no B0 foundation, BE-02 or Metrics implementation is authorized by the approval, and **no production activation of any kind is authorized**. The pinned-Juniper duplicate-top-level-mutation-execution finding is retained as a **separate** GraphQL execution concern whose eventual protection mechanism is dispositioned separately; the mutation guard is neither activated nor removed by this decision |
| `ADR-0008` | [Machine roles and durable job primitives](ADR-0008-machine-roles-and-durable-job-primitives.md) | APPROVED | Shared Engineering Control / Shared Backend Architecture; Publisher Services; Thoth Metrics; future affected programmes | CTO decision approved 2026-08-14. Machine/service authorization in `thoth` uses dedicated, **least-privilege, domain-specific** project roles; no generic `SERVICE`/`MACHINE`/`WORKER`/`SERVICE_ACCOUNT` catch-all role is established, an unscoped machine role is permitted only for a genuinely global workload, every machine role requires an explicit policy guard, an explicit authorization matrix and least privilege, and `SUPERUSER` authority does not automatically imply machine-role authority. That `SUPERUSER`/machine-role boundary is the whole of what `ADR-0008` decides about how roles relate: it states no general role-composition, role-aggregation or role-inheritance rule, leaving whether one machine role may imply or compose with another to the owning approved authorization matrix or to a later explicit architecture decision. Those requirements are the whole of the approved cross-programme machine-role rule: enumerated permitted-operation lists, enumerated forbidden-operation lists and separate provisioning/credential controls are **not** approved `ADR-0008` architecture, and apply only where existing repository, deployment or identity-provider controls or an adopting task's own approved specification independently require them; `ADR-0008` decides no provisioning mechanism, credential store, rotation policy or identity-provider arrangement. `DISSEMINATION_WORKER` is approved as a **Publisher-Services-specific** machine role for the BE-04/DIS-02 durable distribution workflow; its operation-level authorization matrix remains owned by the BE-04 specification, which still requires its own independent review and approval. Exactly seven durable-job and concurrency conventions are approved — PostgreSQL durability, explicit state machines, database uniqueness, leases, claim tokens, deterministic idempotency and `FOR UPDATE SKIP LOCKED` where justified — as shared **conventions**, creating neither a shared generic job framework nor a reusable cross-programme job API: an approved convention is not a mandatory mechanism, and `SKIP LOCKED` in particular must still be justified by the adopting task. Other concurrency or retry mechanisms remain governed by existing repository controls and the adopting task's own approved specification; they are not approved here as additional cross-programme conventions. BE-04's `distribution_job`, `distribution_job_target` and `distribution_job_attempt` tables, Rust domain types and lifecycle APIs remain **programme-local** and must not be reused by Metrics or another programme by analogy; a future reusable generic job or queue abstraction requires its own explicit cross-programme ADR before implementation. Approved Decision 5 is that this ruling must be recorded in a shared repository ADR before `BE-04` implementation is authorized; separately, and as an existing repository-process control rather than approved decision content, repository authority requires independent exact-head review and merge of the exact approved content into `develop`, and a branch carrying `APPROVED` is not authoritative before it merges. **No implementation authorization follows automatically**: this decision authorizes no runtime, `policy.rs`, machine-role-creation, identity-provider, provisioning, migration, `schema.rs`, GraphQL, worker-deployment, job-creation, deployment or production action. It must be repository-authoritative before `BE-04` implementation may be authorized, and that is a necessary and not a sufficient condition; `BE-04` implementation and Thoth Metrics `WP5` implementation both remain unauthorized, and `WP5` remains `CRITICAL` and `BLOCKED` under WP4 and its own approved bounded slice specifications |

## Merge and implementation rules

These ADRs may be committed as part of the engineering-control foundation while `PROPOSED`.

No implementation task may rely on them until:

1. the CTO explicitly approves the relevant decision;
2. the ADR status is changed to `APPROVED`;
3. the approval date and approver are recorded;
4. the changed ADR receives independent review;
5. the approved version is merged into `develop`.

## Approval sequence

The ADR-0001 approval-and-merge gate is satisfied. The independently reviewed
approval PR [#772](https://github.com/thoth-pub/thoth/pull/772) merged into
`develop` on 2026-07-29 as
`b2c91ff25b95ab0e10a477ff21dbd4702f5db8d4`.

The following work no longer waits on ADR-0001 approval or merge, but remains
subject to its own approved bounded specification, dependencies, review,
migration, rollout and activation controls:

- Publisher Services `BE-01`;
- metrics entitlement implementation;
- protected metrics serving;
- OAI package-gating implementation.

Approve `ADR-0002` before:

- Publisher Services `BE-02`;
- Thoth Metrics platform-registry implementation;
- any shared platform abstraction or mapping.

`ADR-0004` records the approved `DistributionPlatform` inventory produced by
ADR-01.

The decision and final inventory content were independently reviewed at exact
head `44e6f821535fbee56c830dd6eda237fc6d06fbfd` under review `4881233664`
(`APPROVED`) and explicitly approved by Javi, CTO, under review `4881279067`
on 2026-08-07. The approved inventory in
[`platform-inventory.md`](../../publisher-services/platform-inventory.md) is
exactly that reviewed content.

The bounded approval-state recording was carried by PR
[#783](https://github.com/thoth-pub/thoth/pull/783) at head
`82874c2bfb0c211198252e4f4a0b669d31e14836`, which received final independent
exact-head review `4881832108` (`APPROVED`) and CTO merge authorization
`4881847699`. The approved decision became repository-authoritative when PR
#783 merged into `develop` as `299b0eff3b9ac10cc0a3a7024ab311ddb135b7eb` on
2026-08-07T10:02:34Z. Post-merge control closeout is recorded by
[`ADR-01-CLOSEOUT-01`](../ai-delivery/tasks/ADR-01-CLOSEOUT-01.md).

Publisher Services `BE-02` may still not implement or finalize
`DistributionPlatform` from this decision. The ADR-01 dependency is
satisfied, but `BE-02` remains blocked and unauthorized: it requires its own
approved bounded specification and explicit implementation authorization from
the then-current exact `develop` head.

`ADR-0003` selects Architecture A: `thoth-api/src/schema.rs` is the
repository-authoritative, manually maintained Diesel schema contract, and the
Diesel CLI and root `diesel.toml` are retired from the supported workflow. The
CTO selected it on 2026-08-05 and authorized recording it with its directly
related cleanup in one PR. `THOTH-DB-CTRL-02` delivered the decision through PR
[#778](https://github.com/thoth-pub/thoth/pull/778), which merged into
`develop` as `37b802776ae6853affe19d90156f3c1e0654ebe3`; `ADR-0003` is
repository-authoritative.
`ADR-0003` supersedes the `THOTH-DB-CTRL-01` structural-synchronizer approach,
whose implementation PR
[#777](https://github.com/thoth-pub/thoth/pull/777) was closed unmerged.

`ADR-0005` adopts the terminal merge evidence rule for task lifecycle evidence
across programmes. The GitHub review, authorization, CI and merge record is
terminal evidence that a task merged, so no commit or pull request is created
solely to restate it, and approval-state-only commits are prohibited when their
sole purpose is copying that metadata into repository files. Committed control
documents state durable decisions and an authority condition rather than
transient pull-request status. The decision governs lifecycle evidence only: it
changes neither what must be reviewed nor who may approve, merge, deploy or
activate.

The CTO approved the decision under
[issue #786](https://github.com/thoth-pub/thoth/issues/786) on 2026-08-07. It is
delivered by [`CTRL-MERGE-01`](../ai-delivery/tasks/CTRL-MERGE-01.md).

The decision is repository-authoritative when the exact approved ADR content is
reachable from `develop`; it is not effective from an unmerged branch. Live
independent-review, merge-authorization, CI and merge evidence is the GitHub
pull-request record. It applies to tasks starting after the approved content
reaches `develop`; historical records are preserved as written and are not
rewritten to conform.

`ADR-0007` (CTO-approved 2026-08-11) supersedes `ADR-0006` and its
guard-coupled adoption ordering below. The current shared batching architecture
is the conventional request-scoped non-cached DataLoader on the pinned Juniper
async execution path; batching availability no longer depends on mutation-guard
mode or the `ENFORCE` activation chain, and `THOTH-GQL-BATCH-01` is no longer
the current implementation path. The `ADR-0006` narrative and dependency
sequence below are preserved as the historical record of the superseded
decision. The duplicate-mutation-execution finding and the mutation guard's
runtime disposition remain live, separately controlled concerns under
`ADR-0007` sections 4.13, 7.4 and 9.3.

`ADR-0006` proposes a shared Thoth GraphQL architecture: a request-scoped,
look-ahead-driven set-based prefetch mechanism on the GraphQL `Context`, so that
a child field on a list of parents can be resolved with a bounded number of
database statements instead of one per parent. It exists because
`thoth-api/AGENTS.md` section 6 requires new lists to avoid N+1 access and use
set-based SQL or batched loaders, while the repository provides no mechanism with
which a new field can comply. `BE-02` surfaced the gap and escalated it; the
concern is shared, not Publisher Services-specific.

The CTO selected request-scoped batching / set-based loading in principle, and
subsequently selected **uniform top-level-response-key scoping** as the
architecture direction for the mutation-payload boundary. Loader state is
therefore owned by one GraphQL request but partitioned by the current top-level
GraphQL response key, giving the store identity
`(top-level response key, loader identity, normalized load shape, parent key)`
uniformly for queries and mutation payloads. No loader entry crosses top-level
response-key scopes, so correctness does not depend on the executor serializing
top-level mutation fields, and no mutation resolver retrofit is required. The
scope is derived through one isolated pinned-Juniper compatibility shim, which
carries a revalidation obligation on any Juniper upgrade. The accepted cost is
that reuse no longer spans two top-level query response keys.

The CTO explicitly approved the exact ADR content merged through PR
[#789](https://github.com/thoth-pub/thoth/pull/789) on 2026-08-08, so the
architecture decision itself is made; that approval covers the final F2
architecture and the central mutation guard. The remaining gate is durable and
unchanged in kind: the approved state must receive independent exact-head review
and merge into `develop` before the decision becomes repository-authoritative.
The decision is repository-authoritative when it is `APPROVED` **and** the exact
approved content is reachable from `develop`; it is not effective from an
unmerged branch. Architecture approval is not implementation authorization and
is not production activation.

`ADR-0006` is delivered as a runtime foundation by
[`THOTH-GQL-BATCH-01`](../ai-delivery/tasks/THOTH-GQL-BATCH-01.md), which is
itself `DRAFT` and unauthorized. The dependency order is:

```text
ADR-0006 independently reviewed, CTO-approved and repository-authoritative
  -> THOTH-GQL-BATCH-01 implementation separately authorized
  -> THOTH-GQL-BATCH-01 implemented, independently reviewed,
     CTO merge-authorized and merged
       (merged state: guard mode OFF, loader store unavailable,
        production request acceptance unchanged. Merging deploys
        nothing: the deployed production release predates this work
        and is PRE-GUARD, with no guard mode at all -- established
        by repository release-code evidence and scoped deployment
        metadata together, not by either alone)
  -> runtime-operations evidence for mode control verified
     (control gap CG-13, or a bounded successor, satisfied for this feature).
      This gate is NOT satisfied by documenting the mechanism. The bounded
      successors, none of which closes CG-13, are:
        THOTH-GQL-OPS-01  control record, provisional runbook and the
                          prerequisite specifications; terminates at
                          disposition C - BLOCKED, gate NOT SATISFIED.
                          Its delivered control record and PROVISIONAL
                          runbook live in docs/engineering/repository-map/
        THOTH-GQL-OPS-02  mode-control path, so the value can be consumed
                          at all. MERGED (PR #797): the production-applicable
                          `init` command now registers the guard argument, so
                          OFF/OBSERVE/ENFORCE are consumed, an absent value
                          yields OFF and an invalid value fails startup.
                          Capability gap 1 is CLOSED in-repository. The mode
                          is now settable; it is not set, and nothing is
                          activated
        THOTH-GQL-OPS-03  mechanism proving the effective mode of every
                          serving instance. Capability gap 2 remains OPEN:
                          configured intent is still not proof of
                          process-effective mode, and no surface, log or
                          signal reports the mode a serving process actually
                          computed
        THOTH-GQL-OPS-04  fresh bounded verification and closure; the
                          earliest point at which this gate may be
                          satisfied, and only on evidence
      THOTH-GQL-OPS-03 and THOTH-GQL-OPS-04 are specified, DRAFT and
      implementation NOT AUTHORIZED, and neither of their reserved
      IMPLEMENTATION branches exists --
        feature/shared-architecture/graphql-guard-mode-fleet-verification
        feature/shared-architecture/graphql-runtime-ops-closure
      A THOTH-GQL-OPS-03-SPEC documentation branch does exist while its
      specification pull request is open; a specification branch is NOT an
      implementation branch and constitutes no implementation authorization.
      Specifying a task is not delivering it, and a specification for a
      verifier is not a verified fleet.
      THOTH-GQL-OPS-02 merging closed one capability gap and closed no gate:
      CG-13 remains OPEN, the runtime-operations gate remains NOT SATISFIED,
      the mode-transition runbook remains PROVISIONAL, and both activations
      remain unauthorized
  -> service-health signals and activation thresholds verified
  -> preview/staging acceptance of the exact implementation candidate,
     including performance evidence and a rehearsed, timed rollback
  -> explicit CTO production activation approval for OFF -> OBSERVE
  -> explicit OBSERVE compatibility window completed
  -> compatibility AND operational evidence reviewed, with zero unresolved
     legitimate-client blockers and no service-health regression
  -> separate explicit CTO production activation approval for OBSERVE -> ENFORCE
  -> ENFORCE activated and required observation/acceptance evidence completed
  -> BE-02 specification amended on PR #788 to adopt the now-active mechanism
  -> fresh independent exact-head review of the BE-02 specification
  -> explicit CTO approval of the BE-02 specification
  -> fresh exact-develop verification
  -> separate CTO BE-02 implementation authorization
```

These are distinct gates and must not be conflated. In particular, **CTO merge
authorization is not production activation authorization**: the foundation merges
with the guard in `OFF`, and the transitions `OFF -> OBSERVE` and
`OBSERVE -> ENFORCE` **each** require their own explicit CTO production
activation approval. `OBSERVE` is itself HIGH-risk production behaviour — it
parses and selects an operation for every GraphQL request — so approving it does
not approve `ENFORCE`, and approving `ENFORCE` does not retroactively approve
`OBSERVE`.

Production activation is currently **BLOCKED**, independently of ADR approval or
merge readiness. Control gap **CG-13 — "Thoth runtime operations unmapped"**
leaves the mechanism for changing, propagating, verifying and rolling back the
mode value unestablished, and this repository holds no authoritative GraphQL
latency or error-rate baseline from which activation thresholds could be derived.
Neither blocks merging an inert `OFF` foundation; both block `OBSERVE`.

`BE-02` is a dependent of `THOTH-GQL-BATCH-01`, not a dependency of it, and is
not unblocked either by `ADR-0006` being drafted or by its being approved. Under
the conservative ordering the specification currently selects, `BE-02`
implementation authorization additionally waits for the batching foundation's
`ENFORCE` gate: the store is structurally unavailable outside `ENFORCE`, so an
adopting field would otherwise take its direct fallback while claiming N+1
compliance.
Existing child resolvers are not automatically migrated by
`THOTH-GQL-BATCH-01`; legacy remediation is evidence-led follow-up work under
`ADR-0006` section 10.

`ADR-0006` distinguishes two properties that must not be conflated. A
loader-backed field is **correct** on every path, because a direct per-parent
fallback always exists. It is **N+1 compliant** only where every material
list/fan-out path capable of producing N child queries is covered by a set-based
prefetch and that coverage is measured. The existence of a fallback is never, by
itself, compliance evidence. Each adopting task therefore owes an exact-base
fan-out path inventory, coverage of every material path or an explicit
escalation, and per-path statement-count evidence. For
`Publisher.distributionPlatforms` that inventory belongs to the `BE-02` adoption
task, not to `THOTH-GQL-BATCH-01`.

`ADR-0007` adopts the conventional request-scoped, non-cached DataLoader
architecture on the current pinned Juniper 0.16.x stack and makes async GraphQL
execution the supported model for loader-backed resolvers and general GraphQL
test execution. It retires `ADR-0006`'s look-ahead-prefetch/store architecture
and its batching-specific coupling to mutation-guard mode. The decision rests on
the two independently reviewed disposable B0 spikes
(`THOTH-GQL-DATALOADER-SPIKE-01`/`-02`, final review `APPROVED - B0 FEASIBLE`),
whose portable evidence bundle was transferred for inspection: bounded
set-based batch chunking at the configured maximum batch size, the loader-first
adoption rule replacing any universal sublinearity claim, `try_load`-only
access with total batch functions, GraphQL-visible error equivalence for both
existing direct-path error conventions, request-local loader lifecycle, genuine
non-caching with read-after-write freshness, byte-identical production SDL, and
a bounded async test-harness migration.

The CTO approved the exact `ADR-0007` content on 2026-08-11. The decision is
repository-authoritative when it is `APPROVED` **and** the exact approved
content is reachable from `develop`; it is not effective from an unmerged
branch. Live independent-review, merge-authorization, CI and merge evidence is
the GitHub pull-request record. Architecture approval is not implementation
authorization and is not production activation: the bounded B0 foundation task,
`BE-02` adoption and Metrics adoption each require their own approved
specification and explicit authorization from the then-current exact `develop`
head, and no mutation-guard mode transition (`OFF -> OBSERVE` or
`OBSERVE -> ENFORCE`) is authorized, required or removed by this decision. The
pinned-Juniper duplicate-top-level-mutation-execution defect remains a separate
production-safety concern with its own future disposition; PR #799 / OPS-03
remain frozen and are neither a prerequisite for nor unblocked by `ADR-0007`.

`ADR-0008` settles the cross-programme machine-role and durable-job questions
that Publisher Services and Thoth Metrics reached independently: the BE-04
durable distribution-job design needs a non-human caller, and Metrics `WP5 -
Service auth and entitlements` lists a "role decision" as its first blocking
dependency, over the same crate, the same `policy.rs`, the same `Role` enum and
the same ZITADEL project. At the ADR's verification base, repository policy
defines no role as a dedicated machine/service role, so whichever programme
landed first would have set the precedent for the other by delivery order rather
than by decision.

The CTO approved five decisions on 2026-08-14. Machine authorization is
domain-specific and least-privilege, with no generic catch-all machine role;
`DISSEMINATION_WORKER` is Publisher-Services-specific and settles nothing for
Metrics; the seven listed durable-job conventions are conventions rather than a
framework; BE-04's job tables, domain types and lifecycle APIs stay
programme-local, and a future generic cross-programme job or queue abstraction
needs its own ADR; and the ruling must be recorded in a shared repository ADR
before `BE-04` implementation is authorized.

Separately from those five decisions, and under the repository's existing process
controls, the record is repository-authoritative when it is `APPROVED` **and** the
exact approved content is independently reviewed and reachable from `develop`; it
is not effective from an unmerged branch. Live independent-review,
merge-authorization, CI and merge evidence is the GitHub pull-request record.
Architecture approval is not implementation authorization: `ADR-0008` must be
repository-authoritative before `BE-04` implementation may be authorized, which
is a necessary and not a sufficient condition, and the `BE-04` specification
candidate is not approved by `ADR-0008`. `BE-04` implementation, Thoth Metrics
`WP5` implementation, machine-role creation, role provisioning,
identity-provider change, worker deployment, durable job creation and every
production action all remain unauthorized.
