DROP TABLE IF EXISTS funding;
DROP TABLE IF EXISTS funder;

DROP TABLE IF EXISTS subject;
DROP TYPE IF EXISTS subject_type;

DROP TABLE IF EXISTS price;
DROP TYPE IF EXISTS currency_code;

DROP TABLE IF EXISTS publication;
DROP TYPE IF EXISTS publication_type;

DROP TABLE IF EXISTS contribution;
DROP TYPE IF EXISTS contribution_type;
DROP TABLE IF EXISTS contributor;

DROP TABLE IF EXISTS issue;
DROP TABLE IF EXISTS series;
DROP TYPE IF EXISTS series_type;

DROP TABLE IF EXISTS language;
DROP TYPE IF EXISTS language_code;
DROP TYPE IF EXISTS language_relation;

DROP TABLE IF EXISTS work;
DROP TYPE IF EXISTS work_type;
DROP TYPE IF EXISTS work_status;

DROP TABLE IF EXISTS imprint;
DROP TABLE IF EXISTS publisher;

DROP EXTENSION IF EXISTS "uuid-ossp";
