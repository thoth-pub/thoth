# Changelog
All notable changes to thoth will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
  - [805](https://github.com/thoth-pub/thoth/pull/805) - `BE-02`: implement the approved bounded **distribution platform model** as an inactive additive foundation. Adds the closed 17-value `DistributionPlatform` inventory from `ADR-0004` (no `OTHER`, no fallback, no `Default`, no shared enum or conversion with Thoth Metrics), code-owned compile-time-exhaustive platform descriptors, the PostgreSQL `distribution_platform` enum and the additive `publisher_distribution_platform` relation (composite primary key `(publisher_id, platform)`, `ON DELETE CASCADE` foreign key to `publisher`, named `enabled == (disabled_at IS NULL)` check constraint, partial enabled index and `set_updated_at` trigger), with `thoth-api/src/schema.rs` updated manually and atomically in the same PR under `ADR-0003` Architecture A. Implements the six-transition assignment activation lifecycle (retained disabled rows, application-generated activation UUIDs, one transaction timestamp per transition, same-state no-ops that move no timestamp) and atomic OAPEN/DOAB linked normalization that repairs one-sided, split-activation and split-timestamp pairs rather than treating them as idempotent; `OCLC_KB` and `EX_LIBRIS_KB` stay independently assignable, and `JISC_NBK` is included but inactive and non-assignable, failing closed before any write through the new stable `ThothError::DistributionPlatformNotAssignable` variant. Adds exactly four public additive GraphQL read surfaces — `distributionPlatformOptions`, `publishersByDistributionPlatform`, `publisherCountByDistributionPlatform` and `Publisher.distributionPlatforms` — with two new object types and three new enums, and **no** new mutation, input, scalar or interface; activation IDs, disabled history, adapter/feed identity, package/capability state and protected configuration are not exposed. `Publisher.distributionPlatforms` is the **first production consumer of the `ADR-0007` request-local non-cached DataLoader foundation**: a typed loader added to the existing `RequestLoaders` bundle, keyed on `publisher_id`, built through `configured_loader` with explicit `200`/`10`, loader-first at resolver entry, `try_load` only, total fail-closed batches, and one set-based `eq_any` statement per dispatch chunk executed entirely inside `tokio::task::spawn_blocking`. **Inactive foundation only: no distribution is activated, no distribution job or dissemination exists, the migration creates zero assignment rows, no production migration, deployment, backfill or assignment is performed, mutation-guard mode is unchanged, and no `BE-03`, `BE-04`, `MIG-01`, OAI, Metrics or PR [799](https://github.com/thoth-pub/thoth/pull/799) work is included.**
  - [788](https://github.com/thoth-pub/thoth/pull/788) - `BE-02-SPEC`: reconcile the bounded BE-02 distribution-platform model specification against repository-authoritative `ADR-0007` and the merged request-scoped non-cached DataLoader foundation; replace the obsolete N+1 architecture escalation with the first-production-consumer loader-first contract (request-local non-cached loader, explicit `200`/`10`, `try_load`, total fail-closed batches, set-based Diesel inside `spawn_blocking`, and real-SQL `250 -> [200, 50] -> 2 statements` evidence), while preserving the approved 17-value distribution-platform inventory, assignment lifecycle, linked OAPEN/DOAB normalization, migration, public GraphQL API and rollback decisions. Documentation/control only - no runtime, Cargo, GraphQL implementation, schema implementation, migration implementation, workflow or infrastructure change; no implementation branch; no deployment or production action; no BE-03, BE-04, MIG-01, dissemination, Metrics, mutation-guard or PR #799 action. BE-02 implementation remains **NOT AUTHORIZED** pending fresh independent exact-head specification review, CTO specification approval, merge/repository authority and then separate fresh-base implementation authorization.
  - [802](https://github.com/thoth-pub/thoth/pull/802) - `THOTH-GQL-DATALOADER-01`: implement the `ADR-0007` B0 request-scoped GraphQL **DataLoader foundation** and retire the superseded `ADR-0006` A2 batching machinery. Adds the pinned `dataloader 0.18.0` dependency (default features off, Tokio runtime support on) and a direct `tokio` (`rt`) dependency to `thoth-api`; a request-local, **non-cached** loader bundle (`RequestLoaders`) owned directly by the real GraphQL `Context` and dropped with it, with explicit `200` max-batch-size / `10` yield-count construction, `try_load`-only load API, total fail-closed batch functions, set-based Diesel (`eq_any`) behind a `tokio::task::spawn_blocking` boundary with no connection across `.await`, and a safe cloneable non-serde batch-error projection preserving each field family's current GraphQL error convention. Migrates the general GraphQL unit-test execution path from `juniper::execute_sync` to async Juniper through one bounded central bridge with explicit nested-runtime misuse failure. Removes the unused `ADR-0006` A2 infrastructure (`GraphqlBatchStore`/batching store, look-ahead prefetch, response-scope shim, A2 fixtures and A2-specific tests) and decouples loader availability from mutation-guard mode (`MutationGuardMode::store_available()` removed; `Context` construction no longer takes a guard mode), rehosting the guard/query-path/baseline/directive/duplicate-mutation regression evidence onto A2-independent fixtures. **Adopts no production GraphQL child field; changes no public SDL; performs no database or data migration; changes no guard mode (production remains `OFF`); activates no `OBSERVE`/`ENFORCE`; implements no `BE-02` or Thoth Metrics adoption; deploys nothing.** The pinned-Juniper duplicate top-level mutation execution finding remains a live, separately controlled concern and is **not** fixed by this change
  - [801](https://github.com/thoth-pub/thoth/pull/801) - `THOTH-GQL-DATALOADER-SPEC-01`: propose the `THOTH-GQL-DATALOADER-01` implementation specification for the `ADR-0007` B0 request-scoped GraphQL **DataLoader foundation** and `ADR-0006` A2 retirement (`docs/engineering/ai-delivery/tasks/THOTH-GQL-DATALOADER-01.md`). **Specification and control record only — no runtime, Cargo, GraphQL, schema, migration, workflow or infrastructure change, no implementation, and no implementation branch created.** The proposed specification carries the approved `ADR-0007` architecture into a bounded implementable task: request-local non-cached `dataloader 0.18.x` loaders owned by the real GraphQL `Context`, async Juniper as the supported test/resolver execution model with one bounded central test bridge replacing `execute_sync`, explicit `200`/`10` batching configuration, `try_load`-only load API with total fail-closed batch functions, set-based Diesel behind a `tokio::task::spawn_blocking` boundary with no connection across `.await`, a safe non-panicking shareable batch-error representation (the spike's serde round-trip clone is prohibited as production plumbing), retirement of the unused A2 `GraphqlBatchStore`/prefetch/scope infrastructure, decoupling of batching availability from mutation-guard mode, and rehosting of the guard/duplicate-mutation regression evidence onto A2-independent fixtures. The foundation adopts **no** production GraphQL field and must keep production SDL byte-identical. Status is `APPROVED`: after independent exact-head review (`CHANGES REQUIRED`), bounded remediation of the three review findings (`query_path` preservation split, `mutation_guard.rs` boundary/`store_available()` removal, explicit observability/operations disposition) and fresh review of the corrected content, the CTO granted **specification approval only** on 2026-08-11 at exact PR head `bb4f57e0d446270b4c404879ef61b5e229e90bbf`; `THOTH-GQL-DATALOADER-01` implementation remains **NOT AUTHORIZED** — implementation authorization is a distinct later CTO decision after the approved specification is repository-authoritative on `develop`; `BE-02` and Thoth Metrics adoption, `OBSERVE`/`ENFORCE` activation, and PR [799](https://github.com/thoth-pub/thoth/pull/799)/`OPS-03` all remain outside scope and unauthorized.
  - [798](https://github.com/thoth-pub/thoth/pull/798) - `THOTH-GQL-OPS-03-SPEC`: finalize the `THOTH-GQL-OPS-03` effective-mode **fleet-verification** specification as an approval candidate, reconciled against the post-`THOTH-GQL-OPS-02` repository state and resolving its one outstanding architecture decision. **Specification and control only — no runtime, GraphQL, schema, migration, workflow or infrastructure change, no implementation, and no branch created for the implementation task.** The specification's mandatory section 3.2 **information-disclosure boundary is now selected**: the effective-mode verification signal must be available only through an **orchestration/administrative-plane or equivalent out-of-band per-instance mechanism**, a **public unauthenticated effective-mode surface is REJECTED**, the public GraphQL schema stays unchanged, and the mechanism may disclose only the process's actual effective `MutationGuardMode` plus the **minimum runtime identity** needed to correlate one observation to one orchestrator-enumerated instance — never a secret, credential, environment-variable value, deployment configuration, publisher, user or request data, or unnecessary topology metadata. The reason is recorded rather than assumed: the mode describes server-side **request-acceptance policy** and therefore carries reconnaissance value, no public caller needs it, and a public surface cannot address an individual replica behind the shared load balancer, so it would buy the disclosure without buying the capability; the alternatives considered — a public route or `ApiConfig` field, a public GraphQL field, and an authenticated public surface — are each recorded with their disposition, and reopening the decision is reserved to the CTO rather than available to the implementing task. **The pre-`THOTH-GQL-OPS-02` premise is corrected throughout:** the specification no longer describes the image-default `init` path as ignoring `THOTH_GRAPHQL_MUTATION_GUARD_MODE`, because `THOTH-GQL-OPS-02` merged and closed capability gap 1 in-repository. The **silent-adoption failure class survives that fix and is now stated generically** — configured intent is not proof of process-effective mode — with capability gap 2 recorded as the one still open: no mechanism proves the effective mode of every serving process. Acceptance criteria were **strengthened, never weakened**: complete fleet enumeration is now *required* rather than merely supported, incomplete coverage must **fail closed** with `UNKNOWN` kept structurally distinct from `OFF` (new AC-22), a negative test must prove **no public unauthenticated caller** can obtain the mode from any existing route (new AC-11.1), the disclosed correlation identity must be justified field by field (new AC-23), the silent-adoption test must construct the divergence deliberately rather than depend on the closed `init` defect, and the merged `THOTH-GQL-OPS-02` behaviour must be regression-pinned in both build profiles. Dependencies are reconciled to current truth — `ADR-0006` repository-authoritative, `THOTH-GQL-BATCH-01`, `THOTH-GQL-OPS-01` and now `THOTH-GQL-OPS-02` merged — and a fresh exact `develop` base plus separate explicit CTO implementation authorization remain required. **This is an approval candidate, not an approval:** the specification stays `Status: DRAFT` with `Implementation: NOT AUTHORIZED`, the reserved implementation branch `feature/shared-architecture/graphql-guard-mode-fleet-verification` is **absent** and must remain so until implementation is authorized, and approving the specification decides only the specification and its section 3.2 boundary. **CG-13 remains OPEN**, the `ADR-0006` runtime-operations gate remains **NOT SATISFIED**, the mode-transition runbook remains **PROVISIONAL**, `OBSERVE`, `ENFORCE` and `BE-02` runtime remain **NOT AUTHORIZED**, `THOTH-GQL-OPS-04` remains unimplemented, control limitation CL-1 on implementing-agent access to secret-bearing production configuration is unchanged and unclosed, and no deployment, configuration, mode transition or protected-source read occurred
  - [793](https://github.com/thoth-pub/thoth/pull/793) - `THOTH-GQL-OPS-01`: implement the approved GraphQL mutation-guard **runtime-operations** control work. Adds the evidenced operational-control record and the **PROVISIONAL** mode-transition runbook required by `ADR-0006` section 8.3.5, and the three prerequisite task specifications `THOTH-GQL-OPS-02`, `THOTH-GQL-OPS-03` and `THOTH-GQL-OPS-04` — each `DRAFT` with implementation `NOT AUTHORIZED` and no branch created. Every operational statement carries a named evidence source and an evidence class (`[REPO]`, `[EXTERNAL]`, `[REPO + EXTERNAL]` or `[UNVERIFIED]`), and no deployed-state conclusion is attributed to repository evidence alone. Re-derived independently at the exact base and **confirmed, not refuted**: the mode is read **once at process start** with no reload path, signal handler, watcher or admin route anywhere in the workspace, so changing the effective mode of a running process is impossible; **no** surface exposes the effective mode of a serving instance, leaving `OFF` and `OBSERVE` externally indistinguishable; and the image's default `init` command does not register the guard argument, so — traced through pinned `clap_builder` 4.6.0, where `verify_arg` returns `UnknownArgument` only under `cfg(debug_assertions)`, and reproduced in an isolated throwaway probe outside this repository — a **guard-enabled** release build started through `init` silently ignores `THOTH_GRAPHQL_MUTATION_GUARD_MODE` and is unconditionally `OFF`, while the same path panics in a debug build. Newly established and recorded: on the `init` path an **invalid** value is also silently ignored rather than rejected, because the `value_parser` never runs; the container environment is the **sole** configuration source, with pinned `dotenv` unable to override it and no `.env` present in the `FROM scratch` image; the production GraphQL API service supplies **no** container command override and so inherits `init`, while a different Thoth service in the same definition does override its command, making the inheritance a deliberate configuration state; the serving fleet is **autoscaled**, so the expected population is a range with a live current value and never a copied number; rolling replacement makes a mixed-mode window **structurally guaranteed** rather than exceptional; a mode change is **both** a configuration change **and** a deployment, positively contradicting the withdrawn "configuration change without a deploy" claim; rollback uses the forward change's configuration/deployment mechanism and is technically executed by the same execution-capability team, while its actual latency and whether it additionally requires CTO approval both remain `[UNVERIFIED]` — no authorization equivalence is inferred from sharing the technical mechanism — and it is therefore **not** a kill switch; and runtime log retention is configured to a **finite** duration, while the approved `OBSERVE` observation-window duration, whether that retention covers it, and any remedy are all `[UNVERIFIED]` and **downstream** — a remedy cannot be selected before the duration it must cover is approved, so `THOTH-GQL-OPS-04` records the requirement and the unresolved dependency rather than choosing one. The **test** environment was established to be **pre-guard** as well as production — both run a release whose binary contains no mutation guard, so neither has a guard mode and neither may be described as `MutationGuardMode::OFF`; there is consequently no environment in which a mode could currently be changed, and the future rehearsal must first deploy a guard-enabled candidate to a non-production environment. Ownership is recorded as roles, not individuals, and **execution capability is deliberately not relabelled as accountable ownership**: the technical team holding maintain permission on the authoritative deployment source is established as technically able to execute a change, approval authority for merge and for **each** activation is the CTO, but the **accountable production runtime owner** and the **post-activation observation sign-off owner** are both recorded `[UNVERIFIED]` — the first needs a CTO designation no amount of further reading can supply, the second is a proposal awaiting the CTO's own confirmation — and whether operational rollback additionally requires CTO approval is likewise unresolved. Four acceptance criteria are therefore recorded **FAIL/BLOCKED** rather than converted into passes, which is the expected shape of a task terminating at disposition `C`. The fleet-verification mechanism is **specified and deliberately not implemented**, with its information-disclosure boundary made explicit rather than treated as a formatting choice; partial-fleet handling distinguishes the relatively benign `OFF`/`OBSERVE` observation gap from `OBSERVE`/`ENFORCE` request-acceptance inconsistency; and the timed rollback rehearsal is defined with its four measurements but **not executed** and **not owned by `THOTH-GQL-OPS-04`** — it, the service-health/activation-threshold gate and the preview/staging acceptance gate all remain **downstream** of the runtime-operations gate, so every timing field stays marked `TO BE MEASURED AT PREVIEW/STAGING GATE` and no numeric threshold or duration is invented anywhere. The `THOTH-GQL-OPS-02` remediation class remains feature-local and must **preserve all existing `init` migration and startup semantics**; a production container-command override is recorded everywhere it is mentioned as **not** an interchangeable fix, because `init` runs migrations before starting the API and an override would remove migration execution from the deployment path, and the mechanism is deliberately **not** selected here. Documentation and control only — no runtime, GraphQL, schema, migration, `Cargo`, `Dockerfile` or workflow change; no production configuration value, secret, resource identifier or private hostname recorded; all reads of the secret-bearing private deployment source were read-only, metadata-only and narrowly scoped, and the credential exposure encountered there is escalated rather than remediated. No deployment, no production action, no environment transitioned and no guard-enabled candidate activated. Terminal disposition **`C - insufficient operational capability/evidence; BLOCKED`**, with the `ADR-0006` runtime-operations gate recorded **NOT SATISFIED**: both capability gaps remain open, **CG-13 remains OPEN**, `THOTH-GQL-OPS-02` and `THOTH-GQL-OPS-03` remain blocking prerequisites and `THOTH-GQL-OPS-04` the earliest possible closure. Production `OBSERVE` and `ENFORCE` remain `NOT AUTHORIZED`, `BE-02` runtime remains `NOT AUTHORIZED`, [PR #788](https://github.com/thoth-pub/thoth/pull/788) and [issue #765](https://github.com/thoth-pub/thoth/issues/765) are unmodified, and the exact implementation head requires fresh independent exact-head review and separate explicit CTO merge authorization
  - [792](https://github.com/thoth-pub/thoth/pull/792) - `THOTH-GQL-OPS-01-SPEC`: add the `THOTH-GQL-OPS-01` specification for GraphQL mutation-guard **runtime operations** — which opens, and deliberately does **not** close, the runtime-operations gate in the approved `ADR-0006` activation sequence after the inert `THOTH-GQL-BATCH-01` foundation merged. It is a **bounded feature-specific successor** to control gap CG-13, covering only mode control for `THOTH_GRAPHQL_MUTATION_GUARD_MODE`: runtime owner, configuration authority, restart/redeploy semantics, propagation to every serving replica, fleet-wide verification of the **effective** mode, partial-fleet detection and handling, rollback, change/rollback authority, a timed preview/staging rehearsal requirement and retained audit evidence. Discovery established, at the exact base, that the mode is read **once at process start** with no reload path anywhere in the workspace, so a mode change always requires a new process; that **no** surface exposes the effective mode of a serving instance, leaving `OFF` and `OBSERVE` externally indistinguishable; and — reproduced in an isolated probe outside this repository against pinned `clap` 4.6.1 — that the image's default `init` command does **not** register the guard argument, so in a release build a **guard-enabled** process started through `init` silently ignores `THOTH_GRAPHQL_MUTATION_GUARD_MODE` and is unconditionally `OFF` (in a debug build the same path panics). It also established that merge, release and activation are three distinct states: the release currently deployed to production **predates** `THOTH-GQL-BATCH-01`, its binary contains no mutation guard at all, and it is therefore recorded as **pre-guard** rather than as `MutationGuardMode::OFF` — merging the foundation deployed nothing. That conclusion carries explicit evidence provenance and is attributed to no single source: `[REPO]` establishes only that the relevant release/`master` code contains no `MutationGuardMode`, defines no `mutation_guard_mode()` and has no guard wiring on its GraphQL startup path, while `[EXTERNAL]` — previously established scoped authoritative deployment metadata — establishes which release/image production actually runs, so current production is `PRE-GUARD` on `[REPO + EXTERNAL]` evidence together; a deployed-state conclusion may never be attributed to repository evidence alone, and where the external half is unobtainable under the scoped-read rules it is downgraded to `[UNVERIFIED]` rather than re-derived or obtained by widening access. That failure is silent but **fail-safe** and does not affect the correctness of the merged inert state; it is recorded as a bounded, separately specified, separately reviewed prerequisite of `OBSERVE`, explicitly **not** remediated here and explicitly **not** an architecture change. **These are capability gaps, not merely evidence gaps:** production configuration inherits the image's `init` command, so the current deployment path could not consume the guard-mode input once guard-enabled code is deployed — making an `OFF -> OBSERVE` transition of a guard-enabled candidate not operationally performable through that path — and no implemented mechanism could verify a change if one could be made — so `THOTH-GQL-OPS-01` is required to terminate at CG-13 disposition **`C - BLOCKED`** with the `ADR-0006` runtime-operations gate recorded **NOT SATISFIED**, and is forbidden from returning disposition `A` while either gap holds. Closing them requires separate bounded, independently reviewed and merged tasks — `THOTH-GQL-OPS-02` (mode-control path) and `THOTH-GQL-OPS-03` (fleet-verification mechanism) — after which `THOTH-GQL-OPS-04` may re-verify and decide on evidence; specifying a prerequisite is never delivery of it, and a specification for a verifier is never a verified fleet. The permitted `-02` remediation class is feature-local and must **preserve all existing `init` migration and startup semantics**; replacing the production container command with `start graphql-api` is **not** an interchangeable fix, because `init` runs migrations before starting the API and an override would remove migration execution from the deployment path, requiring separate migration/deployment-control analysis and approval. The specification requires every operational statement to carry an evidence class, forbids inventing any propagation or rollback duration, forbids inferring a runtime platform from the Docker image or release workflow, requires the smallest fleet-verification mechanism to be **specified but not implemented**, marks the runbook **PROVISIONAL**, binds all reads of the secret-bearing private deployment source to scoped, metadata-only retrieval, and keeps service-health signals and activation thresholds as the separate next gate. Documentation and control only — no runtime, GraphQL, schema, migration, `Cargo`, `Dockerfile` or workflow change, no production configuration value or secret recorded, no deployment, no environment transitioned and no guard-enabled candidate activated, and no modification of [PR #788](https://github.com/thoth-pub/thoth/pull/788), its branch or [issue #765](https://github.com/thoth-pub/thoth/issues/765). **CG-13 remains OPEN** and is only cross-referenced to the proposed successor; `THOTH-GQL-OPS-01` is `DRAFT` with implementation `NOT AUTHORIZED`, production `OBSERVE` and `ENFORCE` remain `NOT AUTHORIZED`, `BE-02` runtime remains `NOT AUTHORIZED`, and the exact specification head requires fresh independent exact-head review and explicit CTO specification approval
  - [791](https://github.com/thoth-pub/thoth/pull/791) - `THOTH-GQL-BATCH-01`: request-scoped GraphQL batching foundation implementing approved `ADR-0006`. Adds a request-scoped batch store on the GraphQL `Context` keyed by `(top-level response key, loader identity, normalized load shape, parent key)` with the three-state `NotLoaded` / `Loaded` (including `Loaded([])`) / `LoadFailed` model; a look-ahead-driven, alias-safe, synchronous set-based prefetch supporting both direct and **descendant** paths; a single isolated pinned-Juniper compatibility shim deriving the top-level response scope; and the central mutation request guard with modes `OFF` / `OBSERVE` / `ENFORCE` behind a baseline eligibility gate that leaves juniper the sole authority for parse, validation, operation-selection and input errors. **The merged state is inert**: the guard defaults to `OFF`, the store is derived-unavailable outside `ENFORCE` (so `OFF + store enabled` and `OBSERVE + store enabled` are structurally unrepresentable), no request-path overhead is added, and production request acceptance is unchanged. The foundation is adopted by **no production field**: no production child resolver, none of the 88 `MutationRoot` resolvers, and no public GraphQL contract changes — the generated SDL is byte-identical. No migration, no `schema.rs` change, no `policy.rs` change and no new dependency. Merge does **not** authorize activation: production `OBSERVE` and `ENFORCE` each require separate explicit CTO production activation approval and remain NOT AUTHORIZED, CG-13 and the monitoring/threshold blockers remain open, and `BE-02` runtime remains NOT AUTHORIZED
  - [790](https://github.com/thoth-pub/thoth/pull/790) - Record CTO approval of `ADR-0006` as merged through PR [789](https://github.com/thoth-pub/thoth/pull/789): request-scoped GraphQL batching / set-based loading, F2 response-key scoping, the central mutation request guard and its staged `OFF`/`OBSERVE`/`ENFORCE` controls are now the approved architecture decision. Documentation/control only; no architecture, runtime or production behaviour changes. `THOTH-GQL-BATCH-01` runtime implementation remains NOT AUTHORIZED, `OBSERVE` and `ENFORCE` remain NOT AUTHORIZED, CG-13 and the monitoring/threshold blockers remain open, and `BE-02` runtime remains NOT AUTHORIZED
  - [789](https://github.com/thoth-pub/thoth/pull/789) - Documentation-consistency sweep of the already-selected request-scoped GraphQL batching controls (`CHANGES REQUIRED`; no new architecture blocker, and nothing reopened - F2, the baseline-validation eligibility gate, the non-mutation fast path, effective variables, the `OFF`/`OBSERVE`/`ENFORCE` lifecycle, scoped identities, descendant prefetch, the compatibility shim, the three-state store and actual-SQL evidence are all unchanged). **Activation wording.** The final model has **two** production activations, but stale text still framed activation as `ENFORCE`-only: the risk table and classification paragraph described production feature activation as engaged "at `ENFORCE`, not at merge"; a bounding factor said behaviour "changes only at a separately authorized `ENFORCE` activation"; the HIGH-risk controls list named CTO approval only for `ENFORCE`; the task metadata mentioned only `ENFORCE` production approval; section 11.6 described the observation period as watching "the rejection event stream" after `ENFORCE`; the implementation-report requirement described only `ENFORCE` approval; and an `ADR-0006` consequence said `ENFORCE` "requires its own CTO production activation approval". All corrected: `OFF -> OBSERVE` is itself production activation because it adds live request-path behaviour (parse and operation selection on every GraphQL request, additional mutation validation and analysis, structured compatibility events), and `OBSERVE -> ENFORCE` is a second production activation because it additionally changes accepted mutation-request semantics - so `OBSERVE` is **operational/request-processing activation** while `ENFORCE` is **operational activation plus a client-visible request-acceptance change**, each requiring its own explicit CTO production activation approval under the binding rule `merge authorization != OBSERVE activation authorization != ENFORCE activation authorization`. Section 11.6 now specifies two observation stages: the `OBSERVE` window must evaluate **both** compatibility (would-be rejection events, caller investigation) **and** operational health (latency, error rate, availability, resource saturation, gate internal failure), both of which must pass before `ENFORCE`, with collision and rejection events explicitly recorded as the compatibility signal only; `ENFORCE` observation continues actual rejections, legitimate-client incidents, service health and mode/fleet correctness. The implementation-report requirement now records all three states - merge (`OFF`, store unavailable), `OBSERVE` (separate CTO authorization plus runtime-operations, monitoring/threshold and preview prerequisites) and `ENFORCE` (second separate CTO authorization plus passed compatibility **and** operational-health evidence) - while requiring that no approval identifier be committed, since under ADR-0005 live approval evidence belongs to the GitHub/release record. **Performance wording.** `ADR-0006` still carried a normative consequence that once in `OBSERVE`/`ENFORCE` the guard adds "one document parse per mutation request", contradicting the accepted blast radius; corrected to parse **and** operation selection on **every** GraphQL request, with document/schema validation, input-variable validation and duplicate-key traversal on mutations only and queries/subscriptions exiting through the non-mutation fast path after parse and selection, ordinary Juniper still parsing and validating again for requests that continue, and `OFF` adding none of it. A follow-up correction in the same sweep fixed the last remaining instance, in the task's approval section, which still said merge authorization is not production activation authorization because "the transition to guard mode `ENFORCE` requires its own explicit CTO approval" - now stating the full binding rule `merge authorization != OBSERVE activation authorization != ENFORCE activation authorization`, with `OFF -> OBSERVE` requiring explicit CTO production activation approval and `OBSERVE -> ENFORCE` requiring a second, separate one, and neither authorized by merge approval or by the other. Documentation and control only - no runtime, GraphQL, schema, migration, `Cargo` dependency, workflow or production-configuration change, no control-gap document edited, no guard mode activated, no modification of [PR #788](https://github.com/thoth-pub/thoth/pull/788), its branch or [issue #765](https://github.com/thoth-pub/thoth/issues/765), and no implementation branch created; CG-13 remains open and monitoring thresholds remain unverified, so ADR-0006 remains `PROPOSED`, `THOTH-GQL-BATCH-01` remains `DRAFT` and `HIGH` with implementation `NOT AUTHORIZED`, production `OBSERVE` and `ENFORCE` remain `NOT AUTHORIZED`, `BE-02` remains `NOT AUTHORIZED`, and the changed head requires a fresh independent exact-head review
  - [789](https://github.com/thoth-pub/thoth/pull/789) - Correct four production-control defects in the request-scoped GraphQL batching specification (`CHANGES REQUIRED`; architecture and specification only, F2 and the baseline-validation eligibility gate both retained and not reopened). **Blast radius.** The specification claimed the eligibility gate's duplicate parse/validation cost was "bounded to mutations" with an availability surface on "the mutation path"; that is false, because the gate must parse and select an operation before it can know the operation kind, so in `OBSERVE`/`ENFORCE` it necessarily touches every GraphQL request. A safe earlier non-mutation discriminator was investigated against pinned source and **found**: operation type is determinable after parse plus `get_operation` alone, which is safe because `operation_type` is a parser-level token obtained by exactly the call juniper itself makes, because the fast path only ever decides to do *less*, and because parse/selection failure exits as "no decision" — verified with **zero mismatches** against juniper's own operation typing across simple, anonymous, named-operation-selected and invalid documents of both kinds. The fast path is adopted, but it does **not** eliminate the cost: probe measurement showed document and input validation to be roughly three fifths of gate cost, leaving parse and selection on every request. The blast radius is therefore restated as three costs that must never be conflated — parse plus operation selection on **every** request; document and input validation on mutations only; duplicate-key traversal and rejection on mutations only — and all availability, latency, monitoring and threshold statements are rewritten against the **common request path**. **OBSERVE authorization.** `OBSERVE` previously proceeded under ordinary task ownership after preview acceptance; since it parses and selects for every request, validates and analyses mutations and emits structured logs, it is live production behaviour, and `release-gates.md` section 5 requires CTO approval for production activation of HIGH-risk work. Both `OFF -> OBSERVE` and `OBSERVE -> ENFORCE` now require their own explicit CTO production activation approval, neither implied by merge authorization nor by the other, with ownership tabulated and the two owners the repository cannot currently identify recorded as activation blockers rather than invented. **Rollback certainty.** The claims that rollback is "certain" and "a configuration change without a deploy" are **withdrawn**: the `clap` `Arg::env(..)` pattern proves a configuration input exists and nothing about dynamic reload, restart/deploy requirements, propagation timing, cross-replica atomicity, orchestration ownership, change authorization or rollback verification, all of which remain unmapped under open control gap **CG-13 ("Thoth runtime operations unmapped")**. Production `OBSERVE` and `ENFORCE` are now **blocked** until CG-13 or a bounded successor answers ten feature-specific questions, including propagation interval, fleet-wide mode verification and partial-fleet handling — the last being load-bearing, because store availability derives from the mode and mutation isolation depends on `ENFORCE`, so a split fleet yields inconsistent client acceptance *and* inconsistent store availability at once. **Monitoring.** "Required metrics/alerts: none … the event stream is the signal" is withdrawn: the collision stream is the compatibility signal only and says nothing about latency, error rate or availability. Service-health signals must now be verified to exist before `OBSERVE`, with explicit activation/rollback thresholds derived from existing baselines or SLOs; because this repository establishes no authoritative GraphQL latency or error-rate baseline, no number is invented and the recorded status is `BLOCKED FOR PRODUCTION ACTIVATION - MONITORING / THRESHOLDS UNVERIFIED`. `OBSERVE` must now answer two distinct questions — compatibility **and** operational — with the inference "zero collision events implies ENFORCE is safe" explicitly prohibited, and rollback triggered by **either** legitimate-client rejection or material service-health regression. Also corrects the Juniper API characterisation: `parse_document_source`, `ValidatorContext`, `visit_all_rules`, `validation::visit`, `get_operation`, `validate_input_values`, `RootNode::schema` and `RootNode::introspection_disabled` are all exported **and** `#[doc(hidden)]` on pinned 0.16.2, so "stable public API" is replaced by "public-callable, several of them doc-hidden", with no semantic-version stability promise, revalidation required before merge **and** activation, and a fail-closed requirement if any surface changes. Risk re-derived and now recorded as two separate things: **implementation task risk `HIGH`** (no `Critical` criterion is engaged, assessed criterion by criterion) and **production activation readiness `BLOCKED`**, with rollback certainty explicitly *not* offered as a reason for avoiding `Critical`. Documentation and control only - no runtime, GraphQL, schema, migration, `Cargo` dependency, workflow or production-configuration change, no control-gap document edited to imply CG-13 is resolved, no guard mode activated, no modification of [PR #788](https://github.com/thoth-pub/thoth/pull/788), its branch or [issue #765](https://github.com/thoth-pub/thoth/issues/765), and no implementation branch created; ADR-0006 remains `PROPOSED`, `THOTH-GQL-BATCH-01` remains `DRAFT` and `HIGH` with implementation `NOT AUTHORIZED`, production `OBSERVE` and `ENFORCE` remain `NOT AUTHORIZED`, `BE-02` remains `NOT AUTHORIZED`, and the changed head requires a fresh independent exact-head review
  - [789](https://github.com/thoth-pub/thoth/pull/789) - Correct a validation-ordering defect in the mutation guard specified for request-scoped GraphQL batching (`CHANGES REQUIRED`; architecture and specification only, F2 retained and not reopened, and the previously accepted `OFF`/`OBSERVE`/`ENFORCE` staging, effective-variable and scope-bearing-identity corrections all preserved). The guard was specified to run before `data.execute(&st, &ctx).await`, but `GraphQLRequest::execute` delegates to juniper's crate-level `execute`, and both `execute` and `execute_sync` perform parse, document/schema validation, operation selection and input-variable validation **inside** that call before `execute_validated_query`; a guard placed before it therefore sees documents juniper would reject before executing any resolver, so the ADR's claim that such a document "never reaches the guard at all" was **false**. This was load-bearing rather than cosmetic: a document can be simultaneously invalid under ordinary validation and shaped as a duplicate-response-key collision, in which case `ENFORCE` could **replace** juniper's canonical validation, operation-selection or input error with a guard rejection, `OBSERVE` could record a would-be rejection for traffic juniper would never execute and thereby **corrupt the very compatibility evidence** used to decide whether `ENFORCE` is safe, and directive evaluation could run on AST shapes the executor only handles post-validation. Successful ordinary juniper validation is now a **prerequisite** for duplicate-key analysis: a **baseline eligibility gate** reproduces juniper's own pipeline stages in order - `parse_document_source`, `ValidatorContext` plus `visit_all_rules` (plus the `disable_introspection` rule where `RootNode::introspection_disabled`), `get_operation`, and `validate_input_values` - and any error means the request is baseline-invalid, so the guard performs no analysis, emits no observation event, returns no guard error, and lets the ordinary Juniper path produce its canonical response. The gate is an **eligibility gate, not a replacement executor**: it introduces no externally visible validation semantics and never returns, rewrites or suppresses a validation error. Verified by compiling and running the whole gate against pinned juniper 0.16.2 using **only** public APIs - no private field access, no `unsafe`, no raw-source manipulation and no second GraphQL implementation - across eight documents that were each baseline-invalid **and** duplicate-shaped (unknown field, invalid scalar sub-selection, unknown directive, non-null variable declaring a default, missing required variable, wrong variable type, multiple operations without `operationName`, unknown `operationName`): all eight were classified ineligible while juniper produced its canonical error and executed zero resolvers, and every baseline-valid regression decision was preserved. Records the equivalence boundary honestly - no claim of byte-identical validation, only that the gate calls juniper's **own** helpers in the **same** order and treats any error as ineligible - and **withdraws the understated "one additional document parse" cost** in favour of the accurate tradeoff of duplicate parse **and** document validation **and** operation selection **and** input validation on the guarded path, deliberately not optimised away via juniper internals; `OFF` must short-circuit ahead of all of it, which is now binding rather than incidental. Adds a binding invalid-request collision test matrix requiring, for every failing gate, that the guarded path be compared against ordinary juniper with no guard present and produce no guard rejection, no `OBSERVE` event, zero resolvers/writes and a byte-comparable error and HTTP status. Also sweeps stale active-at-merge binding language: the task non-goal no longer says the guard "takes effect on the common GraphQL request path at merge" nor that CTO merge authorization covers guard behaviour, the opening boundary's unqualified "is rejected at the request boundary" is now mode-qualified and baseline-qualified, and the vague "store is unavailable whenever the guard is not applied" - which fails in `OBSERVE`, where the detector runs but the store must stay unavailable - is replaced by the explicit `loader store available => guard mode == ENFORCE` form with `OFF + store` and `OBSERVE + store` required to be structurally unrepresentable. Reconciles the decision register's dependency chain, which still ran merge straight into the `BE-02` amendment, to the full conservative ordering through preview/staging acceptance, `OBSERVE` activation authorization, the observation window, evidence review, explicit CTO `ENFORCE` activation approval and post-activation evidence before the `BE-02` amendment, recording explicitly that merge authorization is not activation authorization. Risk re-derived and remains **`HIGH`**: the correction **reduces** external-API behaviour risk and **materially improves** `OBSERVE` evidence integrity while **slightly increasing** availability risk and **increasing** guarded-path performance cost and pinned-dependency coupling (the guard now depends on juniper's request-pipeline composition, not only its executor semantics), with rollback certainty unchanged and "production feature activation at merge" remaining withdrawn as a ground. Documentation only - no runtime, GraphQL, schema, migration, `Cargo` dependency, workflow or production-configuration change, no guard mode activated, no modification of [PR #788](https://github.com/thoth-pub/thoth/pull/788), its branch or [issue #765](https://github.com/thoth-pub/thoth/issues/765), and no implementation branch created; ADR-0006 remains `PROPOSED`, `THOTH-GQL-BATCH-01` remains `DRAFT` and `HIGH` with implementation `NOT AUTHORIZED`, `BE-02` remains `NOT AUTHORIZED`, and the changed head requires a fresh independent exact-head review
  - [789](https://github.com/thoth-pub/thoth/pull/789) - Remediate the independent review of the request-scoped GraphQL batching proposal (`CHANGES REQUIRED`; architecture and specification only, F2 retained and not reopened): correct the mutation guard's rollout controls, its directive evaluation, and one remaining unscoped store identity. **Rollout.** The previous revision made the guard active on every mutation request from the merge commit behind a kill switch defaulting to enabled, and treated CTO merge authorization as authorizing that activation; that conflicts with `release-gates.md`, which prefers safe disabled-by-default post-merge behaviour, requires a merge that itself changes production behaviour to satisfy the production-ready gate first, and requires production activation of HIGH-risk work to carry preview acceptance, controlled activation, monitoring, rollback, an activation owner, an observation period and explicit CTO approval. The guard now has three modes - **`OFF`** (the default and the merged state: evaluates nothing, rejects nothing, loader store unavailable, production request acceptance unchanged), **`OBSERVE`** (evaluates exactly as `ENFORCE` would, rejects nothing, leaves the response and resolver behaviour untouched, records one bounded event per would-be rejection, store still unavailable) and **`ENFORCE`** (rejects duplicate executable top-level mutation response keys before resolver execution, zero resolvers and zero writes on rejection, store may become available) - so that `repository merge != production ENFORCE activation`, with `OFF -> OBSERVE` and `OBSERVE -> ENFORCE` as separate transitions and `ENFORCE` requiring **separate explicit CTO production activation approval** distinct from merge authorization. The fail-closed coupling is strengthened to `loader store available => guard mode == ENFORCE`, with `guard OFF + store enabled` and `guard OBSERVE + store enabled` required to be **unrepresentable** - store availability derived structurally from the single mode value rather than left to operator discipline. The previous rationale that a shadow/comparison period adds no evidence is **withdrawn**: it answered the wrong question, since the decision function is deterministic and test-covered while the open question - whether real production traffic contains documents `ENFORCE` would reject - can only be answered against real traffic, and matters precisely because the repository cannot enumerate its external API clients; `OBSERVE` is therefore recorded as the controlled compatibility pilot discharging the HIGH-risk pilot control, a non-zero would-be-rejection count **blocks** `ENFORCE` until affected callers are identified, and rollback from `ENFORCE` is `ENFORCE -> OBSERVE` or `ENFORCE -> OFF` by configuration without a deploy. **Directive evaluation.** The previous revision evaluated `@skip`/`@include` against raw request variables, but pinned Juniper applies operation variable defaults first: `execute_validated_query{,_async}` clone the request variables then `entry(name).or_insert(default)` for every operation-level default, and that `final_vars` map is what `is_excluded` sees. The guard must now build `effective_variables = operation_defaults overridden_by request_variables` from the selected operation's `variable_definitions`, mirroring `or_insert` exactly with no additional coercion; reproduced against the pinned sources by comparing guard verdicts with Juniper's **actual** resolver execution counts, the raw-variable form **over-rejected six of thirteen** documents - every omitted-but-defaulted case - while the effective map matched actual execution in **all thirteen**, including through named fragments, fragment-spread directives and inline-fragment directives. Adds binding tests for `@skip` default true omitted (accepted) and overridden false (rejected), `@include` default false omitted (accepted) and overridden true (rejected), defaulted cases through named and inline fragments, request-value precedence, and no-default regression cases - each asserted against Juniper's observed execution rather than a separately invented expectation table - plus a `BLOCKED - MUTATION GUARD CANNOT MATCH PINNED JUNIPER EXECUTABLE-SELECTION SEMANTICS` stop condition. Records a pinned-stack constraint found while proving this: `default_values_of_correct_type` rejects a non-null variable declaring a default (`Boolean! = true`), a deviation from the current GraphQL specification, so defaulted-variable tests must declare the variable nullable or they never reach the guard. **Identity.** The previous claim that no binding unscoped identity remained was false - descendant results were still required to be stored under `(loader, shape, terminal key)`; corrected to `(scope, loader, shape, terminal key)` with the semantics stated explicitly ("ordinary" means not a special ancestor-prefetched namespace, not unscoped), along with the descendant invariant, the descendant acceptance criterion, the ADR validation entry, and the failure and child-lookup identities now stated as `(scope, loader, shape, attempted key set)` and `(scope, loader, shape, parent key)`. **Observability.** Replaces the contradictory "Required logs: none / Required metrics/alerts: none / Operational runbook changes: none" with a per-component split: no production store observability before first adoption, but required structured guard events - one per would-be rejection in `OBSERVE` and per actual rejection in `ENFORCE`, carrying the mode, colliding response key and operation name only when supplied, and never the document, variables, argument values or any publisher or user payload data - plus a minimal runbook covering how to change mode, what blocks `ENFORCE`, what triggers rollback and how to verify store unavailability outside `ENFORCE`. Risk re-derived from scratch and remains **`HIGH`**, but no longer on the withdrawn grounds that activation occurs at merge: it is now grounded in production feature activation at `ENFORCE`, cross-repository API contract change, idempotency/deduplication, canonical data semantics and processing scope, with escalation applying because external clients cannot be enumerated; it meets no `Critical` criterion and rollback is certain. Also makes the seven gates explicit and non-conflatable (ADR approval, specification approval, implementation authorization, merge authorization, `OBSERVE` activation, `ENFORCE` activation, `BE-02` store adoption) and records the safest reading of the `BE-02` gates: `BE-02` implementation authorization requires `ENFORCE` completed and observed, and `BE-02` cannot reach production before `ENFORCE` because the store is structurally unavailable outside it. Documentation only - no runtime, GraphQL, schema, migration, `Cargo` dependency, workflow or production change, no mode activated, no modification of [PR #788](https://github.com/thoth-pub/thoth/pull/788), its branch or [issue #765](https://github.com/thoth-pub/thoth/issues/765), and no implementation branch created; ADR-0006 remains `PROPOSED`, `THOTH-GQL-BATCH-01` remains `DRAFT` and `HIGH` with implementation `NOT AUTHORIZED`, `BE-02` remains unauthorized, and the changed head requires a fresh independent exact-head review
  - [789](https://github.com/thoth-pub/thoth/pull/789) - Resolve the mutation execution-scope blocker in the request-scoped GraphQL batching proposal (architecture and specification only; the previously reviewed head was `BLOCKED` because a top-level GraphQL response key does **not** uniquely identify a mutation execution on the pinned Juniper). The previous entry's claim that repeated occurrences of one response key safely share one scope is **withdrawn**: reproduced against the pinned `juniper` 0.16.2 sources, `OverlappingFieldsCanBeMerged` rejects only *incompatible* repeats and never inspects directives, the sync executor calls `resolve_field` once per `Selection::Field` occurrence, the async executor pushes one future per occurrence into a `FuturesOrdered`, and the per-occurrence results are afterwards deep-merged by `merge_key_into` - not, as previously cited, replaced by `Object::add_field` - so one valid top-level response key can correspond to **several actual mutation resolver executions**, all deriving the same scope, which broke the central read-after-write isolation invariant. An execution-occurrence scope was investigated and **rejected on evidence**: `Executor::field_path` is private with no public accessor, `Executor::location()` returns only the currently executing field's own position, `ExecutionError::path()` carries response-key names with no positional component, `look_ahead()` matches the first occurrence by response name, and - decisively - for a mutation duplicating one response key whose payloads reach the terminal field through one shared fragment, two distinct mutation executions were observed in which the terminal resolver's `path()` **and** `location()` were both identical. Correcting the execution layer so compatible repeated top-level fields execute once was also **rejected as architecture expansion**, since Juniper's field collection is `pub(crate)` inside a private module, the only external interception is hand-writing the mutation root's `GraphQLValue`/`GraphQLValueAsync`, and correctness would additionally require fixing the pinned async path's concurrent (specification-deviating, pre-existing) execution of top-level mutation fields. The architecture is therefore now **two coordinated controls**: a **central mutation request guard** that, before any resolver executes, rejects a mutation operation in which one executable top-level response key occurs more than once - expanding named and inline fragments, evaluating `@skip`/`@include` against coerced variables so a definitely-excluded duplicate is still accepted, rejecting conservatively only where a condition cannot be resolved, leaving query operations and non-top-level duplicates entirely unrestricted, using only public non-`unsafe` pinned Juniper API (`parse_document_source`, `get_operation`, `Operation::operation_type`, the re-exported `Selection`/`Definition` AST, `InputValue::into_const`, `RuleError::new`, `GraphQLResponse::from_result`) without replacing `GraphQLRequest::execute`, modifying none of the 88 `MutationRoot` resolvers, making no authorization decision, and returning the repository's existing GraphQL validation-failure shape so the existing handler yields HTTP 400 with no `data` key and no new branch; **and** the loader store scoped by top-level response key, whose one-to-one correspondence with a write execution now explicitly **depends on** that guard, so the store must fail closed and be unavailable wherever the guard is not applied - "batching on, guard off" must be unrepresentable. Records the guard as a **deliberate server safety restriction** compensating for pinned Juniper's repeated mutation execution, **not** as ordinary specification-conformant validation, since it rejects some documents the GraphQL specification considers merge-compatible; the public schema and generated SDL are unchanged while the set of accepted **requests** narrows. Distinguishes storage lifetime (one GraphQL request) from reuse/execution scope (one unique executable top-level response key), corrects the too-broad multi-site reuse rule to same-scope reuse with cross-scope isolation as a separate required test, and requires failure-dispatch identity to match load identity exactly so a `LoadFailed` in one scope never poisons another. Sweeps `THOTH-GQL-BATCH-01` for stale pre-scoping binding identities - `(loader, shape)` and `(loader, shape, key)` in failure recording, dedup semantics, acceptance criteria, required tests and `BE-02` inheritance, and an acceptance criterion still asserting an unscoped store identity - and adds the mandatory guard test matrix: direct duplicate, named-fragment duplicate, inline-fragment duplicate, distinct aliases, directive and variable behaviour in both directions, query duplicates, error-shape parity against a real validation failure, guard-disabled fallback, and **measured** zero mutation resolver executions and zero database writes for every rejection, plus a shared-terminal-fragment isolation test covering exactly the case that rejected an occurrence scope. Because the guard is live on the common GraphQL request path from the merge commit, the claim "no production effect because no field adopts batching" is **explicitly withdrawn and prohibited**: rollout, rollback, observability and the expected implementation report are corrected accordingly, and a kill switch defaulting to enabled - built on the repository's established `clap` `Arg::env(..)` pattern, disabling the guard and the store together - discharges the HIGH-risk feature-flag control that is now genuinely engaged. The risk classification was re-derived from scratch against `risk-classification.md` rather than carried forward and remains `HIGH`, now additionally engaging "production feature activation" and cross-repository API contract change while meeting no `Critical` criterion and with rollback explicitly certain. Retains look-ahead-driven set-based prefetch into request-owned state with no external DataLoader, typed loader-owned normalized load shapes, the `NotLoaded`/`Loaded`/`LoadFailed` state machine and failure ownership, descendant prefetch and its four-concept contract with alias-safe `field_original_name()` traversal, the correctness-versus-compliance and terminal-versus-legacy evidence distinctions, and actual-SQL measurement through a fresh post-instrumentation pool. Documentation only - no runtime, GraphQL, schema, migration, `Cargo` dependency, workflow or production change, no modification of [PR #788](https://github.com/thoth-pub/thoth/pull/788), its branch or [issue #765](https://github.com/thoth-pub/thoth/issues/765), and no implementation branch created; ADR-0006 remains `PROPOSED` and now explicitly asks the CTO to approve the request-boundary restriction **as its own decision** rather than as a consequence of the previously recorded direction, `THOTH-GQL-BATCH-01` remains `DRAFT` and `HIGH` with implementation `NOT AUTHORIZED`, `BE-02` remains unauthorized, and the changed head requires a fresh independent exact-head review
  - [789](https://github.com/thoth-pub/thoth/pull/789) - Resolve the remaining architecture blocker in the request-scoped GraphQL batching proposal by encoding the CTO's selection of **uniform top-level-response-key scoping** (architecture and specification only; the previously reviewed head was `BLOCKED` pending a choice between an explicit query-only compliance boundary and expanding the architecture, and the CTO selected the latter): loader state is now owned by one GraphQL request but **partitioned by the current top-level GraphQL response key**, making the store identity `(top-level response key, loader identity, normalized load shape, parent key)` with dispatch-level failure state tied to the attempted key set under that same scope, applied **uniformly to query and mutation operations** so no resolver detects operation type and the raw GraphQL document is never parsed to derive scope. Storage lifetime and reuse namespace are now explicitly distinct - the store still lives on the request-scoped `Context` and still never crosses requests, while reuse is confined to one top-level response key within that request - so this is not a cross-request or independent cache. The withdrawn rule confining prefetch sites to resolvers unreachable from `MutationRoot` payload selections, the temporary hold and the open decision set are removed rather than annotated: mutation-payload fan-out such as `updatePublisher -> Publisher.contacts -> Contact.publisher -> Publisher.distributionPlatforms` is now an ordinary covered path inventoried, covered and measured by the same adoption algorithm as query paths, and no production mutation resolver is modified because correctness comes from scope isolation rather than from invalidation on write. Correctness no longer depends on the executor serializing top-level mutation fields - entries beneath one top-level response key are structurally unreachable from another even when the pinned async path drives mutation root fields concurrently through `FuturesOrdered` while the sync path is serial. The scope is derived through **one isolated pinned-Juniper compatibility shim** using `Executor::new_error(..)` plus `ExecutionError::path()`, which is evidenced side-effect-free (it builds and returns an `ExecutionError` without touching the executor's shared error collection, unlike `push_error_at`), must fail closed to the `NotLoaded` direct-query fallback rather than to a shared namespace, must be the only site in the codebase using the technique, adds no package dependency, and carries a binding revalidation obligation on any Juniper change affecting `Executor`, `ExecutionError`, field-path construction, alias/response-key handling or `new_error()`, discharged through existing dependency-change review rather than a new process. Scope keys are GraphQL **response keys** and therefore aliases, never normalized to the schema field name - `a: publishers` and `b: publishers` are two scopes - which the ADR distinguishes explicitly from selection-path matching, which continues to use `field_original_name()` because it identifies schema fields; repeated occurrences of one response key share one scope, since Juniper's executor does not merge selections while the `OverlappingFieldsCanBeMerged` validation rule rejects incompatible same-key selections before execution, so **no** source-position or AST-occurrence component is added. The accepted cost is recorded rather than hidden: the same `(loader, shape, key)` beneath two top-level query response keys is loaded once per scope, adding a number of set-based dispatches bounded by the operation's top-level structure and independent of parent list size, so `1 + N` cannot arise, and request-wide reuse across top-level fields is no longer an invariant - the foundation must prove this with a two-top-level-field query test reporting 2 dispatches rather than `N + N`. Adds invariants 20-26, updates invariants 6, 10 and 13, adds per-scope SQL-count reporting, a store collision matrix including `LoadFailed` non-poisoning across scopes, shim path-extraction/side-effect/fail-closed/isolation acceptance criteria, mutation read-after-write and cross-top-level isolation tests proven by scope isolation rather than execution order including under async interleaving, and new stop conditions; the risk classification was re-run against `risk-classification.md` rather than carried forward and remains `HIGH` (matching "changes to canonical data semantics" and "changes capable of broadening processing scope" with the escalation rules applying, while meeting no `Critical` criterion). Retains typed load shapes, the `NotLoaded`/`Loaded`/`LoadFailed` state machine, failure ownership, descendant prefetch and its four-concept contract, and actual-SQL measurement through a fresh post-instrumentation pool. Documentation only - no runtime, GraphQL, schema, migration, `Cargo` dependency, workflow or production change, no modification of [PR #788](https://github.com/thoth-pub/thoth/pull/788), its branch or [issue #765](https://github.com/thoth-pub/thoth/issues/765), no new ADR and no implementation branch created; ADR-0006 remains `PROPOSED` because the CTO's architecture direction is not approval of the resulting exact content, `THOTH-GQL-BATCH-01` remains `DRAFT` and unauthorized, `BE-02` remains unauthorized, and the changed head requires a fresh independent exact-head review
  - [789](https://github.com/thoth-pub/thoth/pull/789) - Remediate the second independent architecture review of the request-scoped GraphQL batching proposal (`CHANGES REQUIRED`; architecture and specification only, Option A/A2 still accepted and the four earlier remediations unchanged): specify **descendant prefetch**, because ADR-0006's own coverage rule identifies material fan-out paths - `QueryRoot.imprints -> Imprint.publisher -> Publisher.distributionPlatforms` and the equivalent contact routes - in which the loader-backed field is not a direct child of the list item, so a direct-child-only mechanism mandated coverage of paths it could not express and would have forced `BE-02` to invent a second batching architecture; a prefetch site may now target a direct child or a descendant, settling four concepts (selection path, terminal loader identity, terminal normalized load-shape constructor, and a key projector from the resolved list item to the terminal key, for example `Imprint -> imprint.publisher_id`) without mandating Rust type names, with recursive traversal over `LookAheadSelection::children()` matching `field_original_name()` at **every** segment rather than only the terminal field, every matching terminal selection collected across every matching intermediate branch, load shapes taken from the terminal selection rather than an ancestor, and results stored under the ordinary `(loader, shape, terminal key)` identity so indirectly prefetched entries share one namespace and satisfy each other's lookups; add a four-condition key-projection security rule so a projected key must be deterministic from data already on the resolved authorized row, must be one the GraphQL relationship would itself expose, must not bypass an intermediate authorization decision and must not reach child-protected data without its authorization context; and make explicit that descendant prefetch does **not** remediate legacy intermediate N+1 - `Imprint.publisher` and `Contact.publisher` still call `Publisher::from_id` once per parent - by defining loader-backed-field compliance and legacy intermediate resolver performance as separate evidence scopes that must be reported as distinct figures, prohibiting any claim that a whole operation is free of N+1 access unless every intermediate path was separately measured. Also **withdraw** the rule confining prefetch sites to resolvers unreachable from `MutationRoot` payload selections, which contradicted the all-material-path coverage rule now that `updatePublisher -> Publisher -> contacts -> publisher -> distributionPlatforms` is a live fan-out: operation-scoped mutation batching was investigated against the pinned sources and is **not implementable**, because a resolver cannot determine the operation type through stable public Juniper API (a field resolver's executor is always a sub-executor whose `current_type` is the field's own type, `FieldPath::Root` carries only a `SourcePosition`, `SchemaType` describes schema shape rather than the operation in flight, and `GraphQLRequest` exposes only a caller-chosen `operation_name`), while the only public route to the execution path is an off-label `Executor::new_error(..)` plus `ExecutionError::path()` and the pinned async path drives mutation root fields through `FuturesOrdered` with no `OperationType`-aware serialization at all, unlike the serial sync path; the decision is therefore escalated to the CTO as M1 (explicit query-only compliance boundary, recorded as a scoped control exception) or M2 (expand the architecture, whose only workable shape is a top-level-response-key scope applied uniformly at the cost of cross-top-level-field query reuse), with neither selected here and **no** silent exclusion written - mutation-payload paths remain correct via the `NotLoaded` fallback, are recorded as blocked rather than covered or excluded, and the hold on mutation-reachable prefetch sites is stated as temporary pending the decision. Adds invariants 17-19, rewrites invariant 10 to state the open decision, extends the adoption algorithm with operation-kind classification, nearest-suitable-site selection, key projector and intermediate authorization boundary recording, and requires the foundation's test-only schema to prove an indirect path alongside the direct one with a `BLOCKED - A2 CANNOT COVER INDIRECT FAN-OUT WITHOUT NEW EXECUTION ARCHITECTURE` stop condition. Documentation only - no runtime, GraphQL, schema, migration, `Cargo` dependency, workflow or production change, no modification of [PR #788](https://github.com/thoth-pub/thoth/pull/788), its branch or [issue #765](https://github.com/thoth-pub/thoth/issues/765), no argument added to any production field, no legacy resolver migrated and no implementation branch created; ADR-0006 remains `PROPOSED`, `THOTH-GQL-BATCH-01` remains `DRAFT` and `HIGH` risk and unauthorized, `BE-02` remains blocked, and the changed head requires a fresh independent exact-head review
  - [789](https://github.com/thoth-pub/thoth/pull/789) - Remediate the independent architecture review of the request-scoped GraphQL batching proposal (`CHANGES REQUIRED`; architecture and specification only, Option A/A2 unchanged): make the store identity `(loader identity, normalized load shape, parent key)` rather than a loader/key pair, because existing child fields such as `Publisher.imprints` and `Publisher.contacts` already take result-changing `limit`/`offset`/`filter`/`order` arguments that a pair key would collide, with typed loader-specific shapes rather than serialized argument strings, one loader-owned shape constructor shared by the prefetch site and the child lookup, explicit default normalization (Juniper's `LookAheadSelection::arguments()` reads only literal AST arguments and never applies schema defaults, while the child resolver receives the default-applied value), one dispatch per unique `(loader, shape)`, and `Unit` recorded as the shape for `BE-02`'s argument-free `Publisher.distributionPlatforms`; replace the internally inconsistent failure rule - which required a failed prefetch both to leave keys absent and to suppress the fallback that absence triggers - with a three-state store (`NotLoaded` falls back, `Loaded` including `Loaded([])` never queries, `LoadFailed` returns the error with no retry), failure recorded once per dispatch with its attempted key set, the parent list field still resolving so the error surfaces at the child field that would have failed on the direct path, and a GraphQL-visible equivalence contract covering `errors[].path`, null propagation and `extensions.type` rather than matching error text, with the one intentional difference documented rather than hidden; separate correctness from N+1 compliance and add a binding adoption coverage rule, since `Publisher` is reachable under a fan-out through `QueryRoot.imprints -> Imprint.publisher` and `Contact.publisher` as well as `QueryRoot.publishers`, so a loader-backed field with a single prefetch site can still issue one child query per parent - adopting tasks must inventory fan-out paths at their exact base, cover or explicitly escalate, and measure per path, with the `Publisher.distributionPlatforms` inventory belonging to `BE-02`; and correct the SQL-count measurement so it uses a dedicated pool constructed after the instrumentation hook rather than the process-wide `OnceLock` test pool whose connections may already be established. Also requires the test fixture to prove multi-shape and multi-site behaviour through an argument-bearing test-only field, adds invariants 13-16 and the corresponding acceptance criteria and tests, makes whole-store invalidation explicit across loaded and failed state, and records shape-normalization as the highest-consequence residual risk because it is the one failure mode the correctness fallback does not cover. Documentation only - no runtime, GraphQL, schema, migration, `Cargo` dependency, workflow or production change, no modification of [PR #788](https://github.com/thoth-pub/thoth/pull/788), its branch or [issue #765](https://github.com/thoth-pub/thoth/issues/765), no argument added to any production field or change to the approved `BE-02` API contract, and no implementation branch created; ADR-0006 remains `PROPOSED`, `THOTH-GQL-BATCH-01` remains `DRAFT` and unauthorized, `BE-02` remains blocked, and the changed head requires a fresh independent exact-head review
  - [789](https://github.com/thoth-pub/thoth/pull/789) - Propose the shared request-scoped GraphQL batching architecture (architecture and specification authoring only): add [ADR-0006](docs/engineering/decisions/ADR-0006-request-scoped-graphql-batching.md) at `PROPOSED` and the bounded `THOTH-GQL-BATCH-01` runtime implementation specification at `DRAFT`, resolving the shared architecture gap that `BE-02` escalated - `thoth-api/AGENTS.md` section 6 requires new GraphQL lists to avoid N+1 access and use set-based SQL or batched loaders, while the GraphQL `Context` carries no request-scoped state, Juniper look-ahead is unused, there is no DataLoader, and every child field queries once per parent, so no compliant repository pattern exists for a new nested field to follow. The ADR determines the concrete mechanism against the pinned stack rather than assuming a conventional loader: a deferred-dispatch DataLoader is rejected on evidence, because `juniper_codegen` 0.16.0 wraps non-async resolver bodies in `futures::future::ready(..)` so they evaluate before any future is polled, the sync resolver generated for an `async fn` field is a `panic!` while the entire GraphQL test suite executes through `juniper::execute_sync`, and neither `FuturesOrdered` selection-set driver exposes a batch-dispatch signal; the selected mechanism is instead look-ahead-driven set-based prefetch into request-scoped state, which is fully synchronous and therefore behaves identically under the production async `execute` path and the sync test path, needs no new dependency and no execution-model change. Settles where the state lives and that it is per-request with no global or cross-request cache, the closed loader-identity and typed key representation, the single-statement `WHERE key = ANY(...)` loader contract over raw canonical rows, de-duplication, deterministic partitioning, the binding distinction between "loaded, empty" and "not loaded" with a mandatory direct-query fallback that keeps a loader-backed field correct whether or not a prefetch ran, fail-closed database-error semantics that leave affected keys absent rather than empty, alias and duplicate-key behaviour, bounded batch sizes, read-after-write coherence confined structurally to prefetch sites unreachable from mutation payloads, the authorization rule that keys may come only from already-resolved, already-authorized parents so batching cannot broaden publisher scope, the unchanged transaction and connection model, explicit two-place opt-in, and SQL statement-count evidence measured at the driver for at least two parent-list sizes with wall-clock time explicitly rejected as an acceptance metric. Existing child resolvers are not migrated: legacy remediation is evidence-led follow-up work that measures before changing, and is not a prerequisite for `BE-02`. The HIGH-risk `THOTH-GQL-BATCH-01` specification proves the mechanism through a test-only GraphQL schema so no existing resolver and no public schema changes, and excludes `BE-02` runtime implementation, `DistributionPlatform`, `publisher_distribution_platform`, wholesale resolver migration, async-execution migration, new schema, new dependencies, authorization changes and production activation. Documentation only - no runtime, GraphQL, schema, migration, `Cargo` dependency, workflow or production change, no modification of [PR #788](https://github.com/thoth-pub/thoth/pull/788), its branch or [issue #765](https://github.com/thoth-pub/thoth/issues/765), and no implementation branch created; ADR-0006 is `PROPOSED` and authorizes nothing, `THOTH-GQL-BATCH-01` runtime implementation is not authorized, and `BE-02` remains blocked - it is a dependent of the batching foundation, not a dependency of it, and is unblocked neither by the ADR being drafted nor by its being approved
  - [786](https://github.com/thoth-pub/thoth/issues/786) - Adopt terminal merge evidence and non-recursive closeout as a Shared Engineering Control (`CTRL-MERGE-01`, documentation and control only): add [ADR-0005](docs/engineering/decisions/ADR-0005-terminal-merge-evidence.md) and the canonical `CTRL-MERGE-01` task record establishing that the GitHub review, CTO authorization, CI and merge record is terminal task lifecycle evidence, so no commit or pull request is created solely to restate a review identifier, an approval identifier, a merge-authorization identifier, the merge commit, the merged timestamp or a transition to "merged" or "complete"; prohibit approval-state-only commits whose sole purpose is copying existing GitHub review or approval metadata into repository files (such commits move the reviewed head and invalidate the very exact-head review that justified them); add durable-versus-transient status-writing guidance so committed documents state a durable decision and an authority condition instead of transient status such as `PENDING MERGE`, and therefore remain truthful before review, after review, before merge and after merge; define the lifecycle-evidence authority order and the exact criteria under which a material post-merge correction still requires its own bounded task and pull request. Every substantive control is retained: one bounded task per branch and pull request, approved written specifications, independent substantive review of the actual diff/tests/CI/migrations/authorization, no implementer self-approval or self-merge, exact-head review binding with head changes invalidating prior review, expected-head guarded merge, CTO merge authorization for HIGH and CRITICAL risk and wherever explicitly gated at any risk level, separately authorized production activation/deployment/migration execution/release, and missing evidence is missing work. Updates `operating-model.md`, `release-gates.md`, `docs/engineering/AGENTS.md`, the task specification, decision record and independent review templates, the AI delivery README and the decision register; historical pull requests, reviews, evidence comments, implementation reports and control records are preserved as written and are not rewritten to conform. No runtime, code, schema, migration, API, GraphQL, dissemination, `thoth-app`, CI workflow, deployment, release-automation, branch-protection, repository-settings or production change, and no programme-specific architecture change; the decision is CTO-approved under [issue #786](https://github.com/thoth-pub/thoth/issues/786) and becomes repository-authoritative when the exact approved content is reachable from `develop`, so it is not effective from an unmerged branch
  - [785](https://github.com/thoth-pub/thoth/pull/785) - Close out merged ADR-01 control state: record [PR #783](https://github.com/thoth-pub/thoth/pull/783) as merged into `develop` (merge commit `299b0eff`, 2026-08-07T10:02:34Z) with ADR-0004 and the final distribution-platform inventory as repository-authoritative, record ADR-01 as `MERGED - COMPLETE` (an evidence and architecture-decision task; not runtime implemented and not production ready), resolve CG-07, and reconcile BE-02's dependency state so its ADR-01 dependency is satisfied while BE-02 remains blocked and unauthorized pending its own approved bounded specification and explicit implementation authorization; adds the dedicated ADR-01-CLOSEOUT-01 implementation report with the classified stale-state inventory and a bounded post-merge addendum to the ADR-01 implementation report, keeping the approved content head `44e6f821`, the approval-state head `82874c2b` and the merge commit `299b0eff` distinct; documentation/control only - no ADR-0004 architecture, inventory entry, evidence claim, evidence count, evidence ledger, CG-11 or CG-13 change, and no runtime behaviour changed
  - [784](https://github.com/thoth-pub/thoth/pull/784) - Specify the ADR-01 post-merge closeout (specification only): add the bounded `ADR-01-CLOSEOUT-01` task record defining the documentation-only control closeout for ADR-01 / PR [#783](https://github.com/thoth-pub/thoth/pull/783) (merge commit `299b0eff`, merged 2026-08-07T10:02:34Z) - record PR #783 as merged, make ADR-0004 and the final platform inventory repository-authoritative, record ADR-01 as `MERGED - COMPLETE` (not runtime implemented and not production ready), resolve CG-07, remove or historicalize the active pre-merge gate language the merge left stale, and record that BE-02's ADR-01 dependency is satisfied while BE-02 remains blocked and unauthorized; the specification keeps the approved substantive content head `44e6f821`, the approval-state head `82874c2b` and the merge commit `299b0eff` distinct, mandates a classified stale-state search with global find-and-replace prohibited, preserves all 22 approved architecture invariants and the evidence counts (17 included, 10 excluded, 34 repository-verified, 21 source-owner-confirmed, 0 production-verified, 0 unknown/provisional), and requires a dedicated closeout implementation report so the immutable pre-merge implementation history is preserved; this PR specifies the closeout and does not perform it (CG-07 remains open, ADR-01 is not marked closeout-complete, BE-02 dependency state is unchanged and BE-02 remains blocked, CG-11 and CG-13 are unchanged, and no architecture, inventory, evidence, evidence-ledger, runtime, schema, migration, API, workflow, app, dissemination or production behaviour changes); approval or merge of the specification does not authorize the closeout implementation, which requires fresh explicit authorization from the then-current exact `develop` head
  - [783](https://github.com/thoth-pub/thoth/pull/783) - Decide the final distribution-platform inventory (ADR-01 implementation, documentation only): add ADR-0004 proposing the exhaustive 17-value `DistributionPlatform` inventory with no `OTHER` or fallback (16 assignable destinations plus the included-but-inactive, non-assignable, job-free `JISC_NBK`), the complete per-candidate ADR-01 evidence matrix (27 candidates: 17 included, 10 recorded exclusions; exact commits `thoth` 32123d3, `thoth-dissemination` 7a16edc0, `thoth-app` 6f826390/2632315; 0 unknown or provisional fields in included values), the linked `OAPEN_DOAB` group and duplicate-safe shared `OCLC_KBART_PUBLIC` feed serving `OCLC_KB` and `EX_LIBRIS_KB`, the BE-02 descriptor contract and future dissemination mapping, the conservative initial update/withdrawal policy, the Thoth-managed source-file invariant (recorded, not implemented), the current ProQuest EPUB-only/PDF-ISBN defect and historical/resolved Project MUSE record, and reconcile the Publisher Services control documents (ADR-0004 and the final inventory remain PROPOSED pending independent review and explicit CTO approval; BE-02 remains blocked; CG-07 remains open; no runtime, schema, migration, API, workflow, app or dissemination change)
  - [782](https://github.com/thoth-pub/thoth/pull/782) - Close out the merged ADR-01 specification amendment: record PR #781 as merged (merge commit a511e01c) with the corrected ADR-01 specification repository-authoritative, add the bounded ADR-01-SPEC-AMEND-01-CLOSEOUT-01 task record and implementation report, reconcile the Publisher Services control records and CG-07, and safely delete the obsolete local pre-amendment ADR-01 branch (control status reconciliation only; ADR-01 implementation remains separately unauthorized, ADR-0004 not started, the final platform inventory provisional and BE-02 blocked)
  - [781](https://github.com/thoth-pub/thoth/pull/781) - Add the bounded ADR-01-SPEC-AMEND-01 amendment task record and the sanitized CTO-approved ADR-01 evidence ledger (EBSCO, ProQuest and knowledge-base distribution, prepared 6 August 2026), preserving every evidence ID, source identity, stable identifier, exact claim, limitation, status, claim-to-evidence index and unresolved gap without reproducing private documents, emails, publisher lists or sensitive values
  - [780](https://github.com/thoth-pub/thoth/pull/780) - Add the bounded ADR-01 platform inventory and final architecture implementation specification, independently reviewed and explicitly CTO-approved, defining the read-only cross-repository evidence scope, the required per-destination record, the repository-verified/source-owner-confirmed/production-verified/provisional/unknown evidence classification, the settled invariants, the decisions ADR-01 must produce for BE-02 and dissemination, and the exact stop labels that fire when evidence is missing (no platform decision is included and ADR-01 implementation remains separately unauthorized)
  - [779](https://github.com/thoth-pub/thoth/pull/779) - Add the inactive BE-01 publisher package foundation: the PostgreSQL `thoth_package` enum, the non-null `publisher.subscription_package` column defaulting every existing and new publisher to `OASIS`, the closed `ThothPackage` and `PublisherCapability` Rust/GraphQL enums with stable codes, and the single code-owned exhaustive package-to-capability mapping approved by ADR-0001, with no public package or capability GraphQL surface and no OAI, Metrics, distribution or job behaviour activated
  - [778](https://github.com/thoth-pub/thoth/pull/778) - Add ADR-0003 selecting Architecture A, the repository-authoritative, manually maintained `thoth-api/src/schema.rs` Diesel schema contract, together with the THOTH-DB-CTRL-02 replacement control and a migration-chain reapply step in `run_migrations.yml` (apply, revert, reapply on the disposable database)
  - [764](https://github.com/thoth-pub/thoth/pull/764) - Add the AI-led engineering operating model, task and review templates, risk classification, release gates, and GitHub Flow controls

### Changed
  - [800](https://github.com/thoth-pub/thoth/pull/800) - Supersede the shared GraphQL batching architecture: record CTO-approved ADR-0007 (2026-08-11) selecting the conventional request-scoped **non-cached DataLoader** on the pinned Juniper 0.16.x **async** execution path — binding loader-first adoption rule, `try_load`-only access, total batch functions, set-based SQL behind the approved blocking Diesel boundary — and mark ADR-0006 `SUPERSEDED` while preserving its historical record; the pinned-Juniper duplicate-top-level-mutation-execution finding remains a separately controlled concern and the mutation guard is neither activated nor removed. Documentation and control records only: no DataLoader implementation, no production-field adoption, no GraphQL schema/SDL change, no migration, and no runtime or request-acceptance change
  - [783](https://github.com/thoth-pub/thoth/pull/783) - Record ADR-0004 distribution platform inventory approval and final inventory approval (approval-state recording only): set ADR-0004 to `APPROVED` and the final inventory to `FINAL INVENTORY APPROVED` at approved content head `44e6f821535fbee56c830dd6eda237fc6d06fbfd` under independent exact-head review `4881233664` (`APPROVED`) and explicit CTO approval `4881279067`, record ADR-01 as `IMPLEMENTATION DESIGN APPROVED` (not implemented, not production ready), and reconcile the Publisher Services control documents; no ADR-0004 decision, inventory entry, evidence matrix, evidence ledger, platform mapping or architecture content changed; the approval is a content approval that becomes repository-authoritative only on merge; CG-07 remains open pending merge and closeout, CG-11 and CG-13 are unchanged, BE-02 remains blocked, and no runtime behaviour changed
  - [781](https://github.com/thoth-pub/thoth/pull/781) - Amend the approved ADR-01 specification from the CTO-approved evidence ledger and the explicit CTO decisions of 2026-08-06: set the live status to AMENDMENT PROPOSED while preserving the historical approval for the superseded pre-amendment content only; add the binding destination-versus-adapter/feed-profile distinction and the Thoth-managed source-file invariant; record the Google Play Books single-destination naming, the confirmed EBSCO_HOST push destination with EBSCO_KB excluded as unverified, the canonical PROQUEST_EBOOK_CENTRAL destination with EX_LIBRIS_KB as a separate consumer of the shared OCLC_KBART_PUBLIC feed profile and PROQUEST_SERIALS_SOLUTIONS_KB excluded, the JISC_NBK MARC/S3 destination as included but initially inactive and non-assignable, the explicit initial exclusions, the conservative initial update and withdrawal policy, the operational ownership and configuration authority, and the manual-destination confirmation; reclassify the Project MUSE key mismatch as historical/resolved while preserving the current ProQuest EPUB-only/PDF-ISBN ordering defect; and reconcile the Publisher Services control documents and CG-07 with the amendment state (corrected content independently reviewed - review 4873802457 APPROVED - and explicitly CTO-approved at exact head 1276c70a81e73f57d833eecb0e6886bd0cabf69e; subsequently merged as a511e01c under CTO merge authorization; ADR-01 implementation remains separately unauthorized with no runtime authorization; no ADR-0004; final inventory remains provisional)
  - [780](https://github.com/thoth-pub/thoth/pull/780) - Reconcile the Publisher Services control records with the approved ADR-01 specification: record the independently reviewed, CTO-approved specification with ADR-01 READY for separately authorized implementation and ADR-01 implementation itself not authorized, record that BE-02 requires the merged ADR-01 implementation rather than the specification alone, add the programme dependency DAG and parallel thoth-app readiness track with the reserved BE-03/APP-01 GraphQL schema-pinning control, and correct the stale README claim that BE-01 implementation is blocked on THOTH-DB-CTRL-01, which contradicted the merged ADR-0003 Architecture A decision and the CG-12 RESOLVED record
  - [778](https://github.com/thoth-pub/thoth/pull/778) - Mark THOTH-DB-CTRL-01 superseded (structural-synchronizer architecture rejected; implementation PR #777 closed unmerged with no code becoming authoritative), rewrite `AGENTS.md` and `thoth-api/AGENTS.md` around the manual atomic schema workflow, and reconcile CG-12, the `thoth` repository map, BE-01 readiness, and the Publisher Services and Metrics trackers to Architecture A
  - [776](https://github.com/thoth-pub/thoth/pull/776) - Close out the merged THOTH-DB-CTRL-01 specification, recording PR #775's merge into `develop` and reconciling the Thoth Metrics tracker while retaining the implementation gate
  - [775](https://github.com/thoth-pub/thoth/pull/775) - Specify the shared THOTH-DB-CTRL-01 Diesel migration and schema-generation control, defining deterministic schema verification, safe disposable-database testing, and the remaining implementation gate
  - [774](https://github.com/thoth-pub/thoth/pull/774) - Approve the bounded BE-01 publisher package model implementation specification, defining the non-null OASIS default, exhaustive package capabilities, migration evidence, protected GraphQL boundary, and inactive rollout controls
  - [773](https://github.com/thoth-pub/thoth/pull/773) - Reconcile active engineering, Publisher Services, and Metrics control records with ADR-0001's merged state while preserving all implementation and rollout blockers
  - [772](https://github.com/thoth-pub/thoth/pull/772) - Approve the shared publisher-package capability model, excluding managed OASIS metrics collection while permitting configured, private and non-blocking OBELISK collection
  - [771](https://github.com/thoth-pub/thoth/pull/771) - Skip heavy Rust, migration, and Docker jobs for documentation-only changes while preserving protected check contexts through complete-range, fail-closed CI classification
  - [770](https://github.com/thoth-pub/thoth/pull/770) - Correct ADR-0002 approval evidence and engineering rollout controls following post-merge review, preserving programme gates and issue baselines without enabling implementation
  - [769](https://github.com/thoth-pub/thoth/pull/769) - Record CTO approval of ADR-0002, establishing separate distribution and metrics platform domains with no initial cross-domain mapping, and reconcile the Publisher Services and Metrics control records without enabling implementation
  - [767](https://github.com/thoth-pub/thoth/pull/767) - Reconcile the Publisher Services programme controls with the merged P0-01 foundation while preserving the remaining independent-review, ADR, inventory and branch-readiness gates
  - [768](https://github.com/thoth-pub/thoth/pull/768) - Finalize the Publisher Services P0-01 repository closeout: record the merged closeout PR #767's independent approval and final-head CI evidence, mark P0-01 closed as the authoritative repository record, and replace the issue-synchronization rollback with a guarded procedure (documentation and control records only)

### Removed
  - [778](https://github.com/thoth-pub/thoth/pull/778) - Remove the stale, unused root `diesel.toml`, which never parsed, did not target the canonical `thoth-api/src/schema.rs`, and was not part of any supported build, test, migration, or schema-generation command; the Diesel Rust crates and the embedded `diesel_migrations` runner are unaffected

### Fixed
  - [797](https://github.com/thoth-pub/thoth/pull/797) - `THOTH-GQL-OPS-02`: make the existing GraphQL mutation-guard mode **consumable on the production-applicable command path**, closing capability gap 1 of `THOTH-GQL-OPS-01`. The container's default command is `init`, which dispatches into the same handler as `start graphql-api` but did not register the guard-mode argument; traced through pinned `clap_builder` 4.6.0, where `ArgMatches::verify_arg` rejects an unregistered argument only under `cfg(debug_assertions)`, a **release** build therefore resolved the fallback `OFF` while a **debug** build panicked, and the `value_parser` never ran so an invalid value was silently accepted as `OFF` too. `init` now registers the same `arguments::mutation_guard_mode()` that `start graphql-api` already used, which simultaneously restores the declared `OFF` default, the `THOTH_GRAPHQL_MUTATION_GUARD_MODE` binding, the value validation and identical behaviour in both build profiles with no panic. **This is not behaviour-neutral on the `init` path and is not described as though it were:** an unset value or `OFF` is unchanged (`OFF`), but `OBSERVE` and `ENFORCE` **intentionally change** from silently ignored/`OFF` to the supplied mode, and an **invalid** value **intentionally changes** from a successful startup in `OFF` to a **startup failure**, aligning `init` with `start graphql-api` and removing a silent-misconfiguration class. Every row of that matrix is pinned by its own test in **both** the debug and release profiles, alongside tests that `init` still runs migrations **first** and still aborts startup when they fail, that all eleven pre-existing `init` arguments keep their name, environment binding and default, that `start graphql-api` is unchanged, that loader-store availability remains derived only from `ENFORCE`, and that the new startup failure leaks no secret-bearing value. **The merged state stays inert and authorizes nothing:** the default remains `OFF`, no environment is transitioned, no mode is set anywhere, no container-command override is made or proposed, migration ordering and failure behaviour are untouched, and the public GraphQL schema and generated SDL are unchanged. The specification's required **execution-time re-confirmation** of two external deployment facts — that the production service still supplies no container-command override, and that no environment currently supplies a value that would newly fail startup — was satisfied by authorized **sanitized Route B operator evidence**, confirming that both the current Production and Test GraphQL API deployments supply no container-command override and do not set the variable, so both take the `unset -> OFF` row; **AC-17 and the container-command re-confirmation are `PASS`**. A CTO-directed control review separately encountered a **control-process exception** — a reviewer read path unexpectedly exposed secret material and was stopped immediately; it supplied no acceptance evidence, and no secret detail is recorded — which the CTO/control owner explicitly disposed as **AC-18 `PASS`** in PR [797](https://github.com/thoth-pub/thoth/pull/797) comment `5251314845`, creating no standing exception or authorization for AI access to secret-bearing configuration. The implementing agent read no private deployment source, and no deployment, mode transition or Production configuration change occurred; runtime activation remains **unauthorized**. **CG-13 remains OPEN**, the runtime-operations gate remains **NOT SATISFIED**, the mode-transition runbook remains **PROVISIONAL**, `OBSERVE`, `ENFORCE` and `BE-02` runtime remain **NOT AUTHORIZED**, and `THOTH-GQL-OPS-03` and `THOTH-GQL-OPS-04` remain unimplemented with no branches

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
  - [749](https://github.com/thoth-pub/thoth/pull/749) - Correct locale code formatting in Crossref metadata output

### Changed
  - [749](https://github.com/thoth-pub/thoth/pull/749) - Remove ISBN limit in Crossref metadata output (introduced in v0.8.7)
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
  - [732](https://github.com/thoth-pub/thoth/pull/732) - Add default fields for common metadata values to `Imprint`
  - [697](https://github.com/thoth-pub/thoth/pull/697) - Migrated GraphQL API authentication to OIDC via Zitadel. Internal JWT handling has been replaced with introspection of Zitadel-issued tokens. Authorisation is now based entirely on token claims, removing the need for the internal `account` and `publisher_account` tables.
  - [697](https://github.com/thoth-pub/thoth/pull/697) - Improved and standardised backend model test coverage.
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
  - [687](https://github.com/thoth-pub/thoth/pull/687) - Upgrade database engine to PostgreSQL v17
  - [684](https://github.com/thoth-pub/thoth/pull/684) - Refactor internal work and publication APIs
  - [687](https://github.com/thoth-pub/thoth/pull/687) - Use test subdomains when building staging docker image
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
  - [676](https://github.com/thoth-pub/thoth/pull/676) - Upgrade rust to `1.85.0` in production `Dockerfile`
  - [676](https://github.com/thoth-pub/thoth/pull/676) - Upgrade `actix-cors` to v0.7.1
  - [676](https://github.com/thoth-pub/thoth/pull/676) - Upgrade `actix-http ` to v3.10.0
  - [676](https://github.com/thoth-pub/thoth/pull/676) - Upgrade `actix-web` to v4.10.2
  - [676](https://github.com/thoth-pub/thoth/pull/676) - Upgrade `chrono` to v0.4.40
  - [676](https://github.com/thoth-pub/thoth/pull/676) - Upgrade `csv` to v1.3.1
  - [676](https://github.com/thoth-pub/thoth/pull/676) - Upgrade `deadpool-redis` to v0.20.0
  - [676](https://github.com/thoth-pub/thoth/pull/676) - Upgrade `diesel` to v2.2.8
  - [676](https://github.com/thoth-pub/thoth/pull/676) - Upgrade `env_logger` to v0.11.7
  - [676](https://github.com/thoth-pub/thoth/pull/676) - Upgrade `jsonwebtoken` to v9.3.1
  - [676](https://github.com/thoth-pub/thoth/pull/676) - Upgrade `log` to v0.4.26
  - [676](https://github.com/thoth-pub/thoth/pull/676) - Upgrade `rand` to v0.9.0
  - [676](https://github.com/thoth-pub/thoth/pull/676) - Upgrade `regex` to v1.11.1
  - [676](https://github.com/thoth-pub/thoth/pull/676) - Upgrade `reqwest-middleware` to v0.4.1
  - [676](https://github.com/thoth-pub/thoth/pull/676) - Upgrade `semver` to v1.0.26
  - [676](https://github.com/thoth-pub/thoth/pull/676) - Upgrade `strum` to v0.27.1
  - [676](https://github.com/thoth-pub/thoth/pull/676) - Upgrade `tokio` to v1.44.1
  - [676](https://github.com/thoth-pub/thoth/pull/676) - Upgrade `trunk` to v0.21.9
  - [676](https://github.com/thoth-pub/thoth/pull/676) - Upgrade `uuid` to v1.16.0
  - [676](https://github.com/thoth-pub/thoth/pull/676) - Upgrade `wasm-bindgen` to v0.2.100
  - [676](https://github.com/thoth-pub/thoth/pull/676) - Upgrade `web-sys` to v0.3.77
  - [676](https://github.com/thoth-pub/thoth/pull/676) - Upgrade `xml-rs` to v0.8.25

## [[0.13.6]](https://github.com/thoth-pub/thoth/releases/tag/v0.13.6) - 2025-01-28
### Changed
  - [667](https://github.com/thoth-pub/thoth/pull/667) - Refactor binary using new submodules `commands` and `arguments`
  - [667](https://github.com/thoth-pub/thoth/pull/667) - Trigger `run\_migrations` github action when binary source changes

### Added
  - [667](https://github.com/thoth-pub/thoth/pull/667) - CLI subcommand `thoth account publishers` to modify which publisher(s) an account has access to

## [[0.13.5]](https://github.com/thoth-pub/thoth/releases/tag/v0.13.5) - 2025-01-17
### Changed
  - [665](https://github.com/thoth-pub/thoth/pull/665) - Removed unnecessary `map_or()` to comply with [`rustc 1.84.0`](https://github.com/rust-lang/rust/releases/tag/1.84.0)
  - [666](https://github.com/thoth-pub/thoth/pull/666) - Upgrade rust to `1.84.0` in production `Dockerfile`

### Added
  - [666](https://github.com/thoth-pub/thoth/pull/666) - CLI subcommand `thoth cache delete` to delete cached metadata records by specification ID

## [[0.13.4]](https://github.com/thoth-pub/thoth/releases/tag/v0.13.4) - 2024-12-11
### Added
  - [661](https://github.com/thoth-pub/thoth/pull/661) - Implement caching errors in export API

## [[0.13.3]](https://github.com/thoth-pub/thoth/releases/tag/v0.13.3) - 2024-12-02
### Changed
  - [660](https://github.com/thoth-pub/thoth/pull/660) - Upgrade rust to `1.83.0` in production `Dockerfile`
  - [660](https://github.com/thoth-pub/thoth/pull/660) - Use latest tag in development `Dockerfile`
  - [658](https://github.com/thoth-pub/thoth/pull/658) - Upgrade `rustls` to v0.23.19
  - [658](https://github.com/thoth-pub/thoth/pull/658) - Upgrade `chrono` to v0.4.38
  - [658](https://github.com/thoth-pub/thoth/pull/658) - Upgrade `uuid` to v0.11.0
  - [658](https://github.com/thoth-pub/thoth/pull/658) - Upgrade `diesel` to v2.2.5
  - [658](https://github.com/thoth-pub/thoth/pull/658) - Upgrade `paperclip` to v0.9.4
  - [658](https://github.com/thoth-pub/thoth/pull/658) - Upgrade `regex` to v1.11.1
  - [658](https://github.com/thoth-pub/thoth/pull/658) - Upgrade `xml-rs` to v0.8.23
  - [658](https://github.com/thoth-pub/thoth/pull/658) - Upgrade `futures` to v0.3.31
  - [658](https://github.com/thoth-pub/thoth/pull/658) - Upgrade `web-sys` to v0.3.72
  - [658](https://github.com/thoth-pub/thoth/pull/658) - Upgrade `wasm-bindgen` to v0.2.95
  - [658](https://github.com/thoth-pub/thoth/pull/658) - Upgrade `log` to v0.4.22
  - [658](https://github.com/thoth-pub/thoth/pull/658) - Upgrade `thiserror` to v2.0.3
  - [658](https://github.com/thoth-pub/thoth/pull/658) - Upgrade `reqwest-middleware` to v0.4.0
  - [658](https://github.com/thoth-pub/thoth/pull/658) - Upgrade `reqwest-retry` to v0.7.0
  - [658](https://github.com/thoth-pub/thoth/pull/658) - Upgrade `trunk` to v0.21.4
  - [658](https://github.com/thoth-pub/thoth/pull/658) - Upgrade `actix-identity` to v0.8.0
  - [658](https://github.com/thoth-pub/thoth/pull/658) - Upgrade `actix-session` to v0.10.1

## Removed
  - Remove redundant dependencies in thoth-app: `anyhow`, `log`, `url`

## [[0.13.2]](https://github.com/thoth-pub/thoth/releases/tag/v0.13.2) - 2024-11-26
### Added
  - [656](https://github.com/thoth-pub/thoth/pull/656) - Add database indexes to common attributes to improve performance

## [[0.13.1]](https://github.com/thoth-pub/thoth/releases/tag/v0.13.1) - 2024-11-25
### Added
  - [593](https://github.com/thoth-pub/thoth/issues/593) - Log GraphQL queries alongside request logs

## [[0.13.0]](https://github.com/thoth-pub/thoth/releases/tag/v0.13.0) - 2024-11-19
### Added
  - [651](https://github.com/thoth-pub/thoth/pull/651) - Implement Redis connection pools using `deadpool-redis`
  - [651](https://github.com/thoth-pub/thoth/pull/651) - Implement Redis caching in export API
  - [651](https://github.com/thoth-pub/thoth/pull/651) - Added `WorkLastUpdatedQuery` and `WorksLastUpdatedQuery` queries to thoth-client
  - [651](https://github.com/thoth-pub/thoth/pull/651) - Implement `Copy`, `Ord` and `PartialOrd` traits for `Timestamp`
  - [651](https://github.com/thoth-pub/thoth/pull/651) - Implement parsing from and to RFC 3339 strings for `Timestamp`
  - [651](https://github.com/thoth-pub/thoth/pull/651) - Implement `Copy` trait for `WorkType`, `WorkStatus`, `PublicationType`, `CountryCode`, `LanguageRelation`, `LanguageCode`, `LocationPlatform`, `LengthUnit`, `WeightUnit`, `CurrencyCode`, and `SeriesType`
  - [651](https://github.com/thoth-pub/thoth/pull/651) - Allow supplying `DATABASE_URL` as binary argument
  - [648](https://github.com/thoth-pub/thoth/issues/648) - Added new `LocationPlatform`, `THOTH`, for Locations where file is hosted directly by Thoth on S3.

### Changed
  - [650](https://github.com/thoth-pub/thoth/issues/650) - Allow only superusers to create/update/delete a `Location` when the `LocationPlatform` is `THOTH`.
  - [651](https://github.com/thoth-pub/thoth/pull/651) - Use Github Container registry instead of DockerHub

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
  - [628](https://github.com/thoth-pub/thoth/pull/628) - Upgrade rust to `1.82.0` in production and development `Dockerfile`

## [[0.12.12]](https://github.com/thoth-pub/thoth/releases/tag/v0.12.12) - 2024-10-15
### Fixed
  - [636](https://github.com/thoth-pub/thoth/issues/636) - OpenAPI documentation was displaying the public URL of the export API with an extra protocol

## [[0.12.11]](https://github.com/thoth-pub/thoth/releases/tag/v0.12.11) - 2024-10-14
### Changed
  - [324](https://github.com/thoth-pub/thoth/issues/324) - Make Locations editable, including the ability to change the Canonical Location for a Publication
  - [635](https://github.com/thoth-pub/thoth/pull/635) - Upgrade `reqwest` to v0.12.8
  - [635](https://github.com/thoth-pub/thoth/pull/635) - Upgrade `reqwest-middleware` to v0.3.3
  - [635](https://github.com/thoth-pub/thoth/pull/635) - Upgrade `reqwest-retry` to v0.6.1
  - [635](https://github.com/thoth-pub/thoth/pull/635) - Upgrade `paperclip` to v0.9.2

## [[0.12.10]](https://github.com/thoth-pub/thoth/releases/tag/v0.12.10) - 2024-10-01
### Added
  - [628](https://github.com/thoth-pub/thoth/pull/628) - Implement OpenAPI v3 schema in export API, served under `/openapi.json`
  - [628](https://github.com/thoth-pub/thoth/pull/628) - Added terms of service to export API

### Changed
  - [551](https://github.com/thoth-pub/thoth/issues/551) - Only include chapters in Crossref metadata output if they have DOIs
  - [628](https://github.com/thoth-pub/thoth/pull/628) - Upgrade `paperclip` to v0.9.1
  - [628](https://github.com/thoth-pub/thoth/pull/628) - Upgrade rust to `1.81.0` in production and development `Dockerfile`
  - [544](https://github.com/thoth-pub/thoth/issues/544) - Implement non-OA metadata in export outputs

### Fixed
  - [565](https://github.com/thoth-pub/thoth/issues/565) - Don't generate Crossref metadata output if no DOIs (work or chapter) are present
  - [632](https://github.com/thoth-pub/thoth/pull/632) - Add second order by clause (work\_id) to work queries for consistent ordering when multiple works share the same user-ordered field, such as publication date

## [[0.12.9]](https://github.com/thoth-pub/thoth/releases/tag/v0.12.9) - 2024-09-06
### Added
  - [595](https://github.com/thoth-pub/thoth/issues/595), [626](https://github.com/thoth-pub/thoth/pull/626) - Remove infrequently used and unused work statuses (unspecified, no longer our product, out of stock indefinitely, out of print, inactive, unknown, remaindered, recalled). Require a publication date for active, withdrawn, and superseded works in Thoth. Add a new `Superseded` work status to replace Out of Print for older editions of Works. Require a withdrawn date for Superseded works.
  - [582](https://github.com/thoth-pub/thoth/issues/582) - Add Crossmark metadata in Crossref DOI deposit when a Crossmark policy is present in the publisher record. Add Crossmark update new\_edition metadata when a book is replaced by a new edition, and withdrawal metadata when a book is withdrawn from sale.
  - [574](https://github.com/thoth-pub/thoth/issues/574), [626](https://github.com/thoth-pub/thoth/pull/626) - Add descriptions to all remaining items in schema

### Fixed
  - [548](https://github.com/thoth-pub/thoth/issues/548) - Prevent users from deleting contributors/institutions which are linked to works by other publishers

### Changed
  - [623](https://github.com/thoth-pub/thoth/pull/623) - Convert connection pool errors (`r2d2::Error`) to `ThothError`
  - [625](https://github.com/thoth-pub/thoth/pull/625) - Use relationcode 13 for physical ISBNs in ONIX 2.1 EBSCOHost output

## [[0.12.8]](https://github.com/thoth-pub/thoth/releases/tag/v0.12.8) - 2024-09-03
### Fixed
  - [622](https://github.com/thoth-pub/thoth/pull/622) - Fix bug where list of contributors in New/Edit Contribution form was truncated

## [[0.12.7]](https://github.com/thoth-pub/thoth/releases/tag/v0.12.7) - 2024-08-28
### Changed
  - [538](https://github.com/thoth-pub/thoth/issues/538) - Update Project MUSE ONIX 3.0 export to reflect new specifications provided by Project MUSE.
  - [616](https://github.com/thoth-pub/thoth/pull/616) - Removed unused constant to comply with [`rustc 1.80.0`](https://github.com/rust-lang/rust/releases/tag/1.80.0)
  - [616](https://github.com/thoth-pub/thoth/pull/616) - Upgrade `time` to v0.3.36
  - [616](https://github.com/thoth-pub/thoth/pull/616), [621](https://github.com/thoth-pub/thoth/pull/621) - Upgrade `actix-web` to v4.9
  - [616](https://github.com/thoth-pub/thoth/pull/616) - Upgrade `openssl` to v0.10.66
  - [586](https://github.com/thoth-pub/thoth/issues/586) - Upgrade `juniper` to v0.16.1
  - [586](https://github.com/thoth-pub/thoth/issues/586) - Upgrade `uuid` to v1.10.0
  - [586](https://github.com/thoth-pub/thoth/issues/586) - Upgrade `graphql_client` to v0.14.0
  - [586](https://github.com/thoth-pub/thoth/issues/586) - Upgrade `chrono` to v0.4.38
  - [586](https://github.com/thoth-pub/thoth/issues/586) - Upgrade `trunk` to v0.20.3
  - [586](https://github.com/thoth-pub/thoth/issues/586) - Upgrade `wasm-bindgen` to v0.2.93
  - [586](https://github.com/thoth-pub/thoth/issues/586) - Upgrade rust to `1.80.1` in production and development `Dockerfile`
  - [621](https://github.com/thoth-pub/thoth/pull/621) - Upgrade `clap` to v4.5.16
  - [621](https://github.com/thoth-pub/thoth/pull/621) - Upgrade `env_logger` to v0.11.5
  - [621](https://github.com/thoth-pub/thoth/pull/621) - Upgrade `futures` to v0.3.30
  - [621](https://github.com/thoth-pub/thoth/pull/621) - Upgrade `jsonwebtoken` to v9.3.0
  - [621](https://github.com/thoth-pub/thoth/pull/621) - Upgrade `lazy_static` to v1.5.0
  - [621](https://github.com/thoth-pub/thoth/pull/621) - Upgrade `regex` to v1.10.6
  - [621](https://github.com/thoth-pub/thoth/pull/621) - Upgrade `strum` to v0.26.3
  - [621](https://github.com/thoth-pub/thoth/pull/621) - Upgrade `anyhow` to v1.0.86
  - [621](https://github.com/thoth-pub/thoth/pull/621) - Upgrade `thiserror` to v1.0.63
  - [621](https://github.com/thoth-pub/thoth/pull/621) - Upgrade `semver` to v1.0.23
  - [621](https://github.com/thoth-pub/thoth/pull/621) - Replace deprecated diesel macro `sql_function` with `define_sql_function`
  - [617](https://github.com/thoth-pub/thoth/issues/617) - Update publication types to include audiobook formats (MP3 and WAV)

### Fixed
  - [610](https://github.com/thoth-pub/thoth/issues/610) - Update <WebsiteRole> code for Work Landing Page in all ONIX exports from "01" (Publisher’s corporate website) to "02" (Publisher’s website for a specified work).

### Security
  - [621](https://github.com/thoth-pub/thoth/pull/621) - Upgrade `diesel` to v2.2.3
  - [621](https://github.com/thoth-pub/thoth/pull/621) - Upgrade `diesel-derive-newtype` to v2.1.2
  - [621](https://github.com/thoth-pub/thoth/pull/621) - Upgrade `diesel_migrations` to v2.2.0

## [[0.12.6]](https://github.com/thoth-pub/thoth/releases/tag/v0.12.6) - 2024-06-17
### Fixed
  - [#513](https://github.com/thoth-pub/thoth/issues/513) - Expand DOI regex to include `+`, `[`, and `]`

### Changed
  - [607](https://github.com/thoth-pub/thoth/pull/607) - Upgrade rust to `1.79.0` in production and development `Dockerfile`

### Added
  - [607](https://github.com/thoth-pub/thoth/pull/607) - Add caching steps to Github actions

## [[0.12.5]](https://github.com/thoth-pub/thoth/releases/tag/v0.12.5) - 2024-05-07
### Changed
  - [601](https://github.com/thoth-pub/thoth/pull/601) - Upgrade rust to `1.78.0` in production and development `Dockerfile`
  - [601](https://github.com/thoth-pub/thoth/pull/601) - Upgrade `trunk` to v0.20.0
  - [601](https://github.com/thoth-pub/thoth/pull/601) - Added `-vv` option to build command in Makefile and GitHub actions

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
  - [591](https://github.com/thoth-pub/thoth/pull/591) - Upgrade rust to `1.77.2` in production and development `Dockerfile`
  - [591](https://github.com/thoth-pub/thoth/pull/591) - Added favicons to export API and GraphQL API docs
  - [591](https://github.com/thoth-pub/thoth/pull/591) - Replaced static logo files with CDN paths
  - [591](https://github.com/thoth-pub/thoth/pull/591) - Moved thoth CSS to root directory in thoth-app
  - [591](https://github.com/thoth-pub/thoth/pull/591) - Replace unnecessary pageloader CSS with an actual loader
  - [591](https://github.com/thoth-pub/thoth/pull/591) - Apply Thoth theming to rapidocs
  - [591](https://github.com/thoth-pub/thoth/pull/591) - Upgrade `graphiql` to v3.2
  - [591](https://github.com/thoth-pub/thoth/pull/591) - Upgrade `trunk` to v0.19.2
  - [591](https://github.com/thoth-pub/thoth/pull/591) - Upgrade `wasm-bindgen` to v0.2.92

### Fixed
  - [591](https://github.com/thoth-pub/thoth/pull/591) - Replaced broken logo URL in export API docs

## [[0.12.1]](https://github.com/thoth-pub/thoth/releases/tag/v0.12.1) - 2024-04-8
### Fixed
  - [589](https://github.com/thoth-pub/thoth/issues/589) - Truncation of `short_abstract` in Thoth ONIX results in Invalid UTF-8 sequences

## [[0.12.0]](https://github.com/thoth-pub/thoth/releases/tag/v0.12.0) - 2024-03-14
### Removed
  - [549](https://github.com/thoth-pub/thoth/pull/549) - Deprecate public-facing pages in Thoth APP in favour of a separate, standalone, website

### Added
  - [549](https://github.com/thoth-pub/thoth/pull/549) - Build and push staging docker images on pull requests

### Changed
 - [549](https://github.com/thoth-pub/thoth/pull/549) - Upgrade GitHub actions dependencies (`docker/setup-qemu-action@v3`, `docker/setup-buildx-action@v3`, `docker/login-action@v3`, `docker/build-push-action@v5`, `actions/checkout@v4`, `actions/setup-node@v4`)

## [[0.11.18]](https://github.com/thoth-pub/thoth/releases/tag/v0.11.18) - 2024-03-07
### Added
  - [441](https://github.com/thoth-pub/thoth/issues/441) - Implement ONIX 3.0 "Thoth" specification (i.e. complete record reflecting full data model)
  - [401](https://github.com/thoth-pub/thoth/issues/401) - Add BDS Live to list of supported platforms for JSTOR ONIX output

### Fixed
  - [475](https://github.com/thoth-pub/thoth/issues/475) - Add seconds to timestamp for Crossref metadata output
  - [571](https://github.com/thoth-pub/thoth/issues/571) - Fix overlapping URL text for Locations in Thoth Admin panel on website in Safari and Chromium browsers

### Changed
 - [578](https://github.com/thoth-pub/thoth/pull/578) - Upgrade `actix-identity` to v0.7.1
 - [578](https://github.com/thoth-pub/thoth/pull/578) - Upgrade `actix-session` to v0.9.0

### Security
  - [572](https://github.com/thoth-pub/thoth/pull/572) - Upgrade `mio` to v0.8.11

## [[0.11.17]](https://github.com/thoth-pub/thoth/releases/tag/v0.11.17) - 2024-02-29
### Changed
  - [568](https://github.com/thoth-pub/thoth/issues/568) - Allow building `thoth-app` directly from cargo, using a build script in `thoth-app-server`
  - [569](https://github.com/thoth-pub/thoth/pull/569) - Build `thoth-app` with `trunk, instead of `wasm-pack`
  - [569](https://github.com/thoth-pub/thoth/pull/569) - Optionally load `thoth-export-server` env variables from `.env` at build time
  - [569](https://github.com/thoth-pub/thoth/pull/569) - Optionally load `thoth-app` env variables from `.env` at build time
  - [569](https://github.com/thoth-pub/thoth/pull/569) - Upgrade `jsonwebtoken` to v9.2.0
  - [569](https://github.com/thoth-pub/thoth/pull/569) - Mark `jsonwebtoken` as an optional dependency, built with the `backend` feature
  - [569](https://github.com/thoth-pub/thoth/pull/569) - Upgrade `env\_logger` to v0.11.2
  - [569](https://github.com/thoth-pub/thoth/pull/569) - Upgrade `semver` to v1.0.22
  - [569](https://github.com/thoth-pub/thoth/pull/569) - Upgrade `gloo-storage` to v0.3.0
  - [569](https://github.com/thoth-pub/thoth/pull/569) - Upgrade `gloo-timers` to v0.3.0
  - [569](https://github.com/thoth-pub/thoth/pull/569) - Upgrade `strum` to v0.26.1
  - [569](https://github.com/thoth-pub/thoth/pull/569) - Upgrade`reqwest-retry` to v0.3.0
  - [499](https://github.com/thoth-pub/thoth/issues/499) - Default main\_contribution to true

### Fixed
  - [564](https://github.com/thoth-pub/thoth/issues/564) - Fix error in BibTeX not outputting editors in work types other than edited volume
  - [447](https://github.com/thoth-pub/thoth/issues/447) - Prevents Google Books Onix3 format output from Export API if Thoth record doesn't contain at least one BIC, BISAC or LCC subject code
  - [404](https://github.com/thoth-pub/thoth/issues/404) - Prevents JSTOR Onix3 format output from Export API if Thoth record doesn't contain at least one BISAC subject code

### Security
  - [569](https://github.com/thoth-pub/thoth/pull/569) - Upgrade `actix-web` to v4.5.1
  - [569](https://github.com/thoth-pub/thoth/pull/569) - Upgrade `tempfile` to v3.10.1
  - [569](https://github.com/thoth-pub/thoth/pull/569) - Upgrade `openssl` to v0.10.64
  - [569](https://github.com/thoth-pub/thoth/pull/569) - Upgrade `serde\_yaml` to v0.9.25

## [[0.11.16]](https://github.com/thoth-pub/thoth/releases/tag/v0.11.16) - 2024-02-19
### Changed
  - [561](https://github.com/thoth-pub/thoth/issues/561) - Add "Publisher Website" as a location platform
  - [553](https://github.com/thoth-pub/thoth/pull/553) - Upgrade rust to `1.76.0` in production and development `Dockerfile`
  - [305](https://github.com/thoth-pub/thoth/issues/305) - Update rust edition to 2021
  - [555](https://github.com/thoth-pub/thoth/pull/555) - Remove thoth-client's schema.json with auto-generated GraphQL schema language file on compilation

### Added
  - [244](https://github.com/thoth-pub/thoth/issues/244) - Expose GraphQL schema file in /schema.graphql
  - [503](https://github.com/thoth-pub/thoth/issues/503) - Allow reverting migrations in the CLI and check that migrations can be reverted in run-migration github action
  - [557](https://github.com/thoth-pub/thoth/pull/557) - Added github action to chech that the changelog has been updated on PRs

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
  - [531](https://github.com/thoth-pub/thoth/pull/531) - Fix bug where New Publication form for Chapter could have an ISBN pre-populated but greyed out

## [[0.11.12]](https://github.com/thoth-pub/thoth/releases/tag/v0.11.12) - 2023-12-20
### Fixed
  - [530](https://github.com/thoth-pub/thoth/pull/530) - Fix pagination offset calculation in export API
  - [530](https://github.com/thoth-pub/thoth/pull/530) - Do not allow to create more than one price in the same currency for the same publication

## [[0.11.11]](https://github.com/thoth-pub/thoth/releases/tag/v0.11.11) - 2023-12-19
### Changed
  - Upgrade rust to `1.74.1` in production and development `Dockerfile`
  - Upgrade build dependencies (npm `v10.2.5`, node `v20.10.0` and rollup `v4.9.1`) in production and development `Dockerfile`

## [[0.11.10]](https://github.com/thoth-pub/thoth/releases/tag/v0.11.10) - 2023-11-27
### Fixed
  - [524](https://github.com/thoth-pub/thoth/pull/524) - Bibliography note not being retrieved on work page

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
  - [522](https://github.com/thoth-pub/thoth/pull/522) - Improve MARC records with further recommendations

## [[0.11.8]](https://github.com/thoth-pub/thoth/releases/tag/v0.11.8) - 2023-10-31
### Changed
  - Upgrade rust to `1.73.0` in production and development `Dockerfile`
  - Upgrade build dependencies (npm `v10.2.0`, node `v18.18.2`, n `v9.2.0` and rollup `v4.1.4`) in production and development `Dockerfile`
  - [519](https://github.com/thoth-pub/thoth/issues/519) - Update ProQuest Ebrary (Ebook Central) ONIX output pricing

## [[0.11.7]](https://github.com/thoth-pub/thoth/releases/tag/v0.11.7) - 2023-10-02
### Changed
  - [508](https://github.com/thoth-pub/thoth/pull/508) - Improve MARC records with recommendations
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
  - [492](https://github.com/thoth-pub/thoth/pull/492) - Add Thoth's MARC organization code to MARC records
  - [492](https://github.com/thoth-pub/thoth/pull/492) - Add ORCID IDs to MARC
  - [492](https://github.com/thoth-pub/thoth/pull/492) - Add contact details to APP

### Changed
  - [492](https://github.com/thoth-pub/thoth/pull/492) - Streamline `thoth-export-server`'s XML module

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
  - Upgrade rust to `1.68.1` in production and development `Dockerfile`
  - Upgrade build dependencies (npm `v9.6.2`, node `v18.15.0` and rollup `v3.20.2`) in production and development `Dockerfile`
  - Upgrade `wasm-pack` to v0.11.0

## [[0.9.16]](https://github.com/thoth-pub/thoth/releases/tag/v0.9.16) - 2023-03-24
### Added
  - [#480](https://github.com/thoth-pub/thoth/pull/480) Add field to work table to track when the work or any of its relations was last updated

### Changed
  - Removed manual character checks and derivable defaults to comply with [`rustc 1.68.0`](https://github.com/rust-lang/rust/releases/tag/1.68.0)
  - [484](https://github.com/thoth-pub/thoth/pull/484) GraphQL queries: support filtering on multiple enum variants for work status and language relation, and add filtering for works last updated before/after a specified timestamp

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
  - Upgrade rust to `1.67.1` in production and development `Dockerfile`
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
  - Upgrade rust to `1.67.0` in production and development `Dockerfile`
  - Upgrade build dependencies (npm `v9.4.0`, node `v18.13.0` and rollup `v3.12.0`) in production and development `Dockerfile`
  - [#457](https://github.com/thoth-pub/thoth/issues/457) - Upgrade `juniper` to v0.15.10
  - Upgrade `diesel` to v2.0.2
  - Upgrade `uuid` to v0.8.2
  - Upgrade `paperclip` to v0.8.0
  - Upgrade `graphql_client` to v0.12.0
  - Upgrade `chrono` to v0.4.23

### Fixed
  - [#469](https://github.com/thoth-pub/thoth/issues/469) - Expand DOI regex to include square brackets

## [[0.9.5]](https://github.com/thoth-pub/thoth/releases/tag/v0.9.5) - 2023-01-17
### Changed
  - Upgrade rust to `1.66.0` in production and development `Dockerfile`
  - Upgrade build dependencies (npm `v9.2.0`, n `v9.0.1`, node `v18.12.1` and rollup `v3.7.4`) in production and development `Dockerfile`

### Fixed
  - [#463](https://github.com/thoth-pub/thoth/issues/463) - Update Thema codes to v1.5

## [[0.9.4]](https://github.com/thoth-pub/thoth/releases/tag/v0.9.4) - 2022-12-05
### Added
  - [#414](https://github.com/thoth-pub/thoth/pull/414) - Synchronise chapters' `work_status` and `publication_date` with parent's upon parent's update

## [[0.9.3]](https://github.com/thoth-pub/thoth/releases/tag/v0.9.3) - 2022-11-21
### Added
  - [#456](https://github.com/thoth-pub/thoth/pull/456) - Implement JSON output format

### Changed
  - [#455](https://github.com/thoth-pub/thoth/pull/455) - Extend CSV output format to include all available fields

## [[0.9.2]](https://github.com/thoth-pub/thoth/releases/tag/v0.9.2) - 2022-11-01
### Changed
  - [#396](https://github.com/thoth-pub/thoth/pull/396) - Expand the list of contribution types with: SoftwareBy, ResearchBy, ContributionsBy, Indexer
  - [#451](https://github.com/thoth-pub/thoth/pull/451) - Output both short and long abstracts in Crossref DOI deposit

## [[0.9.1]](https://github.com/thoth-pub/thoth/releases/tag/v0.9.1) - 2022-10-27
### Changed
  - [#449](https://github.com/thoth-pub/thoth/pull/449) - Update EBSCO Host ONIX price type code

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
  - [#390](https://github.com/thoth-pub/thoth/pull/390) - Implement OverDrive ONIX 3.0 specification

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
  - [#339](https://github.com/thoth-pub/thoth/pull/339) - Update publication types to include AZW3, DOCX and FictionBook
  - [#331](https://github.com/thoth-pub/thoth/pull/331) - Update series model to include description and CFP URL
  - Allow triggering docker action manually

### Added
  - Add code of conduct and support document to repository

## [[0.7.1]](https://github.com/thoth-pub/thoth/releases/tag/v0.7.1) - 2022-01-24
### Changed
  - Removed redundant `to_string` calls to comply with [`rustc 1.58.0`](https://github.com/rust-lang/rust/releases/tag/1.58.0)
  - [#329](https://github.com/thoth-pub/thoth/pull/329) - Update EBSCO Host ONIX pricing and contributor display logic
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
  - [#299](https://github.com/thoth-pub/thoth/pull/299) - Update Project MUSE ONIX subject output logic
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
  - Updated `yew` to [`v0.18.0`](https://github.com/yewstack/yew/releases/tag/0.18.0)
  - Updated `actix-web` to [`3.3.2`](https://github.com/actix/actix-web/releases/tag/web-v3.3.2)
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
