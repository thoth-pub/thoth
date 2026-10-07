-- MET-WP4-03C-B1 downgrade: remove exactly the B1-owned derived objects.
--
-- Everything removed here is derived, rebuildable state or an access path
-- over untouched canonical tables: the coverage-run trigger and its
-- function, the four B1 secondary indexes and the `metric_coverage_run`
-- table. No `metric_coverage`, `metric_import`, `metric_record` or
-- `metric_source_account` row is read, modified or removed, so dropping
-- these objects erases no raw coverage evidence, no import evidence and no
-- canonical record; a later reapplication recreates the same definitions
-- empty, and a separately authorized rebuild reproduces every run exactly
-- from the surviving raw evidence.
--
-- Deployment order (reviewed): roll back the application code first, so no
-- running verification or rebuild references `metric_coverage_run`, then
-- run this downgrade. Dropping the trigger while a terminalization is in
-- flight waits for that transaction to finish (`DROP TRIGGER` takes
-- `ACCESS EXCLUSIVE` on `metric_import`); no terminalization is lost.
--
-- Order: the trigger first (it references the function), then the function,
-- then the four indexes, then the table (its primary-key index is dropped
-- implicitly). The two statements that remove the SQL objects are exactly
-- the frozen forms of the B1 handoff; no pattern-based or `IF EXISTS`
-- deletion is used, so a definition that was already removed, renamed or
-- never created fails this downgrade loudly instead of being skipped. No
-- pre-existing function or trigger is touched.

DROP TRIGGER metric_import_maintain_coverage_runs
    ON public.metric_import;

DROP FUNCTION public.maintain_metric_coverage_runs_from_import();

DROP INDEX public.metric_record_native_grain_idx;

DROP INDEX public.metric_import_terminal_without_completion_idx;

DROP INDEX public.metric_coverage_source_account_id_period_end_idx;

DROP INDEX public.metric_coverage_import_id_idx;

DROP TABLE public.metric_coverage_run;
