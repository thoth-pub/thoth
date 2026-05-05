DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM endorsement
        WHERE author_name IS NULL OR octet_length(author_name) < 1
    ) THEN
        RAISE EXCEPTION 'Cannot make endorsement.author_name required: existing rows contain NULL or empty values';
    END IF;

    IF EXISTS (
        SELECT 1
        FROM work_featured_video
        WHERE title IS NULL OR octet_length(title) < 1
    ) THEN
        RAISE EXCEPTION 'Cannot make work_featured_video.title required: existing rows contain NULL or empty values';
    END IF;
END $$;

ALTER TABLE endorsement
    ALTER COLUMN author_name SET NOT NULL,
    ADD CONSTRAINT endorsement_author_name_check CHECK (octet_length(author_name) >= 1);

ALTER TABLE work_featured_video
    ALTER COLUMN title SET NOT NULL,
    ADD CONSTRAINT work_featured_video_title_check CHECK (octet_length(title) >= 1);
