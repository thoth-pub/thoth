ALTER TABLE series
    ALTER COLUMN issn_print SET NOT NULL;

ALTER TABLE series
    ALTER COLUMN issn_digital SET NOT NULL;

ALTER TABLE work
    DROP CONSTRAINT work_withdrawn_date_check,
    DROP COLUMN withdrawn_date;
