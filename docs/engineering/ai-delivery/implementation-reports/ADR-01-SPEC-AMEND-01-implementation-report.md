# ADR-01-SPEC-AMEND-01 Implementation Report

## 1. Repository state

Repository: `thoth-pub/thoth`
Workflow: STANDARD
Base branch: `develop`
Base commit: `590ff437bbd25b8aa5fde800dd8a38772b7e453e` (verified equal to
`origin/develop` at preflight, at branch creation, before the first commit
and immediately before push)
PR target: `develop`
Programme integration branch: None
Task branch: `feature/publisher-services/adr-01-spec-amend-01`
Head commit: the report/changelog commit; the exact head SHA is recorded in
the immutable exact-head evidence comment on the pull request
Pull request: [#781](https://github.com/thoth-pub/thoth/pull/781) (draft)
Expected branch deletion after merge: YES
Final programme PR required: NO
Implementing model: Claude (Fable 5), implementing agent for the bounded
amendment task
Reasoning level: HIGH

## 2. Scope confirmation

Approved specification: the CTO-authorized `ADR-01-SPEC-AMEND-01` task prompt
of 2026-08-06, recorded in
[`ADR-01-SPEC-AMEND-01.md`](../tasks/ADR-01-SPEC-AMEND-01.md).

Implemented objective: correct and extend the approved ADR-01 written
specification using the CTO-approved evidence ledger and the explicit CTO
decisions of 2026-08-06; integrate the sanitized ledger; reconcile the
Publisher Services control documents; produce this report, the changelog
entry, one draft PR, exact-head CI observation and immutable evidence.

Out-of-scope changes made: NONE.

## 3. Commits

- `d2ace3de` - docs(publisher-services): specify ADR-01 evidence amendment
  (task record and sanitized evidence ledger)
- `887af81f` - docs(publisher-services): correct ADR-01 platform evidence
  requirements (ADR-01 specification corrections and control-document
  reconciliation)
- this commit - docs(publisher-services): report ADR-01 specification
  amendment (implementation report and changelog)

Normal commits only; no amend, rebase, squash or force-push.

## 4. Files changed

- `docs/engineering/ai-delivery/tasks/ADR-01-SPEC-AMEND-01.md`
  - reason: bounded repository task record for the amendment.
  - behavioural effect: none (documentation/control record).
- `docs/publisher-services/adr-01-evidence-ledger.md`
  - reason: sanitized, reviewable version of the CTO-approved evidence
    ledger (source SHA-256
    `4395c9b7203cdb5c07f5ad6399879827b1964bf8aeb1edc150bfc4d77221e9d7`).
  - behavioural effect: none.
- `docs/engineering/ai-delivery/tasks/ADR-01.md`
  - reason: the substantive specification corrections listed in section 5.
  - behavioural effect: none; the corrected content is proposed, not
    approved.
- `docs/engineering/repository-map/control-gaps.md`
  - reason: record the CG-07 amendment state; CG-07 remains open; CG-11 and
    CG-13 unchanged.
- `docs/publisher-services/README.md`
  - reason: programme status reconciliation with the proposed amendment
    state.
- `docs/publisher-services/decisions.md`
  - reason: record the amendment-proposed inputs to the decisions delegated
    to ADR-01, explicitly pending approval.
- `docs/publisher-services/platform-inventory.md`
  - reason: correct the inaccurate current-defect claim for Project MUSE,
    preserve the current ProQuest defect, and record amendment-proposed
    dispositions; the inventory remains explicitly provisional.
- `docs/publisher-services/rollout-plan.md`
  - reason: add the `ADR-01-SPEC -> ADR-01-SPEC-AMEND-01 -> ADR-01 -> BE-02`
    gate to the dependency record.
- `docs/publisher-services/task-status.md`
  - reason: add the amendment task row (`IN PROGRESS / CORRECTED CONTENT
    PROPOSED`), set ADR-01 to `BLOCKED` pending the approved and merged
    amendment, and update next actions.
- `docs/engineering/ai-delivery/implementation-reports/ADR-01-SPEC-AMEND-01-implementation-report.md`
  - reason: this report.
- `CHANGELOG.md`
  - reason: required Unreleased entries for PR #781.

Every changed path is documentation or `CHANGELOG.md`.

## 5. Implementation decisions

Substantive corrections applied (all from the authorized task prompt; none
invented):

1. **Approval state:** ADR-01 status set to `AMENDMENT PROPOSED`; the
   historical approval (content head
   `820f9cfa22d284f8f347db338aa2461408f4ed12`, review `4866683359`, CTO
   approval 2026-08-05, PR #780) is preserved in ADR-01 sections 0 and 19.1
   and applies only to the superseded pre-amendment content.
2. **Destination versus adapter/feed profile:** binding distinction added as
   ADR-01 section 7.1 with new invariants 17-20; no second overlapping
   public business enum (for example `PublisherDestination`) is introduced;
   ADR-01 decides platform values, BE-02 later implements exhaustive
   descriptors, dissemination later implements the adapter mapping.
3. **Google naming:** source-owner-confirmed single destination; canonical
   display name `Google Play Books`; `Google Books` a legacy alias; one
   stable enum code (ADR-01 section 8.1).
4. **EBSCO:** `EBSCO_HOST` confirmed current push destination on its
   SFTP/content/ONIX route; `EBSCO_KB` distinct product excluded from the
   initial enum as currently unverified; historical KB evidence not
   represented as current (ADR-01 section 8.2; ledger EBSCO-01..05).
5. **ProQuest/Ex Libris:** canonical `PROQUEST_EBOOK_CENTRAL`; legacy
   aliases `ProQuest` (current usage), `Ebrary` and the `proquest_ebrary`
   export flavour; umbrella vendor name not an enum value; `EX_LIBRIS_KB` a
   separate destination and pull-feed consumer of the shared
   `OCLC_KBART_PUBLIC` feed profile; `PROQUEST_SERIALS_SOLUTIONS_KB`
   excluded as unverified; the three are not described as aliases (ADR-01
   section 8.3; ledger PROQUEST-01..06, KBART-02/03).
6. **OCLC KBART shared feed:** `OCLC_KB -> OCLC_KBART_PUBLIC` and
   `EX_LIBRIS_KB -> OCLC_KBART_PUBLIC` as independently onboarded consumer
   assignments of one publisher-level feed profile; duplicate feed
   generation must be prevented; no inference that EBSCO KB or Serial
   Solutions consumes the feed (ADR-01 section 8.4; ledger KBART-01..03).
7. **Jisc:** `JISC_KB` replaced by `JISC_NBK` (display `Jisc NBK`); MARC21
   files through S3; proposed adapter `JISC_NBK_MARC_S3`; included but
   initially inactive and non-assignable, creating no job or delivery until
   a separate implementation task delivers and approves the adapter,
   onboarding controls, operational evidence, failure handling, tests and
   rollout controls; the required platform record (section 5) and descriptor
   contract (section 9.1) now carry mechanism readiness / assignment
   availability (ADR-01 section 8.5; ledger JISC-01/02).
8. **Explicit initial exclusions:** `EBSCO_KB`,
   `PROQUEST_SERIALS_SOLUTIONS_KB`, `OVERDRIVE`, `BDS_LIVE`,
   `RNIB_BOOKSHARE`, `SCIELO_BOOKS`, `ZOTERO`, `THOTH`,
   `PUBLISHER_WEBSITE`; `OTHER` prohibited; export formats, export-registry
   consumers, locations, aliases and vendor family names are not
   automatically destinations; exclusion does not require deleting existing
   export-registry or `LocationPlatform` values (ADR-01 section 8.6).
9. **Updates and withdrawals:** conservative initial policy added as ADR-01
   section 9.3 exactly as CTO-confirmed.
10. **Operational ownership and configuration authority:** recorded as
    ADR-01 section 9.4 (Statement of Work; Thoth assignment; temporary
    operational mirrors; COO accountable; Metadata Specialist responsible
    and target credential owner; honest recording of shared credential
    responsibility during transition; fail-closed mismatch handling; no
    publisher lists in the repository).
11. **Manual destinations:** the CTO confirmation of 2026-08-06 recorded as
    ADR-01 section 8.7 and ledger section 9, explicitly as
    source-owner-confirmed evidence, not a permanent claim.
12. **Thoth-managed source-file invariant:** recorded as ADR-01 section 7.2;
    requirement only; implementation is a separate future HIGH-risk task; no
    source-selection code or dissemination behaviour changed by this PR.
13. **Project MUSE:** requirement to record a current key mismatch removed;
    reclassified `Historical/resolved Project MUSE key mismatch; not a
    current defect` (verification in section 10 below).
14. **ProQuest defect preservation:** the current EPUB-only/PDF-ISBN
    ordering defect remains recorded in ADR-01 section 8.8 and
    `platform-inventory.md` section 5, and is not fixed.
15. **Acceptance and stop conditions:** ADR-01 sections 10 and 12 updated to
    enforce the distinct destination/adapter relationships, duplicate-safe
    shared feeds, non-assignable inactive destinations, Thoth-managed source
    files, the conservative update/withdrawal policy, the exclusions, the
    historical/current defect separation and independent ledger
    traceability.

List of deviations from the specification: NONE.

## 6. Database and migration effects

Migration added: NO. No database, schema, `thoth-api/src/schema.rs`, model or
data effect of any kind.

## 7. API and compatibility effects

GraphQL/API changes: NONE.
Generated schema/client updates: NONE.
Backwards compatibility: unaffected; documentation only.
Deprecations: NONE.
Cross-repository dependencies: none changed. `thoth-app` and
`thoth-dissemination` were not edited; `thoth-dissemination` was inspected
read-only for the Project MUSE and ProQuest verification.

## 8. Authorization and security

Authorization paths changed: NONE.
Roles/scopes involved: none at runtime; the amendment records the intended
operational RACI as documentation.
Negative authorization tests: not applicable (documentation only).
Secret or personal-data handling: no secret value, credential, private
environment content, sensitive object URL, private publisher list, full
private document or full email body was read into the repository or
recorded. The sanitized ledger retains only stable access-controlled
Drive/Gmail identifiers and named-role attributions required for authorized
traceability.
Security limitations: none introduced.

## 9. Tests and checks

### Formatting / whitespace

Command:

```text
git diff --check 590ff437bbd25b8aa5fde800dd8a38772b7e453e..HEAD
```

Result:

```text
exit 0; no whitespace errors, no conflict markers
```

### Documentation-only path check

Command:

```text
git diff --name-only 590ff437bbd25b8aa5fde800dd8a38772b7e453e..HEAD
```

Result:

```text
11 paths; all under docs/ except CHANGELOG.md; no runtime, migration,
schema, API, workflow, CI, app or dissemination path changed; no ADR-0004
file exists in the diff
```

### Relative-link validation

Command:

```text
python3 link checker over every changed markdown file (resolves each
relative link target against the repository tree)
```

Result:

```text
ALL RELATIVE LINKS RESOLVE
```

### Ledger traceability validation

Command:

```text
diff of evidence-ID sets and of all Drive/Gmail stable identifiers between
the approved source ledger (SHA-256 verified) and
docs/publisher-services/adr-01-evidence-ledger.md
```

Result:

```text
18/18 evidence IDs identical (EBSCO-01..05, PROQUEST-01..06, KBART-01..03,
JISC-01..02, HIST-01, ADR-01-SOURCE); all source identifiers preserved
exactly
```

### Sensitive-data scan

Command:

```text
grep of the full diff for secret-like patterns (passwords, keys, tokens,
private key blocks, AWS key IDs)
```

Result:

```text
NO SECRET-LIKE CONTENT IN DIFF
```

### Terminology checks

Result:

```text
JISC_KB appears only as a quoted historical name; EBSCO_KB and
PROQUEST_SERIALS_SOLUTIONS_KB appear only as exclusions; OTHER appears only
as a prohibition; no non-historical Project MUSE current-defect claim
remains; the ProQuest current defect remains recorded
```

### Unit / integration / lint

Not applicable: documentation-only change; no Rust, migration or workflow
surface touched. Heavy CI jobs are expected to be skipped by the
documentation-only classifier.

## 10. Manual verification

Environment: local read-only clones of `thoth-pub/thoth` and
`thoth-pub/thoth-dissemination` (fetched from origin; no checkout of, or
modification to, any `thoth-dissemination` branch).

Project MUSE verification (repository-verified):

- at release commit `7a16edc08d4570f3ecc108453298a3aa43f6d753`,
  `.github/workflows/muse_bulk_disseminate.yaml` passes
  `platform: 'ProjectMUSE'` and `disseminator.py` `UPLOADERS` accepts
  `ProjectMUSE`;
- the identical matching state exists at the provisional baseline
  `5e88ce1b58e5f962cc4f4ef6fb00c08f50b57add`;
- the historical fixing commit was independently verified:
  `1a66da8f1700d8c76bf8fda2938b8729be0a93b6`, "Correct mistyped platform
  name in Project MUSE GitHub Action file" (23 April 2026), a one-line
  change of `platform: 'MUSE'` to `platform: 'ProjectMUSE'`, confirmed by
  `git merge-base --is-ancestor` to be an ancestor of both the provisional
  baseline and the current release commit.

ProQuest defect verification (repository-verified): at
`7a16edc08d4570f3ecc108453298a3aa43f6d753`, `proquestuploader.py` sets the
filename root from `get_isbn('PDF')` before the PDF/EPUB fallback, with the
in-code comment that this fails if only an EPUB exists. The defect is
current and is preserved, not fixed.

## 11. CI

CI status: recorded in the immutable exact-head evidence comment on
[PR #781](https://github.com/thoth-pub/thoth/pull/781) after the
automatically triggered workflows complete.
Checks: automatic workflows only; nothing was dispatched or rerun manually.
Expected documentation-only behaviour: classifiers succeed; heavy
build/test/lint/format, migration and Docker jobs skipped;
`check-changelog` executes and succeeds.

## 12. Rollout and rollback

Initial state after merge: documentation and control records only; no
runtime effect; the corrected ADR-01 content becomes repository-authoritative
only on merge after fresh independent review and explicit CTO approval.
Activation required: NONE.
Feature flag/configuration: not applicable.
Migration sequence: not applicable.
Rollback/disable procedure: revert the documentation PR.
Monitoring required: none.

## 13. Known limitations and deferred work

- The corrected ADR-01 content is proposed, not approved; every remaining
  gate is listed in the amendment task record section 10.
- The Thoth-managed source-file invariant is recorded, not implemented;
  implementation is a separate future HIGH-risk task.
- `JISC_NBK` activation requires a separately implemented and approved
  MARC/S3 adapter with onboarding controls and operational evidence.
- The ledger's unresolved gaps (EBSCO KB, Serial Solutions, `ProQuest` SLA
  label ambiguity, Clarivate commercial bundling, Jisc initial-load
  completion) remain unresolved and support exclusion, not inclusion.
- The obsolete local pre-amendment `feature/publisher-services/adr-01`
  branch (clean, unpushed, commit-free) awaits deletion or archival handling
  under a later gate; this task did not touch it.

## 14. Unresolved issues

- NONE within the authorized scope.

## 15. Agent self-assessment

The implementing agent may identify risks but may not approve the task.

Suggested review focus:

- confirm every ADR-01 correction against the authorized amendment prompt
  and the sanitized ledger, and that no correction broadens scope;
- confirm the sanitized ledger against the approved source record (SHA-256
  above): evidence IDs, identifiers, claims, limitations, statuses and gaps;
- independently re-verify the Project MUSE evidence at
  `7a16edc08d4570f3ecc108453298a3aa43f6d753`,
  `5e88ce1b58e5f962cc4f4ef6fb00c08f50b57add` and
  `1a66da8f1700d8c76bf8fda2938b8729be0a93b6`;
- confirm the control documents record the amendment as proposed, never as
  approved or merged;
- confirm the diff is documentation-only and contains no sensitive data.
