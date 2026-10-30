-- MET-WP1-04: canonical Metrics record, revision and provenance foundation
-- (issue #872).
--
-- Additive and initially inactive. Creates the closed record-revision status
-- and provenance-classification enums and the three canonical history tables:
-- metric_record (the stable identity of one reporting cell),
-- metric_record_revision (immutable value revisions for one record) and
-- metric_record_provenance (durable evidence describing how every normalized
-- source row relates to canonical state). No row is seeded. No existing table
-- or row is touched.
--
-- This slice stores canonical structure only. There is deliberately no
-- identity or content hashing, no normalized-observation validation, no
-- DOI/ISBN/ROR resolution, no first-arrival, duplicate, revision, conflict,
-- rejected-row or retraction transaction, no managed-source revision
-- authorization, no publisher finality, no rollup delta and no period-overlap
-- detection or concurrency protocol: those belong to the later bounded WP2
-- ingestion work.
--
-- Overlap decision (reviewed): period-overlap enforcement is explicitly
-- deferred to WP2. This migration therefore introduces no GiST exclusion
-- constraint, no btree_gist extension and no advisory-lock protocol. Only
-- exact-identity uniqueness and period ordering are enforced here.
--
-- Country decision (reviewed): Metrics country storage is a separate nullable
-- CHAR(2) representation enforcing shape only (exactly two uppercase ASCII
-- letters). The existing bibliographic alpha-3 country_code enum is neither
-- modified, aliased nor reused, and full ISO 3166-1 alpha-2 membership
-- validation belongs to later WP2 normalized-observation validation.
--
-- Hash decision (reviewed): identity_hash and content_hash reject blank and
-- whitespace-only values but carry no algorithm, encoding or length rule. The
-- hashing algorithm itself is WP2 work.
--
-- Index decision (reviewed): the complete intended index set is exactly the
-- three primary keys, metric_record(identity_hash) UNIQUE, the four
-- design-required metric_record access indexes on work_id, platform_id,
-- measure_id and period_start, metric_record_revision(record_id,
-- revision_number) UNIQUE, the metric_record_revision(record_id,
-- record_revision_id) UNIQUE key that carries same-record referential
-- integrity, the partial unique current-revision index, and the three
-- metric_record_provenance audit indexes on import_id, record_id and
-- identity_hash. No speculative dashboard composite over publisher, imprint
-- or series is created: current attribution is derived from live Thoth
-- metadata by later rollup/query design, not stored on the canonical record.

CREATE TYPE public.metric_record_revision_status AS ENUM (
    'CURRENT',
    'SUPERSEDED',
    'RETRACTED'
);

CREATE TYPE public.metric_record_provenance_classification AS ENUM (
    'WINNER',
    'DUPLICATE',
    'REVISION',
    'CONFLICT',
    'REJECTED'
);

-- metric_record is the stable identity of one canonical reporting cell:
-- platform, measure, work, optional publication, half-open period and
-- optional country/institution dimensions, plus the winning acquisition
-- account. Canonical identity deliberately excludes the acquisition route and
-- the value itself, which is why identity_hash rather than a natural
-- composite key carries uniqueness.
--
-- current_revision_id is created nullable and without its foreign key here.
-- The approved design is intentionally circular — a revision references its
-- record and a record points at its current revision — so the constraint is
-- added after metric_record_revision exists. A record row can therefore be
-- created before its first revision inside a later WP2 transaction.
--
-- All foreign keys are deliberately non-cascading: deleting a referenced
-- work, publication, platform, measure, institution or source account that
-- still has canonical records fails instead of silently deleting canonical
-- history.
--
-- reporting_grain reuses the existing MET-WP1-01 metric_reporting_grain enum;
-- no duplicate grain enum is created.
--
-- Deliberately absent: publisher_id, imprint_id and series_id. Current
-- publisher, imprint and series attribution is derived from live Thoth
-- metadata and is not stored authoritatively on the canonical record. Also
-- deliberately absent: any constraint asserting that an optional publication
-- belongs to work_id or that an optional institution carries a ROR. Those are
-- semantic resolutions performed by the later ingestion path against
-- canonical Thoth entities, not database-representable invariants at this
-- foundation stage.
CREATE TABLE public.metric_record (
    record_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    identity_hash text NOT NULL,
    work_id uuid NOT NULL,
    publication_id uuid,
    platform_id uuid NOT NULL,
    measure_id uuid NOT NULL,
    period_start date NOT NULL,
    period_end date NOT NULL,
    reporting_grain public.metric_reporting_grain NOT NULL,
    country_code character(2),
    institution_id uuid,
    winning_source_account_id uuid NOT NULL,
    current_revision_id uuid,
    first_received_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT metric_record_pkey PRIMARY KEY (record_id),
    CONSTRAINT metric_record_identity_hash_key UNIQUE (identity_hash),
    CONSTRAINT metric_record_identity_hash_check CHECK (identity_hash ~ '[^[:space:]]'),
    -- Periods are half-open calendar-date ranges: period_start inclusive,
    -- period_end exclusive. Ordering is enforced; overlap is not.
    CONSTRAINT metric_record_period_check CHECK (period_end > period_start),
    -- Shape only: exactly two uppercase ASCII letters when supplied. NULL
    -- stays valid because country is an optional dimension.
    CONSTRAINT metric_record_country_code_check CHECK (country_code ~ '^[A-Z]{2}$'),
    CONSTRAINT metric_record_work_id_fkey FOREIGN KEY (work_id)
        REFERENCES public.work(work_id),
    CONSTRAINT metric_record_publication_id_fkey FOREIGN KEY (publication_id)
        REFERENCES public.publication(publication_id),
    CONSTRAINT metric_record_platform_id_fkey FOREIGN KEY (platform_id)
        REFERENCES public.metric_platform(platform_id),
    CONSTRAINT metric_record_measure_id_fkey FOREIGN KEY (measure_id)
        REFERENCES public.metric_measure(measure_id),
    CONSTRAINT metric_record_institution_id_fkey FOREIGN KEY (institution_id)
        REFERENCES public.institution(institution_id),
    CONSTRAINT metric_record_winning_source_account_id_fkey
        FOREIGN KEY (winning_source_account_id)
        REFERENCES public.metric_source_account(source_account_id)
);

SELECT diesel_manage_updated_at('public.metric_record');

-- metric_record_revision holds immutable value revisions for one canonical
-- record. Canonical history is preserved by appending revisions rather than
-- by destructive replacement.
--
-- value stays signed BIGINT. No blanket value >= 0 constraint is added:
-- usage measures reject negatives while sales measures may report signed net
-- units, so measure-specific validation belongs to WP2 ingestion against
-- metric_measure.allow_negative rather than to a schema-wide rule.
--
-- The foreign keys to metric_record and MET-WP1-03 metric_import are
-- deliberately non-cascading, so durable canonical history cannot be erased
-- through parent deletion.
--
-- Same-record integrity (reviewed): supersedes_revision_id must never name a
-- revision owned by another record. That is enforced declaratively by a
-- self-referential composite foreign key over (record_id,
-- supersedes_revision_id), supported by the
-- (record_id, record_revision_id) unique key below. Under MATCH SIMPLE the
-- composite key is not enforced while supersedes_revision_id is NULL, so an
-- initial revision remains insertable. No trigger, stored procedure or new
-- dependency is used.
--
-- Deliberately absent: any constraint or trigger tying revision status to
-- metric_record.current_revision_id. WP2 owns the transaction that inserts a
-- revision, supersedes its predecessor and moves the record pointer
-- atomically; this slice must not hide that state machine in the schema.
CREATE TABLE public.metric_record_revision (
    record_revision_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    record_id uuid NOT NULL,
    revision_number integer NOT NULL,
    import_id uuid NOT NULL,
    value bigint NOT NULL,
    content_hash text NOT NULL,
    status public.metric_record_revision_status NOT NULL,
    supersedes_revision_id uuid,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT metric_record_revision_pkey PRIMARY KEY (record_revision_id),
    CONSTRAINT metric_record_revision_record_id_revision_number_key
        UNIQUE (record_id, revision_number),
    -- The supporting unique key that lets both same-record composite foreign
    -- keys reference a revision together with its owning record.
    CONSTRAINT metric_record_revision_record_id_record_revision_id_key
        UNIQUE (record_id, record_revision_id),
    CONSTRAINT metric_record_revision_revision_number_check CHECK (revision_number > 0),
    CONSTRAINT metric_record_revision_content_hash_check CHECK (content_hash ~ '[^[:space:]]'),
    CONSTRAINT metric_record_revision_record_id_fkey FOREIGN KEY (record_id)
        REFERENCES public.metric_record(record_id),
    CONSTRAINT metric_record_revision_import_id_fkey FOREIGN KEY (import_id)
        REFERENCES public.metric_import(import_id),
    CONSTRAINT metric_record_revision_supersedes_revision_id_fkey
        FOREIGN KEY (record_id, supersedes_revision_id)
        REFERENCES public.metric_record_revision (record_id, record_revision_id)
);

-- At most one CURRENT revision per record. The partial unique index is the
-- precise PostgreSQL mechanism for this: SUPERSEDED and RETRACTED revisions
-- stay unconstrained, so full history is retained.
CREATE UNIQUE INDEX metric_record_revision_record_id_current_idx
    ON public.metric_record_revision (record_id)
    WHERE status = 'CURRENT';

-- The circular half of the approved design, added only now that
-- metric_record_revision exists. The composite shape over
-- (record_id, current_revision_id) is what prevents a record from naming a
-- revision owned by a different record; under MATCH SIMPLE it is not enforced
-- while current_revision_id is NULL, so a record still needs no revision at
-- creation time. The down migration drops this constraint explicitly before
-- dropping either table.
ALTER TABLE public.metric_record
    ADD CONSTRAINT metric_record_current_revision_id_fkey
        FOREIGN KEY (record_id, current_revision_id)
        REFERENCES public.metric_record_revision (record_id, record_revision_id);

-- The four design-required metric_record access indexes. Each is the smallest
-- useful shape at this foundation stage; later bounded query/rollup slices
-- must derive any composite from their concrete access patterns.
CREATE INDEX metric_record_work_id_idx ON public.metric_record (work_id);

CREATE INDEX metric_record_platform_id_idx ON public.metric_record (platform_id);

CREATE INDEX metric_record_measure_id_idx ON public.metric_record (measure_id);

CREATE INDEX metric_record_period_start_idx ON public.metric_record (period_start);

-- metric_record_provenance keeps every normalized source row explicit and
-- auditable: per thoth-api/AGENTS.md no import row or state transition may
-- disappear silently.
--
-- record_id is deliberately nullable. Rejected and conflicting rows require
-- durable evidence without a canonical record link, so provenance must be
-- recordable before — or without — any canonical record existing. Both
-- foreign keys are non-cascading.
--
-- source_record_id and source_row_number stay nullable and carry no origin
-- convention: whether rows are counted from zero, from one, or after a header
-- belongs to the later per-format normalizer contract. details is generic
-- non-null JSONB defaulting to an empty object; no source-specific schema is
-- imposed. This slice stores the classification but does not implement the
-- algorithm that assigns it.
CREATE TABLE public.metric_record_provenance (
    record_provenance_id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    record_id uuid,
    import_id uuid NOT NULL,
    source_record_id text,
    source_row_number bigint,
    identity_hash text NOT NULL,
    content_hash text NOT NULL,
    classification public.metric_record_provenance_classification NOT NULL,
    details jsonb DEFAULT '{}'::jsonb NOT NULL,
    received_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT metric_record_provenance_pkey PRIMARY KEY (record_provenance_id),
    CONSTRAINT metric_record_provenance_identity_hash_check
        CHECK (identity_hash ~ '[^[:space:]]'),
    CONSTRAINT metric_record_provenance_content_hash_check
        CHECK (content_hash ~ '[^[:space:]]'),
    CONSTRAINT metric_record_provenance_record_id_fkey FOREIGN KEY (record_id)
        REFERENCES public.metric_record(record_id),
    CONSTRAINT metric_record_provenance_import_id_fkey FOREIGN KEY (import_id)
        REFERENCES public.metric_import(import_id)
);

CREATE INDEX metric_record_provenance_import_id_idx
    ON public.metric_record_provenance (import_id);

CREATE INDEX metric_record_provenance_record_id_idx
    ON public.metric_record_provenance (record_id);

CREATE INDEX metric_record_provenance_identity_hash_idx
    ON public.metric_record_provenance (identity_hash);
