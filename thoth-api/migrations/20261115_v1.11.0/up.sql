-- THOTH-ASYNC-01-ENGINE-01A (E1): the shared job engine's durable storage
-- (issue #1026, programme #957, ADR-0012 section 8 item 1, Release N).
--
-- Additive only. Creates nine enum types, nine tables, their checks, indexes
-- and triggers, and nothing else: no existing table, column, constraint,
-- index, trigger, function or enum is altered, referenced, locked or dropped,
-- and the migration creates zero rows. Release N ships inert: no production
-- path writes any `async_*` row.
--
-- Rollback: `down.sql` is for disposable and development databases only.
-- Once production holds any `async_*` row, schema rollback is a forward-repair
-- decision (ADR-0012 section 10), never a routine down migration.
--
-- Every trigger and function name below avoids `reconciliation`, `coverage`
-- and a leading `maintain`, which existing Metrics catalog guards reserve.

CREATE TYPE public.async_job_status AS ENUM (
    'PENDING', 'RUNNING', 'WAITING', 'RETRY_SCHEDULED',
    'SUCCEEDED', 'FAILED', 'CANCELLED', 'RECONCILIATION_REQUIRED'
);

CREATE TYPE public.async_effect_phase AS ENUM (
    'PRE_WRITE', 'EFFECT_STARTED', 'EFFECT_CONFIRMED'
);

CREATE TYPE public.async_job_effect_class AS ENUM ('READ_ONLY', 'EFFECTFUL');

CREATE TYPE public.async_job_revision_mode AS ENUM ('CURRENT_STATE', 'REVISION_BOUND');

CREATE TYPE public.async_job_lineage AS ENUM ('ORIGINAL', 'MANUAL_RETRY');

CREATE TYPE public.async_attempt_purpose AS ENUM (
    'EXECUTION', 'RECONCILIATION', 'STAFF_RECONCILIATION'
);

CREATE TYPE public.async_attempt_result AS ENUM (
    'SUCCEEDED', 'RETRYABLE_FAILURE', 'PERMANENT_FAILURE', 'INDETERMINATE',
    'WAITING', 'RELEASED', 'LEASE_EXPIRED', 'CANCELLED', 'STILL_REQUIRED'
);

CREATE TYPE public.async_control_command_kind AS ENUM (
    'CANCEL_JOB', 'RETRY_JOB', 'RESOLVE_RECONCILIATION', 'REDACT_JOB_DIAGNOSTICS'
);

CREATE TYPE public.async_control_command_outcome AS ENUM ('APPLIED', 'REFUSED');

-- ---------------------------------------------------------------------------
-- async_job (spec section 7.2)
-- ---------------------------------------------------------------------------

CREATE TABLE public.async_job (
    async_job_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    kind text NOT NULL,
    payload_version smallint NOT NULL,
    payload jsonb NOT NULL,
    idempotency_key text NOT NULL,
    effect_class public.async_job_effect_class NOT NULL,
    revision_mode public.async_job_revision_mode NOT NULL,
    retry_budget smallint NOT NULL,
    max_execution_claims smallint NOT NULL,
    max_reconciliation_attempts smallint NOT NULL,
    backoff_base_seconds integer NOT NULL,
    backoff_max_seconds integer NOT NULL,
    source_order bigint,
    source_fingerprint text,
    source_ref text,
    status public.async_job_status DEFAULT 'PENDING' NOT NULL,
    priority smallint DEFAULT 0 NOT NULL,
    available_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    effect_phase public.async_effect_phase DEFAULT 'PRE_WRITE' NOT NULL,
    attempt_count integer DEFAULT 0 NOT NULL,
    execution_claim_count smallint DEFAULT 0 NOT NULL,
    prewrite_failure_count smallint DEFAULT 0 NOT NULL,
    reconciliation_attempt_count smallint DEFAULT 0 NOT NULL,
    claim_token uuid,
    claimed_by text,
    executor_principal text,
    claimed_at timestamp with time zone,
    lease_expires_at timestamp with time zone,
    attempt_runtime_deadline timestamp with time zone,
    first_claimed_at timestamp with time zone,
    waiting_deadline timestamp with time zone,
    checkpoint jsonb,
    checkpoint_version integer DEFAULT 0 NOT NULL,
    provider_ref text,
    terminal_at timestamp with time zone,
    last_outcome_code text,
    redacted_at timestamp with time zone,
    lineage public.async_job_lineage DEFAULT 'ORIGINAL' NOT NULL,
    parent_job_id uuid,
    correlation_id uuid,
    causation_ref text,
    created_by text NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,

    CONSTRAINT async_job_pkey PRIMARY KEY (async_job_id),

    CONSTRAINT async_job_idempotency_key_key UNIQUE (idempotency_key),

    CONSTRAINT async_job_parent_job_id_fkey
        FOREIGN KEY (parent_job_id)
        REFERENCES public.async_job(async_job_id) ON DELETE RESTRICT,

    CONSTRAINT async_job_kind_check
        CHECK (kind ~ '^[A-Z][A-Z0-9_]{2,63}$'),

    CONSTRAINT async_job_payload_version_check
        CHECK (payload_version >= 1),

    CONSTRAINT async_job_payload_check CHECK (
        jsonb_typeof(payload) = 'object'
        AND octet_length(payload::text) <= 16384
    ),

    CONSTRAINT async_job_idempotency_key_check CHECK (
        starts_with(idempotency_key, kind || ':')
        AND char_length(idempotency_key) <= 512
    ),

    CONSTRAINT async_job_retry_budget_check
        CHECK (retry_budget BETWEEN 1 AND 20),

    CONSTRAINT async_job_max_execution_claims_check
        CHECK (max_execution_claims BETWEEN retry_budget AND 100),

    CONSTRAINT async_job_max_reconciliation_attempts_check
        CHECK (max_reconciliation_attempts BETWEEN 1 AND 50),

    CONSTRAINT async_job_backoff_base_seconds_check
        CHECK (backoff_base_seconds >= 1),

    CONSTRAINT async_job_backoff_max_seconds_check
        CHECK (backoff_max_seconds BETWEEN backoff_base_seconds AND 86400),

    CONSTRAINT async_job_source_fingerprint_check CHECK (
        source_fingerprint IS NULL
        OR (source_fingerprint ~ '^[!-~]+$' AND char_length(source_fingerprint) <= 256)
    ),

    CONSTRAINT async_job_source_ref_check CHECK (
        source_ref IS NULL
        OR (source_ref ~ '^[!-~]+$' AND char_length(source_ref) <= 256)
    ),

    CONSTRAINT async_job_priority_check
        CHECK (priority BETWEEN -100 AND 100),

    CONSTRAINT async_job_attempt_count_check
        CHECK (attempt_count >= 0),

    CONSTRAINT async_job_execution_claim_count_check
        CHECK (execution_claim_count BETWEEN 0 AND max_execution_claims),

    CONSTRAINT async_job_prewrite_failure_count_check
        CHECK (prewrite_failure_count BETWEEN 0 AND retry_budget),

    CONSTRAINT async_job_recon_attempt_count_check
        CHECK (reconciliation_attempt_count BETWEEN 0 AND max_reconciliation_attempts),

    CONSTRAINT async_job_claimed_by_check CHECK (
        claimed_by IS NULL
        OR (claimed_by ~ '[^[:space:]]' AND char_length(claimed_by) <= 128)
    ),

    CONSTRAINT async_job_executor_principal_check CHECK (
        executor_principal IS NULL OR char_length(executor_principal) <= 256
    ),

    CONSTRAINT async_job_first_claimed_at_check
        CHECK ((first_claimed_at IS NOT NULL) = (attempt_count > 0)),

    CONSTRAINT async_job_checkpoint_check CHECK (
        checkpoint IS NULL
        OR (jsonb_typeof(checkpoint) = 'object'
            AND octet_length(checkpoint::text) <= 65536)
    ),

    CONSTRAINT async_job_provider_ref_check CHECK (
        provider_ref IS NULL OR char_length(provider_ref) <= 512
    ),

    CONSTRAINT async_job_last_outcome_code_check CHECK (
        last_outcome_code IS NULL
        OR (last_outcome_code ~ '^[A-Z][A-Z0-9_]*$'
            AND char_length(last_outcome_code) <= 64)
    ),

    CONSTRAINT async_job_causation_ref_check CHECK (
        causation_ref IS NULL
        OR (causation_ref ~ '^[!-~]+$' AND char_length(causation_ref) <= 256)
    ),

    CONSTRAINT async_job_created_by_check CHECK (
        created_by ~ '[^[:space:]]' AND char_length(created_by) <= 256
    ),

    -- claim state: RUNNING has all six claim columns; RECONCILIATION_REQUIRED
    -- has all six (a reconciliation claim) or none; every other status none.
    CONSTRAINT async_job_claim_state_check CHECK (
        CASE status
            WHEN 'RUNNING' THEN
                num_nonnulls(claim_token, claimed_by, executor_principal,
                             claimed_at, lease_expires_at,
                             attempt_runtime_deadline) = 6
            WHEN 'RECONCILIATION_REQUIRED' THEN
                num_nonnulls(claim_token, claimed_by, executor_principal,
                             claimed_at, lease_expires_at,
                             attempt_runtime_deadline) IN (0, 6)
            ELSE
                num_nonnulls(claim_token, claimed_by, executor_principal,
                             claimed_at, lease_expires_at,
                             attempt_runtime_deadline) = 0
        END
    ),

    CONSTRAINT async_job_terminal_check CHECK (
        (terminal_at IS NOT NULL) = (status IN ('SUCCEEDED', 'FAILED', 'CANCELLED'))
    ),

    CONSTRAINT async_job_redacted_at_check
        CHECK (redacted_at IS NULL OR terminal_at IS NOT NULL),

    CONSTRAINT async_job_waiting_check CHECK (
        status <> 'WAITING'
        OR (checkpoint IS NOT NULL AND waiting_deadline IS NOT NULL)
    ),

    CONSTRAINT async_job_pre_write_states_check CHECK (
        status NOT IN ('PENDING', 'RETRY_SCHEDULED') OR effect_phase = 'PRE_WRITE'
    ),

    CONSTRAINT async_job_effect_class_check
        CHECK (effect_class = 'EFFECTFUL' OR effect_phase = 'PRE_WRITE'),

    CONSTRAINT async_job_revision_check CHECK (
        (revision_mode = 'REVISION_BOUND') = (source_order IS NOT NULL)
        AND (revision_mode = 'REVISION_BOUND') = (source_fingerprint IS NOT NULL)
    ),

    CONSTRAINT async_job_lineage_check
        CHECK ((lineage = 'ORIGINAL') = (parent_job_id IS NULL)),

    CONSTRAINT async_job_counters_check CHECK (
        attempt_count >= execution_claim_count + reconciliation_attempt_count
    )
);

SELECT diesel_manage_updated_at('public.async_job');

-- Ready scan (execution claims).
CREATE INDEX async_job_ready_idx
    ON public.async_job (kind, priority DESC, available_at, created_at, async_job_id)
    WHERE status IN ('PENDING', 'RETRY_SCHEDULED', 'WAITING');

-- Reconciliation scan (unclaimed RECONCILIATION_REQUIRED jobs).
CREATE INDEX async_job_recon_scan_idx
    ON public.async_job (kind, available_at, async_job_id)
    WHERE status = 'RECONCILIATION_REQUIRED' AND claim_token IS NULL;

-- Lease scan (recovery of expired claims).
CREATE INDEX async_job_lease_idx
    ON public.async_job (lease_expires_at)
    WHERE claim_token IS NOT NULL;

-- Waiting-deadline scan (recovery of exhausted waits).
CREATE INDEX async_job_waiting_deadline_idx
    ON public.async_job (waiting_deadline)
    WHERE status = 'WAITING';

CREATE INDEX async_job_kind_status_idx
    ON public.async_job (kind, status);

CREATE INDEX async_job_status_updated_at_idx
    ON public.async_job (status, updated_at);

CREATE INDEX async_job_correlation_id_idx
    ON public.async_job (correlation_id)
    WHERE correlation_id IS NOT NULL;

-- One manual retry per job, so retry chains are linear.
CREATE UNIQUE INDEX async_job_parent_job_id_key
    ON public.async_job (parent_job_id)
    WHERE parent_job_id IS NOT NULL;

CREATE INDEX async_job_created_at_idx
    ON public.async_job (created_at DESC, async_job_id DESC);

-- ---------------------------------------------------------------------------
-- async_job_target (spec section 7.3)
-- ---------------------------------------------------------------------------

CREATE TABLE public.async_job_target (
    async_job_id uuid NOT NULL,
    target_key text NOT NULL,

    CONSTRAINT async_job_target_pkey PRIMARY KEY (async_job_id, target_key),

    CONSTRAINT async_job_target_async_job_id_fkey
        FOREIGN KEY (async_job_id)
        REFERENCES public.async_job(async_job_id) ON DELETE RESTRICT,

    -- A lower-case namespace of 2 to 32 characters, a colon, then 1 to 400
    -- characters of printable non-space ASCII. The namespace has no colon, so
    -- `position` finds its end.
    CONSTRAINT async_job_target_target_key_check CHECK (
        target_key ~ '^[a-z][a-z0-9_]{1,31}:[!-~]+$'
        AND char_length(target_key) - position(':' in target_key) <= 400
    )
);

CREATE INDEX async_job_target_target_key_idx
    ON public.async_job_target (target_key);

-- ---------------------------------------------------------------------------
-- async_target_hold (spec section 7.4)
-- ---------------------------------------------------------------------------

CREATE TABLE public.async_target_hold (
    target_key text NOT NULL,
    async_job_id uuid NOT NULL,
    acquired_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,

    CONSTRAINT async_target_hold_pkey PRIMARY KEY (target_key),

    CONSTRAINT async_target_hold_target_fkey
        FOREIGN KEY (async_job_id, target_key)
        REFERENCES public.async_job_target(async_job_id, target_key) ON DELETE RESTRICT
);

CREATE INDEX async_target_hold_async_job_id_idx
    ON public.async_target_hold (async_job_id);

-- ---------------------------------------------------------------------------
-- async_control_command (spec section 7.7)
-- ---------------------------------------------------------------------------

CREATE TABLE public.async_control_command (
    async_control_command_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    command_kind public.async_control_command_kind NOT NULL,
    request_id uuid NOT NULL,
    actor_principal text NOT NULL,
    reason text NOT NULL,
    async_job_id uuid NOT NULL,
    request jsonb NOT NULL,
    outcome public.async_control_command_outcome,
    outcome_code text,
    result_job_id uuid,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    decided_at timestamp with time zone,

    CONSTRAINT async_control_command_pkey PRIMARY KEY (async_control_command_id),

    CONSTRAINT async_control_command_request_id_key
        UNIQUE (command_kind, request_id),

    CONSTRAINT async_control_command_async_job_id_fkey
        FOREIGN KEY (async_job_id)
        REFERENCES public.async_job(async_job_id) ON DELETE RESTRICT,

    CONSTRAINT async_control_command_result_job_id_fkey
        FOREIGN KEY (result_job_id)
        REFERENCES public.async_job(async_job_id) ON DELETE RESTRICT,

    CONSTRAINT async_control_command_actor_principal_check
        CHECK (char_length(actor_principal) <= 256),

    CONSTRAINT async_control_command_reason_check CHECK (
        reason ~ '[^[:space:]]' AND char_length(reason) <= 1000
    ),

    CONSTRAINT async_control_command_request_check CHECK (
        jsonb_typeof(request) = 'object'
        AND octet_length(request::text) <= 4096
    ),

    -- `outcome_code` and `decided_at` are set once, together with `outcome`.
    CONSTRAINT async_control_command_decision_check CHECK (
        (outcome IS NULL) = (outcome_code IS NULL)
        AND (outcome IS NULL) = (decided_at IS NULL)
    ),

    -- `result_job_id` is the new job of an applied RETRY_JOB.
    CONSTRAINT async_control_command_result_job_id_check CHECK (
        result_job_id IS NULL
        OR (command_kind = 'RETRY_JOB' AND outcome = 'APPLIED')
    )
);

-- ---------------------------------------------------------------------------
-- async_job_attempt (spec section 7.6)
-- ---------------------------------------------------------------------------

CREATE TABLE public.async_job_attempt (
    async_job_attempt_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    async_job_id uuid NOT NULL,
    attempt_number integer NOT NULL,
    purpose public.async_attempt_purpose NOT NULL,
    claim_token uuid NOT NULL,
    claimed_by text NOT NULL,
    executor_principal text NOT NULL,
    control_command_id uuid,
    started_at timestamp with time zone NOT NULL,
    last_heartbeat_at timestamp with time zone NOT NULL,
    phase_at_start public.async_effect_phase NOT NULL,
    effect_started_at timestamp with time zone,
    effect_confirmed_at timestamp with time zone,
    finished_at timestamp with time zone,
    result public.async_attempt_result,
    job_status_after public.async_job_status,
    outcome_code text,
    error_code text,
    error_detail text,
    provider_ref text,
    evidence jsonb,
    redacted_at timestamp with time zone,
    redaction_command_id uuid,

    CONSTRAINT async_job_attempt_pkey PRIMARY KEY (async_job_attempt_id),

    CONSTRAINT async_job_attempt_async_job_id_fkey
        FOREIGN KEY (async_job_id)
        REFERENCES public.async_job(async_job_id) ON DELETE RESTRICT,

    CONSTRAINT async_job_attempt_control_command_id_fkey
        FOREIGN KEY (control_command_id)
        REFERENCES public.async_control_command(async_control_command_id),

    CONSTRAINT async_job_attempt_redaction_command_id_fkey
        FOREIGN KEY (redaction_command_id)
        REFERENCES public.async_control_command(async_control_command_id),

    CONSTRAINT async_job_attempt_number_key UNIQUE (async_job_id, attempt_number),

    -- One token is bound to one attempt for all time.
    CONSTRAINT async_job_attempt_claim_token_key UNIQUE (claim_token),

    CONSTRAINT async_job_attempt_number_check CHECK (attempt_number >= 1),

    CONSTRAINT async_job_attempt_command_check CHECK (
        (control_command_id IS NOT NULL) = (purpose = 'STAFF_RECONCILIATION')
    ),

    CONSTRAINT async_job_attempt_finished_check
        CHECK ((finished_at IS NULL) = (result IS NULL)),

    CONSTRAINT async_job_attempt_job_status_after_check
        CHECK ((job_status_after IS NULL) = (finished_at IS NULL)),

    CONSTRAINT async_job_attempt_outcome_code_check CHECK (
        outcome_code IS NULL
        OR (outcome_code ~ '^[A-Z][A-Z0-9_]*$' AND char_length(outcome_code) <= 64)
    ),

    CONSTRAINT async_job_attempt_error_code_check CHECK (
        error_code IS NULL
        OR (error_code ~ '^[A-Z][A-Z0-9_]*$' AND char_length(error_code) <= 64)
    ),

    CONSTRAINT async_job_attempt_error_detail_check CHECK (
        error_detail IS NULL
        OR (error_code IS NOT NULL AND char_length(error_detail) <= 2048)
    ),

    CONSTRAINT async_job_attempt_provider_ref_check CHECK (
        provider_ref IS NULL OR char_length(provider_ref) <= 512
    ),

    CONSTRAINT async_job_attempt_evidence_check CHECK (
        evidence IS NULL
        OR (jsonb_typeof(evidence) = 'object'
            AND octet_length(evidence::text) <= 16384)
    ),

    CONSTRAINT async_job_attempt_redaction_check
        CHECK ((redacted_at IS NULL) = (redaction_command_id IS NULL))
);

-- One open attempt per job.
CREATE UNIQUE INDEX async_job_attempt_open_key
    ON public.async_job_attempt (async_job_id)
    WHERE finished_at IS NULL;

-- Attempt-duration signals.
CREATE INDEX async_job_attempt_finished_at_idx
    ON public.async_job_attempt (finished_at)
    WHERE finished_at IS NOT NULL;

-- ---------------------------------------------------------------------------
-- async_effect_target_state (spec section 7.5)
-- ---------------------------------------------------------------------------

CREATE TABLE public.async_effect_target_state (
    target_key text NOT NULL,
    applied_order bigint,
    applied_fingerprint text NOT NULL,
    applied_source_ref text,
    last_job_id uuid NOT NULL,
    last_attempt_id uuid NOT NULL,
    regressed_at timestamp with time zone,
    regressed_from_order bigint,
    regressed_from_fingerprint text,
    updated_at timestamp with time zone NOT NULL,

    CONSTRAINT async_effect_target_state_pkey PRIMARY KEY (target_key),

    CONSTRAINT async_effect_target_state_last_job_id_fkey
        FOREIGN KEY (last_job_id)
        REFERENCES public.async_job(async_job_id),

    CONSTRAINT async_effect_target_state_last_attempt_id_fkey
        FOREIGN KEY (last_attempt_id)
        REFERENCES public.async_job_attempt(async_job_attempt_id),

    CONSTRAINT async_effect_target_state_applied_fingerprint_check CHECK (
        applied_fingerprint ~ '^[!-~]+$' AND char_length(applied_fingerprint) <= 256
    ),

    CONSTRAINT async_effect_target_state_applied_source_ref_check CHECK (
        applied_source_ref IS NULL
        OR (applied_source_ref ~ '^[!-~]+$' AND char_length(applied_source_ref) <= 256)
    ),

    CONSTRAINT async_effect_target_state_regressed_check CHECK (
        num_nonnulls(regressed_at, regressed_from_order, regressed_from_fingerprint) IN (0, 3)
    )
);

-- Regressed-target signal.
CREATE INDEX async_effect_target_state_regressed_idx
    ON public.async_effect_target_state (target_key)
    WHERE regressed_at IS NOT NULL;

-- ---------------------------------------------------------------------------
-- async_job_transition (spec section 7.8)
-- ---------------------------------------------------------------------------

CREATE TABLE public.async_job_transition (
    async_job_transition_id bigint GENERATED ALWAYS AS IDENTITY NOT NULL,
    async_job_id uuid NOT NULL,
    from_status public.async_job_status,
    to_status public.async_job_status NOT NULL,
    cause text NOT NULL,
    async_job_attempt_id uuid,
    async_control_command_id uuid,
    occurred_at timestamp with time zone DEFAULT clock_timestamp() NOT NULL,

    CONSTRAINT async_job_transition_pkey PRIMARY KEY (async_job_transition_id),

    CONSTRAINT async_job_transition_async_job_id_fkey
        FOREIGN KEY (async_job_id)
        REFERENCES public.async_job(async_job_id),

    CONSTRAINT async_job_transition_async_job_attempt_id_fkey
        FOREIGN KEY (async_job_attempt_id)
        REFERENCES public.async_job_attempt(async_job_attempt_id),

    CONSTRAINT async_job_transition_async_control_command_id_fkey
        FOREIGN KEY (async_control_command_id)
        REFERENCES public.async_control_command(async_control_command_id)
);

-- Single-job read.
CREATE INDEX async_job_transition_job_idx
    ON public.async_job_transition (async_job_id, async_job_transition_id);

-- ---------------------------------------------------------------------------
-- async_executor_poll and async_recovery_failure (spec section 7.9)
-- ---------------------------------------------------------------------------

CREATE TABLE public.async_executor_poll (
    executor_name text NOT NULL,
    executor_principal text NOT NULL,
    kind text NOT NULL,
    purpose public.async_attempt_purpose NOT NULL,
    payload_versions smallint[] NOT NULL,
    last_poll_at timestamp with time zone NOT NULL,
    last_claim_at timestamp with time zone,

    CONSTRAINT async_executor_poll_pkey
        PRIMARY KEY (executor_name, executor_principal, kind, purpose),

    CONSTRAINT async_executor_poll_purpose_check
        CHECK (purpose IN ('EXECUTION', 'RECONCILIATION'))
);

CREATE TABLE public.async_recovery_failure (
    async_job_id uuid NOT NULL,
    first_failed_at timestamp with time zone NOT NULL,
    last_failed_at timestamp with time zone NOT NULL,
    failure_count integer NOT NULL,
    error_code text NOT NULL,

    CONSTRAINT async_recovery_failure_pkey PRIMARY KEY (async_job_id),

    CONSTRAINT async_recovery_failure_async_job_id_fkey
        FOREIGN KEY (async_job_id)
        REFERENCES public.async_job(async_job_id) ON DELETE RESTRICT,

    CONSTRAINT async_recovery_failure_failure_count_check
        CHECK (failure_count >= 1)
);

-- ---------------------------------------------------------------------------
-- Row triggers (spec section 7.10). Every refusal raises
-- `integrity_constraint_violation` with a fixed message and the guard's name
-- as its constraint, and never quotes row content.
-- ---------------------------------------------------------------------------

-- BEFORE INSERT on async_job: a new row is PENDING at PRE_WRITE with zero
-- counters and no claim, wait, diagnostic, terminal or redaction state.
CREATE FUNCTION public.async_job_before_insert() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    IF NEW.status <> 'PENDING'
       OR NEW.effect_phase <> 'PRE_WRITE'
       OR NEW.attempt_count <> 0
       OR NEW.execution_claim_count <> 0
       OR NEW.prewrite_failure_count <> 0
       OR NEW.reconciliation_attempt_count <> 0
       OR NEW.checkpoint_version <> 0
       OR num_nonnulls(NEW.claim_token, NEW.claimed_by, NEW.executor_principal,
                       NEW.claimed_at, NEW.lease_expires_at,
                       NEW.attempt_runtime_deadline, NEW.first_claimed_at,
                       NEW.waiting_deadline, NEW.checkpoint, NEW.provider_ref,
                       NEW.terminal_at, NEW.last_outcome_code,
                       NEW.redacted_at) <> 0
    THEN
        RAISE EXCEPTION 'async_job: a new job must be PENDING at PRE_WRITE with no claim or progress'
            USING ERRCODE = 'integrity_constraint_violation',
                  CONSTRAINT = 'async_job_insert_guard';
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER async_job_insert_guard
    BEFORE INSERT ON public.async_job
    FOR EACH ROW EXECUTE FUNCTION public.async_job_before_insert();

-- BEFORE UPDATE on async_job: rules 1 to 7 of spec section 7.10.
CREATE FUNCTION public.async_job_before_update() RETURNS trigger
LANGUAGE plpgsql AS $$
DECLARE
    terminal_old boolean := OLD.status IN ('SUCCEEDED', 'FAILED', 'CANCELLED');
    legal boolean;
BEGIN
    -- Rule 1: identity and snapshot columns never change.
    IF NEW.kind IS DISTINCT FROM OLD.kind
       OR NEW.payload_version IS DISTINCT FROM OLD.payload_version
       OR NEW.payload IS DISTINCT FROM OLD.payload
       OR NEW.idempotency_key IS DISTINCT FROM OLD.idempotency_key
       OR NEW.effect_class IS DISTINCT FROM OLD.effect_class
       OR NEW.revision_mode IS DISTINCT FROM OLD.revision_mode
       OR NEW.retry_budget IS DISTINCT FROM OLD.retry_budget
       OR NEW.max_execution_claims IS DISTINCT FROM OLD.max_execution_claims
       OR NEW.max_reconciliation_attempts IS DISTINCT FROM OLD.max_reconciliation_attempts
       OR NEW.backoff_base_seconds IS DISTINCT FROM OLD.backoff_base_seconds
       OR NEW.backoff_max_seconds IS DISTINCT FROM OLD.backoff_max_seconds
       OR NEW.source_order IS DISTINCT FROM OLD.source_order
       OR NEW.source_fingerprint IS DISTINCT FROM OLD.source_fingerprint
       OR NEW.source_ref IS DISTINCT FROM OLD.source_ref
       OR NEW.lineage IS DISTINCT FROM OLD.lineage
       OR NEW.parent_job_id IS DISTINCT FROM OLD.parent_job_id
       OR NEW.correlation_id IS DISTINCT FROM OLD.correlation_id
       OR NEW.causation_ref IS DISTINCT FROM OLD.causation_ref
       OR NEW.created_by IS DISTINCT FROM OLD.created_by
       OR NEW.created_at IS DISTINCT FROM OLD.created_at
       OR NEW.async_job_id IS DISTINCT FROM OLD.async_job_id
    THEN
        RAISE EXCEPTION 'async_job: identity and policy snapshot columns are immutable'
            USING ERRCODE = 'integrity_constraint_violation',
                  CONSTRAINT = 'async_job_update_guard';
    END IF;

    -- Rule 2: a terminal row never changes, except one redaction.
    IF terminal_old THEN
        IF OLD.redacted_at IS NULL
           AND NEW.redacted_at IS NOT NULL
           AND NEW.provider_ref IS NULL
           AND NEW.checkpoint IS NULL
           AND (to_jsonb(NEW) - ARRAY['provider_ref', 'checkpoint', 'redacted_at', 'updated_at'])
               = (to_jsonb(OLD) - ARRAY['provider_ref', 'checkpoint', 'redacted_at', 'updated_at'])
        THEN
            RETURN NEW;
        END IF;
        RAISE EXCEPTION 'async_job: a terminal job is frozen except for one redaction'
            USING ERRCODE = 'integrity_constraint_violation',
                  CONSTRAINT = 'async_job_update_guard';
    END IF;

    -- Rule 3: a legal status pair (spec section 9.2), or the same non-terminal
    -- status.
    IF NEW.status IS DISTINCT FROM OLD.status THEN
        legal := CASE OLD.status
            WHEN 'PENDING' THEN NEW.status IN ('RUNNING', 'CANCELLED')
            WHEN 'RETRY_SCHEDULED' THEN NEW.status IN ('RUNNING', 'CANCELLED')
            WHEN 'WAITING' THEN NEW.status IN ('RUNNING', 'CANCELLED', 'FAILED',
                                               'RECONCILIATION_REQUIRED')
            WHEN 'RUNNING' THEN NEW.status IN ('SUCCEEDED', 'FAILED', 'RETRY_SCHEDULED',
                                               'WAITING', 'PENDING', 'CANCELLED',
                                               'RECONCILIATION_REQUIRED')
            WHEN 'RECONCILIATION_REQUIRED' THEN NEW.status IN ('SUCCEEDED', 'FAILED',
                                                               'CANCELLED', 'WAITING')
            ELSE false
        END;
        IF NOT legal THEN
            RAISE EXCEPTION 'async_job: illegal status transition'
                USING ERRCODE = 'integrity_constraint_violation',
                      CONSTRAINT = 'async_job_update_guard';
        END IF;

        -- Rule 4: phase rules on the new status.
        IF NEW.status = 'CANCELLED' AND NEW.effect_phase = 'EFFECT_CONFIRMED' THEN
            RAISE EXCEPTION 'async_job: a job at EFFECT_CONFIRMED cannot be cancelled'
                USING ERRCODE = 'integrity_constraint_violation',
                      CONSTRAINT = 'async_job_update_guard';
        END IF;
        IF NEW.status = 'FAILED' AND NEW.effect_phase = 'EFFECT_CONFIRMED'
           AND OLD.status <> 'RECONCILIATION_REQUIRED'
        THEN
            RAISE EXCEPTION 'async_job: a job at EFFECT_CONFIRMED fails only through staff resolution'
                USING ERRCODE = 'integrity_constraint_violation',
                      CONSTRAINT = 'async_job_update_guard';
        END IF;
        IF NEW.status = 'CANCELLED' AND OLD.status IN ('RUNNING', 'WAITING')
           AND NEW.effect_phase <> 'PRE_WRITE'
        THEN
            RAISE EXCEPTION 'async_job: a running or waiting job is cancelled only at PRE_WRITE'
                USING ERRCODE = 'integrity_constraint_violation',
                      CONSTRAINT = 'async_job_update_guard';
        END IF;
        IF NEW.status = 'FAILED' AND OLD.status = 'WAITING'
           AND NEW.effect_phase <> 'PRE_WRITE'
        THEN
            RAISE EXCEPTION 'async_job: a waiting job fails directly only at PRE_WRITE'
                USING ERRCODE = 'integrity_constraint_violation',
                      CONSTRAINT = 'async_job_update_guard';
        END IF;
    END IF;

    -- Rule 5: the effect phase never moves backwards.
    IF NEW.effect_phase < OLD.effect_phase THEN
        RAISE EXCEPTION 'async_job: the effect phase never moves backwards'
            USING ERRCODE = 'integrity_constraint_violation',
                  CONSTRAINT = 'async_job_update_guard';
    END IF;

    -- Rule 6: counters and the checkpoint version never decrease.
    IF NEW.attempt_count < OLD.attempt_count
       OR NEW.execution_claim_count < OLD.execution_claim_count
       OR NEW.prewrite_failure_count < OLD.prewrite_failure_count
       OR NEW.reconciliation_attempt_count < OLD.reconciliation_attempt_count
       OR NEW.checkpoint_version < OLD.checkpoint_version
    THEN
        RAISE EXCEPTION 'async_job: counters never decrease'
            USING ERRCODE = 'integrity_constraint_violation',
                  CONSTRAINT = 'async_job_update_guard';
    END IF;

    -- Rule 7: write-once timestamps.
    IF (OLD.first_claimed_at IS NOT NULL
            AND NEW.first_claimed_at IS DISTINCT FROM OLD.first_claimed_at)
       OR (OLD.waiting_deadline IS NOT NULL
            AND NEW.waiting_deadline IS DISTINCT FROM OLD.waiting_deadline)
    THEN
        RAISE EXCEPTION 'async_job: first_claimed_at and waiting_deadline are write-once'
            USING ERRCODE = 'integrity_constraint_violation',
                  CONSTRAINT = 'async_job_update_guard';
    END IF;

    RETURN NEW;
END;
$$;

CREATE TRIGGER async_job_update_guard
    BEFORE UPDATE ON public.async_job
    FOR EACH ROW EXECUTE FUNCTION public.async_job_before_update();

-- AFTER INSERT and AFTER UPDATE of status on async_job: append the
-- transition log row. The cause and the attempt and command ids come from the
-- engine's transaction-local settings; a write that sets none is UNSPECIFIED.
CREATE FUNCTION public.async_job_record_transition() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    INSERT INTO public.async_job_transition (
        async_job_id, from_status, to_status, cause,
        async_job_attempt_id, async_control_command_id
    ) VALUES (
        NEW.async_job_id,
        CASE WHEN TG_OP = 'INSERT' THEN NULL ELSE OLD.status END,
        NEW.status,
        COALESCE(NULLIF(current_setting('thoth.async_cause', true), ''), 'UNSPECIFIED'),
        NULLIF(current_setting('thoth.async_attempt_id', true), '')::uuid,
        NULLIF(current_setting('thoth.async_command_id', true), '')::uuid
    );
    RETURN NULL;
END;
$$;

CREATE TRIGGER async_job_transition_on_insert
    AFTER INSERT ON public.async_job
    FOR EACH ROW EXECUTE FUNCTION public.async_job_record_transition();

CREATE TRIGGER async_job_transition_on_update
    AFTER UPDATE ON public.async_job
    FOR EACH ROW
    WHEN (OLD.status IS DISTINCT FROM NEW.status)
    EXECUTE FUNCTION public.async_job_record_transition();

-- async_job_target rows are written only at job creation.
CREATE FUNCTION public.async_job_target_guard() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'async_job_target: job targets are immutable'
        USING ERRCODE = 'integrity_constraint_violation',
              CONSTRAINT = 'async_job_target_guard';
END;
$$;

CREATE TRIGGER async_job_target_guard
    BEFORE UPDATE OR DELETE ON public.async_job_target
    FOR EACH ROW EXECUTE FUNCTION public.async_job_target_guard();

-- A hold is only inserted or deleted.
CREATE FUNCTION public.async_target_hold_guard() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'async_target_hold: a hold is only inserted or deleted'
        USING ERRCODE = 'integrity_constraint_violation',
              CONSTRAINT = 'async_target_hold_guard';
END;
$$;

CREATE TRIGGER async_target_hold_guard
    BEFORE UPDATE ON public.async_target_hold
    FOR EACH ROW EXECUTE FUNCTION public.async_target_hold_guard();

-- Applied evidence moves only under the target's hold, by the holder's open
-- attempt, never to a null order and never backwards except by the staff
-- ACCEPT_REGRESSION write; the trigger alone maintains `regressed_*` and
-- `updated_at` (spec section 7.5).
CREATE FUNCTION public.async_effect_target_state_guard() RETURNS trigger
LANGUAGE plpgsql AS $$
DECLARE
    attempt_purpose public.async_attempt_purpose;
    attempt_finished timestamp with time zone;
    job_mode public.async_job_revision_mode;
    job_order bigint;
    job_fingerprint text;
    accepted boolean := false;
BEGIN
    IF TG_OP = 'DELETE' THEN
        RAISE EXCEPTION 'async_effect_target_state: applied evidence is never deleted'
            USING ERRCODE = 'integrity_constraint_violation',
                  CONSTRAINT = 'async_effect_target_state_guard';
    END IF;

    IF TG_OP = 'UPDATE' AND NEW.target_key IS DISTINCT FROM OLD.target_key THEN
        RAISE EXCEPTION 'async_effect_target_state: the target key is immutable'
            USING ERRCODE = 'integrity_constraint_violation',
                  CONSTRAINT = 'async_effect_target_state_guard';
    END IF;

    IF NOT EXISTS (
        SELECT 1
          FROM public.async_target_hold h
         WHERE h.target_key = NEW.target_key
           AND h.async_job_id = NEW.last_job_id
    ) THEN
        RAISE EXCEPTION 'async_effect_target_state: only the holder of the target writes its evidence'
            USING ERRCODE = 'integrity_constraint_violation',
                  CONSTRAINT = 'async_effect_target_state_guard';
    END IF;

    SELECT a.purpose, a.finished_at
      INTO attempt_purpose, attempt_finished
      FROM public.async_job_attempt a
     WHERE a.async_job_attempt_id = NEW.last_attempt_id
       AND a.async_job_id = NEW.last_job_id;
    IF NOT FOUND OR attempt_finished IS NOT NULL THEN
        RAISE EXCEPTION 'async_effect_target_state: evidence is written by an open attempt of the holding job'
            USING ERRCODE = 'integrity_constraint_violation',
                  CONSTRAINT = 'async_effect_target_state_guard';
    END IF;

    IF TG_OP = 'UPDATE' THEN
        IF OLD.applied_order IS NOT NULL AND NEW.applied_order IS NULL THEN
            RAISE EXCEPTION 'async_effect_target_state: a stored order never becomes null'
                USING ERRCODE = 'integrity_constraint_violation',
                      CONSTRAINT = 'async_effect_target_state_guard';
        END IF;

        accepted := OLD.applied_order IS NOT NULL
            AND (NEW.applied_order < OLD.applied_order
                 OR (NEW.applied_order = OLD.applied_order
                     AND NEW.applied_fingerprint IS DISTINCT FROM OLD.applied_fingerprint));

        IF accepted THEN
            SELECT j.revision_mode, j.source_order, j.source_fingerprint
              INTO job_mode, job_order, job_fingerprint
              FROM public.async_job j
             WHERE j.async_job_id = NEW.last_job_id;
            IF attempt_purpose IS DISTINCT FROM 'STAFF_RECONCILIATION'
               OR job_mode IS DISTINCT FROM 'REVISION_BOUND'
               OR NEW.applied_order IS DISTINCT FROM job_order
               OR NEW.applied_fingerprint IS DISTINCT FROM job_fingerprint
            THEN
                RAISE EXCEPTION 'async_effect_target_state: applied evidence never moves backwards'
                    USING ERRCODE = 'integrity_constraint_violation',
                          CONSTRAINT = 'async_effect_target_state_guard';
            END IF;
            NEW.regressed_at := clock_timestamp();
            IF OLD.regressed_at IS NULL THEN
                NEW.regressed_from_order := OLD.applied_order;
                NEW.regressed_from_fingerprint := OLD.applied_fingerprint;
            ELSE
                NEW.regressed_from_order := OLD.regressed_from_order;
                NEW.regressed_from_fingerprint := OLD.regressed_from_fingerprint;
            END IF;
        ELSIF OLD.regressed_at IS NOT NULL
              AND (NEW.applied_order > OLD.regressed_from_order
                   OR (NEW.applied_order = OLD.regressed_from_order
                       AND NEW.applied_fingerprint = OLD.regressed_from_fingerprint))
        THEN
            NEW.regressed_at := NULL;
            NEW.regressed_from_order := NULL;
            NEW.regressed_from_fingerprint := NULL;
        ELSE
            NEW.regressed_at := OLD.regressed_at;
            NEW.regressed_from_order := OLD.regressed_from_order;
            NEW.regressed_from_fingerprint := OLD.regressed_from_fingerprint;
        END IF;
    ELSE
        NEW.regressed_at := NULL;
        NEW.regressed_from_order := NULL;
        NEW.regressed_from_fingerprint := NULL;
    END IF;

    NEW.updated_at := clock_timestamp();
    RETURN NEW;
END;
$$;

CREATE TRIGGER async_effect_target_state_guard
    BEFORE INSERT OR UPDATE OR DELETE ON public.async_effect_target_state
    FOR EACH ROW EXECUTE FUNCTION public.async_effect_target_state_guard();

-- Attempt immutability (spec section 7.6).
CREATE FUNCTION public.async_job_attempt_guard() RETURNS trigger
LANGUAGE plpgsql AS $$
DECLARE
    open_columns text[] := ARRAY[
        'last_heartbeat_at', 'effect_started_at', 'effect_confirmed_at',
        'provider_ref', 'finished_at', 'result', 'job_status_after',
        'outcome_code', 'error_code', 'error_detail', 'evidence'];
    redaction_columns text[] := ARRAY[
        'redacted_at', 'redaction_command_id', 'error_detail', 'provider_ref',
        'evidence'];
BEGIN
    IF TG_OP = 'DELETE' THEN
        RAISE EXCEPTION 'async_job_attempt: attempts are never deleted'
            USING ERRCODE = 'integrity_constraint_violation',
                  CONSTRAINT = 'async_job_attempt_guard';
    END IF;

    IF OLD.finished_at IS NULL THEN
        -- (a) an open attempt: heartbeat, write-once effect marks, the
        -- provider reference, and the closing columns set once together.
        IF (to_jsonb(NEW) - open_columns) IS DISTINCT FROM (to_jsonb(OLD) - open_columns)
           OR (OLD.effect_started_at IS NOT NULL
               AND NEW.effect_started_at IS DISTINCT FROM OLD.effect_started_at)
           OR (OLD.effect_confirmed_at IS NOT NULL
               AND NEW.effect_confirmed_at IS DISTINCT FROM OLD.effect_confirmed_at)
           OR (NEW.finished_at IS NULL
               AND (NEW.result IS DISTINCT FROM OLD.result
                    OR NEW.job_status_after IS DISTINCT FROM OLD.job_status_after
                    OR NEW.outcome_code IS DISTINCT FROM OLD.outcome_code
                    OR NEW.error_code IS DISTINCT FROM OLD.error_code
                    OR NEW.error_detail IS DISTINCT FROM OLD.error_detail
                    OR NEW.evidence IS DISTINCT FROM OLD.evidence))
        THEN
            RAISE EXCEPTION 'async_job_attempt: illegal change to an open attempt'
                USING ERRCODE = 'integrity_constraint_violation',
                      CONSTRAINT = 'async_job_attempt_guard';
        END IF;
        RETURN NEW;
    END IF;

    -- (b) the one redaction of a finished attempt.
    IF OLD.redacted_at IS NULL
       AND NEW.redacted_at IS NOT NULL
       AND NEW.redaction_command_id IS NOT NULL
       AND NEW.error_detail IS NULL
       AND NEW.provider_ref IS NULL
       AND NEW.evidence IS NULL
       AND (to_jsonb(NEW) - redaction_columns) = (to_jsonb(OLD) - redaction_columns)
    THEN
        RETURN NEW;
    END IF;
    RAISE EXCEPTION 'async_job_attempt: a finished attempt changes only by one redaction'
        USING ERRCODE = 'integrity_constraint_violation',
              CONSTRAINT = 'async_job_attempt_guard';
END;
$$;

CREATE TRIGGER async_job_attempt_guard
    BEFORE UPDATE OR DELETE ON public.async_job_attempt
    FOR EACH ROW EXECUTE FUNCTION public.async_job_attempt_guard();

-- Command rows are append-only: one deciding update (spec section 7.7).
CREATE FUNCTION public.async_control_command_guard() RETURNS trigger
LANGUAGE plpgsql AS $$
DECLARE
    decision_columns text[] := ARRAY['outcome', 'outcome_code', 'decided_at', 'result_job_id'];
BEGIN
    IF TG_OP = 'DELETE' THEN
        RAISE EXCEPTION 'async_control_command: command rows are never deleted'
            USING ERRCODE = 'integrity_constraint_violation',
                  CONSTRAINT = 'async_control_command_guard';
    END IF;
    IF OLD.outcome IS NULL
       AND OLD.result_job_id IS NULL
       AND NEW.outcome IS NOT NULL
       AND NEW.outcome_code IS NOT NULL
       AND NEW.decided_at IS NOT NULL
       AND (to_jsonb(NEW) - decision_columns) = (to_jsonb(OLD) - decision_columns)
    THEN
        RETURN NEW;
    END IF;
    RAISE EXCEPTION 'async_control_command: a command is decided once and never changes again'
        USING ERRCODE = 'integrity_constraint_violation',
              CONSTRAINT = 'async_control_command_guard';
END;
$$;

CREATE TRIGGER async_control_command_guard
    BEFORE UPDATE OR DELETE ON public.async_control_command
    FOR EACH ROW EXECUTE FUNCTION public.async_control_command_guard();

-- The transition log is append-only.
CREATE FUNCTION public.async_job_transition_guard() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'async_job_transition: the transition log is append-only'
        USING ERRCODE = 'integrity_constraint_violation',
              CONSTRAINT = 'async_job_transition_guard';
END;
$$;

CREATE TRIGGER async_job_transition_guard
    BEFORE UPDATE OR DELETE ON public.async_job_transition
    FOR EACH ROW EXECUTE FUNCTION public.async_job_transition_guard();

-- ---------------------------------------------------------------------------
-- Deferred constraint triggers (spec section 7.10). Each re-reads the
-- affected job's current state when it fires and never tests the queued row,
-- so a transaction is judged only by its final state.
-- ---------------------------------------------------------------------------

-- S5: a RUNNING, WAITING or RECONCILIATION_REQUIRED job holds every one of
-- its targets; a job in any other status holds none. The composite foreign key
-- makes holds a subset of targets, so equal counts mean equal sets.
CREATE FUNCTION public.async_job_hold_check() RETURNS trigger
LANGUAGE plpgsql AS $$
DECLARE
    job_id uuid;
    job_status public.async_job_status;
    target_count bigint;
    hold_count bigint;
BEGIN
    IF TG_OP = 'DELETE' THEN
        job_id := OLD.async_job_id;
    ELSE
        job_id := NEW.async_job_id;
    END IF;

    SELECT j.status INTO job_status
      FROM public.async_job j
     WHERE j.async_job_id = job_id;
    IF NOT FOUND THEN
        RETURN NULL;
    END IF;

    SELECT count(*) INTO target_count
      FROM public.async_job_target t
     WHERE t.async_job_id = job_id;
    SELECT count(*) INTO hold_count
      FROM public.async_target_hold h
     WHERE h.async_job_id = job_id;

    IF job_status IN ('RUNNING', 'WAITING', 'RECONCILIATION_REQUIRED') THEN
        IF hold_count <> target_count THEN
            RAISE EXCEPTION 'async_job: a running, waiting or reconciliation-required job must hold all of its targets'
                USING ERRCODE = 'integrity_constraint_violation',
                      CONSTRAINT = 'async_job_hold_consistency';
        END IF;
    ELSIF hold_count <> 0 THEN
        RAISE EXCEPTION 'async_job: a job that is not running, waiting or reconciliation-required holds no target'
            USING ERRCODE = 'integrity_constraint_violation',
                  CONSTRAINT = 'async_job_hold_consistency';
    END IF;
    RETURN NULL;
END;
$$;

CREATE CONSTRAINT TRIGGER async_job_hold_consistency
    AFTER INSERT OR UPDATE OF status ON public.async_job
    DEFERRABLE INITIALLY DEFERRED
    FOR EACH ROW EXECUTE FUNCTION public.async_job_hold_check();

CREATE CONSTRAINT TRIGGER async_job_hold_consistency
    AFTER INSERT OR DELETE ON public.async_target_hold
    DEFERRABLE INITIALLY DEFERRED
    FOR EACH ROW EXECUTE FUNCTION public.async_job_hold_check();

-- Open attempts match the claim, and a FAILED or CANCELLED job past PRE_WRITE
-- was closed by the attempt that proves it.
CREATE FUNCTION public.async_job_attempt_check() RETURNS trigger
LANGUAGE plpgsql AS $$
DECLARE
    job public.async_job%ROWTYPE;
    open_count bigint;
    open_purpose public.async_attempt_purpose;
    open_token uuid;
    latest_purpose public.async_attempt_purpose;
    latest_outcome text;
BEGIN
    SELECT * INTO job
      FROM public.async_job j
     WHERE j.async_job_id = NEW.async_job_id;
    IF NOT FOUND THEN
        RETURN NULL;
    END IF;

    SELECT count(*), min(a.purpose::text)::public.async_attempt_purpose,
           (array_agg(a.claim_token))[1]
      INTO open_count, open_purpose, open_token
      FROM public.async_job_attempt a
     WHERE a.async_job_id = job.async_job_id
       AND a.finished_at IS NULL;

    IF job.status = 'RUNNING' THEN
        IF open_count <> 1 OR open_purpose <> 'EXECUTION'
           OR open_token IS DISTINCT FROM job.claim_token
        THEN
            RAISE EXCEPTION 'async_job: a running job has exactly one open execution attempt with its token'
                USING ERRCODE = 'integrity_constraint_violation',
                      CONSTRAINT = 'async_job_attempt_consistency';
        END IF;
    ELSIF job.status = 'RECONCILIATION_REQUIRED' AND job.claim_token IS NOT NULL THEN
        IF open_count <> 1 OR open_purpose <> 'RECONCILIATION'
           OR open_token IS DISTINCT FROM job.claim_token
        THEN
            RAISE EXCEPTION 'async_job: a claimed reconciliation-required job has exactly one open reconciliation attempt with its token'
                USING ERRCODE = 'integrity_constraint_violation',
                      CONSTRAINT = 'async_job_attempt_consistency';
        END IF;
    ELSIF open_count <> 0 THEN
        RAISE EXCEPTION 'async_job: an unclaimed job has no open attempt'
            USING ERRCODE = 'integrity_constraint_violation',
                  CONSTRAINT = 'async_job_attempt_consistency';
    END IF;

    IF job.status IN ('FAILED', 'CANCELLED') AND job.effect_phase <> 'PRE_WRITE' THEN
        SELECT a.purpose, a.outcome_code
          INTO latest_purpose, latest_outcome
          FROM public.async_job_attempt a
         WHERE a.async_job_id = job.async_job_id
           AND a.finished_at IS NOT NULL
         ORDER BY a.attempt_number DESC
         LIMIT 1;
        IF job.effect_phase = 'EFFECT_STARTED'
           AND latest_outcome IS DISTINCT FROM 'NO_EFFECT_PROVEN'
        THEN
            RAISE EXCEPTION 'async_job: a failed or cancelled job at EFFECT_STARTED was closed with proven no effect'
                USING ERRCODE = 'integrity_constraint_violation',
                      CONSTRAINT = 'async_job_attempt_consistency';
        END IF;
        IF job.effect_phase = 'EFFECT_CONFIRMED'
           AND (job.status <> 'FAILED'
                OR latest_purpose IS DISTINCT FROM 'STAFF_RECONCILIATION'
                OR latest_outcome IS DISTINCT FROM 'CONFIRMED_EFFECT_NOT_APPLIED')
        THEN
            RAISE EXCEPTION 'async_job: a failed job at EFFECT_CONFIRMED was closed by staff with not-applied evidence'
                USING ERRCODE = 'integrity_constraint_violation',
                      CONSTRAINT = 'async_job_attempt_consistency';
        END IF;
    END IF;
    RETURN NULL;
END;
$$;

CREATE CONSTRAINT TRIGGER async_job_attempt_consistency
    AFTER INSERT OR UPDATE ON public.async_job
    DEFERRABLE INITIALLY DEFERRED
    FOR EACH ROW EXECUTE FUNCTION public.async_job_attempt_check();

CREATE CONSTRAINT TRIGGER async_job_attempt_consistency
    AFTER INSERT OR UPDATE ON public.async_job_attempt
    DEFERRABLE INITIALLY DEFERRED
    FOR EACH ROW EXECUTE FUNCTION public.async_job_attempt_check();

-- No undecided command can commit.
CREATE FUNCTION public.async_control_command_decided_check() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    IF EXISTS (
        SELECT 1
          FROM public.async_control_command c
         WHERE c.async_control_command_id = NEW.async_control_command_id
           AND c.outcome IS NULL
    ) THEN
        RAISE EXCEPTION 'async_control_command: a command must be decided before commit'
            USING ERRCODE = 'integrity_constraint_violation',
                  CONSTRAINT = 'async_control_command_decided';
    END IF;
    RETURN NULL;
END;
$$;

CREATE CONSTRAINT TRIGGER async_control_command_decided
    AFTER INSERT ON public.async_control_command
    DEFERRABLE INITIALLY DEFERRED
    FOR EACH ROW EXECUTE FUNCTION public.async_control_command_decided_check();
