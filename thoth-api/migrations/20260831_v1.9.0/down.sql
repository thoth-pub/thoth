-- MET-WP1-04 downgrade: remove only the schema introduced by this migration.
--
-- Dependency-safe order matters here because the approved design is
-- intentionally circular. metric_record points at its current revision while
-- metric_record_revision points back at its record, so neither table can be
-- dropped while metric_record_current_revision_id_fkey exists: PostgreSQL
-- refuses to drop metric_record_revision because that constraint on
-- metric_record depends on it. The circular constraint is therefore dropped
-- explicitly first, without DROP ... CASCADE, so the removal stays exact.
--
-- The MET-WP1-01 registry schema and its measure seed rows, the MET-WP1-02
-- source-state schema and the MET-WP1-03 import-state schema are untouched,
-- as are the existing bibliographic work, publication and institution tables.
-- The metric_record and metric_record_provenance indexes, the partial unique
-- current-revision index and every unique key are dropped implicitly with
-- their tables.

-- 1. Provenance holds no dependants, so it goes first.
DROP TABLE IF EXISTS public.metric_record_provenance;

-- 2. Break the circular relationship before dropping either participant.
ALTER TABLE IF EXISTS public.metric_record
    DROP CONSTRAINT IF EXISTS metric_record_current_revision_id_fkey;

-- 3. Revisions reference records, so revisions go before records.
DROP TABLE IF EXISTS public.metric_record_revision;

DROP TABLE IF EXISTS public.metric_record;

-- 4. Exactly this task's two enums. metric_reporting_grain belongs to
--    MET-WP1-01 and must survive.
DROP TYPE IF EXISTS public.metric_record_provenance_classification;

DROP TYPE IF EXISTS public.metric_record_revision_status;
