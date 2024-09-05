ALTER TYPE work_status RENAME VALUE 'withdrawn-from-sale' TO 'withdrawn';

-- Drop constraints that reference old 'withdrawn-from-sale' work_status
ALTER TABLE work
    DROP CONSTRAINT IF EXISTS work_active_withdrawn_date_check,
    DROP CONSTRAINT IF EXISTS work_inactive_no_withdrawn_date_check,
    DROP CONSTRAINT IF EXISTS work_active_publication_date_check;

-- add new constraints (with same names as in v0.12.9) to work table
ALTER TABLE work
    -- withdrawn and superseded works must have withdrawn_date
    -- note that this constraint has the same name as migration from v.0.12.3,
    -- but changes previous constraint by adding superseded alongside withdrawn
    ADD CONSTRAINT work_inactive_no_withdrawn_date_check CHECK
        (((work_status = 'withdrawn' OR work_status = 'superseded') AND withdrawn_date IS NOT NULL)
        OR (work_status NOT IN ('withdrawn', 'superseded'))),
    -- all other work statuses must not have withdrawn_date; see above, adds superseded
    ADD CONSTRAINT work_active_withdrawn_date_check CHECK
        ((work_status = 'withdrawn' OR work_status = 'superseded')
        OR (work_status NOT IN ('withdrawn', 'superseded') AND withdrawn_date IS NULL)),
    -- active, withdrawn-from-sale, and superseded works must have publication_date
    ADD CONSTRAINT work_active_publication_date_check CHECK
        ((work_status IN ('active', 'withdrawn', 'superseded') AND publication_date IS NOT NULL)
        OR (work_status NOT IN ('active', 'withdrawn', 'superseded')));
