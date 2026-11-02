-- MET-WP4-03C-A: yearly derived-state producer foundation (issue #952).
--
-- Authority: Specification Amendments 4, 4A and 4B of #952, the Unit A
-- producer contract of MET-WP4-03C. Additive. Creates exactly the two
-- derived, rebuildable yearly section tables of that contract: one resolved
-- yearly per-country projection and one resolved yearly per-institution
-- projection. Nothing else is created or altered: no canonical table, no
-- monthly table, no work-day projection column, no trigger, no enum, no
-- secondary index and no seed row. There is no yearly total table (the TOTAL
-- timeline is monthly by definition) and no yearly ambiguity table (the
-- sparse monthly ambiguity summary already covers every month of a year).
--
-- Unit A maintains these tables inside every rollup completion, verifies
-- them through verifyMetricRollupMonths and rebuilds them through
-- rebuildMetricRollupMonths. Nothing reads them for serving: the
-- metricDashboard reader of the yearly layer (Unit B of MET-WP4-03C) is
-- separately gated and on HOLD, and no current dashboard source serves
-- complete calendar years from these tables.
--
-- Canonical boundary (reviewed): canonical Metrics authority remains
-- `metric_record`, `metric_record_revision`, the durable rollup deltas and
-- the MET-WP4-01 work-day frontier. Every row created in these two tables is
-- derived exclusively from the reviewed MET-WP4-03A monthly section
-- projections, which are themselves derived from `metric_rollup_work_day`,
-- by the completion transaction that maintains them; a full rebuild derives
-- them from the freshly rebuilt monthly rows inside the same transaction.
-- No canonical row is read, modified or repaired here.
--
-- Empty at migration (reviewed): this migration populates nothing, so after
-- it and before a separately authorized historical rebuild
-- (`rebuildMetricRollupMonths`) the yearly state is incomplete by
-- construction and verification reports the yearly families as missing.
-- Unit B must remain inactive against the yearly layer until that rebuild
-- has populated these tables, an independent six-family verification is
-- exact at a recorded `metric_rollup_work_day_state` frontier, and the
-- separately authorized Unit B activation gate has been passed. Those gates
-- are operational and outside this migration.
--
-- Semantics fixed by the approved specification and implemented by the
-- completion transaction, recorded here so the schema reads correctly:
--
--   * a yearly row is the regrouping, by calendar year, of exactly the
--     resolved daily contributions its monthly rows already hold: `value` is
--     the sum of the monthly values of the year, the dependency flag is the
--     OR of the monthly flags, and `watermark` is the greatest monthly
--     watermark. Because a monthly row is itself SUM / OR / MAX over the
--     resolved daily contributions of `(work, publication, platform,
--     measure, month, dimension)`, dropping the month key regroups the same
--     contributions and every semantic is preserved by associativity; no
--     daily base cell is ever re-resolved at the year;
--   * `publication_id` is retained exactly as the monthly rows hold it: a
--     NULL row and publication-specific rows of one year are additive
--     contributions, never re-ranked against each other;
--   * `year_start` is the 1 January of the calendar year of the monthly
--     rows' `month_start`;
--   * `watermark` is local derived-state evidence and never a serving
--     boundary: the only safe global serving frontier remains
--     `metric_rollup_work_day_state.applied_through_sequence`;
--   * ambiguity is not aggregated: it stays sparse and monthly.
--
-- Uniqueness decision (reviewed): the logical identities keep the optional
-- `publication_id` dimension, so they use `UNIQUE NULLS NOT DISTINCT`
-- exactly as the monthly section tables do; ordinary SQL uniqueness would
-- let unlimited duplicate "no publication" rows silently split a yearly
-- total.
--
-- Index decision (reviewed): the complete intended index set per table is
-- exactly its primary-key index and the index PostgreSQL creates to enforce
-- its logical identity. No secondary performance index is created. The
-- earlier K-3 yearly-projection spike is historical technical evidence only,
-- not implementation authority; it measured no candidate index that made a
-- clipped-year window interactive. Any later index needs exact-head
-- query-plan evidence and an explicit amendment.
--
-- Foreign keys are non-cascading, matching every other Metrics key: deleting
-- a work, publication, platform, measure or institution that still has a
-- projected yearly total must fail rather than silently erase it. Adding a
-- table with a foreign key takes SHARE ROW EXCLUSIVE on each referenced
-- parent for the duration of this migration transaction, which blocks
-- concurrent writes (not reads) to those parents while the two empty tables
-- are created; the uncontended duration is milliseconds.

-- ---------------------------------------------------------------------------
-- 1. Resolved yearly per-country totals
-- ---------------------------------------------------------------------------
CREATE TABLE public.metric_rollup_work_country_year (
    rollup_work_country_year_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    work_id uuid NOT NULL,
    publication_id uuid,
    platform_id uuid NOT NULL,
    measure_id uuid NOT NULL,
    year_start date NOT NULL,
    country_code character(2) NOT NULL,
    value bigint NOT NULL,
    requires_institution_coverage boolean NOT NULL,
    watermark bigint NOT NULL,
    CONSTRAINT metric_rollup_work_country_year_pkey
        PRIMARY KEY (rollup_work_country_year_id),
    CONSTRAINT metric_rollup_work_country_year_year_start_check
        CHECK (EXTRACT(MONTH FROM year_start) = 1 AND EXTRACT(DAY FROM year_start) = 1),
    CONSTRAINT metric_rollup_work_country_year_country_code_check
        CHECK (country_code ~ '^[A-Z]{2}$'),
    CONSTRAINT metric_rollup_work_country_year_watermark_check CHECK (watermark > 0),
    CONSTRAINT metric_rollup_work_country_year_identity_key
        UNIQUE NULLS NOT DISTINCT
        (work_id, publication_id, platform_id, measure_id, year_start, country_code),
    CONSTRAINT metric_rollup_work_country_year_work_id_fkey FOREIGN KEY (work_id)
        REFERENCES public.work(work_id),
    CONSTRAINT metric_rollup_work_country_year_publication_id_fkey
        FOREIGN KEY (publication_id) REFERENCES public.publication(publication_id),
    CONSTRAINT metric_rollup_work_country_year_platform_id_fkey FOREIGN KEY (platform_id)
        REFERENCES public.metric_platform(platform_id),
    CONSTRAINT metric_rollup_work_country_year_measure_id_fkey FOREIGN KEY (measure_id)
        REFERENCES public.metric_measure(measure_id)
);

-- ---------------------------------------------------------------------------
-- 2. Resolved yearly per-institution totals
-- ---------------------------------------------------------------------------
CREATE TABLE public.metric_rollup_work_institution_year (
    rollup_work_institution_year_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    work_id uuid NOT NULL,
    publication_id uuid,
    platform_id uuid NOT NULL,
    measure_id uuid NOT NULL,
    year_start date NOT NULL,
    institution_id uuid NOT NULL,
    value bigint NOT NULL,
    requires_country_coverage boolean NOT NULL,
    watermark bigint NOT NULL,
    CONSTRAINT metric_rollup_work_institution_year_pkey
        PRIMARY KEY (rollup_work_institution_year_id),
    CONSTRAINT metric_rollup_work_institution_year_year_start_check
        CHECK (EXTRACT(MONTH FROM year_start) = 1 AND EXTRACT(DAY FROM year_start) = 1),
    CONSTRAINT metric_rollup_work_institution_year_watermark_check CHECK (watermark > 0),
    CONSTRAINT metric_rollup_work_institution_year_identity_key
        UNIQUE NULLS NOT DISTINCT
        (work_id, publication_id, platform_id, measure_id, year_start, institution_id),
    CONSTRAINT metric_rollup_work_institution_year_work_id_fkey FOREIGN KEY (work_id)
        REFERENCES public.work(work_id),
    CONSTRAINT metric_rollup_work_institution_year_publication_id_fkey
        FOREIGN KEY (publication_id) REFERENCES public.publication(publication_id),
    CONSTRAINT metric_rollup_work_institution_year_platform_id_fkey
        FOREIGN KEY (platform_id) REFERENCES public.metric_platform(platform_id),
    CONSTRAINT metric_rollup_work_institution_year_measure_id_fkey
        FOREIGN KEY (measure_id) REFERENCES public.metric_measure(measure_id),
    CONSTRAINT metric_rollup_work_institution_year_institution_id_fkey
        FOREIGN KEY (institution_id) REFERENCES public.institution(institution_id)
);
