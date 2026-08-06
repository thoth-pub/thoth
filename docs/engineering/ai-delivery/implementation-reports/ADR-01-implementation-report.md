# ADR-01 Implementation Report

## 1. Repository state

Repository: `thoth-pub/thoth`
Programme: Publisher Services and Distribution Configuration
Task: ADR-01 - Platform inventory and final architecture
Risk: MEDIUM
Workflow: STANDARD
Base branch: `develop`
Base commit: `32123d363a6806d377ac322e3814fb432a803453` (verified equal on
local `develop` and `origin/develop` before any edit)
PR target: `develop`
Programme integration branch: None
Task branch: `feature/publisher-services/adr-01`
Head commit: the PR-number follow-up commit on
[draft PR #783](https://github.com/thoth-pub/thoth/pull/783); the exact
head SHA and its CI evidence are recorded in the immutable exact-head
evidence comment on that PR
Pull request: [draft PR #783](https://github.com/thoth-pub/thoth/pull/783)
Expected branch deletion after merge: YES
Final programme PR required: NO
Implementing model: Claude (Fable 5), implementation-capable model approved
for MEDIUM-risk programme work
Reasoning level: HIGH
Authorization: explicit CTO implementation authorization, Javi, CTO,
2026-08-06, from the exact base above (fresh authorization required by the
merged amended specification; granted in the task authorization of
2026-08-06)

## 2. Scope confirmation

Approved specification:
[`docs/engineering/ai-delivery/tasks/ADR-01.md`](../tasks/ADR-01.md)
(approved corrected-content head `1276c70a`; repository-authoritative
through PR [#781](https://github.com/thoth-pub/thoth/pull/781) merge commit
`a511e01c`), narrowed by the CTO implementation authorization of
2026-08-06.

Implemented objective: determine and record the final exhaustive
user-visible `DistributionPlatform` inventory and associated operational
architecture as [ADR-0004](../../decisions/ADR-0004-distribution-platform-inventory.md)
(`PROPOSED`), with the complete
[evidence matrix](../../../publisher-services/adr-01-evidence-matrix.md),
the proposed
[final inventory](../../../publisher-services/platform-inventory.md), the
BE-02 descriptor contract, the future dissemination mapping and
programme-control reconciliation. Evidence and architecture decision only:
no runtime code, enum, migration, schema, API or workflow was written.

Out-of-scope changes made: NONE

## 3. Preflight record

All mandatory preflight checks passed before any edit:

- `origin` fetched and pruned; working tree clean.
- Local `develop` == `origin/develop` ==
  `32123d363a6806d377ac322e3814fb432a803453`.
- PR [#782](https://github.com/thoth-pub/thoth/pull/782) verified MERGED
  into `develop` (head `dc1dc78ac0d002c417dec4c5df47e8b8ef4a8147`, merge
  commit `32123d36...`, merged 2026-08-06T14:04:07Z).
- Independent closeout review `4875403240` (`APPROVED`, exact head
  `dc1dc78a`) and CTO closeout approval / merge authorization `4875441989`
  verified on PR #782; CTO corrected-content approval comment `5203642323`
  verified on PR #781.
- Dependency states verified from the merged control records: P0-01 CLOSED;
  ADR-0001, ADR-0002, ADR-0003 APPROVED AND MERGED; CG-12 RESOLVED; ADR-01
  specification approved and merged; ADR-01-SPEC-AMEND-01 merged and
  complete; the closeout merged.
- No remote branch `feature/publisher-services/adr-01`, no open ADR-01 PR,
  no untracked or uncommitted earlier ADR-01 implementation work.
- Decision register contained ADR-0001 through ADR-0003 and no ADR-0004;
  no `ADR-0004-*` file existed. ADR-0004 is the authorized decision number.
- Evidence-ledger provenance boundary verified: ledger section 0.0 records
  the SHA-256
  `4395c9b7203cdb5c07f5ad6399879827b1964bf8aeb1edc150bfc4d77221e9d7` as
  covering only the original 18 evidence entries (sections 1-8); section 9
  is a separately attributable CTO record. The two sources are cited
  separately throughout (`LEDGER:<ID>` vs `CTO-2026-08-06`).
- No repository file or merged decision superseded any settled ADR-01
  invariant or input.

Deviation (recorded): a local branch `feature/publisher-services/adr-01`
already existed and was checked out at session start, pointing **exactly**
at the authorized base `32123d36...`, commit-free, with a clean tree and no
remote counterpart. It cannot be the deleted pre-amendment branch (which sat
at the superseded base and was deleted during the closeout) and is exactly
the branch the authorization's section 6 creates. It was treated as
satisfying branch creation rather than triggering
`BLOCKED - ADR-01 IMPLEMENTATION ALREADY EXISTS`, because no prior
implementation work, remote branch or PR existed. No other deviation
occurred.

## 4. Commits

- `f56b0dd7` - `docs(publisher-services): decide distribution platform
  inventory` - ADR-0004, evidence matrix, final proposed inventory,
  control-document reconciliation, changelog
- `2643fb1d` - `docs(publisher-services): report ADR-01 implementation` -
  this report
- follow-up commit (the PR head) - records PR #783 in the changelog,
  tracker and this report; its SHA is recorded in the immutable exact-head
  evidence comment on PR #783

Normal commits only: no amend, rebase, squash, reset or force-push.

## 5. Files changed

- `docs/engineering/decisions/ADR-0004-distribution-platform-inventory.md`
  - reason: the ADR-01 decision record (PROPOSED)
  - behavioural effect: none (documentation)
- `docs/engineering/decisions/decision-register.md`
  - reason: register ADR-0004 as PROPOSED, not approved
  - behavioural effect: none
- `docs/publisher-services/adr-01-evidence-matrix.md`
  - reason: complete per-candidate evidence record (new file)
  - behavioural effect: none
- `docs/publisher-services/platform-inventory.md`
  - reason: replace the provisional baseline with the final proposed
    inventory (marked proposed, not approved)
  - behavioural effect: none
- `docs/publisher-services/decisions.md`
  - reason: record the ADR-01 implementation state on the delegated
    decisions
  - behavioural effect: none
- `docs/publisher-services/task-status.md`
  - reason: ADR-01 row to IN PROGRESS (draft PR pending review); next
    actions reconciled; BE-02 remains BLOCKED
  - behavioural effect: none
- `docs/publisher-services/rollout-plan.md`
  - reason: record the delivered ADR-01 implementation state and remaining
    gates in Stage 0
  - behavioural effect: none
- `docs/engineering/repository-map/control-gaps.md`
  - reason: update CG-07 state (remains OPEN; ADR-0004 proposed)
  - behavioural effect: none
- `docs/engineering/ai-delivery/implementation-reports/ADR-01-implementation-report.md`
  - reason: this report (new file)
  - behavioural effect: none
- `CHANGELOG.md`
  - reason: unreleased entry for the ADR-01 implementation PR
  - behavioural effect: none

All changed paths are documentation or the changelog. The following were
deliberately NOT changed: the ADR-01 specification, the evidence ledger,
ADR-0001 through ADR-0003, every runtime crate (`thoth-api/**`,
`thoth-api-server/**`, `thoth-client/**`, `thoth-errors/**`,
`thoth-export-server/**`), `.github/**`, `Cargo.toml`, `Cargo.lock`, all
migrations, schema, GraphQL and application files, `thoth-app`,
`thoth-dissemination`, and the deferred OAI branch. Issues #765 and #766
were not edited.

## 6. Evidence record

### 6.1 Exact commits inspected (all read-only)

- `thoth-pub/thoth`: `32123d363a6806d377ac322e3814fb432a803453`
  (authorized base)
- `thoth-pub/thoth` deferred OAI branch `feature/oai-pmh-http`:
  `745dd020661e8a8b94d0752e11f10a9d583bd769` (context only; not modified,
  checked out destructively, rebased, merged or pushed)
- `thoth-pub/thoth-dissemination` default branch `main`:
  `7a16edc08d4570f3ecc108453298a3aa43f6d753` (= tag `v1.6.4`, release
  1.6.4, 28 July 2026 - identical to the release commit cited by the
  evidence ledger; drift from the provisional baseline `5e88ce1b` is
  Internet Archive hardening only, recorded in the evidence matrix
  section 2.1)
- `thoth-pub/thoth-app`: `main`
  `6f826390a07efe6266cfda2b4af1f85b6cbfc38a`; `dev`
  `26323158f1145b35eff27bce6f901ff0eb78280a` (duplication evidence only)

### 6.2 Evidence matrix

Complete matrix:
[`docs/publisher-services/adr-01-evidence-matrix.md`](../../../publisher-services/adr-01-evidence-matrix.md)
- 27 candidates (17 included, 10 excluded), every field of the required
record populated for every included value.

Evidence classification counts (matrix section 8):

```text
repository-verified:       34
source-owner-confirmed:    21
production-verified:        0
provisional (included):     0
unknown (included):         0
```

Every `unverified` status appears only in excluded-candidate records, where
it supports exclusion. No unknown survived into any included value; no
unknown blocked completion.

### 6.3 Source-owner confirmations relied upon

All are recorded with attribution and date in the matrix section 9 (S1-S21):
the original ledger entries EBSCO-01..05, PROQUEST-01..06, KBART-01..03,
JISC-01..02, HIST-01 and ADR-01-SOURCE (source record prepared 6 August
2026; CTO-approved), and the separately attributable CTO decisions of
2026-08-06 (Javi, CTO): Ex Libris separation and shared feed; Jisc NBK
inactive inclusion; no known current manual-only destinations; conservative
update/withdrawal policy; Google Play Books single destination; the
section 8.6 exclusions; and the ownership/configuration authority record.

### 6.4 Conflicts discovered and their resolution

- The export registry at the base associates `jisc_kb` with the
  `kbart::oclc` specification, while ledger JISC-01 establishes the current
  Jisc mechanism as MARC21 via S3. Resolution: recorded as a historical
  registry association versus the current agreed mechanism; `JISC_NBK` maps
  to the inactive `JISC_NBK_MARC_S3` identity and must not be modelled as a
  KBART consumer. No silent resolution; both facts recorded.
- The export registry associates `ebsco_kb` and `proquest_kb` with
  `kbart::oclc`. Resolution: registry acceptance is recorded as not being
  evidence of current consumption; both candidates remain excluded as
  currently unverified, and the shared-feed inference prohibition is
  restated in their records.
- No other authoritative-source conflict was found. No contradiction
  between a source-owner statement and inspected code arose.

### 6.5 Dispositions

Included (17): `INTERNET_ARCHIVE`, `OAPEN`, `DOAB`, `SCIENCE_OPEN`,
`CAMBRIDGE_UNIVERSITY_LIBRARY`, `CROSSREF`, `FIGSHARE`, `ZENODO`,
`PROJECT_MUSE`, `JSTOR`, `EBSCO_HOST`, `PROQUEST_EBOOK_CENTRAL`,
`GOOGLE_PLAY`, `BKCI`, `OCLC_KB`, `EX_LIBRIS_KB`, `JISC_NBK` (inactive,
non-assignable, job-free).

Excluded (10, each with an individual recorded reason): `EBSCO_KB`,
`PROQUEST_SERIALS_SOLUTIONS_KB`, `OVERDRIVE`, `BDS_LIVE`,
`RNIB_BOOKSHARE`, `SCIELO_BOOKS`, `ZOTERO`, `THOTH`, `PUBLISHER_WEBSITE`,
`OTHER`.

Unknown/provisional count at completion: 0 in included values.

### 6.6 Defects

- Current: ProQuest EPUB-only/PDF-ISBN ordering defect
  (`proquestuploader.py` at `7a16edc0`, filename root from
  `get_isbn('PDF')` before the PDF/EPUB fallback). Recorded; not fixed; not
  normalized away.
- Historical/resolved: Project MUSE scheduled-workflow key mismatch, fixed
  by `1a66da8f1700d8c76bf8fda2938b8729be0a93b6` (23 April 2026); not
  presented as current.
- Recorded current behaviour: CUL location writeback under
  `LocationPlatform` `'OTHER'` (`culuploader.py:50`).

## 7. Database and migration effects

Migration added: NO. No database, schema (`thoth-api/src/schema.rs`
untouched), model or migration change of any kind.

## 8. API and compatibility effects

GraphQL/API changes: NONE. Generated schema/client updates: NONE.
Backwards compatibility: unaffected (documentation only). Deprecations:
NONE. Cross-repository dependencies: architecture evidence recorded for
later `BE-02`, `DIS-01`, `MIG-01`, `EXP-01` and `APP-01`; no repository
other than `thoth` was changed.

## 9. Authorization and security

Authorization paths changed: NONE. Roles/scopes involved: none at runtime.
Negative authorization tests: not applicable (documentation only).

Secret and personal-data handling - explicit confirmations:

- no workflow was dispatched (in any repository);
- no uploader was run, in dry-run mode or otherwise;
- no credential was used;
- no production or shared resource was accessed;
- no environment-variable value, secret content, private configuration
  content, private document or email body, or publisher list was read or
  recorded; configuration is recorded as name structure only;
- no production claim was made: `production-verified` count is 0;
- credential information is recorded as category and ownership only.

Security limitations: none introduced.

## 10. Tests and checks (local validation)

Formatting/whitespace:

```text
git diff --check   (clean; no whitespace errors)
```

Changed-path check:

```text
git diff --name-only 32123d3..HEAD   - 10 paths, all documentation or
CHANGELOG.md; no runtime, schema, migration, workflow, CI, app or
dissemination file
```

Link validation: every relative link in the changed documents resolves to
an existing repository file (checked mechanically before commit).

Candidate-coverage validation: all 17 provisional candidates plus all 10
discovered candidates carry exactly one final disposition; every included
value's required fields are populated with no `unknown`/`provisional`.

Sensitive-data validation: changed files contain no secret-like value,
publisher list, private identifier beyond those already present in the
sanitized ledger, or sensitive URL.

Unit/integration tests, lint: not applicable - documentation-only change;
no code compiled or executed. Heavy CI jobs are expected to be skipped by
the documentation-only classifier (section 11).

## 11. CI

Exact-head CI (automatically triggered only; nothing dispatched or rerun):
run IDs, conclusions and the documentation-only skip evidence are recorded
in the immutable exact-head evidence comment on the draft PR. Expected
behaviour for a documentation-only head:

```text
build-test-and-check:      classify success; test/lint/build/format skipped
run-migrations:            classify success; run_migrations skipped
publish-to-dockerhub:      classify success; staging image build skipped
check-changelog:           success
```

## 12. Rollout and rollback

Initial state after merge: documentation and architecture records only;
nothing activates. Activation required: none. Feature
flag/configuration: none. Migration sequence: none. Rollback/disable
procedure: revert the documentation PR; no runtime effect. Monitoring
required: none.

Runtime effects: NONE. Schema effects: NONE. Migration effects: NONE. API
effects: NONE. Workflow effects: NONE. App effects: NONE. Dissemination
effects: NONE. Deployment/release effects: NONE.

## 13. Known limitations and deferred work

- ADR-0004 and the final inventory are PROPOSED; they bind nothing until
  independent review, explicit CTO approval and merge.
- The Thoth-managed source-file invariant is recorded, not enforced;
  enforcement is a separate future HIGH-risk task (current code selects the
  canonical location URL regardless of platform).
- `JISC_NBK` activation (adapter, onboarding controls, operational
  evidence, failure handling, tests, rollout) is a separate future task;
  completion of the initial Jisc production load after the 2026 migration
  remains unconfirmed and blocks that future activation, not this record.
- `EBSCO_KB` and `PROQUEST_SERIALS_SOLUTIONS_KB` remain excluded until a
  later approved decision verifies current operation.
- The ProQuest defect and the CUL `'OTHER'` writeback remain in place;
  fixing them is outside ADR-01 scope.
- Risk classification remains MEDIUM: no evidence gathered showed that an
  inventory decision itself determines live delivery or requires an
  operational change, so no HIGH reclassification was proposed.

## 14. Unresolved issues

NONE for the ADR-01 deliverable itself. Every stop condition was evaluated
and none fired: no active destination was unmappable, no included value
rests on assumption, no shared feed can create duplicate work as specified,
no secret or production access was needed, and the base did not move.

## 15. BE-02 readiness statement

`BE-02` can implement the exhaustive `DistributionPlatform` enum and
code-owned descriptors **without guessing**: every value, display label,
alias resolution, behaviour classification, linked group, shared
adapter/feed identity, mechanism readiness, assignment availability,
back-catalogue expectation, update expectation and withdrawal expectation
is stated in ADR-0004 sections 4 and 5, with no fallback arm and no
unresolved placeholder. BE-02 nevertheless remains BLOCKED until the ADR-01
implementation PR is independently reviewed, explicitly CTO-approved and
merged, and until BE-02's own bounded specification is approved.

## 16. Agent self-assessment

The agent may identify risks but may not approve the task. Suggested review
focus:

- verify every cited repository path at the recorded commits (matrix
  sections 2 and 9);
- verify the provenance-boundary handling (`LEDGER:<ID>` vs
  `CTO-2026-08-06` citations) against ledger section 0.0;
- verify that the 17-value inventory, the 10 exclusions and the
  linked/shared-feed semantics match the merged specification's settled
  inputs exactly;
- verify that no included field is `unknown`/`provisional` and that
  excluded-candidate `unverified` statuses support exclusion only;
- verify the deviation record in section 3 (pre-existing commit-free branch
  at the exact authorized base).
