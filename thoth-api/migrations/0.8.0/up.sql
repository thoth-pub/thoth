ALTER TABLE publication
    ADD COLUMN width_mm double precision CHECK (width_mm > 0.0),
    ADD COLUMN width_in double precision CHECK (width_in > 0.0),
    ADD COLUMN height_mm double precision CHECK (height_mm > 0.0),
    ADD COLUMN height_in double precision CHECK (height_in > 0.0),
    ADD COLUMN depth_mm double precision CHECK (depth_mm > 0.0),
    ADD COLUMN depth_in double precision CHECK (depth_in > 0.0),
    ADD COLUMN weight_g double precision CHECK (weight_g > 0.0),
    ADD COLUMN weight_oz double precision CHECK (weight_oz > 0.0),
    ADD CONSTRAINT publication_non_physical_no_dimensions CHECK
        ((width_mm IS NULL AND width_in IS NULL
            AND height_mm IS NULL AND height_in IS NULL
            AND depth_mm IS NULL AND depth_in IS NULL
            AND weight_g IS NULL AND weight_oz IS NULL)
        OR publication_type = 'Paperback' OR publication_type = 'Hardback'),
    ADD CONSTRAINT publication_depth_mm_not_missing CHECK
        (depth_mm IS NOT NULL OR depth_in IS NULL),
    ADD CONSTRAINT publication_depth_in_not_missing CHECK
        (depth_in IS NOT NULL OR depth_mm IS NULL),
    ADD CONSTRAINT publication_weight_g_not_missing CHECK
        (weight_g IS NOT NULL OR weight_oz IS NULL),
    ADD CONSTRAINT publication_weight_oz_not_missing CHECK
        (weight_oz IS NOT NULL OR weight_g IS NULL);

CREATE OR REPLACE FUNCTION publication_chapter_no_dimensions() RETURNS trigger AS $$
BEGIN
    IF (
        (SELECT work_type FROM work WHERE work.work_id = NEW.work_id) = 'book-chapter' AND (
            NEW.width_mm IS NOT NULL OR
            NEW.width_in IS NOT NULL OR
            NEW.height_mm IS NOT NULL OR
            NEW.height_in IS NOT NULL OR
            NEW.depth_mm IS NOT NULL OR
            NEW.depth_in IS NOT NULL OR
            NEW.weight_g IS NOT NULL OR
            NEW.weight_oz IS NOT NULL
        )
    ) THEN
        RAISE EXCEPTION 'Chapters cannot have dimensions (Width/Height/Depth/Weight)';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER publication_chapter_no_dimensions_check BEFORE INSERT OR UPDATE ON publication
    FOR EACH ROW EXECUTE PROCEDURE publication_chapter_no_dimensions();

-- Migrate work dimension information into publication table before dropping work
-- width/height columns. Assume dimensions are same for paperback and hardback.
UPDATE publication
    SET width_mm = work.width
        FROM work
        WHERE publication.work_id = work.work_id
        AND work.width IS NOT NULL
        AND (publication.publication_type = 'Paperback' OR publication.publication_type = 'Hardback');

UPDATE publication
    SET height_mm = work.height
        FROM work
        WHERE publication.work_id = work.work_id
        AND work.height IS NOT NULL
        AND (publication.publication_type = 'Paperback' OR publication.publication_type = 'Hardback');

-- Add imperial dimension information based on metric. Conversion logic used here
-- replicates convert_length_from_to() function in thoth-api/src/model/mod.rs.
UPDATE publication
    SET width_in = round((width_mm / 25.4)::numeric, 2)
    WHERE width_mm IS NOT NULL;

UPDATE publication
    SET height_in = round((height_mm / 25.4)::numeric, 2)
    WHERE height_mm IS NOT NULL;

ALTER TABLE publication
    ADD CONSTRAINT publication_width_mm_not_missing CHECK
        (width_mm IS NOT NULL OR width_in IS NULL),
    ADD CONSTRAINT publication_width_in_not_missing CHECK
        (width_in IS NOT NULL OR width_mm IS NULL),
    ADD CONSTRAINT publication_height_mm_not_missing CHECK
        (height_mm IS NOT NULL OR height_in IS NULL),
    ADD CONSTRAINT publication_height_in_not_missing CHECK
        (height_in IS NOT NULL OR height_mm IS NULL);

ALTER TABLE work
    DROP CONSTRAINT work_chapter_no_width,
    DROP CONSTRAINT work_chapter_no_height,
    DROP COLUMN width,
    DROP COLUMN height;
