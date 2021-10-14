CREATE TYPE location_platform AS ENUM (
    'Project MUSE',
    'OAPEN',
    'JSTOR',
    'Other'
);

CREATE TABLE location (
    location_id         UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    publication_id      UUID NOT NULL REFERENCES publication(publication_id) ON DELETE CASCADE,
    landing_page        TEXT NOT NULL CHECK (landing_page ~* '^[^:]*:\/\/(?:[^\/:]*:[^\/@]*@)?(?:[^\/:.]*\.)+([^:\/]+)'),
    full_text_url       TEXT CHECK (full_text_url ~* '^[^:]*:\/\/(?:[^\/:]*:[^\/@]*@)?(?:[^\/:.]*\.)+([^:\/]+)'),
    location_platform   location_platform NOT NULL DEFAULT 'Other',
    canonical           BOOLEAN NOT NULL DEFAULT True,
    created_at          TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at          TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
SELECT diesel_manage_updated_at('location');

CREATE TABLE location_history (
    location_history_id      UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    location_id              UUID NOT NULL REFERENCES location(location_id) ON DELETE CASCADE,
    account_id               UUID NOT NULL REFERENCES account(account_id),
    data                     JSONB NOT NULL,
    timestamp                TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO location(publication_id, landing_page)
    SELECT publication_id, publication_url FROM publication WHERE publication_url IS NOT NULL;
