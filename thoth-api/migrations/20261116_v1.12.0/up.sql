-- MET-WP4-MONTH-01 (thoth-pub/thoth#1042): account date windows and
-- successions, and the native-month projection.
--
-- 1. `metric_source_account` gains an optional half-open date window
--    `[window_start, window_end)` and an optional succession label, each
--    `NULL` on every existing row, with two CHECKs.
-- 2. Four derived tables hold, per work, platform, measure and month, the
--    resolved total, country and institution values of the `MONTH` records,
--    and every non-`DAY` period that must refuse a read. They have no foreign
--    key, trigger or secondary index; the primary key is the identity, work
--    first. The names avoid the `metric_rollup` and `metric_record` prefixes.
-- 3. The four tables are populated from every non-`DAY` record by the
--    projection rule of `thoth-api/src/model/metric_native_month/mod.rs`.
--
-- Locks: the `ALTER TABLE` takes `ACCESS EXCLUSIVE` on `metric_source_account`,
-- the only lock above `ACCESS SHARE` on an existing table, so this file cannot
-- deadlock. It waits at most `lock_timeout` for ingestion and reconciliation
-- transactions in flight (they hold an account `FOR SHARE`) and is held to
-- commit, so no canonical write runs while step 3 reads and the tables are
-- exact from their first commit. A bound that fires fails the migration
-- closed.

SET LOCAL lock_timeout = '10s';
SET LOCAL statement_timeout = '60s';

-- ---------------------------------------------------------------------------
-- 1. Account windows and successions
-- ---------------------------------------------------------------------------

ALTER TABLE public.metric_source_account
    ADD COLUMN window_start date NULL,
    ADD COLUMN window_end date NULL,
    ADD COLUMN succession text NULL,
    ADD CONSTRAINT metric_source_account_window_check
        CHECK (window_start IS NULL OR window_end IS NULL OR window_end > window_start),
    ADD CONSTRAINT metric_source_account_succession_check
        CHECK (succession IS NULL
               OR (char_length(succession) <= 64
                   AND succession ~ '^[a-z0-9]+(-[a-z0-9]+)*$'));

-- ---------------------------------------------------------------------------
-- 2. The native-month projection
-- ---------------------------------------------------------------------------

-- One row per key holding a served month. `value` is `NULL` when the cell is
-- total-ambiguous; `live` says some served record's value is not 0.
CREATE TABLE public.metric_native_month (
    work_id uuid NOT NULL,
    platform_id uuid NOT NULL,
    measure_id uuid NOT NULL,
    month_start date NOT NULL,
    value bigint NULL,
    live boolean NOT NULL,
    requires_country_coverage boolean NOT NULL,
    requires_institution_coverage boolean NOT NULL,
    country_ambiguous boolean NOT NULL,
    institution_ambiguous boolean NOT NULL,
    CONSTRAINT metric_native_month_pkey
        PRIMARY KEY (work_id, platform_id, measure_id, month_start),
    CONSTRAINT metric_native_month_month_start_check
        CHECK (EXTRACT(DAY FROM month_start) = 1)
);

CREATE TABLE public.metric_native_country_month (
    work_id uuid NOT NULL,
    platform_id uuid NOT NULL,
    measure_id uuid NOT NULL,
    month_start date NOT NULL,
    country_code character(2) NOT NULL,
    value bigint NOT NULL,
    requires_institution_coverage boolean NOT NULL,
    CONSTRAINT metric_native_country_month_pkey
        PRIMARY KEY (work_id, platform_id, measure_id, month_start, country_code),
    CONSTRAINT metric_native_country_month_month_start_check
        CHECK (EXTRACT(DAY FROM month_start) = 1),
    CONSTRAINT metric_native_country_month_country_code_check
        CHECK (country_code ~ '^[A-Z]{2}$')
);

CREATE TABLE public.metric_native_institution_month (
    work_id uuid NOT NULL,
    platform_id uuid NOT NULL,
    measure_id uuid NOT NULL,
    month_start date NOT NULL,
    institution_id uuid NOT NULL,
    value bigint NOT NULL,
    requires_country_coverage boolean NOT NULL,
    CONSTRAINT metric_native_institution_month_pkey
        PRIMARY KEY (work_id, platform_id, measure_id, month_start, institution_id),
    CONSTRAINT metric_native_institution_month_month_start_check
        CHECK (EXTRACT(DAY FROM month_start) = 1)
);

-- One row per non-`DAY` period holding an active `REPORTING_PERIOD` record or
-- an inconsistent record of either grain.
CREATE TABLE public.metric_native_refusal (
    work_id uuid NOT NULL,
    platform_id uuid NOT NULL,
    measure_id uuid NOT NULL,
    period_start date NOT NULL,
    period_end date NOT NULL,
    inconsistent boolean NOT NULL,
    CONSTRAINT metric_native_refusal_pkey
        PRIMARY KEY (work_id, platform_id, measure_id, period_start, period_end)
);

-- ---------------------------------------------------------------------------
-- 3. Populate from every non-`DAY` record
-- ---------------------------------------------------------------------------
--
-- Each record is classified by its revision pointer: withdrawn (a `RETRACTED`
-- revision and no `CURRENT` one) is ignored; an active reporting period (a
-- `REPORTING_PERIOD` record pointing at a `CURRENT` revision) and an
-- inconsistent record (anything else that is not a served month) refuse; a
-- served month (a `MONTH` record of exactly one calendar month pointing at a
-- `CURRENT` revision of itself) is resolved as 03A resolves a day: the
-- undimensioned row if present, else the single represented mask, summed,
-- else total-ambiguous; sections by the unique least target-bearing mask.

INSERT INTO public.metric_native_refusal
    (work_id, platform_id, measure_id, period_start, period_end, inconsistent)
SELECT c.work_id, c.platform_id, c.measure_id, c.period_start, c.period_end,
       bool_or(NOT c.active_reporting_period)
FROM (
    SELECT r.work_id, r.platform_id, r.measure_id, r.period_start, r.period_end,
           (r.reporting_grain = 'REPORTING_PERIOD'
            AND COALESCE(v.status = 'CURRENT', FALSE)) AS active_reporting_period,
           (r.reporting_grain = 'MONTH'
            AND COALESCE(v.status = 'CURRENT', FALSE)
            AND EXTRACT(DAY FROM r.period_start) = 1
            AND r.period_end = (r.period_start + interval '1 month')::date) AS served_month,
           (COALESCE(v.status = 'RETRACTED', FALSE)
            AND NOT EXISTS (SELECT 1 FROM public.metric_record_revision o
                            WHERE o.record_id = r.record_id
                              AND o.status = 'CURRENT')) AS withdrawn
    FROM public.metric_record r
    LEFT JOIN public.metric_record_revision v
      ON v.record_revision_id = r.current_revision_id
     AND v.record_id = r.record_id
    WHERE r.reporting_grain <> 'DAY'
) c
WHERE NOT c.served_month
  AND NOT c.withdrawn
GROUP BY c.work_id, c.platform_id, c.measure_id, c.period_start, c.period_end;

WITH served AS (
    SELECT x.*,
           CASE WHEN x.represented & 1 <> 0 THEN 0
                WHEN x.represented = 2 THEN 1
                WHEN x.represented = 4 THEN 2
                WHEN x.represented = 8 THEN 3
                WHEN x.represented = 16 THEN 4
                WHEN x.represented = 32 THEN 5
                WHEN x.represented = 64 THEN 6
                WHEN x.represented = 128 THEN 7
           END AS total_mask,
           CASE WHEN x.represented & 4 <> 0 THEN 2
                WHEN x.represented & 8 <> 0 AND x.represented & 64 <> 0 THEN NULL
                WHEN x.represented & 8 <> 0 THEN 3
                WHEN x.represented & 64 <> 0 THEN 6
                WHEN x.represented & 128 <> 0 THEN 7
           END AS country_mask,
           CASE WHEN x.represented & 2 <> 0 THEN 1
                WHEN x.represented & 8 <> 0 AND x.represented & 32 <> 0 THEN NULL
                WHEN x.represented & 8 <> 0 THEN 3
                WHEN x.represented & 32 <> 0 THEN 5
                WHEN x.represented & 128 <> 0 THEN 7
           END AS institution_mask
    FROM (
        SELECT s.*,
               bit_or(1 << s.mask) OVER (PARTITION BY s.work_id, s.platform_id,
                                                      s.measure_id, s.month_start) AS represented
        FROM (
            SELECT r.work_id, r.platform_id, r.measure_id, r.period_start AS month_start,
                   r.country_code, r.institution_id, v.value,
                   (r.publication_id IS NOT NULL)::int * 4
                       + (r.country_code IS NOT NULL)::int * 2
                       + (r.institution_id IS NOT NULL)::int AS mask
            FROM public.metric_record r
            JOIN public.metric_record_revision v
              ON v.record_revision_id = r.current_revision_id
             AND v.record_id = r.record_id
            WHERE r.reporting_grain <> 'DAY'
              AND r.reporting_grain = 'MONTH'
              AND v.status = 'CURRENT'
              AND EXTRACT(DAY FROM r.period_start) = 1
              AND r.period_end = (r.period_start + interval '1 month')::date
        ) s
    ) x
)
INSERT INTO public.metric_native_month
    (work_id, platform_id, measure_id, month_start, value, live,
     requires_country_coverage, requires_institution_coverage,
     country_ambiguous, institution_ambiguous)
SELECT s.work_id, s.platform_id, s.measure_id, s.month_start,
       (SUM(s.value) FILTER (WHERE s.mask = s.total_mask))::bigint,
       bool_or(s.value <> 0),
       COALESCE(bool_or(s.mask & 2 <> 0) FILTER (WHERE s.mask = s.total_mask), FALSE),
       COALESCE(bool_or(s.mask & 1 <> 0) FILTER (WHERE s.mask = s.total_mask), FALSE),
       bool_or(s.represented & 4 = 0 AND s.represented & 8 <> 0 AND s.represented & 64 <> 0),
       bool_or(s.represented & 2 = 0 AND s.represented & 8 <> 0 AND s.represented & 32 <> 0)
FROM served s
GROUP BY s.work_id, s.platform_id, s.measure_id, s.month_start;

WITH served AS (
    SELECT x.*,
           CASE WHEN x.represented & 1 <> 0 THEN 0
                WHEN x.represented = 2 THEN 1
                WHEN x.represented = 4 THEN 2
                WHEN x.represented = 8 THEN 3
                WHEN x.represented = 16 THEN 4
                WHEN x.represented = 32 THEN 5
                WHEN x.represented = 64 THEN 6
                WHEN x.represented = 128 THEN 7
           END AS total_mask,
           CASE WHEN x.represented & 4 <> 0 THEN 2
                WHEN x.represented & 8 <> 0 AND x.represented & 64 <> 0 THEN NULL
                WHEN x.represented & 8 <> 0 THEN 3
                WHEN x.represented & 64 <> 0 THEN 6
                WHEN x.represented & 128 <> 0 THEN 7
           END AS country_mask,
           CASE WHEN x.represented & 2 <> 0 THEN 1
                WHEN x.represented & 8 <> 0 AND x.represented & 32 <> 0 THEN NULL
                WHEN x.represented & 8 <> 0 THEN 3
                WHEN x.represented & 32 <> 0 THEN 5
                WHEN x.represented & 128 <> 0 THEN 7
           END AS institution_mask
    FROM (
        SELECT s.*,
               bit_or(1 << s.mask) OVER (PARTITION BY s.work_id, s.platform_id,
                                                      s.measure_id, s.month_start) AS represented
        FROM (
            SELECT r.work_id, r.platform_id, r.measure_id, r.period_start AS month_start,
                   r.country_code, r.institution_id, v.value,
                   (r.publication_id IS NOT NULL)::int * 4
                       + (r.country_code IS NOT NULL)::int * 2
                       + (r.institution_id IS NOT NULL)::int AS mask
            FROM public.metric_record r
            JOIN public.metric_record_revision v
              ON v.record_revision_id = r.current_revision_id
             AND v.record_id = r.record_id
            WHERE r.reporting_grain <> 'DAY'
              AND r.reporting_grain = 'MONTH'
              AND v.status = 'CURRENT'
              AND EXTRACT(DAY FROM r.period_start) = 1
              AND r.period_end = (r.period_start + interval '1 month')::date
        ) s
    ) x
)
INSERT INTO public.metric_native_country_month
    (work_id, platform_id, measure_id, month_start, country_code, value,
     requires_institution_coverage)
SELECT s.work_id, s.platform_id, s.measure_id, s.month_start, s.country_code,
       SUM(s.value)::bigint,
       bool_or(s.mask & 1 <> 0)
FROM served s
WHERE s.mask = s.country_mask
GROUP BY s.work_id, s.platform_id, s.measure_id, s.month_start, s.country_code;

WITH served AS (
    SELECT x.*,
           CASE WHEN x.represented & 1 <> 0 THEN 0
                WHEN x.represented = 2 THEN 1
                WHEN x.represented = 4 THEN 2
                WHEN x.represented = 8 THEN 3
                WHEN x.represented = 16 THEN 4
                WHEN x.represented = 32 THEN 5
                WHEN x.represented = 64 THEN 6
                WHEN x.represented = 128 THEN 7
           END AS total_mask,
           CASE WHEN x.represented & 4 <> 0 THEN 2
                WHEN x.represented & 8 <> 0 AND x.represented & 64 <> 0 THEN NULL
                WHEN x.represented & 8 <> 0 THEN 3
                WHEN x.represented & 64 <> 0 THEN 6
                WHEN x.represented & 128 <> 0 THEN 7
           END AS country_mask,
           CASE WHEN x.represented & 2 <> 0 THEN 1
                WHEN x.represented & 8 <> 0 AND x.represented & 32 <> 0 THEN NULL
                WHEN x.represented & 8 <> 0 THEN 3
                WHEN x.represented & 32 <> 0 THEN 5
                WHEN x.represented & 128 <> 0 THEN 7
           END AS institution_mask
    FROM (
        SELECT s.*,
               bit_or(1 << s.mask) OVER (PARTITION BY s.work_id, s.platform_id,
                                                      s.measure_id, s.month_start) AS represented
        FROM (
            SELECT r.work_id, r.platform_id, r.measure_id, r.period_start AS month_start,
                   r.country_code, r.institution_id, v.value,
                   (r.publication_id IS NOT NULL)::int * 4
                       + (r.country_code IS NOT NULL)::int * 2
                       + (r.institution_id IS NOT NULL)::int AS mask
            FROM public.metric_record r
            JOIN public.metric_record_revision v
              ON v.record_revision_id = r.current_revision_id
             AND v.record_id = r.record_id
            WHERE r.reporting_grain <> 'DAY'
              AND r.reporting_grain = 'MONTH'
              AND v.status = 'CURRENT'
              AND EXTRACT(DAY FROM r.period_start) = 1
              AND r.period_end = (r.period_start + interval '1 month')::date
        ) s
    ) x
)
INSERT INTO public.metric_native_institution_month
    (work_id, platform_id, measure_id, month_start, institution_id, value,
     requires_country_coverage)
SELECT s.work_id, s.platform_id, s.measure_id, s.month_start, s.institution_id,
       SUM(s.value)::bigint,
       bool_or(s.mask & 2 <> 0)
FROM served s
WHERE s.mask = s.institution_mask
GROUP BY s.work_id, s.platform_id, s.measure_id, s.month_start, s.institution_id;
