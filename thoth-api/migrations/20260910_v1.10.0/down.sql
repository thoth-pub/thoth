-- BE-06 Migration 1 (20260910_v1.10.0) down.sql: a documented no-op (R52B §23.2).
--
-- PostgreSQL provides no ALTER TYPE ... DROP VALUE, and the three labels this migration adds (WORK_UPSERT,
-- BINDING_SUPERSEDED, WORK_DELETED) are inert once Migration 2 is reverted: no row uses them and the released code never
-- reads them. Rolling back Migration 1 therefore leaves the labels in place (R52B §23.5).
SELECT 1;
