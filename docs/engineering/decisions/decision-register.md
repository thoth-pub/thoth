# Engineering Decision Register

Status: ACTIVE
Owner: CTO
Last updated: 2026-08-07

| ADR | Decision | Status | Programmes | Approval blocker |
|---|---|---|---|---|
| `ADR-0001` | Publisher package capability model | APPROVED | Publisher Services, Thoth Metrics, OAI-PMH | Satisfied - CTO approved the final package matrix, OASIS/OBELISK collection distinction and upgrade/downgrade/export semantics on 2026-07-28; independently reviewed PR [#772](https://github.com/thoth-pub/thoth/pull/772) merged into `develop` on 2026-07-29 as `b2c91ff25b95ab0e10a477ff21dbd4702f5db8d4` |
| `ADR-0002` | Distribution and metrics platform domain boundaries | APPROVED | Publisher Services, Thoth Metrics | Satisfied - CTO approved strict type separation and no initial cross-domain mapping on 2026-07-27 through PR [#769](https://github.com/thoth-pub/thoth/pull/769) |
| `ADR-0003` | Repository-authoritative Diesel schema contract | APPROVED | Shared Repository Controls, Publisher Services, Thoth Metrics | Satisfied - CTO selected Architecture A on 2026-08-05; recorded and implemented with its directly related cleanup by `THOTH-DB-CTRL-02` through PR [#778](https://github.com/thoth-pub/thoth/pull/778). Becomes repository-authoritative on merge into `develop`; the merge remains subject to independent exact-head review and explicit CTO merge authorization |
| `ADR-0004` | Distribution platform inventory | APPROVED | Publisher Services | Satisfied at the content-approval level - the ADR-0004 and final `DistributionPlatform` inventory content at exact head `44e6f821535fbee56c830dd6eda237fc6d06fbfd` was independently reviewed under review `4881233664` (`APPROVED`) and explicitly approved by Javi, CTO, under review `4881279067` on 2026-08-07, together with the complete [evidence matrix](../../publisher-services/adr-01-evidence-matrix.md) and the approved [final inventory](../../publisher-services/platform-inventory.md). The approval-state recording is delivered through PR [#783](https://github.com/thoth-pub/thoth/pull/783); `ADR-0004` becomes repository-authoritative only when that PR merges into `develop`. `BE-02` therefore remains blocked pending the merge and its own approved bounded specification and explicit implementation authorization |

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

The bounded approval-state recording is carried by PR
[#783](https://github.com/thoth-pub/thoth/pull/783). The approved decision
becomes repository-authoritative only when PR #783 merges into `develop`.

Publisher Services `BE-02` must not implement or finalize
`DistributionPlatform` from this decision before that merge. `BE-02` also
requires its own approved bounded specification and explicit implementation
authorization.

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
