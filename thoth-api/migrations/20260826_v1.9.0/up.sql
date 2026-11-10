-- MET-WP1-01: Metrics registry foundation (issue #836).
--
-- Additive and initially inactive. Creates the four registry enums and the
-- three metric_ registry tables, and seeds exactly the two approved
-- metric_measure rows. No metric_platform or metric_platform_measure row is
-- seeded: source/platform mappings are explicitly unapproved. No existing
-- table or row is touched.
--
-- Index decision (reviewed): the complete intended first-slice index set is
-- exactly the indexes arising from the three primary keys,
-- metric_platform(code) UNIQUE, metric_measure(code) UNIQUE and
-- metric_platform_measure(platform_id, measure_id) UNIQUE. No speculative
-- secondary index is created; later bounded query/admin slices must derive
-- indexes from their concrete access patterns.

CREATE TYPE public.metric_platform_ownership_class AS ENUM (
    'THOTH_MANAGED',
    'PUBLISHER_CONTROLLED',
    'EXTERNAL'
);

CREATE TYPE public.metric_measure_category AS ENUM (
    'USAGE',
    'SALES'
);

CREATE TYPE public.metric_measure_unit AS ENUM (
    'COUNT'
);

CREATE TYPE public.metric_reporting_grain AS ENUM (
    'DAY',
    'MONTH',
    'REPORTING_PERIOD'
);

CREATE TABLE public.metric_platform (
    platform_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    code text NOT NULL,
    display_name text NOT NULL,
    ownership_class public.metric_platform_ownership_class NOT NULL,
    enabled boolean NOT NULL,
    public_description text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT metric_platform_pkey PRIMARY KEY (platform_id),
    CONSTRAINT metric_platform_code_key UNIQUE (code),
    CONSTRAINT metric_platform_code_check CHECK (code ~ '[^[:space:]]'),
    CONSTRAINT metric_platform_display_name_check CHECK (display_name ~ '[^[:space:]]')
);

SELECT diesel_manage_updated_at('public.metric_platform');

CREATE TABLE public.metric_measure (
    measure_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    code text NOT NULL,
    display_name text NOT NULL,
    category public.metric_measure_category NOT NULL,
    unit public.metric_measure_unit NOT NULL,
    allow_negative boolean NOT NULL,
    public_visibility boolean DEFAULT TRUE NOT NULL,
    additive_across_time boolean NOT NULL,
    additive_across_works boolean NOT NULL,
    definition text NOT NULL,
    methodology_version text,
    enabled boolean NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT metric_measure_pkey PRIMARY KEY (measure_id),
    CONSTRAINT metric_measure_code_key UNIQUE (code),
    CONSTRAINT metric_measure_code_check CHECK (code ~ '[^[:space:]]'),
    CONSTRAINT metric_measure_display_name_check CHECK (display_name ~ '[^[:space:]]'),
    CONSTRAINT metric_measure_definition_check CHECK (definition ~ '[^[:space:]]')
);

SELECT diesel_manage_updated_at('public.metric_measure');

-- metric_platform_measure deliberately has no created_at or updated_at
-- columns: the approved Metrics design (§6.3) omits them, and future
-- protected registry-administration/audit work must separately decide whether
-- mutation history is required before exposing a write path.
--
-- The registry foreign keys are deliberately non-cascading: deleting a
-- platform or measure that still has a mapping fails instead of silently
-- deleting the mapping.
--
-- supported_grains enforces, at the database boundary: cardinality greater
-- than zero, no NULL element, and no duplicate reporting grain. Duplicate
-- rejection uses explicit occurrence counts of the three closed enum values.
CREATE TABLE public.metric_platform_measure (
    platform_measure_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    platform_id uuid NOT NULL,
    measure_id uuid NOT NULL,
    supported_grains public.metric_reporting_grain[] NOT NULL,
    supports_country boolean NOT NULL,
    supports_institution boolean NOT NULL,
    supports_publication boolean NOT NULL,
    direct_collection boolean NOT NULL,
    enabled boolean NOT NULL,
    CONSTRAINT metric_platform_measure_pkey PRIMARY KEY (platform_measure_id),
    CONSTRAINT metric_platform_measure_platform_id_measure_id_key
        UNIQUE (platform_id, measure_id),
    CONSTRAINT metric_platform_measure_platform_id_fkey FOREIGN KEY (platform_id)
        REFERENCES public.metric_platform(platform_id),
    CONSTRAINT metric_platform_measure_measure_id_fkey FOREIGN KEY (measure_id)
        REFERENCES public.metric_measure(measure_id),
    CONSTRAINT metric_platform_measure_supported_grains_check CHECK (
        cardinality(supported_grains) > 0
        AND array_position(supported_grains, NULL) IS NULL
        AND cardinality(array_positions(
            supported_grains, 'DAY'::public.metric_reporting_grain)) <= 1
        AND cardinality(array_positions(
            supported_grains, 'MONTH'::public.metric_reporting_grain)) <= 1
        AND cardinality(array_positions(
            supported_grains, 'REPORTING_PERIOD'::public.metric_reporting_grain)) <= 1
    )
);

-- Seed exactly the two approved initial measures. Insertion is deliberately
-- unconditional: on the exact clean baseline no conflicting registry row can
-- exist, and an unexpected conflict must fail the migration and surface drift
-- rather than be concealed by ON CONFLICT DO NOTHING.
INSERT INTO public.metric_measure (
    code,
    display_name,
    category,
    unit,
    allow_negative,
    public_visibility,
    additive_across_time,
    additive_across_works,
    definition,
    methodology_version,
    enabled
) VALUES (
    'title_sessions',
    'Title sessions',
    'USAGE',
    'COUNT',
    FALSE,
    TRUE,
    TRUE,
    TRUE,
    'Count of title sessions: one or more successful qualifying requests for the same work by the same transient user during a rolling 30-minute session, attributed to the UTC date on which the session began and counted once per DOI and country within that session.',
    'cloudfront-title-session/2',
    TRUE
);

INSERT INTO public.metric_measure (
    code,
    display_name,
    category,
    unit,
    allow_negative,
    public_visibility,
    additive_across_time,
    additive_across_works,
    definition,
    methodology_version,
    enabled
) VALUES (
    'net_units',
    'Net units',
    'SALES',
    'COUNT',
    TRUE,
    TRUE,
    TRUE,
    TRUE,
    'Signed net sales units for a work over the reported period; positive values represent net units sold and negative values represent refunds or returns as reported by the source.',
    NULL,
    TRUE
);
