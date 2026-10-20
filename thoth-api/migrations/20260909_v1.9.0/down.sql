-- MET-WP2-01A downgrade (issue #899).
--
-- MET-WP2-01A is cleanly reversible only while its new persistence contract
-- is still inactive. Once MET-WP2-01B has committed batches under it, or
-- administration has assigned real stable source-account codes, reversing
-- this migration would destroy durable ingestion evidence or stable identity
-- that nothing else records.
--
-- So this downgrade is deliberately NOT unconditional. It fails closed on
-- each unsafe state rather than deleting its way to success, exactly as
-- Specification Amendment 1 section A5 requires. The recovery path after
-- activation is a forward fix under separate authorization, never a
-- destructive schema rollback.
--
-- Every guard is evaluated before any object is dropped, and the whole
-- migration runs in one transaction, so a raised exception leaves the 01A
-- contract fully intact.

DO $$
BEGIN
    -- Guard 1: any committed batch means MET-WP2-01B ingestion evidence
    -- exists. Dropping metric_import_batch would orphan the ordered
    -- provenance replay contract that depends on it.
    IF EXISTS (SELECT 1 FROM public.metric_import_batch) THEN
        RAISE EXCEPTION
            'MET-WP2-01A rollback refused: metric_import_batch holds % row(s) '
            'of committed ingestion evidence. Reversing this migration would '
            'discard durable batch identity and its ordered provenance '
            'linkage. This requires a separately authorized reconciliation '
            'decision, not a schema rollback.',
            (SELECT COUNT(*) FROM public.metric_import_batch);
    END IF;

    -- Guard 2: a NULL hash can only have been written under the amended
    -- REJECTED rules. Restoring the pre-01A NOT NULL contract would have to
    -- invent or delete those values.
    IF EXISTS (
        SELECT 1 FROM public.metric_record_provenance
         WHERE identity_hash IS NULL OR content_hash IS NULL
    ) THEN
        RAISE EXCEPTION
            'MET-WP2-01A rollback refused: % provenance row(s) carry a NULL '
            'identity_hash or content_hash recorded under the amended '
            'REJECTED invariant. Restoring the pre-01A NOT NULL contract '
            'would require fabricating or destroying rejection evidence.',
            (SELECT COUNT(*) FROM public.metric_record_provenance
              WHERE identity_hash IS NULL OR content_hash IS NULL);
    END IF;

    -- Guard 3: a code that is no longer its deterministic migration backfill
    -- value is a real stable identity someone assigned. Dropping the column
    -- would lose it, and nothing else in the schema records it.
    IF EXISTS (
        SELECT 1 FROM public.metric_source_account
         WHERE code IS DISTINCT FROM source_account_id::text
    ) THEN
        RAISE EXCEPTION
            'MET-WP2-01A rollback refused: % metric_source_account row(s) '
            'carry a stable code that differs from the deterministic '
            'migration backfill value source_account_id::text. Those codes '
            'are assigned identity and would be permanently lost.',
            (SELECT COUNT(*) FROM public.metric_source_account
              WHERE code IS DISTINCT FROM source_account_id::text);
    END IF;
END
$$;


-- Every guard is clear, so the 01A contract is still inactive and can be
-- removed exactly. Each statement below reverses exactly one 01A change and
-- touches nothing else: no MET-WP1-01..12 table, enum, constraint, index or
-- row, and no bibliographic object.

-- D. The one overlap-supporting index.
DROP INDEX public.metric_record_overlap_lookup_idx;

-- C. Provenance batch linkage, then the pre-01A hash contract.
--
-- The composite foreign key is dropped before its referenced table, and the
-- batch columns are dropped after the constraints that mention them.
ALTER TABLE public.metric_record_provenance
    DROP CONSTRAINT metric_record_provenance_import_batch_fkey,
    DROP CONSTRAINT metric_record_provenance_import_batch_id_batch_row_index_key,
    DROP CONSTRAINT metric_record_provenance_batch_row_index_check,
    DROP CONSTRAINT metric_record_provenance_batch_link_check,
    DROP CONSTRAINT metric_record_provenance_classification_hash_check,
    DROP COLUMN import_batch_id,
    DROP COLUMN batch_row_index;

-- Guard 2 proved no NULL hash exists, so this restores the original contract
-- without rewriting a single value. It is also a second line of defence: if
-- the guard were ever bypassed, this would still fail rather than corrupt.
ALTER TABLE public.metric_record_provenance
    ALTER COLUMN identity_hash SET NOT NULL,
    ALTER COLUMN content_hash SET NOT NULL;

-- B. The batch ledger, proved empty by guard 1.
DROP TABLE public.metric_import_batch;

-- A. The stable source-account code, proved to hold only deterministic
-- migration backfill values by guard 3.
--
-- The existing UNIQUE (source_id, external_key) identity, the external_key
-- nonblank CHECK and all three foreign keys are untouched, so the table
-- returns to exactly its pre-01A contract.
ALTER TABLE public.metric_source_account
    DROP CONSTRAINT metric_source_account_code_key,
    DROP CONSTRAINT metric_source_account_code_check,
    DROP COLUMN code;
