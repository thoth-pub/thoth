ALTER TABLE public.publisher
    ADD COLUMN service_configuration_updated_at timestamp with time zone
        DEFAULT CURRENT_TIMESTAMP NOT NULL;

CREATE TYPE public.publisher_service_configuration_source AS ENUM (
    'SUPERUSER_API',
    'MIGRATION_BACKFILL'
);

CREATE TABLE public.publisher_service_configuration_history (
    publisher_service_configuration_history_id uuid
        DEFAULT public.uuid_generate_v4() NOT NULL,
    publisher_id uuid NOT NULL,
    actor text NOT NULL,
    source public.publisher_service_configuration_source NOT NULL,
    before_state jsonb NOT NULL,
    after_state jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT publisher_service_configuration_history_pkey
        PRIMARY KEY (publisher_service_configuration_history_id),
    CONSTRAINT publisher_service_configuration_history_publisher_id_fkey
        FOREIGN KEY (publisher_id)
        REFERENCES public.publisher(publisher_id) ON DELETE CASCADE,
    CONSTRAINT publisher_service_configuration_history_actor_check
        CHECK (btrim(actor) <> '')
);

CREATE INDEX publisher_service_configuration_history_publisher_created_idx
    ON public.publisher_service_configuration_history
    USING btree (publisher_id, created_at DESC,
                 publisher_service_configuration_history_id DESC);
