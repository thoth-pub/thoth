-- CITATIONS-01 C1 (thoth-pub/thoth#1074): reverse of up.sql.
--
-- Refuses, with a fixed message, while any of the five citation tables holds a
-- row: the rows are collected data and are not rebuilt by `up.sql`. Otherwise
-- drops the five tables and the three enum types.
--
-- Never run `thoth migrate --revert`, which reverts every migration. A
-- rollback runs this file and deletes this migration's row from
-- `__diesel_schema_migrations` in one transaction, under its own grant.

SET LOCAL lock_timeout = '10s';
SET LOCAL statement_timeout = '60s';

LOCK TABLE public.metric_citation_link,
           public.metric_citation_work_state,
           public.metric_citation_refresh,
           public.metric_citation_refresh_seen,
           public.metric_citation_batch
    IN ACCESS EXCLUSIVE MODE;

DO $citations_01$
BEGIN
    IF EXISTS (SELECT 1 FROM public.metric_citation_link)
       OR EXISTS (SELECT 1 FROM public.metric_citation_work_state)
       OR EXISTS (SELECT 1 FROM public.metric_citation_refresh)
       OR EXISTS (SELECT 1 FROM public.metric_citation_refresh_seen)
       OR EXISTS (SELECT 1 FROM public.metric_citation_batch) THEN
        RAISE EXCEPTION 'CITATIONS-01 rollback refused: a citation table holds rows; export or delete them first under their own grant';
    END IF;
END
$citations_01$;

DROP TABLE public.metric_citation_batch;
DROP TABLE public.metric_citation_refresh_seen;
DROP TABLE public.metric_citation_refresh;
DROP TABLE public.metric_citation_link;
DROP TABLE public.metric_citation_work_state;

DROP TYPE public.metric_citation_refresh_status;
DROP TYPE public.metric_citation_link_status;
DROP TYPE public.metric_citation_match_status;
