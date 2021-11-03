ALTER TABLE institution_history RENAME COLUMN institution_history_id TO funder_history_id;
ALTER TABLE institution_history RENAME COLUMN institution_id TO funder_id;

ALTER TABLE institution_history RENAME TO funder_history;

ALTER TABLE institution RENAME COLUMN institution_id TO funder_id;
ALTER TABLE institution RENAME COLUMN institution_name TO funder_name;
ALTER TABLE institution RENAME COLUMN institution_doi TO funder_doi;

ALTER TABLE institution
    DROP COLUMN ror;

ALTER TABLE institution RENAME TO funder;

ALTER TABLE funding RENAME COLUMN institution_id TO funder_id;
DROP TABLE affiliation_history;
DROP TABLE affiliation;
