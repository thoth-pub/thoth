DROP TRIGGER IF EXISTS set_work_updated_at_with_relations ON contribution;

DROP TRIGGER IF EXISTS set_work_updated_at_with_relations ON funding;

DROP TRIGGER IF EXISTS set_work_updated_at_with_relations ON issue;

DROP TRIGGER IF EXISTS set_work_updated_at_with_relations ON language;

DROP TRIGGER IF EXISTS set_work_updated_at_with_relations ON publication;

DROP TRIGGER IF EXISTS set_work_updated_at_with_relations ON reference;

DROP TRIGGER IF EXISTS set_work_updated_at_with_relations ON subject;

DROP TRIGGER IF EXISTS set_work_updated_at_with_relations ON work_relation;

DROP TRIGGER IF EXISTS set_work_updated_at_with_relations ON affiliation;

DROP TRIGGER IF EXISTS set_work_updated_at_with_relations ON location;

DROP TRIGGER IF EXISTS set_work_updated_at_with_relations ON price;

DROP TRIGGER IF EXISTS set_work_updated_at_with_relations ON contributor;

DROP TRIGGER IF EXISTS set_work_updated_at_with_relations ON institution;

DROP TRIGGER IF EXISTS set_work_updated_at_with_relations ON publisher;

DROP TRIGGER IF EXISTS set_work_updated_at_with_relations ON series;

DROP TRIGGER IF EXISTS set_work_updated_at_with_relations ON work;

DROP TRIGGER IF EXISTS set_work_updated_at_with_relations ON imprint;

DROP FUNCTION IF EXISTS work_updated_at_with_relations();

DROP FUNCTION IF EXISTS work_relation_work_updated_at_with_relations();

DROP FUNCTION IF EXISTS affiliation_work_updated_at_with_relations();

DROP FUNCTION IF EXISTS location_work_updated_at_with_relations();

DROP FUNCTION IF EXISTS price_work_updated_at_with_relations();

DROP FUNCTION IF EXISTS contributor_work_updated_at_with_relations();

DROP FUNCTION IF EXISTS institution_work_updated_at_with_relations();

DROP FUNCTION IF EXISTS publisher_work_updated_at_with_relations();

DROP FUNCTION IF EXISTS series_work_updated_at_with_relations();

DROP FUNCTION IF EXISTS work_work_updated_at_with_relations();

DROP FUNCTION IF EXISTS imprint_work_updated_at_with_relations();

ALTER TABLE work
    DROP COLUMN updated_at_with_relations;

DROP TRIGGER IF EXISTS set_updated_at ON work;

DROP FUNCTION IF EXISTS work_set_updated_at();

SELECT diesel_manage_updated_at('work');
