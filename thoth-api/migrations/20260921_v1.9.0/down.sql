-- MET-WP7-PREREQ-02 downgrade (issue #930): guarded, empty-table-only.
--
-- A quarantine row is the only durable copy of an unresolved CloudFront
-- observation's normalized value: provenance keeps its REJECTED / UNKNOWN_DOI
-- classification, but not the measure, period, country, value or methodology
-- a later reconciliation needs. Dropping a populated table would silently
-- erase that evidence. So this downgrade is deliberately NOT unconditional: it
-- fails closed while any row exists, and the recovery path after activation
-- is a forward fix under separate authorization, never a destructive rollback.
--
-- The table is locked ACCESS EXCLUSIVE before it is inspected, so no
-- concurrent insert can commit between the emptiness check and the drop. The
-- whole migration runs in one transaction, so a refusal leaves the table, its
-- constraints and every row exactly as they were.

LOCK TABLE public.metric_identifier_quarantine IN ACCESS EXCLUSIVE MODE;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM public.metric_identifier_quarantine) THEN
        RAISE EXCEPTION
            'MET-WP7-PREREQ-02 rollback refused: metric_identifier_quarantine holds % row(s) '
            'of durable unresolved-DOI evidence. Reversing this migration would discard '
            'observations no other table records. This requires a separately authorized '
            'forward repair, not a schema rollback.',
            (SELECT COUNT(*) FROM public.metric_identifier_quarantine);
    END IF;
END
$$;

-- The guard proved the table empty. Dropping it removes exactly this
-- migration's objects — the table, its primary key, unique key, four check
-- constraints and four foreign keys — and touches no predecessor object.
DROP TABLE public.metric_identifier_quarantine;
