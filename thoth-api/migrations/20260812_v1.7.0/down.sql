DROP TABLE IF EXISTS public.publisher_service_configuration_history;

DROP TYPE IF EXISTS public.publisher_service_configuration_source;

ALTER TABLE public.publisher
    DROP COLUMN IF EXISTS service_configuration_updated_at;
