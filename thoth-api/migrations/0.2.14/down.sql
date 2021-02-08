DROP TRIGGER set_updated_at ON publisher_account;
DROP TABLE publisher_account;

ALTER TABLE account RENAME COLUMN is_superuser TO is_admin;

ALTER TABLE contribution
    DROP COLUMN first_name,
    DROP COLUMN last_name,
    DROP COLUMN full_name;