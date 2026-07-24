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
| `PROJECT_MUSE` | Project MUSE | `ProjectMUSE` FTP | AutomaticPush | - | intended weekly | none; notification flow | unverified | unverified | environment list plus per-publisher credentials | VERIFIED CANDIDATE; CURRENT SCHEDULE BUG |
| `JSTOR` | JSTOR | `JSTOR` FTP | AutomaticPush | - | weekly | none; notification flow | unverified | unverified | environment list plus per-publisher folder | VERIFIED CANDIDATE |
| `EBSCO_HOST` | EBSCOHost | `EBSCOHost` FTP | AutomaticPush | - | weekly | none; notification flow | unverified | unverified | environment publisher list | VERIFIED CANDIDATE |
| `PROQUEST` | ProQuest | `ProQuest` FTP/ebrary format | AutomaticPush | - | weekly | none; notification flow | unverified | unverified | environment publisher list | VERIFIED CANDIDATE; PRODUCT SCOPE OPEN |
| `GOOGLE_PLAY` | Google Play Books | `GooglePlay` bucket delivery | AutomaticPush | - | daily | none | unverified | unverified | environment list plus per-publisher collection code | VERIFIED CANDIDATE |
| `BKCI` | Book Citation Index | `BKCI` FTP | AutomaticPush | - | monthly | none; notification flow | unverified | unverified | environment list plus per-publisher credentials | VERIFIED CANDIDATE |
| `OCLC_KB` | OCLC Knowledge Base | Thoth KBART pull-feed/index | PullFeed | - | destination pull | feed membership, no location writeback | feed reflects current state | feed removal reflects current state | existing OCLC arrangements/records | DESIGN CANDIDATE; VERIFY OPERATIONS |

## 5. Current operational caveats

The current repository documents these defects:

1. The Project MUSE scheduled workflow passes `MUSE`, while current dispatch expects `ProjectMUSE`.
2. The ProQuest uploader's intended EPUB-only fallback can fail because it retrieves a PDF ISBN first.

ADR-01 must record these as current-state defects. It must not normalize the inventory as though scheduled delivery is healthy.

## 6. Unresolved enum questions

### Google Books vs Google Play

The current Google Play delivery sends ONIX using a Google Books export profile.

ADR-01 must establish whether:

- `GOOGLE_PLAY` represents the complete destination;
- Google Books is independently selectable and needs `GOOGLE_BOOKS`;
- both are one operational delivery with two user-visible outcomes.

Do not create both values solely from export-format naming.

### EBSCOHost vs EBSCO Knowledge Base

Current code verifies EBSCOHost push delivery.

Establish whether a separate EBSCO KB pull-feed/configuration exists. If independently meaningful, add `EBSCO_KB`; otherwise do not.

### ProQuest products

Current code exposes one `ProQuest` uploader using a ProQuest ebrary export.

Establish whether staff sell/configure multiple independently selectable ProQuest destinations. If yes, use separate explicit enum values and adapters/descriptors.

### Other manually managed destinations

Interview operations staff and inspect private configuration before enum approval.

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
