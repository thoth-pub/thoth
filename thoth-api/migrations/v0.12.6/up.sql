--  Assign 1900-01-01 as placeholder publication_date for 
-- Active works with no publication date
-- works in local db with this status as of 17-05-2024: 51 works (incl. chapters)
-- Publisher should be notified and add correct publication_date 
-- !!! This step of migration is irreversible.
UPDATE work
    SET
        publication_date = '1900-01-01'
    WHERE
        work_status = 'active' AND publication_date IS NULL;

-- Drop constraints, otherwise it won't be able to cast to text
ALTER TABLE work
    DROP CONSTRAINT IF EXISTS work_active_withdrawn_date_check,
    DROP CONSTRAINT IF EXISTS work_inactive_no_withdrawn_date_check;

ALTER TABLE work ALTER COLUMN work_status TYPE text;

-- delete unused work_status enum 
DROP TYPE work_status;

-- assign works new work_status if theirs is going to be deleted
-- current counts for out of print/inactive/out of stock indefinitely in local db as of 17-05-2024:
-- 148 works (incl. chapters)
-- superseded seems to be the best placeholder for these works,
-- since it's a new status and no works are currently using it.
-- works will need to be manually reassigned correct work_status from the new 
-- options, and withdrawn_date removed as necessary. 
-- !!! This step in the migration is irreversible.
UPDATE work 
    SET 
        work_status = 'superseded', 
        -- assign a withdrawn_date, which will be required for superseded works
        withdrawn_date = CASE 
            WHEN withdrawn_date IS NOT NULL THEN withdrawn_date
            -- + INTERVAL '1 day' is necessary because one work has publication_date on
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

-- these work_status currently have no works with this status in production db;
-- nonetheless, reassign in case works get assigned these work_status
-- before migration is run in production
-- current counts in local db as of 27-05-2024:
-- unspecified, 0 works
-- unknown, 0 works
-- !!! This step of the migration is irreversible, because of merging two
-- different work_status into a single one, and there are lots of 
-- real works with forthcoming work_status that we don't want to 
-- incorrectly change back to 'unspecified' or 'unknown'. However, this doesn't 
-- seem like a big deal, since there are no works with these work_status
-- currently. 
UPDATE work 
    SET work_status = 'forthcoming' 
    WHERE work_status = 'unspecified' OR work_status = 'unknown';

-- current counts in local db as of 27-05-2024:
-- no-longer-our-product, 0 works
-- remaindered, 0 works
-- recalled, 0 works
-- !!! see above: this step of the migration is irreversible.
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
    -- but changes that constraint by adding superseded alongside withdrawn-from-sale
    ADD CONSTRAINT work_inactive_no_withdrawn_date_check CHECK
        (((work_status = 'withdrawn-from-sale' OR work_status = 'superseded') AND withdrawn_date IS NOT NULL)
        OR (work_status NOT IN ('withdrawn-from-sale', 'superseded'))),
    -- all other work statuses must not have withdrawn_date; see above, adds superseded
    ADD CONSTRAINT work_active_withdrawn_date_check CHECK
        ((work_status = 'withdrawn-from-sale' OR work_status = 'superseded')
        OR (work_status NOT IN ('withdrawn-from-sale', 'superseded') AND withdrawn_date IS NULL)),
    -- active works must have publication_date
    ADD CONSTRAINT work_active_publication_date_check CHECK
        ((work_status = 'active' AND publication_date IS NOT NULL)
        OR (work_status != 'active'));
        