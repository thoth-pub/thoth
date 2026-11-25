-- PUB-DASH-URL-01 (thoth-pub/thoth#1061): the public URL of a publisher's
-- metrics dashboard.
--
-- One nullable column, NULL on every existing row, so no data migration and no
-- default. It is written only by the superuser mutation
-- `setPublisherMetricsDashboardUrl`; `NewPublisher` and `PatchPublisher` do not
-- carry it. The CHECK is the backstop for the same rules the API enforces:
-- `https` only, a host, no user information, no whitespace or control characters, at most 2048
-- characters.
--
-- Locks: `ADD COLUMN` without a default takes a brief `ACCESS EXCLUSIVE` lock
-- on `publisher` and rewrites nothing. Adding a CHECK validates the (all NULL)
-- column against a catalogue-only scan of existing rows.

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
