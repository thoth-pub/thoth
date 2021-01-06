DROP TRIGGER set_updated_at ON publisher;
ALTER TABLE publisher
    DROP COLUMN created_at,
    DROP COLUMN updated_at;

DROP TRIGGER set_updated_at ON imprint;
ALTER TABLE imprint
    DROP COLUMN created_at,
    DROP COLUMN updated_at;

DROP TRIGGER set_updated_at ON work;
ALTER TABLE work
    DROP COLUMN created_at,
    DROP COLUMN updated_at;

DROP TRIGGER set_updated_at ON language;
ALTER TABLE language
    DROP COLUMN created_at,
    DROP COLUMN updated_at;

DROP TRIGGER set_updated_at ON series;
ALTER TABLE series
    DROP COLUMN created_at,
    DROP COLUMN updated_at;

DROP TRIGGER set_updated_at ON issue;
ALTER TABLE issue
    DROP COLUMN created_at,
    DROP COLUMN updated_at;

DROP TRIGGER set_updated_at ON contributor;
ALTER TABLE contributor
    DROP COLUMN created_at,
    DROP COLUMN updated_at;

DROP TRIGGER set_updated_at ON contribution;
ALTER TABLE contribution
    DROP COLUMN created_at,
    DROP COLUMN updated_at;

DROP TRIGGER set_updated_at ON publication;
ALTER TABLE publication
    DROP COLUMN created_at,
    DROP COLUMN updated_at;

DROP TRIGGER set_updated_at ON price;
ALTER TABLE price
    DROP COLUMN created_at,
    DROP COLUMN updated_at;

DROP TRIGGER set_updated_at ON subject;
ALTER TABLE subject
    DROP COLUMN created_at,
    DROP COLUMN updated_at;

DROP TRIGGER set_updated_at ON funder;
ALTER TABLE funder
    DROP COLUMN created_at,
    DROP COLUMN updated_at;

DROP TRIGGER set_updated_at ON funding;
ALTER TABLE funding
    DROP COLUMN created_at,
    DROP COLUMN updated_at;

DROP TRIGGER set_updated_at ON account;
ALTER TABLE account
    RENAME COLUMN created_at TO registered;
ALTER TABLE account
    ALTER COLUMN registered TYPE TIMESTAMP WITH TIME ZONE,
    ALTER COLUMN registered SET NOT NULL,
    ALTER COLUMN registered SET DEFAULT now(),
    DROP COLUMN updated_at;
