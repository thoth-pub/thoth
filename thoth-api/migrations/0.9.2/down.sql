-- We cannot drop individual enum values - we must drop the type and recreate it
--
-- Delete contributions with about-to-be-dropped types
DELETE FROM contribution WHERE contribution_type IN (
    'software-by',
    'research-by',
    'contributions-by',
    'indexer'
);
ALTER TABLE contribution ALTER contribution_type TYPE text;
DROP TYPE contribution_type;
CREATE TYPE contribution_type AS ENUM (
    'author',
    'editor',
    'translator',
    'photographer',
    'illustrator',
    'music-editor',
    'foreword-by',
    'introduction-by',
    'afterword-by',
    'preface-by'
);
ALTER TABLE contribution ALTER contribution_type TYPE contribution_type USING contribution_type::contribution_type;