ALTER TABLE funder RENAME TO institution;

ALTER TABLE institution RENAME COLUMN funder_id TO institution_id;
ALTER TABLE institution RENAME COLUMN funder_name TO institution_name;
ALTER TABLE institution RENAME COLUMN funder_doi TO institution_doi;

ALTER TABLE institution
    ADD COLUMN ror TEXT CHECK (ror ~ '^https:\/\/ror\.org\/0[a-hjkmnp-z0-9]{6}\d{2}$');

ALTER TABLE funder_history RENAME TO institution_history;

ALTER TABLE institution_history RENAME COLUMN funder_history_id TO institution_history_id;
ALTER TABLE institution_history RENAME COLUMN funder_id TO institution_id;

ALTER TABLE funding RENAME COLUMN funder_id TO institution_id;

CREATE TABLE affiliation (
    affiliation_id      UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    contribution_id     UUID NOT NULL REFERENCES contribution(contribution_id) ON DELETE CASCADE,
    institution_id      UUID NOT NULL REFERENCES institution(institution_id) ON DELETE CASCADE,
    affiliation_ordinal INTEGER NOT NULL CHECK (affiliation_ordinal > 0),
    position            TEXT CHECK (octet_length(position) >= 1),
    created_at          TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at          TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
SELECT diesel_manage_updated_at('affiliation');

CREATE TABLE affiliation_history (
    affiliation_history_id   UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    affiliation_id           UUID NOT NULL REFERENCES affiliation(affiliation_id) ON DELETE CASCADE,
    account_id               UUID NOT NULL REFERENCES account(account_id),
    data                     JSONB NOT NULL,
    timestamp                TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
