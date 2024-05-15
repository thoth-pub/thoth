

-- Drop constraints, otherwise it won't be able to cast to text
ALTER TABLE work
    DROP CONSTRAINT work_active_withdrawn_date_check,
    DROP CONSTRAINT work_inactive_no_withdrawn_date_check;

ALTER TABLE work ALTER COLUMN work_status TYPE text;

-- assign works new work_status if theirs is going to be deleted
-- out of print, 17 works--> superseded
UPDATE work 
    SET work_status = 'superseded' 
    WHERE work_status = 'out_of_print';
-- inactive, 9 works --> cancelled
UPDATE work 
    SET work_status = 'cancelled' 
    WHERE work_status = 'inactive';

-- these work_status currently have no works with this status in production db;
-- nonetheless, reassign in case works get assigned these work_status
-- before migration is run in production

-- 'unspecified', 'unknown', 0 works —> Forthcoming
-- !!! This step of the migration is irreversible, because of merging two
-- different work_status into a single one, and there are lots of 
-- real works with forthcoming work_status that we don't want to 
-- incorrectly change to 'unspecified' or 'unknown'. However, this doesn't 
-- seem like a big deal, since there are no works with these work_status
-- currently.
UPDATE work 
    SET work_status = 'forthcoming' 
    WHERE work_status = 'unspecified' OR work_status = 'unknown';

-- 'no-longer-our-product’,'out-of-stock-indefinitely’, 'remaindered', 'recalled', 0 works —> withdrawn
-- !!! see above: this step of the migration is irreversible.
UPDATE work 
    SET work_status = 'withdrawn-from-sale' 
    WHERE work_status = 'no-longer-our-product' OR work_status = 'out-of-stock-indefinitely' OR work_status = 'remaindered' OR work_status = 'recalled';

-- delete unused work_status enum values by creating new enum and add superseded
DROP TYPE work_status;
CREATE TYPE work_status AS ENUM (
    'cancelled',
    'forthcoming',
    'postponed-indefinitely',
    'active',
    'withdrawn-from-sale',
    'superseded',
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


