-- MET-WP1-11 downgrade: remove only the schema introduced by this migration.
--
-- The child `metric_reconciliation_issue` is dropped BEFORE its parent
-- `metric_reconciliation_run`, so the run table is free of dependants by the
-- time it is dropped and neither statement needs CASCADE. CASCADE is
-- deliberately NOT used: nothing may be removed beyond these two tables, and
-- in particular the referenced MET-WP1-04 `metric_record (record_id)` primary
-- key must not be reached. This slice created no enum type, trigger, stored
-- procedure, sequence or standalone index, so the two non-cascading DROP TABLE
-- statements are exact; both primary-key indexes are dropped implicitly with
-- their tables.
--
-- The MET-WP1-01 registry schema and its measure seed rows, the MET-WP1-02
-- source-state schema, the MET-WP1-03 import-state schema, the MET-WP1-04
-- canonical record/revision/provenance schema and its enums, the MET-WP1-05
-- coverage schema, the MET-WP1-06 publisher-platform approval schema, the
-- MET-WP1-07 rollup-delta schema, the MET-WP1-08 OPERAS mapping schema, the
-- MET-WP1-09 OPERAS export ledger and the MET-WP1-10 OPERAS import ledger are
-- all untouched, as is the existing bibliographic schema. In particular the
-- referenced MET-WP1-04 `metric_record (record_id)` primary key belongs to
-- migration `20260831_v1.9.0` and must survive.

DROP TABLE IF EXISTS public.metric_reconciliation_issue;
DROP TABLE IF EXISTS public.metric_reconciliation_run;
