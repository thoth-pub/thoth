-- Add title-related columns back to the work table
ALTER TABLE work
    ADD COLUMN full_title TEXT CHECK (octet_length(full_title) >= 1),
    ADD COLUMN title TEXT CHECK (octet_length(title) >= 1),
    ADD COLUMN subtitle TEXT CHECK (octet_length(subtitle) >= 1);

-- Migrate data back from title table to work table
UPDATE work w
SET 
    full_title = regexp_replace(t.full_title, '^<full_title>(.*)</full_title>$', '\\1'),
    title = regexp_replace(t.title, '^<title>(.*)</title>$', '\\1'),
    subtitle = CASE WHEN t.subtitle IS NOT NULL THEN regexp_replace(t.subtitle, '^<subtitle>(.*)</subtitle>$', '\\1') ELSE NULL END
FROM title t
WHERE w.work_id = t.work_id
    AND t.canonical = TRUE;

-- Drop the unique index for canonical titles
DROP INDEX IF EXISTS title_uniq_locale_idx;
-- Drop the unique index for locale codes
DROP INDEX IF EXISTS title_unique_canonical_true_idx;

-- Drop the title_history table
DROP TABLE title_history;

-- Drop the title table
DROP TABLE title;

-- Recreate short_abstract and long_abstract columns in the work table
ALTER TABLE work
    ADD COLUMN short_abstract TEXT CHECK (octet_length(short_abstract) >= 1),
    ADD COLUMN long_abstract TEXT CHECK (octet_length(long_abstract) >= 1);

-- -----------------------------------------------------------------------------
-- Reverse Conversion Function
-- -----------------------------------------------------------------------------
-- This function attempts to convert a JATS XML string back into a format that
-- resembles the original plaintext or Markdown. This is the reverse of the
-- `convert_to_jats` function from the `up` migration.
--
-- NOTE: This is a best-effort reversal. The primary goal is to make the data
-- readable and usable, not to restore the original format with 100% fidelity.
-- -----------------------------------------------------------------------------
CREATE OR REPLACE FUNCTION convert_from_jats(jats_in TEXT)
RETURNS TEXT AS $$
DECLARE
    processed_content TEXT := jats_in;
BEGIN
    -- Return NULL immediately if input is NULL or empty.
    IF processed_content IS NULL OR processed_content = '' THEN
        RETURN NULL;
    END IF;

    -- The order of replacements is important to handle nested tags correctly.

    -- Convert JATS tags back to a Markdown-like format.
    processed_content := regexp_replace(processed_content, '<ext-link xlink:href="([^"]+)">([^<]+)</ext-link>', '[\2](\1)', 'gi');
    processed_content := regexp_replace(processed_content, '<bold>([^<]+)</bold>', '**\1**', 'gi');
    processed_content := regexp_replace(processed_content, '<italic>([^<]+)</italic>', '*\1*', 'gi');
    processed_content := regexp_replace(processed_content, '<monospace>([^<]+)</monospace>', '`\1`', 'gi');
    processed_content := regexp_replace(processed_content, '<sc>([^<]+)</sc>', '\1', 'gi'); -- Revert small-caps to original text
    processed_content := regexp_replace(processed_content, '<sup[^>]*>([^<]+)</sup>', '^\1^', 'gi'); -- A possible representation for superscript
    processed_content := regexp_replace(processed_content, '<sub[^>]*>([^<]+)</sub>', '~\1~', 'gi'); -- A possible representation for subscript
    processed_content := regexp_replace(processed_content, '<break\s*/>', E'\n', 'gi');

    -- Remove paragraph tags and handle the spacing.
    -- Replace closing tags with double newlines to separate paragraphs.
    processed_content := regexp_replace(processed_content, '</p>', E'\n\n', 'gi');
    -- Strip any remaining opening paragraph tags.
    processed_content := regexp_replace(processed_content, '<p>', '', 'gi');

    -- Clean up any leftover simple HTML tags that were not converted.
    processed_content := regexp_replace(processed_content, '<[^>]+>', '', 'g');

    -- Trim leading/trailing whitespace that may result from tag removal.
    processed_content := trim(processed_content);

    RETURN processed_content;
END;
$$ LANGUAGE plpgsql;


-- Migrate data back from the abstract table to the work table using the reverse conversion
UPDATE work
SET
    short_abstract = convert_from_jats(abstract.content)
FROM
    abstract
WHERE
    abstract.work_id = work.work_id
    AND abstract.abstract_type = 'short'
    AND abstract.canonical = TRUE;

UPDATE work
SET
    long_abstract = convert_from_jats(abstract.content)
FROM
    abstract
WHERE
    abstract.work_id = work.work_id
    AND abstract.abstract_type = 'long'
    AND abstract.canonical = TRUE;

-- Drop unique indexes created for the abstract table
DROP INDEX IF EXISTS abstract_unique_canonical_true_idx;
DROP INDEX IF EXISTS abstract_uniq_locale_idx;

-- Drop the abstract_history table
DROP TABLE abstract_history;
-- Drop the abstract table and its related objects
DROP TABLE IF EXISTS abstract;

-- Drop the AbstractType enum
DROP TYPE IF EXISTS abstract_type;

ALTER TABLE contribution
    ADD COLUMN biography TEXT CHECK (octet_length(short_abstract) >= 1);

-- Migrate data back from the abstract table to the work table using the reverse conversion
UPDATE contribution
SET
    biography = convert_from_jats(biography.content)
FROM
    biography
WHERE
    biography.contribution_id = contribution.contribution_id
    AND biography.canonical = TRUE;

-- Drop unique indexes created for the biography table
DROP INDEX IF EXISTS biography_unique_canonical_true_idx;
DROP INDEX IF EXISTS biography_uniq_locale_idx;

-- Drop the biography_history table
DROP TABLE biography_history;
-- Drop the biography table and its related objects
DROP TABLE IF EXISTS biography;

-- Drop the locale_code enum type
DROP TYPE locale_code;

-- Clean up the reverse conversion function
DROP FUNCTION convert_from_jats(TEXT);