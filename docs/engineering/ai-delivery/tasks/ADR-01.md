# ADR-01 - Platform inventory and final architecture

Status: APPROVED
Programme: Publisher Services and Distribution Configuration
Repository: `thoth-pub/thoth`
Workflow: STANDARD
Base branch: `develop`
PR target: `develop`
Programme integration branch: None
Risk: MEDIUM
Owner: CTO
Approved by: Javi, CTO
Approval date: 2026-08-05
Target branch name: `feature/publisher-services/adr-01`

## 1. Objective

Determine and record the final, exhaustive user-visible distribution-platform
inventory for Thoth, together with the operational architecture each destination
requires, so that `BE-02` can implement the `DistributionPlatform` enum and its
code-owned descriptors without guessing, and so that `thoth-dissemination` can
later map every value to exactly one delivery behaviour without ambiguity or
duplicate delivery.

ADR-01 is an evidence and decision task. It produces an approved architecture
decision record and the supporting evidence matrix. It writes no runtime code,
no migration, no enum, and no API.

### 1.1 Risk rationale

ADR-01 is MEDIUM risk because it:

- performs read-only inspection across three repositories and changes no runtime
  behaviour, schema, or deployed system;
- produces a decision record whose errors are caught by `BE-02`'s own HIGH-risk
  implementation controls, independent review, and test evidence before any
  destination is acted upon;
- creates no production side effect, dispatches no workflow, and requires no
  credential.

The classification is nevertheless not LOW: an incorrect or incomplete inventory
propagates into the `BE-02` enum, the `BE-03` contract, `BE-04` job targets,
`MIG-01` production backfill, and `DIS-01`/`DIS-02` adapter routing. The
approved design records incorrect platform inventory as a programme risk whose
mitigation is that ADR-01 is a merge gate with exhaustive enum values.

MEDIUM is the current repository classification for ADR-01 in the Publisher
Services tracker. The implementing agent must not silently reclassify it.

### 1.2 Proposing a different classification

The implementing agent may propose HIGH if, and only if, direct evidence
gathered during ADR-01 shows operational risk beyond the above — for example
that an inventory decision would itself determine whether a live destination
continues to receive deliveries, or that resolving the inventory cannot be
separated from an operational change.

A proposed reclassification is a control decision, not an implementation detail.
It must be:

1. stated explicitly and early, before the decision record is finalized;
2. justified from actual observed operational risk, not from document length or
   effort;
3. accompanied by the additional controls the higher classification requires;
4. approved by the CTO before the ADR-01 PR merges.

Downward reclassification to LOW is not available.

### 1.3 Required MEDIUM-risk controls

The ADR-01 implementation requires:

- this approved specification;
- MEDIUM or HIGH implementation reasoning, selected and justified from the
  actual operational risk of the destinations being inventoried;
- an independent reviewer that did not author the inventory;
- direct inspection by that reviewer of every cited repository path;
- source-owner confirmation for every claim not present in a repository;
- exact-head CI evidence and an immutable exact-head evidence record;
- explicit CTO approval before the ADR-01 PR merges.

ADR-01 receives no runtime, production, or workflow-dispatch authorization at
any point.

Approval of this specification does not authorize the ADR-01 implementation.
That work requires its own separate explicit authorization, its own freshly
verified `develop` base, and its own branch.

## 2. Background and authority

Authoritative sources, in precedence order:

1. merged code, migrations and generated contracts;
2. [ADR-0002 - Distribution and Metrics Platform Domain Boundaries](../../decisions/ADR-0002-platform-domain-boundaries.md);
3. [ADR-0001 - Publisher Package Capability Model](../../decisions/ADR-0001-publisher-package-capability-model.md);
4. [ADR-0003 - Repository-authoritative schema contract](../../decisions/ADR-0003-repository-authoritative-schema-contract.md);
5. the approved private `Publisher Services and Distribution Configuration -
   Technical Design and Implementation Plan`, Drive revision `3`, indexed in
   [`docs/engineering/design-references.md`](../../design-references.md),
   section 8.2 of which defines the ADR-01 epic;
6. this approved specification;
7. [Publisher Services programme controls](../../../publisher-services/README.md),
   including the
   [provisional platform inventory](../../../publisher-services/platform-inventory.md);
8. [repository control gaps](../../repository-map/control-gaps.md) and the
   repository maps for
   [`thoth`](../../repository-map/repositories/thoth.md),
   [`thoth-app`](../../repository-map/repositories/thoth-app.md) and
   [`thoth-dissemination`](../../repository-map/repositories/thoth-dissemination.md).

Related tracker: [issue #765](https://github.com/thoth-pub/thoth/issues/765).
Related control gap: [CG-07](../../repository-map/control-gaps.md#cg-07---publisher-services-platform-adr-open).

### 2.1 Dependencies

| Dependency | Required state for ADR-01 implementation |
|---|---|
| P0-01 | CLOSED |
| ADR-0001 | APPROVED AND MERGED |
| ADR-0002 | APPROVED AND MERGED |
| ADR-0003 (Architecture A) | APPROVED AND MERGED |
| CG-12 | RESOLVED |
| This ADR-01 specification | Must be independently approved and merged before implementation |
| BE-01 | Not an ADR-01 dependency; ADR-01 does not read, alter or depend on the publisher package foundation |
| CG-11, CG-13, BR-APP-01 | Not ADR-01 dependencies; ADR-01 changes none of them |

ADR-01 is not blocked by `BE-01`. The two tasks are independent: `BE-01` owns
the publisher package foundation and `ADR-01` owns the distribution-platform
inventory, and the approved architecture keeps package selection and platform
assignment independent.

### 2.2 What ADR-01 unblocks and what it does not

Merging an approved ADR-01 is the precondition for `BE-02` finalizing
`DistributionPlatform`.

Merging ADR-01 does not:

- start `BE-02`, which requires its own approved bounded specification;
- authorize any migration, enum, table, API or app change;
- authorize any change in `thoth-dissemination`;
- alter `BE-01`, `CG-11`, `CG-13`, `BR-APP-01` or the deferred OAI branch.

## 3. Explicit scope

The task must:

1. inspect, read-only, the current distribution surfaces listed in section 4;
2. record a complete evidence matrix for every candidate destination, using the
   required record fields in section 5;
3. classify every claim using the evidence vocabulary in section 6;
4. resolve every provisional and ambiguous destination listed in section 8, or
   stop;
5. produce the final exhaustive inventory and the decisions listed in
   section 9;
6. write the decision record as a new ADR under
   `docs/engineering/decisions/`, using
   [the decision-record template](../decision-record-template.md), numbered as
   the next available `ADR-000N` at the implementation base;
7. replace the provisional
   [`docs/publisher-services/platform-inventory.md`](../../../publisher-services/platform-inventory.md)
   baseline with the approved final inventory, or record precisely how that file
   is superseded by the new ADR;
8. reconcile the Publisher Services tracker, `decisions.md` and rollout records
   with the approved outcome;
9. update `CHANGELOG.md` and produce the ADR-01 implementation report.

Likely changed paths:

```text
docs/engineering/decisions/ADR-000N-<platform-inventory-slug>.md
docs/engineering/decisions/decision-register.md
docs/publisher-services/platform-inventory.md
docs/publisher-services/decisions.md
docs/publisher-services/task-status.md
docs/publisher-services/rollout-plan.md
docs/engineering/repository-map/control-gaps.md
docs/engineering/ai-delivery/implementation-reports/ADR-01-implementation-report.md
CHANGELOG.md
```

These paths are indicative, not an exhaustive allowlist, and every one of them
is documentation. ADR-01 changes no runtime, schema, migration, workflow or CI
file in any repository.

## 4. Evidence scope

All inspection is read-only. No workflow may be dispatched, no uploader may be
run, no credential may be used, and no production or shared resource may be
accessed.

### 4.1 `thoth-pub/thoth`

Inspect at the exact implementation base:

- the GraphQL API surface and any existing platform-related enum or model;
- the export server, its formats and its route definitions;
- OCLC identifier handling and current KBART behaviour;
- relevant GitHub workflows;
- any publisher configuration currently held in repository files;
- the deferred `feature/oai-pmh-http` branch, for inventory context only.

The OAI branch must be read without checkout side effects that modify it. ADR-01
must not rebase, merge, push or otherwise change `feature/oai-pmh-http`.

### 4.2 `thoth-pub/thoth-dissemination`

Inspect, read-only:

- the uploader registry and its dispatch keys;
- every adapter module;
- workflow entry points, including scheduled and manual invocation paths;
- environment-variable **names** and configuration structure;
- linked or shared upload mechanisms, including any generic DSpace/SWORD
  mechanism used by more than one destination;
- current platform-specific tests;
- release and Docker workflows.

Record the exact commit inspected. The provisional baseline was taken at
`5e88ce1b58e5f962cc4f4ef6fb00c08f50b57add`; ADR-01 must re-verify against the
then-current head and record any drift rather than assuming the baseline still
holds.

Environment-variable names and configuration structure are in scope. Environment
variable **values**, secret contents and private configuration contents are out
of scope and must never be read or recorded.

### 4.3 `thoth-pub/thoth-app`

Read-only inspection only, to identify:

- existing publisher administration surfaces;
- any independently encoded platform names or linkage behaviour already present
  in the app;
- GraphQL operation and code-generation conventions;
- the current `main`/`dev` branch topology;
- current CI and Vercel branch assumptions.

ADR-01 must not change `thoth-app`. Any app-side platform name or linkage rule
discovered during inspection is recorded as evidence of duplication to be
removed by `APP-01` under backend-provided descriptors — not as authority for
the inventory.

### 4.4 Operational and business evidence

Some facts do not exist in any repository. ADR-01 may require confirmation from
the source owner for:

- OCLC arrangements and which OCLC feed or index variants are real destinations;
- manually managed destinations that no code represents;
- private publisher lists;
- credential ownership;
- update and withdrawal behaviour as actually practised;
- whether a destination is separately sold or separately configurable.

Source-owner confirmation must be recorded as an attributable statement with its
date and the person or role who gave it. An assumption is not confirmation.

Any destination whose existence, independence or behaviour rests only on
inference must stop the task under section 12 rather than enter the enum.

## 5. Required platform record

For every candidate destination, ADR-01 must record at least:

```text
stable proposed enum code
display name
aliases and legacy names
independently selectable destination (yes/no, with evidence)
current uploader, feed or manual mechanism
push, pull-feed or manual behaviour
linked platform group
shared adapter
duplicate-delivery risk
back-catalogue support
incremental update support
withdrawal support
retry/idempotency characteristics
required credential category
credential owner
current publisher-configuration source
operational owner
current repository/workflow references
evidence classification
open questions
final decision
```

Every field must be populated. `unknown` is a permitted value only while the
matrix is being built; an `unknown` that survives to approval is a stop
condition, not a record.

### 5.1 Credential recording rule

Credential information is limited to **category** and **ownership**. Permitted
categories include:

```text
platform API credential
SFTP credential
AWS role
GitHub environment
manual staff-managed access
no machine credential
```

ADR-01 must never record:

- secret values;
- secret identifiers, key names or token names that could aid retrieval;
- private environment contents;
- private publisher lists reproduced verbatim;
- sensitive object URLs.

Naming the *structure* of configuration (for example, that a destination is
configured by a publisher-ID environment list) is permitted and required.
Reproducing its contents is not.

## 6. Evidence classification

Every recorded claim carries exactly one classification:

```text
repository-verified
source-owner-confirmed
production-verified
provisional
unknown
```

- `repository-verified` - directly observable in inspected repository content at
  a recorded commit.
- `source-owner-confirmed` - not in a repository; confirmed by an attributable
  named owner with a recorded date.
- `production-verified` - established from production evidence supplied by an
  authorized party. ADR-01 itself performs no production access; it may only
  consume such evidence if it is provided, and must attribute it.
- `provisional` - believed but not yet established. Acceptable during work,
  never at approval for any value entering the enum.
- `unknown` - no basis. Always a stop condition for that destination.

Rules:

1. No `unknown` or `provisional` destination may become a final enum value.
2. A production-only fact may not be claimed without production evidence; absent
   that evidence, the claim is `unknown` and stops the task.
3. Repository evidence outranks recollection. Where a source owner's statement
   contradicts inspected code, ADR-01 records the conflict and escalates rather
   than choosing.
4. Absence of a destination from code is not evidence that it does not exist;
   manual destinations are precisely the case code cannot show.

## 7. Settled invariants

ADR-01 must preserve these already-approved decisions and must not reopen them:

1. `DistributionPlatform` and `MetricPlatform` are separate domains.
2. There is no universal platform enum.
3. There is no `OTHER` distribution-platform value.
4. Package selection does not imply platform assignments.
5. OAPEN and DOAB are separate platform values.
6. OAPEN and DOAB are linked for initial selection behaviour.
7. Linked OAPEN/DOAB behaviour must not produce duplicate uploads.
8. Linked-platform normalization is enforced by the backend.
9. `thoth-app` consumes backend-provided linkage metadata.
10. `thoth-app` must not maintain an independent hard-coded linked-platform
    rule.
11. Pull feeds, push destinations and manual destinations remain behaviourally
    distinct.
12. Failure to retrieve configuration must fail closed.
13. An empty assignment list must not broaden processing to all publishers or
    works.
14. Platform descriptors are code-owned operational metadata, not
    publisher-editable database rows.
15. Work-level platform selection remains deferred.
16. ADR-01 must be merged before `BE-02` finalizes `DistributionPlatform`.

If the evidence appears to require changing any of these, that is a
cross-programme architecture decision and a stop condition, not an ADR-01
decision.

## 8. Provisional inventory questions

The current
[platform inventory](../../../publisher-services/platform-inventory.md) is
explicitly provisional. ADR-01 must resolve, with evidence, the disposition of
every current provisional candidate:

```text
INTERNET_ARCHIVE
OAPEN
DOAB
SCIENCE_OPEN
CAMBRIDGE_UNIVERSITY_LIBRARY
CROSSREF
FIGSHARE
ZENODO
PROJECT_MUSE
JSTOR
EBSCO_HOST
PROQUEST
GOOGLE_PLAY
BKCI
OCLC_KB
```

Listing a code here is not approval of that code. Each requires its own
evidence, and each may be confirmed, renamed, split, merged or excluded.

ADR-01 must additionally investigate explicitly whether any of the following are
separately selectable destinations:

```text
Google Books versus Google Play
EBSCO Host versus EBSCO knowledge-base destinations
multiple ProQuest destinations
OCLC feed/index variants
manual-only destinations not represented in code
```

Guidance already recorded in the provisional baseline and binding here:

- do not create both `GOOGLE_PLAY` and `GOOGLE_BOOKS` solely because an export
  profile carries a different product name;
- add `EBSCO_KB` only if a separate EBSCO knowledge-base destination is
  independently meaningful and separately configurable;
- split ProQuest into multiple values only if staff sell or configure multiple
  independently selectable ProQuest destinations;
- no `OTHER` value may absorb an unidentified destination.

ADR-01 must also record, without normalizing them away, the current-state
defects already documented in the provisional baseline: the Project MUSE
scheduled workflow key mismatch, and the ProQuest EPUB-only fallback that can
fail because a PDF ISBN is retrieved first. The inventory must not present
scheduled delivery as healthy. Recording a defect is in scope; fixing it is not.

This specification resolves none of these questions. Nothing in section 8 may be
treated as a decision.

## 9. Required ADR-01 decisions

The approved ADR-01 must produce:

1. the final exhaustive platform inventory;
2. exact stable enum codes;
3. display labels;
4. alias handling for legacy names;
5. push, pull-feed or manual classification for every value;
6. linked groups;
7. shared-adapter relationships;
8. back-catalogue behaviour;
9. update and withdrawal expectations;
10. credential categories and ownership;
11. publisher-configuration sources;
12. a disposition for every provisional and ambiguous destination;
13. an explicit list of excluded candidates, each with its reason;
14. the exact descriptor contract required by `BE-02`;
15. the exact cross-repository mapping required later by dissemination;
16. no unresolved placeholder that `BE-02` would need to guess.

### 9.1 Descriptor contract for BE-02

The descriptor contract must be specified precisely enough that `BE-02` can
implement it as code-owned operational metadata with an exhaustive compile-time
mapping and no fallback arm. At minimum it must state, for every enum value:
display label, behaviour classification, linked group membership, shared-adapter
identity, and back-catalogue job expectation.

ADR-01 specifies the contract's content and semantics. It does not write Rust,
does not define table structure, and does not choose function signatures.

### 9.2 Cross-repository mapping for dissemination

ADR-01 must record the intended mapping from each final enum value to the
current dissemination mechanism — uploader key, shared adapter, pull feed, or
manual process — so that `DIS-01` can implement exhaustive mapping without
inventing a route. Where a value has no current mechanism, that must be stated
as an explicit declared non-uploader behaviour rather than left blank.

## 10. Acceptance criteria

- [ ] Every active channel maps to exactly one clear platform value or an
  explicit, reasoned exclusion.
- [ ] Every candidate has recorded evidence and a final disposition.
- [ ] OAPEN and DOAB remain separate values and remain linked.
- [ ] Linked platforms cannot produce duplicate delivery through ambiguous
  adapter mapping.
- [ ] Push, pull-feed and manual behaviour are explicit for every value.
- [ ] All aliases and legacy names are resolved to stable codes.
- [ ] Credential categories and owners are documented with no secret value,
  secret identifier or private configuration content recorded.
- [ ] The current publisher-configuration source is identified for every value.
- [ ] Update, withdrawal and back-catalogue support are explicit for every
  value.
- [ ] No `OTHER`, catch-all or fallback value exists.
- [ ] `BE-02` can implement an exhaustive enum and descriptor mapping without
  guessing.
- [ ] `thoth-app` can later consume backend linkage metadata without duplicating
  business rules.
- [ ] No production or shared-resource access was required for any claim, and no
  unsupported claim relies on production behaviour.
- [ ] Every remaining unknown blocked approval rather than becoming a
  provisional implementation assumption.
- [ ] The evidence matrix is complete and independently reviewable.
- [ ] Every cited repository path was verified by the independent reviewer.
- [ ] Current-state defects are recorded rather than normalized away.
- [ ] The risk classification is MEDIUM, or a higher classification is
  explicitly proposed, justified and approved.
- [ ] No runtime, schema, migration, workflow, CI, app or dissemination file
  changed.
- [ ] Explicit CTO approval is recorded before the ADR-01 PR merges.
- [ ] Fresh independent exact-head review returns `APPROVED`.

## 11. Required verification

ADR-01 is documentation-only. Its verification is evidential rather than
executable.

Required:

- `git diff --check`;
- confirmation that every changed path is documentation or changelog;
- confirmation that all relative links resolve;
- confirmation that no unresolved placeholder remains;
- confirmation that no secret-like value is present;
- confirmation that no app, dissemination, runtime, schema, migration, workflow
  or CI file changed;
- exact-head CI, recording workflow names, run IDs, conclusions and whether
  heavy jobs were correctly skipped by the documentation-only classifier;
- for every cited repository claim, the exact commit and path.

Prohibited as verification:

- dispatching any workflow;
- running any uploader, in dry-run mode or otherwise;
- connecting to any external platform;
- using any credential.

## 12. Stop conditions

The implementing agent must stop and report `BLOCKED` if:

- an active destination cannot be mapped confidently;
- a platform's independently selectable status is unknown;
- a linked relationship is based only on assumption;
- resolving the inventory requires secret values;
- resolving the inventory requires unauthorized production access;
- a production-only fact is claimed without production evidence;
- OCLC or manually managed behaviour lacks source-owner confirmation;
- an app-specific rule would need to duplicate backend logic;
- `DistributionPlatform` would need to be merged with `MetricPlatform`;
- an `OTHER` or permissive fallback appears necessary;
- a destination would be implemented only to preserve an ambiguous legacy name;
- branch normalization or Vercel routing changes enter scope;
- a runtime, migration, API, app or dissemination edit becomes necessary;
- a complete reviewable evidence matrix cannot be produced;
- the implementation base moves after branching;
- authoritative sources conflict irreconcilably.

Use these exact stop labels:

```text
BLOCKED - PLATFORM INVENTORY EVIDENCE INCOMPLETE
BLOCKED - SOURCE OWNER CONFIRMATION REQUIRED
BLOCKED - PRODUCTION ACCESS REQUIRED
BLOCKED - CROSS-PROGRAMME ARCHITECTURE DECISION
BLOCKED - APP BRANCH NORMALIZATION OUT OF SCOPE
```

Stopping with an incomplete inventory is the correct outcome when evidence is
missing. Producing a complete-looking inventory containing an assumed value is a
failure, not a partial success.

## 13. Non-goals

ADR-01 must not:

1. implement `BE-02`, `BE-03` or `BE-04`;
2. create the `DistributionPlatform` Rust or GraphQL enum;
3. add a PostgreSQL enum;
4. add a migration;
5. edit `thoth-api/src/schema.rs`;
6. add platform assignments;
7. add platform descriptors as code;
8. add GraphQL fields or queries;
9. alter `BE-01` or its pull request;
10. edit `thoth-app`;
11. edit `thoth-dissemination`;
12. create or normalize app branches;
13. edit Vercel settings;
14. change app code generation, including `thoth-app/codegen.ts`;
15. close `CG-11`;
16. dispatch workflows;
17. access production or shared databases;
18. read secret values;
19. edit issue #765 or issue #766;
20. modify the deferred `feature/oai-pmh-http` branch;
21. fix the recorded Project MUSE or ProQuest defects;
22. deploy, release or activate anything.

## 14. Cross-repository coordination

### 14.1 Coordinated sequence

```text
BE-01 merge gate
ADR-01-SPEC
ADR-01
BE-02
BE-03
APP-01
```

`ADR-01-SPEC` and `ADR-01` are not blocked by the `BE-01` merge gate; the gate
sits on the `BE-03` path, which depends on both `BE-01` and `BE-02`.

Parallel `thoth-app` readiness work proceeds on its own track:

```text
BR-APP-01 branch-topology normalization
a separately specified CG-11 CI closure task
APP-01 specification
APP-01 implementation
```

No authoritative task ID exists in current repository records for the CG-11 CI
closure task. It is referred to by description until that task is specified and
its ID recorded. Do not invent one.

### 14.2 Controls

- `thoth-app` must not begin `APP-01` implementation until `BE-03` exposes the
  approved protected API.
- `thoth-app` branch normalization (`BR-APP-01`) is a separate HIGH-risk task
  because it changes Vercel production and preview routing.
- `APP-01` must use the verified app development branch after normalization, or
  an explicit CTO exception.
- No Publisher Services programme integration branch is introduced. Every task
  uses one fresh branch and one PR.
- Backend and app remain separate repositories with separate branches and PRs.
- Cross-repository compatibility is bound through exact commit SHAs and an exact
  GraphQL schema contract, never through a moving branch name.

## 15. Reserved BE-03/APP-01 GraphQL contract control

This control is **reserved and documented here, not implemented**. It binds the
later `BE-03` and `APP-01` tasks; ADR-01 neither performs nor enables it.

1. `BE-03` produces an exact generated GraphQL SDL at its reviewed head.
2. `APP-01` records the exact `BE-03` commit SHA.
3. `APP-01` code generation consumes either a schema artifact pinned to that
   SHA, or a preview API proven to expose that exact schema.
4. `APP-01` must not generate against an unpinned moving test API and claim
   exact compatibility. The app's current codegen schema source is the shared
   test API; using it unpinned is precisely the case this control forbids.
5. The app pull request records: the backend PR; the backend SHA; the schema
   artifact or preview identity; the generated-code diff; and the
   compatibility-test result.
6. Backend contract availability precedes app merge.
7. Backend additions remain backwards-compatible, so the existing app continues
   to function unchanged.
8. App rollback must not require removing the additive backend foundation.

`thoth-app/codegen.ts` must not be changed by ADR-01. Any future configurable
schema-source change requires its own approved task specification.

## 16. Expected implementation report

Use:

`docs/engineering/ai-delivery/implementation-report-template.md`

The report must include:

- exact base and head;
- actual changed files, all documentation;
- the exact commits inspected in `thoth`, `thoth-dissemination` and `thoth-app`;
- the complete evidence matrix or its exact location;
- evidence classification counts, including every unknown and how it was
  resolved or why it blocked;
- every source-owner confirmation with attribution and date;
- explicit confirmation that no workflow was dispatched, no uploader was run, no
  credential was used and no production or shared resource was accessed;
- explicit confirmation that no secret value or private configuration content
  was recorded;
- exact-head CI evidence;
- the risk classification used, and any proposed reclassification with its
  justification;
- known limitations and deferred work;
- confirmation that `BE-02` can proceed without guessing, or the exact
  placeholders that prevent it.

## 17. Recommended execution

Implementation model: an implementation-capable model approved for MEDIUM-risk
programme work.

Implementation reasoning: MEDIUM or HIGH, selected and justified from the actual
operational risk of the destinations being inventoried. Prefer HIGH where the
inventory touches manually managed or OCLC destinations whose behaviour is not
observable in code.

Independent reviewer: a separate reviewer that did not author the inventory,
with direct access to every cited repository path, and, where source-owner
evidence is relied upon, to the attribution record.

Review reasoning: MEDIUM or HIGH, matching the implementation level.

The implementing agent may provide a self-assessment but may not approve its own
inventory.

## 18. Branch and integration plan

- branch source: then-current verified `develop`, only after this specification
  PR has merged and separate explicit authorization for the ADR-01
  implementation is granted;
- exact base recorded before any edit;
- pull-request target: `develop`;
- task branch: `feature/publisher-services/adr-01`;
- expected merge order: this specification merges; separate explicit
  authorization; fresh `develop` verification and branch creation; ADR-01
  implementation, independent review and CTO merge approval; only then `BE-02`
  specification and implementation;
- programme integration branch: none;
- branch deletion after merge: YES;
- final programme PR required: NO;
- final release path: `develop -> master`.

## 19. Approval

Approved for implementation by: Javi, CTO

Date: 2026-08-05

Notes:

- Approval becomes repository-authoritative when this specification PR is
  independently approved and merged into `develop`.
- Approval of this specification does not authorize the ADR-01 implementation.
  That requires separate explicit authorization and a freshly verified base.
- Approval settles no platform decision. The inventory in
  [`platform-inventory.md`](../../../publisher-services/platform-inventory.md)
  remains explicitly provisional until an approved ADR-01 merges.
- Approval authorizes no production access, no workflow dispatch, no credential
  use, no runtime or schema change, and no change to `thoth-app`,
  `thoth-dissemination`, `BE-01`, `CG-11`, `CG-13`, `BR-APP-01` or the deferred
  OAI branch.
