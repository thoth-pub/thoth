CREATE TABLE publisher_account (
    account_id          UUID NOT NULL REFERENCES account(account_id) ON DELETE CASCADE,
    publisher_id        UUID NOT NULL REFERENCES publisher(publisher_id) ON DELETE CASCADE,
    is_admin            BOOLEAN NOT NULL DEFAULT False,
    created_at          TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at          TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (account_id, publisher_id)
);
SELECT diesel_manage_updated_at('publisher_account');

ALTER TABLE account RENAME COLUMN is_admin TO is_superuser;

ALTER TABLE contribution
    ADD COLUMN first_name TEXT,
    ADD COLUMN last_name TEXT,
    ADD COLUMN full_name TEXT;

UPDATE contribution
    SET first_name = contributor.first_name,
        last_name = contributor.last_name,
        full_name = contributor.full_name
    FROM contributor
    WHERE contribution.contributor_id = contributor.contributor_id;

ALTER TABLE contribution
    ALTER COLUMN last_name SET NOT NULL,
    ALTER COLUMN full_name SET NOT NULL,
    ADD CONSTRAINT contribution_first_name_check CHECK (octet_length(first_name) >= 1),
    ADD CONSTRAINT contribution_last_name_check CHECK (octet_length(last_name) >= 1),
    ADD CONSTRAINT contribution_full_name_check CHECK (octet_length(full_name) >= 1);