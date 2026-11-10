-- MET-WP1-02: Metrics source-state foundation (issue #841).
--
-- Additive and initially inactive. Creates the closed acquisition enum and
-- the three durable source-state tables: metric_source (acquisition-route
-- identity), metric_source_account (concrete source partition/account routed
-- to a metric platform) and metric_source_checkpoint (durable partition
-- checkpoint, progress and lease storage). No row is seeded: source and
-- platform mappings remain explicitly unapproved. No existing table or row
-- is touched, and no lease/claim behaviour is implemented — only the durable
-- columns later bounded checkpoint/claim work will use.
--
-- Index decision (reviewed): the complete intended index set is exactly the
-- indexes arising from the three primary keys, metric_source(code) UNIQUE,
-- metric_source_account(source_id, external_key) UNIQUE,
-- metric_source_checkpoint(source_account_id, partition_key) UNIQUE, plus the
-- single design-required operational index on
-- metric_source_checkpoint(lease_expires_at). No other speculative secondary
-- index is created; later bounded slices must derive indexes from their
-- concrete access patterns.

CREATE TYPE public.metric_source_acquisition_type AS ENUM (
    'DRIVER',
    'PUBLISHER_UPLOAD',
    'OPERAS',
    'ADMIN_IMPORT'
);

-- metric_source deliberately has no created_at or updated_at columns: the
-- approved Metrics design omits them, and future protected
-- source-administration/audit work must separately decide whether mutation
-- history is required before exposing a write path.
--
-- driver_key stays plain nullable text: this slice assigns it no uniqueness,
-- no DRIVER-specific cross-field constraint and no driver-registry semantics.
--
-- The non-negative day checks are the approved bounded decision that negative
-- lookback/finalization delays have no operational meaning and must fail at
-- the canonical database boundary; NULL (source default unset) remains valid.
CREATE TABLE public.metric_source (
    source_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    code text NOT NULL,
    acquisition_type public.metric_source_acquisition_type NOT NULL,
    driver_key text,
    enabled boolean NOT NULL,
    default_lookback_days integer,
    default_finalization_delay_days integer,
    CONSTRAINT metric_source_pkey PRIMARY KEY (source_id),
    CONSTRAINT metric_source_code_key UNIQUE (code),
    CONSTRAINT metric_source_code_check CHECK (code ~ '[^[:space:]]'),
    CONSTRAINT metric_source_default_lookback_days_check
        CHECK (default_lookback_days >= 0),
    CONSTRAINT metric_source_default_finalization_delay_days_check
        CHECK (default_finalization_delay_days >= 0)
);

-- metric_source_account deliberately has no created_at or updated_at columns
-- for the same design reason as metric_source.
--
-- configuration is generic non-secret routing/configuration JSON only.
-- Credentials must never be stored here; because this slice ships no
-- application write path, allowed-field validation belongs to the later
-- protected source-account administration specification, and no heuristic
-- SQL secret detection is invented at the database boundary.
--
-- The foreign keys are deliberately non-cascading: deleting a referenced
-- source, platform or publisher that still has a source account fails instead
-- of silently deleting the account.
CREATE TABLE public.metric_source_account (
    source_account_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    source_id uuid NOT NULL,
    platform_id uuid NOT NULL,
    external_key text NOT NULL,
    expected_publisher_id uuid,
    configuration jsonb DEFAULT '{}'::jsonb NOT NULL,
    enabled boolean NOT NULL,
    CONSTRAINT metric_source_account_pkey PRIMARY KEY (source_account_id),
    CONSTRAINT metric_source_account_source_id_external_key_key
        UNIQUE (source_id, external_key),
    CONSTRAINT metric_source_account_external_key_check
        CHECK (external_key ~ '[^[:space:]]'),
    CONSTRAINT metric_source_account_source_id_fkey FOREIGN KEY (source_id)
        REFERENCES public.metric_source(source_id),
    CONSTRAINT metric_source_account_platform_id_fkey FOREIGN KEY (platform_id)
        REFERENCES public.metric_platform(platform_id),
    CONSTRAINT metric_source_account_expected_publisher_id_fkey
        FOREIGN KEY (expected_publisher_id)
        REFERENCES public.publisher(publisher_id)
);

-- metric_source_checkpoint stores the durable checkpoint/progress/lease
-- columns only. The operation-level concurrency protocol (claim tokens, lease
-- acquisition/release, FOR UPDATE SKIP LOCKED, stale-lease recovery, retries)
-- is deliberately absent: it belongs to the later bounded internal
-- claim/checkpoint API task. cursor stays generic nullable JSONB because its
-- content is source-specific. The approved design specifies no created_at
-- column for checkpoints.
CREATE TABLE public.metric_source_checkpoint (
    source_checkpoint_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    source_account_id uuid NOT NULL,
    partition_key text NOT NULL,
    cursor jsonb,
    last_discovered_at timestamp with time zone,
    last_completed_at timestamp with time zone,
    last_successful_period_end date,
    lease_owner text,
    lease_expires_at timestamp with time zone,
    last_error text,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT metric_source_checkpoint_pkey PRIMARY KEY (source_checkpoint_id),
    CONSTRAINT metric_source_checkpoint_source_account_id_partition_key_key
        UNIQUE (source_account_id, partition_key),
    CONSTRAINT metric_source_checkpoint_partition_key_check
        CHECK (partition_key ~ '[^[:space:]]'),
    CONSTRAINT metric_source_checkpoint_source_account_id_fkey
        FOREIGN KEY (source_account_id)
        REFERENCES public.metric_source_account(source_account_id)
);

SELECT diesel_manage_updated_at('public.metric_source_checkpoint');

-- The single design-required operational index: later lease housekeeping
-- scans by lease expiry.
CREATE INDEX metric_source_checkpoint_lease_expires_at_idx
    ON public.metric_source_checkpoint (lease_expires_at);
