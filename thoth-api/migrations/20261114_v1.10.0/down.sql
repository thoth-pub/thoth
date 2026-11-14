-- ROR-SYNC-01B (thoth-pub/thoth#1023): reverse of up.sql's schema change only.
--
-- LOSSY: this drops the unique index but does not reverse the merge. The
-- merged rows stay deleted; each one can be restored, with its links, by the
-- forward repair script (thoth-api/ops/ror-sync-01b/repair.sql) from its merge
-- row in `institution_history`. The CTO approves this at MERGE AUTH.
--
-- `idx_institution_ror` was never dropped, so nothing is recreated.
--
-- Never run `thoth migrate --revert`, which reverts every migration. A
-- rollback runs this file and deletes this migration's row from
-- `__diesel_schema_migrations` in one transaction, under its own grant.

SET LOCAL lock_timeout = '10s';

DROP INDEX institution_ror_uniq_idx;
