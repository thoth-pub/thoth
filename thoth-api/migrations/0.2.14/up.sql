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
