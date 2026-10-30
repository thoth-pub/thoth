-- MET-WP1-07: rollup-delta persistence foundation (issue #880).
--
-- Additive and initially inactive. Creates the single `metric_rollup_delta`
-- table: the durable accounting bridge between a canonical metric-record
-- revision and the rebuildable work-level rollup projections. The approved
-- Metrics design commits a canonical revision transactionally together with
-- its durable delta — applying a new record adds its value, a revision
-- contributes `new - old`, and a retraction subtracts the old value — so the
-- delta is canonical accounting evidence rather than a derived cache.
--
-- This slice stores delta structure only. There is deliberately no delta
-- generation from ingestion, no first-arrival/revision/retraction transaction,
-- no claiming, lease, retry, backoff or stale-claim recovery, no
-- `FOR UPDATE SKIP LOCKED` application loop, no delta application, no rebuild
-- generation and no active-watermark behaviour: those belong to the later
-- bounded WP4 rollup work. The module exposes no GraphQL, authorization or
-- administration surface.
--
-- Projection boundary (reviewed): the four rebuildable work-level rollup
-- tables named by the approved design — metric_rollup_work_day,
-- metric_rollup_work_month, metric_rollup_work_country_month and
-- metric_rollup_work_institution_month — remain approved future architecture
-- and are deliberately NOT created here. The design fixes their conceptual
-- dimensions but not their relational keys, null-dimension uniqueness,
-- watermark representation or rebuild-generation protocol, and this
-- persistence-only slice must not invent them.
--
-- Status decision (reviewed): `status` is required TEXT. The approved design
-- names the field but defines no closed status vocabulary, transition model,
-- claim ownership, lease or recovery protocol. No PostgreSQL enum is created
-- and no CHECK enumerates values, because doing so would pre-decide the later
-- WP4 claim/application state machine. For the same reason no trigger or
-- stored procedure changes `status`, and no cross-column CHECK ties
-- `applied_at` to any particular status value: `applied_at` is simply a
-- nullable timestamp that later work will populate when a delta has been
-- applied.
--
-- Signed-value decision (reviewed): `delta_value` is a signed BIGINT with
-- deliberately NO non-negative CHECK. Positive, zero and negative values are
-- all valid: a revision contributes the signed difference `new - old` and a
-- retraction subtracts the previously applied value, so a blanket
-- `delta_value >= 0` rule would make correction and retraction accounting
-- unrepresentable.
--
-- Referential-identity decision (reviewed): a delta names both the canonical
-- record and the canonical revision it accounts for, and those two must
-- describe the same canonical pair. That is enforced declaratively by a
-- composite foreign key over `(record_id, revision_id)` against the existing
-- MET-WP1-04 `metric_record_revision (record_id, record_revision_id)` unique
-- key, exactly as MET-WP1-04 itself enforces its same-record revision
-- pointers. A delta therefore cannot pair one record with a revision owned by
-- another record, and no trigger, redundant identity column or new dependency
-- is introduced. Both columns are NOT NULL, so unlike the MET-WP1-04 nullable
-- pointers the MATCH SIMPLE composite key is always enforced here.
--
-- The foreign key is deliberately non-cascading, matching every other Metrics
-- foreign key: deleting a canonical record or revision that still has a
-- durable delta must fail rather than silently erase accounting evidence that
-- a rollup projection may already have consumed.
--
-- Uniqueness decision (reviewed): `UNIQUE(revision_id)` permits at most one
-- durable rollup delta per canonical revision, which is what prevents later
-- double counting from duplicate delta rows. It is deliberately keyed on
-- `revision_id` alone rather than on `(record_id, revision_id)`, because
-- `record_id` is already functionally determined by the revision through the
-- composite foreign key above, and a pair-scoped key would not actually
-- exclude a second delta for the same revision.
--
-- Constraint-naming decision (reviewed): the primary key, the uniqueness key
-- and the composite foreign key all use the PostgreSQL default naming shape
-- `<table>_<referencing columns>_<pkey|key|fkey>`, matching the merged
-- MET-WP1-04 supporting key `metric_record_revision_record_id_record_revision_id_key`.
--
-- Index decision (reviewed): the complete intended index set is exactly the
-- primary-key index on `delta_id` and the index PostgreSQL creates to enforce
-- `UNIQUE(revision_id)`. No claim/retry operational index such as
-- `(status, created_at)` is created: PostgreSQL builds no index for the
-- referencing side of a foreign key, and an access path must not be encoded
-- before the WP4 claim query and its query-plan evidence are approved.
--
-- No delta row is seeded, and no existing table, row, enum, index or
-- constraint is modified.

CREATE TABLE public.metric_rollup_delta (
    delta_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    record_id uuid NOT NULL,
    revision_id uuid NOT NULL,
    delta_value bigint NOT NULL,
    status text NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    applied_at timestamp with time zone,
    CONSTRAINT metric_rollup_delta_pkey PRIMARY KEY (delta_id),
    CONSTRAINT metric_rollup_delta_revision_id_key UNIQUE (revision_id),
    CONSTRAINT metric_rollup_delta_record_id_revision_id_fkey
        FOREIGN KEY (record_id, revision_id)
        REFERENCES public.metric_record_revision (record_id, record_revision_id)
);
