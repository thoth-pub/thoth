-- Add work table field to track when any of the work's relations were last updated.

ALTER TABLE work
    ADD COLUMN relation_updated_at TIMESTAMP NULL;

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

-- Obtain current last relation update timestamp for all existing works.
WITH update_times AS
(
    SELECT w.work_id, GREATEST(
        w.updated_at, c.updated_at, f.updated_at, i.updated_at, iu.updated_at, l.updated_at, p.updated_at,
        r.updated_at, s.updated_at, wr.updated_at, a.updated_at, lo.updated_at, pr.updated_at,
        co.updated_at, inf.updated_at, ina.updated_at, pu.updated_at, se.updated_at, wo.updated_at
    ) last_updated
    FROM work w
        LEFT JOIN contribution c USING (work_id)
        LEFT JOIN funding f USING (work_id)
        LEFT JOIN imprint i USING (imprint_id)
        LEFT JOIN issue iu USING (work_id)
        LEFT JOIN language l USING (work_id)
        LEFT JOIN publication p USING (work_id)
        LEFT JOIN reference r USING (work_id)
        LEFT JOIN subject s USING (work_id)
        LEFT JOIN work_relation wr ON w.work_id = wr.relator_work_id
        LEFT JOIN affiliation a ON c.contribution_id = a.contribution_id
        LEFT JOIN location lo ON p.publication_id = lo.publication_id
        LEFT JOIN price pr ON p.publication_id = pr.publication_id
        LEFT JOIN contributor co ON c.contributor_id = co.contributor_id
        LEFT JOIN institution inf ON f.institution_id = inf.institution_id
        LEFT JOIN institution ina ON a.institution_id = ina.institution_id
        LEFT JOIN publisher pu ON i.publisher_id = pu.publisher_id
        LEFT JOIN series se ON iu.series_id = se.series_id
        LEFT JOIN work wo ON wr.related_work_id = wo.work_id
    GROUP BY w.work_id, last_updated
)
UPDATE work
    SET relation_updated_at = update_times.last_updated
    FROM update_times
    WHERE work.work_id = update_times.work_id;

ALTER TABLE work
    ALTER COLUMN relation_updated_at SET NOT NULL,
    ALTER COLUMN relation_updated_at SET DEFAULT CURRENT_TIMESTAMP;

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
        -- We don't need to also check related_work_ids, as every work_relation
        -- record has a mirrored record with relator/related IDs swapped
        WHERE work_id = OLD.relator_work_id OR work_id = NEW.relator_work_id;
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
        WHERE work.work_id = contribution.work_id AND contribution.contribution_id = OLD.contribution_id
            OR work.work_id = contribution.work_id AND contribution.contribution_id = NEW.contribution_id;
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
        WHERE work.work_id = publication.work_id AND publication.publication_id = OLD.publication_id
            OR work.work_id = publication.work_id AND publication.publication_id = NEW.publication_id;
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
        WHERE work.work_id = publication.work_id AND publication.publication_id = OLD.publication_id
            OR work.work_id = publication.work_id AND publication.publication_id = NEW.publication_id;
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
        WHERE work.work_id = contribution.work_id AND contribution.contributor_id = NEW.contributor_id;
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
        -- Same as contributor above (but can be connected to work via two different tables)
        -- Use two separate UPDATE statements as this is much faster than combining the WHERE clauses
        -- using OR (in tests, this caused several seconds' delay when saving institution updates)
        UPDATE work
        SET relation_updated_at = current_timestamp
        FROM funding
        WHERE work.work_id = funding.work_id AND funding.institution_id = NEW.institution_id;
        UPDATE work
        SET relation_updated_at = current_timestamp
        FROM affiliation, contribution
        WHERE work.work_id = contribution.work_id AND contribution.contribution_id = affiliation.contribution_id AND affiliation.institution_id = NEW.institution_id;
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
        WHERE work.imprint_id = imprint.imprint_id AND imprint.publisher_id = NEW.publisher_id;
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
        WHERE work.work_id = issue.work_id AND issue.series_id = NEW.series_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Same as contributor above
CREATE TRIGGER set_work_table_relation_updated_at AFTER UPDATE ON series
    FOR EACH ROW EXECUTE PROCEDURE series_work_table_relation_updated_at();

-- Works can be related to each other via the work_relation table, with a relationship similar
-- to contributor above (a newly-created work won't have any references yet, etc)
CREATE OR REPLACE FUNCTION work_work_table_relation_updated_at() RETURNS trigger AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD AND
        NEW.relation_updated_at IS NOT DISTINCT FROM OLD.relation_updated_at
    ) THEN
        UPDATE work
        SET relation_updated_at = current_timestamp
        FROM work_relation
        -- The positions of relator/related IDs in this statement don't matter, as
        -- every work_relation record has a mirrored record with relator/related IDs swapped
        WHERE work.work_id = work_relation.relator_work_id AND work_relation.related_work_id = NEW.work_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER set_work_table_relation_updated_at AFTER UPDATE ON work
    FOR EACH ROW EXECUTE PROCEDURE work_work_table_relation_updated_at();

-- Imprint relationship is similar to contributor, although the tables are directly adjacent;
-- new imprints won't be referenced by works yet, and deleting an imprint also deletes its works
CREATE OR REPLACE FUNCTION imprint_work_table_relation_updated_at() RETURNS trigger AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD
    ) THEN
        UPDATE work
        SET relation_updated_at = current_timestamp
        WHERE imprint_id = NEW.imprint_id;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER set_work_table_relation_updated_at AFTER UPDATE ON imprint
    FOR EACH ROW EXECUTE PROCEDURE imprint_work_table_relation_updated_at();
