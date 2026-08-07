# Distribution Platform Inventory

Status: FINAL INVENTORY APPROVED AND REPOSITORY-AUTHORITATIVE
Approved content head: `44e6f821535fbee56c830dd6eda237fc6d06fbfd`
Independent review: `4881233664` (`APPROVED`)
CTO approval: `4881279067`
Approval date: 2026-08-07
Repository authority: established through ADR-01 implementation PR
[#783](https://github.com/thoth-pub/thoth/pull/783), merge commit
`299b0eff3b9ac10cc0a3a7024ab311ddb135b7eb`, merged 2026-08-07T10:02:34Z
Inventory owner: Publisher Services ADR-01
Decision record: [ADR-0004](../engineering/decisions/ADR-0004-distribution-platform-inventory.md) (APPROVED AND REPOSITORY-AUTHORITATIVE)
Evidence: [ADR-01 evidence matrix](adr-01-evidence-matrix.md);
[ADR-01 evidence ledger](adr-01-evidence-ledger.md) (unchanged)
Evidence date: 2026-08-06
Repository evidence: `thoth` at `32123d363a6806d377ac322e3814fb432a803453`;
`thoth-dissemination` at `7a16edc08d4570f3ecc108453298a3aa43f6d753`
(release 1.6.4; supersedes the provisional baseline reference
`5e88ce1b58e5f962cc4f4ef6fb00c08f50b57add` - drift recorded in the matrix);
`thoth-app` at `main` `6f826390a07efe6266cfda2b4af1f85b6cbfc38a` / `dev`
`26323158f1145b35eff27bce6f901ff0eb78280a`

## 1. Purpose

This file records the final approved user-visible `DistributionPlatform`
inventory produced by the ADR-01 implementation, replacing the earlier
provisional baseline. It is exhaustive, contains no unsupported value and no
`OTHER`, distinguishes every destination from the adapter or feed profile
serving it, and identifies inactive/non-assignable destinations.

The inventory content was independently reviewed at exact head
`44e6f821535fbee56c830dd6eda237fc6d06fbfd` (review `4881233664`,
`APPROVED`) and explicitly CTO-approved (review `4881279067`, 2026-08-07)
together with ADR-0004. The approved inventory is exactly that reviewed
content: 17 included destinations, 10 recorded exclusions, no `OTHER`, no
fallback, no unknown or provisional included value, shared adapter/feed
relationships preserved, `JISC_NBK` included but inactive and
non-assignable, `OCLC_KB` and `EX_LIBRIS_KB` sharing `OCLC_KBART_PUBLIC`,
OAPEN/DOAB linked duplicate-safe behaviour, the conservative
update/withdrawal policy, and the Thoth-managed source-file invariant
recorded but not implemented.

The approval is a content approval bound to that exact head, and it became
repository-authoritative when ADR-01 implementation PR
[#783](https://github.com/thoth-pub/thoth/pull/783) merged into `develop` as
`299b0eff3b9ac10cc0a3a7024ab311ddb135b7eb` on 2026-08-07T10:02:34Z. No
`DistributionPlatform` enum is implemented from this inventory: `BE-02`'s
ADR-01 dependency is satisfied, but `BE-02` remains blocked and unauthorized
pending its own approved bounded specification and explicit implementation
authorization.

## 2. Behaviour vocabulary

- `AutomaticPush` - enabling a new publisher assignment may require a durable
  back-catalogue upload job (jobs are BE-04 scope; automatic job creation is
  initially inactive).
- `PullFeed` - the destination retrieves a Thoth feed; assignment records
  feed membership/onboarding; no uploader job is created.
- `Manual` - staff action or destination-specific coordination is required;
  no automatic job is created.

## 3. Final proposed inventory (17 values)

| Enum code | Display label | Behaviour | Linked group | Adapter / feed profile | Assignment availability | Current mechanism at inspected commits | Current publisher-config source (mirror) |
|---|---|---|---|---|---|---|---|
| `INTERNET_ARCHIVE` | Internet Archive | AutomaticPush | - | `IA_API` | assignable | uploader `InternetArchive`, daily schedule, immediate location writeback | `IA_ENV_PUBLISHERS` |
| `OAPEN` | OAPEN | AutomaticPush | `OAPEN_DOAB` | `OAPEN_DOAB_SWORD` (shared) | assignable (linked selection) | uploader `OAPEN` (SWORDv2), weekly schedule, deferred location catch-up | `OAPEN_ENV_PUBLISHERS` |
| `DOAB` | DOAB | AutomaticPush (via linked deposit) | `OAPEN_DOAB` | `OAPEN_DOAB_SWORD` (shared) | assignable (linked selection) | no separate uploader; DOAB ingests from the linked OAPEN deposit | linked with OAPEN |
| `SCIENCE_OPEN` | ScienceOpen | Manual | - | `SCIENCEOPEN_FTP` | assignable (job-free) | uploader `ScienceOpen`, manual invocation only, no schedule | manual staff selection |
| `CAMBRIDGE_UNIVERSITY_LIBRARY` | Cambridge University Library | AutomaticPush | - | `CUL_SWORD` | assignable | uploader `CUL` (SWORDv2), monthly schedule, writeback under `OTHER` location | `CUL_ENV_PUBLISHERS` |
| `CROSSREF` | Crossref | AutomaticPush | - | `CROSSREF_DOI_DEPOSIT` | assignable | uploader `Crossref`, hourly schedule, metadata-only DOI deposit | `CR_ENV_PUBLISHERS` |
| `FIGSHARE` | Figshare | AutomaticPush | - | `FIGSHARE_API` | assignable | uploader `Figshare`, monthly schedule, existing-record guard, immediate writeback | `FS_ENV_PUBLISHERS` |
| `ZENODO` | Zenodo | AutomaticPush | - | `ZENODO_API` | assignable | uploader `Zenodo`, monthly schedule, existing-record guard, immediate writeback | `ZN_ENV_PUBLISHERS` |
| `PROJECT_MUSE` | Project MUSE | AutomaticPush | - | `MUSE_FTP` | assignable | uploader `ProjectMUSE`, weekly schedule, deferred location catch-up | `MUSE_ENV_PUBLISHERS` |
| `JSTOR` | JSTOR | AutomaticPush | - | `JSTOR_FTP` | assignable | uploader `JSTOR`, weekly schedule, per-publisher folder | `JSTOR_ENV_PUBLISHERS` |
| `EBSCO_HOST` | EBSCOHost | AutomaticPush | - | `EBSCO_HOST_SFTP` | assignable | uploader `EBSCOHost`, weekly schedule, ONIX 2.1 + content via SFTP | `EH_ENV_PUBLISHERS` |
| `PROQUEST_EBOOK_CENTRAL` | ProQuest Ebook Central | AutomaticPush | - | `PROQUEST_EBOOK_CENTRAL_FTP` | assignable | uploader `ProQuest`, weekly schedule; current defect recorded (section 5) | `PQ_ENV_PUBLISHERS` |
| `GOOGLE_PLAY` | Google Play Books | AutomaticPush | - | `GOOGLE_PLAY_GCS` | assignable | uploader `GooglePlay`, daily schedule, bucket + per-publisher collection code | `GP_ENV_PUBLISHERS` |
| `BKCI` | Book Citation Index | AutomaticPush | - | `BKCI_FTP` | assignable | uploader `BKCI`, monthly schedule | `BKCI_ENV_PUBLISHERS` |
| `OCLC_KB` | OCLC Knowledge Base | PullFeed | - | `OCLC_KBART_PUBLIC` (shared) | assignable (feed membership; job-free) | monthly OCLC harvest of publisher-level `kbart::oclc` output | OCLC onboarding records |
| `EX_LIBRIS_KB` | Ex Libris Knowledge Base | PullFeed | - | `OCLC_KBART_PUBLIC` (shared) | assignable (feed membership; job-free) | monthly Ex Libris harvest of the same publisher-level output for an independently onboarded subset | Ex Libris onboarding records |
| `JISC_NBK` | Jisc NBK | AutomaticPush class when active | - | `JISC_NBK_MARC_S3` (**inactive**) | **non-assignable; inactive; job-free** | agreed MARC21-via-S3 workflow; no adapter implemented; creates no job or delivery | none |

Aliases and legacy names resolve to these stable codes as recorded in
[ADR-0004 section 4.2](../engineering/decisions/ADR-0004-distribution-platform-inventory.md);
notably `Google Books` -> `GOOGLE_PLAY`; `ProQuest`/`Ebrary`/
`proquest_ebrary` -> `PROQUEST_EBOOK_CENTRAL`; `ProQuest ExLibris` ->
`EX_LIBRIS_KB`; `JISC KB` -> `JISC_NBK`.

## 4. Duplicate-safety semantics

1. `OAPEN` + `DOAB` are separate values linked for initial selection;
   backend-owned normalization; one linked activation produces exactly one
   logical delivery through `OAPEN_DOAB_SWORD`; never two uploads.
2. `OCLC_KB` + `EX_LIBRIS_KB` are independently onboarded consumer
   assignments sharing one publisher-level `OCLC_KBART_PUBLIC` feed; no
   duplicate KBART outputs, no duplicate feed state, no uploader jobs. EBSCO
   KB and Serial Solutions must not be inferred to consume this feed.
3. `JISC_NBK` is included but inactive: non-assignable and creating no job
   or delivery until the separately approved `JISC_NBK_MARC_S3`
   implementation task delivers the adapter, onboarding controls,
   operational evidence, failure handling, tests and rollout controls.
4. Pull-feed and manual destinations never create uploader jobs; empty
   assignments never broaden processing; configuration failure fails closed.
5. Automatic push jobs may use only Thoth-managed publication files
   (ADR-0004 section 4.6); Publisher Website locations and arbitrary
   external URLs are not eligible sources; enforcement is a separate future
   HIGH-risk task.

## 5. Current operational caveats

1. Current defect, preserved: the ProQuest uploader's intended EPUB-only
   fallback can fail because it retrieves a PDF ISBN first
   (`proquestuploader.py` at `7a16edc0`, filename root set from
   `get_isbn('PDF')` before the PDF/EPUB fallback). ProQuest scheduled
   delivery must not be presented as fully healthy.
2. Historical/resolved, not current: the Project MUSE scheduled-workflow key
   mismatch was fixed by `1a66da8f1700d8c76bf8fda2938b8729be0a93b6`
   (23 April 2026); workflow and registry match at the inspected commit.
3. Known current behaviour: CUL location writeback records
   `LocationPlatform` `'OTHER'`; revisiting this is future implementation
   work.
4. Current dissemination source selection uses each publication's canonical
   location URL regardless of hosting platform; the Thoth-managed
   source-file invariant is recorded, not yet enforced.

## 6. Excluded candidates

Recorded with individual reasons in
[ADR-0004 section 4.8](../engineering/decisions/ADR-0004-distribution-platform-inventory.md)
and full records in the
[evidence matrix](adr-01-evidence-matrix.md):

```text
EBSCO_KB                       distinct product; current operation unverified
PROQUEST_SERIALS_SOLUTIONS_KB  historically distinct; current operation unverified
OVERDRIVE                      registry entry only; excluded by CTO decision
BDS_LIVE                       registry entry only; excluded by CTO decision
RNIB_BOOKSHARE                 registry entry only; excluded by CTO decision
SCIELO_BOOKS                   location value only; excluded by CTO decision
ZOTERO                         end-user export target, not a distribution destination
THOTH                          internal managed-location/file-hosting concept
PUBLISHER_WEBSITE              publisher-managed location, not a delivery destination
OTHER                          prohibited by architecture
```

No exclusion deletes an existing export-registry or `LocationPlatform`
value.

## 7. Safety rule

ADR-0004 has been independently reviewed, explicitly CTO-approved and merged,
so this inventory is final and repository-authoritative. It must still not be
converted into production rows, jobs or an implemented enum outside a
separately approved and authorized implementation task. `MIG-01` must use an
approved mapping, dry run and no-job import mode. `BE-02`'s ADR-01 dependency
is satisfied, and `BE-02` remains blocked and unauthorized pending its own
approved bounded specification and explicit implementation authorization.
