ALTER TABLE series
    DROP COLUMN series_description,
    DROP COLUMN series_cfp_url;

-- We cannot drop individual enum values - we must drop the type and recreate it
--
-- Delete publications with about-to-be-dropped types
DELETE FROM publication WHERE publication_type IN ('AZW3', 'DOCX', 'FictionBook');
ALTER TABLE publication ALTER publication_type TYPE text;
DROP TYPE publication_type;
CREATE TYPE publication_type AS ENUM (
    'Paperback',
    'Hardback',
    'PDF',
    'HTML',
    'XML',
    'Epub',
    'Mobi'
);
ALTER TABLE publication ALTER publication_type TYPE publication_type USING publication_type::publication_type;
