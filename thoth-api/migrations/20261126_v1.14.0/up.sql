-- PUB-DASH-URL-01 (thoth-pub/thoth#1061): the public URL of a publisher's
-- metrics dashboard.
--
-- Added: column `publisher.metrics_dashboard_url` (text, nullable, no default)
-- and constraint `publisher_metrics_dashboard_url_check`. No tables, enums or
-- indexes.
--
-- Existing data: every row gets NULL, so there is no backfill and the
-- constraint holds for every existing row. The column is written only by the
-- superuser mutation `setPublisherMetricsDashboardUrl`; `NewPublisher` and
-- `PatchPublisher` do not carry it. The CHECK is the backstop for the same
-- rules the API enforces: `https` only, a host, no user information, no
-- whitespace or control characters, at most 2048 characters.
--
-- Locks and runtime: `ADD COLUMN` without a default is a catalogue change and
-- rewrites nothing; `ADD CONSTRAINT ... CHECK` validates every existing row,
-- so it scans `publisher` once (a table of a few hundred rows, milliseconds).
-- Both take `ACCESS EXCLUSIVE` on `publisher` and are held to commit, and
-- `publisher` is read by every API request and locked `FOR SHARE` by Metrics
-- ingestion transactions. The lock wait is therefore bounded below: if it
-- cannot be taken within `lock_timeout` the migration fails closed.
--
-- Retry and idempotency: the file runs in one transaction and diesel records
-- the version only on success, so a timeout or any failure leaves nothing
-- behind and the migration is simply run again. It is not safe to run twice by
-- hand (the second `ADD COLUMN` fails).
--
-- Deployment order: run the migration before the new API starts serving. The
-- previous API build selects explicit publisher columns, so it keeps working
-- against the new column; the new build needs the column. Down: see down.sql.

SET LOCAL lock_timeout = '10s';
SET LOCAL statement_timeout = '60s';

ALTER TABLE public.publisher
    ADD COLUMN metrics_dashboard_url text;

ALTER TABLE public.publisher
    ADD CONSTRAINT publisher_metrics_dashboard_url_check
    CHECK (
        metrics_dashboard_url IS NULL
        OR (
            char_length(metrics_dashboard_url) <= 2048
            AND metrics_dashboard_url ~ '^https://[^/@?#[:space:][:cntrl:]]+(/[^[:space:][:cntrl:]]*)?$'
        )
    );
