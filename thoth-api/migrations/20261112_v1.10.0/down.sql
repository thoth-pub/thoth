-- MET-WP4-03A-OPS-03 downgrade: remove exactly the run table and its index.
--
-- The table holds run evidence only: no canonical row, no work-day row and no
-- derived projection row lives in it, and nothing else references it, so
-- dropping it loses only the history of runs. Projections need no rollback:
-- every committed build step left its range exact, and an interrupted step
-- rolled back.
--
-- Order: abandon any open run, then run this downgrade with the binary that
-- carries the run operations, then deploy the previous binary (which ignores
-- the table either way). No `IF EXISTS` is used, so an object that was
-- already removed or renamed fails this downgrade loudly instead of being
-- skipped.

DROP INDEX public.metric_rollup_month_rebuild_run_one_open_idx;

DROP TABLE public.metric_rollup_month_rebuild_run;
