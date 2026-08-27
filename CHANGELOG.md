# Changelog
All notable changes to thoth will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
  - `ADM-ADR-01`: propose [`ADR-0010`](docs/engineering/decisions/ADR-0010-staff-operations-console.md), the Staff Operations Console architecture (issue [846](https://github.com/thoth-pub/thoth/issues/846)). The proposal reserves `/admin/*` for the superuser global control plane, keeps one publisher workspace entered through an explicit publisher operating context rather than identity impersonation, and makes Thoth the future canonical owner of staff-facing publisher-service operational records while separating desired state, execution, observed external state and attention/reconciliation. It defines `ServiceOperation` only as a common operational/audit seam — not a generic job/queue framework — preserves ADR-0002 platform-domain separation and ADR-0008 programme-local `distribution_job*`, and selects distribution as the first read-only operational integration before any replay or external-write activation. **Architecture/control proposal only: no runtime, schema, migration, GraphQL/API, generated client, workflow, provider, deployment, external-write, replay or production effect; no implementation is authorized.**
  - `CTRL-BRANCH-NAMESPACE-01`: correct the shared programme-integration branch namespace so a programme can keep a live `feature/<programme>` integration branch and still create bounded slice branches (issue [837](https://github.com/thoth-pub/thoth/issues/837)). Adds [`ADR-0009`](docs/engineering/decisions/ADR-0009-programme-integration-branch-namespace.md): programme integration branches remain `feature/<programme>`; `PROGRAMME_INTEGRATION` slice branches become **siblings** spelled `feature/<programme>--<slice>`; `--` is reserved as the programme/slice separator and may not appear inside a governed identifier; governed programme, area, slice and task identifiers are non-empty single Git path segments; `STANDARD` task branches remain `feature/<area>/<task>`; and a symmetric fail-closed live namespace preflight is required before any governed ref is created. Active shared doctrine no longer prescribes the descendant form `feature/<programme>/<slice>`, which cannot be created while `feature/<programme>` exists because Git cannot hold a ref and a ref namespace at the same path — the exact HTTP 422 `Reference update failed` that blocked `MET-WP1-01` [836](https://github.com/thoth-pub/thoth/issues/836) — and no longer conflates `STANDARD` and `PROGRAMME_INTEGRATION` naming as `feature/<programme-or-area>/<task-id-or-short-name>`. The rejected single-hyphen form `feature/<programme>-<slice>` is not adopted: `-` is not reserved and recreates the live `metrics` / `metrics-control` collision class. Registers `ADR-0009` and adds the `CTRL-BRANCH-NAMESPACE-01` implementation report. `ADR-0009` standardizes the repository ref spelling of programme slices only and does **not** amend the substantive Thoth Metrics architecture: each affected repository still owns its own `feature/metrics` integration branch, focused children are still created from it and merged back into it, and they still do not target `develop` directly. No branch is created, renamed, deleted or moved; `feature/metrics` is untouched; historical task, review, merge and lifecycle evidence is preserved as written rather than rewritten to the new spelling. `MET-WP1-01` remains **HOLD** and does not resume from this change: it requires its own task-specific amendment to `feature/metrics--wp1-registry-foundation`, fresh HOLD-sensitive verification, fresh required review and fresh bounded implementation authorization. **Documentation and control records only: no runtime, schema, data, migration, API, GraphQL, authorization, dependency, workflow, repository-settings or production effect.**
  - `MET-CTRL-01-CLOSEOUT-01`: correct the materially stale active Thoth Metrics programme state left after the `MET-CTRL-01` reconciliation merged (issue [834](https://github.com/thoth-pub/thoth/issues/834), parent task [832](https://github.com/thoth-pub/thoth/issues/832), parent programme [766](https://github.com/thoth-pub/thoth/issues/766)). Active controls now record `MET-CTRL-01` as `MERGED - COMPLETE` with its dependency **satisfied**, and stop asserting that the Metrics programme controls are under `MET-CTRL-01` reconciliation, that `MET-CTRL-01` is `ACTIVE` and awaiting independent exact-head review and merge, or that `MET-CTRL-01` closure is still an unsatisfied Thoth WP1 entry dependency. WP1 remains `HIGH` and `BLOCKED` with exactly two remaining entry gates — separately authorized creation of repository-local `feature/metrics` from a freshly verified `develop` head, and one approved bounded repository-local WP1 child specification — neither of which exists or is authorized by this change; `SPHINX-BOOT-01` has only its `MET-CTRL-01` dependency marked satisfied while `BR-SPHINX-01`, the approved bootstrap specification and every other Sphinx/WP6 blocker stay intact; and CG-08 records the satisfied `MET-CTRL-01` component but remains **OPEN** on the outstanding `feature/metrics` authorization and bounded WP1 child specification, without weakening or closing CG-03, CG-04, CG-09, CG-10, CG-11 or CG-13. Later Sphinx/WP6, client, source-specific/COUNTER/OPERAS and WP5 service-role gates remain attached to their owning later work. Adds the bounded `MET-CTRL-01-CLOSEOUT-01` task specification and implementation report. **Documentation and control records only: no runtime, schema, migration, SQL, `schema.rs`, Rust, GraphQL/API, generated contract, authorization, workflow, provider or production effect, and no Metrics implementation is authorized.** Under `ADR-0005` this change is a section-8 material programme-state correction, not lifecycle-metadata transcription: active trackers reference PR [833](https://github.com/thoth-pub/thoth/pull/833) as the parent lifecycle anchor and do not restate the reviewed source head, merge commit, review identifiers or merge-authorization identifiers; exact review and authorization provenance is retained in the owning `MET-CTRL-01` task and closeout evidence and is not asserted to be recorded in full on GitHub, and no further repository task or pull request will be created solely to record that this closeout itself merged. The prior `MET-CTRL-01` changelog entry below is preserved unchanged as historical evidence of that task's point-in-time gate language, and the two historical `MET-CTRL-01` control-process provenance exceptions — implementation mutations performed before a separate explicit implementation authorization was durably recorded, and a merge that was explicitly authorized before execution but whose authorization was not durably recorded on GitHub beforehand — are preserved distinctly and are neither erased, backdated nor retroactively cured. Issues [832](https://github.com/thoth-pub/thoth/issues/832) and [766](https://github.com/thoth-pub/thoth/issues/766) are untouched, and no `feature/metrics` branch and no WP1 child issue are created
  - `MET-CTRL-01`: reconcile the repository-backed Thoth Metrics programme controls with the live merged state and open the Thoth WP1 gate (issue [832](https://github.com/thoth-pub/thoth/issues/832), parent programme [766](https://github.com/thoth-pub/thoth/issues/766)). Documentation/control only: records the completed shared controls (ADR-0001/ADR-0002 approved and merged; ADR-0003 repository-authoritative through merged PR [778](https://github.com/thoth-pub/thoth/pull/778); ADR-0008 repository-authoritative within its approved scope; `THOTH-DB-CTRL-01` superseded), distinguishes the remaining Thoth-local WP1 entry gate (`MET-CTRL-01` closure, separately authorized `feature/metrics` creation, one approved bounded WP1 child specification) from the later Sphinx/WP6, client, source-specific and WP5 service-role gates, and reconciles CG-08 without weakening CG-03, CG-04, CG-09, CG-10, CG-11 or CG-13. No runtime, migration, schema, GraphQL/API, authorization, workflow, provider or production effect; no Metrics implementation is authorized
  - `MIG-01`: bounded administrative audit/backfill tooling for the canonical publisher package and distribution-platform configuration (issue [828](https://github.com/thoth-pub/thoth/issues/828), parent programme [765](https://github.com/thoth-pub/thoth/issues/765)). A new workspace-visible `thoth-api` administrative facade (`publisher_service_configuration::migration_backfill`) reads an immutable, raw-byte-`SHA-256`-identified input manifest, resolves each publisher against canonical Thoth identity, normalises the desired linked-platform set, and emits a deterministic, canonical, raw-byte-hashed dry-run plan plus a bounded reconciliation report; a separately supplied reviewed plan is then applied — pending entries only, after fail-closed resume classification (`REVIEWED_NOOP`/`PENDING`/`ALREADY_APPLIED_BY_THIS_PLAN`/`DRIFT`) — exclusively through the existing canonical service-configuration coordinator with a fixed `MIGRATION_BACKFILL` source and a plan-derived audit actor. A thin `thoth publisher-services migration-backfill dry-run|apply` CLI passes only paths and operational arguments. The tooling is job-free by design (it creates no `distribution_job`, `distribution_job_target` or `distribution_job_attempt`, with automatic job creation `OFF` or `ON`), audits and reports unsupported licence values and their reviewed dispositions without any licence-normalisation write, reports the per-publisher work-freshness lock footprint, and encodes the strict production job-state preflight. No database migration, `schema.rs`, GraphQL/API contract, authorization, worker/job schema or workflow change; no production or provider data is read or written and nothing is executed against production; automatic distribution-job creation remains `OFF`.

### Fixed
  - `CTRL-CI-CLIPPY-01`: restore the repository-wide Rust lint baseline on `develop` after Clippy 1.98 on the GitHub-hosted runner began rejecting three pre-existing `thoth-api/src/model/tests.rs` assertions under `clippy::useless_format` (issue [844](https://github.com/thoth-pub/thoth/issues/844)). The `test_doi_with_domain`, `test_orcid_with_domain` and `test_ror_with_domain` assertions now call `.to_string()` directly on the domain-qualified `Doi`, `Orcid` and `Ror` values instead of wrapping the same `Display` output in `format!("{}", ...)`. Test names, inputs and expected outputs are unchanged, and the correction adds no lint suppression and no lint-policy, dependency, toolchain or workflow change. Test-only: no production `Doi`/`Orcid`/`Ror`, `Display`, `with_domain`, parsing, schema, migration, GraphQL/API, authorization or runtime behaviour is affected.

## [[1.7.0]](https://github.com/thoth-pub/thoth/releases/tag/v1.7.0) - 2026-08-18
### Added
  - `CTRL-DELIVERY-02-RECONCILE`: reconcile the canonical repository, contract and control records with the five merged repository-local stages of programme `CTRL-DELIVERY-02` (issue [825](https://github.com/thoth-pub/thoth/issues/825), parent programme [824](https://github.com/thoth-pub/thoth/issues/824)). Repository-local root `AGENTS.md` controls are now merged and authoritative in every managed repository, verified live 2026-08-16 at `metrics-dashboard` `dev` `963b0ea7` (PR [#10](https://github.com/thoth-pub/metrics-dashboard/pull/10)), `metrics-widget` `dev` `363bce44` (PR [#2](https://github.com/thoth-pub/metrics-widget/pull/2)) and `cc-license` `develop` `3dd49798` (PR [#2](https://github.com/thoth-pub/cc-license/pull/2), preceded by supporting Clippy repair PR [#4](https://github.com/thoth-pub/cc-license/pull/4)); `control-gaps.md` CG-05 therefore no longer lists any repository as outstanding for the instruction item, and `agent-instructions/rollout-plan.md` no longer states that those three lack a root `AGENTS.md`. **`thoth-pub/baboon` is registered as a managed repository** through a new `repository-map/repositories/baboon.md`, the `repository-map/README.md` index and a `branch-topology.md` row: a private Rust library-oriented MARC exchange service on conforming `develop -> release/* -> master` topology (`develop` `bdf0ee33`, PR [#16](https://github.com/thoth-pub/baboon/pull/16)) that consumes the Thoth GraphQL and export APIs, persists state and cache in S3-compatible object storage, delivers through SFTPGo, and keeps ordinary CI distinct from its HIGH-risk pull-request-triggered live SFTP tests — which write to the production SFTP service, confined to the unwatched `/manchester/.ci-healthcheck/` scratch folder — and from its manual production feed. `contracts.md` records Baboon as a verified consumer requiring upstream impact analysis, and records the verified `thoth-export-server` dependency on published crate `cc_license = "0.1.0"` owned by `thoth-pub/cc-license`, a release predating this programme; no crate publication occurred. `thoth-pyramid`'s repaired `dev` CI **trigger** coverage (`push` and `pull_request` on `main` and `dev`, PR [#17](https://github.com/thoth-pub/thoth-pyramid/pull/17), `dev` `8f2d6faf`) is recorded as resolved in CG-11 and its repository entry. Everything else stays open and distinct: CG-11's four items (App lint/build/codegen, dashboard CI/tests, widget unit tests, cc-license old Actions), Pyramid's broader CI-quality, formatting, dependency/build and codegen concerns, CG-03, CG-04 and all `BR-` branch normalization, Strapi's Docker/Node CI defect, dissemination's README/environment-protection contradiction, CG-08 and CG-13. Documentation and control records only: no source, schema, migration, API/GraphQL contract, authorization, workflow, provider, runtime or deployment change, no Metrics [766](https://github.com/thoth-pub/thoth/issues/766) architecture or implementation change, and no change to any other repository
  - [823](https://github.com/thoth-pub/thoth/pull/823) - `CTRL-DELIVERY-01-CLOSEOUT-01`: reconcile the active Shared Engineering Control rollout and control-gap records with the repository-local instruction controls that have already been independently reviewed and merged (issue [819](https://github.com/thoth-pub/thoth/issues/819), parent programme [818](https://github.com/thoth-pub/thoth/issues/818)). Re-verified live 2026-08-15 that each managed repository carries a merged, authoritative repository-local root `AGENTS.md` on its active development branch at the exact head its instruction PR merged as: `thoth-app` `dev` `7a4e7c6c` (PR [#114](https://github.com/thoth-pub/thoth-app/pull/114)), `thoth-dissemination` `develop` `71ef7724` (PR [#96](https://github.com/thoth-pub/thoth-dissemination/pull/96)), standalone `thoth-client` `develop` `d6ffdc67` (PR [#55](https://github.com/thoth-pub/thoth-client/pull/55)), `thoth-pyramid` `dev` `2ee7a71f` (PR [#15](https://github.com/thoth-pub/thoth-pyramid/pull/15)), `thoth-strapi` `develop` `30622032` (PR [#5](https://github.com/thoth-pub/thoth-strapi/pull/5)) and `thoth-sphinx` `develop` `ff7de985` (reconciliation PR [#4](https://github.com/thoth-pub/thoth-sphinx/pull/4)). `agent-instructions/rollout-plan.md` no longer states that `thoth-app` has no verified root `AGENTS.md` or that `thoth-dissemination` still needs its repository-local control revision, and no longer omits the completed standalone `thoth-client`, `thoth-pyramid` and `thoth-strapi` rollouts in a way that implied they remained outstanding. `repository-map/control-gaps.md` CG-05 is narrowed to the repositories that genuinely remain outstanding — `metrics-dashboard`, `metrics-widget` and `cc-license`, each re-verified live as still having no root `AGENTS.md` on any branch. The instruction item is the **only** thing closed: CG-03 remains **OPEN** for Sphinx branch normalization and bootstrap readiness, `BR-SPHINX-01` and `SPHINX-BOOT-01` remain separate, branch-topology normalization (including `BR-APP-01` and `BR-DIS-01`) remains separate, the CG-11 CI gaps, Pyramid's dev-target CI gap and Strapi's Docker/Node CI defect remain separate, dissemination's README/environment-protection contradiction remains a separate follow-up, and CG-08 Metrics readiness and Metrics-specific documentation remain a separate programme. Documentation and control records only: no source, schema, migration, API/GraphQL contract, authorization, workflow, provider, runtime or deployment change, and no change to any other repository
  - `CTRL-REPO-THOTH-01`: reconcile the canonical Thoth engineering-delivery doctrine with the approved Thoth Engineering Control & Delivery model (issue [819](https://github.com/thoth-pub/thoth/issues/819), parent programme [818](https://github.com/thoth-pub/thoth/issues/818)). Replaces generic implementation-agent permission grants in root `AGENTS.md` and `docs/engineering/ai-delivery/operating-model.md` with a **granular, deny-by-default, non-transitive** action-authorization model covering repository/GitHub read, source write, new-file creation, delete/move/rename, branch, commit, push, PR mutation, issue mutation, manual CI dispatch, provider/runtime read and write, migration execution, release/publication, merge, deployment and production activation, so authorization for one action never implies another. Establishes GitHub issues/PRs/reviews/CI as the durable live task ledger and adds a formal cross-repository impact-analysis gate (`operating-model.md` section 4.1) that every substantive/contract-affecting task must pass before repository scope is approved. Extends `task-specification-template.md`, `implementation-report-template.md` and `independent-review-template.md` with owning-issue/programme-stage/exact-base-SHA identity, write-budget and action-authorization fields, cross-repository impact/consumer-compatibility fields, and authorized-vs-actual/write-budget-compliance verification. Adds a new reusable `implementation-handoff-template.md` for bounded coding-agent prompts. Scopes `branching-and-release-workflow.md` to `thoth-pub/thoth` and makes other repositories' branch topology authoritative in their own repository-map entries rather than imposing this repository's `develop`/`master` model on them. Adds `docs/engineering/repository-map/contracts.md`, verified repository entries for standalone `thoth-pub/thoth-client` (Python, PyPI `thothlibrary`), `thoth-pub/thoth-pyramid` and `thoth-pub/thoth-strapi`, and explicitly distinguishes the standalone Python `thoth-client` from the internal Rust `thoth-client` workspace member in this repository. Reconciles `branch-topology.md` and `repository-map/README.md` against live-verified branch state (`thoth-app`/`thoth-pyramid`: `dev`; `thoth-dissemination`/`thoth-strapi`/standalone `thoth-client`: `develop`; `thoth-sphinx`: both `main` and `develop` exist and remain identical placeholder-only, re-verified 2026-08-15). **Documentation and control records only: no runtime, schema, migration, GraphQL, authorization implementation, CI workflow, branch-protection or deployment change.** This shared doctrine must be independently reviewed and merged before repository-local control tasks `#113`, `#95`, `#54`, `thoth-sphinx#1`, `thoth-pyramid#14` and `thoth-strapi#4` may scope their own controls against it
  - `BE-04`: implement **durable distribution jobs** against the approved repository-authoritative specification (`docs/engineering/ai-delivery/tasks/BE-04.md`, merged through PR [814](https://github.com/thoth-pub/thoth/pull/814)). Adds the additive migration `20260814_v1.7.0` creating the four closed enums — `distribution_job_kind`, `distribution_job_status`, `distribution_job_attempt_result` and `distribution_job_cancellation_reason`, none with `OTHER`, `UNKNOWN` or a `Default` — and the `distribution_job`, `distribution_job_target` and `distribution_job_attempt` relations with every named foreign key, unique constraint, check constraint and index of the specification, including the claim-state check that makes the claim token, worker identity, claim time and lease expiry non-null **if and only if** the job is `RUNNING`, the database-enforced `0 <= attempt_count <= 5` bound tied by test to `DISTRIBUTION_JOB_MAX_ATTEMPTS`, and the deduplication-key formula check that proves the stored key equals `PUBLISHER_BACK_CATALOGUE:<publisher_id>:<activation_id>`, and the **NULL-safe** attempt-error check `(error_code IS NULL AND error_detail IS NULL) OR (result IS NOT NULL AND result = 'FAILED')`, which — unlike the expression PostgreSQL admits whenever it evaluates to `UNKNOWN` — refuses an **open** attempt carrying error fields, proven as a full three-valued truth table on `INSERT` and `UPDATE` with each rejection attributed to that constraint by name. `thoth-api/src/schema.rs` is updated manually and atomically under `ADR-0003` Architecture A. Adds the programme-local Rust domain model with **no** generic `Crud`, no generic `Job`/`Queue`/`Lease`/`Worker` abstraction and no shared claim protocol, keeping `BE-04`'s tables, types and lifecycle APIs Publisher-Services-specific under `ADR-0008` sections 3.4 and 3.5. Widens `BE-02`'s connection-scoped lifecycle outcome to `Unchanged`/`Activated`/`Repaired`/`Disabled`, decided inside `enable_on` from the member rows it already reads and leaving `BE-02`'s public behaviour, pool-level signatures and merged semantics unchanged, and extends **`BE-03`'s existing single coordinator transaction in place** — no second transaction, savepoint, hook, callback or after-the-fact path — so a job cannot exist without the desired-state change that justified it and that change cannot commit without the job it qualified for. A linked OAPEN/DOAB activation produces exactly one job with two targets from one shared activation identity; a **repair** produces none, because a repair is not a new zero-enabled-to-enabled activation, and that implies nothing whatever about prior delivery, adapter execution or back-catalogue presence; `PullFeed`, `Manual`, package-only, no-op, stale and `MIGRATION_BACKFILL` writes produce none; and non-assignable `JISC_NBK` can never become a target. Adds the complete state machine with atomic claims, leases, budget-aware lease-expiry recovery, deterministic bounded backoff and cancellation, enforced by PostgreSQL: the claim is one atomic statement that selects with `FOR UPDATE SKIP LOCKED` in deterministic `(available_at, distribution_job_id)` order, mints one distinct token per job, inserts exactly one attempt row per claimed job and **returns exactly the jobs it claimed**, with the whole claim payload resolved in a constant four statements at batch sizes 1, 10 and 50; an expired lease within budget returns the job to `PENDING` without moving the attempt count, and an expired final attempt transitions **directly to `FAILED`**, so no `attempt_number = 6` is reachable on any path of reported failures, crashes, expiries and re-claims. Adds one narrow unscoped `DISSEMINATION_WORKER` project role with one explicit guard, permitting exactly `claimDistributionJobs`, `completeDistributionJob` and `failDistributionJob` and conferring no publisher scope, no `CDN_WRITE` capability and no Metrics permission, with `SUPERUSER` **denied** those three operations and retaining superuser-only `cancelDistributionJob` and the staff report; `is_superuser()` behaviour is byte-identical and no role-composition rule is introduced. Adds the four additive mutations and their types, with the claim token returned **only** on `ClaimedDistributionJob` and never on `DistributionJob`, and exactly four new `ThothError` variants with four explicit `into_field_error` arms. Extends `BE-03`'s superuser staff report additively with a nullable `latestBackCatalogueJob` and `jobStatuses`/`withoutBackCatalogueJob` filters on both the list and the count, where "no job" is `null` and only `null` and is never read as evidence that delivery did or did not occur, backed by **one** first-level request-local `ADR-0007` composite loader keyed by `publisher_id` whose value is the complete field — the latest job together with its targets and its attempts, or `null` — resolved in one `spawn_blocking` boundary on one pooled connection as three set-based statements per dispatch chunk, and only one for a chunk whose latest-job statement returns nothing, so the report's statement count follows the specified per-chunk arithmetic `2 + 3 * C_job_nonempty + 1 * C_job_empty + 1 * C_assign` and was measured as **five** (job-only) and **six** (full report) on a page containing at least one job and **three** and **four** on a page with none, at page sizes 1, 25 and 200 alike, reproducibly, with the second-level target and attempt loaders recording **zero** dispatches on the report path and retained only for the single-job mutation payloads; `ADR-0007`'s `200`/`10` configuration is unchanged, and no look-ahead, request-scoped result store or loader merge was introduced. **Automatic creation is `OFF` by default and merging this activates nothing**: `THOTH_DISTRIBUTION_JOB_CREATION` is registered on both production-capable command paths, `start graphql-api` and `init`, reaches the resolver through `Context` and the write context rather than any ambient lookup, runs no sweep, backfill, startup scan or lazy creation, and while it is `OFF` a `SUPERUSER_API` transaction producing a qualifying `AutomaticPush` activation **fails closed and rolls back in full**, leaving assignment state, `activation_id`, the configuration token, the publisher row, the audit table, the works' freshness signal and the job tables with zero committed change. The migration creates zero rows and changes no existing row; establishing its two foreign keys does take `SHARE ROW EXCLUSIVE` locks on the existing `public.publisher` and `public.work` tables, which is measured from a second session with a duration, a deterministic lock-contention result under a `lock_timeout` and an unchanged-`relfilenode` proof, and the foreign keys are not weakened to avoid it. `zitadel setup` declares the new role and grants nothing. **Merge authorizes repository integration only**: no deployment, no environment or production migration execution, no identity-provider change, no role grant or credential provisioning, no worker deployment, no `OFF -> ON` activation, no pilot, no dissemination, no external platform contact and no production access, and PR [799](https://github.com/thoth-pub/thoth/pull/799) is untouched. Delivered on `feature/publisher-services/be-04` through PR [816](https://github.com/thoth-pub/thoth/pull/816), and reconciled in place against the corrected specification `BE-04-SPEC-ADDENDUM-01` (PR [817](https://github.com/thoth-pub/thoth/pull/817)) under a fresh CTO implementation reconciliation authorization bound to that corrected base: the two corrections the addendum required are the NULL-safe attempt-error check and the single composite report loader described above, the section 25.12 statement-count test now derives every expectation from the measured per-chunk classification instead of accepting the withdrawn divergence, and the generated SDL is byte-identical across the reconciliation
  - [815](https://github.com/thoth-pub/thoth/pull/815) - `ADR-0008-RECORD`: record the CTO-approved cross-programme decision `ADR-0008` - **machine roles and durable job primitives** (`docs/engineering/decisions/ADR-0008-machine-roles-and-durable-job-primitives.md`), approved by Javi, CTO on 2026-08-14. Establishes **domain-specific, least-privilege machine-role conventions** for `thoth`: no generic `SERVICE`/`MACHINE`/`WORKER`/`SERVICE_ACCOUNT` catch-all role, an unscoped machine role only for a genuinely global workload, and an explicit policy predicate/guard, an explicit authorization matrix and least privilege for every machine role, with `SUPERUSER` authority not automatically conferring machine-role authority. That `SUPERUSER`/machine-role boundary is the whole of what the ADR decides about how roles relate: it states no general role-composition, role-aggregation or role-inheritance rule, leaving whether one machine role may imply or compose with another to the owning approved authorization matrix or to a later explicit architecture decision. Those requirements are the whole of the approved cross-programme machine-role rule: enumerated permitted-operation lists, enumerated forbidden-operation lists and separate provisioning/credential controls are **not** approved `ADR-0008` architecture and apply only where existing repository, deployment or identity-provider controls or an adopting task's own approved specification independently require them; `ADR-0008` decides no provisioning mechanism, credential store, rotation policy or identity-provider arrangement. Approves **`DISSEMINATION_WORKER` as a Publisher-Services-specific** machine role for the BE-04/DIS-02 durable distribution workflow, to be implemented later with exactly the permissions its independently reviewed and approved BE-04 specification defines; the operation-level authorization matrix stays with that specification, and the role authorizes no Thoth Metrics operation and determines no Metrics role name, permission or readiness. Establishes exactly seven shared **durable-job and concurrency conventions** — PostgreSQL durability, explicit state machines, database uniqueness, leases, claim tokens, deterministic idempotency and `FOR UPDATE SKIP LOCKED` where justified — **without creating either a generic job framework or a reusable cross-programme job API**: an approved convention is not a mandatory mechanism, `SKIP LOCKED` in particular must still be justified by the adopting task rather than copied mechanically, and every other concurrency or retry mechanism remains governed by existing repository controls and the adopting task's own approved specification rather than approved here as an additional cross-programme convention. Keeps BE-04's `distribution_job`, `distribution_job_target` and `distribution_job_attempt` tables, Rust domain types and lifecycle APIs **programme-local**, reusable by no other programme by analogy, and **requires a separate explicit cross-programme ADR** before any future reusable generic job or queue abstraction may be implemented. Reconciles the shared decision register, records the shared machine-role convention as WP5's former "role decision" dependency in the Thoth Metrics tracker under `ADR-0008`'s authority condition, and records the new `ADR-0008` control boundary in the Publisher Services tracker. **Documentation and control records only: no runtime, `policy.rs`, machine-role, identity-provider, provisioning, schema, migration, GraphQL, generated contract or workflow change.** The ruling must be recorded in a shared repository ADR before `BE-04` implementation is authorized; separately, and under the repository's existing process controls rather than as approved decision content, the record is repository-authoritative only when its exact approved content is independently reviewed at its exact head and reachable from `develop`, and a branch carrying `APPROVED` is not authoritative before it merges. **No implementation or production action is authorized**: `BE-04` implementation remains NOT AUTHORIZED and its specification candidate is not approved by this decision, Thoth Metrics `WP5` implementation remains NOT AUTHORIZED with `WP5` still `CRITICAL` and `BLOCKED` under WP4 and its own approved bounded slice specifications, and no machine-role creation, role provisioning, identity-provider change, worker deployment, durable job creation, dissemination, deployment or production access follows from it
  - `BE-04-SPEC`: propose the complete bounded implementation specification for `BE-04` - **durable distribution jobs** (`docs/engineering/ai-delivery/tasks/BE-04.md`), and remediate it against three independent review rounds and repository-authoritative `ADR-0008`. Specifies durable publisher back-catalogue jobs and their persistence: the closed `distribution_job_kind` (`PUBLISHER_BACK_CATALOGUE`), `distribution_job_status` (`PENDING`, `RUNNING`, `SUCCEEDED`, `FAILED`, `CANCELLED`), `distribution_job_attempt_result` (`SUCCEEDED`, `FAILED`, `CANCELLED`, `ABANDONED`) and `distribution_job_cancellation_reason` (`ADMINISTRATIVE`, `ASSIGNMENT_DISABLED`) types — none with `OTHER`, `UNKNOWN` or a `Default` — and the `distribution_job`, `distribution_job_target` and `distribution_job_attempt` relations with every named foreign key, unique constraint, check constraint and index fixed, including a claim-state check making the claim token, worker identity, claim time and lease expiry non-null **if and only if** the job is `RUNNING`, a database-enforced `0 <= attempt_count <= 5` bound tied by test to the `DISTRIBUTION_JOB_MAX_ATTEMPTS` constant, an immutable composite-keyed target relation, and an insert-once/close-once attempt relation whose `UNIQUE (claim_token)` binds a token to exactly one attempt for all time. Defines **atomic creation rules and deduplication**: the deduplication key is derived from `BE-02`'s actual activation semantics — one shared `activation_id` per linked group, newly minted on every enable, re-enable and repair — as the exact formula `PUBLISHER_BACK_CATALOGUE:<publisher_id>:<activation_id>`, stored in one kind-agnostic unique text column and proven equal to that formula by a database check constraint built only from immutable expressions, so repeated creation is idempotent through `ON CONFLICT DO NOTHING` rather than an application check-then-insert, a linked OAPEN/DOAB activation yields exactly **one** job with **two** targets and one adapter execution, and a genuine re-enable yields a new legitimate onboarding job. Fixes the complete creation matrix — a qualifying **new** activation with at least one `AutomaticPush` member creates a job; a linked-state **repair** does not, because a repair is **not** a new zero-enabled-to-enabled activation, and that carries **no inference** about prior delivery, adapter execution or back-catalogue presence; `PullFeed` and `Manual` destinations never create an uploader job; package-only, no-op and stale replacements create nothing; `MIGRATION_BACKFILL` creates nothing unconditionally and commits normally, so `MIG-01` imports remain job-free as an explicit migration boundary; and non-assignable `JISC_NBK` can never become a target — resolved from code-owned descriptors rather than destination names, and requiring `BE-02`'s connection-scoped enable primitive to distinguish `Activated` from `Repaired` from rows it already reads, without changing `BE-02`'s public behaviour. Requires creation to extend **`BE-03`'s existing single coordinator transaction** between the lifecycle writes and the single publisher `UPDATE` — never a second transaction, savepoint, hook, callback or after-the-fact best-effort path — with the exact step order, the accepted inheritance of the `AFTER UPDATE` `set_work_updated_at_with_relations` cascade and its N-work-row lock footprint, an unwidened audit key set, and the invariant that a job can never exist without a token bump and exactly one audit row. Defines **claims, leases, retry and cancellation**: `FOR UPDATE SKIP LOCKED` selected against four recorded alternatives, deterministic `(available_at, distribution_job_id)` ordering, a bounded clamped batch, an eligibility predicate requiring every target to still be enabled **under the job's own activation** **and** the attempt budget to be unexhausted, a single atomic claim statement that inserts exactly one attempt per claimed job and **returns exactly the jobs it claimed** (zero rows when none), with targets and payloads resolved by bounded set-based statements and no N+1 path, attempt-row creation atomic with the claim, **budget-aware lease-expiry recovery** that returns a job to `PENDING` within budget but transitions an expired final attempt **directly to `FAILED`** so five attempts can never become six on any path, absolute-timestamp exponential backoff, retry encoded by `failDistributionJob(retryable: true)` rather than an invented mutation, and a superuser-only cancellation that invalidates a live claim, fails closed from every terminal state, deletes no history and is documented as unable to undo an already-performed external upload. Defines **worker authorization and API**: one narrow least-privilege unscoped `DISSEMINATION_WORKER` project role — approved by `ADR-0008` as **Publisher-Services-specific**, with its operation-level matrix owned by this specification — expressed through the unscoped-role check pattern `SUPERUSER` already establishes, permitting exactly `claimDistributionJobs`, `completeDistributionJob` and `failDistributionJob` and conferring no publisher scope, no Metrics permission and no `CDN_WRITE` capability, with `SUPERUSER` deliberately **denied** the three worker operations as BE-04's own least-privilege choice — `ADR-0008` requires neither allowing nor denying it, and sharing an unscoped check *pattern* is not sharing authority — while retaining cancellation and the staff report, `CDN_WRITE` reuse explicitly rejected, worker identity derived from the authenticated account rather than accepted as an input, the claim token exposed only in the claim payload, and exactly four new error variants (`STALE_DISTRIBUTION_JOB_CLAIM`, `DISTRIBUTION_JOB_TERMINAL`, `DISTRIBUTION_JOB_CREATION_DISABLED` and `INVALID_DISTRIBUTION_JOB_ERROR_CODE`, the last being the named contract for a malformed or over-length worker `errorCode`, which changes no job or attempt state, echoes no part of the rejected value and deliberately does not map to `INTERNAL_ERROR`). Fixes the top-level `lastErrorCode`/`lastErrorDetail` semantic as **the most recent worker-reported failure** - set by the two worker-failure transitions, cleared on success, and left untouched by lease-expiry recovery and by every form of cancellation - so a job terminalized by an expired final attempt may legitimately carry a null last error, a retained older failure is never described as the abandonment or terminalization cause, and attempt history remains the authoritative record of how a job ended. Defines **additive job-aware reporting**: one nullable `latestBackCatalogueJob` field on `BE-03`'s superuser-only summary plus `jobStatuses` and `withoutBackCatalogueJob` filters on the report and its count, with "no job" represented by `null`, never collapsed into `FAILED`, `PENDING`, `UNKNOWN` or zero, and never read as evidence that delivery did or did not occur, `BE-03`'s configuration-only surfaces and publisher-user read left untouched, and three new `ADR-0007` request-local loaders - latest job, targets and attempts - keeping the report's statement count constant in page size at two selection-dependent bounds — **six** set-based statements per page for the full report selection, which includes `enabledDistributionPlatforms` and so invokes `BE-02`'s existing assignment loader, and **five** for the full job-only selection, which does not — both stated per dispatch chunk rather than assumed to be one, and both required to be measured at page sizes 1, 25 and 200 so the test pins selection-dependent execution rather than merely counting queries. **Leaves automatic creation inactive, and makes `OFF` fail closed**: a new `THOTH_DISTRIBUTION_JOB_CREATION` switch defaulting to `OFF` following the merged mutation-guard convention, consulted at exactly one site inside the coordinator, with **no** sweep, backfill, startup scan or lazy creation so enabling it can never enqueue an existing back catalogue retroactively — and, while it is `OFF`, a `SUPERUSER_API` configuration transaction producing a new `Activated` group with at least one `AutomaticPush` member **fails and rolls back in full** rather than committing an activation whose onboarding job would never be created, leaving assignment state, `activation_id`, the configuration token, the publisher row, the audit table and the job tables with zero committed change, while `PullFeed`, `Manual`, package-only, repair, disable and `MIGRATION_BACKFILL` writes remain permitted. **Consumes `ADR-0008` exactly**: the cross-programme machine-role and durable-job question this candidate escalated is resolved by `ADR-0008`, repository-authoritative through PR [815](https://github.com/thoth-pub/thoth/pull/815), which approves `DISSEMINATION_WORKER` as Publisher-Services-specific, keeps BE-04's `distribution_job*` tables, Rust domain types and lifecycle APIs programme-local, creates no generic shared job framework, and holds any future generic job/queue abstraction behind its own later cross-programme ADR; the specification enumerates the seven approved shared conventions exactly and separately attributes its own stale-token rejection, deterministic ordering, database-enforced concurrency, bounded lease semantics, deduplication formula, GraphQL worker protocol and operation lists to BE-04's own HIGH-risk requirements, to the root and `thoth-api` `AGENTS.md` controls, or to outside the task, rather than presenting any of them as additional `ADR-0008`-approved cross-programme architecture. Thoth Metrics `WP5` does not use `DISSEMINATION_WORKER` and its eventual role name and permissions remain its own work. Records the future migration's **true locking model**: the `distribution_job*` relations are new and empty and no existing application table is rewritten, but establishing the specified foreign keys to `public.publisher` and `public.work` does take `SHARE ROW EXCLUSIVE` locks on those existing tables, blocking concurrent writes to them for the duration, so the migration requires observed `pg_locks` evidence, a duration, a deterministic lock-contention result and an unchanged-`relfilenode` proof, and its production execution remains a separately authorized operation whose lock window must be accounted for; the foreign keys are not weakened to avoid this. Narrows the dual-role authorization rationale so that permitting a principal holding both `SUPERUSER` and `DISSEMINATION_WORKER` to exercise both roles' independently authorized operations is stated as a BE-04-specific matrix decision that establishes **no** general role-composition, aggregation or inheritance rule. **Specification and control records only - no runtime, Cargo, GraphQL, schema, migration, workflow or infrastructure change, no migration created or executed; `schema.rs` and `policy.rs` untouched; no implementation, and no implementation branch created.** The specification is **not approved**: three independent review rounds identified remediation requirements, which this change addresses, and fresh exact-head independent review and explicit CTO specification approval both remain required. Satisfying `ADR-0008` is a **necessary and not a sufficient** condition. `BE-04` implementation remains **NOT AUTHORIZED** and `feature/publisher-services/be-04` must not exist; automatic job creation is **not activated**; no `distribution_job`, `distribution_job_target` or `distribution_job_attempt` relation and no distribution job exists; and `MIG-01`, `APP-01`, `APP-02`, `DIS-01`, `DIS-02`, deployment, environment and production migration execution, worker deployment, credential provisioning, dissemination, distribution activation, `OBSERVE`/`ENFORCE`, workflow dispatch, production access and PR [799](https://github.com/thoth-pub/thoth/pull/799) all remain outside scope and unauthorized

### Removed
  - [778](https://github.com/thoth-pub/thoth/pull/778) - Remove the stale, unused root `diesel.toml`, which never parsed, did not target the canonical `thoth-api/src/schema.rs`, and was not part of any supported build, test, migration, or schema-generation command; the Diesel Rust crates and the embedded `diesel_migrations` runner are unaffected

## [[1.6.3]](https://github.com/thoth-pub/thoth/releases/tag/v1.6.3) - 2026-08-13
### Changed
  - [806](https://github.com/thoth-pub/thoth/pull/806) - Enforce that a book chapter Work (`work_type = 'book-chapter'`) may be attached to at most one distinct parent Work: assigning a second distinct `is-child-of` parent, or changing a Work's `work_type` to `book-chapter` while it already has more than one distinct parent, is now rejected (`work_relation_single_book_chapter_parent`). Zero-parent chapters remain valid and non-book-chapter Works are unaffected (#803)

## [[1.6.2]](https://github.com/thoth-pub/thoth/releases/tag/v1.6.2) - 2026-08-10
### Fixed
  - [794](https://github.com/thoth-pub/thoth/pull/794) - Declare the `xlink` namespace on the Crossref DOI deposit root element (`CROSSREF_NS`), so the DOI deposit XML binds the `xlink` prefix required by JATS abstract links using `xlink:href`, and deposits whose abstract markup contains `<jats:ext-link xlink:href="...">` are no longer rejected by Crossref with "The prefix "xlink" for attribute "xlink:href" ... is not bound"

## [[1.6.1]](https://github.com/thoth-pub/thoth/releases/tag/v1.6.1) - 2026-07-23
### Fixed
  - [763](https://github.com/thoth-pub/thoth/pull/763) - Validate a publication upload's canonical location before copying the object to its final key and inserting the file record in `complete_file_upload`, so a `LocationUrlError` (e.g. a work with no landing page) no longer leaves an orphaned file record and S3 object behind (`FileUpload::precheck_related_metadata`)

## [[1.6.0]](https://github.com/thoth-pub/thoth/releases/tag/v1.6.0) - 2026-07-21
### Fixed
  - [762](https://github.com/thoth-pub/thoth/pull/762) - Always invalidate the CloudFront cache for the canonical object key when completing a file upload (`storage::reconcile_replaced_object`), so replacing a cover already cached at that URL no longer serves the stale object when no managed file record existed before

## [[1.5.0]](https://github.com/thoth-pub/thoth/releases/tag/v1.5.0) - 2026-07-17
### Changed
  - [761](https://github.com/thoth-pub/thoth/pull/761) - Restrict cover image format to JPEG

## [[1.4.0]](https://github.com/thoth-pub/thoth/releases/tag/v1.4.0) - 2026-07-16
### Added
  - [757](https://github.com/thoth-pub/thoth/pull/757) - Add `oclc_number` column to OCLC KBART export (`kbart::oclc`)

### Fixed
  - [758](https://github.com/thoth-pub/thoth/pull/758) - Fixed `clippy::useless_borrows_in_formatting` lints raised by newer Rust in formatting macros
  - [760](https://github.com/thoth-pub/thoth/pull/760) - Export contributors with their true role code in Google Books ONIX (`onix3::google_books`), instead of re-coding the first contributor of a wholly-edited book as an author (`A01`)

## [[1.3.4]](https://github.com/thoth-pub/thoth/releases/tag/v1.3.4) - 2026-07-09
### Changed
  - [756](https://github.com/thoth-pub/thoth/pull/756) - Upgrade `actix-http` to v3.13.1
  - [756](https://github.com/thoth-pub/thoth/pull/756) - Upgrade `actix-web` to v4.14.0
  - [756](https://github.com/thoth-pub/thoth/pull/756) - Upgrade `aws-config` to v1.9.0
  - [756](https://github.com/thoth-pub/thoth/pull/756) - Upgrade `aws-credential-types` to v1.3.0
  - [756](https://github.com/thoth-pub/thoth/pull/756) - Upgrade `aws-sdk-cloudfront` to v1.124.0
  - [756](https://github.com/thoth-pub/thoth/pull/756) - Upgrade `aws-sdk-s3` to v1.138.0
  - [756](https://github.com/thoth-pub/thoth/pull/756) - Upgrade `chrono` to v0.4.45
  - [756](https://github.com/thoth-pub/thoth/pull/756) - Upgrade `diesel` to v2.3.10
  - [756](https://github.com/thoth-pub/thoth/pull/756) - Upgrade `diesel-derive-newtype` to v2.1.3
  - [756](https://github.com/thoth-pub/thoth/pull/756) - Upgrade `env_logger` to v0.11.11
  - [756](https://github.com/thoth-pub/thoth/pull/756) - Upgrade `jsonwebtoken` to v10.4.0
  - [756](https://github.com/thoth-pub/thoth/pull/756) - Upgrade `log` to v0.4.33
  - [756](https://github.com/thoth-pub/thoth/pull/756) - Upgrade `pulldown-cmark` to v0.13.4
  - [756](https://github.com/thoth-pub/thoth/pull/756) - Upgrade `regex` to v1.12.4
  - [756](https://github.com/thoth-pub/thoth/pull/756) - Upgrade `serde_json` to v1.0.150
  - [756](https://github.com/thoth-pub/thoth/pull/756) - Upgrade `tokio` to v1.52.3
  - [756](https://github.com/thoth-pub/thoth/pull/756) - Upgrade `tonic` to v0.14.6
  - [756](https://github.com/thoth-pub/thoth/pull/756) - Upgrade `uuid` to v1.23.4

### Security
  - [756](https://github.com/thoth-pub/thoth/pull/756) - Upgrade `cmov` to v0.5.4
  - [756](https://github.com/thoth-pub/thoth/pull/756) - Upgrade `openssl` to v0.10.81

## [[1.3.3]](https://github.com/thoth-pub/thoth/releases/tag/v1.3.3) - 2026-07-07
### Fixed
  - [754](https://github.com/thoth-pub/thoth/pull/754) - Fixed JATS list handling so ordered, bullet, and untyped lists are preserved across JATS, HTML, and Markdown conversions

## [[1.3.2]](https://github.com/thoth-pub/thoth/releases/tag/v1.3.2) - 2026-05-14
### Fixed
  - [751](https://github.com/thoth-pub/thoth/pull/751) - Escape XML reserved characters when writing JATS-formatted text to database

## [[1.3.1]](https://github.com/thoth-pub/thoth/releases/tag/v1.3.1) - 2026-05-06
### Security
  - Upgrade `openssl` to v0.10.79

## [[1.3.0]](https://github.com/thoth-pub/thoth/releases/tag/v1.3.0) - 2026-05-06
### Fixed
  - [749](https://github.com/thoth-pub/thoth/issues/749) - Correct locale code formatting in Crossref metadata output

### Changed
  - [749](https://github.com/thoth-pub/thoth/issues/749) - Remove ISBN limit in Crossref metadata output (introduced in v0.8.7)
  - [748](https://github.com/thoth-pub/thoth/pull/748) - Require endorsement author names and featured video titles

## [[1.2.0]](https://github.com/thoth-pub/thoth/releases/tag/v1.2.0) - 2026-05-04
### Added
  - [747](https://github.com/thoth-pub/thoth/pull/747) - Add `checksum` and `checksum_algorithm` fields to `Location`

## [[1.1.1]](https://github.com/thoth-pub/thoth/releases/tag/v1.1.1) - 2026-04-24
### Security
  - Upgrade `openssl` to v0.10.78
  - Upgrade `actix-http` to v3.12.1
  - Upgrade `rustls-webpki` to v0.103.12
  - Upgrade `rand` to v0.8.6

## [[1.1.0]](https://github.com/thoth-pub/thoth/releases/tag/v1.1.0) - 2026-04-17
### Added
  - [745](https://github.com/thoth-pub/thoth/pull/745) - Add Venda locale support for titles/books with `ve` and `ve-ZA`
  - [745](https://github.com/thoth-pub/thoth/pull/745) - Implement markup support for series description

## [[1.0.5]](https://github.com/thoth-pub/thoth/releases/tag/v1.0.5) - 2026-04-15
### Added
  - Add `workStatuses` filtering to `subjects` and `subjectCount` queries

## [[1.0.4]](https://github.com/thoth-pub/thoth/releases/tag/v1.0.4) - 2026-04-13
### Fixed
  - Aggregate Crossref Crossmark updates into a single `<updates>` block

## [[1.0.3]](https://github.com/thoth-pub/thoth/releases/tag/v1.0.3) - 2026-04-07
### Fixed
  - [741](https://github.com/thoth-pub/thoth/pull/741) - Harden JATS rich-text handling by rejecting malformed or nested markup and abstract line breaks on write, and normalise Crossref abstract output to avoid invalid nested `jats:p` and `jats:break` elements

## [[1.0.2]](https://github.com/thoth-pub/thoth/releases/tag/v1.0.2) - 2026-04-03
### Security
  - Upgrade `aws-lc-sys` to v0.39.1
  - Upgrade `jsonwebtoken` to v10.3.0
  - Upgrade `rustls-webpki` to v0.103.10

### Fixed
  - Allow publisher admins to update metadata without storage-field superuser check

## [[1.0.1]](https://github.com/thoth-pub/thoth/releases/tag/v1.0.1) - 2026-04-01
### Fixed
  - Return raw JATS XML rather than attempting to validate it

## [[1.0.0]](https://github.com/thoth-pub/thoth/releases/tag/v1.0.0) - 2026-04-01
### Changed
  - [736](https://github.com/thoth-pub/thoth/pull/736) - Remove `Funding.jurisdiction` and `Language.mainLanguage`, and add `Issue.issueNumber`
  - [732](https://github.com/thoth-pub/thoth/pull/732) - Migrated GraphQL API authentication to OIDC via Zitadel. Internal JWT handling has been replaced with introspection of Zitadel-issued tokens. Authorisation is now based entirely on token claims, removing the need for the internal `account` and `publisher_account` tables.
  - [689](https://github.com/thoth-pub/thoth/issues/689) - Move `Work.fullTitle`, `Work.title` and `Work.subtitle` into a dedicated `Title` table, supporting multilingual and rich text fields
  - [689](https://github.com/thoth-pub/thoth/issues/689) - Move `Work.shortAbstract` and `Work.longAbstract` into a dedicated `Abstract` table with `abstractType`, supporting multilingual and rich text fields
  - [689](https://github.com/thoth-pub/thoth/issues/689) - Move `Contribution.biography` into a dedicated `Biography` table, supporting multilingual and rich text fields
  - [689](https://github.com/thoth-pub/thoth/issues/689) - Store all rich text fields internally as JATS XML, supporting conversion to/from HTML, Markdown, and plain text
  - [689](https://github.com/thoth-pub/thoth/issues/689) - Mark existing GraphQL fields as deprecated and return only the canonical version
  - [701](https://github.com/thoth-pub/thoth/issues/701) - Add accessibility-related metadata to Thoth data model and outputs
  - [682](https://github.com/thoth-pub/thoth/issues/682) - Improve ONIX 3.0 and 3.1 outputs based on feedback from EDItEUR

### Added
  - [711](https://github.com/thoth-pub/thoth/pull/711) - Allow filtering work queries by publication date
  - [715](https://github.com/thoth-pub/thoth/pull/715) - Support reordering items which have ordinals
  - [713](https://github.com/thoth-pub/thoth/issues/713) - Add a secure and scalable file upload architecture for books and chapters.
  - Added new work-linked domain entities: `additional_resource`, `award`, `endorsement`, `book_review`, `work_featured_video`

### Fixed
  - [712](https://github.com/thoth-pub/thoth/pull/712) - Make `updated_at_with_relations` propagation less deadlock-prone

### Removed
  - [710](https://github.com/thoth-pub/thoth/pull/710) - Deprecated thoth-app


## [[0.13.16]](https://github.com/thoth-pub/thoth/releases/tag/v0.13.16) - 2026-03-06
### Changed
  - [#731](https://github.com/thoth-pub/thoth/pull/731) - Ignore hyphens when filtering publications on ISBN

## [[0.13.15]](https://github.com/thoth-pub/thoth/releases/tag/v0.13.15) - 2025-12-03
### Changed
- [#717](https://github.com/thoth-pub/thoth/pull/717) - Update Thema codes to v1.6

## [[0.13.14]](https://github.com/thoth-pub/thoth/releases/tag/v0.13.14) - 2025-10-14
### Changed
  - [708](https://github.com/thoth-pub/thoth/pull/708) - Replace ISBN parsing library with [`isbn`](https://crates.io/crates/isbn)

## [[0.13.13]](https://github.com/thoth-pub/thoth/releases/tag/v0.13.13) - 2025-06-05
### Changed
  - [691](https://github.com/thoth-pub/thoth/issues/691) - Require a license for full KBART output, fall back to work\_id for KBART title ID if no DOI available

## [[0.13.12]](https://github.com/thoth-pub/thoth/releases/tag/v0.13.12) - 2025-05-28
### Changed
  - Add security policies to APP headers

## [[0.13.11]](https://github.com/thoth-pub/thoth/releases/tag/v0.13.11) - 2025-05-14
### Changed
  - [687](https://github.com/thoth-pub/thoth/issues/687) - Upgrade database engine to PostgreSQL v17
  - [684](https://github.com/thoth-pub/thoth/issues/684) - Refactor internal work and publication APIs
  - [687](https://github.com/thoth-pub/thoth/issues/687) - Use test subdomains when building staging docker image
  - [685](https://github.com/thoth-pub/thoth/issues/685) - Update Thoth logo in README files

## [[0.13.10]](https://github.com/thoth-pub/thoth/releases/tag/v0.13.10) - 2025-04-24
### Changed
  - [634](https://github.com/thoth-pub/thoth/issues/634) - Prevent non-superusers from unpublishing a Work once it is published. Once a `Work`'s `WorkStatus` is set to `Active`, `Withdrawn`, or `Superseded`, it cannot be set to `Forthcoming`, `Cancelled` or `PostponedIndefinitely`.
  - [659](https://github.com/thoth-pub/thoth/issues/659) - Prevent non-superusers from deleting a `Work` that is published (i.e. `WorkStatus` is `Active`, `Withdrawn`, or `Superseded`.)

## [[0.13.9]](https://github.com/thoth-pub/thoth/releases/tag/v0.13.9) - 2025-04-10
### Fixed
  - [679](https://github.com/thoth-pub/thoth/issues/679) - Remove extraneous <custom_metadata> tag from Crossmark block in Crossref DOI deposit export when an Imprint has recorded a Crossmark DOI but a Work has no license or funding metadata.

### Changed
  - [683](https://github.com/thoth-pub/thoth/pull/683) - Upgrade rust to `1.86.0` in production `Dockerfile`

### Security
  - [683](https://github.com/thoth-pub/thoth/pull/683) - Upgrade `tokio` to v1.44.2
  - [683](https://github.com/thoth-pub/thoth/pull/683) - Upgrade `openssl` to v0.10.72

## [[0.13.8]](https://github.com/thoth-pub/thoth/releases/tag/v0.13.8) - 2025-03-26
### Added
  - [577](https://github.com/thoth-pub/thoth/pull/577) - Implement ONIX 3.1 "Thoth" specification (i.e. complete record reflecting all updates up to ONIX 3.1.2 that can be implemented based on existing data model)

### Fixed
  - [677](https://github.com/thoth-pub/thoth/pull/677) - Remove module directive in rapidocs import, which was cauisng CORS errors

## [[0.13.7]](https://github.com/thoth-pub/thoth/releases/tag/v0.13.7) - 2025-03-18
### Changed
  - [676](https://github.com/thoth-pub/thoth/issues/676) - Upgrade rust to `1.85.0` in production `Dockerfile`
  - [676](https://github.com/thoth-pub/thoth/issues/676) - Upgrade `actix-cors` to v0.7.1
  - [676](https://github.com/thoth-pub/thoth/issues/676) - Upgrade `actix-http` to v3.10.0
  - [676](https://github.com/thoth-pub/thoth/issues/676) - Upgrade `actix-web` to v4.10.2
  - [676](https://github.com/thoth-pub/thoth/issues/676) - Upgrade `chrono` to v0.4.40
  - [676](https://github.com/thoth-pub/thoth/issues/676) - Upgrade `csv` to v1.3.1
  - [676](https://github.com/thoth-pub/thoth/issues/676) - Upgrade `deadpool-redis` to v0.20.0
  - [676](https://github.com/thoth-pub/thoth/issues/676) - Upgrade `diesel` to v2.2.8
  - [676](https://github.com/thoth-pub/thoth/issues/676) - Upgrade `env_logger` to v0.11.7
  - [676](https://github.com/thoth-pub/thoth/issues/676) - Upgrade `jsonwebtoken` to v9.3.1
  - [676](https://github.com/thoth-pub/thoth/issues/676) - Upgrade `rand` to v0.9.0
  - [676](https://github.com/thoth-pub/thoth/issues/676) - Upgrade `regex` to v1.11.1
  - [676](https://github.com/thoth-pub/thoth/issues/676) - Upgrade `reqwest-middleware` to v0.4.1
  - [676](https://github.com/thoth-pub/thoth/issues/676) - Upgrade `reqwest-retry` to v0.7.0
  - [676](https://github.com/thoth-pub/thoth/issues/676) - Upgrade `semver` to v1.0.26
  - [676](https://github.com/thoth-pub/thoth/issues/676) - Upgrade `strum` to v0.27.1
  - [676](https://github.com/thoth-pub/thoth/issues/676) - Upgrade `tokio` to v1.44.1
  - [676](https://github.com/thoth-pub/thoth/issues/676) - Upgrade `trunk` to v0.21.9
  - [676](https://github.com/thoth-pub/thoth/issues/676) - Upgrade `uuid` to v1.16.0
  - [676](https://github.com/thoth-pub/thoth/issues/676) - Upgrade `wasm-bindgen` to v0.2.100
  - [676](https://github.com/thoth-pub/thoth/issues/676) - Upgrade `web-sys` to v0.3.77
  - [676](https://github.com/thoth-pub/thoth/issues/676) - Upgrade `xml-rs` to v0.8.25

## [[0.13.6]](https://github.com/thoth-pub/thoth/releases/tag/v0.13.6) - 2025-01-28
### Changed
  - [667](https://github.com/thoth-pub/thoth/issues/667) - Refactor binary using new submodules `commands` and `arguments`
  - [667](https://github.com/thoth-pub/thoth/issues/667) - Trigger `run\_migrations` github action when binary source changes

### Added
  - [667](https://github.com/thoth-pub/thoth/issues/667) - CLI subcommand `thoth account publishers` to modify which publisher(s) an account has access to

## [[0.13.5]](https://github.com/thoth-pub/thoth/releases/tag/v0.13.5) - 2025-01-17
### Changed
  - [665](https://github.com/thoth-pub/thoth/issues/665) - Removed unnecessary `map_or()` to comply with [`rustc 1.84.0`](https://github.com/rust-lang/rust/releases/tag/1.84.0)
  - [666](https://github.com/thoth-pub/thoth/issues/666) - Upgrade rust to `1.84.0` in production `Dockerfile`

### Added
  - [666](https://github.com/thoth-pub/thoth/issues/666) - CLI subcommand `thoth cache delete` to delete cached metadata records by specification ID

## [[0.13.4]](https://github.com/thoth-pub/thoth/releases/tag/v0.13.4) - 2024-12-11
### Added
  - [661](https://github.com/thoth-pub/thoth/issues/661) - Implement caching errors in export API

## [[0.13.3]](https://github.com/thoth-pub/thoth/releases/tag/v0.13.3) - 2024-12-02
### Changed
  - [660](https://github.com/thoth-pub/thoth/issues/660) - Upgrade rust to `1.83.0` in production `Dockerfile`
  - [660](https://github.com/thoth-pub/thoth/issues/660) - Use latest tag in development `Dockerfile`
  - [658](https://github.com/thoth-pub/thoth/issues/658) - Upgrade `rustls` to v0.23.19
  - [658](https://github.com/thoth-pub/thoth/issues/658) - Upgrade `chrono` to v0.4.38
  - [658](https://github.com/thoth-pub/thoth/issues/658) - Upgrade `uuid` to v0.11.0
  - [658](https://github.com/thoth-pub/thoth/issues/658) - Upgrade `diesel` to v2.2.5
  - [658](https://github.com/thoth-pub/thoth/issues/658) - Upgrade `paperclip` to v0.9.4
  - [658](https://github.com/thoth-pub/thoth/issues/658) - Upgrade `regex` to v1.11.1
  - [658](https://github.com/thoth-pub/thoth/issues/658) - Upgrade `xml-rs` to v0.8.23
  - [658](https://github.com/thoth-pub/thoth/issues/658) - Upgrade `log` to v0.4.22
  - [658](https://github.com/thoth-pub/thoth/issues/658) - Upgrade `thiserror` to v2.0.3
  - [658](https://github.com/thoth-pub/thoth/issues/658) - Upgrade `reqwest-middleware` to v0.4.0
  - [658](https://github.com/thoth-pub/thoth/issues/658) - Upgrade `reqwest-retry` to v0.7.0
  - [658](https://github.com/thoth-pub/thoth/issues/658) - Upgrade `trunk` to v0.21.4
  - [658](https://github.com/thoth-pub/thoth/issues/658) - Upgrade `actix-identity` to v0.8.0
  - [658](https://github.com/thoth-pub/thoth/issues/658) - Upgrade `actix-session` to v0.10.1

## Removed
  - Remove redundant dependencies in thoth-app: `anyhow`, `log`, `url`

## [[0.13.2]](https://github.com/thoth-pub/thoth/releases/tag/v0.13.2) - 2024-11-26
### Added
  - [656](https://github.com/thoth-pub/thoth/issues/656) - Add database indexes to common attributes to improve performance

## [[0.13.1]](https://github.com/thoth-pub/thoth/releases/tag/v0.13.1) - 2024-11-25
### Added
  - [593](https://github.com/thoth-pub/thoth/issues/593) - Log GraphQL queries alongside request logs

## [[0.13.0]](https://github.com/thoth-pub/thoth/releases/tag/v0.13.0) - 2024-11-19
### Added
  - [651](https://github.com/thoth-pub/thoth/issues/651) - Implement Redis connection pools using `deadpool-redis`
  - [651](https://github.com/thoth-pub/thoth/issues/651) - Implement Redis caching in export API
  - [651](https://github.com/thoth-pub/thoth/issues/651) - Added `WorkLastUpdatedQuery` and `WorksLastUpdatedQuery` queries to thoth-client
  - [651](https://github.com/thoth-pub/thoth/issues/651) - Allow supplying `DATABASE_URL` as binary argument
  - [648](https://github.com/thoth-pub/thoth/issues/648) - Added new `LocationPlatform`, `THOTH`, for Locations where file is hosted directly by Thoth on S3.

### Changed
  - [650](https://github.com/thoth-pub/thoth/issues/650) - Allow only superusers to create/update/delete a `Location` when the `LocationPlatform` is `THOTH`.
  - [651](https://github.com/thoth-pub/thoth/issues/651) - Use Github Container registry instead of DockerHub

### Fixed
  - [631](https://github.com/thoth-pub/thoth/issues/631) - Fix slow loading of Contributor dropdown in Contribution form

## [[0.12.14]](https://github.com/thoth-pub/thoth/releases/tag/v0.12.14) - 2024-11-04
### Changed
  - [642](https://github.com/thoth-pub/thoth/issues/642) - Output `ProductAvailability` based on work status in Thoth ONIX 3.0
  - [642](https://github.com/thoth-pub/thoth/issues/642) - Use `UnpricedItemType` code `01` (Free of charge) for unpriced products in Thoth ONIX 3.0

## [[0.12.13]](https://github.com/thoth-pub/thoth/releases/tag/v0.12.13) - 2024-10-23
### Fixed
  - [639](https://github.com/thoth-pub/thoth/issues/639) - Make new locations canonical by default

### Changed
  - [628](https://github.com/thoth-pub/thoth/issues/628) - Upgrade rust to `1.82.0` in production and development `Dockerfile`

## [[0.12.12]](https://github.com/thoth-pub/thoth/releases/tag/v0.12.12) - 2024-10-15
### Fixed
  - [636](https://github.com/thoth-pub/thoth/issues/636) - OpenAPI documentation was displaying the public URL of the export API with an extra protocol

## [[0.12.11]](https://github.com/thoth-pub/thoth/releases/tag/v0.12.11) - 2024-10-14
### Changed
  - [324](https://github.com/thoth-pub/thoth/issues/324) - Make Locations editable, including the ability to change the Canonical Location for a Publication
  - [635](https://github.com/thoth-pub/thoth/issues/635) - Upgrade `reqwest` to v0.12.8
  - [635](https://github.com/thoth-pub/thoth/issues/635) - Upgrade `reqwest-middleware` to v0.3.3
  - [635](https://github.com/thoth-pub/thoth/issues/635) - Upgrade `reqwest-retry` to v0.6.1
  - [635](https://github.com/thoth-pub/thoth/issues/635) - Upgrade `paperclip` to v0.9.2

## [[0.12.10]](https://github.com/thoth-pub/thoth/releases/tag/v0.12.10) - 2024-10-01
### Added
  - [628](https://github.com/thoth-pub/thoth/issues/628) - Implement OpenAPI v3 schema in export API, served under `/openapi.json`
  - [628](https://github.com/thoth-pub/thoth/issues/628) - Added terms of service to export API

### Changed
  - [551](https://github.com/thoth-pub/thoth/issues/551) - Only include chapters in Crossref metadata output if they have DOIs
  - [628](https://github.com/thoth-pub/thoth/issues/628) - Upgrade `paperclip` to v0.9.1
  - [628](https://github.com/thoth-pub/thoth/issues/628) - Upgrade rust to `1.81.0` in production and development `Dockerfile`
  - [544](https://github.com/thoth-pub/thoth/issues/544) - Implement non-OA metadata in export outputs

### Fixed
  - [565](https://github.com/thoth-pub/thoth/issues/565) - Don't generate Crossref metadata output if no DOIs (work or chapter) are present
  - [632](https://github.com/thoth-pub/thoth/issues/632) - Add second order by clause (work\_id) to work queries for consistent ordering when multiple works share the same user-ordered field, such as publication date

## [[0.12.9]](https://github.com/thoth-pub/thoth/releases/tag/v0.12.9) - 2024-09-06
### Added
  - [595](https://github.com/thoth-pub/thoth/issues/595), [626](https://github.com/thoth-pub/thoth/pull/626) - Remove infrequently used and unused work statuses (unspecified, no longer our product, out of stock indefinitely, out of print, inactive, unknown, remaindered, recalled). Require a publication date for active, withdrawn, and superseded works in Thoth. Add a new `Superseded` work status to replace Out of Print for older editions of Works. Require a withdrawn date for Superseded works.
  - [582](https://github.com/thoth-pub/thoth/issues/582) - Add Crossmark metadata in Crossref DOI deposit when a Crossmark policy is present in the publisher record. Add Crossmark update new\_edition metadata when a book is replaced by a new edition, and withdrawal metadata when a book is withdrawn from sale.
  - [574](https://github.com/thoth-pub/thoth/issues/574), [626](https://github.com/thoth-pub/thoth/pull/626) - Add descriptions to all remaining items in schema

### Fixed
  - [548](https://github.com/thoth-pub/thoth/issues/548) - Prevent users from deleting contributors/institutions which are linked to works by other publishers

### Changed
  - [623](https://github.com/thoth-pub/thoth/issues/623) - Convert connection pool errors (`r2d2::Error`) to `ThothError`
  - [625](https://github.com/thoth-pub/thoth/issues/625) - Use relationcode 13 for physical ISBNs in ONIX 2.1 EBSCOHost output

## [[0.12.8]](https://github.com/thoth-pub/thoth/releases/tag/v0.12.8) - 2024-09-03
### Fixed
  - [622](https://github.com/thoth-pub/thoth/issues/622) - Fix bug where list of contributors in New/Edit Contribution form was truncated

## [[0.12.7]](https://github.com/thoth-pub/thoth/releases/tag/v0.12.7) - 2024-08-28
### Changed
  - [538](https://github.com/thoth-pub/thoth/issues/538) - Update Project MUSE ONIX 3.0 export to reflect new specifications provided by Project MUSE.
  - [616](https://github.com/thoth-pub/thoth/issues/616) - Removed unused constant to comply with [`rustc 1.80.0`](https://github.com/rust-lang/rust/releases/tag/1.80.0)
  - [616](https://github.com/thoth-pub/thoth/issues/616), [621](https://github.com/thoth-pub/thoth/issues/621) - Upgrade `actix-web` to v4.9
  - [616](https://github.com/thoth-pub/thoth/issues/616) - Upgrade `time` to v0.3.36
  - [586](https://github.com/thoth-pub/thoth/issues/586) - Upgrade `juniper` to v0.16.1
  - [586](https://github.com/thoth-pub/thoth/issues/586) - Upgrade `uuid` to v1.10.0
  - [586](https://github.com/thoth-pub/thoth/issues/586) - Upgrade `graphql_client` to v0.14.0
  - [586](https://github.com/thoth-pub/thoth/issues/586) - Upgrade `chrono` to v0.4.38
  - [586](https://github.com/thoth-pub/thoth/issues/586) - Upgrade `trunk` to v0.20.3
  - [586](https://github.com/thoth-pub/thoth/issues/586) - Upgrade `wasm-bindgen` to v0.2.93
  - [586](https://github.com/thoth-pub/thoth/issues/586) - Upgrade rust to `1.80.1` in production and development `Dockerfile`
  - [621](https://github.com/thoth-pub/thoth/issues/621) - Upgrade `clap` to v4.5.16
  - [621](https://github.com/thoth-pub/thoth/issues/621) - Upgrade `env_logger` to v0.11.5
  - [621](https://github.com/thoth-pub/thoth/issues/621) - Upgrade `futures` to v0.3.30
  - [621](https://github.com/thoth-pub/thoth/issues/621) - Upgrade `regex` to v1.10.6
  - [621](https://github.com/thoth-pub/thoth/issues/621) - Upgrade `strum` to v0.26.3
  - [621](https://github.com/thoth-pub/thoth/issues/621) - Upgrade `anyhow` to v1.0.86
  - [621](https://github.com/thoth-pub/thoth/issues/621) - Upgrade `thiserror` to v1.0.63
  - [621](https://github.com/thoth-pub/thoth/issues/621) - Upgrade `semver` to v1.0.23
  - [621](https://github.com/thoth-pub/thoth/issues/621) - Replace deprecated diesel macro `sql_function` with `define_sql_function`
  - [617](https://github.com/thoth-pub/thoth/issues/617) - Update publication types to include audiobook formats (MP3 and WAV)

### Fixed
  - [610](https://github.com/thoth-pub/thoth/issues/610) - Update <WebsiteRole> code for Work Landing Page in all ONIX exports from "01" (Publisher’s corporate website) to "02" (Publisher’s website for a specified work).

### Security
  - [621](https://github.com/thoth-pub/thoth/issues/621) - Upgrade `diesel` to v2.2.3
  - [621](https://github.com/thoth-pub/thoth/issues/621) - Upgrade `diesel-derive-newtype` to v2.1.2
  - [621](https://github.com/thoth-pub/thoth/issues/621) - Upgrade `diesel_migrations` to v2.2.0

## [[0.12.6]](https://github.com/thoth-pub/thoth/releases/tag/v0.12.6) - 2024-06-17
### Fixed
  - [#513](https://github.com/thoth-pub/thoth/issues/513) - Expand DOI regex to include `+`, `[`, and `]`

### Changed
  - [607](https://github.com/thoth-pub/thoth/issues/607) - Upgrade rust to `1.79.0` in production and development `Dockerfile`

### Added
  - [607](https://github.com/thoth-pub/thoth/issues/607) - Add caching steps to Github actions

## [[0.12.5]](https://github.com/thoth-pub/thoth/releases/tag/v0.12.5) - 2024-05-07
### Changed
  - [601](https://github.com/thoth-pub/thoth/issues/601) - Upgrade rust to `1.78.0` in production and development `Dockerfile`
  - [601](https://github.com/thoth-pub/thoth/issues/601) - Upgrade `trunk` to v0.20.0
  - [601](https://github.com/thoth-pub/thoth/issues/601) - Added `-vv` option to build command in Makefile and GitHub actions

## [[0.12.4]](https://github.com/thoth-pub/thoth/releases/tag/v0.12.4) - 2024-04-30
### Changed
  - [545](https://github.com/thoth-pub/thoth/issues/545) - Add Zenodo as a location platform

## [[0.12.3]](https://github.com/thoth-pub/thoth/releases/tag/v0.12.3) - 2024-04-26
### Added
  - [583](https://github.com/thoth-pub/thoth/issues/583) - Add new field, Permanently Withdrawn Date, to Work for Out-of-print or Withdrawn from Sale Works.

### Fixed
  - [597](https://github.com/thoth-pub/thoth/issues/597) - Graphiql not working in chrome and safari

### Changed
 - [218](https://github.com/thoth-pub/thoth/issues/218) - Make series ISSN optional

## [[0.12.2]](https://github.com/thoth-pub/thoth/releases/tag/v0.12.2) - 2024-04-16
### Added
  - [581](https://github.com/thoth-pub/thoth/issues/581) - Add crossmark policy DOI to imprint record

### Changed
  - [591](https://github.com/thoth-pub/thoth/issues/591) - Upgrade rust to `1.77.2` in production and development `Dockerfile`
  - [591](https://github.com/thoth-pub/thoth/issues/591) - Added favicons to export API and GraphQL API docs
  - [591](https://github.com/thoth-pub/thoth/issues/591) - Replaced static logo files with CDN paths
  - [591](https://github.com/thoth-pub/thoth/issues/591) - Moved thoth CSS to root directory in thoth-app
  - [591](https://github.com/thoth-pub/thoth/issues/591) - Replace unnecessary pageloader CSS with an actual loader
  - [591](https://github.com/thoth-pub/thoth/issues/591) - Apply Thoth theming to rapidocs
  - [591](https://github.com/thoth-pub/thoth/issues/591) - Upgrade `graphiql` to v3.2
  - [591](https://github.com/thoth-pub/thoth/issues/591) - Upgrade `trunk` to v0.19.2
  - [591](https://github.com/thoth-pub/thoth/issues/591) - Upgrade `wasm-bindgen` to v0.2.92

### Fixed
  - [591](https://github.com/thoth-pub/thoth/issues/591) - Replaced broken logo URL in export API docs

## [[0.12.1]](https://github.com/thoth-pub/thoth/releases/tag/v0.12.1) - 2024-04-8
### Fixed
  - [589](https://github.com/thoth-pub/thoth/issues/589) - Truncation of `short_abstract` in Thoth ONIX results in Invalid UTF-8 sequences

## [[0.12.0]](https://github.com/thoth-pub/thoth/releases/tag/v0.12.0) - 2024-03-14
### Removed
  - [549](https://github.com/thoth-pub/thoth/issues/549) - Deprecate public-facing pages in Thoth APP in favour of a separate, standalone, website

### Added
  - [549](https://github.com/thoth-pub/thoth/issues/549) - Build and push staging docker images on pull requests

### Changed
 - [549](https://github.com/thoth-pub/thoth/issues/549) - Upgrade GitHub actions dependencies (`docker/setup-qemu-action@v3`, `docker/setup-buildx-action@v3`, `docker/login-action@v3`, `docker/build-push-action@v5`, `actions/checkout@v4`, `actions/setup-node@v4`)

## [[0.11.18]](https://github.com/thoth-pub/thoth/releases/tag/v0.11.18) - 2024-03-07
### Added
  - [441](https://github.com/thoth-pub/thoth/issues/441) - Implement ONIX 3.0 "Thoth" specification (i.e. complete record reflecting full data model)
  - [401](https://github.com/thoth-pub/thoth/issues/401) - Add BDS Live to list of supported platforms for JSTOR ONIX output

### Fixed
  - [475](https://github.com/thoth-pub/thoth/issues/475) - Add seconds to timestamp for Crossref metadata output
  - [571](https://github.com/thoth-pub/thoth/issues/571) - Fix overlapping URL text for Locations in Thoth Admin panel on website in Safari and Chromium browsers

### Changed
 - [578](https://github.com/thoth-pub/thoth/issues/578) - Upgrade `actix-identity` to v0.7.1
 - [578](https://github.com/thoth-pub/thoth/issues/578) - Upgrade `actix-session` to v0.9.0

### Security
  - [572](https://github.com/thoth-pub/thoth/issues/572) - Upgrade `mio` to v0.8.11

## [[0.11.17]](https://github.com/thoth-pub/thoth/releases/tag/v0.11.17) - 2024-02-29
### Changed
  - [568](https://github.com/thoth-pub/thoth/issues/568) - Allow building `thoth-app` directly from cargo, using a build script in `thoth-app-server`
  - [569](https://github.com/thoth-pub/thoth/issues/569) - Build `thoth-app` with `trunk, instead of `wasm-pack`
  - [569](https://github.com/thoth-pub/thoth/issues/569) - Optionally load `thoth-export-server` env variables from `.env` at build time
  - [569](https://github.com/thoth-pub/thoth/issues/569) - Optionally load `thoth-app` env variables from `.env` at build time
  - [569](https://github.com/thoth-pub/thoth/issues/569) - Upgrade `jsonwebtoken` to v9.2.0
  - [569](https://github.com/thoth-pub/thoth/issues/569) - Upgrade build dependencies (npm `v10.2.5`, node `v20.10.0` and rollup `v4.9.1`) in production and development `Dockerfile`

### Fixed
  - [564](https://github.com/thoth-pub/thoth/issues/564) - Fix error in BibTeX not outputting editors in work types other than edited volume
  - [447](https://github.com/thoth-pub/thoth/issues/447) - Prevents Google Books Onix3 format output from Export API if Thoth record doesn't contain at least one BIC, BISAC or LCC subject code
  - [404](https://github.com/thoth-pub/thoth/issues/404) - Prevents JSTOR Onix3 format output from Export API if Thoth record doesn't contain at least one BISAC subject code

### Security
  - [569](https://github.com/thoth-pub/thoth/issues/569) - Upgrade `actix-web` to v4.5.1
  - [569](https://github.com/thoth-pub/thoth/issues/569) - Upgrade `tempfile` to v3.10.1
  - [569](https://github.com/thoth-pub/thoth/issues/569) - Upgrade `openssl` to v0.10.64
  - [569](https://github.com/thoth-pub/thoth/issues/569) - Upgrade `serde_yaml` to v0.9.25

## [[0.11.16]](https://github.com/thoth-pub/thoth/releases/tag/v0.11.16) - 2024-02-19
### Changed
  - [561](https://github.com/thoth-pub/thoth/issues/561) - Add "Publisher Website" as a location platform
  - [553](https://github.com/thoth-pub/thoth/issues/553) - Upgrade rust to `1.76.0` in production and development `Dockerfile`
  - [305](https://github.com/thoth-pub/thoth/issues/305) - Update rust edition to 2021
  - [555](https://github.com/thoth-pub/thoth/issues/555) - Remove thoth-client's schema.json with auto-generated GraphQL schema language file on compilation

### Added
  - [244](https://github.com/thoth-pub/thoth/issues/244) - Expose GraphQL schema file in /schema.graphql
  - [503](https://github.com/thoth-pub/thoth/issues/503) - Allow reverting migrations in the CLI and check that migrations can be reverted in run-migration github action
  - [557](https://github.com/thoth-pub/thoth/issues/557) - Added github action to chech that the changelog has been updated on PRs

## [[0.11.15]](https://github.com/thoth-pub/thoth/releases/tag/v0.11.15) - 2024-01-18
### Changed
  - [536](https://github.com/thoth-pub/thoth/issues/536) - Rename "SciELO" location platform to "SciELO Books"

## [[0.11.14]](https://github.com/thoth-pub/thoth/releases/tag/v0.11.14) - 2024-01-18
### Changed
  - [#467](https://github.com/thoth-pub/thoth/issues/467), [#403](https://github.com/thoth-pub/thoth/issues/403), [#536](https://github.com/thoth-pub/thoth/issues/536) - Expand the list of location platforms with: GoogleBooks, InternetArchive, ScienceOpen, and Scielo
  - [526](https://github.com/thoth-pub/thoth/issues/526) - Added Brendan to About page

## [[0.11.13]](https://github.com/thoth-pub/thoth/releases/tag/v0.11.13) - 2024-01-08
### Changed
  - Upgrade rust to `1.75.0` in production and development `Dockerfile`
  - Upgrade `juniper` to v0.15.12
  - Upgrade `actix-web` to v4.4.1
  - Upgrade `actix-cors` to v0.7.0
  - Increase size of URL columns in locations component

### Fixed
  - [531](https://github.com/thoth-pub/thoth/issues/531) - Fix bug where New Publication form for Chapter could have an ISBN pre-populated but greyed out

## [[0.11.12]](https://github.com/thoth-pub/thoth/releases/tag/v0.11.12) - 2023-12-20
### Fixed
  - [530](https://github.com/thoth-pub/thoth/issues/530) - Fix pagination offset calculation in export API
  - [530](https://github.com/thoth-pub/thoth/issues/530) - Do not allow to create more than one price in the same currency for the same publication

## [[0.11.11]](https://github.com/thoth-pub/thoth/releases/tag/v0.11.11) - 2023-12-19
### Changed
  - Upgrade rust to `1.74.1` in production and development `Dockerfile`
  - Upgrade build dependencies (npm `v10.2.5`, node `v20.10.0` and rollup `v4.9.1`) in production and development `Dockerfile`

## [[0.11.10]](https://github.com/thoth-pub/thoth/releases/tag/v0.11.10) - 2023-11-27
### Fixed
  - [524](https://github.com/thoth-pub/thoth/issues/524) - Bibliography note not being retrieved on work page

## [[0.11.9]](https://github.com/thoth-pub/thoth/releases/tag/v0.11.9) - 2023-11-22
### Changed
  - Upgrade rust to `1.74.0` in production and development `Dockerfile`
  - Upgrade `xml-rs` to v0.8.19
  - Upgrade `clap` to v4.4.7
  - Upgrade `dialoguer` to v0.11.0
  - Upgrade `futures` to v0.3.29
  - Upgrade `regex` to v1.10.2
  - Upgrade `diesel` to v2.1.3
  - Upgrade `csv` to v1.3.0
  - Upgrade `reqwest-middleware` to v0.2.4
  - Upgrade `reqwest-retry` to v0.2.3
  - [522](https://github.com/thoth-pub/thoth/issues/522) - Improve MARC records with further recommendations

## [[0.11.8]](https://github.com/thoth-pub/thoth/releases/tag/v0.11.8) - 2023-10-31
### Changed
  - Upgrade rust to `1.73.0` in development `Dockerfile`
  - Upgrade build dependencies (npm `v10.2.0`, node `v18.18.2`, n `v9.2.0` and rollup `v4.1.4`) in production and development `Dockerfile`
  - [519](https://github.com/thoth-pub/thoth/issues/519) - Update ProQuest Ebrary (Ebook Central) ONIX output pricing

## [[0.11.7]](https://github.com/thoth-pub/thoth/releases/tag/v0.11.7) - 2023-10-02
### Changed
  - [508](https://github.com/thoth-pub/thoth/issues/508) - Improve MARC records with recommendations
  - Upgrade `actix-identity` to v0.6.0
  - Upgrade `actix-session` to v0.8.0
  - Upgrade `chrono` to v0.4.31
  - Upgrade `marc` to v3.1.1

### Fixed
  - [#513](https://github.com/thoth-pub/thoth/issues/513) - Expand DOI regex to include angle brackets

## [[0.11.6]](https://github.com/thoth-pub/thoth/releases/tag/v0.11.6) - 2023-09-08
### Security
  - Upgrade `chrono` to v0.4.30

## [[0.11.5]](https://github.com/thoth-pub/thoth/releases/tag/v0.11.5) - 2023-09-05
### Security
  - Upgrade `actix-web` to v4.4.0
  - Upgrade `tempfile` to v3.8.0

### Changed
  - Upgrade `diesel` to v2.1.1
  - Upgrade `diesel-derive-enum` to v2.1.0
  - Upgrade `diesel-derive-newtype` to v2.1.0
  - Upgrade `diesel_migrations` to v2.1.0
  - Upgrade `rand` to v0.8.5
  - Upgrade `juniper` to v0.15.11
  - Upgrade `strum` to v0.25.0
  - Upgrade `paperclip` to v0.8.1
  - Upgrade `graphql_client` to v0.13.0
  - Upgrade `reqwest-middleware` to v0.2.3
  - Upgrade `reqwest-retry` to v0.2.3
  - Upgrade `actix-identity` to v0.5.2 and added `actix-session` v0.7.2
  - Upgrade `dialoguer` to v0.10.4
  - Upgrade `futures` to v0.3.28
  - Upgrade `regex` to v1.9.5
  - Upgrade `jsonwebtoken` to v8.3.0
  - Upgrade `csv` to v1.2.2
  - Upgrade `xml-rs` to v0.8.17
  - Upgrade `log` to v0.4.20
  - Upgrade `clap` to v4.4.2
  - Short version of host command is now `-H` instead of `-h` in CLI

## [[0.11.4]](https://github.com/thoth-pub/thoth/releases/tag/v0.11.4) - 2023-08-28
### Security
  - Upgrade `rustls-webpki` to v0.100.2

## [[0.11.3]](https://github.com/thoth-pub/thoth/releases/tag/v0.11.3) - 2023-08-28
### Fixed
  - [500](https://github.com/thoth-pub/thoth/issues/500) - Update ORCID regex

### Security
  - Upgrade `openssl` to v0.10.56
  - Upgrade `reqwest` to v0.11.20
  - Upgrade `chrono` to v0.4.26

### Changed
  - Upgrade rust to `1.72.0` in production and development `Dockerfile`
  - Upgrade build dependencies (npm `v9.8.1`, node `v18.17.1`, n `v9.1.0`, and rollup `v3.28.1`) in production and development `Dockerfile`
  - Upgrade `wasm-pack` to [v0.12.1](https://github.com/rustwasm/wasm-pack/releases/tag/v0.12.1)

### Added
  - Link to privacy policy in navbar

## [[0.11.2]](https://github.com/thoth-pub/thoth/releases/tag/v0.11.2) - 2023-06-19
### Changed
  - Upgrade `wasm-pack` to [v0.12.0](https://github.com/rustwasm/wasm-pack/releases/tag/v0.12.0)
  - Upgrade `clap` to v2.34.0

## [[0.11.1]](https://github.com/thoth-pub/thoth/releases/tag/v0.11.1) - 2023-06-15
### Added
  - Add CC0 license to MARC records

### Changed
  - Upgrade rust to `1.70.0` in production and development `Dockerfile`
  - Upgrade build dependencies (npm `v9.6.7`, node `v18.16.0` and rollup `v3.23.1`) in production and development `Dockerfile`
  - Upgrade `wasm-pack` to v0.11.1
  - Replace `marc` fork with actual crate
  - Update about page

## [[0.11.0]](https://github.com/thoth-pub/thoth/releases/tag/v0.11.0) - 2023-04-14
### Added
  - [490](https://github.com/thoth-pub/thoth/issues/490) - Generate MARC 21 markup
  - [491](https://github.com/thoth-pub/thoth/issues/491) - Generate MARC 21 XML
  - [492](https://github.com/thoth-pub/thoth/issues/492) - Add Thoth's MARC organization code to MARC records
  - [492](https://github.com/thoth-pub/thoth/issues/492) - Add ORCID IDs to MARC
  - [492](https://github.com/thoth-pub/thoth/issues/492) - Add contact details to APP

### Changed
  - [492](https://github.com/thoth-pub/thoth/issues/492) - Streamline `thoth-export-server`'s XML module

## [[0.10.0]](https://github.com/thoth-pub/thoth/releases/tag/v0.10.0) - 2023-04-03
### Added
  - [42](https://github.com/thoth-pub/thoth/issues/42) - Generate MARC 21 records
  - New `work` field `bibliography_note`

## [[0.9.18]](https://github.com/thoth-pub/thoth/releases/tag/v0.9.18) - 2023-03-27
### Security
  - Upgrade `r2d2` to v0.8.10
  - Upgrade `scheduled-thread-pool` to v0.2.7
  - Upgrade `openssl` to v0.10.48
  - Upgrade `remove_dir_all` to v0.5.3

## [[0.9.17]](https://github.com/thoth-pub/thoth/releases/tag/v0.9.17) - 2023-03-25
### Changed
  - Upgrade rust to `1.68.1` in development `Dockerfile`
  - Upgrade build dependencies (npm `v9.6.2`, node `v18.15.0` and rollup `v3.20.2`) in production and development `Dockerfile`
  - Upgrade `wasm-pack` to v0.11.0

## [[0.9.16]](https://github.com/thoth-pub/thoth/releases/tag/v0.9.16) - 2023-03-24
### Added
  - [#480](https://github.com/thoth-pub/thoth/pull/480) Add field to work table to track when the work or any of its relations was last updated

### Changed
  - Removed manual character checks and derivable defaults to comply with [`rustc 1.68.0`](https://github.com/rust-lang/rust/releases/tag/1.68.0)
  - [484](https://github.com/thoth-pub/thoth/issues/484) GraphQL queries: support filtering on multiple enum variants for work status and language relation, and add filtering for works last updated before/after a specified timestamp

## [[0.9.15]](https://github.com/thoth-pub/thoth/releases/tag/v0.9.15) - 2023-03-01
### Fixed
  - Issue adding institutions in previous release

## [[0.9.14]](https://github.com/thoth-pub/thoth/releases/tag/v0.9.14) - 2023-03-01
### Changed
  - Upgrade `openssl-src` to v111.25.0
  - Upgrade `bumpalo` to v3.12.0

### Fixed
  - [#326](https://github.com/thoth-pub/thoth/issues/326) - Debounce search queries

## [[0.9.13]](https://github.com/thoth-pub/thoth/releases/tag/v0.9.13) - 2023-02-21
### Changed
  - Input actix keep alive via CLI arguments
  - Implement a failed request retry policy in client

## [[0.9.12]](https://github.com/thoth-pub/thoth/releases/tag/v0.9.12) - 2023-02-17
### Changed
  - Reduce number of concurrent requests

## [[0.9.11]](https://github.com/thoth-pub/thoth/releases/tag/v0.9.11) - 2023-02-17
### Changed
  - Upgrade rust to `1.67.1` in development `Dockerfile`
  - Upgrade build dependencies (npm `v9.5.0`, node `v18.14.1` and rollup `v3.15.0`) in production and development `Dockerfile`

## [[0.9.10]](https://github.com/thoth-pub/thoth/releases/tag/v0.9.10) - 2023-02-17
### Changed
  - Include `limit` and `offset` in `thoth-client`'s works query
  - Paginate `get_works` requests in export API using concurrent requests
  - Input number of actix workers via CLI arguments

### Added
  - Work count query to `thoth-client`

## [[0.9.9]](https://github.com/thoth-pub/thoth/releases/tag/v0.9.9) - 2023-02-16
### Changed
  - Upgrade `actix-web` to v4.3.0
  - Upgrade `actix-cors` to v0.6.4
  - Upgrade `env_logger` to v0.10.0
  - Upgrade `jsonwebtoken` to v8.2.0
  - Upgrade `strum` to v0.24.1
  - Upgrade `paperclip` to v0.8.0
  - Upgrade `graphql_client` to v0.12.0
  - Output real IP address in actix logs

## [[0.9.8]](https://github.com/thoth-pub/thoth/releases/tag/v0.9.8) - 2023-02-14
### Changed
  - Replace generic error with actual message when migrations fail
  - Upgrade node and rollup in github actions

### Added
  - Github action to check that all migrations run successfully
  - About page with organisation information

## [[0.9.7]](https://github.com/thoth-pub/thoth/releases/tag/v0.9.7) - 2023-02-02
### Fixed
  - Correct wrong fields used in `0.9.6` migration

## [[0.9.6]](https://github.com/thoth-pub/thoth/releases/tag/v0.9.6) - 2023-01-31
### Changed
  - Use inlined syntax in format strings to comply with [`rustc 1.67.0`](https://github.com/rust-lang/rust/releases/tag/1.67.0)
  - Upgrade rust to `1.67.0` in development `Dockerfile`
  - Upgrade build dependencies (npm `v9.4.0`, n `v9.0.1`, node `v18.13.0` and rollup `v3.12.0`) in production and development `Dockerfile`
  - [#457](https://github.com/thoth-pub/thoth/issues/457) - Upgrade `juniper` to v0.15.10
  - Upgrade `diesel` to v2.0.2
  - Upgrade `paperclip` to v0.8.0
  - Upgrade `graphql_client` to v0.12.0
  - Upgrade `chrono` to v0.4.23

### Fixed
  - [#469](https://github.com/thoth-pub/thoth/issues/469) - Expand DOI regex to include square brackets

## [[0.9.5]](https://github.com/thoth-pub/thoth/releases/tag/v0.9.5) - 2023-01-17
### Changed
  - Upgrade rust to `1.66.0` in development `Dockerfile`
  - Upgrade build dependencies (npm `v9.2.0`, n `v9.0.1`, node `v18.12.1` and rollup `v3.7.4`) in production and development `Dockerfile`

### Fixed
  - [#463](https://github.com/thoth-pub/thoth/issues/463) - Update Thema codes to v1.5

## [[0.9.4]](https://github.com/thoth-pub/thoth/releases/tag/v0.9.4) - 2022-12-05
### Added
  - [#414](https://github.com/thoth-pub/thoth/pull/414) - Synchronise chapters' `work_status` and `publication_date` with parent's upon parent's update

## [[0.9.3]](https://github.com/thoth-pub/thoth/releases/tag/v0.9.3) - 2022-11-21
### Added
  - [#456](https://github.com/thoth-pub/thoth/issues/456) - Implement JSON output format

### Changed
  - [#455](https://github.com/thoth-pub/thoth/issues/455) - Extend CSV output format to include all available fields

## [[0.9.2]](https://github.com/thoth-pub/thoth/releases/tag/v0.9.2) - 2022-11-01
### Changed
  - [#396](https://github.com/thoth-pub/thoth/issues/396) - Expand the list of contribution types with: SoftwareBy, ResearchBy, ContributionsBy, Indexer
  - [#451](https://github.com/thoth-pub/thoth/issues/451) - Output both short and long abstracts in Crossref DOI deposit

## [[0.9.1]](https://github.com/thoth-pub/thoth/releases/tag/v0.9.1) - 2022-10-27
### Changed
  - [#449](https://github.com/thoth-pub/thoth/issues/449) - Update EBSCO Host ONIX price type code

## [[0.9.0]](https://github.com/thoth-pub/thoth/releases/tag/v0.9.0) - 2022-10-24
### Added
  - [#333](https://github.com/thoth-pub/thoth/issues/333) - Add references to schema
  - Output references in Crossref DOI deposit
  - [#444](https://github.com/thoth-pub/thoth/issues/444) - Output abstracts in Crossref DOI deposit
  - [#443](https://github.com/thoth-pub/thoth/issues/443) - Output affiliations in Crossref DOI deposit
  - [#446](https://github.com/thoth-pub/thoth/issues/446) - Output fundings in Crossref DOI deposit

### Changed
  - Simplify syntax in CRUD methods

## [[0.8.11]](https://github.com/thoth-pub/thoth/releases/tag/v0.8.11) - 2022-10-07
### Changed
  - [#298](https://github.com/thoth-pub/thoth/issues/298) - Make database constraint errors more user-friendly in API output and APP notifications
  - Replaced docker musl image (no longer maintained) with official images, installing requirements needed for static compilation

## [[0.8.10]](https://github.com/thoth-pub/thoth/releases/tag/v0.8.10) - 2022-09-30
  - [#438](https://github.com/thoth-pub/thoth/issues/438) - Allow specifying query parameters based on the requested specification
  - Upgrade rust to `1.64.0` in development `Dockerfile`

## [[0.8.9]](https://github.com/thoth-pub/thoth/releases/tag/v0.8.9) - 2022-09-21
### Added
  - [#426](https://github.com/thoth-pub/thoth/issues/426) - Add ProQuest Ebrary ONIX 2.1 specification
  - [#420](https://github.com/thoth-pub/thoth/issues/420) - Add RNIB Bookshare to the list of supported platforms for ONIX 2.1
  - [#423](https://github.com/thoth-pub/thoth/issues/423) - Add a link to the Thoth user manual under "Docs" tab of navbar
  - Development workflow in docker

### Changed
  - [#429](https://github.com/thoth-pub/thoth/issues/429) - Incomplete metadata record errors are now returned as a 404 instead of 500
  - Added derives for `Eq` alongside `PartialEq` to comply with [`rustc 1.63.0`](https://github.com/rust-lang/rust/releases/tag/1.63.0)
  - Upgrade rust to `1.63.0` in development `Dockerfile`
  - Order contributions and relations by ordinal, and subjects by type and ordinal

### Fixed
  - [#425](https://github.com/thoth-pub/thoth/issues/425) - Fix typo in contribution type illustrator
  - [#424](https://github.com/thoth-pub/thoth/issues/424) - Fix inactive tag on catalogue

## [[0.8.8]](https://github.com/thoth-pub/thoth/releases/tag/v0.8.8) - 2022-08-02
### Added
  - [#389](https://github.com/thoth-pub/thoth/issues/389) - Streamline chapter (child work) creation process

### Changed
  - [#411](https://github.com/thoth-pub/thoth/issues/411) - Make `copyright_holder` optional
  - [#393](https://github.com/thoth-pub/thoth/issues/393) - Use en-dash in `page_interval` instead of hyphen
  - Ignore `extra_unused_lifetimes` warning until [clippy's fix](https://github.com/rust-lang/rust-clippy/issues/9014) for the false positive is live
  - Split build, test, and lint workflow job into separate jobs

## [[0.8.7]](https://github.com/thoth-pub/thoth/releases/tag/v0.8.7) - 2022-07-22
### Fixed
  - [#379](https://github.com/thoth-pub/thoth/issues/379) - Limit to 6 the number of ISBNs offered in CrossRef metadata export
  - [#388](https://github.com/thoth-pub/thoth/issues/388) - Upgrade packages flagged in Dependabot alerts

### Changed
  - [#370](https://github.com/thoth-pub/thoth/issues/370) - Upgrade Yew to v0.19

## [[0.8.6]](https://github.com/thoth-pub/thoth/releases/tag/v0.8.6) - 2022-07-01
### Added
  - [#390](https://github.com/thoth-pub/thoth/issues/390) - Implement OverDrive ONIX 3.0 specification

### Fixed
  - [#392](https://github.com/thoth-pub/thoth/issues/392) - Fix encoding of print ISBN in JSTOR ONIX output

## [[0.8.5]](https://github.com/thoth-pub/thoth/releases/tag/v0.8.5) - 2022-05-30
### Added
  - [#287](https://github.com/thoth-pub/thoth/issues/287) - Allow editing contributions (and affiliations)

### Fixed
  - [#360](https://github.com/thoth-pub/thoth/issues/360) - Prevent adding 0 as the price of a publication
  - [#376](https://github.com/thoth-pub/thoth/issues/376) - Restrict Licence field entries to URL-formatted strings

## [[0.8.4]](https://github.com/thoth-pub/thoth/releases/tag/v0.8.4) - 2022-05-11
### Added
  - [#29](https://github.com/thoth-pub/thoth/issues/29) - Implement CrossRef DOI Deposit specification
  - [#72](https://github.com/thoth-pub/thoth/issues/72) - Implement Google Books ONIX 3.0 specification

### Changed
  - [#356](https://github.com/thoth-pub/thoth/issues/356) - Upgrade actix to v4

## [[0.8.3]](https://github.com/thoth-pub/thoth/releases/tag/v0.8.3) - 2022-04-18
### Added
  - [#359](https://github.com/thoth-pub/thoth/issues/359) - Allow editing publications

## [[0.8.2]](https://github.com/thoth-pub/thoth/releases/tag/v0.8.2) - 2022-04-06
### Changed
  - Added CA certificates to docker image to allow https requests from containers

## [[0.8.1]](https://github.com/thoth-pub/thoth/releases/tag/v0.8.1) - 2022-03-11
### Added
  - [#104](https://github.com/thoth-pub/thoth/issues/104) - Implement BibTeX specification

### Changed
  - Removed unnecessary title branching logic from KBART/ONIX output formats

## [[0.8.0]](https://github.com/thoth-pub/thoth/releases/tag/v0.8.0) - 2022-03-01
### Added
  - [#341](https://github.com/thoth-pub/thoth/issues/341) - Add weight to publication

### Changed
  - Tidied verbose bools and single-character strings to comply with [`rustc 1.59.0`](https://github.com/rust-lang/rust/releases/tag/1.59.0)
  - [#300](https://github.com/thoth-pub/thoth/issues/300) - Moved width/height to Publication, added depth, improved metric/imperial display
  - Upgrade docker's base images to latest available releases

## [[0.7.2]](https://github.com/thoth-pub/thoth/releases/tag/v0.7.2) - 2022-02-08
### Changed
  - [#339](https://github.com/thoth-pub/thoth/issues/339) - Update publication types to include AZW3, DOCX and FictionBook
  - [#331](https://github.com/thoth-pub/thoth/issues/331) - Update series model to include description and CFP URL
  - Allow triggering docker action manually

### Added
  - Add code of conduct and support document to repository

## [[0.7.1]](https://github.com/thoth-pub/thoth/releases/tag/v0.7.1) - 2022-01-24
### Changed
  - Removed redundant `to_string` calls to comply with `rustc 1.58.0`
  - [#329](https://github.com/thoth-pub/thoth/issues/329) - Update EBSCO Host ONIX pricing and contributor display logic
  - Allow building docker image manually in actions

## [[0.7.0]](https://github.com/thoth-pub/thoth/releases/tag/v0.7.0) - 2022-01-11
### Added
  - [#28](https://github.com/thoth-pub/thoth/issues/28) - Implement chapter structure
  - GraphQL queries: support filtering on multiple enum variants (e.g. work types, language codes)
  - Dashboard: display Institution stats

### Fixed
  - Issues form: typing filter string in series search box has no effect on which series are displayed

## [[0.6.1]](https://github.com/thoth-pub/thoth/releases/tag/v0.6.1) - 2021-12-13
### Changed
  - Removed redundant closures and `impl`s to comply with [`rustc 1.57.0`](https://github.com/rust-lang/rust/releases/tag/1.57.0)

### Fixed
  - [#309](https://github.com/thoth-pub/thoth/issues/309) - Update Thema codes to v1.4

## [[0.6.0]](https://github.com/thoth-pub/thoth/releases/tag/v0.6.0) - 2021-11-29
### Added
  - [#92](https://github.com/thoth-pub/thoth/issues/92) - Implement institution table, replacing funder and standardising contributor affiliations

## [[0.5.0]](https://github.com/thoth-pub/thoth/releases/tag/v0.5.0) - 2021-11-29
### Added
  - [#297](https://github.com/thoth-pub/thoth/issues/297) - Implement publication location

### Changed
  - Requirement to Number fields preventing user from entering numbers below 0 for Counts/below 1 for Editions and Ordinals, and sets Contribution Ordinal default to 1 instead of 0
  - [#299](https://github.com/thoth-pub/thoth/issues/299) - Update Project MUSE ONIX subject output logic
  - Updated if and else branches to comply with [`rustc 1.56.0`](https://github.com/rust-lang/rust/releases/tag/1.56.0)

### Fixed
  - [#292](https://github.com/thoth-pub/thoth/issues/292) - Cannot unset publication date: error when trying to clear a previously set publication date
  - [#295](https://github.com/thoth-pub/thoth/issues/295) - various subforms failing to trim strings before saving (including on mandatory fields which are checked for emptiness)
  - Factored out duplicated logic for handling optional field values, simplifying the code and reducing the likelihood of further bugs such as [#295](https://github.com/thoth-pub/thoth/issues/295) being introduced
  - Minor issue where some required fields were not marked as "required" (so empty values would be sent to the API and raise an error)
  - Issue with subforms where clicking save button bypassed field requirements (so instead of displaying a warning message such as "Please enter a number", invalid values would be sent to the API and raise an error)
  - [#310](https://github.com/thoth-pub/thoth/issues/310) - Add jstor specification to formats

## [[0.4.7]](https://github.com/thoth-pub/thoth/releases/tag/v0.4.7) - 2021-10-04
### Added
  - [#43](https://github.com/thoth-pub/thoth/issues/43), [#49](https://github.com/thoth-pub/thoth/issues/49) - Implement EBSCO Host's ONIX 2.1 specification
  - [#44](https://github.com/thoth-pub/thoth/issues/44) - Implement JSTOR's ONIX 3.0 specification
  - [#253](https://github.com/thoth-pub/thoth/issues/253) - Implement Project MUSE ONIX specification tests

### Changed
  - [#242](https://github.com/thoth-pub/thoth/issues/242) - Move API models to object-specific subdirectories
  - [#274](https://github.com/thoth-pub/thoth/issues/274) - Add width/height units to CSV specification
  - [#263](https://github.com/thoth-pub/thoth/issues/263) - Add `Doi`, `Isbn` and `Orcid` types to client schema

## [[0.4.6]](https://github.com/thoth-pub/thoth/releases/tag/v0.4.6) - 2021-09-02
### Added
  - [#88](https://github.com/thoth-pub/thoth/issues/88) - Implement KBART specification
  - [#266](https://github.com/thoth-pub/thoth/issues/266) - Delete confirmation to publications

### Changed
  - [#272](https://github.com/thoth-pub/thoth/issues/272) - Use more fields in `contributors` filtering

### Fixed
  - [#271](https://github.com/thoth-pub/thoth/issues/271) - Make filter parameter optional in `subjectCount`

## [[0.4.5]](https://github.com/thoth-pub/thoth/releases/tag/v0.4.5) - 2021-08-12
### Added
  - [#259](https://github.com/thoth-pub/thoth/issues/259) - Units selection dropdown to Work and NewWork pages, which updates the Width/Height display on change
  - [#259](https://github.com/thoth-pub/thoth/issues/259) - Local storage key to retain user's choice of units across all Work/NewWork pages
  - [#259](https://github.com/thoth-pub/thoth/issues/259) - Backend function to convert to/from database units (mm): uses 1inch = 25.4mm as conversion factor, rounds mm values to nearest mm, rounds cm values to 1 decimal place, rounds inch values to 2 decimal places
  - [#259](https://github.com/thoth-pub/thoth/issues/259) - Constraints on Width/Height fields depending on unit selection: user may only enter whole numbers when in mm, numbers with up to 1 decimal place when in cm, numbers with up to 2 decimal places when in inches

### Changed
  - [#259](https://github.com/thoth-pub/thoth/issues/259) - GraphQL and APP queries to specify units when submitting new Width/Height values, and handle conversion if required

## [[0.4.4]](https://github.com/thoth-pub/thoth/releases/tag/v0.4.4) - 2021-08-02
### Fixed
  - Read button in catalogue now uses the landing page URL instead of the DOI

### Changed
  - Removed needless borrow to comply with `clippy` under [`rustc 1.54.0`](https://github.com/rust-lang/rust/releases/tag/1.54.0)

## [[0.4.3]](https://github.com/thoth-pub/thoth/releases/tag/v0.4.3) - 2021-07-28
### Added
  - [#48](https://github.com/thoth-pub/thoth/issues/48) - Implement OAPEN ONIX 3.0 specification

### Fixed
  - [#254](https://github.com/thoth-pub/thoth/issues/254) - Ensure order of fields in create work match those in edit work

## [[0.4.2]](https://github.com/thoth-pub/thoth/releases/tag/v0.4.2) - 2021-07-05
### Added
  - [#125](https://github.com/thoth-pub/thoth/issues/125) - Implement `ISBN` type to standardise parsing
  - [#217](https://github.com/thoth-pub/thoth/issues/217) - Add "Contribution Ordinal" field to indicate order of contributions within a work

## [[0.4.1]](https://github.com/thoth-pub/thoth/releases/tag/v0.4.1) - 2021-06-22
### Changed
  - [#234](https://github.com/thoth-pub/thoth/issues/234) - Move database calls out of GraphQL model

### Added
  - [#136](https://github.com/thoth-pub/thoth/issues/135), [#233](https://github.com/thoth-pub/thoth/issues/233) - Implement `Doi` and `Orcid` types to standardise parsing
  - `thoth-errors` crate to share `ThothError` and `ThothResult`

## [[0.4.0]](https://github.com/thoth-pub/thoth/releases/tag/v0.4.0) - 2021-06-15
### Changed
  - Updated `yew` to [`v0.18.0`](https://github.com/yewstack/yew/releases/tag/web-v3.3.2)
  - Catch client errors with `ThothError::EntityNotFound`
  - Use a custom instance of GaphiQL
  - Unify `Work` output structure in client using GraphQL fragments

### Added
  - [#235](https://github.com/thoth-pub/thoth/issues/235) - Export API with openapi schema
  - [#110](https://github.com/thoth-pub/thoth/issues/110) - Output to CSV
  - Rapidoc schema explorer interface

### Removed
  - `actix_rt`

## [[0.3.6]](https://github.com/thoth-pub/thoth/releases/tag/v0.3.6) - 2021-05-11
### Fixed
  - Problem building docker image

## [[0.3.5]](https://github.com/thoth-pub/thoth/releases/tag/v0.3.5) - 2021-05-11
### Added
  - [#213](https://github.com/thoth-pub/thoth/issues/213) - Link to documentation in readme
  - [#206](https://github.com/thoth-pub/thoth/issues/206) - Notify user when a new version of the APP is available
  - [#231](https://github.com/thoth-pub/thoth/issues/231) - Link to publication page in work page
  - [#224](https://github.com/thoth-pub/thoth/issues/224) - Implement limit and offset in linked queries
  - Implement Crud trait with database calls per object

### Changed
  - [#236](https://github.com/thoth-pub/thoth/issues/236) - Split server logic into individual crates
  - Update rustc to 1.51.0 in docker image
  - Replace composite keys in `contribution` and `issue` with standard UUIDs
  - Server configuration parsed from binary

### Fixed
  - [#216](https://github.com/thoth-pub/thoth/issues/216), [#228](https://github.com/thoth-pub/thoth/issues/228) - Error adding multiple subjects


## [[0.3.4]](https://github.com/thoth-pub/thoth/releases/tag/v0.3.4) - 2021-03-29
### Fixed
  - Upgraded rusct in docker image. Moved `wasm-pack` to a less fragile build stage using official image, keeping main build statically compiled

## [[0.3.3]](https://github.com/thoth-pub/thoth/releases/tag/v0.3.3) - 2021-03-26
### Added
  - [#120](https://github.com/thoth-pub/thoth/issues/120) - Implement table sorting by columns in APP
  - [#203](https://github.com/thoth-pub/thoth/issues/203) - Cascade filtering options to relation queries in API

### Changed
  - [#210](https://github.com/thoth-pub/thoth/issues/210) - Specify .xml extension when outputting ONIX files

### Fixed
  - [#182](https://github.com/thoth-pub/thoth/issues/182) - Ensure issue's series and work have the same imprint


## [[0.3.2]](https://github.com/thoth-pub/thoth/releases/tag/v0.3.2) - 2021-03-09
### Added
  - [#202](https://github.com/thoth-pub/thoth/issues/202) - Enum type filtering in GraphQL queries
  - [#202](https://github.com/thoth-pub/thoth/issues/202) - Query works by DOI
  - [#195](https://github.com/thoth-pub/thoth/issues/195) - Prompt confirmation upon delete

### Fixed
  - [#199](https://github.com/thoth-pub/thoth/issues/199), [#201](https://github.com/thoth-pub/thoth/issues/201) - Error displaying publications if filtering on empty ISBN or URL
  - Trigger a warning when the current user does not have any editting permissions

## [[0.3.1]](https://github.com/thoth-pub/thoth/releases/tag/v0.3.1) - 2021-03-04
### Fixed
  - [#197](https://github.com/thoth-pub/thoth/issues/197) - Error deserialising publications in APP

## [[0.3.0]](https://github.com/thoth-pub/thoth/releases/tag/v0.3.0) - 2021-03-03
### Changed
  - [#162](https://github.com/thoth-pub/thoth/issues/162) - Only records linked to publishers user has access to are listed in APP
  - [#167](https://github.com/thoth-pub/thoth/issues/167) - Make work contribution the canonical source of contributor names in ONIX output

### Added
  - [#177](https://github.com/thoth-pub/thoth/issues/177) - Allow querying objects by linked publisher(s)
  - [#159](https://github.com/thoth-pub/thoth/issues/159), [#160](https://github.com/thoth-pub/thoth/issues/160), [#161](https://github.com/thoth-pub/thoth/issues/161) - Add publisher accounts
  - [#163](https://github.com/thoth-pub/thoth/issues/163) - Save a snapshot of each object upon update
  - [#164](https://github.com/thoth-pub/thoth/issues/164), [#165](https://github.com/thoth-pub/thoth/issues/165) - Add contributor names to contribution
  - [#168](https://github.com/thoth-pub/thoth/issues/168) - Warn users when editing a contributor or a funder that is linked to a work
  - [#185](https://github.com/thoth-pub/thoth/issues/185) - Allow resetting user passwords through CLI
  - Allow creating publisher accounts through CLI

### Fixed
  - [#181](https://github.com/thoth-pub/thoth/issues/181) - Enforce numeric values for issue ordinal

## [[0.2.13]](https://github.com/thoth-pub/thoth/releases/tag/v0.2.13) - 2021-01-14
### Changed
  - Update API URL in docker github action
  - Remove staging tag in docker github action

## [[0.2.12]](https://github.com/thoth-pub/thoth/releases/tag/v0.2.12) - 2021-01-12
### Changed
  - [#153](https://github.com/thoth-pub/thoth/issues/153) - Implement created and updated dates to each structure

## [[0.2.11]](https://github.com/thoth-pub/thoth/releases/tag/v0.2.11) - 2021-01-06
### Changed
  - [#151](https://github.com/thoth-pub/thoth/issues/151) - Make browser prompt user to save Onix XML to file
  - [#143](https://github.com/thoth-pub/thoth/issues/143) - Start using Github Actions instead of Travis

### Added
  - [#121](https://github.com/thoth-pub/thoth/issues/121) - Add created and updated dates to each table

## [[0.2.10]](https://github.com/thoth-pub/thoth/releases/tag/v0.2.10) - 2021-01-04
### Changed
  - [#127](https://github.com/thoth-pub/thoth/issues/127) - Do not exit main entity edit pages upon saving
  - [#147](https://github.com/thoth-pub/thoth/issues/147) - Remove subject code validation for non open subject headings

## [[0.2.9]](https://github.com/thoth-pub/thoth/releases/tag/v0.2.9) - 2020-11-24
### Changed
  - Hide creative commons icon when license is unset in APP catalogue

### Added
  - Display book cover placeholder when cover URL is unset
  - Status tags to APP catalogue

## [[0.2.8]](https://github.com/thoth-pub/thoth/releases/tag/v0.2.8) - 2020-11-23
### Changed
  - Upgrade fontawesome to v5.4.0

### Added
  - Information banner to APP homepage
  - New BISAC codes

## [[0.2.7]](https://github.com/thoth-pub/thoth/releases/tag/v0.2.7) - 2020-11-19
### Changed
  - [#118](https://github.com/thoth-pub/thoth/issues/118) - Ensure empty data is sent as null not as empty strings
  - [#131](https://github.com/thoth-pub/thoth/issues/131) - Moved forms with relationships outside main object form

## [[0.2.6]](https://github.com/thoth-pub/thoth/releases/tag/v0.2.6) - 2020-11-13
### Changed
  - Fix pricing functionality ommitted in previous release

## [[0.2.5]](https://github.com/thoth-pub/thoth/releases/tag/v0.2.5) - 2020-11-13
### Added
  - New BISAC codes

## [[0.2.4]](https://github.com/thoth-pub/thoth/releases/tag/v0.2.4) - 2020-11-10
### Added
  - Implemented pricing CRUD in APP

## [[0.2.3]](https://github.com/thoth-pub/thoth/releases/tag/v0.2.3) - 2020-11-06
### Added
  - Implemented pagination in all admin components
  - Implemented pagination in catalogue

## [[0.2.2]](https://github.com/thoth-pub/thoth/releases/tag/v0.2.2) - 2020-11-03
### Changed
  - Set `THOTH_API` on build via docker

## [[0.2.1]](https://github.com/thoth-pub/thoth/releases/tag/v0.2.1) - 2020-11-02
### Changed
  - Redirect to relevant routes upon save and create actions in APP

### Added
  - Delete functionality in all APP objects

## [[0.2.0]](https://github.com/thoth-pub/thoth/releases/tag/v0.2.0) - 2020-10-23
### Changed
  - [#38](https://github.com/thoth-pub/thoth/issues/38) - Split client and server
  - [#98](https://github.com/thoth-pub/thoth/issues/98) - Streamline Thoth logo

### Added
  - [#97](https://github.com/thoth-pub/thoth/issues/97), [#39](https://github.com/thoth-pub/thoth/issues/39), [#41](https://github.com/thoth-pub/thoth/issues/41) - Implement WASM frontend with Yew
  - [#40](https://github.com/thoth-pub/thoth/issues/40) - Implement API authentication

## [[0.1.10]](https://github.com/thoth-pub/thoth/releases/tag/v0.1.10) - 2020-06-03
### Changed
  - Roadmap button in index catalogue

## [[0.1.9]](https://github.com/thoth-pub/thoth/releases/tag/v0.1.9) - 2020-06-03
### Added
  - Roadmap document

## [[0.1.8]](https://github.com/thoth-pub/thoth/releases/tag/v0.1.8) - 2020-06-02
### Changed
  - New design for the index catalogue

## [[0.1.7]](https://github.com/thoth-pub/thoth/releases/tag/v0.1.7) - 2020-03-27
### Changed
  - [#35](https://github.com/thoth-pub/thoth/issues/35) - Fix date format and lack in ONIX sender header
  - Add place of publication to ONIX file
  - Use code 03 (description) instead of 30 (abstract) in OAPEN ONIX

## [[0.1.6]](https://github.com/thoth-pub/thoth/releases/tag/v0.1.6) - 2020-03-26
### Changed
  - Fix incompatibilities with OAPEN ONIX parser
  - Map ONIX parameter to UUID directly, instead of converting afterwards
  - Normalise server route definitions

## [[0.1.5]](https://github.com/thoth-pub/thoth/releases/tag/v0.1.5) - 2020-03-25
### Changed
  - Load assets statically

## [[0.1.4]](https://github.com/thoth-pub/thoth/releases/tag/v0.1.4) - 2020-03-24
### Changed
  - "/" now renders its own page, instead of redirecting to "/graphiql"
  - [#27](https://github.com/thoth-pub/thoth/issues/27) - Produce an OAPEN compatible ONIX file

### Added
  - [#26](https://github.com/thoth-pub/thoth/issues/26) - Create an endpoint to allow generating ONIX streams from "/onix/{workId}"

### Removed
  - Dropped support for creating ONIX from binary

## [[0.1.3]](https://github.com/thoth-pub/thoth/releases/tag/v0.1.3) - 2020-03-16
### Changed
  - Pin compiler's docker image to a specific version (best practice)
  - Use COPY instead of ADD for directories in Dockerfile (best practice)
  - [#24](https://github.com/thoth-pub/thoth/issues/24) - Implemented rust style guidelines

### Added
  - [#23](https://github.com/thoth-pub/thoth/issues/23) - Redirect "/" to "/graphiql"
  - [#18](https://github.com/thoth-pub/thoth/issues/18) - Create ThothError structure to start catching all other types of errors
  - [#24](https://github.com/thoth-pub/thoth/issues/24) - Enforce rust style guidelines using husky (pre-push hook) and travis
  - [#17](https://github.com/thoth-pub/thoth/issues/17) - Allow producing a proto ONIX file from the binary

## [[0.1.2]](https://github.com/thoth-pub/thoth/releases/tag/v0.1.2) - 2020-03-03
### Changed
  - [#10](https://github.com/thoth-pub/thoth/issues/10) - Port exposing is handled in Dockerfile instead of docker-compose
  - [#16](https://github.com/thoth-pub/thoth/issues/16) - Moved server start function from binary to library
  - [#9](https://github.com/thoth-pub/thoth/issues/9) - Docker image is now compiled statically

### Added
  - [#13](https://github.com/thoth-pub/thoth/issues/13) - Added limit and offset arguments to all queries
  - [#13](https://github.com/thoth-pub/thoth/issues/13) - Added default order by clauses to all queries
  - [#15](https://github.com/thoth-pub/thoth/issues/15) - Implemented GraphQL errors for diesel errors
  - [#13](https://github.com/thoth-pub/thoth/issues/13) - Added filter arguments for publishers and works queries

## [[0.1.1]](https://github.com/thoth-pub/thoth/releases/tag/v0.1.1) - 2020-02-27
### Changed
  - Improved Dockerfile to allow running database migrations at run time

### Added
  - Implemented imprints for publisher graphql object
  - [#6](https://github.com/thoth-pub/thoth/issues/6) - Added subcommands to main binary to allow running embedded migrations without having to install diesel\_cli
  - Automatic publication to crates.io

## [[0.1.0]](https://github.com/thoth-pub/thoth/releases/tag/v0.1.0) - 2020-02-21
### Added
  - Database migrations
  - GraphQL handlers implementing the thoth schema
