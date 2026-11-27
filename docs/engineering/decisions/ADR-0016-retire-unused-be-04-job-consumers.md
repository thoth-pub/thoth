# ADR-0016 - Retire the unused BE-04 job surface by removing its consumers

Status: PROPOSED
Date: 2026-11-27
Decision owner: CTO
Decision input: Javi, CTO, 2026-11-27 (recorded on #957): the BE-04 `distribution_job*` API and tables were a first prototype that was never used, so retiring and dropping them is acceptable.
Programmes affected: Shared Backend Architecture (THOTH-ASYNC-01); Publisher Services and Distribution Configuration
Repositories affected: `thoth-pub/thoth`, `thoth-pub/thoth-app`, `thoth-pub/thoth-dissemination`
Parent programme: [THOTH-ASYNC-01 #957](https://github.com/thoth-pub/thoth/issues/957)
Supersedes in part:
- `ADR-0012` section 4.2 and section 4.3 (the two released consumers "must migrate" onto the generic contract), and section 9 Phase B (migrate consumers), to the extent they require migration. Removal of both consumers satisfies them: no released consumer remains.
- The consumer-migration preconditions that `ADR-0012` repeats elsewhere are satisfied by that removal and are not otherwise changed: the opening of section 4.4 ("only after all downstream consumers are verified migrated"), section 4.5 item 3 ("all known downstream consumers are on the generic contract"), Phase B's last paragraph (no legacy field is removed until every consumer is deployed on the generic contract), section 8 items 5, 6 and 7 (the consumer-migration and retirement tasks, read as removal tasks) and invariant 32 ("expand -> migrate consumers -> ..." read as "expand -> remove consumers -> ...").
- `ADR-0015` section 4.2, only its bullets that extend section 4.3's precondition and section 9's Phase A to Phase B rule "to the release that carries the consumer's contract". They apply to no consumer, because none is migrated. Its other bullets stand.
Superseded by: None

Decision: The two released BE-04 consumers are removed, not migrated. `thoth-app` stops querying `latestBackCatalogueJob`, `jobStatuses` and `withoutBackCatalogueJob`, drops the back-catalogue job report and CSV paths, the matching UI and generated types, with no replacement. Its handling of the released `DISTRIBUTION_JOB_CREATION_DISABLED` error is not part of that removal (item 3). `thoth-dissemination` deletes its BE-04 worker (`distribution_job_worker.py`), its workflow and its tests. A dissemination handler on the generic engine is future work and is not a gate. Everything else in `ADR-0012` sections 4.4, 4.5 and 9 (Phases C and D) stays binding; the consumer preconditions named in the header are met by this removal.

Authority condition: this ADR becomes repository-authoritative only when this exact approved version has received independent exact-head review and is reachable from `develop`. It decides architecture only. It authorizes no implementation, schema change, migration, release, deployment or activation.

---

## 1. Context

`ADR-0012` section 4 assumed BE-04 was in use and required an expand, migrate, retire, contract sequence. Evidence gathered on 2026-11-27 shows it was not:

- Evidence below that is not in this repository (production counts, task-definition variable, other-repository searches) is recorded on #957 and cited from there. Only the code facts were checked in the repositories.
- Production tables `distribution_job`, `distribution_job_target` and `distribution_job_attempt` hold 0 rows each (read by the CTO, recorded on #957).
- `THOTH_DISTRIBUTION_JOB_CREATION` is unset in the production graphql-api task definition, so automatic creation is at its OFF default.
- The `thoth-dissemination` Actions variable `DISTRIBUTION_JOB_WORKER_ENABLED` is unset, so the hourly worker workflow never runs.
- The released callers are the `thoth-app` queries above and the `thoth-dissemination` claim, complete and fail lifecycle calls in the BE-04 worker. Both are removed. No code reference exists in metrics-dashboard, metrics-widget, thoth-sphinx or email-automation (default branches, 2026-11-27).

## 2. Decision

1. **Consumer removal replaces consumer migration.** `thoth-app` and `thoth-dissemination` each get a bounded removal change in their own repository. Neither is rewritten onto the generic contract.
2. **Order.** Both removals are deployed before the Thoth release that retires the legacy API, so no released client queries a field that no longer exists. A newly discovered consumer joins the same gate.
3. **Phase C unchanged in substance.** One Thoth release removes the legacy GraphQL surface, the creation toggle and command flags, the `migration_backfill.rs` legacy preflight and every other runtime access to `distribution_job*`, with a repository-wide audit and tests proving zero runtime access. `AUTOMATIC_PUSH` activation stays fail-closed: until a generic creation path is approved it is rejected unconditionally. It never becomes a jobless activation. The released `DISTRIBUTION_JOB_CREATION_DISABLED` error needs the explicit mapping or removal decision of `ADR-0012` section 9. The `thoth-app` handler for it stays while the released API can still return that error, because it is the user message for a rejected `AUTOMATIC_PUSH` activation today. It is replaced or removed with that mapping decision, in step with Phase C.
4. **Phase D stays a separate, later release.** The contraction migration drops the three tables, their indexes and the four queue-only enums, takes locks, asserts zero rows under those locks and aborts otherwise. It is not merged into the Phase C release: `thoth init` migrates before serving, so tasks of the previous version and the rollback target still touch the legacy tables during a rolling deploy. The observation window between the two releases may be one release cycle. Zero production rows reduce the data risk. They do not remove the rollback risk.
5. **Staff visibility is separate.** Operator visibility of jobs (state, failure detail, retry, reconciliation) is a new superuser console on the generic engine's existing protected API. It is its own stage and does not depend on this ADR.

## 3. Consequences

- No generic equivalent of `latestBackCatalogueJob` ships. The admin app shows no job status until the console stage lands.
- Back-catalogue dissemination through BE-04 is no longer available as dormant code. A future dissemination handler is specified on the generic engine.
- Each step still needs its own specification, independent review and grant. The Phase D migration is CRITICAL and needs escalated review.

## 4. Approval

PROPOSED. Approval of the exact content by the CTO and independent exact-head review are required before this is repository-authoritative (`docs/engineering/decisions/README.md`, Route B). Approval is not authorization for any implementation, merge, release or migration.
