ALTER TABLE public.publisher
    DROP COLUMN IF EXISTS subscription_package;

DROP TYPE IF EXISTS public.thoth_package;
