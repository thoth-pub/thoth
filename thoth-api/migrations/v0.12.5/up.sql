-- Assign updated_at as placeholder publication_date for Active works with no publication date
-- works in local db with this status as of 17-05-2024: 1 work
-- Publisher should be notified and add correct publication_date 
-- !!! This step of migration is irreversible.
UPDATE work
    SET
        publication_date = COALESCE(publication_date, updated_at)
    WHERE
        work_status = 'active';

-- Drop constraints, otherwise it won't be able to cast to text
ALTER TABLE work
    DROP CONSTRAINT work_active_withdrawn_date_check,
    DROP CONSTRAINT work_inactive_no_withdrawn_date_check;

ALTER TABLE work ALTER COLUMN work_status TYPE text;

-- assign works new work_status if theirs is going to be deleted
-- current counts in production db as of 17-05-2024:
-- out of print, 17 works
-- inactive, 9 works
-- out of stock indefinitely, 0 works
-- !!! superseded is a placeholder for these works. 
-- works will need to be manually reassigned correct work_status from the new 
-- options, and withdrawn_date removed as necessary. This step in the migration is irreversible.
UPDATE work 
    SET 
        work_status = 'superseded', 
        withdrawn_date = COALESCE(withdrawn_date, updated_at)
    WHERE 
        work_status = 'out-of-print' 
        OR work_status = 'out-of-stock-indefinitely' 
        OR work_status = 'inactive';



-- these work_status currently have no works with this status in production db;
-- nonetheless, reassign in case works get assigned these work_status
-- before migration is run in production
-- current counts in production db as of 17-05-2024:
-- unspecified, 0 works
-- unknown, 0 works
-- !!! This step of the migration is irreversible, because of merging two
-- different work_status into a single one, and there are lots of 
-- real works with forthcoming work_status that we don't want to 
-- incorrectly change to 'unspecified' or 'unknown'. However, this doesn't 
-- seem like a big deal, since there are no works with these work_status
-- currently. 
UPDATE work 
    SET work_status = 'forthcoming' 
    WHERE work_status = 'unspecified' OR work_status = 'unknown';

-- current counts in production db as of 17-05-2024:
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

-- delete unused work_status enum 
DROP TYPE work_status;
-- add new values by creating new enum and add superseded
CREATE TYPE work_status AS ENUM (
    'cancelled',
    'forthcoming',
    'postponed-indefinitely',
    'active',
    'withdrawn-from-sale',
    'superseded',
);
ALTER TABLE work ALTER COLUMN work_status TYPE work_status USING work_status::work_status;

-- add new constraints (with same names as in v0.12.3) to work table
ALTER TABLE work
    -- withdrawn and superseded works must have withdrawn_date
    ADD CONSTRAINT work_inactive_no_withdrawn_date_check CHECK
        (((work_status = 'withdrawn-from-sale' OR work_status = 'superseded') AND withdrawn_date IS NOT NULL)
        OR (work_status NOT IN ('withdrawn-from-sale', 'superseded')));
    -- all other work statuses must not have withdrawn_date
    ADD CONSTRAINT work_active_withdrawn_date_check CHECK
        ((work_status = 'withdrawn-from-sale' OR work_status = 'superseded')
        OR (work_status NOT IN ('withdrawn-from-sale', 'superseded') AND withdrawn_date IS NULL)),
    -- active works must have publication_date
    ADD CONSTRAINT work_active_publication_date_check CHECK
        ((work_status = 'active' AND publication_date IS NOT NULL)
        OR (work_status != 'active'))