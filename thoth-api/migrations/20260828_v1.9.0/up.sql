-- MET-WP1-03: Metrics import and import-error state foundation (issue #863).
--
-- Additive and initially inactive. Creates the closed import lifecycle and
-- import-error severity enums and the two durable evidence tables:
-- metric_import (one normalized ingestion job associated with a raw report,
-- API response, source partition or publisher upload) and
-- metric_import_error (durable, machine-readable row-level findings owned by
-- one import). No row is seeded.
--
-- This slice stores state only. There is deliberately no status-transition
-- constraint, worker claim, lease, retry, stale-claim recovery, counter
-- mutation protocol or "return the existing import" behaviour: those belong
-- to the later bounded WP2/WP3 upload, claim and ingestion work. Metrics
-- deliberately does not reuse the Publisher Services distribution_job*
-- tables or lifecycle by analogy.
--
-- Index decision (reviewed): the complete intended index set is exactly the
-- two primary keys, the two mutually exclusive partial unique idempotency
-- indexes on metric_import, and the single design-required operational index
-- on metric_import(status, created_at). No other speculative secondary index
-- is created; later bounded slices must derive indexes from their concrete
-- access patterns.

CREATE TYPE public.metric_import_status AS ENUM (
    'UPLOADED',
    'QUEUED',
    'PROCESSING',
    'COMPLETED',
    'COMPLETED_WITH_ERRORS',
    'FAILED'
);

CREATE TYPE public.metric_import_error_severity AS ENUM (
    'ERROR',
    'WARNING'
);

-- metric_import has exactly two foreign keys. created_by stays plain non-null
-- text: this slice assigns it no account foreign key, identity-provider
-- binding, UUID requirement, actor namespace or format rule.
--
-- The foreign keys are deliberately non-cascading: deleting a referenced
-- source account or publisher that still has an import fails instead of
-- silently deleting durable import/audit evidence.
--
-- raw_object_key, raw_sha256, upstream_report_id, the import period and
-- completed_at stay nullable exactly as designed. No raw-SHA encoding or
-- length rule, object-key namespace, upload-bucket rule or source-specific
-- manifest schema is invented here.
--
-- The import period deliberately carries no ordering constraint: the
-- approved design places that constraint on metric_record, so malformed
-- source/report period evidence must remain representable at the
-- import/error layer. Likewise no relationship among the six summary
-- counters is constrained; counter mutation and finality semantics belong to
-- later ingestion work. The non-negative counter and non-blank identifier
-- checks are the approved bounded decisions that such values cannot
-- represent a valid count or identifier and must fail at the canonical
-- database boundary.
CREATE TABLE public.metric_import (
    import_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    source_account_id uuid NOT NULL,
    publisher_id uuid,
    format_code text NOT NULL,
    format_version text NOT NULL,
    raw_object_key text,
    raw_sha256 text,
    upstream_report_id text,
    period_start date,
    period_end date,
    status public.metric_import_status NOT NULL,
    received_count bigint DEFAULT 0 NOT NULL,
    accepted_count bigint DEFAULT 0 NOT NULL,
    duplicate_count bigint DEFAULT 0 NOT NULL,
    revision_count bigint DEFAULT 0 NOT NULL,
    conflict_count bigint DEFAULT 0 NOT NULL,
    invalid_count bigint DEFAULT 0 NOT NULL,
    normalizer_version text NOT NULL,
    manifest jsonb DEFAULT '{}'::jsonb NOT NULL,
    created_by text NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    completed_at timestamp with time zone,
    CONSTRAINT metric_import_pkey PRIMARY KEY (import_id),
    CONSTRAINT metric_import_format_code_check CHECK (format_code ~ '[^[:space:]]'),
    CONSTRAINT metric_import_format_version_check CHECK (format_version ~ '[^[:space:]]'),
    CONSTRAINT metric_import_normalizer_version_check
        CHECK (normalizer_version ~ '[^[:space:]]'),
    CONSTRAINT metric_import_created_by_check CHECK (created_by ~ '[^[:space:]]'),
    CONSTRAINT metric_import_received_count_check CHECK (received_count >= 0),
    CONSTRAINT metric_import_accepted_count_check CHECK (accepted_count >= 0),
    CONSTRAINT metric_import_duplicate_count_check CHECK (duplicate_count >= 0),
    CONSTRAINT metric_import_revision_count_check CHECK (revision_count >= 0),
    CONSTRAINT metric_import_conflict_count_check CHECK (conflict_count >= 0),
    CONSTRAINT metric_import_invalid_count_check CHECK (invalid_count >= 0),
    CONSTRAINT metric_import_source_account_id_fkey FOREIGN KEY (source_account_id)
        REFERENCES public.metric_source_account(source_account_id),
    CONSTRAINT metric_import_publisher_id_fkey FOREIGN KEY (publisher_id)
        REFERENCES public.publisher(publisher_id)
);

-- The design-fixed two-path import idempotency contract, expressed as two
-- mutually exclusive partial unique indexes so both source columns stay
-- nullable exactly as designed.
--
-- Path 1: when the source supplies an upstream report identity, the import is
-- unique by (source_account_id, upstream_report_id) alone — a differing
-- format version or raw hash must not create a second logical job for the
-- same upstream report.
--
-- Path 2: otherwise, and only when a raw hash is available, the import is
-- unique by (source_account_id, raw_sha256, format_version).
--
-- The predicates are mutually exclusive on upstream_report_id IS NOT NULL /
-- IS NULL, so exactly one path applies to any row. Deliberately absent: any
-- constraint requiring a newly inserted row to already carry either column.
-- The approved model permits both to be NULL, and the later upload/claim APIs
-- own the rule for when sufficient idempotency evidence is required before
-- queueing or processing. The runtime "return the existing import" behaviour
-- is likewise not implemented here.
CREATE UNIQUE INDEX metric_import_source_account_id_upstream_report_id_idx
    ON public.metric_import (source_account_id, upstream_report_id)
    WHERE upstream_report_id IS NOT NULL;

CREATE UNIQUE INDEX metric_import_source_account_id_raw_sha256_format_version_idx
    ON public.metric_import (source_account_id, raw_sha256, format_version)
    WHERE upstream_report_id IS NULL AND raw_sha256 IS NOT NULL;

-- The single design-required operational index: later bounded queue claiming
-- and monitoring scan by lifecycle status and creation time.
CREATE INDEX metric_import_status_created_at_idx
    ON public.metric_import (status, created_at);

-- metric_import_error keeps every rejected or flagged row explicit and
-- diagnosable: per thoth-api/AGENTS.md no import row may disappear silently.
--
-- The foreign key to the owning import is deliberately non-cascading so
-- durable row-level evidence cannot be erased through parent deletion.
--
-- row_number, field_name and raw_value stay nullable exactly as designed: a
-- finding need not belong to one numbered row or one named field.
-- row_number deliberately carries no sign, lower-bound, upper-bound or origin
-- constraint — whether rows are counted from zero, from one, or after a
-- header belongs to the later per-format normalizer contract. No
-- source-specific error-code registry, downloadable error-file format,
-- localization model or UI representation is invented here.
CREATE TABLE public.metric_import_error (
    import_error_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    import_id uuid NOT NULL,
    row_number bigint,
    error_code text NOT NULL,
    severity public.metric_import_error_severity NOT NULL,
    field_name text,
    message text NOT NULL,
    raw_value text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT metric_import_error_pkey PRIMARY KEY (import_error_id),
    CONSTRAINT metric_import_error_error_code_check CHECK (error_code ~ '[^[:space:]]'),
    CONSTRAINT metric_import_error_message_check CHECK (message ~ '[^[:space:]]'),
    CONSTRAINT metric_import_error_import_id_fkey FOREIGN KEY (import_id)
        REFERENCES public.metric_import(import_id)
);
