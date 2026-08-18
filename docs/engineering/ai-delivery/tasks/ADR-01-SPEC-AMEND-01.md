# ADR-01-SPEC-AMEND-01 - ADR-01 specification amendment from the approved evidence ledger

Status: MERGED - COMPLETE
Programme: Publisher Services and Distribution Configuration
Repository: `thoth-pub/thoth`
Workflow: STANDARD
Base branch: `develop`
PR target: `develop`
Programme integration branch: None
Risk: MEDIUM
Owner: CTO
Dependencies: merged ADR-01-SPEC (PR [#780](https://github.com/thoth-pub/thoth/pull/780)); the CTO-approved ADR-01 evidence ledger dated 6 August 2026
Target branch name: `feature/publisher-services/adr-01-spec-amend-01`
Authorized exact base: `590ff437bbd25b8aa5fde800dd8a38772b7e453e`
Independent reviewer: a separate model/agent instance that did not author the amendment

## 1. Objective

Correct and extend the approved ADR-01 written specification
([`ADR-01.md`](ADR-01.md)) using the completed evidence ledger and the explicit
CTO decisions of 2026-08-06, so that the ADR-01 implementation starts from
accurate evidence requirements rather than from statements the ledger has shown
to be wrong, ambiguous or incomplete.

This is a specification-amendment task. Its output is corrected proposed
written content. The corrected content is not approved until fresh independent
exact-head review and explicit CTO approval are recorded.

## 2. Authorization

The CTO authorized:

```text
I authorize the bounded ADR-01-SPEC-AMEND-01 documentation task in
thoth-pub/thoth, starting from freshly verified develop at
590ff437bbd25b8aa5fde800dd8a38772b7e453e, on
feature/publisher-services/adr-01-spec-amend-01. This authorizes only the
approved specification corrections, evidence-ledger integration,
programme-control reconciliation, documentation validation, commits, a draft
PR, automatic CI observation and immutable evidence. It does not authorize
ADR-01 implementation, runtime or related-repository changes, credentials,
production access, workflow dispatch, deployment, release, approval or merge.
```

The CTO also explicitly confirmed on 2026-08-06:

1. `EX_LIBRIS_KB` is a separate destination because it requires separate
   per-publisher onboarding and collection configuration, even when
   commercially bundled. It maps to the shared OCLC KBART feed and must create
   no duplicate feed or uploader job.
2. `JISC_NBK` is a separate destination but is initially inactive and
   non-assignable until its MARC/S3 adapter, onboarding controls and
   operational evidence are separately implemented and approved.
3. There are no known current manual-only distribution destinations outside
   the inspected repositories and internal operational documentation.
4. The conservative initial update and withdrawal policy recorded in the
   amended ADR-01 specification is correct.
5. The Project MUSE specification correction is required.
6. The evidence ledger supplied for this task is approved as the source record
   for drafting the amendment.

This authorization covers drafting and evidence recording only. It is not
approval of the final amended wording.

## 3. Evidence ledger

The mandated source record is the CTO-approved markdown document:

```text
ADR-01 Evidence Ledger: EBSCO, ProQuest and Knowledge-Base Distribution
Prepared: 6 August 2026
SHA-256: 4395c9b7203cdb5c07f5ad6399879827b1964bf8aeb1edc150bfc4d77221e9d7
```

A sanitized, reviewable version is committed at
[`docs/publisher-services/adr-01-evidence-ledger.md`](../../../publisher-services/adr-01-evidence-ledger.md).
The sanitized ledger preserves every evidence ID, source identity, date,
stable Drive/Gmail/repository identifier, exact supported claim, limitation,
current-versus-historical status, the claim-to-evidence index and the
unresolved evidence gaps. It reproduces no full private document, email body,
publisher list, secret value or sensitive object URL.

## 4. Explicit scope

The task must:

1. create this bounded repository task record;
2. correct [`ADR-01.md`](ADR-01.md) as listed in section 6;
3. integrate the sanitized evidence ledger at
   `docs/publisher-services/adr-01-evidence-ledger.md`;
4. reconcile the Publisher Services control documents
   (`README.md`, `decisions.md`, `platform-inventory.md`, `rollout-plan.md`,
   `task-status.md`) and `control-gaps.md` with the proposed amendment state;
5. create the implementation report at
   `docs/engineering/ai-delivery/implementation-reports/ADR-01-SPEC-AMEND-01-implementation-report.md`;
6. update `CHANGELOG.md`;
7. open one draft PR from `feature/publisher-services/adr-01-spec-amend-01`
   to `develop`;
8. observe automatically triggered CI only and post one immutable exact-head
   evidence comment.

## 5. Non-goals

The task must not:

1. perform the ADR-01 implementation;
2. create ADR-0004;
3. produce or approve the final distribution-platform inventory;
4. create an enum, descriptor implementation, migration, API, schema or
   runtime change;
5. edit `thoth-app` or `thoth-dissemination`;
6. inspect production or use credentials;
7. dispatch a workflow;
8. contact vendors, staff or source owners;
9. edit issues #765 or #766;
10. approve the corrected specification;
11. mark the PR ready;
12. merge;
13. deploy, release or activate anything.

## 6. Required substantive corrections

The amendment must apply, without broadening, every correction in the
authorized task prompt:

1. **Approval state:** change the live ADR-01 status to
   `AMENDMENT PROPOSED`, preserving the historical approval record (reviewed
   content head `820f9cfa`, CTO approval 2026-08-05, PR #780) as applying only
   to the superseded pre-amendment content.
2. **Destination versus adapter/feed profile:** add the binding distinction
   that a `DistributionPlatform` is an independently meaningful destination or
   separately onboarded consumer, that an adapter/feed profile is the technical
   mechanism serving one or more destinations, that multiple platform values
   may map to one adapter/feed profile, and that shared adapters and feeds must
   not create duplicate files, feeds, deposits or uploader jobs. No second
   overlapping public business enum is introduced.
3. **Google naming:** record the source-owner-confirmed decision that Google
   Books and Google Play Books are one destination with canonical display name
   `Google Play Books`, `Google Books` a legacy alias, and one stable enum
   code.
4. **EBSCO:** record EBSCO Host and EBSCO Knowledge Base as distinct products;
   `EBSCO_HOST` confirmed current push destination on its SFTP/content/ONIX
   route; `EBSCO_KB` excluded from the initial enum on evidence grounds.
5. **ProQuest and Ex Libris:** record `PROQUEST_EBOOK_CENTRAL` as the
   canonical push destination with legacy aliases (`ProQuest` current-usage
   meaning, `Ebrary`, the `proquest_ebrary` export flavour); `EX_LIBRIS_KB` as
   a separate pull-feed consumer of the shared `OCLC_KBART_PUBLIC` feed
   profile; `PROQUEST_SERIALS_SOLUTIONS_KB` excluded as unverified.
6. **OCLC KBART shared feed:** record `OCLC_KB -> OCLC_KBART_PUBLIC` and
   `EX_LIBRIS_KB -> OCLC_KBART_PUBLIC` as independently onboarded consumer
   assignments of one publisher-level feed profile with duplicate-feed
   prevention required.
7. **Jisc:** replace `JISC_KB` with `JISC_NBK` (display name `Jisc NBK`),
   MARC21-via-S3 mechanism (`JISC_NBK_MARC_S3`), included but initially
   inactive and non-assignable, with the descriptor contract extended to carry
   mechanism readiness/assignment availability.
8. **Explicit initial exclusions:** record the required dispositions for
   `EBSCO_KB`, `PROQUEST_SERIALS_SOLUTIONS_KB`, `OVERDRIVE`, `BDS_LIVE`,
   `RNIB_BOOKSHARE`, `SCIELO_BOOKS`, `ZOTERO`, `THOTH`, `PUBLISHER_WEBSITE`
   and the prohibited `OTHER`.
9. **Updates and withdrawals:** add the conservative initial policy (Crossref
   DOI deposit updates supported; every other automatic push destination's
   automatic updates disabled pending verification; pull feeds expose current
   state; no automatic withdrawals initially).
10. **Operational ownership and configuration authority:** record the
    Statement of Work as commercial entitlement authority, Thoth assignment as
    runtime desired-state authority, temporary operational mirrors, the
    COO/Metadata Specialist RACI and honest credential-transition recording;
    unexplained mismatches fail closed.
11. **Manual destinations:** record the CTO confirmation of 2026-08-06 that no
    known current manual-only destinations exist outside the inspected
    repositories and internal operational documentation.
12. **Thoth-managed source-file invariant:** record the settled requirement
    that automatic push jobs use only Thoth-managed publication files
    (Thoth-managed location plus file record), failing closed otherwise, with
    implementation deferred to a separate future HIGH-risk task.
13. **Project MUSE correction:** remove the requirement to record a current
    scheduled-workflow key mismatch; reclassify it as historical/resolved with
    repository-verified evidence; preserve the current ProQuest
    EPUB-only/PDF-ISBN ordering defect unchanged.
14. **Acceptance criteria and stop conditions:** update both to enforce the
    corrected requirements listed in the authorized task prompt.

## 7. Acceptance criteria

- [ ] Every destination and adapter relationship in the amended specification
  is distinct and explicit.
- [ ] Shared adapters/feeds are required to be duplicate-safe.
- [ ] Included but inactive destinations are required to be non-assignable and
  job-free.
- [ ] Automatic push source files are required to be Thoth-managed.
- [ ] Updates and withdrawals follow the conservative initial policy.
- [ ] Excluded candidates remain absent from the initial enum requirements.
- [ ] Historical defects are not presented as current; current defects are not
  normalized away.
- [ ] The sanitized evidence ledger is committed, complete and independently
  traceable, with evidence IDs matching the approved source ledger.
- [ ] The historical ADR-01 approval record is preserved and scoped to the
  superseded content.
- [ ] Programme control documents are reconciled without pre-recording
  approval or merge.
- [ ] Every changed path is documentation or `CHANGELOG.md`.
- [ ] No ADR-0004 exists in the diff; no runtime, migration, schema, API,
  workflow, app or dissemination path changed.

## 8. Validation

Required local validation:

- `git diff --check`;
- documentation-only changed-path confirmation;
- relative-link resolution;
- no conflict marker, placeholder, private document body, publisher list or
  credential data;
- evidence IDs and source identifiers match the approved ledger;
- Project MUSE no longer described as a current key defect; ProQuest fallback
  defect preserved;
- `JISC_KB` replaced by `JISC_NBK` except where quoted as a historical name;
- re-fetch of `origin/develop` immediately before push, stopping if the
  authorized base moved.

Exact-head CI: observe automatically triggered workflows only. Expected
documentation-only behaviour is classifier success with heavy jobs skipped and
`check-changelog` executing and succeeding.

## 9. Rollout and rollback

- Initial state after merge: documentation and control records only; no
  runtime effect.
- Rollback: revert the documentation PR; no operational effect.
- The amendment PR remained draft throughout the drafting, remediation and
  approval-state phases; it was marked ready and merged only under the CTO
  merge authorization recorded in section 11.1.

## 10. Review and approval gates

Remaining gates after this task completes:

1. fresh independent exact-head review of the amendment PR;
2. explicit CTO approval of the corrected ADR-01 specification content;
3. approval-state documentation commit;
4. fresh exact-head review and CI of that approval-state head;
5. separate CTO merge authorization;
6. merge of ADR-01-SPEC-AMEND-01;
7. deletion or archival handling of the obsolete local ADR-01 branch;
8. fresh ADR-01 implementation authorization from the new `develop` base.

The implementing agent must not approve its own amendment.

## 11. Content approval record

The corrected substantive content produced by this task was approved:

```text
Approved corrected-content head:
1276c70a81e73f57d833eecb0e6886bd0cabf69e

Independent exact-head review:
4873802457 - APPROVED (no P0, P1 or P2 findings)

CTO corrected-content approval:
Javi, CTO, 2026-08-06
PR #781 comment 5203642323
```

The approval applies to the corrected substantive specification content at
that exact head only.

### 11.1 Merge record

The amendment is merged and complete:

```text
PR: #781
Approved amendment head: bdfded20e8cac65fcd7713b07d189052e0eba745
Approval-state final independent review: 4874093991 - APPROVED
CTO merge authorization: review 4874128610
Merge commit: a511e01c83c5e805a75e0fdaeb3b5297c39ef291
Merged at: 2026-08-06T11:29:53Z
```

The corrected ADR-01 specification content is repository-authoritative
through that merge commit. The complete delivery history is preserved: the
CTO drafting authorization of 2026-08-06; the first reviewed head
`3251bd51` receiving `CHANGES REQUIRED` (review `4873502967`, two P1
findings); the remediation commit and substantive approval at `1276c70a`
(review `4873802457`, CTO comment `5203642323`); the approval-state commit
`bdfded20` with its metadata-only `CHANGES REQUIRED` reviews (`4873996906`,
`4874065598`) resolved through PR-body reconciliation; and the final
approval-state review `4874093991`. ADR-01 implementation remains not
authorized: it requires a fresh task authorization from the then-current
exact `develop` head. Post-merge control reconciliation is recorded by
[`ADR-01-SPEC-AMEND-01-CLOSEOUT-01`](ADR-01-SPEC-AMEND-01-CLOSEOUT-01.md).

## 12. Stop conditions

Return `BLOCKED` without pushing misleading content if:

- the authorized base moved;
- the evidence ledger is unavailable;
- the amendment branch or PR already exists;
- the local ADR-01 branch state changed;
- an evidence citation cannot be traced;
- private information cannot be safely sanitized;
- Project MUSE evidence remains contradictory;
- the destination/adapter distinction cannot be expressed without changing
  approved architecture;
- a runtime or related-repository edit becomes necessary;
- a newly discovered decision requires fresh CTO input;
- programme control documents cannot be reconciled consistently.
