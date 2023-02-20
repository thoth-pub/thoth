-- Add work table field to track when any of the work's relations were last updated.

ALTER TABLE work
    ADD COLUMN relation_updated_at TIMESTAMP NOT NULL DEFAULT '1970-01-01 00:00:00';

-- Add triggers to update this field whenever a relation is created, updated or deleted.

CREATE OR REPLACE FUNCTION work_table_relation_updated_at() RETURNS trigger AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD
    ) THEN
        UPDATE work
        SET relation_updated_at = current_timestamp
        WHERE work_id = OLD.work_id OR work_id = NEW.work_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER set_work_table_relation_updated_at AFTER INSERT OR UPDATE OR DELETE ON contribution
    FOR EACH ROW EXECUTE PROCEDURE work_table_relation_updated_at();

CREATE TRIGGER set_work_table_relation_updated_at AFTER INSERT OR UPDATE OR DELETE ON funding
    FOR EACH ROW EXECUTE PROCEDURE work_table_relation_updated_at();

CREATE TRIGGER set_work_table_relation_updated_at AFTER INSERT OR UPDATE OR DELETE ON issue
    FOR EACH ROW EXECUTE PROCEDURE work_table_relation_updated_at();

CREATE TRIGGER set_work_table_relation_updated_at AFTER INSERT OR UPDATE OR DELETE ON language
    FOR EACH ROW EXECUTE PROCEDURE work_table_relation_updated_at();

CREATE TRIGGER set_work_table_relation_updated_at AFTER INSERT OR UPDATE OR DELETE ON publication
    FOR EACH ROW EXECUTE PROCEDURE work_table_relation_updated_at();

CREATE TRIGGER set_work_table_relation_updated_at AFTER INSERT OR UPDATE OR DELETE ON reference
    FOR EACH ROW EXECUTE PROCEDURE work_table_relation_updated_at();

CREATE TRIGGER set_work_table_relation_updated_at AFTER INSERT OR UPDATE OR DELETE ON subject
    FOR EACH ROW EXECUTE PROCEDURE work_table_relation_updated_at();

CREATE OR REPLACE FUNCTION work_relation_work_table_relation_updated_at() RETURNS trigger AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD
    ) THEN
        UPDATE work
        SET relation_updated_at = current_timestamp
        WHERE work_id = OLD.relator_work_id OR work_id = NEW.relator_work_id
            OR work_id = OLD.related_work_id OR work_id = NEW.related_work_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER set_work_table_relation_updated_at AFTER INSERT OR UPDATE OR DELETE ON work_relation
    FOR EACH ROW EXECUTE PROCEDURE work_relation_work_table_relation_updated_at();

-- Amend existing trigger which sets updated_at value on work table
-- to avoid setting updated_at when relation_updated_at changes.

CREATE OR REPLACE FUNCTION work_set_updated_at() RETURNS trigger AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD AND
        NEW.updated_at IS NOT DISTINCT FROM OLD.updated_at AND
        NEW.relation_updated_at IS NOT DISTINCT FROM OLD.relation_updated_at
    ) THEN
        NEW.updated_at := current_timestamp;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS set_updated_at ON work;

CREATE TRIGGER set_updated_at BEFORE UPDATE ON work
    FOR EACH ROW EXECUTE PROCEDURE work_set_updated_at();
