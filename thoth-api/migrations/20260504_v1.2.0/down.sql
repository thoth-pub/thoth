ALTER TABLE endorsement
    DROP CONSTRAINT IF EXISTS endorsement_author_name_check,
    ALTER COLUMN author_name DROP NOT NULL;

ALTER TABLE work_featured_video
    DROP CONSTRAINT IF EXISTS work_featured_video_title_check,
    ALTER COLUMN title DROP NOT NULL;
