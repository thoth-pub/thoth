-- MET-WP7-PREREQ-03 downgrade (#935): guarded, empty-state-only.
--
-- Reconciliation state is durable audit evidence. Once any attempt exists this
-- migration may only be reversed by a separately authorized forward repair.

LOCK TABLE public.metric_identifier_quarantine_reconciliation IN ACCESS EXCLUSIVE MODE;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM public.metric_identifier_quarantine_reconciliation
    ) THEN
        RAISE EXCEPTION
            'MET-WP7-PREREQ-03 rollback refused: metric_identifier_quarantine_reconciliation contains durable reconciliation state. Use a separately authorized forward repair.';
    END IF;
END
$$;

DROP TABLE public.metric_identifier_quarantine_reconciliation;
DROP INDEX public.metric_identifier_quarantine_created_id_idx;
DROP TYPE public.metric_identifier_quarantine_reconciliation_state;
