CREATE TYPE public.distribution_platform AS ENUM (
    'INTERNET_ARCHIVE',
    'OAPEN',
    'DOAB',
    'SCIENCE_OPEN',
    'CAMBRIDGE_UNIVERSITY_LIBRARY',
    'CROSSREF',
    'FIGSHARE',
    'ZENODO',
    'PROJECT_MUSE',
    'JSTOR',
    'EBSCO_HOST',
    'PROQUEST_EBOOK_CENTRAL',
    'GOOGLE_PLAY',
    'BKCI',
    'OCLC_KB',
    'EX_LIBRIS_KB',
    'JISC_NBK'
);

CREATE TABLE public.publisher_distribution_platform (
    publisher_id uuid NOT NULL,
    platform public.distribution_platform NOT NULL,
    enabled boolean NOT NULL,
    activation_id uuid NOT NULL,
    enabled_at timestamp with time zone NOT NULL,
    disabled_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT publisher_distribution_platform_pkey PRIMARY KEY (publisher_id, platform),
    CONSTRAINT publisher_distribution_platform_publisher_id_fkey FOREIGN KEY (publisher_id)
        REFERENCES public.publisher(publisher_id) ON DELETE CASCADE,
    CONSTRAINT publisher_distribution_platform_enabled_state_check CHECK (
        (enabled AND disabled_at IS NULL)
        OR
        (NOT enabled AND disabled_at IS NOT NULL)
    )
);

CREATE INDEX publisher_distribution_platform_enabled_idx
    ON public.publisher_distribution_platform
    USING btree (platform, publisher_id)
    WHERE enabled;

SELECT diesel_manage_updated_at('public.publisher_distribution_platform');
