UPDATE location SET location_platform = 'Other' WHERE location_platform = 'Thoth';

-- Drop the default and unique constraint, otherwise it won't be able to cast to text
ALTER TABLE location ALTER COLUMN location_platform DROP DEFAULT;
DROP INDEX location_uniq_platform_idx;

ALTER TABLE location ALTER COLUMN location_platform TYPE text;
DROP TYPE location_platform;
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
    'Google Books',
    'Internet Archive',
    'ScienceOpen',
    'SciELO Books',
    'Publisher Website',
    'Zenodo',
    'Other'
    );
ALTER TABLE location ALTER location_platform TYPE location_platform USING location_platform::location_platform;
ALTER TABLE location
    ALTER COLUMN location_platform SET DEFAULT 'Other'::location_platform;

CREATE UNIQUE INDEX location_uniq_platform_idx
    ON location (publication_id, location_platform)
    WHERE NOT location_platform = 'Other'::location_platform;
