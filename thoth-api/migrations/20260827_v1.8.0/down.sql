-- MET-WP1-02 downgrade: remove only the schema introduced by this migration,
-- dropping the source-state tables in dependency-safe order
-- (checkpoint -> source account -> source) and then the acquisition enum
-- type. The MET-WP1-01 registry schema and its seed rows are untouched.
-- The lease-expiry index is dropped implicitly with its table.

DROP TABLE IF EXISTS public.metric_source_checkpoint;

DROP TABLE IF EXISTS public.metric_source_account;

DROP TABLE IF EXISTS public.metric_source;

DROP TYPE IF EXISTS public.metric_source_acquisition_type;
