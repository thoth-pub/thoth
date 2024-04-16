ALTER TABLE series
    ALTER COLUMN issn_print DROP NOT NULL;

ALTER TABLE series
    ALTER COLUMN issn_digital DROP NOT NULL;

