# ADR-0008-RECORD Implementation Report

## 1. Repository state

Repository: `thoth-pub/thoth`
Task ID: `ADR-0008-RECORD`
Owning programme: Shared Engineering Control / Shared Backend Architecture
Affected programmes: Publisher Services and Distribution Configuration; Thoth
Metrics; future `thoth-api` workloads requiring machine/service identities or
durable-job primitives
Risk: HIGH
Workflow: STANDARD
Base branch: `develop`
Base commit: `fac86e38383e2059e8795698e1585932c35b5b6d`
PR target: `develop`
Programme integration branch: None
Task branch: `feature/shared-architecture/adr-0008-machine-roles-job-primitives`
Head commit: recorded in the pull request; this report is written at the branch
head that carries it
Pull request: draft pull request against `develop`; live state is the GitHub
pull-request record
Expected branch deletion after merge: YES
Final programme PR required: NO
Implementing model: Claude Opus 5
Reasoning level: Extra High / xhigh

### 1.1 Risk classification

`HIGH`. The change is documentation and control only and carries no runtime,
schema, migration, API, workflow or production effect. It is nevertheless `HIGH`
because it fixes cross-programme **authorization** and **concurrency**
architecture that future `HIGH` and `CRITICAL` implementation tasks will rely on:
a defect in the recorded convention would propagate into the repository's first
machine role and its first durable-job implementation.

### 1.2 Exact authorized base

The task authorized exactly one base:

```text
fac86e38383e2059e8795698e1585932c35b5b6d
```

### 1.3 Preflight, performed before branch creation and before any edit

```text
git fetch origin --prune             = completed
origin/develop                       = fac86e38383e2059e8795698e1585932c35b5b6d  MATCH
fac86e38 subject                     = "Merge pull request #813 from
                                        thoth-pub/feature/publisher-services/be-03-closeout"
                                        -> confirmed as the merge commit of PR #813
fac86e38 author/date                 = Javier Arias <javier@jarias.org>,
                                        2026-08-14T09:28:38+01:00
Working tree                         = clean (git status --porcelain: no output)
Local branch  feature/shared-architecture/adr-0008-machine-roles-job-primitives
                                     = does not exist (git branch --list '*adr-0008*')
Remote branch feature/shared-architecture/adr-0008-machine-roles-job-primitives
                                     = does not exist
                                        (git branch -r --list '*adr-0008*';
                                         git ls-remote --heads origin '*adr-0008*')
ADR-0008 file at base                = does not exist
                                        (docs/engineering/decisions/ contains
                                         ADR-0001..ADR-0007, README.md,
                                         decision-register.md,
                                         package-capability-matrix.md)
ADR-0008 reference anywhere at base  = none (find + grep over the worktree)
Open ADR-0008 pull request           = none
Open pull requests observed          = #814, #799, #752, #744, #742, #668
PR #814                              = OPEN, DRAFT, head
                                        f7ac8c7abd5db6c642f601658200e92e6b89d0a3,
                                        headRefName
                                        feature/publisher-services/be-04-spec
                                        -> matches the expected head exactly;
                                           PR #814 has not moved
```

`origin/develop` matched the authorized SHA exactly, so no intervening-commit
inspection was required and the authorized base was not changed. PR #814's head
matched the expected SHA exactly, so there was no PR-#814 movement to report as
an observation.

The task branch was created directly from the authorized base commit, not from
the worktree's prior `master`-derived head:

```bash
git checkout -b feature/shared-architecture/adr-0008-machine-roles-job-primitives \
  fac86e38383e2059e8795698e1585932c35b5b6d
```

`git rev-parse HEAD` immediately after branch creation returned
`fac86e38383e2059e8795698e1585932c35b5b6d`, and `git status --porcelain`
returned no output.

### 1.4 CTO-approved decision date

```text
2026-08-14
```

The CTO explicitly approved the five decisions recorded in `ADR-0008` on that
date. Provenance recorded in the ADR and here is:

```text
CTO-approved control ruling supplied for this bounded task on 2026-08-14.
```

No chat, conversation, thread or session identifier is invented or quoted.

### 1.5 Source hierarchy used

Sources were used in the authority order of root
[`AGENTS.md`](../../../../AGENTS.md) section 2:

1. **Merged code, migrations and generated contracts.** The role inventory in
   [`thoth-api/src/policy.rs`](../../../../thoth-api/src/policy.rs) at the
   authorized base — `Superuser`, `PublisherAdmin`, `PublisherUser`,
   `WorkLifecycle`, `CdnWrite`, with `is_superuser()` checking an unscoped
   project role and `has_role_for_org(...)` checking a publisher-scoped role —
   is the factual basis for `ADR-0008` section 1.1. It was read only; it was not
   edited.
2. **Approved ADRs.**
   [`ADR-0001`](../../decisions/ADR-0001-publisher-package-capability-model.md),
   [`ADR-0002`](../../decisions/ADR-0002-platform-domain-boundaries.md),
   [`ADR-0003`](../../decisions/ADR-0003-repository-authoritative-schema-contract.md),
   [`ADR-0004`](../../decisions/ADR-0004-distribution-platform-inventory.md),
   [`ADR-0005`](../../decisions/ADR-0005-terminal-merge-evidence.md),
   [`ADR-0007`](../../decisions/ADR-0007-conventional-request-scoped-graphql-dataloader.md)
   and [`decision-register.md`](../../decisions/decision-register.md).
   `ADR-0007` supplied the ADR structure, the authority-condition construction
   and the "architecture approval is not implementation authorization"
   convention. `ADR-0005` supplied the durable-versus-transient wording rule.
3. **Repository control instructions.** Root
   [`AGENTS.md`](../../../../AGENTS.md) (sections 2, 9, 10, 13, 14),
   [`docs/engineering/AGENTS.md`](../../AGENTS.md) (sections 1, 1.1, 3, 6) and
   [`thoth-api/AGENTS.md`](../../../../thoth-api/AGENTS.md) (sections 1, 5, 7).
4. **Programme-control documents.**
   [`docs/metrics/task-status.md`](../../../metrics/task-status.md),
   [`docs/publisher-services/task-status.md`](../../../publisher-services/task-status.md),
   [`docs/publisher-services/acceptance-matrix.md`](../../../publisher-services/acceptance-matrix.md)
   and
   [`docs/publisher-services/rollout-plan.md`](../../../publisher-services/rollout-plan.md).
5. **Context only, not authority.** The BE-04 specification candidate and its
   implementation report on the unmerged `feature/publisher-services/be-04-spec`
   branch were read to understand **why** the decision was required — in
   particular its escalation of the machine-role adjacency to the CTO and its
   observation about Thoth Metrics WP5's "role decision" dependency. Its
   candidate architecture is **not** authoritative and did not override the CTO
   ruling. Neither file was modified, and neither is referenced by a link from
   any file this task changed, because neither exists on `develop`.

Above every repository source sits the CTO ruling itself. Where the ruling and a
candidate specification could have been read as differing, the ruling governs.

## 2. Scope confirmation

Approved specification: the CTO-approved control ruling supplied for this bounded
task on 2026-08-14, which authorizes recording the five approved decisions
faithfully.

Implemented objective: record `ADR-0008` — machine roles and durable job
primitives — with its authority condition, and reconcile the shared decision
register, the Thoth Metrics tracker and the Publisher Services tracker to the
extent required for those records to stay truthful.

Out-of-scope changes made: NONE.

Explicitly **not** done, because this task authorized none of it:

```text
no Rust or runtime implementation      no worker deployment
no policy.rs edit                      no distribution job creation
no machine-role creation in code       no BE-04 implementation
no ZITADEL/identity-provider change    no Metrics implementation
no role provisioning or grant          no deployment
no migration                           no production access
no schema.rs change                    no workflow dispatch
no GraphQL change                      no automatic-job-creation activation
```

PR #814 was not touched: no commit, comment, review, label, body edit or state
change was made to it. Its head remains the SHA observed in preflight, and its
branch was fetched read-only for context.

## 3. Commits

- `docs(engineering): record ADR-0008 machine-role conventions` — the bounded
  commit carrying this record: the new ADR, the decision-register entry, the
  Thoth Metrics and Publisher Services tracker reconciliations, the changelog
  entry and this report.
- `docs(engineering): correct changelog heading evidence in the ADR-0008 report`
  — additive evidence-accuracy commit. The `grep -n '^## \|^### ' CHANGELOG.md`
  result recorded in section 9 had been captured before the changelog entry was
  added, so it listed the pre-change line numbers. It is replaced with the actual
  post-change result. It touches this report only, and changes no decision, no
  classification, no scope and no other file.
- `docs(engineering): align ADR-0008 with approved control ruling` — the
  remediation commit answering the independent review's five `CHANGES REQUIRED`
  findings at reviewed head `594bdf9a592f3836d5ff6d1c980e0cee6f9e47be`. It
  corrects authorization-test semantics, removes architectural broadening beyond
  approved Decision 1, replaces the unsupported "existing roles are human" claim
  with a repository-grounded one, corrects the `ADR-0005`/PR-reference handling
  and adds the required PR #815 changelog reference, and fixes the five-decision
  mapping. It changes none of the five CTO-approved decisions and touches only
  files already in this pull request. See section 5.4.

No commit was amended, rebased, squashed or force-pushed. The branch head is
recorded in the pull request; the head SHA, review state, merge authorization and
merge state are terminal GitHub evidence under
[`ADR-0005`](../../decisions/ADR-0005-terminal-merge-evidence.md) and are not
transcribed here.

## 4. Files changed

- `docs/engineering/decisions/ADR-0008-machine-roles-and-durable-job-primitives.md`
  — **NEW**
  - reason: record the CTO-approved decision.
  - behavioural effect: none at runtime. It records the durable architecture
    decision, its authority condition and its implementation boundaries. It
    becomes repository-authoritative only when this exact content is
    independently reviewed at its exact head and merged into `develop`.
- `docs/engineering/ai-delivery/implementation-reports/ADR-0008-RECORD-implementation-report.md`
  — **NEW**
  - reason: required implementation report for the bounded task.
  - behavioural effect: none. Evidence record only.
- `docs/engineering/decisions/decision-register.md` — **MODIFIED**
  - reason: the register is the repository's index of engineering decisions; an
    unrecorded ADR-0008 would leave it materially incomplete.
  - behavioural effect: none. Adds the `ADR-0008` row with its decision, status,
    programmes and authority wording, adds the corresponding narrative in the
    approval-sequence section, and updates `Last updated`. No existing ADR entry
    was rewritten.
- `docs/metrics/task-status.md` — **MODIFIED**
  - reason: WP5's blocking dependency was recorded as a bare "role decision",
    which `ADR-0008` names and decides. Leaving it unqualified would understate
    what has been settled; asserting more than the ADR settles would overstate
    it.
  - behavioural effect: none. Names the dependency as the shared machine-role
    convention decided by `ADR-0008` under that ADR's authority condition, and
    adds section 3.1 stating what `ADR-0008` decides, what it does not decide for
    Metrics, and that WP5 remains `CRITICAL` and `BLOCKED` with no Metrics
    implementation authorized. `MET-CTRL-01` and every other Metrics control debt
    is untouched.
- `docs/publisher-services/task-status.md` — **MODIFIED**
  - reason: `ADR-0008` creates a new control boundary on BE-04 that the tracker
    did not record, so the tracker's BE-04 dependency list was materially
    incomplete.
  - behavioural effect: none. Records `ADR-0008` as a BE-04 dependency that must
    be repository-authoritative before BE-04 implementation may be authorized,
    as a necessary and not a sufficient condition; records that `ADR-0008` fixes
    no BE-04 operation-level authorization matrix and approves no BE-04
    specification candidate; and preserves BE-04 as `BLOCKED`/`NOT STARTED` with
    automatic job creation nonexistent and inactive.
- `CHANGELOG.md` — **MODIFIED**
  - reason: root `AGENTS.md` section 13 requires a changelog entry for every PR.
  - behavioural effect: none. One entry added under the existing
    `## [Unreleased]` -> `### Added` heading. No heading was created or
    duplicated.

Files deliberately **not** changed, and verified unchanged:
`docs/engineering/ai-delivery/tasks/BE-04.md` (absent from `develop`),
`docs/engineering/ai-delivery/implementation-reports/BE-04-SPEC-implementation-report.md`
(absent from `develop`), `docs/publisher-services/decisions.md`,
`docs/publisher-services/rollout-plan.md`,
`docs/publisher-services/acceptance-matrix.md`, all runtime code,
`thoth-api/src/policy.rs`, `thoth-api/src/schema.rs`, `thoth-api/migrations/`,
`Cargo.toml`, `Cargo.lock` and `.github/`.

## 5. Implementation decisions

Decisions taken within the approved ruling:

1. **ADR structure follows `ADR-0007`.** The header block, the concise top-level
   `Decision:` statement, the durable `Authority condition:` paragraph and the
   `Verification base:` line follow the most recent approved ADR, so the record
   is legible against its peers.
2. **The required ten-section outline is used exactly**, with the five decisions
   in sections 3.1 to 3.5 as specified. A `Review checklist` (section 10) makes
   the approved constraints directly checkable by an independent reviewer.
3. **No link to BE-04's specification.** The BE-04 specification candidate does
   not exist on `develop`, so a relative link to it would be broken in the merged
   state. It is referred to by name as "the BE-04 specification", which stays
   truthful before and after that specification exists.
4. **No terminal lifecycle metadata in any changed file.** No changed file
   copies review, approval, merge-authorization or terminal merge metadata merely
   to restate lifecycle state. Stable PR references and exact base/preflight
   evidence are permitted where required by repository controls, and are used:
   the ADR references PR [#813](https://github.com/thoth-pub/thoth/pull/813) as
   its verification base, and the changelog entry references PR
   [#815](https://github.com/thoth-pub/thoth/pull/815) as root `AGENTS.md`
   section 13 requires. `ADR-0005` section 5 explicitly permits repository
   documents to reference a pull request; what it prohibits is duplicating a
   GitHub lifecycle event into a commit so a Markdown file can repeat it. No
   review ID, approval ID, merge-authorization ID, merge SHA, merge timestamp,
   draft/ready status or future merge state appears in any changed file, so
   merging this pull request falsifies none of their prose and requires no
   follow-up status commit.
5. **Metrics reconciliation is authority-conditioned, not merge-dated.** The
   Metrics tracker says `ADR-0008` resolves the shared machine-role convention
   *when its exact approved content is repository-authoritative on `develop`*.
   That sentence is true before review, after review, before merge and after
   merge, so the later ADR merge does not make the file stale.
6. **The Publisher Services tracker was changed**, on the truthfulness test in
   the ruling: `ADR-0008` introduces a genuinely new BE-04 control boundary, and
   a dependency list omitting it would be materially incomplete. The edit records
   the boundary and nothing else; it adds no lifecycle metadata and does not
   alter BE-04's status, its `NOT STARTED` acceptance state or any other task
   row.
7. **Nothing was remediated opportunistically.** `MET-CTRL-01` remains
   `CHANGES REQUIRED`; no other Metrics or Publisher Services debt was touched.

Deviations from the approved ruling: NONE.

### 5.1 Semantic correspondence of the five approved decisions

Each approved decision is recorded semantically exactly. The ADR's subsection
numbering is **not** the decision numbering: approved Decision 4 covers both
`ADR-0008` section 3.4 and section 3.5, and approved Decision 5 is the
repository-authority condition and the BE-04 implementation gate, recorded in the
ADR header and section 8 rather than in any `3.x` subsection. The same mapping is
stated at the head of `ADR-0008` section 3. The correspondence is:

| Approved decision | Recorded in | Correspondence |
|---|---|---|
| **1 — Domain-specific machine roles** | `ADR-0008` section 3.1; consequences in section 4 | Machine/service authorization uses dedicated, least-privilege, domain-specific project roles. No generic `SERVICE`, `MACHINE`, `WORKER`, `SERVICE_ACCOUNT` or equivalent catch-all is established, and none may be introduced. An unscoped machine role is permitted **only** when the owning workload genuinely operates globally rather than for one publisher/organisation. Every machine role requires an explicit policy predicate/guard, an explicit authorization matrix, explicit permitted operations, explicit forbidden operations, least privilege and separate provisioning/credential controls — the last of these recorded as a boundary, not a provisioning architecture: provisioning and credential handling remain separately controlled by the owning implementation/deployment task and are not decided by this ADR. `SUPERUSER` authority does **not** automatically imply machine-role authority. Machine roles compose only when each role is explicitly granted; this is not generalized into a repository-wide role-inheritance rule. |
| **2 — `DISSEMINATION_WORKER`** | `ADR-0008` section 3.2; programme effect in 5.1 and 5.2 | Approved as a **Publisher-Services-specific** machine role for the BE-04/DIS-02 durable distribution workflow. It may later be implemented with exactly the permissions approved by the BE-04 specification, after that specification is independently reviewed and approved. This ADR does **not** fix BE-04's operation-level authorization matrix; that bounded detail remains owned by the BE-04 specification. The role establishes the shared enforcement convention of Decision 1; it authorizes no Metrics operation, determines no eventual Metrics machine-role name, determines no Metrics permissions or entitlement semantics, and does **not** make WP5 ready for implementation. Metrics must apply the shared convention under its own approved bounded specification. |
| **3 — Shared durable-job conventions, not a framework** | `ADR-0008` section 3.3 | All ten primitives are recorded verbatim in list form: PostgreSQL as durable owner; explicit state machines; database uniqueness for logical idempotency; deterministic idempotency/deduplication keys; explicit claim tokens; bounded leases with expiry; stale-token rejection; deterministic ordering; database-enforced concurrency; `FOR UPDATE SKIP LOCKED` where justified by the workload and evidence. They are recorded as conventions/primitives and **not** a shared generic job framework. The ADR states explicitly, in a fenced block, `approved primitive/convention != mandatory mechanism in every task`, and states that `FOR UPDATE SKIP LOCKED` must be justified by the adopting task rather than copied mechanically. |
| **4 — BE-04 remains programme-local, including the future-ADR requirement** | `ADR-0008` sections 3.4 **and** 3.5; programme effect in 5.1 and 5.2 | BE-04's future `distribution_job`, `distribution_job_target` and `distribution_job_attempt` tables, Rust domain types, GraphQL operations, state machine and lifecycle API remain Publisher-Services-specific. They are **not** a Metrics job model, a universal queue, a general `Job`/`Queue`/`Lease` API, a reusable cross-programme Rust abstraction or a universal service-worker protocol. Metrics or another programme must not reuse BE-04's tables/types/API merely by analogy. A future proposal for a reusable generic job/queue/service abstraction requires its own explicit cross-programme ADR before implementation (section 3.5). |
| **5 — Repository authority / BE-04 implementation gate** | `ADR-0008` header `Authority condition` and section 8 — **not** any `3.x` subsection | Status `APPROVED`, approved by Javi, CTO, approval date 2026-08-14, decision owner CTO. Repository-authoritative is defined as exact approved ADR content **plus** independent exact-head review **plus** merge into `develop`. The authority condition states that `APPROVED` content on an unmerged branch is not yet repository-authoritative and may not be relied upon for implementation until independently reviewed and merged to `develop`. Section 8.2 states that this ADR must be repository-authoritative before BE-04 implementation may be authorized, and that this is a necessary and not a sufficient condition. The record states in three places that it is neither BE-04 specification approval nor BE-04 implementation authorization. |

### 5.2 Why this is a cross-programme decision

Three independent facts make it cross-programme rather than programme-local:

1. **One authorization surface, two programmes.** Publisher Services' proposed
   worker role and Thoth Metrics WP5's "role decision" both write into the same
   crate, the same `thoth-api/src/policy.rs`, the same `Role` enum and the same
   ZITADEL project. There is no way for one programme to answer the question
   only for itself.
2. **Precedence would otherwise decide it.** At the authorized base, repository
   policy defines no role as a dedicated machine/service role. The first machine
   role merged would become the de-facto convention for the second, and delivery
   order is not a decision process. The BE-04 specification candidate identified exactly this
   and escalated it rather than deciding it.
3. **The same risk applies to durable-job machinery.** BE-04's job tables would
   be the repository's first substantial durable-job implementation. Without an
   explicit programme-local boundary, a later programme could reuse them by
   analogy and convert a bounded design into an unowned shared framework, again
   with no decision having been taken.

`docs/engineering/decisions/README.md` reserves cross-programme approval to the
CTO control process. The decision was escalated and approved accordingly, and
this task records it.

### 5.3 Why no runtime implementation was authorized or performed

The ruling authorizes recording the decision and nothing else. Three independent
reasons make that the correct boundary:

1. **The ADR is not yet repository-authoritative.** By its own authority
   condition, `APPROVED` content on an unmerged branch may not be relied upon for
   implementation. Implementing from it in the same change would rely on it
   before it becomes authoritative.
2. **The dependent specifications are not approved.** BE-04's operation-level
   authorization matrix is owned by the BE-04 specification, which still requires
   its own independent review and approval; Metrics WP5 has no approved bounded
   slice specification. There is nothing approved to implement against.
3. **The repository's controls forbid it.** Root `AGENTS.md` section 1 requires
   an approved written specification before implementation, and section 6
   prohibits broadening scope without one. A machine role additionally needs
   identity-provider provisioning and credential controls, which are outside this
   repository and separately authorized.

Accordingly, no Rust file was edited, no role was created in code, no policy
predicate was added, no migration was written, no GraphQL contract changed, no
identity-provider or provisioning action occurred, no workflow was dispatched and
no deployment or production access took place.

### 5.4 Independent-review remediation

Independent review of head `594bdf9a592f3836d5ff6d1c980e0cee6f9e47be` returned
`CHANGES REQUIRED` with five findings. All five are fidelity and control-record
defects; none reopened the architecture, and **the five CTO-approved decisions
themselves are unchanged**. The corrections were applied by one ordinary additive
commit on the same branch, with no amend, rebase, squash or force push.

**Finding 1 — authorization test semantics.** `ADR-0008` section 4 item 7 had
described the whole caller matrix as "all failing closed". That is wrong: root
`AGENTS.md` section 9 lists the caller matrix to be tested and *separately*
requires that authorization **failures** fail closed. Item 7 now states the
matrix as the two standing controls state it, and adds explicitly that negative
cases fail closed, that positive cases succeed only where the owning approved
specification's authorization matrix permits, and that this ADR pre-decides no
caller — neither `SUPERUSER` nor any publisher-scoped role — as a positive case
for a future machine operation. Section 8 of this report carried the same
incorrect gloss and is corrected identically. Least privilege is unchanged, and
no repository `AGENTS.md` file was modified.

**Finding 2 — unapproved architectural broadening.** Four normative statements
went beyond approved Decision 1 and are removed or demoted:

- "provisioning and credential controls, **distinct from human role
  provisioning**" -> requirement 6 is retained as "separate provisioning and
  credential controls" and is now explicitly a boundary rather than a
  provisioning architecture: provisioning and credential handling remain
  separately controlled by the owning implementation/deployment task and are not
  decided by this ADR, which describes no provisioning mechanism, credential
  store, rotation policy or identity-provider arrangement;
- "**Roles** compose only when each role is explicitly granted" -> scoped to
  "**Machine roles** compose only when each role is explicitly granted";
- "No role implies, inherits or subsumes another" -> **removed**; no
  repository-wide role-inheritance rule is asserted;
- "`SUPERUSER` remains a **human** administrative role" -> "`SUPERUSER` remains
  an administrative role. Holding it does not by itself confer machine-role
  authority."

The approved sentence "`SUPERUSER` authority does not automatically imply
machine-role authority" is preserved. The equivalent wording was corrected in the
decision register, the Metrics tracker, the changelog entry and the correspondence
table in section 5.1 of this report.

**Finding 3 — existing-role fact claim.** `ADR-0008` section 1.1 had asserted
that every existing role "models a human actor". Merged `policy.rs` does not
encode principal type; it distinguishes roles by permission and scope. The
assertion is replaced with the repository-grounded statement that **no existing
role is defined by repository policy as a dedicated machine/service role**, with
the factual five-role inventory and its scope description preserved, and with an
explicit note that this ADR makes no claim about which principals hold any role
or about current identity-provider assignments. Two derived statements — decision
driver 5 in the ADR and the cross-programme narrative in the decision register —
were corrected the same way, as was step 3 of section 10 and section 5.2 item 2
of this report.

**Finding 4 — ADR-0005 / PR-reference handling.** This report had claimed "No PR,
review, approval or merge identifiers in any changed file", which was both false
and overbroad: `ADR-0005` section 5 permits repository documents to reference a
pull request, and the ADR legitimately references PR
[#813](https://github.com/thoth-pub/thoth/pull/813) as its verification base. The
claim is replaced with the purpose-qualified rule — no changed file copies review,
approval, merge-authorization or terminal merge metadata merely to restate
lifecycle state, while stable PR references and exact base/preflight evidence are
permitted where repository controls require them. The PR #813 verification-base
reference is retained. Root `AGENTS.md` section 13 requires referencing the PR
number when available, so the changelog entry now carries a stable
[#815](https://github.com/thoth-pub/thoth/pull/815) reference in the repository's
existing changelog convention of a bracketed PR number linking to the pull
request, followed by the entry text. No review ID, approval
ID, merge-authorization ID, merge SHA, merge timestamp, draft/ready status or
future merge state was added to any committed file.

**Finding 5 — five-decision mapping.** The record had implied that `ADR-0008`
sections 3.1-3.5 are themselves the five approved decisions, which mis-maps
Decision 5. The correct mapping — Decision 1 -> 3.1; Decision 2 -> 3.2;
Decision 3 -> 3.3; Decision 4 -> 3.4 **and** 3.5; Decision 5 -> the header
`Authority condition` **and** section 8 — is now stated in a table at the head of
`ADR-0008` section 3, in the ADR review checklist, in section 5.1 of this report,
in the required-content verification block in section 9 and in the review-focus
list in section 15. `ADR-0008` section 3.5 is retained and now states that it
expresses the future-abstraction portion of Decision 4 rather than a separate
decision.

## 6. Database and migration effects

Migration added: NO.

No migration, no schema change, no `thoth-api/src/schema.rs` change, no data
effect, no locking and no downtime. The complete diff is confined to
`CHANGELOG.md` and `docs/**`, proved in section 9.

## 7. API and compatibility effects

GraphQL/API changes: NONE.
Generated schema/client updates: NONE.
Backwards compatibility: unaffected — no contract exists to break.
Deprecations: NONE.
Cross-repository dependencies: none created. `ADR-0008` constrains how future
`thoth-api` machine roles and durable-job implementations are designed; it adds
no dependency on or from another repository.

## 8. Authorization and security

Authorization paths changed: NONE. `thoth-api/src/policy.rs` was read for factual
accuracy and not modified.

Roles/scopes involved: none at runtime. `ADR-0008` records the future convention
for machine roles and approves `DISSEMINATION_WORKER` as a
Publisher-Services-specific role **name and boundary**. No role is created,
granted, provisioned or checked by this change, and no ZITADEL project role
exists as a result of it.

Negative authorization tests: not applicable — no authorization code path was
added or altered.

`ADR-0008` section 4 item 7 restates the existing obligations without changing
them. Root `AGENTS.md` section 9 and `thoth-api/AGENTS.md` section 7 require the
caller matrix — anonymous caller; authenticated caller without the role; caller
scoped to another publisher; correctly scoped publisher role; superuser;
machine/service role — to be tested as applicable, and separately require
authorization **failures** to fail closed. The matrix is therefore not uniformly
negative: **negative** cases must fail closed, while **positive** cases succeed
only where the owning approved specification's authorization matrix permits them.
`ADR-0008` pre-decides no caller as a positive case for any future machine
operation — in particular it decides neither `SUPERUSER` nor any
publisher-scoped role to be a permitted caller of a machine operation — subject
to the least-privilege requirement in its section 3.1.

Secret or personal-data handling: none. No credential, token, secret or personal
datum appears in any changed file.

Security limitations: the decision is a convention, not an enforcement mechanism.
It constrains future design and review; it cannot by itself prevent an
implementation from being written incorrectly. The compensating controls are the
per-role approved specification, independent exact-head review, and the existing
`thoth-api/AGENTS.md` section 7 test obligations — all unchanged.

## 9. Tests and checks

No test suite applies: the change adds no code. The repository's
documentation-only evidence requirements in root `AGENTS.md` section 8 were
applied.

### Whitespace and conflict-marker check

Command:

```bash
git diff --check
```

Result:

```text
no output (clean)
```

Also run against the staged set as `git diff --cached --check`, with no output.

### Diff containment — only `CHANGELOG.md` and `docs/**`

Command:

```bash
git diff --name-only fac86e38383e2059e8795698e1585932c35b5b6d HEAD
```

Result:

```text
CHANGELOG.md
docs/engineering/ai-delivery/implementation-reports/ADR-0008-RECORD-implementation-report.md
docs/engineering/decisions/ADR-0008-machine-roles-and-durable-job-primitives.md
docs/engineering/decisions/decision-register.md
docs/metrics/task-status.md
docs/publisher-services/task-status.md
```

Command:

```bash
git diff --name-only fac86e38383e2059e8795698e1585932c35b5b6d HEAD \
  | grep -v -E '^(CHANGELOG\.md|docs/)'
```

Result:

```text
no output — every changed path is CHANGELOG.md or under docs/
```

### Zero changes under the protected trees

Command:

```bash
git diff --name-only fac86e38383e2059e8795698e1585932c35b5b6d HEAD -- \
  thoth-api/ thoth-api-server/ thoth-client/ thoth-errors/ thoth-export-server/ \
  .github/ Cargo.toml Cargo.lock
```

Result:

```text
no output — zero changed files under any protected path
```

### Files that must not be modified

Command:

```bash
git diff --name-only fac86e38383e2059e8795698e1585932c35b5b6d HEAD -- \
  docs/engineering/ai-delivery/tasks/BE-04.md \
  docs/engineering/ai-delivery/implementation-reports/BE-04-SPEC-implementation-report.md \
  docs/publisher-services/decisions.md \
  docs/publisher-services/rollout-plan.md \
  docs/publisher-services/acceptance-matrix.md
```

Result:

```text
no output — none of the prohibited files was modified
```

The first two do not exist on `develop` at all; the remaining three exist and are
byte-identical to the base.

### Relative-link verification

Every relative Markdown link in every changed file was resolved against the
filesystem.

Command:

```bash
python3 - <<'PY'
import re, os, sys
files = [
 "CHANGELOG.md",
 "docs/engineering/decisions/ADR-0008-machine-roles-and-durable-job-primitives.md",
 "docs/engineering/decisions/decision-register.md",
 "docs/metrics/task-status.md",
 "docs/publisher-services/task-status.md",
 "docs/engineering/ai-delivery/implementation-reports/ADR-0008-RECORD-implementation-report.md",
]
pat = re.compile(r'\[([^\]]*)\]\(([^)]+)\)')
bad = checked = 0
for f in files:
    base = os.path.dirname(f)
    for m in pat.finditer(open(f, encoding='utf-8').read()):
        t = m.group(2).strip()
        if t.startswith(('http://','https://','#','mailto:')):
            continue
        p = t.split('#')[0]
        if not p:
            continue
        full = os.path.normpath(os.path.join(base, p))
        checked += 1
        if not os.path.exists(full):
            bad += 1
            print(f"BROKEN  {f}: {t} -> {full}")
print(f"relative links checked: {checked}; broken: {bad}")
sys.exit(1 if bad else 0)
PY
```

Result:

```text
relative links checked: 78; broken: 0
```

### Contradictory-wording search

Because the changed Markdown is hard-wrapped, a plain `grep` can miss a phrase
split across a line break. The search therefore normalizes all whitespace to
single spaces before matching, and is case-insensitive.

The searched set is the five substantive changed files. This report is excluded
from the pattern search because it necessarily quotes the forbidden phrases as
literal search patterns and as evidence output; those occurrences are quoted
strings inside this section, not normative assertions anywhere in the repository's
control records.

Command:

```bash
python3 - <<'PY'
import re
files = [
 "CHANGELOG.md",
 "docs/engineering/decisions/ADR-0008-machine-roles-and-durable-job-primitives.md",
 "docs/engineering/decisions/decision-register.md",
 "docs/metrics/task-status.md",
 "docs/publisher-services/task-status.md",
]
pats = ["BE-04 implementation authorized", "BE-04 implementation is authorized",
        "BE-04 is authorized", "WP5 ready", "WP5 is ready", "WP5 becomes ready",
        "Metrics uses DISSEMINATION_WORKER", "universal worker role",
        "shared distribution_job framework", "generic job framework",
        "job framework", "universal queue", "SERVICE_ACCOUNT"]
for p in pats:
    print(f"### {p}")
    hits = 0
    for f in files:
        t = re.sub(r'\s+', ' ', open(f, encoding='utf-8').read())
        for m in re.finditer(re.escape(p), t, re.IGNORECASE):
            hits += 1
            print(f"  {f}: ...{t[max(0,m.start()-110):m.end()+110]}...")
    if not hits:
        print("  (no match)")
PY
```

Result:

```text
BE-04 implementation authorized     no match
BE-04 implementation is authorized  no match
BE-04 is authorized                 no match
WP5 is ready                        no match
WP5 becomes ready                   no match
Metrics uses DISSEMINATION_WORKER   no match
universal worker role               no match
shared distribution_job framework   no match

WP5 ready            matches only inside the negation
                     "it does **not** make Metrics WP5 ready for implementation"
generic job framework / job framework
                     matches only inside negations and prohibitions:
                     "not a shared generic job framework"
                     "without creating a generic job framework"
                     "with no generic job framework created"
                     "conventions rather than a shared job framework"
universal queue      matches only inside the prohibition list "They are **not**:
                     ... a universal queue ..."
SERVICE_ACCOUNT      matches only inside the prohibition of catch-all role names
```

Every hit is a negation or a prohibition. No changed file contains a normative
assertion that BE-04 implementation is authorized, that WP5 is ready, that
Metrics uses `DISSEMINATION_WORKER`, that a universal worker role exists, or that
a shared `distribution_job` framework exists.

### Required-content verification

Confirmed by reading the merged content of
`ADR-0008-machine-roles-and-durable-job-primitives.md`:

```text
all five approved decisions present            YES
  Decision 1                                   3.1
  Decision 2                                   3.2
  Decision 3                                   3.3
  Decision 4                                   3.4 AND 3.5
  Decision 5                                   header authority condition AND 8
  (mapping table at the head of section 3;
   subsection numbering is not decision
   numbering, and 3.5 is part of Decision 4)
creates no generic SERVICE/MACHINE role        YES  (3.1, prohibition)
no unsupported "existing roles are human"
  claim; states only that policy defines no
  dedicated machine/service role               YES  (1.1)
negative authorization cases fail closed;
  positive cases follow the owning approved
  matrix; no caller pre-decided as positive    YES  (4 item 7)
no unapproved shared provisioning or
  role-inheritance architecture                YES  (3.1, 4 items 6 and 8)
does not claim Metrics uses DISSEMINATION_WORKER
                                               YES  (3.2, 5.2 item 5)
does not make WP5 ready                        YES  (3.2, 5.2 items 2-3)
does not authorize BE-04 implementation        YES  (5.1 item 4, 7, 8.2)
does not approve PR #814 / any BE-04 spec      YES  (5.1 item 3, 8.2)
creates no generic job framework               YES  (3.3)
does not mandate SKIP LOCKED universally       YES  (3.3)
requires a future ADR for a shared generic
  job abstraction                              YES  (3.5)
status                                         APPROVED
approval date                                  2026-08-14
approver / decision owner                      Javi, CTO / CTO
authority condition requires independent
  review + merge to develop                    YES  (header, 8.1)
```

### Changelog

One entry added under the existing `## [Unreleased]` -> `### Added` heading. No
heading was added, duplicated or reordered. Verified with:

```bash
grep -n '^## \|^### ' CHANGELOG.md | head -8
```

Result:

```text
7:## [Unreleased]
8:### Added
44:### Changed
61:### Removed
64:### Fixed
67:## [[1.6.3]](https://github.com/thoth-pub/thoth/releases/tag/v1.6.3) - 2026-08-13
68:### Changed
71:## [[1.6.2]](https://github.com/thoth-pub/thoth/releases/tag/v1.6.2) - 2026-08-10
```

The `## [Unreleased]` section retains exactly one `### Added` heading. The
`### Changed`, `### Removed` and `### Fixed` line numbers each moved down by one
relative to the authorized base, which is the single added entry line and nothing
else.

### Rust checks

Not run, and not required. The change contains no Rust, no `Cargo.toml`/
`Cargo.lock` modification and no generated-contract modification, proved by the
protected-tree check above. Repository CI is the authority for the
documentation-only classification; see section 11.

## 10. Manual verification

Environment: local `git` worktree of `thoth-pub/thoth` at the authorized base,
read-only `gh` queries against GitHub for pull-request state. No database, no
service, no deployment target and no production system was contacted.

Steps and observed results:

1. Preflight as recorded in section 1.3 — all conditions met; not `BLOCKED`.
2. Read the authoritative material in the source hierarchy of section 1.5 before
   any edit.
3. Confirmed the current role inventory in `thoth-api/src/policy.rs` so that
   `ADR-0008` section 1.1 states the factual base accurately: five `Role`
   variants; `SUPERUSER` checked as an unscoped project-role key through
   `is_superuser()`; the other four checked with publisher-organisation scope
   through `has_role_for_org(...)`. `policy.rs` distinguishes roles by permission
   and scope and encodes no machine-principal role category, so the grounded
   claim is that **no existing role is defined by repository policy as a
   dedicated machine/service role** — not that any role is necessarily held by a
   human, which `policy.rs` neither states nor tests.
4. Drafted `ADR-0008` against the ruling clause by clause, then re-checked each
   clause against the ADR text; the correspondence is in section 5.1.
5. Reconciled the decision register, the Metrics tracker and the Publisher
   Services tracker.
6. Ran the complete validation set in section 9.
7. Re-read every changed file after editing, checking for stale status prose,
   transient lifecycle metadata, and any sentence that merging this pull request
   would falsify. None found.
8. Confirmed PR #814 untouched: no commit, comment, review, label, body edit or
   state change; its branch was fetched read-only and its head is unchanged from
   preflight.

## 11. CI

CI status: delegated to the live GitHub authority.

Under [`ADR-0005`](../../decisions/ADR-0005-terminal-merge-evidence.md), CI
records are terminal evidence held by GitHub and are not transcribed into
repository files. The actual result for this pull request is the GitHub checks
record on the exact head.

Documentation-only job skips are **expected** for a diff confined to
`CHANGELOG.md` and `docs/**` under the repository's documentation-only CI
classification, but they are **not assumed**: the reviewer should read the actual
check results on the pull request rather than infer them from the diff shape. No
workflow was dispatched manually, and no commit was created to produce CI
evidence.

Failures or warnings: to be read from the live pull-request record.

## 12. Rollout and rollback

Initial state after merge: `ADR-0008` becomes repository-authoritative — the
decision is recorded and binding on future design — and nothing else changes. No
code, no schema, no role, no job, no deployment and no production behaviour is
affected.

Activation required: none, because nothing is activated. The dependent work each
requires its own separate authorization:

```text
BE-04 specification         independent review + explicit approval
BE-04 implementation        approved specification + separate explicit
                            implementation authorization from a freshly
                            verified develop head
Metrics WP5                 WP4 + its own approved bounded slice specifications
machine-role creation       the owning approved specification
role provisioning /         separate identity-provider authorization,
  credentials               outside this repository
```

Feature flag/configuration: none.

Migration sequence: none.

Rollback: ordinary revert of a documentation-only change. Reverting removes the
decision record; it removes no code, schema, role or runtime behaviour, because
none was created.

Monitoring required: none.

## 13. Known limitations and deferred work

1. `ADR-0008` records a convention, not an enforcement mechanism. Compliance is
   established per adopting task through its approved specification and
   independent review.
2. BE-04's operation-level authorization matrix is deliberately not settled here.
   It remains owned by the BE-04 specification, which requires its own
   independent review, any remediation that review requires, and explicit
   approval.
3. The Metrics machine-role name, entitlement model, credential model and
   operation matrix are deliberately not selected. They belong to a future
   approved bounded Metrics specification.
4. Machine-role provisioning and credential controls live in the identity
   provider, outside this repository. `ADR-0008` requires that they be separate;
   it does not describe them, and no such control was created or changed.
5. Whether a reusable generic cross-programme job/queue/service abstraction is
   ever justified is left open. `ADR-0008` fixes only the gate: a separate
   explicit cross-programme ADR before implementation.
6. `MET-CTRL-01` remains `CHANGES REQUIRED` and was deliberately not remediated.
7. `docs/publisher-services/acceptance-matrix.md` and
   `docs/publisher-services/rollout-plan.md` were read and left unchanged: their
   BE-04 rows describe evidence and activation gates that `ADR-0008` neither
   satisfies nor alters.

## 14. Unresolved issues

NONE.

The task did not reach any stop condition. `origin/develop` matched the
authorized base exactly, PR #814's head matched the expected SHA exactly, no
competing ADR-0008 branch, file or pull request existed, and the repository state
matched the task premise.

## 15. Agent self-assessment

The implementing agent did **not** approve this work, did not merge it, did not
mark it ready for review as an approval, and issues no approval decision. The
pull request is left as a draft for independent exact-head review under the
repository controls. Approval and merge authorization belong to the independent
reviewer and the CTO.

Suggested review focus:

1. **Semantic fidelity to the ruling.** Read section 5.1's correspondence table
   against the mapping table at the head of `ADR-0008` section 3 — Decision 1 ->
   3.1, Decision 2 -> 3.2, Decision 3 -> 3.3, Decision 4 -> 3.4 and 3.5,
   Decision 5 -> the header authority condition and section 8 — and confirm
   nothing was weakened, broadened, reconsidered or replaced. In particular
   confirm that the ADR does not settle BE-04's operation-level authorization
   matrix, which the ruling reserves to the BE-04 specification, and that no
   reasonable-sounding addition has been promoted into approved architecture
   beyond the five decisions.
2. **Absence of over-claiming.** Confirm that no changed file asserts BE-04
   implementation authorization, BE-04 specification approval, WP5 readiness, a
   Metrics role selection, or any generic machine role or job framework.
3. **Durability of the tracker wording.** Confirm that the Metrics and Publisher
   Services edits use an authority-condition construction that stays true before
   review, after review, before merge and after merge, so the later ADR merge
   creates no stale state and needs no follow-up status commit.
4. **Boundedness.** Confirm the diff is confined to `CHANGELOG.md` and `docs/**`,
   that the prohibited files are untouched, and that no Metrics or Publisher
   Services debt was remediated opportunistically.
5. **Register consistency.** Confirm the `ADR-0008` register entry matches the
   ADR and that no unrelated ADR entry was rewritten.
