-- MET-WP1-09: OPERAS export ledger persistence foundation (issue #884).
--
-- Additive and initially inactive. Creates the single `metric_operas_export`
-- table: the canonical durable record of one outbound OPERAS export for one
-- canonical metric-record revision, using one configured MET-WP1-08 OPERAS
-- mapping. The approved Metrics design assigns the OPERAS synchronization
-- ledgers to Thoth, and this row is the durable idempotency and audit boundary
-- against which a later external OPERAS write must be reconciled.
--
-- This slice stores export-ledger structure only. There is deliberately no
-- outbound eligibility, source-period finalization or `METRICS_OPERAS_EXPORT`
-- capability enforcement, no export status vocabulary or transition graph, no
-- claim owner, lease, claim token, retry schedule, backoff, stale-claim
-- recovery protocol or attempt-increment behaviour, no `FOR UPDATE SKIP
-- LOCKED` claim logic, no payload construction, normalization or hashing, no
-- OPERAS network call, no correction/divergence handling, and no inbound
-- synchronization or reconciliation: those belong to the later bounded WP5/WP9
-- work. Nothing creates or claims an export row at runtime, and the module
-- exposes no GraphQL, authorization or administration surface.
--
-- Deferred ledger boundary (reviewed): `metric_operas_import`,
-- `metric_reconciliation_run` and `metric_reconciliation_issue` remain
-- approved future architecture and are deliberately NOT created here.
--
-- RETRY-TIME INCONSISTENCY AND ITS DELIBERATE WP9 DEFERRAL (reviewed).
-- The approved Metrics design is internally inconsistent on this point and
-- this migration records the boundary rather than hiding it:
--   * section 6.14 defines the export ledger as exactly the ten fields created
--     below and contains NO retry-time / next-attempt timestamp column;
--   * section 14.4 nevertheless calls for OPERAS export indexes on status and
--     retry time;
--   * the design fixes no final export status vocabulary, claim ownership,
--     lease column, retry schedule, backoff or stale-claim recovery protocol.
-- The independent specification review and the CTO specification approval both
-- decided this is a SAFE DELIBERATE DEFERRAL TO WP9. MET-WP1-09 therefore
-- persists only the fields section 6.14 names and must not invent `retry_at`,
-- `next_attempt_at`, `retry_after`, `next_retry`, `lease_until`, `claim_until`
-- or any equivalent retry-time/claim field, and must not create a speculative
-- status, retry, `(status, created_at)`, mapping or claim index. The later
-- bounded WP9 claim/retry specification owns resolving the missing retry-time
-- representation, defining the actual claim query and protocol, and only then
-- adding the concrete operational index with query-plan evidence.
--
-- Identity decision (reviewed): `export_id UUID PRIMARY KEY` is the
-- repository-standard Metrics UUID surrogate identity. The approved design
-- names `export_id` as the export row identity and current Metrics persistence
-- uses UUID surrogate keys for durable domain rows.
--
-- Uniqueness decision (reviewed): `UNIQUE(record_revision_id)` permits at most
-- one durable export row per canonical revision. Outbound eligibility is
-- defined in singular terms — the revision must not already have been exported
-- — and section 15.3 describes creating *an* export row and then retrying or
-- claiming that durable row rather than creating a new row per attempt. The
-- merged mapping registry already permits at most one canonical mapping per
-- platform/measure pair at a time, so permitting a second export row for the
-- same revision merely because a mapping row changed would make
-- duplicate-delivery prevention ambiguous. This uniqueness defines no
-- success/failure transition, no retry eligibility and no reaction to later
-- mapping administration.
--
-- Referential decisions (reviewed): unlike the MET-WP1-07 and MET-WP1-08
-- composite keys, both relationships here are ordinary single-column foreign
-- keys — to the MET-WP1-04 `metric_record_revision (record_revision_id)`
-- primary key and to the MET-WP1-08 `metric_operas_mapping (mapping_id)`
-- surrogate primary key, which MET-WP1-08 introduced precisely as the
-- referential target this ledger implies. Both are deliberately non-cascading,
-- matching every other Metrics foreign key: deleting canonical revision
-- history or OPERAS mapping configuration while durable export evidence exists
-- must fail rather than silently erase the evidence of an external write.
--
-- Mapping-to-revision correspondence (reviewed, deliberately NOT enforced
-- here): a valid export row must ultimately use the mapping for the canonical
-- revision's own platform/measure pair. The design-shaped export row does not
-- duplicate `record_id`, `platform_id` or `measure_id`, so the database cannot
-- express that cross-table rule with a simple foreign key. This slice must NOT
-- add those redundant columns and must NOT add a trigger merely to make the
-- rule declarative. The later export-enqueue/eligibility path must select and
-- validate the mapping from the revision's canonical record and fail closed if
-- it does not correspond. No GraphQL or runtime write path exists in this
-- slice, so an arbitrary pair of individually valid foreign-key identifiers
-- stays representable at raw database level; that is the approved WP1/WP9
-- boundary, not an oversight.
--
-- Status decision (reviewed): `status` is required TEXT carrying only the
-- existing Metrics required-text CHECK, which rejects blank and whitespace-only
-- values. There is deliberately NO PostgreSQL enum, NO CHECK enumerating status
-- values, NO default, NO trigger or stored procedure changing status, and NO
-- cross-column rule tying status to `completed_at`, `remote_event_id`,
-- `request_hash`, `last_error` or `attempt_count`. The final vocabulary and the
-- transition/claim/recovery protocol belong to the later WP9 specification.
--
-- Attempt decision (reviewed): `attempt_count` is required INTEGER with
-- `CHECK (attempt_count >= 0)` and deliberately NO database default. Attempt
-- counts cannot meaningfully be negative; INTEGER is sufficient for an
-- operational attempt counter and avoids implying that import-row volumes and
-- delivery-attempt counts share one domain. No default is invented because
-- this slice defines neither when an attempt starts nor when the counter
-- increments — a later reviewed enqueue path must state the initial value
-- explicitly under its own approved state machine.
--
-- Delivery-result decisions (reviewed): `remote_event_id`, `request_hash` and
-- `last_error` are nullable because the row exists before remote delivery.
-- `remote_event_id` and `request_hash` carry the nullable form of the
-- required-text idiom — NULL, or at least one non-whitespace character — and
-- nothing stronger: no authoritative OPERAS event-ID syntax and no hash
-- algorithm, encoding or length is fixed by the approved design, so none is
-- invented. Neither column is unique: the design establishes no remote-instance
-- namespace proving global event-ID uniqueness, and two different export rows
-- may legitimately produce equal payload content, so revision uniqueness rather
-- than global hash uniqueness owns canonical idempotency. `last_error` carries
-- no format, length, uniqueness or retention semantics; the later runtime path
-- owns bounded diagnostic content and overwrite behaviour.
--
-- Timestamp decisions (reviewed): `created_at` is required with the
-- repository-standard current-time default. `completed_at` is nullable with no
-- default, because the row exists before completion, and with no invented
-- cross-column invariant tying it to `status`.
--
-- Constraint-naming decision (reviewed): the primary key, the uniqueness key,
-- both foreign keys and the four CHECKs all use the PostgreSQL default naming
-- shape `<table>_<columns>_<pkey|key|fkey|check>`, matching the merged
-- MET-WP1-07 `metric_rollup_delta_revision_id_key` and MET-WP1-08
-- `metric_operas_mapping_platform_id_measure_id_fkey` keys.
--
-- Index decision (reviewed): the complete intended index set is exactly the
-- primary-key index on `export_id` and the index PostgreSQL creates to enforce
-- `UNIQUE(record_revision_id)`. PostgreSQL builds no index for the referencing
-- side of a foreign key, and per the retry-time note above no operational
-- access path may be encoded before WP9 approves the claim query and its
-- query-plan evidence.
--
-- No export row is seeded, and no existing table, row, enum, index or
-- constraint is modified. In particular the MET-WP1-01 `metric_measure` seed
-- rows, the MET-WP1-04 canonical record/revision/provenance schema and enums
-- and the MET-WP1-08 `metric_operas_mapping` configuration are untouched, and
-- no real OPERAS mapping is approved or seeded by this slice.

CREATE TABLE public.metric_operas_export (
    export_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    record_revision_id uuid NOT NULL,
    mapping_id uuid NOT NULL,
    status text NOT NULL,
    attempt_count integer NOT NULL,
    remote_event_id text,
    request_hash text,
    last_error text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    completed_at timestamp with time zone,
    CONSTRAINT metric_operas_export_pkey PRIMARY KEY (export_id),
    CONSTRAINT metric_operas_export_record_revision_id_key
        UNIQUE (record_revision_id),
    CONSTRAINT metric_operas_export_record_revision_id_fkey
        FOREIGN KEY (record_revision_id)
        REFERENCES public.metric_record_revision (record_revision_id),
    CONSTRAINT metric_operas_export_mapping_id_fkey
        FOREIGN KEY (mapping_id)
        REFERENCES public.metric_operas_mapping (mapping_id),
    CONSTRAINT metric_operas_export_status_check
        CHECK (status ~ '[^[:space:]]'),
    CONSTRAINT metric_operas_export_attempt_count_check
        CHECK (attempt_count >= 0),
    CONSTRAINT metric_operas_export_remote_event_id_check
        CHECK (remote_event_id IS NULL OR remote_event_id ~ '[^[:space:]]'),
    CONSTRAINT metric_operas_export_request_hash_check
        CHECK (request_hash IS NULL OR request_hash ~ '[^[:space:]]')
);
