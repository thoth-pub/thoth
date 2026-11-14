# ROR-SYNC-01B: merging institutions that share a ROR ID

Stage ROR-SYNC-01B of thoth-pub/thoth#1002 (stage issue thoth-pub/thoth#1023). This folder holds the operator material for migration `20261114_v1.10.0`. Nothing here is run by `thoth migrate` or `thoth init`.

| File | What it is |
| --- | --- |
| `precheck.sql` | Read-only queries to run before upgrading: the foreign keys on `institution`, every group of rows that share a ROR ID with its references, survivor and deciding rule, and the works the merge will bump. |
| `repair.sql` | The forward repair: restores one merged institution, with its links, from its merge row. Runs only under its own grant. |

## Release notes text

This release merges institutions that share a ROR ID and makes the ROR ID unique.

- **What happens.** Migration `20261114_v1.10.0` finds every ROR ID held by more than one institution. In each group it keeps one institution (the survivor) and merges the others into it: their affiliations, fundings, book reviews, endorsements and history rows move to the survivor, one history row on the survivor records each merge, and the merged institutions are deleted. Then it creates the unique index `institution_ror_uniq_idx`. A create or update with a ROR ID that another institution already holds then fails with `An institution with this ROR ID already exists.`
- **Which institution survives.** The first rule that decides, each among the rows the earlier rules left tied: (0) the institution with Metrics references; (1) the one that holds a DOI; (2) the one with the most affiliations, fundings, book reviews and endorsements; (3) the oldest; (4) the lowest ID. The survivor's own name, DOI and country are not changed, so every moved link now exports the survivor's values.
- **No per-instance review.** On your installation the groups are merged by this rule without anyone reviewing them. Run `thoth-api/ops/ror-sync-01b/precheck.sql` first to see what will merge: query 1 lists the foreign keys on `institution`; query 2 lists every group, each row's references, Metrics references and DOI, the survivor, the deciding rule and whether the group is blocked; query 3 lists the works the merge will bump.
- **Blockers.** The migration aborts, changes nothing, and names every blocker with its remedy, if: the foreign keys on `institution` are not exactly the nine it expects (contact the maintainers before upgrading); two rows of a group hold a DOI (clear the DOI by hand on the row that should not survive); two rows of a group have Metrics references (contact the maintainers); or the Metrics-referenced row and the DOI holder differ (move the DOI by hand to the Metrics-referenced row, clearing it on the holder first, or contact the maintainers). A blocked group shows in query 2 with no survivor and no rule.
- **Single-instance installations** have no old version to fall back on when the upgrade's migration aborts. Run `thoth-api/ops/ror-sync-01b/precheck.sql` and clear any blocker before upgrading; otherwise start the previous image again to clear it.
- **Hold.** The migration takes `SHARE ROW EXCLUSIVE` on `institution` for its whole transaction, so institution creates, updates and deletes wait meanwhile; reads do not. The hold grows with the size of the seven tables that reference `institution` without a leading index (`affiliation`, `funding`, `institution_history` and the four Metrics tables), because deleting each merged row checks each of them. The migration bounds each lock wait at 10 s and each of its two statements at 60 s. If the upgrade fails in this migration with `canceling statement due to statement timeout`, keep the previous version (single-instance: start the previous image again) and report it; the maintainers will ship an index migration ahead of this one.
- **Temporary tables.** The migration creates two temporary tables. The `TEMPORARY` privilege on the database is granted to `PUBLIC` by default; where it has been revoked, the migration aborts before any write.
- **Works bumped.** Moving a link marks its work as updated, so those works enter the next metadata deposits.
- **Rollback.** A redeploy does not undo the merge. Each merged institution can be restored with `thoth-api/ops/ror-sync-01b/repair.sql`. Never run `thoth migrate --revert`, which reverts every migration.

## Forward repair

```sh
PGOPTIONS='-c ror_sync_01b.merge_row=<institution_history_id of the merge row>' \
  psql -X -v ON_ERROR_STOP=1 --single-transaction -f repair.sql "$DATABASE_URL"
```

The merge rows are the `institution_history` rows with `user_id = 'ROR-SYNC-01B'`. The script restores the merged institution with no ROR ID, moves back each listed link that still references the survivor, reports every listed link it did not move, and records a `ROR-SYNC-01B-REPAIR` history row. It refuses if the merged institution already exists.
