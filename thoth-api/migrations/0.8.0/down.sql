ALTER TABLE work
    ADD COLUMN width double precision CHECK (width > 0.0),
    ADD COLUMN height double precision CHECK (height > 0.0),
    ADD CONSTRAINT work_chapter_no_width CHECK
        (width IS NULL OR work_type <> 'book-chapter'),
    ADD CONSTRAINT work_chapter_no_height CHECK
        (height IS NULL OR work_type <> 'book-chapter');

-- Migrate publication dimension information back into work table as far as possible
-- (width/height in mm only) before dropping publication dimension columns. Where
-- dimensions for both paperback and hardback are given, assume the paperback is canonical.
UPDATE work
    SET width = publication.width_mm
        FROM publication
        WHERE work.work_type <> 'book-chapter'
        AND work.work_id = publication.work_id
        AND publication.width_mm IS NOT NULL
        AND publication.publication_type = 'Paperback';
UPDATE work
    SET width = publication.width_mm
        FROM publication
        WHERE work.work_type <> 'book-chapter'
        AND work.work_id = publication.work_id
        AND work.width IS NULL
        AND publication.width_mm IS NOT NULL
        AND publication.publication_type = 'Hardback';

UPDATE work
    SET height = publication.height_mm
        FROM publication
        WHERE work.work_type <> 'book-chapter'
        AND work.work_id = publication.work_id
        AND publication.height_mm IS NOT NULL
        AND publication.publication_type = 'Paperback';
UPDATE work
    SET height = publication.height_mm
        FROM publication
        WHERE work.work_type <> 'book-chapter'
        AND work.work_id = publication.work_id
        AND work.height IS NULL
        AND publication.height_mm IS NOT NULL
        AND publication.publication_type = 'Hardback';

DROP TRIGGER publication_chapter_no_dimensions_check ON publication;

ALTER TABLE publication
    DROP CONSTRAINT publication_non_physical_no_dimensions,
    DROP CONSTRAINT publication_weight_g_not_missing,
    DROP CONSTRAINT publication_weight_oz_not_missing,
    DROP CONSTRAINT publication_width_mm_not_missing,
    DROP CONSTRAINT publication_width_in_not_missing,
    DROP CONSTRAINT publication_height_mm_not_missing,
    DROP CONSTRAINT publication_height_in_not_missing,
    DROP CONSTRAINT publication_depth_mm_not_missing,
    DROP CONSTRAINT publication_depth_in_not_missing,
    DROP COLUMN weight_g,
    DROP COLUMN weight_oz,
    DROP COLUMN width_mm,
    DROP COLUMN width_in,
    DROP COLUMN height_mm,
    DROP COLUMN height_in,
    DROP COLUMN depth_mm,
    DROP COLUMN depth_in;

DROP FUNCTION IF EXISTS publication_chapter_no_dimensions();
