# THOTH-ONIX-ACCESS-01 Implementation Report

Implementation of the approved `THOTH-ONIX-ACCESS-01` specification recorded on
[thoth-pub/thoth#893](https://github.com/thoth-pub/thoth/issues/893). It does
two things:

- enforces the Publication accessibility slot invariant at the authoritative
  domain write boundary;
- makes the canonical ONIX exporters emit no e-publication accessibility detail
  (List 79 type `09`) on print Products.

The report covers two rounds. The first round was reviewed at `72053fcd` as
`CHANGES REQUIRED`. The bounded correction round follows, under Architecture
Amendment 3 (#893 comment `5764154050`); section 16 records it.

The implementing agent records evidence here. It does not approve its own work,
and a fresh independent exact-head review is required.

## 1. Repository state

Owning GitHub issue: [thoth-pub/thoth#893](https://github.com/thoth-pub/thoth/issues/893)
Repository: `thoth-pub/thoth`
Workflow: STANDARD
Base branch: `develop`
Authorized base commit: `d694dd8f3708cbab48c738543711d70cae0d8bcc` (tree `99da58c74dfec97628c81b540ada65cb7a26563b`), per Base / Branch Amendment 2 (#893 comment `5762966680`)
Actual base commit: `d694dd8f3708cbab48c738543711d70cae0d8bcc` (tree `99da58c74dfec97628c81b540ada65cb7a26563b`)
Correction base: `72053fcdccfc07e48d88746d57e0d9b9b627cb19` (tree `07aa46b585802e9b29769059c86f5520f993183a`), per #893 comment `5764154050`
PR target: `develop`
Programme integration branch: None
Task branch: `feature/onix/thoth-onix-access-01`
Head commit: established after push through GitHub PR metadata (this file cannot embed the SHA of its own containing commit); the live PR head is the lifecycle authority
Pull request: draft [#934](https://github.com/thoth-pub/thoth/pull/934), targeting `develop`
Expected branch deletion after merge: YES
Final programme PR required: NO
Implementing model: Claude Opus 5
Reasoning level: HIGH

### 1.1 Preflight before branch creation (first round)

Run from the local `thoth` clone after `git fetch --prune origin`:

```text
git rev-parse origin/develop origin/develop^{tree}
  d694dd8f3708cbab48c738543711d70cae0d8bcc
  99da58c74dfec97628c81b540ada65cb7a26563b
git show-ref --verify refs/heads/feature/onix
  fatal: 'refs/heads/feature/onix' - not a valid ref
git show-ref --verify refs/remotes/origin/feature/onix
  fatal: 'refs/remotes/origin/feature/onix' - not a valid ref
git ls-remote --heads origin 'feature/onix' 'feature/onix/*' 'feature/thoth-onix-access-01'
  (no output)
git branch --list 'feature/onix*' 'feature/thoth-onix-access-01' -a
  (no output)
```

The branch was created in a fresh worktree directly from the exact base:

```text
git worktree add -b feature/onix/thoth-onix-access-01 <worktree> d694dd8f3708cbab48c738543711d70cae0d8bcc
git rev-parse HEAD HEAD^{tree}   -> d694dd8f..., 99da58c7...
git status --short               -> (clean)
```

No competing owner or branch existed. No branch was deleted, renamed, moved or
reset.

### 1.2 Preflight before the correction

```text
local HEAD             72053fcdccfc07e48d88746d57e0d9b9b627cb19  tree 07aa46b585802e9b29769059c86f5520f993183a
origin branch HEAD     72053fcdccfc07e48d88746d57e0d9b9b627cb19
PR #934 head           72053fcdccfc07e48d88746d57e0d9b9b627cb19  OPEN, draft, unmerged, base develop
origin/develop         d694dd8f3708cbab48c738543711d70cae0d8bcc
git status --short     (clean)
```

The correction started from the exact correction base. There was no rebase.

## 2. Scope confirmation

Approved specification and authority:

- the #893 body;
- the persisted-state gate `5712894829`;
- implementation authorization `5762841107`;
- Write-Budget Amendment 1 `5762945523`;
- Base / Branch Amendment 2 `5762966680`;
- **Architecture Amendment 3, the independent review decision and the
  correction authorization, all in `5764154050`**;
- the audit authority `ONIX-AUDIT-ACCESSIBILITY-01`: thoth-app#179 proposal
  `5571562316`, approval `5572432531` and final reconciliation `5572448584`.

Architecture Amendment 3 makes the merged Publication database CHECK
constraints the authoritative backend accessibility contract for this task
(section 6.1). Where an earlier #893 or audit statement implied a broader
backend-supported state space, the amendment supersedes it.

Implemented objective:

1. **Domain slot invariant.** At the authoritative Publication write/domain
   boundary, and for both create and patch:
   - the primary `accessibility_standard` slot accepts only the four WCAG
     values;
   - the additional `accessibility_additional_standard` slot accepts only the
     six EPUB Accessibility / PDF-UA values;
   - no other rule is added; the database CHECK constraints remain responsible
     for the type and combination rules.
2. **Canonical ONIX 3.0 / 3.1.** A print (Paperback/Hardback) Product emits no
   `ProductFormFeatureType 09` accessibility composite at all. That covers:
   - the Publisher statement `00`;
   - the primary, additional and EAA exception codes;
   - the Publication report URL `96`;
   - the Publisher contact `99`.

   This holds even for an in-memory state the database would reject. Digital
   Products are unchanged.
3. **Destination exporters.** Inventoried and reconciled, and
   `onix21_proquest_ebrary.rs` inspected read-only.

Out-of-scope changes made: NONE.

## 3. Commits

- `55d427b0a2cd1ec1c01d1fe5045006509bf7ede3`
  - message: `THOTH-ONIX-ACCESS-01: enforce the accessibility slot invariant and scope type-09 statements to digital Products`
  - tree `777af9094d11f07f6d6ee2abe4ecf6c9930c5eae`, parent `d694dd8f`
  - first-round source and tests
- `72053fcdccfc07e48d88746d57e0d9b9b627cb19`
  - message: `THOTH-ONIX-ACCESS-01: record the changelog entry and implementation report`
  - tree `07aa46b585802e9b29769059c86f5520f993183a`, parent `55d427b0`
  - first-round documentation; the reviewed head
- `dac96ebd00cf0e7a8f90df682fc6ea2ae7b3ad19`
  - message: `THOTH-ONIX-ACCESS-01: emit no type-09 accessibility detail on print Products`
  - tree `30a967f65477e4d6472ccdf69ca5ce68aa300b08`, parent `72053fcd`
  - the exporter correction and its tests
- The correction-round documentation commit updates this report and the
  `CHANGELOG.md` entry
  - its SHA is the PR's live head, per section 1

## 4. Files changed

Authorized write paths, the 14-path maximum from `5762841107` as amended by
`5762945523`, unchanged by `5762966680` and `5764154050`:

1. `thoth-errors/src/lib.rs`
2. `thoth-api/src/model/publication/mod.rs`
3. `thoth-api/src/model/publication/tests.rs`
4. `thoth-api/src/graphql/tests.rs`
5. `thoth-export-server/src/xml/onix3_thoth.rs`
6. `thoth-export-server/src/xml/onix31_thoth.rs`
7. `thoth-export-server/src/xml/onix3_jstor.rs`
8. `thoth-export-server/src/xml/onix3_oapen.rs`
9. `thoth-export-server/src/xml/onix3_overdrive.rs`
10. `thoth-export-server/src/xml/onix3_google_books.rs`
11. `thoth-export-server/src/xml/onix3_project_muse.rs`
12. `thoth-export-server/src/xml/onix21_ebsco_host.rs`
13. `CHANGELOG.md`

Authorized new-file paths:

14. `docs/engineering/ai-delivery/implementation-reports/THOTH-ONIX-ACCESS-01-implementation-report.md`

Read-only: `thoth-export-server/src/xml/onix21_proquest_ebrary.rs`.

Actual files changed:

- `thoth-errors/src/lib.rs`
  - reason: the minimum error representation for the two slots.
  - behavioural effect: adds the two unit variants
    `AccessibilityStandardSlotError` and
    `AccessibilityAdditionalStandardSlotError`.
    - Both carry fixed messages that do not echo input values.
    - Both map, like the other Publication domain errors, to the default GraphQL
      error `type`.
  - within authorized write budget: YES
- `thoth-api/src/model/publication/mod.rs`
  - reason: the invariant, at the shared Publication validation.
  - behavioural effect:
    - `AccessibilityStandard::is_primary_standard` and `is_additional_standard`
      classify every value to exactly one slot, through an exhaustive match
      with no wildcard.
    - `PublicationProperties` gains the `accessibility_standard` /
      `accessibility_additional_standard` accessors and the pure
      `validate_accessibility_constraints()`.
    - `validate()` calls the new check after its existing chapter and dimension
      checks, so their error precedence is unchanged.
  - within authorized write budget: YES
- `thoth-api/src/model/publication/tests.rs`
  - reason: the domain matrix, in a new `accessibility_slots` module gated on
    `backend` like the other DB-backed modules.
  - behavioural effect: tests only.
  - within authorized write budget: YES
- `thoth-api/src/graphql/tests.rs`
  - reason: rejection and acceptance through the real GraphQL mutations, and
    the pin on the database rules.
  - behavioural effect: tests only.
  - within authorized write budget: YES
- `thoth-export-server/src/xml/onix3_thoth.rs`, `thoth-export-server/src/xml/onix31_thoth.rs`
  - reason: the canonical print-Product correction.
  - behavioural effect:
    - A new `is_physical()` returns true for Paperback/Hardback only, matching
      the domain's `PublicationProperties::is_physical`.
    - One `if !is_physical(&publication.publication_type) { ... }` wraps the
      entire type-09 accessibility block: statement `00`, standard / additional
      / exception codes, report `96` and contact `99`.
    - No mapping code changes.
    - Against the base, the production delta in each file (ignoring
      whitespace) is exactly the helper, the guard and one `rustfmt` line
      rewrap, and it is identical in 3.0 and 3.1.
    - The tests are updated and extended as recorded in sections 9 and 16.
  - within authorized write budget: YES
- `CHANGELOG.md`
  - reason: the mandatory entry under `## [Unreleased]` / `### Fixed`, with the
    real PR number.
  - Revised in the correction round to state that print Products emit no type-09
    accessibility detail at all, and that the database constraints are the
    authoritative contract.
  - behavioural effect: none.
  - within authorized write budget: YES

Actual new files created:

- `docs/engineering/ai-delivery/implementation-reports/THOTH-ONIX-ACCESS-01-implementation-report.md` - within authorized new-file list: YES

Files deleted, moved or renamed: NONE

Authorized paths deliberately not changed, with the reasons in section 5.4:

- `onix3_jstor.rs`, `onix3_oapen.rs`, `onix3_overdrive.rs`, `onix3_google_books.rs`, `onix3_project_muse.rs`, `onix21_ebsco_host.rs`
- read-only `onix21_proquest_ebrary.rs`

### 4.1 Write-budget compliance

PASS.

- `git diff --name-only d694dd8f` lists exactly the 8 paths above, of the 14
  authorized.
- The correction round touched only `onix3_thoth.rs`, `onix31_thoth.rs`,
  `CHANGELOG.md` and this report.

## 4.2 Authorized actions actually used

- repository inspection: YES (read-only, including GitHub issue/comment reads)
- source edit: YES, within the budget
- new file creation: YES, this report only
- file deletion/move/rename: NO
- branch creation: YES, first round only, `feature/onix/thoth-onix-access-01` from the exact base
- commit: YES, four ordinary commits (section 3)
- push: YES, the authorized branch only, fast-forward
- PR creation/update: YES, exactly one draft PR (#934) targeting `develop`, created in the first round
  - its title, body and metadata were not edited in the correction round;
  - it was not marked ready, and no review was requested
- issue/comment mutation: NO
- manual CI dispatch/rerun: NO
- provider/runtime read: NO
- provider/runtime write: NO
- migration execution: NO. The local test harness applies the repository's embedded migrations to its disposable test database, as every backend test run does. No migration was run against any shared, provider or production database.
- release/tag/publication: NO
- merge: NO
- deployment: NO
- production activation: NO
- other: NO production GraphQL, database, credential or secret access. The production inventory was not repeated.

Unauthorized actions performed: NONE

## 4.3 Automatic and manual external effects

This was inspected read-only before the first push, from `.github/workflows/*.yml`.

- A push to a `feature/` branch triggers no workflow.
  - `build_test_and_check.yml` and `run_migrations.yml` run on push only for
    `master` and `develop`.
- Each push to the draft PR's branch triggers these natural `pull_request`
  workflows:
  - `build-test-and-check`: build, workspace tests, clippy and fmt, against
    service containers on the runner.
  - `run-migrations`: its classifier skips the migration job when no migration
    changes.
  - `check-changelog`.
  - `publish-to-dockerhub`:
    - builds and **pushes a non-production staging image**
      `ghcr.io/thoth-pub/thoth:staging-pr-934`;
    - is built with `THOTH_EXPORT_API=https://export.test.thoth.pub`;
    - performs no deployment.
- Not triggered: `docker_build_and_push_to_dockerhub_release.yml`, which fires
  only on a published GitHub release.
- No workflow performs a release, publication of a release image, deployment or
  production mutation.

Automatic CI/provider effects observed:

- natural CI runs on each pushed head (section 11);
- the automatic push of the staging image
  `ghcr.io/thoth-pub/thoth:staging-pr-934`, one per PR head. The first-round
  run 35624123748 log confirms the tag.

Manually initiated external actions: NONE

External writes/publication (releases, tags, packages, registries, third-party
services): the automatic PR staging image `ghcr.io/thoth-pub/thoth:staging-pr-934`
only. No release, tag or production image.

## 5. Implementation decisions

### 5.1 Authoritative write boundary

Tracing from GraphQL established the following:

- `createPublication` normalises its input, then calls
  `PublicationPolicy::can_create`, then `Publication::create`.
- `updatePublication` normalises its input, loads the current Publication, then
  calls `PublicationPolicy::can_update`, then `publication.update`.
- Both policies end in `PublicationProperties::validate(db)`, the existing home
  of the Publication domain rules (the chapter and dimension constraints).
- `Crud::create` / `update` are the unvalidated persistence primitives.
- A search of `thoth-api/src` and `thoth-api-server/src` found no other code
  path that writes a Publication or its accessibility fields.

The invariant therefore lives in `PublicationProperties`, covering
`NewPublication`, `PatchPublication` and `Publication`. It runs inside
`validate()`, after the policies' existing publisher-scope checks, so
authorization precedence is unchanged. No 15th path was needed.

### 5.2 Minimal error representation

There are two unit variants, one per slot, following the existing
`DimensionDigitalError` / `WidthEmptyError` convention.

- Neither exposes the rejected value.
- Neither adds a GraphQL extension type or field.
- A value that is wrong in both slots reports the primary slot first.

### 5.3 The slot rule only; the database decides the rest

The domain check decides only which slot a standard belongs to, and adds no
other accessibility rule. Under Architecture Amendment 3, the merged database
CHECK constraints (section 6.1) remain responsible for:

- which Publication types may hold standards or an exception;
- matching the additional standard to PDF/EPUB;
- the exclusivity of standards and an exception;
- the requirement that an additional standard has a primary one.

No thoth-app UI restriction was copied into the backend.

### 5.4 Exporter inventory

| Exporter | Accessibility logic inspected | Invalid-state crash risk | Physical type-09 issue | Source correction required | Reason | Tests added/updated |
|---|---|---|---|---|---|---|
| `onix3_thoth.rs` | Yes: statement `00`; primary `81`/`82`+`85`/`86`; additional `03`/`04`(+level), `05`, `06`; EAA `75`–`77`; report `96`; contact `99` | Slot-specific `_ => unreachable!()` on both slots; unreachable from API-accepted states (domain invariant plus the section 6.1 database rules) | **Yes**: every type-09 composite was emitted on Paperback/Hardback, including report `96`, which the database allows on print | **Yes** | The whole type-09 block is guarded by `!is_physical(...)` | Base Paperback assertion inverted; print-versus-digital loop over all 12 types; 5 print states (report URL alone, each field alone, full impossible state) × Paperback/Hardback asserting zero type-09; the full state on all 10 digital types, unchanged; exhaustive 280-state ordered-code matrix |
| `onix31_thoth.rs` | Byte-identical block to `onix3_thoth.rs` | As above | **Yes** | **Yes** | As above | As above |
| `onix3_jstor.rs` | Identical mapping arms | Same arms, guarded by the invariant | No: a record exists only for a PDF with a canonical Location, always Product `EB`/`E107` | No | Type-09 output is only ever on a digital Product | None |
| `onix3_oapen.rs` | Identical mapping arms | As JSTOR | No: PDF-only `EB`/`E107` | No | As JSTOR | None |
| `onix3_project_muse.rs` | Identical mapping arms | As JSTOR | No: PDF-only `EB`/`E107` | No | As JSTOR | None |
| `onix3_overdrive.rs` | Identical mapping arms on the main Publication | As JSTOR | No: main Publication is EPUB or PDF, Product `EB` | No | As JSTOR | None |
| `onix3_google_books.rs` | Identical mapping arms on the main Publication | As JSTOR | No: EPUB or PDF, Product `EB` | No | As JSTOR | None |
| `onix21_ebsco_host.rs` | EPUB-only additional standard via explicit `==` checks (`03`); report `96` for EPUB/PDF; contact `99`; no primary slot or exception | None: no slot `unreachable!()`, and unsupported values are skipped | No: Product `DG` only | No | Destination-specific ONIX 2.1 subset, deliberately different and preserved | None |
| `onix21_proquest_ebrary.rs` (read-only) | Accessibility block byte-identical to EBSCO Host | None | No: Product `DG` only | **No** | See section 5.6 | None (read-only) |

The exporters' other accessibility mentions are test fixtures only:
- `doideposit_crossref.rs`
- `json_thoth.rs`, which serializes the Work generically
- `kbart_oclc.rs`
- `bibtex_thoth.rs`
- `csv_thoth.rs`
- `marc21record_thoth.rs`

### 5.5 Print-versus-digital definition

- **Print** means `PAPERBACK | HARDBACK`. That is the domain's existing
  `PublicationProperties::is_physical` partition and the `BC`/`BB`
  ProductForms. `is_physical()` was not broadened.
- **Everything else is digital**, and its type-09 output is unchanged.
  - That includes the digital-audio `MP3`/`WAV` (`AN` Products).
  - Architecture Amendment 3 deliberately does not decide whether `AN`
    Products should carry the Publisher `00`/`99`, and this task does not
    change them.

### 5.6 ProQuest Ebrary read-only conclusion

`onix21_proquest_ebrary.rs` lines 159–233 (a `diff` against EBSCO Host lines
158–232 is empty) behave as follows:

- They emit ProductFormFeature only inside a Product whose `ProductForm` is
  `DG` (digital).
- They read the additional slot of the EPUB Publication through explicit `==`
  comparisons against the four EPUB values, skipping anything else.
- They never read the primary slot or the exception.
- They emit report `96` and contact `99`.

There is no slot `unreachable!()`, and no print Product can carry these
composites. The approved invariant is satisfied without changing it.
**No amendment is required**, and no HOLD was raised.

### 5.7 Deviation from the specification

NONE. The implementation follows the specification as amended by Architecture
Amendment 3.

## 6. Database and migration effects

Migration added: NO

`thoth-api/src/schema.rs` is unchanged. No migration directory, `up.sql`,
`down.sql`, Diesel schema, GraphQL schema, API field or enum value was created
or changed. No database constraint was relaxed, and no data was backfilled.

### 6.1 The approved database contract (Architecture Amendment 3)

`thoth-api/migrations/20250000_v1.0.0/up.sql` (lines 2970–2981) defines three
CHECK constraints on `publication`. No later migration alters them; `grep`
across `thoth-api/migrations` finds them only in that migration's `up.sql` and
`down.sql`.

Architecture Amendment 3 (#893 comment `5764154050`) approves them as the
authoritative backend accessibility contract for this task:

- `check_accessibility_standard_rules` sets per-type rules:
  - Paperback, Hardback, MP3 and WAV must have no standard, additional
    standard or exception.
  - PDF accepts WCAG or null in the primary slot, and PDF/UA or null in the
    additional slot.
  - Epub accepts WCAG or null in the primary slot, and EPUB Accessibility or
    null in the additional slot.
  - Every other type accepts WCAG or null in the primary slot, and no
    additional standard.
- `check_additional_standard_pdf_epub` allows an additional standard only on
  PDF and Epub.
- `check_standard_or_exception` makes an exception exclusive of both standards,
  and forbids an additional standard without a primary one.

Consequences recorded for this task:

1. **Wrong-slot state was not persistable before this change.**
   - The first-round RED run of the GraphQL rejection test shows a wrong-slot
     create refused with
     `Database error: new row for relation "publication" violates check constraint "check_accessibility_standard_rules"`.
   - That is a raw, `INTERNAL_ERROR`-typed message.
   - This task adds the domain rule and the stable error ahead of it.
2. **The constraints define the supported state space.**
   - A primary standard beside an EAA exception, or an additional standard
     without a primary one, is not a supported backend state.
   - None was made persistable.
   - `graphql_slot_valid_accessibility_states_the_database_forbids_are_still_refused_by_it`
     pins the unchanged constraints.
3. **No write that succeeded before this change is rejected after it.** Every
   wrong-slot write the domain now refuses was already refused by
   `check_accessibility_standard_rules`.
4. **The constraints do not govern `accessibility_report_url`.**
   - A persistable Paperback/Hardback may carry one.
   - The canonical exporters are therefore made fail-safe on print Products
     themselves (section 16), rather than relying on the database to have
     emptied every accessibility field.

Migration/data assessment:

- No migration, constraint change or backfill is required or authorized.
- The accepted production inventory (`5762841107`) found 0 wrong primary-slot,
  0 wrong additional-slot and 0 physical Publications with accessibility state.
- No production read was made by this task.

## 7. API and compatibility effects

- **GraphQL/API changes:** no field, input, argument, type or enum member was
  added, removed or renamed. Two things change behaviour:
  - `createPublication` / `updatePublication` refuse a wrong-slot value with
    `AccessibilityStandardSlotError` or
    `AccessibilityAdditionalStandardSlotError` messages, instead of the raw
    database constraint error.
  - Canonical ONIX 3.0/3.1 output for Paperback/Hardback Products carries no
    type-09 accessibility composite at all.
- **Generated schema/client updates:** none.
  - The schema is unchanged; the `thoth-client` build-time generated schema is
    unaffected, and `thoth-client/assets/queries.graphql` is unchanged.
  - `git diff --name-only d694dd8f -- thoth-api/migrations thoth-api/src/schema.rs thoth-client`
    is empty.
- **Backwards compatibility:**
  - Every write that succeeded before still succeeds (section 6.1).
  - Every accessibility value keeps its exact existing export code.
  - Digital-Product ONIX output is unchanged, including MP3/WAV.
- **Deprecations:** none.
- **Cross-repository dependencies:**
  - thoth-app#184 (`APP-IMPORT-ONIX-PUB-01`) must not claim final
    accessibility activation until this merges.
    - Its app-side accessibility validation and import contract must then
      follow the actual backend contract: this slot invariant plus the section
      6.1 database constraints.
  - ONIX consumers of canonical Thoth output, including a Thoth → ONIX → Thoth
    round trip, will no longer see any type-09 composite on print Products.
    - The approved accessibility audit (rules 89–90, 138) already classifies
      physical-Product type-09 output as an exporter defect.
    - The audit also forbids importing it as Publication accessibility.
  - No other repository is changed by this task.

## 8. Authorization and security

- Authorization paths changed: NONE.
- Roles/scopes involved: unchanged. The new check runs inside `validate()`,
  after the create and update policies' existing `require_publisher_for` checks.
- Negative authorization tests: the existing policy and `graphql_permissions`
  tests pass unchanged.
- Secret or personal-data handling: none.
  - The new errors carry fixed messages and echo no input.
  - No credentials, tokens or production data were used.
- Security limitations: none identified.

## 9. Tests and checks

The local environment was disposable only:
- a throwaway PostgreSQL 17.10 cluster (Homebrew, `initdb` in the task
  scratchpad, `localhost:55433/thoth_test`);
- a throwaway Redis (`127.0.0.1:56379`).

The environment was:

```text
TEST_DATABASE_URL=postgres://thoth@localhost:55433/thoth_test
TEST_REDIS_URL=redis://127.0.0.1:56379
THOTH_EXPORT_API=https://export.thoth.pub
```

`THOTH_EXPORT_API` is CI's own compile-time value; nothing connects to it.
Toolchain: `cargo 1.97.0`, `rustc 1.97.0`.

### 9.1 Strict TDD evidence (first round)

**Baseline at the exact base, before any edit:**

```text
cargo test -p thoth-api --features backend model::publication
  test result: ok. 40 passed; 0 failed
```

**RED 1: missing error representation.** The domain and GraphQL tests were
written first:

```text
cargo test -p thoth-api --features backend accessibility
  error[E0599]: no variant, associated function, or constant named `AccessibilityStandardSlotError` found for enum `thoth_errors::ThothError`
  error[E0599]: no variant ... `AccessibilityAdditionalStandardSlotError` ...
  error: could not compile `thoth-api` (lib test) due to 7 previous errors
```

All seven errors were the missing variants; everything else compiled.

**RED 2: behavioural.** After adding only the two error variants:

```text
cargo test -p thoth-api --features backend accessibility
  every_additional_standard_is_rejected_in_the_primary_slot_on_create_and_patch ... FAILED
    create validate: Epub   left: Ok(())   right: Err(AccessibilityStandardSlotError)
  every_wcag_standard_is_rejected_in_the_additional_slot_on_create_and_patch ... FAILED
    create validate: Epub   left: Ok(())   right: Err(AccessibilityAdditionalStandardSlotError)
  a_wrong_value_in_both_slots_is_rejected_for_the_primary_slot_first ... FAILED
    create validate: Epub   left: Ok(())   right: Err(AccessibilityStandardSlotError)
  graphql_publication_writes_reject_every_wrong_slot_accessibility_standard ... FAILED
    left:  "Database error: new row for relation \"publication\" violates check constraint \"check_accessibility_standard_rules\""
    right: "Accessibility Standard must be a WCAG standard. EPUB Accessibility and PDF/UA standards belong in Additional Accessibility Standard."
  test result: FAILED. 7 passed; 4 failed
```

The seven that passed:
- four pre-existing enum round-trip tests;
- three new characterization tests that pin behaviour this task must **not**
  change, so they passed at base as expected and are not claimed as RED:
  - `the_slot_invariant_rejects_no_correctly_slotted_combination`
  - `graphql_publication_writes_persist_every_accessibility_value_in_its_own_slot`
  - `graphql_slot_valid_accessibility_states_the_database_forbids_are_still_refused_by_it`

**GREEN (domain and GraphQL)**, after the implementation in `mod.rs`:

```text
cargo test -p thoth-api --features backend accessibility
  test result: ok. 11 passed; 0 failed
```

After refactor (adding `every_standard_belongs_to_exactly_one_slot`, the backend
gate and `cargo fmt`), the result is 12 passed. The added unit test was written
after the implementation and is not claimed as RED.

**Exporter RED (first round)**, with the exporter tests written first:

```text
cargo test -p thoth-export-server thoth_works
  test_onix3_thoth_works  ... FAILED  onix3_thoth.rs:2759  assertion failed: !output.contains("<ProductFormFeatureType>09</ProductFormFeatureType>")
  test_onix31_thoth_works ... FAILED  onix31_thoth.rs:3068 assertion failed: !output.contains("<ProductFormFeatureType>09</ProductFormFeatureType>")
  test result: FAILED. 0 passed; 2 failed
```

A Paperback Product emitted the Publisher statement and contact as type 09. The
first round then guarded those two composites and went GREEN.

The independent review found that this left report `96` (section 16). The
correction-round RED, GREEN and falsification evidence is in section 16.2.

### 9.2 Domain matrix: `thoth-api/src/model/publication/tests.rs`, module `accessibility_slots`

Each case is asserted through four checks:
- `NewPublication::validate`
- `PublicationPolicy::can_create`
- `PatchPublication::validate`
- `PublicationPolicy::can_update`

Each case runs on both an Epub and a Paperback, using a publisher-scoped user.

| Case | Values | Result |
|---|---|---|
| Wrong primary slot | each of the 6 additional-standard values, alone, beside a valid additional standard (`PdfUa1`), and beside an exception | `AccessibilityStandardSlotError` on create and patch |
| Wrong additional slot | each of the 4 WCAG values, alone, beside a valid primary standard (`Wcag21aa`), and beside an exception | `AccessibilityAdditionalStandardSlotError` on create and patch |
| Both slots wrong | — | primary-slot error first |
| Slot rule adds nothing else | (null + 4) primary × (null + 6) additional × (null + 3) exceptions = 140 states | `Ok(())` from the slot check on create and patch; persistence stays with the database constraints |
| Classification | every value | exactly one slot |

### 9.3 GraphQL evidence: `thoth-api/src/graphql/tests.rs`

Executed with a superuser through the real schema.

- `graphql_publication_writes_reject_every_wrong_slot_accessibility_standard`
  - Sends every one of the 6 + 4 wrong-slot values through `createPublication`
    (as a PDF the Work does not yet hold) and through `updatePublication` (on
    an existing EPUB).
  - Each write is refused with the exact domain message and a null payload.
  - Afterwards, the Work holds only the one EPUB, still without accessibility
    state.
- `graphql_publication_writes_persist_every_accessibility_value_in_its_own_slot`
  - Every additional value is created on its database-permitted type (EPUB
    Accessibility on Epub, PDF/UA on PDF) together with a WCAG value.
  - Each is then updated through all 4 WCAG values, alone and beside the
    additional value.
  - Each of the 3 EAA exceptions is created, and every stored row is verified.
- `graphql_slot_valid_accessibility_states_the_database_forbids_are_still_refused_by_it`
  - Covers three correctly slotted states: standard + exception; an additional
    standard alone; and a standard on a Paperback.
  - Each is refused by the approved database constraint named in the error
    (`check_standard_or_exception` or `check_accessibility_standard_rules`),
    never by a slot error.

### 9.4 Exporter evidence: `onix3_thoth.rs`, `onix31_thoth.rs` in `test_onix3_thoth_works` / `test_onix31_thoth_works`

- **Base fixture:** a Paperback with a Publisher statement and an
  accessibility contact emits no type-09 composite. The previous assertion
  encoded the defect and was inverted.
- **Print, all fields:** Paperback and Hardback each carry the fixture's
  Publisher statement and contact, plus one of five Publication-level states.
  Each of the ten outputs contains **zero**
  `<ProductFormFeatureType>09</ProductFormFeatureType>`. The five states are:
  - the report URL alone;
  - `WCAG21AA` alone;
  - `PDF_UA1` alone;
  - `MICRO_ENTERPRISES` alone;
  - the full impossible state `WCAG22AAA` + `EPUB_A11Y11AA` +
    `FUNDAMENTAL_ALTERATION` + report URL.
- **Digital, statement and contact:** each of PDF, HTML, XML, EPUB, MOBI, AZW3,
  DOCX, FICTION_BOOK, MP3 and WAV emits the exact statement (`00`) and contact
  (`99`) blocks.
- **Digital, full state unchanged:** each of the same ten types, MP3 and WAV
  included, emits the ordered type-09 values
  `00, 82, 86, 04, 85, 77, 96, 99` for the full state.
- **Every accepted slot state on an EPUB:** (null + 4) × (null + 6) × (null + 3)
  × (with and without a report URL), which is 280 outputs.
  - The ordered type-09 value list must equal statement `00`, then the primary
    codes, additional codes, exception code, report `96` and contact `99`.
  - The codes are exactly the existing mappings.
  - No state reaches an `unreachable!()`.

### 9.5 Full gates on the final source

These ran on the correction source, `dac96ebd`. The documentation commit
changes only `CHANGELOG.md` and this report.

| Check | Command | Result |
|---|---|---|
| Focused exporter tests | `cargo test -p thoth-export-server thoth_works` | exit 0; `2 passed; 0 failed` |
| Formatting | `cargo fmt --all -- --check` | exit 0 |
| Lint/static analysis | `cargo clippy --all --all-targets --all-features -- -D warnings` | exit 0; the only note is the pre-existing future-incompatibility report for dependency `proc-macro-error2 v2.0.1` |
| API unit/integration tests | `cargo test -p thoth-api --features backend` | exit 0; `1250 passed; 0 failed` (lib), `13 passed; 0 failed` (`tests/graphql_permissions.rs`), `8 ignored` doc-tests |
| Export server tests | `cargo test -p thoth-export-server` | exit 0; `144 passed; 0 failed` (lib), `2 passed` (integration) |
| Export server build | `cargo build -p thoth-export-server` | exit 0 |
| Workspace tests | `cargo test --workspace` | exit 0; per target: 31, 1250, 13, 3, 4, 11, 144, 6 and 2 passed (1,464 total), 0 failed, 8 doc-tests ignored |
| Workspace check | `cargo check --workspace` | exit 0 |
| Whitespace | `git diff --check` | clean |

### 9.6 Other required checks

- **Build without the `backend` feature.** `cargo check -p thoth-api` fails with
  26 errors on both the exact base (checked by stashing the changes) and this
  branch.
  - The errors are in unrelated modules, such as `lib.rs` and
    `model/title/policy.rs`.
  - This is a pre-existing condition, not introduced here.
  - The new DB-backed test module is gated on `backend`, like its siblings.
- **Changed-path audit.** `git diff --name-only d694dd8f` lists exactly the 8
  paths in section 4.
- **Drift check.** `git diff --name-only d694dd8f` restricted to
  `thoth-api/migrations`, `thoth-api/src/schema.rs`, `thoth-client`,
  `onix21_proquest_ebrary.rs` and the six destination exporters is empty.

## 10. Manual verification

- Environment: local disposable PostgreSQL 17.10 and Redis only.
- Steps: none beyond the automated evidence. No production, staging or provider
  environment was used.
- Observed result: see sections 9 and 16.
- Evidence link/screenshot/log reference: the local command outputs recorded
  in sections 9 and 16, and the PR's natural CI.

## 11. CI

CI status: the live PR checks on the current head are the lifecycle authority.
No run was dispatched or rerun manually.

- **First head `55d427b0`:**
  - `run-migrations` 35623831013: success.
  - `check-changelog` 35623831014: failure, as expected; the entry was added in
    `72053fcd`.
  - `build-test-and-check` 35623831038: success.
  - `publish-to-dockerhub` 35623831020: success.
- **Reviewed head `72053fcd`:**
  - `build-test-and-check` 35624123726: success (build, test, lint,
    format_check).
  - `check-changelog` 35624123742: success.
  - `run-migrations` 35624123788: success; the migration job was skipped by the
    classifier because no migration changed.
  - `publish-to-dockerhub` 35624123748: success; pushed
    `ghcr.io/thoth-pub/thoth:staging-pr-934`.
- **Correction source head `dac96ebd`:**
  - `build-test-and-check` 35630163918: success (build, test, lint,
    format_check).
  - `check-changelog` 35630163935: success.
  - `run-migrations` 35630164025: success; the migration job was skipped.
  - `publish-to-dockerhub` 35630163952: success; pushed
    `ghcr.io/thoth-pub/thoth:staging-pr-934`.
- **Final documentation head:** recorded on the PR.

## 12. Rollout and rollback

- **Initial state after merge:**
  - The domain rule applies to all Publication create and update writes once a
    release containing it is deployed.
  - Canonical ONIX output for print Products carries no type-09 accessibility
    detail.
- **Activation required:** none beyond normal release and deployment, which
  are separately gated and not authorized here.
- **Feature flag/configuration:** none.
- **Migration sequence:** none.
- **Rollback/disable procedure:** revert the task commits.
  - There is no data or schema to unwind.
  - After revert, wrong-slot writes are again refused by the database
    constraint alone.
- **Monitoring required:**
  - Watch for any client that relied on the raw database error text for
    wrong-slot writes; none is known.
  - Watch for ONIX recipients of print Products that consumed type-09
    composites.

## 13. Known limitations and deferred work

- **Digital audio (MP3/WAV).**
  - Their type-09 output, including the Publisher statement and contact, is
    unchanged by this task.
  - Architecture Amendment 3 deliberately does not decide whether ONIX `AN`
    Products should carry that detail.
  - Any change needs its own approved decision.
- **Friendly database messages.**
  - `thoth-errors/src/database_errors.rs` has no friendly message for the three
    accessibility CHECK constraints, so their violations still surface as raw
    `Database error: ...`.
  - Examples are a standard on a print type, or a standard beside an exception.
  - That file is outside the write budget.
- **thoth-app alignment.** The downstream thoth-app accessibility form and
  import contract must be aligned with the section 6.1 backend contract. That
  work belongs to thoth-app#184 and is not done here.

## 14. Unresolved issues

- NONE. The earlier specification/database discrepancy is resolved by
  Architecture Amendment 3, which approves the database contract.

## 15. Agent self-assessment

The agent may identify risks but may not approve the task.

Suggested review focus:

- **The print-Product guard.** Confirm that the single `!is_physical(...)` guard
  covers every type-09 source in both canonical exporters. `git diff -w`
  against `d694dd8f` shows the whole production delta. Confirm too that no
  mapping code or digital output changed.
- **Placement and precedence.**
  - Is `PublicationProperties::validate()` the right authoritative boundary for
    the slot invariant?
  - Is the ordering after the chapter and dimension checks acceptable?
- **Unchanged destination exporters.** Is it acceptable to leave the six
  destination exporters unchanged, since they only ever emit digital Products
  and share the canonical mapping arms?
- **Error wording.** The two new error messages, and the absence of a dedicated
  GraphQL extension `type`.

## 16. Remediation (independent-review response)

### 16.1 Review decision and finding

The independent exact-head review of `72053fcd` (tree `07aa46b5`, base
`d694dd8f`), recorded in #893 comment `5764154050`, was **CHANGES REQUIRED**.

- **Resolved by the same comment:** the specification/database conflict,
  through Architecture Amendment 3.
- **The one remaining blocking finding:**
  - The canonical ONIX 3.0/3.1 exporters suppressed Publisher `00`/`99` on
    Paperback/Hardback.
  - A physical Publication carrying `accessibility_report_url`, which the
    database permits, still emitted `ProductFormFeatureType 09` / `96`.

### 16.2 Correction evidence

**RED**, with the tests added before any exporter change.

- In each canonical test, Paperback and Hardback carry the fixture's Publisher
  statement and contact, plus five Publication-level states: the report URL
  alone, each standard or exception alone, and the full impossible state.
- The tests assert zero type-09 composites. They also pin the full state's
  unchanged sequence on all ten digital types.

```text
cargo test -p thoth-export-server thoth_works
  test_onix3_thoth_works  ... FAILED
    panicked at onix3_thoth.rs:3563: PAPERBACK None None None Some("https://report.url")
    left: 1   right: 0
  test_onix31_thoth_works ... FAILED
    panicked at onix31_thoth.rs:3987: PAPERBACK None None None Some("https://report.url")
    left: 1   right: 0
  test result: FAILED. 0 passed; 2 failed
```

Only the report URL was set, with the statement and contact already
suppressed, so the escaping composite is the `96` report URL.

**Correction.** In both canonical exporters, one `if !is_physical(&publication.publication_type) { ... }`
now wraps the entire type-09 accessibility block.

- The block covers statement `00`, standard / additional / exception codes,
  report `96` and contact `99`.
- The first-round per-item guards on the statement and contact are removed
  inside it, back to their base form.
- Mapping codes and `is_physical()` are unchanged.

**GREEN**:

```text
cargo test -p thoth-export-server thoth_works
  test result: ok. 2 passed; 0 failed
```

**Falsification.** These assertions did not execute at RED, because the test
stops at the first failure. Each mutation was temporary, applied to
`onix3_thoth.rs`, restored byte-for-byte from a saved copy and never committed:

```text
A. is_physical() without HARDBACK       -> panicked at onix3_thoth.rs:3432: HARDBACK
B. guard bypassed when a primary        -> panicked at onix3_thoth.rs:3563:
   standard is set (impossible state)      PAPERBACK Some(WCAG21AA) None None None   left: 4  right: 0
C. is_physical() plus MP3               -> panicked at onix3_thoth.rs:3451: MP3
restored file identical (cmp); cargo test -p thoth-export-server test_onix3_thoth_works -> 1 passed
```

- **B** shows the impossible-state assertion is live. A print Product with only
  a primary standard would otherwise emit four composites: `00`, `81`, `85` and
  `99`.
- **C** shows that MP3/WAV output is pinned by the assertions.

**Full gates.** See section 9.5.

**Commits.**

- `dac96ebd00cf0e7a8f90df682fc6ea2ae7b3ad19`
  - tree `30a967f65477e4d6472ccdf69ca5ce68aa300b08`, parent `72053fcd`
  - the exporter correction
- The documentation commit that carries this report
  - its SHA is the live PR head

### 16.3 Correction-round scope

- **Touched:** `onix3_thoth.rs`, `onix31_thoth.rs`, `CHANGELOG.md` (the
  existing PR #934 entry made accurate) and this report.
- **Unchanged:**
  - the API/domain files;
  - the destination exporters;
  - `onix21_proquest_ebrary.rs`;
  - migrations, `schema.rs` and `thoth-client`;
  - MP3/WAV behaviour.
