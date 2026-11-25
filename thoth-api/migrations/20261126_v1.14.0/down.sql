-- PUB-DASH-URL-01 (thoth-pub/thoth#1061): reverse of up.sql.
--
-- Drops the constraint and the column, and with it every stored dashboard URL.
-- Deploy a build that does not select `publisher.metrics_dashboard_url` first.

ALTER TABLE public.publisher
    DROP CONSTRAINT publisher_metrics_dashboard_url_check;

ALTER TABLE public.publisher
    DROP COLUMN metrics_dashboard_url;
