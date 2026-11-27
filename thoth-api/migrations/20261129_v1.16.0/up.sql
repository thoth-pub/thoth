-- MET-QUARTER-01 (thoth-pub/thoth#1079), migration 2 of 2: native periods of
-- one month or one quarter, and one native grain per platform-measure pair.
--
-- 1. The three native month tables gain `span_months` (1 or 3). The column
--    `month_start` keeps its name, so tasks that still run the previous image
--    keep working while this release rolls out; for span 3 it is the first
--    day of a calendar quarter.
-- 2. `metric_platform_measure.supported_grains` may not list both `MONTH` and
--    `QUARTER`.
--
-- Locks: ingestion holds registry rows `FOR SHARE` until it has written the
-- native tables, so the registry table is locked first, then the native
-- tables; otherwise this file could deadlock with an ingestion transaction.
-- A bound that fires fails the migration closed.

SET LOCAL lock_timeout = '10s';
SET LOCAL statement_timeout = '60s';

LOCK TABLE public.metric_platform_measure IN ACCESS EXCLUSIVE MODE;

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
            supported_grains, 'QUARTER'::public.metric_reporting_grain)) <= 1
        AND cardinality(array_positions(
            supported_grains, 'REPORTING_PERIOD'::public.metric_reporting_grain)) <= 1
        AND NOT (
            'MONTH'::public.metric_reporting_grain = ANY (supported_grains)
            AND 'QUARTER'::public.metric_reporting_grain = ANY (supported_grains)
        )
    );

ALTER TABLE public.metric_native_month
    DROP CONSTRAINT metric_native_month_month_start_check,
    ADD COLUMN span_months smallint NOT NULL DEFAULT 1,
    ADD CONSTRAINT metric_native_month_span_months_check
        CHECK (span_months IN (1, 3)),
    ADD CONSTRAINT metric_native_month_month_start_check
        CHECK (EXTRACT(DAY FROM month_start) = 1
               AND (span_months = 1 OR EXTRACT(MONTH FROM month_start) IN (1, 4, 7, 10)));

ALTER TABLE public.metric_native_country_month
    DROP CONSTRAINT metric_native_country_month_month_start_check,
    ADD COLUMN span_months smallint NOT NULL DEFAULT 1,
    ADD CONSTRAINT metric_native_country_month_span_months_check
        CHECK (span_months IN (1, 3)),
    ADD CONSTRAINT metric_native_country_month_month_start_check
        CHECK (EXTRACT(DAY FROM month_start) = 1
               AND (span_months = 1 OR EXTRACT(MONTH FROM month_start) IN (1, 4, 7, 10)));

ALTER TABLE public.metric_native_institution_month
    DROP CONSTRAINT metric_native_institution_month_month_start_check,
    ADD COLUMN span_months smallint NOT NULL DEFAULT 1,
    ADD CONSTRAINT metric_native_institution_month_span_months_check
        CHECK (span_months IN (1, 3)),
    ADD CONSTRAINT metric_native_institution_month_month_start_check
        CHECK (EXTRACT(DAY FROM month_start) = 1
               AND (span_months = 1 OR EXTRACT(MONTH FROM month_start) IN (1, 4, 7, 10)));
