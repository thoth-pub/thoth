-- MET-QUARTER-01 (thoth-pub/thoth#1079), migration 1 of 2: the `QUARTER`
-- reporting grain.
--
-- PostgreSQL does not let a transaction use an enum value it has just added,
-- and Diesel runs each file in one transaction, so this file adds the value
-- and nothing else; migration 2 (20261129) uses it. `IF NOT EXISTS` makes a
-- redo safe. An image that predates this release cannot decode `QUARTER`
-- from `metric_record` or `metric_platform_measure`, so no registry pair may
-- list it until every process that reads Metrics tables runs this release.
ALTER TYPE public.metric_reporting_grain
    ADD VALUE IF NOT EXISTS 'QUARTER' AFTER 'MONTH';
