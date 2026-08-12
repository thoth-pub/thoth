# Publisher Services Decision Summary

Status: ACTIVE SUMMARY
Last updated: 2026-08-12 (BE-02 closed as an inactive merged foundation; BE-03/BE-04/APP-01 phase boundary raised as a specification candidate under a durable authority condition, including the APP-01 reconciliation)
Owner: CTO

This file summarizes decisions. The approved technical design and approved ADRs remain authoritative.

## 1. Settled product decisions

### Packages

The package enum is:

```text
OASIS
OBELISK
SPHINX
PYRAMID
```

Every publisher has exactly one package.

OASIS is the non-null default.

Publisher users may read their own package and its effective capability codes. Only superusers may change the package. Package and capability values are not anonymous public data.

Package choice does not itself enable or disable distribution platforms.

### Distribution assignments

Every independently meaningful destination has its own `DistributionPlatform` value.

Assignments are explicit publisher configuration.

Disabled rows are retained for history. Re-enabling creates a new activation identity.

OAPEN and DOAB are separate destinations but initially form one linked selection. They map to one logical delivery adapter and must not upload twice.

### Licence authority

`thoth-pub/cc-license` is authoritative for supported Creative Commons licences and retained public-domain tools.

Thoth must not independently parse or normalize Creative Commons URLs.

A missing licence is `NULL` and means no declared open licence/All Rights Reserved.

### OAI-PMH eligibility

OAI-PMH is deferred.

A work is eligible only when:

1. the publisher has the approved OAI package capability;
2. the work has a licence recognized as open by `cc-license`;
3. existing lifecycle and metadata requirements pass.

OASIS is excluded. Non-open works are excluded for every package.

### Desired, job and observed state

Keep separate:

- desired publisher configuration;
- durable execution jobs;
- future observed per-work/per-platform delivery state.

This programme initially implements desired state and durable publisher back-catalogue jobs only.

## 2. Shared decisions

### ADR-0001 - Package capability model

Status: `APPROVED` (Javi, CTO, 2026-07-28, approval PR
[#772](https://github.com/thoth-pub/thoth/pull/772)).

Approved architecture:

- code-owned exhaustive capability mappings;
- Thoth ownership and stable GraphQL capability codes;
- no database capability rows or bespoke publisher overrides;
- entitlement remains separate from source accounts, credentials and other
  operational configuration;
- retained metrics visible after an entitled upgrade;
- no automatic historical OPERAS bulk export after upgrade;
- package changes use the resulting package's capabilities, and downgrades
  retain canonical metrics;
- package changes never alter distribution assignments.

The approved package matrix is:

| Package | OAI_PMH | METRICS_COLLECT | METRICS_IMPORT | METRICS_DASHBOARD | METRICS_WIDGET | METRICS_OPERAS_EXPORT |
|---|---:|---:|---:|---:|---:|---:|
| OASIS | No | No | No | No | No | No |
| OBELISK | Yes | Yes | No | No | No | No |
| SPHINX | Yes | Yes | Yes | Yes | Yes | Yes |
| PYRAMID | Yes | Yes | Yes | Yes | Yes | Yes |

OASIS is not entitled to Thoth-managed metrics collection. Under current
operations Thoth has no managed OASIS usage-data source because it does not
operationally distribute OASIS files. This metrics-entitlement decision does not
disable or remove OASIS distribution-platform assignments, prevent superuser
platform configuration, define dissemination eligibility, create a distribution
capability or change distribution-job behaviour. Any permanent OASIS
distribution prohibition requires a separately approved decision through ADR-01
or another cross-programme ADR.

Metrics collection must not infer entitlement from a distribution assignment or
remote location. OBELISK collection is private, requires valid source
credentials and source-specific configuration, and must not block distribution,
metadata, package changes or unrelated publisher services when configuration is
missing or a source fails.

Package changes use the resulting package's capabilities:

- `PYRAMID -> SPHINX` removes no initial capability;
- `SPHINX` or `PYRAMID -> OBELISK` retains OAI-PMH and configured private
  collection while denying publisher import, dashboard, widget and OPERAS
  export;
- any package `-> OASIS` denies all six initial capabilities and stops
  Thoth-managed collection;
- every downgrade retains canonical metric history, leaves distribution
  assignments unchanged and rechecks the relevant capability at the final
  boundary.

Approval settles the shared architecture but does not start or complete `BE-01`,
metrics entitlement work or `OAI-01`. Each remains subject to its own approved
bounded specification and other tracker dependencies.

### ADR-0002 - Platform domain boundaries

Status: `APPROVED` (CTO, 2026-07-27, approval PR
[#769](https://github.com/thoth-pub/thoth/pull/769)). Approved as written.

Approved architecture:

- `DistributionPlatform` and `MetricPlatform` remain separate types;
- no name-based conversion;
- no initial cross-domain mapping table;
- OAPEN/DOAB linkage exists only in the distribution domain unless separately approved.

Approval removes one dependency; it does not make `ADR-01`, `BE-02` or the
metrics platform registry ready.

Implementation dependency:

- Publisher Services `ADR-01` and `BE-02`
- metrics platform registry.

## 3. Decisions delegated to Publisher Services ADR-01

State (2026-08-07): the separately authorized ADR-01 implementation produced
[ADR-0004](../engineering/decisions/ADR-0004-distribution-platform-inventory.md)
(`APPROVED AND REPOSITORY-AUTHORITATIVE`), the complete
[evidence matrix](adr-01-evidence-matrix.md) and the
[final inventory](platform-inventory.md) (`FINAL INVENTORY APPROVED AND
REPOSITORY-AUTHORITATIVE`; 17 values; no `OTHER`; `JISC_NBK` included but
inactive and non-assignable; 10 recorded exclusions). ADR-0004 and the final
inventory were independently reviewed at exact head
`44e6f821535fbee56c830dd6eda237fc6d06fbfd` (review `4881233664`,
`APPROVED`) and explicitly CTO-approved (review `4881279067`, 2026-08-07);
the approval-state head `82874c2bfb0c211198252e4f4a0b669d31e14836` received
final independent review `4881832108` (`APPROVED`) and CTO merge
authorization `4881847699`; and
[PR #783](https://github.com/thoth-pub/thoth/pull/783) merged into `develop`
as `299b0eff3b9ac10cc0a3a7024ab311ddb135b7eb` on 2026-08-07T10:02:34Z,
making the decision repository-authoritative. ADR-01 is `MERGED - COMPLETE`;
it was an evidence and architecture-decision task and is not runtime
implemented and not production ready. Post-merge control reconciliation is
delivered by
[ADR-01-CLOSEOUT-01](../engineering/ai-delivery/tasks/ADR-01-CLOSEOUT-01.md).
`BE-02` consumed that decision under its own approved bounded specification and
separate explicit implementation authorization, and is `CLOSED`: the closed
17-value `DistributionPlatform` enum and the code-owned descriptors were
implemented exactly from this final inventory and merged through
[PR #805](https://github.com/thoth-pub/thoth/pull/805) as an inactive additive
foundation. No destination is activated and no distribution occurs.

The decisions delegated to ADR-01, all now settled by ADR-0004, were:

1. complete user-visible distribution destination inventory;
2. stable enum codes and display names;
3. current uploader, feed or manual mechanism;
4. linked groups;
5. `AutomaticPush`, `PullFeed` or `Manual` behaviour;
6. whether back-catalogue activation creates a job;
7. update and withdrawal support;
8. credential ownership model;
9. current publisher-configuration source;
10. whether the following are separate destinations:
   - Google Books and Google Play;
   - EBSCO KB and EBSCOHost;
   - multiple ProQuest products/destinations.

No `OTHER` enum value is permitted.

### Approved amendment inputs - repository-authoritative

`ADR-01-SPEC-AMEND-01` (2026-08-06) recorded the following evidence-based
inputs to those delegated decisions, supported by the
[ADR-01 evidence ledger](adr-01-evidence-ledger.md) and explicit CTO
decisions of 2026-08-06. The corrected specification content carrying them
was independently reviewed (review `4873802457`, `APPROVED`) and explicitly
CTO-approved (comment `5203642323`, 2026-08-06) at exact content head
`1276c70a81e73f57d833eecb0e6886bd0cabf69e`, and became
repository-authoritative through the merge of
[PR #781](https://github.com/thoth-pub/thoth/pull/781) (merge commit
`a511e01c83c5e805a75e0fdaeb3b5297c39ef291`, 2026-08-06T11:29:53Z, under CTO
merge authorization review `4874128610`). ADR-01 subsequently produced
ADR-0004 and the final inventory under its separately authorized
implementation task, merged through PR #783; no runtime implementation is
authorized:

- a `DistributionPlatform` destination is distinct from the delivery adapter
  or feed profile serving it; shared adapters/feeds must not create duplicate
  files, feeds, deposits or uploader jobs, and no second overlapping public
  business enum is introduced;
- Google Books and Google Play Books are one destination (display name
  `Google Play Books`; `Google Books` a legacy alias; one stable enum code);
- `EBSCO_HOST` is a confirmed current push destination; `EBSCO_KB` is a
  distinct product excluded from the initial enum as currently unverified;
- the canonical ProQuest push destination is `PROQUEST_EBOOK_CENTRAL`
  (aliases `ProQuest`, `Ebrary`, `proquest_ebrary`); `EX_LIBRIS_KB` is a
  separate pull-feed consumer sharing `OCLC_KBART_PUBLIC` with `OCLC_KB`;
  `PROQUEST_SERIALS_SOLUTIONS_KB` is excluded as currently unverified;
- the Jisc destination is `JISC_NBK` (MARC21 via S3, adapter
  `JISC_NBK_MARC_S3`), included but initially inactive and non-assignable;
- `OVERDRIVE`, `BDS_LIVE`, `RNIB_BOOKSHARE`, `SCIELO_BOOKS` and `ZOTERO` are
  excluded from the initial inventory by CTO decision; `THOTH` and
  `PUBLISHER_WEBSITE` remain location concepts, not distribution values;
- incremental updates and withdrawals follow a conservative initial policy
  (Crossref DOI deposit updates supported; all other automatic updates and
  all automatic withdrawals disabled pending separately approved work);
- automatic push jobs may use only Thoth-managed publication files, failing
  closed otherwise (implementation deferred to a separate HIGH-risk task);
- commercial entitlement authority is the publisher Statement of Work;
  runtime desired-state authority is the Thoth publisher/platform
  assignment; the COO is the accountable operational owner and the Metadata
  Specialist the operationally responsible and target credential owner, with
  shared credential responsibility recorded honestly during transition;
- the Project MUSE scheduled-workflow key mismatch is historical/resolved,
  not a current defect; the ProQuest EPUB-only/PDF-ISBN ordering defect
  remains a current recorded defect.

## 3a. Programme decision - BE-03 / BE-04 / APP-01 phase boundary

Decision state: `PROPOSED IN THIS SPECIFICATION CANDIDATE`
Raised by: `BE-03-SPEC`
Decision owner: CTO

**Authority condition.** This decision becomes approved and
repository-authoritative when **both** of the following hold:

1. the exact `BE-03-SPEC` content containing this decision receives explicit CTO
   specification approval; **and**
2. that exact approved content is reachable from `develop`.

Before both conditions hold, this decision is **NOT AUTHORITATIVE FOR
IMPLEMENTATION**. After both hold, it is an **APPROVED PROGRAMME DECISION** for
BE-03 implementation purposes, **without requiring a separate lifecycle-status
edit to this file**.

This is the durable ADR-0005 form deliberately. A mutable literal `APPROVED`
status word would have to be written by a further commit after approval, which
would move the head the approval was bound to and produce exactly the
approval-state-only churn ADR-0005 section 4.1 item 10 prohibits — and, until
that commit landed, BE-03's stop condition would report a false block against
its own approved specification. GitHub remains the terminal evidence for the
exact-head approval and merge lifecycle; it is not transcribed here.

[`BE-03.md`](../engineering/ai-delivery/tasks/BE-03.md) stop condition 5 states
this same rule in the same terms, and the two must be read as one condition.

### The tension

The approved design's API section says that
`replacePublisherServiceConfiguration` creates the required jobs, and that the
staff report includes back-catalogue and job state. The approved task
decomposition says something different: BE-03 owns protected service
configuration, audit, authorization and concurrency, while BE-04 owns the job
table, job target, job attempt, job-creation rules, the worker role, leases and
the claim/complete/fail/retry/cancel lifecycle. The rollout additionally holds
automatic job creation inactive initially.

The same contradiction reaches APP-01. The approved design makes APP-01 depend on
BE-03 **and** lets superusers inspect back-catalogue status. Once durable job
state is correctly deferred to BE-04, those two statements cannot both hold of a
BE-03-only dependency: there is no durable source for job, attempt, failure or
pending-onboarding state until BE-04 exists.

BE-03 cannot satisfy both readings, and guessing would either smuggle BE-04's
schema into BE-03 or fabricate job state that has no durable source. The
boundary is therefore surfaced explicitly for decision rather than resolved
silently.

### Proposed resolution

```text
BE-03 owns DESIRED CONFIGURATION only.

BE-03 does NOT create distribution_job,
distribution_job_target or distribution_job_attempt rows.

BE-03 does NOT create placeholder/pseudo jobs.

BE-03 does NOT expose fabricated job status.

BE-04 owns durable job persistence and job-creation rules.

BE-04 will later extend the same configuration-change transaction boundary so
that assignment activations requiring onboarding can create durable jobs
atomically with the desired-state change once the BE-04 schema exists.

Until BE-04 is implemented and separately authorized:
configuration changes create no upload/back-catalogue job and trigger no
dissemination.
```

**One factual note on that shared transaction boundary**, so BE-04 is planned
against the real transaction rather than a single-row model. `public.publisher`
carries an existing `AFTER UPDATE` trigger,
`set_work_updated_at_with_relations`, which refreshes
`work.updated_at_with_relations` for every work of that publisher through its
imprints. Because the canonical configuration version token is a `publisher`
column under BE-01's approved package storage, a committed configuration change
fires that trigger, and BE-04's extension of the same transaction inherits the
resulting row-lock footprint, transaction duration and downstream freshness
effect. This does not change the ownership boundary above and is not distribution
activation — no job, upload or dissemination is created by it. It is specified,
measured and evidenced under
[`BE-03.md`](../engineering/ai-delivery/tasks/BE-03.md) sections 2.1 item 8, 6.4,
7.8, 7.9, 18.4 and stop condition 19.

### APP-01 reconciliation

This decision candidate **refines and, in that narrow respect, supersedes** the
earlier APP-01 wording that assigned superuser back-catalogue-status inspection
to a BE-03-only dependency. Nothing else in the approved APP-01 record is
changed: APP-01 remains a `thoth-app` task, remains MEDIUM risk, and remains
blocked on BE-03 exposing the approved protected API, app readiness controls, the
exact-SHA schema pinning control and its own approved bounded specification.

Scope available from **BE-03 alone** — the BE-03-dependent part of APP-01:

1. publisher users read their own package, effective capability codes and
   enabled-platform configuration;
2. superusers read **and edit** package and enabled-platform configuration, and
   read any publisher's effective capability codes;
3. capability-driven UI affordances, subject to ADR-0001 section 4.5 (hiding a
   control is not authorization) and section 4.6 (a capability permits a feature
   without configuring or activating it);
4. linked-platform UI behaviour driven by backend metadata rather than duplicated
   frontend rules;
5. optimistic-concurrency handling, including a distinct stale-configuration
   error the UI can render as "configuration changed; reload";
6. server-normalized state replacing local state after a successful mutation.

**Not** available from BE-03 alone, and therefore requiring **BE-04**:

- durable back-catalogue job status;
- attempt state;
- failure state;
- pending onboarding state.

Any APP-01 element that renders those four is BE-04-dependent, not
BE-03-dependent. `APP-02` remains dependent on **both** BE-03 and BE-04 and
remains the full job-aware staff report and CSV surface. BE-04 work must not be
pulled forward into BE-03 to preserve obsolete task wording, and BE-03 must not
fabricate job state to fill the gap.

If the CTO does not accept this refinement, BE-03 is blocked: APP-01's
back-catalogue-status expectation would then require the BE-04 job schema to be
specified, approved and authorized first.

This reconciliation concerns **job state only**. It does not narrow the
protected configuration surface itself: under ADR-0001 section 4.4 that surface
comprises the current package, the effective capability codes and the enabled
distribution platforms, and BE-03 provides all three. Capability exposure is
therefore not part of this proposed decision and needs no decision — it is
already-approved ADR-0001 architecture, which
[`BE-03.md`](../engineering/ai-delivery/tasks/BE-03.md) section 10.1 follows.

### Consequences

1. The BE-03 staff report exposes publisher, package, enabled platform state,
   configuration version and latest configuration-change metadata. It exposes
   no job status, no back-catalogue job or attempt state and no
   pending-onboarding state, and it invents no `NOT_STARTED` or `UNKNOWN`
   placeholder to stand in for them.
2. BE-04 adds those fields additively to the same report. BE-03 must not depend
   on job tables that do not yet exist.
3. `APP-01`'s **configuration** scope depends on BE-03 alone; its **job-aware**
   elements depend on BE-04, per the APP-01 reconciliation above. `APP-02`
   depends on **both** BE-03 and BE-04 for its final job-aware report.
4. BE-03 must keep its configuration-change transaction boundary explicit and its
   steps separable, so BE-04 can extend that same transaction rather than adding a
   second one. BE-03 adds no hook, callback, event or placeholder for it. The
   single service-configuration write coordinator BE-03 specifies is deliberately
   the one place BE-04 will extend.
5. This is consistent with operational invariants 3 and 7 below: backfill
   creates no back-catalogue jobs, and automatic job creation is initially
   inactive. The controlled MIG-01 backfill must nevertheless commit its
   configuration changes through the same write coordinator, so backfilled
   configuration is version-tracked and audited exactly like an API change, while
   still creating no job and triggering no dissemination.

### Boundary of this decision

It settles a phase boundary between tasks in one programme and one repository,
and refines one narrow clause of the earlier APP-01 wording. It changes no
approved architecture, introduces no cross-programme abstraction and no shared
component, and therefore does not require its own ADR. The single-coordinator
rule is an implementation-level control internal to this programme's backend
tasks, recorded in the BE-03 specification rather than as shared architecture.

If the CTO instead decides that BE-03 must create durable jobs, BE-03 is blocked
until the BE-04 job schema is separately specified, approved and authorized.

This decision is settled by the authority condition stated at the head of this
section: explicit CTO specification approval of the exact `BE-03-SPEC` content
carrying it, and that exact content reaching `develop`. No agent may declare it
approved, and no further repository edit is required to record approval once
both conditions hold.

## 4. Operational invariants

1. API outages fail closed.
2. An empty publisher/platform assignment is a successful no-op, never "all".
3. Backfill of existing assignments creates no back-catalogue jobs.
4. One linked activation creates one logical multi-target job.
5. Pull-feed and manual destinations create no uploader job.
6. Claims are leased and stale claim tokens cannot complete current work.
7. Automatic job creation is initially inactive.
8. Comparison mode must be clean before legacy configuration cutover.
9. Production activation requires a pilot, monitoring, rollback and CTO approval.
10. Legacy configuration remains available through the observation period.

## 5. Future decisions explicitly deferred

- work-level `ALL_PUBLISHER_PLATFORMS`, `SELECTED_PLATFORMS`, `NONE`;
- exact work-level platform sets;
- metadata-change outbox;
- work upsert and withdrawal jobs;
- complete observed delivery state;
- delivery fingerprints and remote identifiers;
- scheduled reconciliation;
- Rust ports of uploaders;
- publisher-managed service changes;
- package-to-platform defaults.

These must reuse the approved package, platform, adapter, audit and job foundations.
