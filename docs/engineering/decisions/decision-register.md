# Engineering Decision Register

Status: ACTIVE
Owner: CTO
Last updated: 2026-08-07

| ADR | Decision | Status | Programmes | Approval blocker |
|---|---|---|---|---|
| `ADR-0001` | Publisher package capability model | APPROVED | Publisher Services, Thoth Metrics, OAI-PMH | Satisfied - CTO approved the final package matrix, OASIS/OBELISK collection distinction and upgrade/downgrade/export semantics on 2026-07-28; independently reviewed PR [#772](https://github.com/thoth-pub/thoth/pull/772) merged into `develop` on 2026-07-29 as `b2c91ff25b95ab0e10a477ff21dbd4702f5db8d4` |
| `ADR-0002` | Distribution and metrics platform domain boundaries | APPROVED | Publisher Services, Thoth Metrics | Satisfied - CTO approved strict type separation and no initial cross-domain mapping on 2026-07-27 through PR [#769](https://github.com/thoth-pub/thoth/pull/769) |
| `ADR-0003` | Repository-authoritative Diesel schema contract | APPROVED | Shared Repository Controls, Publisher Services, Thoth Metrics | Satisfied - CTO selected Architecture A on 2026-08-05; recorded and implemented with its directly related cleanup by `THOTH-DB-CTRL-02` through PR [#778](https://github.com/thoth-pub/thoth/pull/778). Becomes repository-authoritative on merge into `develop`; the merge remains subject to independent exact-head review and explicit CTO merge authorization |
| `ADR-0004` | Distribution platform inventory | APPROVED AND REPOSITORY-AUTHORITATIVE | Publisher Services | Satisfied - the ADR-0004 and final `DistributionPlatform` inventory content at exact head `44e6f821535fbee56c830dd6eda237fc6d06fbfd` was independently reviewed under review `4881233664` (`APPROVED`) and explicitly approved by Javi, CTO, under review `4881279067` on 2026-08-07, together with the complete [evidence matrix](../../publisher-services/adr-01-evidence-matrix.md) and the approved [final inventory](../../publisher-services/platform-inventory.md). The approval-state head `82874c2bfb0c211198252e4f4a0b669d31e14836` received final independent exact-head review `4881832108` (`APPROVED`) and CTO merge authorization `4881847699`, and ADR-01 implementation PR [#783](https://github.com/thoth-pub/thoth/pull/783) merged into `develop` as `299b0eff3b9ac10cc0a3a7024ab311ddb135b7eb` on 2026-08-07T10:02:34Z, making `ADR-0004` repository-authoritative. `BE-02`'s ADR-01 dependency is satisfied; `BE-02` remains blocked and unauthorized pending its own approved bounded specification and explicit implementation authorization |
| `ADR-0005` | [Terminal merge evidence and non-recursive closeout](ADR-0005-terminal-merge-evidence.md) | APPROVED | Shared Engineering Control; all programmes using the AI-led delivery controls | Satisfied - CTO-approved Shared Engineering Control decision under [issue #786](https://github.com/thoth-pub/thoth/issues/786), 2026-08-07. Repository-authoritative when the exact approved ADR content is reachable from `develop`; the policy is not effective from an unmerged branch. Live independent-review, merge-authorization, CI and merge evidence is the GitHub pull-request record. Governs lifecycle evidence only: the GitHub review, authorization, CI and merge record is terminal task evidence; approval-state-only commits are prohibited when their sole purpose is copying GitHub review or approval metadata into repository files; no post-merge PR is required merely to record that a PR merged. Every substantive control is retained - exact-head review, no implementer self-approval, guarded merge, HIGH-risk and explicitly gated CTO merge authorization, separate production activation, and bounded PRs for material post-merge corrections. Documentation and control only; no runtime, schema, migration, API, workflow, settings or production effect |
| `ADR-0006` | [Request-scoped GraphQL batching and set-based child loading](ADR-0006-request-scoped-graphql-batching.md) | PROPOSED | Shared Thoth GraphQL / Backend Architecture; Publisher Services and Distribution Configuration; any programme resolving `thoth-api` GraphQL child fields | Open - the CTO selected request-scoped batching / set-based loading in principle, with `BE-02`'s `Publisher.distributionPlatforms` as the first required consumer. The final repository decision requires independent review and explicit CTO approval of the ADR content through its GitHub pull-request record. No implementation task, including [`THOTH-GQL-BATCH-01`](../ai-delivery/tasks/THOTH-GQL-BATCH-01.md), may rely on it until it is `APPROVED` and the approved content is reachable from `develop` |

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
related cleanup in one PR. `THOTH-DB-CTRL-02` delivers the decision through PR
[#778](https://github.com/thoth-pub/thoth/pull/778); it becomes
repository-authoritative through `develop` when that PR merges, which remains
subject to independent exact-head review and explicit CTO merge authorization.
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

`ADR-0006` proposes a shared Thoth GraphQL architecture: a request-scoped,
look-ahead-driven set-based prefetch mechanism on the GraphQL `Context`, so that
a child field on a list of parents can be resolved with a bounded number of
database statements instead of one per parent. It exists because
`thoth-api/AGENTS.md` section 6 requires new lists to avoid N+1 access and use
set-based SQL or batched loaders, while the repository provides no mechanism with
which a new field can comply. `BE-02` surfaced the gap and escalated it; the
concern is shared, not Publisher Services-specific.

The CTO selected request-scoped batching / set-based loading in principle. The
final repository decision requires independent review and explicit CTO approval
of the ADR content. The decision is repository-authoritative when it is
`APPROVED` and the exact approved content is reachable from `develop`; it is not
effective from an unmerged branch.

`ADR-0006` is delivered as a runtime foundation by
[`THOTH-GQL-BATCH-01`](../ai-delivery/tasks/THOTH-GQL-BATCH-01.md), which is
itself `DRAFT` and unauthorized. The dependency order is:

```text
ADR-0006 approved and repository-authoritative
  -> THOTH-GQL-BATCH-01 implemented, reviewed, CTO merge-authorized and merged
  -> BE-02 specification amended on PR #788 to adopt the approved mechanism
  -> fresh exact-head review and explicit CTO approval of the BE-02 specification
  -> separate CTO BE-02 implementation authorization
```

`BE-02` is a dependent of `THOTH-GQL-BATCH-01`, not a dependency of it, and is
not unblocked either by `ADR-0006` being drafted or by its being approved.
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
