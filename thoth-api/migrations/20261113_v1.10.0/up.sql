-- ROR-SYNC-01C (thoth-pub/thoth#1024, programme #1002): a ROR status and a
-- ROR successor for `institution`, and the P3 works-bump rule.
--
-- New:
--   * enum `ror_status` (`active`, `inactive`, `withdrawn`);
--   * columns `institution.ror_status` (NOT NULL, default `active`) and
--     `institution.ror_successor` (nullable);
--   * four named CHECKs: `institution_ror_successor_check`,
--     `institution_ror_state_check`, `institution_ror_successor_self_check`
--     and `institution_ror_active_successor_check`;
--   * a replaced body for `institution_work_updated_at_with_relations()`.
--
-- Existing data: every row becomes `active` with a null successor. The
-- constant default is a catalogue-only change (PostgreSQL 11 and later), so
-- the table is not rewritten and no UPDATE trigger fires. Validating the
-- CHECKs scans the table once; every existing row satisfies all four.
--
-- Locking: `ACCESS EXCLUSIVE` on `institution` from the ALTER until commit,
-- with `lock_timeout` 5 s on every lock wait. While the ALTER waits for its
-- lock, new statements on `institution` (reads, writes and the foreign-key
-- checks of new affiliations and fundings) queue behind it. A lock timeout
-- fails `init` with nothing applied, and the old tasks keep serving.
--
-- Retry and idempotency: one transaction. A concurrent second copy waits at
-- `CREATE TYPE ror_status` on the first copy's uncommitted catalogue row, then
-- fails (duplicate type or lock timeout) and rolls back; its replacement task
-- finds the migration applied.
--
-- Ordering: independent of ROR-SYNC-01B's migration.

SET LOCAL lock_timeout = '5s';

CREATE TYPE ror_status AS ENUM ('active', 'inactive', 'withdrawn');

ALTER TABLE institution
    ADD COLUMN ror_status ror_status NOT NULL DEFAULT 'active',
    ADD COLUMN ror_successor text NULL,
    ADD CONSTRAINT institution_ror_successor_check
        CHECK (ror_successor ~ '^https:\/\/ror\.org\/0[a-hjkmnp-z0-9]{6}\d{2}$'::text),
    ADD CONSTRAINT institution_ror_state_check
        CHECK (ror IS NOT NULL OR (ror_status = 'active' AND ror_successor IS NULL)),
    ADD CONSTRAINT institution_ror_successor_self_check
        CHECK (ror_successor IS NULL OR ror_successor <> ror),
    ADD CONSTRAINT institution_ror_active_successor_check
        CHECK (ror_status <> 'active' OR ror_successor IS NULL);

-- Decision P3 (CTO, 2026-11-13, "Exported fields, capped per run"): bump the
-- linked works only when a field that the exports carry changes.
CREATE OR REPLACE FUNCTION institution_work_updated_at_with_relations() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- The field list below must equal the institution fields selected by the
    -- export documents in `thoth-client/assets/queries.graphql`: `WorkQuery`,
    -- `WorksQuery` and the fragments they use (`Work`, `Funding`,
    -- `Reference`). A later export of `rorStatus` or `rorSuccessor` must add
    -- it here. A test pins this. A change of `ror_status` or `ror_successor`
    -- alone bumps nothing.
    --
    -- Only works linked by affiliation or funding are bumped. Works that
    -- export the institution through a related work are not bumped (as
    -- before this change), so counts based on this trigger undercount the
    -- exports that change.
    IF (
        ROW(NEW.institution_name, NEW.institution_doi, NEW.ror, NEW.country_code)
        IS DISTINCT FROM
        ROW(OLD.institution_name, OLD.institution_doi, OLD.ror, OLD.country_code)
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
