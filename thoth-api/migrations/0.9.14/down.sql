DROP TRIGGER IF EXISTS set_work_table_relation_updated_at ON contribution;

DROP TRIGGER IF EXISTS set_work_table_relation_updated_at ON funding;

DROP TRIGGER IF EXISTS set_work_table_relation_updated_at ON issue;

DROP TRIGGER IF EXISTS set_work_table_relation_updated_at ON language;

DROP TRIGGER IF EXISTS set_work_table_relation_updated_at ON publication;

DROP TRIGGER IF EXISTS set_work_table_relation_updated_at ON reference;

DROP TRIGGER IF EXISTS set_work_table_relation_updated_at ON subject;

DROP TRIGGER IF EXISTS set_work_table_relation_updated_at ON work_relation;

DROP TRIGGER IF EXISTS set_work_table_relation_updated_at ON affiliation;

DROP TRIGGER IF EXISTS set_work_table_relation_updated_at ON location;

DROP TRIGGER IF EXISTS set_work_table_relation_updated_at ON price;

DROP TRIGGER IF EXISTS set_work_table_relation_updated_at ON contributor;

DROP TRIGGER IF EXISTS set_work_table_relation_updated_at ON institution;

DROP TRIGGER IF EXISTS set_work_table_relation_updated_at ON publisher;

DROP TRIGGER IF EXISTS set_work_table_relation_updated_at ON series;

DROP TRIGGER IF EXISTS set_work_table_relation_updated_at ON work;

DROP FUNCTION IF EXISTS work_table_relation_updated_at();

DROP FUNCTION IF EXISTS work_relation_work_table_relation_updated_at();

DROP FUNCTION IF EXISTS affiliation_work_table_relation_updated_at();

DROP FUNCTION IF EXISTS location_work_table_relation_updated_at();

DROP FUNCTION IF EXISTS price_work_table_relation_updated_at();

DROP FUNCTION IF EXISTS contributor_work_table_relation_updated_at();

DROP FUNCTION IF EXISTS institution_work_table_relation_updated_at();

DROP FUNCTION IF EXISTS publisher_work_table_relation_updated_at();

DROP FUNCTION IF EXISTS series_work_table_relation_updated_at();

DROP FUNCTION IF EXISTS work_work_table_relation_updated_at();

ALTER TABLE work
    DROP COLUMN relation_updated_at;

DROP TRIGGER IF EXISTS set_updated_at ON work;

DROP FUNCTION IF EXISTS work_set_updated_at();

SELECT diesel_manage_updated_at('work');
