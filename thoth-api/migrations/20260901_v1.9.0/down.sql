-- MET-WP1-05 downgrade: remove only the schema introduced by this migration.
--
-- metric_coverage holds no dependants, so it is dropped first, then its
-- status enum. The MET-WP1-01 registry schema, the MET-WP1-02 source-state
-- schema, the MET-WP1-03 import-state schema and the MET-WP1-04 canonical
-- history schema are untouched, as is the metric_coverage primary key index,
-- which is dropped implicitly with the table.

DROP TABLE IF EXISTS public.metric_coverage;

DROP TYPE IF EXISTS public.metric_coverage_status;
