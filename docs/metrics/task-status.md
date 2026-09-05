# Thoth Metrics Task Status

Status: ACTIVE TRACKER
Programme owner: CTO
Master issue: [#766](https://github.com/thoth-pub/thoth/issues/766)
Approved design: [private Google Doc](https://docs.google.com/document/d/11AeQFGpm0kUZajBM5PrAqsttmzJlpUrt89tGYyVM8c0/edit), Drive revision `6`
Last updated: 2026-09-05 (`MET-WP1-10`, issue
[#888](https://github.com/thoth-pub/thoth/issues/888): the OPERAS import
ledger persistence foundation slice — the `metric_operas_import` table — is
implemented on its slice branch `feature/metrics--wp1-operas-import`; the
inbound ledger is the durable remote-event evidence the approved design
requires to be stored before normalization, and it remains additive and
inactive: no inbound-ledger row is seeded or created at runtime, and no OPERAS
network or API access, provider or runtime inspection, discovery cursor,
rolling scan, snapshot import, remote polling or scheduling, normalization or
canonical ingestion, automatic creation or completion of a `metric_import`,
`direct_collection` eligibility enforcement, configured-uploader matching,
Thoth-export echo detection, linking or skipping, loop-prevention behaviour,
payload-divergence handling, reconciliation, inbound status vocabulary or state
machine, worker claim, lease or retry logic, GraphQL/admin surface, Sphinx
change or production migration was implemented; the table carries exactly the
six design-named fields, with identity the composite
`PRIMARY KEY (remote_instance, remote_event_id)` and no surrogate inbound ID,
so one remote event observed repeatedly resolves to one durable row while the
same `remote_event_id` stays representable for two distinct remote instances
and no global event-ID uniqueness is established, `remote_instance`,
`remote_event_id`, `payload_hash` and `status` required `TEXT` carrying only
the nonblank required-text CHECK with no URI, hostname, event-ID, hash or
status vocabulary invented and no enum, default or trigger, a deliberately
non-unique `payload_hash` so two different remote events may carry equal
payload content, a nullable and non-unique `import_id` with a single-column
non-cascading foreign key to `metric_import (import_id)` — nullable because
remote-event evidence precedes normalization and a linked or skipped event may
never need a canonical import of its own, non-unique because one import may
represent a batch of many distinct remote events — the standard current-time
`created_at` default as the only timestamp, and an index set of exactly the
composite primary-key index; the approved design's section 14.4 generic import
status/creation-time index requirement is already satisfied by the merged
`metric_import_status_created_at_idx` on `metric_import`, so no speculative
OPERAS-import secondary index is introduced and WP9 may add one only from
actual query-plan evidence; creating this ledger does not imply guaranteed
inbound discovery, because the design's section 15.5 completeness blocker
remains externally unresolved without an adequate cursor/created-at stream,
replication, a complete snapshot or export, or an equivalent reliable
incremental mechanism, so no cursor, remote-created-at, scan or snapshot field
is added and WP9 retains ownership of discovery modes, loop prevention,
reconciliation and completeness reporting; there is deliberately no foreign key
or stored relationship to `metric_operas_export` and no duplicated export,
platform, measure or mapping identifier, loop prevention remaining WP9 runtime
and reconciliation logic; the `metric_reconciliation_run` and
`metric_reconciliation_issue` ledgers remain approved future architecture and
are deliberately not created; ADR-0001 remains the entitlement authority and
no Metrics-specific entitlement table is created; the `MET-WP1-09` OPERAS
export ledger persistence foundation (issue
[#884](https://github.com/thoth-pub/thoth/issues/884)) is merged into
`feature/metrics` through PR
[#886](https://github.com/thoth-pub/thoth/pull/886), merge commit
`f66048fcc3f4eacf8c5ce1ac9c07e9fa3179eb0f`, which is the exact authorized base
of this slice; the `MET-WP1-08` OPERAS mapping persistence foundation (issue
[#882](https://github.com/thoth-pub/thoth/issues/882)) is merged into
`feature/metrics` through PR
[#883](https://github.com/thoth-pub/thoth/pull/883), merge commit
`d4980e6fb3ff6a08acebb95c7cb87306750469f2`; the `MET-WP1-07` rollup-delta
persistence foundation (issue
[#880](https://github.com/thoth-pub/thoth/issues/880)) is merged into
`feature/metrics` through PR
[#881](https://github.com/thoth-pub/thoth/pull/881), merge commit
`6093f0ca7f3b7221c656bf514d71b5812e39ac45`; the `MET-WP1-06`
publisher-platform approval foundation (issue
[#878](https://github.com/thoth-pub/thoth/issues/878)) is merged into
`feature/metrics` through PR
[#879](https://github.com/thoth-pub/thoth/pull/879), merge commit
`6a768a5bf60ad9fa00757b19faea633e1ca21c08`; the `MET-WP1-05` coverage
foundation (issue
[#875](https://github.com/thoth-pub/thoth/issues/875)) is merged/delivered to
`feature/metrics` through PR
[#876](https://github.com/thoth-pub/thoth/pull/876), merge commit
`1392f236d5c2749605261ceb70f659d0c9615f9d`; the `MET-WP1-04` record-history
foundation (issue [#872](https://github.com/thoth-pub/thoth/issues/872)) is
merged/delivered to `feature/metrics` through PR
[#873](https://github.com/thoth-pub/thoth/pull/873); the completed
`MET-MIG-V1.9-RECON-01` migration-identity reconciliation (issue
[#868](https://github.com/thoth-pub/thoth/issues/868)) is recorded; WP1
remains `IN PROGRESS`, not complete; source/platform mappings and real OPERAS
URI values remain unapproved; later Sphinx/client/source/WP5/WP9 gates
unchanged; no `feature/metrics -> develop` integration occurred and no next
Metrics slice is authorized)

## 1. Control rule

A work package is not one implementation task. Each must be decomposed into bounded repository-local tasks with an approved specification, one slice branch/PR, actual base/target, risk, dependencies, tests, migration/rollout/rollback and independent review.

## 2. Foundation and readiness

| Task | Repository | Risk | Status | Base / target | Dependencies | Issue |
|---|---|---:|---|---|---|---|
| MET-CTRL-01 Programme controls | `thoth` | LOW | MERGED - COMPLETE | `develop` -> `develop` | Programme-control reconciliation delivered through PR [#833](https://github.com/thoth-pub/thoth/pull/833) and reachable from `develop`. The `MET-CTRL-01` dependency is satisfied and no longer gates WP1 entry. Shared foundation closed (P0-01 closeout PR #767 merged as `bac598e32abbd0d7e69ff467c82945ee00df02ba`). PR #833 is the parent lifecycle anchor; exact review and authorization provenance is retained in the owning task and closeout evidence, and this active tracker does not restate it | [#832](https://github.com/thoth-pub/thoth/issues/832) |
| MET-WP1-01 Metrics registry foundation | `thoth` | HIGH | MERGED TO `feature/metrics` - FIRST WP1 SLICE | `feature/metrics--wp1-registry-foundation` -> `feature/metrics` | Adds the `metric_platform`, `metric_measure` and `metric_platform_measure` registry tables, their four registry enums, the manually maintained `schema.rs` contract, Rust domain types, focused database/model tests and exactly the two approved seed measures (`title_sessions`, `net_units`), additive and inactive: no platform rows, no platform-measure mappings, no source mappings, no GraphQL/admin surface, no production migration. Exact review and authorization provenance is retained in the owning issue | [#836](https://github.com/thoth-pub/thoth/issues/836) |
| MET-WP1-02 Metrics source-state foundation | `thoth` | HIGH | MERGED TO `feature/metrics` - SECOND WP1 SLICE | `feature/metrics--wp1-source-state` -> `feature/metrics` | Adds the `metric_source`, `metric_source_account` and `metric_source_checkpoint` source-state tables and the closed `metric_source_acquisition_type` enum, the manually maintained `schema.rs` contract, Rust domain types and focused database/model tests, additive and inactive: no source, account or checkpoint rows, no source/platform mappings, no lease/claim concurrency behaviour, no GraphQL/admin surface, no production migration. Depends on the merged `MET-WP1-01` registry (`metric_platform` FK). Exact review and authorization provenance is retained in the owning issue | [#841](https://github.com/thoth-pub/thoth/issues/841) |
| MET-WP1-03 Metrics import-state foundation | `thoth` | HIGH | MERGED TO `feature/metrics` - THIRD WP1 SLICE | `feature/metrics--wp1-import-state` -> `feature/metrics` | Adds the `metric_import` and `metric_import_error` tables and the closed `metric_import_status` and `metric_import_error_severity` enums, the manually maintained `schema.rs` contract, Rust domain types and focused database/model tests, additive and inactive: no import or import-error rows, no upload/claim/complete API, no status-transition, lease, retry or queue behaviour, no idempotent-return runtime path, no GraphQL/admin surface, no production migration. The two design-fixed idempotency paths are enforced as mutually exclusive partial unique indexes, and both evidence columns stay nullable. Depends on the merged `MET-WP1-02` source state (`metric_source_account` FK) and the canonical `publisher`. Exact review and authorization provenance is retained in the owning issue | [#863](https://github.com/thoth-pub/thoth/issues/863) |
| MET-MIG-V1.9-RECON-01 Metrics migration-identity reconciliation | `thoth` | MEDIUM | MERGED TO `feature/metrics` - COMPLETE | `feature/metrics--v1.9-migration-reconcile` -> `feature/metrics` | Reconciles the three merged Metrics migration directories to the `v1.9.0` release suffix they belong to, so the durable migration identities on `feature/metrics` are `20260826_v1.9.0`, `20260827_v1.9.0` and `20260828_v1.9.0`. Diesel derives a migration's version from the text before the first underscore, so the rename is ledger-neutral: no migration is replayed and none becomes pending. No schema, data, seed, model or runtime behaviour changed. Exact review and authorization provenance is retained in the owning issue | [#868](https://github.com/thoth-pub/thoth/issues/868) |
| MET-WP1-04 Metrics record-history foundation | `thoth` | HIGH | MERGED TO `feature/metrics` - FOURTH WP1 SLICE DELIVERED | `feature/metrics--wp1-record-schema` -> `feature/metrics` | Adds the `metric_record`, `metric_record_revision` and `metric_record_provenance` canonical history tables and the closed `metric_record_revision_status` and `metric_record_provenance_classification` enums, the manually maintained `schema.rs` contract, Rust domain types and focused database/model tests, additive and inactive: no record, revision or provenance rows, no identity or content hashing, no normalized-observation validation or identifier resolution, no first-arrival, duplicate, revision, conflict or retraction transaction, no managed-source revision authorization, no publisher finality, no rollup delta, no period-overlap detection or concurrency primitive, no GraphQL/admin surface, no production migration. The intentionally circular record/current-revision relationship and the same-record `supersedes` invariant are enforced declaratively by composite foreign keys without triggers or new dependencies. Metrics alpha-2 country storage is a separate `CHAR(2)` representation and does not reuse or change the existing bibliographic alpha-3 `CountryCode`. Depends on the merged `MET-WP1-01` registry (`metric_platform`, `metric_measure` and `metric_reporting_grain`), `MET-WP1-02` source state (`metric_source_account`), `MET-WP1-03` import state (`metric_import`) and the canonical `work`, `publication` and `institution` entities. Delivered to `feature/metrics` through PR [#873](https://github.com/thoth-pub/thoth/pull/873). Exact review, CI and authorization provenance is retained in the owning issue | [#872](https://github.com/thoth-pub/thoth/issues/872) |
| MET-WP1-05 Metrics coverage foundation | `thoth` | HIGH | MERGED TO `feature/metrics` - FIFTH WP1 SLICE DELIVERED | `feature/metrics--wp1-coverage` -> `feature/metrics` | Adds the `metric_coverage` table and the closed `metric_coverage_status` enum (`COMPLETE`, `PARTIAL`, `UNKNOWN`), the manually maintained `schema.rs` contract, Rust domain types and focused database/model tests, additive and inactive: no coverage row is seeded, no coverage calculation, finalization or zero-versus-unknown behaviour, no normalized ingestion or `ingestMetricBatch` transaction, no GraphQL/admin surface, no production migration. Direct non-cascading foreign keys to `metric_source_account`, `metric_import`, `metric_platform` and `metric_measure`; half-open period ordering (`period_end > period_start`); no coverage uniqueness beyond the primary key and no speculative secondary index. Depends on the merged `MET-WP1-01` registry (`metric_platform`, `metric_measure`), `MET-WP1-02` source state (`metric_source_account`) and `MET-WP1-03` import state (`metric_import`). Delivered to `feature/metrics` through PR [#876](https://github.com/thoth-pub/thoth/pull/876), merge commit `1392f236d5c2749605261ceb70f659d0c9615f9d`. Exact review and authorization provenance is retained in the owning issue | [#875](https://github.com/thoth-pub/thoth/issues/875) |
| MET-WP1-06 Metrics publisher-platform approval foundation | `thoth` | HIGH | MERGED TO `feature/metrics` - SIXTH WP1 SLICE DELIVERED | `feature/metrics--wp1-publisher-approval` -> `feature/metrics` | Adds the `metric_publisher_platform_approval` table and the closed `metric_publisher_platform_approval_status` enum (`PENDING`, `APPROVED`, `REVOKED`), the manually maintained `schema.rs` contract, Rust domain types and focused database/model tests, additive and inactive: no approval row is seeded, no approval creation/transition/revocation service, no `PUBLISHER_CONTROLLED` platform-ownership enforcement, no package/capability entitlement check, no publisher-import authorization, no GraphQL/admin surface, no production migration. Direct non-cascading foreign keys to canonical `publisher` and `metric_platform`; `UNIQUE(publisher_id, platform_id)`; `approved_by` stays nullable `UUID` with deliberately no foreign key or actor-identity semantics, deferred to a later separately reviewed administrative write-path specification; no speculative secondary index beyond the primary key and pair-uniqueness index. Depends on the canonical `publisher` and the merged `MET-WP1-01` registry (`metric_platform`). Delivered to `feature/metrics` through PR [#879](https://github.com/thoth-pub/thoth/pull/879), merge commit `6a768a5bf60ad9fa00757b19faea633e1ca21c08`. Exact review and authorization provenance is retained in the owning issue | [#878](https://github.com/thoth-pub/thoth/issues/878) |
| MET-WP1-07 Metrics rollup-delta persistence foundation | `thoth` | HIGH | MERGED TO `feature/metrics` - SEVENTH WP1 SLICE DELIVERED | `feature/metrics--wp1-rollup-delta` -> `feature/metrics` | Adds the `metric_rollup_delta` table — the durable accounting bridge between one canonical metric-record revision and the rebuildable work-level rollup projections — the manually maintained `schema.rs` contract, Rust domain types and focused database/model tests, additive and inactive: no delta row is seeded, no delta generation from ingestion, no first-arrival/revision/retraction transaction, no delta claiming, lease, retry, backoff or stale-claim recovery, no `FOR UPDATE SKIP LOCKED` application loop, no delta application, no rollup projection table, no rebuild generation or active-watermark behaviour, no `claimMetricRollupDeltas`/`completeMetricRollupDeltas` mutation, no dashboard query, no GraphQL/admin surface, no production migration. `delta_value` is a signed `BIGINT` with deliberately no non-negative CHECK, because a revision contributes `new - old` and a retraction subtracts the previously applied value; `status` is required `TEXT` with no PostgreSQL enum, no CHECK enumerating values and no trigger, because the approved design defines no closed status vocabulary; `applied_at` is nullable with no cross-column invariant tying it to `status`. A single non-cascading composite foreign key over `(record_id, revision_id)` against the merged `MET-WP1-04` `metric_record_revision (record_id, record_revision_id)` unique key prevents a delta naming a revision owned by another record and blocks cascading deletion of durable evidence; `UNIQUE(revision_id)` permits at most one delta per canonical revision; no operational claim index is added before the WP4 claim protocol and query-plan evidence are approved. The four rebuildable rollup projection tables (`metric_rollup_work_day`, `metric_rollup_work_month`, `metric_rollup_work_country_month`, `metric_rollup_work_institution_month`) remain approved future architecture and are deliberately out of this slice. Depends on the merged `MET-WP1-04` record history (`metric_record_revision` same-record composite key). Delivered to `feature/metrics` through PR [#881](https://github.com/thoth-pub/thoth/pull/881), merge commit `6093f0ca7f3b7221c656bf514d71b5812e39ac45`. Exact review and authorization provenance is retained in the owning issue | [#880](https://github.com/thoth-pub/thoth/issues/880) |
| MET-WP1-08 Metrics OPERAS mapping persistence foundation | `thoth` | HIGH | MERGED TO `feature/metrics` - EIGHTH WP1 SLICE DELIVERED | `feature/metrics--wp1-operas-mapping` -> `feature/metrics` | Adds the `metric_operas_mapping` table — the canonical durable configuration naming, for one registered Metrics platform/measure pair, the OPERAS `event_uri`, `measure_uri` and `uploader_uri` to use and whether that mapping is `enabled` — the manually maintained `schema.rs` contract, Rust domain types and focused database/model tests, additive and inactive: no mapping row is seeded, no real OPERAS event/measure/uploader URI or platform/measure mapping value is approved or guessed, no `metric_operas_export`, `metric_operas_import`, `metric_reconciliation_run` or `metric_reconciliation_issue` ledger, no OPERAS payload construction or delivery, no claiming, lease, attempt, retry, backoff or status state machine, no remote event ID, request hash or delivery error, no inbound synchronization, loop prevention, reconciliation or cursor/snapshot discovery, no outbound eligibility or `METRICS_OPERAS_EXPORT` capability enforcement, no GraphQL/admin surface, no Sphinx change, no production migration. `mapping_id` is a surrogate `UUID` primary key with the repository-standard Metrics UUID default, supplying the referential target implied by the design's later `metric_operas_export.mapping_id`; `UNIQUE(platform_id, measure_id)` keeps at most one canonical mapping per registered pair so enabled-state and later `mapping_id` selection stay unambiguous; a single non-cascading composite foreign key over `(platform_id, measure_id)` against the merged `MET-WP1-01` `metric_platform_measure (platform_id, measure_id)` unique key prevents a mapping naming a real platform and a real measure that are not registered together and blocks cascading deletion of interoperability configuration, with no redundant single-column key; `event_uri`, `measure_uri` and `uploader_uri` carry only the existing nonblank required-text CHECK, with no URI scheme, parser, normalization, host or uniqueness semantics invented; `enabled` has no database default; no timestamp and no duplicated `direct_collection` column is added, that flag remaining authoritative on `metric_platform_measure`; the index set is exactly the primary key and the pair-uniqueness index. ADR-0001 remains the entitlement authority, so no Metrics-specific entitlement table is created and WP5 remains responsible for protected-operation capability enforcement. Depends on the merged `MET-WP1-01` registry (`metric_platform_measure` pair unique key). Delivered to `feature/metrics` through PR [#883](https://github.com/thoth-pub/thoth/pull/883), merge commit `d4980e6fb3ff6a08acebb95c7cb87306750469f2`. Exact review and authorization provenance is retained in the owning issue | [#882](https://github.com/thoth-pub/thoth/issues/882) |
| MET-WP1-09 Metrics OPERAS export ledger persistence foundation | `thoth` | HIGH | MERGED TO `feature/metrics` - NINTH WP1 SLICE DELIVERED | `feature/metrics--wp1-operas-export` -> `feature/metrics` | Adds the `metric_operas_export` table — the canonical durable record of one outbound OPERAS export for one canonical metric-record revision, using one configured `MET-WP1-08` OPERAS mapping, and the durable idempotency/audit boundary a later external OPERAS write must be reconciled against — the manually maintained `schema.rs` contract, Rust domain types and focused database/model tests, additive and inactive: no export row is seeded or created at runtime, no outbound eligibility, source-period finalization or `METRICS_OPERAS_EXPORT` capability enforcement, no export status vocabulary or transition graph, no claim owner, lease, claim token, retry schedule, backoff, stale-claim recovery or attempt-increment behaviour, no `FOR UPDATE SKIP LOCKED` claim logic, no payload construction, normalization or hashing, no OPERAS network call or remote event recording, no correction/divergence handling, no inbound synchronization or reconciliation, no `metric_operas_import`, `metric_reconciliation_run` or `metric_reconciliation_issue` ledger, no GraphQL/admin surface, no Sphinx change, no production migration. The table carries exactly the ten design-named fields. `export_id` is a surrogate `UUID` primary key with the repository-standard Metrics UUID default; `UNIQUE(record_revision_id)` permits at most one durable export row per canonical revision, because outbound eligibility is defined in singular terms and the approved design retries or claims one durable row rather than creating a row per attempt; unlike the `MET-WP1-07` and `MET-WP1-08` composite keys, both relationships are ordinary single-column non-cascading foreign keys — to `metric_record_revision (record_revision_id)` and to `metric_operas_mapping (mapping_id)`, the surrogate identity `MET-WP1-08` introduced for exactly this purpose — so deleting canonical revision history or OPERAS mapping configuration while durable export evidence exists fails instead of erasing it. `status` is required `TEXT` with only the nonblank required-text CHECK and no PostgreSQL enum, closed vocabulary, default, trigger or cross-column rule; `attempt_count` is required `INTEGER` with `CHECK (attempt_count >= 0)` and no default; `remote_event_id` and `request_hash` are nullable opaque `TEXT`, nonblank when supplied but with no invented syntax, algorithm, encoding, length or uniqueness rule; `last_error` is nullable with no format or retention semantics; `created_at` uses the repository-standard current-time default and `completed_at` is nullable with no default and no status tie; the index set is exactly the primary key and the revision-uniqueness index. The approved design's section 6.14 names no retry-time field while its section 14.4 refers to export indexes on status and retry time; that reviewed inconsistency is a deliberate deferral to WP9, which owns the eventual claim/retry representation and its query/index contract, so no `retry_at`, `next_attempt_at`, lease or claim column and no speculative status/retry index is introduced. Mapping-to-revision platform/measure correspondence remains a later fail-closed enqueue/eligibility validation: the export row deliberately does not duplicate `record_id`, `platform_id` or `measure_id` and adds no trigger. Depends on the merged `MET-WP1-04` canonical revision history (`metric_record_revision`) and `MET-WP1-08` OPERAS mapping registry (`metric_operas_mapping`). ADR-0001 remains the entitlement authority, so no Metrics-specific entitlement table is created and WP5 remains responsible for protected-operation capability enforcement. Delivered to `feature/metrics` through PR [#886](https://github.com/thoth-pub/thoth/pull/886), merge commit `f66048fcc3f4eacf8c5ce1ac9c07e9fa3179eb0f`. Exact review and authorization provenance is retained in the owning issue | [#884](https://github.com/thoth-pub/thoth/issues/884) |
| MET-WP1-10 Metrics OPERAS import ledger persistence foundation | `thoth` | HIGH | IMPLEMENTED ON `feature/metrics--wp1-operas-import` | `feature/metrics--wp1-operas-import` -> `feature/metrics` | Adds the `metric_operas_import` table — the canonical durable record of one remote OPERAS event observed on one remote OPERAS instance, the identity of the payload that event carried, and an optional link to the canonical `MET-WP1-03` import that normalized it, which the approved design requires to be stored *before* normalization — the manually maintained `schema.rs` contract, Rust domain types and focused database/model tests, additive and inactive: no inbound-ledger row is seeded or created at runtime, no OPERAS network or API access, no provider or runtime inspection, no discovery cursor, rolling scan or snapshot import, no remote polling or scheduling, no normalization or canonical ingestion, no automatic creation or completion of a `metric_import`, no `direct_collection` eligibility enforcement, no configured-uploader matching, no Thoth-export echo detection, linking or skipping, no loop-prevention behaviour, no payload-divergence handling, no `metric_reconciliation_run` or `metric_reconciliation_issue` ledger, no inbound status vocabulary or transition graph, no worker claim, lease, retry or `FOR UPDATE SKIP LOCKED` logic, no GraphQL/admin surface, no Sphinx change, no production migration. The table carries exactly the six design-named fields. Identity is the composite `PRIMARY KEY (remote_instance, remote_event_id)` and nothing else: the approved design names no surrogate inbound-ledger ID and deliberately carries `remote_instance` alongside `remote_event_id`, so one remote event observed repeatedly resolves to the same durable row, the same `remote_event_id` stays representable for two distinct remote instances, and no global event-ID uniqueness is established. `remote_instance`, `remote_event_id`, `payload_hash` and `status` are required `TEXT` carrying only the existing nonblank required-text CHECK, with no URI, hostname, tenant or environment rule, no event-ID syntax or length rule, no hash algorithm, encoding or case rule, and no PostgreSQL enum, closed vocabulary, default, trigger, state machine or cross-column rule; `payload_hash` is deliberately non-unique, so two genuinely different remote events may carry equal payload content. `import_id` is nullable and non-unique with a single-column non-cascading foreign key to `metric_import (import_id)`: nullable because remote-event evidence must be recorded before normalization and a linked or skipped event may never need a canonical import of its own, non-unique because one import may represent a batch containing many distinct remote events, and non-cascading so deleting a referenced canonical import fails rather than erasing the evidence. `created_at` uses the repository-standard current-time default and is deliberately the only timestamp. The index set is exactly the composite primary-key index: the approved design's generic import status/creation-time indexing requirement is already satisfied by the merged `metric_import_status_created_at_idx` on `metric_import`, so there is no outstanding OPERAS-import operational index requirement and WP9 may add one only from actual query-plan evidence. There is deliberately no foreign key or stored relationship to `metric_operas_export` and no duplicated export, platform, measure or mapping identifier, because loop prevention remains WP9 runtime and reconciliation logic. Creating this ledger does not imply guaranteed inbound discovery: the design's section 15.5 completeness blocker remains externally unresolved without an adequate cursor/created-at stream, replication, a complete snapshot or export, or an equivalent reliable incremental mechanism, so no cursor, remote-created-at, scan or snapshot field is added and WP9 retains ownership of discovery modes, loop prevention, reconciliation and completeness reporting. Depends on the merged `MET-WP1-03` import state (`metric_import`). ADR-0001 remains the entitlement authority, so no Metrics-specific entitlement table is created and WP5 remains responsible for protected-operation capability enforcement. Exact review and authorization provenance is retained in the owning issue | [#888](https://github.com/thoth-pub/thoth/issues/888) |
| ADR-0001 Package capability model | `thoth` | MEDIUM | APPROVED | `develop` - proposal introduced by merged PR #764 | CTO approved 2026-07-28; approval PR [#772](https://github.com/thoth-pub/thoth/pull/772) | #766 |
| ADR-0002 Platform boundaries | `thoth` | MEDIUM | APPROVED | `develop` - proposal introduced by merged PR #764 | CTO approved 2026-07-27; approval PR [#769](https://github.com/thoth-pub/thoth/pull/769) | #766 |
| SPHINX-BOOT-01 Repository bootstrap | `thoth-sphinx` | MEDIUM | BLOCKED | current `develop`; target `develop` after BR-SPHINX-01 verification | MET-CTRL-01 (**satisfied**); BR-SPHINX-01; approved bootstrap spec | #766 |
| THOTH-DB-CTRL-01 Diesel generation procedure | `thoth` | HIGH | SUPERSEDED | `develop` -> `develop` | Structural-synchronizer architecture superseded by ADR-0003; implementation PR #777 closed unmerged with no code becoming authoritative. Replaced by THOTH-DB-CTRL-02. | #766 |
| THOTH-DB-CTRL-02 Repository-authoritative schema contract | `thoth` | HIGH | MERGED - REPOSITORY-AUTHORITATIVE | `develop` at `4c53709befc91acb481beac54a1d314926b61d76` -> `develop` | Delivered ADR-0003 (Architecture A) and directly related cleanup through PR [#778](https://github.com/thoth-pub/thoth/pull/778), merged into `develop` as `37b802776ae6853affe19d90156f3c1e0654ebe3`. CG-12 is resolved and the shared Diesel schema-control dependency is satisfied. | #766 |
| BR-DASH-01 Dashboard branch readiness | dashboard | HIGH | BLOCKED | observed `dev -> main`; reconcile stale `develop`, then normalize to `develop -> master` | Vercel rollback | #766 |
| BR-WIDGET-01 Widget branch readiness | widget | HIGH | BLOCKED | actual `dev`/`main` | npm release protection | #766 |
| BR-APP-01 App branch readiness | app | HIGH | BLOCKED | actual `dev`/`main` | Vercel branch plan | #766 |

## 3. Work packages

| WP | Scope | Repositories | Risk | Status | Blocking dependencies | Issue |
|---|---|---|---:|---|---|---|
| WP1 | Domain and database foundation | `thoth` | HIGH | IN PROGRESS | entry gates satisfied; registry foundation (`MET-WP1-01`), source-state foundation (`MET-WP1-02`), import-state foundation (`MET-WP1-03`), record-history foundation (`MET-WP1-04`) and coverage foundation (`MET-WP1-05`) merged/delivered to `feature/metrics`; publisher-platform approval foundation (`MET-WP1-06`), rollup-delta persistence foundation (`MET-WP1-07`), OPERAS mapping persistence foundation (`MET-WP1-08`) and OPERAS export ledger persistence foundation (`MET-WP1-09`) merged to `feature/metrics`; OPERAS import ledger persistence foundation (`MET-WP1-10`) implemented on its slice branch `feature/metrics--wp1-operas-import`; each remaining slice requires its own approved bounded specification and separate authorization | #766 |
| WP2 | Canonical ingestion | `thoth` | CRITICAL | BLOCKED | WP1 | #766 |
| WP3 | Upload API and publisher UI | `thoth`, app | HIGH | BLOCKED | WP1/WP2; BR-APP-01; approved bounded slice specifications | #766 |
| WP4 | Rollups and GraphQL | `thoth` | HIGH | BLOCKED | WP1/WP2; benchmark dataset | #766 |
| WP5 | Service auth and entitlements | `thoth`, clients | CRITICAL | BLOCKED | shared machine-role convention settled: [`ADR-0008`](../engineering/decisions/ADR-0008-machine-roles-and-durable-job-primitives.md) is `APPROVED` and repository-authoritative (see the note below) and selects no Metrics role name, entitlement model, credential model or operation matrix — those remain WP5-owned bounded decisions; WP4; approved bounded slice specifications | #766 |
| WP6 | Sphinx core | `thoth-sphinx` | HIGH | BLOCKED | bootstrap; pinned API contract | #766 |
| WP7 | CloudFront driver | `thoth-sphinx` | HIGH | BLOCKED | WP6; fixtures; methodology confirmation | #766 |
| WP8 | Additional drivers and COUNTER | Sphinx/app | HIGH | BLOCKED | WP6; source fixtures; COUNTER decision | #766 |
| WP9 | OPERAS and reconciliation | Thoth/Sphinx | CRITICAL | BLOCKED | WP1/WP2/WP6; mappings; completeness route | #766 |
| WP10 | Dashboard and widget clients | clients/Thoth | HIGH | BLOCKED | WP4/WP5; client CI/tests | #766 |
| WP11 | Deployment, monitoring, migration | multiple | CRITICAL | BLOCKED | WP1-WP10 | #766 |
| MET-E2E-01 | Integrated acceptance/cutover | multiple | CRITICAL | BLOCKED | all production slices | #766 |

The shared architectural dependencies (ADR-0001, ADR-0002, ADR-0003,
ADR-0008) are satisfied and the Diesel/schema-control blocker is resolved.
The `MET-CTRL-01` programme-control dependency is also satisfied, and both
WP1 entry gates are complete: `feature/metrics` exists under the SHA-bound
authorization recorded in
[#766](https://github.com/thoth-pub/thoth/issues/766), and the approved
`MET-WP1-01` registry foundation, `MET-WP1-02` source-state foundation,
`MET-WP1-03` import-state foundation and `MET-WP1-04` record-history
foundation are merged/delivered to `feature/metrics`; the first three
migration identities were reconciled to the `v1.9.0` release suffix by the
completed `MET-MIG-V1.9-RECON-01`. WP1 is
`IN PROGRESS` and not complete: every remaining WP1 slice requires its own
approved bounded specification and separate implementation authorization.
Every later work package remains blocked by its own listed
repository-readiness, design, fixture, contract and bounded-specification
dependencies, which stay attached to those work packages rather than blocking
WP1. No later work package is ready or authorized.

### 3.1 WP5 and the shared machine-role convention

WP5's dependency previously recorded as a bare "role decision" is the **shared
machine-role convention**. That question is decided by
[`ADR-0008`](../engineering/decisions/ADR-0008-machine-roles-and-durable-job-primitives.md)
— machine roles and durable job primitives — which the CTO approved on
2026-08-14. Under it, machine and service authorization in `thoth` uses
dedicated, least-privilege, **domain-specific** project roles: there is no
generic `SERVICE`/`MACHINE`/`WORKER`/`SERVICE_ACCOUNT` catch-all role, an
unscoped machine role is permitted only for a genuinely global workload, every
machine role requires an explicit policy guard, an explicit authorization matrix
and least privilege, and `SUPERUSER` authority does not automatically imply
machine-role authority. That `SUPERUSER`/machine-role boundary is the whole of
what `ADR-0008` decides about how roles relate: it states no general
role-composition, role-aggregation or role-inheritance rule.

Those requirements are the whole of the approved cross-programme machine-role
rule. Enumerated permitted-operation lists, enumerated forbidden-operation lists
and separate provisioning/credential controls are **not** approved `ADR-0008`
architecture; they bind Metrics only where existing repository, deployment or
identity-provider controls, or WP5's own approved bounded specification,
independently require them. `ADR-0008` decides no provisioning mechanism,
credential store, rotation policy or identity-provider arrangement.

**Authority condition — satisfied.** Under the repository's existing process
controls — not as approved decision content — `ADR-0008` resolves that shared
convention for Metrics when its exact approved content is
repository-authoritative on `develop`, that is, independently reviewed at its
exact head and merged. That condition is satisfied: the approved `ADR-0008`
record (delivered through `ADR-0008-RECORD`, PR
[#815](https://github.com/thoth-pub/thoth/pull/815)) is merged and reachable
from `develop`, so the shared machine-role convention dependency is resolved.

**What `ADR-0008` does not decide for Metrics.** It selects no Metrics
machine-role name, entitlement model, credential model or operation matrix.
Metrics chooses those under its own approved bounded specification while applying
the shared convention. `DISSEMINATION_WORKER` is a Publisher-Services-specific
role for the BE-04/DIS-02 durable distribution workflow; it is not a Metrics
role, confers no Metrics operation and determines no Metrics role name or
permissions. Metrics must not reuse `BE-04`'s durable job tables, Rust domain
types or lifecycle APIs by analogy, and a reusable generic cross-programme job or
queue abstraction would require its own explicit cross-programme ADR.

**WP5 status is unchanged.** WP5 remains `CRITICAL` and `BLOCKED`. It still
depends on WP4 and on its own approved bounded slice specifications, and no
Metrics implementation is authorized — by `ADR-0008` or otherwise.

## 4. Branch strategy

```text
develop -> feature/metrics -> feature/metrics--<slice> -> feature/metrics -> develop
```

Each affected repository owns its own `feature/metrics` integration branch.
Focused Metrics child branches are created from it and target it; they do not
target `develop` directly. Under
[`ADR-0009`](../engineering/decisions/ADR-0009-programme-integration-branch-namespace.md)
the child branch is a **sibling** of the integration branch, separated by the
reserved `--` token. `feature/metrics/<slice>` is not usable beneath a live
`feature/metrics` branch, because Git cannot hold a ref and a ref namespace at
the same path. `ADR-0009` standardizes the repository ref spelling only; it does
not amend the substantive Metrics architecture.

Do not create integration branches until a verified `develop` branch and release-protection decision exist.

Before creating any Metrics branch, run the fail-closed namespace preflight in
`AGENTS.md` section 5.1 against live refs.

For `metrics-dashboard`, do not create `feature/metrics` from the stale
`develop` branch. BR-DASH-01 must first reconcile active `dev` history into the
target `develop` branch, or an explicit CTO exception must authorize another
verified base.

## 5. Immediate next actions

1. The shared foundation closeout is complete: PR #767 was independently
   `APPROVED` and merged as `bac598e32abbd0d7e69ff467c82945ee00df02ba`, closing
   P0-01, and the repository closeout record is reconciled.
2. `ADR-0001` package capabilities is `APPROVED` and merged (Javi, CTO,
   2026-07-28, approval PR
   [#772](https://github.com/thoth-pub/thoth/pull/772)); `ADR-0002` platform
   domain boundaries is `APPROVED` and merged (CTO, 2026-07-27, approval PR
   [#769](https://github.com/thoth-pub/thoth/pull/769)). Neither makes any
   work package ready by itself.
3. The Diesel schema-control question (CG-12) is resolved: `ADR-0003`
   (Architecture A) is repository-authoritative. `THOTH-DB-CTRL-01` is
   `SUPERSEDED`; its replacement `THOTH-DB-CTRL-02` delivered ADR-0003
   through PR [#778](https://github.com/thoth-pub/thoth/pull/778), merged
   into `develop` as `37b802776ae6853affe19d90156f3c1e0654ebe3`.
4. `MET-CTRL-01` (issue
   [#832](https://github.com/thoth-pub/thoth/issues/832)) is
   `MERGED - COMPLETE` through PR
   [#833](https://github.com/thoth-pub/thoth/pull/833); its dependency is
   satisfied and no further programme-control gate stands before WP1 entry.
5. WP1 entry is complete: repository-local `feature/metrics` was separately
   authorized and created (SHA-bound authorization recorded in
   [#766](https://github.com/thoth-pub/thoth/issues/766)), the bounded
   `MET-WP1-01` registry-foundation specification (issue
   [#836](https://github.com/thoth-pub/thoth/issues/836)) was independently
   reviewed and CTO-approved, and its slice is merged into `feature/metrics`
   from the `ADR-0009` sibling slice branch. The `MET-WP1-02` source-state
   foundation (issue
   [#841](https://github.com/thoth-pub/thoth/issues/841)) and the
   `MET-WP1-03` import-state foundation (issue
   [#863](https://github.com/thoth-pub/thoth/issues/863)) followed the same
   bounded path and are merged into `feature/metrics`. The completed
   `MET-MIG-V1.9-RECON-01` reconciliation (issue
   [#868](https://github.com/thoth-pub/thoth/issues/868)) then aligned those
   three merged migration directories with the `v1.9.0` release suffix,
   ledger-neutrally. The `MET-WP1-04` record-history foundation (issue
   [#872](https://github.com/thoth-pub/thoth/issues/872)) followed the same
   bounded specification path and is merged/delivered to `feature/metrics`
   through PR [#873](https://github.com/thoth-pub/thoth/pull/873). Decompose
   each further WP1 slice into its own bounded child issue/specification before
   any additional implementation; none exists, and none is authorized by this
   record.
6. Scope SPHINX-BOOT-01 (with BR-SPHINX-01) for WP6 and later Sphinx work, on
   its own path; it does not gate Thoth WP1 entry.