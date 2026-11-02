-- MET-WP4-03C-B1: H1 coverage-run foundation and H2 native-grain access
-- path (issue #952).
--
-- Authority: Specification Amendments 5, 5A and 5B of #952 as approved by
-- the independent CRITICAL review, and the frozen B1 implementation handoff
-- with its two corrections. Additive. Creates exactly:
--
--   1. the derived, rebuildable coverage-run table `metric_coverage_run`
--      with its approved primary key, four foreign keys and interval check;
--   2. the four permanent secondary indexes of the B1 contract: two raw
--      coverage access indexes, the terminal-without-completion partial
--      index on `metric_import`, and the H2 native-grain partial index on
--      `metric_record`;
--   3. one trigger function, `maintain_metric_coverage_runs_from_import`,
--      and one row trigger on `metric_import`,
--      `metric_import_maintain_coverage_runs`, which keep the coverage runs
--      of one source account exact inside the same transaction that
--      terminalizes (or later changes the terminal identity of) an import.
--
-- Nothing else is created or altered: no canonical table, no enum, no
-- helper function, no fifth index and no seed row.
--
-- Canonical boundary (reviewed): raw Metrics coverage authority remains
-- `metric_coverage` + `metric_import`, and `metric_source_account` remains
-- the authority for the existence of an account, for the serialization of
-- its maintenance and for the complete verification domain. Every row of
-- `metric_coverage_run` is derived exclusively from `metric_coverage` joined
-- to its terminal `metric_import`; a rebuild reads nothing else. No canonical
-- row is read for any other purpose, modified or repaired here.
--
-- Historical stream provenance (Amendment 5B): a run's stream identity is
--
--   source_account_id = metric_coverage.source_account_id
--   platform_id       = metric_coverage.platform_id
--   publisher_id      = metric_import.publisher_id
--   measure_id        = metric_coverage.measure_id
--
-- with `metric_import.import_id = metric_coverage.import_id` and
-- `metric_import.source_account_id = metric_coverage.source_account_id`.
-- A coverage assertion participates only while its import is terminal
-- (`COMPLETED` or `COMPLETED_WITH_ERRORS`), has a recorded `completed_at`
-- and names a publisher. The current `metric_source_account.platform_id`
-- and `expected_publisher_id` are request-time serving eligibility (Model
-- A) and are never written into a run: disabling, re-enabling or
-- reconfiguring an account rewrites no historical run, and no trigger is
-- installed on any account, source or coverage writer.
--
-- Run semantics: for each stream and each day, the winning assertion among
-- those covering the day is the first under
--
--   completed_at DESC, import_id DESC, coverage_status DESC,
--   country_coverage ASC, institution_coverage ASC, coverage_id DESC
--
-- and a run `[run_start, run_end)` is a maximal half-open range of
-- consecutive covered days whose externally visible winning tuple
-- (`coverage_status`, `import_status`, `country_coverage`,
-- `institution_coverage`) is equal. Runs in one stream are ordered,
-- non-overlapping and gap-capable, and adjacent runs with an equal visible
-- tuple are always coalesced: a change of winning import or coverage row
-- alone never splits a run. No per-day row is ever stored; the winner is
-- recomputed from raw evidence whenever a run is maintained, so no winner
-- identity, timestamp, generation or watermark column exists here.
--
-- Empty at migration (reviewed): this migration performs NO historical
-- backfill, switches no reader, mutates no account or source configuration
-- and rewrites no canonical coverage or import row. Until a separately
-- authorized rebuild has populated an account's runs, the independent
-- verifier reports its expected runs as missing. The dashboard reader does
-- not consume this table in B1.
--
-- Locking (reviewed): `CREATE TABLE` with foreign keys takes
-- `SHARE ROW EXCLUSIVE` on each referenced parent (`metric_source_account`,
-- `metric_platform`, `publisher`, `metric_measure`) for the migration
-- transaction, blocking concurrent writes (not reads) to those parents.
-- Each ordinary transactional `CREATE INDEX` takes `SHARE` on its table
-- (`metric_coverage`, `metric_import`, `metric_record`), blocking concurrent
-- writes (not reads) to that table for the duration of the build; the
-- build time is proportional to the table size. `CREATE TRIGGER` takes
-- `SHARE ROW EXCLUSIVE` on `metric_import`. `CONCURRENTLY` is deliberately
-- not used: the repository migration runner executes a migration inside one
-- transaction. Production execution remains separately authorized.

-- ---------------------------------------------------------------------------
-- 1. Derived coverage runs
-- ---------------------------------------------------------------------------
--
-- Exactly the ten approved durable columns, all NOT NULL. The primary key is
-- the stream identity plus `run_start`, which is what makes a stream's runs
-- deterministically unique per start day. The four foreign keys use the
-- repository's ordinary plain `REFERENCES` (NO ACTION) convention, so a
-- source account, platform, publisher or measure that still has a derived
-- run cannot be deleted silently. The check enforces the half-open interval;
-- overlap and maximal coalescing are enforced by the writer below and
-- detected by the independent verifier, not by an exclusion constraint.
CREATE TABLE public.metric_coverage_run (
    source_account_id uuid NOT NULL,
    platform_id uuid NOT NULL,
    publisher_id uuid NOT NULL,
    measure_id uuid NOT NULL,
    run_start date NOT NULL,
    run_end date NOT NULL,
    coverage_status public.metric_coverage_status NOT NULL,
    import_status public.metric_import_status NOT NULL,
    country_coverage boolean NOT NULL,
    institution_coverage boolean NOT NULL,
    CONSTRAINT metric_coverage_run_pkey
        PRIMARY KEY (source_account_id, platform_id, publisher_id, measure_id, run_start),
    CONSTRAINT metric_coverage_run_interval_check CHECK (run_end > run_start),
    CONSTRAINT metric_coverage_run_source_account_id_fkey FOREIGN KEY (source_account_id)
        REFERENCES public.metric_source_account(source_account_id),
    CONSTRAINT metric_coverage_run_platform_id_fkey FOREIGN KEY (platform_id)
        REFERENCES public.metric_platform(platform_id),
    CONSTRAINT metric_coverage_run_publisher_id_fkey FOREIGN KEY (publisher_id)
        REFERENCES public.publisher(publisher_id),
    CONSTRAINT metric_coverage_run_measure_id_fkey FOREIGN KEY (measure_id)
        REFERENCES public.metric_measure(measure_id)
);

-- ---------------------------------------------------------------------------
-- 2. The exhaustive B1 permanent index set
-- ---------------------------------------------------------------------------
--
-- Exactly these four. The import-id index serves the affected-hull lookup of
-- one import's coverage rows; the account/period_end index serves the
-- account-scoped raw recomputation of the rows overlapping a hull; the
-- partial terminal-without-completion index serves the narrow raw probe the
-- later reader keeps for terminal imports without a recorded completion
-- (its predicate is the canonical enum predicate, so a probe written as
-- `status::text IN (...)` would not use it); and the H2 partial index
-- serves the existing bounded native-grain access path over `metric_record`
-- without changing its semantics.

CREATE INDEX metric_coverage_import_id_idx
    ON public.metric_coverage (import_id);

CREATE INDEX metric_coverage_source_account_id_period_end_idx
    ON public.metric_coverage (source_account_id, period_end);

CREATE INDEX metric_import_terminal_without_completion_idx
    ON public.metric_import (source_account_id)
    WHERE status IN (
            'COMPLETED'::public.metric_import_status,
            'COMPLETED_WITH_ERRORS'::public.metric_import_status
          )
      AND completed_at IS NULL;

CREATE INDEX metric_record_native_grain_idx
    ON public.metric_record (
        work_id,
        platform_id,
        measure_id,
        period_start
    )
    WHERE reporting_grain <> 'DAY';

-- ---------------------------------------------------------------------------
-- 3. Incremental maintenance at import terminalization
-- ---------------------------------------------------------------------------
--
-- The only H1 trigger function. It runs inside the transaction that updated
-- one `metric_import` row's `status`, `completed_at` or `publisher_id`, and
-- it returns to that transaction, so import terminalization and run
-- maintenance commit or roll back together. In order:
--
--   1. it requires `READ COMMITTED` and fails closed under any other
--      isolation level: after taking the account lock below, every statement
--      must see the state committed by the previous holder of that lock,
--      which only `READ COMMITTED` guarantees;
--   2. it derives the affected import and source account from OLD/NEW and
--      refuses an import whose account changed in the same update, a state
--      no application path produces;
--   3. it locks `metric_source_account(source_account_id) FOR UPDATE`: the
--      single serialization point every terminalization of that account and
--      every rebuild of that account share, taken after the lifecycle's own
--      checkpoint and import locks and before any run is read or replaced;
--   4. it computes the import's affected hull, the union half-open date hull
--      of its own coverage rows; an import without coverage rows changes no
--      run;
--   5. it removes every run of the account that intersects or touches the
--      hull, keeping the parts of those runs that lie outside the hull as
--      fragments;
--   6. it recomputes, from raw coverage and import evidence alone, the
--      winning runs of every stream of the account inside the hull, using
--      the raw participation predicate (so a removed, re-ranked, re-homed
--      or newly terminal import, OLD publisher and NEW publisher alike, is
--      reflected), then coalesces the fragments and the recomputed runs
--      maximally and inserts the result.
--
-- Recomputing every stream of the account over the hull rather than only
-- the NEW publisher's streams is what makes a publisher change, a
-- terminal-to-non-participating change and a `completed_at` re-ranking
-- exact in one pass. The function sets no `lock_timeout`,
-- `statement_timeout` or `work_mem`: it inherits the lifecycle transaction.
CREATE FUNCTION public.maintain_metric_coverage_runs_from_import()
    RETURNS trigger
    LANGUAGE plpgsql AS $$
DECLARE
    isolation text;
    account_id uuid;
    hull_start date;
    hull_end date;
    displaced public.metric_coverage_run[];
BEGIN
    isolation := current_setting('transaction_isolation');
    IF isolation <> 'read committed' THEN
        RAISE EXCEPTION
            'metric_coverage_run maintenance requires READ COMMITTED isolation; the current transaction is %',
            isolation
            USING ERRCODE = 'invalid_transaction_state';
    END IF;

    IF OLD.source_account_id IS DISTINCT FROM NEW.source_account_id THEN
        RAISE EXCEPTION
            'metric_coverage_run maintenance does not support moving metric import % to another source account',
            NEW.import_id
            USING ERRCODE = 'feature_not_supported';
    END IF;
    account_id := NEW.source_account_id;

    PERFORM 1
       FROM public.metric_source_account sa
      WHERE sa.source_account_id = account_id
        FOR UPDATE;
    IF NOT FOUND THEN
        RAISE EXCEPTION
            'metric source account % of metric import % does not exist',
            account_id, NEW.import_id
            USING ERRCODE = 'no_data_found';
    END IF;

    SELECT MIN(c.period_start), MAX(c.period_end)
      INTO hull_start, hull_end
      FROM public.metric_coverage c
     WHERE c.import_id = NEW.import_id
       AND c.source_account_id = account_id;
    IF hull_start IS NULL THEN
        RETURN NULL;
    END IF;

    WITH removed AS (
        DELETE FROM public.metric_coverage_run r
         WHERE r.source_account_id = account_id
           AND r.run_start <= hull_end
           AND r.run_end >= hull_start
        RETURNING r AS run
    )
    SELECT COALESCE(array_agg(removed.run), '{}')
      INTO displaced
      FROM removed;

    INSERT INTO public.metric_coverage_run (
        source_account_id, platform_id, publisher_id, measure_id,
        run_start, run_end,
        coverage_status, import_status, country_coverage, institution_coverage
    )
    WITH fragments AS (
        SELECT d.platform_id, d.publisher_id, d.measure_id,
               d.run_start, LEAST(d.run_end, hull_start) AS run_end,
               d.coverage_status, d.import_status,
               d.country_coverage, d.institution_coverage
          FROM unnest(displaced) AS d
         WHERE d.run_start < hull_start
        UNION ALL
        SELECT d.platform_id, d.publisher_id, d.measure_id,
               GREATEST(d.run_start, hull_end) AS run_start, d.run_end,
               d.coverage_status, d.import_status,
               d.country_coverage, d.institution_coverage
          FROM unnest(displaced) AS d
         WHERE d.run_end > hull_end
    ),
    evidence AS (
        SELECT c.platform_id, mi.publisher_id, c.measure_id,
               GREATEST(c.period_start, hull_start) AS covered_start,
               LEAST(c.period_end, hull_end) AS covered_end,
               c.coverage_status, mi.status AS import_status,
               c.country_coverage, c.institution_coverage,
               mi.completed_at, mi.import_id, c.coverage_id
          FROM public.metric_coverage c
          JOIN public.metric_import mi
            ON mi.import_id = c.import_id
           AND mi.source_account_id = c.source_account_id
         WHERE c.source_account_id = account_id
           AND c.period_start < hull_end
           AND c.period_end > hull_start
           AND mi.status IN (
                   'COMPLETED'::public.metric_import_status,
                   'COMPLETED_WITH_ERRORS'::public.metric_import_status
               )
           AND mi.completed_at IS NOT NULL
           AND mi.publisher_id IS NOT NULL
    ),
    boundaries AS (
        SELECT DISTINCT e.platform_id, e.publisher_id, e.measure_id, b.boundary
          FROM evidence e
         CROSS JOIN LATERAL (VALUES (e.covered_start), (e.covered_end)) AS b(boundary)
    ),
    segments AS (
        SELECT b.platform_id, b.publisher_id, b.measure_id,
               b.boundary AS segment_start,
               LEAD(b.boundary) OVER (
                   PARTITION BY b.platform_id, b.publisher_id, b.measure_id
                   ORDER BY b.boundary
               ) AS segment_end
          FROM boundaries b
    ),
    winners AS (
        SELECT DISTINCT ON (s.platform_id, s.publisher_id, s.measure_id, s.segment_start)
               s.platform_id, s.publisher_id, s.measure_id,
               s.segment_start AS run_start, s.segment_end AS run_end,
               e.coverage_status, e.import_status,
               e.country_coverage, e.institution_coverage
          FROM segments s
          JOIN evidence e
            ON e.platform_id = s.platform_id
           AND e.publisher_id = s.publisher_id
           AND e.measure_id = s.measure_id
           AND e.covered_start <= s.segment_start
           AND e.covered_end > s.segment_start
         WHERE s.segment_end IS NOT NULL
         ORDER BY s.platform_id, s.publisher_id, s.measure_id, s.segment_start,
                  e.completed_at DESC, e.import_id DESC,
                  e.coverage_status DESC, e.country_coverage ASC,
                  e.institution_coverage ASC, e.coverage_id DESC
    ),
    candidates AS (
        SELECT * FROM fragments
        UNION ALL
        SELECT * FROM winners
    ),
    marked AS (
        SELECT c.*,
               CASE
                   WHEN LAG(c.run_end) OVER stream = c.run_start
                    AND LAG(c.coverage_status) OVER stream = c.coverage_status
                    AND LAG(c.import_status) OVER stream = c.import_status
                    AND LAG(c.country_coverage) OVER stream = c.country_coverage
                    AND LAG(c.institution_coverage) OVER stream = c.institution_coverage
                   THEN 0
                   ELSE 1
               END AS starts_run
          FROM candidates c
        WINDOW stream AS (
            PARTITION BY c.platform_id, c.publisher_id, c.measure_id
            ORDER BY c.run_start
        )
    ),
    numbered AS (
        SELECT m.*,
               SUM(m.starts_run) OVER (
                   PARTITION BY m.platform_id, m.publisher_id, m.measure_id
                   ORDER BY m.run_start
                   ROWS UNBOUNDED PRECEDING
               ) AS run_number
          FROM marked m
    )
    SELECT account_id, n.platform_id, n.publisher_id, n.measure_id,
           MIN(n.run_start), MAX(n.run_end),
           n.coverage_status, n.import_status,
           n.country_coverage, n.institution_coverage
      FROM numbered n
     GROUP BY n.platform_id, n.publisher_id, n.measure_id, n.run_number,
              n.coverage_status, n.import_status,
              n.country_coverage, n.institution_coverage;

    RETURN NULL;
END;
$$;

-- Fires only for an update that changed `status`, `completed_at` or
-- `publisher_id` and where OLD or NEW is a participating assertion owner.
-- The ordinary ingestion counter update touches none of the three columns,
-- so it never fires; a committed replay through `completeMetricImport`
-- returns before any UPDATE; a terminalization, a later change of a
-- terminal import's `completed_at`, `status` or `publisher_id`, and a
-- terminal-to-non-participating change all fire. The gate belongs to the
-- trigger, not to application code.
CREATE TRIGGER metric_import_maintain_coverage_runs
    AFTER UPDATE OF status, completed_at, publisher_id
    ON public.metric_import
    FOR EACH ROW
    WHEN (
        (
            OLD.status IS DISTINCT FROM NEW.status
            OR OLD.completed_at IS DISTINCT FROM NEW.completed_at
            OR OLD.publisher_id IS DISTINCT FROM NEW.publisher_id
        )
        AND (
            (
                OLD.status IN (
                    'COMPLETED'::public.metric_import_status,
                    'COMPLETED_WITH_ERRORS'::public.metric_import_status
                )
                AND OLD.completed_at IS NOT NULL
                AND OLD.publisher_id IS NOT NULL
            )
            OR
            (
                NEW.status IN (
                    'COMPLETED'::public.metric_import_status,
                    'COMPLETED_WITH_ERRORS'::public.metric_import_status
                )
                AND NEW.completed_at IS NOT NULL
                AND NEW.publisher_id IS NOT NULL
            )
        )
    )
    EXECUTE FUNCTION public.maintain_metric_coverage_runs_from_import();
