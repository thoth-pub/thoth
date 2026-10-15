-- MET-WP4-01: MOM-1 rollup projection and delta application (issue #909).
--
-- Additive and initially inactive. Turns the MET-WP1-07 `metric_rollup_delta`
-- foundation into an applicable work-day progress stream, and creates the one
-- MOM-1 derived projection `metric_rollup_work_day` plus the Metrics-local
-- singleton progress row that owns its gap-free ordering and its durable safe
-- watermark.
--
-- Canonical boundary (reviewed): canonical metric records, revisions and
-- provenance remain authoritative and are neither read-modified nor repaired
-- here. `metric_rollup_work_day` is derived, rebuildable state. No canonical
-- row is changed by this migration and no projection row is created by it.
--
-- MOM-1 scope decision (reviewed, Specification Amendment 2 section 2): the
-- ordered progress stream covers exactly those rollup deltas whose canonical
-- record is a valid one-day DAY record, i.e.
-- `reporting_grain = 'DAY' AND period_end = period_start + 1`. Deltas of any
-- other grain stay PENDING with a NULL `work_day_sequence`; they are outside
-- the MOM-1 frontier and must never block it. The monthly,
-- work-country-month and work-institution-month projections named by the
-- approved Metrics design remain deferred and are NOT created here.
--
-- Ordering decision (reviewed, Amendment 2 section 3): a PostgreSQL sequence
-- or BIGSERIAL is deliberately NOT the correctness identity. Sequence
-- allocation is non-transactional, so a rolled-back ingestion transaction
-- would leave a permanent hole and the contiguous-frontier watermark could
-- never close it. Instead one singleton counter row is updated inside the
-- caller's own transaction, so a rolled-back ingestion also rolls back the
-- allocation and committed work-day positions are gap-free from 1 upward.
--
-- Concurrency effect of that decision (reviewed): the BEFORE INSERT trigger
-- below takes a row lock on the singleton state row, so two ingestion
-- transactions that each commit a work-day delta serialize against each other
-- for the remainder of their transactions, and an in-flight rollup claim or
-- completion (which locks the same row FOR UPDATE) blocks work-day delta
-- allocation until it commits. That is the intended MOM-1 trade: strict
-- global ordering of a low-volume accounting stream in exchange for a
-- gap-free frontier. Ingestion transactions acquire this lock last, after the
-- publisher and registry locks they already take, so no new lock-ordering
-- cycle is introduced. Non-work-day ingestion never touches the row at all.

-- Exclude concurrent delta writers for the guard, backfill and constraint
-- window. SHARE ROW EXCLUSIVE conflicts with the ROW EXCLUSIVE lock every
-- INSERT/UPDATE/DELETE takes, so no delta can be created or changed between
-- the fail-closed guard below and the closed lifecycle constraint installed
-- further down. Ordinary ACCESS SHARE reads are not blocked by this mode, and
-- the canonical tables are deliberately not locked: a canonical write that
-- does not create a delta cannot invalidate anything asserted here.
LOCK TABLE public.metric_rollup_delta IN SHARE ROW EXCLUSIVE MODE;

-- Fail-closed pre-WP4 state guard (reviewed, Amendment 2 section 4). No
-- approved pre-WP4 runtime claims or applies a delta: MET-WP1-07 created the
-- table with no claim protocol, and the merged MET-WP2-01B coordinator only
-- ever inserts PENDING rows with a NULL applied_at. Unexpected state is
-- therefore evidence requiring investigation, not data to coerce, so the
-- migration refuses rather than inventing a interpretation of it.
DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM public.metric_rollup_delta
        WHERE status <> 'PENDING'
           OR applied_at IS NOT NULL
    ) THEN
        RAISE EXCEPTION
            'Cannot activate MET-WP4-01 rollup application: one or more metric_rollup_delta rows are already non-PENDING or carry applied_at. Investigate that state before migrating; this migration will not reinterpret it.';
    END IF;
END $$;

-- Fail-closed canonical-shape guard (reviewed, Amendment 2 section 4 step 2).
-- A record that claims DAY grain but does not span exactly one calendar day
-- is canonically inconsistent. The MOM-1 projection keys on `day =
-- period_start`, so silently treating such a record as a work-day delta would
-- attribute a multi-day total to a single day.
DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM public.metric_rollup_delta d
        JOIN public.metric_record r ON r.record_id = d.record_id
        WHERE r.reporting_grain = 'DAY'
          AND r.period_end <> r.period_start + 1
    ) THEN
        RAISE EXCEPTION
            'Cannot activate MET-WP4-01 rollup application: one or more metric_rollup_delta rows reference a DAY-grain metric_record whose period is not exactly one calendar day. Resolve that canonical inconsistency before migrating.';
    END IF;
END $$;

-- ---------------------------------------------------------------------------
-- 1. Durable claim, lease and work-day progress state on metric_rollup_delta
-- ---------------------------------------------------------------------------
--
-- All five columns are nullable: a PENDING non-work-day delta legitimately
-- carries none of them, and a PENDING work-day delta carries only the
-- sequence. The closed lifecycle CHECK installed below is what makes each
-- combination exact; NOT NULL could not express a per-status rule.
--
-- `claimed_by` is the authenticated machine principal derived by Thoth, never
-- a caller-supplied identity. No retry-count, backoff, next-attempt or
-- generic job column is introduced: MOM-1 blocks a poison frontier for
-- separately authorized repair rather than rescheduling around it.
ALTER TABLE public.metric_rollup_delta
    ADD COLUMN work_day_sequence bigint,
    ADD COLUMN claim_token uuid,
    ADD COLUMN claimed_by text,
    ADD COLUMN claimed_at timestamp with time zone,
    ADD COLUMN lease_expires_at timestamp with time zone;

-- Deterministic backfill of the existing PENDING work-day deltas
-- (Amendment 2 section 4 step 3). Ordering is `(created_at, delta_id)`:
-- `created_at` is the durable arrival order and `delta_id` makes a tie
-- impossible, so the assignment is reproducible from the data alone and a
-- reapplied migration produces the same positions. Non-day deltas are left
-- NULL by the WHERE clause rather than by an exception.
WITH ordered AS (
    SELECT d.delta_id,
           row_number() OVER (ORDER BY d.created_at, d.delta_id) AS position
    FROM public.metric_rollup_delta d
    JOIN public.metric_record r ON r.record_id = d.record_id
    WHERE r.reporting_grain = 'DAY'
      AND r.period_end = r.period_start + 1
)
UPDATE public.metric_rollup_delta d
SET work_day_sequence = ordered.position
FROM ordered
WHERE d.delta_id = ordered.delta_id;

-- The closed MOM-1 lifecycle. Any other status value, and any field
-- combination that does not match its status exactly, is rejected by the
-- database rather than by an application check.
--
-- APPLIED deliberately retains `claim_token`, `claimed_by` and `claimed_at`
-- as terminal claim evidence: the lease is closed by nulling
-- `lease_expires_at`, not by erasing who applied the delta and under which
-- batch. That retained token is also what makes an identical
-- timeout-after-commit completion replay recognisable as already applied
-- instead of stale.
ALTER TABLE public.metric_rollup_delta
    ADD CONSTRAINT metric_rollup_delta_work_day_sequence_check
        CHECK (work_day_sequence IS NULL OR work_day_sequence > 0),
    ADD CONSTRAINT metric_rollup_delta_claim_state_check CHECK (
        (status = 'PENDING'
            AND claim_token IS NULL
            AND claimed_by IS NULL
            AND claimed_at IS NULL
            AND lease_expires_at IS NULL
            AND applied_at IS NULL)
        OR (status = 'CLAIMED'
            AND work_day_sequence IS NOT NULL
            AND claim_token IS NOT NULL
            AND claimed_by IS NOT NULL
            AND claimed_by ~ '[^[:space:]]'
            AND claimed_at IS NOT NULL
            AND lease_expires_at IS NOT NULL
            AND applied_at IS NULL)
        OR (status = 'APPLIED'
            AND work_day_sequence IS NOT NULL
            AND claim_token IS NOT NULL
            AND claimed_by IS NOT NULL
            AND claimed_by ~ '[^[:space:]]'
            AND claimed_at IS NOT NULL
            AND lease_expires_at IS NULL
            AND applied_at IS NOT NULL)
    );

-- Exactly the three approved indexes (Amendment 2 section 5).
--
-- The uniqueness index is partial because non-day deltas share a NULL
-- sequence; it is what makes a work-day position unrepeatable. The frontier
-- index covers only non-APPLIED work-day rows, which is the exact set the
-- claim scan walks from `applied_through_sequence + 1`. The claim-token index
-- is the completion path's only lookup. No speculative access path is added.
CREATE UNIQUE INDEX metric_rollup_delta_work_day_sequence_idx
    ON public.metric_rollup_delta (work_day_sequence)
    WHERE work_day_sequence IS NOT NULL;

CREATE INDEX metric_rollup_delta_frontier_idx
    ON public.metric_rollup_delta (work_day_sequence)
    WHERE work_day_sequence IS NOT NULL AND status <> 'APPLIED';

CREATE INDEX metric_rollup_delta_claim_token_idx
    ON public.metric_rollup_delta (claim_token)
    WHERE claim_token IS NOT NULL;

-- ---------------------------------------------------------------------------
-- 2. The singleton work-day progress and watermark state
-- ---------------------------------------------------------------------------
--
-- One row, forever, enforced by the primary key plus `state_id = 1`. It owns
-- the next position to allocate, the contiguous applied-through frontier and
-- the wall-clock time at which that frontier was established.
--
-- `applied_through_sequence < next_sequence` is the structural statement that
-- the frontier can never claim to have applied a position that was never
-- allocated. `updated_at` is maintained explicitly by the allocating trigger
-- and by the completion transaction rather than by the repository-standard
-- `diesel_manage_updated_at` helper, because the approved contract fixes both
-- it and `watermark_at` to the same transaction timestamp as the state change
-- that justified them; a second trigger would be an unapproved object that
-- could only ever agree with, or silently override, that value.
CREATE TABLE public.metric_rollup_work_day_state (
    state_id smallint NOT NULL,
    next_sequence bigint NOT NULL,
    applied_through_sequence bigint NOT NULL,
    watermark_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    CONSTRAINT metric_rollup_work_day_state_pkey PRIMARY KEY (state_id),
    CONSTRAINT metric_rollup_work_day_state_singleton_check CHECK (state_id = 1),
    CONSTRAINT metric_rollup_work_day_state_next_sequence_check CHECK (next_sequence >= 1),
    CONSTRAINT metric_rollup_work_day_state_applied_through_sequence_check
        CHECK (applied_through_sequence >= 0),
    CONSTRAINT metric_rollup_work_day_state_frontier_check
        CHECK (applied_through_sequence < next_sequence)
);

-- Initialization (Amendment 2 section 4 steps 5 to 8). `next_sequence`
-- continues after the highest backfilled position, the frontier starts at 0
-- because this migration applies nothing, and both timestamps record when the
-- sequence-0 frontier was established. That timestamp is explicitly NOT a
-- claim that any pending canonical data has been projected: rollup lag is
-- detected by comparing the frontier with `next_sequence - 1`.
INSERT INTO public.metric_rollup_work_day_state
    (state_id, next_sequence, applied_through_sequence, watermark_at, updated_at)
SELECT 1,
       COALESCE(MAX(work_day_sequence), 0) + 1,
       0,
       transaction_timestamp(),
       transaction_timestamp()
FROM public.metric_rollup_delta;

-- ---------------------------------------------------------------------------
-- 3. Transactional, gap-free work-day sequence allocation
-- ---------------------------------------------------------------------------
--
-- This trigger exists solely so the already-merged MET-WP2-01B insertion
-- boundary keeps working unchanged: neither Sphinx nor the ingestion
-- coordinator allocates rollup progress itself, and neither may. It is not a
-- generic queue, sequence or job framework, it moves no delta between
-- statuses, and it is installed on exactly one table.
--
-- A caller-supplied `work_day_sequence` is refused outright. Accepting one
-- would let a writer choose its own position in the ordering that the
-- exactly-once frontier depends on.
--
-- The `UPDATE ... RETURNING next_sequence - 1` reads back the pre-increment
-- value under the row lock the UPDATE itself takes, so two concurrent
-- allocations cannot observe the same position, and neither can commit its
-- delta without committing its increment.
CREATE FUNCTION public.metric_rollup_delta_assign_work_day_sequence()
    RETURNS trigger
    LANGUAGE plpgsql AS $$
DECLARE
    is_work_day boolean;
    allocated bigint;
BEGIN
    IF NEW.work_day_sequence IS NOT NULL THEN
        RAISE EXCEPTION
            'metric_rollup_delta.work_day_sequence is allocated by Thoth and must not be supplied'
            USING ERRCODE = 'restrict_violation',
                  CONSTRAINT = 'metric_rollup_delta_work_day_sequence_is_allocated';
    END IF;

    SELECT r.reporting_grain = 'DAY' AND r.period_end = r.period_start + 1
      INTO is_work_day
      FROM public.metric_record r
     WHERE r.record_id = NEW.record_id;

    -- A missing canonical record is left to the composite foreign key, which
    -- rejects the row after this BEFORE trigger returns. Allocating a
    -- position for it first would burn one.
    IF NOT COALESCE(is_work_day, false) THEN
        RETURN NEW;
    END IF;

    UPDATE public.metric_rollup_work_day_state
       SET next_sequence = next_sequence + 1,
           updated_at = transaction_timestamp()
     WHERE state_id = 1
    RETURNING next_sequence - 1 INTO allocated;

    IF allocated IS NULL THEN
        RAISE EXCEPTION
            'the metric_rollup_work_day_state singleton row is missing: work-day rollup progress cannot be allocated'
            USING ERRCODE = 'no_data_found';
    END IF;

    NEW.work_day_sequence := allocated;
    RETURN NEW;
END;
$$;

CREATE TRIGGER metric_rollup_delta_assign_work_day_sequence
    BEFORE INSERT ON public.metric_rollup_delta
    FOR EACH ROW
    EXECUTE FUNCTION public.metric_rollup_delta_assign_work_day_sequence();

-- ---------------------------------------------------------------------------
-- 4. The one MOM-1 derived projection
-- ---------------------------------------------------------------------------
--
-- Work-level derived state. Publisher, imprint and series attribution is
-- deliberately not persisted here: it continues to be resolved through
-- current Thoth metadata, so a later imprint move does not need a rollup
-- rewrite.
--
-- Uniqueness decision (reviewed, Amendment 2 section 6): the logical
-- aggregate identity includes three optional dimensions. Ordinary SQL
-- uniqueness treats every NULL as distinct, so a plain UNIQUE would permit
-- unlimited duplicate rows for the same "no publication, no country, no
-- institution" aggregate and the projection would silently split a total
-- across them. `UNIQUE NULLS NOT DISTINCT` (PostgreSQL 15+) is what makes the
-- absent dimension a value for uniqueness purposes, and it is what the
-- application upsert conflict target resolves against.
--
-- `value` is a signed BIGINT with no non-negative rule: a revision applies
-- `new - old`, which is negative whenever a source corrects a figure
-- downwards. PostgreSQL's BIGINT arithmetic is itself checked, so an
-- overflowing application aborts the whole completion transaction instead of
-- wrapping.
--
-- `watermark` is an ordered work-day progress position, never a wall-clock
-- time and never an independent serving boundary: it records the greatest
-- sequence that has actually changed this row. The authoritative safe serving
-- boundary is `metric_rollup_work_day_state.applied_through_sequence`.
--
-- Foreign keys are non-cascading, matching every other Metrics key: deleting
-- a work, publication, platform, measure or institution that still has
-- projected totals must fail rather than silently erase them.
CREATE TABLE public.metric_rollup_work_day (
    rollup_work_day_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    work_id uuid NOT NULL,
    publication_id uuid,
    platform_id uuid NOT NULL,
    measure_id uuid NOT NULL,
    day date NOT NULL,
    country_code character(2),
    institution_id uuid,
    value bigint NOT NULL,
    watermark bigint NOT NULL,
    CONSTRAINT metric_rollup_work_day_pkey PRIMARY KEY (rollup_work_day_id),
    -- Shape only, exactly as the canonical record enforces it: two uppercase
    -- ASCII letters when supplied, NULL when the dimension is absent.
    CONSTRAINT metric_rollup_work_day_country_code_check
        CHECK (country_code ~ '^[A-Z]{2}$'),
    CONSTRAINT metric_rollup_work_day_watermark_check CHECK (watermark > 0),
    CONSTRAINT metric_rollup_work_day_identity_key
        UNIQUE NULLS NOT DISTINCT
        (work_id, publication_id, platform_id, measure_id, day, country_code,
         institution_id),
    CONSTRAINT metric_rollup_work_day_work_id_fkey FOREIGN KEY (work_id)
        REFERENCES public.work(work_id),
    CONSTRAINT metric_rollup_work_day_publication_id_fkey FOREIGN KEY (publication_id)
        REFERENCES public.publication(publication_id),
    CONSTRAINT metric_rollup_work_day_platform_id_fkey FOREIGN KEY (platform_id)
        REFERENCES public.metric_platform(platform_id),
    CONSTRAINT metric_rollup_work_day_measure_id_fkey FOREIGN KEY (measure_id)
        REFERENCES public.metric_measure(measure_id),
    CONSTRAINT metric_rollup_work_day_institution_id_fkey FOREIGN KEY (institution_id)
        REFERENCES public.institution(institution_id)
);

-- The two minimum query indexes fixed by Amendment 2 section 6. Any further
-- index requires query-plan evidence from the MET-WP4-02 read contract and an
-- explicit later amendment.
CREATE INDEX metric_rollup_work_day_work_id_day_idx
    ON public.metric_rollup_work_day (work_id, day);

CREATE INDEX metric_rollup_work_day_measure_id_platform_id_day_idx
    ON public.metric_rollup_work_day (measure_id, platform_id, day);
