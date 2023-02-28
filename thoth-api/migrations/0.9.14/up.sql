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

-- The following tables all reference tables which reference the work table.
-- As they are at the end of this chain of references, any creation, update or
-- deletion on them should also be marked as an update on the 'grandparent' work.
CREATE OR REPLACE FUNCTION affiliation_work_table_relation_updated_at() RETURNS trigger AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD
    ) THEN
        UPDATE work
        SET relation_updated_at = current_timestamp
        FROM contribution
        WHERE work_id = contribution.work_id AND contribution.contribution_id = OLD.contribution_id
            OR work_id = contribution.work_id AND contribution.contribution_id = NEW.contribution_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER set_work_table_relation_updated_at AFTER INSERT OR UPDATE OR DELETE ON affiliation
    FOR EACH ROW EXECUTE PROCEDURE affiliation_work_table_relation_updated_at();

CREATE OR REPLACE FUNCTION location_work_table_relation_updated_at() RETURNS trigger AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD
    ) THEN
        UPDATE work
        SET relation_updated_at = current_timestamp
        FROM publication
        WHERE work_id = publication.work_id AND publication.publication_id = OLD.publication_id
            OR work_id = publication.work_id AND publication.publication_id = NEW.publication_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER set_work_table_relation_updated_at AFTER INSERT OR UPDATE OR DELETE ON location
    FOR EACH ROW EXECUTE PROCEDURE location_work_table_relation_updated_at();

CREATE OR REPLACE FUNCTION price_work_table_relation_updated_at() RETURNS trigger AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD
    ) THEN
        UPDATE work
        SET relation_updated_at = current_timestamp
        FROM publication
        WHERE work_id = publication.work_id AND publication.publication_id = OLD.publication_id
            OR work_id = publication.work_id AND publication.publication_id = NEW.publication_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER set_work_table_relation_updated_at AFTER INSERT OR UPDATE OR DELETE ON price
    FOR EACH ROW EXECUTE PROCEDURE price_work_table_relation_updated_at();

CREATE OR REPLACE FUNCTION contributor_work_table_relation_updated_at() RETURNS trigger AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD
    ) THEN
        UPDATE work
        SET relation_updated_at = current_timestamp
        FROM contribution
        -- No need to check OLD.contributor_id, as this will be the same as NEW.contributor_id in all relevant cases
        -- (contributor_id can't be changed on contributors which are referenced by existing contributions)
        WHERE work_id = contribution.work_id AND contribution.contributor_id = NEW.contributor_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Deleting a contributor will also delete its contributions, setting relation_updated_at where relevant.
-- Adding a contributor will not affect any existing works, because no contributions will reference it yet.
CREATE TRIGGER set_work_table_relation_updated_at AFTER UPDATE ON contributor
    FOR EACH ROW EXECUTE PROCEDURE contributor_work_table_relation_updated_at();

CREATE OR REPLACE FUNCTION institution_work_table_relation_updated_at() RETURNS trigger AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD
    ) THEN
        UPDATE work
        SET relation_updated_at = current_timestamp
        FROM funding, affiliation, contribution
        -- Same as contributor above (but can be connected to work via two different tables)
        WHERE work_id = funding.work_id AND funding.institution_id = NEW.institution_id
            OR work_id = contribution.work_id AND contribution.contribution_id = affiliation.contribution_id AND affiliation.institution_id = NEW.institution_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Same as contributor above
CREATE TRIGGER set_work_table_relation_updated_at AFTER UPDATE ON institution
    FOR EACH ROW EXECUTE PROCEDURE institution_work_table_relation_updated_at();

CREATE OR REPLACE FUNCTION publisher_work_table_relation_updated_at() RETURNS trigger AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD
    ) THEN
        UPDATE work
        SET relation_updated_at = current_timestamp
        FROM imprint
        -- Same as contributor above
        WHERE imprint_id = imprint.imprint_id AND imprint.publisher_id = NEW.publisher_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Same as contributor above
CREATE TRIGGER set_work_table_relation_updated_at AFTER UPDATE ON publisher
    FOR EACH ROW EXECUTE PROCEDURE publisher_work_table_relation_updated_at();

CREATE OR REPLACE FUNCTION series_work_table_relation_updated_at() RETURNS trigger AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD
    ) THEN
        UPDATE work
        SET relation_updated_at = current_timestamp
        FROM issue
        -- Same as contributor above (note that although series is also connected to work
        -- via the imprint_id, changes to a series don't affect its imprint)
        WHERE work_id = issue.work_id AND issue.series_id = NEW.series_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Same as contributor above
CREATE TRIGGER set_work_table_relation_updated_at AFTER UPDATE ON series
    FOR EACH ROW EXECUTE PROCEDURE series_work_table_relation_updated_at();

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
