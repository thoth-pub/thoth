-- ROR-SYNC-01B (thoth-pub/thoth#1023): forward repair of one merge.
--
-- Restores one institution that the ROR-SYNC-01B migration merged, from its
-- merge row in `institution_history` (user_id `ROR-SYNC-01B`). It lives outside
-- thoth-api/migrations/ so that the migration runner never embeds it, and it
-- runs only under its own grant.
--
-- Usage, as one transaction (the script holds no BEGIN or COMMIT of its own,
-- so that a dry check can run it inside a transaction it rolls back):
--
--   PGOPTIONS='-c ror_sync_01b.merge_row=<institution_history_id of the merge row>' \
--     psql -X -v ON_ERROR_STOP=1 --single-transaction -f repair.sql "$DATABASE_URL"
--
-- What it does, in that one transaction, with `lock_timeout` 10 s:
--   1. refuses if the merged row's `institution_id` exists in `institution`;
--   2. re-inserts the merged row from the merge row's `merged_row`, with an
--      explicit list of the columns present in it, and `ror` null (the unique
--      index forbids a second holder). `ror_status` and `ror_successor`
--      (ROR-SYNC-01C), whether present in `merged_row` or not, take their
--      defaults (`active`, null), as a row without a ROR ID requires; so does
--      any column added after the merge;
--   3. repoints each listed affiliation, funding, book review, endorsement and
--      history row back to the restored row, only if it still references the
--      survivor, and reports every listed ID it did not move, without
--      changing it;
--   4. inserts one `institution_history` row on the restored row, user_id
--      `ROR-SYNC-01B-REPAIR`, whose `data` names the merge row.
--
-- Without `--single-transaction` (or an enclosing transaction) the script
-- fails with `relation "ror_sync_01b_repair_report" does not exist` and
-- changes nothing. A second run in the same transaction starts from an empty
-- report.
--
-- The report is the result of the final SELECT: one row per listed ID, with
-- `outcome` `moved`, `missing` (the row no longer exists), `cleared` (it now
-- references no institution, as when the survivor was deleted after the
-- merge) or `elsewhere` (it now references another institution, named in
-- `now_references`).
--
-- The repointed links bump their works again (their AFTER triggers), which
-- counts against the Internet Archive bound of the repair grant.

SET LOCAL lock_timeout = '10s';

DROP TABLE IF EXISTS pg_temp.ror_sync_01b_repair_report;
CREATE TEMPORARY TABLE ror_sync_01b_repair_report (
    tbl text NOT NULL,
    row_id uuid NOT NULL,
    outcome text NOT NULL,
    now_references uuid
) ON COMMIT DROP;

DO $ror_sync_01b_repair$
DECLARE
    merge_history_id uuid;
    merge jsonb;
    merged_row jsonb;
    merged_id uuid;
    survivor uuid;
    restored_columns text;
    ids uuid[];
    target record;
BEGIN
    merge_history_id := nullif(current_setting('ror_sync_01b.merge_row', true), '')::uuid;
    IF merge_history_id IS NULL THEN
        RAISE EXCEPTION 'ROR-SYNC-01B repair: set ror_sync_01b.merge_row to the institution_history_id of the merge row';
    END IF;

    SELECT (h.data #>> '{}')::jsonb INTO merge
    FROM institution_history h
    WHERE h.institution_history_id = merge_history_id
      AND h.user_id = 'ROR-SYNC-01B';
    IF merge IS NULL OR merge ->> 'merge' IS DISTINCT FROM 'ROR-SYNC-01B' THEN
        RAISE EXCEPTION 'ROR-SYNC-01B repair: % is not a ROR-SYNC-01B merge row', merge_history_id;
    END IF;

    merged_row := merge -> 'merged_row';
    merged_id := (merged_row ->> 'institution_id')::uuid;
    survivor := (merge ->> 'survivor_id')::uuid;

    -- 1.
    IF EXISTS (SELECT 1 FROM institution WHERE institution_id = merged_id) THEN
        RAISE EXCEPTION 'ROR-SYNC-01B repair: institution % already exists; nothing was changed', merged_id;
    END IF;

    -- 2. Every current column that `merged_row` holds, except the ROR ID and
    -- the ROR state, which take their defaults.
    SELECT string_agg(quote_ident(a.attname), ', ' ORDER BY a.attnum)
    INTO restored_columns
    FROM pg_attribute a
    WHERE a.attrelid = 'public.institution'::regclass
      AND a.attnum > 0
      AND NOT a.attisdropped
      AND merged_row ? a.attname
      AND a.attname NOT IN ('ror', 'ror_status', 'ror_successor');
    EXECUTE format(
        'INSERT INTO institution (%1$s) SELECT %1$s FROM jsonb_populate_record(NULL::institution, $1)',
        restored_columns)
    USING merged_row;

    -- 3.
    FOR target IN
        SELECT * FROM (VALUES
            ('affiliation', 'affiliation_ids', 'affiliation_id', 'institution_id'),
            ('funding', 'funding_ids', 'funding_id', 'institution_id'),
            ('book_review', 'book_review_ids', 'book_review_id', 'reviewer_institution_id'),
            ('endorsement', 'endorsement_ids', 'endorsement_id', 'author_institution_id'),
            ('institution_history', 'institution_history_ids', 'institution_history_id', 'institution_id')
        ) AS t (tbl, list, pk, fk)
    LOOP
        SELECT coalesce(array_agg(value::uuid), ARRAY[]::uuid[])
        INTO ids
        FROM jsonb_array_elements_text(coalesce(merge -> target.list, '[]'::jsonb));

        EXECUTE format(
            'WITH moved AS ('
            '    UPDATE %1$I SET %3$I = $1 WHERE %2$I = ANY ($3) AND %3$I = $2 RETURNING %2$I AS row_id'
            ') '
            'INSERT INTO ror_sync_01b_repair_report (tbl, row_id, outcome, now_references) '
            'SELECT %4$L, row_id, ''moved'', $1 FROM moved',
            target.tbl, target.pk, target.fk, target.tbl)
        USING merged_id, survivor, ids;

        EXECUTE format(
            'INSERT INTO ror_sync_01b_repair_report (tbl, row_id, outcome, now_references) '
            'SELECT %1$L, listed.id, '
            '       CASE WHEN t.%2$I IS NULL THEN ''missing'' '
            '            WHEN t.%3$I IS NULL THEN ''cleared'' '
            '            ELSE ''elsewhere'' END, t.%3$I '
            'FROM unnest($1::uuid[]) AS listed (id) '
            'LEFT JOIN %4$I t ON t.%2$I = listed.id '
            'WHERE NOT EXISTS (SELECT 1 FROM ror_sync_01b_repair_report r '
            '                  WHERE r.tbl = %1$L AND r.row_id = listed.id)',
            target.tbl, target.pk, target.fk, target.tbl)
        USING ids;
    END LOOP;

    -- 4.
    INSERT INTO institution_history (institution_id, user_id, data)
    SELECT merged_id,
           'ROR-SYNC-01B-REPAIR',
           to_jsonb(jsonb_build_object(
               'repair', 'ROR-SYNC-01B-REPAIR',
               'merge_row', merge_history_id,
               'ror', merge ->> 'ror',
               'survivor_id', survivor,
               'restored_institution_id', merged_id,
               'moved', coalesce((SELECT jsonb_agg(jsonb_build_object('table', r.tbl, 'id', r.row_id) ORDER BY r.tbl, r.row_id)
                                  FROM ror_sync_01b_repair_report r WHERE r.outcome = 'moved'), '[]'::jsonb),
               'not_moved', coalesce((SELECT jsonb_agg(jsonb_build_object('table', r.tbl, 'id', r.row_id, 'outcome', r.outcome, 'now_references', r.now_references) ORDER BY r.tbl, r.row_id)
                                      FROM ror_sync_01b_repair_report r WHERE r.outcome <> 'moved'), '[]'::jsonb)
           )::text);
END
$ror_sync_01b_repair$;

SELECT tbl, row_id, outcome, now_references
FROM ror_sync_01b_repair_report
ORDER BY tbl, outcome, row_id;
