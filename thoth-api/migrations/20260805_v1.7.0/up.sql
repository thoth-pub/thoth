CREATE TYPE public.thoth_package AS ENUM (
    'OASIS',
    'OBELISK',
    'SPHINX',
    'PYRAMID'
);

ALTER TABLE public.publisher
    ADD COLUMN subscription_package public.thoth_package DEFAULT 'OASIS'::public.thoth_package NOT NULL;
