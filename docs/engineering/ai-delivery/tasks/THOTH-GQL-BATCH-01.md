# THOTH-GQL-BATCH-01 - Request-scoped GraphQL batching foundation

Status: DRAFT
Implementation: NOT AUTHORIZED
Programme: Shared Thoth GraphQL / Backend Architecture
Dependent programmes: Publisher Services and Distribution Configuration (first
required consumer, `BE-02`); Thoth Metrics and any other programme resolving
child fields through `thoth-api` GraphQL
Repository: `thoth-pub/thoth`
Workflow: STANDARD
Base branch: `develop`
PR target: `develop`
Programme integration branch: None
Risk: HIGH
Production activation readiness: BLOCKED - runtime-operations (CG-13) and
monitoring/threshold evidence unverified; see sections 1.1, 11 and 12
Owner: Shared backend architecture
Approved by: not yet approved
Dependencies, all required before implementation may begin: `ADR-0006` approved
and repository-authoritative; this specification approved; a freshly verified
exact `develop` base; explicit CTO implementation authorization
Runtime activation: separately controlled. Merge leaves the guard `OFF`.
`OBSERVE` requires explicit CTO production activation approval. `ENFORCE`
requires a second, separate explicit CTO production activation approval. Neither
is authorized by merge approval or by the other activation (section 11)
Target branch name: `feature/shared-architecture/graphql-batching` (**must not
exist** until implementation is authorized)

Authority condition: this record is repository-authoritative when this exact
content is reachable from the repository's authoritative integration branch.
Live review, authorization and merge evidence is the GitHub pull-request record.

This specification does not authorize implementation. It defines what
implementation must do once separately authorized. It also does not authorize
production activation: implementation, merge, `OBSERVE` activation and `ENFORCE`
activation are four separate decisions (section 11.2).

---

## 1. Objective

Give Thoth's GraphQL API a reusable, request-scoped mechanism by which a
loader-backed field reached from a list of parent objects — whether as a direct
child of those parents or as a **descendant** beneath intermediate object fields
— can be resolved with a bounded, set-based number of database statements
instead of one statement per parent, so that new nested fields can satisfy the
`thoth-api/AGENTS.md` section 6 N+1 control by following a repository pattern
rather than escalating an architecture decision.

The task delivers the foundation and proves it. It adopts the foundation in no
production field.

Four boundaries are set by `ADR-0006` and are binding on this task:

- **the central mutation request guard is in scope** (`ADR-0006` section 4.12.6),
  with its behaviour qualified by **mode**:

  ```text
  OFF:      no evaluation, no rejection, no event
  OBSERVE:  evaluate exactly as ENFORCE would, never reject
  ENFORCE:  a baseline-valid mutation operation in which one executable
            top-level response key occurs more than once is rejected
            before any resolver executes
  ```

  In every mode, a request that fails the baseline eligibility gate
  (`ADR-0006` section 4.12.6.5.3) yields **no** guard decision and **no** event;
  ordinary juniper remains the sole authority for its error. This is a **shared
  GraphQL execution prerequisite**, not a batching helper: it protects every
  mutation whether or not a loader-backed field is selected, because the defect
  it compensates — pinned Juniper executing one response key's compatible
  occurrences as several resolver invocations — causes a duplicated **write**.
  Query operations are not restricted;
- **top-level-response-key scoping is in scope** (`ADR-0006` section 4.12). The
  store is owned by one GraphQL request but partitioned by the current top-level
  GraphQL response key, giving the store identity
  `(top-level response key, loader identity, normalized load shape, parent key)`.
  The scoping rule is applied uniformly to queries and mutation payloads; no
  resolver detects operation type. This requires the single pinned-Juniper
  compatibility shim of `ADR-0006` section 4.12.8. Its mutation isolation
  guarantee **depends on** the guard above, and the store must be unavailable
  without it (`ADR-0006` section 4.12.6.6);
- **descendant prefetch is in scope** (`ADR-0006` section 4.19). The material
  fan-out paths the ADR identifies reach the loader-backed field through an
  intermediate object (`QueryRoot.imprints -> Imprint.publisher ->
  Publisher.distributionPlatforms`), so a direct-child-only mechanism would not
  satisfy the ADR's own coverage rule and would force `BE-02` to invent a second
  architecture;
- **mutation-payload fan-out is a supported path**, not an exception. The guard
  plus scope isolation make correctness independent of whether the executor
  serializes or interleaves top-level mutation fields, so no prohibition on
  mutation-reachable prefetch sites applies, and **no production mutation
  resolver is modified**.

### 1.1 Risk rationale

`HIGH`, and the classification is deliberate rather than defaulted.

`risk-classification.md` assigns HIGH to work involving "changes to canonical
data semantics" and to changes "capable of broadening processing scope". This
task adds shared request-scoped state to the GraphQL `Context` used by every
query and mutation resolver, and introduces a data-loading path whose result is
substituted for a field's direct database read. Implemented incorrectly it could
attach one parent's rows to another parent, leak state between requests, return
a stale or empty result where a live query would have returned data, or serve
rows the caller's publisher scope excludes.

The escalation rules also apply: the affected production query volume is
unknown, and the mechanism is intended for reuse by callers that do not yet
exist. Uncertainty raises the level; it does not justify lowering the controls.

Two facts do **not** reduce the classification: that no migration is required,
and that no field adopts the mechanism at merge. Both limit the *blast radius at
merge*; neither limits the correctness surface of the mechanism itself.

#### Re-evaluation after the activation-control remediation

The classification was re-run against `risk-classification.md` from scratch after
the rollout correction of section 11, rather than carried forward. The result is
**`HIGH`**.

**The material fact that changed.** Merge no longer activates the guard. The
foundation merges with `guard OFF, store unavailable, request acceptance
unchanged`, so **"production feature activation occurs at merge" is no longer a
grounds for `HIGH` and is withdrawn as an argument.** Any rationale resting on
it has been removed rather than left standing.

**Why `HIGH` still holds on the repository's actual criteria**, derived rather
than assumed:

| `risk-classification.md` criterion | Engaged by |
|---|---|
| production feature activation | engaged **first at `OBSERVE`**, where the shared GraphQL request path changes — parse and operation selection on every request, plus mutation validation, analysis and structured events — and **again at `ENFORCE`**, where accepted mutation-request semantics additionally change. Not at merge. The task specifies and delivers both activation paths, so it carries the control for both |
| cross-repository API contract change | in `ENFORCE`, the set of accepted GraphQL mutation requests changes for all clients, including clients outside this repository |
| idempotency or deduplication | the guard exists precisely because the pinned executor duplicates a mutation execution; duplicate-write prevention is the mechanism's purpose |
| changes to canonical data semantics | a data-loading path is substituted for a field's direct read |
| changes capable of broadening processing scope | shared request-scoped state on the `Context` used by every resolver |

**Escalation rules** also still apply: the affected production query volume is
unknown, and the repository cannot enumerate its external clients — which is
exactly why `OBSERVE` exists. Uncertainty raises the level; it does not justify
lowering the controls.

**Not raised to `Critical`.** Checked criterion by criterion: no destructive or
irreversible production migration, no canonical data rewrite at scale, no mass
redistribution or external publication, no security boundary affecting all
publishers (the guard only ever **rejects**, so it cannot broaden access, and
authorization is untouched), no secrets or identity-provider work, no historical
metrics recomputation, no source-of-truth cutover, and no material legal, privacy
or contractual consequence. Rollback **certainty is not claimed** as part of this
reasoning: see section 1.1's activation-readiness assessment and section 12.

**Not lowered to `Medium`.** `Medium` covers feature-flagged behaviour with
limited data effect. Two things exceed it: `ENFORCE` changes a cross-client
request contract rather than an internal behaviour, and the store's correctness
surface is a response-correctness concern rather than a limited data effect.

`HIGH` is therefore re-derived on current grounds. The HIGH-risk controls of
section 1.2 apply, with `risk-classification.md`'s "feature flag, comparison mode
or controlled pilot where possible" discharged by the **`OBSERVE` mode**
(section 11.2) rather than by a kill switch.

#### Re-evaluation after the validation-ordering correction

Re-run again against `risk-classification.md` after the eligibility-gate
correction, rather than copied forward. The result is **`HIGH`**, unchanged.

**What the correction changed, assessed factor by factor rather than assumed:**

| Factor | Effect of the eligibility gate |
|---|---|
| **external API behaviour** | **reduced risk.** The gate's purpose is to stop the guard replacing juniper's canonical validation, operation-selection and input errors. Client-visible behaviour for invalid requests is now provably preserved rather than incidentally preserved |
| **availability** | **increased, and on the common request path — not only mutations.** In `OBSERVE`/`ENFORCE` the gate parses and selects for **every** GraphQL request, and validates every mutation, so a panic or pathological input there affects GraphQL availability broadly. Bounded by: it calls juniper's own helpers rather than new parsing logic; `OFF` short-circuits entirely; the fast path limits validation to mutations |
| **performance** | **increased on the common request path.** Every request is parsed and its operation selected twice in `OBSERVE`/`ENFORCE`; mutations are additionally validated twice. Absent in `OFF` and at merge, but **not** bounded to mutations. Measured preview evidence and approved thresholds are activation prerequisites (sections 11.3, 8.3) |
| **rollback certainty** | **not established.** The code-level control is the mode value, but the production mechanism to change, propagate, verify and roll it back is unmapped under open control gap **CG-13**. Rollback timing and mechanism are **unverified**, and this task no longer claims they are certain or deploy-free (section 12) |
| **pinned-dependency coupling** | **increased.** The guard now depends on juniper's *request pipeline* composition — which helpers `execute` calls and in what order — as well as on its executor semantics. This strengthens the revalidation obligation on any juniper upgrade |
| **evidence integrity** | **materially improved.** `OBSERVE` counts can no longer be inflated by traffic juniper would never execute, so the evidence gating `ENFORCE` is now trustworthy |

**Classification.** `HIGH` still holds on the same criteria as before —
production feature activation (**first at `OBSERVE`, again at `ENFORCE`**, and
not at merge), cross-repository API contract change, idempotency/deduplication,
canonical data semantics, and changes capable of broadening processing scope —
with escalation applying because production query volume is unknown and external
clients cannot be enumerated.

The two activations are **not** equivalent, and the specification must not blur
them:

```text
OBSERVE:  operational / request-processing activation
ENFORCE:  operational activation
          + client-visible request-acceptance change
```

**"Production feature activation at merge" remains withdrawn** as a ground: the
merged state is `guard OFF, store unavailable`, and the correction reinforces
this by making `OFF` cost-free.

Not raised to `Critical`: the new work is read-only analysis on an existing
request path that only ever *declines to act*; there is no destructive migration,
canonical rewrite, mass redistribution, security-boundary change, secrets work,
metrics recomputation or source-of-truth cutover. Rollback certainty is **not**
offered as a reason for avoiding `Critical`; see the activation-readiness
assessment below. Not lowered: the added availability, performance and dependency-coupling exposure all
point the other way, and the docs-only nature of **this PR** is irrelevant — the
classification is of the future implementation and release behaviour being
specified.

#### Re-evaluation after the production-control remediation

Re-run against `risk-classification.md` from current repository evidence, not
carried forward. Two things are now recorded **separately**, because conflating
them was part of what the previous review found:

```text
Implementation task risk:      HIGH
Production activation readiness: BLOCKED
```

**Implementation task risk: `HIGH`.** Criterion by criterion — production feature
activation (at `OBSERVE` and again at `ENFORCE`, both of which this task
specifies and delivers the path for); cross-repository API contract change
(`ENFORCE` changes accepted mutation requests for all clients, enumerable or
not); idempotency/deduplication (the guard exists to prevent a duplicated write);
changes to canonical data semantics; and changes capable of broadening processing
scope. Escalation applies: production query volume is unknown, external clients
cannot be enumerated, and — newly — the eligibility gate touches **all** GraphQL
traffic once activated.

**Is `Critical` engaged?** Assessed against each `Critical` criterion rather than
by analogy:

| `Critical` criterion | Engaged? |
|---|---|
| destructive or irreversible production migration | **no** — no migration at all |
| canonical data rewrite at scale | **no** — no persistent state is created |
| mass redistribution or external publication | **no** |
| security boundary affecting all publishers | **no** — the guard only ever declines; authorization is untouched |
| secrets, credential rotation, identity-provider reconfiguration | **no** |
| historical metrics recomputation | **no** |
| cutover from one source of truth to another without immediate rollback | **no** — no source of truth changes; the direct fallback is retained throughout |
| material legal, privacy or contractual consequence | **no** |

No `Critical` criterion is engaged, so the **task** remains `HIGH`. The
escalation rules — incomplete evidence about the current system, unknown affected
production volume, uncertain rollback — are real and do apply, but they are
discharged here by making activation **blocked** rather than by inflating the
implementation classification. That is the honest reading: the code being written
is not `Critical`; **activating it without verified runtime controls would be
unsafe at any classification**, which is a gate, not a label.

**Production activation readiness: `BLOCKED`.** Activation of `OBSERVE`, and
therefore of everything downstream, is blocked until:

1. the runtime-operations prerequisite of `ADR-0006` section 7.2.4 is satisfied
   for this feature — control gap **CG-13**, "Thoth runtime operations unmapped",
   is currently open, so how the mode value is changed, propagated across
   replicas, verified and rolled back is **unknown**;
2. authoritative service-health signals and explicit activation/rollback
   thresholds are verified (`ADR-0006` section 8.3.2). This repository
   establishes no authoritative GraphQL latency or error-rate baseline, so
   thresholds cannot presently be derived, and none is invented here;
3. preview/staging performance evidence and a **rehearsed, timed** rollback exist
   (section 11.3);
4. explicit CTO production activation approval is given — separately for
   `OBSERVE` and for `ENFORCE`.

**Merge-ready is not activation-ready.** None of the above blocks merging the
foundation, provided the merged state is genuinely inert: mode `OFF`, store
unavailable, no request-path overhead, no production behaviour change.

The remainder of this section records the store-side rationale, which is
unchanged.

Factors raising the correctness surface:

- the store is shared request-scoped state on the GraphQL `Context` used by every
  query and mutation resolver;
- **every** loader lookup now depends on correct scope extraction. A wrong or
  insufficiently discriminating scope value could let entries cross top-level
  response keys, which is the mechanism preventing a mutation payload from
  serving a stale read;
- scope collision is a response-correctness risk, not merely a performance one;
- mutation read-after-write safety is now architecturally load-bearing, since
  mutation-payload fan-out is a supported path;
- the design acquires a documented compatibility coupling to the pinned Juniper
  API (`ADR-0006` sections 4.12.8, 4.12.14);
- descendant prefetch adds recursive alias-safe traversal and a key projector
  crossing an intermediate field.

Factors bounding it:

- no database migration, no `schema.rs` change, no data semantics change on disk;
- no production consumer of the **store** at initial merge, and the store is
  unavailable outside `ENFORCE`;
- **no production behaviour changes at merge at all**: the guard merges in `OFF`,
  so there is no request-path overhead and no change in request acceptance.
  Production behaviour **first** changes at a separately authorized `OBSERVE`
  activation, where the eligibility fast path and gate become active on the live
  GraphQL request path; request-acceptance semantics **additionally** change at a
  separately authorized `ENFORCE` activation (`ADR-0006` section 7.2). Neither is
  implied by merge;
- no new workspace dependency, including for the shim and the guard;
- no public GraphQL **schema** change, and the generated SDL is byte-identical.
  The set of accepted **requests** does change (`ADR-0006` section 4.12.6.7);
- the always-correct direct fallback means a scope-extraction failure degrades to
  unbatched-but-correct, not to wrong data (`ADR-0006` section 4.12.9);
- the guard fails **closed**: at most it declines a request, and a request it
  rejects executes no resolver and performs no write, so its failure mode is
  refusal rather than incorrect data. It never rejects outside `ENFORCE`, and
  never for a baseline-invalid request in any mode.

Against the framework, this sits squarely in `HIGH`: it matches "changes to
canonical data semantics" (a data-loading path substituted for a field's direct
read) and "changes capable of broadening processing scope", and the escalation
rules apply because the affected production query volume is unknown and the
mechanism is intended for callers that do not yet exist.

It does **not** meet any `Critical` criterion: there is no destructive or
irreversible production migration, no canonical data rewrite, no mass
redistribution or external publication, no security boundary change affecting all
publishers (authorization is untouched, and scoping is isolation rather than
authorization — `ADR-0006` section 4.13), no secrets or identity-provider work,
no metrics recomputation, no source-of-truth cutover, and no material legal,
privacy or contractual consequence.

`HIGH` is therefore reached on store-side grounds too, with the shim-specific
acceptance criteria of section 9. The required controls of section 1.2 are
**not** unchanged: the `OBSERVE` pilot, preview acceptance, and a separate CTO
`ENFORCE` activation approval are all newly engaged by the re-derivation
above.

### 1.2 Required HIGH-risk controls

Per `risk-classification.md` and `release-gates.md` section 1:

- approved design (`ADR-0006`) and this approved specification;
- implementation at high or maximum reasoning;
- independent cross-model review;
- failure-path and authorization tests;
- rollout and rollback plan;
- **controlled pilot**, discharging `risk-classification.md`'s "feature flag,
  comparison mode or controlled pilot where possible" for HIGH-risk work: the
  guard's **`OBSERVE`** mode (section 11.2), run for an explicit observation
  window against real traffic before any request is rejected;
- **preview/staging acceptance** of the exact implementation candidate before
  activation (section 11.3);
- explicit CTO merge authorization;
- **explicit CTO production activation approval for `OFF -> OBSERVE`**, separate
  from merge authorization. `OBSERVE` is itself production behaviour on the
  common request path, so `release-gates.md` section 5's "CTO approval for high
  or critical risk" applies to it;
- **a separate explicit CTO production activation approval for
  `OBSERVE -> ENFORCE`** (section 11.2). The binding rule is:

  ```text
  merge authorization
  !=
  OBSERVE activation authorization
  !=
  ENFORCE activation authorization
  ```

  None of the three implies either of the others: merge authorizes neither
  activation, approving `OBSERVE` does not approve `ENFORCE`, and approving
  `ENFORCE` does not retroactively approve `OBSERVE`;
- **verified runtime-operations evidence** for changing, propagating, verifying
  and rolling back the mode, per `ADR-0006` section 7.2.4 — currently blocked by
  open control gap **CG-13**;
- **verified service-health signals and approved activation thresholds** per
  `ADR-0006` section 8.3.2, before `OBSERVE`;
- a named activation owner and an observation period after activation
  (sections 11.5, 11.6);
- production activation of the **store** is gated behind `ENFORCE` and remains
  the adopting task's separately authorized concern.

---

## 2. Background and authority

Authoritative sources:

- [`ADR-0006`](../../decisions/ADR-0006-request-scoped-graphql-batching.md) -
  the governing architecture decision; every binding mechanism detail is settled
  there and is not restated in full here;
- [`thoth-api/AGENTS.md`](../../../../thoth-api/AGENTS.md) section 6 - the N+1
  control this task exists to make satisfiable;
- [`ADR-0003`](../../decisions/ADR-0003-repository-authoritative-schema-contract.md) -
  the schema contract, relevant only to confirm this task does not touch it;
- the `BE-02` specification, sections 9.2.1 and 19.1 - the escalation that
  surfaced the gap. That specification is not yet on `develop`; it is carried by
  PR [#788](https://github.com/thoth-pub/thoth/pull/788), and
  `docs/engineering/ai-delivery/tasks/BE-02.md` becomes linkable from here only
  once that pull request merges. `BE-02` is a **dependent** of this task, not a
  dependency of it;
- [PR #788](https://github.com/thoth-pub/thoth/pull/788) - the `BE-02`
  specification pull request. **This task must not modify it.**

### 2.1 Current behaviour, verified at `5a8c27b1b7c11a4f6bd26d459556468099f8c1f4`

- `Context` (`thoth-api/src/graphql/model.rs:54`) holds `db`, `user`,
  `s3_client`, `cloudfront_client` and nothing else;
- `Context` is constructed once per GraphQL HTTP request
  (`thoth-api-server/src/lib.rs:96`) and once per test
  (`thoth-api/src/model/tests.rs:147,192,197`);
- production executes GraphQL asynchronously —
  `data.execute(&st, &ctx).await` (`thoth-api-server/src/lib.rs:98`);
- the GraphQL test suite executes synchronously — `juniper::execute_sync`
  (`thoth-api/src/graphql/tests.rs:52`). **Both paths must keep working;**
- child fields query once per parent. `Publisher.imprints` calls
  `Imprint::all(..., Some(self.publisher_id), ...)`; `Publisher.contacts` calls
  `Contact::all(...)` the same way; `thoth-api/src/graphql/model.rs` contains 63
  resolver methods taking `context: &Context`;
- Juniper look-ahead is available (`juniper` 0.16.2) and unused in the
  repository; there is no DataLoader and no `dataloader` dependency;
- existing child fields already take result-changing arguments:
  `Publisher.imprints` takes `limit`, `offset`, `filter` and `order`;
  `Publisher.contacts` takes `limit`, `offset`, `order` and `contactTypes`.
  A load-shape-free cache key is therefore not a viable shared architecture
  (`ADR-0006` section 4.4.1);
- `LookAheadSelection::arguments()` reads only literal AST arguments and does
  **not** apply schema defaults (`src/executor/look_ahead.rs:577-590`), while the
  child resolver receives the default-applied value. Shape construction must
  normalize defaults explicitly (`ADR-0006` section 4.4.3);
- `LookAheadChildren::select()` / `has_child()` match on `field_name()`, which is
  **the alias when present** (`src/executor/look_ahead.rs:419-441`). Neither finds
  `a: distributionPlatforms`, and both return only the first match. Prefetch sites
  must iterate `children()` filtering on `field_original_name()`
  (`ADR-0006` section 4.15.1);
- `ThothError` derives `Error, Debug, PartialEq, Eq, Serialize, Deserialize` and
  is **not** `Clone` (`thoth-errors/src/lib.rs:11`); it maps to a `FieldError`
  carrying an `extensions.type` discriminant
  (`thoth-errors/src/lib.rs:183-207`);
- `Publisher` is reachable under a fan-out by more than its own root list query —
  at minimum `QueryRoot.publishers` (`query.rs:521`), `QueryRoot.imprints`
  (`query.rs:593`) `-> Imprint.publisher` (`model.rs:1366`), and
  `QueryRoot.contacts` (`query.rs:1868`) or `Publisher.contacts`
  `-> Contact.publisher` (`model.rs:3120`);
- there is no SQL statement-count test facility. Diesel 2.3.10 provides
  `set_default_instrumentation`, which is unused here. The ordinary test pool is
  a process-wide `OnceLock<Arc<PgPool>>` (`thoth-api/src/model/tests.rs:36,63-70`)
  whose connections may already be established, so it must **not** be the pool
  under measurement (`ADR-0006` section 8.1.1).

The implementing agent must refresh all of the above against its own exact base
before editing.

---

## 3. Explicit scope

The task must:

0. implement the **central mutation request guard** of `ADR-0006` section 4.12.6
   as its own module under `thoth-api/src/graphql/`, separate from the store,
   and invoke it from the production handler
   (`thoth-api-server/src/lib.rs`) **before** `data.execute(&st, &ctx).await`.
   This is numbered first because the store's mutation isolation guarantee
   depends on it. Binding properties, all of which use public non-`unsafe`
   juniper API only:

   - it checks its **mode first**. In `OFF` it returns immediately, before any
     parsing or validation, so `OFF` imposes no duplicate-work cost on production
     traffic (`ADR-0006` section 4.12.6.6);
   - in `OBSERVE` and `ENFORCE` it then runs the **baseline eligibility gate** of
     `ADR-0006` section 4.12.6.5.3, reproducing pinned juniper's own request
     pipeline stages in the same order, using juniper's own public helpers:

     ```text
     parse_document_source                                      (parse)
     get_operation(document, operationName)                     (operation selection)
       -> if not a mutation: EXIT, no decision, no event
     ValidatorContext::new + visit_all_rules                    (document/schema)
       + validation::visit(MultiVisitorNil.with(
           rules::disable_introspection::factory()))
         only when RootNode::introspection_disabled
     validate_input_values(variables, operation, schema)        (input variables)
     ```

     **API-surface note.** Almost all of these are exported but marked
     `#[doc(hidden)]` on pinned 0.16.2 — `parse_document_source`,
     `ValidatorContext`, `visit_all_rules`, `validation::visit`, `get_operation`,
     `validate_input_values`, `RootNode::schema` and
     `RootNode::introspection_disabled`. They are **public-callable without
     `unsafe` or private-field access**, but they carry **no** documentation-level
     or semantic-versioning stability promise, so any juniper version change
     requires revalidating the whole gate before merge **and** before activation,
     and the implementation must fail closed — failing to compile, or degrading
     to `OFF` with the store unavailable — if any surface changes semantics;

     **Selection precedes validation deliberately** (`ADR-0006` section
     4.12.6.5.4). Juniper validates then selects; the gate selects then validates,
     so a non-mutation can exit before the expensive stages. This is safe because
     the gate never surfaces an error in either order and treats any failure as
     ineligible, so every request reaching a **decision** has still passed all
     gates;

     **If any stage reports an error the request is baseline-invalid**: the guard
     performs **no** duplicate-key analysis, emits **no** observation event,
     returns **no** guard error, and lets the ordinary
     `GraphQLRequest::execute()` path produce its canonical error. The gate must
     never return, rewrite or suppress a validation error of its own — it decides
     only whether the guard is *allowed* to decide;
   - only for a baseline-valid request does it read `Operation::operation_type`
     and proceed;
   - it applies **only** to `OperationType::Mutation`. Query operations are
     returned unchanged and are never restricted (4.12.6.8);
   - it expands named fragment spreads and inline fragments before counting
     top-level occurrences, and is cycle-safe;
   - it builds the **effective variable map** before expanding occurrences,
     exactly as the pinned executor does (`ADR-0006` section 4.12.6.5.1):

     ```text
     effective_variables =
         operation_defaults
         overridden_by
         request_variables
     ```

     concretely — start from `GraphQLRequest::variables()`; read the **selected**
     operation's `variable_definitions`; for each variable declaring an
     operation-level default, preserve the request value if one was supplied and
     otherwise insert the default. This mirrors juniper's
     `all_vars.entry(name).or_insert(default)` in
     `execute_validated_query{,_async}`. `VariableDefinitions` and
     `VariableDefinition` are not publicly nameable, so they must be reached by
     field access on the `Operation` value without naming their types. Using raw
     request variables is **prohibited** and demonstrably over-rejects;
   - it evaluates `@skip`/`@include` against that **effective** map, so a
     definitely-excluded occurrence is **not** counted as executable. Juniper's
     own `is_excluded` is `pub(super)`, so this must be reimplemented on public
     API and kept behaviourally identical for literal values, variables,
     operation defaults, request overrides, multiple directives, and directives
     on fields, fragment spreads and inline fragments
     (`ADR-0006` section 4.12.6.5.2);
   - where a directive condition genuinely cannot be resolved **after applying
     operation defaults**, it counts the occurrence as executable — rejecting
     conservatively rather than admitting a possible duplicate write (4.12.6.7).
     An omitted-but-defaulted variable is **resolved**, not unresolved, and must
     never be classified as undecidable;
   - it has a three-state **mode** — `OFF`, `OBSERVE`, `ENFORCE` — per
     `ADR-0006` section 4.12.6.6, defaulting to `OFF`. In `OFF` it evaluates
     nothing; in `OBSERVE` it evaluates exactly as `ENFORCE` would but rejects
     nothing and records one observation event per would-be rejection; only in
     `ENFORCE` does it reject. **Loader store availability must be derived from
     this single value**, so that `guard OFF + store enabled` and
     `guard OBSERVE + store enabled` are unrepresentable (section 11);
   - it rejects when any executable top-level response key occurs more than once,
     returning `GraphQLResponse::from_result(Err(GraphQLError::ValidationError(..)))`
     carrying a `RuleError` with the colliding source positions, so the existing
     handler branch produces **HTTP 400** with the ordinary GraphQL
     validation-error body and no `data` key. No new handler branch and no
     one-off HTTP protocol;
   - the message must not expose loader, store or scope internals, and must not
     imply the document is invalid GraphQL;
   - it **must not** replace `GraphQLRequest::execute`, and must not make any
     authorization decision;
   - rejection precedes execution entirely: mutation resolver execution count and
     database write count are both **zero** for a rejected operation;
   - it modifies **none** of the 88 resolver methods on `MutationRoot`;
   - it emits exactly one warning-level log record per rejection, carrying the
     operation name if supplied and the colliding response key, and **never** the
     document, the variables or any argument value;
   - `juniper::ast` is a private module, so `Fragment`, `Field`, `Directive` and
     `ast::Arguments` are not publicly nameable. Named-fragment expansion must
     hold `&[Selection]` selection sets, and directive evaluation must be written
     so those types are only inferred, never named in a signature;
1. add the request-scoped store to `crate::graphql::model::Context` per
   `ADR-0006` sections 4.1-4.4, initialised empty at construction, keeping
   `Context: Sync` so the async execution path continues to compile;
2. add a focused module under `thoth-api/src/graphql/` containing the store, the
   closed loader-identity discriminant, the **typed load-shape contract**, the
   set-based loader contract, key de-duplication, deterministic result
   partitioning, failure recording, and the look-ahead-driven prefetch helper;
3. key the store by
   `(top-level response key, loader identity, normalized load shape, parent key)`
   per `ADR-0006` sections 4.4-4.4.4 and 4.12, with the load shape typed and
   loader-specific, never a serialized GraphQL argument string, and with a
   **single loader-owned shape constructor** used by both the prefetch site and
   the child lookup so the two cannot drift;
4. implement **top-level response-key partitioning** per `ADR-0006` section
   4.12, applied uniformly to query and mutation operations. Binding properties:

   - no loader entry, successful or failed, crosses top-level response-key
     scopes;
   - the scope key is the GraphQL **response key**, therefore the alias when one
     is present, and is never normalized to the schema field name (4.12.5);
   - **No** source-position or AST-occurrence component is added to the scope
     key. `ADR-0006` section 4.12.6.3 rejects that option on evidence: a nested
     resolver cannot derive a top-level execution-occurrence identity on the
     pinned stack, because `path()` and `location()` can both be identical
     across two distinct top-level mutation executions;
   - in a **query** operation, repeated occurrences of one top-level response key
     share one scope, and that is correct and required (4.12.6.8);
   - in a **mutation** operation, a duplicate executable top-level response key
     cannot arise, because the section 6.6 guard rejects the request before
     execution. The scope's one-to-one correspondence with a write execution
     **depends on** that guard (4.12.6.4);
   - **no** resolver detects operation type, and the raw GraphQL document is
     never parsed to derive scope (4.12.7). The guard is the sole place where the
     document is parsed and the operation type is read, and it runs at the
     request boundary, never inside a resolver;
   - both the prefetch site and the terminal child resolver derive the scope from
     the same helper, so they necessarily agree;
5. implement the **pinned-Juniper compatibility shim** of `ADR-0006` section
   4.12.8 as one small, isolated, separately-documented module exposing a single
   helper materially equivalent to
   `top_level_response_key(executor) -> Result<ScopeKey>`, which must:

   - use `Executor::new_error(..)` solely to materialize the current execution
     path, then read `ExecutionError::path()` and return its first segment;
   - never call `push_error` / `push_error_at`, never modify the GraphQL
     response, and never issue SQL;
   - **fail closed** when no top-level response key can be derived (4.12.9): the
     prefetch is skipped and lookups read `NotLoaded`, falling back to the direct
     query. Substituting a shared or request-global namespace is prohibited;
   - never parse the raw query string, never inspect private Juniper fields, and
     never use `unsafe`;
   - be the **only** site in the codebase using this technique — `new_error(..)`
     path extraction must not be scattered across loaders, prefetch sites or
     resolvers;
   - carry module documentation and tests that state the pinned-Juniper coupling
     and the revalidation obligation of `ADR-0006` section 4.12.14 explicitly.

   No package dependency may be added to implement the shim;
6. implement default normalization so an omitted argument and an explicitly
   supplied schema default produce the same shape, given that look-ahead does not
   apply schema defaults (`ADR-0006` section 4.4.3);
7. dispatch **once per unique
   `(top-level response key, loader identity, load shape)`** over the
   de-duplicated key set, never once per parent and never one dispatch shared
   across argument variants or across scopes (`ADR-0006` section 4.4.4);
8. implement the loader contract as a **single** set-based statement per
   dispatch, using Diesel `.eq_any(...)` (`WHERE key = ANY(...)`), returning raw
   canonical model rows rather than GraphQL objects (`ADR-0006` section 4.5);
9. implement the three-state store — `NotLoaded`, `Loaded(Vec<V>)` including
   `Loaded([])`, and `LoadFailed` — with the child-resolver behaviour table of
   `ADR-0006` section 4.7, so that only `NotLoaded` triggers the direct-query
   fallback;
10. implement failure recording per `ADR-0006` section 4.9: the parent list
   resolver still returns its parents successfully, the failure is recorded once
   per `(scope, loader, shape)` dispatch with the attempted key set, each covered
   child resolver returns the derived `FieldError`, and **no retry query is
   issued**. Failure state is keyed as
   `(scope, loader, shape, attempted key set)` and child-lookup state as
   `(scope, loader, shape, parent key)`; the scope component is never dropped
   from either, so a `LoadFailed` recorded under one scope never poisons another
   (`ADR-0006` sections 4.9.2, 4.12; invariant 31). Whole-store invalidation may
   still clear every scope at once.
   `ThothError` is not `Clone`, so retain a shareable representation;
11. implement duplicate-key handling, alias handling per shape and the
   non-destructive read (`ADR-0006` section 4.6), including that a second
   prefetch covering an already-loaded `(scope, loader, shape, key)` set issues
   no additional SQL **within that scope**;
12. enumerate child selections by iterating `children()` and filtering on
    `field_original_name()`, never via `select()` / `has_child()`
    (`ADR-0006` section 4.15.1), applying this at **every** segment of a
    descendant selection path (`ADR-0006` section 4.19.3);
13. implement **descendant prefetch** per `ADR-0006` section 4.19. A prefetch
    site must be able to target a loader-backed field reached through one or more
    intermediate object fields, and must settle all four concepts the ADR
    requires — selection path, terminal loader identity, terminal normalized
    load-shape constructor, and a key projector from the resolved list item to
    the terminal loader key. The implementation chooses its own Rust
    representation; the ADR mandates the concepts, not type names.

    Binding properties:

    - traversal is recursive or path-based over `LookAheadSelection::children()`,
      matching `field_original_name()` at every segment;
    - **every** matching terminal selection is collected, across every matching
      intermediate branch — traversal must not stop at the first match at any
      level;
    - the terminal load shape is constructed from each matching **terminal**
      selection, never from an ancestor selection;
    - projected keys are de-duplicated before dispatch;
    - results are stored under the **ordinary** terminal identity
      `(scope, loader, shape, terminal key)` — that is, the full identity of
      invariant 13, including its top-level response-key scope component. A
      separate cache namespace for indirectly prefetched entries is prohibited:
      within **one scope**, an entry prefetched from an ancestor and one
      prefetched from the terminal field's own parent list must satisfy each
      other's lookups. Across **different** scopes they are distinct entries.
      The binding rule is:

      ```text
      same scope + same loader + same shape + same terminal key
        => one shared entry, whichever prefetch site produced it
      different scope
        => distinct entries
      ```

      "Ordinary" here means *not a special ancestor-prefetched namespace*. It
      does **not** mean *unscoped*;
    - the key projector may read **only** data already present on
      already-resolved, already-authorized items, and the four conditions of
      `ADR-0006` section 4.19.4 must hold;
14. add the explicit **whole-store** invalidation entry point required by
    `ADR-0006` section 4.12.5 — clearing `Loaded` and `LoadFailed` state across
    all loaders and all shapes — unused by this task;
15. add the SQL statement-count test facility required by `ADR-0006` section
    8.1.1, using a **dedicated pool constructed after the instrumentation hook is
    installed**, under the existing exclusive database test lock, and isolating
    the measured operation's statements from setup and migration statements;
16. prove the mechanism end to end through Juniper execution using a
    **test-only** GraphQL root and object types defined under `#[cfg(test)]`,
    against existing tables, exercising real look-ahead, real set-based SQL and
    real partitioning. The fixture must include an **argument-bearing** test-only
    field so multi-shape behaviour is proven without adopting a production field
    (`ADR-0006` section 4.4.6), must support **two prefetch sites** covering
    one loader so multi-site coverage is proven (`ADR-0006` section 4.18.3), and
    must include **both a direct and an indirect (descendant) path** per section
    3.1 below;
17. update `thoth-api-server/src/lib.rs` only if `Context::new`'s signature
    changes; prefer initialising the store internally so it does not;
18. add the tests of section 10 and the query-count evidence of section 9.

### 3.1 The proof consumer

`ADR-0006` requires a real consumer to prove the abstraction. `BE-02`'s
`DistributionPlatform` model does not exist and cannot be used, and adopting an
existing production child resolver would begin the legacy migration this task
excludes.

Required approach, in order of preference:

1. **A test-only GraphQL schema.** Define, under `#[cfg(test)]`, a root query
   exposing a list of an existing parent type and a loader-backed child field
   over an existing child table (`imprint` keyed by `publisher_id` is a
   suitable, already-seeded relationship — `create_publisher` and
   `create_imprint` helpers exist in `thoth-api/src/model/tests.rs`). Build a
   test-only `RootNode` over it and execute with `juniper::execute_sync`. This
   exercises the whole mechanism with **no public schema change and no change to
   any existing resolver**, and is the required approach if it is workable.
2. If and only if approach 1 proves technically impossible, the implementing
   agent must stop and report `BLOCKED` with the exact obstacle, rather than
   adopting a production resolver on its own authority.

The proof consumer must not trigger any broader cleanup of the relationship it
uses.

The fixture must additionally provide, all within `#[cfg(test)]`:

- an **argument-bearing** loader-backed field, so load-shape behaviour is proven
  against real Juniper argument handling. The field's arguments must include at
  least one with a **schema default**, so default normalization is exercised.
  This must not be achieved by adding arguments to any production field
  (`ADR-0006` sections 4.4.6 and 4.4.5);
- **two distinct prefetch sites** covering the same loader, so `ADR-0006` section
  4.18.3's requirement — that the mechanism supports multi-site coverage without
  duplicate loading — is proven by the foundation rather than deferred to
  `BE-02`;
- **test-only mutations** returning payload objects whose selections reach a
  loader-backed field, so mutation-payload batching, read-after-write and
  cross-top-level scope isolation can be proven without touching any production
  mutation resolver (`ADR-0006` sections 4.12.10-4.12.11). At least two distinct
  test-only top-level mutation fields are required, so a single operation can
  carry two top-level response keys.

#### 3.1.1 Descendant prefetch must be proven, not only direct prefetch

Proving two **direct** parent-list prefetch sites is no longer sufficient. The
fixture must contain at least both shapes:

```text
direct path:    list -> loader-backed child
indirect path:  list -> intermediate object -> loader-backed descendant child
```

The indirect case must prove all of:

- recursive or path-based look-ahead inspection across the intermediate segment;
- alias-safe matching at **every** level, including intermediate segments;
- key projection from the already-resolved list item;
- de-duplication of the projected terminal keys;
- terminal load-shape construction from the terminal selection;
- **one** set-based terminal dispatch for the unique key set;
- correct terminal child results, equal to the direct per-parent result in order;
- **no** terminal fallback statement on the covered path;
- repeated descendant paths and repeated sites reusing already-loaded terminal
  entries rather than re-dispatching;
- the existing intermediate resolver behaviour being **unchanged** — the fixture
  must not require modifying it.

Use test-only GraphQL types or wrappers where necessary. A production resolver
must **not** be modified merely to prove this. The `imprint` -> `publisher`
relationship is a suitable model for a test-only intermediate, since
`Imprint.publisher_id` is present on the resolved row
(`thoth-api/src/model/imprint/mod.rs:44`), but the fixture must express it
through its own `#[cfg(test)]` types.

The measurement must additionally report the terminal-loader statement count
**separately** from any statements issued by the intermediate resolver, per
`ADR-0006` sections 4.19.5 and 8.2 item 9. A single combined figure is not
acceptable evidence.

**Stop condition.** If descendant prefetch cannot be expressed cleanly using
stable pinned Juniper APIs plus already-resolved data, the implementing agent
must stop and report:

```text
BLOCKED - A2 CANNOT COVER INDIRECT FAN-OUT WITHOUT NEW EXECUTION ARCHITECTURE
```

The unresolved architecture must **not** be pushed into `BE-02`.

### 3.2 Execution-path coverage

The mechanism must be proven under **both** execution paths:

- `juniper::execute_sync`, as used by the existing test suite;
- asynchronous `execute`, as used in production.

An implementation that works only under one is non-compliant. The mechanism is
synchronous by design (`ADR-0006` section 3.1 variant A2), so this should follow
from the design rather than require two implementations; it must nonetheless be
demonstrated, not assumed.

---

## 4. Non-goals

The task must not:

1. implement any part of `BE-02` at runtime;
2. add `DistributionPlatform`, `publisher_distribution_platform`, or any `BE-02`
   GraphQL field, type or enum;
3. modify PR [#788](https://github.com/thoth-pub/thoth/pull/788), its branch, or
   the `BE-02` specification;
4. modify [issue #765](https://github.com/thoth-pub/thoth/issues/765);
5. migrate, refactor or "improve" any existing child resolver;
6. perform performance work unrelated to the batching primitive;
7. migrate GraphQL execution to async-only, convert resolvers to `async fn`, or
   change the test suite off `execute_sync`;
8. add a database migration, table, column, enum, index or constraint, or edit
   `thoth-api/src/schema.rs`;
9. change the public GraphQL schema. No production type, field, argument, enum
   value or description may change, and the generated SDL must be unchanged;
10. add a workspace dependency, including any DataLoader crate;
11. change authorization policy, `thoth-api/src/policy.rs`, or any role or scope
    rule;
12. implement `ADR-0006` option C (an N+1 exception) or option D (removing the
    nested field);
13. deploy, release, run a production migration, or **activate any guard mode**.
    The guard's runtime code is delivered by this task, but its merged
    production mode is **`OFF`**: it evaluates nothing, rejects nothing and emits
    no event, so merging changes no production request behaviour (section 11.1).
    Moving to `OBSERVE`, and later to `ENFORCE`, are **separate authorized
    activations** — CTO merge authorization is **not** authorization for guard
    behaviour. No **store** feature is activated for any production field
    either, and the store is unavailable outside `ENFORCE`;
14. change CI workflows, repository settings or branch protection;
15. modify any production mutation resolver, or require mutation resolvers to
    call invalidation. Avoiding that retrofit is a principal reason `ADR-0006`
    section 4.12 partitions by scope rather than invalidating on write;
16. upgrade `juniper`, or attempt to fix Juniper's concurrent async execution of
    mutation root fields. The architecture is designed not to depend on that
    behaviour (`ADR-0006` section 4.12.10);
17. detect the GraphQL operation type at a nested resolver, or parse the raw
    GraphQL document to derive execution **scope**. The section 6.6 guard does
    parse the document and does read the operation type, but it does so once, at
    the **request boundary**, before any resolver runs, and never to derive a
    scope value. That is the only permitted document parsing;
18. use `Executor::new_error(..)` path extraction anywhere outside the single
    compatibility-shim module;
19. broaden the guard's compatibility restriction beyond `ADR-0006` section
    4.12.6.7 — in particular, it must not restrict query operations, duplicate
    response keys below the top level of a mutation, or distinct top-level
    aliases;
20. implement `ADR-0006` option F3 (correcting the execution layer so compatible
    repeated top-level fields execute once), fork or patch Juniper, or begin
    maintaining a custom GraphQL executor. That option is rejected as
    architecture expansion in `ADR-0006` section 4.12.10.1 and would require its
    own architecture decision;
21. add an execution-occurrence or source-position component to the scope key.
    `ADR-0006` section 4.12.6.3 rejects that on evidence.

---

## 5. Invariants

The implementation must preserve every invariant in `ADR-0006` section 5. In
summary, and binding here:

1. request-scoped state never crosses GraphQL requests;
2. one parent's data is never returned for another parent's key;
3. batching never broadens authorization or publisher scope — keys come only
   from already-resolved, already-authorized parents;
4. database errors fail closed, never becoming an empty or unfiltered result;
5. loader output is deterministic for a given input key set and load shape;
6. duplicate keys cause no duplicate backend fetches within one top-level
   response scope, per `(scope, loader, shape)`;
7. per-key ordering matches the owning field's contract and the direct
   per-parent result exactly;
8. no public GraphQL schema change;
9. no global, static or cross-request cache, and no singleton;
10. no stale read-after-write result within one operation;
11. a loader-backed field is correct whether or not a prefetch ran;
12. non-adopting fields are behaviourally unchanged;
13. store identity is
    `(top-level response key, loader identity, normalized load shape, parent key)`;
    argument variants never share an entry, omitted arguments normalize to the
    schema default, and entries never cross top-level response-key scopes;
14. `NotLoaded`, `Loaded([])` and `LoadFailed` are distinguishable, and only
    `NotLoaded` triggers the fallback;
15. a failed prefetch does not fail the parent list field, and issues no retry
    SQL;
16. the existence of a correctness fallback is never treated as N+1 compliance
    evidence;
17. a prefetch site may target a descendant loader-backed field, and descendant
    results are stored under the full scope-bearing terminal identity of
    invariant 13 — `(scope, loader, shape, terminal key)`. No separate cache
    namespace exists for indirectly prefetched entries, but the scope component
    is never dropped: sharing is within one scope, and distinct scopes hold
    distinct entries;
18. a descendant key projector reads only data already present on
    already-resolved, already-authorized items, and never bypasses an
    intermediate authorization decision;
19. terminal-loader compliance and legacy intermediate resolver performance are
    reported as separate evidence scopes, and neither is presented as the other;
20. loader state is owned by one request but partitioned by top-level response
    key: storage lifetime and reuse namespace are distinct, the store still never
    crosses requests, and reuse is confined to one top-level response key within
    a request;
21. the scope key is the GraphQL response key — the alias when present — and is
    never normalized to the schema field name. Selection-path matching continues
    to use `field_original_name()`, because it identifies schema fields;
22. scope derivation happens in exactly one isolated compatibility shim, which is
    side-effect-free: it adds no GraphQL error, alters no `errors[]` entry,
    changes no result data and issues no SQL;
23. scope derivation fails closed: a site that cannot derive its scope performs no
    prefetch and its lookups read `NotLoaded`. No shared or request-global
    namespace is ever substituted;
24. the scoping rule is applied uniformly to queries and mutation payloads; no
    resolver detects operation type, and mutation-payload fan-out is a supported
    covered path rather than an exception;
25. correctness does not depend on the executor serializing top-level mutation
    fields;
26. request-wide reuse across top-level response keys is **not** an invariant: the
    same `(loader, shape, key)` sub-tuple reached under two scopes is two entries
    and is loaded once per scope, and the resulting extra dispatches are bounded
    by the operation's top-level structure rather than by parent count. "Two
    prefetch sites covering the same loader in one request issue no duplicate
    SQL" is **false** as stated and must not appear; the rule is same-scope
    reuse;
27. for an accepted **mutation** operation, each executable top-level response key
    occurs exactly once, guaranteed by the section 6.6 request guard rather than
    by GraphQL validation, so a scope corresponds to exactly one mutation
    resolver execution;
28. a mutation operation rejected **by the guard** executes no mutation resolver
    and performs no database write. Guard rejection happens only in `ENFORCE`,
    and only for a **baseline-valid** request;
29. the guard makes no decision and emits no event for a request that fails the
    baseline eligibility gate (`ADR-0006` section 4.12.6.5.3), in any mode.
    Ordinary juniper remains the sole authority for parse, validation,
    operation-selection and input errors, and the guard neither replaces,
    rewrites nor suppresses any of them;
30. loader store availability is tied to **enforcement**, not to whether the
    guard evaluated:

    ```text
    loader store available  =>  guard mode == ENFORCE

    OFF     -> loader store unavailable
    OBSERVE -> loader store unavailable   (the detector runs; the store does not)
    ENFORCE -> loader store may be available
    ```

    `OFF + store available` and `OBSERVE + store available` must be structurally
    **unrepresentable**, not maintained by operator discipline over two
    independent flags.

---

## 6. Required behaviour

### 6.1 Success behaviour

- a prefetch site that opts in derives its top-level response scope, enumerates
  every requested terminal selection of the loader-backed field — whether a
  direct child of the resolved items or a descendant beneath intermediate object
  fields — derives one normalized terminal load shape per distinct variant,
  projects and de-duplicates the terminal keys from the already-resolved items,
  and issues **one** set-based statement per shape, storing the partitioned
  result under that scope;
- a descendant path resolves identically to the direct path from the terminal
  child resolver's point of view: it reads the same
  `(scope, loader, shape, terminal key)` entry and cannot tell which site
  produced it;
- a terminal child resolver derives the **same** scope as the site that
  prefetched for it, so the entry is found;
- the same key reached under a **different** top-level response key is a
  different entry, reads `NotLoaded`, and is dispatched once for that scope;
- a child resolver whose `(scope, loader, shape, parent key)` entry is `Loaded`
  returns its bucket without any database access, including when the bucket is
  empty;
- a child resolver whose entry is `NotLoaded` performs its ordinary direct query
  and returns the same result it returns today;
- two aliases of the same field with the same normalized shape return identical
  results from one dispatch; two aliases with different shapes return each
  shape's correct result from one dispatch each;
- an omitted argument and an explicitly supplied schema default resolve against
  the same entry;
- results are identical, element for element and in order, to the direct
  per-parent path.

### 6.2 Failure behaviour

- a failed set-based statement is recorded as `LoadFailed` once per
  `(scope, loader, shape)` dispatch, covering the attempted key set, and does not
  affect any other scope;
- the parent list resolver still returns its parents successfully; the prefetch
  failure does not become the parent list field's error;
- each covered child resolver returns a `FieldError` derived from the recorded
  failure, with the same `extensions.type` classification the direct path
  produces;
- **no retry query is issued** for any covered key;
- `LoadFailed` is never represented as absence and never as `Loaded([])`;
- no empty list, unfiltered result, silent per-parent retry, or swallowed error
  is permitted on any path;
- the GraphQL-visible equivalence contract of `ADR-0006` section 4.9.3 —
  `errors[].path`, null propagation, `extensions.type`, no empty substitution, no
  extra SQL — must be verified, not asserted. Any intentional difference must be
  documented explicitly rather than hidden.

### 6.3 Authorization

- the mechanism makes no authorization decision and changes no policy;
- prefetch key sets contain only keys taken from parents the request has already
  resolved and the parent resolver has already authorized;
- the loader is not a general "load by id" facility and must not accept keys
  from user input;
- if the proof consumer touches data protected at the child level rather than
  inherited from the parent, the authorization tests of section 10 are mandatory
  rather than conditional.

### 6.4 Concurrency and idempotency

- two concurrent independent GraphQL requests must not observe each other's
  store contents, and this must be proven by test rather than argued from the
  construction site;
- a repeated prefetch for an already-present `(scope, loader, shape, key)` entry
  does not re-query it, including when the repeat comes from a *different*
  prefetch site **under the same execution scope**. Two prefetch sites covering
  the same loader under *different* scopes are two entries, and the second
  dispatch there is correct and required;
- the mechanism introduces no lock, lease, claim or background job.

### 6.5 Compatibility

- no public GraphQL **schema** contract change; the generated SDL must be
  byte-identical to the base;
- **the set of accepted requests does change**, narrowly and deliberately: a
  mutation operation with a duplicate executable top-level response key is now
  rejected (`ADR-0006` section 4.12.6.7). This must be stated plainly in the
  implementation report and must not be described as a schema change, nor as
  ordinary spec-conformant validation;
- no database, migration or `schema.rs` change;
- no `thoth-client` or downstream generated-client impact. The implementing agent
  must confirm this by inspection rather than assumption, since `thoth-client`
  issues GraphQL requests;
- no dependency change;
- existing GraphQL tests must pass **unmodified**. Editing an existing test to
  accommodate the mechanism is a signal that behaviour changed and requires
  explicit justification in the implementation report.

### 6.6 Central mutation request guard

Binding behaviour, over and above the construction rules in section 3 item 0.

**Accepted, and unchanged from the base:**

```graphql
mutation {
  first:  updateA(...) { ... }
  second: updateA(...) { ... }
}
```

Distinct top-level response keys are allowed, and each executes exactly once.

**Rejected — direct duplicate:**

```graphql
mutation {
  x: updateA(...) { a }
  x: updateA(...) { b }
}
```

One response key, two executable occurrences: rejected before any resolver runs.

**Rejected — duplicate through a named fragment spread**, and **rejected —
duplicate through an inline fragment**: an equivalent duplicate introduced
indirectly is rejected identically. Fragment expansion is not optional.

**Directives and effective variables.** `@skip`, `@include` and variable-driven
conditions must be proven in both directions, evaluated against the **effective**
variable map of `ADR-0006` section 4.12.6.5.1 — operation defaults overridden by
request variables — never against raw request variables. A syntactic duplicate
that is *definitely excluded* for the concrete request must **not** be rejected;
it is not an executable occurrence. This includes a duplicate excluded by a
variable the request **omitted** but the operation **defaults**. Only where the
condition genuinely cannot be resolved after applying defaults is rejection
conservative, and that tradeoff is recorded (`ADR-0006` section 4.12.6.7); it
must not be presented as exact.

**Pinned-stack constraint on defaulted-variable documents.** Juniper's
`default_values_of_correct_type` rule rejects a **non-null** variable declaring a
default (`Argument "x" has type "Boolean!" and is not nullable, so it can't have
a default value`). Defaulted-variable tests must therefore declare the variable
**nullable** — `$skip: Boolean = true` — which the pinned stack accepts in the
non-null `if:` position. A test written as `Boolean! = true` never reaches the
guard and proves nothing.

**Resolver execution count.** For any rejected operation:

```text
mutation resolver execution count = 0
database write count             = 0
```

This is mandatory and must be **measured**. A guard that rejects only after the
first resolver has executed is non-compliant.

**Query operations.** Duplicate compatible response keys remain allowed in
queries and keep ordinary GraphQL/Juniper behaviour. The restriction must not be
broadened to queries, to non-top-level selections, or to distinct aliases.

**Failure behaviour.** HTTP status and response body follow the repository's
existing GraphQL request-validation failure convention: `is_ok()` is `false`, the
existing handler branch returns HTTP 400, and the body is the ordinary
`{"errors":[{"message","locations"}]}` shape with no `data` key. No resolver
executes, no database write occurs, and no partial mutation execution is
possible.

**Baseline eligibility.** Everything in this section applies **only** to a
request that passes the baseline eligibility gate of `ADR-0006` section
4.12.6.5.3. For a baseline-invalid request — parse failure, document/schema
validation failure, introspection-disabled failure where applicable, operation
selection failure, or input-variable validation failure — the guard makes no
decision in any mode, emits no event, and returns no error. Ordinary juniper
produces the canonical response.

**Modes.** The behaviour above is the `ENFORCE` behaviour. Binding per mode
(`ADR-0006` section 4.12.6.6):

| Mode | Parse + selection | Full eligibility gate | Duplicate analysis | Rejects | Observation event | Loader store |
|---|---|---|---|---|---|---|
| `OFF` (default, and the merged state) | **not run** | **not run** | no | no | none | **unavailable** |
| `OBSERVE` | **all requests** | mutations only | mutations only, exactly as `ENFORCE` | **no** | one per would-be rejection, baseline-valid only | **unavailable** |
| `ENFORCE` | **all requests** | mutations only | mutations only | yes, baseline-valid mutations only | one per actual rejection | may be available |

In `OBSERVE` the request continues through existing Juniper execution completely
unchanged; detection must have no effect on the response, the resolver counts or
the errors.

**Non-mutation fast path.** After parsing and selecting the operation, a
non-mutation exits immediately, before document validation and input-variable
validation (`ADR-0006` section 4.12.6.5.4). This is safe because `operation_type`
is a parser-level token obtained by exactly the call juniper itself makes, and
because exiting early makes **no** decision — no rejection, no event — which is
indistinguishable from "no collision".

**Cost, stated accurately — it is not mutation-only.** A previous revision said
the overhead was "bounded to mutations". That is **withdrawn**. Three distinct
costs, which must never be conflated:

| Cost | Applies to | Modes |
|---|---|---|
| parse + operation selection | **every GraphQL request** | `OBSERVE`, `ENFORCE` |
| document/schema + input-variable validation | mutations only, via the fast path | `OBSERVE`, `ENFORCE` |
| duplicate-key traversal, and rejection | mutations only | traversal in both; rejection in `ENFORCE` only |

So in `OBSERVE`/`ENFORCE` **every** request is parsed and its operation selected
twice — once by the gate, once inside `GraphQLRequest::execute()` — and mutations
are additionally validated twice. This must be reported as such, never as "one
extra parse" and never as bounded to mutations. `OFF` short-circuits ahead of all
of it, on every request of every kind.

**Prerequisite coupling.** The store's mutation isolation guarantee derives from
enforcement:

```text
loader store available  =>  guard mode == ENFORCE
```

Because a nested resolver cannot detect operation type, this is all-or-nothing:
outside `ENFORCE` every prefetch site performs no prefetch and every lookup reads
`NotLoaded`, so every path takes its always-correct direct fallback. The
implementation must make both `guard OFF + store enabled` and
`guard OBSERVE + store enabled` **unrepresentable** — store availability derived
from the single mode value, not a second independent setting that operators are
merely told to keep consistent.

---

## 7. Data and migration requirements

Migration required: **NO**

No table, column, enum, index, constraint or trigger is added or altered.
`thoth-api/src/schema.rs` and `thoth-api/migrations/` must be byte-identical to
the base. Per `ADR-0003` section 4.3 and `thoth-api/AGENTS.md` section 4, the
absence of a `schema.rs` change is recorded here as an explicit reviewed
conclusion, not an omission.

Stop condition: if implementation finds a migration is required, stop and report
`BLOCKED` (section 13).

---

## 8. Observability and operations

**Corrected during remediation.** This section previously read "Required logs:
none / Required metrics/alerts: none / Operational runbook changes: none", which
contradicted the guard's own requirement to emit a record per rejection and to
produce observation evidence. The two components have different obligations and
must be stated separately.

### 8.1 Loader store — no production observability required

No production field adopts the store in this task, and the store is unavailable
outside `ENFORCE`, so there is nothing to observe in production.

- required logs: **none**;
- required metrics/alerts: **none** before first adoption;
- query-count observation remains **test and preview** evidence (section 10).

Adding production instrumentation for a mechanism no production field uses would
be unmeasurable noise. Instrumentation for adopted fields is the adopting task's
concern.

### 8.2 Mutation guard — production observability is required

`OBSERVE` and `ENFORCE` both sit on the common request path, and `OBSERVE` exists
precisely to produce evidence, so "required logs: none" is **not** an acceptable
statement for the guard.

| Mode | Required event |
|---|---|
| `OFF` | none — the guard evaluates nothing |
| `OBSERVE` | one structured event per **would-be** rejection, **baseline-valid requests only** |
| `ENFORCE` | one structured event per **actual** rejection |

A request failing the baseline eligibility gate emits **no** guard event in any
mode. This is a correctness requirement, not a volume preference: an event for
traffic juniper would never execute would corrupt the compatibility evidence the
`OBSERVE` window exists to produce (section 11.4).

Each event must carry:

- the guard **mode**, so `OBSERVE` and `ENFORCE` evidence can never be conflated;
- the **colliding response key**;
- the **operation name**, only when the request supplied one.

Each event must **never** carry:

- the full GraphQL document;
- variables;
- mutation argument values;
- any publisher or user payload data.

Mutation arguments carry user and publisher data, so this is a privacy
requirement, not a verbosity preference.

**These events are the compatibility signal only.** A previous revision said
"Required metrics/alerts: none created by this task. The event stream is the
signal." That is **withdrawn**: the collision stream says nothing about the
latency, error-rate or availability effects the eligibility gate introduces on
the common request path.

### 8.3 Service-health signals and activation thresholds

`release-gates.md` section 5 requires "monitoring and alert thresholds" for
production activation, and section 8 requires observation to "monitor
correctness, errors, latency and backlog". Both apply, because `OBSERVE` and
`ENFORCE` add work to every GraphQL request (section 6.6).

**This task creates no dashboard, telemetry pipeline or alerting rule.** It
requires that authoritative signals be **identified and verified to exist**
before activation. Before `OBSERVE` may be authorized, there must be a verified
way to observe:

- GraphQL/API request **latency**, at an agreed percentile;
- server/GraphQL **error rate**;
- service **availability** / HTTP failure rate as appropriate;
- **resource saturation**, where already available and material to the duplicate
  validation work;
- guard/eligibility-path **panic or internal failure**, where distinguishable.

**Thresholds** must be derived from existing operational baselines or SLOs and
approved before activation, in a form such as:

```text
OBSERVE must not proceed, or must roll back, if p95 latency exceeds the
approved baseline threshold.

OBSERVE must roll back if the server error rate exceeds the approved
release threshold.

ENFORCE must not proceed while would-be legitimate-client collisions
remain unresolved.
```

**No numeric value is stated here, deliberately.** This repository establishes no
authoritative production latency, error-rate or SLO baseline for the `thoth`
GraphQL API. If no authoritative metrics, SLO or threshold evidence can be found,
the correct recorded status is:

```text
BLOCKED FOR PRODUCTION ACTIVATION - MONITORING / THRESHOLDS UNVERIFIED
```

and defining them becomes part of the section 11.7 runtime-operations
prerequisite. The specification may still be approved and the `OFF` foundation
may still be implemented and merged if the repository gates otherwise permit,
because the merged state is inert; **`OBSERVE` cannot be production-authorized
without this evidence.**

**After `ENFORCE`**, continue monitoring: actual guard rejections; any
legitimate-client rejection incident; GraphQL latency, error rate and
availability; **mode state across the running fleet**; and store availability
where operationally observable.

### 8.4 Operational runbook

A runbook is required. Its contents depend on facts section 11.7 must establish,
so this specification **requires** them rather than fabricating them. It must
state:

- the **verified mechanism for setting the mode**;
- **whether a restart or redeploy is required** to change it;
- **how a mode change propagates to all replicas**;
- **how to verify the effective mode fleet-wide**;
- the **expected propagation interval**;
- the `OFF -> OBSERVE` procedure;
- the `OBSERVE -> ENFORCE` procedure;
- the `ENFORCE -> OBSERVE/OFF` rollback procedure;
- **who authorizes a rollback**;
- **service-health** rollback criteria (section 8.3 thresholds);
- **compatibility** rollback criteria (legitimate-client rejections);
- **how store unavailability outside `ENFORCE` is verified**, so the fail-closed
  coupling is confirmed operationally rather than assumed.

No other operational machinery is created by this task: no dashboard, no alerting
rule, no on-call procedure. Section 8.3 requires that authoritative health
signals be **identified and verified to exist**, not that new ones be built.

---

## 9. Acceptance criteria

- [ ] `ADR-0006` is `APPROVED` and its approved content is reachable from
      `develop` before implementation begins.

**Central mutation request guard (`ADR-0006` section 4.12.6; section 6.6)**

All rejection criteria below apply to **baseline-valid** requests in guard mode
**`ENFORCE`**. In `OFF` nothing is evaluated; in `OBSERVE` the same decision is
made but nothing is rejected.

- [ ] A **baseline-valid** mutation with a duplicate executable top-level
      response key written directly is rejected before execution, in `ENFORCE`.
- [ ] The same duplicate introduced through a **named fragment spread** is
      rejected.
- [ ] The same duplicate introduced through an **inline fragment** is rejected.
- [ ] For every rejected case, the **measured** mutation resolver execution count
      is `0` and the **measured** database write count is `0`.
- [ ] Distinct top-level mutation aliases are accepted and each executes exactly
      once.
- [ ] A duplicate that `@skip`/`@include` definitely excludes for the concrete
      request is **accepted** and executes once — proven for literal conditions,
      request-supplied variables, **omitted variables carrying an operation
      default**, and request variables **overriding** an operation default, in
      both directions, and through field, fragment-spread and inline-fragment
      directives.
- [ ] The guard builds the effective variable map as operation defaults
      overridden by request variables, mirroring the pinned executor's
      `or_insert`; raw request variables are never used for directive
      evaluation.
- [ ] Every directive decision is asserted against **Juniper's observed
      execution** of the same document and variables — a resolver-invocation
      count from a real execution — not against an independently written
      expectation.
- [ ] An omitted-but-defaulted variable is treated as **resolved**, never as an
      undecidable condition.
- [ ] The guard runs the baseline eligibility gate of `ADR-0006` section
      4.12.6.5.3 — parse, document/schema validation (plus the
      introspection-disabled rule where applicable), operation selection and
      input-variable validation — using pinned juniper's own public helpers, in
      that order, before any duplicate-key analysis.
- [ ] A baseline-invalid request produces **no** guard rejection and **no**
      observation event in any mode, and its externally visible error and HTTP
      status are byte-comparable to ordinary juniper with no guard present.
      Proved for each failing gate: parse, document validation, operation
      selection and input validation.
- [ ] The eligibility gate returns, rewrites and suppresses **no** validation
      error of its own; it only decides whether the guard may decide.
- [ ] Guard mode defaults to `OFF`, and the merged state is `OFF`.
- [ ] In `OFF` the guard short-circuits **before** parsing, selection or
      validating, so it imposes no cost on any request of any kind.
- [ ] In `OBSERVE`/`ENFORCE`, a non-mutation exits after operation-type
      discrimination without document or input validation and without
      duplicate-key analysis, and its response is byte-identical to the no-guard
      baseline.
- [ ] The implementation report states the request-path cost as: parse and
      operation selection on **every** request, plus validation on mutations
      only. It must **not** describe the cost as bounded to mutations.
- [ ] Preview performance evidence exists comparing `OFF` vs `OBSERVE` for
      representative queries and mutations — and `OBSERVE` vs `ENFORCE` where
      materially different — covering validation-failing, high-complexity and
      fragment-heavy documents (section 11.3).
- [ ] Configuration defaults to `OFF`, verified by test.
- [ ] Production activation evidence demonstrates verified runtime mode handling
      **separately** from unit tests: propagation across replicas, fleet-wide
      verification, and a rehearsed rollback with measured timing (section 11.7).
- [ ] Evidence exists that actual production or preview telemetry can observe
      latency, error rate and availability during `OBSERVE` (section 8.3).
- [ ] In `OFF` the guard evaluates nothing and rejects nothing; production
      request acceptance is byte-identical to the base.
- [ ] The implementation report states the overhead as duplicate parse **and
      validation** on the guarded path in `OBSERVE`/`ENFORCE`, never as "one
      extra parse".
- [ ] In `OBSERVE` a document `ENFORCE` would reject is **not** rejected,
      executes normally with an unchanged response, and produces exactly one
      observation event.
- [ ] In `OBSERVE` and `OFF` the loader store is unavailable: every lookup reads
      `NotLoaded` and every path takes the direct fallback.
- [ ] `guard OFF + store enabled` and `guard OBSERVE + store enabled` are
      **unrepresentable** — store availability is derived from the mode value,
      not configured independently. Verified by inspection of the added modules,
      not only by test.
- [ ] Observation and rejection events carry the mode, the colliding response
      key, and the operation name only when supplied, and never the document,
      variables or any argument value.
- [ ] A duplicate compatible response key in a **query** operation is accepted and
      is unaffected by the guard.
- [ ] Duplicate response keys **below** the top level of a mutation are accepted
      and unaffected.
- [ ] The rejection's HTTP status and serialized body are compared against a real
      juniper validation failure and match it; no new handler branch exists.
- [ ] The rejection message exposes no loader, store or scope internals.
- [ ] The guard makes no authorization decision, and `GraphQLRequest::execute` is
      not replaced.
- [ ] None of the 88 `MutationRoot` resolver methods is modified.
- [ ] Exactly one warning-level log record is emitted per rejection, carrying no
      document, variables or argument values.
- [ ] The guard runs under **both** the async `GraphQLRequest::execute` path and
      the `execute_sync` test path.
- [ ] Mode transitions are exercised in test: `OFF -> OBSERVE -> ENFORCE` and
      `ENFORCE -> OBSERVE -> OFF`, with store availability following the mode in
      every direction.

**Store and scoping**

- [ ] The request-scoped store exists on `Context`, is initialised empty per
      construction, and `Context` remains `Sync`.
- [ ] Store contents are unique per GraphQL request; concurrent independent
      requests share nothing, proven by test.
- [ ] No global, static or cross-request cache or singleton exists, verified by
      inspection of the added module.
- [ ] The set-based loader issues exactly one statement per
      `(scope, loader, shape)` dispatch, using `.eq_any(...)`, and never iterates
      keys issuing per-key statements.
- [ ] Store identity is
      `(top-level response key, loader identity, normalized load shape, parent key)`;
      the load shape is a typed loader-specific value, not a serialized argument
      string.
- [ ] One loader-owned shape constructor is used by both the prefetch site and
      the child lookup.
- [ ] An omitted argument and an explicitly supplied schema default produce the
      same shape and resolve against the same entry.
- [ ] Semantically different argument variants never share a stored entry, and
      each alias returns the correct result for its own shape.
- [ ] Distinct shapes produce exactly one dispatch each — not one per parent, and
      not one shared dispatch across variants.
- [ ] Prefetch sites enumerate child selections via `children()` filtered on
      `field_original_name()`, never via `select()` / `has_child()`, so aliases
      are found.
- [ ] For a covered parent list of size `n`, the measured child-statement count
      does **not** scale linearly with `n`.
- [ ] That measurement is recorded for at least **two distinct values of `n`**,
      reported as `parent count | prefetch child-query count | direct baseline
      child-query count`, with the prefetched count bounded while the baseline
      grows.
- [ ] Statement counts are measured through a pool constructed **after** the
      instrumentation hook is installed, never through the existing
      `OnceLock<Arc<PgPool>>` test pool.
- [ ] Duplicate parent keys produce one key in the statement and no additional
      statements.
- [ ] Repeated aliases of the same normalized shape on the same parent cause no
      additional statements.
- [ ] A second prefetch site covering an already-loaded
      `(scope, loader, shape, key)` set issues no additional SQL **within that
      scope**; across distinct scopes a second bounded dispatch is correct and
      expected.
- [ ] The store distinguishes `NotLoaded`, `Loaded` (including `Loaded([])`) and
      `LoadFailed`.
- [ ] `NotLoaded` falls back to the direct query and returns the correct result.
- [ ] `Loaded([])` returns empty **without** a database query and is never
      treated as a miss.
- [ ] `LoadFailed` returns a field error, issues **no** retry query, and is never
      treated as absence or as an empty result.
- [ ] A mixed present/absent key set resolves every parent correctly.
- [ ] A prefetch failure does **not** fail the parent list field; the parent list
      resolves and the error surfaces at the child field.
- [ ] The GraphQL-visible error contract is verified against the direct path —
      `errors[].path`, null propagation and `extensions.type` — with any
      intentional difference documented explicitly.
- [ ] Prefetched results equal direct per-parent results, element for element
      and in order.
- [ ] Whole-store invalidation clears `Loaded` and `LoadFailed` state across all
      scopes, all loaders, all shapes and all keys.
- [ ] Ordinary correctness — including mutation read-after-write — does **not**
      depend on any mutation resolver invoking invalidation, and no production
      mutation resolver is modified.

#### Top-level response-key scoping (`ADR-0006` section 4.12)

- [ ] The store is keyed by
      `(top-level response key, loader identity, normalized load shape, parent key)`.
- [ ] The prefetch site and its terminal resolvers derive **identical** scope
      values on both direct and descendant paths.
- [ ] No loader entry, successful or failed, is visible across two top-level
      response keys.
- [ ] Two top-level aliases of the same schema field
      (`a: publishers { ... } b: publishers { ... }`) produce **separate** loader
      namespaces.
- [ ] Repeated occurrences of one response key share one scope, and no
      source-position or AST-occurrence component is part of the scope key.
- [ ] No resolver detects operation type, and the raw GraphQL document is never
      parsed to derive scope.

##### Compatibility shim — path extraction

- [ ] A top-level field **without** an alias returns its field response key.
- [ ] A top-level **aliased** field returns the alias.
- [ ] A nested **direct child** returns the same first path segment as its parent
      site.
- [ ] A **deeply nested descendant** returns the same first path segment.
- [ ] **Inline fragments** preserve the same first scope.
- [ ] **Named fragments** preserve the same first scope.
- [ ] Direct prefetch site, descendant prefetch site and terminal resolver all
      derive identical scope values within one top-level field.

##### Compatibility shim — side effects

- [ ] Calling the scope helper adds **no** GraphQL error.
- [ ] It changes **no** `errors[]` entry.
- [ ] It changes **no** result data.
- [ ] It has **no** database side effect.

##### Compatibility shim — failure and isolation

- [ ] If the scope cannot be derived the helper **fails closed**: the prefetch is
      skipped, lookups read `NotLoaded`, and the field falls back to its direct
      query. No shared or request-global namespace is substituted, and the parent
      list field does not fail.
- [ ] `Executor::new_error(..)` path extraction appears in **exactly one** module;
      no loader, prefetch site or resolver uses the technique directly.
- [ ] The shim's module documentation and tests state its coupling to the pinned
      Juniper API and the revalidation obligation of `ADR-0006` section 4.12.14.
- [ ] No package dependency was added to implement the shim.

##### Store collision matrix

- [ ] Same parent key + same loader + same shape under **different scopes** never
      collides.
- [ ] Same scope + same parent key + **different loaders** never collides.
- [ ] Same scope + same loader + same parent + **different shapes** never
      collides.
- [ ] The same full key returns the expected stored value.
- [ ] A `LoadFailed` recorded under scope `A` does **not** poison scope `B`; the
      lookup under `B` is `NotLoaded` and falls back.
- [ ] Whole-store invalidation clears both scopes.

#### Mutation behaviour (`ADR-0006` sections 4.12.10-4.12.12)

- [ ] Read-after-write holds **within one top-level mutation field**: a test-only
      mutation that writes child data and then selects the affected loader-backed
      field in the same operation returns the written value, not a prefetched one.
- [ ] Batching inside a mutation payload fan-out works, and stays bounded as the
      nested parent count rises.
- [ ] Two top-level mutation fields in one operation are isolated: entries created
      under the first cannot satisfy the second, and the second's nested selection
      observes the second write rather than cached state from the first.
- [ ] That isolation holds **under async execution where the executor may
      interleave the two top-level futures**, and is proven by scope isolation
      rather than by relying on execution order.
- [ ] No production mutation resolver is modified.

#### Descendant prefetch (`ADR-0006` section 4.19)

- [ ] A **direct** path (`list -> loader-backed child`) batches correctly.
- [ ] An **indirect** path (`list -> intermediate object -> loader-backed
      descendant`) batches correctly.
- [ ] Aliases at **intermediate** path segments are matched by
      `field_original_name()` and do not defeat detection.
- [ ] Aliases at the **terminal** segment are matched the same way.
- [ ] **Every** matching terminal selection is discovered, including two
      intermediate branches each carrying a terminal selection; traversal stops
      at no level's first match.
- [ ] Terminal load shapes normalize correctly, and are constructed from the
      terminal selection rather than from an ancestor selection.
- [ ] The descendant key projector derives keys **only** from data already
      present on the already-resolved parent items.
- [ ] Duplicate projected terminal keys are de-duplicated before dispatch.
- [ ] A second ancestor or list site does not reload an already-loaded terminal
      key **within the same execution scope**; results are stored under the full
      `(scope, loader, shape, terminal key)` identity, with **no** separate
      namespace for indirectly prefetched entries.
- [ ] The same descendant terminal key reached under a **different** top-level
      response key is a distinct entry and is dispatched once for that scope.
- [ ] Indirect prefetch never bypasses an intermediate authorization decision
      (`ADR-0006` section 4.19.4).
- [ ] The terminal child resolver issues **no** fallback statement on a covered
      descendant path.
- [ ] The terminal statement count stays bounded as the ancestor list grows,
      measured at two distinct list sizes.
- [ ] Statements issued by the **intermediate** resolver are reported as a
      separate figure and are **not** presented as part of terminal-loader
      compliance (`ADR-0006` section 4.19.5).
- [ ] The mechanism is proven under both `juniper::execute_sync` and
      asynchronous `execute`.
- [ ] The generated GraphQL SDL is byte-identical to the base.
- [ ] `thoth-api/src/schema.rs`, `thoth-api/migrations/`, `Cargo.toml` and
      `Cargo.lock` are byte-identical to the base.
- [ ] `thoth-api/src/policy.rs` is unchanged.
- [ ] No existing child resolver is modified, and no argument is added to any
      production field.
- [ ] The foundation declares **no** production field N+1 compliant; the
      adoption-coverage obligations of `ADR-0006` section 4.18.2 attach to
      adopting tasks.
- [ ] Existing GraphQL tests pass unmodified.
- [ ] `BE-02` remains unimplemented: no `DistributionPlatform`, no
      `publisher_distribution_platform`, no `BE-02` GraphQL field.
- [ ] PR #788 and issue #765 are unmodified.

---

## 10. Required tests

### Central mutation request guard

Real GraphQL tests, exercised through both `juniper::execute_sync` and the async
`GraphQLRequest::execute` harness (`thoth-api/tests/support/mod.rs:108`). Test-only
mutations and types; **no production mutation resolver may be modified**.

- **allowed — distinct aliases**

  ```graphql
  mutation {
    first:  updateA(...) { ... }
    second: updateA(...) { ... }
  }
  ```

  accepted; each executes exactly once;
- **rejected — direct duplicate**

  ```graphql
  mutation {
    x: updateA(...) { a }
    x: updateA(...) { b }
  }
  ```

  rejected before resolver execution;
- **rejected — duplicate through a named fragment** — the equivalent duplicate
  introduced by a fragment spread is rejected;
- **rejected — duplicate through an inline fragment** — likewise;
- **baseline-invalid requests must not produce guard decisions.** Binding, and
  the whole matrix is required. Each document must be **both** baseline-invalid
  **and** shaped as a duplicate top-level mutation response key, so that a guard
  which ignored the eligibility gate would visibly misbehave. For every case,
  run the same request through the guarded path and through ordinary pinned
  juniper with no guard present, and assert all of:

  - in `OBSERVE`: the request is **not** rejected by the guard, **no**
    would-be-rejection compatibility event is emitted, and the externally
    visible error is the ordinary juniper error;
  - in `ENFORCE`: the guard returns **no** duplicate-key rejection, zero
    application resolvers run and zero writes occur (as they already would under
    a validation failure), and the externally visible error is the ordinary
    juniper error;
  - in both: the response body and HTTP status are byte-comparable to the
    no-guard baseline.

  Required categories, each with a duplicate top-level response key present:

  | Category | Example | Failing gate |
  |---|---|---|
  | unknown field | a top-level field not on `MutationRoot` | document validation |
  | invalid field selection | a sub-selection on a scalar field | document validation |
  | invalid directive usage | an unknown directive on a top-level field | document validation |
  | non-null variable declaring a default | `$skip: Boolean! = true` — the form pinned juniper itself rejects | document validation |
  | missing required variable | `$skip: Boolean!` with no value supplied | input validation |
  | invalid variable value/type | `$skip: Boolean!` supplied a string | input validation |
  | multiple operations, no `operationName` | two named mutations, none selected | operation selection |
  | unknown `operationName` | a name matching no operation | operation selection |
  | parse failure | a syntactically invalid document | parse |

  The operation-selection cases specifically prove the guard makes **no**
  collision decision before operation selection has succeeded;

- **`OFF` short-circuits before the gate** — in `OFF`, none of the above
  documents is parsed, selected or validated by the guard at all, and behaviour
  is byte-identical to the no-guard baseline;

- **query-path behaviour and overhead.** Because the gate touches every request
  in `OBSERVE`/`ENFORCE`, prove:

  - in `OFF`, a query invokes **no** eligibility parsing, selection or
    validation;
  - in `OBSERVE`, a **valid query** passes through the gate, exits at
    operation-type discrimination, performs **no** document validation, **no**
    input validation, **no** duplicate-key analysis and **no** rejection, emits
    **no** event, and its response is byte-identical to the no-guard baseline;
  - in `ENFORCE`, a valid query is likewise never restricted and its response is
    byte-identical;
  - an **invalid query** produces no guard event and no guard error in any mode,
    and juniper's canonical error is preserved;
  - query behaviour is equivalent to the baseline in every observable respect
    **except** measurable overhead;
  - a **subscription** operation, if reachable, is treated as a non-mutation and
    exits at the same point;

- **directives — literal and supplied variables** — `@skip` and `@include` with
  literal Boolean conditions and with request-supplied variables, proven in both
  directions. A syntactic duplicate that is definitely excluded for the concrete
  request must be **accepted** and execute once. A genuinely undecidable
  condition rejects conservatively, and the test must assert that recorded
  tradeoff rather than an exact result;
- **directives — operation variable defaults.** Binding, and all of the
  following are required. Note that the defaulted variable must be declared
  **nullable**: the pinned `default_values_of_correct_type` rule rejects
  `Boolean! = true` outright, so a `Boolean! = true` document is classified
  **baseline-invalid** at the eligibility gate and never reaches the guard's
  directive logic — it exercises juniper's validation rejection instead, and
  proves nothing about directive evaluation. (It *does* reach the guard's
  eligibility gate, which runs before `GraphQLRequest::execute()`; that case is
  covered separately by the baseline-invalid matrix above.)

  - **`@skip` default true, variable omitted**

    ```graphql
    mutation Q($skip: Boolean = true) {
      x: updateA(...) @skip(if: $skip) { id }
      x: updateA(...) { id }
    }
    ```

    with `$skip` omitted → **one** executable occurrence → **accepted**, and in
    `ENFORCE` it must **not** be rejected;
  - **explicit override false** — the same document with `{"skip": false}` →
    **two** executable occurrences → **rejected** in `ENFORCE`, and recorded as
    a would-be rejection in `OBSERVE`;
  - **`@include` default false, variable omitted** — the equivalent document
    proving the default **excludes** the duplicate → **accepted**;
  - **explicit override true** — the same with `{"inc": true}` → the duplicate
    becomes executable → **rejected**;
  - **through a named fragment** — at least one defaulted-variable case where
    the duplicate arrives through a named fragment spread, including the case
    where the directive sits on the **spread** itself;
  - **through an inline fragment** — the equivalent case with the directive on
    the inline fragment;
  - **request value precedence** — an explicitly supplied request variable
    overrides the operation default exactly as juniper's `or_insert` does;
  - **no regression** — variables with **no** default, supplied `true` and
    `false`, behave exactly as before this change.

  Every one of these must assert the guard's verdict against **Juniper's
  observed execution** of the same document and variables — a resolver-invocation
  count from a real execution — not against a separately written expectation
  table (`ADR-0006` section 4.12.6.5.2);
- **resolver execution count** — for each rejected operation, a resolver-call
  counter on the test-only mutation reads `0` **and** the statement-count harness
  of section 10's performance tests reads `0` database writes. Both are required;
  neither alone is sufficient;
- **query operations** — a duplicate compatible response key in a query is
  accepted, both occurrences share one scope, and the terminal loader issues no
  additional statement for the second;
- **nested duplicates** — duplicate response keys below the top level of a
  mutation are accepted and unaffected;
- **error shape parity** — the rejection's HTTP status and serialized body are
  compared directly against a real juniper validation failure (for example an
  unknown field) and match: `is_ok()` false, HTTP 400, an `errors` array of
  `{message, locations}`, and no `data` key;
- **operation selection** — with several named operations in one document and an
  `operationName` supplied, the guard evaluates the **selected** operation only;
- **mode OFF** — the merged default. The previously rejected documents are
  accepted, the guard evaluates nothing, no observation event is emitted, **and**
  the store is unavailable: every lookup reads `NotLoaded`, every path falls
  back, and results are byte-identical to the pre-foundation base;
- **mode OBSERVE** — every document `ENFORCE` would reject is **accepted** and
  executes normally with an unchanged response and unchanged resolver counts,
  while producing exactly one observation event carrying the mode, the colliding
  response key and the operation name when supplied. The store remains
  unavailable;
- **observation event content** — asserted positively and negatively: the event
  contains the mode and colliding response key, and contains **no** document
  text, **no** variables and **no** argument values;
- **mode transitions** — `OFF -> OBSERVE -> ENFORCE` and
  `ENFORCE -> OBSERVE -> OFF`, with store availability following the mode in
  both directions;
- **coupling is structural** — `guard OFF + store enabled` and
  `guard OBSERVE + store enabled` cannot be constructed. Demonstrated by the
  shape of the API rather than only by a runtime assertion.

### Unit — state model

- `NotLoaded` is representationally distinct from `Loaded([])`;
- `LoadFailed` is representationally distinct from both;
- a failed dispatch never triggers the fallback;
- a successful empty bucket never triggers the fallback;
- a genuine miss does trigger the fallback;
- non-destructive read: two reads of the same entry return the same bucket.

### Unit — load shapes

- the same normalized shape de-duplicates to one entry and one dispatch;
- different argument values produce different shapes and do not collide;
- an omitted argument and an explicitly supplied schema default normalize to the
  same shape;
- one parent key may hold several shapes simultaneously, each correct;
- two loaders with identical parent key types cannot read each other's entries;
- the shape type's equality is semantic — two shapes built from equivalent
  argument sets compare equal regardless of construction order or formatting.

### Unit — partitioning and store

- key de-duplication: `n` references to one key yield one key;
- partitioning determinism: identical rows and keys yield identical buckets,
  repeatedly;
- partitioning correctness: every returned row lands in the bucket for its own
  key and no other;
- whole-store invalidation clears `Loaded` and `LoadFailed` across all loaders
  and all shapes.

### Integration/database

Against the disposable test database, through Juniper execution:

- **single key** — a one-parent list resolves correctly;
- **many keys** — an `n`-parent list resolves every parent correctly;
- **duplicate keys** — a parent appearing more than once resolves correctly with
  no extra statement;
- **absent key** — a parent reached without a prefetch site falls back and
  returns the correct result;
- **mixed present/absent** — one operation containing both cases resolves both
  correctly;
- **same-shape aliases** — two aliases of the same field with the same normalized
  arguments yield one shape, one dispatch, and identical correct results for both
  aliases;
- **different-shape aliases** — two aliases of the same field with different
  arguments yield distinct shapes, one dispatch each, no cross-contamination, and
  the correct result for each alias;
- **default equivalence** — the omitted-argument and explicit-default forms
  resolve against the same entry and issue one dispatch between them;
- **same-scope multi-site reuse** — two prefetch sites covering the same
  `(scope, loader, shape)` **under the same execution scope** reuse the same
  entries, issue no duplicate SQL, and resolve every parent correctly;
- **cross-scope isolation** — the same two prefetch sites placed under
  *different* top-level response keys are two entries: each scope issues its own
  bounded dispatch, neither reads the other's entries, and every parent still
  resolves correctly. This test must be distinct from the one above, and a
  design that merges them is not compliant;
- **database error** — using the existing `failing_pool()` helper
  (`thoth-api/src/model/tests.rs:73`) or equivalent, the prefetch failure is
  stored as `LoadFailed`, the parent list field still resolves, each covered
  child field emits the failure, no retry SQL is issued, and no empty-list result
  is produced;
- **error contract** — the prefetched failure's `errors[].path`, null propagation
  and `extensions.type` are compared against the direct per-parent failure, with
  any intentional difference recorded;
- **equivalence** — prefetched output equals direct per-parent output, in order;
- **concurrent independent requests** — two `Context` values executing
  concurrently observe disjoint store contents;
- **request isolation** — a second request with a fresh `Context` observes an
  empty store regardless of what the first loaded;
- **read-after-write coherence within one top-level mutation field** — a
  test-only mutation writes data, returns a payload object, a nested list/fan-out
  resolver prefetches after the write, and the loader-backed terminal field
  receives the newly written value. Batching stays bounded as the nested parent
  count rises. Use test-only mutations and types; **no production mutation
  resolver may be modified**;
- **isolation across top-level mutation fields, distinct aliases** — one
  operation contains two distinct top-level response keys, the first creating
  loader state and the second writing different data and reading the same logical
  loader/key. The second nested selection must observe the second write, never
  the first scope's cached value. The assertion must rest on **scope isolation,
  not execution order**;
- **duplicate compatible response keys** — the same read-after-write scenario
  written with one duplicated top-level response key instead of two aliases must
  be **rejected by the guard**, with zero resolver executions and zero writes. A
  design that handles only the distinct-alias case and lets the duplicate through
  is **not compliant**: the duplicate is precisely the case that defeated the
  previous architecture (`ADR-0006` section 4.12.6.1);
- **fragment-created duplicates** — the same, with the duplicate introduced
  through a named fragment spread and through an inline fragment;
- **shared terminal fragment** — two top-level mutation fields with **distinct
  aliases** whose payload selections reach the terminal loader-backed field
  through **one shared named fragment**. Scope isolation must still hold, even
  though the terminal resolver's own source position is identical on both paths.
  This is the case that rejected an execution-occurrence scope
  (`ADR-0006` section 4.12.6.3), and it must be covered explicitly;
- **async interleaving** — the isolation above must hold under async `execute`
  where the executor may interleave the two top-level futures. Where practical,
  use a deliberately yielding async test-only mutation or resolver to force
  interleaving. If the pinned macro and test architecture cannot produce such a
  yielding fixture while preserving the sync-parity requirement of section 3.2,
  document the exact limitation and use the strongest available proof — but do
  **not** weaken the isolation invariant itself;
- **query scope isolation** — see the two-top-level-field query test in section
  9's scoping criteria; entries under one top-level query response key are never
  reused under another;
- **descendant path** — `list -> intermediate object -> loader-backed
  descendant` issues one terminal dispatch over the de-duplicated projected key
  set, returns results equal to the direct per-parent result in order, and issues
  no terminal fallback statement;
- **descendant alias matrix** — aliases at intermediate and terminal segments,
  and two aliased intermediate branches each carrying a terminal selection, are
  all detected, with every terminal selection collected;
- **descendant reuse** — a terminal key already loaded by one site is not
  re-dispatched by a second site, proving the single shared terminal namespace;
- **evidence separation** — the terminal-loader statement count and the
  intermediate resolver's statement count are reported as distinct figures on the
  same measured descendant path;
- **execution-path parity** — the same operation produces the same result and
  the same statement count under `execute_sync` and under async `execute`. The
  pinned Juniper serializes top-level fields on the sync path but drives them
  through `FuturesOrdered` on the async path (`ADR-0006` section 4.12.10), so
  parity must be demonstrated on both rather than inferred from the sync result.
  Under both paths, verify specifically:

  - the same top-level response-key derivation;
  - the same direct-path result;
  - the same descendant-path result;
  - the same alias behaviour;
  - the same scope isolation;
  - no GraphQL errors generated by scope extraction.

### Authorization/security

- keys are drawn only from already-resolved parents: an operation whose parent
  list is scope-restricted must not cause child rows for out-of-scope parents to
  be fetched or returned;
- if the proof consumer touches child-level protected data, the full negative
  matrix required by `thoth-api/AGENTS.md` section 7 applies — no
  authentication, wrong role, wrong publisher scope, correct scope, superuser.

### Regression

- the complete existing `thoth-api` GraphQL test suite passes **unmodified**;
- `cargo test --workspace` passes.

### Manual verification

- confirm the generated GraphQL SDL is byte-identical to the base, recording the
  exact command and diff result;
- confirm by `git diff --stat` that no runtime file outside the intended set,
  and no dependency manifest, changed.

### Performance

Wall-clock time is **not** an acceptance metric.

The acceptance signal is **SQL statement count and bounded database work**, per
`ADR-0006` section 8. Required evidence:

- statement counts observed at the driver via
  `diesel::connection::set_default_instrumentation`, or an equivalent method that
  observes actual SQL if instrumentation proves unworkable;
- application-level counters alone are insufficient, because they cannot detect
  a per-parent statement issued by a fallback path;
- counts recorded for at least two distinct `n`, reported **per top-level
  response scope**, per `ADR-0006` section 8.2:

  ```text
  top-level scope | parent count | prefetch terminal-query count |
  direct baseline terminal-query count | legacy intermediate-query count, if any
  ```

  with the prefetched count bounded within each scope while the direct baseline
  grows with `n`.

**Two-top-level-field query test.** Using the test-only schema, prove the
accepted cross-scope reuse tradeoff explicitly rather than hiding it
(`ADR-0006` section 4.12.13):

```graphql
query {
  first:  parents { childLoaderField }
  second: parents { childLoaderField }
}
```

Expected and required to be reported:

- `first` issues **one** bounded set-based child dispatch;
- `second` issues **one** bounded set-based child dispatch;
- results are correct under both;
- **no** cross-scope reuse occurs;
- the total is **2** dispatches for the two top-level scopes — **not** `N + N`;
- increasing the parent count within either field does **not** increase that
  field's child dispatch count.

The second dispatch is the accepted cost of `ADR-0006` section 4.12.13 and must
appear in the reported evidence.

**Mutation fan-out counts.** Exact statement-count evidence must also be produced
inside a mutation payload fan-out, not only inside a query fan-out, using the
same per-scope reporting unit.

Measurement-pool lifecycle, per `ADR-0006` section 8.1.1. The hook applies only
to connections established after installation, and the repository's ordinary test
pool is a process-wide `OnceLock<Arc<PgPool>>` that may already hold established
connections. Holding the test lock serializes tests; it does not recreate
connections. The measurement must therefore **not** use that singleton pool.

Required sequence:

1. acquire the existing exclusive database test lock (`test_lock()`);
2. reset and prepare the disposable test database;
3. install the instrumentation hook;
4. construct a **new dedicated pool** after hook installation;
5. run the measured operation through that pool;
6. count actual `StartQuery` events;
7. isolate the count from setup, fixture and migration statements.

If Diesel instrumentation proves unsuitable, any mechanism observing actual
PostgreSQL/Diesel SQL — for example PostgreSQL statement-log capture — is
acceptable. Narrative or application-counter-only evidence is not.

### Required commands

```bash
cargo test -p thoth-api --features backend
cargo test --workspace
cargo check --workspace
cargo clippy --all --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

Record exact commands and concise results. `passed` without the command and
result is not acceptable evidence.

---

## 11. Rollout

**Corrected during remediation.** A previous revision made the guard active on
every mutation request from the merge commit, behind a kill switch defaulting to
enabled, and treated CTO merge authorization as authorizing that activation.
That is withdrawn: `release-gates.md` prefers a safe post-merge state, requires a
merge that itself changes production behaviour to satisfy the production-ready
gate first, and requires production activation of HIGH-risk work to carry preview
acceptance, controlled activation, monitoring, rollback, an activation owner, an
observation period and explicit CTO approval. A cross-client request-contract
change must not ride in on a merge.

**The rule is:**

```text
repository merge  !=  production ENFORCE activation
```

### 11.1 State at merge

```text
guard mode                 = OFF
loader store               = unavailable
production request accept  = unchanged
```

- the **store** merges present but unavailable, adopted by no production field;
- the **guard** merges present in `OFF` — it evaluates nothing and rejects
  nothing;
- existing per-parent child resolvers are unchanged;
- merging therefore changes **no** production behaviour.

### 11.2 Activation lifecycle

Staged, and each transition is separately authorized. It must not be collapsed
into the merge event.

```text
merge (guard OFF, store unavailable, no request-path overhead)
  -> runtime-operations evidence for mode control verified (11.7 / CG-13)
    -> service-health signals and activation thresholds verified (8.3)
  -> preview/staging acceptance of the exact candidate, including the
     performance evidence of 11.3 and a rehearsed, timed rollback
  -> explicit CTO production activation approval for OFF -> OBSERVE
  -> controlled OBSERVE activation
  -> explicit observation window
  -> compatibility AND operational evidence reviewed and signed off (11.4)
  -> production-ready evidence for ENFORCE
  -> separate explicit CTO production activation approval for OBSERVE -> ENFORCE
  -> ENFORCE activation
  -> post-activation observation period
  -> (only then) a later task may adopt the store on a mutation path
```

**Corrected during remediation: `OBSERVE` is itself a HIGH-risk production
activation.** A previous revision let `OBSERVE` proceed under ordinary task
ownership after preview acceptance. That is withdrawn. `OBSERVE` parses and
selects an operation for **every** GraphQL request, validates and analyses
mutations, and emits structured logs — live behaviour on the common request path.
`release-gates.md` section 5 requires CTO approval for production activation of
high-risk work, so:

- `OFF -> OBSERVE` requires **explicit CTO production activation approval**;
- `OBSERVE -> ENFORCE` requires a **separate** explicit CTO approval;
- **CTO merge authorization is not production activation authorization** for
  either;
- approving `OBSERVE` does **not** approve `ENFORCE`, and approving `ENFORCE`
  does **not** retroactively approve `OBSERVE`. They are separate decisions over
  different production effects — added overhead on all traffic, versus changed
  request acceptance for clients.

**`OBSERVE` is the controlled compatibility pilot.** A previous revision argued
that a shadow-comparison period adds no evidence because the guard's decision is
discrete and deterministic. **That reasoning is rejected.** It answers the wrong
question. The decision function is deterministic and fully covered by tests; the
open question is:

```text
Does real production traffic contain documents that ENFORCE would reject?
```

No unit test can answer that, and the repository cannot enumerate its external
API clients, so the absence of a known caller using duplicate top-level mutation
response keys is **not** evidence that none does. `OBSERVE` answers exactly this
against real traffic while rejecting nothing. It therefore discharges
`risk-classification.md`'s "feature flag, comparison mode or controlled pilot
where possible" control, and it is mandatory before `ENFORCE`.

No percentage-based or per-tenant staged rollout is required: the repository has
no established mechanism for one, and `OBSERVE` already covers all traffic.

### 11.3 Preview/staging acceptance, required before OBSERVE and ENFORCE

**Performance evidence, required before `OBSERVE`.** Because the eligibility gate
touches the common GraphQL request path — parse and operation selection for
**every** request in `OBSERVE`/`ENFORCE` (section 6.6) — overhead must be
measured, not assumed. Using repository-supported tooling rather than a new
benchmark subsystem, measure at least:

- representative **query** latency with mode `OFF`;
- the same representative queries with `OBSERVE`;
- representative **mutation** latency with `OFF`;
- the same representative mutations with `OBSERVE`;
- CPU/resource impact where existing test or preview instrumentation exposes it;
- behaviour for requests that fail validation;
- high-complexity but valid GraphQL documents;
- fragment-heavy documents;
- and, where materially different from `OBSERVE`, the same measurements for
  `ENFORCE`.

The evidence must answer:

```text
Does enabling OBSERVE create an unacceptable latency, error-rate or
resource regression on the real common request path?
```

**Thresholds must be derived from repository evidence, and approved before
`OBSERVE`.** No numeric threshold is stated in this specification, because no
authoritative production latency or error-rate baseline for the `thoth` GraphQL
API exists in this repository (section 8.3). Deriving and approving explicit
acceptance thresholds is a **hard activation prerequisite**; a number must not be
invented to unblock activation.

**Rollback rehearsal.** The mode change **and its rollback** must be rehearsed in
preview/staging, with the propagation and rollback timing measured and recorded
(section 11.7). Rollback must not be described as certain or deploy-free until
that evidence exists.

Exercise the **exact implementation candidate** in a production-like or preview
environment and prove:

- ordinary single-field mutations remain accepted;
- distinct top-level aliases remain accepted;
- a duplicate direct top-level response key is detected;
- a named-fragment duplicate is detected;
- an inline-fragment duplicate is detected;
- directive and operation-variable-default cases behave correctly (section 6.6);
- in `OBSERVE`, none of those requests is rejected;
- in `ENFORCE`, a rejected request executes zero resolvers and performs zero
  writes;
- ordinary validation failures are unchanged, including when the invalid document
  also carries a duplicate top-level response key: the client receives juniper's
  canonical error and no guard event is recorded;
- the store is unavailable in `OFF` and in `OBSERVE`.

### 11.4 OBSERVE evidence, required before ENFORCE

Run `OBSERVE` for an explicit, recorded observation window and record:

```text
number of mutation requests inspected
number of would-be duplicate-response-key rejections
operation names, where supplied
colliding response keys
period observed
```

Never recorded: full GraphQL documents, variables, mutation argument values, or
any publisher or user payload data (section 8.2).

**A non-zero would-be-rejection count blocks `ENFORCE`** until the affected
callers have been identified and addressed. It must not be waved through on an
assumption that the traffic is synthetic or unimportant, and the absence of a
known affected caller is not a substitute for investigating a non-zero count.

**`OBSERVE` must answer two distinct questions, and both must pass:**

| Question | Evidence |
|---|---|
| **Compatibility** — are legitimate production clients sending baseline-valid mutation documents that `ENFORCE` would reject? | would-be rejection count; caller investigation; zero unresolved legitimate-client blockers |
| **Operational** — does the eligibility gate materially degrade the GraphQL service? | latency; error rate; availability; resource pressure where relevant; incidents |

Explicitly prohibited:

```text
zero collision events  =>  ENFORCE is safe
```

That inference is invalid if service health regressed. A clean compatibility
result with a degraded operational result blocks `ENFORCE` exactly as firmly as a
non-zero collision count does.

### 11.5 Activation ownership

Using the repository's existing control terminology, and naming no individual:

| Decision | Owner |
|---|---|
| merge authorization | **CTO**, because the task is HIGH risk |
| `OFF -> OBSERVE` production activation approval | **CTO** |
| `OBSERVE -> ENFORCE` production activation approval | **CTO**, separately |
| execution of a mode change | the **named engineering/release owner** on this task |
| post-activation observation sign-off | the role identified by the runtime-operations prerequisite (section 11.7) |

**Two of these are not currently identifiable, and that is an activation
blocker.** `release-gates.md` section 5 requires an "explicit activation owner"
and section 8 requires observation to end with "an explicit sign-off". The
repository identifies neither a production execution owner nor an observation
sign-off owner for `thoth` runtime, because **CG-13** is open. Section 11.7 must
establish them; this specification must not invent them.

### 11.6 Other rollout properties

- **pilot:** `OBSERVE` is the pilot (section 11.2). No separate pilot cohort is
  proposed;
- **observation:** there are **two** observation stages, not one, and the
  rejection event stream is only part of the first. Collision and rejection
  events are the **compatibility** signal alone; they say nothing about service
  health (sections 8.2, 8.3).

  **`OBSERVE` window** — must evaluate **both**, and both must pass before
  `ENFORCE`:

  | Question | Signals |
  |---|---|
  | **Compatibility** — are legitimate baseline-valid mutation documents present that `ENFORCE` would reject? | would-be rejection events; caller investigation; zero unresolved legitimate-client blockers |
  | **Operational health** — does the activated gate materially degrade GraphQL service health? | latency; server/API error rate; availability; relevant resource saturation; gate internal failure or panic where observable |

  **`ENFORCE` observation** — after activation, continue observing: actual guard
  rejections; legitimate-client rejection incidents; GraphQL service health
  (latency, error rate, availability); and mode/fleet correctness as established
  by the runtime-operations controls of section 11.7;
- **store adoption:** `BE-02` becomes the first required consumer, in its own
  separately specified, reviewed and authorized task, and only once `ENFORCE` is
  active and observed (section 18);
- **mass adoption:** prohibited. Existing child resolvers are unchanged and are
  migrated, if at all, under `ADR-0006` section 10.

### 11.7 Runtime-operations activation prerequisite (CG-13)

**Corrected during remediation.** A previous revision claimed rollback was
certain and deploy-free. Those claims are withdrawn; they were not
repository-authoritative.

What is established is only that the mode would be supplied through the existing
`clap` `Arg::env(..)` process-start configuration pattern
(`src/bin/arguments/mod.rs`). That proves a configuration **input** exists. It
proves nothing about dynamic reload, restart or deploy requirements, propagation
timing, cross-replica atomicity, orchestration ownership, change authorization or
rollback verification.

Control gap **CG-13 — "Thoth runtime operations unmapped"**
(`docs/engineering/repository-map/control-gaps.md`) records that runtime,
deployment, rollback, restore verification and approvers are undocumented, and
the same register states that Thoth hosting/rollback remain unverified.

```text
Production activation prerequisite:
CG-13, or a bounded successor runtime-operations control resolving the
relevant Thoth GraphQL configuration / deployment / rollback mechanism,
must be satisfied for this feature before OBSERVE.
```

A bounded partial resolution is acceptable where the repository's control process
permits one — CG-13 need not be resolved for unrelated systems — but these
feature-specific questions must be answered before `OBSERVE`:

1. the production runtime/deployment owner;
2. where the mode value is configured;
3. whether a mode change requires a process restart or a deploy;
4. how a mode change reaches **all** replicas;
5. the expected and observed propagation interval;
6. how the effective mode is verified on the running service;
7. the rollback procedure;
8. who authorizes a rollback;
9. a preview/staging rehearsal of the change and the rollback, with timing;
10. how a partial-fleet mode change is detected and handled.

**Item 10 is load-bearing.** Store availability is derived from the mode and
mutation isolation depends on `ENFORCE`, so a fleet with one replica in `ENFORCE`
and another in `OBSERVE` produces inconsistent client acceptance **and**
inconsistent store availability at the same time. The operations plan must
establish either an atomic or shared configuration mechanism, or rollout
semantics and verification that make mixed-mode periods safe and bounded. This
specification deliberately does **not** invent that mechanism.

**If CG-13 requires its own remediation task to answer these questions, that task
is a dependency of production activation** — not of merge, and not of this
specification's approval. It must be recorded as such rather than resolved here;
resolving CG-13 is outside this task's authorization.

**Merge-ready is not activation-ready.** None of this blocks merging the
foundation while the merged state is genuinely inert: mode `OFF`, store
unavailable, no request-path overhead, no production behaviour change.

---

## 12. Rollback

- **primary control, at the code level:** the mode value.

  ```text
  ENFORCE -> OBSERVE     stop rejecting; keep collecting evidence
  ENFORCE -> OFF         stop evaluating entirely
  OBSERVE -> OFF         stop evaluating entirely
  ```

  Each restores prior request acceptance, and because store availability is
  derived from the mode, each also makes the store unavailable — so no path is
  left depending on a guarantee no longer enforced, and the direct fallback
  carries every affected field.

  **The production mechanism and timing are unverified.** How the value is
  changed, whether a restart or deploy is required, how long it propagates, how
  it is verified across replicas and who authorizes it are all open under CG-13
  (section 11.7). This task therefore **does not claim** rollback is certain,
  immediate, or achievable without a deploy. Those properties must be
  established and rehearsed with measured timing before `OBSERVE`;
- **rollback signals — both are required, neither alone is sufficient:**

  ```text
  legitimate client rejection
  material service-health regression attributable to the guard
  ```

  Collision events must **not** be the sole rollback signal (section 8.3);
- **code rollback:** revert the merge commit. Since the merged state is
  `guard OFF, store unavailable`, this is a no-op for production behaviour and
  nothing depends on the store at that point;
- **after a later store adoption:** revert the adopting field to its direct
  per-parent query. Because that query is retained as the mandatory fallback, the
  field's *result* is unchanged by rollback — only its statement count is;
- **data rollback or forward repair:** none. The task creates no persistent
  state, and a rejected request performs no write;
- **external side-effect handling:** none. No external system is contacted.

---

## 13. Stop conditions

The implementing agent must stop and report `BLOCKED` if:

- `ADR-0006` is not `APPROVED`, or its approved content is not reachable from
  `develop`;
- explicit CTO implementation authorization, bound to a freshly verified exact
  `develop` head, has not been given;
- the approach of section 3.1 (a test-only GraphQL schema) proves technically
  impossible, so proving the mechanism would require adopting or modifying a
  production resolver;
- the mechanism cannot be made to work under **both** `execute_sync` and async
  `execute` without changing Juniper's execution model;
- the statement-count evidence of section 10 cannot be produced by any method
  that observes actual SQL;
- a database migration or a `thoth-api/src/schema.rs` change turns out to be
  required;
- a public GraphQL schema change turns out to be required;
- a new workspace dependency turns out to be required;
- keeping `Context: Sync` conflicts with the store, breaking the async execution
  path;
- read-after-write coherence within one top-level mutation field cannot be
  satisfied without editing production mutation resolvers. **Report `BLOCKED`
  rather than widening scope to touch the mutation resolvers**; the same applies
  if the load-shape, `LoadFailed` or descendant model turns out to require
  mutation-resolver changes;
- descendant prefetch cannot be expressed cleanly using stable pinned Juniper
  APIs plus already-resolved data. Report exactly:

  ```text
  BLOCKED - A2 CANNOT COVER INDIRECT FAN-OUT WITHOUT NEW EXECUTION ARCHITECTURE
  ```

  and do **not** push the unresolved architecture into `BE-02`;
- the top-level response key cannot be derived from a nested resolver through the
  compatibility shim of `ADR-0006` section 4.12.8 using only stable public
  Juniper API — for example if `Executor::new_error(..)` or
  `ExecutionError::path()` does not yield the execution path, or the first
  segment is not the top-level response key. **Report `BLOCKED`**; do not parse
  the raw GraphQL document, do not inspect private Juniper fields, do not use
  `unsafe`, and do not fall back to a request-global namespace;
- the scope helper cannot be made side-effect-free — for instance if
  materializing the path proves to mutate the executor's error state. Report
  `BLOCKED` rather than accepting a mechanism that alters the GraphQL response;
- scope isolation across two top-level mutation fields cannot be demonstrated
  under async execution. Report `BLOCKED` rather than weakening the isolation
  invariant. (Documenting a limitation in the *yielding-fixture* technique is
  permitted by section 10 and is not itself a stop condition; failing to
  establish the invariant at all is.);
- the scoping rule turns out to require modifying production mutation resolvers,
  detecting operation type at nested resolvers, or a `new_error(..)` call outside
  the single shim module. **Report `BLOCKED`** — each of these is prohibited by
  `ADR-0006` section 4.12;
- the central mutation request guard cannot be implemented at the request
  boundary using only public, non-`unsafe` pinned Juniper API — for example if
  `parse_document_source`, `get_operation`, `Operation::operation_type` or the
  public `Selection` AST proves insufficient to expand fragments and identify
  executable top-level occurrences. **Report `BLOCKED`**; do not modify Juniper,
  do not upgrade it, do not fork it, do not add a dependency, and do not
  retrofit the 88 `MutationRoot` resolvers;
- the guard cannot guarantee **zero** mutation resolver executions and **zero**
  database writes for a rejected operation. A guard that rejects after the first
  resolver has run is non-compliant and must be reported as `BLOCKED` rather than
  shipped;
- the guard cannot avoid rejecting a duplicate that `@skip`/`@include`
  definitely excludes for the concrete request, so ordinary conditional documents
  would be rejected. Report `BLOCKED` rather than broadening the compatibility
  restriction beyond `ADR-0006` section 4.12.6.7;
- the baseline eligibility gate cannot be reproduced on stable public pinned
  juniper APIs — `parse_document_source`, `ValidatorContext`, `visit_all_rules`,
  `validation::visit` with `MultiVisitorNil` and
  `rules::disable_introspection::factory`, `get_operation`,
  `validate_input_values`, and the public `RootNode::schema` /
  `RootNode::introspection_disabled` fields. Report `BLOCKED`; do **not** reach
  into private fields, use `unsafe`, manipulate raw source, or build a second
  GraphQL implementation to get there;
- a baseline-invalid request cannot be prevented from producing a guard
  rejection or an observation event. Report `BLOCKED` rather than shipping a
  guard that can replace juniper's canonical errors or corrupt the `OBSERVE`
  compatibility evidence;
- `OFF` cannot be made to short-circuit ahead of the eligibility gate, so `OFF`
  would impose parse/validation cost on production traffic. Report `BLOCKED`
  rather than accepting overhead in the mode whose purpose is to preserve
  current behaviour exactly;
- the guard's directive evaluation cannot be made behaviourally equivalent to the
  pinned executor's effective-selection semantics — including operation variable
  defaults, request-variable override, and directives on fields, fragment spreads
  and inline fragments. Report exactly:

  ```text
  BLOCKED - MUTATION GUARD CANNOT MATCH PINNED JUNIPER EXECUTABLE-SELECTION SEMANTICS
  ```

  Do **not** weaken the invariant to accommodate a mismatch, and do not narrow
  the tests until they stop detecting it;
- the guard mode cannot be threaded through the existing configuration mechanism,
  or store availability cannot be **derived** from it so that
  `guard OFF + store enabled` and `guard OBSERVE + store enabled` are
  unrepresentable. Report `BLOCKED` rather than shipping two independent settings
  with a documented convention that operators must keep them consistent;
- `OBSERVE` cannot be made to leave the response and resolver behaviour
  completely unchanged while still detecting would-be rejections. Report
  `BLOCKED` rather than shipping an observation mode that perturbs traffic;
- the store cannot be made unavailable outside `ENFORCE`, so batching could run
  without its prerequisite. Report `BLOCKED` rather than shipping it;
- correcting the execution layer so that compatible repeated top-level fields
  execute once turns out to be required after all. That is option F3, rejected as
  architecture expansion in `ADR-0006` section 4.12.10.1. Report exactly:

  ```text
  BLOCKED - REQUIRES GRAPHQL EXECUTION-LAYER ARCHITECTURE OUTSIDE THIS TASK
  ```

  and do **not** begin maintaining a custom GraphQL executor or patching Juniper;
- the three-state store cannot be represented such that `NotLoaded`,
  `Loaded([])` and `LoadFailed` are mutually unambiguous;
- a prefetch failure cannot be surfaced at the child field without failing the
  parent list field, so the `ADR-0006` section 4.9.3 equivalence contract cannot
  be met and no acceptable documented difference exists;
- the normalized load shape cannot be constructed identically at the prefetch
  site and at the child lookup, so the two could drift;
- an argument-bearing test-only field cannot be added under `#[cfg(test)]`, so
  multi-shape behaviour could only be proven by adding arguments to a production
  field;
- approved architecture would need to change;
- the scope cannot be completed without unrelated changes;
- repository state differs materially from section 2.1.

The agent must report the stop condition rather than weakening an `ADR-0006`
invariant, adopting a production resolver on its own authority, substituting
narrative for query-count evidence, or shipping an unmeasured per-parent query.

---

## 14. Expected implementation report

The agent must use:

`docs/engineering/ai-delivery/implementation-report-template.md`

The report must additionally contain:

- the statement-count table reported **per top-level response scope** — scope,
  parent count, prefetch terminal-query count, direct baseline terminal-query
  count and legacy intermediate-query count — for at least two values of `n`, for
  **both** the direct and the descendant path, and inside **both** a query and a
  mutation payload fan-out;
- the two-top-level-field query result showing **2** dispatches rather than
  `N + N`, presented as the accepted `ADR-0006` section 4.12.13 tradeoff rather
  than omitted;
- for the descendant path, the **terminal-loader** statement count and the
  **intermediate-resolver** statement count as two separate figures, with an
  explicit statement that bounding the terminal loader does not make the
  operation globally free of N+1 access (`ADR-0006` section 4.19.5);
- the descendant prefetch representation actually implemented — how the selection
  path, terminal loader identity, terminal shape constructor and key projector
  are expressed — and the stable Juniper APIs relied on for recursive,
  alias-safe traversal;
- confirmation that indirectly prefetched entries are stored under the ordinary
  terminal identity, with no separate cache namespace;
- the compatibility shim as implemented: its exact signature, the Juniper APIs
  it calls, the evidence that it is side-effect-free, its fail-closed behaviour,
  and confirmation that it is the **only** site in the codebase using
  `new_error(..)` path extraction;
- confirmation that the shim's module documentation and tests record the pinned
  Juniper coupling and the section 4.12.14 revalidation obligation, and that no
  package dependency was added for it;
- the scope-isolation evidence: identical scope derivation at prefetch site and
  terminal resolver, two top-level aliases of one schema field producing separate
  namespaces, `LoadFailed` under one scope not poisoning another, and the store
  collision matrix results;
- the mutation evidence: read-after-write within one top-level mutation field,
  isolation across two top-level mutation fields, the async-interleaving proof
  (or the documented limitation of the yielding-fixture technique and the
  strongest proof achieved in its place), and confirmation that **no production
  mutation resolver was modified**;
- the exact method used to observe SQL statements, **including how the measured
  pool was constructed relative to the instrumentation hook**;
- the load-shape type for each loader implemented, and how defaults normalize;
- the observed GraphQL error contract comparison (`errors[].path`, null
  propagation, `extensions.type`) between the prefetched and direct failure
  paths, with any intentional difference stated explicitly;
- the **guard** as implemented: its exact placement in the request path, the
  public Juniper APIs it calls, how it expands named and inline fragments, how it
  handles a genuinely undecidable condition, and confirmation that it does not
  replace `GraphQLRequest::execute` and makes no authorization decision;
- the **baseline eligibility gate** as implemented: the exact pinned juniper
  entry points called, in what order, the evidence that they match
  `execute`/`execute_sync`'s own pipeline, and confirmation that no private
  field, `unsafe`, raw-source manipulation or second GraphQL implementation was
  used;
- the **baseline-invalid matrix results**: for each failing gate, the guarded
  path compared against ordinary juniper with no guard present, showing identical
  externally visible error and HTTP status, no guard rejection, no observation
  event, and zero resolvers/writes;
- the **overhead** stated accurately: parse and operation selection on **every**
  request in `OBSERVE`/`ENFORCE`, plus document and input validation on mutations
  only, with confirmation that `OFF` short-circuits ahead of all of it. Neither
  "one extra parse" nor "bounded to mutations" may appear;
- the **preview performance evidence** of section 11.3, with the `OFF` vs
  `OBSERVE` comparison for representative queries and mutations, and the approved
  thresholds — or, if no authoritative baseline exists, an explicit
  `BLOCKED FOR PRODUCTION ACTIVATION - MONITORING / THRESHOLDS UNVERIFIED`
  statement;
- the **juniper API-surface characterisation**: which required surfaces are
  `doc(hidden)` on pinned 0.16.2, and confirmation that the implementation fails
  closed if they change. The phrase "stable public API" must not be used for
  them;
- the **runtime-operations status** (section 11.7): which of the ten activation
  questions are answered, which remain open under CG-13, and an explicit
  statement that rollback timing and mechanism are unverified until they are.
  Rollback must not be described as certain or deploy-free;
- the **effective-variable construction** as implemented: how operation
  `variable_definitions` defaults are read, how request variables override them,
  and the evidence that this matches the pinned executor's `or_insert`
  construction. State explicitly that raw request variables are not used;
- the **guard mode** as implemented: the three states, that the default and
  merged state is `OFF`, and — with code evidence, not assertion — that store
  availability is **derived** from the mode so `guard OFF + store enabled` and
  `guard OBSERVE + store enabled` are unrepresentable;
- the `OBSERVE` behaviour evidence: that a would-be-rejected document executes
  with an unchanged response and unchanged resolver counts, and produces exactly
  one observation event;
- the observation/rejection **event content**, asserted positively and
  negatively, confirming no document, variables or argument values are recorded;
- the guard's **measured** zero-execution evidence: resolver call counts and
  database write counts for every rejected case — direct duplicate, named
  fragment duplicate and inline fragment duplicate — under both execution paths;
- the guard's directive matrix results, showing which duplicates are accepted and
  which rejected, and stating the conservative-rejection tradeoff explicitly
  rather than implying exactness;
- the observed HTTP status and serialized body of a guard rejection, compared
  side by side with a real juniper validation failure;
- confirmation that queries, non-top-level selections and distinct aliases are
  unaffected, and that none of the 88 `MutationRoot` resolvers was modified;
- the mode configuration as implemented, and the evidence that outside `ENFORCE`
  the store is unavailable and every path falls back — that is, that
  `guard OFF + store enabled` and `guard OBSERVE + store enabled` are
  unrepresentable;
- an explicit, accurate **production activation boundary** statement, recording
  all three states separately:

  ```text
  merge state:
    OFF, store unavailable, no request-path overhead,
    production request acceptance unchanged

  OBSERVE:
    separate explicit CTO production authorization
    runtime-operations prerequisite satisfied (section 11.7)
    monitoring/threshold prerequisite satisfied (section 8.3)
    preview/staging evidence satisfied (section 11.3)

  ENFORCE:
    separate second explicit CTO production authorization
    OBSERVE compatibility evidence passed
    OBSERVE operational-health evidence passed
  ```

  The report must **not** describe CTO merge authorization as activation
  authorization for either transition, must **not** present `ENFORCE` as the only
  activation, and must **not** treat approval of one activation as approval of
  the other. Live approval and activation evidence belongs to the relevant GitHub
  or release record under `ADR-0005`; **no approval identifier is to be committed
  to this repository**;
- confirmation that the section 8.2 observability and section 8.3 runbook
  obligations are met, and that the previous "Required logs: none" statement no
  longer applies to the guard;
- an explicit statement that the foundation declares **no** production field N+1
  compliant, and that adoption-coverage obligations attach to adopting tasks;
- the generated-SDL comparison result;
- confirmation that `schema.rs`, `migrations/`, `Cargo.toml`, `Cargo.lock` and
  `policy.rs` are unchanged;
- confirmation that no existing child resolver was modified;
- the reviewed conclusion that no migration was required.

---

## 15. Recommended execution

Implementation model: strongest available Codex coding model
Reasoning level: HIGH, using maximum practical reasoning for the store lifetime,
load-shape normalization, partitioning, failure-state and coherence semantics.
Load-shape normalization warrants the highest care: it is the one part of this
design whose failure returns confidently wrong data rather than a miss, because
the correctness fallback does not cover a shape collision.
Independent reviewer: strongest available Claude model, per `model-selection.md`
section 3 (different model family from the implementer)
Review reasoning level: HIGH

Independent cross-model review is mandatory. Explicit CTO merge authorization is
mandatory for this HIGH-risk runtime implementation. Production deployment
remains a separate, separately authorized event.

---

## 16. Branch and integration plan

- branch source: freshly verified `develop`
- pull-request target: `develop`
- task branch: `feature/shared-architecture/graphql-batching`
- expected merge order: after `ADR-0006` is approved and reachable from
  `develop`; before any `BE-02` runtime implementation
- parent programme branch refresh requirement: not applicable (STANDARD
  workflow)
- branch deletion after merge: YES
- final programme PR required: NO
- final release path: `develop -> master`

The implementation branch must not be created before explicit CTO implementation
authorization.

---

## 17. Approval

Approved for implementation by: not yet approved
Date: not yet approved
Notes: this specification requires independent review and explicit CTO approval,
and it depends on `ADR-0006` being approved and repository-authoritative.
Approval of this specification is not implementation authorization; that is a
separate explicit decision bound to a freshly verified exact `develop` head.

Implementation authorization is not merge authorization.

Merge authorization is not **production activation** authorization:
`OFF -> OBSERVE` requires explicit CTO production activation approval, and
`OBSERVE -> ENFORCE` requires a second, separate explicit CTO production
activation approval.

```text
merge authorization
!=
OBSERVE activation authorization
!=
ENFORCE activation authorization
```

Neither activation is authorized by merge approval or by the other activation.
See section 11.2.

Record only the durable implementation authorization here. Independent review
decisions, CTO merge authorization and the merge itself are terminal GitHub
evidence under `ADR-0005` and must not be copied back into this file.

---

## 18. Relationship to `BE-02`

`BE-02` is a **dependent** of this task, not a dependency of it.

```text
ADR-0006 approved + repository-authoritative
        |
        v
THOTH-GQL-BATCH-01 specification approved
        |
        v
explicit CTO implementation authorization on a freshly verified exact base
        |
        v
implementation + CI/tests + fresh independent exact-head review
        |
        v
CTO merge authorization
        |
        v
merge  --  guard mode OFF, store unavailable, request acceptance unchanged,
           no request-path overhead
        |
        v
runtime-operations evidence for mode control verified (11.7 / CG-13)
        |
        v
service-health signals and activation thresholds verified (8.3)
        |
        v
preview/staging acceptance, incl. performance evidence and a
rehearsed, timed rollback (11.3)
        |
        v
explicit CTO production activation approval for OFF -> OBSERVE
        |
        v
OBSERVE window -> compatibility AND operational evidence reviewed (11.4)
        |
        v
separate explicit CTO production activation approval for OBSERVE -> ENFORCE
        |
        v
ENFORCE + post-activation observation period
        |
        v
BE-02 specification amended on its existing PR #788 to replace the open
N+1 architecture gate with the approved mechanism and dependency
        |
        v
fresh independent exact-head review of PR #788
        |
        v
explicit CTO approval of the BE-02 specification
        |
        v
fresh develop verification + separate CTO BE-02 implementation authorization
```

Nine distinct gates, which must never be conflated: ADR approval, specification
approval, implementation authorization, merge authorization, runtime-operations
verification (CG-13), monitoring/threshold verification, **CTO `OBSERVE`
activation**, **CTO `ENFORCE` activation**, and store adoption by `BE-02`. Both
activations require their own explicit CTO production approval; neither is
implied by merge authorization or by the other.

**Where `ENFORCE` sits relative to `BE-02`, at the safest reading of the
repository gates.** Two questions were considered separately rather than
collapsed:

- *may `BE-02` runtime implementation begin before `ENFORCE`?* The conservative
  answer is taken: **no**. Authorizing implementation against a mechanism whose
  production activation has not been evidenced would mean building on a
  foundation that may still be rolled back to `OFF`. `BE-02` implementation
  authorization therefore requires `ENFORCE` activation completed and its
  observation period passed;
- *may `BE-02` reach production before `ENFORCE`?* Categorically **no**, and
  structurally rather than procedurally: outside `ENFORCE` the store is
  unavailable, so an adopting field would silently take its direct fallback and
  its N+1 compliance claim would be false.

The store must **not** be activated merely because `BE-02` later adopts it.
Adoption is not activation, and activation carries its own CTO approval.

This task must not implement, prepare or anticipate any part of `BE-02`, and
must not modify PR #788 or its branch. `BE-02` runtime implementation remains
blocked and unauthorized throughout.

### 18.1 What `BE-02` inherits, and what it must do itself

`BE-02` inherits from this foundation:

- the mechanism, the three-state store, the load-shape contract and the
  measurement approach;
- the recorded fact that `Publisher.distributionPlatforms` has **no** field
  arguments in its approved contract, so its load shape is `Unit`
  (`ADR-0006` section 4.4.5). This specification adds no argument to that field
  and changes nothing in the approved `BE-02` API contract;
- proof that several prefetch sites can cover one `(scope, loader, shape)` under
  one execution scope without duplicate loading, and that the same sites under
  distinct scopes are correctly isolated;
- the central mutation request guard, which protects `BE-02`'s mutation-payload
  paths and which `BE-02` must not reimplement or weaken. `BE-02` inherits it
  only in its **`ENFORCE`** state; it must not assume enforcement is active, and
  must not itself change the guard mode.

`BE-02` must do for itself, and this task must **not** attempt on its behalf:

- the exact-base fan-out path inventory for `Publisher.distributionPlatforms`
  required by `ADR-0006` section 4.18.2, searching its own implementation base
  rather than treating the minimum investigation set in `ADR-0006` section 4.18.3
  as exhaustive;
- installing a prefetch site on every material fan-out path, or escalating if
  compliant coverage would need architecture outside its approved scope;
- per-path statement-count evidence.

The foundation therefore leaves `BE-02` a mechanism and an obligation. It does
not leave it a compliance claim.
