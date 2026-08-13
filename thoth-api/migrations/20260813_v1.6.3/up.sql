-- THOTH-CHAPTER-01 / #803 — enforce a single parent for BookChapter works.
--
-- A Work with work_type = 'book-chapter' may have at most ONE distinct parent,
-- represented by a work_relation row with relation_type = 'is-child-of'
-- (relator_work_id = chapter, related_work_id = parent). Zero parents remain
-- valid. Non-book-chapter works are unaffected by the rule.
--
-- ATOMIC ACTIVATION: Diesel runs each migration inside a single transaction
-- (see the repository migration runner, crate::db::run_migrations, which calls
-- run_pending_migrations). The table locks taken below are therefore held until
-- this migration commits — i.e. until both triggers are in place. Combined with
-- the guard, there is NO window in which the clean-data check has passed, writes
-- are again possible, and enforcement is not yet active.

-- Exclude concurrent writers for the guard + install window. SHARE ROW EXCLUSIVE
-- conflicts with the ROW EXCLUSIVE lock every INSERT/UPDATE/DELETE takes, so no
-- DML can change relevant state between the guard and trigger creation. Ordinary
-- ACCESS SHARE reads are not blocked by this mode.
LOCK TABLE work, work_relation IN SHARE ROW EXCLUSIVE MODE;

-- Corrupt-data guard: refuse to activate if any book-chapter already has more
-- than one DISTINCT is-child-of parent. Never repair, reparent, or delete data.
DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM work_relation wr
        JOIN work w ON w.work_id = wr.relator_work_id
        WHERE wr.relation_type = 'is-child-of'
          AND w.work_type = 'book-chapter'
        GROUP BY wr.relator_work_id
        HAVING COUNT(DISTINCT wr.related_work_id) > 1
    ) THEN
        RAISE EXCEPTION
            'Cannot enforce the single-parent rule (#803): one or more book-chapter works already have more than one distinct parent. Resolve these records before enabling enforcement.';
    END IF;
END $$;

-- Enforcement on the relation side. For any is-child-of row whose relator is a
-- book-chapter, reject a second DISTINCT parent. The lock and work_type read
-- happen in ONE locking read (FOR NO KEY UPDATE) BEFORE deciding whether the
-- rule applies, so a concurrent work_type transition on the same work row cannot
-- race this decision. A non-book-chapter is-child-of mutation also briefly takes
-- this row lock: that is an intentional synchronization effect, not a semantic
-- restriction on the non-book-chapter work.
CREATE FUNCTION work_relation_enforce_single_book_chapter_parent()
    RETURNS trigger
    LANGUAGE plpgsql AS $$
DECLARE
    relator_type text;
BEGIN
    IF NEW.relation_type = 'is-child-of' THEN
        SELECT work_type::text
          INTO relator_type
          FROM work
         WHERE work_id = NEW.relator_work_id
           FOR NO KEY UPDATE;

        IF relator_type = 'book-chapter' THEN
            IF EXISTS (
                SELECT 1
                  FROM work_relation
                 WHERE relator_work_id = NEW.relator_work_id
                   AND relation_type = 'is-child-of'
                   AND related_work_id <> NEW.related_work_id
                   AND work_relation_id <> NEW.work_relation_id
            ) THEN
                RAISE EXCEPTION 'A book chapter may belong to only one parent work'
                    USING ERRCODE = 'unique_violation',
                          CONSTRAINT = 'work_relation_single_book_chapter_parent';
            END IF;
        END IF;
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER work_relation_single_book_chapter_parent
    BEFORE INSERT OR UPDATE ON work_relation
    FOR EACH ROW
    EXECUTE FUNCTION work_relation_enforce_single_book_chapter_parent();

-- Enforcement on the work_type-transition side. Entering book-chapter state must
-- not create a >1-parent chapter. Uses the SAME per-work serialization lock,
-- taken before counting, and only fires on a genuine transition into
-- book-chapter (not on unrelated updates to an already-book-chapter work).
CREATE FUNCTION work_enforce_single_book_chapter_parent_on_type()
    RETURNS trigger
    LANGUAGE plpgsql AS $$
BEGIN
    PERFORM 1 FROM work WHERE work_id = NEW.work_id FOR NO KEY UPDATE;

    IF (
        SELECT COUNT(DISTINCT related_work_id)
          FROM work_relation
         WHERE relator_work_id = NEW.work_id
           AND relation_type = 'is-child-of'
    ) > 1 THEN
        RAISE EXCEPTION 'A book chapter may belong to only one parent work'
            USING ERRCODE = 'unique_violation',
                  CONSTRAINT = 'work_relation_single_book_chapter_parent';
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER work_single_book_chapter_parent_on_type
    BEFORE UPDATE OF work_type ON work
    FOR EACH ROW
    WHEN (OLD.work_type IS DISTINCT FROM NEW.work_type AND NEW.work_type = 'book-chapter')
    EXECUTE FUNCTION work_enforce_single_book_chapter_parent_on_type();
