-- MET-WP1-01 downgrade: remove only the schema and data introduced by this
-- migration, dropping the three registry tables in dependency-safe order and
-- then the four registry enum types.

DROP TABLE IF EXISTS public.metric_platform_measure;

DROP TABLE IF EXISTS public.metric_measure;

DROP TABLE IF EXISTS public.metric_platform;

DROP TYPE IF EXISTS public.metric_reporting_grain;

DROP TYPE IF EXISTS public.metric_measure_unit;

DROP TYPE IF EXISTS public.metric_measure_category;

DROP TYPE IF EXISTS public.metric_platform_ownership_class;
