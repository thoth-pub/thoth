CREATE TYPE location_platform AS ENUM (
    'Project MUSE',
    'OAPEN',
    'DOAB',
    'JSTOR',
    'EBSCO Host',
    'OCLC KB',
    'ProQuest KB',
    'ProQuest ExLibris',
    'EBSCO KB',
    'JISC KB',
    'Other'
);

CREATE TABLE location (
    location_id         UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    publication_id      UUID NOT NULL REFERENCES publication(publication_id) ON DELETE CASCADE,
    landing_page        TEXT CHECK (landing_page ~* '^[^:]*:\/\/(?:[^\/:]*:[^\/@]*@)?(?:[^\/:.]*\.)+([^:\/]+)'),
    full_text_url       TEXT CHECK (full_text_url ~* '^[^:]*:\/\/(?:[^\/:]*:[^\/@]*@)?(?:[^\/:.]*\.)+([^:\/]+)'),
    location_platform   location_platform NOT NULL DEFAULT 'Other',
    canonical           BOOLEAN NOT NULL DEFAULT False,
    created_at          TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at          TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- Location must contain at least one of landing_page or full_text_url
    CONSTRAINT location_url_check CHECK (landing_page IS NOT NULL OR full_text_url IS NOT NULL)
);
SELECT diesel_manage_updated_at('location');

-- Only allow one canonical location per publication
CREATE UNIQUE INDEX location_uniq_canonical_true_idx ON location(publication_id)
    WHERE canonical;

-- Only allow one instance of each platform (except 'Other') per publication
CREATE UNIQUE INDEX location_uniq_platform_idx ON location(publication_id, location_platform)
    WHERE NOT location_platform = 'Other';

CREATE TABLE location_history (
    location_history_id      UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    location_id              UUID NOT NULL REFERENCES location(location_id) ON DELETE CASCADE,
    account_id               UUID NOT NULL REFERENCES account(account_id),
    data                     JSONB NOT NULL,
    timestamp                TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create location entries for every existing publication_url (assume all are landing pages)
-- If a publication has locations, exactly one of them must be canonical;
-- this command will create at most one location per publication, so make them all canonical.
INSERT INTO location(publication_id, landing_page, canonical)
    SELECT publication_id, publication_url, True FROM publication WHERE publication_url IS NOT NULL;

ALTER TABLE publication
    -- Only allow one publication of each type per work (existing data may breach this)
    -- To check for records which breach this constraint:
    -- `select * from publication a where (select count(*) from publication b where a.publication_type = b.publication_type and a.work_id = b.work_id) > 1 order by work_id, publication_type;`
    ADD CONSTRAINT publication_publication_type_work_id_uniq UNIQUE (publication_type, work_id),
    -- Remove publication_url column (all data should have been migrated to location table above)
    DROP COLUMN publication_url;
