ALTER TABLE series
    ALTER COLUMN issn_print SET NOT NULL;

ALTER TABLE series
    ALTER COLUMN issn_digital SET NOT NULL;

ALTER TABLE work
    DROP CONSTRAINT work_active_withdrawn_date_check,
    DROP CONSTRAINT work_inactive_no_withdrawn_date_check,
    DROP CONSTRAINT work_withdrawn_date_after_publication_date_check,
    DROP COLUMN withdrawn_date;

