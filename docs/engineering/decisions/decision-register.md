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
