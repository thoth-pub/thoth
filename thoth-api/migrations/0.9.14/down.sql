DROP TRIGGER set_work_table_relation_updated_at ON contribution;

DROP TRIGGER set_work_table_relation_updated_at ON funding;

DROP TRIGGER set_work_table_relation_updated_at ON issue;

DROP TRIGGER set_work_table_relation_updated_at ON language;

DROP TRIGGER set_work_table_relation_updated_at ON publication;

DROP TRIGGER set_work_table_relation_updated_at ON reference;

DROP TRIGGER set_work_table_relation_updated_at ON subject;

DROP TRIGGER set_work_table_relation_updated_at ON work_relation;

DROP TRIGGER set_work_table_relation_updated_at ON affiliation;

DROP TRIGGER set_work_table_relation_updated_at ON location;

DROP TRIGGER set_work_table_relation_updated_at ON price;

DROP TRIGGER set_work_table_relation_updated_at ON contributor;

DROP TRIGGER set_work_table_relation_updated_at ON institution;

DROP TRIGGER set_work_table_relation_updated_at ON publisher;

DROP TRIGGER set_work_table_relation_updated_at ON series;

DROP FUNCTION IF EXISTS work_table_relation_updated_at();

DROP FUNCTION IF EXISTS work_relation_work_table_relation_updated_at();

DROP FUNCTION IF EXISTS affiliation_work_table_relation_updated_at();

DROP FUNCTION IF EXISTS location_work_table_relation_updated_at();

DROP FUNCTION IF EXISTS price_work_table_relation_updated_at();

DROP FUNCTION IF EXISTS contributor_work_table_relation_updated_at();

DROP FUNCTION IF EXISTS institution_work_table_relation_updated_at();

DROP FUNCTION IF EXISTS publisher_work_table_relation_updated_at();

DROP FUNCTION IF EXISTS series_work_table_relation_updated_at();

ALTER TABLE work
    DROP COLUMN relation_updated_at;

DROP TRIGGER IF EXISTS set_updated_at ON work;

DROP FUNCTION IF EXISTS work_set_updated_at();

SELECT diesel_manage_updated_at('work');
