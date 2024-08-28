-- We cannot drop individual enum values - we must drop the type and recreate it

-- Drop constraints, otherwise it won't be able to cast to text
ALTER TABLE publication
    DROP CONSTRAINT IF EXISTS publication_publication_type_work_id_uniq,
    DROP CONSTRAINT IF EXISTS publication_non_physical_no_dimensions;

-- Delete publications with about-to-be-dropped types
DELETE FROM publication WHERE publication_type IN ('MP3', 'WAV');
ALTER TABLE publication ALTER COLUMN publication_type TYPE text;
DROP TYPE publication_type;
CREATE TYPE publication_type AS ENUM (
    'Paperback',
    'Hardback',
    'PDF',
    'HTML',
    'XML',
    'Epub',
    'Mobi',
    'AZW3',
    'DOCX',
    'FictionBook'
);
ALTER TABLE publication ALTER COLUMN publication_type TYPE publication_type USING publication_type::publication_type;

ALTER TABLE publication
    ADD CONSTRAINT publication_publication_type_work_id_uniq UNIQUE (publication_type, work_id),
    ADD CONSTRAINT publication_non_physical_no_dimensions CHECK
        ((width_mm IS NULL AND width_in IS NULL
            AND height_mm IS NULL AND height_in IS NULL
            AND depth_mm IS NULL AND depth_in IS NULL
            AND weight_g IS NULL AND weight_oz IS NULL)
        OR publication_type = 'Paperback' OR publication_type = 'Hardback');
