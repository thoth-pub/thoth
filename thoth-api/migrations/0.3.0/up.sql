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

CREATE TABLE publisher_history (
    publisher_history_id     UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    publisher_id             UUID NOT NULL REFERENCES publisher(publisher_id) ON DELETE CASCADE,
    account_id               UUID NOT NULL REFERENCES account(account_id),
    data                     JSONB NOT NULL,
    timestamp                TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE imprint_history (
    imprint_history_id       UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    imprint_id               UUID NOT NULL REFERENCES imprint(imprint_id) ON DELETE CASCADE,
    account_id               UUID NOT NULL REFERENCES account(account_id),
    data                     JSONB NOT NULL,
    timestamp                TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE work_history (
    work_history_id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    work_id                  UUID NOT NULL REFERENCES work(work_id) ON DELETE CASCADE,
    account_id               UUID NOT NULL REFERENCES account(account_id),
    data                     JSONB NOT NULL,
    timestamp                TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE language_history (
    language_history_id      UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    language_id              UUID NOT NULL REFERENCES language(language_id) ON DELETE CASCADE,
    account_id               UUID NOT NULL REFERENCES account(account_id),
    data                     JSONB NOT NULL,
    timestamp                TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE series_history (
    series_history_id        UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    series_id                UUID NOT NULL REFERENCES series(series_id) ON DELETE CASCADE,
    account_id               UUID NOT NULL REFERENCES account(account_id),
    data                     JSONB NOT NULL,
    timestamp                TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE issue_history (
    issue_history_id         UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    series_id                UUID NOT NULL,
    work_id                  UUID NOT NULL,
    account_id               UUID NOT NULL REFERENCES account(account_id),
    data                     JSONB NOT NULL,
    timestamp                TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (series_id, work_id) REFERENCES issue(series_id, work_id) ON DELETE CASCADE
);

CREATE TABLE contributor_history (
    contributor_history_id   UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    contributor_id           UUID NOT NULL REFERENCES contributor(contributor_id) ON DELETE CASCADE,
    account_id               UUID NOT NULL REFERENCES account(account_id),
    data                     JSONB NOT NULL,
    timestamp                TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE contribution_history (
    contribution_history_id  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    work_id                  UUID NOT NULL,
    contributor_id           UUID NOT NULL,
    contribution_type        contribution_type NOT NULL,
    account_id               UUID NOT NULL REFERENCES account(account_id),
    data                     JSONB NOT NULL,
    timestamp                TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (work_id, contributor_id, contribution_type) REFERENCES contribution(work_id, contributor_id, contribution_type) ON DELETE CASCADE
);

CREATE TABLE publication_history (
    publication_history_id   UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    publication_id           UUID NOT NULL REFERENCES publication(publication_id) ON DELETE CASCADE,
    account_id               UUID NOT NULL REFERENCES account(account_id),
    data                     JSONB NOT NULL,
    timestamp                TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE price_history (
    price_history_id         UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    price_id                 UUID NOT NULL REFERENCES price(price_id) ON DELETE CASCADE,
    account_id               UUID NOT NULL REFERENCES account(account_id),
    data                     JSONB NOT NULL,
    timestamp                TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE subject_history (
    subject_history_id       UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    subject_id               UUID NOT NULL REFERENCES subject(subject_id) ON DELETE CASCADE,
    account_id               UUID NOT NULL REFERENCES account(account_id),
    data                     JSONB NOT NULL,
    timestamp                TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE funder_history (
    funder_history_id        UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    funder_id                UUID NOT NULL REFERENCES funder(funder_id) ON DELETE CASCADE,
    account_id               UUID NOT NULL REFERENCES account(account_id),
    data                     JSONB NOT NULL,
    timestamp                TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE funding_history (
    funding_history_id       UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    funding_id               UUID NOT NULL REFERENCES funding(funding_id) ON DELETE CASCADE,
    account_id               UUID NOT NULL REFERENCES account(account_id),
    data                     JSONB NOT NULL,
    timestamp                TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
