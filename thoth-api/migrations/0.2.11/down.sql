ALTER TABLE publisher
    DROP COLUMN created_at,
    DROP COLUMN updated_at;

ALTER TABLE imprint
    DROP COLUMN created_at,
    DROP COLUMN updated_at;

ALTER TABLE work
    DROP COLUMN created_at,
    DROP COLUMN updated_at;

ALTER TABLE language
    DROP COLUMN created_at,
    DROP COLUMN updated_at;

ALTER TABLE series
    DROP COLUMN created_at,
    DROP COLUMN updated_at;

ALTER TABLE issue
    DROP COLUMN created_at,
    DROP COLUMN updated_at;

ALTER TABLE contributor
    DROP COLUMN created_at,
    DROP COLUMN updated_at;

ALTER TABLE contribution
    DROP COLUMN created_at,
    DROP COLUMN updated_at;

ALTER TABLE publication
    DROP COLUMN created_at,
    DROP COLUMN updated_at;

ALTER TABLE price
    DROP COLUMN created_at,
    DROP COLUMN updated_at;

ALTER TABLE subject
    DROP COLUMN created_at,
    DROP COLUMN updated_at;

ALTER TABLE funder
    DROP COLUMN created_at,
    DROP COLUMN updated_at;

ALTER TABLE funding
    DROP COLUMN created_at,
    DROP COLUMN updated_at;
