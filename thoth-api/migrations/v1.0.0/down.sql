BEGIN;

-------------------------------------------------------------------------------
-- 1. Drop the current deterministic work_relation_work_updated_at_with_relations
--    and its trigger
-------------------------------------------------------------------------------

DROP TRIGGER IF EXISTS set_work_relation_updated_at_with_relations ON work_relation;
DROP FUNCTION IF EXISTS work_relation_work_updated_at_with_relations() CASCADE;

-------------------------------------------------------------------------------
-- 2. Restore the previous work_relation_work_updated_at_with_relations()
--    that bumps all involved works whenever a relation row changes
-------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION work_relation_work_updated_at_with_relations() RETURNS trigger AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD
        ) THEN
        UPDATE work
        SET updated_at_with_relations = current_timestamp
        WHERE work_id = OLD.relator_work_id OR work_id = NEW.relator_work_id
           OR work_id = OLD.related_work_id OR work_id = NEW.related_work_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER set_work_updated_at_with_relations AFTER INSERT OR UPDATE OR DELETE ON work_relation
    FOR EACH ROW EXECUTE PROCEDURE work_relation_work_updated_at_with_relations();

-------------------------------------------------------------------------------
-- 3. Restore work_work_updated_at_with_relations() and its trigger on work
-------------------------------------------------------------------------------

DROP TRIGGER IF EXISTS set_work_work_updated_at_with_relations ON work;
DROP FUNCTION IF EXISTS work_work_updated_at_with_relations() CASCADE;

CREATE OR REPLACE FUNCTION work_work_updated_at_with_relations() RETURNS trigger AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD
        ) THEN
        UPDATE work
        SET updated_at_with_relations = current_timestamp
        FROM work_relation
        -- The positions of relator/related IDs in this statement don't matter, as
        -- every work_relation record has a mirrored record with relator/related IDs swapped
        WHERE work.work_id = work_relation.relator_work_id AND work_relation.related_work_id = NEW.work_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER set_work_work_updated_at_with_relations
    AFTER UPDATE ON work
    FOR EACH ROW EXECUTE PROCEDURE work_work_updated_at_with_relations();

COMMIT;