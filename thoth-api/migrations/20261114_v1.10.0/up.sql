-- ROR-SYNC-01B (thoth-pub/thoth#1023, programme #1002): merge the institutions
-- that share a ROR ID, then make the ROR ID unique.
--
-- Added: index `institution_ror_uniq_idx`. `idx_institution_ror` stays, now
-- redundant (dropping it needs ACCESS EXCLUSIVE; a later migration that
-- already takes that lock drops it). No new tables, columns, enums or
-- constraints.
--
-- Existing data: every non-null `ror` held by more than one row is a group.
-- In each group one row survives (the survivor rule below) and the others are
-- merged into it: their affiliations, fundings, book reviews, endorsements and
-- history rows are repointed to the survivor, one `institution_history` row
-- per merged row records the merge on the survivor, and the merged rows are
-- deleted. No survivor field changes. No link is deleted or nulled; a link
-- that duplicates another one of the same group is kept and listed in the
-- merge row. Defaults and nullability of every column are unchanged.
--
-- Survivor rule, per ROR ID, first rule that decides, each among the rows the
-- earlier rules left tied:
--   0. the row referenced by any of the four Metrics tables;
--   1. the row that holds an `institution_doi`;
--   2. the most affiliations, fundings, book reviews and endorsements;
--   3. the oldest `created_at`;
--   4. the lowest `institution_id`.
--
-- Validation before any write aborts the whole migration, naming every
-- blocker and its remedy, if: the foreign keys on `institution` are not
-- exactly the nine expected; two rows of a group have Metrics references; two
-- rows of a group hold a DOI; or rules 0 and 1 pick different rows.
--
-- Locking:
--   * SHARE ROW EXCLUSIVE on `institution` for the whole transaction:
--     institution inserts, updates and deletes wait; reads and foreign-key
--     checks against other institutions do not;
--   * FOR UPDATE on every row of every group: new references to them wait;
--   * row locks on the repointed rows and on the works their triggers bump;
--   * `lock_timeout` 10 s on every lock wait, row locks included;
--   * `statement_timeout` 60 s on the DO block and 60 s on the index build.
-- The hold is dominated by the referential checks of the DELETE: one scan of
-- each of the seven referencing tables without a leading index per deleted
-- row. A bound that fires fails the migration with nothing applied.
--
-- Temporary tables: `ror_sync_01b_map` and `ror_sync_01b_moved`, ON COMMIT
-- DROP. Where the TEMPORARY privilege has been revoked, the migration aborts
-- before any write.
--
-- Retry and idempotency: one transaction; a failure leaves nothing applied. A
-- re-run on a database without duplicates merges nothing and creates only the
-- index. A concurrent second copy waits on the table lock, then finds no
-- group and fails at CREATE UNIQUE INDEX (or on `lock_timeout`), and rolls
-- back.
--
-- No NOTICE is raised: migrations run with no notice handler. An abort's
-- message carries everything an operator needs.
--
-- Ordering: independent of ROR-SYNC-01C's migration.

SET LOCAL lock_timeout = '10s';
SET LOCAL statement_timeout = '60s';
LOCK TABLE institution IN SHARE ROW EXCLUSIVE MODE;

DO $ror_sync_01b$
DECLARE
    -- (schema.table.column.delete action): c cascade, n set null, a no action.
    expected_foreign_keys CONSTANT text[] := ARRAY[
        'public.affiliation.institution_id.c',
        'public.book_review.reviewer_institution_id.n',
        'public.endorsement.author_institution_id.n',
        'public.funding.institution_id.c',
        'public.institution_history.institution_id.c',
        'public.metric_record.institution_id.a',
        'public.metric_rollup_work_day.institution_id.a',
        'public.metric_rollup_work_institution_month.institution_id.a',
        'public.metric_rollup_work_institution_year.institution_id.a'
    ];
    unexpected_foreign_keys text[];
    missing_foreign_keys text[];
    group_row_ids uuid[];
    blockers text[] := ARRAY[]::text[];
    merged_ids uuid[] := ARRAY[]::uuid[];
    survivor_ids uuid[] := ARRAY[]::uuid[];
    merged_rors text[] := ARRAY[]::text[];
    survivor_rules smallint[] := ARRAY[]::smallint[];
    duplicate_affiliation_ids uuid[];
    duplicate_funding_ids uuid[];
    still_referencing text[];
    candidate record;
BEGIN
    -- Created before anything else, so that a revoked TEMPORARY privilege
    -- aborts before any write.
    CREATE TEMPORARY TABLE ror_sync_01b_map (
        merged_id uuid PRIMARY KEY,
        survivor_id uuid NOT NULL,
        ror text NOT NULL,
        survivor_rule smallint NOT NULL
    ) ON COMMIT DROP;
    CREATE TEMPORARY TABLE ror_sync_01b_moved (
        merged_id uuid NOT NULL,
        tbl text NOT NULL,
        row_id uuid NOT NULL
    ) ON COMMIT DROP;

    -- S3.1: the foreign keys on `institution`, compared as a multiset of
    -- (table, column, delete action), never by count or constraint name.
    WITH found AS (
        SELECT n.nspname::text || '.' || cl.relname::text || '.' || a.attname::text || '.' || c.confdeltype::text AS fk
        FROM pg_constraint c
        JOIN pg_class cl ON cl.oid = c.conrelid
        JOIN pg_namespace n ON n.oid = cl.relnamespace
        JOIN pg_attribute a ON a.attrelid = c.conrelid AND a.attnum = ANY (c.conkey)
        WHERE c.contype = 'f' AND c.confrelid = 'public.institution'::regclass
    )
    SELECT
        ARRAY(SELECT fk FROM found EXCEPT ALL SELECT unnest(expected_foreign_keys) ORDER BY 1),
        ARRAY(SELECT unnest(expected_foreign_keys) EXCEPT ALL SELECT fk FROM found ORDER BY 1)
    INTO unexpected_foreign_keys, missing_foreign_keys;
    IF cardinality(unexpected_foreign_keys) > 0 OR cardinality(missing_foreign_keys) > 0 THEN
        blockers := blockers || format(
            'The foreign keys on institution differ from the nine this migration expects (unexpected: %s; missing: %s). Remedy: contact the maintainers before upgrading.',
            coalesce(nullif(array_to_string(unexpected_foreign_keys, ', '), ''), 'none'),
            coalesce(nullif(array_to_string(missing_foreign_keys, ', '), ''), 'none'));
    END IF;

    -- S3.2: every non-null `ror` held by more than one row.
    SELECT coalesce(array_agg(institution_id ORDER BY institution_id), ARRAY[]::uuid[])
    INTO group_row_ids
    FROM institution
    WHERE ror IN (SELECT ror FROM institution WHERE ror IS NOT NULL GROUP BY ror HAVING count(*) > 1);

    -- S3.3: lock every row of every group, survivors included, in ascending
    -- `institution_id`. A new reference to one of them waits until commit.
    PERFORM 1 FROM institution WHERE institution_id = ANY (group_row_ids)
    ORDER BY institution_id FOR UPDATE;

    -- S3.4 and S3.5: references, one pass per referencing table for all
    -- groups, then the survivor rule and every blocker. The ranking is the one
    -- of the release notes' pre-check query 2.
    FOR candidate IN
        WITH grp AS (
          SELECT institution_id FROM institution WHERE institution_id = ANY (group_row_ids)
        ), refs AS (
          SELECT institution_id AS id, 'affiliation' AS t FROM affiliation WHERE institution_id IN (SELECT institution_id FROM grp)
          UNION ALL SELECT institution_id, 'funding' FROM funding WHERE institution_id IN (SELECT institution_id FROM grp)
          UNION ALL SELECT reviewer_institution_id, 'book_review' FROM book_review WHERE reviewer_institution_id IN (SELECT institution_id FROM grp)
          UNION ALL SELECT author_institution_id, 'endorsement' FROM endorsement WHERE author_institution_id IN (SELECT institution_id FROM grp)
          UNION ALL SELECT institution_id, 'metrics' FROM metric_record WHERE institution_id IN (SELECT institution_id FROM grp)
          UNION ALL SELECT institution_id, 'metrics' FROM metric_rollup_work_day WHERE institution_id IN (SELECT institution_id FROM grp)
          UNION ALL SELECT institution_id, 'metrics' FROM metric_rollup_work_institution_month WHERE institution_id IN (SELECT institution_id FROM grp)
          UNION ALL SELECT institution_id, 'metrics' FROM metric_rollup_work_institution_year WHERE institution_id IN (SELECT institution_id FROM grp)
        ), per_row AS (
          SELECT i.ror, i.institution_id, i.institution_doi, i.created_at,
                 count(r.t) FILTER (WHERE r.t = 'affiliation') AS affiliations,
                 count(r.t) FILTER (WHERE r.t = 'funding') AS fundings,
                 count(r.t) FILTER (WHERE r.t = 'book_review') AS book_reviews,
                 count(r.t) FILTER (WHERE r.t = 'endorsement') AS endorsements,
                 count(r.t) FILTER (WHERE r.t = 'metrics') AS metrics_references
          FROM institution i
          JOIN grp USING (institution_id)
          LEFT JOIN refs r ON r.id = i.institution_id
          GROUP BY i.institution_id
        ), ranked AS (
          SELECT p.*,
                 count(*) FILTER (WHERE p.metrics_references > 0) OVER g AS metrics_rows_in_group,
                 count(p.institution_doi) OVER g AS doi_holders_in_group,
                 bool_or(p.metrics_references > 0 AND p.institution_doi IS NOT NULL) OVER g AS one_row_has_both,
                 row_number() OVER s AS position,
                 first_value(p.institution_id) OVER s_all AS survivor_id,
                 nth_value(p.affiliations + p.fundings + p.book_reviews + p.endorsements, 1) OVER s_all AS references_1,
                 nth_value(p.affiliations + p.fundings + p.book_reviews + p.endorsements, 2) OVER s_all AS references_2,
                 nth_value(p.created_at, 1) OVER s_all AS created_1,
                 nth_value(p.created_at, 2) OVER s_all AS created_2,
                 string_agg(
                   p.institution_id::text
                     || CASE WHEN p.institution_doi IS NOT NULL THEN ' (holds DOI ' || p.institution_doi || ')' ELSE '' END
                     || CASE WHEN p.metrics_references > 0 THEN ' (has Metrics references)' ELSE '' END,
                   ', ') OVER g_all AS rows_described
          FROM per_row p
          WINDOW g AS (PARTITION BY p.ror),
                 g_all AS (PARTITION BY p.ror ORDER BY p.institution_id
                           ROWS BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING),
                 s AS (PARTITION BY p.ror
                       ORDER BY (p.metrics_references > 0) DESC,
                                (p.institution_doi IS NOT NULL) DESC,
                                p.affiliations + p.fundings + p.book_reviews + p.endorsements DESC,
                                p.created_at, p.institution_id),
                 s_all AS (s ROWS BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING)
        )
        SELECT d.*,
               (d.metrics_rows_in_group > 1
                OR d.doi_holders_in_group > 1
                OR (d.metrics_rows_in_group = 1 AND d.doi_holders_in_group = 1 AND NOT d.one_row_has_both)) AS group_blocked,
               CASE WHEN d.metrics_rows_in_group > 0 THEN 0
                    WHEN d.doi_holders_in_group > 0 THEN 1
                    WHEN d.references_1 > d.references_2 THEN 2
                    WHEN d.created_1 <> d.created_2 THEN 3
                    ELSE 4 END::smallint AS deciding_rule
        FROM ranked d
        ORDER BY d.ror, d.position
    LOOP
        IF candidate.group_blocked THEN
            IF candidate.position = 1 THEN
                IF candidate.metrics_rows_in_group > 1 THEN
                    blockers := blockers || format(
                        'ROR ID %s, rows %s: two rows have Metrics references. Remedy: contact the maintainers.',
                        candidate.ror, candidate.rows_described);
                END IF;
                IF candidate.doi_holders_in_group > 1 THEN
                    blockers := blockers || format(
                        'ROR ID %s, rows %s: two rows hold a DOI. Remedy: clear the DOI by hand on the row that should not survive.',
                        candidate.ror, candidate.rows_described);
                END IF;
                IF candidate.metrics_rows_in_group = 1 AND candidate.doi_holders_in_group = 1 AND NOT candidate.one_row_has_both THEN
                    blockers := blockers || format(
                        'ROR ID %s, rows %s: the row with Metrics references (rule 0) and the row holding the DOI (rule 1) differ. Remedy: move the DOI by hand to the Metrics-referenced row (clear it on the holder first, because the DOI index is unique), or contact the maintainers.',
                        candidate.ror, candidate.rows_described);
                END IF;
            END IF;
        ELSIF candidate.position > 1 THEN
            merged_ids := merged_ids || candidate.institution_id;
            survivor_ids := survivor_ids || candidate.survivor_id;
            merged_rors := merged_rors || candidate.ror;
            survivor_rules := survivor_rules || candidate.deciding_rule;
        END IF;
    END LOOP;

    IF cardinality(blockers) > 0 THEN
        RAISE EXCEPTION USING
            ERRCODE = 'P0001',
            MESSAGE = format(
                E'ROR-SYNC-01B cannot merge the institutions that share a ROR ID. Nothing was changed. %s blocker(s):\n%s',
                cardinality(blockers),
                array_to_string(blockers, E'\n'));
    END IF;

    IF cardinality(merged_ids) = 0 THEN
        RETURN;
    END IF;

    -- S5.1: one row per merged (non-surviving) row.
    INSERT INTO ror_sync_01b_map (merged_id, survivor_id, ror, survivor_rule)
    SELECT * FROM unnest(merged_ids, survivor_ids, merged_rors, survivor_rules);

    -- S5.2: duplicate links, for the whole group, before any repoint. They
    -- are kept and listed, never deleted.
    SELECT coalesce(array_agg(a.affiliation_id ORDER BY a.affiliation_id), ARRAY[]::uuid[])
    INTO duplicate_affiliation_ids
    FROM affiliation a
    JOIN ror_sync_01b_map m ON m.merged_id = a.institution_id
    WHERE EXISTS (
        SELECT 1
        FROM affiliation o
        JOIN institution g ON g.institution_id = o.institution_id
        WHERE o.contribution_id = a.contribution_id
          AND o.affiliation_id <> a.affiliation_id
          AND g.ror = m.ror);
    SELECT coalesce(array_agg(f.funding_id ORDER BY f.funding_id), ARRAY[]::uuid[])
    INTO duplicate_funding_ids
    FROM funding f
    JOIN ror_sync_01b_map m ON m.merged_id = f.institution_id
    WHERE EXISTS (
        SELECT 1
        FROM funding o
        JOIN institution g ON g.institution_id = o.institution_id
        WHERE o.work_id = f.work_id
          AND o.funding_id <> f.funding_id
          AND g.ror = m.ror);

    -- S5.3: repoint, one statement per table. The merge rows list exactly
    -- what RETURNING reports.
    WITH moved AS (
        UPDATE affiliation t SET institution_id = m.survivor_id
        FROM ror_sync_01b_map m WHERE t.institution_id = m.merged_id
        RETURNING m.merged_id, t.affiliation_id AS row_id)
    INSERT INTO ror_sync_01b_moved (merged_id, tbl, row_id)
    SELECT merged_id, 'affiliation', row_id FROM moved;

    WITH moved AS (
        UPDATE funding t SET institution_id = m.survivor_id
        FROM ror_sync_01b_map m WHERE t.institution_id = m.merged_id
        RETURNING m.merged_id, t.funding_id AS row_id)
    INSERT INTO ror_sync_01b_moved (merged_id, tbl, row_id)
    SELECT merged_id, 'funding', row_id FROM moved;

    WITH moved AS (
        UPDATE book_review t SET reviewer_institution_id = m.survivor_id
        FROM ror_sync_01b_map m WHERE t.reviewer_institution_id = m.merged_id
        RETURNING m.merged_id, t.book_review_id AS row_id)
    INSERT INTO ror_sync_01b_moved (merged_id, tbl, row_id)
    SELECT merged_id, 'book_review', row_id FROM moved;

    WITH moved AS (
        UPDATE endorsement t SET author_institution_id = m.survivor_id
        FROM ror_sync_01b_map m WHERE t.author_institution_id = m.merged_id
        RETURNING m.merged_id, t.endorsement_id AS row_id)
    INSERT INTO ror_sync_01b_moved (merged_id, tbl, row_id)
    SELECT merged_id, 'endorsement', row_id FROM moved;

    -- Decision P4 (CTO, 2026-11-13, "Move history, then delete"). The `data`
    -- of a moved row keeps the merged row's ID.
    WITH moved AS (
        UPDATE institution_history t SET institution_id = m.survivor_id
        FROM ror_sync_01b_map m WHERE t.institution_id = m.merged_id
        RETURNING m.merged_id, t.institution_history_id AS row_id)
    INSERT INTO ror_sync_01b_moved (merged_id, tbl, row_id)
    SELECT merged_id, 'institution_history', row_id FROM moved;

    -- S5.4: one merge row per merged row, on its survivor, in the existing
    -- shape of `data` (a JSON string).
    INSERT INTO institution_history (institution_id, user_id, data)
    SELECT m.survivor_id,
           'ROR-SYNC-01B',
           to_jsonb(jsonb_build_object(
               'merge', 'ROR-SYNC-01B',
               'ror', m.ror,
               'survivor_id', m.survivor_id,
               'survivor_rule', m.survivor_rule,
               'merged_row', to_jsonb(i),
               'affiliation_ids', coalesce((SELECT jsonb_agg(v.row_id ORDER BY v.row_id) FROM ror_sync_01b_moved v WHERE v.merged_id = m.merged_id AND v.tbl = 'affiliation'), '[]'::jsonb),
               'funding_ids', coalesce((SELECT jsonb_agg(v.row_id ORDER BY v.row_id) FROM ror_sync_01b_moved v WHERE v.merged_id = m.merged_id AND v.tbl = 'funding'), '[]'::jsonb),
               'book_review_ids', coalesce((SELECT jsonb_agg(v.row_id ORDER BY v.row_id) FROM ror_sync_01b_moved v WHERE v.merged_id = m.merged_id AND v.tbl = 'book_review'), '[]'::jsonb),
               'endorsement_ids', coalesce((SELECT jsonb_agg(v.row_id ORDER BY v.row_id) FROM ror_sync_01b_moved v WHERE v.merged_id = m.merged_id AND v.tbl = 'endorsement'), '[]'::jsonb),
               'institution_history_ids', coalesce((SELECT jsonb_agg(v.row_id ORDER BY v.row_id) FROM ror_sync_01b_moved v WHERE v.merged_id = m.merged_id AND v.tbl = 'institution_history'), '[]'::jsonb),
               'duplicate_affiliation_ids', coalesce((SELECT jsonb_agg(v.row_id ORDER BY v.row_id) FROM ror_sync_01b_moved v WHERE v.merged_id = m.merged_id AND v.tbl = 'affiliation' AND v.row_id = ANY (duplicate_affiliation_ids)), '[]'::jsonb),
               'duplicate_funding_ids', coalesce((SELECT jsonb_agg(v.row_id ORDER BY v.row_id) FROM ror_sync_01b_moved v WHERE v.merged_id = m.merged_id AND v.tbl = 'funding' AND v.row_id = ANY (duplicate_funding_ids)), '[]'::jsonb)
           )::text)
    FROM ror_sync_01b_map m
    JOIN institution i ON i.institution_id = m.merged_id;

    -- S5.5: nothing may still reference a merged row.
    SELECT array_remove(ARRAY[
        CASE WHEN EXISTS (SELECT 1 FROM affiliation WHERE institution_id IN (SELECT merged_id FROM ror_sync_01b_map)) THEN 'affiliation' END,
        CASE WHEN EXISTS (SELECT 1 FROM funding WHERE institution_id IN (SELECT merged_id FROM ror_sync_01b_map)) THEN 'funding' END,
        CASE WHEN EXISTS (SELECT 1 FROM institution_history WHERE institution_id IN (SELECT merged_id FROM ror_sync_01b_map)) THEN 'institution_history' END,
        CASE WHEN EXISTS (SELECT 1 FROM book_review WHERE reviewer_institution_id IN (SELECT merged_id FROM ror_sync_01b_map)) THEN 'book_review' END,
        CASE WHEN EXISTS (SELECT 1 FROM endorsement WHERE author_institution_id IN (SELECT merged_id FROM ror_sync_01b_map)) THEN 'endorsement' END,
        CASE WHEN EXISTS (SELECT 1 FROM metric_record WHERE institution_id IN (SELECT merged_id FROM ror_sync_01b_map)) THEN 'metric_record' END,
        CASE WHEN EXISTS (SELECT 1 FROM metric_rollup_work_day WHERE institution_id IN (SELECT merged_id FROM ror_sync_01b_map)) THEN 'metric_rollup_work_day' END,
        CASE WHEN EXISTS (SELECT 1 FROM metric_rollup_work_institution_month WHERE institution_id IN (SELECT merged_id FROM ror_sync_01b_map)) THEN 'metric_rollup_work_institution_month' END,
        CASE WHEN EXISTS (SELECT 1 FROM metric_rollup_work_institution_year WHERE institution_id IN (SELECT merged_id FROM ror_sync_01b_map)) THEN 'metric_rollup_work_institution_year' END
    ], NULL)
    INTO still_referencing;
    IF cardinality(still_referencing) > 0 THEN
        RAISE EXCEPTION 'ROR-SYNC-01B: % still reference a merged institution after the repoint. Nothing was changed.',
            array_to_string(still_referencing, ', ');
    END IF;

    -- S5.6
    DELETE FROM institution WHERE institution_id IN (SELECT merged_id FROM ror_sync_01b_map);

    -- S5.7
    IF EXISTS (SELECT 1 FROM institution WHERE ror IS NOT NULL GROUP BY ror HAVING count(*) > 1) THEN
        RAISE EXCEPTION 'ROR-SYNC-01B: a ROR ID is still held by more than one institution after the merge. Nothing was changed.';
    END IF;
END
$ror_sync_01b$;

-- S6: NULLs stay distinct, so any number of rows may have no ROR ID.
-- `institution_ror_check` already forces the lower-case https://ror.org/ form.
CREATE UNIQUE INDEX institution_ror_uniq_idx ON institution (ror);
