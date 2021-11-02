ALTER TABLE funder RENAME TO institution;

ALTER TABLE institution RENAME COLUMN funder_id TO institution_id;
ALTER TABLE institution RENAME COLUMN funder_name TO institution_name;
ALTER TABLE institution RENAME COLUMN funder_doi TO institution_doi;

ALTER TABLE funder_history RENAME TO institution_history;

ALTER TABLE institution_history RENAME COLUMN funder_history_id TO institution_history_id;
ALTER TABLE institution_history RENAME COLUMN funder_id TO institution_id;

ALTER TABLE funding RENAME COLUMN funder_id TO institution_id;
