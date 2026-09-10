-- MET-WP2-01A: canonical ingestion contract and schema closure (issue #899).
--
-- Additive and initially inactive. This migration closes only the
-- persistence/model prerequisites the repository-internal MET-WP2-01B
-- ingestion coordinator needs. It implements no ingestion behaviour: no
-- normalization, no identity/content hashing, no first-arrival, duplicate,
-- revision or conflict transaction, no overlap detection, no counter
-- mutation, no coverage or rollup consequence and no GraphQL surface. It
-- seeds no row.
--
-- Four bounded changes, each specified by Specification Amendment 1:
--
--   A. metric_source_account gains the globally unique stable code required
--      by thoth-normalized-metrics/1, introduced safely on an already
--      populated table.
--   B. metric_import_batch records durable bounded-batch identity under an
--      existing metric_import.
--   C. metric_record_provenance gains ordered linkage to that batch, so one
--      committed batch's per-row outcomes can be replayed deterministically.
--      metric_record_provenance remains the single authoritative per-row
--      classification/evidence store; no second result table is created.
--   D. metric_record gains exactly one overlap-supporting lookup index.


-- A. Stable source-account code.
--
-- The final contract is `code TEXT NOT NULL UNIQUE`. The table may already
-- hold rows, so the column cannot simply be created NOT NULL. The approved
-- additive transition is add-nullable, deterministically backfill, constrain,
-- then tighten, in exactly that order.
--
-- The backfill value `source_account_id::text` is migration identity for rows
-- that predate this stable-code contract and nothing more. It does not
-- redefine external_key, does not redefine source identity, and does not
-- prescribe how a real stable code is later chosen. Rows created after this
-- migration must supply their own explicit code.
--
-- The existing UNIQUE (source_id, external_key) source-scoped identity is
-- deliberately untouched: the new code is global stable identity alongside
-- it, not a replacement for it.
--
-- Stable code matching is exact PostgreSQL TEXT identity. Nothing here or
-- later may trim, case-fold, Unicode-normalize or alias it; the only
-- constraint is the Metrics-standard nonblank rule shared with
-- metric_platform.code, metric_measure.code and metric_source.code.

ALTER TABLE public.metric_source_account
    ADD COLUMN code text;

UPDATE public.metric_source_account
   SET code = source_account_id::text
 WHERE code IS NULL;

ALTER TABLE public.metric_source_account
    ADD CONSTRAINT metric_source_account_code_check
        CHECK (code ~ '[^[:space:]]');

ALTER TABLE public.metric_source_account
    ADD CONSTRAINT metric_source_account_code_key UNIQUE (code);

ALTER TABLE public.metric_source_account
    ALTER COLUMN code SET NOT NULL;


-- B. Durable bounded-batch identity.
--
-- One row is one bounded ingestion batch attempt under an existing import.
-- MET-WP2-01B later reads it to return an already committed batch result
-- without repeating canonical, provenance, counter or delta writes.
--
-- The shape is deliberately minimal. There is no status, no completion
-- timestamp, no per-classification counter, no opaque result JSON and no
-- classification column: the authoritative per-row outcome already lives in
-- metric_record_provenance, and duplicating it here would create a second
-- competing classification store that could disagree with it.
--
-- request_hash is stored as opaque nonblank text. This task deliberately does
-- NOT define the canonicalization algorithm that produces it; that decision
-- belongs to MET-WP2-01B and must not be guessed here.
--
-- The redundant-looking UNIQUE (import_id, import_batch_id) exists to give
-- metric_record_provenance a composite foreign-key target, so a provenance
-- row cannot reference a batch belonging to a different import. PostgreSQL
-- requires a unique constraint on exactly the referenced column pair.
--
-- The foreign key is non-cascading: deleting an import must not silently
-- destroy the batch evidence recorded under it.

CREATE TABLE public.metric_import_batch (
    import_batch_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    import_id uuid NOT NULL,
    batch_key text NOT NULL,
    request_hash text NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT metric_import_batch_pkey PRIMARY KEY (import_batch_id),
    CONSTRAINT metric_import_batch_import_id_batch_key_key
        UNIQUE (import_id, batch_key),
    CONSTRAINT metric_import_batch_import_id_import_batch_id_key
        UNIQUE (import_id, import_batch_id),
    CONSTRAINT metric_import_batch_batch_key_check
        CHECK (batch_key ~ '[^[:space:]]'),
    CONSTRAINT metric_import_batch_request_hash_check
        CHECK (request_hash ~ '[^[:space:]]'),
    CONSTRAINT metric_import_batch_import_id_fkey FOREIGN KEY (import_id)
        REFERENCES public.metric_import(import_id)
);


-- C. Provenance-linked ordered batch results, and the REJECTED hash invariant.
--
-- C.1 Hash nullability.
--
-- A row can be refused before a canonical identity can be formed, or after
-- identity but before valid canonical content can be formed, so REJECTED
-- provenance must be recordable without one or both hashes. Every other
-- classification still describes a row that reached canonical comparison and
-- therefore still requires both.
--
-- Dropping NOT NULL does not weaken the existing nonblank rules: a CHECK that
-- evaluates to NULL is satisfied, so metric_record_provenance_identity_hash_check
-- and metric_record_provenance_content_hash_check continue to reject blank
-- values exactly as before whenever a hash is actually present.

ALTER TABLE public.metric_record_provenance
    ALTER COLUMN identity_hash DROP NOT NULL,
    ALTER COLUMN content_hash DROP NOT NULL;

-- The exact approved truth table. content_hash without identity_hash is
-- nonsensical under every classification, including REJECTED: content is
-- only meaningful for a row whose canonical identity was already formed.
ALTER TABLE public.metric_record_provenance
    ADD CONSTRAINT metric_record_provenance_classification_hash_check CHECK (
        CASE
            WHEN classification = 'REJECTED'
                THEN NOT (identity_hash IS NULL AND content_hash IS NOT NULL)
            ELSE identity_hash IS NOT NULL AND content_hash IS NOT NULL
        END
    );

-- C.2 Ordered batch linkage.
--
-- Both columns are nullable together so every pre-WP2 provenance row stays
-- valid unchanged. MET-WP2-01B owns the runtime rule that every row it
-- processes populates both.

ALTER TABLE public.metric_record_provenance
    ADD COLUMN import_batch_id uuid,
    ADD COLUMN batch_row_index bigint;

-- The two batch fields are meaningless apart: an index without a batch has no
-- ordering context, and a batch link without an index cannot be replayed
-- deterministically.
ALTER TABLE public.metric_record_provenance
    ADD CONSTRAINT metric_record_provenance_batch_link_check
        CHECK ((import_batch_id IS NULL) = (batch_row_index IS NULL));

-- Row indices are non-negative. Unlike source_row_number, which deliberately
-- carries no origin convention because it mirrors whatever the upstream
-- format counted, batch_row_index is Thoth's own replay ordering and is
-- therefore fixed as zero-based-or-greater.
ALTER TABLE public.metric_record_provenance
    ADD CONSTRAINT metric_record_provenance_batch_row_index_check
        CHECK (batch_row_index >= 0);

-- One position in one batch holds exactly one provenance row, which is what
-- makes ordered replay deterministic. NULLs compare as distinct in a
-- PostgreSQL unique constraint, so unlinked pre-WP2 rows are unaffected and
-- two different batches may reuse the same index.
ALTER TABLE public.metric_record_provenance
    ADD CONSTRAINT metric_record_provenance_import_batch_id_batch_row_index_key
        UNIQUE (import_batch_id, batch_row_index);

-- The composite foreign key carries import_id deliberately: it makes it
-- impossible for provenance recorded under one import to reference a batch
-- committed under another. Default MATCH SIMPLE semantics mean the constraint
-- is not enforced while import_batch_id is NULL, which is exactly what keeps
-- existing unlinked provenance rows valid.
--
-- Non-cascading, like every other Metrics evidence foreign key.
ALTER TABLE public.metric_record_provenance
    ADD CONSTRAINT metric_record_provenance_import_batch_fkey
        FOREIGN KEY (import_id, import_batch_id)
        REFERENCES public.metric_import_batch(import_id, import_batch_id);


-- D. Overlap-supporting lookup index.
--
-- Exactly one index, supporting the MET-WP2-01B same-dimensional-cell
-- half-open overlap lookup:
--
--     platform + measure + work + publication/NULL + country/NULL
--       + institution/NULL
--     AND existing.period_start < incoming.period_end
--     AND existing.period_end   > incoming.period_start
--
-- The six cell dimensions lead in equality/IS NULL position, and period_start
-- trails as the only range-scannable half of the predicate. period_end stays
-- a residual filter by design: a btree cannot serve both half-open bounds,
-- and adding it would widen the index without removing the residual.
--
-- Serialization of concurrent same-cell writers is MET-WP2-01B's
-- transaction-scoped advisory cell lock, not this index. 01A deliberately
-- introduces no exclusion constraint, no btree_gist extension, no trigger and
-- no advisory-lock runtime.

CREATE INDEX metric_record_overlap_lookup_idx ON public.metric_record (
    platform_id,
    measure_id,
    work_id,
    publication_id,
    country_code,
    institution_id,
    period_start
);
