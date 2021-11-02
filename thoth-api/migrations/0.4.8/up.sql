ALTER TABLE funder RENAME TO institution;

ALTER TABLE institution RENAME COLUMN funder_id TO institution_id;
ALTER TABLE institution RENAME COLUMN funder_name TO institution_name;
ALTER TABLE institution RENAME COLUMN funder_doi TO institution_doi;

ALTER TABLE institution
    ADD COLUMN ror TEXT CHECK (ror ~ '^https:\/\/ror\.org\/0[a-hjkmnp-z0-9]{6}\d{2}$');

ALTER TABLE funder_history RENAME TO institution_history;

ALTER TABLE institution_history RENAME COLUMN funder_history_id TO institution_history_id;
ALTER TABLE institution_history RENAME COLUMN funder_id TO institution_id;

ALTER TABLE funding RENAME COLUMN funder_id TO institution_id;
