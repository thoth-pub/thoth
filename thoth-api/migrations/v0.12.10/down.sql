ALTER TYPE work_status RENAME VALUE 'withdrawn' TO 'withdrawn-from-sale';

ALTER TABLE work
    DROP CONSTRAINT IF EXISTS work_active_withdrawn_date_check,
    DROP CONSTRAINT IF EXISTS work_inactive_no_withdrawn_date_check,
    DROP CONSTRAINT IF EXISTS work_active_publication_date_check;
