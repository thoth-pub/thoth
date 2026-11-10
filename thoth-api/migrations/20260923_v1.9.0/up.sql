-- MET-WP4-03A: derived monthly dashboard serving projections (issue #940).
--
-- Additive and initially inactive. Creates exactly the four derived,
-- rebuildable monthly serving tables the approved dashboard architecture
-- (MET-WP4-03 / #937 through Amendment 2, #940 through Amendment 1) proved
-- necessary: three resolved monthly value projections and one sparse
-- ambiguity-state table. Nothing else is created or altered: no canonical
-- table, no work-day projection column, no trigger, no enum, no secondary
-- index and no seed row.
--
-- Canonical boundary (reviewed): canonical Metrics authority remains
-- `metric_record`, `metric_record_revision`, the durable rollup deltas and
-- the MET-WP4-01 work-day frontier. Every row created in these four tables is
-- derived exclusively from `metric_rollup_work_day` by the completion
-- transaction that maintains them, and a full rebuild reads only that
-- work-day projection. No canonical row is read, modified or repaired here.
--
-- Empty at migration (reviewed, #940 Amendment 1 section 3): this migration
-- populates nothing. It is safe because MET-WP4-03A adds no reader of these
-- tables. No consumer may serve from them until a separately authorized
-- historical rebuild from `metric_rollup_work_day` has populated them and
-- been reconciled at a recorded `metric_rollup_work_day_state`
-- frontier. That activation gate is operational and outside this migration.
--
-- Semantics fixed by the approved specification and implemented by the
-- completion transaction, recorded here so the schema reads correctly:
--
--   * every daily base cell `(work_id, platform_id, measure_id, day)` of the
--     work-day projection is resolved FIRST — the #910 Amendment 6 total rule
--     for totals, and the unique-least target-dimension rule for country and
--     institution — and only the resolved daily contributions are summed
--     into a month. Raw day rows are never compacted into a month and then
--     re-resolved;
--   * `publication_id` is retained only when the selected daily
--     representation carries it; otherwise it is NULL. Within one month a
--     NULL row and publication-specific rows from different resolved days are
--     additive contributions, never re-ranked against each other;
--   * `requires_country_coverage` / `requires_institution_coverage` are the
--     #910 range-wide dependency booleans, OR-ed across the resolved daily
--     contributions grouped into that monthly row;
--   * `watermark` on a value row is the greatest `metric_rollup_work_day`
--     watermark over exactly the current resolved daily source rows that
--     contribute to that row; on an ambiguity row it is the greatest
--     watermark over the union of daily source rows whose current represented
--     state establishes any currently-true flag. It is local derived-state
--     evidence and never a serving boundary: the only safe global serving
--     frontier remains `metric_rollup_work_day_state.applied_through_sequence`;
--   * ambiguity is sparse: a `(work_id, platform_id, measure_id, month_start)`
--     row exists only while at least one of its three flags is true, and it
--     may exist for a month that has no value row in the ambiguous section.
--
-- Uniqueness decision (reviewed): the value projections keep the optional
-- `publication_id` dimension, so their logical identities use
-- `UNIQUE NULLS NOT DISTINCT` exactly as `metric_rollup_work_day` does;
-- ordinary SQL uniqueness would let unlimited duplicate "no publication" rows
-- silently split a monthly total. The ambiguity identity has no nullable
-- column and uses plain `UNIQUE`.
--
-- Index decision (reviewed, BENCH-01): the complete intended index set per
-- table is exactly its primary-key index and the index PostgreSQL creates to
-- enforce its logical identity. No secondary performance index is created;
-- the accepted benchmark rejected every candidate, and any later index needs
-- exact-head query-plan evidence and an explicit amendment.
--
-- Foreign keys are non-cascading, matching every other Metrics key: deleting
-- a work, publication, platform, measure or institution that still has a
-- projected monthly total must fail rather than silently erase it. Adding a
-- table with a foreign key takes SHARE ROW EXCLUSIVE on each referenced
-- parent for the duration of this migration transaction, which blocks
-- concurrent writes (not reads) to those parents while the four empty tables
-- are created; the uncontended duration is milliseconds.

-- ---------------------------------------------------------------------------
-- 1. Resolved monthly totals
-- ---------------------------------------------------------------------------
CREATE TABLE public.metric_rollup_work_month (
    rollup_work_month_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    work_id uuid NOT NULL,
    publication_id uuid,
    platform_id uuid NOT NULL,
    measure_id uuid NOT NULL,
    month_start date NOT NULL,
    value bigint NOT NULL,
    requires_country_coverage boolean NOT NULL,
    requires_institution_coverage boolean NOT NULL,
    watermark bigint NOT NULL,
    CONSTRAINT metric_rollup_work_month_pkey PRIMARY KEY (rollup_work_month_id),
    CONSTRAINT metric_rollup_work_month_month_start_check
        CHECK (EXTRACT(DAY FROM month_start) = 1),
    CONSTRAINT metric_rollup_work_month_watermark_check CHECK (watermark > 0),
    CONSTRAINT metric_rollup_work_month_identity_key
        UNIQUE NULLS NOT DISTINCT
        (work_id, publication_id, platform_id, measure_id, month_start),
    CONSTRAINT metric_rollup_work_month_work_id_fkey FOREIGN KEY (work_id)
        REFERENCES public.work(work_id),
    CONSTRAINT metric_rollup_work_month_publication_id_fkey FOREIGN KEY (publication_id)
        REFERENCES public.publication(publication_id),
    CONSTRAINT metric_rollup_work_month_platform_id_fkey FOREIGN KEY (platform_id)
        REFERENCES public.metric_platform(platform_id),
    CONSTRAINT metric_rollup_work_month_measure_id_fkey FOREIGN KEY (measure_id)
        REFERENCES public.metric_measure(measure_id)
);

-- ---------------------------------------------------------------------------
-- 2. Resolved monthly totals per country
-- ---------------------------------------------------------------------------
--
-- `country_code` is required and shaped exactly as the canonical record
-- enforces it: two uppercase ASCII letters. A country row's only dependency
-- flag is institution coverage, because country coverage is what the row
-- itself represents.
CREATE TABLE public.metric_rollup_work_country_month (
    rollup_work_country_month_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    work_id uuid NOT NULL,
    publication_id uuid,
    platform_id uuid NOT NULL,
    measure_id uuid NOT NULL,
    month_start date NOT NULL,
    country_code character(2) NOT NULL,
    value bigint NOT NULL,
    requires_institution_coverage boolean NOT NULL,
    watermark bigint NOT NULL,
    CONSTRAINT metric_rollup_work_country_month_pkey
        PRIMARY KEY (rollup_work_country_month_id),
    CONSTRAINT metric_rollup_work_country_month_month_start_check
        CHECK (EXTRACT(DAY FROM month_start) = 1),
    CONSTRAINT metric_rollup_work_country_month_country_code_check
        CHECK (country_code ~ '^[A-Z]{2}$'),
    CONSTRAINT metric_rollup_work_country_month_watermark_check CHECK (watermark > 0),
    CONSTRAINT metric_rollup_work_country_month_identity_key
        UNIQUE NULLS NOT DISTINCT
        (work_id, publication_id, platform_id, measure_id, month_start, country_code),
    CONSTRAINT metric_rollup_work_country_month_work_id_fkey FOREIGN KEY (work_id)
        REFERENCES public.work(work_id),
    CONSTRAINT metric_rollup_work_country_month_publication_id_fkey
        FOREIGN KEY (publication_id) REFERENCES public.publication(publication_id),
    CONSTRAINT metric_rollup_work_country_month_platform_id_fkey FOREIGN KEY (platform_id)
        REFERENCES public.metric_platform(platform_id),
    CONSTRAINT metric_rollup_work_country_month_measure_id_fkey FOREIGN KEY (measure_id)
        REFERENCES public.metric_measure(measure_id)
);

-- ---------------------------------------------------------------------------
-- 3. Resolved monthly totals per institution
-- ---------------------------------------------------------------------------
--
-- The mirror of the country projection with institution as the target
-- dimension, so its only dependency flag is country coverage.
CREATE TABLE public.metric_rollup_work_institution_month (
    rollup_work_institution_month_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    work_id uuid NOT NULL,
    publication_id uuid,
    platform_id uuid NOT NULL,
    measure_id uuid NOT NULL,
    month_start date NOT NULL,
    institution_id uuid NOT NULL,
    value bigint NOT NULL,
    requires_country_coverage boolean NOT NULL,
    watermark bigint NOT NULL,
    CONSTRAINT metric_rollup_work_institution_month_pkey
        PRIMARY KEY (rollup_work_institution_month_id),
    CONSTRAINT metric_rollup_work_institution_month_month_start_check
        CHECK (EXTRACT(DAY FROM month_start) = 1),
    CONSTRAINT metric_rollup_work_institution_month_watermark_check
        CHECK (watermark > 0),
    CONSTRAINT metric_rollup_work_institution_month_identity_key
        UNIQUE NULLS NOT DISTINCT
        (work_id, publication_id, platform_id, measure_id, month_start, institution_id),
    CONSTRAINT metric_rollup_work_institution_month_work_id_fkey FOREIGN KEY (work_id)
        REFERENCES public.work(work_id),
    CONSTRAINT metric_rollup_work_institution_month_publication_id_fkey
        FOREIGN KEY (publication_id) REFERENCES public.publication(publication_id),
    CONSTRAINT metric_rollup_work_institution_month_platform_id_fkey
        FOREIGN KEY (platform_id) REFERENCES public.metric_platform(platform_id),
    CONSTRAINT metric_rollup_work_institution_month_measure_id_fkey
        FOREIGN KEY (measure_id) REFERENCES public.metric_measure(measure_id),
    CONSTRAINT metric_rollup_work_institution_month_institution_id_fkey
        FOREIGN KEY (institution_id) REFERENCES public.institution(institution_id)
);

-- ---------------------------------------------------------------------------
-- 4. Sparse monthly ambiguity state
-- ---------------------------------------------------------------------------
--
-- One row per `(work_id, platform_id, measure_id, month_start)` while at
-- least one section of that month is ambiguous. The three flags are
-- independent; a row with all three false is not representable, so a month
-- whose ambiguity clears is removed rather than kept for its old watermark.
-- Ambiguity never blocks rollup completion or frontier progress: it is
-- fail-closed serving evidence for the later MET-WP4-03B reader.
CREATE TABLE public.metric_rollup_work_month_ambiguity (
    rollup_work_month_ambiguity_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    work_id uuid NOT NULL,
    platform_id uuid NOT NULL,
    measure_id uuid NOT NULL,
    month_start date NOT NULL,
    total_ambiguous boolean NOT NULL,
    country_ambiguous boolean NOT NULL,
    institution_ambiguous boolean NOT NULL,
    watermark bigint NOT NULL,
    CONSTRAINT metric_rollup_work_month_ambiguity_pkey
        PRIMARY KEY (rollup_work_month_ambiguity_id),
    CONSTRAINT metric_rollup_work_month_ambiguity_month_start_check
        CHECK (EXTRACT(DAY FROM month_start) = 1),
    CONSTRAINT metric_rollup_work_month_ambiguity_flags_check
        CHECK (total_ambiguous OR country_ambiguous OR institution_ambiguous),
    CONSTRAINT metric_rollup_work_month_ambiguity_watermark_check CHECK (watermark > 0),
    CONSTRAINT metric_rollup_work_month_ambiguity_identity_key
        UNIQUE (work_id, platform_id, measure_id, month_start),
    CONSTRAINT metric_rollup_work_month_ambiguity_work_id_fkey FOREIGN KEY (work_id)
        REFERENCES public.work(work_id),
    CONSTRAINT metric_rollup_work_month_ambiguity_platform_id_fkey FOREIGN KEY (platform_id)
        REFERENCES public.metric_platform(platform_id),
    CONSTRAINT metric_rollup_work_month_ambiguity_measure_id_fkey FOREIGN KEY (measure_id)
        REFERENCES public.metric_measure(measure_id)
);
