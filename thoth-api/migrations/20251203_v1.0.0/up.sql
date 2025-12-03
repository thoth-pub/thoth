-------------------------------------------------------------------------------
-- 1. Remove the helper function, and associated triggers, that propagates
-- from work -> related works
-------------------------------------------------------------------------------

DROP FUNCTION IF EXISTS work_work_updated_at_with_relations() CASCADE;

-------------------------------------------------------------------------------
-- 2. Redefine work_relation_work_updated_at_with_relations() to update the
--    two endpoint works in deterministic order (LEAST/ GREATEST).
-------------------------------------------------------------------------------

DROP FUNCTION IF EXISTS work_relation_work_updated_at_with_relations() CASCADE;

CREATE OR REPLACE FUNCTION work_relation_work_updated_at_with_relations()
    RETURNS trigger AS $$
DECLARE
    w1 uuid;  -- smaller work_id
    w2 uuid;  -- larger work_id
BEGIN
    -- If nothing really changed, skip
    IF NEW IS NOT DISTINCT FROM OLD THEN
        RETURN NULL;
    END IF;

    -- Determine the two work IDs involved in this relation
    IF TG_OP = 'DELETE' THEN
        w1 := LEAST(OLD.relator_work_id, OLD.related_work_id);
        w2 := GREATEST(OLD.relator_work_id, OLD.related_work_id);
    ELSE
        w1 := LEAST(NEW.relator_work_id, NEW.related_work_id);
        w2 := GREATEST(NEW.relator_work_id, NEW.related_work_id);
    END IF;

    -- Always lock/update in deterministic order: smaller ID first, then larger
    UPDATE work
    SET updated_at_with_relations = current_timestamp
    WHERE work_id = w1;

    IF w2 IS DISTINCT FROM w1 THEN
        UPDATE work
        SET updated_at_with_relations = current_timestamp
        WHERE work_id = w2;
    END IF;

    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER set_work_relation_updated_at_with_relations
    AFTER INSERT OR UPDATE OR DELETE ON work_relation
    FOR EACH ROW EXECUTE PROCEDURE work_relation_work_updated_at_with_relations();
