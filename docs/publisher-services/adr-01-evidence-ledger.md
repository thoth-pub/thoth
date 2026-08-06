# ADR-01 Evidence Ledger: EBSCO, ProQuest and Knowledge-Base Distribution

Status: SANITIZED PUBLIC LEDGER; SOURCE RECORD CTO-APPROVED 2026-08-06
Prepared: 6 August 2026
Owner: CTO
Integrated by: ADR-01-SPEC-AMEND-01

## 0. Provenance and sanitization

This file is the sanitized, reviewable version of the CTO-approved source
record:

```text
ADR-01 Evidence Ledger: EBSCO, ProQuest and Knowledge-Base Distribution
Prepared: 6 August 2026
SHA-256: 4395c9b7203cdb5c07f5ad6399879827b1964bf8aeb1edc150bfc4d77221e9d7
```

Scope: evidence for deciding which distribution destinations require
independent configuration values and which consumers share a delivery adapter
or public feed.

Sanitization rules applied:

- evidence IDs, source titles, dates, source owners/editors/senders, stable
  Drive file IDs, Gmail thread/message IDs, repository/commit/path references,
  relevant sections, statuses, exact supported claims, limitations, the
  claim-to-evidence index and the unresolved gaps are preserved exactly;
- no full private document or email body, private publisher list, secret
  value, credential, private environment content or sensitive object URL is
  reproduced;
- no conclusion absent from the approved source ledger has been added.

Drive file IDs and Gmail thread/message IDs are access-controlled stable
identifiers retained for authorized traceability; they expose no content.

### 0.1 Source status vocabulary

Each evidence item carries a **source status**:

- **Current:** the latest operational, contractual or implementation record
  found.
- **Historical:** evidence of an earlier workflow or product relationship. It
  must not, by itself, be treated as proof of current operation.
- **Superseded:** an older record replaced by more recent operational
  evidence.
- **Unresolved:** the available evidence does not establish the current
  position.

### 0.2 Claim classification vocabulary

Separately from the source status, each item's claims carry exactly one
classification from the ADR-01 evidence vocabulary:

- **repository-verified:** directly observable in inspected repository content
  at a recorded commit;
- **source-owner-confirmed:** attributable internal documents, correspondence
  or CTO statements with a recorded date;
- **production-verified:** not used in this ledger; no entry supplies
  authorized production evidence.

Source status and claim classification are independent: a *historical* source
can still be a *repository-verified* or *source-owner-confirmed* record of
what it historically states. Unresolved current status supports exclusion from
the initial enum, never inclusion.

---

## 1. EBSCO evidence

### EBSCO-01 — EBSCO systems are operationally distinct

- **Document title:** `EBSCO.docx`
- **Document owner:** not exposed by the Drive connector. Meeting participants
  and contributors are named in the document.
- **Document date:** meeting held 24 September 2021; Drive file created
  3 December 2021.
- **Storage location:** Google Drive file ID `1XWAkh42mZ-TBDJU_4NaSIx8uHjcDsCgN`
- **Relevant section:** meeting introduction, "Key Points" and "Further
  comments from Tim".
- **Source status:** Historical product and system evidence.
- **Claim classification:** source-owner-confirmed.

**Exact claim supported:** EBSCO Knowledgebase and EBSCOHost are not aliases.
The meeting concerned EBSCO Discovery Service and EBSCO Knowledgebase and
explicitly excluded EBSCOHost and GOBI. Participants described the EBSCO
systems as separate silos with different purposes, requirements and data
practices. The Knowledgebase was described as requiring comparatively basic
coverage information, whereas EBSCOHost could require richer metadata.

**Limit:** this proves product and system separation. It does not prove that
Thoth currently sells or configures both as independently selectable services.

### EBSCO-02 — Current Thoth EBSCOHost workflow

- **Document title:** `EBSCOHost Distribution Workflow`
- **Document owner/editor:** document records `HH`, understood internally as
  Hannah Hillen; only the initials appear in the document.
- **Document date:** last updated 4 October 2024. The current Drive copy was
  created 21 May 2026.
- **Storage location:** Google Drive file ID
  `1sMl7oxv7mbtaelgFnoc96CGN2IVIYuLhPKfCELmDbxE`
- **Relevant section:** semi-manual and manual workflows.
- **Source status:** Current operational reference; latest EBSCOHost workflow
  found.
- **Claim classification:** source-owner-confirmed.

**Exact claim supported:** Thoth has an EBSCOHost-specific route. The workflow
selects `EBSCOHost` as the platform and sends ONIX 2.1 metadata, book files
and a cover through SFTP. The document does not present this as an EBSCO
Knowledge Base workflow.

**Limit:** the source establishes EBSCOHost configuration only.

### EBSCO-03 — Current EBSCOHost implementation

- **Document title:** `ebscouploader.py`
- **Owner:** `thoth-pub/thoth-dissemination` repository.
- **Document date:** immutable repository snapshot at release commit
  `7a16edc08d4570f3ecc108453298a3aa43f6d753`, associated with release 1.6.4
  dated 28 July 2026.
- **Storage location:** repository `thoth-pub/thoth-dissemination`; path
  `ebscouploader.py`; commit SHA above.
- **Relevant section:** `EBSCOUploader.upload_to_platform`.
- **Source status:** Current implementation snapshot.
- **Claim classification:** repository-verified.

**Exact claim supported:** the uploader is explicitly for EBSCOHost. It
requires PDF or EPUB content, a cover and the `onix_2.1::ebsco_host` export,
and uploads them to the EBSCOHost SFTP service.

**Limit:** the absence of an EBSCO KB uploader does not prove that no manual
or pull-feed arrangement exists. It proves only that the current push adapter
is Host-specific.

### EBSCO-04 — Historical separate EBSCO KB route

- **Document title:** `230807-obp-platform-submission-workflows.md`
- **Document owner:** not stated. Meeting attendees are Ross Higman, Laura and
  Toby Steiner.
- **Document date:** workflow discussion dated 7 August 2023; stored copy
  created 2 November 2023.
- **Storage location:** Google Drive file ID `1lvUnDfipqJMD421DHBhbcU7WaBpSG33z`
- **Relevant section:** `FTP upload`.
- **Source status:** Historical OBP-derived workflow evidence.
- **Claim classification:** source-owner-confirmed.

**Exact claim supported:** the source lists `EBSCO KB` and `EBSCO Host` as two
separate FTP destinations. It notes that EBSCO KB did not require a book file,
while Host was listed separately.

**Limit:** this describes an OBP workflow considered during early Thoth Plus
planning. It is not proof that a separate EBSCO KB route remains active or
selectable in 2026.

### EBSCO-05 — Current commercial/service-selection evidence

- **Document title:** `Main Thoth Service Level Agreement (SLA) (template)`
- **Document owner:** Thoth Open Metadata; no individual author is stated.
- **Document date:** created 18 September 2025; updated 21 September 2025.
- **Storage location:** Google Drive document ID
  `1SwWSj78nqfHjbWG4uFNkgDqApmbV1oYJ-Fk-9b1GZwA`
- **Relevant section:** section 2.1.1, recurring dissemination tasks;
  Appendix A.
- **Source status:** Current contractual template.
- **Claim classification:** source-owner-confirmed.

**Exact claim supported:** the selectable service list contains `EBSCOHost`.
It does not contain a separate `EBSCO Knowledge Base` item.

**Limit:** absence from the template is evidence that EBSCO KB is not
currently represented as an independent standard SLA selection. It is not
conclusive proof that no bespoke or legacy publisher arrangement exists.

---

## 2. ProQuest evidence

### PROQUEST-01 — Current Ebook Central and Ex Libris workflows are different

- **Document title:** `ProQuest Distribution Workflows`
- **Document owner/editor:** document records `HH`, understood internally as
  Hannah Hillen; only the initials appear in the document.
- **Document date:** last updated 15 May 2025. Current Drive copy created
  28 July 2026.
- **Storage location:** Google Drive document ID
  `1ge611Q4uD9BSqj572Bm9Bt5BDXuadTdSqIYQr1OJh34`
- **Relevant sections:** `Proquest Ebook Central` and
  `Proquest ExLibris & Knowledge Bases (for Alma/Community Zone ingest)`.
- **Source status:** Current operational reference; latest consolidated
  ProQuest workflow found.
- **Claim classification:** source-owner-confirmed.

**Exact claims supported:**

1. **ProQuest Ebook Central** is a book-hosting destination receiving ONIX
   metadata and content through an SFTP delivery.
2. **ProQuest Ex Libris** is a knowledge-base ingestion arrangement for Alma,
   SFX and 360.
3. New Ex Libris publisher collections require notification and onboarding.
4. Ex Libris is supplied with a publisher-level Thoth OCLC KBART URL and
   harvests it monthly.
5. Ex Libris has separate collection identifiers for individual publisher
   collections.

**Limit:** the document groups Ex Libris under the ProQuest family, but the
operational mechanism is separate from Ebook Central. It does not document a
current Serial Solutions/ProQuest KB workflow.

### PROQUEST-02 — Generic `ProQuest` in current code means Ebook Central

- **Document title:** `proquestuploader.py`
- **Owner:** `thoth-pub/thoth-dissemination` repository.
- **Document date:** immutable repository snapshot at commit
  `7a16edc08d4570f3ecc108453298a3aa43f6d753`, release 1.6.4 dated
  28 July 2026.
- **Storage location:** repository `thoth-pub/thoth-dissemination`; path
  `proquestuploader.py`; commit SHA above.
- **Relevant section:** `ProquestUploader.upload_to_platform`.
- **Source status:** Current implementation snapshot.
- **Claim classification:** repository-verified.

**Exact claims supported:**

1. The class and its documentation identify the destination as **ProQuest
   Ebook Central**.
2. It is an SFTP content-and-metadata uploader.
3. Its metadata export flavour remains named `onix_2.1::proquest_ebrary`.

**Interpretation supported by the evidence:** in implementation terminology,
generic `ProQuest` is Ebook Central. `Ebrary` is retained as a legacy
export-flavour name and is not implemented as a second destination.

### PROQUEST-03 — Current uploader registry

- **Document title:** `disseminator.py`
- **Owner:** `thoth-pub/thoth-dissemination` repository.
- **Document date:** immutable repository snapshot at commit
  `7a16edc08d4570f3ecc108453298a3aa43f6d753`, release 1.6.4 dated
  28 July 2026.
- **Storage location:** repository `thoth-pub/thoth-dissemination`; path
  `disseminator.py`; commit SHA above.
- **Relevant section:** `UPLOADERS` registry.
- **Source status:** Current implementation snapshot.
- **Claim classification:** repository-verified.

**Exact claim supported:** the push-uploader registry contains `EBSCOHost` and
`ProQuest`. The `ProQuest` entry maps to `ProquestUploader`, which the
implementation identifies as Ebook Central. There are no separate push-registry
values for ProQuest Ex Libris, ProQuest KB/Serial Solutions or EBSCO KB.

**Limit:** pull-feed consumers should not necessarily appear in a push-uploader
registry. Their absence here is not evidence that they do not exist; it
establishes only how the existing CLI names the push destinations.

### PROQUEST-04 — Historical naming and product relationships

- **Document title:** `230807-obp-platform-submission-workflows.md`
- **Document owner:** not stated; meeting attendees are Ross Higman, Laura and
  Toby Steiner.
- **Document date:** workflow discussion dated 7 August 2023; stored copy
  created 2 November 2023.
- **Storage location:** Google Drive file ID `1lvUnDfipqJMD421DHBhbcU7WaBpSG33z`
- **Relevant sections:** `FTP upload` and
  `Automatic retrieval from Thoth by distributor`.
- **Source status:** Historical.
- **Claim classification:** source-owner-confirmed.

**Exact claims supported:**

1. `ProQuest [Ebook Central]` was formerly called `Ebrary`.
2. `ProQuest - was ExLibris` was listed as an automatic-retrieval destination.
3. `ProQuest KB - was Serial Solutions` was listed separately from Ex Libris.
4. Ebook Central, Ex Libris and Serial Solutions were therefore treated as
   different historical workflows, not three names for a single destination.

**Limit:** the record does not establish that the old Serial Solutions route
remains active, sold or configurable today.

### PROQUEST-05 — Separate historical workflow documents existed

- **Document title:** `Thoth Plus Distribution Workflows Notes.docx`
- **Document owner:** not exposed by the Drive connector.
- **Document date:** 4 December 2023.
- **Storage location:** Google Drive file ID `16lBBJNJGi8iD72Kjb3U_DuXpuASG-F0D`
- **Relevant section:** inventory of redacted workflow documents transferred
  to the Thoth Nextcloud `Distribution Workflows` folder.
- **Source status:** Historical planning record.
- **Claim classification:** source-owner-confirmed.

**Exact claim supported:** the inventory names separate instructions for:

- `ProQuest Ebrary`
- `ProQuest ExLibris`
- `ProQuest KB Serial Solutions`

It explicitly describes Ebrary as "aka Ebook Central". It also warns that OBP
workflows were only a starting point and that Thoth would still need
confirmation from each platform before using those arrangements for additional
publishers.

### PROQUEST-06 — Current commercial/service-selection evidence

- **Document title:** `Main Thoth Service Level Agreement (SLA) (template)`
- **Document owner:** Thoth Open Metadata; no individual author stated.
- **Document date:** created 18 September 2025; updated 21 September 2025.
- **Storage location:** Google Drive document ID
  `1SwWSj78nqfHjbWG4uFNkgDqApmbV1oYJ-Fk-9b1GZwA`
- **Relevant section:** section 2.1.1 and Appendix A.
- **Source status:** Current contractual template.
- **Claim classification:** source-owner-confirmed.

**Exact claim supported:** the current standard selection is labelled only
`ProQuest`. The SLA does not expose separate selectable entries for Ebook
Central, Ex Libris or ProQuest KB/Serial Solutions.

**Limit:** because the current uploader maps `ProQuest` to Ebook Central, the
SLA label is likely intended to mean Ebook Central, but the SLA itself does
not resolve that ambiguity. It should not be used as evidence that Ex Libris
or Serial Solutions are included automatically.

---

## 3. OCLC and Ex Libris KBART evidence

### KBART-01 — Current OCLC harvesting arrangement

- **Document title:** `OCLC KB - Distribution Workflow`
- **Document owner:** not exposed by the Drive connector.
- **Document date:** last updated 13 May 2025. Current Drive copy created
  20 May 2026.
- **Storage location:** Google Drive document ID
  `1jE-BOod0hB6PuWdDO0H2zgzHLbJ_x7eX0kP3cyV2cFk`
- **Relevant section:** method, metadata and publisher collection codes.
- **Source status:** Current operational reference.
- **Claim classification:** source-owner-confirmed.

**Exact claims supported:**

1. OCLC automatically harvests metadata from the Thoth Export API.
2. The metadata format is OCLC KBART.
3. The harvest is scheduled monthly.
4. New publishers must be communicated to OCLC for inclusion.
5. Individual Thoth Plus publishers receive distinct OCLC collection
   identifiers.

**Implication supported:** OCLC is a separately onboarded consumer, even
though the underlying output is generated through a common Thoth OCLC KBART
export profile.

### KBART-02 — Ex Libris harvests the same OCLC KBART feed family

- **Email subject:** `OCLC KBART > ProQuest ExLibris`
- **Senders:** Hannah Hillen, Ross Higman and Javier Arias.
- **Email dates:** 8-11 July 2025.
- **Storage location:** Gmail thread ID `197f9116017944ee`
- **Relevant messages:**
  - Hannah Hillen, 8 July 2025 — message ID `197eabe087114332`
  - Hannah Hillen, 10 July 2025 — message ID `197f3d31eeb2bd5a`
  - Javier Arias, 11 July 2025 — message ID `197f9116017944ee`
- **Source status:** Current operational correspondence.
- **Claim classification:** source-owner-confirmed.

**Exact claims supported:**

1. ProQuest Ex Libris was harvesting OCLC KBART metadata for a subset of
   Thoth Plus publishers.
2. OCLC KB was also harvesting that metadata feed.
3. The harvested resources were the publisher-level Thoth Export API outputs.
4. The publisher-level URLs were intended to remain persistent.
5. The subset of Ex Libris publishers is an onboarding/configuration choice,
   not a different export format.

**Evidence limitation:** this is internal correspondence reporting an
Ex Libris request and confirming Thoth's implementation. It is not a direct
contract or public Ex Libris product statement. It is nevertheless direct
evidence of the actual Thoth arrangement.

### KBART-03 — The shared-feed relationship in the operational workflow

- **Document title:** `ProQuest Distribution Workflows`
- **Document owner/editor:** `HH`, understood internally as Hannah Hillen.
- **Document date:** last updated 15 May 2025.
- **Storage location:** Google Drive document ID
  `1ge611Q4uD9BSqj572Bm9Bt5BDXuadTdSqIYQr1OJh34`
- **Relevant section:**
  `Proquest ExLibris & Knowledge Bases (for Alma/Community Zone ingest)`.
- **Source status:** Current operational reference.
- **Claim classification:** source-owner-confirmed.

**Exact claim supported:** Ex Libris is supplied with the Thoth URL for the
**OCLC KBART** export and harvests it monthly. The workflow does not define an
Ex Libris-specific KBART export flavour.

**Implementation consequence supported:** `OCLC_KB` and `EX_LIBRIS_KB` may be
separate publisher/consumer assignments while mapping to the same OCLC KBART
feed adapter. Enabling both must not generate two equivalent files.

---

## 4. Jisc evidence

### JISC-01 — Current Jisc destination is NBK using MARC/S3

- **Email subject:** `Thoth Data transfer to NBK - S3 bucket`, subsequently
  carried under Jisc support case references.
- **Senders:** Javier Arias, Toby Steiner and the Jisc NBK contributor-support
  team, including Bethan Ruddock.
- **Email dates:** 21 May 2025 to 25 March 2026.
- **Storage location:** Gmail thread ID `19d253816d7b988d`
- **Relevant messages:**
  - Jisc NBK, 21 May 2025 — confirmation of the S3 transfer route.
  - Javier Arias, 5 September 2025 — proposed record structure and update
    workflow.
  - Jisc NBK, 25 March 2026 — request for a full export to be uploaded for
    loading after Jisc's system migration.
- **Source status:** Current agreed workflow. Production loading was delayed
  by a Jisc system migration as of 25 March 2026; no newer completion
  confirmation was found.
- **Claim classification:** source-owner-confirmed.

**Exact claims supported:**

1. The destination is the Jisc National Bibliographic Knowledgebase, or NBK.
2. Thoth transfers records to an S3 bucket rather than exposing a KBART URL
   for Jisc to harvest.
3. The proposed unit is one MARC21 `.mrc` file per book.
4. The workflow covers initial back-catalogue delivery, new books and
   overwriting changed records.
5. Jisc asked for a full export to be added to its loading queue.

The email search record independently identifies the NBK/S3 thread.

**Implementation consequence supported:** current `JISC_NBK` must not be
modelled as another consumer of the public OCLC KBART feed. Its delivery
mechanism is a separate MARC/S3 adapter.

### JISC-02 — Current contractual naming

- **Document title:** `Main Thoth Service Level Agreement (SLA) (template)`
- **Document owner:** Thoth Open Metadata.
- **Document date:** September 2025.
- **Storage location:** Google Drive document ID
  `1SwWSj78nqfHjbWG4uFNkgDqApmbV1oYJ-Fk-9b1GZwA`
- **Relevant section:** section 2.1.1 and Appendix A.
- **Source status:** Current contractual template.
- **Claim classification:** source-owner-confirmed.

**Exact claim supported:** the current standard destination name is
`Jisc NBK`, not the older `Jisc KB` label.

---

## 5. Historical knowledge-base inventory

### HIST-01 — Older OBP knowledge-base terminology

- **Document title:** `OBP-Distribution-guidelines.docx`
- **Document owner:** Open Book Publishers; individual author not exposed by
  the connector.
- **Document date:** stored copy created 15 January 2020.
- **Storage location:** Google Drive file ID `1CXSTSKMRy2b6G-AwqkPKFWU07gGiIzz0`
- **Relevant section:** metadata aggregators and knowledge-base distribution.
- **Source status:** Superseded, OBP-specific documentation.
- **Claim classification:** source-owner-confirmed.

**Exact historical claims supported:** the document separately names JISC
Knowledge Base, OCLC Knowledge Base, ProQuest/Ex Libris, ProQuest KB/Serial
Solutions and EBSCO Knowledge Base.

**Limit:** this document must not define the 2026 Thoth architecture. In
particular, the current Jisc NBK/S3 arrangement supersedes its older Jisc
KBART/email description.

---

## 6. ADR architecture evidence

### ADR-01-SOURCE — Required distinction between destination and adapter

- **Document title:** `Publisher Services and Distribution Configuration -
  Technical Design and Implementation Plan`
- **Document owner:** Thoth engineering/product design; no named individual
  author appears in the document.
- **Document date:** created and updated 23 July 2026.
- **Storage location:** Google Drive document ID
  `1kr2Ft0Y4pxgcXGyFAKs_wfFx4I0jlxEvaceswE5Dus8`
- **Relevant sections:** 3.2 Distribution platforms; 4.3 Platform descriptors
  and adapters; 5.2 Distribution platform enum; 8.2 ADR-01.
- **Source status:** Current, approved for phased implementation.
- **Claim classification:** source-owner-confirmed.

**Exact architectural requirements supported:**

1. Each independently meaningful destination receives its own enum value.
2. Multiple platform values may map to one delivery adapter.
3. Shared adapters must not cause duplicate deliveries.
4. The initial platform list is provisional.
5. ADR-01 must resolve whether EBSCO KB and multiple ProQuest destinations
   require additional independently selectable values.
6. No implementation may rely on an unverified provisional inventory.

---

## 7. Claim-to-evidence index

### Question 1: EBSCOHost versus EBSCO Knowledge Base

- **They are different EBSCO systems, not aliases:** EBSCO-01.
- **EBSCOHost has a current Thoth-specific workflow and uploader:** EBSCO-02
  and EBSCO-03.
- **EBSCO KB had a historically separate metadata-only route:** EBSCO-04.
- **A current independently selectable EBSCO KB service is not established:**
  EBSCO-05, the current uploader registry and the absence of a current KB
  workflow.
- **ADR status:** keep EBSCO KB unresolved rather than silently aliasing it to
  EBSCOHost.

### Question 2: ProQuest, Ebrary, ProQuest KB and Ex Libris

- **Generic `ProQuest` in current Thoth code means Ebook Central:**
  PROQUEST-01, PROQUEST-02 and PROQUEST-03.
- **Ebrary is the former name/legacy export label for Ebook Central:**
  PROQUEST-02, PROQUEST-04 and PROQUEST-05.
- **Ex Libris is a separately onboarded KB consumer with a separate
  mechanism:** PROQUEST-01 and KBART-02/03.
- **ProQuest KB was historically the Serial Solutions route and was separate
  from Ex Libris:** PROQUEST-04 and PROQUEST-05.
- **Current active or separately sold Serial Solutions/ProQuest KB status is
  not established:** PROQUEST-01 and PROQUEST-06.
- **Current SLA terminology remains ambiguous:** PROQUEST-06.

### Question 3: Five proposed KBART consumers

- **OCLC KB:** confirmed monthly OCLC KBART pull with per-publisher
  onboarding — KBART-01.
- **ProQuest Ex Libris:** confirmed monthly harvest of the same
  publisher-level OCLC KBART output for an independently onboarded subset —
  KBART-02 and KBART-03.
- **Jisc:** current evidence establishes a MARC21/S3 push to Jisc NBK, not a
  KBART pull — JISC-01 and JISC-02.
- **EBSCO KB:** historically separate, but its current delivery mechanism and
  selectable status are unresolved — EBSCO-01, EBSCO-04 and EBSCO-05.
- **ProQuest KB/Serial Solutions:** historically separate, but its current
  delivery mechanism and selectable status are unresolved — PROQUEST-04
  through PROQUEST-06.

---

## 8. Evidence gaps ADR-01 must preserve as unresolved

1. No current EBSCO Knowledge Base workflow, endpoint, active publisher list
   or current SLA selection was found.
2. No current ProQuest KB/Serial Solutions workflow, endpoint, active
   publisher list or current SLA selection was found.
3. The generic `ProQuest` SLA label does not state whether it means only
   Ebook Central or a broader ProQuest service family.
4. The evidence establishes operational configurations, but not how Clarivate
   commercially sells or bundles every product.
5. The current Jisc transfer mechanism is documented, but completion of the
   initial production load after Jisc's 2026 system migration was not
   confirmed.
6. Ex Libris and OCLC are independently onboarded consumers, but they use one
   shared Thoth OCLC KBART output profile. They should not create duplicate
   feed-generation jobs.

No unsupported claim should be converted into an enum value or alias until one
of the unresolved routes is confirmed by a current workflow, agreement,
endpoint or direct vendor correspondence.

---

## 9. CTO decisions of 2026-08-06 (source-owner-confirmed)

Javi, CTO, explicitly confirmed on 2026-08-06, as attributable
source-owner-confirmed input for the ADR-01 specification amendment:

1. `EX_LIBRIS_KB` is a separate destination because it requires separate
   per-publisher onboarding and collection configuration, even when
   commercially bundled. It maps to the shared OCLC KBART feed and must
   create no duplicate feed or uploader job.
2. `JISC_NBK` is a separate destination but is initially inactive and
   non-assignable until its MARC/S3 adapter, onboarding controls and
   operational evidence are separately implemented and approved.
3. There are no known current manual-only distribution destinations outside
   the inspected repositories and internal operational documentation.
4. The conservative initial update and withdrawal policy recorded in the
   amended ADR-01 specification is correct.
5. The Project MUSE specification correction is required.
6. The evidence ledger recorded above is approved as the source record for
   drafting the amendment.

These decisions are drafting authority for ADR-01-SPEC-AMEND-01. They are not
approval of the amended wording, and statement 3 is a confirmation of current
knowledge, not a claim that no manual destination can ever exist.
