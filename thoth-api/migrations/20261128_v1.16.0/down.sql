-- MET-QUARTER-01 migration 1 of 2: PostgreSQL has no `DROP VALUE` for an enum
-- type, so the `QUARTER` value stays. This file is a deliberate no-op; the
-- rest of the stage is reversed by 20261129_v1.15.0/down.sql.
SELECT 1;
