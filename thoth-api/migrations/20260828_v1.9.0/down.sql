-- MET-WP1-03 downgrade: remove only the schema introduced by this migration,
-- dropping the two import-state tables in dependency-safe order
-- (import error -> import) and then both enum types. The MET-WP1-01 registry
-- schema, its seed rows and the MET-WP1-02 source-state schema are untouched.
-- The two partial unique idempotency indexes and the operational
-- (status, created_at) index are dropped implicitly with their table.

DROP TABLE IF EXISTS public.metric_import_error;

DROP TABLE IF EXISTS public.metric_import;

DROP TYPE IF EXISTS public.metric_import_error_severity;

DROP TYPE IF EXISTS public.metric_import_status;
