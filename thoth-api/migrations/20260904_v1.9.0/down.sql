-- MET-WP1-08 downgrade: remove only the schema introduced by this migration.
--
-- metric_operas_mapping holds no dependants — nothing references it, and this
-- slice created no enum type, trigger, sequence or standalone index — so a
-- single non-cascading DROP TABLE is exact. Its primary-key index and its
-- `(platform_id, measure_id)` uniqueness index are dropped implicitly with the
-- table.
--
-- The MET-WP1-01 registry schema and its measure seed rows, the MET-WP1-02
-- source-state schema, the MET-WP1-03 import-state schema, the MET-WP1-04
-- canonical record/revision/provenance schema and its enums, the MET-WP1-05
-- coverage schema, the MET-WP1-06 publisher-platform approval schema and the
-- MET-WP1-07 rollup-delta schema are all untouched, as is the existing
-- bibliographic schema. In particular the referenced
-- `metric_platform_measure (platform_id, measure_id)` unique key belongs to
-- MET-WP1-01 and must survive.

DROP TABLE IF EXISTS public.metric_operas_mapping;
