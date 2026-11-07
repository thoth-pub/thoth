-- MET-WP1-05: Metrics coverage foundation (issue #875).
--
-- Additive and initially inactive. Creates the closed coverage-status enum
-- and the single `metric_coverage` table: one durable record of what a
-- source account's import reported it covers for one platform/measure over
-- one period, independent of and prior to any canonical `metric_record`
-- computation.
--
-- This slice stores coverage structure only. There is deliberately no
-- coverage calculation, finalization, zero-versus-unknown behaviour or
-- normalized-ingestion/`ingestMetricBatch` transaction: those belong to
-- later bounded WP2/WP4 work. The module exposes no GraphQL or
-- administration surface, so the enum below is not a `juniper::GraphQLEnum`.
--
-- Foreign-key decision (reviewed): all four foreign keys — to
-- `metric_source_account`, `metric_import`, `metric_platform` and
-- `metric_measure` — are direct and deliberately non-cascading, so deleting a
-- referenced source account, import, platform or measure fails instead of
-- silently deleting coverage history.
--
-- Uniqueness decision (reviewed): no coverage uniqueness constraint beyond
-- the primary key is created. The approved design defines no coverage
-- uniqueness tuple, so none is invented here; whether a source account may
-- report overlapping or duplicate coverage for the same platform/measure/
-- period is a later WP2/WP4 semantic question, not a database invariant at
-- this foundation stage.
--
-- Index decision (reviewed): the complete intended index set is exactly the
-- primary key. No speculative secondary index is created: the approved
-- design defines no coverage-specific access pattern at this stage.
--
-- Period decision (reviewed): `period_start`/`period_end` follow the
-- programme's half-open period convention. Only ordering
-- (`period_end > period_start`) is enforced; overlap detection is
-- deliberately out of scope for this slice.
--
-- No coverage row is seeded.

CREATE TYPE public.metric_coverage_status AS ENUM (
    'COMPLETE',
    'PARTIAL',
    'UNKNOWN'
);

-- metric_coverage is one durable record of what a source account's import
-- reported it covers for one platform/measure over one half-open period.
-- `country_coverage` and `institution_coverage` record whether that reported
-- coverage includes the country and institution dimensions respectively;
-- both are plain non-null booleans, not the closed status enum, because they
-- describe dimension presence rather than completeness. `notes` is an
-- optional free-text annotation.
CREATE TABLE public.metric_coverage (
    coverage_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    source_account_id uuid NOT NULL,
    import_id uuid NOT NULL,
    platform_id uuid NOT NULL,
    measure_id uuid NOT NULL,
    period_start date NOT NULL,
    period_end date NOT NULL,
    coverage_status public.metric_coverage_status NOT NULL,
    country_coverage boolean NOT NULL,
    institution_coverage boolean NOT NULL,
    notes text,
    CONSTRAINT metric_coverage_pkey PRIMARY KEY (coverage_id),
    -- Periods are half-open calendar-date ranges: period_start inclusive,
    -- period_end exclusive. Ordering is enforced; overlap is not.
    CONSTRAINT metric_coverage_period_check CHECK (period_end > period_start),
    CONSTRAINT metric_coverage_source_account_id_fkey FOREIGN KEY (source_account_id)
        REFERENCES public.metric_source_account(source_account_id),
    CONSTRAINT metric_coverage_import_id_fkey FOREIGN KEY (import_id)
        REFERENCES public.metric_import(import_id),
    CONSTRAINT metric_coverage_platform_id_fkey FOREIGN KEY (platform_id)
        REFERENCES public.metric_platform(platform_id),
    CONSTRAINT metric_coverage_measure_id_fkey FOREIGN KEY (measure_id)
        REFERENCES public.metric_measure(measure_id)
);
