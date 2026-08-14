-- BE-04: durable publisher back-catalogue distribution jobs.
--
-- Additive only. No existing table, column, constraint, index, trigger or enum
-- is altered or dropped, and the migration creates zero rows. Establishing the
-- two foreign keys below does take a SHARE ROW EXCLUSIVE lock on the existing
-- populated `public.publisher` and `public.work` tables for the duration of the
-- migration transaction; that lock is measured rather than assumed, and the
-- foreign keys are deliberately not weakened, deferred or made NOT VALID to
-- shorten it.

CREATE TYPE public.distribution_job_kind AS ENUM (
    'PUBLISHER_BACK_CATALOGUE'
);

CREATE TYPE public.distribution_job_status AS ENUM (
    'PENDING',
    'RUNNING',
    'SUCCEEDED',
    'FAILED',
    'CANCELLED'
);

CREATE TYPE public.distribution_job_attempt_result AS ENUM (
    'SUCCEEDED',
    'FAILED',
    'CANCELLED',
    'ABANDONED'
);

CREATE TYPE public.distribution_job_cancellation_reason AS ENUM (
    'ADMINISTRATIVE',
    'ASSIGNMENT_DISABLED'
);

CREATE TABLE public.distribution_job (
    distribution_job_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    kind public.distribution_job_kind NOT NULL,
    publisher_id uuid NOT NULL,
    work_id uuid,
    activation_id uuid NOT NULL,
    status public.distribution_job_status DEFAULT 'PENDING' NOT NULL,
    deduplication_key text NOT NULL,
    attempt_count integer DEFAULT 0 NOT NULL,
    available_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    claim_token uuid,
    claimed_by text,
    claimed_at timestamp with time zone,
    lease_expires_at timestamp with time zone,
    completed_at timestamp with time zone,
    cancellation_reason public.distribution_job_cancellation_reason,
    last_error_code text,
    last_error_detail text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,

    CONSTRAINT distribution_job_pkey
        PRIMARY KEY (distribution_job_id),

    CONSTRAINT distribution_job_publisher_id_fkey
        FOREIGN KEY (publisher_id)
        REFERENCES public.publisher(publisher_id) ON DELETE CASCADE,

    CONSTRAINT distribution_job_work_id_fkey
        FOREIGN KEY (work_id)
        REFERENCES public.work(work_id) ON DELETE CASCADE,

    CONSTRAINT distribution_job_deduplication_key_key
        UNIQUE (deduplication_key),

    CONSTRAINT distribution_job_deduplication_key_formula_check CHECK (
        kind <> 'PUBLISHER_BACK_CATALOGUE'
        OR deduplication_key =
            'PUBLISHER_BACK_CATALOGUE:' || publisher_id::text
                                        || ':' || activation_id::text
    ),

    CONSTRAINT distribution_job_deduplication_key_length_check
        CHECK (char_length(deduplication_key) BETWEEN 1 AND 256),

    CONSTRAINT distribution_job_back_catalogue_work_check
        CHECK (kind <> 'PUBLISHER_BACK_CATALOGUE' OR work_id IS NULL),

    CONSTRAINT distribution_job_attempt_count_check
        CHECK (attempt_count >= 0 AND attempt_count <= 5),

    CONSTRAINT distribution_job_claim_state_check CHECK (
        (status = 'RUNNING'
            AND claim_token IS NOT NULL
            AND claimed_by IS NOT NULL
            AND claimed_at IS NOT NULL
            AND lease_expires_at IS NOT NULL)
        OR
        (status <> 'RUNNING'
            AND claim_token IS NULL
            AND claimed_by IS NULL
            AND claimed_at IS NULL
            AND lease_expires_at IS NULL)
    ),

    CONSTRAINT distribution_job_claimed_by_check
        CHECK (claimed_by IS NULL OR claimed_by ~ '[^[:space:]]'),

    CONSTRAINT distribution_job_completed_at_check CHECK (
        (status IN ('SUCCEEDED', 'FAILED', 'CANCELLED')
            AND completed_at IS NOT NULL)
        OR
        (status IN ('PENDING', 'RUNNING') AND completed_at IS NULL)
    ),

    CONSTRAINT distribution_job_cancellation_reason_check CHECK (
        (status = 'CANCELLED' AND cancellation_reason IS NOT NULL)
        OR
        (status <> 'CANCELLED' AND cancellation_reason IS NULL)
    ),

    CONSTRAINT distribution_job_last_error_check
        CHECK (last_error_detail IS NULL OR last_error_code IS NOT NULL),

    CONSTRAINT distribution_job_last_error_code_format_check CHECK (
        last_error_code IS NULL
        OR (last_error_code ~ '^[A-Z][A-Z0-9_]*$'
            AND char_length(last_error_code) <= 64)
    ),

    CONSTRAINT distribution_job_last_error_detail_length_check CHECK (
        last_error_detail IS NULL OR char_length(last_error_detail) <= 2048
    )
);

SELECT diesel_manage_updated_at('public.distribution_job');

CREATE TABLE public.distribution_job_target (
    distribution_job_id uuid NOT NULL,
    platform public.distribution_platform NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,

    CONSTRAINT distribution_job_target_pkey
        PRIMARY KEY (distribution_job_id, platform),

    CONSTRAINT distribution_job_target_distribution_job_id_fkey
        FOREIGN KEY (distribution_job_id)
        REFERENCES public.distribution_job(distribution_job_id) ON DELETE CASCADE
);

CREATE TABLE public.distribution_job_attempt (
    distribution_job_attempt_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    distribution_job_id uuid NOT NULL,
    attempt_number integer NOT NULL,
    claim_token uuid NOT NULL,
    claimed_by text NOT NULL,
    started_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    finished_at timestamp with time zone,
    result public.distribution_job_attempt_result,
    error_code text,
    error_detail text,

    CONSTRAINT distribution_job_attempt_pkey
        PRIMARY KEY (distribution_job_attempt_id),

    CONSTRAINT distribution_job_attempt_distribution_job_id_fkey
        FOREIGN KEY (distribution_job_id)
        REFERENCES public.distribution_job(distribution_job_id) ON DELETE CASCADE,

    CONSTRAINT distribution_job_attempt_number_key
        UNIQUE (distribution_job_id, attempt_number),

    CONSTRAINT distribution_job_attempt_claim_token_key
        UNIQUE (claim_token),

    CONSTRAINT distribution_job_attempt_number_check
        CHECK (attempt_number >= 1),

    CONSTRAINT distribution_job_attempt_claimed_by_check
        CHECK (claimed_by ~ '[^[:space:]]'),

    CONSTRAINT distribution_job_attempt_closure_check CHECK (
        (finished_at IS NULL AND result IS NULL)
        OR
        (finished_at IS NOT NULL AND result IS NOT NULL)
    ),

    CONSTRAINT distribution_job_attempt_interval_check
        CHECK (finished_at IS NULL OR finished_at >= started_at),

    CONSTRAINT distribution_job_attempt_error_result_check CHECK (
        (error_code IS NULL AND error_detail IS NULL)
        OR result = 'FAILED'
    ),

    CONSTRAINT distribution_job_attempt_error_pairing_check
        CHECK (error_detail IS NULL OR error_code IS NOT NULL),

    CONSTRAINT distribution_job_attempt_error_code_format_check CHECK (
        error_code IS NULL
        OR (error_code ~ '^[A-Z][A-Z0-9_]*$' AND char_length(error_code) <= 64)
    ),

    CONSTRAINT distribution_job_attempt_error_detail_length_check CHECK (
        error_detail IS NULL OR char_length(error_detail) <= 2048
    )
);

CREATE INDEX distribution_job_claimable_idx
    ON public.distribution_job
    USING btree (available_at, distribution_job_id)
    WHERE status = 'PENDING';

CREATE INDEX distribution_job_lease_idx
    ON public.distribution_job
    USING btree (lease_expires_at, distribution_job_id)
    WHERE status = 'RUNNING';

CREATE INDEX distribution_job_publisher_latest_idx
    ON public.distribution_job
    USING btree (publisher_id, kind, created_at DESC, distribution_job_id DESC);
