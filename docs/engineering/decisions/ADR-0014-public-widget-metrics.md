# ADR-0014 - Public, capability-gated widget metrics

Status: APPROVED
Date: 2026-11-10
Approved by: Javi, CTO
Approval date: 2026-11-10
Decision owner: CTO
Programmes affected: Thoth Metrics; Publisher Services (package capabilities)
Repositories affected: `thoth`, `metrics-widget`; `infrastructure` at activation (edge rate limit and server switch configuration)
Supersedes in part: `ADR-0001` section 4.4 and section 4.5 as they apply to widget metrics, and the `METRICS_WIDGET` row of its normative appendix `package-capability-matrix.md`
Superseded by: None

Decision: Thoth serves one bounded, read-only, work-level Metrics read, `metricWidget`, to anonymous callers through the public GraphQL API, for works whose publisher's package grants `METRICS_WIDGET`. Protection comes from data minimisation, uniform refusal and cost bounds, not from secrecy: a served response discloses, by design, that the publisher holds `METRICS_WIDGET`. Every other rule of `ADR-0001` is unchanged.

Authority condition: this ADR becomes repository-authoritative only when this exact approved version has received independent exact-head review and is reachable from `develop`. It decides architecture only. It authorizes no implementation, schema change, migration, deployment, release or activation. `metricWidget` is specified, reviewed and authorized separately (`MET-WP4-WIDGET-01`, thoth-pub/thoth#1007), and it ships inactive behind a server switch whose activation needs its own grant per environment.

---

## 1. Context

The `metrics-widget` npm package is embedded on third-party book pages and runs entirely in readers' browsers. It reads the OPERAS metrics API today, anonymously, and must move to Thoth Metrics.

`ADR-0001` (approved 2026-07-28, repository-authoritative) assumed a protected widget path:

- section 4.4: "Anonymous users and users scoped only to another publisher cannot read package or capability values."
- section 4.5: capabilities are enforced at "protected dashboard/widget metrics queries".
- Before this decision the capability matrix defined `METRICS_WIDGET` as "A Thoth-owned authenticated service may serve bounded work-level widget metrics".
- `docs/metrics/README.md` routed dashboard and widget browser clients through a Thoth-owned server route that holds a machine credential, as the Metrics contract register still does for the dashboard read.

A server route that the widget can call from any browser, with no user credential, is itself an anonymous public endpoint. A machine credential behind it adds a service to run and a secret to rotate without changing who can read the data. On 2026-11-10 the CTO chose a public `metricWidget` query in Thoth instead, and approved its specification decisions on thoth-pub/thoth#1007.

`docs/engineering/decisions/README.md` requires a new superseding ADR for a material change to an ADR that has been repository-authoritative. Letting anonymous callers read capability-gated metrics, and so infer one capability, is such a change.

## 2. Decision drivers

- The widget runs in third-party pages and can never hold a credential.
- Readers already see usage numbers like these today, through OPERAS, without authentication; public sales figures would be new (section 4.5).
- One authorization point: capability checks stay centralized in Thoth (`ADR-0001` invariant 1).
- Publisher state that is not meant for readers (package, import health, configuration, institutions) must stay private.
- An anonymous read must not be able to exhaust the shared API (database pool, workers).
- No new service or secret if a simpler shape gives the same security.

## 3. Options considered

### Option A - Thoth-owned server route with a machine credential

The widget calls a Thoth-owned service that holds a `METRICS_READ_SERVICE` credential and calls `metricDashboard`.

- The route must accept anonymous browser calls, so it is a public endpoint with the same exposure.
- It adds a deployed service, a long-lived secret and a second place to enforce bounds.
- The dashboard read's bounds and response (warnings, institutions, publisher-wide coverage) are wider than a public widget needs, so the route would have to re-filter them.

### Option B - Public, capability-gated `metricWidget` in the Thoth API (chosen)

A dedicated read with its own narrow shape and bounds, gated by `METRICS_WIDGET` inside the operation, reusing the dashboard engine's semantics per served platform and measure.

- No credential anywhere; one enforcement point.
- The capability is observable from a served response.
- It needs explicit cost controls because anyone can call it.

### Option C - Per-publisher embed keys

Each publisher embeds a key in its pages. Any embedded key is public, so it protects nothing and adds key issuance and rotation.

### Option D - Pre-rendered static metrics files

A job publishes per-book JSON to a CDN. Cheapest to serve, but it needs a generator, storage and freshness rules, and duplicates the read semantics. Not chosen now; it remains possible later behind the same public contract.

## 4. Decision

### 4.1 Public widget read

Thoth may serve `metricWidget` to anonymous callers through its public GraphQL API. It is read-only, work-level (one work, named by DOI, and the child works that Thoth records for it; a child work of another publisher is never served), bounded, and serves only measures the Metrics registry marks public. It serves no institution data, no warnings and no package value. Its exact contract is specified by `MET-WP4-WIDGET-01` and recorded in the Metrics contract register when that stage merges.

### 4.2 Meaning of `METRICS_WIDGET`

`METRICS_WIDGET` means: Thoth may serve bounded, public, work-level widget metrics for this publisher's works to anonymous callers through the public API.

This replaces the `METRICS_WIDGET` row of `package-capability-matrix.md`, which `ADR-0014` owns from this decision on. The package mapping in `ADR-0001` section 4.2 and the capability set in section 4.3 are unchanged.

### 4.3 Disclosure (supersedes `ADR-0001` section 4.4 for widget metrics)

While the read is switched on, any caller can learn whether a work's publisher holds `METRICS_WIDGET`, whether or not the work has metrics: a served `metricWidget` response discloses it, and so does any refusal that only an entitled publisher's work can reach, such as a refusal of the caller's child-work selection. Whether the switch is on is not secret either; a served response or an input error shows it. Under the package mapping in `ADR-0001` section 4.2, holding `METRICS_WIDGET` also means that the publisher's package is `SPHINX` or `PYRAMID`, though not which. All of this is accepted by design.

Thoth still never returns a package or capability value to any caller of `metricWidget`. The refusals that would otherwise say why a work has no widget metrics (switch off, unknown work, publisher without `METRICS_WIDGET`, and the publisher-state Metrics refusals that `MET-WP4-WIDGET-01` maps to it) are one uniform error that does not say which condition failed. Input errors, bound refusals (including a work too large for the read's bounds), busy refusals and internal errors are separate classes; none carries a package or capability value, SQL or configuration. `ADR-0001` section 4.4 continues to govern every other read of package or capability values.

### 4.4 Enforcement (supersedes `ADR-0001` section 4.5 for widget metrics)

The widget read is public and capability-gated: Thoth checks `METRICS_WIDGET` inside the operation, with no role. No user or service authorization is required for it, so rule 3 of `package-capability-matrix.md` section 5 asks nothing of the caller; rules 1, 2 and 4 apply unchanged (the package grant, the registry's public and enabled flags, and the server switch). For the widget only, this also replaces the server-route client model of the approved Metrics design, `docs/metrics/README.md` and the Metrics contract register. Dashboard reads remain protected (`METRICS_READ_SERVICE` and `METRICS_DASHBOARD`), and browser clients of the dashboard still reach Thoth only through a Thoth-owned server route.

### 4.5 Public scope

- Only measures the registry marks public, enabled and additive are served, on enabled platforms. A public sales measure may be served.
- Before activation in any environment, the CTO signs off a recorded audit of every measure's public flag and every enabled platform in that environment, stating each sales measure left public and, per entitled publisher, the basis for publishing its sales. A later registry change that makes a measure public or enables a platform where the widget is on needs the same sign-off. A package change that grants `METRICS_WIDGET` to a publisher in an environment where the widget is on and a sales measure is public needs the same recorded basis for that publisher first, unless the audit records the package terms as the basis.
- No institution data and no warnings. The only import or configuration state served is each served pair's coverage (status, data-through date and whether it has country coverage), each section's data-through date and partial-data flag, and the rollup watermark.

### 4.6 Cost and abuse

Because anyone can call it, the read is bounded per process (a small fixed number of concurrent widget reads, separate from any dashboard guard), per statement (a short timeout) and per request (one widget read per HTTP request, and a deadline), and it never truncates. Its date range, child-work count and pair count are bounded. Activation additionally requires at least an edge rate limit and either a shared response cache or a named CTO waiver.

### 4.7 Activation

The read ships inactive behind a server switch, off by default. Turning it on in an environment is a separate grant, classified CRITICAL unless the CTO decides otherwise at that grant.

## 5. Consequences

Positive:

- The widget needs no credential and no extra service.
- One enforcement point in Thoth, with a public shape narrower than the dashboard's.
- Readers keep seeing the metrics they see today.

Negative:

- Anyone can learn which publishers hold `METRICS_WIDGET` (under the current package mapping, which are on `SPHINX` or `PYRAMID`) and which do not, one work at a time.
- An anonymous read path exists in the API and must be watched for cost.
- Public sales figures become a publishing decision that needs a recorded basis per publisher.

Risks:

- Registry flags set carelessly would publish data; mitigated by the activation audit and the standing sign-off rule.
- Load from embeds; mitigated by the per-process limit, timeouts, the edge rate limit and the shared cache (unless waived).

## 6. Invariants created by this decision

1. The widget holds no Thoth credential, and no Metrics browser client holds a Thoth machine credential.
2. No package or capability value is returned to an anonymous caller.
3. The widget's unavailable refusal is one uniform error for switch off, unknown work, a publisher without `METRICS_WIDGET` and the publisher-state Metrics refusals mapped to it, and never says which of them applied; no widget response or error carries a package or capability value.
4. Only registry-public, enabled, additive measures on enabled platforms are served publicly; no institution data.
5. Dashboard authorization is unchanged.
6. The anonymous read is bounded and never truncates.
7. The anonymous read is off by default; activation is a separate grant per environment.
8. Public sales measures are served only after the CTO-signed audit for that environment.
9. Only works of the publisher whose `METRICS_WIDGET` was checked are served; a caller can narrow, but never add to, the child works Thoth records.

## 7. Implementation impact

This ADR's own change, documentation only (`MET-WP4-WIDGET-ADR`, thoth-pub/thoth#1008):

- this file and its `decision-register.md` row; `ADR-0001`'s register row notes the partial supersession;
- `ADR-0001`'s supersession header (`Superseded by: None in full` and a new `Superseded in part by` line naming this ADR), which the CTO records as a factual clarification naming `ADR-0001`'s prior approved blob; no other line of `ADR-0001` changes;
- `package-capability-matrix.md`: the `METRICS_WIDGET` row takes the meaning in section 4.2, and the matrix records `ADR-0014` as the normative owner and approval of that row, with `ADR-0001` remaining owner of the rest; `ADR-0001`'s approval does not cover the new row's text;
- `docs/metrics/README.md` and the repository map (`repositories/metrics-widget.md`, `contracts.md`, `repositories/thoth.md`): the widget's path is the public read, and the dashboard's stays the protected server route;
- `docs/publisher-services/decisions.md`: one sentence that a served widget response shows that the publisher holds `METRICS_WIDGET`.

Until `MET-WP4-WIDGET-01` merges, the `PublisherCapability::MetricsWidget` GraphQL description still carries the old wording; nothing is served in that window.

Implementation (`MET-WP4-WIDGET-01`, separately authorized): the `metricWidget` operation, the `PublisherCapability::MetricsWidget` GraphQL description, the contract register section, tests and the server switch.

## 8. Migration, rollout and rollback

Migration: none. This decision needs no database schema change or data migration, and `MET-WP4-WIDGET-01`, as approved at R9, adds none.

Rollout: documentation only; no runtime effect. The read itself rolls out under section 4.7.

Rollback of the decision requires a further superseding ADR. Runtime rollback of the read is the server switch (off, then restart).

## 9. Validation

- `MET-WP4-WIDGET-01` specification reviews R3 to R9 on thoth-pub/thoth#1007, each by a fresh Opus and a fresh Fable reviewer: every leg from R5 to R8 found that a new superseding ADR is the route `docs/engineering/decisions/README.md` requires (precedent: `ADR-0007`'s supersession of `ADR-0006`), and R8 and R9 were approved by both.
- This ADR's own specification review and independent exact-head review on thoth-pub/thoth#1008.

## 10. Approval

Approved by Javi, CTO, on 2026-11-10, on thoth-pub/thoth#1008, the owning stage issue of `MET-WP4-WIDGET-ADR`. The approval covers this exact text, the `package-capability-matrix.md` row edit and the matrix's split-ownership record. Repository authority follows only from independent exact-head review and merge into `develop`; `MET-WP4-WIDGET-01` implementation may rely on this ADR only after that.
