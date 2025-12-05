-- Drop tables
DROP TABLE IF EXISTS public.work_relation_history CASCADE;
DROP TABLE IF EXISTS public.work_relation CASCADE;
DROP TABLE IF EXISTS public.work_history CASCADE;
DROP TABLE IF EXISTS public.work CASCADE;
DROP TABLE IF EXISTS public.subject_history CASCADE;
DROP TABLE IF EXISTS public.subject CASCADE;
DROP TABLE IF EXISTS public.series_history CASCADE;
DROP TABLE IF EXISTS public.series CASCADE;
DROP TABLE IF EXISTS public.reference_history CASCADE;
DROP TABLE IF EXISTS public.reference CASCADE;
DROP TABLE IF EXISTS public.publisher_history CASCADE;
DROP TABLE IF EXISTS public.publisher_account CASCADE;
DROP TABLE IF EXISTS public.publisher CASCADE;
DROP TABLE IF EXISTS public.publication_history CASCADE;
DROP TABLE IF EXISTS public.publication CASCADE;
DROP TABLE IF EXISTS public.price_history CASCADE;
DROP TABLE IF EXISTS public.price CASCADE;
DROP TABLE IF EXISTS public.location_history CASCADE;
DROP TABLE IF EXISTS public.location CASCADE;
DROP TABLE IF EXISTS public.language_history CASCADE;
DROP TABLE IF EXISTS public.language CASCADE;
DROP TABLE IF EXISTS public.issue_history CASCADE;
DROP TABLE IF EXISTS public.issue CASCADE;
DROP TABLE IF EXISTS public.institution_history CASCADE;
DROP TABLE IF EXISTS public.institution CASCADE;
DROP TABLE IF EXISTS public.imprint_history CASCADE;
DROP TABLE IF EXISTS public.imprint CASCADE;
DROP TABLE IF EXISTS public.funding_history CASCADE;
DROP TABLE IF EXISTS public.funding CASCADE;
DROP TABLE IF EXISTS public.contributor_history CASCADE;
DROP TABLE IF EXISTS public.contributor CASCADE;
DROP TABLE IF EXISTS public.contribution_history CASCADE;
DROP TABLE IF EXISTS public.contribution CASCADE;
DROP TABLE IF EXISTS public.affiliation_history CASCADE;
DROP TABLE IF EXISTS public.affiliation CASCADE;
DROP TABLE IF EXISTS public.account CASCADE;
DROP TABLE IF EXISTS public.__diesel_schema_migrations CASCADE;

-- Drop functions
DROP FUNCTION IF EXISTS public.affiliation_work_updated_at_with_relations() CASCADE;
DROP FUNCTION IF EXISTS public.contributor_work_updated_at_with_relations() CASCADE;
DROP FUNCTION IF EXISTS public.imprint_work_updated_at_with_relations() CASCADE;
DROP FUNCTION IF EXISTS public.institution_work_updated_at_with_relations() CASCADE;
DROP FUNCTION IF EXISTS public.location_work_updated_at_with_relations() CASCADE;
DROP FUNCTION IF EXISTS public.price_work_updated_at_with_relations() CASCADE;
DROP FUNCTION IF EXISTS public.publisher_work_updated_at_with_relations() CASCADE;
DROP FUNCTION IF EXISTS public.series_work_updated_at_with_relations() CASCADE;
DROP FUNCTION IF EXISTS public.work_relation_work_updated_at_with_relations() CASCADE;
DROP FUNCTION IF EXISTS public.work_work_updated_at_with_relations() CASCADE;
DROP FUNCTION IF EXISTS public.work_updated_at_with_relations() CASCADE;
DROP FUNCTION IF EXISTS public.work_set_updated_at() CASCADE;
DROP FUNCTION IF EXISTS public.publication_chapter_no_dimensions() CASCADE;
DROP FUNCTION IF EXISTS public.publication_location_canonical_urls() CASCADE;
DROP FUNCTION IF EXISTS public.diesel_set_updated_at() CASCADE;
DROP FUNCTION IF EXISTS public.diesel_manage_updated_at(regclass) CASCADE;

-- Drop enum types
DROP TYPE IF EXISTS public.work_type;
DROP TYPE IF EXISTS public.work_status;
DROP TYPE IF EXISTS public.subject_type;
DROP TYPE IF EXISTS public.series_type;
DROP TYPE IF EXISTS public.relation_type;
DROP TYPE IF EXISTS public.publication_type;
DROP TYPE IF EXISTS public.location_platform;
DROP TYPE IF EXISTS public.language_relation;
DROP TYPE IF EXISTS public.language_code;
DROP TYPE IF EXISTS public.currency_code;
DROP TYPE IF EXISTS public.country_code;
DROP TYPE IF EXISTS public.contribution_type;

-- Drop extension
DROP EXTENSION IF EXISTS "uuid-ossp" CASCADE;
