

-- Drop constraints, otherwise it won't be able to cast to text
ALTER TABLE work
    DROP CONSTRAINT work_active_withdrawn_date_check,
    DROP CONSTRAINT work_inactive_no_withdrawn_date_check;
-- TODO: drop new constraints in up migration

ALTER TABLE work ALTER COLUMN work_status TYPE text;

-- !!! if this down migration is run, 'out-of-print' should
-- be treated as a placeholder work_status. 
-- Works will need to be manually reassigned correct work_status.
UPDATE work 
    SET work_status = 'out-of-print' 
    WHERE work_status = 'superseded';

DROP TYPE work_status;
CREATE TYPE work_status AS ENUM (
    'unspecified',
    'cancelled',
    'forthcoming',
    'postponed-indefinitely',
    'active',
    'no-longer-our-product',
    'out-of-stock-indefinitely',
    'out-of-print',
    'inactive',
    'unknown',
    'remaindered',
    'withdrawn-from-sale',
    'recalled'
);
ALTER TABLE work ALTER COLUMN work_status TYPE work_status USING work_status::work_status;

-- add constraints back to work table
ALTER TABLE work
    ADD CONSTRAINT work_active_withdrawn_date_check CHECK
        ((work_status = 'withdrawn-from-sale' OR work_status = 'out-of-print')
        OR (work_status NOT IN ('withdrawn-from-sale', 'out-of-print') AND withdrawn_date IS NULL)),

    ADD CONSTRAINT work_inactive_no_withdrawn_date_check CHECK
        (((work_status = 'withdrawn-from-sale' OR work_status = 'out-of-print') AND withdrawn_date IS NOT NULL)
        OR (work_status NOT IN ('withdrawn-from-sale', 'out-of-print')));