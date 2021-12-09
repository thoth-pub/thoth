DROP TABLE work_relation_history;
DROP TRIGGER set_updated_at ON work_relation;
DROP TABLE work_relation;
DROP TYPE IF EXISTS relation_type;

ALTER TABLE work
    DROP CONSTRAINT work_non_chapter_has_edition,
    DROP CONSTRAINT work_chapter_no_edition,
    DROP CONSTRAINT work_chapter_no_width,
    DROP CONSTRAINT work_chapter_no_height,
    DROP CONSTRAINT work_chapter_no_toc,
    DROP CONSTRAINT work_chapter_no_lccn,
    DROP CONSTRAINT work_chapter_no_oclc;

-- Set a default edition value for any chapter records before
-- reintroducing the original blanket edition-not-null constraint.
UPDATE work
    SET edition = 1
    WHERE work_type = 'book-chapter';

ALTER TABLE work
    ALTER COLUMN edition SET NOT NULL;
