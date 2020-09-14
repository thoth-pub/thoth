-------------------- Account
CREATE TABLE account (
    account_id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name                TEXT NOT NULL CHECK (octet_length(name) >= 1),
    surname             TEXT NOT NULL CHECK (octet_length(surname) >= 1),
    email               TEXT NOT NULL CHECK (octet_length(email) >= 1),
    password            TEXT NOT NULL CHECK (octet_length(password) >= 1),
    is_admin            BOOLEAN NOT NULL DEFAULT False,
    is_bot              BOOLEAN NOT NULL DEFAULT False,
    is_active           BOOLEAN NOT NULL DEFAULT True,
    registered          TIMESTAMP WITH TIME ZONE DEFAULT now() NOT NULL,
    token               TEXT NULL CHECK (OCTET_LENGTH(token) >= 1)
);

-- case-insensitive UNIQ index on email
CREATE UNIQUE INDEX email_uniq_idx ON account(lower(email));
