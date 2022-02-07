ALTER TABLE series
    DROP COLUMN series_description,
    DROP COLUMN series_cfp_url;

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
ALTER TABLE publication ALTER publication_type TYPE publication_type USING publication::publication_type;
