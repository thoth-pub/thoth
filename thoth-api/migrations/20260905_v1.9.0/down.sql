-- MET-WP1-09 downgrade: remove only the schema introduced by this migration.
--
-- metric_operas_export holds no dependants — nothing references it, and this
-- slice created no enum type, trigger, sequence or standalone index — so a
-- single non-cascading DROP TABLE is exact. Its primary-key index and its
-- `record_revision_id` uniqueness index are dropped implicitly with the table.
--
-- The MET-WP1-01 registry schema and its measure seed rows, the MET-WP1-02
-- source-state schema, the MET-WP1-03 import-state schema, the MET-WP1-04
-- canonical record/revision/provenance schema and its enums, the MET-WP1-05
-- coverage schema, the MET-WP1-06 publisher-platform approval schema, the
-- MET-WP1-07 rollup-delta schema and the MET-WP1-08 OPERAS mapping schema are
-- all untouched, as is the existing bibliographic schema. In particular the
-- referenced MET-WP1-04 `metric_record_revision (record_revision_id)` primary
-- key and MET-WP1-08 `metric_operas_mapping (mapping_id)` primary key belong
-- to those migrations and must survive.

DROP TABLE IF EXISTS public.metric_operas_export;
