-- MET-WP1-06 downgrade: remove only the schema introduced by this migration.
--
-- metric_publisher_platform_approval holds no dependants, so it is dropped
-- first, then its status enum. The MET-WP1-01 registry schema and canonical
-- `publisher` schema are untouched, as is the
-- metric_publisher_platform_approval primary-key index and its
-- `(publisher_id, platform_id)` uniqueness index, both dropped implicitly
-- with the table.

DROP TABLE IF EXISTS public.metric_publisher_platform_approval;

DROP TYPE IF EXISTS public.metric_publisher_platform_approval_status;
