-- MET-WP4-01 downgrade: guarded PRE-ACTIVATION rollback only.
--
-- This is deliberately not a general reverse of rollup application. Once a
-- delta has been claimed or applied, or once a projection row exists, the
-- new columns and tables hold the only durable evidence of which canonical
-- accounting has already been projected. Dropping them would not undo the
-- projection; it would erase the record of it, and a later reapplication
-- would renumber the surviving deltas from 1 and reapply work that had
-- already been applied.
--
-- The rollback boundary is therefore explicit (Specification Amendment 2
-- section 14): destructive removal is permitted only while nothing has been
-- claimed, applied or projected. After activation, repair is forward repair
-- or rebuild from canonical records and durable deltas, under its own
-- separate operational authorization. It is not this file.
--
-- Discarding unallocated positions in the permitted case is safe precisely
-- because no projection, claim or watermark effect depends on them: the
-- surviving PENDING rows are restored to their original MET-WP1-07 shape,
-- and reapplying the migration backfills them deterministically again from
-- `(created_at, delta_id)`.

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM public.metric_rollup_work_day_state
        WHERE applied_through_sequence <> 0
    ) THEN
        RAISE EXCEPTION
            'Refusing to remove MET-WP4-01 rollup state: the durable watermark has advanced past sequence 0. Applied rollup progress must be repaired forward, not erased by a down migration.';
    END IF;

    IF EXISTS (
        SELECT 1
        FROM public.metric_rollup_delta
        WHERE status <> 'PENDING'
           OR claim_token IS NOT NULL
           OR claimed_by IS NOT NULL
           OR claimed_at IS NOT NULL
           OR lease_expires_at IS NOT NULL
           OR applied_at IS NOT NULL
    ) THEN
        RAISE EXCEPTION
            'Refusing to remove MET-WP4-01 rollup state: one or more metric_rollup_delta rows carry claim or application evidence. Resolve the outstanding claims and applied progress first; this rollback must not erase them.';
    END IF;

    IF EXISTS (SELECT 1 FROM public.metric_rollup_work_day) THEN
        RAISE EXCEPTION
            'Refusing to remove MET-WP4-01 rollup state: the metric_rollup_work_day projection is not empty. Rebuild or repair it forward under separate authorization rather than dropping it here.';
    END IF;
END $$;

DROP TRIGGER IF EXISTS metric_rollup_delta_assign_work_day_sequence
    ON public.metric_rollup_delta;
DROP FUNCTION IF EXISTS public.metric_rollup_delta_assign_work_day_sequence();

DROP TABLE IF EXISTS public.metric_rollup_work_day;
DROP TABLE IF EXISTS public.metric_rollup_work_day_state;

DROP INDEX IF EXISTS public.metric_rollup_delta_claim_token_idx;
DROP INDEX IF EXISTS public.metric_rollup_delta_frontier_idx;
DROP INDEX IF EXISTS public.metric_rollup_delta_work_day_sequence_idx;

ALTER TABLE public.metric_rollup_delta
    DROP CONSTRAINT IF EXISTS metric_rollup_delta_claim_state_check,
    DROP CONSTRAINT IF EXISTS metric_rollup_delta_work_day_sequence_check;

-- The MET-WP1-07 columns — delta_id, record_id, revision_id, delta_value,
-- status, created_at and applied_at — and every durable PENDING row survive
-- untouched, as do the canonical MET-WP1-04 record/revision tables and the
-- composite foreign key between them.
ALTER TABLE public.metric_rollup_delta
    DROP COLUMN IF EXISTS lease_expires_at,
    DROP COLUMN IF EXISTS claimed_at,
    DROP COLUMN IF EXISTS claimed_by,
    DROP COLUMN IF EXISTS claim_token,
    DROP COLUMN IF EXISTS work_day_sequence;
