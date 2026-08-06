# ADR-0004 - Distribution platform inventory

Status: PROPOSED - INDEPENDENT REVIEW AND CTO APPROVAL REQUIRED
Date: 2026-08-06
Decision owner: CTO
Programmes affected: Publisher Services and Distribution Configuration
Repositories affected: `thoth` (decision record only; future `BE-02`), with
architecture evidence for later `thoth-dissemination` (`DIS-01`) and
`thoth-app` (`APP-01`) work
Supersedes: the provisional baseline previously recorded in
[`docs/publisher-services/platform-inventory.md`](../../publisher-services/platform-inventory.md)
Superseded by: None

## 1. Context

Publisher Services requires a closed, exhaustive, user-visible
`DistributionPlatform` inventory so that `BE-02` can implement an exhaustive
enum with code-owned descriptors and no fallback arm, and so that
dissemination can later map every value to exactly one delivery behaviour
without ambiguity or duplicate delivery.

The inventory decision was delegated to ADR-01 by the approved programme
design and the merged, CTO-approved ADR-01 specification
([`docs/engineering/ai-delivery/tasks/ADR-01.md`](../ai-delivery/tasks/ADR-01.md),
approved corrected-content head `1276c70a`, repository-authoritative through
PR [#781](https://github.com/thoth-pub/thoth/pull/781)). This ADR is the
ADR-01 decision record, produced under the explicit CTO implementation
authorization of 2026-08-06 from exact base
`32123d363a6806d377ac322e3814fb432a803453`.

This is an evidence and architecture decision only. It writes no runtime
code, enum, migration, schema, API or workflow, and it activates nothing.

## 2. Decision drivers

- An exhaustive closed enum with no `OTHER` and no fallback (settled
  invariants 2, 3).
- Destinations distinct from the adapters and feed profiles serving them
  (settled invariant 17).
- No duplicate uploads, feeds, deposits or jobs from linked or shared
  mechanisms (settled invariants 7, and specification section 8.4).
- Explicit push / pull-feed / manual behaviour for every value (settled
  invariant 11).
- Explicit representation of included-but-inactive destinations (settled
  invariant 18).
- Evidence-based inclusion only: no `unknown` or `provisional` value enters
  the enum; unresolved status supports exclusion, never inclusion.
- Fail-closed configuration and conservative initial update/withdrawal
  policy (settled invariants 12, 13, 20).

## 3. Evidence methodology

The complete per-candidate evidence, exact commits, paths, attributions,
classifications and counts are recorded in the
[ADR-01 evidence matrix](../../publisher-services/adr-01-evidence-matrix.md).
Summary:

- read-only inspection of `thoth-pub/thoth` at the exact authorized base
  `32123d363a6806d377ac322e3814fb432a803453` (plus the deferred, unmodified
  `feature/oai-pmh-http` branch at `745dd020` for context);
- read-only inspection of `thoth-pub/thoth-dissemination` at default-branch
  head `7a16edc08d4570f3ecc108453298a3aa43f6d753` (= release 1.6.4, the
  commit already cited by the evidence ledger; drift from the provisional
  baseline `5e88ce1b` is Internet Archive hardening only and is recorded);
- read-only inspection of `thoth-pub/thoth-app` at `main` `6f826390` and
  `dev` `26323158` (duplication evidence only, never inventory authority);
- the sanitized CTO-approved
  [evidence ledger](../../publisher-services/adr-01-evidence-ledger.md)
  (original 18 entries, provenance-bounded) as the authorized source-owner
  evidence record, plus the separately attributable CTO decisions of
  2026-08-06;
- every claim classified as exactly one of `repository-verified`,
  `source-owner-confirmed`, `production-verified`, `provisional`, `unknown`;
  final counts: 34 repository-verified, 21 source-owner-confirmed,
  0 production-verified, 0 provisional or unknown in included values;
- no workflow dispatched, no uploader run, no credential used, no production
  or shared resource accessed, no secret or private configuration content
  read or recorded.

## 4. Decision

### 4.1 Final exhaustive inventory

`DistributionPlatform` comprises exactly these 17 values:

| Enum code | Display label | Behaviour | Linked group | Adapter / feed profile | Assignment availability |
|---|---|---|---|---|---|
| `INTERNET_ARCHIVE` | Internet Archive | AutomaticPush | - | `IA_API` | assignable |
| `OAPEN` | OAPEN | AutomaticPush | `OAPEN_DOAB` | `OAPEN_DOAB_SWORD` (shared) | assignable (linked selection) |
| `DOAB` | DOAB | AutomaticPush (via linked deposit) | `OAPEN_DOAB` | `OAPEN_DOAB_SWORD` (shared) | assignable (linked selection) |
| `SCIENCE_OPEN` | ScienceOpen | Manual | - | `SCIENCEOPEN_FTP` | assignable (job-free) |
| `CAMBRIDGE_UNIVERSITY_LIBRARY` | Cambridge University Library | AutomaticPush | - | `CUL_SWORD` | assignable |
| `CROSSREF` | Crossref | AutomaticPush (metadata-only DOI deposit) | - | `CROSSREF_DOI_DEPOSIT` | assignable |
| `FIGSHARE` | Figshare | AutomaticPush | - | `FIGSHARE_API` | assignable |
| `ZENODO` | Zenodo | AutomaticPush | - | `ZENODO_API` | assignable |
| `PROJECT_MUSE` | Project MUSE | AutomaticPush | - | `MUSE_FTP` | assignable |
| `JSTOR` | JSTOR | AutomaticPush | - | `JSTOR_FTP` | assignable |
| `EBSCO_HOST` | EBSCOHost | AutomaticPush | - | `EBSCO_HOST_SFTP` | assignable |
| `PROQUEST_EBOOK_CENTRAL` | ProQuest Ebook Central | AutomaticPush | - | `PROQUEST_EBOOK_CENTRAL_FTP` | assignable |
| `GOOGLE_PLAY` | Google Play Books | AutomaticPush | - | `GOOGLE_PLAY_GCS` | assignable |
| `BKCI` | Book Citation Index | AutomaticPush | - | `BKCI_FTP` | assignable |
| `OCLC_KB` | OCLC Knowledge Base | PullFeed | - | `OCLC_KBART_PUBLIC` (shared) | assignable (feed membership; job-free) |
| `EX_LIBRIS_KB` | Ex Libris Knowledge Base | PullFeed | - | `OCLC_KBART_PUBLIC` (shared) | assignable (feed membership; job-free) |
| `JISC_NBK` | Jisc NBK | AutomaticPush class when active | - | `JISC_NBK_MARC_S3` (inactive) | **non-assignable; inactive; job-free** |

There is no `OTHER`, catch-all or fallback value. The enum is exhaustive:
every active channel observed in the inspected repositories maps to exactly
one value above or to an explicit exclusion in section 4.8.

### 4.2 Aliases and legacy names

All aliases resolve to stable codes; no alias is a separate value:

| Stable code | Aliases and legacy names |
|---|---|
| `INTERNET_ARCHIVE` | uploader key `InternetArchive` |
| `OAPEN` | uploader key `OAPEN` |
| `DOAB` | export-registry platform `doab` |
| `SCIENCE_OPEN` | uploader key `ScienceOpen` |
| `CAMBRIDGE_UNIVERSITY_LIBRARY` | uploader key `CUL` |
| `CROSSREF` | uploader key `Crossref`; legacy capitalization `CrossRef` |
| `FIGSHARE` | uploader key `Figshare` |
| `ZENODO` | uploader key `Zenodo` |
| `PROJECT_MUSE` | uploader key `ProjectMUSE`; historical workflow key `MUSE` (resolved) |
| `JSTOR` | uploader key `JSTOR` |
| `EBSCO_HOST` | uploader key `EBSCOHost`; `EBSCO Host` (location/registry) |
| `PROQUEST_EBOOK_CENTRAL` | `ProQuest` (current uploader key and SLA usage); `Ebrary`; export flavour `proquest_ebrary` |
| `GOOGLE_PLAY` | `Google Books` (legacy/user-recognized alias; location value; registry `google_books`); uploader key `GooglePlay` |
| `BKCI` | Clarivate Web of Science Book Citation Index |
| `OCLC_KB` | `OCLC KB` (location/registry); app label `OCLC` |
| `EX_LIBRIS_KB` | `ProQuest ExLibris` (location/registry/app label); historical `ProQuest - was ExLibris` |
| `JISC_NBK` | `Jisc KB` / `JISC KB` (superseded label; location/registry/app) |

`ProQuest` as a vendor umbrella is not an enum value. ProQuest Ebook
Central, Ex Libris and Serial Solutions are not aliases of one another.
Historical EBSCO KB evidence is not described as current operation.

### 4.3 Linked groups and shared adapters/feeds

- `OAPEN_DOAB`: OAPEN and DOAB are separate values, linked for initial
  selection. Backend logic owns linked-platform normalization; one linked
  activation produces exactly one logical delivery through the shared
  `OAPEN_DOAB_SWORD` adapter and can never upload twice. `thoth-app` later
  consumes backend-provided linkage metadata and maintains no independent
  linkage rule.
- `OCLC_KBART_PUBLIC`: one publisher-level Thoth OCLC KBART output profile
  (export specification `kbart::oclc`) serves `OCLC_KB` and `EX_LIBRIS_KB`
  as independently onboarded consumer assignments. Enabling both must not
  generate or maintain duplicate KBART outputs or feed state, and creates no
  uploader job. It must **not** be inferred that EBSCO KB or Serial
  Solutions consumes this feed.
- `JISC_NBK_MARC_S3`: explicitly inactive mechanism identity for `JISC_NBK`
  (MARC21 `.mrc` per book via S3 - not OCLC KBART). Creates no job or
  delivery while inactive.
- `SwordV2Uploader`/`DSpaceUploader` code reuse between the OAPEN and CUL
  adapters is shared implementation, not a shared destination or deposit.

### 4.4 Duplicate-prevention requirements

1. One linked `OAPEN_DOAB` activation yields one logical delivery job and
   one deposit.
2. One `OCLC_KBART_PUBLIC` feed per publisher; consumer assignments add no
   feed state and no jobs.
3. Pull-feed and manual destinations never create uploader jobs.
4. An included-but-inactive destination (`JISC_NBK`) is non-assignable and
   creates no job or delivery.
5. Backfill of existing assignments creates no back-catalogue jobs
   (`MIG-01` no-job import mode).
6. Empty assignment lists are successful no-ops and never broaden
   processing.

### 4.5 Conservative initial update and withdrawal policy

Adopted as confirmed by the CTO on 2026-08-06:

```text
Incremental updates

- Crossref DOI deposits: supported.
- Every other automatic push destination: automatic updates disabled until
  the platform-specific update semantics are separately verified and
  approved.
- Pull-feed destinations expose current feed state and create no
  push-update job.
- Manual destinations create no automatic update behaviour.

Withdrawals

- No automatic withdrawal is supported for any destination initially.
- Withdrawal remains an operator-managed external process where the
  destination permits removal.
- A destination that does not permit removal is recorded as such only when
  verified.
- Any automated update or withdrawal support requires separately specified,
  reviewed and approved work.
```

No remote update or withdrawal capability is claimed without evidence.

### 4.6 Thoth-managed source-file invariant

Binding architecture, recorded here and implemented later:

```text
An automatic push-dissemination job may use only a publication file managed
by Thoth and represented by:

1. the appropriate Thoth-managed location; and
2. a corresponding Thoth file record.

Publisher Website locations, arbitrary external canonical URLs and other
publisher-hosted URLs are not eligible automatic-dissemination source files.

Failure to locate an eligible Thoth-managed file fails closed and creates no
external-delivery attempt.
```

Current dissemination behaviour does not yet enforce this: at
`thoth-dissemination@7a16edc0`, `uploader.py` selects each publication's
canonical-location `fullTextUrl` regardless of hosting platform. This ADR
records the requirement only. Its future runtime implementation is a
separate HIGH-risk task requiring migration/compatibility assessment, tests,
comparison mode, pilot controls, monitoring and rollback. No source-selection
code changes are made by ADR-01.

### 4.7 Ownership and configuration authority

```text
Commercial entitlement authority:
Statement of Work agreed with the publisher

Runtime desired-state authority:
Thoth publisher/platform assignment

Temporary operational mirrors:
environment publisher lists, uploader configuration and existing workflow
configuration

Accountable operational owner:
COO

Operationally responsible role:
Metadata Specialist

Target credential owner:
Metadata Specialist

Current credential responsibility:
shared during transition; recorded honestly per destination in the evidence
matrix
```

Any unexplained mismatch between the Statement of Work, the Thoth assignment
and temporary operational configuration fails closed and must not broaden
processing. No publisher list is copied into the repository.

### 4.8 Excluded candidates

Each exclusion is individual and evidence-based; none deletes an existing
export-registry or `LocationPlatform` value:

| Candidate | Reason |
|---|---|
| `EBSCO_KB` | distinct EBSCO product (not an alias of EBSCOHost); historically separate metadata route; **current** independently selectable workflow, SLA selection, endpoint and publisher configuration unverified; revisitable through a later approved decision |
| `PROQUEST_SERIALS_SOLUTIONS_KB` | historically distinct Serial Solutions route; current workflow, endpoint, publisher coverage and selectable status unverified; revisitable through a later approved decision |
| `OVERDRIVE` | export-registry entry only; no uploader, workflow, configuration or evidence of current operation; excluded by CTO decision 2026-08-06 |
| `BDS_LIVE` | export-registry entry only (consumes the JSTOR ONIX flavour); no mechanism or current-operation evidence; excluded by CTO decision 2026-08-06 |
| `RNIB_BOOKSHARE` | export-registry entry only (consumes the EBSCOHost ONIX flavour); no mechanism or current-operation evidence; excluded by CTO decision 2026-08-06 |
| `SCIELO_BOOKS` | location value only; no mechanism or current-operation evidence; excluded by CTO decision 2026-08-06 |
| `ZOTERO` | end-user reference-manager export target (BibTeX), not a publisher distribution destination; excluded by CTO decision 2026-08-06 |
| `THOTH` | internal managed-location/file-hosting concept; central to the source-file invariant; not a distribution destination |
| `PUBLISHER_WEBSITE` | publisher-managed location, not a Thoth-operated delivery destination; not an eligible automatic-dissemination source under section 4.6 |
| `OTHER` | prohibited by existing architecture; no catch-all or fallback value may exist |

### 4.9 Manual destinations

As of 2026-08-06 the CTO confirms there are no known current manual-only
distribution destinations outside the inspected repositories and internal
operational evidence (source-owner-confirmed; a statement of current
knowledge, not a permanent claim). The only Manual value in the initial
inventory is `SCIENCE_OPEN`, whose uploader exists and is staff-invoked.

### 4.10 Known defects

- **Current defect (recorded, not fixed):** the ProQuest uploader's
  intended EPUB-only fallback can fail because the content filename root is
  taken from `get_isbn('PDF')` before the PDF/EPUB fallback
  (`proquestuploader.py` at `7a16edc0`). ProQuest scheduled delivery must
  not be presented as fully healthy while this stands. Fixing it is outside
  ADR-01 scope.
- **Historical/resolved (not current):** the Project MUSE scheduled-workflow
  key mismatch (`platform: 'MUSE'`) was fixed by
  `1a66da8f1700d8c76bf8fda2938b8729be0a93b6` (23 April 2026); the workflow
  and registry match at both the provisional baseline and the current
  release commit.
- **Known current behaviour (recorded):** the CUL uploader writes back
  locations under `LocationPlatform` `'OTHER'` (`culuploader.py:50`);
  revisiting that is future implementation work, not an inventory question.

## 5. BE-02 descriptor contract

`BE-02` implements code-owned operational metadata with an exhaustive
compile-time mapping and **no fallback arm**. For every enum value the
descriptor must state exactly:

| Value | Display label | Behaviour | Linked group | Adapter/feed profile | Mechanism readiness | Assignment availability | Back-catalogue expectation | Update expectation | Withdrawal expectation |
|---|---|---|---|---|---|---|---|---|---|
| `INTERNET_ARCHIVE` | Internet Archive | push | none | `IA_API` | active | assignable | job on activation | disabled | none |
| `OAPEN` | OAPEN | push | `OAPEN_DOAB` | `OAPEN_DOAB_SWORD` | active | assignable (linked) | one linked logical job | disabled | none |
| `DOAB` | DOAB | push | `OAPEN_DOAB` | `OAPEN_DOAB_SWORD` | active | assignable (linked) | via linked logical job; never a second upload | disabled | none |
| `SCIENCE_OPEN` | ScienceOpen | manual | none | `SCIENCEOPEN_FTP` | active (manual invocation) | assignable | no job | none | none |
| `CAMBRIDGE_UNIVERSITY_LIBRARY` | Cambridge University Library | push | none | `CUL_SWORD` | active | assignable | job on activation | disabled | none |
| `CROSSREF` | Crossref | push | none | `CROSSREF_DOI_DEPOSIT` | active | assignable | job on activation | **supported** (DOI redeposit) | none |
| `FIGSHARE` | Figshare | push | none | `FIGSHARE_API` | active | assignable | job on activation | disabled | none |
| `ZENODO` | Zenodo | push | none | `ZENODO_API` | active | assignable | job on activation | disabled | none |
| `PROJECT_MUSE` | Project MUSE | push | none | `MUSE_FTP` | active | assignable | job on activation | disabled | none |
| `JSTOR` | JSTOR | push | none | `JSTOR_FTP` | active | assignable | job on activation | disabled | none |
| `EBSCO_HOST` | EBSCOHost | push | none | `EBSCO_HOST_SFTP` | active | assignable | job on activation | disabled | none |
| `PROQUEST_EBOOK_CENTRAL` | ProQuest Ebook Central | push | none | `PROQUEST_EBOOK_CENTRAL_FTP` | active (current defect recorded) | assignable | job on activation | disabled | none |
| `GOOGLE_PLAY` | Google Play Books | push | none | `GOOGLE_PLAY_GCS` | active | assignable | job on activation | disabled | none |
| `BKCI` | Book Citation Index | push | none | `BKCI_FTP` | active | assignable | job on activation | disabled | none |
| `OCLC_KB` | OCLC Knowledge Base | pull-feed | none | `OCLC_KBART_PUBLIC` | active | assignable (feed membership) | no job; feed exposes state | feed state only | feed state only |
| `EX_LIBRIS_KB` | Ex Libris Knowledge Base | pull-feed | none | `OCLC_KBART_PUBLIC` | active | assignable (feed membership) | no job; feed exposes state | feed state only | feed state only |
| `JISC_NBK` | Jisc NBK | push (when active) | none | `JISC_NBK_MARC_S3` | **inactive** | **non-assignable** | none while inactive | none | none |

Contract semantics:

- the mapping is exhaustive over the enum with no default/fallback arm;
- descriptors are code-owned operational metadata, not publisher-editable
  rows;
- `mechanism readiness` and `assignment availability` are explicit
  first-class descriptor dimensions so that an included-but-inactive value
  is represented without special cases;
- linked-group membership is backend metadata consumed (later) by
  `thoth-app`; the app maintains no independent linkage rule;
- shared adapter/feed identity is metadata; job/feed deduplication is
  enforced by backend logic, not by descriptor consumers.

ADR-01 specifies content and semantics only. It writes no Rust, chooses no
function signatures, creates no table, migration or GraphQL surface, and
implements no descriptor.

## 6. Future dissemination mapping (architecture evidence for DIS-01)

| Enum value | Current dissemination route |
|---|---|
| `INTERNET_ARCHIVE` | uploader key `InternetArchive` (`IAUploader`) |
| `OAPEN` | uploader key `OAPEN` (`OAPENSWORDUploader`) - one shared linked delivery |
| `DOAB` | no own uploader key; delivered by the same linked `OAPEN` deposit; never a second job |
| `SCIENCE_OPEN` | uploader key `ScienceOpen` (`SOUploader`), manual invocation only; no scheduled or automatic job |
| `CAMBRIDGE_UNIVERSITY_LIBRARY` | uploader key `CUL` (`CULUploader`) |
| `CROSSREF` | uploader key `Crossref` (`CrossrefUploader`) |
| `FIGSHARE` | uploader key `Figshare` (`FigshareUploader`) |
| `ZENODO` | uploader key `Zenodo` (`ZenodoUploader`) |
| `PROJECT_MUSE` | uploader key `ProjectMUSE` (`MUSEUploader`) |
| `JSTOR` | uploader key `JSTOR` (`JSTORUploader`) |
| `EBSCO_HOST` | uploader key `EBSCOHost` (`EBSCOUploader`) |
| `PROQUEST_EBOOK_CENTRAL` | uploader key `ProQuest` (`ProquestUploader`) |
| `GOOGLE_PLAY` | uploader key `GooglePlay` (`GooglePlayUploader`) |
| `BKCI` | uploader key `BKCI` (`BKCIUploader`) |
| `OCLC_KB` | no uploader; shared `OCLC_KBART_PUBLIC` pull feed (export `kbart::oclc`); explicit no-uploader behaviour |
| `EX_LIBRIS_KB` | no uploader; shared `OCLC_KBART_PUBLIC` pull feed; explicit no-uploader behaviour |
| `JISC_NBK` | explicitly inactive `JISC_NBK_MARC_S3` mechanism; creates no job or delivery |

No mapping field is blank; every value has a route, an explicit no-uploader
behaviour, or an explicitly inactive mechanism. This table is architecture
evidence for later `DIS-01` work; ADR-01 edits nothing in
`thoth-dissemination`.

## 7. Consequences

### Positive

- `BE-02` can implement the exhaustive enum and descriptors without
  guessing any value, behaviour, linkage, readiness or availability.
- Duplicate-safety requirements for the linked pair and the shared KBART
  feed are explicit before any implementation exists.
- Excluded candidates are recorded with reasons, preventing silent aliasing
  (notably EBSCO KB into EBSCOHost, or Serial Solutions into Ex Libris).
- The Thoth-managed source-file invariant is recorded against the observed
  current behaviour, making the future hardening task concrete.

### Negative

- Publishers currently reached through unverified historical routes
  (EBSCO KB, Serial Solutions) have no enum value until a later approved
  decision verifies those routes.
- `JISC_NBK` appears in the inventory before it can be assigned, which
  requires explicit readiness/availability handling in `BE-02`.

### Risks

- The recorded ProQuest defect persists until separately fixed; scheduled
  ProQuest delivery is not fully healthy.
- Temporary operational mirrors (environment publisher lists) can drift
  from Statements of Work until `MIG-01`/`DIS-01` comparison work lands;
  the fail-closed rule bounds the impact.
- Unresolved risks for included values: **none** - every included value's
  required fields are resolved in the evidence matrix.

## 8. Invariants created or confirmed by this decision

1. `DistributionPlatform` has exactly the 17 values of section 4.1 until a
   later approved ADR changes them.
2. No `OTHER`, catch-all or fallback value exists.
3. A destination is distinct from its adapter or feed profile; multiple
   values may share one mechanism without duplicate work.
4. `OAPEN`/`DOAB` remain separate, linked, backend-normalized and
   duplicate-safe.
5. `OCLC_KB` and `EX_LIBRIS_KB` share one duplicate-safe publisher-level
   KBART feed and create no uploader jobs.
6. `JISC_NBK` is included, inactive, non-assignable and job-free until a
   separately approved implementation activates it.
7. Automatic push dissemination may use only Thoth-managed publication
   files (recorded; enforcement is deferred HIGH-risk work).
8. Updates and withdrawals follow the section 4.5 conservative policy.
9. Configuration failure and authority mismatch fail closed; empty
   assignments never broaden processing.

## 9. Implementation impact

Affected tasks:

- `BE-02` (blocked until this ADR is approved and merged): implements the
  enum and descriptor contract of section 5.
- `MIG-01`: uses section 4.1 plus the per-destination
  publisher-configuration sources in the evidence matrix for its approved
  mapping; no-job import mode.
- `DIS-01`: uses section 6 for exhaustive mapping; comparison mode against
  the temporary operational mirrors.
- `EXP-01`: OCLC KBART per-publisher index work follows the `OCLC_KB`
  decision.
- `APP-01`: later consumes backend descriptors; removes the duplicated
  app-side platform labels recorded as evidence.
- Separate future HIGH-risk task: enforce the section 4.6 source-file
  invariant in dissemination source selection.
- Separate future task: implement and approve `JISC_NBK_MARC_S3` before
  `JISC_NBK` becomes assignable.

Required sequencing: this ADR merges (after independent review and CTO
approval) before `BE-02` finalizes the enum. Required migrations: none by
this ADR. Required client changes: none by this ADR. Required operational
changes: none by this ADR.

## 10. Rollout and rollback implications

Rollout: documentation-only merge; nothing activates. Downstream rollout
follows the staged
[rollout plan](../../publisher-services/rollout-plan.md) under each task's
own gates.

Rollback: revert the documentation PR; no runtime effect. If a later
approved decision changes the inventory, it supersedes this ADR explicitly
rather than editing history.

## 11. Validation

Evidence required to prove the decision works:

- the complete
  [evidence matrix](../../publisher-services/adr-01-evidence-matrix.md) with
  every candidate resolved and zero unknown/provisional included fields
  (delivered with this ADR);
- independent exact-head review verifying every cited repository path;
- `BE-02` compile-time exhaustiveness over exactly these 17 values with no
  fallback arm;
- linked-pair and shared-feed duplicate-safety tests in `BE-02`/`BE-04`/
  `DIS-01`;
- `MIG-01` dry-run reconciliation against the temporary operational
  mirrors.

## 12. Approval

Approval required from: CTO, after fresh independent exact-head review of
the ADR-01 implementation PR.

Approved by: NOT APPROVED - PROPOSED
Approval date: -
Notes: This decision record is produced by the ADR-01 implementation task
(draft PR, `feature/publisher-services/adr-01`, base `32123d3`). It becomes
authoritative only after independent review, explicit CTO approval and
merge. `BE-02` remains blocked until then.
