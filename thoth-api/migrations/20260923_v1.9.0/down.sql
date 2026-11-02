-- MET-WP4-03A downgrade: remove only the four derived monthly serving tables.
--
-- All four tables are derived, rebuildable state whose only source is the
-- untouched `metric_rollup_work_day` projection, so dropping them erases no
-- canonical accounting, no rollup delta, no claim evidence and no frontier
-- position. That is the difference from the guarded MET-WP4-01 downgrade:
-- there, the dropped state was the only record of what had been applied;
-- here, a fresh rebuild from the surviving work-day projection reproduces
-- every row, value, dependency flag, ambiguity flag and watermark exactly.
--
-- Deployment order (reviewed): roll back the application code first and
-- verify no running completion transaction still references these tables,
-- then run this downgrade. After a later reapplication the four tables are
-- empty again and must be repopulated by a separately authorized historical
-- rebuild and reconciled at the recorded work-day frontier before any reader
-- may serve from them; the completion transaction alone maintains only the
-- months it touches from that point on.
--
-- Each table holds no dependants and this migration created no enum,
-- trigger, sequence or standalone index, so four non-cascading DROP TABLEs
-- are exact. Primary-key and identity indexes are dropped implicitly. The
-- MET-WP1-01..13, MET-WP2-01A, MET-WP4-01 and MET-WP7-PREREQ-02/03 schema,
-- `metric_rollup_work_day`, `metric_rollup_work_day_state`, every rollup
-- delta and the bibliographic schema are untouched.

DROP TABLE IF EXISTS public.metric_rollup_work_month_ambiguity;

DROP TABLE IF EXISTS public.metric_rollup_work_institution_month;

DROP TABLE IF EXISTS public.metric_rollup_work_country_month;

DROP TABLE IF EXISTS public.metric_rollup_work_month;
