-- MET-WP4-03A-OPS-03: the run table of the resumable monthly rebuild and the
-- paged verification (issue #1012).
--
-- Authority: the approved OPS-03 specification (R9 with erratum R9.1) on
-- #1012, section 5. Additive. Creates exactly one table,
-- `metric_rollup_month_rebuild_run`, and one index on it,
-- `metric_rollup_month_rebuild_run_one_open_idx`. Nothing else is created or
-- altered: no column, index, trigger, function or enum on any existing table,
-- and no seed row.
--
-- What a run is: the durable cursor and counters of one caller-driven
-- traversal of the six derived rollup families (four monthly, two yearly) in
-- units `(work_id, year_start)`. A `REBUILD` run goes `BUILDING`, then
-- `VERIFYING`, then a terminal status; a `VERIFY` run is `VERIFYING` only.
-- Every transition is made by an explicit request of the protected
-- `METRICS_INGEST_SERVICE` caller, one bounded step or page per request,
-- each in its own short transaction. Thoth runs nothing on its own: there is
-- no worker, queue, lease, scheduler or timer, and this table is a
-- maintenance cursor, not a job or an event. At most one run is open, which
-- the partial unique index below enforces.
--
-- Canonical boundary (reviewed): the table holds run evidence only. It
-- references no row (the cursor is a position in unit order, not a reference,
-- so it carries no foreign key), and nothing reads it except the run
-- operations themselves. Canonical Metrics authority, the work-day frontier
-- and the derived projections are untouched by this migration.
--
-- Closed vocabularies are `CHECK`s, not PostgreSQL enums, as elsewhere in
-- Metrics. The checks below make every row shape the state machine can
-- produce representable and every other shape impossible: a terminal run has
-- no cursor (except `ABANDONED`, which keeps the position where it stopped)
-- and has at least one verify page with both verify frontiers set, the
-- frontiers never decrease from the run's start through its build steps to
-- its verify pages, `RECONCILED` means every page was exact at one frontier,
-- `VERIFIED` means every page was exact over a frontier that moved, and
-- `MISMATCHED` means some page found a difference.
--
-- Empty at migration (reviewed): no run exists until a caller starts one.
-- Merging and deploying start nothing.
--
-- Locking (reviewed): `CREATE TABLE` and `CREATE UNIQUE INDEX` on a new,
-- empty table. No lock is taken on any existing table.

CREATE TABLE public.metric_rollup_month_rebuild_run (
    run_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    mode text NOT NULL,
    status text NOT NULL,
    step_count bigint DEFAULT 0 NOT NULL,
    cursor_work_id uuid,
    cursor_year_start date,
    -- The authenticated service principal that started the run. Stored for
    -- the operator's audit, never returned by any operation.
    started_by text NOT NULL,
    started_at timestamptz NOT NULL,
    updated_at timestamptz NOT NULL,
    build_completed_at timestamptz,
    completed_at timestamptz,
    start_applied_through_sequence bigint NOT NULL,
    build_steps bigint DEFAULT 0 NOT NULL,
    build_replaced_steps bigint DEFAULT 0 NOT NULL,
    build_units bigint DEFAULT 0 NOT NULL,
    build_month_keys bigint DEFAULT 0 NOT NULL,
    build_work_day_rows bigint DEFAULT 0 NOT NULL,
    build_found_missing_rows bigint DEFAULT 0 NOT NULL,
    build_found_extra_rows bigint DEFAULT 0 NOT NULL,
    build_found_mismatched_rows bigint DEFAULT 0 NOT NULL,
    build_first_frontier bigint,
    build_last_frontier bigint,
    verify_pages bigint DEFAULT 0 NOT NULL,
    verify_units bigint DEFAULT 0 NOT NULL,
    verify_month_keys bigint DEFAULT 0 NOT NULL,
    verify_work_day_rows bigint DEFAULT 0 NOT NULL,
    verify_first_frontier bigint,
    verify_last_frontier bigint,
    verify_total_expected_rows bigint DEFAULT 0 NOT NULL,
    verify_total_actual_rows bigint DEFAULT 0 NOT NULL,
    verify_total_missing_rows bigint DEFAULT 0 NOT NULL,
    verify_total_extra_rows bigint DEFAULT 0 NOT NULL,
    verify_total_mismatched_rows bigint DEFAULT 0 NOT NULL,
    verify_country_expected_rows bigint DEFAULT 0 NOT NULL,
    verify_country_actual_rows bigint DEFAULT 0 NOT NULL,
    verify_country_missing_rows bigint DEFAULT 0 NOT NULL,
    verify_country_extra_rows bigint DEFAULT 0 NOT NULL,
    verify_country_mismatched_rows bigint DEFAULT 0 NOT NULL,
    verify_institution_expected_rows bigint DEFAULT 0 NOT NULL,
    verify_institution_actual_rows bigint DEFAULT 0 NOT NULL,
    verify_institution_missing_rows bigint DEFAULT 0 NOT NULL,
    verify_institution_extra_rows bigint DEFAULT 0 NOT NULL,
    verify_institution_mismatched_rows bigint DEFAULT 0 NOT NULL,
    verify_ambiguity_expected_rows bigint DEFAULT 0 NOT NULL,
    verify_ambiguity_actual_rows bigint DEFAULT 0 NOT NULL,
    verify_ambiguity_missing_rows bigint DEFAULT 0 NOT NULL,
    verify_ambiguity_extra_rows bigint DEFAULT 0 NOT NULL,
    verify_ambiguity_mismatched_rows bigint DEFAULT 0 NOT NULL,
    verify_country_year_expected_rows bigint DEFAULT 0 NOT NULL,
    verify_country_year_actual_rows bigint DEFAULT 0 NOT NULL,
    verify_country_year_missing_rows bigint DEFAULT 0 NOT NULL,
    verify_country_year_extra_rows bigint DEFAULT 0 NOT NULL,
    verify_country_year_mismatched_rows bigint DEFAULT 0 NOT NULL,
    verify_institution_year_expected_rows bigint DEFAULT 0 NOT NULL,
    verify_institution_year_actual_rows bigint DEFAULT 0 NOT NULL,
    verify_institution_year_missing_rows bigint DEFAULT 0 NOT NULL,
    verify_institution_year_extra_rows bigint DEFAULT 0 NOT NULL,
    verify_institution_year_mismatched_rows bigint DEFAULT 0 NOT NULL,
    CONSTRAINT metric_rollup_month_rebuild_run_pkey PRIMARY KEY (run_id),

    -- The two closed vocabularies.
    CONSTRAINT metric_rollup_month_rebuild_run_mode_check
        CHECK (mode IN ('REBUILD', 'VERIFY')),
    CONSTRAINT metric_rollup_month_rebuild_run_status_check
        CHECK (status IN ('BUILDING', 'VERIFYING', 'RECONCILED', 'VERIFIED', 'MISMATCHED',
                          'ABANDONED')),

    -- The cursor is a unit: both halves or neither, on a 1 January.
    CONSTRAINT metric_rollup_month_rebuild_run_cursor_check
        CHECK ((cursor_work_id IS NULL) = (cursor_year_start IS NULL)
               AND (cursor_year_start IS NULL
                    OR (EXTRACT(MONTH FROM cursor_year_start) = 1
                        AND EXTRACT(DAY FROM cursor_year_start) = 1))),

    -- A finished traversal has no cursor. `ABANDONED` keeps the position
    -- where the run stopped.
    CONSTRAINT metric_rollup_month_rebuild_run_terminal_cursor_check
        CHECK (status NOT IN ('RECONCILED', 'VERIFIED', 'MISMATCHED')
               OR cursor_work_id IS NULL),

    -- A `REBUILD` run abandoned before its build finished never verified
    -- anything. (A `VERIFY` run abandoned after a page keeps its counters.)
    CONSTRAINT metric_rollup_month_rebuild_run_abandoned_build_check
        CHECK (NOT (status = 'ABANDONED' AND mode = 'REBUILD'
                    AND build_completed_at IS NULL)
               OR (verify_pages = 0 AND verify_units = 0 AND verify_month_keys = 0
                   AND verify_work_day_rows = 0
                   AND verify_total_expected_rows = 0 AND verify_total_actual_rows = 0
                   AND verify_total_missing_rows = 0 AND verify_total_extra_rows = 0
                   AND verify_total_mismatched_rows = 0
                   AND verify_country_expected_rows = 0 AND verify_country_actual_rows = 0
                   AND verify_country_missing_rows = 0 AND verify_country_extra_rows = 0
                   AND verify_country_mismatched_rows = 0
                   AND verify_institution_expected_rows = 0
                   AND verify_institution_actual_rows = 0
                   AND verify_institution_missing_rows = 0
                   AND verify_institution_extra_rows = 0
                   AND verify_institution_mismatched_rows = 0
                   AND verify_ambiguity_expected_rows = 0 AND verify_ambiguity_actual_rows = 0
                   AND verify_ambiguity_missing_rows = 0 AND verify_ambiguity_extra_rows = 0
                   AND verify_ambiguity_mismatched_rows = 0
                   AND verify_country_year_expected_rows = 0
                   AND verify_country_year_actual_rows = 0
                   AND verify_country_year_missing_rows = 0
                   AND verify_country_year_extra_rows = 0
                   AND verify_country_year_mismatched_rows = 0
                   AND verify_institution_year_expected_rows = 0
                   AND verify_institution_year_actual_rows = 0
                   AND verify_institution_year_missing_rows = 0
                   AND verify_institution_year_extra_rows = 0
                   AND verify_institution_year_mismatched_rows = 0
                   AND verify_first_frontier IS NULL AND verify_last_frontier IS NULL)),

    CONSTRAINT metric_rollup_month_rebuild_run_started_by_check
        CHECK (btrim(started_by) <> ''),

    -- Open exactly while the run is open.
    CONSTRAINT metric_rollup_month_rebuild_run_completed_at_check
        CHECK ((completed_at IS NULL) = (status IN ('BUILDING', 'VERIFYING'))),

    -- The build phase belongs to `REBUILD` runs only and precedes every page.
    CONSTRAINT metric_rollup_month_rebuild_run_building_check
        CHECK (status <> 'BUILDING'
               OR (mode = 'REBUILD' AND build_completed_at IS NULL
                   AND verify_pages = 0 AND verify_units = 0 AND verify_month_keys = 0
                   AND verify_work_day_rows = 0
                   AND verify_total_expected_rows = 0 AND verify_total_actual_rows = 0
                   AND verify_total_missing_rows = 0 AND verify_total_extra_rows = 0
                   AND verify_total_mismatched_rows = 0
                   AND verify_country_expected_rows = 0 AND verify_country_actual_rows = 0
                   AND verify_country_missing_rows = 0 AND verify_country_extra_rows = 0
                   AND verify_country_mismatched_rows = 0
                   AND verify_institution_expected_rows = 0
                   AND verify_institution_actual_rows = 0
                   AND verify_institution_missing_rows = 0
                   AND verify_institution_extra_rows = 0
                   AND verify_institution_mismatched_rows = 0
                   AND verify_ambiguity_expected_rows = 0 AND verify_ambiguity_actual_rows = 0
                   AND verify_ambiguity_missing_rows = 0 AND verify_ambiguity_extra_rows = 0
                   AND verify_ambiguity_mismatched_rows = 0
                   AND verify_country_year_expected_rows = 0
                   AND verify_country_year_actual_rows = 0
                   AND verify_country_year_missing_rows = 0
                   AND verify_country_year_extra_rows = 0
                   AND verify_country_year_mismatched_rows = 0
                   AND verify_institution_year_expected_rows = 0
                   AND verify_institution_year_actual_rows = 0
                   AND verify_institution_year_missing_rows = 0
                   AND verify_institution_year_extra_rows = 0
                   AND verify_institution_year_mismatched_rows = 0
                   AND verify_first_frontier IS NULL AND verify_last_frontier IS NULL)),

    -- A `REBUILD` run verifies only after its build phase reached the end.
    CONSTRAINT metric_rollup_month_rebuild_run_rebuild_built_check
        CHECK (NOT (mode = 'REBUILD'
                    AND status IN ('VERIFYING', 'RECONCILED', 'VERIFIED', 'MISMATCHED'))
               OR build_completed_at IS NOT NULL),

    -- A `VERIFY` run never builds.
    CONSTRAINT metric_rollup_month_rebuild_run_verify_mode_check
        CHECK (mode <> 'VERIFY'
               OR (build_completed_at IS NULL
                   AND build_steps = 0 AND build_replaced_steps = 0 AND build_units = 0
                   AND build_month_keys = 0 AND build_work_day_rows = 0
                   AND build_found_missing_rows = 0 AND build_found_extra_rows = 0
                   AND build_found_mismatched_rows = 0
                   AND build_first_frontier IS NULL AND build_last_frontier IS NULL)),

    -- Each frontier pair is both set or both unset.
    CONSTRAINT metric_rollup_month_rebuild_run_frontier_pairs_check
        CHECK ((build_first_frontier IS NULL) = (build_last_frontier IS NULL)
               AND (verify_first_frontier IS NULL) = (verify_last_frontier IS NULL)),

    -- The frontiers never decrease: start <= build first <= build last <=
    -- verify first <= verify last, with an unset build pair bridged by
    -- start <= verify first. Each comparison is written to hold only when
    -- both of its sides are set, so the chain does not depend on how a
    -- `CHECK` treats `NULL`.
    CONSTRAINT metric_rollup_month_rebuild_run_frontier_order_check
        CHECK ((build_first_frontier IS NULL
                OR start_applied_through_sequence <= build_first_frontier)
               AND (build_first_frontier IS NULL OR build_last_frontier IS NULL
                    OR build_first_frontier <= build_last_frontier)
               AND (build_last_frontier IS NULL OR verify_first_frontier IS NULL
                    OR build_last_frontier <= verify_first_frontier)
               AND (verify_first_frontier IS NULL OR verify_last_frontier IS NULL
                    OR verify_first_frontier <= verify_last_frontier)
               AND (build_first_frontier IS NOT NULL OR verify_first_frontier IS NULL
                    OR start_applied_through_sequence <= verify_first_frontier)),

    -- A finished traversal verified at least one page, at recorded
    -- frontiers. Written explicitly because a `CHECK` passes on `NULL`.
    CONSTRAINT metric_rollup_month_rebuild_run_terminal_verify_check
        CHECK (status NOT IN ('RECONCILED', 'VERIFIED', 'MISMATCHED')
               OR (verify_pages >= 1
                   AND verify_first_frontier IS NOT NULL
                   AND verify_last_frontier IS NOT NULL)),

    -- `RECONCILED`: every page exact at one frontier.
    CONSTRAINT metric_rollup_month_rebuild_run_reconciled_check
        CHECK (status <> 'RECONCILED'
               OR (verify_first_frontier = verify_last_frontier
                   AND verify_total_missing_rows = 0 AND verify_total_extra_rows = 0
                   AND verify_total_mismatched_rows = 0
                   AND verify_country_missing_rows = 0 AND verify_country_extra_rows = 0
                   AND verify_country_mismatched_rows = 0
                   AND verify_institution_missing_rows = 0
                   AND verify_institution_extra_rows = 0
                   AND verify_institution_mismatched_rows = 0
                   AND verify_ambiguity_missing_rows = 0 AND verify_ambiguity_extra_rows = 0
                   AND verify_ambiguity_mismatched_rows = 0
                   AND verify_country_year_missing_rows = 0
                   AND verify_country_year_extra_rows = 0
                   AND verify_country_year_mismatched_rows = 0
                   AND verify_institution_year_missing_rows = 0
                   AND verify_institution_year_extra_rows = 0
                   AND verify_institution_year_mismatched_rows = 0)),

    -- `VERIFIED`: every page exact, over a frontier that moved.
    CONSTRAINT metric_rollup_month_rebuild_run_verified_check
        CHECK (status <> 'VERIFIED'
               OR (verify_first_frontier < verify_last_frontier
                   AND verify_total_missing_rows = 0 AND verify_total_extra_rows = 0
                   AND verify_total_mismatched_rows = 0
                   AND verify_country_missing_rows = 0 AND verify_country_extra_rows = 0
                   AND verify_country_mismatched_rows = 0
                   AND verify_institution_missing_rows = 0
                   AND verify_institution_extra_rows = 0
                   AND verify_institution_mismatched_rows = 0
                   AND verify_ambiguity_missing_rows = 0 AND verify_ambiguity_extra_rows = 0
                   AND verify_ambiguity_mismatched_rows = 0
                   AND verify_country_year_missing_rows = 0
                   AND verify_country_year_extra_rows = 0
                   AND verify_country_year_mismatched_rows = 0
                   AND verify_institution_year_missing_rows = 0
                   AND verify_institution_year_extra_rows = 0
                   AND verify_institution_year_mismatched_rows = 0)),

    -- `MISMATCHED`: some page found a missing, extra or mismatched row.
    CONSTRAINT metric_rollup_month_rebuild_run_mismatched_check
        CHECK (status <> 'MISMATCHED'
               OR verify_total_missing_rows <> 0 OR verify_total_extra_rows <> 0
               OR verify_total_mismatched_rows <> 0
               OR verify_country_missing_rows <> 0 OR verify_country_extra_rows <> 0
               OR verify_country_mismatched_rows <> 0
               OR verify_institution_missing_rows <> 0
               OR verify_institution_extra_rows <> 0
               OR verify_institution_mismatched_rows <> 0
               OR verify_ambiguity_missing_rows <> 0 OR verify_ambiguity_extra_rows <> 0
               OR verify_ambiguity_mismatched_rows <> 0
               OR verify_country_year_missing_rows <> 0
               OR verify_country_year_extra_rows <> 0
               OR verify_country_year_mismatched_rows <> 0
               OR verify_institution_year_missing_rows <> 0
               OR verify_institution_year_extra_rows <> 0
               OR verify_institution_year_mismatched_rows <> 0),

    -- A finished build ran at least one step, at recorded frontiers.
    CONSTRAINT metric_rollup_month_rebuild_run_build_completed_check
        CHECK (build_completed_at IS NULL
               OR (build_steps >= 1
                   AND build_first_frontier IS NOT NULL
                   AND build_last_frontier IS NOT NULL)),

    -- Every counter is non-negative, and the step count is the steps plus
    -- the pages.
    CONSTRAINT metric_rollup_month_rebuild_run_counters_check
        CHECK (step_count >= 0
               AND build_steps >= 0 AND build_replaced_steps >= 0 AND build_units >= 0
               AND build_month_keys >= 0 AND build_work_day_rows >= 0
               AND build_found_missing_rows >= 0 AND build_found_extra_rows >= 0
               AND build_found_mismatched_rows >= 0
               AND verify_pages >= 0 AND verify_units >= 0 AND verify_month_keys >= 0
               AND verify_work_day_rows >= 0
               AND verify_total_expected_rows >= 0 AND verify_total_actual_rows >= 0
               AND verify_total_missing_rows >= 0 AND verify_total_extra_rows >= 0
               AND verify_total_mismatched_rows >= 0
               AND verify_country_expected_rows >= 0 AND verify_country_actual_rows >= 0
               AND verify_country_missing_rows >= 0 AND verify_country_extra_rows >= 0
               AND verify_country_mismatched_rows >= 0
               AND verify_institution_expected_rows >= 0
               AND verify_institution_actual_rows >= 0
               AND verify_institution_missing_rows >= 0
               AND verify_institution_extra_rows >= 0
               AND verify_institution_mismatched_rows >= 0
               AND verify_ambiguity_expected_rows >= 0 AND verify_ambiguity_actual_rows >= 0
               AND verify_ambiguity_missing_rows >= 0 AND verify_ambiguity_extra_rows >= 0
               AND verify_ambiguity_mismatched_rows >= 0
               AND verify_country_year_expected_rows >= 0
               AND verify_country_year_actual_rows >= 0
               AND verify_country_year_missing_rows >= 0
               AND verify_country_year_extra_rows >= 0
               AND verify_country_year_mismatched_rows >= 0
               AND verify_institution_year_expected_rows >= 0
               AND verify_institution_year_actual_rows >= 0
               AND verify_institution_year_missing_rows >= 0
               AND verify_institution_year_extra_rows >= 0
               AND verify_institution_year_mismatched_rows >= 0),
    CONSTRAINT metric_rollup_month_rebuild_run_step_count_check
        CHECK (step_count = build_steps + verify_pages)
);

-- At most one open run. A second `BUILDING` or `VERIFYING` row violates
-- this index, which is how a concurrent first arrival of two starts yields
-- exactly one run.
CREATE UNIQUE INDEX metric_rollup_month_rebuild_run_one_open_idx
    ON public.metric_rollup_month_rebuild_run ((true))
    WHERE status IN ('BUILDING', 'VERIFYING');
