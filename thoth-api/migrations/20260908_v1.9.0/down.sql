-- MET-WP1-12 downgrade (issue #894).
--
-- Removes exactly the objects the MET-WP1-12 up migration created: the one
-- append-only registry audit table and the two audit enums. It touches no
-- MET-WP1-01..11 Metrics table, enum, constraint, index, trigger or row, and
-- no bibliographic object.
--
-- The audit table is dropped before its enums because both enum types are
-- still referenced by its column definitions.
--
-- This is a genuinely lossy downgrade of audit evidence: reverting discards
-- every recorded registry mutation. That is accepted because the table is
-- additive and empty at deployment, no MET-WP1-12 execution path can have
-- written a row before the administration surface is used, and the canonical
-- registry rows themselves are untouched by this revert.

DROP TABLE public.metric_registry_history;

DROP TYPE public.metric_registry_history_action;

DROP TYPE public.metric_registry_history_entity;
