# ADR-01 Evidence Matrix

Status: COMPLETE EVIDENCE RECORD FOR ADR-0004 - DECISION PROPOSED, NOT APPROVED
Task: ADR-01 - Platform inventory and final architecture
Prepared: 2026-08-06
Authorized base (`thoth-pub/thoth`): `32123d363a6806d377ac322e3814fb432a803453`
Decision record: [ADR-0004](../engineering/decisions/ADR-0004-distribution-platform-inventory.md)
Source ledger (unchanged): [adr-01-evidence-ledger.md](adr-01-evidence-ledger.md)

## 1. Purpose and provenance separation

This matrix records the complete per-candidate evidence for the final
`DistributionPlatform` inventory proposed by ADR-0004. It separates:

1. **evidence** - what was directly observed or attributably confirmed, with
   exact commits, paths and attributions;
2. **final decision** - the disposition ADR-0004 proposes from that evidence.

It also preserves the provenance boundary of the
[ADR-01 evidence ledger](adr-01-evidence-ledger.md):

- ledger sections 1 through 8 reproduce the hash-identified source record
  (SHA-256
  `4395c9b7203cdb5c07f5ad6399879827b1964bf8aeb1edc150bfc4d77221e9d7`,
  original 18 evidence entries: `EBSCO-01..05`, `PROQUEST-01..06`,
  `KBART-01..03`, `JISC-01..02`, `HIST-01`, `ADR-01-SOURCE`);
- ledger section 9 records the CTO decisions of 2026-08-06 as a separately
  added attributable `source-owner-confirmed` record **not** authenticated by
  that SHA-256.

Citations below of the form `LEDGER:<ID>` refer to the original 18 entries;
citations of the form `CTO-2026-08-06` refer to the separately attributable
CTO decisions (ledger section 9, the merged ADR-01 specification section 8,
and the ADR-01 implementation authorization of 2026-08-06). The two provenance
sources are never conflated.

## 2. Repository evidence basis

All repository inspection was read-only file inspection at the recorded
immutable commits. No workflow was dispatched, no uploader was run, no
credential was used, and no production or shared resource was accessed.

| Repository | Ref inspected | Exact commit |
|---|---|---|
| `thoth-pub/thoth` | authorized ADR-01 base (`develop`) | `32123d363a6806d377ac322e3814fb432a803453` |
| `thoth-pub/thoth` | deferred `feature/oai-pmh-http` (context only, unmodified) | `745dd020661e8a8b94d0752e11f10a9d583bd769` |
| `thoth-pub/thoth-dissemination` | default branch `main` head (= tag `v1.6.4`, release 1.6.4, 28 July 2026) | `7a16edc08d4570f3ecc108453298a3aa43f6d753` |
| `thoth-pub/thoth-app` | `main` head | `6f826390a07efe6266cfda2b4af1f85b6cbfc38a` |
| `thoth-pub/thoth-app` | `dev` head | `26323158f1145b35eff27bce6f901ff0eb78280a` |

### 2.1 Drift record

- The provisional inventory baseline was taken at `thoth-dissemination`
  `5e88ce1b58e5f962cc4f4ef6fb00c08f50b57add`. The current default-branch head
  `7a16edc0` is **identical to the release commit already cited by the
  evidence ledger** (release 1.6.4). The drift
  `5e88ce1b..7a16edc0` consists of releases v1.6.1 through v1.6.4: Internet
  Archive idempotency, propagation-verification, deterministic JSON sidecar
  and convergence work (PRs #91 through #94). It changes no uploader registry
  key, no scheduled workflow, and no platform inventory fact; it strengthens
  the Internet Archive retry/idempotency evidence recorded below. Harmless,
  recorded drift.
- `thoth-app` `main` and `dev` are identical for every inspected
  platform-relevant path (`src/shared/utils/locations/index.ts`,
  `src/shared/constants/formFields.ts`, `codegen.ts`; empty diff verified).

### 2.2 Key repository observations (thoth at `32123d3`)

- `thoth-api/src/model/location/mod.rs` - `LocationPlatform` enum with 18
  values: `Project MUSE`, `OAPEN`, `DOAB`, `JSTOR`, `EBSCO Host`, `OCLC KB`,
  `ProQuest KB`, `ProQuest ExLibris`, `EBSCO KB`, `JISC KB`, `Google Books`,
  `Internet Archive`, `ScienceOpen`, `SciELO Books`, `Zenodo`,
  `Publisher Website`, `Thoth`, `Other` (default). These are **location**
  values; ADR-0004 deletes none of them.
- `thoth-export-server/src/data.rs` - export registry: 18 registry platforms
  (`thoth`, `project_muse`, `oapen`, `doab`, `jstor`, `google_books`,
  `overdrive`, `bds_live`, `ebsco_host`, `oclc_kb`, `proquest_kb`,
  `proquest_exlibris`, `ebsco_kb`, `jisc_kb`, `zotero`, `crossref`,
  `rnib_bookshare`, `proquest_ebrary`) and 17 specifications. One KBART
  specification `kbart::oclc` is accepted by `oclc_kb`, `proquest_kb`,
  `proquest_exlibris`, `ebsco_kb` and `jisc_kb`; `onix_2.1::ebsco_host` is
  accepted by `ebsco_host` and `rnib_bookshare`; `onix_3.0::oapen` is
  accepted by `oapen` and `doab`; `onix_3.0::jstor` is accepted by `jstor`
  and `bds_live`. Registry acceptance records which specification a consumer
  takes; it is **not** evidence of current consumption or onboarding.
- `thoth-export-server/src/csv/kbart_oclc.rs` - the OCLC KBART output,
  including the `oclc_number` column fed from the work-level `oclc` field
  (`thoth-api/src/model/work/mod.rs:207`).
- `thoth-api/src/model/file/` and `thoth-api/src/model/location/` - the
  Thoth-managed file-record and location concepts referenced by the
  Thoth-managed source-file invariant (ADR-0004 section 4.6).
- `.github/workflows/` - `build_test_and_check.yml`, `check_changelog.yml`,
  `run_migrations.yml`, `docker_build_and_push_to_dockerhub.yml` (+ release
  variant). Documentation-only changes classify heavy jobs as skipped.
- Deferred branch `feature/oai-pmh-http` at `745dd020` adds a
  `thoth-oai-server` crate (OAI-PMH HTTP service, `oai_dc`/`openaire`
  formats). Inventory context only: OAI-PMH is a deferred metadata-harvest
  protocol surface, not a distribution destination; the branch was not
  modified, rebased, merged or pushed.

### 2.3 Key repository observations (thoth-dissemination at `7a16edc0`)

- `disseminator.py` `UPLOADERS` registry - exactly 13 push dispatch keys:
  `InternetArchive`, `OAPEN`, `ScienceOpen`, `CUL`, `Crossref`, `Figshare`,
  `Zenodo`, `ProjectMUSE`, `JSTOR`, `EBSCOHost`, `ProQuest`, `GooglePlay`,
  `BKCI`. No key exists for EBSCO KB, any ProQuest KB/Serial Solutions or
  Ex Libris route, Jisc, OCLC, DOAB, OverDrive, BDS Live, RNIB Bookshare,
  SciELO Books or Zotero.
- Scheduled workflows (all pass a registry key): `cr_bulk_disseminate.yml`
  (`Crossref`, hourly), `ia_bulk_disseminate.yml` (`InternetArchive`, daily),
  `gp_bulk_disseminate.yaml` (`GooglePlay`, daily), `oapen_bulk_disseminate.yaml`
  (`OAPEN`, weekly Mon), `eh_bulk_disseminate.yaml` (`EBSCOHost`, weekly Tue),
  `jstor_bulk_disseminate.yaml` (`JSTOR`, weekly Wed),
  `muse_bulk_disseminate.yaml` (`ProjectMUSE`, weekly Thu),
  `pq_bulk_disseminate.yaml` (`ProQuest`, weekly Fri),
  `bkci_bulk_disseminate.yaml` (`BKCI`, monthly day 6),
  `cul_bulk_disseminate.yml` (`CUL`), `fs_bulk_disseminate.yml` (`Figshare`),
  `zn_bulk_disseminate.yml` (`Zenodo`) (monthly day 7); plus
  `muse_catchup_locations.yaml`, `oapen_catchup_locations.yaml`,
  `ia_reconcile.yml` maintenance schedules, and manual entry points
  `disseminate.yml` / `manual_disseminate.yml`. `ScienceOpen` has **no**
  scheduled workflow.
- Configuration structure (structure only; no value read; exact secret and
  credential identifiers intentionally omitted): publisher-assignment mirrors
  are per-platform GitHub repository variables `<PREFIX>_ENV_PUBLISHERS` /
  `<PREFIX>_ENV_EXCEPTIONS` (`IA`, `OAPEN`, `EH`, `JSTOR`, `MUSE`, `PQ`,
  `GP`, `CR`, `CUL`, `FS`, `ZN`, `BKCI`), passed to the shared
  `bulk_disseminate.yml`. These are publisher-list configuration, not
  credential retrieval. Separately, `config.env.template` establishes the
  credential configuration categories and their scope: global credentials
  for Internet Archive, ScienceOpen, OAPEN, CUL, JSTOR, EBSCOHost, ProQuest,
  Figshare and Zenodo; per-publisher credential entries for Project MUSE,
  BKCI and Crossref; a per-publisher folder-name configuration for JSTOR; a
  per-publisher collection-code configuration for Google Play; a cloud
  service-account/bucket configuration for Google Play; a Thoth
  personal-access-token entry for location writeback; and per-platform
  notification-address entries. Credential configuration exists in the
  dissemination repository; this matrix records category, scope and
  responsibility only.
- Source-file selection: `uploader.py` `get_publication_source` selects the
  **canonical location `fullTextUrl` of each publication regardless of the
  hosting platform**. Current behaviour therefore does not yet enforce the
  Thoth-managed source-file invariant; ADR-0004 records the invariant as
  architecture with a separate future HIGH-risk implementation task.
- Duplicate/idempotency behaviour: `iauploader.py` inspects item existence
  and converges owned items (v1.6.1-v1.6.4 hardening);
  `fsuploader.py` and `zenodouploader.py` fail when a record for the work
  already exists; `crossrefuploader.py` posts DOI deposit files to the
  Crossref deposit endpoint (redeposit updates the DOI record); FTP/SFTP
  uploaders re-upload files on re-run.
- Location writeback: Internet Archive immediate (`write_locations.py`,
  `reconcile_internet_archive.py`); Figshare and Zenodo immediate
  per-publication; CUL immediate but recorded under `LocationPlatform`
  `'OTHER'` (`culuploader.py:50`); OAPEN and Project MUSE deferred catch-up
  (`obtain_oapen_locations.py`, `obtain_muse_locations.py`); no writeback for
  JSTOR, EBSCOHost, ProQuest, GooglePlay, BKCI, Crossref, ScienceOpen.
- Work selection for bulk runs (`obtain_new_ids.py`): active (published)
  works of types `MONOGRAPH, EDITED_BOOK, JOURNAL_ISSUE, TEXTBOOK, BOOK_SET`
  from the configured publishers, within the lookback window.
- Generic mechanism: `swordv2uploader.py` -> `dspaceuploader.py` is a shared
  SWORDv2/DSpace implementation base used by `oapensworduploader.py` and
  `culuploader.py`. It is an implementation mechanism, not a destination and
  not a user-visible uploader key.
- Project MUSE historical defect: `muse_bulk_disseminate.yaml` passes
  `platform: 'ProjectMUSE'`, which the registry accepts; the historical
  `platform: 'MUSE'` mismatch was fixed by commit
  `1a66da8f1700d8c76bf8fda2938b8729be0a93b6` (23 April 2026), an ancestor of
  both `5e88ce1b` and `7a16edc0`. Historical/resolved; not current.
- ProQuest current defect: `proquestuploader.py` sets
  `filename = self.get_isbn('PDF')` (line 43) **before** the PDF/EPUB
  fallback (lines 63-73), so the intended EPUB-only fallback can fail when no
  PDF publication exists. Current defect; recorded, not fixed.
- Tests: `tests/` covers the Internet Archive uploader/workflows,
  reconciliation, `obtain_new_ids`, `obtain_oapen_locations`, `thothapi`,
  `uploader` and `write_locations`. Release/Docker workflows:
  `docker_build_and_push_to_dockerhub.yml`, `tests.yml`.

### 2.4 Key repository observations (thoth-app at `6f826390` / `26323158`)

- `src/shared/utils/locations/index.ts` and
  `src/shared/constants/formFields.ts` hard-code `LocationPlatform` display
  labels and option lists (including `EBSCO_KB`, `JISC_KB`, `OCLC_KB`,
  `PROQUEST_KB`, `PROQUEST_EXLIBRIS`, `GOOGLE_BOOKS`). This is **duplication
  evidence** to be replaced by backend-provided descriptors under later
  `APP-01` work; it is not authority for the inventory.
- `codegen.ts` generates from the unpinned shared test API
  (`https://api.test.thoth.pub/graphql`), confirming the reserved BE-03/APP-01
  contract control's concern.
- Branch topology: `main` and `dev`; CI is `test.yml` only (consistent with
  open CG-11); publisher administration lives under `src/features/publisher`.
- No app change is made or required by ADR-01.

## 3. Candidate coverage index

Every provisional candidate and every discovered candidate has exactly one
final disposition. 27 candidates: 17 included (16 assignable + 1 inactive),
10 excluded.

| # | Candidate | Origin | Disposition |
|---|---|---|---|
| 1 | `INTERNET_ARCHIVE` | provisional list | INCLUDED - AutomaticPush, assignable |
| 2 | `OAPEN` | provisional list | INCLUDED - AutomaticPush, assignable, linked `OAPEN_DOAB` |
| 3 | `DOAB` | provisional list | INCLUDED - AutomaticPush (via linked deposit), assignable, linked `OAPEN_DOAB` |
| 4 | `SCIENCE_OPEN` | provisional list | INCLUDED - Manual, assignable |
| 5 | `CAMBRIDGE_UNIVERSITY_LIBRARY` | provisional list | INCLUDED - AutomaticPush, assignable |
| 6 | `CROSSREF` | provisional list | INCLUDED - AutomaticPush, assignable |
| 7 | `FIGSHARE` | provisional list | INCLUDED - AutomaticPush, assignable |
| 8 | `ZENODO` | provisional list | INCLUDED - AutomaticPush, assignable |
| 9 | `PROJECT_MUSE` | provisional list | INCLUDED - AutomaticPush, assignable |
| 10 | `JSTOR` | provisional list | INCLUDED - AutomaticPush, assignable |
| 11 | `EBSCO_HOST` | provisional list | INCLUDED - AutomaticPush, assignable |
| 12 | `PROQUEST_EBOOK_CENTRAL` | provisional list | INCLUDED - AutomaticPush, assignable (current defect recorded) |
| 13 | `GOOGLE_PLAY` | provisional list | INCLUDED - AutomaticPush, assignable |
| 14 | `BKCI` | provisional list | INCLUDED - AutomaticPush, assignable |
| 15 | `OCLC_KB` | provisional list | INCLUDED - PullFeed, assignable, shared `OCLC_KBART_PUBLIC` |
| 16 | `EX_LIBRIS_KB` | provisional list | INCLUDED - PullFeed, assignable, shared `OCLC_KBART_PUBLIC` |
| 17 | `JISC_NBK` | provisional list | INCLUDED - inactive, non-assignable, job-free |
| 18 | `EBSCO_KB` | discovered (LocationPlatform, export registry, app, ledger) | EXCLUDED |
| 19 | `PROQUEST_SERIALS_SOLUTIONS_KB` | discovered (LocationPlatform `ProQuest KB`, export registry `proquest_kb`, ledger) | EXCLUDED |
| 20 | `OVERDRIVE` | discovered (export registry `overdrive`) | EXCLUDED |
| 21 | `BDS_LIVE` | discovered (export registry `bds_live`) | EXCLUDED |
| 22 | `RNIB_BOOKSHARE` | discovered (export registry `rnib_bookshare`) | EXCLUDED |
| 23 | `SCIELO_BOOKS` | discovered (LocationPlatform `SciELO Books`) | EXCLUDED |
| 24 | `ZOTERO` | discovered (export registry `zotero`) | EXCLUDED |
| 25 | `THOTH` | discovered (LocationPlatform/export registry `thoth`) | EXCLUDED - location/hosting concept |
| 26 | `PUBLISHER_WEBSITE` | discovered (LocationPlatform) | EXCLUDED - location concept |
| 27 | `OTHER` | discovered (LocationPlatform default) | EXCLUDED - prohibited by architecture |

Alias-only discoveries resolved to stable codes rather than separate
candidates: `Google Books` / `google_books` / `onix_3.0::google_books` ->
`GOOGLE_PLAY`; `ProQuest` / `Ebrary` / `proquest_ebrary` /
`onix_2.1::proquest_ebrary` -> `PROQUEST_EBOOK_CENTRAL`;
`ProQuest ExLibris` / `proquest_exlibris` -> `EX_LIBRIS_KB`;
`JISC KB` / `jisc_kb` -> `JISC_NBK`; `EBSCOHost` / `EBSCO Host` /
`ebsco_host` -> `EBSCO_HOST`; `CUL` -> `CAMBRIDGE_UNIVERSITY_LIBRARY`.
The generic DSpace/SWORD mechanism (`swordv2uploader.py`,
`dspaceuploader.py`) is an implementation mechanism, not a candidate. No
active channel observed in any inspected repository lacks a disposition.

## 4. Shared ownership and configuration-authority record

The following applies to **every** included destination
(source-owner-confirmed: Javi, CTO, 2026-08-06, ADR-01 specification section
9.4 and implementation authorization; `CTO-2026-08-06`):

```text
Commercial entitlement authority:
Statement of Work agreed with the publisher

Runtime desired-state authority:
Thoth publisher/platform assignment (future BE-02/BE-03 implementation;
not yet implemented at the inspected commits)

Temporary operational configuration source (mirror, not authority):
per-platform GitHub repository variables (publisher lists), uploader
configuration and existing workflow configuration in thoth-dissemination

Accountable operational owner:
COO

Operationally responsible role:
Metadata Specialist

Target credential owner:
Metadata Specialist

Current credential responsibility:
shared between engineering and operations during transition; recorded
honestly per destination below
```

Any unexplained mismatch between the Statement of Work, the Thoth assignment
and temporary operational configuration fails closed and must not broaden
processing. No publisher list is reproduced in this repository.

## 5. Shared adapter and feed profiles

A destination is distinct from the adapter or feed profile serving it.
Proposed adapter/feed-profile identities for later `DIS-01` mapping:

| Profile | Serves | Duplicate-safety requirement |
|---|---|---|
| `IA_API` | `INTERNET_ARCHIVE` | item-existence inspection/convergence |
| `OAPEN_DOAB_SWORD` | `OAPEN` + `DOAB` (linked) | one SWORD deposit for the linked pair; never two uploads |
| `SCIENCEOPEN_FTP` | `SCIENCE_OPEN` (manual invocation) | staff-controlled |
| `CUL_SWORD` | `CAMBRIDGE_UNIVERSITY_LIBRARY` | per-work deposit |
| `CROSSREF_DOI_DEPOSIT` | `CROSSREF` | redeposit updates the same DOI record |
| `FIGSHARE_API` | `FIGSHARE` | existing-record guard |
| `ZENODO_API` | `ZENODO` | existing-record guard |
| `MUSE_FTP` | `PROJECT_MUSE` | per-work upload |
| `JSTOR_FTP` | `JSTOR` | per-work upload into per-publisher folder |
| `EBSCO_HOST_SFTP` | `EBSCO_HOST` | per-work upload |
| `PROQUEST_EBOOK_CENTRAL_FTP` | `PROQUEST_EBOOK_CENTRAL` | per-work upload |
| `GOOGLE_PLAY_GCS` | `GOOGLE_PLAY` | bucket delivery under per-publisher collection code |
| `BKCI_FTP` | `BKCI` | per-work upload |
| `OCLC_KBART_PUBLIC` | `OCLC_KB` + `EX_LIBRIS_KB` (shared pull feed) | one publisher-level KBART output; no duplicate feed state; no uploader job |
| `JISC_NBK_MARC_S3` | `JISC_NBK` | **inactive**; creates no job or delivery |

`SwordV2Uploader`/`DSpaceUploader` code reuse between `OAPEN_DOAB_SWORD` and
`CUL_SWORD` is shared implementation, not a shared deposit: the two profiles
deposit to different services and create no cross-destination duplication.

## 6. Included destination records

Each record populates every field required by the ADR-01 specification
section 5 and the implementation authorization section 10. Ownership fields
shown as "shared record" carry the exact section 4 values.

### 6.1 INTERNET_ARCHIVE

```text
stable proposed enum code:      INTERNET_ARCHIVE
display name:                   Internet Archive
aliases and legacy names:       uploader key "InternetArchive";
                                LocationPlatform "Internet Archive"
independently selectable:       yes - dedicated uploader, dedicated daily
                                workflow, dedicated publisher list
                                (repository-verified)
current mechanism:              IAUploader (Internet Archive S3-like API) via
                                uploader key "InternetArchive"
behaviour:                      AutomaticPush
linked platform group:          none
shared adapter/feed profile:    IA_API (dedicated)
mechanism readiness:            active
assignment availability:        assignable
duplicate-delivery risk:        low - item-existence inspection and
                                convergence for owned items (v1.6.1-v1.6.4)
back-catalogue support:         yes - per-work uploader invocation; scheduled
                                run covers newly published works; back
                                catalogue delivered through staff-initiated
                                runs pending BE-04 durable jobs
accepted work types:            MONOGRAPH, EDITED_BOOK, JOURNAL_ISSUE,
                                TEXTBOOK, BOOK_SET (obtain_new_ids.py)
required file/metadata formats: canonical PDF publication file, cover,
                                IA item metadata including deterministic
                                JSON sidecar
incremental update support:     code converges owned items idempotently;
                                policy: automatic updates disabled initially
                                (conservative policy, CTO-2026-08-06)
withdrawal support:             none; operator-managed external process
retry/idempotency:              idempotent convergence, delayed-propagation
                                verification, conservative apply-mode batch
                                cap, reconcile workflow (ia_reconcile.yml)
location writeback:             immediate Thoth location writeback
                                (write_locations.py;
                                reconcile_internet_archive.py)
required credential category:   platform API credential; global
current credential
responsibility:                 shared (engineering-held dissemination
                                secrets) during transition
target credential owner:        Metadata Specialist (shared record)
commercial entitlement
authority:                      publisher Statement of Work (shared record)
runtime desired-state
authority:                      Thoth publisher/platform assignment
                                (shared record)
temporary operational
configuration source:           IA_ENV_PUBLISHERS repository variable and
                                dissemination secrets (shared record)
current publisher-config
source:                         IA_ENV_PUBLISHERS (contents not read)
accountable operational owner:  COO (shared record)
operationally responsible role: Metadata Specialist (shared record)
repository references:          thoth-dissemination@7a16edc0:
                                disseminator.py, iauploader.py,
                                internet_archive_policy.py,
                                .github/workflows/ia_bulk_disseminate.yml,
                                .github/workflows/ia_reconcile.yml,
                                write_locations.py, tests/test_iauploader.py
read-only verification method:  file inspection at recorded commit
evidence classification:        repository-verified
open questions:                 none blocking
known defects:                  none current
migration/backfill:             MIG-01 maps IA_ENV_PUBLISHERS membership to
                                assignments; no-job import mode
final decision:                 INCLUDED - AutomaticPush, assignable
```

### 6.2 OAPEN

```text
stable proposed enum code:      OAPEN
display name:                   OAPEN
aliases and legacy names:       uploader key "OAPEN"; LocationPlatform
                                "OAPEN"; deprecated FTP route retained in
                                config structure
independently selectable:       yes - dedicated uploader, weekly workflow,
                                dedicated publisher list
                                (repository-verified); linked to DOAB for
                                initial selection (settled invariant)
current mechanism:              OAPENSWORDUploader (SWORDv2 deposit to OAPEN
                                DSpace) via uploader key "OAPEN"
behaviour:                      AutomaticPush
linked platform group:          OAPEN_DOAB
shared adapter/feed profile:    OAPEN_DOAB_SWORD (shared with DOAB)
mechanism readiness:            active
assignment availability:        assignable (linked selection with DOAB;
                                backend-owned normalization)
duplicate-delivery risk:        controlled - one SWORD deposit serves the
                                linked pair; enabling both must not upload
                                twice (settled invariant)
back-catalogue support:         yes - per-work invocation; weekly scheduled
                                run; staff-initiated back catalogue pending
                                BE-04
accepted work types:            MONOGRAPH, EDITED_BOOK, JOURNAL_ISSUE,
                                TEXTBOOK, BOOK_SET (obtain_new_ids.py)
required file/metadata formats: PDF publication file; ONIX 3.0
                                onix_3.0::oapen export (accepted by oapen
                                and doab registry platforms); OAPEN metadata
                                profile incl. Thema handling (oapenthema.py)
incremental update support:     disabled initially (conservative policy)
withdrawal support:             none; operator-managed external process
retry/idempotency:              per-work deposit; re-run re-deposits;
                                notification email step (address
                                configuration in the dissemination
                                repository)
location writeback:             deferred catch-up
                                (oapen_catchup_locations.yaml,
                                obtain_oapen_locations.py)
required credential category:   platform service credential (SWORDv2);
                                global
current credential
responsibility:                 shared during transition (shared record)
target credential owner:        Metadata Specialist (shared record)
commercial entitlement
authority:                      publisher Statement of Work (shared record)
runtime desired-state
authority:                      Thoth publisher/platform assignment
temporary operational
configuration source:           OAPEN_ENV_PUBLISHERS variable and
                                dissemination secrets
current publisher-config
source:                         OAPEN_ENV_PUBLISHERS (contents not read)
accountable operational owner:  COO (shared record)
operationally responsible role: Metadata Specialist (shared record)
repository references:          thoth-dissemination@7a16edc0:
                                disseminator.py, oapensworduploader.py,
                                dspaceuploader.py, swordv2uploader.py,
                                oapenthema.py,
                                .github/workflows/oapen_bulk_disseminate.yaml,
                                .github/workflows/oapen_catchup_locations.yaml;
                                thoth@32123d3:
                                thoth-export-server/src/data.rs
read-only verification method:  file inspection at recorded commits
evidence classification:        repository-verified
open questions:                 none blocking
known defects:                  none current
migration/backfill:             MIG-01 maps OAPEN_ENV_PUBLISHERS to linked
                                OAPEN+DOAB assignments; no-job import;
                                linked-state anomalies reported
final decision:                 INCLUDED - AutomaticPush, assignable,
                                linked OAPEN_DOAB
```

### 6.3 DOAB

```text
stable proposed enum code:      DOAB
display name:                   DOAB
aliases and legacy names:       LocationPlatform "DOAB"; export registry
                                platform "doab"
independently selectable:       yes as a destination identity
                                (separately meaningful listing; ADR-0002 and
                                settled invariants); initially selected only
                                through the linked OAPEN_DOAB pair
current mechanism:              no separate uploader key; delivery through
                                the OAPEN SWORD deposit, from which DOAB
                                ingests (registry: doab accepts
                                onix_3.0::oapen)
behaviour:                      AutomaticPush (delivered via linked deposit)
linked platform group:          OAPEN_DOAB
shared adapter/feed profile:    OAPEN_DOAB_SWORD (shared with OAPEN)
mechanism readiness:            active (via shared adapter)
assignment availability:        assignable (linked selection; backend-owned
                                normalization; no independent app rule)
duplicate-delivery risk:        controlled - linked pair produces exactly one
                                deposit; no separate DOAB upload exists
back-catalogue support:         follows the linked OAPEN deposit
accepted work types:            as OAPEN (shared delivery)
required file/metadata formats: as OAPEN (onix_3.0::oapen)
incremental update support:     disabled initially (conservative policy)
withdrawal support:             none; operator-managed external process
retry/idempotency:              as OAPEN (single shared deposit)
location writeback:             deferred catch-up records OAPEN/DOAB
                                locations (obtain_oapen_locations.py)
required credential category:   no separate machine credential (shared
                                OAPEN deposit credential)
current credential
responsibility:                 as OAPEN (shared record)
target credential owner:        Metadata Specialist (shared record)
commercial entitlement
authority:                      publisher Statement of Work (shared record)
runtime desired-state
authority:                      Thoth publisher/platform assignment
temporary operational
configuration source:           same linked configuration as OAPEN
current publisher-config
source:                         OAPEN_ENV_PUBLISHERS (linked; contents not
                                read)
accountable operational owner:  COO (shared record)
operationally responsible role: Metadata Specialist (shared record)
repository references:          thoth@32123d3:
                                thoth-export-server/src/data.rs (doab);
                                thoth-dissemination@7a16edc0:
                                obtain_oapen_locations.py, disseminator.py
                                (absence of separate key)
read-only verification method:  file inspection at recorded commits
evidence classification:        repository-verified
open questions:                 none blocking
known defects:                  none current
migration/backfill:             created together with OAPEN assignments as a
                                linked pair; no-job import
final decision:                 INCLUDED - AutomaticPush via linked deposit,
                                assignable, linked OAPEN_DOAB
```

### 6.4 SCIENCE_OPEN

```text
stable proposed enum code:      SCIENCE_OPEN
display name:                   ScienceOpen
aliases and legacy names:       uploader key "ScienceOpen"; LocationPlatform
                                "ScienceOpen"
independently selectable:       yes - dedicated uploader and credentials
                                (repository-verified); invoked manually only
current mechanism:              SOUploader (FTP) via uploader key
                                "ScienceOpen"; manual invocation through
                                disseminate.yml / manual_disseminate.yml
behaviour:                      Manual (staff-initiated push uploader; no
                                schedule; no automatic job)
linked platform group:          none
shared adapter/feed profile:    SCIENCEOPEN_FTP (dedicated)
mechanism readiness:            active for manual staff invocation
assignment availability:        assignable (assignment records the
                                destination; creates no automatic job -
                                manual destinations create no uploader job)
duplicate-delivery risk:        staff-controlled; re-run re-uploads
back-catalogue support:         manual staff-initiated runs only
accepted work types:            as uploader invocation selects (per-work
                                manual dispatch of the same work-type set)
required file/metadata formats: PDF publication file and cover per
                                souploader.py
incremental update support:     none automatic (manual destination)
withdrawal support:             none; operator-managed external process
retry/idempotency:              manual re-run; no automatic retry
location writeback:             none automatic
required credential category:   SFTP/FTP credential; global
current credential
responsibility:                 shared during transition (shared record)
target credential owner:        Metadata Specialist (shared record)
commercial entitlement
authority:                      publisher Statement of Work (shared record)
runtime desired-state
authority:                      Thoth publisher/platform assignment
temporary operational
configuration source:           dissemination secrets; manual eligibility
                                decision at invocation time
current publisher-config
source:                         manual staff selection (no publisher-list
                                variable exists for ScienceOpen)
accountable operational owner:  COO (shared record)
operationally responsible role: Metadata Specialist (shared record)
repository references:          thoth-dissemination@7a16edc0:
                                disseminator.py, souploader.py,
                                .github/workflows/disseminate.yml,
                                .github/workflows/manual_disseminate.yml
read-only verification method:  file inspection at recorded commit
evidence classification:        repository-verified
open questions:                 none blocking
known defects:                  none current
migration/backfill:             assignments created only for publishers with
                                a Statement-of-Work ScienceOpen service;
                                no-job import
final decision:                 INCLUDED - Manual, assignable, job-free
```

### 6.5 CAMBRIDGE_UNIVERSITY_LIBRARY

```text
stable proposed enum code:      CAMBRIDGE_UNIVERSITY_LIBRARY
display name:                   Cambridge University Library
aliases and legacy names:       uploader key "CUL"
independently selectable:       yes - dedicated uploader, monthly workflow,
                                dedicated publisher list
                                (repository-verified)
current mechanism:              CULUploader (SWORDv2 deposit) via uploader
                                key "CUL"
behaviour:                      AutomaticPush
linked platform group:          none
shared adapter/feed profile:    CUL_SWORD (dedicated; shares SwordV2Uploader
                                implementation code with OAPEN, not a shared
                                deposit)
mechanism readiness:            active
assignment availability:        assignable
duplicate-delivery risk:        per-work deposit; re-run re-deposits
back-catalogue support:         yes - per-work invocation; monthly scheduled
                                run; staff-initiated back catalogue pending
                                BE-04
accepted work types:            MONOGRAPH, EDITED_BOOK, JOURNAL_ISSUE,
                                TEXTBOOK, BOOK_SET (obtain_new_ids.py)
required file/metadata formats: PDF publication file and metadata per
                                culuploader.py SWORD deposit
incremental update support:     disabled initially (conservative policy)
withdrawal support:             none; operator-managed external process
retry/idempotency:              per-work deposit; no existing-record guard
location writeback:             immediate, but recorded under
                                LocationPlatform 'OTHER' (culuploader.py:50)
                                - known current behaviour to revisit in later
                                implementation work
required credential category:   platform service credential (SWORDv2);
                                global; pilot collaboration service
current credential
responsibility:                 shared during transition (shared record)
target credential owner:        Metadata Specialist (shared record)
commercial entitlement
authority:                      publisher Statement of Work (shared record)
runtime desired-state
authority:                      Thoth publisher/platform assignment
temporary operational
configuration source:           CUL_ENV_PUBLISHERS variable and dissemination
                                secrets
current publisher-config
source:                         CUL_ENV_PUBLISHERS (contents not read)
accountable operational owner:  COO (shared record)
operationally responsible role: Metadata Specialist (shared record)
repository references:          thoth-dissemination@7a16edc0:
                                disseminator.py, culuploader.py,
                                swordv2uploader.py,
                                .github/workflows/cul_bulk_disseminate.yml
read-only verification method:  file inspection at recorded commit
evidence classification:        repository-verified
open questions:                 none blocking (the 'OTHER' writeback is
                                recorded behaviour, not an inventory
                                ambiguity)
known defects:                  none current
migration/backfill:             MIG-01 maps CUL_ENV_PUBLISHERS to
                                assignments; no-job import
final decision:                 INCLUDED - AutomaticPush, assignable
```

### 6.6 CROSSREF

```text
stable proposed enum code:      CROSSREF
display name:                   Crossref
aliases and legacy names:       uploader key "Crossref"; registry platform
                                "crossref" ("CrossRef" legacy capitalization)
independently selectable:       yes - dedicated uploader, hourly workflow,
                                per-publisher credentials
                                (repository-verified)
current mechanism:              CrossrefUploader (HTTPS DOI deposit to the
                                Crossref deposit endpoint) via uploader key
                                "Crossref"
behaviour:                      AutomaticPush (metadata-only DOI deposit)
linked platform group:          none
shared adapter/feed profile:    CROSSREF_DOI_DEPOSIT (dedicated)
mechanism readiness:            active
assignment availability:        assignable
duplicate-delivery risk:        low - redeposit updates the same DOI record;
                                deposit API reports errors asynchronously
back-catalogue support:         yes - per-work deposit; hourly scheduled run
accepted work types:            MONOGRAPH, EDITED_BOOK, JOURNAL_ISSUE,
                                TEXTBOOK, BOOK_SET (obtain_new_ids.py);
                                deposit requires a DOI
required file/metadata formats: doideposit::crossref XML only; no content
                                file
incremental update support:     SUPPORTED - Crossref DOI deposits are the
                                one destination with approved automatic
                                update support (conservative policy,
                                CTO-2026-08-06)
withdrawal support:             none; operator-managed external process
retry/idempotency:              redeposit-safe by DOI; minimal deposit API
                                error surface recorded in code comments
location writeback:             none
required credential category:   per-publisher platform API credential
                                (exact identifiers intentionally omitted)
current credential
responsibility:                 shared during transition (shared record)
target credential owner:        Metadata Specialist (shared record)
commercial entitlement
authority:                      publisher Statement of Work (shared record)
runtime desired-state
authority:                      Thoth publisher/platform assignment
temporary operational
configuration source:           CR_ENV_PUBLISHERS variable and per-publisher
                                dissemination secrets
current publisher-config
source:                         CR_ENV_PUBLISHERS (contents not read)
accountable operational owner:  COO (shared record)
operationally responsible role: Metadata Specialist (shared record)
repository references:          thoth-dissemination@7a16edc0:
                                disseminator.py, crossrefuploader.py,
                                .github/workflows/cr_bulk_disseminate.yml;
                                thoth@32123d3:
                                thoth-export-server/src/xml/doideposit_crossref.rs
read-only verification method:  file inspection at recorded commits
evidence classification:        repository-verified
open questions:                 none blocking
known defects:                  none current
migration/backfill:             MIG-01 maps CR_ENV_PUBLISHERS to
                                assignments; no-job import
final decision:                 INCLUDED - AutomaticPush, assignable,
                                update-supported
```

### 6.7 FIGSHARE

```text
stable proposed enum code:      FIGSHARE
display name:                   Figshare
aliases and legacy names:       uploader key "Figshare"
independently selectable:       yes - dedicated uploader, monthly workflow,
                                dedicated publisher list
                                (repository-verified)
current mechanism:              FigshareUploader (fsuploader.py, Figshare
                                API) via uploader key "Figshare"
behaviour:                      AutomaticPush
linked platform group:          none
shared adapter/feed profile:    FIGSHARE_API (dedicated)
mechanism readiness:            active
assignment availability:        assignable
duplicate-delivery risk:        low - upload fails when an item for the work
                                already exists (fsuploader.py existing-record
                                guard)
back-catalogue support:         yes - per-work invocation; monthly scheduled
                                run
accepted work types:            MONOGRAPH, EDITED_BOOK, JOURNAL_ISSUE,
                                TEXTBOOK, BOOK_SET (obtain_new_ids.py)
required file/metadata formats: publication files and metadata articles per
                                fsuploader.py (private-to-public publish
                                flow)
incremental update support:     disabled initially (conservative policy);
                                current guard blocks simple repeat upload
withdrawal support:             none; operator-managed external process
retry/idempotency:              existing-record guard prevents duplicates;
                                re-run after partial failure requires staff
                                attention
location writeback:             immediate per-publication writeback
required credential category:   platform API credential (token category);
                                global
current credential
responsibility:                 shared during transition (shared record)
target credential owner:        Metadata Specialist (shared record)
commercial entitlement
authority:                      publisher Statement of Work (shared record)
runtime desired-state
authority:                      Thoth publisher/platform assignment
temporary operational
configuration source:           FS_ENV_PUBLISHERS variable and dissemination
                                secrets
current publisher-config
source:                         FS_ENV_PUBLISHERS (contents not read)
accountable operational owner:  COO (shared record)
operationally responsible role: Metadata Specialist (shared record)
repository references:          thoth-dissemination@7a16edc0:
                                disseminator.py, fsuploader.py,
                                .github/workflows/fs_bulk_disseminate.yml
read-only verification method:  file inspection at recorded commit
evidence classification:        repository-verified
open questions:                 none blocking
known defects:                  none current
migration/backfill:             MIG-01 maps FS_ENV_PUBLISHERS to
                                assignments; no-job import
final decision:                 INCLUDED - AutomaticPush, assignable
```

### 6.8 ZENODO

```text
stable proposed enum code:      ZENODO
display name:                   Zenodo
aliases and legacy names:       uploader key "Zenodo"; LocationPlatform
                                "Zenodo"
independently selectable:       yes - dedicated uploader, monthly workflow,
                                dedicated publisher list
                                (repository-verified)
current mechanism:              ZenodoUploader (Zenodo API) via uploader key
                                "Zenodo"
behaviour:                      AutomaticPush
linked platform group:          none
shared adapter/feed profile:    ZENODO_API (dedicated)
mechanism readiness:            active
assignment availability:        assignable
duplicate-delivery risk:        low - upload fails when an item with the
                                Work ID already exists (zenodouploader.py
                                existing-record guard)
back-catalogue support:         yes - per-work invocation; monthly scheduled
                                run
accepted work types:            MONOGRAPH, EDITED_BOOK, JOURNAL_ISSUE,
                                TEXTBOOK, BOOK_SET (obtain_new_ids.py)
required file/metadata formats: publication files and deposition metadata
                                per zenodouploader.py
incremental update support:     disabled initially (conservative policy);
                                published depositions reject re-publication
withdrawal support:             none; operator-managed external process
retry/idempotency:              existing-record guard prevents duplicates
location writeback:             immediate per-publication writeback
required credential category:   platform API credential (token category);
                                global
current credential
responsibility:                 shared during transition (shared record)
target credential owner:        Metadata Specialist (shared record)
commercial entitlement
authority:                      publisher Statement of Work (shared record)
runtime desired-state
authority:                      Thoth publisher/platform assignment
temporary operational
configuration source:           ZN_ENV_PUBLISHERS variable and dissemination
                                secrets
current publisher-config
source:                         ZN_ENV_PUBLISHERS (contents not read)
accountable operational owner:  COO (shared record)
operationally responsible role: Metadata Specialist (shared record)
repository references:          thoth-dissemination@7a16edc0:
                                disseminator.py, zenodouploader.py,
                                .github/workflows/zn_bulk_disseminate.yml
read-only verification method:  file inspection at recorded commit
evidence classification:        repository-verified
open questions:                 none blocking
known defects:                  none current
migration/backfill:             MIG-01 maps ZN_ENV_PUBLISHERS to
                                assignments; no-job import
final decision:                 INCLUDED - AutomaticPush, assignable
```

### 6.9 PROJECT_MUSE

```text
stable proposed enum code:      PROJECT_MUSE
display name:                   Project MUSE
aliases and legacy names:       uploader key "ProjectMUSE"; historical
                                workflow key "MUSE" (resolved 2026-04-23);
                                LocationPlatform "Project MUSE"
independently selectable:       yes - dedicated uploader, weekly workflow,
                                per-publisher credentials
                                (repository-verified)
current mechanism:              MUSEUploader (FTP) via uploader key
                                "ProjectMUSE"
behaviour:                      AutomaticPush
linked platform group:          none
shared adapter/feed profile:    MUSE_FTP (dedicated)
mechanism readiness:            active
assignment availability:        assignable
duplicate-delivery risk:        per-work upload; re-run re-uploads
back-catalogue support:         yes - per-work invocation; weekly scheduled
                                run
accepted work types:            MONOGRAPH, EDITED_BOOK, JOURNAL_ISSUE,
                                TEXTBOOK, BOOK_SET (obtain_new_ids.py)
required file/metadata formats: PDF publication file, cover and ONIX 3.0
                                onix_3.0::project_muse export
incremental update support:     disabled initially (conservative policy)
withdrawal support:             none; operator-managed external process
retry/idempotency:              per-work upload; deferred location catch-up
                                tolerates delayed MUSE publication
location writeback:             deferred catch-up
                                (muse_catchup_locations.yaml,
                                obtain_muse_locations.py)
required credential category:   per-publisher SFTP/FTP credential
                                (exact identifiers intentionally omitted)
current credential
responsibility:                 shared during transition (shared record)
target credential owner:        Metadata Specialist (shared record)
commercial entitlement
authority:                      publisher Statement of Work (shared record)
runtime desired-state
authority:                      Thoth publisher/platform assignment
temporary operational
configuration source:           MUSE_ENV_PUBLISHERS variable and
                                per-publisher dissemination secrets
current publisher-config
source:                         MUSE_ENV_PUBLISHERS (contents not read)
accountable operational owner:  COO (shared record)
operationally responsible role: Metadata Specialist (shared record)
repository references:          thoth-dissemination@7a16edc0:
                                disseminator.py, museuploader.py,
                                .github/workflows/muse_bulk_disseminate.yaml,
                                .github/workflows/muse_catchup_locations.yaml,
                                obtain_muse_locations.py; fix commit
                                1a66da8f1700d8c76bf8fda2938b8729be0a93b6;
                                thoth@32123d3:
                                thoth-export-server/src/xml/onix3_project_muse.rs
read-only verification method:  file inspection at recorded commits
evidence classification:        repository-verified
open questions:                 none blocking
known defects:                  none current; historical/resolved workflow
                                key mismatch ("MUSE" vs "ProjectMUSE"),
                                fixed by 1a66da8f on 23 April 2026 - must
                                not be presented as current
migration/backfill:             MIG-01 maps MUSE_ENV_PUBLISHERS to
                                assignments; no-job import
final decision:                 INCLUDED - AutomaticPush, assignable
```

### 6.10 JSTOR

```text
stable proposed enum code:      JSTOR
display name:                   JSTOR
aliases and legacy names:       uploader key "JSTOR"; LocationPlatform
                                "JSTOR"
independently selectable:       yes - dedicated uploader, weekly workflow,
                                per-publisher folder configuration
                                (repository-verified)
current mechanism:              JSTORUploader (FTP into per-publisher
                                folder) via uploader key "JSTOR"
behaviour:                      AutomaticPush
linked platform group:          none
shared adapter/feed profile:    JSTOR_FTP (dedicated)
mechanism readiness:            active
assignment availability:        assignable
duplicate-delivery risk:        per-work upload; re-run re-uploads
back-catalogue support:         yes - per-work invocation; weekly scheduled
                                run
accepted work types:            MONOGRAPH, EDITED_BOOK, JOURNAL_ISSUE,
                                TEXTBOOK, BOOK_SET (obtain_new_ids.py)
required file/metadata formats: PDF publication file, cover and ONIX 3.0
                                (onix_3.0::jstor; jstor registry platform
                                also accepts onix_3.0::project_muse)
incremental update support:     disabled initially (conservative policy)
withdrawal support:             none; operator-managed external process
retry/idempotency:              per-work upload; no existing-record guard
location writeback:             none automatic
required credential category:   SFTP/FTP credential (global) plus
                                per-publisher folder-name configuration
                                (exact identifiers intentionally omitted)
current credential
responsibility:                 shared during transition (shared record)
target credential owner:        Metadata Specialist (shared record)
commercial entitlement
authority:                      publisher Statement of Work (shared record)
runtime desired-state
authority:                      Thoth publisher/platform assignment
temporary operational
configuration source:           JSTOR_ENV_PUBLISHERS variable and
                                dissemination secrets
current publisher-config
source:                         JSTOR_ENV_PUBLISHERS (contents not read)
accountable operational owner:  COO (shared record)
operationally responsible role: Metadata Specialist (shared record)
repository references:          thoth-dissemination@7a16edc0:
                                disseminator.py, jstoruploader.py,
                                .github/workflows/jstor_bulk_disseminate.yaml;
                                thoth@32123d3:
                                thoth-export-server/src/xml/onix3_jstor.rs
read-only verification method:  file inspection at recorded commits
evidence classification:        repository-verified
open questions:                 none blocking
known defects:                  none current
migration/backfill:             MIG-01 maps JSTOR_ENV_PUBLISHERS to
                                assignments; no-job import
final decision:                 INCLUDED - AutomaticPush, assignable
```

### 6.11 EBSCO_HOST

```text
stable proposed enum code:      EBSCO_HOST
display name:                   EBSCOHost
aliases and legacy names:       uploader key "EBSCOHost"; LocationPlatform /
                                export registry "EBSCO Host" / "ebsco_host";
                                SLA selection "EBSCOHost"
independently selectable:       yes - dedicated uploader, weekly workflow,
                                dedicated publisher list; confirmed current
                                push destination (repository-verified;
                                LEDGER:EBSCO-02, EBSCO-03, EBSCO-05)
current mechanism:              EBSCOUploader (SFTP: PDF and/or EPUB, cover,
                                onix_2.1::ebsco_host metadata) via uploader
                                key "EBSCOHost"
behaviour:                      AutomaticPush
linked platform group:          none
shared adapter/feed profile:    EBSCO_HOST_SFTP (dedicated). NOT the OCLC
                                KBART feed; NOT related to EBSCO_KB
mechanism readiness:            active
assignment availability:        assignable
duplicate-delivery risk:        per-work upload; re-run re-uploads
back-catalogue support:         yes - per-work invocation; weekly scheduled
                                run
accepted work types:            MONOGRAPH, EDITED_BOOK, JOURNAL_ISSUE,
                                TEXTBOOK, BOOK_SET (obtain_new_ids.py)
required file/metadata formats: PDF and/or EPUB content, cover,
                                ONIX 2.1 onix_2.1::ebsco_host export
incremental update support:     disabled initially (conservative policy)
withdrawal support:             none; operator-managed external process
retry/idempotency:              per-work upload; notification email step
                                (address configuration in the dissemination
                                repository)
location writeback:             none automatic
required credential category:   SFTP credential; global
current credential
responsibility:                 shared during transition (shared record)
target credential owner:        Metadata Specialist (shared record)
commercial entitlement
authority:                      publisher Statement of Work (shared record;
                                LEDGER:EBSCO-05 - SLA lists EBSCOHost)
runtime desired-state
authority:                      Thoth publisher/platform assignment
temporary operational
configuration source:           EH_ENV_PUBLISHERS variable and dissemination
                                secrets
current publisher-config
source:                         EH_ENV_PUBLISHERS (contents not read)
accountable operational owner:  COO (shared record)
operationally responsible role: Metadata Specialist (shared record)
repository references:          thoth-dissemination@7a16edc0:
                                disseminator.py, ebscouploader.py,
                                .github/workflows/eh_bulk_disseminate.yaml;
                                thoth@32123d3:
                                thoth-export-server/src/xml/onix21_ebsco_host.rs
read-only verification method:  file inspection at recorded commits;
                                ledger attribution for SLA/workflow evidence
evidence classification:        repository-verified (mechanism);
                                source-owner-confirmed (commercial
                                selection: LEDGER:EBSCO-02, EBSCO-05)
open questions:                 none blocking
known defects:                  none current
migration/backfill:             MIG-01 maps EH_ENV_PUBLISHERS to
                                assignments; no-job import
final decision:                 INCLUDED - AutomaticPush, assignable
```

### 6.12 PROQUEST_EBOOK_CENTRAL

```text
stable proposed enum code:      PROQUEST_EBOOK_CENTRAL
display name:                   ProQuest Ebook Central
aliases and legacy names:       "ProQuest" (current uploader key and SLA
                                label; LEDGER:PROQUEST-02, PROQUEST-03,
                                PROQUEST-06); "Ebrary" (former name;
                                LEDGER:PROQUEST-04, PROQUEST-05); export
                                flavour "onix_2.1::proquest_ebrary" and
                                registry platform "proquest_ebrary". The
                                vendor umbrella name "ProQuest" is not an
                                enum value.
independently selectable:       yes - dedicated uploader, weekly workflow,
                                dedicated publisher list
                                (repository-verified; LEDGER:PROQUEST-01,
                                PROQUEST-02, PROQUEST-03)
current mechanism:              ProquestUploader (FTP content-and-metadata)
                                via uploader key "ProQuest"
behaviour:                      AutomaticPush
linked platform group:          none
shared adapter/feed profile:    PROQUEST_EBOOK_CENTRAL_FTP (dedicated).
                                Distinct from EX_LIBRIS_KB and from any
                                Serial Solutions route (not aliases)
mechanism readiness:            active, with a recorded current defect
assignment availability:        assignable
duplicate-delivery risk:        per-work upload; re-run re-uploads
back-catalogue support:         yes - per-work invocation; weekly scheduled
                                run
accepted work types:            MONOGRAPH, EDITED_BOOK, JOURNAL_ISSUE,
                                TEXTBOOK, BOOK_SET (obtain_new_ids.py)
required file/metadata formats: PDF and/or EPUB content, cover,
                                ONIX 2.1 onix_2.1::proquest_ebrary export
incremental update support:     disabled initially (conservative policy)
withdrawal support:             none; operator-managed external process
retry/idempotency:              per-work upload; scheduled delivery must not
                                be presented as fully healthy while the
                                defect below stands
location writeback:             none automatic
required credential category:   SFTP/FTP credential; global
current credential
responsibility:                 shared during transition (shared record)
target credential owner:        Metadata Specialist (shared record)
commercial entitlement
authority:                      publisher Statement of Work (shared record;
                                LEDGER:PROQUEST-06 - SLA label "ProQuest"
                                means Ebook Central in current usage)
runtime desired-state
authority:                      Thoth publisher/platform assignment
temporary operational
configuration source:           PQ_ENV_PUBLISHERS variable and dissemination
                                secrets
current publisher-config
source:                         PQ_ENV_PUBLISHERS (contents not read)
accountable operational owner:  COO (shared record)
operationally responsible role: Metadata Specialist (shared record)
repository references:          thoth-dissemination@7a16edc0:
                                disseminator.py, proquestuploader.py,
                                .github/workflows/pq_bulk_disseminate.yaml;
                                thoth@32123d3:
                                thoth-export-server/src/xml/onix21_proquest_ebrary.rs
read-only verification method:  file inspection at recorded commits;
                                ledger attribution for naming evidence
evidence classification:        repository-verified (mechanism, defect);
                                source-owner-confirmed (naming/aliases:
                                LEDGER:PROQUEST-01 through PROQUEST-06)
open questions:                 none blocking
known defects:                  CURRENT - EPUB-only/PDF-ISBN ordering
                                defect: proquestuploader.py sets the
                                filename root from get_isbn('PDF') (line 43)
                                before the PDF/EPUB fallback, so EPUB-only
                                works can fail. Recorded, not fixed, not
                                normalized away.
migration/backfill:             MIG-01 maps PQ_ENV_PUBLISHERS to
                                assignments; no-job import
final decision:                 INCLUDED - AutomaticPush, assignable,
                                current defect recorded
```

### 6.13 GOOGLE_PLAY

```text
stable proposed enum code:      GOOGLE_PLAY
display name:                   Google Play Books
aliases and legacy names:       "Google Books" (legacy/user-recognized
                                alias; LocationPlatform "Google Books";
                                export registry "google_books" and flavour
                                "onix_3.0::google_books"); uploader key
                                "GooglePlay". One destination, one code
                                (CTO-2026-08-06, specification section 8.1)
independently selectable:       yes - dedicated uploader, daily workflow,
                                dedicated publisher list and per-publisher
                                collection codes (repository-verified)
current mechanism:              GooglePlayUploader (cloud bucket delivery
                                with per-publisher collection code) via
                                uploader key "GooglePlay"
behaviour:                      AutomaticPush
linked platform group:          none
shared adapter/feed profile:    GOOGLE_PLAY_GCS (dedicated)
mechanism readiness:            active
assignment availability:        assignable
duplicate-delivery risk:        bucket re-delivery overwrites the same
                                object path per work
back-catalogue support:         yes - per-work invocation; daily scheduled
                                run
accepted work types:            MONOGRAPH, EDITED_BOOK, JOURNAL_ISSUE,
                                TEXTBOOK, BOOK_SET (obtain_new_ids.py)
required file/metadata formats: content files plus ONIX 3.0
                                onix_3.0::google_books export
incremental update support:     disabled initially (conservative policy)
withdrawal support:             none; operator-managed external process
retry/idempotency:              re-delivery to bucket; Google-side ingest is
                                asynchronous
location writeback:             none automatic
required credential category:   cloud service-account credential (workload
                                identity federation; bucket configuration)
current credential
responsibility:                 shared during transition (shared record)
target credential owner:        Metadata Specialist (shared record)
commercial entitlement
authority:                      publisher Statement of Work (shared record)
runtime desired-state
authority:                      Thoth publisher/platform assignment
temporary operational
configuration source:           GP_ENV_PUBLISHERS variable, per-publisher
                                collection-code configuration (exact
                                identifiers intentionally omitted) and
                                dissemination secrets
current publisher-config
source:                         GP_ENV_PUBLISHERS (contents not read)
accountable operational owner:  COO (shared record)
operationally responsible role: Metadata Specialist (shared record)
repository references:          thoth-dissemination@7a16edc0:
                                disseminator.py, googleplayuploader.py,
                                .github/workflows/gp_bulk_disseminate.yaml;
                                thoth@32123d3:
                                thoth-export-server/src/xml/onix3_google_books.rs
read-only verification method:  file inspection at recorded commits
evidence classification:        repository-verified (mechanism);
                                source-owner-confirmed (single-destination
                                naming decision: CTO-2026-08-06)
open questions:                 none blocking
known defects:                  none current
migration/backfill:             MIG-01 maps GP_ENV_PUBLISHERS to
                                assignments; no-job import
final decision:                 INCLUDED - AutomaticPush, assignable,
                                single Google destination
```

### 6.14 BKCI

```text
stable proposed enum code:      BKCI
display name:                   Book Citation Index
aliases and legacy names:       uploader key "BKCI"; Clarivate Web of
                                Science Book Citation Index
independently selectable:       yes - dedicated uploader, monthly workflow,
                                per-publisher credentials
                                (repository-verified)
current mechanism:              BKCIUploader (FTP) via uploader key "BKCI"
behaviour:                      AutomaticPush
linked platform group:          none
shared adapter/feed profile:    BKCI_FTP (dedicated)
mechanism readiness:            active
assignment availability:        assignable
duplicate-delivery risk:        per-work upload; re-run re-uploads
back-catalogue support:         yes - per-work invocation; monthly scheduled
                                run
accepted work types:            MONOGRAPH, EDITED_BOOK, JOURNAL_ISSUE,
                                TEXTBOOK, BOOK_SET (obtain_new_ids.py)
required file/metadata formats: content and metadata files per
                                bkciuploader.py
incremental update support:     disabled initially (conservative policy)
withdrawal support:             none; operator-managed external process
retry/idempotency:              per-work upload; notification email step
                                (address configuration in the dissemination
                                repository)
location writeback:             none automatic
required credential category:   per-publisher SFTP/FTP credential
                                (exact identifiers intentionally omitted)
current credential
responsibility:                 shared during transition (shared record)
target credential owner:        Metadata Specialist (shared record)
commercial entitlement
authority:                      publisher Statement of Work (shared record)
runtime desired-state
authority:                      Thoth publisher/platform assignment
temporary operational
configuration source:           BKCI_ENV_PUBLISHERS variable and
                                per-publisher dissemination secrets
current publisher-config
source:                         BKCI_ENV_PUBLISHERS (contents not read)
accountable operational owner:  COO (shared record)
operationally responsible role: Metadata Specialist (shared record)
repository references:          thoth-dissemination@7a16edc0:
                                disseminator.py, bkciuploader.py,
                                .github/workflows/bkci_bulk_disseminate.yaml
read-only verification method:  file inspection at recorded commit
evidence classification:        repository-verified
open questions:                 none blocking
known defects:                  none current
migration/backfill:             MIG-01 maps BKCI_ENV_PUBLISHERS to
                                assignments; no-job import
final decision:                 INCLUDED - AutomaticPush, assignable
```

### 6.15 OCLC_KB

```text
stable proposed enum code:      OCLC_KB
display name:                   OCLC Knowledge Base
aliases and legacy names:       LocationPlatform "OCLC KB"; export registry
                                platform "oclc_kb"; app label "OCLC"
independently selectable:       yes - separately onboarded consumer with
                                per-publisher OCLC collection identifiers
                                (LEDGER:KBART-01, KBART-02)
current mechanism:              monthly OCLC harvest of the publisher-level
                                Thoth Export API OCLC KBART output; no
                                uploader
behaviour:                      PullFeed
linked platform group:          none (independently onboarded; shares only
                                the feed profile with EX_LIBRIS_KB)
shared adapter/feed profile:    OCLC_KBART_PUBLIC (= Thoth export
                                specification kbart::oclc, publisher-level
                                output; shared with EX_LIBRIS_KB)
mechanism readiness:            active (feed exists and is harvested)
assignment availability:        assignable (assignment = feed
                                membership/onboarding record; creates no
                                uploader job)
duplicate-delivery risk:        none if duplicate-safety holds: one
                                publisher-level KBART output profile; no
                                duplicate feed state; enabling both KBART
                                consumers must not generate two equivalent
                                outputs (LEDGER:KBART-03)
back-catalogue support:         feed exposes current full state; no
                                back-catalogue job exists or is created
accepted work types:            works exposed by the kbart::oclc export
                                (KBART rows per work/publication;
                                kbart_oclc.rs)
required file/metadata formats: OCLC KBART TSV (kbart::oclc), including
                                oclc_number when available
incremental update support:     feed reflects current state on each harvest;
                                no push-update job (conservative policy)
withdrawal support:             removal from the feed reflects current
                                state; no push withdrawal
retry/idempotency:              destination-controlled harvest schedule
                                (monthly); feed regeneration is stateless
location writeback:             none (feed membership only)
required credential category:   no machine credential (public feed URL)
current credential
responsibility:                 not applicable - no machine credential;
                                onboarding communication is staff-managed
target credential owner:        Metadata Specialist (shared record; for the
                                onboarding communication responsibility)
commercial entitlement
authority:                      publisher Statement of Work (shared record)
runtime desired-state
authority:                      Thoth publisher/platform assignment
temporary operational
configuration source:           existing OCLC arrangements/records; new
                                publishers communicated to OCLC
                                (LEDGER:KBART-01)
current publisher-config
source:                         OCLC onboarding records (source-owner
                                domain; no repository list; contents not
                                read)
accountable operational owner:  COO (shared record)
operationally responsible role: Metadata Specialist (shared record)
repository references:          thoth@32123d3:
                                thoth-export-server/src/data.rs (kbart::oclc,
                                oclc_kb),
                                thoth-export-server/src/csv/kbart_oclc.rs,
                                thoth-api/src/model/work/mod.rs (oclc field)
read-only verification method:  file inspection at recorded commit; ledger
                                attribution for harvest arrangement
evidence classification:        repository-verified (feed output and
                                registry); source-owner-confirmed (current
                                monthly harvest and onboarding:
                                LEDGER:KBART-01, KBART-02)
open questions:                 none blocking
known defects:                  none current
migration/backfill:             assignments backfilled from existing OCLC
                                onboarding records; creates no feed job;
                                EXP-01 later serves the per-publisher index
final decision:                 INCLUDED - PullFeed, assignable, shared
                                OCLC_KBART_PUBLIC
```

### 6.16 EX_LIBRIS_KB

```text
stable proposed enum code:      EX_LIBRIS_KB
display name:                   Ex Libris Knowledge Base
aliases and legacy names:       "ProQuest ExLibris" (LocationPlatform,
                                export registry "proquest_exlibris", app
                                label); "ProQuest - was ExLibris"
                                (historical; LEDGER:PROQUEST-04). Not an
                                alias of Ebook Central or Serial Solutions.
independently selectable:       yes - separate per-publisher onboarding and
                                collection configuration for Alma/SFX/360
                                ingest, even when commercially bundled
                                (LEDGER:PROQUEST-01, KBART-02, KBART-03;
                                CTO-2026-08-06)
current mechanism:              monthly Ex Libris harvest of the same
                                publisher-level Thoth OCLC KBART output for
                                an independently onboarded publisher subset;
                                no uploader
behaviour:                      PullFeed
linked platform group:          none (independently onboarded; shares only
                                the feed profile with OCLC_KB)
shared adapter/feed profile:    OCLC_KBART_PUBLIC (shared with OCLC_KB)
mechanism readiness:            active (harvest arrangement confirmed)
assignment availability:        assignable (assignment = onboarded
                                consumer subset membership; creates no
                                uploader job)
duplicate-delivery risk:        none if duplicate-safety holds: enabling
                                OCLC_KB and EX_LIBRIS_KB together must not
                                generate or maintain duplicate KBART
                                outputs and must create no uploader job
                                (LEDGER:KBART-03; settled input 13.4)
back-catalogue support:         feed exposes current full state; no job
accepted work types:            works exposed by the kbart::oclc export
required file/metadata formats: OCLC KBART TSV (kbart::oclc); no Ex
                                Libris-specific KBART flavour exists
                                (LEDGER:KBART-03)
incremental update support:     feed reflects current state on each
                                monthly harvest; no push-update job
withdrawal support:             removal from the feed reflects current
                                state; no push withdrawal
retry/idempotency:              destination-controlled monthly harvest;
                                persistent publisher-level URLs
                                (LEDGER:KBART-02)
location writeback:             none
required credential category:   no machine credential (public feed URL)
current credential
responsibility:                 not applicable - no machine credential;
                                onboarding/notification staff-managed
target credential owner:        Metadata Specialist (shared record; for the
                                onboarding communication responsibility)
commercial entitlement
authority:                      publisher Statement of Work (shared record)
runtime desired-state
authority:                      Thoth publisher/platform assignment
temporary operational
configuration source:           Ex Libris onboarding records and collection
                                identifiers (source-owner domain;
                                LEDGER:PROQUEST-01)
current publisher-config
source:                         Ex Libris onboarded-subset records (no
                                repository list; contents not read)
accountable operational owner:  COO (shared record)
operationally responsible role: Metadata Specialist (shared record)
repository references:          thoth@32123d3:
                                thoth-export-server/src/data.rs
                                (proquest_exlibris -> kbart::oclc)
read-only verification method:  file inspection at recorded commit; ledger
                                attribution for harvest and onboarding
evidence classification:        source-owner-confirmed (current harvest,
                                separate onboarding: LEDGER:PROQUEST-01,
                                KBART-02, KBART-03; separation decision:
                                CTO-2026-08-06); repository-verified
                                (registry association to the shared KBART
                                specification)
open questions:                 none blocking
known defects:                  none current
migration/backfill:             assignments backfilled from Ex Libris
                                onboarding records; creates no feed job
final decision:                 INCLUDED - PullFeed, assignable, shared
                                OCLC_KBART_PUBLIC
```

### 6.17 JISC_NBK

```text
stable proposed enum code:      JISC_NBK
display name:                   Jisc NBK
aliases and legacy names:       "Jisc KB" / "JISC KB" (superseded label;
                                LocationPlatform "JISC KB"; export registry
                                "jisc_kb"; app label "JISC"). The registry's
                                historical jisc_kb -> kbart::oclc
                                association is NOT the current mechanism
                                (LEDGER:JISC-01 supersedes; HIST-01 is
                                superseded OBP terminology).
independently selectable:       yes as an independently meaningful
                                destination (LEDGER:JISC-01, JISC-02;
                                CTO-2026-08-06) - but initially inactive
current mechanism:              agreed MARC21 (.mrc per book) transfer to a
                                Jisc S3 bucket; NOT the OCLC KBART feed; no
                                dissemination adapter implemented
behaviour:                      AutomaticPush class when active; inactive
                                initially
linked platform group:          none
shared adapter/feed profile:    JISC_NBK_MARC_S3 (proposed identity;
                                explicitly inactive; not implemented)
mechanism readiness:            INACTIVE - no adapter, onboarding controls
                                or operational evidence implemented/approved
assignment availability:        NON-ASSIGNABLE until a separately approved
                                implementation task delivers the MARC/S3
                                adapter, onboarding controls, operational
                                evidence, failure handling, tests and
                                rollout controls
duplicate-delivery risk:        none while inactive - creates no job or
                                delivery
back-catalogue support:         expected initial full back-catalogue export
                                on activation (workflow agreed with Jisc:
                                initial delivery, new books, overwriting
                                changed records - LEDGER:JISC-01); none
                                while inactive
accepted work types:            per agreed record structure (one MARC21
                                record per book); none processed while
                                inactive
required file/metadata formats: MARC21 .mrc files (one per book)
incremental update support:     none while inactive; agreed future workflow
                                includes overwriting changed records,
                                subject to separate approval
withdrawal support:             none; unverified remotely; conservative
                                policy applies
retry/idempotency:              not applicable while inactive
location writeback:             none
required credential category:   manual staff-managed access currently;
                                future S3/cloud storage credential category
                                for the adapter when separately approved
current credential
responsibility:                 staff-managed (shared record)
target credential owner:        Metadata Specialist (shared record)
commercial entitlement
authority:                      publisher Statement of Work (shared record;
                                SLA name "Jisc NBK" - LEDGER:JISC-02)
runtime desired-state
authority:                      Thoth publisher/platform assignment (none
                                permitted while non-assignable)
temporary operational
configuration source:           none (no workflow, no publisher list, no
                                uploader configuration exists)
current publisher-config
source:                         none - no current operational configuration
accountable operational owner:  COO (shared record)
operationally responsible role: Metadata Specialist (shared record)
repository references:          thoth@32123d3:
                                thoth-export-server/src/data.rs (historical
                                jisc_kb registry entry);
                                thoth-dissemination@7a16edc0: absence of any
                                Jisc uploader/workflow (verified)
read-only verification method:  file inspection at recorded commits; ledger
                                attribution for the NBK/S3 arrangement
evidence classification:        source-owner-confirmed (destination,
                                mechanism, naming: LEDGER:JISC-01, JISC-02;
                                inactive/non-assignable decision:
                                CTO-2026-08-06); repository-verified
                                (absence of implementation)
open questions:                 completion of the initial production load
                                after Jisc's 2026 system migration remains
                                unconfirmed (LEDGER gap 5) - does not block
                                inclusion because the destination is
                                inactive, non-assignable and job-free; it
                                blocks any future activation task instead
known defects:                  none current (nothing is implemented)
migration/backfill:             no assignments created by MIG-01;
                                activation requires its own approved task
final decision:                 INCLUDED - inactive, non-assignable,
                                job-free pending separately approved
                                JISC_NBK_MARC_S3 implementation
```

## 7. Excluded candidate records

Exclusion from `DistributionPlatform` deletes no existing export-registry or
`LocationPlatform` value. For excluded candidates, fields that cannot be
established are recorded as `unverified`; that is the evidence-based reason
for exclusion, and the `unknown`-blocks-approval rule applies only to
included values.

### 7.1 EBSCO_KB

```text
candidate code:                 EBSCO_KB
display name (if included):     EBSCO Knowledge Base
aliases and legacy names:       LocationPlatform "EBSCO KB"; export registry
                                "ebsco_kb"; app label "EBSCO Knowledge Base"
distinct product:               yes - EBSCO KB and EBSCOHost are separate
                                EBSCO systems, not aliases (LEDGER:EBSCO-01)
historical route:               historically separate metadata-only FTP
                                destination (LEDGER:EBSCO-04)
current operation:              unverified - no current independently
                                selectable workflow, no current SLA
                                selection, no endpoint, no publisher
                                configuration, no uploader key
                                (LEDGER:EBSCO-05; disseminator.py registry;
                                config structure)
shared-feed inference:          prohibited - must not be inferred to consume
                                OCLC_KBART_PUBLIC (settled input 13.4)
evidence classification:        source-owner-confirmed (distinctness,
                                historical route); repository-verified
                                (absence of current mechanism); current
                                selectable status unresolved
final decision:                 EXCLUDED from the initial enum - distinct
                                product whose current Thoth-selectable
                                operation is unverified; historical evidence
                                must not be represented as current;
                                revisitable through a later approved decision
```

### 7.2 PROQUEST_SERIALS_SOLUTIONS_KB

```text
candidate code:                 PROQUEST_SERIALS_SOLUTIONS_KB
display name (if included):     ProQuest Serials Solutions Knowledge Base
aliases and legacy names:       "ProQuest KB - was Serial Solutions"
                                (LEDGER:PROQUEST-04); LocationPlatform
                                "ProQuest KB"; export registry "proquest_kb";
                                app label "ProQuest Knowledge Base". Not an
                                alias of Ebook Central or Ex Libris.
distinct product:               yes - historically distinct route
                                (LEDGER:PROQUEST-04, PROQUEST-05)
current operation:              unverified - no current workflow, endpoint,
                                publisher coverage or selectable status
                                (LEDGER:PROQUEST-01, PROQUEST-06;
                                disseminator.py registry; config structure)
shared-feed inference:          prohibited - must not be inferred to consume
                                OCLC_KBART_PUBLIC (settled input 13.4)
evidence classification:        source-owner-confirmed (historical
                                distinctness); repository-verified (absence
                                of current mechanism); current status
                                unresolved
final decision:                 EXCLUDED from the initial enum - historically
                                distinct, currently unverified; revisitable
                                through a later approved decision
```

### 7.3 OVERDRIVE

```text
candidate code:                 OVERDRIVE
evidence:                       export registry platform "overdrive" with
                                onix_3.0::overdrive specification
                                (thoth@32123d3 data.rs); no uploader key, no
                                workflow, no configuration, no ledger
                                evidence of current operation
evidence classification:        repository-verified (registry existence;
                                absence of mechanism); current operation
                                unverified
final decision:                 EXCLUDED by CTO decision (2026-08-06,
                                specification section 8.6); export-registry
                                entry retained
```

### 7.4 BDS_LIVE

```text
candidate code:                 BDS_LIVE
evidence:                       export registry platform "bds_live"
                                accepting onix_3.0::jstor (thoth@32123d3
                                data.rs); no uploader key, no workflow, no
                                configuration, no ledger evidence of current
                                operation
evidence classification:        repository-verified (registry existence;
                                absence of mechanism); current operation
                                unverified
final decision:                 EXCLUDED by CTO decision (2026-08-06);
                                export-registry entry retained
```

### 7.5 RNIB_BOOKSHARE

```text
candidate code:                 RNIB_BOOKSHARE
evidence:                       export registry platform "rnib_bookshare"
                                accepting onix_2.1::ebsco_host
                                (thoth@32123d3 data.rs); no uploader key, no
                                workflow, no configuration, no ledger
                                evidence of current operation
evidence classification:        repository-verified (registry existence;
                                absence of mechanism); current operation
                                unverified
final decision:                 EXCLUDED by CTO decision (2026-08-06);
                                export-registry entry retained
```

### 7.6 SCIELO_BOOKS

```text
candidate code:                 SCIELO_BOOKS
evidence:                       LocationPlatform "SciELO Books"
                                (thoth@32123d3 location model); no export
                                registry platform, no uploader, no workflow,
                                no configuration
evidence classification:        repository-verified (location value
                                existence; absence of mechanism); current
                                operation unverified
final decision:                 EXCLUDED by CTO decision (2026-08-06);
                                LocationPlatform value retained
```

### 7.7 ZOTERO

```text
candidate code:                 ZOTERO
evidence:                       export registry platform "zotero" accepting
                                bibtex::thoth (thoth@32123d3 data.rs) - an
                                end-user reference-manager export target,
                                not a publisher distribution destination; no
                                uploader, no workflow, no configuration
evidence classification:        repository-verified
final decision:                 EXCLUDED by CTO decision (2026-08-06);
                                export-registry entry retained
```

### 7.8 THOTH

```text
candidate code:                 THOTH
evidence:                       LocationPlatform "Thoth" (publisher CDN
                                hosted by Thoth) and export registry
                                platform "thoth" (generic Thoth-flavour
                                exports) at thoth@32123d3
final decision:                 EXCLUDED - internal managed-location and
                                file-hosting concept, not a distribution
                                destination; location and registry values
                                retained; central to the Thoth-managed
                                source-file invariant (ADR-0004 section 4.6)
evidence classification:        repository-verified
```

### 7.9 PUBLISHER_WEBSITE

```text
candidate code:                 PUBLISHER_WEBSITE
evidence:                       LocationPlatform "Publisher Website" at
                                thoth@32123d3 - a publisher-managed hosting
                                location; uploader.py currently selects
                                canonical fullTextUrl regardless of
                                platform, which the Thoth-managed
                                source-file invariant will later exclude as
                                an automatic-dissemination source
final decision:                 EXCLUDED - publisher-managed location, not a
                                Thoth-operated delivery destination;
                                location value retained; not an eligible
                                automatic-dissemination source under the
                                recorded invariant
evidence classification:        repository-verified
```

### 7.10 OTHER

```text
candidate code:                 OTHER
evidence:                       LocationPlatform default value "Other" at
                                thoth@32123d3; culuploader.py writes CUL
                                locations under 'OTHER'
final decision:                 EXCLUDED - prohibited by existing
                                architecture; no OTHER, catch-all or
                                fallback DistributionPlatform value may
                                exist (settled invariant 3); the
                                LocationPlatform default is unaffected
evidence classification:        repository-verified
```

## 8. Evidence-classification counts

Counting the discrete claims in the claim-to-source index (section 9), each
carrying exactly one classification. The counts were re-derived from the
corrected claim index after the review-4876054508 remediation (which
rewrote claim R4 to record credential configuration categories and scope
without exact identifiers): the remediation changed R4's wording, not the
set of claims, so the totals below remain those of the corrected index:

```text
repository-verified claims:       34
source-owner-confirmed claims:    21
production-verified claims:        0  (no production evidence supplied; no
                                       production access performed)
provisional claims at completion:  0  (none included; none remaining)
unknown claims at completion:      0  in included values
```

`unverified` current-operation status appears only inside excluded-candidate
records (EBSCO_KB, PROQUEST_SERIALS_SOLUTIONS_KB, OVERDRIVE, BDS_LIVE,
RNIB_BOOKSHARE, SCIELO_BOOKS), where unresolved status supports exclusion,
never inclusion. No included destination retains an `unknown` or
`provisional` field required to implement BE-02 safely.

## 9. Claim-to-source index

Repository-verified claims (exact commit and path recorded above):

| # | Claim | Source |
|---|---|---|
| R1 | 13 push uploader dispatch keys | thoth-dissemination@7a16edc0 disseminator.py |
| R2 | Scheduled/manual workflow entry points and schedules | thoth-dissemination@7a16edc0 .github/workflows/ |
| R3 | Publisher-list configuration structure (<PREFIX>_ENV_PUBLISHERS/_EXCEPTIONS) | thoth-dissemination@7a16edc0 workflow files, bulk_disseminate.yml |
| R4 | Credential configuration categories and their global or per-publisher scope (exact secret and credential identifiers intentionally omitted) | thoth-dissemination@7a16edc0 config.env.template |
| R5 | Canonical-location source selection (no Thoth-managed restriction yet) | thoth-dissemination@7a16edc0 uploader.py get_publication_source |
| R6 | IA convergence/idempotency behaviour | thoth-dissemination@7a16edc0 iauploader.py; v1.6.1-v1.6.4 history |
| R7 | Figshare existing-record guard | thoth-dissemination@7a16edc0 fsuploader.py |
| R8 | Zenodo existing-record guard | thoth-dissemination@7a16edc0 zenodouploader.py |
| R9 | Crossref HTTPS DOI deposit mechanism | thoth-dissemination@7a16edc0 crossrefuploader.py |
| R10 | CUL location writeback under 'OTHER' | thoth-dissemination@7a16edc0 culuploader.py:50 |
| R11 | OAPEN/MUSE deferred location catch-up | thoth-dissemination@7a16edc0 catchup workflows and obtain_*_locations.py |
| R12 | IA immediate location writeback | thoth-dissemination@7a16edc0 write_locations.py, reconcile_internet_archive.py |
| R13 | Bulk work-type selection set | thoth-dissemination@7a16edc0 obtain_new_ids.py |
| R14 | Generic SWORDv2/DSpace shared implementation base | thoth-dissemination@7a16edc0 swordv2uploader.py, dspaceuploader.py |
| R15 | EBSCOHost uploader requires PDF/EPUB + cover + onix_2.1::ebsco_host, SFTP | thoth-dissemination@7a16edc0 ebscouploader.py (= LEDGER:EBSCO-03) |
| R16 | ProQuest uploader is Ebook Central; flavour onix_2.1::proquest_ebrary | thoth-dissemination@7a16edc0 proquestuploader.py (= LEDGER:PROQUEST-02) |
| R17 | ProQuest current EPUB-only/PDF-ISBN ordering defect | thoth-dissemination@7a16edc0 proquestuploader.py:43 |
| R18 | Project MUSE workflow key matches registry; mismatch fixed 2026-04-23 | thoth-dissemination@7a16edc0 muse_bulk_disseminate.yaml; commit 1a66da8f |
| R19 | No uploader/workflow for EBSCO KB, ProQuest KB/Serial Solutions, Ex Libris, Jisc, OCLC, DOAB, OverDrive, BDS Live, RNIB, SciELO, Zotero | thoth-dissemination@7a16edc0 disseminator.py, workflows |
| R20 | Registry drift 5e88ce1b..7a16edc0 is IA-only hardening | thoth-dissemination git history |
| R21 | LocationPlatform 18 values incl. Thoth, Publisher Website, Other, JISC KB, Google Books | thoth@32123d3 thoth-api/src/model/location/mod.rs |
| R22 | Export registry platforms/specifications/formats | thoth@32123d3 thoth-export-server/src/data.rs |
| R23 | One kbart::oclc specification accepted by five registry KB platforms | thoth@32123d3 thoth-export-server/src/data.rs |
| R24 | OCLC KBART output includes oclc_number from work.oclc | thoth@32123d3 kbart_oclc.rs; thoth-api/src/model/work/mod.rs |
| R25 | Thoth file-record and location model exist (invariant substrate) | thoth@32123d3 thoth-api/src/model/file/, location/ |
| R26 | thoth workflows and documentation-only CI classification | thoth@32123d3 .github/workflows/ |
| R27 | Deferred OAI branch adds thoth-oai-server (context only; unmodified) | thoth feature/oai-pmh-http@745dd020 |
| R28 | App hard-codes platform labels/option lists (duplication evidence) | thoth-app@6f826390 src/shared/utils/locations/index.ts, src/shared/constants/formFields.ts |
| R29 | App codegen uses unpinned shared test API | thoth-app@6f826390 codegen.ts |
| R30 | App branch topology main/dev; CI test.yml only | thoth-app@6f826390 / dev@26323158 |
| R31 | main/dev identical for inspected platform-relevant paths | thoth-app diff (empty) |
| R32 | ScienceOpen has uploader + credentials but no schedule | thoth-dissemination@7a16edc0 souploader.py, workflows |
| R33 | onix_3.0::oapen accepted by oapen and doab; no separate DOAB uploader | thoth@32123d3 data.rs; thoth-dissemination@7a16edc0 |
| R34 | Google Play delivery uses bucket + per-publisher collection codes | thoth-dissemination@7a16edc0 googleplayuploader.py, config.env.template |

Source-owner-confirmed claims (attribution and date recorded):

| # | Claim | Source |
|---|---|---|
| S1 | EBSCO KB and EBSCOHost are distinct systems | LEDGER:EBSCO-01 (meeting 2021-09-24; recorded 2021-12-03) |
| S2 | Current EBSCOHost workflow (ONIX 2.1 + files via SFTP) | LEDGER:EBSCO-02 (HH, updated 2024-10-04) |
| S3 | Historical separate EBSCO KB FTP route | LEDGER:EBSCO-04 (2023-08-07) |
| S4 | SLA selectable list has EBSCOHost, no EBSCO KB | LEDGER:EBSCO-05 (SLA template, 2025-09) |
| S5 | Ebook Central vs Ex Libris workflows differ | LEDGER:PROQUEST-01 (HH, updated 2025-05-15) |
| S6 | Historical ProQuest naming relationships (Ebrary, ExLibris, Serial Solutions) | LEDGER:PROQUEST-04 (2023-08-07), PROQUEST-05 (2023-12-04) |
| S7 | SLA label "ProQuest" is ambiguous but current usage means Ebook Central | LEDGER:PROQUEST-06 (SLA template, 2025-09) with PROQUEST-02/03 |
| S8 | OCLC monthly harvest, per-publisher collection codes, onboarding | LEDGER:KBART-01 (updated 2025-05-13) |
| S9 | Ex Libris harvests the same publisher-level OCLC KBART outputs | LEDGER:KBART-02 (correspondence 2025-07-08..11: Hannah Hillen, Ross Higman, Javier Arias) |
| S10 | Ex Libris uses the OCLC KBART export; no separate flavour | LEDGER:KBART-03 (HH, updated 2025-05-15) |
| S11 | Jisc NBK MARC21/S3 arrangement, record structure, update workflow | LEDGER:JISC-01 (thread 2025-05-21..2026-03-25: Javier Arias, Toby Steiner, Jisc NBK team incl. Bethan Ruddock) |
| S12 | Current contractual name "Jisc NBK" | LEDGER:JISC-02 (SLA template, 2025-09) |
| S13 | Older OBP knowledge-base terminology (superseded) | LEDGER:HIST-01 (2020-01-15) |
| S14 | Destination-vs-adapter architecture requirements | LEDGER:ADR-01-SOURCE (design doc, 2026-07-23) |
| S15 | EX_LIBRIS_KB separate destination; shared feed; no duplicate job | CTO-2026-08-06 (ledger section 9, item 1) |
| S16 | JISC_NBK included but inactive and non-assignable | CTO-2026-08-06 (ledger section 9, item 2) |
| S17 | No known current manual-only destinations outside inspected repos | CTO-2026-08-06 (ledger section 9, item 3; not a permanent claim) |
| S18 | Conservative initial update/withdrawal policy | CTO-2026-08-06 (ledger section 9, item 4) |
| S19 | Google Books and Google Play Books are one destination, display "Google Play Books" | CTO-2026-08-06 (specification section 8.1) |
| S20 | Initial exclusions OVERDRIVE, BDS_LIVE, RNIB_BOOKSHARE, SCIELO_BOOKS, ZOTERO; THOTH/PUBLISHER_WEBSITE remain location concepts | CTO-2026-08-06 (specification section 8.6) |
| S21 | Ownership/configuration authority (SoW, Thoth assignment, COO, Metadata Specialist) | CTO-2026-08-06 (specification section 9.4) |

## 10. Separation of evidence and decision

Sections 2 through 9 of this matrix are evidence. The final dispositions in
sections 3, 6 and 7 restate the decisions proposed by
[ADR-0004](../engineering/decisions/ADR-0004-distribution-platform-inventory.md),
which is the decision record. ADR-0004 is `PROPOSED`; nothing in this matrix
approves it. The sanitized source ledger
[adr-01-evidence-ledger.md](adr-01-evidence-ledger.md) is unchanged by
ADR-01, and its section 0.0 provenance boundary (original 18 hash-covered
entries versus the separately attributable section 9 CTO decisions) is
preserved by the citation scheme in section 1 above.
