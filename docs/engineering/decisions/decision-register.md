# Engineering Decision Register

Status: ACTIVE
Owner: CTO
Last updated: 2026-08-05

| ADR | Decision | Status | Programmes | Approval blocker |
|---|---|---|---|---|
| `ADR-0001` | Publisher package capability model | APPROVED | Publisher Services, Thoth Metrics, OAI-PMH | Satisfied - CTO approved the final package matrix, OASIS/OBELISK collection distinction and upgrade/downgrade/export semantics on 2026-07-28; independently reviewed PR [#772](https://github.com/thoth-pub/thoth/pull/772) merged into `develop` on 2026-07-29 as `b2c91ff25b95ab0e10a477ff21dbd4702f5db8d4` |
| `ADR-0002` | Distribution and metrics platform domain boundaries | APPROVED | Publisher Services, Thoth Metrics | Satisfied - CTO approved strict type separation and no initial cross-domain mapping on 2026-07-27 through PR [#769](https://github.com/thoth-pub/thoth/pull/769) |
| `ADR-0003` | Repository-authoritative Diesel schema contract | APPROVED | Shared Repository Controls, Publisher Services, Thoth Metrics | CTO selected Architecture A on 2026-08-05; recorded with its directly related cleanup in draft PR [#778](https://github.com/thoth-pub/thoth/pull/778) (`THOTH-DB-CTRL-02`), which remains unmerged pending independent exact-head review and explicit CTO merge authorization |

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

`ADR-0003` selects Architecture A: `thoth-api/src/schema.rs` is the
repository-authoritative, manually maintained Diesel schema contract, and the
Diesel CLI and root `diesel.toml` are retired from the supported workflow. The
CTO selected it on 2026-08-05 and authorized recording it with its directly
related cleanup in one draft PR. While draft PR
[#778](https://github.com/thoth-pub/thoth/pull/778) (`THOTH-DB-CTRL-02`) is open,
the decision is not yet repository-authoritative through `develop`: it becomes
authoritative only when that PR merges after independent exact-head review and
explicit CTO merge authorization. `ADR-0003` supersedes the `THOTH-DB-CTRL-01`
structural-synchronizer approach, whose implementation PR
[#777](https://github.com/thoth-pub/thoth/pull/777) was closed unmerged.
