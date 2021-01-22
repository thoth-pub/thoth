DROP TABLE publisher_account;

ALTER TABLE account RENAME COLUMN is_superuser TO is_admin;
