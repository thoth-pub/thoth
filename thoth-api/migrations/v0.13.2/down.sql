-- Add title-related columns back to the work table
ALTER TABLE work
ADD COLUMN full_title TEXT NOT NULL CHECK (octet_length(full_title) >= 1),
ADD COLUMN title TEXT NOT NULL CHECK (octet_length(title) >= 1),
ADD COLUMN subtitle TEXT CHECK (octet_length(subtitle) >= 1);

-- Restore enlish title data from the title table to the work table
UPDATE work
SET
    full_title = COALESCE(
        (
            SELECT
                full_title
            FROM
                title
                JOIN locale ON title.locale_id = locale.locale_id
            WHERE
                title.work_id = work.work_id
                AND locale.code = 'en'
            LIMIT
                1
        ),
        'Untitled'
    ),
    title = COALESCE(
        (
            SELECT
                title
            FROM
                title
                JOIN locale ON title.locale_id = locale.locale_id
            WHERE
                title.work_id = work.work_id
                AND locale.code = 'en'
            LIMIT
                1
        ),
        'Untitled'
    ),
    subtitle = (
        SELECT
            subtitle
        FROM
            title
            JOIN locale ON title.locale_id = locale.locale_id
        WHERE
            title.work_id = work.work_id
            AND locale.code = 'en'
        LIMIT
            1
    )
WHERE
    EXISTS (
        SELECT
            1
        FROM
            title
            JOIN locale ON title.locale_id = locale.locale_id
        WHERE
            title.work_id = work.work_id
            AND locale.code = 'en'
    );

-- Drop title and locale tables
DROP TABLE title;

DROP TABLE locale;