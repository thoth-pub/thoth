ALTER TABLE public.location
    DROP CONSTRAINT IF EXISTS location_checksum_and_algorithm_all_or_none,
    DROP COLUMN IF EXISTS checksum,
    DROP COLUMN IF EXISTS checksum_algorithm,

DROP TYPE IF EXISTS public.checksum_algorithm;
