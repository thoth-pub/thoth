-- Assign 1900-01-01 as placeholder publication_date for 
-- Active, withdrawn from sale, out of print, out of stock indefinitely works with no publication date
-- Required for work_active_publication_date_check constraint below
-- Affected works in production db with this status, 29-05-2024: 59 works (incl. chapters)
-- Before running migration, make a list of affected works
-- After running migration, publishers should be notified to add correct publication_date 
-- !!! This is irreversible
UPDATE work
    SET
        publication_date = '1900-01-01'
    WHERE
        work_status IN 
        ('active', 'withdrawn-from-sale', 'out-of-print', 'out-of-stock-indefinitely', 'inactive') 
        AND publication_date IS NULL;

-- Drop constraints, otherwise it won't be able to cast to text
ALTER TABLE work
    DROP CONSTRAINT IF EXISTS work_active_withdrawn_date_check,
    DROP CONSTRAINT IF EXISTS work_inactive_no_withdrawn_date_check;

ALTER TABLE work ALTER COLUMN work_status TYPE text;

-- delete unused work_status enum 
DROP TYPE work_status;

-- Assign out of print/inactive/out of stock indefinitely works work_status 'superseded'
-- current counts in production db as of 29-05-2024:
-- 145 works (incl. chapters)
-- Before running migration, make a list of affected works
-- After running migration, publishers should be notified to add correct work_status
-- and remove withdrawn_date as necessary. Many OBP "out of print" works are actually first editions
-- for which superseded is the correct new work_status.
-- !!! This is irreversible
UPDATE work 
    SET 
        work_status = 'superseded', 
        -- assign a withdrawn_date, which is required for superseded works
        withdrawn_date = CASE 
            WHEN withdrawn_date IS NOT NULL THEN withdrawn_date
            -- + INTERVAL '1 day' is necessary because at least one work has publication_date on
            -- the same day as updated_at, but updated_at has a timestamp, so it's
            -- greater than. Which then throws an error with the
            -- work_withdrawn_date_after_publication_date_check constraint.
            WHEN withdrawn_date IS NULL AND publication_date + INTERVAL '1 day' < updated_at THEN updated_at
            ELSE CURRENT_DATE
        END
    WHERE 
        work_status = 'out-of-print' 
        OR work_status = 'out-of-stock-indefinitely' 
        OR work_status = 'inactive';

-- Assign unspecified/unkown works work_status 'forthcoming'
-- current counts in production db as of 29-05-2024:
-- unspecified, 0 works
-- unknown, 0 works
-- !!! This is irreversible
UPDATE work 
    SET work_status = 'forthcoming' 
    WHERE work_status = 'unspecified' OR work_status = 'unknown';

-- Assign no longer our product/remaindered/recalled works work_status 'withdrawn-from-sale'
-- current counts in production db as of 29-05-2024:
-- no-longer-our-product, 0 works
-- remaindered, 0 works
-- recalled, 0 works
-- !!! This is irreversible
UPDATE work 
    SET 
        work_status = 'withdrawn-from-sale',
        withdrawn_date = COALESCE(withdrawn_date, updated_at)
    WHERE 
        work_status = 'no-longer-our-product'
        OR work_status = 'remaindered'
        OR work_status = 'recalled';

-- create new work_status enum, adds superseded
CREATE TYPE work_status AS ENUM (
    'cancelled',
    'forthcoming',
    'postponed-indefinitely',
    'active',
    'withdrawn-from-sale',
    'superseded'
);
ALTER TABLE work ALTER COLUMN work_status TYPE work_status USING work_status::work_status;

-- add new constraints (with same names as in v0.12.3) to work table
ALTER TABLE work
    -- withdrawn and superseded works must have withdrawn_date
    -- note that this constraint has the same name as migration from v.0.12.3,
    -- but changes previous constraint by adding superseded alongside withdrawn-from-sale
    ADD CONSTRAINT work_inactive_no_withdrawn_date_check CHECK
        (((work_status = 'withdrawn-from-sale' OR work_status = 'superseded') AND withdrawn_date IS NOT NULL)
        OR (work_status NOT IN ('withdrawn-from-sale', 'superseded'))),
    -- all other work statuses must not have withdrawn_date; see above, adds superseded
    ADD CONSTRAINT work_active_withdrawn_date_check CHECK
        ((work_status = 'withdrawn-from-sale' OR work_status = 'superseded')
        OR (work_status NOT IN ('withdrawn-from-sale', 'superseded') AND withdrawn_date IS NULL)),
    -- active, withdrawn-from-sale, and superseded works must have publication_date
    ADD CONSTRAINT work_active_publication_date_check CHECK
        ((work_status IN ('active', 'withdrawn-from-sale', 'superseded') AND publication_date IS NOT NULL)
        OR (work_status NOT IN ('active', 'withdrawn-from-sale', 'superseded')));
