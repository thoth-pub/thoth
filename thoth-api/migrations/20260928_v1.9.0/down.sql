-- MET-WP4-03C: remove the two derived yearly dashboard section projections.
--
-- Both tables hold derived, rebuildable state only, so dropping them loses
-- nothing that a later `rebuildMetricRollupMonths` cannot recreate from the
-- work-day projection; no canonical table, monthly table, work-day row,
-- rollup delta or frontier value is touched. Each drop takes ACCESS
-- EXCLUSIVE on its own table for the duration of this transaction and
-- releases the non-cascading foreign keys on work, publication,
-- metric_platform, metric_measure and institution.
DROP TABLE public.metric_rollup_work_institution_year;
DROP TABLE public.metric_rollup_work_country_year;
