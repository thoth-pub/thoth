-- Reverse THOTH-CHAPTER-01 / #803 single-parent enforcement.
--
-- Removes only the triggers and their functions. No Work or WorkRelation data is
-- deleted, reparented, or rewritten; previous write acceptance is restored.
DROP TRIGGER IF EXISTS work_single_book_chapter_parent_on_type ON work;
DROP TRIGGER IF EXISTS work_relation_single_book_chapter_parent ON work_relation;
DROP FUNCTION IF EXISTS work_enforce_single_book_chapter_parent_on_type();
DROP FUNCTION IF EXISTS work_relation_enforce_single_book_chapter_parent();
