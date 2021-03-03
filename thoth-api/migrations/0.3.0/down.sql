DROP TRIGGER set_updated_at ON publisher_account;
DROP TABLE publisher_account;

ALTER TABLE account RENAME COLUMN is_superuser TO is_admin;

ALTER TABLE contribution
    DROP COLUMN first_name,
    DROP COLUMN last_name,
    DROP COLUMN full_name;

DROP TABLE publisher_history;
DROP TABLE imprint_history;
DROP TABLE work_history;
DROP TABLE language_history;
DROP TABLE series_history;
DROP TABLE issue_history;
DROP TABLE contributor_history;
DROP TABLE contribution_history;
DROP TABLE publication_history;
DROP TABLE price_history;
DROP TABLE subject_history;
DROP TABLE funder_history;
DROP TABLE funding_history;
