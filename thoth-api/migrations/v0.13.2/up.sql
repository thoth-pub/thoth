-- Create the locale table
CREATE TABLE locale (
    locale_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    code TEXT NOT NULL CHECK (octet_length(code) >= 1),
    name TEXT NOT NULL CHECK (octet_length(name) >= 1)
);

-- Populate locale table with JSON data and English
INSERT INTO locale (locale_id, code, name)
VALUES (uuid_generate_v4(), 'en', 'English (en)');

-- Create the title table
CREATE TABLE
    title (
        title id UUID PRIMARY KEY DEFAULT uuid_generate_v4 (),
        work_id UUID NOT NULL REFERENCES work (work_id) ON DELETE CASCADE,
        locale_id UUID NOT NULL REFERENCES locale (locale_id) ON DELETE CASCADE,
        full_title TEXT NOT NULL CHECK (octet_length(full_title) >= 1),
        title TEXT NOT NULL CHECK (octet_length(title) >= 1),
        subtitle TEXT CHECK (octet_length(subtitle) >= 1),
    );

-- Migrate existing work titles to the title table with English locale
INSERT INTO
    title (
        id,
        work_id,
        locale_id,
        full_title,
        title,
        subtitle
    )
SELECT
    UUID (),
    work_id,
    (
        SELECT
            locale_id
        FROM
            locale
        WHERE
            code = 'en'
        LIMIT
            1
    ),
    full_title,
    title,
    subtitle
FROM
    work
WHERE
    full_title IS NOT NULL
    AND title IS NOT NULL;

-- Drop title-related columns from the work table
ALTER TABLE work
DROP COLUMN full_title,
DROP COLUMN title,
DROP COLUMN subtitle;