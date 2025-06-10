-- Add title-related columns back to the work table
ALTER TABLE work
    ADD COLUMN full_title TEXT CHECK (octet_length(full_title) >= 1),
    ADD COLUMN title TEXT CHECK (octet_length(title) >= 1),
    ADD COLUMN subtitle TEXT CHECK (octet_length(subtitle) >= 1);

-- Migrate data back from title table to work table
UPDATE work w
SET 
    full_title = t.full_title,
    title = t.title,
    subtitle = t.subtitle
FROM title t
WHERE w.work_id = t.work_id
    AND t.canonical = TRUE;

-- Drop the unique index for canonical titles
DROP INDEX IF EXISTS title_uniq_locale_idx;
-- Drop the unique index for locale codes
DROP INDEX IF EXISTS title_unique_canonical_true_idx;

-- Drop the title table
DROP TABLE title_history;

-- Drop the title table
DROP TABLE title;

-- Drop the locale_code enum type
DROP TYPE locale_code;