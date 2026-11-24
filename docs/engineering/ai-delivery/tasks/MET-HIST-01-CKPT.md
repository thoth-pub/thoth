# MET-HIST-01-CKPT - Advance the source checkpoint for MONTH imports, and for the OAPEN PARTIAL months

Status: APPROVED
Programme: MET-HIST-01 (Metrics history import)
Stage: CKPT (H4 spec R2 section 7 item 5, question Q4, departure 7; OAPEN 2026 import gate 5)
Owning GitHub issue: thoth-pub/thoth#1052
Repository: thoth-pub/thoth
Workflow: STANDARD
Base branch: develop
Exact authorized base commit: bf133b229e4cb1b85337b8e84105d7523d1bc2fd (confirmed as the develop head at approval and again when the task branch was created)
PR target: develop
Programme integration branch: None
Risk: CRITICAL (import lifecycle and checkpoint semantics, production data that has no retraction)
Owner: Thoth engineering control thread (the thoth#1046 release thread)
Approved by: CTO (grant comment 7128051136 on thoth#1052, 2026-11-24)
Dependencies: none upstream (MET-WP4-MONTH-01 landed and released in v1.12.0). Downstream: MET-HIST-01 H5d and H6 (OPERAS MONTH accounts) and MET-HIST-02 (OAPEN 2026 import, thoth-sphinx#74, its gate 5).
Target branch name: `feature/metrics-history/met-hist-01-ckpt`

Namespace preflight (AGENTS.md section 5.1), run against live refs on 2026-11-24: the exact ref is absent; no flat ref `feature/metrics-history` exists. The flat ref `feature/metrics` exists, so `feature/metrics/<task>` is not usable and the area name above is used instead. Re-run at approval; HOLD rather than touch another branch.

Authority condition: this record is repository-authoritative when this exact content is reachable from the repository's authoritative integration branch. Live review, authorization and merge evidence is the GitHub pull-request record.

## 1. Objective

Make a perfect MONTH import advance the source checkpoint (`last_successful_period_end`) and record its manifest in the period-manifest cursor, so that discovery against Thoth's checkpoint works for MONTH accounts as it already does for DAY accounts. In addition, let one named exception advance it for PARTIAL coverage: OAPEN downloads in the months 2026-01 and 2026-08 only. The CloudFront (DAY) behaviour does not change, by construction.

## 2. Background and authority

Authoritative sources:

- merged code at the base commit: `thoth-api/src/model/metric_ingestion_lifecycle/mod.rs` (`update_metric_source_checkpoint`, the coverage evidence query and the `successful` binding) and `thoth-api/src/model/metric_source_checkpoint/mod.rs` (the period-manifest cursor);
- `docs/metrics/contract-register.md` ("Checkpoint progress", the cursor paragraph, the quarantine paragraph);
- MET-HIST-01 H4 spec R2 (thoth-sphinx#63), section 7 item 5, facts F1, F4, F5, F9, question Q4, departure 7, section 3.3 (the fence: every coverage row of a MONTH file has exactly the period `[1st, next 1st)`) and its discovery rule (sections 3.5 and 4.1: the next period is the first bundle period whose start is at or after the checkpoint's `last_successful_period_end`; the operator's expected start must equal it);
- MET-HIST-02 R2 (thoth-sphinx#74), section 3.3 and the CTO decisions of 2026-11-23 (the two estimated OAPEN months are asserted `PARTIAL`) and 2026-11-24 (a PARTIAL row may advance the checkpoint for exactly those months).

Current behaviour (verified at the base commit; the lifecycle module is unchanged since v1.12.0):

- A managed import is exactly one day (`beginMetricImport` refuses any other period). Observations and coverage rows carry their own periods; a coverage row only needs `period_end > period_start` and has no grain column.
- `update_metric_source_checkpoint` counts the coverage rows of the terminal import (`COMPLETED` or `COMPLETED_WITH_ERRORS`) and treats a `COMPLETED` import as successful only when there is at least one row and every row is `COMPLETE` **and** has exactly the import's one-day period (`mod.rs` 1474 to 1488). Only then does it move `last_successful_period_end` (by `GREATEST`) to the import's period end and record the period's manifest digest in the cursor.
- A MONTH import has a one-day envelope (the 1st) and monthly coverage rows, so equality never holds: the checkpoint and the cursor never move, and discovery would rediscover the first month forever.
- The cursor only requires strictly ascending period starts (at most 64 entries), so monthly entries are valid. For a MONTH account 64 entries are 64 months, so the archive driver's cursor agreement then covers the newest 64 months only; older entries are evicted by period, as for days.
- The CloudFront quarantine-only exception records a manifest but never advances the checkpoint; it reuses the same coverage test (`complete_coverage`, `mod.rs` 1495).

## 3. Explicit scope

The task must:

1. Replace the coverage test in `update_metric_source_checkpoint` by the accepted-row rule of section 7.1, evaluated by one pure, table-testable function that returns both `accepted` (the new rule) and `strict_day` (every row satisfies rule A), and keep everything else of that function (locks, lock order, claim checks, replay, the single `UPDATE`) as it is.
2. Add the PARTIAL allowlist of section 7.1 as a code constant with a pin test.
3. Feed `strict_day` (never `accepted`) to the CloudFront quarantine-only exception, and evaluate rule B only for sources whose driver key is not `cloudfront`, so CloudFront is rule A by construction.
4. Update the stale text: the module documentation (`mod.rs` 46 to 85), the function documentation (`mod.rs` 1389 to 1417), and in `docs/metrics/contract-register.md` the three places that state the predicate (about lines 792 to 802 "Checkpoint progress"; 872 to 882, the cursor paragraph, which says a mismatched period, PARTIAL and UNKNOWN leave the cursor unchanged; and 934 to 935, the quarantine paragraph, which must say it uses the strict test and that rule B never applies to it).
5. Add the tests of section 11 and a CHANGELOG entry.

## 4. Non-goals

The task must not:

1. Change the CloudFront (DAY) path or any existing test's expectation.
2. Add a migration, a column, a configuration table or a GraphQL field; or change a published GraphQL description. Two descriptions become stale and stay as they are, because their text sits in the client schema snapshot (`thoth-client/assets/schema.graphql`) that downstream repositories generate from: `lastSuccessfulPeriodEnd` ("successfully and completely collected", `graphql/model.rs` 4834 to 4837) and the `updateMetricSourceCheckpoint` mutation description ("all of which cover exactly the import period completely", `graphql/mutation.rs` 676). The contract register carries the nuance; a follow-up issue for the two descriptions is opened when this lands.
3. Add ordering enforcement (see 7.1, "High-water mark").
4. Change batch ingestion, coverage-run maintenance, serving, rollups or the native-month tables.
5. Make a MONTH or PARTIAL import happen. No import, registry row, account or switch is part of this task.
6. Add a replay or correction mode for an imported month (the Sphinx driver has none; MET-HIST-02 R2 records the PARTIAL months as uncorrectable until one exists).

## 5. Cross-repository impact

Affected contracts:

- database/domain model: NOT AFFECTED (no schema change)
- GraphQL/API schema and behaviour: AFFECTED in behaviour only: `updateMetricSourceCheckpoint` now advances `lastSuccessfulPeriodEnd` and the cursor for MONTH units; no field, type or description changes (two descriptions go stale, section 4 item 2)
- generated clients/types: NOT AFFECTED
- authorization semantics: NOT AFFECTED
- export formats: NOT AFFECTED
- configuration/environment contracts: NOT AFFECTED
- event/job payloads: NOT AFFECTED
- dissemination/platform behaviour: NOT AFFECTED
- UI assumptions: NOT AFFECTED
- CMS/site contracts: NOT AFFECTED
- package/library interfaces: NOT AFFECTED
- deployment/compatibility windows: AFFECTED: the Thoth server carrying this change must be deployed before any MONTH unit is imported (H5d, H6, MET-HIST-02)

Owning repository for the changed behaviour: thoth-pub/thoth.

Known consumers (from `docs/engineering/repository-map/contracts.md` and the H4 spec):

- thoth-pub/thoth-sphinx: REMAINS COMPATIBLE, subject to one gate. Its CloudFront drivers send one-day units with equal coverage and see no change. Its archive driver (MET-HIST-01 H4, thoth-sphinx#63) computes the next period from `lastSuccessfulPeriodEnd` ("first bundle period with start at or after it"), so after a MONTH unit (checkpoint = the 2nd) it asks for the first of the next month. Rule B is stricter than "contains the envelope day" (D1): it accepts only a coverage row that is exactly the calendar month starting on the envelope day. The H4 spec's fence (section 3.3) requires every coverage row of a MONTH file to equal the file's own period `[1st, next 1st)`, which is exactly that shape; but a driver that emitted any other MONTH coverage row would never advance and would stop its run with no procedural fallback. **Gate before the first MONTH import** (its own grant, in the H5d or MET-HIST-02 stage): evidence that the emitted MONTH bundle's coverage rows have exactly that shape (the driver's fence tests and `archive-verify` over the real bundle). HOLD if not.
- thoth-pub/metrics-dashboard, thoth-pub/metrics-widget, thoth-pub/thoth-app, thoth-pub/thoth-dissemination, thoth-pub/email-automation: REMAINS COMPATIBLE (none reads the checkpoint).

Compatibility requirements: a Sphinx runner built before this change keeps working (the mutation and its fields are unchanged). Merge order: this change first, then release and deployment, then any MONTH import stage. Downstream repository-local tasks: None required (the H5d and MET-HIST-02 gates already name this change); the follow-up issue for the stale descriptions belongs to this repository.

## 6. Invariants

The implementation must preserve:

1. Lock order: checkpoint `FOR UPDATE`, import `FOR SHARE`, source account `FOR SHARE`, source `FOR SHARE`. The new reads take no lock (see 7.4).
2. A released replay (claim already released) stays read-only, recognises recorded completion from `last_completed_at` and `last_successful_period_end` exactly as today, and never writes.
3. Neither `last_successful_period_end` nor `last_completed_at` moves backwards.
4. The cursor records the manifest only under the same predicate that advances the checkpoint, or under the one CloudFront quarantine-only exception, in the same single `UPDATE` that releases the claim.
5. `COMPLETED_WITH_ERRORS`, zero coverage rows, any `UNKNOWN` row and any row not accepted by 7.1 never advance the checkpoint.
6. `INTERNAL_STATE_INCONSISTENCY` and `STALE_SOURCE_CLAIM` behaviour is unchanged.
7. Units of a source whose driver key is `cloudfront` behave byte for byte as before whatever the registry holds: they are judged by rule A alone, and rule A is judged from the coverage row alone (no registry read decides it).

## 7. Required behaviour

### 7.1 Success behaviour

Notation. The import's envelope is `[e0, e1)` with `e1 = e0 + 1 day` (already enforced). A coverage row `r` has platform `P`, measure `M`, half-open period `[s, t)` and status `S`. `month_supported(P, M)` is true when the registry pair `(P, M)` (table `metric_platform_measure`) exists and lists `MONTH` in `supported_grains`; a missing pair row is false.

**Accepted row.** `r` is accepted iff A or B holds.

- **A (unchanged, judged from the row alone).** `S = COMPLETE`, `s = e0`, `t = e1`.
- **B (new, the MONTH rule; evaluated only when the locked source's driver key is not `cloudfront`).** `month_supported(P, M)`, and `s = e0`, and `(s, t)` satisfies the existing MONTH period rule (`period_matches_grain(MONTH, s, t)`: `s` is the 1st of a month and `t` is the 1st of the next), that is the exact calendar month that starts on the envelope day, and either
  - `S = COMPLETE`, or
  - `S = PARTIAL` and `(P.code, M.code, s)` is in the PARTIAL allowlist.

**Successful import.** `COMPLETED`, at least one coverage row, and every row accepted. Then, as today, `last_successful_period_end = GREATEST(existing, e1)` (the 2nd of the month for a MONTH unit) and the cursor records the manifest digest for `period_start = e0`.

**The pure function.** Signature (shape, not names): `(groups, total, envelope, rule_b_enabled) -> Result<{accepted, strict_day}, Inconsistent>`. `groups` is the import's coverage grouped as (platform code, measure code, `s`, `t`, status as text, `month_supported`, count); `total` is the plain row count; `envelope` is `[e0, e1)`; the caller sets `rule_b_enabled = (source.driver_key != Some("cloudfront"))`. `accepted` means every row satisfies A or (when `rule_b_enabled`) B; `strict_day` means every row satisfies A. Both are false when `total` is 0 (as today's `total >= 1`). The function returns `Inconsistent` when the group counts do not add up to `total` or a status is not one of the three known values, and the caller maps that to `INTERNAL_STATE_INCONSISTENCY` with no write. The query behind it reads `metric_coverage` itself, with `LEFT JOIN`s to `metric_platform`, `metric_measure` and `metric_platform_measure` and `month_supported = COALESCE('MONTH' = ANY(pm.supported_grains), false)`; platform and measure codes are decoded as nullable and a NULL takes the same inconsistency path (unreachable under the foreign keys, but then a decode failure cannot surface as a database error); `total` is taken in the same statement by an uncorrelated scalar subquery (`COUNT(*) OVER ()` would count groups after the `GROUP BY`, not rows) or in the same transaction. (`metric_coverage` has no foreign key to the pair table, so an inner join could silently drop a row and let the rest pass.) With these inputs the sum check and the CloudFront case are testable without a database.

**PARTIAL allowlist.** A code constant of exactly two entries, `(platform code, measure code, month start)`:

| platform code | measure code | month start |
|---|---|---|
| `oapen` | `downloads` | 2026-01-01 |
| `oapen` | `downloads` | 2026-08-01 |

Codes compare exactly (no case folding, no trimming). The entry's month end is computed (the next 1st), not stored. A pin test fails on any other content. Extending the list is a code change under its own release.

**Decisions taken (approved with the grant):**

- **D1, row shape.** The issue text says "contains the envelope day". This spec is stricter on purpose: a MONTH row is accepted only when it is the exact calendar month starting on the envelope day, only for a platform-measure pair the registry declares MONTH-capable, and only for a source that is not CloudFront. A plain "contains" would accept, for a DAY source, a wider COMPLETE row that today fails closed. Rule B implies "contains", so the intended MONTH behaviour is the same. If the CTO prefers the general "contains", the change is local to B.
- **D2, where the exception lives, and what it matches.** A code allowlist keyed by platform code, measure code and month, with tests; each extension is a release. Per-account configuration would need a migration and an admin write path and widens the surface of a CRITICAL path. The driver key is deliberately not part of the key: platform and measure codes are unique and immutable, an account's platform is immutable, the OAPEN source rows are not in the registry yet (H5d), and the exception concerns the data (OAPEN downloads in two months), not who delivers it. Consequence, stated so it is not a surprise: every account on platform `oapen` matches for those two months (the H4 plan lists five OAPEN MONTH accounts: OBP, punctum, Mattering, African Minds and SUP); today the H4 archive fence ends `oapen`/`downloads` at 2025-10, so nothing overlaps. The codes `oapen` and `downloads` are not seeded by any migration: the tests create them, and a **gate before the first MONTH import** is a read-only read-back of the two codes (and of the pair's `MONTH` grain) in the target registry. If the codes differ, the allowlist is amended before release.
- **D3, what identifies "OAPEN downloads".** The coverage row's platform code `oapen` and measure code `downloads`, exactly the pair the Sphinx fence names (thoth-sphinx#74, MET-HIST-02 R2 section 3.3). The month is the row's own period. No source or account code is used.
- **D4, quarantine path.** The CloudFront quarantine-only exception keeps the strict equality test: it receives `strict_day`.
- **D5, ordering.** Unchanged; see "High-water mark".

**High-water mark.** `last_successful_period_end` stays a high-water mark (`GREATEST`). Thoth has never enforced import order for DAY either: a later day imported first moves the checkpoint past earlier unimported days. The fence against skipping is the driver's expected-start rule (H4 spec 3.5 and 4.1), and this change does not weaken it: after a MONTH unit the checkpoint is the 2nd, so the next discovered period is the first bundle period starting on or after the 2nd, that is the next month. A test pins the property (a later month moves the checkpoint past an unimported earlier month), so it is explicit rather than assumed.

**Replay and idempotency.** `successful` is computed by the same function on the live and the released side. A released replay returns the checkpoint when this import's progress is recorded (`last_completed_at` at or after the import's, and, for a successful import, `last_successful_period_end` at or after its period end) and `STALE_SOURCE_CLAIM` otherwise; it never writes and the cursor plays no part. A registry change after the import can change the judgement in either direction: removing MONTH from a pair can only make a replay judge the import unsuccessful, and a released replay then still returns the already recorded checkpoint; adding MONTH can turn an unsuccessful judgement into success, in which case a released replay before progress is recorded returns `STALE_SOURCE_CLAIM` (nothing written) and a fresh claim that re-runs `begin` (the terminal import is returned unchanged) and the update now advances. Both directions are tested.

**PARTIAL and later replacement.** A PARTIAL month that advanced the checkpoint is not rediscovered. A corrected month would be a new report id under the archive driver's rules (no replay mode today); Thoth would accept it (new import, `REVISION` outcomes), its coverage assertion would win by the coverage-run rule, and the cursor entry for that month would be replaced by the new manifest digest, which the driver's cursor-agreement check then compares with its bundle. That Sphinx-side path is the stated, signed risk of MET-HIST-02 R2 and is not widened or narrowed here.

**Thoth-side confirmation for thoth-sphinx#74 open point 1.** Verified in code at the base commit: the dashboard reads the derived coverage runs (`metric_dashboard/mod.rs` 2982 to 2996); per day the winning assertion is the first under `completed_at DESC, import_id DESC, coverage_status DESC` (`metric_coverage_run/mod.rs` 21 to 23); a PARTIAL status becomes `partial` and the `PARTIAL_COVERAGE` warning (`metric_dashboard/mod.rs` 5019 to 5031, 5638 to 5643); a run is clipped to the account window; and a section is COMPLETE only when every day is COMPLETE, UNKNOWN when any day is UNKNOWN, otherwise PARTIAL (UNKNOWN beats PARTIAL). Existing tests pin the pieces, none month-length: `metric_coverage_run/tests.rs` (a later import wins; a later COMPLETE re-coalesces over PARTIAL) and `metric_dashboard/tests.rs` (the warning). This task adds one month-length test (section 11) so the confirmation is evidence for a MONTH row.

### 7.2 Failure behaviour

- A row not accepted (any case of section 6 item 5) leaves the import unsuccessful: `last_completed_at` advances and the claim is released as for any non-successful terminal import; `last_successful_period_end` and the cursor do not move.
- Provably inconsistent quarantine evidence, an undecodable manifest envelope, an undecodable stored cursor, group counts that do not add up, or an unknown coverage status fail as `INTERNAL_STATE_INCONSISTENCY` with no write.
- A failing statement rolls the transaction back (existing behaviour).

### 7.3 Authorization

Unchanged: the mutation keeps its existing authorization and claim checks. No new operation. The existing negative authorization tests must keep passing unchanged.

### 7.4 Concurrency and idempotency

- The new coverage read is one grouped `SELECT` inside the existing transaction, after the existing locks, bounded by distinct combinations (an import holds at most 100 batch keys of 100 coverage rows each). It takes no lock: the registry rows change only through superuser mutations that lock one mapping row and wait on nothing the lifecycle holds, so there is no lock cycle and, under `READ COMMITTED`, one statement sees one snapshot.
- A concurrent registry change can flip the judgement either way (7.1, "Replay and idempotency"). Both outcomes are safe: no advance now (the driver's next run replays), or an advance judged on the registry state at that statement. The registry's MONTH grain for the pair is itself a gate before any MONTH import (D2).
- Two updates for one account still serialise on the checkpoint row. The existing race test must pass unchanged.

### 7.5 Compatibility

No API, schema or client change. Deployment before any MONTH import (section 12). The behaviour for CloudFront is identical by construction, proved by the unchanged existing tests plus the regression tests of section 11.

## 8. Data and migration requirements

Migration required: NO. `thoth-api/src/schema.rs` impact: none (no table or column changes). Existing checkpoints are untouched; no backfill. The first production effect is the first MONTH unit imported after deployment, under its own grant.

## 9. Observability and operations

Required logs: none new (the lifecycle writes none today).

Required metrics/alerts: none new.

Operational runbook changes: the MONTH import stages read back, after each unit, the account's checkpoint through the existing read-only `metricSourceCheckpoint` query (`lastSuccessfulPeriodEnd` equals the 2nd of the imported month) and the Sphinx summary's `next` (the 1st of the next month). The contract register paragraphs are updated accordingly.

## 10. Acceptance criteria

- [ ] A perfect MONTH unit (every row COMPLETE, exactly the calendar month starting on the envelope day, registry pair MONTH-capable, source not CloudFront) advances `last_successful_period_end` to the 2nd and records the cursor entry for the 1st; the claim is released.
- [ ] A CloudFront unit behaves exactly as before whatever the registry holds; the existing tests pass unchanged; a CloudFront source with a MONTH-capable pair and an exact month COMPLETE row does not advance and stays quarantine-ineligible.
- [ ] A PARTIAL row advances the checkpoint only for `oapen`/`downloads` and exactly 2026-01-01..2026-02-01 or 2026-08-01..2026-09-01; PARTIAL anywhere else (other month, other measure, other platform, a one-day PARTIAL inside an allowlisted month, a PARTIAL beside an UNKNOWN or another non-allowlisted PARTIAL row) does not.
- [ ] `COMPLETED_WITH_ERRORS` (including with an allowlisted PARTIAL row), zero rows, any `UNKNOWN`, a row of another month, a month starting on another day than the envelope's, a month not starting on the 1st never advance and never record a cursor entry.
- [ ] A coverage row whose registry pair row is missing is not accepted under rule B and does not change the verdict of rule-A rows.
- [ ] A released replay is read-only and returns the recorded checkpoint; a replay before the progress is recorded is `STALE_SOURCE_CLAIM`.
- [ ] Lock order, lease release and the single `UPDATE` are unchanged; no migration; the full workspace gate passes.

## 11. Required tests

### Unit (pure function, no database)

Table-driven test of the pure function: rule A cases unchanged; rule B COMPLETE for a 28, 29, 30 and 31 day month, December to January, a leap year; the envelope on the 1st versus not; a row of the previous month, the next month, two months wide, one day wide in a MONTH pair, shifted by one day at either end; `month_supported` false (including a missing pair row); `rule_b_enabled` false (the CloudFront case: rule B never evaluated); zero rows (`accepted` and `strict_day` both false); every PARTIAL case in section 10; UNKNOWN; code comparison exact (case, whitespace); group counts that do not sum to the total and an unknown status (inconsistent); the allowlist pin test (exactly the two entries); `strict_day` equals rule A alone.

### Integration/database

- The perfect MONTH unit of section 10 (checkpoint, cursor, lease), then a second MONTH unit for the next month (checkpoint moves to its 2nd, two cursor entries), then a released replay of the first (read-only); monthly cursor entries across the 64 bound (eviction by period, not insertion).
- A MONTH unit with two measures, both accepted; the same with one UNKNOWN or one non-allowlisted PARTIAL row (no advance); one unit mixing an allowlisted PARTIAL row with a COMPLETE row of another measure (accepted).
- The allowlisted PARTIAL unit advances; the same unit under another month, measure or platform does not (non-success path: progress and release, no cursor); the same unit as `COMPLETED_WITH_ERRORS`: no advance and no cursor.
- The pinned high-water property: month 3 imported while month 2 is not: the checkpoint is month 3's 2nd.
- Zero coverage rows: no advance.
- CloudFront: the existing quarantine-only tests unchanged; a CloudFront source on a MONTH-capable pair with an exact month COMPLETE row: no advance, quarantine stays ineligible; a quarantine-only unit with a month-shaped coverage row stays ineligible.
- Registry: delete the pair row after ingest: a rule B row is not accepted and a rule A row still is. Replay in both directions of 7.1 (MONTH removed from the pair after a successful import; MONTH added after an unsuccessful one).
- PARTIAL MONTH read (7.1): an allowlisted PARTIAL unit leaves one PARTIAL coverage run `[1st, next 1st)` (use the existing `coverage_runs` helper), and a later COMPLETE import for the same month leaves one COMPLETE run. The existing tests named in 7.1 are cited in the report, not repeated.
- Existing lock-order and race tests pass unchanged.

### Authorization/security

No new operation. The existing authorization tests for `updateMetricSourceCheckpoint` pass unchanged.

### Regression

The full `thoth-api` backend tests, with `metric_ingestion_lifecycle` and the GraphQL lifecycle tests named in the report.

### Manual verification

None required beyond the section 12 read-backs.

### Performance

The added read is bounded (7.4) and runs once per checkpoint update; no target beyond not adding a measurable statement time to the existing tests.

## 12. Rollout

- Initial state after merge: develop only. Merge is not release.
- Feature flag/configuration: none. The change is active wherever it is deployed, and affects only MONTH units of non-CloudFront sources (and PARTIAL units of the allowlist); no MONTH account is enabled until the H5d/H6/MET-HIST-02 grants.
- Release: its own grant (tag, image), with the next Thoth release. Deployment: its own grant, outside 01:42 to 06:30 UTC and while no Sphinx or archive writer runs.
- Staging/preview validation: the focused database tests are the evidence (there is no test database for imports). Read-backs after deployment: (1) the first scheduled CloudFront drain after deployment still advances each live account's `lastSuccessfulPeriodEnd` by one day (for each account, the value read before the drain plus one day); (2) V-MONTH and V-NATIVE remain clean.
- Gates before the first MONTH import (each in that stage's own grant): the registry codes and MONTH grain read-back (D2) and the emitted coverage-row shape evidence (section 5).
- Pilot: the first MONTH unit of the next stage, imported alone and read back before any further unit: for account X and month M, `metricSourceCheckpoint(X).lastSuccessfulPeriodEnd` equals M's 1st plus one day, and the Sphinx summary's `next` equals the 1st of the next month.
- Activation approval: none separate; the import stages carry their own grants.
- Observation period: the first MONTH import chunk, then each chunk (V-NATIVE after every chunk, as for all MONTH imports).

## 13. Rollback

- Code rollback: revert the PR and redeploy the previous image, each under its own grant. The behaviour returns to today's (MONTH held). The revert is the kill switch: there is no runtime switch, and exposure is nil until a MONTH import, which needs its own grant.
- Data rollback or forward repair: none needed for CloudFront. Checkpoints that MONTH units advanced before a revert stay advanced (they never move backwards, and an advanced checkpoint is correct for the data imported); resetting one is a production write under its own grant and is not planned. A false accept is irreversible (the driver would not rediscover the month), which is why the rules above fail closed.
- External side-effect handling: none.

## 14. Authorized write paths, new files and prohibited paths

Existing files this task may modify:

- `thoth-api/src/model/metric_ingestion_lifecycle/mod.rs`
- `thoth-api/src/model/metric_ingestion_lifecycle/tests.rs`
- `docs/metrics/contract-register.md`
- `CHANGELOG.md`

New files this task may create:

- `docs/engineering/ai-delivery/tasks/MET-HIST-01-CKPT.md` (this specification)
- `docs/engineering/ai-delivery/implementation-reports/MET-HIST-01-CKPT-implementation-report.md`
- `thoth-api/src/model/metric_ingestion_lifecycle/checkpoint_coverage.rs` (the pure function, the allowlist and their unit tests, if keeping them in `mod.rs` would be unwieldy; one `mod` line in `mod.rs` is then part of the allowed edit)

Prohibited paths: file deletion, move or rename is prohibited. Everything else is denied by default, including `thoth-api/src/schema.rs`, migrations, `thoth-api/src/graphql/**`, `thoth-client/**`, `thoth-api/src/model/metric_coverage_run/**` and every other repository.

## 15. Action authorization matrix

| Action | Authorized |
|---|---|
| repository/GitHub read inspection | YES |
| source/worktree edits within the write budget | YES |
| create the authorized new files | YES |
| delete/move/rename files | NO |
| branch creation from the exact authorized base | YES |
| commit | YES |
| push | YES |
| open/update draft PR | YES |
| issue/comment mutation | YES |
| manual CI dispatch/rerun | NO |
| provider/runtime read | NO |
| provider/runtime write | NO |
| migration execution | NO |
| release/tag/publication | NO |
| merge | NO |
| deployment | NO |
| production activation | NO |

Local disposable databases for the tests are not a provider or runtime.

## 16. Automatic side effects

Opening or updating a pull request that changes Rust source runs, at the base commit: `docker_build_and_push_to_dockerhub.yml` on `pull_request` (a `classify` job decides whether to build, and it fails open), which builds the image and pushes `ghcr.io/thoth-pub/thoth:staging-pr-<N>` to the GitHub container registry with the workflow's `GITHUB_TOKEN`; `run_migrations.yml`; and `check_changelog.yml`. The staging image is the only external write; no production system is touched. No manual dispatch or rerun is authorized.

## 17. HOLD/STOP conditions

The implementing agent must stop and report `HOLD` or `BLOCKED` if:

- the authorized base has moved and has not been freshly reconciled;
- the lifecycle module, the cursor or the registry tables changed on develop since the base in a way this design did not account for;
- an existing test would need a changed expectation (CloudFront and DAY must not change);
- a migration, a schema field or another path becomes necessary;
- approved architecture would need to change, or another repository needs source modification;
- a file deletion/move/rename appears necessary;
- scope cannot be completed without unrelated changes.

Gates that are not the implementer's to check, and that the grant for the first MONTH import must hold: the registry read-back of `oapen`, `downloads` and the pair's MONTH grain (D2), and the evidence of the emitted coverage-row shape (section 5). If either fails, the allowlist or rule B is amended through a specification update before release.

## 18. Expected implementation report

`docs/engineering/ai-delivery/implementation-report-template.md`, with the exact commands and results, the base and head commits, and the full workspace gate. The report also records that `metric_coverage_run/producer_manifest.txt` lists the lifecycle `mod.rs` (role `terminalization`), so the H1 coverage-run producer identity changes by construction; no test pins its digest.

## 19. Recommended execution

Implementation and review follow the CTO's standing instruction for this project of 2026-11-20: one implementation, one independent reviewer on the default model and effort, escalated only for security, data migrations, provider configuration or a CTO request. For a CRITICAL task this departs from `model-selection.md` section 6 and `operating-model.md` section 9.2 (strongest model at maximum effort, two independent fresh reviews, both approving the same head). The CTO's approval of this specification records the departure, and the CTO may require the full doctrine for this task instead. Merge needs a SHA-bound CTO merge authorization on the exact reviewed head and is never batched.

## 20. Branch and integration plan

- branch source: develop at the exact authorized base
- pull-request target: develop
- expected merge order: this PR alone
- parent programme branch refresh requirement: none
- branch deletion after merge: YES
- final programme PR required: NO
- final release path: develop -> master

## 21. Approval

Approved for implementation by: CTO
Date: 2026-11-24
Notes: The grant (comment 7128051136 on thoth-pub/thoth#1052) accepts the decisions D1 to D5 of section 7.1 and the review plan of section 19, including its departure from the CRITICAL doctrine.

Record only the durable implementation authorization here. Independent review decisions, CTO merge authorization and the merge itself are terminal GitHub evidence under `ADR-0005` and must not be copied back into this file.
