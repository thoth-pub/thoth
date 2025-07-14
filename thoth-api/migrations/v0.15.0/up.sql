-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create AbstractType enum
CREATE TYPE abstract_type AS ENUM (
    'short',
    'long'
);

-- Create the abstract table
CREATE TABLE IF NOT EXISTS abstract (
    abstract_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    work_id UUID NOT NULL REFERENCES work (work_id) ON DELETE CASCADE,
    content TEXT NOT NULL CHECK (octet_length(content) >= 1),
    locale_code locale_code NOT NULL,
    abstract_type abstract_type NOT NULL DEFAULT 'short',
    canonical BOOLEAN NOT NULL DEFAULT FALSE
);

-- Insert short abstracts into the abstract table
INSERT INTO abstract (abstract_id, work_id, content, locale_code, abstract_type, canonical)
SELECT 
    uuid_generate_v4() AS abstract_id,
    work_id,
    short_abstract AS content,
    'en'::locale_code, -- Assuming 'en' as the default locale code
    'short'::abstract_type,
    TRUE
FROM 
    work
WHERE 
    short_abstract IS NOT NULL;

-- Insert long abstracts into the abstract table
INSERT INTO abstract (abstract_id, work_id, content, locale_code, abstract_type, canonical)
SELECT 
    uuid_generate_v4() AS abstract_id,
    work_id,
    long_abstract AS content,
    'en'::locale_code, -- Assuming 'en' as the default locale code
    'long'::abstract_type,
    TRUE
FROM 
    work
WHERE 
    long_abstract IS NOT NULL;

-- Only allow one canonical abstract per work                                         
CREATE UNIQUE INDEX IF NOT EXISTS abstract_unique_canonical_true_idx ON abstract(work_id)
    WHERE canonical;

-- Only allow one instance of each locale per work
CREATE UNIQUE INDEX IF NOT EXISTS abstract_uniq_locale_idx ON abstract(work_id, locale_code);
-- Drop title-related columns from the work table
ALTER TABLE work
    DROP COLUMN short_abstract,
    DROP COLUMN long_abstract;