-- MET-WP4-MONTH-01 (thoth-pub/thoth#1042): reverse of up.sql.
--
-- Refuses, with a fixed message, while any metric source account has a
-- window or a succession: clear them first with
-- `updateMetricSourceAccountWindow`. Otherwise it drops the two constraints,
-- the three columns and the four native-month tables, whose rows are derived
-- and are rebuilt by `up.sql`.
--
-- The account lock comes first because ingestion takes it first. An `ON`
-- read takes the native tables before the accounts, so this file can deadlock
-- with one; an aborted run is a clean retry.
--
-- Never run `thoth migrate --revert`, which reverts every migration. A
-- rollback runs this file and deletes this migration's row from
-- `__diesel_schema_migrations` in one transaction, under its own grant.

SET LOCAL lock_timeout = '10s';
SET LOCAL statement_timeout = '60s';
LOCK TABLE public.metric_source_account IN ACCESS EXCLUSIVE MODE;

DO $met_wp4_month_01$
BEGIN
    IF EXISTS (SELECT 1 FROM public.metric_source_account
               WHERE window_start IS NOT NULL
                  OR window_end IS NOT NULL
                  OR succession IS NOT NULL) THEN
        RAISE EXCEPTION 'MET-WP4-MONTH-01 rollback refused: a metric source account has a window or a succession; clear every window and succession first';
    END IF;
END
$met_wp4_month_01$;

ALTER TABLE public.metric_source_account
    DROP CONSTRAINT metric_source_account_window_check,
    DROP CONSTRAINT metric_source_account_succession_check,
    DROP COLUMN window_start,
    DROP COLUMN window_end,
    DROP COLUMN succession;

DROP TABLE public.metric_native_refusal;
DROP TABLE public.metric_native_institution_month;
DROP TABLE public.metric_native_country_month;
DROP TABLE public.metric_native_month;
