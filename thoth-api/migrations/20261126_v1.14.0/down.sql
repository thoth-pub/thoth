-- PUB-DASH-URL-01 (thoth-pub/thoth#1061): reverse of up.sql.
--
-- Drops the constraint and the column, and with it every stored dashboard URL
-- (lossy by design: there is no other copy; set the URLs again after a
-- re-apply). Deploy a build that does not select
-- `publisher.metrics_dashboard_url` before running it, because the API build
-- that has the column in its model fails every publisher read without it.
--
-- Locks: `ACCESS EXCLUSIVE` on `publisher` for a catalogue change, bounded by
-- `lock_timeout`; a timeout leaves nothing changed and the file is run again.

SET LOCAL lock_timeout = '10s';
SET LOCAL statement_timeout = '60s';

ALTER TABLE public.publisher
    DROP CONSTRAINT publisher_metrics_dashboard_url_check;

ALTER TABLE public.publisher
    DROP COLUMN metrics_dashboard_url;
