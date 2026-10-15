-- MET-WP7-PREREQ-03: automatic unresolved-DOI quarantine reconciliation (#935).
--
-- Additive only. Historical quarantine/provenance/import evidence is never
-- rewritten. This table records one server-owned reconciliation state machine
-- per existing quarantine row. Absence of a row means "never attempted".
--
-- Resolved outcomes are terminal and identify the canonical record/revision
-- that explains the result. Pending/blocked outcomes are retryable and carry
-- the next server-owned attempt time. No backfill is performed.

CREATE TYPE public.metric_identifier_quarantine_reconciliation_state AS ENUM (
    'PENDING_UNKNOWN_DOI',
    'BLOCKED_AMBIGUOUS_DOI',
    'BLOCKED_PUBLISHER_SCOPE_MISMATCH',
    'BLOCKED_SOURCE_CONFLICT',
    'BLOCKED_OVERLAPPING_PERIOD',
    'BLOCKED_SAME_IMPORT_ORDER',
    'BLOCKED_IMPORT_ORDER_AMBIGUOUS',
    'BLOCKED_DELTA_OVERFLOW',
    'BLOCKED_INCONSISTENT_EVIDENCE',
    'RESOLVED_WINNER',
    'RESOLVED_DUPLICATE',
    'RESOLVED_REVISION',
    'RESOLVED_SUPERSEDED'
);

CREATE TABLE public.metric_identifier_quarantine_reconciliation (
    identifier_quarantine_id uuid NOT NULL,
    state public.metric_identifier_quarantine_reconciliation_state NOT NULL,
    attempt_count integer NOT NULL,
    last_attempted_by text NOT NULL,
    first_attempt_at timestamp with time zone NOT NULL,
    last_attempt_at timestamp with time zone NOT NULL,
    next_attempt_at timestamp with time zone,
    resolved_at timestamp with time zone,
    record_id uuid,
    record_revision_id uuid,
    CONSTRAINT metric_identifier_quarantine_reconciliation_pkey
        PRIMARY KEY (identifier_quarantine_id),
    CONSTRAINT metric_iqr_attempt_count_check CHECK (attempt_count > 0),
    CONSTRAINT metric_iqr_actor_check CHECK (
        last_attempted_by ~ '[^\u0009\u000A\u000B\u000C\u000D\u0020\u0085\u00A0\u1680\u2000\u2001\u2002\u2003\u2004\u2005\u2006\u2007\u2008\u2009\u200A\u2028\u2029\u202F\u205F\u3000]'
    ),
    CONSTRAINT metric_iqr_attempt_time_check CHECK (
        first_attempt_at <= last_attempt_at
        AND (next_attempt_at IS NULL OR next_attempt_at > last_attempt_at)
    ),
    CONSTRAINT metric_iqr_state_shape_check CHECK (
        (
            state IN (
                'RESOLVED_WINNER',
                'RESOLVED_DUPLICATE',
                'RESOLVED_REVISION',
                'RESOLVED_SUPERSEDED'
            )
            AND resolved_at IS NOT NULL
            AND next_attempt_at IS NULL
            AND record_id IS NOT NULL
            AND record_revision_id IS NOT NULL
        )
        OR
        (
            state NOT IN (
                'RESOLVED_WINNER',
                'RESOLVED_DUPLICATE',
                'RESOLVED_REVISION',
                'RESOLVED_SUPERSEDED'
            )
            AND resolved_at IS NULL
            AND next_attempt_at IS NOT NULL
            AND record_id IS NULL
            AND record_revision_id IS NULL
        )
    ),
    CONSTRAINT metric_iqr_quarantine_fkey
        FOREIGN KEY (identifier_quarantine_id)
        REFERENCES public.metric_identifier_quarantine(identifier_quarantine_id),
    CONSTRAINT metric_iqr_record_fkey
        FOREIGN KEY (record_id)
        REFERENCES public.metric_record(record_id),
    CONSTRAINT metric_iqr_revision_record_fkey
        FOREIGN KEY (record_id, record_revision_id)
        REFERENCES public.metric_record_revision(record_id, record_revision_id)
);

CREATE INDEX metric_identifier_quarantine_created_id_idx
    ON public.metric_identifier_quarantine (created_at, identifier_quarantine_id);

CREATE INDEX metric_iqr_due_idx
    ON public.metric_identifier_quarantine_reconciliation (next_attempt_at, identifier_quarantine_id)
    WHERE resolved_at IS NULL;
