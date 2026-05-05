CREATE TYPE public.checksum_algorithm AS ENUM (
    'MD5',
    'SHA256',
    'SHA1'
);

ALTER TABLE public.location
    ADD COLUMN checksum TEXT,
    ADD COLUMN checksum_algorithm public.checksum_algorithm,
    ADD CONSTRAINT location_checksum_and_algorithm_all_or_none CHECK ((checksum IS NULL AND checksum_algorithm IS NULL) OR (checksum IS NOT NULL AND checksum_algorithm IS NOT NULL));
