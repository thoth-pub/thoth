-- MET-WP1-13 downgrade (issue #904).
--
-- Removes exactly the objects the MET-WP1-13 up migration created: the one
-- append-only source-administration audit table, its two enums and the
-- metric_source driver-key CHECK. It touches no other MET-WP1-01..12 or
-- MET-WP2-01A table, enum, column, constraint, index, trigger or row, and no
-- bibliographic object. metric_source and metric_source_account rows are
-- left exactly as they are.
--
-- The audit table is dropped before its enums because both enum types are
-- still referenced by its column definitions.
--
-- Reverting the audit table is a genuinely lossy downgrade of audit evidence,
-- accepted because the table is additive and empty at deployment and the
-- canonical source/source-account rows it describes are untouched by this
-- revert.

DROP TABLE public.metric_source_registry_history;

DROP TYPE public.metric_source_registry_history_action;

DROP TYPE public.metric_source_registry_history_entity;

ALTER TABLE public.metric_source
    DROP CONSTRAINT metric_source_driver_key_check;
