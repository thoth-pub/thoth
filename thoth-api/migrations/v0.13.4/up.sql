CREATE TYPE contact_type AS ENUM (
    'Accessibility'
);

CREATE TABLE contact (
    contact_id      UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    publisher_id    UUID NOT NULL REFERENCES publisher(publisher_id) ON DELETE CASCADE,
    contact_type    contact_type NOT NULL DEFAULT 'Accessibility',
    email           TEXT NOT NULL CHECK (octet_length(email) >= 1),
    created_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT contact_contact_type_publisher_id_uniq UNIQUE (publisher_id, contact_type)
);
SELECT diesel_manage_updated_at('contact');
CREATE INDEX idx_contact_email ON contact (email);

CREATE TABLE contact_history (
    contact_history_id  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    contact_id          UUID NOT NULL REFERENCES contact(contact_id) ON DELETE CASCADE,
    account_id          UUID NOT NULL REFERENCES account(account_id),
    data                JSONB NOT NULL,
    timestamp           TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

ALTER TABLE publisher
    ADD COLUMN accessibility TEXT CHECK (octet_length(accessibility) >= 1);
