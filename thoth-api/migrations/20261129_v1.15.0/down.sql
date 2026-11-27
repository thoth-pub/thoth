-- MET-QUARTER-01 migration 2 of 2: reverse of up.sql.
--
-- Fails closed, with a fixed message, while any native row has a span of 3 or
-- any registry pair lists `QUARTER`: from the first `QUARTER` record on, only a
-- forward fix exists. The `QUARTER` enum value itself stays (20261128 down).

SET LOCAL lock_timeout = '10s';
SET LOCAL statement_timeout = '60s';

LOCK TABLE public.metric_platform_measure,
           public.metric_native_month,
           public.metric_native_country_month,
           public.metric_native_institution_month
    IN ACCESS EXCLUSIVE MODE;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM public.metric_native_month WHERE span_months <> 1)
       OR EXISTS (SELECT 1 FROM public.metric_native_country_month WHERE span_months <> 1)
       OR EXISTS (SELECT 1 FROM public.metric_native_institution_month WHERE span_months <> 1)
       OR EXISTS (SELECT 1 FROM public.metric_platform_measure
                  WHERE 'QUARTER'::public.metric_reporting_grain = ANY (supported_grains))
       OR EXISTS (SELECT 1 FROM public.metric_record WHERE reporting_grain = 'QUARTER')
    THEN
        RAISE EXCEPTION 'MET-QUARTER-01 down refused: QUARTER data or registry entries exist';
    END IF;
END
$$;

ALTER TABLE public.metric_native_institution_month
    DROP CONSTRAINT metric_native_institution_month_month_start_check,
    DROP CONSTRAINT metric_native_institution_month_span_months_check,
    DROP COLUMN span_months,
    ADD CONSTRAINT metric_native_institution_month_month_start_check
        CHECK (EXTRACT(DAY FROM month_start) = 1);

ALTER TABLE public.metric_native_country_month
    DROP CONSTRAINT metric_native_country_month_month_start_check,
    DROP CONSTRAINT metric_native_country_month_span_months_check,
    DROP COLUMN span_months,
    ADD CONSTRAINT metric_native_country_month_month_start_check
        CHECK (EXTRACT(DAY FROM month_start) = 1);

ALTER TABLE public.metric_native_month
    DROP CONSTRAINT metric_native_month_month_start_check,
    DROP CONSTRAINT metric_native_month_span_months_check,
    DROP COLUMN span_months,
    ADD CONSTRAINT metric_native_month_month_start_check
        CHECK (EXTRACT(DAY FROM month_start) = 1);

ALTER TABLE public.metric_platform_measure
    DROP CONSTRAINT metric_platform_measure_supported_grains_check,
    ADD CONSTRAINT metric_platform_measure_supported_grains_check CHECK (
        cardinality(supported_grains) > 0
        AND array_position(supported_grains, NULL) IS NULL
        AND cardinality(array_positions(
            supported_grains, 'DAY'::public.metric_reporting_grain)) <= 1
        AND cardinality(array_positions(
            supported_grains, 'MONTH'::public.metric_reporting_grain)) <= 1
        AND cardinality(array_positions(
            supported_grains, 'REPORTING_PERIOD'::public.metric_reporting_grain)) <= 1
    );
