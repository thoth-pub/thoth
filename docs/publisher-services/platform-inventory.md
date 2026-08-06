# Distribution Platform Inventory

Status: VERIFIED BASELINE; FINAL ENUM NOT APPROVED
Inventory owner: Publisher Services ADR-01
Evidence date: 2026-07-24
Observed `thoth-dissemination` reference: commit `5e88ce1b58e5f962cc4f4ef6fb00c08f50b57add`

## 1. Purpose

This file records the current operational baseline for ADR-01.

It distinguishes:

- verified uploader/workflow behaviour;
- proposed user-visible `DistributionPlatform` values;
- unresolved inventory decisions.

It must not be used as the final enum until ADR-01 is independently reviewed and approved.

## 2. Behaviour vocabulary

- `AutomaticPush` - enabling a new publisher assignment may require a durable back-catalogue upload job.
- `PullFeed` - the destination retrieves a Thoth feed; no uploader job is created.
- `Manual` - staff action or destination-specific coordination is required; no automatic job is created.

This classification describes desired Publisher Services behaviour, not merely the existence of a current schedule.

## 3. Verified current uploader baseline

Current `disseminator.py` exposes these uploader keys:

```text
InternetArchive
OAPEN
ScienceOpen
CUL
Crossref
Figshare
Zenodo
ProjectMUSE
JSTOR
EBSCOHost
ProQuest
GooglePlay
BKCI
```

`DSPACE` is not a separate current user-visible uploader key; generic DSpace/SWORD support is an implementation mechanism.

## 4. Candidate destination inventory

| Proposed enum | Display name | Current key/mechanism | Proposed behaviour | Linked group | Current schedule | Location handling | Update support | Withdrawal support | Current publisher source | Status |
|---|---|---|---|---|---|---|---|---|---|---|
| `INTERNET_ARCHIVE` | Internet Archive | `InternetArchive` | AutomaticPush | - | daily | immediate Thoth writeback | verified idempotent update for owned items | unverified | `IA_ENV_PUBLISHERS` | VERIFIED CANDIDATE |
| `OAPEN` | OAPEN | `OAPEN` SWORD/metadata delivery | AutomaticPush | `OAPEN_DOAB` | weekly | deferred catch-up | needs ADR verification | unverified | platform publisher list/environment | VERIFIED CANDIDATE |
| `DOAB` | DOAB | linked logical target; no separate uploader | AutomaticPush | `OAPEN_DOAB` | follows OAPEN | deferred catch-up | linked semantics require verification | unverified | same linked configuration | VERIFIED LOGICAL CANDIDATE |
| `SCIENCE_OPEN` | ScienceOpen | `ScienceOpen` | Manual | - | manual | no verified automatic writeback | unverified | unverified | manual eligibility/configuration | VERIFIED CANDIDATE |
| `CAMBRIDGE_UNIVERSITY_LIBRARY` | Cambridge University Library | `CUL` SWORDv2 | AutomaticPush | - | monthly | immediate `OTHER` writeback | unverified | unverified | environment publisher list | VERIFIED CANDIDATE |
| `CROSSREF` | Crossref | `Crossref` HTTPS metadata deposit | AutomaticPush | - | hourly | none | likely repeat deposit; exact semantics require verification | unverified | environment list plus per-publisher credentials | VERIFIED CANDIDATE |
| `FIGSHARE` | Figshare | `Figshare` API | AutomaticPush | - | monthly | immediate per-publication writeback | current existing-record guard blocks simple repeat; clarify desired updates | unverified | environment publisher list | VERIFIED CANDIDATE |
| `ZENODO` | Zenodo | `Zenodo` API | AutomaticPush | - | monthly | immediate per-publication writeback | current existing-record guard blocks simple repeat; clarify desired updates | unverified | environment publisher list | VERIFIED CANDIDATE |
| `PROJECT_MUSE` | Project MUSE | `ProjectMUSE` FTP | AutomaticPush | - | weekly | none; notification flow | unverified | unverified | environment list plus per-publisher credentials | VERIFIED CANDIDATE; historical key mismatch resolved (see section 5) |
| `JSTOR` | JSTOR | `JSTOR` FTP | AutomaticPush | - | weekly | none; notification flow | unverified | unverified | environment list plus per-publisher folder | VERIFIED CANDIDATE |
| `EBSCO_HOST` | EBSCOHost | `EBSCOHost` FTP | AutomaticPush | - | weekly | none; notification flow | unverified | unverified | environment publisher list | VERIFIED CANDIDATE |
| `PROQUEST` | ProQuest | `ProQuest` FTP/ebrary format | AutomaticPush | - | weekly | none; notification flow | unverified | unverified | environment publisher list | VERIFIED CANDIDATE; PRODUCT SCOPE OPEN |
| `GOOGLE_PLAY` | Google Play Books | `GooglePlay` bucket delivery | AutomaticPush | - | daily | none | unverified | unverified | environment list plus per-publisher collection code | VERIFIED CANDIDATE |
| `BKCI` | Book Citation Index | `BKCI` FTP | AutomaticPush | - | monthly | none; notification flow | unverified | unverified | environment list plus per-publisher credentials | VERIFIED CANDIDATE |
| `OCLC_KB` | OCLC Knowledge Base | Thoth KBART pull-feed/index | PullFeed | - | destination pull | feed membership, no location writeback | feed reflects current state | feed removal reflects current state | existing OCLC arrangements/records | DESIGN CANDIDATE; VERIFY OPERATIONS |

## 5. Current operational caveats

1. Historical/resolved Project MUSE key mismatch; not a current defect. At
   `thoth-dissemination` release commit
   `7a16edc08d4570f3ecc108453298a3aa43f6d753` the scheduled workflow passes
   `platform: 'ProjectMUSE'` and dispatch accepts `ProjectMUSE`; the same
   matching state existed at the provisional baseline
   `5e88ce1b58e5f962cc4f4ef6fb00c08f50b57add`. The historical mismatch was
   fixed by commit `1a66da8f1700d8c76bf8fda2938b8729be0a93b6` (23 April 2026),
   an ancestor of both. The earlier claim in this file that the workflow
   passes `MUSE` was inaccurate for the recorded baseline and is corrected by
   `ADR-01-SPEC-AMEND-01`.
2. Current defect, preserved: the ProQuest uploader's intended EPUB-only
   fallback can fail because it retrieves a PDF ISBN first.

ADR-01 must record the ProQuest defect as a current-state defect and the
Project MUSE item as historical/resolved. It must not normalize the inventory
as though ProQuest scheduled delivery is fully healthy, and it must not
present the historical Project MUSE defect as current.

## 6. Enum questions and amendment-proposed dispositions

`ADR-01-SPEC-AMEND-01` records evidence-based dispositions for the questions
below, supported by the
[ADR-01 evidence ledger](adr-01-evidence-ledger.md) and explicit CTO
decisions of 2026-08-06. The corrected ADR-01 specification content carrying
these dispositions was independently reviewed and CTO-approved at exact
content head `1276c70a81e73f57d833eecb0e6886bd0cabf69e`; merge of
[PR #781](https://github.com/thoth-pub/thoth/pull/781) remains pending. This
inventory itself remains provisional: no final enum is approved by that
status change, and ADR-01 still owns the final inventory decision.

### Google Books vs Google Play

Proposed disposition (source-owner-confirmed): Google Books and Google Play
Books are one destination with canonical display name `Google Play Books`;
`Google Books` is a legacy alias; ADR-01 chooses one stable enum code, not
two platform values.

### EBSCOHost vs EBSCO Knowledge Base

Proposed disposition: `EBSCO_HOST` is a confirmed current push destination;
`EBSCO_KB` is a distinct product but is excluded from the initial enum
because no current independently selectable workflow, current SLA selection,
endpoint or publisher configuration was established (ledger EBSCO-01 through
EBSCO-05).

### ProQuest products

Proposed disposition: the canonical push destination is
`PROQUEST_EBOOK_CENTRAL` (aliases: `ProQuest` in current usage, `Ebrary`,
the `proquest_ebrary` export flavour); `EX_LIBRIS_KB` is a separate
pull-feed consumer sharing the `OCLC_KBART_PUBLIC` feed profile with
`OCLC_KB` without duplicate feeds or uploader jobs;
`PROQUEST_SERIALS_SOLUTIONS_KB` is excluded as historically distinct but
currently unverified (ledger PROQUEST-01 through PROQUEST-06, KBART-01
through KBART-03).

### Jisc

Proposed disposition: the destination is `JISC_NBK` (display name
`Jisc NBK`), not the older `Jisc KB` label; its mechanism is MARC21 files
through S3 (`JISC_NBK_MARC_S3`), not OCLC KBART; it is included but
initially inactive and non-assignable, creating no job or delivery (ledger
JISC-01 and JISC-02).

### Other manually managed destinations

As of 2026-08-06, the CTO confirms that there are no known current
manual-only distribution destinations outside the inspected repositories and
internal operational documentation. This is source-owner-confirmed evidence,
not a claim that no manual destination can ever exist.

No `OTHER` value may absorb an unidentified destination.

## 7. ADR-01 evidence checklist

Before approval, attach evidence for every destination:

- [ ] current uploader, feed or manual process;
- [ ] stable enum code;
- [ ] display name;
- [ ] current publisher list/configuration source;
- [ ] linked-platform group;
- [ ] back-catalogue behaviour;
- [ ] accepted work types;
- [ ] required file/metadata formats;
- [ ] update support;
- [ ] withdrawal support;
- [ ] location writeback;
- [ ] credential ownership model;
- [ ] dry-run/read-only verification method;
- [ ] operational owner;
- [ ] known defects;
- [ ] migration/backfill mapping.

## 8. Safety rule

The provisional inventory must not be converted directly into production rows or jobs.

MIG-01 must use an approved mapping, dry run and no-job import mode.
