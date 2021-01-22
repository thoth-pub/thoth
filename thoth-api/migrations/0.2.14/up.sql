CREATE TABLE publisher_account (
    account_id          UUID NOT NULL REFERENCES account(account_id) ON DELETE CASCADE,
    publisher_id        UUID NOT NULL REFERENCES publisher(publisher_id) ON DELETE CASCADE,
    is_admin            BOOLEAN NOT NULL DEFAULT False,
    PRIMARY KEY (account_id, publisher_id)
);

ALTER TABLE account RENAME COLUMN is_admin TO is_superuser;
