-- ROR-SYNC-01C (thoth-pub/thoth#1024): reverse of up.sql, in reverse order.
--
-- LOSSY: every stored ROR status and successor is dropped. The forward repair
-- is the next ROR sync run, which restores them. The CTO approves this at
-- MERGE AUTH. Never run `thoth migrate --revert`, which reverts every
-- migration; a rollback runs this file and deletes this migration's row from
-- `__diesel_schema_migrations` in one transaction, under its own grant.

SET LOCAL lock_timeout = '5s';

CREATE OR REPLACE FUNCTION institution_work_updated_at_with_relations() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD
    ) THEN
        -- Same as contributor above (but can be connected to work via two different tables)
        -- Use two separate UPDATE statements as this is much faster than combining the WHERE clauses
        -- using OR (in tests, this caused several seconds' delay when saving institution updates)
        UPDATE work
        SET updated_at_with_relations = current_timestamp
        FROM funding
        WHERE work.work_id = funding.work_id AND funding.institution_id = NEW.institution_id;
        UPDATE work
        SET updated_at_with_relations = current_timestamp
        FROM affiliation, contribution
        WHERE work.work_id = contribution.work_id AND contribution.contribution_id = affiliation.contribution_id AND affiliation.institution_id = NEW.institution_id;
    END IF;
    RETURN NULL;
END;
$$;

ALTER TABLE institution
    DROP CONSTRAINT institution_ror_active_successor_check,
    DROP CONSTRAINT institution_ror_successor_self_check,
    DROP CONSTRAINT institution_ror_state_check,
    DROP CONSTRAINT institution_ror_successor_check,
    DROP COLUMN ror_successor,
    DROP COLUMN ror_status;

DROP TYPE ror_status;
